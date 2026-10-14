//! Chapter 02, two carts: the smallest system worth a table. Two carts
//! between two walls, three springs of one newton a centimetre (02_1). Slide
//! the left cart a centimetre and two hands appear (02_2); the springs on
//! either side of each cart say what its hand must do, two newtons and minus
//! one (02_3). One move needed two pushes: each part is tied to its
//! neighbours, in a truss and a grid of air as much as here (02_4). The rule
//! from the two moves to the two pushes is a table, read along each row
//! (02_5). Your turn: slide both carts together, what are the pushes (02_6)?
//! One and one, the middle spring not stretched at all, the pushes come back
//! in the pattern of the moves (02_7).
//!
//! Every number is a fact or the model's: the table `K-A1.table`, the pushes
//! `K-A1.push.*`, the quiz answer `K-Q.02_6.answer`; the springs' stretches
//! and the orange arrows are recomputed from the carts' moves every frame
//! (`carts.rs`), never typed.

use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::carts::{CartsPic, CartsSpec, LEFT_ONLY, TOGETHER, pushes_for, row_sum_tex};
use super::draw::{
    MatrixGrid, QuizCard, arrow_points, entries, fade_in_all, fade_out_all, faded_line, group, matrix, minus, moves,
    numbers, own, pushes, run, say, tween_all, whole,
};
use super::truss::{self, Layout};
use crate::model::truss::Truss;

// ------------------------------------------------------------------ choices

/// Where the carts' picture sits while the beats are told.
const CARTS_AT: Vec3 = [0.0, 1.9, 0.0];
/// The three columns of the lower half: moves, table, pushes; and the row
/// they sit on. (09_2's table sits in the same place.)
const MOVES_X: f64 = -3.4;
const PUSHES_X: f64 = 3.4;
const LOWER_Y: f64 = -1.1;
/// The table's and the lists' type size.
const GRID_SIZE: f64 = 52.0;

// ------------------------------------------------------------- the geometry

/// A list of two numbers as the one-column grid's TeX cells.
pub fn column_entries(values: [f64; 2]) -> Vec<Vec<String>> {
    values.iter().map(|&x| vec![whole(x)]).collect()
}

/// A small apple drawn in line, `r` across, detached: the weight a newton is.
fn apple(stage: &mut Stage<'_>, at: Vec3, r: f64) -> fmn::Result<Mob> {
    let dim = Palette::dim();
    let body = Circle::new()
        .radius(r)
        .arc_center(at)
        .style(faded_line(dim, 3.0, 1.0))
        .build();
    let body = stage.arena_mut().add(body);
    let stem = polyline(vec![v(at[0], at[1] + r), v(at[0] + 0.03, at[1] + r + 0.13)])
        .style(faded_line(dim, 3.0, 1.0))
        .build();
    let stem = stage.arena_mut().add(stem);
    let leaf = polyline(vec![
        v(at[0] + 0.04, at[1] + r + 0.08),
        v(at[0] + 0.2, at[1] + r + 0.17),
        v(at[0] + 0.12, at[1] + r + 0.03),
    ])
    .style(faded_line(dim, 3.0, 1.0))
    .build();
    let leaf = stage.arena_mut().add(leaf);
    group(stage, &[body, stem, leaf])
}

/// A thin ring round `mob`, `pad` clear of it, detached.
fn ring(stage: &mut Stage<'_>, mob: Mob, pad: f64) -> Mob {
    let c = stage.get_center(mob);
    let r = stage.get_width(mob).max(stage.get_height(mob)) / 2.0 + pad;
    let ring = Circle::new()
        .radius(r)
        .arc_center(c)
        .style(faded_line(Palette::ink(), 3.0, 1.0))
        .build();
    stage.arena_mut().add(ring)
}

/// A short arrow from `x0` to `x1` at height `y`, detached.
fn flow_arrow(stage: &mut Stage<'_>, x0: f64, x1: f64, y: f64) -> Mob {
    let a = polyline(arrow_points(v(x0, y), v(x1, y), 0.22))
        .style(faded_line(Palette::dim(), 4.0, 1.0))
        .build();
    stage.arena_mut().add(a)
}

/// The faint backdrop of 02_4, behind the carts: a truss and a grid of air
/// cells, every piece joined to the next. In the scene from the start, at
/// opacity 0, so it lies under everything that comes after.
fn backdrop(stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let ink = Palette::ink();
    let t = Truss::pratt();
    let layout = Layout {
        origin: v(-6.5, 1.2),
        scale: 0.2,
    };
    let mut mobs = truss::outline(stage, &t, layout, ink, 1.0);
    // a small grid of air cells, 8 by 3, at the right
    let (cols, rows, cell) = (8, 3, 0.42);
    let (x0, y0) = (2.9, 1.45);
    for r in 0..=rows {
        let y = y0 + cell * f64::from(r);
        let l = polyline(vec![v(x0, y), v(x0 + cell * f64::from(cols), y)])
            .style(faded_line(ink, 2.5, 1.0))
            .build();
        mobs.push(stage.arena_mut().add(l));
    }
    for c in 0..=cols {
        let x = x0 + cell * f64::from(c);
        let l = polyline(vec![v(x, y0), v(x, y0 + cell * f64::from(rows))])
            .style(faded_line(ink, 2.5, 1.0))
            .build();
        mobs.push(stage.arena_mut().add(l));
    }
    for &m in &mobs {
        stage.set_opacity(m, 0.0, true);
        stage.add_to_scene(m)?;
    }
    Ok(mobs)
}

// -------------------------------------------------------------- the scene

pub struct TwoCarts {
    pub kit: Kit,
}

impl SceneConstruct for TwoCarts {
    fn name(&self) -> &str {
        "02_two_carts"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (ink, dim) = (Palette::ink(), Palette::dim());
        let blue = moves();
        let orange = pushes();

        let back = backdrop(stage)?;
        let u = [stage.add_value_tracker(0.0), stage.add_value_tracker(0.0)];
        let pic = CartsPic::build(
            &kit,
            stage,
            CartsSpec {
                centre: CARTS_AT,
                scale: 1.0,
            },
            u,
        )?;

        // ---- 02_1: two carts, three springs, a newton a centimetre
        let line = say(&kit, stage, "02_1")?;
        pic.start(stage)?;
        let mut first = pic.frame.clone();
        first.extend(pic.carts);
        fade_in_all(stage, &first, 0.8)?;
        line.wait_for(stage, "joined by three springs")?;
        fade_in_all(stage, &pic.springs, 0.7)?;
        let note = kit.text(stage, "1 newton per centimetre", 28.0, orange, v(-2.4, 3.45))?;
        let weight = apple(stage, v(0.2, 3.38), 0.19)?;
        line.wait_for(stage, "Each spring pushes back one newton")?;
        fade_in_all(stage, &[note, weight], 0.6)?;

        // ---- 02_2: slide the left cart a centimetre, hold the right one still
        let line = say(&kit, stage, "02_2")?;
        fade_out_all(stage, &[note, weight], 0.4)?;
        tween_all(stage, &[(pic.moves_on, 1.0), (u[0], LEFT_ONLY[0])], 1.3)?;
        line.wait_for(stage, "hold the right cart still")?;
        tween_all(stage, &[(pic.hands_on, 1.0)], 0.6)?;

        // ---- 02_3: what each hand must do
        let line = say(&kit, stage, "02_3")?;
        line.wait_for(stage, "Two springs fight the left cart")?;
        tween_all(stage, &[(pic.pushes_on[0], 1.0)], 0.6)?;
        line.wait_for(stage, "The middle spring shoves the right cart")?;
        tween_all(stage, &[(pic.pushes_on[1], 1.0)], 0.6)?;
        let pushed = pushes_for(LEFT_ONLY);
        let moves_label = kit.text(stage, "moves", 30.0, blue, v(MOVES_X, LOWER_Y + 1.25))?;
        let pushes_label = kit.text(stage, "pushes", 30.0, orange, v(PUSHES_X, LOWER_Y + 1.25))?;
        let moves_list = MatrixGrid::build(
            &kit,
            stage,
            &column_entries(LEFT_ONLY),
            GRID_SIZE,
            blue,
            v(MOVES_X, LOWER_Y),
        )?;
        let pushes_list = MatrixGrid::build(
            &kit,
            stage,
            &column_entries(pushed),
            GRID_SIZE,
            orange,
            v(PUSHES_X, LOWER_Y),
        )?;
        line.wait_for(stage, "a push of minus one")?;
        fade_in_all(
            stage,
            &[moves_label, pushes_label, moves_list.group, pushes_list.group],
            0.7,
        )?;

        // ---- 02_4: one move, two pushes; every part tied to its neighbours
        let line = say(&kit, stage, "02_4")?;
        let zero = ring(stage, moves_list.cells[1][0], 0.2);
        let minus_one = ring(stage, pushes_list.cells[1][0], 0.2);
        fade_in_all(stage, &[zero, minus_one], 0.5)?;
        line.wait_for(stage, "Each part is tied to its neighbours")?;
        let mut seen = Vec::new();
        for &m in &back {
            seen.push(stage.prepare(m.animate().set_anim_args(run(1.0))?.set_opacity(0.2)?)?);
        }
        stage.play_prepared(seen)?;

        // ---- 02_5: the rule is a table, read along each row
        let line = say(&kit, stage, "02_5")?;
        fade_out_all(stage, &[zero, minus_one], 0.4)?;
        let table = MatrixGrid::build(
            &kit,
            stage,
            &entries("K-A1.table", None)?,
            GRID_SIZE,
            ink,
            v(0.0, LOWER_Y),
        )?;
        let into = flow_arrow(stage, MOVES_X + 0.85, -1.55, LOWER_Y);
        let out = flow_arrow(stage, 1.55, PUSHES_X - 0.85, LOWER_Y);
        line.wait_for(stage, "Write it as a table")?;
        fade_in_all(stage, &[table.group, into, out], 0.8)?;
        let matrix_word = kit.text(stage, "matrix", 26.0, dim, v(0.0, LOWER_Y - 1.2))?;
        line.wait_for(stage, "called a matrix")?;
        fade_in_all(stage, &[matrix_word], 0.5)?;
        // the first column is the pushes of 02_3
        let column = table.column_band(stage, 0, own())?;
        let tag = kit.text(
            stage,
            &format!("the pushes of 02_3: {}, {}", whole(pushed[0]), minus(&whole(pushed[1]))),
            26.0,
            dim,
            v(0.0, LOWER_Y + 1.35),
        )?;
        line.wait_for(stage, "Along each row")?;
        fade_in_all(stage, &[column, tag], 0.5)?;
        // then each row, as the sum it is
        let rows = matrix("K-A1.table")?;
        let top = table.row_band(stage, 0, own())?;
        let first_sum = kit.tex(
            stage,
            &row_sum_tex([rows[0][0], rows[0][1]], LEFT_ONLY),
            36.0,
            ink,
            &[],
            v(0.0, LOWER_Y - 1.85),
        )?;
        line.wait_for(stage, "multiply each number by its move")?;
        fade_out_all(stage, &[column, tag], 0.3)?;
        fade_in_all(stage, &[top, first_sum], 0.5)?;
        let bottom = table.row_band(stage, 1, own())?;
        let second_sum = kit.tex(
            stage,
            &row_sum_tex([rows[1][0], rows[1][1]], LEFT_ONLY),
            36.0,
            ink,
            &[],
            v(0.0, LOWER_Y - 2.4),
        )?;
        line.wait_for(stage, "and add")?;
        fade_out_all(stage, &[top], 0.3)?;
        fade_in_all(stage, &[bottom, second_sum], 0.5)?;

        // ---- 02_6: your turn, both carts together
        let line = say(&kit, stage, "02_6")?;
        pic.stop(stage);
        let mut leaving = pic.base();
        leaving.extend(pic.move_arrows);
        leaving.extend(pic.push_arrows);
        leaving.extend(pic.hands);
        leaving.extend([
            moves_label,
            pushes_label,
            moves_list.group,
            pushes_list.group,
            table.group,
            into,
            out,
            matrix_word,
            bottom,
            first_sum,
            second_sum,
        ]);
        leaving.extend(&back);
        fade_out_all(stage, &leaving, 0.6)?;
        let card = QuizCard::build(&kit, stage, "02_6", v(0.0, 0.05), (11.6, 6.6))?;
        let (middle, _, _) = card.content();
        let w = [stage.add_value_tracker(0.0), stage.add_value_tracker(0.0)];
        let small = CartsPic::build(
            &kit,
            stage,
            CartsSpec {
                centre: v(middle[0], middle[1] + 1.15),
                scale: 0.55,
            },
            w,
        )?;
        let row_y = middle[1] - 0.3;
        let card_moves_label = kit.text(stage, "moves", 28.0, blue, v(-3.6, row_y + 0.85))?;
        let card_pushes_label = kit.text(stage, "pushes", 28.0, orange, v(3.6, row_y + 0.85))?;
        let card_moves = MatrixGrid::build(&kit, stage, &column_entries(TOGETHER), 42.0, blue, v(-3.6, row_y))?;
        let card_table = MatrixGrid::build(&kit, stage, &entries("K-A1.table", None)?, 42.0, ink, v(0.0, row_y))?;
        let unknown = [
            kit.text(stage, "?", 54.0, orange, v(3.6, row_y + 0.3))?,
            kit.text(stage, "?", 54.0, orange, v(3.6, row_y - 0.3))?,
        ];
        let card_into = flow_arrow(stage, -2.75, -1.25, row_y);
        let card_out = flow_arrow(stage, 1.25, 2.75, row_y);
        small.start(stage)?;
        let mut problem = small.base();
        problem.extend([
            card_moves_label,
            card_pushes_label,
            card_moves.group,
            card_table.group,
            unknown[0],
            unknown[1],
            card_into,
            card_out,
        ]);
        card.show_frame(stage, &problem, 0.8)?;
        line.wait_last(stage)?;
        card.arm(stage)?;
        card.count(&kit, stage)?;

        // ---- 02_7: one and one, the middle spring not stretched at all
        let line = say(&kit, stage, "02_7")?;
        let answer = numbers("K-Q.02_6.answer")?;
        let card_answer = MatrixGrid::build(
            &kit,
            stage,
            &column_entries([answer[0], answer[1]]),
            42.0,
            orange,
            v(3.6, row_y),
        )?;
        fade_out_all(stage, &unknown, 0.2)?;
        card.land(stage, &[card_answer.group], 0.5)?;
        line.wait_for(stage, "The middle spring is not stretched")?;
        tween_all(stage, &[(w[0], TOGETHER[0]), (w[1], TOGETHER[1])], 1.2)?;
        let sum_a = kit.tex(
            stage,
            &row_sum_tex([rows[0][0], rows[0][1]], TOGETHER),
            34.0,
            ink,
            &[],
            v(-2.6, middle[1] - 1.55),
        )?;
        let sum_b = kit.tex(
            stage,
            &row_sum_tex([rows[1][0], rows[1][1]], TOGETHER),
            34.0,
            ink,
            &[],
            v(2.6, middle[1] - 1.55),
        )?;
        line.wait_for(stage, "The pushes come back in the same pattern")?;
        fade_in_all(stage, &[sum_a, sum_b], 0.6)?;
        // both lists pulse together in gold
        let mut pulse = Vec::new();
        for g in [card_moves.group, card_answer.group] {
            pulse.push(stage.prepare(g.animate().set_anim_args(run(0.5))?.set_color(own())?)?);
        }
        stage.play_prepared(pulse)?;
        kit.hold(stage)?;
        small.stop(stage);
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::{fact, items, testing};
    use super::*;
    use crate::model::carts::{eigen, table};

    /// 02_3: the lists are the facts: the pushes of the left cart's move
    /// are the table's first column, 2 and -1.
    #[test]
    fn the_lists_and_the_first_column_are_the_facts() {
        assert_eq!(column_entries(LEFT_ONLY), vec![vec!["1"], vec!["0"]]);
        assert_eq!(
            column_entries(pushes_for(LEFT_ONLY)),
            vec![vec!["2"], vec!["-1"]],
            "{}",
            fact("K-A1.push.left").unwrap()
        );
        let rows = entries("K-A1.table", None).unwrap();
        assert_eq!(
            (rows[0][0].as_str(), rows[1][0].as_str()),
            ("2", "-1"),
            "the first column"
        );
        assert_eq!(numbers("K-A1.push.left").unwrap(), [2.0, -1.0]);
    }

    /// 02_6 and 02_7: the quiz's answer is the table applied to both carts
    /// together, one and one, and it comes back as the moves: the moves are
    /// an own pattern of the table with eigenvalue one.
    #[test]
    fn the_answer_is_one_and_one_and_it_is_the_moves_again() {
        assert_eq!(items("K-Q.02_6.answer").unwrap(), ["1", "1"]);
        assert_eq!(pushes_for(TOGETHER), TOGETHER);
        let e = eigen();
        assert!((e.values[0] - 1.0).abs() < 1e-9, "{:?}", e.values);
        let k = table();
        assert_eq!(k.mul_vec(&TOGETHER), TOGETHER.to_vec());
    }

    /// The chapter runs on the storyboard with the layout lint clean, types
    /// the strings it should, and stays inside the chapter budget.
    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("02_two_carts").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            "1 newton per centimetre",
            "moves",
            "pushes",
            "matrix",
            "the pushes of 02_3: 2, \u{2013}1",
            r"2 \times 1 + (-1) \times 0 = 2",
            r"(-1) \times 1 + 2 \times 0 = -1",
            "your turn",
            r"2 \times 1 + (-1) \times 1 = 1",
            r"(-1) \times 1 + 2 \times 1 = 1",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never typesets {want:?}: {words:?}"
            );
        }
        assert!(ran.seconds > 45.0 && ran.seconds < 85.0, "{} s", ran.seconds);
    }
}
