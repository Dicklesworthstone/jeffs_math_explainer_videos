"""Second batch: Bayesian view, likelihood area, toy lake 10/10/3, simulations,
assumption violations, history arithmetic."""
from math import comb, lgamma, log, exp, sqrt, floor
from fractions import Fraction as F
import numpy as np

def lc(a, b):
    return lgamma(a + 1) - lgamma(b + 1) - lgamma(a - b + 1)
def logL(N, M, n, k):
    if n - k > N - M or N < n:
        return float('-inf')
    return lc(M, k) + lc(N - M, n - k) - lc(N, n)
def L(N, M, n, k):
    if n - k > N - M or N < n or k > M:
        return F(0)
    return F(comb(M, k) * comb(N - M, n - k), comb(N, n))

M, n, k = 1000, 1000, 100
print("=== A. Likelihood is not a distribution over N: area under q_100(N) ===")
Ns = np.arange(1900, 400001)
ll = np.array([logL(int(N), M, n, k) for N in Ns])
Lv = np.exp(ll)
print("sum_N q_100(N) over N=1900..400000 =", Lv.sum(), "(not 1)")
print("tail check: q at N=400000 =", Lv[-1])

print("\n=== B. Bayesian posteriors for Feller's data ===")
def summarize(w, label):
    w = w / w.sum()
    c = np.cumsum(w)
    q = lambda a: int(Ns[np.searchsorted(c, a)])
    mean = float((Ns * w).sum())
    mode = int(Ns[np.argmax(w)])
    print(f"{label}: mode={mode}, median={q(0.5)}, mean={mean:.0f}, 95% equal-tail CrI=[{q(0.025)}, {q(0.975)}]")
summarize(Lv.copy(), "flat prior on N (1900..400000)")
summarize(Lv / Ns, "prior proportional to 1/N")

print("\n=== C. What-if sensitivities ===")
for kk in [90, 95, 99, 100, 101, 105, 110]:
    print(f"  k={kk}: Mn/k={M*n/kk:.2f}, MLE=floor -> {floor(F(M*n, kk)) if (M*n) % kk else str(M*n//kk - 1)+' & '+str(M*n//kk)}")
print("  P(K=100 | N=10000) =", exp(logL(10000, M, n, 100)))
print("  P(95<=K<=105 | N=10000) =", sum(exp(logL(10000, M, n, j)) for j in range(95, 106)))
print("  P(91<=K<=109 | N=10000) =", sum(exp(logL(10000, M, n, j)) for j in range(91, 110)))

print("\n=== D. Toy lake: tag M=10, catch n=10, see k=3 ===")
M2, n2, k2 = 10, 10, 3
for N in range(17, 61):
    r = F((N - M2) * (N - n2), N * (N - M2 - n2 + k2)) if N > 17 else None
    print(f"  N={N}: L={float(L(N,M2,n2,k2)):.4f}" + (f"  ratio={float(r):.4f}" if r else ""))
tot = sum(float(L(N, M2, n2, k2)) for N in range(17, 200001))
print("  sum over N of L(N) (to 200000) =", tot)
Lmax2 = max(float(L(N, M2, n2, k2)) for N in range(17, 300))
lr = [N for N in range(17, 5000) if float(L(N, M2, n2, k2)) / Lmax2 >= exp(-1.920729410347062)]
print("  LR 95% interval:", min(lr), max(lr))
def P_le(N, kk, M_, n_):
    return sum(float(L(N, M_, n_, j)) for j in range(0, kk + 1))
def P_ge(N, kk, M_, n_):
    return sum(float(L(N, M_, n_, j)) for j in range(kk, n_ + 1))
low = next(N for N in range(17, 5000) if P_le(N, k2, M2, n2) > 0.025)
high = max(N for N in range(17, 3000) if P_ge(N, k2, M2, n2) > 0.025)
print("  exact test-inversion 95% CI:", low, high)
print("  Chapman:", (M2 + 1) * (n2 + 1) / (k2 + 1) - 1)
print("  hypergeometric pmf if true N=33:", [round(float(L(33, M2, n2, j)), 4) for j in range(0, 11)])
print("  hypergeometric pmf if true N=30:", [round(float(L(30, M2, n2, j)), 4) for j in range(0, 11)])

print("\n=== E. Seeded Monte Carlo (numpy) ===")
rng = np.random.default_rng(20261009)
def sim(N, M_, n_, reps):
    ks = rng.hypergeometric(M_, N - M_, n_, size=reps)
    return ks
for (N, M_, n_) in [(10000, 1000, 1000), (1000, 100, 100), (1000, 50, 50), (500, 50, 50)]:
    ks = sim(N, M_, n_, 10000)
    k0 = (ks == 0).sum()
    lp = M_ * n_ / ks[ks > 0]
    ch = (M_ + 1) * (n_ + 1) / (ks + 1) - 1
    sk = float(((lp - lp.mean())**3).mean() / lp.std()**3)
    print(f" N={N},M={M_},n={n_}: k=0 in {k0}/10000; LP mean={lp.mean():.1f}, median={np.median(lp):.1f}, "
          f"2.5%={np.percentile(lp,2.5):.0f}, 97.5%={np.percentile(lp,97.5):.0f}, max={lp.max():.0f}, skew={sk:.2f}; Chapman mean={ch.mean():.1f}")
# exact versions for N=1000, M=n=100
N, M_, n_ = 1000, 100, 100
rows = [(j, exp(logL(N, M_, n_, j))) for j in range(0, 101)]
p0 = rows[0][1]
Z = 1 - p0
meanLP = sum(p * M_ * n_ / j for j, p in rows if j > 0) / Z
meanCH = sum(p * ((M_ + 1) * (n_ + 1) / (j + 1) - 1) for j, p in rows)
print(f" exact N=1000,M=n=100: P(K=0)={p0:.3g}, E[LP|K>0]={meanLP:.1f}, E[Chapman]={meanCH:.3f}")
# exact quantiles of LP for N=1000, M=n=100
cdf = 0; out = {}
for j, p in sorted([(j, p) for j, p in rows if j > 0], key=lambda t: -t[0]):
    cdf += p / Z
    for a in (0.025, 0.5, 0.975):
        if a not in out and cdf >= a:
            out[a] = M_ * n_ / j
print("  exact LP quantiles 2.5/50/97.5:", out)
print("  pmf of K at N=1000,M=n=100 (k:prob) for 3..18:", [(j, round(p, 4)) for j, p in rows[3:19]])

print("\n=== F. Assumption violations (simulation, N=10000, M=n=1000) ===")
def experiment(N, M_, n_, reps, catch_w1, catch_w2_fn):
    """catch_w1: per-fish weights for first catch; catch_w2_fn(tagged_mask) -> weights for second catch.
    Sampling without replacement proportional to weights (Efraimidis-Spirakis)."""
    ests = []
    for _ in range(reps):
        u = rng.random(N)
        keys = np.log(u) / catch_w1
        tagged_idx = np.argpartition(-keys, M_)[:M_]
        tagged = np.zeros(N, bool); tagged[tagged_idx] = True
        w2 = catch_w2_fn(tagged)
        u2 = rng.random(N)
        keys2 = np.log(u2) / w2
        caught2 = np.argpartition(-keys2, n_)[:n_]
        kk = tagged[caught2].sum()
        ests.append(M_ * n_ / kk if kk > 0 else np.inf)
    a = np.array(ests)
    return np.median(a), a.mean()
N = 10000
base = np.ones(N)
print(" ideal:", experiment(N, 1000, 1000, 400, base, lambda t: base))
# heterogeneity: some fish intrinsically 3x easier to catch, same in both catches
het = np.where(rng.random(N) < 0.3, 3.0, 1.0)
print(" heterogeneity (30% of fish 3x catchable, both catches):", experiment(N, 1000, 1000, 400, het, lambda t: het))
print(" trap-happy (tagged 1.5x more catchable later):", experiment(N, 1000, 1000, 400, base, lambda t: np.where(t, 1.5, 1.0)))
print(" trap-shy (tagged 0.67x as catchable later):", experiment(N, 1000, 1000, 400, base, lambda t: np.where(t, 2/3, 1.0)))
# tag loss: 20% of tags lost -> those fish look untagged
def tagloss(t):
    return base
def experiment_tagloss(reps, loss):
    ests = []
    for _ in range(reps):
        tagged = np.zeros(N, bool)
        tagged[rng.choice(N, 1000, replace=False)] = True
        visible = tagged & (rng.random(N) >= loss)
        caught2 = rng.choice(N, 1000, replace=False)
        kk = visible[caught2].sum()
        ests.append(1000 * 1000 / kk)
    return np.median(ests)
print(" tag loss 20%: median estimate", experiment_tagloss(400, 0.2), "(theory ~ 10000/0.8 = 12500)")
# recruitment: 2000 new untagged fish born before 2nd catch (no deaths)
def experiment_recruit(reps, add):
    ests = []
    for _ in range(reps):
        Ntot = N + add
        tagged = np.zeros(Ntot, bool)
        tagged[rng.choice(N, 1000, replace=False)] = True
        caught2 = rng.choice(Ntot, 1000, replace=False)
        kk = tagged[caught2].sum()
        ests.append(1000 * 1000 / kk)
    return np.median(ests)
print(" recruitment +2000 untagged: median", experiment_recruit(400, 2000), "(estimates ~12000 = population at 2nd catch)")
# random deaths 30% (tagged and untagged alike), no recruitment
def experiment_deaths(reps, frac):
    ests = []
    for _ in range(reps):
        tagged = np.zeros(N, bool)
        tagged[rng.choice(N, 1000, replace=False)] = True
        alive = rng.random(N) >= frac
        idx = np.flatnonzero(alive)
        caught2 = rng.choice(idx, 1000, replace=False)
        kk = tagged[caught2].sum()
        ests.append(1000 * 1000 / kk)
    return np.median(ests)
print(" 30% random deaths (no births): median", experiment_deaths(400, 0.3), "(estimates ~10000 = population at 1st catch)")
# poor mixing: tagged fish stay in one half of lake; second catch from same half vs other half
def experiment_mixing(reps, same_side):
    ests = []
    for _ in range(reps):
        side = np.zeros(N, bool); side[: N // 2] = True  # True = east half
        east = np.flatnonzero(side); west = np.flatnonzero(~side)
        tagged = np.zeros(N, bool)
        tagged[rng.choice(east, 1000, replace=False)] = True  # caught & released in east, no mixing
        pool = east if same_side else west
        # second catch: 80% from chosen half, 20% from the other
        other = west if same_side else east
        c = np.concatenate([rng.choice(pool, 800, replace=False), rng.choice(other, 200, replace=False)])
        kk = tagged[c].sum()
        ests.append(1000 * 1000 / kk if kk else np.inf)
    return np.median(ests)
print(" no mixing, 2nd catch mostly SAME side:", experiment_mixing(400, True))
print(" no mixing, 2nd catch mostly OTHER side:", experiment_mixing(400, False))

print("\n=== G. History arithmetic ===")
pop, births3 = 2037615, 215599
ann = F(births3, 3)
ratio = F(pop) / ann
print(" Laplace: annual births in sample =", float(ann), " ratio =", float(ratio), " x 1,000,000 =", float(ratio * 1000000))
print(" Laplace 1783: 26 x 973054.5 =", 26 * 973054.5, "; 26.5 x 973054.5 =", 26.5 * 973054.5)
print(" Graunt: 8 x 48000 =", 8 * 48000, "; 13000*11/3 =", 13000 * 11 / 3)
print(" Lincoln hypothetical: 5000 banded, 600 returns, 5,000,000 killed ->", 5000 * 5000000 / 600)
print(" Dahl/Ricker Rodli tarn 109,177,57: LP =", 109 * 177 / 57, " Chapman =", 110 * 178 / 58 - 1)
print(" Silverman fish example 100,100,20 ->", 100 * 100 / 20)
print(" Kosovo: 4400 named vs 10356 estimated, ratio", 10356 / 4400)
print(" Feller p(n=1900) log10 check done in verify.py")
# German tank example
print(" German tank: serials seen {19, 40, 42, 60}: m + m/k - 1 =", 60 + 60 / 4 - 1)
# two-reviewer bug example (hypothetical)
A, B, both = 30, 24, 12
print(f" Two reviewers A={A}, B={B}, overlap={both}: N_hat={A*B/both}, missed-by-both est={A*B/both - (A+B-both)}; Chapman={(A+1)*(B+1)/(both+1)-1:.2f}")
