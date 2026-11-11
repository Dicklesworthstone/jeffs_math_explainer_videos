"""coverage01.py: which of 995's planned frames the 995 successor still guarantees when the repaired rows' admitted
scopes do not repeat master993's intervals (CC0, read-only; writes only its own output).

995 (69fa3ded) and CC3's successor (5c30a62e) picture_plan() run on the same synthetic film as CC3's own check:
real plan182, the local preview313 records, master993's chapters. Only the repaired rows differ:
  995:       master993's 15 rows' intervals, 09's four playback instants from its row, final969 carried for 09;
  successor: 11 unchanged rows keep master993's intervals; the four repaired rows get the smallest admitted scope
             each could carry (08: exactly 10200-10380, which 995:1102 forces; 01/09/11: (0, 1), seam frames
             already planned), no row playback, and each final extraction = its predicted preview extraction
             (plan list; 08 plus scope02's range and both lint anchors; 09 plus prior775).
Reports the 995 overview samples and native frames the successor would no longer plan, by chapter and reason.

usage: python3 -I -B coverage01.py PR17_DIR CC3_VERIFY_DIR OUT_DIR
"""
import importlib.util
import json
import sys
from pathlib import Path

PR17, CC3, OUT = (Path(x) for x in sys.argv[1:4])
lines = []
say = lines.append


def load(p, name):
    spec = importlib.util.spec_from_file_location(name, p)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def read(p):
    return json.loads(Path(p).read_text())


ov = load(PR17/'csd-final-verify995.py', 'v995')
nv = load(CC3/'csd-final-verify-repaired01.py', 'v_successor')
summary = read(PR17/'completed-master993-summary.json')
plan182 = read(PR17/'final-picture-plan182.json')
preview = dict(summary=read(PR17/'preview313-review333/summary.json'),
               sampling=read(PR17/'preview313-review333/09_boundary/sampling.json'), files=set())
chapters = [dict(chapter=c['chapter'], start_frame=c['start_frame'], frames=c['frames'], kind='x') for c in plan182['chapters']]
registry = [c['chapter'] for c in chapters]
frames_of = {c['chapter']: c['frames'] for c in chapters}
start = {c['chapter']: c['start_frame'] for c in chapters}
assert registry == summary['chapters']
old_intervals = {r['chapter']: [tuple(x) for x in r['sampled_intervals']] for r in summary['rows'] if r['review_basis'] == 'sampled_preview'}
row09 = next(r for r in summary['rows'] if r['chapter'] == '09_boundary')
f969 = read(PR17/'completed-pictures969.json')
p775 = read(PR17/'csd-native855-prior775.json')['selected_frames']
pics = PR17/'final-review995/combined-pictures-root01'


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
ov.read = lambda p, real=ov.read: records[str(p)] if str(p) in records else real(p)
nv.read = lambda p, real=nv.read: records[str(p)] if str(p) in records else real(p)


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
old_s, old_n = ov.picture_plan(film(old_intervals, {'09_boundary': list(row09['review_scope']['playback_seconds'])}, False),
                               layouts, plan182, preview)
minimal = dict(old_intervals, **{'01_the_bump': [(0, 1)], '08_loop_sums': [(10200, 10380)], '09_boundary': [(0, 1)], '11_counting': [(0, 1)]})
new_s, new_n = nv.picture_plan(film(minimal, {}, True), layouts, plan182, preview)
same_s, same_n = nv.picture_plan(film(old_intervals, {}, True), layouts, plan182, preview)
S = lambda xs: {x['frame']: x for x in xs}
os_, on_, ns_, nn_, ss_, sn_ = S(old_s), S(old_n), S(new_s), S(new_n), S(same_s), S(same_n)
say('995 plan: %d overview samples, %d natives' % (len(os_), len(on_)))
say('successor, scopes = master993 intervals (CC3 premise): %d / %d; 995 samples kept %s, natives kept %s'
    % (len(ss_), len(sn_), set(os_) <= set(ss_), set(on_) <= set(sn_)))
say('successor, smallest admissible scopes: %d / %d' % (len(ns_), len(nn_)))
lost_s = sorted(set(os_) - set(ns_))
lost_n = sorted(set(on_) - set(nn_))
say('995 overview samples no longer planned: %d' % len(lost_s))
for k in registry:
    mine = [g for g in lost_s if os_[g]['chapter'] == k]
    if mine:
        locs = [os_[g]['local_frame'] for g in mine]
        runs, a = [], locs[0]
        for x, y in zip(locs, locs[1:]+[None]):
            if y != x+1:
                runs.append('%d-%d' % (a, x) if a != x else str(a))
                a = y
        reasons = sorted({r.split(' (')[0] for g in mine for r in os_[g]['reasons']})
        say('  %s: %d frames, runs %s; 995 reasons: %s' % (k, len(mine), ', '.join(runs), reasons))
say('995 native frames no longer planned: %d' % len(lost_n))
for g in lost_n:
    say('  %s local %d: %s' % (on_[g]['chapter'], on_[g]['local_frame'], on_[g]['reasons']))
with open(OUT/'coverage01.out', 'x', encoding='utf-8') as f:
    f.write('\n'.join(lines)+'\n')
print('\n'.join(lines))
