import time
import os as _os, sys as _sys
_sys.path.insert(0, _os.path.dirname(_os.path.abspath(__file__)))  # works under python3 -I from any cwd
from bakery_mc import *
for var in ["correct","no_choosing","strict_nums","le_nums"]:
    for safe in [False, True]:
        t=time.time()
        m=Bakery(2,4,var,safe=safe,fcfs=True)
        r=explore(m, check_assert2=(var=="correct"), liveness=not safe)
        print(var, "safe" if safe else "atomic", r["states"], r["bad_kind"], "pruned",r["pruned"], "stuck",r.get("stuck"), "a2",r["a2_checked"], f"{time.time()-t:.1f}s")
        if r["bad"] is not None:
            for a,b in r["trace"]: print("    ",a.ljust(38),b)
