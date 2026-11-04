//! A marked square turning in its frame. Silent draft; no final timing claim.
use explainer_kit::{Kit, Palette, Timed, hex, ink_style, rect, stroke_style, v};
use fmn::prelude::*;

pub struct Square {
    pub kit: Kit,
}

const CORNERS: [(f64, f64); 4] = [(1.48, 1.48), (1.48, -1.48), (-1.48, -1.48), (-1.48, 1.48)];

fn fade(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?;
        animations.push(stage.prepare(animation.rt(0.5))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

fn turn(stage: &mut Stage<'_>, square: Mob, markers: &[(Mob, Mob)], completed: u32) -> fmn::Result<()> {
    if markers.len() != CORNERS.len() {
        return Err(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(
            "square rotation requires four corner markers",
        )));
    }
    for step in 1..=12 {
        let t = f64::from(step) / 12.0;
        let mut animations: Vec<Box<dyn Animation>> = Vec::new();
        let delta = std::f64::consts::FRAC_PI_2 / 12.0;
        let rotation = square
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.12),
                ..AnimateArgs::default()
            })?
            .rotate(delta)?;
        animations.push(stage.prepare(rotation)?);
        for (&(dot, label), (x, y)) in markers.iter().zip(CORNERS) {
            let angle = std::f64::consts::FRAC_PI_2 * (f64::from(completed) + t);
            let destination = v(
                -2.6 + x * angle.cos() - y * angle.sin(),
                x * angle.sin() + y * angle.cos(),
            );
            for mob in [dot, label] {
                let movement = mob
                    .animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(0.12),
                        ..AnimateArgs::default()
                    })?
                    .move_to(destination, ORIGIN)?;
                animations.push(stage.prepare(movement)?);
            }
        }
        stage.play_prepared(animations)?;
    }
    Ok(())
}

impl SceneConstruct for Square {
    fn name(&self) -> &str {
        "04_square"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        kit.say(stage, "04_1")?;
        let heading = kit.text(stage, "Mark the corners", 36.0, Palette::ink(), v(0.0, 2.85))?;
        let frame = stage
            .arena_mut()
            .add(rect(v(-4.35, -1.75), 3.5, 3.5, stroke_style(Palette::dim(), 2.0))?);
        let square = stage
            .arena_mut()
            .add(rect(v(-4.35, -1.75), 3.5, 3.5, stroke_style(Palette::hero(), 4.0))?);
        let colors = [Palette::blue(), Palette::gold(), Palette::green(), Palette::hero()];
        let mut all = vec![heading, frame, square];
        let mut markers = Vec::new();
        for (i, &(x, y)) in CORNERS.iter().enumerate() {
            let position = v(-2.6 + x, y);
            let dot = stage.arena_mut().add(
                Circle::new()
                    .radius(0.23)
                    .style(ink_style(colors[i]))
                    .build()
                    .moved_to(position),
            );
            let label = kit.text(stage, &(i + 1).to_string(), 27.0, hex("#0E1116"), position)?;
            all.extend([dot, label]);
            markers.push((dot, label));
        }
        fade(stage, &all)?;
        kit.until_sentence(stage, "04_1", 2)?;
        turn(stage, square, &markers, 0)?;
        kit.say(stage, "04_2")?;
        let question = kit.text(stage, "How many quarter-turns", 28.0, Palette::hero(), v(2.8, 0.8))?;
        let question2 = kit.text(stage, "return every corner?", 28.0, Palette::hero(), v(2.8, 0.2))?;
        fade(stage, &[question, question2])?;
        kit.hold(stage)?;
        stage.wait(3.0)?;
        kit.say(stage, "04_3")?;
        for completed in 1..4 {
            turn(stage, square, &markers, completed)?;
        }
        let answer = kit.text(
            stage,
            "4 quarter-turns = 1 full turn",
            30.0,
            Palette::green(),
            v(2.8, -1.1),
        )?;
        fade(stage, &[answer])?;
        kit.finish(stage)
    }
}
