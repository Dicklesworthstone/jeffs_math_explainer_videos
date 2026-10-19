//! Chapter 01, a check with coins. Someone hands over C as the product of A
//! and B (01_1); redoing it is nine entries of three multiplications each,
//! a billion at a thousand rows and columns (01_2); one spot check misses a
//! single wrong entry eight times in nine (01_3). Toss a coin for each
//! column and add up C's heads columns (01_4), do the same to B and
//! multiply by A (01_5): if C were right the two columns would match, and
//! they do not (01_6). Of the eight tosses, how many miss? (01_7, a
//! prediction.) Four, exactly half (01_8): the ones that leave out the
//! wrong entry's column (01_9). No wrong answer hides from more than half
//! the tosses, so toss again: twenty fresh tosses all miss less than once
//! in a million (01_10).

use explainer_kit::props::{lower_third, precisely, verdict_box};
use explainer_kit::{Kit, Palette, fade_all, rect, thousands, v};
use fmn::prelude::*;

use super::draw::{
    COIN_R, Coin, DY, ENTRY_SIZE, Grid, cell_box, coin_mobs, column_box, column_grid, fade_in_all, fade_in_lagged,
    fade_out_all, fade_to, fail, flash_all, flat, fly, matrix_grid, num, pause, ring, row_box, say, shift_all, stand,
    swap, think, together, toss_coins, wait_until,
};
use crate::facts::{self, AGAIN, BIG_SIDE, MILLION};
use crate::model::{self, A, B, C, Column, FIRST_TOSS, SIDE, SPOT, Toss};
use crate::palette;

pub struct CheckWithCoins {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model and facts)
const NAME_A: &str = "A";
const NAME_B: &str = "B";
const NAME_C: &str = "C";
const TIMES: &str = r"\times";
const EQUALS: &str = "=";
const NOT_EQUAL: &str = r"\neq";
const PLUS: &str = "+";
const ASK: &str = "?";
const REDO_NOTE: &str = "multiplications to redo the product";
const BIG_NOTE: &str = "for a thousand rows and columns";
const SCHOOLBOOK: &str = "the schoolbook method takes n cubed multiplications for two n by n matrices. Faster \
                          methods exist, but none is known that needs only about n squared steps.";
const SPOT_RANDOM: &str = "an entry picked at random";
const SPOT_MISS: &str = "misses a single wrong one";
const HEADS_NOTE: &str = "heads keeps a column";
const NAME_CR: &str = r"C\,r";
const NAME_BR: &str = r"B\,r";
const NAME_ABR: &str = r"A(B\,r)";
const PROGRAMMERS: &str = "for programmers:";
const PROGRAMMERS_NOTE: &str = "B r and C r only add columns, since r holds zeros and ones. A(B r) takes n squared \
                                multiplications, against n cubed to redo the product.";
const WHY: &str = r"A(B\,r) = (AB)\,r";
const WHY_NOTE: &str = "so a right C gives two equal columns";
const WRONG_VERDICT: &str = "C is wrong";
const MISS: &str = "miss";
const CAUGHT: &str = "caught";
const HALF: &str = "exactly half";

// ------------------------------------------------------------------ layout

/// The three matrices (01_1), their names under them, and the signs between.
const MAT_Y: f64 = 2.2;
const A_X: f64 = -5.0;
const B_X: f64 = -1.95;
const C_X: f64 = 1.1;
const TIMES_X: f64 = -3.47;
const EQUALS_X: f64 = -0.42;
const NAME_Y: f64 = 0.95;
const NAME_SIZE: f64 = 34.0;
/// The coins over a matrix's columns (01_4, 01_5).
const COIN_Y: f64 = 3.5;
/// The work row: the added columns C·r, B·r and A·(B·r), and their names.
const WORK_Y: f64 = -0.85;
const WORK_NAME_Y: f64 = -2.2;
/// How far apart the kept columns stand while they are added.
const STAGING_DX: f64 = 0.95;
/// Where A·(B·r) moves to stand beside C·r (01_6), and the signs between.
const PAIR_X: f64 = -0.95;
const SIGN_X: f64 = PAIR_X + (C_X - PAIR_X) / 2.0;
/// The right-hand panel: the cost (01_2, 01_3, 01_5), then the reason and
/// the verdict (01_6).
const PANEL_X: f64 = 4.85;
const PANEL_YS: [f64; 3] = [3.0, 2.25, 1.15];
const PANEL_SIZE: f64 = 32.0;
const NOTE_SIZE: f64 = 20.0;
const NOTE_DY: f64 = 0.45;
const WHY_Y: f64 = -0.35;
const VERDICT_Y: f64 = -1.75;
/// The eight tosses (01_7 to 01_9): two rows of four, three coins each.
const TOSS_XS: [f64; 4] = [-5.1, -2.5, 0.1, 2.7];
const TOSS_YS: [f64; 2] = [-0.25, -1.55];
const COIN_PITCH: f64 = 0.5;
const VERDICT_DY: f64 = -0.45;
const QUESTION_X: f64 = -1.2;
const QUESTION_Y: f64 = -2.75;
const THINK_AT: Vec3 = [5.4, -0.9, 0.0];
/// The halving ladder (01_10): one, two and twenty tosses.
const LADDER: [u32; 3] = [1, 2, AGAIN];
const LADDER_YS: [f64; 3] = [0.1, -0.75, -1.6];
const LADDER_LABEL_X: f64 = -6.1;
const LADDER_X: f64 = -3.2;
/// A chance of one, as a bar's length.
const LADDER_W: f64 = 4.0;
const BAR_H: f64 = 0.3;
/// The narrowest bar the ladder draws, scene units: about a pixel at 1080p
/// (eight units are 1080 pixels there).
const MIN_BAR_W: f64 = 0.01;
const TOO_SMALL: &str = "too small to see";
/// A tails column, dimmed while the heads columns are added.
const DIMMED: f64 = 0.3;
/// How much quicker the column addition runs when it is repeated for B.
const REPEAT_PACE: f64 = 0.6;

/// Row `i` of the work row.
fn row_y(i: usize) -> f64 {
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
fn column_xs(g: &Grid) -> Vec<f64> {
    (0..g.cols).map(|j| g.at(0, j)[0]).collect()
}

/// A number with thousands separators, as TeX writes them.
fn tex_thousands(n: u64) -> String {
    thousands(n).replace(',', "{,}")
}

/// The entry 01_1 works out by hand: row 1 of A times column 3 of B,
/// "1\cdot 0 + 2\cdot 2 + 1\cdot 3 = 7".
fn entry_tex() -> String {
    let (i, j) = SPOT;
    let terms: Vec<String> = A[i]
        .iter()
        .zip(&B)
        .map(|(x, row)| format!(r"{x}\cdot {}", row[j]))
        .collect();
    let value: i64 = facts::spot_products().iter().sum();
    format!("{} = {value}", terms.join(" + "))
}

/// The exact version of 01_10: the conditions the chances need (fair coins,
/// all independent, within a toss and across repeats, and tossed after C is
/// fixed), the bound, and exact arithmetic. Refused unless the theorem's
/// per-toss bound (the pairing proof's, guarded by the finite checks) is one
/// half. The ladder beside it carries the numbers.
fn again_claim() -> Result<String, String> {
    match facts::theorem_bound()? {
        (1, 2) => Ok(
            "with fair coins, all independent and tossed after C is fixed, each toss misses any fixed \
                      wrong C with chance at most half, in exact arithmetic."
                .to_owned(),
        ),
        (num, den) => Err(format!("the per-toss bound is {num}/{den}, not half")),
    }
}

/// The ladder's bar for a chance of one in `odds`, or None when it would be
/// narrower than [`MIN_BAR_W`]: such a chance is labelled too small to see,
/// never drawn wider than it is.
fn bar_width(odds: u64) -> Option<f64> {
    let w = LADDER_W / odds as f64;
    (w >= MIN_BAR_W).then_some(w)
}

/// Seconds a lower-third needs on screen to be read: fifteen characters a
/// second, a quick reader (the video can be paused).
fn read_s(text: &str) -> f64 {
    text.chars().count() as f64 / 15.0
}

/// The bound on the ladder's row for `k` tosses, at `per` to one a toss:
/// "≤ 1/2", and for the last row the million it beats.
fn ladder_tex(k: u32, per: u64) -> Result<String, String> {
    let odds = per.pow(k);
    if k == AGAIN {
        if odds <= MILLION {
            return Err(format!("1/{odds} is not less than one in a million"));
        }
        return Ok(format!(
            r"\le \frac{{1}}{{{}}} < \frac{{1}}{{{}}}",
            tex_thousands(odds),
            tex_thousands(MILLION)
        ));
    }
    Ok(format!(r"\le \frac{{1}}{{{odds}}}"))
}

/// Light the heads columns of `g` and dim its tails columns (01_4, 01_5),
/// at `pace` (1 the first time, less to repeat it). Returns the outlines.
fn keep_columns(stage: &mut Stage<'_>, g: &Grid, toss: &Toss<SIDE>, pace: f64) -> fmn::Result<Vec<Mob>> {
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
fn undim(stage: &mut Stage<'_>, g: &Grid, toss: &Toss<SIDE>) -> fmn::Result<()> {
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

/// Add the heads columns of `g` into one column under it (01_4, 01_5):
/// copies of the kept entries fly down to stand side by side with plus
/// signs between them, then become `sum`, drawn as a column in `color`; at
/// `pace` (1 the first time, less to repeat it).
fn add_heads(
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

/// A times the column `br` (01_5), one row at a time: row `i` of A and the
/// column flash, and entry `i` of the product appears under A.
fn multiply(kit: &Kit, stage: &mut Stage<'_>, a: &Grid, br: &Grid, product: &Column<SIDE>) -> fmn::Result<Grid> {
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

impl SceneConstruct for CheckWithCoins {
    fn name(&self) -> &str {
        "01_check_with_coins"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let ab = facts::true_product();
        let first = facts::first_check();
        let (wrong_row, wrong_col) = facts::wrong_entry().map_err(fail)?;

        // 01_1: A and B, the answer C, the question; one entry by hand
        let line = say(&kit, stage, "01_1")?;
        let a = matrix_grid(&kit, stage, &A, v(A_X, MAT_Y), Palette::ink())?;
        let b = matrix_grid(&kit, stage, &B, v(B_X, MAT_Y), Palette::ink())?;
        let name_a = kit.tex(stage, NAME_A, NAME_SIZE, palette::matrix_a(), &[], v(A_X, NAME_Y))?;
        let name_b = kit.tex(stage, NAME_B, NAME_SIZE, palette::matrix_b(), &[], v(B_X, NAME_Y))?;
        let times = kit.tex(stage, TIMES, 36.0, Palette::dim(), &[], v(TIMES_X, MAT_Y))?;
        let mut arriving = a.mobs();
        arriving.extend([name_a, times]);
        fade_in_all(stage, &arriving, 0.5)?;
        let mut arriving = b.mobs();
        arriving.push(name_b);
        fade_in_all(stage, &arriving, 0.5)?;
        line.wait_for(stage, "and handed you")?;
        let c = matrix_grid(&kit, stage, &C, v(C_X, MAT_Y), Palette::ink())?;
        let name_c = kit.tex(stage, NAME_C, NAME_SIZE, palette::claim(), &[], v(C_X, NAME_Y))?;
        let equals = kit.tex(stage, EQUALS, 36.0, Palette::dim(), &[], v(EQUALS_X, MAT_Y))?;
        let mut arriving = c.mobs();
        arriving.extend([equals, name_c]);
        fade_in_all(stage, &arriving, 0.6)?;
        line.wait_for(stage, "Is C really")?;
        let ask = kit.text(stage, ASK, 34.0, Palette::hero(), v(EQUALS_X, MAT_Y + 0.55))?;
        fade_in_all(stage, &[ask], 0.3)?;
        line.wait_for(stage, "You could multiply")?;
        let (si, sj) = SPOT;
        let boxes = [
            row_box(stage, &a, si, palette::matrix_a())?,
            column_box(stage, &b, sj, palette::matrix_b())?,
            cell_box(stage, &c, si, sj, palette::claim())?,
        ];
        fade_in_all(stage, &boxes, 0.4)?;
        let by_hand = kit.tex(
            stage,
            &entry_tex(),
            PANEL_SIZE,
            Palette::ink(),
            &[],
            v(PANEL_X, PANEL_YS[0]),
        )?;
        fade_in_all(stage, &[by_hand], 0.5)?;

        // 01_2: nine entries of three multiplications; a billion at scale
        let line = say(&kit, stage, "01_2")?;
        let (_, redo) = model::product(&A, &B);
        let per_entry = facts::spot_products().len();
        let count = kit.tex(
            stage,
            &format!(r"{} \times {per_entry} = {redo}", SIDE * SIDE),
            PANEL_SIZE,
            Palette::ink(),
            &[],
            v(PANEL_X, PANEL_YS[1]),
        )?;
        let count_note = kit.text(
            stage,
            REDO_NOTE,
            NOTE_SIZE,
            Palette::dim(),
            v(PANEL_X, PANEL_YS[1] - NOTE_DY),
        )?;
        fade_in_all(stage, &[count, count_note], 0.5)?;
        line.wait_for(stage, "With a thousand")?;
        let big = kit.tex(
            stage,
            &format!(
                "{BIG_SIDE}^3 = {}",
                tex_thousands(model::product_multiplications(BIG_SIDE))
            ),
            PANEL_SIZE,
            Palette::ink(),
            &[],
            v(PANEL_X, PANEL_YS[2]),
        )?;
        let big_note = kit.text(
            stage,
            BIG_NOTE,
            NOTE_SIZE,
            Palette::dim(),
            v(PANEL_X, PANEL_YS[2] - NOTE_DY),
        )?;
        // the exact count comes with the sentence it qualifies, and stays
        // through 01_3 until the coins arrive
        let schoolbook = precisely(&kit, stage, SCHOOLBOOK)?;
        fade_in_all(stage, &[big, big_note, schoolbook], 0.5)?;

        // 01_3: one entry is cheap, and misses a single wrong entry 8 in 9
        let line = say(&kit, stage, "01_3")?;
        fade_out_all(stage, &[count, count_note, big, big_note], 0.3)?;
        let one = kit.text(
            stage,
            &format!("one entry: {per_entry} multiplications"),
            24.0,
            Palette::ink(),
            v(PANEL_X, PANEL_YS[1]),
        )?;
        fade_in_all(stage, &[one], 0.4)?;
        line.wait_for(stage, "but it misses")?;
        // the eight in nine is for an entry chosen at random: a fixed entry
        // always or never catches the slip
        let (agreeing, entries) = model::spot_checks(&A, &B, &C);
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

        // 01_4: a coin for each column of C; its heads columns added
        let line = say(&kit, stage, "01_4")?;
        let mut leaving = vec![by_hand, one, ask];
        leaving.extend(misses);
        leaving.extend(boxes);
        fade_out_all(stage, &leaving, 0.4)?;
        line.wait_for(stage, "a coin for each column")?;
        fade_out_all(stage, &[schoolbook], 0.3)?;
        let c_coins = toss_coins(&kit, stage, &FIRST_TOSS, &column_xs(&c), COIN_Y)?;
        fade_in_lagged(stage, &coin_mobs(&c_coins), 0.9, 0.3)?;
        let legend = kit.text_left(stage, HEADS_NOTE, NOTE_SIZE, palette::heads(), v(C_X + 1.15, COIN_Y))?;
        fade_in_all(stage, &[legend], 0.3)?;
        line.wait_for(stage, "add up the columns")?;
        let c_kept = keep_columns(stage, &c, &FIRST_TOSS, 1.0)?;
        let cr = add_heads(&kit, stage, &c, &FIRST_TOSS, &first.cr, palette::claim(), 1.0)?;
        let name_cr = kit.tex(stage, NAME_CR, NAME_SIZE, palette::claim(), &[], v(C_X, WORK_NAME_Y))?;
        fade_in_all(stage, &[name_cr], 0.3)?;

        // 01_5: the same toss for B, then A times that
        let line = say(&kit, stage, "01_5")?;
        // the second time, quicker: the viewer has seen it once
        let b_coins = toss_coins(&kit, stage, &FIRST_TOSS, &column_xs(&b), COIN_Y)?;
        fade_in_lagged(stage, &coin_mobs(&b_coins), 0.4, 0.3)?;
        let b_kept = keep_columns(stage, &b, &FIRST_TOSS, REPEAT_PACE)?;
        let br = add_heads(
            &kit,
            stage,
            &b,
            &FIRST_TOSS,
            &first.br,
            palette::matrix_b(),
            REPEAT_PACE,
        )?;
        let name_br = kit.tex(stage, NAME_BR, NAME_SIZE, palette::matrix_b(), &[], v(B_X, WORK_NAME_Y))?;
        let code = lower_third(&kit, stage, PROGRAMMERS, PROGRAMMERS_NOTE)?;
        fade_in_all(stage, &[name_br], 0.2)?;
        line.wait_for(stage, "then multiply by A")?;
        fade_in_all(stage, &[code], 0.3)?;
        let a_br = multiply(&kit, stage, &a, &br, &first.a_br)?;
        let name_abr = kit.tex(
            stage,
            NAME_ABR,
            NAME_SIZE,
            palette::matrix_a(),
            &[],
            v(A_X, WORK_NAME_Y),
        )?;
        fade_in_all(stage, &[name_abr], 0.3)?;
        line.wait_for(stage, "That takes")?;
        let cost = [
            kit.text(
                stage,
                &format!("this check: {} multiplications", first.multiplications),
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

        // 01_6: A·(B·r) beside C·r; they must match if C is right, and do not
        let line = say(&kit, stage, "01_6")?;
        let mut leaving = br.mobs();
        leaving.push(name_br);
        leaving.extend(coin_mobs(&b_coins));
        leaving.extend(b_kept);
        leaving.extend(cost);
        fade_out_all(stage, &leaving, 0.4)?;
        undim(stage, &b, &FIRST_TOSS)?;
        let mut pair = a_br.mobs();
        pair.push(name_abr);
        shift_all(stage, &pair, PAIR_X - A_X, 0.0, 0.8)?;
        let a_br = a_br.moved(PAIR_X - A_X);
        let why = kit.tex(stage, WHY, 34.0, Palette::ink(), &[], v(PANEL_X, WHY_Y))?;
        let why_note = kit.text(stage, WHY_NOTE, NOTE_SIZE, Palette::dim(), v(PANEL_X, WHY_Y - 0.55))?;
        fade_in_all(stage, &[why, why_note], 0.5)?;
        line.wait_for(stage, "They don't")?;
        let mut signs = Vec::new();
        for (i, (x, y)) in first.a_br.iter().zip(&first.cr).enumerate() {
            let (sign, color) = if x == y {
                (EQUALS, Palette::dim())
            } else {
                (NOT_EQUAL, palette::mistake())
            };
            signs.push(kit.tex(stage, sign, 30.0, color, &[], v(SIGN_X, row_y(i)))?);
        }
        fade_in_lagged(stage, &signs, 0.6, 0.4)?;
        let mut differ = Vec::new();
        for i in first.disagreements() {
            differ.push(cell_box(stage, &a_br, i, 0, palette::mistake())?);
            differ.push(cell_box(stage, &cr, i, 0, palette::mistake())?);
        }
        fade_in_all(stage, &differ, 0.3)?;
        line.wait_for(stage, "so C is wrong")?;
        let verdict = verdict_box(&kit, stage, WRONG_VERDICT, true, v(PANEL_X, VERDICT_Y))?;
        fade_in_all(stage, &[verdict], 0.4)?;

        // 01_7: that was one toss (its coins stay over C) of eight; the
        // question, and the think time
        let line = say(&kit, stage, "01_7")?;
        let mut leaving = cr.mobs();
        leaving.extend(a_br.mobs());
        leaving.extend([name_cr, name_abr, why, why_note, verdict, code]);
        leaving.extend(signs);
        leaving.extend(differ);
        fade_out_all(stage, &leaving, 0.5)?;
        line.wait_for(stage, "Of the eight ways")?;
        let mut leaving = coin_mobs(&c_coins);
        leaving.extend(c_kept);
        leaving.push(legend);
        fade_out_all(stage, &leaving, 0.3)?;
        undim(stage, &c, &FIRST_TOSS)?;
        let checks = facts::all_checks();
        let mut tosses: Vec<Vec<Coin>> = Vec::with_capacity(checks.len());
        for (k, (toss, _)) in checks.iter().enumerate() {
            let (x, y) = toss_place(k);
            tosses.push(toss_coins(&kit, stage, toss, &coin_xs(x), y)?);
        }
        let all_coins: Vec<Mob> = tosses.iter().flat_map(|t| coin_mobs(t.as_slice())).collect();
        fade_in_lagged(stage, &all_coins, 1.6, 0.15)?;
        let question = kit.text(
            stage,
            &format!("How many of these {} tosses miss the wrong entry?", checks.len()),
            28.0,
            Palette::ink(),
            v(QUESTION_X, QUESTION_Y),
        )?;
        fade_in_all(stage, &[question], 0.4)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 01_8: every toss checked; four miss, exactly half
        let line = say(&kit, stage, "01_8")?;
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
        swap(stage, &[question], &[answer], 0.4)?;

        // 01_9: the wrong entry, and its column's coin in every toss
        let line = say(&kit, stage, "01_9")?;
        let wrong_box = column_box(stage, &c, wrong_col, palette::mistake())?;
        let should = kit.text_left(
            stage,
            &format!("should be {}", num(ab[wrong_row][wrong_col])),
            22.0,
            palette::mistake(),
            v(C_X + c.half_width() + 0.3, c.at(wrong_row, wrong_col)[1]),
        )?;
        fade_in_all(stage, &[wrong_box, should], 0.4)?;
        line.wait_for(stage, "leaves out the column")?;
        let mut rings = Vec::with_capacity(checks.len());
        for (coins, (_, check)) in tosses.iter().zip(&checks) {
            let color = if check.passes() {
                palette::mistake()
            } else {
                palette::caught()
            };
            rings.push(ring(stage, coins[wrong_col].at, COIN_R + 0.07, color));
        }
        fade_in_lagged(stage, &rings, 0.8, 0.3)?;

        // 01_10: never more than half; toss again, and again
        let line = say(&kit, stage, "01_10")?;
        let mut leaving = all_coins;
        leaving.extend(verdicts);
        leaving.extend(rings);
        leaving.push(answer);
        fade_out_all(stage, &leaving, 0.5)?;
        // the ladder is the theorem's bound from the pairing proof, one in
        // `per` a toss, not the example's own share (which happens to meet
        // it, 01_8) and not the finite sweeps (which only check it)
        let per = match facts::theorem_bound().map_err(fail)? {
            (1, per) => per,
            (num, den) => {
                return Err(fail(format!(
                    "a per-toss bound of {num}/{den} is not one in a whole number"
                )));
            }
        };
        // the exact version comes with the sentence it makes precise, and
        // carries the condition the repeats depend on before they are said
        let claim = again_claim().map_err(fail)?;
        let exact = precisely(&kit, stage, &claim)?;
        fade_in_all(stage, &[exact], 0.4)?;
        let card_up = stage.scene().time().to_f64();
        for (n, (&k, &y)) in LADDER.iter().zip(&LADDER_YS).enumerate() {
            match n {
                1 => line.wait_for(stage, "So toss again")?,
                2 => line.wait_for(stage, "Twenty fresh tosses")?,
                _ => {}
            }
            let label = kit.text_left(
                stage,
                &format!("{k} {}", if k == 1 { "toss" } else { "tosses" }),
                26.0,
                Palette::ink(),
                v(LADDER_LABEL_X, y),
            )?;
            let track = stage.arena_mut().add(rect(
                v(LADDER_X, y - BAR_H / 2.0),
                LADDER_W,
                BAR_H,
                flat(Palette::faint(), 0.6),
            )?);
            let bound = kit.tex_left(
                stage,
                &ladder_tex(k, per).map_err(fail)?,
                30.0,
                Palette::ink(),
                &[],
                v(LADDER_X + LADDER_W + 0.3, y),
            )?;
            // the bar to scale, or, for twenty tosses, the honest note that
            // it is too small to see (painted over the track)
            let fill = match bar_width(per.pow(k)) {
                Some(w) => stage.arena_mut().add(rect(
                    v(LADDER_X, y - BAR_H / 2.0),
                    w,
                    BAR_H,
                    flat(palette::mistake(), 0.9),
                )?),
                None => kit.text_left(stage, TOO_SMALL, NOTE_SIZE, Palette::dim(), v(LADDER_X + 0.15, y))?,
            };
            fade_in_all(stage, &[label, track, fill, bound], 0.5)?;
        }
        let up = stage.scene().time().to_f64();
        stand(&kit, stage, up)?;
        wait_until(stage, card_up + read_s(&claim))?;

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
        assert_eq!(entry_tex(), r"1\cdot 0 + 2\cdot 2 + 1\cdot 3 = 7");
        // the exact version keeps the bound, its arithmetic and the
        // repetition caveat; it claims no number the ladder does not show
        let claim = again_claim().unwrap();
        assert!(claim.contains("at most half"), "{claim}");
        assert!(claim.contains("in exact arithmetic"), "{claim}");
        assert!(claim.contains("independent and tossed after C is fixed"), "{claim}");
        assert!(!claim.chars().any(|c| c.is_ascii_digit()), "{claim}");
        // the chances need fair coins, all independent, and hold for any
        // fixed wrong C (a coin heads one time in ten breaks the ladder)
        assert!(claim.contains("with fair coins, all independent"), "{claim}");
        assert!(claim.contains("any fixed wrong C"), "{claim}");
        assert_eq!(ladder_tex(1, 2).unwrap(), r"\le \frac{1}{2}");
        assert_eq!(ladder_tex(2, 2).unwrap(), r"\le \frac{1}{4}");
        assert_eq!(
            ladder_tex(AGAIN, 2).unwrap(),
            r"\le \frac{1}{1{,}048{,}576} < \frac{1}{1{,}000{,}000}"
        );
        // a check that never halves the chance would not beat a million,
        // and the ladder refuses to say it does
        assert!(ladder_tex(AGAIN, 1).is_err());
        assert_eq!(tex_thousands(1_000_000_000), "1{,}000{,}000{,}000");
    }

    /// A chance too small to see is labelled, never drawn wider than it is;
    /// the threshold is about a pixel at 1080p.
    #[test]
    fn the_ladder_draws_only_bars_it_can_show() {
        use fmn::core::constants::FRAME_Y_RADIUS;
        assert_eq!(bar_width(2), Some(LADDER_W / 2.0));
        assert_eq!(bar_width(4), Some(LADDER_W / 4.0));
        assert_eq!(bar_width(1 << AGAIN), None);
        const { assert!(MIN_BAR_W >= 2.0 * FRAME_Y_RADIUS / 1080.0, "a pixel at 1080p") };
    }

    /// The lower thirds keep to two lines, so the work row's names and the
    /// ladder stay clear of them, and each stays up long enough to read.
    #[test]
    fn the_lower_thirds_fit_their_lines() {
        let exact = again_claim().unwrap();
        for text in [SCHOOLBOOK, PROGRAMMERS_NOTE, exact.as_str()] {
            let lines = explainer_kit::animatic::wrap(text, 80, 3);
            assert!(lines.len() <= 2, "{} lines: {text}", lines.len());
            assert!(read_s(text) < 12.0, "{text}");
        }
    }

    /// Every phrase the scene waits for is in its line of script.tsv (a
    /// changed line fails here, not in the middle of a render).
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("01_1", "and handed you"),
            ("01_1", "Is C really"),
            ("01_1", "You could multiply"),
            ("01_2", "With a thousand"),
            ("01_2", "a billion"),
            ("01_3", "but it misses"),
            ("01_4", "a coin for each column"),
            ("01_4", "add up the columns"),
            ("01_5", "then multiply by A"),
            ("01_5", "That takes"),
            ("01_6", "They don't"),
            ("01_6", "so C is wrong"),
            ("01_7", "Of the eight ways"),
            ("01_8", "Four of them"),
            ("01_9", "leaves out the column"),
            ("01_10", "So toss again"),
            ("01_10", "Twenty fresh tosses"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
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

    /// The layout keeps its pieces apart and inside the title-safe area.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        use super::super::draw::{DX, cell};
        use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};
        // the coins sit over C's top row, under the top edge
        const { assert!(COIN_Y + COIN_R <= FRAME_Y_RADIUS - 0.15, "coins inside the safe area") };
        const { assert!(MAT_Y + 1.5 * DY + 0.2 < COIN_Y - COIN_R, "coins clear of the top row") };
        // the work row clears the names above it and the lower third below
        const { assert!(WORK_Y + 1.5 * DY + 0.2 < NAME_Y - 0.2, "work row under the names") };
        const {
            assert!(
                WORK_NAME_Y - 0.2 > -FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36 * 2.0,
                "names over a two-line card"
            )
        };
        // the ladder clears a three-line card
        const {
            assert!(
                LADDER_YS[2] - 0.4 > -FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36 * 3.0,
                "ladder over the card"
            )
        };
        // A·(B·r) and C·r leave room for the signs between them
        const { assert!(C_X - PAIR_X > 2.0 * (DX / 2.0 + 0.05) + 0.6, "room for the signs") };
        // two kept columns stand farther apart than a matrix's columns
        const { assert!(STAGING_DX > DX) };
        // the tosses fit the frame, and stay left of the think ring
        let last = toss_place(7).0 + COIN_PITCH + COIN_R;
        assert!(last < THINK_AT[0] - 0.62 - 0.3);
        assert!(toss_place(0).0 - COIN_PITCH - COIN_R > -FRAME_X_RADIUS + 0.15);
        // the ladder's labels start inside the title-safe edge
        const { assert!(LADDER_LABEL_X >= -FRAME_X_RADIUS + 0.25) };
        // A's left column stays inside the title-safe edge
        assert!(cell(v(A_X, MAT_Y), 3, 3, 0, 0)[0] - 0.3 > -FRAME_X_RADIUS + 0.25);
    }
}
