#!/usr/bin/env python3
"""Exhaustive sweeps: every variant x {atomic, safe} x N x K; plus liveness, FCFS and
Assertion-2 checks for the correct algorithm.  Prints one line per run."""
import os as _os, sys as _sys
_sys.path.insert(0, _os.path.dirname(_os.path.abspath(__file__)))  # works under python3 -I from any cwd
import sys, time
from bakery_mc import Bakery, explore

def run(N, K, var, safe, **kw):
    t = time.time()
    m = Bakery(N, K, var, safe=safe, fcfs=kw.pop("fcfs", False))
    r = explore(m, **kw)
    line = (f"N={N} K={K} {var:13s} {'safe  ' if safe else 'atomic'} states={r['states']:>9,} "
            f"pruned={r['pruned']:>7,} result={r['bad_kind'] or 'OK (no violation)'}")
    if "stuck" in r:
        line += f" | liveness: {'NO-PROGRESS STATE FOUND' if r['stuck'] is not None else 'every trying state can reach a CS entry'}"
    if r["a2_checked"]:
        line += f" | Assertion2 checked in {r['a2_checked']:,} CS states"
    line += f"  ({time.time()-t:.1f}s)"
    print(line, flush=True)
    return r

if __name__ == "__main__":
    which = sys.argv[1] if len(sys.argv) > 1 else "all"
    if which in ("all", "safety"):
        print("== SAFETY (mutual exclusion) sweep, any-order max reads ==")
        for N, Ks in ((2, (3, 5, 8)), (3, (3, 4, 5))):
            for K in Ks:
                for var in ("correct", "no_choosing", "strict_nums", "le_nums", "double_fetch"):
                    if N == 2 and var == "double_fetch":
                        continue
                    for safe in (False, True):
                        run(N, K, var, safe)
    if which in ("all", "liveness"):
        print("== LIVENESS (no reachable trying state without a path to a CS entry; atomic) ==")
        for N, K in ((2, 6), (3, 4)):
            for var in ("correct", "le_nums"):
                run(N, K, var, False, liveness=True)
    if which in ("all", "fcfs"):
        print("== FCFS ghost check + Assertion 2 (correct algorithm) ==")
        for N, K, safe in ((2, 6, False), (2, 6, True), (3, 4, False), (3, 4, True)):
            run(N, K, "correct", safe, fcfs=True, check_assert2=True)
    if which == "big":
        print("== larger runs ==")
        run(3, 6, "correct", True)
        run(4, 4, "correct", False)
        run(4, 3, "correct", True)
