//! Native FrankenManim draft for the addition-to-multiplication property.
//! Unintegrated, uncompiled, unrendered, unlistened and unreviewed.
//! Proposed 01_1 ends after the separate factors; 01_2 gives the combined
//! factor and rule; 01_3 is the second source paragraph. Original takes stay whole.
use explainer_kit::{Kit, Palette, Timed, ink_style, rect, v};
use fmn::prelude::*;

pub struct AdditionToMultiplication {
    pub kit: Kit,
}

fn appear(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?;
        animations.push(stage.prepare(animation.rt(0.5))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

impl SceneConstruct for AdditionToMultiplication {
    fn name(&self) -> &str { "01_addition_to_multiplication" }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let base = crate::facts::BASE;
        let (first, second) = (crate::facts::FIRST, crate::facts::SECOND);
        let invalid = || fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("integer growth overflow"));
        let first_factor = base.checked_pow(first).ok_or_else(invalid)?;
        let second_factor = base.checked_pow(second).ok_or_else(invalid)?;
        let total = first.checked_add(second).ok_or_else(invalid)?;
        let combined = first_factor.checked_mul(second_factor).ok_or_else(invalid)?;
        if base.checked_pow(total) != Some(combined) {
            return Err(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("growth factors disagree")));
        }

        kit.say(stage, "01_1")?;
        let heading = kit.text(stage, "Add intervals; multiply their effects", 35.0, Palette::ink(), v(0.0, 2.9))?;
        let x0 = -3.8;
        let first_width = f64::from(first);
        let second_width = f64::from(second);
        let first_bar = stage.arena_mut().add(rect(v(x0, 0.5), first_width, 0.42, ink_style(Palette::blue()))?);
        let second_bar = stage.arena_mut().add(rect(v(x0 + first_width, 0.5), second_width, 0.42, ink_style(Palette::gold()))?);
        let first_label = kit.text(stage, &format!("{first} steps: ×{first_factor}"), 26.0, Palette::blue(), v(x0 + first_width / 2.0, 1.35))?;
        let second_label = kit.text(stage, &format!("{second} steps: ×{second_factor}"), 26.0, Palette::gold(), v(x0 + first_width + second_width / 2.0, 1.35))?;
        let question = kit.text(stage, "What is the combined factor?", 28.0, Palette::hero(), v(0.0, -0.6))?;
        appear(stage, &[heading, first_bar, second_bar, first_label, second_label, question])?;
        kit.hold(stage)?;
        stage.wait(3.0)?;

        kit.say(stage, "01_2")?;
        let answer = kit.text(stage, &format!("{first_factor} × {second_factor} = {combined}     ({total} steps)"), 30.0, Palette::green(), v(0.0, -1.5))?;
        appear(stage, &[answer])?;
        kit.until_sentence(stage, "01_2", 2)?;
        let identity = kit.text(stage, "f(x + y) = f(x) × f(y)", 30.0, Palette::ink(), v(0.0, -2.35))?;
        appear(stage, &[identity])?;

        kit.say(stage, "01_3")?;
        kit.until_sentence(stage, "01_3", 1)?;
        let ruler = kit.text(stage, "Logarithmic positions add when the numbers multiply", 26.0, Palette::ink(), v(0.0, -3.1))?;
        appear(stage, &[ruler])?;
        kit.finish(stage)
    }
}
