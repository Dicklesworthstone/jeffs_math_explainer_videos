"""Cheap checks of the staged chapter 02 repair: nothing here compiles or renders.

Run from the repository root with: python3 -I -B <this file>

It reads the candidate source and the canonical script, then checks:
  1. the arithmetic the new labels print, with exact fractions;
  2. every say id is in order and every phrase exists in the line being spoken;
  3. the schedule under the scratch voice, walking the candidate's own calls: where each
     step lands, whether it ends inside its line, and whether later cues keep their times;
  4. text boxes in each new view, by CMR10 widths (ems x size x 0.01), against the kit's
     lint limits. The width rule is first checked against text measured on frames.

Scratch times are interpolated by the kit's rule. They are not Oliver, not fine sync and
not listening. A box check is not a picture.
"""
import re
from fractions import Fraction as Fr
from pathlib import Path

HERE = Path(__file__).resolve().parent
CANDIDATE = HERE.parent / "explainer_corpus/apportionment/src/chapters/shares.rs"
SCRIPT = Path("explainer_corpus/apportionment/script.tsv")

source = CANDIDATE.read_text(encoding="utf-8")
rows = {}
for line in SCRIPT.read_text(encoding="utf-8").splitlines():
    parts = line.split("\t")
    if len(parts) >= 2:
        rows[parts[0]] = parts[1]

failures = []


def need(ok, message):
    if not ok:
        failures.append(message)
        print("  FAIL", message)


# ------------------------------------------------------------------ 1 arithmetic
print("1. arithmetic")
POP = {"Ash": 4000, "Birch": 9000, "Cedar": 26000}
total = sum(POP.values())
HOUSE = 15
per_seat = Fr(total, HOUSE)
need(per_seat.denominator == 1, "2,600 must divide the total exactly")
print(f"  {total:,} / {HOUSE} = {per_seat}")


def dec2(q):
    """As model::decimal(q, 2): half up, in exact integers."""
    n = (2 * q.numerator * 100 + q.denominator) // (2 * q.denominator)
    return f"{n // 100}.{n % 100:02d}"


for town, people in POP.items():
    drawn = Fr(people) / per_seat
    model = Fr(HOUSE * people, total)
    need(drawn == model, f"{town}: drawn division differs from the model share")
    print(f"  {people:,} ÷ {per_seat:,} = {dec2(drawn)}  ({drawn})")

for h, want_rounded, want_label in ((14, [1, 3, 9], "1 short"), (15, [2, 3, 10], None), (16, [2, 4, 11], "1 over")):
    quotas = [Fr(h * p, total) for p in POP.values()]
    rounded = [(2 * q.numerator + q.denominator) // (2 * q.denominator) for q in quotas]
    s = sum(rounded)
    label = None if s == h else (f"{h - s} short" if s < h else f"{s - h} over")
    need(rounded == want_rounded, f"{h}: rounded {rounded}")
    need(label == want_label, f"{h}: label {label}")
    moved = "all down" if all(r < q for r, q in zip(rounded, quotas)) else "all up" if all(r > q for r, q in zip(rounded, quotas)) else "mixed"
    print(f"  {h} seats: {[dec2(q) for q in quotas]} -> {rounded} = {s}, tag {label}, {moved}")

# --------------------------------------------------------------- 2 ids and phrases
print("2. say ids and phrases")
said = re.findall(r'say\(&kit, stage, "(\d\d_\d)"\)', source)
want = [k for k in rows if k.startswith("02_")]
need(said == want, f"said {said}, script has {want}")
print(f"  said in order: {said}")

CALL = re.compile(
    r'(?P<say>let s = say\(&kit, stage, "(?P<id>\d\d_\d)"\)\?;)'
    r'|s\.(?:wait_for\(stage, |at\()"(?P<phrase>[^"]+)"'
    r'|\[\(0, "(?P<a>[^"]+)"\), \(1, "(?P<b>[^"]+)"\), \(2, "(?P<c>[^"]+)"\)\]'
    r'|\["(?P<d>[^"]+)", "(?P<e>[^"]+)", "(?P<f>[^"]+)"\]\s*\.into_iter'
)
cue = None
checked = 0
for m in CALL.finditer(source):
    if m.group("say"):
        cue = m.group("id")
        continue
    for phrase in filter(None, (m.group(k) for k in ("phrase", "a", "b", "c", "d", "e", "f"))):
        n = rows[cue].count(phrase)
        checked += 1
        need(n == 1, f"{cue}: phrase {phrase!r} occurs {n} times")
print(f"  {checked} phrases each occur exactly once in their own line")

# Every `.rt(` call needs the trait in scope; draw.rs imports it for the same call.
need("Timed" in source.split("fn morph_and_fade_in")[0], "shares.rs calls .rt() but does not import explainer_kit::Timed")

# Handle discipline: a Transform target must never be used after its morph.
after_gather = source.split("morph_and_fade_in(stage, &gather, &arriving, 0.7)?;")[1]
for t in ("exact.total", "exact.shares", "exact.rounded", "sum15"):
    need(t not in after_gather, f"{t} is a detached target but is used after its morph")
need("rows.bands[i][0]" in source and source.count("bands[i][0]") == 1, "the whole band should only ever be a morph target")
print("  morph targets are not touched after their morphs; later steps use need, reads, marks, bars")

# ------------------------------------------------------------- 3 scratch schedule
print("3. schedule under the scratch voice (cue starts from the preview's cues.tsv)")
CUES = {
    "02_1": (0.600, 15.015),
    "02_2": (16.833, 18.862),
    "02_3": (36.300, 13.862),
    "02_4": (50.767, 14.246),
    "02_5": (70.100, 16.938),
    "02_6": (87.667, 13.092),
    "02_7": (101.967, 15.015),
}


def sentence_parts(text):
    out, start = [], 0
    for i, c in enumerate(text):
        if c in ".?!" and (i + 1 == len(text) or text[i + 1] == " "):
            out.append(text[start : i + 1].strip())
            start = i + 1
    return out


def offsets(text):
    out = [0]
    for i in range(len(text) - 1):
        if text[i] in ".?!" and text[i + 1] == " " and i + 2 < len(text):
            out.append(i + 2)
    return out


def at(cue_id, phrase):
    text = rows[cue_id]
    start, length = CUES[cue_id]
    speech = length - 0.4
    parts = sentence_parts(text)
    whole = sum(len(s) for s in parts)
    before, starts = 0, []
    for s in parts:
        starts.append(speech * before / whole)
        before += len(s)
    off = offsets(text)
    i = text.index(phrase)
    j = max(k for k, o in enumerate(off) if o <= i)
    t0 = starts[j]
    t1 = starts[j + 1] if j + 1 < len(starts) else length
    to = off[j + 1] if j + 1 < len(off) else len(text)
    return start + t0 + (t1 - t0) * (i - off[j]) / max(to - off[j], 1)


# The candidate's own calls, in order: (cue, wait phrase or None, offset, seconds, what).
STEPS = [
    ("02_1", None, 0, 1.2, "bars fade in"),
    ("02_1", "Together the towns", 0, 1.2 + 0.5, "bars slide into the line; 39,000 people"),
    ("02_1", "Fifteen seats", 0, 0.4 + 0.4, "slide over; ÷ 15 seats"),
    ("02_1", None, 0, 1.5, "ticks"),
    ("02_1", "one seat for every", 0, 0.6, "brace and 2,600 people = 1 seat"),
    ("02_2", "population divided", 0, 0.5, "share = people ÷ 2,600"),
    ("02_2", "Ash's share", -1.1, 0.4 + 0.6, "line parts out; names and grids in"),
    ("02_2", "Ash's share", 0, 0.7, "Ash's bar to its row; 4,000 ÷ 2,600 = 1.54"),
    ("02_2", "Birch's is", 0, 0.7, "Birch's bar; 9,000 ÷ 2,600 = 3.46"),
    ("02_2", "Cedar's is", 0, 0.7, "Cedar's bar; 26,000 ÷ 2,600 = 10.00"),
    ("02_2", "call this exact number", 0, 0.5, "share = quota"),
    ("02_3", "Nobody can sit", 0, 1.0, "flash Ash's tail"),
    ("02_3", "Each share has", 0, 0.5, "→ ? on each row"),
    ("02_3", "add up to exactly", 0, 0.5, "? + ? + ? = 15"),
    ("02_4", "round each share", 0, 0.6, "? becomes 2, 3, 10"),
    ("02_4", "because two plus", 0, 0.7, "sum turns green"),
    ("02_5", None, 0, 0.4, "rows view out"),
    ("02_5", None, 0, 0.7, "row 15 gathers; header in"),
    ("02_5", "At fourteen", 0, 0.3, "14"),
    ("02_5", "one point four four", 0, 0.4, "1.44"),
    ("02_5", "three point two three", 0, 0.4, "3.23"),
    ("02_5", "nine point three three", 0, 0.4, "9.33"),
    ("02_5", "Every one rounds down", 0, 0.5, "1 3 9"),
    ("02_5", "a seat short", 0, 0.4 + 0.7, "13 and its flash"),
    ("02_5", "At sixteen", 0, 1.2, "16 and its shares"),
    ("02_5", "every share rounds up", 0, 0.5, "2 4 11"),
    ("02_5", "hand out", 0, 0.4 + 0.7, "17 and its flash"),
    ("02_6", "the total", 0, 0.6 + 1.0, "table dims; totals flash"),
    ("02_6", "a way to settle", 0, 0.5, "1 short, 1 over"),
    ("02_7", None, 0, 0.5, "both empty frames"),
    ("02_7", "One keeps", 0, 0.6, "first label"),
    ("02_7", "The other changes", 0, 0.6, "second label"),
    ("02_7", "The paradox lives", 0, 1.0 + 0.5 + 0.6, "flash; gold card with dark words; beat"),
]
now = 0.0
last_cue = None
for cue_id, phrase, offset, seconds, what in STEPS:
    start, length = CUES[cue_id]
    if cue_id != last_cue:
        now = max(now, start)
        last_cue = cue_id
    wait = (at(cue_id, phrase) + offset) if phrase else now
    begin = max(now, wait)
    late = begin - wait
    now = begin + seconds
    end = start + length
    margin = end - now
    flag = "" if margin >= 0 else "  PAST LINE END"
    need(margin >= 0, f"{cue_id} {what}: ends {-margin:.2f} s past the line")
    voice = f"{phrase!r} {wait:7.3f}" if phrase else " " * 7
    print(f"  {cue_id} {begin:7.2f}-{now:7.2f}  room {margin:5.2f}  late {late:4.2f}  {what}{flag}   [{voice}]")
    need(begin >= start - 1e-9, f"{cue_id} {what}: starts before its line")

for prev, nxt, gap in (("02_2", "02_3", 0.6), ("02_3", "02_4", 0.6), ("02_5", "02_6", 0.6)):
    end = CUES[prev][0] + CUES[prev][1]
    print(f"  {prev} ends {end:.3f}; {nxt} starts {CUES[nxt][0]:.3f}, breath {gap}: later cues keep their times if no step above runs past its line")

# Stillness across the whole chapter, cue joins included. The think ring (drawn from 65.61
# to 70.01) and the 02_7 wipe (101.36 to 101.97) are moving pictures the steps above omit.
moving = [(65.61, 70.01), (101.36, 101.97)]
now = 0.0
last_cue = None
for cue_id, phrase, offset, seconds, what in STEPS:
    start, _ = CUES[cue_id]
    if cue_id != last_cue:
        now = max(now, start)
        last_cue = cue_id
    begin = max(now, (at(cue_id, phrase) + offset) if phrase else now)
    moving.append((begin, begin + seconds))
    now = begin + seconds
moving.sort()
print("  picture still for 4 s or more (scratch), cue joins included:")
end_of_last = 0.0
for a, b in moving:
    if a - end_of_last >= 4.0:
        print(f"    {a - end_of_last:5.2f} s  {end_of_last:7.2f} to {a:7.2f}")
    end_of_last = max(end_of_last, b)

# ------------------------------------------------------------------ 4 text boxes
print("4. text boxes (CMR10 ems x size x 0.01; lint limits: text 0.25 from edge, frame +-7.11, +-4)")
EM = {
    ",": 0.278, ".": 0.278, " ": 0.333, "=": 0.778, "+": 0.778, "?": 0.472, "÷": 0.778, "→": 1.0,
    "a": 0.5, "b": 0.556, "c": 0.444, "d": 0.556, "e": 0.444, "f": 0.306, "g": 0.5, "h": 0.556, "i": 0.278,
    "j": 0.306, "k": 0.528, "l": 0.278, "m": 0.833, "n": 0.556, "o": 0.5, "p": 0.556, "q": 0.528, "r": 0.392,
    "s": 0.394, "t": 0.389, "u": 0.556, "v": 0.528, "w": 0.722, "x": 0.528, "y": 0.528, "z": 0.444,
    "A": 0.75, "B": 0.708, "C": 0.722,
}


def width(text, size):
    return sum(0.5 if c.isdigit() else EM[c] for c in text) * size * 0.01


# The rule against text measured on the preview's frames (px at 1280, 90 px per unit).
for text, size, measured in (("39,000 people", 30, 160 / 90), ("2,600 people = 1 seat", 24, 204 / 90)):
    est = width(text, size)
    print(f"  rule check {text!r}: estimate {est:.2f}, measured {measured:.2f}")
    need(abs(est - measured) < 0.06, f"width rule off for {text!r}")


def box(text, size, x, y, left=False):
    w = width(text, size)
    x0 = x if left else x - w / 2
    h = size * 0.01
    return (text, x0, x0 + w, y - h / 2, y + h / 2)


def check_view(name, boxes):
    print(f"  {name}:")
    for b in boxes:
        print(f"    {b[1]:6.2f} to {b[2]:6.2f}  y {b[3]:5.2f} to {b[4]:5.2f}  {b[0]}")
        need(b[1] > -7.11 + 0.25 and b[2] < 7.11 - 0.25, f"{name}: {b[0]!r} too close to the side")
        need(b[4] < 4 - 0.25 and b[3] > -4 + 0.15, f"{name}: {b[0]!r} too close to top or bottom")
    for i, a in enumerate(boxes):
        for c in boxes[i + 1 :]:
            ox = min(a[2], c[2]) - max(a[1], c[1])
            oy = min(a[4], c[4]) - max(a[3], c[3])
            if ox > 0 and oy > 0:
                smaller = min((a[2] - a[1]) * (a[4] - a[3]), (c[2] - c[1]) * (c[4] - c[3]))
                need(ox * oy <= 0.10 * smaller, f"{name}: {a[0]!r} overlaps {c[0]!r}")


LINE_Y = 0.75
check_view(
    "line view at the rule",
    [
        box("39,000 people", 30, -0.74, 1.6),
        box("÷ 15 seats", 30, 0.25, 1.6, left=True),
        box("share = people ÷ 2,600", 30, 0, 2.75),
        box("2,600 people = 1 seat", 24, -5.85 + 1.4, LINE_Y - 0.95),
    ],
)
gap_left = -0.74 + width("39,000 people", 30) / 2
print(f"    gap between the two parts over the line: {0.25 - gap_left:.2f}; whole spans {-0.74 - width('39,000 people', 30) / 2:.2f} to {0.25 + width('÷ 15 seats', 30):.2f}")

READ_X, DIGIT = 1.15, 0.12
reads = []
for people, result, y in (("4,000", "1.54", 1.5), ("9,000", "3.46", 0.0), ("26,000", "10.00", -1.5)):
    x = READ_X - DIGIT * (len(people) - 5)
    reads.append(box(f"{people} ÷ 2,600 = {result}", 24, x, y, left=True))
    op_at = x + width(f"{people} ", 24)
    print(f"    {people}: ÷ starts at x = {op_at:.3f}")
rows_view = reads + [
    box("share = people ÷ 2,600", 30, 0, 2.75),
    box("share = quota", 26, 4.6, 2.75),
    box("2 + 3 + 10 = 15", 32, -1.6, -2.75),
    box("?", 70, 6.1, -2.9),
    box("pause for more time", 16, 6.1, -2.9 - 0.55 - 0.28),
]
for y, value in ((1.5, "2"), (0.0, "3"), (-1.5, "10")):
    rows_view.append(box("→", 24, 3.75, y, left=True))
    rows_view.append(box(value, 24, 4.11, y, left=True))
for name, y, people in (("Ash", 1.5, "4,000 people"), ("Birch", 0.0, "9,000 people"), ("Cedar", -1.5, "26,000 people")):
    rows_view.append(box(name, 26, -6.5, y + 0.14, left=True))
    rows_view.append(box(people, 18, -6.5, y - 0.24, left=True))
check_view("rows view, 02_4", rows_view)
print("    the waiting line's bars: y 0.57 to 0.93; Ash's grid bottom 1.07, Birch's grid top 0.43, Ash's people label bottom about 1.17, Birch's name top about 0.27")

cols = [-5.2, -3.2, -1.6, 0.0, 2.3, 4.6]
table_view = [box(h, 24, cols[c], 2.2) for c, h in enumerate(["seats", "Ash", "Birch", "Cedar", "rounded", "total"])]
data = {14: (["1.44", "3.23", "9.33"], [1, 3, 9], "13", "1 short"), 15: (["1.54", "3.46", "10.00"], [2, 3, 10], "15", None), 16: (["1.64", "3.69", "10.67"], [2, 4, 11], "17", "1 over")}
for r, h in enumerate((14, 15, 16)):
    y = 1.2 - r
    shares, rounded, total_text, tag = data[h]
    table_view.append(box(str(h), 28, cols[0], y))
    for i, s in enumerate(shares):
        table_view.append(box(s, 28, cols[i + 1], y))
    for i, n in enumerate(rounded):
        table_view.append(box(str(n), 28, cols[4] + (i - 1) * 0.62, y))
    table_view.append(box(total_text, 32, cols[5], y))
    if tag:
        table_view.append(box(tag, 20, cols[5] + 0.55, y, left=True))
check_view("table view with tags, 02_6", table_view)

check_view(
    "cards, 02_7",
    [box("compare what is left over", 28, -3.3, 0.4), box("change the people per seat", 28, 3.3, 0.4)],
)
print(f"    a card at the flash's peak (x 1.25) spans {-3.3 - 3.5:.2f} to {-3.3 + 3.5:.2f}; the second card starts at {3.3 - 2.8:.2f}")


def lin(c):
    c = c / 255
    return c / 12.92 if c <= 0.03928 else ((c + 0.055) / 1.055) ** 2.4


def lum(rgb):
    return 0.2126 * lin(rgb[0]) + 0.7152 * lin(rgb[1]) + 0.0722 * lin(rgb[2])


def contrast(a, b):
    la, lb = sorted((lum(a), lum(b)), reverse=True)
    return (la + 0.05) / (lb + 0.05)


hero, ink, background = (0xFF, 0xD8, 0x66), (0xEC, 0xEF, 0xF4), (0x0E, 0x11, 0x16)
print(f"  palette contrast on the gold card: baseline ink {contrast(ink, hero):.2f}:1, candidate dark {contrast(background, hero):.2f}:1")

print()
print("RESULT:", "FAIL " + str(len(failures)) if failures else "all checks pass (no compile, no render, no listening)")
raise SystemExit(1 if failures else 0)
