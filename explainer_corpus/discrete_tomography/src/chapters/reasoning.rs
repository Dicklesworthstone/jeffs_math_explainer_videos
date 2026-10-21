//! Chapter 02, square by square: this puzzle has a serious side, rebuilding
//! tiny crystals from counts of atoms along lines, often from only a few
//! directions (02_1); how would you rebuild a picture from its shadows?
//! Start with the squares you can be sure of (02_2); the tree's shadows
//! (02_3): the row of five and the column of five are full (02_4), the
//! first and last columns and the rows of one then have their square and
//! the rest of them is empty (02_5), and the second row's last two squares
//! fill: every square decided, the tree (02_6); the face's shadows: its
//! empty row is easy (02_7), and then? (02_8, a prediction: can you be sure
//! of any other square?); no: every other square is filled in some of the
//! sixty-four pictures and empty in others (02_9); reasoning row by row and
//! column by column gets stuck, and any next guess agrees with some of them
//! (02_10).
//!
//! The reasoning is `model::line_solve`'s, told in the order
//! `facts::tree_reasoning` checks; how often each square is filled is
//! `facts::filled_in`, over the 64 pictures of `facts::gallery`.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    Board, COUNT_SIZE, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, pause, say, segment, swap, think,
    wait_until,
};
use super::glyphs::TITLE;
use super::same_shadows::counts;
use super::stops::read_s;
use crate::facts::{self, BATENBURG_YEAR};
use crate::model::{self, Deduction, Line};
use crate::palette;

pub struct SquareBySquare {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const SURE_FILLED: &str = "sure: filled";
const SURE_EMPTY: &str = "sure: empty";
const DECIDED: &str = "every square decided";
const FILLED_IN: &str = "filled in how many of the 64";

/// The grid's pitch, and where it stands: on the left, counts on its right
/// and under it (chapter 01's layout).
const CELL: f64 = 0.62;
const AT: [f64; 2] = [-3.9, 0.35];
/// Where the words on the right start, and their lines.
const TEXT_X: f64 = 0.9;
const WORDS_Y: [f64; 4] = [2.0, 1.4, 0.8, 0.2];
/// The title's place and size (from the chapter's first frame to the card).
const TITLE_Y: f64 = 3.2;
const TITLE_SIZE: f64 = 44.0;
/// The think ring's centre (02_8).
const THINK_AT: [f64; 2] = [4.4, 0.6];
/// The legend's sample square: its pitch, and how far right of it the
/// words start.
const LEGEND_CELL: f64 = 0.4;
const LEGEND_TEXT_DX: f64 = 0.5;
/// The numbers in the undecided squares (02_9), and how strongly the most
/// often filled is tinted.
const SHARE_SIZE: f64 = 20.0;
const SHARE_TINT: f64 = 0.7;

/// The exact scope of 02_1, kept on screen until it can be read
/// (`read_s`): Batenburg's motivation, and the video's flat, exact grids.
fn crystal_claim() -> String {
    format!(
        "atoms can be counted along columns through a nanocrystal, from few images (Batenburg, \
         {BATENBURG_YEAR}, section 1.9.1); here every grid is flat and every count exact, with no microscope"
    )
}

fn guess_text(filled: usize, of: usize) -> [String; 2] {
    [
        format!("filled: {filled} of {of}"),
        format!("empty: {} of {of}", of - filled),
    ]
}

/// Redraw count `i` of `marks` as `n` in `color` at `at`, in place.
fn recount(
    kit: &Kit,
    stage: &mut Stage<'_>,
    marks: &mut [Mob],
    i: usize,
    n: usize,
    color: Srgb,
    at: Vec3,
) -> fmn::Result<()> {
    let new = kit.text(stage, &n.to_string(), COUNT_SIZE, color, at)?;
    swap(stage, &[marks[i]], &[new], 0.25)?;
    marks[i] = new;
    Ok(())
}

/// The picture of deduction `d`'s squares on `board`: filled squares for a
/// fill, small dim dots for an emptying. Detached.
fn settle_marks(stage: &mut Stage<'_>, board: &Board, d: &Deduction) -> fmn::Result<Vec<Mob>> {
    d.squares
        .iter()
        .map(|&(r, c)| {
            if d.fill {
                board.square(stage, r, c, palette::filled())
            } else {
                Ok(board.dot(stage, r, c, Palette::dim()))
            }
        })
        .collect()
}

impl SceneConstruct for SquareBySquare {
    fn name(&self) -> &str {
        "02_square_by_square"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let tree = model::tree();
        let tree_shadows = tree.shadows();
        let [full, met, rest] = facts::tree_reasoning().map_err(fail)?;
        let (rows, cols) = (tree.rows(), tree.cols());
        // the words' shape: the full row, then the full column; the first and
        // last columns, then the rows of one; one row left, the second
        let full_lines: Vec<Line> = full.iter().map(|d| d.line).collect();
        let (met_cols, met_rows): (Vec<&Deduction>, Vec<&Deduction>) =
            met.iter().partition(|d| matches!(d.line, Line::Col(_)));
        let met_col_ids: Vec<Line> = met_cols.iter().map(|d| d.line).collect();
        let shape_ok = matches!(full_lines.as_slice(), [Line::Row(_), Line::Col(_)])
            && met_col_ids == [Line::Col(0), Line::Col(cols - 1)]
            && met_rows.iter().all(|d| d.line.count(&tree_shadows) == 1)
            && matches!(rest.as_slice(), [d] if d.line == Line::Row(1) && d.fill);
        if !shape_ok {
            return Err(fail(format!(
                "the tree's reasoning is not told in the chapter's order: {:?}",
                [&full, &met, &rest]
            )));
        }
        let face = model::face();
        let face_shadows = face.shadows();
        let face_solve = model::line_solve(&face_shadows);
        let [face_step] = face_solve.steps.as_slice() else {
            return Err(fail(format!("the face's reasoning takes {} steps, not one", face_solve.steps.len())));
        };
        let empty_row = face_shadows.rows.iter().position(|&n| n == 0);
        if face_step.fill || Some(face_step.line) != empty_row.map(Line::Row) || face_solve.contradiction.is_some() {
            return Err(fail("the face's reasoning does not empty its empty row and stop".into()));
        }
        let filled = facts::filled_in();
        let of = facts::gallery().len();
        let board = Board::new(AT, CELL, rows, cols);

        // 02_1: the title over an empty grid; beams count along a few lines
        // and leave nothing; on "tiny crystals" the scoped card replaces the
        // title
        let line = say(&kit, stage, "02_1")?;
        let grid = board.lines(stage);
        let title = kit.text(stage, TITLE, TITLE_SIZE, Palette::ink(), v(0.0, TITLE_Y))?;
        let mut arriving = grid.clone();
        arriving.push(title);
        fade_in_all(stage, &arriving, 0.6)?;
        let mut beams: Vec<Mob> = [1, 2, 3]
            .iter()
            .map(|&r| {
                let y = board.at(r, 0)[1];
                segment(stage, v(board.left() - 0.6, y), v(board.right() + 0.2, y), palette::beam(), 3.0)
            })
            .collect();
        beams.extend([1, 3].iter().map(|&c| {
            let x = board.at(0, c)[0];
            segment(stage, v(x, board.top() + 0.6), v(x, board.bottom() - 0.2), palette::beam(), 3.0)
        }));
        fade_in_lagged(stage, &beams, 1.2, 0.3)?;
        fade_out_all(stage, &beams, 0.4)?;
        line.wait_for(stage, "tiny crystals")?;
        let claim = crystal_claim();
        let card = precisely(&kit, stage, &claim)?;
        swap(stage, &[title], &[card], 0.4)?;
        let card_up = stage.scene().time().to_f64();

        // 02_2: the legend: sure filled, sure empty
        let line = say(&kit, stage, "02_2")?;
        line.wait_for(stage, "squares you can be sure of")?;
        let sample = Board::new([TEXT_X + LEGEND_CELL / 2.0, WORDS_Y[0]], LEGEND_CELL, 2, 1);
        let mut legend = vec![
            sample.square(stage, 0, 0, palette::filled())?,
            sample.dot(stage, 1, 0, Palette::dim()),
            kit.text_left(
                stage,
                SURE_FILLED,
                22.0,
                Palette::ink(),
                v(TEXT_X + LEGEND_TEXT_DX, sample.at(0, 0)[1]),
            )?,
        ];
        legend.push(kit.text_left(
            stage,
            SURE_EMPTY,
            22.0,
            Palette::ink(),
            v(TEXT_X + LEGEND_TEXT_DX, sample.at(1, 0)[1]),
        )?);
        fade_in_all(stage, &legend, 0.4)?;

        // 02_3: the card once read; the tree's shadows only
        let line = say(&kit, stage, "02_3")?;
        wait_until(stage, card_up + read_s(&claim))?;
        fade_out_all(stage, &[card], 0.3)?;
        line.wait_for(stage, "Rows of")?;
        let mut row_counts = counts(&kit, stage, &tree_shadows.rows, Palette::ink(), |r| board.row_count_at(r))?;
        fade_in_lagged(stage, &row_counts, 0.8, 0.3)?;
        line.wait_for(stage, "columns of")?;
        let mut col_counts = counts(&kit, stage, &tree_shadows.cols, Palette::ink(), |c| board.col_count_at(c))?;
        fade_in_lagged(stage, &col_counts, 0.8, 0.3)?;

        // 02_4: the row of five fills, then the column of five
        let line = say(&kit, stage, "02_4")?;
        let mut settled: Vec<Mob> = Vec::new();
        for (d, phrase) in full.iter().zip(["The row of five", "the column of five"]) {
            line.wait_for(stage, phrase)?;
            let marks = settle_marks(stage, &board, d)?;
            fade_in_lagged(stage, &marks, 0.6, 0.3)?;
            settled.extend(marks);
            green_count(&kit, stage, &board, &tree_shadows, d.line, &mut row_counts, &mut col_counts)?;
        }

        // 02_5: the first and last columns, then the rows of one, have their
        // square: everything else in them is empty
        let line = say(&kit, stage, "02_5")?;
        for (group, phrase) in [(&met_cols, "the first and last columns"), (&met_rows, "the rows of one")] {
            line.wait_for(stage, phrase)?;
            let mut marks = Vec::new();
            for d in group.iter() {
                marks.extend(settle_marks(stage, &board, d)?);
                green_count(&kit, stage, &board, &tree_shadows, d.line, &mut row_counts, &mut col_counts)?;
            }
            fade_in_lagged(stage, &marks, 0.6, 0.2)?;
            settled.extend(marks);
        }

        // 02_6: the second row's last two squares, ringed, then filled: every
        // square decided, the tree
        let line = say(&kit, stage, "02_6")?;
        let last = &rest[0];
        line.wait_for(stage, "exactly two squares")?;
        let rings: Vec<Mob> = last
            .squares
            .iter()
            .map(|&(r, c)| board.ring(stage, r, c, Palette::ink()))
            .collect();
        fade_in_all(stage, &rings, 0.3)?;
        let marks = settle_marks(stage, &board, last)?;
        fade_in_all(stage, &marks, 0.4)?;
        settled.extend(marks);
        fade_out_all(stage, &rings, 0.2)?;
        for r in 0..rows {
            recount(&kit, stage, &mut row_counts, r, tree_shadows.rows[r], palette::matches(), board.row_count_at(r))?;
        }
        for c in 0..cols {
            recount(&kit, stage, &mut col_counts, c, tree_shadows.cols[c], palette::matches(), board.col_count_at(c))?;
        }
        line.wait_for(stage, "the picture is the tree")?;
        let decided = kit.text(
            stage,
            DECIDED,
            24.0,
            Palette::ink(),
            v(board.centre[0], board.top() + 0.4),
        )?;
        fade_in_all(stage, &[decided], 0.3)?;
        line.wait_to_end(stage)?;
        pause(stage, 1.0)?;

        // 02_7: the face's shadows; its empty row is easy
        let line = say(&kit, stage, "02_7")?;
        let mut leaving = settled;
        leaving.extend(&row_counts);
        leaving.extend(&col_counts);
        leaving.push(decided);
        fade_out_all(stage, &leaving, 0.4)?;
        let mut face_rows = counts(&kit, stage, &face_shadows.rows, Palette::ink(), |r| board.row_count_at(r))?;
        let face_cols = counts(&kit, stage, &face_shadows.cols, Palette::ink(), |c| board.col_count_at(c))?;
        let mut arriving = face_rows.clone();
        arriving.extend(&face_cols);
        fade_in_all(stage, &arriving, 0.4)?;
        line.wait_for(stage, "Its empty row")?;
        let Line::Row(empty) = face_step.line else {
            return Err(fail("the face's first deduction is not a row".into()));
        };
        recount(&kit, stage, &mut face_rows, empty, 0, palette::matches(), board.row_count_at(empty))?;
        let dots = settle_marks(stage, &board, face_step)?;
        fade_in_lagged(stage, &dots, 0.6, 0.3)?;

        // 02_8: and then? The question, and the think time (nothing moves)
        let _line = say(&kit, stage, "02_8")?;
        let q = think(&kit, stage, v(THINK_AT[0], THINK_AT[1]))?;

        // 02_9: no: each undecided square is filled in some of the 64 and
        // empty in others, shaded by how often it is filled
        let line = say(&kit, stage, "02_9")?;
        fade_out_all(stage, &[q], 0.3)?;
        line.wait_for(stage, "Every other square")?;
        let undecided: Vec<(usize, usize)> = (0..rows)
            .flat_map(|r| (0..cols).map(move |c| (r, c)))
            .filter(|&(r, c)| face_solve.grid[r][c].is_none())
            .collect();
        if undecided.iter().any(|&(r, c)| filled[r][c] == 0 || filled[r][c] == of) {
            return Err(fail("an undecided square of the face is the same in all 64 pictures".into()));
        }
        let mut shares = Vec::with_capacity(2 * undecided.len());
        for &(r, c) in &undecided {
            let tint = SHARE_TINT * filled[r][c] as f64 / of as f64;
            shares.push(board.shade(stage, r, c, palette::filled(), tint)?);
            shares.push(kit.text(stage, &filled[r][c].to_string(), SHARE_SIZE, Palette::ink(), board.at(r, c))?);
        }
        let label = kit.text_left(stage, FILLED_IN, 22.0, Palette::ink(), v(TEXT_X, WORDS_Y[0]))?;
        let mut coming = shares.clone();
        coming.push(label);
        swap(stage, &legend, &coming, 0.5)?;

        // 02_10: stuck; whatever you guess next, some of the 64 agree
        let line = say(&kit, stage, "02_10")?;
        line.wait_for(stage, "Whatever you guess")?;
        let (gr, gc) = (0, 0);
        if face_solve.grid[gr][gc].is_some() {
            return Err(fail("the guessed square is already decided".into()));
        }
        let ring = board.ring(stage, gr, gc, Palette::ink());
        let [as_filled, as_empty] = guess_text(filled[gr][gc], of);
        let words = [
            kit.text_left(stage, &as_filled, 24.0, Palette::ink(), v(TEXT_X, WORDS_Y[1]))?,
            kit.text_left(stage, &as_empty, 24.0, Palette::ink(), v(TEXT_X, WORDS_Y[2]))?,
        ];
        fade_in_all(stage, &[ring], 0.3)?;
        fade_in_all(stage, &words, 0.4)?;
        flash_all(stage, &[ring], 0.5)?;
        line.wait_to_end(stage)?;
        pause(stage, 1.2)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

/// Turn line `line`'s count green on `board`: it is settled.
fn green_count(
    kit: &Kit,
    stage: &mut Stage<'_>,
    board: &Board,
    s: &model::Shadows,
    line: Line,
    row_counts: &mut [Mob],
    col_counts: &mut [Mob],
) -> fmn::Result<()> {
    match line {
        Line::Row(r) => recount(kit, stage, row_counts, r, s.rows[r], palette::matches(), board.row_count_at(r)),
        Line::Col(c) => recount(kit, stage, col_counts, c, s.cols[c], palette::matches(), board.col_count_at(c)),
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
            ("02_1", "tiny crystals"),
            ("02_2", "squares you can be sure of"),
            ("02_3", "Rows of"),
            ("02_3", "columns of"),
            ("02_4", "The row of five"),
            ("02_4", "the column of five"),
            ("02_5", "the first and last columns"),
            ("02_5", "the rows of one"),
            ("02_6", "exactly two squares"),
            ("02_6", "the picture is the tree"),
            ("02_7", "Its empty row"),
            ("02_9", "Every other square"),
            ("02_10", "Whatever you guess"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The words on screen are the facts': the guess's counts; the card
    /// names its source and keeps the video's scope in two whole lines, held
    /// under twelve seconds.
    #[test]
    fn the_words_on_screen_are_built_from_the_facts() {
        let f = facts::facts().unwrap();
        assert_eq!(f.get("DT-F12.face.guess"), Some("12,52"));
        assert_eq!(guess_text(12, 64), ["filled: 12 of 64".to_string(), "empty: 52 of 64".to_string()]);
        assert_eq!(FILLED_IN, format!("filled in how many of the {}", f.get("DT-F3.fits").unwrap()));
        let claim = crystal_claim();
        for part in ["Batenburg, 2006", "section 1.9.1", "flat", "exact", "no microscope"] {
            assert!(claim.contains(part), "{part}: {claim}");
        }
        assert!(!claim.contains('§'), "no section sign in the probed glyphs");
        assert!(read_s(&claim) < 12.0, "{}", read_s(&claim));
        for width in (76..=92).step_by(4) {
            let lines = wrap(&claim, width, 3);
            assert!(!lines.iter().any(|l| l.ends_with('…')), "{width}: {lines:?}");
        }
    }

    /// The chapter's layout keeps its pieces apart and inside the
    /// title-safe area (about 0.0045 units a point a character).
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        let board = Board::new(AT, CELL, 5, 5);
        let safe_x = FRAME_X_RADIUS - 0.25;
        let safe_y = FRAME_Y_RADIUS - 0.25;
        let width = |text: &str, size: f64| text.chars().count() as f64 * size * 0.0045;
        // the title stands over the grid and inside
        assert!(TITLE_Y - 0.3 > board.top() + 0.6);
        assert!(TITLE_Y + 0.3 < safe_y);
        assert!(width(TITLE, TITLE_SIZE) / 2.0 < safe_x);
        // the counts right of the grid clear the words; the words end inside
        assert!(board.row_count_at(0)[0] + 0.4 < TEXT_X);
        assert!(TEXT_X + width(FILLED_IN, 22.0) < safe_x);
        assert!(TEXT_X + LEGEND_TEXT_DX + width(SURE_EMPTY, 22.0) < THINK_AT[0] - 0.7);
        // the think ring and its note stay inside
        assert!(THINK_AT[0] + width("pause for more time", 18.0) / 2.0 < safe_x);
        // the column counts and the label above the grid stay over a
        // three-line card and under the title
        let card_top = -FRAME_Y_RADIUS + 0.2 + 0.3 + 3.0 * 0.36;
        assert!(board.col_count_at(0)[1] - 0.2 > card_top);
        assert!(board.top() + 0.4 + 0.2 < TITLE_Y - 0.3);
        // a share's number fits inside its square
        assert!(width("54", SHARE_SIZE) < board.cell * 0.8);
    }
}
