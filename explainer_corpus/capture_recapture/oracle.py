#!/usr/bin/env python3
"""Independent oracle for Video A, "How to Count Fish You've Never Seen" (slug capture_recapture).

Prints one line per fact, ``KEY<TAB>VALUE``, for every row A-F1 .. A-F32 of the facts ledger in
docs/planning/COMPREHENSIVE_PLAN_FOR_JEFFS_MATH_EXPLAINER_VIDEOS.md section 9.5, plus keys that are
not ledger rows, named by chapter: the chapter-03 habitat-dynamics check (``A-CH03.*``) and the
claim-walk keys of gate G3 (``A-CH11.*``, ``A-CH12.*``, ``A-CH14.*``, and additions to A-F9, A-F21,
A-F32 and A-CH03), each for a number the storyboard shows or says that no ledger row covered. The
R2 review round (2026-10-11) added keys for its fixes (``A-CH06.*``, ``A-CH09.*``, ``A-CH13.*``,
``A-CH12.coverage.*``, ``A-CH12.ring_toss_panel.*``, and additions to A-F6, A-F7, A-F18, A-F21,
A-F29, A-CH03, A-CH12.slider and A-CH14); no earlier key's value changed.

Run it as ``./x capture_recapture facts-check`` does (tools/facts_check.py), from this crate's
directory explainer_corpus/capture_recapture/:

    uv run --quiet --no-project --with numpy python3 -I -B oracle.py

Needs the Python 3 standard library and numpy (numpy only for the seeded simulations). Reads and
writes no files. Prints its own runtime and key count on stderr, never on stdout.

Value conventions
  * integers: exact decimal integers.
  * floats: Python ``repr`` (shortest round-trip). Exact rationals also get a ``.exact`` key "p/q".
  * booleans: ``true`` / ``false``.
  * lists: comma-separated, in the order the key's docstring states.
  * Any key containing ``.recorded.`` is an external figure copied from a cited source. It is NOT
    computed here; the fact sheet names the source.

Notation: N lake size, M marked (first catch), n second catch, k caught twice (Feller's n, n1, r, k).
Table cells: a = M - k (marked only), b = n - k (caught only second), u = N - M - n + k (dark cell).

Provenance: the hypergeometric helpers, the exact sampling-distribution summary and the
failure-mode experiments are adapted from docs/research/capture_recapture/scripts/ (verify.py,
verify2.py, appendix.py) and docs/reviews/plan_r1/ (integration_check.py, audit_scripts/).
Those scripts print at top level, so they are adapted here, not imported. The seeded simulations
replay verify2.py's exact numpy call order with numpy.random.default_rng(20261009), so they
reproduce its numbers.

A-F27 (the clustered lake for chapter 01) and A-CH03 (chapter 03's mixing dynamics) are new here.
Their model, parameters, spots and seeds were fixed before the first run and are documented in
``LAKE`` / ``DYN`` below; nothing was re-run to reach a target value.
"""

import itertools
import math
import sys
import time
import warnings
from fractions import Fraction as F
from math import comb, exp, factorial, lgamma, log, sqrt

import numpy as np

T_START = time.perf_counter()
SEED = 20261009  # the project seed used by every research script (numpy.random.default_rng(SEED))

_seen = set()


def emit(key, value):
    """Print one KEY<TAB>VALUE line; refuse duplicate keys."""
    if isinstance(value, (bool, np.bool_)):
        s = "true" if bool(value) else "false"
    elif isinstance(value, (int, np.integer)):
        s = str(int(value))
    elif isinstance(value, F):
        s = repr(float(value))
    elif isinstance(value, (float, np.floating)):
        s = repr(float(value))
    else:
        s = str(value)
    if key in _seen:
        raise SystemExit(f"duplicate key {key}")
    if "\t" in s or "\n" in s:
        raise SystemExit(f"bad value for {key}")
    _seen.add(key)
    print(f"{key}\t{s}")


def emit_exact(key, fr):
    """Emit a Fraction as a float key plus a KEY.exact = 'p/q' key."""
    emit(key, fr)
    emit(key + ".exact", f"{fr.numerator}/{fr.denominator}")


# ---------------------------------------------------------------------------------------------
# Hypergeometric helpers (adapted from verify.py / verify2.py / integration_check.py)
# ---------------------------------------------------------------------------------------------

def num(N, M, n, k):
    """Number of netfuls of size n holding exactly k marked fish: C(M,k)*C(N-M,n-k); 0 if impossible."""
    if k < 0 or k > M or k > n or n - k < 0 or n - k > N - M:
        return 0
    return comb(M, k) * comb(N - M, n - k)


def L(N, M, n, k):
    """Exact P_N(K = k) as a Fraction."""
    if N < M or N < n:
        return F(0)
    return F(num(N, M, n, k), comb(N, n))


def lc(a, b):
    return lgamma(a + 1) - lgamma(b + 1) - lgamma(a - b + 1)


def logL(N, M, n, k):
    """Natural log of P_N(K = k) via lgamma (for large N)."""
    if k < 0 or k > M or k > n or n - k > N - M or N < n:
        return float("-inf")
    return lc(M, k) + lc(N - M, n - k) - lc(N, n)


def cdf_le_num(N, M, n, k):
    """Sum_{j<=k} num(N,M,n,j): the numerator of P_N(K <= k) over C(N,n)."""
    return sum(num(N, M, n, j) for j in range(0, k + 1))


def exact_ci(M, n, k, alpha=F(1, 40)):
    """Exact (1 - 2 alpha) interval for N by inverting the two one-sided hypergeometric tests.

    lower = smallest N with P_N(K <= k) > alpha;  upper = largest N with P_N(K >= k) > alpha.
    Integer arithmetic only. P_N(K <= k) is nondecreasing in N and P_N(K >= k) nonincreasing (the
    hypergeometric is stochastically decreasing in N), so each limit is found by bisection and then
    re-checked on both sides of the boundary. Returns (lo, hi, boundary probabilities as Fractions).
    """
    a, b = alpha.numerator, alpha.denominator
    nmin = M + n - k

    def le_ok(N):
        return cdf_le_num(N, M, n, k) * b > comb(N, n) * a

    def ge_ok(N):
        T = comb(N, n)
        return (T - cdf_le_num(N, M, n, k - 1)) * b > T * a

    # lower limit
    if le_ok(nmin):
        lo = nmin
    else:
        bad, good = nmin, nmin + 1
        while not le_ok(good):
            bad, good = good, good * 2
        while good - bad > 1:
            mid = (good + bad) // 2
            if le_ok(mid):
                good = mid
            else:
                bad = mid
        lo = good
    # upper limit
    good = max(nmin, (M * n) // k)
    assert ge_ok(good)
    bad = good * 2
    while ge_ok(bad):
        good, bad = bad, bad * 2
    while bad - good > 1:
        mid = (good + bad) // 2
        if ge_ok(mid):
            good = mid
        else:
            bad = mid
    hi = good
    assert le_ok(lo) and (lo == nmin or not le_ok(lo - 1))
    assert ge_ok(hi) and not ge_ok(hi + 1)
    p_le = lambda N: F(cdf_le_num(N, M, n, k), comb(N, n))
    p_ge = lambda N: 1 - F(cdf_le_num(N, M, n, k - 1), comb(N, n))
    bounds = {
        "P_le_lo_minus_1": p_le(lo - 1) if lo > nmin else None,
        "P_le_lo": p_le(lo),
        "P_ge_hi": p_ge(hi),
        "P_ge_hi_plus_1": p_ge(hi + 1),
    }
    return lo, hi, bounds


def plain_estimate_law(N, M, n):
    """Exact law of the plain estimate Mn/K given K > 0 (hypergeometric pushed through k -> Mn/k).

    Everything exact (integers / Fractions) until the final square root. Quantiles follow
    lp_summary() in appendix.py: walk k downward (estimate upward) and take the first k whose
    cumulative conditional probability reaches a.
    """
    kmax = min(M, n)
    nums = [num(N, M, n, j) for j in range(kmax + 1)]
    T = comb(N, n)
    Z = T - nums[0]
    lcm = math.lcm(*range(1, kmax + 1))
    s1 = sum(nums[j] * (lcm // j) for j in range(1, kmax + 1))
    lcm2 = lcm * lcm
    s2 = sum(nums[j] * (lcm2 // (j * j)) for j in range(1, kmax + 1))
    mean = F(M * n * s1, Z * lcm)
    m2 = F((M * n) ** 2 * s2, Z * lcm2)
    var = m2 - mean * mean
    q = {}
    cum = 0
    for j in range(kmax, 0, -1):
        cum += nums[j]
        for tag, a in (("q025", F(1, 40)), ("q50", F(1, 2)), ("q975", F(39, 40))):
            if tag not in q and cum * a.denominator >= Z * a.numerator:
                q[tag] = j
    return {
        "p0": F(nums[0], T),
        "mean": mean,
        "var": var,
        "sd": sqrt(float(var)),
        "k": q,
        "est": {t: F(M * n, j) for t, j in q.items()},
    }


def chapman_mean_exact(N, M, n):
    """Exact E[(M+1)(n+1)/(K+1) - 1] as a Fraction."""
    T = comb(N, n)
    kmax = min(M, n)
    lcm = math.lcm(*range(1, kmax + 2))
    s = sum(num(N, M, n, j) * (lcm // (j + 1)) for j in range(kmax + 1))
    return F((M + 1) * (n + 1) * s, T * lcm) - 1


def chapman_bias_closed(N, M, n):
    """Closed form of the Chapman bias: -(N+1) C(N-M, n+1) / C(N+1, n+1) (zero iff M + n >= N)."""
    return -F((N + 1) * comb(N - M, n + 1), comb(N + 1, n + 1))


def sci(fr):
    """Exact scientific notation (mantissa as float, integer exponent) of a positive Fraction."""
    e = len(str(fr.numerator)) - len(str(fr.denominator))
    m = fr / F(10) ** e
    while m < 1:
        m *= 10
        e -= 1
    while m >= 10:
        m /= 10
        e += 1
    return float(m), e


# Feller's data, used by many rows
FM, FN_, FK = 1000, 1000, 100  # M, n, k
FELLER_N_TRUE = 10_000


# ---------------------------------------------------------------------------------------------
# A-F1 .. A-F4: the 20-fish toy lake (N = 20, M = 5, n = 4)
# ---------------------------------------------------------------------------------------------

def af1_choose_20_4():
    """A-F1: C(20,4) = 20*19*18*17/24."""
    falling = 20 * 19 * 18 * 17
    orders = math.factorial(4)
    assert falling % orders == 0
    emit("A-F1.C20_4", comb(20, 4))
    emit("A-F1.falling_product_20_19_18_17", falling)
    emit("A-F1.orders_of_4_fish", orders)
    emit("A-F1.falling_over_orders", falling // orders)


def af2_netful_counts():
    """A-F2: netfuls with k = 0..4 marked fish (N = 20, M = 5, n = 4); their sum is C(20,4) (Vandermonde)."""
    counts = [num(20, 5, 4, k) for k in range(5)]
    for k, c in enumerate(counts):
        emit(f"A-F2.k{k}", c)
    emit("A-F2.sum", sum(counts))
    emit("A-F2.sum_equals_C20_4", sum(counts) == comb(20, 4))


def af3_netful_probabilities():
    """A-F3: P(K = k) for the toy lake, exact; plus the on-screen percents (1 decimal) and their sum."""
    total_pct = F(0)
    for k in range(5):
        p = L(20, 5, 4, k)
        emit_exact(f"A-F3.p{k}", p)
        emit(f"A-F3.p{k}_4dp", round(float(p), 4))
        pct = round(float(p) * 100, 1)
        emit(f"A-F3.screen_pct{k}", pct)
        total_pct += F(str(pct))
    emit("A-F3.screen_pct_sum", float(total_pct))
    # R1 (Maya m10, Dev D21): the screen shows whole percents, the last as "<1"
    for k in range(5):
        pct = L(20, 5, 4, k) * 100
        emit(f"A-F3.screen_whole_pct{k}", "<1" if pct < 1 else round(pct))
    emit("A-F3.sum_4dp", round(sum(round(float(L(20, 5, 4, k)), 4) for k in range(5)), 4))


def af5_toy_stills():
    """R1 (Maya m3): the three stills of 05_4, P_N(K = 3) for (10, 10, 3) at
    N = 20, 33, 60, as percents to one decimal; and the partial area 17..60
    against the whole area 50/3 (R-06, D13)."""
    for N in (20, 33, 60):
        emit(f"A-F5.still_N{N}_pct", round(float(L(N, 10, 10, 3)) * 100, 1))
    emit("A-F5.area_17_to_60", round(float(sum(L(N, 10, 10, 3) for N in range(17, 61))), 2))


def af4_k2_decomposition():
    """A-F4: the k = 2 pile: C(5,2) * C(15,2) = 10 * 105 = 1,050."""
    emit("A-F4.C5_2", comb(5, 2))
    emit("A-F4.C15_2", comb(15, 2))
    emit("A-F4.product", comb(5, 2) * comb(15, 2))
    emit("A-F4.equals_A-F2.k2", comb(5, 2) * comb(15, 2) == num(20, 5, 4, 2))


# ---------------------------------------------------------------------------------------------
# A-F5 .. A-F8: the likelihood toy (M = n = 10, k = 3)
# ---------------------------------------------------------------------------------------------

def af5_toy_best_size():
    """A-F5: exact argmax of L(N) = P_N(K = 3) over N = 17..2000; L(33); the soup guess 100/3."""
    M, n, k = 10, 10, 3
    vals = {N: L(N, M, n, k) for N in range(M + n - k, 2001)}
    best = max(vals.values())
    arg = [N for N, v in vals.items() if v == best]
    emit("A-F5.min_lake", M + n - k)
    emit("A-F5.argmax", ",".join(map(str, arg)))
    emit_exact("A-F5.L33", vals[33])
    emit_exact("A-F5.soup", F(M * n, k))


def af6_toy_comparisons():
    """A-F6: Feller's comparison at the toy's peak: Mn - kN and the ratio L(N)/L(N-1) at N = 33, 34."""
    M, n, k = 10, 10, 3
    for N in (33, 34):
        numer = (N - M) * (N - n)
        denom = N * (N - M - n + k)
        assert L(N, M, n, k) / L(N - 1, M, n, k) == F(numer, denom)
        emit(f"A-F6.N{N}.Mn", M * n)
        emit(f"A-F6.N{N}.kN", k * N)
        emit(f"A-F6.N{N}.Mn_minus_kN", M * n - k * N)
        emit(f"A-F6.N{N}.ratio_numerator", numer)
        emit(f"A-F6.N{N}.ratio_denominator", denom)
        emit(f"A-F6.N{N}.numerator_minus_denominator_equals_Mn_minus_kN", numer - denom == M * n - k * N)


def af7_likelihood_area(feller_logw):
    """A-F7: area under the likelihood, sum_N L(N) = Mn/(k(k-1)) for k >= 2; infinite for k <= 1.

    Closed form (proved by a Beta integral, docs/reviews/plan_r1/ruth_45_audit.md P1) plus numeric
    partial sums: the toy by the exact ratio recurrence in floats from L(17), Feller's by lgamma over
    N = 1,900..400,000 (the same grid as verify2.py). For k = 1 the partial sums keep growing by
    about Mn*ln(10) per decade of N (L(N) ~ Mn/N), shown at N = 10^4, 10^5, 10^6.
    """
    M, n, k = 10, 10, 3
    emit_exact("A-F7.toy_area_closed_form", F(M * n, k * (k - 1)))
    N0 = M + n - k
    cur = float(L(N0, M, n, k))
    s = cur
    for N in range(N0 + 1, 200_001):
        cur *= (N - M) * (N - n) / (N * (N - M - n + k))
        s += cur
    emit("A-F7.toy_area_partial_sum_to_N200000", s)
    emit_exact("A-F7.feller_area_closed_form", F(FM * FN_, FK * (FK - 1)))
    emit("A-F7.feller_area_partial_sum_N1900_to_400000", float(np.exp(feller_logw).sum()))
    # k = 1 (toy M = n = 10): divergence
    M, n, k = 10, 10, 1
    N0 = M + n - k
    cur = float(L(N0, M, n, k))
    s = cur
    marks = {10_000: None, 100_000: None, 1_000_000: None}
    for N in range(N0 + 1, 1_000_001):
        cur *= (N - M) * (N - n) / (N * (N - M - n + k))
        s += cur
        if N in marks:
            marks[N] = s
    for N, v in marks.items():
        emit(f"A-F7.k1_toy_partial_sum_to_N{N}", v)
    emit("A-F7.k1_toy_growth_per_decade_1e5_to_1e6", marks[1_000_000] - marks[100_000])
    emit("A-F7.k1_toy_Mn_ln10", M * n * log(10))
    emit("A-F7.area_k_le_1", "inf")


def af8_toy_interval():
    """A-F8: toy exact 95% interval by test inversion (exact integers) and Chapman's estimate."""
    M, n, k = 10, 10, 3
    lo, hi, _ = exact_ci(M, n, k)
    # independent linear scan, as in verify2.py / g_ci.py
    lo_scan = next(N for N in range(17, 5000) if 40 * cdf_le_num(N, M, n, k) > comb(N, n))
    hi_scan = max(N for N in range(17, 5000) if 40 * (comb(N, n) - cdf_le_num(N, M, n, k - 1)) > comb(N, n))
    assert (lo, hi) == (lo_scan, hi_scan)
    emit("A-F8.toy_ci95_lo", lo)
    emit("A-F8.toy_ci95_hi", hi)
    emit_exact("A-F8.toy_chapman", F((M + 1) * (n + 1), k + 1) - 1)


# ---------------------------------------------------------------------------------------------
# A-F9 .. A-F17: Feller's lake (M = n = 1,000, k = 100)
# ---------------------------------------------------------------------------------------------

def af9_feller_best_size():
    """A-F9: the maximum-likelihood sizes. Exact: the sign of Mn - kN over N = 1,901..40,000 shows L
    rises strictly up to 9,999, L(10,000) = L(9,999) exactly, and falls strictly after; plus an exact
    Fraction equality check and an lgamma argmax cross-check."""
    M, n, k = FM, FN_, FK
    signs = {N: (M * n - k * N > 0) - (M * n - k * N < 0) for N in range(M + n - k + 1, 40_001)}
    assert all(signs[N] == 1 for N in signs if N < 10_000)
    assert signs[10_000] == 0
    assert all(signs[N] == -1 for N in signs if N > 10_000)
    L9999, L10000 = L(9999, M, n, k), L(10000, M, n, k)
    emit("A-F9.argmax", "9999,10000")
    emit("A-F9.tie_exact_L9999_eq_L10000", L9999 == L10000)
    emit("A-F9.k_less_than_min_M_n", k < min(M, n))
    emit("A-F9.L_max", L10000)
    emit("A-F9.L9998_lt_L9999", L(9998, M, n, k) < L9999)
    emit("A-F9.L10001_lt_L10000", L(10001, M, n, k) < L10000)
    best_lgamma = max(range(1900, 40_001), key=lambda N: logL(N, M, n, k))
    emit("A-F9.lgamma_argmax_in_tie", best_lgamma in (9999, 10000))
    # Claim walk (G3), 08_5 lower-third "floating point picks 10,000 by rounding" (D26): which side
    # of the exact tie floats land on depends on the method. lgamma: the float difference
    # logL(9,999) - logL(10,000) is pure rounding error; its sign picks the winner. The float ratio
    # L(10,000)/L(9,999) = 81,000,000/81,000,000 has both integers exact in float64, so a ratio scan
    # ties exactly, as the fractions do.
    diff = logL(9999, M, n, k) - logL(10000, M, n, k)
    emit("A-F9.lgamma_logL9999_minus_logL10000", diff)
    emit("A-F9.lgamma_prefers", 10000 if diff < 0 else 9999 if diff > 0 else "tie")
    emit("A-F9.lgamma_scan_argmax", best_lgamma)
    N = 10000
    emit("A-F9.float_ratio_L10000_over_L9999", float((N - M) * (N - n)) / float(N * (N - M - n + k)))


def af10_feller_comparisons():
    """A-F10: Mn - kN and Feller's ratio (6.3) at N = 9,999 / 10,000 / 10,001."""
    M, n, k = FM, FN_, FK
    for N in (9999, 10000, 10001):
        numer = (N - M) * (N - n)
        denom = N * (N - M - n + k)
        emit(f"A-F10.N{N}.Mn_minus_kN", M * n - k * N)
        emit(f"A-F10.N{N}.ratio_numerator", numer)
        emit(f"A-F10.N{N}.ratio_denominator", denom)
        assert numer - denom == M * n - k * N


def af11_minimum_lake():
    """A-F11: the smallest possible lake M + n - k and P_1900(K = 100) = (1000!)^2 / (100! 1900!):
    exact Fraction (checked against the factorial form), exact scientific notation, lgamma log10."""
    M, n, k = FM, FN_, FK
    Nmin = M + n - k
    p = L(Nmin, M, n, k)
    assert p == F(factorial(1000) ** 2, factorial(100) * factorial(1900))
    mant, ex = sci(p)
    emit("A-F11.min_lake", Nmin)
    emit("A-F11.P1900_equals_factorial_form", True)
    emit("A-F11.P1900_log10_lgamma", (2 * lgamma(1001) - lgamma(101) - lgamma(1901)) / log(10))
    emit("A-F11.P1900_mantissa_exact", mant)
    emit("A-F11.P1900_exponent_exact", ex)
    emit("A-F11.P1900_sci_2sf", f"{mant:.1f}e{ex}")
    return p


def af12_relative_likelihoods():
    """A-F12: L(N)/L(10,000) at N = 8,500, 12,000, 1,000,000 (lgamma)."""
    M, n, k = FM, FN_, FK
    lmax = logL(10000, M, n, k)
    for N in (8500, 12000, 1_000_000):
        emit(f"A-F12.rel_L_N{N}", exp(logL(N, M, n, k) - lmax))
    emit("A-F12.rel_L_N1000000_log10", (logL(1_000_000, M, n, k) - lmax) / log(10))


def af13_feller_interval():
    """A-F13: exact 95% interval for Feller's data by inverting the hypergeometric tests (integers),
    plus the exact boundary probabilities, plus a float linear-scan cross-check as in verify.py."""
    M, n, k = FM, FN_, FK
    lo, hi, b = exact_ci(M, n, k)
    emit("A-F13.ci95_lo", lo)
    emit("A-F13.ci95_hi", hi)
    emit(f"A-F13.P_le_100_at_N{lo - 1}", b["P_le_lo_minus_1"])
    emit(f"A-F13.P_le_100_at_N{lo}", b["P_le_lo"])
    emit(f"A-F13.P_ge_100_at_N{hi}", b["P_ge_hi"])
    emit(f"A-F13.P_ge_100_at_N{hi + 1}", b["P_ge_hi_plus_1"])
    # float cross-check (linear scans; independent of the bisection and of integer arithmetic)
    P_le = lambda N: sum(exp(logL(N, M, n, j)) for j in range(0, k + 1))
    P_ge = lambda N: 1.0 - sum(exp(logL(N, M, n, j)) for j in range(0, k))
    lo_f = next(N for N in range(1900, 20000) if P_le(N) > 0.025)
    hi_f = next(N for N in range(20000, 1900, -1) if P_ge(N) > 0.025)
    emit("A-F13.float_scan_agrees", (lo_f, hi_f) == (lo, hi))
    return lo, hi


def af14_feller_tails():
    """A-F14: exact tails behind Feller's 8,500-12,000: P_8500(K <= 100), P_12000(K >= 100), coverage."""
    M, n, k = FM, FN_, FK
    t1 = F(cdf_le_num(8500, M, n, k), comb(8500, n))
    t2 = 1 - F(cdf_le_num(12000, M, n, k - 1), comb(12000, n))
    emit("A-F14.P8500_K_le_100", t1)
    emit("A-F14.P12000_K_ge_100", t2)
    emit("A-F14.coverage_8500_12000", 1 - t1 - t2)


def af15_sd_of_k():
    """A-F15: exact SD of K at N = 10,000 (hypergeometric variance) vs the binomial approximation."""
    N, M, n = FELLER_N_TRUE, FM, FN_
    var = F(n * M * (N - M) * (N - n), N * N * (N - 1))
    emit_exact("A-F15.var_k_exact", var)
    emit("A-F15.sd_k_exact", sqrt(float(var)))
    emit("A-F15.sd_k_binomial", sqrt(n * (M / N) * (1 - M / N)))


def af16_spread_of_estimate():
    """A-F16: exact SD of Mn/K given K > 0 at N = 10,000 (exact pushforward), its CV two ways,
    and the symmetric approximation sqrt((1-M/N)(1-n/N)/k)."""
    N, M, n, k = FELLER_N_TRUE, FM, FN_, FK
    law = plain_estimate_law(N, M, n)
    emit("A-F16.sd_exact", law["sd"])
    emit("A-F16.mean_given_k_pos_exact", law["mean"])
    emit("A-F16.cv_sd_over_N", law["sd"] / N)
    emit("A-F16.cv_sd_over_mean", law["sd"] / float(law["mean"]))
    emit("A-F16.cv_symmetric_approx", sqrt((1 - M / N) * (1 - n / N) / k))
    emit("A-F16.cv_binomial_textbook_not_used", sqrt(1 / k - 1 / n))
    return law


def af17_chapman_feller():
    """A-F17: Chapman's estimate and its Seber/Chapman SE for Feller's data."""
    M, n, k = FM, FN_, FK
    ch = F((M + 1) * (n + 1), k + 1) - 1
    var = F((M + 1) * (n + 1) * (M - k) * (n - k), (k + 1) ** 2 * (k + 2))
    emit_exact("A-F17.chapman", ch)
    emit("A-F17.chapman_se", sqrt(float(var)))


def af18_chapman_identity():
    """A-F18: E[N_C] = N - (N+1) C(N-M,n+1)/C(N+1,n+1), checked with exact Fractions exhaustively for
    every 1 <= M, n <= N <= 16 and on named cases; zero bias exactly when M + n >= N."""
    cases = 0
    ok = True
    zero_iff = True
    for N in range(1, 17):
        for M in range(1, N + 1):
            for n in range(1, N + 1):
                E = chapman_mean_exact(N, M, n)
                bias = E - N
                ok &= bias == chapman_bias_closed(N, M, n)
                zero_iff &= (bias == 0) == (M + n >= N)
                cases += 1
    for (N, M, n) in [(20, 5, 4), (20, 10, 10), (30, 7, 9), (50, 10, 10), (100, 20, 30), (1000, 50, 50), (1000, 100, 100)]:
        E = chapman_mean_exact(N, M, n)
        ok &= (E - N) == chapman_bias_closed(N, M, n)
        cases += 1
    emit("A-F18.identity_holds", ok)
    emit("A-F18.zero_bias_iff_M_plus_n_ge_N", zero_iff)
    emit("A-F18.cases_checked_exact", cases)
    emit("A-F18.bias_feller_N10000", chapman_bias_closed(FELLER_N_TRUE, FM, FN_))
    for (N, M, n) in [(1000, 64, 64), (10000, 200, 200), (1000, 50, 50)]:
        b = chapman_bias_closed(N, M, n)
        emit(f"A-F18.bias_pct_N{N}_M{M}_n{n}", float(100 * b / N))
        emit(f"A-F18.expected_k_N{N}_M{M}_n{n}", F(M * n, N))


# ---------------------------------------------------------------------------------------------
# Seeded simulations: a faithful replay of verify2.py sections E and F (same rng, same call order)
# ---------------------------------------------------------------------------------------------

def verify2_replay():
    """Replay verify2.py's numpy stream: section E (four lakes, 10,000 experiments each) then
    section F (failure modes, N = 10,000, M = n = 1,000, 400 experiments each), in its order."""
    rng = np.random.default_rng(SEED)
    sims = {}
    for (N, M_, n_) in [(10000, 1000, 1000), (1000, 100, 100), (1000, 50, 50), (500, 50, 50)]:
        sims[(N, M_, n_)] = rng.hypergeometric(M_, N - M_, n_, size=10000)

    def experiment(N, M_, n_, reps, catch_w1, catch_w2_fn):
        # sampling without replacement proportional to weights (Efraimidis-Spirakis), as verify2.py
        ests = []
        for _ in range(reps):
            u = rng.random(N)
            keys = np.log(u) / catch_w1
            tagged_idx = np.argpartition(-keys, M_)[:M_]
            tagged = np.zeros(N, bool)
            tagged[tagged_idx] = True
            w2 = catch_w2_fn(tagged)
            u2 = rng.random(N)
            keys2 = np.log(u2) / w2
            caught2 = np.argpartition(-keys2, n_)[:n_]
            kk = tagged[caught2].sum()
            ests.append(M_ * n_ / kk if kk > 0 else np.inf)
        return float(np.median(np.array(ests)))

    N = 10000
    base = np.ones(N)
    fails = {}
    fails["ideal"] = experiment(N, 1000, 1000, 400, base, lambda t: base)
    het = np.where(rng.random(N) < 0.3, 3.0, 1.0)
    fails["bold_fish"] = experiment(N, 1000, 1000, 400, het, lambda t: het)
    fails["trap_happy"] = experiment(N, 1000, 1000, 400, base, lambda t: np.where(t, 1.5, 1.0))
    fails["trap_shy"] = experiment(N, 1000, 1000, 400, base, lambda t: np.where(t, 2 / 3, 1.0))

    def tagloss(reps, loss):
        ests = []
        for _ in range(reps):
            tagged = np.zeros(N, bool)
            tagged[rng.choice(N, 1000, replace=False)] = True
            visible = tagged & (rng.random(N) >= loss)
            caught2 = rng.choice(N, 1000, replace=False)
            ests.append(1000 * 1000 / visible[caught2].sum())
        return float(np.median(ests))

    def recruit(reps, add):
        ests = []
        for _ in range(reps):
            Ntot = N + add
            tagged = np.zeros(Ntot, bool)
            tagged[rng.choice(N, 1000, replace=False)] = True
            caught2 = rng.choice(Ntot, 1000, replace=False)
            ests.append(1000 * 1000 / tagged[caught2].sum())
        return float(np.median(ests))

    def deaths(reps, frac):
        ests = []
        for _ in range(reps):
            tagged = np.zeros(N, bool)
            tagged[rng.choice(N, 1000, replace=False)] = True
            alive = rng.random(N) >= frac
            idx = np.flatnonzero(alive)
            caught2 = rng.choice(idx, 1000, replace=False)
            ests.append(1000 * 1000 / tagged[caught2].sum())
        return float(np.median(ests))

    def mixing(reps, same_side):
        ests = []
        for _ in range(reps):
            side = np.zeros(N, bool)
            side[: N // 2] = True
            east = np.flatnonzero(side)
            west = np.flatnonzero(~side)
            tagged = np.zeros(N, bool)
            tagged[rng.choice(east, 1000, replace=False)] = True
            pool = east if same_side else west
            other = west if same_side else east
            c = np.concatenate([rng.choice(pool, 800, replace=False), rng.choice(other, 200, replace=False)])
            kk = tagged[c].sum()
            ests.append(1000 * 1000 / kk if kk else np.inf)
        return float(np.median(ests))

    fails["marks_lost_20pct"] = tagloss(400, 0.2)
    fails["newcomers_2000"] = recruit(400, 2000)
    fails["deaths_30pct"] = deaths(400, 0.3)
    fails["release_site"] = mixing(400, True)
    fails["far_side"] = mixing(400, False)
    return sims, fails


def af19_feller_simulation(sims, law_feller):
    """A-F19: exact quantiles of Mn/K at Feller's lake (and their k), exact mean, and the seeded
    simulation's mean / median / percentiles (verify2.py stream, first draw)."""
    M, n = FM, FN_
    for tag in ("q025", "q50", "q975"):
        emit(f"A-F19.exact_{tag}", law_feller["est"][tag])
        emit(f"A-F19.exact_{tag}_k", law_feller["k"][tag])
    emit("A-F19.exact_mean_given_k_pos", law_feller["mean"])
    ks = sims[(10000, 1000, 1000)]
    lp = M * n / ks[ks > 0]
    emit("A-F19.sim_experiments", int(ks.size))
    emit("A-F19.sim_k_zero_count", int((ks == 0).sum()))
    emit("A-F19.sim_mean", float(lp.mean()))
    emit("A-F19.sim_median", float(np.median(lp)))
    p = np.percentile(lp, [2.5, 97.5])
    emit("A-F19.sim_p2_5", float(p[0]))
    emit("A-F19.sim_p97_5", float(p[1]))
    emit("A-F19.sim_k_mean", float(ks.mean()))
    emit("A-F19.sim_k_sd", float(ks.std()))


def af20_small_study_100(sims):
    """A-F20: N = 1,000, M = n = 100: exact quantiles and mean of Mn/K given K > 0; seeded sim."""
    law = plain_estimate_law(1000, 100, 100)
    for tag in ("q025", "q50", "q975"):
        emit(f"A-F20.exact_{tag}", law["est"][tag])
    emit("A-F20.exact_mean_given_k_pos", law["mean"])
    emit("A-F20.exact_P_k0", law["p0"])
    ks = sims[(1000, 100, 100)]
    lp = 100 * 100 / ks[ks > 0]
    emit("A-F20.sim_mean", float(lp.mean()))
    emit("A-F20.sim_median", float(np.median(lp)))
    p = np.percentile(lp, [2.5, 97.5])
    emit("A-F20.sim_p2_5", float(p[0]))
    emit("A-F20.sim_p97_5", float(p[1]))


def af21_small_study_50():
    """A-F21: N = 1,000, M = n = 50: exact P(K = 0), Chapman's exact mean, plain formula given K > 0."""
    N, M, n = 1000, 50, 50
    law = plain_estimate_law(N, M, n)
    emit("A-F21.P_k0", law["p0"])
    emit("A-F21.one_in", float(1 / law["p0"]))
    ch = chapman_mean_exact(N, M, n)
    emit("A-F21.chapman_exact_mean", ch)
    emit("A-F21.chapman_bias_pct", float(100 * (ch - N) / N))
    emit("A-F21.plain_mean_given_k_pos", law["mean"])
    emit("A-F21.plain_bias_pct_given_k_pos", float(100 * (law["mean"] - N) / N))
    # Claim walk (G3), 10_3 notes "At k = 0 Chapman still answers (2,600 here)" (D12)
    ch0 = F((M + 1) * (n + 1), 0 + 1) - 1
    emit("A-F21.chapman_at_k0", ch0.numerator if ch0.denominator == 1 else ch0)


def af22_failure_modes(fails):
    """A-F22: seeded medians of Mn/k under each assumption failure (true lake 10,000, M = n = 1,000;
    400 experiments each; verify2.py section F replayed)."""
    for key in ("ideal", "bold_fish", "trap_happy", "trap_shy", "marks_lost_20pct", "newcomers_2000",
                "deaths_30pct", "release_site", "far_side"):
        emit(f"A-F22.median.{key}", fails[key])


# ---------------------------------------------------------------------------------------------
# A-F23 .. A-F26: history and the Bayesian aside
# ---------------------------------------------------------------------------------------------

def af23_laplace():
    """A-F23: Laplace's 1802 arithmetic. The inputs are recorded from the Essai (2,037,615 inhabitants;
    110,312 boys + 105,287 girls born in 1800-1802; births of France 'supposed' 1,000,000); the
    arithmetic is computed. The odds are a recorded quotation."""
    pop = 2_037_615
    boys, girls = 110_312, 105_287
    emit("A-F23.recorded.inhabitants", pop)
    emit("A-F23.recorded.births_boys_1800_1802", boys)
    emit("A-F23.recorded.births_girls_1800_1802", girls)
    births3 = boys + girls
    emit("A-F23.births_3yr", births3)
    per_yr = F(births3, 3)
    emit_exact("A-F23.births_per_year", per_yr)
    ratio = F(pop) / per_yr
    emit_exact("A-F23.ratio_people_per_annual_birth", ratio)
    emit("A-F23.ratio_6dp", f"{float(ratio):.6f}")
    emit("A-F23.recorded.assumed_births_of_France", 1_000_000)
    est = ratio * 1_000_000
    emit("A-F23.population_estimate", est)
    emit("A-F23.population_estimate_rounded", round(est))
    emit("A-F23.recorded.odds_quote", "almost 300000 against 1 that the error is not half a million")


def af24_two_reviewers():
    """A-F24: two reviewers find 30 and 24 bugs, 12 shared: seen, estimate, hiding, exact 95%
    interval by test inversion, Chapman +- SE."""
    M, n, k = 30, 24, 12
    seen = M + n - k
    emit("A-F24.reviewer_a", M)
    emit("A-F24.reviewer_b", n)
    emit("A-F24.shared", k)
    emit("A-F24.seen", seen)
    emit_exact("A-F24.estimate", F(M * n, k))
    emit("A-F24.hiding", M * n // k - seen)
    lo, hi, _ = exact_ci(M, n, k)
    emit("A-F24.ci95_lo", lo)
    emit("A-F24.ci95_hi", hi)
    emit("A-F24.hiding_lo", lo - seen)
    emit("A-F24.hiding_hi", hi - seen)
    ch = F((M + 1) * (n + 1), k + 1) - 1
    emit_exact("A-F24.chapman", ch)
    emit("A-F24.chapman_se", sqrt(float(F((M + 1) * (n + 1) * (M - k) * (n - k), (k + 1) ** 2 * (k + 2)))))


def af25_kosovo():
    """A-F25: Kosovo (recorded; Ball et al. 2002 report to the ICTY; Spagat 2014). Only the ratio and
    the 'inside the interval' check are computed."""
    named, est, lo, hi, later = 4400, 10356, 9002, 12122, 9790
    emit("A-F25.recorded.unique_named_victims", named)
    emit("A-F25.recorded.estimate", est)
    emit("A-F25.recorded.ci95_lo", lo)
    emit("A-F25.recorded.ci95_hi", hi)
    emit("A-F25.recorded.later_count_kmb", later)
    emit("A-F25.later_count_inside_interval", lo <= later <= hi)
    emit("A-F25.estimate_over_named", est / named)


def af26_bayesian(Ns, feller_logw):
    """A-F26: flat prior on N (truncated at 400,000, as verify2.py): posterior 2.5% / 50% / 97.5%
    and mean (numeric), and the closed-form untruncated mean (M-1)(n-1)/(k-2)."""
    w = np.exp(feller_logw - feller_logw.max())
    w = w / w.sum()
    c = np.cumsum(w)
    q = lambda a: int(Ns[np.searchsorted(c, a)])
    emit("A-F26.flat_q025", q(0.025))
    emit("A-F26.flat_median", q(0.5))
    emit("A-F26.flat_q975", q(0.975))
    emit("A-F26.flat_mean_numeric", float((Ns * w).sum()))
    emit_exact("A-F26.flat_mean_closed_form", F((FM - 1) * (FN_ - 1), FK - 2))


# ---------------------------------------------------------------------------------------------
# A-F27 and A-CH03: one habitat model for the whole video
# ---------------------------------------------------------------------------------------------
#
# Design, fixed before the first run (never re-run to reach a target):
#   * The lake is the unit square (x east, y north), so each cell of a 10 x 10 grid is exactly
#     1/100 of the lake, as the narration says.
#   * It holds N = 10,000 fish (Feller's lake, the true size used in chapters 07 and 09).
#   * Habitat suitability s(x, y) = 1 + 40 g_w + 20 g_d, with g = exp(-|p - c|^2 / (2 sd^2)):
#       a weed bed at c_w = (0.25, 0.75), sd 0.08 (north-west), and a deep hole at
#       c_d = (0.75, 0.25), sd 0.12 (south-east); open water has suitability 1. The 40 and 20 are
#       round "tens of times" contrasts for "fish crowd into weeds and the deep end, and avoid
#       open water"; they were not tuned. The fish density is proportional to s, i.e. U = -log s.
#   * Fish positions are i.i.d. from density s by rejection sampling with SplitMix64 seeded with
#     20261009 (a tiny, portable generator, so the Rust scene can draw the identical lake): draw
#     x, y, a uniform in [0, 1) as (u64 >> 11) * 2^-53 in that order; accept if a * 61 < s(x, y).
#   * The three dip spots are named by habitat, not by count: the cell containing the weed-bed
#     centre, the cell containing the lake's centre (0.5, 0.5) under half-open cells (open water),
#     and the cell containing the deep-hole centre. Estimate = 100 x (fish in the cell).
#   * Chapter 03 uses the same s: overdamped dynamics dX = -D grad U dt + c J grad U dt
#     + sqrt(2D) dW with U = -log s, J the +90 degree rotation, D = 0.01, c = 0.02, Euler-Maruyama
#     dt = 0.01, reflecting shores, numpy default_rng([20261009, 3]); start from the A-F27 lake
#     (already at the stationary density); the first catch is the 1,000 fish nearest the dock at
#     (0.5, 0.0) (south shore), marked and let go where they were caught; snapshots at
#     t = 0, 25, 100, 400 on 3 x 3 regions. Contrast run: identical but without the -D grad U
#     drift (divergence-free circulation + noise only), whose stationary density is uniform.
#   * Added AFTER the first run, as a diagnosis only (it changes no value above): the habitat run's
#     region counts drifted 10-20% from the model's stationary density. A third run with c = 0
#     (drift + noise, no circulation) separates the cause. Each run also reports region counts
#     time-averaged over t = 200..400 (every 1 time unit) and their largest deviation from the
#     model (from uniform for the contrast run).

LAKE = {
    "N": 10_000,
    "grid": 10,
    "base": 1.0,
    "weeds": (0.25, 0.75, 40.0, 0.08),  # centre x, centre y, amplitude, sd
    "deep": (0.75, 0.25, 20.0, 0.12),
    "s_max_bound": 61.0,  # 1 + 40 + 20 >= max s
    "seed": SEED,
}
SPOTS = [("weeds", (0.25, 0.75)), ("open", (0.5, 0.5)), ("deep", (0.75, 0.25))]
DYN = {
    "D": 0.01,
    "c": 0.02,
    "dt": 0.01,
    "times": (0, 25, 100, 400),
    "dock": (0.5, 0.0),
    "marked": 1000,
    "seed": [SEED, 3],
}
REGION_ORDER = "SW,S,SE,W,C,E,NW,N,NE"  # region id = rx + 3*ry, rx west->east, ry south->north

MASK64 = (1 << 64) - 1


class SplitMix64:
    """SplitMix64 (Steele, Lea & Flood 2014; Vigna's reference constants)."""

    def __init__(self, seed):
        self.state = seed & MASK64

    def next_u64(self):
        self.state = (self.state + 0x9E3779B97F4A7C15) & MASK64
        z = self.state
        z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & MASK64
        z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & MASK64
        return z ^ (z >> 31)

    def uniform(self):
        return (self.next_u64() >> 11) * (1.0 / 9007199254740992.0)


def habitat_scalar(x, y):
    wx, wy, wa, ws = LAKE["weeds"]
    dx, dy, da, ds = LAKE["deep"]
    return (LAKE["base"]
            + wa * math.exp(-((x - wx) ** 2 + (y - wy) ** 2) / (2 * ws * ws))
            + da * math.exp(-((x - dx) ** 2 + (y - dy) ** 2) / (2 * ds * ds)))


def habitat_np(x, y):
    """s and grad(log s) at arrays of points."""
    wx, wy, wa, ws = LAKE["weeds"]
    dx, dy, da, ds = LAKE["deep"]
    gw = wa * np.exp(-((x - wx) ** 2 + (y - wy) ** 2) / (2 * ws * ws))
    gd = da * np.exp(-((x - dx) ** 2 + (y - dy) ** 2) / (2 * ds * ds))
    s = LAKE["base"] + gw + gd
    gx = (-gw * (x - wx) / (ws * ws) - gd * (x - dx) / (ds * ds)) / s
    gy = (-gw * (y - wy) / (ws * ws) - gd * (y - dy) / (ds * ds)) / s
    return s, gx, gy


def draw_lake():
    """Draw the A-F27 lake: N fish i.i.d. from density proportional to s (rejection, SplitMix64)."""
    rng = SplitMix64(LAKE["seed"])
    xs, ys = [], []
    candidates = 0
    smax = LAKE["s_max_bound"]
    while len(xs) < LAKE["N"]:
        x = rng.uniform()
        y = rng.uniform()
        a = rng.uniform()
        candidates += 1
        if a * smax < habitat_scalar(x, y):
            xs.append(x)
            ys.append(y)
    return np.array(xs), np.array(ys), candidates


def model_integrals(res=1000):
    """Midpoint-rule integrals of s over the 10 x 10 cells and the 3 x 3 regions (res x res points)."""
    h = 1.0 / res
    g = (np.arange(res) + 0.5) * h
    X, Y = np.meshgrid(g, g, indexing="xy")  # rows = y, columns = x
    s, _, _ = habitat_np(X, Y)
    total = s.sum() * h * h
    cells = s.reshape(10, res // 10, 10, res // 10).sum(axis=(1, 3)) * h * h  # [y_cell, x_cell]
    ry = np.minimum((Y * 3).astype(int), 2)
    rx = np.minimum((X * 3).astype(int), 2)
    regions = np.bincount((rx + 3 * ry).ravel(), weights=s.ravel(), minlength=9) * h * h
    return total, cells, regions


def cell_of(x, y, grid):
    return min(int(x * grid), grid - 1), min(int(y * grid), grid - 1)


def af27_clustered_lake():
    """A-F27: dip-and-multiply estimates from the seeded clustered lake (design above)."""
    xs, ys, candidates = draw_lake()
    G = LAKE["grid"]
    ix = np.minimum((xs * G).astype(int), G - 1)
    iy = np.minimum((ys * G).astype(int), G - 1)
    counts = np.zeros((G, G), int)  # [iy, ix]
    np.add.at(counts, (iy, ix), 1)
    total, cells, _ = model_integrals()
    emit("A-F27.lake_fish", LAKE["N"])
    emit("A-F27.seed_splitmix64", LAKE["seed"])
    emit("A-F27.grid", f"{G}x{G}")
    emit("A-F27.cell_share_of_lake", F(1, G * G))
    emit("A-F27.rejection_candidates", candidates)
    emit("A-F27.check_sum_x", float(xs.sum()))
    emit("A-F27.check_sum_y", float(ys.sum()))
    estimates = {}
    for name, (px, py) in SPOTS:
        cx, cy = cell_of(px, py, G)
        c = int(counts[cy, cx])
        expected = LAKE["N"] * cells[cy, cx] / total
        estimates[name] = c * G * G
        emit(f"A-F27.spot.{name}.cell_ix_iy", f"{cx},{cy}")
        emit(f"A-F27.spot.{name}.fish_in_net", c)
        emit(f"A-F27.spot.{name}.estimate", c * G * G)
        emit(f"A-F27.spot.{name}.model_expected_fish", float(expected))
        emit(f"A-F27.spot.{name}.model_expected_estimate", float(expected * G * G))
    hi, lo = max(estimates.values()), min(estimates.values())
    emit("A-F27.spread_max_over_min", hi / lo)
    emit("A-F27.all_cells.min_fish", int(counts.min()))
    emit("A-F27.all_cells.max_fish", int(counts.max()))
    emit("A-F27.all_cells.median_estimate", float(np.median(counts) * G * G))
    emit("A-F27.all_cells.mean_estimate", float(counts.mean() * G * G))
    emit("A-F27.all_cells.share_of_dips_within_2x_of_truth",
         float(((counts * G * G >= LAKE["N"] / 2) & (counts * G * G <= LAKE["N"] * 2)).mean()))
    emit("A-F27.all_cells.dips_within_2x_of_truth",
         int(((counts * G * G >= LAKE["N"] / 2) & (counts * G * G <= LAKE["N"] * 2)).sum()))
    emit("A-F27.cell_counts_rows_south_to_north",
         ";".join(",".join(str(int(v)) for v in counts[j]) for j in range(G)))
    return xs, ys


def ch03_dynamics(xs0, ys0):
    """A-CH03: does the density stay clumpy while the marked fraction flattens? (design above)"""
    D, c, dt = DYN["D"], DYN["c"], DYN["dt"]
    times = DYN["times"]
    steps_at = {int(round(t / dt)): t for t in times}
    nsteps = max(steps_at)
    total, _, region_int = model_integrals()
    expected = LAKE["N"] * region_int / total
    emit("A-CH03.region_order", REGION_ORDER)
    emit("A-CH03.params", f"D={D};c={c};dt={dt};dock={DYN['dock']};marked={DYN['marked']};seed={DYN['seed']}")
    emit("A-CH03.model_expected_count", ",".join(f"{v:.1f}" for v in expected))
    emit("A-CH03.model_expected_max_over_min", float(expected.max() / expected.min()))
    d = np.hypot(xs0 - DYN["dock"][0], ys0 - DYN["dock"][1])
    marked = np.zeros(xs0.size, bool)
    marked[np.argsort(d, kind="stable")[: DYN["marked"]]] = True
    emit("A-CH03.marked_radius_from_dock", float(np.sort(d)[DYN["marked"] - 1]))

    def regions(x, y):
        rx = np.minimum((x * 3).astype(int), 2)
        ry = np.minimum((y * 3).astype(int), 2)
        rid = rx + 3 * ry
        cnt = np.bincount(rid, minlength=9)
        mk = np.bincount(rid, weights=marked.astype(float), minlength=9).astype(int)
        return cnt, mk

    every = int(round(1 / dt))
    avg_from = int(round(200 / dt))

    def run(label, with_drift, with_circ):
        rng = np.random.default_rng(DYN["seed"])
        x, y = xs0.copy(), ys0.copy()
        amp = sqrt(2 * D * dt)
        out = {}
        acc = np.zeros(9)
        nacc = 0
        for step in range(nsteps + 1):
            if step in steps_at:
                out[steps_at[step]] = regions(x, y)
            if step >= avg_from and step % every == 0:
                acc += regions(x, y)[0]
                nacc += 1
            if step == nsteps:
                break
            _, gx, gy = habitat_np(x, y)
            # grad U = -grad log s ;  -D grad U = D grad log s ;  c J grad U = c (gy, -gx)
            vx = np.zeros_like(x)
            vy = np.zeros_like(y)
            if with_circ:
                vx = vx + c * gy
                vy = vy - c * gx
            if with_drift:
                vx = vx + D * gx
                vy = vy + D * gy
            xi = rng.standard_normal((2, x.size))
            x = x + vx * dt + amp * xi[0]
            y = y + vy * dt + amp * xi[1]
            x = 1.0 - np.abs(1.0 - np.abs(x))  # reflect at both shores
            y = 1.0 - np.abs(1.0 - np.abs(y))
        for t in times:
            cnt, mk = out[t]
            pre = f"A-CH03.{label}.t{t}"
            emit(f"{pre}.count", ",".join(str(int(v)) for v in cnt))
            emit(f"{pre}.count_max_over_min", float(cnt.max() / cnt.min()))
            if label == "habitat":
                frac = mk / cnt
                emit(f"{pre}.marked", ",".join(str(int(v)) for v in mk))
                emit(f"{pre}.marked_fraction", ",".join(f"{v:.4f}" for v in frac))
                emit(f"{pre}.marked_fraction_min", float(frac.min()))
                emit(f"{pre}.marked_fraction_max", float(frac.max()))
                emit(f"{pre}.max_abs_count_change_vs_t0_pct",
                     float(100 * np.max(np.abs(cnt - out[0][0]) / out[0][0])))
                emit(f"{pre}.max_abs_count_dev_vs_model_pct",
                     float(100 * np.max(np.abs(cnt - expected) / expected)))
                # binomial noise scale of a region's marked fraction at 10%
                emit(f"{pre}.binomial_sd_of_fraction_in_smallest_region", sqrt(0.1 * 0.9 / cnt.min()))
        avg = acc / nacc
        ref = expected if with_drift else np.full(9, LAKE["N"] / 9)
        pre = f"A-CH03.{label}.timeavg_t200_400"
        emit(f"{pre}.snapshots", nacc)
        emit(f"{pre}.count", ",".join(f"{v:.1f}" for v in avg))
        emit(f"{pre}.count_max_over_min", float(avg.max() / avg.min()))
        emit(f"{pre}.max_abs_dev_vs_{'model' if with_drift else 'uniform'}_pct",
             float(100 * np.max(np.abs(avg - ref) / ref)))
        return out, x, y  # x, y: every fish's position at the last snapshot (t = 400)

    out_h, x400, y400 = run("habitat", True, True)
    run("contrast_no_drift", False, True)
    run("diag_no_circulation", True, False)
    ch03_insets_and_trays(x400, y400, marked)
    ch03_r2_corners_and_nets(out_h[400], x400, y400, marked)


# Claim walk (G3), 03_5 and 03_6: small catches and zoom insets in the habitat run's t = 400 frame
# (the last snapshot above; all 10,000 fish, 1:1). The side 0.04 is the claim walk's example
# (G3, 2026-10-11); the counts are the narration's. So the insets are SELECTED squares, and the
# keys also say how common such squares are.
#   * Insets (03_6): squares of side 0.04 made of 0.01 bins (a fish is in bin floor(100 x),
#     floor(100 y)), every position on the 0.01 grid inside the lake. At side 0.04 the
#     lake-average square holds 16 fish, so 50 is crowded (3.1x) and 10 is sparse (0.6x). The
#     crowded inset is the square with exactly 50 fish and 5 marked whose centre is nearest the
#     weed-bed centre; the sparse one has exactly 10 fish and 1 marked, nearest the lake centre
#     (A-F27's open-water spot); ties go to the more southern, then the more western square.
#   * Trays (03_5): a net of m fish at a point catches the m fish nearest it. One net of 10 and one
#     of 20 at each of the nine 3 x 3 region centres (the regions A-CH03 reports, REGION_ORDER),
#     plus the exact chance that a random net of 10 (20) from the lake holds exactly 1 (2) marked.
INSET = {"bins": 100, "side_bins": 4,
         "crowded": (50, 5, (0.25, 0.75)), "sparse": (10, 1, (0.5, 0.5))}
TRAY_SIZES = (10, 20)


def ch03_insets_and_trays(x, y, marked):
    """A-CH03.inset.* and A-CH03.trays.* from the t = 400 frame (rules above)."""
    G, w = INSET["bins"], INSET["side_bins"]
    N = x.size
    ix = np.minimum((x * G).astype(int), G - 1)
    iy = np.minimum((y * G).astype(int), G - 1)
    cnt = np.zeros((G, G), np.int64)  # [iy, ix]
    mk = np.zeros((G, G), np.int64)
    np.add.at(cnt, (iy, ix), 1)
    np.add.at(mk, (iy, ix), marked.astype(np.int64))

    def squares(a):
        """[j, i] = sum of a over the w x w bins with south-west bin (i, j)."""
        P = np.zeros((G + 1, G + 1), np.int64)
        P[1:, 1:] = a.cumsum(0).cumsum(1)
        return P[w:, w:] - P[:-w, w:] - P[w:, :-w] + P[:-w, :-w]

    C, K = squares(cnt), squares(mk)
    emit("A-CH03.inset.side", F(w, G))
    emit("A-CH03.inset.squares_scanned", int(C.size))
    emit("A-CH03.inset.lake_average_fish_per_square", F(N * w * w, G * G))
    recount_ok = True
    for name in ("crowded", "sparse"):
        nf, nm, (px, py) = INSET[name]
        same = C == nf
        match = same & (K == nm)
        js, is_ = np.nonzero(match)
        # distances in half-bins (integers): square centre (2i + w, 2j + w), target (2G px, 2G py)
        tx, ty = round(2 * G * px), round(2 * G * py)
        d2 = (2 * is_ + w - tx) ** 2 + (2 * js + w - ty) ** 2
        t = np.lexsort((is_, js, d2))[0]
        i, j = int(is_[t]), int(js[t])
        inside = (ix >= i) & (ix < i + w) & (iy >= j) & (iy < j + w)
        recount_ok &= int(inside.sum()) == int(C[j, i]) and int((inside & marked).sum()) == int(K[j, i])
        pre = f"A-CH03.inset.{name}"
        emit(f"{pre}.square_x0_x1_y0_y1", f"{i / G:.2f},{(i + w) / G:.2f},{j / G:.2f},{(j + w) / G:.2f}")
        emit(f"{pre}.fish", int(C[j, i]))
        emit(f"{pre}.marked", int(K[j, i]))
        emit(f"{pre}.marked_share", F(int(K[j, i]), int(C[j, i])))
        emit(f"{pre}.density_vs_lake_average", F(int(C[j, i]) * G * G, N * w * w))
        emit(f"{pre}.squares_with_{nf}_fish", int(same.sum()))
        emit(f"{pre}.of_those_with_{nm}_marked", int(match.sum()))
    emit("A-CH03.inset.direct_recount_agrees", recount_ok)
    centres = [((2 * rx + 1) / 6, (2 * ry + 1) / 6) for ry in range(3) for rx in range(3)]
    for m in TRAY_SIZES:
        got = []
        for px, py in centres:
            nearest = np.argsort(np.hypot(x - px, y - py), kind="stable")[:m]
            got.append(int(marked[nearest].sum()))
        emit(f"A-CH03.trays.net{m}.marked_at_region_centres", ",".join(map(str, got)))
        emit(f"A-CH03.trays.net{m}.marked_min_max", f"{min(got)},{max(got)}")
    M = int(marked.sum())
    for m in TRAY_SIZES:
        emit(f"A-CH03.trays.P_random_net{m}_has_exactly_{m // 10}_marked", L(N, M, m, m // 10))


# R2 (2026-10-11; Ruth R4, Dev dev-06): the 03_6 insets above were selected to match the spoken
# counts and the 03_5 trays were drawn, not caught. Their replacements, both from the same t = 400
# frame and picked by rules that never look at the marks:
#   * corners (03_6): the most and the least crowded of the nine 3 x 3 regions (by fish count;
#     both happen to be corner regions), with every fish they hold.
#   * nets (03_5): a net of 1,000 fish (the story's second catch, n) at each of the nine region
#     centres, by the trays' rule (the m fish nearest the point). Nets of 10, 20, 50, 100, 200 and
#     500 were looked at in a scratch run first: below a few hundred fish a net's marked share
#     wobbles far from 10% (the 10- and 20-fish keys above), which is chapter 09's point, not 03's.
R2_NET = 1000


def ch03_r2_corners_and_nets(regions_t400, x, y, marked):
    """A-CH03.corners.* and A-CH03.trays.net1000.* from the t = 400 frame (rules above)."""
    cnt, mk = regions_t400
    names = REGION_ORDER.split(",")
    for tag, r in (("most_crowded", int(np.argmax(cnt))), ("least_crowded", int(np.argmin(cnt)))):
        pre = f"A-CH03.corners.{tag}"
        emit(f"{pre}.region", names[r])
        emit(f"{pre}.fish", int(cnt[r]))
        emit(f"{pre}.marked", int(mk[r]))
        emit(f"{pre}.marked_share", F(int(mk[r]), int(cnt[r])))
    emit("A-CH03.corners.unique_extremes", int((cnt == cnt.max()).sum()) == 1 and int((cnt == cnt.min()).sum()) == 1)
    centres = [((2 * rx + 1) / 6, (2 * ry + 1) / 6) for ry in range(3) for rx in range(3)]
    got = []
    for px, py in centres:
        nearest = np.argsort(np.hypot(x - px, y - py), kind="stable")[:R2_NET]
        got.append(int(marked[nearest].sum()))
    pre = f"A-CH03.trays.net{R2_NET}"
    emit(f"{pre}.marked_at_region_centres", ",".join(map(str, got)))
    emit(f"{pre}.marked_min_max", f"{min(got)},{max(got)}")
    emit(f"{pre}.share_min_max", f"{min(got) / R2_NET},{max(got) / R2_NET}")


# Chapter 03 on screen (2026-10-11): the scene runs the mixing itself, so the run it shows must be
# reproducible bit for bit in Rust (model.rs). Same model and steps as the habitat run above (D, c,
# dt = 0.01 to t = 400, reflecting shores, the A-F27 start), with a portable noise source in place
# of numpy's generator; the numpy runs above stay as the research record.
#   * SplitMix64 as a counter stream: draw k (k = 0, 1, ...) is mix(seed + (k + 1) GAMMA) with seed
#     SEED + 3; uniform = (u64 >> 11) 2^-53.
#   * normal = (((u1 + u2) + u3) + u4 - 2) sqrt(3) (Irwin-Hall: mean 0, variance 1), sums of exactly
#     representable uniforms, so Python and Rust compute the same bits.
#   * step s uses draws 8 N s ... 8 N s + 8 N - 1: fish i's x normal from draws 8 N s + 8 i + 0..3,
#     its y normal from 8 N s + 8 i + 4..7.
#   * marked: the 1,000 fish nearest the dock by squared distance (exact), ties to the lower index.
#   * drift and update in exactly the order written in port_step (the Rust port repeats it);
#     floats checked as raw bits, counts as integers.
PORT = {"seed": SEED + 3, "gamma": 0x9E3779B97F4A7C15}
SQRT3 = 1.7320508075688772  # float(sqrt(3)), spelled so both languages use the same bits


def sm64_mix(z):
    z = (z ^ (z >> np.uint64(30))) * np.uint64(0xBF58476D1CE4E5B9)
    z = (z ^ (z >> np.uint64(27))) * np.uint64(0x94D049BB133111EB)
    return z ^ (z >> np.uint64(31))


def port_normals(step, n):
    """Step `step`'s noise: (x normals, y normals) for fish 0..n-1."""
    k = np.arange(8 * n, dtype=np.uint64) + np.uint64(8 * n * step)
    state = np.uint64(PORT["seed"]) + (k + np.uint64(1)) * np.uint64(PORT["gamma"])
    u = (sm64_mix(state) >> np.uint64(11)).astype(np.float64) * (1.0 / 9007199254740992.0)
    u = u.reshape(n, 2, 4)
    z = ((((u[..., 0] + u[..., 1]) + u[..., 2]) + u[..., 3]) - 2.0) * SQRT3
    return z[:, 0], z[:, 1]


def port_step(x, y, step):
    D, c, dt = DYN["D"], DYN["c"], DYN["dt"]
    _, gx, gy = habitat_np(x, y)
    vx = c * gy + D * gx
    vy = -(c * gx) + D * gy
    zx, zy = port_normals(step, x.size)
    amp = sqrt(2 * D * dt)
    x = (x + vx * dt) + amp * zx
    y = (y + vy * dt) + amp * zy
    return 1.0 - np.abs(1.0 - np.abs(x)), 1.0 - np.abs(1.0 - np.abs(y))


def bits(v):
    return int(np.float64(v).view(np.uint64))


def ch03_portable(xs0, ys0):
    """A-CH03.portable.*: the run chapter 03 draws (rules above)."""
    dt = DYN["dt"]
    steps_at = {int(round(t / dt)): t for t in DYN["times"]}
    nsteps = max(steps_at)
    n = xs0.size
    dx, dy = xs0 - DYN["dock"][0], ys0 - DYN["dock"][1]
    d2 = dx * dx + dy * dy
    marked = np.zeros(n, bool)
    marked[np.lexsort((np.arange(n), d2))[: DYN["marked"]]] = True
    zx, zy = port_normals(0, n)
    emit("A-CH03.portable.params", f"seed={PORT['seed']};normal=irwin_hall4_sqrt3;dt={dt};steps={nsteps}")
    emit("A-CH03.portable.normal_bits_step0_fish0_x_y", f"{bits(zx[0])},{bits(zy[0])}")
    emit("A-CH03.portable.normal_bits_step0_fish1_x_y", f"{bits(zx[1])},{bits(zy[1])}")
    x, y = xs0.copy(), ys0.copy()
    names = REGION_ORDER.split(",")
    for step in range(nsteps + 1):
        if step == 1:
            emit("A-CH03.portable.position_bits_step1_fish0_x_y", f"{bits(x[0])},{bits(y[0])}")
        if step in steps_at:
            t = steps_at[step]
            rx = np.minimum((x * 3).astype(int), 2)
            ry = np.minimum((y * 3).astype(int), 2)
            rid = rx + 3 * ry
            cnt = np.bincount(rid, minlength=9)
            mk = np.bincount(rid[marked], minlength=9)
            emit(f"A-CH03.portable.t{t}.count", ",".join(str(int(v)) for v in cnt))
            emit(f"A-CH03.portable.t{t}.marked", ",".join(str(int(v)) for v in mk))
            emit(f"A-CH03.portable.t{t}.marked_percent_1dp",
                 ",".join(f"{100 * int(m) / int(c):.1f}" for m, c in zip(mk, cnt)))
        if step == nsteps:
            break
        x, y = port_step(x, y, step)
    xor = 0
    for v in x:
        xor ^= bits(v)
    emit("A-CH03.portable.t400.position_bits_xor_x", xor)
    for tag, r in (("most_crowded", int(np.argmax(cnt))), ("least_crowded", int(np.argmin(cnt)))):
        pre = f"A-CH03.portable.corners.{tag}"
        emit(f"{pre}.region", names[r])
        emit(f"{pre}.fish", int(cnt[r]))
        emit(f"{pre}.marked", int(mk[r]))
    emit("A-CH03.portable.corners.unique_extremes",
         int((cnt == cnt.max()).sum()) == 1 and int((cnt == cnt.min()).sum()) == 1)
    centres = [((2 * rx + 1) / 6, (2 * ry + 1) / 6) for ry in range(3) for rx in range(3)]
    got = []
    for px, py in centres:
        ex, ey = x - px, y - py
        near = np.lexsort((np.arange(n), ex * ex + ey * ey))[:R2_NET]
        got.append(int(marked[near].sum()))
    emit(f"A-CH03.portable.nets{R2_NET}.marked_at_region_centres", ",".join(map(str, got)))
    emit(f"A-CH03.portable.nets{R2_NET}.marked_min_max", f"{min(got)},{max(got)}")


# ---------------------------------------------------------------------------------------------
# A-F28 .. A-F32
# ---------------------------------------------------------------------------------------------

def af28_split_symmetry():
    """A-F28: marking 2,000 and catching 500 vs marking 500 and catching 2,000 (N = 10,000): the
    pmf of K is identical term by term (Feller 6.2, exact integers), so the exact SD of Mn/K given
    K > 0 is identical; computed both ways by exact pushforward."""
    N = FELLER_N_TRUE
    same = all(F(num(N, 2000, 500, j), comb(N, 500)) == F(num(N, 500, 2000, j), comb(N, 2000)) for j in range(0, 501))
    a = plain_estimate_law(N, 2000, 500)
    b = plain_estimate_law(N, 500, 2000)
    emit("A-F28.pmf_identical_exact", same)
    emit("A-F28.sd_M2000_n500", a["sd"])
    emit("A-F28.sd_M500_n2000", b["sd"])
    emit("A-F28.sd_equal_exact", a["var"] == b["var"])
    emit("A-F28.cv_symmetric_approx", sqrt((1 - 2000 / N) * (1 - 500 / N) / (2000 * 500 / N)))


def af29_table_identity():
    """A-F29: Mn - kN = ab - k u. P = (Mn - kN) - ((M-k)(n-k) - k(N-M-n+k)) has degree <= 2 in each
    variable, so vanishing on {0,1,2,3}^4 proves P == 0. Then u_hat = floor(ab/k) for Feller and the toy."""
    rng4 = range(4)
    ok = all((M * n - k * N) == ((M - k) * (n - k) - k * (N - M - n + k))
             for M in rng4 for n in rng4 for k in rng4 for N in rng4)
    emit("A-F29.identity_proved_on_grid", ok)
    a, b, k = FM - FK, FN_ - FK, FK
    emit("A-F29.feller_a", a)
    emit("A-F29.feller_b", b)
    emit("A-F29.feller_u_hat", (a * b) // k)
    emit("A-F29.feller_ab_over_k_whole", (a * b) % k == 0)
    emit("A-F29.feller_N_hat", a + b + k + (a * b) // k)
    # the balance the voice says (08_3): k·u = a·b = 810,000 (R1 Maya m5)
    emit("A-F29.feller_ab", a * b)
    emit("A-F29.feller_k_times_u_hat", k * ((a * b) // k))
    a, b, k = 7, 7, 3
    emit_exact("A-F29.toy_ab_over_k", F(a * b, k))
    emit("A-F29.toy_u_hat", (a * b) // k)
    emit("A-F29.toy_N_hat", a + b + k + (a * b) // k)


def af30_coin():
    """A-F30: P(exactly 50 heads in 100 fair flips) = C(100,50)/2^100, and that 50 is the mode."""
    p = F(comb(100, 50), 2 ** 100)
    emit("A-F30.P50_of_100", p)
    emit("A-F30.P50_of_100_pct_2dp", round(float(p) * 100, 2))
    emit("A-F30.50_is_mode", all(comb(100, 50) >= comb(100, j) for j in range(101)))


def af31_bending():
    """A-F31: 10^6/90 and 10^6/110 (the hyperbola bends: 10 too few pushes harder than 10 too many)."""
    for k in (90, 110):
        v = F(10 ** 6, k)
        emit(f"A-F31.Mn_over_{k}", v)
        emit(f"A-F31.Mn_over_{k}_rounded", round(v))
    emit("A-F31.excess_above_10000", round(F(10 ** 6, 90)) - 10000)
    emit("A-F31.shortfall_below_10000", 10000 - round(F(10 ** 6, 110)))


def af32_float64(p1900):
    """A-F32: P_1900(K = 100) is below the smallest normal float64 (and the smallest subnormal), so
    float64 underflows to 0.0: float() of the exact Fraction and exp() of the lgamma log both give 0.0."""
    mant, ex = sci(p1900)
    emit("A-F32.smallest_normal_float64", sys.float_info.min)
    emit("A-F32.smallest_subnormal_float64", math.ulp(0.0))
    emit("A-F32.P1900_sci", f"{mant:.4f}e{ex}")
    emit("A-F32.P1900_below_smallest_normal", p1900 < F(sys.float_info.min))
    emit("A-F32.P1900_below_smallest_subnormal", p1900 < F(math.ulp(0.0)))
    emit("A-F32.float_of_exact_underflows_to_zero", float(p1900) == 0.0)
    emit("A-F32.exp_of_lgamma_underflows_to_zero",
         exp(2 * lgamma(1001) - lgamma(101) - lgamma(1901)) == 0.0)
    # Claim walk (G3), 11_2 picture: the dashed marker at the 324th zero ("smallest positive
    # ≈ 4.9 × 10⁻³²⁴"). The smallest subnormal is 2^-1074 exactly; its scientific form; the decimal
    # place of its first nonzero digit. Then the scrolling zeros themselves: P1900's own mantissa
    # (5.35...) over a one followed by Z zeros, converted to float64 (Fraction -> float rounds
    # correctly): the last Z that stays nonzero and the first that rounds to 0.0.
    tiny = F(1, 2 ** 1074)
    emit("A-F32.smallest_subnormal_is_2_pow_minus_1074", F(math.ulp(0.0)) == tiny)
    t_mant, t_ex = sci(tiny)
    emit("A-F32.smallest_subnormal_mantissa", t_mant)
    emit("A-F32.smallest_subnormal_exponent", t_ex)
    emit("A-F32.smallest_subnormal_first_nonzero_decimal_place",
         next(d for d in itertools.count(1) if tiny * 10 ** d >= 1))
    m = p1900 * F(10) ** (-ex)
    zero_at = [Z for Z in range(1, 431) if float(m / F(10) ** Z) == 0.0]
    emit("A-F32.zeros.mantissa", float(m))
    emit("A-F32.zeros.last_Z_float64_nonzero", min(zero_at) - 1)
    emit("A-F32.zeros.first_Z_float64_zero", min(zero_at))
    emit("A-F32.zeros.all_later_Z_zero", zero_at == list(range(min(zero_at), 431)))


# ---------------------------------------------------------------------------------------------
# Claim walk (gate G3, 2026-10-11): numbers the storyboard shows or says that no ledger row covered
# ---------------------------------------------------------------------------------------------

def ch11_one_marked_fish():
    """A-CH11 (11_3 "we'd expect to catch about one marked fish"): E[K] = nM/N at a lake of
    1,000,000 with Feller's catches. The formula is first checked exactly against the
    hypergeometric pmf for every 1 <= M, n <= N <= 25."""
    ok = all(sum(j * num(N, M, n, j) for j in range(min(M, n) + 1)) * N == n * M * comb(N, n)
             for N in range(1, 26) for M in range(1, N + 1) for n in range(1, N + 1))
    emit("A-CH11.mean_formula_exact_N_le_25", ok)
    emit_exact("A-CH11.expected_k_N1000000", F(FM * FN_, 1_000_000))


def ch12_ring_toss(sims):
    """A-CH12.ring_toss (12_2): the rings are the exact 95% ranges (exact_ci) of A-F19's seeded
    experiments (numpy default_rng(20261009), first draw: the 09_6 code panel's own k), in run
    order; the peg is the true lake, 10,000. A range rings 10,000 exactly when both one-sided tests
    at N = 10,000 keep the catch, P(K <= k) > 1/40 and P(K >= k) > 1/40 (exact integers), so the
    ringing catches form an interval [k_lo, k_hi]; exact_ci itself is run at its four edges
    (k_lo - 1, k_lo, k_hi, k_hi + 1) as a cross-check. Exact coverage = P(k_lo <= K <= k_hi)."""
    N, M, n = FELLER_N_TRUE, FM, FN_
    T = comb(N, n)
    nums = [num(N, M, n, j) for j in range(n + 1)]
    cum = list(itertools.accumulate(nums))  # cum[j] = C(N,n) * P(K <= j)
    kept = [j for j in range(n + 1)
            if cum[j] * 40 > T and (T - (cum[j - 1] if j else 0)) * 40 > T]
    k_lo, k_hi = kept[0], kept[-1]
    edges_ok = True
    for j, rings in ((k_lo - 1, False), (k_lo, True), (k_hi, True), (k_hi + 1, False)):
        lo, hi, _ = exact_ci(M, n, j)
        edges_ok &= (lo <= N <= hi) == rings
    ks = sims[(N, M, n)]  # verify2.py's research stream; from R2 on the scene uses ch12_ring_toss_panel
    miss = (ks < k_lo) | (ks > k_hi)
    cov = F(sum(nums[k_lo:k_hi + 1]), T)
    pre = "A-CH12.ring_toss"
    emit(f"{pre}.ringing_k", f"{k_lo},{k_hi}")
    emit(f"{pre}.ringing_k_is_an_interval", kept == list(range(k_lo, k_hi + 1)))
    emit(f"{pre}.edges_agree_with_exact_ci", edges_ok)
    emit(f"{pre}.first100_ringed", int(100 - miss[:100].sum()))
    emit(f"{pre}.first100_missed", int(miss[:100].sum()))
    emit(f"{pre}.first100_missed_runs_1based", ",".join(str(int(r) + 1) for r in np.flatnonzero(miss[:100])))
    emit(f"{pre}.all_runs", int(ks.size))
    emit(f"{pre}.all_missed", int(miss.sum()))
    emit(f"{pre}.exact_coverage_N10000", cov)
    emit(f"{pre}.exact_coverage_N10000_pct_2dp", round(float(cov) * 100, 2))
    emit(f"{pre}.expected_missed", float((1 - cov) * int(ks.size)))
    return k_lo, k_hi, cov


def ch12_slider(feller_ci):
    """A-CH12.slider (12_5): the lake fixed at N = 10,000; for an expected caught-twice count k,
    M = n = sqrt(10,000 k) (whole for k = 25, 100, 400) and the catch is that expectation. The exact
    95% range (exact_ci; k = 100 is Feller's, reused from A-F13), its width and its ends in percent
    of the lake, and the width ratio for each fourfold step in k. At k = 10,000 both catches are
    the whole lake: P(K <= k) = 1 at N = 10,000, and P(K >= k) at N = 10,001 is computed (it only
    falls with N), which gives the range."""
    N = FELLER_N_TRUE
    width = {}
    for k in (25, 100, 400):
        m = math.isqrt(N * k)
        lo, hi = feller_ci if (m, k) == (FM, FK) else exact_ci(m, m, k)[:2]
        width[k] = hi - lo
        pre = f"A-CH12.slider.k{k}"
        emit(f"{pre}.M_eq_n", m)
        emit(f"{pre}.M_eq_n_is_whole", m * m == N * k)
        emit(f"{pre}.ci95", f"{lo},{hi}")
        emit(f"{pre}.width", hi - lo)
        emit(f"{pre}.lo_pct", F(100 * (lo - N), N))
        emit(f"{pre}.hi_pct", F(100 * (hi - N), N))
    emit("A-CH12.slider.width_ratio_k25_to_k100", F(width[100], width[25]))
    emit("A-CH12.slider.width_ratio_k100_to_k400", F(width[400], width[100]))
    m = k = N
    alpha = F(1, 40)
    p_le = F(cdf_le_num(N, m, m, k), comb(N, m))
    p_ge = {Nx: 1 - F(cdf_le_num(Nx, m, m, k - 1), comb(Nx, m)) for Nx in (N, N + 1)}
    lo = m + m - k if p_le > alpha else None
    hi = N if p_ge[N] > alpha >= p_ge[N + 1] else None
    emit("A-CH12.slider.whole_lake.P_le_at_N10000", p_le)
    emit_exact("A-CH12.slider.whole_lake.P_ge_at_N10001", p_ge[N + 1])
    emit("A-CH12.slider.whole_lake.ci95", f"{lo},{hi}")


# 14_6: two reviewers. Models fixed by the claim walk (G3): 'skill' = 60 alike bugs, reviewer A
# finds each with chance 0.5 and B with 0.4, independently (expected 30, 24 and 12 caught twice:
# 14_4's example). 'difficulty' = 30 easy bugs (A 0.8, B 0.7) and 30 hard (A 0.2, B 0.1): the same
# expected 30 and 24. 20,000 runs each; numpy default_rng([20261009, 14]), skill first.
REVIEW = {"bugs": 60, "runs": 20_000, "skill": (0.5, 0.4),
          "easy": (0.8, 0.7), "hard": (0.2, 0.1), "seed": [SEED, 14]}


def ch14_reviewers():
    """A-CH14.reviewers: exact expected counts and the plain estimate a*b/k from those
    expectations, then the seeded runs' median and mean of a*b/k over runs with k > 0."""
    B, R = REVIEW["bugs"], REVIEW["runs"]
    half = B // 2
    models = {
        "skill": ([REVIEW["skill"][0]] * B, [REVIEW["skill"][1]] * B),
        "difficulty": ([REVIEW["easy"][0]] * half + [REVIEW["hard"][0]] * (B - half),
                       [REVIEW["easy"][1]] * half + [REVIEW["hard"][1]] * (B - half)),
    }
    rng = np.random.default_rng(REVIEW["seed"])
    emit("A-CH14.reviewers.params",
         f"bugs={B};runs={R};skill={REVIEW['skill']};easy={REVIEW['easy']};hard={REVIEW['hard']};"
         f"seed={REVIEW['seed']}")
    for name, (pa, pb) in models.items():
        fa = [F(str(p)) for p in pa]
        fb = [F(str(p)) for p in pb]
        ea, eb, ek = sum(fa), sum(fb), sum(x * y for x, y in zip(fa, fb))
        a = rng.random((R, B)) < np.array(pa)
        b = rng.random((R, B)) < np.array(pb)
        na, nb, k = a.sum(axis=1), b.sum(axis=1), (a & b).sum(axis=1)
        pos = k > 0
        est = na[pos] * nb[pos] / k[pos]
        pre = f"A-CH14.reviewers.{name}"
        emit(f"{pre}.true_bugs", B)
        emit_exact(f"{pre}.expected_a", ea)
        emit_exact(f"{pre}.expected_b", eb)
        emit_exact(f"{pre}.expected_caught_twice", ek)
        emit_exact(f"{pre}.estimate_from_expected_counts", ea * eb / ek)
        emit(f"{pre}.runs_k0", int((~pos).sum()))
        emit(f"{pre}.sim_median_estimate", float(np.median(est)))
        emit(f"{pre}.sim_mean_estimate", float(est.mean()))
        emit(f"{pre}.sim_share_below_true", float((est < B).mean()))
        # R2 (Dev dev-05, Ruth R6): where the careful range (exact 95%, exact_ci on each run's own
        # a, b, k) lands against the true count of bugs. Reads the runs above; draws nothing new.
        ranges = {}
        where = []
        for x, y, z in zip(na[pos].tolist(), nb[pos].tolist(), k[pos].tolist()):
            if (x, y, z) not in ranges:
                ranges[(x, y, z)] = exact_ci(x, y, z)[:2]
            lo, hi = ranges[(x, y, z)]
            where.append(0 if lo <= B <= hi else -1 if hi < B else 1)
        where = np.array(where)
        emit(f"{pre}.ci95_contains_true_share", float((where == 0).mean()))
        emit(f"{pre}.ci95_wholly_below_true_share", float((where == -1).mean()))
        emit(f"{pre}.ci95_wholly_above_true_share", float((where == 1).mean()))


# ---------------------------------------------------------------------------------------------
# R2 review round (2026-10-11): numbers the R2 fixes show or say that no key covered
# ---------------------------------------------------------------------------------------------

def emit_q(key, fr):
    """Emit a Fraction as an integer when it is whole, else as a float."""
    emit(key, fr.numerator if fr.denominator == 1 else fr)


TOY = (10, 10, 3)  # the mystery lake of chapter 05: M, n, k


def r2_small_lake_keys():
    """07_2 (Maya M3, Dev dev-04): Feller's ratio at the mystery lake's peak, factor by factor in
    table words: (N - M) unmarked fish, (N - n) fish the second net missed, N the lake size,
    u = N - M - n + k the dark cell; the product of the factors is checked against the exact
    likelihood ratio L(N)/L(N-1). 05_7 (Maya M2): the scores' sum, 50/3, in percent, the unit
    05_4 speaks. 03_7 (Maya M4): the second catch's share in each column of Feller's table at
    N = 10,000. 10_3 (Ruth R17, Maya M18, Dev dev-03): the small study's expected caught-twice
    count, and Chapman's bias bound N e^(-Mn/N) as a percent where 4 are expected."""
    M, n, k = TOY
    for N in (33, 34):
        unmarked, missed, u = N - M, N - n, N - M - n + k
        emit(f"A-F6.N{N}.unmarked_N_minus_M", unmarked)
        emit(f"A-F6.N{N}.missed_by_second_N_minus_n", missed)
        emit(f"A-F6.N{N}.dark_cell_u", u)
        emit(f"A-F6.N{N}.factors_equal_exact_ratio", F(unmarked * missed, N * u) == L(N, M, n, k) / L(N - 1, M, n, k))
    area_pct = F(M * n, k * (k - 1)) * 100
    emit_exact("A-F7.toy_area_pct", area_pct)
    emit("A-F7.toy_area_pct_rounded", round(area_pct))
    b = FN_ - FK
    emit("A-F29.unmarked_column_N10000", FELLER_N_TRUE - FM)
    left, right = F(FK, FM), F(b, FELLER_N_TRUE - FM)
    emit("A-F29.green_share_marked_column_N10000", left)
    emit("A-F29.green_share_unmarked_column_N10000", right)
    emit("A-F29.green_shares_equal_N10000", left == right)
    emit_exact("A-F21.expected_k", F(50 * 50, 1000))
    emit("A-F18.bound_pct_at_expected_k4", 100 * exp(-4))
    emit("A-F18.bound_at_expected_k4_under_2pct", 100 * exp(-4) < 2)
    # the bound itself (10_3 lower-third, R-08): Chapman is never low by more than N e^(-Mn/N).
    # Proof: |bias| = (N+1) prod_{j=0..n} (N-M-j)/(N+1-j) <= (N+1)(1 - M/N)^(n+1) <= N e^(-Mn/N).
    # Checked here with the exact bias (closed form, A-F18) on every 1 <= M, n <= N <= 40.
    cases = held = 0
    for N in range(1, 41):
        for M in range(1, N + 1):
            for n in range(1, N + 1):
                cases += 1
                held += float(-chapman_bias_closed(N, M, n)) <= N * exp(-M * n / N) * (1 + 1e-12)
    emit("A-F18.bound_cases_N_le_40", cases)
    emit("A-F18.bound_holds_N_le_40", held == cases)


def ch06_row_mode_rule():
    """A-CH06 (05/06 notes, Ruth R-27, routed to the description at R2): every fixed-size row
    P_N(K = k) peaks at floor((n+1)(M+1)/(N+2)), tied with the count one below when that quotient
    is whole. Brute force over every 1 <= N <= 40 and 0 <= M, n <= N (exact integers)."""
    cases = bad = 0
    for N in range(1, 41):
        for M in range(N + 1):
            for n in range(N + 1):
                ps = [num(N, M, n, j) for j in range(min(M, n) + 1)]
                top = max(ps)
                modes = [j for j, v in enumerate(ps) if v == top]
                q = F((n + 1) * (M + 1), N + 2)
                fl = q.numerator // q.denominator
                pred = [j for j in ([fl - 1, fl] if q.denominator == 1 else [fl])
                        if 0 <= j <= min(M, n) and ps[j] > 0]
                cases += 1
                bad += modes != pred
    emit("A-CH06.row_mode_rule.cases_N_le_40", cases)
    emit("A-CH06.row_mode_rule.failures", bad)


# 09_6 (R2, Dev dev-01, Ruth R20, Maya M17): the code panel nets fish instead of sampling the
# hypergeometric law, so the check against the exact law tests chapter 04's claim; it names the
# exact line's index j, not k; and it prints the check. The panel's lines, run verbatim here
# (the lines in the order the panel shows them):
#     import numpy as np
#     from math import comb
#     rng = np.random.default_rng(20261009)
#     M = n = 1_000; N = 10_000
#     marked = np.arange(N) < M
#     k = np.array([marked[rng.choice(N, n, replace=False)].sum() for _ in range(10_000)])
#     est = M * n / k
#     p = [comb(M, j) * comb(N - M, n - j) / comb(N, n) for j in range(n + 1)]
#     print(est.mean(), sum(M * n / j * p[j] for j in range(1, n + 1)))
# A fresh generator with the project seed: verify2.py's stream (A-F19.sim_*, A-CH12.ring_toss.*)
# is untouched and keeps its keys. The digits depend on numpy's Generator stream for
# Generator.choice; the fact sheet records the numpy version they were printed with.

def ch09_panel():
    """A-CH09.panel: the 09_6 code panel's numbers (the panel above, verbatim)."""
    rng = np.random.default_rng(SEED)
    M = n = 1_000
    N = 10_000
    marked = np.arange(N) < M
    k = np.array([marked[rng.choice(N, n, replace=False)].sum() for _ in range(10_000)])
    est = M * n / k
    p = [comb(M, j) * comb(N - M, n - j) / comb(N, n) for j in range(n + 1)]
    exact_line = sum(M * n / j * p[j] for j in range(1, n + 1))
    pre = "A-CH09.panel"
    emit(f"{pre}.print_line", " ".join(str(v) for v in (est.mean(), exact_line)))
    emit(f"{pre}.experiments", int(k.size))
    emit(f"{pre}.k_zero_count", int((k == 0).sum()))
    emit(f"{pre}.first12_k", ",".join(str(int(v)) for v in k[:12]))
    emit(f"{pre}.sim_k_mean", float(k.mean()))
    emit(f"{pre}.sim_k_sd", float(k.std()))
    emit(f"{pre}.sim_mean", float(est.mean()))
    emit(f"{pre}.sim_mean_rounded_to_10", int(round(float(est.mean()) / 10) * 10))
    emit(f"{pre}.sim_median", float(np.median(est)))
    q = np.percentile(est, [2.5, 97.5])
    emit(f"{pre}.sim_p2_5", float(q[0]))
    emit(f"{pre}.sim_p97_5", float(q[1]))
    emit(f"{pre}.exact_line_mean", exact_line)
    emit(f"{pre}.p_sum", sum(p))
    emit(f"{pre}.p100", p[100])
    se = float(est.std(ddof=1)) / sqrt(est.size)
    emit(f"{pre}.sim_mean_standard_error", se)
    emit(f"{pre}.mean_gap_in_standard_errors", (float(est.mean()) - exact_line) / se)
    # Dev D9 (description): in numpy, the soup guess at k = 0 is inf, with a RuntimeWarning
    with warnings.catch_warnings(record=True) as caught:
        warnings.simplefilter("always")
        z = M * n / np.array([0])
    emit("A-CH09.numpy_estimate_at_k0", str(float(z[0])))
    emit("A-CH09.numpy_k0_warns", any(issubclass(w.category, RuntimeWarning) for w in caught))
    return k


def ch12_ring_toss_panel(k, ring):
    """A-CH12.ring_toss_panel (12_2 from R2): the ring toss on the 09_6 panel's own experiments,
    in run order; a run rings the peg 10,000 exactly when its k is in A-CH12.ring_toss.ringing_k."""
    k_lo, k_hi, cov = ring
    miss = (k < k_lo) | (k > k_hi)
    pre = "A-CH12.ring_toss_panel"
    emit(f"{pre}.first100_ringed", int(100 - miss[:100].sum()))
    emit(f"{pre}.first100_missed", int(miss[:100].sum()))
    emit(f"{pre}.first100_missed_runs_1based", ",".join(str(int(r) + 1) for r in np.flatnonzero(miss[:100])))
    emit(f"{pre}.all_runs", int(k.size))
    emit(f"{pre}.all_missed", int(miss.sum()))
    emit(f"{pre}.expected_missed", float((1 - cov) * int(k.size)))


def ch12_coverage_over_N():
    """A-CH12.coverage (12_2 "whatever the true size is"; Ruth R9): the exact 95% range's coverage
    as a function of the true lake size, for M = n = 1,000. A range built from k rings N exactly
    when P_N(K <= k) > 1/40 and P_N(K >= k) > 1/40 (as in ch12_ring_toss), so the coverage at N is
    the sum of P_N(K = k) over those k. It is at least 95% for every N by construction (each
    one-sided test rejects with chance at most 1/40, since P(F(K) <= a) <= a for a discrete cdf F);
    this checks it numerically. Floats: log-pmf by the ratio recurrence, normalised; any N whose
    tail sums sit within 1e-9 of 1/40, or whose coverage is within 1e-9 of 0.95, is recomputed in
    exact integers, and so is the minimum. Grid: every N from 1,000 to 20,000, every 10th to
    100,000, every 1,000th to 1,000,000."""
    M = n = 1000
    alpha = 1 / 40
    grid = list(range(1000, 20001)) + list(range(20010, 100001, 10)) + list(range(101000, 1_000_001, 1000))

    def exact(N):
        T = comb(N, n)
        nums = [num(N, M, n, j) for j in range(n + 1)]
        cum = list(itertools.accumulate(nums))
        return F(sum(nums[j] for j in range(n + 1)
                     if cum[j] * 40 > T and (T - (cum[j - 1] if j else 0)) * 40 > T), T)

    covs = {}
    rechecked = 0
    for N in grid:
        lo, hi = max(0, M + n - N), min(M, n)
        ks = np.arange(lo, hi + 1, dtype=float)
        r = np.log((M - ks[:-1]) * (n - ks[:-1])) - np.log((ks[:-1] + 1) * (N - M - n + ks[:-1] + 1))
        lp = np.concatenate([[0.0], np.cumsum(r)])
        pmf = np.exp(lp - lp.max())
        pmf /= pmf.sum()
        le = np.cumsum(pmf)
        ge = np.cumsum(pmf[::-1])[::-1]
        c = float(pmf[(le > alpha) & (ge > alpha)].sum())
        if (np.abs(le - alpha) < 1e-9).any() or (np.abs(ge - alpha) < 1e-9).any() or abs(c - 0.95) < 1e-9:
            c = float(exact(N))
            rechecked += 1
        covs[N] = c
    n_min = min(covs, key=covs.get)
    c_min = exact(n_min)
    assert abs(float(c_min) - covs[n_min]) < 1e-12
    pre = "A-CH12.coverage"
    emit(f"{pre}.M_eq_n", M)
    emit(f"{pre}.grid", "every N 1000-20000; every 10th to 100000; every 1000th to 1000000")
    emit(f"{pre}.lake_sizes_checked", len(grid))
    emit(f"{pre}.rechecked_in_exact_integers", rechecked)
    emit(f"{pre}.min", c_min)
    emit(f"{pre}.min_pct_3dp", round(float(c_min) * 100, 3))
    emit(f"{pre}.argmin_N", n_min)
    emit(f"{pre}.all_at_least_95pct", float(c_min) >= 0.95)
    emit(f"{pre}.at_N10000_float", covs[10_000])


def ch12_slider_r2():
    """A-CH12.slider additions (12_5 caption; Dev dev-11, R1 R-24 / D4): the typical error (1 SD,
    the symmetric approximation sqrt((1 - M/N)(1 - n/N)/k)) at each slider point, beside the rule of
    thumb 1/sqrt(k) it trims; and the planning card's 1/sqrt(k) at 10 and 100 caught twice."""
    N = FELLER_N_TRUE
    for k in (25, 100, 400):
        m = math.isqrt(N * k)
        pre = f"A-CH12.slider.k{k}"
        emit(f"{pre}.typical_error_pct", round(100 * sqrt((1 - m / N) * (1 - m / N) / k), 6))
        emit(f"{pre}.one_over_sqrt_k_pct", round(100 / sqrt(k), 6))
    for k in (10, 100):
        emit(f"A-CH12.planning.one_over_sqrt_k_pct_k{k}", 100 / sqrt(k))


def ch13_targets():
    """A-CH13 (the R2 blocker, Ruth R1 / Maya M1 / Dev dev-02; 13_1 lower-third, 13_6 and 13_8
    gauge marks): what Mn/k aims at under A-F22's models, as Mn/E[k], with E[k] exact.
    Ideal: E[k] = nM/N. Deaths: each fish dies with chance 3/10 between the catches, whatever its
    mark, so by symmetry each fish of the second catch is marked with chance M/N: E[k] = nM/N, and
    the target is the lake on marking day, while the lake at the second catch holds N(1 - 3/10) on
    average. Newcomers: 2,000 unmarked fish join: E[k] = nM/(N + 2000), the lake at the second
    catch. Marks lost: each mark is lost with chance 1/5: E[k] = n(4/5)M/N."""
    N, M, n = FELLER_N_TRUE, FM, FN_
    models = {
        "ideal": F(n * M, N),
        "deaths_30pct": F(n * M, N),
        "newcomers_2000": F(n * M, N + 2000),
        "marks_lost_20pct": F(n, 1) * F(4, 5) * F(M, N),
    }
    for name, ek in models.items():
        emit_q(f"A-CH13.{name}.expected_k", ek)
        emit(f"A-CH13.{name}.expected_marked_share_of_second_catch", ek / n)
        emit_q(f"A-CH13.{name}.target_Mn_over_Ek", F(M * n) / ek)
    emit_q("A-CH13.deaths_30pct.expected_lake_at_second_catch", N * (1 - F(3, 10)))
    emit_q("A-CH13.newcomers_2000.lake_at_second_catch", F(N + 2000))


# ---------------------------------------------------------------------------------------------

def main():
    af1_choose_20_4()
    af2_netful_counts()
    af3_netful_probabilities()
    af4_k2_decomposition()
    af5_toy_best_size()
    af5_toy_stills()
    af6_toy_comparisons()
    # Feller's likelihood on N = 1,900..400,000 (lgamma), shared by A-F7 and A-F26
    Ns = np.arange(1900, 400_001)
    feller_logw = np.array([logL(int(N), FM, FN_, FK) for N in Ns])
    af7_likelihood_area(feller_logw)
    af8_toy_interval()
    af9_feller_best_size()
    af10_feller_comparisons()
    p1900 = af11_minimum_lake()
    af12_relative_likelihoods()
    feller_ci = af13_feller_interval()
    af14_feller_tails()
    af15_sd_of_k()
    law_feller = af16_spread_of_estimate()
    af17_chapman_feller()
    af18_chapman_identity()
    sims, fails = verify2_replay()
    af19_feller_simulation(sims, law_feller)
    af20_small_study_100(sims)
    af21_small_study_50()
    af22_failure_modes(fails)
    af23_laplace()
    af24_two_reviewers()
    af25_kosovo()
    af26_bayesian(Ns, feller_logw)
    xs, ys = af27_clustered_lake()
    af28_split_symmetry()
    af29_table_identity()
    af30_coin()
    af31_bending()
    af32_float64(p1900)
    ch11_one_marked_fish()
    ring = ch12_ring_toss(sims)
    ch12_slider(feller_ci)
    ch14_reviewers()
    ch03_dynamics(xs, ys)
    ch03_portable(xs, ys)
    # R2 (2026-10-11)
    r2_small_lake_keys()
    ch06_row_mode_rule()
    k_panel = ch09_panel()
    ch12_ring_toss_panel(k_panel, ring)
    ch12_coverage_over_N()
    ch12_slider_r2()
    ch13_targets()
    sys.stdout.flush()
    print(f"oracle: {len(_seen)} keys in {time.perf_counter() - T_START:.1f} s", file=sys.stderr)


if __name__ == "__main__":
    main()
