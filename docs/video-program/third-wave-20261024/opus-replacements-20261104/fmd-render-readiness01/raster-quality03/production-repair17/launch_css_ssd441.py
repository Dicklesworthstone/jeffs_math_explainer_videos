"""Qualify exact retained gallery inputs, then one new guarded SSD attempt."""
import argparse
import hashlib
import importlib.util
import json
import subprocess
from pathlib import Path

OLD = Path('/data/projects/jmev-css-render-worker-20261107')
NEW = Path('/data/ssd/work/jmev-gallery-recovery440')
JOB = 'css-reviewed-final-05-gallery352-ssd440'
WORKER_SHA = '9c80ceb68605aa272dfdb67dc72b40c5cce5b71f141122994f16bb891fdae519'

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--launch', action='store_true')
    args = parser.parse_args()
    helper = NEW/'css-gallery-ssd440.py'
    assert sha(helper) == WORKER_SHA
    failed = OLD/'jobs/css-reviewed-final-05-gallery352/resource-receipt.json'
    receipt = json.loads(failed.read_text())
    assert receipt.get('finished') and receipt.get('exit') == -15
    assert receipt.get('stopped_for_resource_limit') is True
    old_inputs = OLD/'qualification01/final-05_gallery-inputs352.json'
    assert sha(old_inputs) == receipt['inputs_sha256'] == receipt['final_inputs_sha256']
    rows = json.loads(old_inputs.read_text())['files']
    for row in rows:
        assert sha(row['path']) == row['sha256'], row['path']
    assert not (NEW/'jobs'/JOB).exists(), 'Never duplicate a retry'
    spec = importlib.util.spec_from_file_location('gallery_ssd440', helper)
    worker = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(worker)
    permit = OLD/'qualification01/repair352-permits/05_gallery-css-permit375.json'
    review, _, binding, carried = worker.check('05_gallery', permit)
    files = {Path(row['path']) for row in rows}
    files.update(worker.required(permit, review, binding, carried))
    files.update([helper, Path(__file__).resolve(), NEW/'guard_css_ssd441.py', failed, old_inputs,
                  OLD/'jobs/allocation-baseline.json', NEW/'jobs/allocation-baseline.json'])
    result = dict(frames=review['frames'], frozen_files=len(files), original_inputs_sha256=sha(old_inputs),
                  worker_sha256=sha(helper), guard_sha256=sha(NEW/'guard_css_ssd441.py'),
                  launch_requested=args.launch, release_accepted=False)
    if not args.launch:
        print(json.dumps(result))
        return 0
    inputs = NEW/'gallery-inputs441.json'
    with inputs.open('x') as f:
        json.dump({'files':[dict(path=str(p), sha256=sha(p)) for p in sorted(files)]}, f, indent=2)
    command = ['/usr/bin/python3','-I','-B',str(NEW/'guard_css_ssd441.py'),
               '--phase','native','--job',JOB,'--inputs',str(inputs),'--',
               '/usr/bin/python3','-I','-B',str(helper),'render','--chapter','05_gallery',
               '--job',JOB,'--inputs',str(inputs),'--permit',str(permit),'--threads','16']
    return subprocess.run(command, check=False).returncode

if __name__ == '__main__':
    raise SystemExit(main())
