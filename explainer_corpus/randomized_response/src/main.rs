//! How to Learn a Secret About a Crowd Without Learning Each Person's
//! Answer: randomized response, its coin rule, the count it lets a survey
//! recover, and what each answer still gives away, rendered end to end by
//! franken_manim's native Rust front door and the corpus's shared kit.
//!
//! ```text
//! survey facts                         # every number the video shows or says (key<TAB>value)
//! survey script                        # the narration (generated from docs/storyboard.md)
//! survey render all|<chapter>... [--res 960x540] [--scratch-voice] [--out DIR] ...
//! survey help                          # the full option list
//! ```

mod chapters;
mod facts;
mod model;
mod palette;
mod score;

use explainer_kit::Explainer;

/// The narration, generated from the storyboard by `tools/storyboard.py`.
pub(crate) const SCRIPT: &str = include_str!("../script.tsv");
/// The storyboard: words, pictures, holds and listen windows per beat (the
/// tests check that script.tsv and the chapter registry follow it).
#[cfg(test)]
pub(crate) const STORYBOARD: &str = include_str!("../docs/storyboard.md");

const EXPLAINER: Explainer = Explainer {
    crate_name: "randomized_response",
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
    /// for and the value those words denote. A changed count or word breaks
    /// this test before it reaches a listener.
    const SPOKEN: &[(&str, &str, &str, &str)] = &[
        ("01_1", "a class of twenty", "R-F1.class", "20"),
        ("01_2", "Four of them did", "R-F1.copied", "4"),
        ("01_4", "flip a second coin", "R-F2.coins", "2"),
        ("01_5", "half the class tells the truth", "R-F3.truthful_share", "1/2"),
        ("01_5", "Two of our four say yes", "R-F3.truthful.yes", "2"),
        ("01_5", "our four", "R-F1.copied", "4"),
        ("01_6", "The other ten", "R-F3.forced", "10"),
        ("01_6", "the second coin", "R-F2.coins", "2"),
        ("01_6", "Five yes", "R-F3.forced.yes", "5"),
        ("01_6", "five no", "R-F3.forced.no", "5"),
        ("01_7", "Seven yes", "R-F3.yes", "7"),
        ("01_7", "thirteen no", "R-F3.no", "13"),
        ("01_8", "those two numbers", "R-F3.counts_seen", "2"),
        ("01_8", "get back to four", "R-F4.estimate", "4"),
        ("01_9", "the second coin's yeses", "R-F2.coins", "2"),
        ("01_9", "that's five", "R-F4.forced_yes", "5"),
        ("01_9", "a quarter of the class", "R-F4.forced_yes_share", "1/4"),
        ("01_9", "Two are left", "R-F4.left", "2"),
        ("01_10", "Those two", "R-F4.left", "2"),
        ("01_10", "the half who told the truth", "R-F3.truthful_share", "1/2"),
        ("01_10", "double them", "R-F4.factor", "2"),
        ("01_10", "Four.", "R-F4.estimate", "4"),
        ("01_11", "three times as often", "R-F5.ratio.yes", "3"),
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
            "one", "two", "three", "four", "five", "seven", "eight", "ten", "thirteen", "twenty", "half", "quarter",
            "double", "twice", "second", "times",
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

    /// The synthesis input, narration_units.json, was built from this
    /// script: it names the script's hash, and holds every line once, in
    /// order, each unit inside one chapter (`tools/coherent_narration.py`
    /// checks the rest of its contract when it reads the file).
    #[test]
    fn the_narration_units_are_built_from_this_script() {
        let units = include_str!("../narration_units.json");
        let hash = explainer_score::digest::sha256_hex(SCRIPT.as_bytes());
        assert!(
            units.contains(&format!("\"script_sha256\": \"{hash}\"")),
            "narration_units.json is stale: it does not name the script's SHA-256 {hash}"
        );
        let script = Script::parse(SCRIPT).unwrap();
        let mut from = 0;
        for (id, _) in script.lines() {
            let quoted = format!("\"{id}\"");
            assert_eq!(units.matches(&quoted).count(), 1, "{id} is not in exactly one unit");
            let at = units[from..]
                .find(&quoted)
                .unwrap_or_else(|| panic!("{id} is out of script order"));
            from += at + quoted.len();
        }
    }

    #[test]
    fn the_storyboard_the_registry_and_the_script_agree() {
        explainer_kit::storyboard::check_video(STORYBOARD, SCRIPT, &keys()).unwrap();
        let script = Script::parse(SCRIPT).unwrap();
        check_script_owners(&script, &keys(), None).unwrap();
    }
}
