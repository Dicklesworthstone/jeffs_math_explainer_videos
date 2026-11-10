#!/usr/bin/env python3
"""Separate exact-rational author checks for finite displayed examples."""
import sys as _oracle_sys


class _RowRecorder:
    """Keeps every printed line, so the last section can check the whole output for duplicate keys."""

    def __init__(self, stream):
        self.stream, self.parts = stream, []

    def write(self, text):
        self.parts.append(text)
        return self.stream.write(text)

    def flush(self):
        self.stream.flush()


_row_recorder = _RowRecorder(_oracle_sys.stdout)
_oracle_sys.stdout = _row_recorder

from fractions import Fraction
from math import factorial

facts = [
    ("slide.a", 2), ("slide.b", 3), ("slide.product", 2 * 3),
    ("growth.base", 2), ("growth.first", 3), ("growth.second", 2),
    ("growth.first_factor", 2 ** 3), ("growth.second_factor", 2 ** 2),
    ("growth.total_factor", 2 ** 5),
]
for n in (1, 2, 4, 16, 64, 1024):
    facts.append((f"limit.{n}", f"{float(Fraction(n + 1, n) ** n):.5f}"))
facts.append(("series.order", 8))
facts.append(("series.sum", f"{float(sum((Fraction(1, factorial(k)) for k in range(9)), Fraction())):.5f}"))
# Separate exact rational formula, not a translation of the Rust Euler loop.
facts.extend([("rate.start", "1.000000"), ("rate.duration", "1.000000")])
counts = (4, 8, 16)
for n in counts:
    facts.append((f"rate.{n}.steps", n))
    for k in range(n + 1):
        t = Fraction(k, n)
        y = Fraction(n + 1, n) ** k
        linear = 1 + t
        for field, value in (("t", t), ("proportional", y), ("constant", linear)):
            facts.append((f"rate.{n}.{k}.{field}", f"{float(value):.6f}"))
        facts.append((f"rate.{n}.{k}.right_readout", f"height {float(y):.2f}: slope {float(y):.2f}"))
        facts.append((f"rate.{n}.{k}.left_readout", f"height {float(linear):.2f}: slope 1.00"))
    previous = Fraction(n + 1, n) ** (n - 1)
    endpoint = Fraction(n + 1, n) ** n
    facts.append((f"rate.{n}.last_step", f"last step: slope {float(previous):.2f}, height now {float(endpoint):.2f}"))
for k in range(1, len(counts) + 1):
    ns = counts[:k]
    names = ", ".join(map(str, ns))
    ends = ", ".join(f"{float(Fraction(n + 1, n) ** n):.3f}" for n in ns)
    facts.append((f"rate.refinement.{k}", f"{names} steps: {ends}"))
assert len(dict(facts)) == len(facts), "duplicate fact key"

for key, value in facts:
    print(f"{key}\t{value}")


#!/usr/bin/env python3
"""Exact polynomial arithmetic; no recurrence or native-model import.

The bars use the exact running product of ratios R/i, not the native power over factorial.
The equations are rebuilt from the column they come from, and each is checked by solving it.
"""
from fractions import Fraction
from math import ceil, factorial

RADIUS = Fraction(3, 2)
SEGMENTS = 60
LETTERS = {1: "a", 2: "b", 3: "c", 4: "d"}


def show(q):
    return str(q.numerator) if q.denominator == 1 else f"{q.numerator}/{q.denominator}"


def five(q):
    """Five places, refusing an exact tie at the sixth, where float rounding could differ."""
    scaled = q * 200000
    assert scaled.denominator != 1 or scaled.numerator % 2 == 0, "an exact tie at the fifth place"
    return f"{float(q):.5f}"


rows = [
    ("factorials.plot.radius", "1.500000"),
    ("factorials.plot.top", "5.000000"),
    ("factorials.plot.segments", SEGMENTS),
]
for n in range(5):
    denominator = factorial(n)
    rows.append((f"factorials.coefficient.{n}", "1" if denominator == 1 else f"1/{denominator}"))
    rows.append((f"factorials.denominator.{n}", denominator))
    rows.append((f"factorials.running.{n}", "1" if n == 0 else r"\cdot ".join(str(i) for i in range(1, n + 1))))
    for j in range(SEGMENTS + 1):
        x = RADIUS * j / SEGMENTS
        y = sum((x**k / factorial(k) for k in range(n + 1)), Fraction())
        rows.append((f"factorials.curve.{n}.{j}.x", f"{float(x):.6f}"))
        rows.append((f"factorials.curve.{n}.{j}.y", five(y)))
        assert 0 <= x <= RADIUS and 0 <= y <= 5

# Each equation is what its column says: n times the new coefficient equals the coefficient above it.
coefficient = {0: Fraction(1)}
for n in range(1, 5):
    above = coefficient[n - 1]
    solved = above / n
    coefficient[n] = solved
    left = LETTERS[n] if n == 1 else f"{n}{LETTERS[n]}"
    rows.append((f"factorials.derivative.{n}", left))
    rows.append((f"factorials.match.{n}.equation", f"{left} = {show(above)}"))
    rows.append((f"factorials.match.{n}.value", f"{LETTERS[n]} = {show(solved)}"))
    assert n * solved == above and solved == Fraction(1, factorial(n))

# The first index with R/index <= 1/2, found by walking the ratios.
index = 1
while RADIUS / index > Fraction(1, 2):
    index += 1
rows.append(("factorials.halving.from", index))
assert RADIUS / index == Fraction(1, 2), "the printed sign is non-strict because equality is reached"

for i in range(1, 9):
    rows.append((f"factorials.ratio.{i}", show(RADIUS / i)))

# Bars: the running product of the ratios themselves.
bar = {0: Fraction(1)}
for k in range(1, 10):
    bar[k] = bar[k - 1] * RADIUS / k
    assert bar[k] == RADIUS**k / factorial(k)
for k in range(10):
    rows.append((f"factorials.bar.{k}", five(bar[k])))

# The ceiling over the zoomed tail, and every bar under it, past the drawn range.
ceiling = {5: bar[5]}
for k in range(6, 30):
    ceiling[k] = ceiling[k - 1] / 2
    assert RADIUS**k / factorial(k) <= ceiling[k]
for k in range(5, 10):
    rows.append((f"factorials.envelope.{k}", five(ceiling[k])))
rows.append(("factorials.zoom", 16))

for n in (4, 8):
    assert n + 2 >= 2 * RADIUS
    bound = 2 * RADIUS ** (n + 1) / factorial(n + 1)
    partial = sum((RADIUS**k / factorial(k) for k in range(n + 1, 60)), Fraction())
    assert partial < bound
    rounded_up = Fraction(ceil(bound * 100000), 100000)
    assert rounded_up >= bound
    rows.append((f"factorials.tail.{n}.bound", f"{float(rounded_up):.5f}"))

assert len(rows) == len(dict(rows)) == 667
for key, value in rows:
    print(f"{key}\t{value}")


#!/usr/bin/env python3
"""Exact arithmetic examples of the complete diagonal identity, not a general proof, plus the grid's fact rows."""
from fractions import Fraction
from math import factorial, comb
import json
import sys

checks = 0
for m in range(13):
    for x in [Fraction(-2), Fraction(-1, 2), Fraction(0), Fraction(1, 3), Fraction(2)]:
        for y in [Fraction(-2), Fraction(-1, 2), Fraction(0), Fraction(1, 3), Fraction(2)]:
            diagonal = sum((x**k * y**(m-k) / (factorial(k)*factorial(m-k)) for k in range(m+1)), Fraction())
            assert diagonal == (x+y)**m/factorial(m)
            checks += 1
    for k in range(m+1):
        assert Fraction(factorial(m),factorial(k)*factorial(m-k)) == comb(m,k)
print(json.dumps({"exact_diagonal_examples":checks,"max_degree":12,"displayed_complete_degrees":[0,1,2,3],"native_comparison":"not run","general_proof":"binomial theorem plus absolute convergence; independent review open"},indent=2), file=sys.stderr)

# Fact rows. Each cell comes through the binomial route C(i+j, i)/(i+j)!, not the native i!j!.
SIDE = 4
rows = [("product.side", SIDE), ("product.cells", SIDE**2)]
for i in range(SIDE):
    for j in range(SIDE):
        value = Fraction(comb(i + j, i), factorial(i + j))
        assert value.numerator == 1
        rows.append((f"product.cell.{i}.{j}.denominator", value.denominator))
for m in range(SIDE):
    rows.append((f"product.diagonal.{m}.cells", sum(1 for i in range(SIDE) for j in range(SIDE) if i + j == m)))
    rows.append((f"product.total.{m}.denominator", sum(comb(m, k) for k in range(m + 1)) * factorial(m) // 2**m))

# 03_7. E(1) between an exact partial sum and that sum plus a bound on its tail, so the printed digits are proved.
partial = sum((Fraction(1, factorial(k)) for k in range(25)), Fraction())
tail = Fraction(2, factorial(25))
assert f"{float(partial):.5f}" == f"{float(partial + tail):.5f}" == "2.71828"
rows.append(("product.e", f"{float(partial):.5f}"))
LOW, HIGH, STEPS = -2, 3, 4
marks = [Fraction(j, STEPS) for j in range(LOW * STEPS, HIGH * STEPS + 1)]
whole = [q for q in marks if q >= 0 and q.denominator == 1]
fractions = [q for q in marks if q > 0 and q.denominator != 1]
negatives = [q for q in marks if q < 0]
assert len(whole) + len(fractions) + len(negatives) == len(marks)
rows.extend([
    ("product.ladder.low", LOW),
    ("product.ladder.high", HIGH),
    ("product.ladder.steps", STEPS),
    ("product.ladder.whole", len(whole)),
    ("product.ladder.fractions", len(fractions)),
    ("product.ladder.negatives", len(negatives)),
])
assert len(rows) == len(dict(rows)) == 33
for key, value in rows:
    print(f"{key}\t{value}")

#!/usr/bin/env python3
"""Exact rational arithmetic for the small-multiplier chapter; imports no native model.

Coefficients come from Pascal's rule, not from the native chain of factors. Endpoints are exact powers of exact
fractions. E at an input is an exact partial sum bracketed by its tail bound, and each printed value is accepted only
when both ends of the bracket print the same digits.
"""
from fractions import Fraction as Q
from math import ceil, factorial

R = Q(3, 2)
TABLE = (4, 8, 16, 64, 1024)
SWEEP = 16
FILL = (4, 16, 64, 256, 1024)
FILL_TOP = 100
FILL_COLUMNS = (3, 100)
WARNING = (4, 16, 40, 100)
WARNING_COLUMN = 3
BARS = (4, 16, 64, 1024)
BAR_TOP = 9
EARLY, CUT = 4, 8
AIM = Q(1, 100)
HEADS = (64, 256, 1024)
EDGE = Q(-3)
EDGE_COUNTS = (1, 2, 3, 4, 16, 64, 1024)
SAMPLES = 90
WALKS = (4, 16)


def places(q, d):
    """Rounded at d places from the exact value. A true tie goes to the even digit, as Rust prints an exact float."""
    scaled = abs(q) * 10**d
    whole = scaled.numerator // scaled.denominator
    rest = scaled - whole
    if rest > Q(1, 2) or (rest == Q(1, 2) and whole % 2 == 1):
        whole += 1
    text = f"{whole:0{d + 1}d}"
    body = text[:-d] + "." + text[-d:]
    return ("-" if q < 0 and whole != 0 else "") + body


def bracket(x, degree=48):
    """E(x) between exact bounds: the partial sum and the first omitted term, doubled."""
    assert degree + 2 >= 2 * abs(x)
    s = sum((x**k / factorial(k) for k in range(degree + 1)), Q(0))
    w = 2 * abs(x) ** (degree + 1) / factorial(degree + 1)
    return s - w, s + w


def e_places(x, d):
    low, high = bracket(x)
    a, b = places(low, d), places(high, d)
    assert a == b, f"E({x}) is too close to a rounding edge at {d} places"
    return a


def size(x, k):
    return abs(x) ** k / factorial(k)


def pascal(n):
    row = [1]
    for _ in range(n):
        row = [1] + [row[i] + row[i + 1] for i in range(len(row) - 1)] + [1]
    return row


def chain_small(n, k):
    """C(n,k) k! / n^k from Pascal's row: nothing here multiplies correction factors."""
    return Q(pascal(n)[k] * factorial(k), n**k) if k <= n else Q(0)


def chain(n, k):
    """The falling factorial over n^k, in integers, still with no 1 - j/n factor. Zero once k passes n."""
    if k > n:
        return Q(0)
    top = 1
    for i in range(k):
        top *= n - i
    return Q(top, n**k)


assert all(chain(n, k) == chain_small(n, k) for n in (1, 2, 3, 4, 16) for k in range(20)), "the two counts must agree"


def up(q):
    """Round up at the fifth place, the rule the printed tail bounds use."""
    return places(Q(ceil(q * 10**5), 10**5), 5)


def tail(cut):
    return up(2 * size(R, cut + 1))


def head(n, cut=CUT):
    return sum((size(R, k) * (1 - chain(n, k)) for k in range(cut + 1)), Q(0))


def distance(x, n):
    """|E(x) - (1+x/n)^n|, bracketed, printed only when both ends agree."""
    power = (1 + x / n) ** n
    low, high = bracket(x)
    a, b = abs(low - power), abs(high - power)
    p, q = places(min(a, b), 5), places(max(a, b), 5)
    assert p == q, f"distance at {x}, {n} is too close to a rounding edge"
    return p


rows = [("tiny.step.multiplier", "5/4")]
for n in TABLE:
    rows.append((f"tiny.one.{n}", places((1 + Q(1, n)) ** n, 3)))
rows.append(("tiny.sweep.cells", SWEEP))
for x in (R, -R):
    rows.append((f"tiny.sweep.{float(x)}.endpoint", places((1 + x / SWEEP) ** SWEEP, 3)))
    rows.append((f"tiny.sweep.{float(x)}.series", e_places(x, 3)))
for k in range(5):
    rows.append((f"tiny.ways.{k}", pascal(4)[k]))
    rows.append((f"tiny.strip.{k}", f"{pascal(4)[k]}/{4**k}"))
    rows.append((f"tiny.chain.4.{k}", places(chain(4, k), 5)))
for n in FILL:
    rows.append((f"tiny.fill.{n}", " ".join(places(chain(n, k), 5) for k in range(FILL_TOP + 1))))
    for k in FILL_COLUMNS:
        rows.append((f"tiny.column.{n}.{k}", places(chain(n, k), 3)))
rows.append(("tiny.warning.column", WARNING_COLUMN))
for n in WARNING:
    rows.append((f"tiny.warning.{n}", f"1/{n}"))
rows.append(("tiny.warning.total", 1))
for n in BARS:
    rows.append((f"tiny.bars.{n}", " ".join(places(size(R, k) * chain(n, k), 5) for k in range(BAR_TOP + 1))))
rows.append(("tiny.aim", "0.01"))
for cut in (EARLY, CUT):
    rows.append((f"tiny.cut.{cut}.tail", tail(cut)))
for n in HEADS:
    printed_head = up(head(n))
    rows.append((f"tiny.head.{n}", printed_head))
    rows.append((f"tiny.total.{n}", places(Q(printed_head) + Q(tail(CUT)), 5)))
    rows.append((f"tiny.distance.{n}", distance(R, n)))
    rows.append((f"tiny.negative.{n}", distance(-R, n)))
for n in EDGE_COUNTS:
    base = 1 + EDGE / n
    value = base**n
    base_text = str(base.numerator) if base.denominator == 1 else f"{base.numerator}/{base.denominator}"
    # A reduced fraction is what the table prints; n = 4 reduces to itself, the others are odd over a power of two.
    rows.append((f"tiny.edge.{n}.base", base_text))
    if n <= 3:
        value_text = str(value.numerator) if value.denominator == 1 else f"{value.numerator}/{value.denominator}"
    else:
        value_text = places(value, 4)
    rows.append((f"tiny.edge.{n}.value", value_text))
rows.append(("tiny.edge.series", e_places(EDGE, 4)))
for j in range(SAMPLES + 1):
    x = EDGE + Q(j, 20)
    rows.append((f"tiny.curve.{j}", f"{places(x, 2)} {e_places(x, 5)}"))
for n in WALKS:
    for side, x in (("right", R), ("left", EDGE)):
        step = 1 + x / n
        rows.append((f"tiny.walk.{n}.{side}", " ".join(places(step**j, 5) for j in range(n + 1))))

assert len(rows) == 1 + 5 + 1 + 4 + 15 + 5 + 10 + 1 + 4 + 1 + 4 + 1 + 2 + 12 + 14 + 1 + 91 + 4, len(rows)
for key, value in rows:
    print(f"{key}\t{value}")
#!/usr/bin/env python3
"""Exact rational arithmetic for the uniqueness chapter; imports no native model.

E at an input is an exact partial sum bracketed by its tail bound. A printed value is accepted only when both ends
of its bracket print the same digits, so a value that could round either way stops this oracle.
"""
from fractions import Fraction as Q
from math import factorial

PANEL = Q(3, 2)
SAMPLES = 30
DEGREES = (2, 4)
FAR = (Q(-2), Q(-1, 2))
NEAR = (Q(-1, 4), Q(1))
MULTIPLE = 3
PIECE_SAMPLES = 15
RATES = ((Q(1, 2), "1/2"), (Q(0), "0"), (Q(-1, 2), "-1/2"))
RATE_START = 2
RATE_END = 2
RATE_SAMPLES = 20
AXIS_TOPS = (5, 3, 3, 6)


def places(q, d):
    """Rounded at d places from the exact value. A true tie goes to the even digit, as Rust prints an exact float."""
    scaled = abs(q) * 10**d
    whole = scaled.numerator // scaled.denominator
    rest = scaled - whole
    if rest > Q(1, 2) or (rest == Q(1, 2) and whole % 2 == 1):
        whole += 1
    text = f"{whole:0{d + 1}d}"
    body = text[:-d] + "." + text[-d:]
    return ("-" if q < 0 and whole != 0 else "") + body


def bracket(x, degree=48):
    assert degree + 2 >= 2 * abs(x)
    s = sum((x**k / factorial(k) for k in range(degree + 1)), Q(0))
    w = 2 * abs(x) ** (degree + 1) / factorial(degree + 1)
    return s - w, s + w


def scaled_places(low, high, d):
    a, b = places(low, d), places(high, d)
    assert a == b, f"too close to a rounding edge: {a} {b}"
    return a


def e5(x, multiplier=Q(1), d=5):
    low, high = bracket(x)
    return scaled_places(min(multiplier * low, multiplier * high), max(multiplier * low, multiplier * high), d)


def partial(x, degree):
    return sum((x**k / factorial(k) for k in range(degree + 1)), Q(0))


def product5(x, degree):
    """The polynomial times E(-x), both ends of E(-x)'s bracket."""
    p = partial(x, degree)
    low, high = bracket(-x)
    return scaled_places(min(p * low, p * high), max(p * low, p * high), 5)


def imagined5(x):
    factor = 1 - x * x / 6
    low, high = bracket(x)
    return scaled_places(min(factor * low, factor * high), max(factor * low, factor * high), 5)


inputs = [-PANEL + 2 * PANEL * Q(j, SAMPLES) for j in range(SAMPLES + 1)]
rows = [
    ("only.e", " ".join(e5(x) for x in inputs)),
    ("only.imagined", " ".join(imagined5(x) for x in inputs)),
]
rows.append(("only.axes.tops", " ".join(str(top) for top in AXIS_TOPS)))


def defect_denominator(d):
    """From the polynomial's coefficients: y' - y keeps only minus its top term, and this is that term's denominator."""
    y = [Q(1, factorial(k)) for k in range(d + 1)]
    slope = [(k + 1) * y[k + 1] for k in range(d)] + [Q(0)]
    defect = [slope[k] - y[k] for k in range(d + 1)]
    assert all(c == 0 for c in defect[:-1]), defect
    assert defect[-1].numerator == -1, defect
    return defect[-1].denominator


for d in DEGREES:
    rows.append((f"only.polynomial.{d}", " ".join(places(partial(x, d), 5) for x in inputs)))
    rows.append((f"only.product.{d}", " ".join(product5(x, d) for x in inputs)))
    rows.append((f"only.defect.{d}", defect_denominator(d)))
rows.append(("only.far.multiple", MULTIPLE))
far = [FAR[0] + (FAR[1] - FAR[0]) * Q(j, PIECE_SAMPLES) for j in range(PIECE_SAMPLES + 1)]
near = [NEAR[0] + (NEAR[1] - NEAR[0]) * Q(j, PIECE_SAMPLES) for j in range(PIECE_SAMPLES + 1)]
rows.append(("only.far", " ".join(e5(x, Q(MULTIPLE)) for x in far)))
rows.append(("only.near", " ".join(e5(x) for x in near)))
rows.append(("only.rate.start", RATE_START))
rows.append(("only.rate.end", RATE_END))
for rate, name in RATES:
    times = [Q(RATE_END * j, RATE_SAMPLES) for j in range(RATE_SAMPLES + 1)]
    rows.append((f"only.rate.{name}", " ".join(e5(rate * t, Q(RATE_START)) for t in times)))
    rows.append((f"only.rate.{name}.end", e5(rate * RATE_END, Q(RATE_START), 3)))

assert len(rows) == 2 + 1 + 6 + 3 + 2 + 6, len(rows)
for key, value in rows:
    print(f"{key}\t{value}")

# Slide rule, independent high-precision logarithmic geometry.
from decimal import Decimal, localcontext
with localcontext() as ctx:
    ctx.prec = 80
    def ruler_x(n):
        return Decimal(-3) + Decimal('5.5') * Decimal(str(n)).ln() / Decimal(10).ln()
    def ruler_label(n):
        return format(n, 'f').rstrip('0').rstrip('.') if '.' in format(n, 'f') else str(n)
    for n in range(1, 11):
        print(f'ruler.tick.{n}\t{ruler_x(n):.8f}')
    for i,(a,b) in enumerate([(Decimal(2),Decimal(3)),(Decimal('2.5'),Decimal(4)),(Decimal(4),Decimal(3))]):
        print(f'ruler.example.{i}.equation\t{ruler_label(a)} × {ruler_label(b)} = {ruler_label(a*b)}')
        print(f'ruler.example.{i}.left_offset\t{ruler_x(a)-ruler_x(1):.8f}')
        value=a*b if a*b<=10 else a*b/10
        print(f'ruler.example.{i}.reading\t{ruler_label(value)}')
    print(f'ruler.wrap.offset\t{ruler_x(4)-ruler_x(10):.8f}')
    print('ruler.division\t6 ÷ 3 = 2')

# 05_6: what the chart draws, what the bound uses, and the gaps each leaves under the ceilings, as exact rationals.
DISPLAY_CELLS, BOUND_CELLS = 16, 1024
print(f"tiny.display.cells\t{DISPLAY_CELLS}")
print(f"tiny.display.bound_cells\t{BOUND_CELLS}")
for n in (DISPLAY_CELLS, BOUND_CELLS):
    gaps = " ".join(places(size(R, k) * (1 - chain(n, k)), 5) for k in range(BAR_TOP + 1))
    print(f"tiny.display.gaps.{n}\t{gaps}")

# Chapters 08 and 09: independent Decimal values for the chosen chemistry and circuit examples, 454 rows. Imports no
# native model. Numbers are printed at ten places from 80-digit values, and the strings the viewer reads are rebuilt
# here and checked against literals.
from decimal import ROUND_FLOOR, localcontext
from decimal import Decimal as D


def decimal_text(value, digits=10):
    """Fixed places from an 80-digit value, refusing a value so close to a tie that the rounding could differ."""
    scaled = value * D(10) ** digits
    rest = scaled - scaled.to_integral_value(rounding=ROUND_FLOOR)
    assert abs(rest - D("0.5")) > D("1e-40"), f"{value} is a tie at {digits} places"
    return f"{value:.{digits}f}"


def application_rows():
    with localcontext() as ctx:
        ctx.prec = 80
        out = []
        k = D(2).ln()
        for n in range(9):
            fraction = (-k * D(n)).exp()
            assert abs(fraction - D(1) / D(2) ** n) < D("1e-75")
            out.append((f"chem_half_{n}", decimal_text(D(100) * fraction)))
        for n in range(9):
            # A' = -A^2 with A(0) = 1: the reciprocal grows linearly, so A = 1 / (1 + n).
            out.append((f"chem_second_order_{n}", decimal_text(D(1) / (1 + D(n)))))
        r, c, v = D(10000), D("0.0001"), D(12)
        tau = r * c
        assert tau == 1
        out.append(("rc_tau_s", decimal_text(tau)))
        for n in range(101):
            t = D(n) / 10
            gap = v * (-t / tau).exp()
            voltage = v - gap
            current = gap / r
            slow_gap = v * (-2 * t / ((2 * r) * c)).exp()
            assert slow_gap == gap
            assert current * r == gap and voltage + gap == v
            for label, value in (
                ("voltage_v", voltage),
                ("gap_v", gap),
                ("current_a", current),
                ("charge_c", c * voltage),
            ):
                out.append((f"rc_{n:03d}_{label}", decimal_text(value)))

        one_gap = v * D(-1).exp()
        warmed_left = (-2 * k).exp()
        assert abs(warmed_left - D(1) / 4) < D("1e-75")
        golden = (D(5).sqrt() - 1) / 2
        waits = sorted(-(1 - (D(i) * golden) % 1).ln() / k for i in range(1, 21))
        assert all(a < b for a, b in zip(waits, waits[1:])), "the molecules change at distinct times"
        run_end, window_start, window_end, window_low = D("3.6"), D(4), D(6), D("11.7")
        resolution = D("0.05")
        settle = tau * (v / resolution).ln()
        assert window_start < settle < window_end
        assert v - v * (-window_start / tau).exp() > window_low
        assert v * (-window_end / tau).exp() > 0
        display = [
            ("decay.interval", decimal_text(k / k)),
            ("decay.fixed.empty_time", decimal_text(1 / k)),
            ("decay.step.0", "100 → 50"),
            ("decay.step.1", "50 → 25"),
            ("decay.step.2", "25 → 12.5"),
            ("decay.second.step", "50 → 25"),
            ("decay.warmed.left_after_old_half_life", f"{warmed_left:.2f}"),
            ("decay.second_order.left_from_1", f"{D(1) / 2:.2f}"),
            ("decay.second_order.left_from_half", f"{D(2) / 3:.2f}"),
            ("decay.molecules", "20"),
            ("decay.molecules.left_at_end", str(sum(1 for w in waits if w > run_end))),
            ("circuit.display.battery", f"{v:.0f} V"),
            ("circuit.display.ohms", f"{r:,.0f} ohms"),
            ("circuit.display.ohms_doubled", f"{2 * r:,.0f} ohms"),
            ("circuit.display.farads", f"{c * 1000000:.0f} microfarads"),
            ("circuit.display.farads_doubled", f"{2 * c * 1000000:.0f} microfarads"),
            ("circuit.display.gap_left", f"{100 * one_gap / v:.1f}%"),
            ("circuit.display.filled", f"{100 * (v - one_gap) / v:.1f}%"),
            ("circuit.display.one_tau_volts", f"{v - one_gap:.2f} V"),
            ("circuit.display.tau", f"{tau:.0f} s"),
            ("circuit.display.tau_r_doubled", f"{2 * r * c:.0f} s"),
            ("circuit.display.tau_c_doubled", f"{r * 2 * c:.0f} s"),
            ("circuit.display.start_current", f"{v / r * 1000:.2f} mA"),
            ("circuit.display.start_current_r_doubled", f"{v / (2 * r) * 1000:.2f} mA"),
            ("circuit.display.start_current_c_doubled", f"{v / r * 1000:.2f} mA"),
            ("circuit.display.resolution", f"{resolution:.2f} V"),
            ("circuit.display.settle_time", f"{settle:.2f} s"),
            ("circuit.display.final_gap", f"{v * (-window_end / tau).exp():.3f} V"),
            ("circuit.display.window_low", f"{window_low:.1f} V"),
            ("circuit.display.window_start", f"{window_start:.0f} s"),
            ("circuit.display.window_end", f"{window_end:.0f} s"),
        ]
        # The computed strings, against what the narration and the table say.
        exact = dict(display)
        for key, literal in (
            ("decay.interval", "1.0000000000"),
            ("decay.fixed.empty_time", "1.4426950409"),
            ("decay.warmed.left_after_old_half_life", "0.25"),
            ("decay.second_order.left_from_1", "0.50"),
            ("decay.second_order.left_from_half", "0.67"),
            ("decay.molecules.left_at_end", "1"),
            ("circuit.display.ohms", "10,000 ohms"),
            ("circuit.display.ohms_doubled", "20,000 ohms"),
            ("circuit.display.farads", "100 microfarads"),
            ("circuit.display.farads_doubled", "200 microfarads"),
            ("circuit.display.gap_left", "36.8%"),
            ("circuit.display.filled", "63.2%"),
            ("circuit.display.one_tau_volts", "7.59 V"),
            ("circuit.display.tau", "1 s"),
            ("circuit.display.tau_r_doubled", "2 s"),
            ("circuit.display.tau_c_doubled", "2 s"),
            ("circuit.display.start_current", "1.20 mA"),
            ("circuit.display.start_current_r_doubled", "0.60 mA"),
            ("circuit.display.start_current_c_doubled", "1.20 mA"),
            ("circuit.display.settle_time", "5.48 s"),
            ("circuit.display.final_gap", "0.030 V"),
        ):
            assert exact[key] == literal, f"{key}: {exact[key]} is not {literal}"
        return out + display


application_data = application_rows()
assert len(application_data) == len(dict(application_data)) == 454, len(application_data)
for key, value in application_data:
    print(f"{key}\t{value}")


# Chapters 10 to 15, computed without translating the Rust: exact rationals where the model is rational, and 80-digit
# decimals (Machin's pi, Taylor sine and cosine, Decimal exp and ln) elsewhere.
def ending_rows():
    from decimal import Decimal, ROUND_HALF_EVEN, localcontext
    from fractions import Fraction
    from math import comb, floor

    rows = []

    def fixed(x, d):
        if isinstance(x, Fraction):
            scaled = abs(x) * 10**d
            whole = floor(scaled)
            rest = scaled - whole
            if rest == Fraction(1, 2):
                raise ValueError(f"exact tie at {d} places: {x}")
            digits = str(whole + (1 if rest > Fraction(1, 2) else 0)).rjust(d + 1, "0")
            text = (digits[:-d] + "." + digits[-d:]) if d else digits
            return ("-" if x < 0 and set(text) - set("0.") else "") + text
        return format(x.quantize(Decimal(1).scaleb(-d), rounding=ROUND_HALF_EVEN), "f")

    def five(x):
        text = fixed(x, 5)
        return "0.00000" if text == "-0.00000" else text

    with localcontext() as ctx:
        ctx.prec = 80

        def dec(q):
            return Decimal(q.numerator) / Decimal(q.denominator)

        def arctan_inverse(n):
            total, power, k = Decimal(0), Decimal(1) / n, 0
            while abs(power) > Decimal(10) ** -78:
                total += power / (2 * k + 1) * (-1) ** k
                power /= n * n
                k += 1
            return total

        pi = 16 * arctan_inverse(5) - 4 * arctan_inverse(239)

        def cos_sin(x):
            c, s, term, k = Decimal(0), Decimal(0), Decimal(1), 0
            while abs(term) > Decimal(10) ** -78 or k < 8:
                if k % 2 == 0:
                    c += term * (-1) ** (k // 2)
                else:
                    s += term * (-1) ** (k // 2)
                k += 1
                term = term * x / k
            return c, s

        def multiply(a, b):
            return (a[0] * b[0] - a[1] * b[1], a[0] * b[1] + a[1] * b[0])

        def size(z):
            return (z[0] * z[0] + z[1] * z[1]).sqrt()

        # Chapter 10: y' = (1 - t) y with y(0) = 1, so y = exp(t - t^2 / 2).
        for k in range(9):
            t = Fraction(k, 4)
            area = t - t * t / 2
            rows += [
                (f"change.{k}.t", fixed(t, 2)),
                (f"change.{k}.rate", fixed(1 - t, 2)),
                (f"change.{k}.area", fixed(area, 5)),
                (f"change.{k}.amount", fixed(dec(area).exp(), 5)),
            ]
        rows += [("change.peak", fixed(dec(Fraction(1, 2)).exp(), 5)), ("change.back", fixed(Decimal(0).exp(), 5))]
        for n in (4, 16, 64, 1024):
            y = Fraction(1)
            for j in range(n):
                y *= 1 + (1 - Fraction(2 * j, n)) * Fraction(2, n)
            rows.append((f"change.steps.{n}", fixed(y, 5)))
        for j in range(4):
            rows.append((f"change.cell.{j}", fixed(1 + (1 - Fraction(j, 2)) * Fraction(1, 2), 2)))

        # Chapter 11, money: quoted 12 percent from 100; multipliers 6/5 and 9/10; 50 taken out between the years.
        for n in (1, 2, 4, 12, 365):
            rows.append((f"money.credit.{n}", fixed(100 * (1 + Fraction(3, 25) / n) ** n, 2)))
        rows.append(("money.continuous", fixed(100 * dec(Fraction(3, 25)).exp(), 2)))
        good, bad = Fraction(6, 5), Fraction(9, 10)
        rows += [
            ("money.rate.good", fixed(dec(good).ln(), 5)),
            ("money.rate.bad", fixed(dec(bad).ln(), 5)),
            ("money.rate.both", fixed(dec(good * bad).ln(), 5)),
            ("money.both", fixed(good * bad, 2)),
        ]
        for name, first, second in (("good_first", good, bad), ("bad_first", bad, good)):
            after = 100 * first
            left = after - 50
            rows += [
                (f"money.{name}.year1", fixed(after, 2)),
                (f"money.{name}.year2", fixed(after * second, 2)),
                (f"money.{name}.cash.year1", fixed(after, 2)),
                (f"money.{name}.cash.after", fixed(left, 2)),
                (f"money.{name}.cash.end", fixed(left * second, 2)),
                (f"money.{name}.cost", fixed(50 * second, 2)),
            ]

        # Chapter 12, the coin: exact binomial counts over 2^100.
        heads, tails = Fraction(3, 2), Fraction(3, 5)
        chance = lambda h: Fraction(comb(100, h), 2**100)
        final = lambda h: heads**h * tails ** (100 - h)
        above = next(h for h in range(101) if final(h) > 1)
        mean = (heads + tails) / 2
        average = mean**100
        rows += [
            ("coin.mean", fixed(mean, 2)),
            ("coin.pair", fixed(heads * tails, 2)),
            ("coin.heads_step", fixed(dec(heads).ln(), 5)),
            ("coin.tails_step", fixed(dec(tails).ln(), 5)),
            ("coin.pair_step", fixed(dec(heads * tails).ln(), 5)),
            ("coin.average", fixed(average, 2)),
            ("coin.typical", fixed((heads * tails) ** 50, 5)),
            ("coin.above_heads", str(above)),
            ("coin.above_share", fixed(sum(chance(h) for h in range(above, 101)), 5)),
            ("coin.rare_heads", "60"),
            ("coin.rare_share", fixed(sum(chance(h) for h in range(60, 101)), 5)),
            ("coin.rare_average_share", fixed(sum(chance(h) * final(h) for h in range(60, 101)) / average, 5)),
        ]
        rows += [(f"coin.share.{h}", fixed(chance(h), 5)) for h in range(30, 71)]

        # Chapter 12, capacity: the closed form P0 K exp(r t) / (K - P0 + P0 exp(r t)), K = r = 1, P0 = 2/25.
        start = Fraction(2, 25)
        rows += [("cap.start", fixed(start, 2)), ("cap.top", fixed(dec(1 / start).ln(), 5))]
        for k in range(5):
            t = Fraction(k, 2)
            grown = dec(t).exp()
            plain = dec(start) * grown
            limited = plain / (1 - dec(start) + plain)
            rows += [
                (f"cap.{k}.t", fixed(t, 1)),
                (f"cap.{k}.unlimited", fixed(plain, 5)),
                (f"cap.{k}.limited", fixed(limited, 5)),
                (f"cap.{k}.ratio", fixed(plain / limited, 5)),
                (f"cap.{k}.gap", fixed(1 / limited - 1, 5)),
            ]

        # Stage10, review 721: four cells reading the rate at their midpoints (H2), one swap of a tail for a head and
        # five of them (M1), and where one run of heads and tails in turn ends on the ruler, in inputs of E (L3).
        midpoint = Fraction(1)
        for j in range(4):
            midpoint *= 1 + (1 - Fraction(2 * j + 1, 4)) * Fraction(2, 4)
        rows += [
            ("change.midpoint.4", fixed(midpoint, 5)),
            ("coin.swap", fixed(heads / tails, 2)),
            ("coin.five_heads", fixed((heads / tails) ** 5, 5)),
            ("coin.alternating_end", fixed(50 * dec(heads).ln() + 50 * dec(tails).ln(), 5)),
        ]

        # Chapters 13 and 14: powers of i, repeated squaring of (1 + i pi / n), and the series split by parity.
        units = ("1 0", "0 1", "-1 0", "0 -1")
        rows += [(f"turn.quarter.{k}", units[k]) for k in range(4)]
        c, s = cos_sin(pi)
        rows += [
            ("turn.half", f"{five(c)} {five(s)}"),
            ("turn.pi", five(pi)),
            ("turn.full_lap", five(2 * pi)),
        ]
        for n, doublings in ((4, 2), (16, 4), (64, 6), (1024, 10)):
            z = (Decimal(1), pi / n)
            for _ in range(doublings):
                z = multiply(z, z)
            rows += [
                (f"turn.cells.{n}.end", f"{five(z[0])} {five(z[1])}"),
                (f"turn.cells.{n}.abs", five(size(z))),
                (f"turn.cells.{n}.miss", five(size((z[0] + 1, z[1])))),
            ]
        term, sizes = Decimal(1), []
        for k in range(17):
            sizes.append(term)
            term = term * pi / (k + 1)
        for k in range(16):
            rows.append((f"turn.term.{k}", five(sizes[k])))
        for k in range(17):
            re = sum((sizes[j] * (-1) ** (j // 2) for j in range(0, k, 2)), Decimal(0))
            im = sum((sizes[j] * (-1) ** (j // 2) for j in range(1, k, 2)), Decimal(0))
            rows.append((f"turn.partial.{k}", f"{five(re)} {five(im)}"))

        # Chapter 15: the tiny multipliers run backward at x = 2.
        two = Decimal(2).ln()
        rows.append(("bridge.value", five(two)))
        for n in (1, 4, 16, 64, 1024):
            rows.append((f"bridge.root.{n}", five(n * ((two / n).exp() - 1))))

        # Stage10, review 721: the worked quarter turn of 13_1 (H1) and the slide rule's unit against the
        # exponential's unit (H3). The example point is a chosen value; its turn uses this file's own multiply.
        example = (Decimal(3) / 2, Decimal(1))
        turned = multiply((Decimal(0), Decimal(1)), example)
        rows += [
            ("turn.example", f"{five(example[0])} {five(example[1])}"),
            ("turn.example_turned", f"{five(turned[0])} {five(turned[1])}"),
            ("bridge.unit_ratio", five(Decimal(10).ln())),
        ]
    return rows


for key, value in ending_rows():
    print(f"{key}\t{value}")

# The whole output: 1,104 rows from chapters 01 to 07, 454 from chapters 08 and 09 and 213 from chapters 10 to 15
# (206, plus seven stage10 rows), with no key printed twice.
_oracle_sys.stdout = _row_recorder.stream
_printed_keys = [line.split("\t", 1)[0] for line in "".join(_row_recorder.parts).splitlines()]
assert len(_printed_keys) == len(set(_printed_keys)) == 1104 + 454 + 206 + 7, len(_printed_keys)
