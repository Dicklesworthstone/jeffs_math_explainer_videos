"""Exact verification of every number for the capture-recapture explainer.
Notation: N = lake size (Feller's n), M = tagged in first catch (Feller's n1),
n = second-catch size (Feller's r), k = tagged fish in second catch (Feller's k).
"""
from math import comb, log10, lgamma, log, exp, floor, sqrt
from fractions import Fraction as F

def L(N, M, n, k):
    """Exact hypergeometric probability P(K=k | N, M, n) as a Fraction."""
    if N < M or N < n or k > M or k > n or n - k > N - M or k < 0:
        return F(0)
    return F(comb(M, k) * comb(N - M, n - k), comb(N, n))

def logL(N, M, n, k):
    """Natural-log likelihood via lgamma (for big N)."""
    def lc(a, b):
        return lgamma(a + 1) - lgamma(b + 1) - lgamma(a - b + 1)
    if n - k > N - M:
        return float('-inf')
    return lc(M, k) + lc(N - M, n - k) - lc(N, n)

print("=== 1. Ratio identity check (exact) ===")
ok = True
for (M, n, k) in [(5, 4, 1), (7, 5, 2), (1000, 1000, 100), (50, 60, 3)]:
    lo = M + n - k
    for N in range(lo + 1, lo + 300):
        lhs = L(N, M, n, k) / L(N - 1, M, n, k)
        rhs = F((N - M) * (N - n), N * (N - M - n + k))
        ok &= (lhs == rhs)
print("ratio identity L(N)/L(N-1) = (N-M)(N-n)/(N(N-M-n+k)) holds on all tests:", ok)

print("\n=== 2. Feller's example M=n=1000, k=100 ===")
M, n, k = 1000, 1000, 100
# exact likelihoods near the peak
for N in [9998, 9999, 10000, 10001]:
    print(N, float(L(N, M, n, k)))
print("L(9999) == L(10000) exactly:", L(9999, M, n, k) == L(10000, M, n, k))
print("ratio at N=10000:", F((10000 - M) * (10000 - n), 10000 * (10000 - M - n + k)))
# brute-force argmax over a wide range using exact log via lgamma
best = max(range(1900, 40001), key=lambda N: logL(N, M, n, k))
print("argmax (lgamma, first max) =", best)
Lmax = float(L(10000, M, n, k))
print("L_max = q_100(10000) =", Lmax)
# Feller's P(N=1900)
p1900 = L(1900, M, n, k)
print("q_100(1900) = (1000!)^2/(100! 1900!); log10 =",
      (2*lgamma(1001) - lgamma(101) - lgamma(1901)) / log(10))
# direct check with exact fraction
num, den = p1900.numerator, p1900.denominator
print("  exact log10 check:", len(str(num)) - len(str(den)), "(digit-count approx)")
# Feller's tail claims
def cdf_le(N, kk):
    return sum(L(N, M, n, j) for j in range(0, kk + 1))
def sf_ge(N, kk):
    return 1 - cdf_le(N, kk - 1)
print("P(K<=100 | N=8500)  exact =", float(cdf_le(8500, 100)), " (Feller: ~0.04 normal approx)")
print("P(K>=100 | N=12000) exact =", float(sf_ge(12000, 100)), " (Feller: ~0.03)")
# Normal approx as Feller would have done (with and without continuity correction)
from statistics import NormalDist
def hyp_mean_var(N):
    p = M / N
    return n * p, n * p * (1 - p) * (N - n) / (N - 1)
for N, side in [(8500, 'le'), (12000, 'ge')]:
    mu, var = hyp_mean_var(N)
    sd = sqrt(var)
    if side == 'le':
        z_cc = (100.5 - mu) / sd; z = (100 - mu) / sd
        print(f"N={N}: mean={mu:.3f}, sd={sd:.3f}, normal P(K<=100) cc={NormalDist().cdf(z_cc):.4f}, nocc={NormalDist().cdf(z):.4f}")
    else:
        z_cc = (99.5 - mu) / sd; z = (100 - mu) / sd
        print(f"N={N}: mean={mu:.3f}, sd={sd:.3f}, normal P(K>=100) cc={1-NormalDist().cdf(z_cc):.4f}, nocc={1-NormalDist().cdf(z):.4f}")

print("\n=== 3. Likelihood curve shape for Feller's numbers ===")
lmax = logL(10000, M, n, k)
def rel(N):
    return exp(logL(N, M, n, k) - lmax)
for N in [1900, 5000, 7000, 8000, 8500, 9000, 9500, 10000, 10500, 11000, 12000, 13000, 15000, 20000, 50000, 10**6]:
    print(f"  N={N:>8}: L/Lmax = {rel(N):.6g}")
# half-max interval and LR 95% interval (rel >= exp(-1.92))
import itertools
def interval(th):
    Ns = [N for N in range(1900, 40001) if rel(N) >= th]
    return min(Ns), max(Ns)
print("half-max (FWHM) range:", interval(0.5))
thr = exp(-3.841458820694124 / 2)
print("LR 95% interval threshold", thr, "->", interval(thr))
print("1/8 likelihood interval:", interval(1/8), " 1/32:", interval(1/32))

print("\n=== 4. Exact 95% CI by inverting the hypergeometric test ===")
# Lower limit: smallest N with P(K>=... ) hmm: small N => many tags expected.
# Reject N (too small) if P(K <= 100 | N) <= 0.025; reject N (too big) if P(K >= 100 | N) <= 0.025.
def logpmf(N, j):
    return logL(N, M, n, j)
def P_le(N, kk):
    return sum(exp(logpmf(N, j)) for j in range(0, kk + 1))
def P_ge(N, kk):
    return sum(exp(logpmf(N, j)) for j in range(kk, n + 1))
lowN = next(N for N in range(1900, 20000) if P_le(N, 100) > 0.025)
highN = next(N for N in range(20000, 1900, -1) if P_ge(N, 100) > 0.025)
print("exact (Clopper-Pearson style) 95% CI for N:", lowN, highN)
print("  check: P(K<=100|N=lowN-1)=", P_le(lowN - 1, 100), " P(K<=100|lowN)=", P_le(lowN, 100))
print("  check: P(K>=100|N=highN)=", P_ge(highN, 100), " P(K>=100|highN+1)=", P_ge(highN + 1, 100))
# Exactly verify the boundary values with Fractions
print("  exact-fraction check lower:", float(cdf_le(lowN - 1, 100)), float(cdf_le(lowN, 100)))
print("  exact-fraction check upper:", float(sf_ge(highN, 100)), float(sf_ge(highN + 1, 100)))

print("\n=== 5. Point estimators and variance formulas ===")
LP = F(M * n, k)
CH = F((M + 1) * (n + 1), k + 1) - 1
print("Lincoln-Petersen Mn/k =", LP, float(LP))
print("Chapman (M+1)(n+1)/(k+1) - 1 =", CH, float(CH))
var_LP = M**2 * n * (n - k) / k**3
print("LP variance approx M^2 n (n-k)/k^3 =", var_LP, "SE =", sqrt(var_LP))
var_CH = (M + 1) * (n + 1) * (M - k) * (n - k) / ((k + 1)**2 * (k + 2))
print("Chapman/Seber variance (M+1)(n+1)(M-k)(n-k)/((k+1)^2(k+2)) =", var_CH, "SE =", sqrt(var_CH))
print("  normal 95% CI around Chapman:", float(CH) - 1.96 * sqrt(var_CH), float(CH) + 1.96 * sqrt(var_CH))
# delta-method from Var(K) at N=10000
mu, var = hyp_mean_var(10000)
print("At N=10000: E[K]=", mu, "Var[K]=", var, "SD=", sqrt(var))
print("  1/k map of k=100 +/- 1.96 sd:", M * n / (100 + 1.96 * sqrt(var)), M * n / (100 - 1.96 * sqrt(var)))
# Wald CI on 1/N scale (proportion p=k/n -> N = M/p)
p = k / n
se_p = sqrt(p * (1 - p) / n)
print("  Wald on p=k/n:", M / (p + 1.96 * se_p), M / (p - 1.96 * se_p))

print("\n=== 6. Chapman exact unbiasedness identity ===")
def E_chapman(N, M, n):
    return sum(L(N, M, n, j) * (F((M + 1) * (n + 1), j + 1) - 1) for j in range(0, min(M, n) + 1))
def chapman_bias_formula(N, M, n):
    # E = N - (N+1) * C(N-M, n+1)/C(N+1, n+1)
    return -F((N + 1) * comb(N - M, n + 1), comb(N + 1, n + 1))
allok = True
for (N, M_, n_) in [(20, 5, 4), (20, 8, 8), (20, 10, 10), (20, 12, 9), (50, 10, 10), (100, 20, 30), (30, 15, 15)]:
    E = E_chapman(N, M_, n_)
    bias = E - N
    allok &= (bias == chapman_bias_formula(N, M_, n_))
    print(f"  N={N}, M={M_}, n={n_}: M+n>=N? {M_ + n_ >= N}; E[Chapman]={float(E):.6f}, bias={float(bias):.6g}")
print("  closed form bias = -(N+1)C(N-M,n+1)/C(N+1,n+1) matches:", allok)
b = chapman_bias_formula(10000, 1000, 1000)
print("  Feller-scale bias at N=10000:", float(b))
print("  P(K=0) at N=10000:", float(L(10000, 1000, 1000, 0)), " log10:", logL(10000, 1000, 1000, 0) / log(10))

print("\n=== 7. Exact sampling distribution of the estimators (no simulation needed) ===")
def est_dist(N, M, n):
    rows = []
    for j in range(0, min(M, n) + 1):
        pj = exp(logL(N, M, n, j))
        rows.append((j, pj))
    return rows
for (N, M_, n_) in [(10000, 1000, 1000), (500, 50, 50), (1000, 50, 50)]:
    rows = est_dist(N, M_, n_)
    p0 = rows[0][1]
    pos = [(j, p) for j, p in rows if j > 0]
    Z = sum(p for _, p in pos)
    mean_LP = sum(p * M_ * n_ / j for j, p in pos) / Z
    mean_CH = sum(p * ((M_ + 1) * (n_ + 1) / (j + 1) - 1) for j, p in rows)
    # quantiles of LP among k>0 (decreasing in j)
    cdf = 0; q = {}
    for j, p in sorted(pos, key=lambda t: -t[0]):
        cdf += p / Z
        for a in (0.025, 0.5, 0.975):
            if a not in q and cdf >= a:
                q[a] = M_ * n_ / j
    # mode of K
    modeK = max(rows, key=lambda t: t[1])[0]
    print(f" N={N}, M={M_}, n={n_}: E[K]={n_*M_/N:.3f}, P(K=0)={p0:.4g}, E[LP|K>0]={mean_LP:.1f} (bias {100*(mean_LP/N-1):+.2f}%),"
          f" E[Chapman]={mean_CH:.2f}, LP quantiles 2.5/50/97.5% = {q[0.025]:.0f}/{q[0.5]:.0f}/{q[0.975]:.0f}, mode K={modeK}")

print("\n=== 8. Toy lakes ===")
print(" Hypergeometric pmf for N=20, M=5, n=4:")
for j in range(0, 5):
    print(f"  k={j}: {comb(5,j)}*{comb(15,4-j)}/{comb(20,4)} = {comb(5,j)*comb(15,4-j)}/{comb(20,4)} = {float(L(20,5,4,j)):.4f}")
print(" sum check:", sum(L(20, 5, 4, j) for j in range(5)))
for (M_, n_, k_) in [(5, 4, 1), (6, 5, 2), (7, 5, 2), (5, 8, 3), (8, 6, 2), (10, 10, 3), (10, 8, 3), (6, 6, 2), (10, 10, 5)]:
    lo = M_ + n_ - k_
    vals = [(N, L(N, M_, n_, k_)) for N in range(max(lo, M_, n_), 80)]
    mx = max(v for _, v in vals)
    arg = [N for N, v in vals if v == mx]
    print(f"  M={M_}, n={n_}, k={k_}: Mn/k={M_*n_/k_:.3f}, argmax={arg}, Lmax={float(mx):.4f}, min N={lo}")

print("\n=== 9. Toy likelihood table M=10, n=8, k=3 (Mn/k=26.67) ===")
for N in range(15, 61, 1):
    r = F((N - 10) * (N - 8), N * (N - 10 - 8 + 3)) if N > 15 else None
    print(f"  N={N}: L={float(L(N,10,8,3)):.4f}" + (f", ratio L(N)/L(N-1)={float(r):.4f}" if r is not None else ""))
