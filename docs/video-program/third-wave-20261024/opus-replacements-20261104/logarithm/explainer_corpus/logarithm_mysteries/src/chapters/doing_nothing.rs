//! Chapter 10, the many ways to do nothing: the identity reached by zero
//! velocity and by one full turn (10_1); flows around ellipses, every point
//! once round its own ellipse, all home at t = 1, and a continuous family
//! of them as the ellipse is stretched (10_2); which one to choose: for
//! numbers the principal angle stayed within a half turn, and flows turn
//! too, at a steady rate round stretched circles or not at all, shown on a
//! "turning line" with the spiral flow (0.872, inside), a stretch flow and
//! the zero flow (0) and the ellipse flows (±2π, outside) (10_3); the same
//! rule picks exactly one logarithm when A sends no direction to zero or to
//! a negative multiple of itself: the principal logarithm, real for real A,
//! 0 for the identity (10_4).
//!
//! Mail 803 repair 2: no "possibly complex stretch directions". The turning
//! rate is earned on screen first (`model::turning_rate`); the word
//! eigenvalue appears only in the textbook line. Statement (quotes in
//! docs/sources.md): N. J. Higham, "Functions of Matrices", MIMS EPrint
//! 2005.21, the Handbook of Linear Algebra (2006) chapter, section 4 Fact 1:
//! if A has no eigenvalues on R⁻, exactly one logarithm X has every
//! eigenvalue in −π < Im < π, and X is real when A is real (zero is excluded
//! too: the square-root fact in the same survey writes "the closed negative
//! real axis", and a singular A has no logarithm at all). The real
//! case is proved as J. Gallier, arXiv 0805.0245, Theorem 3.11(a), which he
//! says is Theorem 1.31 of Higham's SIAM 2008 book (that book not read
//! here). For real 2×2 A a real eigenvalue has a real eigenvector,
//! so "no eigenvalue in (−∞, 0]" is exactly "no direction sent to zero or to
//! a negative multiple of itself". `model::principal_log` computes it.
//!
//! Rebuild candidate for `mx-video-program-program-87do.63`; uncompiled,
//! unrendered, unlistened.

use std::f64::consts::{PI, TAU};

use explainer_kit::{Kit, Palette, play, tween, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::draw::{
    answer_c, clipped_grid, dot, fact, fact_f64, fact_matrix, fade_in_all, fade_out_all, flat, lap_c, line_style,
    matrix_card, number_c, pi_label, position_c, precisely, ring, seg, slide_c, swap, title, value, warn_c,
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

/// The turning line of 10_3/10_4: zero at `TURN_X`, `TURN_PER` scene units
/// per π of turning in one unit of time.
const TURN_X: f64 = 4.1;
const TURN_Y: f64 = 1.55;
const TURN_PER: f64 = 0.85;

fn turn_at(angle: f64) -> Vec3 {
    v(TURN_X + TURN_PER * angle / PI, TURN_Y)
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

        // 10_3: which one? numbers kept the angle within a half turn; flows turn too
        kit.say(stage, "10_3")?;
        let mut clear = vec![family, family_word, infinite, e0, e1, back, zero, full, i_card, i_word];
        clear.push(grid);
        fade_out_all(stage, &clear, 0.5)?;
        let heading2 = title(&kit, stage, "The principal logarithm")?;
        swap(stage, &[heading], &[heading2], 0.4)?;
        kit.until_sentence(stage, "10_3", 1)?;
        // the turning line, in units of π, with the window −π < angle < π
        let mut line = vec![seg(stage, turn_at(-2.3 * PI), turn_at(2.3 * PI), Palette::faint(), 2.5)];
        for k in -2..=2_i64 {
            let p = turn_at(PI * k as f64);
            line.push(seg(
                stage,
                v(p[0], p[1] - 0.09),
                v(p[0], p[1] + 0.09),
                Palette::dim(),
                2.0,
            ));
            line.push(kit.text(stage, &pi_label(k), 18.0, Palette::dim(), v(p[0], p[1] - 0.32))?);
        }
        let window = seg(stage, turn_at(-PI), turn_at(PI), green, 6.0);
        let open_l = ring(stage, turn_at(-PI), 0.09, red, 2.5);
        let open_r = ring(stage, turn_at(PI), 0.09, red, 2.5);
        let numbers_word = kit.text(
            stage,
            "a number's principal angle: within a half turn",
            20.0,
            Palette::ink(),
            v(TURN_X, TURN_Y + 0.5),
        )?;
        fade_in_all(stage, &line, 0.4)?;
        fade_in_all(stage, &[window, open_l, open_r, numbers_word], 0.5)?;
        kit.until_sentence(stage, "10_3", 2)?;
        // flows turn too: the spiral flow, a stretch flow, the zero flow
        let spiral = fact_matrix("matrix.flow")?;
        let stretch_flow: M2 = [[1.5f64.ln(), 0.0], [0.0, 0.6f64.ln()]];
        let flows: [M2; 3] = [spiral, stretch_flow, [[0.0; 2]; 2]];
        let tt = stage.add_value_tracker(0.0);
        let pick = stage.add_value_tracker(0.0);
        let moving = stage.always_redraw(move |s: &mut Arena| {
            let k = (value(s, pick).round() as usize).min(2);
            clipped_grid(s, model::flow(flows[k], value(s, tt)), CENTRE, UNIT, 4, 3.0, green, 0.7)
        });
        stage.add_to_scene(moving)?;
        let flows_word = kit.text(
            stage,
            "a flow's turning in one unit of time",
            20.0,
            Palette::ink(),
            v(TURN_X, TURN_Y + 0.5),
        )?;
        swap(stage, &[numbers_word], &[flows_word], 0.4)?;
        let spiral_trace = stage.always_redraw(move |s: &mut Arena| journeys(s, spiral, value(s, tt), blue));
        stage.add_to_scene(spiral_trace)?;
        play!(stage; tween(tt, 1.0, 2.6)?);
        kit.until_sentence(stage, "10_3", 3)?;
        let rate = fact_f64("matrix.flow_turning")?;
        let spiral_word = kit.text_left(
            stage,
            &format!(
                "spiral flow: turns {} per unit time, about {}π",
                fact("matrix.flow_turning")?,
                fact("matrix.flow_turning_pi")?
            ),
            22.0,
            blue,
            v(right, 0.35),
        )?;
        let spiral_mark = dot(stage, turn_at(rate), 0.1, blue);
        fade_in_all(stage, &[spiral_word, spiral_mark], 0.4)?;
        fade_out_all(stage, &[spiral_trace], 0.3)?;
        stage.set_tracker_value(tt, 0.0)?;
        stage.set_tracker_value(pick, 1.0)?;
        let stretch_trace =
            stage.always_redraw(move |s: &mut Arena| journeys(s, stretch_flow, value(s, tt), slide_c()));
        stage.add_to_scene(stretch_trace)?;
        play!(stage; tween(tt, 1.0, 2.2)?);
        let stretch_word = kit.text_left(
            stage,
            "stretch flow: does not turn round at all",
            22.0,
            slide_c(),
            v(right, -0.1),
        )?;
        let stretch_mark = ring(stage, turn_at(0.0), 0.16, slide_c(), 3.0);
        fade_in_all(stage, &[stretch_word, stretch_mark], 0.4)?;
        kit.until_sentence(stage, "10_3", 4)?;
        fade_out_all(stage, &[stretch_trace], 0.3)?;
        stage.set_tracker_value(tt, 0.0)?;
        stage.set_tracker_value(pick, 2.0)?;
        let zero_word = kit.text_left(stage, "zero flow: turns 0", 22.0, gold, v(right, -0.55))?;
        let zero_mark = dot(stage, turn_at(0.0), 0.09, gold);
        fade_in_all(stage, &[zero_word, zero_mark], 0.4)?;
        kit.until_sentence(stage, "10_3", 5)?;
        let lap_l = ring(stage, turn_at(-TAU), 0.11, purple, 3.0);
        let lap_r = ring(stage, turn_at(TAU), 0.11, purple, 3.0);
        let lap_word = kit.text(
            stage,
            "ellipse flows: ±2π",
            20.0,
            purple,
            v(turn_at(TAU)[0], TURN_Y - 0.7),
        )?;
        fade_in_all(stage, &[lap_l, lap_r, lap_word], 0.5)?;

        // 10_4: the same rule picks one: the principal logarithm
        kit.say(stage, "10_4")?;
        let rule = kit.text(stage, "keep −π < turning < π", 20.0, green, v(TURN_X, TURN_Y - 0.7))?;
        let mut crosses = Vec::new();
        for c in [turn_at(-TAU), turn_at(TAU)] {
            crosses.push(seg(
                stage,
                v(c[0] - 0.16, c[1] - 0.16),
                v(c[0] + 0.16, c[1] + 0.16),
                red,
                3.5,
            ));
            crosses.push(seg(
                stage,
                v(c[0] - 0.16, c[1] + 0.16),
                v(c[0] + 0.16, c[1] - 0.16),
                red,
                3.5,
            ));
        }
        fade_in_all(stage, &[rule], 0.4)?;
        fade_in_all(stage, &crosses, 0.4)?;
        kit.until_sentence(stage, "10_4", 1)?;
        let cond_a = kit.text_left(
            stage,
            "A sends no direction to 0 or to a negative multiple of itself",
            22.0,
            Palette::ink(),
            v(right, -1.05),
        )?;
        let cond_b = kit.text_left(
            stage,
            "⇒ exactly one logarithm turns by less than a half turn",
            22.0,
            Palette::ink(),
            v(right, -1.5),
        )?;
        fade_in_all(stage, &[cond_a, cond_b], 0.6)?;
        kit.until_sentence(stage, "10_4", 2)?;
        let name = kit.text_left(
            stage,
            "the principal logarithm, log A: real when A is real",
            22.0,
            gold,
            v(right, -2.0),
        )?;
        fade_in_all(stage, &[name], 0.5)?;
        kit.until_sentence(stage, "10_4", 3)?;
        let log_i = kit.tex(stage, r"\log I = 0", 40.0, gold, &[], v(right + 1.0, -2.6))?;
        fade_in_all(stage, &[log_i], 0.5)?;
        kit.until_sentence(stage, "10_4", 4)?;
        let textbook = kit.text(
            stage,
            "turning rate of e^(tX) = imaginary part of an eigenvalue of X (complex vectors allowed)",
            18.0,
            Palette::dim(),
            v(0.0, -3.15),
        )?;
        let source = precisely(
            &kit,
            stage,
            "no eigenvalue of A in (−∞, 0] ⇒ one X with e^X = A, all eigenvalues −π < Im < π; real if A is (Higham; Gallier Thm 3.11)",
        )?;
        fade_in_all(stage, &[textbook, source], 0.5)?;
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
    fn the_turning_line_places_each_flow_on_the_right_side_of_the_window() {
        let spiral = [[0.3, -1.0], [0.8, -0.1]];
        let stretch = [[1.5f64.ln(), 0.0], [0.0, 0.6f64.ln()]];
        assert!(model::turning_rate(spiral) < PI && model::turning_rate(spiral) > 0.0);
        assert!(model::turning_rate(stretch) == 0.0);
        for (a, b, t) in [(2.0, 1.0, 0.0), (1.6, 0.7, 30.0)] {
            assert!((model::turning_rate(generator(a, b, t)) - TAU).abs() < 1e-9);
        }
        // each shown flow inside the window is the principal logarithm of where it ends
        for x in [spiral, stretch] {
            let back = model::principal_log(model::flow(x, 1.0)).unwrap();
            assert!(model::mdist(back, x) < 1e-12);
        }
        // the ellipse flows end at I, whose principal logarithm is the zero flow
        assert!(model::mdist(model::principal_log(I2).unwrap(), [[0.0; 2]; 2]) < 1e-15);
        // the line fits right of the plane panel and inside the frame
        assert!(turn_at(-2.3 * PI)[0] > 1.0 && turn_at(2.3 * PI)[0] < 6.9);
    }

    #[test]
    fn the_spiral_and_stretch_journeys_stay_on_screen() {
        for x in [[[0.3, -1.0], [0.8, -0.1]], [[1.5f64.ln(), 0.0], [0.0, 0.6f64.ln()]]] {
            for p in TRACED {
                for k in 0..=40 {
                    let s = screen(model::apply(model::flow(x, f64::from(k) / 40.0), p));
                    assert!(s[0] > -7.1 && s[0] < 0.9 && s[1] > -3.3 && s[1] < 3.0, "{p:?} {s:?}");
                }
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
