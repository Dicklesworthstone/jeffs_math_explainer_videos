#!/usr/bin/env python3
"""Facts oracle for "The Hidden Geometry of Complex Functions" (Opus candidate).

Run from this folder (`./x complex_analysis_visual facts-check` does):

    python3 -I -B oracle.py

stdout: one `KEY<TAB>VALUE` line per fact, the keys src/facts.rs prints, in
the same order. stderr: cross-checks (lines starting with "#"); exit 1 if
any failed.

This is the author's second computation, not an independent review. It
avoids the Rust model's algorithms wherever another one exists:
  * the bump's recipes are closed-form geometric sums, or exact rational
    recurrences from (1 + (a + t)^2) f = 1, never the residue formula
    c_k = -Im[(i - a)^-(k+1)] the model uses;
  * every turn count is zeros minus poles found by numpy.roots on the
    numerator and denominator, counted by |z - centre| < r, never an angle
    sum;
  * gamma is scipy.special.gamma, never Lanczos;
  * zeta is Borwein's accelerated alternating series (algorithm 2 of
    P. Borwein, "An efficient algorithm for the Riemann zeta function",
    1995), with the functional equation and scipy's gamma to the left of
    Re s = 1/2, never Euler-Maclaurin;
  * the zeros on the critical line are sign changes of Hardy's Z function,
    Z(t) = Re(e^{i theta(t)} zeta(1/2 + it)), refined by bisection, never
    Newton;
  * residues are limits (z - p) f(z) at z = p + eps, averaged over eps,
    never 1/h'(p);
  * loop sums are evaluated in closed form (geometric sums of roots of
    unity), never by summing products.
"""
import cmath
import math
import sys
from fractions import Fraction

import numpy as np
import scipy.special as sp

PI, TAU = math.pi, math.tau
rows = []
failures = []


def fixed(x, places):
    """x to `places` decimals; a value that rounds to zero has no sign."""
    s = f"{x:.{places}f}"
    if s.startswith("-") and set(s[1:]) <= {"0", "."}:
        return s[1:]
    return s


def lst(xs, places):
    return ",".join(fixed(x, places) for x in xs)


def add(key, value):
    rows.append((key, str(value).lower() if isinstance(value, bool) else str(value)))


def check(name, ok, detail=""):
    print(f"# {'ok  ' if ok else 'FAIL'} {name} {detail}", file=sys.stderr)
    if not ok:
        failures.append(name)


def deg(w):
    return math.degrees(cmath.phase(w))


def roots_inside(coeffs, centre, r):
    """Roots of the polynomial (numpy order, highest first) inside the circle, with multiplicity."""
    if len(coeffs) <= 1:
        return 0
    return sum(1 for z in np.roots(coeffs) if abs(z - centre) < r)


def turns_rational(num, den, centre, r):
    """Zeros minus poles of num/den inside the circle (no common roots in our examples)."""
    return roots_inside(num, centre, r) - roots_inside(den, centre, r)


# ------------------------------------------------------------ zeta, Borwein
def borwein_d(n):
    d = []
    total = Fraction(0)
    for i in range(n + 1):
        total += Fraction(math.factorial(n + i - 1) * 4 ** i, math.factorial(n - i) * math.factorial(2 * i)) * n
        d.append(total)
    return [float(x) for x in d]


BORWEIN_N = 60
D = borwein_d(BORWEIN_N)


def zeta_borwein(s):
    """zeta(s) for Re s > 0, s != 1: eta(s)/(1 - 2^(1-s)), eta by Borwein's sum."""
    n = BORWEIN_N
    total = sum((-1) ** k * (D[k] - D[n]) / (k + 1) ** s for k in range(n))
    eta = -total / D[n]
    return eta / (1 - 2 ** (1 - s))


def zeta(s):
    if s.real >= 0.5:
        return zeta_borwein(s)
    # functional equation, scipy's gamma
    return 2 ** s * PI ** (s - 1) * cmath.sin(PI * s / 2) * sp.gamma(1 - s) * zeta_borwein(1 - s)


def hardy_z(t):
    theta = sp.loggamma(0.25 + 0.5j * t).imag - 0.5 * t * math.log(PI)
    return (cmath.exp(1j * theta) * zeta_borwein(complex(0.5, t))).real


def critical_zeros(lo, hi, step=0.02):
    found = []
    t = lo
    a = hardy_z(t)
    while t < hi:
        b = hardy_z(t + step)
        if a == 0 or a * b < 0:
            x, y, fx = t, t + step, a
            for _ in range(80):
                m = 0.5 * (x + y)
                fm = hardy_z(m)
                if fx * fm <= 0:
                    y = m
                else:
                    x, fx = m, fm
            found.append(0.5 * (x + y))
        t, a = t + step, b
    return found


def residue(f, p):
    """lim (z - p) f(z), estimated on a small circle around p."""
    eps = 1e-5
    vals = [(eps * cmath.exp(1j * TAU * k / 8)) * f(p + eps * cmath.exp(1j * TAU * k / 8)) for k in range(8)]
    return sum(vals) / 8


def bump(x):
    return 1 / (1 + x * x)


def recipe0_closed(x, degree):
    """1 - x^2 + ... + (-x^2)^(degree/2) as a closed geometric sum."""
    q = -x * x
    m = degree // 2 + 1
    return (1 - q ** m) / (1 - q)


def recipe_exact(a, x, terms):
    """Partial sum of the bump's series about a, coefficients by the exact recurrence
    (1 + a^2) c_k + 2a c_(k-1) + c_(k-2) = [k = 0]."""
    a, t = Fraction(a), Fraction(x) - Fraction(a)
    c = []
    for k in range(terms):
        rhs = Fraction(1 if k == 0 else 0)
        if k >= 1:
            rhs -= 2 * a * c[k - 1]
        if k >= 2:
            rhs -= c[k - 2]
        c.append(rhs / (1 + a * a))
    return float(sum(ck * t ** k for k, ck in enumerate(c)))


# ------------------------------------------------------------- 01 the bump
add("bump.peak", fixed(bump(0.0), 0))
add("bump.read_at", fixed(1.1, 1))
add("bump.at1_1", fixed(bump(1.1), 3))
add("bump.recipe40.at0_5", fixed(recipe0_closed(0.5, 40), 6))
add("bump.recipe40.at0_9", fixed(recipe0_closed(0.9, 40), 3))
add("bump.recipe40.at1_1", fixed(recipe0_closed(1.1, 40), 1))
# radius: smallest |root| of 1 + z^2 seen from 0
add("bump.radius0", fixed(min(abs(z) for z in np.roots([1, 0, 1])), 0))
add("bump.poles", lst(sorted((z.imag for z in np.roots([1, 0, 1])), reverse=True), 0))

# ---------------------------------------------------------- 02 turning
s = (3 + 2j) + (1 - 1j)
add("turn.sum", lst([s.real, s.imag], 0))
w = 1 + 1j
add("turn.one_plus_i.length", fixed(math.sqrt(2), 3))
add("turn.one_plus_i.angle_deg", fixed(math.degrees(math.atan2(1, 1)), 0))
w2 = complex(1 * 1 - 1 * 1, 2 * 1 * 1)  # (x^2 - y^2) + 2xy i
add("turn.square.value", lst([w2.real, w2.imag], 0))
add("turn.square.length", fixed(abs(w) ** 2, 0))
add("turn.square.angle_deg", fixed(2 * 45.0, 0))

# ---------------------------------------------------------- 03 two planes
add("square.circle.out_radius", fixed(0.5 ** 2, 2))
add("identity.circle.turns", turns_rational([1, 0], [1], 0, 0.5))
add("square.circle.turns", turns_rational([1, 0, 0], [1], 0, 0.5))
add("exp.circle_radii", lst([math.exp(-1), 1.0, math.e], 3))
c = complex(math.cos(TAU), math.sin(TAU))
add("exp.after_2pi", lst([c.real, c.imag], 6))

# ---------------------------------------------------------- 04 reading colour
add("color.band_ratio", fixed(math.exp(TAU / 12), 2))
add("color.square.turns", turns_rational([1, 0, 0], [1], 0, 0.5))
add("color.recip.turns", turns_rational([1], [1, 0], 0, 0.5))
add("color.bump.turns_at_i", turns_rational([1], [1, 0, 1], 1j, 0.5))
add("color.bump.turns_at_minus_i", turns_rational([1], [1, 0, 1], -1j, 0.5))

# ---------------------------------------------------------- 05 gallery
add("gallery.cubic.turns_r1_8", turns_rational([1, 0, 0, -1], [1], 0, 1.8))
zdp_den = [1, 2, 1]  # (z + 1)^2
add("gallery.zdp.turns_at_minus1", turns_rational([1, -1], zdp_den, -1, 0.3))
add("gallery.zdp.turns_at_1", turns_rational([1, -1], zdp_den, 1, 0.3))
add("gallery.exp.period", fixed(2 * PI, 2))
sin_i = 1j * math.sinh(1)
sin_3i = 1j * math.sinh(3)
add("gallery.sin.at_i", lst([sin_i.real, sin_i.imag], 2))
add("gallery.sin.at_3i", lst([sin_3i.real, sin_3i.imag], 1))
up, down = cmath.tan(0.4 + 12j), cmath.tan(0.4 - 12j)
add("gallery.tan.far", lst([up.real, up.imag, down.real, down.imag], 4))
add("gallery.gamma.at4", fixed(sp.gamma(4.0), 0))
add("gallery.gamma.at5", fixed(sp.gamma(5.0), 0))
poles = [-k for k in range(5) if not math.isfinite(sp.gamma(float(-k)))]
add("gallery.gamma.poles", lst(poles, 0))
zeros = critical_zeros(10.0, 31.0)
add("gallery.zeta.zeros", lst(zeros, 2))
check("four zeros below height 31", len(zeros) == 4, str([round(z, 6) for z in zeros]))
odlyzko = [14.134725141734694, 21.022039638771555, 25.010857580145689, 30.424876125859513]
check("Z sign changes match Odlyzko", len(zeros) == 4 and all(abs(a - b) < 1e-9 for a, b in zip(zeros, odlyzko)))
add("gallery.zeta.strip_top", fixed(32, 0))
check("the strip drawn holds all four zeros", zeros[-1] < 32)
trivial =[x for x in (-2.0, -4.0, -6.0) if abs(zeta(complex(x))) < 1e-12]
add("gallery.zeta.trivial", lst(trivial, 0))

# ---------------------------------------------------------- 06 one derivative
z0 = 1 + 1j
d = 2 * z0  # d/dz z^2, by the power rule
add("deriv.value", lst([d.real, d.imag], 3))
add("deriv.scale", fixed(abs(d), 2))
add("deriv.turn_deg", fixed(deg(d), 0))
# the worst gap |h^2| sits at a corner, |h| = side/sqrt 2, over |2 z0| side
for side, key, places in [(0.5, "deriv.gap.side0_5", 3), (0.05, "deriv.gap.side0_05", 4)]:
    add(key, fixed((side * side / 2) / (abs(d) * side), places))
x0, y0 = 1.0, 1.0  # u = x^2 - y^2, v = 2xy
add("deriv.jacobian", lst([2 * x0, -2 * y0, 2 * y0, 2 * x0], 3))
add("deriv.conj", lst([1, -1], 0))  # conj: u = x, v = -y; along h and along ih
add("deriv.squash", lst([1, 2], 0))  # x + 2iy

# ---------------------------------------------------------- 07 one point
add("critical.angle_in", fixed(90.0, 0))
add("critical.angle_out", fixed(deg((1j) ** 2) - deg(1 ** 2), 0))
# z^2 multiplies a step h by 2 z1: the two steps' images differ by exactly the factor i
z1 = complex(-0.7, 0.6)
add("critical.angle_kept", fixed(deg((2 * z1 * 1j) / (2 * z1)), 0))
check("z1 is not the critical point", abs(2 * z1) > 1)
add("onepoint.at0", fixed(1e-6, 4))  # | |h|^2 / h | = |h|
add("onepoint.at1", lst([2.0, 0.0], 0))  # (|1 + h|^2 - 1)/h -> 2 along reals; -ih along i h
add("onepoint.squeeze.radii", lst([0.55, 0.06], 2))  # chosen: the ratio's window holds |h| up to 0.6

# ---------------------------------------------------------- 08 loop sums
n = 24
delta = TAU / n


def loop_closed(p):
    """Midpoint loop sum of z^p around |z| = 1: sum_k e^{ip(theta_k + delta/2)} e^{i theta_k}(e^{i delta} - 1)."""
    m = (p + 1) % n
    ring = n if m == 0 else 0  # sum_k e^{i(p+1)theta_k}
    return ring * cmath.exp(1j * p * delta / 2) * (cmath.exp(1j * delta) - 1)


add("loop.z.total", fixed(abs(loop_closed(1)), 6))
add("loop.inv.total24", fixed(2 * n * math.sin(PI / n), 2))
add("loop.inv.all_up", True)  # each product is e^{-i delta/2}(e^{i delta} - 1) = 2i sin(delta/2)
add("loop.two_pi", fixed(TAU, 2))
add("loop.powers.closed", lst([p for p in (-3, -2, -1, 0, 1, 2) if abs(loop_closed(p)) < 1e-12], 0))
check("1/z loop sum closed form", abs(loop_closed(-1) - 2j * n * math.sin(PI / n)) < 1e-12)

# ---------------------------------------------------------- 09 the boundary
# averaging over N points keeps only the powers divisible by N
add("cauchy.samples", "8,16")
add("cauchy.r_half.n8", fixed(1 / (1 - 0.5 ** 8), 4))
add("cauchy.r_half.n16", fixed(1 / (1 - 0.5 ** 16), 6))
# outside the poles: 1/(1 + z^2) = sum_{k>=1} (-1)^(k-1) z^(-2k); keep 2k = 16, 32, ...
add("cauchy.r_two.n16", fixed(-(2.0 ** -16) / (1 + 2.0 ** -16), 6))
pole_share = residue(lambda z: 1 / (z * (1 + z * z)), 1j).real
add("cauchy.pole_share", fixed(pole_share, 1))
# the residue at 0 of f(z)/z is f(0) = 1; the pole at -i has the same share
add("cauchy.r_two.limit", fixed(1 + 2 * pole_share, 0))
check("the 16-point average at radius 2 is near that limit", abs(-(2.0 ** -16) / (1 + 2.0 ** -16) - (1 + 2 * pole_share)) < 1e-4)
alias =1 / (1 - 0.5 ** 16)
add("cauchy.coef", lst([alias, 0.0, -alias, 0.0, alias], 3))
r1 = abs(1 - 1j)
add("recentre1.radius", fixed(r1, 3))
add("recentre1.left", fixed(1 - r1, 2))
add("recentre1.right", fixed(1 + r1, 2))
add("recentre1.check.terms", 60)
add("recentre1.check.at", f"2,{float(Fraction(13, 5))}")
add("recentre1.check.at2", fixed(recipe_exact(1, 2, 60), 6))
add("recentre1.check.at2_6", fixed(recipe_exact(1, Fraction(13, 5), 60), 0))
check("recentred recipe converges at 2", abs(recipe_exact(1, 2, 60) - bump(2)) < 1e-8)

# ---------------------------------------------------------- 10 rigidity
s3, c3 = -math.sinh(3) ** 2, math.cosh(3) ** 2  # sin(3i)^2, cos(3i)^2
add("rigid.trig", lst([s3, c3], 2))
add("rigid.trig.sum", fixed(s3 + c3, 6))
add("rigid.sin_recip.zeros", lst([1 / (PI * k) for k in (1, 2, 3)], 3))

# ---------------------------------------------------------- 11 counting
local = [turns_rational([1, 0], [1], 0, 0.5), turns_rational([1, 0, 0], [1], 0, 0.5), turns_rational([1], [1, 0], 0, 0.5)]
add("argument.local", ",".join(str(t) for t in local))
moeb_num, moeb_den = [1, -0.45], [1, -1.35]
add("argument.rational.r0_8", turns_rational(moeb_num, moeb_den, 0, 0.8))
add("argument.rational.r1_7", turns_rational(moeb_num, moeb_den, 0, 1.7))
add("argument.rational.r1_35_refused", True)  # the circle |z| = 1.35 passes through the pole


def eighth_power_turns(n):
    """Sum of the principal angle changes of z^8 over n equal steps of the unit circle."""
    ws = np.exp(8j * TAU * np.arange(n) / n)
    return float(np.sum(np.angle(np.roll(ws, -1) / ws)) / TAU)


add("argument.coarse.power", 8)
add("argument.coarse.steps", "8,64")
add("argument.coarse.naive", round(eighth_power_turns(8)))
add("argument.coarse.lipschitz", fixed(8, 0))  # |f'| = 8|z|^7 = 8 on the circle
add("argument.coarse.size", fixed(abs(1 ** 8), 0))
coarse_reach = [8 * TAU / n for n in (8, 64)]
add("argument.coarse.reach", lst(coarse_reach, 2))
add("argument.coarse.refused", coarse_reach[0] >= 1)
add("argument.coarse.certified", round(eighth_power_turns(64)))
check("z^8 turns 8 times by a fine sum", abs(eighth_power_turns(4096) - 8) < 1e-9)
check("the fine step passes its test", coarse_reach[1] < 1)
quintic = [1, 0, 0, 0, -3, 1]
add("argument.quintic.unit", turns_rational(quintic, [1], 0, 1.0))
add("argument.quintic.big", turns_rational(quintic, [1], 0, 3.0))
qr = sorted(np.roots(quintic), key=abs)
add("argument.quintic.inside", sum(1 for z in qr if abs(z) < 1))
add("argument.quintic.smallest", fixed(abs(qr[0]), 3))
# Rouché on |z| = 1: |z^5 + 1| <= 2 < 3 = |-3z|, checked densely
ring = np.exp(1j * np.linspace(0, TAU, 20001))
check("leash on |z| = 1", bool(np.all(np.abs(ring ** 5 + 1) < np.abs(-3 * ring))))
big = 3 * ring
check("leash on |z| = 3", bool(np.all(np.abs(-3 * big + 1) < np.abs(big ** 5))))

# ---------------------------------------------------------- 12 residues
res_i = residue(lambda z: 1 / (1 + z * z), 1j)
add("residue.bump.at_i", lst([res_i.real, res_i.imag], 1))

# ---------------------------------------------------------- 13 a real integral
add("integral.bump.loop", fixed((2j * PI * res_i).real, 5))
add("integral.bump.radii", lst([2, 5, 10], 0))
add("integral.bump.arc", lst([PI * R / (R * R - 1) for R in (2.0, 5.0, 10.0)], 3))
q_roots = np.roots([1, 0, 0, 0, 1])
add("integral.quartic.roots_deg", lst(sorted(math.degrees(cmath.phase(p)) % 360 for p in q_roots), 0))
upper = [p for p in q_roots if p.imag > 0]
add("integral.quartic.inside", len(upper))
qsum = sum(residue(lambda z: 1 / (1 + z ** 4), p) for p in upper)
add("integral.quartic.sum", lst([qsum.real, qsum.imag], 5))
value = (2j * PI * qsum).real
add("integral.quartic.value", fixed(value, 4))
add("integral.quartic.value2", fixed(value, 2))
add("integral.quartic.radii", lst([2, 10], 0))
add("integral.quartic.arc", lst([PI * R / (R ** 4 - 1) for R in (2.0, 10.0)], 4))
# the largest n with arc(10) < 1/n, to the hundred below: (10^4 - 1)/(10 pi) = 318.3...
add("integral.quartic.under", fixed(100 * ((10 ** 4 - 1) // (1000 * PI)), 0))
# the real integral itself, by substitution x = tan(u) and Gauss-Legendre on (-pi/2, pi/2)
u, wts = np.polynomial.legendre.leggauss(400)
u = u * PI / 2
numeric = float(np.sum(wts * (PI / 2) / (np.cos(u) ** 2) / (1 + np.tan(u) ** 4)))
add("integral.quartic.numeric_reach", fixed(200, 0))  # the film's midpoint sum; this oracle's own runs over the whole line
add("integral.quartic.numeric", fixed(numeric, 4))
check("pi/sqrt2 three ways", abs(value - PI / math.sqrt(2)) < 1e-9 and abs(numeric - PI / math.sqrt(2)) < 1e-9)

# ---------------------------------------------------------- 14 continuation
add("series.terms", len(range(11)))
add("series.half.at", str(Fraction(1, 2)))
add("series.half.n10", fixed(float(sum(Fraction(1, 2) ** k for k in range(11))), 6))
add("series.half.function", fixed(float(1 / (1 - Fraction(1, 2))), 0))
add("series.one_two.at", float(Fraction(6, 5)))
add("series.one_two.n10", fixed(float(sum(Fraction(6, 5) ** k for k in range(11))), 2))
add("series.one_two.function", fixed(float(1 / (1 - Fraction(6, 5))), 0))
add("continuation.first_disk.radius", fixed(abs(1 - (-0.5)), 1))
add("continuation.two", fixed(1 / (1 - 2), 0))
add("continuation.zeta_minus1", fixed(zeta(complex(-1.0)).real, 6))
check("zeta(-1) = -1/12", abs(zeta(complex(-1.0)) + 1 / 12) < 1e-12)
# the square root carried around: integrate d(log w) = dz/(2z) along the circle
steps = 4096
phase = sum(cmath.phase(cmath.exp(1j * TAU * (k + 1) / steps) / cmath.exp(1j * TAU * k / steps)) / 2 for k in range(steps))
end = cmath.exp(1j * phase)
add("continuation.sqrt.end", lst([end.real, end.imag], 0))
add("lacunary.terms", 12)
add("lacunary.limit", 0.98)

# ---------------------------------------------------------- 15 closing
r3 = abs(3 - 1j)
add("recentre3.centre", fixed(3, 0))
add("recentre3.radius_sq", fixed(3 ** 2 + 1 ** 2, 0))
add("recentre3.radius", fixed(r3, 3))
add("recentre3.ends", lst([3 - r3, 3 + r3], 2))

# cross-checks against the Rust-free literature values
check("gamma(5) = 4!", sp.gamma(5.0) == 24.0)
check("zeta(2) = pi^2/6 (Borwein)", abs(zeta(complex(2.0)) - PI ** 2 / 6) < 1e-13)
check("zeta(1/2)", abs(zeta(complex(0.5)) + 1.4603545088095868) < 1e-13)

for key, val in rows:
    print(f"{key}\t{val}")
print(f"# {len(rows)} facts, {len(failures)} failed cross-checks", file=sys.stderr)
sys.exit(1 if failures else 0)
