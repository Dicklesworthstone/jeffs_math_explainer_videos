//! Chapter 05, what two percent does not say: the two percent is a share of
//! one imaginary world, the world kept apart from the question asked in it
//! (05_1); is it the chance the treatment did nothing (05_2, a prediction)?
//! No: it was counted inside that world (05_3), and it is not the chance the
//! study is wrong either (05_4); an imagined other result, one more pair the
//! other way, 14 (05_5), has its own tail, about 5.8% of the same world
//! (05_6), and a 5% line between the two is a convention for deciding
//! (05_7); two made-up versions of the study's recovery days (05_8) with the
//! same signs and so the same 15, their sizes far apart among those 15
//! favouring pairs (05_9), give the same two percent: the count never used
//! how much sooner (05_10).
//!
//! No posterior is computed or shown. The days are `model::constructed_days`,
//! labelled made up; their bars are observed paired differences, never
//! treatment effects, and every count on them is read from the days.

use explainer_kit::{Kit, Palette, Timed, thousands, v};
use fmn::prelude::*;

use super::count_tail::{bar_overlays, bracket, bracket_y, last_picture};
use super::draw::{
    fade_in_all, fade_in_lagged, fade_out_all, flat, imagined, line_style, say, think_pause, treated, warn,
};
use super::five_percent::outline;
use super::imaginary_world::study_tag;
use super::same_test::small_fraction;
use crate::facts::CONVENTION_PERCENT;
use crate::model::{Days, Q, Tail, constructed_days, study, tail_count, tail_share};

pub struct WhatTwoPercentDoesNotSay {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const WORLD_A: &str = "assumed world: no one's recovery changes";
const WORLD_B: &str = "coin assignments, all equally likely";
const QUESTION: &str = "asked in it: how often";
const OR_MORE: &str = "or more";
const NOT_NULL: &str = "is not the chance the treatment did nothing";
const NOT_WRONG: &str = "is not the chance the study is wrong";
const THE_STUDY: &str = "the study:";
const IMAGINED: &str = "imagined:";
const CONVENTION: &str = "a convention for deciding";
const MOVED: &str = "pair apart: the evidence moved a little";
const NO_SWITCH: &str = "nothing in nature switches at";
const VERSION: &str = "version";
const MADE_UP: &str = "made up for illustration, not trial data";
const BAR_NOTE: &str =
    "each bar: untreated minus treated days in one pair, an observed difference, not a treatment effect";
const OF: &str = "of";
const PAIRS_FAVOURING: &str = "pairs favouring";
const MEDIAN_AMONG: &str = "median among those";
const DAY_SOONER: &str = "day sooner";
const DAYS_SOONER: &str = "days sooner";
const P_SHORT: &str = "one-sided p ≈";
const CLOSING: &str = "how much sooner is what a patient asks, and this count never measured it";

// --- layout (scene units; the frame is 14.2 by 8)

/// The left column's edge (chapter 04's ledger edge).
const ROW_X: f64 = -6.6;
/// 05_1–05_4: the world's two lines in their outline, the question, the
/// prediction's ring, the two readings it is not.
const WORLD_Y: [f64; 2] = [1.85, 1.5];
const WORLD_PAD: f64 = 0.15;
const QUESTION_Y: f64 = 0.55;
const ASK_AT: Vec3 = [-3.9, -2.55, 0.0];
const ASK_R: f64 = 0.45;
const NOT_Y: [f64; 2] = [-2.35, -2.75];
/// 05_5–05_7: the two results, each a label and a fraction; the imagined
/// bracket, above the study's.
const RESULT_Y: [f64; 2] = [1.7, 0.6];
const FRACTION_X: f64 = -1.75;
const BRACKET_RISE: f64 = 0.25;
/// The share line: 0% to 8% across, the 5% convention on it.
const LINE_Y: f64 = -1.3;
const LINE_X0: f64 = -6.3;
const PER_PERCENT: f64 = 0.7;
const LINE_TOP_PERCENT: u32 = 8;
const MOVED_Y: [f64; 2] = [-2.6, -3.0];
/// 05_8–05_10: two strips of paired differences, one bar per pair, each
/// strip's title, the note under the first, and the facts column right.
const STRIP_BASE: [f64; 2] = [1.2, -1.9];
const STRIP_TITLE_Y: [f64; 2] = [2.6, -0.1];
const NOTE_Y: f64 = 2.25;
const STRIP_X0: f64 = -6.3;
const STRIP_DX: f64 = 0.46;
const BAR_W: f64 = 0.3;
const DAY_H: f64 = 0.15;
const SIGN_DY: f64 = -0.65;
const INFO_X: f64 = 3.1;
const INFO_DY: f64 = 0.36;
const INFO_TOP: [f64; 2] = [1.85, -0.9];
const CAPTION_Y: f64 = -3.2;

/// Where the share `q` stands on the share line.
fn line_x(q: Q) -> f64 {
    LINE_X0 + q.f64() * 100.0 * PER_PERCENT
}

/// The x of pair `i`'s bar in a strip.
fn bar_x(i: usize) -> f64 {
    STRIP_X0 + STRIP_DX * i as f64
}

/// Line `row` of a strip's facts column.
fn info_at(strip: usize, row: usize) -> Vec3 {
    v(INFO_X, INFO_TOP[strip] - INFO_DY * row as f64)
}

/// "1 day sooner" or "7 days sooner".
fn sooner_words(days: Q) -> String {
    let unit = if days == Q::ONE { DAY_SOONER } else { DAYS_SOONER };
    format!("{days} {unit}")
}

/// Scale every mobject in `mobs` up and back together, `rt` seconds.
fn pulse(stage: &mut Stage<'_>, mobs: &[Mob], scale: f64, rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let anim = fmn::animation::indicate(stage.arena_mut(), m, scale, None)?;
        anims.push(stage.prepare(anim.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// One result: its words ("the study: 15 or more") and its fraction with
/// the share to two places, in `color`, at height `y`; detached.
fn result_row(
    kit: &Kit,
    stage: &mut Stage<'_>,
    (lead, k): (&str, u32),
    world: &[u64],
    color: Srgb,
    y: f64,
) -> fmn::Result<[Mob; 2]> {
    let words = kit.text_left(stage, &format!("{lead} {k} {OR_MORE}"), 22.0, color, v(ROW_X, y))?;
    let share = tail_share(world, k, Tail::AtLeast);
    let fraction = small_fraction(
        kit,
        stage,
        (tail_count(world, k, Tail::AtLeast), world.iter().sum()),
        color,
        &format!("≈ {}%", share.percent(2)),
        (FRACTION_X, y),
    )?;
    Ok([words, fraction])
}

/// The share line: its axis, a tick and its words every two percent;
/// detached.
fn share_line(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let end = line_x(Q::new(i128::from(LINE_TOP_PERCENT), 100));
    let axis = polyline(vec![v(LINE_X0, LINE_Y), v(end, LINE_Y)])
        .style(line_style(Palette::dim(), 2.0))
        .build();
    let mut parts = vec![stage.arena_mut().add(axis)];
    for pct in (0..=LINE_TOP_PERCENT).step_by(2) {
        let x = line_x(Q::new(i128::from(pct), 100));
        let tick = polyline(vec![v(x, LINE_Y - 0.08), v(x, LINE_Y + 0.08)])
            .style(line_style(Palette::dim(), 2.0))
            .build();
        parts.push(stage.arena_mut().add(tick));
        parts.push(kit.text(stage, &format!("{pct}%"), 20.0, Palette::dim(), v(x, LINE_Y - 0.3))?);
    }
    Ok(parts)
}

/// A result's mark on the share line: a dot and its count above it.
fn share_mark(kit: &Kit, stage: &mut Stage<'_>, q: Q, k: u32, color: Srgb) -> fmn::Result<[Mob; 2]> {
    let x = line_x(q);
    let dot = Dot::new().point(v(x, LINE_Y)).radius(0.09).color(color).build();
    let dot = stage.arena_mut().add(dot);
    let count = kit.text(stage, &k.to_string(), 20.0, color, v(x, LINE_Y + 0.3))?;
    Ok([dot, count])
}

/// One version's strip: a baseline and one bar per pair, its height the
/// pair's observed difference (up when the treated patient recovered
/// sooner, lavender like a lit pair; down and unlit otherwise). Returns the
/// baseline, every bar, and the bars that point up; detached.
fn strip(stage: &mut Stage<'_>, days: &Days, base: f64) -> fmn::Result<(Mob, Vec<Mob>, Vec<Mob>)> {
    let last = bar_x(days.pairs() - 1);
    let baseline = polyline(vec![v(STRIP_X0 - 0.25, base), v(last + 0.25, base)])
        .style(line_style(Palette::dim(), 2.0))
        .build();
    let baseline = stage.arena_mut().add(baseline);
    let (mut bars, mut up) = (Vec::new(), Vec::new());
    for (i, &d) in days.differences().iter().enumerate() {
        let h = DAY_H * d.unsigned_abs() as f64;
        let centre = if d > 0 { base + h / 2.0 } else { base - h / 2.0 };
        let style = if d > 0 {
            flat(treated(), 0.85)
        } else {
            line_style(Palette::dim(), 2.0)
        };
        let bar = Rectangle::new()
            .width(BAR_W)
            .height(h)
            .style(style)
            .build()?
            .moved_to(v(bar_x(i), centre));
        let bar = stage.arena_mut().add(bar);
        bars.push(bar);
        if d > 0 {
            up.push(bar);
        }
    }
    Ok((baseline, bars, up))
}

/// The strips' titles and the note under the first; detached.
fn strip_titles(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let mut titles = Vec::new();
    for (n, y) in STRIP_TITLE_Y.iter().enumerate() {
        titles.push(kit.text_left(
            stage,
            &format!("{VERSION} {}, {MADE_UP}", n + 1),
            22.0,
            Palette::ink(),
            v(ROW_X, *y),
        )?);
    }
    titles.push(kit.text_left(stage, BAR_NOTE, 18.0, Palette::dim(), v(ROW_X, NOTE_Y))?);
    Ok(titles)
}

/// Strip `n`'s facts column, read from its days: the count, the median's
/// heading, the median among the favouring pairs, the one-sided p; detached.
fn facts_column(kit: &Kit, stage: &mut Stage<'_>, n: usize, days: &Days, world: &[u64]) -> fmn::Result<[Mob; 4]> {
    let k = days.favouring();
    let count = format!("{k} {OF} {} {PAIRS_FAVOURING}", days.pairs());
    let median = days
        .median_sooner()
        .ok_or_else(|| super::draw::fail(format!("version {} has no favouring pair", n + 1)))?;
    let share = tail_share(world, k, Tail::AtLeast);
    Ok([
        kit.text_left(stage, &count, 20.0, treated(), info_at(n, 0))?,
        kit.text_left(
            stage,
            &format!("{MEDIAN_AMONG} {k}:"),
            20.0,
            Palette::ink(),
            info_at(n, 1),
        )?,
        kit.text_left(stage, &sooner_words(median), 20.0, Palette::ink(), info_at(n, 2))?,
        kit.text_left(
            stage,
            &format!("{P_SHORT} {}%", share.percent(2)),
            20.0,
            treated(),
            info_at(n, 3),
        )?,
    ])
}

/// The closing caption (05_10).
fn closing(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    kit.text(stage, CLOSING, 22.0, Palette::ink(), v(0.0, CAPTION_Y))
}

/// Chapter 05's last picture, rebuilt still, for chapter 06 to open on: the
/// study's tag, both strips with their titles, signs and facts columns, and
/// the closing caption. Returns everything it put on the stage.
pub(super) fn last_picture(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let s = study();
    let world = s.null_world();
    let mut all = vec![study_tag(kit, stage, s.pairs)?];
    all.extend(strip_titles(kit, stage)?);
    for (n, (days, base)) in constructed_days().iter().zip(STRIP_BASE).enumerate() {
        let (baseline, bars, _) = strip(stage, days, base)?;
        all.push(baseline);
        all.extend(bars);
        all.extend(sign_row(kit, stage, days, base)?);
        all.extend(facts_column(kit, stage, n, days, &world)?);
    }
    all.push(closing(kit, stage)?);
    for &m in &all {
        stage.add_to_scene(m)?;
    }
    Ok(all)
}

/// The signs under a strip, one per pair, read from its days; detached.
fn sign_row(kit: &Kit, stage: &mut Stage<'_>, days: &Days, base: f64) -> fmn::Result<Vec<Mob>> {
    let mut signs = Vec::new();
    for (i, &d) in days.differences().iter().enumerate() {
        let (sign, color) = if d > 0 {
            ("+", treated())
        } else {
            ("−", Palette::dim())
        };
        signs.push(kit.text(stage, sign, 20.0, color, v(bar_x(i), base + SIGN_DY))?);
    }
    Ok(signs)
}

impl SceneConstruct for WhatTwoPercentDoesNotSay {
    fn name(&self) -> &str {
        "05_what_two_percent_does_not_say"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let s = study();
        let world = s.null_world();
        let total: u64 = world.iter().sum();
        let (pairs, k) = (s.pairs, s.observed());
        // the imagined other result: one more pair the other way
        let near = k - 1;
        let (p, p_near) = (
            tail_share(&world, k, Tail::AtLeast),
            tail_share(&world, near, Tail::AtLeast),
        );
        let pic = last_picture(&kit, stage)?;
        let frame = pic.frame;

        // 05_1: two percent is a share of that world; the world and the
        // question asked in it, apart
        let line = say(&kit, stage, "05_1")?;
        let mut leaving = pic.ledger.clone();
        leaving.extend(pic.cards);
        leaving.push(pic.caption);
        fade_out_all(stage, &leaving, 0.5)?;
        line.wait_for(stage, "imaginary world")?;
        let world_a = kit.text_left(stage, WORLD_A, 20.0, Palette::ink(), v(ROW_X, WORLD_Y[0]))?;
        let world_b = kit.text_left(
            stage,
            &format!("{} {WORLD_B}", thousands(total)),
            20.0,
            Palette::ink(),
            v(ROW_X, WORLD_Y[1]),
        )?;
        let w = stage.get_width(world_a).max(stage.get_width(world_b));
        let world_box = outline(
            stage,
            (ROW_X - WORLD_PAD, WORLD_Y[0] + 0.27),
            (ROW_X + w + WORLD_PAD, WORLD_Y[1] - 0.27),
        )?;
        kit.allow_crossing(world_box);
        fade_in_all(stage, &[world_box, world_a, world_b], 0.5)?;
        let question = kit.text_left(
            stage,
            &format!("{QUESTION} {k} {OR_MORE}?"),
            20.0,
            treated(),
            v(ROW_X, QUESTION_Y),
        )?;
        fade_in_all(stage, &[question], 0.4)?;

        // 05_2: is two percent the chance that the treatment did nothing?
        say(&kit, stage, "05_2")?;
        think_pause(&kit, stage, ASK_AT, ASK_R)?;

        // 05_3: no: it was counted inside that world
        let line = say(&kit, stage, "05_3")?;
        line.wait_for(stage, "counted inside that world")?;
        pulse(stage, &[world_box], 1.05, 0.6)?;
        line.wait_for(stage, "can't be the chance")?;
        let not_null = kit.text_left(
            stage,
            &format!("{}% {NOT_NULL}", p.percent(0)),
            22.0,
            warn(),
            v(ROW_X, NOT_Y[0]),
        )?;
        fade_in_all(stage, &[not_null], 0.4)?;

        // 05_4: nor the chance the study is wrong
        let line = say(&kit, stage, "05_4")?;
        line.wait_for(stage, "the study is wrong")?;
        let not_wrong = kit.text_left(
            stage,
            &format!("{}% {NOT_WRONG}", p.percent(0)),
            22.0,
            warn(),
            v(ROW_X, NOT_Y[1]),
        )?;
        fade_in_all(stage, &[not_wrong], 0.4)?;

        // 05_5: an imagined other result, one more pair the other way: 14
        let line = say(&kit, stage, "05_5")?;
        fade_out_all(
            stage,
            &[
                world_box,
                world_a,
                world_b,
                question,
                not_null,
                not_wrong,
                pic.name,
                pic.p_value,
            ],
            0.5,
        )?;
        let study_row = result_row(&kit, stage, (THE_STUDY, k), &world, treated(), RESULT_Y[0])?;
        fade_in_all(stage, &study_row, 0.5)?;
        line.wait_for(stage, "one more pair")?;
        let mut imagined_tail = bar_overlays(stage, &frame, &world, &[near], imagined())?;
        let y = bracket_y(&frame, &world, near) + BRACKET_RISE;
        let words = format!("{IMAGINED} {near} {OR_MORE}");
        imagined_tail.extend(bracket(&kit, stage, &frame, (near, pairs), y, &words, imagined())?);
        fade_in_all(stage, &imagined_tail, 0.6)?;
        line.wait_for(stage, "Fourteen")?;
        let [near_words, near_fraction] = result_row(&kit, stage, (IMAGINED, near), &world, imagined(), RESULT_Y[1])?;
        fade_in_all(stage, &[near_words], 0.4)?;

        // 05_6: fourteen or more: about 5.8% of the same world
        let line = say(&kit, stage, "05_6")?;
        line.wait_for(stage, "about five point eight percent")?;
        fade_in_all(stage, &[near_fraction], 0.6)?;

        // 05_7: a 5% line between them is a convention for deciding
        let line = say(&kit, stage, "05_7")?;
        let axis = share_line(&kit, stage)?;
        fade_in_all(stage, &axis, 0.5)?;
        let study_mark = share_mark(&kit, stage, p, k, treated())?;
        let near_mark = share_mark(&kit, stage, p_near, near, imagined())?;
        fade_in_all(stage, &[study_mark, near_mark].concat(), 0.4)?;
        line.wait_for(stage, "five percent")?;
        let x5 = line_x(Q::new(i128::from(CONVENTION_PERCENT), 100));
        let rule = polyline(vec![v(x5, LINE_Y + 0.45), v(x5, LINE_Y - 0.15)])
            .style(line_style(Palette::ink(), 3.0))
            .build();
        let rule = stage.arena_mut().add(rule);
        let rule_words = kit.text(
            stage,
            &format!("{CONVENTION_PERCENT}%: {CONVENTION}"),
            20.0,
            Palette::ink(),
            v(x5, LINE_Y - 0.72),
        )?;
        fade_in_all(stage, &[rule, rule_words], 0.5)?;
        line.wait_for(stage, "Fifteen falls under it")?;
        pulse(stage, &study_mark, 1.4, 0.5)?;
        line.wait_for(stage, "fourteen over")?;
        pulse(stage, &near_mark, 1.4, 0.5)?;
        line.wait_for(stage, "That line helps decide")?;
        let moved = [
            kit.text_left(
                stage,
                &format!("{} {MOVED}", k - near),
                22.0,
                Palette::ink(),
                v(ROW_X, MOVED_Y[0]),
            )?,
            kit.text_left(
                stage,
                &format!("{NO_SWITCH} {CONVENTION_PERCENT}%"),
                22.0,
                Palette::ink(),
                v(ROW_X, MOVED_Y[1]),
            )?,
        ];
        fade_in_all(stage, &moved, 0.5)?;

        // 05_8: the count also ignored how much sooner: two made-up versions
        let line = say(&kit, stage, "05_8")?;
        let mut wipe = pic.hist.clone();
        wipe.extend(&imagined_tail);
        wipe.extend(study_row);
        wipe.extend([near_words, near_fraction, rule, rule_words]);
        wipe.extend(&axis);
        wipe.extend(study_mark);
        wipe.extend(near_mark);
        wipe.extend(moved);
        fade_out_all(stage, &wipe, 0.6)?;
        let versions = constructed_days();
        let titles = strip_titles(&kit, stage)?;
        fade_in_all(stage, &titles, 0.4)?;
        line.wait_for(stage, "two made-up sets")?;
        let (mut baselines, mut bars, mut up) = (Vec::new(), Vec::new(), Vec::new());
        for (days, base) in versions.iter().zip(STRIP_BASE) {
            let (baseline, all, sooner) = strip(stage, days, base)?;
            baselines.push(baseline);
            bars.extend(all);
            up.push(sooner);
        }
        fade_in_all(stage, &baselines, 0.3)?;
        fade_in_lagged(stage, &bars, 1.4, 0.04)?;

        // 05_9: the same fifteen favouring pairs; among those, sizes far apart
        let line = say(&kit, stage, "05_9")?;
        line.wait_for(stage, "same fifteen favouring pairs")?;
        let mut signs = Vec::new();
        let mut columns = Vec::new();
        for (n, (days, base)) in versions.iter().zip(STRIP_BASE).enumerate() {
            signs.extend(sign_row(&kit, stage, days, base)?);
            columns.push(facts_column(&kit, stage, n, days, &world)?);
        }
        fade_in_lagged(stage, &signs, 1.0, 0.03)?;
        fade_in_all(stage, &[columns[0][0], columns[1][0]], 0.4)?;
        line.wait_for(stage, "Among those")?;
        fade_in_all(stage, &[columns[0][1], columns[1][1]], 0.4)?;
        for (n, phrase) in ["a day sooner", "a week in the other"].into_iter().enumerate() {
            line.wait_for(stage, phrase)?;
            fade_in_all(stage, &[columns[n][2]], 0.4)?;
            pulse(stage, &up[n], 1.08, 0.5)?;
        }

        // 05_10: same signs, same count, same two percent; never how much
        let line = say(&kit, stage, "05_10")?;
        line.wait_for(stage, "same two percent")?;
        fade_in_all(stage, &[columns[0][3], columns[1][3]], 0.4)?;
        line.wait_for(stage, "never asked how much sooner")?;
        pulse(stage, &up.concat(), 1.08, 0.6)?;
        line.wait_for(stage, "A patient would")?;
        let caption = closing(&kit, stage)?;
        fade_in_all(stage, &[caption], 0.5)?;
        kit.hold(stage)?;
        kit.music_breath(stage, 0.8)?;
        // chapter 06 opens on this picture as it rests (`last_picture`)
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What the chapter shows is computed: both results' inclusive tails,
    /// the 5% line between them, each version's count read from its days,
    /// the same tail for both versions, and medians among the favouring pairs
    /// only (1 and 7 days).
    #[test]
    fn the_pictures_are_the_models_numbers() {
        let s = study();
        let world = s.null_world();
        let (k, near) = (s.observed(), s.observed() - 1);
        assert_eq!(tail_count(&world, near, Tail::AtLeast), 60_460);
        assert_eq!(tail_count(&world, k, Tail::AtLeast), 21_700);
        let line = Q::new(i128::from(CONVENTION_PERCENT), 100);
        assert!(tail_share(&world, k, Tail::AtLeast) < line && line < tail_share(&world, near, Tail::AtLeast));
        let versions = constructed_days();
        for days in &versions {
            assert_eq!(days.favouring(), k);
            assert_eq!(days.faster_a(s.treated_a), s.faster_a);
            assert_eq!(tail_share(&world, days.favouring(), Tail::AtLeast).percent(2), "2.07");
        }
        let medians: Vec<String> = versions
            .iter()
            .map(|d| sooner_words(d.median_sooner().unwrap()))
            .collect();
        assert_eq!(medians, vec!["1 day sooner", "7 days sooner"]);
        // the median is among the favouring pairs: the median of all twenty
        // differences is another number, and the caption names the 15
        let mut all = versions[1].differences();
        all.sort_unstable();
        assert_ne!(Q::new(i128::from(all[9] + all[10]), 2), Q::int(7));
        assert!(MEDIAN_AMONG.contains("among those"));
        assert!(BAR_NOTE.contains("not a treatment effect") && MADE_UP.contains("not trial data"));
    }

    /// The words never turn the share into a probability of the world or of
    /// an error, and call the 5% line a convention.
    #[test]
    fn the_words_keep_the_world_and_the_question_apart() {
        assert!(WORLD_A.starts_with("assumed world"));
        assert!(QUESTION.starts_with("asked in it"));
        assert!(NOT_NULL.starts_with("is not") && NOT_WRONG.starts_with("is not"));
        assert!(CONVENTION.contains("convention"));
        for words in [WORLD_A, WORLD_B, QUESTION, CONVENTION, MOVED, NO_SWITCH, CLOSING] {
            assert!(
                !words.contains("probability") && !words.contains("likely true"),
                "{words}"
            );
        }
    }

    /// The layout: the left column stacks without touching and stays left
    /// of the histogram; the share line fits its column; the strips, their
    /// signs and their facts column stack without touching.
    #[test]
    fn the_pieces_stack_without_collisions() {
        // a line of text's half height and width by size, generously
        let half = |size: f64| 0.0068 * size;
        let width = |s: &str, size: f64| s.chars().count() as f64 * 0.0058 * size;
        const FRACTION_HALF: f64 = 0.45;
        let s = study();
        let hist_left = super::super::imaginary_world::hist_frame(&s.null_world()).left();
        // 05_1–05_4: the world box, the question, chapter 04's name and
        // fraction (at -0.6 and -1.45), the ring, the readings
        assert!(WORLD_Y[1] - 0.27 > QUESTION_Y + half(20.0) + 0.1);
        assert!(QUESTION_Y - half(20.0) > -0.6 + half(26.0) + 0.1);
        const {
            assert!(-1.45 - FRACTION_HALF > ASK_AT[1] + ASK_R + 0.05);
            assert!(RESULT_Y[0] - RESULT_Y[1] > 2.0 * FRACTION_HALF + 0.1);
            assert!(RESULT_Y[1] - FRACTION_HALF > LINE_Y + 0.45 + 0.1);
        }
        assert!(NOT_Y[0] - NOT_Y[1] > 2.0 * half(22.0) + 0.05);
        let world_b = format!("1,048,576 {WORLD_B}");
        for (words, size) in [
            (WORLD_A, 20.0),
            (world_b.as_str(), 20.0),
            ("asked in it: how often 15 or more?", 20.0),
            ("2% is not the chance the treatment did nothing", 22.0),
            ("1 pair apart: the evidence moved a little", 22.0),
        ] {
            assert!(ROW_X + width(words, size) + WORLD_PAD < hist_left - 0.3, "{words}");
        }
        // 05_5–05_7: the two results, the share line, the convention's words
        assert!(FRACTION_X + 1.6 < hist_left - 0.3);
        let end = line_x(Q::new(i128::from(LINE_TOP_PERCENT), 100));
        assert!(end < hist_left - 0.3 && LINE_X0 > -7.1 + 0.25);
        assert!(LINE_Y - 0.72 - half(20.0) > MOVED_Y[0] + half(22.0) + 0.05);
        assert!(MOVED_Y[1] - half(22.0) > -4.0 + 0.25);
        // 05_8–05_10: strips, signs, titles, the facts column, the caption
        let versions = constructed_days();
        let tallest = |d: &Days| d.differences().iter().copied().max().unwrap_or(0) as f64 * DAY_H;
        let deepest = |d: &Days| -(d.differences().iter().copied().min().unwrap_or(0) as f64) * DAY_H;
        for (n, d) in versions.iter().enumerate() {
            let base = STRIP_BASE[n];
            assert!(base + tallest(d) < STRIP_TITLE_Y[n] - half(22.0) - 0.1, "strip {n}");
            assert!(base - deepest(d) > base + SIGN_DY + half(20.0) + 0.05, "strip {n}");
            assert!(bar_x(d.pairs() - 1) + 0.25 < INFO_X - 0.3);
        }
        assert!(STRIP_BASE[0] + SIGN_DY - half(20.0) > STRIP_TITLE_Y[1] + half(22.0) + 0.1);
        assert!(info_at(0, 3)[1] - half(20.0) > info_at(1, 0)[1] + half(20.0) + 0.1);
        assert!(info_at(1, 3)[1] - half(20.0) > CAPTION_Y + half(22.0) + 0.1);
        assert!(NOTE_Y - half(18.0) > INFO_TOP[0] + half(20.0) + 0.05);
        assert!(STRIP_TITLE_Y[0] - half(22.0) > NOTE_Y + half(18.0));
        assert!(STRIP_BASE[1] + SIGN_DY - half(20.0) > CAPTION_Y + half(22.0) + 0.1);
        assert!(INFO_X + width("15 of 20 pairs favouring", 20.0) < 7.1 - 0.25);
        assert!(INFO_X + width("median among those 15:", 20.0) < 7.1 - 0.25);
        assert!(width(CLOSING, 22.0) / 2.0 < 7.1 - 0.25);
        assert!(ROW_X + width(BAR_NOTE, 18.0) < 7.1 - 0.25);
    }
}
