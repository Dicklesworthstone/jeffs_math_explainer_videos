"""Score the full Frederick film only after the optimized recovery timeline succeeds.

Root's score-after-timeline01.py with new names and tighter gates; diff the two to audit it.
No old job, helper or exit file is read. The failed debug run is never a baseline.

    no argument  wait for this recovery's timeline, then run the unchanged guard
    --check      read-only: frozen inputs, guard policy, unused names; launches nothing
    --run        the guard's child; never call it by hand
"""
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import time

O = Path(__file__).resolve().parent
RECOVERY = O.parent / 'complex-optimized-recovery-review01/optimized_timeline_recovery01.py'
RECOVERY_SHA256 = '4560e0dcd5534e0b8b3eaf205622f7cc36657c109344107d976c7b852dc086d7'
SPEC = importlib.util.spec_from_file_location('complex_timeline', RECOVERY)
T = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(T)
JOB = 'complex-stage06-optimized-score01'
STAGE = T.E / 'optimized-downstream01'  # these controllers' logs, exit files and manifests
TIMELINE_EXIT = T.STAGE / 'controller01.exit'
EXIT = STAGE / 'optimized-score-controller01.exit'
MANIFEST = STAGE / 'optimized-score-inputs01.json'
OUT = T.B / JOB / 'scored'
WAIT_POLLS = 8640  # 48 h at 20 s: root's old 12 h wait is shorter than the timeline's own limit


def read(path):
    return json.loads(path.read_text())


def sha(path):
    with path.open('rb') as handle:
        return hashlib.file_digest(handle, 'sha256').hexdigest()


def check_timeline():
    assert sha(RECOVERY) == RECOVERY_SHA256, 'Sealed recovery helper changed'
    assert T.JOB != T.T.JOB, 'The failed run is never a baseline'
    assert T.success(T.JOB), 'Timeline is not successfully complete'
    assert read(T.B / T.JOB / 'resource-receipt.json')['policy'] == T.POLICY, 'Guard policy changed'
    summary = read(T.B / T.JOB / 'timeline-summary.json')
    assert summary['binary_sha256'] == T.BIN_SHA256 == sha(T.BIN) and summary['profile'] == 'release', 'Timeline binary differs'
    assert summary['failed_job'] == T.T.JOB, 'Timeline names another failed run'
    assert summary['full_baseline_equivalence_established'] is False, 'Full baseline equivalence cannot be claimed'
    assert summary['prior_chapters_compared'] == list(T.PRIOR_CHAPTERS)
    assert summary['chapters_without_prior_table'] == list(T.CHAPTERS[len(T.PRIOR_CHAPTERS):])
    assert [row['chapter'] for row in summary['chapters']] == list(T.CHAPTERS)
    assert summary['cues'] == 105 and summary['fps'] == 60
    assert summary['native_oracle_fact_rows_matched'] > 0
    folder = T.B / T.JOB / 'timeline-output'
    assert {p.stem.removesuffix('.timeline') for p in folder.glob('*.timeline.tsv')} == set(T.CHAPTERS)
    assert all((folder / f'{chapter}.{table}.tsv').is_file() for chapter in T.CHAPTERS for table in T.TABLES)
    compared = read(T.B / T.JOB / 'prior-table-comparison.json')
    assert compared['prior_job'] == T.T.JOB and compared['new_binary_sha256'] == T.BIN_SHA256
    assert len(compared['tables']) == summary['prior_tables_compared'] == 12
    for row in compared['tables']:
        assert row['normalized_equal'] is True, 'Prior table differs: ' + row['table']
        assert sha(folder / row['table']) == row['new_sha256'], 'New table changed: ' + row['table']
        assert sha(T.PRIOR_OUT / row['table']) == row['prior_sha256'], 'Prior table changed: ' + row['table']
    ordered = [line.split('\t', 1)[0] for line in (T.C / 'script.tsv').read_text().splitlines()
               if line.strip() and not line.startswith('#')]
    seen = []
    total = 0
    for item in summary['chapters']:
        path = folder / (item['chapter'] + '.timeline.tsv')
        assert sha(path) == item['timeline_sha256']
        rows = [line.split('\t') for line in path.read_text().splitlines()]
        def value(key):
            values = [row[1] for row in rows if row[0] == key]
            assert len(values) == 1
            return values[0]
        assert value('chapter') == item['chapter'] and value('fps') == '60'
        frames = int(value('frames'))
        assert frames == item['frames'] and frames > 0
        cues = [row for row in rows if row[0] == 'line']
        assert [row[1] for row in cues] == item['cues']
        starts = [int(row[2]) for row in cues]
        assert starts == sorted(starts) and all(0 <= n < frames for n in starts)
        assert all(Path(row[4]).resolve() == (T.N / (row[1] + '.wav')).resolve() for row in cues)
        seen.extend(row[1] for row in cues)
        total += frames
    assert seen == ordered and len(seen) == len(set(seen)) == 105
    assert total == summary['total_frames']
    return folder


def run_score():
    receipt = read(T.B / JOB / 'resource-receipt.json')
    assert receipt.get('admitted') is True and 'exit' not in receipt
    assert receipt['command'] == [sys.executable, '-I', '-B', str(Path(__file__).resolve()), '--run']
    assert receipt['policy'] == T.POLICY, 'Guard policy changed'
    for row in read(MANIFEST)['files']:
        assert sha(Path(row['path'])) == row['sha256'], row['path']
    folder = check_timeline()
    assert not OUT.exists(), 'Preserve existing score outputs'
    subprocess.run([str(T.BIN), 'score', '--narration', str(T.N), '--timelines',
                    str(folder), '--out', str(OUT)], cwd=T.R, check=True, timeout=T.RENDER_TIMEOUT_SECONDS)
    qa_folder = OUT / 'stems/qa'
    assert {p.stem for p in qa_folder.glob('*.json')} == set(T.CHAPTERS)
    qa = {}
    for chapter in T.CHAPTERS:
        path = qa_folder / (chapter + '.json')
        report = read(path)
        assert report['chapter'] == chapter and report['passed'] is True, chapter
        assert report['metrics'] and all(m.get('pass') is True or m.get('pass') is None
                                         for m in report['metrics']), chapter
        qa[chapter] = sha(path)
    with (T.B / JOB / 'score-summary.json').open('x') as handle:
        json.dump({'qa_sha256': qa, 'chapters': 15, 'cues': 105,
                   'scope': 'Native score QA only. Listening, actual pictures, fine sync and independent final AV review remain open.'}, handle, indent=2)
    print('SCORE PASS: all 15 registry chapter QA reports passed.', flush=True)


def wait_for(*exits):
    """Wait only on this recovery's controllers. Each writes its exit file after its job's receipt
    is final, so no receipt is read while the guard is rewriting it."""
    for _ in range(WAIT_POLLS):
        codes = [path.read_text().strip() if path.exists() else '' for path in exits]
        failed = [path.name for path, code in zip(exits, codes) if code not in ('', '0')]
        if failed:
            raise RuntimeError(f'Upstream controller failed; nothing launched: {failed}')
        if all(code == '0' for code in codes):
            return
        time.sleep(20)
    raise RuntimeError('Wait expired; nothing launched')


def wait_and_launch():
    wait_for(TIMELINE_EXIT)
    check_timeline()
    assert not (T.B / JOB).exists() and not MANIFEST.exists(), 'Duplicate score refused'
    files = {}
    for row in read(T.MANIFEST)['files']:
        assert sha(Path(row['path'])) == row['sha256'], row['path']
        files[row['path']] = row['sha256']
    for path in [T.MANIFEST, Path(__file__).resolve(), O / 'start-optimized-score01.sh'] + [
            p for p in (T.B / T.JOB).rglob('*') if p.is_file()]:
        files[str(path)] = sha(path)
    with MANIFEST.open('x') as handle:
        json.dump({'files': [{'path': path, 'sha256': digest} for path, digest in sorted(files.items())]}, handle, indent=2)
    T.guard_policy()
    guard = T.GUARD
    subprocess.run([sys.executable, '-I', '-B', str(guard), '--phase', 'native', '--job', JOB,
                    '--inputs', str(MANIFEST), '--', sys.executable, '-I', '-B',
                    str(Path(__file__).resolve()), '--run'], cwd=T.R, check=True)
    assert T.success(JOB)


def preflight():
    """Read-only, and valid while the timeline runs."""
    assert sha(RECOVERY) == RECOVERY_SHA256, 'Sealed recovery helper changed'
    report = {'frozen_files': len(T.check()), 'resources': T.guard_policy()}
    assert not (T.B / JOB).exists() and not MANIFEST.exists(), 'New names already used'
    return report


if __name__ == '__main__':
    assert sys.argv[1:] in ([], ['--run'], ['--check'])
    if sys.argv[1:] == ['--check']:
        print(json.dumps({'job': JOB, **preflight(), 'launched': False}))
    else:
        run_score() if sys.argv[1:] else wait_and_launch()
