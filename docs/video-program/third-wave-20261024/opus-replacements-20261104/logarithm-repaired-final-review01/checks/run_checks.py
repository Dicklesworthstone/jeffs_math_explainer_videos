"""Every number REVIEW.md quotes, recomputed from the delivered files. Read-only.

Run from anywhere with `python3 -I -B`. Needs numpy and PIL for the join check.
"""
import csv
import difflib
import hashlib
import json
import pathlib
import re
import statistics

import numpy as np
from PIL import Image

R = pathlib.Path(__file__).resolve().parents[6]
P = R / 'docs/video-program/third-wave-20261024/opus-replacements-20261104'
EL = R / 'narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104'
O = EL / 'remote-mac-mini-max01'
V = O / 'jobs/logarithm-remote-verify01'
W = O / 'jobs/logarithm-remote-render-work01'
M = O / 'jobs/logarithm-remote-master01'
A = O / 'jobs/logarithm-remote-asr02'
S = EL / 'final-picture-repair01'
C = S / 'workspace/explainer_corpus/logarithm_mysteries'
D = P / 'logarithm-publication-draft06'
THUMB = R / 'narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-stage09-thumbnail01/output/logarithm_mysteries_thumbnail.png'

failures = []


def check(name, ok, detail=''):
    print(f"{'PASS' if ok else 'FAIL'}  {name}" + (f'  [{detail}]' if detail else ''))
    if not ok:
        failures.append(name)


def sha(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1 << 20), b''):
            h.update(chunk)
    return h.hexdigest()


def rel(path):
    return str(path.relative_to(R))


def seconds(t):
    h, m, s = t.replace(',', '.').split(':')
    return int(h) * 3600 + int(m) * 60 + float(s)


v = json.loads((V / 'verification.json').read_text())
print('=== 1. identities, measured now')
EXPECT = {
    W / 'logarithm_mysteries_master.mp4': '8534ec211790fb8400b809de19a6d1cb75af6150a5e90566f6a93bb90e7efd69',
    W / 'logarithm_mysteries.en.srt': 'c9321cd7a2912d4457f2dc617735fd06f0b2fc7d684442815fa46b989bc31b7c',
    W / 'logarithm_mysteries.en.vtt': '45833988985413f3cfd8c44d5e18959be47ff5003d0dcbc570ce0b74ddfc0b45',
    V / 'verification.json': 'f40e3999cf4f3400705ee13e505d56900326ec648d6ac9403c5a6bf29f1ffc20',
    V / 'strict-video-verification.log': '57be45b2920a750bf0b19ae52330d2cb0593736108f45f53fea29eb611fe9109',
    M / 'master-summary.json': '0d696cacbb84295c1962bd3f32d97d074b9ff3abf77592a9cab8f1d8606d9ced',
    S / 'integration.json': '8f7c4a9f45c4dd256536d8d5992bd14ebef44ac4fc80e5563f84c9de2bce5952',
    THUMB: '0fed0895987e9d3e5e64ff4067c8a40c223a437be88b8c5a1f93ecd1533209ee',
}
for path, want in EXPECT.items():
    got = sha(path)
    check(path.name, got == want, got[:16])
pack = json.loads((O / 'delivery-package01.json').read_text())
on_disk = {f['kind']: f for f in pack['files']}
check('delivery package lists 8 assets', len(pack['files']) == 8)
check('delivery package master hash', on_disk['master']['sha256'] == EXPECT[W / 'logarithm_mysteries_master.mp4'])
check('delivery package thumbnail hash', on_disk['thumbnail']['sha256'] == EXPECT[THUMB])
for kind, name in (('title', 'title.txt'), ('description', 'description.txt'), ('tags', 'tags.txt'), ('chapter_times', 'chapter-times03.json')):
    check(f'delivery package {kind} is draft06', on_disk[kind]['sha256'] == sha(D / name))
print('package status', pack['status'], 'uploaded', pack['uploaded'], 'publication_accepted', pack['publication_accepted'])

print('=== 2. terminal receipts')
for job in (V, M):
    r = json.loads((job / 'resource-receipt.json').read_text())
    check(f'{job.name} exit 0', r['exit'] == 0 and r['final_inputs_sha256'] == r['inputs_sha256'], r['finished'])
log = (V / 'strict-video-verification.log').read_text()
check('strict verifier printed VERIFY PASS', 'VERIFY PASS' in log)
for line in log.splitlines():
    if line.startswith(('loudness', 'A/V', 'a/v')) or 'decode' in line.lower() and 'log retained' not in line:
        print('  ', line[:110])
prior = (EL / 'verify-after-stage09-master02.log').read_text()
check('prior master had the same -20.1 LUFS and -1.5 dBTP', '"integrated_lufs": "-20.1"' in prior and '"true_peak_dbtp": "-1.5"' in prior)
recorded = v['sheet_sha256']
pairs = recorded.items() if isinstance(recorded, dict) else zip(v['sheets'], recorded)
bad = sum(sha(V / pathlib.Path(name).name) != want for name, want in pairs)
check('all contact sheets equal their recorded hashes', bad == 0, f"{len(v['sheets'])} sheets, {len(v['pictures'])} pictures")

print('=== 3. source bindings')
ints = json.loads((S / 'integration.json').read_text())
final = {pathlib.Path(f['relative_path']).name: f for f in ints['files']}
changes = v['source_changes']
items = changes if isinstance(changes, list) else [dict(x, path=k) for k, x in changes.items()]
print('source changes recorded', len(items))
for c in items:
    name = pathlib.Path(c.get('path', c.get('relative_path', c.get('file', '')))).name
    disk = sha(C / 'src/chapters' / name) if (C / 'src/chapters' / name).exists() else sha(C / 'src' / name)
    want = c.get('sha256', c.get('final_sha256', c.get('after_sha256')))
    check(f'{name} on disk equals verification', disk == want, disk[:12])
    check(f'{name} on disk equals integration', disk == final[name]['sha256'])

print('=== 4. captions')


def blocks(path, vtt=False):
    text = path.read_text()
    if vtt:
        text = text.split('\n\n', 1)[1]
    out = []
    for block in text.strip().split('\n\n'):
        lines = block.split('\n')
        i = next(k for k, line in enumerate(lines) if ' --> ' in line)
        a, b = lines[i].split(' --> ')
        out.append((seconds(a), seconds(b.split()[0]), lines[i + 1:]))
    return out


srt = blocks(W / 'logarithm_mysteries.en.srt')
vtt = blocks(W / 'logarithm_mysteries.en.vtt', True)
check('475 blocks', len(srt) == 475, str(len(srt)))
check('SRT equals VTT', srt == vtt)
check('no overlap, positive durations', all(b > a for a, b, _ in srt) and all(x[1] <= y[0] for x, y in zip(srt, srt[1:])))
check('at most 2 lines of 42 characters', all(len(t) <= 2 and all(len(x) <= 42 for x in t) for _, _, t in srt))
check('ASCII only', all(x.isascii() for _, _, t in srt for x in t))
print('shortest', round(min(b - a for a, b, _ in srt), 2), 'fastest cps', round(max(sum(map(len, t)) / (b - a) for a, b, t in srt), 1))
rows = {}
with (C / 'script.tsv').open(newline='') as stream:
    for row in csv.reader(stream, delimiter='\t', quoting=csv.QUOTE_NONE):
        if row and not row[0].startswith('#'):
            rows[row[0]] = row[1]
norm = lambda x: re.sub(r'\s+', ' ', x).strip()
spans = [(p['cue'], p['start_frame'] / 60) for p in v['placements']]
diff = []
for i, (cue, a) in enumerate(spans):
    b = spans[i + 1][1] if i + 1 < len(spans) else 1e9
    got = ' '.join(' '.join(t) for x, _, t in srt if a - 0.04 <= x < b - 0.04)
    if norm(got) != norm(rows[cue]):
        diff.append(cue)
check('every cue caption equals script text', not diff, f'{len(spans)} cues')
for cue, p in ((p['cue'], p) for p in v['pause_sites'] if p['kind'] == 'reading'):
    a, b = p['start_frame'] / 60, p['think_end_frame'] / 60
    inside = [x for x in srt if x[0] < b and x[1] > a]
    check(f'no caption during reading hold {cue}', not inside, f'{a:.2f}-{b:.2f}')

print('=== 5. mid-sentence caption lag (word onset minus block start, recognizer estimate)')
text = [(a, b, ' '.join(t)) for a, b, t in srt]
words = []
for x in json.loads((A / 'full-large.json').read_text())['result']['segments']:
    t = re.sub(r"[^a-z0-9']", '', x['text'].lower())
    if t:
        words.append((t, x['start_sec'], x['end_sec']))
cw = []
for n, (_, _, line) in enumerate(text):
    for i, word in enumerate(line.split()):
        t = re.sub(r"[^a-z0-9']", '', word.lower())
        if t:
            cw.append((t, n, i))
sm = difflib.SequenceMatcher(None, [w[0] for w in cw], [w[0] for w in words], autojunk=False)
match = {}
for a, b, n in sm.get_matching_blocks():
    for k in range(n):
        match[a + k] = b + k
mid = []
for i, (t, n, pos) in enumerate(cw):
    if pos or i not in match or i - 1 not in match or n == 0:
        continue
    if text[n - 1][2].rstrip().endswith(('.', '?', '!', ':', ';')):
        continue
    w, before = words[match[i]], words[match[i - 1]]
    if w[1] - before[2] > 0.05:
        continue
    mid.append((round(w[1] - text[n][0], 2), n + 1, t, text[n][2][:44]))
mid.sort()
vals = [x[0] for x in mid]
print('blocks measured', len(vals), 'median', statistics.median(vals), 'p10', vals[len(vals) // 10], 'p90', vals[len(vals) * 9 // 10])
print('late by over 0.5 s', sum(x < -0.5 for x in vals), 'over 0.75 s', sum(x < -0.75 for x in vals), 'over 1 s', sum(x < -1 for x in vals))
for row in mid[:11]:
    print('  ', row)

print('=== 6. chapters and publication text')
ct = json.loads((D / 'chapter-times03.json').read_text())
desc = (D / 'description.txt').read_text()
frames = 0
for c, r in zip(ct['chapters'], v['chapters']):
    stamp = f"{int(c['frame_start'] / 60) // 60}:{int(c['frame_start'] / 60) % 60:02d}"
    check(f"{c['timestamp']} {c['title']}", c['frame_start'] == r['start_frame'] == frames and c['timestamp'] == stamp and f"{stamp} {c['title']}\n" in desc)
    frames += c['frames']
check('chapter frames sum to the master', frames == ct['total_frames'] == 116665)
check('chapter file names this master', ct['master_sha256'] == EXPECT[W / 'logarithm_mysteries_master.mp4'])
print('title', len((D / 'title.txt').read_text().strip()), 'description', len(desc.strip()), 'tags', len((D / 'tags.txt').read_text().strip()))
check('no angle brackets in the description', '<' not in desc and '>' not in desc)
print('recap caption', [(round(a, 2), ' '.join(t)) for a, _, t in srt if ' '.join(t).startswith('Look back along')])
sums = dict(reversed(line.split()) for line in (D / 'SHA256SUMS').read_text().splitlines())
check('draft06 files equal its SHA256SUMS', all(sha(D / k) == x for k, x in sums.items()))

print('=== 7. every chapter join: picture ink before and after, from the sheets')


def ink(i):
    im = Image.open(V / f'sheet-{i // 4 + 1:03d}.jpg').convert('RGB')
    w, h = im.size
    k = i % 4
    box = ((k % 2) * w // 2, (k // 2) * h // 2, ((k % 2) + 1) * w // 2, ((k // 2) + 1) * h // 2 - 42)
    return float((np.asarray(im.crop(box)).astype(float).max(axis=2) > 60).mean()) * 100


lows, highs = [], []
for i, p in enumerate(v['pictures']):
    for reason in p['reasons']:
        if re.match(r'join-\d\d-\d\d\.1 ', reason):
            lows.append(ink(i))
        if re.match(r'join-\d\d-\d\d\.2 ', reason):
            highs.append(ink(i))
print('last frame before each of', len(lows), 'joins, ink%', round(min(lows), 2), 'to', round(max(lows), 2))
print('first frame after each join, ink%', round(max(highs), 2))
check('all 12 joins cut from a full picture to an empty one', len(lows) == 12 == len(highs) and max(highs) == 0 and min(lows) > 1)

print()
print('FAILURES', failures)
raise SystemExit(1 if failures else 0)
