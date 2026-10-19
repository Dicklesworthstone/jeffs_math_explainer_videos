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
the fixed point 9/10, the tolerance 1/2, the waits' points, ε = 1/10, the two
polynomials of Davis's note) are restated here; everything else is computed.
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

print("\n".join(out))
if failures:
    print(f"{len(failures)} cross-check(s) failed", file=sys.stderr)
    sys.exit(1)
print(f"all cross-checks passed ({len(out)} keys)", file=sys.stderr)
