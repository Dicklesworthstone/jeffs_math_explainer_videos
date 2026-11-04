//! Shared native rigid tile: one transform drives face, corner discs and labels.
use crate::tile_model::{self, Matrix};
use explainer_kit::{Kit, Palette, hex, ink_style, v};
use fmn::prelude::*;
const POINTS: [(f64, f64); 4] = [(0.9, 0.9), (0.9, -0.9), (-0.9, -0.9), (-0.9, 0.9)];
pub fn add_tile(
    stage: &mut Stage<'_>,
    kit: &Kit,
    progress: Mob,
    center: f64,
    height: f64,
    scale: f64,
    pose: impl Fn(f64) -> Matrix + Copy + Send + Sync + 'static,
) -> fmn::Result<Vec<Mob>> {
    let mut objects = Vec::new();
    let tile = stage.always_redraw(move |arena| {
        let matrix = pose(arena.tracker_value(progress).unwrap_or(0.0));
        let points = tile_model::vertices(matrix).map(|p| {
            let p = tile_model::project(p, 6.0).expect("tile stays in front of camera");
            v(center + scale * p[0], height + scale * p[1])
        });
        let front = tile_model::apply(matrix, [0.0, 0.0, 1.0])[2] >= 0.0;
        arena.add(
            Polygon::new(points)
                .style(
                    Style::default()
                        .fill(if front { hex("#173148") } else { hex("#49391C") }, 1.0)
                        .stroke(if front { Palette::blue() } else { Palette::gold() }, 5.0, 1.0),
                )
                .build(),
        )
    });
    stage.add_to_scene(tile)?;
    objects.push(tile);
    for (i, (x, y)) in POINTS.into_iter().enumerate() {
        let colors = [Palette::blue(), Palette::gold(), Palette::green(), Palette::hero()];
        let marker = stage.always_redraw(move |arena| {
            let matrix = pose(arena.tracker_value(progress).unwrap_or(0.0));
            let point = tile_model::project(tile_model::apply(matrix, [x, y, 0.0]), 6.0)
                .expect("emblem stays in front of camera");
            arena.add(
                Circle::new()
                    .radius(0.23)
                    .style(ink_style(colors[i]))
                    .build()
                    .moved_to(v(center + scale * point[0], height + scale * point[1])),
            )
        });
        stage.add_to_scene(marker)?;
        objects.push(marker);
        let label = kit.text(
            stage,
            &(i + 1).to_string(),
            26.0,
            hex("#0E1116"),
            v(center + scale * x, height + scale * y),
        )?;
        stage.add_to_scene(label)?;
        objects.push(label);
        stage.add_updater(
            label,
            move |arena, _| {
                let matrix = pose(arena.tracker_value(progress).unwrap_or(0.0));
                let point = tile_model::project(tile_model::apply(matrix, [x, y, 0.0]), 6.0)
                    .expect("label stays in front of camera");
                arena.move_to(label, v(center + scale * point[0], height + scale * point[1]), ORIGIN);
            },
            true,
        )?;
    }
    Ok(objects)
}
