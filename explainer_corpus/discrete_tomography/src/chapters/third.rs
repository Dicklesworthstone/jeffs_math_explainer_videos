//! Chapter 10, a third shadow, and a fourth: back to the face's sixty-four
//! pictures, and the counts along the diagonals from lower left to upper
//! right (10_1); a switch keeps every row and column count but always
//! changes a diagonal count, and the opening switch takes a square from two
//! diagonals and gives both to a third (10_2); so none of the fourteen
//! pictures one switch from the face can pass; how many of all sixty-four
//! do? (10_3, a prediction); two: the face and one other picture (10_4), six
//! squares apart, every row, column and diagonal through them losing one and
//! gaining one (10_5); here even three shadows lie (10_6); was the face
//! unlucky? forty-eight of the sixty-four are the only picture with their
//! three shadows, and the face is not one of them (10_7); the face is its own
//! mirror image, so along the other diagonals its counts are the first ones
//! mirrored, and the survivor there is the mirror image of the first (10_8);
//! both diagonal shadows leave only the face: this picture, not a rule
//! (10_9).
//!
//! The pictures are chapter 01's 64 (`facts::gallery`); the third shadow is
//! `model::diagonal_counts`, the fourth `model::falling_counts`; the
//! survivors are `facts::diagonal_survivors`, `facts::falling_survivors`
//! and `facts::both_survivors`; the groups `facts::diagonal_groups`.

use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    Board, COUNT_SIZE, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, pause, say, segment, swap, think,
    thumb,
};
use super::same_shadows::{GALLERY, THUMB_CELL, counts, gallery_at, place, run_switch};
use super::water::{CELL, tap_at};
use crate::facts;
use crate::model::{self, Picture};
use crate::palette;

pub struct AThirdShadow {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const THE_FACE: &str = "the face";
/// The face's name while the opening switch has changed it (05_2).
const ONE_SWITCH_AWAY: &str = "one switch away";
const ANOTHER: &str = "another picture";
const DIAGONAL_SHADOW: &str = "diagonal shadow";
const THROUGH_THEM: &str = "each row, column and diagonal through them: −1 +1";
/// The mirror copy's name (10_8), and the two survivors' (under them).
const OTHER_DIAGONALS: &str = "the face, other diagonals";
const FIRST_DIAGONALS: &str = "first diagonals";
const OTHER_DIAGONALS_SHORT: &str = "other diagonals";
const THIS_FACE_ONLY: &str = "this face only";

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
/// The two survivors under the pictures (10_8): how far each stands from
/// the middle, its height, its square's size.
const MIRROR_DX: f64 = 1.0;
const MIRROR_Y: f64 = -2.5;
const MIRROR_CELL: f64 = 0.12;
/// "this face only" under the verdict (10_9).
const ONLY_DY: f64 = 0.5;
/// A stroke along a diagonal starts this many pitches before its lower-left
/// square's centre; a beam (05_1) ends this far past its upper-right one's,
/// a guide (05_1 to 05_3) far enough to point at its count, short of it.
const BEFORE: f64 = 0.6;
const BEAM_REACH: f64 = 0.3;
const GUIDE_REACH: f64 = 0.7;

fn gallery_label(count: usize) -> String {
    format!("{count} pictures with the face's rows and columns")
}

fn passed_label(passed: usize, count: usize) -> String {
    format!("{passed} of {count} cast the face's diagonal shadow too")
}

fn shadows_label(shadows: usize, pictures: usize) -> String {
    format!("{shadows} shadows, {pictures} pictures")
}

/// "48 of 64 alone with all three of their shadows" (10_7).
fn groups_label(alone: usize, count: usize) -> String {
    format!("{alone} of {count} alone with all three of their shadows")
}

/// The other diagonal `k`'s squares on `board`, upper left to lower right:
/// those whose column plus `rows − 1` minus their row is `k`
/// (`model::falling_counts`).
fn falling_squares(board: &Board, k: usize) -> Vec<(usize, usize)> {
    (0..board.rows)
        .filter_map(|r| (k + r).checked_sub(board.rows - 1).filter(|&c| c < board.cols).map(|c| (r, c)))
        .collect()
}

/// Where the other diagonal `k`'s count stands: one square on past its
/// upper-left end (above the grid, or left of it).
fn falling_count_at(board: &Board, k: usize) -> Vec3 {
    let (r, c) = falling_squares(board, k).first().copied().unwrap_or((0, 0));
    let p = board.at(r, c);
    v(p[0] - board.cell, p[1] + board.cell)
}

/// A stroke along the other diagonal `k`: from `reach` of a pitch before its
/// upper-left square's centre to [`BEFORE`] of a pitch past its lower-right
/// one's. Detached.
fn falling_line(
    stage: &mut Stage<'_>,
    board: &Board,
    k: usize,
    reach: f64,
    color: Srgb,
    width: f64,
) -> fmn::Result<Mob> {
    let squares = falling_squares(board, k);
    let (Some(&(r0, c0)), Some(&(r1, c1))) = (squares.first(), squares.last()) else {
        return Err(fail(format!("the other diagonal {k} has no square")));
    };
    let (a, b) = (board.at(r0, c0), board.at(r1, c1));
    let (before, after) = (board.cell * reach, board.cell * BEFORE);
    Ok(segment(stage, v(a[0] - before, a[1] + before), v(b[0] + after, b[1] - after), color, width))
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

/// Where a stroke along diagonal `k` starts and ends: [`BEFORE`] of a pitch
/// before its lower-left square's centre, `reach` of a pitch past its
/// upper-right one's.
fn diagonal_ends(board: &Board, k: usize, reach: f64) -> fmn::Result<(Vec3, Vec3)> {
    let squares = diagonal_squares(board, k);
    let (Some(&(r0, c0)), Some(&(r1, c1))) = (squares.first(), squares.last()) else {
        return Err(fail(format!("diagonal {k} has no square")));
    };
    let (a, b) = (board.at(r0, c0), board.at(r1, c1));
    let (before, after) = (board.cell * BEFORE, board.cell * reach);
    Ok((v(a[0] - before, a[1] - before), v(b[0] + after, b[1] + after)))
}

/// A stroke along diagonal `k` (see [`diagonal_ends`]). Detached.
fn diagonal_line(
    stage: &mut Stage<'_>,
    board: &Board,
    k: usize,
    reach: f64,
    color: Srgb,
    width: f64,
) -> fmn::Result<Mob> {
    let (a, b) = diagonal_ends(board, k, reach)?;
    Ok(segment(stage, a, b, color, width))
}

/// Root detached `mobs` behind everything on screen, for a fade to bring in:
/// a diagonal's stroke under the picture's squares, so it never covers one.
fn behind(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    for &m in mobs {
        stage.add_to_scene(m)?;
    }
    stage.scene_mut().bring_to_back(mobs)?;
    Ok(())
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
        "10_more_shadows"
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

        // 10_1: the face and its two shadows; a beam along each diagonal
        // leaves its count and a faint guide under the squares, which ties
        // the count to its diagonal until the switch is undone (05_3)
        let line = say(&kit, stage, "10_1")?;
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
            .map(|k| diagonal_line(stage, &left, k, BEAM_REACH, palette::beam(), 3.0))
            .collect::<fmn::Result<_>>()?;
        fade_in_lagged(stage, &beams, 1.2, 0.3)?;
        let mut guides: Vec<Mob> = (0..face_diagonal.len())
            .map(|k| diagonal_line(stage, &left, k, GUIDE_REACH, Palette::faint(), 2.0))
            .collect::<fmn::Result<_>>()?;
        behind(stage, &guides)?;
        let mut left_by_beams = face_diagonals.clone();
        left_by_beams.extend(&guides);
        swap(stage, &beams, &left_by_beams, 0.4)?;
        let label = kit.text(
            stage,
            DIAGONAL_SHADOW,
            22.0,
            Palette::dim(),
            v(left.right() + left.cell * 0.5, left.top() + NAME_DY),
        )?;
        fade_in_all(stage, &[label], 0.3)?;

        // 10_2: the opening switch keeps the rows and columns, and changes
        // three diagonal counts; the switched picture is named for what it
        // is; its three diagonals light up through its corners, the two that
        // each lose a square, then the one that gains both
        let line = say(&kit, stage, "10_2")?;
        line.wait_for(stage, "A switch keeps")?;
        let s = model::OPENING;
        let mut marks = vec![left.rectangle(stage, s, palette::switch())];
        marks.extend(s.corners().map(|(r, c)| left.ring(stage, r, c, palette::switch())));
        fade_in_all(stage, &marks, 0.4)?;
        let twin = run_switch(stage, &left, &mut squares, &face, s, 0.6)?;
        if twin != facts::twin().map_err(fail)? {
            return Err(fail(format!("the opening switch made {}", twin.pattern())));
        }
        let away_name = kit.text(
            stage,
            ONE_SWITCH_AWAY,
            24.0,
            Palette::dim(),
            v(left.centre[0], left.top() + NAME_DY),
        )?;
        fade_out_all(stage, &[face_name], 0.15)?;
        fade_in_all(stage, &[away_name], 0.15)?;
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
        fade_out_all(stage, &old, 0.2)?;
        fade_in_all(stage, &red, 0.2)?;
        let lose: Vec<usize> = changed
            .iter()
            .copied()
            .filter(|&k| twin_diagonal[k] < face_diagonal[k])
            .collect();
        let gain: Vec<usize> = changed
            .iter()
            .copied()
            .filter(|&k| twin_diagonal[k] > face_diagonal[k])
            .collect();
        if lose.len() != 2
            || gain.len() != 1
            || lose.iter().any(|&k| face_diagonal[k] - twin_diagonal[k] != 1)
            || gain.iter().any(|&k| twin_diagonal[k] - face_diagonal[k] != 2)
        {
            return Err(fail(format!(
                "the opening switch takes from diagonals {lose:?} and gives to {gain:?}"
            )));
        }
        for (phrase, lit_up) in [("This one takes", &lose), ("gives both", &gain)] {
            line.wait_for(stage, phrase)?;
            let lit: Vec<Mob> = lit_up
                .iter()
                .map(|&k| diagonal_line(stage, &left, k, GUIDE_REACH, palette::beam(), 4.0))
                .collect::<fmn::Result<_>>()?;
            behind(stage, &lit)?;
            let faint: Vec<Mob> = lit_up.iter().map(|&k| guides[k]).collect();
            swap(stage, &faint, &lit, 0.3)?;
            for (&k, &m) in lit_up.iter().zip(&lit) {
                guides[k] = m;
            }
            let their_counts: Vec<Mob> = lit_up
                .iter()
                .filter_map(|&k| changed.iter().position(|&j| j == k).map(|i| red[i]))
                .collect();
            flash_all(stage, &their_counts, 0.6)?;
        }

        // 10_3: the switch undone, the face named again, the counts and the
        // guides done with; the 64 on the right; none of the fourteen one
        // switch away can pass; the question, the think time
        let line = say(&kit, stage, "10_3")?;
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
        let face_again = kit.text(
            stage,
            THE_FACE,
            24.0,
            Palette::dim(),
            v(left.centre[0], left.top() + NAME_DY),
        )?;
        let mut going = red;
        going.push(away_name);
        let mut coming = restored.clone();
        coming.push(face_again);
        fade_out_all(stage, &going, 0.2)?;
        fade_in_all(stage, &coming, 0.2)?;
        for (&k, &m) in changed.iter().zip(&restored) {
            face_diagonals[k] = m;
        }
        let mut done = marks;
        done.extend(&guides);
        fade_out_all(stage, &done, 0.3)?;
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

        // 10_4: two pass; the other forty-eight that fail go; the other
        // survivor on the right with all three shadows green
        let line = say(&kit, stage, "10_4")?;
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
        fade_out_all(stage, &[caption], 0.15)?;
        fade_in_all(stage, &[passed], 0.15)?;
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

        // 10_5: the six squares where they differ; every line through them
        // loses one and gains one; every count flashes
        let line = say(&kit, stage, "10_5")?;
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

        // 10_6: three shadows, two pictures
        let line = say(&kit, stage, "10_6")?;
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

        // 10_7: was the face unlucky? the 64 grouped by their diagonal
        // counts: those alone framed green, the groups of more dim; the
        // face's group flashes
        let line = say(&kit, stage, "10_7")?;
        let mut leaving = on_right;
        leaving.extend(rings);
        leaving.extend([balance, verdict]);
        fade_out_all(stage, &leaving, 0.4)?;
        let groups = facts::diagonal_groups();
        let face_group = groups
            .iter()
            .find(|g| g.contains(&face))
            .ok_or_else(|| fail("the face is in no group".into()))?;
        let alone_count = groups.iter().filter(|g| g.len() == 1).count();
        let ordered: Vec<(&Picture, bool)> = groups
            .iter()
            .flat_map(|g| g.iter().map(move |p| (p, g.len() == 1)))
            .collect();
        let n = ordered.len();
        if n != gallery.len() || face_group.len() == 1 {
            return Err(fail(format!(
                "{n} pictures grouped, or the face is alone with its three shadows"
            )));
        }
        let mut regrouped = Vec::with_capacity(2 * n);
        let mut face_pair = Vec::new();
        for (k, &(p, alone)) in ordered.iter().enumerate() {
            let frame = if alone { palette::matches() } else { Palette::dim() };
            let mobs = thumb(stage, p, gallery_at(k, n), THUMB_CELL, frame)?;
            if face_group.contains(p) {
                face_pair.extend(&mobs);
            }
            regrouped.extend(mobs);
        }
        let grouped_caption = kit.text(
            stage,
            &groups_label(alone_count, n),
            22.0,
            Palette::ink(),
            v(GALLERY[0], top),
        )?;
        fade_in_lagged(stage, &regrouped, 1.0, 0.2)?;
        fade_in_all(stage, &[grouped_caption], 0.3)?;
        line.wait_for(stage, "not one of them")?;
        flash_all(stage, &face_pair, 0.6)?;
        line.wait_to_end(stage)?;
        pause(stage, 1.0)?;

        // 10_8: the face is its own mirror image; a copy on the right with
        // its counts along the other diagonals, past their upper-left ends;
        // under the pictures, the first survivor and the other diagonals'
        // survivor, mirror images
        let line = say(&kit, stage, "10_8")?;
        let mut going = regrouped;
        going.push(grouped_caption);
        fade_out_all(stage, &going, 0.4)?;
        if model::mirrored(&face) != face {
            return Err(fail("the face is not its own mirror image".into()));
        }
        let falling = model::falling_counts(&face);
        let mut mirror_copy = right.lines(stage);
        mirror_copy.extend(place(stage, &right, &face)?.into_values());
        mirror_copy.push(kit.text(
            stage,
            OTHER_DIAGONALS,
            24.0,
            Palette::dim(),
            v(right.centre[0], right.top() + NAME_DY),
        )?);
        fade_in_all(stage, &mirror_copy, 0.5)?;
        line.wait_for(stage, "the other diagonals")?;
        let beams: Vec<Mob> = (0..falling.len())
            .map(|k| falling_line(stage, &right, k, BEAM_REACH, palette::beam(), 3.0))
            .collect::<fmn::Result<_>>()?;
        fade_in_lagged(stage, &beams, 1.2, 0.3)?;
        let falling_marks = counts(&kit, stage, &falling, Palette::ink(), |k| falling_count_at(&right, k))?;
        swap(stage, &beams, &falling_marks, 0.4)?;
        mirror_copy.extend(&falling_marks);
        line.wait_for(stage, "mirror image")?;
        let falling_survivors = facts::falling_survivors();
        let falling_others: Vec<&Picture> = falling_survivors.iter().filter(|p| **p != face).collect();
        let &[falling_other] = falling_others.as_slice() else {
            return Err(fail(format!(
                "{} pictures besides the face cast its other diagonal shadow",
                falling_others.len()
            )));
        };
        if model::mirrored(other) != *falling_other {
            return Err(fail("the two survivors are not mirror images".into()));
        }
        let mut mirrored_pair = thumb(stage, other, v(-MIRROR_DX, MIRROR_Y), MIRROR_CELL, Palette::faint())?;
        mirrored_pair.extend(thumb(stage, falling_other, v(MIRROR_DX, MIRROR_Y), MIRROR_CELL, Palette::faint())?);
        let half = MIRROR_CELL * face.rows() as f64 / 2.0 + 0.1;
        mirrored_pair.push(segment(
            stage,
            v(0.0, MIRROR_Y + half),
            v(0.0, MIRROR_Y - half),
            Palette::dim(),
            2.0,
        ));
        for (name, x) in [(FIRST_DIAGONALS, -MIRROR_DX), (OTHER_DIAGONALS_SHORT, MIRROR_DX)] {
            mirrored_pair.push(kit.text(stage, name, 18.0, Palette::dim(), v(x, MIRROR_Y - half - 0.2))?);
        }
        fade_in_all(stage, &mirrored_pair, 0.5)?;

        // 10_9: both diagonal shadows leave only the face: four shadows, one
        // picture, this face only
        let line = say(&kit, stage, "10_9")?;
        if facts::both_survivors() != [face.clone()] {
            return Err(fail("both diagonal shadows do not leave the face alone".into()));
        }
        line.wait_for(stage, "only the face is left")?;
        fade_out_all(stage, &mirrored_pair, 0.3)?;
        let four = [&face_rows, &face_cols, &face_diagonals, &falling_marks].len();
        let verdict_four = kit.text(stage, &shadows_label(four, 1), 26.0, Palette::ink(), v(0.0, TOP_Y))?;
        fade_in_all(stage, &[verdict_four], 0.4)?;
        line.wait_for(stage, "not a rule")?;
        let only_this = kit.text(stage, THIS_FACE_ONLY, 22.0, Palette::dim(), v(0.0, TOP_Y - ONLY_DY))?;
        fade_in_all(stage, &[only_this], 0.4)?;
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

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("10_1", "along each diagonal"),
            ("10_2", "A switch keeps"),
            ("10_2", "always changes"),
            ("10_2", "This one takes"),
            ("10_2", "gives both"),
            ("10_3", "none of the fourteen"),
            ("10_3", "all sixty-four"),
            ("10_4", "Sixty-two fail"),
            ("10_4", "one other picture"),
            ("10_5", "six squares"),
            ("10_5", "Each row"),
            ("10_5", "all three shadows"),
            ("10_6", "A third still leaves two"),
            ("10_7", "not one of them"),
            ("10_8", "the other diagonals"),
            ("10_8", "mirror image"),
            ("10_9", "only the face is left"),
            ("10_9", "not a rule"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The words on screen are the facts': the 64, the two that pass, three
    /// shadows; the 48 alone; four shadows and one picture.
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
        assert_eq!(
            groups_label(get("DT-F18.alone"), get("DT-F3.fits")),
            "48 of 64 alone with all three of their shadows"
        );
        assert_eq!(
            shadows_label(get("DT-F18.shadows"), get("DT-F18.both.count")),
            "4 shadows, 1 picture"
        );
        assert_eq!(f.get("DT-F18.face.alone"), Some("false"));
        assert_eq!(f.get("DT-F18.falling.other.mirror"), Some("true"));
    }

    /// The other diagonals: every square on exactly one, nine on a 5 x 5
    /// grid, each listed upper left to lower right with its column plus 4
    /// minus its row equal to its number; its count stands one square on
    /// past its upper-left end, outside the grid; the face's counts there
    /// are the model's.
    #[test]
    fn the_other_diagonals_cover_the_grid_once() {
        let board = Board::new(RIGHT, CELL, 5, 5);
        let mut seen = Vec::new();
        let face = model::face();
        let falling = model::falling_counts(&face);
        for k in 0..9 {
            let squares = falling_squares(&board, k);
            assert!(!squares.is_empty(), "{k}");
            for w in squares.windows(2) {
                assert_eq!((w[0].0 + 1, w[0].1 + 1), w[1], "{k}: {squares:?}");
            }
            assert!(squares.iter().all(|&(r, c)| c + 4 - r == k));
            assert_eq!(squares.iter().filter(|&&(r, c)| face.get(r, c)).count(), falling[k], "{k}");
            seen.extend(&squares);
            let at = falling_count_at(&board, k);
            assert!(at[0] < board.left() || at[1] > board.top(), "{k}");
        }
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), 25);
        assert!(falling_squares(&board, 9).is_empty());
    }

    /// The diagonals: every square on exactly one, nine on a 5 x 5 grid,
    /// each listed lower left to upper right with its row and column adding
    /// to its number; its count stands one square on along it, outside the
    /// grid, and its guide leaves the grid towards the count and stops short
    /// of it.
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
            let (r1, c1) = squares[squares.len() - 1];
            let last = board.at(r1, c1);
            seen.extend(squares);
            let at = diagonal_count_at(&board, k);
            assert!(at[0] > board.right() || at[1] > board.top(), "{k}");
            let (_, end) = diagonal_ends(&board, k, GUIDE_REACH).unwrap();
            assert!(
                end[0] - last[0] > board.cell / 2.0,
                "{k}: the guide stops inside the grid"
            );
            assert!(
                at[0] - end[0] > 0.15 && at[1] - end[1] > 0.15,
                "{k}: the guide runs into its count"
            );
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
        // the switched face's name, over it, clears "diagonal shadow"
        let label_start = left.right() + left.cell * 0.5 - width(DIAGONAL_SHADOW, 22.0) / 2.0;
        assert!(left.centre[0] + width(ONE_SWITCH_AWAY, 24.0) / 2.0 + 0.1 < label_start);
        // the think ring and its note sit between the face and the gallery
        let gallery_left = gallery_at(0, 64)[0] - THUMB_CELL * 2.5 - 0.03;
        assert!(THINK_AT[0] - 0.62 > left_reach - 0.1);
        assert!(THINK_AT[0] + 0.62 < gallery_left);
        assert!(THINK_AT[0] + width("pause for more time", 18.0) / 2.0 < gallery_left);
        // the caption over the gallery ends inside
        let caption_x = GALLERY[0];
        assert!(caption_x + width(&gallery_label(64), 22.0) / 2.0 < safe_x);
        // the other diagonals' counts on the right-hand copy clear the
        // face's diagonal counts on the left; the copy's name stays over them
        let right_counts = falling_count_at(&right, 4)[0] - 0.2;
        assert!(right_counts > left_reach + 0.3);
        assert!(falling_count_at(&right, 8)[1] + 0.2 < right.top() + NAME_DY - 0.15);
        assert!(right.centre[0] + width(OTHER_DIAGONALS, 24.0) / 2.0 < safe_x);
        // the two survivors under the pictures: apart, under the face's
        // column counts, their names inside
        let half = MIRROR_CELL * 5.0 / 2.0 + 0.1;
        assert!(MIRROR_DX - half > 0.15);
        assert!(MIRROR_Y + half + 0.15 < left.col_count_at(0)[1] - 0.2);
        assert!(MIRROR_Y - half - 0.2 - 0.15 > -safe_y);
        assert!(MIRROR_DX + width(FIRST_DIAGONALS, 18.0) / 2.0 < safe_x);
        assert!(width(&groups_label(48, 64), 22.0) / 2.0 + GALLERY[0] < safe_x);
        // "this face only" stands between the two pictures' names
        let only_half = width(THIS_FACE_ONLY, 22.0) / 2.0;
        assert!(only_half + 0.2 < right.centre[0] - width(OTHER_DIAGONALS, 24.0) / 2.0);
        assert!(-only_half - 0.2 > left.centre[0] + width(THE_FACE, 24.0) / 2.0);
        assert!(TOP_Y - ONLY_DY - 0.15 > right.top() + 0.2);
        // the line under the pictures clears their column counts and stands
        // over a two-line card, as do those counts
        let card_top = -FRAME_Y_RADIUS + 0.2 + 0.3 + 2.0 * 0.36;
        assert!(BOTTOM_Y + 0.15 < left.col_count_at(0)[1] - 0.2);
        assert!(BOTTOM_Y - 0.15 > card_top);
        assert!(width(THROUGH_THEM, 22.0) / 2.0 < safe_x);
    }
}
