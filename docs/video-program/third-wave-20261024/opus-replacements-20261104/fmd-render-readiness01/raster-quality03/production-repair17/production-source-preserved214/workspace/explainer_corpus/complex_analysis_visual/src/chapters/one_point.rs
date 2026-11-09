//! 07_one_point: what a derivative at one point does and does not buy
//! (storyboard 07_1–07_6). Squaring at 0 (derivative zero) doubles the angle
//! between two lines; a nonzero derivative keeps angles. Then |z|², which
//! has a complex derivative at 0 and nowhere else: the ratio there is h̄,
//! which shrinks to 0, while at 1 the two directions disagree (2 against 0).
//! A derivative at one point survives multiplying by any bounded function,
//! so it forces nothing: a jagged g, with no limit at 0, is painted on the
//! plane, and the ratio of g·|z|² from a whole disk of steps h is squeezed
//! inside a disk of radius |h| as the steps shrink. The chapter ends on the
//! condition every later
//! claim carries: a complex derivative at every point of an open region
//! (holomorphic on U), drawn as an open blob with a small disk around each
//! point.

use std::f64::consts::{FRAC_PI_2, TAU};

use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{
    Painter, Plane, axis_names, carded, carded_lines, derivative, dot, fact, fade_in_all, fade_out_all, faded_line,
    flat, group, hero, input, line_style, names_now, output, precisely_now, predict, reading_window, rgb, say,
    tween_linear, value, wait_until, warn, zero_mark,
};
use crate::model::{self, C, Func, ZERO};

pub const MAP_TEX: &str = r"z \mapsto z^2";
pub const DERIV_TEX: &str = r"f'(z) = 2z, \quad f'(0) = 0";
pub const KEPT: &str = "derivative ≠ 0: angles kept (conformal)";
pub const MULTIPLIED: &str = "derivative = 0: a critical point, angles multiplied";
pub const CRITICAL: &str =
    "if the first m − 1 derivatives vanish at a point and the m-th does not, angles there are multiplied by m";
pub const ABS_TEX: &str = r"f(z) = |z|^2 = x^2 + y^2";
pub const ONE_HUE: &str = "real outputs: one hue, bands for size";
pub const ASK: [&str; 1] = ["derivative at 0?"];
pub const RATIO_TEX: &str = r"\frac{|h|^2}{h} = \bar h";
pub const RATIO: &str = "the ratio";
pub const SETTLES: &str = "ratio → ";
pub const RIGHT_UP: [&str; 2] = ["right: → ", "up: → "];
pub const ONLY_HERE: &str = "a complex derivative at 0 only";
pub const SQUEEZE_TEX: &str = r"\frac{|g(h)|\,|h|^2}{|h|} \le M\,|h| \to 0";
pub const PRODUCT: &str = "|z|² × jagged g, |g| ≤ 1 = M: derivative 0 at 0";
/// Which line of 07_1 is which, each on its own ray.
pub const LINE_NAMES: [&str; 2] = ["real line", "imaginary line"];
pub const INSIDE_RADIUS: &str = "inside radius";
pub const HOLOMORPHIC: [&str; 3] = [
    "holomorphic on U:",
    "a complex derivative at every point",
    "of the open region U",
];
pub const LONE: &str = "|z|²: one point, no disk";
pub const FROM_HERE: &str = "from here on: holomorphic on an open region";

/// Where 07_2 shows a nonzero derivative keeping a right angle (2z₁ ≠ 0),
/// clear of 07_1's angle labels.
pub const Z1: C = model::CONFORMAL_POINT;

/// The two planes of 07_1–07_2: z and z², each [−1.3, 1.3]².
pub fn planes() -> (Plane, Plane) {
    (
        Plane::new(v(-3.4, -0.15), 5.0, (-1.3, 1.3), (-1.3, 1.3)),
        Plane::new(v(3.4, -0.15), 5.0, (-1.3, 1.3), (-1.3, 1.3)),
    )
}

/// The painted plane of 07_3–07_5, [−2, 2]².
pub fn painted() -> Plane {
    Plane::new(v(-2.8, -0.25), 5.4, (-2.0, 2.0), (-2.0, 2.0))
}

/// The ratio's inset in 07_4, [−0.6, 0.6]².
pub fn inset() -> Plane {
    Plane::new(v(4.4, -0.9), 2.6, (-0.6, 0.6), (-0.6, 0.6))
}

/// How far the two lines of 07_1 reach from 0.
pub const REACH: f64 = 1.1;
/// The radius of the angle mark in the input; its image has radius `ARC_R²`.
pub const ARC_R: f64 = 0.5;
/// Where the map arrow of 07_1 starts and stops, between the planes. Its tail starts past the
/// left plane's "Re", and the map's name stands over its middle.
const MAP_ARROW: (f64, f64) = (-0.42, 0.6);

/// The arc `r e^{iθ}`, θ from 0 to `to`.
pub fn arc(r: f64, to: f64) -> Vec<C> {
    let n = 60;
    (0..=n).map(|k| C::polar(r, to * f64::from(k) / f64::from(n))).collect()
}

/// h along three approaches at `t` (1 far, 0 at 0): from the right, from
/// above, and along a spiral.
pub fn approach(kind: usize, t: f64) -> C {
    let r = 0.5 * t.max(1e-4);
    match kind {
        0 => C::real(r),
        1 => C::new(0.0, r),
        _ => C::polar(r, 0.9 + TAU * 1.25 * t),
    }
}

/// The ratio (f(0 + h) − f(0))/h for f = |z|², which is exactly h̄.
pub fn ratio_at0(h: C) -> C {
    C::real(h.norm_sqr()) / h
}

/// Polar cells of [`jagged`]: this many rings per doubling of |z|, this many
/// sectors round, so a cell keeps its shape at every scale.
const RINGS: f64 = 2.0;
const SECTORS: f64 = 16.0;

/// The ring and sector of `z` ≠ 0.
fn jag_cell(z: C) -> (i64, i64) {
    let i = (z.abs().log2() * RINGS).floor() as i64;
    let j = (z.arg().rem_euclid(TAU) / TAU * SECTORS).floor() as i64;
    (i, j)
}

/// The value of [`jagged`] on a cell: size in [0.4, 1), direction anywhere,
/// scrambled from the cell's indices so neighbours are unrelated.
fn jag_value(i: i64, j: i64) -> C {
    let x = (i as u64).wrapping_mul(73_856_093) ^ (j as u64).wrapping_mul(19_349_663);
    let x = x.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let x = (x ^ (x >> 29)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    let x = x ^ (x >> 32);
    let unit = |bits: u64| bits as f64 / 4_294_967_296.0;
    C::polar(0.4 + 0.6 * unit(x & 0xFFFF_FFFF), TAU * unit(x >> 32))
}

/// A jagged bounded function on the plane: |g| < 1, piecewise constant, with
/// jumps at every scale, so it has no limit at 0. Its value at 0 cannot
/// matter (the product vanishes there); here it is 0.
pub fn jagged(z: C) -> C {
    if z.abs() == 0.0 {
        return ZERO;
    }
    let (i, j) = jag_cell(z);
    jag_value(i, j)
}

/// The ratio at 0 of the product F = g·|z|²: (F(h) − F(0))/h, with F(0) = 0.
pub fn product_ratio(h: C) -> C {
    jagged(h).scale(h.norm_sqr()) / h
}

/// How many steps h the squeeze of 07_5 watches.
pub const SQUEEZE_SAMPLES: usize = 90;

/// The `k`-th step h in the disk of radius `rho`: a sunflower spiral, even
/// over the disk, the same pattern at every radius.
pub fn squeeze_sample(k: usize, rho: f64) -> C {
    let golden = std::f64::consts::PI * (3.0 - 5.0_f64.sqrt());
    C::polar(
        rho * ((k as f64 + 0.5) / SQUEEZE_SAMPLES as f64).sqrt(),
        golden * k as f64,
    )
}

/// The circle |z| = `r` in plane `p`, closed.
fn ring(p: &Plane, r: f64) -> Vec<Vec3> {
    (0..=96).map(|k| p.p(C::polar(r, TAU * f64::from(k) / 96.0))).collect()
}

/// The jagged g drawn out to this radius, in from this one (the cells nearer
/// 0 are smaller than the steps' disk ever gets).
const MOSAIC_REACH: f64 = 1.9;
const MOSAIC_CORE: f64 = 0.03;

/// The jagged g painted on `p`: each polar cell in the hue of g's direction,
/// brighter for a larger size, as one group.
fn jagged_mosaic(stage: &mut Stage<'_>, p: &Plane) -> fmn::Result<Mob> {
    let first = (MOSAIC_CORE.log2() * RINGS).floor() as i64;
    let last = (MOSAIC_REACH.log2() * RINGS).floor() as i64;
    let mut cells = Vec::new();
    for i in first..=last {
        let r0 = 2.0_f64.powf(i as f64 / RINGS);
        let r1 = 2.0_f64.powf((i + 1) as f64 / RINGS).min(MOSAIC_REACH);
        for j in 0..SECTORS as i64 {
            let (a0, a1) = (TAU * j as f64 / SECTORS, TAU * (j + 1) as f64 / SECTORS);
            let at = |r: f64, k: i32| p.p(C::polar(r, a0 + (a1 - a0) * f64::from(k) / 4.0));
            let mut pts: Vec<Vec3> = (0..=4).map(|k| at(r1, k)).collect();
            pts.extend((0..=4).rev().map(|k| at(r0, k)));
            let g = jag_value(i, j);
            let style = flat(rgb(model::wheel(g.arg())), 0.25 + 0.65 * g.abs());
            cells.push(stage.arena_mut().add(Polygon::new(pts).style(style).build()));
        }
    }
    group(stage, &cells)
}

/// The open region U of 07_6 (scene coordinates): a smooth blob.
pub fn blob_radius(theta: f64) -> f64 {
    2.0 + 0.35 * (3.0 * theta + 0.4).cos() + 0.25 * (2.0 * theta).sin()
}

pub const BLOB_CENTRE: [f64; 2] = [-2.4, -0.3];

pub fn blob(n: usize) -> Vec<Vec3> {
    (0..n)
        .map(|k| {
            let th = TAU * k as f64 / n as f64;
            let r = blob_radius(th);
            v(BLOB_CENTRE[0] + r * th.cos(), BLOB_CENTRE[1] + r * th.sin())
        })
        .collect()
}

/// Three points inside U, as offsets from its centre.
pub const INSIDE: [[f64; 2]; 3] = [[-0.9, 0.6], [0.8, 0.4], [0.1, -1.1]];

/// The small disk at each point: 0.8 of its distance to U's edge.
pub fn disk_radius(p: [f64; 2]) -> f64 {
    let at = v(BLOB_CENTRE[0] + p[0], BLOB_CENTRE[1] + p[1]);
    let edge = blob(720)
        .iter()
        .map(|q| ((q[0] - at[0]).powi(2) + (q[1] - at[1]).powi(2)).sqrt())
        .fold(f64::INFINITY, f64::min);
    0.8 * edge
}

pub struct OnePoint {
    pub kit: Kit,
}

impl SceneConstruct for OnePoint {
    fn name(&self) -> &str {
        "07_one_point"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (inp, out) = planes();

        // 07_1: z² at 0: two lines at a right angle come out as opposite rays
        let line = say(&kit, stage, "07_1")?;
        let mut frame = vec![inp.border(stage, input())?, out.border(stage, output())?];
        frame.extend(inp.axes(stage)?);
        frame.extend(out.axes(stage)?);
        frame.extend(axis_names(&kit, stage, &inp)?);
        frame.extend(axis_names(&kit, stage, &out)?);
        let map_arrow = stage.arena_mut().add(
            Arrow::new(v(MAP_ARROW.0, -0.15), v(MAP_ARROW.1, -0.15))
                .thickness(4.0)
                .color(Palette::dim())
                .buff(0.0)
                .build()?,
        );
        let map_at = v((MAP_ARROW.0 + MAP_ARROW.1) / 2.0, 0.35);
        let map = kit.tex(stage, MAP_TEX, 28.0, Palette::ink(), &[], map_at)?;
        frame.extend([map_arrow, map]);
        fade_in_all(stage, &frame, 0.5)?;
        line.wait_sentence(stage, 1)?;
        let deriv = kit.tex(stage, DERIV_TEX, 32.0, derivative(), &[], v(0.0, 3.3))?;
        fade_in_all(stage, &[deriv], 0.4)?;
        line.wait_sentence(stage, 2)?;
        // the lines grow out from 0 both ways; their images grow as r²
        let grow = stage.add_value_tracker(0.0);
        let tiny = 1e-3;
        // one colour per line and its ray, so the two rays read as two and each has its source
        let in_lines = [
            inp.path(stage, &[C::real(-tiny), C::real(tiny)], input(), 5.0)?,
            inp.path(stage, &[C::new(0.0, -tiny), C::new(0.0, tiny)], output(), 5.0)?,
        ];
        let out_rays = [
            out.path(stage, &[ZERO, C::real(tiny)], input(), 5.0)?,
            out.path(stage, &[ZERO, C::real(-tiny)], output(), 5.0)?,
        ];
        let line_names = [
            kit.text(stage, LINE_NAMES[0], 18.0, input(), inp.p(C::new(0.75, -0.16)))?,
            kit.text(stage, LINE_NAMES[1], 18.0, output(), inp.p(C::new(0.4, 1.12)))?,
            kit.text(stage, LINE_NAMES[0], 18.0, input(), out.p(C::new(0.8, -0.16)))?,
            kit.text(stage, LINE_NAMES[1], 18.0, output(), out.p(C::new(-0.72, -0.16)))?,
        ];
        fade_in_all(
            stage,
            &[
                in_lines[0],
                in_lines[1],
                out_rays[0],
                out_rays[1],
                line_names[0],
                line_names[1],
                line_names[2],
                line_names[3],
            ],
            0.2,
        )?;
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        stage.add_updater(
            anchor,
            move |s, _| {
                let r = (REACH * value(s, grow)).max(tiny);
                let _ = s.set_points(
                    in_lines[0],
                    polyline(vec![inp.p(C::real(-r)), inp.p(C::real(r))]).build().points(),
                );
                let _ = s.set_points(
                    in_lines[1],
                    polyline(vec![inp.p(C::new(0.0, -r)), inp.p(C::new(0.0, r))])
                        .build()
                        .points(),
                );
                // the real line folds onto the positive ray, the imaginary line onto the negative one
                let _ = s.set_points(
                    out_rays[0],
                    polyline(vec![out.p(ZERO), out.p(C::real(r * r))]).build().points(),
                );
                let _ = s.set_points(
                    out_rays[1],
                    polyline(vec![out.p(ZERO), out.p(C::real(-r * r))]).build().points(),
                );
            },
            true,
        )?;
        let rt = line.until(stage, "The angle has doubled", 2.5)?;
        stage.play(tween_linear(grow, 1.0, rt)?)?;
        stage.clear_updaters(anchor, true);
        // the angle marks sweep together: θ in, 2θ out
        let sweep = stage.add_value_tracker(0.0);
        let in_arc = inp.path(stage, &arc(ARC_R, 1e-3), hero(), 4.0)?;
        let out_arc = out.path(stage, &arc(ARC_R * ARC_R, 2e-3), hero(), 4.0)?;
        fade_in_all(stage, &[in_arc, out_arc], 0.2)?;
        let anchor2 = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor2)?;
        stage.add_updater(
            anchor2,
            move |s, _| {
                let th = value(s, sweep).max(1e-3);
                let a: Vec<Vec3> = arc(ARC_R, th).iter().map(|&z| inp.p(z)).collect();
                let b: Vec<Vec3> = arc(ARC_R * ARC_R, 2.0 * th).iter().map(|&z| out.p(z)).collect();
                let _ = s.set_points(in_arc, polyline(a).build().points());
                let _ = s.set_points(out_arc, polyline(b).build().points());
            },
            true,
        )?;
        stage.play(tween_linear(sweep, FRAC_PI_2, 1.6)?)?;
        stage.clear_updaters(anchor2, true);
        let angles = [
            kit.text(
                stage,
                &format!("{}°", fact("critical.angle_in")?),
                26.0,
                hero(),
                inp.p(C::new(0.62, 0.62)),
            )?,
            kit.text(
                stage,
                &format!("{}°", fact("critical.angle_out")?),
                26.0,
                hero(),
                out.p(C::new(0.42, 0.42)),
            )?,
        ];
        fade_in_all(stage, &angles, 0.4)?;

        // 07_2: a nonzero derivative keeps angles; at a critical point they multiply
        kit.hold(stage)?;
        let line = say(&kit, stage, "07_2")?;
        fade_out_all(stage, &[deriv], 0.3)?;
        // a small cross at z₁ = −0.7 + 0.6i, where the derivative is 2z₁ ≠ 0
        let z1 = Z1;
        let arm = 0.22;
        let seg = |d: C| -> Vec<C> {
            (0..=20)
                .map(|k| z1 + d.scale(arm * (f64::from(k) / 10.0 - 1.0)))
                .collect()
        };
        let along = [seg(C::real(1.0)), seg(C::new(0.0, 1.0))];
        let cross_in = [
            inp.path(stage, &along[0], input(), 4.0)?,
            inp.path(stage, &along[1], input(), 4.0)?,
        ];
        let sq: Vec<Vec<C>> = along.iter().map(|p| p.iter().map(|&z| z * z).collect()).collect();
        let cross_out = [
            out.path(stage, &sq[0], output(), 4.0)?,
            out.path(stage, &sq[1], output(), 4.0)?,
        ];
        let kept_angle = format!("{}°", fact("critical.angle_kept")?);
        let kept_in = kit.text(stage, &kept_angle, 22.0, Palette::ink(), inp.p(z1 + C::new(0.3, 0.3)))?;
        let kept_out = kit.text(
            stage,
            &kept_angle,
            22.0,
            Palette::ink(),
            out.p(z1 * z1 + C::new(0.35, 0.3)),
        )?;
        let kept = kit.text(stage, KEPT, 26.0, Palette::ink(), v(0.0, 3.3))?;
        fade_in_all(stage, &[cross_in[0], cross_in[1]], 0.3)?;
        fade_in_all(stage, &[cross_out[0], cross_out[1], kept_in, kept_out, kept], 0.5)?;
        line.wait_sentence(stage, 1)?;
        let names = names_now(&kit, stage, &["conformal map", "critical point"])?;
        line.wait_sentence(stage, 2)?;
        let multiplied = kit.text(stage, MULTIPLIED, 26.0, hero(), v(0.0, 3.3))?;
        fade_out_all(stage, &[kept, names], 0.3)?;
        fade_in_all(stage, &[multiplied], 0.4)?;
        let exact = precisely_now(&kit, stage, CRITICAL)?;
        wait_until(stage, line.end())?;

        // 07_3: |z|²: at 0, a complex derivative?
        reading_window(&kit, stage, &line)?;
        let line = say(&kit, stage, "07_3")?;
        let mut gone = frame;
        gone.extend(in_lines);
        gone.extend(out_rays);
        gone.extend(line_names);
        gone.extend([in_arc, out_arc, multiplied, exact, kept_in, kept_out]);
        gone.extend(angles);
        gone.extend(cross_in);
        gone.extend(cross_out);
        fade_out_all(stage, &gone, 0.5)?;
        let p = painted();
        let right = 4.2;
        line.wait_sentence(stage, 1)?;
        let painter = Painter::build(stage, Some(Func::AbsSquare), p, 1.0)?;
        let border = p.border(stage, Palette::faint())?;
        let p_names = axis_names(&kit, stage, &p)?;
        let formula = kit.tex(stage, ABS_TEX, 32.0, Palette::ink(), &[], v(right, 2.6))?;
        let hue = kit.text(stage, ONE_HUE, 20.0, Palette::dim(), v(right, 1.95))?;
        painter.show(stage, 1.0)?;
        fade_in_all(stage, &[border, formula, hue], 0.4)?;
        fade_in_all(stage, &p_names, 0.2)?;
        let o = p.p(ZERO);
        let mut origin = zero_mark(stage, o).to_vec();
        origin.extend(carded(
            &kit,
            stage,
            "0",
            22.0,
            Palette::ink(),
            v(o[0] + 0.45, o[1] - 0.38),
            Palette::faint(),
        )?);
        fade_in_all(stage, &origin, 0.3)?;
        let asked = predict(&kit, stage, &line, &ASK, v(right, 0.6))?;

        // 07_4: it does: the ratio is h̄, which shrinks to 0 from every direction
        let line = say(&kit, stage, "07_4")?;
        fade_out_all(stage, &asked, 0.3)?;
        fade_out_all(stage, &[hue], 0.2)?;
        line.wait_sentence(stage, 1)?;
        let ratio_tex = kit.tex(stage, RATIO_TEX, 34.0, derivative(), &[], v(right, 1.7))?;
        fade_in_all(stage, &[ratio_tex], 0.4)?;
        let ins = inset();
        let ins_frame = ins.border(stage, derivative())?;
        let ins_name = kit.text(
            stage,
            RATIO,
            22.0,
            derivative(),
            v(ins.frame.center[0], ins.frame.top() + 0.25),
        )?;
        let mut ins_axes = ins.axes(stage)?;
        let limit = dot(stage, ins.p(ZERO), 0.06, Palette::ink());
        ins_axes.extend([ins_frame, ins_name, limit]);
        fade_in_all(stage, &ins_axes, 0.3)?;
        line.wait_sentence(stage, 2)?;
        let mut trails = Vec::new();
        for kind in 0..3usize {
            let t = stage.add_value_tracker(1.0);
            let h_arrow = stage.arena_mut().add(
                polyline(vec![p.p(ZERO), p.p(approach(kind, 1.0))])
                    .style(line_style(input(), 4.0))
                    .build(),
            );
            let q_dot = dot(stage, ins.p(ratio_at0(approach(kind, 1.0))), 0.06, derivative());
            let trail = stage.arena_mut().add(
                polyline(vec![ins.p(ratio_at0(approach(kind, 1.0))); 2])
                    .style(faded_line(derivative(), 3.0, 0.8))
                    .build(),
            );
            let anchor = stage.arena_mut().add(VMobject::new());
            stage.add_to_scene(anchor)?;
            fade_in_all(stage, &[h_arrow, q_dot, trail], 0.3)?;
            stage.add_updater(
                anchor,
                move |s, _| {
                    let tv = value(s, t);
                    let hz = approach(kind, tv);
                    let _ = s.set_points(h_arrow, polyline(vec![p.p(ZERO), p.p(hz)]).build().points());
                    s.move_to(q_dot, ins.p(ratio_at0(hz)), ORIGIN);
                    let n = 80;
                    let pts: Vec<Vec3> = (0..=n)
                        .map(|j| {
                            ins.p(ratio_at0(approach(
                                kind,
                                1.0 - (1.0 - tv) * f64::from(j) / f64::from(n),
                            )))
                        })
                        .collect();
                    let _ = s.set_points(trail, polyline(pts).build().points());
                },
                true,
            )?;
            stage.play(tween_linear(t, 0.0, 2.0)?)?;
            stage.clear_updaters(anchor, true);
            fade_out_all(stage, &[h_arrow], 0.2)?;
            trails.extend([q_dot, trail]);
        }
        line.wait_sentence(stage, 3)?;
        let settled = kit.text(
            stage,
            &format!("{SETTLES}{}", fact("onepoint.at0")?),
            24.0,
            derivative(),
            v(ins.frame.center[0], ins.frame.bottom() - 0.35),
        )?;
        fade_in_all(stage, &[settled], 0.4)?;

        // 07_5: at 1 the directions disagree; one point buys almost nothing
        kit.hold(stage)?;
        let line = say(&kit, stage, "07_5")?;
        let mut gone = vec![ratio_tex, settled];
        gone.extend(trails);
        gone.extend(ins_axes);
        fade_out_all(stage, &gone, 0.4)?;
        let one = C::real(1.0);
        let at_one = dot(stage, p.p(one), 0.08, input());
        let o1 = p.p(one);
        let one_name = carded(
            &kit,
            stage,
            "1",
            22.0,
            Palette::ink(),
            v(o1[0], o1[1] - 0.4),
            Palette::faint(),
        )?;
        fade_in_all(stage, &[at_one, one_name[0], one_name[1]], 0.3)?;
        let at1: Vec<&str> = fact("onepoint.at1")?.split(',').collect();
        line.wait_sentence(stage, 1)?;
        let step_right = stage.arena_mut().add(
            Arrow::new(p.p(one), p.p(one + C::real(0.6)))
                .thickness(4.0)
                .color(input())
                .buff(0.0)
                .build()?,
        );
        let r1 = kit.text(
            stage,
            &format!("{}{}", RIGHT_UP[0], at1[0]),
            28.0,
            Palette::ink(),
            v(right, 1.2),
        )?;
        fade_in_all(stage, &[step_right, r1], 0.4)?;
        line.wait_sentence(stage, 2)?;
        let step_up = stage.arena_mut().add(
            Arrow::new(p.p(one), p.p(one + C::new(0.0, 0.6)))
                .thickness(4.0)
                .color(input())
                .buff(0.0)
                .build()?,
        );
        let r2 = kit.text(
            stage,
            &format!("{}{}", RIGHT_UP[1], at1[1]),
            28.0,
            Palette::ink(),
            v(right, 0.6),
        )?;
        fade_in_all(stage, &[step_up, r2], 0.4)?;
        let cross = red_cross(stage, v(right, -0.3))?;
        fade_in_all(stage, &cross, 0.3)?;
        line.wait_sentence(stage, 3)?;
        let only = kit.text(stage, ONLY_HERE, 24.0, hero(), v(right, -1.2))?;
        fade_in_all(stage, &[only], 0.4)?;
        // the product with a jagged bounded g, in the plane: g painted, a
        // whole disk of steps h, and every ratio from them inside the disk of
        // radius |h| in the ratio's window; the disks shrink together
        line.wait_sentence(stage, 4)?;
        let mut gone = vec![
            at_one,
            one_name[0],
            one_name[1],
            step_right,
            step_up,
            r1,
            r2,
            only,
            formula,
        ];
        gone.extend(cross);
        gone.extend(origin);
        fade_out_all(stage, &gone, 0.4)?;
        painter.leave(stage, 0.5)?;
        let mosaic = jagged_mosaic(stage, &p)?;
        fade_in_all(stage, &[mosaic], 0.5)?;
        let squeeze = kit.tex(stage, SQUEEZE_TEX, 32.0, Palette::ink(), &[], v(right, 2.45))?;
        let product_says = kit.text(stage, PRODUCT, 20.0, output(), v(right, 1.55))?;
        fade_in_all(stage, &[squeeze, product_says], 0.4)?;
        let [rho_start, rho_end] = model::SQUEEZE_RADII;
        let rho = stage.add_value_tracker(rho_start);
        let steps_ring = stage.arena_mut().add(
            polyline(ring(&p, rho_start))
                .style(line_style(Palette::ink(), 3.0))
                .build(),
        );
        let mut window = ins.axes(stage)?;
        window.push(ins.border(stage, derivative())?);
        window.push(kit.text(
            stage,
            RATIO,
            22.0,
            derivative(),
            v(ins.frame.center[0], ins.frame.top() + 0.25),
        )?);
        let ratio_ring = stage.arena_mut().add(
            polyline(ring(&ins, rho_start))
                .style(line_style(derivative(), 3.0))
                .build(),
        );
        let mut steps = Vec::with_capacity(SQUEEZE_SAMPLES);
        let mut ratios = Vec::with_capacity(SQUEEZE_SAMPLES);
        for k in 0..SQUEEZE_SAMPLES {
            let h = squeeze_sample(k, rho_start);
            steps.push(dot(stage, p.p(h), 0.03, Palette::ink()));
            ratios.push(dot(stage, ins.p(product_ratio(h)), 0.035, output()));
        }
        let inside_y = ins.frame.bottom() - 0.32;
        let inside_says = kit.text_left(stage, INSIDE_RADIUS, 22.0, derivative(), v(ins.frame.left(), inside_y))?;
        let mut lit = vec![steps_ring, ratio_ring, inside_says];
        lit.extend(&window);
        lit.extend(&steps);
        lit.extend(&ratios);
        fade_in_all(stage, &lit, 0.4)?;
        let inside_x = ins.frame.left() + stage.get_width(inside_says) + 0.12;
        let inside_live = kit.live_value(
            stage,
            move |s| value(s, rho),
            2,
            false,
            22.0,
            derivative(),
            v(inside_x, inside_y),
            LEFT,
        )?;
        line.wait_for(stage, "and the product")?;
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        let (step_mobs, ratio_mobs) = (steps.clone(), ratios.clone());
        stage.add_updater(
            anchor,
            move |a, _| {
                let r = value(a, rho);
                let _ = a.set_points(steps_ring, polyline(ring(&p, r)).build().points());
                let _ = a.set_points(ratio_ring, polyline(ring(&ins, r)).build().points());
                for (k, (&step, &ratio)) in step_mobs.iter().zip(&ratio_mobs).enumerate() {
                    let h = squeeze_sample(k, r);
                    a.move_to(step, p.p(h), ORIGIN);
                    a.move_to(ratio, ins.p(product_ratio(h)), ORIGIN);
                }
            },
            true,
        )?;
        let rt = (line.end() - stage.scene().time().to_f64() - 0.2).max(1.5);
        stage.play(tween_linear(rho, rho_end, rt)?)?;
        stage.clear_updaters(anchor, true);
        stage.remove_from_scene(anchor);
        stage.clear_updaters(inside_live, true);
        let printed: Vec<&str> = fact("onepoint.squeeze.radii")?.split(',').collect();
        let Some(end_says) = printed.last() else {
            return Err(super::draw::fail("07_5 has no squeeze radii in the facts".into()));
        };
        let inside_end = kit.text_left(stage, end_says, 22.0, derivative(), v(inside_x, inside_y))?;
        stage.remove_from_scene(inside_live);
        stage.add_to_scene(inside_end)?;
        wait_until(stage, line.end())?;

        // 07_6: the condition from here on: holomorphic on an open region
        kit.hold(stage)?;
        let line = say(&kit, stage, "07_6")?;
        let mut gone = lit;
        gone.extend([mosaic, border, squeeze, product_says, inside_end]);
        gone.extend(p_names);
        fade_out_all(stage, &gone, 0.5)?;
        line.wait_sentence(stage, 1)?;
        let edge = blob(96);
        let fill = stage
            .arena_mut()
            .add(Polygon::new(edge.clone()).style(flat(hero(), 0.14)).build());
        // a dashed edge: the region is open, its edge not included
        let mut dashes = Vec::new();
        for k in (0..edge.len()).step_by(2) {
            let (a, b) = (edge[k], edge[(k + 1) % edge.len()]);
            dashes.push(
                stage
                    .arena_mut()
                    .add(polyline(vec![a, b]).style(line_style(hero(), 3.0)).build()),
            );
        }
        let rim = group(stage, &dashes)?;
        let u_name = kit.tex(
            stage,
            "U",
            34.0,
            hero(),
            &[],
            v(BLOB_CENTRE[0] - 1.55, BLOB_CENTRE[1] + 1.75),
        )?;
        fade_in_all(stage, &[fill, rim, u_name], 0.6)?;
        let grow = stage.add_value_tracker(0.0);
        let mut disks = Vec::new();
        let mut centres = Vec::new();
        for q in INSIDE {
            let at = v(BLOB_CENTRE[0] + q[0], BLOB_CENTRE[1] + q[1]);
            centres.push(dot(stage, at, 0.06, Palette::ink()));
            disks.push(
                stage.arena_mut().add(
                    Circle::new()
                        .radius(1e-3)
                        .style(line_style(derivative(), 3.0))
                        .build()
                        .moved_to(at),
                ),
            );
        }
        fade_in_all(stage, &centres, 0.3)?;
        fade_in_all(stage, &disks, 0.2)?;
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        let disk_mobs = disks.clone();
        stage.add_updater(
            anchor,
            move |s, _| {
                let share = value(s, grow).max(1e-3);
                for (k, q) in INSIDE.iter().enumerate() {
                    let at = v(BLOB_CENTRE[0] + q[0], BLOB_CENTRE[1] + q[1]);
                    let r = disk_radius(*q) * share;
                    let pts: Vec<Vec3> = (0..=72)
                        .map(|j| {
                            let th = TAU * f64::from(j) / 72.0;
                            v(at[0] + r * th.cos(), at[1] + r * th.sin())
                        })
                        .collect();
                    let _ = s.set_points(disk_mobs[k], polyline(pts).build().points());
                }
            },
            true,
        )?;
        stage.play(tween_linear(grow, 1.0, 1.6)?)?;
        stage.clear_updaters(anchor, true);
        let lone = dot(stage, v(2.0, -2.4), 0.07, warn());
        let lone_says = kit.text_left(stage, LONE, 20.0, Palette::ink(), v(2.2, -2.4))?;
        fade_in_all(stage, &[lone, lone_says], 0.4)?;
        line.wait_sentence(stage, 2)?;
        let definition = carded_lines(&kit, stage, &HOLOMORPHIC, 24.0, Palette::ink(), v(right, 1.2), hero())?;
        fade_in_all(stage, &definition, 0.5)?;
        let search = names_now(&kit, stage, &["holomorphic function", "open set"])?;
        line.wait_sentence(stage, 3)?;
        let from_here = kit.text(stage, FROM_HERE, 20.0, hero(), v(right, -0.6))?;
        fade_in_all(stage, &[from_here], 0.4)?;
        wait_until(stage, line.end())?;
        kit.music_breath(stage, 2.0)?;
        let mut all = vec![fill, rim, u_name, lone, lone_says, search, from_here];
        all.extend(centres);
        all.extend(disks);
        all.extend(definition);
        fade_out_all(stage, &all, 0.5)?;
        kit.finish(stage)
    }
}

/// A red cross: no complex derivative here.
fn red_cross(stage: &mut Stage<'_>, at: Vec3) -> fmn::Result<Vec<Mob>> {
    let d = 0.22;
    let a = polyline(vec![v(at[0] - d, at[1] - d), v(at[0] + d, at[1] + d)])
        .style(line_style(warn(), 7.0))
        .build();
    let b = polyline(vec![v(at[0] - d, at[1] + d), v(at[0] + d, at[1] - d)])
        .style(line_style(warn(), 7.0))
        .build();
    Ok(vec![stage.arena_mut().add(a), stage.arena_mut().add(b)])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn squaring_doubles_the_angle_at_zero_and_keeps_it_elsewhere() {
        // the drawn arc's image is the arc of radius ARC_R² swept twice as far
        for (z, w) in arc(ARC_R, FRAC_PI_2).iter().zip(arc(ARC_R * ARC_R, 2.0 * FRAC_PI_2)) {
            assert!((*z * *z - w).abs() < 1e-12);
        }
        let f = crate::facts::facts().unwrap();
        assert_eq!(f.get("critical.angle_in"), Some("90"));
        assert_eq!(f.get("critical.angle_out"), Some("180"));
        assert_eq!(f.get("critical.angle_kept"), Some("90"));
        // at z₁ the images of the two arms still meet at a right angle, and
        // the drawn cross and its image stay in their windows
        let z1 = Z1;
        let (inp, out) = planes();
        for d in [C::real(0.22), C::real(-0.22), C::new(0.0, 0.22), C::new(0.0, -0.22)] {
            assert!(inp.contains(z1 + d) && out.contains((z1 + d) * (z1 + d)));
        }
        // clear of the 90° label at 0.62 + 0.62i
        assert!((z1 - C::new(0.62, 0.62)).abs() > 1.0);
        let d = 1e-6;
        let a = (C::real(d) + z1) * (C::real(d) + z1) - z1 * z1;
        let b = (C::new(0.0, d) + z1) * (C::new(0.0, d) + z1) - z1 * z1;
        assert!(((b / a).arg() - FRAC_PI_2).abs() < 1e-5);
    }

    #[test]
    fn the_ratio_at_zero_is_h_bar_and_shrinks_every_way() {
        for kind in 0..3 {
            for t in [1.0, 0.5, 0.1, 1e-3] {
                let h = approach(kind, t);
                assert!((ratio_at0(h) - h.conj()).abs() < 1e-12);
                assert!((ratio_at0(h).abs() - h.abs()).abs() < 1e-12);
            }
        }
        let f = crate::facts::facts().unwrap();
        assert_eq!(f.get("onepoint.at0"), Some("0.0000"));
        assert_eq!(f.get("onepoint.at1"), Some("2,0"));
    }

    #[test]
    fn the_jagged_multiplier_is_bounded_with_no_limit_at_zero() {
        // at every scale down to 2⁻³⁰ the values on one ring are far apart
        for k in 0..=60 {
            let r = 2.0_f64.powf(-0.25 - f64::from(k) / 2.0);
            let vals: Vec<C> = (0..16)
                .map(|s| jagged(C::polar(r, TAU * (f64::from(s) + 0.5) / 16.0)))
                .collect();
            assert!(vals.iter().all(|g| g.abs() < 1.0), "|g| < 1 near radius {r}");
            let spread = vals
                .iter()
                .flat_map(|a| vals.iter().map(move |b| (*a - *b).abs()))
                .fold(0.0, f64::max);
            assert!(spread > 1.0, "g settles near radius {r}");
        }
        // the printed claim on the screen
        assert!(PRODUCT.contains("|g| ≤ 1"));
    }

    #[test]
    fn every_ratio_of_the_product_is_inside_the_disk_of_radius_h() {
        let (ins, pl) = (inset(), painted());
        let [start, end] = model::SQUEEZE_RADII;
        assert!(start > end);
        for rho in [start, 0.3, 0.1, end] {
            for k in 0..SQUEEZE_SAMPLES {
                let h = squeeze_sample(k, rho);
                let ratio = product_ratio(h);
                assert!(h.abs() <= rho && pl.contains(h));
                // |g(h)| |h|² / |h| ≤ |h|: the squeeze, for every direction of h
                assert!(ratio.abs() <= h.abs() * (1.0 + 1e-12));
                assert!(ins.contains(ratio));
            }
        }
        // the ratio's whole disk fits its window at the start
        assert!(start < ins.frame.x.1 && start < ins.frame.y.1);
    }

    #[test]
    fn the_mosaic_paints_the_function_itself() {
        for i in -11..=1_i64 {
            for j in 0..SECTORS as i64 {
                let r = 2.0_f64.powf((i as f64 + 0.5) / RINGS);
                let z = C::polar(r, TAU * (j as f64 + 0.5) / SECTORS);
                assert_eq!(jag_cell(z), (i, j));
                assert!((jagged(z) - jag_value(i, j)).abs() == 0.0);
            }
        }
        let p = painted();
        assert!(MOSAIC_REACH < p.frame.x.1 && MOSAIC_REACH < p.frame.y.1);
        // the mosaic reaches in past the steps' last disk
        const { assert!(MOSAIC_CORE < model::SQUEEZE_RADII[1]) };
        // the row under the ratio's window stays above the lower third
        assert!(inset().frame.bottom() - 0.32 > -2.78);
    }

    #[test]
    fn every_small_disk_lies_inside_the_open_region() {
        let edge = blob(720);
        // the blob is star-shaped about its centre with radius between 1.4 and 2.6
        for th in (0..360).map(|d| f64::from(d).to_radians()) {
            let r = blob_radius(th);
            assert!(r > 1.4 && r < 2.6);
        }
        for q in INSIDE {
            let r = disk_radius(q);
            assert!(r > 0.2, "a visible disk at {q:?}");
            let at = [BLOB_CENTRE[0] + q[0], BLOB_CENTRE[1] + q[1]];
            let nearest = edge
                .iter()
                .map(|e| ((e[0] - at[0]).powi(2) + (e[1] - at[1]).powi(2)).sqrt())
                .fold(f64::INFINITY, f64::min);
            assert!(r < nearest);
            // and the point itself is inside: closer to the centre than the edge in its direction
            let th = q[1].atan2(q[0]);
            assert!(q[0].hypot(q[1]) < blob_radius(th));
        }
        // the region stays in the frame's safe area
        for e in &edge {
            assert!(e[0] > -6.96 && e[0] < 0.0 && e[1] > -3.85 && e[1] < 3.85);
        }
    }

    #[test]
    fn the_map_arrow_starts_clear_of_re_and_stops_short_of_the_right_plane() {
        let (inp, out) = planes();
        // "Re" stands 0.24 past the left plane; measured at 1280 × 720 on frame 10 of 07, it
        // ends 11 px past its middle, at 90 px to the scene unit
        let re_ends = inp.frame.right() + 0.24 + 11.0 / 90.0;
        assert!(MAP_ARROW.0 > re_ends + 0.1, "the tail runs into Re: {re_ends}");
        assert!(MAP_ARROW.1 < out.frame.left() - 0.2);
    }

    #[test]
    fn the_pictures_fit_and_do_not_collide() {
        let (inp, out) = planes();
        assert!(inp.frame.right() < -0.6 && out.frame.left() > 0.6);
        for pl in [inp, out, painted()] {
            assert!(pl.frame.top() <= 3.0 && pl.frame.bottom() >= -3.0);
            assert!(pl.frame.left() >= -6.96 && pl.frame.right() <= 6.96);
        }
        // the images of the drawn lines stay in the output window
        assert!(out.contains(C::real(REACH * REACH)) && out.contains(C::real(-REACH * REACH)));
        let ins = inset();
        assert!(painted().frame.right() < ins.frame.left());
        for kind in 0..3 {
            assert!(ins.contains(ratio_at0(approach(kind, 1.0))));
            assert!(painted().contains(approach(kind, 1.0)));
        }
    }
}
