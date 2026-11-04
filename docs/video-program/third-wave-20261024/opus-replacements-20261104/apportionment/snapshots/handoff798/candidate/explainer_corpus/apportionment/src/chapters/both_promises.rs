//! 12 Both promises at once: Balinski and Young's quota rule in its simple
//! form. Jefferson's list is read from the top, one seat at a time, but a
//! town whose next seat would lift it above its share rounded up is skipped
//! for now. At fifteen seats Cedar's eleventh signpost is skipped and Birch's
//! fourth is taken (1, 4, 10); at sixteen Cedar's ceiling rises and it gets
//! the seat (1, 4, 11). Every count from `model::quota_rule`.

use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{
    CARD_H, CARD_W, LIST_X, NAME_SIZE, NAME_X, ROW_H, ROW_Y, SCORE_LABEL_X, SCORE_STEP, SCORE_X, card_y, check, cutoff,
    cutoff_y, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, flat, grid, line_style, morph_all, move_all,
    pause, quota_box, refill, say, score_head, score_row, seat_x, signpost_cards, stroke, think_here, token_at,
    town_color, wipe,
};
use crate::model::{self, ASH, BIRCH, CEDAR, NAMES, POP, Rule, share};

pub struct BothPromises {
    pub kit: Kit,
}

/// How many Jefferson signposts the list shows (down to Ash's second).
const SHOWN: usize = 18;
/// The left panel: names, seat counters and each town's ceiling.
const COUNT_X: f64 = -2.2;
const CEIL_X: f64 = -0.4;

fn counters(kit: &Kit, stage: &mut Stage<'_>, c: &[u64]) -> fmn::Result<Vec<Mob>> {
    (0..3)
        .map(|i| kit.text(stage, &c[i].to_string(), 44.0, town_color(i), v(COUNT_X, ROW_Y[i])))
        .collect()
}

/// "at most 4": each town's share rounded up at `house` seats.
fn ceilings(kit: &Kit, stage: &mut Stage<'_>, house: u64, hot: Option<usize>) -> fmn::Result<Vec<Mob>> {
    (0..3)
        .map(|i| {
            let c = share(&POP, i, house).ceil();
            let color = if hot == Some(i) { Palette::red() } else { Palette::dim() };
            kit.text(stage, &format!("at most {c}"), 20.0, color, v(CEIL_X, ROW_Y[i]))
        })
        .collect()
}

/// A soft glow behind card `rank`, detached.
fn glow(stage: &mut Stage<'_>, rank: usize, color: Srgb) -> fmn::Result<Mob> {
    let g = Rectangle::new()
        .width(CARD_W + 0.14)
        .height(CARD_H + 0.1)
        .corner_radius(0.06)
        .style(flat(color, 0.3))
        .build()?
        .moved_to(v(LIST_X, card_y(rank)));
    Ok(stage.arena_mut().add(g))
}

fn triple(s: &[u64]) -> String {
    s.iter().map(u64::to_string).collect::<Vec<_>>().join(", ")
}

impl SceneConstruct for BothPromises {
    fn name(&self) -> &str {
        "12_both_promises"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let facts = crate::facts::facts().map_err(fail)?;
        let fact = |key: &str| {
            facts
                .get(key)
                .map(str::to_owned)
                .ok_or_else(|| fail(format!("no fact {key}")))
        };
        let list = model::signposts(Rule::Jefferson, &POP, SHOWN);
        let history = model::quota_rule(&POP, 16).map_err(fail)?;
        let jefferson15 = model::divisor(Rule::Jefferson, &POP, 15).map_err(fail)?;
        let rank_of = |town: usize, seat: u64| list.iter().position(|e| e.0 == town && e.1 == seat).map(|r| r + 1);
        let skipped = rank_of(CEDAR, 11).ok_or_else(|| fail("no card for Cedar's eleventh seat".into()))?;
        let taken = rank_of(BIRCH, 4).ok_or_else(|| fail("no card for Birch's fourth seat".into()))?;
        let ash2 = rank_of(ASH, 2).ok_or_else(|| fail("no card for Ash's second seat".into()))?;
        // the story the chapter tells, checked against the model
        if history[14] != model::divisor(Rule::Jefferson, &POP, 14).map_err(fail)?
            || skipped != 15
            || taken != 16
            || history[15] != [history[14][0], history[14][1] + 1, history[14][2]]
            || history[16] != [history[15][0], history[15][1], history[15][2] + 1]
            || share(&POP, CEDAR, 15).ceil() != u128::from(history[14][CEDAR])
        {
            return Err(fail("12: the quota rule's walk differs from the one narrated".into()));
        }

        // 12_1 Jefferson's fifteenth seat lifts Cedar above its ceiling
        let s = say(&kit, stage, "12_1")?;
        let cards = signpost_cards(&kit, stage, &POP, SHOWN)?;
        let mut names = Vec::new();
        for i in 0..3 {
            names.push(kit.text_left(stage, NAMES[i], 30.0, town_color(i), v(-5.8, ROW_Y[i]))?);
        }
        let head = kit.text(stage, "Jefferson's list", 20.0, Palette::dim(), v(LIST_X, 3.78))?;
        let line = cutoff(stage, 15);
        let c_j = counters(&kit, stage, &jefferson15)?;
        let mut first: Vec<Mob> = cards.iter().flatten().copied().collect();
        first.extend(&names);
        first.push(head);
        fade_in_lagged(stage, &first, 1.4, 0.05)?;
        fade_in_all(stage, &[vec![line], c_j.clone()].concat(), 0.5)?;
        s.wait_for(stage, "by giving Cedar its eleventh seat")?;
        let hot = glow(stage, skipped, Palette::red())?;
        fade_in_all(stage, &[hot], 0.4)?;
        flash_all(stage, &[c_j[CEDAR]], 0.9)?;
        s.wait_for(stage, "Suppose a rule refuses")?;
        let ceil15 = ceilings(&kit, stage, 15, Some(CEDAR))?;
        let ceil_head = kit.text(stage, "15 seats", 18.0, Palette::dim(), v(CEIL_X, 2.35))?;
        fade_in_lagged(stage, &[vec![ceil_head], ceil15.clone()].concat(), 1.0, 0.25)?;
        let q = think_here(&kit, stage)?;

        // 12_2 Birch; the rule in its simple form
        let s = say(&kit, stage, "12_2")?;
        fade_out_all(stage, &[q], 0.2)?;
        let take = glow(stage, taken, Palette::hero())?;
        fade_in_all(stage, &[take], 0.4)?;
        flash_all(stage, &cards[taken - 1][1..], 0.9)?;
        s.wait_for(stage, "Michel Balinski and Peyton Young")?;
        let credit = kit.text(
            stage,
            &format!(
                "the quota rule: Balinski and Young, {}",
                fact("history.quota_method_year")?
            ),
            22.0,
            Palette::ink(),
            v(-3.4, 3.3),
        )?;
        fade_in_all(stage, &[credit], 0.5)?;
        s.wait_for(stage, "Hand out seats one at a time")?;
        fade_out_all(stage, &[hot, take], 0.3)?;
        let zero = counters(&kit, stage, &[0, 0, 0])?;
        morph_all(
            stage,
            &c_j.iter().copied().zip(zero.iter().copied()).collect::<Vec<_>>(),
            0.4,
        )?;
        move_all(stage, &[(line, v(LIST_X, cutoff_y(0)))], 0.5)?;
        // down the list, seat by seat, to fourteen
        let mut seats = [0_u64; 3];
        for rank in 1..=14 {
            seats[list[rank - 1].0] += 1;
            let next = counters(&kit, stage, &seats)?;
            move_all(stage, &[(line, v(LIST_X, cutoff_y(rank)))], 0.16)?;
            morph_all(
                stage,
                &c_j.iter().copied().zip(next.iter().copied()).collect::<Vec<_>>(),
                0.08,
            )?;
        }
        if seats[..] != history[14][..] {
            return Err(fail(
                "12_2: the walk to fourteen seats differs from the quota rule".into(),
            ));
        }
        s.wait_for(stage, "check that the seat would not lift it")?;
        flash_all(stage, &ceil15, 1.0)?;
        s.wait_for(stage, "skip that signpost for now")?;
        let strike = stroke(
            stage,
            vec![
                v(LIST_X - CARD_W / 2.0, card_y(skipped)),
                v(LIST_X + CARD_W / 2.0, card_y(skipped)),
            ],
            line_style(Palette::red(), 4.0, 1.0),
        );
        fade_in_all(stage, &[strike], 0.4)?;

        // 12_3 fifteen seats: Cedar skipped, Birch's fourth; sixteen: Cedar
        let s = say(&kit, stage, "12_3")?;
        s.wait_for(stage, "Cedar already has ten")?;
        flash_all(stage, &[c_j[CEDAR], ceil15[CEDAR], strike], 1.0)?;
        s.wait_for(stage, "The next signpost is Birch's fourth")?;
        let take = glow(stage, taken, Palette::hero())?;
        fade_in_all(stage, &[take], 0.4)?;
        let c15 = counters(&kit, stage, &history[15])?;
        morph_all(
            stage,
            &c_j.iter().copied().zip(c15.iter().copied()).collect::<Vec<_>>(),
            0.5,
        )?;
        s.wait_for(stage, "which beats Ash's second")?;
        let ash_glow = glow(stage, ash2, Palette::dim())?;
        fade_in_all(stage, &[ash_glow], 0.3)?;
        flash_all(stage, &cards[ash2 - 1][1..], 0.8)?;
        s.wait_for(stage, "The result is one, four, ten")?;
        let r15 = kit.text(
            stage,
            &format!("15 seats: {}", triple(&history[15])),
            26.0,
            Palette::hero(),
            v(-3.6, -2.75),
        )?;
        fade_in_all(stage, &[r15], 0.4)?;
        s.wait_for(stage, "At sixteen seats Cedar's ceiling rises")?;
        let ceil16 = ceilings(&kit, stage, 16, None)?;
        let ceil_head16 = kit.text(stage, "16 seats", 18.0, Palette::dim(), v(CEIL_X, 2.35))?;
        let mut pairs: Vec<(Mob, Mob)> = ceil15.iter().copied().zip(ceil16.iter().copied()).collect();
        pairs.push((ceil_head, ceil_head16));
        morph_all(stage, &pairs, 0.6)?;
        s.wait_for(stage, "and it gets its seat")?;
        fade_out_all(stage, &[strike, ash_glow], 0.3)?;
        let take15 = glow(stage, skipped, Palette::hero())?;
        fade_in_all(stage, &[take15], 0.3)?;
        move_all(stage, &[(line, v(LIST_X, cutoff_y(16)))], 0.5)?;
        let c16 = counters(&kit, stage, &history[16])?;
        morph_all(
            stage,
            &c_j.iter().copied().zip(c16.iter().copied()).collect::<Vec<_>>(),
            0.5,
        )?;
        let r16 = kit.text(
            stage,
            &format!("16 seats: {}", triple(&history[16])),
            26.0,
            Palette::hero(),
            v(-3.6, -3.3),
        )?;
        fade_in_all(stage, &[r16], 0.4)?;

        // 12_4 both promises; what the film checked and what is cited
        wipe(&kit, stage)?;
        let s = say(&kit, stage, "12_4")?;
        let y0 = 2.0;
        let head = score_head(&kit, stage, 2, y0 + 0.9)?;
        let rows = [
            score_row(&kit, stage, "Hamilton", &[(Some(true), ""), (Some(false), "")], y0)?,
            score_row(
                &kit,
                stage,
                "Jefferson",
                &[(Some(false), ""), (Some(true), "")],
                y0 - SCORE_STEP,
            )?,
            score_row(
                &kit,
                stage,
                "Webster",
                &[(Some(false), "never with 3 towns"), (Some(true), "")],
                y0 - 2.0 * SCORE_STEP,
            )?,
        ];
        let mut shown = head;
        for r in &rows {
            shown.extend(r);
        }
        fade_in_all(stage, &shown, 0.6)?;
        let quota_y = y0 - 3.2 * SCORE_STEP;
        let quota_row = score_row(&kit, stage, "quota rule", &[(None, ""), (None, "")], quota_y)?;
        fade_in_all(stage, &quota_row, 0.4)?;
        s.wait_for(stage, "so nobody ever loses one")?;
        let keep = check(stage, v(SCORE_X[1], quota_y), 0.42, Palette::green());
        fade_in_all(stage, &[keep], 0.4)?;
        s.wait_for(stage, "Balinski and Young proved")?;
        let quota = check(stage, v(SCORE_X[0], quota_y), 0.42, Palette::green());
        fade_in_all(stage, &[quota], 0.4)?;
        let proved = kit.text_left(
            stage,
            "proved by Balinski and Young, as cited by Gölz, Peters and Procaccia",
            16.0,
            Palette::dim(),
            v(SCORE_LABEL_X, quota_y - 0.75),
        )?;
        if fact("quotamethod.within_quota_checked")? != "yes" || fact("quotamethod.never_takes_back_checked")? != "yes"
        {
            return Err(fail("12_4: the film's own check of the quota rule failed".into()));
        }
        let ours = kit.text_left(
            stage,
            &format!(
                "checked here for 1 to {} seats: always within quota, never takes a seat back; at {} seats two signposts tie",
                fact("quotamethod.checked_through")?,
                fact("quotamethod.first_tie_house")?
            ),
            16.0,
            Palette::dim(),
            v(SCORE_LABEL_X, quota_y - 1.1),
        )?;
        fade_in_lagged(stage, &[proved, ours], 0.8, 0.4)?;
        s.wait_for(stage, "Both promises, at every house size")?;
        flash_all(stage, &[keep, quota], 0.9)?;

        // 12_5 one promise, more than one answer
        wipe(&kit, stage)?;
        let s = say(&kit, stage, "12_5")?;
        let hamilton15 = model::hamilton(&POP, 15).map_err(fail)?;
        let mut base = Vec::new();
        let mut boxes = Vec::new();
        for i in 0..3 {
            let y = ROW_Y[i];
            base.push(kit.text_left(stage, NAMES[i], NAME_SIZE, town_color(i), v(NAME_X, y))?);
            base.extend(grid(stage, y, 11)?);
            let q = share(&POP, i, 15);
            boxes.push(quota_box(stage, y, q.floor(), q.ceil(), Palette::dim())?);
        }
        let legend = [
            kit.text_left(stage, "upper: quota rule", 22.0, Palette::ink(), v(1.2, 3.3))?,
            kit.text_left(stage, "lower: Hamilton", 22.0, Palette::dim(), v(1.2, 2.85))?,
        ];
        fade_in_all(stage, &[base.clone(), boxes.clone()].concat(), 0.6)?;
        let dy = ROW_H / 4.0;
        let mut upper = Vec::new();
        let mut lower = Vec::new();
        for i in 0..3 {
            for k in 0..history[15][i] {
                upper.push(token_at(
                    stage,
                    v(seat_x(k as f64 + 0.5), ROW_Y[i] + dy),
                    town_color(i),
                )?);
            }
            for k in 0..hamilton15[i] {
                lower.push(token_at(
                    stage,
                    v(seat_x(k as f64 + 0.5), ROW_Y[i] - dy),
                    Palette::dim(),
                )?);
            }
        }
        s.wait_for(stage, "the quota rule gives one, four, ten")?;
        fade_in_all(stage, &[legend[0]], 0.3)?;
        fade_in_lagged(stage, &upper, 1.0, 0.1)?;
        s.wait_for(stage, "while Hamilton gives two, three, ten")?;
        fade_in_all(stage, &[legend[1]], 0.3)?;
        fade_in_lagged(stage, &lower, 1.0, 0.1)?;
        s.wait_for(stage, "Both keep the quota promise")?;
        let green: Vec<(Mob, Srgb, f64)> = boxes.iter().map(|&b| (b, Palette::green(), 1.0)).collect();
        refill(stage, &green, 0.5)?;
        s.wait_for(stage, "can allow more than one answer")?;
        flash_all(stage, &boxes[..2], 1.0)?;
        let allowed: Vec<String> = (0..3)
            .map(|i| {
                let q = share(&POP, i, 15);
                if q.is_whole() {
                    format!("{} {}", NAMES[i], q.floor())
                } else {
                    format!("{} {} or {}", NAMES[i], q.floor(), q.ceil())
                }
            })
            .collect();
        let both = kit.text(
            stage,
            &format!("{}: both answers fit", allowed.join(", ")),
            22.0,
            Palette::green(),
            v(0.0, -2.8),
        )?;
        fade_in_all(stage, &[both], 0.5)?;
        pause(stage, 0.6)?;
        kit.finish(stage)
    }
}
