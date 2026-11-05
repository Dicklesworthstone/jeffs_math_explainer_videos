"""Recompute the numbers shown in chapters 00-03, measure the same geometry on the frames, and share out sentence times.

Run from the repo root:
  python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage06-review00-03/checks/numbers.py

Read only outside this folder. "shown" strings below are what I read off the named frame by eye; the check says the
arithmetic behind them is right, not that I read them right. Pixel checks use colour masks on 720p frames of the
scratch-voice preview. Times are the scratch voice or a share of letters. None of them is an Oliver time.
"""
import json
import math
import sys

from PIL import Image

B = "narration_lab/third-wave-20261024/root/five-film-release-20261027/"
FR = B + "logarithm-stage06-frames01/"
RENDER = B + "logarithm-stage06-preview01/render/"
SCRIPT = "explainer_corpus/logarithm_mysteries/script.tsv"
CH = {"00": "00_ratio_ruler", "01": "01_building_ruler", "02": "02_slide_rule", "03": "03_folded_decade"}
fails = 0


def check(ok, what):
    global fails
    fails += 0 if ok else 1
    print(("PASS " if ok else "FAIL ") + what)


def frame(c, n):
    return Image.open(f"{FR}{CH[c]}/frame-{n:04d}.png").convert("RGB")


def times(c):
    return json.load(open(f"{FR}{CH[c]}/sampling.json", encoding="utf-8"))["times"]


def px(x):
    return 640 + 90 * x


def green(p):
    return p[1] > 130 and p[1] - p[0] > 30 and p[1] - p[2] > 40


def teal(p):
    return p[1] > 130 and p[1] - p[0] > 60 and p[2] - p[0] > 30 and p[1] - p[2] > 10


def blue(p):
    return p[2] > 150 and p[2] >= p[1] and p[1] - p[0] > 50


def gold(p):
    return p[0] > 190 and p[1] > 160 and p[2] < 150


def red(p):
    return p[0] > 170 and p[1] < 130 and p[2] < 130


def purple(p):
    return p[0] > 100 and p[2] > 120 and p[1] < p[0] - 12 and p[1] < p[2] - 20


def grey(p):
    return min(p) > 95 and max(p) - min(p) < 45


def ink(p):
    return max(p) > 70


def row_runs(im, y, pred, x0=0, x1=1280):
    out, start = [], None
    for x in range(x0, x1):
        on = pred(im.getpixel((x, y)))
        if on and start is None:
            start = x
        if not on and start is not None:
            out.append((start, x - 1))
            start = None
    if start is not None:
        out.append((start, x1 - 1))
    return out


def col_runs(im, x, pred, y0=0, y1=720):
    out, start = [], None
    for y in range(y0, y1):
        on = pred(im.getpixel((x, y)))
        if on and start is None:
            start = y
        if not on and start is not None:
            out.append((start, y - 1))
            start = None
    if start is not None:
        out.append((start, y1 - 1))
    return out


def centres(runs):
    return [(a + b) / 2 for a, b in runs]


def worst(expected, measured):
    return max(min(abs(e - m) for m in measured) for e in expected)


def box_rows(im, box, pred):
    x0, y0, x1, y1 = box
    return [y for y in range(y0, y1) if any(pred(im.getpixel((x, y))) for x in range(x0, x1))]


def box_cols(im, box, pred):
    x0, y0, x1, y1 = box
    return [x for x in range(x0, x1) if any(pred(im.getpixel((x, y))) for y in range(y0, y1))]


def fit(what, expected, measured, tol=1.6):
    w = worst(expected, measured)
    check(w <= tol, f"{what}: worst {w:.2f} px of {len(expected)} marks")
    return w


# ---------------------------------------------------------------- arithmetic
print("== arithmetic: what the frame shows against a value computed here")
lg, lb = math.log10, math.log2
a4 = 440.0
rows = [
    ("00", 18, "55, 110, 220", ", ".join(str(55 * 2**k) for k in range(3))),
    ("00", 18, "440 880 1760 3520", " ".join(str(55 * 2**k) for k in range(3, 7))),
    ("00", 18, "0 880 1760 2640 3520", " ".join(str(880 * k) for k in range(5))),
    ("00", 151, "660 by difference?", f"{(440 + 880) // 2} by difference?"),
    ("00", 197, "622", f"{a4 * 2**0.5:.0f}"),
    ("00", 206, "622.25 Hz", f"{a4 * 2**0.5:.2f} Hz"),
    ("01", 200, "19/12 = 1.583", f"19/12 = {19 / 12:.3f}"),
    ("01", 200, "1318.51", f"{a4 * 2 ** (19 / 12):.2f}"),
    ("01", 200, "1320", f"{3 * 440}"),
    ("01", 225, "1.585", f"{lb(3):.3f}"),
    ("01", 180, "531,441", f"{3**12:,}"),
    ("01", 180, "524,288", f"{2**19:,}"),
    ("01", 180, "1.36% apart", f"{(3**12 / 2**19 - 1) * 100:.2f}% apart"),
    ("02", 65, "0.505", f"{lg(3.2):.3f}"),
    ("02", 40, "0.398", f"{lg(2.5):.3f}"),
    ("02", 103, "0.903", f"{lg(8):.3f}"),
    ("02", 103, "0.398 + 0.505 = 0.903", f"{lg(2.5):.3f} + {lg(3.2):.3f} = {lg(2.5 * 3.2):.3f}"),
    ("02", 100, "2.5 x 3.2 = 8", f"2.5 x 3.2 = {2.5 * 3.2:.0f}"),
    ("02", 225, "4 x 6 = 24", f"4 x 6 = {4 * 6}"),
    ("03", 96, "0.602", f"{lg(4):.3f}"),
    ("03", 96, "0.778", f"{lg(6):.3f}"),
    ("03", 96, "1.380", f"{lg(4) + lg(6):.3f}"),
    ("03", 96, "0.380", f"{lg(2.4):.3f}"),
    ("03", 136, "0.24 2.4 24 240", " ".join(f"{2.4 * 10**k:g}" for k in (-1, 0, 1, 2))),
]
for c, n, shown, computed in rows:
    check(shown == computed, f"{c} frame {n}: {shown!r} computed {computed!r}")
check(0.5 == 1760 / 3520 and 0.125 == 440 / 3520, "00: 1/2 of the ruler = 1760/3520, 1/8 = 440/3520")
check(abs(lg(3) - lb(3) / lb(10)) < 1e-12, "01: log10 x = log2 x / log2 10 at x = 3")
print(f"note: 2^(19/12) = {2 ** (19 / 12):.4f}; sum of the two log10 = {lg(4) + lg(6):.5f}")
print(f"note: moving the marker from 19/12 to log2 3 is {(lb(3) - 19 / 12) * 180:.2f} px at 720p")
print(f"note: 12 threes against 19 twos differ by {12 * lb(3) - 19:.4f} step, {(12 * lb(3) - 19) * 986 / 19:.2f} px at 986/19 px a step, the rows on frame 180")
print(f"note: a closer match if a picture is wanted, new fact: 3^53 / 2^84 = {3**53 / 2**84:.5f}, 84/53 = {84 / 53:.4f}")

# ---------------------------------------------------------------- geometry
print("== geometry on the frames, each mark against its value")
freq = [55 * 2**k for k in range(7)]
lin = [px(-6.2 + 12.4 * f / 3520) for f in freq]
im = frame("00", 18)
runs = row_runs(im, 202, green)
check(abs(runs[0][0] - (lin[0] - 9)) < 2 and abs(runs[0][1] - (lin[1] + 9)) < 2, f"00 frame 18: 55 and 110 dots touch, run {runs[0]}, expected {lin[0] - 9:.1f}..{lin[1] + 9:.1f}")
fit("00 frame 18: dots 220..3520", lin[2:], centres(runs[1:]))
im = frame("00", 197)
fit("00 frame 197: gold 622.25", [px(-6.2 + 12.4 * a4 * 2**0.5 / 3520)], centres(row_runs(im, 190, gold)))
fit("00 frame 197: red 660", [px(-6.2 + 12.4 * 660 / 3520)], centres(row_runs(im, 190, red)))
print(f"note: red 660 stands {12.4 * 90 * (660 - a4 * 2**0.5) / 3520:.1f} px right of gold 622.25 at 720p")

pos = lambda v: px(-3.6 + 2.0 * lb(v))
fit("01 frame 200: ticks 1/2 1 r2 2 4 8", [pos(v) for v in (0.5, 1, 2**0.5, 2, 4, 8)], centres(row_runs(frame("01", 200), 318, green)))
fit("01 frame 287: adds 3 and 10", [pos(v) for v in (0.5, 1, 2**0.5, 2, 3, 4, 8, 10)], centres(row_runs(frame("01", 287), 318, green)))
fit("01 frame 200: marker at 19/12", [pos(3)], centres(row_runs(frame("01", 200), 350, gold)))

D = lambda v: px(-6 + 8 * lg(v))
whole = range(1, 11)
im = frame("02", 15)
fit("02 frame 15: C at home", [D(v) for v in whole], centres(row_runs(im, 283, teal)))
fit("02 frame 15: D", [D(v) for v in whole], centres(row_runs(im, 355, green)))
im = frame("02", 65)
shift = 720 * lg(2.5)
fit("02 frame 65: C slid to 2.5", [D(v) + shift for v in whole], centres(row_runs(im, 283, teal)))
pair = centres(row_runs(im, 312, teal))
ring_c = [(pair[0] + pair[1]) / 2, (pair[2] + pair[3]) / 2]
fit("02 frame 65: rings on C's 1 and C's 3.2", [D(1) + shift, D(3.2) + shift], ring_c)
check(abs(ring_c[1] - D(8)) < 1.6, f"02 frame 65: C's 3.2 ring {ring_c[1]:.1f} over D's 8 {D(8):.1f}")
im = frame("02", 138)
shift = 720 * lg(4)
fit("02 frame 138: C slid to 4", [D(v) + shift for v in whole], centres(row_runs(im, 283, teal)))
im = frame("02", 182)
shift = 720 * (lg(4) - 1)
seen = [D(v) + shift for v in whole if D(v) + shift > 2]
fit("02 frame 182: C's 10 on D's 4", seen, centres(row_runs(im, 283, teal)))
check(abs(D(6) + shift - D(2.4)) < 0.2, f"02: C's 6 at {D(6) + shift:.1f} is D's 2.4 at {D(2.4):.1f}")
print(f"note: after the slide, C's hidden part is below {10 ** ((-7.11 + 6 - 8 * (lg(4) - 1)) / 8):.2f}; nothing spoken needs it")

im = frame("03", 100)
cx, cy, R = 640 + 90 * -2.8, 360 + 90 * 0.3, 2.1 * 90


def circle_hits(im, r, pred):
    hits, start = [], None
    for k in range(-100, 3500):
        a = math.radians(k / 10)
        on = pred(im.getpixel((round(cx + r * math.sin(a)), round(cy - r * math.cos(a)))))
        if on and start is None:
            start = k
        if not on and start is not None:
            hits.append((start + k - 1) / 20)
            start = None
    return hits


want = [360 * lg(v) for v in range(1, 10)]
got = circle_hits(im, R + 10, green)
w = worst(want, got)
check(w < 0.6, f"03 frame 100: dial ticks 1..9, worst {w:.2f} degrees of 9")
got = circle_hits(im, R + 28, gold)
print(f"note: frame 100 gold tick outside the ring at {got}; 2.4 is {360 * lg(2.4):.1f} degrees")
check(any(abs(g - 360 * lg(2.4)) < 1.5 for g in got), "03 frame 100: gold tick on 2.4")
im = frame("03", 187)
hrows = [(a + b) / 2 for a, b in col_runs(im, 511, purple, 150, 520) if b - a >= 8]
crows = [(a + b) / 2 for a, b in col_runs(im, 930, purple, 150, 520) if b - a >= 8]
print(f"helix dots at y {hrows}; column dots at y {crows}")
check(len(hrows) == 4 and len(crows) == 4, "03 frame 187: four helix dots and four column dots found")
print(f"column dots run {max(abs(a - b) for a, b in zip(hrows, crows)):.1f} px off the helix rows at 720p")

# ---------------------------------------------------------------- lint clearances
print("== clearances behind the lint rows, 720p px")
for n in (138, 172):
    cols = box_cols(frame("02", n), (1180, 235, 1280, 268), teal)
    m = 1279 - max(cols)
    print(f"02 frame {n}: C's 10 ends {m} px from the right edge, {m / 90:.3f} units")
    check(m > 10, f"02 frame {n}: C's 10 fully inside")
im = frame("02", 190)
cols = box_cols(im, (0, 238, 100, 268), teal)
print(f"02 frame 190: leftmost C label ink at x {min(cols)}; C's 2 centred at {D(2) + 720 * (lg(4) - 1):.1f}")
check(min(cols) > 10, "02 frame 190: C's 2 is whole")
im = frame("02", 72)
g = col_runs(im, 750, gold, 200, 420)
lab = box_rows(im, (742, 365, 758, 400), green)
print(f"02 frame 72: hairline y {g[0][0]}..{g[-1][1]}; printed 8 from y {min(lab)}, gap {min(lab) - g[-1][1]} px")
check(min(lab) > g[-1][1], "02 frame 72: hairline stops above D's printed 8")
im = frame("02", 143)
r = col_runs(im, 1093, red, 200, 460)
lab = box_rows(im, (1085, 225, 1103, 272), teal)
print(f"02 frame 143: red line from y {r[0][0]}; C's 6 ends y {max(lab)}, gap {r[0][0] - max(lab)} px")
check(r[0][0] > max(lab), "02 frame 143: red line stops under C's 6")
im = frame("02", 190)
g = col_runs(im, 374, gold, 200, 420)
lab = box_rows(im, (365, 225, 383, 268), teal)
print(f"02 frame 190: hairline from y {g[0][0]}; C's 6 ends y {max(lab)}, gap {g[0][0] - max(lab)} px")
check(g[0][0] > max(lab), "02 frame 190: hairline stops under C's 6")
cols = box_cols(frame("01", 200), (1090, 558, 1280, 584), blue)
print(f"01 frame 200: '19 doublings' ends {1279 - max(cols)} px from the right edge, {(1279 - max(cols)) / 90:.3f} units")
check(1279 - max(cols) > 22, "01 frame 200: old 0.16 unit row cleared")
im = frame("00", 151)
top = box_rows(im, (345, 222, 378, 248), grey)
low = box_rows(im, (345, 240, 378, 272), red)
print(f"00 frame 151: dim 880 ends y {max(top)}; red label from y {min(low)}, gap {min(low) - max(top)} px")
check(min(low) > max(top), "00 frame 151: red label clears the dim 880")
im = frame("03", 144)
dial = box_rows(im, (270, 600, 510, 660), ink)
word = box_rows(im, (270, 660, 510, 700), ink)
print(f"03 frame 144: dial ink ends y {max(dial)}; caption from y {min(word)}, gap {min(word) - max(dial)} px")
check(min(word) > max(dial), "03 frame 144: 'a spiral, and its shadow' clears the dial")

# ---------------------------------------------------------------- moments measured on frames
print("== moments on the frames, seconds in the scratch preview")
ts = times("00")


def first(c, box, pred, frames):
    x0, y0, x1, y1 = box
    for n in frames:
        im = frame(c, n)
        if any(pred(im.getpixel((x, y))) for y in range(y0, y1, 1) for x in range(x0, x1, 2)):
            return n
    return None


n_axis = first("00", (600, 200, 680, 204), grey, range(0, 40))
n_dot = first("00", (1190, 198, 1206, 206), green, range(0, 40))
n_lab = first("00", (1176, 152, 1220, 172), green, range(0, 40))
print(f"00: axis at frame {n_axis} ({ts[n_axis]} s), last dot at frame {n_dot} ({ts[n_dot]} s), 3520 label at frame {n_lab} ({ts[n_lab]} s); voice says 'Mark them on an ordinary ruler' at 19.700 s")
check(ts[n_lab] < 8 < 19.7, "00: the ruler, dots and labels are all up more than 11 s before 'Mark them'")
ts = times("03")
for n in (96, 97, 98, 99, 100, 101, 102):
    im = frame("03", n)
    six = max(max(im.getpixel((x, y))) for x in range(152, 172) for y in range(336, 358))
    word = max(max(im.getpixel((x, y))) for x in range(550, 584) for y in range(566, 590))
    print(f"03 frame {n} {ts[n]} s: peak brightness at dial's 6 {six}, at the gold 2.4 {word}")

# ---------------------------------------------------------------- sentence shares
print("== sentence starts, the kit's character share (estimated_starts) at the scratch length; not Oliver")
script = {}
for line in open(SCRIPT, encoding="utf-8"):
    p = line.rstrip("\n").split("\t")
    if p[0][:2] in CH and "_" in p[0]:
        script[p[0]] = p[-1]


def sentences(text):
    out, start = [], 0
    for i, c in enumerate(text):
        if c in ".?!" and (i + 1 == len(text) or text[i + 1] == " "):
            out.append(text[start : i + 1].strip())
            start = i + 1
    return out


cues = {}
for c, name in CH.items():
    for line in open(f"{RENDER}{name}.cues.tsv", encoding="utf-8"):
        cue, start, length = line.split("\t")
        cues[cue] = (float(start), float(length))
for cue, (start, length) in cues.items():
    parts = sentences(script[cue])
    total = sum(len(s) for s in parts)
    before = 0
    row = []
    for s in parts:
        row.append(f"{start + length * before / total:.1f}")
        before += len(s)
    print(f"{cue} start {start:.3f} length {length:.3f} sentences at {' '.join(row)}")


def share(cue, phrase):
    start, length = cues[cue]
    text = script[cue]
    return start + length * text.index(phrase) / len(text)


print("== phrases, a share of letters over the whole cue; not a measured word time")
for cue, phrase in [
    ("00_2", "Mark them"),
    ("00_4", "go up six semitones"),
    ("00_5", "the square root of two"),
    ("00_6", "the complex numbers"),
    ("01_5", "Closer matches"),
    ("02_1", "one that slides"),
    ("02_2", "slide the moving ruler"),
    ("02_4", "twenty-four"),
    ("03_2", "Six adds"),
    ("03_3", "To read a number back"),
    ("03_3", "Zero laps"),
]:
    print(f"{cue} {phrase!r} at about {share(cue, phrase):.1f} s")

# ---------------------------------------------------------------- static holds
print("== longest runs of selected frames with no visible change: fewer than 4 pixels moving more than 10 levels at quarter size")
print("Between samples nothing is proved. The widest gap between samples is 0.5 s.")
for c in CH:
    ts = times(c)
    prev = None
    start = 0
    runs = []
    for n in range(len(ts)):
        cur = Image.open(f"{FR}{CH[c]}/frame-{n:04d}.png").convert("L").resize((320, 180))
        if prev is not None:
            d = sum(1 for a, b in zip(cur.tobytes(), prev.tobytes()) if abs(a - b) > 10)
            if d >= 4:
                if ts[n - 1] - ts[start] >= 5:
                    runs.append((ts[start], ts[n - 1]))
                start = n
        prev = cur
    if ts[-1] - ts[start] >= 5:
        runs.append((ts[start], ts[-1]))
    print(c, " ".join(f"{a:.1f}-{b:.1f} ({b - a:.1f} s)" for a, b in runs))

# ---------------------------------------------------------------- controls
print("== controls: each planted fault must be noticed")
check("0.51" != f"{lg(3.2):.3f}", "a wrong number is refused")
im = frame("02", 65)
check(worst([D(v) + 720 * lg(2.5) + 4 for v in whole], centres(row_runs(im, 283, teal))) > 1.6, "C shifted 4 px is refused")
check(worst([pos(3) + 4], centres(row_runs(frame("01", 200), 350, gold))) > 1.6, "a marker 4 px off is refused")
check(worst([x + 3 for x in lin[2:]], centres(row_runs(frame("00", 18), 202, green)[1:])) > 1.6, "dots 3 px off are refused")
check(worst([360 * lg(v) + 2 for v in range(1, 10)], circle_hits(frame("03", 100), R + 10, green)) > 0.6, "dial ticks 2 degrees off are refused")

print("RESULT " + ("PASS" if fails == 0 else f"FAIL {fails}"))
sys.exit(1 if fails else 0)
