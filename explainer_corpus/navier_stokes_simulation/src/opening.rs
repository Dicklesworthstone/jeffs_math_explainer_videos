//! The wind tunnel, its actual velocity, and the cost of refining one square.
//! Teaching squares are native geometry. They are separate from the saved mesh.

use std::path::PathBuf;

use explainer_kit::Timed;
use explainer_kit::palette::{Palette, hex};
use fmn::prelude::*;

use crate::field::{Frame, TunnelGeometry};
use crate::{error, label};

const WIDTH: f64 = 11.5;
const CENTER_Y: f64 = 1.15;
pub const TAKE_LEAD: f64 = 0.2;

pub struct OpeningScene {
    pub snapshot: Frame,
    pub introduction: Option<(PathBuf, f64)>,
    pub cell_explanation: Option<(PathBuf, f64)>,
}

fn point(x: f64, y: f64) -> Vec3 {
    [x, y, 0.0]
}

fn line(stage: &mut Stage<'_>, a: Vec3, b: Vec3, color: Srgb, width: f64) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(
        Line::new(a, b)
            .style(Style::default().stroke(color, width, 1.0))
            .build()?,
    ))
}

fn box_at(
    stage: &mut Stage<'_>,
    center: Vec3,
    width: f64,
    height: f64,
    rim: Srgb,
    fill: Srgb,
    opacity: f64,
) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(
        Rectangle::new()
            .width(width)
            .height(height)
            .style(Style::default().fill(fill, opacity).stroke(rim, 1.7, 1.0))
            .build()?
            .moved_to(center),
    ))
}

fn fade(stage: &mut Stage<'_>, mobs: &[Mob], appear: bool, seconds: f64) -> fmn::Result<()> {
    let mut prepared = Vec::with_capacity(mobs.len());
    for &mob in mobs {
        let animation = if appear {
            fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?
        } else {
            fade_out(stage.arena_mut(), mob, ORIGIN, 1.0)?
        };
        prepared.push(stage.prepare(animation.rt(seconds))?);
    }
    stage.play_prepared(prepared)?;
    Ok(())
}

fn start_take(
    stage: &mut Stage<'_>,
    take: Option<&(PathBuf, f64)>,
    preview_hold: f64,
) -> fmn::Result<f64> {
    let Some((path, seconds)) = take else {
        return Ok(stage.scene().time().to_f64() + preview_hold);
    };
    let now = stage.scene().time().to_f64();
    stage.scene_mut().add_sound(path, TAKE_LEAD, Some(0.0), None)?;
    println!("Opening take {} at {:.6} s for {:.6} s", path.display(), now + TAKE_LEAD, seconds);
    Ok(now + TAKE_LEAD + seconds + 0.23)
}

fn hold_until(stage: &mut Stage<'_>, end: f64) -> fmn::Result<()> {
    let remaining = end - stage.scene().time().to_f64();
    if remaining > 0.0 {
        stage.wait(remaining)?;
    }
    Ok(())
}

fn tunnel_point(geometry: TunnelGeometry, x: f64, y: f64) -> Vec3 {
    let scale = WIDTH / geometry.length;
    point(scale * (x - geometry.length / 2.0), CENTER_Y + scale * (y - geometry.height / 2.0))
}

fn velocity_arrows(
    stage: &mut Stage<'_>,
    frame: &Frame,
    geometry: TunnelGeometry,
) -> fmn::Result<Vec<Mob>> {
    const COLUMNS: usize = 23;
    const ROWS: usize = 5;
    let mut samples = Vec::new();
    for i in 0..COLUMNS {
        let x = geometry.length * (i as f64 + 0.5) / COLUMNS as f64;
        for j in 0..ROWS {
            let y = geometry.height * (j as f64 + 0.5) / ROWS as f64;
            if let Some([u, v]) = frame.cell_velocity_at(x, y) {
                samples.push((tunnel_point(geometry, x, y), [u, v]));
            }
        }
    }
    let peak = samples.iter().map(|(_, [u, v])| u.hypot(*v)).fold(0.0, f64::max);
    if peak <= 0.0 {
        return Err(error("Opening needs a saved, moving tunnel flow".to_string()));
    }
    let scale = 0.43 / peak;
    let color = Palette::blue();
    let mut arrows = Vec::new();
    for (origin, [u, v]) in samples {
        let length = scale * u.hypot(v);
        if length < 0.045 {
            continue;
        }
        let end = point(origin[0] + scale * u, origin[1] + scale * v);
        arrows.push(stage.arena_mut().add(
            Arrow::new(origin, end)
                .buff(0.0)
                .thickness(1.1)
                .style(Style::default().fill(color, 1.0).stroke(color, 1.1, 1.0))
                .build()?,
        ));
    }
    println!("Opening snapshot {:.6} s, {} arrows with one velocity scale", frame.time, arrows.len());
    Ok(arrows)
}

fn teaching_grid(stage: &mut Stage<'_>, geometry: TunnelGeometry) -> fmn::Result<Vec<Mob>> {
    let cell = geometry.height / 3.0;
    let center = geometry.length / 2.0;
    let color = Palette::faint();
    let p = |x, y| tunnel_point(geometry, x, y);
    let mut lines = Vec::new();
    for row in 1..3 {
        let y = row as f64 * cell;
        lines.push(line(stage, p(0.0, y), p(geometry.length, y), color, 1.1)?);
    }
    let half_columns = (center / cell).floor() as i32;
    for column in -half_columns..=half_columns {
        let x = center + f64::from(column) * cell;
        lines.push(line(stage, p(x, 0.0), p(x, geometry.height), color, 1.1)?);
    }
    Ok(lines)
}

impl SceneConstruct for OpeningScene {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let Some(geometry) = self.snapshot.tunnel_geometry() else {
            return Err(error("Opening needs a tunnel snapshot".to_string()));
        };
        let fonts = FontBook::bundled().map_err(fmn::library::TextMobjectError::Text)?;
        let [x, y, radius] = geometry.cylinder;
        let scale = WIDTH / geometry.length;
        let height = scale * geometry.height;
        let field_color = hex("#17283C");
        let cylinder_color = hex("#0A121F");
        let wall = box_at(stage, point(0.0, CENTER_Y), WIDTH, height, Palette::dim(), field_color, 1.0)?;
        let cylinder = stage.arena_mut().add(
            Circle::new()
                .radius(scale * radius)
                .style(Style::default().fill(cylinder_color, 1.0).stroke(Palette::ink(), 1.6, 1.0))
                .build()
                .moved_to(tunnel_point(geometry, x, y)),
        );
        let end = start_take(stage, self.introduction.as_ref(), 2.7)?;
        fade(stage, &[wall], true, 0.42)?;
        fade(stage, &[cylinder], true, 0.32)?;
        label(stage, &fonts, "A wind tunnel inside a computer", 45.0, Palette::ink(), point(0.0, 3.57))?;
        hold_until(stage, end)?;

        let arrows = velocity_arrows(stage, &self.snapshot, geometry)?;
        fade(stage, &arrows, true, 0.6)?;
        label(stage, &fonts, "Inlet", 23.0, Palette::dim(), point(-5.2, 2.59))?;
        label(stage, &fonts, "Outlet", 23.0, Palette::dim(), point(5.2, 2.59))?;
        let stamp = label(
            stage, &fonts, &format!("Saved velocity at {:.3} s", self.snapshot.time),
            27.0, Palette::blue(), point(0.0, 2.60),
        )?;
        // Mark the same region below the tunnel so the guide never crosses the flow arrows.
        let wake_left = -4.25;
        let wake_right = 0.45;
        let guide_y = CENTER_Y - height / 2.0 - 0.2;
        let guide_color = Palette::hero();
        let downstream = [
            line(stage, point(wake_left, guide_y), point(wake_right, guide_y), guide_color, 2.0)?,
            line(stage, point(wake_left, guide_y), point(wake_left, guide_y + 0.10), guide_color, 2.0)?,
            line(stage, point(wake_right, guide_y), point(wake_right, guide_y + 0.10), guide_color, 2.0)?,
        ];
        let question = label(
            stage, &fonts, "Which details belong to the flow?", 38.0, Palette::hero(),
            point(0.0, -1.32),
        )?;
        fade(stage, &downstream, true, 0.35)?;
        stage.wait(1.85)?;
        fade(stage, &arrows, false, 0.25)?;
        stage.scene_mut().remove(&downstream);
        stage.scene_mut().remove(&[stamp, question]);

        let end = start_take(stage, self.cell_explanation.as_ref(), 5.7)?;
        stage.wait(2.5)?;
        let mesh = teaching_grid(stage, geometry)?;
        fade(stage, &mesh, true, 0.62)?;
        label(stage, &fonts, "Teaching grid", 27.0, Palette::dim(), point(0.0, 2.60))?;
        let width = scale * geometry.height / 3.0;
        let source = box_at(
            stage, point(width / 2.0, CENTER_Y), width, width,
            Palette::gold(), Palette::gold(), 0.16,
        )?;
        fade(stage, &[source], true, 0.30)?;
        hold_until(stage, end)?;

        const SIDE: f64 = 1.8;
        const Y: f64 = -1.72;
        const LEFT: f64 = -2.15;
        const RIGHT: f64 = 2.15;
        let color = Palette::gold();
        let old = box_at(stage, point(LEFT, Y), SIDE, SIDE, color, color, 0.18)?;
        let refined = box_at(stage, point(RIGHT, Y), SIDE, SIDE, color, color, 0.18)?;
        fade(stage, &[old, refined], true, 0.46)?;
        label(stage, &fonts, "same patch", 23.0, Palette::dim(), point(0.0, Y))?;
        let subdivisions = 2;
        let lines = [
            line(stage, point(RIGHT - SIDE / 2.0, Y), point(RIGHT + SIDE / 2.0, Y), color, 2.0)?,
            line(stage, point(RIGHT, Y - SIDE / 2.0), point(RIGHT, Y + SIDE / 2.0), color, 2.0)?,
        ];
        fade(stage, &lines, true, 0.44)?;
        label(stage, &fonts, "1 cell", 30.0, Palette::ink(), point(LEFT, -3.00))?;
        label(
            stage, &fonts, &format!("{} cells", subdivisions * subdivisions),
            30.0, Palette::ink(), point(RIGHT, -3.00),
        )?;
        label(stage, &fonts, "Half the cell width", 26.0, Palette::dim(), point(0.0, -3.65))?;
        stage.wait(2.4)?;
        Ok(())
    }
}
