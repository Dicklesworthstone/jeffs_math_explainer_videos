//! Proposed native proof typography; unintegrated and not visually reviewed.
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;
use super::opening::fade;

pub struct Wedges { pub kit: Kit }
impl SceneConstruct for Wedges {
    fn name(&self) -> &str { "10_wedge_cancellation" }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let title = kit.text(stage, "Why only one derivative survives", 34.0, Palette::ink(), v(0.0, 3.0))?;
        let term = kit.tex(stage, r"-f_2\,dx\wedge dz", 40.0, Palette::hero(), &[], v(0.0, 2.0))?;
        let repeated = kit.tex(stage, r"dx\wedge dx=0\qquad dz\wedge dz=0", 40.0, Palette::red(), &[], v(0.0, 0.9))?;
        fade(stage, &[title, term, repeated], true)?;
        kit.say(stage, "10_1")?;
        kit.until_sentence(stage, "10_1", 2)?;
        let surviving_label = kit.text(stage, "Only dy is missing", 27.0, Palette::blue(), v(-2.1, -0.15))?;
        let surviving = kit.tex(stage, r"\partial_y f_2\,dy", 40.0, Palette::blue(), &[], v(2.0, -0.15))?;
        fade(stage, &[surviving_label, surviving], true)?;
        kit.until_sentence(stage, "10_1", 3)?;
        let signs = kit.tex(stage, r"-\partial_y f_2\,dy\wedge dx\wedge dz=+\partial_y f_2\,dx\wedge dy\wedge dz", 40.0, Palette::green(), &[], v(0.0, -1.2))?;
        let note = kit.text(stage, "One swap cancels the earlier minus", 24.0, Palette::ink(), v(0.0, -2.1))?;
        fade(stage, &[signs, note], true)?;
        kit.hold(stage)?;
        kit.say(stage, "10_2")?;
        fade(stage, &[term, repeated, surviving_label, surviving, signs, note], false)?;
        let general = kit.tex(stage, r"d\omega=(\sum_i\partial_i f_i)\,dx^1\wedge\cdots\wedge dx^n", 42.0, Palette::hero(), &[], v(0.0, 1.2))?;
        let analogy = kit.text(stage, "In 3D the coefficient is divergence", 27.0, Palette::ink(), v(0.0, 0.2))?;
        let next = kit.text(stage, "Next: integrate one coordinate at a time", 26.0, Palette::green(), v(0.0, -1.0))?;
        fade(stage, &[general], true)?;
        kit.until_sentence(stage, "10_2", 1)?;
        fade(stage, &[analogy], true)?;
        kit.until_sentence(stage, "10_2", 2)?;
        fade(stage, &[next], true)?;
        kit.finish(stage)
    }
}
