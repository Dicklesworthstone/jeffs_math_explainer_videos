//! 05 Tails move at different speeds: each share as a hand on a dial (one
//! turn = one seat). Every town's dial has its own house tracker, so a hand
//! moves only when the narration names it: in 05_1 Ash's hand alone turns
//! from no seats to fifteen and passes the top once (its whole part ticks
//! from 0 to 1); in 05_4 Ash's, Birch's and Cedar's hands each take their own
//! step, 4/39, 9/39 and 26/39 of a turn, as each is named. From
//! `model::share`.

use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{
    self, dial, fade_in_all, fade_in_lagged, fade_out_all, flash_all, morph_all, pause, run_tracker, run_trackers, say,
    step_arc, think_here, town_color,
};
use crate::model::{self, ASH, BIRCH, CEDAR, NAMES, POP, decimal, share};

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

/// One house tracker per town, all at `house`: each dial turns on its own.
pub fn house_trackers(stage: &mut Stage<'_>, house: f64) -> [Mob; 3] {
    [
        stage.add_value_tracker(house),
        stage.add_value_tracker(house),
        stage.add_value_tracker(house),
    ]
}

/// Town `i`'s dial, driven by its own tracker (now at `house`), with its name
/// above: [name, ring, top tick, arc, hand, anchor] to fade in. The live
/// whole part is already on stage.
pub fn town_dial(kit: &Kit, stage: &mut Stage<'_>, i: usize, tracker: Mob, house: f64) -> fmn::Result<Vec<Mob>> {
    let at = v(CENTRES[i][0], CENTRES[i][1]);
    let mut out = vec![kit.text(
        stage,
        NAMES[i],
        28.0,
        town_color(i),
        v(at[0], at[1] + draw::DIAL_R + 0.45),
    )?];
    out.extend(dial(
        kit,
        stage,
        at,
        POP[i],
        model::total(&POP),
        tracker,
        house,
        town_color(i),
    )?);
    Ok(out)
}

/// The three dials, each on its own tracker, all at `house`: six mobjects
/// per town (see [`town_dial`]), Ash first.
pub fn three_dials(kit: &Kit, stage: &mut Stage<'_>, trackers: [Mob; 3], house: f64) -> fmn::Result<Vec<Mob>> {
    let mut out = Vec::new();
    for (i, &t) in trackers.iter().enumerate() {
        out.extend(town_dial(kit, stage, i, t, house)?);
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
        // Ash's hand first reaches the top when its share is one whole seat,
        // at P / p seats; on the way to fifteen it passes the top exactly once
        let ash_top = model::tile_len(&POP, ASH).to_f64();
        if share(&POP, ASH, 15).floor() != 1 || ash_top >= 15.0 {
            return Err(draw::fail(
                "05_1 shows Ash's hand passing the top once on the way to fifteen seats".into(),
            ));
        }
        // Ash's dial starts at no seats; Birch's and Cedar's appear at fifteen
        let t = [
            stage.add_value_tracker(0.0),
            stage.add_value_tracker(15.0),
            stage.add_value_tracker(15.0),
        ];

        // 05_1 a share as a hand on a dial: Ash's alone, from no seats to fifteen
        let s = say(&kit, stage, "05_1")?;
        let ash = town_dial(&kit, stage, ASH, t[ASH], 0.0)?;
        fade_in_all(stage, &ash, 0.8)?;
        s.wait_for(stage, "Watch Ash's dial")?;
        run_tracker(stage, t[ASH], ash_top - 0.75, 2.4)?;
        s.wait_for(stage, "Each time the hand passes the top")?;
        // through the top: the whole part ticks from 0 to 1, the tail restarts
        run_tracker(stage, t[ASH], ash_top + 1.25, 1.6)?;
        s.wait_for(stage, "and the tail starts again from zero")?;
        run_tracker(stage, t[ASH], 15.0, 1.8)?;
        s.wait_for(stage, "Birch's and Cedar's dials")?;
        let mut others = town_dial(&kit, stage, BIRCH, t[BIRCH], 15.0)?;
        others.extend(town_dial(&kit, stage, CEDAR, t[CEDAR], 15.0)?);
        fade_in_lagged(stage, &others, 1.2, 0.1)?;
        let mut dials = ash;
        dials.extend(others);

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
            wedges.push([w, label]);
        }

        // 05_3 which hand ends up in front?
        let s = say(&kit, stage, "05_3")?;
        let (rank15, _) = ranking(15);
        let rank_t = kit.text(stage, &rank15, 24.0, Palette::ink(), v(0.0, -2.75))?;
        s.wait_for(stage, "Ash's hand is ahead")?;
        fade_in_all(stage, &[rank_t], 0.6)?;
        let q = think_here(&kit, stage)?;

        // 05_4 each hand takes its own step, as the narration names it
        let s = say(&kit, stage, "05_4")?;
        fade_out_all(stage, &[q], 0.2)?;
        let named = [
            ("Ash's hand creeps", 1.2),
            ("Birch's moves more than twice as far", 1.5),
            ("Cedar's swings two thirds", 1.9),
        ];
        for (i, (phrase, rt)) in named.iter().enumerate() {
            s.wait_for(stage, phrase)?;
            run_tracker(stage, t[i], 16.0, *rt)?;
            fade_out_all(stage, &wedges[i], 0.3)?;
        }
        let (rank16, order16) = ranking(16);
        let rank_t16 = kit.text(stage, &rank16, 24.0, Palette::ink(), v(0.0, -2.75))?;
        morph_all(stage, &[(rank_t, rank_t16)], 0.8)?;
        if order16[2] != ASH {
            return Err(draw::fail(format!(
                "05_4 expects Ash last at 16 seats, got order {order16:?}"
            )));
        }
        s.wait_for(stage, "Nothing about Ash")?;
        flash_all(stage, &[dials[0]], 1.0)?;

        // 05_5 the mechanism: tails wrap around at their own speeds
        let s = say(&kit, stage, "05_5")?;
        s.wait_for(stage, "They wrap around")?;
        run_trackers(stage, &t.map(|m| (m, 20.0)), 5.0, true)?;
        run_trackers(stage, &t.map(|m| (m, 15.0)), 1.6, false)?;
        s.wait_for(stage, "can be overtaken")?;
        // the slow hand is overtaken again, 15 to 16, as narrated
        run_trackers(stage, &t.map(|m| (m, 16.0)), 2.4, true)?;
        pause(stage, 0.6)?;
        kit.finish(stage)
    }
}
