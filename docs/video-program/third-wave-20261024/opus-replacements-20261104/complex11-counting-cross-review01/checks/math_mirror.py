"""An independent mirror of what 11_7's turn claims, in plain Python, written from the mathematics.

It takes the numbers it can from the staged source and model by regex, and computes everything else
itself; it imports none of WhiteSwan's checks. It proves nothing about pixels: the colours here are
the model's arithmetic at cell centres, not a frame.

Usage: python3 -I -B math_mirror.py COUNTING_RS MODEL_RS
Exit 0 when the staged numbers hold and every planted fault is caught.
"""

import cmath
import math
import re
import sys

source = open(sys.argv[1], encoding="utf8").read()
model = open(sys.argv[2], encoding="utf8").read()


def number(pattern, text=source):
    found = re.search(pattern, text)
    if not found:
        raise SystemExit(f"not found: {pattern}")
    return float(found.group(1))


TURN_STEPS = int(number(r"const TURN_STEPS: f64 = ([\d.]+);"))
SQUASH = number(r"const ROOT_WALK_SQUASH: f64 = ([\d.]+);")
OUT_R = number(r"pub const OUT_R: f64 = ([\d.]+);")
WALK_SAMPLES = 720  # draw.rs:741, unchanged by this work
lead = re.search(r"const LEADING: C = C::new\(([-\d.]+), ([-\d.]+)\);", source)
LEADING = complex(float(lead.group(1)), float(lead.group(2)))
PHASE_LINES = int(number(r"pub const PHASE_LINES: u32 = (\d+);", model))
BAND_OFFSET = number(r"pub const BAND_OFFSET: f64 = ([\d.]+);", model)
WHEEL = [tuple(int(c, 16) for c in row) for row in re.findall(r"\[0x(..), 0x(..), 0x(..)\]", model)][:12]
STEP = math.tau / PHASE_LINES
CELLS = (160, 160)  # draw.rs: 5.6 / 0.035 on the input plane, under the film's current plate
WINDOW = (-2.0, 2.0)
OUTPUT_WINDOW = 1.5
NAME_AT, NAME_HALF, OUT_UNIT = complex(-0.12, -0.14), complex(0.06, 0.08), 5.0 / 3.0

results = []


def check(name, ok, detail=""):
    results.append((name, bool(ok)))
    print(("PASS " if ok else "FAIL ") + name + (f": {detail}" if detail else ""))


def p(z):
    return z**5 - 3 * z + 1


def printed(z):
    """The row on screen, f(z) = 2iz⁵ − 6iz + 2i, as the screen writes it."""
    return 2j * z**5 - 6j * z + 2j


def turned_by(t, leading=None):
    leading = LEADING if leading is None else leading
    if leading == 0:
        return 0j
    return cmath.rect(abs(leading) ** t, cmath.phase(leading) * t)


def squash(w, s=SQUASH, r=OUT_R):
    m = abs(w)
    return w if m == 0 else w * (r / (m + s))


def winding(values):
    total = 0.0
    for a, b in zip(values, values[1:]):
        if a == 0 or b == 0:
            return None
        total += cmath.phase(b / a)
    return round(total / math.tau)


def loop(r, n, centre=0j):
    return [centre + cmath.rect(r, math.tau * j / n) for j in range(n + 1)]


def box(half, n):
    """The window's edge, counterclockwise, n points to a side."""
    s = [complex(-half + 2 * half * k / n, -half) for k in range(n)]
    s += [complex(half, -half + 2 * half * k / n) for k in range(n)]
    s += [complex(half - 2 * half * k / n, half) for k in range(n)]
    s += [complex(-half, half - 2 * half * k / n) for k in range(n)]
    return s + s[:1]


def roots_of_p():
    found = []
    for k in range(40):
        z = cmath.rect(1.6, math.tau * k / 40 + 0.1)
        for _ in range(60):
            z -= p(z) / (5 * z**4 - 3)
        if abs(p(z)) < 1e-12 and all(abs(z - r) > 1e-6 for r in found):
            found.append(z)
    return found


def shade(w):
    size = abs(w)
    if not (size > 0 and math.isfinite(size)):
        return None
    size_band = (math.log(size) / STEP + BAND_OFFSET) % 1.0
    phase_band = ((cmath.phase(w) % math.tau) / STEP + BAND_OFFSET) % 1.0
    return (0.58 + 0.42 * size_band) * (0.80 + 0.20 * phase_band)


def wheel(angle):
    t = (angle % math.tau) / math.tau * len(WHEEL)
    k = min(int(t), len(WHEEL) - 1)
    f = t - k
    a, b = WHEEL[k], WHEEL[(k + 1) % len(WHEEL)]
    return tuple((a[c] + (b[c] - a[c]) * f) / 255.0 for c in range(3))


def colour(w):
    s = shade(w)
    if s is None:
        return None
    return tuple(c * s for c in wheel(cmath.phase(w)))


def field(by):
    (cols, rows), (lo, hi) = CELLS, WINDOW
    out = []
    for r in range(rows):
        for c in range(cols):
            z = complex(lo + (hi - lo) * (c + 0.5) / cols, hi - (hi - lo) * (r + 0.5) / rows)
            out.append(colour(by * p(z)))
    return out


def by_of(k):
    return turned_by(k / TURN_STEPS)


def invariants(by_at, f=p, plant=lambda w, by: by * w, radius=OUT_R):
    """Every step's roots, count of zeros, turns and curve. Returns the problems it finds."""
    problems = []
    roots = roots_of_p()
    around_window = box(2.0, 1200)
    unit, big = loop(1.0, 2880), loop(3.0, 5760)
    for k in range(TURN_STEPS + 1):
        by = by_at(k)
        g = lambda z, by=by: plant(f(z), by)  # noqa: E731
        if abs(by) == 0:
            problems.append(f"step {k}: the factor is 0")
            continue
        for r in roots:
            if abs(g(r)) > 1e-9:
                problems.append(f"step {k}: root {r:.4f} moved, value {abs(g(r)):.3g}")
        if winding([g(z) for z in around_window]) != 5:
            problems.append(f"step {k}: zeros in the window are not 5")
        if winding([g(z) for z in unit]) != 1:
            problems.append(f"step {k}: turns on |z| = 1 are not 1")
        if winding([g(z) for z in big]) != 5:
            problems.append(f"step {k}: turns on |z| = 3 are not 5")
        curve = [squash(g(z), r=radius) for z in loop(1.0, WALK_SAMPLES)]
        if winding(curve) != 1:
            problems.append(f"step {k}: the squashed curve does not go round 0 once")
        if min(abs(c) for c in curve) <= abs(NAME_AT) + abs(NAME_HALF) / OUT_UNIT + 0.1:
            problems.append(f"step {k}: the curve reaches the name of 0")
        if max(max(abs(c.real), abs(c.imag)) for c in curve) > OUTPUT_WINDOW:
            problems.append(f"step {k}: the curve leaves the window")
    return problems


# ------------------------------------------------------------------ the example on screen
points = [complex(0.3, -0.2) + cmath.rect(1.7, math.tau * k / 97) for k in range(97)]
check("the printed row is 2i times the quintic", all(abs(printed(z) - 2j * p(z)) < 1e-11 for z in points))
check("the printed row divided by 2i is the quintic", all(abs(printed(z) / 2j - p(z)) < 1e-11 for z in points))
check("the source's leading coefficient is the printed one", LEADING == 2j, str(LEADING))
for row, want in (
    ("BEFORE_DIVIDING_TEX", 'r"f(z) = 2iz^5 - 6iz + 2{i}"'),
    ("AFTER_DIVIDING_TEX", r'r"p(z) = f(z) \div 2i = z^5 - 3z + 1"'),
):
    check(f"{row} prints what this mirror computed", f"pub const {row}: &str = {want};" in source)

# ------------------------------------------------------------------ the factor
check("the factor starts at exactly 1", turned_by(0.0) == 1 + 0j)
check("the factor ends at the coefficient", abs(turned_by(1.0) - LEADING) < 1e-15)
sizes = [abs(by_of(k)) for k in range(TURN_STEPS + 1)]
check("the factor never nears 0", min(sizes) >= 1 - 1e-12, f"smallest size {min(sizes):.6f}")
angle = math.degrees(cmath.phase(by_of(1) / by_of(0)))
print(f"each step turns {angle:.4f} degrees and grows {by_of(1).real * 0 + abs(by_of(1)):.5f} times")
check("each step is under 2 degrees", angle < 2.0)

# ------------------------------------------------------------------ roots, counts, curve: all 49 steps
problems = invariants(by_of)
check(f"all {TURN_STEPS + 1} steps keep 5 roots at the marks, 5 zeros, 1 and 5 turns, curve inside and off the 0", not problems, "; ".join(problems[:4]))
roots = roots_of_p()
check("the mirror found 5 roots", len(roots) == 5, ", ".join(f"{r.real:.4f}{r.imag:+.4f}i" for r in sorted(roots, key=lambda z: z.real)))
for k in (0, TURN_STEPS // 2, TURN_STEPS):
    curve = [squash(by_of(k) * p(z)) for z in loop(1.0, WALK_SAMPLES)]
    radii = [abs(c) for c in curve]
    print(f"curve at step {k}: radius {min(radii):.4f} to {max(radii):.4f}, window half {OUTPUT_WINDOW}")

# colours meet at each mark at every step: the phase goes once round on a tiny circle round it
meet = []
for k in range(TURN_STEPS + 1):
    for r in roots:
        turns = winding([by_of(k) * p(z) for z in loop(0.05, 720, r)])
        if turns != 1:
            meet.append((k, r, turns))
check("at every step all the colours meet at each mark, once", not meet, str(meet[:3]))

# ------------------------------------------------------------------ the colours
base = field(1 + 0j)
landed = field(by_of(0))
check("the field at t = 0 is the painter's quintic, cell for cell", base == landed)
check("no cell of the field is unpainted", all(c is not None for c in base))
out_there = field(by_of(TURN_STEPS))
moved = sum(1 for a, b in zip(base, out_there) if a != b)
print(f"cells that differ between t = 0 and t = 1: {moved} of {len(base)}")
check("the whole picture changes at t = 1", moved == len(base))
exact = field(2j)
near = max(max(abs(x - y) for x, y in zip(a, b)) for a, b in zip(out_there, exact))
print(f"t = 1 against the picture of f itself: worst channel difference {near:.2e}")
check("the field at t = 1 is the picture of f", near < 1e-3)

# a phase edge on the real axis: which steps put a phase line along it, for the eye only
on_axis = []
for k in range(TURN_STEPS + 1):
    edge = (math.degrees(cmath.phase(by_of(k))) / (360.0 / PHASE_LINES) + BAND_OFFSET) % 1.0
    if min(edge, 1.0 - edge) < 1e-9:
        on_axis.append(k)
print(f"steps with a phase line lying along the real axis: {on_axis} (of {TURN_STEPS}, each way)")

# ------------------------------------------------------------------ 11_8's plate, in the same terms
plate, reach, half_rim = 0.105 / 0.7, 8 * math.tau / 64, 0.0175 / 0.7
samples = [cmath.rect(1.0, math.tau * k / 3600) for k in range(3600)]
staged_at = re.search(r"const ZERO_NAME_AT: \(f64, f64\) = \(([-\d.]+), ([-\d.]+)\);", source)
staged_at = complex(float(staged_at.group(1)), float(staged_at.group(2)))


def sweep_of(at):
    """Shares of the sweep: the plate breaks the line, breaks the rim, or the digit is in the disk."""

    def line_touches(c):
        along = max(0.0, min(1.0, at.real * c.real + at.imag * c.imag))
        return abs(at - c * along) < plate and abs(at) > plate

    def rim_touches(c):
        return abs(abs(at - c) - reach) < plate + half_rim

    def inside(c):
        return abs(at - c) < reach

    n = len(samples)
    return tuple(sum(map(t, samples)) / n for t in (line_touches, rim_touches, inside))


for label, place in (("as staged", complex(-0.25, -0.3)), ("with the digit on zero (C3)", 0j)):
    line, rim, inside = sweep_of(place)
    print(f"11_8 sweep, digit at {place}, {label}: the plate breaks the line for {line:.3f} of the turn, the rim for {rim:.3f}; the digit is inside the disk for {inside:.3f}")
    print(f"   at the sweep's floor of 1.6 s: line {line * 1.6:.2f} s, rim {rim * 1.6:.2f} s, inside {inside * 1.6:.2f} s; at 2.4 s: inside {inside * 2.4:.2f} s")
line, rim, inside = sweep_of(0j)
check("with the digit on zero, the disk never holds it and its rim never touches the plate", rim == 0 and inside == 0)
gap = min(abs(c) for c in samples) - reach - plate - half_rim
print(f"   clearance from the plate to the rim's edge at the closest bearing: {gap:.4f} plane units, {gap * 0.7:.4f} scene units, {gap * 0.7 * 270:.1f} px at 4K")
print(f"   the file read here has the digit at {staged_at}")
line, rim, inside = sweep_of(complex(-0.25, -0.3))
check("as staged the digit does go inside the disk (what C3 answers)", inside > 0.2, f"{inside:.3f} of the turn")

# ------------------------------------------------------------------ planted faults must be caught
plants = {
    "the factor reaches 0 at the end": lambda: invariants(lambda k: turned_by(k / TURN_STEPS, 2j) * (1 - k / TURN_STEPS)),
    "the turn adds a constant, so the roots move": lambda: invariants(by_of, plant=lambda w, by: by * w + 0.3),
    "the quintic has the wrong term": lambda: invariants(by_of, f=lambda z: z**5 - 3 * z + 3),
    "a turn that is a reflection": lambda: invariants(by_of, plant=lambda w, by: by * w.conjugate()),
    "a squash radius that carries the curve off the window": lambda: invariants(by_of, radius=2.6),
}
for name, run in plants.items():
    found = run()
    check(f"planted, caught: {name}", bool(found), found[0] if found else "NOT CAUGHT")
check("planted, caught: a printed row with the wrong sign", any(abs(2j * z**5 + 6j * z + 2j - 2j * p(z)) > 1e-6 for z in points))

failed = [name for name, ok in results if not ok]
print(f"{len(results) - len(failed)} of {len(results)} passed")
sys.exit(1 if failed else 0)
