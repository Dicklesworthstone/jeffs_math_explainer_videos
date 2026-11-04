//! Registry overlay for two isolated proof-typography candidates; not a master.
mod opening {
    use explainer_kit::{Timed, stroke_style, v};
    use fmn::prelude::*;
    pub(super) fn fade(stage: &mut Stage<'_>, mobs: &[Mob], incoming: bool) -> fmn::Result<()> {
        let mut animations: Vec<Box<dyn Animation>> = Vec::new();
        for &mob in mobs {
            let a = if incoming { fade_in(stage.arena_mut(),mob,ORIGIN,1.0)? } else { fade_out(stage.arena_mut(),mob,ORIGIN,1.0)? };
            animations.push(stage.prepare(a.rt(0.5))?);
        }
        stage.play_prepared(animations)?;
        Ok(())
    }
    pub(super) fn line(stage: &mut Stage<'_>, a: (f64,f64), b: (f64,f64), color: Srgb, width: f64) -> fmn::Result<Mob> {
        Ok(stage.arena_mut().add(Line::new(v(a.0,a.1),v(b.0,b.1)).style(stroke_style(color,width)).build()?))
    }
    pub(super) fn arrow(stage: &mut Stage<'_>, a: (f64,f64), b: (f64,f64), color: Srgb) -> fmn::Result<Vec<Mob>> {
        let (dx,dy)=(b.0-a.0,b.1-a.1); let len=dx.hypot(dy);
        if len <= f64::EPSILON { return Err(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("zero-length boundary edge"))); }
        let (ux,uy)=(dx/len,dy/len); let tip=(a.0+0.58*dx,a.1+0.58*dy); let base=(tip.0-0.23*ux,tip.1-0.23*uy); let wing=0.13;
        Ok(vec![line(stage,a,b,color,3.0)?,line(stage,tip,(base.0-wing*uy,base.1+wing*ux),color,3.0)?,line(stage,tip,(base.0+wing*uy,base.1-wing*ux),color,3.0)?])
    }
}
#[path = "../chapters/mod.rs"] mod original_chapters;
#[path = "../facts.rs"] mod facts;
#[path = "../score.rs"] mod score;
#[path = "/Users/jemanuel/projects/jeffs_math_explainer_videos/narration_lab/third-wave-20261024/root/elevenlabs-stokes-20261102/wedges-tex-candidate02.rs"] mod wedges;
#[path = "/Users/jemanuel/projects/jeffs_math_explainer_videos/narration_lab/third-wave-20261024/root/elevenlabs-stokes-20261102/box-tex-candidate03.rs"] mod box_proof;
use explainer_kit::{Chapter, Explainer};
fn chapters() -> Vec<Chapter> {
    let mut chapters = original_chapters::registry();
    chapters.iter_mut().find(|c| c.key == "10_wedge_cancellation").expect("chapter 10").make = |kit| Box::new(wedges::Wedges { kit });
    chapters.iter_mut().find(|c| c.key == "11_box_proof").expect("chapter 11").make = |kit| Box::new(box_proof::BoxProof { kit });
    chapters
}
const EXPLAINER: Explainer = Explainer {
    crate_name: "stokes_theorem_visual", script: include_str!("../../script.tsv"), chapters,
    facts: facts::facts, glyph_text: original_chapters::TEXT, glyph_tex: &[], glyph_mono: &[],
    identity: score::identity, narration_gain_db: explainer_kit::narration::GAIN_DB,
    line_gaps: explainer_kit::narration::LineGaps::Constant, line_owner: None, thumbnail: None,
};
fn main() {
    std::process::exit(explainer_kit::cli::run_with_contract(&EXPLAINER, std::env::args().skip(1), std::path::Path::new(env!("CARGO_MANIFEST_DIR"))));
}
