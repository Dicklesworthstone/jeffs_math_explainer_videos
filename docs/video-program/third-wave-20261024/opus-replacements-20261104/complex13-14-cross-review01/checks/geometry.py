"""Recompute the contact bands and clearances of 13_3 and 14_3 from the staged numbers and the old frames' pixels.

Run from the repo root:
  python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex13-14-cross-review01/checks/geometry.py

Written without running or reading Bronze's sweep_check.py. Glyph and ring sizes here are measured on preview02's
frames (canonical source, scratch voice, 720p), which cannot show the repair: there is no native picture of it.
Backings are ideal discs and lines ideal strokes; each px figure is good to about one pixel at 720p. Times use the
scratch rate read from the frames, or the 1.5 s floor, and are not Oliver's.
"""
import json
import math
import os
import re
import sys

from PIL import Image

STAGE = "docs/video-program/third-wave-20261024/opus-replacements-20261104/complex13-radius-repair01/"
CHAPTERS = "explainer_corpus/complex_analysis_visual/src/chapters/"
FRAMES = "narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-combined-picture-frames02/"
ri = open(STAGE + CHAPTERS + "real_integral.rs", encoding="utf-8").read()
co = open(STAGE + "continuation14/" + CHAPTERS + "continuing.rs", encoding="utf-8").read()
fails = 0


def check(ok, what):
    global fails
    fails += 0 if ok else 1
    print(("PASS " if ok else "FAIL ") + what)


def const(name):
    return float(re.search(r"const " + name + r": f64 = (-?[\d.]+);", ri)[1])


R_BACK, I_BACK, I_X, MINUS_I_X = const("R_BACK"), const("I_BACK"), const("POLE_NAME_X"), const("MINUS_POLE_NAME_X")
SHOWN = [float(x) for x in re.search(r"const R_NAME_SHOWN: \[f64; 2\] = \[([\d.]+), ([\d.]+)\];", ri).groups()]
m = re.search(r"fn plane\(\) -> Plane \{\s*Plane::new\(v\((-?[\d.]+), (-?[\d.]+)\), ([\d.]+), \((-?[\d.]+), (-?[\d.]+)\), \((-?[\d.]+), (-?[\d.]+)\)\)", ri)
CX, CY, WIDTH, X0, X1, Y0, Y1 = [float(x) for x in m.groups()]
UNIT = WIDTH / (X1 - X0)
PX = 1280 / (128 / 9)  # 90 px per scene unit at 720p
R = 2.0  # model::ARC_RADII[0], the walk's radius
assert "ARC_RADII: [f64; 3] = [2.0, 5.0, 10.0]" in open("explainer_corpus/complex_analysis_visual/src/model.rs", encoding="utf-8").read()
assert "let r = model::ARC_RADII[0];" in ri or "ARC_RADII[0]" in ri


def plane_px(z):
    return (640 + PX * (CX + UNIT * (z[0] - (X0 + X1) / 2)), 360 - PX * (CY + UNIT * (z[1] - (Y0 + Y1) / 2)))


def half_line(width):
    return width * 0.01 / 2 / UNIT


def r_name(th):
    return (R * 0.72 * math.cos(th) + 0.25 * math.sin(th), R * 0.72 * math.sin(th) - 0.25 * math.cos(th))


def r_shown(th):
    return max(0.0, min(1.0, (th - SHOWN[0]) / (SHOWN[1] - SHOWN[0])))


# ---------------------------------------------------------------- pixels
times13 = json.load(open(FRAMES + "13_real_integral/sampling.json", encoding="utf-8"))["times"]
times14 = json.load(open(FRAMES + "14_continuation/sampling.json", encoding="utf-8"))["times"]


def frame(chapter, n):
    return Image.open(f"{FRAMES}{chapter}/frame-{n:04d}.png").convert("RGB")


GROUND = (0x0E, 0x11, 0x16)


def off_ground(px, tol=14):
    return max(abs(px[i] - GROUND[i]) for i in range(3)) > tol


def bbox(img, cx, cy, half, tol=40):
    xs, ys = [], []
    w = img.load()
    for y in range(int(cy - half), int(cy + half) + 1):
        for x in range(int(cx - half), int(cx + half) + 1):
            if off_ground(w[x, y], tol):
                xs.append(x)
                ys.append(y)
    return (min(xs), min(ys), max(xs) + 1, max(ys) + 1) if xs else None


print("== frames: scale, ground and glyph sizes on preview02 (canonical, scratch voice)")
f88, f100 = frame("13_real_integral", 88), frame("13_real_integral", 100)
check(f88.size == (1280, 720), f"frames are {f88.size}; {PX:.2f} px per scene unit, {PX * UNIT:.2f} per plane unit")
corners = [f88.getpixel(p) for p in ((5, 5), (1275, 5), (5, 715), (1275, 715))]
check(all(not off_ground(c, 3) for c in corners), f"ground() {GROUND} is the frame's ground to 3 levels: corners {corners[0]}, {corners[1]}: a backing is ground on ground")
ring_at = plane_px((0.0, 1.0))
ring = bbox(f88, *ring_at, 14)
ring_mid = ((ring[0] + ring[2]) / 2, (ring[1] + ring[3]) / 2)
ring_half = (ring[2] - ring[0]) / 2 / (PX * UNIT)  # across only: the axis runs through it up and down
check(abs(ring_mid[0] - ring_at[0]) < 1.5 and abs(ring_mid[1] - ring_at[1]) < 1.5, f"the ring's middle on frame 88 {ring_mid} is at z = i by the formula {tuple(round(c, 1) for c in ring_at)}")
print(f"     ring outer half width {ring_half:.3f} plane units (at 40 levels)")

i_at = plane_px((I_X, 1.0))
i_box = bbox(f88, i_at[0], i_at[1], 13, 40)
i_half = ((i_box[2] - i_box[0]) / 2 / (PX * UNIT), (i_box[3] - i_box[1]) / 2 / (PX * UNIT))
i_mid = ((i_box[0] + i_box[2]) / 2, (i_box[1] + i_box[3]) / 2)
far = max(math.hypot(x - i_at[0], y - i_at[1]) for x in (i_box[0], i_box[2]) for y in (i_box[1], i_box[3]))
back_px = I_BACK * PX
print(f"     'i' box {i_box}, half {i_half[0]:.3f} x {i_half[1]:.3f} plane, centre off by {i_mid[0] - i_at[0]:+.1f}, {i_mid[1] - i_at[1]:+.1f} px")
check(far <= back_px + 1.0, f"i_back ({back_px:.1f} px) covers the letter's far corner ({far:.1f} px) to within a pixel")

r_end = plane_px(r_name(math.pi))
r_box = bbox(f100, r_end[0], r_end[1], 15, 40)
r_half = ((r_box[2] - r_box[0]) / 2 / (PX * UNIT), (r_box[3] - r_box[1]) / 2 / (PX * UNIT))
r_mid = ((r_box[0] + r_box[2]) / 2, (r_box[1] + r_box[3]) / 2)
far = max(math.hypot(x - r_end[0], y - r_end[1]) for x in (r_box[0], r_box[2]) for y in (r_box[1], r_box[3]))
back_px = R_BACK * PX
print(f"     moving 'R' at the walk's end on frame 100: box {r_box}, half {r_half[0]:.3f} x {r_half[1]:.3f} plane, centre off by {r_mid[0] - r_end[0]:+.1f}, {r_mid[1] - r_end[1]:+.1f} px")
check(far <= back_px + 1.0, f"r_back ({back_px:.1f} px) covers R's far corner ({far:.1f} px) to within a pixel")

# what a backing would hide, on a frame just before the spoke and R arrive (frame 88, 36.6 s; they fade in from 36.8 s)
print("== what lies under the two backings, frame 88")
w = f88.load()
under_i = [(x, y) for y in range(380, 450) for x in range(310, 380) if math.hypot(x - i_at[0], y - i_at[1]) <= I_BACK * PX and off_ground(w[x, y])]
fringe = bbox(f88, i_at[0], i_at[1], 13, 14)  # the letter at the same 14 levels: its soft edge and the encoder's ringing
outside_letter = [p for p in under_i if not (fringe[0] <= p[0] < fringe[2] and fringe[1] <= p[1] < fringe[3])]
print(f"     the letter at 14 levels: box {fringe}")
check(not outside_letter, f"under i_back: {len(under_i)} pixels off ground, all of them the letter: rooted at full paint, it shows nothing until the spoke reaches it")

union = set()
steps = 720
for k in range(steps + 1):
    th = SHOWN[0] + (math.pi - SHOWN[0]) * k / steps
    if r_shown(th) < 0.1:
        continue  # under 10% a backing moves a 14-level fringe by under 2 levels; the band below covers the near touch
    cx, cy = plane_px(r_name(th))
    rad = R_BACK * PX
    for y in range(int(cy - rad), int(cy + rad) + 2):
        for x in range(int(cx - rad), int(cx + rad) + 2):
            if (x - cx) ** 2 + (y - cy) ** 2 <= rad * rad:
                union.add((x, y))
hit = [p for p in union if off_ground(w[p[0], p[1]])]
axis_x = plane_px((0, 0))[0]
others = [p for p in hit if abs(p[0] - axis_x) > 2.5]
ys = sorted(p[1] for p in hit if abs(p[0] - axis_x) <= 2.5)
check(not others, f"under R's path, th in [{SHOWN[0]}, pi]: {len(hit)} pixels off ground, all on the axis (x = {axis_x:.0f} +- 2.5)" + (f"; others e.g. {others[:6]}" if others else ""))
print(f"     the axis under R's path spans y {ys[0]}..{ys[-1]} at 720p; the union has {len(union)} pixels")
for n in (93, 96, 105):
    w2 = frame("13_real_integral", n).load()
    outside = [p for p in union if p[0] > 600 and off_ground(w2[p[0], p[1]])]
    check(not outside, f"frame {n}: the right-hand panel never reaches R's path")

# ---------------------------------------------------------------- bands
print("== R's and i's places while they wait for the fade-out at sentence 2, frames 96 to 111")
r_px, i_px = plane_px(r_name(math.pi)), plane_px((I_X, 1.0))
last = None
for n in (96, 100, 105, 108, 110, 111):
    img = frame("13_real_integral", n)
    wn = img.load()
    in_r = [(x, y) for y in range(int(r_px[1]) - 14, int(r_px[1]) + 15) for x in range(int(r_px[0]) - 14, int(r_px[0]) + 15) if math.hypot(x - r_px[0], y - r_px[1]) <= R_BACK * PX and off_ground(wn[x, y], 40)]
    box = (min(p[0] for p in in_r), min(p[1] for p in in_r), max(p[0] for p in in_r) + 1, max(p[1] for p in in_r) + 1) if in_r else None
    in_i = [(x, y) for y in range(int(i_px[1]) - 11, int(i_px[1]) + 12) for x in range(int(i_px[0]) - 11, int(i_px[0]) + 12) if math.hypot(x - i_px[0], y - i_px[1]) <= I_BACK * PX and off_ground(wn[x, y], 40)]
    box_i = (min(p[0] for p in in_i), min(p[1] for p in in_i), max(p[0] for p in in_i) + 1, max(p[1] for p in in_i) + 1) if in_i else None
    print(f"     frame {n} ({times13[n]:.1f} s): inside R's backing {box}, inside i's {box_i}")
    last = (box, box_i)
    if n <= 110:
        check(box == r_box or (box and abs(box[0] - r_box[0]) <= 1 and abs(box[2] - r_box[2]) <= 1), f"frame {n}: only R lies within its backing's end place")
    check(box_i and i_box[0] - 1 <= box_i[0] and box_i[2] <= i_box[2] + 1, f"frame {n}: only the letter lies within i's backing")
check(last[0] is None, "frame 111: R has gone and nothing else is there, so the backing's fade shows nothing")

print("== contact bands, 0.01 degree steps; half sizes measured above")


def bands(pred):
    out, start, last = [], None, None
    for k in range(18001):
        deg = k / 100
        if pred(math.radians(deg)):
            if start is None:
                start = deg
            last = deg
        elif start is not None:
            out.append((start, last))
            start = None
    if start is not None:
        out.append((start, last))
    return out


def spoke_dist(th, c):
    """Distance from c to the spoke, a segment from 0 to R at angle th."""
    t = max(0.0, min(R, c[0] * math.cos(th) + c[1] * math.sin(th)))
    return math.hypot(c[0] - t * math.cos(th), c[1] - t * math.sin(th))


def segment_box_hit(th, cx, cy, hx, hy, pad):
    """Does the spoke, thickened by pad, touch the box? Sampled every 0.005 plane units along the spoke."""
    for k in range(401):
        t = R * k / 400
        x, y = t * math.cos(th), t * math.sin(th)
        dx, dy = max(abs(x - cx) - hx, 0.0), max(abs(y - cy) - hy, 0.0)
        if math.hypot(dx, dy) <= pad:
            return True
    return False


spoke_h, gold_h, axis_h = half_line(2.5), half_line(5.0), half_line(2.0)
RING_DOT = 0.13 / UNIT
BACK_R, BACK_I = R_BACK / UNIT, I_BACK / UNIT
floor_w = 1.5


def walker_angle(n):
    """The mean angle of near-white pixels on the half circle: the walker is ink, the path is gold."""
    w = frame("13_real_integral", n).load()
    hits = []
    for k in range(0, 1801):
        th = math.radians(k / 10)
        x, y = plane_px((R * math.cos(th), R * math.sin(th)))
        px = w[int(round(x)), int(round(y))]
        if min(px) > 170:
            hits.append(k / 10)
    return sum(hits) / len(hits) if hits else None


rows_w = [(times13[n], walker_angle(n)) for n in range(90, 96)]
rows_w = [(t, a) for t, a in rows_w if a is not None]
mean_t, mean_a = sum(t for t, a in rows_w) / len(rows_w), sum(a for t, a in rows_w) / len(rows_w)
rate = sum((t - mean_t) * (a - mean_a) for t, a in rows_w) / sum((t - mean_t) ** 2 for t, a in rows_w)
t_zero = mean_t - mean_a / rate
scratch_w = 180 / rate
worst = max(abs(a - rate * (t - t_zero)) for t, a in rows_w)
print("     walker on frames 90-95: " + ", ".join(f"{t:.1f} s {a:.1f}" for t, a in rows_w))
check(worst < 1.5 and abs(t_zero - 37.2) < 0.08, f"steady walk measured here: starts {t_zero:.2f} s (cue 36.1 + 1.1 = 37.2), lasts {scratch_w:.2f} s, worst fit {worst:.2f} degrees: tween_linear holds and T0 holds")


def secs(deg, w_s):
    return deg / 180 * w_s


def row(name, band):
    width = sum(b - a for a, b in band)
    text = ", ".join(f"{a:.1f}-{b:.1f}" for a, b in band) or "none"
    print(f"     {name:<44} {text:<26} {secs(width, scratch_w):.3f} s scratch, {secs(width, floor_w):.3f} s floor")
    return band


row("R's box on the gold segment (canonical)", bands(lambda t: abs(r_name(t)[1]) <= r_half[1] + gold_h and abs(r_name(t)[0]) <= R))
back_on_gold = row("R's backing on the gold segment, if shown", bands(lambda t: abs(r_name(t)[1]) <= BACK_R + gold_h))
row("R's box on the axis", bands(lambda t: abs(r_name(t)[0]) <= r_half[0] + axis_h))
row("R's backing notching the axis", bands(lambda t: abs(r_name(t)[0]) <= BACK_R + axis_h and r_shown(t) > 0))
ring_band = row("spoke touching the mark's dark dot", bands(lambda t: spoke_dist(t, (0, 1)) <= RING_DOT + spoke_h))
letter_band = row("spoke touching the letter i", bands(lambda t: segment_box_hit(t, I_X, 1.0, i_half[0], i_half[1], spoke_h)))
iback_band = row("spoke touching i_back", bands(lambda t: spoke_dist(t, (I_X, 1.0)) <= BACK_I + spoke_h))
clear = iback_band[0][0] - ring_band[0][1]
print(f"     spoke clear of both discs for {clear:.2f} degrees between them: {secs(clear, scratch_w) * 60:.1f} frames at 60 fps, scratch rate")
print(f"     that is a clear spoke, moving; only a short chord is ever hidden: at most {2 * RING_DOT * PX * UNIT:.0f} px by the mark, {2 * BACK_I * PX * UNIT:.0f} px by i_back, of a spoke {R * PX * UNIT:.0f} px long")
print(f"     i_back's band against the letter's alone: {(iback_band[0][1] - iback_band[0][0]) / (letter_band[0][1] - letter_band[0][0]):.2f} x")
check(iback_band[0][1] - iback_band[0][0] < 15 and letter_band[0][1] - letter_band[0][0] < 7, "the handoff's letter band (103.9-107.3, 0.06 s) and hidden band (100.5-110.6, 0.18 s) read at height 1 only: both grow a little, its conclusion stands")

start_deg, full_deg = math.degrees(SHOWN[0]), math.degrees(SHOWN[1])
last_box = max(b for a, b in bands(lambda t: t < 0.6 and abs(r_name(t)[1]) <= r_half[1] + gold_h))
check(last_box < start_deg, f"R's box leaves the segment at {last_box:.2f} degrees, before R starts to show at {start_deg:.2f}")
shown_on_gold = [t for a, b in back_on_gold for t in (a, b) if a <= 90 and r_shown(math.radians(b)) > 0]
check(all(r_shown(math.radians(k / 100)) == 0 for a, b in back_on_gold for k in range(int(a * 100), int(b * 100) + 1) if a <= 90), f"R's backing is at zero wherever it would notch the gold on the way up (band ends {back_on_gold[0][1]:.2f})")
check(not any(a > 90 for a, b in back_on_gold), "and it does not come back down to the gold at the walk's end")
print(f"     ramp {start_deg:.2f} to {full_deg:.2f} degrees: {secs(full_deg - start_deg, scratch_w):.3f} s scratch, {secs(full_deg - start_deg, floor_w) * 60:.1f} frames at the 1.5 s floor")

print("== clearances of R's backing, th over the shown range, plane units (px at 720p)")
angles = [SHOWN[0] + (math.pi - SHOWN[0]) * k / 4000 for k in range(4001)]


def report(name, gap):
    print(f"     {name:<30} {gap:+.3f} ({gap * PX * UNIT:+.1f} px)")
    check(gap > 0, f"clear of {name}")


report("the spoke", min(spoke_dist(t, r_name(t)) for t in angles) - BACK_R - spoke_h)
report("the half circle", min(R - gold_h - math.hypot(*r_name(t)) - BACK_R for t in angles))
report("the mark's dark dot", min(math.hypot(r_name(t)[0], r_name(t)[1] - 1) for t in angles) - BACK_R - RING_DOT)
report("i_back", min(math.hypot(r_name(t)[0] - I_X, r_name(t)[1] - 1) for t in angles) - BACK_R - BACK_I)
report("the gold segment", min(abs(r_name(t)[1]) for t in angles) - BACK_R - gold_h)
print("     (that smallest gap is at the ramp's first step, opacity zero: the disc is rising away from the gold as it appears)")
print("== i_back's own clearances")
report("the axis (i_back)", abs(I_X) - BACK_I - axis_h)
report("the mark's dark dot (i_back)", abs(I_X) - BACK_I - RING_DOT)

print("== the growth, R from 2 to 10, with the poles closing in and the names following (i_back is gone by then)")
lit_h = half_line(8.0)
ks = [2 / (2 + 8 * j / 400) for j in range(401)]
print(f"     height of the pole: {ks[0]:.3f} down to {ks[-1]:.3f}")
report("ring to lit segment", min(ks) - ring_half - lit_h)
report("letter i to lit segment", min(ks) - i_half[1] - lit_h)

print("== teaching: R arrives before the voice says it (share of letters, not a measured word time)")
SAID = "On the half circle, z squared has length "
TO_BUMP = SAID + "R squared, "
share = len(SAID) / len(TO_BUMP)
ON = 1.1  # 0.3 + 0.4 + 0.4 of fades before the walk


def shown_at(t_bump):
    walk = max(t_bump - ON, floor_w)
    return ON + walk * SHOWN[1] / math.pi


limit = next(k / 1000 for k in range(1000, 8000) if shown_at(k / 1000) < share * k / 1000)
check(limit < 2.5, f"R is fully up before 'R squared' whenever 'so the bump' comes later than {limit:.2f} s into the cue; the phrase has {len(TO_BUMP)} letters. Scratch: {scratch_w + ON:.2f} s")

print("== 14_3: the chain on the plane")
m = re.search(r"pub fn chain_plane\(\) -> Plane \{\s*Plane::new\(v\((-?[\d.]+), (-?[\d.]+)\), ([\d.]+), \((-?[\d.]+), (-?[\d.]+)\), \((-?[\d.]+), (-?[\d.]+)\)\)", co)
cx, cy, width, x0, x1, y0, y1 = [float(x) for x in m.groups()]
unit = width / (x1 - x0)
check(abs(unit - 1.0) < 1e-12, "its unit is 1: the mark's dot is 0.13 plane units")
centres = [(1 + math.cos(math.pi + 2 * math.pi * k / 8), math.sin(math.pi + 2 * math.pi * k / 8)) for k in range(9)]
chords = [math.hypot((a[0] + b[0]) / 2 - 1, (a[1] + b[1]) / 2) for a, b in zip(centres, centres[1:])]
check(min(chords) - 0.13 - 0.0125 > 0.7, f"no path piece comes near the pole: nearest chord {min(chords):.3f} from it")
check(all(abs(math.hypot(c[0] - 1, c[1]) - 1) < 1e-12 for c in centres), "the chain's centres are all 1 from the pole, so all eight rims meet there")
check(min(chords) - 0.08 - 0.13 > 0.7, f"the walker moves along the chords: its edge stays {min(chords) - 0.08:.3f} from the pole, {min(chords) - 0.08 - 0.13:.3f} from the backing")
f103 = frame("14_continuation", 103)
mark = (640 + 90 * (cx + unit * (1 - (x0 + x1) / 2)), 360 - 90 * (cy + unit * (0 - (y0 + y1) / 2)))
w3 = f103.load()
# the ring's own pink ends about 10 px out; the backing at 11.7 px; so anything pink past 13.5 px is the name
pink = [(x, y) for y in range(int(mark[1]) - 40, int(mark[1])) for x in range(int(mark[0]), int(mark[0]) + 40) if w3[x, y][0] > 120 and w3[x, y][2] < 170 and w3[x, y][1] < 100 and math.hypot(x - mark[0], y - mark[1]) > 13.5]
gap = min(math.hypot(x - mark[0], y - mark[1]) for x, y in pink) - 0.13 * 90
check(gap > 3, f"frame 103: the name 1 is {gap:.1f} px clear of the mark's backing at 720p, measured to its nearest pixel; the handoff's '12 px' is {12 - gap:+.1f} px off")
print("RESULT " + ("PASS" if fails == 0 else f"FAIL {fails}"))
sys.exit(1 if fails else 0)
