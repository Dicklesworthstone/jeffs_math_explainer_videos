"""Exact full-rank probabilities of random matrices over GF(q), q = 2 and 256 (memo section 2.3).

Usage:  python3 -I rank_probs.py      (stdlib only, < 1 s)
"""
import math
def p_full(q,k,h=0):
    # k x (k+h) random matrix (k rows; or equivalently k unknowns, k+h equations) full rank k
    p=1.0
    for i in range(h+1,k+h+1): p*=1-q**(-i)
    return p
def p_lim(q,h=0,N=2000):
    p=1.0
    for i in range(h+1,N): p*=1-q**(-float(i))
    return p
for q in (2,256):
    print(f"q={q}")
    for k in (1,2,3,4,5,10,20,50,100):
        print(f"   k={k:4d}  P(square invertible)={p_full(q,k):.10f}")
    for h in range(0,5):
        L=p_lim(q,h); print(f"   limit k->inf, h={h}: P(full rank)={L:.12f}  P(fail)={1-L:.6e}  union bound q^-h/(q-1)={q**(-h)/(q-1):.6e}")
# ratios of failure
for q in (2,256):
    f=[1-p_lim(q,h) for h in range(5)]
    print(q,"successive failure ratios:", [f[i]/f[i+1] for i in range(4)])
# expected extra equations needed beyond k (limit) : E[N-k] = sum_{h>=0} P(fail at k+h)
for q in (2,256):
    E=sum(1-p_lim(q,h) for h in range(0,200))
    print(f"q={q}: expected # extra random equations beyond k needed (k->inf) = {E:.6f}")
# 1-(1-1/q) first factor
print("q=256 first factor 1-1/256 =",1-1/256, " 255/256 =",255/256)
# the 'small target' fraction: span of i vectors in GF(q)^k is q^i/q^k
for q in (2,256):
    print(q, "fraction of space hit by span when one dimension missing:", 1/q)
