"""Independent re-verification of the numbers review round 1 introduced into plan
revision 2 (Videos A, B, C). Standard library only. Run: python3 -I integration_check.py
(about 1-3 minutes; section 5 sums slowly). Expected output: integration_check.out.txt."""
from math import comb, exp, lgamma, log, sqrt
from fractions import Fraction as F
import random

def pmf(N, M, n, k):
    if k < 0 or k > M or n - k > N - M or n - k < 0: return F(0)
    return F(comb(M, k) * comb(N - M, n - k), comb(N, n))

# 1. Mn - kN == ab - k*u
for (M, n, k, N) in [(1000,1000,100,9999),(1000,1000,100,10000),(10,10,3,33),(10,10,3,34)]:
    a, b, u = M-k, n-k, N-M-n+k
    assert M*n - k*N == a*b - k*u
print("1. Mn-kN == ab-ku: ok; Feller u_hat =", 900*900//100, "toy u_hat =", 7*7//3, "-> N =", 7+7+3+7*7//3)

# 2. Chapman exact mean, N=1000, M=n=50
N, M, n = 1000, 50, 50
E = sum(pmf(N,M,n,k) * F((M+1)*(n+1), k+1) for k in range(0, n+1)) - 1
p0 = pmf(N,M,n,0)
naive = sum(pmf(N,M,n,k) * F(M*n, k) for k in range(1, n+1)) / (1 - p0)
print(f"2. Chapman exact mean {float(E):.1f} ({float(E/N-1)*100:+.1f}%); P(k=0)={float(p0):.4f}; naive mean given k>0 {float(naive):.0f}")

# 3. exact SD of Mn/K given K>0 at Feller numbers; and the two splits
def sd_est(N, M, n):
    ks = range(1, min(M, n)+1)
    ws = [exp(lgamma(M+1)-lgamma(k+1)-lgamma(M-k+1)+lgamma(N-M+1)-lgamma(n-k+1)-lgamma(N-M-n+k+1)-(lgamma(N+1)-lgamma(n+1)-lgamma(N-n+1))) if n-k <= N-M else 0.0 for k in ks]
    Z = sum(ws); m1 = sum(w*M*n/k for w, k in zip(ws, ks))/Z; m2 = sum(w*(M*n/k)**2 for w, k in zip(ws, ks))/Z
    return m1, sqrt(m2 - m1*m1)
m1, sd = sd_est(10000, 1000, 1000); print(f"3. Feller: mean {m1:.1f} SD {sd:.1f} CV {sd/m1*100:.2f}%")
for (M_, n_) in [(2000, 500), (500, 2000)]:
    m1, sd = sd_est(10000, M_, n_); print(f"   split M={M_}, n={n_}: mean {m1:.1f} SD {sd:.1f} CV {sd/m1*100:.2f}%")
k=100; print(f"   approx CVs: binomial sqrt(1/k-1/n)={sqrt(1/k-1/1000)*100:.2f}%  symmetric sqrt((1-M/N)(1-n/N)/k)={sqrt(0.9*0.9/k)*100:.2f}%  (2000/500): {sqrt(0.8*0.95/100)*100:.2f}%")

# 4. bug example: n1=30, n2=24, k=12 ; exact 95% CI by test inversion
M, n, k = 30, 24, 12
def cdf_le(N, kk): return sum(pmf(N,M,n,j) for j in range(0, kk+1))
lo = next(N for N in range(M+n-k, 2000) if cdf_le(N, k) > F(25,1000))
hi = max(N for N in range(M+n-k, 2000) if 1 - cdf_le(N, k-1) > F(25,1000))
ch = F((M+1)*(n+1), k+1) - 1; var = F((M+1)*(n+1)*(M-k)*(n-k), (k+1)**2*(k+2))
print(f"4. bugs: MLE {M*n//k}; seen {M+n-k}; exact 95% CI [{lo}, {hi}] -> unseen {lo-(M+n-k)}..{hi-(M+n-k)}; Chapman {float(ch):.1f} +- {sqrt(float(var)):.1f}")

# 5. likelihood area identity
for (M, n, k) in [(10,10,3),(1000,1000,100),(7,5,2)]:
    s = sum(exp(lgamma(M+1)-lgamma(k+1)-lgamma(M-k+1)+lgamma(N-M+1)-lgamma(n-k+1)-lgamma(N-M-n+k+1)-(lgamma(N+1)-lgamma(n+1)-lgamma(N-n+1))) for N in range(M+n-k, 3_000_000 if M>100 else 200000))
    print(f"5. area M={M} n={n} k={k}: {s:.4f} vs Mn/(k(k-1)) = {M*n/(k*(k-1)):.4f} = (Mn/k)/(k-1)")

# 6. carousel at 1% loss, K=10000
K, p = 10000, 0.01
EL = sum(1 - (1 - p**m)**K for m in range(0, 30))
print(f"6. carousel mean loops {EL:.3f}; P(all 1e6 phones done by m loops): " + ", ".join(f"m={m}: {(1-p**m)**(K*10**6):.3f}" for m in (4,5,6,7)))
print(f"   coupon K*H_K = {K*sum(1/i for i in range(1,K+1)):.2f}; GF(2) h=20 failure ~ {1-__import__('math').prod(1-2.0**-i for i in range(21, 200)):.2e}")

# 7. Z/256 random matrix invertibility == mod 2 (Monte Carlo) vs GF(2) product
def det_odd_rank_mod2(A):
    m = [[x & 1 for x in row] for row in A]; r = 0; nrows = len(m)
    for c in range(nrows):
        piv = next((i for i in range(r, nrows) if m[i][c]), None)
        if piv is None: return False
        m[r], m[piv] = m[piv], m[r]
        for i in range(nrows):
            if i != r and m[i][c]: m[i] = [x ^ y for x, y in zip(m[i], m[r])]
        r += 1
    return True
random.seed(1); Kk = 8; T = 40000
ok = sum(det_odd_rank_mod2([[random.randrange(256) for _ in range(Kk)] for _ in range(Kk)]) for _ in range(T))
prod = __import__('math').prod(1-2.0**-i for i in range(1, Kk+1))
print(f"7. Z/256 K=8 invertible (via det odd <=> mod-2 rank): MC {ok/T:.4f} vs prod {prod:.4f}; 2*131 % 256 = {2*131 % 256}")

# 8. small checks
print("8. 1e6/90 =", round(1e6/90), " 1e6/110 =", round(1e6/110), " P(50 heads/100) =", round(comb(100,50)/2**100, 4), " 7/8*6/8*4/8 =", 7*6*4/512)
print("   3 reads per other process: 3(N-1); 2^35 =", 2**35, "ms/yr =", 31_557_600_000)
