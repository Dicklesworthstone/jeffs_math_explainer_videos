"""Guarded native encoder-stop experiment; signals only its own test descendants.

Two silent samples exercise actual preview/final encoder sizes, not audiovisual
acceptance or the later audio mux. Existing render jobs are never signalled.
"""
import ctypes
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time

ROOT = Path('/data/projects/jmev-fmd-render-qualification-20261106-1111')
JOB = ROOT / 'jobs/fmd-raster-encoder-stop10'
BIN = ROOT / 'work/bin/production09h'
PIN = json.loads((ROOT/'jobs/fmd-raster-quality-build04/qualification.json').read_text())['binary_sha256']
GUARD = ROOT / 'inputs/repo/narration_lab/production-20261020/second-wave-central/duration-and-storage-20261020/root/run_bounded_phase_oliver_owner48g_target14g_20261102.py'


def digest(path):
    with path.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def census():
    rows = {}
    for path in Path('/proc').glob('[0-9]*/stat'):
        try:
            raw = path.read_text()
            fields = raw[raw.rindex(')') + 2:].split()
            pid = int(path.parent.name)
            rows[pid] = {'pid': pid, 'ppid': int(fields[1]), 'group': int(fields[2]),
                         'state': fields[0], 'start': fields[19],
                         'command': (path.parent / 'cmdline').read_bytes().replace(b'\0', b' ').decode(errors='replace')}
        except (FileNotFoundError, ProcessLookupError, PermissionError):
            continue
    return rows


def descendants(rows):
    owned = {os.getpid()}
    while True:
        added = {pid for pid, row in rows.items() if row['ppid'] in owned} - owned
        if not added:
            return {pid: rows[pid] for pid in owned if pid != os.getpid() and pid in rows}
        owned |= added


def live_owned():
    return {p: r for p, r in descendants(census()).items() if r['state'] != 'Z'}


def cleanup():
    # Subreaper custody keeps orphaned encoders attributable to this experiment.
    for sig in (signal.SIGTERM, signal.SIGKILL):
        for pid, row in live_owned().items():
            current = census().get(pid)
            if current and current['start'] == row['start']:
                try:
                    os.kill(pid, sig)
                except ProcessLookupError:
                    pass
        deadline = time.monotonic() + 3
        while live_owned() and time.monotonic() < deadline:
            time.sleep(0.1)
    assert not live_owned(), 'Owned test process survived cleanup'


def interrupted(signum, frame):
    raise SystemExit(128 + signum)


def main():
    assert sys.argv[1:] == ['--run']
    receipt = json.loads((JOB / 'resource-receipt.json').read_text())
    assert receipt['admitted'] and 'exit' not in receipt
    assert receipt['policy'] == {'admission_gib':10,'stop_free_gib':8,'phase_growth_gib':4,
                                 'work_cap_gib':56,'target_growth_cap_gib':20,'sample_seconds':10}
    assert digest(BIN) == PIN
    assert digest(GUARD) == '819746fb33110d6c133e184812c80a00562d848816f623078ad86f0fb1a316da'
    spec = importlib.util.spec_from_file_location('original_guard', GUARD)
    guard = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(guard)
    adapter_spec = importlib.util.spec_from_file_location("owned_stop", ROOT/"qualification01/owned-process-stop02.py")
    adapter = importlib.util.module_from_spec(adapter_spec)
    adapter_spec.loader.exec_module(adapter)
    adapter.install(guard)
    assert ctypes.CDLL(None, use_errno=True).prctl(36, 1, 0, 0, 0) == 0
    signal.signal(signal.SIGTERM, interrupted)
    signal.signal(signal.SIGINT, interrupted)
    os.environ.update(TMPDIR=str(ROOT/'work/tmp'), XDG_CACHE_HOME=str(ROOT/'work/cache'))
    results = []
    try:
        for name, size, crf, preset in [('preview','960x540','18','veryfast'), ('final','3840x2160','16','slow')]:
            output = JOB / name
            assert not output.exists()
            command = [str(BIN),'render','01_the_bump','--silent','--res',size,'--fps','60',
                       '--out',str(output),'--crf',crf,'--preset',preset]
            with (JOB/(name+'.stdout')).open('xb') as stdout, (JOB/(name+'.stderr')).open('xb') as stderr:
                child = subprocess.Popen(command,cwd=ROOT/'work/probe-cwd',stdout=stdout,
                                         stderr=stderr,start_new_session=True)
                deadline = time.monotonic() + 180
                encoders = {}
                while time.monotonic() < deadline and child.poll() is None:
                    encoders = {p:r for p,r in live_owned().items() if 'ffmpeg' in r['command']}
                    width, height = map(int, size.split('x'))
                    consumed = {}
                    for pid in encoders:
                        try:
                            counters = dict(line.split(': ', 1) for line in Path(f'/proc/{pid}/io').read_text().splitlines())
                            consumed[pid] = int(counters['rchar'])
                        except FileNotFoundError:
                            pass
                    if any(n >= width*height*3 for n in consumed.values()):
                        break
                    time.sleep(0.1)
                assert encoders and child.poll() is None and any(n >= width*height*3 for n in consumed.values()), 'No live encoder with frame-scale input observed'
                # Exercise the unmodified guard stop, not an equivalent fake command.
                before = descendants(census())
                start = time.monotonic()
                guard.stop_owned(child)
                deadline = time.monotonic() + 10
                while live_owned() and time.monotonic() < deadline:
                    time.sleep(0.1)
                survivors = live_owned()
                record = {'size':size,'command':command,'before':before,'encoder_read_bytes':consumed,'native_exit':child.returncode,
                          'elapsed':time.monotonic()-start,'survivors':survivors,'pass':not survivors,'adapter_stop':adapter.LAST_STOP}
                results.append(record)
                with (JOB/(name+'-result.json')).open('x') as f:
                    json.dump(record,f,indent=2)
                assert not survivors, 'Encoder outlived guard stop; preserved failure'
                assert child.returncode != 0, 'Sample finished naturally before stop'
        with (JOB/'stop-summary.json').open('x') as f:
            json.dump({'binary_sha256':PIN,'tests':results,'pass':True,
                       'scope':'Actual silent native video encoders only; not audio mux, all load states, or audiovisual review.'},f,indent=2)
    finally:
        cleanup()


if __name__ == '__main__':
    main()
