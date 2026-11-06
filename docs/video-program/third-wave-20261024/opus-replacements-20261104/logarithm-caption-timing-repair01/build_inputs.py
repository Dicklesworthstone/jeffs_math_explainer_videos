"""Build the inputs tools/retime_captions_from_phrases.py needs for the final logarithm master.

The bound phrase timings hold 1 of the 111 same-sentence caption boundaries, so this file
writes a timing input from real word times instead, using the tool unchanged:

  inputs/recognition-sources.json   the two recognition sources, by hash
  inputs/timings.json               marks only at boundaries that should move
  inputs/timelines/*.timeline.tsv   the 13 real timelines; only each line row's WAV path differs,
                                    from the remote job folder to the local prepared02/narration
  inputs/boundaries.tsv             every one of the 111 boundaries and what happened to it

Word time per boundary is the later of two recognizer onsets, because these onsets run early:
  ASR02    whole encoded master with score
  takes    the selected clean take, mapped through report.json's sample spans
Then the clean cue WAV is consulted (runs of 10 ms frames at or below -60 dBFS, 50 ms or more).
The end of a run is used if the time sits inside it, or if the run starts within 0.35 s after
the time and leaves at least 0.10 s of the recognized word after it. The second limit matters:
a word ending in a comma has its recognized span stretched through the pause that follows it.
Both rules were read against the energy of all 25 boundaries with a run nearby. Last, the time
advances to the first frame at or above -40 dBFS within 0.35 s, because a breath-level pause
never reaches -60 dBFS (block 254). These are estimates, not heard boundaries.

Run from anywhere with `python3 -I -B`. Refuses to write if inputs/ exists.
"""
import difflib
import hashlib
import importlib.util
import json
import math
import pathlib
import re
import wave

import numpy as np

HERE = pathlib.Path(__file__).resolve().parent
R = HERE.parents[4]
EL = R / 'narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104'
FF = R / 'narration_lab/third-wave-20261024/root/five-film-release-20261027'
O = EL / 'remote-mac-mini-max01'
A = EL / 'production-audio-stage09-01'
NARRATION = A / 'prepared02/narration'
SCRIPT = EL / 'final-picture-repair01/workspace/explainer_corpus/logarithm_mysteries/script.tsv'
REPORT = NARRATION / 'report.json'
VERIFICATION = O / 'jobs/logarithm-remote-verify01/verification.json'
SRT = O / 'jobs/logarithm-remote-render-work01/logarithm_mysteries.en.srt'
ASR02 = O / 'jobs/logarithm-remote-asr02/full-large.json'
TIMELINES = O / 'final-timelines-import01'
REMOTE_NARRATION = 'jobs/logarithm-remote-assemble01/narration01'
SELECTION = A / 'selection02.json'
PHRASES = A / 'prepared02-timings/phrase-timings.json'
TRANSFER = O / 'audio-transfer01.json'
TOOL = R / 'tools/retime_captions_from_phrases.py'
RUNS = ('opening', 'middle', 'turns', 'remaining', 'clarity')

PINS = {
    TOOL: '028a934b9d25ef70a49b6a7ec934b284424aebf112fe7366222d4b352570a78f',
    SCRIPT: '559afa3698222952036ece90a25eab926d45e201fc916bf32412e9485a52170a',
    REPORT: '09c01854aabba0c07a979195456dcbc92801260786a1416fe1e91bd3a08bb45a',
    VERIFICATION: 'f40e3999cf4f3400705ee13e505d56900326ec648d6ac9403c5a6bf29f1ffc20',
    SRT: 'c9321cd7a2912d4457f2dc617735fd06f0b2fc7d684442815fa46b989bc31b7c',
    ASR02: '50d281e7f40b1c017e19d1ad7f8e85aa58b19b5cb863ff52cfda2b2507196eb0',
    FF / 'logarithm-opening-recognition01/command.log': '4113e65f9c4e91a3aa5fbdacc6de6d7311fc9acdd9318ed5cda9d44eda46a8e8',
    FF / 'logarithm-middle-recognition01/command.log': '1c7fb420b88a6557d49e6ad395926b28c0199620679a80521180fb0e428957a8',
    FF / 'logarithm-turns-recognition01/command.log': '8d20c0079989fd2d652a9e39c49301d6eb9092d9c959669f7696c1aa60557333',
    FF / 'logarithm-remaining-recognition01/command.log': '3e479160bebb7daf0759caab2f7a538df6527cd53d34b3cf218c5a0295a442b7',
    FF / 'logarithm-clarity-recognition01/command.log': '5145e630ed8be7f0aec89b2f5afe1262f15c8f598ed3fa3f735c129c66a24d75',
}
FLOOR_S = 0.25          # smaller moves cannot be told from the recognizer's early bias
DISAGREE_S = 0.35       # the two sources this far apart: leave the boundary alone
AGREE_S = 0.10          # the two sources this close: they support each other
RUN_AHEAD_S = 0.35      # a quiet run starting this soon after the onset is before the word
WORD_LEFT_S = 0.10      # ... if this much of the recognized word remains after the run
MAX_CPS = 20            # the production caption screen's limit
MIN_BLOCK_MS = 1000     # the shortest native block is 1.03 s
QUIET_DB = -60.0
QUIET_FRAMES = 5        # 50 ms of 10 ms frames
SPEECH_DB = -40.0       # the first frame this loud after a pause is the word's body


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def rel(path):
    return path.relative_to(R).as_posix()


def norm(word):
    return re.sub(r"[^a-z0-9']", '', word.lower().replace('’', "'"))


def load_tool():
    spec = importlib.util.spec_from_file_location('retime_tool', TOOL)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


ONE_WORD = dict(zip(('zero one two three four five six seven eight nine ten eleven twelve thirteen fourteen '
                     'fifteen sixteen seventeen eighteen nineteen twenty').split(), map(str, range(21))))
ONE_WORD.update({'thirty': '30', 'forty': '40', 'fifty': '50', 'sixty': '60', 'seventy': '70', 'eighty': '80', 'ninety': '90'})


def align(script_words, tokens):
    """Script word index -> (token index, whether a matched neighbour stands beside it)."""
    a, b = [norm(w) for w in script_words], [t['w'] for t in tokens]
    pairs = {}
    for tag, i1, i2, j1, j2 in difflib.SequenceMatcher(None, a, b, autojunk=False).get_opcodes():
        if tag == 'equal':
            pairs.update({i1 + k: j1 + k for k in range(i2 - i1)})
        elif tag == 'replace' and i2 - i1 == 1 and j2 - j1 == 1:
            if ONE_WORD.get(a[i1]) == b[j1]:
                pairs[i1] = j1      # a one-word number the recognizer wrote in digits: six / 6
            elif not re.search(r'\d', b[j1]) and difflib.SequenceMatcher(None, a[i1], b[j1]).ratio() >= 0.8:
                pairs[i1] = j1      # a spelling variant such as spiralling / spiraling
    return {i: (j, pairs.get(i - 1) == j - 1 or pairs.get(i + 1) == j + 1) for i, j in pairs.items()}


def tokens_of(segments):
    rows = [{'w': norm(s['text']), 'a': s['start_sec'], 'b': s['end_sec']} for s in segments]
    return [t for t in rows if t['w']]


def levels(path, rate):
    """dBFS of each 10 ms frame."""
    with wave.open(str(path), 'rb') as audio:
        if (audio.getframerate(), audio.getnchannels(), audio.getsampwidth()) != (rate, 1, 2):
            raise ValueError(f'Unexpected WAV format: {path.name}')
        x = np.frombuffer(audio.readframes(audio.getnframes()), dtype='<i2').astype(float) / 32768
    hop = rate // 100
    n = len(x) // hop
    return 20 * np.log10(np.maximum(np.sqrt(np.mean(x[:n * hop].reshape(n, hop) ** 2, axis=1)), 1e-7))


def quiet_runs(db):
    runs, i = [], 0
    n = len(db)
    while i < n:
        if db[i] <= QUIET_DB:
            j = i
            while j < n and db[j] <= QUIET_DB:
                j += 1
            if j - i >= QUIET_FRAMES:
                runs.append((i / 100, j / 100))
            i = j
        else:
            i += 1
    return runs


def main():
    out = HERE / 'inputs'
    for path, want in PINS.items():
        if sha(path) != want:
            raise ValueError(f'Pinned input changed: {rel(path)}')
    tool = load_tool()
    verification = json.loads(VERIFICATION.read_text())
    report = json.loads(REPORT.read_text())
    phrases = json.loads(PHRASES.read_text())
    transfer = {f['path'].split('prepared02/narration/')[-1]: f['sha256']
                for f in json.loads(TRANSFER.read_text())['files'] if 'prepared02/narration/' in f['path']}
    selected = json.loads(SELECTION.read_text())['takes']
    rate = report['meta']['rate']
    script = dict(row.split('\t', 1) for row in SCRIPT.read_text().splitlines()
                  if '\t' in row and not row.startswith('#'))
    if phrases['script_sha256'] != PINS[SCRIPT]:
        raise ValueError('Phrase timings bind another script')
    if [p['cue'] for p in verification['placements']] != list(script):
        raise ValueError('Placement order differs from the script')

    # The real timelines, bound by the hashes in verification.json. The tool opens the WAV named
    # in each line row, and the remote job's folder is not local, so only that field is rewritten.
    timelines = {}
    cue_start = {}
    offset = 0.0
    for chapter in verification['chapters']:
        real = TIMELINES / f"{chapter['chapter']}.timeline.tsv"
        if sha(real) != chapter['timeline_sha256']:
            raise ValueError(f'Timeline differs from verification: {real.name}')
        text = []
        for line in real.read_text().splitlines():
            row = line.split('\t')
            if row[0] == 'line':
                if not row[4].endswith(f'{REMOTE_NARRATION}/{row[1]}.wav'):
                    raise ValueError(f'Unexpected WAV path: {row[1]}')
                row[4] = rel(NARRATION / f'{row[1]}.wav')
                cue_start[row[1]] = offset + int(row[2]) / 60
            text.append('\t'.join(row))
        timelines[real.name] = '\n'.join(text) + '\n'
        offset += chapter['frames'] / 60
    if set(cue_start) != set(script):
        raise ValueError('Timeline cues differ from the script')

    # Every cue WAV: same bytes as the phrase timings and as the transfer to the render host.
    wav_sha = {}
    for cue in script:
        wav_sha[cue] = sha(NARRATION / f'{cue}.wav')
        entry = phrases['cues'][cue]
        if wav_sha[cue] != entry['wav_sha256'] or wav_sha[cue] != transfer[f'{cue}.wav']:
            raise ValueError(f'WAV identity differs: {cue}')
        if hashlib.sha256(script[cue].encode()).hexdigest() != entry['text_sha256']:
            raise ValueError(f'Text identity differs: {cue}')

    # Sentences, words and spans, by the same reading the tool uses.
    sentences, flat, cue_words = [], [], {}
    for cue in script:
        words = script[cue].split()
        cue_words[cue] = words
        raw = list(re.finditer(r'\S+', script[cue]))
        cursor = sample = 0
        for item in report['lines'][cue]:
            count = len(item['sentence'].split())
            if words[cursor:cursor + count] != item['sentence'].split():
                raise ValueError(f'Report words differ: {cue}')
            sentences.append({'cue': cue, 'first': cursor, 'count': count, 'flat': len(flat),
                              'start': sample / rate, 'end': (sample + item['out_samples']) / rate,
                              'source_in': item['source_in_sample'], 'source_out': item['source_out_sample'],
                              'bytes': [len(script[cue][:w.start()].encode()) for w in raw[cursor:cursor + count]]})
            flat.extend(words[cursor:cursor + count])
            sample += item['out_samples'] + item['gap_after_samples']
            cursor += count
    by_cue = {}
    for s in sentences:
        by_cue.setdefault(s['cue'], []).append(s)

    # Source 1: the whole master.
    master_tokens = tokens_of(json.loads(ASR02.read_text())['result']['segments'])
    master_pairs = align(flat, master_tokens)

    # Source 2: the selected clean takes.
    logs = {}
    for run in RUNS:
        for line in (FF / f'logarithm-{run}-recognition01/command.log').read_text(errors='replace').splitlines():
            if line.startswith('{'):
                record = json.loads(line)
                if record.get('status') == 'ok':
                    logs.setdefault(record['input'].split('jeffs_math_explainer_videos/')[-1], []).append(record)
    take_tokens, take_pairs = {}, {}
    for cue, path in selected.items():
        if len(logs.get(path, [])) != 1:
            raise ValueError(f'Selected take needs exactly one recognition record: {cue}')
        take_tokens[cue] = tokens_of(logs[path][0]['report']['result']['segments'])
        take_pairs[cue] = align(cue_words[cue], take_tokens[cue])

    def take_word(sentence, word):
        """(onset, end, supported) in cue seconds, or None."""
        hit = take_pairs[sentence['cue']].get(sentence['first'] + word)
        if hit is None:
            return None
        token = take_tokens[sentence['cue']][hit[0]]
        sample = token['a'] * rate
        if not sentence['source_in'] <= sample < sentence['source_out']:
            return None
        onset = sentence['start'] + (sample - sentence['source_in']) / rate
        return onset, onset + token['b'] - token['a'], hit[1]

    def master_word(sentence, word):
        hit = master_pairs.get(sentence['flat'] + word)
        if hit is None:
            return None
        token = master_tokens[hit[0]]
        onset = token['a'] - cue_start[sentence['cue']]
        if not sentence['start'] <= onset < sentence['end']:
            return None
        return onset, token['b'] - cue_start[sentence['cue']], hit[1]

    level = {cue: levels(NARRATION / f'{cue}.wav', rate) for cue in script}
    quiet = {cue: quiet_runs(level[cue]) for cue in script}
    captions = tool.parse_srt(SRT.read_text())
    groups = []
    si = wi = 0
    for caption in captions:
        count = len(' '.join(caption['lines']).split())
        groups.append((si, wi))
        wi += count
        if wi == sentences[si]['count']:
            si, wi = si + 1, 0
    if si != len(sentences) or wi:
        raise ValueError('Captions do not cover the sentences')

    def chars(caption):
        return len(' '.join(' '.join(caption['lines']).split()))

    def need_ms(caption):
        return max(MIN_BLOCK_MS, math.ceil(chars(caption) * 1000 / MAX_CPS))

    # Left to right, as the tool moves them: a block's start is settled before its end is decided.
    starts = [c['start'] for c in captions]
    table, marks = [], {cue: [] for cue in script}
    for index in range(1, len(captions)):
        s_index, word = groups[index]
        if groups[index - 1][0] != s_index:
            continue
        sentence = sentences[s_index]
        cue = sentence['cue']
        old_ms = captions[index]['start']
        in_master = master_word(sentence, word)
        in_take = take_word(sentence, word)
        agree = in_master and in_take and abs(in_master[0] - in_take[0]) <= AGREE_S
        found = [w for w in (in_take, in_master) if w and (agree or w[2])]
        master = in_master[0] if in_master in found else None
        take = in_take[0] if in_take in found else None
        row = {'block': index + 1, 'cue': cue, 'word': flat[sentence['flat'] + word], 'old_s': old_ms / 1000,
               'master_s': master, 'take_s': take, 'estimate_s': None, 'basis': '', 'new_s': old_ms / 1000,
               'decision': 'unchanged', 'reason': ''}
        table.append(row)
        if not found:
            row['reason'] = 'neither source has this word (inside a number recognized as one digit token)'
            continue
        if len(found) == 2 and abs(master - take) > DISAGREE_S:
            row['reason'] = 'sources disagree'
            continue
        estimate = max(w[0] for w in found)
        word_end = found[0][1]
        basis = 'later of both' if len(found) == 2 else ('master only' if master is not None else 'take only')
        run = next(((a, b) for a, b in quiet[cue] if a <= estimate < b), None)
        if run:
            estimate, basis = run[1], basis + ', end of quiet run around it'
        else:
            run = next(((a, b) for a, b in quiet[cue]
                        if estimate <= a <= estimate + RUN_AHEAD_S and b <= word_end - WORD_LEFT_S), None)
            if run:
                estimate, basis = run[1], basis + ', end of quiet run just after it'
        frame = math.floor(estimate * 100 + 1e-9)
        rise = next((f for f in range(frame, min(frame + round(RUN_AHEAD_S * 100) + 1, len(level[cue])))
                     if level[cue][f] >= SPEECH_DB), None)
        if rise is not None and rise > frame:
            estimate, basis = rise / 100, basis + ', speech rise'
        row['estimate_s'], row['basis'] = estimate, basis
        target = round((cue_start[cue] + estimate) * 1000)
        if abs(target - old_ms) < FLOOR_S * 1000:
            row['reason'] = 'inside the floor'
            continue
        low = starts[index - 1] + need_ms(captions[index - 1])
        high = captions[index]['end'] - need_ms(captions[index])
        if low > high:
            row['reason'] = 'no room for both blocks'
            continue
        limited = min(max(target, low), high)
        if abs(limited - old_ms) < FLOOR_S * 1000:
            row['reason'] = 'guard leaves less than the floor'
            continue
        mark = round(limited / 1000 - cue_start[cue], 6)
        if round((cue_start[cue] + mark) * 1000) != limited or not sentence['start'] < mark < sentence['end']:
            row['reason'] = 'mark outside its sentence'
            continue
        starts[index] = limited
        row['new_s'] = limited / 1000
        row['decision'] = 'moved' if limited == target else 'moved, limited by guard'
        row['reason'] = '' if limited == target else f'wanted {target / 1000:.3f}'
        marks[cue].append({'byte': sentence['bytes'][word], 'seconds': mark, 'basis': basis})

    (out / 'timelines').mkdir(parents=True)
    for name, text in timelines.items():
        with (out / 'timelines' / name).open('x') as stream:
            stream.write(text)
    sources = {'scope': 'Recognizer estimates only, not heard boundaries. Same model on both sources.',
               'model': 'ggml-large-v3-turbo.bin',
               'master_run': {'path': rel(ASR02), 'sha256': PINS[ASR02]},
               'take_runs': [{'path': rel(p), 'sha256': PINS[p]} for p in PINS if p.name == 'command.log'],
               'selection': {'path': rel(SELECTION), 'sha256': sha(SELECTION)},
               'phrase_timings': {'path': rel(PHRASES), 'sha256': sha(PHRASES)}}
    with (out / 'recognition-sources.json').open('x') as stream:
        json.dump(sources, stream, indent=2)
        stream.write('\n')
    timing = {'basis': 'Later of two recognizer onsets, end of quiet run where the onset is inside one; '
                       'only boundaries to move have marks; not heard boundaries',
              'script_sha256': PINS[SCRIPT], 'asr_log_sha256': sha(out / 'recognition-sources.json'),
              'cues': {cue: {'wav_sha256': wav_sha[cue],
                             'text_sha256': hashlib.sha256(script[cue].encode()).hexdigest(),
                             'marks': marks[cue]} for cue in script}}
    with (out / 'timings.json').open('x') as stream:
        json.dump(timing, stream, indent=1)
        stream.write('\n')
    head = ['block', 'cue', 'word', 'old_s', 'master_s', 'take_s', 'estimate_s', 'basis', 'new_s', 'decision', 'reason']
    show = lambda v: '' if v is None else (f'{v:.3f}' if isinstance(v, float) else str(v))
    with (out / 'boundaries.tsv').open('x') as stream:
        stream.write('\t'.join(head) + '\n')
        for row in table:
            stream.write('\t'.join(show(row[k]) for k in head) + '\n')
    moved = [r for r in table if r['decision'] != 'unchanged']
    print(f'{len(table)} boundaries, {len(moved)} to move, '
          f"{sum(r['decision'] == 'moved, limited by guard' for r in table)} limited by a guard")
    tally = {}
    for r in table:
        if r['decision'] == 'unchanged':
            tally[r['reason']] = tally.get(r['reason'], 0) + 1
    print('unchanged:', tally)


if __name__ == '__main__':
    main()
