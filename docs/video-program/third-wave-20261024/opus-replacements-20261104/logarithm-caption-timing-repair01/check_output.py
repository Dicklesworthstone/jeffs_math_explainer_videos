"""Bounded checks of sidecars-timing01 against the real inputs. Nothing here plays or watches anything.

Run from anywhere with `python3 -I -B`. Exit 1 if a check fails. Writes only
outputs/changed-intervals.tsv and outputs/overlay-sample-impact.tsv, and refuses if they exist.
"""
import csv
import hashlib
import importlib.util
import json
import pathlib
import re
import statistics
import sys
import wave

import numpy as np

HERE = pathlib.Path(__file__).resolve().parent


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


B = load('build_inputs_module', HERE / 'build_inputs.py')
R = B.R
tool = B.load_tool()
SCREEN = R / 'narration_lab/third-wave-20261024/root/six-machine-screened-production-20261103/caption-interval-audit01.py'
SCREEN_SHA = '3cc2643615ce5263b01d429ae6f72326c95e777af3e7f29ef9266a05155fa0c4'
RENDER = B.O / 'jobs/logarithm-remote-render-work01'
MASTER = RENDER / 'logarithm_mysteries_master.mp4'
OLD_SRT, OLD_VTT = RENDER / 'logarithm_mysteries.en.srt', RENDER / 'logarithm_mysteries.en.vtt'
NEW = HERE / 'sidecars-timing01'
NEW_SRT, NEW_VTT = NEW / 'captions.en.srt', NEW / 'captions.en.vtt'
PLAN = HERE.parent / 'logarithm-caption-overlay01/plan01.json'
failures = []


def check(name, ok, detail=''):
    print(f"{'PASS' if ok else 'FAIL'}  {name}" + (f'  [{detail}]' if detail else ''))
    if not ok:
        failures.append(name)


print('=== 1. inputs measured now; the master and both native sidecars are untouched')
for path, want in list(B.PINS.items()) + [
        (MASTER, '8534ec211790fb8400b809de19a6d1cb75af6150a5e90566f6a93bb90e7efd69'),
        (OLD_VTT, '45833988985413f3cfd8c44d5e18959be47ff5003d0dcbc570ce0b74ddfc0b45'),
        (SCREEN, SCREEN_SHA)]:
    check(path.name, B.sha(path) == want, want[:16])

print('=== 2. the real timelines + report reproduce the native sentence edges')
verification = json.loads(B.VERIFICATION.read_text())
report = json.loads(B.REPORT.read_text())
script = dict(row.split('\t', 1) for row in B.SCRIPT.read_text().splitlines()
              if '\t' in row and not row.startswith('#'))
rate = report['meta']['rate']
real_ok = copies_ok = 0
for chapter in verification['chapters']:
    name = f"{chapter['chapter']}.timeline.tsv"
    real_ok += B.sha(B.TIMELINES / name) == chapter['timeline_sha256']
    rows_real = [line.split('\t') for line in (B.TIMELINES / name).read_text().splitlines()]
    rows_copy = [line.split('\t') for line in (HERE / 'inputs/timelines' / name).read_text().splitlines()]
    same_shape = len(rows_real) == len(rows_copy)
    for a, b in zip(rows_real, rows_copy):
        if a[0] == 'line':
            same_shape &= a[:4] == b[:4] and len(a) == len(b) == 5 and b[4] == B.rel(B.NARRATION / f'{a[1]}.wav')
        else:
            same_shape &= a == b
    copies_ok += same_shape
check('13 real timelines match the hashes in verification.json', real_ok == 13, f'{real_ok}')
check('the copies given to the tool differ from them only in line rows\' WAV path', copies_ok == 13, f'{copies_ok}')
placements = tool.timeline(list((HERE / 'inputs/timelines').glob('*.timeline.tsv')))
check('13 timelines give the 80 cues in script order', [p[0] for p in placements] == list(script), f'{len(placements)} cues')
cue_start = {cue: start for cue, start, _ in placements}
check('cue starts equal verification frames / 60',
      all(abs(cue_start[p['cue']] - p['start_frame'] / 60) < 1e-6 for p in verification['placements']))
lengths = 0
for p in verification['placements']:
    with wave.open(str(B.NARRATION / f"{p['cue']}.wav"), 'rb') as audio:
        lengths += abs(audio.getnframes() / audio.getframerate() - p['duration_s']) < 1e-6
check('all 80 local cue WAV durations equal the verified placements', lengths == 80, f'{lengths}')
sentences = []
for cue in script:
    sample = 0
    for item in report['lines'][cue]:
        sentences.append({'cue': cue, 'words': item['sentence'].split(),
                          'start': cue_start[cue] + sample / rate,
                          'end': cue_start[cue] + (sample + item['out_samples']) / rate})
        sample += item['out_samples'] + item['gap_after_samples']
old = tool.parse_srt(OLD_SRT.read_text())
new = tool.parse_srt(NEW_SRT.read_text())
groups = []
si = wi = 0
for caption in old:
    groups.append((si, wi))
    wi += len(' '.join(caption['lines']).split())
    if wi == len(sentences[si]['words']):
        si, wi = si + 1, 0
first = [i for i, g in enumerate(groups) if g[1] == 0]
last = [i - 1 for i in first[1:]] + [len(old) - 1]
off_start = [abs(old[i]['start'] - sentences[n]['start'] * 1000) for n, i in enumerate(first)]
check('all 364 native sentence starts reproduced within 1 ms', len(first) == 364 and max(off_start) <= 1.0, f'worst {max(off_start):.3f} ms')
held = [(n, old[i]['end'] - sentences[n]['end'] * 1000) for n, i in enumerate(last)]
exact = [n for n, d in held if abs(d) <= 1.0]
rest = [(n, d) for n, d in held if abs(d) > 1.0]
check('native sentence ends: exact, or later (a hold), never earlier', all(d > 0 for _, d in rest), f'{len(exact)} exact, {len(rest)} held')


def compare(before, after, edges_first, edges_last):
    problems = []
    if len(before) != len(after):
        problems.append('block count')
    for i, (a, b) in enumerate(zip(before, after)):
        if a['lines'] != b['lines']:
            problems.append(f'block {i + 1} text')
    if any(before[i]['start'] != after[i]['start'] for i in edges_first):
        problems.append('a sentence start moved')
    if any(before[i]['end'] != after[i]['end'] for i in edges_last):
        problems.append('a sentence end moved')
    for i in range(1, len(after)):
        if i not in edges_first and after[i - 1]['end'] != after[i]['start']:
            problems.append(f'block {i + 1} not contiguous inside its sentence')
    return problems


print('=== 3. only inside-sentence times changed')
check('475 blocks; words, line breaks, sentence starts and ends all equal', not compare(old, new, first, last), f'{len(new)} blocks')
planted = [dict(c) for c in new]
planted[first[100]] = dict(planted[first[100]], start=planted[first[100]]['start'] + 1)
check('plant: a sentence start moved by 1 ms is caught', 'a sentence start moved' in compare(old, planted, first, last))
planted = [dict(c) for c in new]
planted[50] = dict(planted[50], lines=['changed'])
check('plant: changed text is caught', 'block 51 text' in compare(old, planted, first, last))
rows = list(csv.DictReader((HERE / 'inputs/boundaries.tsv').open(), delimiter='\t'))
check('111 boundaries tabled', len(rows) == 111)
moved = [r for r in rows if r['decision'].startswith('moved')]
got = {i + 1: c['start'] for i, c in enumerate(new) if c['start'] != old[i]['start']}
plan = {int(r['block']): round(float(r['new_s']) * 1000) for r in moved}
check('the tool moved exactly the planned 65 boundaries to the planned times', got == plan, f'{len(got)} moved')
retiming = json.loads((NEW / 'retiming.json').read_text())
check('retiming.json binds the real SRT, script, report', retiming['srt_sha256'] == B.PINS[B.SRT]
      and retiming['script_sha256'] == B.PINS[B.SCRIPT] and retiming['report_sha256'] == B.PINS[B.REPORT])
check('no boundary was refused by the tool for shape', not [s for s in retiming['skipped'] if s['reason'] != 'missing or out-of-sentence estimate'])
old_srt_text, new_srt_text = OLD_SRT.read_text(), NEW_SRT.read_text()
check('native SRT ends with one more newline than the tool writes; no other tail change', old_srt_text == old_srt_text.rstrip('\n') + '\n\n' and new_srt_text == new_srt_text.rstrip('\n') + '\n')
old_srt_lines, new_srt_lines = old_srt_text.rstrip('\n').split('\n'), new_srt_text.rstrip('\n').split('\n')
differs = [i for i, (a, b) in enumerate(zip(old_srt_lines, new_srt_lines)) if a != b]
check('SRT otherwise differs only on timestamp lines', len(old_srt_lines) == len(new_srt_lines) and all(' --> ' in new_srt_lines[i] for i in differs), f'{len(differs)} lines')

print('=== 4. the new VTT says what the new SRT says')
text = NEW_VTT.read_text()
check('VTT header', text.startswith('WEBVTT\n\n'))
vtt = []
for block in text.split('\n\n')[1:]:
    lines = block.strip().split('\n')
    a, b = lines[0].split(' --> ')
    vtt.append({'start': tool.timestamp(a.replace('.', ',')), 'end': tool.timestamp(b.replace('.', ',')), 'lines': lines[1:]})
check('VTT equals SRT in every block', vtt == new, f'{len(vtt)} cues')
old_vtt_text = OLD_VTT.read_text()
check('native VTT has the same one extra final newline', old_vtt_text == old_vtt_text.rstrip('\n') + '\n\n' and text == text.rstrip('\n') + '\n')
old_vtt_lines, new_vtt_lines = old_vtt_text.rstrip('\n').split('\n'), text.rstrip('\n').split('\n')
vdiff = [i for i, (a, b) in enumerate(zip(old_vtt_lines, new_vtt_lines)) if a != b]
check('VTT otherwise differs from native VTT only on timestamp lines', len(old_vtt_lines) == len(new_vtt_lines) and all(' --> ' in new_vtt_lines[i] for i in vdiff), f'{len(vdiff)} lines')

print('=== 5. the production caption screen, unchanged, on both files')
screen = load('caption_screen', SCREEN)
frames = sum(c['frames'] for c in verification['chapters'])
before, after = screen.screen(str(OLD_SRT), frames, 60), screen.screen(str(NEW_SRT), frames, 60)
for label, s in (('native', before), ('new', after)):
    print(f"  {label}: {s['caption_count']} captions, over 20 cps {len(s['over_20_characters_per_second'])}, "
          f"lines over 42 {len(s['lines_over_42_characters'])}, fastest {s['fastest']['characters_per_second']} cps block {s['fastest']['number']}")
check('new SRT: 0 over 20 cps, 0 lines over 42', not after['over_20_characters_per_second'] and not after['lines_over_42_characters'])
cps = lambda c: len(' '.join(' '.join(c['lines']).split())) * 1000 / (c['end'] - c['start'])
for label, cap in (('native', old), ('new', new)):
    speeds = sorted(cps(c) for c in cap)
    durations = sorted(c['end'] - c['start'] for c in cap)
    print(f'  {label}: median {statistics.median(speeds):.2f} cps, over 15: {sum(x > 15 for x in speeds)}, over 17: {sum(x > 17 for x in speeds)}, '
          f'over 18: {sum(x > 18 for x in speeds)}, over 19: {sum(x > 19 for x in speeds)}; shortest block {durations[0] / 1000:.3f} s, under 1.5 s: {sum(d < 1500 for d in durations)}')
print('  five fastest new blocks:', [(i + 1, round(cps(c), 2), ' '.join(c['lines'])[:44]) for i, c in sorted(enumerate(new), key=lambda x: -cps(x[1]))[:5]])

print('=== 6. lags, word onset minus block start; negative means the word is spoken before its block appears')
print('  ASR02 and take rows reuse the marks\' own sources, so their after values are near zero by construction.')


def lags(key):
    result = []
    for r in rows:
        if r[key]:
            c = cue_start[r['cue']]
            result.append((int(r['block']), r['word'], round(c + float(r[key]) - float(r['old_s']), 3), round(c + float(r[key]) - float(r['new_s']), 3)))
    return result


def summary(values):
    return (f'n {len(values)}, median {statistics.median(values):+.3f}, worst {min(values):+.3f}, latest {max(values):+.3f}, '
            f'|lag|>1.0: {sum(abs(x) > 1 for x in values)}, >0.75: {sum(abs(x) > .75 for x in values)}, '
            f'>0.5: {sum(abs(x) > .5 for x in values)}, >0.25: {sum(abs(x) > .25 for x in values)}')


for label, key in (('ASR02 master onset', 'master_s'), ('take onset', 'take_s'), ('estimate (later of both, quiet-run rules)', 'estimate_s')):
    table = lags(key)
    print(f'  {label}')
    print('    before:', summary([x[2] for x in table]))
    print('    after: ', summary([x[3] for x in table]))
table = lags('estimate_s')
print('  worst before, by the estimate:', [(b, w, x, y) for b, w, x, y in sorted(table, key=lambda t: t[2])[:12]])
print('  worst after, by the estimate: ', [(b, w, y) for b, w, _, y in sorted(table, key=lambda t: -abs(t[3]))[:12]])
print('  worst before, ASR02 alone:    ', [(b, w, x) for b, w, x, _ in sorted(lags('master_s'), key=lambda t: t[2])[:12]])
known = lags('master_s')
pairs = [(float(r['master_s']), float(r['take_s'])) for r in rows if r['master_s'] and r['take_s']]
difference = [a - b for a, b in pairs]
print(f'  ASR02 minus take at {len(pairs)} boundaries: median {statistics.median(difference):+.3f}, '
      f'|d|>0.1: {sum(abs(x) > .1 for x in difference)}, >0.2: {sum(abs(x) > .2 for x in difference)}, >0.3: {sum(abs(x) > .3 for x in difference)}')

print('=== 7. every moved boundary has speech energy right after it')
late = []
for r in moved:
    with wave.open(str(B.NARRATION / f"{r['cue']}.wav"), 'rb') as audio:
        audio.setpos(max(0, int((float(r['new_s']) - cue_start[r['cue']]) * rate)))
        x = np.frombuffer(audio.readframes(rate // 5), dtype='<i2').astype(float) / 32768
    hop = rate // 100
    n = len(x) // hop
    level = 20 * np.log10(np.maximum(np.sqrt(np.mean(x[:n * hop].reshape(n, hop) ** 2, axis=1)), 1e-7))
    if not (level >= -40).any():
        late.append((r['block'], r['word']))
check('at least one frame at or above -40 dBFS within 0.2 s after each new boundary', not late, str(late))

print('=== 8. boundaries left unresolved, with the take words around them')
take_rows = {}
for run in B.RUNS:
    for line in (B.FF / f'logarithm-{run}-recognition01/command.log').read_text(errors='replace').splitlines():
        if line.startswith('{'):
            record = json.loads(line)
            if record.get('status') == 'ok':
                take_rows[record['input'].split('jeffs_math_explainer_videos/')[-1]] = record
selected = json.loads(B.SELECTION.read_text())['takes']
for r in rows:
    if r['reason'].startswith('neither'):
        at = float(r['old_s']) - cue_start[r['cue']]
        near = [(s['text'].strip(), round(s['start_sec'], 2), round(s['end_sec'], 2))
                for s in take_rows[selected[r['cue']]]['report']['result']['segments'] if abs(s['start_sec'] - at) < 2.2]
        print(f"  block {r['block']} {r['cue']} {r['word']!r}: starts {at:.3f} in its cue; recording time, take words: {near}")

print('=== 9. where the visible caption differs, and what that does to the 23 overlay samples')
windows = []
for i in range(1, len(new)):
    a, b = old[i]['start'], new[i]['start']
    if a != b:
        # Between the two times the native file shows block i and the new file block i+1, or the reverse.
        windows.append({'from_ms': min(a, b), 'to_ms': max(a, b), 'native_block': i if b < a else i + 1, 'new_block': i + 1 if b < a else i})
total = sum(w['to_ms'] - w['from_ms'] for w in windows)
with (HERE / 'outputs/changed-intervals.tsv').open('x') as stream:
    stream.write('from_s\tto_s\tfirst_frame\tlast_frame\tnative_block\tnew_block\tnative_text\tnew_text\n')
    for w in windows:
        first_frame = int(np.ceil(w['from_ms'] * 60 / 1000))
        last_frame = int(np.ceil(w['to_ms'] * 60 / 1000)) - 1
        stream.write('\t'.join(map(str, (w['from_ms'] / 1000, w['to_ms'] / 1000, first_frame, last_frame, w['native_block'], w['new_block'],
                                         ' / '.join(old[w['native_block'] - 1]['lines']), ' / '.join(new[w['new_block'] - 1]['lines'])))) + '\n')
print(f"  {len(windows)} intervals, {total / 1000:.3f} s of {new[-1]['end'] / 1000:.3f} s, are the only times either file shows different text")
samples = json.loads(PLAN.read_text())['samples']
active = lambda cap, ms: next((i for i, c in enumerate(cap) if c['start'] <= ms < c['end']), None)
text_changed = interval_changed = 0
with (HERE / 'outputs/overlay-sample-impact.tsv').open('x') as stream:
    stream.write('id\tframe\tseconds\tnative_block\tnew_block\ttext_at_frame\tinterval\tnative_start_s\tnative_end_s\tnew_start_s\tnew_end_s\n')
    for s in samples:
        ms = s['frame'] * 1000 / 60
        a, b = active(old, ms), active(new, ms)
        same_text = a == b
        same_times = same_text and (a is None or (old[a]['start'], old[a]['end']) == (new[b]['start'], new[b]['end']))
        text_changed += not same_text
        interval_changed += same_text and not same_times
        stream.write('\t'.join(map(str, (s['id'], s['frame'], s['seconds'], '' if a is None else a + 1, '' if b is None else b + 1,
                                         'same' if same_text else 'DIFFERENT', 'same' if same_times else 'moved',
                                         '' if a is None else old[a]['start'] / 1000, '' if a is None else old[a]['end'] / 1000,
                                         '' if b is None else new[b]['start'] / 1000, '' if b is None else new[b]['end'] / 1000))) + '\n')
print(f'  of {len(samples)} sample frames: {text_changed} show different text, {interval_changed} show the same block with moved times')

print('=== result')
print(f"{'PASS' if not failures else 'FAIL'}: {len(failures)} failed")
sys.exit(1 if failures else 0)
