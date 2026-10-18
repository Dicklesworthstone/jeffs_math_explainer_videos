//! Two Bad Echoes Can Make One Perfect Measurement: Golay's complementary
//! pair, its exact cancellation, and the shared scene it needs, rendered end
//! to end by franken_manim's native Rust front door and the corpus's shared
//! kit.
//!
//! ```text
//! golay facts                         # every number the video shows or says (key<TAB>value)
//! golay script                        # the narration (generated from docs/storyboard.md)
//! golay render all|<chapter>... [--res 960x540] [--scratch-voice] [--out DIR] ...
//! golay help                          # the full option list
//! ```

mod chapters;
mod facts;
mod model;
mod palette;
mod score;

use explainer_kit::Explainer;

/// The narration, generated from the storyboard by `tools/storyboard.py`.
pub(crate) const SCRIPT: &str = include_str!("../script.tsv");
/// The storyboard: words, pictures, holds and listen windows per beat.
pub(crate) const STORYBOARD: &str = include_str!("../docs/storyboard.md");

const EXPLAINER: Explainer = Explainer {
    crate_name: "golay_echoes",
    script: SCRIPT,
    chapters: chapters::registry,
    facts: facts::facts,
    glyph_text: chapters::glyphs::TEXT,
    glyph_tex: chapters::glyphs::TEX,
    glyph_mono: chapters::glyphs::MONO,
    identity: score::identity,
    narration_gain_db: explainer_kit::narration::GAIN_DB,
    // the owner's narration direction: authored, thought-following gaps
    line_gaps: explainer_kit::narration::LineGaps::Thought,
    line_owner: None,
    thumbnail: None,
};

fn main() {
    std::process::exit(explainer_kit::cli::run(&EXPLAINER, std::env::args().skip(1)));
}

#[cfg(test)]
mod tests {
    use explainer_kit::Script;
    use explainer_kit::cli::check_script_owners;

    use super::*;

    fn keys() -> Vec<&'static str> {
        chapters::registry().iter().map(|c| c.key).collect()
    }

    /// Narration truth: every number the voice says, with the fact it speaks
    /// for and the value those words denote. A changed pattern or word
    /// breaks this test before it reaches a listener.
    const SPOKEN: &[(&str, &str, &str, &str)] = &[
        ("01_1", "four pulses", "G-F1.pulses", "4"),
        ("01_1", "Up, up, up, down", "G-F1.code_a", "1,1,1,-1"),
        ("01_3", "add a block", "G-F1.key.agree", "1"),
        ("01_3", "take one away", "G-F1.key.disagree", "-1"),
        ("01_4", "all four agree", "G-F2.a.peak", "4"),
        ("01_4", "A score of four", "G-F2.a.peak", "4"),
        ("01_5", "three pairs line up", "G-F2.a.one_off.pairs", "3"),
        ("01_5", "Two agree", "G-F2.a.one_off.agree", "2"),
        ("01_5", "one disagrees", "G-F2.a.one_off.disagree", "1"),
        ("01_5", "A score of one", "G-F2.a.one_off.score", "1"),
        ("01_6", "a quarter the size", "G-F2.ghost_fraction", "1/4"),
        ("01_7", "With more than one pulse", "G-X1.far_shift_sizes", "1"),
        ("01_7", "Up, up, down, up", "G-F1.code_b", "1,1,-1,1"),
        ("01_8", "the same size", "G-F2.b.largest_ghost", "1"),
        ("01_8", "at the same shifts", "G-F2.b.ghost_shifts", "-3,-1,1,3"),
        ("01_9", "twice as tall", "G-F3.peak_ratio", "2"),
        ("01_10", "not a single ghost", "G-F3.sum.largest_ghost", "0"),
        ("01_11", "one step", "G-F4.moved.step", "1"),
        ("01_11", "ghosts return", "G-F4.moved.largest_ghost", "1"),
    ];

    #[test]
    fn every_spoken_number_is_the_fact_it_speaks_for() {
        let script = Script::parse(SCRIPT).unwrap();
        let f = facts::facts().unwrap();
        for (id, words, key, value) in SPOKEN {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(words), "{id} no longer says {words:?}");
            assert_eq!(f.get(key), Some(*value), "{id} says {words:?} for {key}");
        }
        // and no line carries a number word the table above does not know
        let numbers = [
            "one", "two", "three", "four", "quarter", "twice", "second", "half", "eight",
        ];
        for (id, text) in script.lines() {
            let lower = text.to_lowercase();
            let says_a_number = lower.split(|c: char| !c.is_alphabetic()).any(|w| numbers.contains(&w));
            if says_a_number {
                assert!(
                    SPOKEN.iter().any(|(i, ..)| i == id),
                    "{id} says a number with no row in SPOKEN: {text}"
                );
            }
        }
    }

    /// The spoken patterns are the drawn ones, pulse by pulse, and B's ghosts
    /// are at A's shifts.
    #[test]
    fn the_spoken_patterns_are_the_drawn_ones() {
        let pulses = |words: &str| -> Vec<i64> {
            words
                .split(", ")
                .map(|w| if w.eq_ignore_ascii_case("up") { 1 } else { -1 })
                .collect()
        };
        assert_eq!(pulses("Up, up, up, down"), model::CODE_A);
        assert_eq!(pulses("Up, up, down, up"), model::CODE_B);
        let f = facts::facts().unwrap();
        assert_eq!(f.get("G-F2.a.ghost_shifts"), f.get("G-F2.b.ghost_shifts"));
    }

    #[test]
    fn the_storyboard_the_registry_and_the_script_agree() {
        explainer_kit::storyboard::check_video(STORYBOARD, SCRIPT, &keys()).unwrap();
        let script = Script::parse(SCRIPT).unwrap();
        check_script_owners(&script, &keys(), None).unwrap();
    }
}
