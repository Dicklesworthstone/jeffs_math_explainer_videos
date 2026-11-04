"""Compile and run the repair's new tests with rustc alone: no crate, no fmn, no render.

Run from the repo root:
  python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex06-09-repair03/checks/standalone_tests.py

It lifts each new test, and only the pure items that test needs, out of the staged chapter
files and builds them with the film's real model.rs and small shims for Vec3, Frame2 and
Plane, whose bodies are lifted from the real sources. Then it puts each defect back, one at a
time, and requires the named test to fail.

This is a stand-in. It does not type-check `construct`, does not link fmn and is not the
crate's `cargo test`. It writes only to a new temporary directory.
"""
import os
import subprocess
import tempfile

HERE = "docs/video-program/third-wave-20261024/opus-replacements-20261104/complex06-09-repair03/"
STAGED = HERE + "explainer_corpus/complex_analysis_visual/src/chapters/"
CRATE = "explainer_corpus/complex_analysis_visual/src/"
PLOT = "crates/explainer_kit/src/plot.rs"

CHAPTERS = {
    "one_derivative": {
        "items": ["const DOUBLING_RATES_AT", "const CROSS_AT", "const VERDICT_AT"],
        "tests": [
            "the_reflections_numbers_leave_before_doublings_heading",
            "the_right_column_reads_rates_then_cross_then_verdict",
        ],
    },
    "one_point": {
        "items": ["const MAP_ARROW", "pub fn planes"],
        "tests": ["the_map_arrow_starts_clear_of_re_and_stops_short_of_the_right_plane"],
    },
    "loop_sums": {
        "items": [
            "const CIRCULATION:",
            "const CIRCULATION_HEAD",
            "pub const STEPS:",
            "pub const K:",
            "pub const R:",
            "pub const BIG_R",
            "pub fn step",
            "fn inverse",
            "pub fn parts",
            "pub fn circle_plane",
            "pub fn circulation_head",
        ],
        "tests": [
            "a_circulation_mark_points_the_way_it_turns",
            "a_circulation_head_has_one_point_and_sits_on_its_arc",
            "the_position_is_on_the_ring_while_the_voice_undoes_its_turn",
        ],
    },
    "boundary": {
        "items": ["const CENTRE_CARD_AT", "const SHARE_CARD_FROM_POLE", "pub fn picture"],
        "tests": ["the_centre_card_is_clear_of_what_joins_it_on_the_picture"],
    },
}

# (finding, chapter file, staged text, defect put back, test that must fail)
MUTANTS = [
    ("06-N1 r1 and r2 stay beside the heading", "one_derivative", "conj_says, mirror_line, r1, r2];", "conj_says, mirror_line];", "the_reflections_numbers_leave_before_doublings_heading"),
    ("06-N1 first cross stays", "one_derivative", "        reflection.extend(cross1);\n", "", "the_reflections_numbers_leave_before_doublings_heading"),
    ("06-N1 r1 fades twice", "one_derivative", "            s1,\n            moved,\n", "            s1,\n            r1,\n            moved,\n", "the_reflections_numbers_leave_before_doublings_heading"),
    ("06-N1 verdict leaves too", "one_derivative", "mirror_line, r1, r2];", "mirror_line, r1, r2, no_derivative];", "the_reflections_numbers_leave_before_doublings_heading"),
    ("06-N1 heading comes first", "one_derivative", "        fade_out_all(stage, &reflection, 0.3)?;\n        let squash_says = kit.tex(stage, SQUASH_TEX, 30.0, Palette::ink(), &[], v(-2.6, 2.85))?;\n", "        let squash_says = kit.tex(stage, SQUASH_TEX, 30.0, Palette::ink(), &[], v(-2.6, 2.85))?;\n        fade_out_all(stage, &reflection, 0.3)?;\n", "the_reflections_numbers_leave_before_doublings_heading"),
    ("06-N1 rates back below the verdict", "one_derivative", "const DOUBLING_RATES_AT: (f64, f64) = (4.2, 1.3);", "const DOUBLING_RATES_AT: (f64, f64) = (4.2, -1.3);", "the_right_column_reads_rates_then_cross_then_verdict"),
    ("07-N1 tail back in Re", "one_point", "const MAP_ARROW: (f64, f64) = (-0.42, 0.6);", "const MAP_ARROW: (f64, f64) = (-0.6, 0.6);", "the_map_arrow_starts_clear_of_re_and_stops_short_of_the_right_plane"),
    ("08-N1 dot never comes", "loop_sums", "fade_in_all(stage, &[up_says, position, position_name], 0.3)?;", "fade_in_all(stage, &[up_says], 0.3)?;", "the_position_is_on_the_ring_while_the_voice_undoes_its_turn"),
    ("08-N1 dot left behind on the growing ring", "loop_sums", "gone.extend([up_says, position, position_name]);", "gone.push(up_says);", "the_position_is_on_the_ring_while_the_voice_undoes_its_turn"),
    ("08-N1 wrong step", "loop_sums", "pub const K: usize = 3;", "pub const K: usize = 25;", "the_position_is_on_the_ring_while_the_voice_undoes_its_turn"),
    ("08-N2 head as wide as long, the review's one-number fix", "loop_sums", "const CIRCULATION_HEAD: (f64, f64) = (0.16, 0.11);", "const CIRCULATION_HEAD: (f64, f64) = (0.16, 0.16);", "a_circulation_head_has_one_point_and_sits_on_its_arc"),
    ("08-N2 the old back angle, base off the arc", "loop_sums", "let back = 2.0 * (length / (2.0 * r)).min(1.0).asin();", "let back = 0.35;", "a_circulation_head_has_one_point_and_sits_on_its_arc"),
    ("08-N2 head points clockwise", "loop_sums", "point(CIRCULATION.1 - back)", "point(CIRCULATION.1 + back)", "a_circulation_mark_points_the_way_it_turns"),
    ("09-N2 card back under the share card", "boundary", "const CENTRE_CARD_AT: C = C::new(1.05, -0.4);", "const CENTRE_CARD_AT: C = C::new(0.85, -0.72);", "the_centre_card_is_clear_of_what_joins_it_on_the_picture"),
    ("09-N2 card on the small ring", "boundary", "const CENTRE_CARD_AT: C = C::new(1.05, -0.4);", "const CENTRE_CARD_AT: C = C::new(0.75, -0.3);", "the_centre_card_is_clear_of_what_joins_it_on_the_picture"),
    ("09-N2 card on the large ring", "boundary", "const CENTRE_CARD_AT: C = C::new(1.05, -0.4);", "const CENTRE_CARD_AT: C = C::new(1.6, -0.8);", "the_centre_card_is_clear_of_what_joins_it_on_the_picture"),
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
    if head.lstrip().startswith(("const", "pub const")) and text[start:line_end].rstrip().endswith(";"):
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
    contains = balanced(draw, draw.index("pub fn contains(&self, z: C)"))
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
{contains}
}}
"""


def run(sources, label, workdir):
    """Build the harness from `sources`; return the names of the tests that failed."""
    folder = os.path.join(workdir, label)
    os.makedirs(folder)
    harness = ["#![allow(dead_code, unused_imports)]", "mod model;", "mod shim;"]
    for name, spec in CHAPTERS.items():
        text = sources[name]
        with open(os.path.join(folder, name + ".rs"), "w", encoding="utf-8") as out:
            out.write(text)
        items = "\n".join(item(text, head) for head in spec["items"])
        tests = "\n".join(test_body(text, t) for t in spec["tests"])
        harness.append(
            f"mod {name} {{\n"
            "use crate::shim::*;\nuse crate::model::{self, C, ZERO};\nuse std::f64::consts::TAU;\n"
            f"{items}\n#[cfg(test)]\nmod tests {{\nuse super::*;\n{tests}\n}}\n}}"
        )
    with open(os.path.join(folder, "main.rs"), "w", encoding="utf-8") as out:
        out.write("\n".join(harness) + "\nfn main() {}\n")
    with open(os.path.join(folder, "shim.rs"), "w", encoding="utf-8") as out:
        out.write(shim())
    with open(os.path.join(folder, "model.rs"), "w", encoding="utf-8") as out:
        out.write(read(CRATE + "model.rs"))
    binary = os.path.join(folder, "tests.bin")
    built = subprocess.run(
        ["rustc", "--edition", "2024", "--test", os.path.join(folder, "main.rs"), "-o", binary, "--cap-lints", "allow"],
        capture_output=True,
        text=True,
        check=False,
    )
    if built.returncode != 0:
        return None, built.stderr.replace(folder, "<work>")[:3000]
    names = [f"{name}::tests::{t}" for name, spec in CHAPTERS.items() for t in spec["tests"]]
    failed, ran = [], 0
    for test in names:
        done = subprocess.run([binary, "--exact", test], capture_output=True, text=True, check=False)
        ran += 1
        if done.returncode != 0:
            failed.append(test.split("::")[-1])
    return failed, ran


def main():
    workdir = tempfile.mkdtemp(prefix="complex0609-standalone-")
    version = subprocess.run(["rustc", "--version"], capture_output=True, text=True, check=True).stdout.strip()
    print(version)
    staged = {name: read(STAGED + name + ".rs") for name in CHAPTERS}
    bad = 0
    failed, ran = run(staged, "staged", workdir)
    if failed is None:
        print("STAGED DID NOT BUILD\n" + ran)
        print("RESULT FAIL")
        return
    print(f"staged: {ran} tests, failed {failed}")
    bad += len(failed)
    for k, (finding, name, old, new, expect) in enumerate(MUTANTS):
        text = staged[name]
        # the first occurrence is the scene's; a source-read test may quote the same line later
        assert old in text, finding
        failed, ran = run({**staged, name: text.replace(old, new, 1)}, f"mutant{k:02d}", workdir)
        if failed is None:
            print(f"BUILD FAIL  {finding}\n{ran}")
            bad += 1
            continue
        caught = expect in failed
        print(f"{'caught' if caught else 'MISSED'}  {finding}: failed {failed}")
        bad += 0 if caught else 1
    print(f"mutants: {len(MUTANTS)}")
    print("RESULT " + ("PASS" if bad == 0 else f"FAIL {bad}"))


main()
