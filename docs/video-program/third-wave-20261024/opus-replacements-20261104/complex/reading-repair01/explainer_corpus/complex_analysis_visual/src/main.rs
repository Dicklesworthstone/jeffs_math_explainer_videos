//! "The Hidden Geometry of Complex Functions": the Opus replacement candidate
//! for complex_analysis_visual (bead 87do.39). Uncompiled, unrendered and
//! unnarrated; narration is to be ElevenLabs Oliver, synthesized by root.
//!
//! The film follows one bump, 1/(1 + x²), from the real line into the plane:
//! colour pictures of functions, the complex derivative and why one demand
//! made on an open region is rigid, loop sums, Cauchy's formula, counting
//! zeros and poles by turns, residues, a real integral done by a loop, and
//! what continuation can and cannot do. `docs/storyboard.md` is the source
//! of the narration (`script.tsv`) and of every chapter's beats; all fifteen
//! chapters are native scenes.

mod chapters;
mod facts;
mod model;
mod score;

use explainer_kit::Explainer;

/// The narration, generated from the storyboard's words column.
pub(crate) const SCRIPT: &str = include_str!("../script.tsv");
/// The storyboard: every chapter is a native scene now, so only the tests
/// read it (against the registry, the script, the prediction and the reading windows).
#[cfg(test)]
pub(crate) const STORYBOARD: &str = include_str!("../docs/storyboard.md");

const EXPLAINER: Explainer = Explainer {
    crate_name: "complex_analysis_visual",
    script: SCRIPT,
    chapters: chapters::registry,
    facts: facts::facts,
    glyph_text: chapters::TEXT,
    glyph_tex: chapters::TEX,
    glyph_mono: chapters::MONO,
    identity: score::identity,
    narration_gain_db: explainer_kit::narration::GAIN_DB,
    // as canonical; lie_robotics uses Thought gaps, root's choice with the Oliver takes
    line_gaps: explainer_kit::narration::LineGaps::Constant,
    line_owner: None,
    thumbnail: Some(chapters::thumbnail::make),
};

fn main() {
    std::process::exit(explainer_kit::cli::run_with_contract(
        &EXPLAINER,
        std::env::args().skip(1),
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")),
    ));
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

    /// Every ⟨predict⟩ beat has a listen window, and the scenes count down
    /// exactly that long (`chapters::PREDICTS`).
    #[test]
    fn every_prediction_has_its_window() {
        let board = explainer_kit::storyboard::parse(STORYBOARD).unwrap();
        let asked: Vec<(String, f64)> = board
            .iter()
            .flat_map(|c| &c.beats)
            .filter(|b| b.predict())
            .map(|b| (b.id.clone(), b.listen_s().expect("a predict beat has a listen window")))
            .collect();
        let built: Vec<(String, f64)> = chapters::PREDICTS.iter().map(|&(id, s)| (id.to_owned(), s)).collect();
        assert_eq!(
            built, asked,
            "chapters::PREDICTS must list the storyboard's predictions, in order"
        );
    }

    /// Every `read N s` in the storyboard's hold column is a reading window of that length,
    /// and the scenes wait exactly that long (`chapters::READING_WINDOWS`).
    #[test]
    fn every_reading_window_is_in_the_storyboard() {
        let board = explainer_kit::storyboard::parse(STORYBOARD).unwrap();
        let planned: Vec<(String, f64)> = board
            .iter()
            .flat_map(|c| &c.beats)
            .filter_map(|b| {
                let seconds = b.hold.strip_prefix("read ")?.strip_suffix(" s")?;
                Some((b.id.clone(), seconds.parse::<f64>().unwrap()))
            })
            .collect();
        let built: Vec<(String, f64)> = chapters::READING_WINDOWS
            .iter()
            .map(|&(id, s)| (id.to_owned(), s))
            .collect();
        assert_eq!(
            built, planned,
            "chapters::READING_WINDOWS must list the storyboard's reading windows, in order"
        );
    }
}
