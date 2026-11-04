#!/usr/bin/env python3
"""Run from the repo root. Does the frozen chapter 11 patch do what its bundle says, and only that?

No build. Reads bytes, applies the patch in a temporary directory, parses with rustfmt.
"""
import hashlib
import json
import os
import re
import shutil
import subprocess
import tempfile

ROOT = "docs/video-program/third-wave-20261024/opus-replacements-20261104/"
B = ROOT + "algebra-picture-review-11_permutations/"
CANON = "explainer_corpus/abstract_algebra_expanded/src/permutations.rs"
SCRIPT = "explainer_corpus/abstract_algebra_expanded/script.tsv"
MAP = "explainer_corpus/abstract_algebra_expanded/docs/narration-direction-map-oliver24.json"
REL = "explainer_corpus/abstract_algebra_expanded/src/permutations.rs"


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


print("== bundle hashes")
bad = n = 0
for line in open(B + "HASHES.sha256", encoding="utf-8"):
    digest, path = line.split(None, 1)
    n += 1
    if sha(B + path.strip()) != digest:
        bad += 1
        print("DIFFERS", path.strip())
print(f"{n} files in HASHES.sha256, {bad} differ")
for name in ("README.md", "REVIEW.md", "PROGRESS.md", "HASHES.sha256", "patches/permutations.patch", "baseline/permutations.rs", "candidate/permutations.rs"):
    print(sha(B + name), name)
print(sha(CANON), "canonical", CANON)
print(sha(SCRIPT), "canonical", SCRIPT)
print(sha(MAP), "canonical", MAP)
base = open(B + "baseline/permutations.rs", "rb").read()
cand = open(B + "candidate/permutations.rs", "rb").read()
print("canonical equals baseline:", open(CANON, "rb").read() == base)
print("script bytes in bundle equal canonical:", any(sha(B + p) == sha(SCRIPT) for p in ["baseline/script.tsv"] if os.path.exists(B + p)) or "no script copy in bundle's baseline")

print("\n== patch at fuzz 0, forward and reverse, in a temporary tree")
patch_path = os.path.abspath(B + "patches/permutations.patch")
for direction, start, want in (("forward", base, cand), ("reverse", cand, base)):
    with tempfile.TemporaryDirectory() as tmp:
        target = os.path.join(tmp, REL)
        os.makedirs(os.path.dirname(target))
        open(target, "wb").write(start)
        args = ["patch", "-p1", "--fuzz=0", "--no-backup-if-mismatch", "-i", patch_path]
        if direction == "reverse":
            args.insert(1, "-R")
        run = subprocess.run(args, cwd=tmp, capture_output=True, text=True)
        print(direction, "exit", run.returncode, "| result equals expected:", open(target, "rb").read() == want, "| rejects:", os.path.exists(target + ".rej"))

print("\n== the same patch applied by an independent hunk walker")
text = open(patch_path, encoding="utf-8").read().split("\n")
if text[-1] == "":
    text.pop()
lines = base.decode("utf-8").split("\n")
out, at, added, removed, hunks = [], 0, 0, 0, 0
i = 0
while i < len(text):
    head = re.match(r"@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@", text[i])
    if not head:
        i += 1
        continue
    hunks += 1
    start = int(head.group(1)) - 1
    out.extend(lines[at:start])
    at = start
    i += 1
    while i < len(text) and not text[i].startswith("@@"):
        row = text[i]
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
print(f"hunks {hunks}, added {added}, removed {removed}; result equals candidate:", "\n".join(out).encode("utf-8") == cand)

print("\n== where the change falls")
base_lines = base.decode("utf-8").split("\n")
tests_at = next(k for k, row in enumerate(base_lines, 1) if row.strip() == "#[cfg(test)]" and base_lines[k].startswith("mod "))
print("first tests module starts at baseline line", tests_at)
starts = [int(m.group(1)) for m in re.finditer(r"^@@ -(\d+)", "\n".join(text), re.M)]
print("hunks starting inside tests:", [s for s in starts if s >= tests_at])
print("#[test] count baseline", base.decode().count("#[test]"), "candidate", cand.decode().count("#[test]"))

print("\n== cue order and sentence indices")
SAY = re.compile(r'k\.say\(stage, "(11_\d+)"\)')
UNTIL = re.compile(r'until_sentence\(stage, "(11_\d+)", (\d+)\)')
print("k.say order identical:", SAY.findall(base.decode()) == SAY.findall(cand.decode()))
cues = dict(row.split("\t", 1) for row in open(SCRIPT, encoding="utf-8").read().splitlines() if row.startswith("11_"))


def sentences(line):
    return [s for s in re.split(r"(?<=[.?!])(?= |$)", line) if s.strip()]


bad_index = [(c, j) for c, j in UNTIL.findall(cand.decode()) if int(j) >= len(sentences(cues[c].strip()))]
print("until_sentence indices past their cue's sentences:", bad_index)

units = json.load(open("explainer_corpus/abstract_algebra_expanded/narration_units.json", encoding="utf-8"))
rows = [u for u in units["units"] if u["line_ids"][0].startswith("11_")]
print(f"narration_units.json: sha {sha('explainer_corpus/abstract_algebra_expanded/narration_units.json')}; {len(rows)} chapter 11 units; one cue each: {all(len(u['line_ids']) == 1 for u in rows)}; text equals the script: {all(u['text'].strip() == cues[u['line_ids'][0]].strip() for u in rows)}; binds script {units['direction_contract']['script_sha256'][:12]} and map {units['direction_contract']['map']['sha256'][:12]}")

print("\n== display-string census of the patch: literals only on + or only on - lines")
LIT = re.compile(r'r?"(?:[^"\\]|\\.)*"')
plus, minus = [], []
for row in text:
    if row.startswith("+") and not row.lstrip("+ ").startswith("//") and not row.lstrip("+ ").startswith("///"):
        plus += LIT.findall(row)
    if row.startswith("-") and not row.lstrip("- ").startswith("//"):
        minus += LIT.findall(row)
for lit in sorted(set(plus) - set(minus)):
    print("  new    ", lit)
for lit in sorted(set(minus) - set(plus)):
    print("  removed", lit)

print("\n== old strings anywhere else in the crate")
for needle in ("every line back where it began", "2 fixed, 1 swap, 6 cycles of length 8"):
    hits = []
    for folder, _, files in os.walk("explainer_corpus/abstract_algebra_expanded"):
        if "/target" in folder:
            continue
        for name in files:
            if name.endswith((".rs", ".py", ".tsv", ".toml")):
                path = os.path.join(folder, name)
                if needle in open(path, encoding="utf-8", errors="replace").read():
                    hits.append(path)
    print(" ", needle, "->", hits)

print("\n== rustfmt parse and format, candidate on a temporary copy")
with tempfile.TemporaryDirectory() as tmp:
    copy = os.path.join(tmp, "permutations.rs")
    shutil.copyfile(B + "candidate/permutations.rs", copy)
    run = subprocess.run(["rustfmt", "--check", "--edition", "2024", "--config-path", os.path.abspath("rustfmt.toml"), copy], capture_output=True, text=True)
    print("rustfmt --check exit", run.returncode, "| stderr lines", len(run.stderr.splitlines()), "| diff lines", len(run.stdout.splitlines()))
