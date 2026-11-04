//! Draft native FrankenManim scene for the logarithm's winding ambiguity.
//! Not integrated, compiled, rendered, narrated, listened to, or reviewed.
//! Proposed cues 02_1 and 02_2 are text boundaries, not verified audio cuts.
use explainer_kit::{Kit, Palette, Timed, ink_style, stroke_style, v};
use fmn::prelude::*;

pub struct Winding {
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

impl SceneConstruct for Winding {
    fn name(&self) -> &str { "02_winding" }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let center = (-3.1_f64, 0.0_f64);
        let radius = 1.8_f64;
        let steps = 24_u32;
        let start = v(center.0 + radius, center.1);
        let heading = kit.text(stage, "One point, more than one input", 34.0, Palette::ink(), v(0.0, 2.9))?;
        let circle = stage.arena_mut().add(Circle::new().radius(radius).style(stroke_style(Palette::dim(), 3.0)).build().moved_to(v(center.0, center.1)));
        let point = stage.arena_mut().add(Circle::new().radius(0.14).style(ink_style(Palette::blue())).build().moved_to(start));
        let input = kit.text(stage, "Start: exp(0) = 1", 27.0, Palette::blue(), v(2.2, 1.2))?;
        let output = kit.text(stage, "Track exp(it)", 27.0, Palette::ink(), v(2.2, 0.3))?;
        let question = kit.text(stage, "After one full turn?", 27.0, Palette::hero(), v(2.2, -0.7))?;
        appear(stage, &[heading, circle, point, input, output, question])?;
        // Predict before the narration reveals that one turn returns to one.
        stage.wait(3.0)?;
        kit.say(stage, "02_1")?;
        // Scratch sentence starts are estimated; real speech requires listening.
        kit.until_sentence(stage, "02_1", 1)?;
        for step in 1..=steps {
            let theta = std::f64::consts::TAU * f64::from(step) / f64::from(steps);
            let destination = v(center.0 + radius * theta.cos(), center.1 + radius * theta.sin());
            let animation = point.animate().set_anim_args(AnimateArgs { run_time: Some(0.15), ..AnimateArgs::default() })?.move_to(destination, ORIGIN)?;
            let prepared = stage.prepare(animation)?;
            stage.play_prepared(vec![prepared])?;
        }
        let another_input = kit.text(stage, "End: exp(2πi) = 1", 27.0, Palette::green(), v(2.2, -1.5))?;
        let same = kit.text(stage, "Same output", 27.0, Palette::green(), v(2.2, -2.3))?;
        appear(stage, &[another_input, same])?;

        kit.say(stage, "02_2")?;
        let formula = kit.text(stage, "log z = ln r + i(θ + 2πk)", 28.0, Palette::hero(), v(0.0, -3.0))?;
        let domain = kit.text(stage, "z ≠ 0; k is any integer", 23.0, Palette::ink(), v(2.0, 2.0))?;
        appear(stage, &[formula, domain])?;
        kit.finish(stage)
    }
}
