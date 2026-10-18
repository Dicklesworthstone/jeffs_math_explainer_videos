//! Two Bad Echoes Can Make One Perfect Measurement: Golay's complementary
//! pair, its exact cancellation, and the shared scene it needs, rendered end
//! to end by franken_manim's native Rust front door and the corpus's shared
//! kit.
//!
//! ```text
//! golay facts                         # every number the video shows or says (key<TAB>value)
//! golay script                        # the narration (generated from docs/storyboard.md)
//! golay render all|<chapter>... [--res 960x540] [--scratch-voice] [--out DIR] ...
//! golay help                          # the full option list
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
    crate_name: "golay_echoes",
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
    /// for and the value those words denote. A changed pattern or word
    /// breaks this test before it reaches a listener.
    const SPOKEN: &[(&str, &str, &str, &str)] = &[
        ("01_1", "four pulses", "G-F1.pulses", "4"),
        ("01_1", "Up, up, up, down", "G-F1.code_a", "1,1,1,-1"),
        ("01_3", "add a block", "G-F1.key.agree", "1"),
        ("01_3", "take one away", "G-F1.key.disagree", "-1"),
        ("01_4", "all four agree", "G-F2.a.peak", "4"),
        ("01_4", "A score of four", "G-F2.a.peak", "4"),
        ("01_5", "three pairs line up", "G-F2.a.one_off.pairs", "3"),
        ("01_5", "Two agree", "G-F2.a.one_off.agree", "2"),
        ("01_5", "one disagrees", "G-F2.a.one_off.disagree", "1"),
        ("01_5", "A score of one", "G-F2.a.one_off.score", "1"),
        ("01_6", "a quarter the echo's size", "G-F2.ghost_fraction", "1/4"),
        ("01_7", "With more than one pulse", "G-X1.far_shift_sizes", "1"),
        ("01_7", "Up, up, down, up", "G-F1.code_b", "1,1,-1,1"),
        ("01_8", "the same size", "G-F2.b.largest_ghost", "1"),
        ("01_8", "at the same shifts", "G-F2.b.ghost_shifts", "-3,-1,1,3"),
        ("01_9", "twice as tall", "G-F3.peak_ratio", "2"),
        ("01_9", "not a single ghost", "G-F3.sum.largest_ghost", "0"),
        ("01_10", "one step", "G-F4.moved.step", "1"),
        ("01_10", "ghosts return", "G-F4.moved.largest_ghost", "1"),
        ("02_1", "two things", "G-F5.echoes", "2"),
        ("02_1", "three steps farther", "G-F5.quiet_step", "3"),
        ("02_2", "two echoes", "G-F5.echoes", "2"),
        ("02_3", "At three steps", "G-F5.quiet_step", "3"),
        ("02_3", "the score is zero", "G-F6.a.at_quiet", "0"),
        ("02_4", "Zero is not empty", "G-F6.a.zero_at_echo", "3"),
        ("02_4", "at three steps", "G-F5.quiet_step", "3"),
        ("02_5", "minus four", "G-F6.a.loud_part_at_quiet", "-4"),
        ("02_5", "plus four", "G-F6.a.quiet_part_at_quiet", "4"),
        ("02_6", "four tall", "G-F6.a.loud_ghosts_one_step", "4,4"),
        ("02_6", "as tall as the quiet echo", "G-F6.a.quiet_part_at_quiet", "4"),
        ("02_7", "three steps", "G-F5.quiet_step", "3"),
        ("02_8", "Eight", "G-F7.b.at_quiet", "8"),
        ("02_8", "doubles it", "G-F7.b.over_quiet", "2"),
        ("02_9", "Thirty-two at zero", "G-F8.sum.at_loud", "32"),
        ("02_9", "eight at three", "G-F8.sum.at_quiet", "8"),
        ("02_9", "zero everywhere else", "G-F8.sum.largest_ghost", "0"),
        ("02_9", "eight times over", "G-F8.magnification", "8"),
        ("02_10", "Sending pattern A twice", "G-F9.a_twice.sends", "2"),
        ("02_10", "stays hidden", "G-F9.a_twice.at_quiet", "0"),
        ("03_1", "one pulse", "G-F10.seed_length", "1"),
        ("03_4", "one step", "G-F10.shift", "1"),
        ("03_5", "Inside a half", "G-F10.length_factor", "2"),
        ("03_6", "exactly one flipped pulse", "G-F10.flipped_in_crossing", "1"),
        ("03_8", "twice the old pair's rows", "G-F10.factor", "2"),
        ("03_9", "twice as long", "G-F10.length_factor", "2"),
        ("04_2", "one step", "G-F14.moved.step", "1"),
        ("04_4", "three and five", "G-F14.moved.at_echoes", "3,5"),
        ("04_4", "six shifts", "G-F14.moved.ghost_count", "6"),
        ("04_5", "the second send's echo", "G-F18.sends", "2"),
        ("04_6", "half a turn", "G-F15.phase.flip_factor", "-1"),
        ("04_8", "Cancelled out", "G-F15.phase.at_echo", "0"),
        ("04_8", "every ghost doubles", "G-F15.phase.ghost_ratio", "2"),
        ("04_9", "four-pulse pair", "G-F16.pulses", "4"),
        ("04_9", "eight times the second scene", "G-F16.factor", "8"),
        ("04_11", "two bad echoes", "G-F18.sends", "2"),
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
            "zero", "one", "two", "three", "four", "quarter", "twice", "second", "half", "eight", "thirty", "minus",
            "plus", "doubles",
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

    /// The spoken patterns are the drawn ones, pulse by pulse, and B's ghosts
    /// are at A's shifts.
    #[test]
    fn the_spoken_patterns_are_the_drawn_ones() {
        let pulses = |words: &str| -> Vec<i64> {
            words
                .split(", ")
                .map(|w| if w.eq_ignore_ascii_case("up") { 1 } else { -1 })
                .collect()
        };
        assert_eq!(pulses("Up, up, up, down"), model::CODE_A);
        assert_eq!(pulses("Up, up, down, up"), model::CODE_B);
        let f = facts::facts().unwrap();
        assert_eq!(f.get("G-F2.a.ghost_shifts"), f.get("G-F2.b.ghost_shifts"));
    }

    /// The synthesis input, narration_units.json, was built from this
    /// script: it names the script's hash, and holds every line once, in
    /// order (`python3 -I -B narration_units.py` checks the rest; `--write`
    /// rebuilds it after a script edit).
    #[test]
    fn the_narration_units_are_built_from_this_script() {
        let units = include_str!("../narration_units.json");
        let hash = explainer_score::digest::sha256_hex(SCRIPT.as_bytes());
        assert!(
            units.contains(&format!("\"script_sha256\": \"{hash}\"")),
            "narration_units.json is stale: run python3 -I -B narration_units.py --write"
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
