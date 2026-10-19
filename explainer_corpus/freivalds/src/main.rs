//! Check a Giant Matrix Product Without Doing the Product Again: Freivalds'
//! check, which tosses a coin for each column and compares A·(B·r) with
//! C·r, its exact at-most-half bound, and what repeating it buys, rendered
//! end to end by franken_manim's native Rust front door and the corpus's
//! shared kit.
//!
//! ```text
//! freivalds facts                     # every number the video shows or says (key<TAB>value)
//! freivalds script                    # the narration (generated from docs/storyboard.md)
//! freivalds render all|<chapter>... [--res 960x540] [--scratch-voice] [--out DIR] ...
//! freivalds help                      # the full option list
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
    crate_name: "freivalds",
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
    /// for and the value those words denote. A changed matrix, toss or word
    /// breaks this test before it reaches a listener.
    const SPOKEN: &[(&str, &str, &str, &str)] = &[
        ("01_1", "two matrices", "F-F1.factors", "2"),
        ("01_2", "Nine entries", "F-F2.entries", "9"),
        ("01_2", "three multiplications each", "F-F2.per_entry", "3"),
        ("01_2", "twenty-seven in all", "F-F2.product_multiplications", "27"),
        ("01_2", "a thousand rows and columns", "F-F2.big.side", "1000"),
        ("01_2", "a billion", "F-F2.big.product_multiplications", "1000000000"),
        ("01_3", "Checking one entry", "F-F2.spot.entries_checked", "1"),
        ("01_3", "a single wrong entry", "F-F1.wrong_entries", "1"),
        ("01_3", "eight times in nine", "F-F2.spot.miss_chance", "8/9"),
        ("01_5", "nine multiplications", "F-F2.check_multiplications", "9"),
        ("01_5", "not twenty-seven", "F-F2.product_multiplications", "27"),
        ("01_7", "one toss", "F-F3.tried", "1"),
        ("01_7", "the eight ways", "F-F4.tosses", "8"),
        ("01_7", "three coins", "F-F4.coins", "3"),
        ("01_8", "Check all eight", "F-F4.tosses", "8"),
        ("01_8", "Four of them miss", "F-F4.misses", "4"),
        ("01_8", "exactly half", "F-F4.miss_fraction", "1/2"),
        ("01_10", "more than half", "F-F5.bound_per_toss", "1/2"),
        ("01_10", "Twenty fresh tosses", "F-F5.again", "20"),
        (
            "01_10",
            "once in a million",
            "F-F5.again.under_one_in_a_million",
            "true",
        ),
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
            "one", "single", "two", "three", "four", "seven", "eight", "nine", "twenty", "thousand", "million",
            "billion", "half", "once",
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

    /// The opening claims only the exact theorem: no line of the script
    /// talks about rounding, decimals or floating point, which the exact
    /// bound does not cover (the scope edge is F-X4 in the facts).
    #[test]
    fn the_narration_claims_only_exact_arithmetic() {
        let script = Script::parse(SCRIPT).unwrap();
        for (id, text) in script.lines() {
            let lower = text.to_lowercase();
            for word in ["float", "decimal", "round", "precision", "approximate"] {
                assert!(!lower.contains(word), "{id} says {word:?}: {text}");
            }
        }
    }

    /// The synthesis input, narration_units.json, was built from this
    /// script: it names the script's hash, and holds every line once, in
    /// order (`tools/coherent_narration.py` checks the rest).
    #[test]
    fn the_narration_units_are_built_from_this_script() {
        let units = include_str!("../narration_units.json");
        let hash = explainer_score::digest::sha256_hex(SCRIPT.as_bytes());
        assert!(
            units.contains(&format!("\"script_sha256\": \"{hash}\"")),
            "narration_units.json is stale: rebuild its texts and hash from script.tsv"
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
