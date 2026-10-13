//! How to Find a Lost Submarine — Video H of the corpus: Bayesian search
//! from the USS Scorpion to the Coast Guard. A map of chances built from
//! weighted stories, searches that can miss, Bayes' rule after a search
//! comes back empty, where to look next, and what a drifting target does to
//! all of it, rendered end to end by franken_manim's native Rust front door
//! and the corpus's shared kit.
//!
//! ```text
//! search facts                         # every number the video shows or says (key<TAB>value)
//! search script                        # the narration (generated from docs/storyboard.md)
//! search render all|<chapter>... [--res 960x540] [--scratch-voice] [--out DIR] ...
//! search help                          # the full option list
//! ```
//!
//! Chapters start as animatics (`explainer_kit::animatic`): each speaks its
//! storyboard lines over a card describing the planned picture, so drafts
//! and timelines run before the real scene exists.

mod chapters;
mod facts;
mod model;
mod score;

use explainer_kit::Explainer;

/// The narration, generated from the storyboard by `tools/storyboard.py`.
const SCRIPT: &str = include_str!("../script.tsv");

const EXPLAINER: Explainer = Explainer {
    crate_name: "bayesian_search",
    script: SCRIPT,
    chapters: chapters::registry,
    facts: facts::facts,
    glyph_text: chapters::glyphs::TEXT,
    glyph_tex: chapters::glyphs::TEX,
    glyph_mono: chapters::glyphs::MONO,
    identity: score::identity,
    narration_gain_db: explainer_kit::narration::GAIN_DB,
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

    /// The storyboard: words, pictures, holds and listen windows per beat.
    const STORYBOARD: &str = include_str!("../docs/storyboard.md");

    fn keys() -> Vec<&'static str> {
        chapters::registry().iter().map(|c| c.key).collect()
    }

    #[test]
    fn the_storyboard_the_registry_and_the_script_agree() {
        explainer_kit::storyboard::check_video(STORYBOARD, SCRIPT, &keys()).unwrap();
        let script = Script::parse(SCRIPT).unwrap();
        check_script_owners(&script, &keys(), None).unwrap();
    }
}
