"""Propose caption splits from bound recognition estimates, without changing words.

Only boundaries between captions from the same sentence can move. Sentence
edges and prediction holds remain exactly as in the native SRT. Estimates are
not heard speech boundaries and the output still requires audiovisual review.
"""
import argparse
import hashlib
import json
import re
import wave
from pathlib import Path


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def timestamp(value):
    match = re.fullmatch(r'(\d+):(\d\d):(\d\d),(\d{3})', value)
    if not match:
        raise ValueError(f'Bad SRT timestamp: {value}')
    h, m, s, ms = map(int, match.groups())
    if m >= 60 or s >= 60:
        raise ValueError(f'Bad SRT timestamp: {value}')
    return ((h * 60 + m) * 60 + s) * 1000 + ms


def timecode(ms, separator=','):
    if ms < 0:
        raise ValueError('Negative caption time')
    seconds, fraction = divmod(ms, 1000)
    minutes, second = divmod(seconds, 60)
    hour, minute = divmod(minutes, 60)
    return f'{hour:02d}:{minute:02d}:{second:02d}{separator}{fraction:03d}'


def parse_srt(text):
    result = []
    for index, block in enumerate(re.split(r'\n\s*\n', text.strip()), 1):
        lines = block.splitlines()
        if len(lines) < 3 or lines[0] != str(index) or ' --> ' not in lines[1]:
            raise ValueError(f'Bad caption block {index}')
        a, b = lines[1].split(' --> ')
        caption = {'start': timestamp(a), 'end': timestamp(b), 'lines': lines[2:]}
        if not 1 <= len(caption['lines']) <= 2 or any(len(line) > 42 for line in caption['lines']):
            raise ValueError(f'Invalid caption layout {index}')
        result.append(caption)
    validate(result)
    return result


def validate(captions):
    end = 0
    for caption in captions:
        if caption['start'] < end or caption['end'] <= caption['start']:
            raise ValueError('Captions overlap or have zero duration')
        end = caption['end']


def timeline(paths):
    offset = 0.0
    placements = []
    for path in sorted(paths):
        rows = [line.split('\t') for line in path.read_text().splitlines()
                if line and not line.startswith('#')]
        meta = {row[0]: row[1] for row in rows if len(row) == 2}
        fps, frames = int(meta['fps']), int(meta['frames'])
        if fps <= 0 or frames <= 0:
            raise ValueError(f'Invalid timeline {path}')
        for row in rows:
            if row[0] == 'line':
                if len(row) != 5 or not 0 <= int(row[2]) < frames:
                    raise ValueError(f'Invalid placement in {path}')
                placements.append((row[1], offset + int(row[2]) / fps, Path(row[4])))
        offset += frames / fps
    if not placements or len({cue for cue, _, _ in placements}) != len(placements):
        raise ValueError('Missing or duplicate cue placements')
    return placements


def retime(captions, script, report, timing, placements):
    if [cue for cue, _, _ in placements] != list(script):
        raise ValueError('Timeline cue order differs from script')
    rate = report['meta']['rate']
    if not isinstance(rate, int) or rate <= 0:
        raise ValueError('Invalid report rate')
    if set(timing['cues']) != set(script):
        raise ValueError('Incomplete timing cue coverage')
    sentences = []
    for cue, cue_start, wav in placements:
        text = script[cue]
        entry = timing['cues'][cue]
        if entry['text_sha256'] != hashlib.sha256(text.encode()).hexdigest() or entry['wav_sha256'] != sha(wav):
            raise ValueError(f'Changed text or WAV: {cue}')
        with wave.open(str(wav), 'rb') as audio:
            duration = audio.getnframes() / audio.getframerate()
        raw_words = list(re.finditer(r'\S+', text))
        starts = [len(text[:word.start()].encode()) for word in raw_words]
        words = [word.group() for word in raw_words]
        marks = {}
        last_byte, last_time = -1, 0.0
        for mark in entry['marks']:
            byte, second = mark['byte'], mark['seconds']
            if not isinstance(byte, int) or byte <= last_byte or byte >= len(text.encode()) or not last_time <= second < duration:
                raise ValueError(f'Invalid timing mark: {cue}')
            text.encode()[:byte].decode('utf-8')
            marks[byte] = second
            last_byte, last_time = byte, second
        cursor = 0
        sample = 0
        for item in report['lines'][cue]:
            sentence_words = item['sentence'].split()
            if words[cursor:cursor + len(sentence_words)] != sentence_words:
                raise ValueError(f'Report words differ: {cue}')
            count, gap = item['out_samples'], item['gap_after_samples']
            if not isinstance(count, int) or count <= 0 or not isinstance(gap, int) or gap < 0:
                raise ValueError(f'Invalid sentence samples: {cue}')
            sentences.append({'cue': cue, 'words': sentence_words,
                              'word_bytes': starts[cursor:cursor + len(sentence_words)],
                              'start': sample / rate, 'end': (sample + count) / rate,
                              'cue_start': cue_start, 'marks': marks})
            sample += count + gap
            cursor += len(sentence_words)
        if cursor != len(words) or abs(sample / rate - duration) > 1 / rate:
            raise ValueError(f'Incomplete report coverage: {cue}')
    groups = []
    sentence_index = word_index = 0
    for number, caption in enumerate(captions):
        if sentence_index >= len(sentences):
            raise ValueError('More caption words than report words')
        sentence = sentences[sentence_index]
        words = ' '.join(caption['lines']).split()
        if not words or sentence['words'][word_index:word_index + len(words)] != words:
            raise ValueError(f'Caption {number + 1} differs from its sentence')
        groups.append((sentence_index, word_index))
        word_index += len(words)
        if word_index == len(sentence['words']):
            sentence_index += 1
            word_index = 0
    if sentence_index != len(sentences) or word_index:
        raise ValueError('Caption words do not cover the report')
    changed, skipped = [], []
    for index in range(1, len(captions)):
        old_sentence, _ = groups[index - 1]
        sentence_index, first_word = groups[index]
        if old_sentence != sentence_index:
            continue
        sentence = sentences[sentence_index]
        mark = sentence['marks'].get(sentence['word_bytes'][first_word])
        if mark is None or not sentence['start'] < mark < sentence['end']:
            skipped.append({'caption': index + 1, 'cue': sentence['cue'], 'reason': 'missing or out-of-sentence estimate'})
            continue
        target = round((sentence['cue_start'] + mark) * 1000)
        previous, current = captions[index - 1], captions[index]
        if previous['end'] != current['start'] or not previous['start'] + 100 <= target <= current['end'] - 100:
            skipped.append({'caption': index + 1, 'cue': sentence['cue'], 'reason': 'noncontiguous or too-short caption'})
            continue
        original = current['start']
        previous['end'] = current['start'] = target
        changed.append({'caption': index + 1, 'cue': sentence['cue'], 'old_ms': original, 'new_ms': target})
    validate(captions)
    return changed, skipped


def render(captions, vtt=False):
    separator = '.' if vtt else ','
    blocks = []
    for index, caption in enumerate(captions, 1):
        lines = ([] if vtt else [str(index)]) + [
            f"{timecode(caption['start'], separator)} --> {timecode(caption['end'], separator)}",
            *caption['lines'],
        ]
        blocks.append('\n'.join(lines))
    return ('WEBVTT\n\n' if vtt else '') + '\n\n'.join(blocks) + '\n'


def main():
    parser = argparse.ArgumentParser()
    for name in ('script', 'report', 'timings', 'recognition', 'timelines', 'srt', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
    args = parser.parse_args()
    script = dict(row.split('\t', 1) for row in args.script.read_text().splitlines()
                  if '\t' in row and not row.startswith('#'))
    report, timing = json.loads(args.report.read_text()), json.loads(args.timings.read_text())
    if timing['script_sha256'] != sha(args.script):
        raise ValueError('Timing file binds a different script')
    if timing['asr_log_sha256'] != sha(args.recognition):
        raise ValueError('Timing file binds a different recognition log')
    paths = list(args.timelines.glob('*.timeline.tsv'))
    captions = parse_srt(args.srt.read_text())
    changed, skipped = retime(captions, script, report, timing, timeline(paths))
    args.output.mkdir()
    for name, content in [('captions.en.srt', render(captions)), ('captions.en.vtt', render(captions, True))]:
        with (args.output / name).open('x') as stream:
            stream.write(content)
    with (args.output / 'retiming.json').open('x') as stream:
        json.dump({'script_sha256': sha(args.script), 'report_sha256': sha(args.report),
                   'timings_sha256': sha(args.timings), 'recognition_sha256': sha(args.recognition),
                   'srt_sha256': sha(args.srt),
                   'timeline_sha256': {str(path): sha(path) for path in paths},
                   'changed': changed, 'skipped': skipped,
                   'scope': 'Provisional recognition estimates; not listening or audiovisual acceptance.'}, stream, indent=2)
        stream.write('\n')
    print(f'{len(changed)} split boundaries adjusted; {len(skipped)} unchanged')


if __name__ == '__main__':
    main()
