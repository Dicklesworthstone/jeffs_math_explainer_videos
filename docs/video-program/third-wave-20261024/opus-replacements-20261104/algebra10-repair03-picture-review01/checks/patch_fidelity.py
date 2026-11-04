#!/usr/bin/env python3
"""Run from the repo root: python3 -I -B <this file>

Does the staged patch do what REVIEW.md says, and only that?

No build and no render: nothing here compiled, ran or drew the patched scene.
It reads bytes, applies the patch in a temporary directory, parses with rustfmt,
and measures the existing frames to place the new labels.
"""
import hashlib
import json
import os
import re
import subprocess
import tempfile
from pathlib import Path

from PIL import Image

HERE = "docs/video-program/third-wave-20261024/opus-replacements-20261104/algebra10-repair03-picture-review01/"
PATCH = HERE + "patches/cyclic-picture-followup.patch"
BUNDLE = "docs/video-program/third-wave-20261024/opus-replacements-20261104/algebra-chapter10-picture-repair03/"
REL = "explainer_corpus/abstract_algebra_expanded/src/cyclic.rs"
CRATE = "explainer_corpus/abstract_algebra_expanded/"
FRAMES = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/algebra-chapter10-repair03-frames01/10_cyclic")


def sha(data):
    return hashlib.sha256(data).hexdigest()


def apply(patch_bytes, start, reverse=False):
    with tempfile.TemporaryDirectory() as tmp:
        target = os.path.join(tmp, REL)
        os.makedirs(os.path.dirname(target))
        open(target, "wb").write(start)
        patch_file = os.path.join(tmp, "p.patch")
        open(patch_file, "wb").write(patch_bytes)
        args = ["patch", "-p1", "--fuzz=0", "--no-backup-if-mismatch", "-i", patch_file]
        if reverse:
            args.insert(1, "-R")
        run = subprocess.run(args, cwd=tmp, capture_output=True, text=True)
        return run.returncode, open(target, "rb").read(), os.path.exists(target + ".rej")


def rustfmt_check(data):
    with tempfile.TemporaryDirectory() as tmp:
        copy = os.path.join(tmp, "cyclic.rs")
        open(copy, "wb").write(data)
        run = subprocess.run(["rustfmt", "--check", "--edition", "2024", "--config-path", os.path.abspath("rustfmt.toml"), copy], capture_output=True, text=True)
        return run.returncode, len(run.stderr.splitlines()), len(run.stdout.splitlines())


base = open(REL, "rb").read()
patch_bytes = open(PATCH, "rb").read()

print("== inputs")
print(sha(base), "canonical", REL)
print(sha(open(BUNDLE + "cyclic.rs", "rb").read()), "repair03 bundle cyclic.rs")
print(sha(open(CRATE + "script.tsv", "rb").read()), "script.tsv")
print(sha(open(CRATE + "docs/narration-direction-map-oliver24.json", "rb").read()), "direction map")
print(sha(patch_bytes), "patch")

print("\n== the patch at fuzz 0, forward and reverse, in a temporary tree")
code, cand, rej = apply(patch_bytes, base)
print("forward exit", code, "| rejects:", rej)
print("candidate sha256", sha(cand))
code, back, rej = apply(patch_bytes, cand, reverse=True)
print("reverse exit", code, "| rejects:", rej, "| returns the base bytes:", back == base)

print("\n== the same patch applied by an independent hunk walker")
rows = patch_bytes.decode("utf-8").split("\n")
if rows[-1] == "":
    rows.pop()
lines = base.decode("utf-8").split("\n")
out, at, added, removed, count = [], 0, 0, 0, 0
i = 0
while i < len(rows):
    head = re.match(r"@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@", rows[i])
    if not head:
        i += 1
        continue
    count += 1
    start = int(head.group(1)) - 1
    out.extend(lines[at:start])
    at = start
    i += 1
    while i < len(rows) and not rows[i].startswith("@@"):
        row = rows[i]
        if row.startswith(" ") or row == "":
            assert lines[at] == row[1:], ("context", at + 1, row)
            out.append(lines[at])
            at += 1
        elif row.startswith("-"):
            assert lines[at] == row[1:], ("removed", at + 1, row)
            at += 1
            removed += 1
        elif row.startswith("+"):
            out.append(row[1:])
            added += 1
        i += 1
out.extend(lines[at:])
print(f"hunks {count}, added {added}, removed {removed}; result equals the patch tool's:", "\n".join(out).encode("utf-8") == cand)

print("\n== each finding's hunks alone, at fuzz 0 on the base, then rustfmt")
text = patch_bytes.decode("utf-8")
head_end = text.index("@@")
header = text[:head_end]
hunks = re.split(r"(?m)^(?=@@ )", text[head_end:])
hunks = [h for h in hunks if h]
GROUPS = {"A 10_2": [3], "B 10_10": [0, 1, 7, 8, 9, 10], "C 10_4": [4, 5], "D 10_1": [2, 11], "E 10_6": [6]}
assert sorted(sum(GROUPS.values(), [])) == list(range(len(hunks))), "every hunk belongs to one finding"
for name, members in GROUPS.items():
    part = (header + "".join(hunks[m] for m in members)).encode("utf-8")
    code, data, rej = apply(part, base)
    print(f"{name}: hunks {members} exit {code} rejects {rej} | rustfmt (exit, stderr lines, diff lines) {rustfmt_check(data)} | sha {sha(data)[:12]}")

print("\n== where the change falls")
base_text, cand_text = base.decode("utf-8"), cand.decode("utf-8")
base_rows, cand_rows = base_text.split("\n"), cand_text.split("\n")
tests_at = next(k for k, row in enumerate(base_rows, 1) if row.strip() == "#[cfg(test)]" and base_rows[k].startswith("mod "))
print("tests module starts at base line", tests_at)
starts = [int(m.group(1)) for m in re.finditer(r"(?m)^@@ -(\d+)", text)]
print("hunks starting inside tests:", [s for s in starts if s >= tests_at], "| #[test] base", base_text.count("#[test]"), "candidate", cand_text.count("#[test]"))

print("\n== cue order, sentence indices, phrases")
SAY = re.compile(r'k\.say\(stage, "(10_\d+)"\)')
print("k.say order identical:", SAY.findall(base_text) == SAY.findall(cand_text), "| count", len(SAY.findall(cand_text)))
cues = dict(row.split("\t", 1) for row in open(CRATE + "script.tsv", encoding="utf-8").read().splitlines() if row.startswith("10_"))


def spans(line):
    out, start = [], 0
    for at, c in enumerate(line):
        if c in ".?!" and (at + 1 == len(line) or line[at + 1] == " "):
            a = start
            while line[a] == " ":
                a += 1
            out.append((a, at + 1))
            start = at + 1
    return out


WAIT = re.compile(r'(k\.until_sentence\(stage, |until_phrase\(stage, &k, )"(10_\d+)", (\d+)(?:, (\w+))?\)')
waits = [(m.group(2), int(m.group(3)), m.group(4)) for m in WAIT.finditer(cand_text)]
print("waits in candidate:", len(waits), "| base:", len(WAIT.findall(base_text)))
print("indices past their cue's sentences:", [w for w in waits if w[1] >= len(spans(cues[w[0]]))])
for cue in cues:
    seq = [w[1] for w in waits if w[0] == cue]
    if seq != sorted(seq):
        print("sentence waits out of order in", cue, seq)
print("sentence waits in order within every cue:", all([w[1] for w in waits if w[0] == c] == sorted(w[1] for w in waits if w[0] == c) for c in cues))
constants = dict(re.findall(r'const (\w+_PHRASE): &str = "([^"]+)";', cand_text))
missing = []
for cue, j, name in waits:
    if name:
        a, b = spans(cues[cue])[j]
        if constants[name] not in cues[cue][a:b]:
            missing.append((cue, j, name))
print("phrases not in their sentence:", missing, "| ROTATIONS_PHRASE:", constants.get("ROTATIONS_PHRASE"))
table = re.search(r"const PHRASES: \[\(&str, usize, &str\); (\d+)\] = \[(.*?)\];", cand_text, re.S)
print("PHRASES declares", table.group(1), "and lists", len(re.findall(r"\(\"10_", table.group(2))))
hold = next(k for k, row in enumerate(cand_rows, 1) if row.strip() == "k.hold(stage)?;")
print("first k.hold in candidate at line", hold, "| next lines:", [r.strip() for r in cand_rows[hold : hold + 2]])
units = json.load(open(CRATE + "narration_units.json", encoding="utf-8"))
print("narration_units binds script", units["direction_contract"]["script_sha256"][:12], "and map", units["direction_contract"]["map"]["sha256"][:12])
direction = json.load(open(CRATE + "docs/narration-direction-map-oliver24.json", encoding="utf-8"))
row = next(h for h in direction["visual_holds"] if h["cue"] == "10_1")
print("map hold row for 10_1: line", row["line"], "sha", row["sha256"][:12], "| base line text:", base_rows[row["line"] - 1].strip(), "| needs rebinding to line", hold, "and the candidate sha")

print("\n== every new mobject leaves the screen, and the new names were free")
for name in ("group_size", "each_coprime", "unit_orders", "rotations", "squared", "cell"):
    print(name, "in base:", len(re.findall(rf"\b{name}\b", base_text)), "| in candidate:", len(re.findall(rf"\b{name}\b", cand_text)))
print("hide list carries group_size:", "d4g::hide(stage, &[question, trail, group_size], 0.4)?;" in cand_text)
print("table carries the four new rows:", "table.extend([each_coprime, squares, unit_orders, no, rotations]);" in cand_text, "| hidden next:", "d4g::hide(stage, &table, 0.4)?;" in cand_text)

print("\n== new and removed display strings")
LIT = re.compile(r'r?"(?:[^"\\]|\\.)*"')
plus, minus = [], []
for row in rows:
    body = row[1:].strip()
    if row.startswith("+") and not row.startswith("+++") and not body.startswith("//"):
        plus += LIT.findall(body)
    if row.startswith("-") and not row.startswith("---") and not body.startswith("//"):
        minus += LIT.findall(body)
for lit in sorted(set(plus) - set(minus)):
    print("  new    ", lit)
for lit in sorted(set(minus) - set(plus)):
    print("  removed", lit)

print("\n== an independent mirror of the new test, with mutated inputs that must fail")


def holds(group, n):
    for u in group:
        if any((u * w) % n not in group for w in group):
            return False
        if (u * u) % n != 1:
            return False
        if any(pow(u, m, n) not in (1, u) for m in range(4)):
            return False
    return True


print("{1,5,7,11} mod 12:", holds([1, 5, 7, 11], 12), "| {1,5,7,10} mod 12 (must fail):", holds([1, 5, 7, 10], 12), "| {1,2,3,4} mod 5, which is cyclic (must fail):", holds([1, 2, 3, 4], 5))
print("table:", [[(a * b) % 12 for b in (1, 5, 7, 11)] for a in (1, 5, 7, 11)])

print("\n== placing the new labels: the width law against ink measured on existing frames")
EM = dict(zip("abcdefghijklmnopqrstuvwxyz", [0.5, 0.556, 0.444, 0.556, 0.444, 0.306, 0.5, 0.556, 0.278, 0.306, 0.528, 0.278, 0.833, 0.556, 0.5, 0.556, 0.528, 0.392, 0.394, 0.389, 0.556, 0.528, 0.722, 0.528, 0.528, 0.444]))
EM.update({str(d): 0.5 for d in range(10)})
EM.update({" ": 0.333, ":": 0.278, ",": 0.278, "?": 0.472, "A": 0.75, "E": 0.681, "N": 0.75, "W": 1.028})


def law(text, size):
    return sum(EM[c] for c in text) * size * 0.01


def ink(frame, box):
    """Ink box in a frame, in pixels, or None. 90 px per scene unit; scene origin at (640, 360)."""
    with Image.open(FRAMES / f"frame-{frame:04}.png") as im:
        grey = im.convert("L").crop(box).point(lambda p: 255 if p > 70 else 0)
    found = grey.getbbox()
    return None if found is None else (found[0] + box[0], found[1] + box[1], found[2] + box[0], found[3] + box[1])


def px(x, y):
    return 640 + 90 * x, 360 - 90 * y


CALIBRATE = [
    ("No element generates all four", 31, 560, (640, 435, 1280, 490)),
    ("Add 8: how many positions?", 30, 30, (640, 240, 1280, 300)),
    ("4 orbits, 3 positions each", 26, 192, (640, 275, 1280, 320)),
]
for words, size, frame, box in CALIBRATE:
    b = ink(frame, box)
    print(f"'{words}' size {size}: law {law(words, size):.2f} units | measured frame {frame} {(b[2] - b[0]) / 90:.2f} units, ink box {b}")

print("\nNew labels: width by the law, edges in scene units, and whether the frame's target box is empty today")
NEW = [
    ("Whole clock group: 12 elements", 28, (2.3, -2.15), 100),
    ("Each relatively prime to 12", 28, (2.5, 1.15), 560),
    ("Not the four rotations of a square", 27, (2.5, -1.8), 560),
]
for words, size, (x, y), frame in NEW:
    w = law(words, size)
    left, right = x - w / 2, x + w / 2
    cx, cy = px(x, y)
    box = (int(cx - w * 45 - 10), int(cy - 22), int(cx + w * 45 + 10), int(cy + 22))
    print(f"'{words}' size {size}: width {w:.2f}, x {left:.2f} to {right:.2f}; frame {frame} box {box} ink today: {ink(frame, box)}")

tex = ink(100, (640, 455, 1100, 510))
print("\nord(8)=3 at size 32 on frame 100, ink box:", tex, "| width", round((tex[2] - tex[0]) / 90, 2), "height px", tex[3] - tex[1])
scale = (tex[2] - tex[0]) / 90 / (sum(EM[c] for c in "ord") + 0.389 + 0.5 + 0.389 + 0.778 + 0.556 + 0.5)
print("ems of ord(8)=3 by the TeX widths; units per em measured:", round(scale, 3))
ems = 2 * (sum(EM[c] for c in "ord") + 0.389 + 0.5 + 0.389 + 0.778 + 0.556) + (sum(EM[c] for c in "ord") + 0.389 + 1.0 + 0.389 + 0.778 + 0.556) + 0.5
print(f"ord(5)=ord(7)=ord(11)=2 at size 32: about {ems * scale:.2f} units wide at x 2.5, so x {2.5 - ems * scale / 2:.2f} to {2.5 + ems * scale / 2:.2f}")
above = ink(560, (640, 320, 1280, 375))
below = ink(560, (640, 435, 1280, 490))
cx, cy = px(2.5, -0.5)
half = (tex[3] - tex[1]) / 2
print(f"slot at y -0.5 on frame 560: squares' ink ends at {above[3]} px, 'No element' starts at {below[1]} px; the new row needs {cy - half:.0f} to {cy + half:.0f} px | clearances {cy - half - above[3]:.0f} and {below[1] - (cy + half):.0f} px")
table_box = ink(560, (0, 280, 640, 620))
print("10_10 table ink, below its title, ends at x px", table_box[2], "= scene", round((table_box[2] - 640) / 90, 2), "| squares' ink box", above)
print("candidate rustfmt (exit, stderr lines, diff lines):", rustfmt_check(cand))
