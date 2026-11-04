//! Native exponential/logarithm bridge; unintegrated and unreviewed.
//! Cue 06_1 is the complete retained source passage, not an audio cut.
use explainer_kit::{Kit, Palette, Timed, ink_style, stroke_style, v};
use fmn::prelude::*;

pub struct Closing { pub kit: Kit }

fn fade(stage: &mut Stage<'_>, mobs: &[Mob], incoming: bool) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = if incoming { fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)? }
            else { fade_out(stage.arena_mut(), mob, ORIGIN, 1.0)? };
        animations.push(stage.prepare(animation.rt(0.5))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

impl SceneConstruct for Closing {
    fn name(&self) -> &str { "06_logarithm_bridge" }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        kit.say(stage, "06_1")?;
        let arrows = [
            kit.text(stage, "Exponential: add inputs, multiply outputs", 32.0, Palette::ink(), v(0.0, 2.5))?,
            kit.text(stage, "x", 38.0, Palette::blue(), v(-3.0, 0.8))?,
            kit.text(stage, "exp", 26.0, Palette::hero(), v(0.0, 1.2))?,
            kit.text(stage, "→", 40.0, Palette::hero(), v(0.0, 0.55))?,
            kit.text(stage, "exp(x) > 0", 34.0, Palette::green(), v(3.0, 0.8))?,
            kit.text(stage, "←", 40.0, Palette::blue(), v(0.0, -0.4))?,
            kit.text(stage, "log", 26.0, Palette::blue(), v(0.0, -1.1))?,
        ];
        fade(stage, &arrows, true)?;
        // The circle belongs to the final sentence; timing is provisional
        // until the full retained narration is actually heard and reviewed.
        kit.until_sentence(stage, "06_1", 5)?;
        fade(stage, &arrows, false)?;
        let circle = stage.arena_mut().add(Circle::new().radius(1.5).style(stroke_style(Palette::dim(), 3.0)).build().moved_to(v(-2.5, 0.0)));
        let end = stage.arena_mut().add(Circle::new().radius(0.12).style(ink_style(Palette::hero())).build().moved_to(v(-1.0, 0.0)));
        let labels = [
            kit.text(stage, "One point, many turns", 35.0, Palette::ink(), v(0.0, 2.6))?,
            kit.text(stage, "e⁰ = e^(2πi) = 1", 28.0, Palette::blue(), v(2.5, 0.5))?,
            kit.text(stage, "The logarithm has more to reveal", 24.0, Palette::green(), v(2.5, -0.7))?,
        ];
        let mut items = vec![circle, end];
        items.extend(labels);
        fade(stage, &items, true)?;
        kit.finish(stage)
    }
}
