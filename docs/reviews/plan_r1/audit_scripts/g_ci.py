from math import comb, lgamma, exp
from fractions import Fraction as F
def P(N,M,n,j):
    if j<0 or j>M or n-j>N-M or n-j<0: return F(0)
    return F(comb(M,j)*comb(N-M,n-j),comb(N,n))
M=n=10;k=3
lo=next(N for N in range(17,2000) if sum(P(N,M,n,j) for j in range(k+1))>F(1,40))
hi=max(N for N in range(17,2000) if sum(P(N,M,n,j) for j in range(k,n+1))>F(1,40))
print("toy CI",lo,hi,"Chapman",F(11*11,4)-1)
def lP(N,M,n,j):
    lc=lambda a,b: lgamma(a+1)-lgamma(b+1)-lgamma(a-b+1)
    return exp(lc(M,j)+lc(N-M,n-j)-lc(N,n))
M=n=1000;k=100
le=lambda N: sum(lP(N,M,n,j) for j in range(0,k+1))
ge=lambda N: sum(lP(N,M,n,j) for j in range(k,n+1))
print("Feller tails",le(8500),ge(12000),"lo",le(8399),le(8400),"hi",ge(12073),ge(12074))
print("P(K=100|10000)",lP(10000,M,n,100))
print("Laplace",2037615/(215599/3)*1e6)
