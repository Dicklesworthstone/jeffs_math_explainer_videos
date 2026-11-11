"""recheck02.py: CC0's recheck of CC3's successor02 (the ALT 0f31010e with its comment wrapped). Same checks as
validate-alt01.py (3e66c9ad), run on the successor02 file itself, plus: its Python tokens equal ALT's once comments
and non-logical line breaks are ignored (any other difference is listed). Read-only for every input; writes only into
its own new folder. fix03's copy is run beside it as the witness of the defect, not as a candidate.

(validate-alt01's description follows; "ALT" below means the successor02 file under check.)
CC3's ALT resolves CC3's R1 against CC0 fix03 (b7b3b693): the master993 block moves after the 377 required-window
block and decides its natives on `natives995`, a snapshot taken just before the 363 admitted-scope block.

Synthetic film as CC0 coverage01/fix03 (real plan182 and preview313; predicted final extractions incl. scope02's 08
range and both lint anchors; 09 plus prior775). Seven scope cases for the four repaired rows: CC3's four (premise =
master993 intervals, smallest, wide, disjoint) and three of CC0's own (wide2, partial overlaps, disjoint2).

For ALT in every case:
  A. every frame/reason entry of the sealed successor 5c30a62e is kept (samples and natives);
  B. every 995 overview sample frame and native frame is planned;
  C. every entry ALT adds carries one of the two fix labels;
  D. ALT's master993 natives sit on exactly the old intervals where 995 placed an interval-middle native, and every
     frame of every old interval is a sample with the master993 label;
  E. (premise) ALT's native frames equal 995's.
Plus: ALT's diff from 5c30a62e against fix03's; plan_for on ALT passes at master993's pin and refuses a changed
constant or another summary hash.

usage: python3 -I -B recheck02.py PR17_DIR CC3_BASE_DIR ALT0_FILE SUCCESSOR02_FILE FIX03_COPY OUT_DIR
"""
import difflib
import hashlib
import importlib.util
import io
import json
import re
import sys
import tokenize
from pathlib import Path

PR17, CC3, ALT0, ALT, FIX03, OUT = (Path(x) for x in sys.argv[1:7])
lines, bad = [], []
say = lines.append


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def check(name, ok):
    say('%-4s %s' % ('ok' if ok else 'FAIL', name))
    if not ok:
        bad.append(name)


SRC = CC3/'csd-final-verify-repaired01.py'
assert sha(SRC) == '5c30a62ef9341ff72be6650d462f7de2eb32ad0c5089b826d50897d3c9ef4dbb', 'sealed successor changed'
assert sha(ALT0) == '0f31010e5f328490450e13d7b23a76ce3ba28d87fc979354dbfe3da323331f3c', 'ALT changed'
assert sha(FIX03) == 'b7b3b69341f3e9c0699479e4df8f2070377cddb9b12490d5f46f10e5da4a6c23', 'fix03 copy changed'
say('     sealed %s; ALT %s; fix03 copy %s' % (sha(SRC)[:8], sha(ALT0)[:8], sha(FIX03)[:8]))
say('     successor02 %s %s' % (ALT.name, sha(ALT)))
compile(ALT.read_text(), ALT.name, 'exec')
check('successor02 compiles', True)


def code_tokens(p):
    skip = (tokenize.COMMENT, tokenize.NL)
    return [(t.type, t.string) for t in tokenize.generate_tokens(io.StringIO(Path(p).read_text()).readline) if t.type not in skip]


t0, t2 = code_tokens(ALT0), code_tokens(ALT)
check('successor02 equals ALT 0f31010e in every Python token but comments and line breaks (%d tokens)' % len(t2), t0 == t2)
if t0 != t2:
    for tag, i1, i2, j1, j2 in difflib.SequenceMatcher(None, t0, t2, autojunk=False).get_opcodes():
        if tag != 'equal':
            say('       token %s: ALT %s -> successor02 %s' % (tag, [s for _, s in t0[i1:i2]][:12], [s for _, s in t2[j1:j2]][:12]))
wide = [x for x in ALT.read_text().splitlines() if len(x) > 120]
say('     successor02 lines over 120 characters: %d (sealed has %d)'
    % (len(wide), sum(len(x) > 120 for x in SRC.read_text().splitlines())))


def changed(a, b):
    d = list(difflib.unified_diff(Path(a).read_text().splitlines(True), Path(b).read_text().splitlines(True)))
    return ([x[1:] for x in d if x.startswith('+') and not x.startswith('+++')],
            [x[1:] for x in d if x.startswith('-') and not x.startswith('---')])


alt_add, alt_del = changed(SRC, ALT)
fix_add, fix_del = changed(SRC, FIX03)
say('     ALT vs sealed: +%d -%d; fix03 vs sealed: +%d -%d' % (len(alt_add), len(alt_del), len(fix_add), len(fix_del)))
check('ALT removes from the sealed file only what fix03 removes (the return line it extends)', alt_del == fix_del)
for x in fix_add:
    if x not in alt_add:
        say('       fix03 line not in ALT: %s' % x.rstrip('\n'))
for x in alt_add:
    if x not in fix_add:
        say('       ALT line not in fix03: %s' % x.rstrip('\n'))
only_fix = [x for x in fix_add if x not in alt_add]
only_alt = [x for x in alt_add if x not in fix_add]
check('ALT differs from fix03 only by the snapshot line, the native condition on it and the block comment',
      sorted(x.split('#')[0].strip() for x in only_fix if not x.strip().startswith('#')) ==
      ['if not any(start+a <= g <= start+b for g in natives):'] and
      sorted(x.split('#')[0].strip() for x in only_alt if not x.strip().startswith('#')) ==
      sorted(['if not any(start+a <= g <= start+b for g in natives995):', 'natives995 = set(natives)']))
long_lines = [x for x in alt_add if len(x.rstrip('\n')) > 120]
say('     ALT added lines over 120 characters: %d (CC3 says successor02 wraps the comment)' % len(long_lines))


def load(p, name):
    spec = importlib.util.spec_from_file_location(name, p)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def read(p):
    return json.loads(Path(p).read_text())


ov, av, xv, uv = (load(PR17/'csd-final-verify995.py', 'v995'), load(ALT, 'v_alt'), load(FIX03, 'v_fix03'),
                  load(SRC, 'v_sealed'))
summary = read(PR17/'completed-master993-summary.json')
plan182 = read(PR17/'final-picture-plan182.json')
preview = dict(summary=read(PR17/'preview313-review333/summary.json'),
               sampling=read(PR17/'preview313-review333/09_boundary/sampling.json'), files=set())
chapters = [dict(chapter=c['chapter'], start_frame=c['start_frame'], frames=c['frames'], kind='x') for c in plan182['chapters']]
registry = [c['chapter'] for c in chapters]
frames_of = {c['chapter']: c['frames'] for c in chapters}
start = {c['chapter']: c['start_frame'] for c in chapters}
old = {r['chapter']: [tuple(x) for x in r['sampled_intervals']] for r in summary['rows'] if r['review_basis'] == 'sampled_preview'}
row09 = next(r for r in summary['rows'] if r['chapter'] == '09_boundary')
f969 = read(PR17/'completed-pictures969.json')
p775 = read(PR17/'csd-native855-prior775.json')['selected_frames']
pics = PR17/'final-review995/combined-pictures-root01'
REPAIRED = ('01_the_bump', '08_loop_sums', '09_boundary', '11_counting')
assert {k: tuple(v) for k, v in av.PRIOR993_INTERVALS.items()} == {k: tuple(old[k]) for k in REPAIRED}


def plan_list(path):
    return {int(x) for x in Path(path).read_text().split()}


def lint(at, total):
    return set(range(at, min(at+121, total), 12)) | set(range(at, total, 300))


sel = {'01_the_bump': plan_list(pics/'frames-01_the_bump-all.txt'),
       '08_loop_sums': plan_list(pics/'frames-08_loop_sums-all.txt') | set(range(10200, 10380)) | lint(10788, 14959) | lint(12336, 14959),
       '09_boundary': plan_list(pics/'frames-09_boundary-all.txt') | set(p775),
       '11_counting': plan_list(PR17/'final-review995/counting-pictures02-root/frames-11_counting-all.txt')}
FINAL969 = '/data/projects/jmev-stroke-probe-20261106/jobs/csd-native-pictures969/pictures969.json'
records = {FINAL969: f969, 'proof-old-09': dict(extraction=FINAL969)}
for k, frames in sel.items():
    records['final-'+k] = dict(full_decode=True, frames=frames_of[k], movie_sha256='movie-'+k, selected_frames=sorted(frames))
    records['proof-'+k] = dict(extraction='final-'+k)
for module in (ov, av, xv, uv):
    module.read = lambda p, real=module.read: records[str(p)] if str(p) in records else real(p)


def film(intervals, playback, repaired):
    by = {k: dict(chapter=k, job='job-'+k, frames=frames_of[k], movie_sha256='movie-'+k) for k in registry}
    files = set()
    if repaired:
        for k in sel:
            by[k]['local_proofs'] = dict(path='proof-'+k)
            files.add(Path('final-'+k))
    else:
        by['09_boundary'].update(local_proofs=dict(path='proof-old-09'), movie_sha256=f969['movie_sha256'])
        files.add(Path(FINAL969))
    m = dict(registry=registry, by=by, intervals=intervals, playback=playback, files=files)
    return dict(chapters=chapters, timelines={k: dict(lines=[]) for k in registry}, total=sum(frames_of.values()), m=m)


layouts = {k: [] for k in registry}
S = lambda xs: {x['frame'] for x in xs}
E = lambda xs: {(x['frame'], r) for x in xs for r in x['reasons']}
o_s, o_n = ov.picture_plan(film(old, {'09_boundary': list(row09['review_scope']['playback_seconds'])}, False), layouts, plan182, preview)
say('     995 plan: %d samples / %d natives' % (len(o_s), len(o_n)))
# 995's interval-middle natives on the four repaired rows' old intervals
mid995 = set()
for x in o_n:
    for r in x['reasons']:
        m = re.fullmatch(r'native 4K, sampled interval (\d+)-(\d+) middle \(.*\)', r)
        if m and x['chapter'] in REPAIRED:
            mid995.add((x['chapter'], int(m.group(1)), int(m.group(2))))
say('     995 placed interval-middle natives on %d of the 11 old intervals: %s' % (len(mid995), sorted(mid995)))
form = re.compile(r'(native 4K, )?master993 sampled interval \d+-\d+(, every frame| middle) \(old render review scope\); final still only')

CASES = {
    'premise (CC3)': old,
    'smallest (CC3, from CC0)': dict(old, **{'01_the_bump': [(0, 1)], '08_loop_sums': [(10200, 10380)], '09_boundary': [(0, 1)], '11_counting': [(0, 1)]}),
    'wide (CC3)': dict(old, **{'01_the_bump': [(5400, 6310)], '08_loop_sums': [(3000, 3200), (9000, 10400)], '09_boundary': [(2000, 2400)],
                               '11_counting': [(0, 100), (7400, 8400)]}),
    'disjoint (CC3)': dict(old, **{'01_the_bump': [(100, 120)], '08_loop_sums': [(10190, 10390)], '09_boundary': [(3000, 3010)],
                                   '11_counting': [(9000, 9001)]}),
    'wide2 (CC0)': dict(old, **{'01_the_bump': [(5000, 6310)], '08_loop_sums': [(3050, 3190), (10100, 10500)], '09_boundary': [(2050, 2350)],
                                '11_counting': [(20, 100), (7450, 7500), (8280, 8350)]}),
    'partial overlaps (CC0)': dict(old, **{'01_the_bump': [(5520, 5540), (6260, 6280)], '08_loop_sums': [(3060, 3119), (10200, 10400)],
                                           '09_boundary': [(2100, 2320)], '11_counting': [(80, 100), (7494, 7500), (8340, 8350)]}),
    'disjoint2 (CC0)': dict(old, **{'01_the_bump': [(1000, 1100)], '08_loop_sums': [(5000, 5100), (10200, 10380)],
                                    '09_boundary': [(5000, 5100)], '11_counting': [(3000, 3100)]}),
}
for name, scopes in CASES.items():
    for k in REPAIRED:
        assert all(0 <= a <= b < frames_of[k] for a, b in scopes[k]), (name, k)
    f = film(scopes, {}, True)
    u_s, u_n = uv.picture_plan(f, layouts, plan182, preview)
    a_s, a_n = av.picture_plan(f, layouts, plan182, preview)
    x_s, x_n = xv.picture_plan(f, layouts, plan182, preview)
    say('     %s: sealed %d/%d, ALT %d/%d, fix03 %d/%d (samples/natives)'
        % (name, len(u_s), len(u_n), len(a_s), len(a_n), len(x_s), len(x_n)))
    check('%s: A. ALT keeps every sealed entry (samples %d, natives %d)' % (name, len(E(u_s)), len(E(u_n))),
          E(u_s) <= E(a_s) and E(u_n) <= E(a_n))
    check('%s: B. ALT plans every 995 sample and native frame' % name, S(o_s) <= S(a_s) and S(o_n) <= S(a_n))
    plus = {r for _, r in (E(a_s) - E(u_s)) | (E(a_n) - E(u_n))}
    check('%s: C. ALT adds only the two fix labels (%d distinct)' % (name, len(plus)), all(form.fullmatch(r) for r in plus))
    mids = set()
    for x in a_n:
        for r in x['reasons']:
            m = re.fullmatch(r'native 4K, master993 sampled interval (\d+)-(\d+) middle .*', r)
            if m:
                mids.add((x['chapter'], int(m.group(1)), int(m.group(2))))
    by_frame = {x['frame']: x['reasons'] for x in a_s}
    every = all(any(r.startswith('master993 sampled interval %d-%d, every frame' % (a, b)) for r in by_frame.get(start[k]+g, []))
                for k in REPAIRED for a, b in old[k] for g in range(a, b+1))
    check('%s: D. master993 natives exactly where 995 placed them (%d), every old-interval frame labelled' % (name, len(mids)),
          mids == mid995 and every)
    if name.startswith('premise'):
        check('%s: E. ALT native frames equal 995\'s (%d)' % (name, len(S(o_n))), S(a_n) == S(o_n))
    lost_e = sorted((E(u_s) - E(x_s)) | (E(u_n) - E(x_n)))
    lost_f = sorted(S(u_n) - S(x_n))
    say('       fix03 witness: sealed entries lost %d, sealed native frames lost %d%s'
        % (len(lost_e), len(lost_f), (' ' + str([(g - start[k], k) for g in lost_f for k in REPAIRED
                                                 if start[k] <= g < start[k]+frames_of[k]])) if lost_f else ''))

# plan_for on ALT with the local records, the local master993 summary at the pinned hash in place of the CSD path
work = OUT/'work'
work.mkdir()
for k in registry:
    with open(work/(k+'.layout.tsv'), 'x') as stream:
        stream.write('')
av.Q, av.OUT, av.PLAN182 = PR17, work, PR17/'final-picture-plan182.json'
av.preview313 = lambda: preview
check('local master993 summary has the pinned sha256', sha(PR17/'completed-master993-summary.json') == av.MASTER993_SUMMARY[1])
av.MASTER993_SUMMARY = (PR17/'completed-master993-summary.json', av.MASTER993_SUMMARY[1])
f_old = film(old, {}, True)
samples, natives, used = av.plan_for(f_old)
check('ALT plan_for passes at the pin and binds the summary', PR17/'completed-master993-summary.json' in used)


def refuses(fn, *args):
    try:
        fn(*args)
        return False
    except AssertionError:
        return True


keep = av.PRIOR993_INTERVALS
av.PRIOR993_INTERVALS = dict(keep, **{'11_counting': ((34, 87), (7464, 7494))})
check('ALT plan_for refuses a constant that differs from master993\'s summary', refuses(av.plan_for, f_old))
av.PRIOR993_INTERVALS = keep
av.MASTER993_SUMMARY = (PR17/'completed-master993-summary.json', '0'*64)
check('ALT plan_for refuses a master993 summary that is not the pinned one', refuses(av.plan_for, f_old))
say('RESULT: %s' % ('all checks passed' if not bad else 'FAILED: '+'; '.join(bad)))
with open(OUT/'recheck02.out', 'x', encoding='utf-8') as stream:
    stream.write('\n'.join(lines)+'\n')
print('\n'.join(lines))
