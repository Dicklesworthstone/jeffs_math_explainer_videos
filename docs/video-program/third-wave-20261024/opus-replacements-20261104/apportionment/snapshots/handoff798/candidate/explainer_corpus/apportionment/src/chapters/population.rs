//! 13 The population promise: a later census (Ash shrinks to 3,900, Birch
//! grows to 9,500, Cedar to 26,100) at sixteen seats; Hamilton moves a seat
//! from Cedar, which grew, to Ash, which shrank; divisor rules cannot,
//! because a growing town's signposts only rise and a shrinking town's only
//! fall. The cards move through the real lists (`model::signposts` before,
//! with only the growth, and at the census).

use explainer_kit::{Kit, Palette, live_bar, rect, v};
use fmn::prelude::*;

use super::draw::{
    GRID_X, LIST_X, ROW_Y, SCORE_LABEL_X, SCORE_STEP, SCORE_X, band, cutoff, fade_in_all, fade_in_lagged, fade_out_all,
    fail, flash_all, flat, grid, keyed_cards, morph_all, move_cards, name, pause, readout, refill, say, score_head,
    score_row, think_here, token, tokens, town_color, wipe,
};
use crate::model::{self, ASH, BIRCH, CEDAR, NAMES, POP, POP_GROWN, POP2, Rule, decimal, grouped, share};

pub struct Population {
    pub kit: Kit,
}

/// Population bars, as in chapter 1: 26,000 people span this many units.
const POP_SCALE: f64 = 5.6 / 26_000.0;
const BAR_H: f64 = 0.36;
const COUNT_X: f64 = 2.2;
const NOTE_X: f64 = 3.2;
/// How many signposts chapter 13's list shows.
const SHOWN: usize = 18;

/// The three seat counters left of the list, detached.
fn counters(kit: &Kit, stage: &mut Stage<'_>, c: &[u64]) -> fmn::Result<Vec<Mob>> {
    (0..3)
        .map(|i| kit.text(stage, &c[i].to_string(), 44.0, town_color(i), v(-2.2, ROW_Y[i])))
        .collect()
}

fn triple(s: &[u64]) -> String {
    s.iter().map(u64::to_string).collect::<Vec<_>>().join(", ")
}

impl SceneConstruct for Population {
    fn name(&self) -> &str {
        "13_population"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let before = model::hamilton(&POP, 16).map_err(fail)?;
        let after = model::hamilton(&POP2, 16).map_err(fail)?;
        if !(after[CEDAR] < before[CEDAR]
            && after[ASH] > before[ASH]
            && POP2[CEDAR] > POP[CEDAR]
            && POP2[ASH] < POP[ASH])
        {
            return Err(fail(
                "13_4 says Cedar, which grew, loses a seat to Ash, which shrank".into(),
            ));
        }
        let j_before = model::divisor(Rule::Jefferson, &POP, 16).map_err(fail)?;
        let j_after = model::divisor(Rule::Jefferson, &POP2, 16).map_err(fail)?;
        if j_before != j_after {
            return Err(fail(
                "13_5 shows Jefferson's sixteen seats unchanged by the census".into(),
            ));
        }

        // 13_1 three towns, the third promise
        let s = say(&kit, stage, "13_1")?;
        let mut names = Vec::new();
        let mut bars = Vec::new();
        let mut counts = Vec::new();
        for i in 0..3 {
            names.extend(name(&kit, stage, i, ROW_Y[i], "")?);
            let w = POP[i] as f64 * POP_SCALE;
            let bar = rect(v(GRID_X, ROW_Y[i] - BAR_H / 2.0), w, BAR_H, flat(town_color(i), 0.85))?;
            bars.push(stage.arena_mut().add(bar));
            counts.push(kit.text(
                stage,
                &grouped(u128::from(POP[i])),
                24.0,
                town_color(i),
                v(COUNT_X, ROW_Y[i]),
            )?);
        }
        let mut first = names.clone();
        first.extend(&bars);
        first.extend(&counts);
        fade_in_lagged(stage, &first, 1.2, 0.1)?;
        s.wait_for(stage, "which suggests a third promise")?;
        let promise_t = kit.text(stage, "population promise", 30.0, Palette::hero(), v(0.0, 3.3))?;
        fade_in_all(stage, &[promise_t], 0.5)?;
        s.wait_for(stage, "A town that did not shrink")?;
        let promise = kit.text(
            stage,
            "a town that did not shrink never loses a seat to a town that did not grow",
            22.0,
            Palette::ink(),
            v(0.0, 2.65),
        )?;
        fade_in_all(stage, &[promise], 0.6)?;

        // 13_2 a later census; the council stays at sixteen
        let s = say(&kit, stage, "13_2")?;
        let trackers: Vec<Mob> = (0..3).map(|i| stage.add_value_tracker(POP[i] as f64)).collect();
        let mut live = Vec::new();
        for i in 0..3 {
            live.push(live_bar(
                stage,
                trackers[i],
                v(GRID_X, ROW_Y[i]),
                POP_SCALE,
                BAR_H,
                town_color(i),
            )?);
            live.push(kit.live_count(stage, trackers[i], 24.0, town_color(i), v(COUNT_X, ROW_Y[i]))?);
        }
        let mut statics = bars.clone();
        statics.extend(&counts);
        fade_out_all(stage, &statics, 0.2)?;
        let head = kit.text(stage, "later census", 24.0, Palette::dim(), v(-4.6, 3.3))?;
        s.wait_for(stage, "Take a later census")?;
        fade_out_all(stage, &[promise], 0.3)?;
        fade_in_all(stage, &[head], 0.3)?;
        let mut notes = [None, None, None];
        for (i, phrase) in [
            (ASH, "Ash shrinks"),
            (BIRCH, "Birch grows"),
            (CEDAR, "Cedar grows a little"),
        ] {
            s.wait_for(stage, phrase)?;
            super::draw::ease_tracker(stage, trackers[i], POP2[i] as f64, 1.4)?;
            let (word, color) = if POP2[i] < POP[i] {
                ("shrank", Palette::red())
            } else {
                ("grew", Palette::green())
            };
            let note = kit.text_left(stage, word, 22.0, color, v(NOTE_X, ROW_Y[i]))?;
            fade_in_all(stage, &[note], 0.3)?;
            notes[i] = Some(note);
        }
        let notes: Vec<Mob> = notes.into_iter().flatten().collect();
        s.wait_for(stage, "The council stays at sixteen seats")?;
        let seats16 = kit.text(stage, "16 seats", 30.0, Palette::ink(), v(1.2, 3.3))?;
        fade_in_all(stage, &[seats16], 0.4)?;

        // 13_3 before the census Hamilton gave 1, 4, 11; now?
        let s = say(&kit, stage, "13_3")?;
        let mut grids = Vec::new();
        let mut dims = Vec::new();
        for i in 0..3 {
            grids.extend(grid(stage, ROW_Y[i], 12)?);
            let w = POP2[i] as f64 * POP_SCALE;
            let bar = rect(v(GRID_X, ROW_Y[i] - BAR_H / 2.0), w, BAR_H, flat(town_color(i), 0.12))?;
            dims.push(stage.arena_mut().add(bar));
        }
        let mut swap = dims.clone();
        swap.extend(&grids);
        fade_out_all(stage, &live, 0.4)?;
        fade_in_all(stage, &swap, 0.5)?;
        s.wait_for(stage, "Hamilton gave one, four, eleven")?;
        let mut toks: Vec<Vec<Mob>> = Vec::new();
        for i in 0..3 {
            toks.push(tokens(stage, ROW_Y[i], 0, before[i] as usize, town_color(i))?);
        }
        let all_toks: Vec<Mob> = toks.iter().flatten().copied().collect();
        fade_in_lagged(stage, &all_toks, 1.2, 0.1)?;
        let was = kit.text(
            stage,
            &format!("before: {}", triple(&before)),
            24.0,
            Palette::dim(),
            v(5.0, 3.3),
        )?;
        fade_in_all(stage, &[was], 0.4)?;
        let q = think_here(&kit, stage)?;

        // 13_4 now 2, 4, 10: Cedar, which grew, loses a seat to Ash
        let s = say(&kit, stage, "13_4")?;
        fade_out_all(stage, &[q], 0.2)?;
        let gain = token(stage, ROW_Y[ASH], before[ASH] as usize, Palette::hero())?;
        let lost = *toks[CEDAR].last().ok_or_else(|| fail("no Cedar tokens".into()))?;
        refill(stage, &[(lost, Palette::red(), 1.0)], 0.3)?;
        fade_in_all(stage, &[gain], 0.4)?;
        fade_out_all(stage, &[lost], 0.4)?;
        let now = kit.text(
            stage,
            &format!("now: {}", triple(&after)),
            24.0,
            Palette::hero(),
            v(5.0, 2.85),
        )?;
        fade_in_all(stage, &[now], 0.4)?;
        s.wait_for(stage, "The new shares are")?;
        let mut new_bands = Vec::new();
        let mut reads = Vec::new();
        for i in 0..3 {
            let qi = share(&POP2, i, 16);
            new_bands.push(band(stage, ROW_Y[i], qi.to_f64(), town_color(i))?);
            reads.push(readout(&kit, stage, &decimal(qi, 2), ROW_Y[i], town_color(i))?);
        }
        let mut shown_bands: Vec<Mob> = new_bands.iter().flatten().copied().collect();
        shown_bands.extend(&reads);
        fade_in_lagged(stage, &shown_bands, 1.4, 0.15)?;
        s.wait_for(stage, "Ash's tail now beats Cedar's")?;
        if share(&POP2, ASH, 16).tail() <= share(&POP2, CEDAR, 16).tail() {
            return Err(fail("13_4 says Ash's tail beats Cedar's".into()));
        }
        flash_all(
            stage,
            &[new_bands[ASH][1], new_bands[CEDAR][1], reads[ASH], reads[CEDAR]],
            1.0,
        )?;
        s.wait_for(stage, "so Cedar, which grew")?;
        flash_all(stage, &notes, 1.0)?;
        let broken = kit.text(
            stage,
            "the population promise is broken",
            26.0,
            Palette::red(),
            v(0.0, -2.8),
        )?;
        fade_in_all(stage, &[broken], 0.5)?;

        // 13_5 divisor rules cannot: signposts follow populations
        wipe(&kit, stage)?;
        let s = say(&kit, stage, "13_5")?;
        let shown = keyed_cards(&kit, stage, Rule::Jefferson, &POP, SHOWN)?;
        let mut left = Vec::new();
        for i in 0..3 {
            left.push(kit.text_left(stage, NAMES[i], 30.0, town_color(i), v(-5.8, ROW_Y[i]))?);
        }
        let c_before = counters(&kit, stage, &j_before)?;
        let line = cutoff(stage, 16);
        let list_head = kit.text(
            stage,
            "Jefferson's list, 16 seats",
            20.0,
            Palette::dim(),
            v(LIST_X, 3.78),
        )?;
        let mut first: Vec<Mob> = shown.iter().flat_map(|(_, m)| m.iter().copied()).collect();
        first.extend(&left);
        first.push(list_head);
        fade_in_lagged(stage, &first, 1.4, 0.05)?;
        fade_in_all(stage, &[vec![line], c_before.clone()].concat(), 0.4)?;
        s.wait_for(stage, "Every signpost is a population divided")?;
        let rule = kit.text(
            stage,
            "signpost = population ÷ seat number",
            22.0,
            Palette::ink(),
            v(-3.6, 3.3),
        )?;
        fade_in_all(stage, &[rule], 0.5)?;
        s.wait_for(stage, "When a town grows")?;
        let grown = keyed_cards(&kit, stage, Rule::Jefferson, &POP_GROWN, SHOWN)?;
        let shown = move_cards(stage, shown, grown, 1.4)?;
        s.wait_for(stage, "When a town shrinks")?;
        let census = keyed_cards(&kit, stage, Rule::Jefferson, &POP2, SHOWN)?;
        let shown = move_cards(stage, shown, census, 1.2)?;
        s.wait_for(stage, "can only gain ground")?;
        let ash_cards: Vec<Mob> = shown
            .iter()
            .filter(|((t, _), _)| *t == ASH)
            .flat_map(|(_, m)| m[1..].to_vec())
            .collect();
        let grow_cards: Vec<Mob> = shown
            .iter()
            .filter(|((t, k), _)| (*t == BIRCH && *k == 4) || (*t == CEDAR && *k == 11))
            .flat_map(|(_, m)| m[1..].to_vec())
            .collect();
        flash_all(stage, &grow_cards, 0.9)?;
        flash_all(stage, &ash_cards, 0.9)?;
        s.wait_for(stage, "the shrinking town can never take a seat")?;
        let c_after = counters(&kit, stage, &j_after)?;
        morph_all(
            stage,
            &c_before
                .iter()
                .copied()
                .zip(c_after.iter().copied())
                .collect::<Vec<_>>(),
            0.4,
        )?;
        flash_all(stage, &c_before, 0.9)?;
        let same = kit.text(
            stage,
            &format!("before and after: {}", triple(&j_after)),
            22.0,
            Palette::green(),
            v(-3.6, -2.75),
        )?;
        fade_in_all(stage, &[same], 0.4)?;

        // 13_6 the scorecard with three promises
        wipe(&kit, stage)?;
        let s = say(&kit, stage, "13_6")?;
        let y0 = 2.0;
        let head = score_head(&kit, stage, 3, y0 + 0.9)?;
        let ham = score_row(
            &kit,
            stage,
            "Hamilton",
            &[(Some(true), ""), (Some(false), ""), (Some(false), "")],
            y0,
        )?;
        let mut shown_rows = head;
        shown_rows.extend(&ham);
        fade_in_all(stage, &shown_rows, 0.6)?;
        s.wait_for(stage, "a strong reason to prefer divisor rules")?;
        let div = score_row(
            &kit,
            stage,
            "divisor rules",
            &[(Some(false), ""), (Some(true), ""), (Some(true), "")],
            y0 - SCORE_STEP,
        )?;
        fade_in_lagged(stage, &div, 0.9, 0.2)?;
        s.wait_for(stage, "But they cannot always keep the quota promise")?;
        flash_all(stage, &div[1..3], 0.9)?;
        s.wait_for(stage, "Could some cleverer rule")?;
        let ask_y = y0 - 2.4 * SCORE_STEP;
        let ask = vec![
            kit.text_left(
                stage,
                "a cleverer rule?",
                26.0,
                Palette::hero(),
                v(SCORE_LABEL_X, ask_y),
            )?,
            kit.text(stage, "?", 34.0, Palette::hero(), v(SCORE_X[0], ask_y))?,
            kit.text(stage, "?", 34.0, Palette::hero(), v(SCORE_X[2], ask_y))?,
        ];
        fade_in_lagged(stage, &ask, 0.9, 0.3)?;
        pause(stage, 0.6)?;
        kit.finish(stage)
    }
}
