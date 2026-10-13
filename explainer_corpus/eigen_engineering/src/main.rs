//! The One Idea Behind Bridges, Concert Halls and Economies — Video K of the
//! corpus: what an eigenvector is, on two carts and three springs; how the
//! own patterns of a steel truss untangle its stiffness table and set the
//! notes it rings at; why the air in a room has tones of its own and where a
//! concert hall has too many to count; how the largest eigenvalue of an
//! input-output table says whether an economy can feed itself; and where the
//! idea stops. Rendered end to end by franken_manim's native Rust front door
//! and the corpus's shared kit.
//!
//! ```text
//! eigen facts                         # every number the video shows or says (key<TAB>value)
//! eigen script                        # the narration (generated from docs/storyboard.md)
//! eigen render all|<chapter>... [--res 960x540] [--scratch-voice] [--out DIR] ...
//! eigen help                          # the full option list
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
pub(crate) const SCRIPT: &str = include_str!("../script.tsv");
/// The storyboard: words, pictures, holds and listen windows per beat.
pub(crate) const STORYBOARD: &str = include_str!("../docs/storyboard.md");

const EXPLAINER: Explainer = Explainer {
    crate_name: "eigen_engineering",
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
