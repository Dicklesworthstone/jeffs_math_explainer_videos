//! Chapter 02, two cheap checks. Check one entry at random, three
//! multiplications (02_1); it finds a single wrong entry one time in nine
//! (02_2). At a thousand rows a million entries, and one pick misses it
//! almost every time (02_3): how many different entries for half the time?
//! (02_4, a prediction.) Half a million, half the product's cost (02_5);
//! spot checks only see what they pick (02_6). The sum of every column, only
//! additions (02_7): a right C's must equal A times B's (02_8), nine
//! multiplications, and row 2 disagrees, so C is wrong (02_9). The slip,
//! named (02_10).

use explainer_kit::props::{precisely, verdict_box};
use explainer_kit::{Kit, Palette, fade_all, rect, v};
use fmn::prelude::*;

use super::check_with_coins::{
    PAIR_X, PANEL_X, PANEL_YS, REPEAT_PACE, SUM_NAMES, VERDICT_Y, WORK_NAME_Y, add_heads, correction, keep_columns,
    multiply, signs_between,
};
use super::draw::{
    OPEN_XS, cell_box, column_box, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, flat, fly, now, num,
    opening, read_s, row_box, say, shift_all, stand, tex_thousands, texture, think, together, wait_until,
};
use crate::facts::{self, ALL_HEADS, BIG_SIDE};
use crate::model::{self, A, B, C, SPOT};
use crate::palette;

pub struct CheapChecks {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model and facts)
const SPOT_RANDOM: &str = "an entry picked at random";
const SPOT_MISS: &str = "misses a single wrong one";
const ONE_PICK: &str = "one pick misses it";
const QUESTION: [&str; 2] = ["How many different entries", "for at least half the time?"];
const WHY_LINES: [&str; 2] = [
    "if C = AB, each column of C is A times that column of B,",
    "so C's column sum is A times B's column sum",
];
const WRONG_VERDICT: &str = "C is wrong";
const SLIP: &str = "the slip";
const SUM_NAME_SIZE: f64 = 18.0;

// ------------------------------------------------------------------ layout

/// 02_3 to 02_6: the million entries, as a square of faint lines, a cell of
/// it the wrong entry and a cell the pick; the words at the right.
const SQUARE_AT: Vec3 = [-2.6, 0.4, 0.0];
const SQUARE_SIDE: f64 = 4.4;
const SQUARE_CELLS: usize = 20;
/// The wrong entry's cell (column, row), in the half the shading leaves,
/// and the pick's cell.
const SLIP_CELL: (usize, usize) = (15, 6);
const PICK_CELL: (usize, usize) = (4, 13);
const PICK_R: f64 = 0.09;
const RIGHT_X: f64 = 3.3;
const RIGHT_YS: [f64; 2] = [1.6, 1.15];
const QUESTION_YS: [f64; 2] = [0.1, -0.35];
const RIGHT_THINK_AT: Vec3 = [3.3, -1.6, 0.0];
const COUNT_X: f64 = 0.0;
const COUNT_YS: [f64; 2] = [0.25, -0.45];
/// The count's size: its longer line, about 37 characters at some 0.17 a
/// character, ends inside the right edge.
const COUNT_SIZE: f64 = 24.0;
/// 02_8: the reason, under the work row.
const WHY_YS: [f64; 2] = [-2.95, -3.4];

/// The centre of cell (`column`, `row`) of the square, counted from its top
/// left.
fn square_cell(column: usize, row: usize) -> Vec3 {
    let step = SQUARE_SIDE / SQUARE_CELLS as f64;
    let left = SQUARE_AT[0] - SQUARE_SIDE / 2.0;
    let top = SQUARE_AT[1] + SQUARE_SIDE / 2.0;
    v(left + (column as f64 + 0.5) * step, top - (row as f64 + 0.5) * step)
}

/// The exact version of 02_5, from the facts: k different random entries
/// catch one wrong entry with chance k / n², and the fewest for one half.
fn spot_card() -> Result<String, String> {
    let (entries, picks, _) = facts::spot_half(BIG_SIDE);
    if 2 * picks < entries || 2 * (picks - 1) >= entries {
        return Err(format!("{picks} of {entries} picks is not the fewest for half"));
    }
    Ok(format!(
        "k different entries picked at random catch one wrong entry with chance k / n squared; at n = {BIG_SIDE}, \
         one half needs k = {}",
        num(picks as i64)
    ))
}

impl SceneConstruct for CheapChecks {
    fn name(&self) -> &str {
        "02_cheap_checks"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (agreeing, entries) = model::spot_checks(&A, &B, &C);
        let per_entry = facts::spot_products().len();
        let (big_entries, picks, pick_mults) = facts::spot_half(BIG_SIDE);
        let redo_big = model::product_multiplications(BIG_SIDE);
        let (_, redo) = model::product(&A, &B);
        let all = facts::all_columns_check();
        let (wrong_row, wrong_col) = facts::wrong_entry().map_err(fail)?;
        let card_text = spot_card().map_err(fail)?;
        if 2 * pick_mults != redo_big || all.passes() || SLIP_CELL.0 < SQUARE_CELLS / 2 {
            return Err(fail(
                "half a million picks must cost half the product, the sum of every column must catch the slip, \
                 and the slip's cell must sit outside the shaded half"
                    .into(),
            ));
        }

        // 02_1: the opening picture; one entry, picked at random, redone
        let line = say(&kit, stage, "02_1")?;
        let open = opening(&kit, stage, &C)?;
        fade_in_all(stage, &open.mobs(), 0.5)?;
        line.wait_for(stage, "Pick one at random")?;
        let (si, sj) = SPOT;
        let boxes = [
            row_box(stage, &open.a, si, palette::matrix_a())?,
            column_box(stage, &open.b, sj, palette::matrix_b())?,
            cell_box(stage, &open.c, si, sj, palette::claim())?,
        ];
        fade_in_all(stage, &boxes, 0.4)?;
        let one = kit.text(
            stage,
            &format!("one entry: {per_entry} multiplications"),
            24.0,
            Palette::ink(),
            v(PANEL_X, PANEL_YS[1]),
        )?;
        fade_in_all(stage, &[one], 0.4)?;

        // 02_2: a random pick misses a single wrong entry 8 times in 9
        let line = say(&kit, stage, "02_2")?;
        line.wait_for(stage, "Eight times in nine")?;
        let misses = [
            kit.text(stage, SPOT_RANDOM, 22.0, Palette::ink(), v(PANEL_X, PANEL_YS[2] + 0.45))?,
            kit.text(stage, SPOT_MISS, 22.0, palette::mistake(), v(PANEL_X, PANEL_YS[2]))?,
            kit.text(
                stage,
                &format!("{agreeing} times in {entries}"),
                22.0,
                palette::mistake(),
                v(PANEL_X, PANEL_YS[2] - 0.45),
            )?,
        ];
        fade_in_all(stage, &misses, 0.4)?;

        // 02_3: a million entries; one pick misses almost every time
        let line = say(&kit, stage, "02_3")?;
        let mut leaving = open.mobs();
        leaving.extend(boxes);
        leaving.push(one);
        leaving.extend(misses);
        fade_out_all(stage, &leaving, 0.5)?;
        let square = texture(stage, SQUARE_AT, SQUARE_SIDE, SQUARE_CELLS, palette::claim())?;
        let label = kit.text(
            stage,
            &format!("{} entries", num(big_entries as i64)),
            24.0,
            Palette::ink(),
            v(SQUARE_AT[0], SQUARE_AT[1] + SQUARE_SIDE / 2.0 + 0.35),
        )?;
        let step = SQUARE_SIDE / SQUARE_CELLS as f64;
        let slip_at = square_cell(SLIP_CELL.0, SLIP_CELL.1);
        let slip_cell = stage.arena_mut().add(rect(
            v(slip_at[0] - step / 2.0, slip_at[1] - step / 2.0),
            step,
            step,
            flat(palette::mistake(), 0.95),
        )?);
        let mut arriving = square.clone();
        arriving.extend([label, slip_cell]);
        fade_in_all(stage, &arriving, 0.6)?;
        line.wait_for(stage, "one pick misses")?;
        let pick_at = square_cell(PICK_CELL.0, PICK_CELL.1);
        let pick = stage.arena_mut().add(
            Circle::new()
                .radius(PICK_R)
                .arc_center(v(pick_at[0], SQUARE_AT[1] + SQUARE_SIDE / 2.0 + 0.9))
                .style(flat(Palette::hero(), 1.0))
                .build(),
        );
        fade_in_all(stage, &[pick], 0.2)?;
        together(stage, vec![fly(pick, pick_at, 0.6)?])?;
        let missing = [
            kit.text(stage, ONE_PICK, 22.0, Palette::ink(), v(RIGHT_X, RIGHT_YS[0]))?,
            kit.text(
                stage,
                &format!("{} times in {}", num(big_entries as i64 - 1), num(big_entries as i64)),
                22.0,
                palette::mistake(),
                v(RIGHT_X, RIGHT_YS[1]),
            )?,
        ];
        fade_in_all(stage, &missing, 0.4)?;

        // 02_4: how many different entries for half the time? The think
        // time, with nothing shaded
        let line = say(&kit, stage, "02_4")?;
        line.wait_for(stage, "How many different entries")?;
        let question = [
            kit.text(stage, QUESTION[0], 22.0, Palette::ink(), v(RIGHT_X, QUESTION_YS[0]))?,
            kit.text(stage, QUESTION[1], 22.0, Palette::ink(), v(RIGHT_X, QUESTION_YS[1]))?,
        ];
        fade_in_all(stage, &question, 0.4)?;
        let q = think(&kit, stage, RIGHT_THINK_AT)?;

        // 02_5: half the square shades: half a million entries, half the
        // product's cost
        let line = say(&kit, stage, "02_5")?;
        let mut leaving = vec![q];
        leaving.extend(question);
        fade_out_all(stage, &leaving, 0.2)?;
        let shade = stage.arena_mut().add(rect(
            v(SQUARE_AT[0] - SQUARE_SIDE / 2.0, SQUARE_AT[1] - SQUARE_SIDE / 2.0),
            SQUARE_SIDE / 2.0,
            SQUARE_SIDE,
            flat(palette::claim(), 0.28),
        )?);
        let count = kit.tex_left(
            stage,
            &format!(
                r"{} \text{{ entries}} \times {} = {}",
                tex_thousands(picks),
                tex_thousands(BIG_SIDE),
                tex_thousands(pick_mults)
            ),
            COUNT_SIZE,
            Palette::ink(),
            &[],
            v(COUNT_X, COUNT_YS[0]),
        )?;
        fade_in_all(stage, &[shade, count], 0.5)?;
        line.wait_for(stage, "half the cost")?;
        let half = kit.tex_left(
            stage,
            &format!(r"\text{{half of }} {}", tex_thousands(redo_big)),
            COUNT_SIZE,
            Palette::ink(),
            &[],
            v(COUNT_X, COUNT_YS[1]),
        )?;
        let card = precisely(&kit, stage, &card_text)?;
        fade_in_all(stage, &[half, card], 0.4)?;
        let card_up = now(stage);

        // 02_6: the wrong entry, outside the shaded half
        say(&kit, stage, "02_6")?;
        flash_all(stage, &[slip_cell], 0.6)?;

        // 02_7: the sum of every column of C, additions only
        let line = say(&kit, stage, "02_7")?;
        wait_until(stage, card_up + read_s(&card_text))?;
        let mut leaving = square;
        leaving.extend([label, slip_cell, pick, shade, count, half, card]);
        leaving.extend(missing);
        fade_out_all(stage, &leaving, 0.5)?;
        let open = opening(&kit, stage, &C)?;
        fade_in_all(stage, &open.mobs(), 0.5)?;
        line.wait_for(stage, "Add up all three columns")?;
        let c_kept = keep_columns(stage, &open.c, &ALL_HEADS, 1.0)?;
        let c_sum = add_heads(&kit, stage, &open.c, &ALL_HEADS, &all.cr, palette::claim(), 1.0)?;
        let c_name = kit.text(
            stage,
            SUM_NAMES[2],
            SUM_NAME_SIZE,
            palette::claim(),
            v(OPEN_XS[2], WORK_NAME_Y),
        )?;
        fade_in_all(stage, &[c_name], 0.3)?;

        // 02_8: why a right C's sum is A times B's; B's sum, then A times it
        let line = say(&kit, stage, "02_8")?;
        let why = [
            kit.text(stage, WHY_LINES[0], 22.0, Palette::ink(), v(0.5, WHY_YS[0]))?,
            kit.text(stage, WHY_LINES[1], 22.0, Palette::ink(), v(0.5, WHY_YS[1]))?,
        ];
        fade_in_lagged(stage, &why, 0.6, 0.4)?;
        line.wait_for(stage, "So the sum of C's columns")?;
        let b_kept = keep_columns(stage, &open.b, &ALL_HEADS, REPEAT_PACE)?;
        let b_sum = add_heads(
            &kit,
            stage,
            &open.b,
            &ALL_HEADS,
            &all.br,
            palette::matrix_b(),
            REPEAT_PACE,
        )?;
        let b_name = kit.text(
            stage,
            SUM_NAMES[1],
            SUM_NAME_SIZE,
            palette::matrix_b(),
            v(OPEN_XS[1], WORK_NAME_Y),
        )?;
        fade_in_all(stage, &[b_name], 0.2)?;
        let a_sum = multiply(&kit, stage, &open.a, &b_sum, &all.a_br)?;
        let a_name = kit.text(
            stage,
            SUM_NAMES[0],
            SUM_NAME_SIZE,
            palette::matrix_a(),
            v(OPEN_XS[0], WORK_NAME_Y),
        )?;
        fade_in_all(stage, &[a_name], 0.3)?;

        // 02_9: nine multiplications; row 2 disagrees; C is wrong
        let line = say(&kit, stage, "02_9")?;
        let cost = [
            kit.text(
                stage,
                &format!("this check: {} multiplications", all.multiplications),
                22.0,
                Palette::ink(),
                v(PANEL_X, PANEL_YS[1]),
            )?,
            kit.text(
                stage,
                &format!("redoing it: {redo}"),
                22.0,
                Palette::dim(),
                v(PANEL_X, PANEL_YS[2]),
            )?,
        ];
        fade_in_all(stage, &cost, 0.4)?;
        line.wait_for(stage, "And in row two")?;
        let mut leaving = b_sum.mobs();
        leaving.push(b_name);
        leaving.extend(b_kept);
        leaving.extend(c_kept);
        leaving.extend(why);
        fade_out_all(stage, &leaving, 0.4)?;
        let mut pair = a_sum.mobs();
        pair.push(a_name);
        shift_all(stage, &pair, PAIR_X - OPEN_XS[0], 0.0, 0.8)?;
        let a_sum = a_sum.moved(PAIR_X - OPEN_XS[0]);
        let signs = signs_between(&kit, stage, &all.a_br, &all.cr)?;
        fade_in_lagged(stage, &signs, 0.6, 0.4)?;
        let mut differ = Vec::new();
        for i in all.disagreements() {
            differ.push(cell_box(stage, &a_sum, i, 0, palette::mistake())?);
            differ.push(cell_box(stage, &c_sum, i, 0, palette::mistake())?);
        }
        fade_in_all(stage, &differ, 0.3)?;
        line.wait_for(stage, "So C is wrong")?;
        let verdict = verdict_box(&kit, stage, WRONG_VERDICT, true, v(PANEL_X, VERDICT_Y))?;
        fade_in_all(stage, &[verdict], 0.4)?;

        // 02_10: the slip, outlined on its own and named by its value
        let line = say(&kit, stage, "02_10")?;
        fade_out_all(stage, &cost, 0.3)?;
        line.wait_for(stage, "should be one")?;
        let slip_box = cell_box(stage, &open.c, wrong_row, wrong_col, palette::mistake())?;
        let beside = OPEN_XS[2] + open.c.half_width() + 0.3;
        let row_at = open.c.at(wrong_row, wrong_col)[1];
        let should = kit.text_left(
            stage,
            &correction().map_err(fail)?,
            22.0,
            palette::mistake(),
            v(beside, row_at),
        )?;
        fade_in_all(stage, &[slip_box, should], 0.4)?;
        line.wait_for(stage, "Call it the slip")?;
        let named = kit.text_left(stage, SLIP, 22.0, palette::mistake(), v(beside, row_at - 0.45))?;
        fade_in_all(stage, &[named], 0.4)?;
        let up = now(stage);
        stand(&kit, stage, up)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The card and the counts say what the facts say.
    #[test]
    fn the_screen_words_are_built_from_the_facts() {
        assert_eq!(
            spot_card().unwrap(),
            "k different entries picked at random catch one wrong entry with chance k / n squared; at n = 1000, \
             one half needs k = 500,000"
        );
        assert_eq!(facts::spot_half(BIG_SIDE), (1_000_000, 500_000, 500_000_000));
        let all = facts::all_columns_check();
        assert_eq!(all.cr, [15, 16, 15]);
        assert_eq!(all.a_br, [15, 15, 15]);
        assert_eq!(all.disagreements(), [1]);
    }

    /// The card fits two lines and can be read.
    #[test]
    fn the_card_fits_its_lines() {
        let text = spot_card().unwrap();
        for width in [80, 76] {
            let lines = explainer_kit::animatic::wrap(&text, width, 3);
            assert!(lines.len() <= 2, "{} lines at {width}: {text}", lines.len());
        }
        assert!(read_s(&text) < 12.0, "{text}");
    }

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("02_1", "Pick one at random"),
            ("02_2", "Eight times in nine"),
            ("02_3", "one pick misses"),
            ("02_4", "How many different entries"),
            ("02_5", "half the cost"),
            ("02_7", "Add up all three columns"),
            ("02_8", "So the sum of C's columns"),
            ("02_9", "And in row two"),
            ("02_9", "So C is wrong"),
            ("02_10", "should be one"),
            ("02_10", "Call it the slip"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The layout keeps its pieces apart and inside the title-safe area.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};
        const CARD_2: f64 = -FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36 * 2.0;
        // the square inside the frame, its label under the top edge
        const {
            assert!(
                SQUARE_AT[0] - SQUARE_SIDE / 2.0 > -FRAME_X_RADIUS + 0.15,
                "square inside"
            )
        };
        const {
            assert!(
                SQUARE_AT[1] + SQUARE_SIDE / 2.0 + 0.6 < FRAME_Y_RADIUS - 0.15,
                "label inside"
            )
        };
        const { assert!(SQUARE_AT[1] - SQUARE_SIDE / 2.0 > CARD_2 + 0.1, "square over the card") };
        // the words right of the square, the count right of it too
        const {
            assert!(
                COUNT_X > SQUARE_AT[0] + SQUARE_SIDE / 2.0 + 0.2,
                "count right of the square"
            )
        };
        const {
            assert!(
                RIGHT_X - 2.0 > SQUARE_AT[0] + SQUARE_SIDE / 2.0,
                "words right of the square"
            )
        };
        const { assert!(RIGHT_THINK_AT[1] - 0.62 - 0.45 > CARD_2, "ring over the card") };
        // the slip's cell on the right half, the pick's on the left
        assert!(square_cell(SLIP_CELL.0, SLIP_CELL.1)[0] > SQUARE_AT[0]);
        assert!(square_cell(PICK_CELL.0, PICK_CELL.1)[0] < SQUARE_AT[0]);
        // the reason under the work row's names, inside the bottom edge
        const { assert!(WHY_YS[0] + 0.2 < WORK_NAME_Y - 0.2, "reason under the names") };
        const { assert!(WHY_YS[1] - 0.2 > -FRAME_Y_RADIUS + 0.15, "reason inside the frame") };
    }
}
