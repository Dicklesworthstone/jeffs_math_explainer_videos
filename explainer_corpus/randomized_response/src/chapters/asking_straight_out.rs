//! Chapter 01, asking straight out: a direct question to a class of twenty
//! (01_1); only we can see who copied (01_2); if everyone answers honestly,
//! the survey gets its count but every yes is a confession (01_3); whether
//! people answer honestly is a separate question (01_4); the opposite, a
//! coin alone (01_5), and the prediction, what its yeses say about how many
//! copied (01_6): nothing, ten on average whoever copied (01_7); the two
//! failures side by side (01_8); the rule we want sits in between (01_9,
//! 01_10).
//!
//! The chart's three lines are `model::average_yes` for the direct question
//! (the truth told with chance 1), a coin alone (0) and the rule (1/2): each
//! person's chance of a yes, added up. The coin alone's answers on average
//! are half of each group, checked against that line before they are drawn.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    RING_R, YesChart, disc, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, outline, pause, read_by, ring,
    say, swap, think, wait_until,
};
use super::the_coin::{KEY, KEY_DX, KEY_X, KEY_Y, QUESTION, QUESTION_Y, seat_at};
use crate::facts::{self, Average};
use crate::model::{self, Channel, Q};
use crate::palette;

pub struct AskingStraightOut {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const SEEN_KEY: &str = "copied: the survey sees this too";
const ACROSS: &str = "how many copied";
const UP: &str = "yes answers, on average";
const DIRECT: &str = "asked straight out";
const ALONE: &str = "coin only";
const UNKNOWN: &str = "?";
const HONEST: &str = "will people answer honestly? a separate question";
const OURS: &str = "this video's question: what does each answer give away?";

// ------------------------------------------------------------------ layout

/// The chart on the right: 0 to 20 copied across, 0 to 20 yeses up.
const CHART: YesChart = YesChart {
    n: model::CLASS_SIZE,
    x0: 0.9,
    y0: -1.9,
    w: 5.2,
    h: 4.2,
};
/// The two questions at the upper left, and the coin's note under the class.
const NOTE_X: f64 = -6.45;
const NOTE_Y: [f64; 2] = [2.55, 2.05];
const COIN_NOTE_Y: f64 = -1.45;
/// The think time between the class and the chart.
const THINK_X: f64 = -2.1;
const THINK_Y: f64 = 0.4;
/// The direct line's name, near its top; the coin line's, over its right
/// end; the dashed line's "?", past its right end.
const DIRECT_LABEL_DX: f64 = -1.0;
const DIRECT_LABEL_DY: f64 = 0.25;
const ALONE_LABEL_K: i128 = 16;
const ALONE_LABEL_DY: f64 = 0.25;
const UNKNOWN_DX: f64 = 0.2;
/// The dashed line's pieces.
const DASHES: usize = 9;

/// 01_3's exact version: what an honest survey learns.
fn honest_card() -> String {
    "if everyone answers truthfully, the survey learns each person's answer to this one question".to_owned()
}

/// 01_7's exact version: a coin alone, at the chance `p` of a yes for
/// everyone, gives `alone` yeses on average whoever copied.
fn alone_card(p: Q, alone: Q) -> String {
    format!("a coin alone: yes with chance {p} for everyone, so on average n/2 = {alone} yeses, whatever k is")
}

/// 01_5's note under the class.
fn coin_note(yes: usize) -> String {
    format!("one coin each, on average: {yes} yes")
}

/// A coin alone on average: the k-th member of each group (those who
/// copied, and the others, in seat order) says yes when k is even, so half
/// of each group does.
fn alone_says(k_in_group: usize) -> bool {
    k_in_group.is_multiple_of(2)
}

impl SceneConstruct for AskingStraightOut {
    fn name(&self) -> &str {
        "01_asking_straight_out"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let a: Average = facts::average().map_err(fail)?;
        let (n, copiers) = (a.class.size(), a.class.copiers());
        let (nq, kq) = (Q::int(n as i128), Q::int(copiers as i128));
        let direct = Channel::new(Q::ONE).map_err(fail)?;
        let alone_rule = Channel::new(Q::ZERO).map_err(fail)?;
        let rule = Channel::two_coins();
        let alone = model::average_yes(alone_rule, n, copiers);

        // 01_1: the question, the class, and an empty chart
        let line = say(&kit, stage, "01_1")?;
        let question = kit.text(stage, QUESTION, 34.0, palette::student(), v(0.0, QUESTION_Y))?;
        fade_in_all(stage, &[question], 0.5)?;
        let mut discs = Vec::with_capacity(n);
        for i in 0..n {
            discs.push(disc(stage, palette::student(), seat_at(i)));
        }
        fade_in_lagged(stage, &discs, 2.2, 0.15)?;
        line.wait_for(stage, "Anyone who did")?;
        let axes = CHART.axes(&kit, stage, ACROSS, UP)?;
        fade_in_all(stage, &axes, 0.5)?;

        // 01_2: only we can see who copied
        let line = say(&kit, stage, "01_2")?;
        line.wait_for(stage, "Four of them")?;
        let mut rings = Vec::with_capacity(copiers);
        for i in (0..n).filter(|&i| a.class.copied(i)) {
            rings.push(ring(stage, seat_at(i)));
        }
        if rings.len() != copiers {
            return Err(fail(format!("{} rings for {copiers} copiers", rings.len())));
        }
        fade_in_lagged(stage, &rings, 0.9, 0.4)?;
        let key_ring = ring(stage, v(KEY_X, KEY_Y));
        let mut key_text = kit.text_left(stage, KEY, 20.0, Palette::dim(), v(KEY_X + KEY_DX, KEY_Y))?;
        fade_in_all(stage, &[key_ring, key_text], 0.4)?;

        // 01_3: everyone honest: every yes a confession; the direct line
        let line = say(&kit, stage, "01_3")?;
        let mut honest = Vec::with_capacity(n);
        for i in 0..n {
            honest.push(disc(stage, palette::answer(a.class.copied(i)), seat_at(i)));
        }
        swap(stage, &discs, &honest, 0.5)?;
        discs = honest;
        line.wait_for(stage, "Then every yes")?;
        let seen = kit.text_left(stage, SEEN_KEY, 20.0, Palette::dim(), v(KEY_X + KEY_DX, KEY_Y))?;
        swap(stage, &[key_text], &[seen], 0.3)?;
        key_text = seen;
        flash_all(stage, &rings, 0.5)?;
        let direct_line = CHART.line(
            stage,
            (Q::ZERO, model::average_yes(direct, n, 0)),
            (nq, model::average_yes(direct, n, n)),
            Palette::ink(),
        );
        let top = CHART.p(nq, model::average_yes(direct, n, n));
        let direct_label = kit.text(
            stage,
            DIRECT,
            20.0,
            Palette::ink(),
            v(top[0] + DIRECT_LABEL_DX, top[1] + DIRECT_LABEL_DY),
        )?;
        let direct_point = CHART.point(stage, kq, model::average_yes(direct, n, copiers), palette::copied());
        let honest_text = honest_card();
        let honest_note = precisely(&kit, stage, &honest_text)?;
        fade_in_all(stage, &[direct_line, direct_label, direct_point, honest_note], 0.5)?;
        let honest_read = read_by(stage, &honest_text);

        // 01_4: honesty is a separate question; ours is what an answer gives away
        let line = say(&kit, stage, "01_4")?;
        line.wait_for(stage, "a separate question")?;
        let not_ours = kit.text_left(stage, HONEST, 20.0, Palette::dim(), v(NOTE_X, NOTE_Y[0]))?;
        fade_in_all(stage, &[not_ours], 0.4)?;
        line.wait_for(stage, "Ours is")?;
        let ours = kit.text_left(stage, OURS, 20.0, Palette::ink(), v(NOTE_X, NOTE_Y[1]))?;
        fade_in_all(stage, &[ours], 0.4)?;

        // 01_5: the opposite, a coin alone; half of each group says yes
        let line = say(&kit, stage, "01_5")?;
        wait_until(stage, honest_read)?;
        let plain_key = kit.text_left(stage, KEY, 20.0, Palette::dim(), v(KEY_X + KEY_DX, KEY_Y))?;
        let mut plain = Vec::with_capacity(n);
        for i in 0..n {
            plain.push(disc(stage, palette::student(), seat_at(i)));
        }
        let mut old = discs.clone();
        old.extend([key_text, honest_note]);
        let mut new = plain.clone();
        new.push(plain_key);
        swap(stage, &old, &new, 0.4)?;
        discs = plain;
        line.wait_for(stage, "Heads yes")?;
        let mut seen_in = [0usize; 2];
        let mut said = Vec::with_capacity(n);
        for i in 0..n {
            let group = &mut seen_in[usize::from(a.class.copied(i))];
            said.push(alone_says(*group));
            *group += 1;
        }
        let yes = said.iter().filter(|&&s| s).count();
        if Q::int(yes as i128) != alone {
            return Err(fail(format!(
                "{yes} coin yeses drawn, the coin alone gives {alone} on average"
            )));
        }
        let mut coined = Vec::with_capacity(n);
        for (i, &s) in said.iter().enumerate() {
            coined.push(disc(stage, palette::answer(s), seat_at(i)));
        }
        let note = kit.text_left(stage, &coin_note(yes), 20.0, Palette::dim(), v(NOTE_X, COIN_NOTE_Y))?;
        let mut new = coined.clone();
        new.push(note);
        swap(stage, &discs, &new, 0.4)?;
        discs = coined;

        // 01_6: the prediction; nothing drawn at ten yet
        let line = say(&kit, stage, "01_6")?;
        line.wait_for(stage, "counts the yeses")?;
        let yes_discs: Vec<Mob> = discs.iter().zip(&said).filter(|&(_, &s)| s).map(|(&m, _)| m).collect();
        flash_all(stage, &yes_discs, 0.5)?;
        let q = think(&kit, stage, v(THINK_X, THINK_Y))?;

        // 01_7: nothing; ten on average, four copiers or none
        let line = say(&kit, stage, "01_7")?;
        fade_out_all(stage, &[q], 0.2)?;
        line.wait_for(stage, "Nothing")?;
        let alone_line = CHART.line(stage, (Q::ZERO, alone), (nq, alone), Palette::dim());
        let label_at = CHART.p(Q::int(ALONE_LABEL_K), alone);
        let alone_label = kit.text(
            stage,
            ALONE,
            20.0,
            Palette::dim(),
            v(label_at[0], label_at[1] + ALONE_LABEL_DY),
        )?;
        let alone_text = alone_card(alone_rule.p_said(true, true), alone);
        let alone_note = precisely(&kit, stage, &alone_text)?;
        fade_in_all(stage, &[alone_line, alone_label, alone_note], 0.5)?;
        let alone_read = read_by(stage, &alone_text);
        line.wait_for(stage, "Four copiers or none")?;
        let points = [
            CHART.point(stage, kq, alone, palette::copied()),
            CHART.point(stage, Q::ZERO, model::average_yes(alone_rule, n, 0), Palette::dim()),
        ];
        fade_in_all(stage, &points, 0.3)?;
        flash_all(stage, &points, 0.5)?;
        line.wait_end(stage)?;
        // the two lines stand together for a moment (a named hold)
        pause(stage, 1.0)?;

        // 01_8: the two failures side by side
        let line = say(&kit, stage, "01_8")?;
        line.wait_for(stage, "give the count")?;
        flash_all(stage, &[direct_line], 0.5)?;
        line.wait_for(stage, "exposes the person")?;
        flash_all(stage, &rings, 0.5)?;
        line.wait_for(stage, "A private coin alone")?;
        flash_all(stage, &[alone_line], 0.5)?;

        // 01_9: what we want sits in between
        let line = say(&kit, stage, "01_9")?;
        wait_until(stage, alone_read)?;
        fade_out_all(stage, &[alone_note], 0.3)?;
        line.wait_for(stage, "sits in between")?;
        let (start, end) = (model::average_yes(rule, n, 0), model::average_yes(rule, n, n));
        let mut between = CHART.dashed(stage, (Q::ZERO, start), (nq, end), palette::truth(), DASHES);
        let end_at = CHART.p(nq, end);
        between.push(kit.text(
            stage,
            UNKNOWN,
            26.0,
            palette::truth(),
            v(end_at[0] + UNKNOWN_DX, end_at[1]),
        )?);
        fade_in_lagged(stage, &between, 0.8, 0.2)?;

        // 01_10: the count can still be estimated; no answer is a confession
        let line = say(&kit, stage, "01_10")?;
        line.wait_for(stage, "estimate the count")?;
        let ours_point = CHART.point(stage, kq, model::average_yes(rule, n, copiers), palette::copied());
        fade_in_all(stage, &[ours_point], 0.3)?;
        flash_all(stage, &[ours_point], 0.5)?;
        line.wait_for(stage, "no single answer")?;
        let mut faint = Vec::with_capacity(copiers);
        for i in (0..n).filter(|&i| a.class.copied(i)) {
            faint.push(outline(stage, palette::ghost(), seat_at(i), RING_R));
        }
        swap(stage, &rings, &faint, 0.5)?;
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
            ("01_1", "Anyone who did"),
            ("01_2", "Four of them"),
            ("01_3", "Then every yes"),
            ("01_4", "a separate question"),
            ("01_4", "Ours is"),
            ("01_5", "Heads yes"),
            ("01_6", "counts the yeses"),
            ("01_7", "Nothing"),
            ("01_7", "Four copiers or none"),
            ("01_8", "give the count"),
            ("01_8", "exposes the person"),
            ("01_8", "A private coin alone"),
            ("01_9", "sits in between"),
            ("01_10", "estimate the count"),
            ("01_10", "no single answer"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The chart's lines are the facts': the direct question gives the
    /// copiers themselves, a coin alone ten whoever copied, and the rule
    /// five to fifteen through the class's seven; and a coin alone on
    /// average says yes for half of each group.
    #[test]
    fn the_chart_and_the_coin_are_the_facts() {
        let f = facts::facts().unwrap();
        let alone = model::average_yes(Channel::new(Q::ZERO).unwrap(), 20, 4);
        assert_eq!(f.get("R-F17.coin_alone.average_yes"), Some(alone.to_string().as_str()));
        assert_eq!(f.get("R-F17.rule.at_none"), Some("5"));
        assert_eq!(f.get("R-F17.rule.at_all"), Some("15"));
        assert_eq!(f.get("R-F17.rule.average_yes"), Some("7"));
        assert_eq!(f.get("R-F17.direct.average_yes"), Some("4"));
        let a = facts::average().unwrap();
        let mut seen = [0usize; 2];
        let mut yes = [0usize; 2];
        for i in 0..a.class.size() {
            let g = usize::from(a.class.copied(i));
            if alone_says(seen[g]) {
                yes[g] += 1;
            }
            seen[g] += 1;
        }
        assert_eq!(yes, [8, 2], "half of each group");
        assert_eq!(Q::int((yes[0] + yes[1]) as i128), alone);
        assert_eq!(coin_note(10), "one coin each, on average: 10 yes");
        assert_eq!(
            alone_card(Q::new(1, 2), alone),
            "a coin alone: yes with chance 1/2 for everyone, so on average n/2 = 10 yeses, whatever k is"
        );
    }

    /// The cards stay two lines at every wrap the lower-third can narrow to.
    #[test]
    fn the_cards_are_two_lines() {
        let alone = model::average_yes(Channel::new(Q::ZERO).unwrap(), 20, 4);
        for text in [honest_card(), alone_card(Q::new(1, 2), alone)] {
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
        // the chart right of the class and the think ring, inside the frame
        let last = seat_at(model::CLASS_SIZE - 1);
        assert!(last[0] + RING_R < THINK_X - 0.62 - 0.3);
        assert!(THINK_X + 0.62 < CHART.x0 - 0.3 - 0.3);
        assert!(CHART.x0 + CHART.w + 0.3 < FRAME_X_RADIUS - 0.25);
        // the axis's name over the band; the side axis's name under the question
        assert!(CHART.across_name_at()[1] - 0.15 > BAND_TOP);
        assert!(CHART.up_name_at()[1] + 0.15 < QUESTION_Y - 0.25);
        // the direct line's name inside the right edge and under the question
        let top = CHART.p(Q::int(20), Q::int(20));
        assert!(top[0] + DIRECT_LABEL_DX + 1.0 < FRAME_X_RADIUS - 0.25);
        assert!(top[1] + DIRECT_LABEL_DY + 0.15 < QUESTION_Y - 0.25);
        // the two questions at the upper left over the class, under the question
        const { assert!(NOTE_Y[0] + 0.15 < QUESTION_Y - 0.25, "the notes under the question") };
        let first = seat_at(0);
        assert!(NOTE_Y[1] - 0.15 > first[1] + RING_R + 0.2);
        // the coin's note between the class and the key, over the band
        assert!(COIN_NOTE_Y + 0.15 < last[1] - RING_R - 0.1);
        const { assert!(COIN_NOTE_Y - 0.15 > KEY_Y + 0.2, "the note over the key") };
        const { assert!(THINK_Y - 0.62 - 0.45 > BAND_TOP, "the think note over the band") };
        // the "?" past the dashed line's end, inside the frame
        let end = CHART.p(Q::int(20), Q::int(15));
        assert!(end[0] + UNKNOWN_DX + 0.2 < FRAME_X_RADIUS - 0.25);
    }
}
