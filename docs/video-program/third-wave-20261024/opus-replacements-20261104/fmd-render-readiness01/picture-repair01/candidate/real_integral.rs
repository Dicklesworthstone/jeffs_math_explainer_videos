//! 13_real_integral: two real areas by one loop. The bump's area is π (the
//! pole at i, residue 1/(2i)). The arc's share is shown as length × max: a
//! point walks the half circle with |z²| = R² while the bump's size along
//! the arc, unrolled, stays under 1/(R² − 1); the arc's length πR is the
//! base, the rectangle on it is the bound πR/(R² − 1), and it grows long and
//! flat, its area shrinking, as R goes 2 → 5 → 10. Then 1/(1 + x⁴): its
//! antiderivative shown and set aside, the four poles, the two inside,
//! residues −p/4 by p⁴ = −1, their sum −i√2/4, the area π/√2 ≈ 2.22, the arc
//! bound πR · 1/(R⁴ − 1) and a numerical check; last, the honest note on why
//! the symmetric limit is the integral here (storyboard 13_1–13_11). Every
//! number comes from `facts` or `model`.

use std::f64::consts::PI;

use explainer_kit::{Frame2, Kit, Palette, tween, v};
use fmn::prelude::*;

use super::draw::{
    Plane, carded, dot, fact, fade_in_all, fade_out_all, faded_line, flat, ground, hero, line_style, pause, pole,
    pole_mark, precisely_lifted, precisely_over, predict, reading_window, say, shift_all, tween_linear, value,
    wait_until,
};
use super::the_bump::{POLE_I, POLE_MINUS_I};
use crate::model::{self, C, ZERO, arc_bound};

pub const BUMP_AREA_TEX: &str =
    r"\int_{-\infty}^{\infty} \frac{dx}{1+x^2} = \arctan x \,\Big|_{-\infty}^{\infty} = \pi";
pub const LOOP_TEX: &str = r"2\pi i \cdot \frac{1}{2i} = \pi";
/// 13_3, first sentence: on the half circle, z² has length R², so the bump
/// is at most 1/(R² − 1) there.
pub const ZSQ_TEX: &str = r"|z^2| = R^2";
pub const MAX_TEX: &str = r"\Rightarrow\; \Big|\frac{1}{1+z^2}\Big| \le \frac{1}{R^2 - 1}";
pub const R_TEX: &str = "R";
/// The half circle's length, under its unrolled base.
pub const LENGTH_TEX: &str = r"\pi R";
/// The bound as length × max.
pub const ARC_TEX: &str = r"\Big|\int_{\mathrm{arc}}\Big| \le \pi R \cdot \frac{1}{R^2 - 1} = \frac{\pi R}{R^2 - 1}";
pub const SIZE_ALONG: &str = "the bump's size along the half circle";
/// 13_3's plane, while the panel's R grows: says what the picture has done with its scale.
pub const ZOOM_NOTE: &str = "rescaled as R grows: the loop keeps its size, so i and −i close in";
pub const READ_R: &str = "R =";
pub const AT_MOST: &str = "at most";
pub const QUARTIC_TEX: &str = r"\frac{1}{1+x^4}";
/// The antiderivative of 13_4, in two lines (the tests check it as `antiderivative`).
pub const ANTIDERIVATIVE_TEX: [&str; 2] = [
    r"\frac{1}{4\sqrt{2}}\ln\frac{x^2+\sqrt{2}x+1}{x^2-\sqrt{2}x+1}",
    r"+\,\frac{1}{2\sqrt{2}}\Big(\arctan(\sqrt{2}x+1)+\arctan(\sqrt{2}x-1)\Big)",
];
/// "one over the derivative of the bottom", in 12_2's letters for that bottom.
pub const RES_TEX: &str = r"h = 1 + z^4:\quad \mathrm{Res}_p = \frac{1}{h'(p)} = \frac{1}{4p^3}";
pub const FOURTH_TEX: &str = r"p^4 = -1 \;\Rightarrow\; \frac{1}{p^3} = -p";
pub const MINUS_P_TEX: &str = r"\mathrm{Res}_p = -\frac{p}{4}";
/// 13_8: "each has imaginary part the square root of two over two".
pub const IM_TEX: &str = r"\mathrm{Im}\,p = \frac{\sqrt{2}}{2}";
pub const SUM_TEX: &str = r"\mathrm{Res}_{p_1} + \mathrm{Res}_{p_2} = -\frac{p_1 + p_2}{4} = -\frac{i\sqrt{2}}{4}";
/// 13_9 in two steps: "Multiply by two pi i", then the cancellation the voice walks through.
pub const VALUE_TEX: &str = r"2\pi i \cdot \Big(-\frac{i\sqrt{2}}{4}\Big)";
pub const VALUE_RESULT_TEX: &str = r"= \frac{\pi\sqrt{2}}{2} = \frac{\pi}{\sqrt{2}}";
/// 13_2's other end of the segment; `R_TEX` names the near one.
pub const MINUS_R_TEX: &str = "-R";
/// 13_8's yellow arrow, which is the poles' sum and not yet the residues' sum.
pub const TOTAL_TEX: &str = r"p_1 + p_2";
/// 13_8's second arrow, −(p₁ + p₂)/4: the residues' sum points down.
pub const SUM_NAME: &str = "sum";
/// 13_10: the same mechanism for 1/(1 + z⁴): the max, then length × max.
pub const QUARTIC_MAX_TEX: &str = r"\Big|\frac{1}{1+z^4}\Big| \le \frac{1}{R^4 - 1}";
pub const QUARTIC_ARC_TEX: &str =
    r"\Big|\int_{\mathrm{arc}}\Big| \le \pi R \cdot \frac{1}{R^4 - 1} = \frac{\pi R}{R^4 - 1}";
/// Between the R = 10 bound and the fraction it is under ("less than one
/// three hundredth"): "0.0031 < 1/300".
pub const UNDER: &str = " < 1/";
pub const P_NAMES: [&str; 2] = [r"p_1", r"p_2"];
pub const ASK: [&str; 1] = ["which poles are inside?"];
pub const APPROX: &str = "≈ ";
/// Around the sum's reach: "a numerical sum on [−200, 200]: ".
pub const NUMERIC: [&str; 2] = ["a numerical sum on [−", "]: "];
pub const HONEST: &str = "the integral of |f| over the line is finite, so the improper integral is the limit of the integral from −R to R; for slowly decaying f that symmetric limit can exist when the integral does not";
/// As much of [`HONEST`] as 13_11's voice carries: on screen while it speaks, so the warning
/// the voice never says arrives on its own once it has stopped.
pub const HONEST_SAID: &str =
    "the integral of |f| over the line is finite, so the improper integral is the limit of the integral from −R to R";

/// The antiderivative 13_4 shows: its derivative is 1/(1 + x⁴) (tested).
#[cfg(test)]
pub fn antiderivative(x: f64) -> f64 {
    let r = 2.0_f64.sqrt();
    (1.0 / (4.0 * r)) * ((x * x + r * x + 1.0) / (x * x - r * x + 1.0)).ln()
        + (1.0 / (2.0 * r)) * ((r * x + 1.0).atan() + (r * x - 1.0).atan())
}

/// A real graph window for both areas.
fn graph() -> Frame2 {
    Frame2 {
        center: v(-3.0, 2.25),
        width: 7.0,
        height: 2.4,
        x: (-4.0, 4.0),
        y: (-0.15, 1.2),
    }
}

/// The contour's plane, below the graph: the half disk of radius R = 2
/// above [−2, 2], and room for the pole at −i.
fn plane() -> Plane {
    Plane::new(v(-3.0, -1.15), 5.0, (-2.4, 2.4), (-1.25, 2.25))
}

/// The closed contour: [−R, R], then the upper half circle back. The scene draws it through
/// `contour_walked`, which ends on exactly this (tested).
#[cfg(test)]
fn contour(r: f64) -> Vec<C> {
    let mut zs = vec![C::real(-r), C::real(r)];
    zs.extend(arc(r).into_iter().skip(1));
    zs
}

/// As much of the loop as the first `s` of the walk has covered, `s` in 0..=1: the segment takes
/// the first half of the walk, the half circle the second. It ends where the walker is.
fn contour_walked(r: f64, s: f64) -> Vec<C> {
    if s <= 0.5 {
        return vec![C::real(-r), C::real(-r + 4.0 * r * s)];
    }
    let k = ((2.0 * s - 1.0) * 120.0).round() as usize;
    let mut zs = vec![C::real(-r), C::real(r)];
    zs.extend(arc(r).into_iter().take(k + 1).skip(1));
    zs
}

/// The upper half circle alone, from R round to −R.
fn arc(r: f64) -> Vec<C> {
    (0..=120).map(|k| C::polar(r, PI * f64::from(k) / 120.0)).collect()
}

/// 13_3 lifts the loop's result up the column to make room.
const LIFT: f64 = 1.4;
/// 13_3's row for |z²| = R² ⇒ |f| ≤ 1/(R² − 1).
const ROW_Y: f64 = 1.5;
/// "its size along the half circle", above the unrolled window.
const ALONG_AT: (f64, f64) = (2.9, 0.75);
/// The readouts: names end at `READ_X`, numbers start just after it.
const READ_X: f64 = 5.4;
const READ_GAP: f64 = 0.12;
const READ_Y: [f64; 2] = [0.2, -0.25];
/// 13_3's bound, under the unrolled window.
const BOUND_Y: f64 = -2.0;
/// 13_3's note on the plane's scale, under the plane.
const ZOOM_NOTE_AT: (f64, f64) = (-3.0, -3.25);
/// 13_10: the boxed value moves up; the max, the bound, the two readings
/// and the numerical check take these rows.
const QUARTIC_LIFT: f64 = 1.7;
const QUARTIC_ROWS: [f64; 4] = [0.6, -0.35, -1.15, -1.85];

/// The half circle unrolled: position along the arc to the right (its
/// length is πR), the size of 1/(1 + z²) there upward. One scale holds every
/// radius 13_3 grows through, so the rectangle length × max visibly turns
/// long and flat; its area is the bound.
fn unrolled() -> Frame2 {
    let first = model::ARC_RADII[0];
    let last = model::ARC_RADII[model::ARC_RADII.len() - 1];
    Frame2 {
        center: v(3.9, -0.3),
        width: 5.4,
        height: 1.6,
        x: (0.0, PI * last * 1.02),
        y: (0.0, 1.15 * model::arc_max(2, first)),
    }
}

/// The size of 1/(1 + z²) along the half circle of radius `r`, unrolled,
/// from its start to angle `upto`.
fn profile(u: &Frame2, r: f64, upto: f64) -> Vec<Vec3> {
    let n = ((160.0 * upto / PI).round() as usize).max(1);
    (0..=n)
        .map(|k| {
            let th = upto * k as f64 / n as f64;
            u.p(r * th, model::arc_size(2, C::polar(r, th)))
        })
        .collect()
}

/// The rectangle length × max for radius `r`: [0, πR] × [0, 1/(R² − 1)].
fn length_by_max(u: &Frame2, r: f64) -> Vec<Vec3> {
    let (l, m) = (PI * r, model::arc_max(2, r));
    vec![u.p(0.0, 0.0), u.p(l, 0.0), u.p(l, m), u.p(0.0, m), u.p(0.0, 0.0)]
}

/// The rectangle's top: the max, 1/(R² − 1), over the whole length.
fn max_line(u: &Frame2, r: f64) -> Vec<Vec3> {
    let m = model::arc_max(2, r);
    vec![u.p(0.0, m), u.p(PI * r, m)]
}

/// The rectangle's base: the arc's length, πR.
fn base(u: &Frame2, r: f64) -> Vec<Vec3> {
    vec![u.p(0.0, 0.0), u.p(PI * r, 0.0)]
}

/// Where "πR" sits: under the middle of the base.
fn length_label_at(u: &Frame2, r: f64) -> Vec3 {
    let b = u.p(PI * r / 2.0, 0.0);
    v(b[0], b[1] - 0.28)
}

/// How high i is drawn while the panel reads radius `at`: the plane's loop keeps the size it has
/// at radius `r`, so the scale is r/at and the distance from i to the loop stays in proportion.
fn pole_height(r: f64, at: f64) -> f64 {
    r / at
}

/// Where "R" sits beside the spoke from 0 to the walking point at angle
/// `th`: past its middle, on the clockwise side (below it at the start). Past the middle
/// because the middle of the upright spoke, at R = 2, is the pole at i.
fn r_name_z(r: f64, th: f64) -> C {
    C::polar(r * 0.72, th) + C::polar(0.25, th - PI / 2.0)
}

/// The same, as a scene point.
fn r_name_at(p: &Plane, r: f64, th: f64) -> Vec3 {
    p.p(r_name_z(r, th))
}

/// The walk's angles over which "R" comes in. On the clockwise side it starts under the segment
/// and has to cross it, so it stays away until that is over; until then the "R" that names the
/// segment's end also names the spoke lying along it.
const R_NAME_SHOWN: [f64; 2] = [0.28, 0.42];

/// How much of "R" and its backing shows at walk angle `th`.
fn r_name_opacity(th: f64) -> f64 {
    ((th - R_NAME_SHOWN[0]) / (R_NAME_SHOWN[1] - R_NAME_SHOWN[0])).clamp(0.0, 1.0)
}

/// How far left of the axis the pole's "i" is written, and how far right "−i", in plane units.
const POLE_NAME_X: f64 = -0.28;
const MINUS_POLE_NAME_X: f64 = 0.36;
/// Dark backings under the walk's two names, the pole mark's own device, in scene units. The axis
/// goes behind "R" and the spoke behind "i", so neither line ever runs through lettering.
const R_BACK: f64 = 0.135;
const I_BACK: f64 = 0.1;

/// The graph of `f` with the area under it shaded: `[area, curve]`.
fn area(stage: &mut Stage<'_>, g: &Frame2, f: impl Fn(f64) -> f64) -> [Mob; 2] {
    let n = 320;
    let top: Vec<Vec3> = (0..=n)
        .map(|i| {
            let x = g.x.0 + (g.x.1 - g.x.0) * f64::from(i) / f64::from(n);
            g.p(x, f(x))
        })
        .collect();
    let mut shape = top.clone();
    shape.push(g.p(g.x.1, 0.0));
    shape.push(g.p(g.x.0, 0.0));
    let fill = stage
        .arena_mut()
        .add(Polygon::new(shape).style(flat(hero(), 0.28)).build());
    let curve = stage
        .arena_mut()
        .add(polyline(top).style(line_style(Palette::ink(), 4.0)).build());
    [fill, curve]
}

/// Arrows along the contour: one on the segment, one at the top of the arc.
fn contour_arrows(stage: &mut Stage<'_>, p: &Plane, r: f64) -> fmn::Result<Vec<Mob>> {
    let along = Arrow::new(p.p(C::real(0.2)), p.p(C::real(0.8)))
        .buff(0.0)
        .thickness(4.0)
        .color(hero())
        .build()?;
    let back = Arrow::new(p.p(C::polar(r, PI / 2.0 - 0.15)), p.p(C::polar(r, PI / 2.0 + 0.15)))
        .buff(0.0)
        .thickness(4.0)
        .color(hero())
        .build()?;
    Ok(vec![stage.arena_mut().add(along), stage.arena_mut().add(back)])
}

pub struct RealIntegral {
    pub kit: Kit,
}

impl SceneConstruct for RealIntegral {
    fn name(&self) -> &str {
        "13_real_integral"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let g = graph();
        let p = plane();
        let right = 3.9;
        // the contour drawn is the first half circle 13_3 grows from
        let r = model::ARC_RADII[0];

        // 13_1: the bump's area, by calculus: π
        let line = say(&kit, stage, "13_1")?;
        let axis = stage.arena_mut().add(
            polyline(vec![g.p(g.x.0, 0.0), g.p(g.x.1, 0.0)])
                .style(faded_line(Palette::dim(), 2.0, 0.9))
                .build(),
        );
        let bump = area(stage, &g, model::bump);
        line.wait_sentence(stage, 1)?;
        fade_in_all(stage, &[axis, bump[1]], 0.5)?;
        fade_in_all(stage, &[bump[0]], 0.6)?;
        line.wait_sentence(stage, 3)?;
        let calculus = kit.tex(stage, BUMP_AREA_TEX, 34.0, Palette::ink(), &[], v(right, 2.8))?;
        fade_in_all(stage, &[calculus], 0.5)?;
        // "a second way, with no antiderivative": the first way steps back
        line.wait_sentence(stage, 4)?;
        stage.set_opacity(calculus, 0.5, true);

        // 13_2: the loop: [−R, R] and the half circle; one pole inside; 2πi · 1/(2i) = π
        kit.hold(stage)?;
        let line = say(&kit, stage, "13_2")?;
        let axes = p.axes(stage)?;
        fade_in_all(stage, &axes, 0.4)?;
        // the loop is drawn as the voice walks it, so the path starts as nothing at −R
        let path = p.path(stage, &contour_walked(r, 0.0), hero(), 5.0)?;
        let mut arrows = contour_arrows(stage, &p, r)?;
        // the voice names both ends and the pole: so does the picture
        arrows.push(kit.tex(stage, MINUS_R_TEX, 24.0, Palette::ink(), &[], p.p(C::new(-r, -0.3)))?);
        arrows.push(kit.tex(stage, R_TEX, 24.0, Palette::ink(), &[], p.p(C::new(r, -0.3)))?);
        fade_in_all(stage, &arrows[2..], 0.3)?;
        let on_loop = dot(stage, p.p(C::real(-r)), 0.08, Palette::ink());
        fade_in_all(stage, &[on_loop], 0.2)?;
        stage.add_to_scene(path)?;
        let walked = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        stage.add_updater(
            anchor,
            move |a, _| {
                let zs = contour_walked(r, value(a, walked));
                a.move_to(on_loop, p.p(zs[zs.len() - 1]), ORIGIN);
                let pts: Vec<Vec3> = zs.iter().map(|&z| p.p(z)).collect();
                let _ = a.set_points(path, polyline(pts).build().points());
            },
            true,
        )?;
        let rt = line.until(stage, "then come back", 1.4)?;
        stage.play(tween_linear(walked, 0.5, rt)?)?;
        fade_in_all(stage, &arrows[..1], 0.2)?;
        let rt = line.until(stage, "Inside this loop", 1.6)?;
        stage.play(tween_linear(walked, 1.0, rt)?)?;
        stage.clear_updaters(anchor, true);
        stage.remove_from_scene(anchor);
        fade_in_all(stage, &arrows[1..2], 0.2)?;
        fade_out_all(stage, &[on_loop], 0.2)?;
        line.wait_sentence(stage, 1)?;
        let inside = pole_mark(stage, p.p(C::new(0.0, 1.0)));
        let outside = pole_mark(stage, p.p(C::new(0.0, -1.0)));
        let pole_names = [
            kit.text(stage, POLE_I, 24.0, pole(), p.p(C::new(POLE_NAME_X, 1.0)))?,
            kit.text(stage, POLE_MINUS_I, 24.0, pole(), p.p(C::new(MINUS_POLE_NAME_X, -1.0)))?,
        ];
        fade_in_all(stage, &inside, 0.3)?;
        fade_in_all(stage, &outside, 0.3)?;
        fade_in_all(stage, &pole_names, 0.3)?;
        // set_opacity also writes fill alpha, which would fill the ring: dim its stroke only
        stage.set_opacity(outside[0], 0.35, true);
        stage.set_stroke(outside[1], None, None, Some(0.35), None, true);
        line.wait_sentence(stage, 2)?;
        let by_loop = kit.tex(stage, LOOP_TEX, 36.0, hero(), &[], v(right, 1.6))?;
        fade_in_all(stage, &[by_loop], 0.5)?;
        line.wait_sentence(stage, 3)?;
        let pi_value = kit.text(
            stage,
            &format!("{APPROX}{}", fact("integral.bump.loop")?),
            28.0,
            hero(),
            v(right, 0.85),
        )?;
        fade_in_all(stage, &[pi_value], 0.3)?;

        // 13_3: the arc's share as length × max. A point walks the half
        // circle (|z²| = R² all the way) while the bump's size there is
        // traced along the unrolled arc; it never passes 1/(R² − 1) and
        // touches it at the top, nearest the pole. The arc's length πR is the
        // base, the rectangle on it is the bound, and as R goes 2 → 5 → 10 it
        // grows long and flat while its area, the readout, shrinks.
        kit.hold(stage)?;
        let line = say(&kit, stage, "13_3")?;
        fade_out_all(stage, &[calculus], 0.3)?;
        shift_all(stage, &[by_loop, pi_value], 0.0, LIFT, 0.4)?;
        let u = unrolled();
        let zsq = kit.tex(stage, ZSQ_TEX, 28.0, Palette::ink(), &[], v(right - 1.4, ROW_Y))?;
        let start = C::real(r);
        let zdot = dot(stage, p.p(start), 0.08, Palette::ink());
        let spoke = stage.arena_mut().add(
            polyline(vec![p.p(ZERO), p.p(start)])
                .style(line_style(Palette::ink(), 2.5))
                .build(),
        );
        let r_name = kit.tex(stage, R_TEX, 24.0, Palette::ink(), &[], r_name_at(&p, r, 0.0))?;
        let r_back = dot(stage, r_name_at(&p, r, 0.0), R_BACK, ground());
        let i_back = dot(stage, p.p(C::new(POLE_NAME_X, 1.0)), I_BACK, ground());
        let u_axis = stage.arena_mut().add(
            polyline(vec![u.p(u.x.0, 0.0), u.p(u.x.1, 0.0)])
                .style(faded_line(Palette::dim(), 2.0, 0.9))
                .build(),
        );
        let along = kit.text(stage, SIZE_ALONG, 20.0, Palette::dim(), v(ALONG_AT.0, ALONG_AT.1))?;
        let trace = stage.arena_mut().add(
            polyline(profile(&u, r, 0.0))
                .style(line_style(Palette::ink(), 3.0))
                .build(),
        );
        // the spoke sweeps every point of the half disk, so it has to pass the pole and its name:
        // rooted first, it goes behind them and behind the segment's arrow
        stage.add_to_scene(spoke)?;
        for m in [arrows[0], i_back, inside[0], inside[1], pole_names[0]] {
            stage.add_to_scene(m)?;
        }
        fade_in_all(stage, &[zdot, spoke, zsq, u_axis, along], 0.4)?;
        stage.add_to_scene(trace)?;
        stage.add_to_scene(r_back)?;
        stage.add_to_scene(r_name)?;
        let walk = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        stage.add_updater(
            anchor,
            move |a, _| {
                let th = value(a, walk).clamp(0.0, PI);
                let z = C::polar(r, th);
                a.move_to(zdot, p.p(z), ORIGIN);
                let _ = a.set_points(spoke, polyline(vec![p.p(ZERO), p.p(z)]).build().points());
                a.move_to(r_name, r_name_at(&p, r, th), ORIGIN);
                a.move_to(r_back, r_name_at(&p, r, th), ORIGIN);
                let shown = r_name_opacity(th);
                a.set_opacity(r_name, shown, true);
                a.set_opacity(r_back, shown, true);
                let _ = a.set_points(trace, polyline(profile(&u, r, th)).build().points());
            },
            true,
        )?;
        let rt = line.until(stage, "so the bump", 1.5)?;
        stage.play(tween_linear(walk, PI, rt)?)?;
        stage.clear_updaters(anchor, true);
        stage.remove_from_scene(anchor);
        line.wait_for(stage, "so the bump")?;
        let top = stage
            .arena_mut()
            .add(polyline(max_line(&u, r)).style(line_style(hero(), 3.0)).build());
        let max_tex = kit.tex(stage, MAX_TEX, 28.0, hero(), &[], v(right + 1.25, ROW_Y))?;
        fade_in_all(stage, &[top, max_tex], 0.4)?;
        line.wait_sentence(stage, 1)?;
        let arc_lit = p.path(stage, &arc(r), Palette::ink(), 8.0)?;
        let floor = stage
            .arena_mut()
            .add(polyline(base(&u, r)).style(line_style(Palette::ink(), 6.0)).build());
        let length = kit.tex(stage, LENGTH_TEX, 26.0, Palette::ink(), &[], length_label_at(&u, r))?;
        fade_in_all(stage, &[arc_lit, floor, length], 0.4)?;
        line.wait_sentence(stage, 2)?;
        fade_out_all(stage, &[zdot, spoke, r_name, r_back, i_back], 0.3)?;
        let rect = stage
            .arena_mut()
            .add(polyline(length_by_max(&u, r)).style(flat(hero(), 0.3)).build());
        let bound = kit.tex(stage, ARC_TEX, 30.0, Palette::ink(), &[], v(right, BOUND_Y))?;
        fade_in_all(stage, &[rect, bound], 0.5)?;
        let grown = stage.add_value_tracker(r);
        let r_says = kit.text(stage, READ_R, 24.0, Palette::ink(), v(READ_X, READ_Y[0]))?;
        let at_most = kit.text(stage, AT_MOST, 24.0, hero(), v(READ_X, READ_Y[1]))?;
        stage.move_to(r_says, v(READ_X, READ_Y[0]), RIGHT);
        stage.move_to(at_most, v(READ_X, READ_Y[1]), RIGHT);
        fade_in_all(stage, &[r_says, at_most], 0.3)?;
        // to as many places as the bound's own digits need: at 0 places a paused frame paired
        // "R = 2" with the bound of R = 2.8
        let r_live = kit.live_number(
            stage,
            grown,
            2,
            false,
            24.0,
            Palette::ink(),
            v(READ_X + READ_GAP, READ_Y[0]),
            LEFT,
        )?;
        let b_live = kit.live_value(
            stage,
            move |s| arc_bound(2, value(s, grown)),
            3,
            false,
            24.0,
            hero(),
            v(READ_X + READ_GAP, READ_Y[1]),
            LEFT,
        )?;
        line.wait_for(stage, "which shrinks")?;
        fade_out_all(stage, &[along], 0.3)?;
        let zoom_note = kit.text(
            stage,
            ZOOM_NOTE,
            20.0,
            Palette::dim(),
            v(ZOOM_NOTE_AT.0, ZOOM_NOTE_AT.1),
        )?;
        fade_in_all(stage, &[zoom_note], 0.3)?;
        let total = line.until(stage, "All of pi", 2.4)?;
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        stage.add_updater(
            anchor,
            move |a, _| {
                let at = value(a, grown);
                let _ = a.set_points(rect, polyline(length_by_max(&u, at)).build().points());
                let _ = a.set_points(top, polyline(max_line(&u, at)).build().points());
                let _ = a.set_points(floor, polyline(base(&u, at)).build().points());
                let _ = a.set_points(trace, polyline(profile(&u, at, PI)).build().points());
                a.move_to(length, length_label_at(&u, at), ORIGIN);
                let k = pole_height(r, at);
                for m in inside {
                    a.move_to(m, p.p(C::new(0.0, k)), ORIGIN);
                }
                for m in outside {
                    a.move_to(m, p.p(C::new(0.0, -k)), ORIGIN);
                }
                a.move_to(pole_names[0], p.p(C::new(POLE_NAME_X, k)), ORIGIN);
                a.move_to(pole_names[1], p.p(C::new(MINUS_POLE_NAME_X, -k)), ORIGIN);
            },
            true,
        )?;
        let stops = &model::ARC_RADII[1..];
        for &to in stops {
            stage.play(tween(grown, to, 0.85 * total / stops.len() as f64)?)?;
            pause(stage, 0.15 * total / stops.len() as f64)?;
        }
        stage.clear_updaters(anchor, true);
        stage.remove_from_scene(anchor);
        // the readouts settle on the printed facts for the last radius
        let radii: Vec<&str> = fact("integral.bump.radii")?.split(',').collect();
        let arcs: Vec<&str> = fact("integral.bump.arc")?.split(',').collect();
        let (Some(r_last), Some(b_last)) = (radii.last(), arcs.last()) else {
            return Err(super::draw::fail("13_3 has no radii or arc bounds in the facts".into()));
        };
        if radii.len() != model::ARC_RADII.len() || arcs.len() != radii.len() {
            return Err(super::draw::fail(format!(
                "13_3 grows through {} radii; the facts print {} radii and {} bounds",
                model::ARC_RADII.len(),
                radii.len(),
                arcs.len()
            )));
        }
        stage.clear_updaters(r_live, true);
        stage.clear_updaters(b_live, true);
        let r_fact = kit.text(stage, r_last, 24.0, Palette::ink(), v(READ_X + READ_GAP, READ_Y[0]))?;
        let b_fact = kit.text(stage, b_last, 24.0, hero(), v(READ_X + READ_GAP, READ_Y[1]))?;
        stage.move_to(r_fact, v(READ_X + READ_GAP, READ_Y[0]), LEFT);
        stage.move_to(b_fact, v(READ_X + READ_GAP, READ_Y[1]), LEFT);
        stage.remove_from_scene(r_live);
        stage.remove_from_scene(b_live);
        stage.add_to_scene(r_fact)?;
        stage.add_to_scene(b_fact)?;
        // all of π is the real line's: the segment lit, the bump's area brighter
        line.wait_sentence(stage, 3)?;
        fade_out_all(stage, &[arc_lit], 0.3)?;
        let segment_lit = p.path(stage, &[C::real(-r), C::real(r)], Palette::ink(), 8.0)?;
        fade_in_all(stage, &[segment_lit], 0.4)?;
        stage.set_opacity(bump[0], 0.5, true);

        // 13_4: a harder area, 1/(1 + x⁴); its antiderivative, set aside
        kit.hold(stage)?;
        let line = say(&kit, stage, "13_4")?;
        let mut gone = vec![
            by_loop,
            pi_value,
            zsq,
            max_tex,
            bound,
            bump[0],
            bump[1],
            u_axis,
            trace,
            top,
            floor,
            rect,
            length,
            r_says,
            at_most,
            r_fact,
            b_fact,
            segment_lit,
            zoom_note,
        ];
        gone.extend(inside);
        gone.extend(outside);
        gone.extend(pole_names);
        fade_out_all(stage, &gone, 0.4)?;
        let quartic = area(stage, &g, |x| 1.0 / (1.0 + x.powi(4)));
        let name = kit.tex(stage, QUARTIC_TEX, 36.0, Palette::ink(), &[], v(right, 2.8))?;
        line.wait_sentence(stage, 1)?;
        fade_in_all(stage, &[quartic[1], name], 0.5)?;
        fade_in_all(stage, &[quartic[0]], 0.5)?;
        line.wait_sentence(stage, 2)?;
        let messy = [
            kit.tex(stage, ANTIDERIVATIVE_TEX[0], 24.0, Palette::ink(), &[], v(right, 1.65))?,
            kit.tex(stage, ANTIDERIVATIVE_TEX[1], 24.0, Palette::ink(), &[], v(right, 0.7))?,
        ];
        fade_in_all(stage, &messy, 0.5)?;
        line.wait_sentence(stage, 3)?;
        for m in messy {
            stage.set_opacity(m, 0.3, true);
        }

        // 13_5: the four poles; which are inside?
        kit.hold(stage)?;
        let line = say(&kit, stage, "13_5")?;
        fade_out_all(stage, &messy, 0.3)?;
        let roots = model::quartic_roots();
        let mut dims = Vec::new();
        let (mut dims_above, mut dims_below) = (Vec::new(), Vec::new());
        for &q in &roots {
            let d = super::draw::dot(stage, p.p(q), 0.08, Palette::dim());
            dims.push(d);
            if q.im < 0.0 {
                dims_below.push(d);
            } else {
                dims_above.push(d);
            }
        }
        let unit = p.circle(stage, ZERO, 1.0, Palette::faint(), 2.0)?;
        line.wait_sentence(stage, 1)?;
        fade_in_all(stage, &[unit], 0.3)?;
        fade_in_all(stage, &dims, 0.4)?;
        let degrees: Vec<&str> = fact("integral.quartic.roots_deg")?.split(',').collect();
        let mut deg_labels = Vec::new();
        for (q, d) in roots.iter().zip(&degrees) {
            let at = p.p(q.scale(1.32));
            deg_labels.push(kit.text(stage, &format!("{d}°"), 24.0, Palette::ink(), at)?);
        }
        fade_in_all(stage, &deg_labels, 0.3)?;
        let asked = predict(&kit, stage, &line, &ASK, v(right, 1.2))?;

        // 13_6: the two above the line, p₁ and p₂
        let line = say(&kit, stage, "13_6")?;
        fade_out_all(stage, &asked, 0.3)?;
        let inside_count = fact("integral.quartic.inside")?;
        if inside_count != "2" {
            return Err(super::draw::fail(format!(
                "13_6 says two poles inside, facts say {inside_count}"
            )));
        }
        let mut lit = Vec::new();
        let upper: Vec<C> = roots.iter().copied().filter(|q| q.im > 0.0).collect();
        for (q, n) in upper.iter().zip(P_NAMES) {
            lit.extend(pole_mark(stage, p.p(*q)));
            lit.push(kit.tex(stage, n, 30.0, pole(), &[], p.p(*q + C::new(0.0, 0.32)))?);
        }
        line.wait_sentence(stage, 1)?;
        // the angles have done their work, and the two upper ones would crowd the names
        fade_out_all(stage, &deg_labels, 0.3)?;
        fade_in_all(stage, &lit, 0.4)?;

        // 13_7: residue 1/(4p³) = −p/4
        kit.hold(stage)?;
        let line = say(&kit, stage, "13_7")?;
        let res = kit.tex(stage, RES_TEX, 34.0, Palette::ink(), &[], v(right, 1.6))?;
        fade_in_all(stage, &[res], 0.4)?;
        line.wait_sentence(stage, 1)?;
        let fourth = kit.tex(stage, FOURTH_TEX, 32.0, Palette::ink(), &[], v(right, 0.6))?;
        fade_in_all(stage, &[fourth], 0.4)?;
        line.wait_sentence(stage, 2)?;
        let minus_p = kit.tex(stage, MINUS_P_TEX, 34.0, hero(), &[], v(right, -0.4))?;
        fade_in_all(stage, &[minus_p], 0.4)?;

        // 13_8: add: the real parts cancel; the sum is −i√2/4
        kit.hold(stage)?;
        let line = say(&kit, stage, "13_8")?;
        fade_out_all(stage, &[res, fourth], 0.3)?;
        let (p1, p2) = (upper[0], upper[1]);
        // tip to tail, so no buffer: each arrow starts where the last one ends
        let a1 = Arrow::new(p.p(ZERO), p.p(p1))
            .buff(0.0)
            .thickness(4.0)
            .color(pole())
            .build()?;
        let a2 = Arrow::new(p.p(p1), p.p(p1 + p2))
            .buff(0.0)
            .thickness(4.0)
            .color(pole())
            .build()?;
        let a1 = stage.arena_mut().add(a1);
        let a2 = stage.arena_mut().add(a2);
        line.wait_sentence(stage, 1)?;
        fade_in_all(stage, &[a1], 0.4)?;
        fade_in_all(stage, &[a2], 0.4)?;
        let total = Arrow::new(p.p(ZERO), p.p(p1 + p2))
            .buff(0.0)
            .thickness(5.0)
            .color(hero())
            .build()?;
        let total = stage.arena_mut().add(total);
        let total_name = kit.tex(stage, TOTAL_TEX, 24.0, hero(), &[], p.p(p1 + p2 + C::new(0.62, 0.02)))?;
        fade_in_all(stage, &[total, total_name], 0.4)?;
        line.wait_for(stage, "each has imaginary part")?;
        let im_row = kit.tex(stage, IM_TEX, 30.0, Palette::ink(), &[], v(right, 0.7))?;
        fade_in_all(stage, &[im_row], 0.4)?;
        line.wait_sentence(stage, 2)?;
        // the residues' sum is −(p₁ + p₂)/4: a quarter of that arrow, turned to point down
        let sum_tip = (p1 + p2).scale(-0.25);
        let sum_arrow = Arrow::new(p.p(ZERO), p.p(sum_tip))
            .buff(0.0)
            .thickness(5.0)
            .color(hero())
            .build()?;
        let sum_arrow = stage.arena_mut().add(sum_arrow);
        let sum_name = kit.text_left(stage, SUM_NAME, 22.0, hero(), p.p(C::new(0.14, sum_tip.im + 0.02)))?;
        fade_in_all(stage, &[sum_arrow, sum_name], 0.4)?;
        let sum = kit.tex(stage, SUM_TEX, 30.0, Palette::ink(), &[], v(right, -1.3))?;
        let im = fact("integral.quartic.sum")?
            .split(',')
            .nth(1)
            .ok_or_else(|| super::draw::fail("integral.quartic.sum has no imaginary part".into()))?;
        // set as mathematics, so its i is the italic i of the line above
        let sum_value = kit.tex(stage, &format!(r"\approx {im}\,i"), 26.0, hero(), &[], v(right, -2.0))?;
        fade_in_all(stage, &[sum, sum_value], 0.4)?;

        // 13_9: × 2πi: π/√2 ≈ 2.22
        kit.hold(stage)?;
        let line = say(&kit, stage, "13_9")?;
        fade_out_all(stage, &[minus_p, im_row, sum, sum_value, sum_arrow, sum_name], 0.3)?;
        let worked = kit.tex(stage, VALUE_TEX, 32.0, Palette::ink(), &[], v(right, 1.6))?;
        fade_in_all(stage, &[worked], 0.5)?;
        // the result waits for the cancellation the voice walks through
        line.wait_sentence(stage, 1)?;
        let worked_result = kit.tex(stage, VALUE_RESULT_TEX, 32.0, Palette::ink(), &[], v(right, 0.75))?;
        fade_in_all(stage, &[worked_result], 0.5)?;
        line.wait_sentence(stage, 2)?;
        let boxed = carded(
            &kit,
            stage,
            &format!("{APPROX}{}", fact("integral.quartic.value2")?),
            36.0,
            hero(),
            v(right, 0.0),
            hero(),
        )?;
        fade_in_all(stage, &boxed, 0.4)?;

        // 13_10: the half circle again, by 13_3's mechanism: the max
        // 1/(R⁴ − 1), then length × max; read at the loop drawn and at the
        // last radius, under 1/300 there; a numerical sum agrees
        kit.hold(stage)?;
        let line = say(&kit, stage, "13_10")?;
        fade_out_all(stage, &[worked, worked_result], 0.3)?;
        shift_all(stage, &boxed, 0.0, QUARTIC_LIFT, 0.4)?;
        let arc_lit = p.path(stage, &arc(r), Palette::ink(), 8.0)?;
        fade_in_all(stage, &[arc_lit], 0.4)?;
        line.wait_sentence(stage, 1)?;
        let qmax = kit.tex(stage, QUARTIC_MAX_TEX, 28.0, hero(), &[], v(right, QUARTIC_ROWS[0]))?;
        fade_in_all(stage, &[qmax], 0.4)?;
        line.wait_for(stage, "so that part")?;
        let qbound = kit.tex(
            stage,
            QUARTIC_ARC_TEX,
            30.0,
            Palette::ink(),
            &[],
            v(right, QUARTIC_ROWS[1]),
        )?;
        fade_in_all(stage, &[qbound], 0.4)?;
        let qradii: Vec<&str> = fact("integral.quartic.radii")?.split(',').collect();
        let qarc: Vec<&str> = fact("integral.quartic.arc")?.split(',').collect();
        let (&[r_near, r_far], &[b_near, b_far]) = (qradii.as_slice(), qarc.as_slice()) else {
            return Err(super::draw::fail(format!(
                "13_10 reads two radii; the facts print {} radii and {} bounds",
                qradii.len(),
                qarc.len()
            )));
        };
        let near = kit.text(
            stage,
            &format!("{READ_R} {r_near}:  {b_near}"),
            24.0,
            Palette::ink(),
            v(right - 1.6, QUARTIC_ROWS[2]),
        )?;
        fade_in_all(stage, &[near], 0.3)?;
        line.wait_sentence(stage, 2)?;
        let far = kit.text(
            stage,
            &format!("{READ_R} {r_far}:  {b_far}{UNDER}{}", fact("integral.quartic.under")?),
            24.0,
            hero(),
            v(right + 1.35, QUARTIC_ROWS[2]),
        )?;
        fade_in_all(stage, &[far], 0.4)?;
        line.wait_sentence(stage, 3)?;
        let reach = fact("integral.quartic.numeric_reach")?;
        let numeric = kit.text(
            stage,
            &format!(
                "{}{reach}, {reach}{}{}",
                NUMERIC[0],
                NUMERIC[1],
                fact("integral.quartic.numeric")?
            ),
            24.0,
            Palette::ink(),
            v(right, QUARTIC_ROWS[3]),
        )?;
        fade_in_all(stage, &[numeric], 0.4)?;

        // 13_11: the honest note
        kit.hold(stage)?;
        let line = say(&kit, stage, "13_11")?;
        line.wait_sentence(stage, 1)?;
        // the card is about to cover the lower half plane: the two poles there go first
        fade_out_all(stage, &dims_below, 0.3)?;
        let note = precisely_card(&kit, stage, HONEST)?;
        let said = precisely_lifted(&kit, stage, HONEST_SAID, note)?;
        wait_until(stage, line.end())?;
        precisely_over(stage, said, note)?;
        reading_window(&kit, stage, &line)?;
        kit.music_breath(stage, 2.5)?;
        let mut all = vec![
            axis, quartic[0], quartic[1], name, path, unit, a1, a2, total, total_name, arc_lit, qmax, qbound, near,
            far, numeric, note,
        ];
        all.extend(axes);
        all.extend(arrows);
        all.extend(dims_above);
        all.extend(lit);
        all.extend(boxed);
        fade_out_all(stage, &all, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_antiderivative_shown_is_right() {
        for x in [-3.0, -1.0, -0.4, 0.0, 0.7, 1.0, 2.5] {
            let h = 1e-5;
            let d = (antiderivative(x + h) - antiderivative(x - h)) / (2.0 * h);
            assert!((d - 1.0 / (1.0 + x.powi(4))).abs() < 1e-8, "at {x}");
        }
        // and its rise over the whole line is π/√2
        let rise = antiderivative(1e7) - antiderivative(-1e7);
        assert!((rise - PI / 2.0_f64.sqrt()).abs() < 1e-6);
    }

    /// The path walked in steps no longer than `step`, the closing join included. Every
    /// sample lies on a chord of the path as drawn, so the curve is the same; only the samples
    /// are finer. The picture needs just the two ends of the diameter, but from a point near
    /// the loop that one chord turns `z − q` by more than the quarter turn the sampled count
    /// accepts per step (`model::MAX_STEP_TURN`), and the count rightly refuses it.
    fn walked(path: &[C], step: f64) -> Vec<C> {
        let mut zs = Vec::new();
        for (k, &a) in path.iter().enumerate() {
            let b = path[(k + 1) % path.len()];
            let pieces = ((b - a).abs() / step).ceil().max(1.0) as u32;
            zs.extend((0..pieces).map(|j| a + (b - a).scale(f64::from(j) / f64::from(pieces))));
        }
        zs
    }

    #[test]
    fn the_contour_encloses_exactly_the_upper_poles() {
        let r = model::ARC_RADII[0];
        let path = walked(&contour(r), 1.0 / 256.0);
        for q in model::quartic_roots() {
            let turns = model::winding_count(|z: C| Some(z - q), &path);
            assert_eq!(turns, Some(i64::from(q.im > 0.0)));
        }
        assert_eq!(model::winding_count(|z: C| Some(z - C::new(0.0, 1.0)), &path), Some(1));
        assert_eq!(model::winding_count(|z: C| Some(z - C::new(0.0, -1.0)), &path), Some(0));
        // the loop is the half disk's edge on every side: just inside it, just outside it
        for (q, inside) in [
            (C::new(0.0, r - 0.01), 1),
            (C::new(0.0, r + 0.01), 0),
            (C::new(0.0, 0.01), 1),
            (C::new(0.0, -0.01), 0),
            (C::new(r - 0.02, 0.01), 1),
            (C::new(r + 0.01, 0.0), 0),
            (C::new(0.02 - r, 0.01), 1),
            (C::new(-r - 0.01, 0.0), 0),
        ] {
            let turns = model::winding_count(|z: C| Some(z - q), &path);
            assert_eq!(turns, Some(inside), "at {q:?}");
        }
    }

    #[test]
    fn the_loop_is_walked_to_the_whole_contour() {
        let r = model::ARC_RADII[0];
        // starts as nothing at −R, has the whole segment at the halfway mark, ends as the contour
        assert_eq!(contour_walked(r, 0.0), [C::real(-r), C::real(-r)]);
        assert_eq!(contour_walked(r, 0.5), [C::real(-r), C::real(r)]);
        assert_eq!(contour_walked(r, 1.0), contour(r));
        // it only ever grows
        let (mut last, mut last_len) = (C::real(-r), 2);
        for k in 1..=100 {
            let zs = contour_walked(r, f64::from(k) / 100.0);
            assert!(zs.len() >= last_len);
            assert!((zs[zs.len() - 1] - last).abs() < r);
            (last, last_len) = (zs[zs.len() - 1], zs.len());
        }
    }

    #[test]
    fn the_poles_close_in_by_the_true_ratio() {
        let first = model::ARC_RADII[0];
        let p = plane();
        for at in model::ARC_RADII {
            let k = pole_height(first, at);
            // in the drawn loop's units, i is as far along the radius as it truly is
            assert!((k / first - 1.0 / at).abs() < 1e-12);
            assert!(k > 0.0 && k <= 1.0 && p.contains(C::new(0.0, k)) && p.contains(C::new(0.0, -k)));
        }
        // the note is under the plane, and the voice's picture is not behind it
        assert!(ZOOM_NOTE_AT.1 < p.frame.bottom() - 0.15 && ZOOM_NOTE_AT.1 > -3.75);
    }

    #[test]
    fn the_residue_and_imaginary_part_shown_are_right() {
        let up: Vec<C> = model::quartic_roots().into_iter().filter(|q| q.im > 0.0).collect();
        for q in up {
            // h' = 4z³, and 1/(4p³) = −p/4
            let h = |z: C| C::real(1.0) + z.powi(4);
            let e = 1e-6;
            let slope = (h(q + C::real(e)) - h(q - C::real(e))).scale(1.0 / (2.0 * e));
            let cubed = C::real(4.0) * q.powi(3);
            assert!((slope - cubed).abs() < 1e-7);
            assert!((cubed.recip() + q.scale(0.25)).abs() < 1e-12);
            // Im p = √2/2
            assert!((q.im - 2.0_f64.sqrt() / 2.0).abs() < 1e-12);
        }
    }

    #[test]
    fn the_windows_hold_what_they_show() {
        let p = plane();
        for z in contour(model::ARC_RADII[0]) {
            assert!(p.contains(z));
        }
        let g = graph();
        // the graph sits above the contour's plane, both inside the safe area
        assert!(g.top() <= 3.85 && g.bottom() > p.frame.top());
        assert!(p.frame.bottom() >= -3.85 && g.left() >= -6.96 && p.frame.left() >= -6.96);
    }

    #[test]
    fn the_rectangle_is_length_times_max_and_holds_the_size() {
        let u = unrolled();
        let most = |power: i32, r: f64| {
            (0..=400)
                .map(|k| model::arc_size(power, C::polar(r, PI * f64::from(k) / 400.0)))
                .fold(0.0, f64::max)
        };
        for r in model::ARC_RADII {
            // the bump's size along the arc stays under the max and touches it at the top
            assert!(most(2, r) <= model::arc_max(2, r) * (1.0 + 1e-12), "R = {r}");
            assert!((model::arc_size(2, C::polar(r, PI / 2.0)) - model::arc_max(2, r)).abs() < 1e-12);
            // length × max is the bound the readout shows
            assert!((PI * r * model::arc_max(2, r) - arc_bound(2, r)).abs() < 1e-12);
            // every rectangle fits the unrolled window
            assert!(PI * r <= u.x.1 && model::arc_max(2, r) <= u.y.1);
        }
        // 13_10's max: |1 + z⁴| ≥ R⁴ − 1, reached where z⁴ = −R⁴
        for r in model::QUARTIC_ARC_RADII {
            assert!(most(4, r) <= model::arc_max(4, r) * (1.0 + 1e-12), "R = {r}");
            assert!((model::arc_size(4, C::polar(r, PI / 4.0)) - model::arc_max(4, r)).abs() < 1e-12);
        }
    }

    #[test]
    fn the_unrolled_window_sits_in_the_column() {
        let u = unrolled();
        let p = plane();
        // right of the plane, inside the safe area, between the |z²| row and the bound
        assert!(u.left() > p.frame.right() && u.right() <= 6.96);
        assert!(u.top() < ROW_Y - 0.5 && u.bottom() > BOUND_Y + 0.5);
        // the readouts stay clear of every rectangle: right of the first, above the rest
        let first = model::ARC_RADII[0];
        assert!(u.p(PI * first, 0.0)[0] < READ_X - 1.0);
        for &r in &model::ARC_RADII[1..] {
            assert!(u.p(0.0, model::arc_max(2, r))[1] < READ_Y[1] - 0.3, "R = {r}");
        }
    }

    /// What the walk's names have to stay clear of, in plane units. The half sizes of the two
    /// names and the ring are measured on the 720p pictures of `complex-combined-picture-frames02`
    /// (frames 88 and 100): the walk's "R" is 0.08 × 0.085, "i" is 0.021 × 0.08, the ring 0.107.
    const R_GLYPH: (f64, f64) = (0.08, 0.085);
    const I_GLYPH: (f64, f64) = (0.021, 0.08);
    const MINUS_I_GLYPH: (f64, f64) = (0.085, 0.08);
    const RING: f64 = 0.107;
    /// The segment's arrow, measured on frame 130: its head starts here and reaches this far off
    /// the line.
    const ARROW_HEAD_FROM: f64 = 0.55;
    const ARROW_HEAD_HALF: f64 = 0.145;

    /// A stroke's half width in plane units: 0.01 scene units wide per unit of `width`.
    fn half_line(p: &Plane, width: f64) -> f64 {
        width * 0.01 / 2.0 / p.unit()
    }

    /// Every angle of the walk, a fifth of a degree apart.
    fn walk_angles() -> impl Iterator<Item = f64> {
        (0..=900).map(|k| PI * f64::from(k) / 900.0)
    }

    #[test]
    fn the_walks_r_is_never_shown_across_the_segment() {
        let p = plane();
        let r = model::ARC_RADII[0];
        let (mut crossed, segment) = (false, half_line(&p, 5.0));
        for th in walk_angles() {
            let z = r_name_z(r, th);
            // it does cross the line on its way up, so the wait is doing something
            let touches = z.im.abs() < R_GLYPH.1 + segment + 0.03;
            crossed |= touches;
            if touches {
                assert!(r_name_opacity(th) < 1e-12, "shown across the segment at {th}");
            }
            // nor does its backing notch the loop the moment it can show at all
            if r_name_opacity(th) > 0.0 {
                assert!(z.im - R_BACK / p.unit() > segment, "backing on the segment at {th}");
            }
        }
        assert!(crossed);
        // it is all there well before it passes the pole, and stays to the end of the walk
        assert!(r_name_opacity(0.0) < 1e-12 && r_name_opacity(R_NAME_SHOWN[1]) > 1.0 - 1e-12);
        assert!(walk_angles().all(|th| th < R_NAME_SHOWN[1] || r_name_opacity(th) > 1.0 - 1e-12));
        const { assert!(R_NAME_SHOWN[0] < R_NAME_SHOWN[1] && R_NAME_SHOWN[1] < PI / 4.0) };
    }

    #[test]
    fn the_walks_r_and_its_backing_keep_off_everything_else() {
        let p = plane();
        let r = model::ARC_RADII[0];
        let back = R_BACK / p.unit();
        // the backing covers the letter, so the axis cannot show through it
        assert!(R_GLYPH.0.hypot(R_GLYPH.1) < back);
        let ring_back = 0.13 / p.unit();
        let i_at = C::new(POLE_NAME_X, 1.0);
        for th in walk_angles().filter(|&th| r_name_opacity(th) > 0.0) {
            let z = r_name_z(r, th);
            // off the spoke, measured square to it, whichever way the spoke points
            let off_spoke = z.re * th.sin() - z.im * th.cos();
            assert!(off_spoke - back - half_line(&p, 2.5) > 0.1, "spoke at {th}");
            // off the pole and its name
            assert!((z - C::new(0.0, 1.0)).abs() - back - ring_back > 0.15, "pole at {th}");
            assert!((z - i_at).abs() - back - I_BACK / p.unit() > 0.15, "i at {th}");
            // well inside the half circle and its arrow
            assert!(z.abs() + back < r - ARROW_HEAD_HALF - 0.2, "half circle at {th}");
            // off the segment's arrow
            let over_arrow = z.re + back > 0.2 && z.re - back < 0.8;
            assert!(!over_arrow || z.im - back > ARROW_HEAD_HALF + 0.1, "arrow at {th}");
        }
    }

    #[test]
    fn the_poles_name_keeps_its_backing_off_the_axis_and_the_ring() {
        let p = plane();
        let back = I_BACK / p.unit();
        assert!(I_GLYPH.0.hypot(I_GLYPH.1) < back, "the backing covers the letter");
        assert!(POLE_NAME_X + back + half_line(&p, 2.0) < -0.1, "and leaves the axis");
        assert!(-POLE_NAME_X - back > RING + 0.05, "and does not cut the ring");
        // the spoke does reach both, which is why they go on top of it
        let at_height_one = |th: f64| th.cos() / th.sin();
        assert!(walk_angles().any(|th| th > 0.1 && at_height_one(th).abs() < RING));
        assert!(walk_angles().any(|th| th > 0.1 && (at_height_one(th) - POLE_NAME_X).abs() < I_GLYPH.0));
    }

    #[test]
    fn the_poles_names_stay_clear_as_they_close_in() {
        let p = plane();
        let first = model::ARC_RADII[0];
        // the widest the segment gets: lit, as it is just after the poles are nearest
        let segment = half_line(&p, 8.0);
        let last = model::ARC_RADII[model::ARC_RADII.len() - 1];
        for k in 0..=200 {
            let at = first + (last - first) * f64::from(k) / 200.0;
            let h = pole_height(first, at);
            // "i" and its ring stay above the segment, "−i" and its ring below it
            assert!(h - I_GLYPH.1 - segment > 0.05, "i at R = {at}");
            assert!(h - RING - segment > 0.04, "ring at R = {at}");
            assert!(h - MINUS_I_GLYPH.1 - segment > 0.05, "−i at R = {at}");
        }
        // "−i" is written left of the arrow's head, at every height
        const { assert!(MINUS_POLE_NAME_X + MINUS_I_GLYPH.0 < ARROW_HEAD_FROM - 0.05) };
    }

    #[test]
    fn the_spoke_passes_behind_the_pole_and_both_stay_on_screen() {
        let source = include_str!("real_integral.rs");
        let code = &source[..source.find("#[cfg(test)]\nmod tests").unwrap()];
        let from = code.find("say(&kit, stage, \"13_3\")").unwrap();
        let three = &code[from..from + code[from..].find("say(&kit, stage, \"13_4\")").unwrap()];
        let order = [
            "let r_back = dot(stage, r_name_at(&p, r, 0.0), R_BACK, ground());",
            "stage.add_to_scene(spoke)?;",
            "for m in [arrows[0], i_back, inside[0], inside[1], pole_names[0]] {",
            "fade_in_all(stage, &[zdot, spoke, zsq, u_axis, along], 0.4)?;",
            "stage.add_to_scene(r_back)?;",
            "stage.add_to_scene(r_name)?;",
            "a.move_to(r_back, r_name_at(&p, r, th), ORIGIN);",
            "a.set_opacity(r_name, shown, true);",
            "a.set_opacity(r_back, shown, true);",
            "line.wait_for(stage, \"so the bump\")?;",
            "line.wait_sentence(stage, 2)?;",
            "fade_out_all(stage, &[zdot, spoke, r_name, r_back, i_back], 0.3)?;",
            "let rect = stage",
        ];
        let at: Vec<usize> = order
            .iter()
            .map(|s| three.find(s).unwrap_or_else(|| panic!("{s}")))
            .collect();
        assert!(at.windows(2).all(|w| w[0] < w[1]), "{at:?}");
        // the poles and their names outlast the radius: they are still there as R grows
        assert!(three.contains("a.move_to(pole_names[0], p.p(C::new(POLE_NAME_X, k)), ORIGIN);"));
        let next = &code[from + three.len()..];
        assert!(next.contains("gone.extend(inside);") && next.contains("gone.extend(pole_names);"));
    }

    #[test]
    fn the_sum_of_the_upper_poles_is_i_root_two() {
        let up: Vec<C> = model::quartic_roots().into_iter().filter(|q| q.im > 0.0).collect();
        let s = up[0] + up[1];
        assert!(s.re.abs() < 1e-12 && (s.im - 2.0_f64.sqrt()).abs() < 1e-12);
    }
}
