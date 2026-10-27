"""Bind native consumers to coherent narration before output.

This checks source identity and sample reconstruction, never audible quality.
Canonical plans govern production. Explicit lab review is restricted to one
preview chapter in narration_lab; it does not promote or master a candidate.
"""
import argparse
from collections import Counter
import importlib.util
import json
import math
from pathlib import Path
import re
import struct
import sys

REPO = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('coherent', REPO/'tools/coherent_narration.py')
coherent = importlib.util.module_from_spec(spec)
spec.loader.exec_module(coherent)


def require(condition, message):
    if not condition:
        raise ValueError(message)


def lab_path(path, label):
    require(path is not None, f'lab review requires {label}')
    lexical = path.absolute()
    lab = REPO/'narration_lab'
    require('..' not in path.parts and lexical != lab and lexical.is_relative_to(lab),
            f'{label} must be strictly inside narration_lab without traversal')
    resolved, real_lab = path.resolve(), lab.resolve()
    require(real_lab == lab and resolved != real_lab and resolved.is_relative_to(real_lab),
            f'{label} escapes narration_lab through a symlink')
    return path


def review_plan(args):
    require(args.consumer in ('render', 'captions'), 'lab review cannot score or master')
    require(len(args.chapter_key) == 1, 'lab review requires exactly one chapter')
    lab_path(args.output, 'output')
    units = lab_path(args.review_units, 'review units')
    require(args.score is None, 'lab review excludes score stems')
    if args.consumer == 'render':
        require(args.width is not None and args.height is not None and args.fps is not None
                and 0 < args.width <= 1920 and 0 < args.height <= 1080 and 0 < args.fps <= 30,
                'lab render requires positive dimensions at most 1920x1080 and 30 fps')
    else:
        lab_path(args.timelines, 'timelines')
        require(args.timelines.resolve() == args.output.resolve(),
                'lab captions must use the output directory timelines')
    return units


def sha(path):
    return coherent.cache.digest(Path(path))


def rounded(value):
    return math.floor(value + 0.5) if value >= 0 else -math.floor(-value + 0.5)


def sample(frame, fps):
    require(frame >= 0 and fps > 0, 'invalid frame clock')
    return (2 * frame * 48000 + fps) // (2 * fps)


def finite(text):
    value = float(text)
    require(math.isfinite(value), 'nonfinite timing/gain')
    return value


def bits(text):
    require(text.startswith('0x') and len(text) <= 18, 'invalid f64 bits')
    value = struct.unpack('>d', int(text, 16).to_bytes(8, 'big'))[0]
    require(math.isfinite(value), 'nonfinite cue-log value')
    return value


def rows(path):
    return [line.split('\t') for line in path.read_text().splitlines()
            if line.strip() and not line.startswith('#')]


def timeline(path, key, expected_ids, outputs, gain):
    data = rows(path)
    headers = {}
    lines, sounds = [], []
    for fields in data:
        kind = fields[0]
        if kind in ('chapter', 'fps', 'frames'):
            require(len(fields) == 2 and kind not in headers, 'duplicate/invalid timeline header')
            headers[kind] = fields[1]
        elif kind == 'line':
            require(len(fields) == 5, 'invalid timeline line')
            lines.append(fields[1:])
        elif kind == 'listen':
            require(len(fields) in (4, 5), 'invalid listen window')
            if len(fields) == 5 and fields[4]:
                sounds.append(fields[4])
        else:
            require(kind in ('mark', 'breath'), 'unknown timeline record')
    require(headers.get('chapter') == key, 'timeline chapter mismatch')
    fps, frames = int(headers['fps']), int(headers['frames'])
    require(fps > 0 and frames > 0, 'invalid timeline duration')
    require([line[0] for line in lines] == expected_ids, 'timeline cue coverage/order mismatch')
    expected = Counter()
    previous = -1
    for cue, start, level, wav in lines:
        start = int(start)
        require(previous <= start < frames, 'invalid timeline cue placement')
        previous = start
        require(finite(level) == gain, 'timeline narration gain differs from native gain')
        require(sha(wav) == outputs[cue]['sha256'], f'timeline substitutes narration for {cue}')
        expected[(outputs[cue]['sha256'], sample(start, fps), gain)] += 1
    return fps, expected, sounds


def score_parts(score, key):
    """Exact sample placements for bounded pieces of one long native stem."""
    manifest = score/f'{key}.music.f32.parts.tsv'
    if not manifest.exists():
        return None
    entries = manifest.read_text().splitlines()
    require(entries and entries[0] == 'score-parts\t1', 'unknown score parts header')
    result, end = {}, 0
    for line in entries[1:]:
        fields = line.split('\t')
        require(len(fields) == 5, 'invalid score part row')
        start, frames, rate = map(int, fields[:3])
        digest, name = fields[3:]
        require(name not in ('', '.', '..') and '/' not in name and chr(92) not in name,
                'score part must be a local basename')
        path = score/name
        require(start == end and frames > 0 and rate == 48000, 'noncontiguous score parts')
        require(path.stat().st_size <= 64*1024*1024 and sha(path) == digest,
                'score part bytes differ')
        entry = (digest, start, 0.0)
        require(entry not in result, 'duplicate score part')
        result[entry] = (frames, path.stat().st_size)
        end += frames
    require(len(result) >= 2, 'segmented score needs at least two parts')
    return result


def cue_log(path, fps, expected, sounds, score, key):
    # Other audio must be explicitly declared as a timeline listen sound or
    # the requested chapter score stem. This is not score/mix acceptance.
    extra = {sha(sound) for sound in sounds}
    parts = score_parts(score, key) if score is not None else None
    remaining_parts = set(parts or {})
    if score is not None and parts is None:
        extra.add(sha(score/f'{key}.music.f32.wav'))
    remaining = expected.copy()
    narration_hashes = {entry[0] for entry in expected}
    for fields in rows(path):
        require(len(fields) == 13, 'invalid cue log')
        wav, digest = fields[:2]
        require(sha(wav) == digest, 'cue-log WAV differs from recorded hash')
        frame, rate = int(fields[3]), int(fields[4])
        require(rate == fps, 'cue-log fps differs from timeline')
        offset = bits(fields[6])
        gain = None if fields[8] == '-' else bits(fields[8])
        start = sample(frame, rate) + rounded(offset * 48000)
        require(start == int(fields[12]), 'cue-log start sample is inconsistent')
        entry = (digest, start, gain)
        if remaining[entry] > 0:
            remaining[entry] -= 1
        elif entry in remaining_parts:
            frames, size = parts[entry]
            require(int(fields[2]) == size and int(fields[9]) == 48000
                    and int(fields[10]) == 2 and int(fields[11]) == frames,
                    'score part cue format differs')
            remaining_parts.remove(entry)
        else:
            require(digest not in narration_hashes and digest in extra,
                    'cue log contains substituted, duplicate or unbound audio')
    require(not any(remaining.values()), 'cue log omits verified narration')
    require(not remaining_parts, 'cue log omits score part')


def directed_segments(report, cue, at, held_end=None):
    """Whole display-word segments after verified intra-sentence pauses.

    Called only after strict reconstruction. These are not arbitrary caption
    boundaries: a release exception must match the entire segment's text and
    exported millisecond endpoints. Long segments still need ordinary caption
    balancing; no exception is granted to a fragment of one.
    """
    directions = report['meta']['direction_contract']
    bindings = {r['override_index']: r for r in directions['binding']['resolved']}
    rate = report['meta']['rate']
    cursor = word_offset = 0
    result = []
    sentence_rows = report['lines'][cue]
    for sentence_index, row in enumerate(sentence_rows):
        words = row['sentence'].split()
        boundaries, token_count = {}, 0
        for index, word in enumerate(words, 1):
            token_count += len(coherent.direction_words(word))
            boundaries[token_count] = index
        segments, start, first_word, inserted = [], cursor, 0, 0
        for pause in row.get('intra_sentence_pauses', []):
            binding = bindings[pause['override_index']]
            token_end = binding['end_word_index'] + 1 - word_offset
            require(token_end in boundaries, 'directed caption boundary splits a display word')
            end_word = boundaries[token_end]
            require(first_word < end_word < len(words), 'unordered directed caption boundary')
            # The prefix remains visible through the inserted silence.
            end = cursor + pause['source_sample'] - row['source_in_sample'] + inserted + pause['inserted_samples']
            segments.append((start, end, ' '.join(words[first_word:end_word])))
            start, first_word = end, end_word
            inserted += pause['inserted_samples']
        segments.append((start, cursor + row['out_samples'], ' '.join(words[first_word:])))
        for index in range(1, len(segments)):
            a, b, text = segments[index]
            # Native sentence_captions emits one caption only when its greedy
            # 42-character wrap fits two lines. Do not waive a long segment
            # that the native renderer would balance into several captions.
            widths = []
            for word in text.split():
                if widths and widths[-1] + 1 + len(word) <= 42:
                    widths[-1] += 1 + len(word)
                else:
                    widths.append(len(word))
            if len(widths) > 2:
                continue
            end = at + b / rate
            if sentence_index == len(sentence_rows) - 1 and index == len(segments) - 1 and held_end is not None:
                end = held_end
            result.append({'cue': cue, 'start_ms': rounded((at + a / rate) * 1000),
                           'end_ms': rounded(end * 1000), 'text': text,
                           'preceding_text': segments[index - 1][2]})
        cursor += row['out_samples'] + row['gap_after_samples']
        word_offset += token_count
    return result


def prediction_end(line, following, fps, predicts, outputs, declared, offset):
    """The native mark window, with an exactly matching authored cue pair."""
    if following is None or declared[(line[1], following[1])] != 1:
        return None
    cue = line[1]
    end = int(line[2]) / fps + outputs[cue]['frames'] / outputs[cue]['sample_rate']
    next_start = int(following[2]) / fps
    if any(end <= mark < next_start for mark in predicts):
        return offset + next_start
    return None


def verify(args):
    topic = args.topic.resolve()
    require(topic.parent == REPO/'explainer_corpus', 'topic must be in this repository')
    script, units = topic/'script.tsv', topic/'narration_units.json'
    if args.review_units is not None:
        units = review_plan(args)
    require(json.loads(units.read_text())['direction_contract']['topic'] == topic.name,
            'plan names a different topic')
    texts = coherent.read_script(script)
    require(args.chapter_key and len(set(args.chapter_key)) == len(args.chapter_key),
            'chapter selection is empty or duplicated')
    prefixes = []
    for key in args.chapter_key:
        require(re.fullmatch(r'[0-9]{2}_[A-Za-z0-9_]+', key), 'invalid chapter key')
        prefixes.append(key.split('_', 1)[0])
    require(len(set(prefixes)) == len(prefixes), 'duplicate chapter prefix')
    selected = [cue for cue in texts if cue.split('_', 1)[0] in prefixes]
    require(all(any(cue.startswith(prefix+'_') for cue in selected) for prefix in prefixes),
            'selected chapter has no script cues')
    outputs = json.loads((args.narration/'assembly.json').read_text())['outputs']
    scope = None
    if set(outputs) != set(texts):
        require(args.consumer in ('render', 'captions') and len(prefixes) == 1
                and set(outputs) == set(selected), 'partial assembly does not cover requested consumer')
        scope = prefixes[0]
    if args.consumer in ('score', 'master'):
        require(set(selected) == set(texts), 'whole-film consumer requires all script cues')
    coherent.verify(args.narration, script, units, require_directions=True, chapter=scope)
    caption_segments = []
    prediction_holds = []
    # Native render reads these exact files. Other consumers must also bind
    # their actual timeline/cue-log files instead of merely verifying a directory.
    if args.consumer != 'render':
        require(args.timelines is not None, 'consumer requires timelines')
        gain = finite(args.gain_db)
        report = json.loads((args.narration/'report.json').read_text())
        # The map is already bound by the strict verifier. A timeline mark
        # alone cannot declare a new editorial hold for final diagnostics.
        contract = report['meta']['direction_contract']['binding']['contract']
        direction_map = json.loads(coherent.direction_map_path(
            contract['map']['path'], topic.name).read_text())
        declared_predictions = Counter((h.get('cue'), h.get('next_cue'))
            for h in direction_map.get('visual_holds', [])
            if h.get('video') == topic.name and h.get('kind') == 'prediction')
        offset = 0.0
        for key, prefix in zip(args.chapter_key, prefixes):
            expected_ids = [cue for cue in texts if cue.startswith(prefix+'_')]
            fps, expected, sounds = timeline(args.timelines/f'{key}.timeline.tsv', key,
                                             expected_ids, outputs, gain)
            if args.review_units is not None:
                require(fps <= 30, 'lab caption timeline exceeds 30 fps')
            if args.consumer == 'master':
                cue_log(args.timelines/f'{key}.cuelog.tsv', fps, expected, sounds, args.score, key)
                data = rows(args.timelines/f'{key}.timeline.tsv')
                lines = [r for r in data if r[0] == 'line']
                predicts = [int(r[2]) / fps for r in data if r[0] == 'mark' and r[1] == 'predict']
                for index, line in enumerate(lines):
                    cue, start = line[1], int(line[2]) / fps
                    following = lines[index + 1] if index + 1 < len(lines) else None
                    held_end = prediction_end(line, following, fps, predicts, outputs, declared_predictions, offset)
                    if held_end is not None:
                        prediction_holds.append({'cue': cue, 'next_cue': lines[index + 1][1],
                            'end_ms': rounded(held_end * 1000)})
                    caption_segments.extend(directed_segments(report, cue, offset + start, held_end))
                offset += int(next(r[1] for r in data if r[0] == 'frames')) / fps
    return {'consumer': args.consumer, 'topic': topic.name, 'verified_cues': len(outputs),
            'selected_cues': len(selected), 'chapter_scope': scope,
            'plan_mode': 'lab-review' if args.review_units is not None else 'canonical',
            'promotion': False, 'listening_acceptance': False,
            'directed_caption_segments': caption_segments, 'prediction_holds': prediction_holds}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--topic', type=Path, required=True)
    parser.add_argument('--narration', type=Path, required=True)
    parser.add_argument('--consumer', choices=('render', 'captions', 'score', 'master'), required=True)
    parser.add_argument('--chapter-key', action='append', required=True)
    parser.add_argument('--gain-db', required=True)
    parser.add_argument('--timelines', type=Path)
    parser.add_argument('--score', type=Path)
    parser.add_argument('--review-units', type=Path)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--width', type=int)
    parser.add_argument('--height', type=int)
    parser.add_argument('--fps', type=int)
    args = parser.parse_args()
    try:
        print(json.dumps(verify(args), sort_keys=True))
    except (ValueError, OSError, KeyError, TypeError) as error:
        print(f'narration delivery refused: {error}', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
