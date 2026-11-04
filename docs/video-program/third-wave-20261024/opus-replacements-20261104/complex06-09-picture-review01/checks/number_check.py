"""Recompute what chapters 06-09 show, independently of the film's own model code.

Run from the repo root with: python3 -I -B <this file>
Standard library only. Reads script.tsv and preview02's cue files; writes nothing.

Each `shown` string was read off a preview02 frame. A PASS means the value computed here,
formatted to the same places, equals that string. The word-share section is an estimate on
the scratch voice and proves nothing about Oliver's timing.
"""
import cmath
import math
import re

SCRIPT = "explainer_corpus/complex_analysis_visual/script.tsv"
RENDER = "narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-combined-picture-preview02/render/"
CHAPTERS = {"06_one_derivative": 9, "07_one_point": 6, "08_loop_sums": 5, "09_boundary": 9}

passed = failed = 0


def check(name, ok, detail=""):
    global passed, failed
    if ok:
        passed += 1
    else:
        failed += 1
    print(("PASS " if ok else "FAIL ") + name + (f"  [{detail}]" if detail else ""))


def shown(name, value, text, fmt):
    got = fmt.format(value).replace("-", "−")
    check(f"{name}: shown {text}", got == text, f"computed {value:.10g}, formatted {got}")


def bump(z):
    return 1 / (1 + z * z)


def samples(r, n, a=0.0):
    return [a + r * cmath.exp(2j * math.pi * k / n) for k in range(n)]


def mean(values):
    values = list(values)
    return sum(values) / len(values)


print("== 09: sampled means of the bump, summed directly")
m8 = mean(bump(z) for z in samples(0.5, 8))
m16 = mean(bump(z) for z in samples(0.5, 16))
m2 = mean(bump(z) for z in samples(2.0, 16))
shown("r 1/2, 8 samples", m8.real, "1.0039", "{:.4f}")
shown("r 1/2, 16 samples", m16.real, "1.000015", "{:.6f}")
shown("r 2, 16 samples", m2.real, "−0.000015", "{:.6f}")
check("means are real", abs(m8.imag) + abs(m16.imag) + abs(m2.imag) < 1e-12)
check("voice: within half a percent of one", abs(m8.real - 1) < 0.005, f"{abs(m8.real - 1):.5f}")
check("voice: about fifteen millionths", 14e-6 < abs(m16.real - 1) < 16e-6, f"{abs(m16.real - 1):.3e}")
check("closed form 8 samples 1/(1-r^8)", abs(m8.real - 1 / (1 - 0.5**8)) < 1e-12)
check("closed form 16 samples 1/(1-r^16)", abs(m16.real - 1 / (1 - 0.5**16)) < 1e-12)

print("== 09: residues of bump/z at the poles (the card's minus one half)")
for pole in (1j, -1j):
    residue = 1 / (2 * pole * pole)  # 1/((z - p')(z)) at z = p, with z^2 + 1 = (z - p)(z + p)
    shown(f"pole {pole}", residue.real, "−0.5", "{:.1f}")
check("1 - 0.5 - 0.5 = 0", 1 - 0.5 - 0.5 == 0)

print("== 09: coefficients from the sampled formula at r = 1/2, 16 samples")
for n, text in [(0, "1.000"), (1, "0.000"), (2, "−1.000"), (3, "0.000"), (4, "1.000")]:
    c = mean(bump(z) * cmath.exp(-1j * n * 2 * math.pi * k / 16) for k, z in enumerate(samples(0.5, 16))) / 0.5**n
    value = c.real if abs(c.real) >= 5e-4 else 0.0
    shown(f"c_{n}", value, text, "{:.3f}")
    if n == 2:
        print(f"     c_2 unrounded {c.real:.7f}; mean of the turned-twice values {c.real * 0.25:.7f}")
check("panel: mean turned twice sits at -0.25", abs(mean(bump(z) * cmath.exp(-2j * 2 * math.pi * k / 16) for k, z in enumerate(samples(0.5, 16))).real + 0.25) < 1e-4)
check("panel: turned-twice values take 8 distinct places", len({complex(round((bump(z) * cmath.exp(-2j * 2 * math.pi * k / 16)).real, 9), round((bump(z) * cmath.exp(-2j * 2 * math.pi * k / 16)).imag, 9)) for k, z in enumerate(samples(0.5, 16))}) == 8)
check("panel: turned-once values take 16 distinct places", len({complex(round((bump(z) * cmath.exp(-1j * 2 * math.pi * k / 16)).real, 9), round((bump(z) * cmath.exp(-1j * 2 * math.pi * k / 16)).imag, 9)) for k, z in enumerate(samples(0.5, 16))}) == 16)

print("== 09: radius and the recipe recentred at 1")
radius = abs(1j - 1)
shown("radius from 1 to i", radius, "1.414", "{:.3f}")
shown("left end", 1 - radius, "−0.41", "{:.2f}")
shown("right end", 1 + radius, "2.41", "{:.2f}")


def recipe_at_one(x, terms):
    """Own route: (1 + (1+w)^2) sum c_n w^n = 1 gives 2c_n + 2c_(n-1) + c_(n-2) = 0, c_0 = 1/2."""
    w = x - 1
    c = [0.5, -0.5]
    while len(c) < terms:
        c.append(-(2 * c[-1] + c[-2]) / 2)
    return sum(v * w**n for n, v in enumerate(c[:terms]))


shown("60 terms at 2", recipe_at_one(2.0, 60), "0.200000", "{:.6f}")
shown("60 terms at 2.6", recipe_at_one(2.6, 60), "212", "{:.0f}")
print(f"     bump(2.6) = {bump(2.6):.5f}; so 212 is the recipe failing, as the row says")
check("the 2.6 row lies outside the range", 2.6 > 1 + radius)
check("the 2 row lies inside the range", 2.0 < 1 + radius)

print("== 06: z^2 at 1 + i")
z0 = 1 + 1j
d = 2 * z0
shown("|f'| stretch", abs(d), "2.83", "{:.2f}")
shown("turn degrees", math.degrees(cmath.phase(d)), "45", "{:.0f}")
check("grid [2 -2; 2 2]", (d.real, -d.imag, d.imag, d.real) == (2.0, -2.0, 2.0, 2.0))


def gap_ratio(side):
    """f(z0 + h) - f(z0) - f'(z0) h = h^2. The worst h on the square is a corner, |h|^2 = side^2 / 2."""
    return (side**2 / 2) / (abs(d) * side)


shown("gap at side 0.5", gap_ratio(0.5), "0.088", "{:.3f}")
shown("gap at side 0.05", gap_ratio(0.05), "0.0088", "{:.4f}")
check("reflection: right 1, up -1", (1 + 0j) / 1 == 1 and ((1j) .conjugate()) / 1j == -1)
check("doubling: right 1, up 2", (2 * 1j) / 1j == 2)
print(f"     plane scales: left 4.6/2.4 = {4.6 / 2.4:.3f}, right 4.6/3.2 = {4.6 / 3.2:.3f}; a right unit is {2.4 / 3.2:.2f} of a left unit,")
print(f"     so a stretch of 2.83 draws at {abs(d) * 2.4 / 3.2:.2f} times the blue arrow's length. Frames measured about 2.1.")

print("== 07")
sq = (-0.7 + 0.6j) ** 2
check("(-0.7+0.6i)^2 = 0.13-0.84i", abs(sq - (0.13 - 0.84j)) < 1e-12, str(sq))
check("|h|^2/h = conj(h)", abs(abs(0.3 - 0.2j) ** 2 / (0.3 - 0.2j) - (0.3 + 0.2j)) < 1e-12)
h = 1e-6
check("|z|^2 at 1: right tends to 2", abs((abs(1 + h) ** 2 - 1) / h - 2) < 1e-5)
check("|z|^2 at 1: up tends to 0", abs((abs(1 + 1j * h) ** 2 - 1) / (1j * h)) < 1e-5)

print("== 08: the chain, 24 steps round the unit circle, sampled at each arc's midpoint")
N = 24
points = [cmath.exp(2j * math.pi * k / N) for k in range(N + 1)]
mids = [cmath.exp(2j * math.pi * (k + 0.5) / N) for k in range(N)]
steps = [points[k + 1] - points[k] for k in range(N)]
shown("one step", abs(steps[0]), "0.26", "{:.2f}")


def chain(power):
    return [mids[k] ** power * steps[k] for k in range(N)]


laps = chain(1)
running = [abs(sum(laps[: n + 1])) for n in range(N)]
for n, text in [(1, "0.26"), (2, "0.50"), (4, "0.87"), (6, "1.01"), (7, "0.97"), (9, "0.71"), (11, "0.26"), (12, "0.00")]:
    shown(f"f = z, distance after {n}", running[n - 1], text, "{:.2f}")
for n, text in [(13, "0.26"), (15, "0.71"), (17, "0.97"), (18, "1.01"), (20, "0.87"), (22, "0.50"), (23, "0.26"), (24, "0.00")]:
    shown(f"f = z, distance after {n}", running[n - 1], text, "{:.2f}")
shown("|total| for f = z", abs(sum(laps)), "0.000000", "{:.6f}")
ups = chain(-1)
check("f = 1/z: every product points straight up", all(abs(u.real) < 1e-12 and u.imag > 0 for u in ups))
for n, text in [(1, "0.26"), (6, "1.57"), (11, "2.87"), (16, "4.18"), (21, "5.48"), (24, "6.27")]:
    shown(f"f = 1/z, column after {n}", abs(sum(ups[:n])), text, "{:.2f}")
shown("limit 2 pi", 2 * math.pi, "6.28", "{:.2f}")
scaled = [(2 * mids[k]) ** -1 * (2 * steps[k]) for k in range(N)]
check("a bigger circle gives the same column", all(abs(scaled[k] - ups[k]) < 1e-12 for k in range(N)))
for power in (-3, -2, 0, 1, 2):
    products = chain(power)
    ring = abs(products[0]) / (2 * math.sin(abs(power + 1) * math.radians(7.5)))
    print(f"     z^{power}: total {abs(sum(products)):.1e}, ring radius {ring:.3f}")
    check(f"z^{power} closes up", abs(sum(products)) < 1e-12)

print("== Desk geometry behind proposed corrections; none of this was rendered")
UNIT = 100.8  # native px per plate unit in chapter 09, plate centre at (343, 378)
cx, cy = 343 + 1.05 * UNIT, 378 + 0.4 * UNIT
card = (cx - 44, cy - 22.5, cx + 44, cy + 22.5)  # the a = 0 card measures 88 x 45 on frame 240
print("     09-N2: a = 0 at p.p(1.05, -0.4): x {:.1f}-{:.1f}, y {:.1f}-{:.1f}".format(card[0], card[2], card[1], card[3]))
check("09-N2 clears the lower -0.5 card (top y 455)", card[3] < 455, f"gap {455 - card[3]:.1f} px")
near = math.hypot(card[0] - 343, card[1] - 378)
check("09-N2 clears the r = 1/2 ring, dot radius 5", near > 0.5 * UNIT + 5, f"gap {near - 0.5 * UNIT - 5:.1f} px")
check("09-N2 clears the a label (right end about 368)", card[0] > 368)
check("09-N2 old position agrees with the measured card (x 384-472)", abs(343 + 0.85 * UNIT - 428) < 1)
start = 640 + (4.0 + 0.2) * 90 - 110
print(f"     09-N1: caption at v(right + 0.2, -0.75) spans x {start:.0f}-{start + 220:.0f}, panel axis 893, panel right 1225")
check("09-N1 clears the axis", start > 893 + 8)
check("09-N1 stays in the panel", start + 220 < 1225)
print(f"     07-N1: Re ends about 592; arrow tail at v(-0.42, ...) is x {640 - 0.42 * 90:.0f}, right plane starts {640 + 0.9 * 90:.0f}")
check("07-N1 clears Re", 640 - 0.42 * 90 > 592)
print("     09-N3: 'x' glyph half width about {:.3f}; today's edge margin 7.11 - 6.7 - 0.18 - half = {:.3f}".format(0.5 * 22 * 0.01 / 2, 16 / 9 * 4 - 6.7 - 0.18 - 0.5 * 22 * 0.01 / 2))
check("09-N3 at axis_end + 0.11 gives 0.25", 16 / 9 * 4 - 6.7 - 0.11 - 0.5 * 22 * 0.01 / 2 >= 0.24)
r = 0.26
print("     08-N2: mark radius {} assumed; today's chord {:.4f} vs head 0.16; proposed chord {:.4f}".format(r, 2 * r * math.sin(0.175), 2 * r * math.sin(0.31)))
check("08-N2 proposed chord reaches the head's length", 2 * r * math.sin(0.31) > 0.155)
tip, start_of_chord = r * cmath.exp(1j * 5.4), r * cmath.exp(1j * (5.4 - 0.62))
along = (tip - start_of_chord) / abs(tip - start_of_chord)
check("08-N2 proposed head passes the film's own sign test", (tip.conjugate() * along).imag > 0, f"cross {(tip.conjugate() * along).imag:.3f}")
check("08-N2 proposed base midpoint lands on the arc", abs(tip - 0.16 * along - start_of_chord) < 0.002, f"off by {abs(tip - 0.16 * along - start_of_chord):.4f}")

print("== Word share on the scratch voice: an estimate, for ordering only")
cues = {}
for chapter, count in CHAPTERS.items():
    rows = [line.split("\t") for line in open(RENDER + chapter + ".cues.tsv", encoding="utf-8").read().splitlines() if line.strip()]
    check(f"{chapter} cues read", len(rows) == count, str(len(rows)))
    for row in rows:
        cues[row[0]] = (float(row[1]), float(row[2]))
script = {}
for line in open(SCRIPT, encoding="utf-8"):
    if re.match(r"\d\d_\d+\t", line):
        key, text = line.rstrip("\n").split("\t", 1)
        script[key] = text
for cue, phrase in [
    ("06_1", "Do the same"),
    ("06_1", "Step from z"),
    ("06_1", "divide the change"),
    ("06_1", "As h shrinks"),
    ("06_1", "settle"),
    ("06_5", "which turns the answer"),
    ("06_9", "A reflection reverses"),
    ("06_9", "Squashing fails"),
    ("06_9", "Double every height"),
    ("07_2", "conformal"),
    ("07_2", "A point where"),
    ("07_2", "critical point"),
    ("07_6", "small disk"),
    ("08_3", "dividing by z"),
    ("08_3", "The arrows stack"),
    ("08_3", "whatever the size"),
    ("08_4", "Only one over z"),
    ("08_4", "survives"),
    ("09_3", "Eight sample"),
    ("09_5", "swallowed"),
    ("09_5", "Each pole"),
    ("09_5", "The formula only"),
]:
    text = script[cue]
    at = text.index(phrase)
    before = len(text[:at].split())
    total = len(text.split())
    start, length = cues[cue]
    print(f"     {cue} {phrase!r}: word {before} of {total}, about {start + length * before / total:.1f} s")

print("== Controls: this check must be able to fail")
saved = (passed, failed)
shown("control, wrong mean", m8.real, "1.0040", "{:.4f}")
shown("control, wrong recipe value", recipe_at_one(2.6, 60), "213", "{:.0f}")
control_failures = failed - saved[1]
passed, failed = saved
check("both planted wrong strings failed", control_failures == 2, str(control_failures))

print(f"RESULT {passed} PASS, {failed} FAIL")
