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
//! Rebuild candidate for `mx-video-program-program-87do.63`; picture repairs
//! from the independent review staged in `logarithm-repair01-04`. Type-checked
//! against stand-ins only: not compiled against fmn, unrendered, unlistened.

use std::f64::consts::{E, PI};

use std::collections::HashMap;

use explainer_kit::{Kit, Palette, play, tween, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::draw::{
    answer_c, dashed, dot, fact, fact_f64, fade_in_all, fade_in_lagged, fade_out_all, line_style, number_c, position_c,
    precisely, predict, ring, seg, shift_all, swap, title, value, warn_c,
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
/// The mirror's name in graph units: its left edge, just right of the line's
/// upper end, and its height.
const LINE_NAME_AT: (f64, f64) = (3.05, 2.8);

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

/// The stroke of the tangent at x = `a`, whose slope equals its height.
fn tangent_points(a: f64) -> Vec<Vec3> {
    let h = a.exp();
    let d = 0.8 / (1.0 + h * h).sqrt();
    vec![p(a - d, h - h * d), p(a + d, h + h * d)]
}

/// How far the climbing tangent of 04_2 rises: it stays under the title.
const CLIMB_FROM: f64 = -2.2;
const CLIMB_TO: f64 = 0.95;
/// The words the climb belongs to, in sentence 3 of 04_2.
const CLIMB_PHRASE: &str = "it only ever climbs";
/// The climb ends this long before the next sentence.
const CLIMB_CLEAR: f64 = 0.3;
/// The climb never takes less than this.
const CLIMB_SHORTEST: f64 = 2.0;
const SCRIPT: &str = include_str!("../../script.tsv");

/// When `CLIMB_PHRASE` begins, in scene seconds. `said` is when 04_2 began; `s3`
/// and `s4` are when its sentences 3 and 4 begin. A word-onset mark at exactly the
/// phrase's byte wins (`marks` as `Narrator::phrase_marks` returns them: bound to
/// this take and this text, though still not a heard boundary). Without one, the
/// phrase's share of its sentence's characters interpolates between the two
/// onsets: a rougher estimate, never measured timing.
fn climb_onset(text: &str, said: f64, s3: f64, s4: f64, marks: &HashMap<usize, f64>) -> Option<f64> {
    let byte = text.find(CLIMB_PHRASE)?;
    if let Some(&seconds) = marks.get(&byte) {
        return Some(said + seconds);
    }
    let from = text[..byte].rfind(". ").map_or(0, |k| k + 2);
    let to = text[byte..].find(". ").map_or(text.len(), |k| byte + k + 1);
    let share = text[from..byte].chars().count() as f64 / text[from..to].chars().count() as f64;
    Some(s3 + (s4 - s3) * share)
}

/// When the climb starts and when the next sentence begins, in scene seconds,
/// once 04_2 has been said. `None` without a narrator.
fn climb_window(kit: &Kit) -> Option<(f64, f64)> {
    let n = kit.narrator.as_ref()?;
    let (s3, s4) = (n.sentence_at("04_2", 3)?, n.sentence_at("04_2", 4)?);
    let said = s3 - n.sentence_starts("04_2").get(3).copied()?;
    let text = SCRIPT.lines().find_map(|row| row.strip_prefix("04_2\t"))?;
    Some((climb_onset(text, said, s3, s4, &n.phrase_marks("04_2"))?, s4))
}

/// The dashes of the red floor y = 0, carried part way `m` across the mirror:
/// e^x's floor becomes the logarithm's wall at x = 0.
const FLOOR_DASHES: usize = 22;

fn floor_parts(s: &mut Arena, m: f64, c: Srgb) -> Mob {
    let g = s.add(VMobject::new());
    let at = |t: f64| {
        let (a, b) = flipped(LO + (HI - LO) * t, 0.0, m);
        p(a, b)
    };
    for k in 0..FLOOR_DASHES {
        let (t0, t1) = (k as f64 / FLOOR_DASHES as f64, (k as f64 + 0.55) / FLOOR_DASHES as f64);
        let dash = s.add(polyline(vec![at(t0), at(t1)]).style(line_style(c, 3.0)).build());
        let _ = s.attach(g, dash);
    }
    g
}

/// The slope triangle: its corner on the curve at x = `TRI_A`, then run `TRI_K`,
/// then rise `TRI_K` × height.
const TRI_A: f64 = 0.5;
const TRI_K: f64 = 0.6;

fn tri_points() -> [(f64, f64); 3] {
    let h = TRI_A.exp();
    [(TRI_A, h), (TRI_A + TRI_K, h), (TRI_A + TRI_K, h + TRI_K * h)]
}

/// The ruler of 04_5: where 1 sits, the length of one ×e, and its height. Two
/// rows between the spine and the first step scale carry the product's lengths.
const RULER_LEFT: f64 = -5.6;
const RULER_UNIT: f64 = 2.6;
const RULER_Y: f64 = 0.4;
const PRODUCT_ROW_A: f64 = RULER_Y - 0.3;
const PRODUCT_ROW_B: f64 = RULER_Y - 0.48;

fn ruler_x(n: f64) -> f64 {
    RULER_LEFT + RULER_UNIT * n.ln()
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
/// The think bar of 04_6: just under "×2 when?", which sits at y = 1.5.
const THINK_BAR_Y: f64 = 1.1;
/// The two rows that close the chapter, centred right of the panel.
const TEN_ROWS_X: f64 = 4.4;
/// The name y = e^x: right of the dot at (1, e) that 04_3 places, and left of
/// the mirror y = x that 04_4 draws.
const EXP_NAME_X: f64 = 1.68;

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
            let m = value(s, mirror);
            let pts = exp_curve(m, value(s, drawn));
            let pts = if pts.len() < 2 { vec![p(LO, LO.exp()); 2] } else { pts };
            // blue once it is a logarithm; the change falls where the curve is edge-on
            let c = if m > 0.5 { blue } else { green };
            s.add(polyline(pts).style(line_style(c, 4.0)).build())
        });
        stage.add_to_scene(curve)?;
        play!(stage; tween(drawn, HI, 2.0)?);
        let name = kit.tex(stage, r"y = e^{x}", 34.0, green, &[], p(EXP_NAME_X, 2.4))?;
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
            s.add(polyline(tangent_points(a)).style(line_style(blue, 3.0)).build())
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
        let floor = dashed(stage, p(LO, 0.0), p(HI, 0.0), red, 3.0, FLOOR_DASHES)?;
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
        // "it only ever climbs": the tangent runs up the curve, steeper as it goes
        let climb = stage.add_value_tracker(CLIMB_FROM);
        let climb_line = stage.always_redraw(move |s: &mut Arena| {
            let a = value(s, climb);
            s.add(polyline(tangent_points(a)).style(line_style(blue, 3.0)).build())
        });
        let climb_dot = stage.always_redraw(move |s: &mut Arena| {
            let a = value(s, climb);
            s.add(
                Dot::new()
                    .point(p(a, a.exp()))
                    .radius(0.07)
                    .style(super::draw::flat(blue, 1.0))
                    .build(),
            )
        });
        stage.add_to_scene(climb_line)?;
        stage.add_to_scene(climb_dot)?;
        // it waits at the low end for those words, then moves through "from near
        // zero on the left to as large as you like on the right"
        let run = match climb_window(&kit) {
            Some((onset, next)) => {
                super::draw::wait_until(stage, onset)?;
                (next - stage.scene().time().to_f64() - CLIMB_CLEAR).max(CLIMB_SHORTEST)
            }
            None => 4.0,
        };
        play!(stage; tween(climb, CLIMB_TO, run)?);
        kit.until_sentence(stage, "04_2", 4)?;
        fade_out_all(stage, &[climb_line, climb_dot], 0.3)?;
        // a horizontal probe at height c meets the curve once for c > 0
        let c = stage.add_value_tracker(PROBE_START);
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
        play!(stage; tween(c, PROBE_REST, 1.6)?);

        // 04_3: reversing it — ln, ln 1 = 0, ln e = 1, nothing for c ≤ 0
        kit.say(stage, "04_3")?;
        fade_out_all(
            stage,
            &[three, f1, f2, f3, hits_word, hits, pair_a, pair_b, probe, meet, law],
            0.4,
        )?;
        kit.until_sentence(stage, "04_3", 1)?;
        let named = kit.text(stage, "ln: the natural logarithm", 24.0, blue, v(3.9, 2.0))?;
        let back = kit.tex(
            stage,
            r"\ln(e^{x}) = x \ \ \text{(all } x) \qquad e^{\ln y} = y \ \ (y > 0)",
            30.0,
            Palette::ink(),
            &[(r"\ln", blue)],
            v(3.9, 1.3),
        )?;
        fade_in_all(stage, &[named, back], 0.5)?;
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
            p(1.1, -0.75),
        )?;
        fade_in_lagged(stage, &[across, down, ln2_word], 1.4, 0.5)?;
        kit.until_sentence(stage, "04_3", 2)?;
        let ln1 = kit.tex(stage, r"\ln 1 = 0", 32.0, blue, &[], v(2.4, 0.4))?;
        let at_one_ring = ring(stage, p(0.0, 1.0), 0.17, gold, 3.0);
        fade_in_all(stage, &[ln1, at_one_ring], 0.4)?;
        kit.until_sentence(stage, "04_3", 3)?;
        fade_out_all(stage, &[at_one_ring], 0.3)?;
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
                named,
                ln1,
                ln_e,
                nowhere_word,
                at_one,
            ],
            0.4,
        )?;
        let line_yx = dashed(stage, p(LO, LO), p(HI, HI), Palette::dim(), 2.0, 26)?;
        let line_name = kit.tex_left(
            stage,
            r"y = x",
            24.0,
            Palette::dim(),
            &[],
            p(LINE_NAME_AT.0, LINE_NAME_AT.1),
        )?;
        fade_in_all(stage, &[line_yx, line_name], 0.5)?;
        // a slope triangle rides along: run k, rise k·e^a; flipped, the legs trade
        let tri = stage.always_redraw(move |s: &mut Arena| {
            let m = value(s, mirror);
            let pts = tri_points();
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
        let h = TRI_A.exp();
        let run0 = kit.text(stage, "run", 20.0, blue, p(TRI_A + TRI_K / 2.0, h - 0.2))?;
        let rise0 = kit.text(stage, "rise", 20.0, blue, p(TRI_A + TRI_K + 0.33, h + 0.35 * TRI_K * h))?;
        fade_in_all(stage, &[run0, rise0], 0.4)?;
        // The red marks were about the heights e^x never reaches. Reflected with
        // the curve they say what is true of ln: the floor becomes its wall at
        // x = 0, and the bar lands on the inputs with no real logarithm. Each live
        // copy starts exactly on its still original, so the handover cannot show.
        let wall = stage.always_redraw(move |s: &mut Arena| {
            let m = value(s, mirror);
            floor_parts(s, m, red)
        });
        let barred = stage.always_redraw(move |s: &mut Arena| {
            let m = value(s, mirror);
            let (a0, b0) = flipped(0.0, 0.0, m);
            let (a1, b1) = flipped(0.0, LO, m);
            s.add(polyline(vec![p(a0, b0), p(a1, b1)]).style(line_style(red, 7.0)).build())
        });
        stage.add_to_scene(wall)?;
        stage.add_to_scene(barred)?;
        // the legs stay named until the sentence that swaps them
        kit.until_sentence(stage, "04_4", 1)?;
        fade_out_all(stage, &[name, run0, rise0, floor, nowhere], 0.25)?;
        play!(stage; tween(mirror, 1.0, 2.6)?);
        let ln_name = kit.tex(stage, r"y = \ln x", 34.0, blue, &[], p(2.75, 1.7))?;
        let rise1 = kit.text(stage, "rise", 20.0, blue, p(h - 0.33, TRI_A + TRI_K / 2.0))?;
        let run1 = kit.text(stage, "run", 20.0, blue, p(h + TRI_K * h / 2.0, TRI_A + TRI_K + 0.17))?;
        let no_log = kit.text(stage, "no real logarithm", 22.0, red, p(-1.35, 0.32))?;
        let swapped = kit.text_left(stage, "rise and run trade places", 24.0, Palette::ink(), v(1.0, 2.2))?;
        fade_in_all(stage, &[ln_name, rise1, run1, no_log, swapped], 0.4)?;
        kit.until_sentence(stage, "04_4", 2)?;
        let slope_rule = kit.tex(stage, r"\frac{d}{dx}\ln x = \frac{1}{x}", 36.0, blue, &[], v(4.0, 1.25))?;
        fade_in_all(stage, &[slope_rule], 0.5)?;
        kit.until_sentence(stage, "04_4", 3)?;
        fade_out_all(stage, &[tri, rise1, run1], 0.3)?;
        let one = dot(stage, p(1.0, 0.0), 0.08, gold);
        let tangent1 = seg(stage, p(0.2, -0.8), p(2.2, 1.2), gold, 3.0);
        let one_word = kit.text_left(stage, "slope 1 at x = 1", 22.0, gold, v(1.0, 0.35))?;
        fade_in_all(stage, &[one, tangent1, one_word], 0.5)?;
        kit.until_sentence(stage, "04_4", 4)?;
        let small = kit.tex(
            stage,
            &format!(r"\ln 1.01 \approx {} \approx 0.01", fact("real.ln_one_point_oh_one")?),
            32.0,
            Palette::ink(),
            &[],
            v(4.0, -0.5),
        )?;
        fade_in_all(stage, &[small], 0.5)?;
        kit.until_sentence(stage, "04_4", 5)?;
        let natural = kit.text_left(stage, "+1% in x  →  about +0.01 in ln x", 24.0, gold, v(1.0, -1.3))?;
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
            curve, line_yx, line_name, ln_name, swapped, slope_rule, one, tangent1, one_word, small, natural, why,
            wall, barred, no_log,
        ]);
        // one ruler, numbers in the middle, two step scales above and below; it
        // arrives as the graph leaves, on "this is the ruler again"
        let (unit, left, mid_y) = (RULER_UNIT, RULER_LEFT, RULER_Y);
        let at = ruler_x;
        let spine = seg(stage, v(left, mid_y), v(at(10.0), mid_y), green, 3.0);
        let mut marks = vec![spine];
        for (n, label) in [(1.0, "1"), (2.0, "2"), (E, "e"), (4.0, "4"), (8.0, "8")] {
            marks.push(seg(stage, v(at(n), mid_y - 0.15), v(at(n), mid_y + 0.15), green, 2.5));
            let c = if label == "e" { gold } else { green };
            marks.push(kit.text(stage, label, 24.0, c, v(at(n), mid_y + 0.42))?);
        }
        swap(stage, &graph, &marks, 0.6)?;
        kit.until_sentence(stage, "04_5", 1)?;
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
        // 2 × 4 as lengths: 1 to 2, then a copy of 1 to 4 slid to start where the
        // first one ends. It lands on 8, the slide rule's move on this ruler.
        let product_a = fact_f64("real.product_a")?;
        let product_b = fact_f64("real.product_b")?;
        let product_ab = fact_f64("real.product_ab")?;
        let bar_a = seg(
            stage,
            v(at(1.0) + 0.03, PRODUCT_ROW_A),
            v(at(product_a) - 0.03, PRODUCT_ROW_A),
            blue,
            6.0,
        );
        let bar_b = seg(
            stage,
            v(at(1.0) + 0.03, PRODUCT_ROW_B),
            v(at(product_b) - 0.03, PRODUCT_ROW_B),
            super::draw::slide_c(),
            6.0,
        );
        fade_in_lagged(stage, &[bar_a, bar_b], 0.9, 0.5)?;
        shift_all(stage, &[bar_b], v(at(product_a) - at(1.0), 0.0), 1.2)?;
        let landing = ring(stage, v(at(product_ab), mid_y), 0.18, gold, 3.0);
        let landing_word = kit.text_left(
            stage,
            &format!(
                "{} × {} = {}",
                fact("real.product_a")?,
                fact("real.product_b")?,
                fact("real.product_ab")?
            ),
            24.0,
            gold,
            v(at(product_ab) + 0.3, (PRODUCT_ROW_A + PRODUCT_ROW_B) / 2.0),
        )?;
        fade_in_all(stage, &[landing, landing_word], 0.4)?;
        kit.until_sentence(stage, "04_5", 3)?;
        fade_out_all(stage, &[bar_a, bar_b, landing, landing_word], 0.4)?;
        let two = super::draw::slide_c();
        let mut two_steps = vec![seg(stage, v(left, mid_y - 1.75), v(at(10.0), mid_y - 1.75), two, 2.0)];
        for k in 0..=3 {
            let x = left + unit * f64::from(k) * 2f64.ln();
            two_steps.push(seg(stage, v(x, mid_y - 1.85), v(x, mid_y - 1.65), two, 2.5));
            two_steps.push(kit.text(stage, &k.to_string(), 20.0, two, v(x, mid_y - 2.1))?);
        }
        let two_word = kit.tex_left(
            stage,
            r"\log_2:\ \text{step 1} = \times 2",
            22.0,
            two,
            &[],
            v(at(10.0) + 0.3, mid_y - 1.75),
        )?;
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
            "growth factor on a ratio ruler: each mark is ×2",
            18.0,
            Palette::dim(),
            v(GX + 2.1, g(0.0, 10.75)[1]),
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
        // the money grows: the line draws itself up the ratio ruler
        let grown = stage.add_value_tracker(0.01);
        let line = stage.always_redraw(move |s: &mut Arena| {
            let t = value(s, grown);
            s.add(
                polyline(vec![g(0.0, 0.0), g(t, t * rate / 2f64.ln())])
                    .style(line_style(green, 4.0))
                    .build(),
            )
        });
        stage.add_to_scene(line)?;
        fade_in_all(stage, &[formula], 0.5)?;
        play!(stage; tween(grown, t_end, 2.0)?);
        kit.until_sentence(stage, "04_6", 2)?;
        let double_line = dashed(stage, g(0.0, 1.0), g(150.0, 1.0), gold, 2.5, 30)?;
        let ask = kit.text(stage, "×2 when?", 26.0, gold, v(4.6, 1.5))?;
        fade_in_all(stage, &[double_line, ask], 0.5)?;
        predict(&kit, stage, v(4.6, THINK_BAR_Y), 3.0)?;

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
        let hit_foot = kit.text(stage, fact("real.double_years")?, 18.0, gold, g(doubled, -0.6))?;
        fade_in_all(stage, &[step3, hit, hit_down, hit_foot], 0.5)?;
        kit.until_sentence(stage, "04_7", 3)?;
        let seventy = kit.text(
            stage,
            "rule of seventy: time ≈ 70 ÷ (percent a year)",
            22.0,
            Palette::ink(),
            v(4.3, -0.45),
        )?;
        let why70 = precisely(
            &kit,
            stage,
            &format!(
                "100 × ln 2 ≈ {}; seventy is the round number near it",
                fact("real.rule_of_seventy")?
            ),
        )?;
        fade_in_all(stage, &[seventy, why70], 0.5)?;
        kit.until_sentence(stage, "04_7", 4)?;
        let thousand = kit.tex(
            stage,
            &format!(
                r"t = \frac{{\ln {}}}{{{}}} \approx \frac{{{}}}{{{}}} \approx {}",
                fact("real.growth_target")?,
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
        let thousand_at = fact_f64("real.growth_target")?.log2();
        let far_at = g(fact_f64("real.thousand_years")?, thousand_at);
        let far = dot(stage, far_at, 0.1, green);
        let far_word = kit.text_left(
            stage,
            &format!("×{}", fact("real.growth_target")?),
            20.0,
            green,
            v(far_at[0] + 0.22, far_at[1] - 0.2),
        )?;
        fade_in_all(stage, &[thousand, far, far_word], 0.5)?;
        kit.until_sentence(stage, "04_7", 5)?;
        let mut tens = Vec::new();
        for d in 1..=DOUBLINGS {
            let t = f64::from(d) * 2f64.ln() / rate;
            tens.push(dot(stage, g(t, f64::from(d)), 0.06, gold));
        }
        fade_in_lagged(stage, &tens, 1.6, 0.2)?;
        // two rows, clear of the end of the dashed ×2 line
        let ten = kit.tex(
            stage,
            &format!(r"2^{{10}} = {}", fact("real.two_pow10")?),
            28.0,
            gold,
            &[],
            v(TEN_ROWS_X, -1.95),
        )?;
        let ten_years = kit.tex(
            stage,
            &format!(
                r"10 \text{{ doublings}} \approx {}\ \text{{years}}",
                fact("real.ten_doublings_years")?
            ),
            28.0,
            gold,
            &[],
            v(TEN_ROWS_X, -2.4),
        )?;
        fade_in_all(stage, &[ten, ten_years], 0.5)?;
        kit.finish(stage)
    }
}

/// The height the full-width probe of 04_2 starts from, and the top of its whole
/// path: it goes down from here and comes back only to PROBE_REST. The line stays
/// under the green y = e^x (lowest ink near 2.24) and over the tick label 2.
const PROBE_START: f64 = 2.16;
/// Where the probe rests until it fades: half way between the tick labels 1 and 2,
/// so the still line never lies across a number.
const PROBE_REST: f64 = 1.5;

#[cfg(test)]
mod tests {
    use super::*;

    /// The scene code without these tests, for checks on the order of its beats.
    fn scene_source() -> &'static str {
        include_str!("undoing_growth.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap_or("")
    }

    fn found(text: &str) -> usize {
        scene_source()
            .find(text)
            .unwrap_or_else(|| panic!("not in the scene: {text}"))
    }

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
        for h in [0.15, 1.0, PROBE_REST, PROBE_START] {
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

    #[test]
    fn the_red_marks_land_where_the_logarithm_really_has_no_value() {
        // the bar of heights e^x never reaches lands on the negative inputs of ln
        for y in [-0.3, -1.0, LO] {
            let (a, b) = flipped(0.0, y, 1.0);
            assert!((a - y).abs() < 1e-12 && b.abs() < 1e-12, "{y}");
            assert!(a < 0.0);
        }
        // the floor y = 0 lands on the wall x = 0
        for x in [LO, -1.0, 1.0, HI] {
            let (a, b) = flipped(x, 0.0, 1.0);
            assert!(a.abs() < 1e-12 && (b - x).abs() < 1e-12, "{x}");
        }
        // and the curve stays right of that wall all the way down the window
        for q in exp_curve(1.0, HI) {
            assert!(q[0] - OX > 0.0, "{q:?}");
        }
    }

    #[test]
    fn the_still_red_marks_go_as_the_live_ones_take_over_before_the_flip() {
        let live = found("stage.add_to_scene(barred)");
        let fade = found("&[name, run0, rise0, floor, nowhere]");
        let flip = found("tween(mirror, 1.0, 2.6)");
        assert!(live < fade && fade < flip);
        // the graph's own fade takes the live marks, never the still ones
        let graph = found("let mut graph: Vec<Mob> = frame.clone();");
        let graph_end = graph + scene_source()[graph..].find("]);").expect("the list ends");
        let list = &scene_source()[graph..graph_end];
        assert!(list.contains("wall") && list.contains("barred") && list.contains("no_log"));
        assert!(!list.contains("floor") && !list.contains("nowhere") && !list.contains("tri"));
    }

    #[test]
    fn each_live_red_mark_starts_exactly_on_its_still_original() {
        // m = 0 leaves every point where it was, so the handover cannot show
        for (x, y) in [(LO, 0.0), (HI, 0.0), (0.0, 0.0), (0.0, LO)] {
            assert_eq!(flipped(x, y, 0.0), (x, y));
        }
    }

    #[test]
    fn the_flipped_triangle_has_slope_one_over_x() {
        let [a, b, c] = tri_points();
        // before: slope = rise / run = height
        assert!(((c.1 - b.1) / (b.0 - a.0) - a.1).abs() < 1e-12);
        assert!((a.1 - TRI_A.exp()).abs() < 1e-12);
        // after: the old run stands up and the old rise lies down
        let m = |q: (f64, f64)| flipped(q.0, q.1, 1.0);
        let (a, b, c) = (m(a), m(b), m(c));
        assert!((a.0 - b.0).abs() < 1e-12, "the old run is now vertical");
        assert!((b.1 - c.1).abs() < 1e-12, "the old rise is now level");
        let slope = (b.1 - a.1) / (c.0 - b.0);
        assert!((slope - 1.0 / a.0).abs() < 1e-12, "{slope}");
    }

    #[test]
    fn the_ln_two_label_clears_the_tick_labels_and_the_red_bar() {
        // the label is centred at p(1.1, -0.75), about 1.75 wide at 28 points
        let (centre, half_w, half_h) = (1.1, 0.875, 0.14);
        let tick_label_bottom = -0.3 - 0.07;
        assert!(tick_label_bottom - (-0.75 + half_h) >= 0.1);
        assert!(centre - half_w > 0.05, "right of the red bar at x = 0");
        // its foot, ln 2, stands over the label
        assert!(((centre - half_w)..(centre + half_w)).contains(&2f64.ln()));
    }

    #[test]
    fn the_law_goes_with_04_2_and_the_formula_waits_for_its_name() {
        let fade = found("&[three, f1, f2, f3, hits_word, hits, pair_a, pair_b, probe, meet, law]");
        let named = found(r#"kit.until_sentence(stage, "04_3", 1)"#);
        assert!(fade < named && named < found("fade_in_all(stage, &[named, back], 0.5)"));
    }

    #[test]
    fn the_tangent_climbs_only_inside_its_sentence_and_under_the_title() {
        let three = found(r#"kit.until_sentence(stage, "04_2", 3)"#);
        let four = found(r#"kit.until_sentence(stage, "04_2", 4)"#);
        let onset = found("super::draw::wait_until(stage, onset)");
        let climb = found("tween(climb, CLIMB_TO, run)");
        assert!(three < onset && onset < climb && climb < four);
        let top = tangent_points(CLIMB_TO)[1];
        assert!(top[1] < 3.0, "{top:?}");
        let left = tangent_points(CLIMB_FROM)[0];
        assert!(left[0] > -6.86, "{left:?}");
    }

    #[test]
    fn a_phrase_mark_beats_the_estimate_for_when_the_climb_starts() {
        let text = SCRIPT
            .lines()
            .find_map(|row| row.strip_prefix("04_2\t"))
            .expect("04_2 is in the script");
        assert_eq!(text.matches(CLIMB_PHRASE).count(), 1);
        let byte = text.find(CLIMB_PHRASE).expect("the phrase is in 04_2");
        // climb_window reads sentences 3 and 4, so the phrase must stay in sentence 3
        assert_eq!(text[..byte].matches(". ").count(), 3);
        // a mark at exactly this byte is seconds into the line, whatever the sentences say
        let marked = HashMap::from([(byte, 17.5)]);
        assert_eq!(climb_onset(text, 10.0, 20.0, 30.0, &marked), Some(27.5));
        // a mark on any other word is ignored; 54 of the sentence's 139 characters come first
        let elsewhere = HashMap::from([(byte + 1, 17.5)]);
        let estimate = climb_onset(text, 10.0, 20.0, 30.0, &elsewhere).expect("the phrase is in 04_2");
        assert!((estimate - (20.0 + 10.0 * 54.0 / 139.0)).abs() < 1e-9, "{estimate}");
        assert_eq!(climb_onset(text, 10.0, 20.0, 30.0, &HashMap::new()), Some(estimate));
        assert_eq!(climb_onset("Some other line.", 10.0, 20.0, 30.0, &HashMap::new()), None);
    }

    #[test]
    fn the_legs_stay_named_until_the_sentence_that_swaps_them() {
        let named = found("fade_in_all(stage, &[run0, rise0], 0.4)");
        let sentence = found(r#"kit.until_sentence(stage, "04_4", 1)"#);
        let fade = found("&[name, run0, rise0, floor, nowhere]");
        let flip = found("tween(mirror, 1.0, 2.6)");
        assert!(named < sentence && sentence < fade && fade < flip);
    }

    #[test]
    fn the_thousandfold_formula_reads_its_target_from_the_facts() {
        assert!(!scene_source().contains(r"\ln 1000"));
        assert!(found(r"\frac{{\ln {}}}") < found(r#"fact("real.growth_target")?"#));
    }

    #[test]
    fn the_product_law_is_drawn_two_lengths_landing_on_eight() {
        let (a, b) = (ruler_x(2.0) - ruler_x(1.0), ruler_x(4.0) - ruler_x(1.0));
        assert!((ruler_x(1.0) + a + b - ruler_x(8.0)).abs() < 1e-12);
        // the rows fit between the spine's ticks and the first step scale
        let half = 6.0 * 0.01 / 2.0;
        assert!(RULER_Y - 0.15 - (PRODUCT_ROW_A + half) >= 0.1);
        assert!((PRODUCT_ROW_B - half) - (RULER_Y - 0.65) >= 0.1);
        assert!(PRODUCT_ROW_A - PRODUCT_ROW_B >= 2.0 * half + 0.05);
        // "2 × 4 = 8" is 4.39 ems at 24 points, right of the 8 and under the spine
        assert!(ruler_x(8.0) + 0.3 + 4.39 * 24.0 * 0.01 < 6.86);
    }

    #[test]
    fn the_ruler_is_on_screen_when_the_voice_says_ruler() {
        let say = found(r#"kit.say(stage, "04_5")"#);
        let sentence = found(r#"kit.until_sentence(stage, "04_5", 1)"#);
        let swap_at = found("swap(stage, &graph, &marks, 0.6)");
        assert!(say < swap_at && swap_at < sentence);
    }

    #[test]
    fn the_whole_probe_path_stays_under_the_name_and_rests_clear_of_the_tick_labels() {
        // half the 2.5 stroke, and the least clearance kept from any ink
        const HALF_LINE: f64 = 0.0125;
        const GAP: f64 = 0.03;
        // start, down to 0.15, below zero, up to the rest: the start is the top of the path
        let path = [
            found("stage.add_value_tracker(PROBE_START)"),
            found("tween(c, 0.15, 2.4)"),
            found("tween(c, -1.2, 1.4)"),
            found("tween(c, PROBE_REST, 1.6)"),
        ];
        assert!(path.windows(2).all(|w| w[0] < w[1]));
        const { assert!(PROBE_REST < PROBE_START) };
        // the name's lowest possible ink and the top of the tick label 2
        const { assert!(PROBE_START + HALF_LINE + GAP < 2.4 - 0.17) };
        const { assert!(PROBE_START - HALF_LINE - GAP > 2.0 + 0.07) };
        // at rest, between the tick labels 1 and 2
        const { assert!(PROBE_REST - HALF_LINE - GAP > 1.0 + 0.07) };
        const { assert!(PROBE_REST + HALF_LINE + GAP < 2.0 - 0.07) };
        // the moving dot, radius 0.09, stays left of the name
        assert!(f64::ln(PROBE_START) + 0.09 < EXP_NAME_X - 0.47);
    }

    #[test]
    fn the_small_step_caption_says_it_is_approximate() {
        // ln 1.01 is 0.00995, as the row above it says; the caption stands alone
        assert!(found("\"+1% in x  →  about +0.01 in ln x\"") > 0);
    }

    #[test]
    fn the_flip_labels_keep_clear_of_each_other_the_names_and_the_mirror() {
        let h = TRI_A.exp();
        // "rise" before the flip, and y = e^x beside it (about 0.94 wide, 0.34 tall)
        let (rise_x, rise_y) = (TRI_A + TRI_K + 0.33, h + 0.35 * TRI_K * h);
        assert!(2.4 - 0.17 - (rise_y + 0.09) >= 0.03, "{rise_y}");
        assert!(rise_x < 3.0);
        // the name, about 0.94 wide, clears e's dot at (1, e), which reaches x = 1.08
        const { assert!(EXP_NAME_X - 0.47 > 1.08 + 0.1) };
        // and it ends left of the mirror at the height of its lowest ink
        const { assert!(EXP_NAME_X + 0.47 < 2.4 - 0.17 - 0.05) };
        // y = ln x, about 1.0 wide, stays right of the mirror y = x at its height
        let (name_left, name_top) = (2.75 - 0.5, 1.7 + 0.17);
        assert!(name_left > name_top + 0.1, "{name_left} {name_top}");
        // "run" sits above the level leg and under y = ln x
        let run_y = TRI_A + TRI_K + 0.17;
        assert!(1.7 - 0.17 - (run_y + 0.09) >= 0.05, "{run_y}");
        // the mirror's name, about 0.6 wide and 0.25 tall, stands right of the line
        // at the height of its top, and short of "rise and run trade places" at 1.0
        const { assert!(LINE_NAME_AT.0 > LINE_NAME_AT.1 + 0.125 + 0.1) };
        const { assert!(OX + LINE_NAME_AT.0 + 0.6 < 1.0 - 0.1) };
    }

    #[test]
    fn the_think_bar_sits_under_its_question_and_clear_of_the_next_row() {
        const { assert!(1.5 - THINK_BAR_Y >= 0.3) };
        // it fades before the 0.85 row of 04_7 arrives, and clears it anyway
        const { assert!(THINK_BAR_Y - 0.85 >= 0.2) };
    }

    #[test]
    fn the_closing_rows_start_well_right_of_the_dashed_line() {
        // the wider row is about 3.7 wide at 28 points
        let line_end = g(150.0, 1.0)[0];
        assert!(TEN_ROWS_X - 3.7 / 2.0 - line_end >= 0.5, "{line_end}");
        const { assert!(TEN_ROWS_X + 3.7 / 2.0 <= 6.86) };
    }
}
