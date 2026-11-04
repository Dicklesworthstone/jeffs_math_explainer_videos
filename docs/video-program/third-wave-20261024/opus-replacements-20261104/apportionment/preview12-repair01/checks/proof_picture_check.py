#!/usr/bin/env python3
"""Chapter 12's supposition picture, checked outside Rust with exact fractions.

Run from the repository root:  python3 -I -B <stage>/checks/proof_picture_check.py

Five parts, each independent of the crate's model.rs:
  1. the run, by the tiles (12_7's statement of the rule), against the oracle
  2. the facts: every old oracle value unchanged, exactly 14 new ones
  3. what the picture holds before, during and after the supposition
  4. the counts on screen, and where each tile goes
  5. layout boxes per state, and a timing simulation from the source's own beats

Negative controls prove parts 3 and 5 can fail. Text widths are estimates from
measured frames, and times are simulated from a scratch length: neither is a
render, a listen or an acceptance.
"""

import re
import subprocess
import sys
from fractions import Fraction as F
from pathlib import Path

STAGE = Path(__file__).resolve().parent.parent
CAND = STAGE / "candidate/explainer_corpus/apportionment"
BASE = STAGE / "base/explainer_corpus/apportionment"
CHAPTER = CAND / "src/chapters/both_promises.rs"

POP = (4000, 9000, 26000)
P = sum(POP)
ASH, BIRCH, CEDAR = 0, 1, 2
KEYS = ("ash", "birch", "cedar")
THEN, NOW = 6, 13

failures = []


def check(ok, what):
    print(("  ok   " if ok else "  FAIL ") + what)
    if not ok:
        failures.append(what)


def tile(town, k):
    """Seat k's tile: where the town's share reaches k-1, and k."""
    return (F((k - 1) * P, POP[town]), F(k * P, POP[town]))


def walk(houses):
    """At each house size, of the entered tiles nobody holds yet, take the one
    that ends soonest. Returns the seats after each house, and who got each."""
    seats = [0, 0, 0]
    history = {0: tuple(seats)}
    handed = {}
    for h in range(1, houses + 1):
        entered = []
        for town in range(3):
            start, end = tile(town, seats[town] + 1)
            if start < h:
                entered.append((end, town))
        entered.sort()
        if len(entered) > 1 and entered[0][0] == entered[1][0]:
            raise SystemExit(f"a tie at {h}: this check assumes none")
        town = entered[0][1]
        seats[town] += 1
        history[h] = tuple(seats)
        handed[h] = (town, seats[town])
    return history, handed


def held(seats):
    return {(i, k) for i, n in enumerate(seats) for k in range(1, n + 1)}


def is_run(tiles):
    """Seats go out one at a time, so a town's seats have no hole."""
    return all(k == 1 or (i, k - 1) in tiles for i, k in tiles)


def oracle(root):
    out = subprocess.run(
        [sys.executable, "-I", "-B", str(root / "oracle.py")],
        capture_output=True, text=True, check=True,
    ).stdout
    return dict(line.split("\t", 1) for line in out.splitlines() if "\t" in line)


def mixed(x):
    whole, rest = int(x), x - int(x)
    if rest == 0:
        return str(whole)
    return f"{whole} {rest.numerator}/{rest.denominator}" if whole else f"{rest.numerator}/{rest.denominator}"


history, handed = walk(16)

print("1. the run, by the tiles")
check(history[THEN] == (0, 2, 4), f"seats at {THEN}: {history[THEN]}")
check(history[NOW] == (1, 3, 9), f"seats at {NOW}: {history[NOW]}")
later = handed[THEN]
check(later == (BIRCH, 2), f"handed out at {THEN}: {KEYS[later[0]]} {later[1]}")
later_end = tile(*later)[1]
check(tile(*later)[0] < THEN, "that tile was already entered at then")
waiting = [
    (i, history[THEN][i] + 1) for i in range(3)
    if tile(i, history[THEN][i] + 1)[0] >= THEN
    and tile(i, history[THEN][i] + 1)[1] < later_end
    and tile(i, history[THEN][i] + 1)[1] <= NOW
]
check(waiting == [(CEDAR, 5)], f"the only tile that can be the missing one: {waiting}")
missing = waiting[0]
m_start, m_end = tile(*missing)
check(m_start == THEN, "the missing tile starts exactly at then, so it is not entered there")
check(m_end < later_end, "and ends sooner than the tile the rule really took")
check(m_end <= NOW, "and is passed by now")
after = [handed[h] for h in range(THEN + 1, NOW + 1)]
check(after == [(2, 5), (2, 6), (0, 1), (2, 7), (2, 8), (1, 3), (2, 9)], f"seats {THEN + 1} to {NOW}: {after}")

print("2. the facts")
old, new = oracle(BASE), oracle(CAND)
check(len(old) == 429, f"the base oracle has {len(old)} values")
changed = [k for k in old if new.get(k) != old[k]]
check(not changed, f"no old value changed ({len(changed)})")
added = {k: new[k] for k in new if k not in old}
want = {
    "stretch.need": str(NOW - THEN + 1),
    "stretch.later.town": KEYS[later[0]],
    "stretch.later.seat": str(later[1]),
    "stretch.later.end": mixed(later_end),
    "stretch.missing.town": KEYS[missing[0]],
    "stretch.missing.seat": str(missing[1]),
    "stretch.missing.start": mixed(m_start),
    "stretch.missing.end": mixed(m_end),
}
for i, key in enumerate(KEYS):
    want[f"stretch.then.{key}"] = str(history[THEN][i])
    want[f"stretch.now.{key}"] = str(history[NOW][i])
check(added == want, f"the {len(added)} new values equal this walk's")
check(len(new) == 443, f"the candidate oracle has {len(new)} values")
for h in (14, 15, 16):
    seats_here = tuple(int(new[f"quotamethod.{h}.{key}"]) for key in KEYS)
    check(seats_here == history[h], f"the oracle's rule at {h} equals this walk: {seats_here}")

print("3. before, during and after the supposition")
before = held(history[NOW])
during = held(history[THEN])
restored = during | set(after)
check(len(before) == 13 and is_run(before), "before: the real run at 13, 13 fills, no hole")
check(len(during) == 6 and is_run(during), "during: the real run at 6, 6 fills, no hole")
check(restored == before and is_run(restored), "after: the same 13 fills")
check(not (during & set(after)), "no fill during the supposition was handed out after then")
check(missing in set(after), "the supposition is false of the real run: the missing tile is really handed out")
check(after[0] == missing, "indeed the very next seat, as 12_2's one skip says")
# negative controls: the old picture
old_state = before - {(CEDAR, 8)}
check(not is_run(old_state), "control: the old picture (13 fills less Cedar 8) is no run")
crossing = [t for t in old_state if tile(*t)[0] < THEN < tile(*t)[1] and t in after]
check(crossing == [(ASH, 1)], f"control: in it Ash 1 was handed out after then yet crosses then: {crossing}")
past = [t for t in old_state if tile(*t)[1] > NOW]
check(past == [(CEDAR, 9)], f"control: in it Cedar 9 runs past now: {past}")
c8_end = tile(CEDAR, 8)[1]
entered_at_then = [
    (i, k) for i in range(3) for k in range(1, 14)
    if tile(i, k)[0] < THEN and tile(i, k)[1] > c8_end and k <= history[THEN][i]
]
check(not entered_at_then, "control: with Cedar 8 missing, no tile handed out by 6 ends later, so then = 6 cannot follow")

print("4. the counts on screen, and where each tile goes")
tiles = []
for i in range(3):
    k = 1
    while tile(i, k)[0] < F(39, 2):
        tiles.append((i, k))
        k += 1
check(len(tiles) == 20, f"tiles on the axis: {len(tiles)}")
inside = {t for t in tiles if tile(*t)[0] >= THEN and tile(*t)[1] <= NOW}
per_town = [sum(1 for t in inside if t[0] == i) for i in range(3)]
check(per_town == [0, 1, 4], f"whole tiles inside, town by town: {per_town}")
check(new["stretch.inside.total"] == str(sum(per_town)), "their total equals the fact")
rings = inside - {missing}
check(len(rings) + 1 == sum(per_town), "hero rings plus the red one count those tiles")
seats_n = NOW - THEN
for i in range(3):
    check(F(per_town[i]) <= F(seats_n * POP[i], P), f"{KEYS[i]}: {per_town[i]} <= {seats_n} x {POP[i]}/{P}")
chips_n = len(after) + 1
check(chips_n == seats_n + 1 and sum(per_town) < chips_n, f"{sum(per_town)} fit and there is room for {seats_n}, but {chips_n} chips")
entered_then = {t for t in tiles if tile(*t)[0] < THEN and t != later}
beyond = {t for t in tiles if tile(*t)[0] >= THEN and tile(*t)[1] > NOW}
parts = [entered_then, {later}, beyond, inside]
check(sum(len(p) for p in parts) == 20 and set().union(*parts) == set(tiles), "veils and rings cover every tile once")
unentered = {t for t in tiles if tile(*t)[0] >= NOW}
check(not (unentered & before), "the veils at 'never hands out' cover no fill of the real run")
passed = {t for t in tiles if tile(*t)[1] <= NOW}
check(passed <= before, "and every passed tile is filled: the theorem's instance at 13")

print("5. layout boxes per state, from the source's constants")
src = CHAPTER.read_text()


def const(name):
    return float(re.search(rf"const {name}: f64 = (-?[0-9.]+);", src).group(1))


def const_list(name):
    return [float(x) for x in re.search(rf"const {name}: \[f64; 3\] = \[([^\]]+)\];", src).group(1).split(",")]


NAME_X = -6.5
TILE_H = const("TILE_H")
AXIS_Y = const("AXIS_Y")
TILE_Y = const_list("TILE_Y")
PANEL_Y = const_list("PANEL_Y")
BOUND_X = const_list("BOUND_X")
NEED_X, CANNOT_X = const("NEED_X"), const("CANNOT_X")
CHIP, CHIP_X, CHIP_STEP = const("CHIP"), const("CHIP_X"), const("CHIP_STEP")
MARK_GAP, LEGEND_Y = const("MARK_GAP"), const("LEGEND_Y")
LATER_DY, WAIT_DY = const("LATER_T_DY"), const("WAIT_T_DY")
ax, bx = [float(x) for x in re.search(r"AXIS_X: \(f64, f64\) = \(([^)]+)\)", src).group(1).split(",")]
HOUSES = const("TILE_HOUSES")
PREDICT = [float(x) for x in re.search(r"PREDICT_AT: \(f64, f64\) = \(([^)]+)\)", src).group(1).split(",")]


def hx(h):
    return ax + h * (bx - ax) / HOUSES


def centred(text, size, x, y):
    w, h = len(text) * size * 0.0048, size * 0.01
    return (x - w / 2, y - h / 2, x + w / 2, y + h / 2, text)


def left(text, size, x, y):
    w, h = len(text) * size * 0.0048, size * 0.01
    return (x, y - h / 2, x + w, y + h / 2, text)


def overlap(a, b):
    return a[0] < b[2] and b[0] < a[2] and a[1] < b[3] and b[1] < a[3]


CHIPS = [(CHIP_X + i * CHIP_STEP - CHIP / 2, PANEL_Y[0] - CHIP / 2,
          CHIP_X + i * CHIP_STEP + CHIP / 2, PANEL_Y[0] + CHIP / 2, f"chip {i}") for i in range(seats_n + 1)]
ROOM = (CHIP_X - CHIP / 2, PANEL_Y[0] - CHIP / 2 - 0.06, CHIP_X + (seats_n - 1) * CHIP_STEP + CHIP / 2,
        PANEL_Y[0] - CHIP / 2 - 0.01, "room bracket")

fixed = [
    left("marker at", 24, NAME_X, 3.3),
    centred("13", 30, NAME_X + 2.2, 3.3),
    left("house size", 20, NAME_X, AXIS_Y),
    centred("now", 22, hx(NOW), TILE_Y[0] + 0.75),
] + [centred(str(h), 20, hx(h), AXIS_Y - 0.38) for h in (0, 5, 10, 15)] \
  + [left("9.99", 20, NAME_X, y - 0.28) for y in TILE_Y]
suppose = centred("suppose: a passed tile not handed out", 22, 0, 3.3)
legend = centred("solid: real up to then, hollow: only supposed", 18, 0, LEGEND_Y)
then_t = centred("then", 22, hx(THEN), TILE_Y[0] + 0.75)
or_zero = centred("or 0", 20, hx(0), TILE_Y[0] + 0.75)
later_t = left("Birch's seat 2 ends at 8 2/3, later than 7 1/2", 18, hx(THEN) + MARK_GAP, TILE_Y[BIRCH] + LATER_DY)
wait_t = left("6 to 7 1/2: not entered at 6", 18, hx(THEN) + MARK_GAP, TILE_Y[CEDAR] + WAIT_DY)
need_t = left("7 seats from then to now: 7 + 1 = 8 tiles", 22, NEED_X, PANEL_Y[0])
cannot = left("8 tiles cannot fit", 24, CANNOT_X, PANEL_Y[0])
bounds = [centred(t, 22, BOUND_X[i], PANEL_Y[1]) for i, t in
          enumerate(["Ash 0 ≤ 7 × 4/39", "Birch 1 ≤ 7 × 9/39", "Cedar 4 ≤ 7 × 26/39"])]
total_t = centred("0 + 1 + 4 = 5 ≤ 7 × 39/39 = 7", 24, 0, PANEL_Y[2])
real_t = centred("the real run at 13 seats: 1, 3, 9", 22, 0, PANEL_Y[0])
done = centred("quota method: within quota at every house size", 24, 0, 3.3)

states = {
    "going back": fixed + [suppose, legend, then_t],
    "later tile": fixed + [suppose, legend, then_t, later_t],
    "chips": fixed + [suppose, legend, then_t, or_zero, later_t] + CHIPS,
    "not entered": fixed + [suppose, legend, then_t, or_zero, later_t, wait_t] + CHIPS,
    "ends by now": fixed + [suppose, legend, then_t, or_zero, wait_t] + CHIPS,
    "the count": fixed + [suppose, legend, then_t, or_zero, wait_t, need_t, cannot, total_t, ROOM] + bounds + CHIPS,
    "restored": fixed + [real_t, done],
}
SAFE = (-6.96, -3.85, 6.96, 3.85)
for name, boxes in states.items():
    clash = [(a[4], b[4]) for i, a in enumerate(boxes) for b in boxes[i + 1:] if overlap(a, b)]
    off = [b[4] for b in boxes if b[0] < SAFE[0] or b[1] < SAFE[1] or b[2] > SAFE[2] or b[3] > SAFE[3]]
    check(not clash and not off, f"{name}: {len(boxes)} boxes, clashes {clash}, off the safe area {off}")

rows = [(ax, y - TILE_H / 2 - 0.04, bx, y + TILE_H / 2 + 0.04, f"row {i}") for i, y in enumerate(TILE_Y)]
for box in (later_t, wait_t, need_t, cannot, real_t, or_zero, then_t):
    check(not any(overlap(box, r) for r in rows), f"'{box[4]}' clears the rows")
then_line = (hx(THEN) - 0.02, AXIS_Y + 0.05, hx(THEN) + 0.02, TILE_Y[0] + 0.5, "then line")
now_line = (hx(NOW) - 0.02, AXIS_Y + 0.05, hx(NOW) + 0.02, TILE_Y[0] + 0.5, "now line")
for box in (later_t, wait_t, then_t, legend, suppose):
    check(not overlap(box, then_line) and not overlap(box, now_line), f"'{box[4]}' clears both marker lines")

# the think ring
LIST_LEFT, CARD_RIGHT = 4.45 - 2.1 - 0.07, 4.45 + 2.1 + 0.07
note_half = len("pause for more time") * 16 * 0.0048 / 2
new_ring = (PREDICT[0] - max(0.55, note_half), PREDICT[1] - 0.55 - 0.28 - 0.08, PREDICT[0] + max(0.55, note_half), PREDICT[1] + 0.55)
list_box = (LIST_LEFT, -3.85, CARD_RIGHT, 3.85, "the list")
check(not overlap(new_ring + ("ring",), list_box), f"the think ring's right edge {new_ring[2]:.2f} is left of the list {LIST_LEFT:.2f}")
check(new_ring[1] > SAFE[1], f"its note's bottom {new_ring[1]:.2f} is inside the safe area")

# negative controls: the box test can fail
old_ring = (6.1 - 0.55, -2.9 - 0.55, 6.1 + 0.55, -2.9 + 0.55, "old ring")
check(overlap(old_ring, list_box), "control: the old ring at (6.1, -2.9) does fall on the list")
low_legend = centred("solid: real up to then, hollow: only supposed", 18, 0, 2.55)
check(overlap(low_legend, then_t), "control: a legend at y 2.55 would hit 'then'")
far_need = left("7 seats from then to now: 7 + 1 = 8 tiles", 22, NEED_X + 1.0, PANEL_Y[0])
check(any(overlap(far_need, c) for c in CHIPS), "control: the count moved right 1.0 would hit the chips")

# the same inequalities as the Rust layout tests, which cannot run here
print("   the chapter's Rust layout tests, by hand")


def wide(text, size):
    return len(text) * size * 0.0048


def half(size):
    return size * 0.005


THINK_R, ROW_Y_2, ROW_H, LIST_X, CARD_W = 0.55, -1.5, 0.86, 4.45, 4.2
half_width = max(THINK_R, wide("pause for more time", 16) / 2)
chip = lambda i: CHIP_X + i * CHIP_STEP  # noqa: E731
top = lambda r: TILE_Y[r] + TILE_H / 2 + 0.04  # noqa: E731
bottom = lambda r: TILE_Y[r] - TILE_H / 2 - 0.04  # noqa: E731
rust_tests = [
    ("think: list", LIST_X - CARD_W / 2 - 0.07, PREDICT[0] + half_width),
    ("think: rows", ROW_Y_2 - ROW_H / 2, PREDICT[1] + THINK_R),
    ("think: note", PREDICT[1] - THINK_R - 0.28 - half(16), -3.85),
    ("count: left", NEED_X, -6.96),
    ("count: to chips", chip(0) - CHIP / 2, NEED_X + wide("7 seats from then to now: 7 + 1 = 8 tiles", 22)),
    ("count: chips apart", chip(1) - CHIP / 2, chip(0) + CHIP / 2),
    ("count: to verdict", CANNOT_X, chip(seats_n) + CHIP / 2),
    ("count: right", 6.96, CANNOT_X + wide("8 tiles cannot fit", 24)),
    ("panel: axis", AXIS_Y - 0.38 - half(22), PANEL_Y[0] + half(22)),
    ("panel: bracket", PANEL_Y[0] - CHIP / 2 - 0.06, PANEL_Y[1] + half(22)),
    ("panel: rows", PANEL_Y[1] - half(22), PANEL_Y[2] + half(24)),
    ("panel: bottom", PANEL_Y[2] - half(24), -3.85),
    ("labels: later wide", hx(NOW), hx(THEN) + MARK_GAP + wide("Birch's seat 2 ends at 8 2/3, later than 7 1/2", 18)),
    ("labels: wait wide", hx(NOW), hx(THEN) + MARK_GAP + wide("6 to 7 1/2: not entered at 6", 18)),
    ("labels: later over", TILE_Y[BIRCH] + LATER_DY - half(18), top(BIRCH)),
    ("labels: later under", bottom(ASH), TILE_Y[BIRCH] + LATER_DY + half(18)),
    ("labels: wait over", TILE_Y[CEDAR] + WAIT_DY - half(18), top(CEDAR)),
    ("labels: wait under", bottom(BIRCH), TILE_Y[CEDAR] + WAIT_DY + half(18)),
    ("legend: under suppose", 3.3 - half(22), LEGEND_Y + half(18)),
    ("legend: over labels", LEGEND_Y - half(18), TILE_Y[0] + 0.75 + half(22)),
]
tight = min(rust_tests, key=lambda r: r[1] - r[2])
check(all(b > e for _, b, e in rust_tests), f"{len(rust_tests)} inequalities hold; tightest '{tight[0]}' by {tight[1] - tight[2]:.3f}")

print("6. timing, simulated from the source's own beats (not a render)")
# the repair leaves the script alone, so the words are canonical's
script = STAGE.parents[5] / "explainer_corpus/apportionment/script.tsv"
text = dict(line.split("\t", 1) for line in script.read_text().splitlines() if "\t" in line)
LENS = {"12_8": 29.246, "12_9": 36.169}
GAP = 0.62
a = src.index('say(&kit, stage, "12_8")')
b = src.index("// 12_10 one promise")
section = src[a:b]
TOKEN = re.compile(
    r's\.wait_for\(stage, "(?P<phrase>[^"]+)"\)'
    r'|say\(&kit, stage, "(?P<line>12_\d+)"\)'
    r'|\b(?:fade_in_all|fade_out_all|flash_all|run_tracker|pulse_all)\(\s*stage,.*?,\s*(?P<rt>[0-9.]+),?\s*\)\?;'
    r'|\bfade_in_lagged\(\s*stage,.*?,\s*(?P<lrt>[0-9.]+),\s*[0-9.]+,?\s*\)\?;',
    re.S,
)
beats = []
cur = None
for m in TOKEN.finditer(section):
    if m.group("line"):
        cur = m.group("line")
        beats.append([cur, None, 0.0])
    elif m.group("phrase"):
        beats.append([cur, m.group("phrase"), 0.0])
    else:
        beats[-1][2] += float(m.group("rt") or m.group("lrt"))
check(len(beats) == 20, f"{len(beats)} beats parsed from the source: 2 line starts, 18 phrases")


def simulate(scale):
    t = 0.0
    rows_ = []
    line_start = {}
    line_end = 0.0
    for line, phrase, cost in beats:
        if phrase is None:
            start = max(t, line_end + GAP) if line_start else 0.0
            line_start[line] = start
            line_end = start + LENS[line] * scale
            t = start
            when = start
        else:
            whole = text[line]
            where = whole.index(phrase)
            when = line_start[line] + LENS[line] * scale * where / len(whole)
        late = max(0.0, t - when)
        t = max(t, when) + cost
        rows_.append((line, phrase or "(line starts)", when, late, cost))
    return rows_, t, line_end


worst = {}
for scale in (0.8, 1.0, 1.2):
    rows_, end, line_end = simulate(scale)
    worst[scale] = max(r[3] for r in rows_)
    print(f"  scale {scale}: worst lateness {worst[scale]:.2f} s; picture ends {end - line_end:+.2f} s against the voice")
    if scale == 1.0 or scale == 0.8:
        for line, phrase, when, late, cost in rows_:
            print(f"    {line} {when:7.2f}  late {late:4.2f}  beat {cost:4.2f}  {phrase[:58]}")
check(worst[1.0] <= 0.6, f"at the scratch length no beat starts more than 0.6 s after its phrase ({worst[1.0]:.2f})")
print(f"  (at 0.8 of the scratch length the worst is {worst[0.8]:.2f} s: Oliver's real pace is unknown, root's fine sync sets it)")

print()
if failures:
    print(f"FAILED {len(failures)}:")
    for f_ in failures:
        print(" ", f_)
    sys.exit(1)
print("PASS (exact maths, estimated widths, simulated times; no render, no listening)")
