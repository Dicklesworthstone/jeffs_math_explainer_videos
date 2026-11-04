//! 11 Webster can break the quota promise, but not with three towns: our
//! towns stay inside their boxes; four towns (2,000, 5,000, 8,000 and
//! 27,000 people, fourteen seats) round to fifteen seats; a live yardstick
//! grows until the big town's quotient drops below 8½ first, and the town
//! whose share is exactly nine gets eight. The window and every count come
//! from `model`.

use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{
    NAME_SIZE, NAME_X, READ_SIZE, READ_X, ROW_Y, Rows, SCORE_LABEL_X, SCORE_STEP, SCORE_X, TOKEN, band, check,
    fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, grid, line_style, live_band, pause, quota_box,
    run_tracker, say, score_head, score_row, seat_x, share_readouts, stroke, think_here, token_spot, tokens,
    town_color, wipe,
};
use crate::model::{self, HOUSE4, POP, POP4, Rule, decimal, grouped, share, within_quota};

pub struct WebsterQuota {
    pub kit: Kit,
}

/// The four-town rows.
const ROW4_Y: [f64; 4] = [2.1, 0.7, -0.7, -2.1];

fn color4(i: usize) -> Srgb {
    // the three small towns in colors the main example does not use; the
    // big town in the big town's green
    [Palette::teal(), Palette::purple(), Palette::gold(), Palette::green()][i]
}

/// Webster's count for quotient `q`: the nearest whole number, halves up.
fn nearest(q: f64) -> f64 {
    (q + 0.5 + 1e-9).floor()
}

impl SceneConstruct for WebsterQuota {
    fn name(&self) -> &str {
        "11_webster_quota"
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
        let webster15 = model::divisor(Rule::Webster, &POP, 15).map_err(fail)?;
        let total4 = model::total(&POP4);
        let standard = total4 as f64 / HOUSE4 as f64;
        let rounded: Vec<u64> = (0..4)
            .map(|i| nearest(share(&POP4, i, HOUSE4).to_f64()) as u64)
            .collect();
        let webster4 = model::divisor(Rule::Webster, &POP4, HOUSE4).map_err(fail)?;
        let big = 3;
        let big_q = share(&POP4, big, HOUSE4);
        if rounded.iter().sum::<u64>() != HOUSE4 + 1
            || !big_q.is_whole()
            || u128::from(webster4[big]) >= big_q.floor()
            || within_quota(&webster4, &POP4, HOUSE4)
        {
            return Err(fail(
                "11_3/11_4: the four-town map must round to one seat too many and leave the big town below nine".into(),
            ));
        }
        let first_out = model::signposts(Rule::Webster, &POP4, HOUSE4 as usize + 1)[HOUSE4 as usize];
        if first_out.0 != big {
            return Err(fail("11_4 says the big town's halfway mark comes first".into()));
        }
        let (low, high) = model::window(Rule::Webster, &POP4, HOUSE4).map_err(fail)?;
        let target = (low.to_f64() + high.to_f64()) / 2.0;
        let at_target: Vec<u64> = POP4.iter().map(|&p| nearest(p as f64 / target) as u64).collect();
        if at_target != webster4 {
            return Err(fail(
                "11_4: the shown yardstick must give Webster's fourteen seats".into(),
            ));
        }

        // 11_1 Webster at fifteen seats, every town inside its box
        say(&kit, stage, "11_1")?;
        let rows = Rows::build(&kit, stage, &POP, 15, 12)?;
        let reads = share_readouts(&kit, stage, &POP, 15)?;
        let mut first = rows.all();
        first.extend(&reads);
        let mut boxes = Vec::new();
        for i in 0..3 {
            let q = share(&POP, i, 15);
            first.extend(tokens(stage, ROW_Y[i], 0, webster15[i] as usize, town_color(i))?);
            boxes.push(quota_box(stage, ROW_Y[i], q.floor(), q.ceil(), Palette::green())?);
        }
        let head = kit.text(stage, "Webster, 15 seats", 30.0, Palette::ink(), v(-1.4, 3.3))?;
        first.push(head);
        fade_in_all(stage, &first, 0.6)?;
        fade_in_lagged(stage, &boxes, 0.9, 0.3)?;
        let q = think_here(&kit, stage)?;

        // 11_2 not with three towns; with four it can
        let s = say(&kit, stage, "11_2")?;
        fade_out_all(stage, &[q], 0.2)?;
        s.wait_for(stage, "Not with three towns")?;
        let tick = check(stage, v(4.6, 0.3), 0.8, Palette::green());
        fade_in_all(stage, &[tick], 0.4)?;
        s.wait_for(stage, "Balinski and Young proved")?;
        let proved = kit.text(
            stage,
            "3 towns: Webster always keeps the quota promise",
            20.0,
            Palette::green(),
            v(3.9, -0.6),
        )?;
        let cited = kit.text(
            stage,
            "Balinski and Young, as cited by Gölz, Peters and Procaccia",
            15.0,
            Palette::dim(),
            v(3.9, -1.0),
        )?;
        let scan = kit.text(
            stage,
            &format!(
                "our towns, 1 to 100 seats: {} quota breaks",
                fact("webster.three_town_quota_breaks_1_100")?
            ),
            15.0,
            Palette::dim(),
            v(3.9, -1.35),
        )?;
        fade_in_lagged(stage, &[proved, cited, scan], 1.0, 0.3)?;
        s.wait_for(stage, "With four towns, it can")?;
        wipe(&kit, stage)?;

        // 11_3 four towns, fourteen seats: rounding hands out fifteen
        let s = say(&kit, stage, "11_3")?;
        let d = stage.add_value_tracker(standard);
        let mut base = Vec::new();
        let mut bands = Vec::new();
        let mut static_reads = Vec::new();
        for i in 0..4 {
            let y = ROW4_Y[i];
            base.push(kit.text_left(
                stage,
                &grouped(u128::from(POP4[i])),
                NAME_SIZE,
                color4(i),
                v(NAME_X, y + 0.14),
            )?);
            base.push(kit.text_left(stage, "people", 18.0, Palette::dim(), v(NAME_X, y - 0.24))?);
            base.extend(grid(stage, y, 10)?);
            let qi = share(&POP4, i, HOUSE4);
            bands.push(band(stage, y, qi.to_f64(), color4(i))?);
            static_reads.push(kit.text_left(stage, &decimal(qi, 2), READ_SIZE, color4(i), v(READ_X, y))?);
        }
        let head4 = kit.text(stage, &format!("{HOUSE4} seats"), 30.0, Palette::ink(), v(-1.4, 3.3))?;
        base.push(head4);
        s.wait_for(stage, "Take four towns")?;
        fade_in_lagged(stage, &base, 1.6, 0.05)?;
        let all_bands: Vec<Mob> = bands.iter().flatten().copied().collect();
        fade_in_all(stage, &all_bands, 0.5)?;
        s.wait_for(stage, "The biggest town's share is exactly nine")?;
        fade_in_all(stage, &[static_reads[big]], 0.4)?;
        flash_all(stage, &[static_reads[big]], 0.8)?;
        s.wait_for(stage, "The three small shares")?;
        fade_in_lagged(stage, &static_reads[..3], 1.2, 0.3)?;
        s.wait_for(stage, "each one rounds up")?;
        let mut toks: Vec<Vec<Mob>> = Vec::new();
        for i in 0..4 {
            toks.push(tokens(stage, ROW4_Y[i], 0, rounded[i] as usize, color4(i))?);
        }
        let small: Vec<Mob> = toks[..3].iter().flatten().copied().collect();
        fade_in_lagged(stage, &small, 1.0, 0.3)?;
        // the seat each small town gets by rounding up
        let ups: Vec<Mob> = (0..3).map(|i| toks[i][rounded[i] as usize - 1]).collect();
        flash_all(stage, &ups, 0.9)?;
        s.wait_for(stage, "Rounding everything to the nearest")?;
        fade_in_all(stage, &toks[big], 0.6)?;
        let sum = kit.text(
            stage,
            &format!(
                "{} = {}, one too many",
                rounded.iter().map(u64::to_string).collect::<Vec<_>>().join(" + "),
                HOUSE4 + 1
            ),
            26.0,
            Palette::red(),
            v(3.9, 3.3),
        )?;
        fade_in_all(stage, &[sum], 0.5)?;

        // 11_4 grow the yardstick: the big town's halfway mark comes first
        let s = say(&kit, stage, "11_4")?;
        // the static bands and readouts hand over to live ones
        let mut live = Vec::new();
        for i in 0..4 {
            let p = POP4[i] as f64;
            live.extend(live_band(stage, ROW4_Y[i], color4(i), d, move |x| p / x)?);
            live.push(kit.live_value(
                stage,
                move |s| p / s.tracker_value(d).unwrap_or(1.0),
                2,
                false,
                READ_SIZE,
                color4(i),
                v(READ_X, ROW4_Y[i]),
                LEFT,
            )?);
        }
        let mut handed = all_bands.clone();
        handed.extend(&static_reads);
        fade_out_all(stage, &handed, 0.2)?;
        let stick = kit.live_count(stage, d, 28.0, Palette::hero(), v(-5.2, 3.3))?;
        let stick_t = kit.text_left(stage, "people per seat", 22.0, Palette::hero(), v(-4.45, 3.3))?;
        fade_out_all(stage, &[head4], 0.3)?;
        fade_in_all(stage, &[stick_t], 0.3)?;
        s.wait_for(stage, "until some town's quotient drops below its halfway mark")?;
        let marks: Vec<Mob> = (0..4)
            .map(|i| {
                let x = seat_x(rounded[i] as f64 - 0.5);
                let y = ROW4_Y[i] - 0.3;
                stroke(
                    stage,
                    vec![v(x, y - 0.17), v(x, y + 0.17)],
                    line_style(Palette::hero(), 3.5, 1.0),
                )
            })
            .collect();
        fade_in_lagged(stage, &marks, 1.0, 0.2)?;
        s.wait_for(stage, "The big town's halfway mark comes first")?;
        run_tracker(stage, d, target, 4.0)?;
        flash_all(stage, &[marks[big]], 0.9)?;
        s.wait_for(stage, "Webster gives one, two, three and eight")?;
        let gone = *toks[big].last().ok_or_else(|| fail("no big-town tokens".into()))?;
        fade_out_all(stage, &[gone], 0.5)?;
        let result = kit.text(
            stage,
            &format!(
                "Webster: {}",
                webster4.iter().map(u64::to_string).collect::<Vec<_>>().join(", ")
            ),
            26.0,
            Palette::ink(),
            v(3.9, 3.3),
        )?;
        fade_out_all(stage, &[sum], 0.2)?;
        fade_in_all(stage, &[result], 0.4)?;
        s.wait_for(stage, "The town whose share is exactly nine gets eight")?;
        let bigbox = quota_box(stage, ROW4_Y[big], big_q.floor(), big_q.ceil(), Palette::red())?;
        // the empty ninth cell, outlined in red
        let empty = Rectangle::new()
            .width(TOKEN)
            .height(TOKEN)
            .corner_radius(0.06)
            .style(line_style(Palette::red(), 3.0, 1.0))
            .build()?
            .moved_to(token_spot(ROW4_Y[big], webster4[big] as usize));
        let empty = stage.arena_mut().add(empty);
        fade_in_all(stage, &[bigbox, empty], 0.4)?;
        flash_all(stage, &[bigbox, empty], 1.0)?;
        let _ = (live, stick);

        // 11_5 the scorecard so far, and the open question
        wipe(&kit, stage)?;
        let s = say(&kit, stage, "11_5")?;
        let y0 = 2.0;
        let head = score_head(&kit, stage, 2, y0 + 0.9)?;
        let card = [
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
        for row in &card {
            shown.extend(row);
        }
        fade_in_lagged(stage, &shown, 1.4, 0.15)?;
        s.wait_for(stage, "Is there any rule")?;
        let ask_y = y0 - 3.4 * SCORE_STEP;
        let ask = vec![
            kit.text_left(
                stage,
                "a rule with both?",
                26.0,
                Palette::hero(),
                v(SCORE_LABEL_X, ask_y),
            )?,
            kit.text(stage, "?", 34.0, Palette::hero(), v(SCORE_X[0], ask_y))?,
            kit.text(stage, "?", 34.0, Palette::hero(), v(SCORE_X[1], ask_y))?,
        ];
        fade_in_lagged(stage, &ask, 0.9, 0.3)?;
        pause(stage, 0.6)?;
        kit.finish(stage)
    }
}
