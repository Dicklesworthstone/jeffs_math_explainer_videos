"""Pure LT peeling (no precode, no inactivation) with RFC 6330's Table 1 degree distribution,
K = 10,000 source symbols, N = K(1+eps) received; plus expected equations-to-full-rank (memo 2.5, 2.3).

Usage:  python3 -I lt_peel_sim.py     (stdlib only; ~1-3 min, seeded so output is reproducible)
"""
import random, math, bisect
# RFC 6330 Section 5.3.5.2, Table 1: f[d] for d = 0..30
F=[0,5243,529531,704294,791675,844104,879057,904023,922747,937311,948962,958494,966438,973160,978921,983914,988283,992138,995565,998631,1001391,1003887,1006157,1008229,1010129,1011876,1013490,1014983,1016370,1017662,1048576]
def deg(rng):
    v=rng.randrange(1<<20); return bisect.bisect_right(F,v)
def peel(K,N,rng):
    nb=[]; adj=[[] for _ in range(K)]
    for j in range(N):
        d=deg(rng); s=rng.sample(range(K),d); nb.append(set(s))
        for i in s: adj[i].append(j)
    known=[False]*K; rip=[j for j in range(N) if len(nb[j])==1]; rec=0
    untouched=sum(1 for i in range(K) if not adj[i])
    while rip:
        j=rip.pop()
        if len(nb[j])!=1: continue
        i=next(iter(nb[j]))
        if known[i]: continue
        known[i]=True; rec+=1
        for jj in adj[i]:
            if i in nb[jj]:
                nb[jj].discard(i)
                if len(nb[jj])==1: rip.append(jj)
    return rec/K, untouched/K
rng=random.Random(42)
K=10000
for eps in (0.0,0.02,0.05,0.10,0.20,0.50):
    res=[peel(K,int(K*(1+eps)),rng) for _ in range(5)]
    print(f"K={K} N=K(1+{eps:.2f}): pure-peeling recovered fraction (5 runs): {[round(r[0],4) for r in res]}; never-touched fraction: {[round(r[1],4) for r in res]}")
print("e^-4.8166 =", math.exp(-4.8166))
# expected packets to reach full rank for small k
for q in (2,256):
    for k in (5,8,20):
        E=sum(1/(1-q**(-j)) for j in range(1,k+1)); print(f"q={q} k={k}: expected random equations to reach rank k = {E:.4f} (extra {E-k:.4f})")
