"""Wait for a successful import683 and all 13 final672 chapters, then one guarded master686, then one guarded verify687.

Serial, on the CSD, and prepared for root. No retry: a failed, expired or refused step stops this controller, and any new
attempt needs new job names. It starts no render, and never touches final672, the running 673, import683 or any picture job.
Its success means only that the technical checks ran; it accepts no picture, motion, audio, fine-sync, AV or release result.
"""
import hashlib
import json
import subprocess
import time
from pathlib import Path

D = Path('/data/projects/jmev-stroke-probe-20261106')
Q = D/'qualification01/production-repair17'
JOBS = D/'jobs'
MASTER_HELPER = Q/'apportionment-csd-master686.py'
VERIFY_HELPER = Q/'apportionment-csd-verify687.py'
PINS = {MASTER_HELPER: 'f1dde8735fac0bcd7075030a2e74c1cccdc0e295cbc1e57265ca69ceb680857b',
        VERIFY_HELPER: '8920ba67279a8fa6d614c71125c3c4ebbee13e4e0a003328aedceef5fb6ea43f'}
QUALIFICATION = JOBS/'csd-apportionment-build656/qualification.json'
IMPORT_RECEIPT = JOBS/'csd-apportionment-import683-a1/resource-receipt.json'
GUARD = D/'guard_csd02.py'
PYTHON = '/usr/bin/python3'
WAIT_SECONDS = 86400
FRAMES = 100886


def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def read(path):
    return json.loads(Path(path).read_text())


def terminal(path):
    d = read(path)
    assert d.get('admitted') is True and d.get('finished') and d.get('exit') == 0 and d.get('preflight_only') is False, path
    assert not any(d.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit')), path
    assert d['inputs_sha256'] == d['final_inputs_sha256'], path


def wait(path, label):
    deadline = time.monotonic()+WAIT_SECONDS
    while True:
        try:
            d = read(path)
        except (FileNotFoundError, json.JSONDecodeError):
            d = {}
        if d.get('finished'):
            terminal(path)
            return
        assert time.monotonic() < deadline, ('wait expired; nothing further started', label)
        time.sleep(20)


def run(argv, label):
    code = subprocess.call(argv)
    assert code == 0, (label, code, 'stopped; no retry')


def guarded(name, helper):
    inputs = Q/(name+'-inputs.json')
    run([PYTHON, '-I', '-B', str(helper), 'inputs'], 'inputs '+name)
    print('LAUNCH', name, flush=True)
    run([PYTHON, '-I', '-B', str(GUARD), '--phase', 'native', '--job', name, '--inputs', str(inputs), '--',
         PYTHON, '-I', '-B', str(helper), 'run'], name)
    terminal(JOBS/name/'resource-receipt.json')


def main():
    for path, digest in PINS.items():
        assert sha(path) == digest, ('helper changed', path)
    names = ('csd-apportionment-master686', 'csd-apportionment-verify687')
    for name in names:
        assert not (JOBS/name).exists() and not (Q/(name+'-inputs.json')).exists(), ('no duplicate', name)
    chapters = read(QUALIFICATION)['chapters']
    assert len(chapters) == 15, chapters
    wait(IMPORT_RECEIPT, 'import683')
    print('IMPORT683_READY', flush=True)
    for c in chapters[2:]:
        wait(JOBS/('csd-apportionment-final672-'+c)/'resource-receipt.json', 'final672 '+c)
        print('FINAL_READY', c, flush=True)
    guarded(names[0], MASTER_HELPER)
    summary = read(JOBS/names[0]/'master686-summary.json')
    assert summary['frames'] == FRAMES and summary['chapters'] == chapters, summary['frames']
    print('MASTER686_FINISHED', summary['movie_sha256'], flush=True)
    guarded(names[1], VERIFY_HELPER)
    result = read(JOBS/names[1]/'verification687.json')
    assert result['frames'] == FRAMES and result['movie_sha256'] == summary['movie_sha256'], result['frames']
    assert result['full_av_decode'] is True and result['all_decoded_video_frames_match_chapters'] is True, result['movie_sha256']
    print('VERIFY687_FINISHED; actual picture, motion, audio, fine-sync, independent AV and release review remain', flush=True)


if __name__ == '__main__':
    main()
