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
        ("02_1", "all four coin outcomes", "R-F2.coin_outcomes", "4"),
        ("02_1", "Twenty students", "R-F1.class", "20"),
        ("02_1", "eighty outcomes", "R-F11.class_outcomes", "80"),
        ("02_2", "three of their four", "R-F2.yes_outcomes.copied", "3"),
        ("02_2", "of their four", "R-F2.coin_outcomes", "4"),
        ("02_2", "in one", "R-F2.yes_outcomes.didnt", "1"),
        ("02_2", "Three times as likely", "R-F5.ratio.yes", "3"),
        ("02_3", "three times again", "R-F5.ratio.no", "3"),
        ("02_5", "The four who copied", "R-F1.copied", "4"),
        ("02_5", "have twelve", "R-F11.yes.copied", "12"),
        ("02_5", "the sixteen others", "R-F11.others", "16"),
        ("02_5", "others sixteen", "R-F11.yes.didnt", "16"),
        ("02_6", "Twelve of", "R-F11.yes.copied", "12"),
        ("02_6", "of twenty-eight", "R-F11.yes", "28"),
        ("02_6", "three of the seven", "R-F11.average.yes.copied", "3"),
        ("02_6", "the seven yeses", "R-F11.average.yes", "7"),
        ("02_6", "Three in seven", "R-F11.posterior.yes", "3/7"),
        ("02_6", "not three in four", "R-F2.p_yes.copied", "3/4"),
        ("02_7", "three times the yeses", "R-F5.ratio.yes", "3"),
        ("02_7", "four to one", "R-F11.outnumber", "4"),
        ("02_8", "Four of the", "R-F11.no.copied", "4"),
        ("02_8", "fifty-two no outcomes", "R-F11.no", "52"),
        ("02_8", "one in thirteen", "R-F11.posterior.no", "1/13"),
        ("02_9", "one in five", "R-F1.copied_share", "1/5"),
        ("02_9", "three in seven", "R-F11.posterior.yes", "3/7"),
        ("02_9", "one in thirteen", "R-F11.posterior.no", "1/13"),
        ("02_10", "the three times", "R-F5.worst_ratio", "3"),
        ("03_1", "eight times", "R-F13.runs", "8"),
        ("03_2", "from minus four", "R-F13.lowest", "-4"),
        ("03_2", "to fourteen", "R-F13.highest", "14"),
        ("03_2", "Only one lands", "R-F13.on_four", "1"),
        ("03_2", "lands on four", "R-F14.estimate_mean", "4"),
        ("03_3", "Four to the twentieth", "R-F2.coin_outcomes", "4"),
        ("03_3", "the twentieth", "R-F1.class", "20"),
        ("03_3", "over a trillion", "R-F14.outcomes_trillions", "1"),
        ("03_4", "around four", "R-F14.estimate_mean", "4"),
        ("03_5", "exactly on four", "R-F1.copied", "4"),
        ("03_6", "one class in five", "R-F14.one_in.exactly", "5"),
        ("03_6", "one in eleven", "R-F14.one_in.negative", "11"),
        ("03_6", "below zero", "R-F14.clamp_at", "0"),
        ("03_7", "Below zero", "R-F14.clamp_at", "0"),
        ("03_7", "up to zero", "R-F14.clamp_at", "0"),
        ("03_7", "above four", "R-F14.estimate_mean", "4"),
        ("03_8", "Twenty of them", "R-F1.class", "20"),
        ("03_8", "doubled", "R-F4.factor", "2"),
        ("03_8", "about four people", "R-F14.sd_rounded", "4"),
        ("03_9", "Sample twenty", "R-F1.class", "20"),
        ("03_10", "one in five", "R-F1.copied_share", "1/5"),
        ("03_10", "almost six times", "R-F14.cost_rounded", "6"),
        ("04_1", "three times", "R-F9.asks", "3"),
        ("04_2", "Four coin outcomes", "R-F2.coin_outcomes", "4"),
        ("04_2", "sixty-four", "R-F9.outcomes", "64"),
        ("04_2", "all three", "R-F9.asks", "3"),
        ("04_3", "three yeses", "R-F9.asks", "3"),
        ("04_4", "Twenty-seven.", "R-F9.ratio.all_yes", "27"),
        ("04_4", "yes three times", "R-F9.asks", "3"),
        ("04_4", "in twenty-seven outcomes", "R-F9.all_yes.copied", "27"),
        ("04_4", "anyone else in one", "R-F9.all_yes.didnt", "1"),
        ("04_5", "multiply it by three", "R-F5.worst_ratio", "3"),
        ("04_5", "Three noes", "R-F9.asks", "3"),
        ("04_6", "one in five copied", "R-F1.copied_share", "1/5"),
        ("04_6", "Those four", "R-F1.copied", "4"),
        ("04_6", "a hundred and eight", "R-F15.all_yes.copied", "108"),
        ("04_6", "the sixteen others", "R-F11.others", "16"),
        ("04_6", "others sixteen", "R-F15.all_yes.didnt", "16"),
        ("04_7", "after three yeses", "R-F9.asks", "3"),
        ("04_7", "twenty-seven in thirty-one", "R-F9.posterior.all_yes", "27/31"),
        ("04_7", "three in seven", "R-F11.posterior.yes", "3/7"),
        ("04_8", "three answers", "R-F9.asks", "3"),
        (
            "04_8",
            "twenty-seven times in thirty-two",
            "R-F9.majority_right",
            "27/32",
        ),
        ("04_10", "three yeses", "R-F9.asks", "3"),
        ("04_10", "three times as likely", "R-F10.memoized.ratio", "3"),
        ("04_10", "the same as one", "R-F5.ratio.yes", "3"),
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
            "ten",
            "eleven",
            "twelve",
            "thirteen",
            "fourteen",
            "sixteen",
            "twenty",
            "twentieth",
            "thirty",
            "fifty",
            "sixty",
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
