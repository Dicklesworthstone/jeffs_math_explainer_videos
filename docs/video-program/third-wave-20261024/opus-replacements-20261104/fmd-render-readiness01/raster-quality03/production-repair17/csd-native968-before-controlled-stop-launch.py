"""Prepared manual recovery after864 timeout only; no waiting or automatic retry."""
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

D = Path('/data/projects/jmev-stroke-probe-20261106')
Q = D/'qualification01/production-repair17'
JOB = D/'jobs/csd-native-final968'
INPUTS = Q/'csd-native968-inputs.json'

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def read(p):
    return json.loads(p.read_text())

def success(p):
    d = read(p)
    assert d.get('admitted') and d.get('finished') and d.get('exit') == 0, p
    assert d['inputs_sha256'] == d['final_inputs_sha256'], p
    assert not any(d.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit')), p
    return d

def main():
    assert not JOB.exists() and not INPUTS.exists(), 'Never duplicate or overwrite a job'
    failed = D/'jobs/csd-native-final864'
    failed_receipt = read(failed/'resource-receipt.json')
    assert failed_receipt.get('finished') and failed_receipt.get('exit') not in (None, 0)
    assert not failed_receipt.get('resource_stop')
    assert failed_receipt['inputs_sha256'] == failed_receipt['final_inputs_sha256']
    cleanup = read(failed/'native.cleanup.json')
    assert cleanup['step'] == 'native' and 'TimeoutExpired' in cleanup['reason'], 'Only diagnosed timeout recovery'
    binary = str(D/'jobs/csd-native-build844/complex_analysis_visual-844')
    for proc in Path('/proc').iterdir():
        if not proc.name.isdigit():
            continue
        try:
            args = (proc/'cmdline').read_bytes().split(b'\0')
        except (FileNotFoundError, PermissionError, ProcessLookupError):
            continue
        assert not (args and args[0] == binary.encode() and str(failed/'render').encode() in args), '864 native process still alive'
    prior = D/'jobs/csd-native-pictures855'
    d = success(prior/'resource-receipt.json')
    source = Q/'csd-native855-inputs.json'
    assert sha(source) == d['inputs_sha256']
    files = read(source)['files']
    rows = {x['path']: x['sha256'] for x in files}
    assert len(rows) == len(files)
    for name, digest in rows.items():
        assert sha(name) == digest, name
    admission = Q/'csd-native864-picture-admission.json'
    review = read(admission)  # Must be authored from actual root and independent viewing.
    assert review['safe_to_advance_to_final4k'] is True
    assert review['all_selected_pictures_viewed'] is True
    assert review['concrete_defects'] == [] and review['independent_blockers'] == []
    for key in ('root_review', 'independent_review', 'extraction_record'):
        item = review[key]
        assert sha(item['path']) == item['sha256']
        rows[item['path']] = item['sha256']
    helper = Q/'csd-native853-after-timeline.py'
    assert sha(helper) == '8a3039d8dd2ad41ab09877069b10ea0eb7f4b6502af966723550a9c7be026dae'
    spec = importlib.util.spec_from_file_location('comparison864', helper)
    h = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(h)
    h.tables(D/'jobs/csd-native-preview853/preview')
    assert read(D/'jobs/csd-native-timeline852/comparison852.json')['frames'] == 16504
    assert read(D/'jobs/csd-native-preview853/comparison853.json')['frames'] == 16504
    score = D/'jobs/csd-production-score21'
    success(score/'resource-receipt.json')
    for root in (prior, score/'score/stems'):
        for p in root.rglob('*'):
            if p.is_file():
                rows[str(p)] = sha(p)
    for p in (source, admission, helper, Path(__file__).resolve(), Path(sys.executable).resolve(),
              Q/'csd-native968-run.py', score/'resource-receipt.json',
              failed/'resource-receipt.json', failed/'command.log', failed/'native.cleanup.json',
              Q/'csd-native864-inputs.json'):
        rows[str(p)] = sha(p)
    with INPUTS.open('x') as f:
        json.dump(dict(files=[dict(path=k, sha256=v) for k, v in sorted(rows.items())]), f, indent=2)
    subprocess.run([sys.executable, '-I', '-B', str(D/'guard_csd02.py'), '--phase', 'native',
                    '--job', JOB.name, '--inputs', str(INPUTS), '--', sys.executable,
                    '-I', '-B', str(Q/'csd-native968-run.py')], check=True)
    success(JOB/'resource-receipt.json')
    print('968 render terminated successfully; final extraction, actual review and master gates remain open')

if __name__ == '__main__':
    main()
