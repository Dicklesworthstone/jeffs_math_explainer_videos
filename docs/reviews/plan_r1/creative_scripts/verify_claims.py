from math import comb, prod
from fractions import Fraction as F
import random
# 1. likelihood area identity sum_N L(N) = Mn/(k(k-1))
def L(N,M,n,k):
    if n-k > N-M or k>M or N<n: return F(0)
    return F(comb(M,k)*comb(N-M,n-k), comb(N,n))
for (M,n,k) in [(10,10,3),(5,4,2),(7,9,4)]:
    s=sum(float(L(N,M,n,k)) for N in range(M+n-k, 200000))
    print("area",M,n,k,s, M*n/(k*(k-1)))
# 2. Feller determinant: Mn-kN == (M-k)(n-k) - k*u
M,n,k=1000,1000,100
for N in (9999,10000,10001):
    u=N-M-n+k; print(N, M*n-k*N, (M-k)*(n-k)-k*u, u)
print("toy u*", (10-3)*(10-3)/3)
# 3. Z/256 random matrix invertibility ~ GF(2) product
def inv_mod256(A):
    # invertible iff det odd iff A mod 2 invertible over GF(2)
    K=len(A); B=[[x&1 for x in r] for r in A]
    r=0
    for c in range(K):
        p=next((i for i in range(r,K) if B[i][c]),None)
        if p is None: return False
        B[r],B[p]=B[p],B[r]
        for i in range(K):
            if i!=r and B[i][c]: B[i]=[a^b for a,b in zip(B[i],B[r])]
        r+=1
    return True
# direct check of det parity claim via integer determinant on small matrices
def det(A):
    from fractions import Fraction
    A=[[Fraction(x) for x in r] for r in A]; K=len(A); d=Fraction(1)
    for c in range(K):
        p=next((i for i in range(c,K) if A[i][c]!=0),None)
        if p is None: return 0
        if p!=c: A[c],A[p]=A[p],A[c]; d=-d
        d*=A[c][c]
        for i in range(c+1,K):
            f=A[i][c]/A[c][c]; A[i]=[a-f*b for a,b in zip(A[i],A[c])]
    return int(d)
random.seed(1); T=20000; K=8; cnt=0
for _ in range(T):
    A=[[random.randrange(256) for _ in range(K)] for _ in range(K)]
    cnt+= det(A)%2==1
print("Z/256 invertible frac K=8:",cnt/T, "GF2 prod:", prod(1-2**-i for i in range(1,K+1)))
print("2*131 mod 256 =", 2*131%256)
# 4. union bound vs exact
for q in (2,256):
    for h in (0,1,2):
        exact=1-prod(1-q**-i for i in range(h+1,400))
        print("q",q,"h",h,"exact fail",exact,"bound",q**-h/(q-1))
# 5. GF(256) helpers
def gmul(a,b,poly=0x11D):
    r=0
    while b:
        if b&1: r^=a
        b>>=1; a<<=1
        if a&0x100: a^=poly
    return r
print("3*7",gmul(3,7),"3*5",gmul(3,5),"2*3",gmul(2,3),"2*128",gmul(2,128))
# rotation arithmetic mod x^8+1 : 3*255
def pmulmod(a,b,poly):
    r=0
    for i in range(8):
        if b>>i&1: r^=a<<i
    for d in range(15,7,-1):
        if r>>d&1: r^=poly<<(d-8)
    return r
print("3*255 mod x^8+1 =", pmulmod(3,255,0x101), " 2*x rotation invertible? ", len({pmulmod(2,a,0x101) for a in range(256)}))
# risk at end: P(failure among steps before last 10), q=2
print("risk before last 10 steps (q=2) <=", sum(2**-j for j in range(11,200)))
# K dependence
for K in (1,2,3,4,10,100): print("K",K,"fail GF2",1-prod(1-2**-i for i in range(1,K+1)),"GF256",1-prod(1-256**-i for i in range(1,K+1)))
# CV symmetric
N,M,n=10000,1000,1000
var=n*M*(N-M)*(N-n)/(N*N*(N-1)); print("SD k",var**.5, "CV sym", ((1-M/N)*(1-n/N)/100)**.5)
M,n=2000,500; var=n*M*(N-M)*(N-n)/(N*N*(N-1)); print("split 2000/500 SD k",var**.5, "CV", var**.5/100)
