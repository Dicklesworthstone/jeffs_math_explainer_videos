//! Chapter 04, the corner lemma: the first of the points no tile can dodge
//! (04_1). A tile whose centre is within one tile of both walls of a corner
//! covers the point one tile from both (04_2), however it is turned: turning
//! widens its shadow (04_3, with a four-second think), the walls push its
//! centre out by half the shadow, so the point is at most one minus half a
//! shadow away across and up (04_4), and along the tile's own sides that is
//! at most s (1 − s/2), never more than one half (04_5), the half tile a
//! tile reaches; only a straight tile resting on a wall has the point on its
//! edge (04_6). Without a wall, or with the centre past one tile, the point
//! can escape (04_7). In the box of 2.71 this settles the centres near the
//! corners; the edges and the middle come next (04_8).
//!
//! Friedman's Lemma 1 (DS7), proved as in `model.rs` § the corner lemma.
//! Every frame is a tile from the model (`corner_wander`, `corner_pushed`,
//! a straight tile up the wall, and the two moves that break an assumption);
//! the ring's colour is the point's depth inside that tile, and the readouts
//! read the pose the frame drew. The corner view shows the quadrant at
//! [`CORNER`]'s scale; 04_8 returns to the box at chapter 01's.

use std::cell::Cell;
use std::rc::Rc;

use explainer_kit::{Kit, Palette, fade_all, tween, v};
use fmn::prelude::*;

use super::draw::{
    Board, arc_points, corner, dim, dot, fact, fade_to, fail, faint, flat, ink, line_style, path, polygon, redot,
    reshape, retrace, ring, say, stand, swap, together, touch, turned, warn,
};
use super::the_turned_tile::{BOARD, DEGREE, PRECISELY, RING_AT, RING_R, THINK_S};
use crate::model::{self, Surd, Tile};

pub struct TheCornerLemma {
    pub kit: Kit,
}

/// The narration lines this chapter speaks, in order.
pub const LINES: [&str; 8] = ["04_1", "04_2", "04_3", "04_4", "04_5", "04_6", "04_7", "04_8"];

// the chapter's fixed words
pub const TURN: &str = "turn";
pub const SHADOW: &str = "shadow";
pub const ALONG: &str = "the point along its sides";
pub const AT_MOST: &str = r"\le";
pub const BOUND: &str = r"s\,(1-\tfrac{s}{2}) = \tfrac{1}{2} - \tfrac{1}{2}\,(s-1)^2 \le \tfrac{1}{2}";
pub const S_IS: &str = "s: the shadow, from 1 to";
pub const EDGE: &str = "straight, on a wall: on the edge";
pub const DROPPED: &str = "an assumption dropped";
pub const SETTLED: &str = "settled: centres within one tile of a corner";
pub const NEXT: &str = "next: the edges and the middle";

// ---------------------------------------------------------------- layout

/// The corner view: the quadrant [0, VIEW]² at 2.3 scene units a tile.
pub const CORNER: Board = Board {
    at: [-3.0, -0.4, 0.0],
    k: 2.3,
};
pub const VIEW: f64 = 1.9;
/// The left edge of the panel's text.
pub const TEXT_X: f64 = 1.7;
/// The shadow's bar under the floor (tiles).
pub const SHADOW_Y: (f64, f64) = (-0.14, -0.07);
/// Dots: the point, the tile's centre, the box's points.
pub const POINT_R: f64 = 0.07;
pub const CENTRE_R: f64 = 0.05;
/// The ring round the point.
pub const RING_R_POINT: f64 = 0.16;
/// Below this depth (in tiles) either way the ring calls it the edge: a
/// drawing threshold for f64 frames, not a gate (the facts are exact).
pub const EDGE_BAND: f64 = 1e-9;
/// Draw layers: the allowed centres under the tile, marks above, the ring
/// on top.
pub const LAYER_UNDER: i32 = -1;
pub const LAYER_MARKS: i32 = 2;
pub const LAYER_RING: i32 = 3;

/// What the chapter reads from the facts, checked against the model.
struct Reads {
    shadow_max: String,
    reach: String,
    side: f64,
}

fn reads() -> fmn::Result<Reads> {
    for (key, want) in [
        ("corner.point", "1,1"),
        ("corner.lemma", "yes"),
        ("corner.bound.max", "1/2"),
        ("corner.edges", "edge|edge|edge|inside"),
        ("corner.turned", "inside"),
        ("corner.box.covered", "1"),
    ] {
        if fact(key)? != want {
            return Err(fail(format!(
                "{key} is not {want:?}: the chapter would say what the facts do not"
            )));
        }
    }
    for key in ["corner.fail.centre", "corner.fail.wall"] {
        if !fact(key)?.starts_with("outside|") {
            return Err(fail(format!("{key}: the dropped assumption does not lose the point")));
        }
    }
    let shadow_max = fact("corner.shadow.max.2dp")?;
    if shadow_max != model::shadow_width(model::eighth_turn()).decimals(2) {
        return Err(fail("corner.shadow.max.2dp is not the model's".into()));
    }
    Ok(Reads {
        shadow_max,
        reach: model::corner_bound(Surd::int(1)).decimals(2),
        side: model::turned_side().to_f64(),
    })
}

/// The ring's colour for the point's depth inside the tile.
fn ring_colour(depth: f64) -> Srgb {
    if depth > EDGE_BAND {
        Palette::yellow()
    } else if depth < -EDGE_BAND {
        warn()
    } else {
        touch()
    }
}

/// The corner view's parts every frame redraws.
#[derive(Clone, Copy)]
struct Parts {
    tile: Mob,
    centre: Mob,
    shadow: Mob,
    allowed: Mob,
    across: Mob,
    up: Mob,
    side_u: Mob,
    side_v: Mob,
    ring: Mob,
}

/// Redraw a frame for tile `t`, and record its pose (centre, turn in
/// [0, 90°)) for the readouts.
fn draw(s: &mut fmn::mobject::Stage, parts: &Parts, pose: &Cell<[f64; 3]>, t: &Tile<f64>) {
    let at = |q: (f64, f64)| CORNER.p(VIEW, q);
    let (cx, cy) = t.centre;
    reshape(s, parts.tile, CORNER.tile(VIEW, t));
    redot(s, parts.centre, at(t.centre), CENTRE_R);
    let sh = model::shadow_width((t.cos, t.sin));
    let (x0, x1) = (cx - sh / 2.0, cx + sh / 2.0);
    let (y0, y1) = SHADOW_Y;
    reshape(
        s,
        parts.shadow,
        vec![at((x0, y0)), at((x1, y0)), at((x1, y1)), at((x0, y1))],
    );
    let lo = sh / 2.0;
    reshape(
        s,
        parts.allowed,
        vec![at((lo, lo)), at((1.0, lo)), at((1.0, 1.0)), at((lo, 1.0))],
    );
    retrace(s, parts.across, vec![at(t.centre), at((1.0, cy))]);
    retrace(s, parts.up, vec![at((1.0, cy)), at((1.0, 1.0))]);
    let half = 0.5;
    retrace(
        s,
        parts.side_u,
        vec![at(t.centre), at((cx + half * t.cos, cy + half * t.sin))],
    );
    retrace(
        s,
        parts.side_v,
        vec![at(t.centre), at((cx - half * t.sin, cy + half * t.cos))],
    );
    let depth = model::depth_inside(t, model::corner_point());
    s.set_stroke(parts.ring, Some(ring_colour(depth)), None, None, None, true);
    let theta = t.sin.atan2(t.cos).rem_euclid(std::f64::consts::FRAC_PI_2);
    pose.set([cx, cy, theta]);
}

/// One motion of the tile, drawn as it goes from the model's `frame` (one
/// updater on an anchor in the scene).
struct Motion {
    tracker: Mob,
    anchor: Mob,
}

impl Motion {
    fn install(
        stage: &mut Stage<'_>,
        parts: Parts,
        pose: &Rc<Cell<[f64; 3]>>,
        start: f64,
        frame: Box<dyn Fn(f64) -> Tile<f64>>,
    ) -> fmn::Result<Self> {
        let tracker = stage.add_value_tracker(start);
        let anchor = stage.arena_mut().add(VMobject::new());
        let pose = Rc::clone(pose);
        let mut last = f64::NAN;
        stage.add_updater(
            anchor,
            move |s: &mut fmn::mobject::Stage, _| {
                let now = s.tracker_value(tracker).unwrap_or(start);
                if now.to_bits() == last.to_bits() {
                    return;
                }
                last = now;
                draw(s, &parts, &pose, &frame(now));
            },
            true,
        )?;
        stage.add_to_scene(anchor)?;
        Ok(Self { tracker, anchor })
    }

    fn stop(&self, stage: &mut Stage<'_>) {
        stage.clear_updaters(self.anchor, true);
    }
}

/// Tween `tracker` to `value` over `rt` seconds, played.
fn drive(stage: &mut Stage<'_>, tracker: Mob, value: f64, rt: f64) -> fmn::Result<()> {
    stage.play(tween(tracker, value, rt)?)?;
    Ok(())
}

/// Bring filled parts already in the scene to `opacity` together.
fn reveal(stage: &mut Stage<'_>, mobs: &[(Mob, f64)], rt: f64) -> fmn::Result<()> {
    let moves = mobs
        .iter()
        .map(|&(m, o)| fade_to(m, o, rt))
        .collect::<fmn::Result<Vec<_>>>()?;
    together(stage, moves)
}

/// The tile turned `degrees` past straight at the centre between `from`
/// and `to`, `u` of the way (a move that may break an assumption: 04_7).
fn between(from: (f64, f64), to: (f64, f64), degrees: f64, u: f64) -> Tile<f64> {
    let a = (degrees * u).to_radians();
    Tile {
        centre: (from.0 + (to.0 - from.0) * u, from.1 + (to.1 - from.1) * u),
        cos: a.cos(),
        sin: a.sin(),
    }
}

/// The box of 2.71 at chapter 01's scale: its outline, the centre region,
/// the four corner blocks this lemma settles, the cross left between them,
/// and Friedman's four points. Detached, in drawing order.
fn box_view(stage: &mut Stage<'_>, side: f64) -> Vec<Mob> {
    let at = |q: (f64, f64)| BOARD.p(side, q);
    let rect = |stage: &mut Stage<'_>, x0: f64, y0: f64, x1: f64, y1: f64, c: Srgb, o: f64| {
        polygon(
            stage,
            vec![at((x0, y0)), at((x1, y0)), at((x1, y1)), at((x0, y1))],
            flat(c, o),
        )
    };
    let (lo, hi) = (0.5, side - 0.5);
    let (near, far) = (1.0, side - 1.0);
    let mut out = vec![
        rect(stage, lo, near, hi, far, faint(), 1.0),
        rect(stage, near, lo, far, hi, faint(), 1.0),
    ];
    for (x0, x1) in [(lo, near), (far, hi)] {
        for (y0, y1) in [(lo, near), (far, hi)] {
            out.push(rect(stage, x0, y0, x1, y1, touch(), 0.6));
        }
    }
    out.push(path(stage, BOARD.outline(side), line_style(ink(), 4.0)));
    out.push(path(
        stage,
        vec![at((lo, lo)), at((hi, lo)), at((hi, hi)), at((lo, hi)), at((lo, lo))],
        line_style(dim(), 2.0),
    ));
    for p in model::corner_points(model::turned_side()) {
        out.push(dot(stage, at((p.0.to_f64(), p.1.to_f64())), POINT_R, Palette::purple()));
    }
    out
}

impl SceneConstruct for TheCornerLemma {
    fn name(&self) -> &str {
        "04_the_corner_lemma"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let r = reads()?;
        let at = |q: (f64, f64)| CORNER.p(VIEW, q);
        let point = model::corner_point::<f64>();

        // filled parts enter the scene now, unseen, layered; strokes stay
        // detached and fade in when their beat comes
        let hidden = || vec![at(point); 3];
        let allowed = polygon(stage, hidden(), flat(Palette::purple(), 0.3));
        let tile = polygon(stage, hidden(), flat(corner(), 1.0));
        let shadow = polygon(stage, hidden(), flat(Palette::purple(), 0.85));
        let centre = dot(stage, at(point), CENTRE_R, ink());
        let point_dot = dot(stage, at(point), POINT_R, Palette::purple());
        stage.set_z_index(allowed, LAYER_UNDER, true);
        for &m in &[centre, point_dot] {
            stage.set_z_index(m, LAYER_MARKS, true);
        }
        for &m in &[allowed, tile, shadow, centre, point_dot] {
            stage.add_to_scene(m)?;
            stage.set_opacity(m, 0.0, true);
        }
        let stroke = |stage: &mut Stage<'_>, c: Srgb, w: f64| path(stage, vec![at(point), at(point)], line_style(c, w));
        let across = stroke(stage, dim(), 3.0);
        let up = stroke(stage, dim(), 3.0);
        let side_u = stroke(stage, ink(), 4.0);
        let side_v = stroke(stage, ink(), 4.0);
        let ring_mob = ring(stage, at(point), RING_R_POINT, touch(), 5.0);
        for &m in &[across, up, side_u, side_v] {
            stage.set_z_index(m, LAYER_MARKS, true);
        }
        stage.set_z_index(ring_mob, LAYER_RING, true);
        let parts = Parts {
            tile,
            centre,
            shadow,
            allowed,
            across,
            up,
            side_u,
            side_v,
            ring: ring_mob,
        };
        let pose = Rc::new(Cell::new([0.5, 0.5, 0.0]));
        let walls = [
            path(stage, vec![at((0.0, 0.0)), at((VIEW, 0.0))], line_style(ink(), 6.0)),
            path(stage, vec![at((0.0, 0.0)), at((0.0, VIEW))], line_style(ink(), 6.0)),
        ];
        let unit = path(
            stage,
            vec![
                at((0.0, 0.0)),
                at((1.0, 0.0)),
                at((1.0, 1.0)),
                at((0.0, 1.0)),
                at((0.0, 0.0)),
            ],
            line_style(dim(), 2.0),
        );
        for &m in &walls {
            stage.set_z_index(m, LAYER_MARKS, true);
        }

        // ---------------------------------------------------------- 04_1
        // a corner, the point, a tile in the corner
        let wander = Motion::install(stage, parts, &pose, 0.0, Box::new(model::corner_wander))?;
        let line = say(&kit, stage, LINES[0])?;
        swap(stage, &[], &walls, 0.6)?;
        line.wait_for(stage, "points no tile")?;
        reveal(stage, &[(point_dot, 1.0)], 0.5)?;
        line.wait_for(stage, "Start with a tile")?;
        reveal(stage, &[(tile, 1.0), (centre, 1.0)], 0.6)?;
        swap(stage, &[], &[ring_mob], 0.4)?;

        // ---------------------------------------------------------- 04_2
        // centres within one tile of both walls; every turn, every place
        let line = say(&kit, stage, LINES[1])?;
        swap(stage, &[], &[unit], 0.6)?;
        line.wait_for(stage, "it always covers")?;
        let rt = line.left(stage, 2.5) - 0.2;
        drive(stage, wander.tracker, 0.5, rt)?;

        // ---------------------------------------------------------- 04_3
        // the wider shadow; the question; four seconds to think
        let line = say(&kit, stage, LINES[2])?;
        let readout_x = TEXT_X + 2.4;
        let turn_label = kit.text_left(stage, TURN, 30.0, dim(), v(TEXT_X, 3.0))?;
        let shadow_label = kit.text_left(stage, SHADOW, 30.0, dim(), v(TEXT_X, 2.45))?;
        let degree = kit.text_left(stage, DEGREE, 34.0, turned(), v(readout_x + 0.06, 3.1))?;
        reveal(stage, &[(shadow, 0.85)], 0.4)?;
        swap(stage, &[], &[turn_label, shadow_label, degree], 0.4)?;
        let p = Rc::clone(&pose);
        let turn_value = kit.live_value(
            stage,
            move |_: &fmn::mobject::Stage| p.get()[2].to_degrees(),
            0,
            false,
            34.0,
            turned(),
            v(readout_x, 3.0),
            RIGHT,
        )?;
        let p = Rc::clone(&pose);
        let shadow_value = kit.live_value(
            stage,
            move |_: &fmn::mobject::Stage| {
                let th = p.get()[2];
                model::shadow_width((th.cos(), th.sin()))
            },
            2,
            false,
            34.0,
            ink(),
            v(readout_x, 2.45),
            RIGHT,
        )?;
        // the wander ends just before the question, which the think ring
        // then holds
        let ask = line.at("So could")?;
        let rt = (ask - stage.scene().time().to_f64() - 0.1).max(1.5);
        drive(stage, wander.tracker, 1.0, rt)?;
        line.wait_for(stage, "So could")?;
        let track = ring(stage, RING_AT, RING_R, faint(), 6.0);
        let arc = path(stage, arc_points(RING_AT, RING_R, 1.0), line_style(ink(), 7.0));
        swap(stage, &[], &[track, arc], 0.4)?;
        kit.hold(stage)?;
        let mut elapsed = 0.0;
        stage.add_dt_updater(
            arc,
            move |s: &mut fmn::mobject::Stage, m, dt| {
                elapsed += dt;
                retrace(s, m, arc_points(RING_AT, RING_R, 1.0 - elapsed / THINK_S));
            },
            false,
        )?;
        kit.listen(stage, THINK_S, LINES[2])?;

        // ---------------------------------------------------------- 04_4
        // pushed into the corner: the centres the walls allow, the point's
        // distance across and up
        wander.stop(stage);
        let pushed = Motion::install(stage, parts, &pose, 0.0, Box::new(model::corner_pushed))?;
        let line = say(&kit, stage, LINES[3])?;
        swap(stage, &[track, arc], &[across, up], 0.5)?;
        reveal(stage, &[(allowed, 0.3)], 0.5)?;
        let rt = line.left(stage, 2.0) - 0.3;
        drive(stage, pushed.tracker, std::f64::consts::FRAC_PI_4, rt)?;

        // ---------------------------------------------------------- 04_5
        // along the tile's own sides, at most s (1 − s/2) ≤ ½
        let line = say(&kit, stage, LINES[4])?;
        let along = kit.text_left(stage, ALONG, 26.0, dim(), v(TEXT_X, 1.85))?;
        let le = kit.tex_left(stage, AT_MOST, 34.0, ink(), &[], v(TEXT_X + 2.75, 1.38))?;
        let reach = kit.text_left(stage, &r.reach, 34.0, ink(), v(TEXT_X + 3.2, 1.4))?;
        swap(stage, &[], &[side_u, side_v, along, le, reach], 0.5)?;
        let mut along_values = Vec::new();
        for (k, x) in [(0usize, TEXT_X + 1.0), (1, TEXT_X + 2.4)] {
            let p = Rc::clone(&pose);
            along_values.push(kit.live_value(
                stage,
                move |_: &fmn::mobject::Stage| {
                    let [cx, cy, th] = p.get();
                    let d = (1.0 - cx, 1.0 - cy);
                    let axis = if k == 0 {
                        (th.cos(), th.sin())
                    } else {
                        (-th.sin(), th.cos())
                    };
                    (d.0 * axis.0 + d.1 * axis.1).abs()
                },
                2,
                false,
                34.0,
                ink(),
                v(x, 1.4),
                RIGHT,
            )?);
        }
        line.wait_for(stage, "at most the shadow")?;
        let bound = kit.tex(stage, BOUND, 32.0, ink(), &[], v(3.9, 0.55))?;
        let precise = kit.text_left(stage, PRECISELY, 24.0, turned(), v(TEXT_X, 0.0))?;
        let s_is = kit.text_left(
            stage,
            &format!("{S_IS} {}", r.shadow_max),
            24.0,
            dim(),
            v(TEXT_X + 1.35, 0.0),
        )?;
        swap(stage, &[], &[bound, precise, s_is], 0.6)?;
        let rt = ((line.left(stage, 1.6) - 0.2) / 2.0).max(0.8);
        drive(stage, pushed.tracker, 1.3, rt)?;
        drive(stage, pushed.tracker, std::f64::consts::FRAC_PI_4, rt)?;

        // ---------------------------------------------------------- 04_6
        // the reach; straight on a wall, the point on the edge
        let line = say(&kit, stage, LINES[5])?;
        line.wait_for(stage, "Only a straight tile")?;
        drive(stage, pushed.tracker, 0.0, 1.4)?;
        pushed.stop(stage);
        let resting = Motion::install(stage, parts, &pose, 0.5, Box::new(|y| Tile::straight(0.5, y)))?;
        drive(stage, resting.tracker, 0.8, 1.0)?;
        let edge = kit.text_left(stage, EDGE, 26.0, touch(), v(TEXT_X, -0.8))?;
        swap(stage, &[], &[edge], 0.4)?;

        // ---------------------------------------------------------- 04_7
        // drop an assumption and the point escapes: first the centre strays
        // past one tile, then a turned tile pokes through the walls. The
        // bound and the edge status hold only under the assumptions, so
        // they leave before either move; the live geometry stays
        resting.stop(stage);
        let stray = Motion::install(
            stage,
            parts,
            &pose,
            0.0,
            Box::new(|u| between((0.5, 0.8), (1.6, 0.5), 0.0, u)),
        )?;
        let line = say(&kit, stage, LINES[6])?;
        let dropped = kit.text_left(stage, DROPPED, 26.0, warn(), v(TEXT_X, -1.35))?;
        swap(stage, &[le, reach, bound, precise, s_is, edge], &[dropped], 0.4)?;
        let wall = line.at("or drop a wall")?;
        let rt = (wall - stage.scene().time().to_f64() - 0.1).max(1.0);
        drive(stage, stray.tracker, 1.0, rt)?;
        stray.stop(stage);
        let crossing = Motion::install(
            stage,
            parts,
            &pose,
            0.0,
            Box::new(|u| between((1.6, 0.5), (0.5, 0.5), 45.0, u)),
        )?;
        line.wait_for(stage, "drop a wall")?;
        let rt = (line.left(stage, 1.2) - 0.1).max(1.0);
        drive(stage, crossing.tracker, 1.0, rt)?;

        // ---------------------------------------------------------- 04_8
        // the box of 2.71: the corners settled, the edges and middle next
        crossing.stop(stage);
        let line = say(&kit, stage, LINES[7])?;
        let corner_view = [
            tile, centre, shadow, allowed, point_dot, across, up, side_u, side_v, ring_mob, walls[0], walls[1], unit,
        ];
        let panel = [
            along,
            dropped,
            turn_label,
            shadow_label,
            degree,
            turn_value,
            shadow_value,
        ];
        let mut outs: Vec<Mob> = corner_view.to_vec();
        outs.extend(panel);
        outs.extend(along_values);
        swap(stage, &outs, &[], 0.5)?;
        let boxed = box_view(stage, r.side);
        swap(stage, &[], &boxed, 0.7)?;
        line.wait_for(stage, "settles")?;
        let settled = kit.text_left(stage, SETTLED, 24.0, touch(), v(TEXT_X, 1.0))?;
        swap(stage, &[], &[settled], 0.4)?;
        line.wait_for(stage, "The edges")?;
        let next = kit.text_left(stage, NEXT, 24.0, dim(), v(TEXT_X, 0.5))?;
        swap(stage, &[], &[next], 0.4)?;

        // the line and its breath, and the closing reveal stands before the
        // wipe (it comes on the line's last words)
        let up = stage.scene().time().to_f64();
        stand(&kit, stage, up)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_lines_are_the_chapters() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let ids: Vec<&str> = script
            .lines()
            .iter()
            .map(|(id, _)| id.as_str())
            .filter(|id| id.starts_with("04_"))
            .collect();
        assert_eq!(ids, LINES.to_vec());
    }

    /// The corner view and the box view stay in the frame and left of the
    /// panel; the shadow's bar sits under the floor.
    #[test]
    fn the_layout_fits_the_frame() {
        let (lo, hi) = (CORNER.p(VIEW, (0.0, 0.0)), CORNER.p(VIEW, (VIEW, VIEW)));
        assert!(lo[0] > -FRAME_X_RADIUS + 0.3 && lo[1] > -FRAME_Y_RADIUS + 0.3, "{lo:?}");
        assert!(hi[1] < FRAME_Y_RADIUS - 0.3, "{hi:?}");
        assert!(hi[0] < TEXT_X - 0.8, "the corner view clears the panel");
        let bar = CORNER.p(VIEW, (0.0, SHADOW_Y.0))[1];
        assert!(bar < lo[1] && bar > -FRAME_Y_RADIUS + 0.2);
        let side = model::turned_side().to_f64();
        assert!(
            BOARD.p(side, (side, side))[0] < TEXT_X - 0.8,
            "the box clears the panel"
        );
    }

    /// The moves of 04_7 break exactly the assumption they are meant to:
    /// the first keeps the walls and loses the point once the centre is past
    /// one tile; the second ends through both walls, losing the point.
    #[test]
    fn the_failures_break_their_assumption() {
        let point = model::corner_point::<f64>();
        let end = between((0.5, 0.8), (1.6, 0.5), 0.0, 1.0);
        assert!(model::in_quadrant(&end));
        assert!(model::depth_inside(&end, point) < -EDGE_BAND);
        let crossed = between((1.6, 0.5), (0.5, 0.5), 45.0, 1.0);
        assert!(!model::in_quadrant(&crossed));
        assert!(model::depth_inside(&crossed, point) < -EDGE_BAND);
        // the straight slide holds the point while the centre is within one
        // tile and a half of the wall, and loses it after
        let start = between((0.5, 0.8), (1.6, 0.5), 0.0, 0.0);
        assert!(model::depth_inside(&start, point) >= -EDGE_BAND);
    }
}
