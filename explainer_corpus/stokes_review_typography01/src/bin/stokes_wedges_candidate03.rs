//! Isolated registry overlay for a narrated typography review, not a master.
mod opening {
    use explainer_kit::Timed;
    use fmn::prelude::*;
    pub(super) fn fade(stage: &mut Stage<'_>, mobs: &[Mob], incoming: bool) -> fmn::Result<()> {
        let mut animations: Vec<Box<dyn Animation>> = Vec::new();
        for &mob in mobs {
            let animation = if incoming { fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)? }
                else { fade_out(stage.arena_mut(), mob, ORIGIN, 1.0)? };
            animations.push(stage.prepare(animation.rt(0.5))?);
        }
        stage.play_prepared(animations)?;
        Ok(())
    }
}
#[path = "../chapters/mod.rs"]
mod original_chapters;
#[path = "../facts.rs"]
mod facts;
#[path = "../score.rs"]
mod score;
#[path = "/Users/jemanuel/projects/jeffs_math_explainer_videos/narration_lab/third-wave-20261024/root/elevenlabs-stokes-20261102/wedges-tex-candidate02.rs"]
mod candidate;
use explainer_kit::{Chapter, Explainer};
fn chapters() -> Vec<Chapter> {
    let mut chapters = original_chapters::registry();
    let target = chapters.iter_mut().find(|c| c.key == "10_wedge_cancellation").expect("chapter 10");
    target.make = |kit| Box::new(candidate::Wedges { kit });
    chapters
}
const EXPLAINER: Explainer = Explainer {
    crate_name: "stokes_theorem_visual",
    script: include_str!("../../script.tsv"),
    chapters,
    facts: facts::facts,
    glyph_text: original_chapters::TEXT,
    glyph_tex: &[],
    glyph_mono: &[],
    identity: score::identity,
    narration_gain_db: explainer_kit::narration::GAIN_DB,
    line_gaps: explainer_kit::narration::LineGaps::Constant,
    line_owner: None,
    thumbnail: None,
};
fn main() {
    std::process::exit(explainer_kit::cli::run_with_contract(
        &EXPLAINER, std::env::args().skip(1), std::path::Path::new(env!("CARGO_MANIFEST_DIR")),
    ));
}
