"""Source checks on the repaired candidates: real scene tests, formatting, cue words, lint, word diffs, oracle merge.

Usage from the repo root:  python3 -I -B source_checks.py SCRATCH_DIR
Writes only in SCRATCH_DIR. Scene tests read each scene's own bytes through include_str!, so they run without fmn.
"""
import difflib
import os
import re
import shutil
import subprocess
import sys

ROOT = os.getcwd()
S = os.path.abspath(sys.argv[1])
os.makedirs(S, exist_ok=True)
R = ROOT + "/docs/video-program/third-wave-20261024/opus-replacements-20261104/"
D = R + "exponential-abc-teaching-repair01/"
C = D + "candidate/"
F = R + "exponential-rate-facts01/candidate/"
SCENES = {"rate_law": C + "rate02/rate_law.rs", "factorials": C + "series01/factorials.rs", "product_grid": C + "product01/product_grid.rs"}
SLICES = {"rate": "rate02/script-slice.tsv", "series": "series01/script-slice.tsv", "product": "product01/script-slice.tsv"}
LINT = os.path.expanduser("~/.claude/skills/math-explainer-videos-end-to-end-with-franken-manim-frankentts/scripts/lint_script.py")
fails = 0


def ck(ok, label, extra=""):
    global fails
    print(("PASS " if ok else "FAIL ") + label + (("  " + extra) if extra else ""))
    fails += 0 if ok else 1


def rel(s):
    return s.replace(ROOT + "/", "").replace(S + "/", "<scratch>/").replace(
        "docs/video-program/third-wave-20261024/opus-replacements-20261104/", ""
    )


print("--- the scene source-order tests, compiled and run for real")
for name, path in SCENES.items():
    d = os.path.join(S, "scene_tests", name)
    os.makedirs(d, exist_ok=True)
    shutil.copy(path, d)
    text = open(path, encoding="utf-8").read()
    head, tests = text.split("#[cfg(test)]", 1)
    # The tests read the scene's text constants through `use super::*`; the rest of the scene needs fmn and stays out.
    consts = "".join(m.group(0) + "\n" for m in re.finditer(r"^const \w+: (?:&str|\[\(&str, &str\); 3\]) =.*?;$", head, re.M | re.S))
    mp = os.path.join(d, "main.rs")
    with open(mp, "w", encoding="utf-8") as out:
        out.write(consts + "#[cfg(test)]" + tests)
    b = os.path.join(d, "t")
    r = subprocess.run(["rustc", "--edition", "2024", "--test", "-A", "warnings", "-o", b, mp], capture_output=True, text=True)
    if r.returncode:
        ck(False, f"{name}: tests compile")
        print(rel(r.stderr)[-1500:])
        continue
    t = subprocess.run([b], capture_output=True, text=True)
    res = [l for l in t.stdout.splitlines() if l.startswith("test ")]
    ck(t.returncode == 0, f"{name}: scene tests", res[-1] if res else "")
    for l in res[:-1]:
        print("    ", l)
    if t.returncode:
        print(rel(t.stdout)[-1500:])

print("--- a scene test that cannot fail proves nothing: move a hook and the order test must fail")
d = os.path.join(S, "scene_tests", "factorials_control")
os.makedirs(d, exist_ok=True)
text = open(SCENES["factorials"], encoding="utf-8").read()
head, tests = text.split("#[cfg(test)]", 1)
moved = head.replace('k.until_sentence(stage, "02_3", 3)?;', "", 1)
moved_file = moved + "#[cfg(test)]" + tests
with open(os.path.join(d, "factorials.rs"), "w", encoding="utf-8") as out:
    out.write(moved_file)
consts = "".join(m.group(0) + "\n" for m in re.finditer(r"^const \w+: (?:&str|\[\(&str, &str\); 3\]) =.*?;$", head, re.M | re.S))
with open(os.path.join(d, "main.rs"), "w", encoding="utf-8") as out:
    out.write(consts + "#[cfg(test)]" + tests)
b = os.path.join(d, "t")
r = subprocess.run(["rustc", "--edition", "2024", "--test", "-A", "warnings", "-o", b, os.path.join(d, "main.rs")], capture_output=True, text=True)
t = subprocess.run([b], capture_output=True, text=True)
ck(r.returncode == 0 and t.returncode != 0, "control: removing 02_3's hook s3 fails the factorial order test")

print("--- rustfmt on scratch copies, repo config (edition 2024, width 120)")
fmt_dir = os.path.join(S, "fmt")
os.makedirs(fmt_dir, exist_ok=True)
shutil.copy(ROOT + "/rustfmt.toml", fmt_dir)
for label, p in (
    ("rate_law.rs", SCENES["rate_law"]),
    ("factorials.rs", SCENES["factorials"]),
    ("product_grid.rs", SCENES["product_grid"]),
    ("series_model.rs", C + "series01/series_model.rs"),
    ("product_model.rs", C + "product01/product_model.rs"),
):
    c = os.path.join(fmt_dir, label)
    shutil.copy(p, c)
    r = subprocess.run(["rustfmt", "--check", "--config-path", fmt_dir, c], capture_output=True, text=True, cwd=fmt_dir)
    ck(r.returncode == 0, f"rustfmt clean: {label}")

print("--- cue texts")
cues, base_cues = {}, {}
for name, f in SLICES.items():
    for store, folder in ((cues, C), (base_cues, D + "base/")):
        for line in open(folder + f, encoding="utf-8"):
            if "\t" in line:
                i, t = line.rstrip("\n").split("\t", 1)
                store[i] = t
print(f"  {len(cues)} cues, {sum(len(t.split()) for t in cues.values())} words; base {len(base_cues)} cues, {sum(len(t.split()) for t in base_cues.values())} words")
ck(not [i for i, t in cues.items() if re.search(r"\d", t)], "no digits in any slice's words")
ck("Michael" not in "".join(cues.values()), "no narrator name in the words")
print("--- every changed word, from base to candidate")
changed = sorted(i for i in cues if cues[i] != base_cues.get(i))
ck(changed == ["02_6", "03_4", "03_5", "03_7"], "exactly 02_6, 03_4, 03_5 and the new 03_7 differ", str(changed))
for i in changed:
    if i not in base_cues:
        print(f"  {i} NEW ({len(cues[i].split())} words): {cues[i]}")
        continue
    a, b = base_cues[i].split(), cues[i].split()
    for op, x1, x2, y1, y2 in difflib.SequenceMatcher(None, a, b).get_opcodes():
        if op != "equal":
            print(f"  {i}: {' '.join(a[x1:x2])!r} -> {' '.join(b[y1:y2])!r}")
print("--- sentence counts: the hook indices each scene depends on")
split = lambda t: re.split(r"(?<=[.?!])\s+", t)
for i in sorted(cues):
    if i in changed and i in base_cues:
        print(f"  {i}: {len(split(base_cues[i]))} sentences before, {len(split(cues[i]))} after")
        ck(len(split(base_cues[i])) == len(split(cues[i])), f"{i}: sentence count unchanged")
print("--- skill lint on the amended slices, unchanged words included")
for name, f in SLICES.items():
    r = subprocess.run(["python3", "-I", "-B", LINT, C + f], capture_output=True, text=True)
    lines = [l for l in r.stdout.splitlines() if l.strip()]
    ck(r.returncode == 0, f"lint {name}", lines[-1] if lines else "")
    for l in lines[:-1]:
        print("    ", l)

print("--- the sentences the clause hooks are cut from are the cue's own words")
for scene, name, cue in (
    ("factorials", "OBLIGATIONS_SENTENCE", "02_4"),
    ("factorials", "DEBTS_SENTENCE", "02_6"),
    ("product_grid", "ROWS_SENTENCE", "03_2"),
    ("product_grid", "LADDER_SENTENCE", "03_7"),
):
    text = open(SCENES[scene], encoding="utf-8").read()
    m = re.search(name + r': &str =\s*"([^"]*)";', text)
    ck(bool(m) and m.group(1) in split(cues[cue]), f"{name} is one whole sentence of {cue}")
print("--- cue ids and text constants in the scenes")
for name, p in SCENES.items():
    text = open(p, encoding="utf-8").read().split("#[cfg(test)]")[0]
    says = re.findall(r'\.say\((?:stage|s), "(\d\d_\d+)"\)', text)
    ck(len(says) == len(set(says)) and all(x in cues for x in says), f"{name}: each cue said once, and in the slice", " ".join(says))
    print(f"  {name}: Transform::new {text.count('Transform::new(')}, transform_from_copy {text.count('transform_from_copy')}, TransformMatchingTex::new {text.count('TransformMatchingTex::new')}, always_redraw {text.count('always_redraw')}, until_sentence {text.count('until_sentence')}")
ft = open(SCENES["factorials"], encoding="utf-8").read().split("#[cfg(test)]")[0]
pt = open(SCENES["product_grid"], encoding="utf-8").read().split("#[cfg(test)]")[0]
ck(r"u_{k+1}" in ft and "a_0" not in ft and "a_{k" not in ft, "N8: the recurrence's letter is u, and a_k is gone")
ck("LETTER_TERMS" in ft and '"+ax"' in ft and "DOTS" in ft, "N2: the polynomial and its dots are drawn")
ck("indicate" in ft and "print_equation" in ft and "solve(" in ft, "N3: matching is movement, not a label")
ck("TEXT[" not in ft and "TEX[" not in ft, "N13 closed: no index into an unchecked array in the series scene")
ck(r"\frac12" in pt and "e_label()" in pt and "E(x)=e^x" in pt, "C4: the name, the ladder and the earned identity are drawn")
print("--- the three oracles merged by plain concatenation, stdout only")
merged = ""
for p in (F + "oracle.py", C + "series01/oracle.py", C + "product01/oracle.py"):
    merged += open(p, encoding="utf-8").read() + "\n"
mp = os.path.join(S, "oracle_merged.py")
with open(mp, "w", encoding="utf-8") as out:
    out.write(merged)
r = subprocess.run(["python3", "-I", "-B", mp], capture_output=True, text=True)
ck(r.returncode == 0, "concatenated oracle runs", f"{len(r.stdout.splitlines())} rows")
if r.returncode:
    print(r.stderr[-800:])
print("RESULT", "PASS" if not fails else f"{fails} FAIL")
sys.exit(1 if fails else 0)
