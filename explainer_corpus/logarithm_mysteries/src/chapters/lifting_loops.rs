//! Chapter 06, carrying a logarithm around a loop: the number walks the unit
//! circle while its logarithm, carried continuously, climbs the column
//! (06_1, 06_2, with a prediction before the return); it comes back 2πi
//! higher, and again, and down (06_3); a loop off to one side, walked a
//! third of the way, then a prediction: will its logarithm come back
//! (06_4)? it does, and the winding number counts the laps (06_5); a
//! concrete candidate first, the angle kept in (−π, π], whose gap from ours
//! jumps a lap just after −1 on a "gap in laps" gauge, then the proof that
//! no continuous logarithm exists on a loop around zero, the gap sitting
//! on one whole rung (06_6, 06_7). Mail 803 repair 5.
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
    Plane, answer_c, arrow_parts, dot, fact, fact_list, fade_in_all, fade_out_all, flat, lap_c, line_style, number_c,
    pi_label, position_c, predict, ring, seg, slide_c, swap, title, value, warn_c,
};
use crate::model::{self, C};

pub struct LiftingLoops {
    pub kit: Kit,
}

/// The number's plane.
const Z: Plane = Plane {
    centre: [-4.1, -0.3, 0.0],
    scale: 1.25,
    // the axis's left end stays inside the safe area, at -6.90
    half_w: 2.8,
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
/// The tall axis's tick labels; its "0" sits a little further out, clear of
/// the base line.
const TICK_LABEL_X: f64 = W.centre[0] - 0.55;
const ZERO_TICK_X: f64 = W.centre[0] - 0.6;
/// The lap bracket and its word, left of the tick labels, so that the word can
/// never read as part of a label.
const LAP_STUB_X: f64 = W.centre[0] - 0.8;
const LAP_BRACKET_X: f64 = W.centre[0] - 0.9;
const LAP_WORD_X: f64 = W.centre[0] - 1.35;
/// 06_6's inset, the candidate's own jump: its axis stands beside the tall
/// one, at the same scale, raised so that a true 2π stays on screen.
const JUMP_X: f64 = 1.95;
const JUMP_ZERO_Y: f64 = -1.3;
/// A trail shorter than this many laps is not drawn: it would only be a speck.
const MIN_TRAIL: f64 = 0.005;
const THEOREM: &str = "no continuous logarithm on a loop that winds around 0";
/// Ours carries no `(z)`: it is the value carried along the walk, and back at 1
/// it is not the value it began with. Only L is a function of the point.
const DIFFER: &str = r"e^{L(z)} = z = e^{\ell} \;\Rightarrow\; L(z) - \ell = 2\pi i\,k,\ \ k \text{ whole}";

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

/// 06_6's candidate, the logarithm whose angle is kept in (−π, π], minus
/// our carried one, in whole laps, a fraction `turn` of the way round from
/// 1: 0 up to and including −1 (angle π by convention), −1 just after.
fn principal_gap(turn: f64) -> f64 {
    let ours = TAU * turn;
    ((model::wrap(ours) - ours) / TAU).round()
}

/// The tall axis's label for kπi.
fn tick_label(kit: &Kit, stage: &mut Stage<'_>, k: i64) -> fmn::Result<Mob> {
    let at = W.p(C::new(0.0, k as f64 * PI));
    kit.text(
        stage,
        &format!("{}i", pi_label(k)),
        18.0,
        Palette::dim(),
        v(TICK_LABEL_X, at[1]),
    )
}

/// The tall axis's label for its base, the logarithm the walk starts from.
fn zero_tick_label(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    kit.text(stage, "0", 18.0, Palette::dim(), v(ZERO_TICK_X, W.centre[1]))
}

/// "one lap", beside its bracket and level with the bracket's middle.
fn lap_word_label(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    let middle = (W.p(C::new(0.0, 0.0))[1] + W.p(C::new(0.0, TAU))[1]) / 2.0;
    kit.text(stage, "one lap", 20.0, lap_c(), v(LAP_WORD_X, middle))
}

/// The tall axis's name, kept near its top once the proof needs the space
/// below it.
fn tall_axis_word(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    kit.text(
        stage,
        "its logarithm",
        22.0,
        position_c(),
        v(W.centre[0] - 1.0, W.p(C::new(0.0, 4.4 * PI))[1]),
    )
}

/// "0 is not inside", under the side circle it is about: under the unit circle
/// it would read as a claim about that one, which does hold 0.
fn outside_word(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    kit.text_left(
        stage,
        "0 is not inside",
        20.0,
        slide_c(),
        v(Z.centre[0] + 1.5, Z.centre[1] - 1.6),
    )
}

/// The point 0 itself, which every loop is now measured against: a small dot and
/// its name, inside the unit circle and clear of both axes. It stays to the end,
/// because the closing theorem is about loops that wind around it.
fn zero_mark(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<(Mob, Mob)> {
    let at = Z.p(C::new(0.0, 0.0));
    let mark = dot(stage, at, 0.06, Palette::ink());
    let word = kit.text(stage, "0", 20.0, Palette::ink(), v(at[0] - 0.2, at[1] - 0.22))?;
    Ok((mark, word))
}

/// The ends of the candidate's jump, πi and −πi, on the inset's own axis.
fn jump_ends() -> (Vec3, Vec3) {
    (
        v(JUMP_X, JUMP_ZERO_Y + W.scale * PI),
        v(JUMP_X, JUMP_ZERO_Y - W.scale * PI),
    )
}

/// 06_6's words for the candidate's jump, from the fact behind them.
fn jump_words() -> fmn::Result<String> {
    let gaps = fact_list("loop.principal_gap")?;
    Ok(format!(
        "candidate: πi → −πi, gap {} → {}",
        super::draw::shown(&format!("{}", gaps[0])),
        super::draw::shown(&format!("{}", gaps[1]))
    ))
}

/// The inset: a short axis of its own marked πi, 0 and −πi, the jump's arrow
/// from πi down to −πi, a true 2π at the tall axis's scale, and its words.
/// Returns the arrow, the words and the axis with its marks.
fn jump_inset(kit: &Kit, stage: &mut Stage<'_>, words: &str) -> fmn::Result<(Mob, Mob, Vec<Mob>)> {
    let (top, bottom) = jump_ends();
    let mut axis = vec![seg(
        stage,
        v(JUMP_X, top[1] + 0.12),
        v(JUMP_X, bottom[1] - 0.12),
        Palette::faint(),
        2.5,
    )];
    for (y, label) in [(top[1], "πi"), (JUMP_ZERO_Y, "0"), (bottom[1], "−πi")] {
        axis.push(seg(
            stage,
            v(JUMP_X - 0.08, y),
            v(JUMP_X + 0.08, y),
            Palette::faint(),
            2.0,
        ));
        axis.push(kit.text(stage, label, 18.0, Palette::dim(), v(JUMP_X - 0.4, y))?);
    }
    let jump = arrow_parts(stage.arena_mut(), top, bottom, warn_c(), 3.5, 0.2);
    let word = kit.text_left(stage, words, 20.0, warn_c(), v(JUMP_X + 0.25, JUMP_ZERO_Y))?;
    Ok((jump, word, axis))
}

/// The closing theorem, under the proof's formula and clear of the tall axis.
fn theorem_line(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    kit.text(stage, THEOREM, 26.0, answer_c(), v(1.2, -3.55))
}

/// Two logarithms of one number differ by whole laps.
fn differ_line(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    kit.tex(
        stage,
        DIFFER,
        28.0,
        Palette::ink(),
        &[(r"L(z)", slide_c()), (r"\ell", position_c())],
        v(1.2, -3.0),
    )
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
            ticks.push(tick_label(&kit, stage, k)?);
        }
        ticks.push(zero_tick_label(&kit, stage)?);
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
        // "as the number turns": it keeps turning under the words, short of the quarter
        play!(stage; tween(turn, 0.2, 2.4)?);
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
        let answer = kit.text_left(stage, "2πi", 26.0, gold, v(top[0] + 0.3, top[1]))?;
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
                v(LAP_STUB_X, base[1]),
                v(LAP_BRACKET_X, base[1]),
                v(LAP_BRACKET_X, top[1]),
                v(LAP_STUB_X, top[1]),
            ],
            purple,
            3.0,
        );
        let lap_word = lap_word_label(&kit, stage)?;
        fade_in_all(stage, &[lap, lap_word], 0.5)?;
        kit.until_sentence(stage, "06_3", 4)?;
        fade_out_all(stage, &[not_back, back, back_word, quarter, half], 0.3)?;
        play!(stage; tween(turn, 2.0, 3.0)?);
        kit.until_sentence(stage, "06_3", 5)?;
        play!(stage; tween(turn, 1.0, 2.2)?);

        // 06_4: a loop off to one side; part way round, a prediction
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
        let outside = outside_word(&kit, stage)?;
        // the gold dot leaves: this walk starts elsewhere, on the side circle
        let (zero_dot, zero_word) = zero_mark(&kit, stage)?;
        swap(stage, &[one], &[side_circle, outside, zero_dot, zero_word], 0.4)?;
        kit.until_sentence(stage, "06_4", 2)?;
        let lift_start = side_lift[0];
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
        let start_mark = ring(stage, W2.p(lift_start), 0.16, gold, 3.0);
        fade_in_all(stage, &[start_mark], 0.3)?;
        // a third of the way round: the logarithm has wandered off its start
        play!(stage; tween(progress, 0.35, 1.8)?);
        kit.until_sentence(stage, "06_4", 3)?;
        let ask = kit.text_left(stage, "back where it began?", 22.0, gold, v(0.9, 2.2))?;
        fade_in_all(stage, &[ask], 0.4)?;
        predict(&kit, stage, v(2.4, 1.8), 2.6)?;

        // 06_5: yes; the winding number counts the laps
        kit.say(stage, "06_5")?;
        play!(stage; tween(progress, 1.0, 2.8)?);
        let returns = kit.text_left(stage, "the logarithm returns: change 0", 22.0, blue, v(0.9, 2.2))?;
        swap(stage, &[ask], &[returns], 0.4)?;
        kit.until_sentence(stage, "06_5", 1)?;
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
        kit.until_sentence(stage, "06_5", 2)?;
        let rule = kit.tex(
            stage,
            r"\text{change in the logarithm} = 2\pi i \times \text{winding number}",
            30.0,
            gold,
            &[],
            v(1.0, -2.6),
        )?;
        fade_in_all(stage, &[rule], 0.5)?;

        // 06_6: a concrete candidate, the angle kept in (−π, π], breaks at −1;
        // then any continuous L: its gap from ours sits on one whole rung
        kit.say(stage, "06_6")?;
        fade_out_all(
            stage,
            &[
                side_walk,
                side_circle,
                outside,
                start_mark,
                returns,
                counts,
                rule,
                w2_axes,
                w2_word,
            ],
            0.5,
        )?;
        let heading2 = title(&kit, stage, "Could one logarithm work all the way round?")?;
        swap(stage, &[heading], &[heading2], 0.4)?;
        let tall_word = tall_axis_word(&kit, stage)?;
        let mut again = ticks.clone();
        again.extend([one, tall_word]);
        fade_in_all(stage, &again, 0.4)?;
        kit.until_sentence(stage, "06_6", 1)?;
        stage.set_tracker_value(turn, 0.0)?;
        let candidate = kit.text_left(
            stage,
            "candidate: keep the angle between −π and π",
            22.0,
            teal,
            v(read_x - 0.2, 2.45),
        )?;
        let ours = kit.text_left(stage, "ours: carried along", 22.0, blue, v(read_x - 0.2, 1.95))?;
        fade_in_all(stage, &[candidate, ours], 0.4)?;
        let walk2 = stage.always_redraw(move |s: &mut Arena| {
            let t = value(s, turn).max(1e-3);
            let g = s.add(VMobject::new());
            if t > MIN_TRAIL {
                let w_trail: Vec<Vec3> = unit_lift(t).iter().map(|&w| W.p(w)).collect();
                let m = s.add(polyline(w_trail).style(line_style(blue, 3.5)).build());
                let _ = s.attach(g, m);
            }
            let a = s.add(
                Dot::new()
                    .point(Z.p(unit_point(t)))
                    .radius(0.1)
                    .style(flat(green, 1.0))
                    .build(),
            );
            let _ = s.attach(g, a);
            g
        });
        stage.add_to_scene(walk2)?;
        kit.until_sentence(stage, "06_6", 2)?;
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
        fade_in_all(stage, &gauge, 0.5)?;
        // the candidate's gap from ours, live: 0 until the walk passes −1
        let cand_marker = stage.always_redraw(move |s: &mut Arena| {
            let k = principal_gap(value(s, turn));
            s.add(Dot::new().point(rung(k)).radius(0.13).style(flat(teal, 1.0)).build())
        });
        stage.add_to_scene(cand_marker)?;
        play!(stage; tween(turn, 0.45, 2.6)?);
        kit.until_sentence(stage, "06_6", 3)?;
        play!(stage; tween(turn, 0.6, 1.6)?);
        let minus_one = ring(stage, Z.p(C::new(-1.0, 0.0)), 0.2, red, 3.5);
        let (jump, jump_word, jump_axis) = jump_inset(&kit, stage, &jump_words()?)?;
        let mut drop = jump_axis;
        drop.extend([jump, jump_word]);
        let mut breaking = vec![minus_one];
        breaking.extend(drop.iter().copied());
        fade_in_all(stage, &breaking, 0.4)?;
        play!(stage; tween(turn, 0.97, 1.8)?);
        kit.until_sentence(stage, "06_6", 4)?;
        let breaks = kit.text_left(stage, "it breaks at −1", 22.0, red, v(read_x - 0.2, 1.45))?;
        fade_in_all(stage, &[breaks], 0.4)?;
        kit.say(stage, "06_20")?;
        breaking.extend([cand_marker, candidate, breaks]);
        fade_out_all(stage, &breaking, 0.4)?;
        // our walk runs back to the start, so nothing jumps in one frame
        play!(stage; tween(turn, 0.0, 0.6)?);
        let suppose = kit.text_left(
            stage,
            "suppose L is a continuous logarithm on the circle",
            22.0,
            teal,
            v(read_x - 0.2, 2.45),
        )?;
        // theirs, while it shadows ours: drawn just right of our path
        let theirs = stage.always_redraw(move |s: &mut Arena| {
            let t = value(s, turn).max(1e-3);
            let g = s.add(VMobject::new());
            if t > MIN_TRAIL {
                let pts: Vec<Vec3> = unit_lift(t)
                    .iter()
                    .map(|&w| {
                        let q = W.p(w);
                        v(q[0] + 0.16, q[1])
                    })
                    .collect();
                let m = s.add(polyline(pts).style(line_style(teal, 2.5)).build());
                let _ = s.attach(g, m);
            }
            g
        });
        stage.add_to_scene(theirs)?;
        let marker = dot(stage, rung(0.0), 0.13, teal);
        fade_in_all(stage, &[suppose, marker], 0.4)?;
        kit.until_sentence(stage, "06_20", 1)?;
        let differ = differ_line(&kit, stage)?;
        // the formula's letter for the carried value, where "ours" is written
        let ours_right = stage.get_bounding_box_point(ours, RIGHT)[0];
        let ours_name = kit.tex_left(stage, r"(\ell)", 22.0, blue, &[], v(ours_right + 0.12, 1.95))?;
        fade_in_all(stage, &[differ, ours_name], 0.5)?;
        kit.until_sentence(stage, "06_20", 2)?;
        // it may only sit on a rung: any move between rungs would be a jump
        let jump = kit.text(
            stage,
            "whole rungs only",
            18.0,
            Palette::dim(),
            v(GAUGE_X, rung(-2.75)[1]),
        )?;
        fade_in_all(stage, &[jump], 0.4)?;
        play!(stage; tween(turn, 0.97, 4.0)?);
        // the sentence's last words: it lands after the walk has shown it
        let fixed = kit.text(stage, "fixed", 22.0, teal, v(GAUGE_X - 0.8, rung(0.0)[1]))?;
        fade_in_all(stage, &[fixed], 0.3)?;

        // 06_7: back at one, theirs must return; ours is 2πi higher
        kit.say(stage, "06_7")?;
        play!(stage; tween(turn, 1.0, 0.6)?);
        kit.until_sentence(stage, "06_7", 1)?;
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
        kit.until_sentence(stage, "06_7", 2)?;
        let ours_end = ring(stage, top, 0.2, blue, 3.5);
        let ours_word = kit.text_left(stage, "ours: 2πi higher", 20.0, blue, v(top[0] + 0.4, top[1]))?;
        fade_in_all(stage, &[ours_end, ours_word], 0.5)?;
        kit.until_sentence(stage, "06_7", 3)?;
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
        kit.until_sentence(stage, "06_7", 4)?;
        let contradiction = kit.text(stage, "contradiction", 30.0, red, v(read_x + 1.4, 1.4))?;
        fade_in_all(stage, &[contradiction], 0.4)?;
        kit.until_sentence(stage, "06_7", 5)?;
        let theorem = theorem_line(&kit, stage)?;
        fade_in_all(stage, &[theorem], 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use fmn::scene::{NullSceneSink, RuntimeConfig};

    use super::*;

    /// The safe area's half width and half height.
    const SAFE: [f64; 2] = [6.96, 3.85];

    /// Run `f` as a scene's `construct`, headless, at 10 fps.
    fn run(f: impl FnMut(&mut Stage<'_>) -> fmn::Result<()>) {
        struct Probe<F>(F);
        impl<F: FnMut(&mut Stage<'_>) -> fmn::Result<()>> SceneConstruct for Probe<F> {
            fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
                (self.0)(stage)
            }
        }
        let config = RuntimeConfig {
            fps: 10,
            ..RuntimeConfig::default()
        };
        fmn::run_scene(&mut Probe(f), config, 7, &mut NullSceneSink).expect("the probe scene runs");
    }

    /// A mobject's typeset box: left, right, bottom, top.
    fn box_of(stage: &Stage<'_>, m: Mob) -> [f64; 4] {
        [
            stage.get_bounding_box_point(m, LEFT)[0],
            stage.get_bounding_box_point(m, RIGHT)[0],
            stage.get_bounding_box_point(m, DOWN)[1],
            stage.get_bounding_box_point(m, UP)[1],
        ]
    }

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
    fn the_principal_candidate_jumps_a_lap_just_after_minus_one_and_only_there() {
        // the edge: at −1 itself the angle is π, so no jump yet
        assert_eq!(principal_gap(0.5), 0.0);
        assert_eq!(principal_gap(0.5 + 1e-9), -1.0);
        for j in 0..=96 {
            let t = f64::from(j) / 96.0 * 0.97;
            let want = if t <= 0.5 { 0.0 } else { -1.0 };
            assert_eq!(principal_gap(t), want, "{t}");
        }
        assert_eq!(super::super::draw::fact("loop.principal_gap").unwrap(), "0,-1");
    }

    #[test]
    fn the_candidates_jump_is_drawn_two_pi_long_at_the_tall_axis_scale() {
        let (top, bottom) = jump_ends();
        assert!(((top[1] - bottom[1]) / W.scale - TAU).abs() < 1e-12);
        assert!(((top[1] + bottom[1]) / 2.0 - JUMP_ZERO_Y).abs() < 1e-12);
        run(|stage| {
            let kit = Kit::new()?;
            let words = jump_words()?;
            assert_eq!(words, "candidate: πi → −πi, gap 0 → −1");
            let (jump, word, axis) = jump_inset(&kit, stage, &words)?;
            // the arrow as drawn, head included, is the whole jump
            assert!((stage.get_height(jump) / W.scale - TAU).abs() < 1e-6);
            assert!((stage.get_center(jump)[0] - JUMP_X).abs() < 1e-6);
            let [word_left, word_right, _, _] = box_of(stage, word);
            assert!(
                word_right < GAUGE_X - 0.25 - 0.1,
                "the words reach the gauge: {word_right}"
            );
            assert!(word_left > JUMP_X + 0.1, "the words reach the arrow");
            for m in axis {
                let [left, right, bottom, _] = box_of(stage, m);
                assert!(left > W.centre[0] + 0.2, "the inset reaches the tall axis");
                assert!(right < word_left, "the inset reaches its words");
                assert!(bottom > -SAFE[1], "the inset leaves the safe area");
            }
            Ok(())
        });
    }

    #[test]
    fn the_lap_word_stands_outside_a_bracket_that_clears_every_tick_label() {
        run(|stage| {
            let kit = Kit::new()?;
            let zero = zero_tick_label(&kit, stage)?;
            let mut labels = vec![zero];
            for k in 1..=4 {
                labels.push(tick_label(&kit, stage, k)?);
            }
            for label in labels {
                assert!(
                    box_of(stage, label)[0] > LAP_STUB_X + 0.05,
                    "a label reaches the bracket"
                );
            }
            // the base line starts right of the "0" it belongs to
            assert!(box_of(stage, zero)[1] < W.p(C::new(-1.2, 0.0))[0] - 0.03);
            let word = lap_word_label(&kit, stage)?;
            let [left, right, _, _] = box_of(stage, word);
            assert!(right < LAP_BRACKET_X - 0.05, "the word reaches its bracket");
            assert!(
                left > Z.centre[0] + Z.half_w + 0.05,
                "the word reaches the number's panel"
            );
            Ok(())
        });
    }

    #[test]
    fn the_theorem_stands_clear_of_the_tall_axis_and_the_formula_above_it() {
        run(|stage| {
            let kit = Kit::new()?;
            let theorem = theorem_line(&kit, stage)?;
            let differ = differ_line(&kit, stage)?;
            let z_word = kit.text(stage, "the number z", 22.0, Palette::ink(), v(Z.centre[0], -3.25))?;
            let [left, right, bottom, top] = box_of(stage, theorem);
            assert!(top < box_of(stage, differ)[2], "the theorem reaches the formula");
            assert!(top < W.p(C::new(0.0, -0.4))[1], "the tall axis crosses the theorem");
            assert!(bottom > -SAFE[1] && right < SAFE[0], "the theorem leaves the safe area");
            assert!(left > box_of(stage, z_word)[1], "the theorem reaches the number's name");
            Ok(())
        });
    }

    #[test]
    fn the_side_circle_note_clears_the_axis_and_both_circles() {
        run(|stage| {
            let kit = Kit::new()?;
            let word = outside_word(&kit, stage)?;
            let [left, right, _, top] = box_of(stage, word);
            assert!(left > Z.centre[0] + 0.05, "the number's axis crosses the note");
            let side = fact_list("loop.side")?;
            let under = Z.p(C::new(side[0], 0.0))[0];
            assert!(((left + right) / 2.0 - under).abs() < 0.1, "the note leaves its circle");
            assert!(
                top < Z.centre[1] - Z.scale * side[1],
                "the side circle crosses the note"
            );
            assert!(top < Z.centre[1] - Z.scale, "the unit circle crosses the note");
            Ok(())
        });
    }

    #[test]
    fn the_tall_axis_keeps_its_name_clear_of_labels_and_panels() {
        run(|stage| {
            let kit = Kit::new()?;
            let word = tall_axis_word(&kit, stage)?;
            let four = tick_label(&kit, stage, 4)?;
            let [left, right, bottom, _] = box_of(stage, word);
            assert!(bottom > box_of(stage, four)[3], "the name reaches the top tick label");
            assert!(right < W.centre[0] - 0.1, "the name reaches the axis");
            assert!(left > Z.centre[0] + Z.half_w, "the name reaches the number's panel");
            Ok(())
        });
    }

    #[test]
    fn zero_is_marked_inside_the_unit_circle_clear_of_both_axes() {
        run(|stage| {
            let kit = Kit::new()?;
            let (_, word) = zero_mark(&kit, stage)?;
            let [left, right, bottom, top] = box_of(stage, word);
            assert!(right < Z.centre[0] - 0.03, "the upright axis reaches the name");
            assert!(top < Z.centre[1] - 0.03, "the level axis reaches the name");
            let far = (left - Z.centre[0]).hypot(bottom - Z.centre[1]);
            assert!(far < Z.scale - 0.1, "the unit circle reaches the name");
            Ok(())
        });
    }

    #[test]
    fn the_formula_gives_ours_no_argument_because_ours_is_not_a_function_of_the_point() {
        assert!(DIFFER.contains(r"L(z) - \ell ="));
        assert!(!DIFFER.contains(r"\ell("));
        // back at 1 the carried value is one lap above where it began
        let lift = unit_lift(1.0);
        let (first, last) = (lift[0], lift[lift.len() - 1]);
        assert!((first.re - last.re).abs() < 1e-9);
        assert!((last.im - first.im - TAU).abs() < 1e-9);
    }

    #[test]
    fn the_numbers_axis_stays_inside_the_safe_area() {
        const { assert!(Z.centre[0] - Z.half_w > -SAFE[0]) };
    }

    #[test]
    fn a_trail_too_short_to_draw_is_shorter_than_its_own_stroke() {
        const { assert!(W.scale * TAU * MIN_TRAIL < 0.02) };
    }

    #[test]
    fn two_laps_fit_the_tall_panel() {
        let top = W.p(C::new(0.0, 2.0 * TAU));
        assert!(top[1] < 2.3, "{}", top[1]);
        assert!(W.p(C::new(0.0, 0.0))[1] > -3.0);
        // the side circle stays inside the number's panel
        const { assert!(Z.centre[0] + Z.scale * (1.7 + 0.8) < W.centre[0] - 0.3) };
    }
}
