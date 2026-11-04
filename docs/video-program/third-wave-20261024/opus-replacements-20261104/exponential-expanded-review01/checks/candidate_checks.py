#!/usr/bin/env python3
"""Check the staged repairs without building anything. Run from the repo root.

1. Each patch, applied to a scratch copy of its base, reproduces the staged candidate,
   and each base still equals the root's original byte for byte.
2. rustfmt parses and accepts the staged Rust. This proves parsing, not types.
3. A Python mirror of the staged fact exporters agrees with the staged oracles, key by
   key. Python floats and Rust f64 share IEEE 754 arithmetic and both formatters round
   the exact binary value, so this predicts facts-check. It is not native execution.
4. Every cue and sentence the staged scenes wait for exists, and the indexed strings in
   the staged scenes are the ones the beats need.
5. A replay of the staged scenes against the Kit's scratch timing rule. Arithmetic from
   source, not measurement: no Oliver take exists. Each scene restates the source's order
   of plays, so a source change needs the same change here.
6. No machine paths or user names in this folder.
"""
import contextlib
import filecmp
import getpass
import importlib.util
import io
import os
import re
import shutil
import subprocess
import sys
import tempfile
from fractions import Fraction

ROOT = "docs/video-program/third-wave-20261024/opus-replacements-20261104/"
HERE = ROOT + "exponential-expanded-review01/"
BREATH = 0.6
failed = False


def check(ok, message):
    global failed
    print(("PASS  " if ok else "FAIL  ") + message)
    failed |= not ok


def read(path):
    return open(path, encoding="utf-8").read()


# ---- 1. patches
SETS = {
    "series01": ("exponential-expanded-series01/", "series-repair.patch", ("factorials.rs", "series_model.rs", "oracle.py")),
    "product01": ("exponential-expanded-product01/", "product-repair.patch", ("product_grid.rs", "product_model.rs", "oracle.py")),
}
for name, (original, patch, files) in SETS.items():
    for f in files:
        check(filecmp.cmp(HERE + f"base/{name}/{f}", ROOT + original + f, shallow=False), f"base {name}/{f} equals the original")
    with tempfile.TemporaryDirectory() as scratch:
        for f in files:
            shutil.copy(HERE + f"base/{name}/{f}", scratch)
        run = subprocess.run(
            ["patch", "-p1", "--no-backup-if-mismatch", "-s", "-i", os.path.abspath(HERE + "patches/" + patch)],
            cwd=scratch,
            capture_output=True,
            text=True,
        )
        check(run.returncode == 0, f"{patch} applies to base {name} ({run.stdout.strip() or 'clean'})")
        for f in files:
            same = filecmp.cmp(os.path.join(scratch, f), HERE + f"candidate/{name}/{f}", shallow=False)
            check(same, f"{patch} reproduces candidate {name}/{f}")

# ---- 2. rustfmt
for path in (
    "series01/factorials.rs",
    "series01/series_model.rs",
    "product01/product_grid.rs",
    "product01/product_model.rs",
):
    run = subprocess.run(
        ["rustfmt", "--edition", "2024", "--config-path", "rustfmt.toml", "--check", HERE + "candidate/" + path],
        capture_output=True,
        text=True,
    )
    check(run.returncode == 0, f"rustfmt --check {path}")
    if run.returncode:
        print(run.stdout[:2000], run.stderr[:2000])


for path in (
    "exponential-expanded-rate02/rate_law.rs",
    "exponential-expanded-rate02/rate_model.rs",
    "exponential-rate-facts01/candidate/src/rate_model.rs",
    "exponential-expanded-series01/factorials.rs",
    "exponential-expanded-series01/series_model.rs",
    "exponential-expanded-product01/product_grid.rs",
    "exponential-expanded-product01/product_model.rs",
):
    run = subprocess.run(
        ["rustfmt", "--edition", "2024", "--config-path", "rustfmt.toml", "--check", ROOT + path],
        capture_output=True,
        text=True,
    )
    check(run.returncode == 0, f"rustfmt --check original {path}")

# ---- script lint from the end-to-end skill, on the originals (the patches change no cue text)
LINT = "~/.claude/skills/math-explainer-videos-end-to-end-with-franken-manim-frankentts/scripts/lint_script.py"
for folder in ("exponential-expanded-rate02", "exponential-expanded-series01", "exponential-expanded-product01"):
    run = subprocess.run(
        [sys.executable, "-I", "-B", os.path.expanduser(LINT), ROOT + folder + "/script-slice.tsv"],
        capture_output=True,
        text=True,
    )
    check(run.returncode == 0, f"script lint {folder}: {run.stdout.strip().splitlines()[-1]}")
    for line in run.stdout.splitlines():
        if "colon" in line:
            print("     ", line)


# ---- 3. fact mirrors
def oracle(path):
    run = subprocess.run([sys.executable, "-I", "-B", path], capture_output=True, text=True)
    assert run.returncode == 0, (path, run.stderr)
    return dict(line.split("\t", 1) for line in run.stdout.splitlines())


def compare(name, ours, theirs):
    wrong = [(k, ours[k], theirs[k]) for k in ours if k in theirs and ours[k] != theirs[k]]
    only = sorted(set(ours) ^ set(theirs))
    check(not wrong and not only, f"{name}: {len(ours)} mirrored, {len(theirs)} oracle rows, {len(wrong)} differ, {len(only)} unmatched")
    for row in wrong[:20]:
        print("  DIFFERS", *row)
    for row in only[:20]:
        print("  UNMATCHED", row)


RADIUS, TOP, SEGMENTS, TAIL = 1.5, 5.0, 60, 4


def factorial(n):
    out = 1
    for k in range(1, n + 1):
        out *= k
    return out


def plot_x(j):
    return RADIUS * float(j) / float(SEGMENTS)


def partial(x, n):
    term = 1.0
    total = term
    for k in range(1, n + 1):
        term *= x / float(k)
        total += term
    return total


def tail_ratio(radius, n):
    return radius / float(n + 2)


def tail_bound(radius, n):
    return radius ** (n + 1) / float(factorial(n + 1)) / (1.0 - tail_ratio(radius, n))


def ceil_float(x):
    whole = int(x)
    return float(whole + 1) if x > whole else float(whole)


def tail_label(bound):
    return f"{ceil_float(bound * 1e5) / 1e5:.5f}"


series = {
    "factorials.plot.radius": f"{RADIUS:.6f}",
    "factorials.plot.top": f"{TOP:.6f}",
    "factorials.plot.segments": str(SEGMENTS),
}
for n in range(5):
    d = factorial(n)
    series[f"factorials.coefficient.{n}"] = "1" if d == 1 else f"1/{d}"
    series[f"factorials.denominator.{n}"] = str(d)
    for j in range(SEGMENTS + 1):
        x = plot_x(j)
        series[f"factorials.curve.{n}.{j}.x"] = f"{x:.6f}"
        series[f"factorials.curve.{n}.{j}.y"] = f"{partial(x, n):.5f}"
series["factorials.tail.order"] = str(TAIL)
series["factorials.tail.ratio"] = f"{tail_ratio(RADIUS, TAIL):.2f}"
series["factorials.tail.bound"] = tail_label(tail_bound(RADIUS, TAIL))
compare("series", series, oracle(HERE + "candidate/series01/oracle.py"))
check(series["factorials.tail.bound"] == "0.08438", "tail label through the native double path prints 0.08438")

# Which precision is free of exact decimal ties? The native sum and the exact fraction can
# round a tie apart, so the export should use a precision with none.
for precision in (3, 4, 5, 6, 7, 8, 9):
    apart = 0
    for n in range(5):
        for j in range(SEGMENTS + 1):
            exact = sum((Fraction(3, 2) * j / SEGMENTS) ** k / factorial(k) for k in range(n + 1))
            apart += f"{partial(plot_x(j), n):.{precision}f}" != f"{float(exact):.{precision}f}"
    print(f"      curve heights at {precision} places: {apart} rows round apart")

SIDE = 4
product = {"product.side": str(SIDE), "product.cells": str(SIDE * SIDE)}
for i in range(SIDE):
    for j in range(SIDE):
        product[f"product.cell.{i}.{j}.denominator"] = str(factorial(i) * factorial(j))
for m in range(SIDE):
    product[f"product.diagonal.{m}.cells"] = str(m + 1)
    product[f"product.total.{m}.denominator"] = str(factorial(m))
compare("product", product, oracle(HERE + "candidate/product01/oracle.py"))


# ---- 4. beats exist, indexed strings
def load(path):
    rows = {}
    for row in read(ROOT + path).splitlines():
        cue, text = row.split("\t", 1)
        rows[cue] = text
    return rows


def sentences(text):
    return [s.strip() for s in re.findall(r".+?[.?!](?= |$)", text)]


series_script = load("exponential-expanded-series01/script-slice.tsv")
grid_script = load("exponential-expanded-product01/script-slice.tsv")
for path, script in (
    ("series01/factorials.rs", series_script),
    ("product01/product_grid.rs", grid_script),
):
    source = read(HERE + "candidate/" + path).split("#[cfg(test)]")[0]
    waits = re.findall(r'until_sentence\(\w+, "(\d\d_\d)", (\d+)\)', source)
    said = re.findall(r'k\.say\(\w+, "(\d\d_\d)"\)', source)
    bad = [(c, j) for c, j in waits if c not in script or int(j) >= len(sentences(script[c]))]
    check(not bad, f"{path}: {len(waits)} sentence waits name real sentences {bad or ''}")
    check(said == sorted(script), f"{path}: says every cue once, in order")
    text_entries = re.search(r"pub const TEXT: &\[&str\] = &\[(.*?)\n\];", source, re.S).group(1)
    tex_entries = re.search(r"pub const TEX: &\[&str\] = &\[(.*?)\n\];", source, re.S).group(1)
    text_count = len(re.findall(r'^\s*"', text_entries, re.M))
    tex_count = len(re.findall(r'^\s*r"', tex_entries, re.M))
    text_used = {int(i) for i in re.findall(r"\bTEXT\[(\d+)\]", source)}
    tex_used = {int(i) for i in re.findall(r"\bTEX\[(\d+)\]", source)}
    tex_used |= {1, 2} if path.endswith("product_grid.rs") else set()
    check(max(text_used) < text_count and max(tex_used) < tex_count, f"{path}: indices in range ({text_count} TEXT, {tex_count} TEX)")
    print(f"      TEXT drawn {sorted(text_used)}; TEX drawn {sorted(tex_used)}")
    for i in range(tex_count):
        if i not in tex_used:
            check(False, f"{path}: TEX[{i}] never drawn")


# The staged source-order tests, replayed, since cargo test cannot run here. Each must
# pass on its candidate and fail on the base, or it would not catch the defect.
def needle_lists(source):
    tests = source.split("#[cfg(test)]", 1)[1]  # the tests quote this marker themselves
    out = []
    for block in re.findall(r"assert_in_order\(&\[(.*?)\]\);", tests, re.S):
        out.append(re.findall(r'r#"(.*?)"#|"(.*?)"', block))
    return [[raw or plain for raw, plain in block] for block in out]


def in_order(source, needles):
    scene, start = source.split("#[cfg(test)]")[0], 0
    for needle in needles:
        at = scene.find(needle, start)
        if at < 0:
            return needle
        start = at + len(needle)
    return None


for name, f in (("series01", "factorials.rs"), ("product01", "product_grid.rs")):
    source = read(HERE + f"candidate/{name}/{f}")
    base = read(HERE + f"base/{name}/{f}")
    lists = needle_lists(source)
    check(len(lists) == 2, f"{f}: two source-order tests found")
    for needles in lists:
        check(in_order(source, needles) is None, f"{f}: order holds in the candidate ({len(needles)} steps) {in_order(source, needles) or ''}")
        check(in_order(base, needles) is not None, f"{f}: the same order fails on the base, at {in_order(base, needles)!r}")

# ---- 4b. widths of the new or moved plain text, from the pinned fonts
spec = importlib.util.spec_from_file_location("text_fit", HERE + "checks/text_fit.py")
fit = importlib.util.module_from_spec(spec)
with contextlib.redirect_stdout(io.StringIO()):
    spec.loader.exec_module(fit)
for text, size, centre in (
    ("|x| ≤ 1.5, degree 4: ratio ≤ 0.25, tail ≤ 0.08438", 25.0, 0.0),
    ("What is the degree-two diagonal?", 23.0, 4.2),
    ("Complete diagonals: degrees 0 through 3", 22.0, 4.7),
    ("Absolute convergence permits regrouping", 22.0, 0.0),
):
    w = fit.width(text, size)
    left, right = centre - w / 2, centre + w / 2
    check(-7.11 < left and right < 7.11, f"on frame: {w:.2f} wide, {left:.2f} to {right:.2f}  {text!r}")


# ---- 5. timing replay
class Sim:
    def __init__(self, script, factor):
        self.script, self.factor = script, factor
        self.t, self.busy = 0.0, 0.0
        self.said = {}
        self.current = None

    def speech(self, cue):
        return len(self.script[cue].split()) / 2.6 * self.factor

    def at(self, cue, j):
        parts = sentences(self.script[cue])
        total = sum(len(s) for s in parts)
        return self.said[cue] + self.speech(cue) * sum(len(s) for s in parts[:j]) / total

    def say(self, cue):
        if self.current:
            self.t = max(self.t, self.busy + BREATH)
        self.said[cue] = self.t
        self.busy = self.t + self.speech(cue) + 0.4
        self.current = cue

    def hold(self):
        self.t = max(self.t, self.busy + BREATH)

    def until(self, cue, j):
        self.t = max(self.t, self.at(cue, j))

    def play(self, seconds):
        self.t += seconds
        return self.t


def factorials(sim):
    out = {}
    sim.say("02_1")
    sim.play(0.35)
    sim.until("02_1", 2)
    out["02_1 table lands, seconds after sentence 2 starts"] = sim.play(0.6) - sim.at("02_1", 2)
    sim.until("02_1", 3)
    sim.play(0.35)
    moves = sim.play(4 * (0.35 + 0.6 + 0.35))
    out["02_1 moves end, seconds before sentence 4 starts"] = sim.at("02_1", 4) - moves
    sim.until("02_1", 4)
    out["02_1 question lands, seconds after sentence 4 starts"] = sim.play(0.35) - sim.at("02_1", 4)
    sim.hold()
    out["02_1 think time before the answer"] = 4.0 + 0.3
    sim.play(4.0)
    sim.play(0.3)
    sim.say("02_2")
    for sentence in (0, 3, 5):
        sim.until("02_2", sentence)
        sim.play(0.35)
    sim.say("02_3")
    sim.until("02_3", 1)
    sim.play(0.35)
    sim.until("02_3", 2)
    sim.play(0.35)
    sim.until("02_3", 4)
    sim.play(0.3)
    sim.play(0.4)
    out["02_3 first curve starts, seconds after sentence 4 starts"] = sim.t - sim.at("02_3", 4)
    done = sim.play(0.5 + 0.6 + 4 * 1.1)
    out["02_3 last curve done, seconds before the cue ends"] = sim.busy - done
    sim.hold()
    sim.say("02_4")
    sim.play(0.35)
    sim.until("02_4", 3)
    sim.play(0.35)
    sim.hold()
    sim.play(0.3)
    sim.say("02_5")
    sim.play(0.4)
    sim.until("02_5", 1)
    sim.play(0.35)
    sim.until("02_5", 3)
    sim.play(0.7)
    sim.until("02_5", 5)
    out["02_5 conclusion lands, seconds after sentence 5 starts"] = sim.play(0.35) - sim.at("02_5", 5)
    sim.hold()
    sim.say("02_6")
    sim.play(0.3 + 0.35)
    sim.hold()
    out["total seconds"] = sim.t
    return out


def grid(sim):
    out = {}
    sim.say("03_1")
    sim.play(0.35)
    sim.until("03_1", 2)
    edges = sim.play(0.4 + 8 * 0.35)
    out["03_1 edges done, seconds before sentence 3 starts"] = sim.at("03_1", 3) - edges
    sim.until("03_1", 3)
    sim.play(0.4)
    cells = sim.play(16 * 0.2)
    out["03_1 cells done, seconds after sentence 3 starts"] = cells - sim.at("03_1", 3)
    out["03_1 cells done, seconds before sentence 5 starts"] = sim.at("03_1", 5) - cells
    sim.until("03_1", 5)
    sim.play(0.35)
    sim.hold()
    sim.say("03_2")
    sim.until("03_2", 1)
    sim.play(0.3 + 0.35)
    sim.until("03_2", 2)
    sim.play(0.3)
    sim.until("03_2", 3)
    sim.play(0.35)
    sim.until("03_2", 4)
    sim.play(0.3)
    sim.until("03_2", 5)
    out["03_2 question lands, seconds after sentence 5 starts"] = sim.play(0.35) - sim.at("03_2", 5)
    sim.hold()
    sim.play(4.0)
    sim.play(0.25)
    sim.say("03_3")
    sim.play(0.35)
    sim.until("03_3", 1)
    sim.play(0.6)
    sim.until("03_3", 3)
    sim.play(0.6 + 0.35)
    sim.hold()
    sim.say("03_4")
    sim.play(0.3 + 0.35)
    sim.until("03_4", 1)
    sim.play(0.35)
    sim.until("03_4", 2)
    sim.play(0.35)
    sim.until("03_4", 3)
    sim.play(0.35)
    sim.hold()
    sim.say("03_5")
    sim.play(0.25)
    sim.until("03_5", 1)
    sim.play(0.35)
    sim.until("03_5", 2)
    sim.play(0.35)
    sim.until("03_5", 3)
    sim.play(0.35)
    sim.hold()
    sim.play(0.5)
    sim.say("03_6")
    sim.until("03_6", 1)
    sim.play(0.35)
    sim.until("03_6", 2)
    sim.play(0.35)
    sim.until("03_6", 3)
    sim.play(0.5)
    sim.until("03_6", 4)
    sim.play(0.35)
    sim.hold()
    out["total seconds"] = sim.t
    return out


for name, scene, script in (("factorials.rs", factorials, series_script), ("product_grid.rs", grid, grid_script)):
    print("timing replay,", name)
    results = {factor: scene(Sim(script, factor)) for factor in (0.8, 1.0, 1.25)}
    for key in results[1.0]:
        print(f"  {key}: {results[1.0][key]:.2f}  ({results[0.8][key]:.2f} at 0.8, {results[1.25][key]:.2f} at 1.25)")

# ---- 6. no machine paths
machine = re.compile("|".join(map(re.escape, ["/Users/", "/home/", "/private/tmp", os.path.expanduser("~"), getpass.getuser()])))
hits = []
for folder, _, names in os.walk(HERE):
    for f in names:
        path = os.path.join(folder, f)
        if f.endswith((".py", ".rs", ".md", ".patch", ".txt", "SHA256SUMS")) and f != "candidate_checks.py":
            if machine.search(open(path, encoding="utf-8", errors="replace").read()):
                hits.append(path)
check(not hits, f"no machine paths or user names in the folder {hits or ''}")

print("RESULT", "FAIL" if failed else "PASS")
sys.exit(1 if failed else 0)
