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
        ("01_7", "Four copiers or none", "R-F1.copied", "4"),
        ("01_7", "ten yeses on average", "R-F17.coin_alone.average_yes", "10"),
        ("02_2", "a second fair coin", "R-F2.coins", "2"),
        ("02_3", "the second coin says yes", "R-F2.coins", "2"),
        ("02_4", "Three yeses in four", "R-F2.yes_outcomes.copied", "3"),
        ("02_4", "yeses in four", "R-F2.coin_outcomes", "4"),
        ("02_4", "of the four give", "R-F2.coin_outcomes", "4"),
        ("02_5", "One.", "R-F2.yes_outcomes.didnt", "1"),
        ("02_6", "three times as often", "R-F5.ratio.yes", "3"),
        ("02_7", "more than three times as likely", "R-F5.worst_ratio", "3"),
        ("03_1", "half the class tells the truth", "R-F3.truthful_share", "1/2"),
        ("03_1", "Two of our four say yes", "R-F3.truthful.yes", "2"),
        ("03_1", "our four", "R-F1.copied", "4"),
        ("03_2", "The other ten", "R-F3.forced", "10"),
        ("03_2", "the second coin", "R-F2.coins", "2"),
        ("03_2", "Five yes", "R-F3.forced.yes", "5"),
        ("03_2", "five no", "R-F3.forced.no", "5"),
        ("03_3", "Seven yes", "R-F3.yes", "7"),
        ("03_3", "thirteen no", "R-F3.no", "13"),
        ("03_4", "those two numbers", "R-F3.counts_seen", "2"),
        ("03_4", "get back to four", "R-F4.estimate", "4"),
        ("03_5", "the second coin's yeses", "R-F2.coins", "2"),
        ("03_5", "that's five", "R-F4.forced_yes", "5"),
        ("03_5", "a quarter of the class", "R-F4.forced_yes_share", "1/4"),
        ("03_5", "Two are left", "R-F4.left", "2"),
        ("03_6", "Those two", "R-F4.left", "2"),
        ("03_6", "the half who told the truth", "R-F3.truthful_share", "1/2"),
        ("03_6", "double them", "R-F4.factor", "2"),
        ("03_6", "Four.", "R-F4.estimate", "4"),
        ("04_2", "half the k copiers", "R-F3.truthful_share", "1/2"),
        ("04_2", "the second coin says yes", "R-F2.coins", "2"),
        ("04_2", "a quarter of the class", "R-F4.forced_yes_share", "1/4"),
        ("04_3", "half of k", "R-F3.truthful_share", "1/2"),
        ("04_3", "a quarter of n", "R-F4.forced_yes_share", "1/4"),
        ("04_3", "two plus", "R-F3.truthful.yes", "2"),
        ("04_3", "plus five", "R-F4.forced_yes", "5"),
        ("04_3", "five, seven.", "R-F17.rule.average_yes", "7"),
        ("04_4", "a quarter of the class", "R-F4.forced_yes_share", "1/4"),
        ("04_4", "double what's left", "R-F4.factor", "2"),
        ("04_6", "A class of forty", "R-F18.class", "40"),
        ("04_6", "eighteen times", "R-F18.yes", "18"),
        ("04_7", "Sixteen.", "R-F18.estimate", "16"),
        ("04_7", "A quarter of forty", "R-F4.forced_yes_share", "1/4"),
        ("04_7", "of forty is ten", "R-F18.forced_yes", "10"),
        ("04_7", "Eighteen take away", "R-F18.yes", "18"),
        ("04_7", "is eight", "R-F18.left", "8"),
        ("04_7", "double eight", "R-F4.factor", "2"),
        ("04_7", "is sixteen", "R-F18.estimate", "16"),
        ("04_8", "The doubling", "R-F4.factor", "2"),
        ("04_8", "The quarter", "R-F4.forced_yes_share", "1/4"),
        ("04_8", "the second coin's yeses", "R-F2.coins", "2"),
        ("05_1", "all four coin outcomes", "R-F2.coin_outcomes", "4"),
        ("05_1", "Twenty students", "R-F1.class", "20"),
        ("05_1", "eighty outcomes", "R-F11.class_outcomes", "80"),
        ("05_2", "three yes outcomes", "R-F2.yes_outcomes.copied", "3"),
        ("05_2", "everyone else one", "R-F2.yes_outcomes.didnt", "1"),
        ("05_4", "The four who copied", "R-F1.copied", "4"),
        ("05_4", "have twelve", "R-F11.yes.copied", "12"),
        ("05_4", "the sixteen others", "R-F11.others", "16"),
        ("05_4", "others sixteen", "R-F11.yes.didnt", "16"),
        ("05_5", "Twelve of", "R-F11.yes.copied", "12"),
        ("05_5", "of twenty-eight", "R-F11.yes", "28"),
        ("05_5", "three of the seven", "R-F11.average.yes.copied", "3"),
        ("05_5", "the seven yeses", "R-F11.average.yes", "7"),
        ("05_5", "Three in seven", "R-F11.posterior.yes", "3/7"),
        ("05_5", "not three in four", "R-F2.p_yes.copied", "3/4"),
        ("05_6", "three times the yeses", "R-F5.ratio.yes", "3"),
        ("05_6", "four to one", "R-F11.outnumber", "4"),
        ("05_7", "Four of the", "R-F11.no.copied", "4"),
        ("05_7", "fifty-two no outcomes", "R-F11.no", "52"),
        ("05_7", "one in thirteen", "R-F11.posterior.no", "1/13"),
        ("05_8", "one in five", "R-F1.copied_share", "1/5"),
        ("05_8", "three in seven", "R-F11.posterior.yes", "3/7"),
        ("05_8", "one in thirteen", "R-F11.posterior.no", "1/13"),
        ("05_9", "the three times", "R-F5.worst_ratio", "3"),
        ("06_1", "eight times", "R-F13.runs", "8"),
        ("06_2", "from minus four", "R-F13.lowest", "-4"),
        ("06_2", "to fourteen", "R-F13.highest", "14"),
        ("06_2", "Only one lands", "R-F13.on_four", "1"),
        ("06_2", "lands on four", "R-F14.estimate_mean", "4"),
        ("06_3", "Four to the twentieth", "R-F2.coin_outcomes", "4"),
        ("06_3", "the twentieth", "R-F1.class", "20"),
        ("06_3", "over a trillion", "R-F14.outcomes_trillions", "1"),
        ("06_4", "around four", "R-F14.estimate_mean", "4"),
        ("06_5", "exactly on four", "R-F1.copied", "4"),
        ("06_6", "one class in five", "R-F14.one_in.exactly", "5"),
        ("06_6", "one in eleven", "R-F14.one_in.negative", "11"),
        ("06_6", "below zero", "R-F14.clamp_at", "0"),
        ("06_7", "Below zero", "R-F14.clamp_at", "0"),
        ("06_7", "up to zero", "R-F14.clamp_at", "0"),
        ("06_7", "above four", "R-F14.estimate_mean", "4"),
        ("07_1", "three times in four", "R-F2.yes_outcomes.copied", "3"),
        ("07_1", "once in four", "R-F2.coin_outcomes", "4"),
        ("07_3", "Twenty of them", "R-F1.class", "20"),
        ("07_3", "doubled", "R-F4.factor", "2"),
        ("07_3", "about four people", "R-F14.sd_rounded", "4"),
        ("07_4", "of fifteen", "R-F7.var_estimate", "15"),
        ("07_5", "two classes in five", "R-F19.miss_at_least.over", "2/5"),
        ("07_5", "four or more", "R-F19.miss_by", "4"),
        ("07_6", "three hundred students", "R-F19.big.class", "300"),
        ("07_7", "About fifteen", "R-F19.big.miss", "15"),
        ("07_7", "three quarters", "R-F7.coin_variance_per_person", "3/4"),
        ("07_7", "of three hundred", "R-F19.big.class", "300"),
        ("07_7", "one in twenty", "R-F19.big.miss_share", "1/20"),
        ("07_7", "miss of four", "R-F14.sd_rounded", "4"),
        ("07_7", "about one in five", "R-F19.class.one_in.miss_share", "5"),
        ("08_1", "Sample twenty", "R-F1.class", "20"),
        ("08_2", "one in five in the school", "R-F1.copied_share", "1/5"),
        ("08_2", "Twenty direct", "R-F1.class", "20"),
        ("08_2", "twenty percent", "R-F6.percent.prior", "20"),
        ("08_2", "about nine percentage points", "R-F20.direct.points", "9"),
        ("08_2", "Twenty coin answers", "R-F1.class", "20"),
        ("08_2", "about twenty-one", "R-F20.coins.points", "21"),
        ("08_3", "only nine points", "R-F20.direct.points", "9"),
        ("08_4", "a hundred and fourteen", "R-F20.match", "114"),
        ("08_4", "one in five copied", "R-F1.copied_share", "1/5"),
        ("08_4", "almost six times", "R-F14.cost_rounded", "6"),
        ("09_2", "the four who copied", "R-F1.copied", "4"),
        ("09_3", "Minus two.", "R-F21.average_estimate", "-2"),
        ("09_3", "The sixteen others", "R-F11.others", "16"),
        ("09_3", "four yeses on average", "R-F21.average_yes", "4"),
        ("09_3", "Double four is eight", "R-F21.twice_yes", "8"),
        ("09_3", "take away ten", "R-F4.half_class", "10"),
        ("09_3", "minus two.", "R-F21.average_estimate", "-2"),
        ("09_4", "The truth is four", "R-F1.copied", "4"),
        ("09_5", "below zero", "R-F14.clamp_at", "0"),
        ("09_5", "one time in eleven", "R-F14.one_in.negative", "11"),
        ("10_1", "three times", "R-F9.asks", "3"),
        ("10_2", "Four coin outcomes", "R-F2.coin_outcomes", "4"),
        ("10_2", "sixty-four", "R-F9.outcomes", "64"),
        ("10_2", "all three", "R-F9.asks", "3"),
        ("10_3", "three yeses", "R-F9.asks", "3"),
        ("10_4", "Twenty-seven.", "R-F9.ratio.all_yes", "27"),
        ("10_4", "yes three times", "R-F9.asks", "3"),
        ("10_4", "in twenty-seven outcomes", "R-F9.all_yes.copied", "27"),
        ("10_4", "anyone else in one", "R-F9.all_yes.didnt", "1"),
        ("10_5", "multiply it by three", "R-F5.worst_ratio", "3"),
        ("10_5", "Three noes", "R-F9.asks", "3"),
        ("10_6", "one in five copied", "R-F1.copied_share", "1/5"),
        ("10_6", "Those four", "R-F1.copied", "4"),
        ("10_6", "a hundred and eight", "R-F15.all_yes.copied", "108"),
        ("10_6", "the sixteen others", "R-F11.others", "16"),
        ("10_6", "others sixteen", "R-F15.all_yes.didnt", "16"),
        ("10_7", "after three yeses", "R-F9.asks", "3"),
        ("10_7", "twenty-seven in thirty-one", "R-F9.posterior.all_yes", "27/31"),
        ("10_7", "three in seven", "R-F11.posterior.yes", "3/7"),
        ("10_8", "three answers", "R-F9.asks", "3"),
        (
            "10_8",
            "twenty-seven times in thirty-two",
            "R-F9.majority_right",
            "27/32",
        ),
        ("11_2", "a hundred answers", "R-F22.asks", "100"),
        ("11_2", "seventy-five yes", "R-F22.yes", "75"),
        ("11_2", "twenty-five no", "R-F22.no", "25"),
        ("11_2", "the evidence by three", "R-F5.worst_ratio", "3"),
        ("11_2", "fifty threes", "R-F22.excess", "50"),
        ("11_3", "twenty-three zeros", "R-F22.zeros", "23"),
        ("11_4", "three yeses now", "R-F9.asks", "3"),
        ("11_5", "three yeses", "R-F9.asks", "3"),
        ("11_5", "three times as likely", "R-F10.memoized.ratio", "3"),
        ("11_5", "the same as one", "R-F5.ratio.yes", "3"),
        ("11_7", "twenty fourteen", "R-X7.rappor.year", "2014"),
        ("12_1", "one half", "R-F16.ours.truth", "1/2"),
        ("12_2", "three quarters", "R-F16.up.truth", "3/4"),
        ("12_2", "four slices", "R-F16.slices", "4"),
        ("12_2", "three saying", "R-F16.up.truth_slices", "3"),
        ("12_3", "At three quarters", "R-F16.up.truth", "3/4"),
        ("12_4", "Seven.", "R-F16.up.ratio", "7"),
        ("12_4", "the eight outcomes", "R-F16.outcomes", "8"),
        ("12_4", "yes in seven", "R-F16.up.yes.copied", "7"),
        ("12_4", "anyone else in one", "R-F16.up.yes.didnt", "1"),
        ("12_6", "Our class of twenty", "R-F1.class", "20"),
        ("12_6", "about two people", "R-F16.up.miss_rounded", "2"),
        ("12_6", "not four", "R-F16.ours.miss_rounded", "4"),
        ("12_7", "at a quarter", "R-F16.down.truth", "1/4"),
        ("12_7", "under twice", "R-F16.down.ratio_under", "2"),
        ("12_7", "about nine", "R-F16.down.miss_rounded", "9"),
        ("12_9", "the second coin", "R-F2.coins", "2"),
        ("13_3", "a quarter of the class", "R-F4.forced_yes_share", "1/4"),
        ("13_3", "double what's left", "R-F4.factor", "2"),
        ("13_4", "three quarters", "R-F7.coin_variance_per_person", "3/4"),
        ("13_6", "the three times", "R-F5.worst_ratio", "3"),
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
            "zero",
            "one",
            "two",
            "three",
            "four",
            "five",
            "six",
            "seven",
            "eight",
            "nine",
            "ten",
            "eleven",
            "twelve",
            "thirteen",
            "fourteen",
            "fifteen",
            "sixteen",
            "eighteen",
            "twenty",
            "twentieth",
            "thirty",
            "forty",
            "fifty",
            "sixty",
            "seventy",
            "eighty",
            "hundred",
            "trillion",
            "minus",
            "half",
            "quarter",
            "double",
            "doubled",
            "twice",
            "second",
            "times",
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
