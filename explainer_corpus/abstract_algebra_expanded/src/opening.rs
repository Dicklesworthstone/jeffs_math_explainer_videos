//! First visual experiment for the expanded film; no narration or review yet.
use explainer_kit::{Kit, Palette, Timed, ink_style, stroke_style, v};
use fmn::prelude::*;

pub struct ClockOpening {
    pub kit: Kit,
}

fn fade(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?;
        animations.push(stage.prepare(animation.rt(0.5))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

fn clock_position(hour: i32, radius: f64) -> Vec3 {
    let angle = std::f64::consts::FRAC_PI_2 - std::f64::consts::TAU * f64::from(hour.rem_euclid(12)) / 12.0;
    v(-2.8 + radius * angle.cos(), radius * angle.sin())
}

impl SceneConstruct for ClockOpening {
    fn name(&self) -> &str {
        "01_clock"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        kit.say(stage, "01_1")?;
        let heading = kit.text(stage, "11 o'clock + 5 hours", 36.0, Palette::ink(), v(0.0, 2.85))?;
        let ring = stage.arena_mut().add(
            Circle::new()
                .radius(1.85)
                .style(stroke_style(Palette::dim(), 3.0))
                .build()
                .moved_to(v(-2.8, 0.0)),
        );
        let mut static_objects = vec![heading, ring];
        for hour in 0..12 {
            let position = clock_position(hour, 1.22);
            let label = if hour == 0 { 12 } else { hour };
            static_objects.push(kit.text(stage, &label.to_string(), 22.0, Palette::ink(), position)?);
        }
        let question = kit.text(stage, "Where will the marker land?", 30.0, Palette::hero(), v(2.8, 1.0))?;
        static_objects.push(question);
        fade(stage, &static_objects)?;
        let marker = stage.arena_mut().add(
            Circle::new()
                .radius(0.14)
                .style(ink_style(Palette::blue()))
                .build()
                .moved_to(clock_position(11, 0.82)),
        );
        fade(stage, &[marker])?;
        kit.hold(stage)?;
        stage.wait(3.0)?;
        kit.say(stage, "01_2")?;
        for step in 1..=5 {
            let animation = marker
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.65),
                    ..AnimateArgs::default()
                })?
                .move_to(clock_position(11 + step, 0.82), ORIGIN)?;
            stage.play(animation)?;
        }
        kit.say(stage, "01_3")?;
        let answer = kit.text(
            stage,
            "16 and 4: same clock position",
            27.0,
            Palette::green(),
            v(2.8, -0.2),
        )?;
        fade(stage, &[answer])?;
        kit.say(stage, "01_4")?;
        let distinction = kit.text(stage, "Equal as integers? No.", 27.0, Palette::ink(), v(2.8, -1.1))?;
        fade(stage, &[distinction])?;
        kit.finish(stage)
    }
}
