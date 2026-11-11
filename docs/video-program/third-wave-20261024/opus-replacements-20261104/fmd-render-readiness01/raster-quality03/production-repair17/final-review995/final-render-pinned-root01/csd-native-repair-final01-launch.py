"""PREPARED, NOT LAUNCHED: manual launch of one repaired chapter's native final 4K render; no waiting or retry.

csd-native968-launch.py's route for 01/08/09/11. Every run proof is checked here first, before the job
name or inputs file exists, so a refusal costs no job name. One final at a time: the runner takes the
first 16 CPUs of its affinity, so two finals would share cores.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

D = Path('/data/projects/jmev-stroke-probe-20261106')
Q = D/'qualification01/production-repair17'
RUNNER = Q/'csd-native-repair-final01-run.py'

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
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--chapter', required=True)
    chapter = ap.parse_args().chapter
    spec = importlib.util.spec_from_file_location('repair_final01_run', RUNNER)
    run = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(run)
    assert chapter in run.CHAPTERS, chapter
    JOB = run.job(chapter)
    INPUTS = Q/(JOB.name+'-inputs.json')
    assert not JOB.exists() and not INPUTS.exists(), 'Never duplicate or overwrite a job'
    pending = run.unresolved(chapter)
    assert not pending, chapter+': unresolved pin(s), not launched: '+', '.join(pending)
    chain = run.CHAINS[run.CHAPTERS[chapter]['kind']]
    binaries = {str(D/'jobs'/c['build']/c['binary']).encode() for c in run.CHAINS.values()}
    for proc in Path('/proc').iterdir():
        if not proc.name.isdigit():
            continue
        try:
            args = (proc/'cmdline').read_bytes().split(b'\0')
        except (FileNotFoundError, PermissionError, ProcessLookupError):
            continue
        assert not (args and args[0] in binaries and b'3840x2160' in args), 'a repaired final render is still alive'
        assert str(RUNNER).encode() not in args, 'a repaired final runner is still alive'
    binary, narration, files = run.proofs(chapter)
    rows = {}
    def add(p, digest=None):
        p, h = str(p), sha(p)
        assert digest is None or h == digest, p
        assert rows.get(p, h) == h, ('input drift', p)
        rows[p] = h
    for p in files:
        add(p)
    prior = D/'jobs'/chain['pictures']
    d = success(prior/'resource-receipt.json')
    source = Q/chain['pictures_inputs']
    assert sha(source) == d['inputs_sha256']
    entries = read(source)['files']
    assert len({x['path'] for x in entries}) == len(entries)
    for x in entries:
        add(x['path'], x['sha256'])
    score = D/'jobs/csd-production-score21'
    success(score/'resource-receipt.json')
    for root in (prior, score/'score/stems', narration):
        for p in root.rglob('*'):
            if p.is_file():
                add(p)
    for p in (source, Path(__file__).resolve(), Path(sys.executable).resolve(), RUNNER,
              Q/'csd-native844-base761.py', score/'resource-receipt.json'):
        add(p)
    add(Q/'csd-native844-base761.py', run.BASE761_SHA)
    with INPUTS.open('x') as f:
        json.dump(dict(files=[dict(path=k, sha256=v) for k, v in sorted(rows.items())]), f, indent=2)
    subprocess.run([sys.executable, '-I', '-B', str(D/'guard_csd02.py'), '--phase', 'native',
                    '--job', JOB.name, '--inputs', str(INPUTS), '--', sys.executable,
                    '-I', '-B', str(RUNNER), '--chapter', chapter], check=True)
    success(JOB/'resource-receipt.json')
    print(chapter+' final render terminated successfully; final extraction, actual review and master gates remain open')

if __name__ == '__main__':
    main()
