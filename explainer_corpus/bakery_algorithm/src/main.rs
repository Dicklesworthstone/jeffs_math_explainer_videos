//! The Algorithm Lamport Is Proudest Of — Video C of the corpus: Lamport's
//! bakery algorithm, rendered end to end by franken_manim's native Rust
//! front door and the corpus's shared kit.
//!
//! ```text
//! bakery facts                         # every number the video shows or says (key<TAB>value)
//! bakery script                        # the narration (generated from docs/storyboard.md)
//! bakery render all|<chapter>... [--res 960x540] [--scratch-voice] [--out DIR] ...
//! bakery help                          # the full option list
//! ```
//!
//! Chapters start as animatics (`explainer_kit::animatic`): each speaks its
//! storyboard lines over a card describing the planned picture, so drafts,
//! timelines, the score and the master run before the real scene exists.

mod chapters;
mod facts;
mod model;

use explainer_kit::Explainer;

/// The narration, generated from the storyboard by `tools/storyboard.py`.
const SCRIPT: &str = include_str!("../script.tsv");
/// The storyboard: words, pictures, holds and listen windows per beat.
pub(crate) const STORYBOARD: &str = include_str!("../docs/storyboard.md");

const EXPLAINER: Explainer = Explainer {
    crate_name: "bakery_algorithm",
    script: SCRIPT,
    chapters: chapters::registry,
    facts: facts::facts,
    glyph_text: &[],
    glyph_tex: &[],
    glyph_mono: &[],
    identity: explainer_score::score::Identity::bakery_algorithm,
    narration_gain_db: explainer_kit::narration::GAIN_DB,
    line_owner: None,
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
