//! 02 Seats do not come in fractions: one seat per 2,600 people, the
//! shares 1.54, 3.46, 10.00, and rounding each share on its own missing the
//! total at 14 and 16 seats. Numbers from `model::share` and `model::naive`.

use explainer_kit::{Kit, Palette, rect, v};
use fmn::prelude::*;

use super::draw::{
    ROW_Y, Rows, fade_in_all, fade_in_lagged, fade_out_all, flash_all, flat, line_style, morph_all, move_all, pause,
    refill, say, share_readouts, stroke, think_here, town_color, wipe,
};
use crate::model::{self, ASH, POP, decimal, grouped, share};

pub struct Shares {
    pub kit: Kit,
}

/// The single line of everyone: 39,000 people across this width.
const LINE_X0: f64 = -5.85;
const LINE_W: f64 = 11.7;
const LINE_Y: f64 = 0.6;

impl SceneConstruct for Shares {
    fn name(&self) -> &str {
        "02_shares"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let total = model::total(&POP);
        let per_seat = total / 15;
        let scale = LINE_W / total as f64;

        // 02_1 everyone on one line, cut into fifteen equal pieces
        let s = say(&kit, stage, "02_1")?;
        let mut bars = Vec::new();
        for i in 0..3 {
            let w = POP[i] as f64 * scale;
            let bar = rect(v(-4.5, ROW_Y[i] - 0.18), w, 0.36, flat(town_color(i), 0.75))?;
            bars.push(stage.arena_mut().add(bar));
        }
        fade_in_lagged(stage, &bars, 1.2, 0.3)?;
        s.wait_for(stage, "Together the towns")?;
        let mut x = LINE_X0;
        let mut moves = Vec::new();
        for i in 0..3 {
            let w = POP[i] as f64 * scale;
            moves.push((bars[i], v(x + w / 2.0, LINE_Y)));
            x += w;
        }
        move_all(stage, &moves, 1.2)?;
        let everyone = kit.text(
            stage,
            &format!("{} people", grouped(u128::from(total))),
            30.0,
            Palette::ink(),
            v(0.0, 1.45),
        )?;
        fade_in_all(stage, &[everyone], 0.5)?;
        s.wait_for(stage, "Fifteen seats")?;
        let mut ticks = Vec::new();
        for k in 0..=15 {
            let tx = LINE_X0 + LINE_W * k as f64 / 15.0;
            ticks.push(stroke(
                stage,
                vec![v(tx, LINE_Y - 0.32), v(tx, LINE_Y + 0.32)],
                line_style(Palette::ink(), 2.5, 0.9),
            ));
        }
        fade_in_lagged(stage, &ticks, 1.5, 0.2)?;
        s.wait_for(stage, "one seat for every")?;
        let x1 = LINE_X0 + LINE_W / 15.0;
        let brace = stroke(
            stage,
            vec![
                v(LINE_X0, LINE_Y - 0.45),
                v(LINE_X0, LINE_Y - 0.6),
                v(x1, LINE_Y - 0.6),
                v(x1, LINE_Y - 0.45),
            ],
            line_style(Palette::hero(), 3.0, 1.0),
        );
        let per = kit.text(
            stage,
            &format!("{} people = 1 seat", grouped(u128::from(per_seat))),
            24.0,
            Palette::hero(),
            v(LINE_X0 + 1.4, LINE_Y - 0.95),
        )?;
        fade_in_all(stage, &[brace, per], 0.6)?;

        // 02_2 each town's share, on the seat grid
        wipe(&kit, stage)?;
        let s = say(&kit, stage, "02_2")?;
        let rows = Rows::build(&kit, stage, &POP, 15, 12)?;
        let mut first = rows.names.clone();
        first.extend(&rows.grids);
        fade_in_all(stage, &first, 0.8)?;
        let reads = share_readouts(&kit, stage, &POP, 15)?;
        for (i, phrase) in [(0, "Ash's share"), (1, "Birch's is"), (2, "Cedar's is")] {
            s.wait_for(stage, phrase)?;
            fade_in_all(stage, &[rows.bands[i][0], rows.bands[i][1], reads[i]], 0.6)?;
        }
        s.wait_for(stage, "call this exact number")?;
        let quota = kit.text(stage, "share = quota", 26.0, Palette::dim(), v(4.6, 3.1))?;
        fade_in_all(stage, &[quota], 0.5)?;

        // 02_3 a seat is a person
        let s = say(&kit, stage, "02_3")?;
        s.wait_for(stage, "Nobody can sit")?;
        flash_all(stage, &[rows.bands[ASH][1]], 1.0)?;
        s.wait_for(stage, "add up to exactly")?;
        let need = kit.text(stage, "? + ? + ? = 15", 32.0, Palette::ink(), v(-1.6, -2.75))?;
        fade_in_all(stage, &[need], 0.5)?;

        // 02_4 round each to the nearest: fine at fifteen
        let s = say(&kit, stage, "02_4")?;
        let r15 = model::naive(&POP, 15);
        let mut arrows = Vec::new();
        for i in 0..3 {
            arrows.push(kit.text_left(stage, &format!("→ {}", r15[i]), 24.0, town_color(i), v(2.45, ROW_Y[i]))?);
        }
        s.wait_for(stage, "that works")?;
        fade_in_lagged(stage, &arrows, 1.2, 0.4)?;
        let sum = format!("{} + {} + {} = {}", r15[0], r15[1], r15[2], r15.iter().sum::<u64>());
        let sum15 = kit.text(stage, &sum, 32.0, Palette::green(), v(-1.6, -2.75))?;
        morph_all(stage, &[(need, sum15)], 0.7)?;
        let q = think_here(&kit, stage)?;

        // 02_5 the table: 14, 15 and 16 seats
        say(&kit, stage, "02_5")?;
        fade_out_all(stage, &[q], 0.2)?;
        explainer_kit::fade_all(stage, 0.5)?;
        let cols = [-5.2, -3.2, -1.6, 0.0, 2.3, 4.6];
        let heads = ["seats", "Ash", "Birch", "Cedar", "rounded", "total"];
        let mut head = Vec::new();
        for (c, h) in heads.iter().enumerate() {
            let color = if (1..=3).contains(&c) {
                town_color(c - 1)
            } else {
                Palette::dim()
            };
            head.push(kit.text(stage, h, 24.0, color, v(cols[c], 2.2))?);
        }
        let rule = stroke(
            stage,
            vec![v(-6.0, 1.85), v(5.4, 1.85)],
            line_style(Palette::faint(), 2.0, 0.8),
        );
        head.push(rule);
        fade_in_all(stage, &head, 0.5)?;
        let mut totals = Vec::new();
        let mut others = head.clone();
        for (r, h) in [14_u64, 15, 16].into_iter().enumerate() {
            let y = 1.2 - 1.0 * r as f64;
            let rounded = model::naive(&POP, h);
            let sum: u64 = rounded.iter().sum();
            let mut cells = vec![kit.text(stage, &h.to_string(), 28.0, Palette::ink(), v(cols[0], y))?];
            for i in 0..3 {
                cells.push(kit.text(
                    stage,
                    &decimal(share(&POP, i, h), 2),
                    28.0,
                    town_color(i),
                    v(cols[i + 1], y),
                )?);
            }
            let r_text = format!("{}  {}  {}", rounded[0], rounded[1], rounded[2]);
            cells.push(kit.text(stage, &r_text, 28.0, Palette::ink(), v(cols[4], y))?);
            let ok = sum == h;
            let t = kit.text(
                stage,
                &sum.to_string(),
                32.0,
                if ok { Palette::green() } else { Palette::red() },
                v(cols[5], y),
            )?;
            totals.push((t, ok));
            others.extend(cells.iter().copied());
            cells.push(t);
            fade_in_lagged(stage, &cells, 1.6, 0.25)?;
            pause(stage, 0.3)?;
        }
        let wrong: Vec<Mob> = totals.iter().filter(|t| !t.1).map(|t| t.0).collect();
        flash_all(stage, &wrong, 1.0)?;

        // 02_6 the total is what ties them together
        let s = say(&kit, stage, "02_6")?;
        s.wait_for(stage, "the total")?;
        let dim: Vec<(Mob, Srgb, f64)> = others.iter().map(|&m| (m, Palette::dim(), 0.35)).collect();
        refill(stage, &dim, 0.6)?;
        let all_totals: Vec<Mob> = totals.iter().map(|t| t.0).collect();
        flash_all(stage, &all_totals, 1.0)?;

        // 02_7 two families
        wipe(&kit, stage)?;
        let s = say(&kit, stage, "02_7")?;
        let mut cards = Vec::new();
        for (j, label) in ["compare what is left over", "change the people per seat"]
            .iter()
            .enumerate()
        {
            let x = if j == 0 { -3.3 } else { 3.3 };
            let bg = Rectangle::new()
                .width(5.6)
                .height(1.4)
                .corner_radius(0.12)
                .style(line_style(Palette::ink(), 3.0, 0.8))
                .build()?
                .moved_to(v(x, 0.4));
            let bg = stage.arena_mut().add(bg);
            let t = kit.text(stage, label, 28.0, Palette::ink(), v(x, 0.4))?;
            cards.push([bg, t]);
        }
        s.wait_for(stage, "One keeps")?;
        fade_in_all(stage, &cards[0], 0.6)?;
        s.wait_for(stage, "The other changes")?;
        fade_in_all(stage, &cards[1], 0.6)?;
        s.wait_for(stage, "The paradox lives")?;
        refill(stage, &[(cards[0][0], Palette::hero(), 1.0)], 0.5)?;
        flash_all(stage, &[cards[0][1]], 1.0)?;
        pause(stage, 0.6)?;
        kit.finish(stage)
    }
}
