"""Numeric mirror of the new geometry in chapters 11-14 and of the new tests' tolerances.

Nothing compiles here, so each `assert` in the new Rust tests is prototyped below before root runs
it. Layout numbers use text width = CMR10 ems x size x 0.01; widths are estimates, not glyph probes.
Run: python3 -I -B <this file>
"""
import cmath
import math

TAU = 2 * math.pi
ok = True


def check(name, value, passed):
    global ok
    ok = ok and passed
    print(f"{'ok  ' if passed else 'FAIL'} {name}: {value}")


def circle(c, r, n):
    return [c + r * cmath.exp(1j * TAU * k / n) for k in range(n)]


def squash(w, s, radius):
    m = abs(w)
    return w if m == 0 else w * (radius / (m + s))


def plane(cx, cy, width, x, y):
    unit = width / (x[1] - x[0])
    height = unit * (y[1] - y[0])
    return dict(
        unit=unit,
        left=cx - width / 2,
        right=cx + width / 2,
        top=cy + height / 2,
        bottom=cy - height / 2,
        p=lambda z: (cx + (z.real - (x[0] + x[1]) / 2) * unit, cy + (z.imag - (y[0] + y[1]) / 2) * unit),
    )


print("--- 11_2: (z - 1)/(z + 1)^2 on |z| = 1.6, the bound facts.rs now certifies")
zdp = lambda z: (z - 1) / (z + 1) ** 2
samples = circle(0, 1.6, 4096)
least = min(abs(zdp(z)) for z in samples)
bound = (abs(3 - 0) + 1.6) / abs(abs(-1 - 0) - 1.6) ** 3
reach = bound * TAU * 1.6 / 4096
check("least |f| on the loop", round(least, 5), least > 0.088)
check("derivative bound (3 + r)/0.6^3", round(bound, 3), abs(bound - 21.296) < 0.001)
truth = max(abs((3 - z) / (z + 1) ** 3) for z in samples)
# the bound is tight: |3 - z| and 1/|z + 1|^3 both peak at z = -1.6, which is a sample
check("bound >= the true max |f'| (equal at z = -1.6)", round(truth, 3), truth <= bound * (1 + 1e-12))
check("one step's reach < least |f|", round(reach, 5), reach < least)
turns = sum(cmath.phase(zdp(samples[(k + 1) % 4096]) / zdp(samples[k])) for k in range(4096)) / TAU
check("sampled turns", round(turns, 9), round(turns) == -1)
coarse = circle(0, 1.6, 720)
steps = [cmath.phase(zdp(coarse[(k + 1) % 720]) / zdp(coarse[k])) for k in range(720)]
check("the live counter's 720 samples: largest step, turns", (round(max(map(abs, steps)), 4), round(sum(steps) / TAU, 9)), max(map(abs, steps)) < math.pi / 2 and abs(sum(steps) / TAU + 1) < 1e-9)
out = plane(3.25, -0.35, 5.0, (-1.5, 1.5), (-1.5, 1.5))
drawn = min(abs(squash(zdp(z), 0.5, 1.35)) for z in samples) * out["unit"]
check("smallest output as drawn, scene units (test asks > 0.3)", round(drawn, 4), drawn > 0.3)
turned = sum(cmath.phase(squash(zdp(samples[(k + 1) % 4096]), 0.5, 1.35) / squash(zdp(samples[k]), 0.5, 1.35)) for k in range(4096)) / TAU
check("the squash keeps the turns", round(turned, 9), round(turned) == -1)

print("--- 11_3: inset direction marks")
inset = plane(5.75, 2.35, 1.5, (-0.6, 0.6), (-0.6, 0.6))
for name, r, a, b in (("outer", 0.45, 1.25, 1.95), ("inner", 0.28, -1.25, -2.05)):
    chord = 2 * r * math.sin(abs(b - a) / 2) * inset["unit"]
    check(f"{name} chord, scene units", round(chord, 3), chord > 0.2)

print("--- 11_5: the ring of the question")
height = 2 * (28 / 48 * 0.62) + 0.3
rx, ry = 4.4, 2.75 - height / 2 - 0.45
check("ring centre", (rx, round(ry, 3)), True)
check("clear of the top edge", round(out["top"] - (ry + 0.30), 3), ry + 0.30 < out["top"])
check("clear of the output's x axis", round(ry - 0.30 - out["p"](0j)[1], 3), ry - 0.30 > out["p"](0j)[1])
check("clear of the output's y axis", round(rx - 0.30 - out["p"](0j)[0], 3), rx - 0.30 > out["p"](0j)[0])
check("card bottom above the window", round(2.75 - height / 2 - out["top"], 3), 2.75 - height / 2 > out["top"])

print("--- 11_6: the name -3z against the walk")
quintic = lambda z: z**5 - 3 * z + 1
far = max(abs(squash(quintic(z), 2.0, 1.35)) for z in circle(0, 1.0, 720))
check("largest output radius", round(far, 3), far < 0.97)
check("name radius less margin (test asks > that)", round(abs(0.95 + 1.05j) - 0.2, 3), abs(0.95 + 1.05j) - 0.2 > far)

print("--- 11_8: the reach carried round")
reach8 = 8 * TAU / 64
step = cmath.exp(1j * TAU / 64) ** 8
check("reach", round(reach8, 4), reach8 < 1)
check("next sample inside the reach", round(abs(step - 1), 4), abs(step - 1) < reach8)
check("disk inside the window +-2", round(1 + reach8, 4), 1 + reach8 < 2)
window = plane(-2.55, 0.55, 2.8, (-2.0, 2.0), (-2.0, 2.0))
name_y = window["p"](2.2j)[1]
check("'reach covers 0' above the window's top edge", (round(name_y, 3), round(window["top"], 3)), name_y - 0.1 > window["top"])
span = 6.6 * 18 * 0.01
nx = window["p"](0.95 - 1.2j)[0]
check("'reach L x step' clear of the y axis", (round(nx - span / 2, 3), round(window["p"](0j)[0], 3)), nx - span / 2 > window["p"](0j)[0])
check("'reach L x step' clear of the disk at rest", round(abs((0.95 - 1.2j) - 1) - reach8, 3), abs((0.95 - 1.2j) - 1) > reach8)

print("--- 12_2: direction marks")
small = plane(-3.6, -0.6, 4.6, (-1.6, 1.6), (-1.6, 1.6))
chord = 2 * 0.25 * math.sin(0.35) * small["unit"]
check("small mark chord, scene units", round(chord, 3), chord > 0.2)
big = 2 * 1.35 * math.sin(0.175) * small["unit"]
check("big mark chord, scene units", round(big, 3), big > 0.5)
for q in (-0.7 + 0.5j, 0.6 + 0.6j, -0.7j):
    a = cmath.phase(q)
    check(f"corridor at {a:.3f} rad misses the big mark 1.75..2.1", round(a, 3), a < 1.75 or a > 2.1)

print("--- 13_3: the plane's poles and note")
integral = plane(-3.0, -1.15, 5.0, (-2.4, 2.4), (-1.25, 2.25))
check("note row below the plane", (-3.25, round(integral["bottom"], 3)), -3.25 < integral["bottom"] - 0.15)
note_width = 30 * 20 * 0.01
check("note's estimated span", (round(-3.0 - note_width / 2, 2), round(-3.0 + note_width / 2, 2)), -3.0 - note_width / 2 > -6.86)
for at in (2.0, 5.0, 10.0):
    k = 2.0 / at
    check(f"i drawn at {k:.2f} when R = {at:g}: same fraction of the radius", round(k / 2.0 - 1 / at, 12), abs(k / 2.0 - 1 / at) < 1e-12)

print("--- 13_7: the residue")
for q in (cmath.exp(1j * math.pi / 4), cmath.exp(3j * math.pi / 4)):
    check("1/(4p^3) = -p/4", round(abs(1 / (4 * q**3) + q / 4), 14), abs(1 / (4 * q**3) + q / 4) < 1e-12)
    check("Im p = sqrt(2)/2", round(q.imag - math.sqrt(2) / 2, 14), abs(q.imag - math.sqrt(2) / 2) < 1e-12)

print("--- 14_2: the small disk")
mini = plane(5.2, 1.65, 1.9, (-1.6, 1.6), (-1.6, 1.6))
check("right of the second chart (right edge 2.7)", round(mini["left"], 3), mini["left"] > 2.7)
check("above the value's row", round(mini["bottom"], 3), 0.05 + 0.5 < mini["bottom"])
check("inside the lint limit", round(mini["right"], 3), mini["right"] < 6.86)

print("--- 14_3: 'small disk' at (-0.47, 0.3), size 18")
half = 4.6 * 18 * 0.01 / 2
left, right = -0.47 - half, -0.47 + half
check("clear of the y axis", round(right, 3), right < 0)
rim = -math.sqrt(1 - (0.3 + 0.09) ** 2)
check("clear of the disk's rim", (round(left, 3), round(rim, 3)), left > rim)

print("RESULT", "PASS" if ok else "FAIL")
raise SystemExit(0 if ok else 1)
