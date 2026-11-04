//! Construct all square symmetries, then prove completeness by adjacent corners.
use crate::{tile_actor, tile_model};
use explainer_kit::{Kit, Palette, Timed, ink_style, stroke_style, tween, v};
use fmn::prelude::*;

pub struct EightConstruction {
    pub kit: Kit,
}
fn animate(stage: &mut Stage<'_>, tracker: Mob, destination: f64, seconds: f64) -> fmn::Result<()> {
    let movement = tween(tracker, destination, seconds)?;
    let animation = stage.prepare(movement)?;
    stage.play_prepared(vec![animation])?;
    Ok(())
}
fn reveal(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut actions = Vec::new();
    for &mob in mobs {
        let action = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?.rt(0.45);
        actions.push(stage.prepare(action)?);
    }
    stage.play_prepared(actions)?;
    Ok(())
}
fn hide(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut actions = Vec::new();
    for &mob in mobs {
        stage.clear_updaters(mob, true);
        let action = fade_out(stage.arena_mut(), mob, ORIGIN, 1.0)?.rt(0.45);
        actions.push(stage.prepare(action)?);
    }
    stage.play_prepared(actions)?;
    Ok(())
}
fn corner(stage: &mut Stage<'_>, kit: &Kit, index: usize, destination: usize) -> fmn::Result<Vec<Mob>> {
    let p = tile_model::CORNERS[destination];
    let at = v(-3.2 + p[0] * 1.125, p[1] * 1.125);
    let color = [Palette::blue(), Palette::gold(), Palette::green(), Palette::hero()][index];
    let dot = stage
        .arena_mut()
        .add(Circle::new().radius(0.23).style(ink_style(color)).build().moved_to(at));
    let label = kit.text(stage, &(index + 1).to_string(), 26., Palette::ink(), at)?;
    Ok(vec![dot, label])
}
impl SceneConstruct for EightConstruction {
    fn name(&self) -> &str {
        "07_eight_moves"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        kit.say(stage, "07_1")?;
        let title = kit.text(stage, "Could there be a ninth move?", 36.0, Palette::ink(), v(0., 3.1))?;
        reveal(stage, &[title])?;
        let rotation = stage.add_value_tracker(0.0);
        let hero = tile_actor::add_tile(stage, &kit, rotation, -3.2, 0.0, 1.25, |t| {
            if t <= 4.0 {
                tile_model::turn(t)
            } else if t <= 5.0 {
                tile_model::flip(t - 4.0)
            } else {
                tile_model::compose(tile_model::turn(t - 5.0), tile_model::flip(1.0))
            }
        })?;
        // Make the narration's unchanged-outline distinction visible before
        // collecting the moves. Return to identity for the first rack slot.
        kit.until_sentence(stage, "07_1", 2)?;
        animate(stage, rotation, 1.0, 1.5)?;
        stage.wait(1.0)?;
        animate(stage, rotation, 0.0, 1.5)?;
        // Each rack thumbnail is computed from exactly the same corner action.
        let mut rack = Vec::new();
        for index in 0..8 {
            let turns = index % 4;
            let reflected = index >= 4;
            let x = 0.6 + turns as f64 * 1.5;
            let y = if reflected { -1.1 } else { 1.1 };
            let matrix = if reflected {
                tile_model::compose(tile_model::turn(turns as f64), tile_model::flip(1.0))
            } else {
                tile_model::turn(turns as f64)
            };
            let vertices = tile_model::vertices(matrix);
            let mut objects = vec![
                stage.arena_mut().add(
                    Polygon::new(vertices.map(|p| v(x + p[0] * 0.48, y + p[1] * 0.48)))
                        .style(stroke_style(Palette::dim(), 2.))
                        .build(),
                ),
            ];
            for (corner, p) in vertices.iter().enumerate() {
                let color = [Palette::blue(), Palette::gold(), Palette::green(), Palette::hero()][corner];
                objects.push(
                    stage.arena_mut().add(
                        Circle::new()
                            .radius(0.1)
                            .style(ink_style(color))
                            .build()
                            .moved_to(v(x + p[0] * 0.36, y + p[1] * 0.36)),
                    ),
                );
                objects.push(kit.text(
                    stage,
                    &(corner + 1).to_string(),
                    16.,
                    Palette::ink(),
                    v(x + p[0] * 0.22, y + p[1] * 0.22),
                )?);
            }
            rack.push(objects);
        }
        kit.say(stage, "07_2")?;
        kit.until_sentence(stage, "07_2", 2)?;
        reveal(stage, &rack[0])?;
        kit.until_sentence(stage, "07_2", 3)?;
        for (turns, card) in rack.iter().enumerate().take(4).skip(1) {
            animate(stage, rotation, turns as f64, 1.1)?;
            reveal(stage, card)?;
        }
        kit.until_sentence(stage, "07_2", 5)?;
        animate(stage, rotation, 4., 1.5)?;
        kit.say(stage, "07_3")?;
        animate(stage, rotation, 5., 2.)?;
        kit.until_sentence(stage, "07_3", 2)?;
        reveal(stage, &rack[4])?;
        kit.until_sentence(stage, "07_3", 3)?;
        for turns in 1..4 {
            animate(stage, rotation, 5. + turns as f64, 0.85)?;
            reveal(stage, &rack[4 + turns])?;
        }
        kit.say(stage, "07_4")?;
        kit.until_sentence(stage, "07_4", 2)?;
        let question = kit.text(
            stage,
            "Track one corner and its neighbor",
            28.,
            Palette::hero(),
            v(0., -2.75),
        )?;
        reveal(stage, &[question])?;
        kit.say(stage, "07_5")?;
        hide(stage, &hero)?;
        let frame = stage.arena_mut().add(
            Polygon::new(tile_model::CORNERS.map(|p| v(-3.2 + p[0] * 1.125, p[1] * 1.125)))
                .style(stroke_style(Palette::dim(), 3.))
                .build(),
        );
        reveal(stage, &[frame])?;
        let mut destinations = Vec::new();
        for p in tile_model::CORNERS {
            let ring = stage.arena_mut().add(
                Circle::new()
                    .radius(0.25)
                    .style(stroke_style(Palette::blue(), 4.))
                    .build()
                    .moved_to(v(-3.2 + p[0] * 1.125, p[1] * 1.125)),
            );
            reveal(stage, &[ring])?;
            destinations.push(ring);
            stage.wait(0.2)?;
        }
        kit.until_sentence(stage, "07_5", 1)?;
        hide(stage, &destinations)?;
        let blue = corner(stage, &kit, 0, 0)?;
        reveal(stage, &blue)?;
        kit.until_sentence(stage, "07_5", 3)?;
        let mut neighbors = Vec::new();
        for neighbor in [v(-3.2 - 1.125, 1.125), v(-3.2 + 1.125, -1.125)] {
            let ring = stage.arena_mut().add(
                Circle::new()
                    .radius(0.3)
                    .style(stroke_style(Palette::gold(), 6.))
                    .build()
                    .moved_to(neighbor),
            );
            reveal(stage, &[ring])?;
            neighbors.push(ring);
            stage.wait(1.)?;
        }
        kit.say(stage, "07_6")?;
        kit.until_sentence(stage, "07_6", 1)?;
        hide(stage, &neighbors)?;
        // With blue fixed at NE, first choose orange at SE; green and yellow
        // then occupy SW and NW. Show the other choice rather than only assert it.
        let mut completion = Vec::new();
        for index in 1..4 {
            let badge = corner(stage, &kit, index, index)?;
            reveal(stage, &badge)?;
            completion.extend(badge);
            stage.wait(0.4)?;
        }
        stage.wait(0.6)?;
        hide(stage, &completion)?;
        for (index, destination) in [(1, 3), (2, 2), (3, 1)] {
            let badge = corner(stage, &kit, index, destination)?;
            reveal(stage, &badge)?;
            stage.wait(0.4)?;
        }
        kit.until_sentence(stage, "07_6", 2)?;
        let conclusion = kit.text(
            stage,
            "4 choices × 2 neighbors = 8",
            30.,
            Palette::green(),
            v(0., -2.15),
        )?;
        reveal(stage, &[conclusion])?;
        kit.finish(stage)
    }
}
