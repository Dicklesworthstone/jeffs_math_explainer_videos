//! Two Shadows Can Lie: pictures of filled squares, the counts along their
//! rows and columns, the switch that changes a picture and neither count,
//! and the difference between a picture that fits the counts and the only
//! one that does; rendered end to end by franken_manim's native Rust front
//! door and the corpus's shared kit.
//!
//! ```text
//! tomography facts                    # every number the video shows or says (key<TAB>value)
//! tomography script                   # the narration (generated from docs/storyboard.md)
//! tomography render all|<chapter>... [--res 960x540] [--scratch-voice] [--out DIR] ...
//! tomography help                     # the full option list
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
    crate_name: "discrete_tomography",
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
    /// for and the value those words denote. A changed picture or word
    /// breaks this test before it reaches a listener.
    const SPOKEN: &[(&str, &str, &str, &str)] = &[
        ("01_2", "two shadows", "DT-F1.directions", "2"),
        ("01_5", "four squares", "DT-F2.differ", "4"),
        ("01_6", "loses one square", "DT-F2.loses", "1"),
        ("01_6", "gains one", "DT-F2.gains", "1"),
        ("01_8", "Sixty-four", "DT-F3.fits", "64"),
        ("01_8", "at most three switches", "DT-F3.farthest", "3"),
        ("01_9", "keep their total", "DT-F4.totals", "9,9"),
        ("01_9", "no picture fits", "DT-F4.fits", "0"),
        ("01_10", "a square in every row", "DT-F4.bottleneck.asks", "5"),
        ("01_10", "the middle row is empty", "DT-F1.face.empty_rows", "2"),
        ("01_11", "two questions", "DT-F7.questions", "2"),
        ("02_3", "at most one unit", "DT-F8.square", "1"),
        ("02_4", "one unit at a time", "DT-F8.room", "1"),
        ("02_5", "Eight are through", "DT-F8.before.through", "8"),
        ("02_5", "has one left", "DT-F8.stuck.left", "1"),
        // rows and columns are numbered from 0 in the facts: the bottom row
        // is row 4, the first column 0, the fourth column 3
        ("02_5", "The bottom row", "DT-F8.stuck.row", "4"),
        ("02_6", "the first column", "DT-F8.takeback.column", "0"),
        ("02_6", "the top row", "DT-F8.takeback.row", "0"),
        ("02_7", "the fourth column", "DT-F8.takeback.to_column", "3"),
        ("02_7", "still needs one", "DT-F8.takeback.to_needs", "1"),
        ("02_7", "one unit short", "DT-F8.short", "1"),
        ("02_8", "they aren't the face", "DT-F8.built.is_face", "false"),
        ("02_8", "sixty-four", "DT-F3.fits", "64"),
        ("02_9", "whenever one exists", "DT-F8.built.fits", "true"),
        ("02_9", "the only one", "DT-F8.built.only_one", "false"),
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
        // "a square in every row": the column asks for as many as there are
        // rows; "the middle row": the empty row is the middle one
        let size: Vec<usize> = f
            .get("DT-F1.size")
            .unwrap()
            .split(',')
            .map(|n| n.parse().unwrap())
            .collect();
        assert_eq!(f.get("DT-F4.bottleneck.asks"), Some(size[0].to_string().as_str()));
        assert_eq!(f.get("DT-F1.face.empty_rows"), Some((size[0] / 2).to_string().as_str()));
        // "the bottom row": the stuck row is the last one
        assert_eq!(f.get("DT-F8.stuck.row"), Some((size[0] - 1).to_string().as_str()));
        // and no line carries a number word the table above does not know
        let numbers = [
            "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "sixty", "half",
            "twice", "hundred", "first", "second", "third", "fourth", "fifth",
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

    /// The synthesis plan, narration_units.json, was built from this script:
    /// it names the script's hash and holds every line once, in order. It is
    /// a grouping of whole thoughts for a future take, not a qualified
    /// synthesis route.
    #[test]
    fn the_narration_units_are_built_from_this_script() {
        let units = include_str!("../narration_units.json");
        let hash = explainer_score::digest::sha256_hex(SCRIPT.as_bytes());
        assert!(
            units.contains(&format!("\"script_sha256\": \"{hash}\"")),
            "narration_units.json does not bind this script.tsv"
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
