//! Chapter 07, a matrix in: the series asks three things of x, to multiply
//! it by itself, divide it by whole numbers and add, and a square matrix can
//! do all three (07_1); so put a matrix where the x was, which is the
//! definition of e to a matrix (07_2); the quiz, J times J (07_3); a half
//! turn, minus the matrix that changes nothing, so J behaves like i, the
//! powers going round a dial beside i, minus one, minus i, one (07_4); run J
//! for a time t and the terms alternate, one along the arrow, the next at
//! right angles, spiralling in to a point on the circle (07_5); and the
//! across parts add up to the cosine and the up parts to the sine, so e to
//! the tJ is the rotation by t radians (07_6).
//!
//! Every number is a fact: J is `F-J.power.k1`, J squared `F-J.power.k2`
//! (the quiz's answer is `F-Q.07_3.answer`), the period of the powers
//! `F-J.period`, Laguerre's year `F-B.laguerre.year`, the entries at t = 1
//! `F-V.07_6.at_one`. The spiral's partial sums are the model's series at
//! t = 1, the exact route to the point of 06_4's circle.

use std::f64::consts::{FRAC_PI_2, PI, TAU};

use super::draw::{hold_band, lower_third, now};
use explainer_kit::{Kit, Palette, Timed, fade_all, play, tween, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::draw::{
    Flow, FlowSpec, MatrixGrid, Plane, QuizCard, arrow, arrow_points, corners as quad_run, entries, fact, fade_in_all,
    fade_out_all, faded_line, flat, lattice, music_hold, result, rotation_map, rule, say, second, turning,
    tween_linear,
};
use super::flow::LiveMatrix;
use crate::model::{quarter_turn, series};

pub struct AMatrixIn {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const TAG_MULTIPLY: &str = "multiply it by itself";
const TAG_DIVIDE: &str = "divide by whole numbers";
const TAG_ADD: &str = "add";
const TWICE: &str = "through the machine twice";
const HALVED: &str = "the arrow that comes out, halved";
const ADDED: &str = "the two arrows that come out, added";
const SQUARE: &str = "square: as many rows as columns";
const EXPONENTIAL: &str = "its exponential";
const WHO_AND_WHEN: &str = "who and when:";
const BEHAVES: &str = "J behaves like i";
const ALONG: &str = "along the arrow";
const AT_RIGHT_ANGLES: &str = "at right angles";
const ACROSS: &str = "across: cosine of t";
const UP: &str = "up: sine of t";

/// The three small machines of 07_1 and where they stand.
const DEMO_X: [f64; 3] = [-4.6, 0.0, 4.6];
const DEMO_Y: f64 = -0.35;
const DEMO_UNIT: f64 = 0.85;
/// The quiz card.
const CARD_AT: Vec3 = [0.0, 0.3, 0.0];
const CARD_SIZE: (f64, f64) = (9.6, 5.8);
/// Where the card's problem stands, left or right of its middle: "J² =", the
/// "?" and, at 07_4, the answer that replaces the "?" (its left bracket at
/// the old offset of -0.7 stood on the "=": 07's dense read).
const J_SQUARED_DX: f64 = -2.6;
const ASK_DX: f64 = -0.3;
/// The answer's grid: its type size and offset (at 70 points it is 3.0 wide,
/// and no offset leaves room on both sides: "J^2 =" ends 1.64 left of the
/// middle, the fan begins 1.8 right of it).
const ANSWER_PT: f64 = 64.0;
const ANSWER_DX: f64 = 0.0;
/// The plane of the spiral (07_5, 07_6) and how far the spiral is drawn.
const SPIRAL: Plane = Plane {
    centre: [-3.4, -0.5, 0.0],
    unit: 2.3,
};
const SPIRAL_TERMS: usize = 14;
/// What the first terms of 07_5's spiral are called and where each name
/// stands, in the spiral's plane units. "t²/2" stands over the spiral's
/// top segment (at 1.0), its foot 0.15 or more clear of it: at 1.2 the
/// segment ran through the fraction's denominator.
const TERM_NAMES: [&str; 4] = ["1", "t", "\\frac{t^{2}}{2}", "\\frac{t^{3}}{6}"];
const TERM_SPOTS: [[f64; 2]; 4] = [[0.5, -0.17], [1.2, 0.5], [0.5, 1.36], [0.2, 0.9]];
/// The dial of powers of 07_4.
const DIAL_AT: Vec3 = [3.6, -0.5, 0.0];
const DIAL_R: f64 = 1.5;
/// The row "J² = [-1 0; 0 -1] = -I" of 07_4 and where its middle stands (at
/// 2.9 the grid's bracket was 0.16 from the frame's top edge), and where the
/// fan of 07_4 stands.
const SQUARE_ROW_Y: f64 = 2.65;
const FAN_AT: Vec3 = [-3.7, -0.3, 0.0];
const FAN_REACH: f64 = 1.6;
/// The fan of 07_3 and 07_4: arrows and the longest reach.
const FAN_N: usize = 8;

/// The row of 07_4, one place for the chapter and its test: "J² =", the grid
/// of J² and "= -I", across the top of the frame at `SQUARE_ROW_Y`.
fn square_row(kit: &Kit, stage: &mut Stage<'_>, rows: &[Vec<String>]) -> fmn::Result<(Mob, MatrixGrid, Mob)> {
    let words = kit.tex(stage, "J^{2} =", 80.0, Palette::ink(), &[], v(-2.4, SQUARE_ROW_Y))?;
    let grid = MatrixGrid::build(kit, stage, rows, 56.0, result(), v(0.2, SQUARE_ROW_Y))?;
    let minus_i = kit.tex(stage, "= -I", 80.0, result(), &[], v(2.7, SQUARE_ROW_Y))?;
    Ok((words, grid, minus_i))
}

/// The partial sums of e^{tJ} applied to the arrow (1, 0), from the origin:
/// the corners of the spiral, term by term.
pub fn spiral_corners(t: f64, terms: usize) -> Vec<[f64; 2]> {
    let steps = series(&quarter_turn().scale(t), terms as u32);
    let mut out = vec![[0.0, 0.0]];
    out.extend(steps.iter().map(|s| {
        let c = s.total.column(0);
        [c[0], c[1]]
    }));
    out
}

/// The polyline of the spiral drawn up to `reach` terms (a fraction draws
/// the next segment part of the way), on `SPIRAL`.
fn spiral_polyline(corners: &[[f64; 2]], reach: f64) -> Vec<Vec3> {
    let whole = (reach.floor() as usize).min(corners.len() - 1);
    let mut pts: Vec<Vec3> = corners[..=whole].iter().map(|c| SPIRAL.p(c[0], c[1])).collect();
    if whole + 1 < corners.len() {
        let (a, b) = (corners[whole], corners[whole + 1]);
        let f = reach - reach.floor();
        pts.push(SPIRAL.p(a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f));
    }
    if pts.len() < 2 {
        pts.push(v(pts[0][0] + 1e-4, pts[0][1]));
    }
    pts
}

/// The eight arrows of the fan, as polylines from `at`, at radius `r`.
fn fan_ends(at: Vec3, r: f64, turn: f64) -> Vec<(Vec3, Vec3)> {
    (0..FAN_N)
        .map(|k| {
            let a = TAU * k as f64 / FAN_N as f64 + turn;
            (at, v(at[0] + r * a.cos(), at[1] + r * a.sin()))
        })
        .collect()
}

impl SceneConstruct for AMatrixIn {
    fn name(&self) -> &str {
        "07_a_matrix_in"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let ink = Palette::ink();
        let j = quarter_turn();
        let mk_line = |stage: &mut Stage<'_>, pts: Vec<Vec3>, color: Srgb, width: f64, o: f64| {
            stage
                .arena_mut()
                .add(polyline(pts).style(faded_line(color, width, o)).build())
        };

        // 07_1: the series asks three things of x; a square matrix can do
        // all three; each shown on the machine
        let line = say(&kit, stage, "07_1")?;
        let series_x = kit.tex(
            stage,
            "e^{x} = 1 + x + \\frac{x^{2}}{2!} + \\frac{x^{3}}{3!} + \\cdots",
            70.0,
            ink,
            &[],
            v(0.0, 2.85),
        )?;
        let tags = [
            kit.text(stage, TAG_MULTIPLY, 32.0, rule(), v(-4.6, 1.7))?,
            kit.text(stage, TAG_DIVIDE, 32.0, rule(), v(0.0, 1.7))?,
            kit.text(stage, TAG_ADD, 32.0, rule(), v(4.6, 1.7))?,
        ];
        // three small planes, each with its arrows
        let demo = |at_x: f64| Plane {
            centre: [at_x, DEMO_Y, 0.0],
            unit: DEMO_UNIT,
        };
        let v_in = [1.0, 0.3];
        let out = j.apply(&v_in);
        let twice = j.apply(&out);
        let b_out = [v_in[0] * 1.0, 0.0];
        let mut pieces: Vec<Vec<Mob>> = Vec::new();
        {
            // A.A: through the machine twice
            let p = demo(DEMO_X[0]);
            let mut m = p.axes(stage, 1.4, 1.4);
            m.push(arrow(stage, p.p(0.0, 0.0), p.p(v_in[0], v_in[1]), ink, 4.0)?);
            m.push(arrow(stage, p.p(0.0, 0.0), p.p(out[0], out[1]), turning(), 4.0)?);
            m.push(arrow(stage, p.p(0.0, 0.0), p.p(twice[0], twice[1]), rule(), 4.0)?);
            m.push(kit.tex(stage, "A \\cdot A", 54.0, ink, &[], v(DEMO_X[0], -1.95))?);
            m.push(kit.text(stage, TWICE, 24.0, Palette::dim(), v(DEMO_X[0], -2.75))?);
            pieces.push(m);
            // A / 2: the arrow that comes out, halved
            let p = demo(DEMO_X[1]);
            let mut m = p.axes(stage, 1.4, 1.4);
            m.push(arrow(stage, p.p(0.0, 0.0), p.p(out[0], out[1]), turning(), 4.0)?);
            m.push(arrow(
                stage,
                p.p(0.0, 0.0),
                p.p(out[0] / 2.0, out[1] / 2.0),
                rule(),
                5.0,
            )?);
            m.push(kit.tex(stage, "\\frac{A}{2}", 54.0, ink, &[], v(DEMO_X[1], -1.95))?);
            m.push(kit.text(stage, HALVED, 24.0, Palette::dim(), v(DEMO_X[1], -2.75))?);
            pieces.push(m);
            // A + B: the two arrows that come out, added
            let p = demo(DEMO_X[2]);
            let sum = [out[0] + b_out[0], out[1] + b_out[1]];
            let mut m = p.axes(stage, 1.4, 1.4);
            m.push(arrow(stage, p.p(0.0, 0.0), p.p(out[0], out[1]), turning(), 4.0)?);
            m.push(arrow(stage, p.p(0.0, 0.0), p.p(b_out[0], b_out[1]), second(), 4.0)?);
            m.push(arrow(stage, p.p(0.0, 0.0), p.p(sum[0], sum[1]), result(), 5.0)?);
            m.push(kit.tex(stage, "A + B", 54.0, ink, &[], v(DEMO_X[2], -1.95))?);
            m.push(kit.text(stage, ADDED, 24.0, Palette::dim(), v(DEMO_X[2], -2.75))?);
            pieces.push(m);
        }
        let square = kit.text(stage, SQUARE, 28.0, Palette::dim(), v(0.0, -3.4))?;
        fade_in_all(stage, &[series_x], 0.6)?;
        line.wait_for(stage, "Multiply it by itself")?;
        let mut first = vec![tags[0]];
        first.extend(&pieces[0]);
        fade_in_all(stage, &first, 0.7)?;
        line.wait_for(stage, "divide by whole numbers")?;
        let mut second_in = vec![tags[1]];
        second_in.extend(&pieces[1]);
        fade_in_all(stage, &second_in, 0.7)?;
        line.wait_for(stage, "and add")?;
        let mut third = vec![tags[2]];
        third.extend(&pieces[2]);
        fade_in_all(stage, &third, 0.7)?;
        line.wait_for(stage, "A square matrix")?;
        fade_in_all(stage, &[square], 0.5)?;
        kit.hold(stage)?;

        // 07_2: put a matrix where the x was: the definition
        let line = say(&kit, stage, "07_2")?;
        let mut leaving: Vec<Mob> = pieces.iter().flatten().copied().collect();
        leaving.extend(tags);
        leaving.push(square);
        fade_out_all(stage, &leaving, 0.5)?;
        // the matrix that changes nothing, as the fact prints it
        let ones = entries("F-H.zero.true", None)?;
        let lhs = kit.tex(stage, "e^{A} =", 80.0, result(), &[], v(-5.1, 0.2))?;
        let one_grid = MatrixGrid::build(&kit, stage, &ones, 70.0, ink, v(-2.6, 0.2))?;
        let letter_i = kit.tex(stage, "I", 90.0, ink, &[], v(-2.6, 0.2))?;
        let rest = kit.tex(
            stage,
            "+ A + \\frac{A^{2}}{2!} + \\frac{A^{3}}{3!} + \\cdots",
            70.0,
            rule(),
            &[],
            v(2.7, 0.2),
        )?;
        let called = kit.text(stage, EXPONENTIAL, 34.0, result(), v(-3.4, -1.5))?;
        line.wait_for(stage, "put a matrix where the x was")?;
        fade_in_all(stage, &[lhs, one_grid.group, rest], 0.8)?;
        // the leading one is the grid that changes nothing; it is called I
        play!(stage;
            fade_out(stage.arena_mut(), one_grid.group, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), letter_i, ORIGIN, 1.0)?.rt(0.5),
        );
        line.wait_for(stage, "also called its exponential")?;
        fade_in_all(stage, &[called], 0.5)?;
        let laguerre = lower_third(
            &kit,
            stage,
            WHO_AND_WHEN,
            &format!("Laguerre, {}", fact("F-B.laguerre.year")?),
        )?;
        fade_in_all(stage, &[laguerre], 0.5)?;
        let laguerre_up = now(stage);
        kit.hold(stage)?;

        // 07_3: your turn: J squared
        let line = say(&kit, stage, "07_3")?;
        // the band came with 07_2's last words: on the finals' read the next
        // line began 2.3 s later
        hold_band(stage, laguerre_up)?;
        fade_out_all(stage, &[series_x, lhs, letter_i, rest, called, laguerre], 0.5)?;
        let card = QuizCard::build(&kit, stage, "07_3", CARD_AT, CARD_SIZE)?;
        let (centre, _, _) = card.content();
        let j_squared = kit.tex(
            stage,
            "J^{2} =",
            100.0,
            ink,
            &[],
            v(centre[0] + J_SQUARED_DX, centre[1]),
        )?;
        let ask = kit.text(stage, "?", 120.0, result(), v(centre[0] + ASK_DX, centre[1]))?;
        let fan_plane = Plane {
            centre: [centre[0] + 3.0, centre[1], 0.0],
            unit: 1.0,
        };
        let mut fan = Vec::new();
        for (from, to) in fan_ends(fan_plane.p(0.0, 0.0), 1.2, 0.0) {
            fan.push(mk_line(stage, arrow_points(from, to, 0.2), ink, 3.5, 1.0));
        }
        let mut shown = vec![j_squared, ask];
        shown.extend(&fan);
        card.show_frame(stage, &shown, 0.6)?;
        line.wait_last(stage)?;
        card.arm(stage)?;
        card.count(&kit, stage)?;

        // 07_4: a half turn: minus the matrix that changes nothing, so J
        // behaves exactly like i; the powers go round a dial beside i
        let line = say(&kit, stage, "07_4")?;
        let answer_rows = entries("F-J.power.k2", None)?;
        let answer = MatrixGrid::build(
            &kit,
            stage,
            &answer_rows,
            ANSWER_PT,
            result(),
            v(centre[0] + ANSWER_DX, centre[1]),
        )?;
        play!(stage;
            fade_out(stage.arena_mut(), ask, ORIGIN, 1.0)?.rt(0.2),
            fade_in(stage.arena_mut(), answer.group, ORIGIN, 1.0)?.rt(0.5),
        );
        card.land(stage, &[], 0.25)?;
        // the answer stays fully up until "A half turn." has been said (the
        // finished read puts that at 1.1 s; the card went at 0.75 s)
        line.wait_first_sentence(stage)?;
        let mut leaving = card.mobs();
        leaving.extend(&fan);
        leaving.extend([j_squared, answer.group]);
        fade_out_all(stage, &leaving, 0.5)?;
        // the fan, then the fan turned twice
        let left = Plane {
            centre: FAN_AT,
            unit: 1.0,
        };
        let turn = stage.add_value_tracker(0.0);
        let ink_fan: Vec<Mob> = fan_ends(left.p(0.0, 0.0), FAN_REACH, 0.0)
            .into_iter()
            .map(|(a, b)| mk_line(stage, arrow_points(a, b, 0.22), ink, 3.5, 1.0))
            .collect();
        let blue_fan: Vec<Mob> = fan_ends(left.p(0.0, 0.0), FAN_REACH, 0.0)
            .into_iter()
            .map(|(a, b)| mk_line(stage, arrow_points(a, b, 0.22), turning(), 4.5, 1.0))
            .collect();
        let mover = stage.arena_mut().add(VMobject::new());
        let (blues, base) = (blue_fan.clone(), left.p(0.0, 0.0));
        let mut seen = f64::NAN;
        stage.add_updater(
            mover,
            move |s, _| {
                let a = s.tracker_value(turn).unwrap_or(0.0);
                if a.to_bits() == seen.to_bits() {
                    return;
                }
                for (&m, (from, to)) in blues.iter().zip(fan_ends(base, FAN_REACH, a)) {
                    let _ = s.set_points(m, &quad_run(&arrow_points(from, to, 0.22)));
                }
                seen = a;
            },
            true,
        )?;
        stage.add_to_scene(mover)?;
        // J squared is minus the identity, in the fact's own numbers
        let (square_words, square_grid, minus_i) = square_row(&kit, stage, &answer_rows)?;
        // the dial of powers beside i
        let dial_arc: Vec<Vec3> = (0..=64)
            .map(|k| {
                let a = TAU * f64::from(k) / 64.0;
                v(DIAL_AT[0] + DIAL_R * a.cos(), DIAL_AT[1] + DIAL_R * a.sin())
            })
            .collect();
        let dial_ring = mk_line(stage, dial_arc, Palette::faint(), 3.0, 1.0);
        let powers = ["J", "-I", "-J", "I"];
        let numbers_on = ["i", "-1", "-i", "1"];
        let mut labels = Vec::new();
        for k in 0..4 {
            // power k + 1 sits a quarter turn further round, from the top
            let a = FRAC_PI_2 * (k as f64 + 1.0);
            let (c, s) = (a.cos(), a.sin());
            labels.push(kit.tex(
                stage,
                powers[k],
                56.0,
                rule(),
                &[],
                v(DIAL_AT[0] + 0.62 * DIAL_R * c, DIAL_AT[1] + 0.62 * DIAL_R * s),
            )?);
            labels.push(kit.tex(
                stage,
                numbers_on[k],
                56.0,
                turning(),
                &[],
                v(DIAL_AT[0] + 1.42 * DIAL_R * c, DIAL_AT[1] + 1.42 * DIAL_R * s),
            )?);
        }
        let behaves = kit.text(stage, BEHAVES, 34.0, ink, v(DIAL_AT[0], -3.05))?;
        let mut ins = ink_fan.clone();
        ins.extend(&blue_fan);
        ins.extend([square_words, square_grid.group, minus_i, dial_ring]);
        fade_in_all(stage, &ins, 0.7)?;
        line.wait_for(stage, "Every arrow reversed")?;
        play!(stage; tween(turn, FRAC_PI_2, 0.9)?);
        play!(stage; tween(turn, PI, 0.9)?);
        line.wait_for(stage, "minus the matrix that changes nothing")?;
        fade_in_all(stage, &labels, 0.7)?;
        line.wait_for(stage, "So J behaves exactly like i")?;
        fade_in_all(stage, &[behaves], 0.5)?;
        kit.hold(stage)?;

        // 07_5: run J for a time t: the terms alternate, along and at right
        // angles, and spiral in to a point on the circle
        let line = say(&kit, stage, "07_5")?;
        let mut leaving = ink_fan.clone();
        leaving.extend(&blue_fan);
        leaving.extend(&labels);
        leaving.extend([square_words, square_grid.group, minus_i, dial_ring, behaves]);
        fade_out_all(stage, &leaving, 0.5)?;
        let circle: Vec<Vec3> = (0..=96)
            .map(|k| {
                let a = TAU * f64::from(k) / 96.0;
                SPIRAL.p(a.cos(), a.sin())
            })
            .collect();
        let unit_circle = mk_line(stage, circle, Palette::faint(), 3.0, 1.0);
        let spiral_axes = SPIRAL.axes(stage, 1.25, 1.25);
        let corners = spiral_corners(1.0, SPIRAL_TERMS);
        let reach = stage.add_value_tracker(0.0);
        let spiral = mk_line(stage, spiral_polyline(&corners, 0.0), rule(), 5.0, 1.0);
        let joint_template: Vec<Vec3> = Dot::new().radius(0.07).build().points().to_vec();
        let end_dot = stage.arena_mut().add(
            Dot::new()
                .point(SPIRAL.p(0.0, 0.0))
                .radius(0.1)
                .style(flat(result(), 1.0))
                .build(),
        );
        let drawer = stage.arena_mut().add(VMobject::new());
        let cs = corners.clone();
        let mut drawn = f64::NAN;
        stage.add_updater(
            drawer,
            move |s, _| {
                let r = s.tracker_value(reach).unwrap_or(0.0);
                if r.to_bits() == drawn.to_bits() {
                    return;
                }
                let pts = spiral_polyline(&cs, r);
                let tip = pts[pts.len() - 1];
                let _ = s.set_points(spiral, &quad_run(&pts));
                let moved: Vec<Vec3> = joint_template
                    .iter()
                    .map(|d| [d[0] + tip[0], d[1] + tip[1], d[2]])
                    .collect();
                let _ = s.set_points(end_dot, &moved);
                drawn = r;
            },
            true,
        )?;
        stage.add_to_scene(drawer)?;
        // what each of the first terms is
        let mut names = Vec::new();
        for (w, p) in TERM_NAMES.iter().zip(TERM_SPOTS) {
            names.push(kit.tex(stage, w, 50.0, ink, &[], SPIRAL.p(p[0], p[1]))?);
        }
        let along = kit.text(stage, ALONG, 30.0, ink, v(2.9, 1.3))?;
        let right_angles = kit.text(stage, AT_RIGHT_ANGLES, 30.0, rule(), v(2.9, 0.6))?;
        let mut ins = spiral_axes.clone();
        ins.extend([unit_circle, spiral, end_dot]);
        fade_in_all(stage, &ins, 0.6)?;
        line.wait_for(stage, "The terms alternate")?;
        fade_in_all(stage, &[names[0], along], 0.4)?;
        play!(stage; tween_linear(reach, 1.0, 0.8)?);
        fade_in_all(stage, &[names[1], right_angles], 0.4)?;
        play!(stage; tween_linear(reach, 3.0, 1.4)?);
        fade_in_all(stage, &[names[2], names[3]], 0.4)?;
        line.wait_for(stage, "and they spiral in")?;
        play!(stage; tween_linear(reach, (corners.len() - 1) as f64, 2.8)?);
        kit.hold(stage)?;

        // 07_6: across is the cosine, up the sine; e to the tJ is the
        // rotation by t radians; at t = 1, the matrix from the start
        let line = say(&kit, stage, "07_6")?;
        let end = corners[corners.len() - 1];
        let across = mk_line(stage, vec![SPIRAL.p(0.0, 0.0), SPIRAL.p(end[0], 0.0)], ink, 8.0, 1.0);
        let up = mk_line(
            stage,
            vec![SPIRAL.p(0.0, 0.0), SPIRAL.p(0.0, end[1])],
            turning(),
            8.0,
            1.0,
        );
        let guide_x = mk_line(
            stage,
            vec![SPIRAL.p(end[0], 0.0), SPIRAL.p(end[0], end[1])],
            Palette::dim(),
            2.5,
            0.8,
        );
        let guide_y = mk_line(
            stage,
            vec![SPIRAL.p(0.0, end[1]), SPIRAL.p(end[0], end[1])],
            Palette::dim(),
            2.5,
            0.8,
        );
        let across_name = kit.text(stage, ACROSS, 28.0, ink, SPIRAL.p(end[0] / 2.0, -0.18))?;
        let up_name = kit.text_left(
            stage,
            UP,
            28.0,
            turning(),
            v(SPIRAL.p(0.0, end[1])[0] - 2.3, SPIRAL.p(0.0, end[1])[1] + 0.35),
        )?;
        fade_out_all(
            stage,
            &[names[0], names[1], names[2], names[3], along, right_angles],
            0.4,
        )?;
        line.wait_for(stage, "Its across part")?;
        fade_in_all(stage, &[across, guide_x, across_name], 0.6)?;
        line.wait_for(stage, "its up part")?;
        fade_in_all(stage, &[up, guide_y, up_name], 0.6)?;
        // e^{tJ}, live, with the plane of dots turning as t runs to one
        let t = stage.add_value_tracker(0.0);
        let cos_sin = |i: usize, k: usize| -> Box<dyn Fn(&Arena) -> f64> {
            Box::new(move |s: &Arena| {
                let t = s.tracker_value(t).unwrap_or(0.0);
                match (i, k) {
                    (0, 1) => -t.sin(),
                    (1, 0) => t.sin(),
                    _ => t.cos(),
                }
            })
        };
        let dots_plane = Plane {
            centre: [3.9, -1.0, 0.0],
            unit: 1.0,
        };
        let flow = Flow::build(
            stage,
            FlowSpec {
                plane: dots_plane,
                starts: lattice(0.5, 1.7),
                dot_radius: 0.05,
                dot_color: ink,
                dot_level: 0.9,
                velocity: None,
                trails: Vec::new(),
                trail_color: result(),
                trail_width: 4.0,
            },
        )?;
        flow.drive(stage, t, rotation_map)?;
        line.wait_for(stage, "So e to the t J")?;
        let name = kit.tex(stage, "e^{tJ} =", 76.0, result(), &[], v(1.7, 2.3))?;
        let live = LiveMatrix::build(&kit, stage, v(4.9, 2.3), 2, 2, (1.5, 0.78), 46.0, 3, result(), cos_sin)?;
        let rotation = kit.text(stage, "the rotation by t radians", 28.0, Palette::dim(), v(3.9, 0.95))?;
        let mut ins = vec![name, flow.group, rotation, live.brackets];
        ins.extend(&live.cells);
        fade_in_all(stage, &ins, 0.7)?;
        play!(stage; tween_linear(t, 1.0, 3.0)?);
        line.wait_for(stage, "At t equals one")?;
        music_hold(&kit, stage)?;

        kit.hold(stage)?;
        flow.stop(stage);
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use fmn::scene::{NullSceneSink, RuntimeConfig};

    use super::super::draw::{matrix, numbers, show, testing};
    use super::*;
    use crate::model::{Mat, expm_t, factorial};

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

    /// 07_3 and 07_4 (the dense read of the scored draft): the answer's grid
    /// lands where the "?" stood, its left bracket clear of the "=" of
    /// "J² =" (at the old offset the bracket stood on the "="), and its right
    /// edge left of the fan on the card.
    #[test]
    fn the_answer_lands_clear_of_the_equals_sign_and_the_fan() {
        run(|stage| {
            let kit = Kit::new()?;
            let card = QuizCard::build(&kit, stage, "07_3", CARD_AT, CARD_SIZE)?;
            let (centre, _, _) = card.content();
            let lhs = kit.tex(
                stage,
                "J^{2} =",
                100.0,
                Palette::ink(),
                &[],
                v(centre[0] + J_SQUARED_DX, centre[1]),
            )?;
            let ask = kit.text(stage, "?", 120.0, result(), v(centre[0] + ASK_DX, centre[1]))?;
            let rows = entries("F-J.power.k2", None)?;
            let grid = MatrixGrid::build(
                &kit,
                stage,
                &rows,
                ANSWER_PT,
                result(),
                v(centre[0] + ANSWER_DX, centre[1]),
            )?;
            // the fan's arrows begin at 1.2 left of its centre, 3.0 right of the card's
            let fan_left = centre[0] + 3.0 - 1.2;
            let eq_right = stage.get_right(lhs)[0];
            let (grid_left, grid_right) = (stage.get_left(grid.group)[0], stage.get_right(grid.group)[0]);
            assert!(
                grid_left > eq_right + 0.2,
                "the grid's left edge {grid_left:.2} against the equals sign's right {eq_right:.2}"
            );
            assert!(
                grid_right < fan_left - 0.2,
                "the grid's right edge {grid_right:.2} against the fan's left {fan_left:.2}"
            );
            // it stands where the question mark did
            let ask_mid = (stage.get_left(ask)[0] + stage.get_right(ask)[0]) / 2.0;
            let grid_mid = (grid_left + grid_right) / 2.0;
            assert!((grid_mid - ask_mid).abs() < 0.6, "{grid_mid:.2} against {ask_mid:.2}");
            Ok(())
        });
    }

    /// 07_5 (the pre-final check): the name of the third term stands clear of
    /// the spiral's top segment (the segment ran through its denominator),
    /// and inside the frame with the room a text keeps.
    #[test]
    fn the_third_terms_name_stands_over_the_spirals_top_segment() {
        use fmn::core::constants::FRAME_Y_RADIUS;
        run(|stage| {
            let kit = Kit::new()?;
            let p = TERM_SPOTS[2];
            let name = kit.tex(stage, TERM_NAMES[2], 50.0, Palette::ink(), &[], SPIRAL.p(p[0], p[1]))?;
            // the spiral's corner at t = 1 is (1, 1): its top segment lies at y = 1
            let segment_y = SPIRAL.p(0.5, 1.0)[1];
            let foot = stage.get_bottom(name)[1];
            assert!(
                foot - segment_y >= 0.15,
                "the name's foot {foot:.2}, the segment {segment_y:.2}"
            );
            assert!(FRAME_Y_RADIUS - stage.get_top(name)[1] >= 0.25);
            // and not on the next term's name (t^3/6 stands left of the downstroke)
            let q = TERM_SPOTS[3];
            let next = kit.tex(stage, TERM_NAMES[3], 50.0, Palette::ink(), &[], SPIRAL.p(q[0], q[1]))?;
            let gap = stage.get_bottom(name)[1] - stage.get_top(next)[1];
            let sideways = stage.get_left(name)[0] - stage.get_right(next)[0];
            assert!(
                gap > 0.0 || sideways > 0.1,
                "the two names overlap: gap {gap:.2}, side {sideways:.2}"
            );
            Ok(())
        });
    }

    /// 07_4 (the dense read of the scored draft): the row "J² = [-1 0; 0 -1]
    /// = -I" has room under the frame's top edge (the grid's bracket stood
    /// 0.16 from it) and over the fan and the dial under it, and its three
    /// parts keep a gap between them.
    #[test]
    fn the_row_of_j_squared_has_room_above_and_below() {
        use fmn::core::constants::FRAME_Y_RADIUS;
        run(|stage| {
            let kit = Kit::new()?;
            let rows = entries("F-J.power.k2", None)?;
            let (words, grid, minus_i) = square_row(&kit, stage, &rows)?;
            let parts = [words, grid.group, minus_i];
            let top = parts.iter().map(|&m| stage.get_top(m)[1]).fold(f64::MIN, f64::max);
            let foot = parts.iter().map(|&m| stage.get_bottom(m)[1]).fold(f64::MAX, f64::min);
            assert!(
                top < FRAME_Y_RADIUS - 0.3,
                "the row's top {top:.2} against the frame's edge {FRAME_Y_RADIUS:.2}"
            );
            let (fan_top, dial_top) = (FAN_AT[1] + FAN_REACH, DIAL_AT[1] + DIAL_R);
            assert!(
                foot > fan_top + 0.2,
                "the row's foot {foot:.2} against the fan's top {fan_top:.2}"
            );
            assert!(
                foot > dial_top + 0.3,
                "the row's foot {foot:.2} against the dial's top {dial_top:.2}"
            );
            for pair in parts.windows(2) {
                let (r, l) = (stage.get_right(pair[0])[0], stage.get_left(pair[1])[0]);
                assert!(l > r + 0.1, "parts of the row {r:.2} and {l:.2} touch");
            }
            Ok(())
        });
    }

    /// 07_3 and 07_4: J times J is minus the matrix that changes nothing,
    /// the powers have the period the fact says and go round with i.
    #[test]
    fn j_squared_is_minus_the_identity_and_its_powers_go_round_with_i() {
        let j = quarter_turn();
        let minus_i = Mat::identity(2).scale(-1.0);
        assert_eq!(j.mul(&j), minus_i);
        assert_eq!(matrix("F-J.power.k2").unwrap(), vec![vec![-1.0, 0.0], vec![0.0, -1.0]]);
        assert_eq!(fact("F-Q.07_3.answer").unwrap(), "-1,0;0,-1");
        assert_eq!(fact("F-J.period").unwrap(), "4");
        // the dial's four stops are J, -I, -J, I; beside i, -1, -i, 1
        let stops = [j.clone(), minus_i.clone(), j.scale(-1.0), Mat::identity(2)];
        for (k, m) in stops.iter().enumerate() {
            assert_eq!(&j.pow(k as u32 + 1), m, "power {}", k + 1);
        }
        let mut z = (1.0_f64, 0.0_f64);
        for want in [(0.0, 1.0), (-1.0, 0.0), (0.0, -1.0), (1.0, 0.0)] {
            z = (-z.1, z.0);
            assert_eq!(z, want);
        }
    }

    /// 07_5: the spiral's corners are the partial sums of the series, they
    /// start at the origin, the first term is one along the arrow, the next
    /// at right angles, and they close on the point of 06_4's circle at t.
    #[test]
    fn the_terms_spiral_in_to_a_point_on_the_circle() {
        let corners = spiral_corners(1.0, SPIRAL_TERMS);
        assert_eq!(corners.len(), SPIRAL_TERMS + 2);
        assert_eq!(corners[0], [0.0, 0.0]);
        assert_eq!(corners[1], [1.0, 0.0], "one along the arrow");
        assert_eq!(corners[2], [1.0, 1.0], "t, at right angles");
        assert!(
            (corners[3][0] - 0.5).abs() < 1e-12 && (corners[3][1] - 1.0).abs() < 1e-12,
            "t^2 / 2, back"
        );
        assert!((corners[4][1] - (1.0 - 1.0 / 6.0)).abs() < 1e-12, "t^3 / 6, down");
        let end = corners[corners.len() - 1];
        assert!((end[0] - 1.0_f64.cos()).abs() < 1e-9 && (end[1] - 1.0_f64.sin()).abs() < 1e-9);
        assert!((end[0].hypot(end[1]) - 1.0).abs() < 1e-9, "on the circle");
        // the terms alternate: even terms move across, odd terms move up
        for k in 1..corners.len() - 1 {
            let (dx, dy) = (corners[k + 1][0] - corners[k][0], corners[k + 1][1] - corners[k][1]);
            if k % 2 == 1 {
                assert!(dy.abs() > 0.0 && dx.abs() < 1e-12, "term {k}");
            } else {
                assert!(dx.abs() > 0.0 && dy.abs() < 1e-12, "term {k}");
            }
        }
        // the move from corner k to corner k + 1 is term k, t^k / k! long
        let size_of = |k: usize| (corners[k + 1][0] - corners[k][0]).hypot(corners[k + 1][1] - corners[k][1]);
        for k in 0..=7_usize {
            assert!(
                (size_of(k) - 1.0 / factorial(k as u32) as f64).abs() < 1e-12,
                "term {k}"
            );
        }
    }

    /// 07_6: the across moves add to the cosine and the up moves to the
    /// sine (the facts' split), and e to the tJ at t = 1 reads the numbers
    /// of the first column the facts print.
    #[test]
    fn across_is_the_cosine_and_up_is_the_sine() {
        let end = spiral_corners(1.0, 30);
        let end = end[end.len() - 1];
        assert_eq!(show(end[0], 6).unwrap(), fact("F-J.split.even").unwrap());
        assert_eq!(show(end[1], 6).unwrap(), fact("F-J.split.odd").unwrap());
        // the live matrix of the picture at t = 1 is the fact's
        let at_one = entries("F-V.07_6.at_one", Some(3)).unwrap();
        assert_eq!(at_one, vec![vec!["0.540", "-0.841"], vec!["0.841", "0.540"]]);
        let m = expm_t(&quarter_turn(), 1.0);
        for (i, row) in at_one.iter().enumerate() {
            for (k, shown) in row.iter().enumerate() {
                assert_eq!(&show(m.get(i, k), 3).unwrap(), shown);
            }
        }
        assert_eq!(numbers("F-N.turn.t1").unwrap().len(), 2);
    }

    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("07_a_matrix_in").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            "e^{A} =",
            "J^{2} =",
            "-I",
            "e^{tJ} =",
            "Laguerre, 1867",
            "pause for more time",
            "the rotation by t radians",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never typesets {want:?}: {words:?}"
            );
        }
        assert!(ran.seconds > 40.0 && ran.seconds < 85.0, "{} s", ran.seconds);
    }
}
