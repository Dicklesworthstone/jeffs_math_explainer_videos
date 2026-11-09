"""Runs the frozen controller 452 (task455, CC1): its source, hash-checked, executed unchanged in a fresh module.
Rebound for a scratch tree D: D, Q, VERIFIER, VERIFIER_SHA (the stand-in's hash), LEDGER and POLL. Seam:
the module's `subprocess` maps argv[0] /usr/bin/python3 to this interpreter (macOS /usr/bin/python3 is 3.9, which
has no hashlib.file_digest). Hooks: mutate_sampling_after_ledger (appends a newline to the overview sampling.json
right after 452 prints its ledger line).
Usage: python3 -I -B run452_455.py REAL452 D WAIT_HOURS POLL [HOOK]
"""
import builtins
import hashlib
from pathlib import Path
import subprocess as _sp
import sys
import types

REAL, D, HOURS, POLL = Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3], float(sys.argv[4])
HOOK = sys.argv[5] if len(sys.argv) > 5 else ''
src = REAL.read_bytes()
assert hashlib.sha256(src).hexdigest() == 'beb4ea0650f577d3da70731bb5a01417c2d3550b7c84eb66aa77c3335d1fceae'
m = types.ModuleType('controller452')
m.__file__ = str(REAL)
exec(compile(src, str(REAL), 'exec'), m.__dict__)
m.D = D
m.Q = D / 'qualification01/production-repair17'
m.VERIFIER = m.Q / 'csd-final-verify451.py'
m.VERIFIER_SHA = hashlib.sha256(m.VERIFIER.read_bytes()).hexdigest()
m.LEDGER = m.Q / 'final-pictures451-overview-ledger452.json'
m.POLL = POLL


class Sub:
    PIPE, STDOUT = _sp.PIPE, _sp.STDOUT

    @staticmethod
    def Popen(argv, *a, **k):
        argv = [str(x) for x in argv]
        if argv[0] == '/usr/bin/python3':
            argv[0] = sys.executable
        return _sp.Popen(argv, *a, **k)


m.subprocess = Sub()
if HOOK == 'mutate_sampling_after_ledger':
    def hooked_print(*a, **k):
        builtins.print(*a, **k)
        if a and str(a[0]).startswith('452 ledger'):
            with (D / 'jobs/csd-final-pictures451-overview/sampling.json').open('a') as stream:
                stream.write('\n')
            builtins.print('HOOK: appended a newline to the overview sampling.json', flush=True)
    m.print = hooked_print
sys.argv = [str(REAL), '--wait-hours', HOURS]
try:
    rc = m.main()
    print('RESULT rc %r' % rc, flush=True)
    raise SystemExit(0 if rc == 0 else 3)
except SystemExit:
    raise
except BaseException as error:
    print('RESULT exc %s: %s' % (type(error).__name__, str(error)[:400]), flush=True)
    raise SystemExit(4)
