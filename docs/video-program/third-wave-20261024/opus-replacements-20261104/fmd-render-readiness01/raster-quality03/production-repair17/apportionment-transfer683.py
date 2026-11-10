"""Exact apportionment recovery transport. Sender reads; receiver writes only fresh numbered files inside its guard job.

Reuses the frozen371 stream stager. No alias, source path, existing destination,
native table, audio sample or master configuration is changed. Import is not release approval.
"""
import argparse
import hashlib
import importlib.util
import json
import re
import sys
from pathlib import Path

MAC = Path('/Users/jemanuel/scratch/jmev-complex-native-20261109')
D = Path('/data/projects/jmev-stroke-probe-20261106')
Q = D/'qualification01/production-repair17'
LIB_SHA = 'ba337aca143316748561c4300d5774386387690b49052e72d29a04b103dd244f'
EXPORT_SHA = '95c6ed1fb4fcc989dbf453ab13b2542c6c06c0e8b7d450d7c56ca91cc54b6d02'
MAGIC = b'APPORTIONMENT-IMPORT683 '
POLICY = dict(admission_gib=10, stop_free_gib=8, phase_growth_gib=4,
              work_cap_gib=56, target_growth_cap_gib=20)

def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def read(path):
    return json.loads(Path(path).read_text())

def validate(m):
    assert m['schema'] == 'apportionment-local-finals-export682-v1'
    assert {k:v['frames'] for k,v in m['chapters'].items()} == {'01_one_more_seat':4806, '02_shares':6647}
    rows=m['files']
    assert len(rows)==38 and [r['path'] for r in rows]==sorted(set(r['path'] for r in rows))
    for r in rows:
        assert set(r)=={'path','sha256','bytes'}
        assert Path(r['path']).is_absolute() and '..' not in Path(r['path']).parts
        assert re.fullmatch('[0-9a-f]{64}',r['sha256']) and type(r['bytes']) is int and r['bytes']>=0
    assert sum(r['bytes'] for r in rows)==m['total_bytes'] < 3*(1<<30)
    assert len(m['reuse_from_prior_imports'])==3967
    assert len({r['path'] for r in rows}|{r['path'] for r in m['reuse_from_prior_imports']})==4005
    return rows

def header(m, digest):
    return MAGIC+json.dumps(dict(manifest_sha256=digest, files=len(m['files']),
                                bytes=m['total_bytes']), sort_keys=True).encode()+b'\n'

def terminal(d):
    assert d.get('admitted') and d.get('finished') and d.get('exit') == 0
    assert not d.get('preflight_only')
    assert not any(d.get(k) for k in ('resource_stop', 'stopped_for_resource_limit', 'error'))
    assert d['inputs_sha256'] == d['final_inputs_sha256']

def closure(m, physical):
    for row in validate(m):
        p=physical[row['path']]
        assert p.stat().st_size==row['bytes'] and sha(p)==row['sha256'],row['path']

def send(args):
    assert sha(args.manifest)=='1c7c690b603ed0a3571ba27e033dc2c1af581cacf74b171e01996e702e2285c8'
    m = read(args.manifest)
    rows = validate(m)
    physical = {r['path']: Path(r['path']) for r in rows}
    closure(m, physical)
    out = sys.stdout.buffer
    out.write(header(m, sha(args.manifest)))
    for row in rows:
        h, count = hashlib.sha256(), 0
        with physical[row['path']].open('rb') as f:
            while block := f.read(1 << 20):
                count += len(block)
                assert count <= row['bytes'], ('Source grew during send', row['path'])
                h.update(block)
                out.write(block)
        assert count == row['bytes'] and h.hexdigest() == row['sha256'], row['path']
    out.flush()

def receive(args):
    assert re.fullmatch('csd-apportionment-import683-a[1-9][0-9]*', args.job)
    job = D/'jobs'/args.job
    assert sorted(p.name for p in job.iterdir()) == ['command.log', 'resource-receipt.json']
    state = read(job/'resource-receipt.json')
    assert state.get('admitted') and state['phase'] == 'native' and not state.get('preflight_only')
    assert 'exit' not in state and 'finished' not in state
    assert {k: state['policy'][k] for k in POLICY} == POLICY
    helper = Path(__file__).resolve()
    manifest, inputs = Path(args.manifest), Path(args.inputs)
    assert helper.parent == manifest.parent == inputs.parent == Q
    expected = ['/usr/bin/timeout', '14400', '/usr/bin/python3', '-I', '-B', str(helper),
                'receive', '--manifest', str(manifest), '--job', args.job, '--inputs', str(inputs)]
    assert state['command'] == expected, 'Guard admitted another command'
    assert sha(inputs) == state['inputs_sha256']
    entries = read(inputs)['files']
    bound = {r['path']: r['sha256'] for r in entries}
    assert len(bound) == len(entries)
    library = Q/'csd-css352-import371.py'
    assert sha(library) == LIB_SHA
    for p in [helper, manifest, library, D/'guard_csd02.py', Path('/usr/bin/python3').resolve()]:
        assert bound[str(p)] == sha(p), p
    for name, digest in bound.items():
        assert sha(name) == digest, name
    assert sha(manifest)=='1c7c690b603ed0a3571ba27e033dc2c1af581cacf74b171e01996e702e2285c8'
    m = read(manifest)
    reused = {}
    for number in ('652', '655'):
        prior = D/('jobs/csd-apportionment-import'+number+'-a1')
        for p in (prior/'resource-receipt.json', prior/'import-result.json'):
            assert bound.get(str(p))==sha(p), p
        terminal(read(prior/'resource-receipt.json'))
        previous = read(prior/'import-result.json')
        export = next(x for x in m['prior_exports'] if x['path'].endswith('export'+number+'.json'))
        assert previous['manifest_sha256']==export['sha256']
        for item in previous['files']:
            if item['path'] in reused:
                assert reused[item['path']]['sha256']==item['sha256']
            reused[item['path']]=item
    reuse_records=[]
    for item in m['reuse_from_prior_imports']:
        old=reused[item['path']];p=Path(old['physical'])
        assert p.is_relative_to(D/'jobs') and not p.is_symlink()
        assert old['sha256']==item['sha256']==sha(p)
        assert bound.get(str(p))==item['sha256']
        reuse_records.append(dict(item,physical=str(p)))
    rows = validate(m)
    assert sys.stdin.buffer.readline(4097) == header(m, sha(manifest)), 'Wrong stream header'
    spec = importlib.util.spec_from_file_location('stager371', library)
    stager = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(stager)
    incoming = job/'incoming'
    incoming.mkdir(mode=0o700)
    # All destinations stay numbered inside the new job. No source path is used
    # as a destination, and no existing source/destination file is replaced.
    plan = [(row, incoming/('%05d' % n), True) for n, row in enumerate(rows)]
    physical = stager.stage(sys.stdin.buffer, plan, incoming)
    closure(m, physical)
    result = dict(chapter='apportionment-completed-local-finals', manifest=str(manifest), manifest_sha256=sha(manifest),
                  files=[dict(row, physical=str(physical[row['path']])) for row in rows],
                  reused_files=reuse_records,
                  scope='Exact transport only. Raw files preserved; no master, picture viewing, listening or final AV acceptance.')
    with (job/'import-result.json').open('x') as f:
        json.dump(result, f, indent=2)
    print('683 imported and rehashed', len(rows), 'files; reused', len(reuse_records))

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('mode', choices=['send', 'receive'])
    parser.add_argument('--manifest', required=True)
    parser.add_argument('--job')
    parser.add_argument('--inputs')
    args = parser.parse_args()
    (send if args.mode == 'send' else receive)(args)
