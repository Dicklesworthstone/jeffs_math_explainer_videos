"""Independent oracle for the logarithm rebuild candidate (mx-video-program-program-87do.63).

Recomputes every key that src/facts.rs prints, by different means where it
matters: exact fractions and integers for the ruler and slide rule, cmath for
the complex logarithm, and the closed-form 2x2 exponential (Putzer/Cayley-
Hamilton) instead of the Rust model's scaled Taylor series. Prints
key<TAB>value lines in the same order as facts.rs, for `./x facts-check`.

Run with `python3 -I oracle.py`. Standard library only.
"""
import cmath
import math
from fractions import Fraction

ROWS = []


def add(key, value):
    ROWS.append((key, str(value)))


def dec(x, places):
    """Fixed decimals, never '-0.000'."""
    s = f"{x:.{places}f}"
    if s.lstrip("-").strip("0.") == "":
        s = s.lstrip("-")
    return s


def num(x):
    """A short number: integers without a decimal point, else as written."""
    if float(x).is_integer():
        return str(int(x))
    return repr(float(x))


# ---------------------------------------------------------------- music
# Equal temperament, A4 = 440 Hz (ISO 16): f(n) = 440 * 2**(n/12) for n
# semitones above A4. The seven A's are A1..A7.
A4 = 440
freqs = [Fraction(A4) * Fraction(2) ** (k - 3) for k in range(7)]
add("music.freqs_hz", ",".join(num(f) for f in freqs))
top = freqs[-1]
add("music.ruler_top_hz", num(top))
add("music.first_four_share", str(freqs[3] / top))
add("music.last_step_share", str((freqs[6] - freqs[5]) / top))
add("music.keys_per_octave", 12)
add("music.start_hz", A4)
add("music.half_up_keys", 6)
half = A4 * 2 ** 0.5
add("music.half_octave_hz", dec(half, 2))
add("music.half_octave_spoken", round(half))
add("music.linear_mid_hz", num(Fraction(A4 + 2 * A4, 2)))
add("music.root_two", dec(2 ** 0.5, 3))
nineteen = 2 ** (19 / 12)
add("music.nineteen_keys_ratio", dec(nineteen, 3))
add("music.nineteen_keys_hz", dec(A4 * nineteen, 2))
add("music.three_times_hz", 3 * A4)

# ---------------------------------------------------------------- ruler
add("ruler.three_pow", 12)
add("ruler.two_pow", 19)
p3, p2 = 3 ** 12, 2 ** 19
add("ruler.three_pow12", p3)
add("ruler.two_pow19", p2)
add("ruler.power_gap_percent", dec(float((Fraction(p3, p2) - 1) * 100), 2))
add("ruler.nineteen_twelfths", dec(19 / 12, 3))
add("ruler.log2_3", dec(math.log2(3), 3))
add("ruler.log2_8", num(math.log2(8)))
add("ruler.log2_sqrt2", num(round(math.log2(math.sqrt(2)), 12)))
add("ruler.log2_half", num(math.log2(0.5)))
add("ruler.change_base_mid", 2 ** 2)
add("ruler.change_base_final", 10)

# ---------------------------------------------------------------- slide rule
a, b = Fraction(5, 2), Fraction(16, 5)
c, d = 4, 6
add("slide.a", num(a))
add("slide.b", num(b))
add("slide.ab", num(a * b))
add("slide.pos_a", dec(math.log10(a), 3))
add("slide.pos_b", dec(math.log10(b), 3))
add("slide.pos_ab", dec(math.log10(a * b), 3))
add("slide.c", c)
add("slide.d", d)
add("slide.cd", c * d)
digits = Fraction(c * d)
laps = 0
while digits >= 10:
    digits /= 10
    laps += 1
add("slide.cd_digits", num(digits))
add("slide.pos_c", dec(math.log10(c), 3))
add("slide.pos_d", dec(math.log10(d), 3))
add("slide.pos_cd", dec(math.log10(c * d), 3))
add("slide.pos_cd_folded", dec(math.log10(c * d) - laps, 3))
add("slide.cd_laps", laps)

# ---------------------------------------------------------------- dial
values = [Fraction(24, 100), Fraction(24, 10), Fraction(24), Fraction(240)]
add("dial.values", ",".join(num(x) for x in values))
add("dial.laps", ",".join(str(math.floor(math.log10(x))) for x in values))
add("dial.mark", dec(math.log10(2.4), 3))

# ---------------------------------------------------------------- real
add("real.e", dec(math.e, 3))
product_a, product_b = Fraction(2), Fraction(4)
add("real.product_a", num(product_a))
add("real.product_b", num(product_b))
add("real.product_ab", num(product_a * product_b))
add("real.growth_target", 10 ** 3)
add("real.ln_one_point_oh_one", dec(math.log(1.01), 5))
rate = 0.05
add("real.rate", num(rate))
add("real.ln2", dec(math.log(2), 3))
add("real.double_years", dec(math.log(2) / rate, 1))
add("real.ln_thousand", dec(math.log(1000), 2))
add("real.thousand_years", round(math.log(1000) / rate))
add("real.two_pow10", 2 ** 10)
add("real.ten_doublings_years", dec(10 * math.log(2) / rate, 1))
add("real.rule_of_seventy", dec(100 * math.log(2), 1))

# ---------------------------------------------------------------- complex
def polar(r, deg):
    return cmath.rect(r, math.radians(deg))


z, w = polar(1.2, 30), polar(1.5, 50)
zw = z * w
add("complex.mul_z", "1.2,30")
add("complex.mul_w", "1.5,50")
add("complex.mul_zw", f"{dec(abs(zw), 1)},{round(math.degrees(cmath.phase(zw)))}")
# the columns of logarithms of -1 and of 1, in units of pi
minus_one = [2 * k + 1 for k in range(-2, 2)]
assert all(abs(cmath.exp(1j * math.pi * m) + 1) < 1e-12 for m in minus_one)
add("complex.minus_one_logs_pi", ",".join(str(m) for m in minus_one))
one = [2 * k for k in range(-1, 3)]
assert all(abs(cmath.exp(1j * math.pi * m) - 1) < 1e-12 for m in one)
add("complex.one_logs_pi", ",".join(str(m) for m in one))
g = cmath.rect(2, 0.6 * math.pi)
add("complex.generic", "2,0.6")
add("complex.generic_ln_len", dec(math.log(abs(g)), 3))

# ---------------------------------------------------------------- loops
# Unwrap the angle of the unit circle walked once, sampled finely.
def lift(points):
    out = [cmath.phase(points[0])]
    for p, q in zip(points, points[1:]):
        out.append(out[-1] + cmath.phase(q / p))
    return out


n = 2000
unit = [cmath.exp(2j * math.pi * k / n) for k in range(n + 1)]
side = [1.7 + 0.8 * cmath.exp(2j * math.pi * k / n) for k in range(n + 1)]
add("loop.unit_winding", round((lift(unit)[-1] - lift(unit)[0]) / (2 * math.pi)))
add("loop.side_winding", round((lift(side)[-1] - lift(side)[0]) / (2 * math.pi)))
add("loop.side", "1.7,0.8")
add("loop.climb_pi", num(round(lift(unit)[n // 4] / math.pi, 9)) + "," + num(round(lift(unit)[n // 2] / math.pi, 9)) + "," + num(round(lift(unit)[n] / math.pi, 9)))

# ---------------------------------------------------------------- product rule
q = Fraction(3, 4)
s = q + q
principal = s - 2 if s > 1 else s
add("product.angle_pi", str(q))
add("product.sum_pi", str(s))
add("product.principal_pi", str(principal))
add("product.lap_pi", str(s - principal))
zz = cmath.exp(1j * math.pi * float(q))
assert abs(cmath.log(zz * zz).imag / math.pi - float(principal)) < 1e-12
# the square-root puzzle with z^(1/2) = exp(Log(z)/2), Arg(-1) = pi
root = lambda u: cmath.exp(cmath.log(u) / 2)
add("sqrt.product_of_roots", dec((root(-1) * root(-1)).real, 0))
add("sqrt.root_of_product", dec(root((-1) * (-1)).real, 0))

# ---------------------------------------------------------------- matrices
def mexp(A):
    """Closed form for 2x2: exp(A) = e^m [cosh(D) I + sinh(D)/D (A - m I)]."""
    (p, q_), (r, t) = A
    m = (p + t) / 2
    det = p * t - q_ * r
    D = cmath.sqrt(m * m - det)
    ch = cmath.cosh(D)
    sh = cmath.sinh(D) / D if abs(D) > 1e-14 else 1.0
    e = cmath.exp(m)
    return [[e * (ch + sh * (p - m)), e * sh * q_], [e * sh * r, e * (ch + sh * (t - m))]]


def close(A, B, tol=1e-9):
    return all(abs(A[i][j] - B[i][j]) < tol for i in range(2) for j in range(2))


J = [[0, -1], [1, 0]]
quarter = [[0, -1], [1, 0]]
speeds = [Fraction(1, 4), Fraction(5, 4), Fraction(-3, 4)]
for sp in speeds:
    th = 2 * math.pi * float(sp)
    assert close(mexp([[0, -th], [th, 0]]), quarter)
add("matrix.quarter_turn_speeds", ",".join(num(float(sp)) for sp in speeds))
I = [[1, 0], [0, 1]]


def ellipse_generator(a_, b_, tilt_deg):
    # X = 2 pi P J P^-1 with P = R(tilt) diag(a, b)
    ct, st = math.cos(math.radians(tilt_deg)), math.sin(math.radians(tilt_deg))
    P = [[a_ * ct, -b_ * st], [a_ * st, b_ * ct]]
    detP = P[0][0] * P[1][1] - P[0][1] * P[1][0]
    Pi = [[P[1][1] / detP, -P[0][1] / detP], [-P[1][0] / detP, P[0][0] / detP]]
    mul = lambda A, B: [[sum(A[i][k] * B[k][j] for k in range(2)) for j in range(2)] for i in range(2)]
    M = mul(mul(P, J), Pi)
    return [[2 * math.pi * M[i][j] for j in range(2)] for i in range(2)]


ellipses = [(2.0, 1.0, 0.0), (1.6, 0.7, 30.0)]
assert all(close(mexp(ellipse_generator(*e)), I) for e in ellipses)
add("matrix.ellipses", ";".join(f"{num(e[0])},{num(e[1])},{num(e[2])}" for e in ellipses))
add("matrix.ellipses_return", "yes")
X = [[0.3, -1.0], [0.8, -0.1]]
add("matrix.flow", "0.3,-1;0.8,-0.1")
add("matrix.flow_trace", num(round(X[0][0] + X[1][1], 12)))


def mdet(A):
    return A[0][0] * A[1][1] - A[0][1] * A[1][0]


E1 = mexp(X)
E3 = mexp([[3 * x for x in row] for row in X])
add("matrix.flow_det_t1", dec(mdet(E1).real, 3))
add("matrix.flow_det_t3", dec(mdet(E3).real, 3))
assert abs(mdet(E3) - math.exp(0.6)) < 1e-9
add("matrix.mirror_det", "-1")
flip = [[-2, 0], [0, -0.5]]
add("matrix.flip_pair", "-2,-0.5")
add("matrix.flip_pair_det", num(mdet(flip)))
L = [[math.log(2) + 1j * math.pi, 0], [0, -math.log(2) + 1j * math.pi]]
assert close(mexp(L), flip)
add("matrix.flip_pair_log_real", f"{dec(math.log(2), 3)},{dec(-math.log(2), 3)}")
assert close(mexp([[0, -math.pi], [math.pi, 0]]), [[-1, 0], [0, -1]])
add("matrix.half_turn_speed", "0.5")
tr_half = (X[0][0] + X[1][1]) / 2
eig = tr_half + cmath.sqrt(tr_half * tr_half - mdet(X))
add("matrix.flow_turning", dec(abs(eig.imag), 3))
add("matrix.flow_turning_pi", dec(abs(eig.imag) / math.pi, 2))
add("matrix.flip_pair_gap", num(-0.5 - (-2)))
# 11_1: the average of I and the mirror is diag(0, 1), which is singular
add("matrix.mirror_average_det", num(mdet([[0.0, 0.0], [0.0, 1.0]])))
# 11_4/11_5: route one is X(Au), route two A(Xu); computed as two matrix-vector
# products each, not through the commutator
add("matrix.probe", "1,1")
add("matrix.probe_candidate", "0.4,0.6;-0.5,-0.3")
Xc = [[0.4, 0.6], [-0.5, -0.3]]


def act(M, w):
    return [M[0][0] * w[0] + M[0][1] * w[1], M[1][0] * w[0] + M[1][1] * w[1]]


r1 = act(Xc, act(flip, [1, 1]))
r2 = act(flip, act(Xc, [1, 1]))
add("matrix.probe_gap", dec(math.hypot(r1[0] - r2[0], r1[1] - r2[1]), 2))

# 12_4-12_6: spins about the vertical rod, in laps per unit time; each must
# end at time 1 in the half turn diag(-1, 1, -1) (rotation matrix about y)
rates = [0.5, 1.5, 2.5, -0.5]
for r in rates:
    a = 2 * math.pi * r
    Ry = [[math.cos(a), 0, math.sin(a)], [0, 1, 0], [-math.sin(a), 0, math.cos(a)]]
    half = [[-1, 0, 0], [0, 1, 0], [0, 0, -1]]
    assert all(abs(Ry[i][j] - half[i][j]) < 1e-12 for i in range(3) for j in range(3)), r
add("puzzle.spin_rates", ",".join(num(r) for r in rates))

# 05_1-05_2: i times i, and e^(ln 2 + i pi/2) = 2i
add("complex.i_times_i", num((1j * 1j).real))
wk = cmath.exp(math.log(2) + 1j * math.pi / 2)
assert abs(wk - 2j) < 1e-12
add("complex.worked", f"{num(round(abs(wk)))},{num(round(math.degrees(cmath.phase(wk))))}")


# 06_6: principal angle (cmath.phase, in (-pi, pi]) minus the carried angle, in laps
def pgap(t):
    th = 2 * math.pi * t
    return round((cmath.phase(cmath.exp(1j * th)) - th) / (2 * math.pi))


add("loop.principal_gap", f"{num(pgap(0.49))},{num(pgap(0.51))}")
# 07_2: Log i and Log(-i), imaginary parts in units of pi
add("branch.log_i_pi", f"{num(cmath.log(1j).imag / math.pi)},{num(cmath.log(-1j).imag / math.pi)}")

for key, value in ROWS:
    print(f"{key}\t{value}")
