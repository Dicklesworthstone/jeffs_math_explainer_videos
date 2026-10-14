//! How to Make a Fair Coin Out of an Unfair One — Video E of the corpus:
//! von Neumann's two-toss trick, why it works for every coin, what it costs,
//! where it breaks, and what it grows into (dice, and squeezing out nearly
//! all the fairness a coin holds), rendered end to end by franken_manim's
//! native Rust front door and the corpus's shared kit.
//!
//! ```text
//! coin facts                         # every number the video shows or says (key<TAB>value)
//! coin script                        # the narration (generated from docs/storyboard.md)
//! coin render all|<chapter>... [--res 960x540] [--scratch-voice] [--out DIR] ...
//! coin help                          # the full option list
//! ```
//!
//! Chapters start as animatics (`explainer_kit::animatic`): each speaks its
//! storyboard lines over a card describing the planned picture, so drafts
//! and timelines run before the real scene exists.

mod chapters;
mod facts;
mod model;
mod score;

use explainer_kit::Explainer;

/// The narration, generated from the storyboard by `tools/storyboard.py`.
pub(crate) const SCRIPT: &str = include_str!("../script.tsv");
/// The storyboard: words, pictures, holds and listen windows per beat.
pub(crate) const STORYBOARD: &str = include_str!("../docs/storyboard.md");

const EXPLAINER: Explainer = Explainer {
    crate_name: "fair_coin",
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
    thumbnail: Some(chapters::thumbnail::make),
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
    /// for and the value those words denote. A changed seed, chance or word
    /// breaks this test before it reaches a listener.
    const SPOKEN: &[(&str, &str, &str, &str)] = &[
        ("01_1", "four times in five", "E-F1.heads_percent", "80"),
        ("01_3", "four bets in five", "E-F1.heads_percent", "80"),
        ("02_3", "four fifths of the width", "E-F1.heads_percent", "80"),
        ("02_4", "four fifths of the way up", "E-F1.heads_percent", "80"),
        (
            "03_1",
            "four fifths wide and one fifth tall",
            "E-F1.square.ht.exact",
            "4/25",
        ),
        (
            "03_1",
            "one fifth wide and four fifths tall",
            "E-F1.square.th.exact",
            "4/25",
        ),
        ("02_5", "sixteen of the twenty-five", "E-F1.square.hh.exact", "16/25"),
        ("02_5", "two tails has one", "E-F1.square.tt.exact", "1/25"),
        ("02_6", "Four in twenty-five", "E-F1.square.ht.exact", "4/25"),
        ("02_6", "Four each", "E-F1.square.th.exact", "4/25"),
        ("02_6", "sixteen percent", "E-F1.square.ht_percent", "16"),
        ("02_8", "A thousand games", "E-S2.trick.games", "1000"),
        ("02_8", "four hundred ninety-four", "E-S2.trick.heads_first", "494"),
        ("02_8", "five hundred six", "E-S2.trick.tails_first", "506"),
        ("03_4", "Nine percent each", "E-F4.p90.ht_percent", "9"),
        ("04_1", "thirty-two times in a hundred", "E-F2.decides_percent", "32"),
        ("04_3", "six and a quarter", "E-F3.expected_tosses", "6.25"),
        ("04_3", "about three pairs", "E-F3.expected_pairs", "3.125"),
        ("04_4", "Ten thousand games", "E-S4.price.games", "10000"),
        ("04_4", "six point two one", "E-S4.price.mean_tosses", "6.21"),
        ("04_5", "averages four tosses", "E-F4.p50.expected_tosses.exact", "4"),
        (
            "04_5",
            "about a hundred and one",
            "E-F4.p99.expected_tosses_rounded",
            "101",
        ),
        ("04_6", "sixty tosses", "E-S4.price.longest_tosses", "60"),
        ("05_2", "Forty percent", "E-F5.two_coins.ht_percent", "40"),
        ("05_2", "against ten", "E-F5.two_coins.th_percent", "10"),
        ("05_2", "four games in five", "E-F5.two_coins.heads_first_percent", "80"),
        ("05_3", "looks fair", "E-F6.sticky.long_run_heads.exact", "1/2"),
        ("05_3", "four times in five", "E-F6.sticky.repeat_percent", "80"),
        ("05_4", "four times as likely", "E-F6.sticky.after_heads.ratio", "4"),
        ("05_5", "four times in five", "E-F6.sticky.repeat_percent", "80"),
        (
            "06_4",
            "four fifths, times four fifths, times one fifth",
            "E-F8.three.heads2.each_percent",
            "12.8",
        ),
        ("06_5", "one, two and one", "E-F8.pascal.row2", "1,2,1"),
        ("06_6", "one, three, three, one", "E-F8.pascal.row3", "1,3,3,1"),
        ("06_6", "one, four, six, four, one", "E-F8.pascal.row4", "1,4,6,4,1"),
        ("07_1", "Six patterns", "E-F8.four_two.patterns", "6"),
        ("07_3", "Twenty-four thousand tosses", "E-S7.coin_die.tosses", "24000"),
        ("07_3", "nine hundred forty-nine rolls", "E-S7.coin_die.dice", "949"),
        (
            "07_4",
            "about twenty-six tosses",
            "E-F8.four_two.expected_tosses_rounded",
            "26",
        ),
        ("08_1", "a six half the time", "E-F9.loaded.six_percent", "50"),
        (
            "09_3",
            "A tenth, by a fifth, by a half",
            "E-F9.brick.sides_percent",
            "10,20,50",
        ),
        ("09_3", "six ways", "E-F9.three_rolls.orders", "6"),
        ("09_4", "a fair choice among six", "E-F9.three_rolls.orders", "6"),
        ("09_5", "Six thousand rolls", "E-S8.loaded.rolls", "6000"),
        ("09_5", "six hundred fifty rolls", "E-S8.three_rolls.dice", "650"),
        ("09_6", "twenty-four orders", "E-F9.orders.4", "24"),
        ("10_1", "nineteen forty-nine", "E-C1.von_neumann.talk_year", "1949"),
        (
            "10_1",
            "twenty-five percent",
            "E-C1.von_neumann.efficiency_percent",
            "25",
        ),
        (
            "10_2",
            "fifteen fair ones",
            "E-S9.squeeze.first_hundred.fair_tosses",
            "15",
        ),
        ("10_2", "the average is sixteen", "E-F10.von_neumann.per_hundred", "16"),
        ("10_3", "four times in five", "E-F1.heads_percent", "80"),
        ("10_3", "just over seventy-two", "E-F10.entropy.per_hundred_1dp", "72.2"),
        ("11_2", "nineteen ninety-nine", "E-C4.intel.year", "1999"),
    ];

    #[test]
    fn every_spoken_number_is_the_fact_it_speaks_for() {
        let script = Script::parse(SCRIPT).unwrap();
        let f = facts::facts().unwrap();
        let line = |id: &str| {
            script
                .lines()
                .iter()
                .find(|(i, _)| i == id)
                .map(|(_, text)| text.as_str())
                .ok_or(id)
                .unwrap()
        };
        for (id, words, key, value) in SPOKEN {
            assert!(line(id).contains(words), "{id} no longer says {words:?}");
            assert_eq!(f.get(key), Some(*value), "{id} says {words:?} for {key}");
        }
        // and no line carries a number the table above does not know: every
        // number word in the script sits in a line the table covers, or is
        // one of the small counting words ("two tosses", "three rolls")
        let big = [
            "hundred", "thousand", "percent", "fifths", "quarter", "point", "nineteen", "twenty", "thirty", "forty",
            "fifty", "sixty", "seventy", "eighty", "ninety",
        ];
        for (id, text) in script.lines() {
            if big.iter().any(|w| text.contains(w)) {
                assert!(
                    SPOKEN.iter().any(|(i, ..)| i == id),
                    "{id} says a number with no row in SPOKEN: {text}"
                );
            }
        }
    }

    #[test]
    fn the_storyboard_the_registry_and_the_script_agree() {
        explainer_kit::storyboard::check_video(STORYBOARD, SCRIPT, &keys()).unwrap();
        let script = Script::parse(SCRIPT).unwrap();
        check_script_owners(&script, &keys(), None).unwrap();
    }
}
