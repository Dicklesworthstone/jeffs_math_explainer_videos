//! 08 One list for every house size: each town's signposts (population ÷
//! 1, 2, 3, …) on one ranked list; Jefferson reads it from the top; the
//! sixteenth signpost is Birch's fourth; a bigger house reads further down
//! the same list, so no seat is ever taken back; and the exact tie at
//! 2,000. The list is `model::signposts`; the counts are read off it.

use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{
    CARD_H, CARD_W, LIST_X, ROW_Y, card_y, cutoff, cutoff_y, fade_in_all, fade_in_lagged, fade_out_all, fail,
    flash_all, flat, line_style, morph_all, move_all, pause, say, signpost_cards, stroke, think_here, town_color,
};
use crate::model::{self, BIRCH, NAMES, POP, Rule};

pub struct Signposts {
    pub kit: Kit,
}

/// Seats per town among the first `n` signposts of `list`.
fn counts(list: &[(usize, u64, model::Ratio)], n: usize) -> [usize; 3] {
    let mut c = [0; 3];
    for e in &list[..n] {
        c[e.0] += 1;
    }
    c
}

/// A brace at `x` from `top` down to `bottom`, opening right, detached.
fn brace(stage: &mut Stage<'_>, x: f64, top: f64, bottom: f64, color: Srgb) -> Mob {
    stroke(
        stage,
        vec![v(x + 0.1, top), v(x, top), v(x, bottom), v(x + 0.1, bottom)],
        line_style(color, 3.0, 1.0),
    )
}

/// The three counters on the left: "Ash  1" and so on, detached.
fn counters(kit: &Kit, stage: &mut Stage<'_>, c: [usize; 3]) -> fmn::Result<Vec<Mob>> {
    (0..3)
        .map(|i| kit.text(stage, &c[i].to_string(), 44.0, town_color(i), v(-2.2, ROW_Y[i])))
        .collect()
}

impl SceneConstruct for Signposts {
    fn name(&self) -> &str {
        "08_signposts"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let list = model::signposts(Rule::Jefferson, &POP, 19);
        let cards = signpost_cards(&kit, stage, &POP, 19)?;
        let ranks_of = |town: usize| -> Vec<usize> { (0..19).filter(|&r| list[r].0 == town).collect() };
        for h in [15_usize, 16, 17] {
            let c = counts(&list, h);
            let want = model::divisor(Rule::Jefferson, &POP, h as u64).map_err(fail)?;
            if (0..3).any(|i| c[i] as u64 != want[i]) {
                return Err(fail(format!(
                    "08: the list's first {h} signposts disagree with Jefferson"
                )));
            }
        }

        // 08_1 Cedar's first three signposts
        let s = say(&kit, stage, "08_1")?;
        let mut names = Vec::new();
        for i in 0..3 {
            names.push(kit.text_left(stage, NAMES[i], 30.0, town_color(i), v(-5.8, ROW_Y[i]))?);
        }
        let head = kit.text(
            stage,
            "signposts: people per seat when a seat opens",
            20.0,
            Palette::dim(),
            v(LIST_X, 3.78),
        )?;
        fade_in_all(stage, &[names.clone(), vec![head]].concat(), 0.6)?;
        let cedar = ranks_of(model::CEDAR);
        for (j, phrase) in ["its first seat", "its second at", "its third at"].iter().enumerate() {
            s.wait_for(stage, phrase)?;
            fade_in_all(stage, &cards[cedar[j]], 0.5)?;
        }

        // 08_2 Ash's and Birch's, then the whole list read from the top
        let s = say(&kit, stage, "08_2")?;
        let ash = ranks_of(model::ASH);
        let birch = ranks_of(BIRCH);
        s.wait_for(stage, "Ash's seats open")?;
        let mut shown: Vec<usize> = cedar[..3].to_vec();
        let ash_two: Vec<Mob> = ash[..2].iter().flat_map(|&r| cards[r].clone()).collect();
        fade_in_lagged(stage, &ash_two, 0.9, 0.3)?;
        shown.extend(&ash[..2]);
        s.wait_for(stage, "Birch's open")?;
        let birch_three: Vec<Mob> = birch[..3].iter().flat_map(|&r| cards[r].clone()).collect();
        fade_in_lagged(stage, &birch_three, 1.2, 0.3)?;
        shown.extend(&birch[..3]);
        s.wait_for(stage, "Put every signpost on one list")?;
        // the rest of the list, except the sixteenth, which is the question
        let rest: Vec<Mob> = (0..19)
            .filter(|r| !shown.contains(r) && *r != 15)
            .flat_map(|r| cards[r].clone())
            .collect();
        fade_in_lagged(stage, &rest, 1.6, 0.1)?;
        s.wait_for(stage, "Fifteen seats means")?;
        let line = cutoff(stage, 15);
        let c15 = counters(&kit, stage, counts(&list, 15))?;
        fade_in_all(stage, &[vec![line], c15.clone()].concat(), 0.6)?;

        // 08_3 whose signpost is sixteenth?
        say(&kit, stage, "08_3")?;
        let blank = Rectangle::new()
            .width(CARD_W)
            .height(CARD_H)
            .corner_radius(0.05)
            .style(line_style(Palette::hero(), 3.0, 1.0))
            .build()?
            .moved_to(v(LIST_X, card_y(16)));
        let blank = stage.arena_mut().add(blank);
        fade_in_all(stage, &[blank], 0.4)?;
        let q = think_here(&kit, stage)?;

        // 08_4 Birch's fourth; then Cedar's twelfth
        let s = say(&kit, stage, "08_4")?;
        fade_out_all(stage, &[q, blank], 0.3)?;
        if list[15].0 != BIRCH {
            return Err(fail("08_4 says the sixteenth signpost is Birch's".into()));
        }
        fade_in_all(stage, &cards[15], 0.5)?;
        flash_all(stage, &cards[15][1..], 0.9)?;
        s.wait_for(stage, "Sixteen seats gives")?;
        move_all(stage, &[(line, v(LIST_X, cutoff_y(16)))], 0.6)?;
        let c16 = counters(&kit, stage, counts(&list, 16))?;
        morph_all(
            stage,
            &c15.iter().copied().zip(c16.iter().copied()).collect::<Vec<_>>(),
            0.5,
        )?;
        s.wait_for(stage, "the seventeenth signpost")?;
        move_all(stage, &[(line, v(LIST_X, cutoff_y(17)))], 0.6)?;
        let c17 = counters(&kit, stage, counts(&list, 17))?;
        morph_all(
            stage,
            &c15.iter().copied().zip(c17.iter().copied()).collect::<Vec<_>>(),
            0.5,
        )?;

        // 08_5 a bigger house reads further down the same list
        let s = say(&kit, stage, "08_5")?;
        move_all(stage, &[(line, v(LIST_X, cutoff_y(15)))], 0.5)?;
        let c15b = counters(&kit, stage, counts(&list, 15))?;
        morph_all(
            stage,
            &c15.iter().copied().zip(c15b.iter().copied()).collect::<Vec<_>>(),
            0.4,
        )?;
        let x0 = LIST_X - CARD_W / 2.0;
        let top = card_y(1) + CARD_H / 2.0;
        s.wait_for(stage, "Sixteen seats are the first sixteen")?;
        let b16 = brace(stage, x0 - 0.5, top, cutoff_y(16), Palette::hero());
        let t16 = kit.text(
            stage,
            "16 seats",
            18.0,
            Palette::hero(),
            v(x0 - 1.05, cutoff_y(16) + 0.2),
        )?;
        fade_in_all(stage, &[b16, t16], 0.5)?;
        s.wait_for(stage, "the first fifteen are among them")?;
        let b15 = brace(stage, x0 - 0.25, top, cutoff_y(15), Palette::ink());
        let t15 = kit.text(
            stage,
            "15 seats",
            18.0,
            Palette::ink(),
            v(x0 - 1.05, cutoff_y(15) + 0.6),
        )?;
        fade_in_all(stage, &[b15, t15], 0.5)?;
        s.wait_for(stage, "reads further down")?;
        move_all(stage, &[(line, v(LIST_X, cutoff_y(19)))], 1.6)?;

        // 08_6 the exact tie at 2,000
        let s = say(&kit, stage, "08_6")?;
        let tie = (1..19)
            .find(|&r| list[r].2 == list[r - 1].2)
            .ok_or_else(|| fail("08_6: no tie".into()))?;
        s.wait_for(stage, "exactly tied")?;
        let glow: Vec<Mob> = [tie - 1, tie]
            .iter()
            .map(|&r| {
                let g = Rectangle::new()
                    .width(CARD_W + 0.12)
                    .height(CARD_H + 0.08)
                    .corner_radius(0.06)
                    .style(flat(Palette::red(), 0.25))
                    .build()
                    .map(|m| m.moved_to(v(LIST_X, card_y(r + 1))));
                g.map(|m| stage.arena_mut().add(m))
            })
            .collect::<Result<_, _>>()?;
        let eq = kit.text(
            stage,
            "=",
            26.0,
            Palette::red(),
            v(LIST_X + CARD_W / 2.0 + 0.3, (card_y(tie) + card_y(tie + 1)) / 2.0),
        )?;
        fade_in_all(stage, &[glow.clone(), vec![eq]].concat(), 0.5)?;
        s.wait_for(stage, "a council of eighteen")?;
        move_all(stage, &[(line, v(LIST_X, cutoff_y(tie)))], 0.8)?;
        flash_all(stage, &[eq], 0.9)?;
        pause(stage, 0.6)?;
        kit.finish(stage)
    }
}
