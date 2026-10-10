from math import comb
from fractions import Fraction as F
def term(N,M,n,k): return F(comb(N-M,n-k),comb(N,n)) if N-M>=n-k and N>=n else F(0)
def tails(M,n,k,upto=12):
    A=F(n,(k-1)*comb(M-1,k-1))   # claimed total of sum_N C(N-M,n-k)/C(N,n)
    out=[];s=F(0);N0=M+n-k
    for N in range(N0,N0+upto):
        T=A-s; out.append((N,T/term(N,M,n,k))); s+=term(N,M,n,k)
    return out
for p in [(5,4,2),(6,9,4),(7,5,3)]:
    print(p,[ (N,str(r)) for N,r in tails(*p)])
