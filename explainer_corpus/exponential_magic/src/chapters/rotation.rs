//! Native FrankenManim draft: the imaginary exponential traces a circle.
//! Unintegrated, uncompiled, unrendered, unlistened and unreviewed.
//! Proposed 05_1 ends after the first sentence; 05_2 contains the remainder.
//! Both retained ElevenLabs takes remain whole and unselected.
use explainer_kit::{Kit, Palette, Timed, ink_style, stroke_style, v};
use fmn::prelude::*;

pub struct ImaginaryRotation {
    pub kit: Kit,
}

fn fade(stage: &mut Stage<'_>, mobs: &[Mob], incoming: bool) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = if incoming {
            fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?
        } else {
            fade_out(stage.arena_mut(), mob, ORIGIN, 1.0)?
        };
        animations.push(stage.prepare(animation.rt(0.5))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

impl SceneConstruct for ImaginaryRotation {
    fn name(&self) -> &str { "05_imaginary_rotation" }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let center = (-3.0_f64, 0.0_f64);
        let radius = 1.8_f64;
        let samples = 24_u32;
        kit.say(stage, "05_1")?;
        let heading = kit.text(stage, "What happens when the exponent turns?", 34.0, Palette::ink(), v(0.0, 2.9))?;
        let start = v(center.0 + radius, center.1);
        let dot = Circle::new().radius(0.12).style(ink_style(Palette::hero())).build().moved_to(start);
        let dot = stage.arena_mut().add(dot);
        let question = kit.text(stage, "Where will the output travel?", 27.0, Palette::hero(), v(2.0, 0.7))?;
        fade(stage, &[heading, dot, question], true)?;
        kit.hold(stage)?;
        stage.wait(3.0)?;

        kit.say(stage, "05_2")?;
        fade(stage, &[question], false)?;
        let ring = Circle::new().radius(radius).style(stroke_style(Palette::dim(), 3.0)).build().moved_to(v(center.0, center.1));
        let ring = stage.arena_mut().add(ring);
        let formula = kit.text(stage, "e^(it) = cos t + i sin t", 27.0, Palette::blue(), v(2.0, 1.1))?;
        let unit = kit.text(stage, "One unit-radius circle", 23.0, Palette::dim(), v(2.0, 0.2))?;
        fade(stage, &[ring, formula, unit], true)?;
        stage.scene_mut().bring_to_front(&[dot])?;
        for step in 1..=samples {
            let t = std::f64::consts::TAU * f64::from(step) / f64::from(samples);
            let destination = v(center.0 + radius * t.cos(), center.1 + radius * t.sin());
            let animation = dot.animate().set_anim_args(AnimateArgs {
                run_time: Some(0.15), ..AnimateArgs::default()
            })?.move_to(destination, ORIGIN)?;
            let prepared = stage.prepare(animation)?;
            stage.play_prepared(vec![prepared])?;
        }
        let end = v(start[0], start[1] + 0.9);
        let tangent = stage.arena_mut().add(Line::new(start, end).style(stroke_style(Palette::green(), 4.0)).build()?);
        let left_head = stage.arena_mut().add(Line::new(v(end[0] - 0.12, end[1] - 0.18), end).style(stroke_style(Palette::green(), 4.0)).build()?);
        let right_head = stage.arena_mut().add(Line::new(v(end[0] + 0.12, end[1] - 0.18), end).style(stroke_style(Palette::green(), 4.0)).build()?);
        let direction = kit.text(stage, "Multiply by i: turn the direction", 23.0, Palette::green(), v(2.0, -0.8))?;
        fade(stage, &[tangent, left_head, right_head, direction], true)?;
        kit.finish(stage)
    }
}
