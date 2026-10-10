from math import comb
ms=365.25*86400*1000; print("ms/Julian yr",int(ms),"2^35",2**35, ms<2**35, "365d",365*86400*1000)
print("2^64 at 1e9/s years",2**64/1e9/(365.25*86400), "2^63:",2**63/1e9/(365.25*86400))
print("C(4,2)",comb(4,2))
# lost update: enumerate interleavings of (r1,w1) and (r2,w2)
import itertools
def leaves(progs, state, step):
    # generic DFS over interleavings; step(i,op,state)-> (newstate) or None if blocked
    res=[]
    def go(pcs,st,trace):
        moved=False
        for i,p in enumerate(progs):
            if pcs[i]<len(p):
                ns=step(i,p[pcs[i]],st)
                if ns is None: continue
                moved=True
                q=list(pcs); q[i]+=1
                go(tuple(q),ns,trace+[(i,p[pcs[i]])])
        if not moved: res.append((st,trace))
    go(tuple(0 for _ in progs),state,[])
    return res
def step_lu(i,op,st):
    x,t=st[0],list(st[1])
    if op=='r': t[i]=x; return (x,tuple(t))
    return (t[i]+1,tuple(t))
L=leaves([['r','w'],['r','w']],(41,(None,None)),step_lu)
print("lost update",len(L),"lose",sum(1 for s,_ in L if s[0]==42))
# naive flag lock: await(flag==0) as one step enabled only if passes; set flag; (inside); exit flag:=0
def step_nf(i,op,st):
    flag,ins=st[0],list(st[1]); bad=st[2]
    if op=='await':
        if flag!=0: return None
        return st
    if op=='set':
        ins[i]=1; nb=bad or sum(ins)>=2; return (1,tuple(ins),nb)
    if op=='exit':
        ins[i]=0; return (0,tuple(ins),bad)
L=leaves([['await','set','exit'],['await','set','exit']],(0,(0,0),False),step_nf)
print("naive flag",len(L),"bad",sum(1 for s,_ in L if s[2]))
