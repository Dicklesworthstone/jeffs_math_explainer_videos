//! Chapter 01, e to a matrix: the question, a grid of four numbers in an
//! exponent over a plane of faint dots that stirs (01_1); the answer, a
//! rotation by one radian, with the central picture already running: every
//! dot carries a small teal arrow at right angles to its place and goes a
//! gold arc for one unit of time (01_2); the title, with its listen window
//! (01_3); what a matrix is, a machine worked row by row on one arrow (01_4);
//! and the fan of arrows turned a quarter turn, which is called J (01_5).
//!
//! Every number is a fact: the grid in the exponent and the machine are
//! `F-J.power.k1`, the arrow and what comes out are `F-M.machine.*`, the
//! answer grid is `F-H.J.true` to the three places it is shown, the value of
//! e under the e is `F-V.02_1.e` and the arc's label `F-V.01_2.degrees.shown`.

use std::f64::consts::{FRAC_PI_2, TAU};

use explainer_kit::animatic::wrap;
use explainer_kit::{Kit, Palette, Timed, fade_all, play, tween, v};
use fmn::prelude::*;

use super::draw::{
    Flow, FlowSpec, MatrixGrid, Plane, Velocity, apply, arrow, arrow_points, chip, corners, entries, fact, fade_in_all,
    fade_out_all, faded_line, lattice, map2, matrix, music_hold, nearest, numbers, predict_hold, result, rotation_map,
    rule, run, say, stir, turning, whole,
};
use super::glyphs;
use crate::model::quarter_turn;

pub struct EToAMatrix {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const J_NAME: &str = "J: the quarter turn";

/// The dots: a disc of this radius, this far apart (about 120 of them).
const DISC: f64 = 3.3;
const SPACING: f64 = 0.55;
/// Where the question goes when the plane takes the stage, and how small.
const CORNER_AT: Vec3 = [-4.8, 3.0, 0.0];
const CORNER_WIDTH: f64 = 4.2;
const CORNER_SCALE: f64 = 0.33;
/// The longest an arrow of the fan reaches (plane units).
const FAN_R: f64 = 2.3;
/// Arrows in the fan.
const FAN_N: usize = 8;

/// Row `row` of a grid applied to `arrow` as it is worked on screen,
/// `0 \times 2 - 1 \times 1 = -1`, checked against what the machine hands back.
fn working(row: &[f64], arrow: &[f64], out: f64) -> fmn::Result<String> {
    let mut s = String::new();
    let mut total = 0.0;
    for (k, (&a, &x)) in row.iter().zip(arrow).enumerate() {
        let term = format!("{} \\times {}", whole(a.abs()), whole(x));
        match (k, a < 0.0) {
            (0, true) => s.push('-'),
            (0, false) => {}
            (_, true) => s.push_str(" - "),
            (_, false) => s.push_str(" + "),
        }
        s.push_str(&term);
        total += a * x;
    }
    if (total - out).abs() > 1e-9 {
        return Err(super::draw::fail(format!(
            "the row {row:?} on {arrow:?} is {total}, not the {out} the fact says"
        )));
    }
    s.push_str(&format!(" = {}", whole(total)));
    Ok(s)
}

/// A column of numbers as grid rows.
fn column(xs: &[f64]) -> Vec<Vec<String>> {
    xs.iter().map(|&x| vec![whole(x)]).collect()
}

impl SceneConstruct for EToAMatrix {
    fn name(&self) -> &str {
        "01_e_to_a_matrix"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let j_rows = entries("F-J.power.k1", None)?;
        let j_grid = matrix("F-J.power.k1")?;
        let input = numbers("F-M.machine.in")?;
        let output = numbers("F-M.machine.out")?;

        // the plane of dots behind everything, faint and stirring: one
        // updater places every dot (and its arrow and trail) at the turn by
        // `t` radians
        let plane = Plane {
            centre: v(0.0, 0.0),
            unit: 1.0,
        };
        let starts = lattice(SPACING, DISC);
        let mut trailing = Vec::new();
        for r in [0.33, 0.66, 1.0] {
            for k in 0..3 {
                let a = TAU * f64::from(k) / 3.0;
                trailing.push(nearest(&starts, [DISC * r * a.cos(), DISC * r * a.sin()]));
            }
        }
        let flow = Flow::build(
            stage,
            FlowSpec {
                plane,
                starts,
                dot_radius: 0.045,
                dot_color: Palette::ink(),
                dot_level: 0.22,
                velocity: Some(Velocity {
                    rule: map2(&quarter_turn()),
                    scale: 0.26,
                    color: rule(),
                    width: 3.0,
                    head: 0.13,
                }),
                trails: trailing,
                trail_color: result(),
                trail_width: 4.0,
            },
        )?;
        let t = stage.add_value_tracker(0.0);
        flow.drive(stage, t, rotation_map)?;
        let stirring = stage.add_value_tracker(1.0);
        stir(stage, t, stirring, 0.16, 1.3)?;
        stage.add_to_scene(flow.group)?;

        // 01_1: e, with a grid of four numbers where an exponent should be
        let line = say(&kit, stage, "01_1")?;
        let e = kit.tex(stage, "e", 230.0, Palette::ink(), &[], v(-3.3, -0.35))?;
        let e_value = format!("e = {}\u{2026}", fact("F-V.02_1.e")?);
        let e_label = kit.text(stage, &e_value, 30.0, Palette::dim(), v(-3.3, -1.75))?;
        let exponent = MatrixGrid::build(&kit, stage, &j_rows, 60.0, rule(), v(-1.7, 1.15))?;
        let equals = kit.tex(stage, "=", 150.0, Palette::ink(), &[], v(0.1, -0.35))?;
        let question = kit.text(stage, "?", 150.0, result(), v(1.55, -0.35))?;
        fade_in_all(stage, &[e, e_label], 0.6)?;
        line.wait_for(stage, "raised to the power")?;
        fade_in_all(stage, &[exponent.group], 0.7)?;
        line.wait_for(stage, "What could that possibly equal")?;
        fade_in_all(stage, &[equals, question], 0.5)?;
        predict_hold(&kit, stage)?;

        // 01_2: the answer, and the plane takes the stage
        let line = say(&kit, stage, "01_2")?;
        stage.set_tracker_value(stirring, 0.0)?;
        let answer_rows = entries("F-H.J.true", Some(3))?;
        let answer = MatrixGrid::build(&kit, stage, &answer_rows, 62.0, result(), v(3.3, -0.35))?;
        play!(stage;
            tween(t, 0.0, 0.5)?,
            fade_out(stage.arena_mut(), question, ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), answer.group, ORIGIN, 1.0)?.rt(0.7),
        );
        line.wait_for(stage, "of the whole plane")?;
        // the whole question, rigidly, to the corner
        let (left, right) = (stage.get_left(e)[0], stage.get_right(answer.group)[0]);
        let bottom = stage.get_bottom(e)[1].min(stage.get_bottom(answer.group)[1]);
        let top = stage.get_top(exponent.group)[1].max(stage.get_top(answer.group)[1]);
        let centre = v((left + right) / 2.0, (bottom + top) / 2.0);
        let factor = CORNER_SCALE.min(CORNER_WIDTH / (right - left));
        let shift = [CORNER_AT[0] - centre[0], CORNER_AT[1] - centre[1], 0.0];
        let mut moves: Vec<Box<dyn Animation>> = Vec::new();
        for m in [e, exponent.group, equals, answer.group] {
            let a = m
                .animate()
                .set_anim_args(run(1.2))?
                .scale_about(factor, Some(centre), None)?
                .shift(shift)?;
            moves.push(stage.prepare(a)?);
        }
        let a = fade_out(stage.arena_mut(), e_label, ORIGIN, 1.0)?;
        moves.push(stage.prepare(a.rt(0.5))?);
        moves.push(stage.prepare(tween(flow.levels[0], 0.85, 1.0)?)?);
        moves.push(stage.prepare(tween(flow.levels[1], 1.0, 1.0)?)?);
        stage.play_prepared(moves)?;
        line.wait_for(stage, "by exactly one radian")?;
        // one unit of time: every dot goes round its arc; a few leave a gold
        // trail, and one arc says how long a radian is
        let words = format!("1 radian = {}\u{b0}", fact("F-V.01_2.degrees.shown")?);
        let arc = chip(&kit, stage, &words, 30.0, result(), plane.p(3.1, 2.0))?;
        play!(stage;
            tween(flow.levels[2], 1.0, 0.4)?,
            tween(t, 1.0, 2.4)?,
        );
        fade_in_all(stage, &[arc], 0.5)?;

        // 01_3: the title, with its listen window
        kit.hold(stage)?;
        flow.stop(stage);
        fade_all(stage, 0.7)?;
        let mut title = Vec::new();
        for (i, l) in wrap(glyphs::TITLE, 22, 2).iter().enumerate() {
            title.push(kit.text(stage, l, 66.0, Palette::ink(), v(0.0, 0.55 - 1.0 * i as f64))?);
        }
        fade_in_all(stage, &title, 0.8)?;
        kit.listen(stage, 3.0, &format!("{}:01_3", kit.chapter))?;

        // 01_4: a matrix is a machine: an arrow goes in, each row mixes its
        // two numbers into one, an arrow comes out
        let line = say(&kit, stage, "01_4")?;
        fade_out_all(stage, &title, 0.4)?;
        let grid = MatrixGrid::build(&kit, stage, &j_rows, 80.0, rule(), v(-4.6, 1.6))?;
        let arrow_in = MatrixGrid::build(&kit, stage, &column(&input), 80.0, Palette::ink(), v(-2.1, 1.6))?;
        let gives = kit.tex(stage, "=", 80.0, Palette::ink(), &[], v(-0.7, 1.6))?;
        let arrow_out = MatrixGrid::build(&kit, stage, &column(&output), 80.0, turning(), v(0.75, 1.6))?;
        let small = Plane {
            centre: v(4.4, 1.6),
            unit: 0.85,
        };
        let axes = small.axes(stage, 2.4, 2.4);
        let seen_in = arrow(
            stage,
            small.p(0.0, 0.0),
            small.p(input[0], input[1]),
            Palette::ink(),
            4.0,
        )?;
        let seen_out = arrow(stage, small.p(0.0, 0.0), small.p(output[0], output[1]), turning(), 4.0)?;
        let mut working_lines = Vec::new();
        for (r, row) in j_grid.iter().enumerate() {
            let src = working(row, &input, output[r])?;
            let at = v(-2.0, -0.8 - 1.1 * r as f64);
            working_lines.push(kit.tex(stage, &src, 64.0, Palette::ink(), &[], at)?);
        }
        fade_in_all(stage, &[grid.group], 0.6)?;
        line.wait_for(stage, "Feed it an arrow")?;
        let mut ins = axes.clone();
        ins.extend([arrow_in.group, seen_in]);
        fade_in_all(stage, &ins, 0.6)?;
        line.wait_for(stage, "hands back a new arrow")?;
        fade_in_all(stage, &[gives, seen_out], 0.6)?;
        line.wait_for(stage, "Each row mixes")?;
        let band0 = grid.row_band(stage, 0, rule())?;
        fade_in_all(stage, &[band0, working_lines[0]], 0.5)?;
        line.wait_for(stage, "into one new number")?;
        let band1 = grid.row_band(stage, 1, rule())?;
        play!(stage;
            fade_out(stage.arena_mut(), band0, ORIGIN, 1.0)?.rt(0.3),
            fade_in(stage.arena_mut(), band1, ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), working_lines[1], ORIGIN, 1.0)?.rt(0.5),
        );
        fade_in_all(stage, &[arrow_out.group], 0.5)?;

        // 01_5: it turns every arrow a quarter turn: the fan goes in and
        // comes out turned, in blue; the grid takes its name
        let line = say(&kit, stage, "01_5")?;
        let mut leaving = axes;
        leaving.extend([
            band1,
            working_lines[0],
            working_lines[1],
            arrow_in.group,
            gives,
            arrow_out.group,
            seen_in,
            seen_out,
        ]);
        fade_out_all(stage, &leaving, 0.5)?;
        let fan = Plane {
            centre: v(2.6, 0.0),
            unit: 1.0,
        };
        let fan_axes = fan.axes(stage, FAN_R + 0.4, FAN_R + 0.4);
        let spokes: Vec<[f64; 2]> = (0..FAN_N)
            .map(|k| {
                let a = TAU * k as f64 / FAN_N as f64;
                [FAN_R * a.cos(), FAN_R * a.sin()]
            })
            .collect();
        let origin = fan.p(0.0, 0.0);
        let mut going_in = Vec::new();
        let mut coming_out = Vec::new();
        for p in &spokes {
            let tip = fan.p(p[0], p[1]);
            let a = polyline(arrow_points(origin, tip, 0.24))
                .style(faded_line(Palette::ink(), 3.5, 1.0))
                .build();
            going_in.push(stage.arena_mut().add(a));
            let b = polyline(arrow_points(origin, tip, 0.24))
                .style(faded_line(turning(), 4.5, 1.0))
                .build();
            coming_out.push(stage.arena_mut().add(b));
        }
        let turned = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        let (outs, ends) = (coming_out.clone(), spokes.clone());
        let mut shown = f64::NAN;
        stage.add_updater(
            anchor,
            move |s, _| {
                let a = s.tracker_value(turned).unwrap_or(0.0);
                if a.to_bits() == shown.to_bits() {
                    return;
                }
                let m = rotation_map(a);
                for (&mob, p) in outs.iter().zip(&ends) {
                    let q = apply(&m, *p);
                    let _ = s.set_points(mob, &corners(&arrow_points(origin, fan.p(q[0], q[1]), 0.24)));
                }
                shown = a;
            },
            true,
        )?;
        stage.add_to_scene(anchor)?;
        let to = v(-4.6, 0.4);
        let from = stage.get_center(grid.group);
        let mut ins = fan_axes;
        ins.extend(&going_in);
        ins.extend(&coming_out);
        let slide = grid
            .group
            .animate()
            .set_anim_args(run(0.8))?
            .shift([to[0] - from[0], to[1] - from[1], 0.0])?;
        let slid = stage.prepare(slide)?;
        let mut anims: Vec<Box<dyn Animation>> = vec![slid];
        for m in ins {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.6))?);
        }
        stage.play_prepared(anims)?;
        line.wait_for(stage, "turns every arrow")?;
        play!(stage; tween(turned, FRAC_PI_2, 1.6)?);
        line.wait_for(stage, "We'll call it J")?;
        let name = kit.text_t2c(stage, J_NAME, 32.0, Palette::ink(), &[("J", rule())], v(-4.6, -1.35))?;
        fade_in_all(stage, &[name], 0.5)?;
        music_hold(&kit, stage)?;

        kit.hold(stage)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::testing;
    use super::*;

    /// The machine of 01_4, worked as the storyboard works it: the rows of
    /// J on the arrow (2, 1) give (-1, 2), and a result that is not the
    /// fact's is refused.
    #[test]
    fn the_machine_is_worked_as_the_storyboard_works_it() {
        let rows = matrix("F-J.power.k1").unwrap();
        let (input, output) = (numbers("F-M.machine.in").unwrap(), numbers("F-M.machine.out").unwrap());
        assert_eq!(
            working(&rows[0], &input, output[0]).unwrap(),
            "0 \\times 2 - 1 \\times 1 = -1"
        );
        assert_eq!(
            working(&rows[1], &input, output[1]).unwrap(),
            "1 \\times 2 + 0 \\times 1 = 2"
        );
        assert!(working(&rows[0], &input, 5.0).is_err());
        // the quarter turn is the rule of the cold open's plane: every
        // arrow it hands back is at right angles to where it was fed
        let j = quarter_turn();
        let out = j.apply(&input);
        assert_eq!(out, output);
        assert_eq!(input[0] * out[0] + input[1] * out[1], 0.0);
    }

    /// 01_5: the fan of eight arrows comes out turned a quarter turn: each
    /// spoke lands on the next-but-one.
    #[test]
    fn a_quarter_turn_takes_each_spoke_two_places_round_the_fan() {
        let spokes: Vec<[f64; 2]> = (0..FAN_N)
            .map(|k| {
                let a = TAU * k as f64 / FAN_N as f64;
                [FAN_R * a.cos(), FAN_R * a.sin()]
            })
            .collect();
        let turn = rotation_map(FRAC_PI_2);
        for (k, p) in spokes.iter().enumerate() {
            let q = apply(&turn, *p);
            let want = spokes[(k + 2) % FAN_N];
            assert!(
                (q[0] - want[0]).abs() < 1e-12 && (q[1] - want[1]).abs() < 1e-12,
                "spoke {k}"
            );
        }
    }

    /// The cold open at its end: the dots have gone one radian round, and
    /// one radian is the 57.3 degrees the label says.
    #[test]
    fn one_radian_is_the_degrees_the_arc_is_labelled_with() {
        assert_eq!(fact("F-V.01_2.degrees.shown").unwrap(), "57.3");
        assert_eq!(fact("F-V.02_1.e").unwrap(), "2.718");
        let (c, s) = (1.0_f64.cos(), 1.0_f64.sin());
        let m = rotation_map(1.0);
        assert_eq!([m[0][0], m[1][0]], [c, s]);
        // every dot's speed is its distance from the centre, so a unit of
        // time is an arc as long as the radius: the radian
        let p = [2.2, 0.0];
        let q = apply(&m, p);
        let chord = (q[0] - p[0]).hypot(q[1] - p[1]);
        let arc = 2.2 * 1.0;
        assert!(chord < arc && chord > 0.95 * arc, "chord {chord}, arc {arc}");
    }

    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("01_e_to_a_matrix").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            "e = 2.718\u{2026}",
            "1 radian = 57.3\u{b0}",
            "0 \\times 2 - 1 \\times 1 = -1",
            "1 \\times 2 + 0 \\times 1 = 2",
            "0.540",
            "-0.841",
            "J: the quarter turn",
            glyphs::TITLE.split(' ').next().unwrap(),
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never typesets {want:?}: {words:?}"
            );
        }
        // five spoken lines and a title: far under the ~85 s a chapter may run
        assert!(ran.seconds > 25.0 && ran.seconds < 85.0, "{} s", ran.seconds);
    }
}
