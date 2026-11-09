"""Runs the frozen controller 388 (task391, CC1): its source, hash-checked, executed unchanged in a fresh module.
Rebound for a scratch tree D: D, Q, VERIFIER, VERIFIER_SHA (the stand-in's hash), LEDGER and POLL. Seam:
the module's `subprocess` maps argv[0] /usr/bin/python3 to this interpreter (macOS /usr/bin/python3 is 3.9, which
has no hashlib.file_digest). Hooks: mutate_sampling_after_ledger (appends a newline to the overview sampling.json
right after 388 prints its ledger line).
Usage: python3 -I -B run388_391.py REAL388 D WAIT_HOURS POLL [HOOK]
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
assert hashlib.sha256(src).hexdigest() == '5599e007e6b1debf37fe98a22ad1db322896e446a07d3f837fcddcd9e33fb59e'
m = types.ModuleType('controller388')
m.__file__ = str(REAL)
exec(compile(src, str(REAL), 'exec'), m.__dict__)
m.D = D
m.Q = D / 'qualification01/production-repair17'
m.VERIFIER = m.Q / 'csd-final-verify377.py'
m.VERIFIER_SHA = hashlib.sha256(m.VERIFIER.read_bytes()).hexdigest()
m.LEDGER = m.Q / 'final-pictures377-overview-ledger388.json'
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
        if a and str(a[0]).startswith('388 ledger'):
            with (D / 'jobs/csd-final-pictures377-overview/sampling.json').open('a') as stream:
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
