"""numerics01.py: an independent prototype of the numbers in counting 21c5043a.

1. the true Moebius walker of 11_3: the image circle by direct sampling (not root's closed form), its gain,
   its turns, how far it misses zero, and where it lands on screen;
2. the two new 11_2 tests of CC3 (winding of the input loop around the marks; the output's -1);
3. the NEEDS cards' widths against the output window;
4. the early 11_6 note ("output size compressed", 18 pt) in its row.

Constants are those of the bound 760 source: MOEBIUS_ZERO 0.45, MOEBIUS_POLE 1.35, OUT_R 1.35,
WALK_SAMPLES 720, SEVERAL_R 1.6, planes() output Plane::new(v(3.25,-0.35), 5.0, (-1.5,1.5), (-1.5,1.5)),
carded = text width + 0.5, kit text at size/100 scene units an em (cmr10), 270 px a unit at 4K.

usage: python3 -I -B numerics01.py OUT
"""
import cmath
import math
import sys

A, B, OUT_R, N, SEVERAL_R = 0.45, 1.35, 1.35, 720, 1.6
OUT_C, OUT_W = (3.25, -0.35), 5.0
UNIT = OUT_W / 3.0
FRAME = (OUT_C[0] - OUT_W / 2, OUT_C[0] + OUT_W / 2, OUT_C[1] - OUT_W / 2, OUT_C[1] + OUT_W / 2)
lines = []
say = lines.append


def moebius(z):
    return (z - A) / (z - B)


def scene(w):
    return OUT_C[0] + w.real * UNIT, OUT_C[1] + w.imag * UNIT


def turns(ws):
    return sum(cmath.phase(ws[k + 1] / ws[k]) for k in range(len(ws) - 1)) / (2 * math.pi)


say("output frame x %.3f..%.3f  y %.3f..%.3f  (unit %.4f scene per plane unit)" % (*FRAME, UNIT))
say("== 1. 11_3 true Moebius walker")
for r in (0.8, 1.7):
    raw = [moebius(cmath.rect(r, 2 * math.pi * j / N)) for j in range(N + 1)]
    xs = [w.real for w in raw]
    # the image is symmetric about the real axis: its extreme real points are a diameter
    c, rho = (max(xs) + min(xs)) / 2, (max(xs) - min(xs)) / 2
    den = r * r - B * B
    c_root, rho_root = (r * r - A * B) / den, r * abs(B - A) / abs(den)
    off = max(abs(abs(w - c) - rho) for w in raw)
    gain = OUT_R / (abs(c) + rho)
    shown = [w * gain for w in raw]
    pts = [scene(w) for w in shown]
    say("r %.1f: sampled centre %.12f radius %.12f | root closed form %.12f %.12f | max off-circle %.1e"
        % (r, c, rho, c_root, rho_root, off))
    say("   gain %.6f  scaled centre %.6f radius %.6f  max|shown| %.12f  min|shown| %.6f  turns %+.12f"
        % (gain, c * gain, rho * gain, max(abs(w) for w in shown), min(abs(w) for w in shown), turns(raw)))
    say("   misses zero by (|c|-rho)*gain = %.6f plane units = %.3f scene = %.0f px at 4K"
        % ((abs(c) - rho) * gain, (abs(c) - rho) * gain * UNIT, (abs(c) - rho) * gain * UNIT * 270)
        if abs(c) > rho else "   encloses zero (|c| %.4f < rho %.4f)" % (abs(c), rho))
    say("   on screen x %.3f..%.3f  y %.3f..%.3f  inside frame: %s"
        % (min(p[0] for p in pts), max(p[0] for p in pts), min(p[1] for p in pts), max(p[1] for p in pts),
           all(FRAME[0] < p[0] < FRAME[1] and FRAME[2] < p[1] < FRAME[3] for p in pts)))
    # the old squash walker (s = 1): radius R|w|/(|w| + 1), for comparison
    old = [w * (OUT_R / (abs(w) + 1.0)) for w in raw]
    say("   old squash s=1: |shown| %.4f..%.4f (a circle would need one centre; the old one is the D shape)"
        % (min(abs(w) for w in old), max(abs(w) for w in old)))
say("   inset card (11_3 sentence 4): x 4.875..6.625  y 1.275..3.225")

say("== 2. CC3's 11_2 tests")
for p in (1 + 0j, -1 + 0j):
    walk = [cmath.rect(SEVERAL_R, 2 * math.pi * k / N) for k in range(N)] + [SEVERAL_R + 0j]
    say("input loop |z| = 1.6 around %s: %+.12f turns" % (p, turns([z - p for z in walk])))
zdp = [(z - 1) / (z + 1) ** 2 for z in (cmath.rect(SEVERAL_R, 2 * math.pi * j / N) for j in range(N + 1))]
say("output turns_so_far(ZeroDoublePole, 0, 1.6, 1): %+.12f (fact argument.several.turns = -1)" % turns(zdp))

say("== 3. NEEDS cards (cmr10 ems x 22/100 x 1.01 + 0.5) in the output window")
EM = {' ': 0.333334, ',': 0.277779, 'i': 0.277779, 'l': 0.277779, 'a': 0.500002, 'g': 0.500002,
      'o': 0.500002, 'b': 0.555557, 'd': 0.555557, 'h': 0.555557, 'n': 0.555557, 'p': 0.555557,
      'u': 0.555557, 'c': 0.444446, 'e': 0.444446, 'z': 0.444446, 'f': 0.305557, 'k': 0.527781,
      'x': 0.527781, 'm': 0.833336, 'r': 0.391668, 's': 0.394445, 't': 0.388891, 'w': 0.722224}


def ems(s):
    return sum(EM[ch] for ch in s)


old = ems("the loop winds once, counterclockwise")
say("old first card: %.4f ems, %.4f units; master ink 991/270 = %.4f; |diff| %.4f (test bound 0.03)"
    % (old, old * 0.22, 991 / 270, abs(old * 0.22 - 991 / 270)))
for n in ("the input loop winds once, counterclockwise", "it passes through no zero and no pole",
          "f is holomorphic inside, except at poles"):
    card = ems(n) * 0.22 * 1.01 + 0.5
    left, right = 3.25 - card / 2, 3.25 + card / 2
    say("%-45s card %.4f  x %.4f..%.4f  margins %.4f / %.4f (test needs > 0.08)  px %.0f..%.0f"
        % (n, card, left, right, left - FRAME[0], FRAME[1] - right, 1920 + left * 270, 1920 + right * 270))

say("== 4. 11_6 early note, 18 pt dim, at (3.25, 2.55)")
note = "output size compressed"
wn = ems(note) * 0.18
say("'%s': %.4f ems, %.4f units: x %.3f..%.3f; row 2.55 sits between frame top %.3f and counter 3.15"
    % (note, ems(note), wn, 3.25 - wn / 2, 3.25 + wn / 2, FRAME[3]))
say("-3z label at THREE_Z_AT (0.95, 1.05): scene (%.3f, %.3f)" % scene(0.95 + 1.05j))
with open(sys.argv[1], "x", encoding="utf-8") as f:
    f.write("\n".join(lines) + "\n")
print("\n".join(lines))
