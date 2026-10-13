//! What Does It Mean to Raise e to a Matrix? — Video J of the corpus: the
//! exponential function as one rule (a rate proportional to the amount)
//! seen three ways, the power series with a square matrix in place of the
//! number, why that series converges for every matrix, what e^{tA} means
//! (the flow of a linear system: two tanks, walks on a graph, heat and
//! chance on a network, a two-state quantum system), how it is computed in
//! practice, why e^{A+B} = e^A e^B needs A and B to commute, and how the
//! failure opens the door to Lie groups and Lie algebras, with "your turn"
//! quizzes on a countdown; rendered end to end by franken_manim's native
//! Rust front door and the corpus's shared kit.
//!
//! ```text
//! expm facts                         # every number the video shows or says (key<TAB>value)
//! expm script                        # the narration (generated from docs/storyboard.md)
//! expm render all|<chapter>... [--res 960x540] [--scratch-voice] [--out DIR] ...
//! expm help                          # the full option list
//! ```

mod chapters;
mod facts;
mod model;
mod score;

use explainer_kit::Explainer;

/// The narration, generated from the storyboard by `tools/storyboard.py`.
const SCRIPT: &str = include_str!("../script.tsv");
const EXPLAINER: Explainer = Explainer {
    crate_name: "matrix_exponential",
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
