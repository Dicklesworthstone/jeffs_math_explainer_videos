//! Chapter 10, the many ways to do nothing: the identity reached by zero
//! velocity and by one full turn (10_1); flows around ellipses, every point
//! once round its own ellipse, all home at t = 1, and a continuous family
//! of them as the ellipse is stretched (10_2); the principal logarithm
//! through eigenvalues, the matrix version of the branch cut (10_3).
//!
//! 10_3's statement is Higham, Functions of Matrices (SIAM 2008), Thm 1.31:
//! if A has no eigenvalue on (−∞, 0], exactly one logarithm X of A has every
//! eigenvalue in −π < Im < π; it is real when A is real. The ellipse
//! generators 2πPJP⁻¹ have eigenvalues ±2πi, so for A = I that one is 0.
//!
//! Rebuild candidate for `mx-video-program-program-87do.63`; uncompiled,
//! unrendered, unlistened.

use std::f64::consts::{PI, TAU};

use explainer_kit::{Kit, Palette, play, tween, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::draw::{
    answer_c, dot, fact, fade_in_all, fade_out_all, faded_line, flat, grid_under, lap_c, line_style, matrix_card,
    number_c, position_c, precisely, ring, seg, slide_c, swap, title, value, warn_c,
};
use crate::model::{self, J, M2};

pub struct DoingNothing {
    pub kit: Kit,
}

const CENTRE: [f64; 3] = [-3.3, -0.4, 0.0];
const UNIT: f64 = 0.72;
/// The grid points whose journeys are traced.
const TRACED: [[f64; 2]; 6] = [[1.0, 0.0], [0.0, 1.0], [1.0, 1.0], [-1.0, 0.5], [0.5, -1.0], [2.0, 0.0]];

fn screen(p: [f64; 2]) -> Vec3 {
    v(CENTRE[0] + UNIT * p[0], CENTRE[1] + UNIT * p[1])
}

/// The ellipse flow's generator for the ellipse with half-axes `a`, `b`
/// tilted by `tilt` degrees: 2π P J P⁻¹, P = R(tilt) diag(a, b).
fn generator(a: f64, b: f64, tilt: f64) -> M2 {
    model::ellipse_generator(a, b, tilt.to_radians())
}

/// Parse "a,b,tilt;a,b,tilt" into ellipses.
fn ellipses(s: &str) -> Vec<(f64, f64, f64)> {
    s.split(';')
        .filter_map(|e| {
            let n: Vec<f64> = e.split(',').filter_map(|x| x.trim().parse().ok()).collect();
            (n.len() == 3).then(|| (n[0], n[1], n[2]))
        })
        .collect()
}

/// The traced points' journeys under e^{tX} from 0 to `t`, and the points.
fn journeys(s: &mut Arena, x: M2, t: f64, c: Srgb) -> Mob {
    let g = s.add(VMobject::new());
    for p in TRACED {
        let n = ((t * 90.0).ceil() as usize).max(2);
        let pts: Vec<Vec3> = (0..=n)
            .map(|k| screen(model::apply(model::flow(x, t * k as f64 / n as f64), p)))
            .collect();
        let tr = s.add(polyline(pts).style(line_style(c, 2.5)).build());
        let d = s.add(
            Dot::new()
                .point(screen(model::apply(model::flow(x, t), p)))
                .radius(0.08)
                .style(flat(c, 1.0))
                .build(),
        );
        let _ = s.attach(g, tr);
        let _ = s.attach(g, d);
    }
    g
}

impl SceneConstruct for DoingNothing {
    fn name(&self) -> &str {
        "10_doing_nothing"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (green, blue, gold, purple, red) = (number_c(), position_c(), answer_c(), lap_c(), warn_c());
        let right = 1.0;

        // 10_1: the identity; zero velocity; one full turn
        kit.say(stage, "10_1")?;
        let heading = title(&kit, stage, "Many ways to do nothing")?;
        let still = stage.add_value_tracker(0.0);
        let which = stage.add_value_tracker(0.0);
        let shapes = ellipses(fact("matrix.ellipses")?);
        let gens: Vec<M2> = std::iter::once(model::mscale(J, TAU))
            .chain(shapes.iter().map(|&(a, b, t)| generator(a, b, t)))
            .collect();
        let gens_redraw = gens.clone();
        let grid = stage.always_redraw(move |s: &mut Arena| {
            let k = (value(s, which).round() as usize).min(gens_redraw.len() - 1);
            clipped_grid(
                s,
                model::flow(gens_redraw[k], value(s, still)),
                CENTRE,
                UNIT,
                4,
                3.0,
                green,
                0.7,
            )
        });
        stage.add_to_scene(grid)?;
        let i_card = matrix_card(&kit, stage, [["1", "0"], ["0", "1"]], v(4.6, 1.6), 32.0, Palette::ink())?;
        let i_word = kit.text(
            stage,
            "the identity: every point stays",
            22.0,
            Palette::ink(),
            v(4.3, 2.45),
        )?;
        fade_in_all(stage, &[heading, i_card, i_word], 0.6)?;
        kit.until_sentence(stage, "10_1", 1)?;
        let zero = kit.tex(stage, r"X = 0:\ \ e^{0} = I", 30.0, blue, &[], v(4.3, 0.55))?;
        fade_in_all(stage, &[zero], 0.4)?;
        kit.until_sentence(stage, "10_1", 2)?;
        let full = kit.tex(
            stage,
            r"X = 2\pi J:\ \ \text{one full turn}",
            30.0,
            purple,
            &[],
            v(4.3, -0.15),
        )?;
        fade_in_all(stage, &[full], 0.4)?;
        let turn_trace =
            stage.always_redraw(move |s: &mut Arena| journeys(s, model::mscale(J, TAU), value(s, still), purple));
        stage.add_to_scene(turn_trace)?;
        play!(stage; tween(still, 1.0, 3.0)?);
        let home = kit.text_left(stage, "home at t = 1", 22.0, purple, v(right, -0.8))?;
        fade_in_all(stage, &[home], 0.4)?;
        kit.until_sentence(stage, "10_1", 3)?;
        let more = kit.text_left(stage, "with matrices, more", 24.0, gold, v(right, -1.35))?;
        fade_in_all(stage, &[more], 0.4)?;

        // 10_2: around ellipses; a continuous family
        kit.say(stage, "10_2")?;
        fade_out_all(stage, &[turn_trace, home, more], 0.4)?;
        stage.set_tracker_value(still, 0.0)?;
        stage.set_tracker_value(which, 1.0)?;
        let (a0, b0, t0) = shapes[0];
        let orbit = stage.always_redraw(move |s: &mut Arena| {
            let g = s.add(VMobject::new());
            for r in [0.5, 1.0, 1.5, 2.0] {
                let pts: Vec<Vec3> = (0..=80)
                    .map(|k| {
                        let th = TAU * f64::from(k) / 80.0;
                        let p = model::apply(
                            model::mmul(model::rotation(t0.to_radians()), [[a0, 0.0], [0.0, b0]]),
                            [r * th.cos() / a0, r * th.sin() / a0],
                        );
                        screen(p)
                    })
                    .collect();
                let m = s.add(polyline(pts).style(line_style(Palette::faint(), 1.6)).build());
                let _ = s.attach(g, m);
            }
            g
        });
        stage.add_to_scene(orbit)?;
        let e0 = kit.text_left(
            stage,
            &format!("ellipse {} by {}", shapes[0].0, shapes[0].1),
            22.0,
            blue,
            v(right, -0.8),
        )?;
        fade_in_all(stage, &[e0], 0.4)?;
        kit.until_sentence(stage, "10_2", 1)?;
        let x1 = gens[1];
        let trace1 = stage.always_redraw(move |s: &mut Arena| journeys(s, x1, value(s, still), blue));
        stage.add_to_scene(trace1)?;
        play!(stage; tween(still, 1.0, 3.6)?);
        let back = kit.text_left(
            stage,
            &format!("everything home at t = 1: {}", fact("matrix.ellipses_return")?),
            22.0,
            blue,
            v(right, -1.3),
        )?;
        fade_in_all(stage, &[back], 0.4)?;
        kit.until_sentence(stage, "10_2", 2)?;
        fade_out_all(stage, &[trace1, orbit], 0.3)?;
        stage.set_tracker_value(still, 0.0)?;
        stage.set_tracker_value(which, 2.0)?;
        let x2 = gens[2];
        let trace2 = stage.always_redraw(move |s: &mut Arena| journeys(s, x2, value(s, still), slide_c()));
        stage.add_to_scene(trace2)?;
        let e1 = kit.text_left(
            stage,
            &format!("ellipse {} by {}, tilted {}°", shapes[1].0, shapes[1].1, shapes[1].2),
            22.0,
            slide_c(),
            v(right, -1.8),
        )?;
        fade_in_all(stage, &[e1], 0.4)?;
        play!(stage; tween(still, 1.0, 3.2)?);
        kit.until_sentence(stage, "10_2", 3)?;
        // a continuous family: stretch the ellipse; every member returns
        fade_out_all(stage, &[trace2], 0.3)?;
        let stretch = stage.add_value_tracker(1.0);
        let family = stage.always_redraw(move |s: &mut Arena| {
            let a = value(s, stretch);
            let x = generator(a, 1.0, 15.0);
            let g = s.add(VMobject::new());
            for r in [0.6, 1.2, 1.8] {
                let pts: Vec<Vec3> = (0..=80)
                    .map(|k| model::apply(model::flow(x, f64::from(k) / 80.0), [r, 0.0]))
                    .map(screen)
                    .collect();
                let m = s.add(polyline(pts).style(line_style(gold, 2.5)).build());
                let _ = s.attach(g, m);
            }
            g
        });
        stage.add_to_scene(family)?;
        let family_word = kit.tex(
            stage,
            r"2\pi\, P J P^{-1} \ \text{for every invertible } P",
            28.0,
            gold,
            &[],
            v(3.9, -2.45),
        )?;
        fade_in_all(stage, &[family_word], 0.4)?;
        play!(stage; tween(stretch, 2.2, 2.6)?);
        play!(stage; tween(stretch, 0.6, 2.4)?);
        let infinite = kit.text_left(
            stage,
            "infinitely many real logarithms of I",
            24.0,
            gold,
            v(right, -3.05),
        )?;
        fade_in_all(stage, &[infinite], 0.4)?;

        // 10_3: which one? the principal logarithm, through eigenvalues
        kit.say(stage, "10_3")?;
        let mut clear = vec![family, family_word, infinite, e0, e1, back, zero, full, i_card, i_word];
        clear.push(grid);
        fade_out_all(stage, &clear, 0.5)?;
        let heading2 = title(&kit, stage, "The principal logarithm")?;
        swap(stage, &[heading], &[heading2], 0.4)?;
        kit.until_sentence(stage, "10_3", 1)?;
        // eigenvalues: the stretch factors along a matrix's own directions
        let stretch_m = [[1.6, 0.0], [0.0, 0.6]];
        let eig = stage.always_redraw(move |s: &mut Arena| grid_under(s, stretch_m, CENTRE, UNIT, 2, green, 0.6));
        stage.add_to_scene(eig)?;
        let along_x = seg(stage, screen([0.0, 0.0]), screen([1.6, 0.0]), gold, 5.0);
        let along_y = seg(stage, screen([0.0, 0.0]), screen([0.0, 0.6]), gold, 5.0);
        let lx = kit.text(stage, "×1.6", 22.0, gold, screen([1.6, -0.35]))?;
        let ly = kit.text(stage, "×0.6", 22.0, gold, screen([-0.45, 0.6]))?;
        let eig_word = kit.text_left(
            stage,
            "eigenvalues: stretch factors along its own directions",
            22.0,
            Palette::ink(),
            v(right - 0.3, 2.35),
        )?;
        fade_in_all(stage, &[along_x, along_y, lx, ly, eig_word], 0.5)?;
        kit.until_sentence(stage, "10_3", 2)?;
        let theorem_a = kit.tex(
            stage,
            r"\text{no eigenvalue of } A \text{ in } (-\infty, 0]",
            28.0,
            Palette::ink(),
            &[],
            v(4.0, 1.6),
        )?;
        let theorem_b = kit.tex(
            stage,
            r"\Rightarrow\ \text{exactly one } X \text{ with } e^{X} = A",
            28.0,
            Palette::ink(),
            &[],
            v(4.0, 1.05),
        )?;
        let theorem_c = kit.tex(
            stage,
            r"\text{whose eigenvalues all have } -\pi < \operatorname{Im} < \pi",
            28.0,
            Palette::ink(),
            &[],
            v(4.0, 0.5),
        )?;
        fade_in_all(stage, &[theorem_a, theorem_b, theorem_c], 0.6)?;
        kit.until_sentence(stage, "10_3", 3)?;
        let name = kit.tex(
            stage,
            r"\text{the principal logarithm } \log A \ (\text{real if } A \text{ is})",
            26.0,
            gold,
            &[],
            v(4.0, -0.05),
        )?;
        let source = precisely(&kit, stage, "Higham, Functions of Matrices (SIAM 2008), Theorem 1.31")?;
        fade_in_all(stage, &[name, source], 0.5)?;
        kit.until_sentence(stage, "10_3", 4)?;
        // the eigenvalue window: 0 inside, ±2πi outside
        let (wx, wy, per) = (5.6, -1.85, 0.24);
        let strip_top = super::draw::dashed(
            stage,
            v(wx - 1.2, wy + per * PI),
            v(wx + 1.2, wy + per * PI),
            red,
            2.0,
            10,
        )?;
        let strip_bot = super::draw::dashed(
            stage,
            v(wx - 1.2, wy - per * PI),
            v(wx + 1.2, wy - per * PI),
            red,
            2.0,
            10,
        )?;
        let inside = dot(stage, v(wx, wy), 0.09, gold);
        let up = ring(stage, v(wx, wy + per * TAU), 0.1, purple, 3.0);
        let down = ring(stage, v(wx, wy - per * TAU), 0.1, purple, 3.0);
        let up_word = kit.text(stage, "ellipse flows: ±2πi", 20.0, purple, v(wx - 1.75, wy + per * TAU))?;
        let in_word = kit.text_left(stage, "0", 20.0, gold, v(wx + 0.25, wy))?;
        let window = kit.text(stage, "window −π < Im < π", 18.0, red, v(wx - 1.75, wy + per * PI))?;
        fade_in_all(
            stage,
            &[strip_top, strip_bot, window, inside, in_word, up, down, up_word],
            0.6,
        )?;
        kit.until_sentence(stage, "10_3", 5)?;
        let log_i = kit.tex(stage, r"\log I = 0", 40.0, gold, &[], v(right + 0.4, -1.55))?;
        fade_in_all(stage, &[log_i], 0.5)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::I2;

    #[test]
    fn every_ellipse_flow_returns_home_and_is_minus_identity_halfway() {
        for (a, b, t) in [(2.0, 1.0, 0.0), (1.6, 0.7, 30.0), (0.6, 1.0, 15.0), (2.2, 1.0, 15.0)] {
            let x = generator(a, b, t);
            assert!(model::mdist(model::flow(x, 1.0), I2) < 1e-9);
            assert!(model::mdist(model::flow(x, 0.5), model::mscale(I2, -1.0)) < 1e-9);
            // eigenvalues ±2πi: trace 0, determinant 4π²
            assert!(model::trace(x).abs() < 1e-9 && (model::det(x) - 4.0 * PI * PI).abs() < 1e-9);
        }
    }

    #[test]
    fn traced_points_ride_their_own_ellipse() {
        // under 2πPJP⁻¹ the quantity |P⁻¹p| is conserved along the orbit
        let (a, b, t) = (1.6, 0.7, 30.0);
        let p = model::mmul(model::rotation(t.to_radians()), [[a, 0.0], [0.0, b]]);
        let q = model::inverse(p);
        let x = generator(a, b, t);
        for start in TRACED {
            let r0 = model::apply(q, start);
            for k in 0..=10 {
                let at = model::apply(model::flow(x, f64::from(k) / 10.0), start);
                let r = model::apply(q, at);
                assert!(((r[0].hypot(r[1])) - r0[0].hypot(r0[1])).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn the_ellipse_fact_parses() {
        assert_eq!(ellipses("2,1,0;1.6,0.7,30"), vec![(2.0, 1.0, 0.0), (1.6, 0.7, 30.0)]);
    }

    #[test]
    fn traced_journeys_stay_on_screen() {
        for (a, b, t) in [(2.0, 1.0, 0.0), (1.6, 0.7, 30.0)] {
            let x = generator(a, b, t);
            for p in TRACED {
                for k in 0..=40 {
                    let s = screen(model::apply(model::flow(x, f64::from(k) / 40.0), p));
                    assert!(s[0] > -7.1 && s[0] < 0.9 && s[1] > -3.3 && s[1] < 3.0, "{p:?} {s:?}");
                }
            }
        }
    }
}
