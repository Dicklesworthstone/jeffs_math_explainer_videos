//! Chapter 02, the obvious guess: e is a number, e squared is e times e, but
//! e times itself a matrix number of times means nothing (02_1); the obvious
//! repair, e raised to each entry, tried on J and on the matrix of four
//! zeros, which it turns into four ones (02_2); why that is wrong, since e to
//! the zero is one and its matrix twin is the grid that changes nothing
//! (02_3); what four ones do, squash the plane onto a line, and the cross
//! (02_4); and what e to the x has meant all along (02_5).
//!
//! The numbers are facts: the value of e is `F-V.02_1.e`, J is
//! `F-J.power.k1`, the guess on J is `F-H.J.guess` (its entries are the
//! little e's of the picture), the guess on zeros `F-H.zero.guess`, the
//! matrix that changes nothing `F-H.zero.true`.

use explainer_kit::props::lower_third;
use explainer_kit::{Kit, Palette, Timed, fade_all, play, tween, v};
use fmn::prelude::*;

use super::draw::{
    Flow, FlowSpec, Map2, MatrixGrid, Plane, arrow, chip, cross_mark, entries, fact, fade_in_all, fade_out_all,
    faded_line, lattice, music_hold, predict_hold, quiz, result, rule, say, warn,
};

pub struct TheObviousGuess {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const HOW_MANY: &str = "how many times?";
const THE_GUESS: &str = "the guess";
const EACH_ENTRY: &str = "e to each entry";
const CHANGES_NOTHING: &str = "the matrix that changes nothing";
const IS_THAT_RIGHT: &str = "is that right?";
const PROGRAMMERS: &str = "for programmers:";
const NP_EXP: &str = "this guess is np.exp(A); the real thing is scipy.linalg.expm(A)";

/// The dots: a disc this wide, this far apart, and where it stands.
const DISC: f64 = 2.0;
const SPACING: f64 = 0.45;
const PLANE: Plane = Plane {
    centre: [3.2, 0.0, 0.0],
    unit: 1.0,
};

/// The map that takes the plane to the plane of four ones as `s` runs from 0
/// to 1: `(1 - s) I + s [[1, 1], [1, 1]]`. At 0 nothing moves; at 1 every
/// point `(x, y)` is at `(x + y, x + y)`.
pub fn squash(s: f64) -> Map2 {
    [[1.0, s], [s, 1.0]]
}

/// A grid's entries each raised to e, as TeX (`e^{-1}`).
fn little_es(rows: &[Vec<String>]) -> Vec<Vec<String>> {
    rows.iter()
        .map(|r| r.iter().map(|x| format!("e^{{{x}}}")).collect())
        .collect()
}

impl SceneConstruct for TheObviousGuess {
    fn name(&self) -> &str {
        "02_the_obvious_guess"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let j_rows = entries("F-J.power.k1", None)?;
        let zero_rows = vec![vec!["0".to_owned(); 2]; 2];
        let ones = entries("F-H.zero.guess", None)?;
        let identity = entries("F-H.zero.true", None)?;
        let ink = Palette::ink();

        // 02_1: e is a number; e squared is e times e; e times itself a
        // matrix number of times?
        let line = say(&kit, stage, "02_1")?;
        let e_is = kit.text(
            stage,
            &format!("e = {}\u{2026}", fact("F-V.02_1.e")?),
            54.0,
            ink,
            v(-3.4, 2.7),
        )?;
        let squared = kit.tex(stage, "e^{2} = e \\cdot e", 86.0, ink, &[], v(-3.4, 1.3))?;
        let e = kit.tex(stage, "e", 150.0, ink, &[], v(-5.4, -1.0))?;
        let exponent = MatrixGrid::build(&kit, stage, &j_rows, 42.0, rule(), v(-3.9, -0.05))?;
        let times = kit.tex(stage, "= e \\cdot e \\cdots e", 86.0, ink, &[], v(0.2, -1.0))?;
        let how_many = kit.text(stage, HOW_MANY, 34.0, warn(), v(0.9, -2.05))?;
        fade_in_all(stage, &[e_is], 0.5)?;
        line.wait_for(stage, "E squared is e times e")?;
        fade_in_all(stage, &[squared], 0.6)?;
        line.wait_for(stage, "But e times itself")?;
        fade_in_all(stage, &[e, exponent.group, times], 0.7)?;
        line.wait_for(stage, "That means nothing")?;
        fade_in_all(stage, &[how_many], 0.5)?;
        kit.hold(stage)?;

        // 02_2: the obvious repair: e to each entry, tried on J and on four
        // zeros; the question holds without a ring
        let line = say(&kit, stage, "02_2")?;
        fade_out_all(stage, &[e_is, squared, e, exponent.group, times, how_many], 0.5)?;
        let label = |kit: &Kit, stage: &mut Stage<'_>, at: Vec3| kit.text(stage, EACH_ENTRY, 26.0, Palette::dim(), at);
        // row one: J, and the little e's on its entries
        let j_in = MatrixGrid::build(&kit, stage, &j_rows, 54.0, rule(), v(-4.6, 1.6))?;
        let j_arrow = arrow(stage, v(-3.0, 1.6), v(-1.1, 1.6), Palette::dim(), 3.0)?;
        let j_label = label(&kit, stage, v(-2.05, 2.15))?;
        let j_guess = MatrixGrid::build(&kit, stage, &little_es(&j_rows), 54.0, warn(), v(1.5, 1.6))?;
        let j_frame = j_guess.outline(stage, warn())?;
        // row two: four zeros, and four ones
        let z_in = MatrixGrid::build(&kit, stage, &zero_rows, 54.0, rule(), v(-4.6, -0.9))?;
        let z_arrow = arrow(stage, v(-3.0, -0.9), v(-1.1, -0.9), Palette::dim(), 3.0)?;
        let z_label = label(&kit, stage, v(-2.05, -0.35))?;
        let z_guess = MatrixGrid::build(&kit, stage, &little_es(&zero_rows), 54.0, warn(), v(1.5, -0.9))?;
        let equals = kit.tex(stage, "=", 60.0, ink, &[], v(3.55, -0.9))?;
        let z_ones = MatrixGrid::build(&kit, stage, &ones, 54.0, warn(), v(5.3, -0.9))?;
        let z_frame = z_ones.outline(stage, warn())?;
        let guess_tag = kit.text(stage, THE_GUESS, 26.0, warn(), v(5.3, -2.0))?;
        line.wait_for(stage, "Raise e to each entry")?;
        fade_in_all(stage, &[j_in.group, j_arrow, j_label], 0.5)?;
        fade_in_all(stage, &[j_guess.group, j_frame], 0.6)?;
        line.wait_for(stage, "Test it on the easiest matrix")?;
        fade_in_all(stage, &[z_in.group, z_arrow, z_label], 0.5)?;
        line.wait_for(stage, "E to the zero is one")?;
        fade_in_all(stage, &[z_guess.group], 0.5)?;
        line.wait_for(stage, "so the guess gives four ones")?;
        fade_in_all(stage, &[equals, z_ones.group, z_frame, guess_tag], 0.6)?;
        line.wait_for(stage, "Is that right")?;
        let programmers = lower_third(&kit, stage, PROGRAMMERS, NP_EXP)?;
        let question = chip(&kit, stage, IS_THAT_RIGHT, 36.0, quiz(), v(5.0, 3.0))?;
        fade_in_all(stage, &[programmers, question], 0.5)?;
        predict_hold(&kit, stage)?;

        // 02_3: no: e to the zero is one, and its matrix twin changes nothing
        let line = say(&kit, stage, "02_3")?;
        fade_out_all(
            stage,
            &[
                j_in.group,
                j_arrow,
                j_label,
                j_guess.group,
                j_frame,
                z_in.group,
                z_arrow,
                z_label,
                z_guess.group,
                equals,
                z_ones.group,
                z_frame,
                guess_tag,
                programmers,
                question,
            ],
            0.5,
        )?;
        let flow = Flow::build(
            stage,
            FlowSpec {
                plane: PLANE,
                starts: lattice(SPACING, DISC),
                dot_radius: 0.05,
                dot_color: ink,
                dot_level: 0.9,
                velocity: None,
                trails: Vec::new(),
                trail_color: result(),
                trail_width: 4.0,
            },
        )?;
        let s = stage.add_value_tracker(0.0);
        flow.drive(stage, s, squash)?;
        let one_times = kit.tex(stage, "1 \\times x = x", 90.0, ink, &[], v(-3.6, 2.5))?;
        let twin = MatrixGrid::build(&kit, stage, &identity, 80.0, result(), v(-3.6, 0.5))?;
        let twin_name = kit.text(stage, CHANGES_NOTHING, 30.0, Palette::dim(), v(-3.6, -1.3))?;
        line.wait_for(stage, "multiplying by one")?;
        fade_in_all(stage, &[one_times], 0.6)?;
        line.wait_for(stage, "Its matrix twin")?;
        fade_in_all(stage, &[twin.group, twin_name], 0.7)?;
        line.wait_for(stage, "leaves every arrow alone")?;
        fade_in_all(stage, &[flow.group], 0.6)?;

        // 02_4: four ones squash the whole plane onto a single line
        let line = say(&kit, stage, "02_4")?;
        let ones_grid = MatrixGrid::build(&kit, stage, &ones, 80.0, warn(), v(-3.6, 0.5))?;
        let ones_name = kit.text(stage, THE_GUESS, 30.0, warn(), v(-3.6, -1.3))?;
        play!(stage;
            fade_out(stage.arena_mut(), one_times, ORIGIN, 1.0)?.rt(0.4),
            fade_out(stage.arena_mut(), twin.group, ORIGIN, 1.0)?.rt(0.4),
            fade_out(stage.arena_mut(), twin_name, ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), ones_grid.group, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), ones_name, ORIGIN, 1.0)?.rt(0.5),
        );
        let diagonal = polyline(vec![PLANE.p(-3.6, -3.6), PLANE.p(3.6, 3.6)])
            .style(faded_line(warn(), 4.0, 1.0))
            .build();
        let diagonal = stage.arena_mut().add(diagonal);
        line.wait_for(stage, "squash the whole plane")?;
        play!(stage; tween(s, 1.0, 2.2)?);
        let cross = cross_mark(stage, v(-3.6, 0.5), 1.35, warn(), 9.0)?;
        play!(stage;
            fade_in(stage.arena_mut(), diagonal, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), cross, v(0.0, 0.3), 1.0)?.rt(0.5),
        );
        kit.hold(stage)?;

        // 02_5: the guess is dead; what has e to the x meant all along?
        let line = say(&kit, stage, "02_5")?;
        flow.stop(stage);
        fade_all(stage, 0.7)?;
        let e_to_x = kit.tex(stage, "e^{x}", 300.0, ink, &[], v(-0.9, 0.1))?;
        let query = kit.text(stage, "?", 200.0, result(), v(2.9, 0.1))?;
        fade_in_all(stage, &[e_to_x], 0.7)?;
        line.wait_for(stage, "What has e to the x")?;
        fade_in_all(stage, &[query], 0.5)?;
        music_hold(&kit, stage)?;

        kit.hold(stage)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::{apply, matrix, testing};
    use super::*;
    use crate::model::{Mat, quarter_turn};

    /// 02_2: "raise e to each entry" is `Mat::entrywise_exp`, and the
    /// facts that name it say the same: on J, e to 0, -1, 1, 0; on four
    /// zeros, four ones.
    #[test]
    fn the_guess_is_e_to_each_entry() {
        let want = quarter_turn().entrywise_exp();
        let guess = entries("F-H.J.guess", Some(3)).unwrap();
        for (i, row) in guess.iter().enumerate() {
            for (j, s) in row.iter().enumerate() {
                assert_eq!(
                    *s,
                    super::super::draw::show(want.get(i, j), 3).unwrap(),
                    "J guess {i},{j}"
                );
            }
        }
        assert_eq!(fact("F-H.zero.guess").unwrap(), "1,1;1,1");
        // the little e's of the picture carry exactly J's entries
        assert_eq!(
            little_es(&entries("F-J.power.k1", None).unwrap()),
            vec![vec!["e^{0}", "e^{-1}"], vec!["e^{1}", "e^{0}"]]
        );
    }

    /// 02_3: e to the zero is one, so the matrix twin of "multiplying by
    /// one" is the identity, and it leaves every arrow alone.
    #[test]
    fn the_twin_of_one_leaves_every_arrow_alone() {
        assert_eq!(fact("F-H.zero.true").unwrap(), "1,0;0,1");
        let id = squash(0.0);
        for p in lattice(SPACING, DISC) {
            assert_eq!(apply(&id, p), p);
        }
        assert_eq!(0.0_f64.exp(), 1.0);
    }

    /// 02_4: "four ones squash the whole plane onto a single line": at the
    /// end of the squash every dot of the disc is at (x + y, x + y), the
    /// determinant is zero, and the image is that one line; on the way,
    /// the map is the straight blend of the identity and four ones.
    #[test]
    fn four_ones_put_every_point_on_the_diagonal() {
        let ones = Mat::from_rows(&[&[1.0, 1.0], &[1.0, 1.0]]);
        assert_eq!(matrix("F-H.zero.guess").unwrap(), vec![vec![1.0, 1.0], vec![1.0, 1.0]]);
        let det: f64 = fact("F-H.zero.guess.det").unwrap().parse().unwrap();
        assert_eq!(det, 0.0);
        let end = squash(1.0);
        for p in lattice(SPACING, DISC) {
            let q = apply(&end, p);
            assert_eq!(q, [p[0] + p[1], p[0] + p[1]]);
            assert_eq!(q, [ones.apply(&p)[0], ones.apply(&p)[1]]);
        }
        // the whole image is the diagonal, not a point: the disc's dots land
        // on many places along it
        let mut ends: Vec<f64> = lattice(SPACING, DISC).iter().map(|&p| apply(&end, p)[0]).collect();
        ends.sort_by(f64::total_cmp);
        ends.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
        assert!(ends.len() > 5, "{} places", ends.len());
        // the blend is straight
        let half = squash(0.5);
        assert_eq!(half, [[1.0, 0.5], [0.5, 1.0]]);
        // and the squashed disc stays on the screen: its far end at (x + y)
        // is at most DISC * sqrt 2 from the centre
        let far = ends.iter().fold(0.0_f64, |m, &x| m.max(x.abs()));
        assert!(PLANE.centre[0] + far < 7.0 && far < 4.0, "{far}");
    }

    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("02_the_obvious_guess").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            "e = 2.718\u{2026}",
            "e^{-1}",
            "1 \\times x = x",
            CHANGES_NOTHING,
            "np.exp(A)",
            IS_THAT_RIGHT,
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never typesets {want:?}: {words:?}"
            );
        }
        // five spoken lines and a hold
        assert!(ran.seconds > 25.0 && ran.seconds < 85.0, "{} s", ran.seconds);
    }
}
