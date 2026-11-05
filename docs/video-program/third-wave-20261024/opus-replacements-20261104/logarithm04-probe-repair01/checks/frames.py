"""Measure chapter 04's existing frames and test the new probe path against that ink. Read only.

Run from the repo root:  python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm04-probe-repair01/checks/frames.py

The frames are root's stage06 preview at 720p with the scratch voice. They show the OLD path. The new path is
a model on top of them: this says where the new line would lie against ink that exists, not what a new render shows.
"""
import json
import sys

from PIL import Image

F = "narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-stage06-frames01/04_undoing_growth/"
times = json.load(open(F + "sampling.json", encoding="utf-8"))["times"]
OX, OY = -3.2, -0.4
HALF_LINE_PX = 0.0125 * 90
START, REST = 2.16, 1.5
OLD_START, OLD_REST = 2.6, 2.0
fails = 0


def check(ok, what):
    global fails
    fails += 0 if ok else 1
    print(("PASS " if ok else "FAIL ") + what)


def load(n):
    im = Image.open(F + f"frame-{n:04d}.png").convert("RGB")
    return im.size[0], im.tobytes()


def px(x):
    return 640 + 90 * (x + OX)


def py(y):
    return 360 - 90 * (y + OY)


def gy(row):
    return (360 - row) / 90 - OY


def box(frame, x0, x1, r0, r1, test):
    w, d = frame
    xs, rs = [], []
    for r in range(r0, r1 + 1):
        for x in range(x0, x1 + 1):
            i = 3 * (r * w + x)
            if test(d[i], d[i + 1], d[i + 2]):
                xs.append(x)
                rs.append(r)
    return (min(xs), max(xs), min(rs), max(rs)) if xs else None


bright = lambda r, g, b: max(r, g, b) > 70
green = lambda r, g, b: g > 90 and g > r + 25 and g > b
gold = lambda r, g, b: r > 200 and g > 170 and b < 150

print("== the ink the line must miss, measured on frame 121 (54.6 s) and frame 228 (100.0 s)")
f121 = load(121)
name = box(f121, 450, 560, 150, 200, green)
print("name y = e^x: columns", name[0], name[1], "rows", name[2], name[3], f"graph y {gy(name[3] + 1):.3f} to {gy(name[2]):.3f}")
labels = {}
for t in (3, 2, 1, -1, -2):
    c = round(py(t))
    labels[t] = box(f121, 318, 331, c - 12, c + 12, bright)
    print(f"y tick label {t}: rows {labels[t][2]} to {labels[t][3]}")
x_row = round(py(-0.3))
# column 352 is the y axis, so the box stops short of it; the labels 1, 2 and 3 are the same size as these
x_labels = box(f121, 150, 335, x_row - 12, x_row + 12, bright)
print(f"x tick labels: rows {x_labels[2]} to {x_labels[3]}")

print("\n== the new start and rest against that ink")
for what, c, above, below in (("start", START, name[3], labels[2][2]), ("rest", REST, labels[2][3], labels[1][2])):
    r = py(c)
    gap_up, gap_down = (r - HALF_LINE_PX) - (above + 1), below - (r + HALF_LINE_PX)
    print(f"{what} {c}: row {r:.1f}, clear above {gap_up:.1f} px, clear below {gap_down:.1f} px")
    check(gap_up > 5 and gap_down > 5, f"{what} leaves at least 5 px on both sides")
old_rest = py(OLD_REST)
check(labels[2][2] <= old_rest <= labels[2][3], f"control: the old rest, row {old_rest:.1f}, is inside the label 2's rows")
old_start, old_low = py(OLD_START), py(0.15)
check(old_start < name[2] and name[3] < old_low, f"control: the old sweep, rows {old_start:.1f} to {old_low:.1f}, passes through the name's rows")
check(py(START) - HALF_LINE_PX > name[3] + 1, "the new top of the path is below the name's lowest ink")

print("\n== what else lies on the two new rows, frame 124 (54.9 s; its line is at 2.0, away from both rows)")
f124 = load(124)
w, d = f124
for what, c in (("start", START), ("rest", REST)):
    r0, r1 = int(py(c) - 3), int(py(c) + 3)
    cols = sorted({x for r in range(r0, r1 + 1) for x in range(116, 623) if bright(*d[3 * (r * w + x) : 3 * (r * w + x) + 3])})
    runs, s = [], None
    for a, b in zip(cols, cols[1:] + [None]):
        s = a if s is None else s
        if b != a + 1:
            runs.append((s, a))
            s = None
    print(f"{what} rows {r0}-{r1}: ink columns {runs}")
    check(len(runs) == 2, f"{what}: only two things cross the row, the axis at column 352 and the curve")
    check(any(a <= 352 <= b for a, b in runs), f"{what}: one of them is the axis")

print("\n== the path model, tested on the old frames")


def smooth(t):
    t = min(max(t, 0.0), 1.0)
    return t * t * t * (10 - 15 * t + 6 * t * t)


LEGS = (2.4, 1.4, 1.6)


def height(t, start, rest, end):
    t0 = end - sum(LEGS)
    points = (start, 0.15, -1.2, rest)
    for k, run in enumerate(LEGS):
        if t <= t0 + run:
            return points[k] + (points[k + 1] - points[k]) * smooth((t - t0) / run)
        t0 += run
    return rest


measured = {}
for n, t in enumerate(times):
    if not 49.0 <= t <= 54.95:
        continue
    w, d = load(n)
    found = [r for r in range(100, 640) if gold(*d[3 * (r * w + 200) : 3 * (r * w + 200) + 3]) and gold(*d[3 * (r * w + 600) : 3 * (r * w + 600) + 3])]
    measured[n] = gy(sum(found) / len(found)) if found else None
# below zero the line is red, so those frames give no reading and do not take part in the fit
used = {n: m for n, m in measured.items() if m is not None and 49.4 <= times[n] <= 54.8}
fit = min((max(abs(m - height(times[n], OLD_START, OLD_REST, e / 1000)) for n, m in used.items()), e / 1000) for e in range(54600, 54951))
worst, END = fit
print(f"fitted end of the sweep {END:.3f} s, {54.867 - END:.3f} s before 04_3's listed start; worst error {worst:.3f} graph units over {len(used)} frames")
for n, m in measured.items():
    print(f"frame {n} {times[n]:.3f} s: old line measured {m if m is None else round(m, 3)}, modelled {height(times[n], OLD_START, OLD_REST, END):.3f}; new line {height(times[n], START, REST, END):.3f}")
check(worst < 0.06, f"smootherstep legs of 2.4, 1.4, 1.6 s reproduce the old line within {worst:.3f} graph units")
new_top = max(height(times[n], START, REST, END) for n in measured)
check(new_top <= START + 1e-9, f"highest new height over these frames {new_top:.3f}")

print("\n== moving crossings the new path keeps, disclosed, on a 0.01 s grid")
bands = {"label 2": labels[2], "label 1": labels[1], "label -1": labels[-1], "x labels": x_labels, "label -2": labels[-2]}
t0 = END - sum(LEGS)
for what, b in bands.items():
    hits, t = [], t0
    while t <= END:
        r = py(height(t, START, REST, END))
        if r + HALF_LINE_PX >= b[2] and r - HALF_LINE_PX <= b[3] + 1:
            hits.append(t)
        t += 0.01
    spans = []
    for a in hits:
        if spans and a - spans[-1][1] < 0.02:
            spans[-1][1] = a
        else:
            spans.append([a, a])
    print(what, [f"{a:.2f}-{z:.2f} s ({z - a:.2f} s)" for a, z in spans] or "not crossed")
rest_hits = [what for what, b in bands.items() if py(REST) + HALF_LINE_PX >= b[2] and py(REST) - HALF_LINE_PX <= b[3] + 1]
check(not rest_hits, f"at rest the line touches no tick label: {rest_hits}")

print("\n== the caption on frame 228, 100.0 s")
f228 = load(228)
line = box(f228, 720, 1270, 465, 490, lambda r, g, b: r > 150 and g > 120 and b < 150)
print("old caption ink: columns", line[0], line[1], "rows", line[2], line[3])
EMS = {"a": 0.5, "b": 0.556, "o": 0.5, "u": 0.556, "t": 0.389, " ": 0.333}
added = sum(EMS[ch] for ch in "about ") * 24 * 0.01 * 90
right = line[1] + added
print(f"'about ' adds {added:.0f} px at 24 points, so the caption ends near column {right:.0f}; CMR10 ems, not measured on a new frame")
check(right < 1280 - 0.14 * 90 - 40, "the longer caption stays well inside the safe edge")
shown = []
for n, t in enumerate(times):
    if 95 <= t <= 106:
        w, d = load(n)
        ink = sum(1 for r in range(468, 486) for x in range(730, 993) if d[3 * (r * w + x)] > 150 and d[3 * (r * w + x) + 2] < 150)
        if ink > 300:
            shown.append(n)
print(f"caption up on frames {shown[0]} ({times[shown[0]]} s) to {shown[-1]} ({times[shown[-1]]} s), sheets {shown[0] // 4} to {shown[-1] // 4}")
for n in (106, 107, 108, 122, 123, 124, 127):
    print(f"frame {n}: {times[n]} s, sheet {n // 4}")
print("RESULT " + ("PASS" if fails == 0 else f"FAIL {fails}"))
sys.exit(1 if fails else 0)
