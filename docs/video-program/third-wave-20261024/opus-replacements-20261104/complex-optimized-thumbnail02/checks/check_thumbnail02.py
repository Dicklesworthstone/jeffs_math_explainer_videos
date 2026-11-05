#!/usr/bin/env python3
"""Minimal checks of thumbnail_optimized02.py in a temporary folder.

Real: the helper's code, its four pinned imports, PIL and the start script's shell. Fake: guard,
binary and job tree. It runs no guard or render, hashes no frozen input, samples no disk and writes
nothing in the repository. Run from the repository root:

    python3 -I -B <bundle>/checks/check_thumbnail02.py
"""
import ast
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
from PIL import Image

HERE = Path(__file__).resolve()
BUNDLE = HERE.parents[1]
SEALED = BUNDLE.parent / 'complex-optimized-downstream01'
SEALED_SUMS = 'bd8508c3e88bb9c529a8369663665c22a9aaa7e83d62a677594846b12443182a'
GUARD = """import json, sys
from pathlib import Path
tmp = Path(TMP)
job = tmp / 'B' / sys.argv[sys.argv.index('--job') + 1]
job.mkdir()
(job / 'resource-receipt.json').write_text(json.dumps({
    'admitted': True, 'exit': 0, 'finished': 'fixture', 'inputs_sha256': 'a', 'final_inputs_sha256': 'a'}))
(tmp / 'guard.json').write_text(json.dumps(sys.argv[1:]))
"""
BINARY = """#!PYTHON -I
import shutil, sys
from pathlib import Path
assert sys.argv[1:4] == ['thumbnail', '--res', '3840x2160']
out = Path(sys.argv[sys.argv.index('--out') + 1])
out.mkdir()
shutil.copy(PNG, out / 'complex_analysis_visual_thumbnail.png')
"""
results = []


def check(name, ok, detail):
    results.append(bool(ok))
    print(f"{'PASS' if ok else 'FAIL'} {name}: {detail}", flush=True)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def load(path):
    spec = importlib.util.spec_from_file_location('thumbnail_under_check', path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def patched(path, tmp):
    H = load(path)
    P, T = H.P, H.T
    B = tmp / 'B'
    T.B = T.T.B = B
    T.BIN, T.GUARD, T.MANIFEST = tmp / 'fake', tmp / 'fake-guard', tmp / 'timeline-inputs.json'
    T.guard_policy = lambda: None
    P.S.TIMELINE_EXIT, P.S.EXIT, P.EXIT = tmp / 'timeline.exit', tmp / 'score.exit', tmp / 'preview.exit'
    P.MANIFEST = tmp / 'preview-inputs.json'
    H.MANIFEST, H.OUT = tmp / 'stage' / H.MANIFEST.name, B / H.JOB / 'output'
    return H


def fixture(tmp, png_size=(3840, 2160)):
    """A finished preview: two frozen files, with only one of them in the recovery manifest."""
    tmp.mkdir()
    H = load(BUNDLE / 'thumbnail_optimized02.py')
    T = H.T
    (tmp / 'stage').mkdir()
    for job in (H.PREVIEW, T.BUILD_JOB):
        (tmp / 'B' / job).mkdir(parents=True)
        (tmp / 'B' / job / 'resource-receipt.json').write_text(json.dumps({
            'admitted': True, 'exit': 0, 'finished': 'fixture', 'inputs_sha256': 'a', 'final_inputs_sha256': 'a'}))
    (tmp / 'B' / H.PREVIEW / 'preview-summary.json').write_text('{}')
    for name in ('timeline.exit', 'score.exit', 'preview.exit'):
        (tmp / name).write_text('0\n')
    for name in ('recovery-row.txt', 'preview-row.txt'):
        (tmp / name).write_text(name)
    rows = [{'path': str(tmp / name), 'sha256': sha(tmp / name)} for name in ('recovery-row.txt', 'preview-row.txt')]
    (tmp / 'timeline-inputs.json').write_text(json.dumps({'files': rows[:1]}))
    (tmp / 'preview-inputs.json').write_text(json.dumps({'files': rows}))
    Image.new('RGB', png_size).save(tmp / 'thumb.png')
    (tmp / 'fake-guard').write_text(GUARD.replace('TMP', repr(str(tmp))))
    (tmp / 'fake').write_text(BINARY.replace('PYTHON', sys.executable).replace('PNG', repr(str(tmp / 'thumb.png'))))
    (tmp / 'fake').chmod(0o755)


def listed(H):
    return {row['path'] for row in json.loads(H.MANIFEST.read_text())['files']}


def refused(action, words):
    try:
        action()
    except (AssertionError, RuntimeError) as error:
        return words in str(error) or words in repr(error)
    return False


def sealed_files():
    names = ('SHA256SUMS', 'thumbnail_optimized01.py', 'preview_optimized01.py', 'score_optimized01.py', 'start-optimized-thumbnail01.sh')
    return {name: sha(SEALED / name) for name in names}


def main():
    before = sealed_files()
    source = (BUNDLE / 'thumbnail_optimized02.py').read_text()
    body = next(node for node in ast.parse(source).body if isinstance(node, ast.FunctionDef) and node.name == 'wait_and_launch')
    names = {f'{node.value.id}.{node.attr}' for node in ast.walk(body)
             if isinstance(node, ast.Attribute) and isinstance(node.value, ast.Name)}
    shell = subprocess.run(['/bin/bash', '-n', str(BUNDLE / 'start-optimized-thumbnail02.sh')])
    check('a-parse', 'P.MANIFEST' in names and 'T.MANIFEST' not in names and shell.returncode == 0,
          'the helper and start script parse, and wait_and_launch reads P.MANIFEST, never T.MANIFEST')

    H = load(BUNDLE / 'thumbnail_optimized02.py')
    sums = (SEALED / 'SHA256SUMS').read_text()
    pinned = {path.name: digest for path, digest in H.IMPORTED_SHA256.items()}
    check('a-pins', sha(SEALED / 'SHA256SUMS') == SEALED_SUMS and len(pinned) == 4
          and all(f"{pinned[name]}  {name}\n" in sums for name in ('preview_optimized01.py', 'score_optimized01.py'))
          and H.P.S.RECOVERY_SHA256 == pinned['optimized_timeline_recovery01.py'],
          "the four helpers load under their pins, and those match the 01 seal and the score helper's own pin")
    check('a-names', H.JOB == 'complex-stage06-optimized-thumbnail02' and H.MANIFEST.parent.name == 'optimized-thumbnail02'
          and not (H.T.B / H.JOB).exists() and not H.MANIFEST.parent.exists()
          and not (H.T.B / 'complex-stage06-optimized-thumbnail01').exists(),
          'job, manifest and stage folder are new and unused, and no 01 thumbnail job exists')

    with tempfile.TemporaryDirectory() as folder:
        root = Path(folder)
        old_case, new_case = root / 'old', root / 'new'
        fixture(old_case)
        fixture(new_case)
        old = patched(SEALED / 'thumbnail_optimized01.py', old_case)
        old.wait_and_launch()
        new = patched(BUNDLE / 'thumbnail_optimized02.py', new_case)
        new.wait_and_launch()
        imports = {str(path) for path in new.IMPORTED_SHA256}
        check('f-01-defect', not {str(SEALED / 'preview_optimized01.py'), str(SEALED / 'score_optimized01.py')} & listed(old)
              and str(old_case / 'preview-row.txt') not in listed(old),
              '01 on the same fixture binds neither the preview nor the score helper, nor the preview rows: the defect root found')
        needed = {*imports, str(new_case / 'preview-row.txt'), str(new_case / 'recovery-row.txt'), str(new_case / 'preview-inputs.json'),
                  str(BUNDLE / 'thumbnail_optimized02.py'), str(BUNDLE / 'start-optimized-thumbnail02.sh'),
                  str(new_case / 'B' / new.PREVIEW / 'preview-summary.json')}
        check('f-manifest', needed <= listed(new),
              'the 02 manifest holds every preview manifest row, that manifest, the four imports, the helper, its start script and the preview results')
        argv = json.loads((new_case / 'guard.json').read_text())
        shown = [Path(x).name if x.startswith('/') and x != sys.executable else x for x in argv]
        check('f-guard-argv', shown == ['--phase', 'native', '--job', new.JOB, '--inputs', 'optimized-thumbnail02-inputs01.json',
                                        '--', sys.executable, '-I', '-B', 'thumbnail_optimized02.py', '--render'],
              'the guard gets the same phase and the new job, manifest and child')

        def ready(name, size=(3840, 2160)):
            tmp = root / name
            fixture(tmp, size)
            H = patched(BUNDLE / 'thumbnail_optimized02.py', tmp)
            H.wait_and_launch()
            (tmp / 'B' / H.JOB / 'resource-receipt.json').write_text(json.dumps({
                'admitted': True, 'policy': H.T.POLICY,
                'command': [sys.executable, '-I', '-B', str(BUNDLE / 'thumbnail_optimized02.py'), '--render']}))
            return tmp, H

        tmp, H = ready('render')
        H.render()
        receipt = json.loads((H.OUT / 'thumbnail-receipt.json').read_text())
        check('f-render', receipt['visual_review'] == 'pending' and receipt['publication'] == 'not uploaded',
              'the fake binary output decodes at 3840x2160; review and publication stay open')
        tmp, H = ready('small', (64, 36))
        check('f-wrong-size', refused(H.render, 'AssertionError') and not (H.OUT / 'thumbnail-receipt.json').exists(),
              'a wrong-size thumbnail is refused and writes no receipt')
        tmp, H = ready('moved')
        (tmp / 'preview-row.txt').write_text('changed')
        check('f-row-changed-at-render', refused(H.render, 'preview-row.txt'), 'a preview manifest row that changes before the child runs is refused')

        tmp = root / 'changed'
        fixture(tmp)
        (tmp / 'preview-row.txt').write_text('changed')
        H = patched(BUNDLE / 'thumbnail_optimized02.py', tmp)
        check('f-row-changed-at-launch', refused(H.wait_and_launch, 'preview-row.txt') and not H.MANIFEST.exists() and not (tmp / 'B' / H.JOB).exists(),
              'a changed preview manifest row is refused before any manifest or job')
        tmp = root / 'failed'
        fixture(tmp)
        (tmp / 'preview.exit').write_text('1\n')
        H = patched(BUNDLE / 'thumbnail_optimized02.py', tmp)
        check('f-upstream-failed', refused(H.wait_and_launch, 'Upstream controller failed') and not H.MANIFEST.exists() and not (tmp / 'B' / H.JOB).exists(),
              'a preview exit of 1 launches nothing')

        tree = root / 'start'
        script = tree / 'a/b/c/d/e/start-optimized-thumbnail02.sh'
        script.parent.mkdir(parents=True)
        shutil.copy(BUNDLE / 'start-optimized-thumbnail02.sh', script)
        (script.parent / 'thumbnail_optimized02.py').write_text("print('ran')\nraise SystemExit(3)\n")
        first = subprocess.run(['/bin/bash', str(script)])
        work = tree / 'narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/optimized-thumbnail02'
        log, code = work / 'optimized-thumbnail02-controller01.log', work / 'optimized-thumbnail02-controller01.exit'
        second = subprocess.run(['/bin/bash', str(script)], capture_output=True, text=True)
        check('s-start', first.returncode == 3 and code.read_text() == '3\n' and second.returncode == 1 and log.read_text() == 'ran\n',
              'the script records the exit code under the new names, and a second start is refused')

    real = load(BUNDLE / 'thumbnail_optimized02.py')
    check('p-sealed', sealed_files() == before and sha(SEALED / 'SHA256SUMS') == SEALED_SUMS
          and not (real.T.B / real.JOB).exists() and not real.MANIFEST.parent.exists(),
          'the 01 seal and its helpers are byte-identical, and the real 02 names are still unused')
    print(f"RESULT {'PASS' if all(results) else 'FAIL'}, {results.count(False)} failed")
    return 0 if all(results) else 1


if __name__ == '__main__':
    sys.exit(main())
