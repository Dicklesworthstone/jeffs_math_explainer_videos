"""Plain arithmetic behind the numbers in REPAIR.md and the staged tests' thresholds.

Pure maths only: no scene code, no engine, no stand-in. Nothing here says the Rust
compiles or that any frame looks right. Run from the repo root:

    python3 -I -B <stage>/checks/pure_math.py
"""
import cmath
import math
import sys

STAGE = sys.argv[0].rsplit("/checks/", 1)[0]
LOG = STAGE + "/explainer_corpus/logarithm_mysteries"

# ------------------------------------------------------------------ 12_6's solid
CUBE_C, CUBE_HALF = (-3.0, -0.55), 1.05
sy, cy = math.sin(math.radians(35)), math.cos(math.radians(35))
sp, cp = math.sin(math.radians(20)), math.cos(math.radians(20))
ROD = (0.0, 1.0, 0.0)
ROD_REACH, OTHER_REACH = 2.0, 3.0
CORNER, TOP, TIP = (1.0, 1.0, 1.0), (0.0, 1.0, 0.0), (0.0, ROD_REACH, 0.0)
START_RING_R, END_RING_R = 0.13, 0.2
VIEW = (-sy * cp, sp, cy * cp)


def unit(a):
    n = math.sqrt(sum(x * x for x in a))
    return tuple(x / n for x in a)


def cross(a, b):
    return (a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0])


def dot3(a, b):
    return sum(x * y for x, y in zip(a, b))


def turn(n, angle, p):
    s, c = math.sin(angle), math.cos(angle)
    k = cross(n, p)
    along = dot3(n, p) * (1 - c)
    return tuple(p[i] * c + k[i] * s + n[i] * along for i in range(3))


def project(p):
    x, z = p[0] * cy + p[2] * sy, -p[0] * sy + p[2] * cy
    return (CUBE_C[0] + CUBE_HALF * x, CUBE_C[1] + CUBE_HALF * (p[1] * cp - z * sp))


def hidden(p, pose):
    axis, angle = pose
    q, d = turn(axis, -angle, p), turn(axis, -angle, VIEW)
    lo, hi = 0.0, math.inf
    for qi, di in zip(q, d):
        if abs(di) < 1e-12:
            if abs(qi) > 1:
                return False
        else:
            a, b = (-1 - qi) / di, (1 - qi) / di
            lo, hi = max(lo, min(a, b)), min(hi, max(a, b))
    return hi > lo + 1e-9 and hi > 1e-6


def runs_of(pts, pose):
    runs, cur = [], []
    for a, b in zip(pts, pts[1:]):
        if hidden(tuple((a[i] + b[i]) / 2 for i in range(3)), pose):
            if cur:
                runs.append(cur)
            cur = []
        else:
            cur = cur or [project(a)]
            cur.append(project(b))
    if cur:
        runs.append(cur)
    return runs


def drawn(runs):
    return sum(math.hypot(r[i + 1][0] - r[i][0], r[i + 1][1] - r[i][1]) for r in runs for i in range(len(r) - 1))


def spin_path(axis, rate, p, now):
    laps = abs(rate * now)
    if laps < 1e-6:
        return []
    n = max(math.ceil(laps * 96), 2)
    return [turn(axis, math.tau * rate * now * k / n, p) for k in range(n + 1)]


def axis_points(a, reach):
    n = math.ceil(reach * 40)
    return [tuple(x * reach * (2 * k / n - 1) for x in a) for k in range(n + 1)]


def pin_points(pose):
    a, b = turn(*pose, TOP), turn(*pose, TIP)
    return [tuple(a[i] + (b[i] - a[i]) * k / 24 for i in range(3)) for k in range(25)]


def nearest(c, runs):
    return min((math.hypot(q[0] - c[0], q[1] - c[1]) for r in runs for q in r), default=9.9)


OTHER = (0.0, 1.0, 1.0)
n = unit(OTHER)
print("12_6: spin about (0, 1, 1)")
print("  top lands", tuple(round(x, 9) + 0.0 for x in turn(n, math.pi, TOP)))
print("  corner lands", tuple(round(x, 9) + 0.0 for x in turn(n, math.pi, CORNER)))
start_ring, end_ring = project(CORNER), project(turn(ROD, math.pi, CORNER))
worst = {}
arc_gap, tip_hidden, ring_hidden, top_shown = 0.0, 0, 0, 0.0
pins = []
J = 40
for j in range(1, J + 1):
    t = j / J
    pose = (n, math.pi * t)
    arc = spin_path(n, 0.5, TIP, t)
    arc_gap = max(arc_gap, abs(drawn(runs_of(arc, pose)) - drawn([[project(q) for q in arc]])))
    tip_hidden += hidden(turn(*pose, TIP), pose)
    ring_hidden += hidden(CORNER, pose) or hidden(turn(ROD, math.pi, CORNER), pose)
    top_shown = max(top_shown, drawn(runs_of(spin_path(n, 0.5, TOP, t), pose)))
    pins.append((t, drawn(runs_of(pin_points(pose), pose))))
print(f"  over {J} poses: tip hidden {tip_hidden}, a ring hidden {ring_hidden}")
print(f"  whole tip arc minus shown tip arc, worst: {arc_gap:.3e}; top centre's path shown, worst: {top_shown:.3e}")
print("  pin drawn at t:", ", ".join(f"{t:.3f}:{d:.2f}" for t, d in pins[::4]))
print(f"  pin drawn: least {min(d for _, d in pins):.3f}, at t {min(pins, key=lambda x: x[1])[0]:.3f}; at the end {pins[-1][1]:.3f}")
print("  guards on what is drawn:")
for t in (0.25, 0.5, 0.75, 1.0):
    pose = (n, math.pi * t)
    print(
        f"    t {t}: corner {drawn(runs_of(spin_path(n, 0.5, CORNER, t), pose)):.3f}"
        f"  tip arc {drawn(runs_of(spin_path(n, 0.5, TIP, t), pose)):.3f}"
    )
for t in (0.0, 1.0):
    print(f"  other axis drawn at t {t}: {drawn(runs_of(axis_points(n, OTHER_REACH), (n, math.pi * t))):.3f}")
for name, ring, r in (("start", start_ring, START_RING_R), ("end", end_ring, END_RING_R)):
    near = {"line": 9.9, "pin": 9.9, "tip arc": 9.9, "corner trace": 9.9}
    for j in range(0, 61):
        t = j / 60
        pose = (n, math.pi * t)
        near["line"] = min(near["line"], nearest(ring, runs_of(axis_points(n, OTHER_REACH), pose)))
        near["pin"] = min(near["pin"], nearest(ring, runs_of(pin_points(pose), pose)))
        if t > 0:
            near["tip arc"] = min(near["tip arc"], nearest(ring, runs_of(spin_path(n, 0.5, TIP, t), pose)))
            if name == "end":
                near["corner trace"] = min(near["corner trace"], nearest(ring, runs_of(spin_path(n, 0.5, CORNER, t), pose)))
    print(f"  {name} ring (r {r}): nearest", {k: round(v, 3) for k, v in near.items() if v < 9})
last = (n, math.pi)
print(
    "  corner dot to end ring at the end:",
    round(math.hypot(project(turn(*last, CORNER))[0] - end_ring[0], project(turn(*last, CORNER))[1] - end_ring[1]), 3),
)

# ------------------------------------------------------------------ the dip's veil
VEIL_HALF = (1.8, 2.1)
xs, ys = [], []
for axis, rate, reach in ((ROD, 2.5, ROD_REACH), (n, 0.5, OTHER_REACH)):
    for j in range(0, 121):
        pose = (axis, math.tau * rate * j / 120)
        for c in [(a, b, d) for a in (-1, 1) for b in (-1, 1) for d in (-1, 1)]:
            q = project(turn(*pose, c))
            xs.append(q[0])
            ys.append(q[1])
    for sign in (-1, 1):
        q = project(tuple(sign * reach * x for x in axis))
        xs.append(q[0])
        ys.append(q[1])
for j in range(0, 73):
    a = math.tau * j / 72
    for h in (1, -1):
        q = project((math.sqrt(2) * math.cos(a), h, math.sqrt(2) * math.sin(a)))
        xs.append(q[0])
        ys.append(q[1])
for j in range(0, 121):
    q = project(turn(n, math.pi * j / 120, TIP))
    xs.append(q[0])
    ys.append(q[1])
print("the veil over 12_6's picture")
print(f"  picture x {min(xs):.3f}..{max(xs):.3f}, y {min(ys):.3f}..{max(ys):.3f}")
print(f"  veil x {CUBE_C[0]-VEIL_HALF[0]:.3f}..{CUBE_C[0]+VEIL_HALF[0]:.3f}, y {CUBE_C[1]-VEIL_HALF[1]:.3f}..{CUBE_C[1]+VEIL_HALF[1]:.3f}")
print(f"  rod word's centre y {project((0, ROD_REACH + 0.3, 0))[1]:.3f}")

# ------------------------------------------------------------------ 12_2's fold
DIAL_C, DIAL_R = (2.3, 1.45), 0.95
FLAT_X = 3.55


def folded_at(fold, turned, laps, inset):
    top = (FLAT_X + (DIAL_C[0] - FLAT_X) * fold, DIAL_C[1] + DIAL_R)
    if fold < 1e-4:
        return (top[0] + math.tau * DIAL_R * (laps - 0.5), top[1] - inset)
    r = DIAL_R / fold
    a = math.pi * fold * (1 - turned) - math.tau * fold * laps
    return (top[0] - (r - inset) * math.sin(a), top[1] - r + (r - inset) * math.cos(a))


def dial_at(laps, r):
    a = math.pi / 2 - math.tau * laps
    return (DIAL_C[0] + r * math.cos(a), DIAL_C[1] + r * math.sin(a))


pts = [folded_at(j / 200, 0, k / 72, 0) for j in range(0, 201) for k in range(73)]
pts += [folded_at(1, j / 40, k / 72, 0) for j in range(0, 41) for k in range(73)]
lowest = min(pts, key=lambda q: q[1])
print("12_2's fold, panel B is x 0.35..6.75, y 0.2..2.7")
print(f"  rim x {min(q[0] for q in pts):.3f}..{max(q[0] for q in pts):.3f}, y {lowest[1]:.3f}..{max(q[1] for q in pts):.3f}")
gap = max(math.dist(folded_at(1, 1, k / 40, 0.14), dial_at(k / 40, DIAL_R - 0.14)) for k in range(41))
print(f"  closed and turned, worst distance from the dial's own marks: {gap:.2e}")
print(f"  just past flat vs flat, worst: {max(math.dist(folded_at(2e-4, 0, k / 40, 0.1), folded_at(0, 0, k / 40, 0.1)) for k in range(41)):.2e}")
print(f"  the 1 and the 10 closed: {math.dist(folded_at(1, 1, 0, 0), folded_at(1, 1, 1, 0)):.2e}")

# ------------------------------------------------------------------ 11_4's shared end


def mmul(a, b):
    return [[sum(a[i][k] * b[k][j] for k in range(2)) for j in range(2)] for i in range(2)]


def madd(a, b):
    return [[a[i][j] + b[i][j] for j in range(2)] for i in range(2)]


def mscale(a, c):
    return [[x * c for x in row] for row in a]


def mexp(x):
    out, term = [[1.0, 0.0], [0.0, 1.0]], [[1.0, 0.0], [0.0, 1.0]]
    for k in range(1, 60):
        term = mscale(mmul(term, x), 1 / k)
        out = madd(out, term)
    return out


def apply(a, p):
    return [a[0][0] * p[0] + a[0][1] * p[1], a[1][0] * p[0] + a[1][1] * p[1]]


CENTRE, UNIT = (-3.6, -0.3), 0.7
to_screen = lambda q: (CENTRE[0] + UNIT * q[0], CENTRE[1] + UNIT * q[1])
X = [[0.3, -1.0], [0.8, -0.1]]
E = mexp(X)
u = [1.0, 1.0]
mid_a, mid_b = apply(X, u), apply(E, u)
end_a, end_b = apply(E, mid_a), apply(X, mid_b)
print("11_4's routes, screen points")
for name, q in (("u", u), ("mid A", mid_a), ("mid B", mid_b), ("end A", end_a), ("end B", end_b)):
    print(f"  {name}: {tuple(round(x, 3) for x in to_screen(q))}")
for drop in (0.62, 0.8):
    at = to_screen([end_a[0], end_a[1] - drop])
    print(f"  drop {drop}: word centre {tuple(round(x, 3) for x in at)}; dot bottom {to_screen(end_a)[1] - 0.12:.3f}; lowest route point {min(to_screen(q)[1] for q in (mid_a, mid_b, end_a)):.3f}")


# ------------------------------------------------------------------ the rule's hold
THEOREM = [
    "A real matrix has a real logarithm exactly",
    "when it is invertible and, for every negative",
    "eigenvalue, its Jordan blocks of each size",
    "occur an even number of times.",
]
VERDICTS = [
    "matching pair: real log",
    "no partners: no real log",
    "−2 paired, 3 positive: real log",
    "one size-2 block: no real log",
]
rule, rows = [len(x) for x in THEOREM], [len(x) for x in VERDICTS]
print("the rule's hold at 15 characters a second")
print(f"  rule {rule} = {sum(rule)} chars, {sum(rule)/15:.3f} s; rows {rows} = {sum(rows)} chars, {sum(rows)/15:.3f} s; window {(sum(rule)+sum(rows))/15:.3f} s")

# ------------------------------------------------------------------ the block numbers


def spin_log(c):
    return [[math.log(c), -math.pi], [math.pi, math.log(c)]]


print("the block rows' numbers")
for c in (1.0, 2.0):
    print(f"  e^(ln {c:g} I + pi J) =", [[round(x, 9) + 0.0 for x in row] for row in mexp(spin_log(c))])
worst = 9.9
for a in (-1.0, 0.0, 0.7):
    for k in (0, 1, -2):
        for shear in (0.0, 0.5, 3.0):
            w = math.pi * (2 * k + 1)
            P = [[1.0, shear], [0.0, 1.0]]
            Pinv = [[1.0, -shear], [0.0, 1.0]]
            real_x = mmul(mmul(P, [[a, -w], [w, a]]), Pinv)
            e = mexp(real_x)
            minus = [[-math.exp(a), 0.0], [0.0, -math.exp(a)]]
            assert max(abs(e[i][j] - minus[i][j]) for i in range(2) for j in range(2)) < 1e-6, (a, k, shear, e)
            worst = min(worst, abs(e[0][1]))
jordan = [[-1.0, 1.0], [0.0, -1.0]]
print("  every real X sampled with eigenvalues a +- i pi (2k+1) gives -e^a I, off-diagonal 0; the size-2 block's off-diagonal is", jordan[0][1])
print(f"  block det {jordan[0][0]*jordan[1][1]-jordan[0][1]*jordan[1][0]:g}, trace {jordan[0][0]+jordan[1][1]:g}")
# and its eigenvalue really is a double -1 with one direction only
print("  block's eigenvalues", [cmath.sqrt(0) - 1, -cmath.sqrt(0) - 1])

# ------------------------------------------------------------------ the phrase hooks
HOOKS = {
    "11_2": ["run backwards"],
    "11_5": ["send it along both routes", "the two routes end in different places"],
    "11_6": ["running one after the other"],
    "12_1": ["the keyboard and the slide rule"],
    "12_6": ["also ends here", "and so does two and a half"],
    "12_21": ["a convention broke the same tie", "the honest answer is the whole family"],
}
script = {}
for line in open(LOG + "/script.tsv", encoding="utf-8"):
    parts = line.rstrip("\n").split("\t")
    if len(parts) >= 2:
        script[parts[0]] = parts[1]
print("phrase hooks: exact, once, in spoken order")
for cue, phrases in HOOKS.items():
    text = script[cue]
    at = [text.find(p) for p in phrases]
    assert all(text.count(p) == 1 for p in phrases), cue
    assert at == sorted(at), cue
    print(f"  {cue}: " + ", ".join(f"{p!r} byte {len(text[:a].encode())}" for p, a in zip(phrases, at)))

# ------------------------------------------------------------------ the shared card helper
# logarithm-repair08-10/patches/shared-draw-helpers.patch sizes a card from its widest entry.
EM = {"\u2212": 0.778, "\u00bd": 0.75, "p": 0.556, "b": 0.556, "c": 0.444, "q": 0.528, "1": 0.5, "2": 0.5, "0": 0.5}


def card(size, entries):
    widest = max(sum(EM[c] for c in e) * size * 0.01 for e in entries)
    dx = max(size * 0.011, (widest + size * 0.25 / 30) / 2)
    return dx + max(size * 0.0095, widest / 2 + size * 0.10 / 30)


print("card half widths under the shared helper, and what each must clear")
RIGHT_EDGE = 6.86
for name, at, size, entries, left_clear, right_clear in (
    ("11_1 mirror", 5.4, 32, ["\u22121", "0", "1"], None, RIGHT_EDGE),
    ("11_5 A", 2.2 + 1.45, 30, ["\u22122", "0", "\u2212\u00bd"], 2.72, 2.2 + 2.7),
    ("11_5 X", 2.2 + 3.95, 30, ["p", "b", "c", "q"], 2.2 + 2.7 + 0.521 * 1.03, RIGHT_EDGE),
    ("11_7 -I", 2.2 + 1.45, 30, ["\u22121", "0"], 2.83, None),
):
    half = card(size, entries)
    print(
        f"  {name}: half {half:.3f}, x {at - half:.3f}..{at + half:.3f}"
        + (f"; left gap {at - half - left_clear:.3f}" if left_clear else "")
        + (f"; right gap {right_clear - at - half:.3f}" if right_clear else "")
    )
    assert (not left_clear or at - half > left_clear) and (not right_clear or at + half < right_clear)
tile = card(20, ["\u22121", "1", "0"])
print(f"  11_20 size-2 tile: half {tile:.3f}, under the wide tile's half 0.49")
assert tile < 0.49
