//! Chapter 05, a third shadow: back to the face's sixty-four pictures, and
//! the counts along the diagonals from lower left to upper right (05_1); a
//! switch keeps every row and column count but always changes a diagonal
//! count, and the opening switch takes a square from two diagonals and
//! gives both to a third (05_2); so none of the fourteen pictures one
//! switch from the face can pass; how many of all sixty-four do? (05_3, a
//! prediction); two: the face and one other picture (05_4), six squares
//! apart, every row, column and diagonal through them losing one and
//! gaining one (05_5); here even three shadows lie (05_6); the two
//! questions kept apart, inside a bounded model (05_7).
//!
//! The pictures are chapter 01's 64 (`facts::gallery`); the third shadow is
//! `model::diagonal_counts`; the survivors are `facts::diagonal_survivors`.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    Board, COUNT_SIZE, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, pause, say, segment, swap, think,
    thumb, wait_until,
};
use super::same_shadows::{GALLERY, QUESTIONS, THUMB_CELL, counts, gallery_at, place, run_switch};
use super::stops::read_s;
use super::water::{CELL, tap_at};
use crate::facts;
use crate::model::{self, Picture};
use crate::palette;

pub struct AThirdShadow {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const THE_FACE: &str = "the face";
const ANOTHER: &str = "another picture";
const DIAGONAL_SHADOW: &str = "diagonal shadow";
const THROUGH_THEM: &str = "each row, column and diagonal through them: −1 +1";
const FIRST_ANSWER: &str = "yes: the face, and one more";

/// The face on the left, the other survivor on the right (from 05_4).
const LEFT: [f64; 2] = [-3.6, 0.35];
const RIGHT: [f64; 2] = [2.6, 0.35];
/// The think ring's centre (05_3), between the face's counts and the gallery.
const THINK_AT: [f64; 2] = [-0.25, 0.45];
/// A picture's name above its grid, over its diagonal counts.
const NAME_DY: f64 = 0.85;
/// The words above both pictures (05_6) and under them (05_5).
const TOP_Y: f64 = 3.3;
/// The result of the test, kept over the other picture from 05_4 until
/// 05_6's words take its place.
const RESULT_Y: f64 = 3.35;
const BOTTOM_Y: f64 = -2.45;
/// The closing questions and answers (05_7), and where they start.
const CLOSING_X: f64 = -1.0;
const CLOSING_Y: [f64; 4] = [2.0, 1.45, 0.6, 0.05];

fn gallery_label(count: usize) -> String {
    format!("{count} pictures with the face's rows and columns")
}

fn passed_label(passed: usize, count: usize) -> String {
    format!("{passed} of {count} cast the face's diagonal shadow too")
}

fn shadows_label(shadows: usize, pictures: usize) -> String {
    format!("{shadows} shadows, {pictures} pictures")
}

fn second_answer(pictures: usize, shadows: usize) -> String {
    format!("no: {pictures} pictures cast all {shadows} shadows")
}

/// The exact bounds of the video's model, kept on screen (05_7).
fn model_claim(rows: usize, cols: usize) -> String {
    format!(
        "everything in this video is about {rows} by {cols} pictures of filled and empty squares, counted \
         exactly along rows, columns and here one diagonal direction"
    )
}

/// Diagonal `k`'s squares on `board`, lower left to upper right: those whose
/// row and column add to `k`.
fn diagonal_squares(board: &Board, k: usize) -> Vec<(usize, usize)> {
    (0..board.rows)
        .rev()
        .filter_map(|r| k.checked_sub(r).filter(|&c| c < board.cols).map(|c| (r, c)))
        .collect()
}

/// Where diagonal `k`'s count stands: one square on past its upper-right
/// end, along the diagonal (above the grid, or right of it).
fn diagonal_count_at(board: &Board, k: usize) -> Vec3 {
    let top = k.saturating_sub(board.cols - 1);
    let p = board.at(top, k - top);
    v(p[0] + board.cell, p[1] + board.cell)
}

/// A beam along diagonal `k`, from before its lower-left square to its
/// upper-right one. Detached.
fn diagonal_beam(stage: &mut Stage<'_>, board: &Board, k: usize) -> fmn::Result<Mob> {
    let squares = diagonal_squares(board, k);
    let (Some(&(r0, c0)), Some(&(r1, c1))) = (squares.first(), squares.last()) else {
        return Err(fail(format!("diagonal {k} has no square")));
    };
    let (a, b) = (board.at(r0, c0), board.at(r1, c1));
    let (before, after) = (board.cell * 0.6, board.cell * 0.3);
    Ok(segment(
        stage,
        v(a[0] - before, a[1] - before),
        v(b[0] + after, b[1] + after),
        palette::beam(),
        3.0,
    ))
}

/// A picture's three shadows as counts on `board`, in `color`: rows left of
/// it, columns under it, diagonals past their upper-right ends. Detached.
fn three_shadows(
    kit: &Kit,
    stage: &mut Stage<'_>,
    board: &Board,
    p: &Picture,
    color: Srgb,
) -> fmn::Result<[Vec<Mob>; 3]> {
    Ok([
        counts(kit, stage, &p.row_counts(), color, |r| tap_at(board, r))?,
        counts(kit, stage, &p.col_counts(), color, |c| board.col_count_at(c))?,
        counts(kit, stage, &model::diagonal_counts(p), color, |k| {
            diagonal_count_at(board, k)
        })?,
    ])
}

impl SceneConstruct for AThirdShadow {
    fn name(&self) -> &str {
        "05_a_third_shadow"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let face = model::face();
        let face_diagonal = model::diagonal_counts(&face);
        let gallery = facts::gallery();
        let survivors = facts::diagonal_survivors();
        let others: Vec<&Picture> = survivors.iter().filter(|p| **p != face).collect();
        let &[other] = others.as_slice() else {
            return Err(fail(format!(
                "{} pictures besides the face cast its three shadows",
                others.len()
            )));
        };
        if other.shadows() != face.shadows() || model::diagonal_counts(other) != face_diagonal {
            return Err(fail(format!("{} does not cast the face's shadows", other.pattern())));
        }
        let changes = facts::line_changes(&face, other);
        if changes.is_empty() || changes.iter().any(|&lg| lg != (1, 1)) {
            return Err(fail(format!(
                "the lines through the squares where they differ change unevenly: {changes:?}"
            )));
        }
        let left = Board::new(LEFT, CELL, face.rows(), face.cols());
        let right = Board::new(RIGHT, CELL, face.rows(), face.cols());

        // 05_1: the face and its two shadows; a beam along each diagonal
        // leaves its count
        let line = say(&kit, stage, "05_1")?;
        let grid = left.lines(stage);
        let mut squares = place(stage, &left, &face)?;
        let [face_rows, face_cols, mut face_diagonals] = three_shadows(&kit, stage, &left, &face, Palette::ink())?;
        let face_name = kit.text(
            stage,
            THE_FACE,
            24.0,
            Palette::dim(),
            v(left.centre[0], left.top() + NAME_DY),
        )?;
        let mut arriving = grid.clone();
        arriving.extend(squares.values().copied());
        arriving.extend(&face_rows);
        arriving.extend(&face_cols);
        arriving.push(face_name);
        fade_in_all(stage, &arriving, 0.6)?;
        line.wait_for(stage, "along each diagonal")?;
        let beams: Vec<Mob> = (0..face_diagonal.len())
            .map(|k| diagonal_beam(stage, &left, k))
            .collect::<fmn::Result<_>>()?;
        fade_in_lagged(stage, &beams, 1.2, 0.3)?;
        swap(stage, &beams, &face_diagonals, 0.4)?;
        let label = kit.text(
            stage,
            DIAGONAL_SHADOW,
            22.0,
            Palette::dim(),
            v(left.right() + left.cell * 0.5, left.top() + NAME_DY),
        )?;
        fade_in_all(stage, &[label], 0.3)?;

        // 05_2: the opening switch keeps the rows and columns, and changes
        // three diagonal counts
        let line = say(&kit, stage, "05_2")?;
        line.wait_for(stage, "A switch keeps")?;
        let s = model::OPENING;
        let mut marks = vec![left.rectangle(stage, s, palette::switch())];
        marks.extend(s.corners().map(|(r, c)| left.ring(stage, r, c, palette::switch())));
        fade_in_all(stage, &marks, 0.4)?;
        let twin = run_switch(stage, &left, &mut squares, &face, s, 0.6)?;
        if twin != facts::twin().map_err(fail)? {
            return Err(fail(format!("the opening switch made {}", twin.pattern())));
        }
        let mut kept = face_rows.clone();
        kept.extend(&face_cols);
        flash_all(stage, &kept, 0.5)?;
        line.wait_for(stage, "always changes")?;
        let twin_diagonal = model::diagonal_counts(&twin);
        let changed: Vec<usize> = (0..face_diagonal.len())
            .filter(|&k| twin_diagonal[k] != face_diagonal[k])
            .collect();
        let mut red = Vec::with_capacity(changed.len());
        for &k in &changed {
            red.push(kit.text(
                stage,
                &twin_diagonal[k].to_string(),
                COUNT_SIZE,
                palette::impossible(),
                diagonal_count_at(&left, k),
            )?);
        }
        let old: Vec<Mob> = changed.iter().map(|&k| face_diagonals[k]).collect();
        swap(stage, &old, &red, 0.4)?;
        line.wait_for(stage, "This one takes")?;
        flash_all(stage, &red, 0.6)?;

        // 05_3: the switch undone; the 64 on the right; none of the
        // fourteen one switch away can pass; the question, the think time
        let line = say(&kit, stage, "05_3")?;
        let back = run_switch(stage, &left, &mut squares, &twin, s, 0.5)?;
        if back != face {
            return Err(fail(format!("undoing the switch made {}", back.pattern())));
        }
        let restored: Vec<Mob> = changed
            .iter()
            .map(|&k| {
                kit.text(
                    stage,
                    &face_diagonal[k].to_string(),
                    COUNT_SIZE,
                    Palette::ink(),
                    diagonal_count_at(&left, k),
                )
            })
            .collect::<fmn::Result<_>>()?;
        swap(stage, &red, &restored, 0.4)?;
        for (&k, &m) in changed.iter().zip(&restored) {
            face_diagonals[k] = m;
        }
        fade_out_all(stage, &marks, 0.3)?;
        let mut thumbs: Vec<Vec<Mob>> = Vec::with_capacity(gallery.len());
        for (k, (p, d)) in gallery.iter().enumerate() {
            let frame = if *d == 0 { Palette::ink() } else { Palette::faint() };
            thumbs.push(thumb(stage, p, gallery_at(k, gallery.len()), THUMB_CELL, frame)?);
        }
        let top = gallery_at(0, gallery.len())[1] + 0.65;
        let caption = kit.text(
            stage,
            &gallery_label(gallery.len()),
            22.0,
            Palette::ink(),
            v(GALLERY[0], top),
        )?;
        let all: Vec<Mob> = thumbs.iter().flatten().copied().collect();
        fade_in_lagged(stage, &all, 1.0, 0.2)?;
        fade_in_all(stage, &[caption], 0.3)?;
        line.wait_for(stage, "none of the fourteen")?;
        let near: Vec<Mob> = thumbs
            .iter()
            .zip(&gallery)
            .filter(|(_, (_, d))| *d == 1)
            .flat_map(|(t, _)| t.iter().copied())
            .collect();
        fade_out_all(stage, &near, 0.6)?;
        line.wait_for(stage, "all sixty-four")?;
        flash_all(stage, &face_diagonals, 0.6)?;
        let q = think(&kit, stage, v(THINK_AT[0], THINK_AT[1]))?;

        // 05_4: two pass; the other forty-eight that fail go; the other
        // survivor on the right with all three shadows green
        let line = say(&kit, stage, "05_4")?;
        fade_out_all(stage, &[q], 0.3)?;
        line.wait_for(stage, "Sixty-two fail")?;
        let failing: Vec<Mob> = thumbs
            .iter()
            .zip(&gallery)
            .filter(|(_, (p, d))| *d != 1 && !survivors.contains(p))
            .flat_map(|(t, _)| t.iter().copied())
            .collect();
        fade_out_all(stage, &failing, 0.8)?;
        let passed = kit.text(
            stage,
            &passed_label(survivors.len(), gallery.len()),
            22.0,
            Palette::ink(),
            v(GALLERY[0], top),
        )?;
        swap(stage, &[caption], &[passed], 0.3)?;
        line.wait_for(stage, "one other picture")?;
        let mut leaving: Vec<Mob> = thumbs
            .iter()
            .zip(&gallery)
            .filter(|(_, (p, _))| survivors.contains(p))
            .flat_map(|(t, _)| t.iter().copied())
            .collect();
        leaving.push(passed);
        // Clear the old caption before revealing its replacement so the
        // words cannot overlap. Keep the original 0.4-second total.
        let result = kit.text(
            stage,
            &passed_label(survivors.len(), gallery.len()),
            22.0,
            Palette::ink(),
            v(RIGHT[0], RESULT_Y),
        )?;
        fade_out_all(stage, &leaving, 0.2)?;
        fade_in_all(stage, &[result], 0.2)?;
        let mut on_right = right.lines(stage);
        let other_squares = place(stage, &right, other)?;
        on_right.extend(other_squares.values().copied());
        let [other_rows, other_cols, other_diagonals] = three_shadows(&kit, stage, &right, other, palette::matches())?;
        on_right.extend(&other_rows);
        on_right.extend(&other_cols);
        on_right.extend(&other_diagonals);
        on_right.push(kit.text(
            stage,
            ANOTHER,
            24.0,
            Palette::dim(),
            v(right.centre[0], right.top() + NAME_DY),
        )?);
        fade_in_all(stage, &on_right, 0.6)?;

        // 05_5: the six squares where they differ; every line through them
        // loses one and gains one; every count flashes
        let line = say(&kit, stage, "05_5")?;
        line.wait_for(stage, "six squares")?;
        let differ = face.differences(other);
        let mut rings = Vec::with_capacity(2 * differ.len());
        for &(r, c) in &differ {
            rings.push(left.ring(stage, r, c, palette::switch()));
            rings.push(right.ring(stage, r, c, palette::switch()));
        }
        fade_in_all(stage, &rings, 0.4)?;
        line.wait_for(stage, "Each row")?;
        let lines_of = |line_of: fn((usize, usize)) -> usize| -> Vec<usize> {
            let mut lines: Vec<usize> = differ.iter().map(|&rc| line_of(rc)).collect();
            lines.sort_unstable();
            lines.dedup();
            lines
        };
        let through: Vec<Mob> = lines_of(|(r, _)| r)
            .into_iter()
            .map(|r| other_rows[r])
            .chain(lines_of(|(_, c)| c).into_iter().map(|c| other_cols[c]))
            .chain(lines_of(|(r, c)| r + c).into_iter().map(|k| other_diagonals[k]))
            .collect();
        if through.len() != changes.len() {
            return Err(fail(format!(
                "{} lines through the six squares drawn, {} changed",
                through.len(),
                changes.len()
            )));
        }
        let balance = kit.text(stage, THROUGH_THEM, 22.0, Palette::ink(), v(0.0, BOTTOM_Y))?;
        flash_all(stage, &through, 0.6)?;
        fade_in_all(stage, &[balance], 0.3)?;
        line.wait_for(stage, "all three shadows")?;
        let mut every_count = face_rows.clone();
        every_count.extend(&face_cols);
        every_count.extend(&face_diagonals);
        every_count.extend(&other_rows);
        every_count.extend(&other_cols);
        every_count.extend(&other_diagonals);
        flash_all(stage, &every_count, 0.7)?;

        // 05_6: three shadows, two pictures
        let line = say(&kit, stage, "05_6")?;
        line.wait_for(stage, "A third still leaves two")?;
        fade_out_all(stage, &[result], 0.3)?;
        let shadows = [&face_rows, &face_cols, &face_diagonals].len();
        let verdict = kit.text(
            stage,
            &shadows_label(shadows, survivors.len()),
            26.0,
            Palette::ink(),
            v(0.0, TOP_Y),
        )?;
        fade_in_all(stage, &[verdict], 0.4)?;

        // 05_7: the bounded model on the card; the two questions with their
        // answers for these three shadows; held until read, then the fade
        let line = say(&kit, stage, "05_7")?;
        let mut leaving = on_right;
        leaving.extend(rings);
        leaving.extend([balance, verdict]);
        fade_out_all(stage, &leaving, 0.4)?;
        let claim = model_claim(face.rows(), face.cols());
        let card = precisely(&kit, stage, &claim)?;
        fade_in_all(stage, &[card], 0.4)?;
        let card_up = stage.scene().time().to_f64();
        let answers = [FIRST_ANSWER.to_string(), second_answer(survivors.len(), shadows)];
        let mut closing = Vec::with_capacity(4);
        for (i, (question, answer)) in QUESTIONS.iter().zip(&answers).enumerate() {
            closing.push(kit.text_left(stage, question, 28.0, Palette::ink(), v(CLOSING_X, CLOSING_Y[2 * i]))?);
            closing.push(kit.text_left(stage, answer, 22.0, Palette::ink(), v(CLOSING_X, CLOSING_Y[2 * i + 1]))?);
        }
        line.wait_for(stage, "Does any picture")?;
        fade_in_all(stage, &closing[..2], 0.4)?;
        let shown = stage.scene().time().to_f64();
        line.wait_for(stage, "And is it")?;
        fade_in_all(stage, &closing[2..], 0.4)?;
        line.wait_to_end(stage)?;
        let all_words = [QUESTIONS[0], answers[0].as_str(), QUESTIONS[1], answers[1].as_str()].join(" ");
        wait_until(stage, (card_up + read_s(&claim)).max(shown + read_s(&all_words)))?;
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
            ("05_1", "along each diagonal"),
            ("05_2", "A switch keeps"),
            ("05_2", "always changes"),
            ("05_2", "This one takes"),
            ("05_3", "none of the fourteen"),
            ("05_3", "all sixty-four"),
            ("05_4", "Sixty-two fail"),
            ("05_4", "one other picture"),
            ("05_5", "six squares"),
            ("05_5", "Each row"),
            ("05_5", "all three shadows"),
            ("05_6", "A third still leaves two"),
            ("05_7", "Does any picture"),
            ("05_7", "And is it"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The words on screen are the facts': the 64, the two that pass, three
    /// shadows; the card bounds the model to the face's grid in two whole
    /// lines (three at every narrower width), and it and the closing words
    /// are held long enough to read and under twelve seconds each.
    #[test]
    fn the_words_on_screen_are_built_from_the_facts() {
        let f = facts::facts().unwrap();
        let get = |key: &str| -> usize { f.get(key).unwrap().parse().unwrap() };
        assert_eq!(
            gallery_label(get("DT-F3.fits")),
            "64 pictures with the face's rows and columns"
        );
        assert_eq!(
            passed_label(get("DT-F11.survivors"), get("DT-F3.fits")),
            "2 of 64 cast the face's diagonal shadow too"
        );
        assert_eq!(
            shadows_label(get("DT-F11.shadows"), get("DT-F11.survivors")),
            "3 shadows, 2 pictures"
        );
        let second = second_answer(get("DT-F11.survivors"), get("DT-F11.shadows"));
        assert_eq!(second, "no: 2 pictures cast all 3 shadows");
        let claim = model_claim(5, 5);
        assert!(claim.ends_with("one diagonal direction"), "{claim}");
        assert_eq!(f.get("DT-F1.size"), Some("5,5"));
        assert!(wrap(&claim, 92, 3).len() <= 2, "{claim}");
        for width in (76..=92).step_by(4) {
            let lines = wrap(&claim, width, 3);
            assert!(!lines.iter().any(|l| l.ends_with('…')), "{width}: {lines:?}");
        }
        let all_words = [QUESTIONS[0], FIRST_ANSWER, QUESTIONS[1], second.as_str()].join(" ");
        for text in [claim.as_str(), all_words.as_str()] {
            assert!(read_s(text) < 12.0, "{text}");
        }
    }

    /// The diagonals: every square on exactly one, nine on a 5 x 5 grid,
    /// each listed lower left to upper right with its row and column adding
    /// to its number; its count stands one square on along it, outside the
    /// grid.
    #[test]
    fn the_diagonals_cover_the_grid_once() {
        let board = Board::new(LEFT, CELL, 5, 5);
        let mut seen = Vec::new();
        for k in 0..9 {
            let squares = diagonal_squares(&board, k);
            assert!(!squares.is_empty(), "{k}");
            for w in squares.windows(2) {
                assert_eq!((w[1].0 + 1, w[1].1), (w[0].0, w[0].1 + 1), "{k}: {squares:?}");
            }
            assert!(squares.iter().all(|&(r, c)| r + c == k));
            seen.extend(squares);
            let at = diagonal_count_at(&board, k);
            assert!(at[0] > board.right() || at[1] > board.top(), "{k}");
        }
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), 25);
        assert!(diagonal_squares(&board, 9).is_empty());
    }

    /// The chapter's layout keeps its pieces apart and inside the
    /// title-safe area (text 0.25 in from the frame's edge; about 0.0045
    /// units a point a character).
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        let left = Board::new(LEFT, CELL, 5, 5);
        let right = Board::new(RIGHT, CELL, 5, 5);
        let safe_x = FRAME_X_RADIUS - 0.25;
        let safe_y = FRAME_Y_RADIUS - 0.25;
        let width = |text: &str, size: f64| text.chars().count() as f64 * size * 0.0045;
        // the face's counts on the left, the other picture's on the right
        assert!(tap_at(&left, 0)[0] - 0.2 > -safe_x);
        let left_reach = diagonal_count_at(&left, 8)[0] + 0.2;
        assert!(left_reach < tap_at(&right, 0)[0] - 0.3);
        assert!(diagonal_count_at(&right, 8)[0] + 0.2 < safe_x);
        assert!(diagonal_count_at(&right, 4)[1] + 0.2 < right.top() + NAME_DY - 0.15);
        assert!(left.top() + NAME_DY + 0.2 < TOP_Y - 0.2);
        assert!(TOP_Y + 0.2 < safe_y);
        // the result over the other picture: above its name, inside, and
        // right of the face's "diagonal shadow" label
        let result_half = width(&passed_label(2, 64), 22.0) / 2.0;
        assert!(RESULT_Y - 0.2 > right.top() + NAME_DY + 0.2);
        assert!(RESULT_Y + 0.2 < safe_y);
        assert!(RIGHT[0] + result_half < safe_x);
        let label_end = left.right() + left.cell * 0.5 + width(DIAGONAL_SHADOW, 22.0) / 2.0;
        assert!(RIGHT[0] - result_half > label_end + 0.2);
        // the think ring and its note sit between the face and the gallery
        let gallery_left = gallery_at(0, 64)[0] - THUMB_CELL * 2.5 - 0.03;
        assert!(THINK_AT[0] - 0.62 > left_reach - 0.1);
        assert!(THINK_AT[0] + 0.62 < gallery_left);
        assert!(THINK_AT[0] + width("pause for more time", 18.0) / 2.0 < gallery_left);
        // the caption over the gallery ends inside
        let caption_x = GALLERY[0];
        assert!(caption_x + width(&gallery_label(64), 22.0) / 2.0 < safe_x);
        // the closing words start right of the face's diagonal counts and
        // end inside
        assert!(CLOSING_X > left_reach);
        assert!(CLOSING_X + width(FIRST_ANSWER, 22.0) < safe_x);
        // the line under the pictures clears their column counts and stands
        // over a two-line card, as do those counts
        let card_top = -FRAME_Y_RADIUS + 0.2 + 0.3 + 2.0 * 0.36;
        assert!(BOTTOM_Y + 0.15 < left.col_count_at(0)[1] - 0.2);
        assert!(BOTTOM_Y - 0.15 > card_top);
        assert!(width(THROUGH_THEM, 22.0) / 2.0 < safe_x);
        assert!(CLOSING_Y[3] - 0.15 > card_top);
    }
}
