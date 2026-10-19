//! Chapter 01, the same shadows: a little face on a grid (01_1); how many
//! squares each row and each column holds, the picture's two shadows (01_2);
//! the picture hidden, could another cast the same shadows? (01_3, a
//! prediction); it can (01_4): the two differ at the corners of a rectangle,
//! filled and empty alternating (01_5), and swapping filled and empty there
//! keeps every count (01_6), a switch (01_7); sixty-four pictures cast these
//! shadows, each within three switches of the face (01_8); the face's row
//! counts with other column counts of the same total fit no picture (01_9,
//! 01_10); two questions (01_11).

use std::collections::HashMap;

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    Board, COUNT_SIZE, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, pause, say, segment, shift_all,
    swap, think, thumb,
};
use crate::facts::{self, RYSER_YEAR};
use crate::model::{self, Picture, Switch};
use crate::palette;

pub struct SameShadows {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const ROW_SHADOW: &str = "row shadow";
const COLUMN_SHADOW: &str = "column shadow";
const THE_FACE: &str = "the face";
const ANOTHER: &str = "another picture";
const SWITCH: &str = "switch";
const LOSES_AND_GAINS: &str = "−1 +1";
const NO_PICTURE: &str = "no picture fits";
/// The two questions 01_11 asks, as the screen shows them.
pub const QUESTIONS: [&str; 2] = ["Does any picture fit?", "Is it the only one?"];

/// The grid's pitch, and where a board stands: alone (01_1 to 01_3), the
/// face on the left and the other picture on the right (from 01_4).
const CELL: f64 = 0.62;
const ALONE: [f64; 2] = [-0.6, 0.35];
const LEFT: [f64; 2] = [-3.9, 0.35];
const RIGHT: [f64; 2] = [2.4, 0.35];
/// The think ring's centre (01_3).
const THINK_AT: [f64; 2] = [4.4, 0.6];
/// The gallery (01_8): its centre, columns, pitch, and a thumbnail's square.
const GALLERY: [f64; 2] = [3.65, 0.35];
const GALLERY_COLS: usize = 8;
const PITCH: f64 = 0.62;
pub(crate) const THUMB_CELL: f64 = 0.09;
/// Where 01_9 to 01_11 write, right of the face's board.
const TEXT_X: f64 = 0.9;
/// A "−1 +1" tag's offset from a row's count, and from a column's.
const ROW_TAG_DX: f64 = 0.35;
const COL_TAG_DY: f64 = -0.4;
/// The word "switch", right of the other picture's counts.
const SWITCH_DX: f64 = 1.3;

/// The exact version of 01_8's walk, kept on screen: Ryser's theorem.
fn chain_claim() -> String {
    format!(
        "any two pictures with the same row and column counts are joined by a chain of switches (H. J. Ryser, {RYSER_YEAR})"
    )
}

/// The gallery's caption, from its count and the farthest walk.
fn gallery_label(count: usize, farthest: usize) -> String {
    format!("{count} pictures, each within {farthest} switches of the face")
}

/// Where the gallery's `k`th of `count` thumbnails stands: rows of
/// [`GALLERY_COLS`], centred on [`GALLERY`] (chapter 02 shows the same
/// gallery).
pub(crate) fn gallery_at(k: usize, count: usize) -> Vec3 {
    let rows = count.div_ceil(GALLERY_COLS);
    let (row, col) = (k / GALLERY_COLS, k % GALLERY_COLS);
    v(
        GALLERY[0] + (col as f64 - (GALLERY_COLS as f64 - 1.0) / 2.0) * PITCH,
        GALLERY[1] + ((rows as f64 - 1.0) / 2.0 - row as f64) * PITCH,
    )
}

/// Counts as text, one per value, each at `at(i)`. Detached.
fn counts(
    kit: &Kit,
    stage: &mut Stage<'_>,
    values: &[usize],
    color: Srgb,
    at: impl Fn(usize) -> Vec3,
) -> fmn::Result<Vec<Mob>> {
    values
        .iter()
        .enumerate()
        .map(|(i, n)| kit.text(stage, &n.to_string(), COUNT_SIZE, color, at(i)))
        .collect()
}

/// The filled squares of `p` on `board`, by place. Detached.
fn place(stage: &mut Stage<'_>, board: &Board, p: &Picture) -> fmn::Result<HashMap<(usize, usize), Mob>> {
    p.filled()
        .into_iter()
        .map(|(r, c)| Ok(((r, c), board.square(stage, r, c, palette::filled())?)))
        .collect()
}

/// A switch marked on `board`: its rectangle, and a ring on each corner in
/// order around it. Detached.
fn marks(stage: &mut Stage<'_>, board: &Board, s: Switch) -> (Mob, [Mob; 4]) {
    let rect = board.rectangle(stage, s, palette::switch());
    let rings = s.corners().map(|(r, c)| board.ring(stage, r, c, palette::switch()));
    (rect, rings)
}

/// Run switch `s` on picture `p`, drawn on `board` as `squares`: its filled
/// corners fade as its empty corners fill. Returns the new picture.
fn run_switch(
    stage: &mut Stage<'_>,
    board: &Board,
    squares: &mut HashMap<(usize, usize), Mob>,
    p: &Picture,
    s: Switch,
    rt: f64,
) -> fmn::Result<Picture> {
    let sides = s
        .sides(p)
        .ok_or_else(|| fail(format!("{s:?} is not a switch of {}", p.pattern())))?;
    let mut old = Vec::with_capacity(2);
    for rc in sides.emptied {
        old.push(
            squares
                .remove(&rc)
                .ok_or_else(|| fail(format!("no square is drawn at {rc:?}")))?,
        );
    }
    let mut new = Vec::with_capacity(2);
    for (r, c) in sides.filled {
        let m = board.square(stage, r, c, palette::filled())?;
        squares.insert((r, c), m);
        new.push(m);
    }
    swap(stage, &old, &new, rt)?;
    s.apply(p)
        .ok_or_else(|| fail(format!("{s:?} did not apply to {}", p.pattern())))
}

impl SceneConstruct for SameShadows {
    fn name(&self) -> &str {
        "01_same_shadows"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let face = model::face();
        let twin = facts::twin().map_err(fail)?;
        if twin.shadows() != face.shadows() {
            return Err(fail(format!("{} does not cast the face's shadows", twin.pattern())));
        }

        // 01_1: the face, square by square
        let line = say(&kit, stage, "01_1")?;
        let mut board = Board::new(ALONE, CELL, face.rows(), face.cols());
        let grid = board.lines(stage);
        fade_in_all(stage, &grid, 0.6)?;
        line.wait_for(stage, "a little face")?;
        let squares = place(stage, &board, &face)?;
        let ordered: Vec<Mob> = face.filled().iter().map(|rc| squares[rc]).collect();
        fade_in_lagged(stage, &ordered, 1.2, 0.3)?;

        // 01_2: a beam along each row, then down each column, leaves its count
        let line = say(&kit, stage, "01_2")?;
        let row_counts = counts(&kit, stage, &face.row_counts(), Palette::ink(), |r| {
            board.row_count_at(r)
        })?;
        let mut col_counts = counts(&kit, stage, &face.col_counts(), Palette::ink(), |c| {
            board.col_count_at(c)
        })?;
        let row_beams: Vec<Mob> = (0..board.rows)
            .map(|r| {
                let y = board.at(r, 0)[1];
                segment(
                    stage,
                    v(board.left() - 0.6, y),
                    v(board.right() + 0.2, y),
                    palette::beam(),
                    3.0,
                )
            })
            .collect();
        fade_in_lagged(stage, &row_beams, 1.0, 0.3)?;
        swap(stage, &row_beams, &row_counts, 0.4)?;
        line.wait_for(stage, "and in each column")?;
        let col_beams: Vec<Mob> = (0..board.cols)
            .map(|c| {
                let x = board.at(0, c)[0];
                segment(
                    stage,
                    v(x, board.top() + 0.6),
                    v(x, board.bottom() - 0.2),
                    palette::beam(),
                    3.0,
                )
            })
            .collect();
        fade_in_lagged(stage, &col_beams, 1.0, 0.3)?;
        swap(stage, &col_beams, &col_counts, 0.4)?;
        line.wait_for(stage, "two shadows")?;
        let labels = [
            kit.text_left(
                stage,
                ROW_SHADOW,
                22.0,
                Palette::dim(),
                v(board.right() + 0.25, board.top() + 0.4),
            )?,
            kit.text(
                stage,
                COLUMN_SHADOW,
                22.0,
                Palette::dim(),
                v(board.left() - 1.3, board.col_count_at(0)[1]),
            )?,
        ];
        fade_in_all(stage, &labels, 0.4)?;

        // 01_3: the picture hidden, the shadows kept; the question, and the
        // think time (the other picture stays hidden until "It can")
        let line = say(&kit, stage, "01_3")?;
        line.wait_for(stage, "Hide the picture")?;
        fade_out_all(stage, &ordered, 0.6)?;
        let q = think(&kit, stage, v(THINK_AT[0], THINK_AT[1]))?;

        // 01_4: the face back on the left, the other picture on the right,
        // its counts matching the face's one by one
        let line = say(&kit, stage, "01_4")?;
        fade_out_all(stage, &[q, labels[0], labels[1]], 0.3)?;
        let mut moving = grid.clone();
        moving.extend(&row_counts);
        moving.extend(&col_counts);
        shift_all(stage, &moving, LEFT[0] - ALONE[0], LEFT[1] - ALONE[1], 0.7)?;
        board = board.moved(LEFT);
        let mut squares = place(stage, &board, &face)?;
        let right = board.moved(RIGHT);
        let right_grid = right.lines(stage);
        let mut right_squares = place(stage, &right, &twin)?;
        let face_name = kit.text(
            stage,
            THE_FACE,
            24.0,
            Palette::dim(),
            v(board.centre[0], board.top() + 0.4),
        )?;
        let other_name = kit.text(
            stage,
            ANOTHER,
            24.0,
            Palette::dim(),
            v(right.centre[0], right.top() + 0.4),
        )?;
        let mut arriving: Vec<Mob> = squares.values().copied().collect();
        arriving.extend(&right_grid);
        arriving.extend(right_squares.values().copied());
        arriving.extend([face_name, other_name]);
        fade_in_all(stage, &arriving, 0.5)?;
        line.wait_for(stage, "Every row count")?;
        let twin_rows = counts(&kit, stage, &twin.row_counts(), palette::matches(), |r| {
            right.row_count_at(r)
        })?;
        flash_all(stage, &row_counts, 0.4)?;
        fade_in_lagged(stage, &twin_rows, 0.8, 0.3)?;
        line.wait_for(stage, "every column count")?;
        let twin_cols = counts(&kit, stage, &twin.col_counts(), palette::matches(), |c| {
            right.col_count_at(c)
        })?;
        flash_all(stage, &col_counts, 0.4)?;
        fade_in_lagged(stage, &twin_cols, 0.8, 0.3)?;

        // 01_5: the four squares where they differ, the corners of a
        // rectangle; around it, filled and empty alternate
        let line = say(&kit, stage, "01_5")?;
        let s = model::OPENING;
        let mut corners = s.corners().to_vec();
        corners.sort_unstable();
        if face.differences(&twin) != corners {
            return Err(fail(
                "the face and the other picture differ off the switch's corners".into(),
            ));
        }
        line.wait_for(stage, "four squares")?;
        let (face_rect, face_rings) = marks(stage, &board, s);
        let (twin_rect, twin_rings) = marks(stage, &right, s);
        let mut marked = vec![face_rect, twin_rect];
        marked.extend(face_rings);
        marked.extend(twin_rings);
        fade_in_all(stage, &marked, 0.5)?;
        line.wait_for(stage, "alternating")?;
        for (a, b) in face_rings.iter().zip(&twin_rings) {
            flash_all(stage, &[*a, *b], 0.3)?;
        }

        // 01_6: on the other picture, filled and empty swap at the corners
        // and it becomes the face; each of the switch's rows and columns
        // loses one square and gains one
        let line = say(&kit, stage, "01_6")?;
        let switched = run_switch(stage, &right, &mut right_squares, &twin, s, 0.6)?;
        if switched != face {
            return Err(fail(format!("the switch made {}, not the face", switched.pattern())));
        }
        line.wait_for(stage, "loses one square")?;
        let mut tags = Vec::with_capacity(4);
        let mut touched = Vec::with_capacity(4);
        for r in s.rows {
            let at = right.row_count_at(r);
            tags.push(kit.text_left(
                stage,
                LOSES_AND_GAINS,
                20.0,
                Palette::dim(),
                v(at[0] + ROW_TAG_DX, at[1]),
            )?);
            touched.push(twin_rows[r]);
        }
        for c in s.cols {
            let at = right.col_count_at(c);
            tags.push(kit.text(
                stage,
                LOSES_AND_GAINS,
                20.0,
                Palette::dim(),
                v(at[0], at[1] + COL_TAG_DY),
            )?);
            touched.push(twin_cols[c]);
        }
        fade_in_all(stage, &tags, 0.3)?;
        flash_all(stage, &touched, 0.6)?;

        // 01_7: the move's name; every count, unchanged
        let line = say(&kit, stage, "01_7")?;
        fade_out_all(stage, &tags, 0.3)?;
        let middle = (right.at(s.rows[0], 0)[1] + right.at(s.rows[1], 0)[1]) / 2.0;
        let word = kit.text(
            stage,
            SWITCH,
            26.0,
            palette::switch(),
            v(right.right() + SWITCH_DX, middle),
        )?;
        fade_in_all(stage, &[word], 0.4)?;
        line.wait_for(stage, "neither shadow")?;
        let mut every_count = row_counts.clone();
        every_count.extend(&col_counts);
        every_count.extend(&twin_rows);
        every_count.extend(&twin_cols);
        flash_all(stage, &every_count, 0.7)?;

        // 01_8: the other picture goes; a gallery of every picture with
        // these shadows fills in waves, nearest the face first, landing on
        // the count; then the face takes the walk to a farthest one
        let line = say(&kit, stage, "01_8")?;
        let mut leaving = right_grid;
        leaving.extend(right_squares.values().copied());
        leaving.extend(&twin_rows);
        leaving.extend(&twin_cols);
        leaving.extend([other_name, face_name, word]);
        leaving.extend(marked);
        fade_out_all(stage, &leaving, 0.4)?;
        let gallery = facts::gallery();
        let mut thumbs: Vec<Vec<Mob>> = Vec::with_capacity(gallery.len());
        let mut index: HashMap<Picture, usize> = HashMap::new();
        for (k, (p, d)) in gallery.iter().enumerate() {
            let frame = if *d == 0 { Palette::ink() } else { Palette::faint() };
            thumbs.push(thumb(stage, p, gallery_at(k, gallery.len()), THUMB_CELL, frame)?);
            index.insert(p.clone(), k);
        }
        let farthest = gallery.iter().map(|(_, d)| *d).max().unwrap_or(0);
        let weights: Vec<f64> = (0..=farthest)
            .map(|d| 0.1 + gallery.iter().filter(|(_, e)| *e == d).count() as f64 / gallery.len() as f64)
            .collect();
        let weight: f64 = weights.iter().sum();
        let span = (line.at("Sixty-four")? - stage.scene().time().to_f64() - 0.2).max(1.6);
        for (d, w) in weights.iter().enumerate() {
            let wave: Vec<Mob> = thumbs
                .iter()
                .zip(&gallery)
                .filter(|(_, (_, e))| *e == d)
                .flat_map(|(t, _)| t.iter().copied())
                .collect();
            fade_in_lagged(stage, &wave, span * w / weight, 0.25)?;
        }
        line.wait_for(stage, "Sixty-four")?;
        let top = gallery_at(0, gallery.len())[1] + 0.65;
        let count = kit.text(
            stage,
            &format!("{} pictures", gallery.len()),
            26.0,
            Palette::ink(),
            v(GALLERY[0], top),
        )?;
        fade_in_all(stage, &[count], 0.3)?;
        line.wait_for(stage, "at most three")?;
        let caption = kit.text(
            stage,
            &gallery_label(gallery.len(), farthest),
            24.0,
            Palette::ink(),
            v(GALLERY[0], top),
        )?;
        swap(stage, &[count], &[caption], 0.3)?;
        let (chain, end) = facts::walk().map_err(fail)?;
        if chain.len() != farthest {
            return Err(fail(format!("the walk takes {} switches, not {farthest}", chain.len())));
        }
        let mut at = face.clone();
        for &step in &chain {
            let rect = board.rectangle(stage, step, palette::switch());
            fade_in_all(stage, &[rect], 0.15)?;
            at = run_switch(stage, &board, &mut squares, &at, step, 0.35)?;
            let k = *index
                .get(&at)
                .ok_or_else(|| fail(format!("{} is not in the gallery", at.pattern())))?;
            flash_all(stage, &thumbs[k], 0.35)?;
            fade_out_all(stage, &[rect], 0.15)?;
        }
        if at != end {
            return Err(fail(format!(
                "the walk ended at {}, not {}",
                at.pattern(),
                end.pattern()
            )));
        }
        let card = precisely(&kit, stage, &chain_claim())?;
        fade_in_all(stage, &[card], 0.4)?;

        // 01_9: the face's rows, other column counts with the same total;
        // no picture fits
        let line = say(&kit, stage, "01_9")?;
        let mut leaving: Vec<Mob> = thumbs.into_iter().flatten().collect();
        leaving.extend([caption, card]);
        leaving.extend(std::mem::take(&mut squares).into_values());
        fade_out_all(stage, &leaving, 0.5)?;
        let no_fit = facts::no_fit();
        if no_fit.rows != face.row_counts() || no_fit.row_total() != no_fit.col_total() {
            return Err(fail(
                "the no-fit shadows are not the face's rows with the same total".into(),
            ));
        }
        let new_cols = counts(&kit, stage, &no_fit.cols, Palette::ink(), |c| board.col_count_at(c))?;
        swap(stage, &col_counts, &new_cols, 0.6)?;
        col_counts = new_cols;
        line.wait_for(stage, "keep their total")?;
        let totals = kit.text_left(
            stage,
            &format!("both shadows add up to {}", no_fit.row_total()),
            26.0,
            Palette::dim(),
            v(TEXT_X, 1.6),
        )?;
        fade_in_all(stage, &[totals], 0.4)?;
        line.wait_for(stage, "no picture fits")?;
        let verdict = kit.text_left(stage, NO_PICTURE, 30.0, palette::impossible(), v(TEXT_X, 0.9))?;
        fade_in_all(stage, &[verdict], 0.4)?;

        // 01_10: the first column asks for a square in every row; the
        // middle row has none to give
        let line = say(&kit, stage, "01_10")?;
        let (set, asks, _) = facts::no_fit_bottleneck().map_err(fail)?;
        let &[col] = set.as_slice() else {
            return Err(fail(format!("the bottleneck is {set:?}, not one column")));
        };
        if asks != board.rows {
            return Err(fail(format!("the column asks for {asks}, not a square in every row")));
        }
        let empty = no_fit
            .rows
            .iter()
            .position(|&r| r == 0)
            .ok_or_else(|| fail("no row is empty".into()))?;
        if 2 * empty + 1 != board.rows {
            return Err(fail(format!("the empty row {empty} is not the middle one")));
        }
        let red_ask = kit.text(
            stage,
            &asks.to_string(),
            COUNT_SIZE,
            palette::impossible(),
            board.col_count_at(col),
        )?;
        swap(stage, &[col_counts[col]], &[red_ask], 0.3)?;
        let slots: Vec<Mob> = (0..board.rows)
            .map(|r| board.ring(stage, r, col, Palette::ink()))
            .collect();
        fade_in_lagged(stage, &slots, 0.8, 0.35)?;
        line.wait_for(stage, "the middle row")?;
        let red_zero = kit.text(
            stage,
            &no_fit.rows[empty].to_string(),
            COUNT_SIZE,
            palette::impossible(),
            board.row_count_at(empty),
        )?;
        swap(stage, &[row_counts[empty]], &[red_zero], 0.3)?;
        let cross = board.cross(stage, empty, col, palette::impossible());
        fade_in_all(stage, &cross, 0.3)?;

        // 01_11: the two questions, as they are asked
        let line = say(&kit, stage, "01_11")?;
        line.wait_for(stage, "Does any picture")?;
        let first = kit.text_left(stage, QUESTIONS[0], 32.0, Palette::ink(), v(TEXT_X, -0.3))?;
        fade_in_all(stage, &[first], 0.4)?;
        line.wait_for(stage, "And is it")?;
        let second = kit.text_left(stage, QUESTIONS[1], 32.0, Palette::ink(), v(TEXT_X, -1.1))?;
        fade_in_all(stage, &[second], 0.4)?;
        line.wait_to_end(stage)?;
        pause(stage, 1.2)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::*;

    /// Every phrase the scene waits for is in its line of script.tsv (a
    /// changed line fails here, not in the middle of a render).
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("01_1", "a little face"),
            ("01_2", "and in each column"),
            ("01_2", "two shadows"),
            ("01_3", "Hide the picture"),
            ("01_4", "Every row count"),
            ("01_4", "every column count"),
            ("01_5", "four squares"),
            ("01_5", "alternating"),
            ("01_6", "loses one square"),
            ("01_7", "neither shadow"),
            ("01_8", "Sixty-four"),
            ("01_8", "at most three"),
            ("01_9", "keep their total"),
            ("01_9", "no picture fits"),
            ("01_10", "the middle row"),
            ("01_11", "Does any picture"),
            ("01_11", "And is it"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// What the screen asks at 01_11 is what the voice asks.
    #[test]
    fn the_questions_on_screen_are_the_spoken_ones() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let spoken = script.text("01_11").unwrap().to_lowercase();
        for q in QUESTIONS {
            assert!(q.ends_with('?'), "{q}");
            assert!(spoken.contains(&q.to_lowercase()), "01_11 does not ask {q:?}");
        }
    }

    /// The card names the theorem's year from the facts and fits in two
    /// lines; the caption says what the gallery shows.
    #[test]
    fn the_words_on_screen_are_built_from_the_facts() {
        let claim = chain_claim();
        assert!(claim.contains("Ryser, 1957"), "{claim}");
        assert!(explainer_kit::animatic::wrap(&claim, 92, 3).len() <= 2, "{claim}");
        let f = facts::facts().unwrap();
        let caption = gallery_label(
            f.get("DT-F3.fits").unwrap().parse().unwrap(),
            f.get("DT-F3.farthest").unwrap().parse().unwrap(),
        );
        assert_eq!(caption, "64 pictures, each within 3 switches of the face");
    }

    /// The chapter's layout keeps its pieces apart and inside the
    /// title-safe area (text 0.25 in from the frame's edge).
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        let face = model::face();
        let (rows, cols) = (face.rows(), face.cols());
        let alone = Board::new(ALONE, CELL, rows, cols);
        let left = Board::new(LEFT, CELL, rows, cols);
        let right = Board::new(RIGHT, CELL, rows, cols);
        let safe_x = FRAME_X_RADIUS - 0.25;
        let safe_y = FRAME_Y_RADIUS - 0.25;
        // 01_2's labels, and 01_3's think ring and its note, clear the board
        assert!(alone.left() - 1.3 - 1.0 > -safe_x, "the column shadow's label");
        assert!(alone.top() + 0.4 < safe_y);
        assert!(THINK_AT[0] - 0.62 > alone.row_count_at(0)[0] + 0.3);
        assert!(THINK_AT[0] + 1.2 < safe_x, "the think note");
        // the face's counts clear the other picture; the other picture's
        // tags and the word "switch" stay inside
        assert!(left.row_count_at(0)[0] + 0.3 < right.left());
        assert!(right.row_count_at(0)[0] + ROW_TAG_DX + 0.8 < safe_x);
        assert!(right.right() + SWITCH_DX + 0.5 < safe_x);
        assert!(right.col_count_at(0)[1] + COL_TAG_DY - 0.15 > -safe_y);
        // the gallery: right of the face's counts, inside the frame, its
        // caption under the top edge and its last row over a two-line
        // lower-third (top at about 1.22 above the frame's bottom)
        let count = 64;
        let half = THUMB_CELL * cols as f64 / 2.0 + 0.03;
        let (first, last) = (gallery_at(0, count), gallery_at(count - 1, count));
        assert!(first[0] - half > left.row_count_at(0)[0] + 0.5);
        assert!(last[0] + half < safe_x);
        assert!(first[1] + 0.65 + 0.2 < safe_y, "the caption");
        assert!(last[1] - half > -FRAME_Y_RADIUS + 1.25, "over the lower-third");
        assert!(PITCH > 2.0 * half + 0.1, "thumbnails apart");
        // 01_9 to 01_11's words stand right of the board's counts
        assert!(TEXT_X > left.row_count_at(0)[0] + 0.5);
        assert!(
            left.col_count_at(0)[1] - 0.2 > -FRAME_Y_RADIUS + 1.25,
            "the counts over the card"
        );
    }
}
