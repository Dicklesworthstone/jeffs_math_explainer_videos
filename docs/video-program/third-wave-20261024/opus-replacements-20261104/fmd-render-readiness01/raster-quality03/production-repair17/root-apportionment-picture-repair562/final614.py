"""Render one final 4K chapter under the existing serial guard after actual preview review."""
import hashlib
import importlib.util
import json
import signal
import subprocess
import time
import sys
from pathlib import Path

R = Path('/Users/jemanuel/projects/jeffs_math_explainer_videos')
P = Path(__file__).resolve().parent
B = R/'narration_lab/third-wave-20261024/root/five-film-release-20261027'
CHAPTER = sys.argv[1] if len(sys.argv) == 2 else ''
assert CHAPTER and all(x.isalnum() or x == '_' for x in CHAPTER)
JOB = B/('apportionment-final614-'+CHAPTER)
NARR = B/'apportionment-picture-assembly566/narration'
BINARY = B/'apportionment-picture-build564/apportionment-native564'
CHAPTERS = (CHAPTER,)

def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def main():
    spec = importlib.util.spec_from_file_location('owned528', P.parent/'root-apportionment-final-support613.py')
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
    for row in json.loads((P/('final614-'+CHAPTER+'-inputs.json')).read_text())['files']:
        assert sha(row['path']) == row['sha256'], row['path']
    for name in ('apportionment-picture-timeline567', 'apportionment-picture-build564',
                 'apportionment-picture-assembly566', 'apportionment-frederick-score445',
                 'apportionment-picture-preview569', 'apportionment-picture-review584'):
        owned.success(B/name/'resource-receipt.json')
    assert sha(P.parent/'root-apportionment-final-support613.py') == 'dfa086950e9b771e03d0feb59f755348a9df5b4fd40519ba07ea7936df4133b0'
    assert sha(P/'pictures-viewed610.json') == 'a76504654c821ab8bd7de38e0a0748d3dee9e68e2a55483fdd2de46916f58398'
    review = json.loads((P/'pictures-viewed610.json').read_text())
    assert review['total_selected_pictures_viewed'] == 379 and review['total_sheets_viewed'] == 49
    assert review['concrete_sampled_layout_defects_found'] == []
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
    out = JOB/'render'
    assert not out.exists()
    record, left = {}, 14400
    for c in CHAPTERS:
        mine, limit = owned.pressure()
        assert mine <= limit//2 and left > 0, (mine, limit, left)
        command = [str(BINARY), 'render', c, '--res', '3840x2160', '--fps', '60', '--lint',
                   '--narration', str(NARR), '--score', str(owned.STEMS), '--out', str(out),
                   '--crf', '16', '--preset', 'slow', '--threads', '4']
        print('FINAL614', c, flush=True)
        started = time.monotonic()
        code, tracked = owned.run_owned(c, command, left)
        left -= time.monotonic()-started
        assert code == 0, (c, code)
        record[c] = dict(owned.chapter(out, c), tracking=tracked)
        print('TECHNICALLY_CHECKED', c, flush=True)
    with (JOB/'final614-summary.json').open('x') as f:
        json.dump(dict(chapters=record, single_chapter=CHAPTER,
                       scope='Final chapter technical checks only. Final picture review, listening, motion, fine sync and independent final AV remain open.'), f, indent=2)

if __name__ == '__main__':
    main()
