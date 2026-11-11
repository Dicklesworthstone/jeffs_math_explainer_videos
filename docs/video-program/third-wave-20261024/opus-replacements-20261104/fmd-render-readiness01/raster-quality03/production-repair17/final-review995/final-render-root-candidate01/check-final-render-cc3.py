"""Local static checks of the repaired-chapter final render helpers against csd-native968 (CC3, read-only).

Usage: python3 -I -B check-final-render-cc3.py HELPER_DIR Q

HELPER_DIR holds csd-native-repair-final01-run.py and -launch.py; Q is the repo's production-repair17 folder.
Nothing on CSD is read and nothing is run: the helpers are compiled and parsed, the run module is imported
(it defines tables and functions only), and every pin in its tables is compared with the local records.
"""
import ast
import hashlib
import importlib.util
import json
import sys
from pathlib import Path

helper_dir, Q = Path(sys.argv[1]), Path(sys.argv[2])
F = Q/'final-review995'
RUN, LAUNCH = helper_dir/'csd-native-repair-final01-run.py', helper_dir/'csd-native-repair-final01-launch.py'
problems, notes = [], []


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def check(ok, what):
    (notes if ok else problems).append(('ok   ' if ok else 'FAIL ')+what)


def literal(path, name):
    """The value of a module-level or function-level `name = <literal>` assignment."""
    for node in ast.walk(ast.parse(Path(path).read_text())):
        if isinstance(node, ast.Assign) and any(isinstance(t, ast.Name) and t.id == name for t in node.targets):
            return node.value
    raise KeyError((path, name))


# 1. syntax: compile both helpers (in memory, no .pyc)
for p in (RUN, LAUNCH):
    compile(p.read_text(), str(p), 'exec')
    check(True, 'compiles: '+p.name)

spec = importlib.util.spec_from_file_location('repair_final01_run', RUN)
run = importlib.util.module_from_spec(spec)
spec.loader.exec_module(run)
C, K = run.CHAPTERS, run.CHAINS
check(list(C) == ['01_the_bump', '08_loop_sums', '09_boundary', '11_counting'], 'chapters 01/08/09/11')
check({c['kind'] for c in C.values()} == set(K) == {'native-combined01', 'native-counting02'}, 'two kinds')
check([c['kind'] for c in C.values()] == ['native-combined01']*3+['native-counting02'], 'option A: 01/08/09 combined01, 11 counting02')
check(sum(c['frames'] for c in C.values()) == 49706, 'frames total 49706')

# 2. pins: format; binary pins unresolved and refused before any file read
hexes = [v for c in C.values() for k, v in c.items() if k.endswith('sha256') or k in ('timeline_normalized', 'preview_pcm', 'staged_capture')]
hexes += [v for c in C.values() for v in c['stems'].values()]
check(all(isinstance(v, str) and len(v) == 64 and set(v) <= set('0123456789abcdef') for v in hexes), f'{len(hexes)} hex pins well formed')
check(all(k['binary_sha256'] is run.UNRESOLVED is None for k in K.values()), 'both binary pins UNRESOLVED (pending builds)')
for ch in C:
    try:
        run.proofs(ch)
        check(False, ch+': proofs() did not refuse an unresolved pin')
    except AssertionError as e:
        check('unresolved pin' in str(e), ch+': proofs() refuses before reading files: '+str(e)[:70])
check(run.job('09_boundary').name == 'csd-native-repair-final01-09-boundary', 'job name pattern')

# 3. preserved from csd-native968-run.py: base761, policy, stems folder, cores block, render args, run_step
O = Q/'csd-native968-run.py'
check(sha(O) == '2634c842bacc9b5894511c47ca22c16d1d952e40ea126fd1844d69914e0e48dc', '968 run is the pinned 2634c842')
otext, ntext = O.read_text(), RUN.read_text()
check(run.BASE761_SHA in otext, 'base761 pin equals 968 line 32')
check("'policy'] == dict(admission_gib=10, stop_free_gib=8,\n        phase_growth_gib=4, work_cap_gib=56, target_growth_cap_gib=20, sample_seconds=10)" in otext
      and run.POLICY == dict(admission_gib=10, stop_free_gib=8, phase_growth_gib=4, work_cap_gib=56,
                             target_growth_cap_gib=20, sample_seconds=10), 'guard policy equals 968 lines 44-45')
check(str(run.STEMS) == '/data/projects/jmev-stroke-probe-20261106/jobs/csd-production-score21/score/stems', 'score21 stems folder equals 968 line 97')
block = otext[otext.index("    cpus = sorted(os.sched_getaffinity(0))"):otext.index("    args = [binary")]
check(block in ntext, '16 distinct physical cores block identical to 968 lines 107-116')
old = [ast.dump(x) for x in literal(O, 'args').elts]
ret = next(n for n in ast.walk(ast.parse(ntext)) if isinstance(n, ast.FunctionDef) and n.name == 'render_args').body[-1].value
new = [ast.dump(x) for x in ret.elts]
varying = {0, 2, 10, 14}      # binary, chapter, narration, out
check(len(old) == len(new) == 20 and all(old[i] == new[i] for i in range(20) if i not in varying | {12}),
      'render args: every constant (3840x2160, 60, 16 threads, --score, --lint, --crf 16, --preset veryfast) equals 968 lines 117-119')
check(ast.literal_eval(literal(O, 'args').elts[2]) == '09_boundary' and run.render_args('x', 'b', 'n', 'o')[12] == run.STEMS,
      'only binary, chapter, narration and out vary; --score is score21 stems')
check("h.run_step(JOB, 'native', args, 86400, None)" in otext and "h.run_step(JOB, 'native', args, 86400, None)" in ntext, 'run_step native 86400 unchanged')
expected968 = ast.literal_eval(literal(O, 'expected'))
check(C['09_boundary']['stems'] == expected968, '09 stems equal 968 lines 98-104')
for line in ("assert receipt['admitted'] is True and 'finished' not in receipt and 'exit' not in receipt",
             "assert receipt['phase'] == 'native'",
             "assert os.environ['CARGO_TARGET_DIR'] == str(D/'work')",
             "assert sorted(p.name for p in JOB.iterdir()) == ['command.log', 'resource-receipt.json']",
             "assert review['all_selected_pictures_viewed'] is True",
             "assert review['concrete_defects'] == [] and review['independent_blockers'] == []",
             "assert review['safe_to_advance_to_final4k'] is True",
             "assert root['viewed_frame_indices'] == frames",
             "assert set(independent['viewed_frame_indices']) <= set(frames)",
             "os.environ.update(PATH='/usr/bin:/bin', TMPDIR=str(D/'work/tmp'), XDG_CACHE_HOME=str(D/'work/cache'))"):
    check(line in otext and line in ntext, '968 line kept: '+line[:60])

# 4. pins against local records
m392 = json.loads((Q/'css352-import371-01-the-bump-manifest392.json').read_text())
rows = {r['path']: r['sha256'] for r in m392['files']}
for ch, c in C.items():
    for name, digest in c['stems'].items():
        check(rows.get(str(run.STEMS/name)) == digest, f'{ch} stem {name} = manifest392 row')
    listed = sorted(p.rsplit('/', 1)[1] for p in rows if p.startswith(str(run.STEMS)+'/'+ch+'.'))
    check(listed == sorted(n for n in c['stems'] if '/' not in n), f'{ch} stem set = manifest392 listing {listed}')
previews = {'01_the_bump': 'csd-review-repair-preview274', '08_loop_sums': 'csd-review-repair-preview243',
            '09_boundary': 'csd-review-repair-preview274', '11_counting': 'csd-review-repair-preview274'}
for ch, job in previews.items():
    p = f'/data/projects/jmev-stroke-probe-20261106/jobs/{job}/preview/soundtrack/{ch}/soundtrack_0.wav'
    check(rows.get(p) == C[ch]['preview_pcm'], f'{ch} narration-only preview capture = {job} row')
check(C['09_boundary']['preview_pcm'] in otext, '09 preview capture equals 968 line 63')
bundle = json.loads((F/'counting-build02-root/csd-counting02-bundle03-manifest.json').read_text())
repl = {r['workspace_row'].rsplit('/', 1)[1] if 'workspace_row' in r else r['file'].rsplit('/', 1)[1]: r['replacement_sha256'] for r in bundle['rows']}
for ch, c in C.items():
    check(repl.get(c['scene']) == c['scene_sha256'] == sha(F/'counting-build02-root'/('csd-counting02-'+c['scene'])), f'{ch} scene {c["scene"]} = bundle03 staged')
    if c['kind'] == 'native-combined01':
        check(sha(F/'combined-build01-root-frozen01'/('csd-combined01-'+c['scene'])) == c['scene_sha256'], f'{ch} scene = combined01 frozen copy')
helpers = {'native-combined01': [F/'combined-delivery-root01/csd-combined01-timeline.py', F/'combined-preview-root01/csd-combined01-preview.py'],
           'native-counting02': [F/'counting-delivery02-root/csd-counting02-timeline.py', F/'counting-preview02-root/csd-counting02-preview.py']}
for kind, paths in helpers.items():
    for p in paths:
        expected = ast.literal_eval(literal(p, 'EXPECTED'))
        mine = {ch: (c['frames'], c['timeline_normalized']) for ch, c in C.items() if c['kind'] == kind}
        check({k: v[:2] for k, v in expected.items()} == mine, f'{kind} frames and normalized timelines = {p.name} EXPECTED')
        text = p.read_text()
        check(f"JOB = D/'jobs/{K[kind]['timeline' if 'timeline' in p.name else 'preview']}'" in text, f'{p.name} job name')
        check(K[kind]['comparison'] in text and f"tests_passed'] == {K[kind]['tests']}" in text, f'{p.name}: comparison name and {K[kind]["tests"]} tests')
        check(f"['selected_cues'] == {K[kind]['repaired_cues']}" in text, f'{p.name}: {K[kind]["repaired_cues"]} repaired cues')
for kind, (build, delivery, pictures, after) in {
        'native-combined01': (F/'combined-build01-root-frozen01/csd-combined01-build.py', F/'combined-delivery-root01/csd-combined01-delivery.py',
                              F/'combined-pictures-root01/csd-combined01-pictures.py', F/'combined-pictures-root01/csd-combined01-pictures-after-preview.py'),
        'native-counting02': (F/'counting-build02-root/csd-counting02-build.py', F/'counting-delivery02-root/csd-counting02-delivery.py',
                              F/'counting-pictures02-root/csd-counting02-pictures.py', F/'counting-pictures02-root/csd-counting02-pictures-after-preview.py')}.items():
    k, b, d, p, a = K[kind], build.read_text(), delivery.read_text(), pictures.read_text(), after.read_text()
    check(f"JOB_NAME = '{k['build']}'" in b and f"BINARY_NAME = '{k['binary']}'" in b and f"SOURCE = D/'{k['source']}'" in b, f'{kind} build job, binary and source')
    check(f"JOB = R / 'jobs/{k['delivery']}'" in d and k['delivery_record'] in d and "wavs_byte_identical=105" in d, f'{kind} delivery job and record')
    check(f"JOB = R/'jobs/{k['pictures']}'" in p and f"PREVIEW = R/'jobs/{k['preview']}'" in p and f"Q/'{k['sampling']}'/('frames-'+chapter+'-all.txt')" in p
          and "job=JOB/chapter" in p and "(job/'pictures.json')" in p, f'{kind} pictures job, preview, sampling folder, per-chapter record')
    check(f"INPUTS=Q/'{k['pictures_inputs']}'" in a, f'{kind} pictures inputs file name')
local_frames = {'01_the_bump': F/'combined-pictures-root01/frames-01_the_bump-all.txt', '08_loop_sums': F/'combined-pictures-root01/frames-08_loop_sums-all.txt',
                '09_boundary': F/'combined-pictures-root01/frames-09_boundary-all.txt', '11_counting': F/'counting-pictures02-root/frames-11_counting-all.txt'}
for ch, p in local_frames.items():
    check(sha(p) == C[ch]['sampling_sha256'], f'{ch} sampling list = local root copy')
check(sha(Q/C['09_boundary']['prior']) == C['09_boundary']['prior_sha256'], '09 prior775 = combined pictures01 pin')
check("prior=Q/'csd-native855-prior775.json'" in (F/'combined-pictures-root01/csd-combined01-pictures.py').read_text(), 'combined pictures01 retains prior775 for 09')
summary = json.loads((Q/'completed-master993-summary.json').read_text())
by = {r['chapter']: r for r in summary['rows']}
check(all(by[ch]['frames'] == c['frames'] for ch, c in C.items()), 'frames = master993 rows')
check(summary['boundary_replay']['capture']['sha256'] == C['09_boundary']['staged_capture'], '09 staged capture = master993 boundary_replay')
m533 = json.loads((Q/'css352-import371-08-loop-sums-manifest533.json').read_text())
r533 = {r['path']: r['sha256'] for r in m533['files']}
css = '/data/projects/jmev-css-render-worker-20261107/jobs/css-reviewed-final-%s352/render/soundtrack/%s/soundtrack_0.wav'
check(rows.get(css % ('01-the-bump', '01_the_bump')) == C['01_the_bump']['staged_capture'], '01 staged capture = manifest392 row')
check(r533.get(css % ('08-loop-sums', '08_loop_sums')) == C['08_loop_sums']['staged_capture'], '08 staged capture = manifest533 row')
check(C['11_counting']['staged_capture'] == '82bdc443218a5a761c678244089bc8d983a01466c77a7efed23ea358a4ec490a', '11 staged capture = root CSD sha256sum')

# 5. launch: same guard command shape as 968; runner path shared; proofs before any write
L, OL = LAUNCH.read_text(), (Q/'csd-native968-launch.py').read_text()
check(sha(Q/'csd-native968-launch.py') == '47b8f22b8f07d0770465e696d5c8094218a2f613fc5dc79b710701e759142236', '968 launch is the pinned 47b8f22b')
check("RUNNER = Q/'csd-native-repair-final01-run.py'" in L and str(run.RUNNER).endswith('csd-native-repair-final01-run.py'), 'launch and run name one runner')
check("'--phase', 'native',\n                    '--job', JOB.name, '--inputs', str(INPUTS), '--', sys.executable,\n                    '-I', '-B', str(RUNNER), '--chapter', chapter], check=True)" in L
      and "'--phase', 'native',\n                    '--job', JOB.name, '--inputs', str(INPUTS), '--', sys.executable,\n                    '-I', '-B', str(Q/'csd-native968-run.py')], check=True)" in OL,
      'guard command equals 968 lines 86-88 plus --chapter')
order = [L.index(s) for s in ("assert not JOB.exists() and not INPUTS.exists()", "run.unresolved(chapter)", "run.proofs(chapter)",
                              "with INPUTS.open('x')", "subprocess.run(")]
check(order == sorted(order), 'launch order: no job, pins resolved, all run proofs, then inputs written exclusively, then guard')
for line in ("assert sha(source) == d['inputs_sha256']", "with INPUTS.open('x') as f:", "success(JOB/'resource-receipt.json')"):
    check(line in L and line in OL, '968 launch line kept: '+line)

for line in notes + problems:
    print(line)
print(f'\n{len(notes)} passed, {len(problems)} failed')
sys.exit(1 if problems else 0)
