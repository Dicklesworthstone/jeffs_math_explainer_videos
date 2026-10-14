//! Proofs You Can See — Video I of the corpus: nine visual proofs and one
//! picture that lies. A pattern that breaks (regions of a circle), squares
//! built from L-shaped layers, two staircases, the cubes cut into slabs that
//! tile a square, three stacks that fill a box, a square halved and a square
//! quartered, the cut square and cube, four rectangles round a hole,
//! Pythagoras by rearrangement, and the missing-square puzzle, rendered end
//! to end by franken_manim's native Rust front door and the corpus's shared
//! kit.
//!
//! ```text
//! proofs facts                         # every number the video shows or says (key<TAB>value)
//! proofs script                        # the narration (generated from docs/storyboard.md)
//! proofs render all|<chapter>... [--res 960x540] [--scratch-voice] [--out DIR] ...
//! proofs help                          # the full option list
//! ```
//!
//! Every chapter is a real scene (`chapters`), built on the shared board,
//! card and cube kits and tested against the model and the facts.

mod chapters;
mod facts;
mod model;
mod score;

use explainer_kit::Explainer;

/// The narration, generated from the storyboard by `tools/storyboard.py`.
const SCRIPT: &str = include_str!("../script.tsv");
/// The storyboard: words, pictures, holds and listen windows per beat (the
/// tests check the registry, the script and the quizzes against it).
#[cfg(test)]
pub(crate) const STORYBOARD: &str = include_str!("../docs/storyboard.md");

const EXPLAINER: Explainer = Explainer {
    crate_name: "visual_proofs",
    script: SCRIPT,
    chapters: chapters::registry,
    facts: facts::facts,
    glyph_text: chapters::glyphs::TEXT,
    glyph_tex: chapters::glyphs::TEX,
    glyph_mono: chapters::glyphs::MONO,
    identity: score::identity,
    narration_gain_db: explainer_kit::narration::GAIN_DB,
    line_gaps: explainer_kit::narration::LineGaps::Constant,
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

    /// Every quiz in `model::QUIZZES` is a storyboard beat that asks a
    /// question and then counts down for exactly the quiz's seconds, and
    /// every countdown in the storyboard is one of those quizzes.
    #[test]
    fn every_countdown_is_a_quiz_and_every_quiz_has_its_countdown() {
        let board = explainer_kit::storyboard::parse(STORYBOARD).unwrap();
        let beats: Vec<_> = board.iter().flat_map(|c| &c.beats).collect();
        for q in model::QUIZZES {
            let asked: Vec<_> = beats.iter().filter(|b| b.id == q.line).collect();
            assert_eq!(asked.len(), 1, "{}: one beat asks the quiz", q.line);
            assert!(asked[0].predict(), "{}: a quiz asks a question", q.line);
            assert_eq!(
                asked[0].listen_s(),
                Some(f64::from(q.seconds)),
                "{}: the countdown",
                q.line
            );
        }
        let counted: Vec<&str> = beats
            .iter()
            .filter(|b| b.hold.contains("countdown"))
            .map(|b| b.id.as_str())
            .collect();
        let quizzes: Vec<&str> = model::QUIZZES.iter().map(|q| q.line).collect();
        assert_eq!(counted, quizzes);
    }
}
