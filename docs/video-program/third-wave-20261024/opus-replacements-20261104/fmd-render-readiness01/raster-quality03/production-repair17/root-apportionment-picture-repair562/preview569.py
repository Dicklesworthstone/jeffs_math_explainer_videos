"""Render only the three repaired chapters after all fifteen schedules match."""
import hashlib
import importlib.util
import json
import signal
import subprocess
import time
from pathlib import Path

R = Path('/Users/jemanuel/projects/jeffs_math_explainer_videos')
P = Path(__file__).resolve().parent
B = R/'narration_lab/third-wave-20261024/root/five-film-release-20261027'
JOB = B/'apportionment-picture-preview569'
NARR = B/'apportionment-picture-assembly566/narration'
BINARY = B/'apportionment-picture-build564/apportionment-native564'
CHAPTERS = ('08_signposts', '13_population', '15_recap')

def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def main():
    spec = importlib.util.spec_from_file_location('owned528', P.parent/'cc2-apportionment-preview-recovery528.py')
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
    for row in json.loads((P/'preview569-inputs.json').read_text())['files']:
        assert sha(row['path']) == row['sha256'], row['path']
    for name in ('apportionment-picture-timeline567', 'apportionment-picture-build564',
                 'apportionment-picture-assembly566', 'apportionment-frederick-score445'):
        owned.success(B/name/'resource-receipt.json')
    comparison = json.loads((B/'apportionment-picture-timeline567/comparison567.json').read_text())
    assert comparison['all_three_tables_equal_after_narration_path_only'] is True
    assert comparison['total_frames'] == 100886 and comparison['score_qa_passed'] == 15
    registry = subprocess.check_output([str(BINARY), 'chapters'], cwd=R, text=True, timeout=120).splitlines()
    assert len(registry) == 15 and set(registry) == set(comparison['chapters'])
    assert set(CHAPTERS) <= set(registry)
    qa = owned.STEMS/'qa'
    assert {x.stem for x in qa.glob('*.json')} == set(registry)
    assert all(json.loads((qa/(c+'.json')).read_text())['passed'] is True for c in registry)
    assert sha(BINARY) == '87a34bb262419c8a24b1c75b80d2eb97046cbc1f72fd5b2be11043ba5982a107'
    owned.REF = B/'apportionment-picture-timeline567/timeline'
    out = JOB/'preview'
    assert not out.exists()
    record, left = {}, 14400
    for c in CHAPTERS:
        mine, limit = owned.pressure()
        assert mine <= limit//2 and left > 0, (mine, limit, left)
        command = [str(BINARY), 'render', c, '--res', '960x540', '--fps', '60', '--lint',
                   '--narration', str(NARR), '--score', str(owned.STEMS), '--out', str(out),
                   '--crf', '16', '--preset', 'slow', '--threads', '4']
        print('REPAIRED_PREVIEW569', c, flush=True)
        started = time.monotonic()
        code, tracked = owned.run_owned(c, command, left)
        left -= time.monotonic()-started
        assert code == 0, (c, code)
        record[c] = dict(owned.chapter(out, c), tracking=tracked)
        print('TECHNICALLY_CHECKED', c, flush=True)
    with (JOB/'preview569-summary.json').open('x') as f:
        json.dump(dict(chapters=record, other_twelve_chapters_not_rendered=True,
                       scope='Repaired previews only. Actual picture review, listening, motion, fine sync and independent final AV remain open.'), f, indent=2)

if __name__ == '__main__':
    main()
