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
        ("03_1", "five, one, one, one and one", "DT-F4.cols", "5,1,1,1,1"),
        ("03_2", "eight of nine", "DT-F9.flow", "8"),
        ("03_2", "the ninth through", "DT-F4.totals", "9,9"),
        ("03_3", "The first column", "DT-F4.bottleneck", "0"),
        ("03_3", "needs five", "DT-F9.first.needs", "5"),
        ("03_3", "at most one", "DT-F8.square", "1"),
        ("03_3", "only four rows", "DT-F9.rows_with_water", "4"),
        ("03_4", "Four into the first column", "DT-F9.cut.into_unreached", "4"),
        ("03_4", "four out of the other drains", "DT-F9.cut.reached_drains", "4"),
        ("03_4", "Eight is all", "DT-F9.cut", "8"),
        ("03_5", "at most one square per column", "DT-F8.square", "1"),
        ("03_8", "sixty-four", "DT-F3.fits", "64"),
        ("03_8", "not whether only one", "DT-F8.built.only_one", "false"),
        ("04_1", "the only one", "DT-F8.built.only_one", "false"),
        ("04_2", "twelve switches", "DT-F10.face.switches", "12"),
        ("04_2", "A second picture fits", "DT-F10.face.second.fits", "true"),
        ("04_3", "one, three, five, one and one", "DT-F6.tree.rows", "1,3,5,1,1"),
        ("04_3", "one, two, five, two and one", "DT-F6.tree.cols", "1,2,5,2,1"),
        ("04_4", "the tree itself", "DT-F10.tree.built.is_tree", "true"),
        ("04_5", "two rows", "DT-F10.switch.rows", "2"),
        ("04_6", "any two rows", "DT-F10.tree.nested", "true"),
        ("04_6", "the tree has no switch", "DT-F10.tree.built.switches", "0"),
        ("04_7", "no other picture fits", "DT-F6.tree.fits", "1"),
        ("04_8", "whether it's the only one", "DT-X4.disagree", "0"),
        ("05_1", "sixty-four pictures", "DT-F3.fits", "64"),
        ("05_1", "one more shadow", "DT-F11.shadows", "3"),
        ("05_2", "takes a square from two diagonals", "DT-F11.twin.losers", "2"),
        ("05_2", "gives both to a third", "DT-F11.twin.gain.by", "2"),
        ("05_3", "the fourteen pictures one switch", "DT-F11.one_switch", "14"),
        ("05_3", "none of the fourteen", "DT-F11.one_switch.pass", "0"),
        ("05_3", "all sixty-four", "DT-F3.fits", "64"),
        ("05_4", "Two.", "DT-F11.survivors", "2"),
        ("05_4", "Sixty-two fail", "DT-F11.dropped", "62"),
        ("05_4", "one other picture", "DT-F11.others", "1"),
        ("05_4", "all three of the face's shadows", "DT-F11.shadows", "3"),
        ("05_5", "six squares", "DT-F11.other.differ", "6"),
        ("05_5", "loses one square", "DT-F11.other.loses", "1"),
        ("05_5", "gains one", "DT-F11.other.gains", "1"),
        ("05_5", "all three shadows stay", "DT-F11.shadows", "3"),
        ("05_6", "Two shadows", "DT-F1.directions", "2"),
        ("05_6", "left sixty-four", "DT-F3.fits", "64"),
        ("05_6", "A third still leaves two", "DT-F11.survivors", "2"),
        ("05_6", "even three shadows lie", "DT-F11.only_one", "false"),
        ("05_7", "the two questions", "DT-F7.questions", "2"),
        ("05_7", "the only one", "DT-F11.only_one", "false"),
    ];

    /// The counts 04_3 reads out are the tree's, its rows then its columns.
    #[test]
    fn the_spoken_tree_counts_are_the_trees() {
        let script = Script::parse(SCRIPT).unwrap();
        let text = script.text("04_3").unwrap();
        let read = |words: &str| -> Vec<usize> {
            words
                .split([',', ' '])
                .filter(|w| !w.is_empty() && *w != "and")
                .map(|w| match w {
                    "one" => 1,
                    "two" => 2,
                    "three" => 3,
                    "five" => 5,
                    other => panic!("an unexpected count word {other:?}"),
                })
                .collect()
        };
        let rows = text
            .split("Rows of ")
            .nth(1)
            .unwrap()
            .split(", columns of ")
            .next()
            .unwrap();
        let cols = text.split("columns of ").nth(1).unwrap().split('.').next().unwrap();
        let tree = model::tree();
        assert_eq!(read(rows), tree.row_counts());
        assert_eq!(read(cols), tree.col_counts());
    }

    /// The column counts 03_1 reads out are the ones the chapter draws.
    #[test]
    fn the_spoken_column_counts_are_the_drawn_ones() {
        let script = Script::parse(SCRIPT).unwrap();
        let text = script.text("03_1").unwrap();
        let words = text.split("asking for ").nth(1).unwrap().trim_end_matches('.');
        let counts: Vec<usize> = words
            .split([',', ' '])
            .filter(|w| !w.is_empty() && *w != "and")
            .map(|w| match w {
                "one" => 1,
                "five" => 5,
                other => panic!("an unexpected count word {other:?}"),
            })
            .collect();
        assert_eq!(counts, model::NO_FIT_COLS);
    }

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
            "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven", "twelve",
            "fourteen", "forty", "sixty", "half", "twice", "hundred", "first", "second", "third", "fourth", "fifth",
            "sixth", "seventh", "eighth", "ninth",
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
