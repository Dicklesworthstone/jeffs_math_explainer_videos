#!/usr/bin/env python3
"""Independent checks for the logarithm caption placement review. Run from the repository root.

    python3 -I -B <bundle>/check_review.py MAPS

MAPS is the folder holding LavenderPike's cue_max/NNN.png maps and cues.json. It is only read, and may also hold their
measured-0.json. No decode, no render, no YouTube access: the inputs are files that already exist. Writes outputs/, which
must not exist. Cue readings in inputs/reading.tsv are a person's judgment from stills; this script only attaches times.
"""
import hashlib
import json
from pathlib import Path
import re
import sys
import textwrap

import numpy as np
from PIL import Image, ImageDraw, ImageFont

HERE = Path(__file__).resolve().parent
P = HERE.parent
OUT = HERE / 'outputs'
EL = Path('narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104')
O = EL / 'remote-mac-mini-max01'
JOB = O / 'css-overlay01/jobs/logarithm-overlay-css-01'
ROBOT = Path('narration_lab/third-wave-20261024/root/elevenlabs-robot-20261028')
C = P / 'logarithm-caption-safe-overlay01'
SIDE = P / 'logarithm-caption-timing-repair01/sidecars-timing01'
SETTING = 'line:2% position:50% align:center'
ARIAL = '/System/Library/Fonts/Supplemental/Arial.ttf'
SEALED = {'captions.en.srt': 'b2914c6fdb517f6316117b539ff84a1f64ded7f0a44d29fceea79a01cb20cb22',
          'captions.en.vtt': 'a2fb0d69b99652b9457fe39bf724f7affc64296674ea2e3a08265d44c0a64397'}
PAD_EM, INK_MAP, INK_CLEAN, FLOOR = 0.5, 34, 55, 130
SHIFTS = (-60, -30, 0, 30, 60, 90, 120, 146)
failed = []


def check(name, ok, detail=''):
    print(('PASS ' if ok else 'FAIL ') + name + (': ' + str(detail) if detail != '' else ''), flush=True)
    if not ok:
        failed.append(name)


def note(text):
    print('MEASURE ' + text, flush=True)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def ms(stamp):
    h, m, s = stamp.replace(',', '.').split(':')
    sec, frac = s.split('.')
    return (int(h) * 3600 + int(m) * 60 + int(sec)) * 1000 + int(frac)


def clock(value):
    return f'{int(value // 60):02}:{value % 60:04.1f}'


def read_srt(path):
    cues = []
    for block in Path(path).read_text().strip().split('\n\n'):
        lines = block.split('\n')
        a, b = lines[1].split(' --> ')
        cues.append({'id': int(lines[0]), 'a': a.replace(',', '.'), 'b': b.replace(',', '.'), 's': ms(a) / 1000, 'e': ms(b) / 1000, 'lines': lines[2:]})
    return cues


def read_vtt(raw):
    head, *blocks = raw.rstrip('\n').split('\n\n')
    cues = []
    for block in blocks:
        lines = block.split('\n')
        found = re.fullmatch(r'(\d\d:\d\d:\d\d\.\d{3}) --> (\d\d:\d\d:\d\d\.\d{3})(?: (.*))?', lines[0])
        assert found, lines[0]
        cues.append({'a': found[1], 'b': found[2], 'settings': found[3], 'lines': lines[1:]})
    return head, cues


def same_cues(base, other):
    """Equal times and text, cue by cue, ignoring settings."""
    return len(base) == len(other) and all(x['a'] == y['a'] and x['b'] == y['b'] and x['lines'] == y['lines'] for x, y in zip(base, other))


def main():
    assert len(sys.argv) == 2, __doc__
    maps = Path(sys.argv[1])
    OUT.mkdir()
    (OUT / 'sheets').mkdir()
    reference = json.loads((O / 'youtube-caption-reference01.json').read_text())
    summary = json.loads((JOB / 'overlay-summary.json').read_text())
    positions = json.loads((C / 'positions02.json').read_text())
    placements = json.loads((C / 'placements03.json').read_text())
    plan = json.loads((C / 'plan02.json').read_text())
    moved = sorted(cue for row in positions['windows'] for cue in range(row['first'], row['last'] + 1))
    moved_set = set(moved)

    print('--- reviewed bytes (the candidate is unsealed)')
    for name in ('HANDOFF.md', 'candidate02.en.vtt', 'candidate02.ass', 'candidate03.ass', 'placements03.json', 'positions02.json', 'plan02.json', 'overlay_css02.py', 'checks/make_candidate02.py', 'checks/run_checks02.py'):
        print(name, sha(C / name))
    print('youtube-caption-reference01.json', sha(O / 'youtube-caption-reference01.json'))
    digest = hashlib.sha256()
    for path in [maps / 'cues.json', *sorted((maps / 'cue_max').iterdir())]:
        digest.update(path.read_bytes())
    print('maps: cues.json plus', len(list((maps / 'cue_max').iterdir())), 'pngs, rolled sha256', digest.hexdigest())

    print('--- A. root measurement')
    W, H = reference['player_css_width'], reference['player_css_height']
    top, bottom = reference['caption_top_relative_px'], reference['caption_bottom_relative_px']
    check('A1 16:9 player, so player and frame fractions agree', W * 9 == H * 16, f'{W}x{H}')
    check('A2 block height is two lines', bottom - top == reference['caption_css_height'] == 2 * reference['line_css_height'])
    check('A3 the mail\'s ratios', [round(x, 3) for x in (reference['font_css_px'] / H * 100, reference['line_css_height'] / H * 100, reference['caption_css_width'] / W * 100, top / H, bottom / H)] == [4.444, 5.217, 42.826, 0.808, 0.912])
    scale = 2160 / H
    em, pitch, shown = reference['font_css_px'] * scale, reference['line_css_height'] * scale, bottom * scale
    note(f'4K em {em:.1f}, pitch {pitch:.1f}, block rows {top * scale:.1f} to {shown:.1f}, scene y {(1080 - top * scale) / 270:.2f} to {(1080 - shown) / 270:.2f}')
    note(f'gap under the block {H - bottom:.1f} css px; raw rect, left edge, family and background are not in the record')
    robot = read_srt(ROBOT / 'final-picture01/lie_robotics.en.srt')
    receipt = json.loads((ROBOT / 'publication-receipt01.json').read_text())
    sample = ['Its job is to put its gripper at a pose, a', 'position and an orientation together.']
    live = [c for c in robot if c['lines'] == sample]
    check('A4 Robot SRT is the published one', sha(ROBOT / 'final-picture01/lie_robotics.en.srt') == receipt['replacement']['assets']['captions']['sha256'])
    check('A5 root\'s text is one cue with the file\'s line break, up during the second shown as 0:50', len(live) == 1 and reference['paused_seconds'] <= live[0]['s'] < reference['paused_seconds'] + 1 < live[0]['e'], f"starts {live[0]['s']:.3f} s")
    check('A6 that SRT has no tags, so this is default placement', not re.search(r'<[^>]+>|\{\\', (ROBOT / 'final-picture01/lie_robotics.en.srt').read_text()))

    print('--- B. libass proxy, real first-run boxes')
    unit = ImageFont.truetype(ARIAL, 1000)
    fits, rates = [], []
    for row in summary['samples']:
        x0, _, x1, _ = row['caption_box']
        widest = max(row['lines'], key=lambda line: unit.getlength(line))
        fits.append((x1 - x0 - 16) / (unit.getlength(widest) / 1000))
        rates.append((x1 - x0) / len(max(row['lines'], key=len)))
    proxy_em = float(np.median(fits))
    two = [r['caption_box'] for r in summary['samples'] if len(r['lines']) == 2]
    one = [r['caption_box'] for r in summary['samples'] if len(r['lines']) == 1]
    proxy_pitch = min(b[1] for b in one) - min(b[1] for b in two)
    note(f'em fit {proxy_em:.1f} (range {min(fits):.1f} to {max(fits):.1f}), {proxy_em / 21.6:.2f}% of height, {proxy_em / em:.3f} times YouTube')
    note(f'line pitch {proxy_pitch}, {proxy_pitch / pitch:.3f} times YouTube; nominal 16/288 against 46/1035 would say {(16 / 288) / (46 / 1035):.3f}')
    note(f'two-line ink rows {min(b[1] for b in two)} to {max(b[3] for b in two)}; layout rows about 1845 to 2085; widest px per char {max(rates):.1f}')
    arial46 = ImageFont.truetype(ARIAL, 46)
    width = arial46.getlength(sample[0]) + PAD_EM * 46
    note(f"Arial 46 plus {PAD_EM} em gives {width:.1f} css px for root's line, measured {reference['caption_css_width']}; one line only, family unrecorded")
    note(f'Arial normal line height at 46 would be {sum(arial46.getmetrics())} px, measured {reference["line_css_height"]}')

    print('--- C. candidate VTT against the sealed sidecars')
    base_raw = (SIDE / 'captions.en.vtt').read_text()
    cand_raw = (C / 'candidate02.en.vtt').read_text()
    _, base = read_vtt(base_raw)
    head, cand = read_vtt(cand_raw)
    srt = read_srt(SIDE / 'captions.en.srt')
    native = read_srt(O / 'jobs/logarithm-remote-render-work01/logarithm_mysteries.en.srt')
    for name, digest_value in SEALED.items():
        check('C1 sealed ' + name, sha(SIDE / name) == digest_value)
    check('C2 plan binds both sealed files', plan['base_srt_sha256'] == SEALED['captions.en.srt'] and plan['base_vtt_sha256'] == SEALED['captions.en.vtt'])
    check('C3 header and 475 cues', head == 'WEBVTT' and len(cand) == 475)
    check('C4 every time and text equals the sealed VTT', same_cues(base, cand))
    check('C5 every time and text equals the sealed SRT', same_cues(srt, cand))
    holds = [ms(c['b']) - ms(c['a']) for c in cand]
    check('C6 every hold equals the sealed VTT', holds == [ms(c['b']) - ms(c['a']) for c in base], f'min {min(holds)} ms, max {max(holds)} ms')
    check('C7 removing the one setting gives the sealed bytes', cand_raw.replace(' ' + SETTING, '') == base_raw)
    check('C8 the setting sits on exactly the 116 window cues', [n for n, c in enumerate(cand, 1) if c['settings'] == SETTING] == moved and {c['settings'] for c in cand} == {None, SETTING} and len(moved) == 116)
    check('C9 no cue needs VTT escaping', not any(re.search(r'[<>&]|-->', ' '.join(c['lines'])) for c in cand))
    edges = [(k, key) for row in positions['windows'] for k, key in ((row['first'], 'a'), (row['last'], 'b')) if srt[k - 1][key] != native[k - 1][key]]
    check('C10 no window edge moved between native and retimed', not edges, f"{2 * len(positions['windows'])} edges")
    faulty_text = [dict(c) for c in cand]
    faulty_text[99] = {**faulty_text[99], 'lines': [faulty_text[99]['lines'][0] + 'x', *faulty_text[99]['lines'][1:]]}
    faulty_time = [dict(c) for c in cand]
    faulty_time[99] = {**faulty_time[99], 'b': '00:00:00.001'}
    check('C11 planted faults are caught: one letter, one time', not same_cues(base, faulty_text) and not same_cues(base, faulty_time))
    for name in ('candidate02.ass', 'candidate03.ass'):
        rows = [line for line in (C / name).read_text().splitlines() if line.startswith('Dialogue:')]
        worst = 0
        for row, c in zip(rows, cand):
            parts = row.split(',', 9)
            worst = max(worst, abs(ms(parts[1] + '0') - ms(c['a'])), abs(ms(parts[2] + '0') - ms(c['b'])))
        check(f'C12 {name} times within 5 ms of the VTT, 475 events', len(rows) == 475 and worst <= 5, f'worst {worst} ms')
    margins = [min(row['frame'] * 1000 / 60 - ms(cand[row['cue'] - 1]['a']), ms(cand[row['cue'] - 1]['b']) - row['frame'] * 1000 / 60) for row in plan['samples']]
    check('C13 sample frames clear their cue edges by more than the rounding', min(margins) > 5, f'smallest {min(margins):.1f} ms')

    print('--- D. shared model and the 23 clean frames')
    font = ImageFont.truetype(ARIAL, round(em))
    pad = PAD_EM * em

    def rects(lines, low=None, high=None, cx=1920.0):
        n = len(lines)
        y = high if high is not None else low - n * pitch
        return [(cx - (font.getlength(line) + pad) / 2, y + i * pitch, cx + (font.getlength(line) + pad) / 2, y + (i + 1) * pitch) for i, line in enumerate(lines)]

    def full(mask, boxes):
        return sum(int(mask[max(int(b), 0):min(int(d), 2160), max(int(a), 0):min(int(c), 3840)].sum()) for a, b, c, d in boxes)

    def small(mask, boxes):
        return sum(int(mask[max(int(b / 4), 0):min(int(np.ceil(d / 4)), 540), max(int(a / 4), 0):min(int(np.ceil(c / 4)), 960)].sum()) for a, b, c, d in boxes)

    placed = {cue: row for row in placements for cue in range(row['first'], row['last'] + 1)}
    frames, clean_rows, contained, faint, nearby = {}, [], [], [], []

    def map_of(k):
        with Image.open(maps / 'cue_max' / f'{k:03d}.png') as image:
            return np.asarray(image.convert('L'))

    for row in summary['samples']:
        with Image.open(JOB / 'clean' / (row['id'] + '.png')) as image:
            rgb = image.convert('RGB')
        frames[row['id']] = rgb
        gray = np.asarray(rgb.convert('L'))
        mask = gray > INK_CLEAN
        x0, y0, x1, y1 = row['caption_box']
        spot = placed[row['cue']]
        spot = spot.get('none', spot)
        wrapped = row['lines'] if spot['wrap'] == 'full' else textwrap.wrap(' '.join(row['lines']), width=int((int(spot['wrap'][1:]) - 60) / 50.9))
        clear = [b for b in range(1600, 2161, 2) if full(mask, rects(row['lines'], low=b)) == 0]
        rect = spot['rect']
        clean_rows.append({'id': row['id'], 'cue': row['cue'], 'proxy': int(mask[y0:y1, x0:x1].sum()), 'measured': full(mask, rects(row['lines'], low=shown)),
                           'clear': (max(c for c in clear if c <= shown + 2) if [c for c in clear if c <= shown + 2] else None), 'line2': full(mask, rects(row['lines'], high=43.2)),
                           'rect': int(mask[rect[1]:rect[3], rect[0]:rect[2]].sum()), 'real_at_rect': full(mask, rects(wrapped, high=rect[1], cx=(rect[0] + rect[2]) / 2))})
        own = map_of(row['cue'])
        around = np.maximum(np.maximum(map_of(max(row['cue'] - 1, 1)), own), map_of(min(row['cue'] + 1, 475))) >= INK_MAP
        shrunk = np.asarray(rgb.convert('L').resize((960, 540), Image.BOX))
        clear_ink = shrunk >= 60
        contained.append(float(((own >= INK_MAP) & clear_ink).sum()) / max(int(clear_ink.sum()), 1))
        nearby.append(float((around & clear_ink).sum()) / max(int(clear_ink.sum()), 1))
        faint.append(float(((own >= INK_MAP) & (shrunk >= INK_MAP)).sum()) / max(int((shrunk >= INK_MAP).sum()), 1))
    check('D1 positive control: ink in the proxy box in all 23, as root saw', all(r['proxy'] > 0 for r in clean_rows))
    check('D2 the maps hold the master: clear ink (L 60+) of each clean frame is in its own or a neighbouring cue map', min(nearby) >= 0.999, f'own map alone: {sum(c >= 0.999 for c in contained)} of 23 complete, smallest {min(contained):.3f}')
    note(f'D2 detail: down to L {INK_MAP} the own-map share falls to {min(faint):.3f} (S16, 0.05 s into its cue, before its first 2 fps sample); S16 is the map limit, reported in the review')
    check('D3 candidate B rectangles are near clean on the local frames', max(r['rect'] for r in clean_rows) <= 8 and max(r['real_at_rect'] for r in clean_rows) == 0, f"at most {max(r['rect'] for r in clean_rows)} px")
    note(f"line:2% at measured size has ink on {sum(r['line2'] > 0 for r in clean_rows)} of 23 frames, {min(r['line2'] for r in clean_rows)} to {max(r['line2'] for r in clean_rows)} px")
    note('measured box, 4K ink px per frame: ' + ' '.join(f"{r['id']}={r['measured']}" for r in clean_rows))
    (OUT / 'clean-frames.tsv').write_text('frame\tcue\tink_proxy_box\tink_measured_block\tlowest_clear_bottom_row_at_or_above_measured\tink_at_line2pct\tink_candidate_b_rect\n' + ''.join(
        f"{r['id']}\t{r['cue']}\t{r['proxy']}\t{r['measured']}\t{r['clear'] if r['clear'] else ''}\t{r['line2']}\t{r['rect']}\n" for r in clean_rows))

    print('--- E. candidate proxies against the geometry')
    note(f'proxy A top row {2 / 288 * 2160:.0f}; line:2% top row {0.02 * 2160:.1f}; two lines at measured size end at row {43.2 + 2 * pitch:.1f}')
    written = [line for line in (C / 'candidate03.ass').read_text().splitlines() if line.startswith('Dialogue:')]
    overflow = []
    for k, line in enumerate(written, 1):
        if k in placed:
            spot = placed[k].get('none', placed[k])
            count = line.split('}', 1)[1].count('\\N') + 1
            if count > spot['lines']:
                overflow.append((k, count, spot['lines'], spot['rect'], clock(srt[k - 1]['s']), clock(srt[k - 1]['e'])))
    check('E1 FINDING: exactly one candidate B cue overflows its declared lines', overflow == [(239, 5, 4, [2224, 1184, 3328, 1680], '15:45.7', '15:50.9')], overflow)

    print('--- F. all 475 cues on the existing maps, retimed intervals')
    old = {c['id']: c for c in json.loads((maps / 'cues.json').read_text())}
    check('F1 map intervals are the native cues', len(old) == 475 and all(abs(old[c['id']]['a'] - c['s']) < 1e-6 and abs(old[c['id']]['b'] - c['e']) < 1e-6 for c in native))
    pictures, reach = {}, 0
    spans = {k: (round(q['a'] * 1000), round(q['b'] * 1000)) for k, q in old.items()}  # integer ms: abutting cues must not count as overlapping
    for c in srt:
        ids = [k for k, (a, b) in spans.items() if ms(c['a']) < b and a < ms(c['b'])]
        reach = max(reach, max(abs(k - c['id']) for k in ids))
        stack = None
        for k in ids:
            now = map_of(k)
            stack = now if stack is None else np.maximum(stack, now)
        pictures[c['id']] = stack
    check('F2 a retimed cue reaches at most one neighbouring map', reach == 1)
    ink = {c['id']: {shift: small(pictures[c['id']] >= INK_MAP, rects(c['lines'], low=shown + shift)) for shift in SHIFTS} for c in srt}
    for shift in SHIFTS:
        hit = [k for k, row in ink.items() if row[shift] >= FLOOR]
        note(f'bottom row {shown + shift:.0f} (shift {shift:+d}): {len(hit)} cues flagged, {sum(srt[k - 1]["e"] - srt[k - 1]["s"] for k in hit):.1f} s')
    flagged = {k for k, row in ink.items() if row[0] >= FLOOR}
    tiny = sum(0 < row[0] < FLOOR for row in ink.values())
    note(f'floor {FLOOR} map px at shift 0: {len(flagged & moved_set)} of the moved 116 flagged, {len(flagged - moved_set)} flagged outside it, {len(moved_set - flagged)} of the 116 not flagged; {tiny} cues under the floor were not read')
    flat = reference['caption_css_width'] * scale / len(sample[0]) * 1.05
    ratio = [(font.getlength(max(c['lines'], key=len)) + pad) / (len(max(c['lines'], key=len)) * flat) for c in srt]
    note(f'flat {flat:.1f} px per char is narrower than Arial plus padding on {sum(r > 1 for r in ratio)} of 475 cues, median ratio {float(np.median(ratio)):.3f}')
    long_lines = [font.getlength(line) / len(line) for c in srt for line in c['lines'] if len(line) >= 30]
    note(f'Arial px per char, lines of 30+ chars: median {float(np.median(long_lines)):.1f}; the Robot line {font.getlength(sample[0]) / len(sample[0]):.1f}')
    peers = {}
    if (maps / 'measured-0.json').exists():
        peers = {row['id']: row['count'] for row in json.loads((maps / 'measured-0.json').read_text())}
        theirs = {k for k, v in peers.items() if v >= FLOOR}
        print('measured-0.json', sha(maps / 'measured-0.json'))
        note(f'mine {len(flagged)}, Lavender {len(theirs)}, both {len(flagged & theirs)}, only mine {sorted(flagged - theirs)}, only Lavender {sorted(theirs - flagged)}')

    print('--- G. reading table: times attached to the judgment in inputs/reading.tsv')
    reading = [line.split('\t') for line in (HERE / 'inputs/reading.tsv').read_text().splitlines()[1:]]
    union = flagged | {k for k, v in peers.items() if v >= FLOOR}
    classes, rows = {}, []
    for first, last, kind, text in reading:
        cues = [k for k in range(int(first), int(last) + 1) if k in union]
        assert cues, first
        seconds = sum(srt[k - 1]['e'] - srt[k - 1]['s'] for k in cues)
        rows.append(f"{clock(srt[cues[0] - 1]['s'])}\t{clock(srt[cues[-1] - 1]['e'])}\t{cues[0]}\t{cues[-1]}\t{len(cues)}\t{seconds:.1f}\t{kind}\t{text}\n")
        for k in cues:
            classes[k] = kind
    (OUT / 'obstructions.tsv').write_text('start\tend\tfirst_cue\tlast_cue\tflagged_cues\tseconds\tclass\ton_screen\n' + ''.join(rows))
    for kind in 'TLE':
        cues = [k for k, v in classes.items() if v == kind]
        note(f'class {kind}: {len(cues)} cues, {sum(srt[k - 1]["e"] - srt[k - 1]["s"] for k in cues):.1f} s, in {sum(1 for row in reading if row[2] == kind)} spots; {len(set(cues) & moved_set)} are in the moved 116')
    note(f'flagged but read as thin lines or diagram strokes: {len(union - set(classes))} cues')
    outside = sorted(k for k, v in classes.items() if v == 'T' and k not in moved_set)
    note(f'class T cues Candidate A would leave at the bottom: {outside}')
    (OUT / 'cue-conflicts.tsv').write_text('cue\tstart\tend\tlines\tcandidate_a_moves\t' + '\t'.join(f'ink_shift_{s:+d}' for s in SHIFTS) + '\tlavender_ink\treading\ttext\n' + ''.join(
        f"{c['id']}\t{c['a']}\t{c['b']}\t{len(c['lines'])}\t{int(c['id'] in moved_set)}\t" + '\t'.join(str(ink[c['id']][s]) for s in SHIFTS) + f"\t{peers.get(c['id'], 0)}\t{classes.get(c['id'], 'G' if c['id'] in union else '')}\t{' '.join(c['lines'])}\n" for c in srt))

    label = ImageFont.truetype(ARIAL, 22)
    tiny_label = ImageFont.truetype(ARIAL, 15)
    samples = summary['samples']
    for sheet, start in enumerate(range(0, len(samples), 8), 1):
        group = samples[start:start + 8]
        canvas = Image.new('RGB', (1280, 8 + 205 * len(group)), (40, 40, 40))
        draw = ImageDraw.Draw(canvas)
        y = 4
        for row in group:
            canvas.paste(frames[row['id']].crop((0, 1640, 3840, 2160)).resize((1280, 173), Image.LANCZOS), (0, y + 26))
            a, b, c, d = row['caption_box']
            draw.rectangle(((a / 3, (b - 1640) / 3 + y + 26), (c / 3, (d - 1640) / 3 + y + 26)), outline=(255, 160, 40), width=1)
            for a, b, c, d in rects(row['lines'], low=shown):
                draw.rectangle(((a / 3, (b - 1640) / 3 + y + 26), (c / 3, (d - 1640) / 3 + y + 26)), outline=(60, 230, 255), width=2)
            draw.text((6, y + 1), f"{row['id']} cue {row['cue']} {clock(row['frame'] / 60)}  cyan measured, controls visible; orange proxy box. Outlines only, not YouTube.", font=label, fill=(255, 255, 255))
            y += 205
        canvas.save(OUT / 'sheets' / f'clean-frames-{sheet}.jpg', quality=88)
    ids = sorted(union)
    tiles = []
    for k in ids:
        frame = Image.fromarray(pictures[k]).convert('RGB')
        draw = ImageDraw.Draw(frame)
        for a, b, c, d in rects(srt[k - 1]['lines'], low=shown):
            draw.rectangle((a / 4, b / 4, c / 4, d / 4), outline=(60, 230, 255))
        tile = frame.crop((200, 428, 760, 540))
        ImageDraw.Draw(tile).text((2, 0), f"{k} {clock(srt[k - 1]['s'])}-{clock(srt[k - 1]['e'])} ink {ink[k][0]}", fill=(120, 255, 120), font=tiny_label)
        tiles.append(tile)
    for part in range((len(tiles) + 23) // 24):
        group = tiles[part * 24:(part + 1) * 24]
        sheet = Image.new('RGB', (1126, ((len(group) + 1) // 2) * 116), (70, 70, 70))
        for n, tile in enumerate(group):
            sheet.paste(tile, ((n % 2) * 566, (n // 2) * 116))
        sheet.save(OUT / 'sheets' / f'cue-maps-{part + 1}.jpg', quality=90)
    note(f'{len(tiles)} cue tiles on {(len(tiles) + 23) // 24} sheets')
    print('RESULT ' + ('PASS' if not failed else 'FAIL ' + ', '.join(failed)))
    raise SystemExit(1 if failed else 0)


if __name__ == '__main__':
    main()
