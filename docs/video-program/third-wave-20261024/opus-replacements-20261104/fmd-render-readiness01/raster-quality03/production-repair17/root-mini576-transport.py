"""Exact native10 transport. Sender reads; receiver writes only fresh numbered files inside its guard job.

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
EXPORT_SHA = 'f918f54791e7eacf3a0c16e71bd80c705099b2e9588bf58d1f3689462cb73ce8'
MAGIC = b'NATIVE-MAC-IMPORT576 '
POLICY = dict(admission_gib=10, stop_free_gib=8, phase_growth_gib=4,
              work_cap_gib=56, target_growth_cap_gib=20)

def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def read(path):
    return json.loads(Path(path).read_text())

def validate(m):
    assert m['schema'] == 'native-mac-export575-v1' and m['chapter'] == '10_rigidity'
    assert m['source_root'] == str(MAC) and m['render_job'] == str(MAC/'jobs/mini-final555')
    assert m['verification_job'] == str(MAC/'jobs/mini-final-pictures560') and m['frames'] == 7442
    rows = m['files']
    assert 0 < len(rows) <= 30000
    paths = [r['path'] for r in rows]
    assert paths == sorted(set(paths)), 'Paths must be unique and sorted'
    for row in rows:
        assert set(row) == {'path', 'sha256', 'bytes'}
        assert Path(row['path']).is_absolute() and '..' not in Path(row['path']).parts
        assert not any(c in row['path'] for c in ('\n', '\r', '\t', '\x00'))
        assert re.fullmatch('[0-9a-f]{64}', row['sha256'])
        assert type(row['bytes']) is int and row['bytes'] >= 0
    assert sum(r['bytes'] for r in rows) == m['total_bytes'] < (4*(1 << 30) - (1 << 29))
    bound = {r['path']: r['sha256'] for r in rows}
    assert bound[str(MAC/'qualification/root-mini575-export.py')] == EXPORT_SHA
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
    rows = validate(m)
    bound = {r['path']: r for r in rows}
    for name, row in bound.items():
        p = physical[name]
        assert p.stat().st_size == row['bytes'] and sha(p) == row['sha256'], name
    for job, manifest, digest in [
        ('mini-final555', 'root-mini-final555-inputs.json', m['render_inputs_sha256']),
        ('mini-final-pictures560', 'mini-final-pictures560-inputs.json', m['verification_inputs_sha256'])]:
        state = read(physical[str(MAC/'jobs'/job/'resource-receipt.json')])
        terminal(state)
        name = str(MAC/'qualification'/manifest)
        assert bound[name]['sha256'] == digest == state['inputs_sha256']
        entries = read(physical[name])['files']
        assert len(entries) == len({x['path'] for x in entries})
        for entry in entries:
            assert bound[entry['path']]['sha256'] == entry['sha256'], entry['path']
    report = read(physical[str(MAC/'jobs/mini-final-pictures560/pictures560.json')])
    assert report['frames'] == 7442 and report['full_decode'] is True
    assert len(report['pictures']) == len(report['selected_frames']) == 113 and len(report['sheets']) == 15
    movie = str(MAC/'jobs/mini-final555/render/10_rigidity.mp4')
    assert report['movie'] == movie and report['movie_sha256'] == m['movie_sha256'] == bound[movie]['sha256']
    for row in report['pictures']+report['sheets']:
        assert bound[row['path']]['sha256'] == row['sha256']
    for ext, digest in report['tables'].items():
        assert bound[str(MAC/'jobs/mini-final555/render'/('10_rigidity.'+ext))]['sha256'] == digest

def send(args):
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
    assert re.fullmatch('csd-mini-import576-10-a[1-9][0-9]*', args.job)
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
    m = read(manifest)
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
    result = dict(chapter='10_rigidity', manifest=str(manifest), manifest_sha256=sha(manifest),
                  files=[dict(row, physical=str(physical[row['path']])) for row in rows],
                  scope='Exact transport only. Raw files preserved; no master, picture viewing, listening or final AV acceptance.')
    with (job/'import-result.json').open('x') as f:
        json.dump(result, f, indent=2)
    print('576 imported and rehashed', len(rows), 'files')

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('mode', choices=['send', 'receive'])
    parser.add_argument('--manifest', required=True)
    parser.add_argument('--job')
    parser.add_argument('--inputs')
    args = parser.parse_args()
    (send if args.mode == 'send' else receive)(args)
