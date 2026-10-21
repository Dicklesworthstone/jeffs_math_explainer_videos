//! Chapters 05 and 06: mistakes of other shapes, and never more than half.
//!
//! 05, mistakes of other shapes. Is half special to this slip (05_1)? Two
//! in one row, both one too big, hide only from tails on both their coins
//! (05_2); one too big and one too small hide whenever their coins agree,
//! half again (05_3), the pair that fooled the sum of every column, the
//! all-heads toss (05_4). One, one and minus one in a row: how many tosses
//! does it hide from? (05_5, a prediction.) Three, not a power of two
//! (05_6). One per row and column hides only from all tails (05_7). Two can
//! hide as often as one; what matters is cancelling (05_8). Every nonzero
//! small mistake checked, at most half (05_9), but checking is not a proof
//! (05_10).
//!
//! 06, never more than half. Pick a nonzero entry of the mistake (06_1);
//! pair the tosses by its column's coin, four pairs (06_2). One pair walked
//! through: with the other coins fixed, row 2's sum is −1 (06_3); flip the
//! first coin: how much does it change? (06_4, a prediction.) By the
//! mistake's entry, one: −1 becomes 0 (06_5). Every pair the same: at most
//! one miss each (06_6). Over a column where the mistake is all zero the
//! sum never moves and pairs miss twice (06_7). Pairs cover every toss once,
//! at any size (06_8).

use explainer_kit::props::{lower_third, precisely};
use explainer_kit::{Kit, Palette, fade_all, thousands, v};
use fmn::prelude::*;

use super::draw::{
    COIN_R, Coin, Grid, cell_box, coin_mobs, column_box, fade_in_all, fade_in_lagged, fade_out_all, fade_to, fail,
    flash_all, line_style, matrix_grid, now, num, read_s, ring, row_box, say, stand, swap, think, together, toss_coins,
    wait_until,
};
use crate::facts::{self, ALL_HEADS, CHAPTER_02_SHAPES, MISSES_PER_PAIR, PROOF_SHAPE, SWEEP_3};
use crate::model::{self, A, B, C, Matrix, SIDE, Toss};
use crate::palette;

pub struct ShapesOfMistakes {
    pub kit: Kit,
}

pub struct NeverMoreThanHalf {
    pub kit: Kit,
}

// the chapters' fixed words (numbers are built from the model and facts)
const MISTAKE_NAME: &str = r"C - AB";
const SLIP: &str = "the slip";
const IN_ONE_ROW: &str = "in one row";
const PER_ROW_AND_COLUMN: &str = "per row and column";
const MISS: &str = "miss";
const CAUGHT: &str = "caught";
const CHECKED: &str = "checked:";
const ALL_HEADS_NOTE: &str = "all heads = the sum of every column";
const ANY_SIZE: &str = "any size, any whole numbers";
const GENERAL: &str = r"2^n \text{ tosses},\; 2^{n-1} \text{ pairs}";
const FIXED_NOTE: &str = "fixed: tails, heads";
const ALL_ZEROS: &str = "all zeros";

// ------------------------------------------------------------------ layout

/// The mistake's grid and its name under it (also chapter 04's, 04_4).
pub(super) const D_X: f64 = -4.9;
const MAT_Y: f64 = 2.2;
const NAME_Y: f64 = 0.95;
/// The tally of shapes, top right: one line a shape.
const TALLY_X: f64 = 2.5;
const TALLY_YS: [f64; 5] = [3.3, 2.75, 2.2, 1.65, 1.1];
const TALLY_SIZE: f64 = 20.0;
/// The eight tosses: two rows of four, three coins each, and under each
/// toss its row sum or its verdict.
const TOSS_XS: [f64; 4] = [-5.1, -2.5, 0.1, 2.7];
const TOSS_YS: [f64; 2] = [-0.25, -1.55];
const COIN_PITCH: f64 = 0.5;
const BELOW: f64 = -0.45;
const SUM_SIZE: f64 = 24.0;
const QUESTION_X: f64 = -1.2;
const QUESTION_Y: f64 = -2.75;
const THINK_AT: Vec3 = [5.4, -0.9, 0.0];
/// The pairing: a link from each top toss's sum down to its partner, and
/// the change across it beside the link.
const LINK_TOP: f64 = TOSS_YS[0] + BELOW - 0.22;
const LINK_BOTTOM: f64 = TOSS_YS[1] + COIN_R + 0.08;
const LINK_MID: f64 = LINK_BOTTOM + (LINK_TOP - LINK_BOTTOM) / 2.0;
const CHANGE_DX: f64 = 0.32;
/// A ring around a sum the mistake hides behind.
const HIDDEN_R: f64 = 0.22;
/// The count (06_8), right of the tosses, and the pairs' count (06_2) over it.
const COUNT_X: f64 = 5.1;
const GENERAL_SIZE: f64 = 26.0;
const COUNT_YS: [f64; 4] = [-0.35, -0.85, -1.35, -1.95];
const PAIRS_AT: Vec3 = [5.1, 1.0, 0.0];
/// 06_3: the fixed coins' note, over the walked pair.
const FIXED_DY: f64 = 0.55;
/// The dimmed pairs while one is walked through.
const DIMMED: f64 = 0.25;
/// 06_7: "all zeros" over the column the slip is right in.
const ZEROS_DY: f64 = 0.35;

/// Where toss `k` (counting order) stands: two rows of four. With three
/// coins the first coin is the row: toss k and toss k + 4 differ only in it,
/// so the pairs by coin 1 stand one above the other.
fn toss_place(k: usize) -> (f64, f64) {
    (TOSS_XS[k % TOSS_XS.len()], TOSS_YS[k / TOSS_XS.len()])
}

/// The x of each of a toss's coins, about its centre `x`.
fn coin_xs(x: f64) -> Vec<f64> {
    (0..SIDE)
        .map(|j| x + (j as f64 - (SIDE as f64 - 1.0) / 2.0) * COIN_PITCH)
        .collect()
}

/// The one row a mistake is wrong in, if it is wrong in only one: its sum
/// under each toss decides whether it hides.
fn single_wrong_row(d: &Matrix<SIDE>) -> Option<usize> {
    let rows: Vec<usize> = (0..SIDE).filter(|&i| d[i].iter().any(|&x| x != 0)).collect();
    match rows[..] {
        [row] => Some(row),
        _ => None,
    }
}

/// How many wrong entries a mistake has in each row and in each column,
/// when that number is the same everywhere.
fn per_row_and_column(d: &Matrix<SIDE>) -> Option<usize> {
    let rows = d.iter().map(|row| row.iter().filter(|&&x| x != 0).count());
    let cols = (0..SIDE).map(|j| d.iter().filter(|row| row[j] != 0).count());
    let counts: Vec<usize> = rows.chain(cols).collect();
    let first = *counts.first()?;
    counts.iter().all(|&c| c == first).then_some(first)
}

/// A sum, change or entry with its sign: "+1", "0", "−1".
fn signed(x: i64) -> String {
    if x > 0 { format!("+{}", num(x)) } else { num(x) }
}

/// A change across a pair as written beside its link: "+1", or "+0" where
/// the flip moves nothing (06_7).
fn change_text(x: i64) -> String {
    if x == 0 { "+0".to_owned() } else { signed(x) }
}

/// The tally line of mistake `name`: its wrong values and how many of the
/// tosses it hides from, "1, 1, −1 in one row: 3 of 8".
fn tally_text(name: &str, d: &Matrix<SIDE>) -> Result<String, String> {
    let values: Vec<String> = model::wrong_entries(d).iter().map(|&(i, j)| num(d[i][j])).collect();
    let label = match (name, per_row_and_column(d)) {
        ("one_entry", _) => format!("{SLIP} ({})", values.join(", ")),
        ("every_column", Some(per)) => format!("{per} {PER_ROW_AND_COLUMN}"),
        ("every_column", None) => return Err("the every-column shape is not one per row and column".into()),
        _ => format!("{} {IN_ONE_ROW}", values.join(", ")),
    };
    let tosses = model::tosses::<SIDE>().len();
    Ok(format!("{label}: {} of {tosses}", facts::hiding(d).len()))
}

/// Tally line `n`, left-aligned in its place. Detached.
fn tally_line(kit: &Kit, stage: &mut Stage<'_>, tallies: &[String], n: usize) -> fmn::Result<Mob> {
    kit.text_left(stage, &tallies[n], TALLY_SIZE, Palette::ink(), v(TALLY_X, TALLY_YS[n]))
}

/// The "checked" lower-third: the finite sweep of every nonzero small
/// mistake, labelled as a check of the bound, not its proof. Refused unless
/// the theorem's bound stands.
fn checked_card() -> Result<String, String> {
    facts::theorem_bound()?;
    let wrong = model::sweep(&A, &B, &SWEEP_3).wrong;
    let values: Vec<String> = SWEEP_3.iter().map(|&x| num(x)).collect();
    let (last, rest) = values.split_last().ok_or("no sweep values")?;
    Ok(format!(
        "all {} nonzero three by three mistakes with entries {} or {last} hide from at most half the tosses: a \
         finite check; the pairing proves every size.",
        thousands(wrong as u64),
        rest.join(", ")
    ))
}

/// The sweep's count under the tosses (05_9): "19,682 checked: most misses
/// 4 of 8".
fn checked_count() -> String {
    let s = model::sweep(&A, &B, &SWEEP_3);
    format!(
        "{} checked: most misses {} of {}",
        thousands(s.wrong as u64),
        s.most,
        model::tosses::<SIDE>().len()
    )
}

/// The exact version of 06_8: the proof's step with a nonzero entry of the
/// mistake, and its constant, refused unless the theorem's bound is one half.
fn pairing_card() -> Result<String, String> {
    match facts::theorem_bound()? {
        (1, 2) => Ok(format!(
            "pick an entry where the mistake C \u{2212} AB is not 0; flipping its column's coin moves its row's sum \
             by that entry, so a pair holds at most {MISSES_PER_PAIR} miss"
        )),
        (num, den) => Err(format!("the per-toss bound is {num}/{den}, not half")),
    }
}

/// The count at 06_8 for pairs holding `misses_per_pair` misses among
/// `tosses` tosses: the tosses and pairs, the proof's bound per pair (the
/// constant, never the example's most), and what that gives for all the
/// tosses. Refused if any pair holds more than the bound.
fn count_texts(misses_per_pair: &[usize], tosses: usize) -> Result<[String; 3], String> {
    if let Some(&most) = misses_per_pair.iter().find(|&&m| m > MISSES_PER_PAIR) {
        return Err(format!(
            "a pair holds {most} misses, more than the proof's {MISSES_PER_PAIR}"
        ));
    }
    let pairs = misses_per_pair.len();
    Ok([
        format!("{tosses} tosses, {pairs} pairs"),
        format!("at most {MISSES_PER_PAIR} miss in each pair"),
        format!("at most {} of {tosses}", MISSES_PER_PAIR * pairs),
    ])
}

/// A mistake drawn as a grid, its wrong entries outlined in the mistake
/// role. Detached.
pub(super) struct Mistake {
    pub(super) grid: Grid,
    pub(super) boxes: Vec<Mob>,
}

impl Mistake {
    pub(super) fn mobs(&self) -> Vec<Mob> {
        let mut all = self.grid.mobs();
        all.extend(&self.boxes);
        all
    }
}

/// The mistake `d` at the mistake's place, its nonzero entries outlined.
pub(super) fn mistake_grid(kit: &Kit, stage: &mut Stage<'_>, d: &Matrix<SIDE>) -> fmn::Result<Mistake> {
    let grid = matrix_grid(kit, stage, d, v(D_X, MAT_Y), Palette::ink())?;
    let mut boxes = Vec::new();
    for (i, j) in model::wrong_entries(d) {
        boxes.push(cell_box(stage, &grid, i, j, palette::mistake())?);
    }
    Ok(Mistake { grid, boxes })
}

/// What decides toss `t` for the mistake `d`: the sum of its one wrong row
/// (red where it is zero, so the mistake hides), or with more than one
/// wrong row the verdict itself. Detached, under the toss's place `k`.
fn under_toss(kit: &Kit, stage: &mut Stage<'_>, d: &Matrix<SIDE>, k: usize, t: &Toss<SIDE>) -> fmn::Result<Mob> {
    let (x, y) = toss_place(k);
    let hides = model::hides(d, t);
    let (words, color) = match (single_wrong_row(d), hides) {
        (Some(i), true) => (num(model::kept_sum(d, t)[i]), palette::mistake()),
        (Some(i), false) => (num(model::kept_sum(d, t)[i]), Palette::dim()),
        (None, true) => (MISS.to_owned(), palette::mistake()),
        (None, false) => (CAUGHT.to_owned(), palette::caught()),
    };
    kit.text(stage, &words, SUM_SIZE, color, v(x, y + BELOW))
}

/// [`under_toss`] for every toss, in counting order. Detached.
fn under_tosses(kit: &Kit, stage: &mut Stage<'_>, d: &Matrix<SIDE>) -> fmn::Result<Vec<Mob>> {
    let all = model::tosses::<SIDE>();
    let mut marks = Vec::with_capacity(all.len());
    for (k, t) in all.iter().enumerate() {
        marks.push(under_toss(kit, stage, d, k, t)?);
    }
    Ok(marks)
}

/// The eight tosses as coins, two rows of four. Detached.
fn all_toss_coins(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Vec<Coin>>> {
    let mut tosses = Vec::new();
    for (k, toss) in model::tosses::<SIDE>().iter().enumerate() {
        let (x, y) = toss_place(k);
        tosses.push(toss_coins(kit, stage, toss, &coin_xs(x), y)?);
    }
    Ok(tosses)
}

/// Toss `t`'s place in counting order.
fn index_of(t: &Toss<SIDE>) -> usize {
    model::tosses::<SIDE>().iter().position(|u| u == t).unwrap_or(0)
}

/// The links of the pairs by coin `column`, each from a top toss's sum down
/// to its partner. Refused unless every pair stands one above the other.
fn pair_links(stage: &mut Stage<'_>, column: usize) -> fmn::Result<Vec<Mob>> {
    let mut links = Vec::new();
    for (tails, heads) in model::coin_pairs::<SIDE>(column) {
        if index_of(&heads) != index_of(&tails) + TOSS_XS.len() {
            return Err(fail("a pair by the first coin must stand one above the other".into()));
        }
        let (x, _) = toss_place(index_of(&tails));
        let link = polyline(vec![v(x, LINK_TOP), v(x, LINK_BOTTOM)])
            .style(line_style(Palette::hero(), 2.5))
            .build();
        links.push(stage.arena_mut().add(link));
    }
    Ok(links)
}

/// The change across each pair by coin `column` in row `row` of `d`, beside
/// its link. Detached, in pair order.
fn change_labels(
    kit: &Kit,
    stage: &mut Stage<'_>,
    d: &Matrix<SIDE>,
    row: usize,
    column: usize,
) -> fmn::Result<Vec<Mob>> {
    let pairs = model::coin_pairs::<SIDE>(column);
    let mut labels = Vec::with_capacity(pairs.len());
    for ((tails, _), change) in pairs.iter().zip(model::pair_changes(d, row, column)) {
        let (x, _) = toss_place(index_of(tails));
        labels.push(kit.text_left(
            stage,
            &change_text(change),
            20.0,
            Palette::hero(),
            v(x + CHANGE_DX, LINK_MID),
        )?);
    }
    Ok(labels)
}

impl SceneConstruct for ShapesOfMistakes {
    fn name(&self) -> &str {
        "05_shapes_of_mistakes"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let ab = facts::true_product();
        let shapes: Vec<Matrix<SIDE>> = CHAPTER_02_SHAPES
            .iter()
            .map(|&name| facts::shape(name))
            .collect::<Result<_, _>>()
            .map_err(fail)?;
        let tallies: Vec<String> = CHAPTER_02_SHAPES
            .iter()
            .zip(&shapes)
            .map(|(&name, d)| tally_text(name, d))
            .collect::<Result<_, _>>()
            .map_err(fail)?;
        let all = model::tosses::<SIDE>();
        let heads_k = index_of(&ALL_HEADS);
        if !model::hides(&shapes[2], &ALL_HEADS) {
            return Err(fail("the cancelling pair must hide from all heads".into()));
        }

        // 05_1: the slip's mistake, and its tally
        let line = say(&kit, stage, "05_1")?;
        let slip = model::mistake(&C, &ab);
        if slip != shapes[0] {
            return Err(fail("chapter 05 must open on the slip".into()));
        }
        let mut d_now = mistake_grid(&kit, stage, &slip)?;
        let name = kit.tex(stage, MISTAKE_NAME, 34.0, palette::mistake(), &[], v(D_X, NAME_Y))?;
        let mut arriving = d_now.mobs();
        arriving.push(name);
        fade_in_all(stage, &arriving, 0.6)?;
        line.wait_for(stage, "Try mistakes")?;
        let mut tally_mobs = vec![tally_line(&kit, stage, &tallies, 0)?];
        fade_in_all(stage, &tally_mobs, 0.3)?;

        // 05_2: both one too big, in one row; every toss and its row sum
        let line = say(&kit, stage, "05_2")?;
        let next = mistake_grid(&kit, stage, &shapes[1])?;
        swap(stage, &d_now.mobs(), &next.mobs(), 0.4)?;
        d_now = next;
        let tosses = all_toss_coins(&kit, stage)?;
        let all_coins: Vec<Mob> = tosses.iter().flat_map(|t| coin_mobs(t.as_slice())).collect();
        fade_in_lagged(stage, &all_coins, 1.2, 0.15)?;
        let mut under = under_tosses(&kit, stage, &shapes[1])?;
        fade_in_lagged(stage, &under, 0.6, 0.3)?;
        line.wait_for(stage, "Two of eight")?;
        tally_mobs.push(tally_line(&kit, stage, &tallies, 1)?);
        fade_in_all(stage, &tally_mobs[1..], 0.3)?;

        // 05_3: one too big and one too small cancel when their coins agree
        let line = say(&kit, stage, "05_3")?;
        let next = mistake_grid(&kit, stage, &shapes[2])?;
        let next_under = under_tosses(&kit, stage, &shapes[2])?;
        let mut old = d_now.mobs();
        old.extend(&under);
        let mut new = next.mobs();
        new.extend(&next_under);
        swap(stage, &old, &new, 0.5)?;
        (d_now, under) = (next, next_under);
        line.wait_for(stage, "Four of eight")?;
        tally_mobs.push(tally_line(&kit, stage, &tallies, 2)?);
        fade_in_all(stage, &tally_mobs[2..], 0.3)?;

        // 05_4: the pair that fooled the sum of every column: the all-heads
        // toss
        let line = say(&kit, stage, "05_4")?;
        let (hx, hy) = toss_place(heads_k);
        let heads_ring = ring(stage, v(hx, hy + BELOW), HIDDEN_R, Palette::hero());
        fade_in_all(stage, &[heads_ring], 0.3)?;
        line.wait_for(stage, "which is just the toss")?;
        let heads_note = kit.text(stage, ALL_HEADS_NOTE, 22.0, Palette::hero(), v(QUESTION_X, QUESTION_Y))?;
        fade_in_all(stage, &[heads_note], 0.4)?;

        // 05_5: one, one and minus one in a row; the question, and the
        // think time with no sum on screen
        let line = say(&kit, stage, "05_5")?;
        fade_out_all(stage, &[heads_ring, heads_note], 0.3)?;
        let next = mistake_grid(&kit, stage, &shapes[3])?;
        let mut old = d_now.mobs();
        old.extend(&under);
        swap(stage, &old, &next.mobs(), 0.5)?;
        d_now = next;
        line.wait_for(stage, "How many tosses")?;
        let question = kit.text(
            stage,
            &format!("How many of these {} tosses does it hide from?", all.len()),
            28.0,
            Palette::ink(),
            v(QUESTION_X, QUESTION_Y),
        )?;
        fade_in_all(stage, &[question], 0.4)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 05_6: three, not half and not a power of two; the sums come only
        // now, after the think time
        say(&kit, stage, "05_6")?;
        fade_out_all(stage, &[q, question], 0.2)?;
        under = under_tosses(&kit, stage, &shapes[3])?;
        fade_in_lagged(stage, &under, 0.6, 0.3)?;
        tally_mobs.push(tally_line(&kit, stage, &tallies, 3)?);
        fade_in_all(stage, &tally_mobs[3..], 0.3)?;

        // 05_7: one wrong entry per row and column hides only from all tails
        let line = say(&kit, stage, "05_7")?;
        let next = mistake_grid(&kit, stage, &shapes[4])?;
        let next_under = under_tosses(&kit, stage, &shapes[4])?;
        let mut old = d_now.mobs();
        old.extend(&under);
        let mut new = next.mobs();
        new.extend(&next_under);
        swap(stage, &old, &new, 0.5)?;
        line.wait_for(stage, "only from all tails")?;
        tally_mobs.push(tally_line(&kit, stage, &tallies, 4)?);
        fade_in_all(stage, &tally_mobs[4..], 0.3)?;

        // 05_8: two wrong entries can hide as often as one; what matters is
        // whether they cancel
        let line = say(&kit, stage, "05_8")?;
        if facts::hiding(&shapes[0]).len() != facts::hiding(&shapes[2]).len() {
            return Err(fail(
                "the slip and the cancelling pair must hide from as many tosses".into(),
            ));
        }
        flash_all(stage, &[tally_mobs[0], tally_mobs[2]], 0.6)?;
        line.wait_for(stage, "What matters")?;
        flash_all(stage, &[tally_mobs[1], tally_mobs[4]], 0.6)?;

        // 05_9: every nonzero small mistake checked; a finite check
        let line = say(&kit, stage, "05_9")?;
        let card_text = checked_card().map_err(fail)?;
        let checked = lower_third(&kit, stage, CHECKED, &card_text)?;
        fade_in_all(stage, &[checked], 0.4)?;
        let card_up = now(stage);
        line.wait_for(stage, "All nineteen thousand")?;
        let count = kit.text(stage, &checked_count(), 24.0, Palette::ink(), v(QUESTION_X, QUESTION_Y))?;
        fade_in_all(stage, &[count], 0.4)?;

        // 05_10: but checking is not a proof
        let line = say(&kit, stage, "05_10")?;
        together(stage, vec![fade_to(count, 0.35, 0.4)?])?;
        line.wait_for(stage, "Mistakes can hold")?;
        let open_question = [
            kit.text(stage, "?", 40.0, Palette::hero(), v(TALLY_X - 0.5, TALLY_YS[4] - 0.75))?,
            kit.text_left(stage, ANY_SIZE, 22.0, Palette::hero(), v(TALLY_X, TALLY_YS[4] - 0.75))?,
        ];
        fade_in_all(stage, &open_question, 0.4)?;
        let up = now(stage);
        stand(&kit, stage, up)?;
        wait_until(stage, card_up + read_s(&card_text))?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

impl SceneConstruct for NeverMoreThanHalf {
    fn name(&self) -> &str {
        "06_never_more_than_half"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let ab = facts::true_product();
        let proof = facts::shape(PROOF_SHAPE).map_err(fail)?;
        let (row, column) = facts::proof_entry().map_err(fail)?;
        let (walk_tails, walk_heads) = facts::walk_pair().map_err(fail)?;
        let all = model::tosses::<SIDE>();
        let pairs = model::coin_pairs::<SIDE>(column);
        let walk = pairs
            .iter()
            .position(|(t, _)| *t == walk_tails)
            .ok_or_else(|| fail("the walked pair is not a pair".into()))?;
        let (top_k, bottom_k) = (index_of(&walk_tails), index_of(&walk_heads));
        if proof[row][column] == 0 || !model::hides(&proof, &walk_heads) {
            return Err(fail(
                "the chosen entry must be nonzero, and the walk end on a miss".into(),
            ));
        }

        // 06_1: the 1, 1, −1 mistake; pick a nonzero entry of it
        let line = say(&kit, stage, "06_1")?;
        let d = mistake_grid(&kit, stage, &proof)?;
        let name = kit.tex(stage, MISTAKE_NAME, 34.0, palette::mistake(), &[], v(D_X, NAME_Y))?;
        let tosses = all_toss_coins(&kit, stage)?;
        let all_coins: Vec<Mob> = tosses.iter().flat_map(|t| coin_mobs(t.as_slice())).collect();
        let mut arriving = d.mobs();
        arriving.push(name);
        fade_in_all(stage, &arriving, 0.6)?;
        fade_in_lagged(stage, &all_coins, 1.2, 0.15)?;
        line.wait_for(stage, "take row two, column one")?;
        let chosen = model::wrong_entries(&proof)
            .iter()
            .position(|&e| e == (row, column))
            .map(|n| d.boxes[n])
            .ok_or_else(|| fail("the chosen entry has no outline".into()))?;
        let picked = column_box(stage, &d.grid, column, Palette::hero())?;
        fade_in_all(stage, &[picked], 0.3)?;
        flash_all(stage, &[chosen], 0.6)?;

        // 06_2: pair the tosses that differ only in that column's coin
        let line = say(&kit, stage, "06_2")?;
        let coin_rings: Vec<Mob> = tosses
            .iter()
            .map(|coins| ring(stage, coins[column].at, COIN_R + 0.07, Palette::hero()))
            .collect();
        fade_in_all(stage, &coin_rings, 0.4)?;
        let links = pair_links(stage, column)?;
        fade_in_lagged(stage, &links, 0.8, 0.3)?;
        line.wait_for(stage, "that makes four pairs")?;
        let pair_count = kit.text(
            stage,
            &format!("{} pairs", pairs.len()),
            24.0,
            Palette::hero(),
            PAIRS_AT,
        )?;
        fade_in_all(stage, &[pair_count], 0.3)?;

        // 06_3: one pair, the other coins fixed; row 2's sum with the first
        // coin tails
        let line = say(&kit, stage, "06_3")?;
        let mut dims = Vec::new();
        for (k, coins) in tosses.iter().enumerate() {
            if k != top_k && k != bottom_k {
                for m in coin_mobs(coins) {
                    dims.push(fade_to(m, DIMMED, 0.4)?);
                }
                dims.push(fade_to(coin_rings[k], DIMMED, 0.4)?);
            }
        }
        for (p, &link) in links.iter().enumerate() {
            if p != walk {
                dims.push(fade_to(link, DIMMED, 0.4)?);
            }
        }
        together(stage, dims)?;
        let mut fixed_rings = Vec::new();
        for k in [top_k, bottom_k] {
            for (j, coin) in tosses[k].iter().enumerate() {
                if j != column {
                    fixed_rings.push(ring(stage, coin.at, COIN_R + 0.07, Palette::hero()));
                }
            }
        }
        let (fx, fy) = toss_place(top_k);
        let fixed_note = kit.text(stage, FIXED_NOTE, 20.0, Palette::hero(), v(fx, fy + FIXED_DY))?;
        let mut arriving = fixed_rings.clone();
        arriving.push(fixed_note);
        fade_in_all(stage, &arriving, 0.4)?;
        line.wait_for(stage, "With the first coin tails")?;
        let top_sum = under_toss(&kit, stage, &proof, top_k, &walk_tails)?;
        fade_in_all(stage, &[top_sum], 0.4)?;

        // 06_4: flip the first coin: how much does row 2's sum change? The
        // think time, the partner's sum not shown
        let line = say(&kit, stage, "06_4")?;
        line.wait_for(stage, "How much")?;
        let question = kit.text(
            stage,
            &format!(
                "Coin {} flips to heads: how much does row {}'s sum change?",
                column + 1,
                row + 1
            ),
            26.0,
            Palette::ink(),
            v(QUESTION_X, QUESTION_Y),
        )?;
        fade_in_all(stage, &[question], 0.4)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 06_5: by the mistake's entry there, one; −1 becomes 0
        let line = say(&kit, stage, "06_5")?;
        fade_out_all(stage, &[q, question], 0.2)?;
        let changes = change_labels(&kit, stage, &proof, row, column)?;
        fade_in_all(stage, &[changes[walk]], 0.3)?;
        line.wait_for(stage, "Minus one becomes zero")?;
        let bottom_sum = under_toss(&kit, stage, &proof, bottom_k, &walk_heads)?;
        fade_in_all(stage, &[bottom_sum], 0.3)?;
        line.wait_for(stage, "Flipping that coin")?;
        flash_all(stage, &[chosen, bottom_sum], 0.6)?;

        // 06_6: every pair the same; at most one miss in each
        let line = say(&kit, stage, "06_6")?;
        let mut undims = Vec::new();
        for m in all_coins.iter().chain(&coin_rings).chain(&links) {
            undims.push(fade_to(*m, 1.0, 0.4)?);
        }
        together(stage, undims)?;
        let mut leaving = fixed_rings;
        leaving.push(fixed_note);
        fade_out_all(stage, &leaving, 0.3)?;
        let under = under_tosses(&kit, stage, &proof)?;
        swap(stage, &[top_sum, bottom_sum], &under, 0.4)?;
        let rest: Vec<Mob> = changes
            .iter()
            .enumerate()
            .filter(|&(p, _)| p != walk)
            .map(|(_, &m)| m)
            .collect();
        fade_in_lagged(stage, &rest, 0.6, 0.3)?;
        // the row's one outline takes over from its entries' own (the same
        // red, the same height), so no outlines stack in it
        let the_row = row_box(stage, &d.grid, row, palette::mistake())?;
        let in_row: Vec<Mob> = model::wrong_entries(&proof)
            .iter()
            .zip(&d.boxes)
            .filter(|&(&(i, _), _)| i == row)
            .map(|(_, &m)| m)
            .collect();
        swap(stage, &in_row, &[the_row], 0.3)?;
        line.wait_for(stage, "At most one")?;
        let hidden: Vec<Mob> = all
            .iter()
            .enumerate()
            .filter(|(_, t)| model::hides(&proof, t))
            .map(|(k, _)| {
                let (x, y) = toss_place(k);
                ring(stage, v(x, y + BELOW), HIDDEN_R, palette::mistake())
            })
            .collect();
        let shown_hidden: Vec<Mob> = under
            .iter()
            .zip(&all)
            .filter(|(_, t)| model::hides(&proof, t))
            .map(|(&m, _)| m)
            .collect();
        fade_in_all(stage, &hidden, 0.3)?;
        flash_all(stage, &shown_hidden, 0.5)?;

        // 06_7: over a column where the slip's mistake is all zero, the sum
        // never moves, and pairs miss twice
        let line = say(&kit, stage, "06_7")?;
        let slip = model::mistake(&C, &ab);
        let (slip_row, _) = model::wrong_entries(&slip)
            .first()
            .copied()
            .ok_or_else(|| fail("the slip has no wrong entry".into()))?;
        if slip.iter().any(|r| r[column] != 0) {
            return Err(fail("the slip must be all zero in the paired column".into()));
        }
        let mut leaving = vec![picked, the_row];
        leaving.extend(&changes);
        leaving.extend(&hidden);
        leaving.extend(&under);
        let remaining: Vec<Mob> = d
            .boxes
            .iter()
            .zip(model::wrong_entries(&proof))
            .filter(|&(_, (i, _))| i != row)
            .map(|(&m, _)| m)
            .collect();
        leaving.extend(remaining);
        fade_out_all(stage, &leaving, 0.3)?;
        let slip_d = mistake_grid(&kit, stage, &slip)?;
        swap(stage, &d.grid.mobs(), &slip_d.mobs(), 0.5)?;
        line.wait_for(stage, "a column where its mistake is all zeros")?;
        let zero_box = column_box(stage, &slip_d.grid, column, Palette::dim())?;
        let zeros = kit.text(
            stage,
            ALL_ZEROS,
            20.0,
            Palette::dim(),
            v(slip_d.grid.at(0, column)[0], MAT_Y + 1.5 * super::draw::DY + ZEROS_DY),
        )?;
        fade_in_all(stage, &[zero_box, zeros], 0.4)?;
        line.wait_for(stage, "never moves")?;
        let slip_changes = change_labels(&kit, stage, &slip, slip_row, column)?;
        let slip_under = under_tosses(&kit, stage, &slip)?;
        let mut arriving = slip_changes.clone();
        arriving.extend(&slip_under);
        fade_in_lagged(stage, &arriving, 0.8, 0.15)?;
        line.wait_for(stage, "Two of those pairs")?;
        let per_pair = model::misses_per_pair(&slip, column);
        if !per_pair.iter().any(|&m| m > MISSES_PER_PAIR) {
            return Err(fail("over a zero column some pair must miss twice".into()));
        }
        let mut twice = Vec::new();
        for ((tails, heads), &misses) in pairs.iter().zip(&per_pair) {
            if misses > MISSES_PER_PAIR {
                for t in [tails, heads] {
                    let (x, y) = toss_place(index_of(t));
                    twice.push(ring(stage, v(x, y + BELOW), HIDDEN_R, palette::mistake()));
                }
            }
        }
        fade_in_all(stage, &twice, 0.4)?;

        // 06_8: the pairs cover every toss once, at any size
        let line = say(&kit, stage, "06_8")?;
        // Restore the nonzero-column proof before stating its bound. The
        // preceding zero-column example has two misses in some pairs.
        let mut counterexample = slip_d.mobs();
        counterexample.extend([zero_box, zeros]);
        counterexample.extend(slip_changes);
        counterexample.extend(slip_under);
        counterexample.extend(twice);
        fade_out_all(stage, &counterexample, 0.3)?;
        let restored = mistake_grid(&kit, stage, &proof)?;
        let restored_column = column_box(stage, &restored.grid, column, Palette::hero())?;
        let restored_row = row_box(stage, &restored.grid, row, palette::mistake())?;
        let mut proof_picture = restored.grid.mobs();
        proof_picture.extend([restored_column, restored_row]);
        proof_picture.extend(under_tosses(&kit, stage, &proof)?);
        proof_picture.extend(change_labels(&kit, stage, &proof, row, column)?);
        for (k, toss) in all.iter().enumerate() {
            if model::hides(&proof, toss) {
                let (x, y) = toss_place(k);
                proof_picture.push(ring(stage, v(x, y + BELOW), HIDDEN_R, palette::mistake()));
            }
        }
        fade_in_all(stage, &proof_picture, 0.5)?;
        let claim = pairing_card().map_err(fail)?;
        let exact = precisely(&kit, stage, &claim)?;
        fade_in_all(stage, &[exact], 0.3)?;
        let card_up = now(stage);
        let texts = count_texts(&model::misses_per_pair(&proof, column), all.len()).map_err(fail)?;
        let colors = [Palette::ink(), Palette::ink(), palette::mistake()];
        let mut count = Vec::with_capacity(texts.len());
        for ((text, color), y) in texts.iter().zip(colors).zip(COUNT_YS) {
            count.push(kit.text(stage, text, 22.0, color, v(COUNT_X, y))?);
        }
        fade_out_all(stage, &[pair_count], 0.2)?;
        fade_in_lagged(stage, &count, 0.9, 0.4)?;
        line.wait_for(stage, "for matrices of any size")?;
        let general = kit.tex(
            stage,
            GENERAL,
            GENERAL_SIZE,
            Palette::ink(),
            &[],
            v(COUNT_X, COUNT_YS[3]),
        )?;
        fade_in_all(stage, &[general], 0.4)?;
        let up = now(stage);
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

    /// The tally says what the model says, shape by shape.
    #[test]
    fn the_tally_is_built_from_the_model() {
        let lines: Vec<String> = CHAPTER_02_SHAPES
            .iter()
            .map(|name| tally_text(name, &facts::shape(name).unwrap()).unwrap())
            .collect();
        assert_eq!(
            lines,
            [
                "the slip (1): 4 of 8",
                "1, 1 in one row: 2 of 8",
                "1, \u{2212}1 in one row: 4 of 8",
                "1, 1, \u{2212}1 in one row: 3 of 8",
                "1 per row and column: 1 of 8",
            ]
        );
        assert_eq!(signed(1), "+1");
        assert_eq!(signed(-1), "\u{2212}1");
        assert_eq!(signed(0), "0");
        assert_eq!(change_text(0), "+0");
        assert_eq!(change_text(1), "+1");
        assert_eq!(checked_count(), "19,682 checked: most misses 4 of 8");
    }

    /// Which row decides each shape, and the shape that has none.
    #[test]
    fn the_row_that_decides() {
        let rows: Vec<Option<usize>> = CHAPTER_02_SHAPES
            .iter()
            .map(|name| single_wrong_row(&facts::shape(name).unwrap()))
            .collect();
        assert_eq!(rows, [Some(1), Some(1), Some(1), Some(1), None]);
        assert_eq!(per_row_and_column(&facts::shape("every_column").unwrap()), Some(1));
        assert_eq!(per_row_and_column(&facts::shape(PROOF_SHAPE).unwrap()), None);
    }

    /// The cards keep the finite check and the proof apart; the proof's
    /// step picks a nonzero entry of the mistake; both are two lines.
    #[test]
    fn the_cards_say_check_and_proof_apart() {
        let checked = checked_card().unwrap();
        assert!(
            checked.starts_with("all 19,682 nonzero three by three mistakes"),
            "{checked}"
        );
        assert!(checked.contains("entries \u{2212}1, 0 or 1"), "{checked}");
        assert!(
            checked.contains("finite check; the pairing proves every size"),
            "{checked}"
        );
        let pairing = pairing_card().unwrap();
        assert!(pairing.starts_with("pick an entry where the mistake"), "{pairing}");
        assert!(pairing.contains("is not 0"), "{pairing}");
        assert!(!pairing.contains("wrong entry"), "{pairing}");
        for text in [checked.as_str(), pairing.as_str()] {
            for width in [80, 76] {
                let lines = explainer_kit::animatic::wrap(text, width, 3);
                assert!(lines.len() <= 2, "{} lines at {width}: {text}", lines.len());
            }
            assert!(read_s(text) < 12.0, "{text}");
        }
    }

    /// The count's bound per pair is the proof's constant: the worked case
    /// stays within it, and pairs over it (the slip, paired by the coin of
    /// a column it is right in) are refused, not shown.
    #[test]
    fn the_count_uses_the_proofs_bound() {
        let tosses = model::tosses::<SIDE>().len();
        let proof = facts::shape(PROOF_SHAPE).unwrap();
        let (_, column) = facts::proof_entry().unwrap();
        assert_eq!(
            count_texts(&model::misses_per_pair(&proof, column), tosses).unwrap(),
            ["8 tosses, 4 pairs", "at most 1 miss in each pair", "at most 4 of 8"]
        );
        let slip = facts::shape("one_entry").unwrap();
        assert_eq!(model::misses_per_pair(&slip, 0), [2, 2, 0, 0]);
        assert!(count_texts(&model::misses_per_pair(&slip, 0), tosses).is_err());
    }

    /// The pairs by coin 1 stand one above the other, the walked pair is
    /// the second, and the proof's mistake hides from at most one of each.
    #[test]
    fn the_pairs_stand_one_above_the_other() {
        let (row, column) = facts::proof_entry().unwrap();
        assert_eq!((row, column), (1, 0));
        for (t, u) in model::coin_pairs::<SIDE>(column) {
            let (k, l) = (index_of(&t), index_of(&u));
            assert_eq!(l, k + TOSS_XS.len());
        }
        let (t, u) = facts::walk_pair().unwrap();
        assert_eq!((index_of(&t), index_of(&u)), (1, 5));
        let proof = facts::shape(PROOF_SHAPE).unwrap();
        assert!(model::misses_per_pair(&proof, column).iter().all(|&m| m <= 1));
        assert_eq!(index_of(&ALL_HEADS), 7);
    }

    /// Every phrase the scenes wait for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("05_1", "Try mistakes"),
            ("05_2", "Two of eight"),
            ("05_3", "Four of eight"),
            ("05_4", "which is just the toss"),
            ("05_5", "How many tosses"),
            ("05_7", "only from all tails"),
            ("05_8", "What matters"),
            ("05_9", "All nineteen thousand"),
            ("05_10", "Mistakes can hold"),
            ("06_1", "take row two, column one"),
            ("06_2", "that makes four pairs"),
            ("06_3", "With the first coin tails"),
            ("06_4", "How much"),
            ("06_5", "Minus one becomes zero"),
            ("06_5", "Flipping that coin"),
            ("06_6", "At most one"),
            ("06_7", "a column where its mistake is all zeros"),
            ("06_7", "never moves"),
            ("06_7", "Two of those pairs"),
            ("06_8", "for matrices of any size"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The layout keeps its pieces apart and inside the title-safe area.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        use super::super::draw::DY;
        use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};
        const { assert!(TALLY_YS[0] + 0.2 <= FRAME_Y_RADIUS - 0.25, "tally inside the top edge") };
        const { assert!(TALLY_YS[4] - 0.2 > TOSS_YS[0] + COIN_R + 0.1, "tally above the tosses") };
        const {
            assert!(
                TALLY_YS[4] - 0.75 - 0.25 > TOSS_YS[0] + COIN_R,
                "open question above the tosses"
            )
        };
        const { assert!(LINK_TOP > LINK_BOTTOM + 0.2, "a link to draw") };
        const {
            assert!(
                COUNT_YS[3] - 0.3 > -FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36 * 2.0,
                "count over a card"
            )
        };
        const { assert!(QUESTION_Y - 0.2 > -FRAME_Y_RADIUS + 0.15, "question inside the frame") };
        // a hidden sum's ring stays off its pair's link
        const { assert!(TOSS_YS[0] + BELOW - HIDDEN_R >= LINK_TOP, "ring above the link") };
        // the pairs' count (chapter 06 has no tally) clears the think ring
        const { assert!(PAIRS_AT[1] - 0.2 > THINK_AT[1] + 0.62, "pairs' count over the ring") };
        const { assert!(PAIRS_AT[0] + 0.6 < FRAME_X_RADIUS - 0.15, "pairs' count inside") };
        // the fixed coins' note over the walked toss, under the mistake's name
        const { assert!(TOSS_YS[0] + FIXED_DY + 0.15 < NAME_Y - 0.25, "note under the name") };
        const {
            assert!(
                TOSS_YS[0] + FIXED_DY - 0.15 > TOSS_YS[0] + COIN_R,
                "note over the coins"
            )
        };
        // "all zeros" over the mistake, under the top edge
        const {
            assert!(
                MAT_Y + 1.5 * DY + ZEROS_DY + 0.15 < FRAME_Y_RADIUS - 0.15,
                "note inside"
            )
        };
        // the last toss, its link label and the count stay apart
        let last = toss_place(7).0 + COIN_PITCH + COIN_R;
        assert!(last < COUNT_X - 1.6, "tosses left of the count");
        assert!(
            toss_place(3).0 + CHANGE_DX + 0.3 < COUNT_X - 1.6,
            "change label left of the count"
        );
        const { assert!(TOSS_XS[0] - COIN_PITCH - COIN_R > -FRAME_X_RADIUS + 0.15) };
    }
}
