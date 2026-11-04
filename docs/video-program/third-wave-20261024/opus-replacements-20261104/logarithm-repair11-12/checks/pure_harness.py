"""Lift the repairs' pure helpers and new tests, verbatim, into two scratch files.

Usage, from the repo root: python3 -I -B <this file> OUT_DIR
Then compile each OUT_DIR/pure1N.rs alone with rustc --test (no Cargo, no fmn).

This compiles and runs only what is lifted. It says nothing about the scene code,
which needs fmn. Five things are stand-ins, not the crate's own: Vec3 and v(), a
Script parser that follows narration.rs's format, and fact(), fact_f64() and
fact_list() returning the values printed by facts.rs for the keys these tests use.
model.rs and letter_f are the repo's own bytes.
"""
import sys
from pathlib import Path

HERE = "docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-repair11-12"
CH = HERE + "/explainer_corpus/logarithm_mysteries/src/chapters/"
CORPUS = Path("explainer_corpus/logarithm_mysteries").resolve()
OUT = Path(sys.argv[1])


def scan_end(src, start, stop_at_semicolon):
    """Index just past the item starting at `start`: its closing brace or top-level semicolon."""
    depth, i, in_str = 0, start, False
    while i < len(src):
        c = src[i]
        if in_str:
            if c == "\\":
                i += 1
            elif c == '"':
                in_str = False
        elif c == '"':
            in_str = True
        elif c == "'" and src[i + 2 : i + 3] == "'":
            i += 2  # a one-character literal such as '{' or b'.'
        elif c in "([{":
            depth += 1
        elif c in ")]}":
            depth -= 1
            if depth == 0 and c == "}" and not stop_at_semicolon:
                return i + 1
        elif c == ";" and depth == 0 and stop_at_semicolon:
            return i + 1
        i += 1
    raise SystemExit("unterminated item at " + src[start : start + 60])


def item(src, head, semicolon=False):
    found = src.count(head)
    if found != 1:
        raise SystemExit(f"{head!r} found {found} times")
    start = src.index(head)
    return src[start : scan_end(src, start, semicolon)]


def consts(src, names):
    out = []
    for n in names:
        for head in (f"const {n}:", f"type {n} "):
            if head in src:
                out.append(item(src, head, True))
                break
        else:
            raise SystemExit("no const " + n)
    return "\n".join(out)


def fns(src, names):
    return "\n".join(item(src, f"fn {n}(") for n in names)


def tests(src, names):
    return "\n".join("#[test]\n" + item(src, f"fn {n}()") for n in names)


PRELUDE = f"""#![allow(dead_code, unused_imports)]
#[path = "{CORPUS}/src/model.rs"]
mod model;
use std::collections::HashMap;
use std::f64::consts::{{PI, SQRT_2, TAU}};
use model::{{I2, J, M2}};

// stand-ins: see pure_harness.py
type Vec3 = [f64; 3];
fn v(x: f64, y: f64) -> Vec3 {{
    [x, y, 0.0]
}}
struct Script {{
    lines: Vec<(String, String)>,
}}
impl Script {{
    fn parse(text: &str) -> Result<Self, String> {{
        let lines = text
            .lines()
            .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
            .map(|l| l.split_once('\\t').map(|(a, b)| (a.to_owned(), b.to_owned())).ok_or_else(|| l.to_owned()))
            .collect::<Result<_, _>>()?;
        Ok(Self {{ lines }})
    }}
    fn text(&self, id: &str) -> Option<&str> {{
        self.lines.iter().find(|(k, _)| k == id).map(|(_, t)| t.as_str())
    }}
}}
fn fact_list(key: &str) -> Result<Vec<f64>, String> {{
    assert_eq!(key, "matrix.flip_pair");
    Ok(vec![-2.0, -0.5])
}}
fn fact(key: &str) -> Result<&'static str, String> {{
    match key {{
        "dial.values" => Ok("0.24,2.4,24,240"),
        _ => Err(format!("no stand-in for {{key}}")),
    }}
}}
fn fact_f64(key: &str) -> Result<f64, String> {{
    assert_eq!(key, "dial.mark");
    Ok(0.380)
}}
const SCRIPT: &str = include_str!("{CORPUS}/script.tsv");
"""

DRAW = Path("explainer_corpus/logarithm_mysteries/src/chapters/draw.rs").read_text(encoding="utf-8")
LETTER = item(DRAW, "pub fn letter_f(").replace("crate::model::M2", "M2")

# ---------------------------------------------------------------- chapter 11
S = Path(CH + "unreachable.rs").read_text(encoding="utf-8")
body11 = "\n".join(
    [
        PRELUDE,
        LETTER,
        consts(
            S,
            "CENTRE UNIT CLIP HALFWAY SQUASHES ENDS_DIFFER KEEP_AXES TURNS_EVERY SOME_DIRECTION EVERY_DIRECTION "
            "HALF_TURN_STRETCH BLOCKS MATCHING FLAT_SCALE AXES_REACH DIRECTION_LEN BACK_REACH SWEEP_STEP STRETCH_C "
            "GAP_X ROWS_X TILE_STEP TILE_HALF WIDE_TILE_HALF ROWS_TOP ROWS_PITCH BLOCKS_HEAD_Y VERDICT_X BRACKET_TOP "
            "BRACKET_FOOT Block BLOCK_ROWS CHORD_CORNERS".split()
        ),
        fns(
            S,
            "lerp phrase_seconds run_by chord_midpoints reverses reversed_spans direction paired pairs_up "
            "tile_half tile_x".split(),
        ),
        "#[cfg(test)]\nmod tests {\nuse super::*;\n",
        consts(S, "FLOW PAIR WIDTH_ROOM MIRROR BASE".split()),
        fns(S, "em width on_panel".split()),
        tests(
            S,
            "a_phrase_lands_on_its_mark_or_on_its_share_of_the_measured_sentence "
            "every_phrase_hook_is_in_its_line_once_and_they_come_in_spoken_order "
            "a_run_ends_by_its_moment_within_its_limits "
            "the_average_collapses_the_whole_plane_onto_the_axis_without_turning_back "
            "the_midpoints_of_the_dashes_are_where_the_average_sends_the_corners "
            "the_half_lap_turns_each_axis_away_and_lays_it_back_on_itself_reversed "
            "the_sweep_marks_exactly_the_directions_sent_straight_back "
            "the_sweep_stays_on_the_panel "
            "the_block_rows_follow_the_rule_and_each_verdict_says_what_the_rule_says "
            "a_half_turn_with_an_equal_stretch_has_a_real_logarithm_that_commutes_with_every_x "
            "a_real_spin_with_any_shear_that_reaches_a_negative_stretch_reaches_it_in_every_direction "
            "the_sweep_words_and_block_rows_stay_out_of_the_text_column".split(),
        ),
        "}\n",
    ]
)
(OUT / "pure11.rs").write_text(body11, encoding="utf-8")

# ---------------------------------------------------------------- chapter 12
S = Path(CH + "what_it_forgets.rs").read_text(encoding="utf-8")
body12 = "\n".join(
    [
        PRELUDE,
        consts(S, "PANELS HALF_H DIAL_C DIAL_R MIN_RUN ALSO_ENDS_HERE AND_TWO_AND_A_HALF ARRIVAL_HOLD LAPS_CHIP_ROOM".split()),
        fns(S, "phrase_seconds run_by dial_at".split()),
        "#[cfg(test)]\nmod tests {\nuse super::*;\n",
        consts(S, ["WIDTH_ROOM"]),
        fns(S, "em width".split()),
        tests(
            S,
            "a_phrase_lands_on_its_mark_or_on_its_share_of_the_measured_sentence "
            "the_lap_hooks_are_in_the_second_sentence_once_and_in_spoken_order "
            "a_spin_ends_by_its_moment_within_its_limits "
            "the_one_mark_is_on_the_rim_labelled_and_clear_of_its_neighbours".split(),
        ),
        "}\n",
    ]
)
(OUT / "pure12.rs").write_text(body12, encoding="utf-8")
print("wrote pure11.rs and pure12.rs")
