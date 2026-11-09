"""Freeze all qualified chapter outputs and run guarded assembly once.
Requires an explicit completed 15-chapter plan; does not create review permits.
"""
import argparse
import importlib.util
import json
from pathlib import Path
import subprocess
import time

R = Path('/data/projects/jmev-render-worker-20261107')
D = Path('/data/projects/jmev-stroke-probe-20261106')
Q = R / 'qualification01/production19'

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--plan', required=True, type=Path)
    args = parser.parse_args()
    helper = Q / 'trj-final-master54.py'
    worker = Q / 'trj-final-chapter19-02.py'
    spec = importlib.util.spec_from_file_location('chapter_worker', worker)
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    plan_hash = m.sha(args.plan)
    plan = m.read(args.plan)
    qualification = D / 'jobs/csd-production-repair-build19/qualification.json'
    chapters = m.read(qualification)['chapters']
    assert len(chapters) == len(set(chapters)) == 15
    assert [row['chapter'] for row in plan['chapters']] == chapters
    assert len({row['job'] for row in plan['chapters']}) == 15
    workers = {}
    for row in plan['chapters']:
        assert Path(row['job']).name == row['job']
        wp = Path(row.get('worker', str(worker)))
        spec = importlib.util.spec_from_file_location('worker_' + row['chapter'], wp)
        cm = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(cm)
        prefixes = {
            R: 'trj-',
            Path('/data/projects/jmev-hz2-render-worker-20261107'): 'hz2-',
            Path('/data/projects/jmev-css-render-worker-20261107'): 'css-',
            Path('/data/projects/jmev-ts1-render-worker-20261107'): 'ts1-',
            Path('/data/projects/jmev-ts2-render-worker-20261107'): 'ts2-',
            D: 'csd-',
        }
        assert cm.ROOT in prefixes
        prefix = prefixes[cm.ROOT]
        repaired09 = (
            prefix == 'hz2-' and row['chapter'] == '09_boundary'
            and wp == cm.ROOT / 'qualification01/production19/hz2-repaired-final158.py'
            and row['job'] == 'hz2-repaired-final-09-boundary158'
        )
        repaired08 = (
            prefix == 'csd-' and row['chapter'] == '08_loop_sums'
            and wp == D / 'qualification01/production-repair17/csd-repaired-final170.py'
            and row['job'] == 'csd-repaired-final-08-loop-sums170'
        )
        assert repaired08 or repaired09 or row['job'].startswith(prefix + 'production19-final-')
        cm.check(row['chapter'], Path(row['permit']))
        workers[row['chapter']] = (cm, wp, m.sha(wp))
        if cm.ROOT != R:
            m.success(Path(row['import_receipt']))
            imported = m.read(Path(row['import_receipt']))
            assert m.sha(row['import_inputs']) == imported['inputs_sha256']
    job = 'trj-production19-master01'
    assert not (R / 'jobs' / job).exists(), 'Assembly job already exists; do not duplicate'
    deadline = time.monotonic() + 24 * 3600
    while True:
        assert m.sha(args.plan) == plan_hash, 'Plan changed while waiting'
        done = True
        for row in plan['chapters']:
            cm, wp, wh = workers[row['chapter']]
            assert m.sha(wp) == wh, 'Worker changed while waiting'
            receipt = cm.ROOT / 'jobs' / row['job'] / 'resource-receipt.json'
            if not receipt.exists() or not m.read(receipt).get('finished'):
                done = False
                continue
            m.success(receipt)
        if done:
            break
        if time.monotonic() >= deadline:
            raise RuntimeError('Chapter waiting deadline; no assembly launched')
        time.sleep(30)
    files = {Path(__file__).resolve(), args.plan.resolve(), helper, worker,
             qualification, R / 'guard_trj01.py', m.BIN}
    for row in plan['chapters']:
        cm, wp, wh = workers[row['chapter']]
        assert m.sha(wp) == wh
        files.update((wp, cm.BIN))
        source_job = cm.ROOT / 'jobs' / row['job']
        if cm.ROOT != R:
            ip, im = Path(row['import_receipt']), Path(row['import_inputs'])
            imported = m.success(ip)
            assert m.sha(im) == imported['inputs_sha256']
            files.update((ip, im))
            for item in m.read(im)['files']:
                assert m.sha(item['path']) == item['sha256']
                files.add(Path(item['path']))
        receipt = source_job / 'resource-receipt.json'
        state = m.success(receipt)
        manifest = Path(row['inputs'])
        assert m.sha(manifest) == state['inputs_sha256']
        files.update((receipt, manifest, Path(row['permit'])))
        for item in m.read(manifest)['files']:
            assert m.sha(item['path']) == item['sha256'], item['path']
            files.add(Path(item['path']))
        review, _ = cm.check(row['chapter'], Path(row['permit']))
        cm.verify_tables(row['chapter'], source_job / 'render', review['frames'])
        summary_path = source_job / 'render-summary.json'
        summary = m.read(summary_path)
        assert (summary['chapter'], summary['frames']) == (row['chapter'], review['frames'])
        assert summary['binary_sha256'] == m.sha(cm.BIN)
        assert summary['permit_sha256'] == m.sha(row['permit'])
        files.add(summary_path)
        for item in summary['files']:
            assert m.sha(item['path']) == item['sha256']
            files.add(Path(item['path']))
    assert m.sha(args.plan) == plan_hash
    inputs = Q / 'master01-inputs.json'
    with inputs.open('x') as stream:
        json.dump({'files': [{'path': str(p), 'sha256': m.sha(p)}
                             for p in sorted(files)]}, stream, indent=2)
    subprocess.run(['python3', '-I', '-B', str(R / 'guard_trj01.py'),
                    '--phase', 'native', '--job', job, '--inputs', str(inputs), '--',
                    'python3', '-I', '-B', str(helper), '--job', job,
                    '--inputs', str(inputs), '--worker', str(worker),
                    '--plan', str(args.plan)], check=True)

if __name__ == '__main__':
    main()
