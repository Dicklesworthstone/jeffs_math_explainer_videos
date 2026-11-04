//! 03 Whole parts first, then the largest tails: Hamilton's rule at fifteen
//! seats, why the tails always add up to the number of leftover seats, and
//! the quota promise it therefore keeps. From `model::share`/`hamilton`.

use explainer_kit::{Kit, Palette, rect, v};
use fmn::prelude::*;

use super::draw::{
    ROW_Y, Rows, check, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, flat, line_style, morph_all,
    move_all, pause, quota_box, refill, say, stroke, think_here, token_at, token_spot, tokens, town_color,
};
use crate::model::{self, ASH, POP, decimal, share};

pub struct LargestTails {
    pub kit: Kit,
}

/// Tails drawn out of the rows: this many scene units per seat.
const CHIP_UNIT: f64 = 2.0;
const CHIP_H: f64 = 0.3;
const CHIP_X: f64 = 2.7;

impl SceneConstruct for LargestTails {
    fn name(&self) -> &str {
        "03_largest_tails"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let h = 15;
        let wholes: Vec<u64> = (0..3).map(|i| share(&POP, i, h).floor() as u64).collect();
        let tails: Vec<model::Ratio> = (0..3).map(|i| share(&POP, i, h).tail()).collect();
        let seats = model::hamilton(&POP, h).map_err(fail)?;
        let left = h - wholes.iter().sum::<u64>();

        // 03_1 the rule, on the grid
        let s = say(&kit, stage, "03_1")?;
        let rows = Rows::build(&kit, stage, &POP, h, 12)?;
        let mut base = rows.names.clone();
        base.extend(&rows.grids);
        fade_in_all(stage, &base, 0.8)?;
        let all_bands: Vec<Mob> = rows.bands.iter().flatten().copied().collect();
        fade_in_lagged(stage, &all_bands, 1.2, 0.2)?;
        s.wait_for(stage, "Give every town the whole part")?;
        let whole_bands: Vec<Mob> = rows.bands.iter().map(|b| b[0]).collect();
        flash_all(stage, &whole_bands, 1.0)?;
        s.wait_for(stage, "largest leftover fractions")?;
        let tail_bands: Vec<Mob> = rows.bands.iter().map(|b| b[1]).collect();
        flash_all(stage, &tail_bands, 1.0)?;

        // 03_2 whole parts as seats; what is left over
        let s = say(&kit, stage, "03_2")?;
        s.wait_for(stage, "the whole parts are")?;
        let mut toks: Vec<Vec<Mob>> = Vec::new();
        for i in 0..3 {
            let row = tokens(stage, ROW_Y[i], 0, wholes[i] as usize, town_color(i))?;
            fade_in_lagged(stage, &row, 0.2 * row.len() as f64 + 0.4, 0.4)?;
            toks.push(row);
        }
        let sum = format!(
            "{} + {} + {} = {}",
            wholes[0],
            wholes[1],
            wholes[2],
            wholes.iter().sum::<u64>()
        );
        let sum_t = kit.text(stage, &sum, 30.0, Palette::ink(), v(-1.6, -2.7))?;
        fade_in_all(stage, &[sum_t], 0.5)?;
        s.wait_for(stage, "one seat is left over")?;
        let spare = token_at(stage, v(CHIP_X + 0.3, 2.9), Palette::hero())?;
        let spare_t = kit.text_left(
            stage,
            &format!("{left} left over"),
            24.0,
            Palette::hero(),
            v(CHIP_X + 0.7, 2.9),
        )?;
        fade_in_all(stage, &[spare, spare_t], 0.5)?;

        // 03_3 the tails, lifted out side by side
        let s = say(&kit, stage, "03_3")?;
        let mut chips = Vec::new();
        let mut chip_labels = Vec::new();
        for i in 0..3 {
            let w = (tails[i].to_f64() * CHIP_UNIT).max(0.004);
            let c = rect(
                v(CHIP_X, ROW_Y[i] - CHIP_H / 2.0),
                w,
                CHIP_H,
                flat(Palette::hero(), 0.9),
            )?;
            chips.push(stage.arena_mut().add(c));
            chip_labels.push(kit.text_left(
                stage,
                &decimal(tails[i], 2),
                24.0,
                town_color(i),
                v(CHIP_X + CHIP_UNIT + 0.25, ROW_Y[i]),
            )?);
        }
        s.wait_for(stage, "The tails are")?;
        flash_all(stage, &tail_bands, 0.8)?;
        let mut lifted = chips.clone();
        lifted.extend(&chip_labels);
        fade_in_lagged(stage, &lifted, 1.4, 0.2)?;
        let q = think_here(&kit, stage)?;

        // 03_4 the largest tail wins: Ash
        say(&kit, stage, "03_4")?;
        fade_out_all(stage, &[q, spare_t], 0.3)?;
        let winner = (0..3)
            .max_by(|&a, &b| tails[a].cmp(&tails[b]))
            .ok_or_else(|| fail("no tails".into()))?;
        if winner != ASH || seats[ASH] != wholes[ASH] + 1 {
            return Err(fail(format!(
                "03_4 expects Ash to take the leftover seat; model gives {seats:?}"
            )));
        }
        flash_all(stage, &[chips[winner], chip_labels[winner]], 0.8)?;
        move_all(
            stage,
            &[(spare, token_spot(ROW_Y[winner], wholes[winner] as usize))],
            0.8,
        )?;
        refill(stage, &[(spare, town_color(winner), 0.95)], 0.3)?;
        toks[winner].push(spare);
        let result = kit.text(
            stage,
            &format!("{}, {}, {}", seats[0], seats[1], seats[2]),
            34.0,
            Palette::hero(),
            v(-1.6, -2.7),
        )?;
        morph_all(stage, &[(sum_t, result)], 0.6)?;

        // 03_5 the tails add up to the leftover count, at 15 and at 16
        let s = say(&kit, stage, "03_5")?;
        fade_out_all(stage, &[result], 0.3)?;
        let line_a = -2.55;
        s.wait_for(stage, "the tails always add up")?;
        let mut x = -2.4;
        let mut moves = Vec::new();
        for i in 0..3 {
            let w = tails[i].to_f64() * CHIP_UNIT;
            moves.push((chips[i], v(x + w.max(0.004) / 2.0, line_a)));
            x += w;
        }
        move_all(stage, &moves, 1.0)?;
        fade_out_all(stage, &chip_labels, 0.3)?;
        let ruler_a = ruler(stage, -2.4, line_a, left)?;
        let eq_a = format!(
            "{} + {} + {} = {}",
            decimal(tails[0], 2),
            decimal(tails[1], 2),
            decimal(tails[2], 2),
            left
        );
        let eq_a = kit.text_left(stage, &eq_a, 24.0, Palette::ink(), v(2.0, line_a))?;
        let mut a_mobs = ruler_a;
        a_mobs.push(eq_a);
        fade_in_all(stage, &a_mobs, 0.6)?;
        s.wait_for(stage, "Every tail is less than one")?;
        let h16 = 16;
        let tails16: Vec<model::Ratio> = (0..3).map(|i| share(&POP, i, h16).tail()).collect();
        let left16 = h16 - (0..3).map(|i| share(&POP, i, h16).floor() as u64).sum::<u64>();
        let line_b = -3.35;
        let mut x = -2.4;
        let mut b_mobs = Vec::new();
        for i in 0..3 {
            let w = tails16[i].to_f64() * CHIP_UNIT;
            let c = rect(
                v(x, line_b - CHIP_H / 2.0),
                w.max(0.004),
                CHIP_H,
                flat(town_color(i), 0.85),
            )?;
            b_mobs.push(stage.arena_mut().add(c));
            x += w;
        }
        b_mobs.extend(ruler(stage, -2.4, line_b, left16)?);
        let eq_b = format!(
            "{} + {} + {} = {}   (16 seats)",
            decimal(tails16[0], 2),
            decimal(tails16[1], 2),
            decimal(tails16[2], 2),
            left16
        );
        b_mobs.push(kit.text_left(stage, &eq_b, 24.0, Palette::ink(), v(2.0, line_b))?);
        fade_in_lagged(stage, &b_mobs, 1.4, 0.2)?;

        // 03_6 the quota promise, kept
        let s = say(&kit, stage, "03_6")?;
        s.wait_for(stage, "rounded down or rounded up")?;
        let mut boxes = Vec::new();
        for i in 0..3 {
            let q = share(&POP, i, h);
            boxes.push(quota_box(stage, ROW_Y[i], q.floor(), q.ceil(), Palette::green())?);
        }
        fade_in_lagged(stage, &boxes, 1.2, 0.3)?;
        s.wait_for(stage, "keeps it at every house size")?;
        let ok = check(stage, v(1.6, 2.4), 0.6, Palette::green());
        fade_in_all(stage, &[ok], 0.4)?;
        flash_all(stage, &boxes, 0.9)?;
        pause(stage, 0.6)?;
        kit.finish(stage)
    }
}

/// A ruler of `seats` seats from `x0` along height `y`: a base line with a
/// tick at every whole seat, detached.
fn ruler(stage: &mut Stage<'_>, x0: f64, y: f64, seats: u64) -> fmn::Result<Vec<Mob>> {
    let x1 = x0 + seats as f64 * CHIP_UNIT;
    let mut out = vec![stroke(
        stage,
        vec![v(x0, y - 0.24), v(x1, y - 0.24)],
        line_style(Palette::ink(), 2.0, 0.8),
    )];
    for k in 0..=seats {
        let x = x0 + k as f64 * CHIP_UNIT;
        out.push(stroke(
            stage,
            vec![v(x, y - 0.32), v(x, y + 0.22)],
            line_style(Palette::ink(), 2.5, 0.9),
        ));
    }
    Ok(out)
}
