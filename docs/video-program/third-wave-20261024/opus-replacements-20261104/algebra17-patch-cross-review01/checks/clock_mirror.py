"""A hand-written mirror of 17_applications: clock, cue order and object lifecycle.

Run from the repo root with python3 -I -B. Reads only; prints to stdout.

It mirrors before-applications.rs (P=False) and proposed-applications.rs (P=True) call
by call, written by the cross-reviewer from the Rust, not from RubyRaven's checks. It is
a model, not a render: nothing here compiles or draws anything.

Rules it copies from the source:
- Kit::say: wait until the previous line's end plus BREATH (0.6, LineGaps::Constant at
  main.rs:674), on the frame grid. A picture that overruns delays the voice.
- Kit::until_sentence / until_phrase: wait if the moment is ahead, nothing if it passed.
- A play's frames are its run time rounded up to the frame grid.
- A fade out of something not on stage is the F1 failure (begin_animations re-roots it);
  a fade in of something already on stage is reported too.

The baseline is the positive control: its line starts must equal the render's cue table,
and the lifecycle check must find F1 and nothing else.
"""
import math
import re
import sys
from pathlib import Path

BREATH = 0.6
REVIEW = Path(
    "docs/video-program/third-wave-20261024/opus-replacements-20261104/algebra-picture-review-17_applications"
)
CUES = Path(
    "narration_lab/third-wave-20261024/root/five-film-release-20261027/"
    "algebra-combined-repair-preview01/render/17_applications.cues.tsv"
)
SCRIPT = Path("explainer_corpus/abstract_algebra_expanded/script.tsv")


def sentences(text):
    out, start = [], 0
    for i, c in enumerate(text):
        if c in ".?!" and (i + 1 == len(text) or text[i + 1] == " "):
            out.append(text[start : i + 1].strip())
            start = i + 1
    return out


def scratch_table():
    """Cue start and length from the render; sentence offsets from measure.out.txt."""
    cues = {}
    for row in CUES.read_text().splitlines():
        cue, start, length = row.split("\t")[:3]
        cues[cue] = {"start": float(start), "len": float(length), "offsets": []}
    current = None
    for row in (REVIEW / "checks/measure.out.txt").read_text().splitlines():
        m = re.match(r"(17_\d+)\s+([\d.]+) \+ ([\d.]+)", row)
        if m:
            current = m.group(1)
            continue
        m = re.match(r"\s+(\d+)\s+([\d.]+)\s+\S", row)
        if m and current:
            cues[current]["offsets"].append(float(m.group(2)) - cues[current]["start"])
        if row.startswith("# arithmetic"):
            current = None
    text = {}
    for raw in SCRIPT.read_text(encoding="utf-8").splitlines():
        if raw.startswith("17_"):
            cue, t = raw.split("\t", 1)
            text[cue] = t
    return cues, text


class Mob:
    n = 0

    def __init__(self, name):
        self.name, self.on = name, False


class Sim:
    def __init__(self, cues, text, fps, scale, fix=()):
        self.fps, self.scale, self.cues, self.text, self.fix = fps, scale, cues, text, set(fix)
        self.f = 0
        self.busy = 0.0
        self.said = {}
        self.order = []
        self.events = []
        self.problems = []
        self.late_lines = []
        self.late_beats = []
        self.notes = {}
        self.cue = None

    @property
    def t(self):
        return self.f / self.fps

    def covering(self, seconds):
        return math.ceil(seconds * self.fps - 1e-9)

    def m(self, name):
        return Mob(name)

    def ms(self, name, n):
        return [Mob(f"{name}{i}") for i in range(n)]

    def wait(self, seconds):
        self.f += self.covering(seconds)

    def wait_to(self, at):
        self.f = max(self.f, self.covering(at))

    def note(self, label, start=None):
        self.notes.setdefault(label, []).append((self.cue, start if start is not None else self.t, self.t))

    def play(self, seconds, label=None):
        start = self.t
        self.f += self.covering(seconds)
        self.events.append((start, self.t, self.cue, label or "play"))
        if label:
            self.note(label, start)

    def fade(self, items, show, label=None, rt=0.35):
        for m in items:
            if show and m.on:
                self.problems.append(f"{self.t:8.3f} {self.cue}: fade IN of {m.name}, already on stage")
            if not show and not m.on:
                self.problems.append(f"{self.t:8.3f} {self.cue}: fade OUT of {m.name}, not on stage (re-root, F1)")
            m.on = show
        self.play(rt, label or ("in " if show else "out ") + ",".join(m.name for m in items[:3]))

    def transform(self, old, new, seconds, label):
        if not old.on:
            self.problems.append(f"{self.t:8.3f} {self.cue}: transform of {old.name}, not on stage")
        old.on, new.on = False, True
        self.play(seconds, label)

    def say(self, cue):
        assert cue not in self.said, cue
        due = self.busy + BREATH
        if self.order and self.t > due + 1.5 / self.fps:
            self.late_lines.append((cue, self.t - due))
        self.wait_to(due)
        self.cue = cue
        self.said[cue] = self.t
        self.order.append(cue)
        self.busy = self.t + self.cues[cue]["len"] * self.scale

    def sentence_at(self, cue, j):
        return self.said[cue] + self.cues[cue]["offsets"][j] * self.scale

    def until(self, cue, j):
        assert cue == self.cue, (cue, self.cue)
        at = self.sentence_at(cue, j)
        if self.t > at + 1.5 / self.fps:
            self.late_beats.append((cue, j, self.t - at))
        self.wait_to(at)

    def phrase_at(self, cue, j, phrase):
        said = sentences(self.text[cue])
        s = said[j]
        share = s.index(phrase) / len(s)
        end = self.sentence_at(cue, j + 1) if j + 1 < len(said) else self.said[cue] + self.cues[cue]["len"] * self.scale
        start = self.sentence_at(cue, j)
        return start + (end - start) * share

    def phrase(self, cue, j, phrase):
        at = self.phrase_at(cue, j, phrase)
        if self.t > at + 1.5 / self.fps:
            self.late_beats.append((cue, f"{j}:{phrase}", self.t - at))
        self.wait_to(at)


def stack(s, label, ones, P):
    rows = []
    if P:
        heading = s.m(f"{label}.heading")
        s.fade([heading], True, f"{label} heading")
        rows.append(heading)
    for i in range(ones):
        r = s.m(f"{label}.row{i}")
        s.fade([r], True)
        rows.append(r)
    rule, result = s.m(f"{label}.rule"), s.m(f"{label}.result")
    s.fade([rule, result], True, f"{label} result")
    rows.append(rule)
    return rows, result


class Word:
    def __init__(self, s):
        self.s = s
        self.fixed = s.ms("box", 14)
        self.digits = s.ms("bit", 7)

    def all(self):
        return self.fixed + self.digits

    def flip(self, p, label):
        new = self.s.m(f"bit{p - 1}'")
        self.s.transform(self.digits[p - 1], new, 0.5, label)
        self.digits[p - 1] = new


def hamming(s, P):
    title = s.m("hamming.title")
    s.fade([title], True)
    word = Word(s)
    s.fade(word.all(), True, "word")
    s.until("17_9", 1)
    data_note = s.m("data_note")
    s.fade([data_note], True, "data note")
    s.until("17_9", 2)
    cells = s.ms("cell", 128)
    s.fade(cells, True, "grid")
    s.play(1.0 + 15 * 0.05 * 1.0, "top row fills")
    count = s.m("count")
    s.fade([count], True, "16 of 128")
    s.until("17_9", 3)
    xor_rows = s.ms("xor", 3)
    s.fade([count], False)
    s.fade(xor_rows, True, "xor rows")

    s.say("17_15")
    s.fade(xor_rows, False)
    s.fade([data_note], False)
    s.until("17_15", 1)
    rows_are = None
    if P:
        rows_are = s.m("rows_are")
        s.fade([rows_are], True, "each row: a coset")
    for row in range(7):
        s.play(0.4 if P else 1.0, f"row {row + 1} fills")
    s.until("17_15", 2)
    lagrange = s.m("lagrange")
    s.fade([lagrange], True, "128/16 = 8")
    s.until("17_15", 3)
    situations = s.ms("situation", 8)
    s.until("17_15", 4)
    for m in situations:
        m.on = True
    s.play(0.4 + 7 * 0.2 * 0.4, "situations")

    s.say("17_16")
    s.fade([lagrange, rows_are] if P else [lagrange], False)
    ring = s.m("ring")
    s.fade([ring], True, "ring")
    word.flip(5, "flip bit 5")
    s.play(1.2, "ring to row 5")
    s.until("17_16", 1)
    labels = s.ms("label", 8)
    s.fade(labels, True, "000-111 labels")
    name = None
    if P and "syndrome" not in s.fix:
        name = s.m("syndrome_name")
        s.fade([name], True, "syndrome name")
    st, result = stack(s, "stack1", 5, P)
    if "syndrome" in s.fix:
        s.phrase("17_16", 1, "called the syndrome")
        name = s.m("syndrome_name")
        s.fade([name], True, "syndrome name")
    s.until("17_16", 3)
    key = s.ms("key", 6)
    s.fade(key, True, "place key")
    s.until("17_16", 4)
    s.play(0.8, "pulse")
    digits = s.ms("slot", 3)
    s.fade(digits, True, "slots 101")
    s.until("17_16", 5)
    worth = s.ms("worth", 3)
    s.fade(worth, True, "worths")
    s.until("17_16", 6)
    total = s.m("place_total")
    s.fade([total], True, "4+0+1=5")
    s.play(1.0, "row label lit")
    s.until("17_16", 7)
    word.flip(5, "repair bit 5")
    s.play(1.2, "ring back")
    s.play(1.0, "row label dim")

    s.say("17_17")
    old = list(st) + [result] + digits + worth + [total]
    if P:
        old.append(name)
    s.fade(old, False, "17_17 wipe")
    frame = s.m("quotient_frame")
    keeps = s.m("keeps")
    frame.on = True
    s.play(0.6, "quotient frame")
    s.fade([keeps], True, "keeps the row")
    s.until("17_17", 1)
    forgets = s.m("forgets")
    s.fade([forgets], True, "forgets the column")
    s.until("17_17", 3)
    word.flip(2, "flip 2")
    word.flip(5, "flip 5")
    s.play(1.2, "ring to row 7")
    st, result = stack(s, "stack2", 4, P)
    s.until("17_17", 4)
    digits = s.ms("slot2.", 3)
    s.fade(digits, True, "slots 111")
    worth = s.ms("worth2.", 3)
    s.fade(worth, True, "worths 421")
    total = s.m("place_total2")
    s.fade([total], True, "4+2+1=7")
    word.flip(7, "miscorrect bit 7")
    s.play(1.2, "ring to row 0")
    sent_col, wrong_col = s.m("sent_col"), s.m("wrong_col")
    sent_col.on = wrong_col.on = True
    s.play(0.5, "column frames")
    verdict = s.m("verdict")
    sent_name = wrong_name = None
    if P:
        sent_name, wrong_name = s.m("sent_name"), s.m("wrong_name")
        s.fade([verdict, sent_name, wrong_name], True, "verdict and names")
    else:
        s.fade([verdict], True, "verdict")

    s.say("17_18")
    s.fade([verdict], False)
    memory = s.m("memory")
    s.fade([memory], True, "memory line")
    s.until("17_18", 1)
    parity, parity_name = s.m("parity"), s.m("parity_name")
    if "flip" in s.fix:
        parity_bit = s.m("parity_bit")
        one_flip, two_flips = s.m("one_flip"), s.m("two_flips")
        s.fade([parity, parity_name], True, "parity box and bit")
        s.phrase("17_18", 1, "correct one flipped bit")
        s.fade([one_flip], True, "one flip line")
        s.phrase("17_18", 1, "while reporting two")
        word.flip(7, "bit 7 back")
        s.play(0.6, "ring back to row 7")
        s.fade([sent_col, wrong_col, sent_name, wrong_name], False, "columns out")
        s.fade([parity_bit, two_flips], True, "two flips line")
    else:
        if P:
            word.flip(7, "bit 7 back")
            s.play(0.6, "ring back to row 7")
            s.fade([sent_col, wrong_col, sent_name, wrong_name], False, "columns out")
        if P:
            parity_bit = s.m("parity_bit")
            s.fade([parity, parity_name, parity_bit], True, "parity box and bit")
        else:
            s.fade([parity, parity_name], True, "parity box")
        one_flip = s.m("one_flip")
        s.fade([one_flip], True, "one flip line")
        two_flips = s.m("two_flips")
        s.fade([two_flips], True, "two flips line")
    s.until("17_18", 2)
    summary = s.m("summary")
    s.fade([forgets], False)
    s.fade([summary], True, "summary")

    out = [title, ring, frame, keeps]
    if not P:
        out += [sent_col, wrong_col]
    out += [parity, parity_name]
    if P:
        out.append(parity_bit)
    out += [memory, one_flip, two_flips, summary, result, total]
    out += key + digits + worth + word.all() + cells + situations + labels + st
    return out


def scene(s, P):
    s.say("17_1")
    title = s.m("title1")
    digits = s.ms("digit", 10)
    s.fade([title], True, "title 1")
    if P:
        s.until("17_1", 1)
    s.fade(digits, True, "ISBN digits")
    if P:
        s.until("17_1", 3)
    note = s.m("note")
    s.fade([note], True, "not a security system")

    s.say("17_2")
    if P:
        s.until("17_2", 1)
    weights = s.ms("weight", 10)
    s.fade(weights, True, "weights")
    if P:
        s.until("17_2", 2)
    products = []
    total = s.m("total0")
    s.fade([total], True, "running total 0")
    divisible = None
    if P:
        s.until("17_2", 3)
        divisible = s.m("divisible")
        s.fade([divisible], True, "divisible by 11 line")

    s.say("17_3")
    if P:
        s.phrase("17_3", 1, "gives weighted contributions")
    rt, hold = (0.1, 0.2) if P else (0.35, 0.15)
    for i in range(10):
        ring = s.m(f"walk_ring{i}")
        s.fade([ring], True, f"column {i} ring", rt=rt)
        p = s.m(f"product{i}")
        products.append(p)
        s.fade([p], True, f"column {i} product", rt=rt)
        nxt = s.m(f"total_after{i}")
        s.fade([total], False, rt=rt)
        s.fade([nxt], True, f"running total after column {i}", rt=rt)
        total = nxt
        s.wait(hold)
        s.fade([ring], False, rt=rt)
    if P:
        s.until("17_3", 3)
    valid = s.m("valid")
    if P:
        s.fade([divisible], False)
    s.fade([valid], True, "132 passes")

    s.say("17_4")
    ring = s.m("ring4")
    s.fade([ring], True, "ring on column 3")
    digit, product = s.m("red7"), s.m("red49")
    if P:
        s.fade([digits[3], valid] if "valid" in s.fix else [digits[3]], False)
        s.fade([digit], True, "red 7")
        digits[3] = digit
        s.until("17_4", 1)
        s.fade([products[3], total] if "valid" in s.fix else [products[3], total, valid], False)
        s.fade([product], True, "red 49")
        products[3] = product
        total = s.m("total139")
        s.fade([total], True, "132 + 7 = 139")
        s.until("17_4", 2)
        fails = s.m("fails")
        s.fade([fails], True, "remainder 7")
    else:
        s.fade([digits[3], products[3], total, valid], False)
        s.fade([digit, product], True, "red 7")
        total = s.m("total139")
        fails = s.m("fails")
        s.fade([total, fails], True, "132 + 7 = 139")

    s.say("17_5")
    old = [title, note, ring, total, fails] if P else [title, note, ring, digit, product, total, fails]
    old += digits + weights + products
    s.fade(old, False, "17_5 wipe")
    heading = s.m("heading5")
    s.fade([heading], True, "17_5 heading")
    if P:
        s.until("17_5", 1)
    rule = s.m("rule5")
    s.fade([rule], True, "rule dS = w dd")
    if P:
        s.until("17_5", 3)
    caption = s.m("caption5")
    row = []
    if P:
        a, b = s.m("change_label"), s.m("product_label")
        s.fade([a, b] if "caption" in s.fix else [caption, a, b], True, "caption and labels")
        row = [a, b]
    else:
        s.fade([caption], True, "caption")
    for d in range(10):
        a, b = s.m(f"dd{d}"), s.m(f"7dd{d}")
        s.fade([a, b], True, f"row column {d}")
        row += [a, b]
    if P:
        s.until("17_5", 4)
    condition = s.m("condition")
    s.fade([caption, condition] if "caption" in s.fix else [condition], True, "prime modulus line")

    s.say("17_6")
    s.fade([heading, rule, caption, condition] + row, False, "17_6 wipe")
    title = s.m("title6")
    s.fade([title], True, "17_6 title")
    dots = s.ms("dot", 6)
    s.fade(dots, True, "six clock")
    jump = s.m("jump")
    s.fade([jump], True, "+3")
    at0 = s.m("at0")
    s.fade([at0], True, "marker at 0")
    s.play(0.9, "marker to 3")
    s.wait(0.5)
    s.play(0.9, "marker to 0")
    fact = s.m("fact")
    s.fade([fact], True, "2x3=0 mod 6")

    s.say("17_7")
    s.fade([title, jump, at0, fact] + dots, False, "17_7 wipe")
    title = s.m("title7")
    s.fade([title], True, "17_7 title")
    swap = s.ms("swap_digit", 10)
    s.fade(swap, True, "swap digits")
    swap_weights = []
    if P:
        swap_weights = s.ms("swap_weight", 2)
        s.fade(swap_weights, True, "x7 x6")
        s.until("17_7", 1)
    before = s.m("before")
    s.fade([before], True, "7x6+6x4=66")
    if "swap" in s.fix:
        s.phrase("17_7", 1, "exchanging the values")
    s.play(0.4, "lane")
    s.play(0.4, "lane")
    s.play(1.0, "digits cross")
    s.play(0.4, "lane")
    s.play(0.4, "lane")
    after = s.m("after")
    s.fade([after], True, "7x4+6x6=64")
    why = None
    delta = s.m("delta7")
    if P:
        if "swap" not in s.fix:
            s.phrase("17_7", 1, "changes the total")
        why = s.m("why0")
        s.fade([delta, why], True, "dS = (7-6)(4-6)")
        for j in (2, 3, 4):
            s.until("17_7", j)
            nxt = s.m(f"why{j}")
            s.fade([why], False)
            s.fade([nxt], True, f"why line {j}")
            why = nxt
    else:
        s.fade([delta], True, "dS = (7-6)(4-6)")

    s.say("17_8")
    if P:
        s.until("17_8", 1)
        s.fade([title, before, after, delta, why] + swap + swap_weights, False, "17_8 wipe")
    else:
        s.fade([title, before, after, delta] + swap, False, "17_8 wipe")
        title = s.m("title8")
        s.fade([title], True, "two errors can cancel")
    shown = s.ms("shown", 10)
    weights = s.ms("weight8.", 10)
    s.fade(shown, True, "17_8 digits")
    s.fade(weights, True, "17_8 weights")
    total = s.m("wt132")
    s.fade([total], True, "weighted total 132")
    wheel = s.ms("wheel", 11)
    s.fade(wheel, True, "wheel")
    marker = s.m("marker")
    marker.on = True
    legend = s.m("legend")
    s.fade([legend], True, "legend")
    if P:
        s.until("17_8", 2)
        title = s.m("title8")
        s.fade([title], True, "two errors can cancel")
    s.until("17_8", 3)
    nxt = s.m("first_red")
    s.fade([shown[0]], False)
    shown[0] = nxt
    s.fade([nxt], True, "first digit")
    delta = s.m("delta8a")
    s.fade([delta], True, "first change adds 10")
    nxt = s.m("wt142")
    s.fade([total], False)
    total = nxt
    s.fade([total], True, "weighted total 142")
    s.play(1.5, "marker moves")
    s.until("17_8", 4)
    nxt = s.m("second_red")
    s.fade([shown[1]], False)
    shown[1] = nxt
    s.fade([nxt], True, "second digit")
    s.fade([delta], False)
    delta = s.m("delta8b")
    s.fade([delta], True, "+45")
    nxt = s.m("wt187")
    s.fade([total], False)
    total = nxt
    s.fade([total], True, "weighted total 187")
    s.play(1.5, "marker moves")
    s.until("17_8", 5)
    limit = s.m("limit")
    s.fade([limit], True, "the error hides")

    s.say("17_14")
    s.say("17_9")
    s.fade([title, total, delta, legend, marker, limit] + shown + weights + wheel, False, "17_9 wipe")
    empty_before = [m for m in (title, total, delta, legend, marker, limit) if m.on]
    assert not empty_before
    ham = hamming(s, P)

    s.say("17_11")
    s.fade(ham, False, "hamming wipe")
    s.leftover = [m.name for m in ham if m.on]
    title = s.m("title11")
    s.fade([title], True, "follow the operation")
    clock = s.ms("clock", 12)
    s.fade(clock, True, "twelve clock")
    triangles = s.ms("triangle", 4)
    if P:
        square = s.m("square")
        s.fade([square], True, "square thumb")
        clock.append(square)
        s.until("17_11", 1)
        group = s.m("group")
        s.fade([group], True, "group line")
        s.until("17_11", 2)
        subgroup = s.m("subgroup")
        s.fade([triangles[0], subgroup], True, "blue triangle and subgroup line")
        s.until("17_11", 3)
        iso = s.m("isomorphism")
        s.fade([iso], True, "isomorphism line")
        s.until("17_11", 4)
        partition, quotient = s.m("partition"), s.m("quotient")
        for t in triangles[1:]:
            s.fade([t], True, "another triangle")
        s.fade([partition], True, "cosets line")
        s.fade([quotient], True, "normal subgroup line")
    else:
        subgroup = s.m("subgroup")
        s.fade([subgroup], True, "subgroup line")
        for t in triangles:
            s.fade([t], True, "triangle")
        clock += triangles
        partition, quotient = s.m("partition"), s.m("quotient")
        s.fade([partition, quotient], True, "cosets and quotient lines")

    s.say("17_12")
    if P:
        clock += triangles + [title, group, subgroup, iso, partition, quotient]
    else:
        clock += [title, subgroup, partition, quotient]
    s.fade(clock, False, "17_12 wipe")
    quiz = []
    for i in range(3):
        if P:
            s.until("17_12", i + 1)
        q = s.m(f"prompt{i}")
        s.fade([q], True, f"prompt {i + 1}")
        quiz.append(q)
    s.wait_to(s.busy + BREATH)  # k.hold
    s.wait(5.0)
    s.think_end = s.t

    s.say("17_13")
    s.fade(quiz, False, "prompts out")
    title = s.m("title13")
    s.fade([title], True, "answers title")
    answer = s.ms("answer", 12)
    s.fade(answer, True, "answer clock")
    selection = s.m("selection")
    s.fade([selection], True, "selection")
    for _ in range(2):
        s.play(0.8, "selection moves")
        s.wait(0.4)
    if P:
        s.until("17_13", 1)
    a, b = s.m("a"), s.m("b")
    s.fade([a, b], True, "answer 2")
    if P:
        s.until("17_13", 2)
    c = s.m("c")
    s.fade([c], True, "hH=H")
    d = s.m("d")
    s.fade([d], True, "closure line")
    s.wait_to(s.busy + BREATH)  # k.finish holds


def run(P, fps=30, scale=1.0, fix=()):
    cues, text = scratch_table()
    s = Sim(cues, text, fps, scale, fix)
    scene(s, P)
    return s


def holds(s, at_least):
    out = []
    ev = sorted(s.events)
    for (a0, a1, cue, label), (b0, _b1, _cue2, label2) in zip(ev, ev[1:]):
        if b0 - a1 >= at_least:
            out.append((b0 - a1, a1, b0, cue, label, label2))
    return sorted(out, reverse=True)


def main():
    cues, text = scratch_table()
    print("# sentence table check: each cue's offsets count against the script's sentences")
    bad = [c for c in cues if len(cues[c]["offsets"]) != len(sentences(text[c]))]
    print("  mismatched cues:", bad or "none")

    print("\n# positive control: baseline mirror at 30 fps, scale 1, against the render's cue table")
    base = run(False)
    worst = max(abs(base.said[c] - cues[c]["start"]) for c in cues)
    print(f"  order: {' '.join(base.order)}")
    print(f"  largest line-start difference from the render: {worst:.3f} s")
    print("  lines delayed by the picture:", base.late_lines or "none")
    print("  lifecycle findings:")
    for p in base.problems:
        print("   ", p)
    print("  left on stage after the Hamming wipe:", base.leftover or "none")
    print(f"  think gap, prompts end to 17_13 start: last prompt at {base.notes['prompt 3'][0][2]:.3f}, 17_13 at {base.said['17_13']:.3f}")

    print("\n# proposed mirror at 30 fps, scale 1")
    prop = run(True)
    worst = max(abs(prop.said[c] - cues[c]["start"]) for c in cues)
    print(f"  order: {' '.join(prop.order)}")
    print(f"  largest line-start difference from the scratch table: {worst:.3f} s")
    print("  lines delayed by the picture:", prop.late_lines or "none")
    print("  beats that miss their wait (cue, sentence, seconds late):", [(c, j, round(x, 2)) for c, j, x in prop.late_beats] or "none")
    print("  lifecycle findings:", prop.problems or "none")
    print("  left on stage after the Hamming wipe:", prop.leftover or "none")
    print(f"  17_13 starts at {prop.said['17_13']:.3f}; last prompt fully up at {prop.notes['prompt 3'][0][2]:.3f}")

    print("\n# the review's estimated-timing table, against this mirror at 60 fps, scale 1 (final frame grid)")
    p60 = run(True, fps=60)
    b60 = run(False, fps=60)
    rows = [
        "ISBN digits", "not a security system", "weights", "running total 0", "divisible by 11 line",
        "132 passes", "red 7", "red 49", "132 + 7 = 139", "remainder 7", "rule dS = w dd", "caption and labels",
        "row column 9", "prime modulus line", "x7 x6", "7x6+6x4=66", "digits cross", "7x4+6x6=64", "dS = (7-6)(4-6)", "why line 2", "why line 3",
        "why line 4", "17_8 wipe", "legend", "two errors can cancel", "square thumb", "blue triangle and subgroup line",
        "another triangle", "cosets line", "normal subgroup line", "prompt 1", "prompt 2", "prompt 3", "answer 2", "hH=H",
        "each row: a coset", "row 7 fills", "128/16 = 8", "syndrome name", "bit 7 back", "columns out", "parity box and bit", "one flip line", "two flips line",
    ]
    for label in rows:
        got = p60.notes.get(label)
        if got:
            cue, a, b = got[-1]
            old = b60.notes.get(label)
            was = f"{old[-1][1]:8.3f}" if old else "    none"
            print(f"  {cue:6} {label:34} starts {a:8.3f} ends {b:8.3f}   baseline start {was}")
    walk = p60.notes["column 0 ring"][0][1], p60.notes["running total after column 9"][0][2]
    print(f"  17_3 walk: starts {walk[0]:.3f}, total 132 fully up at {walk[1]:.3f}")

    print("\n# long static gaps between any two picture changes (at least 8 s), 60 fps, scale 1")
    for name, s in (("baseline", b60), ("proposed", p60)):
        print(f"  {name}:")
        for gap, a, b, cue, l1, l2 in holds(s, 8.0):
            print(f"    {gap:6.2f} s  {a:8.3f} to {b:8.3f}  {cue:6} after '{l1}' until '{l2}'")

    print("\n# can waits or new beats delay later actions? Voice scaled against fixed picture durations, 60 fps")
    print("  scale = Oliver/scratch line length. Measured on cues that have takes (chapters 10 and 13): 0.87 to 1.25, median 1.07")
    for scale in (0.75, 0.85, 1.0, 1.07, 1.25):
        for name, P in (("baseline", False), ("proposed", True)):
            s = run(P, fps=60, scale=scale)
            late = [(c, round(x, 2)) for c, x in s.late_lines]
            beats = [(c, j, round(x, 2)) for c, j, x in s.late_beats if P or x > 0]
            print(f"  scale {scale:4.2f} {name}: lines delayed {late or 'none'}; beats late {beats or 'none'}; lifecycle {len(s.problems)}")

    print("\n# voice words against picture, proposed, scale 1 and 1.07 (voice by share of characters; estimates)")
    for scale in (1.0, 1.07):
        s = run(True, fps=60, scale=scale)
        print(f"  scale {scale}")
        checks = [
            ("17_3 first number 'zero, twenty-seven'", s.phrase_at("17_3", 1, "zero, twenty-seven"), s.notes["column 0 product"][0][1]),
            ("17_3 'twenty-seven'", s.phrase_at("17_3", 1, "twenty-seven"), s.notes["column 1 product"][0][1]),
            ("17_3 'and two' (last number)", s.phrase_at("17_3", 1, "and two"), s.notes["column 9 product"][0][1]),
            ("17_3 'The total is' sentence 2", s.sentence_at("17_3", 2), s.notes["running total after column 9"][0][1]),
            ("17_7 'exchanging the values'", s.phrase_at("17_7", 1, "exchanging the values"), s.notes["digits cross"][0][1]),
            ("17_8 'Here is a concrete failure'", s.sentence_at("17_8", 2), s.notes["17_8 wipe"][0][1]),
            ("17_16 'called the syndrome'", s.phrase_at("17_16", 1, "called the syndrome"), s.notes["syndrome name"][0][1]),
            ("17_18 'parity check'", s.phrase_at("17_18", 1, "parity check"), s.notes["parity box and bit"][0][1]),
            ("17_18 'correct one flipped bit'", s.phrase_at("17_18", 1, "correct one flipped bit"), s.notes["one flip line"][0][1]),
            ("17_18 'while reporting two'", s.phrase_at("17_18", 1, "while reporting two"), s.notes["two flips line"][0][1]),
            ("17_18 'instead of miscorrecting'", s.phrase_at("17_18", 1, "instead of miscorrecting"), s.notes["bit 7 back"][0][1]),
            ("17_13 'The four-clock' sentence 1", s.sentence_at("17_13", 1), s.notes["answer 2"][0][1]),
            ("17_5 'Because eleven is prime' s4, caption", s.sentence_at("17_5", 4), s.notes["caption and labels"][0][1]),
        ]
        for label, voice, picture in checks:
            print(f"    {label:44} voice {voice:8.3f}  picture {picture:8.3f}  picture leads by {voice - picture:6.2f} s")

    print("\n# new labels against the word they describe, proposed, 60 fps, scale 1")
    s = run(True, fps=60)
    a = s.notes["repair bit 5"][0][2]
    b = s.notes["17_17 wipe"][0][2]
    print(f"  'positions holding a 1' over stack 1 after bit 5 is repaired: {a:.3f} to {b:.3f}, {b - a:.2f} s")
    a = s.notes["miscorrect bit 7"][0][2]
    b = s.notes["bit 7 back"][0][1]
    print(f"  'positions holding a 1' over stack 2 after the miscorrection: {a:.3f} to {b:.3f}, {b - a:.2f} s")
    a = s.notes["red 7"][0][2]
    b = s.notes["red 49"][0][1] - 0.35
    print(f"  green 'passes' beside the red 7: {a:.3f} to {b:.3f}, {b - a:.2f} s, on a scale-1 voice")
    print(f"  chapter length: baseline {run(False, fps=60).t:.3f}, proposed {s.t:.3f}")
    waits = sum(1 for _ in s.events)  # plays, for scale only
    print(f"  plays in the mirror: {waits}")

    print("\n# corrections on the same clock")
    layers = [("C1", ("valid",)), ("C1+C2", ("valid", "caption", "swap", "syndrome", "flip"))]
    for name, fix in layers:
        for scale in (0.75, 0.85, 1.0, 1.07, 1.25):
            c = run(True, fps=60, scale=scale, fix=fix)
            base_starts = run(True, fps=60, scale=scale)
            moved = [k for k in c.said if abs(c.said[k] - base_starts.said[k]) > 1e-6]
            print(
                f"  {name:6} scale {scale:4.2f}: lifecycle {len(c.problems)}; line starts moved {moved or 'none'}; "
                f"lines delayed {[(k, round(x, 2)) for k, x in c.late_lines]}; beats late {[(k, j, round(x, 2)) for k, j, x in c.late_beats] or 'none'}"
            )
    c = run(True, fps=60, fix=layers[1][1])
    print("  C1+C2 at scale 1, voice against picture:")
    for label, voice, picture in [
        ("17_5 'Because eleven is prime', caption", c.sentence_at("17_5", 4), c.notes["prime modulus line"][0][1]),
        ("17_7 'exchanging the values', lanes start", c.phrase_at("17_7", 1, "exchanging the values"), c.notes["digits cross"][0][1] - 0.8),
        ("17_7 'changes the total', 64 starts", c.phrase_at("17_7", 1, "changes the total"), c.notes["7x4+6x6=64"][0][1]),
        ("17_7 'by the product', dS starts", c.phrase_at("17_7", 1, "by the product"), c.notes["dS = (7-6)(4-6)"][0][1]),
        ("17_16 'called the syndrome', name", c.phrase_at("17_16", 1, "called the syndrome"), c.notes["syndrome name"][0][1]),
        ("17_18 'parity check', box", c.phrase_at("17_18", 1, "parity check"), c.notes["parity box and bit"][0][1]),
        ("17_18 'correct one flipped bit', line", c.phrase_at("17_18", 1, "correct one flipped bit"), c.notes["one flip line"][0][1]),
        ("17_18 'while reporting two', bit 7 back", c.phrase_at("17_18", 1, "while reporting two"), c.notes["bit 7 back"][0][1]),
        ("17_18 'instead of miscorrecting', line", c.phrase_at("17_18", 1, "instead of miscorrecting"), c.notes["two flips line"][0][1]),
    ]:
        print(f"    {label:44} voice {voice:8.3f}  picture {picture:8.3f}  picture leads by {voice - picture:6.2f} s")
    a, b = c.notes["two flips line"][0][2], c.sentence_at("17_18", 2)
    print(f"    17_18 last beat of sentence 1 ends {a:.3f}; sentence 2 starts {b:.3f}; margin {b - a:.2f} s")
    print("  C1+C2 long static gaps (at least 8 s):")
    for gap, a, b, cue, l1, l2 in holds(c, 8.0):
        print(f"    {gap:6.2f} s  {a:8.3f} to {b:8.3f}  {cue:6} after '{l1}' until '{l2}'")


if __name__ == "__main__":
    sys.exit(main())
