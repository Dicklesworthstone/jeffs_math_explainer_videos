#!/usr/bin/env python3
"""Facts oracle for "Check a Giant Matrix Product Without Doing the Product Again".

Run from this folder (`./x freivalds facts-check` does):

    python3 -I -B oracle.py

stdout: one `KEY<TAB>VALUE` line per fact, the same keys `freivalds facts`
prints (src/facts.rs): F-F rows (the example) and F-X rows (the edges of the
always/never sentences).

stderr: cross-checks (lines starting with "#") and a summary; exit 1 if any
cross-check failed.

How the values are obtained, independently of the Rust model (which adds the
heads columns of B and C and multiplies A by that column, entry by entry):
  * the product A·B as a sum of outer products, column k of A times row k of
    B, accumulated over k, and cross-checked against row-times-column;
  * each toss's check by actually multiplying by the 0/1 column r, and again
    as the mistake D = C − A·B times r (a toss passes exactly when D·r = 0);
  * the at-most-half sweep over every small mistake from per-row solution
    sets: for each possible row, the set of tosses that row sends to zero,
    intersected over D's rows (no check is run);
  * the per-toss bound is the pairing proof's one half (any n, any ring),
    not anything the finite sweeps compute: they check it at each size
    (never exceeded, the pairing step never broken, an equality case), and
    the twenty-toss bound is its twentieth power (the example's own share
    is a separate fact);
  * "a known toss can be fooled" by brute-force search for a nonzero D with
    D·r = 0 among all 3 × 3 mistakes with entries −1, 0, 1 (the Rust
    constructs one), and the same search at 2 × 2 and 1 × 1; chapter 03's
    blind spots shown on screen follow the Rust's construction (it is a
    choice of counterexample) and are checked here by multiplying;
  * two tosses as all 64 ordered ways, each checked twice by multiplying,
    not as a product of shares;
  * fractions with fractions.Fraction, whose str() is the format the video
    prints;
  * chapter 04's doubles near 10^16 from Python floats and math.nextafter,
    and both rounded examples' columns by the explicit loop below, against
    exact integer columns from multiplying;
  * doubles with an explicit left-to-right loop: Python's sum() of floats is
    compensated (Neumaier) since 3.12 and would hide the rounding.
"""
import math
import sys
from fractions import Fraction
from itertools import permutations, product

_EMITTED = {}
_FAILED = []
_CHECKS = [0]

A = ((1, 2, 1), (0, 3, 1), (3, 2, 0))
B = ((2, 1, 0), (1, 0, 2), (2, 1, 3))
C = ((6, 2, 7), (5, 2, 9), (8, 3, 4))
A2 = ((1, 2), (0, 3))
B2 = ((2, 1), (1, 0))
FIRST_TOSS = (1, 1, 0)
SPOT = (0, 2)
BIG_SIDE = 1000
AGAIN = 20
MILLION = 10**6
SHAPES = (
    ("one_entry", ((0, 0, 0), (0, 1, 0), (0, 0, 0))),
    ("two_in_a_row", ((0, 0, 0), (0, 1, 1), (0, 0, 0))),
    ("cancelling", ((0, 0, 0), (0, 1, -1), (0, 0, 0))),
    ("three_in_a_row", ((0, 0, 0), (1, 1, -1), (0, 0, 0))),
    ("two_rows", ((1, 0, 0), (0, 1, 0), (0, 0, 0))),
    ("every_column", ((1, 0, 0), (0, 1, 0), (0, 0, 1))),
)
# The pairing proof (UIUC CS 473, Fall 2019, lecture 12): two tosses that
# differ only in the coin of a column where C is wrong cannot both pass, so at
# most one toss of each pair of two passes, for any n over any ring. A
# constant of the proof; the example's pairs are checked against it.
MISSES_PER_PAIR = 1
PAIRING_BOUND = Fraction(MISSES_PER_PAIR, 2)
BIG = 10**16
ROUNDED_REJECTS = (((1, 1), (0, 0)), ((BIG, 1), (-BIG, 1)), ((0, 2), (0, 0)))
ROUNDED_PASSES = (((BIG, 1), (0, 0)), ((1, 0), (1, 0)), ((BIG, 0), (0, 0)))


def emit(key, value):
    if key in _EMITTED:
        raise SystemExit(f"oracle: {key} emitted twice")
    if isinstance(value, bool):
        value = "true" if value else "false"
    _EMITTED[key] = str(value)
    print(f"{key}\t{value}")


def check(name, ok):
    _CHECKS[0] += 1
    if not ok:
        _FAILED.append(name)
    print(f"# {'ok  ' if ok else 'FAIL'} {name}", file=sys.stderr)


def csv(values):
    return ",".join(str(x) for x in values)


def matrix_text(m):
    return ";".join(csv(row) for row in m)


def outer_product_sum(a, b):
    """a·b as the sum over k of (column k of a) times (row k of b)."""
    n = len(a)
    out = [[0] * n for _ in range(n)]
    for k in range(n):
        for i in range(n):
            for j in range(n):
                out[i][j] += a[i][k] * b[k][j]
    return tuple(tuple(row) for row in out)


def row_times_column(a, b):
    n = len(a)
    return tuple(tuple(sum(a[i][k] * b[k][j] for k in range(n)) for j in range(n)) for i in range(n))


def apply(m, r):
    """m times the column r, by multiplying."""
    return tuple(sum(x * y for x, y in zip(row, r)) for row in m)


def minus(x, y):
    return tuple(tuple(p - q for p, q in zip(rx, ry)) for rx, ry in zip(x, y))


def plus(x, y):
    return tuple(tuple(p + q for p, q in zip(rx, ry)) for rx, ry in zip(x, y))


def tosses(n):
    """Counting order, first coin first: 000, 001, ..., 111."""
    return list(product((0, 1), repeat=n))


def rounded_apply(m, r):
    out = []
    for row in m:
        acc = 0.0
        for x, y in zip(row, r):
            acc = acc + float(x) * float(y)
        out.append(acc)
    return out


def rounded_passes(a, b, c, r):
    return rounded_apply(a, rounded_apply(b, r)) == rounded_apply(c, r)


def zero_sets(n, values):
    """For every row of n entries from `values`: the set of tosses it sends to zero."""
    all_tosses = tosses(n)
    return {row: frozenset(t for t in all_tosses if sum(x * y for x, y in zip(row, t)) == 0)
            for row in product(values, repeat=n)}


def sweep(n, values):
    """Every nonzero n × n mistake from `values`: the tosses each one passes,
    from the rows' solution sets."""
    rows = zero_sets(n, values)
    all_tosses = tosses(n)
    everything = frozenset(all_tosses)
    wrong = fewest = most = at_half = uneven = pairs = 0
    fewest = len(all_tosses) + 1
    for d in product(product(values, repeat=n), repeat=n):
        if all(x == 0 for row in d for x in row):
            continue
        passing = everything
        for row in d:
            passing = passing & rows[row]
        count = len(passing)
        wrong += 1
        fewest = min(fewest, count)
        most = max(most, count)
        at_half += 2 * count == len(all_tosses)
        uneven += count & (count - 1) != 0
        for j in range(n):
            if all(row[j] == 0 for row in d):
                continue
            for t in passing:
                if t[j] == 0:
                    partner = t[:j] + (1,) + t[j + 1:]
                    pairs += partner in passing
    return wrong, fewest, most, at_half, uneven, pairs


def facts():
    n = len(A)
    ab = outer_product_sum(A, B)
    d = minus(C, ab)
    wrong = [(i, j) for i in range(n) for j in range(n) if d[i][j] != 0]

    # 01_1
    emit("F-F1.factors", len((A, B)))
    emit("F-F1.side", n)
    emit("F-F1.a", matrix_text(A))
    emit("F-F1.b", matrix_text(B))
    emit("F-F1.ab", matrix_text(ab))
    emit("F-F1.c", matrix_text(C))
    emit("F-F1.wrong_entries", len(wrong))
    (wi, wj), = wrong
    emit("F-F1.wrong.row", wi + 1)
    emit("F-F1.wrong.column", wj + 1)
    emit("F-F1.wrong.claimed", C[wi][wj])
    emit("F-F1.wrong.true", ab[wi][wj])

    # 01_2, 01_3
    si, sj = SPOT
    terms = [A[si][k] * B[k][sj] for k in range(n)]
    agreeing = sum(1 for i in range(n) for j in range(n) if C[i][j] == ab[i][j])
    multiplications = len([(i, j, k) for i in range(n) for j in range(n) for k in range(n)])
    emit("F-F2.spot.row", si + 1)
    emit("F-F2.spot.column", sj + 1)
    emit("F-F2.spot.products", csv(terms))
    emit("F-F2.spot.value", sum(terms))
    emit("F-F2.spot.claimed", C[si][sj])
    emit("F-F2.spot.entries_checked", len([SPOT]))
    emit("F-F2.per_entry", len(terms))
    emit("F-F2.entries", n * n)
    emit("F-F2.product_multiplications", multiplications)
    emit("F-F2.spot.agreeing", agreeing)
    emit("F-F2.spot.miss_chance", Fraction(agreeing, n * n))
    check_mults = len([(i, k) for i in range(n) for k in range(n)])
    emit("F-F2.check_multiplications", check_mults)
    emit("F-F2.big.side", BIG_SIDE)
    emit("F-F2.big.product_multiplications", BIG_SIDE**3)
    emit("F-F2.big.check_multiplications", BIG_SIDE**2)

    # 01_4 to 01_6
    r = FIRST_TOSS
    br, cr = apply(B, r), apply(C, r)
    a_br = apply(A, br)
    emit("F-F3.tried", len([FIRST_TOSS]))
    emit("F-F3.toss", "".join(str(x) for x in r))
    emit("F-F3.r", csv(r))
    emit("F-F3.heads_columns", csv(j + 1 for j in range(n) if r[j]))
    emit("F-F3.br", csv(br))
    emit("F-F3.a_br", csv(a_br))
    emit("F-F3.cr", csv(cr))
    emit("F-F3.disagree_rows", csv(i + 1 for i in range(n) if a_br[i] != cr[i]))
    emit("F-F3.passes", a_br == cr)
    emit("F-F3.multiplications", check_mults)

    # 01_7 to 01_9
    all_tosses = tosses(n)
    passing = [t for t in all_tosses if apply(A, apply(B, t)) == apply(C, t)]
    emit("F-F4.coins", len(FIRST_TOSS))
    emit("F-F4.tosses", len(all_tosses))
    emit("F-F4.passing", ",".join("".join(str(x) for x in t) for t in passing))
    emit("F-F4.misses", len(passing))
    emit("F-F4.miss_fraction", Fraction(len(passing), len(all_tosses)))
    emit("F-F4.misses_are_tails_on_wrong_column", set(passing) == {t for t in all_tosses if t[wj] == 0})
    emit("F-F4.right_answer.passing", sum(1 for t in all_tosses if apply(A, apply(B, t)) == apply(ab, t)))

    # the per-toss bound is the pairing proof's; this oracle's own sweep (row
    # solution sets, no check run) only checks it at each size: never
    # exceeded, the pairing step never broken, and an equality case. The
    # example meeting the bound is a separate fact.
    bound = PAIRING_BOUND
    sweeps = {key: (size, values, sweep(size, values))
              for key, size, values in (("n3", 3, (-1, 0, 1)), ("n2", 2, (-2, -1, 0, 1, 2)))}
    for key, (size, _, result) in sweeps.items():
        share = Fraction(result[2], 2**size)
        check(f"{key}: no small mistake passes more than the pairing bound", share <= bound)
        check(f"{key}: no coin pair passes twice", result[5] == 0)
        check(f"{key}: some small mistake meets the bound (an equality case)", share == bound)
    emit("F-F4.example_meets_bound", Fraction(len(passing), len(all_tosses)) == bound)

    # 01_10
    pairs = list(product(all_tosses, repeat=2))
    both = sum(1 for s, t in pairs if s in passing and t in passing)
    emit("F-F5.two.pairs", len(pairs))
    emit("F-F5.two.both_miss", both)
    emit("F-F5.two.fraction", Fraction(both, len(pairs)))
    reused = sum(1 for t in all_tosses if t in passing and t in passing)
    emit("F-F5.same_toss_twice.misses", reused)
    emit("F-F5.same_toss_twice.fraction", Fraction(reused, len(all_tosses)))
    emit("F-F5.bound_per_toss", bound)
    again_bound = bound**AGAIN
    emit("F-F5.again", AGAIN)
    emit("F-F5.again.bound", again_bound)
    emit("F-F5.million", MILLION)
    emit("F-F5.again.under_one_in_a_million", again_bound < Fraction(1, MILLION))
    again = AGAIN * BIG_SIDE**2
    emit("F-F5.big.again_multiplications", again)
    emit("F-F5.big.again_share", Fraction(again, BIG_SIDE**3))

    # the bound's edges
    for key, (size, values, result) in sweeps.items():
        wrong_n, fewest, most, at_half, uneven, coin_pairs = result
        count = 2**size
        emit(f"F-X1.{key}.values", csv(values))
        emit(f"F-X1.{key}.tosses", count)
        emit(f"F-X1.{key}.wrong", wrong_n)
        emit(f"F-X1.{key}.fewest", fewest)
        emit(f"F-X1.{key}.most", most)
        emit(f"F-X1.{key}.most_share", Fraction(most, count))
        emit(f"F-X1.{key}.at_half", at_half)
        emit(f"F-X1.{key}.uneven", uneven)
        emit(f"F-X1.{key}.pairs_both_passing", coin_pairs)
        emit(f"F-X1.{key}.attains_bound", Fraction(most, count) == PAIRING_BOUND)
    for name, shape in SHAPES:
        emit(f"F-X2.{name}.misses", sum(1 for t in all_tosses if all(x == 0 for x in apply(shape, t))))

    # a toss known in advance: search every small mistake for one it passes
    small = [dd for dd in product(product((-1, 0, 1), repeat=n), repeat=n) if any(x for row in dd for x in row)]
    fooled = sum(1 for t in all_tosses if any(all(x == 0 for x in apply(dd, t)) for dd in small))
    emit("F-X3.known_toss.fooled", fooled)
    emit("F-X3.known_toss.tosses", len(all_tosses))

    # the scope edge: rounding
    two = tosses(2)
    a, b, c = ROUNDED_REJECTS
    failing = [t for t in two if not rounded_passes(a, b, c, t)]
    emit("F-X4.tosses", len(two))
    emit("F-X4.reject.right_answer", row_times_column(a, b) == c)
    emit("F-X4.reject.exact_failures", sum(1 for t in two if apply(a, apply(b, t)) != apply(c, t)))
    emit("F-X4.reject.rounded_failures", len(failing))
    emit("F-X4.reject.failing_toss", ",".join("".join(str(x) for x in t) for t in failing))
    a, b, c = ROUNDED_PASSES
    mistake = minus(c, row_times_column(a, b))
    emit("F-X4.pass.wrong_entries", sum(1 for row in mistake for x in row if x != 0))
    emit("F-X4.pass.exact_passes", sum(1 for t in two if apply(a, apply(b, t)) == apply(c, t)))
    emit("F-X4.pass.rounded_passes", sum(1 for t in two if rounded_passes(a, b, c, t)))
    chapter_02(ab)
    chapter_03(ab, passing)
    chapter_04()
    return ab, d, passing


CHAPTER_02_SHAPES = ("one_entry", "two_in_a_row", "cancelling", "three_in_a_row", "every_column")
PROOF_SHAPE = "three_in_a_row"


def toss_text(t):
    return "".join(str(x) for x in t)


def chapter_02(ab):
    """Chapter 02's facts, by multiplying the mistake by the 0/1 column."""
    n = len(A)
    all_tosses = tosses(n)
    shapes = dict(SHAPES)
    zero = (0,) * n

    slip = minus(C, ab)
    emit("F-F6.slip.mistake", matrix_text(slip))
    emit("F-F6.first_toss.heads_sum", csv(apply(slip, FIRST_TOSS)))
    differs = zero_ok = True
    for shape in shapes.values():
        c = plus(ab, shape)
        for t in all_tosses:
            a_br, cr = apply(A, apply(B, t)), apply(c, t)
            differs &= tuple(x - y for x, y in zip(cr, a_br)) == apply(shape, t)
            zero_ok &= (a_br == cr) == (apply(shape, t) == zero)
    emit("F-F6.difference_is_mistake_sum", differs)
    emit("F-F6.misses_are_zero_sums", zero_ok)

    for name in CHAPTER_02_SHAPES:
        shape = shapes[name]
        wrong = [x for row in shape for x in row if x != 0]
        hide = [t for t in all_tosses if apply(shape, t) == zero]
        emit(f"F-F7.{name}.mistake", matrix_text(shape))
        emit(f"F-F7.{name}.wrong_count", len(wrong))
        emit(f"F-F7.{name}.wrong_values", csv(wrong))
        emit(f"F-F7.{name}.hides_from", ",".join(toss_text(t) for t in hide))
        emit(f"F-F7.{name}.miss_share", Fraction(len(hide), len(all_tosses)))
        emit(f"F-F7.{name}.power_of_two", len(hide) > 0 and len(hide) & (len(hide) - 1) == 0)
    each = shapes["every_column"]
    counts = {sum(1 for x in row if x) for row in each} | {sum(1 for row in each if row[j]) for j in range(n)}
    emit("F-F7.every_column.wrong_per_row_and_column", csv(sorted(counts)))

    nonzero = [x for x in range(-3, 4) if x]
    same = [(a, b) for a in nonzero for b in nonzero if (a > 0) == (b > 0)]
    same_bad = sum(1 for a, b in same for t in all_tosses
                   if (apply(((0, 0, 0), (0, a, b), (0, 0, 0)), t) == zero) != (t[1] == 0 and t[2] == 0))
    cancel_bad = sum(1 for a in nonzero for t in all_tosses
                     if (apply(((0, 0, 0), (0, a, -a), (0, 0, 0)), t) == zero) != (t[1] == t[2]))
    emit("F-X5.same_sign.cases", len(same))
    emit("F-X5.same_sign.violations", same_bad)
    emit("F-X5.cancelling.cases", len(nonzero))
    emit("F-X5.cancelling.violations", cancel_bad)
    each_cases = each_bad = 0
    for cols in permutations(range(n)):
        for vals in product((-2, -1, 1, 2), repeat=n):
            dd = tuple(tuple(vals[i] if j == cols[i] else 0 for j in range(n)) for i in range(n))
            each_cases += 1
            each_bad += sum(1 for t in all_tosses if (apply(dd, t) == zero) != (t == zero))
    emit("F-X5.one_per_row_and_column.cases", each_cases)
    emit("F-X5.one_per_row_and_column.violations", each_bad)
    three = shapes[PROOF_SHAPE]
    emit("F-X5.three_in_a_row.wrong_in_every_column", all(any(row[j] for row in three) for j in range(n)))

    row, column = next((i, j) for i in range(n) for j in range(n) if three[i][j])
    pairs = [(t, t[:column] + (1,) + t[column + 1:]) for t in all_tosses if t[column] == 0]
    tails = [apply(three, t)[row] for t, _ in pairs]
    heads = [apply(three, u)[row] for _, u in pairs]
    per_pair = [sum(1 for x in pair if apply(three, x) == zero) for pair in pairs]
    emit("F-F8.row", row + 1)
    emit("F-F8.column", column + 1)
    emit("F-F8.wrong_entry", three[row][column])
    emit("F-F8.pairs", ",".join(f"{toss_text(t)}-{toss_text(u)}" for t, u in pairs))
    emit("F-F8.row_sums.tails", csv(tails))
    emit("F-F8.row_sums.heads", csv(heads))
    emit("F-F8.changes", csv(h - t for t, h in zip(tails, heads)))
    emit("F-F8.misses_per_pair", csv(per_pair))
    emit("F-F8.misses_per_pair_bound", MISSES_PER_PAIR)
    emit("F-F8.most_misses_per_pair", max(per_pair))
    emit("F-F8.pair_count", len(pairs))
    emit("F-F8.partners_per_toss", csv(sorted({sum(1 for pair in pairs if t in pair) for t in all_tosses})))
    emit("F-F8.pairs_over_tosses", Fraction(len(pairs), len(all_tosses)))
    check("chapter 02: every edge sentence holds", same_bad == cancel_bad == each_bad == 0)
    check("chapter 02: each flip moves the row sum by the wrong entry, so no pair holds more than the proof's bound",
          all(h - t == three[row][column] for t, h in zip(tails, heads)) and max(per_pair) <= MISSES_PER_PAIR)

    # the hypothesis matters: paired by the coin of a column where C is
    # right, the slip's row sum never moves and both tosses of a pair miss
    (slip_row, _), = [(i, j) for i in range(n) for j in range(n) if slip[i][j]]
    right = next(j for j in range(n) if all(r[j] == 0 for r in slip))
    right_pairs = [(t, t[:right] + (1,) + t[right + 1:]) for t in all_tosses if t[right] == 0]
    right_misses = [sum(1 for x in pair if apply(slip, x) == zero) for pair in right_pairs]
    emit("F-X5.right_column.column", right + 1)
    emit("F-X5.right_column.misses_per_pair", csv(right_misses))
    emit("F-X5.right_column.changes", csv(apply(slip, u)[slip_row] - apply(slip, t)[slip_row] for t, u in right_pairs))
    check("chapter 02: paired by a column where C is right, a pair misses twice",
          max(right_misses) > MISSES_PER_PAIR)


TOSS_NAMES = ("first toss", "second toss")


def passes(c, t):
    """The check of c on toss t, by multiplying: A(B r) = C r."""
    return apply(A, apply(B, t)) == apply(c, t)


def chapter_03(ab, passing):
    """Chapter 03's facts: two tosses as every ordered way, one toss reused,
    and the blind spot of every toss."""
    n = len(A)
    all_tosses = tosses(n)
    misses = [t for t in all_tosses if t in passing]
    order = misses + [t for t in all_tosses if t not in passing]
    emit("F-F9.tosses", len(TOSS_NAMES))
    emit("F-F9.order", ",".join(toss_text(t) for t in order))
    emit("F-F9.first.misses", len(misses))
    emit("F-F9.second.misses", len(misses))
    ways = list(product(all_tosses, repeat=len(TOSS_NAMES)))
    both = {w for w in ways if all(passes(C, t) for t in w)}
    emit("F-F9.both_miss_is_product", len(both) == len(misses) ** 2)
    m = len(misses)
    emit("F-F9.both_miss_fill_the_corner",
         all(((s, t) in both) == (i < m and j < m) for i, s in enumerate(order) for j, t in enumerate(order)))
    two_bound = PAIRING_BOUND ** len(TOSS_NAMES)
    emit("F-F9.two.bound", two_bound)
    emit("F-F9.two.example_meets_bound", Fraction(len(both), len(ways)) == two_bound)
    diagonal = [(t, t) for t in all_tosses]
    reused = sum(1 for w in diagonal if w in both)
    emit("F-F9.same_toss.ways", len(diagonal))
    emit("F-F9.same_toss.above_two_bound", Fraction(reused, len(diagonal)) > two_bound)

    # the blind spot of each toss: a 1 in row 1 under the first tails coin,
    # and under all heads a 1 and a -1 in row 1 (the Rust's construction)
    spots = []
    for t in all_tosses:
        dd = [[0] * n for _ in range(n)]
        if 0 in t:
            dd[0][t.index(0)] = 1
        else:
            dd[0][0], dd[0][1] = 1, -1
        spots.append((t, tuple(tuple(row) for row in dd)))
    for t, dd in spots:
        emit(f"F-X3.blind_spot.{toss_text(t)}.mistake", matrix_text(dd))
    with_tails = [(t, dd) for t, dd in spots if 0 in t]
    emit("F-X3.blind_spot.tails.wrong_entries",
         csv(sorted({sum(1 for row in dd for x in row if x) for _, dd in with_tails})))
    emit("F-X3.blind_spot.tails.under_tails",
         all(t[j] == 0 for t, dd in with_tails for row in dd for j in range(n) if row[j]))
    heads = next(dd for t, dd in spots if 0 not in t)
    emit("F-X3.blind_spot.all_heads.values", csv(x for row in heads for x in row if x))
    emit("F-X3.blind_spot.all_heads.one_row", len({i for i in range(n) for j in range(n) if heads[i][j]}) == 1)
    catches = [sum(1 for s in all_tosses if not passes(plus(ab, dd), s)) for _, dd in spots]
    emit("F-X3.blind_spot.fresh_catches", csv(catches))
    emit("F-X3.blind_spot.fresh_catch_share", ",".join(sorted({str(Fraction(c, len(all_tosses))) for c in catches})))
    # the edge in size: search every small mistake at 2 × 2, and at 1 × 1
    two = tosses(2)
    small2 = [dd for dd in product(product((-1, 0, 1), repeat=2), repeat=2) if any(x for row in dd for x in row)]
    emit("F-X3.n2.fooled", sum(1 for t in two if any(all(x == 0 for x in apply(dd, t)) for dd in small2)))
    emit("F-X3.n2.tosses", len(two))
    emit("F-X3.n1.heads_fooled", any(apply(((x,),), (1,)) == (0,) for x in range(-3, 4) if x))
    check("chapter 03: both tosses miss in exactly the misses of each, and two fresh tosses meet (1/2)^2",
          len(both) == m * m and Fraction(len(both), len(ways)) == two_bound and reused == m)
    # the shown counterexamples, exactly: each caught by exactly half the
    # fresh tosses (the general at-most-half bound is the pairing checks')
    check("chapter 03: every toss lets its blind spot through, and fresh tosses catch each exactly half the time",
          all(passes(plus(ab, dd), t) for t, dd in spots) and all(2 * c == len(all_tosses) for c in catches))


def whole(column):
    """A column of floats that hold whole numbers, as ints (exact)."""
    return tuple(int(x) for x in column)


def chapter_04():
    """Chapter 04's facts: doubles at N = 10^16 from Python floats, and the
    two rounded examples by the explicit loop against exact integers."""
    big = BIG
    near = float(big)
    emit("F-X4.big", big)
    emit("F-X4.big_exponent", len(str(big)) - 1)
    emit("F-X4.plus_one.exact", big + 1)
    emit("F-X4.plus_one.double", int(near + 1.0))
    emit("F-X4.lost", big + 1 - int(near + 1.0))
    emit("F-X4.gap", int(math.nextafter(near, math.inf) - near))
    emit("F-X4.doubles_near", csv(k for k in range(-2, 3) if int(float(big + k)) == big + k))
    emit("F-X4.significand_bits", sys.float_info.mant_dig)
    emit("F-X4.big_above_2_53", big > 2 ** sys.float_info.mant_dig)

    two = tosses(2)
    a, b, c = ROUNDED_REJECTS
    failing = [t for t in two if rounded_apply(a, rounded_apply(b, t)) != rounded_apply(c, t)]
    (shown,) = failing
    br, br_d = apply(b, shown), whole(rounded_apply(b, shown))
    emit("F-X4.reject.br.exact", csv(br))
    emit("F-X4.reject.br.double", csv(br_d))
    emit("F-X4.reject.a_br.exact", csv(apply(a, br)))
    emit("F-X4.reject.a_br.double", csv(whole(rounded_apply(a, rounded_apply(b, shown)))))
    emit("F-X4.reject.cr.double", csv(whole(rounded_apply(c, shown))))
    emit("F-X4.reject.lost", sum(x - y for x, y in zip(br, br_d)))

    def verdicts(a, b, c):
        exact = [apply(a, apply(b, t)) == apply(c, t) for t in two]
        doubled = [rounded_apply(a, rounded_apply(b, t)) == rounded_apply(c, t) for t in two]
        return ["true" if v else "false" for v in exact], ["true" if v else "false" for v in doubled]

    exact, doubled = verdicts(a, b, c)
    emit("F-X4.reject.exact_matches", ",".join(exact))
    emit("F-X4.reject.double_matches", ",".join(doubled))

    a, b, c = ROUNDED_PASSES
    shown = next(t for t in two if apply(a, apply(b, t)) != apply(c, t)
                 and rounded_apply(a, rounded_apply(b, t)) == rounded_apply(c, t))
    emit("F-X4.pass.shown_toss", toss_text(shown))
    emit("F-X4.pass.a_br.exact", csv(apply(a, apply(b, shown))))
    emit("F-X4.pass.a_br.double", csv(whole(rounded_apply(a, rounded_apply(b, shown)))))
    emit("F-X4.pass.cr.double", csv(whole(rounded_apply(c, shown))))
    mistake = minus(c, row_times_column(a, b))
    (value,) = [x for row in mistake for x in row if x]
    emit("F-X4.pass.mistake", value)
    exact, doubled = verdicts(a, b, c)
    emit("F-X4.pass.exact_matches", ",".join(exact))
    emit("F-X4.pass.double_matches", ",".join(doubled))
    emit("F-X4.pass.exact_catches", exact.count("false"))
    # the tolerance: the right C's largest gap in doubles, the smallest that
    # lets it through; with it, the wrong C in exact sums
    ra, rb, rc = ROUNDED_REJECTS
    right = [max(abs(x - y) for x, y in zip(whole(rounded_apply(ra, rounded_apply(rb, t))), whole(rounded_apply(rc, t))))
             for t in two]
    wrong = [max(abs(x - y) for x, y in zip(apply(a, apply(b, t)), apply(c, t))) for t in two]
    tolerance = max(right)
    emit("F-X4.tolerance", tolerance)
    emit("F-X4.tolerance.right_residuals", csv(right))
    emit("F-X4.tolerance.right_passes", sum(1 for r in right if r <= tolerance))
    emit("F-X4.tolerance.wrong_residuals", csv(wrong))
    emit("F-X4.tolerance.wrong_passes", sum(1 for r in wrong if r <= tolerance))
    check("chapter 04: the smallest tolerance that lets the right C through in doubles also passes the wrong C on "
          "every toss in exact sums, and one less would still catch the right C",
          all(r <= tolerance for r in right) and all(r <= tolerance for r in wrong)
          and any(r > tolerance - 1 for r in right) and any(r > 0 for r in wrong))
    check("chapter 04: in doubles N + 1 is N, the right C fails one toss and the wrong C passes all four, "
          "while exact sums keep both promises",
          int(near + 1.0) == big and len(failing) == 1 and doubled.count("true") == len(two)
          and all(apply(ROUNDED_REJECTS[0], apply(ROUNDED_REJECTS[1], t)) == apply(ROUNDED_REJECTS[2], t) for t in two)
          and exact.count("true") * 2 <= len(two))


def cross_checks(ab, d, passing):
    n = len(A)
    all_tosses = tosses(n)
    check("A·B by outer products equals A·B row times column", ab == row_times_column(A, B))
    check("the product's nine entries are the digits 1 to 9", sorted(x for row in ab for x in row) == list(range(1, 10)))
    check("a toss passes exactly when the mistake sends it to zero",
          all((apply(A, apply(B, t)) == apply(C, t)) == all(x == 0 for x in apply(d, t)) for t in all_tosses))
    check("C = A·B plus the first shape", C == plus(ab, SHAPES[0][1]))
    # adding the heads columns is multiplying by the toss
    kept = all(tuple(sum(row[j] for j in range(n) if t[j]) for row in m) == apply(m, t)
               for m in (A, B, C) for t in all_tosses)
    check("adding the heads columns is multiplying by the 0/1 column", kept)
    # one wrong entry anywhere, of any size, hides from exactly half
    single = []
    for i, j, value in product(range(n), range(n), (-3, -1, 1, 5)):
        dd = tuple(tuple(value if (p, q) == (i, j) else 0 for q in range(n)) for p in range(n))
        single.append(sum(1 for t in all_tosses if all(x == 0 for x in apply(dd, t))))
    check("every one-entry mistake hides from exactly 4 of 8 tosses", set(single) == {4})
    # a right answer passes every toss, on a family of small integer matrices
    right = True
    for x in product((-1, 0, 2), repeat=4):
        for y in product((1, -2, 0), repeat=4):
            a2 = (x[0:2], x[2:4])
            b2 = (y[0:2], y[2:4])
            p2 = row_times_column(a2, b2)
            right &= all(apply(a2, apply(b2, t)) == apply(p2, t) for t in tosses(2))
    check("a right answer passes every toss (6561 small products)", right)
    # the n = 2 sweep through the real check agrees with the row solution sets
    real = 0
    ab2 = row_times_column(A2, B2)
    for dd in product(product((-2, -1, 0, 1, 2), repeat=2), repeat=2):
        if not any(x for row in dd for x in row):
            continue
        c2 = plus(ab2, dd)
        real = max(real, sum(1 for t in tosses(2) if apply(A2, apply(B2, t)) == apply(c2, t)))
    check("n = 2: the real check's worst mistake passes 2 of 4", real == 2)
    # the rounding itself
    check("doubles: 1e16 + 1 rounds to 1e16", float(BIG) + 1.0 == float(BIG))
    check("doubles: 1e16 + 2 does not", float(BIG) + 2.0 != float(BIG))
    check("the misses are 000, 001, 100, 101", ["".join(map(str, t)) for t in passing] == ["000", "001", "100", "101"])


def main():
    rows = facts()
    cross_checks(*rows)
    print(f"# {_CHECKS[0] - len(_FAILED)}/{_CHECKS[0]} cross-checks passed", file=sys.stderr)
    return 1 if _FAILED else 0


if __name__ == "__main__":
    sys.exit(main())
