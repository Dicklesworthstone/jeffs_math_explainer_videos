"""Wait for repaired schedules, then qualify stopping and render one guarded preview."""
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path('/data/projects/jmev-stroke-probe-20261106')
HERE = ROOT/'qualification01'

def read(p): return json.loads(p.read_text())
def sha(p):
    with p.open('rb') as f: return hashlib.file_digest(f, 'sha256').hexdigest()
def success(name):
    r = read(ROOT/'jobs'/name/'resource-receipt.json')
    assert r.get('finished') and r['exit'] == 0 and r['inputs_sha256'] == r['final_inputs_sha256']
    assert not r.get('resource_stop') and not r.get('stopped_for_resource_limit') and not r.get('error')
    return r
def freeze(name, manifests, extras):
    result = {}
    for manifest in manifests:
        for row in read(manifest)['files']:
            path = Path(row['path'])
            assert sha(path) == row['sha256'], str(path)
            if str(path) in result: assert result[str(path)] == row['sha256']
            result[str(path)] = row['sha256']
    for path in extras + manifests + [Path(__file__).resolve()]:
        digest = sha(path)
        if str(path) in result: assert result[str(path)] == digest
        result[str(path)] = digest
    out = ROOT/(name+'-inputs.json')
    with out.open('x') as f: json.dump({'files': [{'path': p, 'sha256': d} for p,d in sorted(result.items())]}, f, indent=2)
    return out
def launch(name, helper, manifests, extras, timeout):
    assert not (ROOT/'jobs'/name).exists(), 'Never duplicate a job'
    frozen = freeze(name, manifests, extras + [helper, ROOT/'guard_csd02.py'])
    subprocess.run(['/usr/bin/python3', '-I', '-B', str(ROOT/'guard_csd02.py'), '--phase', 'native',
                    '--job', name, '--inputs', str(frozen), '--', '/usr/bin/timeout', str(timeout),
                    '/usr/bin/python3', '-I', '-B', str(helper), '--run'], check=True, cwd=ROOT)
    success(name)
    return frozen

assert sys.argv[1:] == ['--run']
assert not (ROOT/'jobs/stage08-picture-preview01').exists()
timeline = ROOT/'jobs/stage08-picture-timeline01'
deadline = time.monotonic() + 43200
while True:
    receipt = read(timeline/'resource-receipt.json') if (timeline/'resource-receipt.json').exists() else {}
    if receipt.get('finished') or 'exit' in receipt:
        success('stage08-picture-timeline01')
        break
    assert time.monotonic() < deadline, 'Waiting deadline; no job restarted'
    time.sleep(20)
comparison = read(timeline/'comparison.json')
assert comparison['table_count'] == len(comparison['tables']) == 45
assert all(row['full_rows_equal_after_exact_narration_directory_substitution'] for row in comparison['tables'])
extra = [timeline/'resource-receipt.json', timeline/'comparison.json', HERE/'picture-preview01.py', HERE/'picture-encoder-stop01.py']
extra += list((timeline/'timeline-output').glob('*.tsv'))
stop_manifest = launch('stage08-picture-encoder-stop01', HERE/'picture-encoder-stop01.py',
                       [ROOT/'stage08-picture-timeline01-inputs.json', ROOT/'stage07-preview01-inputs.json'], extra, 900)
spec = importlib.util.spec_from_file_location('picture_preview', HERE/'picture-preview01.py')
preview = importlib.util.module_from_spec(spec)
spec.loader.exec_module(preview)
preview.check_score()
stop = ROOT/'jobs/stage08-picture-encoder-stop01'
extra += [stop/'resource-receipt.json', stop/'stop-summary.json', stop/'preview-result.json', stop/'final-result.json']
launch('stage08-picture-preview01', HERE/'picture-preview01.py', [stop_manifest], extra, 172800)
print('Repaired preview technical checks complete. Actual pictures/native4K/listening/fine-sync/independent AV remain open.')
