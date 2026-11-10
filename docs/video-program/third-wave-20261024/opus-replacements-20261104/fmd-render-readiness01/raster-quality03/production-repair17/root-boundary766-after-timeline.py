"""Render repaired09 preview only after stable765 and exact full timing tables."""
import hashlib
import json
import os
import subprocess
import sys
import time
from pathlib import Path

R = Path.home()/'scratch/jmev-complex-native-20261109'
Q = R/'qualification'
PRIOR = R/'jobs/mini-boundary-timeline765'
DELIVERY = R/'jobs/mini-boundary-delivery764'
JOB = R/'jobs/mini-boundary-preview766'
INPUTS = Q/'root-boundary766-inputs.json'
REF = R/'reference558'

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def read(p):
    return json.loads(p.read_text())

def success(p):
    d = read(p)
    assert d.get('admitted') and d.get('finished') and d.get('exit') == 0, p
    assert d['inputs_sha256'] == d['final_inputs_sha256'], p
    assert not any(d.get(k) for k in ('resource_stop', 'stopped_for_resource_limit', 'error')), p
    return d

def tables(folder):
    old = b'/data/projects/jmev-stroke-probe-20261106/jobs/csd-review-repair-rebind313/narration'
    new = str(DELIVERY/'narration').encode()
    result = {}
    for ext in ('timeline.tsv', 'cues.tsv', 'cuelog.tsv'):
        name = '09_boundary.'+ext
        assert (REF/name).read_bytes().replace(old, new) == (folder/name).read_bytes(), name
        result[name] = sha(folder/name)
    header = dict(line.split('\t', 1) for line in (folder/'09_boundary.timeline.tsv').read_text().splitlines()
                  if line.startswith(('chapter\t', 'fps\t', 'frames\t')))
    assert header == dict(chapter='09_boundary', fps='60', frames='16504')
    return result

def main():
    assert not JOB.exists() and not INPUTS.exists()
    deadline = time.monotonic()+86400
    while time.monotonic() < deadline:
        try:
            d = read(PRIOR/'resource-receipt.json')
        except (FileNotFoundError, json.JSONDecodeError):
            d = {}
        if 'finished' in d:
            success(PRIOR/'resource-receipt.json')
            if (PRIOR/'comparison765.json').exists():
                break
        time.sleep(15)
    else:
        raise SystemExit('765 still pending; no preview launched')
    source = Q/'root-boundary765-inputs.json'
    assert sha(source) == d['inputs_sha256']
    rows = {x['path']: x['sha256'] for x in read(source)['files']}
    for name, digest in rows.items():
        assert sha(Path(name)) == digest, name
    tables(PRIOR/'timeline-output')
    assert read(PRIOR/'comparison765.json')['frames'] == 16504
    success(DELIVERY/'resource-receipt.json')
    binary = R/'jobs/native-boundary761/complex_analysis_visual-761'
    assert sha(binary) == '71736c26d9953f08af8f4ce4f6fc0f13aeb98e7135b3dd415fb3ebf00e161c26'
    for p in [source, binary, Path(__file__).resolve(), Path(sys.executable).resolve()]:
        rows[str(p)] = sha(p)
    for p in PRIOR.rglob('*'):
        if p.is_file():
            rows[str(p)] = sha(p)
    with INPUTS.open('x') as f:
        json.dump(dict(files=[dict(path=k, sha256=v) for k, v in sorted(rows.items())]), f, indent=2)
    out = JOB/'preview'
    command = [sys.executable, '-I', '-B', str(Q/'cc2-mini-max-guard508.py'), '--phase', 'native',
               '--job', JOB.name, '--inputs', str(INPUTS), '--', str(binary), 'render', '09_boundary',
               '--res', '960x540', '--fps', '60', '--threads', '4', '--narration', str(DELIVERY/'narration'),
               '--out', str(out), '--lint', '--crf', '18', '--preset', 'veryfast']
    env = dict(os.environ, PATH=str(Path.home()/'.local/bin')+':/opt/homebrew/bin:/usr/bin:/bin')
    print('766 launching guarded native09 preview, bindings', len(rows), flush=True)
    subprocess.run(command, env=env, check=True)
    success(JOB/'resource-receipt.json')
    result = tables(out)
    with (JOB/'comparison766.json').open('x') as f:
        json.dump(dict(frames=16504, tables=result,
                       layout_sha256=sha(out/'09_boundary.layout.tsv'),
                       scope='Native preview schedules only. Actual pixels, motion, listening, fine sync and independent final AV remain open.'), f, indent=2)
    print('766 preview completed with exact full tables; pictures not yet reviewed', flush=True)

if __name__ == '__main__':
    main()
