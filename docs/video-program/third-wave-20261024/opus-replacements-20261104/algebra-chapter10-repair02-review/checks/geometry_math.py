"""Layout arithmetic and an independent recomputation of the recap's numbers.

Run from the repo root: python3 -I -B <this file>
Reads only. This is arithmetic on the source's coordinates, not a picture of
the candidate. The two pixel measurements below come from baseline frames, which
show before-cyclic.rs, and were read by eye from the 1280x720 frames.
"""
import math
import re
from pathlib import Path

CANDIDATE = Path("docs/video-program/third-wave-20261024/opus-replacements-20261104/algebra-chapter10-picture-repair02/cyclic.rs")
AXIOMS = Path("explainer_corpus/abstract_algebra_expanded/src/axioms.rs")
source = CANDIDATE.read_text(encoding="utf-8")
axioms = AXIOMS.read_text(encoding="utf-8")


def const(name):
    m = re.search(rf"const {name}: [^=]+= ([^;]+);", source)
    if not m:
        raise SystemExit(f"constant not found: {name}")
    return eval(m.group(1).replace("[", "(").replace("]", ")"), {})  # the literal numbers only


CLOCK, RING_R, PANEL_X = const("CLOCK"), const("RING_R"), const("PANEL_X")
TABLE_X, TABLE_MARK_Y = const("TABLE_X"), const("TABLE_MARK_Y")
print(f"source: CLOCK {CLOCK}, RING_R {RING_R}, PANEL_X {PANEL_X}, TABLE_X {TABLE_X}, TABLE_MARK_Y {TABLE_MARK_Y}")

PX = 90.0  # px per unit at 1280x720: frame-0264's ring radius is 171 px for RING_R 1.9.
ORBIT_RIGHT_PX = 933  # frame-0155: right edge of the orbit text.
FORMULA_W_36 = 410 / PX  # frame-0264: the same TeX at 36 pt spans 696 to 1106 px.
FORMULA_H_36 = 75 / PX  # frame-0264: 565 to 640 px.

failures = []


def check(ok, text):
    print(("  ok   " if ok else "  FAIL ") + text)
    if not ok:
        failures.append(text)


print("\nThe box around the successful column (10_3)")
m = re.search(r"let first = frame\(stage, \(TABLE_X\[checks\.len\(\) - 1\], ([\d.]+)\), \(([\d.]+), ([\d.]+)\)", source)
cy, w, h = map(float, m.groups())
x = TABLE_X[2]
left, right, bottom, top = x - w / 2, x + w / 2, cy - h / 2, cy + h / 2
print(f"  box x {left:.3f}..{right:.3f}, y {bottom:.3f}..{top:.3f}")
for stroke in ("(v(x - 0.17, y + 0.01), v(x - 0.05, y - 0.12))", "(v(x - 0.05, y - 0.12), v(x + 0.19, y + 0.16))"):
    if stroke not in axioms:
        raise SystemExit("mark's coordinates moved; reread axioms.rs")
tick_low, tick_high = TABLE_MARK_Y - 0.12, TABLE_MARK_Y + 0.16
tick_left, tick_right = x - 0.17, x + 0.19
check(bottom < tick_low and tick_high < top and left < tick_left and tick_right < right, f"the tick (y {tick_low:.2f}..{tick_high:.2f}) is inside the box")
print(f"  gap below the tick's centreline {tick_low - bottom:.2f}; the old box's bottom edge was at 0.05, crossing the tick (frame-0155)")
check(ORBIT_RIGHT_PX < 640 + PX * left, f"orbit text ends at {ORBIT_RIGHT_PX} px, box starts at {640 + PX * left:.0f} px")
pulsed_left, pulsed_top = x - w * 1.25 / 2, cy + h * 1.25 / 2
check(ORBIT_RIGHT_PX < 640 + PX * pulsed_left, f"and clear at the 1.25 pulse too, box then starts at {640 + PX * pulsed_left:.0f} px")
check(pulsed_top < 2.9, f"pulsed top {pulsed_top:.2f} stays under the title, whose bottom is near y 2.9")
print(f"  gap above the '3' (top near y 1.7) is {top - 1.7:.2f}, below the tick {tick_low - bottom:.2f}: not centred, not a collision")

print("\nThe recap's clock and panel (10_11)")
dial_right = CLOCK[0] + RING_R
label_y = CLOCK[1] + RING_R + 0.3
w32, h32 = FORMULA_W_36 * 32 / 36, FORMULA_H_36 * 32 / 36
print(f"  formula at 32 pt: {w32:.2f} wide, {h32:.2f} tall")
check(dial_right < PANEL_X - w32 / 2, f"dial right edge {dial_right:.2f} clears formula left edge {PANEL_X - w32 / 2:.2f}")
check(PANEL_X + w32 / 2 < 7.11, f"formula right edge {PANEL_X + w32 / 2:.2f} is on screen (half width 7.11)")
order_y, formula_y = 0.5, -0.6
order_half = 0.2
check(formula_y + h32 / 2 < order_y - order_half * 1.25, f"formula top {formula_y + h32 / 2:.2f} clears pulsed ord(8)=3 bottom {order_y - order_half * 1.25:.2f}")

print("\nThe path pulse: Indicate scales about the path's own centre (baseline frame 256 shows this motion)")
hour = lambda h: (CLOCK[0] + RING_R * math.sin(2 * math.pi * h / 12), CLOCK[1] + RING_R * math.cos(2 * math.pi * h / 12))
points = [hour(h) for h in (0, 8, 4)]
xs, ys = [p[0] for p in points], [p[1] for p in points]
centre_y = (max(ys) + min(ys)) / 2
apex = centre_y + 1.25 * (max(ys) - centre_y)
base = centre_y + 1.25 * (min(ys) - centre_y)
print(f"  triangle y {min(ys):.3f}..{max(ys):.3f}; its centre {centre_y:.3f}, the clock's {CLOCK[1]}")
print(f"  at the 1.25 peak: apex y {apex:.3f}, base y {base:.3f}; ring top {CLOCK[1] + RING_R:.3f}, '+8' label centre y {label_y:.3f}")
print(f"  so the apex reaches the label for the middle of the 0.6 s pulse (finding F5)")

print("\nThe numbers on screen, recomputed from the definitions")
N, K = const("N"), const("K")
walk, at = [0], K % N
while at != 0:
    walk.append(at)
    at = (at + K) % N
order = len(walk)
g = math.gcd(N, K)
check(order == 3 and walk == [0, 8, 4], f"walk of +{K} on {N}: {walk}, first return after {order} moves")
check(N // g == order and g == 4, f"n/gcd(n,k) = {N}/{g} = {N // g}")
exponents = const("EXPONENTS")
h_set = set(walk)
members = [m for m in range(exponents[0], exponents[1] + 1) if m % N in h_set]
least = min(m for m in members if m > 0)
check(members == [-8, -4, 0, 4, 8, 12], f"exponents in H on the line: {members}")
check(least == 4 and all(m % least == 0 for m in members), f"smallest positive exponent {least} divides every one")
chunks = [(i * least, (i + 1) * least) for i in range(exponents[0] // least, exponents[1] // least)]
check(len(chunks) == 5 and chunks[0][0] == -8 and chunks[-1][1] == 12, f"{len(chunks)} chunks of {least}, from {chunks[0][0]} to {chunks[-1][1]}")
check({(least * m) % N for m in range(N)} == h_set, f"<g^{least}> on the clock is {sorted(h_set)}")

print("\nThe recap reuses TeX the chapter already sets")
literal = r'r"\operatorname{ord}(8)=3"'
recap = r'r"\operatorname{{ord}}({K})={}", order(N, K)'
check(literal in source and recap in source and K == 8 and order == 3, "ord(8)=3: the recap's format! evaluates to the literal 10_2 already sets")
formula = r'r"\operatorname{{ord}}(k)=\frac{{n}}{{\gcd(n,k)}}=\frac{{{N}}}{{{d}}}={}"'
check(source.count(formula) == 2, "the formula: the same format string as 10_5, set at 32 pt instead of 36")

print()
print("GEOMETRY AND MATH OK" if not failures else f"FAILURES: {len(failures)}")
raise SystemExit(1 if failures else 0)
