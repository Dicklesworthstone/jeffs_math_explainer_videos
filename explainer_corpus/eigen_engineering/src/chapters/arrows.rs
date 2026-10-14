//! The arrow plane, the picture that carries the idea of the video
//! (storyboard "The one picture"): a move drawn as a blue arrow, the push
//! that holds it as an orange one recomputed from the move every frame, the
//! angle between them as a readout, and gold when they line up. It returns
//! in 03, 04, 12_5, 13 and 14_5.
//!
//! The plane is `draw::Plane`: the left cart's move across, the right cart's
//! up, in centimetres (pushes in newtons share the axes). Everything that
//! moves is rewritten by one updater from trackers, only when they change.
#![allow(dead_code, reason = "shared props land before the chapters that draw them")]

use explainer_kit::{Kit, Palette};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::draw::{Map2, Plane, apply, arrow_points, corners, faded_line, group, moves, own, pushes};
use crate::model::carts::turn_between;

/// A tracker's value as an updater reads it (0 for a tracker that is gone).
fn value(s: &Arena, m: Mob) -> f64 {
    s.tracker_value(m).unwrap_or(0.0)
}

/// The angle, in degrees, between a move `u` and the push `table` gives
/// back for it (0 when they line up, and when either is zero).
pub fn turn_deg(table: &Map2, u: [f64; 2]) -> f64 {
    let f = apply(table, u);
    if u[0].hypot(u[1]) < 1e-9 || f[0].hypot(f[1]) < 1e-9 {
        return 0.0;
    }
    turn_between(u, f)
}

/// The direction `(cos, sin)` of an angle in degrees.
pub fn unit_at(degrees: f64) -> [f64; 2] {
    let r = degrees.to_radians();
    [r.cos(), r.sin()]
}

/// The two ends of a band along direction `dir` through the origin, long
/// enough to span a box of `half_w` by `half_h` (plane units): where the
/// line leaves the box.
pub fn band_ends(dir: [f64; 2], half_w: f64, half_h: f64) -> ([f64; 2], [f64; 2]) {
    let len = dir[0].hypot(dir[1]);
    let (dx, dy) = (dir[0] / len, dir[1] / len);
    let tx = if dx.abs() < 1e-12 {
        f64::INFINITY
    } else {
        half_w / dx.abs()
    };
    let ty = if dy.abs() < 1e-12 {
        f64::INFINITY
    } else {
        half_h / dy.abs()
    };
    let t = tx.min(ty);
    ([-dx * t, -dy * t], [dx * t, dy * t])
}

/// A sideways shift of `amount` for an arrow pointing along `along`
/// (rotated a quarter turn counter-clockwise).
pub fn sideways(along: [f64; 2], amount: f64) -> [f64; 2] {
    let len = along[0].hypot(along[1]);
    if len < 1e-12 {
        return [0.0, 0.0];
    }
    [-along[1] / len * amount, along[0] / len * amount]
}

/// A mix of two colours, `t` of the way from `a` to `b`.
pub fn mix(a: Srgb, b: Srgb, t: f64) -> Srgb {
    let t = t.clamp(0.0, 1.0);
    Srgb {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
    }
}

// ------------------------------------------------------------ the arrow test

/// The arrow test on a plane: the move `u` (blue), the push the table gives
/// back for it (orange, drawn thinner inside the blue), faint drop lines
/// from the move's tip to its two numbers on the axes.
pub struct ArrowTest {
    pub plane: Plane,
    pub table: Map2,
    /// The move, in plane units: trackers, shared with whatever else the
    /// move drives (the carts above).
    pub u: [Mob; 2],
    /// Drive the move from an angle and a length instead (the sweep): when
    /// `polar` reads above one half, `u` is `radius` at `degrees`.
    pub polar: Mob,
    pub degrees: Mob,
    pub radius: Mob,
    /// How much of the blue arrow, the orange arrow and the drop lines
    /// shows, 0 to 1.
    pub blue_on: Mob,
    pub orange_on: Mob,
    pub drops_on: Mob,
    /// How far the orange arrow is slid sideways out from under the blue
    /// one, plane units.
    pub slide: Mob,
    pub axes: Vec<Mob>,
    pub blue: Mob,
    pub orange: Mob,
    pub drops: Mob,
    anchor: Mob,
}

impl ArrowTest {
    /// Build the plane's axes, arrows and drop lines, detached, with the
    /// updater ready ([`ArrowTest::start`] adds it to the scene).
    pub fn build(
        stage: &mut Stage<'_>,
        plane: Plane,
        table: Map2,
        u: [Mob; 2],
        reach: (f64, f64),
    ) -> fmn::Result<Self> {
        let axes = plane.axes(stage, reach.0, reach.1);
        let blue = stage.arena_mut().add(
            polyline(arrow_points(plane.p(0.0, 0.0), plane.p(1e-3, 0.0), 0.28))
                .style(faded_line(moves(), 8.0, 0.0))
                .build(),
        );
        let orange = stage.arena_mut().add(
            polyline(arrow_points(plane.p(0.0, 0.0), plane.p(1e-3, 0.0), 0.2))
                .style(faded_line(pushes(), 4.0, 0.0))
                .build(),
        );
        let drops = stage.arena_mut().add(
            polyline(vec![plane.p(0.0, 0.0), plane.p(0.0, 0.0), plane.p(0.0, 0.0)])
                .style(faded_line(Palette::dim(), 2.0, 0.0))
                .build(),
        );
        let polar = stage.add_value_tracker(0.0);
        let degrees = stage.add_value_tracker(0.0);
        let radius = stage.add_value_tracker(1.0);
        let (blue_on, orange_on, drops_on, slide) = (
            stage.add_value_tracker(0.0),
            stage.add_value_tracker(0.0),
            stage.add_value_tracker(0.0),
            stage.add_value_tracker(0.0),
        );
        let anchor = stage.arena_mut().add(VMobject::new());
        let mut last = [f64::NAN; 9];
        stage.add_updater(
            anchor,
            move |s, _| {
                if value(s, polar) > 0.5 {
                    let d = unit_at(value(s, degrees));
                    let r = value(s, radius);
                    let _ = s.set_tracker_value(u[0], r * d[0]);
                    let _ = s.set_tracker_value(u[1], r * d[1]);
                }
                let mv = [value(s, u[0]), value(s, u[1])];
                let now = [
                    mv[0],
                    mv[1],
                    value(s, blue_on),
                    value(s, orange_on),
                    value(s, drops_on),
                    value(s, slide),
                    value(s, polar),
                    value(s, degrees),
                    value(s, radius),
                ];
                if now.iter().zip(&last).all(|(a, b)| a.to_bits() == b.to_bits()) {
                    return;
                }
                last = now;
                let push = apply(&table, mv);
                let off = sideways(push, now[5]);
                let origin = plane.p(0.0, 0.0);
                let _ = s.set_points(blue, &corners(&arrow_points(origin, plane.p(mv[0], mv[1]), 0.28)));
                s.set_stroke(blue, None, None, Some(now[2]), None, false);
                let _ = s.set_points(
                    orange,
                    &corners(&arrow_points(
                        plane.p(off[0], off[1]),
                        plane.p(push[0] + off[0], push[1] + off[1]),
                        0.22,
                    )),
                );
                s.set_stroke(orange, None, None, Some(now[3]), None, false);
                let _ = s.set_points(
                    drops,
                    &corners(&[plane.p(mv[0], 0.0), plane.p(mv[0], mv[1]), plane.p(0.0, mv[1])]),
                );
                s.set_stroke(drops, None, None, Some(0.7 * now[4]), None, false);
            },
            true,
        )?;
        Ok(Self {
            plane,
            table,
            u,
            polar,
            degrees,
            radius,
            blue_on,
            orange_on,
            drops_on,
            slide,
            axes,
            blue,
            orange,
            drops,
            anchor,
        })
    }

    /// Add the updater and the arrows (at opacity 0 until their trackers
    /// rise) to the scene; the axes are faded in by the chapter.
    pub fn start(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.add_to_scene(self.anchor)?;
        for m in [self.drops, self.blue, self.orange] {
            stage.add_to_scene(m)?;
        }
        Ok(())
    }

    /// Stop the updater.
    pub fn stop(&self, stage: &mut Stage<'_>) {
        stage.clear_updaters(self.anchor, true);
    }

    /// The move and its push right now, in plane units.
    pub fn now(&self, stage: &Stage<'_>) -> ([f64; 2], [f64; 2]) {
        let mv = [
            stage.tracker_value(self.u[0]).unwrap_or(0.0),
            stage.tracker_value(self.u[1]).unwrap_or(0.0),
        ];
        (mv, apply(&self.table, mv))
    }
}

// ------------------------------------------------------------- own lines

/// A wide soft gold band and a thin gold core along each direction in
/// `dirs`, through the plane's origin, spanning `half_w` by `half_h` plane
/// units; detached, opacity 0 (bring them in with a tracker or a fade).
pub fn own_lines(stage: &mut Stage<'_>, plane: Plane, dirs: &[[f64; 2]], half_w: f64, half_h: f64) -> Vec<Mob> {
    let mut out = Vec::new();
    for &d in dirs {
        let (a, b) = band_ends(d, half_w, half_h);
        let (a, b) = (plane.p(a[0], a[1]), plane.p(b[0], b[1]));
        let soft = polyline(vec![a, b]).style(faded_line(own(), 26.0, 0.0)).build();
        let core = polyline(vec![a, b]).style(faded_line(own(), 3.0, 0.0)).build();
        out.push(stage.arena_mut().add(soft));
        out.push(stage.arena_mut().add(core));
    }
    out
}

// -------------------------------------------------------- a plane arrow

/// One arrow on the plane from tracker-driven ends, whose colour passes from
/// `from_colour` to `to_colour` as `mix` runs 0 to 1: the pieces of 04's
/// split (orange turning blue), the chain of 13's rounds.
pub struct PlaneArrow {
    pub mob: Mob,
    /// Tail and tip, in plane units.
    pub tail: [Mob; 2],
    pub tip: [Mob; 2],
    /// 0 for `from_colour`, 1 for `to_colour`.
    pub mix: Mob,
    /// Opacity, 0 to 1.
    pub on: Mob,
    anchor: Mob,
}

impl PlaneArrow {
    /// An arrow from `tail` to `tip`, detached; its updater is added to the
    /// scene with [`PlaneArrow::start`].
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        stage: &mut Stage<'_>,
        plane: Plane,
        from_colour: Srgb,
        to_colour: Srgb,
        width: f64,
        head: f64,
        tail: [f64; 2],
        tip: [f64; 2],
    ) -> fmn::Result<Self> {
        let mob = stage.arena_mut().add(
            polyline(arrow_points(plane.p(tail[0], tail[1]), plane.p(tip[0], tip[1]), head))
                .style(faded_line(from_colour, width, 0.0))
                .build(),
        );
        let tails = [stage.add_value_tracker(tail[0]), stage.add_value_tracker(tail[1])];
        let tips = [stage.add_value_tracker(tip[0]), stage.add_value_tracker(tip[1])];
        let (mixer, on) = (stage.add_value_tracker(0.0), stage.add_value_tracker(0.0));
        let anchor = stage.arena_mut().add(VMobject::new());
        let mut last = [f64::NAN; 6];
        stage.add_updater(
            anchor,
            move |s, _| {
                let now = [
                    value(s, tails[0]),
                    value(s, tails[1]),
                    value(s, tips[0]),
                    value(s, tips[1]),
                    value(s, mixer),
                    value(s, on),
                ];
                if now.iter().zip(&last).all(|(a, b)| a.to_bits() == b.to_bits()) {
                    return;
                }
                last = now;
                let _ = s.set_points(
                    mob,
                    &corners(&arrow_points(plane.p(now[0], now[1]), plane.p(now[2], now[3]), head)),
                );
                s.set_stroke(
                    mob,
                    Some(mix(from_colour, to_colour, now[4])),
                    None,
                    Some(now[5]),
                    None,
                    false,
                );
            },
            true,
        )?;
        Ok(Self {
            mob,
            tail: tails,
            tip: tips,
            mix: mixer,
            on,
            anchor,
        })
    }

    pub fn start(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.add_to_scene(self.anchor)?;
        stage.add_to_scene(self.mob)?;
        Ok(())
    }

    pub fn stop(&self, stage: &mut Stage<'_>) {
        stage.clear_updaters(self.anchor, true);
    }
}

/// The labels at the ends of the two axes of the carts' plane, detached:
/// "left cart" along the first axis, "right cart" along the second.
pub fn axis_labels(kit: &Kit, stage: &mut Stage<'_>, plane: Plane, reach: (f64, f64)) -> fmn::Result<Vec<Mob>> {
    let dim = Palette::dim();
    let across = kit.text(stage, "left cart's move", 24.0, dim, plane.p(reach.0 - 0.1, -0.28))?;
    let up = kit.text(stage, "right cart's move", 24.0, dim, plane.p(0.0, reach.1 + 0.3))?;
    Ok(vec![across, up])
}

/// A group of mobjects as one, for fading a whole prop at once.
pub fn together(stage: &mut Stage<'_>, parts: &[Mob]) -> fmn::Result<Mob> {
    group(stage, parts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chapters::draw::{map2, numbers};
    use crate::model::carts::{sweep, table};

    fn k() -> Map2 {
        map2(&table())
    }

    /// 03_4: the push lines up with the move on two lines only, at 45 and
    /// 135 degrees (and their opposites), and is furthest off, 30 degrees,
    /// between them; at the left cart's own move it is 26.6 degrees off.
    #[test]
    fn the_push_lines_up_on_two_lines_and_is_at_most_thirty_degrees_off() {
        let table = k();
        for d in [45.0, 135.0, 225.0, 315.0] {
            assert!(turn_deg(&table, unit_at(d)) < 1e-6, "{d}");
        }
        assert!((turn_deg(&table, unit_at(0.0)) - 26.565).abs() < 1e-3);
        let worst = (0..3600)
            .map(|i| turn_deg(&table, unit_at(f64::from(i) / 10.0)))
            .fold(0.0_f64, f64::max);
        assert!((worst - 30.0).abs() < 1e-3, "{worst}");
        // the model's sweep and the facts say the same
        let (lined, model_worst) = sweep();
        assert_eq!(lined, [45.0, 135.0, 225.0, 315.0]);
        assert!((model_worst - worst).abs() < 1e-9);
        assert_eq!(numbers("K-A2.lined_up_deg").unwrap(), lined);
        assert_eq!(numbers("K-A2.worst_turn_deg").unwrap(), [30.0]);
        // a move of nothing turns by nothing, not NaN
        assert_eq!(turn_deg(&table, [0.0, 0.0]), 0.0);
    }

    /// 03_5: on each own line the push is the move scaled by the eigenvalue:
    /// the same length along "together", three times along "against".
    #[test]
    fn on_the_gold_lines_the_push_is_the_move_times_its_eigenvalue() {
        let table = k();
        let eig = numbers("K-A2.eigenvalues").unwrap();
        let together = unit_at(45.0);
        let against = unit_at(-45.0);
        let (pt, pa) = (apply(&table, together), apply(&table, against));
        assert!((pt[0] - eig[0] * together[0]).abs() < 1e-12 && (pt[1] - eig[0] * together[1]).abs() < 1e-12);
        assert!((pa[0] - eig[1] * against[0]).abs() < 1e-12 && (pa[1] - eig[1] * against[1]).abs() < 1e-12);
    }

    #[test]
    fn a_band_spans_the_box_and_stays_inside_it() {
        let (a, b) = band_ends([1.0, 1.0], 3.0, 2.0);
        // 45 degrees in a 3 by 2 box leaves through the top and bottom
        assert!((b[0] - 2.0).abs() < 1e-12 && (b[1] - 2.0).abs() < 1e-12, "{b:?}");
        assert!((a[0] + b[0]).abs() < 1e-12 && (a[1] + b[1]).abs() < 1e-12);
        let (_, across) = band_ends([1.0, 0.0], 3.0, 2.0);
        assert_eq!(across, [3.0, 0.0]);
        let (_, up) = band_ends([0.0, -5.0], 3.0, 2.0);
        assert!(up[0].abs() < 1e-12 && (up[1] + 2.0).abs() < 1e-12, "{up:?}");
    }

    #[test]
    fn sliding_an_arrow_out_is_sideways_and_by_the_amount() {
        let s = sideways([2.0, 0.0], 0.3);
        assert!(s[0].abs() < 1e-12 && (s[1] - 0.3).abs() < 1e-12);
        let d = sideways([1.0, 1.0], 1.0);
        assert!((d[0] * 1.0 + d[1] * 1.0).abs() < 1e-12, "at right angles to the arrow");
        assert!((d[0].hypot(d[1]) - 1.0).abs() < 1e-12);
        assert_eq!(sideways([0.0, 0.0], 1.0), [0.0, 0.0]);
    }

    #[test]
    fn mixing_runs_from_one_colour_to_the_other() {
        let (a, b) = (pushes(), moves());
        assert_eq!(mix(a, b, 0.0), a);
        assert_eq!(mix(a, b, -3.0), a);
        let m = mix(a, b, 0.5);
        assert!((m.r - (a.r + b.r) / 2.0).abs() < 1e-12);
        let e = mix(a, b, 1.0);
        assert!((e.g - b.g).abs() < 1e-12);
    }

    /// The arrows really follow their trackers: on a bare stage (no
    /// render), the blue arrow's tip is the move, the orange one's the
    /// push the table gives, the polar drive sets the move from an angle,
    /// and a plane arrow's tip follows its trackers; every run written is
    /// one the engine accepts (an odd quad path).
    #[test]
    fn the_arrows_really_follow_the_trackers() {
        use explainer_kit::v;
        crate::chapters::carts::probe::run_scene(|stage| {
            let plane = Plane {
                centre: v(1.0, -0.5),
                unit: 0.7,
            };
            let u = [stage.add_value_tracker(0.0), stage.add_value_tracker(0.0)];
            let test = ArrowTest::build(stage, plane, k(), u, (3.0, 3.0))?;
            test.start(stage)?;
            stage.set_tracker_value(u[0], 1.0)?;
            stage.set_tracker_value(u[1], -1.0)?;
            stage.set_tracker_value(test.blue_on, 1.0)?;
            stage.set_tracker_value(test.orange_on, 1.0)?;
            stage.wait(0.2)?;
            let near = |a: Vec3, b: Vec3| (a[0] - b[0]).abs() < 1e-5 && (a[1] - b[1]).abs() < 1e-5;
            // a five-vertex arrow is nine points; its second vertex, the tip, is the third point
            let blue = stage.get_points(test.blue).expect("the arrow has points");
            assert_eq!((blue.len(), blue.len() % 2), (9, 1));
            assert!(near(blue[2], plane.p(1.0, -1.0)), "{:?}", blue[2]);
            let orange = stage.get_points(test.orange).expect("the arrow has points");
            assert!(near(orange[2], plane.p(3.0, -3.0)), "{:?}", orange[2]);
            // the polar drive: at 45 degrees and length 1 the move is (1, 1) / sqrt 2
            stage.set_tracker_value(test.degrees, 45.0)?;
            stage.set_tracker_value(test.radius, 1.0)?;
            stage.set_tracker_value(test.polar, 1.0)?;
            stage.wait(0.2)?;
            let moved = test.now(stage).0;
            let h = std::f64::consts::FRAC_1_SQRT_2;
            assert!((moved[0] - h).abs() < 1e-9 && (moved[1] - h).abs() < 1e-9, "{moved:?}");
            // a plane arrow's tip follows its trackers
            let a = PlaneArrow::build(stage, plane, pushes(), moves(), 6.0, 0.2, [0.0, 0.0], [0.0, 0.0])?;
            a.start(stage)?;
            stage.set_tracker_value(a.tip[0], 0.5)?;
            stage.set_tracker_value(a.tip[1], 0.5)?;
            stage.set_tracker_value(a.on, 1.0)?;
            stage.wait(0.2)?;
            let pts = stage.get_points(a.mob).expect("the arrow has points");
            assert_eq!(pts.len() % 2, 1);
            assert!(near(pts[2], plane.p(0.5, 0.5)), "{:?}", pts[2]);
            Ok(())
        });
    }
}
