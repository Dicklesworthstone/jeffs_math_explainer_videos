//! Draft native FrankenManim bookends for the logarithm video.
//! Unintegrated, uncompiled, unrendered, unlistened and unreviewed.
//! 00_1 and 06_1 refer to proposed text cues, not verified audio cuts.
use explainer_kit::{Kit, Palette, Timed, ink_style, rect, v};
use fmn::prelude::*;

pub struct Opening { pub kit: Kit }
pub struct Closing { pub kit: Kit }

fn appear(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?;
        animations.push(stage.prepare(animation.rt(0.5))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

impl SceneConstruct for Opening {
    fn name(&self) -> &str { "00_opening" }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        kit.say(stage, "00_1")?;
        let title = kit.text(stage, "The logarithm: finding the missing input", 34.0, Palette::ink(), v(0.0,2.9))?;
        let first = stage.arena_mut().add(rect(v(-4.8,-0.6),1.7,1.0,ink_style(Palette::blue()))?);
        let second = stage.arena_mut().add(rect(v(3.1,-0.6),1.7,2.0,ink_style(Palette::green()))?);
        let one = kit.text(stage, "1", 26.0, Palette::ink(), v(-3.95,-1.15))?;
        let two = kit.text(stage, "2", 26.0, Palette::ink(), v(3.95,-1.15))?;
        let question = kit.text(stage, "exp(t) = 2. What is t in years?", 29.0, Palette::hero(), v(0.0,1.25))?;
        appear(stage, &[title,first,second,one,two,question])?;
        kit.hold(stage)?;
        stage.wait(3.0)?;
        kit.say(stage, "00_2")?;
        let growth = kit.text(stage, "balance = initial balance × exp(t)", 26.0, Palette::ink(), v(0.0,-1.45))?;
        appear(stage, &[growth])?;
        kit.until_sentence(stage, "00_2", 1)?;
        let answer = kit.text(stage, "t = ln(2) ≈ 0.69 years", 30.0, Palette::green(), v(0.0,-2.15))?;
        let next = kit.text(stage, "What changes when inputs can turn?", 24.0, Palette::blue(), v(0.0,-2.9))?;
        appear(stage, &[answer])?;
        kit.until_sentence(stage, "00_2", 3)?;
        appear(stage, &[next])?;
        kit.finish(stage)
    }
}

impl SceneConstruct for Closing {
    fn name(&self) -> &str { "06_closing" }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        kit.say(stage, "06_1")?;
        let title = kit.text(stage, "What information did the exponential keep?", 32.0, Palette::ink(), v(0.0,2.8))?;
        let real = kit.text(stage, "Real: one positive output, one real input", 33.0, Palette::blue(), v(0.0,1.2))?;
        let complex = kit.text(stage, "Complex: complete turns share an output", 33.0, Palette::green(), v(0.0,0.2))?;
        let matrix = kit.text(stage, "Matrices: the allowed field changes the answer", 33.0, Palette::hero(), v(0.0,-0.8))?;
        let close = kit.text(stage, "An inverse asks what the forward map forgot", 31.0, Palette::ink(), v(0.0,-2.1))?;
        appear(stage, &[title])?;
        appear(stage, &[real])?;
        // Sentence positions in a scratch review are estimates, not heard sync.
        kit.until_sentence(stage, "06_1", 1)?;
        appear(stage, &[complex])?;
        kit.until_sentence(stage, "06_1", 2)?;
        appear(stage, &[matrix])?;
        kit.until_sentence(stage, "06_1", 4)?;
        appear(stage, &[close])?;
        kit.finish(stage)
    }
}
