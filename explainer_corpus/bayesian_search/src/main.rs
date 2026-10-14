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
/// The storyboard: words, pictures, holds and listen windows per beat. Every
/// chapter is a real scene now, so only the tests that check it against the
/// script, the registry and the quizzes read it.
#[cfg(test)]
pub(crate) const STORYBOARD: &str = include_str!("../docs/storyboard.md");

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
