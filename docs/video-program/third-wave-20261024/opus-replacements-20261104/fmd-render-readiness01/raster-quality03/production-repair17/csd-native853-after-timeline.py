"""Render Linux844 repaired09 preview only after stable852 and exact full timing tables."""
import hashlib
import json
import os
import subprocess
import sys
import time
from pathlib import Path

R = Path('/data/projects/jmev-stroke-probe-20261106')
Q = R/'qualification01/production-repair17'
PRIOR = R/'jobs/csd-native-timeline852'
DELIVERY = R/'jobs/csd-native-delivery849'
JOB = R/'jobs/csd-native-preview853'
INPUTS = Q/'csd-native853-inputs.json'
REF = R/'jobs/csd-review-repair-timeline313/timeline-output'

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
            if (PRIOR/'comparison852.json').exists():
                break
        time.sleep(15)
    else:
        raise SystemExit('852 still pending; no preview launched')
    source = Q/'csd-native852-inputs.json'
    assert sha(source) == d['inputs_sha256']
    rows = {x['path']: x['sha256'] for x in read(source)['files']}
    for name, digest in rows.items():
        assert sha(Path(name)) == digest, name
    tables(PRIOR/'timeline-output')
    assert read(PRIOR/'comparison852.json')['frames'] == 16504
    success(DELIVERY/'resource-receipt.json')
    binary = R/'jobs/csd-native-build844/complex_analysis_visual-844'
    assert sha(binary) == read(R/'jobs/csd-native-build844/qualification.json')['binary_sha256']
    for p in [source, binary, Path(__file__).resolve(), Path(sys.executable).resolve()]:
        rows[str(p)] = sha(p)
    for p in PRIOR.rglob('*'):
        if p.is_file():
            rows[str(p)] = sha(p)
    with INPUTS.open('x') as f:
        json.dump(dict(files=[dict(path=k, sha256=v) for k, v in sorted(rows.items())]), f, indent=2)
    out = JOB/'preview'
    command = [sys.executable, '-I', '-B', str(R/'guard_csd02.py'), '--phase', 'native',
               '--job', JOB.name, '--inputs', str(INPUTS), '--', sys.executable,'-I','-B',
               str(Q/'csd-native852-run.py'),'preview']
    print('853 launching guarded native09 preview, bindings', len(rows), flush=True)
    subprocess.run(command, check=True)
    success(JOB/'resource-receipt.json')
    result = tables(out)
    with (JOB/'comparison853.json').open('x') as f:
        json.dump(dict(frames=16504, tables=result,
                       layout_sha256=sha(out/'09_boundary.layout.tsv'),
                       scope='Native preview schedules only. Actual pixels, motion, listening, fine sync and independent final AV remain open.'), f, indent=2)
    print('853 preview completed with exact full tables; pictures not yet reviewed', flush=True)

if __name__ == '__main__':
    main()
