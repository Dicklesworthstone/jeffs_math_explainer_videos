//! Chapter 11, transformations no steady real flow reaches: a mirror, and
//! the straight slide that flattens the plane on its way (11_1); the signed
//! area of a unit square under a flow, on a gauge it never crosses at zero,
//! forwards or backwards (11_2); a positive determinant is not enough:
//! diag(−2, −½), with a prediction (11_3); why: a logarithm commutes with
//! A, so it keeps both axes, so it only stretches; a half turn and then a
//! stretch get there with two different flows, never one (11_4); flipping
//! both axes alike is a half turn, the 2×2 rule and Culver's exact
//! statement (11_5); complex entries remove the obstacle, and only
//! flattening matrices have no logarithm (11_6).
//!
//! Screen statements and their sources: det e^{X} = e^{tr X}, and for a
//! time-dependent linear flow det = e^{∫ tr} (Liouville's formula);
//! W. J. Culver, "On the existence and uniqueness of the real logarithm of
//! a matrix", Proc. Amer. Math. Soc. 17 (1966), no. 5, 1146–1151, doi
//! 10.1090/S0002-9939-1966-0202740-6 (volume, pages and the theorem's
//! wording matched to secondary statements, e.g. arXiv 2512.23735; the
//! paper's own text not yet read); N. J. Higham, Functions of
//! Matrices (SIAM 2008), Theorem 1.27 (every nonsingular complex matrix has
//! a logarithm).
//!
//! Screen layout: the plane panel at the left (x −5.8..−1.4), the signed
//! area gauge at x 0.9, a text column from x 2.2.
//!
//! Rebuild candidate for `mx-video-program-program-87do.63`; uncompiled,
//! unrendered, unlistened.

use std::f64::consts::PI;

use explainer_kit::{Kit, Palette, play, tween, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::draw::{
    answer_c, arrow, clipped_grid, fact, fact_f64, fact_list, fact_matrix, fade_in_all, fade_out_all, flat, lap_c,
    letter_f, matrix_card, number_c, position_c, precisely, predict, ring, seg, shown, slide_c, swap, title, value,
    warn_c,
};
use crate::model::{self, I2, J, M2};

pub struct Unreachable {
    pub kit: Kit,
}

const CENTRE: [f64; 3] = [-3.6, -0.3, 0.0];
const UNIT: f64 = 0.7;
/// Grid lines are kept within this many plane units of the centre.
const CLIP: f64 = 3.2;
/// The signed-area gauge: its x, the screen height of det = 0, and the
/// screen length of one unit of area; it shows −1.3 to 2.2.
const GAUGE_X: f64 = 0.9;
const GAUGE_ZERO: f64 = -0.9;
const GAUGE_PER: f64 = 1.35;
const GAUGE_RANGE: (f64, f64) = (-1.3, 2.2);
/// The flow's clock runs over this range in 11_2 (forwards, then back).
const FLOW_TIMES: (f64, f64) = (-2.0, 3.0);

fn gauge(d: f64) -> Vec3 {
    v(GAUGE_X, GAUGE_ZERO + GAUGE_PER * d)
}

fn lerp(a: M2, b: M2, t: f64) -> M2 {
    model::madd(model::mscale(a, 1.0 - t), model::mscale(b, t))
}

/// A number for the screen: halves as fractions, a typographic minus.
fn shown_num(x: f64) -> String {
    if (x - 0.5).abs() < 1e-12 {
        "½".into()
    } else if (x + 0.5).abs() < 1e-12 {
        "\u{2212}½".into()
    } else {
        shown(&format!("{x}"))
    }
}

fn to_screen(q: [f64; 2]) -> Vec3 {
    v(CENTRE[0] + UNIT * q[0], CENTRE[1] + UNIT * q[1])
}

/// The plane under `m`, for a redraw: clipped grid, optionally the unit
/// square (blue, its signed area on the gauge), and the letter F.
fn plane(s: &mut Arena, m: M2, square: bool) -> Mob {
    let g = s.add(VMobject::new());
    let grid = clipped_grid(s, m, CENTRE, UNIT, 4, CLIP, number_c(), 0.6);
    let _ = s.attach(g, grid);
    if square {
        let corners: Vec<Vec3> = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
            .iter()
            .map(|&p| to_screen(model::apply(m, p)))
            .collect();
        let sq = s.add(Polygon::new(corners).style(flat(position_c(), 0.45)).build());
        let _ = s.attach(g, sq);
    }
    let f = s.add(
        Polygon::new(letter_f(m, CENTRE, UNIT))
            .style(flat(answer_c(), 0.9))
            .build(),
    );
    let _ = s.attach(g, f);
    g
}

/// The two-stage journey to diag(−e^{a}, −e^{b}) at time `t` in [0, 2]: a
/// half-turn spin (generator πJ) for t ≤ 1, then the stretch
/// diag(e^{a(t−1)}, e^{b(t−1)}) applied after −I. Two steady flows in turn.
fn two_stage(t: f64, a: f64, b: f64) -> M2 {
    if t <= 1.0 {
        model::flow(model::mscale(J, PI), t)
    } else {
        let u = t - 1.0;
        model::mmul([[(a * u).exp(), 0.0], [0.0, (b * u).exp()]], model::mscale(I2, -1.0))
    }
}

/// A steady flow that keeps both axes: diag(e^{ta}, e^{tb}).
fn diagonal_flow(a: f64, b: f64, t: f64) -> M2 {
    [[(a * t).exp(), 0.0], [0.0, (b * t).exp()]]
}

impl SceneConstruct for Unreachable {
    fn name(&self) -> &str {
        "11_unreachable"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (green, blue, gold, purple, red) = (number_c(), position_c(), answer_c(), lap_c(), warn_c());
        let right = 2.2;
        let mirror: M2 = [[-1.0, 0.0], [0.0, 1.0]];
        let pair = fact_list("matrix.flip_pair")?;
        let flip_pair: M2 = [[pair[0], 0.0], [0.0, pair[1]]];
        // the stretch rates of the flip pair's halves, exact for the motion
        let (la, lb) = (pair[0].abs().ln(), pair[1].abs().ln());

        // 11_1: a mirror; the straight slide flattens the F on its way
        kit.say(stage, "11_1")?;
        let heading = title(&kit, stage, "Places no steady flow reaches")?;
        fade_in_all(stage, &[heading], 0.5)?;
        kit.until_sentence(stage, "11_1", 1)?;
        let m_card = matrix_card(&kit, stage, [["-1", "0"], ["0", "1"]], v(5.4, 1.6), 32.0, red)?;
        let m_word = kit.text(stage, "mirror", 24.0, red, v(5.4, 2.2))?;
        fade_in_all(stage, &[m_card, m_word], 0.5)?;
        kit.until_sentence(stage, "11_1", 2)?;
        let slide = stage.add_value_tracker(0.0);
        let mirrored = stage.always_redraw(move |s: &mut Arena| plane(s, lerp(I2, mirror, value(s, slide)), false));
        stage.add_to_scene(mirrored)?;
        kit.until_sentence(stage, "11_1", 3)?;
        play!(stage; tween(slide, 0.5, 1.4)?);
        let flat_word = kit.text_left(
            stage,
            "a straight slide squashes it flat half way",
            22.0,
            red,
            v(right, 0.6),
        )?;
        fade_in_all(stage, &[flat_word], 0.3)?;
        play!(stage; tween(slide, 1.0, 1.4)?);

        // 11_2: the signed area along a flow starts at 1 and never reaches 0
        kit.say(stage, "11_2")?;
        fade_out_all(stage, &[mirrored, flat_word], 0.4)?;
        let x = fact_matrix("matrix.flow")?;
        let time = stage.add_value_tracker(0.0);
        let flowing = stage.always_redraw(move |s: &mut Arena| plane(s, model::flow(x, value(s, time)), true));
        stage.add_to_scene(flowing)?;
        let mut scale = vec![seg(
            stage,
            gauge(GAUGE_RANGE.0),
            gauge(GAUGE_RANGE.1),
            Palette::faint(),
            2.5,
        )];
        for k in -1..=2 {
            let y = gauge(f64::from(k))[1];
            scale.push(seg(
                stage,
                v(GAUGE_X - 0.1, y),
                v(GAUGE_X + 0.1, y),
                Palette::dim(),
                2.0,
            ));
            scale.push(kit.text(
                stage,
                &shown(&k.to_string()),
                18.0,
                Palette::dim(),
                v(GAUGE_X - 0.35, y),
            )?);
        }
        let zero_line = seg(
            stage,
            v(GAUGE_X - 0.35, GAUGE_ZERO),
            v(GAUGE_X + 0.35, GAUGE_ZERO),
            red,
            4.0,
        );
        let area_word = kit.text(stage, "signed area of the square:", 22.0, blue, v(GAUGE_X, 2.45))?;
        fade_in_all(stage, &scale, 0.4)?;
        fade_in_all(stage, &[zero_line, area_word], 0.4)?;
        let needle = stage.always_redraw(move |s: &mut Arena| {
            let d = model::det(model::flow(x, value(s, time)));
            s.add(Dot::new().point(gauge(d)).radius(0.1).style(flat(blue, 1.0)).build())
        });
        stage.add_to_scene(needle)?;
        let det_val = kit.live_value(
            stage,
            move |s| model::det(model::flow(x, value(s, time))),
            3,
            false,
            24.0,
            blue,
            v(2.0, 2.45),
            LEFT,
        )?;
        kit.until_sentence(stage, "11_2", 1)?;
        let one = ring(stage, gauge(1.0), 0.16, blue, 2.5);
        fade_in_all(stage, &[one], 0.3)?;
        kit.until_sentence(stage, "11_2", 2)?;
        play!(stage; tween(time, 1.0, 1.6)?);
        let d1 = fact_f64("matrix.flow_det_t1")?;
        let t1 = kit.text_left(
            stage,
            &format!("t = 1: {}", fact("matrix.flow_det_t1")?),
            18.0,
            Palette::dim(),
            v(GAUGE_X + 0.3, gauge(d1)[1]),
        )?;
        fade_in_all(stage, &[t1], 0.3)?;
        play!(stage; tween(time, FLOW_TIMES.1, 2.0)?);
        let d3 = fact_f64("matrix.flow_det_t3")?;
        let t3 = kit.text_left(
            stage,
            &format!("t = 3: {}", fact("matrix.flow_det_t3")?),
            18.0,
            Palette::dim(),
            v(GAUGE_X + 0.3, gauge(d3)[1]),
        )?;
        fade_in_all(stage, &[t3], 0.3)?;
        kit.until_sentence(stage, "11_2", 3)?;
        play!(stage; tween(time, FLOW_TIMES.0, 3.0)?);
        let backwards = kit.text_left(
            stage,
            "run backwards: never flat, never zero",
            22.0,
            Palette::ink(),
            v(right, -1.4),
        )?;
        fade_in_all(stage, &[backwards], 0.4)?;
        kit.until_sentence(stage, "11_2", 4)?;
        let positive = seg(stage, v(GAUGE_X, GAUGE_ZERO + 0.06), gauge(GAUGE_RANGE.1), green, 6.0);
        let stays = kit.text_left(
            stage,
            "starts at 1, never touches 0: stays positive",
            22.0,
            green,
            v(right, -1.9),
        )?;
        fade_in_all(stage, &[positive, stays], 0.5)?;
        play!(stage; tween(time, 0.0, 1.2)?);
        kit.until_sentence(stage, "11_2", 5)?;
        let mirror_det = fact_f64("matrix.mirror_det")?;
        let mirror_mark = ring(stage, gauge(mirror_det), 0.16, red, 3.5);
        let mirror_word = kit.text_left(
            stage,
            &format!("mirror: {}", shown(fact("matrix.mirror_det")?)),
            20.0,
            red,
            v(GAUGE_X + 0.3, gauge(mirror_det)[1]),
        )?;
        let liouville = precisely(
            &kit,
            stage,
            "for any real linear flow, steady or not, det = e^(∫ trace dt) > 0 (Liouville)",
        )?;
        fade_in_all(stage, &[mirror_mark, mirror_word, liouville], 0.5)?;

        // 11_3: det < 0 rules a real logarithm out; det > 0 is not enough
        kit.say(stage, "11_3")?;
        let det_word = kit.text(stage, "det = signed area factor:", 22.0, gold, v(GAUGE_X, 2.45))?;
        swap(stage, &[area_word, m_card, m_word], &[det_word], 0.4)?;
        kit.until_sentence(stage, "11_3", 1)?;
        let no_neg = kit.text_left(stage, "det < 0: no real logarithm", 24.0, red, v(right + 0.5, 2.3))?;
        fade_in_all(stage, &[no_neg], 0.4)?;
        kit.until_sentence(stage, "11_3", 2)?;
        let mut gone = scale.clone();
        gone.extend([
            flowing,
            needle,
            det_val,
            det_word,
            zero_line,
            one,
            t1,
            t3,
            backwards,
            positive,
            stays,
            mirror_mark,
            mirror_word,
            liouville,
        ]);
        fade_out_all(stage, &gone, 0.5)?;
        let enough = kit.text_left(stage, "det > 0: enough?", 26.0, gold, v(right, 1.7))?;
        fade_in_all(stage, &[enough], 0.4)?;
        kit.until_sentence(stage, "11_3", 3)?;
        let grid = clipped_grid(stage.arena_mut(), I2, CENTRE, UNIT, 4, CLIP, green, 0.5);
        let before = stage.arena_mut().add(
            Polygon::new(letter_f(I2, CENTRE, UNIT))
                .style(flat(Palette::faint(), 0.8))
                .build(),
        );
        let a_word = kit.text_left(stage, "A =", 28.0, gold, v(right, 1.05))?;
        let (e00, e11) = (shown_num(pair[0]), shown_num(pair[1]));
        let a_card = matrix_card(
            &kit,
            stage,
            [[e00.as_str(), "0"], ["0", e11.as_str()]],
            v(right + 1.45, 1.05),
            30.0,
            gold,
        )?;
        fade_in_all(stage, &[grid, before, a_word, a_card], 0.5)?;
        kit.until_sentence(stage, "11_3", 4)?;
        let o = to_screen([0.0, 0.0]);
        let e1 = arrow(stage, o, to_screen([pair[0], 0.0]), green, 4.0)?;
        let e1_word = kit.text(
            stage,
            &format!("×{}, flipped", shown_num(-pair[0])),
            20.0,
            green,
            v(to_screen([pair[0], 0.0])[0], CENTRE[1] + 0.35),
        )?;
        fade_in_all(stage, &[e1, e1_word], 0.5)?;
        kit.until_sentence(stage, "11_3", 5)?;
        let e2 = arrow(stage, o, to_screen([0.0, pair[1]]), slide_c(), 4.0)?;
        let e2_word = kit.text_left(
            stage,
            &format!("×{}, flipped", shown_num(-pair[1])),
            20.0,
            slide_c(),
            v(CENTRE[0] + 0.25, to_screen([0.0, pair[1]])[1] - 0.2),
        )?;
        let after = stage.arena_mut().add(
            Polygon::new(letter_f(flip_pair, CENTRE, UNIT))
                .style(flat(gold, 0.9))
                .build(),
        );
        fade_in_all(stage, &[e2, e2_word, after], 0.5)?;
        kit.until_sentence(stage, "11_3", 6)?;
        let area = kit.text_left(
            stage,
            &format!(
                "det = ({}) × ({}) = +{}",
                shown_num(pair[0]),
                shown_num(pair[1]),
                fact("matrix.flip_pair_det")?
            ),
            24.0,
            Palette::ink(),
            v(right, 0.35),
        )?;
        fade_in_all(stage, &[area], 0.4)?;
        kit.until_sentence(stage, "11_3", 7)?;
        let split = kit.text_left(
            stage,
            &format!("A = (−I) × diag({}, {})", shown_num(-pair[0]), shown_num(-pair[1])),
            24.0,
            purple,
            v(right, -0.25),
        )?;
        let split_word = kit.text_left(stage, "a half turn, then a stretch", 20.0, purple, v(right, -0.8))?;
        fade_in_all(stage, &[split, split_word], 0.4)?;
        kit.until_sentence(stage, "11_3", 8)?;
        predict(&kit, stage, v(right + 1.6, -1.4), 3.0)?;

        // 11_4: no; X commutes with A, keeps both axes, so only stretches
        kit.say(stage, "11_4")?;
        let no = kit.text_left(stage, "det > 0: not enough", 26.0, red, v(right, 1.7))?;
        swap(stage, &[enough], &[no], 0.3)?;
        kit.until_sentence(stage, "11_4", 1)?;
        let commute = kit.tex(
            stage,
            r"A = e^{X} = I + X + \tfrac{1}{2}X^{2} + \text{...} \ \Rightarrow\ XA = AX",
            26.0,
            Palette::ink(),
            &[],
            v(4.6, 2.3),
        )?;
        swap(stage, &[no_neg, area], &[commute], 0.4)?;
        kit.until_sentence(stage, "11_4", 2)?;
        let reach = UNIT * CLIP;
        let ax = seg(
            stage,
            v(CENTRE[0] - reach, CENTRE[1]),
            v(CENTRE[0] + reach, CENTRE[1]),
            gold,
            5.0,
        );
        let ay = seg(
            stage,
            v(CENTRE[0], CENTRE[1] - reach),
            v(CENTRE[0], CENTRE[1] + reach),
            gold,
            5.0,
        );
        let keeps = kit.text_left(
            stage,
            &format!("{} ≠ {}: X must keep each axis", shown_num(pair[0]), shown_num(pair[1])),
            22.0,
            gold,
            v(right, 0.35),
        )?;
        fade_in_all(stage, &[ax, ay, keeps], 0.5)?;
        kit.until_sentence(stage, "11_4", 3)?;
        // a steady flow that keeps both axes is diagonal: it only stretches
        fade_out_all(stage, &[grid, before, after, e1, e1_word, e2, e2_word], 0.4)?;
        let diag = stage.add_value_tracker(0.0);
        let stretching =
            stage.always_redraw(move |s: &mut Arena| plane(s, diagonal_flow(la, lb, value(s, diag)), true));
        stage.add_to_scene(stretching)?;
        play!(stage; tween(diag, 1.0, 2.0)?);
        let never = kit.text_left(
            stage,
            "keeps both axes: stretches, never flips",
            22.0,
            blue,
            v(right, -1.4),
        )?;
        fade_in_all(stage, &[never], 0.4)?;
        play!(stage; tween(diag, -1.0, 2.0)?);
        kit.until_sentence(stage, "11_4", 4)?;
        fade_out_all(stage, &[stretching], 0.3)?;
        let journey = stage.add_value_tracker(0.0);
        let two = stage.always_redraw(move |s: &mut Arena| plane(s, two_stage(value(s, journey), la, lb), true));
        stage.add_to_scene(two)?;
        let stage_word = kit.text_left(
            stage,
            "a half turn, then a stretch: two flows in turn",
            22.0,
            purple,
            v(right, -2.0),
        )?;
        fade_in_all(stage, &[stage_word], 0.3)?;
        play!(stage; tween(journey, 1.0, 2.0)?);
        play!(stage; tween(journey, 2.0, 1.8)?);
        kit.until_sentence(stage, "11_4", 5)?;
        let never_one = kit.text_left(stage, "no single steady flow does both", 26.0, red, v(right, -2.6))?;
        fade_in_all(stage, &[never_one], 0.4)?;

        // 11_5: flipping both axes alike is a half turn; the exact rule
        kit.say(stage, "11_5")?;
        let away = [
            two, stage_word, never_one, never, ax, ay, keeps, commute, a_word, a_card, split, split_word, no,
        ];
        fade_out_all(stage, &away, 0.5)?;
        let alike = kit.text_left(stage, "both axes flipped alike:", 24.0, green, v(right, 2.3))?;
        let minus_word = kit.text_left(stage, "−I =", 28.0, green, v(right, 1.6))?;
        let minus_i = matrix_card(
            &kit,
            stage,
            [["-1", "0"], ["0", "-1"]],
            v(right + 1.45, 1.6),
            30.0,
            green,
        )?;
        fade_in_all(stage, &[alike, minus_word, minus_i], 0.4)?;
        kit.until_sentence(stage, "11_5", 1)?;
        let turn = stage.add_value_tracker(0.0);
        let speed = fact_f64("matrix.half_turn_speed")?;
        let spinning = stage.always_redraw(move |s: &mut Arena| {
            plane(s, model::flow(model::mscale(J, 2.0 * PI * speed), value(s, turn)), true)
        });
        stage.add_to_scene(spinning)?;
        play!(stage; tween(turn, 1.0, 2.4)?);
        let spin = kit.tex(stage, r"e^{\pi J} = -I", 30.0, green, &[], v(right + 1.2, 0.75))?;
        let spin_word = kit.text_left(
            stage,
            &format!("a steady spin, {} lap", fact("matrix.half_turn_speed")?),
            20.0,
            green,
            v(right + 2.6, 0.75),
        )?;
        fade_in_all(stage, &[spin, spin_word], 0.4)?;
        kit.until_sentence(stage, "11_5", 2)?;
        let two_d = kit.text_left(
            stage,
            "2×2, with a negative eigenvalue: a real log only if A = −cI, c > 0",
            20.0,
            Palette::ink(),
            v(0.6, 0.0),
        )?;
        fade_in_all(stage, &[two_d], 0.4)?;
        kit.until_sentence(stage, "11_5", 3)?;
        let cite = precisely(&kit, stage, "Culver, Proc. Amer. Math. Soc. 17 (1966) 1146–1151")?;
        fade_in_all(stage, &[cite], 0.4)?;
        kit.until_sentence(stage, "11_5", 4)?;
        let rule_a = kit.text_left(
            stage,
            "A real invertible matrix has a real logarithm exactly when, for each",
            20.0,
            gold,
            v(0.6, -0.8),
        )?;
        let rule_b = kit.text_left(
            stage,
            "negative eigenvalue, its Jordan blocks of each size come in even numbers.",
            20.0,
            gold,
            v(0.6, -1.2),
        )?;
        fade_in_all(stage, &[rule_a, rule_b], 0.6)?;

        // 11_6: complex logarithms; only flattening matrices have none
        kit.say(stage, "11_6")?;
        fade_out_all(
            stage,
            &[
                spinning, alike, minus_word, minus_i, spin, spin_word, two_d, rule_a, rule_b, cite,
            ],
            0.5,
        )?;
        let heading2 = title(&kit, stage, "Allow complex entries")?;
        swap(stage, &[heading], &[heading2], 0.4)?;
        kit.until_sentence(stage, "11_6", 1)?;
        let flip = kit.tex(stage, r"e^{i\pi} = -1", 34.0, purple, &[], v(0.0, 2.2))?;
        fade_in_all(stage, &[flip], 0.4)?;
        let big = format!("{}", -pair[0]);
        let complex_log = kit.tex(
            stage,
            &format!(r"\log A = \operatorname{{diag}}(\ln {big} + i\pi,\ -\ln {big} + i\pi)"),
            32.0,
            blue,
            &[],
            v(0.0, 1.3),
        )?;
        let check = kit.tex(
            stage,
            &format!(r"e^{{\ln {big} + i\pi}} = -{big}, \qquad e^{{-\ln {big} + i\pi}} = -\tfrac{{1}}{{{big}}}"),
            28.0,
            Palette::ink(),
            &[],
            v(0.0, 0.45),
        )?;
        let logs = fact_list("matrix.flip_pair_log_real")?;
        let approx = kit.text(
            stage,
            &format!("ln {big} ≈ {}", logs[0]),
            20.0,
            Palette::dim(),
            v(0.0, -0.1),
        )?;
        fade_in_all(stage, &[complex_log, check, approx], 0.6)?;
        kit.until_sentence(stage, "11_6", 2)?;
        let every = kit.text(
            stage,
            "every invertible complex matrix has a complex logarithm",
            26.0,
            gold,
            v(0.0, -0.9),
        )?;
        let src = precisely(&kit, stage, "Higham, Functions of Matrices (SIAM 2008), Theorem 1.27")?;
        fade_in_all(stage, &[every, src], 0.5)?;
        kit.until_sentence(stage, "11_6", 3)?;
        let singular = kit.tex(
            stage,
            r"\operatorname{det} A = 0 \quad\text{but}\quad \operatorname{det} e^{X} = e^{\operatorname{tr} X} \neq 0",
            28.0,
            red,
            &[],
            v(0.0, -1.85),
        )?;
        let like_zero = kit.text(
            stage,
            "flattening matrices, like zero, have no logarithm",
            22.0,
            red,
            v(0.0, -2.5),
        )?;
        fade_in_all(stage, &[singular, like_zero], 0.5)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FLOW: M2 = [[0.3, -1.0], [0.8, -0.1]];
    const PAIR: (f64, f64) = (-2.0, -0.5);

    /// Rough fmn text width: one em is size / 144 scene units, an average
    /// character about 0.55 em.
    fn width(s: &str, size: f64) -> f64 {
        s.chars().count() as f64 * 0.55 * size / 144.0
    }

    #[test]
    fn the_straight_slide_to_the_mirror_is_flat_half_way() {
        let m = lerp(I2, [[-1.0, 0.0], [0.0, 1.0]], 0.5);
        assert!(model::det(m).abs() < 1e-12);
        // and the sign really changes along it: no slide avoids zero
        assert!(model::det(lerp(I2, [[-1.0, 0.0], [0.0, 1.0]], 0.49)) > 0.0);
        assert!(model::det(lerp(I2, [[-1.0, 0.0], [0.0, 1.0]], 0.51)) < 0.0);
    }

    #[test]
    fn signed_area_along_a_flow_is_e_to_trace_times_t_at_the_shown_times() {
        let tr = model::trace(FLOW);
        for k in -40..=60 {
            let t = f64::from(k) / 20.0;
            let d = model::det(model::flow(FLOW, t));
            assert!(d > 0.0 && (d - (tr * t).exp()).abs() < 1e-9, "{t}");
        }
        // at the ends of the clock's range, and the gauge holds them
        for t in [FLOW_TIMES.0, FLOW_TIMES.1] {
            let d = model::det(model::flow(FLOW, t));
            assert!(d > 0.0 && d > GAUGE_RANGE.0 && d < GAUGE_RANGE.1, "{t}: {d}");
        }
    }

    #[test]
    fn no_flow_reaches_negative_area_even_with_a_strongly_shrinking_generator() {
        // the edge: a trace far below zero shrinks area hard, never past zero
        for x in [
            [[-6.0, 3.0], [-2.0, -5.0]],
            [[0.0, 9.0], [-9.0, -0.4]],
            [[-0.01, 0.0], [0.0, -20.0]],
        ] {
            for t in [0.5, 2.0, 5.0] {
                assert!(model::det(model::flow(x, t)) > 0.0);
            }
        }
    }

    #[test]
    fn anything_commuting_with_the_flip_pair_is_diagonal() {
        let a = [[PAIR.0, 0.0], [0.0, PAIR.1]];
        // (XA − AX) has off-diagonal entries x01 (a11 − a00) and x10 (a00 − a11)
        for x in [
            [[0.3, 0.7], [-0.2, 0.1]],
            [[1.0, 1e-3], [0.0, 2.0]],
            [[0.0, 0.0], [1e-3, 0.0]],
        ] {
            let c = model::commutator(x, a);
            let off = x[0][1].abs() + x[1][0].abs();
            assert!((c[0][1].abs() + c[1][0].abs() - off * (PAIR.0 - PAIR.1).abs()).abs() < 1e-12);
            assert!(model::mdist(c, [[0.0; 2]; 2]) > 0.0);
        }
        assert!(model::mdist(model::commutator([[0.4, 0.0], [0.0, -3.0]], a), [[0.0; 2]; 2]) < 1e-15);
    }

    #[test]
    fn a_diagonal_flow_never_flips_an_axis() {
        for (a, b) in [(0.4, -1.0), (-3.0, 2.0), (2f64.ln(), -(2f64.ln()))] {
            for t in [-1.0, 0.5, 1.0, 4.0] {
                let e = diagonal_flow(a, b, t);
                assert!(e[0][0] > 0.0 && e[1][1] > 0.0);
            }
        }
    }

    #[test]
    fn two_flows_in_turn_reach_the_flip_pair_continuously() {
        let (la, lb) = (PAIR.0.abs().ln(), PAIR.1.abs().ln());
        let end = two_stage(2.0, la, lb);
        assert!(model::mdist(end, [[PAIR.0, 0.0], [0.0, PAIR.1]]) < 1e-12);
        assert!(model::mdist(two_stage(1.0, la, lb), two_stage(1.0 + 1e-9, la, lb)) < 1e-6);
        // the pair's determinant is +1, so −ln|a| = ln|b| (used on screen)
        assert!((PAIR.0 * PAIR.1 - 1.0).abs() < 1e-15 && (la + lb).abs() < 1e-15);
    }

    #[test]
    fn the_half_turn_spin_reaches_minus_identity() {
        let e = model::flow(model::mscale(J, 2.0 * PI * 0.5), 1.0);
        assert!(model::mdist(e, model::mscale(I2, -1.0)) < 1e-12);
    }

    #[test]
    fn the_complex_logarithm_of_the_flip_pair_exponentiates_back() {
        let (la, lb) = (PAIR.0.abs().ln(), PAIR.1.abs().ln());
        for (l, target) in [(la, PAIR.0), (lb, PAIR.1)] {
            let e = model::C::new(l, PI).exp();
            assert!((e.re - target).abs() < 1e-12 && e.im.abs() < 1e-12);
        }
    }

    #[test]
    fn the_panels_and_text_column_fit() {
        assert!(gauge(GAUGE_RANGE.0)[1] > -3.2 && gauge(GAUGE_RANGE.1)[1] < 2.3);
        assert!(CENTRE[0] + UNIT * CLIP < GAUGE_X - 0.6);
        for (s, size, x) in [
            ("a straight slide squashes it flat half way", 22.0, 2.2),
            ("starts at 1, never touches 0: stays positive", 22.0, 2.2),
            ("a half turn, then a stretch: two flows in turn", 22.0, 2.2),
            (
                "2×2, with a negative eigenvalue: a real log only if A = −cI, c > 0",
                20.0,
                0.6,
            ),
            (
                "negative eigenvalue, its Jordan blocks of each size come in even numbers.",
                20.0,
                0.6,
            ),
        ] {
            assert!(x + width(s, size) < 7.0, "{s}");
        }
        assert!(width("every invertible complex matrix has a complex logarithm", 26.0) < 13.6);
    }

    #[test]
    fn shown_numbers_use_halves_and_true_minus() {
        assert_eq!(shown_num(-0.5), "\u{2212}½");
        assert_eq!(shown_num(0.5), "½");
        assert_eq!(shown_num(-2.0), "\u{2212}2");
        assert_eq!(shown_num(2.0), "2");
    }
}
