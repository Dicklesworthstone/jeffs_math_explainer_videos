//! Chapter 04, any class: a class of n with k who copied (04_1); on average
//! half the copiers say yes by the truth and the second coin says yes for a
//! quarter of the class (04_2), k/2 + n/4, for us seven (04_3); run
//! backwards, the average gives back k exactly (04_4); but one survey's
//! count gives an estimate, right on average and able to miss (04_5); the
//! prediction, a class of forty with eighteen yeses (04_6), and its answer,
//! sixteen, an estimate, not a count (04_7); what each step undoes (04_8).
//!
//! The chart is chapter 01's: the rule's line is `model::average_yes` at the
//! two coins. The real class is chapter 06's seeded draw `facts::SHOWN_RUN`,
//! its estimate `model::undo`'s; the faint bars along the yes axis are the
//! class's exact distribution (`model::yes_count_distribution`); the class
//! of forty is `model::undo` on `facts::FORTY`.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, rect, signed_thousands, v};
use fmn::prelude::*;

use super::draw::{
    YesChart, bar, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, flat, lane, pause, read_by, say, think,
    wait_until,
};
use crate::facts::{self, Average, FORTY, SHOWN_RUN};
use crate::model::{self, Channel, Q, Tally};
use crate::palette;

pub struct AnyClass {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const ACROSS: &str = "k who copied, 0 to n";
const UP: &str = "yes answers, on average";
const TRUTH_PART: &str = "k/2";
const COIN_PART: &str = "n/4";
const Y_NAME: &str = "y: yes answers on average";
const NOT_A_COUNT: &str = "an estimate, not a count";
const ESTIMATE_Q: &str = "estimate = ?";
const UNDOES_FIRST: &str = "2 ×: undoes the first coin";
const SECOND_YESES: &str = "n/4: the second coin's yeses";

// ------------------------------------------------------------------ layout

/// The chart, where chapter 01 drew it.
const CHART: YesChart = YesChart {
    n: model::CLASS_SIZE,
    x0: 0.9,
    y0: -1.9,
    w: 5.2,
    h: 4.2,
};
/// The stacked bar's width, and its words' offset to its right.
const STACK_W: f64 = 0.32;
const STACK_LABEL_DX: f64 = 0.45;
/// The left column: the formula, its reading, the inverse, then the class
/// of forty's working and its closing words; a 40-point fraction reaches
/// about 0.41 above and below its centre.
const LEFT_X: f64 = -3.4;
const FORMULA_Y: f64 = 2.75;
const NAME_Y: f64 = 2.1;
const SUM_Y: f64 = 1.6;
const INVERSE_Y: f64 = 0.95;
const FRACTION_HALF: f64 = 0.41;
const STEP_Y: [f64; 4] = [0.15, -0.4, -0.95, -1.5];
const NOT_COUNT_Y: f64 = -2.05;
const WORDS_Y: [f64; 2] = [0.25, -0.2];
/// The real class's words, at the left.
const SHOWN_Y: f64 = 0.15;
/// The faint distribution along the yes axis: bar length per unit of chance.
const CLOUD_SCALE: f64 = 2.0;
const CLOUD_H: f64 = 0.08;
/// The think time, right of the left column.
const THINK_X: f64 = -0.9;
const THINK_Y: f64 = -0.6;

/// 04_3's exact version: the average and its inverse.
fn average_card() -> String {
    "on average yes = k/2 + n/4 (Dwork and Roth, section 2.3: share 1/4 + p/2), so k = 2 × (yes − n/4)".to_owned()
}

/// 04_5's exact version: what the estimate does, and does not, promise.
fn estimate_card() -> String {
    "estimate = 2 × yes − n/2: over every coin outcome it averages k, for every k; one survey's estimate can miss"
        .to_owned()
}

/// The real class's words: its yeses and estimate.
fn shown_note(yes: usize, estimate: Q) -> fmn::Result<String> {
    let e = estimate
        .integer()
        .map(signed_thousands)
        .ok_or_else(|| fail(format!("{estimate} is not a whole number")))?;
    Ok(format!("one real class: {yes} yes, estimate {e}"))
}

impl SceneConstruct for AnyClass {
    fn name(&self) -> &str {
        "04_any_class"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let a: Average = facts::average().map_err(fail)?;
        let (n, copiers) = (a.class.size(), a.class.copiers());
        let (nq, kq) = (Q::int(n as i128), Q::int(copiers as i128));
        let rule = Channel::two_coins();
        let [forced, left, estimate] = facts::undo(&a).map_err(fail)?;
        let truth_part = estimate - left;
        let average = model::average_yes(rule, n, copiers);
        if truth_part + forced != average {
            return Err(fail(format!("{truth_part} + {forced} is not the average {average}")));
        }

        // 04_1: the chart again, a class of n with k who copied
        let line = say(&kit, stage, "04_1")?;
        let mut opening = CHART.axes(&kit, stage, ACROSS, UP)?;
        let direct = Channel::new(Q::ONE).map_err(fail)?;
        let alone = Channel::new(Q::ZERO).map_err(fail)?;
        for c in [direct, alone] {
            opening.push(CHART.line(
                stage,
                (Q::ZERO, model::average_yes(c, n, 0)),
                (nq, model::average_yes(c, n, n)),
                Palette::faint(),
            ));
        }
        fade_in_all(stage, &opening, 0.5)?;
        line.wait_end(stage)?;

        // 04_2: half the copiers by the truth; a quarter of the class by the coin
        let line = say(&kit, stage, "04_2")?;
        let base = CHART.p(kq, Q::ZERO);
        let truth_top = CHART.p(kq, truth_part);
        let green = bar(
            stage,
            palette::truth(),
            base[0],
            base[1],
            STACK_W,
            truth_top[1] - base[1],
        )?;
        let green_label = kit.text_left(
            stage,
            TRUTH_PART,
            20.0,
            palette::truth(),
            v(base[0] + STACK_LABEL_DX, (base[1] + truth_top[1]) / 2.0),
        )?;
        fade_in_all(stage, &[green, green_label], 0.5)?;
        line.wait_for(stage, "the second coin")?;
        let top = CHART.p(kq, average);
        let dim = bar(
            stage,
            palette::forced(),
            base[0],
            truth_top[1],
            STACK_W,
            top[1] - truth_top[1],
        )?;
        let dim_label = kit.text_left(
            stage,
            COIN_PART,
            20.0,
            palette::forced(),
            v(base[0] + STACK_LABEL_DX, (truth_top[1] + top[1]) / 2.0),
        )?;
        fade_in_all(stage, &[dim, dim_label], 0.5)?;

        // 04_3: k/2 + n/4; the rule's line; for us 2 + 5 = 7
        let line = say(&kit, stage, "04_3")?;
        let formula = kit.tex(
            stage,
            r"y = \frac{k}{2} + \frac{n}{4}",
            40.0,
            Palette::ink(),
            &[],
            v(LEFT_X, FORMULA_Y),
        )?;
        let y_name = kit.text(stage, Y_NAME, 20.0, Palette::dim(), v(LEFT_X, NAME_Y))?;
        let rule_line = CHART.line(
            stage,
            (Q::ZERO, model::average_yes(rule, n, 0)),
            (nq, model::average_yes(rule, n, n)),
            palette::truth(),
        );
        let card_text = average_card();
        let card = precisely(&kit, stage, &card_text)?;
        fade_in_all(stage, &[formula, y_name, rule_line, card], 0.5)?;
        let card_read = read_by(stage, &card_text);
        line.wait_for(stage, "two plus five")?;
        let sum = kit.tex(
            stage,
            &format!("{truth_part} + {forced} = {average}"),
            34.0,
            Palette::ink(),
            &[],
            v(LEFT_X, SUM_Y),
        )?;
        let point = CHART.point(stage, kq, average, palette::copied());
        fade_in_all(stage, &[sum, point], 0.4)?;
        flash_all(stage, &[point], 0.5)?;

        // 04_4: run backwards: the average gives back k exactly
        let line = say(&kit, stage, "04_4")?;
        let back = lane(
            stage,
            vec![CHART.p(Q::ZERO, average), CHART.p(kq, average), CHART.p(kq, Q::ZERO)],
            palette::copied(),
        );
        let inverse = kit.tex(
            stage,
            r"k = 2 \times (y - \frac{n}{4})",
            40.0,
            Palette::ink(),
            &[],
            v(LEFT_X, INVERSE_Y),
        )?;
        fade_in_all(stage, &[back, inverse], 0.5)?;
        if (average - forced) / rule.truth() != kq {
            return Err(fail("running the average backwards does not give k".into()));
        }
        line.wait_end(stage)?;
        // the arrow stands a moment (a named hold)
        pause(stage, 1.0)?;

        // 04_5: one survey's count: an estimate, right on average, able to miss
        let line = say(&kit, stage, "04_5")?;
        let runs = facts::runs().map_err(fail)?;
        let drawn = runs
            .get(SHOWN_RUN)
            .ok_or_else(|| fail(format!("no seeded class {SHOWN_RUN}")))?;
        let shown_yes = Tally::of(&a.class, drawn).map_err(fail)?.answers(true);
        let shown_estimate = model::undo(shown_yes, n)[2];
        wait_until(stage, card_read)?;
        fade_out_all(stage, &[card], 0.3)?;
        line.wait_for(stage, "a single count")?;
        let yes_q = Q::int(shown_yes as i128);
        let shown_arrow = lane(
            stage,
            vec![
                CHART.p(Q::ZERO, yes_q),
                CHART.p(shown_estimate, yes_q),
                CHART.p(shown_estimate, Q::ZERO),
            ],
            Palette::ink(),
        );
        let shown_words = kit.text(
            stage,
            &shown_note(shown_yes, shown_estimate)?,
            20.0,
            Palette::ink(),
            v(LEFT_X, SHOWN_Y),
        )?;
        fade_in_all(stage, &[shown_arrow, shown_words], 0.4)?;
        line.wait_for(stage, "right on average")?;
        let dist = model::yes_count_distribution(n, copiers).map_err(fail)?;
        let mut cloud = Vec::with_capacity(dist.len());
        for (y, &p) in dist.iter().enumerate() {
            let len = CLOUD_SCALE * p.to_f64();
            if len < 0.01 {
                continue;
            }
            let at = CHART.p(Q::ZERO, Q::int(y as i128));
            let r = rect(
                v(at[0], at[1] - CLOUD_H / 2.0),
                len,
                CLOUD_H,
                flat(Palette::faint(), 0.9),
            )?;
            cloud.push(stage.arena_mut().add(r));
        }
        let estimate_text = estimate_card();
        let estimate_note = precisely(&kit, stage, &estimate_text)?;
        fade_in_lagged(stage, &cloud, 0.8, 0.05)?;
        fade_in_all(stage, &[estimate_note], 0.4)?;
        let estimate_read = read_by(stage, &estimate_text);

        // 04_6: the prediction: a class of forty, eighteen yeses
        let line = say(&kit, stage, "04_6")?;
        let (forty, forty_yes) = FORTY;
        fade_out_all(stage, &[shown_arrow, shown_words], 0.3)?;
        line.wait_for(stage, "What does")?;
        let given = [
            kit.tex(
                stage,
                &format!(r"n = {forty},\quad y = {forty_yes}"),
                34.0,
                Palette::ink(),
                &[],
                v(LEFT_X, STEP_Y[0]),
            )?,
            kit.text(stage, ESTIMATE_Q, 22.0, Palette::dim(), v(LEFT_X, STEP_Y[1]))?,
        ];
        fade_in_all(stage, &given, 0.3)?;
        let q = think(&kit, stage, v(THINK_X, THINK_Y))?;

        // 04_7: sixteen: a quarter of forty is ten, eight left, doubled
        let line = say(&kit, stage, "04_7")?;
        let [forty_forced, forty_left, forty_estimate] = model::undo(forty_yes, forty);
        wait_until(stage, estimate_read)?;
        fade_out_all(stage, &[q, given[1], estimate_note], 0.2)?;
        let quarter = Q::int(forty as i128) / forty_forced;
        let steps = [
            (
                "A quarter of forty",
                format!(r"\tfrac{{{forty}}}{{{quarter}}} = {forty_forced}"),
            ),
            (
                "Eighteen take away",
                format!("{forty_yes} - {forty_forced} = {forty_left}"),
            ),
            ("double eight", format!(r"2 \times {forty_left} = {forty_estimate}")),
        ];
        let mut working = Vec::with_capacity(steps.len() + 1);
        for ((phrase, src), &y) in steps.iter().zip(&STEP_Y[1..]) {
            line.wait_for(stage, phrase)?;
            let m = kit.tex(stage, src, 34.0, Palette::ink(), &[], v(LEFT_X, y))?;
            fade_in_all(stage, &[m], 0.3)?;
            working.push(m);
        }
        line.wait_for(stage, "An estimate")?;
        let not_count = kit.text(stage, NOT_A_COUNT, 22.0, Palette::dim(), v(LEFT_X, NOT_COUNT_Y))?;
        fade_in_all(stage, &[not_count], 0.3)?;
        working.push(not_count);

        // 04_8: the doubling undoes the first coin; the quarter is the coin's
        let line = say(&kit, stage, "04_8")?;
        let mut leaving = working;
        leaving.push(given[0]);
        fade_out_all(stage, &leaving, 0.3)?;
        let first = kit.text(stage, UNDOES_FIRST, 22.0, palette::truth(), v(LEFT_X, WORDS_Y[0]))?;
        fade_in_all(stage, &[first], 0.3)?;
        line.wait_for(stage, "The quarter")?;
        let second = kit.text(stage, SECOND_YESES, 22.0, palette::forced(), v(LEFT_X, WORDS_Y[1]))?;
        fade_in_all(stage, &[second], 0.3)?;
        flash_all(stage, &[inverse], 0.5)?;
        line.wait_end(stage)?;
        pause(stage, 1.0)?;

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
            ("04_2", "the second coin"),
            ("04_3", "two plus five"),
            ("04_5", "a single count"),
            ("04_5", "right on average"),
            ("04_6", "What does"),
            ("04_7", "A quarter of forty"),
            ("04_7", "Eighteen take away"),
            ("04_7", "double eight"),
            ("04_7", "An estimate"),
            ("04_8", "The quarter"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The numbers on screen are the facts': 2 + 5 = 7 for us, the shown
    /// class's 9 yeses and estimate 8 (a miss inside 0 to n), and the class
    /// of forty's 10, 8 and 16.
    #[test]
    fn the_numbers_are_the_facts() {
        let f = facts::facts().unwrap();
        let a = facts::average().unwrap();
        let [forced, left, estimate] = facts::undo(&a).unwrap();
        assert_eq!((estimate - left, forced), (Q::int(2), Q::int(5)));
        assert_eq!(f.get("R-F17.rule.average_yes"), Some("7"));
        let runs = facts::runs().unwrap();
        let yes = Tally::of(&a.class, &runs[SHOWN_RUN]).unwrap().answers(true);
        assert_eq!(f.get("R-F18.shown.yes"), Some(yes.to_string().as_str()));
        let e = model::undo(yes, 20)[2];
        assert_eq!(f.get("R-F18.shown.estimate"), Some(e.to_string().as_str()));
        assert_eq!(shown_note(yes, e).unwrap(), "one real class: 9 yes, estimate 8");
        let (forty, forty_yes) = FORTY;
        assert_eq!(model::undo(forty_yes, forty), [Q::int(10), Q::int(8), Q::int(16)]);
        assert_eq!(f.get("R-F18.estimate"), Some("16"));
        assert!(average_card().contains("Dwork and Roth, section 2.3"));
        assert_eq!(f.get("R-F18.unbiased_every_k"), Some("true"));
    }

    /// The cards stay two lines at every wrap the lower-third can narrow to.
    #[test]
    fn the_cards_are_two_lines() {
        for text in [average_card(), estimate_card()] {
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
        // the left column left of the chart and its tick values, inside the frame
        assert!(LEFT_X + 2.4 < CHART.x0 - 0.3 - 0.3);
        assert!(LEFT_X - 2.6 > -FRAME_X_RADIUS + 0.25);
        const {
            assert!(
                FORMULA_Y + FRACTION_HALF < FRAME_Y_RADIUS - 0.25,
                "the formula inside the top"
            )
        };
        // its rows apart, top to bottom, the last over the band
        let rows = [
            FORMULA_Y,
            NAME_Y,
            SUM_Y,
            INVERSE_Y,
            STEP_Y[0],
            STEP_Y[1],
            STEP_Y[2],
            STEP_Y[3],
            NOT_COUNT_Y,
        ];
        for pair in rows.windows(2) {
            assert!(pair[0] - pair[1] >= 0.45, "{pair:?}");
        }
        // the fractions clear the rows beside them
        const {
            assert!(
                FORMULA_Y - FRACTION_HALF > NAME_Y + 0.12,
                "the formula over its reading"
            )
        };
        const { assert!(INVERSE_Y + FRACTION_HALF < SUM_Y - 0.15, "the inverse under the sum") };
        const {
            assert!(
                INVERSE_Y - FRACTION_HALF > STEP_Y[0] + 0.2,
                "the inverse over the working"
            )
        };
        const { assert!(NOT_COUNT_Y - 0.15 > BAND_TOP + 0.1, "the closing words over the band") };
        const {
            assert!(
                WORDS_Y[0] + 0.15 < INVERSE_Y - FRACTION_HALF,
                "the closing words under the inverse"
            )
        };
        // the think ring between the left column's steps and the chart
        assert!(THINK_X + 0.62 < CHART.x0 - 0.3 - 0.2);
        const { assert!(THINK_Y - 0.62 - 0.45 > BAND_TOP, "the think note over the band") };
        // the stacked bar's words inside the chart, right of the bar
        let base = CHART.p(Q::int(4), Q::ZERO);
        assert!(base[0] + STACK_LABEL_DX + 0.5 < CHART.x0 + CHART.w);
        assert!(STACK_LABEL_DX > STACK_W / 2.0 + 0.05);
        // the cloud's longest bar stays short of the rule's line
        let dist = model::yes_count_distribution(20, 4).unwrap();
        let longest = dist.iter().map(|p| p.to_f64()).fold(0.0, f64::max) * CLOUD_SCALE;
        assert!(longest < 0.6, "{longest}");
    }
}
