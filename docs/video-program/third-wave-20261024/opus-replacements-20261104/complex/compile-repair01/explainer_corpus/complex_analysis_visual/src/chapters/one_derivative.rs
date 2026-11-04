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
    Plane, derivative, dot, fact, fade_in_all, fade_out_all, faded_line, input, line_style, names_now, output, predict,
    say, tween_linear, value, wait_until, warn,
};
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
pub const ZOOMED: &str = "zoomed in about 2i to keep the image's size";

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
        let z0_dot = dot(stage, inp.p(Z0), 0.08, input());
        let w0_dot = dot(stage, out.p(w0), 0.08, output());
        let names = [
            kit.tex(
                stage,
                r"z_0 = 1 + i",
                28.0,
                input(),
                &[],
                inp.p(Z0 + C::new(-0.45, 0.25)),
            )?,
            kit.tex(stage, r"z_0^2 = 2i", 28.0, output(), &[], out.p(w0 + C::new(-0.6, 0.3)))?,
        ];
        fade_in_all(stage, &frame, 0.4)?;
        fade_in_all(stage, &axes, 0.3)?;
        fade_in_all(stage, &[z0_dot, w0_dot, names[0], names[1]], 0.4)?;
        line.wait_sentence(stage, 2)?;
        let h = C::polar(0.45, 0.6);
        let step = stage.arena_mut().add(
            Arrow::new(inp.p(Z0), inp.p(Z0 + h))
                .thickness(4.0)
                .color(input())
                .build()?,
        );
        let change = stage.arena_mut().add(
            Arrow::new(out.p(w0), out.p(f(Z0 + h)))
                .thickness(4.0)
                .color(output())
                .build()?,
        );
        fade_in_all(stage, &[step], 0.4)?;
        fade_in_all(stage, &[change], 0.4)?;
        let quotient = kit.tex(stage, QUOTIENT_TEX, 36.0, Palette::ink(), &[], v(0.0, 3.1))?;
        fade_in_all(stage, &[quotient], 0.4)?;

        // 06_2: from every direction the ratio settles on 2 + 2i
        kit.hold(stage)?;
        let line = say(&kit, stage, "06_2")?;
        fade_out_all(stage, &[step, change], 0.3)?;
        let target = fact("deriv.value")?;
        let parts: Vec<&str> = target.split(',').collect();
        let inset = Plane::new(v(5.85, 2.75), 1.9, (1.2, 2.8), (1.2, 2.8));
        let inset_frame = inset.border(stage, derivative())?;
        let limit = dot(stage, inset.p(C::new(2.0, 2.0)), 0.06, Palette::ink());
        fade_in_all(stage, &[inset_frame, limit], 0.3)?;
        let mut trails = Vec::new();
        for kind in 0..3usize {
            if kind == 0 {
                line.wait_sentence(stage, 1)?;
            }
            let t = stage.add_value_tracker(1.0);
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
            fade_in_all(stage, &[h_arrow, q_dot, trail], 0.3)?;
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
            fade_out_all(stage, &[h_arrow], 0.2)?;
            trails.extend([q_dot, trail]);
        }
        let settled = kit.text(
            stage,
            &format!("→ {} + {} i", parts[0], parts[1]),
            22.0,
            derivative(),
            v(5.85, 1.5),
        )?;
        fade_in_all(stage, &[settled], 0.3)?;

        // 06_3: a tiny square lands as a tiny square: stretched and turned
        kit.hold(stage)?;
        let line = say(&kit, stage, "06_3")?;
        line.wait_sentence(stage, 4)?;
        let side = stage.add_value_tracker(0.5);
        let sq = inp.path(stage, &square_edge(Z0, 0.5, 30), input(), 4.0)?;
        let img = out.path(stage, &image_at(0.5), output(), 4.0)?;
        let lin = out.path(stage, &linear_at(), derivative(), 3.0)?;
        fade_in_all(stage, &[sq, img], 0.5)?;
        fade_in_all(stage, &[lin], 0.5)?;

        // 06_4: × 2.83 and an eighth of a turn; shrink, and the gap shrinks too
        kit.hold(stage)?;
        let line = say(&kit, stage, "06_4")?;
        let readout = kit.text(
            stage,
            &format!("{STRETCH}{}{TURN}{}°", fact("deriv.scale")?, fact("deriv.turn_deg")?),
            26.0,
            derivative(),
            v(2.4, 2.3),
        )?;
        fade_in_all(stage, &[readout], 0.4)?;
        line.wait_sentence(stage, 2)?;
        // the input square shrinks where it is; the right picture zooms in
        // about z₀² = 2i so the image keeps the side-0.5 size, and the green
        // image closes on the fixed teal square (`kept_size`)
        let zoomed = kit.text(stage, ZOOMED, 18.0, Palette::dim(), out.p(C::new(0.0, 0.6)))?;
        fade_in_all(stage, &[zoomed], 0.3)?;
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
            readout,
            gap,
            zoomed,
            inset_frame,
            limit,
            settled,
            quotient,
        ];
        gone.extend(trails);
        gone.extend([z0_dot, w0_dot, names[0], names[1]]);
        gone.extend(frame);
        gone.extend(axes);
        fade_out_all(stage, &gone, 0.4)?;
        line.wait_sentence(stage, 2)?;
        let right_row = [
            kit.text(stage, LABELS[0], 26.0, Palette::dim(), v(-1.2, 3.2))?,
            kit.tex(stage, RIGHT_TEX, 34.0, Palette::ink(), &[], v(1.2, 3.2))?,
        ];
        fade_in_all(stage, &right_row, 0.4)?;
        line.wait_sentence(stage, 3)?;
        let up_row = [
            kit.text(stage, LABELS[1], 26.0, Palette::dim(), v(-1.2, 2.35))?,
            kit.tex(stage, UP_TEX, 34.0, Palette::ink(), &[], v(1.6, 2.35))?,
        ];
        fade_in_all(stage, &up_row, 0.4)?;

        // 06_6: the Cauchy–Riemann equations
        kit.hold(stage)?;
        let line = say(&kit, stage, "06_6")?;
        line.wait_sentence(stage, 1)?;
        let cr = explainer_kit::props::equation_card(&kit, stage, CR_TEX, 40.0, &[], v(0.2, 1.2))?;
        fade_in_all(stage, &[cr], 0.5)?;
        line.wait_sentence(stage, 3)?;
        let search = names_now(&kit, stage, &["Cauchy–Riemann equations"])?;

        // 06_7: the local matrix is a stretch and a turn: [[2, −2], [2, 2]]
        kit.hold(stage)?;
        let line = say(&kit, stage, "06_7")?;
        fade_out_all(stage, &[search], 0.3)?;
        let j: Vec<&str> = fact("deriv.jacobian")?.split(',').collect();
        let grid = matrix(&kit, stage, &[j[0], j[1], j[2], j[3]], v(0.2, -0.3))?;
        let kind = kit.text(stage, STRETCH_TURN, 24.0, derivative(), v(0.2, -1.55))?;
        line.wait_sentence(stage, 1)?;
        fade_in_all(stage, &grid, 0.5)?;
        line.wait_sentence(stage, 2)?;
        fade_in_all(stage, &[kind], 0.4)?;

        // 06_8: reflection: a derivative anywhere?
        kit.hold(stage)?;
        let line = say(&kit, stage, "06_8")?;
        let mut gone = vec![cr, kind];
        gone.extend(right_row);
        gone.extend(up_row);
        gone.extend(grid);
        fade_out_all(stage, &gone, 0.4)?;
        let mirror = Plane::new(v(-2.6, -0.4), 5.4, (-0.6, 2.6), (-1.6, 1.6));
        let m_axes = mirror.axes(stage)?;
        let blue_sq = mirror.path(stage, &square_edge(Z0, 0.6, 30), input(), 4.0)?;
        let blue_turn = orientation(stage, &mirror, Z0, 0.6, false, input())?;
        line.wait_sentence(stage, 2)?;
        fade_in_all(stage, &m_axes, 0.3)?;
        fade_in_all(stage, &[blue_sq, blue_turn], 0.4)?;
        let flipped: Vec<C> = square_edge(Z0, 0.6, 30).into_iter().map(|z| z.conj()).collect();
        let green_sq = mirror.path(stage, &flipped, output(), 4.0)?;
        let green_turn = orientation(stage, &mirror, Z0.conj(), 0.6, true, output())?;
        fade_in_all(stage, &[green_sq, green_turn], 0.4)?;
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
        let cross1 = red_cross(stage, v(4.2, -0.1))?;
        fade_in_all(stage, &cross1, 0.3)?;
        line.wait_sentence(stage, 4)?;
        fade_out_all(stage, &[green_sq, green_turn], 0.3)?;
        let tall: Vec<C> = square_edge(Z0, 0.6, 30)
            .into_iter()
            .map(|z| C::new(z.re, 2.0 * z.im - 1.0))
            .collect();
        let squashed = mirror.path(stage, &tall, output(), 4.0)?;
        let sq_vals: Vec<&str> = fact("deriv.squash")?.split(',').collect();
        let s1 = kit.text(
            stage,
            &format!("{}{}   {}{}", RIGHT_UP[0], sq_vals[0], RIGHT_UP[1], sq_vals[1]),
            26.0,
            Palette::ink(),
            v(4.2, -1.3),
        )?;
        let moved = kit.text(stage, MOVED, 20.0, Palette::dim(), mirror.p(C::new(1.0, -1.45)))?;
        line.wait_sentence(stage, 5)?;
        fade_in_all(stage, &[squashed, s1, moved], 0.4)?;
        let cross2 = red_cross(stage, v(4.2, -2.3))?;
        fade_in_all(stage, &cross2, 0.3)?;
        wait_until(stage, line.end())?;
        kit.music_breath(stage, 2.0)?;
        let mut all = vec![blue_sq, blue_turn, squashed, s1, r1, r2, moved];
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

/// A curved arrow inside the square at `c` showing its orientation
/// (counterclockwise, or clockwise when `reversed`).
fn orientation(stage: &mut Stage<'_>, p: &Plane, c: C, side: f64, reversed: bool, color: Srgb) -> fmn::Result<Mob> {
    let r = side * 0.28;
    let (a0, a1) = if reversed { (2.4, -1.6) } else { (-2.4, 1.6) };
    let pts: Vec<C> = (0..=20)
        .map(|k| c + C::polar(r, a0 + (a1 - a0) * f64::from(k) / 20.0))
        .collect();
    let n = pts.len();
    let body = polyline(pts.iter().map(|&z| p.p(z)))
        .style(line_style(color, 3.0))
        .build();
    let head = Arrow::new(p.p(pts[n - 3]), p.p(pts[n - 1]))
        .thickness(3.0)
        .color(color)
        .build()?;
    let b = stage.arena_mut().add(body);
    let h = stage.arena_mut().add(head);
    super::draw::group(stage, &[b, h])
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
        out.push(kit.text(stage, &c.replace('-', "−"), 32.0, Palette::ink(), v(x, y))?);
    }
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
    Ok(out)
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
}
