from math import comb, lgamma, log, exp, sqrt, log10
from fractions import Fraction as F
# Toy lake
c=[comb(5,k)*comb(15,4-k) for k in range(5)]
print("C(20,4)=",comb(20,4),"split",c,sum(c))
print("probs 4dp",[round(x/4845,4) for x in c],"3dp",[round(x/4845,3) for x in c],"sum3dp",sum(round(x/4845,3) for x in c))
def L(N,M,n,k):
    if N-M<n-k or N<n or N<M: return F(0)
    return F(comb(M,k)*comb(N-M,n-k),comb(N,n))
# toy ratios
for N in (33,34):
    M=n=10;k=3
    num=(N-M)*(N-n); den=N*(N-M-n+k)
    print("toy N",N,num,den,num-den, M*n-k*N, L(N,M,n,k)/L(N-1,M,n,k)==F(num,den))
print("L(33)",float(L(33,10,10,3)), "argmax", max(range(17,200),key=lambda N:L(N,10,10,3)))
for N in (9999,10000,10001):
    M=n=1000;k=100
    num=(N-M)*(N-n); den=N*(N-M-n+k); print("Feller N",N,num,den,num-den)
# MLE edge case: k=n (all of second catch marked) -> Mn/k=M integer
for (M,n,k) in [(10,4,4),(4,10,4),(10,10,10),(6,5,2)]:
    lo=M+n-k
    vals={N:L(N,M,n,k) for N in range(max(M,n),lo+300)}
    mx=max(vals.values()); arg=[N for N,v in vals.items() if v==mx]
    print("MLE edge",(M,n,k),"Mn/k=",F(M*n,k),"argmax",arg,"L(Mn/k -1)=",vals.get(M*n//k-1))
# q100(1900)
M=n=1000;k=100
lq=(2*lgamma(1001)-lgamma(101)-lgamma(1901))/log(10)
print("log10 q(1900)",lq, "=",10**(lq-int(lq)+1),"e",int(lq)-1)
def logL(N,M,n,k):
    lc=lambda a,b: lgamma(a+1)-lgamma(b+1)-lgamma(a-b+1)
    return lc(M,k)+lc(N-M,n-k)-lc(N,n)
print("log10 q(1e6)",logL(10**6,M,n,k)/log(10)," rel",(logL(10**6,M,n,k)-logL(10000,M,n,k))/log(10))
# exact sampling dist of Mn/k for Feller: SD, and delta SEs
pm={j:exp(logL(10000,M,n,j)) for j in range(1,1001)}
Z=sum(pm.values()); mean=sum(p*M*n/j for j,p in pm.items())/Z
sd=sqrt(sum(p*(M*n/j-mean)**2 for j,p in pm.items())/Z)
print("exact E[Mn/k|k>0]",mean,"exact SD of Mn/k",sd)
print("delta SE no FPC",sqrt(M*M*n*(n-k)/k**3),"with FPC (1-k/M)",sqrt(M*M*n*(n-k)/k**3*(1-k/M)))
# CV symmetric version and 'same k different split'
def exact_cv(N,M,n):
    lc=lambda a,b: lgamma(a+1)-lgamma(b+1)-lgamma(a-b+1)
    ps={j:exp(lc(M,j)+lc(N-M,n-j)-lc(N,n)) for j in range(1,min(M,n)+1) if n-j<=N-M}
    Z=sum(ps.values()); m=sum(p*M*n/j for j,p in ps.items())/Z
    s=sqrt(sum(p*(M*n/j-m)**2 for j,p in ps.items())/Z); return s/N
for (Mm,nn) in [(1000,1000),(2000,500),(500,2000)]:
    kk=Mm*nn/10000
    print("split",Mm,nn,"cv binom",sqrt(1/kk-1/nn),"cv FPC",sqrt((1/kk-1/nn)*(1-kk/Mm)),"exact cv",exact_cv(10000,Mm,nn))
# small study
N,M,n=1000,50,50
p0=F(comb(N-M,n),comb(N,n)); print("P(k=0) small",float(p0),"1 in",float(1/p0))
EC=sum(L(N,M,n,j)*(F((M+1)*(n+1),j+1)-1) for j in range(0,51))
print("exact Chapman mean small",float(EC), "bias%",float((EC-N)/N*100))
# Chapman SD in small study (to judge sim 920)
EC2=sum(L(N,M,n,j)*(F((M+1)*(n+1),j+1)-1)**2 for j in range(0,51))
sdc=sqrt(float(EC2-EC*EC)); print("Chapman SD small",sdc,"MC SE (1e4)",sdc/100,"z of 920:",(920.2-float(EC))/(sdc/100))
# Chapman identity proof check
for (N,M,n) in [(20,5,4),(30,7,9),(12,5,8),(40,3,3)]:
    E=sum(L(N,M,n,j)*(F((M+1)*(n+1),j+1)-1) for j in range(0,min(M,n)+1))
    print("Chapman id",(N,M,n), E==N-F((N+1)*comb(N-M,n+1),comb(N+1,n+1)))
