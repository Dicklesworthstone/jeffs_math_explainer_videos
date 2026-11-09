"""Test stand-in for D/guard_csd02.py (task391, CC1). Not root's guard.

Executes root's base runner run_bounded_phase_oliver_owner48g_target14g_20261102.py (819746fb, the CSD bytes)
unchanged from its repo file, with the in-memory changes guard_owner56g_target20g.py (1c19a9b8) makes (target cap
text 14 -> 20, WORK_CAP_GIB 56) and guard_csd02.py (f1cae1db) makes (ROOT/REPO = D, WORK = D/jobs, TARGET = D/work).
Test-only: ADMISSION_GIB native 0 and STOP_FREE_GIB 0 (this Mac has about 3 GiB free), raised to 10**6 for the
refuse_verify / stop_overview faults; owned-process-stop02 is not installed (the runner's own stop_owned runs);
the runner's `subprocess` is a shim: `sbh check` returns exit 0, and the child argv drops /usr/bin/timeout 86400
(absent on macOS) and maps /usr/bin/python3 to this interpreter. The receipt, lock, command.log and printed line
are the runner's own code.
"""
import hashlib
import os
from pathlib import Path
import subprocess as _sp
import sys
import types

RUNNER = Path(os.environ['FAKEGUARD_RUNNER'])
data = RUNNER.read_bytes()
assert hashlib.sha256(data).hexdigest() == '819746fb33110d6c133e184812c80a00562d848816f623078ad86f0fb1a316da'
text = data.decode()
assert text.count("14 * GIB") == 2 and text.count('"target_growth_cap_gib": 14') == 1
text = text.replace("14 * GIB", "TARGET_GROWTH_CAP_GIB * GIB")
text = text.replace('"target_growth_cap_gib": 14', '"target_growth_cap_gib": TARGET_GROWTH_CAP_GIB')
g = types.ModuleType('guard391')
g.__file__ = str(RUNNER)
g.TARGET_GROWTH_CAP_GIB = 20
exec(compile(text, str(RUNNER), 'exec'), g.__dict__)
D = Path(__file__).resolve().parent
g.ROOT, g.REPO, g.WORK, g.TARGET, g.WORK_CAP_GIB = D, D, D / 'jobs', D / 'work', 56
fault = os.environ.get('FAKEGUARD_FAULT', '')
job = sys.argv[sys.argv.index('--job') + 1]
g.ADMISSION_GIB['native'] = 10**6 if (fault == 'refuse_verify' and 'verify' in job) else 0
g.STOP_FREE_GIB = 10**6 if (fault == 'stop_overview' and job.endswith('overview')) else 0


class Sub:
    PIPE, STDOUT, TimeoutExpired, CompletedProcess = _sp.PIPE, _sp.STDOUT, _sp.TimeoutExpired, _sp.CompletedProcess

    @staticmethod
    def run(argv, *a, **k):
        if list(argv[:2]) == ['sbh', 'check']:
            return _sp.CompletedProcess(argv, 0, '{"stub": "sbh"}', '')
        return _sp.run(argv, *a, **k)

    @staticmethod
    def Popen(argv, *a, **k):
        argv = [str(x) for x in argv]
        if argv[:2] == ['/usr/bin/timeout', '86400']:
            argv = argv[2:]
        argv = [sys.executable if x == '/usr/bin/python3' else x for x in argv]
        return _sp.Popen(argv, *a, **k)


g.subprocess = Sub()
raise SystemExit(g.main())
