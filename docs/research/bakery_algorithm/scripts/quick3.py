import os as _os, sys as _sys
_sys.path.insert(0, _os.path.dirname(_os.path.abspath(__file__)))  # works under python3 -I from any cwd
import time, sys
from bakery_mc import *
N=int(sys.argv[1]); K=int(sys.argv[2]); safe = sys.argv[3]=="safe"; var=sys.argv[4]
t=time.time()
m=Bakery(N,K,var,safe=safe,fcfs=False)
r=explore(m)
print(N,K,var,"safe" if safe else "atomic", r["states"], r["bad_kind"], "pruned",r["pruned"], f"{time.time()-t:.1f}s", flush=True)
