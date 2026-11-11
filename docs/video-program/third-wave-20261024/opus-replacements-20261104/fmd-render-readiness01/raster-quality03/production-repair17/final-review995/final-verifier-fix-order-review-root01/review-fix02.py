"""CC3 independent review of CC0 fix02 (master993 intervals kept in the 995 successor's picture plan).

usage: python3 -I -B review-fix02.py PR17 CC3_VERIFY_DIR CC0_FIX_DIR OUT
Reads only; writes a new OUT folder (an alternative placement as a test copy, and the report lines).
Plans compared on one synthetic film (real plan182, preview313 records, master993 summary; final extraction
records stand in for the not-yet-existing finals): 995, CC3's sealed successor (unpatched), CC0's test copy, and
ALT (the same block placed after the required windows, with 995's native rule evaluated on the natives that
existed before the admitted scopes were planned).
"""
import hashlib
import importlib.util
import json
from pathlib import Path
import sys

PR17, CC3, CC0, OUT = (Path(x) for x in sys.argv[1:5])
OUT.mkdir()
lines, bad = [], []


def say(s):
    lines.append(s)


def check(name, ok):
    say('%-4s %s' % ('ok' if ok else 'FAIL', name))
    if not ok:
        bad.append(name)


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def read(p):
    return json.loads(Path(p).read_text())


SRC, COPY = CC3/'csd-final-verify-repaired01.py', CC0/'test-copy-csd-final-verify-repaired01-fix02.py'
say('     sealed successor %s; CC0 test copy %s' % (sha(SRC)[:8], sha(COPY)[:8]))

# ALT: CC0's text with its picture_plan block moved after the required windows and 995's native rule kept exact.
text = COPY.read_text()
start = text.index("    # CC0 review01 F1: master993's sampled intervals of the four repaired rows")
end = text.index("    # 363: M1 on the new review basis")
block = text[start:end]
text = text[:start] + text[end:]
assert text.count("    # 363: M1 on the new review basis") == 1
text = text.replace("    # 363: M1 on the new review basis",
                    "    natives995 = set(natives)   # the natives 995's rule saw before the sampled intervals (fix02 below)\n"
                    "    # 363: M1 on the new review basis", 1)
block = block.replace("            if not any(start+a <= g <= start+b for g in natives):\n",
                      "            if not any(start+a <= g <= start+b for g in natives995):\n", 1)
block = block.replace("planned exactly as 995 planned them\n    # (here, by the same rule).",
                      "planned exactly as 995 planned them\n    # (995's rule, on the natives before the intervals), after "
                      "every admitted and required entry, which it never displaces.", 1)
anchor = "    # 377 (367 L2): both ends of each observation range root added, as overview samples\n"
assert text.count(anchor) == 1 and block.count('natives995') == 1
text = text.replace(anchor, block + anchor, 1)
ALT = OUT/'alt-test-copy.py'
with ALT.open('x') as f:
    f.write(text)
compile(text, ALT.name, 'exec')
say('     ALT test copy %s' % sha(ALT)[:8])


def load(p, name):
    spec = importlib.util.spec_from_file_location(name, p)
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m


v995, unp, cc0, alt = (load(PR17/'csd-final-verify995.py', 'v995'), load(SRC, 'unp'), load(COPY, 'cc0'), load(ALT, 'alt'))
summary = read(PR17/'completed-master993-summary.json')
plan182 = read(PR17/'final-picture-plan182.json')
preview = dict(summary=read(PR17/'preview313-review333/summary.json'),
               sampling=read(PR17/'preview313-review333/09_boundary/sampling.json'), files=set())
chapters = [dict(chapter=c['chapter'], start_frame=c['start_frame'], frames=c['frames']) for c in plan182['chapters']]
registry = [c['chapter'] for c in chapters]
frames_of = {c['chapter']: c['frames'] for c in chapters}
start_of = {c['chapter']: c['start_frame'] for c in chapters}
old = {r['chapter']: [tuple(x) for x in r['sampled_intervals']] for r in summary['rows'] if r['review_basis'] == 'sampled_preview'}
row09 = next(r for r in summary['rows'] if r['chapter'] == '09_boundary')
f969 = read(PR17/'completed-pictures969.json')
pics = PR17/'final-review995/combined-pictures-root01'
words = lambda p: sorted({int(x) for x in Path(p).read_text().split()})
sel = {'01_the_bump': words(pics/'frames-01_the_bump-all.txt'), '08_loop_sums': words(pics/'frames-08_loop_sums-all.txt'),
       '09_boundary': sorted(set(words(pics/'frames-09_boundary-all.txt')) | set(read(PR17/'csd-native855-prior775.json')['selected_frames'])),
       '11_counting': words(PR17/'final-review995/counting-pictures02-root/frames-11_counting-all.txt')}
F969 = '/data/projects/jmev-stroke-probe-20261106/jobs/csd-native-pictures969/pictures969.json'
records = {F969: f969, 'proof-old-09': dict(extraction=F969)}
for k, fr in sel.items():
    records['final-'+k] = dict(full_decode=True, frames=frames_of[k], movie_sha256='movie-'+k, selected_frames=fr)
    records['proof-'+k] = dict(extraction='final-'+k)
for m in (v995, unp, cc0, alt):
    m.read = lambda p, real=m.read: records[str(p)] if str(p) in records else real(p)


def film(intervals, playback, repaired):
    by = {k: dict(chapter=k, job='job-'+k, frames=frames_of[k], movie_sha256='movie-'+k) for k in registry}
    files = set()
    if repaired:
        for k in sel:
            by[k]['local_proofs'] = dict(path='proof-'+k)
            files.add(Path('final-'+k))
    else:
        by['09_boundary'].update(local_proofs=dict(path='proof-old-09'), movie_sha256=f969['movie_sha256'])
        files.add(Path(F969))
    return dict(chapters=chapters, timelines={k: dict(lines=[]) for k in registry}, total=sum(frames_of.values()),
                m=dict(registry=registry, by=by, intervals=intervals, playback=playback, files=files))


layouts = {k: [] for k in registry}
o_s, o_n = v995.picture_plan(film(old, {'09_boundary': row09['review_scope']['playback_seconds']}, False), layouts, plan182, preview)
F = lambda xs: {x['frame'] for x in xs}
E = lambda s, n: {('s', x['frame'], r) for x in s for r in x['reasons']} | {('n', x['frame'], r) for x in n for r in x['reasons']}
FIX = ('master993 sampled interval', 'native 4K, master993 sampled interval')
say('     995 plan: %d samples / %d natives' % (len(F(o_s)), len(F(o_n))))

SCOPES = {
    'premise (scopes = master993 intervals)': old,
    'smallest (CC0)': dict(old, **{'01_the_bump': [(0, 1)], '08_loop_sums': [(10200, 10380)], '09_boundary': [(0, 1)], '11_counting': [(0, 1)]}),
    'wide (08 run inside 9000-10400; others enclose the old intervals)':
        dict(old, **{'01_the_bump': [(5400, 6310)], '08_loop_sums': [(3000, 3200), (9000, 10400)], '09_boundary': [(2000, 2400)], '11_counting': [(0, 100), (7400, 8400)]}),
    'disjoint (scopes miss every old interval but the 08 run)':
        dict(old, **{'01_the_bump': [(100, 120)], '08_loop_sums': [(10190, 10390)], '09_boundary': [(3000, 3010)], '11_counting': [(9000, 9001)]}),
}
for name, scopes in SCOPES.items():
    f = film(scopes, {}, True)
    u = unp.picture_plan(f, layouts, plan182, preview)
    for label, m in (('CC0', cc0), ('ALT', alt)):
        s, n = m.picture_plan(f, layouts, plan182, preview)
        removed = E(*u) - E(s, n)
        added = {e[-1] for e in E(s, n) - E(*u)}
        miss_s, miss_n = F(o_s) - F(s), F(o_n) - F(n)
        say('     %s | %s: %d samples / %d natives; removed %d entries; missing 995 samples %d, natives %d'
            % (name, label, len(F(s)), len(F(n)), len(removed), len(miss_s), len(miss_n)))
        for e in sorted(removed)[:6]:
            ch = next(k for k in registry if start_of[k] <= e[1] < start_of[k]+frames_of[k])
            say('       removed %s %s local %d: %s' % (e[0], ch, e[1]-start_of[ch], e[2][:90]))
        if label == 'ALT':
            check('%s: ALT removes no entry of the sealed successor' % name, not removed)
            check('%s: ALT keeps every 995 sample and native' % name, not miss_s and not miss_n)
            check('%s: ALT adds only master993-interval labels' % name, all(r.startswith(FIX) for r in added))
        else:
            say('       CC0: removes no entry: %s; keeps every 995 sample and native: %s'
                % (not removed, not miss_s and not miss_n))
    if name.startswith('premise'):
        check('premise: ALT natives equal 995 natives exactly', F(alt.picture_plan(f, layouts, plan182, preview)[1]) == F(o_n))

say('RESULT: %s' % ('all checks passed' if not bad else 'FAILED: ' + '; '.join(bad)))
with (OUT/'review-fix02.out').open('x') as f:
    f.write('\n'.join(lines) + '\n')
print('\n'.join(lines))
