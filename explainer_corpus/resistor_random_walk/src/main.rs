//! A Resistor Network Is Secretly Answering a Probability Question: one
//! small network of identical wires, its voltages and a random walker's
//! chances computed independently and shown to obey the same averaging
//! rule, rendered end to end by franken_manim's native Rust front door and
//! the corpus's shared kit.
//!
//! ```text
//! resistor facts                         # every number the video shows or says (key<TAB>value)
//! resistor script                        # the narration (generated from docs/storyboard.md)
//! resistor render all|<chapter>... [--res 960x540] [--scratch-voice] [--out DIR] ...
//! resistor help                          # the full option list
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
    crate_name: "resistor_random_walk",
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
    /// for and the value those words denote. A changed value or word breaks
    /// this test before it reaches a listener.
    const SPOKEN: &[(&str, &str, &str, &str)] = &[
        ("01_1", "four junctions", "R-F1.junctions", "4"),
        ("01_1", "doubled in two places", "R-F1.doubled", "2"),
        ("01_1", "doubled", "R-F1.side_by_side", "2"),
        ("01_1", "the top at one volt", "R-F1.top_volts", "1"),
        ("01_1", "the bottom at zero", "R-F1.bottom_volts", "0"),
        ("01_2", "its four wires", "R-F1.wires_at_middle", "4"),
        (
            "01_3",
            "one, one, two fifths and zero",
            "R-F2.left.far_ends",
            "1,1,2/5,0",
        ),
        ("01_3", "average to three fifths", "R-F2.left.average", "3/5"),
        ("01_3", "three fifths", "R-F2.left", "3/5"),
        ("01_4", "one of its four wires", "R-F1.wires_at_middle", "4"),
        ("01_5", "its four wires", "R-F1.wires_at_middle", "4"),
        ("01_6", "From the top the chance is one", "R-F3.top_chance", "1"),
        ("01_6", "from the bottom it's zero", "R-F3.bottom_chance", "0"),
        ("01_7", "Three fifths again", "R-F3.left", "3/5"),
        ("01_7", "Three fifths again", "R-F2.left", "3/5"),
        ("01_7", "only one solution", "R-X1.solutions", "1"),
        ("01_8", "The two numbers agree", "R-F3.same", "true"),
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
            "zero", "one", "two", "three", "four", "five", "fifths", "half", "quarter", "twice", "doubled", "once",
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
    /// order (`python3 -I narration_units.py --check` checks the rest; with
    /// no argument it rebuilds the file after a script edit).
    #[test]
    fn the_narration_units_are_built_from_this_script() {
        let units = include_str!("../narration_units.json");
        let hash = explainer_score::digest::sha256_hex(SCRIPT.as_bytes());
        assert!(
            units.contains(&format!("\"script_sha256\": \"{hash}\"")),
            "narration_units.json is stale: run python3 -I narration_units.py"
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

    /// The one prediction is the chapter's walk question, and its answer is
    /// the next line's first words.
    #[test]
    fn the_prediction_is_asked_once_and_answered_next() {
        let boards = explainer_kit::storyboard::parse(STORYBOARD).unwrap();
        let beats: Vec<_> = boards.iter().flat_map(|c| c.beats.iter()).collect();
        let asked: Vec<usize> = (0..beats.len()).filter(|&i| beats[i].predict()).collect();
        assert_eq!(asked.len(), 1, "one prediction in the opening");
        let (question, answer) = (beats[asked[0]], beats[asked[0] + 1]);
        assert!(question.spoken().unwrap().ends_with("left junction?"));
        assert!(answer.spoken().unwrap().starts_with("Three fifths"));
    }
}
