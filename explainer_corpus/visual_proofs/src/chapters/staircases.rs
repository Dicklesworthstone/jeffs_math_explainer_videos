//! Chapter 04, staircases: the second trick, and the reason the cubes' square
//! is fifteen on a side. A staircase of five columns rises column by column
//! in the five size colours and its count runs 1, 3, 6, 10, 15 (04_1); a
//! copy lifts off and turns half way round (04_2); the quiz: how wide and
//! how tall a rectangle the two make (04_3); five wide and six tall, every
//! pair of columns coming to six, thirty tiles and so fifteen in each
//! (04_4); the same at every height, the staircase grown to twelve columns,
//! and a hundred steps of tiles (04_5); and the card, with its close-up of
//! three pairs (04_6), after which the fifteen is lifted off in gold to wait
//! for chapter 05.
//!
//! Every tile is `model::layers::staircase(n)` and `model::layers::turned(n)`,
//! and every number is a fact (`F-T.columns`, `F-T.counts`, `F-T.count`,
//! `F-T.rectangle`, `F-T.rectangle.cells`, `F-T.count.hundred`,
//! `F-Q.rectangle.answer`) checked against the tiles before anything shows.

use std::f64::consts::PI;

use explainer_kit::{Kit, Timed, fade_all, play, thousands, v};
use fmn::prelude::*;

use super::draw::{
    ANSWER_PT, Board, CardRow, Cell, LENS_AT, ProofCard, QuizCard, QuizLook, Side, Words, breath, dim, fact_int,
    fact_ints, fade_in_all, fade_in_lagged, fade_out_all, fail, file_on_shelf, flat, fly, formula, group, hero, ink,
    lens, lens_mark, live_counter, lower_third, pause, quiz_named, say, size, swap, tex_row, tween_linear,
};
use crate::model::Rect;
use crate::model::layers::{staircase, turned};

pub struct Staircases {
    pub kit: Kit,
}

/// The narration lines this chapter speaks, in order.
pub const LINES: [&str; 6] = ["04_1", "04_2", "04_3", "04_4", "04_5", "04_6"];

// the chapter's fixed words (numbers are built from facts)
const TILES: &str = "tiles";
const IN_EACH: &str = "in each staircase";
const QUESTION_WIDE: &str = "wide";
const QUESTION_TALL: &str = "tall";
const TRIANGULAR: &str = "triangular numbers:";
const TRIANGULAR_TEXT: &str = "the tiles of a staircase: 1, 3, 6, 10, 15, and on";
const WORD_COUNT: &str = "n: how many columns";
const WORD_TALL: &str = "n + 1: one taller";
const WORD_HALF: &str = "÷ 2: half of the rectangle";
const SHOWN_TEX: &str =
    r"\text{for every whole number } n \text{ from 1, } 1 + 2 + \cdots + n \text{ is half of } n \times (n + 1)";
const ASSUMED_TEX: &str =
    r"\text{column } k \text{ of one meets column } n + 1 - k \text{ of the other, so every pair adds to } n + 1";
const REASON: &str = "pairs of six at five columns";
const PAIR_PLUS: &str = "+";
/// The quiz this chapter asks (`model::QUIZZES`).
const QUIZ: &str = "rectangle";
/// The slot of the shelf this proof takes (03, 04, 06, 09, ...).
const SLOT: usize = 1;

// ------------------------------------------------------------------ geometry

/// Scene units a tile (as chapter 03's), and the board's lower-left corner:
/// the rectangle is five by six, 3.72 tall, under the shelf's floor.
const CELL: f64 = 0.62;
const BOARD_AT: Vec3 = [-5.9, -0.8, 0.0];
/// How far the copy hovers to the right of the first staircase, scene units.
const ASIDE: f64 = 3.5;
/// The column of readouts beside the picture: its middle.
const COLUMN_X: f64 = 2.9;
/// The running count of tiles.
const TOTAL_PT: f64 = 96.0;
/// Where a fifteen waits for chapter 05, in a corner of the frame.
pub const WAITING_AT: Vec3 = [6.1, 3.3, 0.0];
/// The sweep's tile: twelve columns and thirteen rows are 3.51 tall.
const SWEEP_CELL: f64 = 0.27;
/// How many columns the sweep grows to, and the steps of the last line.
const SWEEP_TO: i64 = 12;
const STEPS: i64 = 100;

/// The cells of column `x` of a staircase, as `layers` gives them.
fn column(cells: &[Cell], x: i64) -> Vec<Cell> {
    cells.iter().copied().filter(|c| c.0 == x).collect()
}

/// The five columns of a staircase of `n` as separate pieces, column `x` in
/// size `x + 1`'s colour. Detached.
fn columns_of(stage: &mut Stage<'_>, board: &Board, n: i64) -> fmn::Result<Vec<Mob>> {
    let cells = staircase(n);
    (0..n)
        .map(|x| board.tiles(stage, &column(&cells, x), size(x as usize + 1)))
        .collect()
}

/// The same staircase turned half way round into its place against the first:
/// column `x` of the turned copy is the first staircase's column `n − 1 − x`,
/// so it keeps that column's colour. Detached.
fn turned_columns_of(stage: &mut Stage<'_>, board: &Board, n: i64) -> fmn::Result<Vec<Mob>> {
    let cells = turned(n);
    (0..n)
        .map(|x| board.tiles(stage, &column(&cells, x), size((n - x) as usize)))
        .collect()
}

/// The pair `(a, b)` that meets in column `x` of the rectangle: the first
/// staircase's `x + 1` tiles and the turned copy's `n − x`.
fn pair(n: i64, x: i64) -> (i64, i64) {
    (x + 1, n - x)
}

impl SceneConstruct for Staircases {
    fn name(&self) -> &str {
        "04_staircases"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();

        // the numbers, and the tiles they must agree with
        let columns = fact_ints("F-T.columns")?;
        let counts = fact_ints("F-T.counts")?;
        let n = columns.len() as i64;
        let total = fact_int("F-T.count")?;
        let cells_in_rectangle = fact_int("F-T.rectangle.cells")?;
        let rectangle = fact_ints("F-T.rectangle")?;
        let answer = fact_ints("F-Q.rectangle.answer")?;
        let hundred = fact_int("F-T.count.hundred")?;
        if columns != (1..=n).collect::<Vec<_>>() || staircase(n).len() as i64 != total {
            return Err(fail("F-T.columns and F-T.count are not the staircase's".into()));
        }
        if counts != (1..=n).map(|m| staircase(m).len() as i64).collect::<Vec<_>>() {
            return Err(fail("F-T.counts are not the staircases' tile counts".into()));
        }
        if rectangle != [n, n + 1] || answer != rectangle || cells_in_rectangle != 2 * total {
            return Err(fail("the rectangle's facts are not two staircases of the model".into()));
        }
        if hundred != STEPS * (STEPS + 1) / 2 || staircase(STEPS).len() as i64 != hundred {
            return Err(fail("F-T.count.hundred is not a hundred-step staircase".into()));
        }

        let board = Board::new(BOARD_AT, CELL);

        // ---------------------------------------------------------- 04_1
        // a staircase, column by column; its tiles counted 1, 3, 6, 10, 15
        let line = say(&kit, stage, LINES[0])?;
        let first = columns_of(stage, &board, n)?;
        let tally = stage.add_value_tracker(0.0);
        let counter = live_counter(&kit, stage, tally, TOTAL_PT, ink(), v(COLUMN_X, 1.4))?;
        let tiles_word = kit.text(stage, TILES, 40.0, dim(), v(COLUMN_X, 0.5))?;
        fade_in_all(stage, &[tiles_word], 0.4)?;
        line.wait_for(stage, "Build a staircase")?;
        for (k, &m) in first.iter().enumerate() {
            line.wait_for(stage, ["One,", "two,", "three,", "four,", "five."][k])?;
            play!(
                stage;
                fade_in(stage.arena_mut(), m, v(0.0, -0.3), 1.0)?.rt(0.45),
                tween_linear(tally, counts[k] as f64, 0.45)?,
            );
            kit.mark(stage, "pizz:column");
        }
        line.wait_end(stage)?;

        // ---------------------------------------------------------- 04_2
        // a copy lifts off and turns half way round
        let line = say(&kit, stage, LINES[1])?;
        let copy_cols = columns_of(stage, &board, n)?;
        let outline = board.outline(stage, &staircase(n), flat(ink(), 0.0))?;
        // the outline is a stroke: set it as one
        stage.set_stroke(outline, Some(ink()), Some(3.0), Some(1.0), None, false);
        let mut parts = copy_cols.clone();
        parts.push(outline);
        let copy = group(stage, &parts)?;
        line.wait_for(stage, "make a copy")?;
        fade_in_all(stage, &[copy], 0.4)?;
        let middle = stage.get_center(copy);
        play!(stage; fly(copy, v(middle[0] + ASIDE, middle[1] + 0.2), 1.0, 0.9)?);
        line.wait_for(stage, "turn it half way round")?;
        let spin = Rotating::new(copy).with_angle(PI).rt(1.4);
        play!(stage; spin);
        line.wait_end(stage)?;

        // ---------------------------------------------------------- 04_3
        // your turn: how wide, how tall? the copy hovers, turned, not in place
        let quiz = quiz_named(QUIZ)?;
        let line = say(&kit, stage, quiz.line)?;
        let (wide, tall) = (answer[0], answer[1]);
        let question = format!("? {QUESTION_WIDE}, ? {QUESTION_TALL}");
        let said = format!("{wide} {QUESTION_WIDE}, {tall} {QUESTION_TALL}");
        let card = QuizCard::build(
            &kit,
            stage,
            QUIZ,
            Words::Text(&question),
            Words::Text(&said),
            QuizLook::BESIDE,
        )?;
        line.wait_last(stage)?;
        fade_out_all(stage, &[counter, tiles_word], 0.4)?;
        card.show(stage, 0.5)?;
        card.count(&kit, stage)?;

        // ---------------------------------------------------------- 04_4
        // five wide and six tall: the copy settles, every pair comes to six
        let line = say(&kit, stage, LINES[3])?;
        fade_out_all(stage, &[card.back, card.label, card.question], 0.4)?;
        // the copy settles: from where it hovers, turned about its own middle,
        // to the place the model's turned staircase has it (one tile up)
        let to = board.p(n as f64 / 2.0, (n as f64 + 2.0) / 2.0);
        play!(stage; fly(copy, to, 1.0, 1.1)?);
        // the exact fit: the copy's tiles are the model's turned staircase
        let rectangle_rect = Rect::new(0, 0, n, n + 1);
        let width = board.dimension(
            &kit,
            stage,
            rectangle_rect,
            Side::Above,
            Words::Text(&wide.to_string()),
            34.0,
            ink(),
        )?;
        let height = board.dimension(
            &kit,
            stage,
            rectangle_rect,
            Side::Left,
            Words::Text(&tall.to_string()),
            34.0,
            ink(),
        )?;
        let mut braces = width.mobs().to_vec();
        braces.extend(height.mobs());
        fade_out_all(stage, &[card.answer], 0.3)?;
        fade_in_all(stage, &braces, 0.5)?;
        // under every column, the two heights that meet in it
        line.wait_for(stage, "Each column of one staircase")?;
        let mut pairs: Vec<Vec<Mob>> = Vec::new();
        for x in 0..n {
            let (a, b) = pair(n, x);
            let at = board.p(x as f64 + 0.5, -0.45);
            let parts = [
                (a.to_string(), size(x as usize + 1)),
                (PAIR_PLUS.to_owned(), ink()),
                (b.to_string(), size((n - x) as usize)),
            ];
            let row = tex_row(
                &kit,
                stage,
                &parts.iter().map(|(s, c)| (s.as_str(), *c)).collect::<Vec<_>>(),
                24.0,
                at,
                0.08,
                CELL,
            )?;
            pairs.push(row);
        }
        for row in &pairs {
            fade_in_all(stage, row, 0.35)?;
        }
        line.wait_for(stage, "every pair comes to six")?;
        let sum_word = (n + 1).to_string();
        let comes_to = kit.text(
            stage,
            &format!("= {sum_word}"),
            30.0,
            hero(),
            board.p(n as f64 + 0.45, -0.45),
        )?;
        fade_in_all(stage, &[comes_to], 0.4)?;
        // thirty tiles, so fifteen in each staircase
        line.wait_for(stage, "Thirty tiles")?;
        let tally2 = stage.add_value_tracker(0.0);
        let thirty = live_counter(&kit, stage, tally2, TOTAL_PT, ink(), v(COLUMN_X, 1.4))?;
        let tiles2 = kit.text(stage, TILES, 40.0, dim(), v(COLUMN_X, 0.5))?;
        fade_in_all(stage, &[tiles2], 0.3)?;
        play!(stage; tween_linear(tally2, cells_in_rectangle as f64, 0.9)?);
        line.wait_for(stage, "fifteen in each staircase")?;
        let each = kit.text(stage, &format!("{total} {IN_EACH}"), 34.0, hero(), v(COLUMN_X, -0.4))?;
        fade_in_all(stage, &[each], 0.5)?;
        line.wait_end(stage)?;

        // ---------------------------------------------------------- 04_5
        // at any height: the sweep, the formula, a hundred steps
        let line = say(&kit, stage, LINES[4])?;
        let mut leaving = vec![thirty, tiles2, each, comes_to];
        leaving.extend(&braces);
        leaving.extend(pairs.iter().flatten());
        leaving.extend(&first);
        leaving.extend(&copy_cols);
        leaving.push(outline);
        leaving.push(copy);
        // the first staircase and its copy go, and the sweep's takes their
        // place a size smaller: the rectangle one taller than it is wide, from
        // one column to twelve
        let sweep = Board::new(v(BOARD_AT[0], BOARD_AT[1] + 0.1), SWEEP_CELL);
        let mut stages: Vec<(Mob, Mob, Mob)> = Vec::new();
        for m in 1..=SWEEP_TO {
            let up = sweep.tiles(stage, &staircase(m), super::draw::copy(0))?;
            let down = sweep.tiles(stage, &turned(m), super::draw::copy(1))?;
            // the pairs that meet in the first columns, and dots for the rest:
            // one column has only the one pair, two have both and no more
            let said = match m {
                1 => format!("1 {PAIR_PLUS} 1"),
                2 => format!("1 {PAIR_PLUS} 2   2 {PAIR_PLUS} 1"),
                _ => format!("1 {PAIR_PLUS} {m}   2 {PAIR_PLUS} {}   …", m - 1),
            };
            let label = kit.text(stage, &said, 30.0, ink(), v(COLUMN_X, 1.7))?;
            stages.push((up, down, label));
        }
        let first_stage = [stages[0].0, stages[0].1, stages[0].2];
        swap(stage, &leaving, &first_stage, 0.5)?;
        line.wait_for(stage, "Two staircases always make")?;
        for w in stages.windows(2) {
            let (a, b) = (w[0], w[1]);
            let mut anims: Vec<Box<dyn Animation>> = Vec::new();
            for (old, new) in [(a.0, b.0), (a.1, b.1), (a.2, b.2)] {
                let o = fade_out(stage.arena_mut(), old, ORIGIN, 1.0)?;
                anims.push(stage.prepare(o.rt(0.2))?);
                let i = fade_in(stage.arena_mut(), new, ORIGIN, 1.0)?;
                anims.push(stage.prepare(i.rt(0.2))?);
            }
            stage.play_prepared(anims)?;
        }
        // in letters: the staircase is half of the rectangle
        line.wait_for(stage, "So the staircase is half")?;
        let letters = formula(
            &kit,
            stage,
            r"1 + 2 + \cdots + n = \frac{n\,(n + 1)}{2}",
            52.0,
            ink(),
            v(COLUMN_X, -0.4),
        )?;
        fade_in_all(stage, &[letters.mob], 0.6)?;
        let words: Vec<Mob> = [WORD_COUNT, WORD_TALL, WORD_HALF]
            .iter()
            .enumerate()
            .map(|(i, w)| kit.text(stage, w, 26.0, dim(), v(COLUMN_X, -1.3 - 0.4 * i as f64)))
            .collect::<fmn::Result<_>>()?;
        fade_in_lagged(stage, &words, 1.2, 0.5)?;
        // a hundred steps: five thousand and fifty tiles
        line.wait_for(stage, "A staircase of a hundred steps")?;
        let steps = format!("{STEPS} × {} ÷ 2 = {}", STEPS + 1, thousands(hundred as u64));
        let hundred_line = kit.text(stage, &steps, 40.0, hero(), v(COLUMN_X, 0.7))?;
        let third = lower_third(&kit, stage, TRIANGULAR, TRIANGULAR_TEXT)?;
        let last = stages[stages.len() - 1];
        fade_in_all(stage, &[hundred_line, third], 0.6)?;
        line.wait_end(stage)?;

        // ---------------------------------------------------------- 04_6
        // the card; fifteen lifts off in gold and waits
        let line = say(&kit, stage, LINES[5])?;
        let mut off = vec![last.0, last.1, last.2, letters.mob, hundred_line, third];
        off.extend(&words);
        fade_out_all(stage, &off, 0.5)?;
        // the picture of the card: the two staircases locked, five columns
        let mut locked = columns_of(stage, &board, n)?;
        locked.extend(turned_columns_of(stage, &board, n)?);
        fade_in_all(stage, &locked, 0.6)?;
        let mut card = ProofCard::build(
            &kit,
            stage,
            &[
                CardRow::Shown(Words::Tex(SHOWN_TEX)),
                CardRow::Assumed(Words::Tex(ASSUMED_TEX)),
            ],
        )?;
        line.wait_for(stage, "Remember fifteen")?;
        card.show(stage, 0.6)?;
        // the close-up of the fit: three pairs of columns, six tall each
        let ring = lens(stage);
        let unit = 0.27;
        let mut close = vec![ring];
        let shown_pairs = (n + 1) / 2;
        for k in 0..shown_pairs {
            let (a, b) = pair(n, k);
            let x = LENS_AT[0] + (k as f64 - (shown_pairs as f64 - 1.0) / 2.0) * (unit + 0.14);
            let bottom = LENS_AT[1] - (n as f64 + 1.0) * unit / 2.0;
            for (rows, from, color) in [(a, 0, size(k as usize + 1)), (b, a, size((n - k) as usize))] {
                let tile = Rectangle::new()
                    .width(unit)
                    .height(rows as f64 * unit - 0.03)
                    .style(flat(color, 1.0))
                    .build()?
                    .moved_to(v(x, bottom + (from as f64 + rows as f64 / 2.0) * unit));
                close.push(stage.arena_mut().add(tile));
            }
        }
        fade_in_all(stage, &close, 0.5)?;
        line.wait_for(stage, "The cubes' square")?;
        pause(stage, 0.3)?;
        card.tick(&kit, stage, 1, REASON)?;
        let tick = lens_mark(stage, LENS_AT, 0.7, true);
        fade_in_all(stage, &[tick], 0.3)?;
        pause(stage, 0.8)?;
        // the fifteen lifts off in gold and waits in the corner
        let fifteen = kit.text(
            stage,
            &total.to_string(),
            ANSWER_PT,
            hero(),
            board.p(n as f64 / 2.0, n as f64 / 2.0),
        )?;
        fade_in_all(stage, &[fifteen], 0.4)?;
        play!(stage; fly(fifteen, WAITING_AT, 0.35, 1.0)?);
        // the card has passed: its picture goes to the shelf
        let mut done = close.clone();
        done.push(tick);
        done.extend(card.mobs());
        fade_out_all(stage, &done, 0.4)?;
        let middle = board.p(n as f64 / 2.0, (n as f64 + 1.0) / 2.0);
        file_on_shelf(stage, &locked, middle, (n as f64 + 1.0) * CELL, SLOT, 1.0)?;
        line.wait_end(stage)?;
        breath(&kit, stage)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::testing;
    use super::*;

    /// The chapter speaks exactly the script's 04_ lines, in order.
    #[test]
    fn the_chapter_speaks_the_scripts_lines_in_order() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let ids: Vec<&str> = script
            .lines()
            .iter()
            .map(|(id, _)| id.as_str())
            .filter(|id| id.starts_with("04_"))
            .collect();
        assert_eq!(ids, LINES);
    }

    /// Two staircases lock into an n by n + 1 rectangle, exactly once each
    /// cell, for every size the chapter draws and the cards claim.
    #[test]
    fn two_staircases_lock_into_a_rectangle_for_every_size() {
        for n in 1..=40 {
            let mut cells = staircase(n);
            cells.extend(turned(n));
            crate::model::tiles(cells, Rect::new(0, 0, n, n + 1), &[]).unwrap();
        }
    }

    /// The copy of 04_2 is the model's turned staircase: a half turn about
    /// its own middle, then up one tile (the chapter settles it to
    /// `board.p(n / 2, (n + 2) / 2)`), and every column x of the rectangle
    /// holds the pair (x + 1, n − x) that adds to n + 1.
    #[test]
    fn the_turned_copy_is_a_half_turn_and_the_pairs_add_to_one_more_than_the_width() {
        for n in 1..=30 {
            let mut half_turn: Vec<Cell> = staircase(n).into_iter().map(|(x, y)| (n - 1 - x, n - 1 - y)).collect();
            half_turn.iter_mut().for_each(|c| c.1 += 1);
            let (mut a, mut b) = (half_turn, turned(n));
            a.sort_unstable();
            b.sort_unstable();
            assert_eq!(a, b, "n = {n}");
            for x in 0..n {
                let (up, down) = pair(n, x);
                assert_eq!(up + down, n + 1);
                assert_eq!(column(&staircase(n), x).len() as i64, up);
                assert_eq!(column(&turned(n), x).len() as i64, down);
            }
        }
    }

    /// A staircase of n columns is half of an n by n + 1 rectangle, and a
    /// hundred steps are 5,050 tiles.
    #[test]
    fn a_staircase_is_half_of_its_rectangle() {
        for n in 1..=200 {
            assert_eq!(2 * staircase(n).len() as i64, n * (n + 1));
        }
        assert_eq!(staircase(STEPS).len(), 5050);
    }

    /// The picture stands under the shelf and above the card; the copy's
    /// hover is left of the quiz card (which starts at x = 0.8).
    #[test]
    fn the_layout_stands_between_the_shelf_and_the_card() {
        const {
            assert!(BOARD_AT[1] + 6.0 * CELL <= super::super::draw::SHELF_FLOOR + 1e-9);
            assert!(BOARD_AT[1] - 0.45 - 0.2 > super::super::draw::CARD_TOP - 0.3);
            assert!(BOARD_AT[0] + ASIDE + 5.0 * CELL <= 0.8 + 0.2);
            assert!(WAITING_AT[0] < FRAME_X_RADIUS - 0.4 && WAITING_AT[1] < FRAME_Y_RADIUS - 0.4);
            // the sweep's rectangle, twelve by thirteen, fits the same room
            assert!(13.0 * SWEEP_CELL <= 6.0 * CELL);
        }
    }

    /// The scene constructs end to end and the kit's layout lint finds
    /// nothing.
    #[test]
    fn the_chapter_runs_and_the_layout_lint_is_clean() {
        let ran = testing::run("04_staircases").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        assert!(ran.seconds > 30.0 && ran.seconds < 90.0, "{} s", ran.seconds);
    }
}
