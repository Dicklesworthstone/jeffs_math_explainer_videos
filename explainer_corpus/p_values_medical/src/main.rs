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

    #[test]
    fn the_storyboard_the_registry_and_the_script_agree() {
        explainer_kit::storyboard::check_video(STORYBOARD, SCRIPT, &keys()).unwrap();
        let script = Script::parse(SCRIPT).unwrap();
        check_script_owners(&script, &keys(), None).unwrap();
    }
}
