//! 15 The whole map: the scorecard of every rule against the three
//! promises; a try-it-yourself (under Webster, can a growing Cedar ever
//! raise Ash?) answered by moving the real signpost list for a Cedar of
//! 30,000 (`model::POP_PUZZLE`): Cedar's cards travel through the lane with
//! ghosts where they leave and land while Ash's and Birch's keep their
//! values and are only pushed down; and the dials once more, each hand on
//! its own tracker, 15 to 16 seats and round again.

use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::dials::{house_trackers, ranking, three_dials};
use super::draw::{
    CARD_H, CARD_W, CardKey, LIST_X, ListCard, ROW_Y, SCORE_STEP, card_y, cutoff, fade_in_all, fade_in_lagged,
    fade_out_all, fail, flash_all, flat, keyed_cards, morph_all, pause, rank_column, run_trackers, say, score_head,
    score_row, slide_cards, think, town_color, wipe,
};
use crate::model::{self, ASH, BIRCH, CEDAR, NAMES, POP, POP_PUZZLE, Rule, grouped, people};

pub struct Recap {
    pub kit: Kit,
}

/// The try-it-yourself list: Webster signposts, cutoff at fifteen.
const SHOWN: usize = 17;
const HOUSE: u64 = 15;
/// 15_3 and 15_4: the counters and the notes stay left of the lane a moving
/// card travels along (`draw::LANE_X`).
const LIST_COUNT_X: f64 = -3.6;
const NOTE_X: f64 = -6.6;

fn counters(kit: &Kit, stage: &mut Stage<'_>, c: &[u64]) -> fmn::Result<Vec<Mob>> {
    (0..3)
        .map(|i| kit.text(stage, &c[i].to_string(), 44.0, town_color(i), v(LIST_COUNT_X, ROW_Y[i])))
        .collect()
}

/// "cutoff for 15 seats: 2,667", the value of the last signpost above the
/// cutoff, under the list (clear of the lane and of cards leaving below it).
fn cutoff_label(kit: &Kit, stage: &mut Stage<'_>, pops: &[u64]) -> fmn::Result<Mob> {
    let list = model::signposts(Rule::Webster, pops, HOUSE as usize);
    let value = list[HOUSE as usize - 1].2;
    kit.text(
        stage,
        &format!("cutoff for {HOUSE} seats: {}", people(value)),
        20.0,
        Palette::hero(),
        v(LIST_X, card_y(SHOWN + 2) - 0.32),
    )
}

/// A soft glow behind the cards at `ranks`, detached.
fn glows(stage: &mut Stage<'_>, ranks: &[usize]) -> fmn::Result<Vec<Mob>> {
    ranks
        .iter()
        .map(|&r| {
            let g = Rectangle::new()
                .width(CARD_W + 0.14)
                .height(CARD_H + 0.1)
                .corner_radius(0.06)
                .style(flat(Palette::hero(), 0.28))
                .build()?
                .moved_to(v(LIST_X, card_y(r)));
            Ok(stage.arena_mut().add(g))
        })
        .collect()
}

impl SceneConstruct for Recap {
    fn name(&self) -> &str {
        "15_recap"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let before = model::divisor(Rule::Webster, &POP, HOUSE).map_err(fail)?;
        let after = model::divisor(Rule::Webster, &POP_PUZZLE, HOUSE).map_err(fail)?;
        if POP_PUZZLE[ASH] != POP[ASH] || POP_PUZZLE[BIRCH] != POP[BIRCH] || POP_PUZZLE[CEDAR] <= POP[CEDAR] {
            return Err(fail("15_3: only Cedar grows in the try-it-yourself".into()));
        }
        if after[ASH] > before[ASH] || after[BIRCH] > before[BIRCH] {
            return Err(fail("15_4 says a growing Cedar never raises Ash or Birch".into()));
        }

        // 15_1 the whole map: Hamilton
        let s = say(&kit, stage, "15_1")?;
        let y0 = 2.0;
        let head = score_head(&kit, stage, 3, y0 + 0.9)?;
        fade_in_all(stage, &head, 0.6)?;
        s.wait_for(stage, "Hamilton's rule keeps the quota promise")?;
        let ham = score_row(
            &kit,
            stage,
            "Hamilton",
            &[(Some(true), ""), (Some(false), ""), (Some(false), "")],
            y0,
        )?;
        fade_in_all(stage, &ham[..2], 0.4)?;
        s.wait_for(stage, "it can take a seat back")?;
        fade_in_all(stage, &ham[2..4], 0.4)?;
        s.wait_for(stage, "it can punish a town for growing")?;
        fade_in_all(stage, &ham[4..], 0.4)?;

        // 15_2 the divisor rules, and the quota method
        let s = say(&kit, stage, "15_2")?;
        let mut divisor_rows = Vec::new();
        for (j, (rule, note)) in [
            ("Jefferson", ""),
            ("Adams", ""),
            ("Webster", "never with 3 towns"),
            ("Hill", ""),
        ]
        .iter()
        .enumerate()
        {
            let y = y0 - SCORE_STEP * (j as f64 + 1.0);
            divisor_rows.push(score_row(
                &kit,
                stage,
                rule,
                &[(Some(false), *note), (Some(true), ""), (Some(true), "")],
                y,
            )?);
        }
        let all_div: Vec<Mob> = divisor_rows.iter().flatten().copied().collect();
        fade_in_lagged(stage, &all_div, 1.6, 0.05)?;
        s.wait_for(stage, "but each can break the quota promise")?;
        let crosses: Vec<Mob> = divisor_rows.iter().flat_map(|r| r[1..3].to_vec()).collect();
        flash_all(stage, &crosses, 1.0)?;
        s.wait_for(stage, "The quota method of Balinski and Young")?;
        let qy = y0 - SCORE_STEP * 5.0;
        let quota = score_row(
            &kit,
            stage,
            "quota method",
            &[(Some(true), ""), (Some(true), ""), (Some(false), "")],
            qy,
        )?;
        fade_in_all(stage, &quota[..3], 0.5)?;
        s.wait_for(stage, "the theorem says it must break")?;
        fade_in_all(stage, &quota[3..], 0.4)?;
        let some = kit.text(
            stage,
            &format!("for some map of {} or more towns", model::GPP_A.len()),
            20.0,
            Palette::dim(),
            v(5.4, qy - 0.45),
        )?;
        fade_in_all(stage, &[some], 0.4)?;

        // 15_3 try it yourself: under Webster, Cedar grows
        wipe(&kit, stage)?;
        let s = say(&kit, stage, "15_3")?;
        let shown = keyed_cards(&kit, stage, Rule::Webster, &POP, SHOWN)?;
        let ranks = rank_column(&kit, stage, SHOWN)?;
        let mut left = Vec::new();
        for (i, town) in NAMES.iter().enumerate() {
            left.push(kit.text_left(stage, town, 30.0, town_color(i), v(NOTE_X, ROW_Y[i]))?);
        }
        let list_head = kit.text(stage, "Webster's list, 15 seats", 20.0, Palette::dim(), v(LIST_X, 3.78))?;
        let c_before = counters(&kit, stage, &before)?;
        let line = cutoff(stage, HOUSE as usize);
        let cut_t = cutoff_label(&kit, stage, &POP)?;
        s.wait_for(stage, "Under Webster's rule")?;
        let mut first: Vec<Mob> = shown.iter().flat_map(|c| c.mobs.iter().copied()).collect();
        first.extend(&ranks);
        first.extend(&left);
        first.push(list_head);
        fade_in_lagged(stage, &first, 1.4, 0.05)?;
        fade_in_all(stage, &[vec![line, cut_t], c_before.clone()].concat(), 0.4)?;
        s.wait_for(stage, "suppose Cedar grows")?;
        let cedar_ranks: Vec<usize> = shown
            .iter()
            .enumerate()
            .filter(|(_, c)| c.key.0 == CEDAR)
            .map(|(r, _)| r + 1)
            .collect();
        let glow = glows(stage, &cedar_ranks)?;
        fade_in_all(stage, &glow, 0.5)?;
        let q = think(&kit, stage, v(-1.0, -2.7))?;

        // 15_4 no: Cedar's signposts rise, Ash's and Birch's keep their
        // values and can only be pushed down, the cutoff rises
        let s = say(&kit, stage, "15_4")?;
        fade_out_all(stage, &[q], 0.2)?;
        let example = kit.text_left(
            stage,
            &format!("for example, Cedar {} people", grouped(u128::from(POP_PUZZLE[CEDAR]))),
            20.0,
            Palette::dim(),
            v(NOTE_X, 3.3),
        )?;
        fade_out_all(stage, &glow, 0.3)?;
        fade_in_all(stage, &[example], 0.4)?;
        let next = keyed_cards(&kit, stage, Rule::Webster, &POP_PUZZLE, SHOWN)?;
        // the line's claims: no card of Ash's or Birch's moves up
        let old_keys: Vec<CardKey> = shown.iter().map(|c| c.key).collect();
        let rank_in = |list: &[ListCard], key: CardKey| list.iter().position(|c| c.key == key);
        for (a, &key) in old_keys.iter().enumerate() {
            if key.0 != CEDAR && rank_in(&next, key).is_some_and(|b| b < a) {
                return Err(fail(format!("15_4 says {key:?} cannot move up the list")));
            }
        }
        s.wait_for(stage, "Cedar's signposts all get bigger")?;
        let shown = slide_cards(&kit, stage, shown, next, CEDAR, 1.6)?;
        s.wait_for(stage, "keep exactly the same values")?;
        let same: Vec<Mob> = shown.iter().filter(|c| c.key.0 != CEDAR).map(|c| c.mobs[2]).collect();
        flash_all(stage, &same, 0.9)?;
        s.wait_for(stage, "those are pushed down the list")?;
        let pushed: Vec<Mob> = shown
            .iter()
            .enumerate()
            .filter(|(b, c)| c.key.0 != CEDAR && old_keys.iter().position(|&k| k == c.key) != Some(*b))
            .map(|(_, c)| c.mobs[1])
            .collect();
        flash_all(stage, &pushed, 0.9)?;
        s.wait_for(stage, "So the cutoff for fifteen seats can only rise")?;
        let cut_new = cutoff_label(&kit, stage, &POP_PUZZLE)?;
        morph_all(stage, &[(cut_t, cut_new)], 0.6)?;
        flash_all(stage, &[cut_t, line], 0.8)?;
        s.wait_for(stage, "any signpost of Ash's that was below it")?;
        // no Ash card on the shown list was below the old cutoff; Ash's next
        // signpost, further down, keeps its value and stays below the
        // higher cutoff
        let ash_next = model::signpost(Rule::Webster, POP[ASH], before[ASH] + 1);
        let cut_at = |pops: &[u64]| model::signposts(Rule::Webster, pops, HOUSE as usize)[HOUSE as usize - 1].2;
        let facts = crate::facts::facts().map_err(fail)?;
        let next_fact = facts
            .get("webster.15.ash_next_signpost")
            .ok_or_else(|| fail("no fact webster.15.ash_next_signpost".into()))?;
        if ash_next >= cut_at(&POP) || cut_at(&POP) > cut_at(&POP_PUZZLE) || people(ash_next) != next_fact {
            return Err(fail(
                "15_4 says Ash's next signpost was below the cutoff, which only rose".into(),
            ));
        }
        let next_t = [
            kit.text_left(
                stage,
                &format!("Ash's next signpost: {next_fact}"),
                20.0,
                town_color(ASH),
                v(NOTE_X, -2.4),
            )?,
            kit.text_left(
                stage,
                "below the cutoff, before and after",
                20.0,
                Palette::dim(),
                v(NOTE_X, -2.8),
            )?,
        ];
        fade_in_lagged(stage, &next_t, 0.8, 0.3)?;
        s.wait_for(stage, "Ash can keep its seats or lose some")?;
        let c_after = counters(&kit, stage, &after)?;
        morph_all(
            stage,
            &c_before
                .iter()
                .copied()
                .zip(c_after.iter().copied())
                .collect::<Vec<_>>(),
            0.5,
        )?;
        flash_all(stage, &[c_before[ASH]], 0.9)?;

        // 15_5 back where we started: the dials, 15 to 16, then round again
        wipe(&kit, stage)?;
        let s = say(&kit, stage, "15_5")?;
        let t = house_trackers(stage, 15.0);
        let dials = three_dials(&kit, stage, t, 15.0)?;
        let (rank15, _) = ranking(15);
        let rank_t = kit.text(stage, &rank15, 24.0, Palette::ink(), v(0.0, -2.0))?;
        let mut first = dials;
        first.push(rank_t);
        fade_in_all(stage, &first, 0.6)?;
        s.wait_for(stage, "Ash lost a seat when the council grew")?;
        run_trackers(stage, &t.map(|m| (m, 16.0)), 2.5, false)?;
        let (rank16, _) = ranking(16);
        let rank_t16 = kit.text(stage, &rank16, 24.0, Palette::ink(), v(0.0, -2.0))?;
        morph_all(stage, &[(rank_t, rank_t16)], 0.6)?;
        s.wait_for(stage, "tails wrap around at different speeds")?;
        // the ranking holds only at sixteen: it steps aside while the hands
        // turn on, each at its own speed, and comes back with them
        fade_out_all(stage, &[rank_t], 0.3)?;
        run_trackers(stage, &t.map(|m| (m, 20.0)), 4.0, true)?;
        run_trackers(stage, &t.map(|m| (m, 16.0)), 1.6, false)?;
        let rank_back = kit.text(stage, &rank16, 24.0, Palette::ink(), v(0.0, -2.0))?;
        fade_in_all(stage, &[rank_back], 0.4)?;
        s.wait_for(stage, "Fair fractions are easy")?;
        let easy = kit.text(stage, "Fair fractions are easy.", 30.0, Palette::ink(), v(0.0, -2.85))?;
        fade_in_all(stage, &[easy], 0.5)?;
        s.wait_for(stage, "Fair whole numbers force a choice")?;
        let choice = kit.text(
            stage,
            "Fair whole numbers force a choice.",
            30.0,
            Palette::hero(),
            v(0.0, -3.4),
        )?;
        fade_in_all(stage, &[choice], 0.5)?;
        pause(stage, 1.0)?;
        kit.finish(stage)
    }
}
