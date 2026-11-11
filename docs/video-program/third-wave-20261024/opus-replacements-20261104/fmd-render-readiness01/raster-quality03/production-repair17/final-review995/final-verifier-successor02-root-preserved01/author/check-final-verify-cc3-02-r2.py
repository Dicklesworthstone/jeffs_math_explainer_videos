"""Local checks of the 995 successor02 (CC0 fix02, ALT placement) and the unchanged after-master launcher (CC3, read-only).

Usage: python3 -I -B check-final-verify-cc3-02-r2.py HERE MASTER_DIR RENDER_DIR PICTURES_DIR Q WORK HERE01 FIXREVIEW

HERE holds successor02; HERE01 the sealed final-verify-cc3 folder (successor01 and the after-master launcher);
FIXREVIEW the sealed fix02-review-cc3 folder (its ALT test copy); MASTER_DIR, RENDER_DIR and PICTURES_DIR are the sealed final-master-cc3,
final-render-cc3 and final-pictures-cc3 folders; Q is the repo's production-repair17 folder; WORK is a new empty
folder for synthetic records. Nothing on CSD is read or run. Both 995 and its successor are imported and their
picture_plan() runs on the same synthetic master (real plan182, preview313 records, 995 windows, master993's
intervals); the successor's plan_for() runs on the local Q records. No video is decoded and nothing is extracted.
"""
import ast
import difflib
import hashlib
import importlib.util
import json
import subprocess
import sys
from pathlib import Path

HERE, MDIR, RENDER, PICS, Q, WORK, HERE01, FIXREVIEW = (Path(p) for p in sys.argv[1:9])
assert WORK.is_dir() and not any(WORK.iterdir()), 'WORK must be a new empty folder'
V, AM, V01 = HERE/'csd-final-verify-repaired02.py', HERE01/'csd-final-after-master-repaired01.py', HERE01/'csd-final-verify-repaired01.py'
V995, AM995 = Q/'csd-final-verify995.py', Q/'csd-final-after-master995.py'
MS, RUN, PIC = MDIR/'combined-master-repaired01.py', RENDER/'csd-native-repair-final01-run.py', PICS/'csd-native-repair-pictures01.py'
CSD_Q = '/data/projects/jmev-stroke-probe-20261106/qualification01/production-repair17'
problems, notes, info = [], [], []


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def check(ok, what):
    (notes if ok else problems).append(('ok   ' if ok else 'FAIL ')+what)


def load(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def refuses(fn, *args):
    try:
        fn(*args)
    except Exception as error:                  # any refusal
        return repr(error)[:100]
    return None


def tops(path):
    tree = ast.parse(Path(path).read_text())
    funcs = {n.name: ast.dump(n) for n in tree.body if isinstance(n, ast.FunctionDef)}
    names = {}
    for n in tree.body:
        if isinstance(n, ast.Assign):
            for t in n.targets:
                if isinstance(t, ast.Name):
                    names[t.id] = ast.dump(n.value)
    return funcs, names


def diffstat(a, b):
    lines = list(difflib.unified_diff(Path(a).read_text().splitlines(), Path(b).read_text().splitlines(), lineterm='', n=0))
    return (sum(x.startswith('+') and not x.startswith('+++') for x in lines),
            sum(x.startswith('-') and not x.startswith('---') for x in lines))


def dump(path, value):
    with path.open('x') as f:
        json.dump(value, f, indent=2)


# 1. syntax and pinned bases
for p in (V, AM):
    compile(p.read_text(), str(p), 'exec')
    check(True, 'compiles: '+p.name)
for p, digest in ((V995, '69fa3ded0960f6344f3cbad557caf951a769dfe7e0d57426a614ea8d7dd2b7a2'), (AM995, '9a055c26'),
                  (MS, '17388c9d'), (RUN, '6dceac47'), (PIC, '5fa242b8'),
                  (Q/'root-preview313-playback343.json', 'd9f250ec00007fe0687c7bf2a3155652b30df45956c5dfe53e9922dd0ec67e81'),
                  (V01, '5c30a62e'), (AM, '15094001'), (FIXREVIEW/'work01/alt-test-copy.py', '0f31010e'),
                  (Q/'completed-master993-summary.json', '38b52105520640e495ada17aa00da4ddbc16316a5a77c4568e102e1ca3adf185')):
    check(sha(p).startswith(digest), f'base {p.name} is {digest[:8]}')

# 2. what changed against 995
fo, no = tops(V995)
fn, nn = tops(V)
changed = {k for k in set(fo) | set(fn) if fo.get(k) != fn.get(k)}
check(changed == {'unresolved', 'expected_worker451', 'master350', 'picture_plan', 'plan_for', 'verify', 'pictures', 'main'},
      f'995 functions changed: {sorted(changed)}')
moved = {k for k in set(no) | set(nn) if no.get(k) != nn.get(k)}
check(moved == {'VERSION', 'MASTER_VERSION', 'MASTER_HELPER_NAME', 'MASTER_HELPER', 'MASTER_HELPER_SHA', 'MASTER', 'MASTER_INPUTS',
                'VERIFY', 'VERIFY_INPUTS', 'PICTURES', 'PICTURES_INPUTS', 'STAGE_VERSION', 'WORKER_NAME', 'WORKER_SHA', 'REPAIRED',
                'REPAIRED_BINARY', 'PLAYBACK343', 'BOUNDARY_INSTANTS', 'NATIVE608', 'MASTER993_SUMMARY', 'PRIOR993_INTERVALS'},
      f'995 module names changed or added: {sorted(moved)}')
kept = ('TRANSITION_WINDOWS', 'TRANSITION_SOURCES', 'FINAL_FRAME_CHECKS', 'FINAL_ENDPOINT_CHECKS', 'REQUIRED_WINDOWS',
        'PREVIOUS_LAYOUT', 'PREVIOUS_LAYOUT_SOURCES', 'EXIT_WINDOW', 'ABSENT_ANCHORS', 'PERSISTENCE', 'ANCHOR_DELTAS',
        'GALLERY_SITES', 'ONE_POINT_INSTANTS', 'FLAGS', 'PINS', 'EXPECT', 'DELIVERY347', 'NATIVE327_CAPTIONS', 'ADOPTION354',
        'WORKERS', 'PICTURE_PART_BYTES', 'VERIFY_FREE_BYTES', 'OUT')
check(all(no[k] == nn[k] for k in kept), '995 windows, layout anchors, 09 checks, 07 instants, flags, caption pins, budgets and OUT unchanged')
for name in ('schedules', 'delivery347', 'film', 'frame_hashes', 'runtime', 'pillow_runtime', 'growth_upper', 'part_done', 'launch',
             'stable', 'shared', 'build313', 'source_job451', 'verified', 'anchors', 'clean', 'sheet_label'):
    check(fo[name] == fn[name], '995 function unchanged: '+name)
def segment(path, fname):
    text = Path(path).read_text()
    node = next(n for n in ast.parse(text).body if isinstance(n, ast.FunctionDef) and n.name == fname)
    return ast.get_source_segment(text, node).splitlines()
for fname in ('verify', 'pictures'):
    # undo exactly the successor's text edits; the function must then be 995's, line for line
    back = [l.replace(', boundary_instants343=BOUNDARY_INSTANTS', '').replace('%s master', '993 master').replace("' % MASTER_VERSION +", "'")
            for l in segment(V, fname) if l.strip() != 'prior993_intervals=PRIOR993_INTERVALS,']
    edited = sum(a != b for a, b in zip([l for l in segment(V, fname) if l.strip() != 'prior993_intervals=PRIOR993_INTERVALS,'], back))
    check(back == segment(V995, fname) and edited, f'{fname}(): 995\'s lines except the master version in {edited} texts'
          + (' and the boundary_instants343 and prior993_intervals record fields' if fname == 'pictures' else ''))
check([ast.dump(x) for x in ast.parse(V.read_text()).body[1:]] == [ast.dump(x) for x in ast.parse((FIXREVIEW/'work01/alt-test-copy.py').read_text()).body[1:]],
      'successor02 code (docstring aside) equals the ALT test copy the fix02 review ran its four scope cases on')
f1, n1 = tops(V01)
check({k for k in set(f1) | set(fn) if f1.get(k) != fn.get(k)} == {'picture_plan', 'plan_for', 'pictures'}
      and {k for k in set(n1) | set(nn) if n1.get(k) != nn.get(k)} == {'MASTER993_SUMMARY', 'PRIOR993_INTERVALS'},
      'against successor01 only picture_plan, plan_for and pictures change, and only the two fix02 names are added')
plus, minus = diffstat(V01, V)
info.append(f'info diff from successor01: +{plus} -{minus}')
plus, minus = diffstat(V995, V)
info.append(f'info diff from 995: +{plus} -{minus}')
plus, minus = diffstat(AM995, AM)
info.append(f'info diff from after-master995: +{plus} -{minus}')

# 3. tables
nv, ov, ms = load(V, 'verify_successor'), load(V995, 'verify995'), load(MS, 'master_successor_tables')
check(nv.unresolved() == ['VERSION', 'MASTER_VERSION', 'MASTER_HELPER_NAME', 'MASTER_HELPER_SHA', 'STAGE_VERSION', 'WORKER_NAME',
                          'WORKER_SHA', 'REPAIRED_BINARY[native-combined01]', 'REPAIRED_BINARY[native-counting02]'],
      'pins pending: '+', '.join(nv.unresolved()))
check(nv.REPAIRED == ms.REPAIRED and all(nv.KINDS[k] == kind for k, kind in nv.REPAIRED.items()) and nv.KINDS['10_rigidity'] == 'native521'
      and {k: v for k, v in nv.KINDS.items() if k not in nv.REPAIRED} == {k: v for k, v in ov.KINDS.items() if k not in nv.REPAIRED},
      'KINDS: 01/08/09/11 the master successor kinds, 10 native521, the other ten as in 995')
check(all(nv.REVIEW_BASES[k] == 'sampled_preview' for k in nv.REPAIRED.values()), 'REVIEW_BASES: both repaired kinds sampled_preview')
check(nv.NATIVE608['10_rigidity'] == ov.NATIVE608['10_rigidity'] and set(nv.NATIVE608) == {'10_rigidity', *nv.REPAIRED},
      'NATIVE608: 10 unchanged; the four repaired rows added (09 native844 entry replaced)')
summary = json.loads((Q/'completed-master993-summary.json').read_text())
row09 = next(r for r in summary['rows'] if r['chapter'] == '09_boundary')
r343 = json.loads((Q/'root-preview313-playback343.json').read_text())
check(list(nv.BOUNDARY_INSTANTS) == r343['playback_observed_seconds'] == row09['review_scope']['playback_seconds'],
      'BOUNDARY_INSTANTS = root 343 playback_observed_seconds = master993 09 row playback_seconds (the four 995 planned)')
check(all(r343[k] is False for k in r343 if k.endswith('_acceptance')) and 'not a continuous video observation or listening' in r343['scope'],
      'root 343 claims no acceptance and states no continuous viewing or listening')
f969 = json.loads((Q/'completed-pictures969.json').read_text())['selected_frames']
p775 = json.loads((Q/'csd-native855-prior775.json').read_text())['selected_frames']
check(sorted(f969) == sorted(p775) and len(f969) == 814, 'final969 frames = prior775 frames (814), locally')
rt, pt = RUN.read_text(), PIC.read_text()
check("required |= set(read(prior)['selected_frames'])" in rt and "assert required <= set(frames), 'the extraction dropped retained samples'" in rt
      and "assert required<=set(frames),'the admitted preview extraction lacks retained samples'" in pt and 'assert set(kept)<=chosen' in pt,
      'the 09 final extraction must hold prior775 (run proofs, retained(), choose()), so it holds 969\'s 814 frames')

# 4. both picture plans on one synthetic master
plan182 = json.loads((Q/'final-picture-plan182.json').read_text())
preview = dict(summary=json.loads((Q/'preview313-review333/summary.json').read_text()),
               sampling=json.loads((Q/'preview313-review333/09_boundary/sampling.json').read_text()), files=set())
chapters = [dict(chapter=c['chapter'], start_frame=c['start_frame'], frames=c['frames'], kind='x') for c in plan182['chapters']]
registry = [c['chapter'] for c in chapters]
check(registry == summary['chapters'] and {c['chapter']: c['frames'] for c in chapters} == {r['chapter']: r['frames'] for r in summary['rows']},
      'plan182 chapters = master993 registry and frames')
intervals = {r['chapter']: [tuple(x) for x in r['sampled_intervals']] for r in summary['rows'] if r['review_basis'] == 'sampled_preview'}
layouts = {k: [] for k in registry}
timelines = {k: dict(lines=[]) for k in registry}
final969 = '/data/projects/jmev-stroke-probe-20261106/jobs/csd-native-pictures969/pictures969.json'
frames_of = {c['chapter']: c['frames'] for c in chapters}
sel = {'09_boundary': sorted(f969),
       '01_the_bump': sorted(int(x) for x in (Q/'final-review995/combined-pictures-root01/frames-01_the_bump-all.txt').read_text().split()),
       '08_loop_sums': sorted(int(x) for x in (Q/'final-review995/combined-pictures-root01/frames-08_loop_sums-all.txt').read_text().split()),
       '11_counting': sorted(int(x) for x in (Q/'final-review995/counting-pictures02-root/frames-11_counting-all.txt').read_text().split())}


def master(kind):
    by, files = {}, set()
    for k in registry:
        by[k] = dict(chapter=k, job='job-'+k, frames=frames_of[k], movie_sha256='movie-'+k)
    if kind == 'old':
        proof = WORK/'proof-old-09.json'
        dump(proof, dict(extraction=final969))
        by['09_boundary'].update(local_proofs=dict(path=str(proof)), movie_sha256=json.loads((Q/'completed-pictures969.json').read_text())['movie_sha256'])
        files.add(Path(final969))
        playback = {'09_boundary': list(row09['review_scope']['playback_seconds'])}
    else:
        for k, frames in sel.items():
            (WORK/k).mkdir()
            record = WORK/k/'pictures.json'
            dump(record, dict(full_decode=True, frames=frames_of[k], movie_sha256='movie-'+k, selected_frames=frames))
            proof = WORK/('proof-new-%s.json' % k)
            dump(proof, dict(extraction=str(record)))
            by[k]['local_proofs'] = dict(path=str(proof))
            files.add(record)
        playback = {}
    return dict(registry=registry, by=by, intervals=intervals, playback=playback, files=files)


real_read = ov.read
ov.read = lambda p: json.loads((Q/'completed-pictures969.json').read_text()) if str(p) == final969 else real_read(p)
f_old = dict(chapters=chapters, timelines=timelines, total=sum(frames_of.values()), m=master('old'))
f_new = dict(chapters=chapters, timelines=timelines, total=sum(frames_of.values()), m=master('new'))
os_, on_ = ov.picture_plan(f_old, layouts, plan182, preview)
ns_, nn_ = nv.picture_plan(f_new, layouts, plan182, preview)
S = lambda xs: {x['frame'] for x in xs}
start = {c['chapter']: c['start_frame'] for c in chapters}
added = {start[k]+x for k, xs in sel.items() if k != '09_boundary' for x in xs}
check(S(os_) <= S(ns_) and S(ns_) - S(os_) == added - S(os_),
      f'overview: every 995 sample kept ({len(S(os_))}); the only new ones are 01/08/11 final extraction frames (+{len(S(ns_)-S(os_))})')
check(S(on_) == S(nn_), f'native 4K frames identical to 995\'s ({len(S(nn_))}), the four 09 instant natives included')
by_new = {x['frame']: x['reasons'] for x in ns_}
inst = {start['09_boundary']+round(s*60)+d for s in nv.BOUNDARY_INSTANTS for d in range(-2, 3)}
check(all(any('root 343 instant' in r and 'final still only' in r for r in by_new[g]) for g in inst),
      'each 09 instant frame +-2 is planned and labelled a final still only (root 343)')
check(all(any('actual final repaired sample' in r for r in by_new[start[k]+x]) for k, xs in sel.items() for x in xs),
      'every frame of the four repaired final extractions is planned (09: all 814 of 969)')
reasons = [r for x in ns_ + nn_ for r in x['reasons']]
check(not any('listen' in r.lower() for r in reasons) and not any('permit playback instant' in r for r in reasons),
      'no planned reason claims listening; no playback instant comes from a repaired row\'s scope')
info.append(f'info plan sizes: 995 {len(os_)} samples / {len(on_)} natives; successor {len(ns_)} / {len(nn_)}; '
            f'overview upper {nv.part_upper("overview", ns_, nn_)/2**30:.2f} GiB, native upper {nv.part_upper("native", ns_, nn_)/2**30:.2f} GiB '
            f'(budget {nv.PICTURE_PART_BYTES/2**30:.1f} GiB each)')

# 5. plan_for on the local Q records (layouts empty, preview313 from its local copy)
out = WORK/'out'
out.mkdir()
for k in registry:
    (out/(k+'.layout.tsv')).write_text('')
nv.Q, nv.OUT, nv.PLAN182 = Q, out, Q/'final-picture-plan182.json'   # module constants built from the CSD Q
nv.preview313 = lambda: preview
check(nv.MASTER993_SUMMARY[0] == Path('/data/projects/jmev-stroke-probe-20261106/jobs/csd-combined-master993/master-summary.json'),
      'MASTER993_SUMMARY names master993\'s own job summary on CSD')
nv.MASTER993_SUMMARY = (Q/'completed-master993-summary.json', nv.MASTER993_SUMMARY[1])   # local copy, same sha256
samples, natives, used = nv.plan_for(f_new)
check(S(samples) == S(ns_) and Q/'root-preview313-playback343.json' in used
      and Q/'completed-master993-summary.json' in used,
      'plan_for(): passes every pinned record, the 06 and 08 required windows, and binds root 343 and master993\'s summary')
keep = nv.PRIOR993_INTERVALS
nv.PRIOR993_INTERVALS = dict(keep, **{'11_counting': ((34, 87), (7464, 7494))})
check(refuses(nv.plan_for, f_new) is not None, 'plan_for() refuses PRIOR993_INTERVALS that differ from master993\'s summary')
nv.PRIOR993_INTERVALS = keep
nv.MASTER993_SUMMARY = (Q/'completed-master993-summary.json', '0'*64)
check(refuses(nv.plan_for, f_new) is not None, 'plan_for() refuses a master993 summary other than the pinned one')
nv.MASTER993_SUMMARY = (Q/'completed-master993-summary.json', '38b52105520640e495ada17aa00da4ddbc16316a5a77c4568e102e1ca3adf185')
narrow = dict(f_new, m=dict(f_new['m'], intervals=dict(intervals, **{'08_loop_sums': [(10216, 10358)]})))
check(refuses(nv.plan_for, narrow) is not None, 'plan_for() still refuses an 08 scope without 10200-10380 (995:1102-1104 not weakened)')
nv.BOUNDARY_INSTANTS = nv.BOUNDARY_INSTANTS[:3]
check(refuses(nv.plan_for, f_new) is not None, 'plan_for() refuses instants that differ from root 343\'s record')

# 6. both helpers refuse with pins unset and write nothing
for argv in ([str(V), 'check'], [str(AM)]):
    run = subprocess.run([sys.executable, '-I', '-B', *argv], cwd=WORK/'out', capture_output=True, text=True)
    check(run.returncode == 1 and 'unresolved pins' in run.stderr and sorted(p.name for p in out.iterdir()) == sorted(k+'.layout.tsv' for k in registry),
          Path(argv[0]).name+': refuses with pins unset, writes nothing')
at, ot = AM.read_text(), AM995.read_text()
for line in ('deadline=time.monotonic()+43200', "if r.get('finished'):", "subprocess.run(['/usr/bin/python3','-I','-B',str(HELPER),mode],check=True)",
             "assert sha(HELPER)==HELPER_SHA"):
    check(line in at and line in ot, 'after-master line kept: '+line)
check("if job:success(D/'jobs'/job/'resource-receipt.json')" in ot and "if job:success(job/'resource-receipt.json')" in at,
      'after-master: each job\'s success is required before the next mode (995 named jobs; the successor holds paths)')
check("jobs=[h.VERIFY]+[h.part_paths(p)[0] for p in h.PICTURE_PARTS]" in at and "('launch-verify',jobs[0])" in at
      and "('launch-pictures',jobs[1])" in at and "('launch-pictures',jobs[2])" in at, 'after-master: job names from the pinned verifier, 995 order')
for p in (V, AM):
    t = p.read_text()
    check('—' not in t and '/Users/' not in t, p.name+': no prose em dash, no home path')

for line in notes + problems + info:
    print(line)
print(f'\n{len(notes)} passed, {len(problems)} failed')
sys.exit(1 if problems else 0)
