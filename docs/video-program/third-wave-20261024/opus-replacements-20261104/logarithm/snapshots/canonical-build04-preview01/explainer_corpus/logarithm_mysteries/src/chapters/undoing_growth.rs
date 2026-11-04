//! Chapter 04, the logarithm on its home ground: e^x as a multiplier with a
//! live tangent whose slope equals its height (04_1); the three facts, with a
//! horizontal probe that meets the curve once for every positive height and
//! never at or below zero (04_2); ln as the reverse reading, ln 1 = 0,
//! ln e = 1, no real logarithm of zero or negatives (04_3); the reflection in
//! y = x turning slope = height into slope = 1/x, and ln 1.01 ≈ 0.01 (04_4);
//! the e-ruler and the base-2 ruler as one ruler with two step sizes (04_5);
//! the doubling time and the thousandfold time on a ratio-ruler axis, where
//! steady growth is a straight line (04_6, 04_7).
//!
//! Live readouts (tangent slope and height) are computed from `f64::exp` in
//! the scene; every fixed number is a facts.rs key.
//!
//! Rebuild candidate for `mx-video-program-program-87do.63`; uncompiled,
//! unrendered, unlistened.

use std::f64::consts::{E, PI};

use explainer_kit::{Kit, Palette, play, tween, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::draw::{
    answer_c, dashed, dot, fact, fact_f64, fade_in_all, fade_in_lagged, fade_out_all, line_style, number_c, position_c,
    precisely, predict, seg, swap, title, value, warn_c,
};

pub struct UndoingGrowth {
    pub kit: Kit,
}

/// The graph panel: scene point of (0, 0), one unit = one scene unit, the
/// same window on both axes so the mirror in y = x is a true reflection.
const OX: f64 = -3.2;
const OY: f64 = -0.4;
const LO: f64 = -2.6;
const HI: f64 = 3.0;

fn p(x: f64, y: f64) -> Vec3 {
    v(OX + x, OY + y)
}

fn inside(x: f64, y: f64) -> bool {
    (LO..=HI).contains(&x) && (LO..=HI).contains(&y)
}

/// The exponential's graph point at `x`, carried part way `m` across the
/// mirror y = x (a flip about that line seen edge-on: the point swings
/// through the line as cos(πm), so m = 1 is the exact reflection).
fn flipped(x: f64, y: f64, m: f64) -> (f64, f64) {
    let mid = (x + y) / 2.0;
    let c = (PI * m).cos();
    (mid + (x - mid) * c, mid + (y - mid) * c)
}

/// The exponential's curve, flipped by `m`, clipped to the window.
fn exp_curve(m: f64, upto: f64) -> Vec<Vec3> {
    let n = 160;
    (0..=n)
        .map(|k| LO + (upto.min(HI) - LO) * f64::from(k) / f64::from(n))
        .map(|x| flipped(x, x.exp(), m))
        .filter(|&(a, b)| inside(a, b))
        .map(|(a, b)| p(a, b))
        .collect()
}

fn axes(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let mut parts = vec![
        seg(stage, p(LO, 0.0), p(HI, 0.0), Palette::faint(), 2.5),
        seg(stage, p(0.0, LO), p(0.0, HI), Palette::faint(), 2.5),
    ];
    for k in [-2, -1, 1, 2, 3] {
        let t = f64::from(k);
        parts.push(seg(stage, p(t, -0.07), p(t, 0.07), Palette::faint(), 2.0));
        parts.push(seg(stage, p(-0.07, t), p(0.07, t), Palette::faint(), 2.0));
        let n = super::draw::shown(&k.to_string());
        parts.push(kit.text(stage, &n, 18.0, Palette::dim(), p(t, -0.3))?);
        parts.push(kit.text(stage, &n, 18.0, Palette::dim(), p(-0.3, t))?);
    }
    Ok(parts)
}

/// The growth panel of 04_6–04_7: years along, doublings up (a ratio
/// ruler standing on its end), so steady growth draws a straight line.
const GX: f64 = -5.6;
const GY: f64 = -2.8;
const PER_YEAR: f64 = 7.0 / 150.0;
const PER_DOUBLING: f64 = 0.5;
const DOUBLINGS: i32 = 10;

fn g(years: f64, doublings: f64) -> Vec3 {
    v(GX + PER_YEAR * years, GY + PER_DOUBLING * doublings)
}

impl SceneConstruct for UndoingGrowth {
    fn name(&self) -> &str {
        "04_undoing_growth"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (green, blue, gold, red) = (number_c(), position_c(), answer_c(), warn_c());

        // 04_1: e^x, a multiplier; slope equals height
        kit.say(stage, "04_1")?;
        let heading = title(&kit, stage, "The logarithm on its home ground")?;
        fade_in_all(stage, &[heading], 0.5)?;
        kit.until_sentence(stage, "04_1", 1)?;
        let frame = axes(&kit, stage)?;
        fade_in_all(stage, &frame, 0.5)?;
        let drawn = stage.add_value_tracker(LO);
        let mirror = stage.add_value_tracker(0.0);
        let curve = stage.always_redraw(move |s: &mut Arena| {
            let pts = exp_curve(value(s, mirror), value(s, drawn));
            let pts = if pts.len() < 2 { vec![p(LO, LO.exp()); 2] } else { pts };
            s.add(polyline(pts).style(line_style(green, 4.0)).build())
        });
        stage.add_to_scene(curve)?;
        play!(stage; tween(drawn, HI, 2.0)?);
        let name = kit.tex(stage, r"y = e^{x}", 34.0, green, &[], p(1.6, 2.4))?;
        fade_in_all(stage, &[name], 0.4)?;
        kit.until_sentence(stage, "04_1", 2)?;
        let law = kit.tex(
            stage,
            r"e^{a+b} = e^{a}\, e^{b}",
            38.0,
            Palette::ink(),
            &[],
            v(3.9, 2.2),
        )?;
        fade_in_all(stage, &[law], 0.5)?;
        kit.until_sentence(stage, "04_1", 3)?;
        let at_one = dot(stage, p(0.0, 1.0), 0.08, gold);
        let one_word = kit.text(stage, "1 at 0", 22.0, gold, p(-0.75, 1.25))?;
        fade_in_all(stage, &[at_one, one_word], 0.4)?;
        let x0 = stage.add_value_tracker(-1.6);
        let tangent = stage.always_redraw(move |s: &mut Arena| {
            let a = value(s, x0);
            let h = a.exp();
            let d = 0.8 / (1.0 + h * h).sqrt();
            s.add(
                polyline(vec![p(a - d, h - h * d), p(a + d, h + h * d)])
                    .style(line_style(blue, 3.0))
                    .build(),
            )
        });
        let touch = stage.always_redraw(move |s: &mut Arena| {
            let a = value(s, x0);
            s.add(
                Dot::new()
                    .point(p(a, a.exp()))
                    .radius(0.07)
                    .style(super::draw::flat(blue, 1.0))
                    .build(),
            )
        });
        stage.add_to_scene(tangent)?;
        stage.add_to_scene(touch)?;
        let h_word = kit.text_left(stage, "height", 26.0, green, v(1.0, 1.2))?;
        let s_word = kit.text_left(stage, "slope", 26.0, blue, v(1.0, 0.6))?;
        let h_val = kit.live_value(
            stage,
            move |s| value(s, x0).exp(),
            3,
            false,
            28.0,
            green,
            v(2.6, 1.2),
            LEFT,
        )?;
        let s_val = kit.live_value(
            stage,
            move |s| value(s, x0).exp(),
            3,
            false,
            28.0,
            blue,
            v(2.6, 0.6),
            LEFT,
        )?;
        fade_in_all(stage, &[h_word, s_word], 0.4)?;
        play!(stage; tween(x0, 0.95, 3.2)?);
        play!(stage; tween(x0, -0.5, 1.4)?);
        kit.until_sentence(stage, "04_1", 4)?;
        let companion = kit.text_left(
            stage,
            "built from scratch in the companion film",
            22.0,
            Palette::dim(),
            v(1.0, -0.1),
        )?;
        fade_in_all(stage, &[companion], 0.4)?;

        // 04_2: never zero, never negative, always climbing: each height once
        kit.say(stage, "04_2")?;
        fade_out_all(
            stage,
            &[tangent, touch, h_word, s_word, h_val, s_val, companion, one_word],
            0.4,
        )?;
        let three = kit.text_left(stage, "three facts", 28.0, Palette::ink(), v(1.0, 1.25))?;
        fade_in_all(stage, &[three], 0.4)?;
        kit.until_sentence(stage, "04_2", 1)?;
        let f1 = kit.tex(
            stage,
            r"e^{x}\, e^{-x} = e^{0} = 1 \;\Rightarrow\; e^{x} \neq 0",
            30.0,
            Palette::ink(),
            &[],
            v(3.9, 0.6),
        )?;
        let pair_a = dot(stage, p(1.0, E), 0.07, green);
        let pair_b = dot(stage, p(-1.0, 1.0 / E), 0.07, green);
        fade_in_all(stage, &[f1, pair_a, pair_b], 0.5)?;
        kit.until_sentence(stage, "04_2", 2)?;
        let floor = dashed(stage, p(LO, 0.0), p(HI, 0.0), red, 3.0, 22)?;
        let f2 = kit.text_left(
            stage,
            "starts at 1; to go negative it would cross 0",
            22.0,
            Palette::ink(),
            v(1.0, 0.0),
        )?;
        fade_in_all(stage, &[floor, f2], 0.5)?;
        kit.until_sentence(stage, "04_2", 3)?;
        let f3 = kit.text_left(
            stage,
            "slope = height > 0: always climbing",
            22.0,
            Palette::ink(),
            v(1.0, -0.55),
        )?;
        fade_in_all(stage, &[f3], 0.4)?;
        kit.until_sentence(stage, "04_2", 4)?;
        // a horizontal probe at height c meets the curve once for c > 0
        let c = stage.add_value_tracker(2.6);
        let probe = stage.always_redraw(move |s: &mut Arena| {
            let h = value(s, c);
            let col = if h > 0.0 { gold } else { red };
            s.add(polyline(vec![p(LO, h), p(HI, h)]).style(line_style(col, 2.5)).build())
        });
        let meet = stage.always_redraw(move |s: &mut Arena| {
            let h = value(s, c);
            if h > 0.0 && inside(h.ln(), h) {
                s.add(
                    Dot::new()
                        .point(p(h.ln(), h))
                        .radius(0.09)
                        .style(super::draw::flat(gold, 1.0))
                        .build(),
                )
            } else {
                s.add(VMobject::new())
            }
        });
        let hits_word = kit.text_left(
            stage,
            "times the curve reaches this height:",
            22.0,
            Palette::ink(),
            v(1.0, -1.25),
        )?;
        let hits = kit.live_value(
            stage,
            move |s| if value(s, c) > 0.0 { 1.0 } else { 0.0 },
            0,
            false,
            28.0,
            gold,
            v(5.7, -1.25),
            LEFT,
        )?;
        stage.add_to_scene(probe)?;
        stage.add_to_scene(meet)?;
        fade_in_all(stage, &[hits_word], 0.4)?;
        play!(stage; tween(c, 0.15, 2.4)?);
        play!(stage; tween(c, -1.2, 1.4)?);
        play!(stage; tween(c, 2.0, 1.6)?);

        // 04_3: reversing it — ln, ln 1 = 0, ln e = 1, nothing for c ≤ 0
        kit.say(stage, "04_3")?;
        fade_out_all(
            stage,
            &[three, f1, f2, f3, hits_word, hits, pair_a, pair_b, probe, meet],
            0.4,
        )?;
        let back = kit.tex(
            stage,
            r"\ln(e^{x}) = x \ \ \text{(all } x) \qquad e^{\ln y} = y \ \ (y > 0)",
            30.0,
            Palette::ink(),
            &[(r"\ln", blue)],
            v(3.9, 1.3),
        )?;
        fade_in_all(stage, &[back], 0.5)?;
        kit.until_sentence(stage, "04_3", 1)?;
        // read backwards: from height 2 across to the curve, down to ln 2
        let ln2 = fact_f64("real.ln2")?;
        let across = seg(stage, p(0.0, 2.0), p(ln2, 2.0), gold, 3.0);
        let down = super::draw::arrow(stage, p(ln2, 2.0), p(ln2, 0.05), gold, 3.0)?;
        let ln2_word = kit.tex(
            stage,
            &format!(r"\ln 2 \approx {}", fact("real.ln2")?),
            28.0,
            gold,
            &[],
            p(1.5, -0.45),
        )?;
        fade_in_lagged(stage, &[across, down, ln2_word], 1.4, 0.5)?;
        kit.until_sentence(stage, "04_3", 2)?;
        let ln1 = kit.tex(stage, r"\ln 1 = 0", 32.0, blue, &[], v(2.4, 0.4))?;
        fade_in_all(stage, &[ln1], 0.4)?;
        kit.until_sentence(stage, "04_3", 3)?;
        let e_dot = dot(stage, p(1.0, E), 0.08, gold);
        let e_down = dashed(stage, p(1.0, E), p(1.0, 0.0), gold, 2.5, 8)?;
        let ln_e = kit.tex(
            stage,
            &format!(r"\ln e = 1 \quad (e \approx {})", fact("real.e")?),
            32.0,
            blue,
            &[],
            v(4.4, -0.3),
        )?;
        fade_in_all(stage, &[e_dot, e_down, ln_e], 0.5)?;
        kit.until_sentence(stage, "04_3", 4)?;
        let nowhere = seg(stage, p(0.0, 0.0), p(0.0, LO), red, 7.0);
        let nowhere_word = kit.text_left(stage, "0 and below: no real logarithm", 24.0, red, v(1.0, -1.1))?;
        fade_in_all(stage, &[nowhere, nowhere_word], 0.5)?;

        // 04_4: the mirror image — slope 1/x, and ln 1.01 ≈ 0.01
        kit.say(stage, "04_4")?;
        fade_out_all(
            stage,
            &[
                across,
                down,
                ln2_word,
                e_dot,
                e_down,
                back,
                ln1,
                ln_e,
                nowhere_word,
                at_one,
                law,
            ],
            0.4,
        )?;
        let line_yx = dashed(stage, p(LO, LO), p(HI, HI), Palette::dim(), 2.0, 26)?;
        fade_in_all(stage, &[line_yx], 0.5)?;
        kit.until_sentence(stage, "04_4", 1)?;
        // a slope triangle rides along: run 1, rise e^a; flipped, run e^a, rise 1
        let a = 0.5f64;
        let tri = stage.always_redraw(move |s: &mut Arena| {
            let m = value(s, mirror);
            let (h, k) = (a.exp(), 0.6);
            let pts = [(a, h), (a + k, h), (a + k, h + k * h)];
            let mapped: Vec<Vec3> = pts
                .iter()
                .chain(std::iter::once(&pts[0]))
                .map(|&(x, y)| {
                    let (u, w) = flipped(x, y, m);
                    p(u, w)
                })
                .collect();
            s.add(polyline(mapped).style(line_style(blue, 3.0)).build())
        });
        stage.add_to_scene(tri)?;
        let ln_name = kit.tex(stage, r"y = \ln x", 34.0, blue, &[], p(2.4, 1.55))?;
        play!(stage; tween(mirror, 1.0, 2.6)?);
        swap(stage, &[name], &[ln_name], 0.4)?;
        let swapped = kit.text_left(stage, "rise and run trade places", 24.0, Palette::ink(), v(1.0, 2.2))?;
        fade_in_all(stage, &[swapped], 0.4)?;
        kit.until_sentence(stage, "04_4", 2)?;
        let slope_rule = kit.tex(stage, r"\frac{d}{dx}\ln x = \frac{1}{x}", 36.0, blue, &[], v(4.0, 1.25))?;
        fade_in_all(stage, &[slope_rule], 0.5)?;
        kit.until_sentence(stage, "04_4", 3)?;
        let one = dot(stage, p(1.0, 0.0), 0.08, gold);
        let tangent1 = seg(stage, p(0.2, -0.8), p(2.2, 1.2), gold, 3.0);
        let one_word = kit.text_left(stage, "slope 1 at x = 1", 22.0, gold, v(1.0, 0.35))?;
        fade_in_all(stage, &[one, tangent1, one_word], 0.5)?;
        kit.until_sentence(stage, "04_4", 4)?;
        let small = kit.tex(
            stage,
            &format!(r"\ln 1.01 = {} \approx 0.01", fact("real.ln_one_point_oh_one")?),
            32.0,
            Palette::ink(),
            &[],
            v(4.0, -0.5),
        )?;
        fade_in_all(stage, &[small], 0.5)?;
        kit.until_sentence(stage, "04_4", 5)?;
        let natural = kit.text_left(stage, "+1% in x  →  +0.01 in ln x", 24.0, gold, v(1.0, -1.3))?;
        let why = kit.text_left(
            stage,
            "that is why base e is called natural",
            22.0,
            Palette::dim(),
            v(1.0, -1.85),
        )?;
        fade_in_all(stage, &[natural, why], 0.5)?;

        // 04_5: the ruler again — step 1 is ×e; base 2 is the same ruler
        kit.say(stage, "04_5")?;
        let mut graph: Vec<Mob> = frame.clone();
        graph.extend([
            curve, tri, line_yx, ln_name, swapped, slope_rule, one, tangent1, one_word, small, natural, why, floor,
            nowhere,
        ]);
        fade_out_all(stage, &graph, 0.6)?;
        kit.until_sentence(stage, "04_5", 1)?;
        // one ruler, numbers in the middle, two step scales above and below
        let unit = 2.6;
        let left = -5.6;
        let at = |n: f64| left + unit * n.ln();
        let mid_y = 0.4;
        let spine = seg(stage, v(left, mid_y), v(at(10.0), mid_y), green, 3.0);
        let mut marks = vec![spine];
        for (n, label) in [(1.0, "1"), (2.0, "2"), (E, "e"), (4.0, "4"), (8.0, "8")] {
            marks.push(seg(stage, v(at(n), mid_y - 0.15), v(at(n), mid_y + 0.15), green, 2.5));
            let c = if label == "e" { gold } else { green };
            marks.push(kit.text(stage, label, 24.0, c, v(at(n), mid_y + 0.42))?);
        }
        let e_val = kit.text(
            stage,
            &format!("e ≈ {}", fact("real.e")?),
            22.0,
            gold,
            v(at(E) + 0.2, mid_y + 0.85),
        )?;
        let mut ln_steps = vec![seg(stage, v(left, mid_y - 0.75), v(at(10.0), mid_y - 0.75), blue, 2.0)];
        for k in 0..=2 {
            let x = left + unit * f64::from(k);
            ln_steps.push(seg(stage, v(x, mid_y - 0.85), v(x, mid_y - 0.65), blue, 2.5));
            ln_steps.push(kit.text(stage, &k.to_string(), 20.0, blue, v(x, mid_y - 1.1))?);
        }
        let ln_word = kit.text_left(stage, "ln: step 1 = ×e", 22.0, blue, v(at(10.0) + 0.3, mid_y - 0.75))?;
        fade_in_all(stage, &marks, 0.6)?;
        fade_in_all(stage, &[e_val], 0.4)?;
        fade_in_all(stage, &ln_steps, 0.6)?;
        fade_in_all(stage, &[ln_word], 0.4)?;
        kit.until_sentence(stage, "04_5", 2)?;
        let product = kit.tex(
            stage,
            r"\ln(ab) = \ln a + \ln b \qquad (a, b > 0)",
            36.0,
            Palette::ink(),
            &[(r"\ln", blue)],
            v(0.0, 2.3),
        )?;
        fade_in_all(stage, &[product], 0.5)?;
        kit.until_sentence(stage, "04_5", 3)?;
        let two = super::draw::slide_c();
        let mut two_steps = vec![seg(stage, v(left, mid_y - 1.75), v(at(10.0), mid_y - 1.75), two, 2.0)];
        for k in 0..=3 {
            let x = left + unit * f64::from(k) * 2f64.ln();
            two_steps.push(seg(stage, v(x, mid_y - 1.85), v(x, mid_y - 1.65), two, 2.5));
            two_steps.push(kit.text(stage, &k.to_string(), 20.0, two, v(x, mid_y - 2.1))?);
        }
        let two_word = kit.text_left(stage, "log₂: step 1 = ×2", 22.0, two, v(at(10.0) + 0.3, mid_y - 1.75))?;
        fade_in_all(stage, &two_steps, 0.6)?;
        fade_in_all(stage, &[two_word], 0.4)?;
        kit.until_sentence(stage, "04_5", 4)?;
        let change = kit.tex(
            stage,
            r"\log_2 x = \frac{\ln x}{\ln 2}",
            38.0,
            Palette::ink(),
            &[],
            v(0.0, -2.75),
        )?;
        fade_in_all(stage, &[change], 0.5)?;

        // 04_6: doubling at 5% a year — on a ratio ruler, growth is a line
        kit.say(stage, "04_6")?;
        let mut ruler: Vec<Mob> = marks.clone();
        ruler.extend(ln_steps.iter().copied());
        ruler.extend(two_steps.iter().copied());
        ruler.extend([e_val, ln_word, two_word, product, change]);
        fade_out_all(stage, &ruler, 0.5)?;
        let heading2 = title(&kit, stage, "How long to double?")?;
        swap(stage, &[heading], &[heading2], 0.4)?;
        kit.until_sentence(stage, "04_6", 1)?;
        let rate = fact_f64("real.rate")?;
        let mut panel = vec![
            seg(stage, g(0.0, 0.0), g(150.0, 0.0), Palette::faint(), 2.5),
            seg(
                stage,
                g(0.0, 0.0),
                g(0.0, f64::from(DOUBLINGS) + 0.4),
                Palette::faint(),
                2.5,
            ),
        ];
        for d in 0..=DOUBLINGS {
            panel.push(seg(
                stage,
                g(-1.5, f64::from(d)),
                g(1.5, f64::from(d)),
                Palette::faint(),
                2.0,
            ));
            panel.push(kit.text(
                stage,
                &(1u32 << d).to_string(),
                16.0,
                Palette::dim(),
                v(GX - 0.45, g(0.0, f64::from(d))[1]),
            )?);
        }
        for t in [50, 100, 150] {
            panel.push(kit.text(stage, &t.to_string(), 18.0, Palette::dim(), g(f64::from(t), -0.6))?);
        }
        let years = kit.text(stage, "years", 20.0, Palette::dim(), g(75.0, -1.05))?;
        let up = kit.text(
            stage,
            "growth factor, on a ratio ruler",
            18.0,
            Palette::dim(),
            v(GX + 1.9, g(0.0, 10.75)[1]),
        )?;
        panel.extend([years, up]);
        fade_in_all(stage, &panel, 0.6)?;
        let formula = kit.tex(
            stage,
            &format!(r"\text{{factor}} = e^{{{}\,t}}", fact("real.rate")?),
            34.0,
            green,
            &[],
            v(4.6, 2.4),
        )?;
        let t_end = (f64::from(DOUBLINGS) + 0.4) * 2f64.ln() / rate;
        let line = seg(stage, g(0.0, 0.0), g(t_end, f64::from(DOUBLINGS) + 0.4), green, 4.0);
        fade_in_all(stage, &[formula, line], 0.7)?;
        kit.until_sentence(stage, "04_6", 2)?;
        let double_line = dashed(stage, g(0.0, 1.0), g(150.0, 1.0), gold, 2.5, 30)?;
        let ask = kit.text(stage, "×2 when?", 26.0, gold, v(4.6, 1.5))?;
        fade_in_all(stage, &[double_line, ask], 0.5)?;
        predict(&kit, stage, v(4.6, -2.75), 3.0)?;

        // 04_7: take ln of both sides: 13.9 years; the rule of 70; ×1000
        kit.say(stage, "04_7")?;
        let step1 = kit.tex(
            stage,
            &format!(r"e^{{{}\,t}} = 2", fact("real.rate")?),
            32.0,
            Palette::ink(),
            &[],
            v(4.6, 1.5),
        )?;
        swap(stage, &[ask], &[step1], 0.4)?;
        kit.until_sentence(stage, "04_7", 1)?;
        let step2 = kit.tex(
            stage,
            &format!(r"{}\,t = \ln 2 \approx {}", fact("real.rate")?, fact("real.ln2")?),
            32.0,
            blue,
            &[],
            v(4.6, 0.85),
        )?;
        fade_in_all(stage, &[step2], 0.5)?;
        kit.until_sentence(stage, "04_7", 2)?;
        let doubled = fact_f64("real.double_years")?;
        let step3 = kit.tex(
            stage,
            &format!(r"t \approx {}\ \text{{years}}", fact("real.double_years")?),
            34.0,
            gold,
            &[],
            v(4.6, 0.2),
        )?;
        let hit = dot(stage, g(doubled, 1.0), 0.1, gold);
        let hit_down = dashed(stage, g(doubled, 1.0), g(doubled, 0.0), gold, 2.5, 5)?;
        fade_in_all(stage, &[step3, hit, hit_down], 0.5)?;
        kit.until_sentence(stage, "04_7", 3)?;
        let seventy = kit.text(
            stage,
            "doubling time ≈ 70 ÷ (percent a year)",
            22.0,
            Palette::ink(),
            v(4.3, -0.45),
        )?;
        let why70 = precisely(
            &kit,
            stage,
            &format!(
                "100 × ln 2 = {}; seventy is the round number near it",
                fact("real.rule_of_seventy")?
            ),
        )?;
        fade_in_all(stage, &[seventy, why70], 0.5)?;
        kit.until_sentence(stage, "04_7", 4)?;
        let thousand = kit.tex(
            stage,
            &format!(
                r"t = \frac{{\ln 1000}}{{{}}} \approx \frac{{{}}}{{{}}} \approx {}",
                fact("real.rate")?,
                fact("real.ln_thousand")?,
                fact("real.rate")?,
                fact("real.thousand_years")?
            ),
            30.0,
            green,
            &[],
            v(4.4, -1.3),
        )?;
        let thousand_at = 1000f64.log2();
        let far = dot(stage, g(fact_f64("real.thousand_years")?, thousand_at), 0.1, green);
        fade_in_all(stage, &[thousand, far], 0.5)?;
        kit.until_sentence(stage, "04_7", 5)?;
        let mut tens = Vec::new();
        for d in 1..=DOUBLINGS {
            let t = f64::from(d) * 2f64.ln() / rate;
            tens.push(dot(stage, g(t, f64::from(d)), 0.06, gold));
        }
        fade_in_lagged(stage, &tens, 1.6, 0.2)?;
        let ten = kit.tex(
            stage,
            &format!(
                r"2^{{10}} = {} \quad\Rightarrow\quad 10 \text{{ doublings}} \approx {}\ \text{{years}}",
                fact("real.two_pow10")?,
                fact("real.ten_doublings_years")?
            ),
            28.0,
            gold,
            &[],
            v(4.0, -2.1),
        )?;
        fade_in_all(stage, &[ten], 0.5)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_flip_is_the_mirror_in_y_equals_x_and_fixes_the_line() {
        let (a, b) = flipped(0.5, 0.5f64.exp(), 1.0);
        assert!((a - 0.5f64.exp()).abs() < 1e-12 && (b - 0.5).abs() < 1e-12);
        let (c, d) = flipped(1.3, 1.3, 0.37);
        assert!((c - 1.3).abs() < 1e-12 && (d - 1.3).abs() < 1e-12);
        // half way, every point sits on the mirror line
        let (e, f) = flipped(-1.0, 2.0, 0.5);
        assert!((e - f).abs() < 1e-12);
    }

    #[test]
    fn the_flipped_curve_is_the_graph_of_ln() {
        for (x, y) in exp_curve(1.0, HI).iter().map(|q| (q[0] - OX, q[1] - OY)) {
            assert!((y - x.ln()).abs() < 1e-9, "{x} {y}");
        }
    }

    #[test]
    fn the_probe_meets_the_curve_inside_the_window_for_the_heights_shown() {
        for h in [0.15, 1.0, 2.0, 2.6] {
            assert!(inside(f64::ln(h), h), "{h}");
        }
        // ln 2 and the slope-1 tangent at (1, 0) both sit on screen
        assert!(inside(2f64.ln(), 2.0) && inside(2.2, 1.2) && inside(0.2, -0.8));
    }

    #[test]
    fn the_growth_panel_holds_ten_doublings_and_the_thousandfold_point() {
        let t = 10.4 * 2f64.ln() / 0.05;
        assert!(g(t, 10.4)[0] < 1.6 && g(t, 10.4)[1] < 3.0);
        let k = 1000f64.ln() / 0.05;
        assert!((k - 138.155).abs() < 0.01 && g(k, 0.0)[0] < 1.6);
        assert!(g(0.0, 0.0)[1] > -3.0);
    }
}
