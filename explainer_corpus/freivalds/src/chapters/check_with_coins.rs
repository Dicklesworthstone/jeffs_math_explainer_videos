//! Chapters 03 and 04: the check with coins, and all eight tosses.
//!
//! 03, coins choose the columns. The sum of every column passes a C whose
//! slip has the entry beside it one too small (03_1, 03_2); with three
//! columns every fixed choice can be fooled, so let coins choose, after C is
//! handed over (03_3). Heads keeps a column (03_4); C's heads columns added,
//! B's, and A times that: do the two columns match? (03_5, a prediction.)
//! No: a right C would match, and row 2 does not (03_6), so C is wrong, for
//! certain, and anyone shown the toss can see it (03_7).
//!
//! 04, all eight tosses. Of the eight ways three coins land, how many miss
//! the slip? (04_1, a prediction.) Four, exactly half (04_2): the ones that
//! leave out its column (04_3). The mistake on its own, C − AB (04_4): the
//! check misses exactly when its heads columns add up to zero in every row
//! (04_5). A count is a chance when every toss is equally likely (04_6);
//! with coins heads one time in ten the slip gets past nine times in ten
//! (04_7, a prediction; 04_8).
//!
//! The column-adding steps (`keep_columns`, `add_heads`, `multiply`) and the
//! work row are shared with chapters 02 and 11.

use explainer_kit::props::{lower_third, precisely, verdict_box};
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    COIN_R, Coin, DY, ENTRY_SIZE, Grid, OPEN_NAME_Y, OPEN_XS, OPEN_Y, blank_coin, cell_box, coin_mobs, column_box,
    column_grid, fade_in_all, fade_in_lagged, fade_out_all, fade_to, fail, flash_all, fly, now, num, opening, pause,
    read_s, ring, say, shift_all, stand, swap, tex_of, think, together, toss_coins, wait_until,
};
use super::never_more_than_half::{D_X, mistake_grid};
use crate::facts::{self, ALL_HEADS, FAIR, UNFAIR};
use crate::model::{self, A, B, C, Column, FIRST_TOSS, SIDE, Toss};
use crate::palette;

pub struct CoinsChoose {
    pub kit: Kit,
}

pub struct EightTosses {
    pub kit: Kit,
}

// the chapters' fixed words (numbers are built from the model and facts)
const NAME_CR: &str = r"C\,r";
const NAME_BR: &str = r"B\,r";
const NAME_ABR: &str = r"A(B\,r)";
const EQUALS: &str = "=";
const NOT_EQUAL: &str = r"\neq";
const PLUS: &str = "+";
const HEADS_NOTE: &str = "heads keeps a column";
const PROGRAMMERS: &str = "for programmers:";
const PROGRAMMERS_NOTE: &str = "B r and C r only add columns, since r holds zeros and ones. A(B r) takes n squared \
                                multiplications, against n cubed to redo the product.";
const WHY: &str = r"A(B\,r) = (AB)\,r";
const WHY_NOTE: &str = "so a right C gives two equal columns";
const WRONG_VERDICT: &str = "C is wrong";
const PASSES_VERDICT: &str = "passes a wrong C";
const MATCH_QUESTION: &str = "Will the two columns match?";
const WRONG_ROW: &str = "holds a wrong entry";
const RECHECK: &str = "anyone can recheck:";
const MISS: &str = "miss";
const CAUGHT: &str = "caught";
const HALF: &str = "exactly half";
/// The names of the sum of every column, before r is named (chapter 02).
pub(super) const SUM_NAMES: [&str; 3] = ["A times B's sums", "B's sums", "C's sums"];
const SUM_NAME_SIZE: f64 = 18.0;
/// 03_3: why the coins choose, and its edges.
const FIXED_CARD: &str = "from two by two up, every fixed choice of columns misses some wrong C; with a single \
                          column, heads checks the product exactly";
/// 04_6: what the half guarantee assumes.
const FAIR_CARD: &str = "four of eight is a chance of 1/2 when every toss is equally likely; the half guarantee \
                         assumes fair coins, independent of each other and of C";
const UNFAIR_NOTE: &str = "heads: 1 time in 10";
const MISTAKE_NAME: &str = r"C - AB";
const IDENTITY: &str = r"C\,r - A(B\,r) = (C - AB)\,r";
const ARROW: &str = r"\to";
const CAUGHT_NOTE: &str = "not all zero: caught";

// ------------------------------------------------------------------ layout

/// The opening picture's places (`draw::opening`).
const A_X: f64 = OPEN_XS[0];
const B_X: f64 = OPEN_XS[1];
const C_X: f64 = OPEN_XS[2];
const NAME_SIZE: f64 = 34.0;
/// The coins over a matrix's columns.
const COIN_Y: f64 = 3.5;
/// The work row: the added columns C·r, B·r and A·(B·r), and their names.
pub(super) const WORK_Y: f64 = -0.85;
pub(super) const WORK_NAME_Y: f64 = -2.2;
/// How far apart the kept columns stand while they are added.
const STAGING_DX: f64 = 0.95;
/// Where A·(B·r) moves to stand beside C·r, and the signs between.
pub(super) const PAIR_X: f64 = -0.95;
pub(super) const SIGN_X: f64 = PAIR_X + (C_X - PAIR_X) / 2.0;
/// The right-hand panel, then the reason and the verdict.
pub(super) const PANEL_X: f64 = 4.85;
pub(super) const PANEL_YS: [f64; 3] = [3.0, 2.25, 1.15];
const NOTE_SIZE: f64 = 20.0;
const WHY_Y: f64 = -0.35;
pub(super) const VERDICT_Y: f64 = -1.75;
/// Under the verdict (03_7): the wrong row named, and the toss as proof.
const WRONG_ROW_Y: f64 = -2.4;
const RECHECK_YS: [f64; 2] = [-3.0, -3.4];
/// The eight tosses (04_1 to 04_8): two rows of four, three coins each.
const TOSS_XS: [f64; 4] = [-5.1, -2.5, 0.1, 2.7];
const TOSS_YS: [f64; 2] = [-0.25, -1.55];
const COIN_PITCH: f64 = 0.5;
const VERDICT_DY: f64 = -0.45;
const QUESTION_X: f64 = -1.2;
const QUESTION_Y: f64 = -2.75;
const THINK_AT: Vec3 = [5.4, -0.9, 0.0];
/// 03_5: the think ring and its question, right of the work row.
const MATCH_THINK_AT: Vec3 = [5.4, -0.9, 0.0];
/// 04_4, 04_5: the mistake's sum beside it, and the identity over it.
const SUM_X: f64 = -2.4;
const ARROW_X: f64 = -3.3;
const CAUGHT_X: f64 = -1.75;
const IDENTITY_AT: Vec3 = [-0.3, 3.45, 0.0];
/// 04_6 to 04_8: each toss's chance under it, and the total at the top.
const CHANCE_DY: f64 = -0.62;
const TOTAL_X: f64 = 0.6;
const TOTAL_YS: [f64; 2] = [2.75, 2.05];
const CHANCE_SIZE: f64 = 22.0;
/// A tails column, dimmed while the heads columns are added.
const DIMMED: f64 = 0.3;
/// How much quicker the column addition runs when it is repeated for B.
pub(super) const REPEAT_PACE: f64 = 0.6;

/// Row `i` of the work row.
pub(super) fn row_y(i: usize) -> f64 {
    WORK_Y + DY * (1.0 - i as f64)
}

/// Where toss `k` (counting order) stands: two rows of four.
fn toss_place(k: usize) -> (f64, f64) {
    (TOSS_XS[k % TOSS_XS.len()], TOSS_YS[k / TOSS_XS.len()])
}

/// The x of each of a toss's coins, about its centre `x`.
fn coin_xs(x: f64) -> Vec<f64> {
    (0..SIDE)
        .map(|j| x + (j as f64 - (SIDE as f64 - 1.0) / 2.0) * COIN_PITCH)
        .collect()
}

/// The x of each column of `g`.
pub(super) fn column_xs(g: &Grid) -> Vec<f64> {
    (0..g.cols).map(|j| g.at(0, j)[0]).collect()
}

/// The label put beside the slip's row (02_10, 04_3): the wrong entry's
/// value and the true one, "2 should be 1", so it cannot read as a
/// correction of the entry beside it. Refused unless C really differs from
/// AB there.
pub(super) fn correction() -> Result<String, String> {
    let (i, j) = facts::wrong_entry()?;
    let right = facts::true_product()[i][j];
    if C[i][j] == right {
        return Err(format!("C's entry ({i}, {j}) is right, not the slip"));
    }
    Ok(format!("{} should be {}", num(C[i][j]), num(right)))
}

/// The cancelling pair's sum beside the claim's row (03_2): "+1 + (−1) = 0".
fn cancel_text(values: &[i64]) -> String {
    let terms: Vec<String> = values
        .iter()
        .map(|&x| {
            if x < 0 {
                format!("({})", num(x))
            } else {
                format!("+{}", num(x))
            }
        })
        .collect();
    format!("{} = {}", terms.join(" + "), num(values.iter().sum()))
}

/// The toss as proof (03_7): "toss 110: row 2 gives 6 ≠ 7", as TeX.
fn recheck_tex(row: usize, a_br: i64, cr: i64) -> String {
    format!(
        r"\text{{toss {}: row {} gives }} {a_br} \neq {cr}",
        model::toss_text(&FIRST_TOSS),
        row + 1
    )
}

/// Each toss's chance (`texts`, in counting order) under it, red under the
/// tosses that miss the slip. Detached.
fn chance_marks(kit: &Kit, stage: &mut Stage<'_>, texts: &[String]) -> fmn::Result<Vec<Mob>> {
    let checks = facts::all_checks();
    let mut marks = Vec::with_capacity(texts.len());
    for (k, (text, (_, check))) in texts.iter().zip(&checks).enumerate() {
        let (x, y) = toss_place(k);
        let color = if check.passes() {
            palette::mistake()
        } else {
            Palette::dim()
        };
        marks.push(kit.tex(stage, text, CHANCE_SIZE, color, &[], v(x, y + CHANCE_DY))?);
    }
    Ok(marks)
}

/// The chance under a toss and the total line (04_6, 04_8), from the
/// chances of the tosses (`facts::toss_weights`), the misses' in red.
struct Chances {
    under: Vec<String>,
    total: String,
}

/// The chances of the slip's tosses when each coin lands heads with chance
/// `heads`. With every chance the same, the total is the count times it
/// ("4 × 1/8 = 1/2"); otherwise the misses' numerators added over the
/// common denominator. Refused unless the total is `facts::pass_chance`'s.
fn chances(heads: (u64, u64)) -> Result<Chances, String> {
    let weights = facts::toss_weights([heads; SIDE]);
    let checks = facts::all_checks();
    let under = weights
        .iter()
        .map(|&(w, d)| facts::ratio(w as i64, d as i64).map(|r| tex_of(&r)))
        .collect::<Result<Vec<_>, _>>()?;
    let missing: Vec<(u64, u64)> = checks
        .iter()
        .zip(&weights)
        .filter(|((_, ch), _)| ch.passes())
        .map(|(_, &w)| w)
        .collect();
    let (num_sum, den) = facts::pass_chance(&C, [heads; SIDE]);
    let all = tex_of(&facts::ratio(num_sum as i64, den as i64)?);
    let total = match missing.first() {
        Some(&(w, d)) if missing.iter().all(|&m| m == (w, d)) => format!(
            r"\text{{misses: }} {} \times {} = {all}",
            missing.len(),
            tex_of(&facts::ratio(w as i64, d as i64)?)
        ),
        Some(&(_, d)) => {
            let terms: Vec<String> = missing.iter().map(|&(w, _)| w.to_string()).collect();
            format!(r"\text{{misses: }} \frac{{{}}}{{{d}}} = {all}", terms.join(" + "))
        }
        None => return Err("the slip must get past some toss".into()),
    };
    if missing.iter().map(|&(w, _)| w).sum::<u64>() != num_sum {
        return Err("the misses' chances do not add up to the pass chance".into());
    }
    Ok(Chances { under, total })
}

/// Light the heads columns of `g` and dim its tails columns, at `pace` (1
/// the first time, less to repeat it). Returns the outlines.
pub(super) fn keep_columns(stage: &mut Stage<'_>, g: &Grid, toss: &Toss<SIDE>, pace: f64) -> fmn::Result<Vec<Mob>> {
    let mut boxes = Vec::new();
    let mut dims = Vec::new();
    for (j, &heads) in toss.iter().enumerate() {
        if heads {
            boxes.push(column_box(stage, g, j, palette::heads())?);
        } else {
            for m in g.column(j) {
                dims.push(fade_to(m, DIMMED, 0.4 * pace)?);
            }
        }
    }
    fade_in_all(stage, &boxes, 0.4 * pace)?;
    together(stage, dims)?;
    Ok(boxes)
}

/// Bring the tails columns of `g` back to full strength.
pub(super) fn undim(stage: &mut Stage<'_>, g: &Grid, toss: &Toss<SIDE>) -> fmn::Result<()> {
    let mut moves = Vec::new();
    for (j, &heads) in toss.iter().enumerate() {
        if !heads {
            for m in g.column(j) {
                moves.push(fade_to(m, 1.0, 0.3)?);
            }
        }
    }
    together(stage, moves)
}

/// Add the heads columns of `g` into one column under it: copies of the
/// kept entries fly down to stand side by side with plus signs between
/// them, then become `sum`, drawn as a column in `color`; at `pace` (1 the
/// first time, less to repeat it).
pub(super) fn add_heads(
    kit: &Kit,
    stage: &mut Stage<'_>,
    g: &Grid,
    toss: &Toss<SIDE>,
    sum: &Column<SIDE>,
    color: Srgb,
    pace: f64,
) -> fmn::Result<Grid> {
    let kept: Vec<usize> = (0..g.cols).filter(|&j| toss[j]).collect();
    let spread = |k: usize| (k as f64 - (kept.len() as f64 - 1.0) / 2.0) * STAGING_DX;
    let x0 = g.centre[0];
    let mut copies = Vec::new();
    let mut moves = Vec::new();
    for (k, &j) in kept.iter().enumerate() {
        for (i, row) in g.values.iter().enumerate() {
            let m = kit.text(stage, &num(row[j]), ENTRY_SIZE, color, g.at(i, j))?;
            stage.add_to_scene(m)?;
            moves.push(fly(m, v(x0 + spread(k), row_y(i)), 0.8 * pace)?);
            copies.push(m);
        }
    }
    together(stage, moves)?;
    let mut signs = Vec::new();
    for k in 1..kept.len() {
        for i in 0..g.rows {
            let x = x0 + spread(k) - STAGING_DX / 2.0;
            signs.push(kit.text(stage, PLUS, 24.0, Palette::dim(), v(x, row_y(i)))?);
        }
    }
    fade_in_all(stage, &signs, 0.3 * pace)?;
    pause(stage, 0.4 * pace)?;
    let total = column_grid(kit, stage, sum, v(x0, WORK_Y), color)?;
    copies.extend(signs);
    swap(stage, &copies, &total.mobs(), 0.5 * pace)?;
    Ok(total)
}

/// A times the column `br`, one row at a time: row `i` of A and the column
/// flash, and entry `i` of the product appears under A.
pub(super) fn multiply(
    kit: &Kit,
    stage: &mut Stage<'_>,
    a: &Grid,
    br: &Grid,
    product: &Column<SIDE>,
) -> fmn::Result<Grid> {
    let out = column_grid(kit, stage, product, v(a.centre[0], WORK_Y), palette::matrix_a())?;
    fade_in_all(stage, &out.brackets, 0.25)?;
    let column = br.column(0);
    for (i, entry) in out.entries.iter().enumerate() {
        let mut lit = a.row(i);
        lit.extend(&column);
        flash_all(stage, &lit, 0.3)?;
        fade_in_all(stage, entry, 0.2)?;
    }
    Ok(out)
}

/// The signs between two columns standing side by side in the work row: =
/// where they agree, ≠ in red where they do not. Detached.
pub(super) fn signs_between(
    kit: &Kit,
    stage: &mut Stage<'_>,
    left: &Column<SIDE>,
    right: &Column<SIDE>,
) -> fmn::Result<Vec<Mob>> {
    let mut signs = Vec::with_capacity(SIDE);
    for (i, (x, y)) in left.iter().zip(right).enumerate() {
        let (sign, color) = if x == y {
            (EQUALS, Palette::dim())
        } else {
            (NOT_EQUAL, palette::mistake())
        };
        signs.push(kit.tex(stage, sign, 30.0, color, &[], v(SIGN_X, row_y(i)))?);
    }
    Ok(signs)
}

impl SceneConstruct for CoinsChoose {
    fn name(&self) -> &str {
        "03_coins_choose"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let first = facts::first_check();
        let (claim, (ci, cj)) = facts::cancelling_claim().map_err(fail)?;
        let (wrong_row, wrong_col) = facts::wrong_entry().map_err(fail)?;
        let ab = facts::true_product();
        let fooled = model::check(&A, &B, &claim, &ALL_HEADS);
        if !fooled.passes() || first.passes() {
            return Err(fail(
                "the sum of every column must pass the cancelling claim, and the first toss catch C".into(),
            ));
        }

        // 03_1: the opening picture, the slip boxed; the 9 beside it becomes 8
        let line = say(&kit, stage, "03_1")?;
        let mut open = opening(&kit, stage, &C)?;
        let slip_box = cell_box(stage, &open.c, wrong_row, wrong_col, palette::mistake())?;
        let mut arriving = open.mobs();
        arriving.push(slip_box);
        fade_in_all(stage, &arriving, 0.5)?;
        line.wait_for(stage, "were written as eight")?;
        let was = open.c.entries[ci][cj];
        let changed = kit.text(
            stage,
            &num(claim[ci][cj]),
            ENTRY_SIZE,
            Palette::ink(),
            open.c.at(ci, cj),
        )?;
        let changed_box = cell_box(stage, &open.c, ci, cj, palette::mistake())?;
        swap(stage, &[was], &[changed], 0.4)?;
        fade_in_all(stage, &[changed_box], 0.3)?;
        flash_all(stage, &[changed], 0.5)?;
        line.wait_for(stage, "Now row two")?;
        let c_sums = column_grid(&kit, stage, &fooled.cr, v(C_X, WORK_Y), palette::claim())?;
        let ab_sums = column_grid(&kit, stage, &fooled.a_br, v(PAIR_X, WORK_Y), palette::matrix_a())?;
        let sum_names = [
            kit.text(
                stage,
                SUM_NAMES[2],
                SUM_NAME_SIZE,
                palette::claim(),
                v(C_X, WORK_NAME_Y),
            )?,
            kit.text(
                stage,
                SUM_NAMES[0],
                SUM_NAME_SIZE,
                palette::matrix_a(),
                v(PAIR_X, WORK_NAME_Y),
            )?,
        ];
        let mut arriving = c_sums.mobs();
        arriving.extend(ab_sums.mobs());
        arriving.extend(sum_names);
        fade_in_all(stage, &arriving, 0.5)?;
        let signs = signs_between(&kit, stage, &fooled.a_br, &fooled.cr)?;
        fade_in_lagged(stage, &signs, 0.6, 0.4)?;

        // 03_2: the extra one and the missing one cancel; the check passes it
        let line = say(&kit, stage, "03_2")?;
        let d = model::mistake(&claim, &ab);
        let values: Vec<i64> = model::wrong_entries(&d).iter().map(|&(i, j)| d[i][j]).collect();
        if values.iter().sum::<i64>() != 0 {
            return Err(fail("the cancelling pair must add up to zero".into()));
        }
        let cancel = kit.text_left(
            stage,
            &cancel_text(&values),
            22.0,
            Palette::hero(),
            v(C_X + open.c.half_width() + 0.3, open.c.at(wrong_row, 0)[1]),
        )?;
        fade_in_all(stage, &[cancel], 0.4)?;
        line.wait_for(stage, "this check passes")?;
        let passes = verdict_box(&kit, stage, PASSES_VERDICT, false, v(PANEL_X, VERDICT_Y))?;
        fade_in_all(stage, &[passes], 0.4)?;

        // 03_3: every fixed choice can be fooled, so coins choose, after C
        let line = say(&kit, stage, "03_3")?;
        let fixed = precisely(&kit, stage, FIXED_CARD)?;
        fade_in_all(stage, &[fixed], 0.4)?;
        let fixed_up = now(stage);
        line.wait_for(stage, "Let coins choose")?;
        let mut blanks = Vec::with_capacity(SIDE);
        for x in column_xs(&open.c) {
            blanks.push(blank_coin(&kit, stage, v(x, COIN_Y))?);
        }
        fade_in_lagged(stage, &coin_mobs(&blanks), 0.6, 0.3)?;

        // 03_4: back to the slip; the coins land heads, heads, tails
        let line = say(&kit, stage, "03_4")?;
        let back = kit.text(stage, &num(C[ci][cj]), ENTRY_SIZE, Palette::ink(), open.c.at(ci, cj))?;
        let mut leaving = c_sums.mobs();
        leaving.extend(ab_sums.mobs());
        leaving.extend(sum_names);
        leaving.extend(signs);
        leaving.extend([cancel, passes, changed_box]);
        fade_out_all(stage, &leaving, 0.4)?;
        swap(stage, &[changed], &[back], 0.4)?;
        // the grid holds the entry now on screen, so dimming its column
        // never brings back the one that left
        open.c.entries[ci][cj] = back;
        line.wait_for(stage, "Toss a coin")?;
        let c_coins = toss_coins(&kit, stage, &FIRST_TOSS, &column_xs(&open.c), COIN_Y)?;
        swap(stage, &coin_mobs(&blanks), &coin_mobs(&c_coins), 0.5)?;
        let legend = kit.text_left(stage, HEADS_NOTE, NOTE_SIZE, palette::heads(), v(C_X + 1.15, COIN_Y))?;
        fade_in_all(stage, &[legend], 0.3)?;

        // 03_5: C's heads columns, B's, A times that; do they match? The
        // think time, with no sign between the columns
        let line = say(&kit, stage, "03_5")?;
        // the card that stood since 03_3 has been read before the next one
        wait_until(stage, fixed_up + read_s(FIXED_CARD))?;
        fade_out_all(stage, &[fixed], 0.3)?;
        let c_kept = keep_columns(stage, &open.c, &FIRST_TOSS, 1.0)?;
        let cr = add_heads(&kit, stage, &open.c, &FIRST_TOSS, &first.cr, palette::claim(), 1.0)?;
        let name_cr = kit.tex(stage, NAME_CR, NAME_SIZE, palette::claim(), &[], v(C_X, WORK_NAME_Y))?;
        fade_in_all(stage, &[name_cr], 0.3)?;
        line.wait_for(stage, "then B's")?;
        let b_coins = toss_coins(&kit, stage, &FIRST_TOSS, &column_xs(&open.b), COIN_Y)?;
        fade_in_lagged(stage, &coin_mobs(&b_coins), 0.4, 0.3)?;
        let b_kept = keep_columns(stage, &open.b, &FIRST_TOSS, REPEAT_PACE)?;
        let br = add_heads(
            &kit,
            stage,
            &open.b,
            &FIRST_TOSS,
            &first.br,
            palette::matrix_b(),
            REPEAT_PACE,
        )?;
        let name_br = kit.tex(stage, NAME_BR, NAME_SIZE, palette::matrix_b(), &[], v(B_X, WORK_NAME_Y))?;
        let code = lower_third(&kit, stage, PROGRAMMERS, PROGRAMMERS_NOTE)?;
        fade_in_all(stage, &[name_br, code], 0.3)?;
        let code_up = now(stage);
        line.wait_for(stage, "multiply B's by A")?;
        let a_br = multiply(&kit, stage, &open.a, &br, &first.a_br)?;
        let name_abr = kit.tex(
            stage,
            NAME_ABR,
            NAME_SIZE,
            palette::matrix_a(),
            &[],
            v(A_X, WORK_NAME_Y),
        )?;
        let still = kit.text(
            stage,
            &format!("still {} multiplications", first.multiplications),
            22.0,
            Palette::ink(),
            v(PANEL_X, PANEL_YS[1]),
        )?;
        fade_in_all(stage, &[name_abr, still], 0.3)?;
        line.wait_for(stage, "Will the two columns")?;
        let question = kit.text(stage, MATCH_QUESTION, 24.0, Palette::ink(), v(PANEL_X, PANEL_YS[2]))?;
        fade_in_all(stage, &[question], 0.4)?;
        let q = think(&kit, stage, MATCH_THINK_AT)?;

        // 03_6: no; a right C would match, for the same reason as before
        let line = say(&kit, stage, "03_6")?;
        fade_out_all(stage, &[q, question, still], 0.2)?;
        let mut leaving = br.mobs();
        leaving.push(name_br);
        leaving.extend(coin_mobs(&b_coins));
        leaving.extend(b_kept);
        fade_out_all(stage, &leaving, 0.4)?;
        undim(stage, &open.b, &FIRST_TOSS)?;
        let mut pair = a_br.mobs();
        pair.push(name_abr);
        shift_all(stage, &pair, PAIR_X - A_X, 0.0, 0.8)?;
        let a_br = a_br.moved(PAIR_X - A_X);
        let why = kit.tex(stage, WHY, 34.0, Palette::ink(), &[], v(PANEL_X, WHY_Y))?;
        let why_note = kit.text(stage, WHY_NOTE, NOTE_SIZE, Palette::dim(), v(PANEL_X, WHY_Y - 0.55))?;
        fade_in_all(stage, &[why, why_note], 0.5)?;
        line.wait_for(stage, "But in row two")?;
        let signs = signs_between(&kit, stage, &first.a_br, &first.cr)?;
        fade_in_lagged(stage, &signs, 0.6, 0.4)?;
        let mut differ = Vec::new();
        for i in first.disagreements() {
            differ.push(cell_box(stage, &a_br, i, 0, palette::mistake())?);
            differ.push(cell_box(stage, &cr, i, 0, palette::mistake())?);
        }
        fade_in_all(stage, &differ, 0.3)?;

        // 03_7: C is wrong, for certain; the row is a wrong row; the toss is
        // proof anyone can recheck
        let line = say(&kit, stage, "03_7")?;
        wait_until(stage, code_up + read_s(PROGRAMMERS_NOTE))?;
        fade_out_all(stage, &[code], 0.3)?;
        let rows = first.disagreements();
        let [row] = rows[..] else {
            return Err(fail("the first toss must disagree in exactly one row".into()));
        };
        let verdict = verdict_box(&kit, stage, WRONG_VERDICT, true, v(PANEL_X, VERDICT_Y))?;
        let wrong_row_text = kit.text(
            stage,
            &format!("row {} {WRONG_ROW}", row + 1),
            NOTE_SIZE,
            palette::mistake(),
            v(PANEL_X, WRONG_ROW_Y),
        )?;
        fade_in_all(stage, &[verdict, wrong_row_text], 0.4)?;
        line.wait_for(stage, "Show anyone this toss")?;
        let proof = [
            kit.tex(
                stage,
                &recheck_tex(row, first.a_br[row], first.cr[row]),
                26.0,
                Palette::ink(),
                &[],
                v(0.0, RECHECK_YS[0]),
            )?,
            kit.text(
                stage,
                &format!("{RECHECK} {} multiplications", first.multiplications),
                22.0,
                palette::caught(),
                v(0.0, RECHECK_YS[1]),
            )?,
        ];
        fade_in_lagged(stage, &proof, 0.6, 0.4)?;
        let up = now(stage);
        stand(&kit, stage, up)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

impl SceneConstruct for EightTosses {
    fn name(&self) -> &str {
        "04_eight_tosses"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (wrong_row, wrong_col) = facts::wrong_entry().map_err(fail)?;
        let fair = chances(FAIR).map_err(fail)?;
        let unfair = chances(UNFAIR).map_err(fail)?;
        let checks = facts::all_checks();

        // 04_1: that was one toss of eight; the question, and the think time
        let line = say(&kit, stage, "04_1")?;
        let open = opening(&kit, stage, &C)?;
        let slip_box = cell_box(stage, &open.c, wrong_row, wrong_col, palette::mistake())?;
        let mut arriving = open.mobs();
        arriving.push(slip_box);
        fade_in_all(stage, &arriving, 0.5)?;
        line.wait_for(stage, "Of the eight ways")?;
        let mut tosses: Vec<Vec<Coin>> = Vec::with_capacity(checks.len());
        for (k, (toss, _)) in checks.iter().enumerate() {
            let (x, y) = toss_place(k);
            tosses.push(toss_coins(&kit, stage, toss, &coin_xs(x), y)?);
        }
        let all_coins: Vec<Mob> = tosses.iter().flat_map(|t| coin_mobs(t.as_slice())).collect();
        fade_in_lagged(stage, &all_coins, 1.6, 0.15)?;
        let question = kit.text(
            stage,
            &format!("How many of these {} tosses miss the slip?", checks.len()),
            28.0,
            Palette::ink(),
            v(QUESTION_X, QUESTION_Y),
        )?;
        fade_in_all(stage, &[question], 0.4)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 04_2: every toss checked; four miss, exactly half
        let line = say(&kit, stage, "04_2")?;
        fade_out_all(stage, &[q], 0.2)?;
        let mut verdicts = Vec::with_capacity(checks.len());
        for (k, (_, check)) in checks.iter().enumerate() {
            let (x, y) = toss_place(k);
            let (word, color) = if check.passes() {
                (MISS, palette::mistake())
            } else {
                (CAUGHT, palette::caught())
            };
            verdicts.push(kit.text(stage, word, 22.0, color, v(x, y + VERDICT_DY))?);
        }
        fade_in_lagged(stage, &verdicts, 1.2, 0.3)?;
        line.wait_for(stage, "Four of them")?;
        let passing = checks.iter().filter(|(_, ch)| ch.passes()).count();
        let share = facts::ratio(passing as i64, checks.len() as i64).map_err(fail)?;
        if share != "1/2" {
            return Err(fail(format!("{share} of the tosses miss, not half")));
        }
        let answer = kit.text(
            stage,
            &format!("{passing} of {} miss: {HALF}", checks.len()),
            30.0,
            Palette::ink(),
            v(QUESTION_X, QUESTION_Y),
        )?;
        fade_out_all(stage, &[question], 0.2)?;
        fade_in_all(stage, &[answer], 0.2)?;

        // 04_3: the slip, outlined on its own and named by its value beside
        // its row, inside its column (picked out in gold), whose coin
        // decides every toss
        let line = say(&kit, stage, "04_3")?;
        let wrong_box = column_box(stage, &open.c, wrong_col, Palette::hero())?;
        let should = kit.text_left(
            stage,
            &correction().map_err(fail)?,
            22.0,
            palette::mistake(),
            v(C_X + open.c.half_width() + 0.3, open.c.at(wrong_row, wrong_col)[1]),
        )?;
        fade_in_all(stage, &[wrong_box, should], 0.4)?;
        line.wait_for(stage, "leaves out")?;
        let mut rings = Vec::with_capacity(checks.len());
        let mut caught_rings = Vec::new();
        for (coins, (_, check)) in tosses.iter().zip(&checks) {
            let color = if check.passes() {
                palette::mistake()
            } else {
                palette::caught()
            };
            let r = ring(stage, coins[wrong_col].at, COIN_R + 0.07, color);
            if !check.passes() {
                caught_rings.push(r);
            }
            rings.push(r);
        }
        fade_in_lagged(stage, &rings, 0.8, 0.3)?;
        line.wait_for(stage, "When that coin lands heads")?;
        caught_rings.push(slip_box);
        flash_all(stage, &caught_rings, 0.6)?;

        // 04_4: the mistake on its own, apart from C
        let line = say(&kit, stage, "04_4")?;
        let mut leaving = open.mobs();
        leaving.extend(all_coins);
        leaving.extend(verdicts);
        leaving.extend(rings);
        leaving.extend([answer, wrong_box, should, slip_box]);
        fade_out_all(stage, &leaving, 0.5)?;
        let slip = model::mistake(&C, &facts::true_product());
        let d = mistake_grid(&kit, stage, &slip)?;
        let name = kit.tex(stage, MISTAKE_NAME, 34.0, palette::mistake(), &[], v(D_X, OPEN_NAME_Y))?;
        let mut arriving = d.mobs();
        arriving.push(name);
        line.wait_for(stage, "C minus the true product")?;
        fade_in_all(stage, &arriving, 0.6)?;

        // 04_5: what the check sees of it: the mistake's heads columns
        let line = say(&kit, stage, "04_5")?;
        let xs: Vec<f64> = (0..SIDE).map(|j| d.grid.at(0, j)[0]).collect();
        let coins = toss_coins(&kit, stage, &FIRST_TOSS, &xs, COIN_Y)?;
        fade_in_lagged(stage, &coin_mobs(&coins), 0.6, 0.3)?;
        let mut kept = Vec::new();
        for (j, &heads) in FIRST_TOSS.iter().enumerate() {
            if heads {
                kept.push(column_box(stage, &d.grid, j, palette::heads())?);
            }
        }
        fade_in_all(stage, &kept, 0.3)?;
        let sum = column_grid(
            &kit,
            stage,
            &model::kept_sum(&slip, &FIRST_TOSS),
            v(SUM_X, OPEN_Y),
            Palette::ink(),
        )?;
        let arrow = kit.tex(stage, ARROW, 34.0, Palette::dim(), &[], v(ARROW_X, OPEN_Y))?;
        let identity = kit.tex(stage, IDENTITY, 30.0, Palette::ink(), &[], IDENTITY_AT)?;
        let mut arriving = sum.mobs();
        arriving.extend([arrow, identity]);
        fade_in_all(stage, &arriving, 0.4)?;
        if model::hides(&slip, &FIRST_TOSS) {
            return Err(fail("the first toss must catch the slip".into()));
        }
        line.wait_for(stage, "So the check misses")?;
        let caught = kit.text_left(stage, CAUGHT_NOTE, 22.0, palette::caught(), v(CAUGHT_X, OPEN_Y))?;
        fade_in_all(stage, &[caught], 0.4)?;

        // 04_6: a count is a chance when every toss is equally likely
        let line = say(&kit, stage, "04_6")?;
        let mut leaving = coin_mobs(&coins);
        leaving.extend(kept);
        leaving.extend(sum.mobs());
        leaving.extend([arrow, identity, caught]);
        fade_out_all(stage, &leaving, 0.4)?;
        let mut tosses: Vec<Vec<Coin>> = Vec::with_capacity(checks.len());
        for (k, (toss, _)) in checks.iter().enumerate() {
            let (x, y) = toss_place(k);
            tosses.push(toss_coins(&kit, stage, toss, &coin_xs(x), y)?);
        }
        let all_coins: Vec<Mob> = tosses.iter().flat_map(|t| coin_mobs(t.as_slice())).collect();
        fade_in_lagged(stage, &all_coins, 1.0, 0.15)?;
        let under = chance_marks(&kit, stage, &fair.under)?;
        fade_in_lagged(stage, &under, 0.8, 0.2)?;
        let total = kit.tex_left(stage, &fair.total, 28.0, Palette::ink(), &[], v(TOTAL_X, TOTAL_YS[0]))?;
        let card = precisely(&kit, stage, FAIR_CARD)?;
        fade_in_all(stage, &[total, card], 0.4)?;
        let card_up = now(stage);

        // 04_7: coins heads one time in ten; the chances and their fair
        // total go blank; the question, and the think time
        let line = say(&kit, stage, "04_7")?;
        let mut fair_values = under;
        fair_values.push(total);
        fade_out_all(stage, &fair_values, 0.3)?;
        let note = kit.text_left(stage, UNFAIR_NOTE, 24.0, Palette::hero(), v(TOTAL_X, TOTAL_YS[1]))?;
        fade_in_all(stage, &[note], 0.3)?;
        line.wait_for(stage, "How often")?;
        wait_until(stage, card_up + read_s(FAIR_CARD))?;
        fade_out_all(stage, &[card], 0.2)?;
        let question = kit.text(
            stage,
            "Heads 1 time in 10: how often does the slip get past?",
            26.0,
            Palette::ink(),
            v(QUESTION_X, QUESTION_Y),
        )?;
        fade_in_all(stage, &[question], 0.3)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 04_8: nine times in ten, computed; the half guarantee uses fair
        // coins
        let line = say(&kit, stage, "04_8")?;
        fade_out_all(stage, &[q, question], 0.2)?;
        let under = chance_marks(&kit, stage, &unfair.under)?;
        fade_in_lagged(stage, &under, 0.8, 0.2)?;
        let unfair_total = kit.tex_left(
            stage,
            &unfair.total,
            28.0,
            palette::mistake(),
            &[],
            v(TOTAL_X, TOTAL_YS[0]),
        )?;
        fade_in_all(stage, &[unfair_total], 0.4)?;
        line.wait_for(stage, "Our half guarantee")?;
        let (fair_num, fair_den) = facts::pass_chance(&C, [FAIR; SIDE]);
        let fair_share = facts::ratio(fair_num as i64, fair_den as i64).map_err(fail)?;
        let fair_line = kit.text(
            stage,
            &format!("fair coins: {fair_share}"),
            24.0,
            palette::caught(),
            v(QUESTION_X, QUESTION_Y),
        )?;
        fade_in_all(stage, &[fair_line], 0.4)?;
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

    /// The words on screen say what the model says.
    #[test]
    fn the_screen_words_are_built_from_the_model() {
        // the slip's label names its own value, not the 9 beside it
        assert_eq!(correction().unwrap(), "2 should be 1");
        assert_eq!(cancel_text(&[1, -1]), "+1 + (\u{2212}1) = 0");
        let fair = chances(FAIR).unwrap();
        assert_eq!(fair.under, vec![r"\frac{1}{8}".to_owned(); 8]);
        assert_eq!(fair.total, r"\text{misses: } 4 \times \frac{1}{8} = \frac{1}{2}");
        let unfair = chances(UNFAIR).unwrap();
        assert_eq!(unfair.under[0], r"\frac{729}{1000}");
        assert_eq!(unfair.under[7], r"\frac{1}{1000}");
        assert_eq!(
            unfair.total,
            r"\text{misses: } \frac{729 + 81 + 81 + 9}{1000} = \frac{9}{10}"
        );
        assert_eq!(SUM_NAMES, ["A times B's sums", "B's sums", "C's sums"]);
    }

    /// The tosses the screen marks "miss" are the ones whose coin over the
    /// wrong entry's column is tails, and they are half.
    #[test]
    fn the_misses_marked_are_tails_over_the_wrong_column() {
        let (_, col) = facts::wrong_entry().unwrap();
        let checks = facts::all_checks();
        let marked: Vec<bool> = checks.iter().map(|(_, ch)| ch.passes()).collect();
        let tails: Vec<bool> = checks.iter().map(|(t, _)| !t[col]).collect();
        assert_eq!(marked, tails);
        assert_eq!(2 * marked.iter().filter(|&&m| m).count(), checks.len());
    }

    /// The cards keep their conditions in two lines that can be read.
    #[test]
    fn the_cards_fit_their_lines() {
        assert!(FIXED_CARD.contains("from two by two up"), "{FIXED_CARD}");
        assert!(
            FIXED_CARD.contains("single column, heads checks the product exactly"),
            "{FIXED_CARD}"
        );
        assert!(FAIR_CARD.contains("assumes fair coins"), "{FAIR_CARD}");
        assert!(FAIR_CARD.contains("independent of each other and of C"), "{FAIR_CARD}");
        for text in [FIXED_CARD, FAIR_CARD, PROGRAMMERS_NOTE] {
            for width in [80, 76] {
                let lines = explainer_kit::animatic::wrap(text, width, 3);
                assert!(lines.len() <= 2, "{} lines at {width}: {text}", lines.len());
            }
            assert!(read_s(text) < 12.0, "{text}");
            assert!(!text.contains('^'), "{text}");
        }
    }

    /// Every phrase the scenes wait for is in its line of script.tsv (a
    /// changed line fails here, not in the middle of a render).
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("03_1", "were written as eight"),
            ("03_1", "Now row two"),
            ("03_2", "this check passes"),
            ("03_3", "Let coins choose"),
            ("03_4", "Toss a coin"),
            ("03_5", "then B's"),
            ("03_5", "multiply B's by A"),
            ("03_5", "Will the two columns"),
            ("03_6", "But in row two"),
            ("03_7", "Show anyone this toss"),
            ("04_1", "Of the eight ways"),
            ("04_2", "Four of them"),
            ("04_3", "leaves out"),
            ("04_3", "When that coin lands heads"),
            ("04_4", "C minus the true product"),
            ("04_5", "So the check misses"),
            ("04_7", "How often"),
            ("04_8", "Our half guarantee"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The layout keeps its pieces apart and inside the title-safe area.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        use super::super::draw::{BRACKET_GAP, DX, cell};
        use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};
        // the coins sit over C's top row, under the top edge
        const { assert!(COIN_Y + COIN_R <= FRAME_Y_RADIUS - 0.15, "coins inside the safe area") };
        const { assert!(OPEN_Y + 1.5 * DY + 0.2 < COIN_Y - COIN_R, "coins clear of the top row") };
        // the work row clears the names above it and a two-line card below
        const { assert!(WORK_Y + 1.5 * DY + 0.2 < OPEN_NAME_Y - 0.2, "work row under the names") };
        const {
            assert!(
                WORK_NAME_Y - 0.2 > -FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36 * 2.0,
                "names over a two-line card"
            )
        };
        // A·(B·r) and C·r leave room for the signs between them
        const {
            assert!(
                C_X - PAIR_X > 2.0 * (DX / 2.0 + BRACKET_GAP) + 0.6,
                "room for the signs"
            )
        };
        const { assert!(STAGING_DX > DX) };
        // the proof under the verdict, inside the bottom edge
        const { assert!(WRONG_ROW_Y < VERDICT_Y - 0.4, "row under the verdict") };
        const { assert!(RECHECK_YS[1] - 0.2 > -FRAME_Y_RADIUS + 0.15, "proof inside the frame") };
        // the tosses fit the frame, and stay left of the think ring
        let last = toss_place(7).0 + COIN_PITCH + COIN_R;
        assert!(last < THINK_AT[0] - 0.62 - 0.3);
        assert!(toss_place(0).0 - COIN_PITCH - COIN_R > -FRAME_X_RADIUS + 0.15);
        // a chance under a toss clears the next row's coins
        const {
            assert!(
                TOSS_YS[0] + CHANCE_DY - 0.3 > TOSS_YS[1] + COIN_R,
                "chances over the next row"
            )
        };
        // A's left column stays inside the title-safe edge
        assert!(cell(v(A_X, OPEN_Y), 3, 3, 0, 0)[0] - 0.3 > -FRAME_X_RADIUS + 0.25);
        // the mistake (04_4) stands where chapter 05 draws it
        const {
            assert!(
                D_X - 1.5 * DX - BRACKET_GAP > -FRAME_X_RADIUS + 0.15,
                "mistake inside the left edge"
            )
        };
    }
}
