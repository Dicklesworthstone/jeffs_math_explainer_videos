//! Epsilon and Delta: Why Getting Closer Wasn't Enough. Unbroken curves x^n
//! on [0, 1] whose limit jumps; a fixed point whose gap shrinks, and a
//! moving point whose gap stays one half; then the guarantee epsilon and
//! delta make precise, two constructive notes on choosing delta, and the
//! neighbourhood picture behind topology. Rendered end to end by
//! franken_manim's native Rust front door and the corpus's shared kit.
//!
//! ```text
//! epsdelta facts                      # every number the video shows or says (key<TAB>value)
//! epsdelta script                     # the narration (generated from docs/storyboard.md)
//! epsdelta render all|<chapter>... [--res 960x540] [--scratch-voice] [--out DIR] ...
//! epsdelta help                       # the full option list
//! ```

mod chapters;
mod facts;
mod model;
mod score;

use explainer_kit::Explainer;

/// The narration, generated from the storyboard by `tools/storyboard.py`.
pub(crate) const SCRIPT: &str = include_str!("../script.tsv");
/// The storyboard: words, pictures, holds and listen windows per beat. Only
/// the tests read it; the binary reads `script.tsv`.
#[cfg(test)]
pub(crate) const STORYBOARD: &str = include_str!("../docs/storyboard.md");

const EXPLAINER: Explainer = Explainer {
    crate_name: "epsilon_delta",
    script: SCRIPT,
    chapters: chapters::registry,
    facts: facts::facts,
    glyph_text: chapters::glyphs::TEXT,
    glyph_tex: chapters::glyphs::TEX,
    glyph_mono: chapters::glyphs::MONO,
    identity: score::identity,
    narration_gain_db: explainer_kit::narration::GAIN_DB,
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

    fn all_text() -> String {
        let script = Script::parse(SCRIPT).unwrap();
        script
            .lines()
            .iter()
            .map(|(_, t)| t.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Narration truth: every number the voice says, with the fact it speaks
    /// for and the value those words denote. A changed number, word or fact
    /// breaks this test before it reaches a listener.
    const SPOKEN: &[(&str, &str, &str, &str)] = &[
        ("01_1", "from zero", "C1.domain.left", "0"),
        ("01_1", "to one.", "C1.domain.right", "1"),
        ("01_1", "Square every height", "C1.squared", "2"),
        ("01_2", "from zero", "C1.domain.left", "0"),
        ("01_2", "up to one.", "C1.domain.right", "1"),
        ("01_3", "x equals nine tenths", "C1.fixed.x", "0.9"),
        ("01_3", "point nine,", "C1.fixed.n1", "0.9"),
        ("01_3", "point eight one,", "C1.fixed.n2", "0.81"),
        ("01_3", "point seven two nine,", "C1.fixed.n3", "0.729"),
        ("01_3", "toward zero.", "C1.limit.below_one", "0"),
        ("01_4", "At x equals one,", "C1.domain.right", "1"),
        ("01_4", "One to any power", "C1.domain.right", "1"),
        ("01_4", "power is one.", "C1.endpoint.height", "1"),
        ("01_6", "Flat at zero", "C1.limit.below_one", "0"),
        ("01_6", "all the way to one,", "C1.domain.right", "1"),
        ("01_6", "at height one.", "C1.limit.at_one", "1"),
        ("01_7", "Below one,", "C1.domain.right", "1"),
        ("01_7", "less than one.", "C1.domain.right", "1"),
        ("01_9", "half height", "C1.half", "1/2"),
        ("01_10", "at one half and", "C1.half", "1/2"),
        ("01_10", "the limit is zero.", "C1.limit.below_one", "0"),
        ("01_10", "toward one,", "C1.domain.right", "1"),
        ("01_10", "gap stays one half.", "C1.witness.gap", "1/2"),
        ("01_11", "shrinks to zero.", "C1.limit.below_one", "0"),
        ("02_1", "above one half.", "C1.half", "1/2"),
        ("02_1", "up to one,", "C1.domain.right", "1"),
        ("02_1", "x equals one belongs", "C1.domain.right", "1"),
        ("02_2", "at one,", "C1.domain.right", "1"),
        ("02_2", "at height one.", "C1.limit.at_one", "1"),
        ("02_2", "The gap there is zero.", "C2.gap_at_one", "0"),
        ("02_3", "at least one half", "C1.half", "1/2"),
        ("02_3", "short of one.", "C1.domain.right", "1"),
        ("02_6", "ten times,", "C2.zoom.0", "10"),
        ("02_6", "a hundred times,", "C2.zoom.1", "100"),
        ("02_6", "a thousand times.", "C2.zoom.2", "1000"),
        ("02_7", "x to the tenth.", "C2.restart_power", "10"),
        ("02_7", "power tenfold", "C2.zoom.step", "10"),
        ("02_7", "stretch tenfold", "C2.zoom.step", "10"),
        ("02_7", "point six nine", "C2.ln2.2dp", "0.69"),
        ("02_8", "gap of one half.", "C1.witness.gap", "1/2"),
    ];

    /// The words that carry a number in this script.
    const NUMBER_WORDS: &[&str] = &[
        "zero", "one", "half", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "tenth",
        "tenths", "tenfold", "square", "fifty", "hundred", "thousand",
    ];

    /// Number words used as plain language, not as a quantity a fact holds.
    const PLAIN: &[(&str, &str)] = &[("01_3", "Pin one point")];

    /// Each word of `text` and the byte it starts at.
    fn words(text: &str) -> Vec<(usize, &str)> {
        let mut out = Vec::new();
        let mut start = None;
        for (i, c) in text.char_indices().chain(std::iter::once((text.len(), ' '))) {
            match (c.is_alphabetic(), start) {
                (true, None) => start = Some(i),
                (false, Some(s)) => {
                    out.push((s, &text[s..i]));
                    start = None;
                }
                _ => {}
            }
        }
        out
    }

    #[test]
    fn every_spoken_number_is_the_fact_it_speaks_for() {
        let script = Script::parse(SCRIPT).unwrap();
        let f = facts::facts().unwrap();
        for (id, words, key, value) in SPOKEN {
            let line = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(line.contains(words), "{id} no longer says {words:?}");
            assert_eq!(f.get(key), Some(*value), "{id} says {words:?} for {key}");
        }
        // every number word of every line sits inside a phrase the table checks
        for (id, text) in script.lines() {
            let lower = text.to_lowercase();
            let phrases = SPOKEN.iter().map(|&(i, w, ..)| (i, w)).chain(PLAIN.iter().copied());
            let covered: Vec<(usize, usize)> = phrases
                .filter(|(i, _)| i == id)
                .filter_map(|(_, w)| {
                    let w = w.to_lowercase();
                    lower.find(&w).map(|at| (at, at + w.len()))
                })
                .collect();
            for (start, word) in words(&lower) {
                if NUMBER_WORDS.contains(&word) {
                    assert!(
                        covered.iter().any(|&(a, b)| a <= start && start + word.len() <= b),
                        "{id}: {word:?} at {start} has no row in SPOKEN: {text}"
                    );
                }
            }
        }
    }

    /// The opening never claims the picture proves the limit, never calls
    /// the failure one of uniform continuity, and never dates or names an
    /// inventor: the historical attribution is not yet qualified (brief).
    #[test]
    fn the_opening_claims_only_what_it_shows() {
        let all = all_text();
        let lower = all.to_lowercase();
        assert!(all.contains("A picture of a few curves can't prove that. Arithmetic can."));
        for wrong in [
            "uniform continuity",
            "uniformly continuous",
            "the picture proves",
            "as you can see, the limit",
            "cauchy",
            "weierstrass",
            "bolzano",
            "berkeley",
            "newton",
            "leibniz",
            "invented",
            "first to",
        ] {
            assert!(!lower.contains(wrong), "the script says {wrong:?}");
        }
        assert!(
            !all.chars().any(|c| c.is_ascii_digit()),
            "numbers are spelled as spoken"
        );
    }

    /// The fixed point and the moving point are kept apart: the fixed point
    /// is pinned before its heights are read, the moving point's gap is one
    /// half, and the closing line contrasts who picks the point.
    #[test]
    fn fixed_and_moving_are_never_confused() {
        let script = Script::parse(SCRIPT).unwrap();
        let text = |id: &str| script.text(id).unwrap();
        assert!(text("01_3").starts_with("Pin one point, x equals nine tenths."));
        assert!(text("01_9").starts_with("Now let the point move."));
        let close = text("01_11");
        let fix = close.find("Fix it first").unwrap();
        let shrinks = close.find("shrinks to zero").unwrap();
        let moves = close.find("Let it move with the curve").unwrap();
        assert!(fix < shrinks && shrinks < moves && close.ends_with("the gap never does."));
    }

    #[test]
    fn the_storyboard_the_registry_and_the_script_agree() {
        explainer_kit::storyboard::check_video(STORYBOARD, SCRIPT, &keys()).unwrap();
        let script = Script::parse(SCRIPT).unwrap();
        check_script_owners(&script, &keys(), None).unwrap();
    }
}
