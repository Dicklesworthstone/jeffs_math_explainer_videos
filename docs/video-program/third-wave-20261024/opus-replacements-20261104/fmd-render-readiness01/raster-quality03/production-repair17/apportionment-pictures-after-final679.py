"""Wait for a successful import683 and each successful final672 chapter, then run one guarded picture extraction.

Serial through chapters 03-15, on the CSD. No retry: a failed, expired or refused step stops this controller, and any new
attempt needs new job names. It starts no render and never touches final672, the running 673 or chapters 01 and 02.
"""
import hashlib
import json
import subprocess
import time
from pathlib import Path

D = Path('/data/projects/jmev-stroke-probe-20261106')
Q = D/'qualification01/production-repair17'
JOBS = D/'jobs'
HELPER = Q/'apportionment-csd-pictures678.py'
HELPER_SHA = '7a2942b6bd33765e40076294d88473505b531b900a2f097aacd3d95cf0ffa061'
QUALIFICATION = JOBS/'csd-apportionment-build656/qualification.json'
IMPORT_RECEIPT = JOBS/'csd-apportionment-import683-a1/resource-receipt.json'
GUARD = D/'guard_csd02.py'
PYTHON = '/usr/bin/python3'
WAIT_SECONDS = 86400


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


def main():
    assert sha(HELPER) == HELPER_SHA, 'helper 678 changed'
    chapters = read(QUALIFICATION)['chapters']
    assert len(chapters) == 15, chapters
    chapters = chapters[2:]
    for c in chapters:
        name = 'csd-apportionment-pictures678-'+c
        assert not (JOBS/name).exists() and not (Q/(name+'-inputs.json')).exists(), ('no duplicate', name)
    wait(IMPORT_RECEIPT, 'import683')
    print('IMPORT683_READY', flush=True)
    run([PYTHON, '-I', '-B', str(HELPER), 'preflight'], 'preflight')
    for c in chapters:
        wait(JOBS/('csd-apportionment-final672-'+c)/'resource-receipt.json', 'final672 '+c)
        print('FINAL_READY', c, flush=True)
        name = 'csd-apportionment-pictures678-'+c
        run([PYTHON, '-I', '-B', str(HELPER), 'inputs', c], 'inputs '+c)
        print('EXTRACTING', name, flush=True)
        run([PYTHON, '-I', '-B', str(GUARD), '--phase', 'native', '--job', name, '--inputs', str(Q/(name+'-inputs.json')), '--',
             PYTHON, '-I', '-B', str(HELPER), 'extract', c], name)
        terminal(JOBS/name/'resource-receipt.json')
        done = read(JOBS/name/'pictures678-summary.json')
        assert done['chapter'] == c and done['samples'] > 0 and done['sheets'] > 0, done
        print('PICTURES_READY_FOR_ROOT_REVIEW', c, done['samples'], done['sheets'], flush=True)
    print('ALL_13_EXTRACTED; no picture, sound, AV or release acceptance', flush=True)


if __name__ == '__main__':
    main()
