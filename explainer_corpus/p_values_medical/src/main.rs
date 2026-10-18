//! Why P-Values Confuse Even Doctors: a survey of residents who were
//! confident about p-values and an answer key that was itself disputed; a
//! positive test counted out of ten thousand people, so the five percent and
//! the answer the patient wants are seen to count different groups; then a
//! treatment study's imaginary world of label assignments, whose whole tail
//! is the p-value. Rendered end to end by franken_manim's native Rust front
//! door and the corpus's shared kit.
//!
//! ```text
//! pvalues facts                      # every number the video shows or says (key<TAB>value)
//! pvalues script                     # the narration (generated from docs/storyboard.md)
//! pvalues render all|<chapter>... [--res 960x540] [--scratch-voice] [--out DIR] ...
//! pvalues help                       # the full option list
//! ```

mod chapters;
mod facts;
mod model;
mod score;

use explainer_kit::Explainer;

/// The narration, generated from the storyboard by `tools/storyboard.py`.
pub(crate) const SCRIPT: &str = include_str!("../script.tsv");
/// The storyboard: words, pictures, holds and listen windows per beat. Only
/// the tests read it (the script, the score's motif windows and the title's
/// listen window are checked against it); the binary reads `script.tsv`.
#[cfg(test)]
pub(crate) const STORYBOARD: &str = include_str!("../docs/storyboard.md");

const EXPLAINER: Explainer = Explainer {
    crate_name: "p_values_medical",
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

    /// Narration truth: every number the voice says, with the fact it speaks
    /// for and the value those words denote. A changed rate, count or word
    /// breaks this test before it reaches a listener.
    const SPOKEN: &[(&str, &str, &str, &str)] = &[
        (
            "01_1",
            "published in two thousand seven",
            "M-C1.survey.published_year",
            "2007",
        ),
        ("01_1", "two hundred seventy-seven", "M-C1.survey.respondents", "277"),
        ("01_1", "Eighty-eight percent", "M-C1.survey.confident_percent", "88"),
        (
            "01_2",
            "about fifty-nine percent",
            "M-C1.survey.keyed_percent_spoken",
            "59",
        ),
        ("01_4", "five percent of people", "M-F1.false_alarm_percent", "5"),
        ("01_5", "ninety-five percent", "M-F1.tempting_percent", "95"),
        ("01_6", "Ten thousand", "M-F1.people", "10000"),
        ("01_6", "one in a hundred", "M-F1.prevalence.one_in", "100"),
        ("01_7", "one row", "M-F1.grid.disease_rows", "1"),
        ("01_7", "of a hundred", "M-F1.with", "100"),
        ("01_7", "catches ninety", "M-F1.true_pos", "90"),
        ("01_8", "nine thousand nine hundred", "M-F1.without", "9900"),
        ("01_8", "Five percent of them", "M-F1.false_alarm_percent", "5"),
        ("01_8", "Four hundred ninety-five false alarms", "M-F1.false_pos", "495"),
        ("01_10", "Ninety have the disease", "M-F1.true_pos", "90"),
        ("01_10", "four hundred ninety-five", "M-F1.false_pos", "495"),
        ("01_10", "about fifteen percent", "M-F1.ppv_percent", "15"),
        ("01_11", "Five percent", "M-F1.false_alarm_percent", "5"),
        ("02_1", "one person in ten", "M-F2.prevalence.one_in", "10"),
        ("02_2", "catches nine in ten", "M-F2.sensitivity.exact", "9/10"),
        ("02_2", "five percent of the rest", "M-F2.false_alarm_percent", "5"),
        ("02_4", "A thousand", "M-F2.with", "1000"),
        ("02_4", "nine hundred of them", "M-F2.true_pos", "900"),
        ("02_5", "nine thousand", "M-F2.without", "9000"),
        ("02_5", "five percent", "M-F2.false_alarm_percent", "5"),
        ("02_5", "four hundred fifty", "M-F2.false_pos", "450"),
        ("02_6", "Nine hundred", "M-F2.true_pos", "900"),
        ("02_6", "thirteen hundred fifty", "M-F2.positives", "1350"),
        ("02_6", "two in three", "M-F2.ppv.exact", "2/3"),
        ("02_7", "sixty-seven percent", "M-F2.ppv_percent", "67"),
        ("02_7", "not fifteen", "M-F1.ppv_percent", "15"),
        ("02_8", "ten times as many", "M-F2.caught_vs_a", "10"),
        ("03_1", "the twenty pairs", "M-T1.pairs", "20"),
        ("03_4", "In fifteen of the twenty", "M-T1.observed", "15"),
        ("03_4", "the twenty pairs", "M-T1.pairs", "20"),
        ("03_7", "The twenty coins", "M-T1.pairs", "20"),
        ("03_7", "just over a million", "M-T1.assignments", "1048576"),
        ("03_9", "fifteen or more", "M-T1.observed", "15"),
        ("04_1", "found fifteen", "M-T1.observed", "15"),
        ("04_2", "as favourable as fifteen", "M-T1.observed", "15"),
        ("04_3", "Fifteen itself", "M-T1.observed", "15"),
        ("04_3", "up to twenty", "M-T1.pairs", "20"),
        ("04_4", "All twenty pairs", "M-T1.pairs", "20"),
        ("04_4", "only one way", "M-T4.count_20", "1"),
        ("04_5", "For nineteen", "M-T4.k_one_against", "19"),
        ("04_5", "any of the twenty", "M-T1.pairs", "20"),
        ("04_5", "Twenty ways", "M-T4.count_19", "20"),
        ("04_6", "Down to fifteen", "M-T1.observed", "15"),
        (
            "04_6",
            "twenty-one thousand seven hundred",
            "M-T1.tail.at_least",
            "21700",
        ),
        ("04_7", "just over a million", "M-T1.assignments", "1048576"),
        ("04_7", "about two percent", "M-T1.p_percent_0dp", "2"),
        ("04_8", "exactly fifteen", "M-T1.observed", "15"),
        (
            "04_8",
            "about one and a half percent",
            "M-X1.exactly_percent_1dp",
            "1.5",
        ),
        ("04_8", "more than fifteen", "M-T1.observed", "15"),
        (
            "04_8",
            "about six tenths of a percent",
            "M-X1.more_than_percent_1dp",
            "0.6",
        ),
        ("05_1", "Two percent", "M-T1.p_percent_0dp", "2"),
        ("05_2", "is two percent", "M-T1.p_percent_0dp", "2"),
        ("05_3", "The two percent", "M-T1.p_percent_0dp", "2"),
        ("05_5", "one more pair", "M-T2.moved", "1"),
        ("05_5", "Fourteen.", "M-T2.observed", "14"),
        ("05_6", "Fourteen or more", "M-T2.observed", "14"),
        ("05_6", "about five point eight percent", "M-T2.p_percent_1dp", "5.8"),
        ("05_7", "a line at five percent", "M-T5.line_percent", "5"),
        ("05_7", "Fifteen falls under it", "M-T1.observed", "15"),
        ("05_7", "fourteen over", "M-T2.observed", "14"),
        ("05_8", "two made-up sets", "M-D.versions", "2"),
        ("05_9", "the same fifteen favouring pairs", "M-D1.favouring", "15"),
        ("05_9", "the same fifteen favouring pairs", "M-D2.favouring", "15"),
        ("05_9", "typically a day sooner", "M-D1.median_sooner_days", "1"),
        ("05_9", "a week in the other", "M-D2.median_sooner_days", "7"),
        ("05_10", "same two percent", "M-D1.p_percent_0dp", "2"),
        ("05_10", "same two percent", "M-D2.p_percent_0dp", "2"),
        (
            "06_2",
            "a p-value above five percent",
            "M-C1.survey.keyed_for_percent",
            "5",
        ),
        ("06_3", "the two percent", "M-T1.p_percent_0dp", "2"),
        ("06_4", "fifteen or more pairs", "M-T1.observed", "15"),
        ("06_4", "about two percent", "M-T1.p_percent_0dp", "2"),
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
        // and no line carries a number the table above does not know
        let big = [
            "hundred", "thousand", "percent", "seven", "eight", "nine", "fifteen", "fifty", "ninety", "twenty",
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

    /// The survey is spoken as it was: a regional sample of residents, most
    /// of whom picked the keyed answer; never "most doctors" or a majority
    /// for a wrong choice (research brief, S1 and S2).
    #[test]
    fn the_survey_is_not_inflated() {
        let script = Script::parse(SCRIPT).unwrap();
        let all: String = script
            .lines()
            .iter()
            .map(|(_, t)| t.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(all.contains("residents in Connecticut"));
        for wrong in [
            "most doctors",
            "all doctors",
            "doctors failed",
            "most picked the wrong",
            "got it wrong",
        ] {
            assert!(!all.contains(wrong), "the script says {wrong:?}");
        }
        // the spoken year is the publication's: the survey ran the year before
        assert!(all.contains("published in two thousand seven"));
        assert!(!all.contains("In two thousand seven"), "the survey did not run in 2007");
        let f = facts::facts().unwrap();
        let year = |key: &str| f.get(key).unwrap().parse::<u32>().unwrap();
        assert!(year("M-C1.survey.collected_year") < year("M-C1.survey.published_year"));
        // most residents picked the keyed answer: the dispute is about the key
        let keyed: f64 = f.get("M-C1.survey.keyed_percent").unwrap().parse().unwrap();
        assert!(keyed > 50.0, "most residents picked the keyed answer");
    }

    /// The closing question is conditioned as the study was counted: the
    /// world and the coins are the condition, fifteen or more is what is
    /// counted. No line turns a p-value into the chance a hypothesis is true
    /// or the chance the treatment works, or conditions the other way round.
    #[test]
    fn the_closing_question_is_conditioned_the_way_it_was_counted() {
        let script = Script::parse(SCRIPT).unwrap();
        let line = |id: &str| {
            script
                .lines()
                .iter()
                .find(|(i, _)| i == id)
                .map(|(_, t)| t.clone())
                .unwrap()
        };
        let q = line("06_4");
        assert!(q.starts_with("If the treatment changed no one's recovery, and the coins were tossed again,"));
        let (condition, counted) = q.split_once("again,").unwrap();
        assert!(!condition.contains("fifteen") && counted.contains("fifteen or more pairs would favour it"));
        let all: String = script
            .lines()
            .iter()
            .map(|(_, t)| t.to_lowercase())
            .collect::<Vec<_>>()
            .join(" ");
        for wrong in [
            "chance the treatment works",
            "chance that the treatment works",
            "probability the treatment",
            "given fifteen",
            "chance the null",
            "percent chance the treatment did nothing",
        ] {
            assert!(!all.contains(wrong), "the script says {wrong:?}");
        }
        assert!(line("06_2").contains("chance alone. Chance of what, counted how?"));
    }

    #[test]
    fn the_storyboard_the_registry_and_the_script_agree() {
        explainer_kit::storyboard::check_video(STORYBOARD, SCRIPT, &keys()).unwrap();
        let script = Script::parse(SCRIPT).unwrap();
        check_script_owners(&script, &keys(), None).unwrap();
    }
}
