//! Chapter 04, the only picture: the water builds one picture; can it alone
//! say whether it is the only one? (04_1); it can: chapter 02's picture has
//! twelve switches, and swapping one's corners keeps every count, so a
//! second picture fits (04_2); the tree's shadows only, and the method's
//! real units build the tree itself (04_3, 04_4, a prediction: could another
//! picture cast them?); no: a switch needs two rows that each have a filled
//! square where the other is empty (04_5), and of any two of the tree's rows
//! one has every square of the other, so it has no switch (04_6); any two
//! pictures with the same shadows are joined by a chain of switches, and
//! from the tree none can start, so it is the only one (04_7); the water
//! answers whether any picture fits, a switch in its picture whether it is
//! the only one (04_8).
//!
//! Every picture comes from the method: chapter 02's water
//! (`facts::face_flow`) and the tree's (`facts::tree_flow`), each replayed
//! by `model::replay`; switches from `model::switches`, nesting from
//! `model::rows_nested`.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    Board, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, path_line, pause, say, swap, think, thumb,
    wait_until,
};
use super::same_shadows::{QUESTIONS, counts, place, run_switch};
use super::stops::read_s;
use super::water::{
    AT, CELL, TAP_GAP, TEXT_X, THINK_AT, THROUGH_Y, TO_DRAIN, TO_SEND, Water, send_quick, tap_at, through_text,
};
use crate::facts::{self, RYSER_YEAR};
use crate::model::{self, Picture, Switch};
use crate::palette;

pub struct TheOnlyPicture {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const WATERS_PICTURE: &str = "the water's picture";
const SECOND_PICTURE: &str = "a second picture, the same shadows";
const FROM_BEFORE: &str = "the switch from before";
const EACH_ROW: [&str; 2] = ["each row has a filled square", "where the other row is empty"];
const NESTED: &str = "of any two rows, one has all the other's squares";
/// The answers 04_8 gives chapter 01's two questions, in their order.
const ANSWERS: [&str; 2] = [
    "yes, exactly when the totals match and every unit gets through",
    "yes, exactly when the water's picture has no switch",
];

/// The words on the right: the first line, and the next ones down.
const TOP_Y: f64 = 2.0;
const NEXT_Y: f64 = 1.4;
const ALONE_Y: f64 = 0.8;
/// The water's picture's switches (04_2): one small copy each, in rows of
/// [`SWITCH_COLS`] under their count on the right; a copy's square, the
/// distance between copies across and down, the top of the first row, and
/// how far the frame round the switch being swapped stands outside its copy.
const SWITCH_COLS: usize = 4;
const SWITCH_CELL: f64 = 0.22;
const SWITCH_PITCH: [f64; 2] = [1.45, 1.35];
const SWITCH_TOP: f64 = 1.55;
const PICKED_GAP: f64 = 0.1;
/// The small copy of chapter 02's picture (04_5): its centre and pitch, and
/// its words on its right.
const SMALL_AT: [f64; 2] = [0.9, 0.55];
const SMALL_CELL: f64 = 0.34;
const SMALL_TEXT_X: f64 = 2.1;
const SMALL_TEXT_Y: [f64; 2] = [0.85, 0.3];
/// The closing questions and their answers (04_8), top to bottom.
const CLOSING_Y: [f64; 4] = [2.0, 1.45, 0.6, 0.05];
/// How far a row's outline stands from its squares' centres, as a share of
/// the pitch: outside the squares, inside the grid's lines (04_6).
const RUN_HALF: f64 = 0.47;

fn switches_text(n: usize) -> String {
    format!("{n} switches")
}

fn none_among_text(found: usize, rectangles: usize) -> String {
    format!("{found} switches among {rectangles} rectangles")
}

fn alone_text(count: usize) -> String {
    format!("{count} picture fits these shadows")
}

/// The exact version of 04_7, kept on screen until it can be read
/// (`read_s`): Ryser's Theorem 3.1 for the video's pictures.
fn chain_claim() -> String {
    format!(
        "any two pictures of filled and empty squares on one grid with the same row counts and the same \
         column counts are joined by a finite chain of switches (Ryser, {RYSER_YEAR})"
    )
}

/// Where the `k`th switch's copy of a `rows` by `cols` picture stands (04_2).
fn switch_at(k: usize, rows: usize, cols: usize) -> [f64; 2] {
    let (w, h) = (SWITCH_CELL * cols as f64, SWITCH_CELL * rows as f64);
    let (row, col) = (k / SWITCH_COLS, k % SWITCH_COLS);
    [
        TEXT_X + w / 2.0 + SWITCH_PITCH[0] * col as f64,
        SWITCH_TOP - h / 2.0 - SWITCH_PITCH[1] * row as f64,
    ]
}

/// A small copy of `p` at `at`, switch `s`'s four corners ringed: one of its
/// switches, apart from the others (04_2). Detached.
fn switch_copy(stage: &mut Stage<'_>, p: &Picture, s: Switch, at: [f64; 2]) -> fmn::Result<Vec<Mob>> {
    let mut out = thumb(stage, p, v(at[0], at[1]), SWITCH_CELL, Palette::faint())?;
    let small = Board::new(at, SWITCH_CELL, p.rows(), p.cols());
    out.extend(s.corners().map(|(r, c)| small.ring(stage, r, c, palette::switch())));
    Ok(out)
}

/// A frame round the copy at `at` of a `rows` by `cols` picture, outside its
/// own: the switch being swapped on the grid (04_2). Detached.
fn picked_frame(stage: &mut Stage<'_>, at: [f64; 2], rows: usize, cols: usize) -> Mob {
    let half_w = SWITCH_CELL * cols as f64 / 2.0 + PICKED_GAP;
    let half_h = SWITCH_CELL * rows as f64 / 2.0 + PICKED_GAP;
    let corners = [
        v(at[0] - half_w, at[1] + half_h),
        v(at[0] + half_w, at[1] + half_h),
        v(at[0] + half_w, at[1] - half_h),
        v(at[0] - half_w, at[1] - half_h),
        v(at[0] - half_w, at[1] + half_h),
    ];
    path_line(stage, &corners, palette::switch(), 3.0)
}

/// The squares row `r` of `p` holds, as one run of columns, first to last;
/// `None` for an empty row or one with a gap.
fn run(p: &Picture, r: usize) -> Option<(usize, usize)> {
    let held: Vec<usize> = (0..p.cols()).filter(|&c| p.get(r, c)).collect();
    let (&first, &last) = (held.first()?, held.last()?);
    (last - first + 1 == held.len()).then_some((first, last))
}

/// An outline round the squares of row `r` from column `first` to `last`.
/// Detached.
fn run_outline(stage: &mut Stage<'_>, board: &Board, r: usize, (first, last): (usize, usize), color: Srgb) -> Mob {
    let h = board.cell * RUN_HALF;
    let (left, right) = (board.at(r, first)[0] - h, board.at(r, last)[0] + h);
    let y = board.at(r, first)[1];
    let corners = [
        v(left, y + h),
        v(right, y + h),
        v(right, y - h),
        v(left, y - h),
        v(left, y + h),
    ];
    path_line(stage, &corners, color, 3.0)
}

impl SceneConstruct for TheOnlyPicture {
    fn name(&self) -> &str {
        "04_the_only_picture"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let face_shadows = model::face().shadows();
        let (_, face_states) = facts::face_flow().map_err(fail)?;
        let built = face_states
            .last()
            .ok_or_else(|| fail("the face's flow sends nothing".into()))?
            .wet
            .clone();
        if built.shadows() != face_shadows {
            return Err(fail(format!("{} does not cast the face's shadows", built.pattern())));
        }
        let found = model::switches(&built);
        let &first = found
            .first()
            .ok_or_else(|| fail("the water's picture has no switch".into()))?;
        let sides = first
            .sides(&built)
            .ok_or_else(|| fail(format!("{first:?} is not a switch of the water's picture")))?;
        let tree = model::tree();
        let tree_shadows = tree.shadows();
        let (tree_water, tree_states) = facts::tree_flow().map_err(fail)?;
        let tree_built = &tree_states
            .last()
            .ok_or_else(|| fail("the tree's flow sends nothing".into()))?
            .wet;
        let tree_switches = model::switches(&tree);
        if *tree_built != tree || !tree_switches.is_empty() || !model::rows_nested(&tree) {
            return Err(fail(format!(
                "the tree's water built {}, or the tree has a switch or rows that do not nest",
                tree_built.pattern()
            )));
        }
        let only = model::fits(&tree_shadows).len();
        if only != 1 {
            return Err(fail(format!("{only} pictures fit the tree's shadows, not one")));
        }
        let runs: Vec<(usize, usize)> = (0..tree.rows())
            .map(|r| run(&tree, r))
            .collect::<Option<_>>()
            .ok_or_else(|| fail("a row of the tree is not one run of squares".into()))?;
        let board = Board::new(AT, CELL, built.rows(), built.cols());

        // 04_1: chapter 02's grid with the face's counts; the water's
        // picture, square by square
        let line = say(&kit, stage, "04_1")?;
        let grid = board.lines(stage);
        let row_counts = counts(&kit, stage, &face_shadows.rows, Palette::ink(), |r| tap_at(&board, r))?;
        let col_counts = counts(&kit, stage, &face_shadows.cols, Palette::ink(), |c| {
            board.col_count_at(c)
        })?;
        let mut arriving = grid.clone();
        arriving.extend(&row_counts);
        arriving.extend(&col_counts);
        fade_in_all(stage, &arriving, 0.5)?;
        line.wait_for(stage, "builds one picture")?;
        let mut squares = place(stage, &board, &built)?;
        let ordered: Vec<Mob> = built.filled().iter().map(|rc| squares[rc]).collect();
        let label = kit.text(
            stage,
            WATERS_PICTURE,
            24.0,
            Palette::dim(),
            v(board.centre[0], board.top() + 0.4),
        )?;
        fade_in_lagged(stage, &ordered, 0.9, 0.3)?;
        fade_in_all(stage, &[label], 0.3)?;

        // 04_2: its twelve switches, counted as they are spoken, each on a
        // small copy of its own on the right; the first swapped on the grid,
        // its copy framed; every count stays, and a second picture fits
        let line = say(&kit, stage, "04_2")?;
        line.wait_for(stage, "This one has")?;
        let how_many = kit.text_left(
            stage,
            &switches_text(found.len()),
            26.0,
            palette::switch(),
            v(TEXT_X, TOP_Y),
        )?;
        let copies: Vec<Vec<Mob>> = found
            .iter()
            .enumerate()
            .map(|(k, &s)| switch_copy(stage, &built, s, switch_at(k, built.rows(), built.cols())))
            .collect::<fmn::Result<_>>()?;
        fade_in_all(stage, &[how_many], 0.3)?;
        for one in &copies {
            fade_in_all(stage, one, 0.1)?;
        }
        line.wait_for(stage, "Swap the corners")?;
        let picked = picked_frame(
            stage,
            switch_at(0, built.rows(), built.cols()),
            built.rows(),
            built.cols(),
        );
        let rings = first.corners().map(|(r, c)| board.ring(stage, r, c, palette::switch()));
        let mut marked = rings.to_vec();
        marked.push(picked);
        fade_in_all(stage, &marked, 0.3)?;
        let second = run_switch(stage, &board, &mut squares, &built, first, 0.6)?;
        if second.shadows() != face_shadows || second == built {
            return Err(fail(format!(
                "the switch made {}, not a second picture with the face's shadows",
                second.pattern()
            )));
        }
        line.wait_for(stage, "every count stays")?;
        let touched: Vec<Mob> = first
            .rows
            .iter()
            .map(|&r| row_counts[r])
            .chain(first.cols.iter().map(|&c| col_counts[c]))
            .collect();
        flash_all(stage, &touched, 0.5)?;
        let same_rows = counts(&kit, stage, &second.row_counts(), palette::matches(), |r| {
            tap_at(&board, r)
        })?;
        let same_cols = counts(&kit, stage, &second.col_counts(), palette::matches(), |c| {
            board.col_count_at(c)
        })?;
        let mut old = row_counts.clone();
        old.extend(&col_counts);
        let mut new = same_rows.clone();
        new.extend(&same_cols);
        swap(stage, &old, &new, 0.4)?;
        line.wait_for(stage, "A second picture")?;
        let second_label = kit.text(
            stage,
            SECOND_PICTURE,
            24.0,
            Palette::dim(),
            v(board.centre[0], board.top() + 0.4),
        )?;
        fade_out_all(stage, &[label], 0.2)?;
        fade_in_all(stage, &[second_label], 0.2)?;
        let second_up = stage.scene().time().to_f64();

        // 04_3: the second picture, its name and the switches stay into the
        // line's opening until the name can be read, then go; the tree's
        // shadows only, as taps and drains; the method's real units build a
        // picture, the last one taking water back
        let line = say(&kit, stage, "04_3")?;
        let mut leaving: Vec<Mob> = std::mem::take(&mut squares).into_values().collect();
        leaving.extend(marked);
        leaving.extend(copies.into_iter().flatten());
        leaving.extend(&same_rows);
        leaving.extend(&same_cols);
        leaving.extend([second_label, how_many]);
        wait_until(stage, second_up + read_s(SECOND_PICTURE))?;
        fade_out_all(stage, &leaving, 0.4)?;
        line.wait_for(stage, "Rows of")?;
        let taps = counts(&kit, stage, &tree_shadows.rows, Palette::ink(), |r| tap_at(&board, r))?;
        let send_label = kit.text(
            stage,
            TO_SEND,
            22.0,
            Palette::dim(),
            v(board.left() - TAP_GAP, board.top() + 0.4),
        )?;
        fade_in_lagged(stage, &taps, 0.8, 0.3)?;
        fade_in_all(stage, &[send_label], 0.3)?;
        line.wait_for(stage, "columns of")?;
        let drains = counts(&kit, stage, &tree_shadows.cols, Palette::ink(), |c| {
            board.col_count_at(c)
        })?;
        let drain_label = kit.text(
            stage,
            TO_DRAIN,
            22.0,
            Palette::dim(),
            v(board.left() - 1.0, board.col_count_at(0)[1]),
        )?;
        let counter = kit.text_left(
            stage,
            &through_text(0, tree_shadows.row_total()),
            26.0,
            Palette::ink(),
            v(TEXT_X, THROUGH_Y),
        )?;
        fade_in_lagged(stage, &drains, 0.8, 0.3)?;
        fade_in_all(stage, &[drain_label, counter], 0.3)?;
        line.wait_for(stage, "The water builds")?;
        let mut water = Water::new(board, taps, drains, &tree_shadows);
        water.through = Some(counter);
        for (path, state) in tree_water.paths.iter().zip(&tree_states) {
            send_quick(&kit, stage, &mut water, &tree_shadows, path, state)?;
        }

        // 04_4: the counters become the tree's counts; the water built the
        // tree itself; the question, and the think time (nothing moves)
        let line = say(&kit, stage, "04_4")?;
        let tree_rows = counts(&kit, stage, &tree.row_counts(), palette::matches(), |r| {
            tap_at(&board, r)
        })?;
        let tree_cols = counts(&kit, stage, &tree.col_counts(), palette::matches(), |c| {
            board.col_count_at(c)
        })?;
        let mut old = water.taps.clone();
        old.extend(&water.drains);
        let mut new = tree_rows.clone();
        new.extend(&tree_cols);
        fade_out_all(stage, &old, 0.25)?;
        fade_in_all(stage, &new, 0.25)?;
        let mut going = vec![send_label, drain_label];
        going.extend(water.through);
        fade_out_all(stage, &going, 0.3)?;
        line.wait_for(stage, "the tree itself")?;
        let wet: Vec<Mob> = tree
            .filled()
            .iter()
            .map(|rc| water.wet.get(rc).copied())
            .collect::<Option<_>>()
            .ok_or_else(|| fail("a square of the tree is not wet".into()))?;
        let tree_label = kit.text(
            stage,
            WATERS_PICTURE,
            24.0,
            Palette::dim(),
            v(board.centre[0], board.top() + 0.4),
        )?;
        flash_all(stage, &wet, 0.6)?;
        fade_in_all(stage, &[tree_label], 0.3)?;
        let q = think(&kit, stage, v(THINK_AT[0], THINK_AT[1]))?;

        // 04_5: what a switch needs, on the switch from 04_2: each of its
        // rows has a filled square where the other is empty
        let line = say(&kit, stage, "04_5")?;
        fade_out_all(stage, &[q], 0.3)?;
        line.wait_for(stage, "A switch needs")?;
        let small = Board::new(SMALL_AT, SMALL_CELL, built.rows(), built.cols());
        let mut copy = small.lines(stage);
        copy.extend(place(stage, &small, &built)?.into_values());
        copy.push(small.rectangle(stage, first, palette::switch()));
        copy.push(kit.text(
            stage,
            FROM_BEFORE,
            20.0,
            Palette::dim(),
            v(small.centre[0], small.top() + 0.3),
        )?);
        fade_in_all(stage, &copy, 0.5)?;
        line.wait_for(stage, "each have a filled square")?;
        let corner_rings = sides.emptied.map(|(r, c)| small.ring(stage, r, c, palette::switch()));
        let mut words = Vec::with_capacity(EACH_ROW.len());
        for (text, y) in EACH_ROW.iter().zip(SMALL_TEXT_Y) {
            words.push(kit.text_left(stage, text, 24.0, Palette::ink(), v(SMALL_TEXT_X, y))?);
        }
        fade_in_all(stage, &corner_rings, 0.4)?;
        fade_in_all(stage, &words, 0.4)?;
        let words_up = stage.scene().time().to_f64();

        // 04_6: the tree's rows nest, shortest first, while the switch's copy
        // and words stay on the right until read; then they go, and the
        // nesting's words take their place; no switch among its rectangles
        let line = say(&kit, stage, "04_6")?;
        line.wait_for(stage, "of any two rows")?;
        let mut order: Vec<usize> = (0..tree.rows()).collect();
        order.sort_by_key(|&r| (runs[r].1 - runs[r].0, r));
        let outlines: Vec<Mob> = order
            .iter()
            .map(|&r| run_outline(stage, &board, r, runs[r], Palette::ink()))
            .collect();
        let nested = kit.text_left(stage, NESTED, 24.0, Palette::ink(), v(TEXT_X, TOP_Y))?;
        fade_in_lagged(stage, &outlines, 1.2, 0.3)?;
        copy.extend(corner_rings);
        copy.extend(words);
        wait_until(stage, words_up + read_s(&EACH_ROW.join(" ")))?;
        fade_out_all(stage, &copy, 0.4)?;
        fade_in_all(stage, &[nested], 0.3)?;
        line.wait_for(stage, "no switch")?;
        let none = kit.text_left(
            stage,
            &none_among_text(tree_switches.len(), model::rectangles(&tree)),
            26.0,
            palette::switch(),
            v(TEXT_X, NEXT_Y),
        )?;
        fade_in_all(stage, &[none], 0.4)?;

        // 04_7: Ryser's theorem, scoped, held until it can be read; from the
        // tree no chain can start: one picture fits
        let line = say(&kit, stage, "04_7")?;
        let claim = chain_claim();
        let card = precisely(&kit, stage, &claim)?;
        fade_in_all(stage, &[card], 0.4)?;
        let card_up = stage.scene().time().to_f64();
        line.wait_for(stage, "no other picture fits")?;
        let alone = kit.text_left(stage, &alone_text(only), 26.0, palette::matches(), v(TEXT_X, ALONE_Y))?;
        fade_in_all(stage, &[alone], 0.4)?;
        line.wait_to_end(stage)?;
        wait_until(stage, card_up + read_s(&claim))?;

        // 04_8: chapter 01's two questions, each with its answer as it is
        // spoken; held until they can be read, then the fade
        let line = say(&kit, stage, "04_8")?;
        let mut leaving = outlines;
        leaving.extend([card, nested, none, alone]);
        fade_out_all(stage, &leaving, 0.4)?;
        let mut closing = Vec::with_capacity(4);
        for (i, (question, answer)) in QUESTIONS.iter().zip(ANSWERS).enumerate() {
            closing.push(kit.text_left(stage, question, 28.0, Palette::ink(), v(TEXT_X, CLOSING_Y[2 * i]))?);
            closing.push(kit.text_left(stage, answer, 22.0, Palette::ink(), v(TEXT_X, CLOSING_Y[2 * i + 1]))?);
        }
        line.wait_for(stage, "the water answers")?;
        fade_in_all(stage, &closing[..2], 0.4)?;
        let shown = stage.scene().time().to_f64();
        line.wait_for(stage, "A switch in")?;
        fade_in_all(stage, &closing[2..], 0.4)?;
        line.wait_to_end(stage)?;
        let all_words = [QUESTIONS[0], ANSWERS[0], QUESTIONS[1], ANSWERS[1]].join(" ");
        wait_until(stage, shown + read_s(&all_words))?;
        pause(stage, 1.2)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use explainer_kit::animatic::wrap;
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::*;

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("04_1", "builds one picture"),
            ("04_2", "This one has"),
            ("04_2", "Swap the corners"),
            ("04_2", "every count stays"),
            ("04_2", "A second picture"),
            ("04_3", "Rows of"),
            ("04_3", "columns of"),
            ("04_3", "The water builds"),
            ("04_4", "the tree itself"),
            ("04_5", "A switch needs"),
            ("04_5", "each have a filled square"),
            ("04_6", "of any two rows"),
            ("04_6", "no switch"),
            ("04_7", "no other picture fits"),
            ("04_8", "the water answers"),
            ("04_8", "A switch in"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The words on screen are the facts': the water's picture's switches,
    /// none among the tree's rectangles, the one picture that fits. The card
    /// states Ryser's theorem within its scope, keeps to two whole lines at
    /// the band's width and to three at every width it may narrow to; it
    /// and the closing words are held long enough to read and under twelve
    /// seconds each.
    #[test]
    fn the_words_on_screen_are_built_from_the_facts() {
        let f = facts::facts().unwrap();
        let get = |key: &str| -> usize { f.get(key).unwrap().parse().unwrap() };
        assert_eq!(switches_text(get("DT-F10.face.switches")), "12 switches");
        assert_eq!(
            none_among_text(get("DT-F10.tree.built.switches"), get("DT-F10.tree.rectangles")),
            "0 switches among 100 rectangles"
        );
        assert_eq!(alone_text(get("DT-F6.tree.fits")), "1 picture fits these shadows");
        let claim = chain_claim();
        for part in [
            "any two pictures of filled and empty squares on one grid",
            "the same row counts and the same column counts",
            "a finite chain of switches",
            "Ryser, 1957",
        ] {
            assert!(claim.contains(part), "{part}: {claim}");
        }
        assert!(wrap(&claim, 92, 3).len() <= 2, "{claim}");
        for width in (76..=92).step_by(4) {
            let lines = wrap(&claim, width, 3);
            assert!(!lines.iter().any(|l| l.ends_with('…')), "{width}: {lines:?}");
        }
        let all_words = [QUESTIONS[0], ANSWERS[0], QUESTIONS[1], ANSWERS[1]].join(" ");
        for text in [claim.as_str(), all_words.as_str()] {
            assert!(read_s(text) < 12.0, "{text}");
        }
    }

    /// A row's run is its squares first to last, and only when they have no
    /// gap; every row of the tree is one run, and each run lies within the
    /// columns of every longer one (its rows nest).
    #[test]
    fn the_trees_rows_are_runs_that_nest() {
        let p = Picture::parse(&["01110", "10100", "00000"]).unwrap();
        assert_eq!(run(&p, 0), Some((1, 3)));
        assert_eq!(run(&p, 1), None);
        assert_eq!(run(&p, 2), None);
        let tree = model::tree();
        let runs: Vec<(usize, usize)> = (0..tree.rows()).map(|r| run(&tree, r).unwrap()).collect();
        assert_eq!(runs, [(2, 2), (1, 3), (0, 4), (2, 2), (2, 2)]);
        for a in &runs {
            for b in &runs {
                assert!((b.0 <= a.0 && a.1 <= b.1) || (a.0 <= b.0 && b.1 <= a.1), "{a:?} {b:?}");
            }
        }
    }

    /// The chapter's layout keeps its pieces apart and inside the
    /// title-safe area (text 0.25 in from the frame's edge; about 0.0045
    /// units a point a character).
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        let board = Board::new(AT, CELL, 5, 5);
        let small = Board::new(SMALL_AT, SMALL_CELL, 5, 5);
        let safe_x = FRAME_X_RADIUS - 0.25;
        let safe_y = FRAME_Y_RADIUS - 0.25;
        let ends = |x: f64, text: &str, size: f64| x + text.chars().count() as f64 * size * 0.0045;
        // the words on the right clear the grid, and end inside
        assert!(TEXT_X > board.right() + 0.5);
        assert!(ends(TEXT_X, NESTED, 24.0) < safe_x);
        assert!(ends(TEXT_X, ANSWERS[0], 22.0) < safe_x);
        assert!(ends(TEXT_X, ANSWERS[1], 22.0) < safe_x);
        assert!(ends(TEXT_X, &none_among_text(0, 100), 26.0) < safe_x);
        // the switches' copies (04_2): under their count, right of the grid,
        // apart with room for the frame round one, and inside
        let switches: usize = facts::facts()
            .unwrap()
            .get("DT-F10.face.switches")
            .unwrap()
            .parse()
            .unwrap();
        let (w, h) = (SWITCH_CELL * 5.0, SWITCH_CELL * 5.0);
        let first = switch_at(0, 5, 5);
        assert!(first[1] + h / 2.0 + PICKED_GAP + 0.15 < TOP_Y - 0.15, "under the count");
        assert!(first[0] - w / 2.0 - PICKED_GAP > board.right() + 0.5);
        assert!(SWITCH_PITCH[0] - w > 2.0 * PICKED_GAP + 0.1);
        assert!(SWITCH_PITCH[1] - h > 2.0 * PICKED_GAP);
        for k in 0..switches {
            let at = switch_at(k, 5, 5);
            assert!(at[0] + w / 2.0 + PICKED_GAP < safe_x, "{k}");
            assert!(at[1] - h / 2.0 - PICKED_GAP > -safe_y, "{k}");
        }
        // the small copy stands right of the grid, its words right of it
        assert!(small.left() > board.right() + 0.5);
        assert!(SMALL_TEXT_X > small.right() + 0.2);
        for text in EACH_ROW {
            assert!(ends(SMALL_TEXT_X, text, 24.0) < safe_x, "{text}");
        }
        assert!(small.top() + 0.3 + 0.2 < safe_y);
        // the labels above the grid stay inside and clear the taps' label
        assert!(board.top() + 0.4 + 0.2 < safe_y);
        let half = SECOND_PICTURE.chars().count() as f64 * 24.0 * 0.0045 / 2.0;
        assert!(board.centre[0] - half > -safe_x);
        // the lowest words stand over a two-line card (its top 0.2 + 0.3 +
        // 2 x 0.36 above the frame's bottom), as do the column counts
        let card_top = -FRAME_Y_RADIUS + 0.2 + 0.3 + 2.0 * 0.36;
        assert!(CLOSING_Y[3] - 0.15 > card_top);
        assert!(board.col_count_at(0)[1] - 0.2 > card_top);
        // a row's outline stays inside its grid square's lines (half the
        // pitch) and outside its filled square (0.84 of the pitch)
        const { assert!(RUN_HALF < 0.5 && RUN_HALF > 0.42) };
    }
}
