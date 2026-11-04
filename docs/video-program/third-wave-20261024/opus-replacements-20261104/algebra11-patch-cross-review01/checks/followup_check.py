#!/usr/bin/env python3
"""Run from the repo root. The follow-up patch: fidelity at fuzz 0, the chain from canonical, parse, and simple source facts.

No build. Python mirrors one Rust test (every_anchor_names_words_in_its_own_sentence) for the new anchor only.
"""
import hashlib
import os
import re
import subprocess
import tempfile

ROOT = "docs/video-program/third-wave-20261024/opus-replacements-20261104/"
B = ROOT + "algebra-picture-review-11_permutations/"
HERE = ROOT + "algebra11-patch-cross-review01/"
REL = "explainer_corpus/abstract_algebra_expanded/src/permutations.rs"
FIRST = os.path.abspath(B + "patches/permutations.patch")
SECOND = os.path.abspath(HERE + "patches/permutations-followup.patch")


def sha(data):
    return hashlib.sha256(data).hexdigest()


def apply(tmp, patch, reverse=False):
    args = ["patch", "-p1", "--fuzz=0", "--no-backup-if-mismatch", "-i", patch]
    if reverse:
        args.insert(1, "-R")
    run = subprocess.run(args, cwd=tmp, capture_output=True, text=True)
    return run.returncode, os.path.exists(os.path.join(tmp, REL + ".rej"))


canon = open(REL, "rb").read()
cand = open(B + "candidate/permutations.rs", "rb").read()
print("follow-up patch", sha(open(SECOND, "rb").read()))
print("base: frozen candidate", sha(cand))
with tempfile.TemporaryDirectory() as tmp:
    target = os.path.join(tmp, REL)
    os.makedirs(os.path.dirname(target))
    open(target, "wb").write(canon)
    print("canonical + bundle patch: exit, reject", apply(tmp, FIRST), "| equals frozen candidate:", open(target, "rb").read() == cand)
    print("+ follow-up: exit, reject", apply(tmp, SECOND))
    result = open(target, "rb").read()
    print("result", sha(result))
    keep = result
    print("reverse follow-up: exit, reject", apply(tmp, SECOND, True), "| equals frozen candidate:", open(target, "rb").read() == cand)
    print("reverse bundle patch: exit, reject", apply(tmp, FIRST, True), "| equals canonical:", open(target, "rb").read() == canon)
    open(target, "wb").write(keep)
    run = subprocess.run(["rustfmt", "--check", "--edition", "2024", "--config-path", os.path.abspath("rustfmt.toml"), target], capture_output=True, text=True)
    print("rustfmt --check exit", run.returncode, "| stderr lines", len(run.stderr.splitlines()), "| diff lines", len(run.stdout.splitlines()))
    print(subprocess.run(["python3", "-I", "-B", HERE + "checks/hold_map.py", target], capture_output=True, text=True).stdout.split("== follow-up", 1)[1].join(["== follow-up", ""]))

text = keep.decode("utf-8")
diff = open(SECOND, encoding="utf-8").read()
print("hunks", len(re.findall(r"^@@", diff, re.M)), "| added", len(re.findall(r"^\+(?!\+\+ )", diff, re.M)), "| removed", len(re.findall(r"^-(?!-- )", diff, re.M)))
body = text.split("const ANCHORS: [Anchor; ", 1)[1]
size = int(body.split("]", 1)[0])
entries = re.findall(r"^    [A-Z_]+,$", body.split("];", 1)[0], re.M)
print("ANCHORS length", size, "| entries", len(entries), "| THEY_COMPOSE listed:", "    THEY_COMPOSE," in entries)
script = dict(row.split("\t", 1) for row in open("explainer_corpus/abstract_algebra_expanded/script.tsv", encoding="utf-8").read().splitlines() if row.startswith("11_"))
first = re.split(r"(?<=[.?!])(?= |$)", script["11_20"].strip())[0]
print("anchor phrase in 11_20 sentence 0:", "since they compose" in first)
print("walk_cycle calls:", re.findall(r"walk_cycle\(stage, [^\n]*", text))
print("walk_cycle parameters:", len(re.findall(r"^    \w+: ", text.split("fn walk_cycle(", 1)[1].split(") -> fmn::Result", 1)[0], re.M)), "(Clippy too_many_arguments starts above 7)")
start = text.index("fn twelve_of_twenty_four")
scene = text[start : text.index("// ---- 11_10, 11_22 and 11_23", start)]
print("even_key uses in 11_21:", scene.count("even_key"), "| odd_key:", scene.count("odd_key"), "| in the returned list:", "even_key" in scene.split("on_screen", 1)[1])
print("order in sentence 5: keys leave before the first recolour:", scene.index("fade(stage, &[even_key, odd_key], false)") < scene.index("recolor(stage, &nothing"))
print("old words left in the file:", [w for w in ("Gold: even", "Blue: odd", "to slot {to}", "starting lines", "wiring eight lines") if w in text])
