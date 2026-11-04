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

use std::collections::HashMap;
use std::f64::consts::{PI, TAU};

use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{Kit, Palette, Script, play, tween, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::draw::{
    answer_c, clipped_grid, dot, fact, fact_f64, fact_matrix, fade_in_all, fade_out_all, fail, flat, lap_c, line_style,
    matrix_card, number_c, pi_label, position_c, precisely, ring, seg, slide_c, swap, title, value, warn_c,
};
use crate::model::{self, J, M2};

pub struct DoingNothing {
    pub kit: Kit,
}

const SCRIPT: &str = include_str!("../../script.tsv");

/// A flow rewinding to the identity before the next one starts, seconds.
const REWIND: f64 = 0.5;
/// The shortest run a flow gets, however little of its words is left.
const MIN_RUN: f64 = 1.0;
/// The spiral's first run under "Matrix flows turn too", at its longest, and
/// the fade of its words that follows.
const FIRST_RUN: f64 = 2.6;
const SPIRAL_WORD: f64 = 0.4;
/// The rule under the turning line sits this far left of the line's zero, and
/// the ellipse flows' label this far right of 2π, so that neither prints on
/// the other.
const RULE_BACK: f64 = 0.25;
const LAP_ASIDE: f64 = 0.15;
/// The chapter's fine print, with the prefix `draw::precisely` adds counted in
/// the test below.
const TEXTBOOK: &str = "turning rate of the flow of X = imaginary part of an eigenvalue of X";
const SOURCE: &str = "no eigenvalue of A in (−∞, 0] ⇒ one logarithm has all eigenvalues −π < Im < π; real if A is (Higham; Gallier Thm 3.11)";
/// How long the fine print stays before the chapter ends: its characters at
/// fifteen a second, a proposal to judge against the voice.
const FINE_PRINT_S: f64 = 13.2;

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

/// A point of a circle of radius `r` squashed `u` of the way to the flow's
/// ellipse (half-axes `a`, `b`, tilted by `tilt` degrees): at u = 0 the circle,
/// at u = 1 the ellipse through the point `r` along the long axis.
fn squashed_point((a, b, tilt): (f64, f64, f64), u: f64, r: f64, th: f64) -> [f64; 2] {
    let short = 1.0 + (b / a - 1.0) * u;
    model::apply(model::rotation(tilt.to_radians()), [r * th.cos(), short * r * th.sin()])
}

/// Four such rings, faint.
fn squashed_rings(s: &mut Arena, shape: (f64, f64, f64), u: f64) -> Mob {
    let g = s.add(VMobject::new());
    for r in [0.5, 1.0, 1.5, 2.0] {
        let pts: Vec<Vec3> = (0..=80)
            .map(|k| screen(squashed_point(shape, u, r, TAU * f64::from(k) / 80.0)))
            .collect();
        let m = s.add(polyline(pts).style(line_style(Palette::faint(), 1.6)).build());
        let _ = s.attach(g, m);
    }
    g
}

/// A tracker's move at constant speed: the voice says "a steady rate", and
/// `tween` takes the engine's default easing.
fn steady(tracker: Mob, to: f64, run_time: f64) -> fmn::Result<AnimBuilder> {
    Ok(tracker
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(run_time),
            rate_func: Some(fmn::core::rate::linear),
            ..AnimateArgs::default()
        })?
        .set_value(to)?)
}

/// Where each sentence of a line begins, in bytes: the narrator splits after
/// `.`, `?` or `!` followed by a space.
fn sentence_offsets(text: &str) -> Vec<usize> {
    let mut at = vec![0];
    for (i, pair) in text.as_bytes().windows(2).enumerate() {
        if matches!(pair[0], b'.' | b'?' | b'!') && pair[1] == b' ' {
            at.push(i + 2);
        }
    }
    at
}

/// Seconds from a line's start to where `phrase` begins: the take's own word
/// mark when it has one there, otherwise the phrase's share of its sentence by
/// characters (an estimate, not a heard boundary). `None` if the line has no
/// such phrase, or the narrator split the line differently.
fn phrase_seconds(text: &str, starts: &[f64], len: f64, marks: &HashMap<usize, f64>, phrase: &str) -> Option<f64> {
    let byte = text.find(phrase)?;
    if let Some(&at) = marks.get(&byte) {
        return Some(at);
    }
    let offsets = sentence_offsets(text);
    if offsets.len() != starts.len() {
        return None;
    }
    let j = offsets.iter().rposition(|&at| at <= byte)?;
    let (from, to) = (offsets[j], offsets.get(j + 1).copied().unwrap_or(text.len()));
    let (t0, t1) = (starts[j], starts.get(j + 1).copied().unwrap_or(len));
    Some(t0 + (t1 - t0) * (byte - from) as f64 / (to - from).max(1) as f64)
}

/// The scene time `phrase` of line `id`, already said, begins; `None` without
/// a narrator. A phrase that cannot be timed fails the render, never a guess.
fn phrase_time(kit: &Kit, id: &str, phrase: &str) -> fmn::Result<Option<f64>> {
    let Some(narrator) = kit.narrator.as_ref() else {
        return Ok(None);
    };
    let starts = narrator.sentence_starts(id);
    let (Some(&first), Some(began)) = (starts.first(), narrator.sentence_at(id, 0)) else {
        return Ok(None);
    };
    let script = Script::parse(SCRIPT).map_err(fail)?;
    let text = script
        .text(id)
        .ok_or_else(|| fail(format!("line {id} is not in the script")))?;
    let len = narrator.length(id).unwrap_or_default();
    phrase_seconds(text, &starts, len, &narrator.phrase_marks(id), phrase)
        .map(|at| Some(began - first + at))
        .ok_or_else(|| fail(format!("line {id}: cannot time {phrase:?}")))
}

/// Wait until `phrase` of line `id` begins; nothing if that moment has passed,
/// or without a narrator.
fn until_phrase(kit: &Kit, stage: &mut Stage<'_>, id: &str, phrase: &str) -> fmn::Result<()> {
    if let Some(at) = phrase_time(kit, id, phrase)? {
        let (now, fps) = (stage.scene().time().to_f64(), stage.scene().time().fps());
        if at > now {
            wait_frames(stage, frames_covering(at - now, fps))?;
        }
    }
    Ok(())
}

/// A run time that ends `margin` seconds before scene time `at`, and is never
/// shorter than `MIN_RUN`; `fallback` when there is no time to aim for.
fn run_to(now: f64, at: Option<f64>, margin: f64, fallback: f64) -> f64 {
    at.map_or(fallback, |t| (t - now - margin).max(MIN_RUN))
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
        // "Squash the circle": rings fade in as circles and really squash
        let first_shape = shapes[0];
        let circles = squashed_rings(stage.arena_mut(), first_shape, 0.0);
        swap(stage, &[turn_trace, home, more], &[circles], 0.4)?;
        stage.set_tracker_value(still, 0.0)?;
        stage.set_tracker_value(which, 1.0)?;
        let squash = stage.add_value_tracker(0.0);
        let orbit = stage.always_redraw(move |s: &mut Arena| {
            let u = value(s, squash);
            squashed_rings(s, first_shape, u)
        });
        stage.scene_mut().remove(&[circles]);
        stage.add_to_scene(orbit)?;
        play!(stage; tween(squash, 1.0, 1.1)?);
        let e0 = kit.text_left(
            stage,
            &format!("ellipse {} by {}", shapes[0].0, shapes[0].1),
            22.0,
            blue,
            v(right, -0.8),
        )?;
        fade_in_all(stage, &[e0], 0.3)?;
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
        // a number's window is the one chapter 08 drew, closed at π: −1 has
        // principal angle π. Only the rule for flows, at 10_4, opens this end.
        let closed_r = dot(stage, turn_at(PI), 0.09, green);
        let numbers_word = kit.text(
            stage,
            "a number's principal angle: within a half turn",
            20.0,
            Palette::ink(),
            v(TURN_X, TURN_Y + 0.5),
        )?;
        fade_in_all(stage, &line, 0.4)?;
        fade_in_all(stage, &[window, open_l, closed_r, numbers_word], 0.5)?;
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
        let flows_word = kit.text(
            stage,
            "a flow's turning in one unit of time",
            20.0,
            Palette::ink(),
            v(TURN_X, TURN_Y + 0.5),
        )?;
        // the grid fades in at rest; the live one takes over, the same pixels
        let resting_grid = clipped_grid(stage.arena_mut(), model::I2, CENTRE, UNIT, 4, 3.0, green, 0.7);
        swap(stage, &[numbers_word], &[flows_word, resting_grid], 0.4)?;
        stage.scene_mut().remove(&[resting_grid]);
        stage.add_to_scene(moving)?;
        let spiral_trace = stage.always_redraw(move |s: &mut Arena| journeys(s, spiral, value(s, tt), blue));
        stage.add_to_scene(spiral_trace)?;
        // a quicker voice reaches the spiral's own words sooner: this run gives way
        let first_run = run_to(
            stage.scene().time().to_f64(),
            phrase_time(&kit, "10_3", "turns round at a steady")?,
            SPIRAL_WORD + REWIND,
            FIRST_RUN,
        )
        .min(FIRST_RUN);
        play!(stage; tween(tt, 1.0, first_run)?);
        kit.until_sentence(stage, "10_3", 3)?;
        let rate = fact_f64("matrix.flow_turning")?;
        let spiral_word = kit.text_left(
            stage,
            &format!(
                "spiral flow: turns {} radians per unit time, about {}π",
                fact("matrix.flow_turning")?,
                fact("matrix.flow_turning_pi")?
            ),
            22.0,
            blue,
            v(right, 0.35),
        )?;
        let spiral_mark = dot(stage, turn_at(rate), 0.1, blue);
        fade_in_all(stage, &[spiral_word, spiral_mark], SPIRAL_WORD)?;
        // the spiral again, steadily, under the words that describe it: it
        // stays on screen until the voice reaches the flow that does not turn
        play!(stage; tween(tt, 0.0, REWIND)?);
        until_phrase(&kit, stage, "10_3", "turns round at a steady")?;
        let sweep = run_to(
            stage.scene().time().to_f64(),
            phrase_time(&kit, "10_3", "or does not turn")?,
            0.5,
            3.4,
        );
        play!(stage; steady(tt, 1.0, sweep)?);
        fade_out_all(stage, &[spiral_trace], 0.2)?;
        play!(stage; tween(tt, 0.0, 0.3)?);
        stage.set_tracker_value(pick, 1.0)?;
        let stretch_trace =
            stage.always_redraw(move |s: &mut Arena| journeys(s, stretch_flow, value(s, tt), slide_c()));
        stage.add_to_scene(stretch_trace)?;
        until_phrase(&kit, stage, "10_3", "or does not turn")?;
        let stretch_word = kit.text_left(
            stage,
            "stretch flow: does not turn round at all",
            22.0,
            slide_c(),
            v(right, -0.1),
        )?;
        let stretch_mark = ring(stage, turn_at(0.0), 0.16, slide_c(), 3.0);
        fade_in_all(stage, &[stretch_word, stretch_mark], 0.3)?;
        let stretch_run = run_to(
            stage.scene().time().to_f64(),
            kit.narrator.as_ref().and_then(|n| n.sentence_at("10_3", 4)),
            0.2,
            2.2,
        );
        play!(stage; tween(tt, 1.0, stretch_run)?);
        kit.until_sentence(stage, "10_3", 4)?;
        fade_out_all(stage, &[stretch_trace], 0.3)?;
        play!(stage; tween(tt, 0.0, REWIND)?);
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
            v(turn_at(TAU)[0] + LAP_ASIDE, TURN_Y - 0.7),
        )?;
        fade_in_all(stage, &[lap_l, lap_r, lap_word], 0.5)?;

        // 10_4: the same rule picks one: the principal logarithm
        kit.say(stage, "10_4")?;
        let rule = kit.text(
            stage,
            "keep −π < turning < π",
            20.0,
            green,
            v(TURN_X - RULE_BACK, TURN_Y - 0.7),
        )?;
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
        // "strictly": unlike a number's window, the flows' rule shuts out π too
        until_phrase(&kit, stage, "10_4", "strictly between")?;
        let open_r = ring(stage, turn_at(PI), 0.09, red, 2.5);
        swap(stage, &[closed_r], &[open_r], 0.3)?;
        fade_in_all(stage, &crosses, 0.4)?;
        kit.until_sentence(stage, "10_4", 1)?;
        // the three flows have done their work: their marks stay on the line,
        // and the statement of the rule takes their rows
        fade_out_all(stage, &[spiral_word, stretch_word, zero_word], 0.3)?;
        let cond_a = kit.text_left(
            stage,
            "A sends no direction to 0 or to a negative multiple of itself",
            22.0,
            Palette::ink(),
            v(right, 0.25),
        )?;
        let cond_b = kit.text_left(
            stage,
            "⇒ exactly one logarithm turns by less than a half turn",
            22.0,
            Palette::ink(),
            v(right, -0.2),
        )?;
        fade_in_all(stage, &[cond_a, cond_b], 0.5)?;
        kit.until_sentence(stage, "10_4", 2)?;
        let name = kit.text_left(
            stage,
            "the principal logarithm, log A: real when A is real",
            22.0,
            gold,
            v(right, -0.75),
        )?;
        fade_in_all(stage, &[name], 0.5)?;
        kit.until_sentence(stage, "10_4", 3)?;
        let log_i = kit.tex(stage, r"\log I = 0", 40.0, gold, &[], v(right + 1.0, -1.5))?;
        fade_in_all(stage, &[log_i], 0.5)?;
        kit.until_sentence(stage, "10_4", 4)?;
        let textbook = kit.text(stage, TEXTBOOK, 18.0, Palette::dim(), v(0.0, -3.15))?;
        let source = precisely(&kit, stage, SOURCE)?;
        fade_in_all(stage, &[textbook, source], 0.5)?;
        let shown_from = stage.scene().time().to_f64();
        kit.hold(stage)?;
        // the voice may finish first; the fine print still gets its reading time
        let read_for = stage.scene().time().to_f64() - shown_from;
        if read_for < FINE_PRINT_S {
            kit.music_breath(stage, FINE_PRINT_S - read_for)?;
        }
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
        let (a, b, t): (f64, f64, f64) = (1.6, 0.7, 30.0);
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
    fn a_numbers_window_closes_at_plus_pi() {
        // −1's principal angle is +π, the end 10_3 draws closed for numbers
        assert!((model::C::new(-1.0, 0.0).log().im - PI).abs() < 1e-15);
    }

    #[test]
    fn the_rings_squash_from_circles_onto_the_flows_own_ellipses() {
        let shape = (2.0, 1.0, 0.0);
        let p = model::mmul(model::rotation(0.0), [[2.0, 0.0], [0.0, 1.0]]);
        let q = model::inverse(p);
        for k in 0..=16 {
            let th = TAU * f64::from(k) / 16.0;
            for r in [0.5, 1.0, 1.5, 2.0] {
                let start = squashed_point(shape, 0.0, r, th);
                assert!((start[0].hypot(start[1]) - r).abs() < 1e-12);
                // |P⁻¹p| is what the flow keeps, so this point rides one orbit
                let end = model::apply(q, squashed_point(shape, 1.0, r, th));
                assert!((end[0].hypot(end[1]) - r / 2.0).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn every_timed_phrase_is_in_its_line_and_they_fall_in_spoken_order() {
        let script = Script::parse(SCRIPT).unwrap();
        // the scratch voice's way: each sentence's share of the line, by characters
        let shares = |text: &str, len: f64| -> Vec<f64> {
            sentence_offsets(text)
                .iter()
                .map(|&at| len * at as f64 / text.len() as f64)
                .collect()
        };
        let none = HashMap::new();
        let text = script.text("10_3").unwrap();
        let starts = shares(text, 30.0);
        assert_eq!(starts.len(), 6);
        let turns = phrase_seconds(text, &starts, 30.0, &none, "turns round at a steady").unwrap();
        let not = phrase_seconds(text, &starts, 30.0, &none, "or does not turn").unwrap();
        assert!(starts[3] < turns && turns < not && not < starts[4], "{turns} {not}");
        let text = script.text("10_4").unwrap();
        let starts = shares(text, 30.0);
        let strictly = phrase_seconds(text, &starts, 30.0, &none, "strictly between").unwrap();
        assert!(0.0 < strictly && strictly < starts[1], "{strictly}");
    }

    #[test]
    fn a_takes_word_mark_beats_the_estimate_and_a_bad_phrase_is_refused() {
        let text = "One two three. Four five.";
        let starts = [0.0, 2.0];
        let byte = text.find("five").unwrap();
        let marks = HashMap::from([(byte, 3.25)]);
        assert_eq!(phrase_seconds(text, &starts, 4.0, &marks, "five"), Some(3.25));
        let estimate = phrase_seconds(text, &starts, 4.0, &HashMap::new(), "five").unwrap();
        assert!(2.0 < estimate && estimate < 4.0);
        assert_eq!(phrase_seconds(text, &starts, 4.0, &marks, "six"), None);
        // a narrator that splits the line differently is an error, never a guess
        assert_eq!(phrase_seconds(text, &[0.0], 4.0, &HashMap::new(), "five"), None);
    }

    #[test]
    fn a_run_ends_before_its_mark_and_is_never_too_short() {
        assert!((run_to(10.0, Some(15.0), 0.5, 9.0) - 4.5).abs() < 1e-12);
        assert_eq!(run_to(10.0, Some(10.2), 0.5, 9.0), MIN_RUN);
        assert_eq!(run_to(10.0, None, 0.5, 9.0), 9.0);
    }

    #[test]
    fn the_labels_under_the_turning_line_keep_apart() {
        // widths from preview02 frame 170: the rule 3.10 to 5.14, the label 5.03 to 6.56
        let (rule_w, lap_w) = (5.14 - 3.10, 6.56 - 5.03);
        let rule_right = TURN_X - RULE_BACK + rule_w / 2.0;
        let lap_at = turn_at(TAU)[0] + LAP_ASIDE;
        assert!(lap_at - lap_w / 2.0 - rule_right > 0.25);
        assert!(lap_at + lap_w / 2.0 < 6.86);
    }

    #[test]
    fn the_fine_print_has_no_caret_and_time_to_be_read() {
        assert!(!TEXTBOOK.contains('^') && !SOURCE.contains('^'));
        let characters = TEXTBOOK.chars().count() + "precisely: ".chars().count() + SOURCE.chars().count();
        assert!(characters as f64 / FINE_PRINT_S <= 15.0, "{characters}");
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
