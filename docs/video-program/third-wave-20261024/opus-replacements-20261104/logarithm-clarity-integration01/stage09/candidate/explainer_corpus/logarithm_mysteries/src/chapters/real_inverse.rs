//! Draft native FrankenManim scene for real logarithms and multiplication.
//! Unintegrated, uncompiled, unrendered, unlistened and unreviewed.
//! Proposed cue boundaries 01_1 and 01_2 are not verified audio cuts.
use explainer_kit::{Kit, Palette, Timed, ink_style, rect, v};
use fmn::prelude::*;

pub struct RealInverse { pub kit: Kit }

fn appear(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?;
        animations.push(stage.prepare(animation.rt(0.5))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

fn disappear(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = fade_out(stage.arena_mut(), mob, ORIGIN, 1.0)?;
        animations.push(stage.prepare(animation.rt(0.4))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

impl SceneConstruct for RealInverse {
    fn name(&self) -> &str { "01_real_inverse" }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        kit.say(stage, "01_1")?;
        let heading = kit.text(stage, "Which input makes this output?", 34.0, Palette::ink(), v(0.0, 2.9))?;
        let left_label = kit.text(stage, "exponent x", 24.0, Palette::blue(), v(-3.0, 1.9))?;
        let right_label = kit.text(stage, "positive output exp(x)", 24.0, Palette::green(), v(3.0, 1.9))?;
        let left_line = stage.arena_mut().add(rect(v(-5.5,0.3),5.0,0.035,ink_style(Palette::dim()))?);
        let right_line = stage.arena_mut().add(rect(v(0.5,0.3),5.0,0.035,ink_style(Palette::dim()))?);
        let lzero = kit.text(stage, "0", 25.0, Palette::blue(), v(-4.6, -0.1))?;
        let lone = kit.text(stage, "1", 25.0, Palette::blue(), v(-1.4, -0.1))?;
        let rone = kit.text(stage, "1", 25.0, Palette::green(), v(1.4, -0.1))?;
        let re = kit.text(stage, "e", 25.0, Palette::green(), v(4.6, -0.1))?;
        let question = kit.text(stage, "What input gives e?", 27.0, Palette::hero(), v(0.0,-1.4))?;
        appear(stage, &[heading,left_label,right_label,left_line,right_line,lzero,lone,rone,re])?;
        kit.until_sentence(stage, "01_1", 1)?;
        let positive = kit.text(stage, "Exponential outputs stay positive", 28.0, Palette::green(), v(0.0,-0.9))?;
        appear(stage, &[positive])?;
        kit.until_sentence(stage, "01_1", 2)?;
        let unique = kit.text(stage, "Each positive output has one input", 28.0, Palette::blue(), v(0.0,-1.6))?;
        appear(stage, &[unique])?;
        kit.until_sentence(stage, "01_1", 3)?;
        disappear(stage, &[positive,unique])?;
        appear(stage, &[question])?;
        kit.hold(stage)?;
        stage.wait(3.0)?;
        kit.say(stage, "01_2")?;
        let answer = kit.text(stage, "ln(e) = 1; ln(1) = 0", 29.0, Palette::green(), v(0.0,-2.2))?;
        let domain = kit.text(stage, "For real logs, the input must be positive", 23.0, Palette::ink(), v(0.0,-3.0))?;
        appear(stage, &[answer,domain])?;
        kit.say(stage, "01_3")?;
        let law = kit.text(stage, "ln(ab) = ln(a) + ln(b),  a,b > 0", 26.0, Palette::hero(), v(0.0,1.0))?;
        appear(stage, &[law])?;
        kit.finish(stage)
    }
}
