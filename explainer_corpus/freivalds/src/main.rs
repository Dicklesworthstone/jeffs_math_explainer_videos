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
            "02_1",
            "C minus the true product",
            "F-F6.slip.mistake",
            "0,0,0;0,1,0;0,0,0",
        ),
        ("02_2", "Two in one row", "F-F7.two_in_a_row.wrong_count", "2"),
        ("02_2", "both one too big", "F-F7.two_in_a_row.wrong_values", "1,1"),
        ("02_2", "Two of eight", "F-X2.two_in_a_row.misses", "2"),
        ("02_2", "of eight", "F-F4.tosses", "8"),
        (
            "02_3",
            "one too big and one too small by the same amount",
            "F-F7.cancelling.wrong_values",
            "1,-1",
        ),
        ("02_3", "Four of eight", "F-X2.cancelling.misses", "4"),
        ("02_3", "half again", "F-F7.cancelling.miss_share", "1/2"),
        (
            "02_4",
            "one, one and minus one",
            "F-F7.three_in_a_row.wrong_values",
            "1,1,-1",
        ),
        ("02_5", "Three", "F-X2.three_in_a_row.misses", "3"),
        ("02_5", "Not half", "F-F7.three_in_a_row.miss_share", "3/8"),
        (
            "02_5",
            "not a power of two",
            "F-F7.three_in_a_row.power_of_two",
            "false",
        ),
        (
            "02_6",
            "one wrong entry in each row and column",
            "F-F7.every_column.wrong_per_row_and_column",
            "1",
        ),
        ("02_6", "only from all tails", "F-F7.every_column.hides_from", "000"),
        ("02_7", "more than half", "F-F5.bound_per_toss", "1/2"),
        ("02_8", "by the wrong entry", "F-F8.changes", "1,1,1,1"),
        // the proof's constant, not the example's most
        ("02_8", "At most one of each pair", "F-F8.misses_per_pair_bound", "1"),
        ("02_9", "every toss once", "F-F8.partners_per_toss", "1"),
        ("02_9", "at most half of all tosses", "F-F8.pairs_over_tosses", "1/2"),
        ("03_1", "One toss", "F-F3.tried", "1"),
        ("03_1", "misses the slip half the time", "F-F4.miss_fraction", "1/2"),
        ("03_2", "Two tosses", "F-F9.tosses", "2"),
        ("03_2", "sixty-four ways", "F-F5.two.pairs", "64"),
        ("03_3", "Sixteen", "F-F5.two.both_miss", "16"),
        ("03_3", "exactly a quarter", "F-F5.two.fraction", "1/4"),
        ("03_3", "four ways for the first toss", "F-F9.first.misses", "4"),
        ("03_3", "times four for the second", "F-F9.second.misses", "4"),
        // this slip's quarter, and the theorem's, kept apart
        (
            "03_4",
            "For this slip, that quarter is exact",
            "F-F9.two.example_meets_bound",
            "true",
        ),
        ("03_4", "two fresh tosses", "F-F9.tosses", "2"),
        ("03_4", "at most a quarter", "F-F9.two.bound", "1/4"),
        ("03_5", "reuse one toss", "F-F3.tried", "1"),
        ("03_5", "Four of the eight", "F-F5.same_toss_twice.misses", "4"),
        ("03_5", "of the eight", "F-F9.same_toss.ways", "8"),
        ("03_5", "still miss, half", "F-F5.same_toss_twice.fraction", "1/2"),
        (
            "03_7",
            "Put one wrong entry",
            "F-X3.blind_spot.tails.wrong_entries",
            "1",
        ),
        // equal and opposite: what cancelling needs
        (
            "03_7",
            "one entry too big and one too small by the same amount",
            "F-X3.blind_spot.all_heads.values",
            "1,-1",
        ),
        ("03_8", "every one of the eight tosses", "F-X3.known_toss.fooled", "8"),
        ("03_8", "of the eight tosses", "F-X3.known_toss.tosses", "8"),
        (
            "03_8",
            "caught half the time",
            "F-X3.blind_spot.fresh_catch_share",
            "1/2",
        ),
        // the cost, counted in multiplications
        ("04_1", "At a thousand rows", "F-F2.big.side", "1000"),
        ("04_1", "twenty fresh checks", "F-F5.again", "20"),
        (
            "04_1",
            "twenty million multiplications",
            "F-F5.big.again_multiplications",
            "20000000",
        ),
        ("04_1", "a fiftieth", "F-F5.big.again_share", "1/50"),
        (
            "04_1",
            "the billion to redo the product",
            "F-F2.big.product_multiplications",
            "1000000000",
        ),
        // N + 1 in doubles, computed
        ("04_2", "ten to the sixteen", "F-X4.big", "10000000000000000"),
        (
            "04_2",
            "ten to the sixteen plus one",
            "F-X4.plus_one.exact",
            "10000000000000001",
        ),
        (
            "04_3",
            "Ten to the sixteen",
            "F-X4.plus_one.double",
            "10000000000000000",
        ),
        ("04_3", "two apart", "F-X4.gap", "2"),
        ("04_3", "the one is lost", "F-X4.lost", "1"),
        // the two rounded examples
        ("04_4", "This C is A times B", "F-X4.reject.right_answer", "true"),
        ("04_4", "two lost ones", "F-X4.reject.lost", "2"),
        ("04_5", "too small by one", "F-X4.pass.mistake", "-1"),
        ("04_5", "all four tosses miss it", "F-X4.pass.rounded_passes", "4"),
        ("04_5", "all four tosses", "F-X4.tosses", "4"),
        // the tolerance is the right C's largest gap in doubles, so it lets
        // the right one through on every toss; the wrong C then passes all
        (
            "04_6",
            "would let the right one through",
            "F-X4.tolerance.right_passes",
            "4",
        ),
        ("04_6", "passes every toss", "F-X4.tolerance.wrong_passes", "4"),
        ("04_6", "no half guarantee", "F-F5.bound_per_toss", "1/2"),
        // the promise
        ("04_7", "A right C matches every toss", "F-F4.right_answer.passing", "8"),
        ("04_7", "at most half the time", "F-F5.bound_per_toss", "1/2"),
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
            "billion", "half", "once", "minus", "sixteen", "sixty", "quarter", "ten", "fiftieth",
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

    /// The voice claims only the exact theorem: no line of the script uses
    /// the words of rounding, decimals or floating point, which the exact
    /// bound does not cover. Chapter 04 names the boundary as doubles and
    /// shows the rounding on screen and in its cards, from the F-X4 facts.
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
