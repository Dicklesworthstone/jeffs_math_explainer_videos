"""Source checks on the staged candidates: real scene tests, formatting, ids, oracle merge, what the screen says.

Usage from the repo root:  python3 -I -B source_checks.py SCRATCH_DIR     (SCRATCH_DIR as used by scene_typecheck.py)
Writes only in SCRATCH_DIR.
"""
import glob
import os
import re
import shutil
import subprocess
import sys

ROOT = os.getcwd()
S = os.path.abspath(sys.argv[1])
D = ROOT + "/target/debug/deps"
R = ROOT + "/docs/video-program/third-wave-20261024/opus-replacements-20261104/"
W = R + "exponential-expanded-review01/"
F = R + "exponential-rate-facts01/candidate/"
SCENES = {
    "rate_law": R + "exponential-expanded-rate02/rate_law.rs",
    "factorials": W + "candidate/series01/factorials.rs",
    "product_grid": W + "candidate/product01/product_grid.rs",
}
SLICES = {
    "rate": R + "exponential-expanded-rate02/script-slice.tsv",
    "series": R + "exponential-expanded-series01/script-slice.tsv",
    "product": R + "exponential-expanded-product01/script-slice.tsv",
}
fails = 0


def ck(ok, label, extra=""):
    global fails
    print(("PASS " if ok else "FAIL ") + label + (("  " + extra) if extra else ""))
    fails += 0 if ok else 1


def rel(s):
    return s.replace(ROOT + "/", "").replace(S + "/", "<scratch>/").replace(
        "docs/video-program/third-wave-20261024/opus-replacements-20261104/", ""
    )


print("--- the scene source-order tests, compiled and run for real (not replayed in Python)")
for name, path in SCENES.items():
    d = os.path.join(S, "scene_tests", name)
    os.makedirs(d, exist_ok=True)
    shutil.copy(path, d)  # include_str! reads the scene's own bytes
    text = open(path, encoding="utf-8").read()
    head, tests = text.split("#[cfg(test)]", 1)
    prefix = ""
    if "use super::TEXT" in tests:
        m = re.search(r"pub const TEXT: &\[&str\] = &\[.*?\];", head, re.S)
        prefix = m.group(0) + "\n"
    main = prefix + "#[cfg(test)]" + tests
    mp = os.path.join(d, "main.rs")
    with open(mp, "w", encoding="utf-8") as out:
        out.write(main)
    b = os.path.join(d, "t")
    r = subprocess.run(["rustc", "--edition", "2024", "--test", "-A", "warnings", "-o", b, mp], capture_output=True, text=True)
    if r.returncode:
        ck(False, f"{name}: tests compile")
        print(rel(r.stderr)[-1200:])
        continue
    t = subprocess.run([b], capture_output=True, text=True)
    res = [l for l in t.stdout.splitlines() if l.startswith("test ")]
    ck(t.returncode == 0, f"{name}: scene tests", res[-1] if res else "")
    for l in res[:-1]:
        print("    ", l)

print("--- rustfmt on scratch copies, repo config (edition 2024, width 120)")
fmt_dir = os.path.join(S, "fmt")
os.makedirs(fmt_dir, exist_ok=True)
shutil.copy(ROOT + "/rustfmt.toml", fmt_dir)
for label, p in (
    ("rate_law.rs", SCENES["rate_law"]),
    ("factorials.rs", SCENES["factorials"]),
    ("product_grid.rs", SCENES["product_grid"]),
    ("facts01 rate_model.rs", F + "src/rate_model.rs"),
    ("series_model.rs", W + "candidate/series01/series_model.rs"),
    ("product_model.rs", W + "candidate/product01/product_model.rs"),
    ("facts01 facts.rs", F + "src/facts.rs"),
    ("canonical facts.rs", ROOT + "/explainer_corpus/exponential_magic/src/facts.rs"),
):
    c = os.path.join(fmt_dir, label.replace(" ", "_"))
    shutil.copy(p, c)
    r = subprocess.run(["rustfmt", "--check", "--config-path", fmt_dir, c], capture_output=True, text=True, cwd=fmt_dir)
    hunks = r.stdout.count("Diff in")
    print(f"  {'clean' if r.returncode == 0 else 'NOT clean'}: {label}" + (f" ({hunks} hunks)" if hunks else ""))

print("--- cue texts")
cues = {}
for name, p in SLICES.items():
    for line in open(p, encoding="utf-8"):
        if "\t" in line:
            i, t = line.rstrip("\n").split("\t", 1)
            cues[i] = t
print(f"  {len(cues)} cues, {sum(len(t.split()) for t in cues.values())} words")
ck(not [i for i, t in cues.items() if re.search(r"\d", t)], "no digits in any slice's words (final_check treats a caption digit as a defect)")
colons = {i: t.count(":") + t.count(";") for i, t in cues.items() if ":" in t or ";" in t}
print("  colons or semicolons:", colons)
for i, t in cues.items():
    if ":" in t:
        before = t[: t.index(":")]
        print(f"    {i}: the colon sits in sentence {len(re.findall(r'[.?!] ', before))}; a full stop there would move every later index in that cue up by one")
for word in ("dots", "boxes", "bars", "column", "rows", "grey", "brackets"):
    print(f"  voice names {word!r}: {[i for i, t in cues.items() if word in t]}")

print("--- cue ids against the canonical script")
canon = [l.split("\t")[0] for l in open(ROOT + "/explainer_corpus/exponential_magic/script.tsv", encoding="utf-8") if "\t" in l]
both = sorted(set(canon) & set(cues))
print(f"  canonical has {len(canon)} cues; {len(both)} ids collide with the slices: {both}")
keys = ["00_slide_rule", "01_addition_to_multiplication", "02_natural_slope", "03_three_definitions", "04_applications", "05_imaginary_rotation", "06_logarithm_bridge"]
add = ["01_rate_law", "02_factorials", "03_product_grid"]
print("  owners of prefix 01 if the chapters are added beside canonical's:", [k for k in keys + add if k.startswith("01_")], "(check_script_owners requires exactly 1)")

print("--- literal strings and indices in the scenes")
for name, p in SCENES.items():
    text = open(p, encoding="utf-8").read().split("#[cfg(test)]")[0]
    says = re.findall(r'\.say\((?:stage|s), "(\d\d_\d+)"\)', text)
    ck(len(says) == len(set(says)), f"{name}: each cue said once", " ".join(says))
    m = re.search(r"pub const TEXT: &\[&str\] = &\[(.*?)\];", text, re.S)
    if m:
        strings = re.findall(r'"((?:[^"\\]|\\.)*)"', m.group(1))
        used = sorted({int(x) for x in re.findall(r"TEXT\[(\d+)\]", text)})
        rest = text.replace(m.group(0), "")
        dead = [(k, s) for k, s in enumerate(strings) if k not in used and f'"{s}"' not in rest]
        print(f"  {name}: TEXT has {len(strings)}; indexed {used}; never drawn: {dead}")
    t = re.search(r"pub const TEX: &\[&str\] = &\[(.*?)\];", text, re.S)
    if t:
        n = len(re.findall(r'r"', t.group(1)))
        used = {int(x) for x in re.findall(r"TEX\[(\d+)\]", text)}
        # product_grid.rs:186 draws TEX[1] and TEX[2] through TEX[index]; the pairs name them.
        loop = re.search(r"for \(sentence, index\) in \[(.*?)\]", text)
        if loop:
            used |= {int(b) for _, b in re.findall(r"\((\d+), (\d+)\)", loop.group(1))}
        used = sorted(used)
        print(f"  {name}: TEX has {n}; drawn {used}; all drawn: {used == list(range(n))}")
    print(f"  {name}: Transform::new {text.count('Transform::new(')}, TransformMatchingTex {text.count('TransformMatchingTex')}, tracker {len(re.findall('ValueTracker|tracker', text))}, Frame2 {text.count('Frame2')}, Axes {text.count('Axes')}")

print("--- what the screen ever prints that speaks to the voice")
ft = open(SCENES["factorials"], encoding="utf-8").read().split("#[cfg(test)]")[0]
body = ft.split("impl SceneConstruct")[1]
before_tail = body.split('until_sentence(stage, "02_5", 3)')[0]
tex_all = re.findall(r'r"((?:[^"])*)"', re.search(r"pub const TEX: &\[&str\] = &\[(.*?)\];", ft, re.S).group(1))
text_all = re.findall(r'"((?:[^"\\]|\\.)*)"', re.search(r"pub const TEXT: &\[&str\] = &\[(.*?)\];", ft, re.S).group(1))
tex_used = sorted({int(x) for x in re.findall(r"TEX\[(\d+)\]", before_tail)})
text_used = sorted({int(x) for x in re.findall(r"TEXT\[(\d+)\]", before_tail)})
print("  factorials: TEX drawn before 02_5 sentence 3:", tex_used, "; any with a factorial:", any("!" in tex_all[i] for i in tex_used))
print("  factorials: TEXT drawn before 02_5 sentence 3:", text_used, "; any with a factorial:", any("!" in text_all[i] for i in text_used))
print("  factorials: first TEX with a factorial:", next(i for i, t in enumerate(tex_all) if "!" in t), "(TEX[2] is drawn at 02_5 sentence 3, :222)")
print("  factorials: run-time labels before then come from coefficient(n), which prints 1/d with no factorial sign")
print("  factorials: polynomial 1+ax+bx^2 anywhere in the source:", bool(re.search(r"1\s*\+\s*a\s*x|1\+ax", ft)))
print("  factorials: an ellipsis (cdots, dots, …) anywhere in the source:", bool(re.search(r"cdots|ldots|dotsb|…|\.\.\.", ft)))
print("  factorials: highlight or colour change on the table in 02_2:", "animate()" in body.split('k.say(stage, "02_2")')[1].split('k.say(stage, "02_3")')[0])
pt = open(SCENES["product_grid"], encoding="utf-8").read().split("#[cfg(test)]")[0]
b5 = pt.split('k.say(s, "03_5")')[1].split('k.say(s, "03_6")')[0]
print("  product_grid: colour_diagonal or any animate on cells or totals in 03_5:", "color_diagonal" in b5 or "animate()" in b5)
print("  product_grid: distinct colours among the totals and the lit diagonal:", sorted(set(re.findall(r"Palette::(\w+)\(\)", pt.split("fn color_diagonal")[1].split("Ok(())")[0]))), "for cells;", "green for every total")

print("--- the three oracles merged by plain concatenation, stdout only")
merged = ""
for p in (F + "oracle.py", W + "candidate/series01/oracle.py", W + "candidate/product01/oracle.py"):
    merged += open(p, encoding="utf-8").read() + "\n"
mp = os.path.join(S, "oracle_merged.py")
with open(mp, "w", encoding="utf-8") as out:
    out.write(merged)
r = subprocess.run(["python3", "-I", "-B", mp], capture_output=True, text=True)
ck(r.returncode == 0, "concatenated oracle runs", f"{len(r.stdout.splitlines())} rows; stderr {len(r.stderr)} bytes")
if r.returncode:
    print(r.stderr[-800:])
want = open(os.path.join(S, "..", "native", "oracle_combined.tsv"), encoding="utf-8").read() if os.path.exists(os.path.join(S, "..", "native", "oracle_combined.tsv")) else None
if want is not None:
    ck(r.stdout == want, "concatenated oracle's stdout equals the three run apart")
print("RESULT", "PASS" if not fails else f"{fails} FAIL")
