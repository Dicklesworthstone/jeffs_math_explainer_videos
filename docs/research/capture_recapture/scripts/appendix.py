# Consolidated verification of the key numbers (Python 3; numpy only for the Monte Carlo).
# Notation: N lake size, M tagged (Feller n1), n second catch (Feller r), k tagged in catch.
from math import comb, lgamma, log, exp, sqrt
from fractions import Fraction as F
import numpy as np

def L(N, M, n, k):                       # exact P(K=k | N,M,n)
    if n - k > N - M or k > M or N < n: return F(0)
    return F(comb(M, k) * comb(N - M, n - k), comb(N, n))
def logL(N, M, n, k):
    lc = lambda a, b: lgamma(a+1) - lgamma(b+1) - lgamma(a-b+1)
    return lc(M, k) + lc(N-M, n-k) - lc(N, n) if n - k <= N - M else float('-inf')

# 1. Ratio identity L(N)/L(N-1) = (N-M)(N-n) / (N(N-M-n+k)), exact
assert all(L(N,M,n,k)/L(N-1,M,n,k) == F((N-M)*(N-n), N*(N-M-n+k))
           for (M,n,k) in [(5,4,1),(10,10,3),(1000,1000,100)]
           for N in range(M+n-k+1, M+n-k+200))
# 2. Toy lake N=20, M=5, n=4
print([comb(5,j)*comb(15,4-j) for j in range(5)], comb(20,4))     # [1365,2275,1050,150,5] 4845
# 3. Feller: M=n=1000, k=100
M, n, k = 1000, 1000, 100
print(L(9999,M,n,k) == L(10000,M,n,k), float(L(10000,M,n,k)))     # True 0.04428917...
print((2*lgamma(1001) - lgamma(101) - lgamma(1901)) / log(10))     # -429.27 (Feller: ~10^-430)
le = lambda N, kk: sum(L(N,M,n,j) for j in range(kk+1))
print(float(le(8500,100)), float(1 - le(12000,99)))                 # 0.03468, 0.02887
lmax = logL(10000,M,n,k); rel = lambda N: exp(logL(N,M,n,k) - lmax)
print([round(rel(N),4) for N in (8000,8500,9000,11000,12000,13000)])
lr = [N for N in range(1900, 30000) if rel(N) >= exp(-3.841458820694124/2)]
print("LR 95%:", lr[0], lr[-1])                                      # 8445 12021
P_le = lambda N: sum(exp(logL(N,M,n,j)) for j in range(k+1))
P_ge = lambda N: sum(exp(logL(N,M,n,j)) for j in range(k, n+1))
lo = next(N for N in range(1900, 20000) if P_le(N) > 0.025)
hi = next(N for N in range(20000, 1900, -1) if P_ge(N) > 0.025)
print("exact test-inversion 95%:", lo, hi)                           # 8400 12073
print("Chapman:", (M+1)*(n+1)/(k+1) - 1,                             # 9919.80
      "SE_LP:", sqrt(M*M*n*(n-k)/k**3),                              # 948.68
      "SE_Ch:", sqrt((M+1)*(n+1)*(M-k)*(n-k)/((k+1)**2*(k+2))))      # 883.19
# 4. Chapman exact bias: E = N - (N+1) C(N-M,n+1)/C(N+1,n+1)  (zero iff M+n >= N)
for (N_, M_, n_) in [(20,5,4),(20,10,10),(1000,50,50)]:
    E = sum(L(N_,M_,n_,j) * (F((M_+1)*(n_+1), j+1) - 1) for j in range(min(M_,n_)+1))
    assert E - N_ == -F((N_+1)*comb(N_-M_, n_+1), comb(N_+1, n_+1)); print(N_, M_, n_, float(E))
# 5. Exact sampling distribution of Mn/k (no simulation needed): quantiles and conditional mean
def lp_summary(N_, M_, n_):
    rows = [(j, exp(logL(N_,M_,n_,j))) for j in range(min(M_,n_)+1)]
    Z = 1 - rows[0][1]; c = 0; q = {}
    for j, p in sorted(rows[1:], key=lambda t: -t[0]):
        c += p / Z
        for a in (.025, .5, .975):
            if a not in q and c >= a: q[a] = M_*n_/j
    return rows[0][1], sum(p*M_*n_/j for j, p in rows[1:]) / Z, q
print(lp_summary(10000,1000,1000))   # P(K=0)=4.4e-49, E[.|K>0]=10082.5, 8475/10000/12048
print(lp_summary(1000,100,100))      # 1.5e-5, 1102.0, 625/1000/2000
# 6. Bayesian posterior (flat prior and 1/N prior) on N in [1900, 400000]
Ns = np.arange(1900, 400001); w = np.exp([logL(int(x),M,n,k) for x in Ns])
print("area under likelihood:", w.sum(), "vs Mn/(k(k-1)) =", M*n/(k*(k-1)))   # 101.0101
for lab, ww in (("flat", w), ("1/N", w/Ns)):
    c = np.cumsum(ww/ww.sum()); qq = lambda a: int(Ns[np.searchsorted(c, a)])
    print(lab, qq(.025), qq(.5), qq(.975))   # flat 8536 10121 12186 ; 1/N 8475 10039 12073
# 7. Monte Carlo rain (seeded)
rng = np.random.default_rng(20261009)
ks = rng.hypergeometric(M, 10000-M, n, size=10000); est = M*n/ks
print(est.mean(), np.median(est), np.percentile(est, [2.5, 97.5]))  # ~10080, 10000, [8475, 12048]
# 8. History arithmetic
print(2037615 / (215599/3) * 1_000_000)   # Laplace 1802: 28,352,844.86 -> 28,352,845
