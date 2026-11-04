#!/usr/bin/env python3
"""Mirror of the numbers behind REVIEW.md and the candidate patches. Run with python3 -I -B.

Nothing here reads the render. Part 1 mirrors fmn pin a399d626's Arrow::build for the Arrow::new
calls in chapters 11-15 (crates/fmn-library/src/line.rs, key_dimensions and the end trim), with
each plane's scale taken from the chapter sources. Parts 2-6 mirror the new test assertions.
"""
import cmath
import math

BUFF = 0.25  # MED_SMALL_BUFF, fmn-core constants.rs
MULT = 0.015  # THICKNESS_MULTIPLIER, line.rs
TIP_RATIO, TIP_ANGLE, MAX_TIP, MAX_WIDTH = 5.0, math.pi / 3, 0.5, 0.1


def key_dimensions(length, thickness):
    """line.rs key_dimensions; negative lengths are possible, as there."""
    width = thickness * MULT
    w_ratio = MAX_WIDTH / (width / length)
    if w_ratio < 1.0:
        width *= w_ratio
    tip_width = TIP_RATIO * width
    tip_length = tip_width / (2.0 * math.tan(TIP_ANGLE / 2))
    t_ratio = MAX_TIP / (tip_length / length)
    if t_ratio < 1.0:
        tip_length *= t_ratio
        tip_width *= t_ratio
    return width, tip_width, tip_length


def arrow(asked, thickness=3.0, buff=BUFF):
    """What Arrow::new(a, b) draws for |b - a| = asked: drawn length, and which way the head points."""
    drawn = asked - 2 * buff
    _, _, tip = key_dimensions(asked - buff, thickness)
    shaft_way = 1 if drawn >= 0 else -1
    head_way = shaft_way * (1 if tip >= 0 else -1)
    return drawn, "forward" if head_way > 0 else "BACKWARD"


print("1. Arrow::new as drawn, scene units; buffer 0.25 at each end")
CASES = [
    ("12_1 loop direction, residues.rs:122", 2 * 0.9 * math.sin(0.175) * 4.6 / 3.2, 5.0),
    ("13 contour along, real_integral.rs:203", 0.6 * 5.0 / 4.8, 4.0),
    ("13 contour back, real_integral.rs:207", 2 * 2 * math.sin(0.15) * 5.0 / 4.8, 4.0),
    ("13_8 p1, real_integral.rs:529", 1.0 * 5.0 / 4.8, 4.0),
    ("13_8 p2, real_integral.rs:530", 1.0 * 5.0 / 4.8, 4.0),
    ("13_8 p1+p2, real_integral.rs:536", math.sqrt(2) * 5.0 / 4.8, 5.0),
    ("14_8 hop, continuing.rs:602", 1.2 * 7.5 / 7.8, 3.0),
    ("15 box 1 carry, closing.rs:95", math.hypot(0.6, 0.25) * 2.3 / 2.6, 3.0),
    ("15 box 3 along, closing.rs:117", 2 * math.sin(0.2) * 2.3 / 2.6, 3.0),
]
for name, asked, thick in CASES:
    drawn, way = arrow(asked, thick)
    print(f"  {name}: asked {asked:.3f}, drawn {abs(drawn):.3f}, head {way}")

print("   12_1 chains, residues.rs:137 through draw.rs:967, 24 steps, radius 0.9")
STEPS, LOOP_R = 24, 0.9
TERMS = [("1/z^2", lambda z: 1 / (z * z)), ("1/z", lambda z: 1 / z), ("h(z)", lambda z: 1 + z / 2)]


def chain(f):
    """model::loop_products: the value at each step's midpoint times the step."""
    pts = [0j]
    zs = [cmath.rect(LOOP_R, 2 * math.pi * k / STEPS) for k in range(STEPS + 1)]
    for k, (a, b) in enumerate(zip(zs, zs[1:])):
        pts.append(pts[-1] + f(cmath.rect(LOOP_R, 2 * math.pi * (k + 0.5) / STEPS)) * (b - a))
    return pts


for name, f in TERMS:
    pts = chain(f)
    xs, ys = [p.real for p in pts], [p.imag for p in pts]
    scale = min(2.0 / max(max(xs) - min(xs), 1e-9), 2.4 / max(max(ys) - min(ys), 1e-9), 1.6)
    lens = [abs(b - a) * scale for a, b in zip(pts, pts[1:])]
    ways = [arrow(x)[1] for x in lens]
    print(f"  {name}: scale {scale:.3f}, asked {min(lens):.3f}-{max(lens):.3f}, "
          f"forward {ways.count('forward')}, BACKWARD {ways.count('BACKWARD')}")
    if name == "h(z)":
        for k in (0, 6, 12, 18):
            d = pts[k + 1] - pts[k]
            print(f"    step {k}: true direction {math.degrees(cmath.phase(d)):+.0f} deg, "
                  f"asked {lens[k]:.3f}, head {ways[k]}")

print("2. 11_4 walk: |z| = 1.35 from -1.35 anticlockwise, stopping 0.03 short of the pole")
A, B, NEAR = 0.45, 1.35, 0.03
END = 2 * math.asin(NEAR / (2 * B))


def gap(s):
    return math.pi * (END / math.pi) ** s


def walk(s):
    return cmath.rect(B, -gap(s))


def f(z):
    return (z - A) / (z - B)


for k in range(65):
    s = k / 64
    w = f(walk(s))
    assert abs(w.real - 2 / 3) < 1e-9, s
    assert w.imag >= -1e-9, s
assert abs(walk(0.0) - (-B)) < 1e-12
assert abs(abs(walk(1.0) - B) - NEAR) < 1e-12
TOP = 1.5
K = (B - A) / (2 * B)
EXIT_GAP = 2 * math.atan(K / TOP)
EXIT = math.log(math.pi / EXIT_GAP) / math.log(math.pi / END)
assert 0 < EXIT < 1 and abs(f(walk(EXIT)).imag - TOP) < 1e-9
print(f"  Re f = 2/3 at 65 samples; leaves the window (Im = {TOP}) at s = {EXIT:.4f}; "
      f"|f| at the stop = {abs(f(walk(1.0))):.4f}, printed {abs(f(walk(1.0))):.1f}")
print("  |f| along the walk:", " ".join(f"{abs(f(walk(k / 8))):.2f}" for k in range(9)))

print("3. 11_7 lengths on |z| = 3")
RING = [cmath.rect(3.0, 2 * math.pi * k / 720) for k in range(720)]
walker = max(abs(z**5) for z in RING)
leash = [abs(-3 * z + 1) for z in RING]
print(f"  walker {walker:.6f}; leash {min(leash):.6f} to {max(leash):.6f}; ratio {max(leash) / walker:.4f}")
squash = lambda m: 1.35 * m / (m + 200.0)
PX = 5.0 / 3.0 * 90.0
print(f"  on screen at 720p: walker {squash(243.0) * PX:.1f} px from 0, radial leash at most "
      f"{(squash(253.0) - squash(243.0)) * PX:.1f} px, sideways at most {10 / 243 * squash(243.0) * PX:.1f} px; "
      f"dot radii {0.09 * 90:.1f} and {0.08 * 90:.1f} px")
BAR = 3.0 / 243.0
print(f"  bars: 243 -> {243 * BAR:.3f}, 10 -> {10 * BAR:.3f} scene units")

print("4. 14_5 bars")
SUMS = [sum(2**k for k in range(n + 1)) for n in range(11)]
print("  partial sums", SUMS)
for top in (40.0, 2200.0):
    print(f"  chart top {top:g}: heights at 3.6 scene units:",
          " ".join(f"{min(s, top) / top * 3.6:.3f}" for s in SUMS))

print("5. 14_7 rim")


def level(k):
    if k == 0:
        return [0.0]
    n = 1 << k
    return [2 * math.pi * j / n for j in range(1, n, 2)]


def lacunary(z):
    total, w = 0j, z
    for _ in range(64):
        total += w
        w = w * w
    return total


seen = []
for k in range(7):
    angles = level(k)
    seen += angles
    assert len(angles) == (1 if k == 0 else 1 << (k - 1))
    for a in angles:
        zeta = cmath.rect(1.0, a)
        assert abs(zeta ** (1 << k) - 1) < 1e-9
assert len(seen) == 64
smallest = math.inf
for a in seen:
    zeta = cmath.rect(1.0, a)
    near, nearer = abs(lacunary(0.999 * zeta)), abs(lacunary(0.999999999 * zeta))
    smallest = min(smallest, nearer - near)
    assert nearer > near + 10.0, a
print(f"  64 ticks in 7 levels; |sum| grows from r = 0.999 to 0.999999999 by at least {smallest:.3f} at every tick")

print("6. 15 labels, closing plane units")
A3, R3 = 3.0, math.sqrt(10.0)
SCALE = 7.6 / 8.0
for name, cx, cy, em in (("-0.16", -0.16 + 0.56, -0.33, 2.556), ("6.16", 6.16 - 0.46, -0.33, 1.778)):
    half = em * 22 * 0.01 / SCALE / 2
    left, right = cx - half, cx + half
    xc = A3 - math.sqrt(R3**2 - cy**2), A3 + math.sqrt(R3**2 - cy**2)
    print(f"  {name}: spans {left:.3f} to {right:.3f} at y {cy}; big circle at {xc[0]:.3f}, {xc[1]:.3f}; "
          f"axis 0; old ring at {math.sqrt(1 - cy**2):.3f}")
    assert not left < xc[0] < right and not left < xc[1] < right and not left < 0 < right
    assert not left < math.sqrt(1 - cy**2) < right
for name, x, y in (("i", 0.42, 1.28), ("-i", 0.5, -1.28)):
    assert math.hypot(x, y) > 1.2
    print(f"  {name} at ({x}, {y}): {math.hypot(x, y) - 1:.3f} outside the radius 1 ring")
print("   box 4 arc: pole ring outer radius 0.085 + 0.025 scene at 0.885 scene per unit")
for r in (1.1, 1.22):
    gap_units = r - 1.0
    print(f"  arc radius {r}: {gap_units * 2.3 / 2.6:.3f} scene from pole centre to arc")

print("7. 13_8")
P1, P2 = cmath.rect(1, math.pi / 4), cmath.rect(1, 3 * math.pi / 4)
print(f"  p1 + p2 = {P1 + P2:.6f}; -(p1 + p2)/4 = {-(P1 + P2) / 4:.6f}; "
      f"residues add: {1 / (4 * P1**3) + 1 / (4 * P2**3):.6f}")
print(f"  2 pi i x sum = {2j * math.pi * (-(P1 + P2) / 4):.6f}; pi/sqrt 2 = {math.pi / math.sqrt(2):.6f}")
