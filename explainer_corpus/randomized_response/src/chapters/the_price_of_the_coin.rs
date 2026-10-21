//! Chapters 06 to 08, on one axis of estimates.
//!
//! Chapter 06, real coins scatter: the same class asked eight times with
//! fresh coins, real seeded draws, estimates from minus four to fourteen
//! (06_1, 06_2); every one of the 4²⁰ equally likely ways the coins can
//! land, counted into a distribution of yes counts and estimates (06_3,
//! 06_4); the prediction, how often exactly four (06_5), and its answer,
//! about one class in five, with one in eleven below zero (06_6); what below
//! zero means, and why rounding it up would be wrong (06_7).
//!
//! Chapter 07, how far one survey misses: why every answer wobbles the same
//! (07_1, 07_2); the fixed class's typical miss, √15 (07_3), defined as the
//! root of the average squared miss (07_4), typical, not a limit (07_5);
//! the prediction, three hundred students (07_6), and its answer: more
//! people, a smaller share (07_7, 07_8).
//!
//! Chapter 08, the price of the coin: a sample from a big school adds who
//! gets picked (08_1); the share's misses in percentage points, nine
//! directly and twenty-one with the coins (08_2); the prediction, how many
//! coin answers match nine points (08_3), and its answer, a hundred and
//! fourteen, almost six times as many people (08_4).
//!
//! Every row is `model::answer` on a real draw (`facts::runs`); every bar is
//! the exact chance of its yes count (`model::yes_count_distribution`), its
//! estimate `model::undo`'s; the numbers drawn are checked against the
//! facts. Chapters 07 and 08 open on the axis the chapter before ended on,
//! rebuilt and faded in.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, signed_thousands, thousands, v};
use fmn::prelude::*;

use super::draw::{
    bar, coin, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, grow_lagged, lane, outline, pause, read_by,
    say, square, swap, tex_q, think, wait_until,
};
use crate::facts::{self, Average, BIG_CLASS, RUNS, SEED};
use crate::model::{self, COIN_PAIRS, Channel, Q, Tally};
use crate::palette;

pub struct RealCoinsScatter {
    pub kit: Kit,
}

pub struct HowFarOneSurveyMisses {
    pub kit: Kit,
}

pub struct ThePriceOfTheCoin {
    pub kit: Kit,
}

// the chapters' fixed words (numbers are built from facts)
const YES_HEAD: &str = "yes";
const ESTIMATE_HEAD: &str = "estimate";
const PICKED: &str = "who gets picked, plus the coins";
/// 08_1's formula is the share's variance, not the count's (07_2): its label.
const SHARE: &str = "variance of the estimated share";
const PEOPLE: &str = "times as many people, for the same precision";
const COPIED: &str = "copied";
const DIDNT: &str = "didn't";
const TYPICAL: &str = "typical miss: the root of the average squared miss";
const BIG_TITLE: &str = "300 students, same coins";
const SLIDER: &str = "coin answers";

// ------------------------------------------------------------------ layout

/// The eight classes, one row each: small discs for the students, their
/// yes count and estimate in two columns to the right.
const RUN_X0: f64 = -4.8;
const RUN_DX: f64 = 0.22;
const RUN_Y0: f64 = 3.15;
const RUN_DY: f64 = 0.4;
const DOT_R: f64 = 0.075;
const DOT_RING: f64 = 0.11;
const YES_COL_X: f64 = 0.3;
const EST_COL_X: f64 = 1.9;
const HEAD_Y: f64 = 3.6;
/// The distribution: an axis of yes counts with their estimates under them,
/// one bar per yes count, `BAR_SCALE` units tall per unit of chance.
const AXIS_X0: f64 = -5.6;
const UNIT: f64 = 0.28;
pub(super) const AXIS_Y: f64 = -1.2;
const BAR_W: f64 = 0.44;
pub(super) const BAR_SCALE: f64 = 12.75;
/// Bars shorter than this are not drawn (their chances are under 0.1%).
const MIN_BAR: f64 = 0.01;
const YES_TICK_DY: f64 = 0.3;
const EST_TICK_DY: f64 = 0.65;
/// The axis rows' names start here: 0.31 inside the frame's edge, over the
/// title-safe 0.25 (draft05 had them at −6.95, 0.16 in; Mail 3916).
const ROW_NAME_X: f64 = -6.8;
const TICK_EVERY: usize = 5;
/// The true count's line and words; the scatter's bracket above them.
const TRUTH_H: f64 = 2.75;
const TRUTH_LABEL_Y: f64 = 1.75;
const SD_Y: f64 = 2.15;
const SD_LABEL_Y: f64 = 2.5;
/// Words at the upper left, the derivations and formulas at the right, the
/// total over the right of the axis, the think time at the right.
pub(super) const NOTE_X: f64 = -6.4;
pub(super) const NOTE_Y: [f64; 3] = [2.95, 2.6, 2.25];
const RIGHT_X: f64 = 4.4;
const DERIVE_Y: [f64; 4] = [3.0, 2.45, 1.9, 1.35];
const FORMULA_X: f64 = 3.6;
const FORMULA_Y: [f64; 4] = [2.8, 2.1, 1.4, 0.85];
/// The share label's height over the formula.
const SHARE_DY: f64 = 0.62;
const TOTAL_X: f64 = 3.0;
const TOTAL_Y: f64 = 3.2;
pub(super) const THINK_X: f64 = 5.4;
pub(super) const THINK_Y: f64 = 1.6;
/// Chapter 07's two rows of one answer's outcomes, at the upper left: their
/// names, their squares, each row's variance beside it.
const ROWS_NAME_X: f64 = -6.45;
const ROWS_X0: f64 = -5.0;
const ROWS_DX: f64 = 0.42;
const ROWS_SIDE: f64 = 0.32;
const ROWS_Y: [f64; 2] = [3.05, 2.55];
const ROWS_VAR_X: f64 = -3.25;
/// Chapter 07's caption for the typical miss, under the derivation (about
/// 5.4 wide at 20 points).
const TYPICAL_X: f64 = 3.4;
/// Chapter 07's three hundred: a line of estimates 30 to 90 at the upper
/// right, its title over it, the mark's words under it, the root's TeX
/// under them; the share bars and captions under those.
const BIG_X0: f64 = 2.4;
const BIG_X1: f64 = 6.2;
const BIG_LOW: f64 = 30.0;
const BIG_HIGH: f64 = 90.0;
const BIG_Y: f64 = 2.45;
const BIG_TITLE_Y: f64 = 3.05;
const BIG_MARK_H: f64 = 0.18;
const BIG_WORDS_DY: f64 = 0.38;
const BIG_ROOT_Y: f64 = 1.55;
const BIG_THINK_X: f64 = 4.3;
const BIG_THINK_Y: f64 = 1.4;
const SHARE_BAR_X0: f64 = 2.4;
const SHARE_BAR_SCALE: f64 = 5.0;
const SHARE_BAR_H: f64 = 0.16;
const SHARE_BAR_Y: [f64; 2] = [0.95, 0.6];
const SHARE_WORDS_DX: f64 = 0.25;
const CAPTION_Y: [f64; 2] = [0.2, -0.15];
/// Chapter 08's share axis at the left: −40% to 80%, a tick every 20
/// points; its two brackets over it, their words over them; the slider
/// under the lower bracket; the think time under that.
const SHARE_X0: f64 = -6.0;
const SHARE_X1: f64 = -0.6;
const SHARE_LOW: f64 = -40.0;
const SHARE_HIGH: f64 = 80.0;
const SHARE_TICKS: [i32; 7] = [-40, -20, 0, 20, 40, 60, 80];
const SHARE_AXIS_Y: f64 = 0.2;
const SHARE_TICK_DY: f64 = 0.3;
const BRACKET_Y: [f64; 2] = [1.6, 1.0];
const BRACKET_WORDS_DY: f64 = 0.3;
const SLIDER_Y: f64 = -0.35;
const MATCH_Y: [f64; 2] = [-1.0, -1.55];
const SHARE_THINK_X: f64 = -3.3;
const SHARE_THINK_Y: f64 = -1.2;

/// Where estimate `e` sits on the axis, the lowest estimate (`low`) at the
/// left end.
pub(super) fn est_x(e: Q, low: Q) -> f64 {
    AXIS_X0 + UNIT * (e - low).to_f64()
}

/// Where estimate `e` of the three hundred sits on chapter 07's short line.
fn big_x(e: f64) -> f64 {
    BIG_X0 + (BIG_X1 - BIG_X0) * (e - BIG_LOW) / (BIG_HIGH - BIG_LOW)
}

/// Where a share of `points` percentage points sits on chapter 08's axis.
fn share_x(points: f64) -> f64 {
    SHARE_X0 + (SHARE_X1 - SHARE_X0) * (points - SHARE_LOW) / (SHARE_HIGH - SHARE_LOW)
}

/// A whole number as the chapter writes it ("−4" with a typographic minus).
fn whole_text(q: Q) -> fmn::Result<String> {
    q.integer()
        .map(signed_thousands)
        .ok_or_else(|| fail(format!("{q} is not a whole number")))
}

/// A large whole number in TeX, thousands separated: 1{,}099{,}511{,}627{,}776.
fn tex_thousands(n: u128) -> String {
    thousands(n).replace(',', "{,}")
}

/// 06_2's exact version: where the eight classes come from. Short, since it
/// is on screen about 5 s (draft06 had 126 characters in 5.13 s; Mail 3941).
fn runs_card() -> String {
    format!("the first {RUNS} draws from PCG64DXSM, seed {SEED}, none thrown away; too few to show the rates")
}

/// 06_6's exact version: the two chances over every way the coins can land.
fn rates_card(outcomes: u128, truth: Q, exactly: Q, negative: Q) -> String {
    format!(
        "over all {} equally likely ways the coins can land: exactly {truth} about {}% of the time, below 0 \
         about {}%",
        thousands(outcomes),
        exactly.percent(),
        negative.percent()
    )
}

/// 06_7's exact version: with its negative values allowed, the estimate
/// averages `mean`; with them replaced by 0, `clamped`. Short, since it is on
/// screen about 5 s (draft06 had 117 characters in 5.27 s; Mail 3941).
fn clamp_card(n: usize, mean: Q, clamped: Q) -> String {
    let factor = Q::ONE / Channel::two_coins().truth();
    format!(
        "allow negatives: {factor} × yes − {} averages {mean}; replace them with 0: the average is {:.2}",
        n / 2,
        clamped.to_f64()
    )
}

/// 07_2's exact version (the old 03_8's): the fixed class's standard
/// deviation, step by step. Doubling the count multiplies its variance by
/// the factor squared (4), not by 2: the same step as the TeX derivation
/// beside it (FuchsiaHawk, Mail 3899, caught "doubled, 15").
fn sd_card(n: usize, answer: Q, count: Q, estimate: Q) -> String {
    let factor = Q::ONE / Channel::two_coins().truth();
    format!(
        "each answer's variance is {answer} for anyone; {n} make {count}; doubling the count multiplies variance by \
         {}: {estimate}. Standard deviation: about {:.2}",
        factor * factor,
        estimate.to_f64().sqrt()
    )
}

/// 07_4's exact version: what "miss by about" means.
fn typical_card(sd: f64) -> String {
    format!(
        "typical miss: the standard deviation, the square root of the average squared miss over every equally \
         likely outcome: about {sd:.2}"
    )
}

/// 07_5's exact version: typical, not a bound, with the class's tails.
fn bound_card(by: Q, at_least: Q, twice: Q) -> String {
    let one_in = (Q::ONE / twice).to_f64().round();
    format!(
        "not a bound: {}% of outcomes miss by {by} or more, about 1 in {one_in} by {} or more",
        at_least.percent(),
        Q::int(2) * by
    )
}

/// A miss as the share bars (and chapter 13's table) write it: whole when it
/// is whole, else to two places.
pub(super) fn miss_text(x: f64) -> String {
    if (x - x.round()).abs() < 1e-9 {
        format!("{x:.0}")
    } else {
        format!("{x:.2}")
    }
}

/// 07_8's exact version: a fixed class's typical miss for any size.
fn sizes_card(n: usize, sd: f64, big: usize, big_miss: i128) -> String {
    format!(
        "fixed class of n, everyone answers: the typical miss is the square root of 3n/4, whatever k; {n}: {sd:.2} \
         ({:.0}% of {n}); {big}: {big_miss} ({}% of {big})",
        100.0 * sd / n as f64,
        Q::new(big_miss, big as i128).percent()
    )
}

/// 08_1's exact version, on screen from "Sample twenty" to the end: what the
/// sampling formula, and 08_4's "almost six times as many people", assume.
/// Placed late in the old 03_10 it had 4.20 s for 143 characters (Mail 3941).
fn sample_card(share: Q, cost: Q) -> String {
    format!(
        "assumes independent draws, with replacement or from a large population, and truthful direct answers. \
         At p = {share} that is {cost}, about {:.2} times",
        cost.to_f64()
    )
}

/// The axis of yes counts and estimates for a class of `n`, its rows'
/// names and ticks. Detached.
pub(super) fn axis_frame(kit: &Kit, stage: &mut Stage<'_>, n: usize) -> fmn::Result<Vec<Mob>> {
    let low = model::undo(0, n)[2];
    let high = model::undo(n, n)[2];
    let mut frame = vec![lane(
        stage,
        vec![v(est_x(low, low) - 0.3, AXIS_Y), v(est_x(high, low) + 0.3, AXIS_Y)],
        Palette::dim(),
    )];
    frame.push(kit.text_left(
        stage,
        YES_HEAD,
        20.0,
        palette::yes(),
        v(ROW_NAME_X, AXIS_Y - YES_TICK_DY),
    )?);
    frame.push(kit.text_left(
        stage,
        ESTIMATE_HEAD,
        20.0,
        palette::student(),
        v(ROW_NAME_X, AXIS_Y - EST_TICK_DY),
    )?);
    for y in (0..=n).step_by(TICK_EVERY) {
        let x = est_x(model::undo(y, n)[2], low);
        frame.push(kit.text(stage, &y.to_string(), 20.0, palette::yes(), v(x, AXIS_Y - YES_TICK_DY))?);
        frame.push(kit.text(
            stage,
            &whole_text(model::undo(y, n)[2])?,
            20.0,
            palette::student(),
            v(x, AXIS_Y - EST_TICK_DY),
        )?);
    }
    Ok(frame)
}

/// One bar per yes count of `dist` (a class of `dist.len() − 1`), as tall
/// as its chance, in `color`; the bars too short to see are left out.
/// Detached.
pub(super) fn bars_of(stage: &mut Stage<'_>, dist: &[Q], color: Srgb) -> fmn::Result<Vec<(usize, Mob)>> {
    let n = dist.len().saturating_sub(1);
    let low = model::undo(0, n)[2];
    let mut bars = Vec::with_capacity(dist.len());
    for (y, &p) in dist.iter().enumerate() {
        let h = BAR_SCALE * p.to_f64();
        if h < MIN_BAR {
            continue;
        }
        let x = est_x(model::undo(y, n)[2], low);
        bars.push((y, bar(stage, color, x, AXIS_Y, BAR_W, h)?));
    }
    Ok(bars)
}

/// The true count's line on the axis and its words, `copiers` copied, for a
/// class whose lowest estimate is `low`. Detached.
pub(super) fn truth_marker(kit: &Kit, stage: &mut Stage<'_>, copiers: usize, low: Q) -> fmn::Result<[Mob; 2]> {
    let x = est_x(Q::int(copiers as i128), low);
    Ok([
        lane(stage, vec![v(x, AXIS_Y), v(x, AXIS_Y + TRUTH_H)], palette::copied()),
        kit.text(
            stage,
            &format!("{copiers} copied"),
            22.0,
            palette::copied(),
            v(x, TRUTH_LABEL_Y),
        )?,
    ])
}

/// The fixed class's bracket, one standard deviation of `estimate_var`
/// either side of the truth at `truth_x`, and its ± label (review PinkDuck
/// F4). Detached.
fn bracket(kit: &Kit, stage: &mut Stage<'_>, truth_x: f64, estimate_var: Q) -> fmn::Result<[Mob; 2]> {
    let spread = estimate_var.to_f64().sqrt();
    let (left, right) = (truth_x - UNIT * spread, truth_x + UNIT * spread);
    Ok([
        lane(
            stage,
            vec![
                v(left, SD_Y - 0.12),
                v(left, SD_Y),
                v(right, SD_Y),
                v(right, SD_Y - 0.12),
            ],
            Palette::ink(),
        ),
        kit.tex(
            stage,
            &format!(r"\pm\sqrt{{{estimate_var}}} \approx \pm{:.2}", spread),
            30.0,
            Palette::ink(),
            &[],
            v(truth_x, SD_LABEL_Y),
        )?,
    ])
}

/// Redraw the bars whose estimate `which` picks in `color` (fresh bars over
/// the old ones, the same heights), returning the old and the new to swap.
pub(super) fn recolor_bars(
    stage: &mut Stage<'_>,
    bars: &mut [(usize, Mob)],
    dist: &[Q],
    which: impl Fn(Q) -> bool,
    color: Srgb,
) -> fmn::Result<(Vec<Mob>, Vec<Mob>)> {
    let n = dist.len().saturating_sub(1);
    let low = model::undo(0, n)[2];
    let (mut old, mut new) = (Vec::new(), Vec::new());
    for entry in bars.iter_mut() {
        let (y, m) = *entry;
        let e = model::undo(y, n)[2];
        if !which(e) {
            continue;
        }
        let fresh = bar(stage, color, est_x(e, low), AXIS_Y, BAR_W, BAR_SCALE * dist[y].to_f64())?;
        old.push(m);
        new.push(fresh);
        entry.1 = fresh;
    }
    Ok((old, new))
}

/// A horizontal bracket from `from` to `to` at height `y`, its ends turned
/// down. Detached.
fn span(stage: &mut Stage<'_>, from: f64, to: f64, y: f64, color: Srgb) -> Mob {
    lane(
        stage,
        vec![v(from, y - 0.1), v(from, y), v(to, y), v(to, y - 0.1)],
        color,
    )
}

/// The class of twenty's scene state chapters 07 and 08 open on: the axis,
/// its dim bars and the truth's line and words. Detached.
struct Opening {
    frame: Vec<Mob>,
    bars: Vec<(usize, Mob)>,
    truth: [Mob; 2],
}

impl Opening {
    fn build(kit: &Kit, stage: &mut Stage<'_>, n: usize, copiers: usize, dist: &[Q]) -> fmn::Result<Opening> {
        let low = model::undo(0, n)[2];
        Ok(Opening {
            frame: axis_frame(kit, stage, n)?,
            bars: bars_of(stage, dist, Palette::dim())?,
            truth: truth_marker(kit, stage, copiers, low)?,
        })
    }

    /// Every part as it stands now.
    fn all(&self) -> Vec<Mob> {
        let mut out = self.frame.clone();
        out.extend(self.bars.iter().map(|&(_, m)| m));
        out.extend(self.truth);
        out
    }
}

impl SceneConstruct for RealCoinsScatter {
    fn name(&self) -> &str {
        "06_real_coins_scatter"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let a: Average = facts::average().map_err(fail)?;
        let (n, copiers) = (a.class.size(), a.class.copiers());
        let truth = Q::int(copiers as i128);
        let runs = facts::runs().map_err(fail)?;
        let dist = model::yes_count_distribution(n, copiers).map_err(fail)?;
        let low = model::undo(0, n)[2];

        // 06_1: real coins; the class asked eight times
        let line = say(&kit, stage, "06_1")?;
        let heads = [
            kit.text(stage, YES_HEAD, 22.0, palette::yes(), v(YES_COL_X, HEAD_Y))?,
            kit.text(stage, ESTIMATE_HEAD, 22.0, palette::student(), v(EST_COL_X, HEAD_Y))?,
        ];
        fade_in_all(stage, &heads, 0.3)?;
        line.wait_for(stage, "Ask the class")?;
        let mut panel = heads.to_vec();
        let mut estimates = Vec::with_capacity(RUNS);
        for (r, coins) in runs.iter().enumerate() {
            let y = RUN_Y0 - RUN_DY * r as f64;
            let mut row = Vec::with_capacity(n + copiers + 1);
            for (seat, &(first, second)) in coins.iter().enumerate() {
                let copied = a.class.copied(seat);
                let at = v(RUN_X0 + RUN_DX * seat as f64, y);
                row.push(coin(
                    stage,
                    palette::answer(model::answer(copied, first, second)),
                    at,
                    DOT_R,
                ));
                if copied {
                    row.push(outline(stage, palette::copied(), at, DOT_RING));
                }
            }
            let yes = Tally::of(&a.class, coins).map_err(fail)?.answers(true);
            row.push(kit.text(stage, &yes.to_string(), 22.0, palette::yes(), v(YES_COL_X, y))?);
            fade_in_all(stage, &row, 0.3)?;
            panel.extend(row);
            estimates.push(model::undo(yes, n)[2]);
        }

        // 06_2: their estimates, from the lowest to the highest
        let line = say(&kit, stage, "06_2")?;
        let runs_text = runs_card();
        let card = precisely(&kit, stage, &runs_text)?;
        let mut est_texts = Vec::with_capacity(RUNS);
        for (r, &e) in estimates.iter().enumerate() {
            let y = RUN_Y0 - RUN_DY * r as f64;
            est_texts.push(kit.text(stage, &whole_text(e)?, 22.0, palette::student(), v(EST_COL_X, y))?);
        }
        let mut shown = est_texts.clone();
        shown.push(card);
        fade_in_lagged(stage, &shown, 1.2, 0.2)?;
        let runs_read_until = read_by(stage, &runs_text);
        panel.extend(est_texts.iter().copied());
        let lowest = estimates
            .iter()
            .copied()
            .min()
            .ok_or_else(|| fail("no classes".into()))?;
        let highest = estimates
            .iter()
            .copied()
            .max()
            .ok_or_else(|| fail("no classes".into()))?;
        line.wait_for(stage, "minus four to fourteen")?;
        let extremes: Vec<Mob> = estimates
            .iter()
            .zip(&est_texts)
            .filter(|&(&e, _)| e == lowest || e == highest)
            .map(|(_, &m)| m)
            .collect();
        flash_all(stage, &extremes, 0.6)?;
        line.wait_for(stage, "Only one lands")?;
        let on_truth: Vec<Mob> = estimates
            .iter()
            .zip(&est_texts)
            .filter(|&(&e, _)| e == truth)
            .map(|(_, &m)| m)
            .collect();
        flash_all(stage, &on_truth, 0.6)?;

        // 06_3: every way the coins can land; the distribution rises
        wait_until(stage, runs_read_until)?;
        let line = say(&kit, stage, "06_3")?;
        panel.push(card);
        fade_out_all(stage, &panel, 0.5)?;
        let frame = axis_frame(&kit, stage, n)?;
        fade_in_all(stage, &frame, 0.5)?;
        line.wait_for(stage, "Four to the twentieth")?;
        let configurations = (COIN_PAIRS.len() as u128).pow(n as u32);
        let total = kit.tex(
            stage,
            &format!(r"{}^{{{n}}} = {}", COIN_PAIRS.len(), tex_thousands(configurations)),
            34.0,
            Palette::ink(),
            &[],
            v(TOTAL_X, TOTAL_Y),
        )?;
        fade_in_all(stage, &[total], 0.4)?;
        // one bar per yes count, as tall as its exact chance
        let bars = bars_of(stage, &dist, Palette::dim())?;
        let bar_mobs: Vec<Mob> = bars.iter().map(|&(_, m)| m).collect();
        grow_lagged(stage, &bar_mobs, DOWN, 2.0, 0.15)?;

        // 06_4: piled up around four, the true count
        let line = say(&kit, stage, "06_4")?;
        let mean = model::estimate_mean(&dist);
        if mean != truth {
            return Err(fail("the distribution's estimate does not average the truth".into()));
        }
        let marker = truth_marker(&kit, stage, copiers, low)?;
        fade_in_all(stage, &marker, 0.4)?;
        line.wait_for(stage, "around four")?;
        let near: Vec<Mob> = bars
            .iter()
            .filter(|&&(y, _)| (model::undo(y, n)[2] - truth).to_f64().abs() <= 2.0)
            .map(|&(_, m)| m)
            .collect();
        flash_all(stage, &near, 0.6)?;

        // 06_5: the prediction; no chance is on screen yet
        let _line = say(&kit, stage, "06_5")?;
        fade_out_all(stage, &[total], 0.3)?;
        let q = think(&kit, stage, v(THINK_X, THINK_Y))?;

        // 06_6: about one class in five; about one in eleven below zero
        let line = say(&kit, stage, "06_6")?;
        fade_out_all(stage, &[q], 0.2)?;
        let f = facts::facts().map_err(fail)?;
        let fact = |key: &str| -> fmn::Result<String> {
            f.get(key)
                .map(str::to_owned)
                .ok_or_else(|| fail(format!("no fact {key}")))
        };
        let at_mean = model::mean(&dist)
            .integer()
            .and_then(|m| usize::try_from(m).ok())
            .ok_or_else(|| fail("the mean yes count is not a whole number".into()))?;
        let exactly = dist[at_mean];
        let negative = model::p_negative_estimate(&dist);
        let mut bars_now = bars.clone();
        let (old, mut new) = recolor_bars(stage, &mut bars_now, &dist, |e| e == truth, Palette::ink())?;
        let note_exactly = kit.text_left(
            stage,
            &format!(
                "exactly {truth}: about 1 class in {} ({}%)",
                fact("R-F14.one_in.exactly")?,
                exactly.percent()
            ),
            20.0,
            Palette::ink(),
            v(NOTE_X, NOTE_Y[0]),
        )?;
        new.push(note_exactly);
        swap(stage, &old, &new, 0.4)?;
        line.wait_for(stage, "below zero")?;
        let (old, mut new) = recolor_bars(stage, &mut bars_now, &dist, |e| e < Q::ZERO, Palette::red())?;
        let forced = model::undo(0, n)[0];
        let note_negative = kit.text_left(
            stage,
            &format!(
                "below 0 (under {} yeses): about 1 in {} ({}%)",
                whole_text(forced)?,
                fact("R-F14.one_in.negative")?,
                negative.percent()
            ),
            20.0,
            Palette::red(),
            v(NOTE_X, NOTE_Y[1]),
        )?;
        new.push(note_negative);
        let rates_text = rates_card(configurations, truth, exactly, negative);
        let card = precisely(&kit, stage, &rates_text)?;
        new.push(card);
        swap(stage, &old, &new, 0.4)?;
        let rates_read_until = read_by(stage, &rates_text);

        // 06_7: below zero, honestly; rounding up would push the average up
        wait_until(stage, rates_read_until)?;
        let line = say(&kit, stage, "06_7")?;
        let red: Vec<Mob> = bars_now
            .iter()
            .filter(|&&(y, _)| model::undo(y, n)[2] < Q::ZERO)
            .map(|&(_, m)| m)
            .collect();
        flash_all(stage, &red, 0.6)?;
        line.wait_for(stage, "Rounding those up")?;
        let clamped = model::clamped_mean(&dist);
        if clamped <= truth {
            return Err(fail(format!("rounding up leaves the average at {clamped}")));
        }
        let note_clamped = kit.text_left(
            stage,
            &format!("rounded up to 0, the average would be {:.2}", clamped.to_f64()),
            20.0,
            Palette::dim(),
            v(NOTE_X, NOTE_Y[2]),
        )?;
        let clamp_text = clamp_card(n, mean, clamped);
        let clamp = precisely(&kit, stage, &clamp_text)?;
        fade_out_all(stage, &[card], 0.2)?;
        fade_in_all(stage, &[note_clamped, clamp], 0.2)?;
        let clamp_read_until = read_by(stage, &clamp_text);
        line.wait_end(stage)?;
        wait_until(stage, clamp_read_until)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

impl SceneConstruct for HowFarOneSurveyMisses {
    fn name(&self) -> &str {
        "07_how_far_one_survey_misses"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let a: Average = facts::average().map_err(fail)?;
        let (n, copiers) = (a.class.size(), a.class.copiers());
        let truth = Q::int(copiers as i128);
        let dist = model::yes_count_distribution(n, copiers).map_err(fail)?;
        let low = model::undo(0, n)[2];
        let truth_x = est_x(truth, low);

        // 07_1: the axis chapter 06 ended on; one answer's outcomes, each truth
        let line = say(&kit, stage, "07_1")?;
        let mut start = Opening::build(&kit, stage, n, copiers, &dist)?;
        fade_in_all(stage, &start.all(), 0.5)?;
        let mut rows = Vec::new();
        let mut row_squares: [Vec<Mob>; 2] = [Vec::new(), Vec::new()];
        for (r, copied) in [(0, true), (1, false)] {
            let (word, color) = if copied {
                (COPIED, palette::copied())
            } else {
                (DIDNT, palette::student())
            };
            rows.push(kit.text_left(stage, word, 20.0, color, v(ROWS_NAME_X, ROWS_Y[r]))?);
            for (k, &(first, second)) in COIN_PAIRS.iter().enumerate() {
                let at = v(ROWS_X0 + ROWS_DX * k as f64, ROWS_Y[r]);
                let s = square(
                    stage,
                    palette::answer(model::answer(copied, first, second)),
                    at,
                    ROWS_SIDE,
                )?;
                rows.push(s);
                row_squares[r].push(s);
            }
        }
        line.wait_for(stage, "A copier")?;
        fade_in_lagged(stage, &rows, 0.8, 0.1)?;

        // 07_2: mirror images: each answer's variance 3/16 for anyone
        let line = say(&kit, stage, "07_2")?;
        let channel = Channel::two_coins();
        let answer = channel.answer_variance(true);
        if channel.answer_variance(false) != answer {
            return Err(fail("the two truths' answers vary differently".into()));
        }
        let count = model::variance(&dist);
        let factor = Q::ONE / channel.truth();
        let estimate = factor * factor * count;
        if answer * Q::int(n as i128) != count {
            return Err(fail(format!("{n} answers of {answer} do not make {count}")));
        }
        let mut variances = Vec::with_capacity(2);
        for &y in &ROWS_Y {
            variances.push(kit.tex(stage, &tex_q(answer, true), 28.0, Palette::ink(), &[], v(ROWS_VAR_X, y))?);
        }
        fade_in_all(stage, &variances, 0.4)?;
        let sd_text = sd_card(n, answer, count, estimate);
        let sd_note = precisely(&kit, stage, &sd_text)?;
        let steps = [
            format!(r"{} \times {n} = {}", tex_q(answer, true), tex_q(count, true)),
            format!(r"{} \times {} = {estimate}", tex_q(count, true), factor * factor),
            format!(r"\sqrt{{{estimate}}} \approx {:.2}", estimate.to_f64().sqrt()),
        ];
        let mut derivation = Vec::with_capacity(steps.len());
        for (src, &y) in steps.iter().zip(&DERIVE_Y) {
            derivation.push(kit.tex(stage, src, 34.0, Palette::ink(), &[], v(RIGHT_X, y))?);
        }
        fade_in_all(stage, &[derivation[0], sd_note], 0.4)?;
        let sd_read = read_by(stage, &sd_text);
        line.wait_for(stage, "Every answer wobbles")?;
        flash_all(stage, &variances, 0.5)?;

        // 07_3: twenty, doubled: a miss of about four, √15
        let line = say(&kit, stage, "07_3")?;
        fade_in_lagged(stage, &derivation[1..], 1.0, 0.6)?;
        line.wait_for(stage, "miss by about four")?;
        let sd_bracket = bracket(&kit, stage, truth_x, estimate)?;
        fade_in_all(stage, &sd_bracket, 0.4)?;

        // 07_4: the typical miss: the root of the average squared miss
        let line = say(&kit, stage, "07_4")?;
        let mut rows_out = rows.clone();
        rows_out.extend(variances.iter().copied());
        fade_out_all(stage, &rows_out, 0.3)?;
        line.wait_for(stage, "Square each")?;
        let sd = estimate.to_f64().sqrt();
        let typical = kit.text(stage, TYPICAL, 20.0, Palette::ink(), v(TYPICAL_X, DERIVE_Y[3]))?;
        let typical_text = typical_card(sd);
        let typical_note = precisely(&kit, stage, &typical_text)?;
        fade_in_all(stage, &[typical], 0.4)?;
        wait_until(stage, sd_read)?;
        swap(stage, &[sd_note], &[typical_note], 0.4)?;
        let typical_read = read_by(stage, &typical_text);

        // 07_5: typical, not a limit: more than two in five miss by four or more
        let line = say(&kit, stage, "07_5")?;
        let miss_by = Q::int(sd.round() as i128);
        let at_least = model::p_miss_at_least(&dist, truth, miss_by);
        let twice = model::p_miss_at_least(&dist, truth, Q::int(2) * miss_by);
        line.wait_for(stage, "More than two")?;
        let (old, mut new) = recolor_bars(
            stage,
            &mut start.bars,
            &dist,
            |e| e - truth >= miss_by || truth - e >= miss_by,
            Palette::ink(),
        )?;
        let note = kit.text_left(
            stage,
            &format!("miss by {miss_by} or more: {}% of outcomes", at_least.percent()),
            20.0,
            Palette::ink(),
            v(NOTE_X, NOTE_Y[0]),
        )?;
        new.push(note);
        swap(stage, &old, &new, 0.4)?;
        let bound_text = bound_card(miss_by, at_least, twice);
        let bound_note = precisely(&kit, stage, &bound_text)?;
        wait_until(stage, typical_read)?;
        swap(stage, &[typical_note], &[bound_note], 0.4)?;
        let bound_read = read_by(stage, &bound_text);

        // 07_6: the prediction: three hundred students, same coins
        let line = say(&kit, stage, "07_6")?;
        let mut leaving = derivation.clone();
        leaving.extend([typical, note]);
        fade_out_all(stage, &leaving, 0.3)?;
        line.wait_for(stage, "Does the estimate")?;
        let panel = [
            kit.text(
                stage,
                BIG_TITLE,
                22.0,
                Palette::ink(),
                v((BIG_X0 + BIG_X1) / 2.0, BIG_TITLE_Y),
            )?,
            lane(stage, vec![v(BIG_X0, BIG_Y), v(BIG_X1, BIG_Y)], Palette::dim()),
        ];
        fade_in_all(stage, &panel, 0.4)?;
        let q = think(&kit, stage, v(BIG_THINK_X, BIG_THINK_Y))?;

        // 07_7: more, about fifteen; but one in twenty of them, against one in five
        let line = say(&kit, stage, "07_7")?;
        fade_out_all(stage, &[q], 0.2)?;
        let big_copied = Q::int(BIG_CLASS as i128) * Q::new(copiers as i128, n as i128);
        let big_var = channel
            .coin_variance()
            .map(|w| Q::int(BIG_CLASS as i128) * w)
            .ok_or_else(|| fail("no variance for three hundred".into()))?;
        let big_miss = big_var.to_f64().sqrt();
        let mid = big_x(big_copied.to_f64());
        let (from, to) = (
            big_x(big_copied.to_f64() - big_miss),
            big_x(big_copied.to_f64() + big_miss),
        );
        let big = [
            lane(
                stage,
                vec![v(mid, BIG_Y - BIG_MARK_H), v(mid, BIG_Y + BIG_MARK_H)],
                palette::copied(),
            ),
            kit.text(
                stage,
                &format!("{big_copied} copied"),
                18.0,
                palette::copied(),
                v(mid, BIG_Y - BIG_WORDS_DY),
            )?,
            span(stage, from, to, BIG_Y + 0.3, Palette::ink()),
            kit.tex(
                stage,
                &format!(
                    r"\sqrt{{{} \times {BIG_CLASS}}} = \sqrt{{{big_var}}} = {big_miss}",
                    tex_q(channel.coin_variance().unwrap_or(Q::ZERO), true)
                ),
                30.0,
                Palette::ink(),
                &[],
                v((BIG_X0 + BIG_X1) / 2.0, BIG_ROOT_Y),
            )?,
        ];
        fade_in_all(stage, &big, 0.5)?;
        line.wait_for(stage, "one in twenty")?;
        let class_share = sd / n as f64;
        let big_share = big_miss / BIG_CLASS as f64;
        let mut shares = Vec::with_capacity(4);
        for (&(count, miss, part), &y) in [(n, sd, class_share), (BIG_CLASS, big_miss, big_share)]
            .iter()
            .zip(&SHARE_BAR_Y)
        {
            let len = SHARE_BAR_SCALE * part;
            shares.push(bar(
                stage,
                Palette::ink(),
                SHARE_BAR_X0 + len / 2.0,
                y - SHARE_BAR_H / 2.0,
                len,
                SHARE_BAR_H,
            )?);
            shares.push(kit.text_left(
                stage,
                &format!("{count} students: {}, {:.0}% of {count}", miss_text(miss), 100.0 * part),
                18.0,
                Palette::ink(),
                v(SHARE_BAR_X0 + len + SHARE_WORDS_DX, y),
            )?);
        }
        fade_in_all(stage, &shares, 0.4)?;
        line.wait_end(stage)?;
        // the two shares stand side by side a moment (a named hold)
        pause(stage, 1.0)?;

        // 07_8: in people the miss grows; as a share it shrinks
        let line = say(&kit, stage, "07_8")?;
        let captions = [
            kit.text(
                stage,
                &format!("in people: from {sd:.2} to {big_miss}"),
                20.0,
                Palette::ink(),
                v((BIG_X0 + BIG_X1) / 2.0, CAPTION_Y[0]),
            )?,
            kit.text(
                stage,
                &format!(
                    "as a share: from {:.0}% to {:.0}%",
                    100.0 * class_share,
                    100.0 * big_share
                ),
                20.0,
                Palette::ink(),
                v((BIG_X0 + BIG_X1) / 2.0, CAPTION_Y[1]),
            )?,
        ];
        fade_in_all(stage, &captions[..1], 0.3)?;
        line.wait_for(stage, "As a share")?;
        fade_in_all(stage, &captions[1..], 0.3)?;
        let big_miss_whole = big_var
            .integer()
            .map(|_| big_miss.round() as i128)
            .ok_or_else(|| fail(format!("{big_var} is not whole")))?;
        let sizes_text = sizes_card(n, sd, BIG_CLASS, big_miss_whole);
        let sizes_note = precisely(&kit, stage, &sizes_text)?;
        wait_until(stage, bound_read)?;
        swap(stage, &[bound_note], &[sizes_note], 0.4)?;
        let sizes_read = read_by(stage, &sizes_text);
        line.wait_end(stage)?;
        pause(stage, 1.0)?;
        wait_until(stage, sizes_read)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

impl SceneConstruct for ThePriceOfTheCoin {
    fn name(&self) -> &str {
        "08_the_price_of_the_coin"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let a: Average = facts::average().map_err(fail)?;
        let (n, copiers) = (a.class.size(), a.class.copiers());
        let truth = Q::int(copiers as i128);
        let dist = model::yes_count_distribution(n, copiers).map_err(fail)?;
        let low = model::undo(0, n)[2];
        let channel = Channel::two_coins();
        let factor = Q::ONE / channel.truth();
        let estimate = factor * factor * model::variance(&dist);

        // 08_1: the fixed class; a sample from a big school adds who is picked
        let line = say(&kit, stage, "08_1")?;
        let mut fixed = Opening::build(&kit, stage, n, copiers, &dist)?.all();
        fixed.extend(bracket(&kit, stage, est_x(truth, low), estimate)?);
        fade_in_all(stage, &fixed, 0.5)?;
        let share = Q::new(copiers as i128, n as i128);
        let with_coins =
            model::sampling_variance(channel, share, 1).ok_or_else(|| fail("no sampling variance".into()))?;
        let cost = with_coins / model::direct_variance(share, 1);
        line.wait_for(stage, "Sample twenty")?;
        let coin_part = channel
            .coin_variance()
            .ok_or_else(|| fail("the two coins have no coin variance".into()))?;
        let formula = kit.tex(
            stage,
            &format!(
                r"\frac{{p(1-p)}}{{n}} + \frac{{{}}}{{{}n}}",
                coin_part.num(),
                coin_part.den()
            ),
            44.0,
            Palette::ink(),
            &[],
            v(FORMULA_X, FORMULA_Y[0]),
        )?;
        let picked = kit.text(stage, PICKED, 20.0, Palette::dim(), v(FORMULA_X, FORMULA_Y[1]))?;
        // a change of scale from 07_2's count: name it
        let share_label = kit.text(
            stage,
            SHARE,
            20.0,
            Palette::dim(),
            v(FORMULA_X, FORMULA_Y[0] + SHARE_DY),
        )?;
        let sample = precisely(&kit, stage, &sample_card(share, cost))?;
        fade_in_all(stage, &[share_label, formula, picked, sample], 0.4)?;

        // 08_2: the school's share in percentage points: nine against twenty-one
        let line = say(&kit, stage, "08_2")?;
        fade_out_all(stage, &fixed, 0.4)?;
        let direct_n = model::direct_variance(share, n as i128);
        let coins_n =
            model::sampling_variance(channel, share, n as i128).ok_or_else(|| fail("no sampling variance".into()))?;
        let points = |q: Q| 100.0 * q.to_f64().sqrt();
        let (direct_points, coin_points) = (points(direct_n), points(coins_n));
        let at = 100.0 * share.to_f64();
        let mut axis = vec![lane(
            stage,
            vec![v(SHARE_X0, SHARE_AXIS_Y), v(SHARE_X1, SHARE_AXIS_Y)],
            Palette::dim(),
        )];
        for tick in SHARE_TICKS {
            axis.push(kit.text(
                stage,
                &format!("{}%", signed_thousands(i128::from(tick))),
                18.0,
                Palette::dim(),
                v(share_x(f64::from(tick)), SHARE_AXIS_Y - SHARE_TICK_DY),
            )?);
        }
        axis.push(lane(
            stage,
            vec![v(share_x(at), SHARE_AXIS_Y), v(share_x(at), BRACKET_Y[0] + 0.15)],
            palette::copied(),
        ));
        fade_in_all(stage, &axis, 0.4)?;
        line.wait_for(stage, "nine percentage points")?;
        let direct_bracket = [
            span(
                stage,
                share_x(at - direct_points),
                share_x(at + direct_points),
                BRACKET_Y[0],
                Palette::ink(),
            ),
            kit.text(
                stage,
                &format!("{n} direct answers: about {direct_points:.2} points"),
                18.0,
                Palette::ink(),
                v(share_x(at), BRACKET_Y[0] + BRACKET_WORDS_DY),
            )?,
        ];
        fade_in_all(stage, &direct_bracket, 0.4)?;
        line.wait_for(stage, "twenty-one")?;
        let coin_bracket = [
            span(
                stage,
                share_x(at - coin_points),
                share_x(at + coin_points),
                BRACKET_Y[1],
                palette::forced(),
            ),
            kit.text(
                stage,
                &format!("{n} coin answers: about {coin_points:.2} points"),
                18.0,
                palette::forced(),
                v(share_x(at), BRACKET_Y[1] + BRACKET_WORDS_DY),
            )?,
        ];
        fade_in_all(stage, &coin_bracket, 0.4)?;
        line.wait_end(stage)?;
        // the two spreads stand together a moment (a named hold)
        pause(stage, 1.0)?;

        // 08_3: the prediction: how many coin answers miss by only nine points?
        let _line = say(&kit, stage, "08_3")?;
        let slider = kit.text(
            stage,
            &format!("{SLIDER}: {n}"),
            20.0,
            Palette::dim(),
            v(share_x(at), SLIDER_Y),
        )?;
        fade_in_all(stage, &[slider], 0.3)?;
        let q = think(&kit, stage, v(SHARE_THINK_X, SHARE_THINK_Y))?;

        // 08_4: a hundred and fourteen: almost six times as many people
        let line = say(&kit, stage, "08_4")?;
        let match_exact = with_coins / direct_n;
        let matched = model::at_least(match_exact);
        let at_match = model::sampling_variance(channel, share, matched).ok_or_else(|| fail("no variance".into()))?;
        if at_match > direct_n {
            return Err(fail(format!("{matched} coin answers do not match {n} direct ones")));
        }
        let match_points = points(at_match);
        let narrowed = [
            span(
                stage,
                share_x(at - match_points),
                share_x(at + match_points),
                BRACKET_Y[1],
                palette::forced(),
            ),
            kit.text(
                stage,
                &format!("{matched} coin answers: about {match_points:.2} points"),
                18.0,
                palette::forced(),
                v(share_x(at), BRACKET_Y[1] + BRACKET_WORDS_DY),
            )?,
            kit.text(
                stage,
                &format!("{SLIDER}: {matched}"),
                20.0,
                Palette::dim(),
                v(share_x(at), SLIDER_Y),
            )?,
        ];
        let mut old = coin_bracket.to_vec();
        old.extend([slider, q]);
        swap(stage, &old, &narrowed, 0.6)?;
        let working = [
            kit.tex(
                stage,
                &format!(
                    r"\frac{{{:.2}}}{{m}} = \frac{{{:.2}}}{{{n}}}",
                    with_coins.to_f64(),
                    model::direct_variance(share, 1).to_f64()
                ),
                30.0,
                Palette::ink(),
                &[],
                v(SHARE_THINK_X, MATCH_Y[0]),
            )?,
            kit.text(
                stage,
                &format!("m = {:.2}, so {matched}", match_exact.to_f64()),
                20.0,
                Palette::ink(),
                v(SHARE_THINK_X, MATCH_Y[1]),
            )?,
        ];
        fade_in_all(stage, &working, 0.4)?;
        line.wait_for(stage, "almost six times")?;
        let ratio = kit.tex(
            stage,
            &format!(r"{} \approx {:.2}", tex_q(cost, true), cost.to_f64()),
            36.0,
            Palette::ink(),
            &[],
            v(FORMULA_X, FORMULA_Y[2]),
        )?;
        let people = kit.text(stage, PEOPLE, 20.0, Palette::dim(), v(FORMULA_X, FORMULA_Y[3]))?;
        fade_in_all(stage, &[ratio, people], 0.4)?;
        line.wait_end(stage)?;
        pause(stage, 2.0)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::*;

    /// Every phrase the scenes wait for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scenes_wait_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("06_1", "Ask the class"),
            ("06_2", "minus four to fourteen"),
            ("06_2", "Only one lands"),
            ("06_3", "Four to the twentieth"),
            ("06_4", "around four"),
            ("06_6", "below zero"),
            ("06_7", "Rounding those up"),
            ("07_1", "A copier"),
            ("07_2", "Every answer wobbles"),
            ("07_3", "miss by about four"),
            ("07_4", "Square each"),
            ("07_5", "More than two"),
            ("07_6", "Does the estimate"),
            ("07_7", "one in twenty"),
            ("07_8", "As a share"),
            ("08_1", "Sample twenty"),
            ("08_2", "nine percentage points"),
            ("08_2", "twenty-one"),
            ("08_4", "almost six times"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The words on screen say what the facts say.
    #[test]
    fn the_cards_are_built_from_the_facts() {
        let f = facts::facts().unwrap();
        assert_eq!(
            runs_card(),
            "the first 8 draws from PCG64DXSM, seed 3, none thrown away; too few to show the rates"
        );
        let dist = model::yes_count_distribution(20, 4).unwrap();
        let rates = rates_card(4u128.pow(20), Q::int(4), dist[7], model::p_negative_estimate(&dist));
        assert!(
            rates.contains("over all 1,099,511,627,776 equally likely ways"),
            "{rates}"
        );
        assert!(
            rates.contains("exactly 4 about 20% of the time, below 0 about 9%"),
            "{rates}"
        );
        assert_eq!(f.get("R-F7.percent.exactly_right"), Some("20"));
        assert_eq!(f.get("R-F7.percent.negative"), Some("9"));
        let clamp = clamp_card(20, model::estimate_mean(&dist), model::clamped_mean(&dist));
        assert_eq!(
            clamp,
            "allow negatives: 2 × yes − 10 averages 4; replace them with 0: the average is 4.26"
        );
        assert_eq!(f.get("R-F14.estimate_mean"), Some("4"));
        assert_eq!(f.get("R-F14.clamped_mean_2dp"), Some("4.26"));
        let sd = sd_card(20, Q::new(3, 16), Q::new(15, 4), Q::int(15));
        assert!(
            sd.contains("3/16 for anyone; 20 make 15/4; doubling the count multiplies variance by 4: 15."),
            "{sd}"
        );
        assert!(sd.ends_with("Standard deviation: about 3.87"), "{sd}");
        // the defect Mail 3899 found: doubling does not double the variance
        assert!(!sd.contains("doubled, 15"), "{sd}");
        assert_eq!(Q::new(15, 4) * Q::int(4), Q::int(15));
        let typical = typical_card(15f64.sqrt());
        assert!(typical.ends_with("about 3.87"), "{typical}");
        assert_eq!(f.get("R-F7.sd_estimate"), Some("3.87"));
        let truth = Q::int(4);
        let bound = bound_card(
            Q::int(4),
            model::p_miss_at_least(&dist, truth, Q::int(4)),
            model::p_miss_at_least(&dist, truth, Q::int(8)),
        );
        assert_eq!(
            bound,
            "not a bound: 44% of outcomes miss by 4 or more, about 1 in 15 by 8 or more"
        );
        assert_eq!(f.get("R-F19.percent.miss_at_least"), Some("44"));
        assert_eq!(f.get("R-F19.one_in.miss_at_least_twice"), Some("15"));
        let sizes = sizes_card(20, 15f64.sqrt(), BIG_CLASS, 15);
        assert!(sizes.ends_with("20: 3.87 (19% of 20); 300: 15 (5% of 300)"), "{sizes}");
        assert_eq!(f.get("R-F19.class.percent.miss_share"), Some("19"));
        assert_eq!(f.get("R-F19.big.percent.miss_share"), Some("5"));
        assert_eq!(f.get("R-F19.big.miss"), Some("15"));
        let sample = sample_card(Q::new(1, 5), Q::new(91, 16));
        assert!(
            sample.ends_with("At p = 1/5 that is 91/16, about 5.69 times"),
            "{sample}"
        );
        assert_eq!(f.get("R-F8.cost"), Some("91/16"));
        assert_eq!(tex_thousands(4u128.pow(20)), "1{,}099{,}511{,}627{,}776");
        // chapter 08's points and match are the facts'
        let share = Q::new(1, 5);
        let points = |q: Q| format!("{:.2}", 100.0 * q.to_f64().sqrt());
        assert_eq!(
            f.get("R-F20.direct.points_2dp"),
            Some(points(model::direct_variance(share, 20)).as_str())
        );
        let coins = model::sampling_variance(Channel::two_coins(), share, 20).unwrap();
        assert_eq!(f.get("R-F20.coins.points_2dp"), Some(points(coins).as_str()));
        let exact =
            model::sampling_variance(Channel::two_coins(), share, 1).unwrap() / model::direct_variance(share, 20);
        assert_eq!(f.get("R-F20.match"), Some(model::at_least(exact).to_string().as_str()));
    }

    /// The cards stay two lines at every wrap the lower-third can narrow to.
    #[test]
    fn the_cards_are_two_lines() {
        let dist = model::yes_count_distribution(20, 4).unwrap();
        let truth = Q::int(4);
        for text in [
            runs_card(),
            rates_card(4u128.pow(20), Q::int(4), dist[7], model::p_negative_estimate(&dist)),
            clamp_card(20, model::estimate_mean(&dist), model::clamped_mean(&dist)),
            sd_card(20, Q::new(3, 16), Q::new(15, 4), Q::int(15)),
            typical_card(15f64.sqrt()),
            bound_card(
                Q::int(4),
                model::p_miss_at_least(&dist, truth, Q::int(4)),
                model::p_miss_at_least(&dist, truth, Q::int(8)),
            ),
            sizes_card(20, 15f64.sqrt(), BIG_CLASS, 15),
            sample_card(Q::new(1, 5), Q::new(91, 16)),
        ] {
            for width in (72..=92).rev().step_by(4) {
                let lines = explainer_kit::animatic::wrap(&text, width, 3);
                assert!(lines.len() <= 2, "{} lines at {width}: {text}", lines.len());
            }
        }
    }

    /// The scene draws the facts' runs: their yes counts and estimates.
    #[test]
    fn the_rows_are_the_facts_runs() {
        let a = facts::average().unwrap();
        let runs = facts::runs().unwrap();
        let yeses: Vec<usize> = runs
            .iter()
            .map(|coins| Tally::of(&a.class, coins).unwrap().answers(true))
            .collect();
        let f = facts::facts().unwrap();
        assert_eq!(Some(model::csv(&yeses).as_str()), f.get("R-F13.yes"));
        let estimates: Vec<Q> = yeses.iter().map(|&y| model::undo(y, 20)[2]).collect();
        assert_eq!(Some(model::csv(&estimates).as_str()), f.get("R-F13.estimates"));
    }

    /// The layout keeps its pieces apart and inside the frame.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        const BAND_TOP: f64 = -FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36 * 2.0;
        let n = model::CLASS_SIZE;
        // the rows: inside the frame, over the band, clear of their columns
        let last_dot = RUN_X0 + RUN_DX * (n - 1) as f64;
        const {
            assert!(
                RUN_X0 - DOT_RING > -FRAME_X_RADIUS + 0.15,
                "the first ring inside the left edge"
            )
        };
        assert!(last_dot + DOT_RING < YES_COL_X - 0.4);
        const {
            assert!(
                RUN_Y0 - RUN_DY * (RUNS - 1) as f64 - DOT_RING > BAND_TOP + 0.3,
                "the last row over the band"
            )
        };
        const { assert!(RUN_Y0 + DOT_RING < HEAD_Y - 0.2, "the rows under their heads") };
        const { assert!(HEAD_Y + 0.15 < FRAME_Y_RADIUS - 0.15, "the heads inside the top edge") };
        const { assert!(RUN_DX > 2.0 * DOT_R + 0.05, "the dots apart") };
        // the axis: inside the frame; its labels over the band
        let low = model::undo(0, n)[2];
        let high = model::undo(n, n)[2];
        assert!(est_x(low, low) - BAR_W / 2.0 > ROW_NAME_X + 0.95);
        const {
            assert!(
                ROW_NAME_X > -FRAME_X_RADIUS + 0.25,
                "the row names inside the title-safe edge"
            )
        };
        assert!(est_x(high, low) + 0.3 < FRAME_X_RADIUS - 0.25);
        const { assert!(AXIS_Y - EST_TICK_DY - 0.15 > BAND_TOP, "the estimates over the band") };
        // the tallest bar under the truth's words, which sit under the bracket
        let dist = model::yes_count_distribution(n, model::COPIERS.len()).unwrap();
        let tallest = dist.iter().map(|p| p.to_f64()).fold(0.0, f64::max) * BAR_SCALE;
        assert!(AXIS_Y + tallest < TRUTH_LABEL_Y - 0.2);
        const {
            assert!(
                TRUTH_LABEL_Y + 0.15 < SD_Y - 0.12,
                "the truth's words under the bracket"
            )
        };
        const { assert!(SD_LABEL_Y + 0.2 < FRAME_Y_RADIUS - 0.15, "the bracket's label inside") };
        // the notes (up to x −1.9) clear of the bars under them, and of the
        // truth's words (their top about 0.12 over TRUTH_LABEL_Y)
        let near_notes = dist
            .iter()
            .enumerate()
            .filter(|&(y, _)| est_x(model::undo(y, n)[2], low) < -1.9)
            .map(|(_, p)| p.to_f64())
            .fold(0.0, f64::max)
            * BAR_SCALE;
        assert!(AXIS_Y + near_notes < NOTE_Y[2] - 0.2);
        const {
            assert!(
                NOTE_Y[2] - 0.12 > TRUTH_LABEL_Y + 0.12 + 0.1,
                "the notes over the truth's words"
            )
        };
        // the right side's pieces clear of the bars there
        let right_bars = dist
            .iter()
            .enumerate()
            .filter(|&(y, _)| est_x(model::undo(y, n)[2], low) > 1.4)
            .map(|(_, p)| p.to_f64())
            .fold(0.0, f64::max)
            * BAR_SCALE;
        assert!(AXIS_Y + right_bars < FORMULA_Y[3] - 0.2);
        assert!(AXIS_Y + right_bars < CAPTION_Y[1] - 0.2);
        // the share label over its formula (a 44-point fraction, about 0.45
        // above its centre), inside the top edge
        const {
            assert!(
                FORMULA_Y[0] + SHARE_DY - 0.12 > FORMULA_Y[0] + 0.45,
                "the label over the formula"
            )
        };
        const {
            assert!(
                FORMULA_Y[0] + SHARE_DY + 0.15 < FRAME_Y_RADIUS - 0.15,
                "the label inside the top edge"
            )
        };
        assert!(AXIS_Y + right_bars < THINK_Y - 0.62 - 0.45);
        // chapter 07's rows at the upper left, clear of the bracket's label
        let truth_x = est_x(Q::int(4), low);
        const { assert!(ROWS_NAME_X > -FRAME_X_RADIUS + 0.25, "the rows' names inside") };
        const {
            assert!(
                ROWS_X0 - ROWS_SIDE / 2.0 > ROWS_NAME_X + 0.8,
                "the squares right of the names"
            )
        };
        // the variances (about 0.3 wide) left of the bracket's label (about 1.1
        // either side of the truth)
        assert!(ROWS_VAR_X + 0.3 < truth_x - 1.15);
        const {
            assert!(
                ROWS_Y[0] + ROWS_SIDE / 2.0 < FRAME_Y_RADIUS - 0.25,
                "the rows inside the top"
            )
        };
        const { assert!(ROWS_Y[1] - ROWS_SIDE / 2.0 > SD_Y + 0.05, "the rows over the bracket") };
        // the typical miss's caption inside the right edge, over the bars there
        const {
            assert!(
                TYPICAL_X + 2.8 < FRAME_X_RADIUS - 0.25,
                "the caption inside the right edge"
            )
        };
        assert!(AXIS_Y + right_bars < DERIVE_Y[3] - 0.2);
        // the three hundred's panel at the upper right, right of the bracket
        assert!(BIG_X0 - 0.3 > truth_x + UNIT * 15f64.sqrt() + 0.5);
        const { assert!(BIG_X1 + 0.3 < FRAME_X_RADIUS - 0.25, "the line inside the right edge") };
        const { assert!(BIG_TITLE_Y + 0.15 < FRAME_Y_RADIUS - 0.25, "the title inside the top") };
        const { assert!(BIG_TITLE_Y - 0.15 > BIG_Y + 0.3 + 0.1, "the title over the bracket") };
        const {
            assert!(
                BIG_ROOT_Y + 0.3 < BIG_Y - BIG_WORDS_DY - 0.12,
                "the root under the mark's words"
            )
        };
        const {
            assert!(
                BIG_THINK_Y + 0.62 < BIG_Y - BIG_MARK_H - 0.05,
                "the think ring under the line"
            )
        };
        const {
            assert!(
                SHARE_BAR_Y[0] + 0.15 < BIG_ROOT_Y - 0.3,
                "the share bars under the root"
            )
        };
        const {
            assert!(
                CAPTION_Y[0] + 0.15 < SHARE_BAR_Y[1] - 0.15,
                "the captions under the bars"
            )
        };
        assert!(big_x(60.0 + 15.0) < BIG_X1 && big_x(60.0 - 15.0) > BIG_X0);
        // the widest share bar and its words inside the right edge (about 2.5 wide)
        assert!(SHARE_BAR_X0 + SHARE_BAR_SCALE * 0.2 + SHARE_WORDS_DX + 2.6 < FRAME_X_RADIUS - 0.25);
        // chapter 08's share axis at the left, under the formula's column
        const { assert!(SHARE_X0 > -FRAME_X_RADIUS + 0.5, "the axis inside the left edge") };
        const { assert!(SHARE_X1 + 0.4 < FORMULA_X - 2.0, "the axis left of the formulas") };
        assert!(share_x(20.0 - 21.33) > SHARE_X0 && share_x(20.0 + 21.33) < SHARE_X1);
        const {
            assert!(
                BRACKET_Y[0] + BRACKET_WORDS_DY + 0.15 < FRAME_Y_RADIUS - 0.25,
                "inside the top"
            )
        };
        const {
            assert!(
                BRACKET_Y[1] + BRACKET_WORDS_DY + 0.12 < BRACKET_Y[0] - 0.1,
                "the brackets' words apart"
            )
        };
        const { assert!(SHARE_AXIS_Y + 0.1 < BRACKET_Y[1] - 0.1, "the axis under the brackets") };
        const {
            assert!(
                SLIDER_Y + 0.12 < SHARE_AXIS_Y - SHARE_TICK_DY - 0.12,
                "the slider under the ticks"
            )
        };
        const { assert!(SHARE_THINK_Y + 0.62 < SLIDER_Y - 0.1, "the think ring under the slider") };
        const { assert!(SHARE_THINK_Y - 0.62 - 0.45 > BAND_TOP, "the think note over the band") };
        const { assert!(MATCH_Y[1] - 0.15 > BAND_TOP, "the match's words over the band") };
    }
}
