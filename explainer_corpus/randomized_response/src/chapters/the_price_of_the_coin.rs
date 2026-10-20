//! Chapter 03, the price of the coin: the same class asked eight times with
//! fresh coins, real seeded draws, estimates from minus four to fourteen
//! (03_1, 03_2); every one of the 4²⁰ equally likely ways the coins can land,
//! counted into a distribution of yes counts and estimates (03_3, 03_4);
//! the prediction, how often exactly four (03_5), and its answer, about one
//! class in five, with one in eleven below zero (03_6); what below zero
//! means, and why rounding it up would be wrong (03_7); the fixed class's
//! scatter, √15 (03_8); and how a sample from a big school differs (03_9,
//! 03_10).
//!
//! Every row is `model::answer` on a real draw (`facts::runs`); every bar is
//! the exact chance of its yes count (`model::yes_count_distribution`), its
//! estimate `model::undo`'s; the numbers drawn are checked against the facts.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, signed_thousands, thousands, v};
use fmn::prelude::*;

use super::draw::{
    bar, coin, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, grow_lagged, lane, outline, pause, say,
    swap, tex_q, think, wait_until,
};
use crate::facts::{self, Average, RUNS, SEED};
use crate::model::{self, COIN_PAIRS, Channel, Q, Tally};
use crate::palette;

pub struct ThePriceOfTheCoin {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const YES_HEAD: &str = "yes";
const ESTIMATE_HEAD: &str = "estimate";
const PICKED: &str = "who gets picked, plus the coins";
/// 03_9's formula is the share's variance, not the count's (03_8): its label.
const SHARE: &str = "variance of the estimated share";
const PEOPLE: &str = "times as many people, for the same precision";

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
const AXIS_Y: f64 = -1.2;
const BAR_W: f64 = 0.44;
const BAR_SCALE: f64 = 12.75;
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
const NOTE_X: f64 = -6.4;
const NOTE_Y: [f64; 3] = [2.95, 2.6, 2.25];
const RIGHT_X: f64 = 4.4;
const DERIVE_Y: [f64; 3] = [3.0, 2.45, 1.9];
const FORMULA_X: f64 = 3.6;
const FORMULA_Y: [f64; 4] = [2.8, 2.1, 1.4, 0.85];
/// The share label's height over the formula.
const SHARE_DY: f64 = 0.62;
const TOTAL_X: f64 = 3.0;
const TOTAL_Y: f64 = 3.2;
const THINK_X: f64 = 5.4;
const THINK_Y: f64 = 1.6;

/// Where estimate `e` sits on the axis, the lowest estimate (`low`) at the
/// left end.
fn est_x(e: Q, low: Q) -> f64 {
    AXIS_X0 + UNIT * (e - low).to_f64()
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

/// 03_2's exact version: where the eight classes come from. Short, since it
/// is on screen about 5 s (draft06 had 126 characters in 5.13 s; Mail 3941).
fn runs_card() -> String {
    format!("the first {RUNS} draws from PCG64DXSM, seed {SEED}, none thrown away; too few to show the rates")
}

/// 03_6's exact version: the two chances over every way the coins can land.
fn rates_card(outcomes: u128, truth: Q, exactly: Q, negative: Q) -> String {
    format!(
        "over all {} equally likely ways the coins can land: exactly {truth} about {}% of the time, below 0 \
         about {}%",
        thousands(outcomes),
        exactly.percent(),
        negative.percent()
    )
}

/// 03_7's exact version: with its negative values allowed, the estimate
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

/// 03_8's exact version: the fixed class's standard deviation, step by
/// step. Doubling the count multiplies its variance by the factor squared
/// (4), not by 2: the same step as the TeX derivation beside it
/// (FuchsiaHawk, Mail 3899, caught "doubled, 15").
fn sd_card(n: usize, answer: Q, count: Q, estimate: Q) -> String {
    let factor = Q::ONE / Channel::two_coins().truth();
    format!(
        "each answer's variance is {answer} for anyone; {n} make {count}; doubling the count multiplies variance by \
         {}: {estimate}. Standard deviation: about {:.2}",
        factor * factor,
        estimate.to_f64().sqrt()
    )
}

/// 03_9's exact version, on screen from "Sample twenty" to the end: what the
/// sampling formula, and 03_10's "almost six times as many people", assume.
/// Placed late in 03_10 it had 4.20 s for 143 characters (Mail 3941).
fn sample_card(share: Q, cost: Q) -> String {
    format!(
        "assumes independent draws, with replacement or from a large population, and truthful direct answers. \
         At p = {share} that is {cost}, about {:.2} times",
        cost.to_f64()
    )
}

/// Redraw the bars whose estimate `which` picks in `color` (fresh bars over
/// the old ones, the same heights), returning the old and the new to swap.
fn recolor_bars(
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

impl SceneConstruct for ThePriceOfTheCoin {
    fn name(&self) -> &str {
        "03_the_price_of_the_coin"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let a: Average = facts::average().map_err(fail)?;
        let (n, copiers) = (a.class.size(), a.class.copiers());
        let truth = Q::int(copiers as i128);
        let runs = facts::runs().map_err(fail)?;
        let dist = model::yes_count_distribution(n, copiers).map_err(fail)?;
        let low = model::undo(0, n)[2];

        // 03_1: real coins; the class asked eight times
        let line = say(&kit, stage, "03_1")?;
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

        // 03_2: their estimates, from the lowest to the highest
        let line = say(&kit, stage, "03_2")?;
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
        let runs_read_until = stage.scene().time().to_f64() + runs_text.chars().count() as f64 / 15.0;
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

        // 03_3: every way the coins can land; the distribution rises
        wait_until(stage, runs_read_until)?;
        let line = say(&kit, stage, "03_3")?;
        panel.push(card);
        fade_out_all(stage, &panel, 0.5)?;
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
        let mut bars = Vec::with_capacity(n + 1);
        for (y, &p) in dist.iter().enumerate() {
            let h = BAR_SCALE * p.to_f64();
            if h < MIN_BAR {
                continue;
            }
            let x = est_x(model::undo(y, n)[2], low);
            bars.push((y, bar(stage, Palette::dim(), x, AXIS_Y, BAR_W, h)?));
        }
        let bar_mobs: Vec<Mob> = bars.iter().map(|&(_, m)| m).collect();
        grow_lagged(stage, &bar_mobs, DOWN, 2.0, 0.15)?;

        // 03_4: piled up around four, the true count
        let line = say(&kit, stage, "03_4")?;
        let mean = model::estimate_mean(&dist);
        if mean != truth {
            return Err(fail("the distribution's estimate does not average the truth".into()));
        }
        let truth_x = est_x(truth, low);
        let truth_line = lane(
            stage,
            vec![v(truth_x, AXIS_Y), v(truth_x, AXIS_Y + TRUTH_H)],
            palette::copied(),
        );
        let truth_label = kit.text(
            stage,
            &format!("{copiers} copied"),
            22.0,
            palette::copied(),
            v(truth_x, TRUTH_LABEL_Y),
        )?;
        fade_in_all(stage, &[truth_line, truth_label], 0.4)?;
        line.wait_for(stage, "around four")?;
        let near: Vec<Mob> = bars
            .iter()
            .filter(|&&(y, _)| (model::undo(y, n)[2] - truth).to_f64().abs() <= 2.0)
            .map(|&(_, m)| m)
            .collect();
        flash_all(stage, &near, 0.6)?;

        // 03_5: the prediction; no chance is on screen yet
        let _line = say(&kit, stage, "03_5")?;
        fade_out_all(stage, &[total], 0.3)?;
        let q = think(&kit, stage, v(THINK_X, THINK_Y))?;

        // 03_6: about one class in five; about one in eleven below zero
        let line = say(&kit, stage, "03_6")?;
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
        let rates_read_until = stage.scene().time().to_f64() + rates_text.chars().count() as f64 / 15.0;

        // 03_7: below zero, honestly; rounding up would push the average up
        wait_until(stage, rates_read_until)?;
        let line = say(&kit, stage, "03_7")?;
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
        let clamp_read_until = stage.scene().time().to_f64() + clamp_text.chars().count() as f64 / 15.0;

        // 03_8: every answer wobbles the same; twenty, doubled: √15
        wait_until(stage, clamp_read_until)?;
        let line = say(&kit, stage, "03_8")?;
        let answer = Channel::two_coins().answer_variance(true);
        let count = model::variance(&dist);
        let factor = Q::ONE / Channel::two_coins().truth();
        let estimate = factor * factor * count;
        if answer * Q::int(n as i128) != count {
            return Err(fail(format!("{n} answers of {answer} do not make {count}")));
        }
        let sd_text = sd_card(n, answer, count, estimate);
        let sd = precisely(&kit, stage, &sd_text)?;
        fade_out_all(stage, &[note_exactly, note_negative, note_clamped, clamp], 0.2)?;
        fade_in_all(stage, &[sd], 0.2)?;
        let sd_read_until = stage.scene().time().to_f64() + sd_text.chars().count() as f64 / 15.0;
        let steps = [
            format!(r"{} \times {n} = {}", tex_q(answer, true), tex_q(count, true)),
            format!(r"{} \times {} = {estimate}", tex_q(count, true), factor * factor),
            format!(r"\sqrt{{{estimate}}} \approx {:.2}", estimate.to_f64().sqrt()),
        ];
        let mut derivation = Vec::with_capacity(steps.len());
        for (src, &y) in steps.iter().zip(&DERIVE_Y) {
            derivation.push(kit.tex(stage, src, 34.0, Palette::ink(), &[], v(RIGHT_X, y))?);
        }
        fade_in_lagged(stage, &derivation, 1.5, 0.6)?;
        line.wait_for(stage, "miss by about four")?;
        let spread = estimate.to_f64().sqrt();
        let (left, right) = (truth_x - UNIT * spread, truth_x + UNIT * spread);
        let bracket = lane(
            stage,
            vec![
                v(left, SD_Y - 0.12),
                v(left, SD_Y),
                v(right, SD_Y),
                v(right, SD_Y - 0.12),
            ],
            Palette::ink(),
        );
        let bracket_label = kit.tex(
            stage,
            &format!(r"\sqrt{{{estimate}}} \approx {:.2}", spread),
            30.0,
            Palette::ink(),
            &[],
            v(truth_x, SD_LABEL_Y),
        )?;
        fade_in_all(stage, &[bracket, bracket_label], 0.4)?;

        // 03_9: a sample from a big school adds who gets picked; the fixed
        // class's card goes, and the sample's assumptions stay to the end
        wait_until(stage, sd_read_until)?;
        let line = say(&kit, stage, "03_9")?;
        let share = Q::new(copiers as i128, n as i128);
        let with_coins = model::sampling_variance(Channel::two_coins(), share, 1)
            .ok_or_else(|| fail("no sampling variance".into()))?;
        let cost = with_coins / model::direct_variance(share, 1);
        line.wait_for(stage, "Sample twenty")?;
        let coin_part = Channel::two_coins()
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
        // a change of scale from 03_8's count: name it
        let share_label = kit.text(
            stage,
            SHARE,
            20.0,
            Palette::dim(),
            v(FORMULA_X, FORMULA_Y[0] + SHARE_DY),
        )?;
        let sample = precisely(&kit, stage, &sample_card(share, cost))?;
        let mut fixed_class = derivation;
        fixed_class.push(sd);
        fade_out_all(stage, &fixed_class, 0.2)?;
        fade_in_all(stage, &[share_label, formula, picked, sample], 0.2)?;

        // 03_10: at one in five, almost six times the people
        let line = say(&kit, stage, "03_10")?;
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

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("03_1", "Ask the class"),
            ("03_2", "minus four to fourteen"),
            ("03_2", "Only one lands"),
            ("03_3", "Four to the twentieth"),
            ("03_4", "around four"),
            ("03_6", "below zero"),
            ("03_7", "Rounding those up"),
            ("03_8", "miss by about four"),
            ("03_9", "Sample twenty"),
            ("03_10", "almost six times"),
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
        let sample = sample_card(Q::new(1, 5), Q::new(91, 16));
        assert!(
            sample.ends_with("At p = 1/5 that is 91/16, about 5.69 times"),
            "{sample}"
        );
        assert_eq!(f.get("R-F8.cost"), Some("91/16"));
        assert_eq!(tex_thousands(4u128.pow(20)), "1{,}099{,}511{,}627{,}776");
    }

    /// The cards stay two lines at every wrap the lower-third can narrow to.
    #[test]
    fn the_cards_are_two_lines() {
        let dist = model::yes_count_distribution(20, 4).unwrap();
        for text in [
            runs_card(),
            rates_card(4u128.pow(20), Q::int(4), dist[7], model::p_negative_estimate(&dist)),
            clamp_card(20, model::estimate_mean(&dist), model::clamped_mean(&dist)),
            sd_card(20, Q::new(3, 16), Q::new(15, 4), Q::int(15)),
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
        // the share's label over its formula (a 44-point fraction, about 0.45
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
    }
}
