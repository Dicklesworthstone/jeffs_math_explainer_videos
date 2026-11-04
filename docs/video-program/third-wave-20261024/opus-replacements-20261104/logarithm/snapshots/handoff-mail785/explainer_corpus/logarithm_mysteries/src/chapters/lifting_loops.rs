//! Chapter 06, carrying a logarithm around a loop: the number walks the unit
//! circle while its logarithm, carried continuously, climbs the column
//! (06_1, 06_2, with a prediction before the return); it comes back 2πi
//! higher, and again, and down (06_3); a loop off to one side returns its
//! logarithm, and the winding number counts the laps (06_4); the proof that
//! no continuous logarithm exists on a loop around zero, run on a "gap in
//! laps" gauge that can only sit on whole rungs (06_5, 06_6).
//!
//! Every path is the real lift (`model::lift`) of a sampled loop, so the
//! climbing picture is computed, not drawn by hand.
//!
//! Rebuild candidate for `mx-video-program-program-87do.63`; uncompiled,
//! unrendered, unlistened.

use std::f64::consts::{PI, TAU};

use explainer_kit::{Kit, Palette, play, tween, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::draw::{
    Plane, answer_c, dot, fact, fact_list, fade_in_all, fade_out_all, flat, lap_c, line_style, number_c, pi_label,
    position_c, predict, ring, seg, slide_c, swap, title, value, warn_c,
};
use crate::model::{self, C};

pub struct LiftingLoops {
    pub kit: Kit,
}

/// The number's plane.
const Z: Plane = Plane {
    centre: [-4.1, -0.3, 0.0],
    scale: 1.25,
    half_w: 2.9,
    half_h: 2.7,
};
/// The logarithm's plane, tall: zero near the bottom so two laps fit.
const W: Plane = Plane {
    centre: [0.9, -2.7, 0.0],
    scale: 0.38,
    half_w: 0.7,
    half_h: 5.0,
};
/// The logarithm's plane for the side loop (06_4), close up.
const W2: Plane = Plane {
    centre: [2.6, -0.3, 0.0],
    scale: 1.3,
    half_w: 2.0,
    half_h: 1.6,
};
/// The gap gauge of 06_5–06_6: rungs for whole laps.
const GAUGE_X: f64 = 5.6;
const GAUGE_Y: f64 = -0.3;
const RUNG: f64 = 0.85;
const SAMPLES: usize = 240;

fn unit_point(turn: f64) -> C {
    C::polar(1.0, TAU * turn)
}

/// The lift of the unit circle walked from 1 for `turn` laps (either way),
/// as `model::lift` computes it from samples.
fn unit_lift(turn: f64) -> Vec<C> {
    let n = ((turn.abs() * SAMPLES as f64).ceil() as usize).max(2);
    let path: Vec<C> = (0..=n).map(|k| unit_point(turn * k as f64 / n as f64)).collect();
    model::lift(&path)
}

fn rung(k: f64) -> Vec3 {
    v(GAUGE_X, GAUGE_Y + RUNG * k)
}

impl SceneConstruct for LiftingLoops {
    fn name(&self) -> &str {
        "06_lifting_loops"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (green, blue, gold, purple, teal, red) =
            (number_c(), position_c(), answer_c(), lap_c(), slide_c(), warn_c());

        // 06_1: one, its column of logarithms; choose 0 and start walking
        kit.say(stage, "06_1")?;
        let heading = title(&kit, stage, "Carry the logarithm around")?;
        let z_axes = Z.axes(stage)?;
        let z_circle = Z.unit_circle(stage);
        let z_word = kit.text(stage, "the number z", 22.0, green, v(Z.centre[0], -3.25))?;
        let w_axis = seg(
            stage,
            W.p(C::new(0.0, -0.4)),
            W.p(C::new(0.0, 4.4 * PI)),
            Palette::faint(),
            2.5,
        );
        let w_base = seg(
            stage,
            W.p(C::new(-1.2, 0.0)),
            W.p(C::new(1.2, 0.0)),
            Palette::faint(),
            2.5,
        );
        let mut ticks = vec![w_axis, w_base];
        for k in 1..=4i64 {
            let at = W.p(C::new(0.0, k as f64 * PI));
            ticks.push(seg(
                stage,
                v(at[0] - 0.08, at[1]),
                v(at[0] + 0.08, at[1]),
                Palette::faint(),
                2.0,
            ));
            ticks.push(kit.text(
                stage,
                &format!("{}i", pi_label(k)),
                18.0,
                Palette::dim(),
                v(at[0] - 0.55, at[1]),
            )?);
        }
        let w_word = kit.text(stage, "its logarithm", 22.0, blue, v(W.centre[0] + 0.3, -3.25))?;
        fade_in_all(stage, &[heading, z_axes, z_circle, z_word], 0.6)?;
        fade_in_all(stage, &ticks, 0.5)?;
        fade_in_all(stage, &[w_word], 0.4)?;
        let one = dot(stage, Z.p(C::new(1.0, 0.0)), 0.1, gold);
        let mut column = Vec::new();
        for k in 0..=2 {
            column.push(dot(stage, W.p(C::new(0.0, TAU * f64::from(k))), 0.09, purple));
        }
        let which = kit.text_left(
            stage,
            "logarithms of 1: which one?",
            22.0,
            purple,
            v(W.centre[0] + 0.5, W.p(C::new(0.0, TAU))[1]),
        )?;
        fade_in_all(stage, &[one], 0.4)?;
        fade_in_all(stage, &column, 0.5)?;
        fade_in_all(stage, &[which], 0.4)?;
        kit.until_sentence(stage, "06_1", 1)?;
        let chosen = ring(stage, W.p(C::new(0.0, 0.0)), 0.17, gold, 3.0);
        fade_out_all(stage, &[which], 0.3)?;
        fade_in_all(stage, &[chosen], 0.4)?;
        let turn = stage.add_value_tracker(0.0);
        let walk = stage.always_redraw(move |s: &mut Arena| {
            let t = value(s, turn);
            let g = s.add(VMobject::new());
            let lifted = unit_lift(t);
            let n = lifted.len();
            let z_trail: Vec<Vec3> = (0..n).map(|k| Z.p(unit_point(t * k as f64 / (n - 1) as f64))).collect();
            let w_trail: Vec<Vec3> = lifted.iter().map(|&w| W.p(w)).collect();
            for (pts, c) in [(z_trail, green), (w_trail, blue)] {
                let m = s.add(polyline(pts).style(line_style(c, 3.5)).build());
                let _ = s.attach(g, m);
            }
            let here = lifted[n - 1];
            let a = s.add(
                Dot::new()
                    .point(Z.p(unit_point(t)))
                    .radius(0.1)
                    .style(flat(green, 1.0))
                    .build(),
            );
            let b = s.add(Dot::new().point(W.p(here)).radius(0.1).style(flat(blue, 1.0)).build());
            let _ = s.attach(g, a);
            let _ = s.attach(g, b);
            g
        });
        stage.add_to_scene(walk)?;
        let read_x = W.centre[0] + 0.7;
        let read_y = 2.45;
        let log_word = kit.text_left(stage, "carried logarithm:", 22.0, blue, v(read_x, read_y))?;
        let log_val = kit.live_value(
            stage,
            move |s| 2.0 * value(s, turn),
            2,
            false,
            26.0,
            blue,
            v(read_x + 2.55, read_y),
            LEFT,
        )?;
        let log_unit = kit.text_left(stage, "πi", 22.0, blue, v(read_x + 3.35, read_y))?;
        fade_in_all(stage, &[log_word, log_unit], 0.4)?;
        kit.until_sentence(stage, "06_1", 2)?;
        play!(stage; tween(turn, 0.1, 1.2)?);

        // 06_2: only the angle changes, so the logarithm climbs straight up
        kit.say(stage, "06_2")?;
        let climbs = fact_list("loop.climb_pi")?;
        let straight = kit.text_left(
            stage,
            "length stays 1: real part stays 0",
            20.0,
            Palette::dim(),
            v(read_x, read_y - 0.5),
        )?;
        fade_in_all(stage, &[straight], 0.4)?;
        kit.until_sentence(stage, "06_2", 1)?;
        play!(stage; tween(turn, climbs[0] / 2.0, 1.4)?);
        let quarter = kit.text_left(
            stage,
            "πi/2",
            20.0,
            blue,
            v(
                W.p(C::new(0.0, climbs[0] * PI))[0] + 0.25,
                W.p(C::new(0.0, climbs[0] * PI))[1],
            ),
        )?;
        fade_in_all(stage, &[quarter], 0.3)?;
        kit.until_sentence(stage, "06_2", 2)?;
        play!(stage; tween(turn, climbs[1] / 2.0, 1.4)?);
        let half = kit.text_left(
            stage,
            "πi, at −1",
            20.0,
            blue,
            v(W.p(C::new(0.0, PI))[0] + 0.25, W.p(C::new(0.0, PI))[1]),
        )?;
        fade_in_all(stage, &[half], 0.3)?;
        kit.until_sentence(stage, "06_2", 3)?;
        play!(stage; tween(turn, 0.93, 1.6)?);
        let ask = kit.text_left(stage, "back at 1: logarithm = ?", 22.0, gold, v(read_x, read_y - 1.0))?;
        fade_in_all(stage, &[ask], 0.4)?;
        predict(&kit, stage, v(Z.centre[0], 2.55), 2.6)?;

        // 06_3: 2πi. The number came back; the logarithm did not.
        kit.say(stage, "06_3")?;
        play!(stage; tween(turn, climbs[2] / 2.0, 0.8)?);
        let top = W.p(C::new(0.0, TAU));
        let answer = kit.text_left(stage, "2πi", 26.0, gold, v(top[0] + 0.25, top[1]))?;
        swap(stage, &[ask], &[answer], 0.4)?;
        kit.until_sentence(stage, "06_3", 1)?;
        let back = ring(stage, Z.p(C::new(1.0, 0.0)), 0.2, green, 3.0);
        let back_word = kit.text(
            stage,
            "came back",
            20.0,
            green,
            v(Z.p(C::new(1.0, 0.0))[0] + 0.85, Z.centre[1] - 0.35),
        )?;
        fade_in_all(stage, &[back, back_word], 0.4)?;
        kit.until_sentence(stage, "06_3", 2)?;
        let not_back = ring(stage, top, 0.2, red, 3.0);
        fade_in_all(stage, &[not_back], 0.4)?;
        kit.until_sentence(stage, "06_3", 3)?;
        let base = W.p(C::new(0.0, 0.0));
        let lap = super::draw::path(
            stage,
            vec![
                v(base[0] - 0.35, base[1]),
                v(base[0] - 0.45, base[1]),
                v(top[0] - 0.45, top[1]),
                v(top[0] - 0.35, top[1]),
            ],
            purple,
            3.0,
        );
        let lap_word = kit.text(
            stage,
            "one lap",
            20.0,
            purple,
            v(base[0] - 1.05, (base[1] + top[1]) / 2.0),
        )?;
        fade_in_all(stage, &[lap, lap_word], 0.5)?;
        kit.until_sentence(stage, "06_3", 4)?;
        fade_out_all(stage, &[not_back, back, back_word, quarter, half], 0.3)?;
        play!(stage; tween(turn, 2.0, 3.0)?);
        kit.until_sentence(stage, "06_3", 5)?;
        play!(stage; tween(turn, 1.0, 2.2)?);

        // 06_4: a loop off to one side returns; winding number counts laps
        kit.say(stage, "06_4")?;
        let mut tall = ticks.clone();
        tall.extend(column.iter().copied());
        tall.extend([
            chosen, walk, log_word, log_val, log_unit, straight, answer, lap, lap_word, w_word,
        ]);
        fade_out_all(stage, &tall, 0.5)?;
        let w2_axes = W2.axes(stage)?;
        let w2_word = kit.text(stage, "its logarithm, close up", 22.0, blue, v(W2.centre[0], -3.25))?;
        fade_in_all(stage, &[w2_axes, w2_word], 0.5)?;
        kit.until_sentence(stage, "06_4", 1)?;
        let side = fact_list("loop.side")?;
        let ring_path = model::circle_path(C::new(side[0], 0.0), side[1], 1.0, SAMPLES);
        let side_lift = model::lift(&ring_path);
        let side_circle = ring(stage, Z.p(C::new(side[0], 0.0)), Z.scale * side[1], teal, 2.5);
        fade_in_all(stage, &[side_circle], 0.4)?;
        let progress = stage.add_value_tracker(0.0);
        let side_walk = stage.always_redraw(move |s: &mut Arena| {
            let f = value(s, progress).clamp(0.0, 1.0);
            let upto = ((f * SAMPLES as f64).round() as usize).clamp(1, ring_path.len() - 1);
            let g = s.add(VMobject::new());
            let zt: Vec<Vec3> = ring_path[..=upto].iter().map(|&z| Z.p(z)).collect();
            let wt: Vec<Vec3> = side_lift[..=upto].iter().map(|&w| W2.p(w)).collect();
            for (pts, c) in [(zt, teal), (wt, blue)] {
                let m = s.add(polyline(pts).style(line_style(c, 3.5)).build());
                let _ = s.attach(g, m);
            }
            let a = s.add(
                Dot::new()
                    .point(Z.p(ring_path[upto]))
                    .radius(0.1)
                    .style(flat(teal, 1.0))
                    .build(),
            );
            let b = s.add(
                Dot::new()
                    .point(W2.p(side_lift[upto]))
                    .radius(0.1)
                    .style(flat(blue, 1.0))
                    .build(),
            );
            let _ = s.attach(g, a);
            let _ = s.attach(g, b);
            g
        });
        stage.add_to_scene(side_walk)?;
        play!(stage; tween(progress, 1.0, 4.0)?);
        let returns = kit.text_left(stage, "the logarithm returns: change 0", 22.0, blue, v(0.9, 2.2))?;
        fade_in_all(stage, &[returns], 0.4)?;
        kit.until_sentence(stage, "06_4", 2)?;
        let unit_w = fact("loop.unit_winding")?;
        let side_w = fact("loop.side_winding")?;
        let counts = kit.text_left(
            stage,
            &format!("times round 0:  unit circle {unit_w},  side circle {side_w}"),
            22.0,
            Palette::ink(),
            v(0.9, 1.6),
        )?;
        fade_in_all(stage, &[counts], 0.5)?;
        kit.until_sentence(stage, "06_4", 3)?;
        let rule = kit.tex(
            stage,
            r"\text{change in the logarithm} = 2\pi i \times \text{winding number}",
            30.0,
            gold,
            &[],
            v(1.0, -2.6),
        )?;
        fade_in_all(stage, &[rule], 0.5)?;

        // 06_5: suppose a continuous logarithm L existed on the circle
        kit.say(stage, "06_5")?;
        fade_out_all(
            stage,
            &[side_walk, side_circle, returns, counts, rule, w2_axes, w2_word],
            0.5,
        )?;
        let heading2 = title(&kit, stage, "Could one logarithm work all the way round?")?;
        swap(stage, &[heading], &[heading2], 0.4)?;
        fade_in_all(stage, &ticks, 0.4)?;
        let suppose = kit.text_left(
            stage,
            "suppose L is a continuous logarithm on the circle",
            22.0,
            teal,
            v(read_x - 0.2, 2.45),
        )?;
        fade_in_all(stage, &[suppose], 0.5)?;
        kit.until_sentence(stage, "06_5", 1)?;
        stage.set_tracker_value(turn, 0.0)?;
        let ours = kit.text_left(stage, "ours: carried along", 22.0, blue, v(read_x - 0.2, 1.95))?;
        // theirs, while it shadows ours: drawn just right of our path
        let theirs = stage.always_redraw(move |s: &mut Arena| {
            let t = value(s, turn).max(1e-3);
            let pts: Vec<Vec3> = unit_lift(t)
                .iter()
                .map(|&w| {
                    let q = W.p(w);
                    v(q[0] + 0.16, q[1])
                })
                .collect();
            s.add(polyline(pts).style(line_style(teal, 2.5)).build())
        });
        fade_in_all(stage, &[ours], 0.4)?;
        let walk2 = stage.always_redraw(move |s: &mut Arena| {
            let t = value(s, turn).max(1e-3);
            let g = s.add(VMobject::new());
            let w_trail: Vec<Vec3> = unit_lift(t).iter().map(|&w| W.p(w)).collect();
            let m = s.add(polyline(w_trail).style(line_style(blue, 3.5)).build());
            let a = s.add(
                Dot::new()
                    .point(Z.p(unit_point(t)))
                    .radius(0.1)
                    .style(flat(green, 1.0))
                    .build(),
            );
            let _ = s.attach(g, m);
            let _ = s.attach(g, a);
            g
        });
        stage.add_to_scene(walk2)?;
        stage.add_to_scene(theirs)?;
        kit.until_sentence(stage, "06_5", 2)?;
        let differ = kit.tex(
            stage,
            r"e^{L(z)} = z = e^{\ell(z)} \;\Rightarrow\; L(z) - \ell(z) = 2\pi i\,k,\ \ k \text{ whole}",
            28.0,
            Palette::ink(),
            &[(r"L(z)", teal), (r"\ell(z)", blue)],
            v(1.2, -3.0),
        )?;
        fade_in_all(stage, &[differ], 0.5)?;
        kit.until_sentence(stage, "06_5", 3)?;
        let mut gauge = vec![seg(stage, rung(-2.3), rung(1.3), Palette::faint(), 2.5)];
        for k in -2..=1 {
            gauge.push(seg(
                stage,
                v(GAUGE_X - 0.25, rung(f64::from(k))[1]),
                v(GAUGE_X + 0.25, rung(f64::from(k))[1]),
                Palette::dim(),
                3.0,
            ));
            gauge.push(kit.text(
                stage,
                &super::draw::shown(&k.to_string()),
                20.0,
                Palette::dim(),
                v(GAUGE_X + 0.55, rung(f64::from(k))[1]),
            )?);
        }
        gauge.push(kit.text(stage, "gap in laps, k", 20.0, teal, v(GAUGE_X, rung(1.75)[1]))?);
        let marker = dot(stage, rung(0.0), 0.13, teal);
        fade_in_all(stage, &gauge, 0.5)?;
        fade_in_all(stage, &[marker], 0.3)?;
        // it may only sit on a rung: any move between rungs would be a jump
        let jump = kit.text(
            stage,
            "whole rungs only",
            18.0,
            Palette::dim(),
            v(GAUGE_X, rung(-2.75)[1]),
        )?;
        fade_in_all(stage, &[jump], 0.4)?;
        kit.until_sentence(stage, "06_5", 4)?;
        let fixed = kit.text(stage, "fixed", 22.0, teal, v(GAUGE_X - 0.8, rung(0.0)[1]))?;
        fade_in_all(stage, &[fixed], 0.3)?;
        play!(stage; tween(turn, 0.97, 4.0)?);

        // 06_6: back at one, theirs must return; ours is 2πi higher
        kit.say(stage, "06_6")?;
        play!(stage; tween(turn, 1.0, 0.6)?);
        kit.until_sentence(stage, "06_6", 1)?;
        let start = W.p(C::new(0.0, 0.0));
        let theirs_end = dot(stage, v(start[0] + 0.16, start[1]), 0.12, teal);
        let theirs_word = kit.text_left(
            stage,
            "L(1) has one value: where it began",
            20.0,
            teal,
            v(start[0] + 0.4, start[1] + 0.35),
        )?;
        fade_in_all(stage, &[theirs_end, theirs_word], 0.5)?;
        kit.until_sentence(stage, "06_6", 2)?;
        let ours_end = ring(stage, top, 0.2, blue, 3.5);
        let ours_word = kit.text_left(stage, "ours: 2πi higher", 20.0, blue, v(top[0] + 0.4, top[1]))?;
        fade_in_all(stage, &[ours_end, ours_word], 0.5)?;
        kit.until_sentence(stage, "06_6", 3)?;
        let would = super::draw::arrow(
            stage,
            v(GAUGE_X - 0.3, rung(0.0)[1] - 0.1),
            v(GAUGE_X - 0.3, rung(-1.0)[1] + 0.12),
            red,
            3.5,
        )?;
        let would_word = kit.text(
            stage,
            "k would have to jump",
            20.0,
            red,
            v(GAUGE_X - 1.6, rung(-0.5)[1]),
        )?;
        fade_out_all(stage, &[fixed], 0.2)?;
        fade_in_all(stage, &[would, would_word], 0.5)?;
        kit.until_sentence(stage, "06_6", 4)?;
        let contradiction = kit.text(stage, "contradiction", 30.0, red, v(read_x + 1.4, 1.4))?;
        fade_in_all(stage, &[contradiction], 0.4)?;
        kit.until_sentence(stage, "06_6", 5)?;
        let theorem = kit.text(
            stage,
            "no continuous logarithm on a loop that winds around 0",
            26.0,
            gold,
            v(0.6, 0.8),
        )?;
        fade_in_all(stage, &[theorem], 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_carried_logarithm_climbs_two_pi_per_lap_and_back() {
        for (t, want) in [(0.25, PI / 2.0), (0.5, PI), (1.0, TAU), (2.0, 2.0 * TAU), (-1.0, -TAU)] {
            let w = *unit_lift(t).last().expect("samples");
            assert!(w.re.abs() < 1e-9 && (w.im - want).abs() < 1e-6, "{t}: {:?}", w);
        }
    }

    #[test]
    fn the_side_loop_returns_its_logarithm() {
        let path = model::circle_path(C::new(1.7, 0.0), 0.8, 1.0, SAMPLES);
        let lifted = model::lift(&path);
        let (a, b) = (lifted[0], lifted[lifted.len() - 1]);
        assert!((a.re - b.re).abs() < 1e-9 && (a.im - b.im).abs() < 1e-9);
        assert_eq!(model::winding(&path), 0);
        // and the close-up panel holds its whole lift
        for w in lifted {
            assert!((w.re * W2.scale).abs() < W2.half_w && (w.im * W2.scale).abs() < W2.half_h);
        }
    }

    #[test]
    fn two_laps_fit_the_tall_panel() {
        let top = W.p(C::new(0.0, 2.0 * TAU));
        assert!(top[1] < 2.3, "{}", top[1]);
        assert!(W.p(C::new(0.0, 0.0))[1] > -3.0);
        // the side circle stays inside the number's panel
        assert!(Z.centre[0] + Z.scale * (1.7 + 0.8) < W.centre[0] - 0.3);
    }
}
