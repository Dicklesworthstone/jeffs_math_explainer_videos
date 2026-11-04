//! Compare the images of marked corners under two orders of operations.
use crate::tile_model::{self, Matrix};
use explainer_kit::{Kit, Palette, Timed, ink_style, rect, stroke_style, tween, v};
use fmn::prelude::*;

pub struct Composition {
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

fn state(progress: f64, left: bool) -> Matrix {
    let t = progress.clamp(0.0, 2.0);
    if t == 0.0 {
        return tile_model::IDENTITY;
    }
    if t <= 1.0 {
        if left { tile_model::turn(t) } else { tile_model::flip(t) }
    } else if left {
        tile_model::compose(tile_model::flip(t - 1.0), tile_model::turn(1.0))
    } else {
        tile_model::compose(tile_model::turn(t - 1.0), tile_model::flip(1.0))
    }
}

fn transform(stage: &mut Stage<'_>, progress: Mob, second: bool) -> fmn::Result<()> {
    let animation = tween(progress, if second { 2.0 } else { 1.0 }, 3.0)?;
    let prepared = stage.prepare(animation)?;
    stage.play_prepared(vec![prepared])?;
    Ok(())
}

impl SceneConstruct for Composition {
    fn name(&self) -> &str {
        "05_order"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        kit.say(stage, "05_1")?;
        let title = kit.text(stage, "Same result?", 36.0, Palette::ink(), v(0.0, 2.9))?;
        let left_title = kit.text(stage, "Turn then flip", 30.0, Palette::blue(), v(-3.1, 2.1))?;
        let right_title = kit.text(stage, "Flip then turn", 30.0, Palette::gold(), v(3.1, 2.1))?;
        let mut objects = vec![title, left_title, right_title];
        let left_progress = stage.add_value_tracker(0.0);
        let right_progress = stage.add_value_tracker(0.0);
        for (center, left) in [(-3.1, true), (3.1, false)] {
            let progress = if left { left_progress } else { right_progress };
            let frame = stage.arena_mut().add(rect(
                v(center - 1.35, -1.35),
                2.7,
                2.7,
                stroke_style(Palette::dim(), 2.0),
            )?);
            objects.push(frame);
            crate::tile_actor::add_tile(stage, &kit, progress, center, 0.0, 1.25, move |t| state(t, left))?;
        }
        fade(stage, &objects)?;
        kit.say(stage, "05_2")?;
        transform(stage, left_progress, false)?;
        kit.say(stage, "05_3")?;
        let hinge = stage
            .arena_mut()
            .add(rect(v(-4.65, -0.015), 3.1, 0.03, ink_style(Palette::dim()))?);
        fade(stage, &[hinge])?;
        transform(stage, left_progress, true)?;
        kit.say(stage, "05_4")?;
        let question = kit.text(
            stage,
            "Where will the blue corner finish?",
            30.0,
            Palette::hero(),
            v(0.0, -1.9),
        )?;
        fade(stage, &[question])?;
        kit.hold(stage)?;
        stage.wait(4.0)?;
        let disappear = fade_out(stage.arena_mut(), question, ORIGIN, 1.0)?.rt(0.3);
        let prepared = stage.prepare(disappear)?;
        stage.play_prepared(vec![prepared])?;
        kit.say(stage, "05_5")?;
        let right_hinge = stage
            .arena_mut()
            .add(rect(v(1.55, -0.015), 3.1, 0.03, ink_style(Palette::dim()))?);
        fade(stage, &[right_hinge])?;
        transform(stage, right_progress, false)?;
        kit.until_sentence(stage, "05_5", 2)?;
        transform(stage, right_progress, true)?;
        kit.say(stage, "05_6")?;
        let mut corner_rings = Vec::new();
        for (x, y) in [(-4.225, -1.125), (4.225, 1.125)] {
            corner_rings.push(
                stage.arena_mut().add(
                    Circle::new()
                        .radius(0.31)
                        .style(stroke_style(Palette::blue(), 4.0))
                        .build()
                        .moved_to(v(x, y)),
                ),
            );
        }
        fade(stage, &corner_rings)?;
        kit.until_sentence(stage, "05_6", 1)?;
        let result = kit.text(stage, "The order matters", 32.0, Palette::green(), v(0.0, -1.8))?;
        fade(stage, &[result])?;
        kit.until_sentence(stage, "05_6", 2)?;
        let ordinary = kit.text(stage, "3 × 5 = 5 × 3", 24.0, Palette::ink(), v(0.0, 0.5))?;
        fade(stage, &[ordinary])?;
        kit.until_sentence(stage, "05_6", 3)?;
        let disappear = fade_out(stage.arena_mut(), ordinary, ORIGIN, 1.0)?.rt(0.3);
        let prepared = stage.prepare(disappear)?;
        stage.play_prepared(vec![prepared])?;
        kit.say(stage, "05_7")?;
        for index in 0..8 {
            let rotation = tile_model::turn((index % 4) as f64);
            let matrix = if index < 4 {
                rotation
            } else {
                tile_model::compose(rotation, tile_model::flip(1.0))
            };
            let center = -5.25 + index as f64 * 1.5;
            let mut card = vec![stage.arena_mut().add(rect(
                v(center - 0.48, -3.1),
                0.96,
                0.96,
                stroke_style(Palette::dim(), 2.0),
            )?)];
            for (i, p) in tile_model::vertices(matrix).iter().enumerate() {
                let colors = [Palette::blue(), Palette::gold(), Palette::green(), Palette::hero()];
                card.push(
                    stage.arena_mut().add(
                        Circle::new()
                            .radius(0.09)
                            .style(ink_style(colors[i]))
                            .build()
                            .moved_to(v(center + p[0] * 0.35, -2.62 + p[1] * 0.35)),
                    ),
                );
            }
            for (i, p) in tile_model::vertices(matrix).iter().enumerate() {
                card.push(kit.text(
                    stage,
                    &(i + 1).to_string(),
                    13.0,
                    Palette::ink(),
                    v(center + p[0] * 0.24, -2.62 + p[1] * 0.24),
                )?);
            }
            fade(stage, &card)?;
        }
        kit.say(stage, "05_8")?;
        let disappear = fade_out(stage.arena_mut(), title, ORIGIN, 1.0)?.rt(0.3);
        let prepared = stage.prepare(disappear)?;
        stage.play_prepared(vec![prepared])?;
        let arithmetic = kit.text(stage, "An arithmetic of moves", 36.0, Palette::ink(), v(0.0, 2.9))?;
        fade(stage, &[arithmetic])?;
        kit.until_sentence(stage, "05_8", 1)?;
        let roadmap = kit.text(
            stage,
            "Combine · undo · forget details",
            24.0,
            Palette::ink(),
            v(0.0, 3.5),
        )?;
        fade(stage, &[roadmap])?;
        kit.until_sentence(stage, "05_8", 2)?;
        let quotient = kit.text(stage, "Quotient groups", 24.0, Palette::hero(), v(0.0, 0.5))?;
        fade(stage, &[quotient])?;
        kit.finish(stage)
    }
}
