#!/usr/bin/env python3
"""Facts oracle for "Why P-Values Confuse Even Doctors" (p_values_medical).

Run from this folder (`./x p_values_medical facts-check` does):

    python3 -I -B oracle.py

stdout: one `KEY<TAB>VALUE` line per fact, the same keys `pvalues facts`
prints (src/facts.rs), in the rows of docs/fact-sheet.md: M-C (cited), M-F
(the clinics, counted), M-T (the study's null world, enumerated) and M-X (the
wrong conditionals and wrong tails, which the video names as not the answer).

stderr: cross-checks (lines starting with "#") and a summary; exit 1 if any
cross-check failed.

How the values are obtained, independently of the Rust model:
  * every clinic is built here as an explicit list of people, one tuple per
    person, from its rates with `fractions.Fraction` (refusing any rate that
    would split a person); every conditional chance is a filter-and-count over
    that list, and Bayes' rule is checked against the count;
  * the study's null world lists all 2**20 assignments of the treatment label
    by `itertools.product` over the pairs (not by bit tricks) and compares
    with `math.comb`;
  * CITED literals from the sources in docs/fact-sheet.md.
  * chapter 03's re-tosses replayed with NumPy's own PCG64DXSM (the one
    numpy use; `./x p_values_medical facts-check` runs the oracle with numpy),
    and small studies listed exhaustively over every finishing order.
"""
import math
import sys
from fractions import Fraction as F
from itertools import product

_EMITTED = {}
_FAILED = []
_CHECKS = [0]


def emit(key, value):
    if key in _EMITTED:
        raise SystemExit(f"oracle: {key} emitted twice")
    _EMITTED[key] = str(value)
    print(f"{key}\t{value}")


def check(name, ok):
    _CHECKS[0] += 1
    if not ok:
        _FAILED.append(name)
    print(f"# {'ok  ' if ok else 'FAIL'} {name}", file=sys.stderr)


def dec(x, places):
    """The decimal with `places` digits, rounded half away from zero, in integers."""
    x = F(x)
    scaled = abs(x) * 10**places
    q = (2 * scaled.numerator + scaled.denominator) // (2 * scaled.denominator)
    sign = "-" if x < 0 and q != 0 else ""
    if places == 0:
        return f"{sign}{q}"
    return f"{sign}{q // 10**places}.{q % 10**places:0{places}d}"


def exact(x):
    x = F(x)
    return str(x.numerator) if x.denominator == 1 else f"{x.numerator}/{x.denominator}"


def pct(x, places=0):
    return dec(F(x) * 100, places)


# ------------------------------------------------------------------- cited

# S1: Windish, Huot and Green, JAMA 2007;298(9):1010-1022,
# doi 10.1001/jama.298.9.1010, published 2007-09-05. Abstract (read via Europe
# PMC, 2026-10-17): "The survey was completed by 277 of 367 residents (75.5%)
# in 11 residency programs." Methods (JAMA's indexed publisher text, read by
# the orchestrator on 2026-10-17; a direct fetch timed out): data collected
# February to July 2006, in 11 Connecticut programs. Published 2007, run 2006.
PUBLISHED_YEAR = 2007
PUBLISHED = "2007-09-05"
COLLECTED_YEAR = 2006
COLLECTED_MONTHS = "February to July"
RESPONDENTS = 277
RESIDENTS = 367
RESPONSE_PERCENT_PRINTED = "75.5"
PROGRAMS = 11
# S2: Gao, "P-values - a chronic conundrum", BMC Med Res Methodol 2020;20:167,
# doi 10.1186/s12874-020-01051-6 (full text read via Europe PMC PMC7315482,
# 2026-10-17): "88% of the residents expressed fair to complete confidence in
# understanding p-values" and "58.8% of the residents selected choice c which
# was designated by the authors as the correct answer."
CONFIDENT_PERCENT = 88
KEYED_CHOICE = "c"
KEYED_PERCENT = F(588, 10)
GAO_YEAR = 2020


def cited():
    emit("M-C1.survey.published_year", PUBLISHED_YEAR)
    emit("M-C1.survey.published", PUBLISHED)
    emit("M-C1.survey.collected_year", COLLECTED_YEAR)
    emit("M-C1.survey.collected_months", COLLECTED_MONTHS)
    check("the survey was collected before it was published",
          COLLECTED_YEAR < PUBLISHED_YEAR and PUBLISHED.startswith(str(PUBLISHED_YEAR)))
    emit("M-C1.survey.respondents", RESPONDENTS)
    emit("M-C1.survey.residents", RESIDENTS)
    response = pct(F(RESPONDENTS, RESIDENTS), 1)
    check("277 of 367 is the abstract's 75.5%", response == RESPONSE_PERCENT_PRINTED)
    emit("M-C1.survey.response_percent", response)
    emit("M-C1.survey.programs", PROGRAMS)
    emit("M-C1.survey.confident_percent", CONFIDENT_PERCENT)
    emit("M-C1.survey.keyed_choice", KEYED_CHOICE)
    emit("M-C1.survey.keyed_percent", dec(KEYED_PERCENT, 1))
    # "about fifty-nine percent": the printed 58.8 rounded to a whole percent
    emit("M-C1.survey.keyed_percent_spoken", dec(KEYED_PERCENT, 0))
    # the script never says most residents chose a distractor: most chose the key
    check("more than half picked the keyed answer", KEYED_PERCENT > 50)
    emit("M-C2.gao.year", GAO_YEAR)


# ----------------------------------------------------------------- clinics

def people_of(n, prevalence, sensitivity, false_alarm):
    """Every person of a clinic as (disease, positive), built one by one."""
    with_ = F(n) * prevalence
    if with_.denominator != 1:
        raise ValueError(f"the disease splits a person: {with_} of {n}")
    with_ = int(with_)
    tp = F(with_) * sensitivity
    fp = F(n - with_) * false_alarm
    if tp.denominator != 1 or fp.denominator != 1:
        raise ValueError(f"a rate splits a person: {tp} caught, {fp} false alarms")
    tp, fp = int(tp), int(fp)
    people = []
    for i in range(with_):
        people.append((True, i < tp))
    for i in range(n - with_):
        people.append((False, i < fp))
    return people


def given(people, event, condition):
    """The share of the people `condition` picks out for whom `event` holds."""
    group = [p for p in people if condition(p)]
    return F(sum(1 for p in group if event(p)), len(group))


DISEASE = lambda p: p[0]  # noqa: E731
NO_DISEASE = lambda p: not p[0]  # noqa: E731
POSITIVE = lambda p: p[1]  # noqa: E731
NEGATIVE = lambda p: not p[1]  # noqa: E731
EVERYONE = lambda p: True  # noqa: E731


def bayes(q, s, f):
    return s * q / (s * q + f * (1 - q))


BLOCK_COLS = 45  # both chapters draw the positives in rows of 45


def clinic(prefix, n, q, s, f):
    people = people_of(n, q, s, f)
    count = lambda pred: sum(1 for p in people if pred(p))  # noqa: E731
    with_, without = count(DISEASE), count(NO_DISEASE)
    tp = count(lambda p: p[0] and p[1])
    fn = count(lambda p: p[0] and not p[1])
    fp = count(lambda p: not p[0] and p[1])
    tn = count(lambda p: not p[0] and not p[1])
    pos = count(POSITIVE)
    ppv = given(people, DISEASE, POSITIVE)
    check(f"{prefix}: counting the positives agrees with Bayes' rule", ppv == bayes(q, s, f))
    check(f"{prefix}: the people reproduce the clinic's rates",
          given(people, POSITIVE, NO_DISEASE) == f
          and given(people, POSITIVE, DISEASE) == s
          and given(people, DISEASE, EVERYONE) == q)
    emit(f"{prefix}.prevalence.exact", exact(q))
    emit(f"{prefix}.with", with_)
    emit(f"{prefix}.without", without)
    emit(f"{prefix}.true_pos", tp)
    emit(f"{prefix}.false_pos", fp)
    emit(f"{prefix}.positives", pos)
    emit(f"{prefix}.ppv.exact", exact(ppv))
    emit(f"{prefix}.ppv_percent_2dp", pct(ppv, 2))
    emit(f"{prefix}.people", n)
    emit(f"{prefix}.prevalence.one_in", exact(1 / q))
    emit(f"{prefix}.sensitivity.exact", exact(s))
    emit(f"{prefix}.sensitivity_percent", pct(s))
    emit(f"{prefix}.false_alarm.exact", exact(f))
    emit(f"{prefix}.false_alarm_percent", pct(f))
    emit(f"{prefix}.false_neg", fn)
    emit(f"{prefix}.true_neg", tn)
    emit(f"{prefix}.ppv_percent", pct(ppv))
    reverse = given(people, NO_DISEASE, POSITIVE)
    emit(f"{prefix}.reverse.exact", exact(reverse))
    emit(f"{prefix}.reverse_percent_2dp", pct(reverse, 2))
    tempting = given(people, NEGATIVE, NO_DISEASE)
    emit(f"{prefix}.tempting_percent", pct(tempting))
    # M-X: the swapped conditional and the tempting answer are different numbers
    false_alarm = given(people, POSITIVE, NO_DISEASE)
    check(f"{prefix}: P(positive | no disease) != P(no disease | positive)", false_alarm != reverse)
    check(f"{prefix}: the tempting 95% is 1 - the false-alarm rate", tempting == 1 - false_alarm)
    check(f"{prefix}: the tempting 95% is not the PPV", tempting != ppv)
    check(f"{prefix}: the reversal, not the false-alarm rate, complements the PPV",
          reverse == 1 - ppv and false_alarm != 1 - ppv)
    # the icon array: rows of a hundred; whole disease rows; equal false alarms per other row
    cols = 100
    rows = n // cols
    healthy_rows = without // cols
    check(f"{prefix}: rows of {cols} fit the clinic exactly",
          n % cols == 0 and with_ % cols == 0 and fp % healthy_rows == 0)
    emit(f"{prefix}.grid.cols", cols)
    emit(f"{prefix}.grid.rows", rows)
    emit(f"{prefix}.grid.disease_rows", with_ // cols)
    emit(f"{prefix}.grid.false_pos_per_row", fp // healthy_rows)
    # the regrouped positives, in rows of 45: both groups must fill whole rows
    block = BLOCK_COLS
    check(f"{prefix}: both groups of positives fill rows of {block}", tp % block == 0 and fp % block == 0)
    check(f"{prefix}: the block's disease rows are the PPV", F(tp // block, pos // block) == ppv)
    emit(f"{prefix}.block.cols", block)
    emit(f"{prefix}.block.rows", pos // block)
    emit(f"{prefix}.block.disease_rows", tp // block)
    return people, ppv


def refused(n, q, s, f):
    """True when a clinic would split a person (so it cannot be drawn)."""
    try:
        people_of(n, q, s, f)
    except ValueError:
        return True
    return False


def clinics():
    people_a, ppv_a = clinic("M-F1", 10_000, F(1, 100), F(9, 10), F(1, 20))
    people_b, ppv_b = clinic("M-F2", 10_000, F(1, 10), F(9, 10), F(1, 20))
    check("the same test in another clinic: the false-alarm rate holds and the PPV moves",
          given(people_a, POSITIVE, NO_DISEASE) == given(people_b, POSITIVE, NO_DISEASE)
          and ppv_a != ppv_b)
    check("the same test in another clinic: the share caught holds too",
          given(people_a, POSITIVE, DISEASE) == given(people_b, POSITIVE, DISEASE) == F(9, 10))
    check("chapter 02's shift is 2/13 to 2/3", (ppv_a, ppv_b) == (F(2, 13), F(2, 3)))
    check("chapter 01's widest block (gcd 90, 495) is the 45 both chapters draw",
          math.gcd(90, 495) == BLOCK_COLS)
    # M-F2 against M-F1: ten times the prevalence, ten times the people caught
    tp_a = sum(1 for p in people_a if p[0] and p[1])
    tp_b = sum(1 for p in people_b if p[0] and p[1])
    emit("M-F2.prevalence_vs_a", exact(F(1, 10) / F(1, 100)))
    emit("M-F2.caught_vs_a", exact(F(tp_b, tp_a)))
    # the swapped conditional in clinic B, counted: 1/20 one way, 1/3 the other
    check("clinic B: P(positive | no disease) = 1/20, P(no disease | positive) = 1/3",
          given(people_b, POSITIVE, NO_DISEASE) == F(1, 20) and given(people_b, NO_DISEASE, POSITIVE) == F(1, 3))
    # the PPV rises with the prevalence while the test's rates stay put (counted, not by formula)
    sweep, rates_hold = [], True
    for q in (F(0), F(1, 100), F(1, 20), F(1, 10), F(1, 4), F(1, 2), F(9, 10)):
        ppl = people_of(10_000, q, F(9, 10), F(1, 20))
        rates_hold &= given(ppl, POSITIVE, NO_DISEASE) == F(1, 20)
        if q:
            rates_hold &= given(ppl, POSITIVE, DISEASE) == F(9, 10)
        sweep.append(given(ppl, DISEASE, POSITIVE))
    check("over seven clinics the test's rates hold", rates_hold)
    check("over seven clinics the PPV only rises", all(a < b for a, b in zip(sweep, sweep[1:])))
    # whole people at the edges: near misses are refused, not rounded
    check("10,001 people at 1 in 10 would split a person", refused(10_001, F(1, 10), F(9, 10), F(1, 20)))
    check("1,000 people at 1 in 100 would give 49.5 false alarms", refused(1_000, F(1, 100), F(9, 10), F(1, 20)))
    t = people_of(10_000, F(1, 20), F(9, 10), F(1, 20))
    tp_t = sum(1 for p in t if p[0] and p[1])
    fp_t = sum(1 for p in t if not p[0] and p[1])
    check("a 1-in-20 clinic's positives (450, 475) cannot fill rows of 45",
          (tp_t, fp_t) == (450, 475) and fp_t % BLOCK_COLS != 0)
    # Bayes against counting over a sweep of whole-person clinics
    ok = True
    for q in (F(1, 100), F(1, 20), F(1, 10), F(1, 4), F(1, 2)):
        for f in (F(1, 100), F(1, 20), F(1, 10)):
            ppl = people_of(10_000, q, F(9, 10), f)
            ok &= given(ppl, DISEASE, POSITIVE) == bayes(q, F(9, 10), f)
    check("Bayes' rule equals counting on 15 whole-person clinics", ok)


# ------------------------------------------------------------ the null world

PAIRS = 20
# the synthetic study (src/model.rs `study`): bit i set when patient A of pair i
# recovered faster; the observed assignment gave A the treatment except where
# the treated patient lost (pairs 3, 7, 11, 16, 19)
FASTER_A = 0b1011_0110_1100_1011_0101
AGAINST = (3, 7, 11, 16, 19)
TREATED_A = FASTER_A ^ sum(1 << i for i in AGAINST)


def favouring(treated_a_bits, faster_a_bits):
    """Pairs whose faster patient carries the treatment label (lists, not bits)."""
    return sum(1 for t, w in zip(treated_a_bits, faster_a_bits) if t == w)


def bits(mask):
    return [(mask >> i) & 1 for i in range(PAIRS)]


def null_world():
    faster = bits(FASTER_A)
    observed = favouring(bits(TREATED_A), faster)
    world = [0] * (PAIRS + 1)
    for assignment in product((0, 1), repeat=PAIRS):
        world[favouring(assignment, faster)] += 1
    check("the null world is the binomial (math.comb)",
          world == [math.comb(PAIRS, k) for k in range(PAIRS + 1)])
    check("2**20 equally likely assignments", sum(world) == 2**PAIRS)
    return observed, world


def tail(world, k, kind):
    n = len(world) - 1
    keep = {
        "at_least": lambda j: j >= k,
        "exactly": lambda j: j == k,
        "more_than": lambda j: j > k,
        "at_most": lambda j: j <= k,
        "two_sided": lambda j: abs(2 * j - n) >= abs(2 * k - n),
    }[kind]
    return sum(c for j, c in enumerate(world) if keep(j))


RETOSS_SEED = 2006  # chapter 03, fixed before any toss was looked at
RETOSSES = 4


def chapter_03(world, observed):
    """The re-tossed coins, the histogram's top, and the planted mistakes."""
    import numpy as np

    faster = bits(FASTER_A)
    top = max(world)
    emit("M-T1.null.max", top)
    emit("M-T1.null.mode", world.index(top))
    raw = np.random.PCG64DXSM(RETOSS_SEED).random_raw(RETOSSES)
    tosses = [int(r) % 2**PAIRS for r in raw]
    counts = [favouring(bits(a), faster) for a in tosses]
    emit("M-T3.retoss.seed", RETOSS_SEED)
    emit("M-T3.retoss.assignments", ",".join(str(a) for a in tosses))
    emit("M-T3.retoss.counts", ",".join(str(k) for k in counts))
    check("chapter 03's re-tosses are four new assignments, none the observed one",
          len(set(tosses + [TREATED_A])) == RETOSSES + 1)
    # the planted wrong statistic: patients A treated, not pairs favouring
    a_treated = sum(bits(TREATED_A))
    emit("M-X3.a_treated", a_treated)
    check("patients A treated (9) is not the pairs favouring (15)", (a_treated, observed) == (9, 15))
    # one swapped label changes that pair only, by exactly one
    treated = bits(TREATED_A)
    ok = True
    for i in range(PAIRS):
        swapped = treated[:i] + [1 - treated[i]] + treated[i + 1:]
        ok &= abs(favouring(swapped, faster) - observed) == 1
    check("swapping any one pair's labels moves the count by exactly one", ok)
    # small studies, every finishing order, every assignment: the binomial,
    # and each assignment in exactly one bar
    ok = True
    for n in range(1, 8):
        assignments = list(product((0, 1), repeat=n))
        for order in assignments:
            bars = {}
            for a in assignments:
                bars.setdefault(favouring(a, order), []).append(a)
            ok &= [len(bars.get(k, [])) for k in range(n + 1)] == [math.comb(n, k) for k in range(n + 1)]
            ok &= sorted(a for bar in bars.values() for a in bar) == sorted(assignments)
    check("1 to 7 pairs, every finishing order: binomial bars, no assignment counted twice", ok)


def study():
    observed, world = null_world()
    total = sum(world)
    emit("M-T1.pairs", PAIRS)
    emit("M-T1.patients", len([patient for _ in range(PAIRS) for patient in ("A", "B")]))
    emit("M-T1.observed", observed)
    emit("M-T1.assignments", total)
    emit("M-T1.null.counts", ",".join(str(c) for c in world))
    chapter_03(world, observed)
    at_least = tail(world, observed, "at_least")
    p = F(at_least, total)
    emit("M-T1.tail.at_least", at_least)
    emit("M-T1.p.exact", exact(p))
    emit("M-T1.p_10dp", dec(p, 10))
    emit("M-T1.p_percent_2dp", pct(p, 2))
    emit("M-T1.p_percent_1dp", pct(p, 1))
    check("the research brief's 21,700 / 1,048,576 = 0.0206947327",
          at_least == 21_700 and total == 1_048_576 and dec(p, 10) == "0.0206947327")
    # M-X1: the wrong tails, each a different number
    exactly = tail(world, observed, "exactly")
    more = tail(world, observed, "more_than")
    at_most = tail(world, observed, "at_most")
    two = tail(world, observed, "two_sided")
    emit("M-X1.exactly", exactly)
    emit("M-X1.exactly_percent_2dp", pct(F(exactly, total), 2))
    emit("M-X1.more_than", more)
    emit("M-X1.more_than_percent_2dp", pct(F(more, total), 2))
    emit("M-X1.at_most", at_most)
    emit("M-X1.two_sided", two)
    emit("M-X1.two_sided_percent_2dp", pct(F(two, total), 2))
    check("the tail is the observation plus everything more extreme", at_least == exactly + more)
    check("leaving out either part changes the number", exactly != at_least and more != at_least)
    check("the other direction is everything but the more extreme", at_most == total - more)
    check("two-sided doubles the symmetric one-sided tail", two == 2 * at_least)
    # M-T2: one pair fewer favours treatment
    near = observed - 1
    near_tail = tail(world, near, "at_least")
    emit("M-T2.observed", near)
    emit("M-T2.tail.at_least", near_tail)
    emit("M-T2.p_10dp", dec(F(near_tail, total), 10))
    check("the brief's 60,460 / 1,048,576 = 0.0576591492",
          near_tail == 60_460 and dec(F(near_tail, total), 10) == "0.0576591492")
    # the edges of "a p-value here is never zero" and "the least favourable count has p = 1"
    check("K >= 0 has p = 1", tail(world, 0, "at_least") == total)
    check("the most favourable count has p = 1/2**20, not zero", tail(world, PAIRS, "at_least") == 1)


def main():
    cited()
    clinics()
    study()
    print(f"# {_CHECKS[0] - len(_FAILED)} of {_CHECKS[0]} cross-checks passed", file=sys.stderr)
    if _FAILED:
        print("# FAILED: " + "; ".join(_FAILED), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
