#!/usr/bin/env python3
"""Independent arithmetic checks for the Shamir draft (review by GentleLantern,
2026-10-24, Bead mx-video-program-program-87do.46.2).

Run from the repository root (standard library only):

    python3 -I -B docs/video-program/third-wave-20261024/reviews/shamir-GentleLantern/check_shamir.py
    ... check_shamir.py --oracle narration_lab/third-wave-20261024/root/tend-0610/intake/shamir_secret_sharing/oracle.py

Written without reusing the author's oracle.py or model.rs. The methods
differ on purpose:
  * division on the dial is Fermat's little theorem, d**(p-2) mod p (the model
    uses the extended Euclidean algorithm, the oracle a search);
  * two shares are solved in closed form (Cramer's rule mod p), and the
    "every secret fits once" counts are a tally of all p**3 recipes by the
    shares they deal (the oracle loops over share values, then spins);
  * reconstruction is Lagrange's product formula with Fractions, and Gaussian
    elimination mod p as a third way;
  * the padlock bounds come from the argument "a lock that every trio can open
    stops at most one pair", tried on every possible key set.

Each CLAIM line pairs a draft line id with the value the draft says and the
value computed here. The negative controls rerun the same claims with one
assumption broken and must fail. They print their exact failures.

Exit status: 0 if every claim holds and every control fails as expected.
"""

import itertools
import math
import subprocess
import sys
from fractions import Fraction

P = 11
TEN = 10
TRUSTEES = (1, 2, 3, 4, 5)


# ------------------------------------------------------------- arithmetic

def inv(d, p):
    """d's partner mod p, or None. Fermat when p is prime, else by gcd test and pow."""
    d %= p
    if d == 0 or math.gcd(d, p) != 1:
        return None
    k = pow(d, p - 2, p) if is_prime(p) else pow(d, -1, p)
    assert (d * k) % p == 1
    return k


def is_prime(n):
    return n >= 2 and all(n % q for q in range(2, math.isqrt(n) + 1))


def on_dial(q, p):
    """A Fraction onto the dial: numerator times the denominator's partner."""
    q = Fraction(q)
    k = inv(q.denominator, p)
    return None if k is None else (q.numerator * k) % p


def height(s, spins, x):
    return s + sum(c * x ** (i + 1) for i, c in enumerate(spins))


def share(s, spins, x, p):
    return height(s, spins, x) % p


def lagrange_weights(xs, at=0):
    """Ordinary-number weights by the product formula."""
    out = []
    for i, xi in enumerate(xs):
        w = Fraction(1)
        for j, xj in enumerate(xs):
            if j != i:
                w *= Fraction(at - xj, xi - xj)
        out.append(w)
    return out


def solve_mod(rows, p):
    """Gaussian elimination mod a prime p on an augmented matrix; None if singular."""
    rows = [r[:] for r in rows]
    n = len(rows)
    for c in range(n):
        piv = next((r for r in range(c, n) if inv(rows[r][c], p) is not None), None)
        if piv is None:
            return None
        rows[c], rows[piv] = rows[piv], rows[c]
        k = inv(rows[c][c], p)
        rows[c] = [(v * k) % p for v in rows[c]]
        for r in range(n):
            if r != c and rows[r][c] % p:
                f = rows[r][c]
                rows[r] = [(u - f * w) % p for u, w in zip(rows[r], rows[c])]
    return [rows[r][n] for r in range(n)]


def two_share_spins(s, pts, p):
    """Cramer's rule: the one (a, b) with s + a x + b x^2 = y at both points."""
    (x1, y1), (x2, y2) = pts
    det = x1 * x2 * x2 - x1 * x1 * x2
    k = inv(det, p)
    if k is None:
        return None
    r1, r2 = y1 - s, y2 - s
    return ((r1 * x2 * x2 - x1 * x1 * r2) * k % p, (x1 * r2 - r1 * x2) * k % p)


def tally(xs, p, n_spins=2, spin_values=None, spin_rule=None):
    """For trustees at xs: shares -> {secret: number of allowed spin tuples dealing them}."""
    values = range(p) if spin_values is None else spin_values
    out = {}
    for s in range(p):
        for spins in itertools.product(values, repeat=n_spins):
            if spin_rule and not spin_rule(s, spins):
                continue
            ys = tuple(share(s, spins, x, p) for x in xs)
            d = out.setdefault(ys, {})
            d[s] = d.get(s, 0) + 1
    return out


def learns_nothing(xs, p, **kw):
    """Every possible view of xs is dealt equally often by every secret."""
    t = tally(xs, p, **kw)
    return all(len(d) == p and len(set(d.values())) == 1 for d in t.values())


# ------------------------------------------------------------- the claims

def claims(prm):
    """(id, what, draft value, computed value) for the parameters `prm`."""
    p, s, spins = prm["p"], prm["secret"], prm["spins"]
    rule = prm.get("spin_rule")
    vals = prm.get("spin_values")
    out = []

    def c(i, what, said, got):
        out.append((i, what, said, got))

    sh = [share(s, spins, x, p) for x in TRUSTEES]
    c("04_1/09_6", "the five shares", [8, 3, 1, 2, 6], sh)
    c("09_3/09_4", "ordinary heights", [19, 47, 89, 145, 215], [height(s, spins, x) for x in TRUSTEES])
    c("09_6", "47 = 4 elevens and 3", (4, 3), divmod(height(s, spins, 2), p))

    # 02-03: two of two
    c("02_4", "B's ordinary 3 leaves", 8, len([t for t in range(p) if t >= 3]))
    a_held = [(a + 7) % p for a in range(p)]
    c("03_1", "9 + 7 on the dial", 5, (9 + 7) % p)
    c("03_2", "10 + 1, 5 - 7", (0, 9), ((10 + 1) % p, (5 - 7) % p))
    c("03_5", "secret if A held 0..10", [7, 8, 9, 10, 0, 1, 2, 3, 4, 5, 6], a_held)

    # 05, 10: the pair 4 and 5 sees (2, 6); every secret fits one way
    pair = (4, 5)
    view = tuple(sh[x - 1] for x in pair)
    t = tally(pair, p, spin_values=vals, spin_rule=rule)
    counts = [t.get(view, {}).get(k, 0) for k in range(p)]
    c("05_3", "ways each secret fits the pair's (2, 6)", [1] * 11, counts)
    closed = [two_share_spins(k, list(zip(pair, view)), p) for k in range(p)]
    c("10_3", "spins for secrets 0 and 1", [(1, 4), (0, 9)], closed[:2])
    c("10_1", "the true secret's spins", (7, 7), closed[5])
    c("05_3", "a different spin pair each time", 11, len(set(closed)))
    every_pair = all(learns_nothing(m, p, spin_values=vals, spin_rule=rule)
                     for m in itertools.combinations(TRUSTEES, 2))
    c("11_6", "every pair, every view: all secrets equally often", True, every_pair)
    # posterior equals prior for a lopsided prior (05_4: "as likely as before")
    prior = [Fraction(k + 1, sum(range(1, p + 1))) for k in range(p)]
    joint = [prior[k] * counts[k] for k in range(p)]
    tot = sum(joint)
    post = [j / tot for j in joint] if tot else None
    c("05_4", "posterior = prior (prior ~ secret+1)", True, post == prior)

    # 11: the joiner, trustee 1 holding 8
    at1 = [share(k, closed[k], 1, p) if closed[k] else None for k in range(p)]
    c("11_5", "each curve at position 1", [5, 10, 4, 9, 3, 8, 2, 7, 1, 6, 0], at1)
    c("11_5", "secrets left after trustee 1 joins", [5], [k for k in range(p) if at1[k] == sh[0]])
    c("11_2", "paper spins for secret 0", (Fraction(37, 4), Fraction(27, 4)), paper_spins(0, pair, s, spins))
    whole = [k for k in range(p) if all(q.denominator == 1 and 0 <= q < p
                                        for q in paper_spins(k, pair, s, spins))]
    c("11_2", "secrets with whole paper spins 0..10", [5], whole)

    # 07, 08, 13: the line
    line = [height(s, (4,), x) for x in TRUSTEES]
    c("07_3", "line heights", [9, 13, 17, 21, 25], line)
    c("07_6", "paper: secrets fitting 25 at 5", [0, 5, 10],
      sorted({k for k in range(p) for a in range(p) if k + 5 * a == 25}))
    c("08_1", "line shares", [9, 2, 6, 10, 3], [v % p for v in line])
    c("08_4", "step back twice from (2, 2)", (9, 5), ((2 - 4) % p, (2 - 8) % p))
    c("08_6", "trustee 2 alone: secret by spin", [2, 0, 9, 7, 5, 3, 1, 10, 8, 6, 4],
      [(line[1] % p - 2 * a) % p for a in range(p)])
    c("08_7", "doubles", [0, 2, 4, 6, 8, 10, 1, 3, 5, 7, 9], [(2 * k) % p for k in range(p)])
    c("13_4", "times 5", [0, 5, 10, 4, 9, 3, 8, 2, 7, 1, 6], [(5 * k) % p for k in range(p)])

    # 14-16: rebuilding
    trio = (1, 2, 3)
    w = lagrange_weights(trio)
    c("15_1/15_2", "opening trio weights", [3, -3, 1], w)
    total = sum(wi * sh[x - 1] for wi, x in zip(w, trio))
    c("15_4", "24 - 9 + 1 and where it lands", (16, 5), (total, total % p))
    c("14_5", "(x-2)(x-3) at 1", 2, (1 - 2) * (1 - 3))
    second = (2, 4, 5)
    w2 = lagrange_weights(second)
    c("16_2", "second trio weights", [Fraction(10, 3), -5, Fraction(8, 3)], w2)
    c("16_4", "dividing by 3 is times", 4, inv(3, p))
    dw2 = [on_dial(q, p) for q in w2]
    c("16_5", "weights on the dial", [7, 6, 10], dw2)
    s2 = sum(d * sh[x - 1] for d, x in zip(dw2, second)) if None not in dw2 else None
    c("16_6", "7*3 + 6*2 + 10*6 and where it lands", (93, 5), (s2, s2 % p) if s2 is not None else None)
    c("16_7", "ten-mark marks with no partner", [2, 4, 5, 6, 8], [k for k in range(1, TEN) if inv(k, TEN) is None])
    gauss = solve_mod([[1, x, x * x, sh[x - 1]] for x in trio], p)
    c("15_4", "Gaussian elimination mod p: the trio's recipe", [5, 7, 7], gauss)

    # 17: any three
    rebuilt = []
    for g in itertools.combinations(TRUSTEES, 3):
        dw = [on_dial(q, p) for q in lagrange_weights(g)]
        rebuilt.append(None if None in dw else sum(d * sh[x - 1] for d, x in zip(dw, g)) % p)
    c("17_1", "secret rebuilt by each of the ten trios", [5] * 10, rebuilt)
    most = max(sum(1 for x in range(p) if share(r[0], r[1:], x, p) == 0)
               for r in itertools.product(range(p), repeat=3) if any(r))
    c("17_4", "most zeros of a nonzero recipe", 2, most)

    # 18: the promise
    view_line = two_share_spins_line(list(zip(pair, view)), p)
    c("18_4", "the straight line through the pair meets 0 at", 8, view_line)
    nz = tally(pair, p, spin_values=vals, spin_rule=lambda k, sp: sp[1] != 0 and (rule is None or rule(k, sp)))
    left = sorted(nz.get(view, {}))
    c("18_4", "secrets left when b = 0 is refused", [k for k in range(p) if k != 8], left)
    lie = [8, 3, 2]
    c("18_5", "8*3 - 3*3 + 2*1 and where it lands", (17, 6),
      (sum(wi * y for wi, y in zip(w, lie)), sum(wi * y for wi, y in zip(w, lie)) % p))
    w_at4 = [on_dial(q, p) for q in lagrange_weights(trio, at=4)]
    c("18_6", "the lie predicts trustee 4 holds", 5, sum(d * y for d, y in zip(w_at4, lie)) % p)

    # 19: four of seven, three shares seen at positions 1, 2, 3
    fits = [sum(1 for t3 in itertools.product(range(p), repeat=3)
                if all(share(k, t3, x, p) == y for x, y in ((1, 4), (2, 0), (3, 9)))) for k in range(p)]
    c("19_4", "four of seven: ways each secret fits three shares", [1] * 11, fits)
    return out


def paper_spins(k, pair, s, spins):
    """The paper curve k + a x + b x^2 through the pair's ordinary heights."""
    (x1, x2) = pair
    h1, h2 = height(s, spins, x1) - k, height(s, spins, x2) - k
    det = Fraction(x1 * x2 * x2 - x1 * x1 * x2)
    return ((h1 * x2 * x2 - x1 * x1 * h2) / det, (x1 * h2 - h1 * x2) / det)


def two_share_spins_line(pts, p):
    """Where the one straight line through two points meets position 0, on the dial."""
    (x1, y1), (x2, y2) = pts
    slope = (y2 - y1) * inv(x2 - x1, p) % p
    return (y1 - slope * x1) % p


# ------------------------------------------------------------ side checks

def side_checks():
    """Facts the review cites that are not a draft line's number; printed, then asserted."""
    rows = []

    def r(what, got, want):
        rows.append((what, got, want))

    # 12: ten marks. Which pairs of trustees learn something?
    leak = [m for m in itertools.combinations(TRUSTEES, 2) if not learns_nothing(m, TEN)]
    safe = [m for m in itertools.combinations(TRUSTEES, 2) if m not in leak]
    r("12_5 ten-mark pairs that leak", len(leak), 9)
    r("ten-mark pair that learns nothing", safe, [(1, 3)])
    # the criterion behind it: the pair learns nothing iff (1, 1) is a value of a*x + b*x^2
    crit = [m for m in itertools.combinations(TRUSTEES, 2)
            if any(all((a * x + b * x * x) % TEN == 1 for x in m) for a in range(TEN) for b in range(TEN))]
    r("same pairs by the (1, 1)-in-image criterion", crit, safe)
    t = tally((4, 5), TEN)
    r("12_2 ten-mark (4, 5) seeing (5, 5): secrets", sorted(t[(5, 5)]), [5])
    r("12_3 5*y4 - 4*y5 = s on ten marks, every recipe", all(
        (5 * share(s, (a, b), 4, TEN) - 4 * share(s, (a, b), 5, TEN)) % TEN == s
        for s in range(TEN) for a in range(TEN) for b in range(TEN)), True)
    r("12_5 the same sum is s + 2b on eleven, every recipe", all(
        (5 * share(s, (a, b), 4, P) - 4 * share(s, (a, b), 5, P)) % P == (s + 2 * b) % P
        for s in range(P) for a in range(P) for b in range(P)), True)
    r("12_5 the pair's sum on eleven", (5 * 2 - 4 * 6, (5 * 2 - 4 * 6) % P), (-14, 8))
    # 13: the line on ten marks
    r("13_2 ten-mark line, trustee 5 holding 5: secrets",
      sorted({s for s in range(TEN) for a in range(TEN) if (s + 5 * a) % TEN == 5}), [0, 5])
    r("ten-mark line, trustee 2 holding 3: secrets (parity)",
      sorted({s for s in range(TEN) for a in range(TEN) if (s + 2 * a) % TEN == 3}), [1, 3, 5, 7, 9])
    r("13_7 smallest prime above five", next(q for q in range(6, 50) if is_prime(q)), 7)
    r("13_7 seven marks: every pair of 1..5 learns nothing",
      all(learns_nothing(m, 7) for m in itertools.combinations(TRUSTEES, 2)), True)
    r("six marks (not prime): pairs that leak",
      sum(1 for m in itertools.combinations(TRUSTEES, 2) if not learns_nothing(m, 6)) > 0, True)

    # 17_6: on ten marks, can three trustees fail to determine the secret?
    stuck = []
    for g in itertools.combinations(TRUSTEES, 3):
        tg = tally(g, TEN)
        if any(len(d) > 1 for d in tg.values()):
            stuck.append(g)
    rows.append(("ten marks: trios whose shares can leave two or more secrets", stuck, None))
    sh135 = tuple(share(5, (7, 7), x, TEN) for x in (1, 3, 5))
    rows.append((f"ten marks: trio (1, 3, 5) holding {sh135}: secrets", sorted(tally((1, 3, 5), TEN)[sh135]), None))
    r("ten marks: 5 + 5x^2 is zero at 1, 3 and 5", [(5 + 5 * x * x) % TEN for x in (1, 3, 5)], [0, 0, 0])
    r("17_6 5x + 5x^2 on ten marks: zeros", sum(1 for x in range(TEN) if (5 * x + 5 * x * x) % TEN == 0), 10)
    t123 = tally((1, 2, 3), TEN)
    rows.append(("ten marks: opening trio's example shares (9, 7, 9): secrets", sorted(t123[(9, 7, 9)]), None))

    # 14-16: whole-number weights wrap freely; fractions need the dial
    hts = {x: height(5, (7, 7), x) for x in TRUSTEES}
    shs = {x: hts[x] % P for x in TRUSTEES}
    for g in ((1, 2, 3), (2, 4, 5)):
        w = lagrange_weights(g)
        r(f"trio {g}: paper sum of true heights", sum(q * hts[x] for q, x in zip(w, g)), 5)
        rows.append((f"trio {g}: paper sum of wrapped shares", sum(q * shs[x] for q, x in zip(w, g)), None))
    naive = {}
    for g in itertools.combinations(TRUSTEES, 3):
        w = lagrange_weights(g)
        naive[g] = sum(q * shs[x] for q, x in zip(w, g))
    frac = [g for g, v in naive.items() if v.denominator != 1]
    rows.append(("trios whose paper sum of wrapped shares is a fraction", frac, None))
    r("every trio's naive sum, put on the dial, is 5", all(on_dial(v, P) == 5 for v in naive.values()), True)
    # the same trio under every other dealing of the secret 5: when do the thirds fail to cancel?
    w245 = lagrange_weights((2, 4, 5))
    frac_deals = [(a, b) for a in range(P) for b in range(P)
                  if sum(q * share(5, (a, b), x, P) for q, x in zip(w245, (2, 4, 5))).denominator != 1]
    rows.append(("secret 5: spin pairs whose trio (2, 4, 5) naive sum is a fraction", len(frac_deals), None))
    ex = [share(5, (0, 1), x, P) for x in (2, 4, 5)]
    ex_sum = sum(q * y for q, y in zip(w245, ex))
    r("e.g. spins 0 and 1: trio (2, 4, 5) holds, naive sum, on the dial",
      (ex, ex_sum, on_dial(ex_sum, P)), ([9, 10, 8], Fraction(4, 3), 5))
    r("a whole naive sum always wraps to 5 (11 is coprime to every weight's bottom)",
      all(v % P == 5 for v in naive.values() if v.denominator == 1), True)

    # 06: padlocks. A lock is the set K of trustees holding its key. Every trio
    # must open it (meets K), so a lock stops at most one pair (outside K).
    groups = [frozenset(k) for n in range(6) for k in itertools.combinations(TRUSTEES, n)]
    usable = [k for k in groups if all(set(tr) & k for tr in itertools.combinations(TRUSTEES, 3))]
    stops = [sum(1 for pr in itertools.combinations(TRUSTEES, 2) if not set(pr) & k) for k in usable]
    r("06_5 a lock every trio opens stops at most", max(stops), 1)
    r("06_5 so the fewest locks is the number of pairs", math.comb(5, 2), 10)
    r("06_3 keys each = pairs without that trustee", math.comb(4, 2), 6)
    r("06_7 Liu: locks C(11,5), keys C(10,5)", (math.comb(11, 5), math.comb(10, 5)), (462, 252))

    # 19_4: four of seven, any three nonzero positions of 1..7 on eleven marks
    dets = []
    for xs in itertools.combinations(range(1, 8), 3):
        m = [[x, x * x, x ** 3] for x in xs]
        d = (m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1]) - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
             + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]))
        dets.append(d % P)
    r("19_4 every 3 of positions 1..7: spins fixed by secret + shares (det != 0 mod 11)",
      all(dets), True)

    # random-coefficient assumptions: which broken assumptions make the pair (4, 5) learn something
    r("spins fair over 0..10: pair (4,5) learns nothing", learns_nothing((4, 5), P), True)
    r("b = 0 refused: pair (4,5) learns something",
      learns_nothing((4, 5), P, spin_rule=lambda s, sp: sp[1] != 0), False)
    r("a = 0 refused: pair (4,5) learns something",
      learns_nothing((4, 5), P, spin_rule=lambda s, sp: sp[0] != 0), False)
    r("spare dial of ten marks (spins 0..9): pair (4,5) learns something",
      learns_nothing((4, 5), P, spin_values=range(10)), False)
    r("first spin tied to the secret (a = s): pair (4,5) learns something",
      learns_nothing((4, 5), P, spin_rule=lambda s, sp: sp[0] == s), False)
    r("one trustee alone, fair spins: learns nothing (each position)",
      all(learns_nothing((x,), P) for x in TRUSTEES), True)
    return rows


# --------------------------------------------------------- timeline estimate

# Chapter 01 as the_vault.rs plays it: ("say", id) | ("wait", phrase) | ("anim", s)
# | ("end",) | ("read", chars) | ("until_read",) | ("hold",). Durations are the
# run times in the source; `frames_covering` rounding is ignored.
VAULT = [
    ("say", "01_1"), ("anim", 1.6), ("wait", "one vault"), ("anim", 0.6), ("wait", "a wrong mark"),
    ("anim", 0.7 + 0.15 + 0.5 + 0.3 + 0.5),
    ("say", "01_2"), ("wait", "any of the eleven marks"), ("anim", 1.4), ("wait", "This strip shows"),
    ("anim", 0.4), ("end",), ("anim", 1.0),
    ("say", "01_3"), ("wait", "tell one trustee"), ("anim", 0.4 + 0.4 + 0.8), ("wait", "one lost note"),
    ("anim", 0.4 + 0.8 + 0.5), ("wait", "one bribe"), ("anim", 0.3 + 0.4 + 0.6),
    ("say", "01_4"), ("wait", "a copy"), ("anim", 0.3 + 0.5 + 0.6), ("wait", "One absence"),
    ("anim", 0.3 + 0.3 + 0.4 + 0.3), ("wait", "open it alone"), ("anim", 5 * (0.08 + 0.26 + 0.08)),
    ("wait", "let it slip"), ("anim", 0.25 + 0.5),
    ("say", "01_5"), ("anim", 0.4), ("wait", "Any three of them"), ("anim", 0.4 + 10 * (0.15 + 0.12 + 0.15)),
    ("wait", "any two should learn nothing"), ("anim", 0.3 + 0.6 + 0.4), ("wait", "That number, three"),
    ("anim", 0.4),
    ("say", "01_6"), ("anim", 0.3 + 0.5), ("read", 99), ("wait", "two trustees can be away"), ("anim", 0.4 + 0.3),
    ("wait", "Before any mathematics"), ("until_read",), ("anim", 0.4 + 0.3 + 0.6),
    ("hold",), ("anim", 0.2 + 4.0 + 0.2), ("anim", 0.5 + 0.6), ("hold",),
]


def timeline(script, wps, gap):
    """Estimated cue lateness: each line lasts words/wps, sentences share it by
    characters (draw.rs's rule with no measured onsets), and a line starts
    `gap` after the previous one ends (the narrator's rule) or when said, if later."""
    now, line, end_prev, late = 0.0, None, 0.0, []
    read_at = None
    for ev in VAULT:
        kind = ev[0]
        if kind == "say":
            text = script[ev[1]]
            start = max(now, end_prev + gap if line else 0.0)
            now = start
            n = len(text.split())
            line = (ev[1], text, start, n / wps)
            end_prev = start + n / wps
        elif kind == "anim":
            now += ev[1]
        elif kind == "wait":
            lid, text, start, length = line
            i = text.find(ev[1])
            assert i >= 0, f"{lid} has no {ev[1]!r}"
            t = start + length * i / len(text)
            if now > t + 1e-9:
                late.append((lid, ev[1], round(now - t, 2)))
            now = max(now, t)
        elif kind == "end":
            now = max(now, end_prev)
        elif kind == "read":
            read_at = now + ev[1] / 15.0
        elif kind == "until_read":
            now = max(now, read_at)
        elif kind == "hold":
            now = max(now, end_prev + min(gap, 0.6))
    return late, now


# ---------------------------------------------------------------- the run

def run_claims(prm, label, quiet=False):
    bad = []
    for i, what, said, got in claims(prm):
        ok = normal(said) == normal(got)
        if not ok:
            bad.append((i, what, said, got))
        if not quiet or not ok:
            print(f"{label} {'ok  ' if ok else 'FAIL'} {i}: {what}: draft {fmt(said)}; computed {fmt(got)}")
    return bad


def normal(v):
    if isinstance(v, (list, tuple)):
        return [normal(x) for x in v]
    if isinstance(v, Fraction) and v.denominator == 1:
        return int(v)
    return v


def fmt(v):
    if isinstance(v, (list, tuple)):
        return "[" + ", ".join(fmt(x) for x in v) + "]"
    return str(v)


SNAP = "narration_lab/third-wave-20261024/root/tend-0610/intake/shamir_secret_sharing/"


def read(rel):
    with open(SNAP + rel, encoding="utf-8") as fh:
        return fh.read()


def script_lines(rel):
    out = {}
    for line in read(rel).splitlines():
        if line.startswith("#") or "\t" not in line:
            continue
        k, v = line.split("\t", 1)
        out[k] = v
    return out


def source_checks(facts):
    """The production storyboard and script against the draft, and main.rs's
    uncompiled SPOKEN test replayed in Python (its tables parsed from the source)."""
    import re
    ok = True
    rows = lambda md: [ln for ln in md.splitlines() if ln.startswith("| 01_")]  # noqa: E731
    same = rows(read("docs/storyboard.md")) == rows(read("docs/storyboard-draft.md"))
    print(f"{'ok  ' if same else 'FAIL'} chapter 01 rows: storyboard.md = storyboard-draft.md ({len(rows(read('docs/storyboard.md')))} rows)")
    prod, draft = script_lines("script.tsv"), script_lines("docs/script-draft.tsv")
    same_s = all(draft.get(k) == v for k, v in prod.items()) and sorted(prod) == sorted(k for k in draft if k.startswith("01_"))
    print(f"{'ok  ' if same_s else 'FAIL'} script.tsv lines = script-draft.tsv's 01 lines ({len(prod)} lines)")
    ok &= same and same_s
    main_rs = read("src/main.rs")
    spoken = re.findall(r'\(\s*"(\d\d_\d+)",\s*"([^"]+)",\s*"(S-[^"]+)",\s*"([^"]+)",?\s*\)', main_rs)
    not_count = re.findall(r'\("(\d\d_\d+)", "([^"]+)"\)', main_rs.split("NOT_A_COUNT")[1].split(";")[0])
    words = re.findall(r'"([a-z]+)"', main_rs.split("const NUMBER_WORDS")[1].split("];")[0])
    bad = []
    for lid, phrase, key, value in spoken:
        if phrase not in prod.get(lid, ""):
            bad.append(f"{lid} lacks {phrase!r}")
        if facts and facts.get(key) != value:
            bad.append(f"{lid} {phrase!r}: {key} is {facts.get(key)}, table says {value}")
    for lid, text in prod.items():
        spans = [(m.start(), m.end()) for lid2, ph in [(a, b) for a, b, *_ in spoken] + not_count if lid2 == lid
                 for m in re.finditer(re.escape(ph), text)]
        for m in re.finditer(r"[A-Za-z]+", text):
            if m.group().lower() in words and not any(a <= m.start() < b for a, b in spans):
                bad.append(f"{lid}: {m.group()!r} at {m.start()} is in no SPOKEN or NOT_A_COUNT phrase")
    for b2 in bad:
        print(f"FAIL {b2}")
    print(f"{'ok  ' if not bad else 'FAIL'} SPOKEN replay: {len(spoken)} rows, {len(not_count)} not-a-count, "
          f"{len(words)} number words, facts {'checked' if facts else 'not loaded'}")
    return ok and not bad


def oracle_compare(path):
    """Run the author's oracle (unchanged, from the repository root) and compare."""
    proc = subprocess.run([sys.executable, "-I", "-B", path, "--self-test"], capture_output=True, text=True)
    print(f"# oracle exit {proc.returncode}; its stderr follows verbatim")
    for line in proc.stderr.splitlines():
        print(f"# oracle stderr: {line}")
    facts = dict(line.split("\t", 1) for line in proc.stdout.splitlines() if "\t" in line)
    mine = {
        "S-F6.shares": "8,3,1,2,6",
        "S-F7.spins_by_secret": ",".join(f"{a}:{b}" for a, b in
                                         (two_share_spins(k, [(4, 2), (5, 6)], P) for k in range(P))),
        "S-F7.join.values_by_secret": "5,10,4,9,3,8,2,7,1,6,0",
        "S-F7.straight_one": str(two_share_spins_line([(4, 2), (5, 6)], P)),
        "S-F8.ten.pairs_that_leak": str(sum(1 for m in itertools.combinations(TRUSTEES, 2)
                                            if not learns_nothing(m, TEN))),
        "S-F8.ten.pair_secrets": ",".join(map(str, sorted(tally((4, 5), TEN)[(5, 5)]))),
        "S-F9.weights": "3,-3,1",
        "S-F9.weights_on_dial": ",".join(str(on_dial(q, P)) for q in lagrange_weights((1, 2, 3))),
        "S-F10.weights": "10/3,-5,8/3",
        "S-F10.weights_on_dial": ",".join(str(on_dial(q, P)) for q in lagrange_weights((2, 4, 5))),
        "S-F10.inverses": ",".join(f"{d}:{inv(d, P)}" for d in range(1, P)),
        "S-F11.trios_giving_secret": "10",
        "S-F11.most_zeros": "2",
        "S-F11.ten.opening_trio_secrets": ",".join(map(str, sorted(tally((1, 2, 3), TEN)[(9, 7, 9)]))),
        "S-F12.nonzero_last.excluded": "8",
        "S-F12.liar_result": "6",
        "S-F12.fourth.predicted": "5",
        "S-F12.fourth.actual": "2",
        "S-F13.four_of_seven.curves_left": "11",
        "S-F3.shamir.locks": str(math.comb(11, 5)),
        "S-F3.shamir.keys_each": str(math.comb(10, 5)),
    }
    diff = [(k, v, facts.get(k)) for k, v in mine.items() if facts.get(k) != v]
    for k, v, o in diff:
        print(f"# oracle DIFFERS {k}: mine {v}, oracle {o}")
    print(f"# oracle: {len(facts)} facts read; {len(mine)} compared with mine; {len(diff)} differ")
    return proc.returncode == 0 and not diff, facts


def main(argv):
    base = {"p": P, "secret": 5, "spins": (7, 7)}
    ok = True
    print("== claims, the draft's parameters (p = 11, secret 5, spins 7 and 7)")
    if run_claims(base, "claim"):
        ok = False

    print("\n== negative controls: one assumption broken; each must fail somewhere")
    controls = [
        ("ten-mark dial", {"p": TEN, "secret": 5, "spins": (7, 7)}, {"05_3", "11_6", "05_4"}),
        ("spins 7 and 6", {"p": P, "secret": 5, "spins": (7, 6)}, {"04_1/09_6", "10_1"}),
        ("b = 0 refused", {**base, "spin_rule": lambda s, sp: sp[1] != 0}, {"05_3", "11_6", "05_4"}),
        ("ten-mark spare dial", {**base, "spin_values": range(10)}, {"05_3", "11_6", "05_4"}),
    ]
    for name, prm, must in controls:
        try:
            bad = run_claims(prm, f"control[{name}]", quiet=True)
        except (TypeError, ValueError, KeyError, AssertionError) as e:
            print(f"control[{name}] raised {type(e).__name__}: {e}")
            bad = [("raised", "", "", "")]
        hit = {b[0] for b in bad}
        missing = must - hit
        if missing:
            ok = False
            print(f"control[{name}] DID NOT FAIL where expected: {sorted(missing)}")
        else:
            print(f"control[{name}] fails as expected at {sorted(must)} ({len(bad)} claims failed in all)")

    print("\n== side checks")
    for what, got, want in side_checks():
        if want is None:
            print(f"info {what}: {fmt(got)}")
            continue
        good = normal(got) == normal(want)
        ok &= good
        print(f"{'ok  ' if good else 'FAIL'} {what}: {fmt(got)}" + ("" if good else f" (expected {fmt(want)})"))

    print("\n== chapter 01 timeline estimate (no audio; words/wps, character shares)")
    script = script_lines("script.tsv")
    for wps in (2.4, 2.68, 3.0):
        late, total = timeline(script, wps, 0.81)
        cues = "; ".join(f"{lid} '{ph}' +{d}s" for lid, ph, d in late) or "none"
        print(f"wps {wps}: chapter ~{total:.1f} s; cues the picture reaches late: {cues}")

    facts = None
    if "--oracle" in argv:
        path = argv[argv.index("--oracle") + 1]
        print("\n== the author's oracle, run unchanged")
        good, facts = oracle_compare(path)
        ok &= good

    print("\n== source consistency (snapshot files)")
    ok &= source_checks(facts)

    print(f"\n== {'ALL HOLD' if ok else 'SOMETHING FAILED'}")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
