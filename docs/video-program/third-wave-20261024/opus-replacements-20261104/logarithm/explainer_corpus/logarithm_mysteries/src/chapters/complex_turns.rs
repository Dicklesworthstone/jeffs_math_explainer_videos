//! Chapter 05, the exponential folds the plane: complex multiplication as
//! lengths multiplying and angles adding, ×i a quarter turn, and a
//! prediction: i × i = ? (05_1); i × i = −1 as two quarter turns, then two
//! rulers, one ×, one +, and a worked input, e^(ln 2 + iπ/2) = 2i
//! (05_2); the input strip mapped live onto the output
//! plane, horizontal lines to rays, vertical lines to circles wound again
//! and again (05_3); 0, ±2πi, 4πi all land on 1 (05_4); the prediction for
//! −1 (05_5) and its column of logarithms (05_6); every nonzero number's
//! column, and zero left out (05_7).
//!
//! The wound circle's trace is drawn a little further out on each lap so the
//! laps stay visible; the picture says so on screen (a drawing aid, not
//! geometry). Input logarithms are blue, outputs green, laps purple.
//!
//! Rebuild candidate for `mx-video-program-program-87do.63`; uncompiled,
//! unrendered, unlistened.

use std::f64::consts::{PI, TAU};

use explainer_kit::{Kit, Palette, play, tween, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::draw::{
    Plane, answer_c, arc_points, arrow, dot, fact, fact_list, fade_in_all, fade_in_lagged, fade_out_all, lap_c,
    line_style, number_c, pi_label, position_c, predict, ring, seg, slide_c, swap, title, value, warn_c,
};
use crate::model::C;

pub struct ComplexTurns {
    pub kit: Kit,
}

/// 05_1–05_2: one plane for multiplying arrows.
const ARROWS: Plane = Plane {
    centre: [-3.4, -0.9, 0.0],
    scale: 1.25,
    half_w: 3.4,
    half_h: 2.6,
};
/// 05_3 on: the input strip (x from −2 to 1.2, y from −3π to 3π) ...
const IN: Plane = Plane {
    centre: [-4.9, -0.15, 0.0],
    scale: 0.3,
    half_w: 0.6,
    half_h: 3.0,
};
const IN_X: (f64, f64) = (-2.0, 1.2);
const IN_Y: f64 = 3.0 * PI;
/// ... and the output plane it lands on.
const OUT: Plane = Plane {
    centre: [2.7, -0.15, 0.0],
    scale: 0.85,
    half_w: 3.9,
    half_h: 3.0,
};
/// The vertical input lines drawn, and their circles.
const XS: [f64; 5] = [-1.0, -0.5, 0.0, 0.5, 1.0];
/// How much further out each lap of the wound trace is drawn (drawing aid).
const LAP_SPREAD: f64 = 0.045;

/// The input kπi as a label: "0" for k = 0, else "kπi".
fn pi_i(k: i64) -> String {
    if k == 0 {
        "0".into()
    } else {
        format!("{}i", pi_label(k))
    }
}

fn deg(d: f64) -> f64 {
    d * PI / 180.0
}

/// The output of the input `z` under the exponential, on screen.
fn image(z: C) -> Vec3 {
    OUT.p(z.exp())
}

/// The trace of the input point climbing x = `x` from height `from` to `to`,
/// each lap drawn `LAP_SPREAD` (relative) further out than the one before.
fn wound(x: f64, from: f64, to: f64) -> Vec<Vec3> {
    let n = (((to - from).abs() / TAU) * 120.0).ceil().max(2.0) as usize;
    (0..=n)
        .map(|k| {
            let y = from + (to - from) * k as f64 / n as f64;
            let laps = (y - from) / TAU;
            let r = x.exp() * (1.0 + LAP_SPREAD * laps.abs());
            OUT.p(C::polar(r, y))
        })
        .collect()
}

impl SceneConstruct for ComplexTurns {
    fn name(&self) -> &str {
        "05_complex_turns"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (green, blue, gold, purple, teal, red) =
            (number_c(), position_c(), answer_c(), lap_c(), slide_c(), warn_c());
        let (z, w) = (fact_list("complex.mul_z")?, fact_list("complex.mul_w")?);
        let zw = fact_list("complex.mul_zw")?;

        // 05_1: arrows in the plane; lengths multiply, angles add; ×i turns
        kit.say(stage, "05_1")?;
        let heading = title(&kit, stage, "Numbers that turn")?;
        fade_in_all(stage, &[heading], 0.5)?;
        kit.until_sentence(stage, "05_1", 1)?;
        let axes = ARROWS.axes(stage)?;
        let unit = ARROWS.unit_circle(stage);
        let zc = C::polar(z[0], deg(z[1]));
        let z_arrow = arrow(stage, ARROWS.p(C::new(0.0, 0.0)), ARROWS.p(zc), green, 4.0)?;
        let z_word = kit.text(stage, "z", 28.0, green, ARROWS.p(zc.scale(1.12)))?;
        fade_in_all(stage, &[axes, unit], 0.5)?;
        fade_in_all(stage, &[z_arrow, z_word], 0.5)?;
        kit.until_sentence(stage, "05_1", 2)?;
        let wc = C::polar(w[0], deg(w[1]));
        let pc = C::polar(zw[0], deg(zw[1]));
        let w_arrow = arrow(stage, ARROWS.p(C::new(0.0, 0.0)), ARROWS.p(wc), teal, 4.0)?;
        let w_word = kit.text(stage, "w", 28.0, teal, ARROWS.p(wc.scale(1.12)))?;
        let o = ARROWS.centre;
        let arcs = vec![
            super::draw::path(stage, arc_points(o, 0.45, 0.0, deg(z[1]), 20), green, 2.5),
            super::draw::path(stage, arc_points(o, 0.62, 0.0, deg(w[1]), 20), teal, 2.5),
        ];
        fade_in_all(stage, &[w_arrow, w_word], 0.5)?;
        fade_in_all(stage, &arcs, 0.4)?;
        let p_arrow = arrow(stage, ARROWS.p(C::new(0.0, 0.0)), ARROWS.p(pc), gold, 4.5)?;
        let p_word = kit.text(stage, "zw", 28.0, gold, ARROWS.p(pc.scale(1.12)))?;
        let p_arc = super::draw::path(stage, arc_points(o, 0.8, 0.0, deg(zw[1]), 30), gold, 2.5);
        let lengths = kit.text_left(
            stage,
            &format!("lengths multiply:  {} × {} = {}", z[0], w[0], zw[0]),
            26.0,
            Palette::ink(),
            v(1.0, 1.6),
        )?;
        let angles = kit.text_left(
            stage,
            &format!("angles add:  {}° + {}° = {}°", z[1], w[1], zw[1]),
            26.0,
            Palette::ink(),
            v(1.0, 1.0),
        )?;
        fade_in_all(stage, &[p_arrow, p_word, p_arc], 0.6)?;
        fade_in_all(stage, &[lengths, angles], 0.5)?;
        kit.until_sentence(stage, "05_1", 3)?;
        let iz = zc.mul(C::new(0.0, 1.0));
        let i_arrow = arrow(stage, ARROWS.p(C::new(0.0, 0.0)), ARROWS.p(iz), purple, 4.0)?;
        let i_word = kit.text(stage, "iz", 28.0, purple, ARROWS.p(iz.scale(1.15)))?;
        let quarter = super::draw::path(
            stage,
            arc_points(o, 1.0, deg(z[1]), deg(z[1]) + PI / 2.0, 24),
            purple,
            3.0,
        );
        let times_i = kit.text_left(stage, "×i: same length, a quarter turn", 26.0, purple, v(1.0, 0.4))?;
        fade_in_all(stage, &[quarter, i_arrow, i_word, times_i], 0.6)?;
        kit.until_sentence(stage, "05_1", 4)?;
        // a prediction: i itself is a quarter turn of 1, so what is i × i?
        let i_dot = dot(stage, ARROWS.p(C::new(0.0, 1.0)), 0.09, gold);
        let i_name = kit.text_left(stage, "i", 26.0, gold, ARROWS.p(C::new(0.12, 1.15)))?;
        let ask_i = kit.text_left(stage, "i × i = ?", 26.0, gold, v(1.0, -0.2))?;
        fade_in_all(stage, &[i_dot, i_name, ask_i], 0.4)?;
        predict(&kit, stage, v(2.5, -0.62), 3.0)?;

        // 05_2: i × i = −1, two quarter turns; then two rulers — length
        // multiplies (×), angle adds (+); a worked input: ln 2 + iπ/2 → 2i
        kit.say(stage, "05_2")?;
        let r1 = ARROWS.scale;
        let i_turns = vec![
            super::draw::path(stage, arc_points(o, r1, 0.0, PI / 2.0, 24), gold, 4.0),
            super::draw::path(stage, arc_points(o, r1, PI / 2.0, PI, 24), gold, 4.0),
        ];
        let minus_one = fact("complex.i_times_i")?;
        let m1_dot = dot(
            stage,
            ARROWS.p(C::new(fact_list("complex.i_times_i")?[0], 0.0)),
            0.11,
            gold,
        );
        let answer_i = kit.text_left(
            stage,
            &format!(
                "i × i = {}: two quarter turns, a half turn",
                super::draw::shown(minus_one)
            ),
            26.0,
            gold,
            v(1.0, -0.2),
        )?;
        fade_in_lagged(stage, &i_turns, 1.2, 0.6)?;
        fade_in_all(stage, &[m1_dot], 0.3)?;
        swap(stage, &[ask_i], &[answer_i], 0.3)?;
        let left_x = 1.0;
        kit.until_sentence(stage, "05_2", 2)?;
        fade_out_all(stage, &[answer_i], 0.3)?;
        let per_ln = 6.5;
        let len_y = -0.45;
        let len_spine = seg(
            stage,
            v(left_x, len_y),
            v(left_x + per_ln * 0.75, len_y),
            Palette::faint(),
            2.5,
        );
        let len_a = seg(
            stage,
            v(left_x, len_y),
            v(left_x + per_ln * z[0].ln(), len_y),
            green,
            7.0,
        );
        let len_b = seg(
            stage,
            v(left_x + per_ln * z[0].ln(), len_y),
            v(left_x + per_ln * zw[0].ln(), len_y),
            teal,
            7.0,
        );
        let len_word = kit.text_left(
            stage,
            "length ruler (logarithm): the ×",
            22.0,
            blue,
            v(left_x, len_y - 0.4),
        )?;
        fade_in_all(stage, &[len_spine, len_word], 0.4)?;
        fade_in_lagged(stage, &[len_a, len_b], 1.0, 0.5)?;
        kit.until_sentence(stage, "05_2", 3)?;
        let per_deg = 0.055;
        let ang_y = -1.45;
        let ang_spine = seg(
            stage,
            v(left_x, ang_y),
            v(left_x + per_deg * 90.0, ang_y),
            Palette::faint(),
            2.5,
        );
        let ang_a = seg(stage, v(left_x, ang_y), v(left_x + per_deg * z[1], ang_y), green, 7.0);
        let ang_b = seg(
            stage,
            v(left_x + per_deg * z[1], ang_y),
            v(left_x + per_deg * zw[1], ang_y),
            teal,
            7.0,
        );
        let ang_word = kit.text_left(stage, "angle ruler: plain +", 22.0, purple, v(left_x, ang_y - 0.4))?;
        fade_in_all(stage, &[ang_spine, ang_word], 0.4)?;
        fade_in_lagged(stage, &[ang_a, ang_b], 1.0, 0.5)?;
        kit.until_sentence(stage, "05_2", 4)?;
        let both = kit.tex(
            stage,
            r"e^{x+iy} = e^{x}\,(\cos y + i \sin y)",
            36.0,
            Palette::ink(),
            &[(r"e^{x}", blue), (r"y", purple)],
            v(3.6, 2.35),
        )?;
        let heading2 = title(&kit, stage, "The complex exponential")?;
        swap(stage, &[heading], &[heading2], 0.4)?;
        fade_in_all(stage, &[both], 0.6)?;
        kit.until_sentence(stage, "05_2", 6)?;
        let sets_len = kit.text_left(stage, "x sets the length, e^x", 24.0, blue, v(left_x, -2.35))?;
        fade_in_all(stage, &[sets_len], 0.4)?;
        kit.until_sentence(stage, "05_2", 7)?;
        let sets_ang = kit.text_left(stage, "y sets the angle", 24.0, purple, v(left_x, -2.85))?;
        fade_in_all(stage, &[sets_ang], 0.4)?;
        kit.until_sentence(stage, "05_2", 8)?;
        // worked: x = ln 2 gives length 2, y = π/2 a quarter turn: out comes 2i
        let worked = fact_list("complex.worked")?;
        let two_i = C::polar(worked[0], deg(worked[1]));
        let len = super::draw::shown(&format!("{}", worked[0]));
        let example = kit.tex(
            stage,
            &format!(r"e^{{\ln {len} + i\pi/2}} = {len}i"),
            34.0,
            blue,
            &[],
            v(3.9, 1.3),
        )?;
        let example_word = kit.text_left(
            stage,
            &format!("length e^(ln {len}) = {len}, angle a quarter turn"),
            22.0,
            Palette::ink(),
            v(left_x, 0.55),
        )?;
        let two_i_arrow = arrow(stage, ARROWS.p(C::new(0.0, 0.0)), ARROWS.p(two_i), blue, 4.5)?;
        let two_i_word = kit.text_left(
            stage,
            &format!("{len}i"),
            26.0,
            blue,
            ARROWS.p(two_i.add(C::new(0.15, 0.0))),
        )?;
        swap(stage, &[lengths, angles, times_i], &[example, example_word], 0.4)?;
        fade_in_all(stage, &[two_i_arrow, two_i_word], 0.5)?;

        // 05_3: the grid — the input strip, and where its lines go
        kit.say(stage, "05_3")?;
        let mut old = vec![
            axes, unit, z_arrow, z_word, w_arrow, w_word, p_arrow, p_word, p_arc, i_arrow, i_word,
        ];
        old.extend(i_turns.iter().copied());
        old.extend([i_dot, i_name, m1_dot, example, example_word, two_i_arrow, two_i_word]);
        old.extend(arcs.iter().copied());
        old.extend([quarter, len_spine, len_a, len_b, len_word, ang_spine, ang_a, ang_b]);
        old.extend([ang_word, sets_len, sets_ang]);
        fade_out_all(stage, &old, 0.6)?;
        fade_out_all(stage, &[both], 0.4)?;
        kit.until_sentence(stage, "05_3", 1)?;
        let mut strip = Vec::new();
        for k in -6..=6 {
            let y = f64::from(k) * PI / 2.0;
            strip.push(seg(
                stage,
                IN.p(C::new(IN_X.0, y)),
                IN.p(C::new(IN_X.1, y)),
                Palette::faint(),
                1.6,
            ));
        }
        for &x in &XS {
            strip.push(seg(
                stage,
                IN.p(C::new(x, -IN_Y)),
                IN.p(C::new(x, IN_Y)),
                Palette::faint(),
                1.6,
            ));
        }
        for k in [-3i64, -2, -1, 1, 2, 3] {
            let at = IN.p(C::new(IN_X.0, k as f64 * PI));
            strip.push(kit.text(stage, &pi_label(k), 18.0, Palette::dim(), v(at[0] - 0.4, at[1]))?);
        }
        let in_word = kit.text(stage, "input x + iy", 22.0, blue, v(IN.centre[0], -3.25))?;
        let out_axes = OUT.axes(stage)?;
        let out_word = kit.text(stage, "output", 22.0, green, v(OUT.centre[0] + 2.6, -3.25))?;
        fade_in_all(stage, &strip, 0.6)?;
        fade_in_all(stage, &[in_word, out_axes, out_word], 0.5)?;
        kit.until_sentence(stage, "05_3", 2)?;
        // one horizontal line, swept: its image runs out along one ray
        let h_y = PI / 4.0;
        let sweep = stage.add_value_tracker(IN_X.0);
        let h_line = seg(stage, IN.p(C::new(IN_X.0, h_y)), IN.p(C::new(IN_X.1, h_y)), teal, 3.5);
        fade_in_all(stage, &[h_line], 0.3)?;
        let h_dot = stage.always_redraw(move |s: &mut Arena| {
            let x = value(s, sweep);
            let a = s.add(
                Dot::new()
                    .point(IN.p(C::new(x, h_y)))
                    .radius(0.07)
                    .style(super::draw::flat(teal, 1.0))
                    .build(),
            );
            let b = s.add(
                Dot::new()
                    .point(image(C::new(x, h_y)))
                    .radius(0.08)
                    .style(super::draw::flat(teal, 1.0))
                    .build(),
            );
            let ray = s.add(
                polyline(vec![
                    image(C::new(IN_X.0, h_y)),
                    image(C::new(x.max(IN_X.0 + 1e-3), h_y)),
                ])
                .style(line_style(teal, 3.5))
                .build(),
            );
            let g = s.add(VMobject::new());
            let _ = s.attach(g, ray);
            let _ = s.attach(g, a);
            let _ = s.attach(g, b);
            g
        });
        stage.add_to_scene(h_dot)?;
        play!(stage; tween(sweep, IN_X.1, 2.2)?);
        // and every horizontal line: a ray (lines 2π apart share one ray)
        let mut rays = Vec::new();
        for k in 0..8 {
            let y = f64::from(k) * PI / 4.0;
            rays.push(seg(
                stage,
                image(C::new(IN_X.0, y)),
                image(C::new(IN_X.1, y)),
                green,
                1.8,
            ));
        }
        fade_in_lagged(stage, &rays, 1.0, 0.15)?;
        kit.until_sentence(stage, "05_3", 3)?;
        // one vertical line, x = 0: its image is the unit circle
        let climb = stage.add_value_tracker(0.0);
        let v_line = seg(stage, IN.p(C::new(0.0, -IN_Y)), IN.p(C::new(0.0, IN_Y)), blue, 3.5);
        fade_in_all(stage, &[v_line], 0.3)?;
        let climber = stage.always_redraw(move |s: &mut Arena| {
            let y = value(s, climb);
            let g = s.add(VMobject::new());
            let a = s.add(
                Dot::new()
                    .point(IN.p(C::new(0.0, y)))
                    .radius(0.08)
                    .style(super::draw::flat(blue, 1.0))
                    .build(),
            );
            let b = s.add(
                Dot::new()
                    .point(image(C::new(0.0, y)))
                    .radius(0.09)
                    .style(super::draw::flat(green, 1.0))
                    .build(),
            );
            if y > 1e-3 {
                let t = s.add(polyline(wound(0.0, 0.0, y)).style(line_style(green, 3.0)).build());
                let _ = s.attach(g, t);
            }
            let _ = s.attach(g, a);
            let _ = s.attach(g, b);
            g
        });
        stage.add_to_scene(climber)?;
        play!(stage; tween(climb, TAU, 2.6)?);
        kit.until_sentence(stage, "05_3", 4)?;
        let closes = kit.text(
            stage,
            "the circle closes after one lap",
            22.0,
            Palette::ink(),
            v(OUT.centre[0], 2.6),
        )?;
        fade_in_all(stage, &[closes], 0.4)?;
        kit.until_sentence(stage, "05_3", 5)?;
        let laps_word = kit.text_left(stage, "laps:", 24.0, purple, v(OUT.centre[0] + 1.9, 1.7))?;
        let laps = kit.live_value(
            stage,
            move |s| (value(s, climb) / TAU).floor(),
            0,
            false,
            26.0,
            purple,
            v(OUT.centre[0] + 2.7, 1.7),
            LEFT,
        )?;
        let aid = kit.text(
            stage,
            "each lap drawn a little further out, to keep the laps apart",
            18.0,
            Palette::dim(),
            v(OUT.centre[0], -2.8),
        )?;
        fade_in_all(stage, &[laps_word, aid], 0.4)?;
        play!(stage; tween(climb, 3.0 * PI, 2.2)?);
        // every vertical line wraps its own circle
        let mut circles = Vec::new();
        for &x in &XS {
            circles.push(ring(stage, OUT.centre, OUT.scale * x.exp(), green, 1.8));
        }
        fade_in_lagged(stage, &circles, 1.2, 0.15)?;

        // 05_4: 0, 2πi, 4πi and −2πi all land on 1
        kit.say(stage, "05_4")?;
        fade_out_all(stage, &[closes, h_line, h_dot], 0.3)?;
        let one = OUT.p(C::new(1.0, 0.0));
        let one_dot = dot(stage, one, 0.11, gold);
        let one_word = kit.text(stage, "1", 26.0, gold, v(one[0] + 0.3, one[1] - 0.3))?;
        let ones = fact_list("complex.one_logs_pi")?;
        let mut lands = Vec::new();
        let mut marks = Vec::new();
        // in the order the narration names them: 0, 2π, then 4π, −2π
        let spoken: Vec<f64> = [1usize, 2, 3, 0].iter().filter_map(|&i| ones.get(i).copied()).collect();
        for &k in &spoken {
            let at = IN.p(C::new(0.0, k * PI));
            if at[1] > IN.centre[1] + IN.half_h + 0.01 {
                // 4πi sits above the strip: an arrow out of its top says so
                let top = IN.p(C::new(0.0, IN_Y));
                marks.push(super::draw::arrow(
                    stage,
                    v(top[0], top[1] - 0.25),
                    v(top[0], top[1] + 0.12),
                    purple,
                    3.0,
                )?);
                marks.push(kit.text(
                    stage,
                    &format!("{} above", pi_i(k as i64)),
                    18.0,
                    purple,
                    v(top[0] + 0.95, top[1]),
                )?);
                continue;
            }
            marks.push(dot(stage, at, 0.09, purple));
            marks.push(kit.text(stage, &pi_i(k as i64), 18.0, purple, v(at[0] + 0.85, at[1]))?);
            lands.push(at);
        }
        fade_in_all(stage, &[one_dot, one_word], 0.4)?;
        fade_in_lagged(stage, &marks[..4], 0.8, 0.3)?;
        let mut links = Vec::new();
        for &at in &lands {
            links.push(seg(stage, v(at[0] + 0.1, at[1]), v(one[0] - 0.1, one[1]), purple, 1.5));
        }
        fade_in_lagged(stage, &links[..2.min(links.len())], 0.8, 0.3)?;
        kit.until_sentence(stage, "05_4", 1)?;
        fade_in_lagged(stage, &marks[4..], 0.8, 0.3)?;
        if links.len() > 2 {
            fade_in_lagged(stage, &links[2..], 0.6, 0.3)?;
        }
        kit.until_sentence(stage, "05_4", 2)?;
        let bracket_a = IN.p(C::new(0.55, 0.0));
        let bracket_b = IN.p(C::new(0.55, TAU));
        let bracket = super::draw::path(
            stage,
            vec![
                v(bracket_a[0] - 0.08, bracket_a[1]),
                bracket_a,
                bracket_b,
                v(bracket_b[0] - 0.08, bracket_b[1]),
            ],
            purple,
            3.0,
        );
        let lap_word = kit.text_left(
            stage,
            "2π up = one lap round",
            20.0,
            purple,
            v(bracket_a[0] + 0.2, (bracket_a[1] + bracket_b[1]) / 2.0),
        )?;
        fade_in_all(stage, &[bracket, lap_word], 0.5)?;

        // 05_5: which inputs land on −1?
        kit.say(stage, "05_5")?;
        let mut tidy = marks.clone();
        tidy.extend(links.iter().copied());
        tidy.extend([bracket, lap_word, one_dot, one_word, climber, laps, laps_word, aid]);
        fade_out_all(stage, &tidy, 0.5)?;
        kit.until_sentence(stage, "05_5", 1)?;
        let minus = OUT.p(C::new(-1.0, 0.0));
        let target = dot(stage, minus, 0.12, gold);
        let target_word = kit.text(stage, "−1", 28.0, gold, v(minus[0] - 0.35, minus[1] + 0.35))?;
        let which = kit.text(stage, "which inputs?", 24.0, gold, v(IN.centre[0] + 2.2, 2.25))?;
        fade_in_all(stage, &[target, target_word, which], 0.5)?;
        predict(&kit, stage, v(OUT.centre[0], -2.75), 3.0)?;

        // 05_6: length 1 → real part 0; angle π + laps → a column
        kit.say(stage, "05_6")?;
        let len_one = ring(stage, OUT.centre, OUT.scale, blue, 3.5);
        let real_zero = kit.text(stage, "length 1 → x = ln 1 = 0", 20.0, blue, v(IN.centre[0] + 2.4, 1.6))?;
        fade_in_all(stage, &[len_one, real_zero], 0.5)?;
        kit.until_sentence(stage, "05_6", 1)?;
        let half = seg(stage, OUT.p(C::new(0.0, 0.0)), minus, green, 4.0);
        let pi_line = seg(stage, IN.p(C::new(IN_X.0, PI)), IN.p(C::new(IN_X.1, PI)), green, 3.5);
        let angle_word = kit.text(stage, "angle π", 20.0, green, v(IN.centre[0] + 2.4, 1.05))?;
        fade_in_all(stage, &[half, pi_line, angle_word], 0.5)?;
        kit.until_sentence(stage, "05_6", 2)?;
        let mut more = Vec::new();
        for k in [-3i64, -1, 3] {
            more.push(super::draw::dashed(
                stage,
                IN.p(C::new(IN_X.0, k as f64 * PI)),
                IN.p(C::new(IN_X.1, k as f64 * PI)),
                green,
                2.5,
                6,
            )?);
        }
        fade_in_lagged(stage, &more, 0.9, 0.3)?;
        kit.until_sentence(stage, "05_6", 3)?;
        let column = fact_list("complex.minus_one_logs_pi")?;
        let mut col = Vec::new();
        for &k in &column {
            let at = IN.p(C::new(0.0, k * PI));
            col.push(dot(stage, at, 0.1, purple));
            col.push(kit.text(stage, &pi_i(k as i64), 18.0, purple, v(at[0] + 0.85, at[1]))?);
        }
        fade_in_lagged(stage, &col, 1.4, 0.2)?;
        let apart = kit.text(stage, "2π apart", 20.0, purple, v(IN.centre[0] + 2.4, 0.5))?;
        fade_in_all(stage, &[apart], 0.4)?;
        kit.until_sentence(stage, "05_6", 4)?;
        let each = kit.tex(
            stage,
            r"e^{(\pi + 2\pi k) i} = -1",
            30.0,
            gold,
            &[],
            v(IN.centre[0] + 2.6, -0.1),
        )?;
        fade_in_all(stage, &[each], 0.5)?;

        // 05_7: every nonzero number has a column; zero has none
        kit.say(stage, "05_7")?;
        let generic = fact_list("complex.generic")?;
        let (glen, gang) = (generic[0], generic[1] * PI);
        let gz = C::polar(glen, gang);
        let g_dot = dot(stage, OUT.p(gz), 0.11, gold);
        let g_word = kit.text(
            stage,
            &format!("length {}, angle {}π", glen, generic[1]),
            20.0,
            gold,
            v(OUT.p(gz)[0] - 0.4, OUT.p(gz)[1] + 0.4),
        )?;
        let g_ray = seg(stage, OUT.p(C::new(0.0, 0.0)), OUT.p(gz), gold, 3.0);
        let mut olds = col.clone();
        olds.extend(more.iter().copied());
        olds.extend([
            target,
            target_word,
            which,
            len_one,
            v_line,
            real_zero,
            half,
            pi_line,
            angle_word,
            apart,
            each,
        ]);
        fade_out_all(stage, &olds, 0.5)?;
        fade_in_all(stage, &[g_ray, g_dot, g_word], 0.5)?;
        kit.until_sentence(stage, "05_7", 1)?;
        let x = glen.ln();
        let g_line = seg(stage, IN.p(C::new(x, -IN_Y)), IN.p(C::new(x, IN_Y)), blue, 3.0);
        let mut gcol = Vec::new();
        for k in -2..=1 {
            let y = gang + TAU * f64::from(k);
            if y.abs() <= IN_Y {
                gcol.push(dot(stage, IN.p(C::new(x, y)), 0.1, purple));
            }
        }
        let g_label = kit.text(
            stage,
            &format!("real part ln {} = {}", glen, fact("complex.generic_ln_len")?),
            20.0,
            blue,
            v(IN.centre[0] + 2.5, 1.6),
        )?;
        let g_laps = kit.text(stage, "angles a lap apart", 20.0, purple, v(IN.centre[0] + 2.5, 1.1))?;
        fade_in_all(stage, &[g_line, g_label], 0.5)?;
        fade_in_lagged(stage, &gcol, 0.9, 0.3)?;
        fade_in_all(stage, &[g_laps], 0.4)?;
        kit.until_sentence(stage, "05_7", 2)?;
        let zero = ring(stage, OUT.centre, 0.16, red, 4.0);
        let zero_word = kit.text(
            stage,
            "0: no logarithm",
            22.0,
            red,
            v(OUT.centre[0] + 1.25, OUT.centre[1] - 0.45),
        )?;
        fade_in_all(stage, &[zero, zero_word], 0.5)?;
        kit.until_sentence(stage, "05_7", 3)?;
        let never = kit.tex(
            stage,
            r"|e^{x+iy}| = e^{x} > 0",
            30.0,
            red,
            &[],
            v(IN.centre[0] + 2.6, 0.4),
        )?;
        fade_in_all(stage, &[never], 0.5)?;
        kit.until_sentence(stage, "05_7", 4)?;
        let dial = kit.text(stage, "a dial that forgets laps", 26.0, purple, v(OUT.centre[0], 2.6))?;
        fade_in_all(stage, &[dial], 0.5)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn horizontal_lines_go_to_rays_and_vertical_lines_to_circles() {
        for y in [0.3, PI / 4.0, 2.0] {
            let a = C::new(-1.0, y).exp();
            let b = C::new(0.7, y).exp();
            assert!((a.arg() - b.arg()).abs() < 1e-12);
        }
        for x in XS {
            for y in [-5.0, 0.0, 2.0, 9.0] {
                assert!((C::new(x, y).exp().abs() - x.exp()).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn the_column_of_minus_one_lies_on_the_strip_and_lands_on_minus_one() {
        for k in [-3.0, -1.0, 1.0, 3.0] {
            let z = C::new(0.0, k * PI);
            assert!(z.im.abs() <= IN_Y + 1e-12);
            let e = z.exp();
            assert!((e.re + 1.0).abs() < 1e-12 && e.im.abs() < 1e-12);
        }
        // and 4πi is above the strip, so the scene points to it instead
        assert!(4.0 * PI > IN_Y);
    }

    #[test]
    fn the_panels_fit_the_frame_and_do_not_overlap() {
        let in_right = IN.centre[0] + IN.scale * IN_X.1;
        let out_left = OUT.centre[0] - OUT.scale * 1f64.exp() * (1.0 + LAP_SPREAD * 1.5);
        assert!(in_right + 2.5 < out_left, "{in_right} {out_left}");
        assert!(IN.centre[0] + IN.scale * IN_X.0 - 0.6 > -7.1);
        assert!(OUT.centre[0] + OUT.scale * 1f64.exp() < 7.1);
        assert!(IN.centre[1] + IN.scale * IN_Y < 3.0);
    }

    #[test]
    fn the_wound_trace_starts_on_the_circle_and_spreads_only_a_little() {
        let t = wound(0.0, 0.0, 3.0 * PI);
        let first = t[0];
        assert!((first[0] - (OUT.centre[0] + OUT.scale)).abs() < 1e-12);
        let last = t[t.len() - 1];
        let r = ((last[0] - OUT.centre[0]).powi(2) + (last[1] - OUT.centre[1]).powi(2)).sqrt() / OUT.scale;
        assert!((r - (1.0 + LAP_SPREAD * 1.5)).abs() < 1e-9);
    }

    #[test]
    fn i_times_i_and_the_worked_input_match_their_words() {
        // 05_2 says "i × i = −1" and draws the dot at −1
        assert_eq!(fact("complex.i_times_i").unwrap(), "-1");
        assert_eq!(fact_list("complex.i_times_i").unwrap(), vec![-1.0]);
        // the worked line's words: "ln 2 + iπ/2", "length 2", "a quarter turn", "2i"
        let worked = fact_list("complex.worked").unwrap();
        assert_eq!(worked, vec![2.0, 90.0]);
        assert_eq!(format!("{}", worked[0]), "2");
        let out = C::new(2f64.ln(), PI / 2.0).exp();
        let drawn = C::polar(worked[0], deg(worked[1]));
        assert!((out.re - drawn.re).abs() < 1e-12 && (out.im - drawn.im).abs() < 1e-12);
        // both arrows' tips stay inside the panel
        assert!(ARROWS.shows(drawn) && ARROWS.shows(C::new(-1.0, 0.0)));
    }
}
