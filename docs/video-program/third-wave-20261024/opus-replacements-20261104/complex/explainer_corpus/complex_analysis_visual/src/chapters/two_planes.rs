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
    predict, say, tween_linear, value, wait_until, warn,
};
// "f(z) = e^z" is the gallery's title for the same map; "radius " the bump's label
use super::gallery::TITLES_TEX as GALLERY_TITLES;
use super::the_bump::RADIUS;
use crate::model::{C, Func, I, ONE, ZERO};

pub const TITLES: [&str; 2] = ["input z", "output f(z)"];
pub const GRAPH: &str = "a graph would need four dimensions";
pub const FN_TEX: [&str; 2] = [r"f(z) = z^2", r"f(z) = \frac{1}{z}"];
pub const SQUARE_RULE_TEX: &str = r"|z^2| = |z|^2, \quad \arg z^2 = 2 \arg z";
pub const TURNS_LABEL: &str = "turns:";
pub const ASK: [&str; 1] = ["straight after z²?"];
pub const RAYS: [&str; 2] = ["real axis → positive ray", "imaginary axis → negative ray"];
pub const KEPT: &str = "right angles kept, except at 0";
pub const TRADE: &str = "inside → outside, outside → inside";
pub const UNDEFINED: &str = "1/0 undefined";
pub const TIMES_E_TEX: &str = r"\times e";
pub const PERIOD_TEX: &str = r"e^{z + 2\pi i} = e^z";
pub const NONZERO_TEX: &str = r"e^z \ne 0";
pub const AFTER_TEX: &str = r"e^{2\pi i} = ";

/// The two planes: input on the left, output on the right; both 5.4 wide at
/// the same height, so their frames stay put when the windows change (no
/// window is ever ticked, and each beat uses one scale on both sides).
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
/// The walk of 03_2 at one scale on both sides, so radius ¼ is drawn half
/// as large as radius ½.
pub const WALK_HALVES: (f64, f64) = (1.5, 1.5);
/// The grid of 03_3 – 03_4, one scale on both sides: the images reach ±2,
/// and each copy starts exactly on top of its input line's shape.
pub const SQUARE_HALVES: (f64, f64) = (2.2, 2.2);
/// 1/z's windows (03_5).
pub const INVERSE_HALVES: (f64, f64) = (2.5, 2.5);
/// e^z's output window (03_6, 03_7), on the same frame.
pub const EXP_OUT_HALF: f64 = 3.2;

/// e^z's input window: tall enough for a climb of 2π, with the same top
/// and bottom as the other frames.
pub fn exp_input() -> Plane {
    Plane::new(v(-3.5, -0.35), 3.6, (-2.4, 2.4), (-0.5, 6.8))
}

/// Where the one-line captions sit, under both frames.
pub const CAPTION_Y: f64 = -3.5;
/// Where e^{2πi}'s computed value is carded in the output plane (03_7).
pub const LANDED_AT: C = C::new(0.0, 1.9);

/// The walk of 03_2: once around |z| = ½.
pub const WALK_R: f64 = 0.5;

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

/// The point that walks out and back along a ray in 03_5: its distance
/// from 0 over the walk's share `t`.
pub fn ray_distance(t: f64) -> f64 {
    let (near, far) = (0.45, 2.2);
    near + (far - near) * (PI * t).sin()
}
pub const RAY_ANGLE: f64 = 0.6;

/// The lines of 03_6: vertical x = −1, 0, 1 for 0 ≤ y ≤ 2π, and
/// horizontal y = 0, π/2, π, 3π/2 for −1.5 ≤ x ≤ 1.15.
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
        .map(|k| f64::from(k) * PI / 2.0)
        .map(|y| line(C::new(-1.5, y), C::new(1.15, y)))
        .collect();
    (vertical, horizontal)
}

/// Where the radius readout of e^x (x = −1, 0, 1 as `k` = 0, 1, 2) sits in
/// the output plane: under the real axis, except the middle one, above.
pub fn radius_label_at(k: usize, r: f64) -> C {
    C::new(r, if k == 1 { 0.4 } else { -0.4 })
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
        fade_in_all(stage, &[in_dot], 0.3)?;
        fade_in_all(stage, &[carry, out_dot], 0.5)?;
        line.wait_sentence(stage, 1)?;
        let graph = kit.text(stage, GRAPH, 22.0, Palette::dim(), caption_at)?;
        fade_in_all(stage, &[graph], 0.4)?;
        line.wait_sentence(stage, 2)?;
        let in_frame = framed(stage, &inp, input())?;
        let out_frame = framed(stage, &out, output())?;
        fade_in_all(stage, &in_frame, 0.4)?;
        fade_in_all(stage, &out_frame, 0.4)?;
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
        fade_out_all(stage, &[in_dot, out_dot, carry, graph], 0.3)?;
        let fn_tex = kit.tex(stage, FN_TEX[0], 32.0, Palette::ink(), &[], fn_at)?;
        fade_in_all(stage, &[fn_tex], 0.4)?;
        line.wait_sentence(stage, 1)?;
        let rule = kit.tex(stage, SQUARE_RULE_TEX, 28.0, hero(), &[], caption_at)?;
        fade_in_all(stage, &[rule], 0.4)?;
        line.wait_sentence(stage, 2)?;
        let ring_in = inp.circle(stage, ZERO, WALK_R, hero(), 4.0)?;
        let ring_out = out.circle(stage, ZERO, WALK_R * WALK_R, Palette::faint(), 2.0)?;
        let walked = stage.add_value_tracker(0.0);
        let w_in = dot(stage, inp.p(C::real(WALK_R)), 0.08, input());
        let w_out = dot(stage, out.p(C::real(WALK_R * WALK_R)), 0.08, output());
        let trail = curve(stage, &out, &[C::real(WALK_R * WALK_R); 2], output(), 4.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        stage.add_updater(
            anchor,
            move |a, _| {
                let t = value(a, walked).clamp(0.0, 1.0);
                let z = C::polar(WALK_R, TAU * t);
                a.move_to(w_in, inp.p(z), ORIGIN);
                a.move_to(w_out, out.p(z * z), ORIGIN);
                let k = ((t * 240.0).ceil() as usize).max(1);
                let pts: Vec<Vec3> = (0..=k)
                    .map(|j| {
                        let z = C::polar(WALK_R, TAU * t * j as f64 / k as f64);
                        out.p(z * z)
                    })
                    .collect();
                let _ = a.set_points(trail, polyline(pts).build().points());
            },
            true,
        )?;
        let labels = [
            kit.text(stage, TURNS_LABEL, 22.0, input(), v(-4.3, -2.65))?,
            kit.text(stage, TURNS_LABEL, 22.0, output(), v(2.7, -2.65))?,
        ];
        let counts = [
            kit.live_value(
                stage,
                move |s| Walker::turns_so_far(Func::Identity, ZERO, WALK_R, value(s, walked)),
                1,
                false,
                28.0,
                input(),
                v(-3.3, -2.65),
                ORIGIN,
            )?,
            kit.live_value(
                stage,
                move |s| Walker::turns_so_far(Func::Square, ZERO, WALK_R, value(s, walked)),
                1,
                false,
                28.0,
                output(),
                v(3.7, -2.65),
                ORIGIN,
            )?,
        ];
        fade_in_all(stage, &[ring_in, ring_out, w_in, w_out, trail], 0.4)?;
        fade_in_all(stage, &labels, 0.3)?;
        let rt = (line.end() - stage.scene().time().to_f64()).max(3.0);
        stage.play(tween_linear(walked, 1.0, rt)?)?;
        stage.clear_updaters(anchor, true);
        for c in counts {
            stage.clear_updaters(c, true);
            stage.remove_from_scene(c);
        }
        let finals = [
            kit.text(stage, fact("identity.circle.turns")?, 28.0, input(), v(-3.3, -2.65))?,
            kit.text(stage, fact("square.circle.turns")?, 28.0, output(), v(3.7, -2.65))?,
        ];
        let radius = kit.text(
            stage,
            &format!("{RADIUS}{}", fact("square.circle.out_radius")?),
            20.0,
            output(),
            out.p(C::new(0.0, -0.55)),
        )?;
        for f in finals {
            stage.add_to_scene(f)?;
        }
        fade_in_all(stage, &[radius], 0.3)?;

        // 03_3: a grid: will the vertical lines stay straight after z²?
        kit.hold(stage)?;
        let line = say(&kit, stage, "03_3")?;
        let mut gone = vec![rule, ring_in, ring_out, w_in, w_out, trail, radius];
        gone.extend(labels);
        gone.extend(finals);
        fade_out_all(stage, &gone, 0.4)?;
        // same frames, a wider window on both sides for the grid's images
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
        let asked = predict(&kit, stage, &line, &ASK, out.p(C::new(0.0, 0.4)))?;

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
        // the fold: both axes double back at the origin's image
        let fold = curve(
            stage,
            &out,
            &[C::new(0.0, -0.12), C::new(0.0, 0.12)],
            Palette::ink(),
            4.0,
        );
        fade_in_all(stage, &[fold], 0.2)?;
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
        let mut gone = vec![fn_tex, fold, kept, at_zero_in, at_zero_out];
        gone.extend(in_lines);
        gone.extend(copies.iter().map(|(m, _)| *m));
        gone.extend(marks_in);
        gone.extend(marks_out);
        fade_out_all(stage, &gone, 0.4)?;
        // same frames, wider windows
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
        let colors = [hero(), input(), output(), Palette::ink()];
        let mut ticks = Vec::new();
        for (&th, &c) in TICKS.iter().zip(&colors) {
            let a = dot(stage, inp.p(C::polar(1.0, th)), 0.1, c);
            fade_in_all(stage, &[a], 0.2)?;
            let b = dot(stage, out.p(C::polar(1.0, th).recip()), 0.1, c);
            fade_in_all(stage, &[b], 0.3)?;
            ticks.extend([a, b]);
        }
        line.wait_sentence(stage, 3)?;
        let trade = kit.text(stage, TRADE, 22.0, Palette::ink(), caption_at)?;
        fade_in_all(stage, &[trade], 0.4)?;
        line.wait_sentence(stage, 4)?;
        let cross = red_cross(stage, inp.p(ZERO));
        let undefined = carded_lines(
            &kit,
            stage,
            &[UNDEFINED],
            20.0,
            warn(),
            inp.p(C::new(0.0, -0.6)),
            warn(),
        )?;
        fade_in_all(stage, &cross, 0.3)?;
        fade_in_all(stage, &undefined, 0.3)?;
        wait_until(stage, line.end())?;

        // 03_6: e^z: right by one is ×e, up turns; vertical lines → circles, horizontal → rays
        kit.hold(stage)?;
        let line = say(&kit, stage, "03_6")?;
        let mut gone = vec![fn_tex, near, far, trade];
        gone.extend(cross);
        gone.extend(circles);
        gone.extend(ticks);
        gone.extend(undefined);
        gone.extend(in_frame);
        fade_out_all(stage, &gone, 0.4)?;
        let inp = exp_input();
        let (_, out) = planes(INVERSE_HALVES.0, EXP_OUT_HALF);
        let in_frame = framed(stage, &inp, input())?;
        let fn_tex = kit.tex(stage, GALLERY_TITLES[2], 32.0, Palette::ink(), &[], fn_at)?;
        fade_in_all(stage, &in_frame, 0.4)?;
        fade_in_all(stage, &[fn_tex], 0.4)?;
        line.wait_sentence(stage, 1)?;
        let radii = fact_list("exp.circle_radii")?;
        let printed: Vec<&str> = fact("exp.circle_radii")?.split(',').collect();
        let mut steps = Vec::new();
        for (k, x) in [-1.0, 0.0, 1.0].into_iter().enumerate() {
            let a = dot(stage, inp.p(C::real(x)), 0.08, input());
            let b = dot(stage, out.p(C::real(radii[k])), 0.08, output());
            // 1/e and 1 sit close: the middle label goes above the axis
            let l = kit.text(stage, printed[k], 18.0, output(), out.p(radius_label_at(k, radii[k])))?;
            fade_in_all(stage, &[a], 0.2)?;
            fade_in_all(stage, &[b, l], 0.3)?;
            steps.extend([a, b, l]);
        }
        let times_e = kit.tex(stage, TIMES_E_TEX, 26.0, hero(), &[], out.p(C::new(1.85, 0.4)))?;
        fade_in_all(stage, &[times_e], 0.3)?;
        steps.push(times_e);
        line.wait_sentence(stage, 2)?;
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
        all.extend(out_frame);
        all.extend(titles);
        fade_out_all(stage, &all, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get(key: &str) -> &'static str {
        crate::facts::facts().unwrap().get(key).unwrap()
    }

    #[test]
    fn the_walk_counts_match_the_facts() {
        let once = Walker::turns_so_far(Func::Identity, ZERO, WALK_R, 1.0);
        let twice = Walker::turns_so_far(Func::Square, ZERO, WALK_R, 1.0);
        assert!((once - get("identity.circle.turns").parse::<f64>().unwrap()).abs() < 1e-9);
        assert!((twice - get("square.circle.turns").parse::<f64>().unwrap()).abs() < 1e-9);
        assert!((WALK_R * WALK_R - get("square.circle.out_radius").parse::<f64>().unwrap()).abs() < 1e-12);
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
            assert_eq!(
                (a.frame.center, a.frame.width, a.frame.height),
                (b.frame.center, b.frame.width, b.frame.height)
            );
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
                assert!(inp.contains(z) && out.contains(z.exp()));
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
