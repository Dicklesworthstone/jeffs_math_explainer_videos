# Independent brute-force enumeration (no memo) of one-shot N=2 schedules, to cross-check tree_count.py
import os as _os, sys as _sys
_sys.path.insert(0, _os.path.dirname(_os.path.abspath(__file__)))  # works under python3 -I from any cwd
from tree_count import make_bakery
def brute(progs, names):
    res=[0,0,0]
    def dfs(pcs, sh, ms, inc, bad):
        moved=False
        for i in range(2):
            if pcs[i]>=len(progs[i]): continue
            op=progs[i][pcs[i]]
            sh2=dict(sh); ms2=list(ms); inc2=list(inc)
            if op[0]=="await":
                if not op[1](sh,i): continue
                if op[2]: inc2[i]=1
            elif op[0]=="w": sh2[op[1]]=op[2](sh,ms[i],i)
            elif op[0]=="rmax": ms2[i]=max(ms[i],sh[op[1]])
            elif op[0]=="exit": inc2[i]=0; sh2[op[1]]=0
            moved=True
            p2=list(pcs); p2[i]+=1
            dfs(p2,sh2,ms2,inc2,bad or sum(inc2)>=2)
        if not moved:
            res[0]+=1; res[1]+=int(bad); res[2]+=int(not all(pcs[i]>=len(progs[i]) for i in range(2)))
    dfs([0,0],{v:0 for v in names},[0,0],[0,0],False)
    return res
for wc in (False,True):
    for cmp in ("lex","strict","le"):
        p,n=make_bakery(2,wc,cmp); print(wc,cmp,brute(p,n))
