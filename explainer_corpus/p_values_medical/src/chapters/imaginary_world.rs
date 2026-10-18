//! Chapter 03, the imaginary world: the twenty pairs again, and in each the
//! patient the coin treated (03_1); what counts as good news, decided before
//! looking (03_2); the sharp null in words (03_3); only then what happened,
//! fifteen pairs favouring the treatment (03_4); with the finishing order
//! held fixed the coins are tossed again, four times, each toss a real
//! assignment and its own count (03_5, 03_6); all 2^20 assignments counted
//! into a histogram, the tosses and the study's fifteen on their bars
//! (03_7); what this world is and is not (03_8); and the question how often
//! fifteen or more happens (03_9, a prediction the next chapter answers).
//!
//! Everything drawn is the model's study (`model::study`): who recovered
//! first and who was treated are bits, an order and a label. No recovery
//! time is drawn or said, because the model has none.

use explainer_kit::props::{grow_histogram, histogram, precisely};
use explainer_kit::{Frame2, Kit, Palette, Timed, thousands, tween, v};
use fmn::prelude::*;

use super::draw::{card, fade_in_all, fade_in_lagged, fade_out_all, line_style, say, think_pause, treated};
use super::five_percent::outline;
use super::same_test::{
    DISC_DX, DISC_R, NULL_WORLD, NULL_Y, PAIR_DY, PAIR_H, PAIR_TOP, PAIR_W, STUDY_A, STUDY_B, STUDY_TAG_Y, pair_at,
    study_pair,
};
use crate::facts::{RETOSS_SEED, RETOSSES};
use crate::model::{Study, study};

pub struct TheImaginaryWorld {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const TREATED_BY_COIN: &str = "treated, by the coin";
const FIRST: &str = "recovered first";
const RULE: &str = "favours the treatment: its treated patient recovered first (decided before looking)";
const SHARP_NULL: &str = "sharp null: each patient's recovery would be the same treated or untreated, and no patient \
                          affects another. A zero average effect alone would not give this world.";
const FAVOURING: &str = "pairs favouring the treatment:";
const TOSS: &str = "toss";
const WAYS: &str = "ways the coins can land,";
const EQUALLY: &str = "each equally likely in this world";
const AXIS: &str = "pairs favouring the treatment";
const DOTS: &str = "dots: the tosses";
const THE_STUDY: &str = "the study:";
const THESE_PAIRS: &str = "these pairs: the last re-toss,";
const FAVOURING_SHORT: &str = "favouring";
const THE_SAME: &str = "the same";
const PATIENTS_IN: &str = "patients in";
const NOT_NEW: &str = "pairs, the coins tossed again: not new patients. This world assumes the treatment did \
                       nothing; it does not say how likely that is.";

// --- layout (scene units; the frame is 14.2 by 8)

/// The legend's line, between the study's tag and the pairs.
const LEGEND_Y: f64 = 2.72;
/// The rule's caption, under the pairs.
const RULE_Y: f64 = -1.95;
/// The row of tosses and their counts.
const TOSSES_Y: f64 = -3.2;
const TOSS_X0: f64 = -6.0;
const TOSS_DX: f64 = 3.1;
/// How far under a patient the "recovered first" bar sits.
const BAR_DY: f64 = 0.3;
/// The pairs' place in 03_7: the left half, scaled.
const SMALL: f64 = 0.55;
const SMALL_AT: Vec3 = [-3.9, 0.45, 0.0];
/// The small pairs' label ("these pairs: the last re-toss"), above them.
const SHOWN_Y: f64 = 2.05;
/// The histogram's window, the right half (k = 0..20 across).
const HIST: Frame2 = Frame2 {
    center: [3.55, -0.25, 0.0],
    width: 6.0,
    height: 3.3,
    x: (-0.5, 20.5),
    y: (0.0, 1.0),
};
/// The prediction's ring, under the small pairs.
const ASK_AT: Vec3 = [-3.9, -2.0, 0.0];
const ASK_R: f64 = 0.5;

/// Patient A's or B's disc in pair `i`.
fn disc_at(i: usize, a: bool) -> Vec3 {
    let at = pair_at(i);
    v(if a { at[0] - DISC_DX } else { at[0] + DISC_DX }, at[1])
}

/// The centre of the twenty pairs as chapter 02 left them.
fn grid_centre() -> Vec3 {
    v(0.0, PAIR_TOP - 1.5 * PAIR_DY)
}

/// Where a point of the full-size pairs goes when they move to the left half.
fn small(p: Vec3) -> Vec3 {
    let c = grid_centre();
    v(SMALL_AT[0] + (p[0] - c[0]) * SMALL, SMALL_AT[1] + (p[1] - c[1]) * SMALL)
}

/// The ring on a treated patient.
fn ring(stage: &mut Stage<'_>, at: Vec3) -> Mob {
    let r = Circle::new()
        .radius(DISC_R + 0.07)
        .style(line_style(treated(), 4.0))
        .build()
        .moved_to(at);
    stage.arena_mut().add(r)
}

/// The bar under the patient who recovered first.
fn first_bar(stage: &mut Stage<'_>, disc: Vec3) -> Mob {
    let y = disc[1] - BAR_DY;
    let b = polyline(vec![v(disc[0] - 0.17, y), v(disc[0] + 0.17, y)])
        .style(line_style(Palette::ink(), 4.0))
        .build();
    stage.arena_mut().add(b)
}

/// A pair lit because its treated patient recovered first.
fn lit_box(stage: &mut Stage<'_>, at: Vec3) -> fmn::Result<Mob> {
    let r = Rectangle::new()
        .width(PAIR_W)
        .height(PAIR_H)
        .corner_radius(0.12)
        .style(Style::default().fill(treated(), 0.14).stroke(treated(), 4.0, 1.0))
        .build()?
        .moved_to(at);
    Ok(stage.arena_mut().add(r))
}

/// One entry of the row of tosses: "toss 2: 12".
fn toss_entry(kit: &Kit, stage: &mut Stage<'_>, n: usize, k: u32) -> fmn::Result<Mob> {
    let at = v(TOSS_X0 + TOSS_DX * n as f64, TOSSES_Y);
    kit.text_left(stage, &format!("{TOSS} {}: {k}", n + 1), 22.0, Palette::ink(), at)
}

/// The move of one mark of the full-size pairs (centred at `centre`) to its
/// place among the small pairs of the left half, prepared.
fn to_small(stage: &mut Stage<'_>, m: Mob, centre: Vec3) -> fmn::Result<Box<dyn Animation>> {
    let a = m
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(1.0),
            ..AnimateArgs::default()
        })?
        .scale(SMALL)?
        .move_to(small(centre), ORIGIN)?;
    stage.prepare(a)
}

/// The pairs' marks as they stand: one ring per pair (on the treated
/// patient of `assignment`), one lit box per favouring pair.
struct Marks {
    rings: Vec<Mob>,
    /// The "recovered first" bars, once they are shown (03_4).
    bars: Vec<Mob>,
    lit: Vec<Option<Mob>>,
    assignment: u32,
}

/// Toss the coins again: every ring whose coin landed the other way jumps to
/// the pair's other patient, the lit boxes follow the new favouring pairs,
/// the count moves to the new count; the finishing order never moves.
fn retoss(stage: &mut Stage<'_>, s: &Study, marks: &mut Marks, next: u32, tally: Mob, rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for i in 0..s.pairs {
        let p = i as usize;
        if s.treated_is_a(next, i) != s.treated_is_a(marks.assignment, i) {
            let jump = marks.rings[p]
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(rt),
                    ..AnimateArgs::default()
                })?
                .move_to(disc_at(p, s.treated_is_a(next, i)), ORIGIN)?;
            anims.push(stage.prepare(jump)?);
        }
        match (marks.lit[p], s.favours(next, i)) {
            (Some(m), false) => {
                let anim = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
                anims.push(stage.prepare(anim.rt(rt))?);
                marks.lit[p] = None;
            }
            (None, true) => {
                let m = lit_box(stage, pair_at(p))?;
                let anim = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
                anims.push(stage.prepare(anim.rt(rt))?);
                marks.lit[p] = Some(m);
            }
            _ => {}
        }
    }
    anims.push(stage.prepare(tween(tally, f64::from(s.favouring(next)), rt)?)?);
    stage.play_prepared(anims)?;
    // a newly lit box lands on top: keep the rings and the bars above it
    stage.scene_mut().bring_to_front(&marks.rings)?;
    stage.scene_mut().bring_to_front(&marks.bars)?;
    marks.assignment = next;
    Ok(())
}

impl SceneConstruct for TheImaginaryWorld {
    fn name(&self) -> &str {
        "03_the_imaginary_world"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let s = study();
        let pairs = s.pairs as usize;
        let world = s.null_world();
        let total: u64 = world.iter().sum();
        let tosses = s.retosses(RETOSS_SEED, RETOSSES);

        // chapter 02's last picture, as it rests
        let tag = kit.text(
            stage,
            &format!("{STUDY_A} {pairs} {STUDY_B}"),
            22.0,
            Palette::dim(),
            v(0.0, STUDY_TAG_Y),
        )?;
        let mut boxes = Vec::with_capacity(pairs);
        for i in 0..pairs {
            boxes.push(study_pair(stage, i)?);
        }
        let null = kit.text(stage, NULL_WORLD, 28.0, Palette::ink(), v(0.0, NULL_Y))?;
        for &m in [tag, null].iter().chain(&boxes) {
            stage.add_to_scene(m)?;
        }

        // 03_1: in each pair a fair coin chose who got the treatment
        let line = say(&kit, stage, "03_1")?;
        let swatch = ring(stage, v(-6.45, LEGEND_Y));
        let swatch_words = kit.text_left(stage, TREATED_BY_COIN, 20.0, Palette::dim(), v(-6.15, LEGEND_Y))?;
        fade_in_all(stage, &[swatch, swatch_words], 0.5)?;
        line.wait_for(stage, "a fair coin")?;
        let rings: Vec<Mob> = (0..s.pairs)
            .map(|i| ring(stage, disc_at(i as usize, s.treated_is_a(s.treated_a, i))))
            .collect();
        fade_in_lagged(stage, &rings, 1.6, 0.12)?;
        let mut marks = Marks {
            rings,
            bars: Vec::new(),
            lit: vec![None; pairs],
            assignment: s.treated_a,
        };

        // 03_2: what counts as good news, decided before looking
        let line = say(&kit, stage, "03_2")?;
        line.wait_for(stage, "A pair favours")?;
        let rule = kit.text(stage, RULE, 22.0, Palette::ink(), v(0.0, RULE_Y))?;
        fade_in_all(stage, &[rule], 0.5)?;

        // 03_3: the sharp null, in words, before any outcome
        say(&kit, stage, "03_3")?;
        let null_box = outline(stage, (-4.6, NULL_Y + 0.25), (4.6, NULL_Y - 0.25))?;
        kit.allow_crossing(null_box);
        let lower = precisely(&kit, stage, SHARP_NULL)?;
        fade_in_all(stage, &[null_box, lower], 0.6)?;

        // 03_4: what happened: fifteen of the twenty pairs favour the treatment
        let line = say(&kit, stage, "03_4")?;
        fade_out_all(stage, &[lower, null_box], 0.4)?;
        let bars: Vec<Mob> = (0..s.pairs)
            .map(|i| first_bar(stage, disc_at(i as usize, s.faster_is_a(i))))
            .collect();
        let bar_swatch = first_bar(stage, v(-2.45, LEGEND_Y + BAR_DY));
        let bar_words = kit.text_left(stage, FIRST, 20.0, Palette::dim(), v(-2.15, LEGEND_Y))?;
        let mut first: Vec<Mob> = vec![bar_swatch, bar_words];
        first.extend(&bars);
        fade_in_lagged(stage, &first, 1.2, 0.06)?;
        marks.bars = bars;
        line.wait_for(stage, "fifteen")?;
        let tally = stage.add_value_tracker(0.0);
        let tally_words = kit.text_left(stage, FAVOURING, 22.0, Palette::dim(), v(2.4, LEGEND_Y))?;
        let tally_number = kit.live_number(stage, tally, 0, false, 26.0, treated(), v(6.7, LEGEND_Y), RIGHT)?;
        let mut lit_now = Vec::new();
        for i in 0..s.pairs {
            if s.favours(s.treated_a, i) {
                let m = lit_box(stage, pair_at(i as usize))?;
                marks.lit[i as usize] = Some(m);
                lit_now.push(m);
            }
        }
        fade_in_all(stage, &[tally_words], 0.3)?;
        let mut both: Vec<Box<dyn Animation>> = vec![stage.prepare(tween(tally, f64::from(s.observed()), 1.0)?)?];
        let mut fades: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &lit_now {
            let anim = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            fades.push(stage.prepare(anim)?);
        }
        both.push(Box::new(
            AnimationGroup::with_lag_ratio(stage.arena_mut(), fades, 0.05)?.with_run_time(1.0),
        ));
        stage.play_prepared(both)?;
        stage.scene_mut().bring_to_front(&marks.rings)?;
        stage.scene_mut().bring_to_front(&marks.bars)?;

        // 03_5: the finishing order stays fixed; toss the coins again
        let line = say(&kit, stage, "03_5")?;
        line.wait_for(stage, "stays fixed")?;
        let mut pulse: Vec<Box<dyn Animation>> = Vec::new();
        for &b in &marks.bars {
            let anim = fmn::animation::indicate(stage.arena_mut(), b, 1.3, None)?;
            pulse.push(stage.prepare(anim.rt(0.6))?);
        }
        stage.play_prepared(pulse)?;
        line.wait_for(stage, "Toss them again")?;
        let mut entries = Vec::new();
        retoss(stage, &s, &mut marks, tosses[0], tally, 0.6)?;
        let e = toss_entry(&kit, stage, 0, s.favouring(tosses[0]))?;
        entries.push(e);
        fade_in_all(stage, &[e], 0.3)?;

        // 03_6: and again, and again: each toss its own count
        let line = say(&kit, stage, "03_6")?;
        for (n, phrase) in [(1, None), (2, Some("and again.")), (3, Some("Each toss"))] {
            if let Some(phrase) = phrase {
                line.wait_for(stage, phrase)?;
            }
            retoss(stage, &s, &mut marks, tosses[n], tally, 0.5)?;
            let e = toss_entry(&kit, stage, n, s.favouring(tosses[n]))?;
            entries.push(e);
            fade_in_all(stage, &[e], 0.25)?;
        }

        // 03_7: every way the coins can land, counted
        let line = say(&kit, stage, "03_7")?;
        let mut moves: Vec<Box<dyn Animation>> = Vec::new();
        for i in 0..s.pairs {
            let p = i as usize;
            moves.push(to_small(stage, boxes[p], pair_at(p))?);
            let ring_at = disc_at(p, s.treated_is_a(marks.assignment, i));
            moves.push(to_small(stage, marks.rings[p], ring_at)?);
            let under = disc_at(p, s.faster_is_a(i));
            moves.push(to_small(stage, marks.bars[p], v(under[0], under[1] - BAR_DY))?);
            if let Some(m) = marks.lit[p] {
                moves.push(to_small(stage, m, pair_at(p))?);
            }
        }
        // the small pairs still show the last re-toss: say so, with its own
        // count, so they are never read as the study's fifteen
        let shown = kit.text(
            stage,
            &format!("{THESE_PAIRS} {} {FAVOURING_SHORT}", s.favouring(marks.assignment)),
            20.0,
            Palette::dim(),
            v(SMALL_AT[0], SHOWN_Y),
        )?;
        let anim = fade_in(stage.arena_mut(), shown, ORIGIN, 1.0)?;
        moves.push(stage.prepare(anim.rt(0.6))?);
        // the rule, the caption, the count and the row of tosses make room
        let mut leaving = vec![rule, null, tally_words, tally_number];
        leaving.extend(&entries);
        for m in leaving {
            stage.clear_updaters(m, true);
            let anim = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            moves.push(stage.prepare(anim.rt(0.6))?);
        }
        stage.play_prepared(moves)?;
        line.wait_for(stage, "just over a million")?;
        let top = world.iter().copied().max().unwrap_or(1);
        let frame = Frame2 {
            y: (0.0, top as f64),
            ..HIST
        };
        let title = kit.text(
            stage,
            &format!("{} {WAYS}", thousands(total)),
            22.0,
            Palette::ink(),
            v(frame.center[0], 2.25),
        )?;
        let title_2 = kit.text(stage, EQUALLY, 22.0, Palette::dim(), v(frame.center[0], 1.93))?;
        let axis = polyline(vec![v(frame.left(), frame.bottom()), v(frame.right(), frame.bottom())])
            .style(line_style(Palette::dim(), 2.0))
            .build();
        let axis = stage.arena_mut().add(axis);
        let mut furniture = vec![title, title_2, axis];
        for k in (0..=s.pairs).step_by(5) {
            let x = frame.p(f64::from(k), 0.0)[0];
            furniture.push(kit.text(stage, &k.to_string(), 20.0, Palette::dim(), v(x, frame.bottom() - 0.25))?);
        }
        furniture.push(kit.text(
            stage,
            AXIS,
            20.0,
            Palette::dim(),
            v(frame.center[0], frame.bottom() - 0.62),
        )?);
        fade_in_all(stage, &furniture, 0.5)?;
        line.wait_for(stage, "all equally likely")?;
        let heights: Vec<(f64, f64)> = world.iter().enumerate().map(|(k, &c)| (k as f64, c as f64)).collect();
        let hist = stage.arena_mut().add(histogram(&frame, &heights, 0.8, Palette::dim())?);
        stage.add_to_scene(hist)?;
        let grow = grow_histogram(stage, hist, 0.04)?;
        stage.play(grow.with_run_time(1.6))?;
        // the tallest bar's count, the four tosses as dots on their bars, the
        // study's fifteen marked on its bar
        let mode = world.iter().position(|&c| c == top).unwrap_or(0);
        let tallest = kit.text(
            stage,
            &thousands(top),
            18.0,
            Palette::dim(),
            v(frame.p(mode as f64, 0.0)[0], frame.top() + 0.15),
        )?;
        let mut marks_on_bars = vec![tallest];
        let mut stacked = vec![0usize; pairs + 1];
        for &a in &tosses {
            let k = s.favouring(a) as usize;
            let foot = frame.p(k as f64, world[k] as f64);
            let at = v(foot[0], foot[1] + 0.12 + 0.14 * stacked[k] as f64);
            stacked[k] += 1;
            let d = Dot::new().point(at).radius(0.05).color(Palette::ink()).build();
            marks_on_bars.push(stage.arena_mut().add(d));
        }
        marks_on_bars.push(kit.text_left(stage, DOTS, 18.0, Palette::dim(), v(4.9, 1.0))?);
        let k_obs = s.observed() as usize;
        let foot = frame.p(k_obs as f64, world[k_obs] as f64);
        let obs_dot = Dot::new()
            .point(v(foot[0], foot[1] + 0.14))
            .radius(0.07)
            .color(treated())
            .build();
        marks_on_bars.push(stage.arena_mut().add(obs_dot));
        // beside its dot, over the near-empty bars above 15 (not over bar 14)
        marks_on_bars.push(kit.text_left(
            stage,
            &format!("{THE_STUDY} {k_obs}"),
            20.0,
            treated(),
            v(foot[0] + 0.15, foot[1] + 0.14),
        )?);
        fade_in_all(stage, &marks_on_bars, 0.5)?;

        // 03_8: the same patients, the coins tossed again; it assumes, it does not measure
        say(&kit, stage, "03_8")?;
        let people = 2 * pairs;
        let note = card(
            &kit,
            stage,
            None,
            &format!("{THE_SAME} {people} {PATIENTS_IN} {pairs} {NOT_NEW}"),
            None,
            (20.0, 52),
            Palette::ink(),
            6.2,
            v(ASK_AT[0] + 0.05, ASK_AT[1] - 0.05),
        )?;
        fade_in_all(stage, &[note], 0.5)?;

        // 03_9: how often do fifteen or more favour the treatment?
        say(&kit, stage, "03_9")?;
        fade_out_all(stage, &[note], 0.4)?;
        think_pause(&kit, stage, ASK_AT, ASK_R)?;
        // chapter 04 opens on the histogram and its fifteen, as they rest
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The drawing is the model: each ring sits on the patient the coin
    /// treated, each bar under the patient who recovered first, and a pair
    /// is lit exactly when the two coincide; the lit pairs are the count.
    #[test]
    fn the_marks_are_the_models_bits() {
        let s = study();
        let lit: Vec<u32> = (0..s.pairs).filter(|&i| s.favours(s.treated_a, i)).collect();
        assert_eq!(lit.len() as u32, s.observed());
        for i in 0..s.pairs {
            let ring_at = disc_at(i as usize, s.treated_is_a(s.treated_a, i));
            let bar_at = disc_at(i as usize, s.faster_is_a(i));
            assert_eq!(ring_at == bar_at, s.favours(s.treated_a, i), "pair {i}");
        }
        // every re-toss moves only rings, never bars, and its count is its lit pairs
        for a in s.retosses(RETOSS_SEED, RETOSSES) {
            let lit = (0..s.pairs).filter(|&i| s.favours(a, i)).count() as u32;
            assert_eq!(lit, s.favouring(a));
        }
        // the small pairs in 03_7 show the last re-toss, labelled with its own
        // count, which is not the study's
        let tosses = s.retosses(RETOSS_SEED, RETOSSES);
        let last = s.favouring(tosses[RETOSSES - 1]);
        assert_eq!(last, 12);
        assert_ne!(last, s.observed());
        // "just over a million"
        let total: u64 = s.null_world().iter().sum();
        assert!(total > 1_000_000 && total < 1_100_000, "{total}");
    }

    /// The layout: the small pairs stay in the left half and clear the card;
    /// the histogram stays in the right half under the legend.
    #[test]
    fn the_two_halves_do_not_collide() {
        for i in 0..20 {
            let p = small(pair_at(i));
            assert!(
                p[0] - PAIR_W * SMALL / 2.0 > -7.0 && p[0] + PAIR_W * SMALL / 2.0 < HIST.left() - 0.2,
                "{i}"
            );
            assert!(p[1] - PAIR_H * SMALL / 2.0 > ASK_AT[1] + 0.9, "{i}");
        }
        let top = HIST.center[1] + HIST.height / 2.0;
        assert!(top + 0.9 < LEGEND_Y && HIST.right() < 6.9);
        // the last re-toss's label sits between the legend and the small pairs
        let grid_top = small(pair_at(0))[1] + PAIR_H * SMALL / 2.0;
        assert!(grid_top + 0.2 < SHOWN_Y && SHOWN_Y + 0.35 < LEGEND_Y, "{grid_top}");
        assert!(ASK_AT[0] + ASK_R + 0.2 + 1.9 < HIST.left());
    }
}
