from math import comb
from fractions import Fraction as F
import sys
sys.setrecursionlimit(10000)
# Exact check via telescoping-free route: compare partial sums S(Nmax) with closed form; tail ~ c*Nmax^-(k-1)
def L(N,M,n,k):
    if N-M<n-k or N<n or N<M: return F(0)
    return F(comb(M,k)*comb(N-M,n-k),comb(N,n))
def check(M,n,k,Nmax):
    s=F(0); s1=F(0); sinv=F(0)
    for N in range(M+n-k, Nmax+1):
        l=L(N,M,n,k); s+=l; s1+=N*l; sinv+=l/N
    A=F(M*n,k*(k-1))
    B=F(M*(M-1)*n*(n-1),k*(k-1)*(k-2)) if k>=3 else None
    return float(s),float(A),float(A-s), (float(s1),float(B),float(B-s1)) if B else None, float(sinv), 1/k
for (M,n,k) in [(6,9,4),(12,8,5),(10,10,6),(7,5,3),(20,15,7)]:
    print((M,n,k),check(M,n,k,4000))
