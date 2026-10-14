//! The Arithmetic Where 3 × 7 = 9 — Video B of the corpus: the finite field
//! under RaptorQ fountain codes, rendered end to end by franken_manim's
//! native Rust front door and the corpus's shared kit.
//!
//! ```text
//! gf256 facts                         # every number the video shows or says (key<TAB>value)
//! gf256 script                        # the narration (generated from docs/storyboard.md)
//! gf256 render all|<chapter>... [--res 960x540] [--scratch-voice] [--out DIR] ...
//! gf256 help                          # the full option list
//! ```
//!
//! Chapters start as animatics (`explainer_kit::animatic`): each speaks its
//! storyboard lines over a card describing the planned picture, so drafts,
//! timelines, the score and the master run before the real scene exists.

mod chapters;
// How cards get solved (13–16): lands ahead of the chapters that use all of
// it; drop the allow when 13–16 are built.
#[allow(dead_code)]
mod decoding;
mod facts;
// The shared model lands ahead of the chapters that use all of it; drop the
// allow when all 17 chapters are built.
#[allow(dead_code)]
mod model;

use explainer_kit::Explainer;

/// The narration, generated from the storyboard by `tools/storyboard.py`.
const SCRIPT: &str = include_str!("../script.tsv");
/// The storyboard: words, pictures, holds and listen windows per beat.
pub(crate) const STORYBOARD: &str = include_str!("../docs/storyboard.md");

const EXPLAINER: Explainer = Explainer {
    crate_name: "raptorq_gf256",
    script: SCRIPT,
    chapters: chapters::registry,
    facts: facts::facts,
    glyph_text: &[],
    glyph_tex: &[],
    glyph_mono: &[],
    identity: explainer_score::score::Identity::raptorq_gf256,
    narration_gain_db: explainer_kit::narration::GAIN_DB,
    line_gaps: explainer_kit::narration::LineGaps::Constant,
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

    #[test]
    fn the_storyboard_the_registry_and_the_script_agree() {
        explainer_kit::storyboard::check_video(STORYBOARD, SCRIPT, &keys()).unwrap();
        let script = Script::parse(SCRIPT).unwrap();
        check_script_owners(&script, &keys(), None).unwrap();
    }

    #[test]
    fn the_facts_pass_their_own_pins() {
        let f = facts::facts().unwrap();
        f.self_check().unwrap();
        assert!(!f.is_empty());
    }
}
