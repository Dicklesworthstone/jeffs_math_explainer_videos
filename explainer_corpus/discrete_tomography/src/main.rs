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
        // "One reason" is not a count: the line's claim is the source's
        ("02_1", "tiny crystals", "DT-S4.batenburg", "2006"),
        ("02_3", "one, three, five, one and one", "DT-F6.tree.rows", "1,3,5,1,1"),
        ("02_3", "one, two, five, two and one", "DT-F6.tree.cols", "1,2,5,2,1"),
        ("02_4", "The row of five", "DT-F12.tree.full", "5"),
        ("02_4", "the column of five", "DT-F12.tree.full", "5"),
        // columns are numbered from 0: the first and the last of five
        ("02_5", "the first and last columns", "DT-F12.tree.met.cols", "0,4"),
        ("02_5", "their one square", "DT-F12.tree.met.count", "1"),
        ("02_5", "the rows of one", "DT-F12.tree.met.count", "1"),
        ("02_6", "The second row", "DT-F12.tree.last.row", "1"),
        ("02_6", "needs two more", "DT-F12.tree.last.needs", "2"),
        ("02_6", "exactly two squares are left open", "DT-F12.tree.last.open", "2"),
        ("02_7", "All five squares", "DT-F12.face.settled", "5"),
        ("02_9", "Every other square", "DT-F12.face.free", "20"),
        ("02_9", "sixty-four", "DT-F3.fits", "64"),
        ("02_10", "some of the sixty-four", "DT-F3.fits", "64"),
        ("03_2", "Two choices", "DT-F13.choices", "2"),
        ("03_2", "twenty-five squares", "DT-F13.squares", "25"),
        ("03_2", "over thirty-three million", "DT-F13.millions", "33"),
        ("03_2", "twenty by twenty", "DT-F13.big.side", "20"),
        ("03_2", "a hundred and twenty-one digits", "DT-F13.big.digits", "121"),
        ("03_5", "at most one unit", "DT-F8.square", "1"),
        ("03_6", "one down each filled square", "DT-F8.square", "1"),
        ("03_8", "one unit at a time", "DT-F8.room", "1"),
        ("04_1", "Eight are through", "DT-F8.before.through", "8"),
        ("04_1", "has one left", "DT-F8.stuck.left", "1"),
        // rows and columns are numbered from 0 in the facts: the bottom row
        // is row 4, the first column 0, the fourth column 3
        ("04_1", "The bottom row", "DT-F8.stuck.row", "4"),
        ("04_2", "the first column", "DT-F8.takeback.column", "0"),
        ("04_2", "the top row", "DT-F8.takeback.row", "0"),
        ("04_3", "the fourth column", "DT-F8.takeback.to_column", "3"),
        ("04_3", "still needs one", "DT-F8.takeback.to_needs", "1"),
        ("04_3", "one unit short", "DT-F8.short", "1"),
        ("04_4", "The top row still sends two", "DT-F14.top_row.sends", "2"),
        ("04_4", "the first column still drains one", "DT-F14.first_col.drains", "1"),
        ("04_4", "the first column", "DT-F8.takeback.column", "0"),
        ("04_4", "the fourth column", "DT-F8.takeback.to_column", "3"),
        ("04_5", "one square emptied", "DT-F14.emptied", "1"),
        ("04_5", "two filled", "DT-F14.filled", "2"),
        ("04_5", "The bottom row", "DT-F8.stuck.row", "4"),
        ("04_5", "the first column", "DT-F8.takeback.column", "0"),
        ("04_5", "moved to the fourth", "DT-F8.takeback.to_column", "3"),
        ("04_6", "one more unit", "DT-F8.room", "1"),
        ("04_6", "only nine to send", "DT-F8.through", "9"),
        ("04_7", "they aren't the face", "DT-F8.built.is_face", "false"),
        ("04_7", "sixty-four", "DT-F3.fits", "64"),
        ("04_8", "whenever one exists", "DT-F8.built.fits", "true"),
        ("04_8", "the only one", "DT-F8.built.only_one", "false"),
        ("05_1", "five, one, one, one and one", "DT-F4.cols", "5,1,1,1,1"),
        ("05_2", "eight of nine", "DT-F9.flow", "8"),
        ("05_2", "the ninth through", "DT-F4.totals", "9,9"),
        ("05_3", "The first column", "DT-F4.bottleneck", "0"),
        ("05_3", "needs five", "DT-F9.first.needs", "5"),
        ("05_3", "at most one", "DT-F8.square", "1"),
        ("05_3", "only four rows", "DT-F9.rows_with_water", "4"),
        ("05_4", "every column but the first", "DT-F4.cut", "0"),
        // "one of the other drains" is exact because no unreached tap has water
        ("05_5", "into the first column", "DT-F4.cut", "0"),
        ("05_5", "or out through one of the other drains", "DT-F9.cut.unreached_taps", "0"),
        ("05_6", "Four pipes", "DT-F9.cut.into_unreached", "4"),
        ("05_6", "the other drains hold four", "DT-F9.cut.reached_drains", "4"),
        ("05_6", "eight is all", "DT-F9.cut", "8"),
        ("06_1", "rows of four, four, one, one and one", "DT-F15.rows", "4,4,1,1,1"),
        ("06_1", "columns the same", "DT-F15.cols", "4,4,1,1,1"),
        ("06_2", "No count is zero or five", "DT-F15.counts", "1,4"),
        ("06_2", "no row or column forces a single square", "DT-F15.settled", "0"),
        ("06_2", "each column of four", "DT-F15.largest", "4"),
        ("06_2", "five rows with water", "DT-F15.rows_with_water", "5"),
        ("06_3", "the two columns of four", "DT-F15.pair", "0,1"),
        ("06_3", "eight squares", "DT-F15.pair.asks", "8"),
        ("06_4", "Two, two, one, one and one", "DT-F15.pair.by_row", "2,2,1,1,1"),
        ("06_4", "make seven", "DT-F15.pair.gives", "7"),
        ("06_5", "Eight asked", "DT-F15.pair.asks", "8"),
        ("06_5", "seven possible", "DT-F15.pair.gives", "7"),
        ("06_5", "No picture fits", "DT-F15.fits", "0"),
        ("06_6", "ten of eleven", "DT-F15.flow", "10"),
        ("06_6", "ten of eleven", "DT-F15.totals", "11,11"),
        ("06_6", "exactly those two", "DT-F15.cut_off", "0,1"),
        ("06_7", "at most one square per column", "DT-F8.square", "1"),
        ("06_9", "the rows can give every group the same", "DT-F15.other_pairs.gives", "7,7"),
        ("07_2", "Those two columns", "DT-F15.cut_off", "0,1"),
        ("07_2", "hold three units", "DT-F15.cut.unreached_taps", "3"),
        ("07_2", "the four pipes", "DT-F15.cut.pipes", "4"),
        ("07_3", "the three drains", "DT-F15.cut.reached_drains", "3"),
        ("07_3", "came up one short", "DT-F15.short", "1"),
        ("07_3", "three plus four", "DT-F15.cut.unreached_taps", "3"),
        ("07_3", "three plus four", "DT-F15.cut.pipes", "4"),
        ("07_5", "Rows of two and two", "DT-F16.rows", "2,2"),
        ("07_5", "columns of one and one", "DT-F16.cols", "1,1"),
        ("07_5", "pass every size of the test", "DT-F16.failing", "none"),
        ("07_6", "four squares, and the columns only two", "DT-F16.totals", "4,2"),
        ("07_7", "sixty-four", "DT-F3.fits", "64"),
        ("07_7", "not whether only one", "DT-F8.built.only_one", "false"),
        ("08_1", "the only one", "DT-F8.built.only_one", "false"),
        ("08_2", "twelve switches", "DT-F10.face.switches", "12"),
        ("08_2", "A second picture fits", "DT-F10.face.second.fits", "true"),
        ("08_4", "the tree itself", "DT-F10.tree.built.is_tree", "true"),
        ("08_5", "two rows", "DT-F10.switch.rows", "2"),
        ("08_6", "any two rows", "DT-F10.tree.nested", "true"),
        ("08_6", "the tree has no switch", "DT-F10.tree.built.switches", "0"),
        ("08_8", "a four by four grid", "DT-X3.grid", "4,4"),
        ("08_8", "about one picture in ten", "DT-X3.alone.tenths", "1"),
        ("09_1", "one square in every row and column", "DT-F17.perm.per_line", "1"),
        ("09_1", "Can a single switch turn one into the other", "DT-F17.perm.one_switch", "false"),
        ("09_2", "six squares", "DT-F17.perm.differ", "6"),
        ("09_2", "only four", "DT-F2.differ", "4"),
        ("09_3", "two switches do it", "DT-F17.perm.distance", "2"),
        ("09_4", "joined by a chain of switches", "DT-X2.disconnected", "0"),
        ("09_5", "a switch of the tree", "DT-F6.tree.switches", "0"),
        ("09_6", "no other picture fits", "DT-F6.tree.fits", "1"),
        ("09_7", "the face's sixty-four", "DT-F3.fits", "64"),
        ("09_7", "at most three switches", "DT-F3.farthest", "3"),
        ("09_7", "you reach them all", "DT-F3.connected", "true"),
        ("09_8", "whether it's the only one", "DT-X4.disagree", "0"),
        ("10_1", "sixty-four pictures", "DT-F3.fits", "64"),
        ("10_1", "one more shadow", "DT-F11.shadows", "3"),
        ("10_2", "takes a square from two diagonals", "DT-F11.twin.losers", "2"),
        ("10_2", "gives both to a third", "DT-F11.twin.gain.by", "2"),
        ("10_3", "the fourteen pictures one switch", "DT-F11.one_switch", "14"),
        ("10_3", "none of the fourteen", "DT-F11.one_switch.pass", "0"),
        ("10_3", "all sixty-four", "DT-F3.fits", "64"),
        ("10_4", "Two.", "DT-F11.survivors", "2"),
        ("10_4", "Sixty-two fail", "DT-F11.dropped", "62"),
        ("10_4", "one other picture", "DT-F11.others", "1"),
        ("10_4", "all three of the face's shadows", "DT-F11.shadows", "3"),
        ("10_5", "six squares", "DT-F11.other.differ", "6"),
        ("10_5", "loses one square", "DT-F11.other.loses", "1"),
        ("10_5", "gains one", "DT-F11.other.gains", "1"),
        ("10_5", "all three shadows stay", "DT-F11.shadows", "3"),
        ("10_6", "Two shadows", "DT-F1.directions", "2"),
        ("10_6", "left sixty-four", "DT-F3.fits", "64"),
        ("10_6", "A third still leaves two", "DT-F11.survivors", "2"),
        ("10_6", "even three shadows lie", "DT-F11.only_one", "false"),
        ("10_7", "Forty-eight of the sixty-four", "DT-F18.alone", "48"),
        ("10_7", "all three of their own shadows", "DT-F11.shadows", "3"),
        ("10_7", "The face is not one of them", "DT-F18.face.alone", "false"),
        ("10_8", "the first ones mirrored", "DT-F18.face.mirror", "true"),
        ("10_8", "this one's mirror image", "DT-F18.falling.other.mirror", "true"),
        ("10_9", "only the face is left", "DT-F18.both.count", "1"),
        ("11_1", "from two shadows", "DT-F1.directions", "2"),
        ("11_3", "None means it's the only one", "DT-X3.disagree", "0"),
        ("11_4", "the four steps", "DT-F19.steps", "4"),
        ("11_5", "twelve switches", "DT-F10.face.switches", "12"),
        ("11_5", "it isn't the only one", "DT-F8.built.only_one", "false"),
        ("11_6", "which of the sixty-four", "DT-F3.fits", "64"),
        ("11_6", "Two of the face's own squares", "DT-F12.face.rarest.count", "2"),
        ("11_6", "only twelve of them", "DT-F12.face.rarest", "12"),
        ("11_7", "the two questions", "DT-F7.questions", "2"),
    ];

    /// The counts 02_3 reads out are the tree's, its rows then its columns.
    #[test]
    fn the_spoken_tree_counts_are_the_trees() {
        let script = Script::parse(SCRIPT).unwrap();
        let text = script.text("02_3").unwrap();
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

    /// The column counts 05_1 reads out are the ones the chapter draws.
    #[test]
    fn the_spoken_column_counts_are_the_drawn_ones() {
        let script = Script::parse(SCRIPT).unwrap();
        let text = script.text("05_1").unwrap();
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

    /// The counts 06_1 reads out are chapter 06's rows, and its columns are
    /// "the same".
    #[test]
    fn the_spoken_stall_counts_are_the_drawn_ones() {
        let script = Script::parse(SCRIPT).unwrap();
        let text = script.text("06_1").unwrap();
        let words = text
            .split("rows of ")
            .nth(1)
            .unwrap()
            .split(", and columns the same")
            .next()
            .unwrap();
        let counts: Vec<usize> = words
            .split([',', ' '])
            .filter(|w| !w.is_empty() && *w != "and")
            .map(|w| match w {
                "one" => 1,
                "four" => 4,
                other => panic!("an unexpected count word {other:?}"),
            })
            .collect();
        assert_eq!(counts, model::STALL_ROWS);
        assert_eq!(model::STALL_COLS, model::STALL_ROWS);
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
            "fourteen", "twenty", "thirty", "forty", "sixty", "half", "twice", "hundred", "million", "first", "second",
            "third", "fourth", "fifth", "sixth", "seventh", "eighth", "ninth",
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
