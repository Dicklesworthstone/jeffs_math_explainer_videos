//! Native tunnel handoff for the rotating-wind chapter.

use explainer_kit::{Kit, Palette, Timed, hex};
use fmn::prelude::*;

use crate::error;
use crate::field::{Frame, TunnelGeometry};
use super::rotating_wind01::{self, TunnelCues};

const FULL: View = View {
    x: 0.0,
    y: 1.15,
    width: 11.5,
};
const THUMB: View = View {
    x: 5.4,
    y: 3.25,
    width: 3.0,
};
/// How long a new view of the tunnel fades in. Chapter 02's first view holds this long instead.
const VIEW_FADE_IN: f64 = 0.35;

#[derive(Clone, Copy)]
struct View {
    x: f64,
    y: f64,
    width: f64,
}

impl View {
    fn at(self, g: TunnelGeometry, x: f64, y: f64) -> Vec3 {
        let scale = self.width / g.length;
        [
            self.x + (x - g.length / 2.0) * scale,
            self.y + (y - g.height / 2.0) * scale,
            0.0,
        ]
    }
}

fn fade(stage: &mut Stage<'_>, mobs: &[Mob], incoming: bool, seconds: f64) -> fmn::Result<()> {
    let mut animations = Vec::with_capacity(mobs.len());
    for &mob in mobs {
        let fade = if incoming {
            fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?
        } else {
            fade_out(stage.arena_mut(), mob, ORIGIN, 1.0)?
        };
        animations.push(stage.prepare(fade.rt(seconds))?);
    }
    stage.play_prepared(animations).map(|_| ())
}

struct Tunnel<'a> {
    source: &'a Frame,
    geometry: TunnelGeometry,
    view: View,
    picture: Vec<Mob>,
}

type Point = [f64; 2];

fn midpoint(
    point: Point,
    step: f64,
    first: impl Fn(Point) -> Option<Point>,
    middle: impl Fn(Point) -> Option<Point>,
) -> Option<Point> {
    let [u, v] = first(point)?;
    let center = [point[0] + 0.5 * step * u, point[1] + 0.5 * step * v];
    let [u, v] = middle(center)?;
    let end = [point[0] + step * u, point[1] + step * v];
    middle(end)?;
    Some(end)
}

fn streamline(frame: &Frame, seed: Point) -> Vec<Point> {
    // Trace the direction of the last saved cell velocity in physical space.
    let direction = |[x, y]: Point| {
        let [u, v] = frame.cell_velocity_at(x, y)?;
        let speed = u.hypot(v);
        (speed > 1e-6).then(|| [u / speed, v / speed])
    };
    let mut points = vec![seed];
    let mut point = seed;
    for _ in 0..1_100 {
        let Some(next) = midpoint(point, 0.0025, direction, direction) else {
            break;
        };
        points.push(next);
        point = next;
    }
    points
}

pub(super) fn streakline(frames: &[Frame], nozzle: Point) -> Option<Vec<Point>> {
    // Release a tracer at each recorded frame and move it through all subsequent
    // observations. Use the cell samples in space and linear interpolation in time.
    const SUBSTEPS: u32 = 16;
    let mut tips = Vec::with_capacity(frames.len());
    for release in 0..frames.len() {
        let mut point = nozzle;
        for pair in frames[release..].windows(2) {
            let (a, b) = (&pair[0], &pair[1]);
            let step = (b.time - a.time) / f64::from(SUBSTEPS);
            let between = |[x, y]: Point, share: f64| {
                let [u0, v0] = a.cell_velocity_at(x, y)?;
                let [u1, v1] = b.cell_velocity_at(x, y)?;
                Some([u0 + share * (u1 - u0), v0 + share * (v1 - v0)])
            };
            for k in 0..SUBSTEPS {
                point = midpoint(
                    point,
                    step,
                    |p| between(p, f64::from(k) / f64::from(SUBSTEPS)),
                    |p| between(p, (f64::from(k) + 0.5) / f64::from(SUBSTEPS)),
                )?;
            }
        }
        tips.push(point);
    }
    Some(tips)
}

impl Tunnel<'_> {
    fn set_view(&mut self, stage: &mut Stage<'_>, view: View) -> fmn::Result<()> {
        self.show(stage, view, true)
    }

    /// Chapter 02's first picture: the full tunnel chapter 01 ends on, on screen from the first frame
    /// instead of faded in from nothing at the join. It then holds for VIEW_FADE_IN, which fmn rounds
    /// to frames as it rounds the fade, so 02_1 and every later frame keep their place.
    fn open(&mut self, stage: &mut Stage<'_>, view: View) -> fmn::Result<()> {
        self.show(stage, view, false)
    }

    fn show(&mut self, stage: &mut Stage<'_>, view: View, fade_in: bool) -> fmn::Result<()> {
        let g = self.geometry;
        let [cx, cy, radius] = g.cylinder;
        let scale = view.width / g.length;
        let tank = stage.arena_mut().add(
            Rectangle::new()
                .width(view.width)
                .height(g.height * scale)
                .style(
                    Style::default()
                        .fill(hex("#17283C"), 1.0)
                        .stroke(Palette::dim(), 1.7, 1.0),
                )
                .build()?
                .moved_to([view.x, view.y, 0.0]),
        );
        let obstacle = stage.arena_mut().add(
            Circle::new()
                .radius(radius * scale)
                .arc_center(view.at(g, cx, cy))
                .style(
                    Style::default()
                        .fill(hex("#0A121F"), 1.0)
                        .stroke(Palette::ink(), 1.6, 1.0),
                )
                .build(),
        );
        if !self.picture.is_empty() {
            fade(stage, &self.picture, false, 0.25)?;
        }
        self.view = view;
        self.picture = vec![tank, obstacle];
        if fade_in {
            fade(stage, &self.picture, true, VIEW_FADE_IN)
        } else {
            for &mob in &self.picture {
                stage.add_to_scene(mob)?;
            }
            stage.wait(VIEW_FADE_IN).map(|_| ())
        }
    }

    fn label(
        &mut self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        text: &str,
        y: f64,
        size: f64,
        color: Srgb,
    ) -> fmn::Result<()> {
        let mob = kit.text(stage, text, size, color, [self.view.x, y, 0.0])?;
        fade(stage, &[mob], true, 0.35)?;
        self.picture.push(mob);
        Ok(())
    }

    fn arrows(&self, stage: &mut Stage<'_>, max_length: f64) -> fmn::Result<Vec<Mob>> {
        let g = self.geometry;
        let view = self.view;
        let mut samples = Vec::new();
        for col in 0..9 {
            for row in 0..3 {
                let x = (f64::from(col) + 0.5) * g.length / 9.0;
                let y = (f64::from(row) + 0.5) * g.height / 3.0;
                if let Some([u, v]) = self.source.cell_velocity_at(x, y) {
                    samples.push((view.at(g, x, y), u, v));
                }
            }
        }
        let peak = samples.iter().map(|(_, u, v)| u.hypot(*v)).fold(0.0, f64::max);
        if peak <= 0.0 {
            return Err(error("The saved tunnel field needs moving fluid".to_string()));
        }
        let mut arrows = Vec::new();
        let scale = max_length / peak;
        for (a, u, v) in samples {
            let b = [a[0] + scale * u, a[1] + scale * v, 0.0];
            if u.hypot(v) * scale > 0.02 {
                arrows.push(
                    stage.arena_mut().add(
                        Arrow::new(a, b)
                            .buff(0.0)
                            .thickness(0.9)
                            .color(Palette::blue())
                            .build()?,
                    ),
                );
            }
        }
        Ok(arrows)
    }

    fn trace(&self, stage: &mut Stage<'_>, points: &[Point], style: Style) -> Mob {
        let points: Vec<Vec3> = points
            .iter()
            .map(|&[x, y]| self.view.at(self.geometry, x, y))
            .collect();
        stage.arena_mut().add(polyline(points).style(style).build())
    }
}

impl TunnelCues for Tunnel<'_> {
    fn shrink_to_thumbnail(&mut self, _kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        self.set_view(stage, THUMB)
    }

    fn velocity_stamp(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let arrows = self.arrows(stage, 0.16)?;
        fade(stage, &arrows, true, 0.4)?;
        self.picture.extend(arrows);
        self.label(
            kit,
            stage,
            &format!("Saved velocity at {:.3} s", self.source.time),
            3.79,
            16.0,
            Palette::blue(),
        )
    }
}

pub struct Chapter02 {
    pub kit: Kit,
    pub startup: Option<&'static [Frame]>,
}

impl SceneConstruct for Chapter02 {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let startup = self
            .startup
            .filter(|frames| frames.len() >= 2 && frames.windows(2).all(|p| p[0].time < p[1].time))
            .ok_or_else(|| error("Chapter 02 needs the saved tunnel frames in time order".to_owned()))?;
        let snapshot = startup.last().expect("startup contains at least two frames");
        let geometry = snapshot
            .tunnel_geometry()
            .ok_or_else(|| error("Expected a saved cylinder tunnel frame".to_string()))?;
        let kit = &self.kit;
        let mut tunnel = Tunnel {
            geometry,
            source: snapshot,
            view: FULL,
            picture: Vec::new(),
        };
        tunnel.open(stage, FULL)?;
        let wind = rotating_wind01::chapter02(kit, stage, &mut tunnel, true)?;
        kit.say(stage, "02_11")?;
        wind.leave(stage, 0.5)?;
        tunnel.set_view(stage, FULL)?;
        let edge = FULL.y - FULL.width * geometry.height / geometry.length / 2.0 - 0.20;
        let bracket = stage.arena_mut().add(
            Line::new([-4.25, edge, 0.0], [0.45, edge, 0.0])
                .style(Style::default().stroke(Palette::hero(), 2.0, 1.0))
                .build()?,
        );
        let nozzle = [0.14, 0.265];
        let dye = streakline(startup, nozzle)
            .ok_or_else(|| error("Chapter 02 dye trace left the saved tunnel".to_owned()))?;
        let dye = tunnel.trace(stage, &dye, Style::default().stroke(Palette::purple(), 6.5, 1.0));
        let dot = stage.arena_mut().add(
            Circle::new()
                .radius(0.035)
                .arc_center(FULL.at(geometry, nozzle[0], nozzle[1]))
                .style(Style::default().fill(Palette::purple(), 1.0))
                .build(),
        );
        stage.set_z_index(dye, 2, true);
        stage.set_z_index(dot, 2, true);
        fade(stage, &[bracket, dye, dot], true, 0.35)?;
        tunnel.label(
            kit,
            stage,
            "Dye carries a record of the flow",
            -1.18,
            33.0,
            Palette::hero(),
        )?;
        tunnel.picture.extend([bracket, dye, dot]);
        kit.say(stage, "02_12")?;
        let arrows = tunnel.arrows(stage, 0.42)?;
        let mut lines = Vec::new();
        for share in [0.12, 0.24, 0.37, 0.50, 0.63, 0.76, 0.88] {
            let points = streamline(snapshot, [0.01, geometry.height * share]);
            if points.len() >= 2 {
                lines.push(tunnel.trace(stage, &points, Style::default().stroke(Palette::ink(), 1.6, 0.55)));
            }
        }
        if lines.len() != 7 {
            return Err(error("Chapter 02 could not trace all seven saved-field streamlines".to_owned()));
        }
        let stamp = kit.text(
            stage,
            &format!("Saved velocity at {:.3} s", snapshot.time),
            20.0,
            Palette::blue(),
            [0.0, 3.4, 0.0],
        )?;
        for (text, x, color, mut mobs) in [
            ("Velocity (instant)", -4.1, Palette::blue(), arrows),
            ("Streamlines (instant)", 0.0, Palette::ink(), lines),
            ("Dye (reconstructed)", 4.1, Palette::purple(), vec![stamp]),
        ] {
            let label = kit.text(stage, text, 25.0, color, [x, 2.65, 0.0])?;
            mobs.push(label);
            fade(stage, &mobs, true, 0.33)?;
            tunnel.picture.extend(mobs);
        }
        kit.say(stage, "02_13")?;
        tunnel.set_view(stage, THUMB)?;
        kit.until_sentence(stage, "02_13", 2)?;
        let question = kit.text(
            stage,
            "What makes velocity change?",
            40.0,
            Palette::hero(),
            [0.0, 0.15, 0.0],
        )?;
        fade(stage, &[question], true, 0.45)?;
        kit.finish(stage)
    }
}
