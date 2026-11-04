//! 09 The cost of never taking back: Jefferson gives Cedar eleven seats for
//! a share of exactly ten (above its quota box), the reason (a dropped tail
//! is a bigger slice of a small town: 42.5% of Ash's count, 2.7% of
//! Cedar's), New York after the 1830 census, and the two families' trade.

use explainer_kit::{Kit, Palette, rect, v};
use fmn::prelude::*;

use super::draw::{
    ROW_Y, Rows, fade_in_all, fade_in_lagged, fail, flash_all, flat, pause, quota_box, readout, refill, say,
    score_head, score_row, think_here, tokens, town_color, wipe,
};
use crate::model::{self, ASH, CEDAR, EXAMPLE_YARDSTICK, POP, Ratio, Rule, decimal, share};

pub struct Cost {
    pub kit: Kit,
}

/// The "what rounding down throws away" bars: full length and height.
const BAR_W: f64 = 8.0;
const BAR_H: f64 = 0.5;

impl SceneConstruct for Cost {
    fn name(&self) -> &str {
        "09_cost"
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
        let seats = model::divisor(Rule::Jefferson, &POP, 15).map_err(fail)?;
        let cedar_q = share(&POP, CEDAR, 15);
        if u128::from(seats[CEDAR]) <= cedar_q.ceil() {
            return Err(fail("09_2 says Jefferson puts Cedar above its share rounded up".into()));
        }

        // 09_1 Jefferson at fifteen seats: look at Cedar
        say(&kit, stage, "09_1")?;
        let rows = Rows::build(&kit, stage, &POP, 15, 12)?;
        let mut first = rows.all();
        let mut toks = Vec::new();
        for i in 0..3 {
            let row = tokens(stage, ROW_Y[i], 0, seats[i] as usize, town_color(i))?;
            first.extend(&row);
            toks.push(row);
        }
        let head = kit.text(stage, "Jefferson, 15 seats", 30.0, Palette::ink(), v(-1.4, 3.3))?;
        first.push(head);
        fade_in_all(stage, &first, 0.7)?;
        flash_all(stage, &[rows.names[2 * CEDAR]], 0.9)?;
        let q = think_here(&kit, stage)?;

        // 09_2 above its quota box; Ash at one seat
        let s = say(&kit, stage, "09_2")?;
        super::draw::fade_out_all(stage, &[q], 0.2)?;
        s.wait_for(stage, "Cedar's share at fifteen seats")?;
        let cbox = quota_box(stage, ROW_Y[CEDAR], cedar_q.floor(), cedar_q.ceil(), Palette::red())?;
        let cread = readout(
            &kit,
            stage,
            &format!("share {}", decimal(cedar_q, 2)),
            ROW_Y[CEDAR],
            town_color(CEDAR),
        )?;
        fade_in_all(stage, &[cbox, cread], 0.5)?;
        s.wait_for(stage, "Jefferson gives it eleven")?;
        let extra = *toks[CEDAR].last().ok_or_else(|| fail("no Cedar tokens".into()))?;
        refill(stage, &[(extra, Palette::red(), 1.0)], 0.4)?;
        flash_all(stage, &[extra, cbox], 1.0)?;
        s.wait_for(stage, "the extra seat came from Ash")?;
        let ash_q = share(&POP, ASH, 15);
        let abox = quota_box(stage, ROW_Y[ASH], ash_q.floor(), ash_q.ceil(), Palette::dim())?;
        let aread = readout(
            &kit,
            stage,
            &format!("share {}", decimal(ash_q, 2)),
            ROW_Y[ASH],
            town_color(ASH),
        )?;
        fade_in_all(stage, &[abox, aread], 0.5)?;
        flash_all(stage, &toks[ASH], 0.8)?;

        // 09_3 the same sliver is a bigger slice of a small town
        wipe(&kit, stage)?;
        let s = say(&kit, stage, "09_3")?;
        let d = u128::from(EXAMPLE_YARDSTICK);
        let mut bars = Vec::new();
        for (row, i) in [(0_usize, ASH), (1, CEDAR)] {
            let y = 1.0 - 2.0 * row as f64;
            let x = Ratio::new(u128::from(POP[i]), d);
            let kept = x.floor() as f64 / x.to_f64();
            let x0 = -BAR_W / 2.0 + 0.8;
            let keep = rect(v(x0, y - BAR_H / 2.0), BAR_W * kept, BAR_H, flat(town_color(i), 0.85))?;
            let drop = rect(
                v(x0 + BAR_W * kept, y - BAR_H / 2.0),
                BAR_W * (1.0 - kept),
                BAR_H,
                flat(Palette::red(), 0.85),
            )?;
            let keep = stage.arena_mut().add(keep);
            let drop = stage.arena_mut().add(drop);
            let label = format!("{}: {} seats' worth → {}", model::NAMES[i], decimal(x, 2), x.floor());
            let l = kit.text_left(stage, &label, 24.0, town_color(i), v(x0, y + 0.55))?;
            let pct = fact(&format!("jefferson.drop_pct.{}", model::KEYS[i]))?;
            let p = kit.text_left(
                stage,
                &format!("{pct}% dropped"),
                24.0,
                Palette::red(),
                v(x0 + BAR_W + 0.25, y),
            )?;
            bars.push([l, keep, drop, p]);
        }
        let stick = kit.text(
            stage,
            &format!("{} people per seat", model::grouped(d)),
            26.0,
            Palette::hero(),
            v(0.0, 3.1),
        )?;
        fade_in_all(stage, &[stick], 0.4)?;
        s.wait_for(stage, "At two thousand three hundred")?;
        fade_in_lagged(stage, &bars[0][..3], 1.0, 0.3)?;
        s.wait_for(stage, "Ash throws away")?;
        fade_in_all(stage, &[bars[0][3]], 0.4)?;
        flash_all(stage, &[bars[0][2]], 0.8)?;
        s.wait_for(stage, "Cedar throws away")?;
        fade_in_lagged(stage, &bars[1], 1.2, 0.3)?;
        s.wait_for(stage, "leans toward large towns")?;
        flash_all(stage, &[bars[1][0]], 0.8)?;

        // 09_4 New York after the 1830 census
        wipe(&kit, stage)?;
        let s = say(&kit, stage, "09_4")?;
        let title = kit.text(
            stage,
            "New York, apportionment after the 1830 census (Jefferson)",
            28.0,
            Palette::ink(),
            v(0.0, 1.9),
        )?;
        fade_in_all(stage, &[title], 0.5)?;
        s.wait_for(stage, "New York's exact share")?;
        let share_t = kit.text(
            stage,
            &format!("share {}", fact("history.ny1830.quota")?),
            40.0,
            Palette::blue(),
            v(-2.8, 0.5),
        )?;
        fade_in_all(stage, &[share_t], 0.5)?;
        s.wait_for(stage, "it received forty")?;
        let seats_t = kit.text(
            stage,
            &format!("seats {}", fact("history.ny1830.seats")?),
            40.0,
            Palette::red(),
            v(2.8, 0.5),
        )?;
        let src = kit.text(
            stage,
            "H. P. Young, Fairness in Apportionment, U.S. Census Bureau, Table 4",
            16.0,
            Palette::dim(),
            v(0.0, -2.9),
        )?;
        fade_in_all(stage, &[seats_t, src], 0.5)?;
        flash_all(stage, &[seats_t], 0.9)?;

        // 09_5 the two families trade flaws
        wipe(&kit, stage)?;
        let s = say(&kit, stage, "09_5")?;
        let head = score_head(&kit, stage, 2, 1.6)?;
        fade_in_all(stage, &head, 0.5)?;
        s.wait_for(stage, "Hamilton's rule keeps")?;
        let h = score_row(&kit, stage, "Hamilton", &[(Some(true), ""), (Some(false), "")], 0.6)?;
        fade_in_lagged(stage, &h, 1.0, 0.3)?;
        s.wait_for(stage, "Jefferson's rule never")?;
        let j = score_row(
            &kit,
            stage,
            "Jefferson",
            &[(Some(false), ""), (Some(true), "")],
            0.6 - 0.72,
        )?;
        fade_in_lagged(stage, &j, 1.0, 0.3)?;
        pause(stage, 0.6)?;
        kit.finish(stage)
    }
}
