//! The first two equivalence lanes. Draft; no audio or final review yet.
use explainer_kit::{Kit, Palette, Timed, stroke_style, v};
use fmn::prelude::*;

pub struct Equivalence {
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

fn clear(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = fade_out(stage.arena_mut(), mob, ORIGIN, 1.0)?;
        animations.push(stage.prepare(animation.rt(0.5))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

fn arrow(stage: &mut Stage<'_>, from: Vec3, to: Vec3, color: Srgb) -> fmn::Result<Mob> {
    let shape = Arrow::new(from, to)
        .buff(0.03)
        .thickness(3.0)
        .style(Style::default().fill(color, 0.9).stroke(color, 3.0, 0.9))
        .build()?;
    Ok(stage.arena_mut().add(shape))
}

impl SceneConstruct for Equivalence {
    fn name(&self) -> &str {
        "02_equivalence"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        kit.say(stage, "02_1")?;
        let title = kit.text(stage, "Sort by clock position", 36.0, Palette::ink(), v(0.0, 2.9))?;
        let left = kit.text(stage, "Position 4", 29.0, Palette::blue(), v(-2.7, 1.9))?;
        let right = kit.text(stage, "Position 5", 29.0, Palette::gold(), v(2.7, 1.9))?;
        let mut lanes = vec![title, left, right];
        fade(stage, &lanes)?;
        kit.say(stage, "02_2")?;
        for (index, value) in crate::LANE_FOUR.into_iter().enumerate() {
            let mob = kit.text(stage, &value.to_string(), 29.0, Palette::blue(), v(0.0, 0.6))?;
            fade(stage, &[mob])?;
            let animation = mob
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.6),
                    ..AnimateArgs::default()
                })?
                .move_to(v(-2.7, 1.0 - index as f64 * 0.65), ORIGIN)?;
            stage.play(animation)?;
            lanes.push(mob);
        }
        for (index, value) in crate::LANE_FIVE.into_iter().enumerate() {
            let mob = kit.text(stage, &value.to_string(), 29.0, Palette::gold(), v(0.0, 0.6))?;
            fade(stage, &[mob])?;
            let animation = mob
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.6),
                    ..AnimateArgs::default()
                })?
                .move_to(v(2.7, 1.0 - index as f64 * 0.65), ORIGIN)?;
            stage.play(animation)?;
            lanes.push(mob);
        }
        kit.say(stage, "02_3")?;
        let rule = kit.text(
            stage,
            "Difference is a multiple of 12",
            26.0,
            Palette::green(),
            v(0.0, -2.4),
        )?;
        fade(stage, &[rule])?;
        lanes.push(rule);
        kit.until_sentence(stage, "02_3", 1)?;
        clear(stage, &lanes)?;
        let heading = kit.text(
            stage,
            "Why this makes equivalence classes",
            33.0,
            Palette::ink(),
            v(0.0, 2.9),
        )?;
        let mut nodes = vec![heading];
        for (x, number) in [(-3.1, 4), (0.0, 16), (3.1, 28)] {
            let ring = stage.arena_mut().add(
                Circle::new()
                    .radius(0.45)
                    .style(stroke_style(Palette::dim(), 3.0))
                    .build()
                    .moved_to(v(x, 1.55)),
            );
            let label = kit.text(stage, &number.to_string(), 27.0, Palette::ink(), v(x, 1.55))?;
            nodes.extend([ring, label]);
        }
        let reflexive = kit.text(stage, "Reflexive: 4 − 4 = 0", 28.0, Palette::blue(), v(0.0, 0.3))?;
        let self_ring = stage.arena_mut().add(
            Circle::new()
                .radius(0.56)
                .style(stroke_style(Palette::blue(), 4.0))
                .build()
                .moved_to(v(-3.1, 1.55)),
        );
        nodes.extend([reflexive, self_ring]);
        fade(stage, &nodes)?;
        kit.until_sentence(stage, "02_3", 2)?;
        let forward = arrow(stage, v(-2.55, 1.78), v(-0.55, 1.78), Palette::gold())?;
        let backward = arrow(stage, v(-0.55, 1.28), v(-2.55, 1.28), Palette::gold())?;
        let symmetric = kit.text(
            stage,
            "Symmetric: 16 − 4 = 12; 4 − 16 = −12",
            26.0,
            Palette::gold(),
            v(0.0, -0.6),
        )?;
        fade(stage, &[forward, backward, symmetric])?;
        kit.say(stage, "02_4")?;
        let next = arrow(stage, v(0.55, 1.55), v(2.55, 1.55), Palette::green())?;
        let combined = arrow(stage, v(-2.55, 0.95), v(2.55, 0.95), Palette::green())?;
        let transitive = kit.text(
            stage,
            "Transitive: (28 − 16) + (16 − 4) = 28 − 4",
            25.0,
            Palette::green(),
            v(0.0, -1.55),
        )?;
        let sum = kit.text(stage, "12 + 12 = 24", 28.0, Palette::green(), v(0.0, -2.45))?;
        fade(stage, &[next, combined, transitive, sum])?;
        kit.finish(stage)
    }
}
