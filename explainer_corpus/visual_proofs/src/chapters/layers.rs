//! Chapter 03, layers: the first whole proof. One tile, and an L-shaped
//! layer wrapped round it, its arms and its corner named (03_1); three more
//! layers, each with its count, and a square of five by five (03_2); the
//! quiz: how many tiles will the next layer take (03_3); eleven, in gold,
//! tile by tile, one at the end of every row, one on top of every column and
//! the corner, and each layer two longer (03_4); the sum, grown from the
//! counts, and the same picture pulled back to the hundredth layer (03_5);
//! and the card, for the first time, with its close-up and its tick (03_6).
//!
//! Every tile is `model::layers::layer(k)`; every number is a fact
//! (`F-O.layers`, `F-O.totals`, `F-O.square`, `F-O.sum`, `F-O.hundredth`,
//! `F-O.sum.hundred`, `F-Q.next_layer.answer`) and is checked against the
//! tiles drawn before anything is shown.

use std::collections::HashSet;

use explainer_kit::{Kit, Timed, fade_all, play, thousands, v};
use fmn::prelude::*;

use super::draw::{
    ANSWER_PT, Board, CardRow, Cell, LENS_AT, ProofCard, QuizCard, QuizLook, Side, Words, dim, fact_int, fact_ints,
    fade_in_all, fade_in_lagged, fade_out_all, faded_line, fail, file_on_shelf, flash, flat, fly, formula, glyphs,
    hero, ink, lens, lens_mark, live_counter, lower_third, middle_of, morph, pause, quiz_named, rgb, say, shelve, size,
    slide_in, swap, tween_linear,
};
use crate::model::Rect;
use crate::model::layers::layer;

pub struct Layers {
    pub kit: Kit,
}

/// The narration lines this chapter speaks, in order.
pub const LINES: [&str; 6] = ["03_1", "03_2", "03_3", "03_4", "03_5", "03_6"];

// the chapter's fixed words (numbers are built from facts)
const TILES: &str = "tiles";
const TILES_AND: &str = "tiles, and";
const ARM: &str = "arm";
const CORNER: &str = "corner";
const QUESTION_TAIL: &str = "next layer: ? tiles";
const GNOMON: &str = "gnomon:";
const GNOMON_TEXT: &str = "the Greek word for this L-shaped layer (Aristotle, Physics III.4)";
const SHOWN_TEX: &str =
    r"\text{for every whole number } n \text{ from 1, the first } n \text{ odd numbers add up to } n \times n";
const ASSUMED: &str = "an L: two arms as long as the side it wraps, and one corner tile";
const REASON: &str = "at every size";
const AT_THE_END: &str = "one at the end of every row";
const ON_TOP: &str = "one on top of every column";
const FOR_THE_CORNER: &str = "one for the corner";
const ROWS_WORD: &str = "rows";
const COLUMNS_WORD: &str = "columns";
/// The quiz this chapter asks (`model::QUIZZES`).
const QUIZ: &str = "next_layer";
/// The slot of the shelf this proof takes (03, 04, 06, 09, 10, 11, ...).
pub const SLOT: usize = 0;

// ------------------------------------------------------------------ geometry

/// Scene units a tile: six layers are 3.72 tall, under the shelf's floor.
const CELL: f64 = 0.62;
/// The lower-left corner of the first tile.
const BOARD_AT: Vec3 = [-5.9, -0.8, 0.0];
/// The counts under the layers' columns.
const COUNT_PT: f64 = 36.0;
/// The middle of the column beside the board, and the widest a line of it
/// may be.
const COLUMN_X: f64 = 2.9;
const COLUMN_ROOM: f64 = 7.4;
/// The running count of tiles, and the sums.
const TOTAL_PT: f64 = 100.0;
const SUM_PT: f64 = 48.0;
/// How many tiles the far picture is on a side: ninety-nine layers' square
/// and the hundredth round it, drawn this many scene units tall in all.
const FAR_TALL: f64 = 3.72;

/// The glyphs each term of a sum `a + b + …` takes, in order: (first glyph,
/// how many). A term is its digits; a plus sign is one glyph.
fn term_glyphs(terms: &[String]) -> Vec<(usize, usize)> {
    let mut at = 0;
    terms
        .iter()
        .map(|t| {
            let range = (at, t.chars().count());
            at += range.1 + 1;
            range
        })
        .collect()
}

/// The rows of the far square outside the near one: the rectangles
/// (x, y, w, h) of the layers `near + 1` to `through`, one strip a row, so
/// that a square of ninety-nine layers is ninety-nine short strips and not
/// ten thousand tiles.
fn far_strips(near: i64, through: i64) -> Vec<Rect> {
    (0..through)
        .map(|y| {
            if y < near {
                Rect::new(near, y, through - near, 1)
            } else {
                Rect::new(0, y, through, 1)
            }
        })
        .collect()
}

/// The picture this card files, rebuilt as it stands on the shelf (the
/// chapters after this one open with the shelf so far): the five layers and
/// the sixth in gold, a square six tiles on a side, thumbnail size in its
/// slot. Detached.
pub fn thumb(_kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let n = fact_ints("F-O.layers")?.len() as i64;
    let board = Board::new(BOARD_AT, CELL);
    let mut picture = Vec::new();
    for k in 1..=n {
        picture.push(board.tiles(stage, &layer(k), size(k as usize))?);
    }
    picture.push(board.tiles(stage, &layer(n + 1), hero())?);
    let middle = board.p((n + 1) as f64 / 2.0, (n + 1) as f64 / 2.0);
    shelve(stage, &picture, middle, (n + 1) as f64 * CELL, SLOT);
    Ok(picture)
}

impl SceneConstruct for Layers {
    fn name(&self) -> &str {
        "03_layers"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();

        // the numbers, and the tiles they must agree with
        let counts = fact_ints("F-O.layers")?;
        let totals = fact_ints("F-O.totals")?;
        let side = fact_ints("F-O.square")?;
        let next = fact_int("F-Q.next_layer.answer")?;
        let n = counts.len() as i64;
        for k in 1..=n {
            if layer(k).len() as i64 != counts[k as usize - 1] {
                return Err(fail(format!("layer {k} of the model is not F-O.layers' count")));
            }
        }
        if totals.len() != counts.len() || side != [n, n] || fact_int("F-O.sum")? != n * n {
            return Err(fail(
                "F-O.totals, F-O.square and F-O.sum do not describe one square".into(),
            ));
        }
        if layer(n + 1).len() as i64 != next {
            return Err(fail("the next layer of the model is not the quiz's answer".into()));
        }
        let steps: HashSet<i64> = counts.windows(2).map(|w| w[1] - w[0]).collect();
        let step = match steps.into_iter().collect::<Vec<_>>().as_slice() {
            [one] if next - counts[counts.len() - 1] == *one => *one,
            _ => return Err(fail("the layers do not grow by one step".into())),
        };
        let hundredth = fact_int("F-O.hundredth")?;
        let hundred_sum = fact_int("F-O.sum.hundred")?;
        let root = (1..=hundred_sum).find(|r| r * r >= hundred_sum).unwrap_or(0);
        if root * root != hundred_sum
            || hundredth != (root - 1) * step + counts[0]
            || layer(root).len() as i64 != hundredth
        {
            return Err(fail(
                "F-O.hundredth and F-O.sum.hundred are not the hundredth layer's".into(),
            ));
        }
        let last = root - 1;

        let board = Board::new(BOARD_AT, CELL);
        let mut pieces = Vec::new();
        for k in 1..=n {
            pieces.push(board.tiles(stage, &layer(k), size(k as usize))?);
        }
        // each layer's count stands under its own column
        let count_at = |k: i64| board.p(k as f64 - 0.5, -0.45);
        let mut labels = Vec::new();
        for (i, c) in counts.iter().enumerate() {
            labels.push(kit.text(stage, &c.to_string(), COUNT_PT, size(i + 1), count_at(i as i64 + 1))?);
        }
        let into_place = board.step(-0.8, -0.8);

        // ---------------------------------------------------------- 03_1
        // one tile; an L-shaped layer of three wraps round it
        let line = say(&kit, stage, LINES[0])?;
        let total = stage.add_value_tracker(0.0);
        let counter = live_counter(&kit, stage, total, TOTAL_PT, ink(), v(COLUMN_X, 1.4))?;
        let tiles_word = kit.text(stage, TILES, 40.0, dim(), v(COLUMN_X, 0.5))?;
        line.wait_for(stage, "One square tile")?;
        play!(
            stage;
            fade_in(stage.arena_mut(), tiles_word, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), pieces[0], ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), labels[0], ORIGIN, 1.0)?.rt(0.5),
            tween_linear(total, totals[0] as f64, 0.5)?,
        );
        line.wait_for(stage, "wrap an L-shaped layer")?;
        play!(stage; slide_in(stage, pieces[1], into_place, 0.9)?);
        // the arms and the corner, named in the picture (two cells of the
        // layer are its straight parts, the one at the bend its corner)
        let arm_right = kit.text_left(stage, ARM, 26.0, dim(), board.p(2.15, 0.5))?;
        let arm_top = kit.text(stage, ARM, 26.0, dim(), board.p(0.5, 2.4))?;
        let corner = kit.text_left(stage, CORNER, 26.0, dim(), board.p(2.15, 1.5))?;
        fade_in_all(stage, &[arm_right, arm_top, corner], 0.5)?;
        line.wait_for(stage, "That takes three more tiles")?;
        play!(
            stage;
            fade_in(stage.arena_mut(), labels[1], ORIGIN, 1.0)?.rt(0.5),
            tween_linear(total, totals[1] as f64, 0.6)?,
        );
        line.wait_end(stage)?;
        fade_out_all(stage, &[arm_right, arm_top, corner], 0.4)?;

        // ---------------------------------------------------------- 03_2
        // wrap it again: five, then seven, then nine; a square of five by five
        let line = say(&kit, stage, LINES[1])?;
        play!(stage; slide_in(stage, pieces[2], into_place, 0.8)?);
        line.wait_for(stage, "the layer takes five")?;
        play!(
            stage;
            fade_in(stage.arena_mut(), labels[2], ORIGIN, 1.0)?.rt(0.4),
            tween_linear(total, totals[2] as f64, 0.5)?,
        );
        for i in 3..counts.len() {
            line.wait_sentence(stage, i - 2)?;
            play!(
                stage;
                slide_in(stage, pieces[i], into_place, 0.6)?,
                fade_in(stage.arena_mut(), labels[i], ORIGIN, 1.0)?.rt(0.6),
                tween_linear(total, totals[i] as f64, 0.6)?,
            );
        }
        let square = Rect::new(0, 0, n, n);
        let tall = side[0].to_string();
        let wide = side[1].to_string();
        let tall = board.dimension(&kit, stage, square, Side::Left, Words::Text(&tall), COUNT_PT, ink())?;
        let wide = board.dimension(&kit, stage, square, Side::Above, Words::Text(&wide), COUNT_PT, ink())?;
        line.wait_end(stage)?;
        let mut sides = tall.mobs().to_vec();
        sides.extend(wide.mobs());
        fade_in_all(stage, &sides, 0.6)?;

        // ---------------------------------------------------------- 03_3
        // your turn: how many tiles will the next layer take?
        let quiz = quiz_named(QUIZ)?;
        let line = say(&kit, stage, quiz.line)?;
        let answer = next.to_string();
        let question = format!("{n} by {n}: {QUESTION_TAIL}");
        let card = QuizCard::build(
            &kit,
            stage,
            QUIZ,
            Words::Text(&question),
            Words::Text(&answer),
            QuizLook::BESIDE,
        )?;
        let outline = board.outline(stage, &layer(n + 1), faded_line(dim(), 3.0, 0.9))?;
        line.wait_last(stage)?;
        let mut gone = sides.clone();
        gone.extend([counter, tiles_word]);
        swap(stage, &gone, &[outline], 0.5)?;
        card.show(stage, 0.5)?;
        card.count(&kit, stage)?;

        // ---------------------------------------------------------- 03_4
        // eleven, in gold: the end of every row, the top of every column,
        // the corner; each new square has a row and a column more
        let line = say(&kit, stage, LINES[3])?;
        let eleven = card.answer;
        play!(
            stage;
            fly(eleven, count_at(n + 1), COUNT_PT / (ANSWER_PT * card.look.scale), 0.9)?,
            fade_out(stage.arena_mut(), card.back, ORIGIN, 1.0)?.rt(0.5),
            fade_out(stage.arena_mut(), card.label, ORIGIN, 1.0)?.rt(0.5),
            fade_out(stage.arena_mut(), card.question, ORIGIN, 1.0)?.rt(0.5),
        );
        // the tiles of the new layer, one mobject each, gold
        let right: Vec<Cell> = (0..n).map(|y| (n, y)).collect();
        let top: Vec<Cell> = (0..n).map(|x| (x, n)).collect();
        let corner_cell: Cell = (n, n);
        let mut all = layer(n + 1);
        all.sort_unstable();
        let mut rest: Vec<Cell> = right.iter().chain(&top).chain([&corner_cell]).copied().collect();
        rest.sort_unstable();
        if all != rest {
            return Err(fail(
                "the sixth layer is not the rows' ends, the columns' tops and the corner".into(),
            ));
        }
        let mut right_tiles = Vec::new();
        for &c in &right {
            right_tiles.push(board.tiles(stage, &[c], hero())?);
        }
        let mut top_tiles = Vec::new();
        for &c in &top {
            top_tiles.push(board.tiles(stage, &[c], hero())?);
        }
        let corner_tile = board.tiles(stage, &[corner_cell], hero())?;
        let tally = stage.add_value_tracker(0.0);
        let tally_at = v(COLUMN_X, 1.4);
        let running = live_counter(&kit, stage, tally, TOTAL_PT, hero(), tally_at)?;
        let which: Vec<Mob> = [AT_THE_END, ON_TOP, FOR_THE_CORNER]
            .iter()
            .map(|w| kit.text(stage, w, 34.0, dim(), v(COLUMN_X, 0.5)))
            .collect::<fmn::Result<_>>()?;
        line.wait_for(stage, "One tile at the end")?;
        fade_in_all(stage, &[which[0]], 0.3)?;
        // one at the end of each of the rows, counted 1 to n
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &right_tiles {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a)?);
        }
        let group = AnimationGroup::with_lag_ratio(stage.arena_mut(), anims, 0.6)?.with_run_time(1.3);
        stage.add_to_scene(running)?;
        play!(stage; group, tween_linear(tally, n as f64, 1.3)?);
        line.wait_for(stage, "one on top of every column")?;
        stage.set_tracker_value(tally, 0.0)?;
        swap(stage, &[which[0]], &[which[1]], 0.3)?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &top_tiles {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a)?);
        }
        let group = AnimationGroup::with_lag_ratio(stage.arena_mut(), anims, 0.6)?.with_run_time(1.3);
        play!(stage; group, tween_linear(tally, n as f64, 1.3)?);
        line.wait_for(stage, "and one for the corner")?;
        swap(stage, &[which[1]], &[which[2]], 0.3)?;
        play!(
            stage;
            fade_in(stage.arena_mut(), corner_tile, ORIGIN, 1.0)?.rt(0.4),
            tween_linear(tally, 1.0, 0.4)?,
        );
        // and the total: eleven, as the count under the sixth column says
        line.wait_sentence(stage, 2)?;
        stage.clear_updaters(running, true);
        fade_out_all(stage, &[running, which[2]], 0.3)?;
        // one more row and one more column: the top row and the right column
        // of the new square, gold, once each
        let mut new_row: Vec<Mob> = top_tiles.clone();
        new_row.push(corner_tile);
        let mut new_column: Vec<Mob> = right_tiles.clone();
        new_column.push(corner_tile);
        flash(stage, &new_row, ink(), 1.18, 0.7)?;
        flash(stage, &new_column, ink(), 1.18, 0.7)?;
        // each layer is two longer than the one before: +2 between the counts
        line.wait_for(stage, "so each layer is two longer")?;
        let mut longer = Vec::new();
        let mut row: Vec<Mob> = labels.clone();
        row.push(eleven);
        let all_counts: Vec<i64> = counts.iter().copied().chain([next]).collect();
        for (i, w) in all_counts.windows(2).enumerate() {
            let at = board.p(i as f64 + 1.0, -0.45);
            longer.push(kit.text(stage, &format!("+{}", w[1] - w[0]), 24.0, dim(), v(at[0], at[1] - 0.45))?);
        }
        fade_in_lagged(stage, &longer, 1.5, 0.5)?;
        line.wait_end(stage)?;

        // the odd numbers, one after another, each in its own colour
        let mut pulses: Vec<Box<dyn Animation>> = Vec::new();
        for (i, &m) in row.iter().enumerate() {
            let own = if i < counts.len() { size(i + 1) } else { hero() };
            let a = fmn::animation::indicate(stage.arena_mut(), m, 1.3, Some(rgb(own)))?;
            pulses.push(stage.prepare(a)?);
        }
        let pulses = AnimationGroup::with_lag_ratio(stage.arena_mut(), pulses, 0.3)?.with_run_time(1.6);
        stage.play(pulses)?;

        // ---------------------------------------------------------- 03_5
        // the sum, grown from the counts; the same picture, pulled back to
        // the hundredth layer
        let line = say(&kit, stage, LINES[4])?;
        let terms: Vec<String> = counts.iter().map(ToString::to_string).collect();
        let sum_src = format!(r"{} = {} \times {}", terms.join(" + "), side[0], side[1]);
        let sum_at = v(COLUMN_X, 1.9);
        // the whole sum is typeset once, to find where its two halves sit
        let whole = formula(&kit, stage, &sum_src, SUM_PT, ink(), sum_at)?;
        let whole_glyphs = glyphs(stage, whole.mob);
        let ranges = term_glyphs(&terms);
        let split = ranges.last().map_or(0, |&(at, len)| at + len);
        if split == 0 || whole_glyphs.len() <= split {
            return Err(fail(format!("the sum {sum_src} has no right-hand side")));
        }
        let lhs_at = middle_of(stage, &whole_glyphs[..split])?;
        let rhs_at = middle_of(stage, &whole_glyphs[split..])?;
        let lhs = kit.tex(stage, &terms.join(" + "), SUM_PT, ink(), &[], lhs_at)?;
        let rhs_src = format!(r"= {} \times {}", side[0], side[1]);
        let rhs = kit.tex(stage, &rhs_src, SUM_PT, ink(), &[], rhs_at)?;
        // the terms wear their layers' colours and wait, unseen, for their counts
        let lhs_glyphs = glyphs(stage, lhs);
        let mut seats = Vec::new();
        for (i, &(at, len)) in ranges.iter().enumerate() {
            let term = &lhs_glyphs[at..at + len];
            for &g in term {
                stage.set_color(g, size(i + 1), None, true);
                stage.set_opacity(g, 0.0, true);
            }
            seats.push(middle_of(stage, term)?);
        }
        // the sixth layer, the +2s and the new count go; the five stay
        let mut out = vec![corner_tile, eleven];
        out.extend(&right_tiles);
        out.extend(&top_tiles);
        out.extend(&longer);
        out.push(outline);
        swap(stage, &out, &[lhs], 0.4)?;
        // each count flies up from under its layer into the sum
        let mut copies = Vec::new();
        let mut flights: Vec<Box<dyn Animation>> = Vec::new();
        for (i, term) in terms.iter().enumerate() {
            let from = stage.get_center(labels[i]);
            let copy = kit.text(stage, term, COUNT_PT, size(i + 1), from)?;
            stage.add_to_scene(copy)?;
            flights.push(stage.prepare(fly(copy, seats[i], SUM_PT / COUNT_PT, 0.8)?)?);
            copies.push(copy);
        }
        let now = stage.scene().time().to_f64();
        let span = (line.at("is five times five")? - now - 0.1).clamp(1.2, 3.0);
        let flights = AnimationGroup::with_lag_ratio(stage.arena_mut(), flights, 0.45)?.with_run_time(span);
        stage.play(flights)?;
        for (&(at, len), &copy) in ranges.iter().zip(&copies) {
            for &g in &lhs_glyphs[at..at + len] {
                stage.set_opacity(g, 1.0, true);
            }
            stage.remove_from_scene(copy);
        }
        line.wait_for(stage, "is five times five")?;
        play!(stage; fade_in(stage.arena_mut(), rhs, v(-0.15, 0.0), 1.0)?.rt(0.5));
        let third = lower_third(&kit, stage, GNOMON, GNOMON_TEXT)?;
        fade_in_all(stage, &[third], 0.5)?;

        // the same picture, pulled back: the five layers scale down to a
        // corner and the rows of ninety-nine layers are there
        line.wait_sentence(stage, 1)?;
        let far_cell = FAR_TALL / root as f64;
        let far = Board::new(BOARD_AT, far_cell);
        let shrink = far_cell / CELL;
        let mut strips: Vec<Mob> = Vec::new();
        for r in far_strips(n, last) {
            strips.push(far.slab(stage, r, dim()));
            let m = *strips.last().unwrap_or(&pieces[0]);
            stage.set_opacity(m, 0.35, true);
        }
        let mut pulling: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &pieces {
            let a = m
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(1.8),
                    ..AnimateArgs::default()
                })?
                .scale_about(shrink, Some(BOARD_AT), None)?;
            pulling.push(stage.prepare(a)?);
        }
        for &m in &labels {
            stage.clear_updaters(m, true);
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            pulling.push(stage.prepare(a.rt(0.6))?);
        }
        stage.play_prepared(pulling)?;
        fade_in_lagged(stage, &strips, 1.4, 0.9)?;
        // ninety-nine rows, ninety-nine columns, one corner: the layer round it
        line.wait_sentence(stage, 2)?;
        let arm_a = far.slab(stage, Rect::new(last, 0, 1, last), hero());
        let arm_b = far.slab(stage, Rect::new(0, last, last, 1), hero());
        let corner_c = far.slab(stage, Rect::new(last, last, 1, 1), hero());
        let ninety_nine = last.to_string();
        let right_edge = far.p(last as f64 + 1.0, 0.0)[0];
        let mut tags = Vec::new();
        tags.push(kit.text_left(
            stage,
            &format!("{ninety_nine} {ROWS_WORD}"),
            26.0,
            hero(),
            v(right_edge + 0.2, far.p(0.0, last as f64 / 2.0)[1]),
        )?);
        tags.push(kit.text_left(
            stage,
            &format!("{ninety_nine} {COLUMNS_WORD}"),
            26.0,
            hero(),
            v(right_edge + 0.2, far.p(0.0, last as f64 + 1.0)[1] + 0.05),
        )?);
        line.wait_for(stage, "Ninety-nine rows")?;
        fade_in_all(stage, &[arm_a, tags[0]], 0.5)?;
        line.wait_for(stage, "ninety-nine columns")?;
        fade_in_all(stage, &[arm_b, tags[1]], 0.5)?;
        line.wait_for(stage, "one corner")?;
        fade_in_all(stage, &[corner_c], 0.4)?;
        // the readouts: the hundredth layer's tiles and the square's
        line.wait_sentence(stage, 3)?;
        let lead = &terms[..terms.len().min(3)];
        let far_src = format!(r"{} + \cdots + {hundredth} = {root} \times {root}", lead.join(" + "));
        let far_sum = formula(&kit, stage, &far_src, SUM_PT, ink(), v(COLUMN_X, 0.5))?;
        let far_glyphs = glyphs(stage, far_sum.mob);
        // a second copy of the first sum, in place, is what turns into the far one
        let seed = formula(&kit, stage, &sum_src, SUM_PT, ink(), sum_at)?;
        let seed_glyphs = glyphs(stage, seed.mob);
        for (i, &(at, len)) in ranges.iter().enumerate() {
            for &g in &seed_glyphs[at..at + len] {
                stage.set_color(g, size(i + 1), None, true);
            }
            if i < lead.len() {
                for &g in far_glyphs.get(at..at + len).unwrap_or_default() {
                    stage.set_color(g, size(i + 1), None, true);
                }
            }
        }
        super::draw::fit_width(stage, far_sum.mob, COLUMN_ROOM);
        swap(stage, &[lhs, rhs], &[seed.mob], 0.01)?;
        morph(stage, &seed, &far_sum, 1.2)?;
        let ten_thousand = kit.text(
            stage,
            &format!(
                "{hundredth} {TILES_AND} {root} × {root} = {}",
                thousands(hundred_sum as u64)
            ),
            34.0,
            ink(),
            v(COLUMN_X, -0.5),
        )?;
        fade_in_all(stage, &[ten_thousand], 0.5)?;
        line.wait_end(stage)?;

        // ---------------------------------------------------------- 03_6
        // the card, for the first time: what was shown, what was assumed
        let line = say(&kit, stage, LINES[5])?;
        let mut off = vec![arm_a, arm_b, corner_c, third, far_sum.mob, ten_thousand];
        off.extend(&tags);
        off.extend(&strips);
        // the pieces come back from the corner to their size
        let mut back: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &pieces {
            let a = m
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(1.2),
                    ..AnimateArgs::default()
                })?
                .scale_about(1.0 / shrink, Some(BOARD_AT), None)?;
            back.push(stage.prepare(a)?);
        }
        for &m in &off {
            stage.clear_updaters(m, true);
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            back.push(stage.prepare(a.rt(0.6))?);
        }
        stage.play_prepared(back)?;
        let sixth = board.tiles(stage, &layer(n + 1), hero())?;
        fade_in_all(stage, &[sixth], 0.5)?;
        let mut card = ProofCard::build(
            &kit,
            stage,
            &[
                CardRow::Shown(Words::Tex(SHOWN_TEX)),
                CardRow::Assumed(Words::Text(ASSUMED)),
            ],
        )?;
        line.wait_for(stage, "what the picture showed")?;
        card.show_lines(stage, 0..1, 0.6)?;
        line.wait_for(stage, "what it took for granted")?;
        card.show_lines(stage, 1..2, 0.6)?;
        // the close-up of the fit the second line names: an arm's last tile
        // beside the square's last row
        line.wait_for(stage, "Each arm is as long")?;
        let ring = lens(stage);
        let unit = 0.46;
        let patch: Vec<(Cell, Srgb)> = {
            let m = n; // the last row and column of the square, and the layer round it
            let mut cells = Vec::new();
            for x in m - 2..=m {
                for y in m - 2..=m {
                    let color = if x == m || y == m {
                        hero()
                    } else {
                        size(x.max(y) as usize + 1)
                    };
                    cells.push(((x, y), color));
                }
            }
            cells
        };
        let mut close = vec![ring];
        for ((x, y), color) in patch {
            let at = v(
                LENS_AT[0] + (x as f64 - (n - 1) as f64) * unit,
                LENS_AT[1] + (y as f64 - (n - 1) as f64) * unit,
            );
            let tile = Rectangle::new()
                .width(unit - 0.04)
                .height(unit - 0.04)
                .style(flat(color, 1.0))
                .build()?
                .moved_to(at);
            close.push(stage.arena_mut().add(tile));
        }
        fade_in_all(stage, &close, 0.5)?;
        pause(stage, 0.6)?;
        card.tick(&kit, stage, 1, REASON)?;
        let tick = lens_mark(stage, LENS_AT, 0.7, true);
        fade_in_all(stage, &[tick], 0.3)?;
        // the card has passed: its picture goes to the shelf
        line.wait_for(stage, "Remember this card")?;
        let mut leaving = close.clone();
        leaving.push(tick);
        leaving.extend(card.mobs());
        fade_out_all(stage, &leaving, 0.4)?;
        let mut picture = pieces.clone();
        picture.push(sixth);
        let middle = board.p((n + 1) as f64 / 2.0, (n + 1) as f64 / 2.0);
        file_on_shelf(stage, &picture, middle, (n + 1) as f64 * CELL, SLOT, 1.0)?;
        // (no breath after the filing: the stage holds a thumbnail and
        // nothing else, and the storyboard's hold of 03_6 is the card read
        // before it goes, which it was)
        line.wait_end(stage)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::testing;
    use super::*;

    #[test]
    fn a_sum_s_terms_know_their_glyphs() {
        let terms: Vec<String> = ["1", "3", "15", "7"].map(String::from).to_vec();
        assert_eq!(term_glyphs(&terms), [(0, 1), (2, 1), (4, 2), (7, 1)]);
    }

    /// P5 trim: after the card is filed on the shelf the stage holds a
    /// thumbnail and nothing else, so the chapter ends a fade (0.8 s) after
    /// the last words and not 2.5 s of breath later (the first draft: blank
    /// but for the thumbnail from 78 s to the end).
    #[test]
    fn the_chapter_ends_a_fade_after_its_last_line() {
        let v = testing::run_voiced("03_layers").unwrap();
        let tail = super::super::sound::after_the_last_line(&v);
        assert!(tail <= 1.4, "{tail:.1} s of picture after the last line");
    }

    /// The chapter speaks exactly the script's 03_ lines, in order.
    #[test]
    fn the_chapter_speaks_the_scripts_lines_in_order() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let ids: Vec<&str> = script
            .lines()
            .iter()
            .map(|(id, _)| id.as_str())
            .filter(|id| id.starts_with("03_"))
            .collect();
        assert_eq!(ids, LINES);
    }

    /// The sixth layer is the rows' ends, the columns' tops and the corner:
    /// the picture of 03_4, for every size. The old layer moved out by one
    /// tile each way leaves the two arms' last tiles bare (its arms end one
    /// short), so a layer is two longer than the one before.
    #[test]
    fn a_layer_is_the_rows_ends_the_columns_tops_and_the_corner() {
        for k in 1..=60 {
            let mut want: Vec<Cell> = (0..k).map(|y| (k, y)).chain((0..k).map(|x| (x, k))).collect();
            want.push((k, k));
            want.sort_unstable();
            let mut got = layer(k + 1);
            got.sort_unstable();
            assert_eq!(got, want, "layer {}", k + 1);
            assert_eq!(layer(k + 1).len() - layer(k).len(), 2, "two longer at {k}");
        }
    }

    /// The far picture is the hundredth layer on a square of ninety-nine:
    /// the strips (one a row) cover the layers six to ninety-nine exactly,
    /// the five near layers and the strips tile the square of ninety-nine,
    /// and the gold arms and corner are the hundredth layer, 199 tiles.
    #[test]
    fn the_far_strips_and_the_hundredth_layer_tile_the_hundred_square() {
        let (near, last) = (5i64, 99i64);
        let strips = far_strips(near, last);
        assert_eq!(strips.len() as i64, last, "one strip a row");
        let mut cells: Vec<(i64, i64)> = strips.iter().flat_map(Rect::cells).collect();
        cells.extend((0..near).flat_map(|x| (0..near).map(move |y| (x, y))));
        crate::model::tiles(cells, Rect::new(0, 0, last, last), &[]).unwrap();
        let arms = [
            Rect::new(last, 0, 1, last),
            Rect::new(0, last, last, 1),
            Rect::new(last, last, 1, 1),
        ];
        let area: i64 = arms.iter().map(Rect::area).sum();
        assert_eq!(area, 199);
        let hundred = Rect::new(0, 0, last + 1, last + 1);
        let mut around: Vec<(i64, i64)> = arms.iter().flat_map(Rect::cells).collect();
        around.extend((0..last).flat_map(|x| (0..last).map(move |y| (x, y))));
        crate::model::tiles(around, hundred, &[]).unwrap();
        assert_eq!(layer(last + 1).len() as i64, 199);
    }

    /// The picture is under the shelf's floor and above the card's top, the
    /// column of readouts stands right of it and left of the quiz card, and
    /// the counts under the columns end above the card.
    #[test]
    fn the_layout_stands_between_the_shelf_and_the_card() {
        const {
            assert!(BOARD_AT[1] + 6.0 * CELL <= super::super::draw::SHELF_FLOOR + 1e-9);
            assert!(BOARD_AT[1] - 0.45 - 0.45 - 0.2 > super::super::draw::CARD_TOP - 0.3);
            assert!(BOARD_AT[0] + 6.0 * CELL < COLUMN_X - COLUMN_ROOM / 2.0 + 3.0);
            assert!(BOARD_AT[0] > -FRAME_X_RADIUS + 0.9);
            // the quiz card (beside, scale 0.72) starts at x = 0.8
            assert!(BOARD_AT[0] + 6.0 * CELL < 0.8 - 0.4);
        }
    }

    /// The scene constructs end to end and the kit's layout lint finds
    /// nothing.
    #[test]
    fn the_chapter_runs_and_the_layout_lint_is_clean() {
        let ran = testing::run("03_layers").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        // the card's rows are set near their full size: none is squeezed to
        // less than three quarters of it
        for (text, scale) in &ran.squeezed {
            assert!(*scale >= 0.75, "{text:?} is squeezed to {scale:.2} of its size");
        }
        assert!(ran.seconds > 40.0 && ran.seconds < 90.0, "{} s", ran.seconds);
    }
}
