"""Read-only mirror of the logarithm 11-12 repairs, because the Rust is uncompiled here.

Run from the repo root: python3 -I -B <this file>
It prototypes the numbers the new Rust tests assert, the layout, and the new
beats' timeline on the kit's scratch voice. It is not the Rust tests, and no time
here is an Oliver time. The stress factors are arbitrary scalings to exercise the
clamps, not predictions of any take.
"""
import math
import re
import sys
from pathlib import Path

HERE = "docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-repair11-12"
CH = HERE + "/explainer_corpus/logarithm_mysteries/src/chapters/"
SRC11 = Path(CH + "unreachable.rs").read_text(encoding="utf-8")
SRC12 = Path(CH + "what_it_forgets.rs").read_text(encoding="utf-8")
BASE11 = Path("explainer_corpus/logarithm_mysteries/src/chapters/unreachable.rs").read_text(encoding="utf-8")
BASE12 = Path("explainer_corpus/logarithm_mysteries/src/chapters/what_it_forgets.rs").read_text(encoding="utf-8")
SCRIPT = {}
for line in Path("explainer_corpus/logarithm_mysteries/script.tsv").read_text(encoding="utf-8").splitlines():
    if line.startswith("#") or "\t" not in line:
        continue
    k, t = line.split("\t", 1)
    SCRIPT[k] = t

failures = []


def check(ok, label):
    print(("  ok   " if ok else "  FAIL ") + label)
    if not ok:
        failures.append(label)


# ---------------------------------------------------------------- matrices
I2 = [[1.0, 0.0], [0.0, 1.0]]
J = [[0.0, -1.0], [1.0, 0.0]]


def mmul(a, b):
    return [[sum(a[i][k] * b[k][j] for k in range(2)) for j in range(2)] for i in range(2)]


def madd(a, b):
    return [[a[i][j] + b[i][j] for j in range(2)] for i in range(2)]


def mscale(a, k):
    return [[a[i][j] * k for j in range(2)] for i in range(2)]


def apply(a, p):
    return [a[0][0] * p[0] + a[0][1] * p[1], a[1][0] * p[0] + a[1][1] * p[1]]


def mdist(a, b):
    return max(abs(a[i][j] - b[i][j]) for i in range(2) for j in range(2))


def mexp(x):
    """Scaling and squaring of a thirty-term series, as model.rs describes."""
    norm = max(abs(v) for row in x for v in row)
    k = 0
    while norm > 0.5:
        norm /= 2.0
        k += 1
    y = mscale(x, 0.5**k)
    total, term = I2, I2
    for n in range(1, 31):
        term = mscale(mmul(term, y), 1.0 / n)
        total = madd(total, term)
    for _ in range(k):
        total = mmul(total, total)
    return total


def lerp(a, b, t):
    return madd(mscale(a, 1.0 - t), mscale(b, t))


MIRROR = [[-1.0, 0.0], [0.0, 1.0]]
PAIR = [[-2.0, 0.0], [0.0, -0.5]]
CLIP, UNIT = 3.2, 0.7
CENTRE = (-3.6, -0.3)


def on_panel(q):
    return abs(q[0]) < CLIP and abs(q[1]) < CLIP


def reverses(m, d):
    q = apply(m, d)
    return abs(d[0] * q[1] - d[1] * q[0]) < 1e-9 and d[0] * q[0] + d[1] * q[1] < 0.0


STEP = math.pi / 180.0


def reversed_spans(m, a0, a1):
    first = math.ceil(a0 / STEP - 1e-9)
    last = math.floor(a1 / STEP + 1e-9)
    spans, open_ = [], False
    for j in range(first, last + 1):
        a = j * STEP
        if reverses(m, [math.cos(a), math.sin(a)]):
            if not open_:
                spans.append([a, a])
            else:
                spans[-1][1] = a
            open_ = True
        else:
            open_ = False
    return spans


print("matrices and rules")
for j in range(51):
    m = lerp(I2, MIRROR, 0.5 * j / 50.0)
    check(m[0][0] > -1e-12 and abs(m[1][1] - 1.0) < 1e-12, f"k={0.5 * j / 50.0:.2f} scale {m[0][0]:.2f}, heights kept") if j in (0, 25, 50) else None
check(mdist(lerp(I2, MIRROR, 0.5), [[0.0, 0.0], [0.0, 1.0]]) < 1e-12, "the run ends on the average diag(0, 1)")
check(2.0 * 4.0 * 1e-3 * UNIT * 90.0 < 1.0, "the last drawn flat step is under a pixel")
s = reversed_spans(PAIR, 0.0, math.pi)
check(len(s) == 3 and all(abs(x[0] - x[1]) < 1e-9 for x in s), f"flip pair reverses isolated directions {[round(x[0], 4) for x in s]}")
for c in (1.0, 1.5):
    s = reversed_spans(mscale(I2, -c), math.pi, 3.0 * math.pi)
    check(len(s) == 1 and abs(s[0][0] - math.pi) < 1e-9 and abs(s[0][1] - 3 * math.pi) < 1e-9, f"-{c}I reverses a whole lap as one span")
check(len(reversed_spans(PAIR, 0.0, 1.0)) == 1, "part way round, only the axes passed")
check(not reversed_spans(I2, 0.0, 2 * math.pi), "identity reverses nothing")
check(not reversed_spans([[0.0, -1.0], [1.0, 0.0]], 0.0, 2 * math.pi), "quarter turn reverses nothing")
check(len(reversed_spans(MIRROR, 0.0, math.pi)) == 2, "mirror reverses its one axis, seen twice")
ok = True
for j in range(73):
    a = j * math.pi / 36.0
    d = [1.2 * math.cos(a), 1.2 * math.sin(a)]
    for w in (0.0, 0.5, 1.0):
        for c in (1.0, 1.5):
            ok = ok and on_panel(apply(lerp(PAIR, mscale(I2, -c), w), d))
check(ok, "sweep's gold arrow stays on the panel; furthest 2.4 plane units")
ok = True
for j in range(21):
    r = mexp(mscale(J, math.pi * j / 20.0))
    ok = ok and on_panel(apply(r, [3.0, 0.0])) and on_panel(apply(r, [0.0, 3.0]))
check(ok, "marked axes stay on the panel")
q = apply(mexp(mscale(J, math.pi * 0.5)), [1.0, 0.0])
check(abs(q[0]) < 1e-9 and abs(q[1] - 1.0) < 1e-9, "halfway through the half lap, x axis lies on y axis")
for c in (1.0, 2.0, 1.5):
    x = madd(mscale(I2, math.log(c)), mscale(J, math.pi))
    d = mdist(mexp(x), mscale(I2, -c))
    check(d < 1e-9, f"e^(ln {c} I + pi J) = -{c} I, error {d:.1e}")
worst = 0.0
for a, shear, odd in [(0.0, 1.0, 1.0), (0.4, 2.0, 1.0), (-0.7, 0.5, 1.0), (0.2, 1.5, 3.0)]:
    spin = odd * math.pi
    worst = max(worst, mdist(mexp([[a, -spin * shear], [spin / shear, a]]), mscale(I2, -math.exp(a))))
check(worst < 1e-8, f"sheared spins with a +- i pi(2k+1) exponentiate to -e^a I, worst error {worst:.1e} (Rust test limit 1e-8)")


def paired(blocks, i):
    return sum(1 for b in blocks if abs(b[0] - blocks[i][0]) < 1e-12 and b[1] == blocks[i][1]) % 2 == 0


def pairs_up(blocks):
    return all(blocks[i][0] > 0.0 or paired(blocks, i) for i in range(len(blocks)))


ROWS = [
    ([(-1.0, 1), (-1.0, 1)], "matching pair: real log"),
    ([(-2.0, 1), (-0.5, 1)], "no partners: no real log"),
    ([(-2.0, 1), (-2.0, 1), (3.0, 1)], "−2 paired, 3 positive: real log"),
    ([(-1.0, 2)], "one size-2 block: no real log"),
]
for blocks, verdict in ROWS:
    check(pairs_up(blocks) == ("no real log" not in verdict), f"{blocks}: {verdict}")
    check(f'"{verdict}"' in SRC11, f"verdict is in the candidate verbatim: {verdict}")
check(pairs_up([(-1.0, 2), (-1.0, 2)]) and not pairs_up([(-1.0, 1), (-1.0, 2)]), "counts per size")
check(not pairs_up([(-1.0, 1)] * 3) and pairs_up([(3.0, 1)]), "odd counts fail, positives free")


# ------------------------------------------------------------ width table
def em(c):
    if c in "il,.:;![]":
        return 0.278
    if c in "jf":
        return 0.306
    if c in " -":
        return 0.333
    if c == "I":
        return 0.361
    if c in "rst()":
        return 0.39
    if c in "acez":
        return 0.444
    if c in "go^/0123456789":
        return 0.5
    if c in "kqvxy":
        return 0.528
    if c in "bdhnpu":
        return 0.556
    if c == "w":
        return 0.722
    if c in "=+−×±<>":
        return 0.778
    if c == "m":
        return 0.833
    return 0.75


def width(s, size):
    return sum(em(c) for c in s) * size * 0.01


print("layout (width table: em x size x 0.01, room 1.03; column starts x 2.2)")
GAP_X = -1.15
for t, size in [
    ("white: a direction", 18.0),
    ("gold: where A sends it", 18.0),
    ("dashes: straight back", 18.0),
    ("A = diag(−2, −½)", 22.0),
    ("only the two axes go straight back", 20.0),
    ("A = −I", 22.0),
    ("every direction goes straight back", 20.0),
    ("A = −cI, c > 0", 22.0),
    ("half turn, equal stretch", 20.0),
]:
    r = GAP_X + width(t, size) * 1.03
    check(r < 2.1, f"gap word ends {r:.3f}: {t}")
VERDICT_X = -2.75
for blocks, verdict in ROWS:
    r = VERDICT_X + width(verdict, 22.0) * 1.03
    check(r < 2.1, f"verdict ends {r:.3f}: {verdict}")
ROWS_X, STEP_T, HALF, WIDE = -4.9, 0.74, 0.31, 0.49


def tile_half(size):
    return WIDE if size > 1 else HALF


def tile_x(blocks, i):
    return ROWS_X + STEP_T * i + tile_half(blocks[i][1]) - HALF


for blocks, verdict in ROWS:
    last = len(blocks) - 1
    r = tile_x(blocks, last) + tile_half(blocks[last][1])
    left = tile_x(blocks, 0) - tile_half(blocks[0][1])
    check(r < VERDICT_X - 0.1 and abs(left - (ROWS_X - HALF)) < 1e-12, f"row tiles {left:.2f}..{r:.2f}, verdict at {VERDICT_X}")
head = "each row is one matrix, each tile a block and its stretch"
r = ROWS_X - HALF + width(head, 20.0) * 1.03
check(r < 2.1, f"block heading {ROWS_X - HALF:.2f}..{r:.3f}")
bottom = 1.35 - 3.0 - 0.46
check(bottom > -3.4, f"lowest bracket line y {bottom:.2f}, above the fine print at y -3.5")

# the size-2 card fits its outline: matrix_card half width dx + size*0.0095, half height dy + size*0.0055
dx, dy = 20.0 * 0.011, 20.0 * 0.0055
cw, ch = dx + 20.0 * 0.0095, dy + 20.0 * 0.0055
check(cw < WIDE - 0.05 and ch < HALF - 0.05, f"size-2 card half {cw:.2f} x {ch:.2f} inside outline half {WIDE} x {HALF}")

# the dial mark
DIAL_C, DIAL_R = (2.3, 1.45), 0.95


def dial_at(laps, r):
    a = math.pi / 2.0 - 2 * math.pi * laps
    return (DIAL_C[0] + r * math.cos(a), DIAL_C[1] + r * math.sin(a))


mark = 0.380
at = dial_at(mark, DIAL_R + 0.2)
label = f"{10**mark:.1f}"
check(label == "2.4", f"dial label {label} at ({at[0]:.3f}, {at[1]:.3f})")
for digit in (2.0, 3.0):
    p = dial_at(math.log10(digit), DIAL_R + 0.2)
    dist = math.hypot(at[0] - p[0], at[1] - p[1])
    check(dist > 0.4, f"clear of label {digit:.0f} by {dist:.3f}")
check(at[0] + width(label, 14.0) * 1.03 / 2.0 < 3.5, f"label right edge {at[0] + width(label, 14.0) * 1.03 / 2.0:.3f}, caption at 3.6")
check(at[1] - 0.1 > 1.45 - 1.25, f"label bottom {at[1] - 0.1:.3f}, panel B bottom 0.20")

print("the green x2 arrow, now over later items (R3), does not reach its own label plate")
check(0.1 < 0.35 - 0.13, "arrow head half width 0.10 stays under the plate's foot at 0.22 over the axis")


# ------------------------------------------------------------------ hooks
def sentences(text):
    out, start = [], 0
    for i, c in enumerate(text):
        if c in ".?!" and (i + 1 == len(text) or text[i + 1] == " "):
            out.append(text[start : i + 1].strip())
            start = i + 1
    rest = text[start:].strip()
    if rest:
        out.append(rest)
    return out


def scratch(text):
    speech = len(text.split()) / 2.6
    parts = sentences(text)
    whole = sum(len(p) for p in parts)
    starts, before = [], 0
    for p in parts:
        starts.append(speech * before / whole)
        before += len(p)
    return starts, speech + 0.4


def phrase_seconds(text, phrase, starts, length):
    byte = text.index(phrase)
    opens = [0]
    for i in range(len(text) - 1):
        if text[i] in ".?!" and text[i + 1] == " ":
            opens.append(i + 2)
    assert len(opens) == len(starts)
    j = max(k for k, o in enumerate(opens) if o <= byte)
    close = opens[j + 1] if j + 1 < len(opens) else len(text)
    t0, t1 = starts[j], starts[j + 1] if j + 1 < len(starts) else length
    return t0 + (t1 - t0) * (byte - opens[j]) / max(1, len(text[opens[j] : close].rstrip()))


HOOKS = {
    "11_1": ["a halfway point", "squashes the whole plane"],
    "11_5": ["the two routes end in different places"],
    "11_7": [
        "so nothing forces a logarithm to keep the axes",
        "the spin, which turns every axis",
        "a matrix that sends some direction",
        "only when it does that to every direction alike",
        "a half turn combined with an equal stretch",
        "the matrix's blocks for each negative stretch",
        "come in matching pairs",
    ],
    "12_6": ["also ends here", "and so does two and a half"],
}
print("phrase hooks: exact, once, in order; every hook also a Rust const in the candidate")
for cue, phrases in HOOKS.items():
    last = 0
    src = SRC12 if cue.startswith("12") else SRC11
    for p in phrases:
        n = SCRIPT[cue].count(p)
        at = SCRIPT[cue].find(p)
        check(n == 1 and at >= last and f'"{p}"' in src, f"{cue}: {p!r}")
        last = at

# the known scratch numbers from the review's MEASUREMENTS.md, as a control on the formula
START = {"11_1": 0.600, "11_5": 126.633, "11_7": 187.500, "12_6": 104.567}
LEN = {"11_1": 27.323, "11_5": 33.092, "11_7": 47.708, "12_6": 58.092}
print("control against MEASUREMENTS.md phrase table (the table closes a line's last sentence at speech end)")


def known(cue, phrase):
    starts, _ = scratch(SCRIPT[cue])
    return START[cue] + phrase_seconds(SCRIPT[cue], phrase, starts, LEN[cue] - 0.4)


for cue, phrase, expected in [
    ("11_1", "squashes the whole plane", 24.51),
    ("11_5", "end in different places", 140.86),
    ("11_7", "come in matching pairs", 230.94),
    ("12_6", "also ends here", 111.33),
    ("12_6", "and so does two and a half", 112.56),
]:
    got = known(cue, phrase)
    check(abs(got - expected) < 0.015, f"{cue} {phrase!r}: {got:.2f}, table {expected}")
starts_, _ = scratch(SCRIPT["11_1"])
hook = START["11_1"] + phrase_seconds(SCRIPT["11_1"], "squashes the whole plane", starts_, LEN["11_1"])
check(abs(hook - 24.754) < 0.001, f"the hook closes a last sentence at the line's length, tail included: {hook:.2f}, so 0.25 s after the table here")


# ---------------------------------------------------------------- timeline
def clamp(x, lo, hi):
    return max(lo, min(hi, x))


class Line:
    def __init__(self, cue, factor=1.0):
        self.cue = cue
        self.text = SCRIPT[cue]
        starts, length = scratch(self.text)
        self.starts = [s * factor for s in starts]
        self.len = length * factor
        self.at0 = START[cue]

    def s(self, j):
        return self.at0 + self.starts[j]

    def p(self, phrase):
        return self.at0 + phrase_seconds(self.text, phrase, self.starts, self.len)

    @property
    def end(self):
        return self.at0 + self.len


def beat(rows, name, a, b):
    rows.append((name, a, b))
    return b


def t11_1(f=1.0, show=True):
    L, rows = Line("11_1", f), []
    t = L.s(4)
    t = beat(rows, "avg_a caption", t, t + 0.5)
    t = max(t, L.p("a halfway point"))
    t = beat(rows, "midpoint dots; words out", t, t + 0.4)
    t = max(t, L.p("squashes the whole plane"))
    t = beat(rows, "dashes out", t, t + 0.3)
    t = beat(rows, "grid and F collapse", t, t + clamp(L.end - 0.5 - t, 1.2, 2.6))
    t = beat(rows, "whole line + caption", t, t + 0.4)
    return rows, t, L.end, "line end"


def t11_5(f=1.0):
    L, rows = Line("11_5", f), []
    t = max(L.s(2), L.p("the two routes end in different places"))
    t = beat(rows, "gap bar + caption", t, t + 0.4)
    return rows, t, L.s(3), "s3"


def t11_7a(f=1.0):
    L, rows = Line("11_7", f), []
    t = L.s(3)
    t = beat(rows, "equal captions", t, t + 0.4)
    t = max(t, L.p("so nothing forces a logarithm to keep the axes"))
    t = beat(rows, "rewind, axes on", t, t + clamp(L.p("the spin, which turns every axis") - t, 0.8, 1.8))
    t = max(t, L.p("the spin, which turns every axis"))
    t = beat(rows, "half lap, axes turn", t, t + clamp(L.s(4) - 0.2 - t, 1.0, 2.4))
    return rows, t, L.s(4), "s4"


def t11_7b(f=1.0):
    L, rows = Line("11_7", f), []
    t = L.s(4)
    t = beat(rows, "plane out, rim in", t, t + 0.4)
    t = max(t, L.p("a matrix that sends some direction"))
    t = beat(rows, "legend, A = diag", t, t + 0.4)
    every = L.p("only when it does that to every direction alike")
    t = beat(rows, "sweep half lap, flip pair", t, t + clamp(every - t, 2.0, 6.0))
    t = max(t, every)
    t = beat(rows, "gold slides to -I", t, t + 0.3)
    t = beat(rows, "caption A = -I", t, t + 0.3)
    half = L.p("a half turn combined with an equal stretch")
    t = beat(rows, "sweep whole lap, -I", t, t + clamp(half - t, 1.5, 3.0))
    t = max(t, half)
    t = beat(rows, "caption A = -cI", t, t + 0.3)
    t = beat(rows, "c grows", t, t + clamp(L.s(5) - 0.2 - t, 0.8, 1.6))
    return rows, t, L.s(5), "s5"


def t11_7c(f=1.0):
    L, rows = Line("11_7", f), []
    t = L.s(5)
    t = beat(rows, "sweep out, cite, heading", t, t + 0.4)
    blocks = L.p("the matrix's blocks for each negative stretch")
    t = beat(rows, "tiles lagged", t, t + clamp(blocks - t, 1.2, 4.0))
    t = max(t, blocks)
    match = L.p("come in matching pairs")
    t = beat(rows, "outlines, verdicts lagged", t, t + clamp(match - t, 0.8, 2.4))
    t = max(t, match)
    t = beat(rows, "brackets", t, t + 0.4)
    return rows, t, L.s(6), "s6"


def t12_6(f=1.0):
    L, rows = Line("12_6", f), []
    t = L.s(1)
    t = beat(rows, "word swap", t, t + 0.3)
    t = beat(rows, "1½ run", t, t + clamp(L.p("also ends here") - t, 1.0, 3.0))
    arrival = t
    t = t + 0.5
    t = max(t, L.p("and so does two and a half"))
    rows.append(("1½ held", arrival, t))
    t = beat(rows, "word swap", t, t + 0.3)
    t = beat(rows, "2½ run", t, t + clamp(L.s(2) - 0.5 - t, 1.0, 3.4))
    t = beat(rows, "laps chip", t, t + 0.4)
    return rows, t, L.s(2), "s2"


print("timeline on the kit's scratch voice (seconds of scene time)")
for fn in (t11_1, t11_5, t11_7a, t11_7b, t11_7c, t12_6):
    rows, t, limit, name = fn(1.0)
    for label_, a, b in rows:
        print(f"    {a:8.3f} {b:8.3f}  {label_}")
    check(t <= limit + 1e-9, f"{fn.__name__}: last beat ends {t:.3f}, {name} at {limit:.3f}")

print("clamps under arbitrary scalings of that scratch line: spare seconds at each limit")
print("(negative would delay what follows; these factors are stress values, not takes)")
for f in (0.8, 0.9, 1.1, 1.25):
    print(f"  factor {f}: " + ", ".join(f"{fn.__name__} {fn(f)[2] - fn(f)[1]:+.2f}" for fn in (t11_1, t11_5, t11_7a, t11_7b, t11_7c, t12_6)))

# reading on scratch: the verdicts share the screen with the rule text from s6
L = Line("11_7")
next_fade = 236.33
chars = sum(len(v) for _, v in ROWS)
print(f"reading, scratch: four verdicts {chars} chars over {next_fade - L.p(HOOKS['11_7'][5]):.2f} s = {chars / (next_fade - L.p(HOOKS['11_7'][5])):.1f} chars/s (guideline 15), before counting the rule lines the column adds at s6")
rule = sum(len(x) for x in ["A real matrix has a real logarithm exactly", "when it is invertible and, for every negative", "eigenvalue, its Jordan blocks of each size", "occur an even number of times."])
print(f"reading, scratch: the rule's {rule} chars have {next_fade - L.s(6):.2f} s = {rule / (next_fade - L.s(6)):.1f} chars/s, unchanged by this repair")

# ------------------------------------------------------------- source scans
print("source scans of the candidates")
construct11 = SRC11.split("impl SceneConstruct for Unreachable")[1].split("#[cfg(test)]")[0]


def fades(src):
    return " ".join(re.findall(r"(?:fade_out_all|swap)\([^;]*?;", src, re.S))


listed = " ".join(
    re.findall(r"let mut (?:first_picture|sweep_gone|rule_gone) = vec!\[[^\]]*\]", construct11, re.S)
) + fades(construct11)
for name in [
    "collapsing", "flat_f", "whole_line", "before_f", "after_f", "joins", "before_word", "after_word",
    "turning", "spinning", "sweeping", "still_grid", "rim", "some_a", "some_b", "all_a", "all_b", "any_a", "any_b",
    "blocks_head", "cite",
]:
    check(re.search(rf"\b{name}\b", listed) is not None, f"11: {name} leaves the scene")
for tail in ["first_picture.extend(mids)", "sweep_gone.extend(legend)", "rule_gone.extend(rows.tiles)", "rule_gone.extend(rows.verdicts)", "rule_gone.extend(rows.brackets)"]:
    check(tail in construct11, f"11: {tail}")
check("start_grid," not in construct11.split('kit.say(stage, "11_2")')[1].split("let x = fact_matrix")[0], "11: the replaced static grid is not faded again")
check("replace(start_grid, &[collapsing])" in construct11, "11: the live grid takes the static grid's slot")
check(not re.search(r"\barrow\(", SRC11), "11: no call of draw::arrow left")
check("arrow," not in SRC11.split("use crate::model")[0].replace("arrow_parts,", ""), "11: arrow gone from the imports")
check(SRC11.count("flat(Palette::dim(), 0.65)") == 2 and "flat(Palette::faint(), 0.8)" not in SRC11, "11: R7 at both Fs")
check(SRC11.count("arrow_parts(stage.arena_mut()") == 2, "11: R8 at both arrows")
check("stage.set_z_index(e1, 1, true);" in SRC11, "11: R3 before the fade")
check("routes(s, flip_pair, xs, u, value(s, gap_on) > 0.5)" in SRC11 and "routes(s, x_any, e_any, u, true)" in SRC11, "11: R4 at both callers")
check("mark_dot, mark_word, same" in SRC12 and SRC12.count("mark_dot") >= 3, "12: R2 in its fade and the panels list")

new = set()
for new_src, base in ((SRC11, BASE11), (SRC12, BASE12)):
    was = set(re.findall(r'"([^"\n]*)"', base))
    for t in re.findall(r'"([^"\n]*)"', new_src):
        if t not in was:
            new.add(t)
allowed = set("".join(re.findall(r'"([^"\n]*)"', BASE11 + BASE12)))
allowed |= set(Path("explainer_corpus/logarithm_mysteries/src/chapters/mod.rs").read_text(encoding="utf-8"))
odd = {c for t in new for c in t if c not in allowed}
check(not odd, f"new strings use no character the baseline lacks: {sorted(odd)}")

print()
print(f"FAILURES: {len(failures)}")
for f_ in failures:
    print("  " + f_)
sys.exit(1 if failures else 0)
