//! 04 The sixteenth seat: one house tracker grows every share from 15 to 16
//! seats; the whole parts stay 1, 3, 10; two seats are left over and the
//! tails, re-ranked, give them to Birch and Cedar. Ash's share rose and Ash
//! lost a seat. All values from `model`.

use explainer_kit::{Kit, Palette, rect, v};
use fmn::prelude::*;

use super::draw::{
    self, READ_SIZE, READ_X, ROW_Y, Rows, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, flat, line_style,
    live_band, morph_all, move_all, pause, refill, say, stroke, think_here, token_at, token_spot, tokens, town_color,
};
use crate::model::{self, ASH, POP, decimal, share};

pub struct Sixteenth {
    pub kit: Kit,
}

const CHIP_UNIT: f64 = 2.0;
const CHIP_H: f64 = 0.3;
const CHIP_X: f64 = 3.0;

impl SceneConstruct for Sixteenth {
    fn name(&self) -> &str {
        "04_sixteenth"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let total = model::total(&POP);
        let before = model::hamilton(&POP, 15).map_err(fail)?;
        let after = model::hamilton(&POP, 16).map_err(fail)?;
        let wholes: Vec<u64> = (0..3).map(|i| share(&POP, i, 16).floor() as u64).collect();
        let tails: Vec<model::Ratio> = (0..3).map(|i| share(&POP, i, 16).tail()).collect();
        let left = 16 - wholes.iter().sum::<u64>();
        if (0..3).any(|i| share(&POP, i, 15).floor() as u64 != wholes[i]) {
            return Err(fail(
                "04_2 says the whole parts did not change from 15 to 16 seats".into(),
            ));
        }

        // the 15-seat picture we left: rows, bands, 2, 3, 10
        let rows = Rows::build(&kit, stage, &POP, 15, 12)?;
        let mut base = rows.names.clone();
        base.extend(&rows.grids);
        for b in &rows.bands {
            base.extend(b);
        }
        let mut toks: Vec<Vec<Mob>> = Vec::new();
        for i in 0..3 {
            let row = tokens(stage, ROW_Y[i], 0, before[i] as usize, town_color(i))?;
            base.extend(&row);
            toks.push(row);
        }
        let header_n = kit.text(stage, "15", 34.0, Palette::ink(), v(-2.2, 3.3))?;
        let header_t = kit.text_left(stage, "seats", 34.0, Palette::ink(), v(-1.75, 3.3))?;
        base.extend([header_n, header_t]);
        fade_in_all(stage, &base, 0.6)?;

        // swap the static bands and readouts for live ones driven by the house
        let house = stage.add_value_tracker(15.0);
        let mut live = Vec::new();
        for i in 0..3 {
            let p = POP[i] as f64 / total as f64;
            live.extend(live_band(stage, ROW_Y[i], town_color(i), house, move |h| h * p)?);
            live.push(kit.live_value(
                stage,
                move |s| s.tracker_value(house).unwrap_or(0.0) * p,
                2,
                false,
                READ_SIZE,
                town_color(i),
                v(READ_X, ROW_Y[i]),
                LEFT,
            )?);
        }
        let live_n = kit.live_value(
            stage,
            move |s| s.tracker_value(house).unwrap_or(0.0),
            0,
            false,
            34.0,
            Palette::ink(),
            v(-2.2, 3.3),
            ORIGIN,
        )?;
        let static_bands: Vec<Mob> = rows.bands.iter().flatten().copied().collect();
        fade_out_all(stage, &[static_bands, vec![header_n]].concat(), 0.05)?;

        // 04_1 the sixteenth seat: every share grows
        let s = say(&kit, stage, "04_1")?;
        // leftover seats are decided afresh at each house size: Ash's lifts out
        let ash_spare = toks[ASH][before[ASH] as usize - 1];
        refill(stage, &[(ash_spare, Palette::hero(), 0.95)], 0.4)?;
        move_all(stage, &[(ash_spare, v(CHIP_X + 0.3, 2.9))], 0.6)?;
        s.wait_for(stage, "Every share grows")?;
        draw::ease_tracker(stage, house, 16.0, 2.6)?;
        let _ = live_n;

        // 04_2 the whole parts did not move: two seats left over
        let s = say(&kit, stage, "04_2")?;
        let sum = format!(
            "{} + {} + {} = {}",
            wholes[0],
            wholes[1],
            wholes[2],
            wholes.iter().sum::<u64>()
        );
        let sum_t = kit.text(stage, &sum, 30.0, Palette::ink(), v(-1.6, -2.7))?;
        fade_in_all(stage, &[sum_t], 0.5)?;
        s.wait_for(stage, "two seats are left over")?;
        let spare2 = token_at(stage, v(CHIP_X + 0.75, 2.9), Palette::hero())?;
        let spare_t = kit.text_left(
            stage,
            &format!("{left} left over"),
            24.0,
            Palette::hero(),
            v(CHIP_X + 1.15, 2.9),
        )?;
        fade_in_all(stage, &[spare2, spare_t], 0.5)?;

        // 04_3 the new tails, side by side
        let s = say(&kit, stage, "04_3")?;
        let mut chips = Vec::new();
        let mut labels = Vec::new();
        for i in 0..3 {
            let c = rect(
                v(CHIP_X, ROW_Y[i] - 0.65 - CHIP_H / 2.0),
                tails[i].to_f64() * CHIP_UNIT,
                CHIP_H,
                flat(Palette::hero(), 0.9),
            )?;
            chips.push(stage.arena_mut().add(c));
            labels.push(kit.text_left(
                stage,
                &decimal(tails[i], 2),
                22.0,
                town_color(i),
                v(CHIP_X + CHIP_UNIT + 0.2, ROW_Y[i] - 0.65),
            )?);
        }
        s.wait_for(stage, "Ash's tail is now")?;
        let mut lifted = chips.clone();
        lifted.extend(&labels);
        fade_in_lagged(stage, &lifted, 1.5, 0.25)?;
        let q = think_here(&kit, stage)?;

        // 04_4 sorted by size: Birch, Cedar, then Ash
        let s = say(&kit, stage, "04_4")?;
        fade_out_all(stage, &[q, spare_t], 0.3)?;
        let mut order: Vec<usize> = (0..3).collect();
        order.sort_by(|&a, &b| tails[b].cmp(&tails[a]));
        let mut moves = Vec::new();
        for (rank, &i) in order.iter().enumerate() {
            let y = ROW_Y[rank] - 0.65;
            let w = tails[i].to_f64() * CHIP_UNIT;
            moves.push((chips[i], v(CHIP_X + w / 2.0, y)));
            moves.push((labels[i], v(CHIP_X + CHIP_UNIT + 0.45, y)));
        }
        move_all(stage, &moves, 0.9)?;
        let winners: Vec<usize> = order.iter().take(left as usize).copied().collect();
        for (&w, spare) in winners.iter().zip([ash_spare, spare2]) {
            move_all(stage, &[(spare, token_spot(ROW_Y[w], wholes[w] as usize))], 0.6)?;
            refill(stage, &[(spare, town_color(w), 0.95)], 0.25)?;
        }
        if (0..3).any(|i| after[i] != wholes[i] + u64::from(winners.contains(&i))) {
            return Err(fail(format!(
                "04_4 choreography disagrees with Hamilton at 16 seats: {after:?}"
            )));
        }
        s.wait_for(stage, "Ash's share went up")?;
        flash_all(stage, &[rows.names[2 * ASH], chips[ASH]], 1.0)?;
        let result = kit.text(
            stage,
            &format!("{}, {}, {}", after[0], after[1], after[2]),
            34.0,
            Palette::hero(),
            v(-1.6, -2.7),
        )?;
        morph_all(stage, &[(sum_t, result)], 0.6)?;

        // 04_5 the race in 39ths of a seat
        let s = say(&kit, stage, "04_5")?;
        fade_out_all(stage, &[result], 0.3)?;
        let (x0, x1, y) = (-5.6, 5.6, -3.0);
        let step = (x1 - x0) / 39.0;
        let mut ruler = vec![stroke(
            stage,
            vec![v(x0, y), v(x1, y)],
            line_style(Palette::ink(), 2.0, 0.8),
        )];
        for k in 0..=39 {
            let tall = if k % 13 == 0 { 0.2 } else { 0.1 };
            ruler.push(stroke(
                stage,
                vec![v(x0 + k as f64 * step, y - tall), v(x0 + k as f64 * step, y + tall)],
                line_style(Palette::faint(), 2.0, 0.9),
            ));
        }
        s.wait_for(stage, "Measured in thirty ninths")?;
        fade_in_all(stage, &ruler, 0.6)?;
        let mut marks = Vec::new();
        for i in 0..3 {
            let k = tails[i].num * 39 / tails[i].den;
            let x = x0 + k as f64 * step;
            marks.push(stroke(
                stage,
                vec![v(x, y - 0.05), v(x, y + 0.5)],
                line_style(town_color(i), 6.0, 1.0),
            ));
            marks.push(kit.text(stage, &k.to_string(), 22.0, town_color(i), v(x, y + 0.75))?);
        }
        fade_in_lagged(stage, &marks, 1.2, 0.3)?;
        s.wait_for(stage, "Ash misses by a single")?;
        let ka = tails[ASH].num * 39 / tails[ASH].den;
        let gap = stroke(
            stage,
            vec![
                v(x0 + ka as f64 * step, y + 0.5),
                v(x0 + (ka + 1) as f64 * step, y + 0.5),
            ],
            line_style(Palette::red(), 8.0, 1.0),
        );
        fade_in_all(stage, &[gap], 0.3)?;
        flash_all(stage, &[gap], 0.9)?;
        s.wait_for(stage, "The real question")?;
        flash_all(stage, &labels, 1.0)?;
        pause(stage, 0.6)?;
        kit.finish(stage)
    }
}
