//! 05 Tails move at different speeds: each share as a hand on a dial (one
//! turn = one seat), driven by one house tracker; per added seat Ash's hand
//! turns 4/39, Birch's 9/39, Cedar's 26/39 of a turn, so faster hands lap
//! Ash's while its whole part stays put. From `model::share`.

use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{
    self, dial, fade_in_all, fade_in_lagged, fade_out_all, flash_all, morph_all, pause, run_tracker, say, step_arc,
    think_here, town_color,
};
use crate::model::{self, NAMES, POP, decimal, share};

pub struct Dials {
    pub kit: Kit,
}

/// The dials' centres in this chapter and the next.
pub const CENTRES: [[f64; 2]; 3] = [[-4.0, 0.7], [0.0, 0.7], [4.0, 0.7]];

/// "1st Birch 0.69   2nd Cedar 0.67   3rd Ash 0.64": the tails at `house`,
/// largest first, from the model.
pub fn ranking(house: u64) -> (String, Vec<usize>) {
    let mut order: Vec<usize> = (0..3).collect();
    order.sort_by(|&a, &b| share(&POP, b, house).tail().cmp(&share(&POP, a, house).tail()));
    let place = ["1st", "2nd", "3rd"];
    let parts: Vec<String> = order
        .iter()
        .enumerate()
        .map(|(r, &i)| format!("{} {} {}", place[r], NAMES[i], decimal(share(&POP, i, house).tail(), 2)))
        .collect();
    (parts.join("     "), order)
}

/// The three dials at `house` seats, with names above; returns everything
/// to fade in. The live whole parts are already on stage.
pub fn three_dials(kit: &Kit, stage: &mut Stage<'_>, tracker: Mob, house: f64) -> fmn::Result<Vec<Mob>> {
    let total = model::total(&POP);
    let mut out = Vec::new();
    for i in 0..3 {
        let at = v(CENTRES[i][0], CENTRES[i][1]);
        out.push(kit.text(
            stage,
            NAMES[i],
            28.0,
            town_color(i),
            v(at[0], at[1] + draw::DIAL_R + 0.45),
        )?);
        out.extend(dial(kit, stage, at, POP[i], total, tracker, house, town_color(i))?);
    }
    Ok(out)
}

impl SceneConstruct for Dials {
    fn name(&self) -> &str {
        "05_dials"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let total = model::total(&POP);
        let house = stage.add_value_tracker(15.0);

        // 05_1 a share as a hand on a dial
        let s = say(&kit, stage, "05_1")?;
        let dials = three_dials(&kit, stage, house, 15.0)?;
        fade_in_lagged(stage, &dials, 1.8, 0.15)?;
        s.wait_for(stage, "passes the top")?;
        // a quick demonstration on Cedar's dial alone would move every hand;
        // instead flash the top ticks, where the whole part ticks up
        let ticks: Vec<Mob> = (0..3).map(|i| dials[i * 6 + 2]).collect();
        flash_all(stage, &ticks, 1.0)?;

        // 05_2 each hand's step for one more seat
        let s = say(&kit, stage, "05_2")?;
        let mut wedges = Vec::new();
        let phrases = ["four thirty ninths of a turn", "Birch's hand turns", "Cedar's turns"];
        for i in 0..3 {
            s.wait_for(stage, phrases[i])?;
            let at = v(CENTRES[i][0], CENTRES[i][1]);
            let from = share(&POP, i, 15).tail().to_f64();
            let step = POP[i] as f64 / total as f64;
            let w = step_arc(stage, at, from, step, Palette::ink());
            let num = POP[i] * 39 / total;
            let label = kit.text(
                stage,
                &format!("+{num}/39 turn"),
                22.0,
                Palette::ink(),
                v(at[0], at[1] - 1.75),
            )?;
            fade_in_all(stage, &[w, label], 0.6)?;
            wedges.extend([w, label]);
        }

        // 05_3 which hand ends up in front?
        let s = say(&kit, stage, "05_3")?;
        let (rank15, _) = ranking(15);
        let rank_t = kit.text(stage, &rank15, 24.0, Palette::ink(), v(0.0, -2.75))?;
        s.wait_for(stage, "Ash's hand is ahead")?;
        fade_in_all(stage, &[rank_t], 0.6)?;
        let q = think_here(&kit, stage)?;

        // 05_4 turn every hand by its own step
        let s = say(&kit, stage, "05_4")?;
        fade_out_all(stage, &[q], 0.2)?;
        fade_out_all(stage, &wedges, 0.4)?;
        s.wait_for(stage, "Ash's hand creeps")?;
        run_tracker(stage, house, 16.0, 3.2)?;
        let (rank16, order16) = ranking(16);
        let rank_t16 = kit.text(stage, &rank16, 24.0, Palette::ink(), v(0.0, -2.75))?;
        morph_all(stage, &[(rank_t, rank_t16)], 0.8)?;
        if order16[2] != model::ASH {
            return Err(draw::fail(format!(
                "05_4 expects Ash last at 16 seats, got order {order16:?}"
            )));
        }
        s.wait_for(stage, "Nothing about Ash")?;
        flash_all(stage, &[dials[0]], 1.0)?;

        // 05_5 the mechanism: tails wrap around at their own speeds
        let s = say(&kit, stage, "05_5")?;
        s.wait_for(stage, "They wrap around")?;
        run_tracker(stage, house, 20.0, 5.0)?;
        draw::ease_tracker(stage, house, 16.0, 2.0)?;
        s.wait_for(stage, "A slow hand")?;
        flash_all(stage, &[dials[1], dials[4]], 1.2)?;
        pause(stage, 0.6)?;
        kit.finish(stage)
    }
}
