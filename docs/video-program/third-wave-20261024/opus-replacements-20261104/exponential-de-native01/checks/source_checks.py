"""Source checks: the scene tests for real, formatting, cue words, lint, hook binding, oracle concatenation.

Usage from the repo root:  python3 -I -B source_checks.py SCRATCH_DIR
SCRATCH_DIR/integrated must come from make_integration.py. Scene tests read each scene's own bytes through
include_str! and the script through a stand-in for crate::phrase, so they run without fmn.
"""
import os
import re
import shutil
import subprocess
import sys

ROOT = os.getcwd()
S = os.path.abspath(sys.argv[1])
CRATE = S + "/integrated"
WORK = S + "/source"
os.makedirs(WORK, exist_ok=True)
HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
L, O = HERE + "/candidate/limit01/", HERE + "/candidate/only01/"
SCENES = {"tiny_multipliers": L + "tiny_multipliers.rs", "only_solution": O + "only_solution.rs"}
LINT = os.path.expanduser("~/.claude/skills/math-explainer-videos-end-to-end-with-franken-manim-frankentts/scripts/lint_script.py")
fails = 0


def ck(ok, label, extra=""):
    global fails
    print(("PASS " if ok else "FAIL ") + label + (("  " + extra) if extra else ""))
    fails += 0 if ok else 1


def rel(s):
    return s.replace(ROOT + "/", "").replace(S + "/", "<scratch>/")


def harness(name, path, mutate=None, suffix=""):
    d = os.path.join(WORK, name + suffix)
    os.makedirs(d, exist_ok=True)
    text = open(path, encoding="utf-8").read()
    if mutate:
        text = mutate(text)
    with open(os.path.join(d, name + ".rs"), "w", encoding="utf-8") as out:
        out.write(text)
    shutil.copy(CRATE + "/script.tsv", d)
    head, tests = text.split("#[cfg(test)]", 1)
    consts = "".join(
        m.group(0) + "\n" for m in re.finditer(r"^const \w+: (?:&str|\[&str; \d+\]) =.*?;$", head, re.M | re.S)
    )
    stub = 'mod phrase {\n    pub const SCRIPT: &str = include_str!("script.tsv");\n}\n'
    mp = os.path.join(d, "main.rs")
    with open(mp, "w", encoding="utf-8") as out:
        out.write(stub + consts + "#[cfg(test)]" + tests)
    b = os.path.join(d, "t")
    r = subprocess.run(["rustc", "--edition", "2024", "--test", "-A", "warnings", "-o", b, mp], capture_output=True, text=True)
    if r.returncode:
        return r, None
    return r, subprocess.run([b], capture_output=True, text=True)


print("--- the scene source-order tests, compiled and run for real")
for name, path in SCENES.items():
    r, t = harness(name, path)
    if t is None:
        ck(False, f"{name}: tests compile")
        print(rel(r.stderr)[-2000:])
        continue
    res = [l for l in t.stdout.splitlines() if l.startswith("test ")]
    ck(t.returncode == 0, f"{name}: scene tests", res[-1] if res else "")
    for l in res[:-1]:
        print("    ", l)
    if t.returncode:
        print(rel(t.stdout)[-3000:])

print("--- a scene test that cannot fail proves nothing")
controls = (
    ("tiny_multipliers", "removing 05_5's first hook", lambda t: t.replace('until_phrase(k, stage, "05_5", "so push")?;', "", 1), "the_cut_is_chosen"),
    ("tiny_multipliers", "moving a phrase out of the words is caught", lambda t: t.replace('("05_7", "and three")', '("05_7", "and four")', 1), "every_phrase_hook"),
    ("tiny_multipliers", "moving n before the cut", lambda t: t.replace('let line = stage.arena_mut().add(\n        DashedLine::new(\n            v(cut_x(EARLY_CUT)', 'term_bar(stage, 1, 1)?;\n    let line = stage.arena_mut().add(\n        DashedLine::new(\n            v(cut_x(EARLY_CUT)', 1), None),
    ("only_solution", "showing the answer before the think gap", lambda t: t.replace('k.mark(stage, "predict");', "", 1), "the_think_gap"),
    ("only_solution", "dropping the uniqueness row from the not-used list", lambda t: t.replace('"uniqueness theorem", "logarithm"];', '"logarithm", "logarithm"];', 1), "the_result_enters"),
)
for i, (name, label, mutate, needle) in enumerate(controls):
    if needle is None:
        # Put the term_bar after the DashedLine instead, the direction the test forbids.
        mutate = lambda t: t.replace('    k.allow_crossing(line);\n    let name = k.text(\n        stage,\n        "cut",', '    k.allow_crossing(line);\n    term_bar(stage, 1, 1)?;\n    let name = k.text(\n        stage,\n        "cut",', 1)
    r, t = harness(name, SCENES[name], mutate, f"_c{i}")
    fired = t is not None and t.returncode != 0
    ck(fired, f"control: {label} fails a scene test")
    if t is not None and needle:
        ck(needle in t.stdout, f"  and it is {needle}…")

print("--- rustfmt on scratch copies, repo config (edition 2024, width 120)")
fmt_dir = os.path.join(WORK, "fmt")
os.makedirs(fmt_dir, exist_ok=True)
shutil.copy(ROOT + "/rustfmt.toml", fmt_dir)
for label, p in (
    ("tiny_multipliers.rs", SCENES["tiny_multipliers"]),
    ("only_solution.rs", SCENES["only_solution"]),
    ("limit_model.rs", L + "limit_model.rs"),
    ("solution_model.rs", O + "solution_model.rs"),
):
    c = os.path.join(fmt_dir, label)
    shutil.copy(p, c)
    r = subprocess.run(["rustfmt", "--edition", "2024", "--check", "--config-path", fmt_dir, c], capture_output=True, text=True, cwd=fmt_dir)
    ck(r.returncode == 0, f"rustfmt clean: {label}")
    if r.returncode:
        print(rel(r.stdout)[:1500])

print("--- cue texts")
cues = {}
for f in (L + "script-slice.tsv", O + "script-slice.tsv"):
    for line in open(f, encoding="utf-8"):
        if "\t" in line:
            i, t = line.rstrip("\n").split("\t", 1)
            cues[i] = t
print(f"  {len(cues)} cues, {sum(len(t.split()) for t in cues.values())} words")
ck(sorted(k for k in cues if k.startswith("04_")) == sorted(f"04_{i}" for i in range(1, 7)), "cues 04_1 to 04_6")
ck(sorted(k for k in cues if k.startswith("05_")) == sorted(f"05_{i}" for i in range(1, 9)), "cues 05_1 to 05_8")
ck(sorted(k for k in cues if k.startswith("06_")) == sorted(f"06_{i}" for i in range(1, 10)), "cues 06_1 to 06_9")
ck(not [i for i, t in cues.items() if re.search(r"\d", t)], "no digits in the words")
ck(not re.search(r"Michael|FrankenTTS|Oliver", " ".join(cues.values())), "no narrator name in the words")
base = {l.split("\t", 1)[0] for l in open(CRATE + "/script.tsv", encoding="utf-8") if "\t" in l}
ck(all(i in base for i in cues), "every slice cue is in the integrated script")
ck(len(base) == 18 + 23, "the integrated script holds A-C's 18 and D-E's 23", str(len(base)))
print("--- skill lint on the slices")
for name, f in (("limit", L + "script-slice.tsv"), ("only", O + "script-slice.tsv")):
    r = subprocess.run(["python3", "-I", "-B", LINT, f], capture_output=True, text=True)
    lines = [l for l in r.stdout.splitlines() if l.strip()]
    ck(r.returncode == 0, f"lint {name}", lines[-1] if lines else "")
    for l in lines[:-1]:
        print("    ", l)

print("--- every hook in the scenes names a real sentence or a real phrase")
split = lambda t: re.split(r"(?<=[.?!])\s+", t)
for name, p in SCENES.items():
    code = open(p, encoding="utf-8").read().split("#[cfg(test)]")[0]
    says = re.findall(r'\.say\(stage, "(\d\d_\d+)"\)', code)
    ck(len(says) == len(set(says)) and all(x in cues for x in says), f"{name}: each cue said once, and in its slice", str(len(says)))
    bad = [(c, j) for c, j in re.findall(r'until_sentence\(stage, "(\d\d_\d+)", (\d+)\)', code) if int(j) >= len(split(cues[c]))]
    ck(not bad, f"{name}: every until_sentence index exists", str(bad))
    phrases = re.findall(r'until_phrase\(k, stage, "(\d\d_\d+)", "([^"]+)"\)', code)
    ck(all(ph in cues[c] and cues[c].count(ph) == 1 for c, ph in phrases), f"{name}: every phrase occurs exactly once in its cue", str(len(phrases)))
    ck("until_clause" not in code, f"{name}: no until_clause, which ignores word marks")
    for c in says:
        hooks = [int(j) for cc, j in re.findall(r'until_sentence\(stage, "(\d\d_\d+)", (\d+)\)', code) if cc == c]
        print(f"  {c}: {len(split(cues[c]))} sentences; hooks at {hooks}; phrases {[ph for cc, ph in phrases if cc == c]}")

print("--- the debts keep series-chapter words")
script = open(CRATE + "/src/chapters/factorials.rs", encoding="utf-8").read()
for debt in ("Why adding inputs multiplies outputs", "Why the stepped approximations reach E", "Why E is the only solution from height one"):
    ck(debt in script, f"chapter 02 wrote it: {debt}")
    ck(debt in open(SCENES["only_solution"], encoding="utf-8").read(), "  and 06_9 repeats it")

print("--- nothing earlier in the film claims uniqueness as proved")
abc = "".join(l for l in open(CRATE + "/script.tsv", encoding="utf-8") if re.match(r"0[1-3]_", l))
hits = [l.split("\t")[0] for l in abc.splitlines() if re.search(r"\bonly\b", l)]
print("  A-C lines with 'only':", hits)
for l in abc.splitlines():
    if l.split("\t")[0] in hits:
        print("   ", l[:170])

print("RESULT", "PASS" if not fails else f"{fails} FAIL")
sys.exit(1 if fails else 0)
