//! Chapter 13, running one: the whole survey for programmers, step by step
//! (13_1 to 13_3); its typical miss, the square root of three quarters of
//! the class, the same whoever copied, so it can be stated in advance
//! (13_4); the prediction, what the private coins and the kept negatives
//! protect (13_5), and its answer (13_6); back to the opening question
//! (13_7).
//!
//! The steps are the model's (`model::undo`, `Channel::coin_variance`); the
//! misses are a fixed class's, the square root of n times one person's
//! coins' variance (`R-F23`), checked equal for every number of copiers by
//! the facts. The ± display says what it is: a typical miss, not a limit
//! (root's clarification 2), never a confidence interval.

use explainer_kit::props::{lower_third, precisely};
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    disc, fade_in_all, fade_out_all, fail, flash_all, lane, pause, read_by, say, swap, think, wait_until,
};
use super::the_coin::{QUESTION, QUESTION_Y, seat_at};
use super::the_price_of_the_coin::miss_text;
use crate::facts::{self, Average, BIG_CLASS};
use crate::model::{self, Channel, Q};
use crate::palette;

pub struct RunningOne {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const PROGRAMMERS: &str = "for programmers:";
const STEPS: [&str; 4] = [
    "1. publish the rule and the recipe",
    "2. each person: two private fair coins, one yes or no, once",
    "3. estimate = 2 × (yes − n/4), negatives kept",
    "4. report it with its typical miss: typical, not a limit",
];
const SAME: &str = "the same whoever copied";
const GUARDS: [&str; 3] = [
    "private coins: each answer at most 3 times as likely either way",
    "negatives kept: right on average",
    "once each: no multiplying",
];
/// The step each guard comes from: the coins and "once" are step 2's, the
/// kept negatives step 3's.
const GUARD_STEP: [usize; 3] = [1, 2, 1];
const TYPICAL: &str = "the estimate and its typical miss: typical, not a limit";

// ------------------------------------------------------------------ layout

/// The steps at the upper left, one under another.
const STEP_X: f64 = -6.4;
const STEP_Y0: f64 = 3.0;
const STEP_DY: f64 = 0.55;
/// The typical miss at the right: its formula, its table, its words.
const MISS_X: f64 = 3.6;
const MISS_Y: [f64; 4] = [0.95, 0.35, -0.15, -0.6];
/// The guards, under the steps; the think time at the right.
const GUARD_X: f64 = -6.4;
const GUARD_Y: [f64; 3] = [0.6, 0.1, -0.4];
const THINK_X: f64 = 4.6;
const THINK_Y: f64 = -1.55;
/// The arrows from steps 2 to 4 to their guards start this far right of
/// the steps' left edge.
const ARROW_X: f64 = -6.6;
/// The closing picture: the class back on the left, the estimate beside it.
const ESTIMATE_X: f64 = 0.6;
const ESTIMATE_Y: [f64; 2] = [0.45, -0.25];

/// 13_1's code card: the whole recipe.
fn recipe_card() -> String {
    "two private fair coins each; answer = the truth if coin 1 is heads, else coin 2's face; estimate = 2 * yes - n \
     / 2, negatives kept"
        .to_owned()
}

/// 13_4's exact version: what the typical miss is, and what it assumes.
fn miss_card() -> String {
    "fixed class, rule followed, once each: typical miss = the square root of 3n/4, any k, not a limit; sampling adds \
     n × p(1 − p) to the variance"
        .to_owned()
}

/// 13_6's exact version: what each step guards.
fn guards_card() -> String {
    "private coins: at most 3 times as likely either way; negatives kept: right on average; one answer each: no \
     multiplying"
        .to_owned()
}

/// A fixed class of `n`'s typical miss: the square root of n times one
/// person's coins' variance.
fn typical_miss(n: usize) -> fmn::Result<f64> {
    let per = Channel::two_coins()
        .coin_variance()
        .ok_or_else(|| fail("the two coins have no variance".into()))?;
    Ok((Q::int(n as i128) * per).to_f64().sqrt())
}

impl SceneConstruct for RunningOne {
    fn name(&self) -> &str {
        "13_running_one"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let a: Average = facts::average().map_err(fail)?;
        let n = a.class.size();
        let mut steps = Vec::with_capacity(STEPS.len());
        for (i, s) in STEPS.iter().enumerate() {
            steps.push(kit.text_left(stage, s, 22.0, Palette::ink(), v(STEP_X, STEP_Y0 - STEP_DY * i as f64))?);
        }

        // 13_1: for programmers: publish the rule and the recipe
        let _line = say(&kit, stage, "13_1")?;
        let recipe_text = recipe_card();
        let recipe = lower_third(&kit, stage, PROGRAMMERS, &recipe_text)?;
        fade_in_all(stage, &[steps[0], recipe], 0.4)?;
        let recipe_read = read_by(stage, &recipe_text);

        // 13_2: each person flips privately and answers once
        let _line = say(&kit, stage, "13_2")?;
        fade_in_all(stage, &[steps[1]], 0.4)?;

        // 13_3: the recipe, negatives kept
        let line = say(&kit, stage, "13_3")?;
        fade_in_all(stage, &[steps[2]], 0.4)?;
        // the recipe is chapter 03's and 04's: undo the coins, then double
        let [forced, left, estimate] = model::undo(a.tally.answers(true), n);
        if (left + forced) * Q::int(2) - Q::int(n as i128) / Q::int(2) != estimate {
            return Err(fail("the recipe on screen is not the model's".into()));
        }
        line.wait_end(stage)?;

        // 13_4: the typical miss, the same whoever copied, stated in advance
        let line = say(&kit, stage, "13_4")?;
        fade_in_all(stage, &[steps[3]], 0.4)?;
        let (small, big) = (typical_miss(n)?, typical_miss(BIG_CLASS)?);
        let miss = [
            kit.tex(
                stage,
                r"\sqrt{\frac{3n}{4}}",
                36.0,
                Palette::ink(),
                &[],
                v(MISS_X, MISS_Y[0]),
            )?,
            kit.text(
                stage,
                &format!("n = {n}: {}", miss_text(small)),
                20.0,
                Palette::ink(),
                v(MISS_X, MISS_Y[1]),
            )?,
            kit.text(
                stage,
                &format!("n = {BIG_CLASS}: {}", miss_text(big)),
                20.0,
                Palette::ink(),
                v(MISS_X, MISS_Y[2]),
            )?,
        ];
        fade_in_all(stage, &miss, 0.4)?;
        line.wait_for(stage, "doesn't depend")?;
        let same = kit.text(stage, SAME, 20.0, Palette::dim(), v(MISS_X, MISS_Y[3]))?;
        fade_in_all(stage, &[same], 0.3)?;
        let miss_text_card = miss_card();
        let miss_note = precisely(&kit, stage, &miss_text_card)?;
        wait_until(stage, recipe_read)?;
        swap(stage, &[recipe], &[miss_note], 0.4)?;
        let miss_read = read_by(stage, &miss_text_card);

        // 13_5: the prediction: what do the coins and the negatives protect?
        let _line = say(&kit, stage, "13_5")?;
        flash_all(stage, &[steps[1], steps[2]], 0.5)?;
        let q = think(&kit, stage, v(THINK_X, THINK_Y))?;

        // 13_6: each step's guard, one at a time
        let line = say(&kit, stage, "13_6")?;
        fade_out_all(stage, &[q], 0.2)?;
        let mut guards = Vec::with_capacity(GUARDS.len());
        for (i, (phrase, words)) in [
            ("The private coins", GUARDS[0]),
            ("Keeping the negatives", GUARDS[1]),
            ("Answering once", GUARDS[2]),
        ]
        .into_iter()
        .enumerate()
        {
            line.wait_for(stage, phrase)?;
            let step_y = STEP_Y0 - STEP_DY * GUARD_STEP[i] as f64;
            let arrow = lane(
                stage,
                vec![v(ARROW_X, step_y), v(ARROW_X, GUARD_Y[i]), v(GUARD_X - 0.1, GUARD_Y[i])],
                Palette::dim(),
            );
            let guard = kit.text_left(stage, words, 20.0, palette::truth(), v(GUARD_X, GUARD_Y[i]))?;
            fade_in_all(stage, &[arrow, guard], 0.3)?;
            guards.extend([arrow, guard]);
        }
        let guards_text = guards_card();
        let guards_note = precisely(&kit, stage, &guards_text)?;
        line.wait_end(stage)?;
        // the three guards stand together a moment (a named hold)
        pause(stage, 1.0)?;
        wait_until(stage, miss_read)?;
        swap(stage, &[miss_note], &[guards_note], 0.4)?;
        let guards_read = read_by(stage, &guards_text);

        // 13_7: back to the opening: no answer is a confession; the count is estimated
        let line = say(&kit, stage, "13_7")?;
        let mut leaving = steps.clone();
        leaving.extend(guards);
        leaving.extend(miss);
        leaving.push(same);
        fade_out_all(stage, &leaving, 0.4)?;
        let question = kit.text(stage, QUESTION, 34.0, palette::student(), v(0.0, QUESTION_Y))?;
        let mut class = vec![question];
        for (i, &(first, second)) in a.coins.iter().enumerate() {
            let said = model::answer(a.class.copied(i), first, second);
            class.push(disc(stage, palette::answer(said), seat_at(i)));
        }
        fade_in_all(stage, &class, 0.5)?;
        line.wait_for(stage, "the count")?;
        let shown = [
            kit.tex(
                stage,
                &format!(r"{estimate} \pm {:.2}", small),
                40.0,
                Palette::ink(),
                &[],
                v(ESTIMATE_X, ESTIMATE_Y[0]),
            )?,
            kit.text(stage, TYPICAL, 20.0, Palette::dim(), v(ESTIMATE_X, ESTIMATE_Y[1]))?,
        ];
        fade_in_all(stage, &shown, 0.4)?;
        line.wait_end(stage)?;
        pause(stage, 2.0)?;
        wait_until(stage, guards_read)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::*;

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("13_4", "doesn't depend"),
            ("13_6", "The private coins"),
            ("13_6", "Keeping the negatives"),
            ("13_6", "Answering once"),
            ("13_7", "the count"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The numbers on screen are the facts': 3.87 for the class of twenty,
    /// 15 for three hundred, the variance 3n/4 checked for every k; the
    /// opening class's estimate is chapter 03's four.
    #[test]
    fn the_misses_are_the_facts() {
        let f = facts::facts().unwrap();
        assert_eq!(miss_text(typical_miss(20).unwrap()), "3.87");
        assert_eq!(f.get("R-F7.sd_estimate"), Some("3.87"));
        assert_eq!(miss_text(typical_miss(BIG_CLASS).unwrap()), "15");
        assert_eq!(f.get("R-F19.big.miss"), Some("15"));
        assert_eq!(f.get("R-F23.per_person"), Some("3/4"));
        assert_eq!(f.get("R-F4.estimate"), Some("4"));
        assert!(STEPS[3].ends_with("typical, not a limit"));
        assert!(TYPICAL.ends_with("typical, not a limit"));
        assert!(miss_card().contains("not a limit"));
    }

    /// The cards stay two lines at every wrap the lower-third can narrow to.
    #[test]
    fn the_cards_are_two_lines() {
        for text in [recipe_card(), miss_card(), guards_card()] {
            for width in (72..=92).rev().step_by(4) {
                let lines = explainer_kit::animatic::wrap(&text, width, 3);
                assert!(lines.len() <= 2, "{} lines at {width}: {text}", lines.len());
            }
        }
    }

    /// The layout keeps its pieces apart and inside the frame.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        const BAND_TOP: f64 = -FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36 * 2.0;
        // the steps inside the top-left edges; the longest (about 61
        // characters at 22 points, 6.9 wide) left of the miss's column
        const { assert!(STEP_X > -FRAME_X_RADIUS + 0.25, "the steps inside the left edge") };
        const { assert!(STEP_Y0 + 0.15 < FRAME_Y_RADIUS - 0.25, "the steps inside the top") };
        const { assert!(STEP_X + 6.9 < FRAME_X_RADIUS - 0.25, "the steps inside the right edge") };
        // the miss's column right of the steps, over the band, inside the right edge
        const { assert!(STEP_X + 6.9 < MISS_X - 1.0, "the miss's column right of the steps") };
        const { assert!(MISS_X + 1.6 < FRAME_X_RADIUS - 0.25, "the miss inside the right edge") };
        const { assert!(MISS_Y[3] - 0.15 > BAND_TOP + 0.2, "the miss's words over the band") };
        // the guards under the steps, left of the miss's column, over the band;
        // the longest (about 63 characters at 20 points, 6.6 wide) clear of it
        const {
            assert!(
                GUARD_Y[0] + 0.15 < STEP_Y0 - STEP_DY * 3.0 - 0.15,
                "the guards under the steps"
            )
        };
        const { assert!(GUARD_Y[2] - 0.15 > BAND_TOP + 0.2, "the guards over the band") };
        const { assert!(GUARD_X + 6.6 < MISS_X - 1.4, "the guards left of the miss's table") };
        const { assert!(ARROW_X > -FRAME_X_RADIUS + 0.25, "the arrows inside the left edge") };
        // the think ring under the miss's words, over the band
        const {
            assert!(
                THINK_Y + 0.62 < MISS_Y[3] - 0.15,
                "the think ring under the miss's words"
            )
        };
        const { assert!(GUARD_STEP.len() == GUARDS.len(), "a step for every guard") };
        const { assert!(THINK_Y - 0.62 - 0.45 > BAND_TOP, "the think note over the band") };
        // the closing estimate right of the class, under the question
        let last = seat_at(model::CLASS_SIZE - 1);
        assert!(ESTIMATE_X - 2.6 > last[0] + 0.3);
        const {
            assert!(
                ESTIMATE_Y[0] + 0.3 < QUESTION_Y - 0.3,
                "the estimate under the question"
            )
        };
        const { assert!(ESTIMATE_Y[1] - 0.15 > BAND_TOP + 0.2, "its words over the band") };
    }
}
