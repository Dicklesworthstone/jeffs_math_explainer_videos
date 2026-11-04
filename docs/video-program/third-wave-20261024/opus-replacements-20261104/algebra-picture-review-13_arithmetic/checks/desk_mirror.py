"""Desk checks for the staged chapter 13 candidates: nothing here compiles or renders Rust.

Run from the repo root with `python3 -B desk_mirror.py A_RS AB_RS AB_ORACLE`. PIL is needed, hence no -I.

Checks, each printed with its evidence:
  1. Every sentence index and phrase the candidates wait on exists in script.tsv, split as the kit splits.
  2. The tile facts their new tests assert, on a Python mirror of tile_model.rs and Elem::mul.
  3. The AB oracle's closure value against the mirror of the Rust fact.
  4. Every character of new on-screen strings already appears in the crate's glyph lists.
  5. Clearances of the new boxes, measured on baseline frames 0480 and 0510 at 90 px per scene unit.
  6. Seconds each 13_12 beat asks for, against root's recorded ASR sentence starts on Oliver take01.
"""
import json
import math
import pathlib
import re
import subprocess
import sys

from PIL import Image

A_RS, AB_RS, AB_ORACLE = (pathlib.Path(p) for p in sys.argv[1:4])
CRATE = pathlib.Path("explainer_corpus/abstract_algebra_expanded")
FRAMES = pathlib.Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/algebra-combined-repair-frames01/13_arithmetic")
LAB = pathlib.Path("narration_lab/third-wave-20261024/root/elevenlabs-expanded-algebra-20261103")
script = dict(l.split("\t", 1) for l in (CRATE / "script.tsv").read_text().splitlines() if "\t" in l)
problems = 0


def report(ok, text):
    global problems
    problems += 0 if ok else 1
    print(("ok   " if ok else "FAIL ") + text)


def sentences(text):
    """The split of axioms.rs::sentence_ranges."""
    out, start = [], 0
    for i, c in enumerate(text):
        if c in ".?!" and (i + 1 == len(text) or text[i + 1] == " "):
            out.append(text[start : i + 1].strip())
            start = i + 1
    if text[start:].strip():
        out.append(text[start:].strip())
    return out


print("== 1 sentence indices and phrases")
for name, path in (("A", A_RS), ("AB", AB_RS)):
    src = path.read_text()
    body = src[: src.index("#[cfg(test)]\nmod tests")]
    consts = dict(re.findall(r'const (\w+_PHRASE): &str = "([^"]+)"', src))
    n = 0
    for cue, k in re.findall(r'until_sentence\(stage, "(13_\d+)", (\d+)\)', body):
        n += 1
        if int(k) >= len(sentences(script[cue])):
            report(False, f"{name}: {cue} has no sentence {k}")
    for cue, k, c in re.findall(r'until_phrase\(stage, &kit, "(13_\d+)", (\d+), (\w+)\)', body):
        n += 1
        if c not in consts:
            continue
        said = sentences(script[cue])[int(k)]
        if consts[c] not in said:
            report(False, f"{name}: {cue} sentence {k} lacks {consts[c]!r}")
    report(True, f"{name}: {n} waits checked")
    table = re.search(r"const PHRASES: \[\(&str, usize, &str\); (\d+)\] = \[(.*?)\n\];", src, re.S)
    rows = re.findall(r'\("13_\d+", \d+, [\w\[\]]+\)', table[2])
    report(int(table[1]) == len(rows), f"{name}: PHRASES says {table[1]}, lists {len(rows)}")
    # A wait on a named phrase must be guarded by the table, as the existing test requires.
    used = set(re.findall(r'until_phrase\(stage, &kit, "13_\d+", \d+, (\w+)\)', body)) - {"phrase"}
    missing = [u for u in used if u not in table[2]]
    report(not missing, f"{name}: phrases waited on but not guarded: {missing}")
counts = {c: len(sentences(script[c])) for c in sorted((c for c in script if c.startswith("13_")), key=lambda c: int(c[3:]))}
print("     sentence counts:", counts)
pinned = dict(re.findall(r'\("(13_\d+)", (\d+)\),', AB_RS.read_text()[AB_RS.read_text().index("for (cue, sentences) in") :]))
report(all(int(pinned.get(c, -1)) == n for c, n in counts.items()), f"the test's table agrees: {pinned}")


print("\n== 2 mirror of the tile model and Elem::mul")


def turn(t):
    s, c = math.sin(math.pi / 2 * t), math.cos(math.pi / 2 * t)
    return [[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]]


def flip(t):
    s, c = math.sin(math.pi * t), math.cos(math.pi * t)
    return [[1.0, 0.0, 0.0], [0.0, c, -s], [0.0, s, c]]


def compose(a, b):
    return [[sum(a[i][k] * b[k][j] for k in range(3)) for j in range(3)] for i in range(3)]


def pose(moves, t):
    state = turn(0)
    for i, m in enumerate(moves):
        f = min(max(t - i, 0.0), 1.0)
        state = compose(turn(m * f) if m != "F" else flip(f), state)
    return state


def same(a, b):
    return all(abs(a[i][j] - b[i][j]) < 1e-9 for i in range(3) for j in range(3))


LONG = [1.0, "F", "F", 1.0, -1.0]
SEQ = ["F", 1.0, "F"]
report(same(pose(LONG, 5.0), turn(1)), "LONG_ROUTE ends at one quarter-turn")
report(same(pose(SEQ, 3.0), turn(-1)), "SEQUENCE (flip, turn, flip) ends at the opposite quarter-turn")
report(same(pose(SEQ, 3.0), turn(3)), "and at r cubed")
report(not same(pose(LONG, 2.5), turn(1)), "the long route really leaves that pose on the way")


def mul(a, b):
    carried = (4 - b[0]) % 4 if a[1] else b[0]
    return ((a[0] + carried) % 4, a[1] != b[1])


def inv(a):
    return a if a[1] else ((4 - a[0]) % 4, False)


TEX = {(0, False): "e", (1, False): "r", (2, False): "r^2", (3, False): "r^3", (0, True): "s", (1, True): "rs", (2, True): "r^2s", (3, True): "r^3s"}
E_, R_, R2_, S_ = (0, False), (1, False), (2, False), (0, True)
report(mul(S_, R_) == mul(inv(R_), S_), "s r = r⁻¹ s")
report(mul(S_, S_) == E_, "s s = e")
report(mul(inv(R_), R2_) == R_, "r⁻¹ r² = r")
word = [S_, R_, S_, R2_, S_]
acc = E_
for g in word:
    acc = mul(acc, g)
report(acc == mul(R_, S_), "s r s r² s = r s, the end of the 13_7 chain")
chain = mul(mul(mul(mul(inv(R_), S_), S_), R2_), S_)
report(chain == acc, "after the first rewrite the word r⁻¹ s s r² s is still that element")


print("\n== 3 oracle on AB")
out = subprocess.run([sys.executable, "-I", "-B", str(AB_ORACLE)], capture_output=True, text=True, cwd=CRATE)
report(out.returncode == 0, f"oracle exit {out.returncode}; {len(out.stdout.splitlines())} lines {out.stderr[-200:]}")
line = [l for l in out.stdout.splitlines() if l.startswith("arithmetic.closure.product")]
report(line and line[0].split(None, 1)[-1].split("\t")[-1].strip() == TEX[mul(S_, R_)], f"closure fact {line} against mirror {TEX[mul(S_, R_)]}")
base = subprocess.run([sys.executable, "-I", "-B", "oracle.py"], capture_output=True, text=True, cwd=CRATE).stdout.splitlines()
changed = [(a, b) for a, b in zip(base, out.stdout.splitlines()) if a != b]
report(len(base) == len(out.stdout.splitlines()) and len(changed) == 1, f"exactly one oracle line differs from canonical: {changed}")


print("\n== 4 glyph coverage of new on-screen strings")
main = (CRATE / "src/main.rs").read_text()
text_list = main[main.index("glyph_text: &[") : main.index("glyph_tex: &[")]
tex_list = main[main.index("glyph_tex: &[") : main.index("glyph_mono: &[")]
text_chars = set("".join(re.findall(r'"([^"]*)"', text_list)))
tex_chars = set("".join(re.findall(r'"([^"]*)"', tex_list)))
for s in ("start", "Undoing +3 needs −3, which is missing"):
    gone = sorted(set(s) - text_chars - {" "})
    report(not gone, f"text {s!r}: characters not in glyph_text {gone}")
for s in ("sr=r^{-1}s", "srs=r^{-1}=r^3", "s\\,r=r^3s"):
    gone = sorted(set(s) - tex_chars - {" "})
    report(not gone, f"tex {s!r}: characters not in glyph_tex {gone}")


print("\n== 5 clearances on the baseline frames, 720p, 90 px per unit")


def px(x):
    return 640 + 90 * x


def py(y):
    return 360 - 90 * y


def ink(frame, box, bright=70):
    """Bounding box of bright pixels inside (x0, y0, x1, y1)."""
    with Image.open(FRAMES / f"frame-{frame:04}.png") as f:
        g = f.convert("L").crop(tuple(int(v) for v in box))
    bb = g.point(lambda v: 255 if v > bright else 0).getbbox()
    return None if bb is None else (bb[0] + int(box[0]), bb[1] + int(box[1]), bb[2] + int(box[0]), bb[3] + int(box[1]))


# Frame 0480 shows both trails complete. Ink runs along each row, ignoring the old gold boxes by brightness of ink only.
for label, y, span, inside, outside in (
    ("row 1, f∘(g∘h)", -0.55, (-5.75, 0.35), [(-5.6, -5.2), (-0.2, 0.2)], [(1.25, 1.75)]),
    ("row 2, (f∘g)∘h", -1.55, (-3.35, 3.35), [(-3.2, -2.8), (2.8, 3.2)], [(-4.45, -3.95), (4.7, 6.1)]),
):
    left, right = px(span[0]), px(span[1])
    for a, b in inside:
        bb = ink(480, (px(a), py(y) - 25, px(b), py(y) + 25), 150)
        report(bb and bb[0] > left + 6 and bb[2] < right - 6, f"{label}: value at x {a}..{b} ink {bb} inside box {left:.0f}..{right:.0f}")
    for a, b in outside:
        bb = ink(480, (px(a), py(y) - 25, px(b), py(y) + 25), 100)
        report(bb and (bb[2] < left - 6 or bb[0] > right + 6), f"{label}: x {a}..{b} ink {bb} clear of box {left:.0f}..{right:.0f}")
row_gap = py(-0.55) + 0.33 * 90, py(-1.55) - 0.33 * 90
report(row_gap[1] - row_gap[0] > 20, f"the two trail boxes keep {row_gap[1] - row_gap[0]:.0f} px between them")

# Frame 0510 has the same rack the new row frames surround: origin (-2.25, 0.1), step (1.5, 1.45), k 0.8.
for row in (0, 1):
    y = 0.1 - 1.45 * row - 0.2
    top, bottom = py(y) - 0.675 * 90, py(y) + 0.675 * 90
    left, right = px(-2.95), px(2.95)
    win_top = py(0.1 - 1.45 * row) - 0.75 * 90 + 8
    win_bot = py(0.1 - 1.45 * row) + 1.0 * 90
    bb = ink(510, (px(-3.4), win_top, px(3.4), win_bot), 60)
    report(bb and bb[1] > top + 3 and bb[3] < bottom - 3 and bb[0] > left + 3 and bb[2] < right - 3,
           f"rack row {row}: ink {bb} inside frame x {left:.0f}..{right:.0f}, y {top:.0f}..{bottom:.0f}")
report(py(0.1 - 0.2) + 0.675 * 90 < py(0.1 - 1.45 - 0.2) - 0.675 * 90, "the two row frames do not touch")

# Corner reach under the labels: frames 0358 and 0371 are tiles at about 45 and 30 degrees.
for frame in (358, 371):
    disc = ink(frame, (440, 440, 840, 524), 90)
    report(disc is not None, f"frame {frame}: lowest tile ink ends at y {disc[3] if disc else None}; text at y -2.3 would start near {py(-2.3) - 14:.0f}")
    report(disc and disc[3] < py(-2.3) - 14 - 15, f"frame {frame}: at least 15 px clear of BELOW_TURNING")


print("\n== 6 the 13_12 beats on Oliver take01, ASR sentence starts recorded by root")
audit = json.loads((LAB / "arithmetic-sentence-audit02.json").read_text())
starts = [g["start"] for r in audit["records"] if r["cue"] == "13_12" and r.get("model") == "large" for g in r["punctuation_groups"]]
windows = [starts[i + 1] - starts[i] for i in range(1, 5)]
LIGHT = 0.25 + 0.25 + 0.3
before = [LIGHT + 0.35 + 0.35 + 0.35, 0.35 + 0.2 + LIGHT + 2.1 + 0.35 + 0.3, 0.35 + 0.2 + LIGHT + 0.35 + 0.35, 0.35 + 0.2 + LIGHT + 1.95 + 0.35 + 0.35]
after = [0.4 + 0.35 + 0.35, 0.4 + 2.1 + 0.35, 0.4 + 0.35, 0.4 + 1.95 + 0.35]
print("     beat            window  before: asks, full hold   B: asks, full hold")
for name, w, b, a in zip(("closure", "associativity", "identity", "inverse"), windows, before, after):
    print(f"     {name:14} {w:6.2f}  {b:6.2f} {w - b:6.2f}          {a:6.2f} {w - a:6.2f}")

print(f"\nproblems: {problems}")
sys.exit(1 if problems else 0)
