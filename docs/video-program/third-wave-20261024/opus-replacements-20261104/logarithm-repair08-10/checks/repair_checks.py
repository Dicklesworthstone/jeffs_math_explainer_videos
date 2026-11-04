#!/usr/bin/env python3
"""Lightweight numeric and layout checks for the logarithm 08-10 repair stage.

Run from the repository root:

    python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-repair08-10/checks/repair_checks.py

Reads sources; writes nothing. These are Python mirrors of the crate's arithmetic, not the
crate: nothing here was compiled or rendered, and no frame was seen. Text widths use the
CMR10 table in EMS with width = ems * size * 0.01, or a width measured on preview02 where
one is named. Times follow the scratch voice's rule (words / 2.6 s, sentences by their
share of characters) and illustrative length factors. None of them is an Oliver
measurement. Exit 1 if any check fails.
"""
import hashlib
import math
import sys
from pathlib import Path

BASE = Path("docs/video-program/third-wave-20261024/opus-replacements-20261104")
STAGE = BASE / "logarithm-repair08-10"
CRATE = "explainer_corpus/logarithm_mysteries"
CHAPTERS = f"{CRATE}/src/chapters"
FILES = (
    f"{CHAPTERS}/draw.rs",
    f"{CHAPTERS}/product_rule.rs",
    f"{CHAPTERS}/matrix_logs.rs",
    f"{CHAPTERS}/doing_nothing.rs",
    f"{CRATE}/docs/storyboard.md",
)
SCRIPT = f"{CRATE}/script.tsv"
TAU = math.tau
I2 = [[1.0, 0.0], [0.0, 1.0]]
J = [[0.0, -1.0], [1.0, 0.0]]
failures = []


def check(ok, label):
    print(f"  {'ok  ' if ok else 'FAIL'} {label}")
    if not ok:
        failures.append(label)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def candidate(name):
    return (STAGE / "candidate" / CHAPTERS / name).read_text(encoding="utf-8")


def baseline(name):
    return (STAGE / "baseline" / CHAPTERS / name).read_text(encoding="utf-8")


print("A. Binding: baseline against the canonical tree now, and what the candidate changed")
for f in (*FILES, SCRIPT):
    base, canon = sha(STAGE / "baseline" / f), sha(f)
    cand = STAGE / "candidate" / f
    now = "not in the candidate" if not cand.exists() else ("changed" if sha(cand) != base else "SAME AS BASELINE")
    check(base == canon, f"{f.rsplit('/', 1)[1]}: baseline {base[:16]} equals canonical; candidate {now}")
check(not (STAGE / "candidate" / SCRIPT).exists(), "script.tsv is not in the candidate: no spoken word changes")

print("B. Every finding's words and placements in the candidate source")
EXPECT = (
    ("F1", "draw.rs", "fn card_widths(size: f64, widest: f64)", "dx = size * 0.011;"),
    ("F2", "matrix_logs.rs", "v(DICTIONARY_X, 0.05)", "v(5.4, 0.05)"),
    ("F3", "doing_nothing.rs", "v(TURN_X - RULE_BACK, TURN_Y - 0.7)", "green, v(TURN_X, TURN_Y - 0.7)"),
    ("F3", "doing_nothing.rs", "v(turn_at(TAU)[0] + LAP_ASIDE, TURN_Y - 0.7)", "v(turn_at(TAU)[0], TURN_Y - 0.7)"),
    ("F4", "doing_nothing.rs", "let closed_r = dot(stage, turn_at(PI), 0.09, green);", "&[window, open_l, open_r,"),
    ("F4", "doing_nothing.rs", 'until_phrase(&kit, stage, "10_4", "strictly between")', None),
    ("F5", "draw.rs", "if inside_clip(q, clip) {", "q[0].abs() <= clip && q[1].abs() <= clip"),
    ("F6", "product_rule.rs", "v(zz_at[0] + ZZ_ASIDE, zz_at[1])", "&[], Z.p(zz.scale(1.3)))?"),
    ("F7", "product_rule.rs", "Z.p(C::new(0.0, DEF_HEIGHT))", "Z.p(C::new(0.0, 1.3)),"),
    ("F8", "doing_nothing.rs", "spiral flow: turns {} radians per unit time", "spiral flow: turns {} per unit time"),
    ("F9", "matrix_logs.rs", "let resting = picture(stage.arena_mut(), I2, CENTRE, UNIT, 3);", None),
    ("F10", "doing_nothing.rs", 'phrase_time(&kit, "10_3", "or does not turn")', None),
    ("F11", "matrix_logs.rs", '"J: quarter turn"', '"quarter turn", 22.0'),
    ("F12", "product_rule.rs", "window_shown.push(window_word);", "line.push(window_word);"),
    ("F13", "product_rule.rs", "v(right + 3.0, -1.2)", "v(right + 3.0, -1.0)"),
    ("F14", "matrix_logs.rs", "clipped_grid(s, m, centre, unit, n, CLIP, number_c(), 0.75)", "grid_under(s, m, centre"),
    ("F15", "doing_nothing.rs", "turning rate of the flow of X = imaginary part", "e^(tX) ="),
    ("F16", "doing_nothing.rs", "kit.music_breath(stage, FINE_PRINT_S - read_for)?", None),
    ("F16", "doing_nothing.rs", "fade_out_all(stage, &[spiral_word, stretch_word, zero_word], 0.3)?", None),
    ("F17", "matrix_logs.rs", "stage.scene_mut().remove(&resting_panels);", "stage.add_to_scene(panels[0])?;"),
    ("F17", "doing_nothing.rs", "let squash = stage.add_value_tracker(0.0);", None),
    ("F17", "doing_nothing.rs", "swap(stage, &[numbers_word], &[flows_word, resting_grid], 0.4)?", None),
)
for tag, name, present, absent in EXPECT:
    text = candidate(name)
    ok = present in text and (absent is None or absent not in text)
    check(ok, f"{tag} {name}: {present[:62]}")
check("set_tracker_value(tt, 0.0)" not in candidate("doing_nothing.rs"), "F17 no instant grid jump left in chapter 10")

print("C. Matrix cards as drawn and as repaired (F1 and the chapter 11 effect), scene units")
EMS = {".": 0.278, "−": 0.778, " ": 0.333, "=": 0.778, "A": 0.75, "X": 0.75, "p": 0.556, "b": 0.556, "c": 0.444,
       "q": 0.528, "a": 0.5, "½": 0.75}  # no CMR10 ½: an assumed width, close to a digit and a half


def width(text, size):
    return sum(EMS.get(ch, 0.5) for ch in text) * size * 0.01


def card_widths(size, widest):
    dx = max(size * 0.011, (widest + size * 0.25 / 30.0) / 2.0)
    return dx, dx + max(size * 0.0095, widest / 2.0 + size * 0.10 / 30.0)


def clearances(rows, size, dx, w):
    gaps = [2 * dx - (width(r[0], size) + width(r[1], size)) / 2 for r in rows]
    edge = min(w - (dx + max(width(r[c], size) for r in rows) / 2) for c in (0, 1))
    return min(gaps), edge


CARDS = (
    ("09_1 rotation", 4.4, 34.0, [["0.5", "−0.866"], ["0.866", "0.5"]]),
    ("09_1 stretched", 4.4, 34.0, [["1.6", "0"], ["0", "0.6"]]),
    ("09_1 sheared", 4.4, 34.0, [["1", "1"], ["0", "1"]]),
    ("09_1 flipped", 4.4, 34.0, [["−1", "0"], ["0", "1"]]),
    ("09_2 x_card", 4.6, 30.0, [["0.3", "−1"], ["0.8", "−0.1"]]),
    ("09_2 e_card", 4.6, 30.0, [["0.905", "−0.97"], ["0.776", "0.517"]]),
    ("09_4 target", 5.4, 30.0, [["0", "−1"], ["1", "0"]]),
    ("09_4 pair", 4.9, 26.0, [["a", "−b"], ["b", "a"]]),
    ("10_1 identity", 4.6, 32.0, [["1", "0"], ["0", "1"]]),
    ("11 m_card", 5.4, 32.0, [["−1", "0"], ["0", "1"]]),
    ("11 a_card", 3.65, 30.0, [["−2", "0"], ["0", "−½"]]),
    ("11 x_card", 6.15, 30.0, [["p", "b"], ["c", "q"]]),
    ("11 x_diag", 6.15, 30.0, [["p", "0"], ["0", "q"]]),
    ("11 minus_i", 3.65, 30.0, [["−1", "0"], ["0", "−1"]]),
)
spans = {}
for label, at, size, rows in CARDS:
    widest = max(width(e, size) for r in rows for e in r)
    old = (size * 0.011, size * 0.011 + size * 0.0095)
    new = card_widths(size, widest)
    g0, e0 = clearances(rows, size, *old)
    g1, e1 = clearances(rows, size, *new)
    spans[label] = (at - new[1], at + new[1])
    print(
        f"    {label:15} old gap {g0:+.3f} bracket {e0:+.3f} | new gap {g1:+.3f} bracket {e1:+.3f} | "
        f"half-width {old[1]:.3f} -> {new[1]:.3f}, spans {at - new[1]:.2f} .. {at + new[1]:.2f}"
    )
    check(g1 >= size * 0.25 / 30.0 - 1e-9 and e1 >= size * 0.10 / 30.0 - 1e-9, f"{label}: entries and brackets clear")
print("  what stands beside each card that grew")
x_word_right = 2.6 + width("X, a velocity", 24.0) / 2
check(spans["09_2 x_card"][0] - x_word_right > 0.2, f"09_2: 'X, a velocity' ends {x_word_right:.2f}, x_card starts {spans['09_2 x_card'][0]:.2f}")
check(spans["09_2 e_card"][0] - (2.8 + 0.3) > 0.2, f"09_2: 'e^X =' ends about 3.10, e_card starts {spans['09_2 e_card'][0]:.2f}")
check(spans["09_1 rotation"][0] - (-3.2 + 0.78 * 4.243) > 1.0, f"09_1: the turned grid's right end against the card at {spans['09_1 rotation'][0]:.2f}")
check(max(spans[k][1] for k in ("09_1 rotation", "09_2 e_card", "09_4 target")) < 6.86, "wide cards stay inside the lint limit")
check(5.63 - spans["09_4 pair"][1] > 0.1, f"09_4: pair ends {spans['09_4 pair'][1]:.2f}, the double arrow starts about 5.63")
a_word_right = 2.2 + width("A =", 28.0)
check(spans["11 a_card"][0] - a_word_right > 0.15, f"11: 'A =' ends {a_word_right:.2f}, a_card starts {spans['11 a_card'][0]:.2f}")
check(4.9 - spans["11 a_card"][1] > 0.3, f"11: a_card ends {spans['11 a_card'][1]:.2f}, 'X =' starts 4.90")
check(abs(spans["11 x_card"][1] - 6.765) < 1e-9 and abs(spans["11 x_diag"][1] - 6.765) < 1e-9, "11: x_card and x_diag unchanged")
ruby = BASE / "logarithm-repair11-12" / CHAPTERS / "unreachable.rs"
canon_11 = Path(CHAPTERS, "unreachable.rs").read_text(encoding="utf-8")
if ruby.exists():
    theirs = ruby.read_text(encoding="utf-8")
    sites = lambda t: [ln.strip() for ln in t.splitlines() if "matrix_card(" in ln or "clipped_grid(" in ln]
    added = [s for s in sites(theirs) if s not in sites(canon_11)]
    print(f"    the 11-12 stage at this moment: {len(sites(theirs))} card and grid call lines, canonical {len(sites(canon_11))}; new:")
    for line in added:
        print(f"      {line[:96]}")
    rows = [["−1", "1"], ["0", "−1"]]
    widest = max(width(e, 20.0) for r in rows for e in r)
    dx, w = card_widths(20.0, widest)
    old_w = 20.0 * 0.011 + 20.0 * 0.0095
    print(f"    its size-2 tile (size 20): half-width {old_w:.3f} -> {w:.3f}; its outline half 0.49, its own check wants < 0.44")
    check(w < 0.44, "the 11-12 stage's size-2 tile card still passes that stage's own bound under this patch")
    check("const CLIP: f64 = 3.2;" in theirs, "the 11-12 stage keeps clip 3.2: no grid line lies on its box, so F5's slack changes nothing there")


def mmul(a, b):
    return [
        [a[0][0] * b[0][0] + a[0][1] * b[1][0], a[0][0] * b[0][1] + a[0][1] * b[1][1]],
        [a[1][0] * b[0][0] + a[1][1] * b[1][0], a[1][0] * b[0][1] + a[1][1] * b[1][1]],
    ]


def mscale(a, k):
    return [[a[0][0] * k, a[0][1] * k], [a[1][0] * k, a[1][1] * k]]


def madd(a, b):
    return [[a[0][0] + b[0][0], a[0][1] + b[0][1]], [a[1][0] + b[1][0], a[1][1] + b[1][1]]]


def inverse(a):
    d = a[0][0] * a[1][1] - a[0][1] * a[1][0]
    return [[a[1][1] / d, -a[0][1] / d], [-a[1][0] / d, a[0][0] / d]]


def rotation(t):
    return [[math.cos(t), -math.sin(t)], [math.sin(t), math.cos(t)]]


def apply(m, p):
    return [m[0][0] * p[0] + m[0][1] * p[1], m[1][0] * p[0] + m[1][1] * p[1]]


def mexp(x):
    """model::mexp: scale, 30 Taylor terms, square."""
    norm = max(abs(c) for r in x for c in r)
    s = 0
    while norm / 2.0**s > 0.25:
        s += 1
    small = mscale(x, 1.0 / 2.0**s)
    total, term = I2, I2
    for k in range(1, 31):
        term = mscale(mmul(term, small), 1.0 / k)
        total = madd(total, term)
    for _ in range(s):
        total = mmul(total, total)
    return total


def generator(a, b, tilt_degrees):
    p = mmul(rotation(math.radians(tilt_degrees)), [[a, 0.0], [0.0, b]])
    return mscale(mmul(mmul(p, J), inverse(p)), TAU)


def grid_runs(m, n, clip, slack):
    """draw::clipped_grid: for each line, the samples kept and the extreme x kept."""
    samples = 12 * max(n, 1)
    result = {}
    for k in range(-n, n + 1):
        for vertical in (False, True):
            kept, xs = 0, []
            for j in range(2 * samples + 1):
                t = -n + n * j / samples
                q = apply(m, [float(k), t] if vertical else [t, float(k)])
                if abs(q[0]) <= clip + slack and abs(q[1]) <= clip + slack:
                    kept += 1
                    xs.append(q[0])
            result[(k, vertical)] = (kept, min(xs) if xs else None)
    return result


print("D. Clip tolerance (F5): samples kept on the border lines k = +-3 of a home-coming flow, n = 4, clip 3")
HOME = (
    ("2 pi J", mexp(mscale(J, TAU))),
    ("ellipse 2 by 1", mexp(generator(2.0, 1.0, 0.0))),
    ("ellipse 1.6 by 0.7 tilted 30", mexp(generator(1.6, 0.7, 30.0))),
)
for name, m in HOME:
    old = grid_runs(m, 4, 3.0, 0.0)
    new = grid_runs(m, 4, 3.0, 1e-9)
    border = [(k, vertical) for k in (-3, 3) for vertical in (False, True)]
    o = [old[b][0] for b in border]
    w = [new[b][0] for b in border]
    print(f"    {name}: old {o} new {w}")
    check(all(c == 73 for c in w), f"{name}: all four border lines keep their 73 samples inside the box")
stretch = mexp([[math.log(1.5), 0.0], [0.0, math.log(0.6)]])
print(f"    stretch flow at t = 1: x = 2 lands at {2 * stretch[0][0]!r}")
check(abs(2 * stretch[0][0]) <= 3.0 + 1e-9, "stretch flow: the line x = 2 stays drawn at t = 1")
check(not (3.001 <= 3.0 + 1e-9), "a point at 3.001 stays out: the tolerance hides no real motion")
old_11 = grid_runs(I2, 4, 3.2, 0.0)
new_11 = grid_runs(I2, 4, 3.2, 1e-9)
check(old_11 == new_11, "chapter 11 (clip 3.2, identity grid): identical to the old test, sample for sample")
print("    chapter 11's flip and spin grids at clip 3.2: no line lies on the box, so the slack only matters within 1e-9 of it")

print("E. Clip in chapter 09 (F14): CENTRE -3.2, UNIT 0.78, CLIP 4.4, n = 3")
EXAMPLES = (
    ("turned 60", rotation(TAU / 6)),
    ("stretched", [[1.6, 0.0], [0.0, 0.6]]),
    ("sheared", [[1.0, 1.0], [0.0, 1.0]]),
    ("flipped", [[-1.0, 0.0], [0.0, 1.0]]),
    ("e^X", mexp([[0.3, -1.0], [0.8, -0.1]])),
)
for name, m in EXAMPLES:
    before = min(apply(m, [float(a), float(b)])[0] for a in (-3, 3) for b in (-3, 3))
    runs = grid_runs(m, 3, 4.4, 1e-9)
    left = min(x for _, x in runs.values() if x is not None)
    lost = sum(1 for kept, _ in runs.values() if kept == 0)
    cut = sum(1 for kept, _ in runs.values() if kept < 73)
    print(
        f"    {name}: left edge scene {-3.2 + 0.78 * before:+.2f} -> {-3.2 + 0.78 * left:+.2f}; "
        f"{cut} of 14 lines shortened, {lost} gone"
    )
    check(-3.2 + 0.78 * left > -6.86, f"{name}: inside the lint limit")
turned = grid_runs(rotation(TAU / 6), 3, 4.4, 1e-9)
check(all(kept == 73 for kept, _ in turned.values()), "a turned n = 3 grid loses nothing")
for rate in (0.25, 1.25, -0.75):
    for t in (0.0, 0.31, 0.5, 0.77, 1.0):
        runs = grid_runs(mexp(mscale(J, TAU * rate * t)), 2, 4.4, 1e-9)
        if not all(kept == 49 for kept, _ in runs.values()):
            check(False, f"small panel rate {rate} t {t} was cut")
check(True, "the three small panels lose nothing at any sampled moment of the spin")

print("F. Label spans (F2, F3, F6, F7, F8, F13), measured widths from preview02 where named")
dictionary = 7.26 - 3.54  # lint span at x = 5.4
d_left, d_right = 4.95 - dictionary / 2, 4.95 + dictionary / 2
print(f"    F2 dictionary {d_left:.2f} .. {d_right:.2f}; panel 3 rests with right edge {1.6 + 2 * 0.48:.2f}")
check(d_right < 6.86 and d_left - (1.6 + 2 * 0.48) > 0.3, "F2 inside the lint limit and clear of panel 3")
rule_w, lap_w = 5.14 - 3.10, 6.56 - 5.03
rule = (4.1 - 0.25 - rule_w / 2, 4.1 - 0.25 + rule_w / 2)
lap = (4.1 + 0.85 * 2 + 0.15 - lap_w / 2, 4.1 + 0.85 * 2 + 0.15 + lap_w / 2)
print(f"    F3 rule {rule[0]:.2f} .. {rule[1]:.2f}; lap label {lap[0]:.2f} .. {lap[1]:.2f}; gap {lap[0] - rule[1]:.3f}")
check(lap[0] - rule[1] > 0.25 and lap[1] < 6.86, "F3 labels apart and inside the lint limit")
check(0.36 - 0.15 > 0.15, "F6 z^2 label's left edge is 0.21 right of the axis (half-width 0.15 read from frame 130)")
axis_top, def_y = 0.1 + 2.2, 0.1 + 1.55 * 1.66
print(f"    F7 axis top {axis_top:.2f}; definition centred {def_y:.3f}, extent {def_y - 0.19:.3f} .. {def_y + 0.19:.3f}")
check(def_y - 0.19 - axis_top > 0.15 and def_y + 0.19 < 3.0, "F7 above the axis's end and under the title")
extra = width("radians ", 22.0)
check(5.84 + extra < 6.86, f"F8 right edge 5.84 (frame 200) + {extra:.2f} = {5.84 + extra:.2f}, inside the lint limit")
print("    F13: the product line moves 0.2 down, 18 native px, so its 8-9 px under '= sqrt 1 = 1' becomes about 26 (estimate)")

print("G. Chapter 10's rows after the flows' words leave (F16), scene y")
rows = [("rule and lap label", 0.85), ("cond_a", 0.25), ("cond_b", -0.2), ("name", -0.75), ("log I = 0", -1.5), ("textbook", -3.15), ("precisely", -3.62)]
for (a, ya), (b, yb) in zip(rows, rows[1:]):
    print(f"    {a} {ya:+.2f} to {b} {yb:+.2f}: {ya - yb:.2f}")
    check(ya - yb >= 0.45, f"{a} / {b} rows keep apart")
old_precisely = "no eigenvalue of A in (−∞, 0] ⇒ one X with e^X = A, all eigenvalues −π < Im < π; real if A is (Higham; Gallier Thm 3.11)"
new_precisely = "no eigenvalue of A in (−∞, 0] ⇒ one logarithm has all eigenvalues −π < Im < π; real if A is (Higham; Gallier Thm 3.11)"
textbook = "turning rate of the flow of X = imaginary part of an eigenvalue of X"
text_d = candidate("doing_nothing.rs")
check(new_precisely in text_d and textbook in text_d, "the new fine print is in the source, word for word")
check("^" not in new_precisely and "^" not in textbook, "F15 no literal caret on screen in either line")
print(f"    precisely {len(old_precisely)} -> {len(new_precisely)} characters; ems {sum(EMS.get(c, 0.5) for c in old_precisely):.1f} -> {sum(EMS.get(c, 0.5) for c in new_precisely):.1f}")
check(sum(EMS.get(c, 0.5) for c in new_precisely) <= sum(EMS.get(c, 0.5) for c in old_precisely) + 2, "F15 no wider than the line that fitted, within 2 ems")
reading = len(textbook) + len("precisely: ") + len(new_precisely)
print(f"    {reading} characters at 15 a second: {reading / 15:.2f} s; FINE_PRINT_S 13.2")
check(reading / 13.2 <= 15.0, "F16 the fine print's hold meets the reading floor")

print("H. Sentence and phrase timing, line-relative seconds. Illustrative only: no Oliver data")
LINES = {}
for raw in (STAGE / "baseline" / SCRIPT).read_text(encoding="utf-8").splitlines():
    if "\t" in raw and not raw.startswith("#"):
        i, text = raw.split("\t", 1)
        LINES[i] = text


def sentences(text):
    out, start = [], 0
    for i, ch in enumerate(text):
        if ch in ".?!" and (i + 1 == len(text) or text[i + 1] == " "):
            part = text[start : i + 1].strip()
            if part:
                out.append(part)
            start = i + 1
    rest = text[start:].strip()
    if rest:
        out.append(rest)
    return out


def estimated_starts(text, length):
    parts = sentences(text)
    total = max(sum(len(p) for p in parts), 1)
    before, starts = 0, []
    for p in parts:
        starts.append(length * before / total)
        before += len(p)
    return starts


def sentence_offsets(text):
    at = [0]
    for i in range(len(text.encode()) - 1):
        b = text.encode()
        if b[i] in b".?!" and b[i + 1] == 32:
            at.append(i + 2)
    return at


def phrase_seconds(text, starts, length, marks, phrase):
    raw = text.encode()
    byte = raw.find(phrase.encode())
    if byte < 0:
        return None
    if byte in marks:
        return marks[byte]
    offsets = sentence_offsets(text)
    if len(offsets) != len(starts):
        return None
    j = max(i for i, at in enumerate(offsets) if at <= byte)
    start, end = offsets[j], offsets[j + 1] if j + 1 < len(offsets) else len(raw)
    t0, t1 = starts[j], starts[j + 1] if j + 1 < len(starts) else length
    return t0 + (t1 - t0) * (byte - start) / max(end - start, 1)


def run_to(now, at, margin, fallback):
    return fallback if at is None else max(at - now - margin, 1.0)


for i in ("10_2", "10_3", "10_4", "09_2", "09_4"):
    check(len(sentence_offsets(LINES[i])) == len(sentences(LINES[i])), f"{i}: the phrase timer and the narrator split it alike ({len(sentences(LINES[i]))} sentences)")
for i, phrase in (("10_3", "turns round at a steady"), ("10_3", "or does not turn"), ("10_4", "strictly between")):
    check(phrase in LINES[i], f"{i} holds the phrase {phrase!r}")

for factor in (0.8, 1.0, 1.25):
    words = len(LINES["10_3"].split())
    speech = words / 2.6 * factor
    s = estimated_starts(LINES["10_3"], speech)
    end = speech + 0.4
    turns = phrase_seconds(LINES["10_3"], s, end, {}, "turns round at a steady")
    still = phrase_seconds(LINES["10_3"], s, end, {}, "or does not turn")
    t = 0.5 + 0.4
    t = max(t, s[1]) + 0.4 + 0.5
    t = max(t, s[2]) + 0.4
    t += min(run_to(t, turns, 0.4 + 0.5, 2.6), 2.6)
    late = t - s[3]
    t = max(t, s[3]) + 0.4 + 0.5
    t = max(t, turns)
    sweep_from = t
    t += run_to(t, still, 0.5, 3.4)
    sweep_to = t
    t += 0.2 + 0.3
    t = max(t, still)
    word_at = t
    t += 0.3
    t += run_to(t, s[4], 0.2, 2.2)
    stretch_to = t
    t = max(t, s[4]) + 0.3 + 0.5 + 0.4
    t = max(t, s[5]) + 0.5
    print(
        f"    10_3 x{factor}: S3 {s[3]:.1f}, 'turns round' {turns:.1f}, 'or does not turn' {still:.1f}, S4 {s[4]:.1f}, S5 {s[5]:.1f}, end {end:.1f}\n"
        f"      S2 run ends {late:+.1f} past S3; spiral sweeps {sweep_from:.1f} .. {sweep_to:.1f}; stretch word {word_at:.1f}, runs to {stretch_to:.1f}; beats end {t:.1f}"
    )
    check(sweep_from <= turns + 0.25, f"10_3 x{factor}: the sweep starts within 0.25 s of 'turns round'")
    check(sweep_to <= still, f"10_3 x{factor}: nothing says 'does not turn' while the spiral sweeps")
    check(word_at <= still + 0.25, f"10_3 x{factor}: 'stretch flow' lands within 0.25 s of 'or does not turn'")
    check(stretch_to <= s[4] + 0.3, f"10_3 x{factor}: the stretch run is over by 'The zero flow'")
    check(t <= end + 0.05, f"10_3 x{factor}: the line's beats end with the line ({t:.1f} of {end:.1f})")

    words = len(LINES["10_4"].split())
    speech = words / 2.6 * factor
    s = estimated_starts(LINES["10_4"], speech)
    end = speech + 0.4
    strictly = phrase_seconds(LINES["10_4"], s, end, {}, "strictly between")
    t = max(0.4, strictly) + 0.3 + 0.4
    check(t <= s[1], f"10_4 x{factor}: the +pi end opens ({strictly:.1f}) and crosses end {t:.1f} before S1 {s[1]:.1f}")
    shown = s[4] + 0.5
    read = end - shown
    breath = max(0.0, 13.2 - read)
    print(f"    10_4 x{factor}: fine print up {shown:.1f}, voice ends {end:.1f}, read {read:.1f} s, added end hold {breath:.1f} s")

    speech = len(LINES["10_2"].split()) / 2.6 * factor
    s = estimated_starts(LINES["10_2"], speech)
    check(0.4 + 1.1 + 0.3 <= s[1] + 0.25, f"10_2 x{factor}: rings squash and e0 land ({1.8:.1f} s) by S1 {s[1]:.2f} + 0.25")

    speech = len(LINES["09_2"].split()) / 2.6 * factor
    s = estimated_starts(LINES["09_2"], speech)
    check(s[2] - s[1] >= 0.5, f"09_2 x{factor}: the resting plane's fade fits sentence 1 ({s[2] - s[1]:.2f} s)")

print("I. Picture continuity at each static-to-live change (F9, F17)")
for rate in (0.25, 1.25, -0.75):
    check(mexp(mscale(mscale(J, TAU * rate), 0.0)) == I2, f"09_4 rate {rate}: the live panel at spin 0 is the identity exactly")
check(mexp(mscale([[0.3, -1.0], [0.8, -0.1]], 0.0)) == I2, "09_2: the live flow at time 0 is the identity exactly")
check(mexp(mscale([[math.log(1.5), 0.0], [0.0, math.log(0.6)]], 0.0)) == I2, "10_3: the live grid at time 0 is the identity exactly")


def squashed_point(a, b, tilt, u, r, th):
    short = 1.0 + (b / a - 1.0) * u
    return apply(rotation(math.radians(tilt)), [r * math.cos(th), short * r * math.sin(th)])


def old_ring(a, b, tilt, r, th):
    return apply(mmul(rotation(math.radians(tilt)), [[a, 0.0], [0.0, b]]), [r * math.cos(th) / a, r * math.sin(th) / a])


for a, b, tilt in ((2.0, 1.0, 0.0), (1.6, 0.7, 30.0)):
    worst_end = worst_circle = 0.0
    for r in (0.5, 1.0, 1.5, 2.0):
        for k in range(81):
            th = TAU * k / 80
            p, q = squashed_point(a, b, tilt, 1.0, r, th), old_ring(a, b, tilt, r, th)
            worst_end = max(worst_end, abs(p[0] - q[0]), abs(p[1] - q[1]))
            c = squashed_point(a, b, tilt, 0.0, r, th)
            worst_circle = max(worst_circle, abs(math.hypot(*c) - r))
    check(worst_end < 1e-12, f"F17 ellipse {a} by {b}: at u = 1 the rings are the old orbit (worst {worst_end:.1e})")
    check(worst_circle < 1e-12, f"F17 ellipse {a} by {b}: at u = 0 they are circles (worst {worst_circle:.1e})")

print("J. The new Rust tests' numbers, prototyped (they have not run)")
check(abs(3.0000000000000018) <= 3.0 + 1e-9 and not abs(3.001) <= 3.0 + 1e-9, "draw: a_line_on_the_clip_box_survives_a_flows_rounding")
for size, widest in ((34.0, 1.039), (30.0, 0.767), (30.0, 0.617), (26.0, 0.332), (32.0, 0.16)):
    dx, w = card_widths(size, widest)
    check(2 * dx - widest >= size * 0.25 / 30.0 - 1e-12 and w - (dx + widest / 2) >= size * 0.10 / 30.0 - 1e-12, f"draw: card_columns_and_brackets at ({size}, {widest})")
dx, w = card_widths(20.0, 0.256)
check(abs(w - (20.0 * 0.011 + 20.0 * 0.0095)) < 0.01, "draw: a_small_card_grows_only_as_its_text_does")
dx, w = card_widths(32.0, 0.16)
check(dx == 32.0 * 0.011 and w == 32.0 * 0.011 + 32.0 * 0.0095, "draw: narrow_entries_keep_the_cards_old_spacing (exact)")
check(-3.2 - 0.78 * 4.4 > -6.86 and -3.2 + 0.78 * 4.4 < 1.0 and math.sqrt(18) < 4.4, "matrix_logs: the_clip_holds_the_grid")
check(d_right < 6.86 and d_left > 1.6 + 0.96 + 0.3, "matrix_logs: the_dictionary_line_stays_in_the_frame")
check(math.atan2(0.0, -1.0) == math.pi, "doing_nothing: a_numbers_window_closes_at_plus_pi")
text = LINES["10_3"]
shares = [30.0 * at / len(text) for at in sentence_offsets(text)]
turns = phrase_seconds(text, shares, 30.0, {}, "turns round at a steady")
still = phrase_seconds(text, shares, 30.0, {}, "or does not turn")
check(len(shares) == 6 and shares[3] < turns < still < shares[4], "doing_nothing: every_timed_phrase, 10_3")
text = LINES["10_4"]
shares = [30.0 * at / len(text) for at in sentence_offsets(text)]
check(0.0 < phrase_seconds(text, shares, 30.0, {}, "strictly between") < shares[1], "doing_nothing: every_timed_phrase, 10_4")
text = "One two three. Four five."
byte = text.find("five")
check(phrase_seconds(text, [0.0, 2.0], 4.0, {byte: 3.25}, "five") == 3.25, "doing_nothing: a_takes_word_mark_beats_the_estimate")
check(2.0 < phrase_seconds(text, [0.0, 2.0], 4.0, {}, "five") < 4.0, "doing_nothing: the estimate falls in its sentence")
check(phrase_seconds(text, [0.0], 4.0, {}, "five") is None and phrase_seconds(text, [0.0, 2.0], 4.0, {}, "six") is None, "doing_nothing: a bad phrase or split is refused")
check(abs(run_to(10.0, 15.0, 0.5, 9.0) - 4.5) < 1e-12 and run_to(10.0, 10.2, 0.5, 9.0) == 1.0 and run_to(10.0, None, 0.5, 9.0) == 9.0, "doing_nothing: a_run_ends_before_its_mark")
check(lap[0] - rule[1] > 0.25 and lap[1] < 6.86, "doing_nothing: the_labels_under_the_turning_line_keep_apart")
check(reading / 13.2 <= 15.0, "doing_nothing: the_fine_print_has_no_caret_and_time_to_be_read")
check(0.36 - 0.15 > 0.15 and def_y - 0.19 > axis_top + 0.15 and def_y + 0.19 < 3.2 - 0.2, "product_rule: the_axis_runs_through_neither_label")

print()
if failures:
    print(f"{len(failures)} FAILED")
    for f in failures:
        print(f"  {f}")
    sys.exit(1)
print("all checks passed; these are source-level mirrors, not a build or a render")
