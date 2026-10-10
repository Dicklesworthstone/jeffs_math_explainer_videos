import os as _os, sys as _sys
_sys.path.insert(0, _os.path.dirname(_os.path.abspath(__file__)))  # works under python3 -I from any cwd
from bakery_mc import Bakery, explore
for N,K in ((2,6),(2,9)):
    r=explore(Bakery(N,K,"double_fetch",safe=True))
    print(f"double_fetch N={N} K={K} safe: states={r['states']:,} result={r['bad_kind'] or 'OK'}")
r=explore(Bakery(3,3,"double_fetch",safe=True,any_order=False))
print("double_fetch N=3 K=3 safe fixed-order:", r["states"], r["bad_kind"])
for a,b in r.get("trace",[]): print("   ",a.ljust(44),b)
