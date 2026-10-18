//! Chapter 04, when it breaks: the clean control, set on a shelf beside
//! every failure (04_1); the echo moves one step between the sends (04_2 to
//! 04_4); the same echo comes back upside down, which is not a new object
//! and not a negative distance (04_5, 04_6); the true echo cancels and every
//! ghost doubles (04_7, a prediction, and 04_8); the identity for any change
//! (04_9); a noise blip is scored and the peak survives (04_10); the ending
//! (04_11).
//!
//! Every row is the model's: the control and the moved row from
//! `two_measurements`, the flipped row with `model::FLIPPED`, the blip from
//! `model::NOISE_BLIP`, each drawn as chapter 01's blocks and checked
//! against the model as it stands or settles.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    CHART_Y, Chart, HOVER_Y, LABEL_X, NAME_X, PULSE_H, Place, STEP, TICK_Y, X0, fade_in_all, fade_out_all, fail,
    flash_all, pause, pulse, say, scene_row, score_text, shift_all, swap, think, value_row, wait_until, x_of,
};
use crate::facts::{self, SHIFTS};
use crate::model::{self, CODE_A, CODE_B, FLIPPED, MOVED, NOISE_BLIP, STILL, Scores};
use crate::palette;

pub struct WhenItBreaks {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const FIRST_SEND: &str = "first send";
const SECOND_SEND: &str = "second send";
const SAME_SCENE: &str = "same scene, A + B";
const PATTERN_A: &str = "pattern A";
const PATTERN_B: &str = "pattern B";
const ADDED: &str = "both rows added";
const ADDED_NOISY: &str = "both rows + blip";
const SCORE: &str = "score";
const SHIFT: &str = "shift";
const RECORDING: &str = "first recording";
const BLIP_ROW: &str = "the blip's own row";
const TITLE: &str = "Two bad echoes, one perfect measurement";

/// Each send's scene line, and how tall one unit of strength is drawn.
const FIRST_Y: f64 = 3.3;
const SECOND_Y: f64 = 2.55;
const SCENE_UNIT: f64 = 0.3;
/// The first recording (04_10), with its blip.
const REC_Y: f64 = 1.85;
/// The countdown, right of the rows and under the shelf.
const THINK_AT: [f64; 2] = [4.9, 0.5];
/// The identity (04_9), and where h₁ and h₂ stand beside the sends.
const IDENTITY_AT: [f64; 2] = [0.0, 0.45];
const H_X: f64 = 0.4;
/// The title (04_11).
const TITLE_Y: f64 = 1.45;

/// Pattern A's row for `scene`.
fn row_a(scene: &[i64]) -> Scores {
    model::measure(&CODE_A, scene, &[])
}

/// Pattern B's row for `scene`.
fn row_b(scene: &[i64]) -> Scores {
    model::measure(&CODE_B, scene, &[])
}

/// The pair's peak for one echo of strength 1: the identity's factor.
fn factor() -> fmn::Result<String> {
    facts::ratio(facts::sum().at(0), STILL[0]).map_err(fail)
}

/// A ratio on screen, with a typographic minus.
fn shown(ratio: &str) -> String {
    ratio.replace('-', "\u{2212}")
}

/// The exact version of 04_6, within what the source supports.
fn phase_claim() -> fmn::Result<String> {
    let by = shown(&facts::ratio(FLIPPED[0], STILL[0]).map_err(fail)?);
    Ok(format!(
        "half a turn of phase between the sends multiplies the second echo by {by} (Chi, Pezeshki and Calderbank, eq. 8): the same reflector, at the same distance"
    ))
}

/// The identity (04_9), its factor from the facts.
fn identity_tex() -> fmn::Result<String> {
    Ok(format!("S_A(h_1) + S_B(h_2) = {}h_2 + S_A(h_1 - h_2)", factor()?))
}

/// The identity's terms, made exact (04_9): no added noise, and the factor
/// is this pair's, two for every pulse.
fn scope_claim() -> fmn::Result<String> {
    let n = CODE_A.len() as i64;
    let per_pulse = facts::ratio(facts::sum().at(0), STILL[0] * n).map_err(fail)?;
    Ok(format!(
        "with no added noise: S_A(h) is pattern A's row for scene h, scenes and rows zero past their ends. {} = {per_pulse}N for our N = {n} pulses; any complementary pair of length N gives 2N",
        factor()?
    ))
}

/// The exact version of the ending (04_11): the ideal case, and the noise
/// 04_10 showed still there outside it.
fn closing_claim() -> fmn::Result<String> {
    Ok(format!(
        "perfect means exactly {} times the scene with no ghosts, in the ideal case: the same echo response in both sends, and no added noise. Added noise is scored, not guaranteed to cancel",
        factor()?
    ))
}

/// Every block standing in `chart`, taken out of it.
fn take_all(chart: &mut Chart) -> Vec<Mob> {
    chart
        .take()
        .into_iter()
        .flat_map(|(_, c)| c.into_iter().map(|(m, _)| m))
        .collect()
}

/// A score's number just past its column.
fn score_label(kit: &Kit, stage: &mut Stage<'_>, chart: &Chart, k: i64) -> fmn::Result<Mob> {
    let h = chart.height(k);
    kit.text(
        stage,
        &score_text(h),
        26.0,
        Palette::ink(),
        chart.place.past(k, h, 0.25),
    )
}

impl SceneConstruct for WhenItBreaks {
    fn name(&self) -> &str {
        "04_when_it_breaks"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let name_at = |y: f64| v(NAME_X, y + 0.35);
        let still = facts::echo_step(&STILL).map_err(fail)?;
        let step = facts::echo_step(&MOVED).map_err(fail)? - still;
        // the second send's line spans every step its echo visits
        let span: Vec<i64> = (0..MOVED.len()).map(|d| STILL.get(d).copied().unwrap_or(0)).collect();

        // 04_1: the clean control, then set on a shelf
        let line = say(&kit, stage, "04_1")?;
        let first = scene_row(stage, &span, FIRST_Y, SCENE_UNIT)?;
        let second = scene_row(stage, &span, SECOND_Y, SCENE_UNIT)?;
        let first_name = kit.text_left(stage, FIRST_SEND, 22.0, Palette::dim(), v(NAME_X, FIRST_Y + 0.12))?;
        let second_name = kit.text_left(stage, SECOND_SEND, 22.0, Palette::dim(), v(NAME_X, SECOND_Y + 0.12))?;
        let mut above = Chart::new(stage, Place::hover());
        let mut chart = Chart::new(stage, Place::main());
        let mut frame = first.mobs();
        frame.extend(second.mobs());
        frame.extend([first_name, second_name, above.line, chart.line]);
        for k in SHIFTS.0..=SHIFTS.1 {
            frame.push(kit.text(stage, &score_text(k), 20.0, Palette::dim(), v(x_of(k), TICK_Y))?);
        }
        frame.push(kit.text(stage, SCORE, 22.0, Palette::dim(), v(LABEL_X, CHART_Y))?);
        frame.push(kit.text(stage, SHIFT, 22.0, Palette::dim(), v(LABEL_X, TICK_Y))?);
        let name_a = kit.text_left(stage, PATTERN_A, 22.0, palette::pattern_a(), name_at(HOVER_Y))?;
        let name_b = kit.text_left(stage, PATTERN_B, 22.0, palette::pattern_b(), name_at(CHART_Y))?;
        frame.extend([name_a, name_b]);
        fade_in_all(stage, &frame, 0.4)?;
        above.stand(stage, &row_a(&STILL), 0.4)?;
        chart.stand(stage, &row_b(&STILL), 0.4)?;
        let falling = above.take();
        chart.settle_all(stage, &falling, 0.6, 0.3)?;
        chart.check(&facts::sum(), "the control")?;
        // each label names the row beside it: A's row is gone, B's is the total
        let added = kit.text_left(stage, ADDED, 22.0, Palette::ink(), name_at(CHART_Y))?;
        swap(stage, &[name_a, name_b], &[added], 0.3)?;
        line.wait_for(stage, "every ghost cancels")?;
        fade_out_all(stage, &[added], 0.2)?;
        let mut control = chart;
        control.move_to(stage, Place::shelf(), 0.8)?;
        let shelf = Place::shelf().line_centre();
        let shelf_name = kit.text(stage, SAME_SCENE, 22.0, Palette::dim(), v(shelf[0], shelf[1] - 0.35))?;
        let mut chart = Chart::new(stage, Place::main());
        fade_in_all(stage, &[shelf_name, chart.line], 0.3)?;

        // 04_2: the echo moves one step between the sends
        let line = say(&kit, stage, "04_2")?;
        line.wait_for(stage, "move one step")?;
        shift_all(stage, &second.pulses, STEP * step as f64, 0.0, 0.6)?;

        // 04_3: B's row moves with it; A's drops on it
        let line = say(&kit, stage, "04_3")?;
        let name_a = kit.text_left(stage, PATTERN_A, 22.0, palette::pattern_a(), name_at(HOVER_Y))?;
        let name_b = kit.text_left(stage, PATTERN_B, 22.0, palette::pattern_b(), name_at(CHART_Y))?;
        fade_in_all(stage, &[name_a, name_b], 0.2)?;
        above.stand(stage, &row_a(&STILL), 0.4)?;
        chart.stand(stage, &row_b(&STILL), 0.4)?;
        line.wait_for(stage, "slides with it")?;
        chart.shift_columns(stage, step, 0.5)?;
        chart.check(&row_b(&MOVED), "pattern B, the echo moved")?;
        line.wait_for(stage, "miss their opposites")?;
        let falling = above.take();
        chart.settle_all(stage, &falling, 0.6, 0.3)?;
        let moved = facts::moved();
        chart.check(&moved, "both rows added, the echo moved")?;
        let added = kit.text_left(stage, ADDED, 22.0, Palette::ink(), name_at(CHART_Y))?;
        swap(stage, &[name_a, name_b], &[added], 0.3)?;

        // 04_4: three and five, ghosts at six shifts
        let line = say(&kit, stage, "04_4")?;
        line.wait_for(stage, "three and five")?;
        let echoes = [still, still + step];
        let split = [
            score_label(&kit, stage, &chart, echoes[0])?,
            score_label(&kit, stage, &chart, echoes[1])?,
        ];
        fade_in_all(stage, &split, 0.3)?;
        line.wait_for(stage, "six shifts")?;
        let mut ghosts = Vec::new();
        for k in moved.ghost_shifts(&echoes) {
            ghosts.extend(chart.column(k));
        }
        flash_all(stage, &ghosts, 0.5)?;

        // 04_5: the echo back in place, coming back upside down
        let line = say(&kit, stage, "04_5")?;
        let mut leaving = take_all(&mut chart);
        leaving.extend(split);
        leaving.push(added);
        swap(stage, &leaving, &[], 0.4)?;
        shift_all(stage, &second.pulses, -STEP * step as f64, 0.0, 0.5)?;
        line.wait_for(stage, "upside down")?;
        let turned = scene_row(stage, &FLIPPED, SECOND_Y, SCENE_UNIT)?;
        swap(stage, &second.pulses, &turned.pulses, 0.4)?;

        // 04_6: not a new object, not a negative distance
        let line = say(&kit, stage, "04_6")?;
        let card = precisely(&kit, stage, &phase_claim()?)?;
        fade_in_all(stage, &[card], 0.4)?;
        line.wait_for(stage, "The same echo")?;
        flash_all(stage, &[turned.pulses[0], first.pulses[0]], 0.6)?;

        // 04_7: what happens to the true echo? B's row turned over hangs from
        // the line above, where its −4 has room (under the main line it
        // would cover the shift numbers); A's row stands below
        let line = say(&kit, stage, "04_7")?;
        let name_b = kit.text_left(stage, PATTERN_B, 22.0, palette::pattern_b(), name_at(HOVER_Y))?;
        let name_a = kit.text_left(stage, PATTERN_A, 22.0, palette::pattern_a(), name_at(CHART_Y))?;
        fade_in_all(stage, &[name_b, name_a], 0.2)?;
        above.stand(stage, &row_b(&FLIPPED), 0.4)?;
        above.check(&row_b(&FLIPPED), "pattern B, the echo upside down")?;
        chart.stand(stage, &row_a(&STILL), 0.4)?;
        chart.check(&row_a(&STILL), "pattern A")?;
        line.wait_for(stage, "when we add")?;
        let q = think(&kit, stage, v(THINK_AT[0], THINK_AT[1]))?;

        // 04_8: it cancels; every ghost doubles. B's row drops onto A's
        let line = say(&kit, stage, "04_8")?;
        fade_out_all(stage, &[q], 0.2)?;
        let falling = above.take();
        chart.settle_all(stage, &falling, 0.6, 0.3)?;
        let flipped = facts::flipped();
        chart.check(&flipped, "both rows added, the echo upside down")?;
        let added = kit.text_left(stage, ADDED, 22.0, Palette::ink(), name_at(CHART_Y))?;
        let zero = score_label(&kit, stage, &chart, still)?;
        swap(stage, &[name_a, name_b], &[added, zero], 0.3)?;
        line.wait_for(stage, "every ghost doubles")?;
        let mut doubled = Vec::new();
        for k in flipped.ghost_shifts(&[still]) {
            doubled.extend(chart.column(k));
        }
        flash_all(stage, &doubled, 0.5)?;

        // 04_9: the identity for any change
        let line = say(&kit, stage, "04_9")?;
        let h1 = kit.tex(stage, "h_1", 30.0, palette::echo(), &[], v(H_X, FIRST_Y + 0.15))?;
        let h2 = kit.tex(stage, "h_2", 30.0, palette::echo(), &[], v(H_X, SECOND_Y + 0.15))?;
        fade_out_all(stage, &[zero], 0.2)?;
        fade_in_all(stage, &[h1, h2], 0.3)?;
        line.wait_for(stage, "In general")?;
        let identity = kit.tex(
            stage,
            &identity_tex()?,
            40.0,
            Palette::ink(),
            &[],
            v(IDENTITY_AT[0], IDENTITY_AT[1]),
        )?;
        fade_in_all(stage, &[identity], 0.5)?;
        line.wait_for(stage, "whatever changed")?;
        let scope = precisely(&kit, stage, &scope_claim()?)?;
        swap(stage, &[card], &[scope], 0.3)?;

        // 04_10: a noise blip is scored; the peak survives. The total stands
        // again, the same scene in both sends, still "both rows added"; the
        // identity leaves with its no-added-noise card, as the noise arrives
        let line = say(&kit, stage, "04_10")?;
        let mut leaving = take_all(&mut chart);
        leaving.extend([identity, h1, h2, scope]);
        let upright = scene_row(stage, &STILL, SECOND_Y, SCENE_UNIT)?;
        leaving.extend(&turned.pulses);
        swap(stage, &leaving, &upright.pulses, 0.4)?;
        let clean = facts::sum();
        chart.stand(stage, &clean, 0.4)?;
        chart.check(&clean, "the control again")?;
        // the recording's line runs on to the blip's sample
        let mut clean_rec = model::echo(&CODE_A, &STILL);
        clean_rec.resize(clean_rec.len().max(NOISE_BLIP.len()), 0);
        let recording = value_row(stage, &clean_rec, X0, REC_Y, PULSE_H, palette::pattern_a())?;
        let at = NOISE_BLIP
            .iter()
            .position(|&n| n != 0)
            .ok_or_else(|| fail("the noise holds no blip".into()))?;
        let blip = pulse(NOISE_BLIP[at], x_of(at as i64), REC_Y, Palette::dim(), 1.0)?;
        let blip = stage.arena_mut().add(blip);
        let rec_name = kit.text_left(stage, RECORDING, 22.0, Palette::dim(), v(NAME_X, REC_Y))?;
        let mut shown_rec = recording.mobs();
        shown_rec.extend([blip, rec_name]);
        fade_in_all(stage, &shown_rec, 0.4)?;
        line.wait_for(stage, "A stray blip")?;
        flash_all(stage, &[blip], 0.4)?;
        line.wait_for(stage, "its own little row")?;
        let blip_scores = facts::blip_row();
        above.stand(stage, &blip_scores, 0.4)?;
        let blip_name = kit.text_left(stage, BLIP_ROW, 22.0, Palette::dim(), name_at(HOVER_Y))?;
        fade_in_all(stage, &[blip_name], 0.3)?;
        line.wait_for(stage, "nothing guarantees")?;
        let falling = above.take();
        chart.settle_all(stage, &falling, 0.6, 0.3)?;
        chart.check(&facts::noisy(), "the control with a noise blip")?;
        // the line above is empty now, and the peak's label stands where it
        // was: the line leaves first
        fade_out_all(stage, &[above.line, blip_name], 0.2)?;
        let peak = score_label(&kit, stage, &chart, still)?;
        let noisy_name = kit.text_left(stage, ADDED_NOISY, 22.0, Palette::ink(), name_at(CHART_Y))?;
        swap(stage, &[added], &[noisy_name, peak], 0.3)?;
        let mut scored = Vec::new();
        for k in blip_scores.first..=blip_scores.last() {
            if blip_scores.at(k) != 0 {
                scored.extend(chart.column(k));
            }
        }
        flash_all(stage, &scored, 0.5)?;

        // 04_11: the ending: the control returns under the two still scenes
        let line = say(&kit, stage, "04_11")?;
        let mut leaving = take_all(&mut chart);
        leaving.extend(recording.mobs());
        leaving.extend([blip, rec_name, noisy_name, peak, shelf_name, chart.line]);
        fade_out_all(stage, &leaving, 0.4)?;
        control.move_to(stage, Place::main(), 0.8)?;
        control.check(&clean, "the control, back")?;
        let back = kit.text_left(stage, ADDED, 22.0, Palette::ink(), name_at(CHART_Y))?;
        let title = kit.text(stage, TITLE, 36.0, Palette::ink(), v(0.0, TITLE_Y))?;
        fade_in_all(stage, &[back, title], 0.4)?;
        line.wait_for(stage, "the ideal case")?;
        let closing = precisely(&kit, stage, &closing_claim()?)?;
        fade_in_all(stage, &[closing], 0.3)?;
        let end = line.start + line.len;
        wait_until(stage, end)?;
        pause(stage, 0.8)?;

        fade_all(stage, 0.8)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chapters::draw::BLOCK;

    /// The cards and formulas say what the facts say, within the source.
    #[test]
    fn the_screen_text_is_built_from_the_facts() {
        let phase = phase_claim().unwrap();
        assert!(
            phase.contains("by \u{2212}1 (Chi, Pezeshki and Calderbank, eq. 8)"),
            "{phase}"
        );
        assert!(phase.contains("the same reflector, at the same distance"), "{phase}");
        assert_eq!(identity_tex().unwrap(), "S_A(h_1) + S_B(h_2) = 8h_2 + S_A(h_1 - h_2)");
        let scope = scope_claim().unwrap();
        assert!(scope.starts_with("with no added noise"), "{scope}");
        assert!(scope.contains("zero past their ends"), "{scope}");
        assert!(scope.contains("8 = 2N for our N = 4 pulses"), "{scope}");
        let closing = closing_claim().unwrap();
        assert!(closing.contains("exactly 8 times the scene"), "{closing}");
        assert!(
            closing.contains("the same echo response in both sends, and no added noise"),
            "{closing}"
        );
        assert!(closing.contains("not guaranteed to cancel"), "{closing}");
    }

    /// Every "precisely" card wraps to two lines, clear of the shift numbers.
    #[test]
    fn the_precisely_cards_are_two_lines() {
        for text in [phase_claim().unwrap(), scope_claim().unwrap(), closing_claim().unwrap()] {
            let lines = explainer_kit::animatic::wrap(&text, 92, 3);
            assert!(lines.len() <= 2, "{} lines: {text}", lines.len());
        }
    }

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("04_1", "every ghost cancels"),
            ("04_2", "move one step"),
            ("04_3", "slides with it"),
            ("04_3", "miss their opposites"),
            ("04_4", "three and five"),
            ("04_4", "six shifts"),
            ("04_5", "upside down"),
            ("04_6", "The same echo"),
            ("04_7", "when we add"),
            ("04_8", "every ghost doubles"),
            ("04_9", "In general"),
            ("04_9", "whatever changed"),
            ("04_10", "A stray blip"),
            ("04_10", "its own little row"),
            ("04_10", "nothing guarantees"),
            ("04_11", "the ideal case"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The layout keeps its pieces apart.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        // the scenes: inside the safe area, and the flipped outline clears
        // the tallest row held above (pattern A's peak of 4 blocks)
        const { assert!(FIRST_Y + SCENE_UNIT <= 3.85, "the first scene inside the safe area") };
        const { assert!(REC_Y + PULSE_H < SECOND_Y, "the recording under the second scene") };
        let tallest_a = row_a(&STILL).at(0) as f64;
        assert!(SECOND_Y - SCENE_UNIT > HOVER_Y + tallest_a * BLOCK + 0.1);
        // the recording clears the blip's row held above
        let blip = facts::blip_row();
        let tallest_blip = (blip.first..=blip.last()).map(|k| blip.at(k)).max().unwrap_or(0) as f64;
        assert!(REC_Y - PULSE_H > HOVER_Y + tallest_blip * BLOCK + 0.1);
        // the shelf stays inside the frame and clear of the countdown
        let shelf = Place::shelf();
        let peak = facts::sum().at(0);
        assert!(shelf.past(0, peak, 0.0)[1] < 3.85);
        assert!(THINK_AT[1] + 0.62 < shelf.origin[1] - 0.5);
        // the identity sits between the held row's line and the main row
        let flipped = facts::flipped();
        let top = (flipped.first..=flipped.last())
            .map(|k| flipped.at(k))
            .max()
            .unwrap_or(0) as f64;
        assert!(IDENTITY_AT[1] - 0.3 > CHART_Y + top * BLOCK);
        const { assert!(IDENTITY_AT[1] + 0.3 < HOVER_Y) };
        // a row's name stays short, left of the shift −3 column, near
        // chapter 02's "both rows added" (15 characters)
        for name in [
            PATTERN_A,
            PATTERN_B,
            ADDED,
            ADDED_NOISY,
            BLIP_ROW,
            RECORDING,
            FIRST_SEND,
            SECOND_SEND,
        ] {
            assert!(name.chars().count() <= 18, "{name}");
        }
    }

    /// Rows keep clear of each other and of the shift numbers, and a
    /// score's label clears the line above it.
    #[test]
    fn the_rows_and_labels_clear_each_other() {
        // every row standing on the main line keeps its deepest block above
        // the shift numbers; B's row turned over (−4 at its echo) would
        // not, so it hangs from the line above instead
        let lowest = |s: &Scores| (s.first..=s.last()).map(|k| s.at(k)).min().unwrap_or(0).min(0) as f64;
        for (what, row) in [
            ("B's row", row_b(&STILL)),
            ("B's row, moved", row_b(&MOVED)),
            ("the moved total", facts::moved()),
            ("A's row", row_a(&STILL)),
            ("the flipped total", facts::flipped()),
            ("the clean total", facts::sum()),
            ("the noisy total", facts::noisy()),
        ] {
            assert!(CHART_Y + lowest(&row) * BLOCK > TICK_Y + 0.15, "{what}");
        }
        assert!(CHART_Y + lowest(&row_b(&FLIPPED)) * BLOCK <= TICK_Y + 0.15);
        // a row held above clears the row standing below, column by column
        // (a held column of 0 is the line itself)
        for (what, held, standing) in [
            ("04_1", row_a(&STILL), row_b(&STILL)),
            ("04_3", row_a(&STILL), row_b(&MOVED)),
            ("04_7", row_b(&FLIPPED), row_a(&STILL)),
            ("04_10", facts::blip_row(), facts::sum()),
        ] {
            for k in SHIFTS.0..=SHIFTS.1 {
                let bottom = HOVER_Y + held.at(k).min(0) as f64 * BLOCK;
                let top = CHART_Y + standing.at(k).max(0) as f64 * BLOCK;
                assert!(bottom - top > 0.1, "{what} at shift {k}");
            }
        }
        // the labels 3 and 5 (04_4) and 0 (04_8) clear the line above
        let main = Place::main();
        let still = facts::echo_step(&STILL).unwrap();
        let moved_to = facts::echo_step(&MOVED).unwrap();
        let (moved, flipped) = (facts::moved(), facts::flipped());
        for (k, h) in [
            (still, moved.at(still)),
            (moved_to, moved.at(moved_to)),
            (still, flipped.at(still)),
        ] {
            assert!(main.past(k, h, 0.25)[1] + 0.15 < HOVER_Y, "the label {h}");
        }
        // the peak's label (04_10) stands where that line was, so the line
        // leaves first; the label clears the recording's lowest pulse
        let noisy = facts::noisy();
        assert!(main.past(still, noisy.at(still), 0.25)[1] + 0.15 < REC_Y - PULSE_H);
    }
}
