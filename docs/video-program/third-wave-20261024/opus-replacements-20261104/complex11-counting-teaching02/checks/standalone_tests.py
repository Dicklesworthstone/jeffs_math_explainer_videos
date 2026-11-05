"""Compile and run the staged tests with rustc alone: no crate, no fmn, no render.

Run from the repo root:
  python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex11-counting-teaching02/checks/standalone_tests.py

It lifts each staged test, and only the pure items it needs, out of the staged counting.rs and
builds them with the film's real model.rs, root's candidate script.tsv, and a small shim for
Vec3, Frame2, Plane and squash whose bodies are lifted from the real sources. Then it puts each
defect back, one at a time, and requires the named test to fail.

This is a stand-in. It does not type-check `construct`, `Turning::build` or anything that touches
fmn, and it is not the crate's `cargo test`. It writes only to a new temporary directory.
"""
import os
import subprocess
import tempfile

HERE = "docs/video-program/third-wave-20261024/opus-replacements-20261104/complex11-counting-teaching02/"
COUNTING = "explainer_corpus/complex_analysis_visual/src/chapters/counting.rs"
STAGED = HERE + COUNTING
ROOT_SCRIPT = "narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/counting-wording-repair01/script.tsv"
ROOT_BEFORE = "narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/counting-wording-repair01/before-script.tsv"
CRATE = "explainer_corpus/complex_analysis_visual/src/"
PLOT = "crates/explainer_kit/src/plot.rs"

ITEMS = [
    "const REACH_NAMES_AT",
    "const COARSE_TITLE_AT",
    "pub const ROOTS_CLAIM",
    "fn coarse_planes",
    "const A_NAME_ON_SAMPLE",
    "pub const OUT_R",
    "pub fn planes",
    "pub const LEADING_KEY",
    "pub const BEFORE_DIVIDING_TEX",
    "pub const AFTER_DIVIDING_TEX",
    "pub const EVERY_OUTPUT",
    "pub const ZERO_KEPT",
    "pub const SAME_ROOTS",
    "const ROOT_WALK_SQUASH",
    "const LEADING",
    "const TURN_STEPS",
    "const OVER_FIELD",
    "const OVER_OUTPUT",
    "const SAME_ROOTS_AT",
    "const ZERO_NAME_AT",
    "const ZERO_PLATE",
    "fn turned_by",
    "fn turned_curve",
]
REACH = "the_reach_disk_never_crosses_its_name"
TURNS = "the_reach_labels_take_turns_over_the_window"
SENTENCES = "the_roots_argument_waits_on_the_right_sentences"
EDGES = "the_division_keeps_the_count_and_a_constant_makes_no_turns"
A_NAME = "the_samples_name_keeps_off_the_distance_line"
EXAMPLE = "the_example_is_the_quintic_times_its_leading_coefficient"
TURN = "the_turn_runs_from_one_to_the_coefficient_and_never_reaches_zero"
STEPS = "every_step_of_the_turn_keeps_the_roots_and_the_turns"
CURVE = "the_turning_curve_starts_from_the_walks_and_keeps_off_zero"
TAKEOVER = "the_turning_field_takes_over_unseen_from_the_walks_own_pieces"
WORDS = "the_division_keeps_to_its_words_and_leaves_the_marks_alone"
LABELS = "the_division_labels_leave_the_field_and_the_marks_clear"
ZERO = "the_zero_keeps_its_name_while_the_line_and_the_rim_go_by"
TESTS = [REACH, TURNS, SENTENCES, EDGES, A_NAME, EXAMPLE, TURN, STEPS, CURVE, TAKEOVER, WORDS, LABELS, ZERO]

AT = "const REACH_NAMES_AT: (f64, f64) = (0.0, 2.2);"
NAME = "let reach_name = kit.text(stage, STEP_SKETCH[2], 18.0, derivative_teal(), names_at)?;"
BY = "C::polar(LEADING.abs().powf(t), LEADING.arg() * t)"
A_CONST = "const A_NAME_ON_SAMPLE: C = C::new(0.72, 0.2);"
# (what is put back, staged text, defect, test that must fail)
MUTANTS = [
    # reach label, as in the frozen folder
    ("the label back where the disk crossed it", AT, "const REACH_NAMES_AT: (f64, f64) = (0.95, -1.2);", REACH),
    ("the label in the window's emptiest corner", AT, "const REACH_NAMES_AT: (f64, f64) = (1.14, 1.87);", REACH),
    ("the label on the window's top edge", AT, "const REACH_NAMES_AT: (f64, f64) = (0.0, 2.05);", REACH),
    ("the label under the title", AT, "const REACH_NAMES_AT: (f64, f64) = (-1.6, 2.2);", REACH),
    ("the label past the window's right edge", AT, "const REACH_NAMES_AT: (f64, f64) = (1.3, 2.2);", REACH),
    ("the title moved onto the label", "const COARSE_TITLE_AT: (f64, f64) = (-4.0, 2.4);", "const COARSE_TITLE_AT: (f64, f64) = (-3.0, 2.4);", REACH),
    ("the name built at its old place in the scene", NAME, "let reach_name = kit.text(stage, STEP_SKETCH[2], 18.0, derivative_teal(), out.p(C::new(0.95, -1.2)))?;", TURNS),
    ("covers not cleared before the name comes", "fade_out_all(stage, &[covered, covers, refused, band], 0.3)?;", "fade_out_all(stage, &[covered, refused, band], 0.3)?;", TURNS),
    ("the name carried by the sweep", "            a.move_to(disk, out.p(c), ORIGIN);\n", "            a.move_to(disk, out.p(c), ORIGIN);\n            a.move_to(reach_name, out.p(c), ORIGIN);\n", TURNS),
    # the sample's name, as in the frozen folder
    ("the sample's name on the line from 0", A_CONST, "const A_NAME_ON_SAMPLE: C = C::new(0.72, 0.0);", A_NAME),
    ("the sample's name on the output circle", A_CONST, "const A_NAME_ON_SAMPLE: C = C::new(0.95, 0.2);", A_NAME),
    ("the sample's name starting somewhere else", A_CONST, "const A_NAME_ON_SAMPLE: C = C::new(0.72, -0.2);", A_NAME),
    ("the sample's name outside the disk", A_CONST, "const A_NAME_ON_SAMPLE: C = C::new(0.1, 0.3);", A_NAME),
    # the argument's sentences
    ("degree_n one sentence early", "        line.wait_sentence(stage, 2)?;\n        fade_out_all(stage, &[after], 0.3)?;", "        line.wait_sentence(stage, 1)?;\n        fade_out_all(stage, &[after], 0.3)?;", SENTENCES),
    ("the circle one sentence early", "        line.wait_sentence(stage, 3)?;\n        turning.stop(stage);", "        line.wait_sentence(stage, 2)?;\n        turning.stop(stage);", SENTENCES),
    ("the dog one sentence early", "        line.wait_sentence(stage, 4)?;\n        dog.show(stage)?;", "        line.wait_sentence(stage, 3)?;\n        dog.show(stage)?;", SENTENCES),
    ("the walk ending one sentence early", "let rt = (line.sentence(6)? - stage.scene().time()", "let rt = (line.sentence(5)? - stage.scene().time()", SENTENCES),
    ("the old claim on screen", 'pub const ROOTS_CLAIM: &str = "nonconstant polynomials have roots";', 'pub const ROOTS_CLAIM: &str = "every polynomial has roots";', SENTENCES),
    ("the division with no wait for its sentence", "        line.wait_sentence(stage, 1)?;\n        let over_output", "        let over_output", SENTENCES),
    # the example and its arithmetic
    ("a wrong middle term in the example", r'r"f(z) = 2iz^5 - 6iz + 2{i}"', r'r"f(z) = 2iz^5 - 3iz + 2{i}"', EXAMPLE),
    ("a wrong constant term in the example", r'r"f(z) = 2iz^5 - 6iz + 2{i}"', r'r"f(z) = 2iz^5 - 6iz + 3{i}"', EXAMPLE),
    ("the colour's key matching the constant term too", r'r"f(z) = 2iz^5 - 6iz + 2{i}"', r'r"f(z) = 2iz^5 - 6iz + 2i"', EXAMPLE),
    ("the wrong quintic after the division", r'= z^5 - 3z + 1"', r'= z^5 - 3z - 1"', EXAMPLE),
    ("the code dividing by a different number from the rows", "const LEADING: C = C::new(0.0, 2.0);", "const LEADING: C = C::new(2.0, 0.0);", EXAMPLE),
    ("the nonzero condition off the screen", '"every output ÷ 2i  (2i ≠ 0)"', '"every output ÷ 2i"', EXAMPLE),
    ("zero's arithmetic off the screen", '"0 ÷ 2i = 0, and nothing else lands on 0"', '"nothing else lands on 0"', EXAMPLE),
    # the turn
    ("a turn not starting from the quintic", BY, "C::polar(LEADING.abs().powf(t), LEADING.arg() * (t + 0.1))", TURN),
    ("a shrink instead of a growth", BY, "C::polar(LEADING.abs().powf(-t), LEADING.arg() * t)", TURN),
    ("a straight path, not a steady turn", BY, "ONE + (LEADING - ONE).scale(t)", TURN),
    ("visible steps in the colour", "const TURN_STEPS: f64 = 48.0;", "const TURN_STEPS: f64 = 12.0;", TURN),
    ("a factor that reaches 0, killing every count", BY, "C::polar(LEADING.abs().powf(t) - 1.0, LEADING.arg() * t)", STEPS),
    ("outputs shrunk towards 0", BY, "C::polar(LEADING.abs().powf(-t), LEADING.arg() * t)", STEPS),
    # the curve
    ("the curve squashed differently from the walk", "squash(by * w, ROOT_WALK_SQUASH, OUT_R)", "squash(by * w, ROOT_WALK_SQUASH + 1.0, OUT_R)", CURVE),
    ("the curve larger than the plane", "squash(by * w, ROOT_WALK_SQUASH, OUT_R)", "squash(by * w, ROOT_WALK_SQUASH, OUT_R * 2.0)", CURVE),
    ("the curve on another loop", "C::polar(1.0, TAU * j as f64 / WALK_SAMPLES as f64)))\n", "C::polar(1.1, TAU * j as f64 / WALK_SAMPLES as f64)))\n", CURVE),
    # the takeover
    ("the field in plain hue while the painter shows the portrait", "let strips = portrait(stage, Func::Quintic, &inp, 1.0)?;", "let strips = portrait(stage, Func::Quintic, &inp, 0.0)?;", TAKEOVER),
    ("a repaint before anything has moved", "let mut last = 0.0;", "let mut last = f64::NAN;", TAKEOVER),
    ("the wrong piece of the walk taken as its curve", "(one[0], one[2]);", "(one[0], one[3]);", TAKEOVER),
    # the words and the marks
    ("the example on the wrong words", 'line.wait_for(stage, "complex polynomial")?;', 'line.wait_for(stage, "polynomial has")?;', WORDS),
    ("the turn back landing on the wrong words", 'line.until(stage, "which leaves", 1.2)?', 'line.until(stage, "leading coefficient", 1.2)?', WORDS),
    ("the marks cleared with 11_6", "        gone.extend(inside);\n", "        gone.extend(inside);\n        gone.extend(roots.iter().copied());\n", WORDS),
    ("the marks never cleared", "        acted.extend(roots);\n", "", WORDS),
    ("the marks left under the new field", "        for &mark in &roots {\n            stage.add_to_scene(mark)?;\n        }\n", "", WORDS),
    ("the field swapped after the painter has gone", "        painter.leave(stage, 0.4)?;\n        let row =", "        let row =", WORDS),
    ("a third turn", "        fade_out_all(stage, &[before, every_output], 0.2)?;\n", "        stage.play(tween(turning.turned, 1.0, 0.2)?)?;\n        fade_out_all(stage, &[before, every_output], 0.2)?;\n", WORDS),
    # the labels
    ("the row over the field", "const OVER_FIELD: [(f64, f64); 2] = [(-3.45, 3.5), (-3.45, 2.83)];", "const OVER_FIELD: [(f64, f64); 2] = [(-3.45, 3.5), (-3.45, 2.6)];", LABELS),
    ("the claim on the row", "const OVER_FIELD: [(f64, f64); 2] = [(-3.45, 3.5), (-3.45, 2.83)];", "const OVER_FIELD: [(f64, f64); 2] = [(-3.45, 3.2), (-3.45, 2.83)];", LABELS),
    ("the claim off the top", "const OVER_FIELD: [(f64, f64); 2] = [(-3.45, 3.5), (-3.45, 2.83)];", "const OVER_FIELD: [(f64, f64); 2] = [(-3.45, 3.9), (-3.45, 2.83)];", LABELS),
    ("the card on the lowest mark", "const SAME_ROOTS_AT: (f64, f64) = (0.0, -1.72);", "const SAME_ROOTS_AT: (f64, f64) = (0.0, -1.5);", LABELS),
    ("the card off the bottom of the field", "const SAME_ROOTS_AT: (f64, f64) = (0.0, -1.72);", "const SAME_ROOTS_AT: (f64, f64) = (0.0, -1.95);", LABELS),
    ("the card miscounting", '"same five roots"', '"same four roots"', LABELS),
    # the 0
    ("a plate over the axes", "const ZERO_PLATE: f64 = 0.105;", "const ZERO_PLATE: f64 = 0.2;", ZERO),
    ("a plate too small for its digit", "const ZERO_PLATE: f64 = 0.105;", "const ZERO_PLATE: f64 = 0.06;", ZERO),
    ("a name on the axes", "const ZERO_NAME_AT: (f64, f64) = (-0.25, -0.3);", "const ZERO_NAME_AT: (f64, f64) = (-0.1, -0.3);", ZERO),
    ("the name left under the line", "    stage.add_to_scene(zero_name)?;\n", "", ZERO),
    ("the plate over the name", "    stage.add_to_scene(zero_plate)?;\n    stage.add_to_scene(zero_name)?;\n", "    stage.add_to_scene(zero_name)?;\n    stage.add_to_scene(zero_plate)?;\n", ZERO),
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
    contains = balanced(draw, draw.index("pub fn contains(&self, z: C) -> bool"))
    squash = balanced(draw, draw.index("pub fn squash("))
    walk = draw[draw.index("pub const WALK_SAMPLES") :].split("\n", 1)[0]
    return f"""
use crate::model::C;
pub type Vec3 = [f64; 3];
pub fn v(x: f64, y: f64) -> Vec3 {{ [x, y, 0.0] }}
{walk}
{squash}
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
{contains}
}}
"""


def run(text, script, items, tests, label, workdir):
    """Build the harness from `text`; return the failed tests and the count run."""
    folder = os.path.join(workdir, label)
    os.makedirs(folder)
    for name, body in (
        ("counting.rs", text),
        ("draw.rs", read(CRATE + "chapters/draw.rs")),
        ("script.tsv", script),
        ("model.rs", read(CRATE + "model.rs")),
        ("shim.rs", shim()),
    ):
        with open(os.path.join(folder, name), "w", encoding="utf-8") as out:
            out.write(body)
    lifted = "\n".join(item(text, head) for head in items)
    bodies = "\n".join(test_body(text, t) for t in tests)
    with open(os.path.join(folder, "main.rs"), "w", encoding="utf-8") as out:
        out.write(
            "#![allow(dead_code, unused_imports)]\nmod model;\nmod shim;\n"
            'pub(crate) const SCRIPT: &str = include_str!("script.tsv");\n'
            "mod counting {\nuse crate::shim::*;\nuse crate::model::{self, C, Func, ONE, ZERO};\nuse std::f64::consts::{PI, TAU};\n"
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
    workdir = tempfile.mkdtemp(prefix="complex11-teaching-")
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
    caught_count = 0
    for k, (what, old, new, expect) in enumerate(MUTANTS):
        assert old in staged, what
        # the first occurrence is the scene's; a source-read test may quote the same line later
        failed, _ = run(staged.replace(old, new, 1), script, ITEMS, TESTS, f"mutant{k:02d}", workdir)
        caught = failed is not None and expect in failed
        caught_count += 1 if caught else 0
        others = [] if failed is None else [t for t in failed if t != expect]
        note = "no build" if failed is None else ("also " + ", ".join(others) if others else "alone")
        print(f"{'caught' if caught else 'MISSED'} {what} -> {expect} ({note})")
        bad += 0 if caught else 1
    # the script as it was: six sentences, no division to key to
    failed, _ = run(staged, read(ROOT_BEFORE), ITEMS, TESTS, "old-script", workdir)
    caught = failed is not None and SENTENCES in failed and WORDS in failed
    caught_count += 1 if caught else 0
    print(f"{'caught' if caught else 'MISSED'} the old script under the new anchors -> {failed}")
    bad += 0 if caught else 1
    print(f"planted defects caught: {caught_count}")
    print("RESULT " + ("PASS" if bad == 0 else f"FAIL {bad}"))


main()
