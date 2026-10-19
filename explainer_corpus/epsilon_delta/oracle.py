#!/usr/bin/env python3
"""Independent oracle for "Epsilon and Delta: Why Getting Closer Wasn't Enough".

    python3 -I -B oracle.py        key<TAB>value on stdout, cross-checks on stderr

Prints every fact key the video's `facts` command prints, computed without the
Rust model's methods: decimal powers and roots from Python's decimal module
(exact integer powers, correctly rounded square roots, 50-digit fractional
powers), exact comparisons with fractions.Fraction, and polynomial recentring
by binomial sums instead of synthetic division. `./x epsilon_delta facts-check`
compares the two outputs key by key. Standard library only.

Exit 1 if any cross-check fails. The inputs the video chooses (the powers drawn,
the fixed point 9/10, the tolerance 1/2, the waits' points, chapter 03's point
1/2, increments and tolerances, ε = 1/10, the two polynomials of Davis's note,
chapter 06's point a = 2 and its cap 1 on h) and chapter 03's page references
are restated here; everything else is computed.
"""
import math
import sys
from decimal import ROUND_HALF_UP, Decimal, localcontext
from fractions import Fraction

POWERS = [1, 2, 3, 4, 5, 10, 20, 50]
FIXED = Fraction(9, 10)
EXACT_UP_TO = 5
PLACES = 4
HALF = Fraction(1, 2)
WAITS = [Fraction(9, 10), Fraction(99, 100), Fraction(999, 1000)]
EPS = Fraction(1, 10)

failures = []


def check(ok, what):
    print(f"{'ok  ' if ok else 'FAIL'} {what}", file=sys.stderr)
    if not ok:
        failures.append(what)


def plain(d):
    """A Decimal as fixed-point text without trailing zeros ("0.729", "1")."""
    text = format(d.normalize(), "f")
    return text


def rounded(d, places):
    return str(d.quantize(Decimal(1).scaleb(-places), rounding=ROUND_HALF_UP))


def frac(f):
    return f"{f.numerator}/{f.denominator}"


def to_decimal(f):
    with localcontext() as ctx:
        ctx.prec = 120
        return Decimal(f.numerator) / Decimal(f.denominator)


out = []


def emit(key, value):
    out.append(f"{key}\t{value}")


# ---------------------------------------------------------------- chapter 01

emit("C1.domain.left", 0)
emit("C1.domain.right", 1)
emit("C1.powers", ",".join(map(str, POWERS)))
emit("C1.curves", len(POWERS))
emit("C1.squared", POWERS[1])
emit("C1.max_power", POWERS[-1])

x0 = to_decimal(FIXED)
emit("C1.fixed.x", plain(x0))
with localcontext() as ctx:
    ctx.prec = 200  # 0.9^50 has 50 places: integer powers stay exact
    for n in POWERS:
        h = x0**n
        check(Fraction(h) == FIXED**n, f"0.9^{n} is exact in decimal")
        emit(f"C1.fixed.n{n}.{PLACES}dp", rounded(h, PLACES))
        emit(f"C1.fixed.n{n}", plain(h))
emit("C1.fixed.exact_up_to", EXACT_UP_TO)

emit("C1.endpoint.height", 1**POWERS[-1])
# the limit, from the definition f = 0 on [0, 1) and f(1) = 1 (MIT 18.100B,
# lecture 20, Example 1); the fixed point is below 1
emit("C1.limit.below_one", 0 if FIXED < 1 else 1)
emit("C1.limit.at_one", 1)

emit("C1.half", frac(HALF))
witness = {}
with localcontext() as ctx:
    ctx.prec = 50
    for n in POWERS:
        x = Decimal(2) ** (Decimal(-1) / Decimal(n))
        witness[n] = x
        emit(f"C1.witness.n{n}.{PLACES}dp", rounded(x, PLACES))
        check(abs(x**n - Decimal("0.5")) < Decimal("1e-45"), f"x_{n}^{n} = 1/2")
        check(0 < x < 1, f"x_{n} = {x:.6f} lies inside [0, 1)")
        check(abs(float(x) - 2 ** (-1 / n)) < 1e-15, f"x_{n} agrees with floating point")
emit("C1.witness.gap", frac(HALF - 0))  # the curve at 1/2, the limit at 0 there

emit("C1.wait.tolerance", frac(HALF))
for x in WAITS:
    n = 1
    while x**n >= HALF:
        n += 1
    closed = math.floor(math.log(0.5) / math.log(x)) + 1
    check(n == closed, f"x = {float(x)}: the first n with x^n < 1/2 is {n} (closed form {closed})")
    check(x ** (n - 1) >= HALF > x**n, f"x = {float(x)}: n - 1 = {n - 1} does not work, n does")
    emit(f"C1.wait.x{plain(to_decimal(x))}", n)

# no single n: just past each crossing, still below 1, the gap is above 1/2
for n in range(1, 61):
    with localcontext() as ctx:
        ctx.prec = 50
        x = Decimal(2) ** (Decimal(-1) / Decimal(n))
    q = Fraction(x.quantize(Decimal("0.0001"), rounding=ROUND_HALF_UP)) + Fraction(1, 20000)
    if not (q < 1 and q**n > HALF):
        check(False, f"n = {n}: a point below 1 with gap above 1/2")
        break
else:
    check(True, "n = 1..60: a rational point below 1 with gap above 1/2 for each curve")

# the witness is sufficient, not the worst: the gap's supremum is 1
for n in POWERS:
    x = ((1 + 0.999) / 2) ** (1 / n)
    if not (x < 1 and x**n > 0.999):
        check(False, f"n = {n}: a gap above 0.999 below 1")
        break
else:
    check(True, "every drawn curve has gaps above 0.999 just below x = 1")

# ---------------------------------------------------------------- chapter 02

MORE_POWERS = [100, 200, 500, 1000]
ZOOMS = [10, 100, 1000]
PLOT_WIDTH = Fraction(48, 10)  # scene units: the plot's width
FRAME_WIDTH = Fraction(8) * Fraction(16, 9)  # 8 units tall at 16:9
PIXELS_ACROSS = 3840  # the final picture

emit("C2.powers", ",".join(map(str, MORE_POWERS)))


def width(n):
    """1 − 2^(−1/n) to 60 digits."""
    with localcontext() as ctx:
        ctx.prec = 60
        return 1 - Decimal(2) ** (Decimal(-1) / Decimal(n))


for n in [POWERS[-1], *MORE_POWERS]:
    emit(f"C2.width.n{n}.6dp", rounded(width(n), 6))
emit("C2.gap_at_one", 1 ** POWERS[-1] - 1)  # the curve and the limit are both 1 at x = 1
for i, m in enumerate(ZOOMS):
    emit(f"C2.zoom.{i}", m)
    emit(f"C2.window.x{m}", plain(1 - Decimal(1) / Decimal(m)))
    with localcontext() as ctx:
        ctx.prec = 60
        emit(f"C2.share.n{m}.3dp", rounded(m * width(m), 3))
emit("C2.zoom.step", ZOOMS[1] // ZOOMS[0])
LENS_WIDTH, LENS_HEIGHT = PLOT_WIDTH, Fraction(16, 10)  # scene units: the zoom window
PLOT_HEIGHT = Fraction(48, 10)
check(all(LENS_WIDTH / (PLOT_WIDTH / m) == m for m in ZOOMS),
      "the window labelled ×m draws [1 − 1/m, 1] exactly m times as wide as the plot does")
emit("C2.lens.heights", frac(LENS_HEIGHT / PLOT_HEIGHT))
emit("C2.restart_power", ZOOMS[0])  # 02_7 starts the paired run over at n = m = 10
with localcontext() as ctx:
    ctx.prec = 60
    ln2 = Decimal(2).ln()
emit("C2.ln2.4dp", rounded(ln2, 4))
emit("C2.ln2.2dp", rounded(ln2, 2))
px = PLOT_WIDTH / FRAME_WIDTH * PIXELS_ACROSS
check(px.denominator == 1, f"the plot spans {px} pixels of the 4K picture, a whole number")
emit("C2.pixels.plot_4k", px.numerator)
pixel_point, power_n, n = 1 - 1 / px, Fraction(1), 0
while power_n >= HALF:
    power_n *= pixel_point
    n += 1
emit("C2.subpixel_from", n)
check(width(n) < Decimal(1) / Decimal(px.numerator) <= width(n - 1),
      f"the strip is narrower than one of {px} pixels from n = {n}, not at n = {n - 1}")

# the two strips: curve above 1/2 on (x_n, 1], gap at least 1/2 on [x_n, 1)
def above_half(x, n):
    return x**n > HALF


def gap_at_least_half(x, n):
    return x < 1 and x**n >= HALF


check(above_half(Fraction(1), 50) and not gap_at_least_half(Fraction(1), 50),
      "x = 1 is in the curve-above strip and not in the gap strip")
check(not above_half(HALF, 1) and gap_at_least_half(HALF, 1),
      "n = 1, x = 1/2 exactly: the curve is not above 1/2, the gap is 1/2")
with localcontext() as ctx:
    ctx.prec = 60
    ok = True
    for n in [1, 2, 10, 50, 100, 200, 500, 898, 1000, 10**6]:
        t = ln2 / n
        w = width(n)
        ok = ok and t - t * t / 2 < w < t
    check(ok, "ln 2/n − (ln 2)²/(2n²) < 1 − x_n < ln 2/n at every n tried")
    scaled = [n * width(n) for n in [10, 100, 1000, 10**4, 10**5]]
    check(all(a < b < ln2 for a, b in zip(scaled, scaled[1:])), "n(1 − x_n) climbs toward ln 2 from below")
for n in range(1, 61):
    with localcontext() as ctx:
        ctx.prec = 60
        x4 = Decimal(2) ** (Decimal(-1) / Decimal(n))
    if Fraction(rounded(width(n), 4)) + Fraction(rounded(x4, 4)) != 1:
        check(False, f"n = {n}: the printed width is 1 minus the printed crossing")
        break
else:
    check(True, "n = 1..60: the printed width is 1 minus the printed crossing")

# ---------------------------------------------------------------- chapter 03

# the passage cited, as read in D. R. Wilkins's 2002 edition of The Analyst
# (restated, not computed): its note on the text (p. iii) dates the first
# editions 1734; §§XIII–XIV run over printed pages 5 and 6; the quoted
# sentence is on page 6
emit("C3.year", 1734)
emit("C3.edition", 2002)
emit("C3.page.first", 5)
emit("C3.page.last", 6)
emit("C3.quote.page", 6)

A = Fraction(1, 2)  # our modern example, not Berkeley's notation: the square at x = 1/2
NUDGES = [Fraction(1, 2), Fraction(1, 4), Fraction(1, 10), Fraction(-1, 2), Fraction(-1, 4), Fraction(-1, 10)]
TOLERANCES = [Fraction(1, 10), Fraction(1, 20)]


def fr(f):
    """A Fraction as "1/2", "-1/4", or "1" when whole."""
    return str(f.numerator) if f.denominator == 1 else f"{f.numerator}/{f.denominator}"


def in_h(coeffs):
    """A polynomial in h, lowest power first, as the screen writes it ("1 + h")."""
    terms = []
    for k, c in enumerate(coeffs):
        if c == 0:
            continue
        check(c.denominator == 1 and c > 0, f"the coefficient {c} of h^{k} is a whole positive number")
        power = "" if k == 0 else "h" if k == 1 else f"h^{k}"
        terms.append(str(c.numerator) if k == 0 else power if c == 1 else f"{c.numerator}{power}")
    return " + ".join(terms) or "0"


def slope(h):
    """The chord's slope from 1/2 to 1/2 + h, computed directly (raises at h = 0)."""
    return ((A + h) ** 2 - A**2) / h


emit("C3.power", 2)
emit("C3.base", fr(A))
emit("C3.base.height", fr(A**2))
# (a + h)^2 - a^2 by the binomial theorem: the coefficient of h^k is
# C(2, k) a^(2 - k), and the constant term cancels; dividing by h != 0
# lowers every power by one
change = [Fraction(0)] + [math.comb(2, k) * A ** (2 - k) for k in (1, 2)]
quotient = change[1:]
emit("C3.change", in_h(change))
emit("C3.quotient", in_h(quotient))
target = quotient[0]  # the slope is quotient[0] + quotient[1]·h, with quotient[1] = 1
emit("C3.limit", fr(target))
emit("C3.h.to", 0)
try:
    slope(Fraction(0))
    check(False, "the slope at h = 0 has no value")
except ZeroDivisionError:
    check((A + 0) ** 2 - A**2 == 0, "at h = 0 the change is 0 and the slope is 0/0: no value")
emit("C3.at_zero", f"{fr((A + 0) ** 2 - A**2)}/0")
for i, h in enumerate(NUDGES):
    s = slope(h)
    check(s == quotient[0] + quotient[1] * h, f"h = {fr(h)}: the slope {fr(s)} is 1 + h")
    emit(f"C3.h.{i}", fr(h))
    emit(f"C3.slope.{i}", plain(to_decimal(s)))
check(all(abs(slope(sign * Fraction(1, 10**k)) - target) == Fraction(1, 10**k) for k in range(1, 13) for sign in (1, -1)),
      "h = ±10^-k: the slope is exactly 10^-k from 1, k = 1..12")
for i, t in enumerate(TOLERANCES):
    w = t / abs(quotient[1])  # |slope - target| = |quotient[1]|·|h|
    inside = all(abs(slope(w * Fraction(k, 1000)) - target) < t for k in range(-999, 1000) if k)
    edge = all(abs(slope(h) - target) == t for h in (w, -w))
    check(inside and edge, f"tolerance {fr(t)}: every h with 0 < |h| < {fr(w)} tried is within it; |h| = {fr(w)} is not")
    emit(f"C3.eps.{i}", fr(t))
    emit(f"C3.delta.{i}", fr(w))

# ---------------------------------------------------------------- chapter 04

CHALLENGE, WIDE, LUCKY = Fraction(1, 10), Fraction(1, 5), Fraction(1, 20)
PICKS = [Fraction(-2, 25), Fraction(1, 25), Fraction(9, 100)]
GOOD, BAD = Fraction(1), Fraction(3, 2)


def dec(f):
    """A Fraction of either sign as its exact decimal."""
    return plain(to_decimal(f))


def within(v, centre, t):
    return abs(v - centre) < t


emit("C4.eps", fr(CHALLENGE))
w4 = CHALLENGE / abs(quotient[1])  # |slope - target| = |h|
emit("C4.delta", fr(w4))
for i, h in enumerate(PICKS):
    check(h != 0 and abs(h) < w4 and within(slope(h), target, CHALLENGE), f"pick {dec(h)} is in the window and lands in the band")
    emit(f"C4.pick.{i}", dec(h))
    emit(f"C4.pick_slope.{i}", dec(slope(h)))
emit("C4.wide", fr(WIDE))
check(abs(LUCKY) < WIDE and within(slope(LUCKY), target, CHALLENGE), "the lucky pick is inside the wide window and lands")
emit("C4.lucky", dec(LUCKY))
emit("C4.lucky_slope", dec(slope(LUCKY)))
# every pick stays an input of the square's domain: 0 <= 1/2 + h <= 1
REACH = min(A, 1 - A)


def in_domain_picks(d, steps=2000):
    """A grid of nonzero picks h with |h| < d and 1/2 + h in [0, 1]."""
    top = min(d, REACH)
    return [top * Fraction(k, steps) for k in range(-steps, steps + 1) if k and abs(top * Fraction(k, steps)) < d]


# the wide window loses: an independent grid search of its in-domain picks
# finds misses; the midpoint (t + delta)/2 shown is one of them
misses = [h for h in in_domain_picks(WIDE) if not within(slope(h), target, CHALLENGE)]
catch = (CHALLENGE + WIDE) / 2
check(bool(misses) and abs(catch) < WIDE and abs(catch) <= REACH and not within(slope(catch), target, CHALLENGE),
      f"the window {fr(WIDE)} loses at {dec(catch)} (grid search: {len(misses)} misses)")
check(all(any(not within(slope(h), target, CHALLENGE) for h in in_domain_picks(d, 400))
          for d in [Fraction(11, 100), Fraction(9, 10), Fraction(91, 100), Fraction(1), Fraction(7, 3), Fraction(1000)]),
      "grid search: every wider window tried, up to 1000, has an in-domain pick that misses")
check(all(abs(h) < d and abs(h) <= REACH and not within(slope(h), target, CHALLENGE)
          for d in [Fraction(9, 10), Fraction(91, 100), Fraction(7, 3)] for h in [min((CHALLENGE + d) / 2, REACH)]),
      "the model's formula min((t + d)/2, 1/2) checked: in the window and the domain, and a miss")
emit("C4.catch", dec(catch))
emit("C4.catch_slope", dec(slope(catch)))


def g(h, fill):
    """The slope's graph with its hole filled."""
    return fill if h == 0 else slope(h)


# filled with 1: the grid holds h = 0 and every in-domain pick (the
# argument for every h is |g(h) - 1| = |h| < delta = epsilon, and g(0) = 1)
check(all(within(g(h, GOOD), g(0, GOOD), CHALLENGE) for h in [Fraction(0)] + in_domain_picks(w4, 200)),
      "filled with 1, the window |h| < 1/10 holds h = 0 and every grid pick (sampled)")
emit("C4.fill.good", fr(GOOD))
emit("C4.fill.bad", dec(BAD))
apart = abs(target - BAD)
room = apart - CHALLENGE
gap4 = apart - room / 2
check(room < REACH and gap4 > CHALLENGE, f"room {fr(room)} inside the reach, gap {fr(gap4)} over the tolerance")
check(all(abs(g(h, BAD) - BAD) >= gap4 for h in [room / 2 * Fraction(k, 500) for k in range(1, 501)]),
      "every pick in (0, room/2] leaves g at least the gap from 1.5 (sampled)")
emit("C4.fill.room", fr(room))
emit("C4.fill.gap", fr(gap4))
# filled with 3/2: an independent grid search finds a losing in-domain pick
# in every window tried; h = 0 passes in all of them (g(0) = 3/2)
windows = [Fraction(1, 1000), Fraction(1, 40), Fraction(1, 10), Fraction(2, 5), Fraction(1), Fraction(7, 3)]
check(all(within(g(0, BAD), BAD, CHALLENGE) and any(not within(g(h, BAD), BAD, CHALLENGE) for h in in_domain_picks(d, 400))
          for d in windows),
      "filled with 3/2, grid search: every window tried has a losing in-domain pick, and h = 0 passes")
witness = min(w4, room) / 2
check(all(0 < h < d and abs(h) <= REACH and not within(g(h, BAD), BAD, CHALLENGE)
          for d in windows for h in [min(d, room, REACH) / 2]),
      "the model's formula min(d, room, 1/2)/2 checked: in the window and the domain, and losing")
emit("C4.fill.witness", dec(witness))
emit("C4.fill.witness_value", dec(g(witness, BAD)))
blind = abs(witness) / 2
check(not abs(witness) < blind, "a window of |h|/2 does not hold h")
emit("C4.cheat.delta", fr(blind))

# continuity of x^2 at 1 on [0, 1] against 1/2: |x^2 - 1| < 1/2 exactly when
# x^2 > 1/2, that is x > 2^(-1/2); x = 1 is in the domain and the window
emit("C4.end.x", 1)
emit("C4.end.power", 2)
emit("C4.end.eps", frac(HALF))
emit("C4.end.level", fr(1 - HALF))
with localcontext() as ctx:
    ctx.prec = 60
    edge = Decimal(2).sqrt() / 2
check(abs(edge**2 - Decimal("0.5")) < Decimal("1e-55") and 1**2 > HALF, "the window's edge is where x^2 = 1/2, and x = 1 is inside")
emit("C4.end.edge.4dp", rounded(edge, 4))
emit("C4.end.delta.4dp", rounded(1 - edge, 4))

# ---------------------------------------------------------------- chapter 05

# continuity of x^2 at 1/2 on the domain [0, 1] against 1/10, searched for
A5, EPS5 = Fraction(1, 2), Fraction(1, 10)
emit("C5.a", fr(A5))
emit("C5.value", fr(A5 * A5))
# the factor x + 1/2 over [0, 1]: a straight line, so its largest value is at
# an end; check the ends and a grid independently
grid = [Fraction(k, 1000) for k in range(1001)]
bound5 = max(x + A5 for x in (Fraction(0), Fraction(1)))
check(all(x + A5 <= bound5 for x in grid) and Fraction(1) + A5 == bound5, f"x + 1/2 <= {fr(bound5)} on [0, 1], equal at x = 1")
emit("C5.bound", fr(bound5))
emit("C5.near", fr(A5 + A5))
ratio5 = 1 / bound5
emit("C5.ratio", fr(ratio5))
emit("C5.eps", fr(EPS5))
delta5 = ratio5 * EPS5
# the chain on every grid point: |x^2 - 1/4| = |x - 1/2| (x + 1/2) <= (3/2)|x - 1/2|,
# and so every grid point of the window passes (sampled; the chain is the proof)
check(all(abs(x * x - A5 * A5) <= bound5 * abs(x - A5) for x in grid), "|x^2 - 1/4| <= (3/2)|x - 1/2| at every grid point of [0, 1]")
check(all(abs(x * x - A5 * A5) < EPS5 for x in grid if abs(x - A5) < delta5), f"every grid point with |x - 1/2| < {fr(delta5)} passes")
emit("C5.delta", fr(delta5))
emit("C5.delta.3dp", rounded(to_decimal(delta5), 3))
# the widest symmetric window, measured from the curve: the right side
# reaches x^2 = 1/4 + 1/10 first
with localcontext() as ctx:
    ctx.prec = 60
    right = (to_decimal(A5 * A5 + EPS5)).sqrt() - to_decimal(A5)
    left = to_decimal(A5) - (to_decimal(A5 * A5 - EPS5)).sqrt()
widest5 = min(right, left)
check(right < left and Fraction(delta5) < Fraction(widest5), f"the widest window {widest5:.6f} (right side) is wider than {fr(delta5)}")
beyond = A5 + Fraction(2, 25)
check(abs(beyond - A5) > delta5 and abs(beyond * beyond - A5 * A5) < EPS5, "x = 0.58 is outside the window found and still passes")
emit("C5.widest.3dp", rounded(widest5, 3))

# ------------------------------------------------- continuation (brief §3–4)


def recenter(coeffs, a):
    """P(a + h) = sum c_k h^k with c_k = sum_{j >= k} p_j C(j, k) a^(j - k)."""
    return [sum(p * math.comb(j, k) * a ** (j - k) for j, p in enumerate(coeffs) if j >= k) for k in range(len(coeffs))]


def bound(shifted):
    return sum(abs(c) for c in shifted[1:])


def davis(c, eps):
    return min(Fraction(1), eps / c) if c else Fraction(1)


def change(shifted, h):
    return sum(c * h**k for k, c in enumerate(shifted) if k >= 1)


emit("S.eps", frac(EPS))
with localcontext() as ctx:
    ctx.prec = 60
    sprague = (Decimal(4) + to_decimal(EPS)).sqrt() - 2
    check(abs(sprague**2 + 4 * sprague - to_decimal(EPS)) < Decimal("1e-55"), "Sprague's δ solves t² + 4t = ε")
emit("S.delta_sprague.10dp", rounded(sprague, 10))
# an exact half rounds up: at ε = δ² + 4δ for a rational δ, Decimal's square
# root is exact, so the rounding below meets the tie itself (the reviewer's
# ε = 81/400 gives δ = 1/20, printed 0.1 at one place)
ties = {(Fraction(81, 400), 1): "0.1"}
for d, places, want in [(Fraction(1, 40), 2, "0.03"), (Fraction(1, 8), 2, "0.13"), (Fraction(3, 400), 3, "0.008")]:
    ties[(d * d + 4 * d, places)] = want
for (eps, places), want in ties.items():
    with localcontext() as ctx:
        ctx.prec = 60
        exact_delta = (Decimal(4) + to_decimal(eps)).sqrt() - 2
    check(rounded(exact_delta, places) == want, f"ε = {frac(eps)}: δ = {exact_delta} prints {want} at {places} places")
square = recenter([0, 0, 1], 2)
emit("S.square_at_2", ",".join(map(str, square)))
simple = davis(bound(square), EPS)
emit("S.delta_simple", frac(simple))
emit("S.delta_simple.decimal", plain(to_decimal(simple)))
check(simple < Fraction(sprague), "the simpler δ is smaller than Sprague's, and still works:")
check(abs(change(square, simple)) < EPS and abs(change(square, -simple)) < EPS, "  |h| = 1/50 keeps x² within 1/10")

cubic = recenter([-6, 0, 5, 1], 1)
emit("D1.shifted", ",".join(map(str, cubic)))
emit("D1.bound", bound(cubic))
d1 = davis(bound(cubic), EPS)
emit("D1.delta", frac(d1))
worst = max(abs(change(cubic, d1 * Fraction(m, 1000))) for m in range(-1000, 1001))
check(worst < EPS, f"Davis's δ = {frac(d1)} keeps the cubic within 1/10 (largest change {float(worst):.5f})")

quartic = recenter([2, -8, 0, 0, -3], -2)
emit("D2.value_at_a", quartic[0])
emit("D2.shifted", ",".join(map(str, quartic[1:])))
emit("D2.bound", bound(quartic))
check(all(sum(p * (-2 + h) ** j for j, p in enumerate([2, -8, 0, 0, -3])) == quartic[0] + change(quartic, h)
          for h in range(-5, 6)), "the quartic's recentring reproduces P(-2 + h)")

eps_naive = Fraction(1)
naive = eps_naive / square[1]
emit("N.naive.delta", frac(naive))
h = Fraction(math.ceil(naive * 100) - 1, 100)
emit("N.naive.h", frac(h))
moved = change(square, h)
check(abs(h) < naive and moved > eps_naive, f"the naive δ fails: h = {frac(h)} moves x² by {frac(moved)}")
emit("N.naive.change", frac(moved))

# ---------------------------------------------------------------- chapter 06

# x² on all real numbers at a = 2 (Sprague's example), recentred by
# evaluating (2 + h)² − 4 at h = −1, 0, 1, not by the recentring above
A6 = 2


def change6(h):
    return (A6 + h) ** 2 - A6**2


lin6 = Fraction(change6(1) - change6(-1), 2)
quad6 = Fraction(change6(1) + change6(-1), 2) - change6(0)
check(change6(0) == 0 and quad6 == 1 and all(change6(Fraction(n, 7)) == Fraction(n, 7) ** 2 + lin6 * Fraction(n, 7)
                                              for n in range(-30, 31)), "(2 + h)² − 4 = h² + 4h")
emit("C6.a", A6)
emit("C6.h.at_a", A6 - A6)
emit("C6.value", A6**2)
emit("C6.slope", int(lin6))
half6 = lin6 / 2
check(all((t + half6) ** 2 == t * t + lin6 * t + half6**2 for t in [Fraction(n, 9) for n in range(40)]),
      "|h|² + 4|h| + 4 = (|h| + 2)²")
emit("C6.half_slope", int(half6))
emit("C6.added", int(half6**2))
naive6 = EPS / lin6
emit("C6.naive.delta", frac(naive6))
# the first four-place h > 0 where h² + 4h reaches ε, scanned from zero
k6 = next(k for k in range(1, 10**4) if change6(Fraction(k, 10**4)) >= EPS)
h6 = Fraction(k6, 10**4)
check(h6 < naive6 and change6(h6) > EPS and change6(h6 - Fraction(1, 10**4)) < EPS,
      f"h = {plain(to_decimal(h6))} is inside ε/4 = {frac(naive6)} and moves x² by {plain(to_decimal(change6(h6)))} > ε")
emit("C6.naive.h", rounded(to_decimal(h6), PLACES))
emit("C6.naive.change.5dp", rounded(to_decimal(change6(h6)), PLACES + 1))
check(Decimal(rounded(to_decimal(change6(h6)), PLACES + 1)) > to_decimal(EPS), "  and still reads past ε at five places")
emit("C6.delta.4dp", rounded(sprague, PLACES))
check(Fraction(sprague) < h6, "the loss lies past Sprague's δ")
# the note cited, as read: the owner's scan of its page (title, author, the
# expansion and δ) and Crossref's record for DOI 10.2307/2308663 (The
# American Mathematical Monthly 67(8), October 1960, p. 780)
emit("C6.note.volume", 67)
emit("C6.note.year", 1960)
emit("C6.note.page", 780)
emit("C6.cap", 1)
cap_factor = 2 * A6 + 1
check(all(abs(A6 + Fraction(m, 1000) + A6) < cap_factor for m in range(-999, 1000)),
      f"|h| < 1 keeps |x + 2| below {cap_factor}")
emit("C6.cap.factor", cap_factor)
check(min(Fraction(1), EPS / cap_factor) == simple, "the cap's δ = min(1, ε/5) is the simple δ")
# every tolerance, not only one tenth: Sprague's δ is positive and every h
# strictly inside it passes, either sign and zero; the slope's ε/4 is wider;
# the cap's δ is never wider
for eps6 in [Fraction(1, 1000), EPS, Fraction(1), Fraction(5), Fraction(21), Fraction(1000)]:
    with localcontext() as ctx:
        ctx.prec = 60
        d6 = Fraction((Decimal(4) + to_decimal(eps6)).sqrt() - 2)
    below = d6 * Fraction(999_999, 1_000_000)
    inside = [below * Fraction(m, 50) for m in range(-50, 51)]
    capped6 = min(Fraction(1), eps6 / cap_factor)
    check(d6 > 0 and all(abs(change6(h)) < eps6 for h in inside) and eps6 / lin6 > d6
          and capped6**2 + lin6 * capped6 <= eps6,
          f"ε = {frac(eps6)}: Sprague's δ ≈ {float(d6):.6g} > 0 works for every h inside, either sign or zero")

print("\n".join(out))
if failures:
    print(f"{len(failures)} cross-check(s) failed", file=sys.stderr)
    sys.exit(1)
print(f"all cross-checks passed ({len(out)} keys)", file=sys.stderr)
