#!/usr/bin/env python3
# oracle-packages: scipy
"""Facts oracle for Video J, "What Does It Mean to Raise e to a Matrix?".

Run from this folder (`./x matrix_exponential facts-check` does, with numpy
and scipy through uv):

    uv run --no-project --with numpy --with scipy python3 -I -B oracle.py

stdout: one `KEY<TAB>VALUE` line per fact, the same keys `expm facts` prints
(src/facts.rs), in the rows of docs/fact-sheet.md: F-N (the number
exponential), F-H (the hook), F-J (the quarter turn), F-S (does the series
settle), F-T (two tanks), F-G (walks on a network), F-D (spreading), F-C
(computing it), F-O (order matters), F-L (the doorway to Lie theory), F-X
(the edges of the always sentences), F-M (the machine and the rules drawn as
fields), F-B (cited), F-Q (the quizzes) and F-V (the numbers as the
narration says them and the screen shows them).

stderr: cross-checks (lines starting with "#") and a summary; exit 1 if any
cross-check failed.

How the values are obtained, independently of the Rust model (which computes
every exponential by scaling and squaring in 64-bit floats):
  * every exponential of a matrix with rational entries is the exact
    rational power series (Python Fractions), summed until the remainder is
    provably below 1e-30;
  * exponentials of matrices that carry pi come from closed forms (cosine
    and sine; Rodrigues' formula for a spin);
  * every one of those is then cross-checked against SciPy's `expm` (Pade
    with scaling and squaring: Al-Mohy and Higham 2009), a third route;
  * sizes (operator 2-norms) by NumPy's singular values;
  * walks by listing every sequence of nodes and keeping the ones that
    follow links;
  * the 32-bit runs repeated operation for operation in NumPy float32;
  * the edge cases drawn from the same PCG64DXSM stream and judged with
    SciPy's `expm` and NumPy's norms; the named degenerate cases (zero, a
    nilpotent block, a defective shear, a large non-normal matrix, …)
    judged in exact fractions;
  * heat on the 400-node grid by exponentiating its whole 400 by 400
    Laplacian with SciPy (the video multiplies two 20 by 20 exponentials);
  * the loops (the square's and the cube's) multiplied in the order they
    are played, the cube's from Rodrigues' formula;
  * CITED literals typed from the sources listed in docs/fact-sheet.md.
"""
import itertools
import math
import sys
from decimal import Decimal, getcontext
from fractions import Fraction as F

import numpy as np
from scipy.linalg import expm as scipy_expm

# compounding a million times is done in 60-digit decimals: the exact
# fraction (1 + 1/n)^n has six million digits
getcontext().prec = 60

_EMITTED = {}
_FAILED = []
_CHECKS = [0]


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


def dec(x, places):
    """x to `places` decimals by exact rational rounding; a rounded zero has
    no sign; a value within 1e-6 of a rounding tie is refused (src/facts.rs
    refuses the same ones)."""
    x = F(x)
    scaled = abs(x) * 10**places
    frac = scaled - math.floor(scaled)
    if abs(frac - F(1, 2)) < F(1, 10**6):
        raise SystemExit(f"oracle: {float(x)} is a rounding tie at {places} places")
    n = math.floor(scaled + F(1, 2))
    digits = str(n).rjust(places + 1, "0")
    body = digits[:-places] + "." + digits[-places:] if places else digits
    return ("-" if x < 0 and n != 0 else "") + body


def row(v, places):
    return ",".join(dec(x, places) for x in v)


def mat(m, places):
    return ";".join(row(r, places) for r in m)


def whole(m):
    out = []
    for r in m:
        for x in r:
            if F(x).denominator != 1 and abs(F(x) - round(F(x))) > F(1, 10**9):
                raise SystemExit(f"oracle: {x} is not a whole number")
        out.append(",".join(str(int(round(F(x)))) for x in r))
    return ";".join(out)


def join(items):
    return ",".join(str(x) for x in items)


def tkey(t):
    return str(int(t)) if float(t).is_integer() else str(t)


# ---------------------------------------------------------------- exact matrices


def fmat(rows):
    return [[F(x) for x in r] for r in rows]


def ident(n):
    return [[F(int(i == j)) for j in range(n)] for i in range(n)]


def mmul(a, b):
    n = len(a)
    return [[sum(a[i][k] * b[k][j] for k in range(n)) for j in range(n)] for i in range(n)]


def madd(a, b):
    return [[x + y for x, y in zip(r, s)] for r, s in zip(a, b)]


def msub(a, b):
    return [[x - y for x, y in zip(r, s)] for r, s in zip(a, b)]


def mscale(a, c):
    return [[x * c for x in r] for r in a]


def mpow(a, k):
    out = ident(len(a))
    for _ in range(k):
        out = mmul(out, a)
    return out


def transpose(a):
    return [list(c) for c in zip(*a)]


def apply(a, v):
    return [sum(x * y for x, y in zip(r, v)) for r in a]


def column(a, j):
    return [r[j] for r in a]


def commutator(a, b):
    return msub(mmul(a, b), mmul(b, a))


def norm_inf(a):
    return max(sum(abs(x) for x in r) for r in a)


def as_np(a):
    return np.array([[float(x) for x in r] for r in a], dtype=np.float64)


def size(a):
    """The operator 2-norm: the largest singular value."""
    return float(np.linalg.svd(as_np(a), compute_uv=False)[0])


_EXPM_CHECKED = [0]


def expm_exact(a, tail=F(1, 10**30)):
    """e^A for rational A: the exact series, stopped when the remainder
    (bounded by a geometric series in the infinity norm) is below `tail`;
    then cross-checked against SciPy's expm."""
    a = fmat(a)
    n = len(a)
    norm = norm_inf(a)
    term = ident(n)
    total = ident(n)
    bound = F(1)
    k = 0
    while True:
        k += 1
        term = mscale(mmul(term, a), F(1, k))
        total = madd(total, term)
        bound = bound * norm / k
        # remainder after term k: at most bound * q / (1 - q), q = norm/(k+1)
        q = norm / (k + 1)
        if q < 1 and bound * q / (1 - q) < tail:
            break
    theirs = scipy_expm(as_np(a))
    ours = as_np(total)
    scale = max(1.0, float(np.abs(ours).max()), float(np.abs(theirs).max()))
    # SciPy's own accuracy on the non-normal matrices here is about 1e-12
    if not float(np.abs(ours - theirs).max()) <= 1e-9 * scale:
        check(f"scipy.expm agrees with the exact series for {as_np(a).tolist()}", False)
    _EXPM_CHECKED[0] += 1
    return total


def expm_t(a, t):
    return expm_exact(mscale(fmat(a), F(t)))


def series(a, kmax):
    """(k, term, running total) for k = 0..kmax, exactly."""
    a = fmat(a)
    term = ident(len(a))
    total = ident(len(a))
    out = [(0, term, total)]
    for k in range(1, kmax + 1):
        term = mscale(mmul(term, a), F(1, k))
        total = madd(total, term)
        out.append((k, term, total))
    return out


def tiny_steps(a, t, n):
    a = fmat(a)
    return mpow(madd(ident(len(a)), mscale(a, F(t) / n)), n)


def exp_fraction(x, digits=40):
    """e^x for rational x as a Fraction good to `digits` places."""
    return expm_exact([[F(x)]], tail=F(1, 10**digits))[0][0]


J = fmat([[0, -1], [1, 0]])
E_EXACT = exp_fraction(1)
PI = math.pi
TAU = 2 * math.pi

# ---------------------------------------------------------------- cited literals

MVL = [[-49, 24], [-64, 31]]
MVL_CORRECT = "-0.735759,0.551819;-1.471518,1.103638"
MVL_IBM_RESULT = "-22.25880,-1.432766;-61.49931,-3.474280"
MVL_EIGENVALUES = "-17,-1"
EULER_E = "2.71828182845904523536028"


# ---------------------------------------------------------------- F-N


def compound(x, n):
    """(1 + x/n)^n to 60 significant digits."""
    x = Decimal(x.numerator) / Decimal(x.denominator)
    return F((Decimal(1) + x / Decimal(n)) ** n)


def number_facts():
    emit("F-N.e", dec(E_EXACT, 6))
    emit("F-N.e.digits15", dec(E_EXACT, 15))
    check("e to 15 places is the start of Euler's printed digits", dec(E_EXACT, 15) == EULER_E[:17])
    # Euler printed 23 decimals without rounding the last one up
    check("e cut off after 23 places is Euler's printed number",
          str(math.floor(E_EXACT * 10**23)) == EULER_E.replace(".", ""))
    for n in [1, 2, 4, 12, 365, 1_000_000]:
        emit(f"F-N.compound.n{n}", dec(compound(F(1), n), 5))
    check("the 60-digit compounding is the exact fraction for n = 365",
          abs(compound(F(1), 365) - (1 + F(1, 365)) ** 365) < F(1, 10**50))
    total = F(0)
    for k in range(0, 11):
        total += F(1, math.factorial(k))
        emit(f"F-N.series.k{k}", dec(total, 6))
    emit("F-N.factorials", join(math.factorial(k) for k in range(7)))
    emit("F-N.growth.rate", "0.05")
    emit("F-N.growth.years", "20")
    emit("F-N.growth.factor", dec(exp_fraction(F(5, 100) * 20), 6))
    emit("F-N.growth.doubling_years", dec(math.log(2) / 0.05, 4))
    e2, e3, e5 = exp_fraction(2), exp_fraction(3), exp_fraction(5)
    emit("F-N.add.e2", dec(e2, 6))
    emit("F-N.add.e3", dec(e3, 6))
    emit("F-N.add.product", dec(e2 * e3, 6))
    emit("F-N.add.e5", dec(e5, 6))
    check("e^2 e^3 = e^5 to 30 places", abs(e2 * e3 - e5) < F(1, 10**30))
    emit("F-N.turn.t1", row([math.cos(1), math.sin(1)], 6))
    emit("F-N.turn.pi", row([math.cos(PI), math.sin(PI)], 6))
    for n in [4, 16, 100, 1000]:
        z = (1 + 1j * PI / n) ** n
        emit(f"F-N.turn.steps.n{n}", row([z.real, z.imag], 4))
    emit("F-N.turn.radian_degrees", dec(180 / PI, 4))
    # the three definitions are one function: the limit and the series agree with exp
    for x in [F(-2), F(1, 2), F(3)]:
        limit = compound(x, 10**7)
        check(f"(1 + x/n)^n at n = 10^7 is within 1e-5 of e^x, x = {x}", abs(limit - exp_fraction(x)) < F(1, 10**5) * exp_fraction(x))


# ---------------------------------------------------------------- F-H


def hook_facts():
    guess_zero = [[math.exp(0), math.exp(0)], [math.exp(0), math.exp(0)]]
    emit("F-H.zero.guess", whole(guess_zero))
    emit("F-H.zero.guess.det", dec(guess_zero[0][0] * guess_zero[1][1] - guess_zero[0][1] * guess_zero[1][0], 6))
    emit("F-H.zero.true", whole(expm_exact([[0, 0], [0, 0]])))
    guess = [[math.exp(float(x)) for x in r] for r in J]
    emit("F-H.J.guess", mat(guess, 6))
    true = [[math.cos(1), -math.sin(1)], [math.sin(1), math.cos(1)]]
    emit("F-H.J.true", mat(true, 6))
    check("the exact series for e^J is the rotation by one radian",
          np.abs(as_np(expm_exact(J)) - np.array(true)).max() < 1e-15)
    g = np.array(guess)
    emit("F-H.twice.guess", mat((g @ g).tolist(), 6))
    emit("F-H.twice.guess.doubled", mat([[math.exp(2 * float(x)) for x in r] for r in J], 6))
    e1 = expm_exact(J)
    emit("F-H.twice.true", max(abs(x) for r in msub(mmul(e1, e1), expm_t(J, 2)) for x in r) < F(1, 10**25))


# ---------------------------------------------------------------- F-J


def quarter_facts():
    for k in range(1, 5):
        emit(f"F-J.power.k{k}", whole(mpow(J, k)))
    emit("F-J.period", next(k for k in range(1, 9) if mpow(J, k) == ident(2)))

    def rot(t):
        return [[math.cos(t), -math.sin(t)], [math.sin(t), math.cos(t)]]

    emit("F-J.exp.quarter", mat(rot(PI / 2), 6))
    emit("F-J.exp.half", mat(rot(PI), 6))
    emit("F-J.exp.full", mat(rot(TAU), 6))
    for t in [PI / 2, PI, TAU]:
        check(f"scipy.expm(tJ) is the rotation by t = {t:.4f}",
              np.abs(scipy_expm(t * as_np(J)) - np.array(rot(t))).max() < 1e-13)
    even = sum(F((-1) ** m, math.factorial(2 * m)) for m in range(0, 16))
    odd = sum(F((-1) ** m, math.factorial(2 * m + 1)) for m in range(0, 15))
    emit("F-J.split.even", dec(even, 6))
    emit("F-J.split.odd", dec(odd, 6))
    check("the even terms sum to cos 1 and the odd terms to sin 1",
          abs(float(even) - math.cos(1)) < 1e-15 and abs(float(odd) - math.sin(1)) < 1e-15)


# ---------------------------------------------------------------- F-S


def settle_facts():
    t = 5
    a = mscale(J, F(t))
    steps = series(a, 20)
    sizes = [F(t**k, math.factorial(k)) for k in range(21)]
    for k, term, _ in steps:
        check(f"the size of term {k} of e^(5J) is 5^{k}/{k}!", abs(size(term) - float(sizes[k])) < 1e-12 * max(1.0, float(sizes[k])))
    emit("F-S.t", "5")
    emit("F-S.size", dec(size(a), 6))
    emit("F-S.terms", row(sizes, 5))
    peak = max(sizes)
    emit("F-S.peak", dec(peak, 5))
    emit("F-S.peak.k", join(k for k, s in enumerate(sizes) if s == peak))
    for k, _, total in steps:
        emit(f"F-S.total.k{k}", row(column(total, 0), 4))

    def radius2(step):
        return sum(x * x for x in column(step[2], 0))

    far = max(steps, key=radius2)
    emit("F-S.farthest.k", far[0])
    emit("F-S.farthest.radius", dec(math.sqrt(radius2(far)), 4))
    emit("F-S.limit", row([math.cos(5), math.sin(5)], 6))
    check("the exact series for e^(5J) lands on (cos 5, sin 5)",
          np.abs(as_np(expm_exact(a))[:, 0] - np.array([math.cos(5), math.sin(5)])).max() < 1e-15)
    emit("F-S.bound", dec(exp_fraction(5), 6))
    check("the sizes of all the terms add up to e^5 (so the series sits under a number series that settles)",
          abs(sum(F(t**k, math.factorial(k)) for k in range(200)) - exp_fraction(5)) < F(1, 10**30))
    emit("F-S.first_below_thousandth.k", next(k for k, s in enumerate(sizes) if s < F(1, 1000)))
    emit("F-S.term.k20", dec(sizes[20], 7))
    shear = fmat([[1, 1], [0, 1]])
    emit("F-S.shear.size", dec(size(shear), 6))
    emit("F-S.shear.squared.size", dec(size(mpow(shear, 2)), 6))
    emit("F-S.shear.size.squared", dec(size(shear) ** 2, 6))
    check("the shear's size is the golden ratio, and its square's is 1 + root 2",
          abs(size(shear) - (1 + math.sqrt(5)) / 2) < 1e-14 and abs(size(mpow(shear, 2)) - (1 + math.sqrt(2))) < 1e-14)
    nil = fmat([[0, 1], [0, 0]])
    emit("F-S.nil.size", dec(size(nil), 6))
    emit("F-S.nil.squared", whole(mpow(nil, 2)))
    strict = fmat([[3, 3], [0, 3]])
    sz = size(strict)
    steps = series(strict, 12)
    terms = [size(term) for _, term, _ in steps]
    ceilings = [sz**k / math.factorial(k) for k in range(13)]
    emit("F-S.strict.matrix", whole(strict))
    emit("F-S.strict.size", dec(sz, 3))
    emit("F-S.strict.terms", row(terms, 3))
    emit("F-S.strict.ceilings", row(ceilings, 3))
    emit("F-S.strict.bound", dec(math.exp(sz), 1))
    emit("F-S.strict.below_from_k2", all(t < c * 0.999 for t, c in list(zip(terms, ceilings))[2:]))
    # three times the shear (1, k; 0, 1): its k-th power has a closed-form size
    for k in range(13):
        closed = 3**k * math.sqrt((k * k + 2 + k * math.sqrt(k * k + 4)) / 2) / math.factorial(k)
        check(f"term {k} of the strict matrix has the closed-form size", abs(terms[k] - closed) < 1e-9 * max(1.0, closed))


# ---------------------------------------------------------------- F-T

TANKS = fmat([[F(-1, 10), F(1, 10)], [F(1, 10), F(-1, 10)]])
START = [F(100), F(0)]


def tank_closed(t):
    """Grams in each tank at minute t, from the eigenvalues 0 and -0.2:
    50 (1 + e^{-0.2 t}) and 50 (1 - e^{-0.2 t})."""
    decay = math.exp(-0.2 * t)
    return [50 * (1 + decay), 50 * (1 - decay)]


def tank_facts():
    emit("F-T.rule", mat(TANKS, 1))
    emit("F-T.start", row(START, 0))
    for t in [1, 5, 10, 20, 60]:
        exact = apply(expm_t(TANKS, t), START)
        emit(f"F-T.at.t{tkey(t)}", row(exact, 4))
        check(f"the exact series agrees with the closed form at minute {t}",
              max(abs(float(x) - y) for x, y in zip(exact, tank_closed(t))) < 1e-12)
    for n in [1, 2, 5, 10, 100, 1000]:
        stepped = apply(tiny_steps(TANKS, 10, n), START)
        emit(f"F-T.steps.n{n}", row(stepped, 4))
        # each tiny step is 50 (1 + (1 - 2/n)^n) in the first tank
        check(f"{n} tiny steps: the first tank holds 50 (1 + (1 - 2/{n})^{n})",
              stepped[0] == 50 * (1 + (1 - F(2, n)) ** n))
    emit("F-T.total", dec(sum(apply(expm_t(TANKS, 7), START)), 6))
    emit("F-T.settles", row(tank_closed(1000), 4))
    eig = sorted(np.linalg.eigvalsh(as_np(TANKS)).tolist())
    emit("F-T.eigenvalues", row(eig, 6))
    emit("F-T.within_a_gram.minutes", dec(math.log(50) / 0.2, 4))
    exact = F(tank_closed(10)[0])
    emit("F-T.off_by.n10", dec(exact - apply(tiny_steps(TANKS, 10, 10), START)[0], 2))
    emit("F-T.off_by.n1000", dec(exact - apply(tiny_steps(TANKS, 10, 1000), START)[0], 3))
    check("ten times the steps buys one more digit: the error is about 13.5/n grams",
          all(abs(float(exact - apply(tiny_steps(TANKS, 10, n), START)[0]) * n / 13.53 - 1) < 0.1 for n in (10, 100, 1000)))
    check("at that minute the first tank holds 51 grams", abs(tank_closed(math.log(50) / 0.2)[0] - 51) < 1e-12)


# ---------------------------------------------------------------- F-G

NODES = 4
LINKS = [(0, 1), (0, 2), (1, 2), (2, 3)]
LINKED = {(i, j) for i, j in LINKS} | {(j, i) for i, j in LINKS}
ADJ = fmat([[int((i, j) in LINKED) for j in range(NODES)] for i in range(NODES)])


def count_walks(k):
    """Walks of exactly k links, by listing every sequence of k + 1 nodes."""
    count = [[0] * NODES for _ in range(NODES)]
    for seq in itertools.product(range(NODES), repeat=k + 1):
        if all((seq[i], seq[i + 1]) in LINKED for i in range(k)):
            count[seq[0]][seq[-1]] += 1
    return count


def network_facts():
    emit("F-G.nodes", NODES)
    emit("F-G.links", join(f"{i + 1}-{j + 1}" for i, j in LINKS))
    emit("F-G.degrees", join(sum(1 for link in LINKS if i in link) for i in range(NODES)))
    walks = {k: count_walks(k) for k in range(0, 9)}
    for k in range(1, 5):
        emit(f"F-G.walks.k{k}", ";".join(join(r) for r in walks[k]))
    for k in range(0, 9):
        check(f"listed walks of length {k} are the entries of A^{k}", fmat(walks[k]) == mpow(ADJ, k))
    e = expm_exact(ADJ)
    emit("F-G.exp", mat(e, 4))
    emit("F-G.exp.trace", dec(sum(e[i][i] for i in range(NODES)), 4))
    running, partial = [], F(0)
    for k in range(0, 9):
        partial += F(walks[k][0][3], math.factorial(k))
        running.append(partial)
    emit("F-G.exp.one_to_four.running", row(running, 4))
    emit("F-G.walks.one_to_four", join(walks[k][0][3] for k in range(0, 9)))
    # Estrada and Hatano's spectral form: sum over eigenpairs of phi(p) phi(q) e^lambda
    lam, phi = np.linalg.eigh(as_np(ADJ))
    spectral = (phi * np.exp(lam)) @ phi.T
    check("e^A equals the spectral form of the communicability (Estrada and Hatano, eq. 3)",
          np.abs(spectral - as_np(e)).max() < 1e-12)


# ---------------------------------------------------------------- F-D


def laplacian(n, links):
    lap = [[F(0)] * n for _ in range(n)]
    for i, j in links:
        lap[i][j] -= 1
        lap[j][i] -= 1
        lap[i][i] += 1
        lap[j][j] += 1
    return lap


def quantum_pair(t):
    """A two-state system with Hamiltonian the pair's Laplacian, started at
    the first node: the amplitudes are (1 + e^{-2it})/2 and (1 - e^{-2it})/2,
    from the eigenvalues 0 and 2."""
    z = complex(math.cos(2 * t), -math.sin(2 * t))
    return [abs((1 + z) / 2) ** 2, abs((1 - z) / 2) ** 2]


def big_facts():
    """The 20 by 20 grid network: its whole Laplacian exponentiated by SciPy,
    and corner-to-corner walks counted over its list of neighbours."""
    side, source, t, steps = 20, (9, 9), 2.0, 40
    node = lambda r, c: r * side + c
    links = [(node(r, c), node(r, c + 1)) for r in range(side) for c in range(side - 1)]
    links += [(node(r, c), node(r + 1, c)) for r in range(side - 1) for c in range(side)]
    lap = np.zeros((side * side, side * side))
    for a, b in links:
        lap[a, b] -= 1
        lap[b, a] -= 1
        lap[a, a] += 1
        lap[b, b] += 1
    heat = scipy_expm(-t * lap)[:, node(*source)]
    emit("F-D.big.nodes", side * side)
    emit("F-D.big.links", len(links))
    emit("F-D.big.heat.source", dec(float(heat[node(*source)]), 4))
    emit("F-D.big.heat.neighbour", dec(float(heat[node(source[0], source[1] + 1)]), 4))
    emit("F-D.big.heat.total", dec(float(heat.sum()), 6))
    emit("F-D.big.heat.above_a_thousandth", int((heat > 0.001).sum()))
    neighbours = {n: [] for n in range(side * side)}
    for a, b in links:
        neighbours[a].append(b)
        neighbours[b].append(a)
    def corner_walks(length):
        count = {0: 1}
        for _ in range(length):
            nxt = {}
            for n, ways in count.items():
                for m in neighbours[n]:
                    nxt[m] = nxt.get(m, 0) + ways
            count = nxt
        return count.get(side * side - 1, 0)

    emit("F-D.big.walks", corner_walks(steps))
    check("no walk of 39 steps joins opposite corners, and the 38-step ones are the C(38, 19) shortest routes",
          corner_walks(39) == 0 and corner_walks(38) == math.comb(38, 19))
    # the video's route: the grid's exponential is the product of two path exponentials
    path = np.zeros((side, side))
    for i in range(side - 1):
        path[i, i + 1] -= 1
        path[i + 1, i] -= 1
        path[i, i] += 1
        path[i + 1, i + 1] += 1
    h = scipy_expm(-t * path)
    check("the whole grid's exponential is the product of two path exponentials",
          np.abs(np.kron(h, h)[:, node(*source)] - heat).max() < 1e-12)


def spreading_facts():
    lap = laplacian(NODES, LINKS)
    emit("F-D.L", whole(lap))
    neg = mscale(lap, F(-1))
    for t in ["0.25", "0.5", "1", "2", "5"]:
        heat = column(expm_t(neg, F(t)), 3)
        emit(f"F-D.heat.t{t}", row(heat, 4))
        check(f"heat at t = {t}: totals one, nothing negative", sum(heat) > 1 - F(1, 10**25) and min(heat) > 0)
    emit("F-D.heat.settles", row([F(1, 4)] * 4, 4))
    lam = np.linalg.eigvalsh(as_np(lap))
    check("the kite's Laplacian has one zero eigenvalue and the rest positive, so heat settles at the average",
          abs(lam[0]) < 1e-12 and lam[1] > 0.5)
    check("after time 200 the slowest mode is below 1e-40", math.exp(-200 * lam[1]) < 1e-40)
    emit("F-D.chance.rows_sum_to_zero", all(sum(r) == 0 for r in neg))
    emit("F-D.chance.from_hub.t1", row(expm_t(neg, 1)[2], 4))
    emit("F-D.chance.from_hub.settles", row([F(1, 4)] * 4, 4))
    pair = laplacian(2, [(0, 1)])
    emit("F-D.pair.L", whole(pair))
    emit("F-D.pair.is_the_tanks", mscale(pair, F(-1, 10)) == TANKS)
    for t in ["0.5", "1", "2"]:
        heat = column(expm_t(mscale(pair, F(-1)), F(t)), 0)
        emit(f"F-D.pair.heat.t{t}", row(heat, 6))
        check(f"heat on the pair at t = {t} is (1 + e^(-2t))/2",
              abs(float(heat[0]) - (1 + math.exp(-2 * float(t))) / 2) < 1e-15)
        emit(f"F-D.pair.quantum.t{t}", row(quantum_pair(float(t)), 6))
    emit("F-D.pair.quantum.quarter", row(quantum_pair(PI / 4), 6))
    emit("F-D.pair.quantum.half", row(quantum_pair(PI / 2), 6))
    emit("F-D.pair.quantum.full", row(quantum_pair(PI), 6))
    # a hopper that takes one hop per tick: the long-run chances are the
    # left eigenvector of its transition matrix
    adj = as_np(ADJ)
    step = adj / adj.sum(axis=1, keepdims=True)
    vals, vecs = np.linalg.eig(step.T)
    still = np.real(vecs[:, np.argmin(np.abs(vals - 1))])
    still = still / still.sum()
    emit("F-D.chance.per_tick", row(still.tolist(), 3))
    check("the per-tick hopper's long-run chance at the busy node is 3/8", abs(still[2] - 3 / 8) < 1e-12)
    big_facts()
    for t in [0.5, 1.0, 2.0, PI / 4]:
        u = scipy_expm(-1j * t * as_np(pair))
        theirs = (np.abs(u[:, 0]) ** 2).tolist()
        check(f"scipy.expm(-itL) gives the same chances at t = {t:.4f}, and they are cos^2 t and sin^2 t",
              max(abs(a - b) for a, b in zip(theirs, quantum_pair(t))) < 1e-13
              and abs(theirs[0] - math.cos(t) ** 2) < 1e-13)


# ---------------------------------------------------------------- F-C


def f32_mul(a, b):
    return [[np.float32(a[i][0] * b[0][j] + a[i][1] * b[1][j]) for j in range(2)] for i in range(2)]


def series_f32(a):
    """The series in 32-bit arithmetic, every operation rounded to float32,
    in the order src/model.rs performs them."""
    a = [[np.float32(x) for x in r] for r in a]
    term = [[np.float32(1), np.float32(0)], [np.float32(0), np.float32(1)]]
    total = [r[:] for r in term]
    k = 0
    while True:
        k += 1
        product = f32_mul(term, a)
        term = [[product[i][j] / np.float32(k) for j in range(2)] for i in range(2)]
        nxt = [[total[i][j] + term[i][j] for j in range(2)] for i in range(2)]
        if nxt == total:
            return total, k
        total = nxt


def series_f64(a):
    """The same loop in 64-bit floats (Python's own), same order."""
    a = [[float(x) for x in r] for r in a]
    term = [[1.0, 0.0], [0.0, 1.0]]
    total = [r[:] for r in term]
    k = 0
    while True:
        k += 1
        product = [[term[i][0] * a[0][j] + term[i][1] * a[1][j] for j in range(2)] for i in range(2)]
        term = [[product[i][j] / float(k) for j in range(2)] for i in range(2)]
        nxt = [[total[i][j] + term[i][j] for j in range(2)] for i in range(2)]
        if nxt == total:
            return total, k
        total = nxt


def scale_square_f32(a, halvings, degree):
    scale = np.float32(0.5**halvings)
    x = [[np.float32(v) * scale for v in r] for r in a]
    term = [[np.float32(1), np.float32(0)], [np.float32(0), np.float32(1)]]
    total = [r[:] for r in term]
    for k in range(1, degree + 1):
        product = f32_mul(term, x)
        term = [[product[i][j] / np.float32(k) for j in range(2)] for i in range(2)]
        total = [[total[i][j] + term[i][j] for j in range(2)] for i in range(2)]
    for _ in range(halvings):
        total = f32_mul(total, total)
    return total


def pade(m, x):
    """The [m/m] Pade approximant of e^x: numerator sum c_j x^j, denominator
    sum c_j (-x)^j, c_j = (2m - j)! m! / ((2m)! j! (m - j)!)."""
    c = [F(math.factorial(2 * m - j) * math.factorial(m), math.factorial(2 * m) * math.factorial(j) * math.factorial(m - j))
         for j in range(m + 1)]
    return sum(c[j] * x**j for j in range(m + 1)) / sum(c[j] * (-x) ** j for j in range(m + 1))


def computing_facts():
    a = fmat(MVL)
    emit("F-C.mvl.A", whole(a))
    eig = sorted(np.linalg.eigvals(as_np(a)).real.tolist())
    emit("F-C.mvl.eigenvalues", ",".join(dec(x, 0) for x in eig))
    check("the eigenvalues are the cited -17 and -1", ",".join(dec(x, 0) for x in eig) == MVL_EIGENVALUES)
    # their factorization: A = V diag(-1, -17) V^-1 with V = [[1, 3], [2, 4]]
    v = fmat([[1, 3], [2, 4]])
    v_inv = mscale(fmat([[4, -3], [-2, 1]]), F(1, -2))
    check("A = V diag(-1, -17) V^-1 exactly (Moler and Van Loan's construction)",
          mmul(mmul(v, fmat([[-1, 0], [0, -17]])), v_inv) == a)
    exact = expm_exact(a)
    emit("F-C.mvl.exact", mat(exact, 6))
    check("e^A to six places is what Moler and Van Loan print", mat(exact, 6) == MVL_CORRECT)
    closed = mmul(mmul(v, [[exp_fraction(-1), F(0)], [F(0), exp_fraction(-17)]]), v_inv)
    check("the exact series agrees with V diag(e^-1, e^-17) V^-1 to 25 places",
          max(abs(x) for r in msub(closed, exact) for x in r) < F(1, 10**25))
    steps = series(a, 80)
    for k in [16, 17]:
        emit(f"F-C.mvl.term.k{k}", mat(steps[k][1], 1))
    check("term 16 to one place is what Moler and Van Loan print",
          mat(steps[16][1], 1) == "6977251.9,-3488626.0;9303002.5,-4651501.3")
    emit("F-C.mvl.term.largest", dec(max(abs(x) for _, term, _ in steps for r in term for x in r), 1))
    emit("F-C.mvl.size", dec(size(a), 4))
    wrong, terms = series_f32(MVL)
    wrong = [[F(float(x)) for x in r] for r in wrong]
    emit("F-C.mvl.f32.series", mat(wrong, 6))
    emit("F-C.mvl.f32.terms", terms)
    emit("F-C.mvl.f32.worst_error", dec(max(abs(x) for r in msub(wrong, exact) for x in r), 4))
    emit("F-C.mvl.f32.wrong_signs", sum(1 for r, s in zip(wrong, exact) for x, y in zip(r, s) if (x < 0) != (y < 0)))
    scaled = [[F(float(x)) for x in r] for r in scale_square_f32(MVL, 8, 8)]
    emit("F-C.mvl.f32.scaled", mat(scaled, 6))
    emit("F-C.mvl.f32.scaled.halvings", 8)
    emit("F-C.mvl.f32.scaled.divisor", 2**8)
    emit("F-C.mvl.f32.scaled.terms", 8)
    emit("F-C.mvl.f32.scaled.worst_error", dec(max(abs(x) for r in msub(scaled, exact) for x in r), 6))
    theirs = scipy_expm(as_np(a))
    check("scipy.expm (64-bit) has e^A right to 1e-12", np.abs(theirs - as_np(exact)).max() < 1e-12)
    for m in range(1, 5):
        p = pade(m, F(1))
        emit(f"F-C.pade.m{m}", f"{p.numerator}/{p.denominator}")
        emit(f"F-C.pade.m{m}.value", dec(p, 6))
    pade_error = abs(pade(3, F(1)) - E_EXACT)
    series_error = abs(sum(F(1, math.factorial(k)) for k in range(7)) - E_EXACT)
    emit("F-C.pade.m3.error", dec(pade_error, 6))
    emit("F-C.series.k6.error", dec(series_error, 6))
    emit("F-C.pade.m3.times_closer", dec(series_error / pade_error, 1))
    # the [m/m] approximant agrees with e^x through x^(2m): its first error
    # term is (m!)^2 / ((2m)! (2m + 1)!) x^(2m + 1)
    for m in range(1, 7):
        x = F(1, 1000)
        gap = abs(pade(m, x) - exp_fraction(x, 70))
        first = F(math.factorial(m) ** 2, math.factorial(2 * m) * math.factorial(2 * m + 1)) * x ** (2 * m + 1)
        check(f"[{m}/{m}] Pade agrees with e^x through x^{2 * m}: its gap at x = 0.001 is the x^{2 * m + 1} term",
              abs(gap / first - 1) < F(1, 100))
    check("[13/13] Pade at x = 1 is e to 1e-15", abs(float(pade(13, F(1))) - math.e) < 1e-15)
    lam, vec = np.linalg.eigh(as_np(TANKS))
    emit("F-C.diag.tanks.exp_eigenvalues.t10", row(sorted(np.exp(10 * lam).tolist()), 6))
    rebuilt = (vec * np.exp(10 * lam)) @ vec.T
    emit("F-C.diag.tanks.rebuilt", bool(np.abs(rebuilt - as_np(expm_t(TANKS, 10))).max() < 1e-12))
    nil = fmat([[0, 1], [0, 0]])
    emit("F-C.nil.exp.t3", whole(expm_t(nil, 3)))
    once, once_terms = series_f64(MVL)
    emit("F-C.mvl.f64.terms", once_terms)
    emit("F-C.mvl.f64.right_to_six_places", mat([[F(x) for x in r] for r in once], 6) == mat(exact, 6))
    twice, _ = series_f64([[2 * x for x in r] for r in MVL])
    twice_exact = expm_exact(mscale(a, F(2)))
    emit("F-C.mvl.f64.doubled.entries_right_to_two_digits",
         sum(1 for r, t in zip(twice, twice_exact) for x, y in zip(r, t) if f"{x:.1e}" == f"{float(y):.1e}"))
    for h in [4, 8, 12, 16, 20]:
        got = [[F(float(x)) for x in r] for r in scale_square_f32(MVL, h, 8)]
        emit(f"F-C.mvl.f32.halvings.h{h}", dec(max(abs(x) for r in msub(got, exact) for x in r), 6))
    emit("F-C.mvl.f32.halvings.h20.result", mat([[F(float(x)) for x in r] for r in scale_square_f32(MVL, 20, 8)], 3))
    check("the series for the nilpotent matrix stops after two terms", mpow(nil, 2) == fmat([[0, 0], [0, 0]]))


# ---------------------------------------------------------------- F-O


def hat(v):
    a, b, c = v
    return [[0, -c, b], [c, 0, -a], [-b, a, 0]]


def rodrigues(v):
    """The rotation about the axis v by the angle |v| (Rodrigues' formula):
    I + sin(angle) K + (1 - cos(angle)) K^2, K the unit axis's skew matrix."""
    angle = math.sqrt(sum(x * x for x in v))
    if angle == 0:
        return np.eye(3)
    k = np.array(hat([x / angle for x in v]), dtype=np.float64)
    return np.eye(3) + math.sin(angle) * k + (1 - math.cos(angle)) * (k @ k)


def order_facts():
    a, b = fmat([[0, 1], [0, 0]]), fmat([[0, 0], [1, 0]])
    emit("F-O.A", whole(a))
    emit("F-O.B", whole(b))
    ea, eb = expm_exact(a), expm_exact(b)
    emit("F-O.expA", whole(ea))
    emit("F-O.expB", whole(eb))
    emit("F-O.expA_expB", whole(mmul(ea, eb)))
    emit("F-O.expB_expA", whole(mmul(eb, ea)))
    both = [[math.cosh(1), math.sinh(1)], [math.sinh(1), math.cosh(1)]]
    emit("F-O.exp_sum", mat(both, 6))
    check("the exact series for e^(A+B) is cosh 1 and sinh 1",
          np.abs(as_np(expm_exact(madd(a, b))) - np.array(both)).max() < 1e-15)
    emit("F-O.AB", whole(mmul(a, b)))
    emit("F-O.BA", whole(mmul(b, a)))
    emit("F-O.commutator", whole(commutator(a, b)))
    for t in ["1", "0.1", "0.01"]:
        ft = F(t)
        gap = msub(mmul(expm_t(a, ft), expm_t(b, ft)), expm_t(madd(a, b), ft))
        emit(f"F-O.gap.t{t}", mat(mscale(gap, 2 / (ft * ft)), 3))
    c, d = fmat([[1, 2], [2, 1]]), fmat([[3, 1], [1, 3]])
    emit("F-O.commuting.commutator", whole(commutator(c, d)))
    emit("F-O.commuting.rule_holds",
         max(abs(x) for r in msub(expm_exact(madd(c, d)), mmul(expm_exact(c), expm_exact(d))) for x in r) < F(1, 10**20))
    x, y = [3 * TAU, 0, 0], [0, 4 * TAU, 0]
    both = [p + q for p, q in zip(x, y)]
    eye = np.eye(3)
    emit("F-O.turns.x_is_identity", bool(np.abs(rodrigues(x) - eye).max() < 1e-9))
    emit("F-O.turns.y_is_identity", bool(np.abs(rodrigues(y) - eye).max() < 1e-9))
    emit("F-O.turns.sum_is_identity", bool(np.abs(rodrigues(both) - eye).max() < 1e-9))
    emit("F-O.turns.sum_full_turns", dec(math.sqrt(sum(v * v for v in both)) / TAU, 6))
    hx, hy = np.array(hat(x)), np.array(hat(y))
    emit("F-O.turns.commute", bool(np.abs(hx @ hy - hy @ hx).max() < 1e-9))
    # the square round a loop, in the order played: A, B, undo A, undo B
    def square_loop(t):
        t = F(t)
        return mmul(mmul(mmul(expm_t(b, -t), expm_t(a, -t)), expm_t(b, t)), expm_t(a, t))

    emit("F-O.loop.t0.5", mat(square_loop("0.5"), 4))
    t = F("0.01")
    leftover = mscale(msub(square_loop(t), ident(2)), 1 / (t * t))
    emit("F-O.loop.scaled.t0.01", mat(leftover, 3))
    emit("F-O.loop.is_minus_commutator", max(abs(x) for r in madd(leftover, commutator(a, b)) for x in r) < F(2, 100))
    check("the square's loop is exactly (1 - t^2, -t^3; t^3, 1 + t^2 + t^4)",
          square_loop(t) == [[1 - t**2, -(t**3)], [t**3, 1 + t**2 + t**4]])
    for n in [10, 1000]:
        step = mmul(expm_t(b, F(1, n)), expm_t(a, F(1, n)))
        emit(f"F-O.alternate.n{n}", mat(mpow(step, n), 3))
    for v in (x, y, both):
        check("scipy.expm of the spin agrees with Rodrigues' formula",
              np.abs(scipy_expm(np.array(hat(v), dtype=np.float64)) - rodrigues(v)).max() < 1e-9)


# ---------------------------------------------------------------- F-L


def lie_facts():
    lx, ly, lz = fmat(hat([1, 0, 0])), fmat(hat([0, 1, 0])), fmat(hat([0, 0, 1]))
    emit("F-L.Lz", whole(lz))
    emit("F-L.bracket.xy", whole(commutator(lx, ly)))
    emit("F-L.bracket.xy_is_z", commutator(lx, ly) == lz)
    emit("F-L.bracket.yz_is_x", commutator(ly, lz) == lx)
    emit("F-L.bracket.zx_is_y", commutator(lz, lx) == ly)
    emit("F-L.exp.Lz.t1", mat(rodrigues([0, 0, 1]).tolist(), 6))
    check("the exact series for e^(Lz) is the rotation about z by one radian",
          np.abs(as_np(expm_exact(lz)) - rodrigues([0, 0, 1])).max() < 1e-15)
    def cube_loop(t):
        """Tip about x, about y, undo x, undo y, about the fixed axes, in
        the order played (later motion on the left), by Rodrigues' formula."""
        return rodrigues([0, -t, 0]) @ rodrigues([-t, 0, 0]) @ rodrigues([0, t, 0]) @ rodrigues([t, 0, 0])

    for eps in ["0.1", "0.01"]:
        t = float(eps)
        emit(f"F-L.loop.t{eps}", mat(((cube_loop(t) - np.eye(3)) / (t * t)).tolist(), 3))
    small = (cube_loop(0.001) - np.eye(3)) * 1e6
    emit("F-L.loop.is_minus_z_spin", bool(np.abs(small + as_np(lz)).max() < 0.01))

    def angle_axis(r):
        """Angle from the trace; axis as the eigenvector that does not move,
        signed by the skew part."""
        angle = math.acos(max(-1.0, min(1.0, (np.trace(r) - 1) / 2)))
        vals, vecs = np.linalg.eig(r)
        axis = np.real(vecs[:, np.argmin(np.abs(vals - 1))])
        skew = np.array([r[2, 1] - r[1, 2], r[0, 2] - r[2, 0], r[1, 0] - r[0, 1]])
        if float(axis @ skew) < 0:
            axis = -axis
        return angle, (axis / np.linalg.norm(axis)).tolist()

    one = cube_loop(0.2)
    angle, axis = angle_axis(one)
    emit("F-L.creep.tip", "0.2")
    emit("F-L.creep.tip_degrees", dec(math.degrees(0.2), 1))
    emit("F-L.creep.loops", 20)
    emit("F-L.creep.one.degrees", dec(math.degrees(angle), 2))
    emit("F-L.creep.one.axis", row(axis, 3))
    total, total_axis = angle_axis(np.linalg.matrix_power(one, 20))
    emit("F-L.creep.total.degrees", dec(math.degrees(total), 1))
    emit("F-L.creep.total.axis", row(total_axis, 3))
    lean = math.degrees(math.atan2(math.hypot(total_axis[0], total_axis[1]), abs(total_axis[2])))
    emit("F-L.creep.axis_off_z_degrees", dec(lean, 1))
    emit("F-L.creep.direction", "backwards about z" if total_axis[2] < 0 else "forwards about z")
    check("twenty loops turn twenty times as far about the same axis", abs(total - 20 * angle) < 1e-9)
    check("scipy.expm gives the same loop", np.abs(
        scipy_expm(-0.2 * as_np(ly)) @ scipy_expm(-0.2 * as_np(lx)) @ scipy_expm(0.2 * as_np(ly)) @ scipy_expm(0.2 * as_np(lx)) - one).max() < 1e-13)
    for n in range(2, 6):
        emit(f"F-L.spins.n{n}", len(list(itertools.combinations(range(n), 2))))
    emit("F-L.su3", su_dim(3))
    emit("F-L.su2", su_dim(2))
    emit("F-L.u1", 1)
    emit("F-L.standard_model.steps", su_dim(3) + su_dim(2) + 1)
    check("the tiny steps of SU(3) x SU(2) x U(1) number the cited 8 + 2 + 1 + 1 force carriers",
          su_dim(3) + su_dim(2) + 1 == 8 + 2 + 1 + 1)
    emit("F-L.full_turn_is_identity", bool(np.abs(scipy_expm(TAU * as_np(J)) - np.eye(2)).max() < 1e-12))


def su_dim(n):
    """The dimension of su(n), counted: n by n complex matrices X with
    X* = -X and trace zero. Free real numbers: n(n-1)/2 complex entries above
    the diagonal (two each), n imaginary diagonal entries, less one for the
    trace."""
    return 2 * len(list(itertools.combinations(range(n), 2))) + n - 1


# ---------------------------------------------------------------- F-X

EDGE_SEED = 20_261_013
EDGE_CASES = 200


def same(a, b, tol):
    return float(np.abs(a - b).max()) <= tol * max(1.0, float(np.abs(a).max()), float(np.abs(b).max()))


def norm2(a):
    return float(np.linalg.norm(a, 2))


def edge_cases():
    rng = np.random.Generator(np.random.PCG64DXSM(EDGE_SEED))

    def draw(n):
        m = np.empty((n, n))
        for i in range(n):
            for j in range(n):
                m[i, j] = -3.0 + 6.0 * rng.random()
        return m

    cases = []
    for i in range(EDGE_CASES):
        n = 2 + i % 3
        a = draw(n)
        b = draw(n)
        s = 0.0 + 2.0 * rng.random()
        t = 0.0 + 2.0 * rng.random()
        cases.append((a, b, s, t))
    return cases


def edge_facts():
    emit("F-X.seed", EDGE_SEED)
    cases = edge_cases()
    n = len(cases)

    def settles(a):
        sz = norm2(a)
        term = np.eye(len(a))
        total = np.eye(len(a))
        under = True
        for k in range(1, 121):
            term = term @ a / k
            total = total + term
            if k <= 40 and not norm2(term) <= sz**k / math.factorial(k) * (1 + 1e-9) + 1e-12:
                under = False
        return under and same(total, scipy_expm(a), 1e-9)

    def commuting(a):
        b = 0.1 * (a @ a) - 0.5 * a
        return float(np.abs(a @ b - b @ a).max()) <= 1e-9 and same(scipy_expm(a + b), scipy_expm(a) @ scipy_expm(b), 1e-8)

    def generic(a, b):
        return float(np.abs(a @ b - b @ a).max()) > 1e-3 and not same(scipy_expm(a + b), scipy_expm(a) @ scipy_expm(b), 1e-3)

    def skew(a):
        r = scipy_expm(a - a.T)
        return float(np.abs(r.T @ r - np.eye(len(a))).max()) < 1e-11 and abs(float(np.linalg.det(r)) - 1) < 1e-11

    def undone(a):
        a = 0.25 * a
        e = scipy_expm(a)
        tr = math.exp(float(np.trace(a)))
        return same(e @ scipy_expm(-a), np.eye(len(a)), 1e-10) and abs(float(np.linalg.det(e)) - tr) <= 1e-10 * max(tr, 1.0)

    emit("F-X.series_settles", f"{sum(1 for a, _, _, _ in cases if settles(a))} of {n}")
    emit("F-X.sizes_multiply", f"{sum(1 for a, b, _, _ in cases if norm2(a @ b) <= norm2(a) * norm2(b) * (1 + 1e-12))} of {n}")
    emit("F-X.flow_adds",
         f"{sum(1 for a, _, s, t in cases if same(scipy_expm((s + t) * a), scipy_expm(s * a) @ scipy_expm(t * a), 1e-9))} of {n}")
    emit("F-X.commuting_rule_holds", f"{sum(1 for a, _, _, _ in cases if commuting(a))} of {n}")
    emit("F-X.generic_rule_fails", f"{sum(1 for a, b, _, _ in cases if generic(a, b))} of {n}")
    emit("F-X.skew_gives_rotation", f"{sum(1 for a, _, _, _ in cases if skew(a))} of {n}")
    emit("F-X.exponential_undone", f"{sum(1 for a, _, _, _ in cases if undone(a))} of {n}")
    times = [0.1 * m for m in range(1, 51)]
    heat_ok = 0
    for links, nodes in ((LINKS, NODES), ([(0, 1)], 2)):
        lap = as_np(laplacian(nodes, links))
        for t in times:
            h = scipy_expm(-t * lap)
            if all(abs(float(h[:, j].sum()) - 1) < 1e-12 and float(h[:, j].min()) > 0 for j in range(nodes)):
                heat_ok += 1
    emit("F-X.heat_conserves", f"{heat_ok} of {2 * len(times)}")
    pair = as_np(laplacian(2, [(0, 1)]))
    unitary_ok = sum(1 for t in times if abs(float((np.abs(scipy_expm(-1j * t * pair)[:, 0]) ** 2).sum()) - 1) < 1e-12)
    emit("F-X.quantum_total_one", f"{unitary_ok} of {len(times)}")


# ---------------------------------------------------------------- F-B


def cited_facts():
    emit("F-B.mvl.year", 1978)
    emit("F-B.mvl.revisit_year", 2003)
    emit("F-B.mvl.ways", 19)
    emit("F-B.mvl.ibm.result", MVL_IBM_RESULT)
    emit("F-B.mvl.ibm.terms", 59)
    emit("F-B.pade.degrees", "3,5,7,9,13")
    emit("F-B.higham.year", 2005)
    emit("F-B.al_mohy_higham.year", 2009)
    emit("F-B.estrada_hatano.year", 2008)
    emit("F-B.euler.year", 1748)
    emit("F-B.euler.e", EULER_E)
    emit("F-B.bernoulli.year", 1690)
    emit("F-B.laguerre.year", 1867)
    emit("F-B.peano.year", 1888)
    emit("F-B.lie.born", 1842)
    emit("F-B.lie.died", 1899)
    emit("F-B.bosons", "8,2,1,1")


# ---------------------------------------------------------------- F-M


def machine_facts():
    arrow = [F(2), F(1)]
    emit("F-M.machine.in", row(arrow, 0))
    emit("F-M.machine.out", row(apply(J, arrow), 0))
    spiral = fmat([[F(-3, 10), -1], [1, F(-3, 10)]])
    emit("F-M.spiral.rule", mat(spiral, 1))
    # -0.3 I and J commute, so e^{t(-0.3 I + J)} = e^{-0.3 t} times the rotation by t
    shrink = math.exp(-0.3)
    closed = [[shrink * math.cos(1), -shrink * math.sin(1)], [shrink * math.sin(1), shrink * math.cos(1)]]
    emit("F-M.spiral.exp.t1", mat(closed, 3))
    emit("F-M.spiral.shrink_per_unit_time", dec(shrink, 3))
    check("the exact series for the spiral rule is the shrunken rotation",
          np.abs(as_np(expm_exact(spiral)) - np.array(closed)).max() < 1e-15)


# ---------------------------------------------------------------- F-X, named


def fdet(m):
    """The determinant of a matrix of Fractions, by elimination."""
    m = [r[:] for r in m]
    n, det = len(m), F(1)
    for c in range(n):
        pivot = next((r for r in range(c, n) if m[r][c] != 0), None)
        if pivot is None:
            return F(0)
        if pivot != c:
            m[c], m[pivot] = m[pivot], m[c]
            det = -det
        det *= m[c][c]
        for r in range(c + 1, n):
            factor = m[r][c] / m[c][c]
            m[r] = [x - factor * y for x, y in zip(m[r], m[c])]
    return det


def near(a, b, tol=F(1, 10**20)):
    return max(abs(x) for r in msub(a, b) for x in r) < tol


def named_facts():
    lap = laplacian(NODES, LINKS)
    cases = {
        "zero": fmat([[0, 0], [0, 0]]),
        "identity": ident(2),
        "nilpotent": fmat([[0, 1], [0, 0]]),
        "defective": fmat([[1, 1], [0, 1]]),
        "quarter_turn": J,
        "five_quarter_turns": mscale(J, F(5)),
        "tanks": TANKS,
        "moler_van_loan": fmat(MVL),
        "one_by_one": fmat([[-4]]),
        "spin": fmat(hat([1, 2, 3])),
        "network": ADJ,
        "minus_laplacian": mscale(lap, F(-1)),
        "two_planes": fmat([[0, -1, 0, 0], [1, 0, 0, 0], [0, 0, 0, -2], [0, 0, 2, 0]]),
    }

    def settles(a):
        sz = size(a)
        under = all(size(term) <= sz**k / math.factorial(k) * (1 + 1e-9) + 1e-12 for k, term, _ in series(a, 40))
        before = len(_FAILED)
        expm_exact(a)  # the exact series, summed until its remainder is below 1e-30, against scipy.expm
        return under and len(_FAILED) == before

    def undone(a):
        a = mscale(a, F(1, 4))
        e = expm_exact(a)
        trace = sum(a[i][i] for i in range(len(a)))
        return near(mmul(e, expm_exact(mscale(a, F(-1)))), ident(len(a))) and abs(fdet(e) - exp_fraction(trace)) < F(1, 10**20)

    def rotation(a):
        r = expm_exact(a)
        return near(mmul(transpose(r), r), ident(len(a))) and abs(fdet(r) - 1) < F(1, 10**20)

    n = len(cases)
    emit("F-X.named.series_settles", f"{sum(1 for a in cases.values() if settles(a))} of {n}")
    emit("F-X.named.flow_adds",
         f"{sum(1 for a in cases.values() if near(expm_t(a, 2), mmul(expm_t(a, F(1, 2)), expm_t(a, F(3, 2)))))} of {n}")
    emit("F-X.named.exponential_undone", f"{sum(1 for a in cases.values() if undone(a))} of {n}")
    two = [a for a in cases.values() if len(a) == 2]
    emit("F-X.named.sizes_multiply",
         f"{sum(1 for a in two for b in two if size(mmul(a, b)) <= size(a) * size(b) * (1 + 1e-12) + 1e-12)} of {len(two) ** 2}")
    pairs = [
        (cases["zero"], cases["defective"]),
        (cases["identity"], cases["moler_van_loan"]),
        (cases["quarter_turn"], cases["five_quarter_turns"]),
        (cases["nilpotent"], cases["nilpotent"]),
        (cases["tanks"], cases["identity"]),
        (fmat([[1, 0], [0, 2]]), fmat([[3, 0], [0, -1]])),
    ]
    holds = sum(1 for a, b in pairs
                if commutator(a, b) == fmat([[0, 0], [0, 0]]) and near(expm_exact(madd(a, b)), mmul(expm_exact(a), expm_exact(b))))
    emit("F-X.named.commuting_rule_holds", f"{holds} of {len(pairs)}")
    skews = ["zero", "quarter_turn", "five_quarter_turns", "spin", "two_planes"]
    emit("F-X.named.skew_gives_rotation", f"{sum(1 for name in skews if rotation(cases[name]))} of {len(skews)}")
    # where the exact version bites: a network in two pieces levels only within each piece
    split = mscale(laplacian(4, [(0, 1), (2, 3)]), F(-1))
    late = column(expm_t(split, 40), 0)
    check("on a network in two pieces, heat levels only within its own piece (1/2, 1/2, 0, 0)",
          near([late], [[F(1, 2), F(1, 2), F(0), F(0)]], F(1, 10**20)))
    # the 1 by 1 matrix (-40): the exact series is right where a 64-bit sum is not
    exact = exp_fraction(-40, 60)
    total, term = 1.0, 1.0
    for k in range(1, 400):
        term = term * -40.0 / k
        total += term
    check("for the 1 by 1 matrix (-40) a 64-bit sum of the series is wrong and the exact series is not",
          abs(total - float(exact)) > 1.0 and abs(float(exact) - math.exp(-40)) < 1e-30)


# ---------------------------------------------------------------- F-Q

QUIZ_SECONDS = [("03_3", 4), ("04_4", 5), ("05_5", 5), ("06_3", 4), ("07_3", 4), ("08_3", 4),
                ("10_5", 4), ("11_3", 4), ("14_3", 4), ("15_2", 4), ("16_3", 5)]


def quiz_facts():
    a, b = fmat([[0, 1], [0, 0]]), fmat([[0, 0], [1, 0]])
    ea, eb = expm_exact(a), expm_exact(b)
    a_then_b, b_then_a = mmul(eb, ea), mmul(ea, eb)
    sizes = [F(5**k, math.factorial(k)) for k in range(61)]
    worst = max(abs(abs(complex(math.cos(0.1 * m), math.sin(0.1 * m))) - 1) for m in range(1, 63))
    one = (rodrigues([0, -0.2, 0]) @ rodrigues([-0.2, 0, 0]) @ rodrigues([0, 0.2, 0]) @ rodrigues([0.2, 0, 0]))
    off = math.degrees(math.acos((np.trace(one) - 1) / 2))
    rotation_by_one = mat([[math.cos(1), -math.sin(1)], [math.sin(1), math.cos(1)]], 3)
    answers = {
        "03_3": f"no: it settles at {dec(E_EXACT, 5)}",
        "04_4": f"x^3/{math.factorial(3)}",
        "05_5": dec(exp_fraction(F(5, 100) * 20), 3),
        "06_3": f"a circle of radius {dec(1 + worst, 3)}",
        "07_3": whole(mpow(J, 2)),
        "08_3": "they shrink: " + ("true" if all(sizes[k + 1] < sizes[k] for k in range(5, 60)) else "false"),
        "10_5": row(tank_closed(1000), 0),
        "11_3": count_walks(2)[2][2],
        "14_3": f"{'yes' if a_then_b == b_then_a else 'no'}: {whole(a_then_b)} against {whole(b_then_a)}",
        "15_2": f"{'yes' if off < 1e-7 else 'no'}: off by {dec(off, 2)} degrees",
        "16_3": f"a rotation by 1 radian: {rotation_by_one}",
    }
    emit("F-Q.count", len(QUIZ_SECONDS))
    for line, seconds in QUIZ_SECONDS:
        emit(f"F-Q.{line}.seconds", seconds)
        emit(f"F-Q.{line}.answer", answers[line])


# ---------------------------------------------------------------- F-V


def spoken_facts():
    emit("F-V.01_2.degrees.shown", dec(180 / PI, 1))
    emit("F-V.02_1.e", dec(E_EXACT, 3))
    emit("F-V.03_2.two_steps", dec(F(3, 2) ** 2, 2))
    emit("F-V.03_3.four_steps", dec(F(5, 4) ** 4, 2))
    emit("F-V.03_3.twelve_steps", dec(F(13, 12) ** 12, 2))
    emit("F-V.03_4.daily", dec(compound(F(1), 365), 3))
    emit("F-V.03_4.million", dec(compound(F(1), 1_000_000), 5))
    emit("F-V.03_4.e", dec(E_EXACT, 5))
    running, total = [], F(1)
    for k in range(1, 10):
        total += F(1, math.factorial(k))
        running.append(total)
    emit("F-V.05_1.running", row(running[:6], 3))
    emit("F-V.05_1.settles", dec(running[8], 5))
    emit("F-V.05_3.share", "0.05")
    emit("F-V.05_6.rate_times_time", dec(F(5, 100) * 20, 0))
    emit("F-V.05_6.factor", dec(exp_fraction(F(5, 100) * 20), 3))
    for name, n in (("four", 4), ("hundred", 100)):
        z = (1 + 1j * PI / n) ** n
        emit(f"F-V.06_4.{name}_steps", row([z.real, z.imag], 2))
    emit("F-V.07_6.at_one", mat([[math.cos(1), -math.sin(1)], [math.sin(1), math.cos(1)]], 3))
    sizes = [F(5**k, math.factorial(k)) for k in range(21)]
    emit("F-V.08_2.bars", row(sizes[:6], 1))
    emit("F-V.08_2.near", dec(sizes[4], 0))
    steps = series(mscale(J, F(5)), 20)
    far = max(math.sqrt(sum(x * x for x in column(total, 0))) for _, _, total in steps)
    emit("F-V.08_2.swings_past", dec(math.floor(far), 0))
    emit("F-V.08_4.term20.tens_of_millionths", dec(sizes[20] * 10**5, 0))
    emit("F-V.08_4.home", row([math.cos(5), math.sin(5)], 3))
    emit("F-V.08_5.bound", dec(exp_fraction(5), 1))
    emit("F-V.10_3.one_step", row(apply(tiny_steps(TANKS, 10, 1), START), 0))
    emit("F-V.10_4.ten_steps", dec(apply(tiny_steps(TANKS, 10, 10), START)[0], 2))
    emit("F-V.10_4.thousand_steps", dec(apply(tiny_steps(TANKS, 10, 1000), START)[0], 2))
    emit("F-V.10_4.exact", dec(tank_closed(10)[0], 2))
    emit("F-V.10_6.minutes.said", dec(math.log(50) / 0.2, 0))
    emit("F-V.10_6.minutes", dec(math.log(50) / 0.2, 1))
    comm = expm_exact(ADJ)
    emit("F-V.11_5.one_to_four", dec(comm[0][3], 2))
    emit("F-V.11_6.one_to_three", dec(comm[0][2], 2))
    emit("F-V.11_6.one_to_two", dec(comm[0][1], 2))
    emit("F-V.12_2.level", row([F(1, 4)] * 4, 2))
    emit("F-V.12_4.nodes", 20 * 20)
    exact = expm_exact(MVL)
    largest = max(abs(x) for _, term, _ in series(MVL, 80) for r in term for x in r)
    emit("F-V.13_2.millions", dec(math.floor(largest / 10**6), 0))
    emit("F-V.13_2.exact", mat(exact, 3))
    emit("F-V.13_2.all_smaller_than_two", max(abs(x) for r in exact for x in r) < 2)
    wrong, terms = series_f32(MVL)
    wrong = [[F(float(x)) for x in r] for r in wrong]
    emit("F-V.13_3.series", mat(wrong, 3))
    emit("F-V.13_3.stopped_at_k", terms)
    emit("F-V.13_3.entries_right_to_two_digits",
         sum(1 for r, t in zip(wrong, exact) for x, y in zip(r, t) if f"{float(x):.1e}" == f"{float(y):.1e}"))
    scaled = [[F(float(x)) for x in r] for r in scale_square_f32(MVL, 8, 8)]
    emit("F-V.13_4.scaled", mat(scaled, 3))
    emit("F-V.13_4.right_to_three_places", mat(scaled, 3) == mat(exact, 3))
    taylor6 = sum(F(1, math.factorial(k)) for k in range(7))
    emit("F-V.13_5.series", dec(taylor6, 5))
    emit("F-V.13_5.pade", dec(pade(3, F(1)), 5))
    emit("F-V.13_5.times_closer", dec(abs(taylor6 - E_EXACT) / abs(pade(3, F(1)) - E_EXACT), 0))
    emit("F-V.14_4.both_at_once", mat([[math.cosh(1), math.sinh(1)], [math.sinh(1), math.cosh(1)]], 3))
    emit("F-V.15_1.spins", f"{len(list(itertools.combinations('xyz', 2)))},{len(list(itertools.combinations('xyzw', 2)))}")
    emit("F-V.16_2.steps", f"{su_dim(3)},{su_dim(2)},1,{su_dim(3) + su_dim(2) + 1}")


def main():
    number_facts()
    hook_facts()
    quarter_facts()
    settle_facts()
    tank_facts()
    network_facts()
    spreading_facts()
    computing_facts()
    order_facts()
    lie_facts()
    machine_facts()
    edge_facts()
    named_facts()
    cited_facts()
    quiz_facts()
    spoken_facts()
    print(f"# {len(_EMITTED)} keys; {_CHECKS[0]} cross-checks, {len(_FAILED)} failed; "
          f"{_EXPM_CHECKED[0]} exact exponentials compared with scipy.expm", file=sys.stderr)
    if _FAILED:
        for name in _FAILED:
            print(f"# FAILED: {name}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
