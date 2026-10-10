import os as _os, sys as _sys
_sys.path.insert(0, _os.path.dirname(_os.path.abspath(__file__)))  # works under python3 -I from any cwd
import time, bakery_tso as b
for fences in (("F1","F2"),):
    for (N,K,B) in ((2,6,4),(3,3,3)):
        t=time.time(); m=b.TSOBakery(N,K,fences,B=B); n,bad,tr=b.explore_tso(m)
        print(f"TSO N={N} K={K} B={B} fences={'+'.join(fences)}: states={n:,} -> {'VIOLATED' if bad else 'OK'} ({time.time()-t:.0f}s)", flush=True)
