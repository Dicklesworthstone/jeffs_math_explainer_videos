"""Guarded native timing/lint comparison; no picture or audio acceptance."""
import hashlib
import importlib.util
import json
import signal
import subprocess
import sys
import time
from pathlib import Path

R = Path('/Users/jemanuel/projects/jeffs_math_explainer_videos')
P = Path(__file__).resolve().parent
Q = P.parent
B = R/'narration_lab/third-wave-20261024/root/five-film-release-20261027'
G = R/'narration_lab/third-wave-20261024/root/elevenlabs-apportionment-rebuild-20261104/frederick-generation89'
JOB = B/'apportionment-picture-timeline567'
REF = B/'apportionment-frederick-timeline444/timeline'
NARR = B/'apportionment-picture-assembly566/narration'
BINARY = B/'apportionment-picture-build564/apportionment-native564'

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def success(name):
    d = json.loads((B/name/'resource-receipt.json').read_text())
    assert d.get('finished') and d.get('exit') == 0
    assert not any(d.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit'))
    assert d['inputs_sha256'] == d['final_inputs_sha256']

def main():
    for name in ('apportionment-picture-build564', 'apportionment-picture-assembly566',
                 'apportionment-frederick-timeline444', 'apportionment-frederick-score445'):
        success(name)
    for row in json.loads((P/'timeline567-inputs.json').read_text())['files']:
        assert sha(row['path']) == row['sha256'], row['path']
    assert sha(BINARY) == '87a34bb262419c8a24b1c75b80d2eb97046cbc1f72fd5b2be11043ba5982a107'
    spec = importlib.util.spec_from_file_location('owned528', Q/'cc2-apportionment-preview-recovery528.py')
    owned = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(owned)
    owned.own_group()
    owned.STATE['job'] = JOB
    for sig in owned.STOP_SIGNALS:
        signal.signal(sig, owned.on_stop)
    receipt = json.loads((JOB/'resource-receipt.json').read_text())
    assert receipt.get('admitted') and receipt['policy'] == owned.POLICY and not receipt.get('finished')
    assert str(Path(__file__).resolve()) in receipt['command']
    assert sorted(x.name for x in JOB.iterdir()) == ['command.log', 'resource-receipt.json']
    subprocess.run(['rustfmt', '--check', '--edition', '2024',
                    'explainer_corpus/apportionment/src/chapters/signposts.rs',
                    'explainer_corpus/apportionment/src/chapters/population.rs',
                    'explainer_corpus/apportionment/src/chapters/recap.rs'],
                   cwd=R, check=True, timeout=120)
    subprocess.run([sys.executable, '-I', str(R/'tools/coherent_narration.py'), 'verify', str(NARR),
                    '--script', str(R/'explainer_corpus/apportionment/script.tsv'),
                    '--units', str(R/'explainer_corpus/apportionment/narration_units.json'),
                    '--require-directions'], cwd=R, check=True, timeout=300)
    chapters = subprocess.check_output([str(BINARY), 'chapters'], cwd=R, text=True, timeout=120).splitlines()
    assert len(chapters) == len(set(chapters)) == 15
    qa = B/'apportionment-frederick-score445/score/stems/qa'
    assert {x.stem for x in qa.glob('*.json')} == set(chapters)
    assert all(json.loads((qa/(c+'.json')).read_text())['passed'] is True for c in chapters)
    out = JOB/'timeline'
    assert not out.exists()
    record, left = {}, 14400
    for c in chapters:
        mine, limit = owned.pressure()
        assert mine <= limit//2, (mine, limit)
        assert left > 0
        cmd = [str(BINARY), 'render', c, '--res', '64x36', '--fps', '60', '--lint',
               '--narration', str(NARR), '--out', str(out), '--crf', '16', '--preset', 'slow', '--threads', '4']
        print('TIMELINE567', c, flush=True)
        started = time.monotonic()
        code, tracked = owned.run_owned(c, cmd, left)
        left -= time.monotonic()-started
        assert code == 0, (c, code)
        comparisons = {}
        for ext in ('timeline.tsv', 'cues.tsv', 'cuelog.tsv'):
            new = (out/(c+'.'+ext)).read_text()
            old = (REF/(c+'.'+ext)).read_text()
            assert new.replace(str(NARR), str(G/'narration437')) == old, (c, ext, 'schedule changed')
            comparisons[ext] = sha(out/(c+'.'+ext))
        headers = dict(line.split('\t', 1) for line in (out/(c+'.timeline.tsv')).read_text().splitlines()
                       if line.startswith(('chapter\t', 'frames\t', 'fps\t')))
        assert headers['chapter'] == c and headers['fps'] == '60'
        record[c] = dict(frames=int(headers['frames']), tables=comparisons,
                         layout_sha256=sha(out/(c+'.layout.tsv')), tracking=tracked)
        print('EXACT_SCHEDULE', c, record[c]['frames'], flush=True)
    assert sum(x['frames'] for x in record.values()) == 100886
    with (JOB/'comparison567.json').open('x') as f:
        json.dump(dict(chapters=record, total_frames=100886,
                       all_three_tables_equal_after_narration_path_only=True, score_qa_passed=15,
                       visual_acceptance=False, listening_acceptance=False, fine_sync_acceptance=False), f, indent=2)

if __name__ == '__main__':
    main()
