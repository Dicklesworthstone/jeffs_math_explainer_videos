//! Chapter 09, matrices as motions: a 2×2 matrix moves the whole plane,
//! shown by where the unit arrows and a grid land (09_1); a matrix X read as
//! a velocity field, every point following its arrow for one unit of time,
//! arriving at e^X (09_2); a logarithm of A as a steady flow that ends on A
//! (09_3); the quarter turn reached by three different spins, the column of
//! complex logarithms again (09_4).
//!
//! 09_1 slides each grid from before to after in a straight line, and says
//! so on screen: that slide is not a flow (chapter 11 shows why a straight
//! slide can flatten the plane where no flow ever does). From 09_2 on every
//! moving grid is a true flow e^{tX}, computed by `model::flow`.
//!
//! Rebuild candidate for `mx-video-program-program-87do.63`; uncompiled,
//! unrendered, unlistened.

use std::f64::consts::TAU;

use explainer_kit::{Kit, Palette, play, tween, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::draw::{
    answer_c, arrow_parts, fact_list, fact_matrix, fade_in_all, fade_in_lagged, fade_out_all, flat, grid_under, lap_c,
    letter_f, matrix_card, number_c, position_c, slide_c, swap, title, value,
};
use crate::model::{self, I2, J, M2};

pub struct MatrixLogs {
    pub kit: Kit,
}

/// The main panel: centre and scene units per unit.
const CENTRE: [f64; 3] = [-3.2, -0.4, 0.0];
const UNIT: f64 = 0.78;
/// The three small panels of 09_4.
const SMALL: [[f64; 3]; 3] = [[-5.0, 0.15, 0.0], [-1.7, 0.15, 0.0], [1.6, 0.15, 0.0]];
const SMALL_UNIT: f64 = 0.48;

fn lerp(a: M2, b: M2, t: f64) -> M2 {
    model::madd(model::mscale(a, 1.0 - t), model::mscale(b, t))
}

/// The grid, the two unit arrows and the letter F under `m`.
fn picture(s: &mut Arena, m: M2, centre: [f64; 3], unit: f64, n: i32) -> Mob {
    let g = s.add(VMobject::new());
    let grid = grid_under(s, m, centre, unit, n, number_c(), 0.75);
    let _ = s.attach(g, grid);
    let o = v(centre[0], centre[1]);
    for (col, c) in [(0usize, number_c()), (1usize, slide_c())] {
        let tip = v(centre[0] + unit * m[0][col], centre[1] + unit * m[1][col]);
        let a = arrow_parts(s, o, tip, c, 4.0, 0.16);
        let _ = s.attach(g, a);
    }
    let f = s.add(
        Polygon::new(letter_f(m, centre, unit * 0.9))
            .style(flat(answer_c(), 0.85))
            .build(),
    );
    let _ = s.attach(g, f);
    g
}

/// The velocity field of X: at each grid point an arrow along X·p.
fn field(s: &mut Arena, x: M2, centre: [f64; 3], unit: f64) -> Mob {
    let g = s.add(VMobject::new());
    for i in -2..=2 {
        for j in -2..=2 {
            let p = [f64::from(i), f64::from(j)];
            let d = model::apply(x, p);
            let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
            if len < 1e-9 {
                continue;
            }
            let k = 0.38 / len.max(1.0);
            let a = v(centre[0] + unit * p[0], centre[1] + unit * p[1]);
            let b = v(a[0] + unit * k * d[0], a[1] + unit * k * d[1]);
            let m = arrow_parts(s, a, b, position_c(), 2.5, 0.1);
            let _ = s.attach(g, m);
        }
    }
    g
}

fn entry(x: f64) -> String {
    let r = (x * 1000.0).round() / 1000.0;
    if r.fract() == 0.0 {
        format!("{}", r as i64)
    } else {
        format!("{r}")
    }
}

fn card_of(m: M2) -> [[String; 2]; 2] {
    [[entry(m[0][0]), entry(m[0][1])], [entry(m[1][0]), entry(m[1][1])]]
}

/// A matrix card for `m`'s entries, rounded for the screen.
fn card(kit: &Kit, stage: &mut Stage<'_>, m: M2, at: Vec3, size: f64, c: Srgb) -> fmn::Result<Mob> {
    let e = card_of(m);
    matrix_card(
        kit,
        stage,
        [
            [e[0][0].as_str(), e[0][1].as_str()],
            [e[1][0].as_str(), e[1][1].as_str()],
        ],
        at,
        size,
        c,
    )
}

impl SceneConstruct for MatrixLogs {
    fn name(&self) -> &str {
        "09_matrix_logs"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (green, blue, gold, purple, teal) = (number_c(), position_c(), answer_c(), lap_c(), slide_c());
        let right = 1.4;

        // 09_1: a matrix moves every point; four kinds of motion
        kit.say(stage, "09_1")?;
        let heading = title(&kit, stage, "Matrices move the plane")?;
        fade_in_all(stage, &[heading], 0.5)?;
        kit.until_sentence(stage, "09_1", 1)?;
        let target = stage.add_value_tracker(0.0);
        let pick = stage.add_value_tracker(0.0);
        let examples: [M2; 4] = [
            model::rotation(TAU / 6.0),
            [[1.6, 0.0], [0.0, 0.6]],
            [[1.0, 1.0], [0.0, 1.0]],
            [[-1.0, 0.0], [0.0, 1.0]],
        ];
        let moved = stage.always_redraw(move |s: &mut Arena| {
            let k = (value(s, pick).round() as usize).min(3);
            let m = lerp(I2, examples[k], value(s, target));
            picture(s, m, CENTRE, UNIT, 3)
        });
        stage.add_to_scene(moved)?;
        kit.until_sentence(stage, "09_1", 2)?;
        let arrows = kit.text_left(
            stage,
            "the matrix's columns: where the two unit arrows land",
            22.0,
            Palette::ink(),
            v(right, 2.35),
        )?;
        fade_in_all(stage, &[arrows], 0.4)?;
        kit.until_sentence(stage, "09_1", 3)?;
        let slide_note = kit.text(
            stage,
            "each grid slides straight from before to after (not a flow)",
            18.0,
            Palette::dim(),
            v(CENTRE[0], -3.25),
        )?;
        fade_in_all(stage, &[slide_note], 0.3)?;
        let names = ["turned", "stretched", "sheared", "flipped"];
        let mut shown_card: Option<Mob> = None;
        let mut word: Option<Mob> = None;
        for (k, m) in examples.iter().enumerate() {
            if k > 0 {
                play!(stage; tween(target, 0.0, 0.35)?);
            }
            stage.set_tracker_value(pick, k as f64)?;
            let new_card = card(&kit, stage, *m, v(4.4, 0.6), 34.0, Palette::ink())?;
            let new_word = kit.text(stage, names[k], 28.0, gold, v(4.4, -0.75))?;
            match (shown_card, word) {
                (Some(c), Some(w)) => swap(stage, &[c, w], &[new_card, new_word], 0.25)?,
                _ => fade_in_all(stage, &[new_card, new_word], 0.3)?,
            }
            shown_card = Some(new_card);
            word = Some(new_word);
            play!(stage; tween(target, 1.0, 0.8)?);
        }

        // 09_2: X as a velocity; follow the arrows for one unit of time
        kit.say(stage, "09_2")?;
        let series = kit.tex(
            stage,
            r"e^{X} = I + X + \tfrac{1}{2}X^{2} + \tfrac{1}{6}X^{3} + \cdots",
            32.0,
            Palette::ink(),
            &[],
            v(3.9, 2.35),
        )?;
        let mut gone = vec![moved, arrows, slide_note];
        gone.extend(shown_card.into_iter().chain(word));
        fade_out_all(stage, &gone, 0.4)?;
        fade_in_all(stage, &[series], 0.5)?;
        kit.until_sentence(stage, "09_2", 2)?;
        let x = fact_matrix("matrix.flow")?;
        let arrows_field = stage.always_redraw(move |s: &mut Arena| field(s, x, CENTRE, UNIT));
        stage.add_to_scene(arrows_field)?;
        let x_card = card(&kit, stage, x, v(4.6, 0.85), 30.0, blue)?;
        let x_word = kit.text(stage, "X, a velocity", 24.0, blue, v(2.6, 0.85))?;
        fade_in_all(stage, &[x_card, x_word], 0.5)?;
        kit.until_sentence(stage, "09_2", 3)?;
        let time = stage.add_value_tracker(0.0);
        let flowing =
            stage.always_redraw(move |s: &mut Arena| picture(s, model::flow(x, value(s, time)), CENTRE, UNIT, 3));
        stage.add_to_scene(flowing)?;
        let clock_word = kit.text_left(stage, "time t =", 24.0, Palette::ink(), v(right + 0.4, -0.45))?;
        let clock = kit.live_value(
            stage,
            move |s| value(s, time),
            2,
            false,
            26.0,
            Palette::ink(),
            v(right + 1.85, -0.45),
            LEFT,
        )?;
        fade_in_all(stage, &[clock_word], 0.3)?;
        play!(stage; tween(time, 1.0, 3.4)?);
        kit.until_sentence(stage, "09_2", 4)?;
        let e_card = card(&kit, stage, model::flow(x, 1.0), v(4.6, -1.5), 30.0, green)?;
        let e_word = kit.tex(stage, r"e^{X} =", 30.0, green, &[], v(2.8, -1.5))?;
        fade_in_all(stage, &[e_card, e_word], 0.5)?;

        // 09_3: a logarithm of A: a velocity that lands on A at t = 1
        kit.say(stage, "09_3")?;
        let def = kit.tex(
            stage,
            r"X \text{ is a logarithm of } A \iff e^{X} = A",
            32.0,
            gold,
            &[],
            v(3.6, -2.45),
        )?;
        fade_in_all(stage, &[def], 0.5)?;
        kit.until_sentence(stage, "09_3", 1)?;
        let motion = kit.text_left(
            stage,
            "existence becomes a question about motion",
            22.0,
            Palette::ink(),
            v(right, 2.35),
        )?;
        swap(stage, &[series], &[motion], 0.4)?;
        kit.until_sentence(stage, "09_3", 2)?;
        let ask = kit.text_left(
            stage,
            "is there a steady flow that ends at A?",
            24.0,
            gold,
            v(right, 1.85),
        )?;
        fade_in_all(stage, &[ask], 0.4)?;

        // 09_4: the quarter turn, reached by three spins
        kit.say(stage, "09_4")?;
        fade_out_all(
            stage,
            &[
                arrows_field,
                flowing,
                clock,
                clock_word,
                x_card,
                x_word,
                e_card,
                e_word,
                motion,
                ask,
            ],
            0.5,
        )?;
        let heading2 = title(&kit, stage, "Three ways to reach a quarter turn")?;
        swap(stage, &[heading], &[heading2], 0.4)?;
        let speeds = fact_list("matrix.quarter_turn_speeds")?;
        let spin = stage.add_value_tracker(0.0);
        let mut panels = Vec::new();
        let mut labels = Vec::new();
        for (k, &rate) in speeds.iter().enumerate() {
            let c = SMALL[k];
            let xk = model::mscale(J, TAU * rate);
            let p =
                stage.always_redraw(move |s: &mut Arena| picture(s, model::flow(xk, value(s, spin)), c, SMALL_UNIT, 2));
            panels.push(p);
            let words = match k {
                0 => format!("{} turn per unit time", entry(rate)),
                1 => format!("{} turns per unit time", entry(rate)),
                _ => format!("{} turn the other way", entry(rate.abs())),
            };
            labels.push(kit.text(stage, &super::draw::shown(&words), 20.0, purple, v(c[0], c[1] - 1.75))?);
        }
        kit.until_sentence(stage, "09_4", 1)?;
        stage.add_to_scene(panels[0])?;
        fade_in_all(stage, &[labels[0]], 0.4)?;
        let target_card = matrix_card(&kit, stage, [["0", "-1"], ["1", "0"]], v(5.4, 1.2), 30.0, gold)?;
        let target_word = kit.text(stage, "quarter turn", 22.0, gold, v(5.4, 2.1))?;
        fade_in_all(stage, &[target_card, target_word], 0.4)?;
        kit.until_sentence(stage, "09_4", 2)?;
        stage.add_to_scene(panels[1])?;
        fade_in_all(stage, &[labels[1]], 0.4)?;
        kit.until_sentence(stage, "09_4", 3)?;
        stage.add_to_scene(panels[2])?;
        fade_in_all(stage, &[labels[2]], 0.4)?;
        play!(stage; tween(spin, 1.0, 4.0)?);
        let all_three = kit.text(
            stage,
            "all three end on the same quarter turn",
            22.0,
            gold,
            v(-1.7, -2.45),
        )?;
        fade_in_all(stage, &[all_three], 0.4)?;
        kit.until_sentence(stage, "09_4", 4)?;
        let column = kit.tex(
            stage,
            r"X = \left(\tfrac{\pi}{2} + 2\pi k\right) J \quad\longleftrightarrow\quad \log i = \left(\tfrac{\pi}{2} + 2\pi k\right) i",
            28.0,
            purple,
            &[],
            v(0.0, -3.05),
        )?;
        fade_in_all(stage, &[column], 0.5)?;
        kit.until_sentence(stage, "09_4", 5)?;
        let dictionary = kit.text(
            stage,
            "turn-and-scale matrix ↔ complex number",
            20.0,
            teal,
            v(5.4, 0.05),
        )?;
        let pair = matrix_card(&kit, stage, [["a", "-b"], ["b", "a"]], v(4.9, -0.75), 26.0, teal)?;
        let as_c = kit.tex(stage, r"\leftrightarrow\ a + bi", 26.0, teal, &[], v(6.2, -0.75))?;
        fade_in_all(stage, &[dictionary, pair, as_c], 0.5)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_three_spins_all_reach_the_quarter_turn() {
        for rate in [0.25, 1.25, -0.75] {
            let a = model::flow(model::mscale(J, TAU * rate), 1.0);
            assert!(model::mdist(a, J) < 1e-9, "{rate}");
        }
    }

    #[test]
    fn the_flow_lands_on_its_series_exponential_and_keeps_area_positive() {
        let x = [[0.3, -1.0], [0.8, -0.1]];
        let a = model::flow(x, 1.0);
        assert!(model::mdist(a, model::mexp(x)) < 1e-12);
        for k in 0..=20 {
            let t = f64::from(k) / 10.0;
            assert!(model::det(model::flow(x, t)) > 0.0);
        }
    }

    #[test]
    fn the_small_panels_do_not_overlap() {
        // a rotated n = 2 grid reaches √8 units from its centre
        let reach = SMALL_UNIT * 8f64.sqrt();
        for w in SMALL.windows(2) {
            assert!(w[1][0] - w[0][0] > 2.0 * reach - 0.1, "{reach}");
        }
        assert!(SMALL[0][0] - reach > -7.1 && SMALL[2][0] + reach < 4.3);
    }

    #[test]
    fn matrix_entries_print_cleanly() {
        assert_eq!(entry(-1.0), "-1");
        assert_eq!(entry(0.30000000000000004), "0.3");
        assert_eq!(card_of(J)[0][1], "-1");
    }
}
