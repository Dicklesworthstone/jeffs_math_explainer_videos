"""Desk checks for the staged chapter 11 picture repair. Run from the repo root.

No build, no render: geometry, spoken truth, caption widths, sentence order and
timing budgets, worked from the candidate's source and the script.
"""
import math
import re

O = "docs/video-program/third-wave-20261024/opus-replacements-20261104/algebra-picture-review-11_permutations/"
CAND = open(O + "candidate/permutations.rs", encoding="utf-8").read()
BASE = open(O + "baseline/permutations.rs", encoding="utf-8").read()
SCRIPT = "explainer_corpus/abstract_algebra_expanded/script.tsv"
CUES = "narration_lab/third-wave-20261024/root/five-film-release-20261027/algebra-combined-repair-preview01/render/11_permutations.cues.tsv"
SEP = 0.16
fails = []


def check(ok, what):
    print(("ok   " if ok else "FAIL ") + what)
    if not ok:
        fails.append(what)


# ---- 1. link_offsets, ported ----------------------------------------------
def link_offsets(cycle):
    n = len(cycle)
    if n < 3:
        return [(0.0, 0.0)] * n

    def tail_right(i):
        at, frm, to = cycle[i], cycle[(i + n - 1) % n], cycle[(i + 1) % n]
        return 1.0 / (to - at) > 1.0 / (frm - at)

    out = []
    for i in range(n):
        tail = SEP if tail_right(i) else -SEP
        head = -SEP if tail_right((i + 1) % n) else SEP
        out.append((tail, head))
    return out


def ends(cycle, offsets):
    """Each link as (tail x, head x) in slot units, and the slots' spacing is applied by the caller."""
    n = len(cycle)
    return [((cycle[i], offsets[i][0]), (cycle[(i + 1) % n], offsets[i][1])) for i in range(n)]


def touches_and_crossings(cycle, spacing, offsets):
    arcs = [(a * spacing + da, b * spacing + db) for (a, da), (b, db) in ends(cycle, offsets)]
    touching, crossing = 0, 0
    for i in range(len(arcs)):
        for j in range(i + 1, len(arcs)):
            p, q = sorted(arcs[i]), sorted(arcs[j])
            if set(p) & set(q):
                touching += 1
            elif p[0] < q[0] < p[1] < q[1] or q[0] < p[0] < q[1] < p[1]:
                crossing += 1
    return touching, crossing


print("== 1. head and tail at each slot; arcs that touch or cross on one side of the row")
for name, cycle, spacing in [
    ("11_4 and 11_12 (1 3 5)", [1, 3, 5], 1.5),
    ("11_15 (1 2 3 4)", [1, 2, 3, 4], 1.5),
    ("11_22 six-cycle", [2, 3, 5, 9, 8, 6], 1.2),
]:
    off = link_offsets(cycle)
    n = len(cycle)
    for i in range(n):
        tail, head = off[i][0], off[(i + n - 1) % n][1]
        check(head == -tail and abs(tail) == SEP, f"{name}: slot {cycle[i]} head {head:+.2f}, tail {tail:+.2f}")
    before = touches_and_crossings(cycle, spacing, [(0.0, 0.0)] * n)
    after = touches_and_crossings(cycle, spacing, off)
    print(f"     {name}: baseline touching/crossing {before}; candidate {after}")
    check(after[0] == 0, f"{name}: no head or tail shares a point")
    check(after[1] <= before[1], f"{name}: no crossing added")
for swap in ([2, 6], [5, 6], [4, 7]):
    check(link_offsets(swap) == [(0.0, 0.0)] * 2, f"swap {tuple(swap)} keeps one shared anchor pair")


# ---- 2. Arc tops against the words above them ------------------------------
def sagitta(chord, arc):
    return (chord / 2.0) * math.tan(abs(arc) / 4.0)


print("\n== 2. Arc tops, baseline and candidate, against the lowest edge of the words above (size/100 tall about)")
for name, base_y, chord, arc, word_y, size in [
    ("11_4 closing", 1.41, 6.0, 1.2, 3.0, 34),
    ("11_4 wrong", 1.41, 3.0, 2.2, 3.0, 34),
    ("11_12 closing", 1.66, 6.0, 1.2, 3.05, 32),
    ("11_22 five to nine (head moved out)", 1.66, 4.8, 1.0, 3.05, 32),
    ("11_22 six to two", 1.66, 4.8, 1.0, 3.05, 32),
]:
    # In the two 11_22 arcs both ends move the same way, so the chord does not change.
    grow = 0.0 if "11_22" in name else 2 * SEP
    if name == "11_4 wrong":
        grow = 2 * SEP
    top_b, top_c = base_y + sagitta(chord, arc), base_y + sagitta(chord + grow, arc)
    limit = word_y - size * 0.01 * 0.6
    check(top_c < limit, f"{name}: top {top_b:.2f} -> {top_c:.2f}, words begin {limit:.2f}")

print("\n== 3. The moved slot numbers (L)")
TRAIL = [(-3.2, 1.3), (4.7, -1.1), (3.2, 1.3), (-3.2, -1.1)]
written_top = -2.4 + 0.38 * 0.6
for slot, (x, y) in enumerate(TRAIL, 1):
    label_y = y - 0.62 if y < 0 else y + 0.62
    # the only links are 1-3, 3-4, 4-1 in either direction
    segments = [(TRAIL[0], TRAIL[2]), (TRAIL[2], TRAIL[3]), (TRAIL[3], TRAIL[0])]
    clear = 9.0
    for (ax, ay), (bx, by) in segments:
        dx, dy = bx - ax, by - ay
        t = max(0.0, min(1.0, ((x - ax) * dx + (label_y - ay) * dy) / (dx * dx + dy * dy)))
        clear = min(clear, math.hypot(x - ax - t * dx, label_y - ay - t * dy))
    check(clear > 0.2, f"slot {slot} number at ({x}, {label_y:.2f}): {clear:.2f} from the nearest link")
    if y < 0:
        check(label_y - 0.12 > written_top, f"slot {slot} number clears the written inverse (top {written_top:.2f})")

# ---- 4. Spoken and printed truth ---------------------------------------------
print("\n== 4. Truth of what the new pictures say")


def perfect_shuffle(n, out):
    half = n // 2
    p = [0] * n
    for i in range(half):
        p[i] = 2 * i if out else 2 * i + 1
        p[half + i] = 2 * i + 1 if out else 2 * i
    return p


p10 = perfect_shuffle(10, True)
check(p10[1] == 2, "AA: after one out-shuffle of ten, card 2 is in slot 3, so 'Card 2 went from slot 2 to slot 3' holds")
check(p10[0] == 0 and p10[9] == 9, "AA: slots 1 and 10 stay, so the first moving trail starts at 2")

w = perfect_shuffle(8, True)
state = list(range(8))
for _ in range(3):
    state = [w[s] for s in state]
check(state == list(range(8)), "AH: after 3 stages every token is back on its own line")
state = [w[s] for s in state]
check(state == w, "AG: stage 4 sends tokens exactly as stage 1 did")

row = [0, 1, 2, 3]


def trade(a, b):
    for i, s in enumerate(row):
        if s == a:
            row[i] = b
        elif s == b:
            row[i] = a


for a, b in [(0, 1), (1, 2), (1, 2), (0, 1)]:
    trade(a, b)
check(row == [0, 1, 2, 3], "T: the demonstration ends with the row at home, for the uncounted start of sentence 1")

p52 = perfect_shuffle(52, True)
seen, sizes = set(), []
for i in range(52):
    if i not in seen:
        c, j = 0, i
        while j not in seen:
            seen.add(j)
            j = p52[j]
            c += 1
        sizes.append(c)
check(sorted(sizes) == [1, 1, 2] + [8] * 6, "AC: 2 fixed, 1 swap, 6 cycles of length 8")
order = 1
for c in sizes:
    order = order * c // math.gcd(order, c)
check(order == 8, "AD: the pulse joins lcm(1,2,8)=8 with the counter at 8")

# ---- 5. Widths: CMR10 ems x size x 0.01; 0.50 em a character is on the wide side ---
print("\n== 5. New or changed words, width at 0.50 em a character, half-frame 7.11")
for text, size, cx in [
    ("Transposition: swap two labels", 32, 0.0),
    ("Gold: even", 26, -0.15),
    ("Blue: odd", 26, -0.15),
    ("Card 2 went from slot 2 to slot 3", 27, 0.0),
    ("By size: 2 fixed, 1 swap, 6 cycles of length 8", 27, 0.0),
    ("After 3 stages: all data back on their starting lines", 27, 0.0),
    ("Stage 4", 22, 3.9),
]:
    width = len(text) * 0.50 * size * 0.01
    check(abs(cx) + width / 2 < 7.11 - 0.3, f"{text!r}: {width:.2f} wide at x {cx}")
    if "Gold" in text or "Blue" in text:
        check(cx - width / 2 > -2.0 and cx + width / 2 < 1.7, f"{text!r} clears both blocks of cells")

# ---- 6. Sentence order, and nothing faded that never appeared -----------------
print("\n== 6. Sentence waits in order, within the chapter's counts")
counts = dict(re.findall(r'\("(11_\d+)", (\d+)\),', CAND))
order_by_cue = {}
for cue, j in re.findall(r'until_sentence\(stage, "(11_\d+)", (\d+)\)', CAND):
    order_by_cue.setdefault(cue, []).append(int(j))
for cue, seq in sorted(order_by_cue.items(), key=lambda kv: int(kv[0][3:])):
    if cue == "11_7":
        # witness_proof holds sentence 2 and sits above construct in the file; construct waits for 1, then calls it.
        ordered = CAND.index('until_sentence(stage, "11_7", 1)') < CAND.index("witness_proof(stage, &k)")
    else:
        ordered = seq == sorted(seq)
    check(ordered and max(seq) < int(counts[cue]), f"{cue}: {seq} of {counts[cue]}")

for name in ["swap_name", "swap_cards", "even_key", "odd_key", "reading", "fourth"]:
    bound = rf"let (mut )?{name}\b"
    check(len(re.findall(bound, CAND)) == 1 and not re.search(bound, BASE) and len(re.findall(rf"\b{name}\b", CAND)) >= 3, f"{name}: bound once, new, and used")
check(CAND.count("swap_caption(") == 6, "swap_caption: helper and the five sites")
check(CAND.count("exchange(") == BASE.count("exchange(") - 5, "exchange: five sites moved, the rest untouched")
for stale in ["every line back", "above_top(4), above_top(2)", "above_top(0), above_top(2)", "above_top(4), above_top(0)"]:
    check(stale not in CAND, f"gone: {stale}")

# ---- 7. Timing budgets on the scratch voice's word rate ---------------------------
print("\n== 7. Budgets on the scratch voice (a cue's length spread over its words)")
script = {}
for line in open(SCRIPT, encoding="utf-8"):
    parts = line.rstrip("\n").split("\t")
    if len(parts) >= 2 and parts[0].startswith("11_"):
        script[parts[0]] = parts[-1]
lengths = {}
for line in open(CUES, encoding="utf-8"):
    c, s, l = line.split("\t")
    lengths[c] = float(l)


def sentences(text):
    return re.split(r"(?<=[.?!])\s+", text.strip())


def window(cue, j, upto=None):
    parts = sentences(script[cue])
    words = sum(len(x.split()) for x in parts)
    rate = lengths[cue] / words
    upto = j + 1 if upto is None else upto
    return sum(len(x.split()) for x in parts[j:upto]) * rate


def walk(legs):
    return 0.4 + legs * (0.2 + 0.95) + 0.4


rows = [
    ("I", "11_5", 0, 2, 0.4 + 0.4 + 0.9, 0.4),
    ("I", "11_5", 2, 3, 0.4 + 0.4 + 0.4, 0.8),
    ("L", "11_4", 0, 1, None, None),
    ("T", "11_20", 0, 1, 0.4 + 0.4 + 4 * 0.6 + 0.3, 0.4),
    ("V", "11_21", 3, 4, 0.3 + 0.3 + 0.4 + 0.6 + 0.6, 1.2),
    ("AA", "11_22", 2, 3, 0.4 + walk(6), walk(6)),
    ("AA", "11_22", 3, 4, 0.4 + walk(2), walk(2)),
    ("AE", "11_25", 0, 1, 0.8 + 0.8 + 0.3, 0.3 + 0.8 + 0.3),
    ("AE 645.5", "11_25", 2, 3, 7 * (0.28 + 0.3) + 0.8, 7 * (0.28 + 0.3) + 0.3),
    ("AE 657.5", "11_26", 1, 2, 0.8, 0.3),
    ("AI", "11_26", 3, 4, 3 * 0.7 + 2 * 0.25 + 0.4, 3 * 0.7 + 0.4),
    ("AG", "11_26", 4, 5, 0.4 + 0.7, 0.4 + 0.7),
    ("AM", "11_27", 1, 2, 0.45 + 0.4 + 0.3 + 7 * 0.22, 0.45 + 0.4 + 7 * 0.22),
    ("G", "11_14", 3, 4, 0.8, 0.3),
    ("J", "11_17", 4, 5, 0.8, 0.3),
]
for tag, cue, j, upto, after, before in rows:
    if after is None:
        continue
    span = window(cue, j, upto)
    print(f"{'ok  ' if after <= span else 'LATE'} {tag:9s} {cue} sentences {j}..{upto - 1}: window {span:5.2f} s, block {before:5.2f} -> {after:5.2f} s")
    if after > span:
        fails.append(f"{tag} overruns its window")

print("\nAD pushes 11_25 and everything after it by 1.2 s (0.6 pulse + 0.6 hold). X moves the sr picture by up to 0.35 s.")
print("\nFAILS:", fails if fails else "none")
