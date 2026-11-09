//! 03_two_planes: a complex function drawn as two planes, input and output
//! (storyboard 03_1–03_7).
//!
//! - A point and its image; the two framed planes.
//! - Squaring: a walk once around |z| = ½ whose image goes twice around
//!   |w| = ¼, with live turn counts.
//! - The prediction on a grid; then every line morphs pointwise into its
//!   image under z²: parabolas, the axes folded onto the two real rays, and
//!   right angles kept at three crossings but not at 0.
//! - 1/z: near and far trade places; the unit circle flips across the real
//!   axis; 0 has no image.
//! - e^z: right by one multiplies the length by e, up turns it; vertical
//!   lines become circles, horizontal lines rays; a climb of 2π returns to
//!   the same output, which is never 0.

use std::f64::consts::{PI, TAU};

use explainer_kit::{Kit, Palette, tween, v};
use fmn::prelude::*;

use super::draw::{
    Plane, Walker, carded_lines, dot, fact, fact_list, fade_in_all, fade_out_all, hero, input, line_style, output,
    pole, predict, say, tween_linear, value, wait_until, warn,
};
// "f(z) = e^z" is the gallery's title for the same map; "radius " the bump's label
use super::gallery::TITLES_TEX as GALLERY_TITLES;
use super::the_bump::RADIUS;
use crate::model::{C, Func, I, ONE, ZERO};

pub const TITLES: [&str; 2] = ["input z", "output f(z)"];
pub const GRAPH: &str = "a graph would need four dimensions";
pub const FN_TEX: [&str; 2] = [r"f(z) = z^2", r"f(z) = \frac{1}{z}"];
pub const SQUARE_RULE_TEX: &str = r"|z^2| = |z|^2,\quad \arg z^2 = 2 \arg z\;(\mathrm{mod}\;2\pi),\;z\ne0";
pub const TURNS_LABEL: &str = "turns:";
pub const ASK: [&str; 1] = ["straight after z²?"];
/// In the voice's own words: it says "the horizontal axis" and "the vertical axis".
pub const RAYS: [&str; 2] = ["horizontal axis → positive ray", "vertical axis → negative ray"];
pub const KEPT: &str = "right angles kept, except at 0";
pub const TRADE: &str = "inside → outside, outside → inside";
pub const UNDEFINED: &str = "1/0 undefined";
pub const TIMES_E_TEX: &str = r"\times e";
pub const PERIOD_TEX: &str = r"e^{z + 2\pi i} = e^z";
pub const NONZERO_TEX: &str = r"e^z \ne 0";
pub const AFTER_TEX: &str = r"e^{2\pi i} = ";
/// What the two points of 03_1 are called.
pub const POINT_TEX: [&str; 2] = ["z", "f(z)"];
/// A plane's names for its units.
pub const UNIT_TEX: [&str; 2] = ["1", "i"];
/// The height a climb reaches (03_6, 03_7).
pub const CLIMB_TEX: &str = r"2\pi i";
/// The one change of scale in the chapter, said where it happens.
pub const RESCALED: &str = "new windows: the ticks still mark whole numbers";

/// The two planes: input on the left, output on the right; both 5.4 wide at
/// the same height, so their frames stay put. Every window is ticked at the
/// whole numbers (`scale_marks`), and each beat uses one scale on both sides.
pub fn planes(input_half: f64, output_half: f64) -> (Plane, Plane) {
    (
        Plane::new(
            v(-3.5, -0.35),
            5.4,
            (-input_half, input_half),
            (-input_half, input_half),
        ),
        Plane::new(
            v(3.5, -0.35),
            5.4,
            (-output_half, output_half),
            (-output_half, output_half),
        ),
    )
}
/// The one window of 03_1 – 03_5, on both planes, so a unit keeps its length from beat to
/// beat: ±2.3 holds the grid's images (up to ±2) and 1/z's farthest image (1/0.45).
pub const WINDOW: f64 = 2.3;
/// The walk of 03_2 at one scale on both sides, so radius ¼ is drawn half
/// as large as radius ½.
pub const WALK_HALVES: (f64, f64) = (WINDOW, WINDOW);
/// The grid of 03_3 – 03_4, one scale on both sides: the images reach ±2,
/// and each copy starts exactly on top of its input line's shape.
pub const SQUARE_HALVES: (f64, f64) = (WINDOW, WINDOW);
/// 1/z's windows (03_5).
pub const INVERSE_HALVES: (f64, f64) = (WINDOW, WINDOW);
/// e^z's output window (03_6, 03_7), on the same frame: wider, because e^1.15 is past 3. The
/// ticks fade and return at the new spacing, and `RESCALED` says so.
pub const EXP_OUT_HALF: f64 = 3.2;

/// e^z's input window: tall enough for a climb of 2π, with the same top
/// and bottom as the other frames.
pub fn exp_input() -> Plane {
    Plane::new(v(-3.5, -0.35), 3.6, (-2.4, 2.4), (-0.4, 6.8))
}

/// Half a tick's length, scene units.
pub const TICK: f64 = 0.06;
/// Where "1" and "i" are written: off both axes, off the unit circle, and clear of the
/// images of the grid.
pub const UNIT_AT: [C; 2] = [C::new(1.17, -0.22), C::new(-0.22, 1.17)];
/// Where "2πi" is written beside its tick on e^z's input plane.
pub const CLIMB_NAME_AT: C = C::new(-0.5, TAU);
/// Where "turns:" and its value read under each plane: both on one side of the plane's axis.
pub const TURNS_AT: [(f64, f64); 2] = [(-5.0, -4.2), (2.0, 2.8)];
pub const TURNS_Y: f64 = -2.65;
/// Where the radii of 03_2 are written: beside their circles, off both axes.
pub const RADIUS_AT: [C; 2] = [C::new(0.8, -0.66), C::new(0.7, -0.5)];
/// Where the output's origin is named (03_4), in the gap the images leave around it.
pub const FOLD_NAME_AT: C = C::new(-0.09, -0.18);
/// The 1/0 card (03_5): outside the unit circle, with a leader back to the crossed-out 0.
pub const UNDEFINED_AT: C = C::new(-0.55, -1.6);
/// Half the 1/0 card's height, in plane units (the card is 0.56 tall at this window).
#[cfg(test)]
pub const UNDEFINED_HALF_H: f64 = 0.24;
/// The leader, from the card's top edge to just short of the cross: slanted, so it is not
/// read as a stretch of the vertical axis.
pub const UNDEFINED_LEADER: [C; 2] = [C::new(-0.25, -1.36), C::new(-0.1, -0.2)];

/// Where the one-line captions sit, under both frames.
pub const CAPTION_Y: f64 = -3.5;
/// The 03_3 question: its countdown ring clear of the output's two axes, and the card itself
/// between the "1" above it and the tick of −i below it, so it hides no mark of the scale.
pub const ASK_AT: C = C::new(0.55, -0.65);
/// Where e^{2πi}'s computed value is carded in the output plane (03_7).
pub const LANDED_AT: C = C::new(0.0, 1.9);

/// The walk of 03_2: once around |z| = ½.
pub const WALK_R: f64 = 0.5;

/// The one point that shows the rule of 03_2 before the walk: its length and angle on the left,
/// the length squared and the angle doubled on the right. A rule's picture, so it carries the
/// rule's own letters and no number the facts would have to pin.
pub const RULE_LENGTH: f64 = 1.4;
pub const RULE_ANGLE: f64 = PI / 6.0;
/// The arcs that mark the two angles, in plane units.
pub const RULE_ARC_R: f64 = 0.55;
/// The rule's pieces as `SQUARE_RULE_TEX` writes them: input's, then output's.
pub const RULE_LENGTH_TEX: [&str; 2] = [r"|z|", r"|z|^2"];
pub const RULE_ANGLE_TEX: [&str; 2] = [r"\arg z", r"2 \arg z"];
/// Where those are written: each length past its arrow's tip, each angle inside its arrow.
pub const RULE_LENGTH_AT: [C; 2] = [C::new(1.5, 0.85), C::new(1.35, 1.75)];
pub const RULE_ANGLE_AT: [C; 2] = [C::new(0.92, 0.25), C::new(0.82, 0.48)];
/// How big those are written.
pub const RULE_LABEL_SIZE: f64 = 22.0;

/// The point of the rule's picture.
pub fn rule_point() -> C {
    C::polar(RULE_LENGTH, RULE_ANGLE)
}

/// An arc of radius `RULE_ARC_R` about 0, from angle 0 to `to`.
pub fn rule_arc(to: f64) -> Vec<C> {
    (0..=32)
        .map(|k| C::polar(RULE_ARC_R, to * f64::from(k) / 32.0))
        .collect()
}

/// How big the radii of 03_2 are written: they are spoken numbers.
pub const RADIUS_SIZE: f64 = 24.0;

/// The image of the walk of 03_2 at its share `t`, as the two laps it makes: the first half of
/// the walk draws the first, the second half draws the second over it, so "twice" can be seen.
/// Each lap has at least two points, because a polyline needs them.
pub fn image_laps(t: f64) -> (Vec<C>, Vec<C>) {
    let t = t.clamp(0.0, 1.0);
    let share = |from: f64, to: f64| -> Vec<C> {
        let (a, b) = (from, to.max(from + 1e-6));
        let k = (((b - a) * 240.0).ceil() as usize).max(1);
        (0..=k)
            .map(|j| {
                let z = C::polar(WALK_R, TAU * (a + (b - a) * j as f64 / k as f64));
                z * z
            })
            .collect()
    };
    (share(0.0, t.min(0.5)), share(0.5, t.max(0.5)))
}

/// The grid of 03_3: x and y in [−1, 1], step ½. Vertical lines first,
/// then horizontal, each as `n` + 1 points; the axes are the k = 2 lines.
pub fn grid(n: usize) -> (Vec<Vec<C>>, Vec<Vec<C>>) {
    let line = |a: C, b: C| {
        (0..=n)
            .map(|j| a + (b - a).scale(j as f64 / n as f64))
            .collect::<Vec<C>>()
    };
    let vertical = (-2..=2)
        .map(|k| f64::from(k) / 2.0)
        .map(|x| line(C::new(x, -1.0), C::new(x, 1.0)))
        .collect();
    let horizontal = (-2..=2)
        .map(|k| f64::from(k) / 2.0)
        .map(|y| line(C::new(-1.0, y), C::new(1.0, y)))
        .collect();
    (vertical, horizontal)
}

/// The three crossings whose right angles are marked.
pub const CROSSINGS: [C; 3] = [C::new(0.5, 0.5), C::new(-1.0, 0.5), C::new(0.5, -1.0)];

/// A right-angle mark at `at` with legs along `d` and `i·d`, `size` long.
pub fn right_mark(at: C, d: C, size: f64) -> Vec<C> {
    let d1 = d.scale(size / d.abs());
    let d2 = d1 * I;
    vec![at + d1, at + d1 + d2, at + d2]
}

/// The points of 03_5's ticks on the unit circle.
pub const TICKS: [f64; 4] = [0.6, 2.0, 3.6, 5.0];
/// The ring left where each tick would have stood: the size of its dot, scene units.
pub const FLIP_RING: f64 = 0.1;
/// How much of each flip arrow's ends is left to the ring and the dot, scene units.
pub const FLIP_BUFF: f64 = 0.1;

/// Where the tick at angle `th` would stand in the output plane, and where 1/z puts it.
pub fn flip_chord(th: f64) -> (C, C) {
    let z = C::polar(1.0, th);
    (z, z.recip())
}

/// The point that walks out and back along a ray in 03_5: its distance
/// from 0 over the walk's share `t`.
pub fn ray_distance(t: f64) -> f64 {
    let (near, far) = (0.45, 2.2);
    near + (far - near) * (PI * t).sin()
}
pub const RAY_ANGLE: f64 = 0.6;

/// The lines of 03_6: vertical x = −1, 0, 1 for 0 ≤ y ≤ 2π, and
/// horizontal y = π/4, 3π/4, 5π/4, 7π/4 for −1.5 ≤ x ≤ 1.15. Those heights keep the rays
/// off the output's axes, which are straight lines from 0 already.
pub fn exp_lines(n: usize) -> (Vec<Vec<C>>, Vec<Vec<C>>) {
    let line = |a: C, b: C| {
        (0..=n)
            .map(|j| a + (b - a).scale(j as f64 / n as f64))
            .collect::<Vec<C>>()
    };
    let vertical = [-1.0, 0.0, 1.0]
        .into_iter()
        .map(|x| line(C::new(x, 0.0), C::new(x, TAU)))
        .collect();
    let horizontal = (0..4)
        .map(|k| (f64::from(k) + 0.5) * PI / 2.0)
        .map(|y| line(C::new(-1.5, y), C::new(1.15, y)))
        .collect();
    (vertical, horizontal)
}

/// Half a radius readout's width in e^z's output window (five characters at 18, CMR10).
pub const RADIUS_LABEL_HALF: f64 = 0.243;

/// Where the radius readout of e^x (x = −1, 0, 1 as `k` = 0, 1, 2) sits in the output plane:
/// under the real axis, beside its dot and between circles, so no circle runs through it. The
/// largest circle has no room outside it, so its readout stands just inside.
pub fn radius_label_at(k: usize, r: f64) -> C {
    let x = if k == 2 {
        r - 0.08 - RADIUS_LABEL_HALF
    } else {
        r + 0.04 + RADIUS_LABEL_HALF
    };
    C::new(x, -0.2)
}

/// The whole numbers inside `span`, without 0: where a plane's ticks go.
pub fn whole_ticks(span: (f64, f64)) -> Vec<f64> {
    (span.0.ceil() as i32..=span.1.floor() as i32)
        .filter(|&k| k != 0)
        .map(f64::from)
        .collect()
}

/// Ticks at the whole numbers along both axes of `p`, and the names in `UNIT_AT` listed in
/// `names`: the plane's scale, on the plane.
fn scale_marks(kit: &Kit, stage: &mut Stage<'_>, p: &Plane, names: &[usize]) -> fmn::Result<Vec<Mob>> {
    let tick = |stage: &mut Stage<'_>, a: Vec3, b: Vec3| -> Mob {
        stage
            .arena_mut()
            .add(polyline(vec![a, b]).style(line_style(Palette::dim(), 2.0)).build())
    };
    let mut out = Vec::new();
    for x in whole_ticks(p.frame.x) {
        let at = p.p(C::real(x));
        out.push(tick(stage, v(at[0], at[1] - TICK), v(at[0], at[1] + TICK)));
    }
    for y in whole_ticks(p.frame.y) {
        let at = p.p(C::new(0.0, y));
        out.push(tick(stage, v(at[0] - TICK, at[1]), v(at[0] + TICK, at[1])));
    }
    for &k in names {
        out.push(kit.tex(stage, UNIT_TEX[k], 18.0, Palette::dim(), &[], p.p(UNIT_AT[k]))?);
    }
    Ok(out)
}

/// A red cross at `at`.
fn red_cross(stage: &mut Stage<'_>, at: Vec3) -> Vec<Mob> {
    let d = 0.25;
    let a = polyline(vec![v(at[0] - d, at[1] - d), v(at[0] + d, at[1] + d)])
        .style(line_style(warn(), 7.0))
        .build();
    let b = polyline(vec![v(at[0] - d, at[1] + d), v(at[0] + d, at[1] - d)])
        .style(line_style(warn(), 7.0))
        .build();
    vec![stage.arena_mut().add(a), stage.arena_mut().add(b)]
}

/// A polyline through `zs` in `p`, detached.
fn curve(stage: &mut Stage<'_>, p: &Plane, zs: &[C], color: Srgb, width: f64) -> Mob {
    let pts: Vec<Vec3> = zs.iter().map(|&z| p.p(z)).collect();
    stage
        .arena_mut()
        .add(polyline(pts).style(line_style(color, width)).build())
}

/// The frame of a plane: border in `color` and dim axes.
fn framed(stage: &mut Stage<'_>, p: &Plane, color: Srgb) -> fmn::Result<Vec<Mob>> {
    let mut out = vec![p.border(stage, color)?];
    out.extend(p.axes(stage)?);
    Ok(out)
}

/// Morph each curve (drawn in `out` from its points `zs`) pointwise into
/// `f(zs)`: (1 − s)·z + s·f(z), s from 0 to 1 over `rt`.
fn morph(stage: &mut Stage<'_>, out: Plane, items: &[(Mob, Vec<C>)], f: fn(C) -> C, rt: f64) -> fmn::Result<()> {
    let s = stage.add_value_tracker(0.0);
    let items = items.to_vec();
    let anchor = stage.arena_mut().add(VMobject::new());
    stage.add_to_scene(anchor)?;
    stage.add_updater(
        anchor,
        move |a, _| {
            let t = value(a, s);
            for (m, zs) in &items {
                let pts: Vec<Vec3> = zs.iter().map(|&z| out.p(z.scale(1.0 - t) + f(z).scale(t))).collect();
                let _ = a.set_points(*m, polyline(pts).build().points());
            }
        },
        true,
    )?;
    stage.play(tween(s, 1.0, rt)?)?;
    stage.clear_updaters(anchor, true);
    stage.remove_from_scene(anchor);
    Ok(())
}

/// Draw each curve progressively, the first share t of its points, t from
/// 0 to 1 over `rt` (several at once, in their own planes).
fn reveal(stage: &mut Stage<'_>, items: &[(Mob, Plane, Vec<C>)], rt: f64) -> fmn::Result<()> {
    let t = stage.add_value_tracker(0.0);
    let items = items.to_vec();
    let anchor = stage.arena_mut().add(VMobject::new());
    stage.add_to_scene(anchor)?;
    stage.add_updater(
        anchor,
        move |a, _| {
            let share = value(a, t).clamp(0.0, 1.0);
            for (m, p, zs) in &items {
                let k = ((share * (zs.len() - 1) as f64).ceil() as usize).clamp(1, zs.len() - 1);
                let pts: Vec<Vec3> = zs[..=k].iter().map(|&z| p.p(z)).collect();
                let _ = a.set_points(*m, polyline(pts).build().points());
            }
        },
        true,
    )?;
    stage.play(tween_linear(t, 1.0, rt)?)?;
    stage.clear_updaters(anchor, true);
    stage.remove_from_scene(anchor);
    Ok(())
}

pub struct TwoPlanes {
    pub kit: Kit,
}

impl SceneConstruct for TwoPlanes {
    fn name(&self) -> &str {
        "03_two_planes"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (inp, out) = planes(WALK_HALVES.0, WALK_HALVES.1);
        let fn_at = v(0.0, 3.3);
        let caption_at = v(0.0, CAPTION_Y);

        // 03_1: a point in, a point out; two planes side by side
        let line = say(&kit, stage, "03_1")?;
        let z0 = C::new(0.8, 0.6);
        let in_dot = dot(stage, inp.p(z0), 0.09, input());
        let out_dot = dot(stage, out.p(z0 * z0), 0.09, output());
        let carry = Arrow::new(inp.p(z0), out.p(z0 * z0))
            .thickness(3.0)
            .color(Palette::dim())
            .build()?;
        let carry = stage.arena_mut().add(carry);
        let z_name = kit.tex(stage, POINT_TEX[0], 28.0, input(), &[], inp.p(z0 + C::new(0.0, 0.3)))?;
        let w_name = kit.tex(
            stage,
            POINT_TEX[1],
            28.0,
            output(),
            &[],
            out.p(z0 * z0 + C::new(0.0, 0.32)),
        )?;
        fade_in_all(stage, &[in_dot, z_name], 0.3)?;
        fade_in_all(stage, &[carry, out_dot, w_name], 0.5)?;
        line.wait_sentence(stage, 1)?;
        let graph = kit.text(stage, GRAPH, 22.0, Palette::dim(), caption_at)?;
        fade_in_all(stage, &[graph], 0.4)?;
        line.wait_sentence(stage, 2)?;
        // the planes arrive, and the arrow that would cross both their frames goes
        fade_out_all(stage, &[carry], 0.3)?;
        let in_frame = framed(stage, &inp, input())?;
        let out_frame = framed(stage, &out, output())?;
        let in_scale = scale_marks(&kit, stage, &inp, &[0, 1])?;
        let out_scale = scale_marks(&kit, stage, &out, &[0, 1])?;
        fade_in_all(stage, &in_frame, 0.4)?;
        fade_in_all(stage, &out_frame, 0.4)?;
        fade_in_all(stage, &in_scale, 0.3)?;
        fade_in_all(stage, &out_scale, 0.3)?;
        line.wait_sentence(stage, 3)?;
        let titles = [
            kit.text(stage, TITLES[0], 24.0, input(), v(-3.5, 2.75))?,
            kit.text(stage, TITLES[1], 24.0, output(), v(3.5, 2.75))?,
        ];
        fade_in_all(stage, &titles, 0.4)?;
        wait_until(stage, line.end())?;

        // 03_2: squaring: once around |z| = ½ is twice around |w| = ¼
        kit.hold(stage)?;
        let line = say(&kit, stage, "03_2")?;
        fade_out_all(stage, &[in_dot, out_dot, z_name, w_name, graph], 0.3)?;
        let fn_tex = kit.tex(stage, FN_TEX[0], 32.0, Palette::ink(), &[], fn_at)?;
        fade_in_all(stage, &[fn_tex], 0.4)?;
        line.wait_sentence(stage, 1)?;
        let rule = kit.tex(stage, SQUARE_RULE_TEX, 23.0, hero(), &[], caption_at)?;
        fade_in_all(stage, &[rule], 0.4)?;
        // the planes are not left empty under the rule: one point shows it, length on these
        // words and angle on the next, and all of it makes room as the walk starts
        line.wait_for(stage, "squares each length")?;
        let (rz, rw) = (rule_point(), rule_point() * rule_point());
        let rays = [
            Arrow::new(inp.p(ZERO), inp.p(rz))
                .buff(0.0)
                .thickness(4.0)
                .color(input())
                .build()?,
            Arrow::new(out.p(ZERO), out.p(rw))
                .buff(0.0)
                .thickness(4.0)
                .color(output())
                .build()?,
        ];
        let mut shown_rule = Vec::with_capacity(8);
        for ray in rays {
            shown_rule.push(stage.arena_mut().add(ray));
        }
        for (k, p) in [inp, out].into_iter().enumerate() {
            let color = if k == 0 { input() } else { output() };
            shown_rule.push(kit.tex(
                stage,
                RULE_LENGTH_TEX[k],
                RULE_LABEL_SIZE,
                color,
                &[],
                p.p(RULE_LENGTH_AT[k]),
            )?);
        }
        fade_in_all(stage, &shown_rule, 0.4)?;
        line.wait_for(stage, "doubles each angle")?;
        let mut angles = Vec::with_capacity(4);
        for (k, (p, to)) in [(inp, rz.arg()), (out, rw.arg())].into_iter().enumerate() {
            angles.push(p.path(stage, &rule_arc(to), hero(), 4.0)?);
            angles.push(kit.tex(
                stage,
                RULE_ANGLE_TEX[k],
                RULE_LABEL_SIZE,
                hero(),
                &[],
                p.p(RULE_ANGLE_AT[k]),
            )?);
        }
        fade_in_all(stage, &angles, 0.4)?;
        shown_rule.extend(angles);
        line.wait_sentence(stage, 2)?;
        fade_out_all(stage, &shown_rule, 0.3)?;
        let ring_in = inp.circle(stage, ZERO, WALK_R, hero(), 4.0)?;
        let ring_out = out.circle(stage, ZERO, WALK_R * WALK_R, Palette::faint(), 2.0)?;
        let walked = stage.add_value_tracker(0.0);
        let w_in = dot(stage, inp.p(C::real(WALK_R)), 0.08, input());
        let w_out = dot(stage, out.p(C::real(WALK_R * WALK_R)), 0.06, output());
        let trail = curve(stage, &out, &[C::real(WALK_R * WALK_R); 2], output(), 4.0);
        let second_lap = curve(stage, &out, &[C::real(WALK_R * WALK_R); 2], Palette::ink(), 4.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        stage.add_updater(
            anchor,
            move |a, _| {
                let t = value(a, walked).clamp(0.0, 1.0);
                let z = C::polar(WALK_R, TAU * t);
                a.move_to(w_in, inp.p(z), ORIGIN);
                a.move_to(w_out, out.p(z * z), ORIGIN);
                let (first, second) = image_laps(t);
                for (m, lap) in [(trail, first), (second_lap, second)] {
                    let pts: Vec<Vec3> = lap.into_iter().map(|w| out.p(w)).collect();
                    let _ = a.set_points(m, polyline(pts).build().points());
                }
            },
            true,
        )?;
        let labels = [
            kit.text(stage, TURNS_LABEL, 22.0, input(), v(TURNS_AT[0].0, TURNS_Y))?,
            kit.text(stage, TURNS_LABEL, 22.0, output(), v(TURNS_AT[1].0, TURNS_Y))?,
        ];
        let radii_said = [
            kit.text(
                stage,
                &format!("{RADIUS}{}", fact("square.circle.in_radius")?),
                RADIUS_SIZE,
                input(),
                inp.p(RADIUS_AT[0]),
            )?,
            kit.text(
                stage,
                &format!("{RADIUS}{}", fact("square.circle.out_radius")?),
                RADIUS_SIZE,
                output(),
                out.p(RADIUS_AT[1]),
            )?,
        ];
        let counts = [
            kit.live_value(
                stage,
                move |s| Walker::turns_so_far(Func::Identity, ZERO, WALK_R, value(s, walked)),
                2,
                false,
                28.0,
                input(),
                v(TURNS_AT[0].1, TURNS_Y),
                ORIGIN,
            )?,
            kit.live_value(
                stage,
                move |s| Walker::turns_so_far(Func::Square, ZERO, WALK_R, value(s, walked)),
                2,
                false,
                28.0,
                output(),
                v(TURNS_AT[1].1, TURNS_Y),
                ORIGIN,
            )?,
        ];
        fade_in_all(stage, &[ring_in, ring_out, w_in, w_out, trail], 0.4)?;
        fade_in_all(stage, &[labels[0], labels[1], radii_said[0], radii_said[1]], 0.3)?;
        let rt = (line.end() - stage.scene().time().to_f64()).max(3.0);
        // two halves of one steady walk: the second lap's line comes only when it has a lap
        stage.play(tween_linear(walked, 0.5, rt / 2.0)?)?;
        stage.add_to_scene(second_lap)?;
        stage.add_to_scene(w_out)?;
        stage.play(tween_linear(walked, 1.0, rt / 2.0)?)?;
        stage.clear_updaters(anchor, true);
        for c in counts {
            stage.clear_updaters(c, true);
            stage.remove_from_scene(c);
        }
        // written as the counters read at the end, so the digits do not jump
        let finals = [
            kit.text(
                stage,
                &format!("{}.00", fact("identity.circle.turns")?),
                28.0,
                input(),
                v(TURNS_AT[0].1, TURNS_Y),
            )?,
            kit.text(
                stage,
                &format!("{}.00", fact("square.circle.turns")?),
                28.0,
                output(),
                v(TURNS_AT[1].1, TURNS_Y),
            )?,
        ];
        for f in finals {
            stage.add_to_scene(f)?;
        }

        // 03_3: a grid: will the vertical lines stay straight after z²?
        kit.hold(stage)?;
        let line = say(&kit, stage, "03_3")?;
        let mut gone = vec![rule, ring_in, ring_out, w_in, w_out, trail, second_lap];
        gone.extend(radii_said);
        gone.extend(labels);
        gone.extend(finals);
        fade_out_all(stage, &gone, 0.4)?;
        // same frames, same window
        let (inp, out) = planes(SQUARE_HALVES.0, SQUARE_HALVES.1);
        let (vertical, horizontal) = grid(60);
        let mut in_lines = Vec::new();
        let mut copies: Vec<(Mob, Vec<C>)> = Vec::new();
        for (zs, color) in vertical
            .iter()
            .map(|zs| (zs, input()))
            .chain(horizontal.iter().map(|zs| (zs, hero())))
        {
            in_lines.push(curve(stage, &inp, zs, color, 3.0));
            copies.push((curve(stage, &out, zs, color, 3.0), zs.clone()));
        }
        fade_in_all(stage, &in_lines, 0.5)?;
        let asked = predict(&kit, stage, &line, &ASK, out.p(ASK_AT))?;

        // 03_4: lines bend into parabolas; the axes fold; right angles are kept but at 0
        let line = say(&kit, stage, "03_4")?;
        fade_out_all(stage, &asked, 0.3)?;
        // the copies start on top of the grid's lines, in the output plane
        let (axes, off_axis): (Vec<_>, Vec<_>) = copies.iter().cloned().enumerate().partition(|(k, _)| k % 5 == 2);
        let axes: Vec<(Mob, Vec<C>)> = axes.into_iter().map(|(_, c)| c).collect();
        let off_axis: Vec<(Mob, Vec<C>)> = off_axis.into_iter().map(|(_, c)| c).collect();
        let off_mobs: Vec<Mob> = off_axis.iter().map(|(m, _)| *m).collect();
        fade_in_all(stage, &off_mobs, 0.3)?;
        line.wait_sentence(stage, 1)?;
        let rt = line.until(stage, "The two axes fold", 2.0)?;
        morph(stage, out, &off_axis, |z| z * z, rt)?;
        let axis_mobs: Vec<Mob> = axes.iter().map(|(m, _)| *m).collect();
        fade_in_all(stage, &axis_mobs, 0.3)?;
        let rt = line.until(stage, "The horizontal axis lands", 1.5)?;
        morph(stage, out, &axes, |z| z * z, rt)?;
        // the fold: both axes double back at the origin's image, which is named. A dot, since
        // both folded axes lie along the real line and a stroke across it would be a third
        let fold = dot(stage, out.p(ZERO), 0.06, Palette::ink());
        let fold_name = kit.text(stage, "0", 18.0, Palette::ink(), out.p(FOLD_NAME_AT))?;
        fade_in_all(stage, &[fold, fold_name], 0.2)?;
        line.wait_sentence(stage, 3)?;
        let rays = [
            kit.text(stage, RAYS[0], 20.0, hero(), v(-2.6, CAPTION_Y))?,
            kit.text(stage, RAYS[1], 20.0, input(), v(2.6, CAPTION_Y))?,
        ];
        fade_in_all(stage, &rays, 0.4)?;
        line.wait_sentence(stage, 4)?;
        let size_in = 0.12;
        let size_out = size_in * inp.unit() / out.unit();
        let marks_in: Vec<Mob> = CROSSINGS
            .iter()
            .map(|&z0| inp.path(stage, &right_mark(z0, ONE, size_in), Palette::ink(), 3.0))
            .collect::<fmn::Result<_>>()?;
        fade_in_all(stage, &marks_in, 0.4)?;
        line.wait_sentence(stage, 5)?;
        let marks_out: Vec<Mob> = CROSSINGS
            .iter()
            .map(|&z0| {
                out.path(
                    stage,
                    &right_mark(z0 * z0, z0.scale(2.0), size_out),
                    Palette::ink(),
                    3.0,
                )
            })
            .collect::<fmn::Result<_>>()?;
        let at_zero_in = inp.path(stage, &right_mark(ZERO, ONE, size_in), warn(), 4.0)?;
        let straight: Vec<C> = (0..=24)
            .map(|k| C::polar(size_out * 1.6, PI * f64::from(k) / 24.0))
            .collect();
        let at_zero_out = out.path(stage, &straight, warn(), 4.0)?;
        fade_in_all(stage, &marks_out, 0.4)?;
        line.wait_for(stage, "except at zero")?;
        fade_in_all(stage, &[at_zero_in, at_zero_out], 0.4)?;
        line.wait_sentence(stage, 7)?;
        fade_out_all(stage, &rays, 0.3)?;
        let kept = kit.text(stage, KEPT, 24.0, hero(), caption_at)?;
        fade_in_all(stage, &[kept], 0.4)?;
        wait_until(stage, line.end())?;

        // 03_5: 1/z: near and far trade places; the unit circle flips; 0 has no image
        kit.hold(stage)?;
        let line = say(&kit, stage, "03_5")?;
        let mut gone = vec![fn_tex, fold, fold_name, kept, at_zero_in, at_zero_out];
        gone.extend(in_lines);
        gone.extend(copies.iter().map(|(m, _)| *m));
        gone.extend(marks_in);
        gone.extend(marks_out);
        fade_out_all(stage, &gone, 0.4)?;
        // same frames, same window
        let (inp, out) = planes(INVERSE_HALVES.0, INVERSE_HALVES.1);
        let fn_tex = kit.tex(stage, FN_TEX[1], 32.0, Palette::ink(), &[], fn_at)?;
        fade_in_all(stage, &[fn_tex], 0.4)?;
        line.wait_sentence(stage, 1)?;
        let out_and_back = stage.add_value_tracker(0.0);
        let near = dot(stage, inp.p(C::polar(ray_distance(0.0), RAY_ANGLE)), 0.09, input());
        let far = dot(
            stage,
            out.p(C::polar(ray_distance(0.0), RAY_ANGLE).recip()),
            0.09,
            output(),
        );
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        stage.add_updater(
            anchor,
            move |a, _| {
                let z = C::polar(ray_distance(value(a, out_and_back)), RAY_ANGLE);
                a.move_to(near, inp.p(z), ORIGIN);
                a.move_to(far, out.p(z.recip()), ORIGIN);
            },
            true,
        )?;
        fade_in_all(stage, &[near, far], 0.3)?;
        let rt = line.until(stage, "The unit circle", 2.5)?;
        stage.play(tween(out_and_back, 1.0, rt)?)?;
        stage.clear_updaters(anchor, true);
        let circles = [
            inp.circle(stage, ZERO, 1.0, hero(), 3.0)?,
            out.circle(stage, ZERO, 1.0, hero(), 3.0)?,
        ];
        fade_in_all(stage, &circles, 0.4)?;
        // pairs told apart by colour, in none of the input's or output's: the walking dot
        // and its image are blue and green, and are not one of these pairs
        let colors = [hero(), Palette::purple(), pole(), Palette::ink()];
        let mut ticks = Vec::new();
        for (&th, &c) in TICKS.iter().zip(&colors) {
            let a = dot(stage, inp.p(C::polar(1.0, th)), 0.1, c);
            fade_in_all(stage, &[a], 0.2)?;
            let b = dot(stage, out.p(C::polar(1.0, th).recip()), 0.1, c);
            fade_in_all(stage, &[b], 0.3)?;
            ticks.extend([a, b]);
        }
        // the flip is performed: where each point would have stood on the right, a ring, and an
        // arrow straight across the horizontal axis to where it lands
        line.wait_for(stage, "each point flips")?;
        let mut flips = Vec::with_capacity(2 * TICKS.len());
        for (&th, &c) in TICKS.iter().zip(&colors) {
            let (from, to) = flip_chord(th);
            let ghost = out.circle(stage, from, FLIP_RING / out.unit(), c, 2.0)?;
            let chord = Arrow::new(out.p(from), out.p(to))
                .buff(FLIP_BUFF)
                .thickness(3.0)
                .color(c)
                .build()?;
            let chord = stage.arena_mut().add(chord);
            fade_in_all(stage, &[ghost, chord], 0.2)?;
            flips.extend([ghost, chord]);
        }
        line.wait_sentence(stage, 3)?;
        let trade = kit.text(stage, TRADE, 22.0, Palette::ink(), caption_at)?;
        fade_in_all(stage, &[trade], 0.4)?;
        line.wait_sentence(stage, 4)?;
        let cross = red_cross(stage, inp.p(ZERO));
        // the card stands outside the unit circle and its ticks; a leader joins it to the cross
        let undefined = carded_lines(&kit, stage, &[UNDEFINED], 20.0, warn(), inp.p(UNDEFINED_AT), warn())?;
        let leader = stage.arena_mut().add(
            polyline(vec![inp.p(UNDEFINED_LEADER[0]), inp.p(UNDEFINED_LEADER[1])])
                .style(line_style(warn(), 2.0))
                .build(),
        );
        fade_in_all(stage, &cross, 0.3)?;
        fade_in_all(stage, &[leader], 0.2)?;
        fade_in_all(stage, &undefined, 0.3)?;
        wait_until(stage, line.end())?;

        // 03_6: e^z: right by one is ×e, up turns; vertical lines → circles, horizontal → rays
        kit.hold(stage)?;
        let line = say(&kit, stage, "03_6")?;
        let mut gone = vec![fn_tex, near, far, trade, leader];
        gone.extend(cross);
        gone.extend(circles);
        gone.extend(ticks);
        gone.extend(flips);
        gone.extend(undefined);
        gone.extend(in_frame);
        gone.extend(in_scale);
        gone.extend(out_scale);
        fade_out_all(stage, &gone, 0.4)?;
        let inp = exp_input();
        let (_, out) = planes(INVERSE_HALVES.0, EXP_OUT_HALF);
        let in_frame = framed(stage, &inp, input())?;
        let mut in_scale = scale_marks(&kit, stage, &inp, &[0, 1])?;
        let top = inp.p(C::new(0.0, TAU));
        let climb_tick = stage.arena_mut().add(
            polyline(vec![v(top[0] - 2.0 * TICK, top[1]), v(top[0] + 2.0 * TICK, top[1])])
                .style(line_style(hero(), 3.0))
                .build(),
        );
        in_scale.push(climb_tick);
        in_scale.push(kit.tex(stage, CLIMB_TEX, 18.0, hero(), &[], inp.p(CLIMB_NAME_AT))?);
        // no "1" here: 1.000 is written beside that tick in a moment
        let out_scale = scale_marks(&kit, stage, &out, &[1])?;
        let rescaled = kit.text(stage, RESCALED, 18.0, Palette::dim(), caption_at)?;
        let fn_tex = kit.tex(stage, GALLERY_TITLES[2], 32.0, Palette::ink(), &[], fn_at)?;
        fade_in_all(stage, &in_frame, 0.4)?;
        fade_in_all(stage, &in_scale, 0.3)?;
        fade_in_all(stage, &out_scale, 0.3)?;
        fade_in_all(stage, &[fn_tex, rescaled], 0.4)?;
        line.wait_sentence(stage, 1)?;
        let radii = fact_list("exp.circle_radii")?;
        let printed: Vec<&str> = fact("exp.circle_radii")?.split(',').collect();
        let mut steps = Vec::new();
        for (k, x) in [-1.0, 0.0, 1.0].into_iter().enumerate() {
            let a = dot(stage, inp.p(C::real(x)), 0.08, input());
            let b = dot(stage, out.p(C::real(radii[k])), 0.08, output());
            let l = kit.text(stage, printed[k], 18.0, output(), out.p(radius_label_at(k, radii[k])))?;
            fade_in_all(stage, &[a], 0.2)?;
            fade_in_all(stage, &[b, l], 0.3)?;
            steps.extend([a, b, l]);
        }
        let times_e = kit.tex(stage, TIMES_E_TEX, 26.0, hero(), &[], out.p(C::new(1.85, 0.4)))?;
        fade_in_all(stage, &[times_e], 0.3)?;
        steps.push(times_e);
        line.wait_sentence(stage, 2)?;
        // a note on the windows, read while the frames arrive; it does not stay for the climb
        fade_out_all(stage, &[rescaled], 0.3)?;
        let (vertical, horizontal) = exp_lines(120);
        let image = |zs: &[C]| zs.iter().map(|&z| z.exp()).collect::<Vec<C>>();
        let mut drawn = Vec::new();
        let mut items = Vec::new();
        for (k, zs) in vertical.iter().enumerate() {
            let a = curve(stage, &inp, &zs[..2], input(), 3.0);
            let b = curve(stage, &out, &image(&zs[..2]), output(), 3.0);
            stage.add_to_scene(a)?;
            stage.add_to_scene(b)?;
            items.push((k, (a, inp, zs.clone()), (b, out, image(zs.as_slice()))));
            drawn.extend([a, b]);
        }
        // first the line x = 1 alone: up 2π is once around the circle of radius e
        let first: Vec<(Mob, Plane, Vec<C>)> = items
            .iter()
            .filter(|(k, _, _)| *k == 2)
            .flat_map(|(_, a, b)| [a.clone(), b.clone()])
            .collect();
        let rt = line.until(stage, "So vertical lines", 2.0)?;
        reveal(stage, &first, rt)?;
        line.wait_sentence(stage, 3)?;
        let rest: Vec<(Mob, Plane, Vec<C>)> = items
            .iter()
            .filter(|(k, _, _)| *k != 2)
            .flat_map(|(_, a, b)| [a.clone(), b.clone()])
            .collect();
        reveal(stage, &rest, 1.2)?;
        let mut rays = Vec::new();
        for zs in &horizontal {
            let a = curve(stage, &inp, &zs[..2], hero(), 3.0);
            let b = curve(stage, &out, &image(&zs[..2]), hero(), 3.0);
            stage.add_to_scene(a)?;
            stage.add_to_scene(b)?;
            rays.push((a, inp, zs.clone()));
            rays.push((b, out, image(zs.as_slice())));
            drawn.extend([a, b]);
        }
        let rt = (line.end() - stage.scene().time().to_f64()).max(1.2);
        reveal(stage, &rays, rt)?;
        wait_until(stage, line.end())?;

        // 03_7: a climb of 2π lands on the same output; e^z is never 0
        kit.hold(stage)?;
        let line = say(&kit, stage, "03_7")?;
        fade_out_all(stage, &steps, 0.3)?;
        let climb = stage.add_value_tracker(0.0);
        let climber = dot(stage, inp.p(ZERO), 0.1, hero());
        let image_dot = dot(stage, out.p(ONE), 0.1, hero());
        let path = curve(stage, &inp, &[ZERO; 2], hero(), 6.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        stage.add_updater(
            anchor,
            move |a, _| {
                let y = value(a, climb) * TAU;
                a.move_to(climber, inp.p(C::new(0.0, y)), ORIGIN);
                a.move_to(image_dot, out.p(C::new(0.0, y).exp()), ORIGIN);
                let _ = a.set_points(
                    path,
                    polyline(vec![inp.p(ZERO), inp.p(C::new(0.0, y.max(1e-3)))])
                        .build()
                        .points(),
                );
            },
            true,
        )?;
        fade_in_all(stage, &[climber, image_dot, path], 0.3)?;
        let rt = line.until(stage, "The exponential repeats", 2.5)?;
        stage.play(tween(climb, 1.0, rt)?)?;
        stage.clear_updaters(anchor, true);
        let after: Vec<&str> = fact("exp.after_2pi")?.split(',').collect();
        let landed = explainer_kit::props::equation_card(
            &kit,
            stage,
            &format!(r"{AFTER_TEX}{} + {}i", after[0], after[1]),
            22.0,
            &[],
            out.p(LANDED_AT),
        )?;
        let period = kit.tex(stage, PERIOD_TEX, 28.0, hero(), &[], v(-3.5, caption_at[1]))?;
        fade_in_all(stage, &[landed, period], 0.4)?;
        line.wait_for(stage, "and it never reaches zero")?;
        let nonzero = kit.tex(stage, NONZERO_TEX, 28.0, warn(), &[], v(3.5, caption_at[1]))?;
        let never = out.circle(stage, ZERO, 0.12, warn(), 4.0)?;
        fade_in_all(stage, &[nonzero, never], 0.4)?;
        wait_until(stage, line.end())?;
        kit.music_breath(stage, 2.0)?;
        let mut all = vec![fn_tex, climber, image_dot, path, landed, period, nonzero, never];
        all.extend(drawn);
        all.extend(in_frame);
        all.extend(in_scale);
        all.extend(out_frame);
        all.extend(out_scale);
        all.extend(titles);
        fade_out_all(stage, &all, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get(key: &str) -> &'static str {
        fact(key).unwrap()
    }

    #[test]
    fn the_walk_counts_match_the_facts() {
        let once = Walker::turns_so_far(Func::Identity, ZERO, WALK_R, 1.0);
        let twice = Walker::turns_so_far(Func::Square, ZERO, WALK_R, 1.0);
        assert!((once - get("identity.circle.turns").parse::<f64>().unwrap()).abs() < 1e-9);
        assert!((twice - get("square.circle.turns").parse::<f64>().unwrap()).abs() < 1e-9);
        assert!((WALK_R * WALK_R - get("square.circle.out_radius").parse::<f64>().unwrap()).abs() < 1e-12);
        assert!((WALK_R - get("square.circle.in_radius").parse::<f64>().unwrap()).abs() < 1e-12);
        // the counters end at the values the final cards write with ".00"
        assert_eq!((get("identity.circle.turns"), get("square.circle.turns")), ("1", "2"));
    }

    /// Every plane carries its scale, and from the first point through 1/z the scale is one.
    #[test]
    fn every_plane_is_ticked_and_the_scale_changes_once() {
        assert_eq!(whole_ticks((-WINDOW, WINDOW)), vec![-2.0, -1.0, 1.0, 2.0]);
        assert_eq!(whole_ticks(exp_input().frame.y), vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        assert_eq!(
            whole_ticks((-EXP_OUT_HALF, EXP_OUT_HALF)),
            vec![-3.0, -2.0, -1.0, 1.0, 2.0, 3.0]
        );
        assert_eq!(WALK_HALVES, SQUARE_HALVES);
        assert_eq!(SQUARE_HALVES, INVERSE_HALVES);
        // the window holds what it has to: 1/z's farthest image
        assert!(1.0 / ray_distance(0.0) < WINDOW);
        // e^z's frames have exactly the other frames' height
        let (square, _) = planes(SQUARE_HALVES.0, SQUARE_HALVES.1);
        assert!((exp_input().frame.height - square.frame.height).abs() < 1e-9);
    }

    /// True if nothing the grid maps to under z² enters a box around `at` of half size
    /// `(hx, hy)`, in plane units.
    fn clear_of_images(at: C, hx: f64, hy: f64) -> bool {
        let (vertical, horizontal) = grid(240);
        vertical.iter().chain(&horizontal).flatten().all(|&z| {
            let w = z * z;
            (w.re - at.re).abs() > hx || (w.im - at.im).abs() > hy
        })
    }

    /// The names of 1 and i stand off the axes, off the unit circle and off the grid's images.
    /// The half sizes are CMR10 widths at 18, an estimate: the pictures decide.
    #[test]
    fn the_unit_names_stand_clear() {
        let (hx, hy) = (0.05, 0.06);
        for z in UNIT_AT {
            assert!(z.re.abs() > hx && z.im.abs() > hy, "{z:?}");
            assert!(z.abs() - (hx * hx + hy * hy).sqrt() > 1.05, "{z:?}");
            assert!(clear_of_images(z, hx, hy), "{z:?}");
            assert!(exp_input().contains(z) && planes(WINDOW, WINDOW).1.contains(z));
        }
        // and the name at 0 fits the gap the images leave around the origin
        assert!(clear_of_images(FOLD_NAME_AT, 0.045, 0.055));
        const { assert!(FOLD_NAME_AT.re + 0.045 < 0.0 && FOLD_NAME_AT.im + 0.055 < -0.12) };
    }

    /// "turns:" and its value keep to one side of each plane's axis.
    #[test]
    fn the_turn_counts_keep_to_one_side_of_the_axis() {
        let (inp, out) = planes(WALK_HALVES.0, WALK_HALVES.1);
        for ((word, value), p) in TURNS_AT.into_iter().zip([inp, out]) {
            let axis = p.p(ZERO)[0];
            assert!(word + 0.3 < value - 0.2, "the word and its value overlap");
            assert!(value + 0.2 < axis - 0.2, "the value reaches the axis");
            assert!(word - 0.3 > p.frame.left());
        }
        // the row stands inside the frame, under the radii
        assert!(TURNS_Y > inp.frame.bottom() && TURNS_Y < inp.p(C::new(0.0, RADIUS_AT[0].im - 0.2))[1]);
    }

    /// Radii ½ and ¼ are both written, off both axes, off the circles they name and off the
    /// plane's "1". The half sizes are CMR10 widths at `RADIUS_SIZE` in this window, an
    /// estimate: the pictures decide.
    #[test]
    fn the_walk_radii_stand_off_their_circles_and_the_axes() {
        let half_h = 0.12;
        const { assert!(RADIUS_SIZE >= 24.0) };
        for (at, half, r) in [(RADIUS_AT[0], 0.438, WALK_R), (RADIUS_AT[1], 0.49, WALK_R * WALK_R)] {
            assert!(at.re - half > 0.1, "{at:?}: reaches the axis");
            assert!(at.im + half_h < -0.1, "{at:?}: reaches the axis");
            let near = C::new(at.re - half, at.im + half_h).abs();
            assert!(near > r + 0.1, "{at:?}: reaches its circle");
            assert!(at.im + half_h < UNIT_AT[0].im - 0.06 - 0.05, "{at:?}: reaches the 1");
        }
    }

    /// The source of one cue's picture, up to `end`: read from the source because no render can
    /// run here. What it does to the pixels is for the pictures to show.
    fn beat(id: &str, end: &str) -> &'static str {
        let source = include_str!("two_planes.rs");
        let code = &source[..source.find("#[cfg(test)]\nmod tests").unwrap()];
        let start = code.find(&format!("say(&kit, stage, \"{id}\")")).unwrap();
        let stop = start + code[start..].find(end).unwrap();
        &code[start..stop]
    }

    /// The same, up to the next cue.
    fn cue(id: &str, next: &str) -> &'static str {
        beat(id, &format!("say(&kit, stage, \"{next}\")"))
    }

    /// The rule's point is what squaring does, and every piece of its picture fits the windows
    /// and stands clear of the pieces around it. The half sizes are estimates: the pictures decide.
    #[test]
    fn the_rule_is_shown_on_one_point_that_squaring_really_sends_there() {
        let (inp, out) = planes(WALK_HALVES.0, WALK_HALVES.1);
        let (z, w) = (rule_point(), rule_point() * rule_point());
        assert!((w.abs() - z.abs() * z.abs()).abs() < 1e-12, "length squared");
        assert!((w.arg() - 2.0 * z.arg()).abs() < 1e-12, "angle doubled");
        assert!(inp.contains(z) && out.contains(w));
        // it is not the walk's circle, so nothing has to be unlearned as the walk starts
        const { assert!(RULE_LENGTH > 2.0 * WALK_R) };
        // a length that visibly grows, and an angle that visibly doubles
        assert!(w.abs() - z.abs() > 0.5 && w.arg() - z.arg() > 0.5);
        for (p, (length, angle)) in [inp, out]
            .into_iter()
            .zip(RULE_LENGTH_AT.into_iter().zip(RULE_ANGLE_AT))
        {
            assert!(p.contains(length) && p.contains(angle));
        }
        // each angle's name sits between the real axis and its arrow, past its arc
        for (at, half, arrow) in [(RULE_ANGLE_AT[0], 0.2, z), (RULE_ANGLE_AT[1], 0.26, w)] {
            let corner = C::new(at.re - half, at.im - 0.1);
            assert!(corner.abs() > RULE_ARC_R + 0.05 && corner.im > 0.1, "{at:?}");
            let top_left = C::new(at.re - half, at.im + 0.1);
            assert!(top_left.arg() < arrow.arg() - 0.05, "{at:?}: reaches its arrow");
        }
        // each length's name stands past its arrow's tip, off the plane's names
        for (at, tip) in [(RULE_LENGTH_AT[0], z), (RULE_LENGTH_AT[1], w)] {
            assert!((at - tip).abs() > 0.2 && at.abs() > tip.abs(), "{at:?}");
            for name in UNIT_AT {
                assert!((at - name).abs() > 0.4, "{at:?}");
            }
        }
        // the arcs run from the real axis to the arrows they measure
        for to in [z.arg(), w.arg()] {
            let arc = rule_arc(to);
            assert!(arc[0].im.abs() < 1e-12 && (arc[arc.len() - 1].arg() - to).abs() < 1e-12);
        }
        // the labels are the rule's own pieces, so its glyphs are already on screen
        assert_eq!(RULE_LENGTH_TEX, [r"|z|", r"|z|^2"]);
        assert!(SQUARE_RULE_TEX.contains(r"\arg z") && SQUARE_RULE_TEX.contains(r"2 \arg z"));
        assert!(SQUARE_RULE_TEX.contains(r"|z|^2"));
    }

    /// "Squares each length and doubles each angle" is not left to the formula row: the point
    /// comes on those words, and it is off the planes before the walk's ring comes on.
    #[test]
    fn the_planes_are_not_empty_under_the_rule() {
        let two = cue("03_2", "03_3");
        let order = [
            "fade_in_all(stage, &[rule], 0.4)?;",
            "line.wait_for(stage, \"squares each length\")?;",
            "fade_in_all(stage, &shown_rule, 0.4)?;",
            "line.wait_for(stage, \"doubles each angle\")?;",
            "fade_in_all(stage, &angles, 0.4)?;",
            "line.wait_sentence(stage, 2)?;",
            "fade_out_all(stage, &shown_rule, 0.3)?;",
            "let ring_in = ",
        ];
        let at: Vec<usize> = order
            .iter()
            .map(|s| two.find(s).unwrap_or_else(|| panic!("{s}")))
            .collect();
        assert!(at.windows(2).all(|w| w[0] < w[1]), "{at:?}");
    }

    /// The image goes around twice and the picture can show it: each half of the walk is a
    /// whole lap of the output's circle, and the second lap starts only as the first closes.
    #[test]
    fn each_half_of_the_walk_is_one_lap_of_the_image() {
        let r = WALK_R * WALK_R;
        for t in [0.0, 0.1, 0.25, 0.5, 0.75, 1.0] {
            let (first, second) = image_laps(t);
            assert!(first.len() >= 2 && second.len() >= 2);
            for w in first.iter().chain(&second) {
                assert!((w.abs() - r).abs() < 1e-12);
            }
        }
        let (first, second) = image_laps(0.5);
        assert!(
            (first[0] - first[first.len() - 1]).abs() < 1e-5,
            "the first half closes a lap"
        );
        assert!(
            (second[0] - second[second.len() - 1]).abs() < 1e-3,
            "the second has not started"
        );
        let (first, second) = image_laps(1.0);
        assert!((first[first.len() - 1] - second[0]).abs() < 1e-12);
        assert!(
            (second[0] - second[second.len() - 1]).abs() < 1e-5,
            "the second half closes a lap"
        );
        // walked in two halves that end where the single walk ended, the second lap's line and
        // the dot over it coming between them, and it fades with the rest
        let two = cue("03_2", "03_3");
        let order = [
            "tween_linear(walked, 0.5, rt / 2.0)",
            "stage.add_to_scene(second_lap)?;",
            "stage.add_to_scene(w_out)?;",
            "tween_linear(walked, 1.0, rt / 2.0)",
        ];
        let at: Vec<usize> = order
            .iter()
            .map(|s| two.find(s).unwrap_or_else(|| panic!("{s}")))
            .collect();
        assert!(at.windows(2).all(|w| w[0] < w[1]), "{at:?}");
        assert!(cue("03_3", "03_4").contains("trail, second_lap]"));
    }

    /// The flip the voice names is drawn: each arrow is vertical, crosses the horizontal axis
    /// at its middle, and is long enough to see past its two ends.
    #[test]
    fn each_flip_goes_straight_across_the_horizontal_axis() {
        let (_, out) = planes(INVERSE_HALVES.0, INVERSE_HALVES.1);
        for th in TICKS {
            let (from, to) = flip_chord(th);
            assert!((from.re - to.re).abs() < 1e-12 && (from.im + to.im).abs() < 1e-12);
            assert!(out.contains(from) && out.contains(to));
            let length = (from - to).abs() * out.unit();
            assert!(length > 2.0 * FLIP_BUFF + 0.5, "{th}: {length}");
        }
        let five = cue("03_5", "03_6");
        let ticks = five.find("ticks.extend([a, b]);").unwrap();
        let hook = five.find("line.wait_for(stage, \"each point flips\")?;").unwrap();
        let drawn = five.find("flips.extend([ghost, chord]);").unwrap();
        let next = five.find("line.wait_sentence(stage, 3)?;").unwrap();
        assert!(ticks < hook && hook < drawn && drawn < next);
        assert!(cue("03_6", "03_7").contains("gone.extend(flips);"));
    }

    /// The question's countdown ring keeps off the output's two axes, and its card hides no mark
    /// of the scale (it does lie over a stretch of the imaginary axis, as a card may). The card's
    /// height is `carded_lines`' for one line at 28, and `predict`'s ring hangs under it.
    #[test]
    fn the_question_and_its_ring_stand_clear_of_the_axes() {
        let (_, out) = planes(SQUARE_HALVES.0, SQUARE_HALVES.1);
        let (at, zero) = (out.p(ASK_AT), out.p(ZERO));
        let card_h = 28.0 / 48.0 * 0.62 + 0.3;
        let ring_y = at[1] - card_h / 2.0 - 0.45;
        let ring_r = 0.26;
        assert!(at[0] - ring_r > zero[0] + 0.1, "the ring reaches the imaginary axis");
        assert!(ring_y - ring_r > out.frame.bottom(), "the ring leaves the plane");
        assert!(
            at[1] + card_h / 2.0 < out.p(UNIT_AT[0])[1] - 0.09,
            "the card covers the 1"
        );
        assert!(
            at[1] - card_h / 2.0 > out.p(C::new(0.0, -1.0))[1],
            "the card covers a tick"
        );
        // the card keeps inside its plane: about 8.2 ems at 28 (CMR10), plus its margins
        let half_w = (8.2 * 28.0 * 0.01 + 0.5) / 2.0;
        assert!(at[0] - half_w > out.frame.left() && at[0] + half_w < out.frame.right());
        assert!(cue("03_3", "03_4").contains("out.p(ASK_AT)"));
    }

    /// The foot labels use the voice's names for the axes, and the note on the new windows goes
    /// once it has been read.
    #[test]
    fn the_labels_say_what_the_voice_says() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let said = script.text("03_4").unwrap();
        for (label, word) in RAYS.into_iter().zip(["horizontal axis", "vertical axis"]) {
            assert!(label.starts_with(word) && said.contains(word), "{label}");
        }
        let six = cue("03_6", "03_7");
        let said = six.find("line.wait_sentence(stage, 2)?;").unwrap();
        let goes = six.find("fade_out_all(stage, &[rescaled], 0.3)?;").unwrap();
        assert!(said < goes && !beat("03_7", "kit.music_breath").contains("rescaled"));
        assert!(!six.contains("vec![rescaled]"), "it would fade twice");
    }

    /// No circle of 03_6 runs through a radius readout.
    #[test]
    fn the_radius_readouts_clear_every_circle() {
        let radii: Vec<f64> = get("exp.circle_radii").split(',').map(|x| x.parse().unwrap()).collect();
        let (_, out) = planes(INVERSE_HALVES.0, EXP_OUT_HALF);
        for (k, &r) in radii.iter().enumerate() {
            let at = radius_label_at(k, r);
            let half_h = 0.07;
            assert!(at.im + half_h < -0.1, "{k}: reaches the axis and its dot");
            let lo = C::new(at.re - RADIUS_LABEL_HALF, at.im + half_h).abs();
            let hi = C::new(at.re + RADIUS_LABEL_HALF, at.im - half_h).abs();
            for &other in &radii {
                assert!(
                    other < lo - 0.03 || other > hi + 0.03,
                    "{k}: circle {other} in {lo}..{hi}"
                );
            }
            assert!(out.contains(C::new(at.re + RADIUS_LABEL_HALF, at.im)));
        }
    }

    /// The 1/0 card stands outside the unit circle, and its leader misses every tick.
    #[test]
    fn the_undefined_card_and_its_leader_clear_the_ticks() {
        let (inp, _) = planes(INVERSE_HALVES.0, INVERSE_HALVES.1);
        const {
            assert!(UNDEFINED_AT.im + UNDEFINED_HALF_H < -1.1);
            assert!((UNDEFINED_LEADER[0].im - (UNDEFINED_AT.im + UNDEFINED_HALF_H)).abs() < 1e-9);
            // off the vertical axis the whole way
            assert!(UNDEFINED_LEADER[0].re < -0.05 && UNDEFINED_LEADER[1].re < -0.05);
        };
        assert!(inp.contains(UNDEFINED_AT - C::new(0.0, UNDEFINED_HALF_H)));
        let tick_radius = 0.1 / inp.unit();
        let [a, b] = UNDEFINED_LEADER;
        for th in TICKS {
            let z = C::polar(1.0, th);
            for k in 0..=50 {
                let q = a + (b - a).scale(f64::from(k) / 50.0);
                assert!((q - z).abs() > tick_radius, "tick at {th}");
            }
        }
    }

    /// The same rectangle on screen. `planes` passes the centre and width through untouched,
    /// so those must match exactly. Only the height is computed, as width × span / span: two
    /// rounded operations, each off by at most 2⁻⁵³ of its result, so a height is within
    /// `f64::EPSILON` × width of the width and two heights within twice that. (5.4 × 3 / 3 comes
    /// out one unit in the last place high.) Anything past that bound is a different layout.
    fn same_rectangle(a: Plane, b: Plane) -> bool {
        a.frame.center == b.frame.center
            && a.frame.width == b.frame.width
            && (a.frame.height - b.frame.height).abs() <= 2.0 * f64::EPSILON * a.frame.width
    }

    #[test]
    fn the_grid_and_its_images_fit_the_windows() {
        let (inp, out) = planes(SQUARE_HALVES.0, SQUARE_HALVES.1);
        let (vertical, horizontal) = grid(60);
        assert_eq!((vertical.len(), horizontal.len()), (5, 5));
        for zs in vertical.iter().chain(&horizontal) {
            for &z in zs {
                assert!(inp.contains(z) && out.contains(z) && out.contains(z * z));
                // every step of the morph stays in the window too
                for s in [0.25, 0.5, 0.75] {
                    assert!(out.contains(z.scale(1.0 - s) + (z * z).scale(s)));
                }
            }
        }
        // the axes are the middle lines and fold onto the two real rays
        assert!(
            vertical[2]
                .iter()
                .all(|z| z.re == 0.0 && (*z * *z).im.abs() < 1e-15 && (*z * *z).re <= 0.0)
        );
        assert!(horizontal[2].iter().all(|z| z.im == 0.0 && (*z * *z).re >= 0.0));
        // the frames are the same rectangles whatever the windows
        let (inp2, out2) = planes(INVERSE_HALVES.0, INVERSE_HALVES.1);
        let (inp3, out3) = planes(WALK_HALVES.0, WALK_HALVES.1);
        for (a, b) in [(inp, inp2), (inp, inp3), (out, out2), (out, out3)] {
            assert!(same_rectangle(a, b), "{:?} and {:?}", a.frame, b.frame);
        }
        // and a layout that does differ is told apart, one change at a time from the input
        // plane's own recipe, even at 1e-9 of a scene unit, far under a pixel of the 4K frame
        let (centre, square) = (v(-3.5, -0.35), (-SQUARE_HALVES.0, SQUARE_HALVES.0));
        assert!(same_rectangle(inp, Plane::new(centre, 5.4, square, square)));
        for other in [
            out,
            Plane::new(v(-3.5 + 1e-9, -0.35), 5.4, square, square),
            Plane::new(centre, 5.4 + 1e-9, square, square),
            Plane::new(centre, 5.4, square, (square.0, square.1 + 1e-9)),
            Plane::new(centre, 5.4, square, (square.0 / 2.0, square.1 / 2.0)),
        ] {
            assert!(!same_rectangle(inp, other), "{:?}", other.frame);
        }
        // one scale on both sides in every beat but e^z's
        for (a, b) in [(inp, out), (inp2, out2), (inp3, out3)] {
            assert_eq!(a.unit(), b.unit());
        }
        assert!(out3.contains(C::real(WALK_R)) && WALK_R < WALK_HALVES.0 / 2.0);
    }

    #[test]
    fn the_marks_follow_the_derivative() {
        let f = |z: C| z * z;
        for z0 in CROSSINGS {
            let h = 1e-6;
            let along_x = (f(z0 + C::real(h)) - f(z0)).scale(1.0 / h);
            let along_y = (f(z0 + C::new(0.0, h)) - f(z0)).scale(1.0 / h);
            // the image curves leave z0² along 2z0 and 2z0·i: a right angle
            assert!((along_x - z0.scale(2.0)).abs() < 1e-5);
            assert!((along_y - z0.scale(2.0) * I).abs() < 1e-5);
            let m = right_mark(z0 * z0, z0.scale(2.0), 0.2);
            let (a, b) = (m[0] - z0 * z0, m[2] - z0 * z0);
            assert!((a.re * b.re + a.im * b.im).abs() < 1e-12);
            // each crossing is off the axes, on two grid lines
            assert!(z0.re != 0.0 && z0.im != 0.0);
            assert!((z0.re * 2.0).fract() == 0.0 && (z0.im * 2.0).fract() == 0.0);
        }
    }

    #[test]
    fn the_inverse_walk_and_ticks_fit_and_flip() {
        let (inp, out) = planes(INVERSE_HALVES.0, INVERSE_HALVES.1);
        for k in 0..=20 {
            let z = C::polar(ray_distance(f64::from(k) / 20.0), RAY_ANGLE);
            assert!(inp.contains(z) && out.contains(z.recip()));
        }
        assert!((ray_distance(0.0) - ray_distance(1.0)).abs() < 1e-12);
        for th in TICKS {
            let w = C::polar(1.0, th).recip();
            assert!((w - C::polar(1.0, th).conj()).abs() < 1e-12);
        }
    }

    #[test]
    fn the_exponential_lines_land_on_the_facts() {
        let radii: Vec<f64> = get("exp.circle_radii").split(',').map(|x| x.parse().unwrap()).collect();
        let (vertical, horizontal) = exp_lines(120);
        let inp = exp_input();
        let (_, out) = planes(INVERSE_HALVES.0, EXP_OUT_HALF);
        for (zs, r) in vertical.iter().zip(&radii) {
            for &z in zs {
                assert!(inp.contains(z));
                assert!((z.exp().abs() - r).abs() < 5e-4 && out.contains(z.exp()));
            }
            // once round: the last point's image is the first's
            assert!((zs[zs.len() - 1].exp() - zs[0].exp()).abs() < 1e-12);
        }
        for zs in &horizontal {
            for &z in zs {
                let w = z.exp();
                assert!(inp.contains(z) && out.contains(w));
                // a ray that lay on an axis would read as the axis changing colour
                assert!(w.re.abs() > 0.1 && w.im.abs() > 0.1, "{z:?}");
            }
        }
        assert_eq!(get("exp.after_2pi"), "1.000000,0.000000");
        assert!(inp.contains(C::new(0.0, TAU)));
        // e^z's input frame lines up with the others, under the title and above the captions
        let (square, _) = planes(SQUARE_HALVES.0, SQUARE_HALVES.1);
        assert!((inp.frame.top() - square.frame.top()).abs() < 0.05);
        assert!((inp.frame.bottom() - square.frame.bottom()).abs() < 0.05);
        assert!(inp.frame.top() < 2.55 && inp.frame.bottom() > CAPTION_Y + 0.3);
        // the landing card sits inside the output frame, clear of the image dot at 1
        assert!(out.contains(LANDED_AT) && (LANDED_AT - ONE).abs() > 1.5);
    }
}
