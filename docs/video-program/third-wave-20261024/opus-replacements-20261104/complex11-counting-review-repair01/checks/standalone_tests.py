"""Compile and run the new tests with rustc alone: no crate, no fmn, no render.

Run from the repo root:
  python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex11-counting-review-repair01/checks/standalone_tests.py

It lifts each new test, and only the pure items it needs, out of the staged counting.rs and
builds them with the film's real model.rs, root's candidate script.tsv, and a small shim for
Vec3, Frame2 and Plane whose bodies are lifted from the real sources. Then it puts each defect
back, one at a time, and requires the named test to fail. Patch 03 gets the same treatment on
its own copy.

This is a stand-in. It does not type-check `construct`, does not link fmn and is not the
crate's `cargo test`. It writes only to a new temporary directory.
"""
import os
import subprocess
import tempfile

HERE = "docs/video-program/third-wave-20261024/opus-replacements-20261104/complex11-counting-review-repair01/"
COUNTING = "explainer_corpus/complex_analysis_visual/src/chapters/counting.rs"
STAGED = HERE + COUNTING
ROOT_SCRIPT = "narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/counting-wording-repair01/script.tsv"
ROOT_BEFORE = "narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/counting-wording-repair01/before-script.tsv"
CRATE = "explainer_corpus/complex_analysis_visual/src/"
PLOT = "crates/explainer_kit/src/plot.rs"

ITEMS = ["const REACH_NAMES_AT", "const COARSE_TITLE_AT", "pub const ROOTS_CLAIM", "fn coarse_planes"]
TESTS = [
    "the_reach_disk_never_crosses_its_name",
    "the_reach_labels_take_turns_over_the_window",
    "the_roots_argument_waits_on_the_right_sentences",
    "the_division_keeps_the_count_and_a_constant_makes_no_turns",
]
OPTIONAL_ITEM = "const A_NAME_ON_SAMPLE"
OPTIONAL_TEST = "the_samples_name_keeps_off_the_distance_line"

REACH, TURNS, SENTENCES = TESTS[0], TESTS[1], TESTS[2]
AT = "const REACH_NAMES_AT: (f64, f64) = (0.0, 2.2);"
NAME = "let reach_name = kit.text(stage, STEP_SKETCH[2], 18.0, derivative_teal(), names_at)?;"
# (what is put back, staged text, defect, test that must fail)
MUTANTS = [
    ("the label back where the disk crossed it", AT, "const REACH_NAMES_AT: (f64, f64) = (0.95, -1.2);", REACH),
    ("the label in the window's emptiest corner", AT, "const REACH_NAMES_AT: (f64, f64) = (1.14, 1.87);", REACH),
    ("the label on the window's top edge", AT, "const REACH_NAMES_AT: (f64, f64) = (0.0, 2.05);", REACH),
    ("the label under the title", AT, "const REACH_NAMES_AT: (f64, f64) = (-1.6, 2.2);", REACH),
    ("the label past the window's right edge", AT, "const REACH_NAMES_AT: (f64, f64) = (1.3, 2.2);", REACH),
    ("the title moved onto the label", "const COARSE_TITLE_AT: (f64, f64) = (-4.0, 2.4);", "const COARSE_TITLE_AT: (f64, f64) = (-3.0, 2.4);", REACH),
    ("the name built at its old place in the scene", NAME, "let reach_name = kit.text(stage, STEP_SKETCH[2], 18.0, derivative_teal(), out.p(C::new(0.95, -1.2)))?;", TURNS),
    ("covers not cleared before the name comes", "fade_out_all(stage, &[covered, covers, refused, band], 0.3)?;", "fade_out_all(stage, &[covered, refused, band], 0.3)?;", TURNS),
    ("the name carried by the sweep", "            a.move_to(disk, out.p(c), ORIGIN);\n", "            a.move_to(disk, out.p(c), ORIGIN);\n            a.move_to(reach_name, out.p(c), ORIGIN);\n", TURNS),
    ("degree_n one sentence early, root's first version", "        line.wait_sentence(stage, 2)?;\n        let degree_n", "        line.wait_sentence(stage, 1)?;\n        let degree_n", SENTENCES),
    ("the circle one sentence early", "        line.wait_sentence(stage, 3)?;\n        fade_out_all(stage, &[claim, degree_n], 0.3)?;", "        line.wait_sentence(stage, 2)?;\n        fade_out_all(stage, &[claim, degree_n], 0.3)?;", SENTENCES),
    ("the dog one sentence early", "        line.wait_sentence(stage, 4)?;\n        dog.show(stage)?;", "        line.wait_sentence(stage, 3)?;\n        dog.show(stage)?;", SENTENCES),
    ("the walk ending one sentence early", "let rt = (line.sentence(6)? - stage.scene().time()", "let rt = (line.sentence(5)? - stage.scene().time()", SENTENCES),
    ("the old claim on screen", 'pub const ROOTS_CLAIM: &str = "nonconstant polynomials have roots";', 'pub const ROOTS_CLAIM: &str = "every polynomial has roots";', SENTENCES),
]


def read(path):
    return open(path, encoding="utf-8").read()


def balanced(text, start):
    """The text from `start` through the brace that closes the first brace after it."""
    depth = 0
    at = text.index("{", start)
    while True:
        if text[at] == "{":
            depth += 1
        elif text[at] == "}":
            depth -= 1
            if depth == 0:
                return text[start : at + 1]
        at += 1


def item(text, head):
    start = text.index("\n" + head) + 1
    line_end = text.index("\n", start)
    if "const " in head and text[start:line_end].rstrip().endswith(";"):
        return text[start:line_end]
    return balanced(text, start)


def test_body(text, name):
    start = text.index(f"fn {name}(")
    return "#[test]\n" + balanced(text, start)


def shim():
    plot, draw = read(PLOT), read(CRATE + "chapters/draw.rs")
    frame = "\n".join(balanced(plot, plot.index(f"pub fn {n}(")) for n in ("left", "right", "bottom", "top"))
    frame_p = balanced(plot, plot.index("pub fn p(&self, x: f64, y: f64)"))
    plane_new = balanced(draw, draw.index("pub fn new(center: Vec3, width: f64"))
    plane_p = balanced(draw, draw.index("pub fn p(&self, z: C) -> Vec3"))
    unit = balanced(draw, draw.index("pub fn unit(&self)"))
    return f"""
use crate::model::C;
pub type Vec3 = [f64; 3];
pub fn v(x: f64, y: f64) -> Vec3 {{ [x, y, 0.0] }}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame2 {{ pub center: Vec3, pub width: f64, pub height: f64, pub x: (f64, f64), pub y: (f64, f64) }}
impl Frame2 {{
{frame_p}
{frame}
}}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plane {{ pub frame: Frame2 }}
impl Plane {{
{plane_new}
{plane_p}
{unit}
}}
"""


def run(text, script, items, tests, label, workdir):
    """Build the harness from `text`; return the failed tests and the count run."""
    folder = os.path.join(workdir, label)
    os.makedirs(folder)
    for name, body in (("counting.rs", text), ("script.tsv", script), ("model.rs", read(CRATE + "model.rs")), ("shim.rs", shim())):
        with open(os.path.join(folder, name), "w", encoding="utf-8") as out:
            out.write(body)
    lifted = "\n".join(item(text, head) for head in items)
    bodies = "\n".join(test_body(text, t) for t in tests)
    with open(os.path.join(folder, "main.rs"), "w", encoding="utf-8") as out:
        out.write(
            "#![allow(dead_code, unused_imports)]\nmod model;\nmod shim;\n"
            'pub(crate) const SCRIPT: &str = include_str!("script.tsv");\n'
            "mod counting {\nuse crate::shim::*;\nuse crate::model::{self, C, ONE, ZERO};\nuse std::f64::consts::TAU;\n"
            f"{lifted}\n#[cfg(test)]\nmod tests {{\nuse super::*;\nuse crate::model::{{circle, winding_certified, winding_count}};\n{bodies}\n}}\n}}\n"
            "fn main() {}\n"
        )
    binary = os.path.join(folder, "tests.bin")
    built = subprocess.run(
        ["rustc", "--edition", "2024", "--test", os.path.join(folder, "main.rs"), "-o", binary, "--cap-lints", "allow"],
        capture_output=True,
        text=True,
        check=False,
    )
    if built.returncode != 0:
        return None, built.stderr.replace(folder, "<work>")[:3000]
    failed = []
    for test in tests:
        done = subprocess.run([binary, "--exact", f"counting::tests::{test}"], capture_output=True, text=True, check=False)
        if done.returncode != 0:
            failed.append(test)
    return failed, len(tests)


def main():
    workdir = tempfile.mkdtemp(prefix="complex11-standalone-")
    print(subprocess.run(["rustc", "--version"], capture_output=True, text=True, check=True).stdout.strip())
    staged, script = read(STAGED), read(ROOT_SCRIPT)
    bad = 0
    failed, ran = run(staged, script, ITEMS, TESTS, "staged", workdir)
    if failed is None:
        print("STAGED DID NOT BUILD\n" + ran)
        print("RESULT FAIL")
        return
    print(f"staged: {ran} tests, failed {failed}")
    bad += len(failed)
    for k, (what, old, new, expect) in enumerate(MUTANTS):
        assert old in staged, what
        # the first occurrence is the scene's; a source-read test may quote the same line later
        failed, _ = run(staged.replace(old, new, 1), script, ITEMS, TESTS, f"mutant{k:02d}", workdir)
        caught = failed is not None and expect in failed
        print(f"{'caught' if caught else 'MISSED'} {what} -> {failed if failed is not None else 'no build'}")
        bad += 0 if caught else 1
    # the script as it was: six sentences
    failed, _ = run(staged, read(ROOT_BEFORE), ITEMS, TESTS, "old-script", workdir)
    caught = failed is not None and SENTENCES in failed
    print(f"{'caught' if caught else 'MISSED'} the old script under the new anchors -> {failed}")
    bad += 0 if caught else 1

    print("patch 03, optional, on its own copy")
    folder = os.path.join(workdir, "optional")
    os.makedirs(os.path.dirname(os.path.join(folder, COUNTING)))
    with open(os.path.join(folder, COUNTING), "w", encoding="utf-8") as out:
        out.write(staged)
    done = subprocess.run(["patch", "-p1", "--no-backup-if-mismatch", "-i", os.path.abspath(HERE + "patches/03-optional-a-name.patch")], cwd=folder, capture_output=True, text=True, check=False)
    assert done.returncode == 0, done.stdout
    optional = read(os.path.join(folder, COUNTING))
    items, tests = ITEMS + [OPTIONAL_ITEM], TESTS + [OPTIONAL_TEST]
    failed, ran = run(optional, script, items, tests, "optional-staged", workdir)
    if failed is None:
        print("OPTIONAL DID NOT BUILD\n" + ran)
        print("RESULT FAIL")
        return
    print(f"optional: {ran} tests, failed {failed}")
    bad += len(failed)
    const = "const A_NAME_ON_SAMPLE: C = C::new(0.72, 0.2);"
    for k, (what, new) in enumerate(
        [
            ("the name on the line from 0", "const A_NAME_ON_SAMPLE: C = C::new(0.72, 0.0);"),
            ("the name on the output circle", "const A_NAME_ON_SAMPLE: C = C::new(0.95, 0.2);"),
            ("the name starting somewhere else", "const A_NAME_ON_SAMPLE: C = C::new(0.72, -0.2);"),
            ("the name outside the disk", "const A_NAME_ON_SAMPLE: C = C::new(0.1, 0.3);"),
        ]
    ):
        failed, _ = run(optional.replace(const, new, 1), script, items, tests, f"optional-mutant{k}", workdir)
        caught = failed is not None and OPTIONAL_TEST in failed
        print(f"{'caught' if caught else 'MISSED'} {what} -> {failed}")
        bad += 0 if caught else 1
    print("RESULT " + ("PASS" if bad == 0 else f"FAIL {bad}"))


main()
