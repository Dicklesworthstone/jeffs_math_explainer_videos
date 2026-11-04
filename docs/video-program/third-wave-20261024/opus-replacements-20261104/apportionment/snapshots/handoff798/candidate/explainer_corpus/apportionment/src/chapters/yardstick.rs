//! 07 Change the yardstick, not the rounding: Jefferson's rule. One tracker
//! holds the number of people per seat; every quotient (population ÷
//! yardstick), band and the whole-part total follow it from 2,600 down to
//! 2,300, where the whole parts add up to fifteen. The window of yardsticks
//! that work comes from `model::window`.

use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{
    self, READ_SIZE, READ_X, ROW_Y, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, grid, line_style,
    live_band, name, pause, run_tracker, say, stroke, think_here, token, tokens, town_color,
};
use crate::model::{self, CEDAR, EXAMPLE_YARDSTICK, POP, Rule, grouped, number};

pub struct Yardstick {
    pub kit: Kit,
}

/// The yardstick number line: 2,000 to 2,700 people per seat.
const AXIS: (f64, f64) = (2_000.0, 2_700.0);
const AXIS_X: (f64, f64) = (-5.4, 5.4);
const AXIS_Y: f64 = -2.95;

fn axis_x(d: f64) -> f64 {
    AXIS_X.0 + (d - AXIS.0) / (AXIS.1 - AXIS.0) * (AXIS_X.1 - AXIS_X.0)
}

impl SceneConstruct for Yardstick {
    fn name(&self) -> &str {
        "07_yardstick"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let total = model::total(&POP);
        let standard = (total / 15) as f64;
        let target = EXAMPLE_YARDSTICK as f64;
        let floors = |d: f64| -> Vec<u64> { POP.iter().map(|&p| (p as f64 / d + 1e-9).floor() as u64).collect() };
        let at_standard = floors(standard);
        let jefferson = model::divisor(Rule::Jefferson, &POP, 15).map_err(fail)?;
        if floors(target) != jefferson {
            return Err(fail(
                "07_3: the example yardstick must give Jefferson's fifteen seats".into(),
            ));
        }

        // 07_1 back to fifteen seats: drop every tail, fourteen seats
        let s = say(&kit, stage, "07_1")?;
        let d = stage.add_value_tracker(standard);
        let mut base = Vec::new();
        for i in 0..3 {
            base.extend(name(
                &kit,
                stage,
                i,
                ROW_Y[i],
                &format!("{} people", grouped(u128::from(POP[i]))),
            )?);
            base.extend(grid(stage, ROW_Y[i], 12)?);
        }
        fade_in_all(stage, &base, 0.6)?;
        let mut live = Vec::new();
        let mut tails = Vec::new();
        for i in 0..3 {
            let p = POP[i] as f64;
            let band = live_band(stage, ROW_Y[i], town_color(i), d, move |x| p / x)?;
            tails.push(band[1]);
            live.extend(band);
            live.push(kit.live_value(
                stage,
                move |s| p / s.tracker_value(d).unwrap_or(1.0),
                2,
                false,
                READ_SIZE,
                town_color(i),
                v(READ_X, ROW_Y[i]),
                LEFT,
            )?);
        }
        let stick = kit.live_count(stage, d, 30.0, Palette::hero(), v(-2.4, 3.3))?;
        let stick_t = kit.text_left(stage, "people per seat", 26.0, Palette::hero(), v(-1.55, 3.3))?;
        fade_in_all(stage, &[stick_t], 0.4)?;
        s.wait_for(stage, "drop every tail")?;
        flash_all(stage, &tails, 0.9)?;
        let mut toks: Vec<Vec<Mob>> = Vec::new();
        for i in 0..3 {
            let row = tokens(stage, ROW_Y[i], 0, at_standard[i] as usize, town_color(i))?;
            fade_in_lagged(stage, &row, 0.15 * row.len() as f64 + 0.3, 0.4)?;
            toks.push(row);
        }
        let seats_live = kit.live_value(
            stage,
            move |s| {
                let x = s.tracker_value(d).unwrap_or(1.0);
                POP.iter().map(|&p| (p as f64 / x + 1e-9).floor()).sum::<f64>()
            },
            0,
            false,
            34.0,
            Palette::ink(),
            v(4.6, 3.3),
            ORIGIN,
        )?;
        let seats_t = kit.text_left(stage, "seats", 26.0, Palette::ink(), v(5.1, 3.3))?;
        fade_in_all(stage, &[seats_t], 0.4)?;
        s.wait_for(stage, "Fourteen seats, one short")?;
        flash_all(stage, &[seats_t], 0.8)?;

        // 07_2 up or down?
        say(&kit, stage, "07_2")?;
        let q = think_here(&kit, stage)?;

        // 07_3 down: slide the yardstick to 2,300
        let s = say(&kit, stage, "07_3")?;
        fade_out_all(stage, &[q], 0.2)?;
        s.wait_for(stage, "Slide the yardstick")?;
        run_tracker(stage, d, target, 3.5)?;
        s.wait_for(stage, "Drop the tails")?;
        let gained: Vec<usize> = (0..3).filter(|&i| jefferson[i] > at_standard[i]).collect();
        if gained != [CEDAR] {
            return Err(fail(format!(
                "07_3 shows only Cedar gaining; the model gains {gained:?}"
            )));
        }
        let new = token(stage, ROW_Y[CEDAR], at_standard[CEDAR] as usize, town_color(CEDAR))?;
        fade_in_all(stage, &[new], 0.5)?;
        toks[CEDAR].push(new);
        s.wait_for(stage, "Fifteen seats exactly")?;
        flash_all(stage, &[seats_t, new], 0.9)?;

        // 07_4 the window of yardsticks that give fifteen
        let s = say(&kit, stage, "07_4")?;
        let (low, high) = model::window(Rule::Jefferson, &POP, 15).map_err(fail)?;
        let mut axis = vec![stroke(
            stage,
            vec![v(AXIS_X.0, AXIS_Y), v(AXIS_X.1, AXIS_Y)],
            line_style(Palette::ink(), 2.0, 0.8),
        )];
        for k in 0..=7 {
            let val = AXIS.0 + 100.0 * k as f64;
            let x = axis_x(val);
            axis.push(stroke(
                stage,
                vec![v(x, AXIS_Y - 0.1), v(x, AXIS_Y + 0.1)],
                line_style(Palette::faint(), 2.0, 0.9),
            ));
            if k % 2 == 0 {
                axis.push(kit.text(stage, &grouped(val as u128), 16.0, Palette::dim(), v(x, AXIS_Y - 0.35))?);
            }
        }
        fade_in_all(stage, &axis, 0.5)?;
        s.wait_for(stage, "Any yardstick above")?;
        let (xl, xh) = (axis_x(low.to_f64()), axis_x(high.to_f64()));
        let shade = stroke(
            stage,
            vec![v(xl, AXIS_Y), v(xh, AXIS_Y)],
            line_style(Palette::hero(), 12.0, 0.8),
        );
        let open = Circle::new()
            .radius(0.09)
            .style(line_style(Palette::hero(), 3.0, 1.0))
            .build()
            .moved_to(v(xl, AXIS_Y));
        let open = stage.arena_mut().add(open);
        let shut = Circle::new()
            .radius(0.09)
            .style(draw::flat(Palette::hero(), 1.0))
            .build()
            .moved_to(v(xh, AXIS_Y));
        let shut = stage.arena_mut().add(shut);
        let lo_t = kit.text(
            stage,
            &format!("above {}", number(low)),
            18.0,
            Palette::hero(),
            v(xl - 0.5, AXIS_Y + 0.4),
        )?;
        let hi_t = kit.text(
            stage,
            &format!("up to {}", number(high)),
            18.0,
            Palette::hero(),
            v(xh + 0.5, AXIS_Y + 0.4),
        )?;
        fade_in_lagged(stage, &[shade, open, shut, lo_t, hi_t], 1.4, 0.2)?;
        let mark = stroke(
            stage,
            vec![v(axis_x(target), AXIS_Y - 0.25), v(axis_x(target), AXIS_Y + 0.25)],
            line_style(Palette::ink(), 5.0, 1.0),
        );
        fade_in_all(stage, &[mark], 0.4)?;
        s.wait_for(stage, "That is Jefferson's rule")?;
        flash_all(stage, &[stick_t], 0.9)?;

        // 07_5 one yardstick for every town
        let s = say(&kit, stage, "07_5")?;
        s.wait_for(stage, "no town's tail")?;
        flash_all(stage, &tails, 0.9)?;
        let _ = (live, seats_live, toks, stick);
        pause(stage, 0.6)?;
        kit.finish(stage)
    }
}
