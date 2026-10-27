#!/usr/bin/env python3
"""Facts oracle for "Three Keys Open the Secret. Two Reveal Nothing."

Run from this folder (standard library only, no numpy):

    python3 -I -B oracle.py               # KEY<TAB>VALUE facts on stdout
    python3 -I -B oracle.py --self-test   # edge tests and negative controls

stdout: one `KEY<TAB>VALUE` line per fact, the keys `vault facts` prints
(src/facts.rs): S-F rows are computed, S-X rows are the primary source's
own words and numbers (quoted in docs/brief.md).

stderr: cross-checks (lines starting with "#") and a summary; exit 1 if any
cross-check failed.

How the values are obtained, independently of the Rust model (which
evaluates a recipe by Horner's rule, divides with inverses from the extended
Euclidean algorithm and rebuilds the secret with Lagrange's weights):
  * a share is the definition written out, (s + a*x + b*x*x) % p, with
    Python's integers;
  * division on the dial is a search: the inverse of d is the mark k with
    d*k landing on 1, found by trying every mark;
  * rebuilding from three shares is a search over all p**3 recipes (s, a, b)
    for the ones that deal those shares; the scene's weights are found the
    same way, as the value at zero of the one recipe dealing 1 to one
    trustee and 0 to the other two;
  * the ordinary-number weights (10/3 and so on) are solved from the three
    equations by Gaussian elimination over fractions, not from the product
    formula the video shows (the two are compared); the paper sums of
    chapters 15 and 16 use the heights written out, before any wrapping;
  * "three times the secret lands on 4" (chapters 17 and 18) is undone by
    trying every mark;
  * "two shares reveal nothing" is counted: for each secret, the spin pairs
    out of all p**2 that deal the two shares seen; chapter 11's table over
    every pair, every two shares and every third share deals all p**3
    recipes and groups them by the shares they deal;
  * the padlock scheme is built as one lock per pair and every group of
    trustees is tried against every lock; Shamir's 462 and 252 are
    math.comb(11, 5) and math.comb(10, 5), pinned to the paper's text;
  * the ten-mark dial's failures are found by the same counts with p = 10;
    chapter 20's recipe that is zero at three trustees is the difference of
    two recipes dealing them the same shares;
  * a liar among five (chapter 22) is found by trying every recipe against
    the five shares handed in.

Assumptions (also in docs/brief.md): the secret is one mark of an
eleven-mark dial; trustees sit at marks 1 to 5 (Shamir's D_i = q(i)); the
spins are fair and independent, every mark equally likely, zero included
(Shamir: "randomly chosen from a uniform distribution over the integers in
[0, p)"); the dealer's two spins in the worked example are chosen for the
picture (7 and 7), and every claim about privacy is checked for every spin
pair, not only that one.
"""

import itertools
import math
import sys
from fractions import Fraction

# ---------------------------------------------------------------- the video
P = 11                      # marks on the vault's dial, 0 to 10
TEN = 10                    # the ordinary ten-digit dial, the negative control
TRUSTEES = (1, 2, 3, 4, 5)  # each trustee's mark (Shamir's x = i)
THRESHOLD = 3
SECRET = 5
SPINS = (7, 7)              # the worked example's two spins: times x, times x squared
OPENING_TRIO = (1, 2, 3)
PAIR = (4, 5)
SECOND_TRIO = (2, 4, 5)
# chapter numbers below are the full draft's (docs/storyboard-draft.md)
A_SHARE = 9                 # chapter 03: trustee A's spin
LINE_SLOPE = 4              # chapters 07 and 08: the two-of-five line's spin
LINE_HOLDER = 2             # chapter 08: the one trustee alone
ADJACENT = (2, 3)           # chapter 08: two neighbouring trustees
TEN_HOLDER = 5              # chapter 07 on paper, chapter 13 on ten marks: the narrowest share
TEN_PARITY_HOLDER = 2       # chapter 13: the share that gives away odd or even
OTHER_SPINS = (0, 1)        # chapter 17: another dealing, whose thirds do not cancel
LIAR = (3, 2)               # chapter 22: trustee 3 hands in 2 (the true share is 1)
FOURTH = 4                  # chapter 22: the fourth trustee whose share shows a disagreement
ORDINARY_PIECES = (2, 3)    # chapter 02: A's and B's ordinary pieces of the secret
JOINER = 1                  # chapter 11: the third trustee who joins the pair
ASKED = (0, 1)              # chapters 10 and 11: the secrets asked about after the dealer's

# Shamir 1979, section 1: the problem is quoted from Liu (1968); the counts
# are Shamir's own sentence ("the minimal solution uses 462 locks and 252
# keys per scientist")
SHAMIR_LOCKS = 462
SHAMIR_KEYS = 252
SHAMIR_SCIENTISTS = 11
SHAMIR_NEEDED = 6

FAILURES = []


def check(ok, what):
    if not ok:
        FAILURES.append(what)
        print(f"# FAIL {what}", file=sys.stderr)


def csv(xs):
    return ",".join(str(x) for x in xs)


def share(recipe, x, p=P):
    """The definition: secret + a*x + b*x*x, wrapped onto a p-mark dial."""
    s, a, b = recipe
    return (s + a * x + b * x * x) % p


def line_share(s, a, x, p=P):
    return (s + a * x) % p


def inverse(d, p=P):
    """The mark k with d*k landing on 1, by trying every mark; None if no mark does."""
    hits = [k for k in range(p) if (d * k) % p == 1]
    return hits[0] if len(hits) == 1 else None


def recipes(p=P):
    return itertools.product(range(p), repeat=3)


def rebuild(marks, values, p=P):
    """Every secret of every recipe dealing `values` at `marks` (a search)."""
    return sorted({r[0] for r in recipes(p) if all(share(r, x, p) == y for x, y in zip(marks, values))})


def unit_weight(marks, i, p=P):
    """The value at zero of the one recipe dealing 1 at marks[i] and 0 at the others."""
    want = [1 if j == i else 0 for j in range(len(marks))]
    found = [r for r in recipes(p) if all(share(r, x, p) == y for x, y in zip(marks, want))]
    if len(found) != 1:
        return None
    return found[0][0]


def rational_weights(marks):
    """Solve s + a*x + b*x*x = e_i at the three marks over fractions; return each s."""
    out = []
    for i in range(len(marks)):
        rows = [[Fraction(1), Fraction(x), Fraction(x * x), Fraction(1 if j == i else 0)] for j, x in enumerate(marks)]
        n = 3
        for c in range(n):
            pivot = next(r for r in range(c, n) if rows[r][c] != 0)
            rows[c], rows[pivot] = rows[pivot], rows[c]
            for r in range(n):
                if r != c and rows[r][c] != 0:
                    f = rows[r][c] / rows[c][c]
                    rows[r] = [u - f * w for u, w in zip(rows[r], rows[c])]
        out.append(rows[0][3] / rows[0][0])
    return out


def frac(q):
    q = Fraction(q)
    return str(q.numerator) if q.denominator == 1 else f"{q.numerator}/{q.denominator}"


def wind(h, p=P):
    """The whole turns a height makes round a p-mark dial and the mark it lands on,
    by taking p away until what is left is under p (chapters 08 and 09)."""
    check(h >= 0, f"a height {h} below zero")
    turns = 0
    while h >= p:
        h -= p
        turns += 1
    return turns, h


def paper_curve(points):
    """The ordinary curve s + a*x + b*x*x through three (x, height) points, solved by
    Gaussian elimination over fractions: (s, a, b)."""
    rows = [[Fraction(1), Fraction(x), Fraction(x * x), Fraction(h)] for x, h in points]
    n = 3
    for c in range(n):
        pivot = next(r for r in range(c, n) if rows[r][c] != 0)
        rows[c], rows[pivot] = rows[pivot], rows[c]
        for r in range(n):
            if r != c and rows[r][c] != 0:
                f = rows[r][c] / rows[c][c]
                rows[r] = [u - f * w for u, w in zip(rows[r], rows[c])]
    return tuple(rows[i][3] / rows[i][i] for i in range(n))


def fitting_spins(marks, values, s, p=P):
    """The spin pairs (a, b) that, with secret s, deal `values` at `marks`."""
    return [(a, b) for a in range(p) for b in range(p) if all(share((s, a, b), x, p) == y for x, y in zip(marks, values))]


def pair_learns_nothing(p, marks, spin_ok=lambda a, b: True):
    """For every pair of shares two trustees could see: is every secret dealt by the
    same number of allowed spin pairs? Returns (ok, the first leak found)."""
    allowed = [(a, b) for a in range(p) for b in range(p) if spin_ok(a, b)]
    for values in itertools.product(range(p), repeat=len(marks)):
        counts = [sum(1 for a, b in allowed if all(share((s, a, b), x, p) == y for x, y in zip(marks, values)))
                  for s in range(p)]
        if len(set(counts)) != 1:
            return False, (values, counts)
    return True, None


def zeros(recipe, p):
    return sum(1 for x in range(p) if share(recipe, x, p) == 0)


def losses_to_shut(holders):
    """The fewest trustees whose loss leaves nobody holding a copy (a search over groups)."""
    return min(k for k in range(len(TRUSTEES) + 1)
               for lost in itertools.combinations(TRUSTEES, k) if not set(holders) - set(lost))


def bribes_to_open(holders):
    """The fewest trustees who, bribed, include someone holding a copy (a search over
    groups); None when no group does (nobody holds a copy)."""
    return min((k for k in range(len(TRUSTEES) + 1)
                for bribed in itertools.combinations(TRUSTEES, k) if set(holders) & set(bribed)), default=None)


def fewest_locks(people, k):
    """The fewest locks of any plan (a lock is the set of people holding its key) with
    which every k of `people` open them all and every k - 1 are stopped by one: plans
    are tried smallest first. None if none works."""
    holder_sets = [set(s) for r in range(len(people) + 1) for s in itertools.combinations(people, r)]
    usable = [h for h in holder_sets if all(h & set(g) for g in itertools.combinations(people, k))]
    stop = [set(g) for g in itertools.combinations(people, k - 1)]
    for n in range(1, len(usable) + 1):
        if any(all(any(not (h & g) for h in plan) for g in stop) for plan in itertools.combinations(usable, n)):
            return n
    return None


def facts():
    f = {}

    def put(key, value):
        if key in f:
            raise SystemExit(f"oracle: {key} twice")
        f[key] = str(value).lower() if isinstance(value, bool) else str(value)

    recipe = (SECRET, *SPINS)
    shares = [share(recipe, x) for x in TRUSTEES]

    # 01_the_vault and 02_cutting_it_up: one keeper, copies, and ordinary pieces that add up
    put("S-F0.copies.who_can_open_alone", sum(1 for _ in TRUSTEES))
    put("S-F0.copies.secrets_left_for_one", len([SECRET]))
    # a plan that hands the secret itself to `holders`: the fewest trustees whose loss
    # leaves no copy, and the fewest who, bribed, give it away (searched over groups)
    keeper = (TRUSTEES[0],)
    put("S-F0.keepers", len(keeper))
    put("S-F0.keeper.losses_to_shut", losses_to_shut(keeper))
    put("S-F0.keeper.bribes_to_open", bribes_to_open(keeper))
    put("S-F0.copies.losses_to_shut", losses_to_shut(TRUSTEES))
    put("S-F0.copies.bribes_to_open", bribes_to_open(TRUSTEES))
    put("S-F0.pieces", csv(ORDINARY_PIECES))
    put("S-F0.pieces.a", ORDINARY_PIECES[0])
    put("S-F0.pieces.b", ORDINARY_PIECES[1])
    put("S-F0.pieces.sum", sum(ORDINARY_PIECES))
    check(sum(ORDINARY_PIECES) == SECRET, "the ordinary pieces add up to the secret")
    # ordinary whole numbers, 0 or more: a piece b fits secret s only if s - b is a piece too
    b_piece = ORDINARY_PIECES[1]
    b_fits = [s for s in range(P) if s - b_piece >= 0]
    put("S-F0.pieces.b_alone_candidates", csv(b_fits))
    put("S-F0.pieces.b_alone_left", len(b_fits))
    put("S-F0.pieces.b_alone_ruled_out", csv(s for s in range(P) if s not in b_fits))
    put("S-F0.pieces.both_needed", len(ORDINARY_PIECES))

    # 01_the_vault: the trustees, the threshold, the dial
    put("S-F1.trustees", len(TRUSTEES))
    put("S-F1.threshold", THRESHOLD)
    put("S-F1.too_few", THRESHOLD - 1)
    # the most trustees who can be away, whichever they are, with a threshold still present
    put("S-F1.can_be_away", max(k for k in range(len(TRUSTEES) + 1)
                                if all(len(set(TRUSTEES) - set(away)) >= THRESHOLD
                                       for away in itertools.combinations(TRUSTEES, k))))
    put("S-F1.marks", P)
    put("S-F1.opening_marks", sum(1 for m in range(P) if m == SECRET))
    put("S-F1.secret", SECRET)
    put("S-F1.opening_trio", csv(OPENING_TRIO))
    put("S-F1.opening_shares", csv(shares[x - 1] for x in OPENING_TRIO))
    put("S-F1.pair", csv(PAIR))
    put("S-F1.pair_shares", csv(shares[x - 1] for x in PAIR))
    check(sorted(OPENING_TRIO + PAIR) == list(TRUSTEES), "the pair is the trio's complement")

    # 03_the_dial: one share alone (two of two)
    b_share = next(k for k in range(P) if (A_SHARE + k) % P == SECRET)
    put("S-F2.top", P - 1)
    put("S-F2.a_share", A_SHARE)
    put("S-F2.b_share", b_share)
    put("S-F2.sum_unwrapped", A_SHARE + b_share)
    put("S-F2.sum", (A_SHARE + b_share) % P)
    by_a = [(a + b_share) % P for a in range(P)]
    put("S-F2.turn_back", (SECRET - b_share) % P)
    put("S-F2.ten_plus_one", (P - 1 + 1) % P)
    put("S-F2.secret_if_a_held", csv(by_a))
    put("S-F2.if_a_held.0", by_a[0])
    put("S-F2.if_a_held.1", by_a[1])
    put("S-F2.distinct_secrets", len(set(by_a)))
    put("S-F2.ways_each", max(by_a.count(s) for s in range(P)))
    put("S-F2.chance_each", frac(Fraction(1, P)))

    # 06_padlocks: one per pair, its key with everyone outside the pair
    pairs = list(itertools.combinations(TRUSTEES, 2))
    trios = list(itertools.combinations(TRUSTEES, 3))
    keys = {t: [lk for lk in pairs if t not in lk] for t in TRUSTEES}
    opens = lambda group: all(any(lk in keys[t] for t in group) for lk in pairs)  # noqa: E731
    stuck_at = lambda group: sum(1 for lk in pairs if not any(lk in keys[t] for t in group))  # noqa: E731
    put("S-F3.locks", len(pairs))
    put("S-F3.keys_each", csv(sorted({len(v) for v in keys.values()})))
    put("S-F3.trios", len(trios))
    put("S-F3.trios_that_open", sum(1 for g in trios if opens(g)))
    put("S-F3.pairs", len(pairs))
    put("S-F3.pairs_stuck", sum(1 for g in pairs if not opens(g)))
    put("S-F3.locks_a_pair_cannot_open", csv(sorted({stuck_at(g) for g in pairs})))
    put("S-F3.keys_total", sum(len(v) for v in keys.values()))
    put("S-F3.shamir.scientists", SHAMIR_SCIENTISTS)
    put("S-F3.shamir.needed", SHAMIR_NEEDED)
    put("S-F3.shamir.locks", math.comb(SHAMIR_SCIENTISTS, SHAMIR_NEEDED - 1))
    put("S-F3.shamir.keys_each", math.comb(SHAMIR_SCIENTISTS - 1, SHAMIR_NEEDED - 1))
    check(math.comb(11, 5) == SHAMIR_LOCKS and math.comb(10, 5) == SHAMIR_KEYS, "Shamir's 462 and 252")
    # "no five can": the most scientists who must stay locked out
    put("S-F3.shamir.too_few", max(k for k in range(SHAMIR_SCIENTISTS + 1) if k < SHAMIR_NEEDED))
    # one lock for each pair, and the lock for the pair has its keys with the three outside it
    put("S-F3.locks_per_pair", csv(sorted({sum(1 for lk in pairs if set(lk) == set(g)) for g in pairs})))
    put("S-F3.pair_lock.holders", csv(t for t in TRUSTEES if tuple(PAIR) in keys[t]))
    # a lock stopping two different pairs keeps its key from everyone in either
    two_pairs = list(itertools.combinations(pairs, 2))
    put("S-F3.merged.pairs", len(two_pairs[0]))
    put("S-F3.merged.unions", csv(sorted({len(set(p) | set(q)) for p, q in two_pairs})))
    check(all(any(set(tr) <= set(p) | set(q) for tr in trios) for p, q in two_pairs),
          "every lock serving two pairs shuts out a trio")
    put("S-F3.fewest_locks", fewest_locks(TRUSTEES, THRESHOLD))
    check(fewest_locks(TRUSTEES, 2) == len(TRUSTEES), "any two of five needs a lock per trustee")
    # the dealer's method: one number for each trustee
    put("S-F3.dealer.numbers_each", len(shares) // len(TRUSTEES))

    # 07, 08: a line through the secret (any two of five)
    put("S-F4.slope", LINE_SLOPE)
    put("S-F4.heights", csv(SECRET + LINE_SLOPE * x for x in TRUSTEES))
    line = [line_share(SECRET, LINE_SLOPE, x) for x in TRUSTEES]
    put("S-F4.shares", csv(line))
    put("S-F4.times2", csv((2 * x) % P for x in range(P)))
    # on ordinary paper (no wrapping), with secret and spin each a mark 0 to 10
    tall = SECRET + LINE_SLOPE * TEN_HOLDER
    put("S-F4.paper.holder", TEN_HOLDER)
    put("S-F4.paper.share", tall)
    put("S-F4.paper.candidates", csv(sorted({s for s in range(P) for a in range(P) if s + a * TEN_HOLDER == tall})))
    put("S-F4.paper.count", len({s for s in range(P) for a in range(P) if s + a * TEN_HOLDER == tall}))

    # 07: the fewest trustees whose shares always leave one secret of the line, by search
    def line_left(xs):
        got = [line_share(SECRET, LINE_SLOPE, x) for x in xs]
        return {s for s in range(P) for a in range(P) if all(line_share(s, a, x) == y for x, y in zip(xs, got))}
    need = min(k for k in range(1, len(TRUSTEES) + 1)
               if all(len(line_left(xs)) == 1 for xs in itertools.combinations(TRUSTEES, k)))
    put("S-F4.needed", need)
    put("S-F4.too_few", need - 1)
    put("S-F4.spin_count", len((LINE_SLOPE,)))
    put("S-F4.secret_position", next(x for x in range(P) if SECRET + LINE_SLOPE * x == SECRET))
    # each trustee sits at the position of its number: its share is the height there
    for t in TRUSTEES[:2]:
        put(f"S-F4.position_of.{t}", next(x for x in range(P) if line[t - 1] == line_share(SECRET, LINE_SLOPE, x)))
    u, w = (line[x - 1] for x in ADJACENT)
    step = (w - u) % P
    put("S-F4.adjacent", csv(ADJACENT))
    put("S-F4.adjacent.shares", csv((u, w)))
    put("S-F4.adjacent.step", step)
    put("S-F4.adjacent.secret_unwrapped", u - ADJACENT[0] * step)
    put("S-F4.adjacent.secret", (u - ADJACENT[0] * step) % P)
    # 08: each height wound round the dial, eleven taken away at a time
    for t in TRUSTEES:
        h = SECRET + LINE_SLOPE * t
        turns, mark = wind(h)
        check(mark == line[t - 1], f"height {h} winds onto {mark}, not trustee {t}'s share")
        put(f"S-F4.wrap.{t}", csv((h, turns, mark)))
    put("S-F4.adjacent.apart", ADJACENT[1] - ADJACENT[0])
    # 08: step back from trustee 2, one position at a time: the mark that a turn on by
    # the step would carry onto where we are, by trying every mark
    here, walked = u, [u]
    for _ in range(ADJACENT[0], 0, -1):
        here = next(k for k in range(P) if (k + step) % P == here)
        walked.append(here)
    check(walked[-1] == SECRET, f"two steps back from {u} reach {walked[-1]}")
    put("S-F4.back.steps", len(walked) - 1)
    put("S-F4.back.from", ADJACENT[0])
    put("S-F4.back.1", csv((walked[0], step, walked[1])))
    put("S-F4.back.2", csv((walked[1], step, walked[2])))
    # 08: doubling every mark, and how often each mark is hit
    put("S-F4.times2.of", csv(range(P)))
    put("S-F4.times2.hits_each", csv(sorted({sum(1 for x in range(P) if (2 * x) % P == k) for k in range(P)})))

    # 08: one share on the dial
    held = line[LINE_HOLDER - 1]
    by_slope = [next(s for s in range(P) if line_share(s, a, LINE_HOLDER) == held) for a in range(P)]
    put("S-F5.holder", LINE_HOLDER)
    put("S-F5.share", held)
    put("S-F5.secret_by_slope", csv(by_slope))
    put("S-F5.distinct", len(set(by_slope)))
    put("S-F5.true_slope_secret", by_slope[LINE_SLOPE])
    # 08: how many secrets each spin allows, by search, and spins 0 and 1's
    per_spin = {len([s for s in range(P) if line_share(s, a, LINE_HOLDER) == held]) for a in range(P)}
    put("S-F5.per_spin", csv(sorted(per_spin)))
    put("S-F5.spin.0", by_slope[0])
    put("S-F5.spin.1", by_slope[1])
    check((held - LINE_HOLDER * 1) % P == by_slope[1], "spin 1: the share take away twice 1")
    put("S-F5.spin.1.working", csv((held, LINE_HOLDER * 1, by_slope[1])))

    # 09: a curve for three
    put("S-F6.spins", csv(SPINS))
    put("S-F6.heights", csv(SECRET + SPINS[0] * x + SPINS[1] * x * x for x in TRUSTEES))
    put("S-F6.shares", csv(shares))
    put("S-F6.spin_pairs", P * P)
    put("S-F6.spin_count", len(SPINS))
    # 09: trustees 1 and 2 term by term; the others' heights
    curve_heights = {x: SECRET + SPINS[0] * x + SPINS[1] * x * x for x in TRUSTEES}
    for x in TRUSTEES[:2]:
        terms = (SECRET, SPINS[0] * x, SPINS[1] * x * x)
        put(f"S-F6.terms.{x}", csv((x, *terms, sum(terms))))
    put("S-F6.rest", csv(TRUSTEES[2:]))
    put("S-F6.rest.heights", csv(curve_heights[x] for x in TRUSTEES[2:]))
    # 09: on paper, the curves through every three of the five points (Gaussian
    # elimination over fractions), each checked against all five
    paper_curves = set()
    for trio in itertools.combinations(TRUSTEES, THRESHOLD):
        s, a, b = paper_curve([(x, curve_heights[x]) for x in trio])
        check(all(s + a * x + b * x * x == h for x, h in curve_heights.items()),
              f"trustees {trio}' paper curve misses a point")
        paper_curves.add((s, a, b))
    check(paper_curves == {(SECRET, *SPINS)}, f"paper curves {paper_curves}")
    put("S-F6.paper.curves", len(paper_curves))
    # 09: trustee 2's 47 asked about, then wound round; every height's turns
    asked = TRUSTEES[1]
    put("S-F6.asked", csv((asked, curve_heights[asked])))
    turns, mark = wind(curve_heights[asked])
    check(mark == shares[asked - 1], f"{curve_heights[asked]} winds onto {mark}")
    put(f"S-F6.wrap.{asked}", csv((curve_heights[asked], turns, mark)))
    put("S-F6.turns", csv(wind(curve_heights[x])[0] for x in TRUSTEES))

    # 05, 10: two shares
    pv = [shares[x - 1] for x in PAIR]
    per_secret = [fitting_spins(PAIR, pv, s) for s in range(P)]
    put("S-F7.fitting", sum(len(x) for x in per_secret))
    put("S-F7.ways_each", csv(sorted({len(x) for x in per_secret})))
    put("S-F7.spins_by_secret", ",".join(f"{a}:{b}" for (a, b), in per_secret))
    put("S-F7.chance_each", frac(Fraction(1, P * P)))
    put("S-F7.true_spins", ":".join(str(v) for v in per_secret[SECRET][0]))
    line_secret = next(s for s in range(P) if per_secret[s][0][1] == 0)
    put("S-F7.straight_one", line_secret)
    # 04, 05: the secrets fed in, and those no spin pair deals the pair's shares with
    put("S-F7.secrets", csv(range(P)))
    put("S-F7.ruled_out", sum(1 for ways in per_secret if not ways))
    # 10: the pair's two points; the secrets asked about after the dealer's, their spins,
    # and the first one's heights at the pair written out, then wound round the dial; the
    # board: each secret's curve at positions 0 to 5, and the marks each position holds
    put("S-F7.pair.count", len(PAIR))
    put("S-F7.asked.first", ASKED[0])
    put("S-F7.asked.second", ASKED[1])
    for i, s in enumerate(ASKED):
        (a, b), = per_secret[s]
        put(f"S-F7.spins.{i}", csv((a, b)))
    (a, b), = per_secret[ASKED[0]]
    for x, y in zip(PAIR, pv):
        h = ASKED[0] + a * x + b * x * x
        check(wind(h)[1] == y, f"the secret {ASKED[0]}'s height {h} at {x} winds onto {y}")
        put(f"S-F7.check.{x}", csv((x, h, y)))
    positions = range(len(TRUSTEES) + 1)
    board = [[share((s, *per_secret[s][0]), x) for x in positions] for s in range(P)]
    for s, row in enumerate(board):
        put(f"S-F7.board.{s}", csv(row))
    put("S-F7.board.columns", csv(len({row[x] for row in board}) for x in positions))
    # the same pair on ordinary paper: a curve through every secret, but whole spins 0 to 10 for one
    tall_pair = [SECRET + SPINS[0] * x + SPINS[1] * x * x for x in PAIR]
    paper_curves = []
    for s in range(P):
        rows = [[Fraction(x), Fraction(x * x), Fraction(h - s)] for x, h in zip(PAIR, tall_pair)]
        det = rows[0][0] * rows[1][1] - rows[0][1] * rows[1][0]
        if det != 0:
            paper_curves.append((s, (rows[0][2] * rows[1][1] - rows[0][1] * rows[1][2]) / det,
                                 (rows[0][0] * rows[1][2] - rows[0][2] * rows[1][0]) / det))
    put("S-F7.paper.heights", csv(tall_pair))
    put("S-F7.paper.curves", len(paper_curves))
    whole = sorted({s for s in range(P) for a in range(P) for b in range(P)
                    if all(s + a * x + b * x * x == h for x, h in zip(PAIR, tall_pair))})
    put("S-F7.paper.whole_spin_secrets", csv(whole))
    check(whole == [s for s, a, b in paper_curves if a.denominator == b.denominator == 1 and 0 <= a < P and 0 <= b < P],
          "paper pair: search agrees with the solved curves")
    put("S-F7.paper.secret0_spins", csv(frac(q) for q in paper_curves[0][1:]))
    put("S-F7.paper.secret5_spins", csv(frac(q) for q in next(c for c in paper_curves if c[0] == SECRET)[1:]))
    # a third share joins: each of the eleven curves meets the joiner's position at its own mark
    at_joiner = [share((s, *per_secret[s][0]), JOINER) for s in range(P)]
    put("S-F7.join.trustee", JOINER)
    put("S-F7.join.share", shares[JOINER - 1])
    put("S-F7.join.values_by_secret", csv(at_joiner))
    put("S-F7.join.distinct", len(set(at_joiner)))
    put("S-F7.join.secrets_left", csv(s for s in range(P) if at_joiner[s] == shares[JOINER - 1]))
    joined = [s for s in range(P) if at_joiner[s] == shares[JOINER - 1]]
    put("S-F7.join.point", csv((JOINER, shares[JOINER - 1])))
    put("S-F7.join.count", len(joined))
    put("S-F7.join.drop", csv((sum(1 for ways in per_secret if ways), len(joined))))
    # 11: every pair of trustees, all 121 pairs of shares and every third trustee's every
    # share, by dealing all 1331 recipes: how many spin pairs each secret fits two shares
    # with, how many secrets two shares leave, and how many a third share leaves
    all_recipes = list(recipes())
    pairs = list(itertools.combinations(TRUSTEES, 2))
    share_pairs = list(itertools.product(range(P), repeat=2))
    ways, two_leave, third_leaves = set(), set(), set()
    for pair in pairs:
        by_two = {}
        for r in all_recipes:
            by_two.setdefault(tuple(share(r, x) for x in pair), []).append(r[0])
        for values in share_pairs:
            got = by_two.get(values, [])
            ways.update(got.count(s) for s in range(P))
            two_leave.add(len(set(got)))
        for j in TRUSTEES:
            if j in pair:
                continue
            by_three = {}
            for r in all_recipes:
                by_three.setdefault(tuple(share(r, x) for x in (*pair, j)), set()).add(r[0])
            for values in itertools.product(range(P), repeat=3):
                third_leaves.add(len(by_three.get(values, ())))
    put("S-F7.all_pairs.pairs", len(pairs))
    put("S-F7.all_pairs.share_pairs", len(share_pairs))
    put("S-F7.all_pairs.ways_each", csv(sorted(ways)))
    put("S-F7.all_pairs.two_leave", csv((len(PAIR), *sorted(two_leave))))
    put("S-F7.all_pairs.third_leaves", csv((THRESHOLD, *sorted(third_leaves))))

    # 12, 13: why eleven: the same recipe, and the line, on a ten-mark dial
    ten_curve = [share(recipe, x, TEN) for x in TRUSTEES]
    tpv = [ten_curve[x - 1] for x in PAIR]
    put("S-F8.ten.marks", TEN)
    put("S-F8.ten.shares", csv(ten_curve))
    put("S-F8.ten.pair_shares", csv(tpv))
    put("S-F8.ten.pair_secrets", csv(s for s in range(TEN) if fitting_spins(PAIR, tpv, s, TEN)))
    # the pair's shortcut, found by a search over small whole-number weights
    shortcut = [(u, w) for u in range(-9, 10) for w in range(-9, 10)
                if all((u * share(r, PAIR[0], TEN) + w * share(r, PAIR[1], TEN)) % TEN == r[0] for r in recipes(TEN))]
    # the smallest weights, a positive first one where two tie (-5 and 5 are one mark on ten)
    put("S-F8.ten.pair_formula", csv(min(shortcut, key=lambda uw: (abs(uw[0]) + abs(uw[1]), uw[0] < 0))))
    put("S-F8.ten.pair_formula_value", (5 * tpv[0] - 4 * tpv[1]))
    # in ordinary numbers 5*y4 - 4*y5 = s - 20*b: what is left of the -20 on each dial
    lost = [5 * PAIR[0] ** 2 - 4 * PAIR[1] ** 2, 5 * PAIR[0] - 4 * PAIR[1], 5 - 4]
    put("S-F8.pair_sum.spin_terms", csv(lost))
    put("S-F8.ten.leftover", lost[0] % TEN)
    put("S-F8.eleven.leftover", lost[0] % P)
    put("S-F8.eleven.pair_formula_unwrapped", 5 * pv[0] - 4 * pv[1])
    put("S-F8.eleven.pair_formula_value", (5 * pv[0] - 4 * pv[1]) % P)
    check(all((5 * share(r, 4) - 4 * share(r, 5)) % P == (r[0] + 2 * r[2]) % P for r in recipes()),
          "on eleven marks 5*y4 - 4*y5 is s + 2b")
    put("S-F8.ten.pairs_that_leak",
        sum(1 for m in itertools.combinations(TRUSTEES, 2) if not pair_learns_nothing(TEN, m)[0]))
    put("S-F8.ten.zero_product", csv((2, 5)))
    put("S-F8.ten.times5", csv((5 * x) % TEN for x in range(TEN)))
    put("S-F8.eleven.times5", csv((5 * x) % P for x in range(P)))
    put("S-F8.ten.rows_that_shuffle", csv(k for k in range(1, TEN) if len({(k * x) % TEN for x in range(TEN)}) == TEN))
    put("S-F8.eleven.rows_that_shuffle", sum(1 for k in range(1, P) if len({(k * x) % P for x in range(P)}) == P))
    primes = [q for q in range(2, 100) if all(q % d for d in range(2, q))]
    put("S-F8.smallest_prime_for_five", next(q for q in primes if q > len(TRUSTEES)))
    ten = [line_share(SECRET, LINE_SLOPE, x, TEN) for x in TRUSTEES]

    def ten_candidates(x):
        seen = ten[x - 1]
        return sorted({s for s in range(TEN) for a in range(TEN) if line_share(s, a, x, TEN) == seen})

    put("S-F8.line.shares", csv(ten))
    put("S-F8.line.holder", TEN_HOLDER)
    put("S-F8.line.holder_share", ten[TEN_HOLDER - 1])
    put("S-F8.line.candidates", csv(ten_candidates(TEN_HOLDER)))
    put("S-F8.line.parity_holder", TEN_PARITY_HOLDER)
    put("S-F8.line.parity_share", ten[TEN_PARITY_HOLDER - 1])
    put("S-F8.line.parity_candidates", csv(ten_candidates(TEN_PARITY_HOLDER)))
    leaky = [x for x in range(1, TEN) if len({(a * x) % TEN for a in range(TEN)}) < TEN]
    put("S-F8.line.leaky_marks", csv(leaky))

    # 12 (native): the pair's two tallies, by trying every spin pair for every secret
    ten_ways = [len(fitting_spins(PAIR, tpv, s, TEN)) for s in range(TEN)]
    eleven_ways = [len(fitting_spins(PAIR, pv, s)) for s in range(P)]
    put("S-F8.ten.pair_ways", csv(ten_ways))
    put("S-F8.eleven.pair_ways", csv(eleven_ways))
    put("S-F8.ten.pair_spin_pairs", sum(ten_ways))
    # 12_4, 12_5: each height's amounts of the secret, the first spin and the second; the
    # fewest copies of each that hold the same amount of the first spin, found by search
    amounts = {x: (1, x, x * x) for x in PAIR}
    for x in PAIR:
        put(f"S-F8.pair_sum.height.{x}", csv(amounts[x]))
    copies = next((c4, c5) for n in range(2, 99) for c4 in range(1, n) for c5 in [n - c4]
                  if c4 * amounts[PAIR[0]][1] == c5 * amounts[PAIR[1]][1])
    put(f"S-F8.pair_sum.copies.{PAIR[0]}", csv((copies[0], PAIR[0])))
    put(f"S-F8.pair_sum.copies.{PAIR[1]}", csv((copies[1], PAIR[1])))
    scaled = {x: [c * t for t in amounts[x]] for c, x in zip(copies, PAIR)}
    put("S-F8.pair_sum.first_spin", scaled[PAIR[0]][1])
    for x in PAIR:
        put(f"S-F8.pair_sum.scaled.{x}", csv(scaled[x]))
    left = [u - w for u, w in zip(scaled[PAIR[0]], scaled[PAIR[1]])]
    check(left == [1, 0, -20], f"the difference of the copies is s - 20b: {left}")
    put("S-F8.pair_sum.first_spin_left", left[1])
    # 12_6, 12_7: the -20 wound onto each dial, and the sum on the pair's cards term by term
    put("S-F8.ten.twenty_lands", csv((-left[2], -left[2] % TEN)))
    put("S-F8.ten.pair_formula_terms", csv((copies[0], tpv[0], -copies[1], tpv[1])))
    put("S-F8.ten.pair_formula_products", csv((copies[0] * tpv[0], -copies[1] * tpv[1])))
    put("S-F8.eleven.minus_twenty_lands", csv((left[2], left[2] % P)))
    put("S-F8.eleven.pair_formula_terms", csv((copies[0], pv[0], -copies[1], pv[1])))
    put("S-F8.eleven.pair_formula_products", csv((copies[0] * pv[0], -copies[1] * pv[1])))
    eight = (copies[0] * pv[0] - copies[1] * pv[1]) % P
    put("S-F8.eleven.second_spin_each",
        csv(sorted({sum(1 for b in range(P) if (s + (left[2] % P) * b) % P == eight) for s in range(P)})))
    # 12_8: every pair of trustees on ten marks, every pair of shares some recipe deals them
    # (all 1000 recipes): how many secrets are left, and whether those all fit equally often
    by_left, even = {}, 0
    for pair in itertools.combinations(TRUSTEES, 2):
        dealt_by = {}
        for r in recipes(TEN):
            dealt_by.setdefault(tuple(share(r, x, TEN) for x in pair), []).append(r[0])
        lefts = {len(set(v)) for v in dealt_by.values()}
        check(len(lefts) == 1, f"ten marks {pair}: what is left depends on the shares: {lefts}")
        by_left.setdefault(lefts.pop(), []).append(pair)
        even += all(len({v.count(s) for s in set(v)}) == 1 for v in dealt_by.values())
    for n, group in sorted(by_left.items()):
        put(f"S-F8.ten.pairs_left.{n}", csv(f"{a}-{b}" for a, b in group))
    put("S-F8.ten.pairs_that_find", len(by_left.get(1, [])))
    nothing = by_left.get(TEN, [])
    check(len(nothing) == 1, f"one pair learns nothing on ten marks: {nothing}")
    put("S-F8.ten.pair_that_learns_nothing", csv(nothing[0]))
    put("S-F8.ten.pairs_even_left", even)

    # 13 (native): multiplying on ten and eleven marks, every product tried
    zp = [(a, b) for a in range(2, TEN) for b in range(a, TEN) if a * b == TEN]
    check(zp == [(2, 5)], f"the factor pairs of ten: {zp}")
    put("S-F8.ten.zero_product_sum", csv((*zp[0], zp[0][0] * zp[0][1])))
    put("S-F8.ten.zero_product_mark", (zp[0][0] * zp[0][1]) % TEN)
    put("S-F8.multiplier", TEN_HOLDER)
    k = TEN_HOLDER
    ten_hits = [sum(1 for x in range(TEN) if (k * x) % TEN == m) for m in range(TEN)]
    eleven_hits = [sum(1 for x in range(P) if (k * x) % P == m) for m in range(P)]
    put("S-F8.ten.times5_values", csv(m for m in range(TEN) if ten_hits[m]))
    put("S-F8.ten.times5_reached", sum(1 for h in ten_hits if h))
    put("S-F8.ten.times5_hits", csv(ten_hits))
    put("S-F8.line.candidate_count", len(ten_candidates(TEN_HOLDER)))
    put("S-F8.eleven.times5_reached", sum(1 for h in eleven_hits if h))
    put("S-F8.eleven.times5_hits", csv(eleven_hits))
    put("S-F8.eleven.times5_each", csv(sorted(set(eleven_hits))))
    for x in (1, 2, 3):
        put(f"S-F8.eleven.times5_step.{x}", csv((x, k * x) if k * x < P else (x, k * x, (k * x) % P)))
    eleven_line = line_share(SECRET, LINE_SLOPE, TEN_HOLDER)
    put("S-F8.eleven.line_candidates",
        len({s for s in range(P) for a in range(P) if line_share(s, a, TEN_HOLDER) == eleven_line}))
    put("S-F8.ten.split", csv((TEN, *zp[0])))
    put("S-F8.eleven.factors", csv(d for d in range(1, P + 1) if P % d == 0))
    put("S-F8.eleven.zero_product_check", csv((*zp[0], (zp[0][0] * zp[0][1]) % P)))
    check(all((a * b) % P for a in range(1, P) for b in range(1, P)), "no two nonzero marks of eleven multiply to 0")
    put("S-F8.eleven.nonzero_range", csv((1, P - 1)))
    put("S-F8.ten.reach", csv(len({(m * x) % TEN for x in range(TEN)}) for m in range(1, TEN)))
    put("S-F8.eleven.reach", csv(len({(m * x) % P for x in range(P)}) for m in range(1, P)))
    put("S-F8.primes_for_five", csv([q for q in primes if q > len(TRUSTEES)][:3]))

    # 14, 15: three shares rebuild it (the opening trio)
    tv = [shares[x - 1] for x in OPENING_TRIO]
    put("S-F9.helper_at_own", csv(math.prod(x - y for y in OPENING_TRIO if y != x) for x in OPENING_TRIO))
    put("S-F9.helper_at_zero", csv(math.prod(0 - y for y in OPENING_TRIO if y != x) for x in OPENING_TRIO))
    rw = rational_weights(OPENING_TRIO)
    put("S-F9.weights", csv(frac(q) for q in rw))
    dw = [unit_weight(OPENING_TRIO, i) for i in range(3)]
    put("S-F9.weights_on_dial", csv(dw))
    total = sum(q * y for q, y in zip(rw, tv))
    # on paper, the same weights on the heights before wrapping give the secret itself
    tall_trio = [SECRET + SPINS[0] * x + SPINS[1] * x * x for x in OPENING_TRIO]
    put("S-F9.paper.heights", csv(tall_trio))
    put("S-F9.paper.terms", csv(frac(q * h) for q, h in zip(rw, tall_trio)))
    paper_total = sum(q * h for q, h in zip(rw, tall_trio))
    put("S-F9.paper.sum", frac(paper_total))
    check(paper_total == SECRET, "15 on paper the weights turn the heights into the secret")
    # each share is its height less whole elevens; whole weights keep the sums whole elevens apart
    put("S-F9.paper.elevens", csv((h - y) // P for h, y in zip(tall_trio, tv)))
    put("S-F9.sum_unwrapped", frac(total))
    put("S-F9.sum_minus_secret", frac(total - paper_total))
    check((total - paper_total).denominator == 1 and int(total - paper_total) % P == 0, "15 the two sums are whole elevens apart")
    put("S-F9.secret", csv(rebuild(OPENING_TRIO, tv)))
    check(all(q.denominator == 1 and int(q) % P == d for q, d in zip(rw, dw)), "15 weights wrap onto the dial weights")

    # 14 (native): each helper is the ordinary curve through 1 at its own trustee and 0 at
    # the other two (Gaussian elimination, not the product the scene writes); its zeros by
    # trying every whole position from -20 to 20; the scene's product of factors compared
    helpers = [paper_curve([(x, 1 if j == i else 0) for j, x in enumerate(OPENING_TRIO)]) for i in range(3)]

    def at(curve, x):
        return curve[0] + curve[1] * x + curve[2] * x * x

    for i, (x, curve) in enumerate(zip(OPENING_TRIO, helpers)):
        n = i + 1
        put(f"S-F9.helper.{n}.position", x)
        put(f"S-F9.helper.{n}.at_trio", csv(frac(at(curve, y)) for y in OPENING_TRIO))
        put(f"S-F9.helper.{n}.zeros", csv(y for y in range(-20, 21) if at(curve, y) == 0))
        factors = [x - y for y in OPENING_TRIO if y != x]
        put(f"S-F9.helper.{n}.factors_at_own", csv(factors))
        put(f"S-F9.helper.{n}.own", math.prod(factors))
        check(all(at(curve, y) * math.prod(factors) == math.prod(y - z for z in OPENING_TRIO if z != x)
                  for y in range(-20, 21)), f"14 helper {n} is its product of factors over {math.prod(factors)}")
        put(f"S-F9.helper.{n}.scaled", csv((tall_trio[i], n)))
    for i, x in enumerate(OPENING_TRIO):
        value = sum(h * at(curve, x) for h, curve in zip(tall_trio, helpers))
        put(f"S-F9.sum_at.{i + 1}", frac(value))
        check(value == tall_trio[i], f"14 the scaled helpers' sum passes through trustee {x}'s point")
    put("S-F9.sum_at.1.zero_helpers", csv(j + 1 for j, curve in enumerate(helpers) if at(curve, OPENING_TRIO[0]) == 0))
    on = {at(curve, x) for x, curve in zip(OPENING_TRIO, helpers)}
    off = {at(curve, y) for x, curve in zip(OPENING_TRIO, helpers) for y in OPENING_TRIO if y != x}
    check(len(on) == 1 and len(off) == 1, "14 every helper is on at its own trustee and off at the others alike")
    put("S-F9.helper.on", frac(min(on)))
    put("S-F9.helper.off", frac(min(off)))
    put("S-F9.helper.others", sum(1 for y in OPENING_TRIO if y != OPENING_TRIO[0]))
    sum_curve = tuple(sum(h * c[k] for h, c in zip(tall_trio, helpers)) for k in range(3))
    check(sum_curve == (SECRET, *SPINS) and at(sum_curve, 0) == SECRET, "14 the scaled helpers' sum is the dealer's curve")
    check(paper_curve(list(zip(OPENING_TRIO, tall_trio))) == (SECRET, *SPINS), "14 one parabola through the three points")

    # 15 (native): each helper read at position zero, as the scene works it, against the weight
    # elimination found; the weighted heights and cards, the sum on the cards wound round the
    # dial, and each height split into its card and whole elevens by winding
    for i, x in enumerate(OPENING_TRIO):
        n = i + 1
        factors = [0 - y for y in OPENING_TRIO if y != x]
        put(f"S-F9.helper.{n}.factors_at_zero", csv(factors))
        put(f"S-F9.helper.{n}.at_zero", math.prod(factors))
        put(f"S-F9.helper.{n}.weight", frac(rw[i]))
        check(Fraction(math.prod(factors), math.prod(x - y for y in OPENING_TRIO if y != x)) == rw[i] == at(helpers[i], 0),
              f"15 helper {n} at zero is its weight")
    put("S-F9.paper.weighted", csv(frac(v) for q, h in zip(rw, tall_trio) for v in (q, h)))
    put("S-F9.cards.weighted", csv(frac(v) for q, y in zip(rw, tv) for v in (q, y)))
    put("S-F9.share_terms", csv(frac(q * y) for q, y in zip(rw, tv)))
    turns, mark = wind(int(total))
    check(total.denominator == 1 and mark == SECRET, "15 the sum on the cards lands on the secret")
    put("S-F9.sum_lands", csv((frac(total), mark)))
    splits = [wind(h) for h in tall_trio]
    for i, (h, (t, m)) in enumerate(zip(tall_trio, splits)):
        check(m == tv[i] and h == m + t * P, f"15 height {h} is card {tv[i]} and whole elevens")
        put(f"S-F9.split.{i + 1}", csv((h, m, t)))
    weighted_elevens = sum(q * t for q, (t, _) in zip(rw, splits))
    put("S-F9.weighted_elevens", frac(weighted_elevens))
    check(paper_total == total + P * weighted_elevens, "15 the paper sum is the cards' sum and the weighted elevens")
    put("S-F9.sum_split", csv((frac(total), mark, turns)))
    check(turns == -weighted_elevens, "15 the cards' sum is the secret and one eleven: the elevens cancel")

    # 16 to 18: fractional weights, and dividing on the dial (trustees 2, 4 and 5)
    sv = [shares[x - 1] for x in SECOND_TRIO]
    put("S-F10.trio", csv(SECOND_TRIO))
    put("S-F10.trio_shares", csv(sv))
    own = [math.prod(x - y for y in SECOND_TRIO if y != x) for x in SECOND_TRIO]
    at0 = [math.prod(0 - y for y in SECOND_TRIO if y != x) for x in SECOND_TRIO]
    put("S-F10.helper_at_own", csv(own))
    put("S-F10.helper_at_zero", csv(at0))
    rw2 = rational_weights(SECOND_TRIO)
    put("S-F10.weights", csv(frac(q) for q in rw2))
    check([Fraction(z, o) for z, o in zip(at0, own)] == rw2, "16 product formula equals elimination")
    tall_second = [SECRET + SPINS[0] * x + SPINS[1] * x * x for x in SECOND_TRIO]
    put("S-F10.paper.heights", csv(tall_second))
    put("S-F10.paper.sum", frac(sum(q * h for q, h in zip(rw2, tall_second))))
    # the fractions kept and used on the shares: here the thirds happen to cancel
    put("S-F10.fraction_terms", csv(frac(q * y) for q, y in zip(rw2, sv)))
    put("S-F10.fraction_sum", frac(sum(q * y for q, y in zip(rw2, sv))))
    # another dealing of the same secret, whose thirds do not cancel
    other = (SECRET, *OTHER_SPINS)
    ov = [share(other, x) for x in SECOND_TRIO]
    put("S-F10.other.spins", csv(OTHER_SPINS))
    put("S-F10.other.heights", csv(SECRET + OTHER_SPINS[0] * x + OTHER_SPINS[1] * x * x for x in SECOND_TRIO))
    put("S-F10.other.shares", csv(ov))
    other_fraction = sum(q * y for q, y in zip(rw2, ov))
    put("S-F10.other.fraction_sum", frac(other_fraction))
    check(other_fraction.denominator != 1, "17 the other dealing's fraction sum is no whole number")
    put("S-F10.other.secret", csv(rebuild(SECOND_TRIO, ov)))
    # how many of the 121 spin pairs for this secret leave the fraction sum a fraction
    put("S-F10.fractional_spin_pairs",
        sum(1 for a in range(P) for b in range(P)
            if sum(q * share((SECRET, a, b), x) for q, x in zip(rw2, SECOND_TRIO)).denominator != 1))
    # clear the thirds: three times each weight is whole, and gives three times the secret
    clear = math.lcm(*(q.denominator for q in rw2))
    cleared = [clear * q for q in rw2]
    check(all(q.denominator == 1 for q in cleared), "17 three times the weights are whole numbers")
    put("S-F10.clear", clear)
    put("S-F10.cleared_weights", csv(frac(q) for q in cleared))
    put("S-F10.other.cleared_terms", csv(frac(q*y) for q, y in zip(cleared, ov)))
    other_cleared = sum(q * y for q, y in zip(cleared, ov))
    put("S-F10.other.cleared_sum", frac(other_cleared))
    put("S-F10.other.cleared_lands", int(other_cleared) % P)
    real_cleared = sum(q * y for q, y in zip(cleared, sv))
    put("S-F10.cleared_sum", frac(real_cleared))
    put("S-F10.cleared_lands", int(real_cleared) % P)
    put("S-F10.times3", csv((3 * x) % P for x in range(P)))
    undone = [k for k in range(P) if (3 * k) % P == int(real_cleared) % P]
    check(undone == [k for k in range(P) if (3 * k) % P == int(other_cleared) % P], "17 both dealings land on the same mark")
    check(len(undone) == 1, "17 exactly one mark, times three, lands there")
    put("S-F10.thrice", csv(undone))
    put("S-F10.thrice_unwrapped", 3 * undone[0])
    put("S-F10.third", inverse(3))
    dw2 = [unit_weight(SECOND_TRIO, i) for i in range(3)]
    put("S-F10.weights_on_dial", csv(dw2))
    check(all((q.numerator * inverse(q.denominator % P)) % P == d for q, d in zip(rw2, dw2)), "18 fractions on the dial")
    put("S-F10.sum_unwrapped", sum(d * y for d, y in zip(dw2, sv)))
    put("S-F10.secret", csv(rebuild(SECOND_TRIO, sv)))
    put("S-F10.inverses", ",".join(f"{d}:{inverse(d)}" for d in range(1, P)))
    put("S-F10.ten.invertible", csv(d for d in range(1, TEN) if inverse(d, TEN) is not None))
    put("S-F10.partner_product", clear * inverse(clear))
    put("S-F10.undo_product", (int(real_cleared) % P) * inverse(clear))
    put("S-F10.weight_products", csv(q.numerator * inverse(q.denominator) for q in rw2))
    put("S-F10.cleared_terms", csv(frac(q*y) for q,y in zip(cleared,sv)))
    put("S-F10.dial_terms", csv(d*y for d,y in zip(dw2,sv)))
    put("S-F10.whole_turns", sum(d*y for d,y in zip(dw2,sv)) // P)
    put("S-F10.ten.noninvertible", csv(d for d in range(1,TEN) if inverse(d,TEN) is None))


    # 19 and 20: any three, and the ten-mark trio that cannot
    every = list(itertools.combinations(TRUSTEES, 3))
    put("S-F11.trios", len(every))
    put("S-F11.trios_giving_secret", sum(1 for g in every if rebuild(g, [shares[x - 1] for x in g]) == [SECRET]))
    a, b, third = SECOND_TRIO
    diff = (a*b % P, (-a-b) % P, 1)
    trial = tuple((u+v) % P for u,v in zip(recipe,diff))
    put("S-F11.trial", csv(trial))
    put("S-F11.difference", csv(diff))
    put("S-F11.trial_shares", csv(share(trial,x) for x in SECOND_TRIO))
    put("S-F11.difference_row", csv((x-a)*(x-b) % P for x in range(P)))
    put("S-F11.left_row", csv((x-a) % P for x in range(P)))
    put("S-F11.right_row", csv((x-b) % P for x in range(P)))
    put("S-F11.roots", csv(x for x in range(P) if (x-a)*(x-b) % P == 0))
    put("S-F11.third_difference", (third-a)*(third-b) % P)
    put("S-F11.most_zeros", max(zeros(r, P) for r in recipes() if r != (0, 0, 0)))
    line_zeros = lambda p: max(sum(1 for x in range(p) if (c + d * x) % p == 0)  # noqa: E731
                               for c in range(p) for d in range(p) if (c, d) != (0, 0))
    put("S-F11.line_most_zeros", line_zeros(P))
    put("S-F11.ten.line_most_zeros", line_zeros(TEN))
    ten_shares = [share(recipe, x, TEN) for x in OPENING_TRIO]
    put("S-F11.ten.opening_trio_secrets", csv(rebuild(OPENING_TRIO, ten_shares, TEN)))
    # on ten marks, the trios whose three shares leave more than one secret
    ten_all = [share(recipe, x, TEN) for x in TRUSTEES]
    loose = [g for g in every if len(rebuild(g, [ten_all[x - 1] for x in g], TEN)) > 1]
    check(len(loose) == 1, f"ten marks: one trio cannot pin the secret, found {loose}")
    put("S-F11.ten.trios_that_pin", len(every) - len(loose))
    put("S-F11.ten.trio_table", ";".join(csv(g)+":"+csv(rebuild(g,[ten_all[x-1] for x in g],TEN)) for g in every))
    ten_trio = loose[0]
    ten_trio_shares = [ten_all[x - 1] for x in ten_trio]
    put("S-F11.ten.trio", csv(ten_trio))
    put("S-F11.ten.trio_shares", csv(ten_trio_shares))
    dealing = [r for r in recipes(TEN) if all(share(r, x, TEN) == y for x, y in zip(ten_trio, ten_trio_shares))]
    put("S-F11.ten.trio_secrets", csv(sorted({r[0] for r in dealing})))
    # the difference of the dealer's recipe and another dealing the same shares: a ghost,
    # zero at the trio, that moves the secret; the simplest is a line
    ghosts = sorted({tuple((u - v) % TEN for u, v in zip(r, recipe)) for r in dealing} - {(0, 0, 0)},
                    key=lambda g: (g[2], g[1], g[0]))
    ghost = next(g for g in ghosts if g[0] != 0)
    put("S-F11.ten.ghost", csv(ghost))
    put("S-F11.ten.ghost.heights", csv(ghost[0] + ghost[1] * x + ghost[2] * x * x for x in ten_trio))
    put("S-F11.ten.ghost.zeros", csv(x for x in range(TEN) if share(ghost, x, TEN) == 0))
    moved = tuple((u + v) % TEN for u, v in zip(recipe, ghost))
    put("S-F11.ten.moved_recipe", csv(moved))
    check(all(share(moved, x, TEN) == y for x, y in zip(ten_trio, ten_trio_shares)) and moved[0] != SECRET,
          "20 the ghost moves the secret and keeps the trio's shares")

    # 04 (the guess), 21: what it promises, 22: a liar
    put("S-F12.guess_chance", frac(Fraction(1, P)))
    put("S-F12.guess_misses", sum(1 for g in range(P) if g != SECRET))
    allowed = [s for s in range(P) if any(b != 0 for _, b in per_secret[s])]
    put("S-F12.nonzero_last.excluded", csv(sorted(set(range(P)) - set(allowed))))
    put("S-F12.nonzero_last.left", len(allowed))
    straight_recipe = (line_secret, *per_secret[line_secret][0])
    put("S-F12.straight_recipe", csv(straight_recipe))
    put("S-F12.straight_heights", csv(sum(a*x**i for i,a in enumerate(straight_recipe)) for x in PAIR))
    liar_values = [shares[x - 1] if x != LIAR[0] else LIAR[1] for x in OPENING_TRIO]
    put("S-F12.liar", LIAR[0])
    put("S-F12.liar_true", shares[LIAR[0] - 1])
    put("S-F12.liar_hands_in", LIAR[1])
    put("S-F12.liar_result_unwrapped", frac(sum(q * y for q, y in zip(rw, liar_values))))
    put("S-F12.liar_result", csv(rebuild(OPENING_TRIO, liar_values)))
    fooled = [r for r in recipes() if all(share(r, x) == y for x, y in zip(OPENING_TRIO, liar_values))]
    put("S-F12.fourth", FOURTH)
    put("S-F12.fourth.predicted", share(fooled[0], FOURTH))
    put("S-F12.fourth.actual", shares[FOURTH - 1])
    handed = [LIAR[1] if x == LIAR[0] else shares[x - 1] for x in TRUSTEES]
    four = [x for x in TRUSTEES if x <= FOURTH]
    put("S-F12.four.curves_through_all",
        sum(1 for r in recipes() if all(share(r, x) == handed[x - 1] for x in four)))
    # any one of the four could be the liar: drop each, and the other three fit a curve
    if_liar = []
    for suspect in four:
        rest = [x for x in four if x != suspect]
        if_liar.append(csv(rebuild(rest, [handed[x - 1] for x in rest])))
    put("S-F12.four.secret_if_liar", ",".join(if_liar))
    put("S-F12.liar.predictions", csv(share(fooled[0], x) for x in TRUSTEES))
    candidate_rows = []
    for suspect in four:
        fits = [r for r in recipes() if all(share(r, x) == handed[x - 1] for x in four if x != suspect)]
        check(len(fits) == 1, "one curve per omitted position")
        candidate_rows.append(str(fits[0][0])+":"+csv(share(fits[0], x) for x in TRUSTEES))
    put("S-F12.four.candidates", ";".join(candidate_rows))
    put("S-F12.five.min_common", min(len(set(a)&set(b)) for a in itertools.combinations(TRUSTEES,4) for b in itertools.combinations(TRUSTEES,4)))
    # with all five, exactly one curve passes through four of them: the dealer's
    near = [r for r in recipes() if sum(1 for x in TRUSTEES if share(r, x) == handed[x - 1]) >= 4]
    put("S-F12.five.curves_through_four", len(near))
    put("S-F12.five.curve_secret", csv(r[0] for r in near))
    put("S-F12.five.odd_one_out", csv(x for x in TRUSTEES if share(near[0], x) != handed[x - 1]))

    # 23: the rule again
    put("S-F13.shares_seen", THRESHOLD - 1)
    put("S-F13.curves_left", sum(len(x) for x in per_secret))
    # chapter 23's question: four of seven on the same dial, three shares seen
    marks7, seen7 = (1, 2, 3), (4, 0, 9)
    fits = [sum(1 for t in itertools.product(range(P), repeat=3)
                if all((s + t[0] * x + t[1] * x * x + t[2] * x ** 3) % P == y for x, y in zip(marks7, seen7)))
            for s in range(P)]
    put("S-F13.four_of_seven.spins", 4 - 1)
    put("S-F13.four_of_seven.spin_triples", P ** 3)
    put("S-F13.four_of_seven.curves_left", sum(fits))
    put("S-F13.four_of_seven.ways_each", csv(sorted(set(fits))))
    candidates = []
    for secret in range(P):
        rows = [(secret,)+abc for abc in itertools.product(range(P),repeat=3)
                if all((secret+sum(co*x**(i+1) for i,co in enumerate(abc)))%P==y for x,y in zip(marks7,seen7))]
        check(len(rows)==1,"one cubic for each secret")
        candidates.append(rows[0])
    evaluate = lambda row,x: sum(co*x**i for i,co in enumerate(row))%P
    example = candidates[SECRET]
    put("S-F13.four_of_seven.needed",4)
    put("S-F13.four_of_seven.trustees",7)
    put("S-F13.four_of_seven.recipe",csv(example))
    put("S-F13.four_of_seven.cards",csv(evaluate(example,x) for x in range(1,8)))
    put("S-F13.four_of_seven.fourth_by_secret",csv(evaluate(row,4) for row in candidates))


    # S-X: the primary source (Shamir, Comm. ACM 22(11), 612-613, November 1979)
    put("S-X1.shamir.year", 1979)
    put("S-X1.shamir.locks", SHAMIR_LOCKS)
    put("S-X1.shamir.keys", SHAMIR_KEYS)
    return f


def self_test():
    """Edge tests for every always/never sentence, and negative controls that
    show the same checks do fail when the property is false."""
    # every pair of distinct nonzero marks, every pair of values, every secret: one spin pair
    for marks in itertools.combinations(range(1, P), 2):
        for values in itertools.product(range(P), repeat=2):
            for s in range(P):
                check(len(fitting_spins(marks, values, s)) == 1, f"two shares {marks}={values} secret {s}")
    # one share: every secret, exactly p spin pairs
    for x in range(1, P):
        for y in range(P):
            for s in range(P):
                n = sum(1 for a in range(P) for b in range(P) if share((s, a, b), x) == y)
                check(n == P, f"one share {x}={y} secret {s}: {n}")
    # three distinct nonzero marks: every recipe is rebuilt exactly, by the search
    for marks in itertools.combinations(range(1, P), 3):
        table = {}
        for r in recipes():
            table.setdefault(tuple(share(r, x) for x in marks), []).append(r[0])
        check(len(table) == P ** 3 and all(len(v) == 1 for v in table.values()), f"three shares {marks}")
    # two of two: every B value, every secret, one A value
    for bv in range(P):
        check(sorted((a + bv) % P for a in range(P)) == list(range(P)), f"two of two B={bv}")
    # the times rows: every nonzero row shuffles on 11; on 10 only 1, 3, 7 and 9
    check(all(sorted((k * x) % P for x in range(P)) == list(range(P)) for k in range(1, P)), "11 rows shuffle")
    check([k for k in range(1, TEN) if sorted((k * x) % TEN for x in range(TEN)) == list(range(TEN))] == [1, 3, 7, 9],
          "10 rows")
    # every nonzero mark of 11 has one inverse; on 10 only the shuffling rows do
    check(all(inverse(d) is not None for d in range(1, P)), "11 inverses")
    check([d for d in range(1, TEN) if inverse(d, TEN) is not None] == [1, 3, 7, 9], "10 inverses")
    # edge: a share at mark 0 is the secret, which is why no trustee sits there
    check(all(share(r, 0) == r[0] for r in recipes()), "mark zero is the secret")
    # edge: two trustees at one mark hold one share; "three" shares then leave every secret
    check(rebuild((1, 1, 2), [share((SECRET, *SPINS), x) for x in (1, 1, 2)]) == list(range(P)), "repeated mark")
    # edge: a nonzero recipe has at most two zeros on 11 marks (so three shares fix it)
    check(all(zeros(r, P) <= 2 for r in recipes() if r != (0, 0, 0)), "at most two zeros")
    # the scheme on the vault's dial passes the pair check (all ten pairs)
    for marks in itertools.combinations(TRUSTEES, 2):
        ok, leak = pair_learns_nothing(P, marks)
        check(ok, f"pair {marks} on 11: {leak}")

    # negative control 1: the same pair check on a ten-mark dial finds leaks
    leaks = [m for m in itertools.combinations(TRUSTEES, 2) if not pair_learns_nothing(TEN, m)[0]]
    check(len(leaks) > 0, "negative control: ten marks should leak for some pair")
    # negative control 2: forbidding a zero last spin leaks for every pair
    for marks in itertools.combinations(TRUSTEES, 2):
        ok, _ = pair_learns_nothing(P, marks, spin_ok=lambda a, b: b != 0)
        check(not ok, f"negative control: nonzero last spin should leak for {marks}")
    # negative control 2b: on ten marks, trustees 4 and 5 alone always rebuild the secret
    check(all((5 * share(r, 4, TEN) - 4 * share(r, 5, TEN)) % TEN == r[0] for r in recipes(TEN)),
          "negative control: on ten marks, 5*y4 - 4*y5 is the secret")
    # negative control 3: the line scheme on ten marks leaks exactly for marks sharing a factor with 10
    for x in range(1, TEN):
        cands = {s for s in range(TEN) for a in range(TEN) if line_share(s, a, x, TEN) == line_share(SECRET, LINE_SLOPE, x, TEN)}
        check(len(cands) == TEN // math.gcd(x, TEN), f"ten-mark line holder {x}: {len(cands)}")
    # edge: a third share always leaves one secret, whichever two were seen first
    for marks in itertools.combinations(TRUSTEES, 2):
        for j in TRUSTEES:
            if j in marks:
                continue
            for values in itertools.product(range(P), repeat=2):
                met = [share((s, *fitting_spins(marks, values, s)[0]), j) for s in range(P)]
                check(sorted(met) == list(range(P)), f"joiner {j} after {marks}={values}")
    # chapter 11: the joiner's own card leaves the secret alone; every other card it could
    # hold leaves exactly one other secret, a different one for each card (the control)
    dealt_ = [share((SECRET, *SPINS), x) for x in TRUSTEES]
    pair_values = [dealt_[x - 1] for x in PAIR]
    picked = []
    for y in range(P):
        left = sorted({s for s in range(P) for a, b in fitting_spins(PAIR, pair_values, s)
                       if share((s, a, b), JOINER) == y})
        check(len(left) == 1 and (left == [SECRET]) == (y == dealt_[JOINER - 1]), f"joiner holding {y}: {left}")
        picked.extend(left)
    check(sorted(picked) == list(range(P)), "every card the joiner could hold picks its own secret")
    # negative control 0: ordinary pieces that add up leak; dial pieces never do
    for s in range(P):
        for a in range(s + 1):
            b = s - a
            fits = [t for t in range(P) if t - b >= 0]
            check(len(fits) == P - b, f"ordinary B={b} leaves {P - b}")
    check(any(len([t for t in range(P) if t - b >= 0]) < P for b in range(1, P)),
          "negative control: some ordinary piece rules a secret out")
    # edge and control for 01_3 and 01_4: one keeper is shut by one loss, copies only by
    # losing all five; any one holder, bribed, gives the secret away under either plan
    check(losses_to_shut((TRUSTEES[0],)) == 1 and losses_to_shut(TRUSTEES) == len(TRUSTEES),
          "one loss shuts a single keeper's vault; copies need every copy lost")
    check(all(bribes_to_open((t,)) == 1 and bribes_to_open(TRUSTEES) == 1 for t in TRUSTEES),
          "any one holder, bribed, gives the secret away")
    # negative control: a plan that hands nobody a copy is shut from the start and cannot be bribed open
    check(losses_to_shut(()) == 0 and bribes_to_open(()) is None, "an empty plan: shut, and no bribe opens it")
    # negative control 4: the rebuild search notices a lie only with a fourth share
    f = facts()
    check(f["S-F12.liar_result"] != str(SECRET), "a lie changes the rebuilt secret")
    check(f["S-F12.fourth.predicted"] != f["S-F12.fourth.actual"], "a fourth share exposes the lie")
    # chapter 15: whole weights give the same mark from shares and from heights, for every
    # recipe; chapters 16 and 17: fractional weights on shares miss for some recipe (the control)
    w_open = rational_weights(OPENING_TRIO)
    w_second = rational_weights(SECOND_TRIO)
    for r in recipes():
        tall = [r[0] + r[1] * x + r[2] * x * x for x in OPENING_TRIO]
        dealt_ = [share(r, x) for x in OPENING_TRIO]
        check(sum(q * h for q, h in zip(w_open, tall)) == r[0], f"paper weights {r}")
        check(int(sum(q * y for q, y in zip(w_open, dealt_))) % P == r[0], f"whole weights on shares {r}")
    check(any(sum(q * share(r, x) for q, x in zip(w_second, SECOND_TRIO)).denominator != 1 for r in recipes()),
          "negative control: fractional weights on shares are not always whole")
    # chapter 14: for every three of the five trustees, each helper found by elimination is
    # 1 at its own trustee, 0 at the other two, and its product of factors over its own value;
    # chapter 15: the paper sum and the cards' sum differ by eleven times the weighted elevens
    for trio in itertools.combinations(TRUSTEES, 3):
        for i, x in enumerate(trio):
            c = paper_curve([(y, 1 if j == i else 0) for j, y in enumerate(trio)])
            own = math.prod(x - z for z in trio if z != x)
            check([c[0] + c[1] * y + c[2] * y * y for y in trio] == [1 if y == x else 0 for y in trio]
                  and all((c[0] + c[1] * y + c[2] * y * y) * own == math.prod(y - z for z in trio if z != x)
                          for y in range(-10, 11)), f"helper {i + 1} of {trio}")
    for r in recipes():
        tall = [r[0] + r[1] * x + r[2] * x * x for x in OPENING_TRIO]
        dealt_ = [share(r, x) for x in OPENING_TRIO]
        gap = sum(q * h for q, h in zip(w_open, tall)) - sum(q * y for q, y in zip(w_open, dealt_))
        check(gap == P * sum(q * ((h - y) // P) for q, h, y in zip(w_open, tall, dealt_)), f"whole elevens {r}")
    # negative control 5: a nudged weight, or a helper scaled by the wrong height, misses
    nudged_weights = (3, -3, 2)
    check(sum(q * y for q, y in zip(nudged_weights, (8, 3, 1))) % P != SECRET,
          "negative control: weights 3, -3, 2 on the trio's cards miss the secret")
    swapped = [paper_curve([(x, 1 if j == i else 0) for j, x in enumerate(OPENING_TRIO)]) for i in range(3)]
    wrong = sum(h * (c[0] + c[1] + c[2]) for h, c in zip((47, 19, 89), swapped))
    check(wrong != 19, "negative control: heights 47 and 19 swapped, the sum at position 1 is not 19")
    # chapter 22: among five shares with one lie, exactly one curve fits four of them,
    # the dealer's; four shares with one lie fit no curve (every recipe, liar and lie)
    tables = {g: {tuple(share(r, x) for x in g): r for r in recipes()} for g in itertools.combinations(TRUSTEES, 3)}
    for r in recipes():
        true = [share(r, x) for x in TRUSTEES]
        for liar in TRUSTEES:
            for lie in range(P):
                if lie == true[liar - 1]:
                    continue
                handed = [lie if x == liar else true[x - 1] for x in TRUSTEES]
                fit4 = {c for g, t in tables.items() for c in [t[tuple(handed[x - 1] for x in g)]]
                        if sum(1 for x in TRUSTEES if share(c, x) == handed[x - 1]) >= 4}
                check(fit4 == {r}, f"five shares, liar {liar}, recipe {r}")
                if liar <= FOURTH:
                    # the curve through the three honest ones among the first four misses the lie
                    honest = tuple(x for x in range(1, FOURTH + 1) if x != liar)
                    c = tables[honest][tuple(handed[x - 1] for x in honest)]
                    check(share(c, liar) != handed[liar - 1], f"four shares fit no curve, liar {liar}, recipe {r}")
    # negative control 5: on ten marks a nonzero line can have five zeros; on eleven, one
    check(f["S-F11.ten.line_most_zeros"] == "5" and f["S-F11.line_most_zeros"] == "1", "line zeros on ten and eleven")
    # chapters 08 and 09: winding takes off whole turns and lands where the remainder
    # does, for every height up to 250
    check(all(wind(h)[0] * P + wind(h)[1] == h and wind(h)[1] == h % P for h in range(250)), "winding")
    # chapter 08: one share leaves every secret, one spin each, for every holder and
    # share on eleven marks; on ten marks holder 2 leaves only five (the control)
    for x in range(1, P):
        for y in range(P):
            check(sorted((y - x * a) % P for a in range(P)) == list(range(P)), f"one share {x}={y}")
    check(len({(2 - 2 * a) % TEN for a in range(TEN)}) == 5, "negative control: ten marks, holder 2 leaves five")
    # chapter 09: on paper, any three of the five heights give back the recipe, for every
    # recipe with spins 0 to 10; a height off by one gives another curve (the control)
    for r in recipes():
        tall = {x: r[0] + r[1] * x + r[2] * x * x for x in TRUSTEES}
        for trio in itertools.combinations(TRUSTEES, THRESHOLD):
            check(paper_curve([(x, tall[x]) for x in trio]) == r, f"paper curve {r} from {trio}")
    nudged = [(1, 19), (2, 47), (3, 90)]
    check(paper_curve(nudged) != (SECRET, *SPINS), "negative control: a nudged height moves the paper curve")
    # chapter 12: on ten marks, whatever two cards trustees 4 and 5 hold, the only secret
    # that fits is 5*y4 - 4*y5, ten ways; so a wrong card picks another secret (the control)
    for y4 in range(TEN):
        for y5 in range(TEN):
            ways = [len(fitting_spins(PAIR, (y4, y5), s, TEN)) for s in range(TEN)]
            want = (5 * y4 - 4 * y5) % TEN
            check(ways == [TEN if s == want else 0 for s in range(TEN)], f"ten marks, the pair holding {y4},{y5}")
    check(f["S-F8.ten.pair_secrets"] == "5" and [s for s in range(TEN) if fitting_spins(PAIR, (6, 5), s, TEN)] == [0],
          "negative control: on ten marks the pair holding 6 and 5 finds 0, not 5")
    # wrong modulus: the -20 vanishes only on a dial whose marks divide 20; of 7 to 13 marks,
    # only ten; on twelve it leaves 4b and on eleven 2b
    check([p for p in range(7, 14) if 20 % p == 0] == [TEN] and -20 % 12 == 4 and -20 % P == 2,
          "the -20 vanishes on ten marks alone")
    # chapter 13: on a prime dial with more marks than trustees, every pair learns nothing
    # (7 and 13 marks); on nine and twelve marks some pairs leak (the controls)
    for p, leaking in ((7, 0), (13, 0), (9, 4), (12, 9)):
        got = sum(1 for m in itertools.combinations(TRUSTEES, 2) if not pair_learns_nothing(p, m)[0])
        check(got == leaking, f"{p} marks: {got} leaking pairs, expected {leaking}")
    # edge: with only five marks trustee 5 sits at 0 and holds the secret itself
    check(all(share(r, 5, 5) == r[0] for r in recipes(5)), "on five marks trustee 5's share is the secret")
    # a product lands on 0 only when a factor is 0, on 7, 11 and 13 marks; on ten and
    # twelve marks, some nonzero marks multiply to 0 (the control)
    for p in (7, P, 13):
        check(all((a * b) % p != 0 for a in range(1, p) for b in range(1, p)), f"{p} marks: no zero products")
        check(all(len({(m * x) % p for x in range(p)}) == p for m in range(1, p)), f"{p} marks: every row shuffles")
    for p in (TEN, 12):
        check(any((a * b) % p == 0 for a in range(1, p) for b in range(1, p)), f"negative control: {p} marks")
    # a composite dial's rows do not all collapse: on twelve marks, times 5 shuffles too
    check(len({(5 * x) % 12 for x in range(12)}) == 12, "on twelve marks, times 5 shuffles")
    return len(leaks)


def main(argv):
    f = facts()
    for k, v in f.items():
        print(f"{k}\t{v}")
    if "--self-test" in argv:
        leaks = self_test()
        print(f"# self-test: ten-mark dial pairs that leak: {leaks} of 10", file=sys.stderr)
    print(f"# {len(f)} facts, {len(FAILURES)} failed cross-checks", file=sys.stderr)
    return 1 if FAILURES else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
