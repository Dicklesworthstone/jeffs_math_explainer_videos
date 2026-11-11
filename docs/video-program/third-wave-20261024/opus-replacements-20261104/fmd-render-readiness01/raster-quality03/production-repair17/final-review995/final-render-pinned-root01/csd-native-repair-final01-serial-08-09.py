"""PREPARED, NOT LAUNCHED: serial controller for the repaired native finals 08 then 09, after 01.

1. Wait (also while 01's guard queues behind the exclusive lane, before its job folder exists) until
   csd-native-repair-final01-01-the-bump has a terminal receipt, nothing of that final is alive,
   and the receipt bytes stay the same for STABLE seconds; then require success (admitted, finished,
   exit 0, inputs = final inputs, no stop or error key). If 01 is neither alive nor terminal, stop.
2. For 08_loop_sums, then 09_boundary: check the staged launcher and runner hashes, refuse if the job or
   its inputs file exists or any repaired final is alive, run the staged launcher once (it keeps every
   run proof and guard_csd02's resource policy), then wait for that receipt as in 1.
Any refusal or failure stops the chain: no retry, no duplicate, nothing deleted or overwritten. It writes
nothing itself. Run it as /usr/bin/python3 -I -B, the interpreter of the 01 launch, so every final's
receipt command names the same interpreter. No picture, motion, audio or AV claim.
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

D = Path('/data/projects/jmev-stroke-probe-20261106')
Q = D/'qualification01/production-repair17'
LAUNCH = Q/'csd-native-repair-final01-launch.py'
LAUNCH_SHA = '3c292758cfea2a496ff6c035db80ba1b4698a4e7c7c3dcd18a042152ea23a978'
RUNNER = Q/'csd-native-repair-final01-run.py'
RUNNER_SHA = '7cafacfa788b1579ce66b9db9880c25890112b7d1d7737ebcd44d7827daf1a09'
FIRST = '01_the_bump'
SERIAL = ('08_loop_sums', '09_boundary')
POLL, STABLE, LIMIT = 60, 120, 30*3600   # 01 may first queue behind the exclusive lane


def say(text):
    print(time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()), text, flush=True)


def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def job(chapter):
    return D/'jobs'/('csd-native-repair-final01-'+chapter.replace('_', '-'))


def alive(chapter=None):
    """Processes of a repaired final: its launcher, its guard or its runner (any chapter when None)."""
    found = []
    for proc in Path('/proc').iterdir():
        if not proc.name.isdigit() or proc.name == str(os.getpid()):
            continue
        try:
            args = (proc/'cmdline').read_bytes().split(b'\0')
        except (FileNotFoundError, PermissionError, ProcessLookupError):
            continue
        ours = str(LAUNCH).encode() in args or str(RUNNER).encode() in args or \
            any(a.startswith(b'csd-native-repair-final01-') for a in args)
        if ours and (chapter is None or chapter.encode() in args or job(chapter).name.encode() in args):
            found.append(int(proc.name))
    return found


def terminal(chapter):
    receipt = job(chapter)/'resource-receipt.json'
    deadline = time.monotonic()+LIMIT
    while True:
        try:
            d = json.loads(receipt.read_text())
        except (FileNotFoundError, json.JSONDecodeError):
            d = None
        done = d is not None and ('exit' in d or 'finished' in d)
        running = alive(chapter)
        if done and not running:
            break
        assert running or done, chapter+': neither alive nor terminal; stopping'
        assert time.monotonic() < deadline, chapter+': still not terminal after %d s; stopping' % LIMIT
        time.sleep(POLL)
    first = receipt.read_bytes()
    time.sleep(STABLE)
    assert receipt.read_bytes() == first and not alive(chapter), chapter+': receipt not stable'
    d = json.loads(first)
    assert d.get('admitted') and d.get('finished') and d.get('exit') == 0, (chapter, d.get('exit'))
    assert d['inputs_sha256'] == d['final_inputs_sha256'], chapter+': inputs changed'
    assert not any(d.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit', 'preflight_only')), chapter
    say('%s final terminal 0, stable receipt %s, inputs %s' % (chapter, hashlib.sha256(first).hexdigest()[:8],
                                                                d['inputs_sha256'][:8]))


def main():
    assert sys.argv[1:] == [], 'no arguments'
    assert sys.executable == '/usr/bin/python3' and sys.flags.isolated, 'run as /usr/bin/python3 -I -B'
    assert sha(LAUNCH) == LAUNCH_SHA and sha(RUNNER) == RUNNER_SHA, 'staged launcher or runner changed'
    for chapter in SERIAL:
        assert not job(chapter).exists() and not (Q/(job(chapter).name+'-inputs.json')).exists(), chapter+' already exists'
    say('waiting for %s' % FIRST)
    terminal(FIRST)
    for chapter in SERIAL:
        assert sha(LAUNCH) == LAUNCH_SHA and sha(RUNNER) == RUNNER_SHA, 'staged launcher or runner changed'
        assert not job(chapter).exists() and not (Q/(job(chapter).name+'-inputs.json')).exists(), chapter+' already exists'
        assert not alive(), 'a repaired final is alive: %s' % alive()
        say('launching %s' % chapter)
        subprocess.run([sys.executable, '-I', '-B', str(LAUNCH), '--chapter', chapter], check=True)
        terminal(chapter)
    say('08 and 09 finals terminal 0; final extraction, actual review and master gates remain open')


if __name__ == '__main__':
    main()
