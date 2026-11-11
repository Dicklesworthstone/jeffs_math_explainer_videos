"""Local, read-only review aid: compare the actual pictures1008 records (dumped from CSS by dump1008-records.py) with the
pictures985 seeds already reviewed by root and the independent reviewers.

Why images should be byte-identical: verify1005's inputs bind master987/final/KEY.mp4 at the same sha256 as the
final983 chapter file pictures985 decoded (all 15 checked 2026-11-10), pictures1008 selects exactly the 985 frames
(local = global - offset), and its ffmpeg argv equals 985's plus `-frames:v N`, with the same /usr/bin/ffmpeg.

usage: python3 -I -B compare1008-985.py P808 records1008.json [VERIFICATION_SHA256]
       python3 -I -B compare1008-985.py P808 --self-test
Exit 0: all hard checks pass and every picture equals its 985 picture; 2: hard checks pass but some pictures differ
(listed: view those sheets first); 1: a hard check failed."""
import copy
import json
import sys
from pathlib import Path

MOVIE = 'fe33b9e48bcb456b3cad41f7651bd8e5f8e3bb1b7c6deba657e6b4d9e5fa2b80'
D = '/data/projects/jmev-css-render-worker-20261107'


def load_seeds(p808):
    ct = json.loads((p808 / 'publication976/chapter-times982.json').read_text())
    seeds = {}
    for c in ct['chapters']:
        rec = json.loads((p808 / f"completed-pictures985-{c['key'][:2]}.json").read_text())['chapters'][0]
        assert rec['chapter'] == c['key'] and rec['frames'] == c['frames']
        seeds[c['key']] = (c['start_frame'], rec)
    return seeds


def compare(seeds, dump, pin=None):
    hard, differ, lines, vshas = [], [], [], set()
    tot = same = sheets = 0
    for key, (off, s) in seeds.items():
        ch = dump['chapters'].get(key) or {}
        r, rec = ch.get('receipt') or {}, ch.get('record')
        bad = lambda what: hard.append(f'{key}: {what}')
        if not (r.get('admitted') is True and r.get('finished') and r.get('exit') == 0 and not r.get('resource_stop') and not r.get('error')
                and r.get('inputs_sha256') == r.get('final_inputs_sha256')):
            bad(f'job receipt not a stable success {r}')
        if rec is None:
            bad('no pictures1008.json')
            continue
        es = rec.get('extract_source') or {}
        if rec.get('chapter') != key or rec.get('movie_sha256') != MOVIE:
            bad('chapter or master sha256')
        if (es.get('path'), es.get('sha256'), es.get('offset_frames'), es.get('frames')) != (f'{D}/jobs/css-exponential-master987/final/{key}.mp4', s['movie_sha256'], off, s['frames']):
            bad(f"extract_source {es.get('path')} {str(es.get('sha256'))[:8]} offset {es.get('offset_frames')} frames {es.get('frames')} != 985 chapter {s['movie_sha256'][:8]} offset {off} frames {s['frames']}")
        vshas.add(rec.get('verification_sha256'))
        pics, seedp = rec.get('pictures') or [], s['pictures']
        if len(pics) != len(seedp) or rec.get('samples') != len(seedp):
            bad(f'{len(pics)} pictures, samples {rec.get("samples")}, 985 has {len(seedp)}')
        for i, (a, b) in enumerate(zip(pics, seedp)):
            if (a.get('local_frame'), a.get('frame'), a.get('reasons')) != (b['frame'], off + b['frame'], b['reasons']):
                bad(f"picture {i + 1}: local {a.get('local_frame')} global {a.get('frame')} != 985 frame {b['frame']} + {off}, or reasons differ")
            if not str(a.get('path', '')).endswith(f'/css-exponential-master-pictures1008-{key}/pictures/frame-{i + 1:06d}.jpg'):
                bad(f"picture {i + 1} path {a.get('path')}")
            if a.get('sha256') == b['sha256']:
                same += 1
            else:
                differ.append(f"{key} picture {i + 1} master frame {a.get('frame')} (local {a.get('local_frame')}, sheet {i // 8 + 1:03})")
        tot += len(pics)
        sh = rec.get('sheets') or []
        sheets += len(sh)
        if [x.get('frames') for x in sh] != [[off + f for f in x['frames']] for x in s['sheets']]:
            bad(f'sheets: {len(sh)} vs 985 {len(s["sheets"])}, or frames not the 985 sheet frames + offset')
        st = rec.get('stream') or {}
        lines.append(f"{key:22} offset {off:6} pictures {len(pics):4} same-as-985 {sum(1 for a, b in zip(pics, seedp) if a.get('sha256') == b['sha256']):4} "
                     f"sheets {len(sh):3} stream matched {bool(st.get('matched'))} tags {st.get('tags')}")
    if len(vshas) != 1 or (pin and vshas != {pin}):
        hard.append(f'verification_sha256 values {sorted(map(str, vshas))} (pin {pin})')
    rc = dump.get('recheck')
    if rc is None:
        hard.append('no recheck job yet')
    else:
        rr, rv = rc.get('receipt') or {}, rc.get('record') or {}
        if not (rr.get('admitted') is True and rr.get('finished') and rr.get('exit') == 0 and not rr.get('resource_stop') and not rr.get('error')):
            hard.append(f'recheck receipt not a stable success {rr}')
        if not (rv.get('records_exact') is True and rv.get('upstream_rows_unchanged') is True and rv.get('samples') == 3163 and rv.get('verification_sha256') in vshas):
            hard.append('recheck1008.json: records_exact/upstream_rows_unchanged/samples 3163/verification sha')
        got = {x.get('chapter'): x.get('record_sha256') for x in rv.get('chapters', [])}
        want = {k: (dump['chapters'].get(k) or {}).get('record_sha256') for k in seeds}
        if got != want:
            hard.append('recheck record_sha256 rows differ from the dumped chapter records')
    lines.append(f'total pictures {tot} (985: 3163), same bytes as 985 {same}, sheets {sheets} (985: 402)')
    if tot != 3163 or sheets != 402:
        hard.append(f'totals {tot}/{sheets} != 3163/402')
    return hard, differ, lines


def synthetic(seeds):
    v = 'f' * 64
    dump = {'chapters': {}, 'recheck': None}
    ok = {'admitted': True, 'finished': 'x', 'exit': 0, 'resource_stop': None, 'error': None, 'inputs_sha256': 'i', 'final_inputs_sha256': 'i'}
    for key, (off, s) in seeds.items():
        job = f'{D}/jobs/css-exponential-master-pictures1008-{key}'
        dump['chapters'][key] = {'job_exists': True, 'receipt': ok, 'record_sha256': key[:2] * 32, 'record': {
            'chapter': key, 'movie_sha256': MOVIE, 'samples': len(s['pictures']), 'verification_sha256': v,
            'extract_source': {'path': f'{D}/jobs/css-exponential-master987/final/{key}.mp4', 'sha256': s['movie_sha256'], 'offset_frames': off, 'frames': s['frames']},
            'pictures': [{'path': f'{job}/pictures/frame-{i + 1:06d}.jpg', 'sha256': b['sha256'], 'frame': off + b['frame'], 'local_frame': b['frame'], 'reasons': b['reasons']} for i, b in enumerate(s['pictures'])],
            'sheets': [{'frames': [off + f for f in x['frames']]} for x in s['sheets']], 'stream': {'matched': True, 'tags': {}}}}
    dump['recheck'] = {'receipt': ok, 'record': {'records_exact': True, 'upstream_rows_unchanged': True, 'samples': 3163, 'verification_sha256': v,
                                                 'chapters': [{'chapter': k, 'record_sha256': k[:2] * 32} for k in seeds]}}
    return dump, v


def self_test(seeds):
    base, v = synthetic(seeds)
    h, d, _ = compare(seeds, base, v)
    assert not h and not d, (h[:3], d[:3])
    print('self-test positive: no hard problems, 3163/3163 same')
    cases = []
    m = copy.deepcopy(base); m['chapters']['07_calculating_ruler']['record']['pictures'][5]['sha256'] = '0' * 64; cases.append(('one image differs in 07', m, 0, 1))
    m = copy.deepcopy(base); m['chapters']['03_product_grid']['record']['pictures'][10]['local_frame'] += 1; cases.append(('local frame +1 in 03', m, 1, 0))
    m = copy.deepcopy(base); m['chapters']['12_chance_and_limits']['record']['extract_source']['sha256'] = 'e' * 64; cases.append(('other chapter bytes in 12', m, 1, 0))
    m = copy.deepcopy(base); m['chapters']['15_logarithm_bridge']['record']['sheets'][0]['frames'][0] -= 1; cases.append(('sheet label frame in 15', m, 1, 0))
    m = copy.deepcopy(base); m['chapters']['01_rate_law']['receipt'] = dict(base['chapters']['01_rate_law']['receipt'], exit=1); cases.append(('failed 01 job', m, 1, 0))
    m = copy.deepcopy(base); m['recheck'] = None; cases.append(('no recheck', m, 1, 0))
    m = copy.deepcopy(base); m['chapters']['09_charging_gap']['record']['verification_sha256'] = 'a' * 64; cases.append(('other verification pin in 09', m, 1, 0))
    for name, dump, want_h, want_d in cases:
        h, d, _ = compare(seeds, dump, v)
        assert (len(h) >= 1) == bool(want_h) and len(d) == want_d, (name, h, d)
        print(f'self-test control {name}: hard {len(h)} differing {len(d)} -> {(h or d)[0][:110]}')
    print('SELF-TEST PASS')


if __name__ == '__main__':
    assert sys.flags.isolated and sys.dont_write_bytecode, 'run with python3 -I -B'
    seeds = load_seeds(Path(sys.argv[1]))
    if sys.argv[2:] == ['--self-test']:
        self_test(seeds)
        raise SystemExit(0)
    dump = json.loads(Path(sys.argv[2]).read_text())
    hard, differ, lines = compare(seeds, dump, sys.argv[3] if len(sys.argv) > 3 else None)
    print('\n'.join(lines))
    for x in hard:
        print('HARD', x)
    for x in differ[:200]:
        print('DIFFERS', x)
    print(f'hard problems {len(hard)}, pictures differing from 985 {len(differ)}')
    raise SystemExit(1 if hard else 2 if differ else 0)
