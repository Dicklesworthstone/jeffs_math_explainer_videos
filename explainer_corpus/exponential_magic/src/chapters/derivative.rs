//! Native FrankenManim draft for why the normalized exponential is its derivative.
//! Unintegrated, uncompiled, unrendered, unlistened and unreviewed.
//! Proposed cues 02_1 and 02_2 are the two source paragraphs; the retained
//! ElevenLabs recording is not cut, selected or aligned.
use explainer_kit::{Kit, Palette, Timed, ink_style, stroke_style, v};
use fmn::prelude::*;

pub struct NaturalSlope {
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

impl SceneConstruct for NaturalSlope {
    fn name(&self) -> &str { "02_natural_slope" }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let initial_x = 0.0_f64;
        let initial_value = initial_x.exp();
        let initial_slope = initial_value;
        let origin = (-3.25_f64, -1.55_f64);
        let x_scale = 1.2_f64;
        let y_scale = 0.95_f64;
        let map = |x: f64, y: f64| v(origin.0 + x_scale * x, origin.1 + y_scale * y);

        kit.say(stage, "02_1")?;
        let heading = kit.text(stage, "Why choose the natural exponential?", 34.0, Palette::ink(), v(0.0, 2.9))?;
        let x_axis = stage.arena_mut().add(Line::new(map(-1.8, 0.0), map(1.2, 0.0)).style(stroke_style(Palette::dim(), 2.0)).build()?);
        let y_axis = stage.arena_mut().add(Line::new(map(0.0, -0.15), map(0.0, 3.1)).style(stroke_style(Palette::dim(), 2.0)).build()?);
        let mut picture = vec![heading, x_axis, y_axis];
        let segments = 64_u32;
        for i in 0..segments {
            let a = -1.5 + 2.5 * f64::from(i) / f64::from(segments);
            let b = -1.5 + 2.5 * f64::from(i + 1) / f64::from(segments);
            picture.push(stage.arena_mut().add(Line::new(map(a, a.exp()), map(b, b.exp())).style(stroke_style(Palette::blue(), 3.0)).build()?));
        }
        let tangent = stage.arena_mut().add(Line::new(
            map(-0.6, initial_value - 0.6 * initial_slope),
            map(0.6, initial_value + 0.6 * initial_slope),
        ).style(stroke_style(Palette::gold(), 3.0)).build()?);
        let dot = Circle::new().radius(0.09).style(ink_style(Palette::hero())).build().moved_to(map(initial_x, initial_value));
        picture.push(tangent);
        picture.push(stage.arena_mut().add(dot));
        picture.push(kit.text(stage, &format!("f(0) = {initial_value}"), 27.0, Palette::blue(), v(2.2, 1.8))?);
        picture.push(kit.text(stage, &format!("f′(0) = {initial_slope}"), 27.0, Palette::gold(), v(2.2, 1.0))?);
        let question = kit.text(stage, "What happens to the slope away from zero?", 25.0, Palette::hero(), v(0.0, -2.5))?;
        picture.push(question);
        appear(stage, &picture)?;
        kit.hold(stage)?;
        stage.wait(3.0)?;

        kit.say(stage, "02_2")?;
        kit.until_sentence(stage, "02_2", 1)?;
        let multiplication = kit.text(stage, "f(x + h) = f(x) f(h)", 26.0, Palette::ink(), v(2.2, 0.1))?;
        appear(stage, &[multiplication])?;
        kit.until_sentence(stage, "02_2", 2)?;
        let change = kit.text(stage, "f(x + h) − f(x) = f(x)(f(h) − 1)", 25.0, Palette::ink(), v(2.1, -0.8))?;
        appear(stage, &[change])?;
        kit.until_sentence(stage, "02_2", 3)?;
        let derivative = kit.text(stage, "f′(x) = f(x) f′(0) = f(x)", 26.0, Palette::green(), v(1.8, -1.65))?;
        appear(stage, &[derivative])?;
        kit.finish(stage)
    }
}
