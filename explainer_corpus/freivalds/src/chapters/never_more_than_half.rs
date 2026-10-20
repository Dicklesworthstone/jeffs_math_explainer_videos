//! Chapter 02, never more than half. The mistake on its own, C − A·B: the
//! check misses exactly when its heads columns add up to zero in every row
//! (02_1). Two in one row, both one too big, hide only from tails on both
//! their coins (02_2); one too big and one too small hide whenever their
//! coins agree, half again (02_3). One, one and minus one in a row: how many
//! tosses does it hide from? (02_4, a prediction.) Three, not a power of two
//! (02_5). One wrong entry in each row and column hides only from all tails
//! (02_6). Why never more than half: pair the tosses that differ only in
//! the coin of a column where C is wrong (02_7); across a pair that row's
//! sum moves by the wrong entry, so at most one of the pair hides it (02_8);
//! the pairs cover every toss once, at any size (02_9).

use explainer_kit::props::{lower_third, precisely};
use explainer_kit::{Kit, Palette, fade_all, thousands, v};
use fmn::prelude::*;

use super::draw::{
    COIN_R, Coin, Grid, cell_box, coin_mobs, column_box, column_grid, fade_in_all, fade_in_lagged, fade_out_all, fail,
    flash_all, line_style, matrix_grid, num, ring, row_box, say, stand, swap, think, toss_coins, wait_until,
};
use crate::facts::{self, CHAPTER_02_SHAPES, MISSES_PER_PAIR, PROOF_SHAPE, SWEEP_3};
use crate::model::{self, A, B, FIRST_TOSS, Matrix, SIDE, Toss};
use crate::palette;

pub struct NeverMoreThanHalf {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model and facts)
const MISTAKE_NAME: &str = r"C - AB";
const IDENTITY: &str = r"C\,r - A(B\,r) = (C - AB)\,r";
const ARROW: &str = r"\to";
const CAUGHT_NOTE: &str = "not all zero: caught";
const SLIP: &str = "the slip";
const IN_ONE_ROW: &str = "in one row";
const PER_ROW_AND_COLUMN: &str = "per row and column";
const MISS: &str = "miss";
const CAUGHT: &str = "caught";
const CHECKED: &str = "checked:";
const GENERAL: &str = r"2^n \text{ tosses},\; 2^{n-1} \text{ pairs}";
const EXACT_CARD: &str = "exact arithmetic: flips move row sums by the wrong entry. With fair coins, all independent \
                          and tossed after C is fixed, a toss misses with chance at most half.";

// ------------------------------------------------------------------ layout

/// The mistake's grid, its name under it, and the coins over its columns.
const D_X: f64 = -4.9;
const MAT_Y: f64 = 2.2;
const NAME_Y: f64 = 0.95;
const COIN_Y: f64 = 3.5;
/// 02_1: the first toss's heads columns of the mistake, added, beside it.
const SUM_X: f64 = -2.4;
const ARROW_X: f64 = -3.3;
const CAUGHT_X: f64 = -1.75;
const IDENTITY_AT: Vec3 = [-0.3, 3.45, 0.0];
/// The tally of shapes, top right: one line a shape.
const TALLY_X: f64 = 2.5;
const TALLY_YS: [f64; 5] = [3.3, 2.75, 2.2, 1.65, 1.1];
const TALLY_SIZE: f64 = 20.0;
/// The eight tosses, as in chapter 01: two rows of four, three coins each,
/// and under each toss its row sum or its verdict.
const TOSS_XS: [f64; 4] = [-5.1, -2.5, 0.1, 2.7];
const TOSS_YS: [f64; 2] = [-0.25, -1.55];
const COIN_PITCH: f64 = 0.5;
const BELOW: f64 = -0.45;
const SUM_SIZE: f64 = 24.0;
const QUESTION_X: f64 = -1.2;
const QUESTION_Y: f64 = -2.75;
const THINK_AT: Vec3 = [5.4, -0.9, 0.0];
/// The pairing (02_7, 02_8): a link from each top toss's sum down to its
/// partner, and the change across it beside the link.
const LINK_TOP: f64 = TOSS_YS[0] + BELOW - 0.22;
const LINK_BOTTOM: f64 = TOSS_YS[1] + COIN_R + 0.08;
const LINK_MID: f64 = LINK_BOTTOM + (LINK_TOP - LINK_BOTTOM) / 2.0;
const CHANGE_DX: f64 = 0.32;
/// A ring around a sum the mistake hides behind.
const HIDDEN_R: f64 = 0.22;
/// The count (02_9), right of the tosses.
const COUNT_X: f64 = 5.1;
const GENERAL_SIZE: f64 = 26.0;
const COUNT_YS: [f64; 4] = [-0.35, -0.85, -1.35, -1.95];

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

/// Seconds a lower-third needs on screen to be read: fifteen characters a
/// second, a quick reader (the video can be paused).
fn read_s(text: &str) -> f64 {
    text.chars().count() as f64 / 15.0
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

/// The "checked" lower-third: the finite sweep, labelled as a check of the
/// bound, not its proof. Refused unless the theorem's bound stands.
fn checked_card() -> Result<String, String> {
    facts::theorem_bound()?;
    let wrong = model::sweep(&A, &B, &SWEEP_3).wrong;
    let values: Vec<String> = SWEEP_3.iter().map(|&x| num(x)).collect();
    let (last, rest) = values.split_last().ok_or("no sweep values")?;
    Ok(format!(
        "all {} three by three mistakes with entries {} or {last} hide from at most half the tosses. That is a \
         finite check; the pairing proves every size.",
        thousands(wrong as u64),
        rest.join(", ")
    ))
}

/// The exact version of 02_9, refused unless the theorem's bound is one half.
fn exact_card() -> Result<&'static str, String> {
    match facts::theorem_bound()? {
        (1, 2) => Ok(EXACT_CARD),
        (num, den) => Err(format!("the per-toss bound is {num}/{den}, not half")),
    }
}

/// The count at 02_9 for pairs holding `misses_per_pair` misses among
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
struct Mistake {
    grid: Grid,
    boxes: Vec<Mob>,
}

impl Mistake {
    fn mobs(&self) -> Vec<Mob> {
        let mut all = self.grid.mobs();
        all.extend(&self.boxes);
        all
    }
}

fn mistake_grid(kit: &Kit, stage: &mut Stage<'_>, d: &Matrix<SIDE>) -> fmn::Result<Mistake> {
    let grid = matrix_grid(kit, stage, d, v(D_X, MAT_Y), Palette::ink())?;
    let mut boxes = Vec::new();
    for (i, j) in model::wrong_entries(d) {
        boxes.push(cell_box(stage, &grid, i, j, palette::mistake())?);
    }
    Ok(Mistake { grid, boxes })
}

/// Under each toss, what decides it for the mistake `d`: the sum of its one
/// wrong row (red where it is zero, so the mistake hides), or with more than
/// one wrong row the verdict itself. Detached, in toss order.
fn under_tosses(kit: &Kit, stage: &mut Stage<'_>, d: &Matrix<SIDE>) -> fmn::Result<Vec<Mob>> {
    let row = single_wrong_row(d);
    model::tosses::<SIDE>()
        .iter()
        .enumerate()
        .map(|(k, t)| {
            let (x, y) = toss_place(k);
            let hides = model::hides(d, t);
            let (words, color) = match (row, hides) {
                (Some(i), true) => (num(model::kept_sum(d, t)[i]), palette::mistake()),
                (Some(i), false) => (num(model::kept_sum(d, t)[i]), Palette::dim()),
                (None, true) => (MISS.to_owned(), palette::mistake()),
                (None, false) => (CAUGHT.to_owned(), palette::caught()),
            };
            kit.text(stage, &words, SUM_SIZE, color, v(x, y + BELOW))
        })
        .collect()
}

impl SceneConstruct for NeverMoreThanHalf {
    fn name(&self) -> &str {
        "02_never_more_than_half"
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
        let proof = facts::shape(PROOF_SHAPE).map_err(fail)?;
        let (row, column) = facts::proof_entry().map_err(fail)?;

        // 02_1: the slip's mistake, C − A·B; what the check sees of it
        let line = say(&kit, stage, "02_1")?;
        let slip = model::mistake(&model::C, &ab);
        if slip != shapes[0] {
            return Err(fail("chapter 02 must open on chapter 01's slip".into()));
        }
        let mut d_now = mistake_grid(&kit, stage, &slip)?;
        let name = kit.tex(stage, MISTAKE_NAME, 34.0, palette::mistake(), &[], v(D_X, NAME_Y))?;
        let mut arriving = d_now.mobs();
        arriving.push(name);
        fade_in_all(stage, &arriving, 0.6)?;
        line.wait_for(stage, "The check misses")?;
        let xs: Vec<f64> = (0..SIDE).map(|j| d_now.grid.at(0, j)[0]).collect();
        let coins = toss_coins(&kit, stage, &FIRST_TOSS, &xs, COIN_Y)?;
        fade_in_lagged(stage, &coin_mobs(&coins), 0.6, 0.3)?;
        let mut kept = Vec::new();
        for (j, &heads) in FIRST_TOSS.iter().enumerate() {
            if heads {
                kept.push(column_box(stage, &d_now.grid, j, palette::heads())?);
            }
        }
        fade_in_all(stage, &kept, 0.3)?;
        let sum = column_grid(
            &kit,
            stage,
            &model::kept_sum(&slip, &FIRST_TOSS),
            v(SUM_X, MAT_Y),
            Palette::ink(),
        )?;
        let arrow = kit.tex(stage, ARROW, 34.0, Palette::dim(), &[], v(ARROW_X, MAT_Y))?;
        let mut arriving = sum.mobs();
        arriving.push(arrow);
        fade_in_all(stage, &arriving, 0.4)?;
        if model::hides(&slip, &FIRST_TOSS) {
            return Err(fail("chapter 01's first toss must catch the slip".into()));
        }
        let identity = kit.tex(stage, IDENTITY, 30.0, Palette::ink(), &[], IDENTITY_AT)?;
        let caught = kit.text_left(stage, CAUGHT_NOTE, 22.0, palette::caught(), v(CAUGHT_X, MAT_Y))?;
        fade_in_all(stage, &[identity, caught], 0.4)?;
        let mut tally_mobs = vec![tally_line(&kit, stage, &tallies, 0)?];
        fade_in_all(stage, &tally_mobs, 0.3)?;

        // 02_2: both one too big, in one row; every toss and its row sum
        let line = say(&kit, stage, "02_2")?;
        let mut leaving = coin_mobs(&coins);
        leaving.extend(kept);
        leaving.extend(sum.mobs());
        leaving.extend([arrow, identity, caught]);
        fade_out_all(stage, &leaving, 0.4)?;
        let next = mistake_grid(&kit, stage, &shapes[1])?;
        swap(stage, &d_now.mobs(), &next.mobs(), 0.4)?;
        d_now = next;
        let checks = facts::all_checks();
        let mut tosses: Vec<Vec<Coin>> = Vec::with_capacity(checks.len());
        for (k, (toss, _)) in checks.iter().enumerate() {
            let (x, y) = toss_place(k);
            tosses.push(toss_coins(&kit, stage, toss, &coin_xs(x), y)?);
        }
        let all_coins: Vec<Mob> = tosses.iter().flat_map(|t| coin_mobs(t.as_slice())).collect();
        fade_in_lagged(stage, &all_coins, 1.2, 0.15)?;
        let mut under = under_tosses(&kit, stage, &shapes[1])?;
        fade_in_lagged(stage, &under, 0.6, 0.3)?;
        line.wait_for(stage, "Two of eight")?;
        tally_mobs.push(tally_line(&kit, stage, &tallies, 1)?);
        fade_in_all(stage, &tally_mobs[1..], 0.3)?;

        // 02_3: one too big and one too small cancel when their coins agree
        let line = say(&kit, stage, "02_3")?;
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

        // 02_4: one, one and minus one in a row; the question, and the
        // think time with no sum on screen
        let line = say(&kit, stage, "02_4")?;
        let next = mistake_grid(&kit, stage, &shapes[3])?;
        let mut old = d_now.mobs();
        old.extend(&under);
        swap(stage, &old, &next.mobs(), 0.5)?;
        d_now = next;
        line.wait_for(stage, "How many tosses")?;
        let question = kit.text(
            stage,
            &format!("How many of these {} tosses does it hide from?", checks.len()),
            28.0,
            Palette::ink(),
            v(QUESTION_X, QUESTION_Y),
        )?;
        fade_in_all(stage, &[question], 0.4)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 02_5: three, not half and not a power of two; the sums come only
        // now, after the think time
        say(&kit, stage, "02_5")?;
        fade_out_all(stage, &[q, question], 0.2)?;
        under = under_tosses(&kit, stage, &shapes[3])?;
        fade_in_lagged(stage, &under, 0.6, 0.3)?;
        tally_mobs.push(tally_line(&kit, stage, &tallies, 3)?);
        fade_in_all(stage, &tally_mobs[3..], 0.3)?;

        // 02_6: one wrong entry per row and column hides only from all tails
        let line = say(&kit, stage, "02_6")?;
        let next = mistake_grid(&kit, stage, &shapes[4])?;
        let next_under = under_tosses(&kit, stage, &shapes[4])?;
        let mut old = d_now.mobs();
        old.extend(&under);
        let mut new = next.mobs();
        new.extend(&next_under);
        swap(stage, &old, &new, 0.5)?;
        (d_now, under) = (next, next_under);
        line.wait_for(stage, "only from all tails")?;
        tally_mobs.push(tally_line(&kit, stage, &tallies, 4)?);
        fade_in_all(stage, &tally_mobs[4..], 0.3)?;

        // 02_7: why never more than half: the finite check, then the pairs
        let line = say(&kit, stage, "02_7")?;
        let next = mistake_grid(&kit, stage, &proof)?;
        let next_under = under_tosses(&kit, stage, &proof)?;
        let mut old = d_now.mobs();
        old.extend(&under);
        let mut new = next.mobs();
        new.extend(&next_under);
        swap(stage, &old, &new, 0.5)?;
        (d_now, under) = (next, next_under);
        let checked = lower_third(&kit, stage, CHECKED, &checked_card().map_err(fail)?)?;
        fade_in_all(stage, &[checked], 0.4)?;
        line.wait_for(stage, "Pick a column")?;
        // the column in the gold of its coins' rings, apart from the red of
        // the wrong row it meets at 02_8
        let picked = column_box(stage, &d_now.grid, column, Palette::hero())?;
        let mut rings: Vec<Mob> = tosses
            .iter()
            .map(|coins| ring(stage, coins[column].at, COIN_R + 0.07, Palette::hero()))
            .collect();
        rings.push(picked);
        fade_in_all(stage, &rings, 0.4)?;
        line.wait_for(stage, "Pair the tosses")?;
        let pairs = model::coin_pairs::<SIDE>(column);
        let all = model::tosses::<SIDE>();
        let index = |t: &Toss<SIDE>| all.iter().position(|u| u == t).unwrap_or(0);
        let mut links = Vec::with_capacity(pairs.len());
        for (tails, heads) in &pairs {
            if index(heads) != index(tails) + TOSS_XS.len() {
                return Err(fail("a pair by the first coin must stand one above the other".into()));
            }
            let (x, _) = toss_place(index(tails));
            let link = polyline(vec![v(x, LINK_TOP), v(x, LINK_BOTTOM)])
                .style(line_style(Palette::hero(), 2.5))
                .build();
            links.push(stage.arena_mut().add(link));
        }
        fade_in_lagged(stage, &links, 0.8, 0.3)?;

        // 02_8: across a pair the row's sum moves by the wrong entry, so at
        // most one of the pair hides it
        let line = say(&kit, stage, "02_8")?;
        // the row's one outline takes over from its entries' own (the same
        // red, the same height), so no outlines stack in it
        let the_row = row_box(stage, &d_now.grid, row, palette::mistake())?;
        let in_row: Vec<Mob> = model::wrong_entries(&proof)
            .iter()
            .zip(&d_now.boxes)
            .filter(|&(&(i, _), _)| i == row)
            .map(|(_, &m)| m)
            .collect();
        swap(stage, &in_row, &[the_row], 0.3)?;
        let mut changes = Vec::with_capacity(pairs.len());
        for ((tails, _), change) in pairs.iter().zip(model::pair_changes(&proof, row, column)) {
            let (x, _) = toss_place(index(tails));
            changes.push(kit.text_left(
                stage,
                &signed(change),
                20.0,
                Palette::hero(),
                v(x + CHANGE_DX, LINK_MID),
            )?);
        }
        fade_in_lagged(stage, &changes, 0.8, 0.3)?;
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

        // 02_9: the pairs cover every toss once, at any size
        let line = say(&kit, stage, "02_9")?;
        let claim = exact_card().map_err(fail)?;
        let exact = precisely(&kit, stage, claim)?;
        fade_out_all(stage, &[checked], 0.2)?;
        fade_in_all(stage, &[exact], 0.2)?;
        let card_up = stage.scene().time().to_f64();
        let texts = count_texts(&model::misses_per_pair(&proof, column), all.len()).map_err(fail)?;
        let colors = [Palette::ink(), Palette::ink(), palette::mistake()];
        let mut count = Vec::with_capacity(texts.len());
        for ((text, color), y) in texts.iter().zip(colors).zip(COUNT_YS) {
            count.push(kit.text(stage, text, 22.0, color, v(COUNT_X, y))?);
        }
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
        let up = stage.scene().time().to_f64();
        stand(&kit, stage, up)?;
        wait_until(stage, card_up + read_s(claim))?;

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

    /// The cards keep the finite check and the proof apart, and the
    /// conditions the chance needs; both are two lines and readable.
    #[test]
    fn the_cards_say_check_and_proof_apart() {
        let checked = checked_card().unwrap();
        assert!(checked.starts_with("all 19,682 three by three mistakes"), "{checked}");
        assert!(checked.contains("entries \u{2212}1, 0 or 1"), "{checked}");
        assert!(
            checked.contains("finite check; the pairing proves every size"),
            "{checked}"
        );
        let exact = exact_card().unwrap();
        assert!(exact.starts_with("exact arithmetic"), "{exact}");
        // the chance and every condition it needs, in the card's own words
        assert!(
            exact.contains(
                "With fair coins, all independent and tossed after C is fixed, a toss misses with chance at most half."
            ),
            "{exact}"
        );
        for text in [checked.as_str(), exact] {
            let lines = explainer_kit::animatic::wrap(text, 80, 3);
            assert!(lines.len() <= 2, "{} lines: {text}", lines.len());
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

    /// The pairs by coin 1 stand one above the other, and the proof's
    /// mistake hides from at most one of each.
    #[test]
    fn the_pairs_stand_one_above_the_other() {
        let all = model::tosses::<SIDE>();
        let (row, column) = facts::proof_entry().unwrap();
        assert_eq!((row, column), (1, 0));
        for (t, u) in model::coin_pairs::<SIDE>(column) {
            let (k, l) = (
                all.iter().position(|x| *x == t).unwrap(),
                all.iter().position(|x| *x == u).unwrap(),
            );
            // the same column of the grid, one row down
            assert_eq!(l, k + TOSS_XS.len());
            assert_eq!(k % TOSS_XS.len(), l % TOSS_XS.len());
        }
        let proof = facts::shape(PROOF_SHAPE).unwrap();
        assert!(model::misses_per_pair(&proof, column).iter().all(|&m| m <= 1));
    }

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("02_1", "The check misses"),
            ("02_2", "Two of eight"),
            ("02_3", "Four of eight"),
            ("02_4", "How many tosses"),
            ("02_6", "only from all tails"),
            ("02_7", "Pick a column"),
            ("02_7", "Pair the tosses"),
            ("02_8", "At most one"),
            ("02_9", "for matrices of any size"),
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
        const { assert!(COIN_Y + COIN_R <= FRAME_Y_RADIUS - 0.15, "coins inside the safe area") };
        const { assert!(MAT_Y + 1.5 * DY + 0.2 < COIN_Y - COIN_R, "coins clear of the top row") };
        const { assert!(TALLY_YS[0] + 0.2 <= FRAME_Y_RADIUS - 0.25, "tally inside the top edge") };
        const { assert!(TALLY_YS[4] - 0.2 > TOSS_YS[0] + COIN_R + 0.1, "tally above the tosses") };
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
        // the identity clears the coins, and the tally starts right of it
        const { assert!(IDENTITY_AT[0] - 2.2 > D_X + 1.0, "identity right of the coins") };
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
