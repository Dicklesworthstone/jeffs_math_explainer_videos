#!/usr/bin/env python3
"""Independent oracle for Video A, "How to Count Fish You've Never Seen" (slug capture_recapture).

Prints one line per fact, ``KEY<TAB>VALUE``, for every row A-F1 .. A-F32 of the facts ledger in
docs/planning/COMPREHENSIVE_PLAN_FOR_JEFFS_MATH_EXPLAINER_VIDEOS.md section 9.5, plus the chapter-03
habitat-dynamics check (keys ``A-CH03.*``; that is not a ledger row).

Run from the repository root:

    python3 -I docs/facts/capture_recapture/oracle.py

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

import math
import sys
import time
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
        return out

    run("habitat", True, True)
    run("contrast_no_drift", False, True)
    run("diag_no_circulation", True, False)


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
    af13_feller_interval()
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
    ch03_dynamics(xs, ys)
    sys.stdout.flush()
    print(f"oracle: {len(_seen)} keys in {time.perf_counter() - T_START:.1f} s", file=sys.stderr)


if __name__ == "__main__":
    main()
