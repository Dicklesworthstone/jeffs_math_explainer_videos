//! Chapter 01, two bad echoes: send pattern A and find its echo by sliding
//! a copy along it and adding the products that line up (01_1 to 01_4); one
//! step off, a score where nothing is (01_5); the whole row, ghosts on both
//! sides (01_6); every pattern of more than one pulse has them, so send a
//! second (01_7); its ghosts are the same size at the same shifts, and the
//! two rows are about to be added (01_8, a prediction); added, only the echo
//! is left, twice as tall (01_9, 01_10); the guarantee needs the same scene
//! for both sends, and one step of movement brings the ghosts back (01_11).

use explainer_kit::props::{lower_third, precisely};
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    CHART_Y, COPY_Y, Chart, ECHO_Y, HOVER_Y, LABEL_X, Place, Row, SEND_Y, STAGE_Y, STEP, TICK_Y, X0, block,
    fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, flat, pause, product_text, pulse, row, say, score_text,
    shift_all, swap, think, wait_until, x_of,
};
use crate::facts::{self, SHIFTS};
use crate::model::{self, CODE_A, CODE_B, MOVED, STILL};
use crate::palette;

pub struct TwoBadEchoes {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const ECHO: &str = "echo";
const COPY: &str = "copy";
const SCORE: &str = "score";
const SHIFT: &str = "shift";
const GHOST: &str = "ghost";
const AGREE: &str = "agree";
const DISAGREE: &str = "disagree";
const PATTERN_A: &str = "pattern A";
const PATTERN_B: &str = "pattern B";
const ADDED: &str = "both rows added";
const MOVED_TAG: &str = "moved one step";
const PROGRAMMERS: &str = "for programmers:";
const SCORE_CODE: &str = "score[k] = sum over n of echo[n] * copy[n - k]";

/// Where the sender and the thing that bounces the pulses back stand.
const SENDER_X: f64 = -6.4;
const THING_X: f64 = 6.4;
/// The sent pattern's first pulse (01_1), before it becomes the copy.
const SENT_X: f64 = -5.6;
/// The key (01_3): the pulse pairs, the blocks, the words.
const KEY_X: f64 = 4.2;
const KEY_Y: [f64; 2] = [1.25, 0.05];

/// The far-shift rule, the exact version of 01_7: built from the edge
/// test's fact (every pattern of 2 to 10 pulses scores ±1 there).
fn far_rule() -> fmn::Result<String> {
    let size = facts::far_shift_size().map_err(fail)?;
    Ok(format!(
        "with more than one pulse, the copy's last shift lines up only the two end pulses, so that score is {} or {}, never {}. A single pulse has no other shift",
        product_text(size),
        score_text(-size),
        score_text(0)
    ))
}

/// The exact version of 01_10: what the pair removes (the patterns' own
/// ghosts), and what it does not promise to remove (receiver noise).
fn exact_claim() -> String {
    format!(
        "when both sends meet the same still scene, every score away from the echo is exactly {}. The pair removes the patterns' own ghosts. Noise picked up by the receiver is scored too, and is not guaranteed to cancel",
        score_text(facts::sum().largest_ghost(&[0]))
    )
}

/// How fast a shift is scored: the pairs flash, the products appear, drop
/// into the column, and cancelled pairs vanish.
#[derive(Clone, Copy)]
struct Pace {
    step: f64,
    flash: f64,
    appear: f64,
    drop: f64,
    cancel: f64,
}

const SLOW: Pace = Pace {
    step: 0.45,
    flash: 0.5,
    appear: 0.35,
    drop: 0.6,
    cancel: 0.4,
};

const FAST: Pace = Pace {
    step: 0.18,
    flash: 0.15,
    appear: 0.12,
    drop: 0.25,
    cancel: 0.18,
};

/// Score shift `k` with the copy already there: the lined-up pairs flash,
/// one block per product appears between the rows under its echo pulse,
/// and the blocks settle into the column of shift `k`.
fn score_shift(
    stage: &mut Stage<'_>,
    chart: &mut Chart,
    echo: &Row,
    copy: &Row,
    pattern: &[i64],
    k: i64,
    pace: Pace,
) -> fmn::Result<()> {
    let products = model::products(pattern, pattern, k);
    let lit: Vec<Mob> = products
        .iter()
        .flat_map(|p| [echo.pulses[p.recording], copy.pulses[p.copy]])
        .collect();
    flash_all(stage, &lit, pace.flash)?;
    let mut arriving = Vec::with_capacity(products.len());
    for p in &products {
        let at = v(x_of(p.recording as i64), STAGE_Y);
        arriving.push((block(stage, p.value, at, 1.0)?, p.value));
    }
    let blocks: Vec<Mob> = arriving.iter().map(|&(m, _)| m).collect();
    fade_in_all(stage, &blocks, pace.appear)?;
    chart.settle_all(stage, &[(k, arriving)], pace.drop, pace.cancel)
}

/// The key: two pulses that agree over a green block, two that disagree over
/// a red one, each with its word and its product. Detached, one list each.
fn key(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<[Vec<Mob>; 2]> {
    let pairs = facts::key_pairs();
    let mut out = [Vec::new(), Vec::new()];
    for (i, (&(top, bottom), &y)) in pairs.iter().zip(&KEY_Y).enumerate() {
        let product = top * bottom;
        let upper = pulse(top, KEY_X, y + 0.12, palette::pattern_a(), 0.6)?;
        let lower = pulse(bottom, KEY_X, y - 0.32, palette::pattern_a(), 0.6)?;
        out[i].push(stage.arena_mut().add(upper));
        out[i].push(stage.arena_mut().add(lower));
        out[i].push(block(stage, product, v(KEY_X + 0.6, y - 0.1), 1.0)?);
        let (word, color) = if product > 0 {
            (AGREE, palette::agree())
        } else {
            (DISAGREE, palette::disagree())
        };
        out[i].push(kit.text_left(stage, word, 24.0, color, v(KEY_X + 0.95, y - 0.1))?);
        out[i].push(kit.text_left(
            stage,
            &product_text(product),
            24.0,
            Palette::ink(),
            v(KEY_X + 2.15, y - 0.1),
        )?);
    }
    Ok(out)
}

impl SceneConstruct for TwoBadEchoes {
    fn name(&self) -> &str {
        "01_two_bad_echoes"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (a, b) = (facts::score_a(), facts::score_b());

        // 01_1: four pulses go out, and something bounces them back
        let line = say(&kit, stage, "01_1")?;
        let sender = Circle::new()
            .radius(0.16)
            .style(flat(Palette::ink(), 1.0))
            .build()
            .moved_to(v(SENDER_X, SEND_Y));
        let sender = stage.arena_mut().add(sender);
        let thing = Rectangle::new()
            .width(0.45)
            .height(1.1)
            .corner_radius(0.08)
            .style(flat(Palette::dim(), 0.6))
            .build()?
            .moved_to(v(THING_X, SEND_Y));
        let thing = stage.arena_mut().add(thing);
        let sent = row(stage, &CODE_A, SENT_X, SEND_Y, palette::pattern_a())?;
        fade_in_all(stage, &[sender, thing, sent.line], 0.4)?;
        line.wait_for(stage, "Up, up")?;
        let spoken = line.at("Something")? - line.at("Up, up")?;
        fade_in_lagged(stage, &sent.pulses, spoken.clamp(0.8, 2.0), 0.6)?;
        line.wait_for(stage, "Something")?;
        // under the sent row, at the height of the thing it bounces off
        let dot = Circle::new()
            .radius(0.09)
            .style(flat(palette::pattern_a(), 1.0))
            .build()
            .moved_to(v(SENDER_X + 0.3, SEND_Y - 0.5));
        let dot = stage.arena_mut().add(dot);
        let run = THING_X - 0.35 - (SENDER_X + 0.3);
        fade_in_all(stage, &[dot], 0.15)?;
        shift_all(stage, &[dot], run, 0.0, 0.8)?;
        shift_all(stage, &[dot], -run, 0.0, 0.8)?;
        let echo = row(stage, &CODE_A, X0, ECHO_Y, palette::pattern_a())?;
        let echo_label = kit.text(stage, ECHO, 24.0, Palette::dim(), v(LABEL_X, ECHO_Y))?;
        let mut arriving = echo.mobs();
        arriving.push(echo_label);
        swap(stage, &[dot, thing, sender], &arriving, 0.5)?;

        // 01_2: the sent pattern is the receiver's copy; it slides along the
        // echo, three steps to the left first
        let line = say(&kit, stage, "01_2")?;
        let copy = sent;
        let copy_label = kit.text(stage, COPY, 24.0, Palette::dim(), v(LABEL_X, COPY_Y))?;
        let mut at_shift = SHIFTS.0;
        shift_all(stage, &copy.mobs(), x_of(at_shift) - SENT_X, COPY_Y - SEND_Y, 0.8)?;
        let mut chart = Chart::new(stage, Place::main());
        let mut frame = vec![copy_label, chart.line];
        for k in a.first..=a.last() {
            frame.push(kit.text(stage, &score_text(k), 20.0, Palette::dim(), v(x_of(k), TICK_Y))?);
        }
        frame.push(kit.text(stage, SCORE, 22.0, Palette::dim(), v(LABEL_X, CHART_Y))?);
        frame.push(kit.text(stage, SHIFT, 22.0, Palette::dim(), v(LABEL_X, TICK_Y))?);
        fade_in_all(stage, &frame, 0.5)?;
        line.wait_for(stage, "slide a copy")?;
        while at_shift < 0 {
            shift_all(stage, &copy.mobs(), STEP, 0.0, 0.4)?;
            at_shift += 1;
        }
        line.wait_for(stage, "and multiply")?;
        let code = lower_third(&kit, stage, PROGRAMMERS, SCORE_CODE)?;
        fade_in_all(stage, &[code], 0.4)?;

        // 01_3: the key, one pair at a time
        let line = say(&kit, stage, "01_3")?;
        let [agree, disagree] = key(&kit, stage)?;
        fade_in_lagged(stage, &agree, 0.9, 0.3)?;
        line.wait_for(stage, "Opposite ways")?;
        fade_in_lagged(stage, &disagree, 0.9, 0.3)?;

        // 01_4: lined up exactly, four agreements, the echo
        let line = say(&kit, stage, "01_4")?;
        fade_out_all(stage, &[code], 0.3)?;
        score_shift(stage, &mut chart, &echo, &copy, &CODE_A, 0, SLOW)?;
        line.wait_for(stage, "A score of four")?;
        let h = chart.height(0);
        let peak = kit.text(
            stage,
            &score_text(h),
            30.0,
            Palette::ink(),
            chart.place.past(0, h, 0.25),
        )?;
        let echo_word = kit.text(stage, ECHO, 28.0, palette::echo(), chart.place.past(0, h, 0.6))?;
        fade_in_all(stage, &[peak, echo_word], 0.4)?;

        // 01_5: one step off, a score where nothing is
        let line = say(&kit, stage, "01_5")?;
        shift_all(stage, &copy.mobs(), STEP, 0.0, SLOW.step)?;
        at_shift += 1;
        score_shift(stage, &mut chart, &echo, &copy, &CODE_A, at_shift, SLOW)?;
        line.wait_for(stage, "A score of one")?;
        let h = chart.height(at_shift);
        let one = kit.text(
            stage,
            &score_text(h),
            30.0,
            Palette::ink(),
            chart.place.past(at_shift, h, 0.25),
        )?;
        fade_in_all(stage, &[one], 0.3)?;
        line.wait_for(stage, "where nothing is")?;
        let first_ghost = kit.text(stage, GHOST, 20.0, Palette::dim(), chart.place.past(at_shift, h, 0.55))?;
        fade_in_all(stage, &[first_ghost], 0.3)?;

        // 01_6: every other shift, quickly; then the ghosts are named
        let line = say(&kit, stage, "01_6")?;
        fade_out_all(stage, &[one, first_ghost], 0.25)?;
        let rest: Vec<i64> = (at_shift + 1..=a.last()).chain((a.first..0).rev()).collect();
        for k in rest {
            let steps = (k - at_shift) as f64;
            shift_all(stage, &copy.mobs(), STEP * steps, 0.0, FAST.step * steps.abs())?;
            at_shift = k;
            score_shift(stage, &mut chart, &echo, &copy, &CODE_A, k, FAST)?;
        }
        chart.check(&a, "pattern A")?;
        let mut ghost_words = Vec::new();
        for k in a.ghost_shifts(&[0]) {
            let h = chart.height(k);
            ghost_words.push(kit.text(stage, GHOST, 20.0, Palette::dim(), chart.place.past(k, h, 0.25))?);
        }
        line.wait_for(stage, "a quarter the size")?;
        fade_in_lagged(stage, &ghost_words, 0.8, 0.3)?;

        // 01_7: the far shifts always score; a second pattern is sent
        let line = say(&kit, stage, "01_7")?;
        let far = CODE_A.len() as i64 - 1;
        let mut ends = chart.column(-far);
        ends.extend(chart.column(far));
        flash_all(stage, &ends, 0.6)?;
        let rule = precisely(&kit, stage, &far_rule()?)?;
        fade_in_all(stage, &[rule], 0.4)?;
        line.wait_for(stage, "So send a second one")?;
        let mut labels = vec![peak, echo_word];
        labels.extend(&ghost_words);
        fade_out_all(stage, &labels, 0.3)?;
        let mut chart_a = chart;
        let mut chart = Chart::new(stage, Place::main());
        chart_a.move_to(stage, Place::shelf(), 0.8)?;
        let shelf = Place::shelf().line_centre();
        let shelf_name = kit.text(
            stage,
            PATTERN_A,
            22.0,
            palette::pattern_a(),
            v(shelf[0], shelf[1] + 0.85),
        )?;
        fade_in_all(stage, &[shelf_name, chart.line], 0.4)?;
        line.wait_for(stage, "Up, up, down, up")?;
        let echo_b = row(stage, &CODE_B, X0, ECHO_Y, palette::pattern_b())?;
        let copy_b = row(stage, &CODE_B, x_of(at_shift), COPY_Y, palette::pattern_b())?;
        let mut old = echo.mobs();
        old.extend(copy.mobs());
        let mut new = echo_b.mobs();
        new.extend(copy_b.mobs());
        swap(stage, &old, &new, 0.5)?;

        // 01_8: B's row, every shift at once after a quick sweep; A comes
        // down over it; the question, and the think time
        let line = say(&kit, stage, "01_8")?;
        fade_out_all(stage, &[rule], 0.3)?;
        shift_all(stage, &copy_b.mobs(), STEP * (b.last() - at_shift) as f64, 0.0, 1.0)?;
        let mut leaving = echo_b.mobs();
        leaving.extend(copy_b.mobs());
        leaving.extend(&agree);
        leaving.extend(&disagree);
        leaving.push(echo_label);
        leaving.push(copy_label);
        fade_out_all(stage, &leaving, 0.3)?;
        let mut arrivals = Vec::new();
        let mut appearing = Vec::new();
        for k in b.first..=b.last() {
            let mut column = Vec::new();
            for (j, p) in model::products(&CODE_B, &CODE_B, k).iter().enumerate() {
                let m = block(stage, p.value, v(x_of(k), STAGE_Y + 0.27 * j as f64), 1.0)?;
                column.push((m, p.value));
                appearing.push(m);
            }
            arrivals.push((k, column));
        }
        fade_in_all(stage, &appearing, 0.3)?;
        chart.settle_all(stage, &arrivals, 0.5, 0.25)?;
        chart.check(&b, "pattern B")?;
        line.wait_for(stage, "What if we add")?;
        let name_a = kit.text_left(
            stage,
            PATTERN_A,
            22.0,
            palette::pattern_a(),
            v(LABEL_X - 0.75, HOVER_Y + 0.35),
        )?;
        let name_b = kit.text_left(
            stage,
            PATTERN_B,
            22.0,
            palette::pattern_b(),
            v(LABEL_X - 0.75, CHART_Y + 0.35),
        )?;
        fade_out_all(stage, &[shelf_name], 0.2)?;
        chart_a.move_to(stage, Place::hover(), 0.9)?;
        fade_in_all(stage, &[name_a, name_b], 0.3)?;
        let q = think(&kit, stage, v(5.3, 0.6))?;

        // 01_9: A's row drops onto B's: every ghost meets its opposite
        let line = say(&kit, stage, "01_9")?;
        let added = kit.text_left(stage, ADDED, 22.0, Palette::ink(), v(LABEL_X - 0.75, CHART_Y + 0.35))?;
        let falling = chart_a.take();
        fade_out_all(stage, &[q, chart_a.line, name_a], 0.3)?;
        chart.settle_all(stage, &falling, 1.0, 0.4)?;
        chart.check(&facts::sum(), "both rows added")?;
        swap(stage, &[name_b], &[added], 0.3)?;
        line.wait_for(stage, "Only the echo")?;
        let h = chart.height(0);
        let peak = kit.text(
            stage,
            &score_text(h),
            30.0,
            Palette::ink(),
            chart.place.past(0, h, 0.25),
        )?;
        let echo_word = kit.text(stage, ECHO, 28.0, palette::echo(), chart.place.past(0, h, 0.6))?;
        fade_in_all(stage, &[peak, echo_word], 0.4)?;

        // 01_10: not a single ghost, precisely
        let line = say(&kit, stage, "01_10")?;
        line.wait_for(stage, "not a single ghost")?;
        let exact = precisely(&kit, stage, &exact_claim())?;
        fade_in_all(stage, &[exact], 0.4)?;
        // the storyboard's hold, kept short: thought gaps already lengthen
        // the chapter, which must stay under 85 s
        kit.music_breath(stage, 2.0)?;

        // 01_11: the same scene for both sends; one step of movement
        let line = say(&kit, stage, "01_11")?;
        let mut leaving = vec![exact, peak, echo_word, added];
        leaving.extend(
            chart
                .take()
                .into_iter()
                .flat_map(|(_, c)| c.into_iter().map(|(m, _)| m)),
        );
        fade_out_all(stage, &leaving, 0.4)?;
        let mut chart_a = Chart::new(stage, Place::hover());
        let name_a = kit.text_left(
            stage,
            PATTERN_A,
            22.0,
            palette::pattern_a(),
            v(LABEL_X - 0.75, HOVER_Y + 0.35),
        )?;
        let name_b = kit.text_left(
            stage,
            PATTERN_B,
            22.0,
            palette::pattern_b(),
            v(LABEL_X - 0.75, CHART_Y + 0.35),
        )?;
        fade_in_all(stage, &[chart_a.line, name_a, name_b], 0.3)?;
        chart_a.stand(stage, &model::measure(&CODE_A, &STILL, &[]), 0.4)?;
        chart.stand(stage, &model::measure(&CODE_B, &STILL, &[]), 0.4)?;
        line.wait_for(stage, "Move the echo")?;
        let echoes = [
            facts::echo_step(&STILL).map_err(fail)?,
            facts::echo_step(&MOVED).map_err(fail)?,
        ];
        let step = echoes[1] - echoes[0];
        let tag = kit.text_left(
            stage,
            MOVED_TAG,
            20.0,
            Palette::dim(),
            v(LABEL_X - 0.75, CHART_Y - 0.35),
        )?;
        let tick = kit.text(
            stage,
            &score_text(SHIFTS.1),
            20.0,
            Palette::dim(),
            v(x_of(SHIFTS.1), TICK_Y),
        )?;
        fade_in_all(stage, &[tag, tick], 0.3)?;
        chart.shift_columns(stage, step, 0.6)?;
        chart.check(&model::measure(&CODE_B, &MOVED, &[]), "pattern B, moved")?;
        line.wait_for(stage, "and ghosts return")?;
        let falling = chart_a.take();
        fade_out_all(stage, &[chart_a.line, name_a], 0.3)?;
        chart.settle_all(stage, &falling, 0.9, 0.35)?;
        let moved = facts::moved();
        chart.check(&moved, "both rows added, moved")?;
        let mut ghosts = Vec::new();
        for k in moved.ghost_shifts(&echoes) {
            ghosts.extend(chart.column(k));
        }
        let mut split = Vec::new();
        for k in echoes {
            let h = chart.height(k);
            split.push(kit.text(
                stage,
                &score_text(h),
                26.0,
                Palette::ink(),
                chart.place.past(k, h, 0.25),
            )?);
        }
        fade_in_all(stage, &split, 0.3)?;
        flash_all(stage, &ghosts, 0.6)?;
        let end = line.start + line.len;
        wait_until(stage, end)?;
        pause(stage, 0.8)?;

        fade_all(stage, 1.0)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The words on screen say what the facts say, and no more: the noise
    /// sentence promises nothing either way.
    #[test]
    fn the_exact_versions_are_built_from_the_facts() {
        let rule = far_rule().unwrap();
        assert!(rule.contains("+1 or \u{2212}1, never 0"), "{rule}");
        let claim = exact_claim();
        assert!(claim.contains("exactly 0."), "{claim}");
        assert!(claim.contains("not guaranteed to cancel"), "{claim}");
        assert!(!claim.contains("does not cancel"), "{claim}");
    }

    /// Every phrase the scene waits for is in its line of script.tsv (a
    /// changed line fails here, not in the middle of a render).
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("01_1", "Up, up"),
            ("01_1", "Something"),
            ("01_2", "slide a copy"),
            ("01_2", "and multiply"),
            ("01_3", "Opposite ways"),
            ("01_4", "A score of four"),
            ("01_5", "A score of one"),
            ("01_5", "where nothing is"),
            ("01_6", "a quarter the size"),
            ("01_7", "So send a second one"),
            ("01_7", "Up, up, down, up"),
            ("01_8", "What if we add"),
            ("01_9", "Only the echo"),
            ("01_10", "not a single ghost"),
            ("01_11", "Move the echo"),
            ("01_11", "and ghosts return"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The key's two pairs are pulses of pattern A: one agreeing pair and one
    /// disagreeing pair.
    #[test]
    fn the_key_shows_one_agreement_and_one_disagreement() {
        let products: Vec<i64> = facts::key_pairs().iter().map(|&(t, b)| t * b).collect();
        assert_eq!(products, [1, -1]);
    }

    /// The chapter's layout keeps its pieces apart.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        use crate::chapters::draw::{BLOCK, PULSE_H};
        assert!(
            x_of(SHIFTS.0) - 0.35 > LABEL_X + 0.5,
            "the copy's far left clears the names"
        );
        // the hovering row (A) clears the main row (B) column above column
        let (a, b) = (facts::score_a(), facts::score_b());
        let deepest_a = (a.first..=a.last()).map(|k| a.at(k)).min().unwrap_or(0) as f64;
        let tallest_b = (b.first..=b.last()).map(|k| b.at(k)).max().unwrap_or(0) as f64;
        assert!(HOVER_Y + deepest_a * BLOCK > CHART_Y + tallest_b * BLOCK);
        // a shift's products sit between the copy's down pulses and the
        // peak's label
        assert!(COPY_Y - PULSE_H > STAGE_Y + 0.12, "under the copy");
        assert!(
            Place::main().past(0, a.at(0), 0.6)[1] + 0.12 < STAGE_Y - 0.12,
            "over the echo's word"
        );
        // the key stands right of every row and under the shelf
        assert!(KEY_X - 0.1 > x_of(a.last()) + STEP * 3.0 + 0.35);
        let shelf = Place::shelf();
        assert!(KEY_Y[0] + 0.12 + 0.6 * PULSE_H < shelf.slot(0, 1, -1)[1] - 0.12);
    }
}
