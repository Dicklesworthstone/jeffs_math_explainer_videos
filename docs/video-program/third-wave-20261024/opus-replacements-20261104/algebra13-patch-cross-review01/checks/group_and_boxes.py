"""Independent checks of the mathematics and static claims of candidate A+B, read as text from its source.

Imports nothing from BronzeJaguar's bundle. Runs both oracle.py copies as subprocesses from a scratch-free cwd
(they only print). Usage (repo root): python3 -I -B <this file> <candidate root> <baseline root>
"""
import math
import pathlib
import re
import subprocess
import sys

root, base = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
CRATE = "explainer_corpus/abstract_algebra_expanded"
src = (root / CRATE / "src/arithmetic.rs").read_text(encoding="utf-8")
script = {}
for row in pathlib.Path(CRATE, "script.tsv").read_text(encoding="utf-8").splitlines():
    cols = row.split("\t")
    if len(cols) >= 2 and cols[0].startswith("13_"):
        script[cols[0]] = cols[1]
bad = 0


def check(ok, text):
    global bad
    print(("ok   " if ok else "FAIL ") + text)
    bad += 0 if ok else 1


def mat(a, b):
    return [[sum(a[i][k] * b[k][j] for k in range(3)) for j in range(3)] for i in range(3)]


def turn(t):
    c, s = math.cos(math.pi / 2 * t), math.sin(math.pi / 2 * t)
    return [[c, -s, 0], [s, c, 0], [0, 0, 1]]


def flip(t):
    c, s = math.cos(math.pi * t), math.sin(math.pi * t)
    return [[1, 0, 0], [0, c, -s], [0, s, c]]


def same(a, b):
    return all(abs(a[i][j] - b[i][j]) < 1e-9 for i in range(3) for j in range(3))


print("== 1 Elem against matrices, d4_group's rule rewritten from its definition")
ALL = [(t, f) for f in (False, True) for t in range(4)]
TEX = ["e", "r", "r^2", "r^3", "s", "rs", "r^2s", "r^3s"]
E, RR, R2, R3, SS, RS, R2S, R3S = ALL


def mul(a, b):
    carried = (4 - b[0]) % 4 if a[1] else b[0]
    return ((a[0] + carried) % 4, a[1] != b[1])


def matrix(g):
    return mat(turn(g[0]), flip(1)) if g[1] else turn(g[0])


def tex(g):
    return TEX[ALL.index(g)]


rust = pathlib.Path(CRATE, "src/d4_group.rs").read_text(encoding="utf-8")
check("(4 - other.turns) % 4" in rust and '["e", "r", "r^2", "r^3", "s", "rs", "r^2s", "r^3s"]' in rust, "d4_group.rs has the rule and the TeX table mirrored here")
check(all(same(matrix(mul(a, b)), mat(matrix(a), matrix(b))) for a in ALL for b in ALL), "all 64 products equal composed matrices, left acting last")

print("\n== 2 closure.product")
pair = re.search(r"const CLOSURE_PAIR: \(Elem, Elem\) = \((\w+), (\w+)\);", src).groups()
names = {"S": SS, "R": RR}
p, q = names[pair[0]], names[pair[1]]
check(pair == ("S", "R"), f"CLOSURE_PAIR is {pair}")
check(tex(mul(p, q)) == "r^3s", f"s r = {tex(mul(p, q))}, which main.rs emits through .tex()")
main = pathlib.Path(CRATE, "src/main.rs").read_text(encoding="utf-8")
check('facts.add("arithmetic.closure.product", p.mul(q).tex())' in main, "main.rs emits p.mul(q).tex(), not .name(), so the fact is ASCII r^3s")
check(same(matrix(mul(p, q)), mat(flip(1), turn(1))), "s r composed: the turn acts first, as 13_5 says for s r")
check(same(matrix(mul(p, q)), mat(turn(-1), flip(1))), "s r = r inverse s, the rule 13_7 states")


def run_oracle(where):
    out = subprocess.run(["/usr/bin/python3", "-I", "-B", str(where / CRATE / "oracle.py")], capture_output=True, text=True, check=True)
    rows = {}
    for row in out.stdout.splitlines():
        key, _, value = row.partition("\t")
        if not value:
            key, _, value = row.partition(" ")
        rows[key] = value
    return rows


new, old = run_oracle(root), run_oracle(base)
diff = sorted(k for k in set(new) | set(old) if new.get(k) != old.get(k))
print(f"     oracle rows: baseline {len(old)}, candidate {len(new)}; differing keys {diff}")
check(diff == ["arithmetic.closure.product"], "exactly one oracle key changes")
check(old["arithmetic.closure.product"].strip() == "rs" and new["arithmetic.closure.product"].strip() == "r^3s", "it moves from rs to r^3s, equal to the Rust side")
check(new["arithmetic.closure.product"] == new["arithmetic.swap.ab"], "it now equals swap.ab, the same product 13_9 uses")

print("\n== 3 tile poses against names, using the candidate's own move lists")


def moves(name):
    body = re.search(rf"const {name}: \[Move; \d+\] = \[(.*?)\];", src, re.S).group(1)
    return re.findall(r"Move::(Turn\(-?[\d.]+\)|Flip)", body)


def pose(ms, t):
    state = turn(0)
    for i, mv in enumerate(ms):
        frac = min(max(t - i, 0.0), 1.0)
        nxt = flip(frac) if mv == "Flip" else turn(float(mv[5:-1]) * frac)
        state = mat(nxt, state)
    return state


LEFT, RIGHT, ROUTE, SEQ = moves("LEFT"), moves("RIGHT"), moves("LONG_ROUTE"), moves("SEQUENCE")
named = [("left", LEFT, 0, E), ("left", LEFT, 2, R2), ("left", LEFT, 3, R3), ("left", LEFT, 4, E), ("left", LEFT, 5, SS), ("left", LEFT, 6, E),
         ("left", LEFT, 8, RS), ("right", RIGHT, 0, E), ("right", RIGHT, 2, R3S)]
for who, ms, t, g in named:
    check(same(pose(ms, t), matrix(g)), f"{who} at t={t}: {tex(g)}")
check(tex(mul(RR, SS)) == "rs" and tex(mul(SS, RR)) == "r^3s", "13_5's r s and s r are different elements")
check(same(pose(ROUTE, 5), turn(1)), "13_1: the long route ends where one quarter-turn ends")
check(same(pose(SEQ, 3), turn(-1)), "13_6: flip, turn, flip equals the opposite quarter-turn")
check(same(pose(SEQ, 3), matrix(R3)), "13_6: and r cubed")
for t, g in ((1, SS), (2, RS), (3, R3)):
    check(same(pose(SEQ, t), matrix(g)), f"13_9: both tiles at t={t} show {tex(g)}")

print("\n== 4 13_7 and 13_12 algebra")
word = [SS, RR, SS, R2, SS]
inv = {g: next(h for h in ALL if mul(g, h) == E) for g in ALL}


def product(gs):
    out = E
    for g in gs:
        out = mul(out, g)
    return out


check(mul(SS, RR) == mul(inv[RR], SS), "s r = r⁻¹ s, the rewrite of the first two factors")
check(product([inv[RR], SS, SS, R2, SS]) == product(word), "after moving the flip: r⁻¹ s s r² s")
check(product([inv[RR], R2, SS]) == product(word), "after the pair cancels: r⁻¹ r² s")
check(product([RR, SS]) == product(word) and tex(product(word)) == "rs", "after the turns combine: r s")
check(mul(RR, mul(R3S, E)) == SS, "r (r³s) = s")
check(next(g for g in ALL if mul(RR, g) == SS) == R3S and mul(inv[RR], SS) == R3S, "x = r⁻¹ s = r³s, and it is the only solution")
check(mul(E, RR) == RR == mul(RR, E), "e r = r = r e")
check(ALL.index(SS) == 4 and ALL.index(RR) == 1 and ALL.index(R3S) == 7, "closure frames land on rack slots 4, 1 and 7")

print("\n== 5 13_10 trail boxes, from the candidate's constants")


def const_num(name):
    return float(re.search(rf"const {name}: f64 = (-?[\d.]+);", src).group(1))


MX = [float(x) for x in re.search(r"const MACHINE_X: \[f64; 3\] = \[(.*?)\];", src).group(1).split(",")]
CHIP = const_num("CHIP_IN")
first_two = (CHIP - 0.35, MX[1] + 0.35)
last_two = (MX[0] - 0.35, MX[2] + 0.35)
check("const FIRST_TWO_TRAIL: (f64, f64) = (CHIP_IN - 0.35, MACHINE_X[1] + 0.35);" in src, "FIRST_TWO_TRAIL is defined as mirrored")
check("const LAST_TWO_TRAIL: (f64, f64) = (MACHINE_X[0] - 0.35, MACHINE_X[2] + 0.35);" in src, "LAST_TWO_TRAIL is defined as mirrored")
h, g, f = (lambda x: x + 2), (lambda x: 3 * x), (lambda x: x - 1)
check(re.search(r"const H_ADD: i32 = 2;", src) and re.search(r"const G_TIMES: i32 = 3;", src) and re.search(r"const F_SUB: i32 = 1;", src), "machines are x+2, 3x, x-1")
trail = [1, h(1), g(h(1)), f(g(h(1)))]
check(trail == [1, 3, 9, 8], f"trail {trail}")
values = [CHIP] + MX
arrows = [(CHIP + MX[0]) / 2, (MX[0] + MX[1]) / 2, (MX[1] + MX[2]) / 2]
inside = lambda box, x: box[0] < x < box[1]
got = [trail[k] for k, x in enumerate(values) if inside(first_two, x)]
check(got == [1, 3, 9] and g(h(got[0])) == got[-1], f"row 1 box {first_two} holds values {got}: g∘h takes {got[0]} to {got[-1]}")
got = [trail[k] for k, x in enumerate(values) if inside(last_two, x)]
check(got == [3, 9, 8] and f(g(got[0])) == got[-1], f"row 2 box {last_two} holds values {got}: f∘g takes {got[0]} to {got[-1]}")
check([inside(first_two, a) for a in arrows] == [True, True, False], "row 1 box holds the first two arrows, not the third")
check([inside(last_two, a) for a in arrows] == [False, True, True], "row 2 box holds the last two arrows, not the first")
check(r'FIRST_TWO_TEX: &str = r"f\circ(g\circ h)"' in src and r'LAST_TWO_TEX: &str = r"(f\circ g)\circ h"' in src, "row labels name the package each box holds")
check(g(h(3)) == 15 and f(g(9)) == 26, "the author's D1 arithmetic: baseline boxes would read 3 to 15 and 9 to 26 as packages")
span1 = re.search(r"const FIRST_TWO_SPAN: \(f64, f64\) = \((-?[\d.]+), (-?[\d.]+)\);", src).groups()
span2 = re.search(r"const LAST_TWO_SPAN: \(f64, f64\) = \((-?[\d.]+), (-?[\d.]+)\);", src).groups()
check(float(span1[0]) < MX[0] - 1 and MX[1] + 1 < float(span1[1]) < MX[2] - 1, f"gold package {span1} wraps machines h and g only")
check(MX[0] + 1 < float(span2[0]) < MX[1] - 1 and float(span2[1]) > MX[2] + 1, f"gold package {span2} wraps machines g and f only")


print("\n== 6 waits and phrases, every cue in the candidate")


def sentences(text):
    out, start = [], 0
    for i, c in enumerate(text):
        if c in ".?!" and (i + 1 == len(text) or text[i + 1] == " "):
            out.append(text[start:i + 1].strip())
            start = i + 1
    return out


table = re.findall(r'\("(13_\d+)", (\d+)\),', src)
for cue, n in table:
    check(len(sentences(script[cue])) == int(n), f"{cue}: {n} sentences")
order = {}
for cue, j in re.findall(r'until_sentence\(stage, "(13_\d+)", (\d+)\)', src):
    order.setdefault(cue, []).append(int(j))
for cue, js in order.items():
    check(max(js) < len(sentences(script[cue])), f"{cue} waits {js} are in range")
    check(js == sorted(js), f"{cue} waits run in order")
declared = int(re.search(r"const PHRASES: \[\(&str, usize, &str\); (\d+)\]", src).group(1))
block = re.search(r"const PHRASES: .*? = \[(.*?)\n\];", src, re.S).group(1)
rows = re.findall(r'\("(13_\d+)", (\d+), (\w+(?:\[\d\])?)\)', block)
check(len(rows) == declared, f"PHRASES declares {declared}, lists {len(rows)}")
laws = re.findall(r'"([^"]+)"', re.search(r"const LAW_PHRASES: \[&str; 4\] = \[(.*?)\];", src).group(1))
for cue, j, name in rows:
    k = re.fullmatch(r"LAW_PHRASES\[(\d)\]", name)
    phrase = laws[int(k.group(1))] if k else re.search(rf'const {name}: &str = "(.*?)";', src).group(1)
    check(phrase in sentences(script[cue])[int(j)], f"{cue} sentence {j} contains {phrase!r}")
used = re.findall(r'until_phrase\(stage, &kit, "(13_\d+)", (\d+), (\w+)\)', src)
named = [u for u in used if u[2] != "phrase"]
check(all(u in rows for u in named), f"each of {len(named)} named until_phrase calls is guarded by a PHRASES row")
check(("13_11", "0", "phrase") in used and ".zip(LAW_PHRASES)" in src, "the loop over LAW_PHRASES is guarded by its four rows")

print("\n== 7 caption text against caption's math arms")
arms = set()
m = re.search(r"let math = match text \{(.*?)_ => None,", src, re.S)
for line in m.group(1).splitlines():
    arms.update(re.findall(r'"([^"]+)"(?=\s*(?:\||=>))', line))
literal = set(re.findall(r'caption\(stage, &kit, "([^"]+)"', src))
literal |= set(re.findall(r'replace\(stage, &kit, \w+, "([^"]+)"', src))
literal |= set(re.findall(r'step\(stage, &kit, \w+, [^,]+, \w+, "([^"]+)"', src))
for head in re.findall(r"for \(\w+, text\) in \[(.*?)\]", src):
    literal |= set(re.findall(r'"([^"]+)"', head))
for head in re.findall(r"for \(index, text\) in \[(.*?)\]", src):
    literal |= set(re.findall(r'"([^"]+)"', head))
fancy = sorted(t for t in literal if any(ord(c) > 127 for c in t) and t not in arms)
check(not fancy, f"no non-ASCII caption reaches kit.text ({len(literal)} literals, {len(arms)} arms): {fancy}")
print("     arms not used by a literal:", sorted(arms - literal))
check("−" in re.search(r'"Undoing \+\{STEP\} needs (.)\{STEP\}', src).group(0), "13_11 caption uses U+2212")
check("0123456789−" in main and "4 − 16 = −12" in main, "U+2212 is already in the crate's text glyph list and another film's text")

print("\n== 8 stale or dead text the patch left")
check("// Closure: r times s is again one of the eight." not in src, "the 13_12 closure comment names the new pair")
reached_left = [float(x) for x in re.findall(r"\bstep\(stage, &kit, left, ([\d.]+)", src) + re.findall(r"perform\(stage, left, ([\d.]+)", src)]
reached_right = [float(x) for x in re.findall(r"perform\(stage, right, ([\d.]+)", src) + re.findall(r"\bstep\(stage, &kit, right, ([\d.]+)", src)]
print(f"     left reaches t={max(reached_left):g} of {len(LEFT)} moves; right reaches t={max(reached_right):g} of {len(RIGHT)} moves")
print("     `pose(&LEFT, 9.)` and `pose(&RIGHT, 5.)` in the test:", "pose(&LEFT, 9.), pose(&RIGHT, 5.)" in src)
print(f"\n{bad} failures")
