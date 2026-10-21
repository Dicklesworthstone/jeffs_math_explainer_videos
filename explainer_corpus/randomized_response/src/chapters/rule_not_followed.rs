//! Chapter 09, when the rule isn't followed: everything so far assumed that
//! everyone lets the coins decide (09_1); the prediction, the four who
//! copied always say no (09_2), and its answer, minus two on average (09_3);
//! that is not scatter, the average itself moves, and more people do not fix
//! it (09_4); a negative estimate is a hint, not proof (09_5); RAPPOR's
//! designers on compliance (09_6).
//!
//! One pattern of noncompliance, stated as such
//! (`model::yes_count_distribution_copiers_say_no`, `R-F21`); other patterns
//! push either way or cancel, so nothing here says compliance is necessary,
//! only that the calculation assumes it. The bars are chapter 06's axis.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    coin, fade_in_all, fade_out_all, fail, flash_all, grow_lagged, lane, outline, pause, read_by, say, swap, think,
    wait_until,
};
use super::the_price_of_the_coin::{AXIS_Y, THINK_X, THINK_Y, axis_frame, bars_of, est_x, recolor_bars, truth_marker};
use crate::facts::{self, ASKED, Average, RAPPOR_COMPLIANCE};
use crate::model::{self, Channel, Q};
use crate::palette;

pub struct WhenTheRuleIsNotFollowed {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const ASSUMED: &str = "assumed: everyone lets the coins decide";
const ALWAYS_NO: &str = "always no";
const CHART_UP: &str = "estimated share, on average";
const CHART_ACROSS: &str = "people asked";

// ------------------------------------------------------------------ layout

/// The class as one row of small discs at the top, its words over it.
const ROW_X0: f64 = -4.8;
const ROW_DX: f64 = 0.22;
const ROW_Y: f64 = 3.05;
const DOT_R: f64 = 0.075;
const DOT_RING: f64 = 0.11;
const ROW_WORDS_Y: f64 = 3.45;
const ALWAYS_NO_DY: f64 = 0.3;
/// The broken rule's average on the axis: a line this tall, and its TeX.
const MEAN_H: f64 = 2.2;
const TEX_X: f64 = 2.6;
const TEX_Y: f64 = 2.55;
/// The small chart at the upper right: share −20% to 30% up, the three
/// sizes asked across.
const CHART_X0: f64 = 3.0;
const CHART_X1: f64 = 6.3;
const CHART_Y0: f64 = -0.05;
const CHART_Y1: f64 = 1.6;
const CHART_LOW: f64 = -20.0;
const CHART_HIGH: f64 = 30.0;
const CHART_WORDS_DY: f64 = 0.3;
/// The honest class's note, at the upper left.
const NOTE_X: f64 = -6.4;
const NOTE_Y: f64 = 2.45;

/// 09_4's exact version: the pattern assumed, and what it does.
fn pattern_card() -> String {
    "assumed: every copier says no and everyone else follows the rule; yes averages (n − k)/4 and the estimate \
     −k/2; as a share, −p/2 for any n"
        .to_owned()
}

/// 09_6's exact version: RAPPOR's own words.
fn compliance_card() -> String {
    format!(
        "RAPPOR (Erlingsson, Pihur and Korolova, CCS 2014), section 1: absolute compliance is '{RAPPOR_COMPLIANCE}'"
    )
}

/// Where a share in percentage points sits on the small chart.
fn chart_y(points: f64) -> f64 {
    CHART_Y0 + (CHART_Y1 - CHART_Y0) * (points - CHART_LOW) / (CHART_HIGH - CHART_LOW)
}

/// Where the i-th of the sizes asked sits on the small chart.
fn chart_x(i: usize) -> f64 {
    CHART_X0 + (CHART_X1 - CHART_X0) * (i as f64 + 0.5) / ASKED.len() as f64
}

impl SceneConstruct for WhenTheRuleIsNotFollowed {
    fn name(&self) -> &str {
        "09_when_the_rule_is_not_followed"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let a: Average = facts::average().map_err(fail)?;
        let (n, copiers) = (a.class.size(), a.class.copiers());
        let truth = Q::int(copiers as i128);
        let honest = model::yes_count_distribution(n, copiers).map_err(fail)?;
        let broken = model::yes_count_distribution_copiers_say_no(n, copiers).map_err(fail)?;
        let low = model::undo(0, n)[2];
        let rule = Channel::two_coins();
        let share = Q::new(copiers as i128, n as i128);

        // 09_1: the class; everything assumed everyone lets the coins decide
        let line = say(&kit, stage, "09_1")?;
        let mut class = Vec::with_capacity(n);
        let mut rings = Vec::with_capacity(copiers);
        for i in 0..n {
            let at = v(ROW_X0 + ROW_DX * i as f64, ROW_Y);
            class.push(coin(stage, palette::student(), at, DOT_R));
            if a.class.copied(i) {
                rings.push(outline(stage, palette::copied(), at, DOT_RING));
            }
        }
        let words = kit.text_left(stage, ASSUMED, 20.0, Palette::ink(), v(ROW_X0, ROW_WORDS_Y))?;
        let mut frame = axis_frame(&kit, stage, n)?;
        let mut honest_bars = bars_of(stage, &honest, Palette::dim())?;
        frame.extend(truth_marker(&kit, stage, copiers, low)?);
        let mut opening = class.clone();
        opening.extend(rings.iter().copied());
        opening.push(words);
        opening.extend(frame.iter().copied());
        opening.extend(honest_bars.iter().map(|&(_, m)| m));
        fade_in_all(stage, &opening, 0.5)?;
        line.wait_end(stage)?;

        // 09_2: the prediction: the four who copied always say no
        let line = say(&kit, stage, "09_2")?;
        line.wait_for(stage, "always say no")?;
        let mut locked = Vec::with_capacity(copiers);
        let mut unlocked = Vec::with_capacity(copiers);
        for i in (0..n).filter(|&i| a.class.copied(i)) {
            unlocked.push(class[i]);
            let fresh = coin(stage, palette::no(), v(ROW_X0 + ROW_DX * i as f64, ROW_Y), DOT_R);
            class[i] = fresh;
            locked.push(fresh);
        }
        let first = (0..n).find(|&i| a.class.copied(i)).unwrap_or(0);
        let always = kit.text_left(
            stage,
            ALWAYS_NO,
            18.0,
            palette::no(),
            v(ROW_X0 + ROW_DX * first as f64, ROW_Y - ALWAYS_NO_DY),
        )?;
        let mut coming = locked.clone();
        coming.push(always);
        swap(stage, &unlocked, &coming, 0.4)?;
        let q = think(&kit, stage, v(THINK_X, THINK_Y))?;

        // 09_3: minus two: the sixteen others give four yeses on average
        let line = say(&kit, stage, "09_3")?;
        fade_out_all(stage, &[q], 0.2)?;
        let others_yes = model::average_yes(rule, n - copiers, 0);
        let others_yes_whole = others_yes
            .integer()
            .and_then(|y| usize::try_from(y).ok())
            .ok_or_else(|| fail(format!("{others_yes} yeses on average is not whole")))?;
        let average = model::estimate_mean(&broken);
        if model::mean(&broken) != others_yes || average != model::undo(others_yes_whole, n)[2] {
            return Err(fail(format!(
                "the broken rule averages {average}, not the others' yeses undone"
            )));
        }
        // on average one in four of the others says yes (the rule's 1 of 4)
        let (mut old, mut new) = (Vec::new(), Vec::new());
        let mut seen = 0usize;
        for i in (0..n).filter(|&i| !a.class.copied(i)) {
            let said = seen.is_multiple_of(4);
            seen += 1;
            let fresh = coin(
                stage,
                palette::answer(said),
                v(ROW_X0 + ROW_DX * i as f64, ROW_Y),
                DOT_R,
            );
            old.push(class[i]);
            new.push(fresh);
            class[i] = fresh;
        }
        swap(stage, &old, &new, 0.4)?;
        let drawn_yes = (0..seen).filter(|k| k.is_multiple_of(4)).count();
        if drawn_yes != others_yes_whole {
            return Err(fail(format!("{drawn_yes} yeses drawn, {others_yes_whole} on average")));
        }
        line.wait_for(stage, "Double four")?;
        let factor = Q::ONE / rule.truth();
        let tex = kit.tex(
            stage,
            &format!(
                r"{factor} \times {others_yes} - {} = {average}",
                Q::int(n as i128) / factor
            ),
            34.0,
            Palette::ink(),
            &[],
            v(TEX_X, TEX_Y),
        )?;
        let mean_x = est_x(average, low);
        let mean_line = lane(
            stage,
            vec![v(mean_x, AXIS_Y), v(mean_x, AXIS_Y + MEAN_H)],
            palette::no(),
        );
        fade_in_all(stage, &[tex, mean_line], 0.4)?;

        // 09_4: not scatter: the average itself moves; more people don't fix it
        let line = say(&kit, stage, "09_4")?;
        line.wait_for(stage, "pulled down")?;
        let broken_bars = bars_of(stage, &broken, palette::no())?;
        let (gone, came) = recolor_bars(stage, &mut honest_bars, &honest, |_| true, Palette::faint())?;
        swap(stage, &gone, &came, 0.4)?;
        let rising: Vec<Mob> = broken_bars.iter().map(|&(_, m)| m).collect();
        grow_lagged(stage, &rising, DOWN, 1.2, 0.1)?;
        line.wait_for(stage, "asking more people")?;
        let share_mean =
            model::share_mean_copiers_say_no(rule, share).ok_or_else(|| fail("no share estimate".into()))?;
        let mut chart = vec![
            lane(
                stage,
                vec![v(CHART_X0, CHART_Y0), v(CHART_X0, CHART_Y1)],
                Palette::dim(),
            ),
            lane(
                stage,
                vec![v(CHART_X0, chart_y(0.0)), v(CHART_X1, chart_y(0.0))],
                Palette::dim(),
            ),
            lane(
                stage,
                vec![
                    v(CHART_X0, chart_y(100.0 * share.to_f64())),
                    v(CHART_X1, chart_y(100.0 * share.to_f64())),
                ],
                palette::copied(),
            ),
            kit.text_left(
                stage,
                CHART_UP,
                18.0,
                Palette::dim(),
                v(CHART_X0, CHART_Y1 + CHART_WORDS_DY),
            )?,
            kit.text(
                stage,
                CHART_ACROSS,
                18.0,
                Palette::dim(),
                v((CHART_X0 + CHART_X1) / 2.0, CHART_Y0 - 2.0 * CHART_WORDS_DY),
            )?,
        ];
        // the share's average is a person's, so the same however many are
        // asked (the oracle checks 20, 100 and 300 by their distributions)
        let y = chart_y(100.0 * share_mean.to_f64());
        for (i, &size) in ASKED.iter().enumerate() {
            chart.push(coin(stage, palette::no(), v(chart_x(i), y), 0.08));
            chart.push(kit.text(
                stage,
                &size.to_string(),
                18.0,
                Palette::dim(),
                v(chart_x(i), CHART_Y0 - CHART_WORDS_DY),
            )?);
        }
        let pattern_text = pattern_card();
        let pattern = precisely(&kit, stage, &pattern_text)?;
        chart.push(pattern);
        fade_in_all(stage, &chart, 0.5)?;
        let pattern_read = read_by(stage, &pattern_text);

        // 09_5: a negative estimate is a hint, not proof: one in eleven honest
        let line = say(&kit, stage, "09_5")?;
        line.wait_for(stage, "Our class")?;
        let negative = model::p_negative_estimate(&honest);
        let f = facts::facts().map_err(fail)?;
        let one_in = f
            .get("R-F14.one_in.negative")
            .ok_or_else(|| fail("no fact R-F14.one_in.negative".into()))?
            .to_owned();
        let mut old: Vec<Mob> = broken_bars.iter().map(|&(_, m)| m).collect();
        let mut new = Vec::new();
        // below zero red, the rest dim again
        for (below, color) in [(true, Palette::red()), (false, Palette::dim())] {
            let (o, c) = recolor_bars(stage, &mut honest_bars, &honest, |e| (e < Q::ZERO) == below, color)?;
            old.extend(o);
            new.extend(c);
        }
        new.push(kit.text_left(
            stage,
            &format!("rule followed: below 0 about 1 in {one_in} ({}%)", negative.percent()),
            20.0,
            Palette::red(),
            v(NOTE_X, NOTE_Y),
        )?);
        swap(stage, &old, &new, 0.5)?;

        // 09_6: RAPPOR's designers on compliance
        let line = say(&kit, stage, "09_6")?;
        let quote_text = compliance_card();
        let quote = precisely(&kit, stage, &quote_text)?;
        wait_until(stage, pattern_read)?;
        swap(stage, &[pattern], &[quote], 0.4)?;
        let quote_read = read_by(stage, &quote_text);
        line.wait_end(stage)?;
        flash_all(stage, &[quote], 0.4)?;
        pause(stage, 1.0)?;
        wait_until(stage, quote_read)?;

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
            ("09_2", "always say no"),
            ("09_3", "Double four"),
            ("09_4", "pulled down"),
            ("09_4", "asking more people"),
            ("09_5", "Our class"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The numbers and words on screen are the facts': four yeses from the
    /// sixteen others, an average of −2, a share of −1/10 for every size
    /// asked; RAPPOR's words verbatim.
    #[test]
    fn the_numbers_and_quote_are_the_facts() {
        let f = facts::facts().unwrap();
        let broken = model::yes_count_distribution_copiers_say_no(20, 4).unwrap();
        assert_eq!(
            f.get("R-F21.average_yes"),
            Some(model::mean(&broken).to_string().as_str())
        );
        assert_eq!(
            f.get("R-F21.average_estimate"),
            Some(model::estimate_mean(&broken).to_string().as_str())
        );
        assert_eq!(model::average_yes(Channel::two_coins(), 16, 0), model::mean(&broken));
        assert_eq!(f.get("R-F21.share_mean"), Some("-1/10"));
        assert_eq!(f.get("R-F21.asked"), Some("20,100,300"));
        assert_eq!(
            f.get("R-X6.rappor.compliance"),
            Some("an assumption that may not hold for human subjects")
        );
        assert!(compliance_card().ends_with("'an assumption that may not hold for human subjects'"));
        // one in four of the sixteen others says yes on average
        assert_eq!((0..16usize).filter(|k| k.is_multiple_of(4)).count(), 4);
    }

    /// The cards stay two lines at every wrap the lower-third can narrow to.
    #[test]
    fn the_cards_are_two_lines() {
        for text in [pattern_card(), compliance_card()] {
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
        // the row inside the frame, its words over it, inside the top
        const {
            assert!(
                ROW_X0 - DOT_RING > -FRAME_X_RADIUS + 0.25,
                "the row inside the left edge"
            )
        };
        const { assert!(ROW_WORDS_Y + 0.15 < FRAME_Y_RADIUS - 0.25, "the words inside the top") };
        const { assert!(ROW_WORDS_Y - 0.15 > ROW_Y + DOT_RING, "the words over the row") };
        let last = ROW_X0 + ROW_DX * 19.0;
        assert!(last + DOT_RING < TEX_X - 1.6);
        // the broken rule's tallest bar under the note
        use crate::chapters::the_price_of_the_coin::BAR_SCALE;
        let broken = model::yes_count_distribution_copiers_say_no(20, 4).unwrap();
        let tallest = broken.iter().map(|p| p.to_f64()).fold(0.0, f64::max) * BAR_SCALE;
        assert!(AXIS_Y + tallest < NOTE_Y - 0.2, "{tallest}");
        // the mean's line under the note and the row's "always no"
        const { assert!(AXIS_Y + MEAN_H < NOTE_Y - 0.2, "the mean's line under the note") };
        // the small chart at the upper right, under the TeX, over the axis
        const {
            assert!(
                CHART_X1 + 0.3 < FRAME_X_RADIUS - 0.25,
                "the chart inside the right edge"
            )
        };
        const {
            assert!(
                CHART_Y1 + CHART_WORDS_DY + 0.12 < TEX_Y - 0.25,
                "the chart under the TeX"
            )
        };
        const {
            assert!(
                CHART_Y0 - 2.0 * CHART_WORDS_DY - 0.12 > AXIS_Y + 0.3,
                "the chart over the axis"
            )
        };
        const { assert!(TEX_X + 1.6 < FRAME_X_RADIUS - 0.25, "the TeX inside the right edge") };
        const { assert!(CHART_HIGH > 20.0 && CHART_LOW < -10.0, "the chart shows +20% and -10%") };
        const { assert!(THINK_Y - 0.62 - 0.45 > BAND_TOP, "the think note over the band") };
        const { assert!(NOTE_Y - 0.15 > AXIS_Y + MEAN_H, "the note over the mean's line") };
    }
}
