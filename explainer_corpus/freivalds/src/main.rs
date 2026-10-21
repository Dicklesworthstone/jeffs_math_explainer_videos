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
        // 01: the cold open, the tiny example, the cost at scale
        ("01_1", "two giant matrices", "F-F1.factors", "2"),
        ("01_2", "For a thousand rows and columns", "F-F2.big.side", "1000"),
        ("01_2", "twenty checks", "F-F5.again", "20"),
        (
            "01_2",
            "a fiftieth of the multiplications",
            "F-F5.big.again_share",
            "1/50",
        ),
        ("01_2", "Right answers always pass", "F-F4.right_answer.passing", "8"),
        (
            "01_2",
            "less than once in a million",
            "F-F5.again.under_one_in_a_million",
            "true",
        ),
        ("01_3", "two matrices", "F-F1.factors", "2"),
        ("01_4", "Row one", "F-F2.spot.row", "1"),
        ("01_4", "column three", "F-F2.spot.column", "3"),
        ("01_4", "zero, plus four, plus three", "F-F2.spot.products", "0,4,3"),
        ("01_4", "Seven.", "F-F2.spot.value", "7"),
        ("01_5", "C has seven there", "F-F2.spot.claimed", "7"),
        ("01_5", "nine entries", "F-F2.entries", "9"),
        ("01_5", "three multiplications each", "F-F2.per_entry", "3"),
        ("01_5", "twenty-seven in all", "F-F2.product_multiplications", "27"),
        ("01_6", "a thousand rows", "F-F2.big.side", "1000"),
        ("01_7", "One billion", "F-F2.big.product_multiplications", "1000000000"),
        ("01_7", "a million entries", "F-F11.big.entries", "1000000"),
        ("01_7", "a thousand multiplications each", "F-F2.big.side", "1000"),
        (
            "01_7",
            "another billion",
            "F-F2.big.product_multiplications",
            "1000000000",
        ),
        // 02: spot checks, at three by three and at a million entries; the
        // sum of every column
        ("02_1", "Pick one at random", "F-F2.spot.entries_checked", "1"),
        ("02_1", "Three multiplications", "F-F2.per_entry", "3"),
        ("02_2", "a single wrong entry", "F-F1.wrong_entries", "1"),
        ("02_2", "one time in nine", "F-F2.entries", "9"),
        ("02_2", "Eight times in nine", "F-F2.spot.miss_chance", "8/9"),
        ("02_3", "a thousand rows", "F-F2.big.side", "1000"),
        ("02_3", "a million entries", "F-F11.big.entries", "1000000"),
        (
            "02_3",
            "almost every time",
            "F-F11.big.one_pick_misses",
            "999999/1000000",
        ),
        ("02_4", "at least half the time", "F-F11.big.catch_chance", "1/2"),
        ("02_5", "Half a million", "F-F11.big.entries_for_half", "500000"),
        ("02_5", "a thousand multiplications each", "F-F2.big.side", "1000"),
        (
            "02_5",
            "five hundred million",
            "F-F11.big.multiplications_for_half",
            "500000000",
        ),
        ("02_5", "half the cost", "F-F11.big.share_of_product", "1/2"),
        ("02_6", "one wrong entry", "F-F1.wrong_entries", "1"),
        ("02_7", "all three columns", "F-F1.side", "3"),
        ("02_9", "nine multiplications", "F-F10.multiplications", "9"),
        ("02_9", "not twenty-seven", "F-F2.product_multiplications", "27"),
        ("02_9", "in row two", "F-F10.disagree_rows", "2"),
        ("02_9", "C's sum says sixteen", "F-F10.c_sums", "15,16,15"),
        ("02_9", "the other side says fifteen", "F-F10.a_b_sums", "15,15,15"),
        ("02_10", "the two in the middle", "F-F1.wrong.claimed", "2"),
        ("02_10", "should be one", "F-F1.wrong.true", "1"),
        // 03: the cancelling claim fools the sum; coins choose; one toss
        ("03_1", "the nine beside the slip", "F-F10.cancelling.true", "9"),
        ("03_1", "written as eight", "F-F10.cancelling.claimed", "8"),
        ("03_1", "adds up to fifteen", "F-F10.cancelling.c_sums", "15,15,15"),
        (
            "03_2",
            "The extra one and the missing one",
            "F-F7.cancelling.wrong_values",
            "1,-1",
        ),
        (
            "03_2",
            "this check passes a wrong answer",
            "F-F10.cancelling.passes_all_heads",
            "true",
        ),
        ("03_3", "With three columns", "F-F1.side", "3"),
        ("03_3", "every fixed choice of columns", "F-X3.known_toss.fooled", "8"),
        ("03_4", "the C with one slip", "F-F1.wrong_entries", "1"),
        ("03_5", "heads, heads, tails", "F-F3.toss", "110"),
        ("03_5", "still nine multiplications", "F-F3.multiplications", "9"),
        ("03_6", "No.", "F-F3.passes", "false"),
        ("03_6", "in row two", "F-F3.disagree_rows", "2"),
        ("03_6", "C says seven", "F-F3.cr", "8,7,11"),
        ("03_6", "the other side says six", "F-F3.a_br", "8,6,11"),
        // a caught row is a wrong row: no exception over every small mistake
        ("03_7", "row two holds a wrong entry", "F-X6.violations", "0"),
        ("03_7", "row two", "F-F3.disagree_rows", "2"),
        ("03_7", "nine multiplications", "F-F3.multiplications", "9"),
        // 04: all eight tosses; the mistake; fair and unfair coins
        ("04_1", "one toss", "F-F3.tried", "1"),
        ("04_1", "the eight ways", "F-F4.tosses", "8"),
        ("04_1", "three coins", "F-F4.coins", "3"),
        ("04_2", "Check all eight", "F-F4.tosses", "8"),
        ("04_2", "Four of them miss", "F-F4.misses", "4"),
        ("04_2", "exactly half", "F-F4.miss_fraction", "1/2"),
        ("04_3", "the middle column", "F-F1.wrong.column", "2"),
        (
            "04_3",
            "Every miss leaves out",
            "F-F4.misses_are_tails_on_wrong_column",
            "true",
        ),
        ("04_3", "the slip's extra one", "F-F6.slip.mistake", "0,0,0;0,1,0;0,0,0"),
        ("04_3", "lands in row two", "F-F1.wrong.row", "2"),
        (
            "04_4",
            "C minus the true product",
            "F-F6.slip.mistake",
            "0,0,0;0,1,0;0,0,0",
        ),
        ("04_4", "a single one", "F-F7.one_entry.wrong_values", "1"),
        (
            "04_5",
            "always differ by the mistake's heads columns",
            "F-F6.difference_is_mistake_sum",
            "true",
        ),
        (
            "04_5",
            "add up to zero in every row",
            "F-F6.misses_are_zero_sums",
            "true",
        ),
        ("04_6", "Four of eight", "F-F4.misses", "4"),
        ("04_6", "of eight", "F-F4.tosses", "8"),
        ("04_6", "a chance of one half", "F-F12.fair_slip_passes", "1/2"),
        ("04_7", "one time in ten", "F-F12.heads", "1/10"),
        ("04_8", "Nine times in ten", "F-F12.slip_passes", "9/10"),
        ("04_8", "Our half guarantee", "F-F5.bound_per_toss", "1/2"),
        // 05: mistakes of other shapes, and the finite check
        ("05_1", "Is half special", "F-F4.miss_fraction", "1/2"),
        ("05_2", "Two in one row", "F-F7.two_in_a_row.wrong_count", "2"),
        ("05_2", "both one too big", "F-F7.two_in_a_row.wrong_values", "1,1"),
        ("05_2", "Two of eight", "F-X2.two_in_a_row.misses", "2"),
        ("05_2", "of eight", "F-F4.tosses", "8"),
        (
            "05_3",
            "one too big and one too small by the same amount",
            "F-F7.cancelling.wrong_values",
            "1,-1",
        ),
        ("05_3", "Four of eight", "F-X2.cancelling.misses", "4"),
        ("05_3", "half again", "F-F7.cancelling.miss_share", "1/2"),
        (
            "05_4",
            "the pair that fooled the sum of every column",
            "F-F10.cancelling.passes_all_heads",
            "true",
        ),
        (
            "05_5",
            "one, one and minus one",
            "F-F7.three_in_a_row.wrong_values",
            "1,1,-1",
        ),
        ("05_6", "Three", "F-X2.three_in_a_row.misses", "3"),
        ("05_6", "Not half", "F-F7.three_in_a_row.miss_share", "3/8"),
        (
            "05_6",
            "not a power of two",
            "F-F7.three_in_a_row.power_of_two",
            "false",
        ),
        (
            "05_7",
            "one wrong entry in each row and column",
            "F-F7.every_column.wrong_per_row_and_column",
            "1",
        ),
        ("05_7", "only from all tails", "F-F7.every_column.hides_from", "000"),
        ("05_7", "One of eight", "F-X2.every_column.misses", "1"),
        // the cancelling pair and the slip hide from as many tosses
        ("05_8", "two wrong entries", "F-F7.cancelling.wrong_count", "2"),
        ("05_8", "can hide as often", "F-X2.cancelling.misses", "4"),
        ("05_8", "as often as one", "F-X2.one_entry.misses", "4"),
        ("05_9", "three by three", "F-F1.side", "3"),
        ("05_9", "minus one, zero or one", "F-X1.n3.values", "-1,0,1"),
        (
            "05_9",
            "nineteen thousand, six hundred eighty-two",
            "F-X1.n3.wrong",
            "19682",
        ),
        ("05_9", "at most half the tosses", "F-X1.n3.most_share", "1/2"),
        // 06: the pairing proof, with a nonzero entry of the mistake
        ("06_1", "more than half", "F-F5.bound_per_toss", "1/2"),
        (
            "06_1",
            "one, one, minus one",
            "F-F7.three_in_a_row.wrong_values",
            "1,1,-1",
        ),
        ("06_1", "row two", "F-F8.row", "2"),
        ("06_1", "column one", "F-F8.column", "1"),
        ("06_2", "three coins", "F-F4.coins", "3"),
        ("06_2", "four pairs", "F-F8.pair_count", "4"),
        ("06_3", "say tails, then heads", "F-F14.others", "01"),
        ("06_3", "row two's", "F-F8.row", "2"),
        ("06_3", "add up to minus one", "F-F14.row_sums", "-1,0"),
        ("06_4", "row two's sum", "F-F8.row", "2"),
        ("06_5", "the mistake's entry there", "F-F8.wrong_entry", "1"),
        ("06_5", "one, exactly", "F-F14.change", "1"),
        ("06_5", "Minus one becomes zero", "F-F14.row_sums", "-1,0"),
        (
            "06_6",
            "row two's sums differ by the chosen entry",
            "F-F8.changes",
            "1,1,1,1",
        ),
        // the proof's constant, not the example's most
        (
            "06_6",
            "At most one toss of each pair",
            "F-F8.misses_per_pair_bound",
            "1",
        ),
        ("06_7", "by the first coin", "F-X5.right_column.column", "1"),
        (
            "06_7",
            "its row sum never moves",
            "F-X5.right_column.changes",
            "0,0,0,0",
        ),
        (
            "06_7",
            "Two of those pairs miss twice",
            "F-X5.right_column.misses_per_pair",
            "2,2,0,0",
        ),
        ("06_8", "every toss exactly once", "F-F8.partners_per_toss", "1"),
        ("06_8", "at most half of all tosses", "F-F8.pairs_over_tosses", "1/2"),
        // 07: two fresh tosses, reuse, the ladder, one way
        ("07_1", "One toss", "F-F3.tried", "1"),
        ("07_1", "misses the slip half the time", "F-F4.miss_fraction", "1/2"),
        ("07_2", "Two tosses", "F-F9.tosses", "2"),
        ("07_2", "sixty-four ways", "F-F5.two.pairs", "64"),
        ("07_3", "Sixteen", "F-F5.two.both_miss", "16"),
        ("07_3", "exactly a quarter", "F-F5.two.fraction", "1/4"),
        ("07_3", "four ways for the first toss", "F-F9.first.misses", "4"),
        ("07_3", "times four for the second", "F-F9.second.misses", "4"),
        // this slip's quarter, and the theorem's, kept apart
        (
            "07_4",
            "For this slip, that quarter is exact",
            "F-F9.two.example_meets_bound",
            "true",
        ),
        ("07_4", "two fresh tosses", "F-F9.tosses", "2"),
        ("07_4", "at most a quarter", "F-F9.two.bound", "1/4"),
        ("07_5", "reuse one toss", "F-F3.tried", "1"),
        ("07_5", "Four of the eight", "F-F5.same_toss_twice.misses", "4"),
        ("07_5", "of the eight", "F-F9.same_toss.ways", "8"),
        ("07_5", "still miss, half", "F-F5.same_toss_twice.fraction", "1/2"),
        ("07_6", "at least halves the chance", "F-F5.bound_per_toss", "1/2"),
        ("07_6", "Twenty fresh tosses", "F-F5.again", "20"),
        (
            "07_6",
            "less than once in a million",
            "F-F5.again.under_one_in_a_million",
            "true",
        ),
        ("07_8", "a right C matches every toss", "F-F4.right_answer.passing", "8"),
        // 08: a toss seen first
        ("08_1", "can write a wrong C it misses", "F-X3.known_toss.fooled", "8"),
        ("08_2", "heads, heads, tails", "F-X7.toss", "110"),
        ("08_3", "the third column", "F-X7.tails_column", "3"),
        ("08_3", "Anywhere in the third column", "F-X7.tails_column.hiding", "6"),
        // equal and opposite: what cancelling needs
        (
            "08_4",
            "one entry too big and one too small by the same amount",
            "F-X3.blind_spot.all_heads.values",
            "1,-1",
        ),
        ("08_4", "in one row", "F-X3.blind_spot.all_heads.one_row", "true"),
        ("08_5", "every one of the eight tosses", "F-X3.known_toss.fooled", "8"),
        ("08_5", "of the eight tosses", "F-X3.known_toss.tosses", "8"),
        (
            "08_5",
            "caught half the time",
            "F-X3.blind_spot.fresh_catch_share",
            "1/2",
        ),
        ("08_6", "any one toss", "F-X3.known_toss.fooled", "8"),
        // 09: the cost, counted in multiplications
        ("09_1", "At a thousand rows", "F-F2.big.side", "1000"),
        (
            "09_1",
            "a million multiplications",
            "F-F2.big.check_multiplications",
            "1000000",
        ),
        ("09_1", "a thousandth", "F-F13.big.one_check_share", "1/1000"),
        (
            "09_1",
            "the billion for the product",
            "F-F2.big.product_multiplications",
            "1000000000",
        ),
        ("09_2", "twenty fresh checks", "F-F5.again", "20"),
        ("09_2", "twenty million", "F-F5.big.again_multiplications", "20000000"),
        ("09_2", "a fiftieth", "F-F5.big.again_share", "1/50"),
        ("09_4", "three by three", "F-F1.side", "3"),
        ("09_5", "Three.", "F-F13.small.break_even", "3"),
        ("09_5", "Nine multiplications each", "F-F2.check_multiplications", "9"),
        ("09_5", "twenty-seven in all", "F-F2.product_multiplications", "27"),
        ("09_7", "twenty times in all", "F-F5.again", "20"),
        // 10: N + 1 in doubles, computed, and the two rounded examples
        ("10_2", "ten to the sixteen", "F-X4.big", "10000000000000000"),
        (
            "10_2",
            "ten to the sixteen plus one",
            "F-X4.plus_one.exact",
            "10000000000000001",
        ),
        (
            "10_3",
            "Ten to the sixteen",
            "F-X4.plus_one.double",
            "10000000000000000",
        ),
        ("10_3", "two apart", "F-X4.gap", "2"),
        ("10_3", "the one is lost", "F-X4.lost", "1"),
        ("10_4", "This C is A times B", "F-X4.reject.right_answer", "true"),
        ("10_4", "two lost ones", "F-X4.reject.lost", "2"),
        ("10_5", "too small by one", "F-X4.pass.mistake", "-1"),
        ("10_5", "all four tosses miss it", "F-X4.pass.rounded_passes", "4"),
        ("10_5", "all four tosses", "F-X4.tosses", "4"),
        // the tolerance is the right C's largest gap in doubles, so it lets
        // the right one through on every toss; the wrong C then passes all
        (
            "10_6",
            "would let the right one through",
            "F-X4.tolerance.right_passes",
            "4",
        ),
        ("10_6", "passes every toss", "F-X4.tolerance.wrong_passes", "4"),
        ("10_6", "no half guarantee", "F-F5.bound_per_toss", "1/2"),
        // 11: the promise, back to the start, the billion, why
        (
            "11_1",
            "Right answers match every toss",
            "F-F4.right_answer.passing",
            "8",
        ),
        ("11_1", "at most half the time", "F-F5.bound_per_toss", "1/2"),
        (
            "11_3",
            "twenty-seven multiplications",
            "F-F2.product_multiplications",
            "27",
        ),
        ("11_3", "One toss took nine", "F-F3.multiplications", "9"),
        ("11_3", "in row two", "F-F3.disagree_rows", "2"),
        ("11_4", "At a thousand rows", "F-F2.big.side", "1000"),
        ("11_4", "twenty fresh checks", "F-F5.again", "20"),
        ("11_4", "a fiftieth", "F-F5.big.again_share", "1/50"),
        (
            "11_4",
            "less than once in a million",
            "F-F5.again.under_one_in_a_million",
            "true",
        ),
        ("11_5", "one in a billion", "F-F13.target", "1000000000"),
        ("11_6", "Thirty.", "F-F13.target.checks", "30"),
        ("11_6", "Two to the thirty", "F-F13.target.odds", "1073741824"),
        ("11_6", "just over a billion", "F-F13.target.just_over", "true"),
        ("11_6", "at a thousand rows", "F-F2.big.side", "1000"),
        ("11_6", "three hundredths", "F-F13.target.share", "3/100"),
        (
            "11_7",
            "add up to zero in every row",
            "F-F6.misses_are_zero_sums",
            "true",
        ),
        ("11_7", "breaks that", "F-F15.flipped_caught", "true"),
    ];

    /// The words the guard below listens for: a line holding any of them
    /// must have a row in [`SPOKEN`].
    const NUMBER_WORDS: &[&str] = &[
        "one",
        "single",
        "two",
        "three",
        "four",
        "five",
        "six",
        "seven",
        "eight",
        "nine",
        "ten",
        "fifteen",
        "sixteen",
        "nineteen",
        "twenty",
        "thirty",
        "sixty",
        "eighty",
        "hundred",
        "hundredths",
        "thousand",
        "thousandth",
        "million",
        "billion",
        "half",
        "halves",
        "once",
        "minus",
        "zero",
        "quarter",
        "fiftieth",
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
        for (id, text) in script.lines() {
            let lower = text.to_lowercase();
            let says_a_number = lower
                .split(|c: char| !c.is_alphabetic())
                .any(|w| NUMBER_WORDS.contains(&w));
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
    /// bound does not cover. Chapter 10 names the boundary as doubles and
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

    /// The proof's step names a nonzero entry of the mistake, never "a wrong
    /// entry isn't zero" (a wrong entry of C could itself be 0), and the
    /// claim about fixed checks keeps its size (with one column, heads
    /// checks the product exactly).
    #[test]
    fn the_proof_and_the_fixed_check_keep_their_scope() {
        let script = Script::parse(SCRIPT).unwrap();
        for (id, text) in script.lines() {
            let lower = text.to_lowercase();
            assert!(!lower.contains("wrong entry isn't zero"), "{id}: {text}");
            assert!(!lower.contains("wrong entry is not zero"), "{id}: {text}");
            if lower.contains("every fixed choice") {
                assert!(lower.contains("with three columns"), "{id}: {text}");
            }
        }
        let proof = script.text("06_1").unwrap();
        assert!(proof.contains("where the mistake isn't zero"), "{proof}");
        let step = script.text("06_6").unwrap();
        assert!(step.contains("the chosen entry of the mistake"), "{step}");
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
