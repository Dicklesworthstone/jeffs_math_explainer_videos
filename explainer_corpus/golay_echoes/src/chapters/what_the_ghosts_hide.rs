//! Chapter 02, what the ghosts hide: a loud and a quiet thing three steps
//! apart (02_1); pattern A's two echoes overlap and add into one recording
//! (02_2); scored, A shows nothing where the quiet one is (02_3); because A's
//! row is each echo's own row added, and there the loud one's ghost meets
//! the quiet one's peak (02_4, 02_5); its other ghosts are as tall as the
//! quiet echo (02_6); pattern B's ghost has the other sign (02_7, a
//! prediction) and doubles it (02_8); added, the scene itself, eight times
//! over (02_9); sending A twice keeps the quiet one hidden (02_10).
//!
//! Every row is the model's (`facts::hidden_a` and its kin, each echo's own
//! row from `model::contributions`), and every chart is checked against it
//! as it is drawn. A column is one thin slab per unit of score.

use explainer_kit::props::{lower_third, precisely};
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    BLOCK, CHART_Y, Chart, NAME_X, Place, TICK_Y, X0, fade_in_all, fade_out_all, fail, flash_all, line_style, pause,
    product_text, row, say, scene_row, score_text, shift_all, swap, think, value_row, wait_until, x_of,
};
use crate::facts::{self, SHIFTS_02};
use crate::model::{self, CODE_A, LOUD_AND_QUIET, Scores};
use crate::palette;

pub struct WhatTheGhostsHide {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const SCENE: &str = "the scene";
const LOUD: &str = "loud";
const QUIET: &str = "quiet";
const RECORDING: &str = "recording";
const LOUD_ECHO: &str = "loud echo";
const QUIET_ECHO: &str = "quiet echo";
const PATTERN_A: &str = "pattern A";
const PATTERN_B: &str = "pattern B";
const LOUD_ALONE: &str = "loud alone";
const QUIET_ALONE: &str = "quiet alone";
const ADDED: &str = "both rows added";
const A_TWICE: &str = "A sent twice";
const SHIFT: &str = "shift";
const PROGRAMMERS: &str = "for programmers:";

/// The scene's outlines, and how tall a unit of strength is drawn there.
const SCENE_Y: f64 = 3.15;
const SCENE_UNIT: f64 = 0.12;
/// The recording's line and unit; the quiet echo's own row before it joins.
const REC_Y: f64 = 2.2;
const REC_UNIT: f64 = 0.12;
const QUIET_Y: f64 = 1.3;
/// The copy that sweeps the recording (02_3), under it.
const COPY_Y: f64 = 1.3;
/// The line of a row held above the main one.
const ABOVE_Y: f64 = 0.55;
/// Seconds of scene time each "precisely" card stays fully visible once it
/// is in: 02_9's before 02_10 begins, 02_10's before the chapter fades. Its
/// own line and the narrator's gap count toward it.
const CARD_DWELL_S: f64 = 10.0;

fn main_place() -> Place {
    Place::slabs(CHART_Y, SHIFTS_02)
}

fn above_place() -> Place {
    Place::slabs(ABOVE_Y, SHIFTS_02)
}

/// For programmers (02_4): the score is linear in the scene, with the
/// chapter's own strengths and step.
fn linear() -> fmn::Result<String> {
    let [(loud_step, loud), (quiet_step, quiet)] = facts::loud_and_quiet().map_err(fail)?;
    Ok(format!(
        "the score is linear in the scene: row(scene) = {loud} * row(one echo) + {quiet} * row(one echo, {} steps later)",
        quiet_step - loud_step
    ))
}

/// The exact version of 02_9: the assumption, the magnification, and what
/// is not promised (receiver noise).
fn exact_claim() -> fmn::Result<String> {
    let [_, (quiet_step, quiet)] = facts::loud_and_quiet().map_err(fail)?;
    let times = facts::ratio(facts::hidden_sum().at(quiet_step), quiet).map_err(fail)?;
    Ok(format!(
        "with the same still scene for both sends, the two rows add to exactly {times} times the scene. Receiver noise is scored too, and is not guaranteed to cancel"
    ))
}

/// The exact version of 02_10, from the exhaustive count: the pairs that
/// give this scene back are exactly the complementary ones.
fn wrong_pair_claim() -> fmn::Result<String> {
    let [pairs, back, back_complementary, complementary] = facts::pairs_giving_back();
    if back != back_complementary || back != complementary {
        return Err(fail(format!(
            "{back} pairs give the scene back, {back_complementary} of them complementary, {complementary} complementary in all"
        )));
    }
    Ok(format!(
        "of the {pairs} ordered pairs of {}-pulse patterns, exactly the {back} whose ghosts cancel give this scene back. Any other pair gets every scene with an echo wrong",
        CODE_A.len()
    ))
}

/// A gold outline `units` tall on `place`'s column `k`: what one echo alone
/// would score there. Detached.
fn outline(stage: &mut Stage<'_>, place: &Place, k: i64, units: i64) -> fmn::Result<Mob> {
    let h = units.unsigned_abs() as f64 * BLOCK * place.rise;
    let y = if units >= 0 { h / 2.0 } else { -h / 2.0 };
    let o = Rectangle::new()
        .width(place.unit_w + 0.12)
        .height(h)
        .style(line_style(palette::echo(), 2.5))
        .build()?
        .moved_to(v(place.column_x(k), place.origin[1] + y));
    Ok(stage.arena_mut().add(o))
}

/// A score's number by its column, just past the column's end.
fn score_label(kit: &Kit, stage: &mut Stage<'_>, chart: &Chart, k: i64, signed: bool) -> fmn::Result<Mob> {
    let h = chart.height(k);
    let words = if signed { product_text(h) } else { score_text(h) };
    kit.text(stage, &words, 26.0, Palette::ink(), chart.place.past(k, h, 0.2))
}

/// The row an echo at `step` adds, from the model.
fn part(parts: &[(i64, Scores)], step: i64) -> fmn::Result<Scores> {
    parts
        .iter()
        .find(|(d, _)| *d == step)
        .map(|(_, r)| r.clone())
        .ok_or_else(|| fail(format!("no echo at step {step}")))
}

/// Every unit standing in `chart`, taken out of it.
fn take_all(chart: &mut Chart) -> Vec<Mob> {
    chart
        .take()
        .into_iter()
        .flat_map(|(_, c)| c.into_iter().map(|(m, _)| m))
        .collect()
}

impl SceneConstruct for WhatTheGhostsHide {
    fn name(&self) -> &str {
        "02_what_the_ghosts_hide"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let [(loud_step, loud), (quiet_step, quiet)] = facts::loud_and_quiet().map_err(fail)?;
        let (a, b) = (facts::hidden_a(), facts::hidden_b());
        let parts = model::contributions(&CODE_A, &LOUD_AND_QUIET);
        let (loud_row, quiet_row) = (part(&parts, loud_step)?, part(&parts, quiet_step)?);
        let name_at = |y: f64| v(NAME_X, y + 0.35);

        // 02_1: the scene, what the receiver is trying to find
        let line = say(&kit, stage, "02_1")?;
        let scene = scene_row(stage, &LOUD_AND_QUIET, SCENE_Y, SCENE_UNIT)?;
        let scene_name = kit.text_left(stage, SCENE, 22.0, Palette::dim(), v(NAME_X, SCENE_Y + 0.2))?;
        fade_in_all(stage, &[scene.line, scene_name], 0.4)?;
        line.wait_for(stage, "A loud one")?;
        let loud_word = kit.text_left(
            stage,
            LOUD,
            20.0,
            Palette::dim(),
            v(x_of(loud_step) + 0.3, SCENE_Y + 0.15),
        )?;
        fade_in_all(stage, &[scene.pulses[0], loud_word], 0.4)?;
        line.wait_for(stage, "a quiet one")?;
        let quiet_word = kit.text_left(
            stage,
            QUIET,
            20.0,
            Palette::dim(),
            v(x_of(quiet_step) + 0.3, SCENE_Y + 0.15),
        )?;
        fade_in_all(stage, &[scene.pulses[1], quiet_word], 0.4)?;

        // 02_2: pattern A comes back from both; the two echoes overlap and add
        let line = say(&kit, stage, "02_2")?;
        let loud_echo = value_row(
            stage,
            &CODE_A.map(|p| p * loud),
            x_of(loud_step),
            REC_Y,
            REC_UNIT,
            palette::pattern_a(),
        )?;
        let quiet_echo = value_row(
            stage,
            &CODE_A.map(|p| p * quiet),
            x_of(quiet_step),
            QUIET_Y,
            REC_UNIT,
            palette::pattern_a(),
        )?;
        let loud_name = kit.text_left(stage, LOUD_ECHO, 22.0, Palette::dim(), v(NAME_X, REC_Y))?;
        let quiet_name = kit.text_left(stage, QUIET_ECHO, 22.0, Palette::dim(), v(NAME_X, QUIET_Y))?;
        let mut first = loud_echo.mobs();
        first.push(loud_name);
        fade_in_all(stage, &first, 0.4)?;
        let mut second = quiet_echo.mobs();
        second.push(quiet_name);
        fade_in_all(stage, &second, 0.4)?;
        line.wait_for(stage, "overlap")?;
        shift_all(stage, &quiet_echo.mobs(), 0.0, REC_Y - QUIET_Y, 0.6)?;
        line.wait_for(stage, "records their sum")?;
        let recording = value_row(
            stage,
            &model::echo(&CODE_A, &LOUD_AND_QUIET),
            X0,
            REC_Y,
            REC_UNIT,
            palette::pattern_a(),
        )?;
        let rec_name = kit.text_left(stage, RECORDING, 22.0, Palette::dim(), v(NAME_X, REC_Y))?;
        let mut old = loud_echo.mobs();
        old.extend(quiet_echo.mobs());
        old.extend([loud_name, quiet_name]);
        let mut new = recording.mobs();
        new.push(rec_name);
        swap(stage, &old, &new, 0.4)?;

        // 02_3: scored as before, A's row: tall at the loud one, zero at the
        // quiet one
        let line = say(&kit, stage, "02_3")?;
        let copy = row(stage, &CODE_A, x_of(SHIFTS_02.0), COPY_Y, palette::pattern_a())?;
        let mut chart = Chart::new(stage, main_place());
        let mut frame = vec![chart.line];
        for k in SHIFTS_02.0..=SHIFTS_02.1 {
            frame.push(kit.text(stage, &score_text(k), 20.0, Palette::dim(), v(x_of(k), TICK_Y))?);
        }
        frame.push(kit.text_left(stage, SHIFT, 22.0, Palette::dim(), v(NAME_X, TICK_Y))?);
        frame.extend(copy.mobs());
        fade_in_all(stage, &frame, 0.4)?;
        let sweep = x_of(SHIFTS_02.1) - x_of(SHIFTS_02.0);
        shift_all(stage, &copy.mobs(), sweep, 0.0, 1.2)?;
        fade_out_all(stage, &copy.mobs(), 0.2)?;
        chart.stand(stage, &a, 0.6)?;
        chart.check(&a, "pattern A")?;
        let name_a = kit.text_left(stage, PATTERN_A, 22.0, palette::pattern_a(), name_at(CHART_Y))?;
        fade_in_all(stage, &[name_a], 0.3)?;
        line.wait_for(stage, "the score is zero")?;
        let guide = polyline(vec![
            v(x_of(quiet_step), SCENE_Y - 0.05),
            v(x_of(quiet_step), CHART_Y + 0.5),
        ])
        .style(line_style(palette::echo(), 2.0))
        .build();
        let guide = stage.arena_mut().add(guide);
        let zero = score_label(&kit, stage, &chart, quiet_step, false)?;
        let mut leaving = recording.mobs();
        leaving.push(rec_name);
        swap(stage, &leaving, &[guide, zero], 0.4)?;

        // 02_4: each echo's own row, the loud one's above the quiet one's
        let line = say(&kit, stage, "02_4")?;
        fade_out_all(stage, &[guide, zero], 0.3)?;
        line.wait_for(stage, "Each echo")?;
        let mut leaving = take_all(&mut chart);
        leaving.push(name_a);
        fade_out_all(stage, &leaving, 0.3)?;
        let mut above = Chart::new(stage, above_place());
        let loud_alone = kit.text_left(stage, LOUD_ALONE, 22.0, Palette::dim(), name_at(ABOVE_Y))?;
        let quiet_alone = kit.text_left(stage, QUIET_ALONE, 22.0, Palette::dim(), name_at(CHART_Y))?;
        fade_in_all(stage, &[above.line, loud_alone, quiet_alone], 0.3)?;
        above.stand(stage, &loud_row, 0.5)?;
        chart.stand(stage, &quiet_row, 0.5)?;
        above.check(&loud_row, "the loud echo's own row")?;
        chart.check(&quiet_row, "the quiet echo's own row")?;
        let code = lower_third(&kit, stage, PROGRAMMERS, &linear()?)?;
        fade_in_all(stage, &[code], 0.3)?;
        line.wait_for(stage, "they collide")?;
        let mut meeting = above.column(quiet_step);
        meeting.extend(chart.column(quiet_step));
        flash_all(stage, &meeting, 0.5)?;

        // 02_5: minus four and plus four; the loud row drops on the quiet one
        let line = say(&kit, stage, "02_5")?;
        fade_out_all(stage, &[code], 0.2)?;
        line.wait_for(stage, "minus four")?;
        let minus = score_label(&kit, stage, &above, quiet_step, false)?;
        fade_in_all(stage, &[minus], 0.3)?;
        line.wait_for(stage, "plus four")?;
        let plus = score_label(&kit, stage, &chart, quiet_step, true)?;
        fade_in_all(stage, &[plus], 0.3)?;
        line.wait_for(stage, "They cancel")?;
        let falling = above.take();
        fade_out_all(stage, &[minus, plus, above.line, loud_alone], 0.2)?;
        chart.settle_all(stage, &falling, 0.6, 0.3)?;
        chart.check(&a, "the two echoes' rows added")?;

        // 02_6: the loud ghosts beside it are as tall as the quiet echo
        let line = say(&kit, stage, "02_6")?;
        let name_a = kit.text_left(stage, PATTERN_A, 22.0, palette::pattern_a(), name_at(CHART_Y))?;
        swap(stage, &[quiet_alone], &[name_a], 0.3)?;
        line.wait_for(stage, "four tall")?;
        let near = [loud_step - 1, loud_step + 1];
        let mut fours = Vec::new();
        let mut beside = Vec::new();
        for k in near {
            fours.push(score_label(&kit, stage, &chart, k, false)?);
            beside.extend(chart.column(k));
        }
        fade_in_all(stage, &fours, 0.3)?;
        flash_all(stage, &beside, 0.5)?;
        line.wait_for(stage, "as tall as the quiet echo")?;
        let alone = outline(stage, &chart.place, quiet_step, quiet_row.at(quiet_step))?;
        fade_in_all(stage, &[alone], 0.4)?;

        // 02_7: A's row waits above; what will B score at three steps?
        let line = say(&kit, stage, "02_7")?;
        fade_out_all(stage, &fours, 0.2)?;
        let mut above = chart;
        above.move_to(stage, above_place(), 0.8)?;
        shift_all(stage, &[name_a], 0.0, ABOVE_Y - CHART_Y, 0.3)?;
        let mut chart = Chart::new(stage, main_place());
        let ask = kit.text(
            stage,
            "?",
            30.0,
            Palette::hero(),
            chart.place.past(quiet_step, quiet_row.at(quiet_step), 0.45),
        )?;
        fade_in_all(stage, &[chart.line, ask], 0.3)?;
        line.wait_for(stage, "What will B score")?;
        let q = think(&kit, stage, v(5.3, 0.6))?;

        // 02_8: eight, the quiet echo doubled
        let line = say(&kit, stage, "02_8")?;
        fade_out_all(stage, &[q, ask], 0.2)?;
        chart.stand(stage, &b, 0.5)?;
        chart.check(&b, "pattern B")?;
        let name_b = kit.text_left(stage, PATTERN_B, 22.0, palette::pattern_b(), name_at(CHART_Y))?;
        let eight = score_label(&kit, stage, &chart, quiet_step, false)?;
        fade_in_all(stage, &[name_b, eight], 0.3)?;
        line.wait_for(stage, "doubles it")?;
        flash_all(stage, &chart.column(quiet_step), 0.5)?;

        // 02_9: A's row drops onto B's: the scene, eight times over
        let line = say(&kit, stage, "02_9")?;
        let falling = above.take();
        fade_out_all(stage, &[eight, alone, above.line, name_a], 0.3)?;
        chart.settle_all(stage, &falling, 0.9, 0.35)?;
        let sum = facts::hidden_sum();
        chart.check(&sum, "both rows added")?;
        let added = kit.text_left(stage, ADDED, 22.0, Palette::ink(), name_at(CHART_Y))?;
        swap(stage, &[name_b], &[added], 0.3)?;
        line.wait_for(stage, "Thirty-two")?;
        let big = score_label(&kit, stage, &chart, loud_step, false)?;
        fade_in_all(stage, &[big], 0.3)?;
        line.wait_for(stage, "eight at three")?;
        let small = score_label(&kit, stage, &chart, quiet_step, false)?;
        fade_in_all(stage, &[small], 0.3)?;
        line.wait_for(stage, "The scene")?;
        let mut lit = chart.column(loud_step);
        lit.extend(chart.column(quiet_step));
        lit.extend(&scene.pulses);
        flash_all(stage, &lit, 0.5)?;
        let exact = precisely(&kit, stage, &exact_claim()?)?;
        fade_in_all(stage, &[exact], 0.4)?;
        // the card stays over the sum it makes exact until it can be read
        let shown = stage.scene().time().to_f64();
        wait_until(stage, shown + CARD_DWELL_S)?;

        // 02_10: the wrong pair, A sent twice: the quiet one stays hidden
        let line = say(&kit, stage, "02_10")?;
        let mut leaving = take_all(&mut chart);
        leaving.extend([big, small, added]);
        fade_out_all(stage, &leaving, 0.4)?;
        let card = precisely(&kit, stage, &wrong_pair_claim()?)?;
        swap(stage, &[exact], &[card], 0.3)?;
        let card_shown = stage.scene().time().to_f64();
        let mut above = Chart::new(stage, above_place());
        let top_name = kit.text_left(stage, PATTERN_A, 22.0, palette::pattern_a(), name_at(ABOVE_Y))?;
        let bottom_name = kit.text_left(stage, PATTERN_A, 22.0, palette::pattern_a(), name_at(CHART_Y))?;
        fade_in_all(stage, &[above.line, top_name, bottom_name], 0.3)?;
        above.stand(stage, &a, 0.4)?;
        chart.stand(stage, &a, 0.4)?;
        line.wait_for(stage, "doubles its ghosts")?;
        let falling = above.take();
        fade_out_all(stage, &[above.line, top_name], 0.2)?;
        chart.settle_all(stage, &falling, 0.7, 0.3)?;
        chart.check(&facts::a_twice(), "A sent twice")?;
        let twice = kit.text_left(stage, A_TWICE, 22.0, Palette::ink(), name_at(CHART_Y))?;
        let hidden = score_label(&kit, stage, &chart, quiet_step, false)?;
        swap(stage, &[bottom_name], &[twice, hidden], 0.3)?;
        let mut ghosts = Vec::new();
        for k in near {
            ghosts.extend(chart.column(k));
        }
        flash_all(stage, &ghosts, 0.5)?;
        let end = line.start + line.len;
        wait_until(stage, end)?;
        pause(stage, 0.5)?;
        wait_until(stage, card_shown + CARD_DWELL_S)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The words on screen say what the facts say, and promise no more.
    #[test]
    fn the_cards_are_built_from_the_facts() {
        let claim = exact_claim().unwrap();
        assert!(claim.contains("same still scene for both sends"), "{claim}");
        assert!(claim.contains("exactly 8 times the scene"), "{claim}");
        assert!(claim.contains("not guaranteed to cancel"), "{claim}");
        let wrong = wrong_pair_claim().unwrap();
        assert!(
            wrong.contains("of the 256 ordered pairs of 4-pulse patterns"),
            "{wrong}"
        );
        assert!(wrong.contains("exactly the 32 whose ghosts cancel"), "{wrong}");
        assert!(
            linear()
                .unwrap()
                .contains("4 * row(one echo) + 1 * row(one echo, 3 steps later)")
        );
    }

    /// Both "precisely" cards wrap to two lines, so they clear the shift
    /// numbers (chapter 01's draft taught that).
    #[test]
    fn the_precisely_cards_are_two_lines() {
        for text in [exact_claim().unwrap(), wrong_pair_claim().unwrap()] {
            let lines = explainer_kit::animatic::wrap(&text, 92, 3);
            assert!(lines.len() <= 2, "{} lines: {text}", lines.len());
        }
    }

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("02_1", "A loud one"),
            ("02_1", "a quiet one"),
            ("02_2", "overlap"),
            ("02_2", "records their sum"),
            ("02_3", "the score is zero"),
            ("02_4", "Each echo"),
            ("02_4", "they collide"),
            ("02_5", "minus four"),
            ("02_5", "plus four"),
            ("02_5", "They cancel"),
            ("02_6", "four tall"),
            ("02_6", "as tall as the quiet echo"),
            ("02_7", "What will B score"),
            ("02_8", "doubles it"),
            ("02_9", "Thirty-two"),
            ("02_9", "eight at three"),
            ("02_9", "The scene"),
            ("02_10", "doubles its ghosts"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The chapter's layout keeps its pieces apart, at the model's own
    /// heights: the row held above clears the row below, the tallest row
    /// stays under the scene, the deepest one over the shift numbers.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        let (above, main) = (above_place(), main_place());
        let rows = [
            facts::hidden_a(),
            facts::hidden_b(),
            facts::hidden_sum(),
            facts::a_twice(),
        ];
        let span = |r: &Scores| {
            let values: Vec<i64> = (r.first..=r.last()).map(|k| r.at(k)).collect();
            (
                values.iter().copied().max().unwrap_or(0),
                values.iter().copied().min().unwrap_or(0),
            )
        };
        let unit = BLOCK * main.rise;
        for r in &rows {
            let (top, bottom) = span(r);
            assert!(
                main.origin[1] + top as f64 * unit + 0.6 < SCENE_Y,
                "{r:?} reaches the scene"
            );
            assert!(
                main.origin[1] + bottom as f64 * unit > TICK_Y + 0.15,
                "{r:?} reaches the shift numbers"
            );
        }
        // what stands above at the same time as a row below: the loud echo's
        // row over the quiet one's (02_4, 02_5), A over B (02_7 to 02_9), A
        // over A (02_10)
        let parts = model::contributions(&CODE_A, &LOUD_AND_QUIET);
        let (loud, quiet) = (parts[0].1.clone(), parts[1].1.clone());
        for (upper, lower) in [
            (loud, quiet),
            (facts::hidden_a(), facts::hidden_b()),
            (facts::hidden_a(), facts::hidden_a()),
        ] {
            let ((_, low), (high, _)) = (span(&upper), span(&lower));
            assert!(above.origin[1] + low as f64 * unit > main.origin[1] + high as f64 * unit + 0.2);
        }
        // the recording's lowest bar clears the copy that sweeps under it
        let deepest = model::echo(&CODE_A, &LOUD_AND_QUIET).into_iter().min().unwrap_or(0);
        assert!(REC_Y + deepest as f64 * REC_UNIT > COPY_Y + crate::chapters::draw::PULSE_H);
    }
}
