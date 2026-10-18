//! Why Turning a Square Can Make the Whole Box Smaller: five equal tiles,
//! the obvious box three wide, and the turn of the middle tile that lets a
//! squeeze reach 2 + 1/√2; what a packing that fits shows, and what it
//! cannot. Rendered by franken_manim's native Rust front door and the
//! corpus's shared kit.
//!
//! ```text
//! squares facts                         # every number the video shows or says (key<TAB>value)
//! squares script                        # the narration (generated from docs/storyboard.md)
//! squares render all|<chapter>... [--res 960x540] [--scratch-voice] [--out DIR] ...
//! squares help                          # the full option list
//! ```
//!
//! The geometry is `model` (exact on numbers a + b√2), checked by `oracle.py`
//! along another route; every chapter draws the model's packings.

mod chapters;
mod facts;
mod model;
mod score;

use explainer_kit::Explainer;

/// The narration, generated from the storyboard by `tools/storyboard.py`.
const SCRIPT: &str = include_str!("../script.tsv");
/// The storyboard: words, pictures and windows per beat (the tests check
/// the registry and the script against it).
#[cfg(test)]
pub(crate) const STORYBOARD: &str = include_str!("../docs/storyboard.md");

const EXPLAINER: Explainer = Explainer {
    crate_name: "square_packing",
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

    #[test]
    fn the_storyboard_the_registry_and_the_script_agree() {
        explainer_kit::storyboard::check_video(STORYBOARD, SCRIPT, &keys()).unwrap();
        let script = Script::parse(SCRIPT).unwrap();
        check_script_owners(&script, &keys(), None).unwrap();
    }

    /// The one prediction beat is the think window the chapter opens, and it
    /// lasts what the storyboard says.
    #[test]
    fn the_think_window_is_the_storyboard_s() {
        let board = explainer_kit::storyboard::parse(STORYBOARD).unwrap();
        let beats: Vec<_> = board.iter().flat_map(|c| &c.beats).collect();
        let asks: Vec<_> = beats.iter().filter(|b| b.predict()).collect();
        assert_eq!(asks.len(), 1);
        assert_eq!(asks[0].id, chapters::the_turned_tile::LINES[3]);
        assert_eq!(asks[0].listen_s(), Some(chapters::the_turned_tile::THINK_S));
    }
}
