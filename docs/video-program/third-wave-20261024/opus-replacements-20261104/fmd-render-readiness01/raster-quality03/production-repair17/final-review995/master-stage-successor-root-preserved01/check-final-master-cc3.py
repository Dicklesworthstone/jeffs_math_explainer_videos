"""Local checks of the master993 / stage994 successors and the after-final02 launcher (CC3, read-only).

Usage: python3 -I -B check-final-master-cc3.py HERE RENDER_DIR PICTURES_DIR CC0_DIR Q WORK

HERE holds the three new helpers; RENDER_DIR and PICTURES_DIR are the sealed final-render-cc3 and
final-pictures-cc3 folders; CC0_DIR is CC0's final4k-impl01; Q is the repo's production-repair17
folder; WORK is a new empty folder for synthetic records. Nothing on CSD is read or run. Helpers are
compiled, parsed and imported (they define tables and functions only; PIL is stubbed). The compose
step and the master's gates() run on synthetic stage records over master993's manifest, rebuilt
locally from manifest625 and master993's summary; film() and the chapter gates are stubbed there.
"""
import ast
import contextlib
import difflib
import hashlib
import io
import importlib.util
import json
import subprocess
import sys
import types
from pathlib import Path

HERE, RENDER, PICS, CC0, Q, WORK = (Path(p) for p in sys.argv[1:7])
assert WORK.is_dir() and not any(WORK.iterdir()), 'WORK must be a new empty folder'
M, S, A = HERE/'combined-master-repaired01.py', HERE/'csd-native-repair-stage01.py', HERE/'csd-native-repair-pictures01-after-final02.py'
M993, S994, A969 = Q/'combined-master993.py', Q/'csd-native994-stage-launch.py', PICS/'csd-native-repair-pictures01-after-final.py'
MAT, GATE = CC0/'cc0-native-materialize-repaired01.py', CC0/'cc0-native-master-gate-repaired01.py'
RUN, PIC = RENDER/'csd-native-repair-final01-run.py', PICS/'csd-native-repair-pictures01.py'
problems, notes, findings, info = [], [], [], []


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def check(ok, what):
    (notes if ok else problems).append(('ok   ' if ok else 'FAIL ')+what)


def load(path, name, register=False):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    if register:
        sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def refuses(fn, *args):
    try:
        fn(*args)
    except Exception as error:                  # any refusal: assert, missing file, bad key
        return repr(error)[:90] or 'refused'
    return None


def tops(path):
    """Top-level functions (AST dump) and assignments (source) of a module."""
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


# 1. syntax and the pinned bases
for p in (M, S, A):
    compile(p.read_text(), str(p), 'exec')
    check(True, 'compiles: '+p.name)
for p, digest in ((M993, '1955ca3202c8a4f382fea1682ac754eadf19fd708b2ef983c3f91f12607b6edb'),
                  (S994, 'f064103286a0e1c57bba3eece9084832b1c9f1111c8d5ef22fcfb739c8ce8de9'),
                  (A969, 'c27f4ae2'), (RUN, '6dceac47'), (PIC, '5fa242b8'),
                  (MAT, '85557673b898ad406f36f8b5177db3717932e0bce8f0516e089bcb524aaaf957'),
                  (GATE, '2a855393b4ffc55614f3003784236f958c38888605fea86407cb59dde902e739')):
    check(sha(p).startswith(digest), f'base {p.name} is {digest[:8]}')

# 2. master successor: only chapter_gate, gates, launch, master and the new unresolved() differ from 993
f993, n993 = tops(M993)
fnew, nnew = tops(M)
changed = {k for k in set(f993) | set(fnew) if f993.get(k) != fnew.get(k)}
check(changed == {'chapter_gate', 'gates', 'launch', 'master', 'unresolved'}, f'993 functions changed: {sorted(changed)}')
moved = {k for k in set(n993) | set(nnew) if n993.get(k) != nnew.get(k)}
check(moved == {'VERSION', 'JOB', 'INPUTS', 'REPAIRED', 'REPAIRED_KINDS', 'BASE_MANIFEST', 'BASE_MANIFEST_SHA',
                'REPAIRED_GATE_NAME', 'REPAIRED_GATE_SHA'}, f'993 module names changed or added: {sorted(moved)}')
plus, minus = diffstat(M993, M)
info.append(f'info diff from 993: +{plus} -{minus}')
gate_text = GATE.read_text()
used = sorted({n.attr for n in ast.walk(ast.parse(gate_text)) if isinstance(n, ast.Attribute)
               and isinstance(n.value, ast.Name) and n.value.id == 'base'})
check(all(u in fnew or u in nnew for u in used), f"every base.<name> CC0's gate reads is defined in the successor ({len(used)} names)")
mt = M.read_text()
for line in ("boundary_rows = [r for r in records if r['chapter'] == BOUNDARY_CHAPTER and r['kind'] == REPAIRED[BOUNDARY_CHAPTER]]",
             "capture_check_before_master=boundary['capture_check_before_master'])",
             "if r['kind'] in ('native521',)+REPAIRED_KINDS else",
             "return load(native, 'native_repaired').gate(sys.modules[__name__], row, frames_of)",
             "assert not unresolved(), 'unresolved pins, refusing: '+', '.join(unresolved())"):
    check(line in mt, 'master successor line: '+line[:80])
check('native844' not in mt.split('def chapter_gate', 1)[1].split('def gates', 1)[0],
      'chapter_gate has no native844 branch (gates() refuses boundary313/native844 rows)')
for marker in ('def schedule(', 'def replays_of', "'captions', ['all']", 'def preflight', 'def film', 'def claimed'):
    check(marker in mt and marker in M993.read_text(), 'kept from 993: '+marker)

# 3. import the successor and the four tables it must agree with
pil = types.ModuleType('PIL')
pil.Image, pil.ImageDraw = types.ModuleType('PIL.Image'), types.ModuleType('PIL.ImageDraw')
sys.modules.update({'PIL': pil, 'PIL.Image': pil.Image, 'PIL.ImageDraw': pil.ImageDraw})
m = load(M, 'master_successor', register=True)
mat, gate, run, x = load(MAT, 'cc0_mat'), load(GATE, 'cc0_gate'), load(RUN, 'cc3_run'), load(PIC, 'cc3_pictures')
st = load(S, 'cc3_stage')
check(m.unresolved() == ['VERSION', 'REPAIRED_GATE_NAME', 'REPAIRED_GATE_SHA'], 'master successor pins pending: '+', '.join(m.unresolved()))
kinds = {ch: c['kind'] for ch, c in mat.CHAPTERS.items()}
check(m.REPAIRED == kinds == {ch: c['kind'] for ch, c in gate.CHAPTERS.items()} == {ch: c['kind'] for ch, c in run.CHAPTERS.items()},
      'REPAIRED = materializer, gate and run tables: 01/08/09 native-combined01, 11 native-counting02')
check(set(m.REPAIRED_KINDS) == set(kinds.values()) and set(m.CHANGED) >= set(m.REPAIRED), 'both kinds; repaired chapters are 993 CHANGED chapters')
for ch, c in mat.CHAPTERS.items():
    r, chain = run.CHAPTERS[ch], run.CHAINS[c['kind']]
    check(all(chain[k] == c[k] for k in ('build', 'delivery', 'timeline', 'preview', 'pictures')) and r['frames'] == c['frames']
          and r['scene_sha256'] == c['source_sha'] == gate.CHAPTERS[ch]['source_sha'] and r['staged_capture'] == c['staged993'],
          f'{ch}: chain jobs, frames, scene and staged capture agree (run, materializer, gate)')
    check(chain['pictures_inputs'] == st.CHAIN_INPUTS[c['kind']]+'-pictures-inputs.json', f'{ch}: chain inputs prefix '+st.CHAIN_INPUTS[c['kind']])
check(st.RUNNER == run.RUNNER == x.RUNNER and str(st.PICTURES).endswith('/csd-native-repair-pictures01.py')
      and "HELPER=Q/'csd-native-repair-pictures01.py'" in A.read_text(), 'stage RUNNER/PICTURES = the names the run, extraction and after-final helpers bind')
check(st.MANIFEST == m.BASE_MANIFEST and st.MANIFEST_SHA == m.BASE_MANIFEST_SHA == mat.MANIFEST_SHA == gate.MANIFEST993_SHA,
      'one base manifest: master993 8da77ed7 (stage, master, materializer, gate)')
check(st.unresolved() == ['WORKER_NAME', 'WORKER_SHA', 'RUNNER_SHA', 'PICTURES_SHA'] and refuses(st.helpers),
      'stage pins pending and helpers() refuses: '+', '.join(st.unresolved()))
summary = json.loads((Q/'completed-master993-summary.json').read_text())
check(summary['manifest']['path'] == str(m.BASE_MANIFEST) and summary['manifest']['sha256'] == m.BASE_MANIFEST_SHA,
      'BASE_MANIFEST is the manifest completed-master993-summary.json names')

# chain input names: the INPUTS each local chain launcher writes
helpers_text = ''.join(p.read_text() for p in (Q/'final-review995').rglob('*.py'))
for prefix in st.CHAIN_INPUTS.values():
    for part in ('', '-delivery', '-timeline', '-preview', '-pictures'):
        name = prefix+part+'-inputs.json'
        check(f"INPUTS = Q/'{name}'" in helpers_text or f"INPUTS=Q/'{name}'" in helpers_text, 'a chain launcher writes Q/'+name)

# 4. the stage successor against 994 and CC0's materializer and gate
s994, snew = S994.read_text().splitlines(), S.read_text()
kept = [n for n in range(23, 145) if s994[n-1].strip()]
same = [n for n in kept if s994[n-1].strip() in {l.strip() for l in snew.splitlines()}]
check(len(same) >= 60, f'994 lines kept verbatim: {len(same)} of {len(kept)} non-blank code lines')
plus, minus = diffstat(S994, S)
info.append(f'info diff from 994: +{plus} -{minus} (docstring and the compose mode are new)')
tree = ast.parse(snew)
call = next(n for n in ast.walk(tree) if isinstance(n, ast.Call) and isinstance(n.func, ast.Attribute)
            and n.func.attr == 'run' and isinstance(n.args[0], ast.List))
ns = dict(sys=types.SimpleNamespace(executable='/usr/bin/python3'), str=str, D=Path('/D'), a=types.SimpleNamespace(job='JOB'),
          inputs=Path('/Q/I.json'), worker=Path('/Q/W.py'), ch='08_loop_sums', proof_path=Path('/Q/P.json'), original=Path('/Q/M.json'))
argv = eval(compile(ast.Expression(call.args[0]), 'stage', 'eval'), ns)
inner = argv[argv.index('--')+1:]
check(len(inner) == 14 and inner[:6] == ['/usr/bin/python3', '-I', '-B', '/Q/W.py', '--chapter', '08_loop_sums']
      and inner[6:12] == ['--job', 'JOB', '--inputs', '/Q/I.json', '--proofs', '/Q/P.json'] and inner[12:] == ['--manifest', '/Q/M.json'],
      'guarded worker argv = CC0 gate command[:14] and the materializer state command')
check(argv[:argv.index('--')] == ['/usr/bin/python3', '-I', '-B', '/D/guard_csd02.py', '--phase', 'native', '--job', 'JOB', '--inputs', '/Q/I.json'],
      'guard prefix = 994 lines 132-133')
pattern = "r'csd-native-stage%s-%s-a[1-9][0-9]*'"
check(pattern in snew and "csd-native-stage%s-%s-a[1-9][0-9]*' % (STAGE_VERSION, CHAPTER[:2])" in MAT.read_text()
      and "csd-native-stage%s-%s-a[1-9][0-9]*' % (STAGE_VERSION, CHAPTER[:2])" in gate_text, 'stage job regex = materializer and gate')
def proof_keys(text):
    return {n.slice.value for n in ast.walk(ast.parse(text)) if isinstance(n, ast.Subscript) and isinstance(n.value, ast.Name)
            and n.value.id == 'proofs' and isinstance(n.slice, ast.Constant)}
check(proof_keys(MAT.read_text()) | proof_keys(gate_text) == {'prerequisites', 'extraction', 'root_review', 'independent_review'}
      and 'proofs=dict(prerequisites=prerequisites,extraction=str(extraction),root_review=str(a.root_review),independent_review=str(a.independent_review))' in snew,
      'proofs keys written = keys the materializer and gate read')
for line in ("assert run.CHAPTERS[ch]['frames']==c['frames'] and not any(run.unresolved(k) for k in run.CHAPTERS)",
             "(c['build'],prefix+'-inputs.json'),", "(final.name,final.name+'-inputs.json'),", "(picked.name,picked.name+'-inputs.json')]:",
             "_,_,used=run.proofs(ch)", "frames=report['selected_frames'];assert frames==sorted(set(frames)) and set(previewed['selected_frames'])<=set(frames)",
             "assert review['source_sha256']==c['source_sha']", "made=read(job/'materialized.json');assert made['unchanged_other_manifest_rows']==14 and made['chapter']==ch",
             "assert x.RUNNER==run.RUNNER==RUNNER and final.name==w.FINAL_JOBS[ch]"):
    check(line in snew, 'stage line: '+line[:84])
check("(c['build'],c['delivery'],c['timeline'],c['preview'],c['pictures'],FINAL)" in MAT.read_text()
      and "(c['build'],c['delivery'],c['timeline'],c['preview'],c['pictures'],FINAL)" in gate_text
      and 'assert receipt in actual_jobs' in gate_text, 'stage prerequisites cover the materializer and gate job sets plus the extraction receipt')
check(set(json.loads((Q/'completed-pictures969.json').read_text())) >= {'movie', 'movie_sha256', 'full_decode', 'frames', 'resolution',
      'selected_frames', 'pictures', 'sheets'}, 'extraction record fields the stage reads exist in 969 (kept by the successor)')
order = [snew.index(s) for s in ('w,run,x=helpers()', "assert all(not p.exists() for p in (job,inputs,proof_path,publish_manifest))",
                                 '_,_,used=run.proofs(ch)', 'write(proof_path,proofs)', 'write(inputs,', 'subprocess.run(', 'publish_manifest.open')]
check(order == sorted(order), 'stage order: pins, no duplicate, proofs read, proofs then inputs written, guard, publish')
for args in (['stage', '--chapter', '01_the_bump', '--job', 'csd-native-stage1-01-a1', '--root-review', '/r', '--independent-review', '/i'],
             ['compose', '--stage', 'a', 'b', 'c', 'd']):
    out = subprocess.run([sys.executable, '-I', '-B', str(S)]+args, cwd=WORK, capture_output=True, text=True)
    check(out.returncode == 1 and 'unresolved pins' in out.stderr and not any(WORK.iterdir()), f'stage {args[0]}: refuses with pins unset, writes nothing')
out = subprocess.run([sys.executable, '-I', '-B', str(M), 'check', '--manifest', '/x.json'], cwd=WORK, capture_output=True, text=True)
check(out.returncode == 1 and 'unresolved pins, refusing' in out.stderr and not any(WORK.iterdir()), 'master successor check mode refuses with pins unset')

# 5. master993's manifest rebuilt locally, then compose and the master's gates() on synthetic stages
m625 = json.loads((Q/'root-master-manifest625.json').read_text())
nine = next(r for r in summary['rows'] if r['chapter'] == '09_boundary')
QQ = '/data/projects/jmev-stroke-probe-20261106/qualification01/production-repair17'
row09 = dict(chapter='09_boundary', kind='native844', job=nine['job'], worker=nine['worker'], worker_sha256=nine['worker_sha256'],
             inputs=QQ+'/'+nine['job']+'-inputs.json', proofs=nine['local_proofs']['path'], picture_review=nine['native_final_samples']['record'])
base = dict(m625, chapters=[row09 if r['chapter'] == '09_boundary' else r for r in m625['chapters']])
fake_w = types.SimpleNamespace(STAGE_VERSION='999', CHAPTERS=mat.CHAPTERS)
st.helpers = lambda: (fake_w, None, None)


def dump(path, value):
    with path.open('x') as f:
        json.dump(value, f, indent=2); f.write('\n')


def build(root, tweak=lambda ch, parts: None):
    """A fresh synthetic CSD tree: master993's manifest and one successful stage per chapter, each made as
    the materializer makes it (one row replaced) and published to Q as 994 publishes it."""
    qdir, jobs = root/'Q', root/'jobs'
    for p in (root, qdir, jobs):
        p.mkdir()
    basefile = qdir/'csd-native-stage970-09-a2-master-manifest.json'
    dump(basefile, base)
    names = {}
    for ch, c in mat.CHAPTERS.items():
        name = names[ch] = 'csd-native-stage999-%s-a1' % ch[:2]
        new_row = dict(chapter=ch, kind=c['kind'], job=name, worker=QQ+'/W.py', worker_sha256='w', inputs=QQ+'/'+name+'-inputs.json',
                       proofs=QQ+'/'+name+'-proofs.json', picture_review=dict(path=QQ+'/r.json', sha256='r'))
        parts = dict(receipt=dict(admitted=True, finished=True, exit=0, inputs_sha256='h', final_inputs_sha256='h'),
                     manifest=dict(base, chapters=[new_row if r['chapter'] == ch else r for r in base['chapters']]),
                     made=dict(chapter=ch, unchanged_other_manifest_rows=14, original_manifest_sha256=m.BASE_MANIFEST_SHA))
        tweak(ch, parts)
        job = jobs/name
        job.mkdir()
        dump(job/'resource-receipt.json', parts['receipt']); dump(job/'master-manifest.json', parts['manifest'])
        dump(job/'materialized.json', parts['made'])
        published = parts.get('published', dict(base, chapters=[new_row if r['chapter'] == ch else r for r in base['chapters']]))
        dump(qdir/(name+'-master-manifest.json'), published)
    st.Q, st.D, st.MANIFEST = qdir, root, basefile
    return qdir, jobs, basefile, names


qdir, jobs, basefile, names = build(WORK/'positive')
check(sha(basefile) == m.BASE_MANIFEST_SHA, 'manifest625 + master993 row 09 rebuilds 8da77ed7 byte for byte')
order4 = [names[k] for k in ('09_boundary', '01_the_bump', '11_counting', '08_loop_sums')]
shown = io.StringIO()
with contextlib.redirect_stdout(shown):
    st.compose(types.SimpleNamespace(stage=order4))
composed_path = qdir/'csd-native-stage999-composed-master-manifest.json'
said = json.loads(shown.getvalue())
check(said['manifest'] == str(composed_path) and said['manifest_sha256'] == sha(composed_path) and said['stages'] == order4,
      'compose prints the composed manifest path, its sha256 and the four stages')
composed = json.loads(composed_path.read_text())
stage_rows = {ch: next(r for r in json.loads((jobs/n/'master-manifest.json').read_text())['chapters'] if r['chapter'] == ch) for ch, n in names.items()}
check([r for r in composed['chapters'] if r['chapter'] not in m.REPAIRED] == [r for r in base['chapters'] if r['chapter'] not in m.REPAIRED]
      and all(next(r for r in composed['chapters'] if r['chapter'] == ch) == row for ch, row in stage_rows.items())
      and [k for k in composed if k != 'chapters'] == [k for k in base if k != 'chapters'],
      'compose: 11 rows = 8da77ed7, 4 rows = each stage row, stages in any order')
check(refuses(st.compose, types.SimpleNamespace(stage=order4)) is not None, 'compose refuses when the composed manifest exists (no overwrite)')


def second_row(ch, parts):
    if ch == '01_the_bump':
        rows = parts['manifest']['chapters']
        rows[1] = dict(rows[1], job='changed')
        parts['published'] = parts['manifest']


def other_copy(ch, parts):
    if ch == '08_loop_sums':
        parts['published'] = base


def failed(ch, parts):
    if ch == '11_counting':
        parts['receipt'] = dict(parts['receipt'], exit=1)


def other_base(ch, parts):
    if ch == '09_boundary':
        parts['made'] = dict(parts['made'], original_manifest_sha256='6fde63a4')


for label, tweak, pick in (('a stage that changes a second row', second_row, None), ('a Q copy unlike the job manifest', other_copy, None),
                           ('a failed stage', failed, None), ('a stage made from another base', other_base, None),
                           ('a repeated chapter', lambda ch, parts: None, lambda n: [n['01_the_bump']]*2+[n['11_counting'], n['08_loop_sums']]),
                           ('three stages', lambda ch, parts: None, lambda n: [n['01_the_bump'], n['11_counting'], n['08_loop_sums']]),
                           ('a stage job that does not exist', lambda ch, parts: None,
                            lambda n: [n['01_the_bump'], n['11_counting'], n['08_loop_sums'], 'csd-native-stage999-09-a2'])):
    root = WORK/('case-'+label.replace(' ', '-'))
    _, _, _, n = build(root, tweak)
    check(refuses(st.compose, types.SimpleNamespace(stage=pick(n) if pick else [n[k] for k in n])) is not None, 'compose refuses '+label)
    check(not (root/'Q/csd-native-stage999-composed-master-manifest.json').exists(), 'no composed manifest after refusing '+label)
st.Q, st.D, st.MANIFEST = qdir, WORK/'positive', basefile

# the master successor's gates() on the composed manifest (film() and the chapter gates stubbed)
frames = {r['chapter']: r['frames'] for r in summary['rows']}
registry = [r['chapter'] for r in base['chapters']]
m.Q, m.BASE_MANIFEST = qdir, basefile
m.film = lambda: (registry, dict(frames), set())
real_gate = m.chapter_gate
m.chapter_gate = lambda row, frames_of: (dict(chapter=row['chapter'], kind=row['kind'], frames=frames_of[row['chapter']]), set())
_, records, _ = m.gates(composed_path)
check([r['kind'] for r in records] == [m.REPAIRED.get(r['chapter'], b['kind']) for r, b in zip(records, base['chapters'])]
      and sum(r['frames'] for r in records) == m.TOTAL_FRAMES, 'gates(): composed manifest passes; 4 repaired kinds, 11 kept kinds, 133662 frames')
def bad_manifest(edit, label):
    d = json.loads(composed_path.read_text()); edit(d['chapters'])
    p = qdir/('bad-%s.json' % label)
    dump(p, d)
    return refuses(m.gates, p)
idx = {r['chapter']: i for i, r in enumerate(base['chapters'])}
cases = {'kept-row-edited': lambda rows: rows.__setitem__(idx['04_reading_color'], dict(rows[idx['04_reading_color']], job='x')),
         '09-native844': lambda rows: rows.__setitem__(idx['09_boundary'], row09),
         '11-combined01': lambda rows: rows.__setitem__(idx['11_counting'], dict(rows[idx['11_counting']], kind='native-combined01')),
         '01-old-combined': lambda rows: rows.__setitem__(idx['01_the_bump'], base['chapters'][idx['01_the_bump']]),
         'order': lambda rows: rows.reverse()}
for label, edit in cases.items():
    check(bad_manifest(edit, label) is not None, 'gates() refuses: '+label)
check(refuses(m.gates, basefile) is not None, 'gates() refuses master993 manifest itself (no repaired row)')
# chapter_gate dispatch: pinned gate only, both kinds, no native844
fake = qdir/'fake-gate.py'
fake.write_text('def gate(base, row, frames_of):\n    return dict(chapter=row["chapter"], kind=row["kind"], frames=frames_of[row["chapter"]], via="repaired"), set()\n')
m.chapter_gate = real_gate
row = next(r for r in composed['chapters'] if r['chapter'] == '11_counting')
check(refuses(m.chapter_gate, row, frames) is not None, 'chapter_gate refuses a repaired row while pins are unset')
m.VERSION, m.REPAIRED_GATE_NAME, m.REPAIRED_GATE_SHA = '999', fake.name, sha(fake)
check(all(m.chapter_gate(next(r for r in composed['chapters'] if r['chapter'] == ch), frames)[0].get('via') == 'repaired' for ch in m.REPAIRED),
      'chapter_gate sends all four repaired rows (both kinds) to the pinned gate')
m.REPAIRED_GATE_SHA = '0'*64
check(refuses(m.chapter_gate, row, frames) is not None, 'chapter_gate refuses a gate file that is not the pinned bytes')
m.REPAIRED_GATE_SHA = sha(fake)
check(refuses(m.chapter_gate, row09, frames) is not None, 'chapter_gate refuses a native844 09 row')
check(refuses(m.chapter_gate, dict(row, kind='native-combined01'), frames) is not None, 'chapter_gate refuses 11 as native-combined01')

# 6. after-final02 = the sealed after-final plus one pre-wait chapter assert and a docstring note
plus, minus = diffstat(A969, A)
at = A.read_text()
check((plus, minus) == (4, 1) and at.index('assert chapter in run.CHAPTERS,chapter') < at.index('while time.monotonic()<deadline:'),
      f'after-final02: +{plus} -{minus}; chapter checked before the 30 h wait (CC2 L1)')

# 7. findings in CC0's drafts (reported to CC0; not failures of these helpers)
part = 'stems/01_the_bump.music.f32.wav.part'
keys = ['stems/'+n for n in run.CHAPTERS['01_the_bump']['stems'] if '/' not in n]
got = {k[len('stems/'):] for k in keys if k.startswith(part) and k.endswith('.wav')}
cuelog = (Q/'final01-terminal142/01_the_bump.cuelog.tsv').read_text()
if not got and '/score/stems/01_the_bump.music.f32.wav\t' in cuelog and part in MAT.read_text().replace("'stems/'+CHAPTER+'.music.f32.wav.part'", part):
    findings.append('FINDING materializer: 01 has one unsplit score stem (01_the_bump.music.f32.wav, run table and final01 cuelog), '
                    'so stem_names from the .wav.part prefix is empty and `assert stem_names` refuses chapter 01')

for line in notes + problems + info + findings:
    print(line)
print(f'\n{len(notes)} passed, {len(problems)} failed, {len(findings)} finding(s) for CC0')
sys.exit(1 if problems else 0)
