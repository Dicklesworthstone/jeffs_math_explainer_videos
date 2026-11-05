"""Compile and test this candidate against stand-in crates, with mutation controls for its new tests.

Run from the repo root:  python3 -I -B .../logarithm04-probe-repair01/checks/stand_in_run.py SCRATCH_DIR

Writes only under SCRATCH_DIR. Bare rustc and clippy-driver, no Cargo, no fmn, no real Kit. It uses the stand-ins of
logarithm-repair01-04/checks/stub, copied and brought up to canonical in three ways: Stage::get_width, Kit::mark, and
draw.rs's test module dropped because it needs the real engine. Other chapters are canonical.

This finds type, borrow, panic and bookkeeping errors. It proves nothing about a real signature, TeX, a glyph or a frame.
"""
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

REPO = Path.cwd()
OPUS = REPO / "docs/video-program/third-wave-20261024/opus-replacements-20261104"
HERE = OPUS / "logarithm04-probe-repair01"
OLD = OPUS / "logarithm-repair01-04/checks"
CRATE = "explainer_corpus/logarithm_mysteries"
CHAPTER = "src/chapters/undoing_growth.rs"
OTHERS = ("draw", "building_ruler", "slide_rule", "folded_decade")
out = Path(sys.argv[1]).resolve()
fails = 0


def clean(text):
    return text.replace(str(out), "SCRATCH").replace(str(REPO), "REPO")


def check(ok, what):
    global fails
    fails += 0 if ok else 1
    print(("PASS " if ok else "FAIL ") + what)


def tree(name, chapter_bytes):
    dest = out / name / CRATE
    (dest / "src/chapters").mkdir(parents=True, exist_ok=True)
    shutil.copyfile(REPO / CRATE / "script.tsv", dest / "script.tsv")
    for f in ("facts.rs", "model.rs"):
        shutil.copyfile(REPO / CRATE / "src" / f, dest / "src" / f)
    for f in OTHERS:
        shutil.copyfile(REPO / CRATE / "src/chapters" / f"{f}.rs", dest / "src/chapters" / f"{f}.rs")
    draw = dest / "src/chapters/draw.rs"
    lines = draw.read_text(encoding="utf-8").split("#[cfg(test)]")[0].rstrip("\n").split("\n")
    while lines and (lines[-1].strip().startswith(("///", "#[")) or not lines[-1].strip()):
        lines.pop()
    draw.write_text("\n".join(lines) + "\n", encoding="utf-8")
    (dest / CHAPTER).write_bytes(chapter_bytes)


shutil.copytree(OLD / "stub", out / "checks/stub", dirs_exist_ok=True)
shutil.copyfile(OLD / "run_against_stub.py", out / "checks/run_against_stub.py")
kit = out / "checks/stub/kit_stub.rs"
text = kit.read_text(encoding="utf-8")
text = re.sub(r'#\[path = "[^"]*crates/explainer_kit/src/([a-z_]+\.rs)"\]', lambda m: f'#[path = "{REPO}/crates/explainer_kit/src/{m.group(1)}"]', text)
if "pub fn mark" not in text:
    text = text.replace("    /// The prediction hold: music alone", "    pub fn mark(&self, _: &Stage<'_>, _: &str) {}\n\n    /// The prediction hold: music alone", 1)
kit.write_text(text, encoding="utf-8")
stub = out / "checks/stub/fmn_stub.rs"
text = stub.read_text(encoding="utf-8")
if "pub fn get_width" not in text:
    text = text.replace("    pub fn get_center(&self, mob: Mob) -> Vec3 {", "    pub fn get_width(&self, _: Mob) -> f64 {\n        1.0\n    }\n\n    pub fn get_center(&self, mob: Mob) -> Vec3 {", 1)
stub.write_text(text, encoding="utf-8")

base = (REPO / CRATE / CHAPTER).read_bytes()
candidate = (HERE / CRATE / CHAPTER).read_bytes()
tree("base", base)
tree("candidate", candidate)
build = out / "build"
build.mkdir(exist_ok=True)
done = subprocess.run([sys.executable, "-I", "-B", str(out / "checks/run_against_stub.py"), str(build), "base", "candidate"], cwd=REPO, capture_output=True, text=True)
report = clean(done.stdout + done.stderr)
keep = re.compile(r"^(######|-- |test result|test .*FAILED|error|\(clippy clean\)|== |chapter ends|\d+ object-ledger|.*failed:)")
for line in report.split("\n"):
    if keep.match(line):
        print(line)
check(done.returncode == 0, "stand-in run finished for both trees")
results = re.findall(r"test result: ok\. (\d+) passed; 0 failed", report)
check(len(results) == 2 and int(results[1]) == int(results[0]) + 2, f"tests passed {results}: base, then candidate with two more")
check("(clippy clean)" in report, "clippy-driver -D warnings clean on the candidate, with and without tests")
ends = re.findall(r"== (\S+) at ([0-9.]+) words per second.*?chapter ends at ([0-9.]+) s", report, re.S)
half = len(ends) // 2
check(half > 0 and ends[:half] == ends[half:], f"all {half} scene runs end at the same time in both trees")
print("\n-- mutation controls: each must fail to compile or fail a test")
links = ["-L", str(build), "--extern", f"fmn={build}/libfmn.rlib", "--extern", f"explainer_kit={build}/libexplainer_kit.rlib"]
controls = (
    ("no mutation (positive control)", None, None, "pass"),
    ("start back at 2.6", b"const PROBE_START: f64 = 2.16;", b"const PROBE_START: f64 = 2.6;", "E0080"),
    ("start at the label 2", b"const PROBE_START: f64 = 2.16;", b"const PROBE_START: f64 = 2.05;", "E0080"),
    ("rest back at 2.0", b"const PROBE_REST: f64 = 1.5;", b"const PROBE_REST: f64 = 2.0;", "E0080"),
    ("rest at the label 1", b"const PROBE_REST: f64 = 1.5;", b"const PROBE_REST: f64 = 1.05;", "E0080"),
    ("caption back to bare +0.01", "→  about +0.01 in ln x\"".encode(), "→  +0.01 in ln x\"".encode(), "the_small_step_caption_says_it_is_approximate"),
    ("rest literal in the scene", b"tween(c, PROBE_REST, 1.6)", b"tween(c, 2.0, 1.6)", "the_whole_probe_path_stays_under_the_name_and_rests_clear_of_the_tick_labels"),
    ("start literal in the scene", b"add_value_tracker(PROBE_START)", b"add_value_tracker(2.6)", "the_whole_probe_path_stays_under_the_name_and_rests_clear_of_the_tick_labels"),
)
support = out / "base" / CRATE / "src"
chapters = out / "mutant" / CRATE / "src/chapters"
rows = ["#![allow(dead_code, clippy::should_implement_trait)]", f'#[path = "{support}/facts.rs"]', "pub mod facts;", f'#[path = "{support}/model.rs"]', "pub mod model;", "pub mod chapters {", f'    #[path = "{support}/chapters/draw.rs"]', "    pub mod draw;"]
for name in ("building_ruler", "slide_rule", "folded_decade", "undoing_growth"):
    rows += [f'    #[path = "{chapters}/{name}.rs"]', f"    pub mod {name};"]
rows += ["}", f'include!("{out}/checks/stub/harness_main.rs");']
root = build / "mutant_root.rs"
root.write_text("\n".join(rows) + "\n", encoding="utf-8")
binary = build / "mutant_tests"
env = dict(os.environ, TREE_ROOT=str(out / "mutant" / CRATE))
for what, old, new, expect in controls:
    parts = candidate.split(b"#[cfg(test)]")
    if old is not None:
        assert parts[0].count(old) == 1, what
        # the change is in the scene code and constants, never in the test's own text
        parts[0] = parts[0].replace(old, new)
    tree("mutant", b"#[cfg(test)]".join(parts))
    comp = subprocess.run(["rustc", "--edition", "2024", *links, "--crate-name", "chapter", "--test", "-o", str(binary), str(root)], capture_output=True, text=True, env=env)
    if comp.returncode:
        codes = sorted(set(re.findall(r"error\[(E\d+)\]", comp.stderr)))
        first = clean(comp.stderr).split("\n")[0][:120]
        check(expect in codes, f"{what}: compile refused, codes {codes}, first line: {first}")
        continue
    run = subprocess.run([str(binary), "--test-threads=1"], capture_output=True, text=True)
    failed = re.findall(r"^test (\S+) \.\.\. FAILED", run.stdout, re.M)
    if expect == "pass":
        check(run.returncode == 0, f"{what}: compiles, and {re.findall(r'test result: (.*?);', run.stdout)} tests")
    else:
        check(failed == [f"chapters::undoing_growth::tests::{expect}"], f"{what}: compiles, fails {failed}")
print("RESULT " + ("PASS" if fails == 0 else f"FAIL {fails}"))
sys.exit(1 if fails else 0)
