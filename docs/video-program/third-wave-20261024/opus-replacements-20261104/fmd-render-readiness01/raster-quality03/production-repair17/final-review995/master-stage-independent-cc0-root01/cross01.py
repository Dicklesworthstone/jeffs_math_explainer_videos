"""cross01.py: CC0's independent static cross-check of CC3's 993/994 successors (read-only; writes only its own output).

Inputs (all read, none run as a job): CC3's master and stage successors, CC3's run and final-extraction helpers,
CC0's materializer impl02 and gate impl01, master993 and 994, root's chain launchers and the scope02 controller.
1. by AST: which top-level statements of the master successor differ from 993, and of the stage from 994;
2. every attribute the stage reads from the run/extraction helpers and the materializer exists, and their
   per-chapter values agree with the gate's (chains, scenes, frames, captures, normalized tables, records);
3. names that must match across files: final jobs, admissions, extraction jobs, guard inputs, base manifest;
4. every base.<name> the gate reads is a top-level name of the master successor;
5. the stage's worker argv has the order the materializer and gate assert; compose reads only keys the
   materializer writes; the materializer's row keys are the gate's.

usage: python3 -I -B cross01.py PR17_DIR CC3_DIR CC0_IMPL01 CC0_IMPL02 OUT_DIR
"""
import ast
import hashlib
import importlib.util
import re
import sys
from pathlib import Path

PR17, CC3, IMPL01, IMPL02, OUT = (Path(x) for x in sys.argv[1:6])
F = PR17/'final-review995'
lines, bad = [], []
say = lines.append


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def check(name, ok):
    say('%-4s %s' % ('ok' if ok else 'FAIL', name))
    if not ok:
        bad.append(name)


def load(p, name):
    spec = importlib.util.spec_from_file_location(name, p)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


FILES = dict(
    m993=PR17/'combined-master993.py', s994=PR17/'csd-native994-stage-launch.py',
    master=CC3/'final-master-cc3/combined-master-repaired01.py', stage=CC3/'final-master-cc3/csd-native-repair-stage01.py',
    run=CC3/'final-render-cc3/csd-native-repair-final01-run.py', pictures=CC3/'final-pictures-cc3/csd-native-repair-pictures01.py',
    worker=IMPL02/'cc0-native-materialize-repaired02.py', gate=IMPL01/'cc0-native-master-gate-repaired01.py')
for k, p in FILES.items():
    say('     %-8s %s %s' % (k, sha(p)[:8], p.name))
preserved = F/'master-stage-successor-root-preserved01'
check('root-preserved master and stage equal CC3 copies',
      sha(preserved/'combined-master-repaired01.py') == sha(FILES['master'])
      and sha(preserved/'csd-native-repair-stage01.py') == sha(FILES['stage']))


def top(p):
    """Top-level statements keyed by name (defs, assignments, the main guard)."""
    out = {}
    for n in ast.parse(p.read_text()).body:
        if isinstance(n, (ast.FunctionDef, ast.ClassDef)):
            out['def '+n.name] = ast.dump(n)
        elif isinstance(n, ast.Assign):
            out['= '+','.join(ast.unparse(t) for t in n.targets)] = ast.dump(n)
        elif isinstance(n, ast.If):
            out['if '+ast.unparse(n.test)] = ast.dump(n)
        elif isinstance(n, ast.Expr) and isinstance(n.value, ast.Constant):
            out['docstring'] = ast.dump(n)
        else:
            out.setdefault('other', []).append(ast.dump(n))
    return out


for old, new in (('m993', 'master'), ('s994', 'stage')):
    a, b = top(FILES[old]), top(FILES[new])
    changed = sorted(k for k in set(a) & set(b) if a[k] != b[k])
    say('     %s -> %s: changed %s; added %s; removed %s' % (old, new, changed, sorted(set(b)-set(a)), sorted(set(a)-set(b))))
a, b = top(FILES['m993']), top(FILES['master'])
check('master: only docstring, VERSION/JOB/INPUTS, chapter_gate, gates, launch, master and the main guard change',
      {k for k in set(a) & set(b) if a[k] != b[k]} <= {'docstring', '= VERSION', '= JOB', '= INPUTS', 'def chapter_gate',
                                                        'def gates', 'def launch', 'def master', "if __name__ == '__main__'"}
      and not set(a)-set(b))

master = load(FILES['master'], 'master_successor')
stage = load(FILES['stage'], 'stage_successor')
run = load(FILES['run'], 'run_helper')
pictures = load(FILES['pictures'], 'pictures_helper')
w = load(FILES['worker'], 'materializer02')
g = load(FILES['gate'], 'gate01')

# 2. per-chapter agreement
check('same four chapters everywhere', set(run.CHAPTERS) == set(w.CHAPTERS) == set(g.CHAPTERS) == set(master.REPAIRED))
for ch in sorted(w.CHAPTERS):
    c, gc, r = w.CHAPTERS[ch], g.CHAPTERS[ch], run.CHAPTERS[ch]
    chain = run.CHAINS[r['kind']]
    agree = {
        'kind': (r['kind'], c['kind'], gc['kind'], master.REPAIRED[ch]),
        'frames': (r['frames'], c['frames'], gc['frames']),
        'scene sha': (r['scene_sha256'], c['source_sha'], gc['source_sha']),
        'scene file': (r['scene'], c['scene'], gc['scene']),
        'staged capture': (r['staged_capture'], c['staged993'], gc['staged993']),
        'preview pcm': (r['preview_pcm'], gc['preview_pcm']),
        'normalized': (r['timeline_normalized'], gc['normalized']),
        'binary name': (chain['binary'], c['binary'], gc['binary']),
        'tests': (chain['tests'], c['tests'], gc['tests']),
        'delivery record': (chain['delivery_record'], gc['delivery_record']),
        'comparison': (chain['comparison'], gc['comparison']),
        'source dir': (str(run.D/chain['source']), str(c['workspace'])),
    }
    for k in ('build', 'delivery', 'timeline', 'preview', 'pictures'):
        agree[k] = (chain[k], c[k], gc[k])
    wrong = [k for k, v in agree.items() if len(set(v)) != 1]
    check('%s: run helper, materializer, gate and master agree on %d fields%s' % (ch, len(agree), (' (differ: %s)' % wrong) if wrong else ''), not wrong)
    scene_run = run.D/chain['source']/run.SCENES/r['scene']
    scene_w = c['workspace']/w.SCENE/c['scene']
    check('%s: scene path the run proofs bind equals the materializer\'s' % ch, str(scene_run) == str(scene_w))
    # 3. names
    say('     %s: final %s; admission %s; extraction %s' % (ch, run.job(ch).name, run.admission(ch).name, pictures.job(ch).name))
    check('%s: final job name has the form CC3 names for FINAL_JOBS' % ch, run.job(ch).name == 'csd-native-repair-final01-'+ch.replace('_', '-'))
    check('%s: stage reads the preview extraction record where the chain writes it' % ch,
          str(run.D/'jobs'/c['pictures']/ch/'pictures.json') == str(run.D/'jobs'/chain['pictures']/ch/'pictures.json'))
check('extraction helper and run helper name the same runner as the stage', pictures.RUNNER == run.RUNNER == stage.RUNNER)
check('stage PICTURES is the extraction helper\'s own HELPER name', stage.PICTURES == pictures.HELPER)
check('base manifest: stage, materializer, gate and master all pin 8da77ed7',
      stage.MANIFEST_SHA == w.MANIFEST_SHA == g.MANIFEST993_SHA == master.BASE_MANIFEST_SHA ==
      '8da77ed79e9ccc8a06e703e570c85c44de5cd884a047eaed61355167e397e100')
import json
summary993 = json.loads((PR17/'completed-master993-summary.json').read_text())
check('base manifest name is the one master993 recorded', str(stage.MANIFEST) == str(master.BASE_MANIFEST) == summary993['manifest']['path'])

# guard inputs names against root's actual chain launchers and the scope02 controller
launchers = {
    'native-combined01': [F/'combined-build01-root-frozen01/csd-combined01-launch.py', F/'combined-delivery-root01/csd-combined01-after-build.py',
                          F/'combined-delivery-root01/csd-combined01-after-delivery.py', F/'combined-preview-root01/csd-combined01-after-timeline.py',
                          F/'combined-pictures-root01/csd-combined01-pictures-after-preview-scope02.py'],
    'native-counting02': [F/'counting-build02-root/csd-counting02-launch.py', F/'counting-delivery02-root/csd-counting02-after-build.py',
                          F/'counting-delivery02-root/csd-counting02-after-delivery.py', F/'counting-preview02-root/csd-counting02-after-timeline.py',
                          F/'counting-pictures02-root/csd-counting02-pictures-after-preview.py']}
for kind, paths in launchers.items():
    prefix = stage.CHAIN_INPUTS[kind]
    want = [prefix+s for s in ('-inputs.json', '-delivery-inputs.json', '-timeline-inputs.json', '-preview-inputs.json', '-pictures-inputs.json')]
    got = []
    for p in paths:
        m = re.search(r"^INPUTS\s*=\s*Q\s*/\s*'([^']+)'", p.read_text(), re.M)
        got.append(m[1] if m else None)
    check('%s: the stage\'s five guard inputs names are the launchers\' INPUTS (%s)' % (kind, ', '.join(map(str, got))), got == want)
    check('%s: run helper pictures_inputs equals the stage\'s' % kind, run.CHAINS[kind]['pictures_inputs'] == want[-1])

# 4. base.<name> in the gate
gate_tree = ast.parse(FILES['gate'].read_text())
names = sorted({n.attr for n in ast.walk(gate_tree) if isinstance(n, ast.Attribute) and isinstance(n.value, ast.Name) and n.value.id == 'base'})
missing = [n for n in names if not hasattr(master, n)]
check('every base.<name> the gate reads (%d) is defined in the master successor%s' % (len(names), (' (missing %s)' % missing) if missing else ''), not missing)

# 5. argv order, compose keys, row keys
stage_src = FILES['stage'].read_text()
argv = re.search(r"'--',sys\.executable,'-I','-B',str\(worker\),'--chapter',ch,'--job',a\.job,\s*'--inputs',str\(inputs\),'--proofs',str\(proof_path\),'--manifest',str\(original\)\]", stage_src)
check('stage worker argv: exe -I -B worker --chapter --job --inputs --proofs --manifest (the 14 items the materializer and gate assert)', bool(argv))
w_tree = ast.parse(FILES['worker'].read_text())
made_keys = set()
row_keys = set()
for n in ast.walk(w_tree):
    if isinstance(n, ast.Call) and getattr(n.func, 'id', '') == 'write' and len(n.args) == 2 and 'materialized' in ast.unparse(n.args[0]):
        made_keys = {k.arg for k in n.args[1].keywords}
    if isinstance(n, ast.Assign) and getattr(n.targets[0], 'id', '') == 'new_row':
        row_keys = {k.arg for k in n.value.keywords}
used = set(re.findall(r"made\['([a-z_0-9]+)'\]", stage_src))
check('stage and compose read only materialized.json keys the materializer writes (%s)' % sorted(used), used <= made_keys)
check('materializer row keys are exactly the gate\'s', row_keys == {'chapter', 'kind', 'job', 'worker', 'worker_sha256', 'inputs', 'proofs', 'picture_review'})
check('stage job regex equals the materializer\'s and the gate\'s',
      "r'csd-native-stage%s-%s-a[1-9][0-9]*'" in stage_src and "r'csd-native-stage%s-%s-a[1-9][0-9]*'" in FILES['worker'].read_text()
      and "r'csd-native-stage%s-%s-a[1-9][0-9]*'" in FILES['gate'].read_text())
check('materializer and gate require the same prerequisite jobs, all among the stage\'s seven',
      "(c['build'],c['delivery'],c['timeline'],c['preview'],c['pictures'],FINAL)" in FILES['worker'].read_text()
      and "(c['build'],c['delivery'],c['timeline'],c['preview'],c['pictures'],FINAL)" in FILES['gate'].read_text())
say('RESULT: %s' % ('all checks passed' if not bad else 'FAILED: '+'; '.join(bad)))
with open(OUT/'cross01.out', 'x', encoding='utf-8') as f:
    f.write('\n'.join(lines)+'\n')
print('\n'.join(lines))
