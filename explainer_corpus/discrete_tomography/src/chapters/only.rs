//! Chapter 08, the only picture: the water builds one picture; can it alone
//! say whether it is the only one? (08_1); it can: chapter 03's picture has
//! twelve switches, and swapping one's corners keeps every count, so a
//! second picture fits (08_2); the tree's shadows from before, and the
//! method's real units build the tree itself (08_3, 08_4, a prediction:
//! could another picture cast them? a callback to chapter 02); no: a switch
//! needs two rows that each have a filled square where the other is empty
//! (08_5), and of any two of the tree's rows one has every square of the
//! other, so it has no switch (08_6); a switch is what stopped reasoning on
//! the face: no reasoning can settle a square a switch changes (08_7); on a
//! four by four grid only about one picture in ten is the only one with its
//! shadows (08_8).
//!
//! Every picture comes from the method: chapter 03's water
//! (`facts::face_flow`) and the tree's (`facts::tree_flow`), each replayed
//! by `model::replay`; switches from `model::switches`, nesting from
//! `model::rows_nested`; the undecided squares from `model::line_solve`; the
//! four by four count from `facts::edges`. Ryser's chain theorem, which the
//! tree's uniqueness rests on, is chapter 09's.

use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    Board, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, path_line, pause, say, swap, think, thumb,
    wait_until,
};
use super::same_shadows::{counts, place, run_switch};
use super::stops::read_s;
use super::water::{
    AT, CELL, TEXT_X, THINK_AT, THROUGH_Y, Water, drain_label, send_label, send_quick, tap_at, through_text,
};
use crate::facts::{self, EDGE_SIDE, RYSER_YEAR};
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
const STUCK: [&str; 2] = ["squares some switch changes", "the shadows cannot settle them"];
/// The answers 09_8 gives chapter 01's two questions, in their order.
pub(crate) const ANSWERS: [&str; 2] = [
    "yes, exactly when the totals match and every unit gets through",
    "yes, exactly when the water's picture has no switch",
];

/// The words on the right: the first line, and the next ones down.
const TOP_Y: f64 = 2.0;
const NEXT_Y: f64 = 1.4;
pub(crate) const ALONE_Y: f64 = 0.8;
/// The water's picture's switches (08_2): one small copy each, in rows of
/// [`SWITCH_COLS`] under their count on the right; a copy's square, the
/// distance between copies across and down, the top of the first row, and
/// how far the frame round the switch being swapped stands outside its copy.
const SWITCH_COLS: usize = 4;
const SWITCH_CELL: f64 = 0.22;
const SWITCH_PITCH: [f64; 2] = [1.45, 1.35];
const SWITCH_TOP: f64 = 1.55;
const PICKED_GAP: f64 = 0.1;
/// The small copy of chapter 03's picture (08_5, 08_7): its centre and pitch, and
/// its words on its right.
const SMALL_AT: [f64; 2] = [0.9, 0.55];
const SMALL_CELL: f64 = 0.34;
const SMALL_TEXT_X: f64 = 2.1;
const SMALL_TEXT_Y: [f64; 2] = [0.85, 0.3];
/// The closing questions and their answers (09_8), top to bottom.
pub(crate) const CLOSING_Y: [f64; 4] = [2.0, 1.45, 0.6, 0.05];
/// How far a row's outline stands from its squares' centres, as a share of
/// the pitch: outside the squares, inside the grid's lines (08_6).
const RUN_HALF: f64 = 0.47;

fn switches_text(n: usize) -> String {
    format!("{n} switches")
}

fn none_among_text(found: usize, rectangles: usize) -> String {
    format!("{found} switches among {rectangles} rectangles")
}

pub(crate) fn alone_text(count: usize) -> String {
    format!("{count} picture fits these shadows")
}

/// 08_7's count of the face's squares a switch can change.
fn stuck_text(free: usize) -> String {
    format!("{free} {}", STUCK[0])
}

/// 08_8's words: every 4 by 4 picture, and how many are alone.
fn grids_text(side: usize, pictures: usize) -> String {
    format!("{side} by {side} grids: {} pictures", grouped(pictures))
}

fn common_text(alone: usize, tenths: usize) -> String {
    format!(
        "{} are the only picture with their shadows, about {tenths} in 10",
        grouped(alone)
    )
}

/// A whole number with commas between its thousands: "65,536".
fn grouped(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

/// The exact version of 09_4, kept on screen until it can be read
/// (`read_s`): Ryser's Theorem 3.1 for the video's pictures.
pub(crate) fn chain_claim() -> String {
    format!(
        "any two pictures of filled and empty squares on one grid with the same row counts and the same \
         column counts are joined by a finite chain of switches (Ryser, {RYSER_YEAR})"
    )
}

/// Where the `k`th switch's copy of a `rows` by `cols` picture stands (08_2).
fn switch_at(k: usize, rows: usize, cols: usize) -> [f64; 2] {
    let (w, h) = (SWITCH_CELL * cols as f64, SWITCH_CELL * rows as f64);
    let (row, col) = (k / SWITCH_COLS, k % SWITCH_COLS);
    [
        TEXT_X + w / 2.0 + SWITCH_PITCH[0] * col as f64,
        SWITCH_TOP - h / 2.0 - SWITCH_PITCH[1] * row as f64,
    ]
}

/// A small copy of `p` at `at`, switch `s`'s four corners ringed: one of its
/// switches, apart from the others (08_2). Detached.
fn switch_copy(stage: &mut Stage<'_>, p: &Picture, s: Switch, at: [f64; 2]) -> fmn::Result<Vec<Mob>> {
    let mut out = thumb(stage, p, v(at[0], at[1]), SWITCH_CELL, Palette::faint())?;
    let small = Board::new(at, SWITCH_CELL, p.rows(), p.cols());
    out.extend(s.corners().map(|(r, c)| small.ring(stage, r, c, palette::switch())));
    Ok(out)
}

/// A frame round the copy at `at` of a `rows` by `cols` picture, outside its
/// own: the switch being swapped on the grid (08_2). Detached.
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
        "08_the_only_picture"
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

        // 08_1: chapter 02's grid with the face's counts; the water's
        // picture, square by square
        let line = say(&kit, stage, "08_1")?;
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

        // 08_2: its twelve switches, counted as they are spoken, each on a
        // small copy of its own on the right; the first swapped on the grid,
        // its copy framed; every count stays, and a second picture fits
        let line = say(&kit, stage, "08_2")?;
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

        // 08_3: the second picture, its name and the switches stay into the
        // line's opening until the name can be read (F5), then go; on "from
        // before", once they have gone, the tree's shadows as taps and
        // drains; the method's real units build a picture, the last one
        // taking water back
        let line = say(&kit, stage, "08_3")?;
        let mut leaving: Vec<Mob> = std::mem::take(&mut squares).into_values().collect();
        leaving.extend(marked);
        leaving.extend(copies.into_iter().flatten());
        leaving.extend(&same_rows);
        leaving.extend(&same_cols);
        leaving.extend([second_label, how_many]);
        wait_until(stage, second_up + read_s(SECOND_PICTURE))?;
        fade_out_all(stage, &leaving, 0.4)?;
        line.wait_for(stage, "from before")?;
        let taps = counts(&kit, stage, &tree_shadows.rows, Palette::ink(), |r| tap_at(&board, r))?;
        let drains = counts(&kit, stage, &tree_shadows.cols, Palette::ink(), |c| {
            board.col_count_at(c)
        })?;
        let to_send = send_label(&kit, stage, &board)?;
        let to_drain = drain_label(&kit, stage, &board)?;
        let counter = kit.text_left(
            stage,
            &through_text(0, tree_shadows.row_total()),
            26.0,
            Palette::ink(),
            v(TEXT_X, THROUGH_Y),
        )?;
        fade_in_lagged(stage, &taps, 0.6, 0.3)?;
        fade_in_lagged(stage, &drains, 0.6, 0.3)?;
        fade_in_all(stage, &[to_send, to_drain, counter], 0.3)?;
        line.wait_for(stage, "The water builds")?;
        let mut water = Water::new(board, taps, drains, &tree_shadows);
        water.through = Some(counter);
        for (path, state) in tree_water.paths.iter().zip(&tree_states) {
            send_quick(&kit, stage, &mut water, &tree_shadows, path, state)?;
        }

        // 08_4: the counters become the tree's counts; the water built the
        // tree itself; the question, and the think time (nothing moves)
        let line = say(&kit, stage, "08_4")?;
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
        let mut going = vec![to_send, to_drain];
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

        // 08_5: what a switch needs, on the switch from 08_2: each of its
        // rows has a filled square where the other is empty
        let line = say(&kit, stage, "08_5")?;
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

        // 08_6: the tree's rows nest, shortest first, while the switch's copy
        // and words stay on the right until read; then they go, and the
        // nesting's words take their place; no switch among its rectangles
        let line = say(&kit, stage, "08_6")?;
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

        // 08_7: a switch is what stopped reasoning on the face: a small copy
        // of the face, the squares reasoning left undecided shaded behind its
        // squares, one of its switches' corners ringed
        let line = say(&kit, stage, "08_7")?;
        let face = model::face();
        let face_solve = model::line_solve(&face.shadows());
        let undecided: Vec<(usize, usize)> = (0..face.rows())
            .flat_map(|r| (0..face.cols()).map(move |c| (r, c)))
            .filter(|&(r, c)| face_solve.grid[r][c].is_none())
            .collect();
        let face_switches = model::switches(&face);
        let &face_first = face_switches
            .first()
            .ok_or_else(|| fail("the face has no switch".into()))?;
        if face_first.corners().iter().any(|rc| !undecided.contains(rc)) {
            return Err(fail("a corner of the face's switch is a square reasoning settled".into()));
        }
        fade_out_all(stage, &[nested, none], 0.3)?;
        let small = Board::new(SMALL_AT, SMALL_CELL, face.rows(), face.cols());
        let mut copy = small.lines(stage);
        for &(r, c) in &undecided {
            copy.push(small.shade(stage, r, c, Palette::dim(), 0.35)?);
        }
        copy.extend(place(stage, &small, &face)?.into_values());
        copy.extend(face_first.corners().map(|(r, c)| small.ring(stage, r, c, palette::switch())));
        let stuck_words = [stuck_text(undecided.len()), STUCK[1].to_string()];
        let mut stuck = Vec::with_capacity(stuck_words.len());
        for (text, y) in stuck_words.iter().zip(SMALL_TEXT_Y) {
            stuck.push(kit.text_left(stage, text, 24.0, Palette::ink(), v(SMALL_TEXT_X, y))?);
        }
        fade_in_all(stage, &copy, 0.5)?;
        line.wait_for(stage, "No reasoning can settle")?;
        fade_in_all(stage, &stuck, 0.4)?;
        let stuck_up = stage.scene().time().to_f64();

        // 08_8: how common a single answer is, on every 4 by 4 picture
        let line = say(&kit, stage, "08_8")?;
        let edges = facts::edges();
        let tenths = (10 * edges.alone + edges.pictures / 2) / edges.pictures;
        wait_until(stage, stuck_up + read_s(&stuck_words.join(" ")))?;
        let mut going = copy;
        going.extend(stuck);
        fade_out_all(stage, &going, 0.4)?;
        let grids = kit.text_left(
            stage,
            &grids_text(EDGE_SIDE, edges.pictures),
            24.0,
            Palette::ink(),
            v(TEXT_X, TOP_Y),
        )?;
        fade_in_all(stage, &[grids], 0.3)?;
        line.wait_for(stage, "only about one")?;
        let common = common_text(edges.alone, tenths);
        let common_line = kit.text_left(stage, &common, 22.0, Palette::ink(), v(TEXT_X, NEXT_Y))?;
        fade_in_all(stage, &[common_line], 0.4)?;
        let common_up = stage.scene().time().to_f64();
        line.wait_to_end(stage)?;
        wait_until(stage, common_up + read_s(&common))?;
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
            ("08_1", "builds one picture"),
            ("08_2", "This one has"),
            ("08_2", "Swap the corners"),
            ("08_2", "every count stays"),
            ("08_2", "A second picture"),
            ("08_3", "from before"),
            ("08_3", "The water builds"),
            ("08_4", "the tree itself"),
            ("08_5", "A switch needs"),
            ("08_5", "each have a filled square"),
            ("08_6", "of any two rows"),
            ("08_6", "no switch"),
            ("08_7", "No reasoning can settle"),
            ("08_8", "only about one"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The words on screen are the facts': the water's picture's switches,
    /// none among the tree's rectangles, the one picture that fits, the
    /// face's undecided squares, the four by four count. The card (chapter
    /// 09's) states Ryser's theorem within its scope, keeps to two whole
    /// lines at the band's width and to three at every width it may narrow
    /// to; it and the closing words are held long enough to read and under
    /// twelve seconds each.
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
        let questions = crate::chapters::same_shadows::QUESTIONS;
        let all_words = [questions[0], ANSWERS[0], questions[1], ANSWERS[1]].join(" ");
        for text in [claim.as_str(), all_words.as_str()] {
            assert!(read_s(text) < 12.0, "{text}");
        }
        assert_eq!(stuck_text(get("DT-F12.face.free")), "20 squares some switch changes");
        assert_eq!(
            grids_text(EDGE_SIDE, get("DT-X3.pictures")),
            "4 by 4 grids: 65,536 pictures"
        );
        assert_eq!(f.get("DT-X3.grid"), Some("4,4"));
        assert_eq!(
            common_text(get("DT-X3.alone"), get("DT-X3.alone.tenths")),
            "6,902 are the only picture with their shadows, about 1 in 10"
        );
        assert_eq!(grouped(0), "0");
        assert_eq!(grouped(65536), "65,536");
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
        // the switches' copies (08_2): under their count, right of the grid,
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
        for text in [stuck_text(20).as_str(), STUCK[1]] {
            assert!(ends(SMALL_TEXT_X, text, 24.0) < safe_x, "{text}");
        }
        // 08_8's words end inside
        assert!(ends(TEXT_X, &common_text(6902, 1), 22.0) < safe_x);
        assert!(ends(TEXT_X, &grids_text(4, 65536), 24.0) < safe_x);
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
