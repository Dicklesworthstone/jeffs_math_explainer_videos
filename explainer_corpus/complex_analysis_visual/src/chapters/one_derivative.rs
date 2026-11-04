//! 06_one_derivative: the complex derivative as one ratio that every
//! direction of approach must agree on; why that makes every small square a
//! stretched, turned square (z² at 1 + i: × 2.83, an eighth of a turn, the
//! gap shrinking with the square); the Cauchy–Riemann equations from the two
//! directions; the local matrix as a stretch and a turn; and why a
//! reflection and a squash have no complex derivative anywhere (storyboard
//! 06_1–06_9). The pointwise claim is only this: at one point. What a
//! derivative on a whole open region forces comes in chapters 07 and 10.

use std::f64::consts::TAU;

use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{
    Plane, arrow_head_points, axis_names, derivative, dot, fact, fade_in_all, fade_out_all, faded_line, flat, hero,
    input, line_style, names_now, output, predict, say, shift_all, tween_linear, value, wait_until, warn,
};
// the map's name and the ratio's, as the chapters beside this one write them
use super::one_point::RATIO;
use super::two_planes::FN_TEX;
use crate::model::C;
#[cfg(test)]
use crate::model::square_gap_ratio;

pub const QUOTIENT_TEX: &str = r"\frac{f(z_0 + h) - f(z_0)}{h}";
pub const RIGHT_TEX: &str = r"u_x + i\,v_x";
pub const UP_TEX: &str = r"\frac{u_y + i\,v_y}{i} = v_y - i\,u_y";
pub const CR_TEX: &str = r"u_x = v_y \qquad v_x = -u_y";
pub const LABELS: [&str; 2] = ["step right:", "step up:"];
pub const STRETCH: &str = "stretch ";
pub const TURN: &str = ", turn ";
pub const GAP: &str = "largest gap ÷ image side: ";
pub const ASK: [&str; 1] = ["a derivative anywhere?"];
pub const RIGHT_UP: [&str; 2] = ["right: ", "up: "];
pub const STRETCH_TURN: &str = "a stretch and a turn: no squash, no reflection";
/// The squashed square is drawn moved down by i so it fits the window; a
/// translation changes no derivative.
pub const MOVED: &str = "image drawn moved down by i";
/// The right picture's zoom in 06_4, said on screen.
pub const ZOOMED: &str = "zoomed in about 2i";
pub const ZOOMED_WHY: &str = "to keep the image's size";
pub const CHANGE: &str = "change in the output";
pub const SETTLES_TEX: &str = r"\to\, f'(z_0)";
/// The three approaches of 06_2, as the voice lists them.
pub const FROM: [&str; 3] = ["from the right", "from above", "along a spiral"];
pub const TIMES_TEX: &str = r"\times\, f'(z_0)";
pub const A_STRETCH_AND_TURN: &str = "a stretch and a turn";
pub const IMAGE_NAME: &str = "the image";
pub const EXACT_NAME: &str = "exact stretch and turn";
pub const COORDS_TEX: [&str; 2] = [r"z = x + i\,y", r"f = u + i\,v"];
pub const RATE: &str = "is how fast u changes as x changes";
pub const AGREE: &str = "must agree";
pub const CR_PARTS: [&str; 2] = [r"u_x = v_y", r"v_x = -u_y"];
/// The local map's grid in symbols, read across: u_x, u_y, then v_x, v_y.
pub const JACOBIAN: [&str; 4] = ["u_x", "u_y", "v_x", "v_y"];
pub const CONJ_TEX: &str = r"x + i\,y \mapsto x - i\,y";
pub const SQUASH_TEX: &str = r"x + i\,y \mapsto x + 2i\,y";
pub const NO_DERIVATIVE: &str = "no complex derivative";

/// z₀ = 1 + i, where the chapter differentiates z².
pub const Z0: C = C::new(1.0, 1.0);

/// The planes: input around z₀, output around z₀² = 2i.
pub fn planes() -> (Plane, Plane) {
    (
        Plane::new(v(-3.6, -0.6), 4.6, (-0.2, 2.2), (-0.2, 2.2)),
        Plane::new(v(2.4, -0.6), 4.6, (-1.6, 1.6), (0.4, 3.6)),
    )
}

/// The edge of the square of side `side` centred at `c`, counterclockwise,
/// `n` points an edge.
pub fn square_edge(c: C, side: f64, n: usize) -> Vec<C> {
    let h = side / 2.0;
    let corners = [C::new(-h, -h), C::new(h, -h), C::new(h, h), C::new(-h, h)];
    let mut out = Vec::with_capacity(4 * n + 1);
    for k in 0..4 {
        let (a, b) = (corners[k], corners[(k + 1) % 4]);
        for j in 0..n {
            out.push(c + a + (b - a).scale(j as f64 / n as f64));
        }
    }
    out.push(c + corners[0]);
    out
}

/// The image of the side-`side` square at z₀ under z², rescaled by 1/side
/// about z₀² (the zoom of 06_4): it tends to 2z₀ times the unit square.
pub fn zoomed_image(side: f64) -> Vec<C> {
    square_edge(C::new(0.0, 0.0), 1.0, 40)
        .into_iter()
        .map(|u| {
            let z = Z0 + u.scale(side);
            (z * z - Z0 * Z0).scale(1.0 / side)
        })
        .collect()
}

/// What the zoomed right plane of 06_4 shows: the image of the side-`side`
/// square magnified by 0.5/side about z₀², so it keeps the side-0.5 size.
/// At side 0.5 it is the true image; as the side shrinks it closes on the
/// teal square z₀² + 2z₀(z − z₀) of [`linear_at`].
pub fn kept_size(side: f64) -> Vec<C> {
    let w0 = Z0 * Z0;
    zoomed_image(side).into_iter().map(|d| w0 + d.scale(0.5)).collect()
}

/// h along three approaches at `t` (1 far, 0 at z₀): from the right, from
/// above, and along a spiral.
pub fn approach(kind: usize, t: f64) -> C {
    let r = 0.45 * t.max(1e-4);
    match kind {
        0 => C::real(r),
        1 => C::new(0.0, r),
        _ => C::polar(r, 0.6 + TAU * 1.5 * t),
    }
}

/// The fan of 06_3: four small steps, a quarter turn apart.
pub const FAN_LEN: f64 = 0.3;

/// Fan step `k`.
pub fn fan_step(k: usize) -> C {
    C::polar(FAN_LEN, 0.35 + TAU / 4.0 * k as f64)
}

/// Fan step `k` after `stretch` and `turn` of the way to multiplying by f′(z₀) = 2z₀: at
/// (0, 0) the step itself, at (1, 1) exactly f′(z₀)·h.
pub fn fan_image(k: usize, stretch: f64, turn: f64) -> C {
    let d = Z0.scale(2.0);
    fan_step(k) * C::polar(1.0 + (d.abs() - 1.0) * stretch, d.arg() * turn)
}

/// The reflection of 06_8 caught `t` of the way, with 1 the point itself and −1 its mirror
/// image; 0 is the flat moment on the axis.
pub fn flip_at(z: C, t: f64) -> C {
    C::new(z.re, t * z.im)
}

/// The squash of 06_9 at height factor `t`: 1 is the point itself, 2 the tall rectangle,
/// kept in the window by the move down by i the caption says.
pub fn squash_at(z: C, t: f64) -> C {
    C::new(z.re, t * z.im - (t - 1.0))
}

/// Which of the rates go together in the two equations, in one colour per equation.
fn pairs() -> [(&'static str, Srgb); 4] {
    [
        ("u_x", derivative()),
        ("v_y", derivative()),
        ("v_x", hero()),
        ("u_y", hero()),
    ]
}

/// A fact printed to three places, as the voice says it: "2.000" as "2", "-2.000" as "-2".
fn whole(s: &str) -> &str {
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.')
    } else {
        s
    }
}

pub struct OneDerivative {
    pub kit: Kit,
}

impl SceneConstruct for OneDerivative {
    fn name(&self) -> &str {
        "06_one_derivative"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (inp, out) = planes();
        let f = |z: C| z * z;
        let w0 = f(Z0);

        // 06_1: step by h, divide the change by h
        let line = say(&kit, stage, "06_1")?;
        let frame = vec![inp.border(stage, input())?, out.border(stage, output())?];
        let mut axes = inp.axes(stage)?;
        axes.extend(out.axes(stage)?);
        axes.extend(axis_names(&kit, stage, &inp)?);
        axes.extend(axis_names(&kit, stage, &out)?);
        let titles = [
            kit.tex(
                stage,
                "z",
                30.0,
                input(),
                &[],
                v(inp.frame.center[0], inp.frame.top() + 0.3),
            )?,
            kit.tex(
                stage,
                FN_TEX[0],
                28.0,
                output(),
                &[],
                v(out.frame.left() + 0.95, out.frame.top() + 0.3),
            )?,
        ];
        let z0_dot = dot(stage, inp.p(Z0), 0.08, input());
        let w0_dot = dot(stage, out.p(w0), 0.08, output());
        let names = [
            kit.tex(
                stage,
                r"z_0 = 1 + i",
                28.0,
                input(),
                &[],
                inp.p(Z0 + C::new(-0.55, 0.6)),
            )?,
            kit.tex(stage, r"z_0^2 = 2i", 28.0, output(), &[], out.p(w0 + C::new(-0.6, 0.3)))?,
        ];
        fade_in_all(stage, &frame, 0.4)?;
        fade_in_all(stage, &axes, 0.3)?;
        fade_in_all(stage, &[z0_dot, w0_dot, names[0], names[1], titles[0], titles[1]], 0.4)?;
        line.wait_sentence(stage, 2)?;
        let h = C::polar(0.45, 0.6);
        let step = stage.arena_mut().add(
            Arrow::new(inp.p(Z0), inp.p(Z0 + h))
                .thickness(4.0)
                .color(input())
                .buff(0.0)
                .build()?,
        );
        let change = stage.arena_mut().add(
            Arrow::new(out.p(w0), out.p(f(Z0 + h)))
                .thickness(4.0)
                .color(output())
                .buff(0.0)
                .build()?,
        );
        let h_place = move |t: f64| inp.p(Z0 + h.scale(0.5 * t) + C::new(0.14, -0.14));
        let h_name = kit.tex(stage, "h", 28.0, input(), &[], h_place(1.0))?;
        let change_name = kit.text(stage, CHANGE, 20.0, output(), out.p(C::new(0.92, 2.72)))?;
        fade_in_all(stage, &[step, h_name], 0.4)?;
        fade_in_all(stage, &[change, change_name], 0.4)?;
        let quotient = kit.tex(stage, QUOTIENT_TEX, 36.0, Palette::ink(), &[], v(0.0, 3.1))?;
        fade_in_all(stage, &[quotient], 0.4)?;
        // as h shrinks, the two arrows shrink with it, and the ratio has a name to settle on
        line.wait_sentence(stage, 3)?;
        let shrink = stage.add_value_tracker(1.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        stage.add_updater(
            anchor,
            move |s, _| {
                let t = value(s, shrink);
                let tip = Z0 + h.scale(t);
                if let Ok(a) = Arrow::new(inp.p(Z0), inp.p(tip)).thickness(4.0).buff(0.0).build() {
                    let _ = s.set_points(step, a.points());
                }
                if let Ok(a) = Arrow::new(out.p(w0), out.p(tip * tip)).thickness(4.0).buff(0.0).build() {
                    let _ = s.set_points(change, a.points());
                }
                s.move_to(h_name, h_place(t), ORIGIN);
            },
            true,
        )?;
        let rt = line.until(stage, "that ratio should settle", 1.2)?.min(2.0);
        stage.play(tween_linear(shrink, 0.25, rt)?)?;
        stage.clear_updaters(anchor, true);
        stage.remove_from_scene(anchor);
        let settles = kit.tex(stage, SETTLES_TEX, 32.0, derivative(), &[], v(2.75, 3.1))?;
        fade_in_all(stage, &[settles], 0.4)?;

        // 06_2: from every direction the ratio settles on 2 + 2i
        kit.hold(stage)?;
        let line = say(&kit, stage, "06_2")?;
        fade_out_all(stage, &[step, change, h_name, change_name], 0.3)?;
        let target = fact("deriv.value")?;
        let parts: Vec<&str> = target.split(',').collect();
        let inset = Plane::new(v(5.85, 2.75), 1.9, (1.2, 2.8), (1.2, 2.8));
        let inset_frame = inset.border(stage, derivative())?;
        let limit = dot(stage, inset.p(C::new(2.0, 2.0)), 0.06, Palette::ink());
        let ratio_name = kit.text(stage, RATIO, 22.0, derivative(), v(5.85, 1.55))?;
        fade_in_all(stage, &[inset_frame, limit, ratio_name], 0.3)?;
        // one live reading, for whichever approach is running: the ratio itself, 2z₀ + h
        let ts = [
            stage.add_value_tracker(1.0),
            stage.add_value_tracker(1.0),
            stage.add_value_tracker(1.0),
        ];
        let which = stage.add_value_tracker(0.0);
        let mut live = Vec::new();
        let plus_i = [
            kit.text(stage, "+", 22.0, derivative(), v(5.62, 1.12))?,
            kit.text(stage, "i", 22.0, derivative(), v(6.38, 1.12))?,
        ];
        let mut trails = Vec::new();
        for kind in 0..3usize {
            if kind == 0 {
                line.wait_sentence(stage, 1)?;
                live.push(kit.live_value(
                    stage,
                    move |s| {
                        let k = value(s, which) as usize;
                        quot(approach(k, value(s, ts[k]))).re
                    },
                    2,
                    false,
                    22.0,
                    derivative(),
                    v(4.98, 1.12),
                    LEFT,
                )?);
                live.push(kit.live_value(
                    stage,
                    move |s| {
                        let k = value(s, which) as usize;
                        quot(approach(k, value(s, ts[k]))).im
                    },
                    2,
                    false,
                    22.0,
                    derivative(),
                    v(5.82, 1.12),
                    LEFT,
                )?);
                fade_in_all(stage, &plus_i, 0.1)?;
            }
            stage.set_tracker_value(which, kind as f64)?;
            let t = ts[kind];
            let dir = kit.text(stage, FROM[kind], 20.0, Palette::ink(), v(5.85, 0.72))?;
            let h_arrow = stage.arena_mut().add(
                polyline(vec![inp.p(Z0), inp.p(Z0 + approach(kind, 1.0))])
                    .style(line_style(input(), 4.0))
                    .build(),
            );
            let q_dot = dot(stage, inset.p(quot(approach(kind, 1.0))), 0.06, derivative());
            let trail = stage.arena_mut().add(
                polyline(vec![inset.p(quot(approach(kind, 1.0))); 2])
                    .style(faded_line(derivative(), 3.0, 0.8))
                    .build(),
            );
            let anchor = stage.arena_mut().add(VMobject::new());
            stage.add_to_scene(anchor)?;
            fade_in_all(stage, &[h_arrow, q_dot, trail, dir], 0.3)?;
            // the target stays on top of the trails
            stage.remove_from_scene(limit);
            stage.add_to_scene(limit)?;
            stage.add_updater(
                anchor,
                move |s, _| {
                    let tv = value(s, t);
                    let hz = approach(kind, tv);
                    let _ = s.set_points(h_arrow, polyline(vec![inp.p(Z0), inp.p(Z0 + hz)]).build().points());
                    s.move_to(q_dot, inset.p(quot(hz)), ORIGIN);
                    let n = 80;
                    let pts: Vec<Vec3> = (0..=n)
                        .map(|j| inset.p(quot(approach(kind, 1.0 - (1.0 - tv) * f64::from(j) / f64::from(n)))))
                        .collect();
                    let _ = s.set_points(trail, polyline(pts).build().points());
                },
                true,
            )?;
            stage.play(tween_linear(t, 0.0, 2.2)?)?;
            stage.clear_updaters(anchor, true);
            fade_out_all(stage, &[h_arrow, dir], 0.2)?;
            trails.extend([q_dot, trail]);
        }
        for &m in &live {
            stage.clear_updaters(m, true);
            stage.remove_from_scene(m);
        }
        stage.remove_from_scene(plus_i[0]);
        stage.remove_from_scene(plus_i[1]);
        let settled = kit.text(
            stage,
            &format!("→ {} + {} i", parts[0], parts[1]),
            22.0,
            derivative(),
            v(5.85, 1.12),
        )?;
        fade_in_all(stage, &[settled], 0.3)?;

        // 06_3: a tiny square lands as a tiny square: stretched and turned
        kit.hold(stage)?;
        let line = say(&kit, stage, "06_3")?;
        // every small step goes in and comes out the same way: four steps, all multiplied by
        // f′(z₀), first as themselves, then stretched, then turned
        line.wait_for(stage, "the function moves")?;
        // z₀²'s name goes to the corner the square's image will leave free
        shift_all(stage, &[names[1]], -0.4 * out.unit(), -1.35 * out.unit(), 0.3)?;
        let mut fan_in = Vec::with_capacity(4);
        let mut fan_out = Vec::with_capacity(4);
        for k in 0..4 {
            let blue = Arrow::new(inp.p(Z0), inp.p(Z0 + fan_step(k)))
                .thickness(4.0)
                .color(input())
                .buff(0.0)
                .build()?;
            fan_in.push(stage.arena_mut().add(blue));
            let green = Arrow::new(out.p(w0), out.p(w0 + fan_image(k, 0.0, 0.0)))
                .thickness(4.0)
                .color(output())
                .buff(0.0)
                .build()?;
            fan_out.push(stage.arena_mut().add(green));
        }
        let mut fan_all = fan_in.clone();
        fan_all.extend(&fan_out);
        fade_in_all(stage, &fan_all, 0.4)?;
        line.wait_sentence(stage, 2)?;
        let times = kit.tex(stage, TIMES_TEX, 30.0, derivative(), &[], v(3.3, 2.3))?;
        fade_in_all(stage, &[times], 0.3)?;
        line.wait_sentence(stage, 3)?;
        let named = kit.text(stage, A_STRETCH_AND_TURN, 22.0, derivative(), v(3.45, 1.35))?;
        fade_in_all(stage, &[named], 0.3)?;
        let stretch = stage.add_value_tracker(0.0);
        let turn = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        let green = fan_out.clone();
        stage.add_updater(
            anchor,
            move |s, _| {
                let (a, b) = (value(s, stretch), value(s, turn));
                for (k, &m) in green.iter().enumerate() {
                    let tip = out.p(w0 + fan_image(k, a, b));
                    if let Ok(arrow) = Arrow::new(out.p(w0), tip).thickness(4.0).buff(0.0).build() {
                        let _ = s.set_points(m, arrow.points());
                    }
                }
            },
            true,
        )?;
        stage.play(tween_linear(stretch, 1.0, 0.9)?)?;
        stage.play(tween_linear(turn, 1.0, 0.9)?)?;
        stage.clear_updaters(anchor, true);
        stage.remove_from_scene(anchor);
        line.wait_sentence(stage, 4)?;
        let mut fan_gone = fan_all;
        fan_gone.extend([times, named]);
        fade_out_all(stage, &fan_gone, 0.3)?;
        let side = stage.add_value_tracker(0.5);
        let sq = inp.path(stage, &square_edge(Z0, 0.5, 30), input(), 4.0)?;
        let img = out.path(stage, &image_at(0.5), output(), 4.0)?;
        let lin = out.path(stage, &linear_at(), derivative(), 3.0)?;
        let img_name = kit.text_left(stage, IMAGE_NAME, 18.0, output(), v(2.75, 1.45))?;
        let lin_name = kit.text_left(stage, EXACT_NAME, 18.0, derivative(), v(2.75, 1.15))?;
        fade_in_all(stage, &[sq, img, img_name], 0.5)?;
        fade_in_all(stage, &[lin, lin_name], 0.5)?;

        // 06_4: × 2.83 and an eighth of a turn; shrink, and the gap shrinks too
        kit.hold(stage)?;
        let line = say(&kit, stage, "06_4")?;
        let readout = kit.text(
            stage,
            &format!("{STRETCH}{}{TURN}{}°", fact("deriv.scale")?, fact("deriv.turn_deg")?),
            26.0,
            derivative(),
            v(3.2, 2.3),
        )?;
        fade_in_all(stage, &[readout], 0.4)?;
        line.wait_sentence(stage, 2)?;
        // the input square shrinks where it is; the right picture zooms in
        // about z₀² = 2i so the image keeps the side-0.5 size, and the green
        // image closes on the fixed teal square (`kept_size`)
        let zoomed = [
            kit.text_left(stage, ZOOMED, 18.0, Palette::dim(), v(2.55, -2.35))?,
            kit.text_left(stage, ZOOMED_WHY, 18.0, Palette::dim(), v(2.55, -2.62))?,
        ];
        fade_in_all(stage, &zoomed, 0.3)?;
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        stage.add_updater(
            anchor,
            move |s, _| {
                let sv = value(s, side);
                let _ = s.set_points(
                    sq,
                    polyline(square_edge(Z0, sv, 30).iter().map(|&z| inp.p(z)))
                        .build()
                        .points(),
                );
                let _ = s.set_points(img, polyline(kept_size(sv).iter().map(|&w| out.p(w))).build().points());
            },
            true,
        )?;
        stage.play(tween_linear(side, 0.05, 3.0)?)?;
        stage.clear_updaters(anchor, true);
        let gap = kit.text(
            stage,
            &format!("{GAP}{} → {}", fact("deriv.gap.side0_5")?, fact("deriv.gap.side0_05")?),
            24.0,
            Palette::ink(),
            v(2.4, -3.35),
        )?;
        fade_in_all(stage, &[gap], 0.4)?;

        // 06_5: the two directions in coordinates (the planes give way to formulas)
        kit.hold(stage)?;
        let line = say(&kit, stage, "06_5")?;
        let mut gone = vec![
            sq,
            img,
            lin,
            img_name,
            lin_name,
            readout,
            gap,
            zoomed[0],
            zoomed[1],
            inset_frame,
            limit,
            ratio_name,
            settled,
            quotient,
            settles,
            titles[0],
            titles[1],
        ];
        gone.extend(trails);
        gone.extend([z0_dot, w0_dot, names[0], names[1]]);
        gone.extend(frame);
        gone.extend(axes);
        fade_out_all(stage, &gone, 0.4)?;
        // the coordinates the voice asks for, before any row needs them
        let z_is = kit.tex(stage, COORDS_TEX[0], 34.0, input(), &[], v(-2.2, 3.35))?;
        fade_in_all(stage, &[z_is], 0.3)?;
        line.wait_sentence(stage, 1)?;
        let f_is = kit.tex(stage, COORDS_TEX[1], 34.0, output(), &[], v(2.2, 3.35))?;
        fade_in_all(stage, &[f_is], 0.3)?;
        line.wait_sentence(stage, 2)?;
        let right_row = [
            kit.text(stage, LABELS[0], 26.0, Palette::dim(), v(-1.2, 2.55))?,
            kit.tex(stage, RIGHT_TEX, 34.0, Palette::ink(), &pairs(), v(1.2, 2.55))?,
        ];
        let rate = [
            kit.tex(stage, JACOBIAN[0], 24.0, derivative(), &[], v(-1.0, 2.08))?,
            kit.text_left(stage, RATE, 22.0, Palette::dim(), v(-0.7, 2.08))?,
        ];
        fade_in_all(stage, &right_row, 0.4)?;
        fade_in_all(stage, &rate, 0.3)?;
        line.wait_sentence(stage, 3)?;
        let up_row = [
            kit.text(stage, LABELS[1], 26.0, Palette::dim(), v(-1.2, 1.35))?,
            kit.tex(stage, UP_TEX, 34.0, Palette::ink(), &pairs(), v(1.6, 1.35))?,
        ];
        fade_in_all(stage, &up_row, 0.4)?;
        // "the two answers must agree": a bar joins the two rows
        line.wait_sentence(stage, 4)?;
        let agree = [
            stage.arena_mut().add(
                polyline(vec![v(-2.2, 2.55), v(-2.2, 1.35)])
                    .style(line_style(hero(), 4.0))
                    .build(),
            ),
            kit.text(stage, AGREE, 24.0, hero(), v(-3.2, 1.95))?,
        ];
        fade_in_all(stage, &agree, 0.3)?;

        // 06_6: the Cauchy–Riemann equations
        kit.hold(stage)?;
        let line = say(&kit, stage, "06_6")?;
        // each equation on its own sentence, its rates in the colours they have in the rows
        line.wait_sentence(stage, 1)?;
        let first = explainer_kit::props::equation_card(&kit, stage, CR_PARTS[0], 40.0, &pairs(), v(-1.9, 0.1))?;
        fade_in_all(stage, &[first], 0.5)?;
        line.wait_sentence(stage, 2)?;
        let second = explainer_kit::props::equation_card(&kit, stage, CR_PARTS[1], 40.0, &pairs(), v(2.1, 0.1))?;
        fade_in_all(stage, &[second], 0.5)?;
        line.wait_sentence(stage, 3)?;
        let search = names_now(&kit, stage, &["Cauchy–Riemann equations"])?;

        // 06_7: the local matrix is a stretch and a turn: [[2, −2], [2, 2]]
        kit.hold(stage)?;
        let line = say(&kit, stage, "06_7")?;
        fade_out_all(stage, &[search], 0.3)?;
        // the grid in symbols with what it must be, then its numbers, then what they multiply by
        let symbols = symbol_matrix(&kit, stage, &JACOBIAN, v(-2.4, -1.45))?;
        let kind = kit.text(stage, STRETCH_TURN, 24.0, derivative(), v(0.2, -2.7))?;
        fade_in_all(stage, &symbols, 0.5)?;
        fade_in_all(stage, &[kind], 0.4)?;
        line.wait_sentence(stage, 1)?;
        let j: Vec<&str> = fact("deriv.jacobian")?.split(',').collect();
        let equals = kit.tex(stage, "=", 34.0, Palette::ink(), &[], v(-0.55, -1.45))?;
        let grid = matrix(&kit, stage, &[j[0], j[1], j[2], j[3]], v(1.35, -1.45))?;
        let mut numbers = vec![equals];
        numbers.extend(&grid);
        fade_in_all(stage, &numbers, 0.5)?;
        line.wait_sentence(stage, 2)?;
        let times_it = kit.tex(
            stage,
            &format!(r"= \times\,({} + {}\,i)", whole(parts[0]), whole(parts[1])),
            34.0,
            derivative(),
            &[],
            v(4.8, -1.45),
        )?;
        fade_in_all(stage, &[times_it], 0.4)?;

        // 06_8: reflection: a derivative anywhere?
        kit.hold(stage)?;
        let line = say(&kit, stage, "06_8")?;
        let mut gone = vec![first, second, kind, z_is, f_is];
        gone.extend(right_row);
        gone.extend(rate);
        gone.extend(up_row);
        gone.extend(agree);
        gone.extend(symbols);
        gone.extend(numbers);
        gone.push(times_it);
        fade_out_all(stage, &gone, 0.4)?;
        // the plane and its square from the start; the green copy flips over the axis as the
        // voice says to, its mark turning with it, so the reversal is seen and not told
        let mirror = Plane::new(v(-2.6, -0.4), 5.4, (-0.6, 2.6), (-1.6, 1.6));
        let mut m_axes = mirror.axes(stage)?;
        m_axes.extend(axis_names(&kit, stage, &mirror)?);
        let blue_sq = mirror.path(stage, &square_edge(Z0, 0.6, 30), input(), 4.0)?;
        let [blue_arc, blue_head] = orientation(stage, &mirror, input(), 1.0)?;
        fade_in_all(stage, &m_axes, 0.3)?;
        fade_in_all(stage, &[blue_sq, blue_arc, blue_head], 0.4)?;
        line.wait_sentence(stage, 1)?;
        let mirror_line = mirror.path(stage, &[C::new(-0.6, 0.0), C::new(2.6, 0.0)], hero(), 3.0)?;
        let green_sq = mirror.path(stage, &square_edge(Z0, 0.6, 30), output(), 4.0)?;
        let [green_arc, green_head] = orientation(stage, &mirror, output(), 1.0)?;
        fade_in_all(stage, &[mirror_line, green_sq, green_arc, green_head], 0.3)?;
        let flip = stage.add_value_tracker(1.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        stage.add_updater(
            anchor,
            move |s, _| {
                let t = value(s, flip);
                let edge: Vec<Vec3> = square_edge(Z0, 0.6, 30)
                    .into_iter()
                    .map(|z| mirror.p(flip_at(z, t)))
                    .collect();
                let _ = s.set_points(green_sq, polyline(edge).build().points());
                let (arc_pts, head_pts) = mark_points(&mirror, t);
                let _ = s.set_points(green_arc, polyline(arc_pts).build().points());
                let _ = s.set_points(green_head, Polygon::new(head_pts).build().points());
            },
            true,
        )?;
        let rt = line.until(stage, "It sends", 1.2)?.min(2.2);
        stage.play(tween_linear(flip, -1.0, rt)?)?;
        stage.clear_updaters(anchor, true);
        stage.remove_from_scene(anchor);
        line.wait_sentence(stage, 2)?;
        let conj_says = kit.tex(stage, CONJ_TEX, 30.0, Palette::ink(), &[], v(-2.6, 2.85))?;
        fade_in_all(stage, &[conj_says], 0.4)?;
        let asked = predict(&kit, stage, &line, &ASK, v(4.2, 2.4))?;

        // 06_9: nowhere: right gives 1, up gives −1; the squash fails too
        let line = say(&kit, stage, "06_9")?;
        fade_out_all(stage, &asked, 0.3)?;
        let conj: Vec<&str> = fact("deriv.conj")?.split(',').collect();
        line.wait_sentence(stage, 1)?;
        let r1 = kit.text(
            stage,
            &format!("{}{}", RIGHT_UP[0], conj[0]),
            28.0,
            Palette::ink(),
            v(4.2, 1.6),
        )?;
        fade_in_all(stage, &[r1], 0.3)?;
        line.wait_sentence(stage, 2)?;
        let r2 = kit.text(
            stage,
            &format!("{}{}", RIGHT_UP[1], conj[1].replace('-', "−")),
            28.0,
            Palette::ink(),
            v(4.2, 1.0),
        )?;
        fade_in_all(stage, &[r2], 0.3)?;
        line.wait_sentence(stage, 3)?;
        let cross1 = red_cross(stage, v(4.2, 0.3))?;
        let no_derivative = kit.text(stage, NO_DERIVATIVE, 22.0, warn(), v(4.2, -0.25))?;
        fade_in_all(stage, &cross1, 0.3)?;
        fade_in_all(stage, &[no_derivative], 0.3)?;
        line.wait_sentence(stage, 4)?;
        fade_out_all(stage, &[green_sq, green_arc, green_head, conj_says, mirror_line], 0.3)?;
        let squash_says = kit.tex(stage, SQUASH_TEX, 30.0, Palette::ink(), &[], v(-2.6, 2.85))?;
        fade_in_all(stage, &[squash_says], 0.3)?;
        // the square itself grows tall as the voice doubles the height
        let squashed = mirror.path(stage, &square_edge(Z0, 0.6, 30), output(), 4.0)?;
        let sq_vals: Vec<&str> = fact("deriv.squash")?.split(',').collect();
        let s1 = kit.text(
            stage,
            &format!("{}{}   {}{}", RIGHT_UP[0], sq_vals[0], RIGHT_UP[1], sq_vals[1]),
            26.0,
            Palette::ink(),
            v(4.2, -1.3),
        )?;
        let moved = kit.text(stage, MOVED, 20.0, Palette::ink(), mirror.p(C::new(1.0, -1.45)))?;
        line.wait_sentence(stage, 5)?;
        fade_in_all(stage, &[squashed], 0.2)?;
        let tall = stage.add_value_tracker(1.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        stage.add_updater(
            anchor,
            move |s, _| {
                let t = value(s, tall);
                let edge: Vec<Vec3> = square_edge(Z0, 0.6, 30)
                    .into_iter()
                    .map(|z| mirror.p(squash_at(z, t)))
                    .collect();
                let _ = s.set_points(squashed, polyline(edge).build().points());
            },
            true,
        )?;
        stage.play(tween_linear(tall, 2.0, 1.2)?)?;
        stage.clear_updaters(anchor, true);
        stage.remove_from_scene(anchor);
        fade_in_all(stage, &[s1, moved], 0.4)?;
        let cross2 = red_cross(stage, v(4.2, -2.3))?;
        fade_in_all(stage, &cross2, 0.3)?;
        wait_until(stage, line.end())?;
        kit.music_breath(stage, 2.0)?;
        let mut all = vec![
            blue_sq,
            blue_arc,
            blue_head,
            squashed,
            s1,
            r1,
            r2,
            moved,
            squash_says,
            no_derivative,
        ];
        all.extend(m_axes);
        all.extend(cross1);
        all.extend(cross2);
        fade_out_all(stage, &all, 0.5)?;
        kit.finish(stage)
    }
}

/// The difference quotient of z² at z₀ for step `h` (exactly 2z₀ + h).
fn quot(h: C) -> C {
    ((Z0 + h) * (Z0 + h) - Z0 * Z0) / h
}

/// The image under z² of the side-`side` square at z₀.
fn image_at(side: f64) -> Vec<C> {
    square_edge(Z0, side, 30).into_iter().map(|z| z * z).collect()
}

/// The linear image of the side-0.5 square: z₀² + 2z₀(z − z₀).
fn linear_at() -> Vec<C> {
    square_edge(Z0, 0.5, 30)
        .into_iter()
        .map(|z| Z0 * Z0 + Z0.scale(2.0) * (z - Z0))
        .collect()
}

/// The anticlockwise mark inside the square at z₀ of 06_8, caught `t` of the way through the
/// reflection: the arc's points and the corners of its head, both in scene coordinates. The
/// head has its own size, since the arc's last chord is too short for an `Arrow`'s head.
fn mark_points(p: &Plane, t: f64) -> (Vec<Vec3>, Vec<Vec3>) {
    let arc: Vec<Vec3> = (0..=20)
        .map(|k| {
            let at = -2.4 + 4.0 * f64::from(k) / 20.0;
            p.p(flip_at(Z0 + C::polar(0.6 * 0.3, at), t))
        })
        .collect();
    let head = arrow_head_points(arc[20], arc[19], 0.2);
    (arc, head)
}

/// The mark of the square at z₀, detached: `[arc, head]`.
fn orientation(stage: &mut Stage<'_>, p: &Plane, color: Srgb, t: f64) -> fmn::Result<[Mob; 2]> {
    let (arc, head) = mark_points(p, t);
    let body = polyline(arc).style(line_style(color, 4.0)).build();
    let tip = Polygon::new(head).style(flat(color, 1.0)).build();
    Ok([stage.arena_mut().add(body), stage.arena_mut().add(tip)])
}

/// A red cross: no complex derivative.
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

/// A 2 × 2 grid of numbers between brackets, centred at `at`.
fn matrix(kit: &Kit, stage: &mut Stage<'_>, cells: &[&str; 4], at: Vec3) -> fmn::Result<Vec<Mob>> {
    let (dx, dy) = (0.7, 0.42);
    let mut out = Vec::new();
    for (i, c) in cells.iter().enumerate() {
        let (row, col) = (i / 2, i % 2);
        let x = at[0] + if col == 0 { -dx } else { dx };
        let y = at[1] + if row == 0 { dy } else { -dy };
        out.push(kit.text(stage, &whole(c).replace('-', "−"), 32.0, Palette::ink(), v(x, y))?);
    }
    out.extend(brackets(stage, at));
    Ok(out)
}

/// The same grid with typeset entries, its rates in the colours of [`pairs`].
fn symbol_matrix(kit: &Kit, stage: &mut Stage<'_>, cells: &[&str; 4], at: Vec3) -> fmn::Result<Vec<Mob>> {
    let (dx, dy) = (0.7, 0.42);
    let mut out = Vec::new();
    for (i, c) in cells.iter().enumerate() {
        let (row, col) = (i / 2, i % 2);
        let x = at[0] + if col == 0 { -dx } else { dx };
        let y = at[1] + if row == 0 { dy } else { -dy };
        out.push(kit.tex(stage, c, 32.0, Palette::ink(), &pairs(), v(x, y))?);
    }
    out.extend(brackets(stage, at));
    Ok(out)
}

/// The two brackets of a 2 × 2 grid centred at `at`.
fn brackets(stage: &mut Stage<'_>, at: Vec3) -> Vec<Mob> {
    let mut out = Vec::with_capacity(2);
    for side in [-1.0, 1.0] {
        let x = at[0] + side * 1.45;
        let tip = side * -0.15;
        let pts = vec![
            v(x + tip, at[1] + 0.8),
            v(x, at[1] + 0.8),
            v(x, at[1] - 0.8),
            v(x + tip, at[1] - 0.8),
        ];
        out.push(
            stage
                .arena_mut()
                .add(polyline(pts).style(line_style(Palette::ink(), 3.0)).build()),
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_typed_names_are_the_point_drawn() {
        // the labels "z₀ = 1 + i" and "z₀² = 2i" of 06_1
        assert!((Z0 - C::new(1.0, 1.0)).abs() < 1e-12);
        assert!((Z0 * Z0 - C::new(0.0, 2.0)).abs() < 1e-12);
    }

    #[test]
    fn every_approach_settles_on_two_plus_two_i() {
        for kind in 0..3 {
            let q = quot(approach(kind, 1e-3));
            assert!((q - C::new(2.0, 2.0)).abs() < 1e-3, "approach {kind}: {q:?}");
        }
    }

    #[test]
    fn the_zoomed_image_tends_to_the_teal_square() {
        let teal: Vec<C> = square_edge(C::new(0.0, 0.0), 1.0, 40)
            .into_iter()
            .map(|u| Z0.scale(2.0) * u)
            .collect();
        let worst = |side: f64| {
            zoomed_image(side)
                .iter()
                .zip(&teal)
                .map(|(a, b)| (*a - *b).abs())
                .fold(0.0, f64::max)
        };
        // the zoomed gap is exactly side × |u|² ≤ side/2, so it shrinks with the square
        assert!(worst(0.5) > worst(0.05) * 9.0);
        // and the ratio the film prints agrees with the model's
        assert!((square_gap_ratio(Z0, 0.5) - 0.0884).abs() < 1e-3);
    }

    #[test]
    fn the_zoomed_right_plane_starts_true_and_closes_on_the_teal_square() {
        let units = square_edge(C::new(0.0, 0.0), 1.0, 40);
        let w0 = Z0 * Z0;
        // at side 0.5 the zoomed picture is the true image, so the zoom starts without a jump
        for (w, u) in kept_size(0.5).iter().zip(&units) {
            let z = Z0 + u.scale(0.5);
            assert!((*w - z * z).abs() < 1e-12);
        }
        // the teal square is z₀² + z₀u, and the gap is 0.5·side·|u|² ≤ side/4
        for side in [0.5, 0.2, 0.05] {
            let worst = kept_size(side)
                .iter()
                .zip(&units)
                .map(|(w, u)| (*w - (w0 + Z0 * *u)).abs())
                .fold(0.0, f64::max);
            assert!(
                worst <= side / 4.0 + 1e-12 && worst > side / 4.0 - 1e-9,
                "side {side}: {worst}"
            );
        }
        // and the picture stays in the right plane
        let (_, out) = planes();
        assert!(kept_size(0.05).iter().all(|&w| out.contains(w)));
    }

    #[test]
    fn reflection_reverses_orientation_and_squash_does_not_keep_squares() {
        let sq = square_edge(Z0, 0.6, 10);
        let area = |pts: &[C]| {
            pts.windows(2)
                .map(|w| w[0].re * w[1].im - w[1].re * w[0].im)
                .sum::<f64>()
                / 2.0
        };
        let flipped: Vec<C> = sq.iter().map(|z| z.conj()).collect();
        assert!(area(&sq) > 0.0 && area(&flipped) < 0.0);
        let tall: Vec<C> = sq.iter().map(|z| C::new(z.re, 2.0 * z.im)).collect();
        assert!((area(&tall) / area(&sq) - 2.0).abs() < 1e-9);
    }

    #[test]
    fn the_windows_hold_the_pictures() {
        let (inp, out) = planes();
        for z in square_edge(Z0, 0.5, 10) {
            assert!(inp.contains(z) && out.contains(z * z));
        }
        assert!(inp.frame.right() < out.frame.left());
    }

    #[test]
    fn the_fan_ends_as_the_derivative_times_each_step() {
        let (inp, out) = planes();
        let d = Z0.scale(2.0);
        for k in 0..4 {
            assert!((fan_image(k, 0.0, 0.0) - fan_step(k)).abs() < 1e-12);
            assert!((fan_image(k, 1.0, 1.0) - d * fan_step(k)).abs() < 1e-12);
            assert!(inp.contains(Z0 + fan_step(k)));
            assert!(out.contains(Z0 * Z0 + fan_image(k, 1.0, 1.0)));
        }
    }

    #[test]
    fn the_flip_ends_on_the_mirror_image_and_the_squash_on_the_tall_rectangle() {
        for z in square_edge(Z0, 0.6, 10) {
            assert!((flip_at(z, 1.0) - z).abs() < 1e-12);
            assert!((flip_at(z, -1.0) - z.conj()).abs() < 1e-12);
            assert!((squash_at(z, 1.0) - z).abs() < 1e-12);
            assert!((squash_at(z, 2.0) - C::new(z.re, 2.0 * z.im - 1.0)).abs() < 1e-12);
        }
    }

    #[test]
    fn the_printed_rates_keep_their_value() {
        assert_eq!(whole("2.000"), "2");
        assert_eq!(whole("-2.000"), "-2");
        assert_eq!(whole("10.500"), "10.5");
        assert_eq!(whole("20"), "20");
        let parts: Vec<&str> = fact("deriv.value").unwrap().split(',').collect();
        assert_eq!((whole(parts[0]), whole(parts[1])), ("2", "2"));
    }
}
