//! Chapter 04, one count, one placement. From 03's title card the card
//! comes back with the sheet at the first placement and a live count of 7.
//! Will sliding keep it at 7? (04_1, a prediction.) The sheet slides 1.5 cm
//! up, slowly through the brief 8, and a log under the count keeps what it
//! read, 7, 9, 8, 7 (04_2). An eighth of a turn about the card's middle,
//! then the slide that keeps the lines clearest of the coast's turning
//! points: 8 (04_3, 04_4). A count belongs to a placement, two choices,
//! shown as two controls bound to the sheet (04_5). One gap's slide (04_6,
//! a prediction) puts every line where its neighbour was: 8 again, so only
//! slides within a gap differ (04_7). Half a turn about a point midway
//! between two lines puts them back in the same places, so only directions
//! within half a turn differ (04_8, 04_9). Back to the first placement, its
//! seven crossings ringed (04_10); the plan, every slide and every
//! direction, averaged (04_11). Every line, dot, count, knob and needle is
//! drawn from the model at the placement shown, every frame. Starts where
//! chapter 03 ends. Not registered yet: root wires it in
//! (`docs/candidate-04-05.md`).

use std::f64::consts::PI;

use explainer_kit::{Kit, Palette, tween, v};
use fmn::core::rate::{linear, smooth};
use fmn::prelude::*;

use super::draw::{
    EVERY, card_panel, card_point, dim_line, fade_in_all, fade_out_all, fade_to, fail, flat, line_style, now, path,
    say, stand, stop, think, together,
};
use super::glyphs::TITLE;
use super::placement::{
    self, COUNT_RIGHT, CROSSINGS, Controls, Move, Placement, Pose, QUESTION, QUESTION_AT, Run, WORD_GAP, at_once, widen,
};
use super::sheet::{self, COUNT_AT, COUNT_SIZE, LABEL_SIZE, Sheet, THINK_AT, TOP_Y, count_words};
use crate::facts;
use crate::model::{self, Curve, GAP, P2};
use crate::palette;

pub struct OneCountOnePlacement {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model and facts);
// 03's arrow, as 03 left it beside the count
const ARROW: &str = r"\rightarrow\ ?\text{ cm}";
const ASK: &str = "?";

/// What the voice says the model shows: the slide's counts in order
/// (04_2, the oracle's `slide.sequence`), the count after the eighth of a
/// turn and after each repeat (04_4, 04_7, 04_9, `turn.45.crossings`), and
/// the first count (04_10, `open.crossings`).
const SAID_SLIDE: [usize; 4] = [7, 9, 8, 7];
const SAID_TURNED: usize = 8;
const SAID_FIRST: usize = 7;

// ------------------------------------------------------------------ layout

const COAST_W: f64 = 4.0;
const PICKED_W: f64 = 8.0;
/// What 03's title card dimmed the card and everything on it to, and the
/// card's own opacity (`card_panel`'s) it comes back to.
const DIMMED: f64 = 0.25;
const CARD_OPACITY: f64 = 0.55;
const TITLE_AT: Vec3 = [0.0, 0.25, 0.0];
const TITLE_SIZE: f64 = 50.0;
/// The log of counts read during 04_1's slide, in the band over the coast
/// to the count's right, frame units, and the ring round one entry.
const LOG_X: f64 = -0.5;
const LOG_STEP: f64 = 0.5;
const LOG_SIZE: f64 = 26.0;
const RING_R: f64 = 0.22;
/// 04_10's rings round the first placement's crossings, frame units.
const CROSSING_RING_R: f64 = 0.17;
/// 04_6's bracket: from line 1 of the turned sheet (at 3.20 cm) to where
/// line 2 lies, 2.4 cm left of line 1's point nearest the card's middle,
/// over the empty card above the cove; its tick half-length and how far
/// along the lines its "2 cm" stands from its middle, cm.
const BRACKET_LINE: i64 = 1;
const BRACKET_ALONG: f64 = -2.4;
const BRACKET_TICK: f64 = 0.15;
const BRACKET_LABEL: f64 = 0.55;

// ------------------------------------------------------------------ timing

const OPEN_S: f64 = 1.0;
const FADE_S: f64 = 0.4;
const WIDEN_S: f64 = 0.35;
/// 04_1's slide: 1.5 cm (`slide.distance`) at 0.35 cm a second, a fifth
/// of that from 0.05 cm before the brief count to 0.05 cm after it.
const SLIDE_BY: f64 = 1.5;
const FAST: f64 = 0.35;
const SLOW: f64 = FAST / 5.0;
const SLOW_MARGIN: f64 = 0.05;
/// The oracle's steps for the slide's runs (`SLIDE_SHOWN_STEPS`).
const RUN_STEPS: usize = 15000;
const TURN_S: f64 = 5.0;
const SETTLE_S: f64 = 2.5;
const NUDGE_BY: f64 = 0.3;
const NUDGE_TURN: f64 = -5.0;
const NUDGE_S: f64 = 1.0;
/// One gap at a steady 0.25 cm a second: its briefest count, 0.006 cm,
/// then lasts about 1.4 frames at 60 fps.
const GAP_S: f64 = 8.0;
const HALF_S: f64 = 7.0;
const BACK_TURN_S: f64 = 4.0;
const BACK_SLIDE_S: f64 = 2.0;
const SURVEY_SLIDE_S: f64 = 4.0;
const SURVEY_TURN_S: f64 = 5.0;

/// Every move the chapter makes, from the model: each starts where the
/// last one ended, named the controls' way where the scene settles.
struct Plan {
    first: Pose,
    /// 04_1's slide, its runs, and the shares of it to crawl through.
    slide: Move,
    slide_runs: Vec<Run>,
    slow: [f64; 2],
    /// 04_3: an eighth of a turn about the card's middle, then to the
    /// slide shown for that direction.
    eighth: Move,
    settle: Move,
    /// 04_5: the slide up and back, then a small turn and back.
    nudges: [Move; 4],
    /// 04_7: one gap; 04_8: half a turn about the point midway between
    /// two lines.
    gap: Move,
    half: Move,
    /// 04_10: the direction back about the middle, then the slide back.
    back: [Move; 2],
    /// 04_11: every slide, then every direction.
    survey: [Move; 2],
}

fn plan(coast: &Curve) -> Plan {
    let first = Pose::new(0.0, sheet::shown_slide(coast, 0.0));
    let slide = Move::slide(first, SLIDE_BY);
    let slide_runs = placement::runs(coast, &slide, RUN_STEPS);
    let brief = slide_runs
        .iter()
        .min_by(|a, b| (a.to - a.from).total_cmp(&(b.to - b.from)))
        .copied()
        .unwrap_or(Run {
            count: 0,
            from: 0.0,
            to: 0.0,
        });
    let slow = [
        (brief.from - SLOW_MARGIN / SLIDE_BY).max(0.0),
        (brief.to + SLOW_MARGIN / SLIDE_BY).min(1.0),
    ];
    let eighth = Move::turn(slide.end(), PI / 4.0, [0.0, 0.0]);
    let turned = sheet::shown_slide(coast, PI / 4.0);
    let settle = Move::slide(eighth.end(), turned - eighth.end().slide);
    let up = Move::slide(settle.end(), NUDGE_BY);
    let down = Move::slide(up.end(), -NUDGE_BY);
    let turn = Move::turn(down.end(), NUDGE_TURN.to_radians(), [0.0, 0.0]);
    let unturn = Move::turn(turn.end(), -NUDGE_TURN.to_radians(), [0.0, 0.0]);
    let gap = Move::slide(unturn.end(), GAP);
    let repeated = gap.end().canonical();
    let half = Move::turn(repeated, PI, placement::half_turn_pin(repeated, 0.0));
    let returned = half.end().canonical();
    let back_turn = Move::turn(returned, -returned.beta, [0.0, 0.0]);
    let back_slide = Move::slide(back_turn.end(), first.slide - back_turn.end().slide);
    let every_slide = Move::slide(back_slide.end(), GAP);
    let wrapped = every_slide.end().canonical();
    let every_direction = Move::turn(wrapped, PI, placement::half_turn_pin(wrapped, 0.0));
    Plan {
        first,
        slide,
        slide_runs,
        slow,
        eighth,
        settle,
        nudges: [up, down, turn, unturn],
        gap,
        half,
        back: [back_turn, back_slide],
        survey: [every_slide, every_direction],
    }
}

impl Plan {
    /// 04_1's legs: steady, slower through the brief count, steady.
    fn slide_legs(&self) -> [(f64, f64); 3] {
        let [a, z] = self.slow;
        [
            (a, a * SLIDE_BY / FAST),
            (z, (z - a) * SLIDE_BY / SLOW),
            (1.0, (1.0 - z) * SLIDE_BY / FAST),
        ]
    }

    /// Every move, in the order made.
    #[cfg(test)]
    fn moves(&self) -> Vec<Move> {
        let mut all = vec![self.slide, self.eighth, self.settle];
        all.extend(self.nudges);
        all.extend([self.gap, self.half]);
        all.extend(self.back);
        all.extend(self.survey);
        all
    }
}

/// The chapter's words that carry numbers, each from the model or the
/// facts. Refused unless the gap is the facts', the facts (which the
/// oracle checks) say what the voice says, and the model gives every
/// count the voice says: 7, 9, 8, 7 on the slide, 8 after the eighth of a
/// turn and after both repeats, 7 back at the first placement and at the
/// end.
struct Words {
    first: String,
    log: Vec<String>,
    gap: String,
}

fn words(coast: &Curve, plan: &Plan) -> Result<Words, String> {
    let f = facts::facts()?;
    let get = |key: &str| f.get(key).map(str::to_owned).ok_or_else(|| format!("no fact {key}"));
    let gap = get("sheet.gap")?;
    if gap != facts::fixed(GAP, 0) {
        return Err(format!("the sheet is drawn {GAP} cm apart, the facts say {gap}"));
    }
    let said = [
        ("slide.distance", facts::fixed(SLIDE_BY, 1)),
        ("slide.sequence", SAID_SLIDE.map(|n| n.to_string()).join(",")),
        ("turn.45.crossings", SAID_TURNED.to_string()),
        ("open.crossings", SAID_FIRST.to_string()),
    ];
    for (key, want) in said {
        let got = get(key)?;
        if got != want {
            return Err(format!("the chapter says {want} for {key}, the facts {got}"));
        }
    }
    let count = |p: Pose| sheet::crossings(coast, p.beta, p.slide).len();
    let seen: Vec<usize> = plan.slide_runs.iter().map(|r| r.count).collect();
    if seen != SAID_SLIDE {
        return Err(format!(
            "04_2 says the slide reads {SAID_SLIDE:?}, the model reads {seen:?}"
        ));
    }
    let turned = [plan.settle.end(), plan.gap.end(), plan.half.end()].map(count);
    if turned != [SAID_TURNED; 3] {
        return Err(format!(
            "04_4, 04_7 and 04_9 say {SAID_TURNED}, the model reads {turned:?}"
        ));
    }
    let home = [plan.first, plan.back[1].end(), plan.survey[1].end()].map(count);
    if home != [SAID_FIRST; 3] {
        return Err(format!(
            "04_10 says the first count is {SAID_FIRST}, the model reads {home:?}"
        ));
    }
    Ok(Words {
        first: count_words(count(plan.first)),
        log: seen.iter().map(usize::to_string).collect(),
        gap: format!("{gap} cm"),
    })
}

/// 04_6's bracket at `pose`: from line [`BRACKET_LINE`] to where the next
/// line lies, one gap across, with a tick at each end, and where its label
/// goes (card cm).
fn bracket(pose: Pose) -> (Vec<[P2; 2]>, P2) {
    let n = [-pose.beta.sin(), pose.beta.cos()];
    let d = [pose.beta.cos(), pose.beta.sin()];
    let h = pose.slide + BRACKET_LINE as f64 * GAP;
    let p = [h * n[0] + BRACKET_ALONG * d[0], h * n[1] + BRACKET_ALONG * d[1]];
    let q = [p[0] + GAP * n[0], p[1] + GAP * n[1]];
    let tick = |c: P2| {
        [
            [c[0] - BRACKET_TICK * d[0], c[1] - BRACKET_TICK * d[1]],
            [c[0] + BRACKET_TICK * d[0], c[1] + BRACKET_TICK * d[1]],
        ]
    };
    let mid = [(p[0] + q[0]) / 2.0, (p[1] + q[1]) / 2.0];
    (
        vec![[p, q], tick(p), tick(q)],
        [mid[0] + BRACKET_LABEL * d[0], mid[1] + BRACKET_LABEL * d[1]],
    )
}

/// A gold ring of radius `r` round the frame point `at`. Detached.
fn ring(stage: &mut Stage<'_>, at: Vec3, r: f64) -> Mob {
    let ring = Circle::new()
        .radius(r)
        .arc_center(at)
        .style(line_style(palette::touch(), 2.5))
        .build();
    stage.arena_mut().add(ring)
}

/// Where log entry `i` stands, frame units.
fn log_at(i: usize) -> Vec3 {
    v(LOG_X + i as f64 * LOG_STEP, TOP_Y)
}

/// Keep the moving ruled sheet behind legible labels. These are detached
/// overlay panels, not cuts in the mathematical sheet or its intersections.
fn backing(stage: &mut Stage<'_>, at: Vec3, width: f64, height: f64) -> fmn::Result<Mob> {
    let panel = Rectangle::new()
        .width(width)
        .height(height)
        .corner_radius(0.08)
        .style(flat(Palette::faint(), 1.0))
        .build()?
        .moved_to(at);
    Ok(stage.arena_mut().add(panel))
}

impl SceneConstruct for OneCountOnePlacement {
    fn name(&self) -> &str {
        "04_one_count_one_placement"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let coast = model::coast();
        let plan = plan(&coast);
        let words = words(&coast, &plan).map_err(fail)?;

        // where 03 ended: its title over the dimmed card, coast, sheet,
        // dots, count and arrow; the envelope bright
        let card = card_panel(stage)?;
        stage.set_family_opacity(card, DIMMED);
        stage.add_to_scene(card)?;
        let pieces: Vec<Mob> = coast
            .parts
            .iter()
            .map(|p| {
                path(
                    stage,
                    &coast.between(p.start, p.end(), EVERY),
                    palette::coast(),
                    COAST_W,
                )
            })
            .collect();
        for &m in &pieces {
            stage.set_color_lanes(m, "stroke_rgba", None, Some(DIMMED), true);
            stage.add_to_scene(m)?;
        }
        let panel = Sheet::new(stage, plan.first.slide, 0.0)?.panel;
        stage.add_to_scene(panel)?;
        let place = Placement::new(stage, plan.first);
        // 04_6's halo, made first so the lines are drawn over it
        let halo_alpha = stage.add_value_tracker(0.0);
        placement::live_halo(stage, place, BRACKET_LINE, halo_alpha)?;
        let lines_alpha = stage.add_value_tracker(DIMMED);
        placement::live_lines(stage, place, lines_alpha)?;
        // Keep the later footprints below the labels even when revealed.
        let steps = placement::footprints(stage, plan.gap.from)?;
        for &m in &steps {
            stage.set_family_opacity(m, 0.0);
            stage.add_to_scene(m)?;
        }
        let dots_alpha = stage.add_value_tracker(DIMMED);
        placement::live_dots(stage, place, &coast, dots_alpha)?;
        for (at, width, height) in [(v(-5.6, 2.92), 1.5, 1.2), (v(-1.05, TOP_Y), 4.9, 0.64)] {
            let panel = backing(stage, at, width, height)?;
            stage.add_to_scene(panel)?;
        }
        let old_count = kit.text(stage, &words.first, COUNT_SIZE, palette::touch(), COUNT_AT)?;
        let arrow = kit.tex(
            stage,
            ARROW,
            COUNT_SIZE,
            Palette::ink(),
            &[(ASK, palette::touch())],
            COUNT_AT,
        )?;
        stage.next_to(arrow, old_count, RIGHT, 0.25, ORIGIN);
        for m in [old_count, arrow] {
            stage.set_family_opacity(m, DIMMED);
            stage.add_to_scene(m)?;
        }
        for m in placement::envelope(&kit, stage)? {
            stage.add_to_scene(m)?;
        }
        let title = kit.text(stage, TITLE, TITLE_SIZE, Palette::ink(), TITLE_AT)?;
        stage.add_to_scene(title)?;

        // 04_1: the title goes, everything 03 dimmed comes back, its count
        // and arrow go; the live count arrives; will sliding keep it?
        let line = say(&kit, stage, "04_1")?;
        let mut brighten = vec![
            fade_to(card, CARD_OPACITY, OPEN_S)?,
            tween(lines_alpha, 1.0, OPEN_S)?,
            tween(dots_alpha, 1.0, OPEN_S)?,
        ];
        for &m in &pieces {
            brighten.push(dim_line(m, 1.0, OPEN_S)?);
        }
        at_once(stage, &[], &[title, old_count, arrow], brighten, OPEN_S)?;
        line.wait_for(stage, "the count itself")?;
        let count_alpha = stage.add_value_tracker(0.0);
        let counted = coast.clone();
        let count_now = move |s: &fmn::mobject::Stage| {
            let p = place.pose(s);
            sheet::crossings(&counted, p.beta, p.slide).len()
        };
        placement::live_count(&kit, stage, count_now, count_alpha, COUNT_SIZE, COUNT_RIGHT)?;
        let word_at = v(COUNT_RIGHT[0] + WORD_GAP, COUNT_RIGHT[1]);
        placement::live_word(&kit, stage, CROSSINGS, count_alpha, COUNT_SIZE, word_at)?;
        stage.play(tween(count_alpha, 1.0, FADE_S)?)?;
        let think_back = backing(stage, v(THINK_AT[0], 2.65), 2.8, 2.0)?;
        fade_in_all(stage, &[think_back], 0.2)?;
        let q = think(&kit, stage, THINK_AT)?;

        // the slide, in silence: the count follows the sheet, and each new
        // count is logged the moment it is read
        fade_out_all(stage, &[q, think_back], 0.3)?;
        let mut log = Vec::with_capacity(words.log.len());
        for (i, n) in words.log.iter().enumerate() {
            let m = kit.text(stage, n, LOG_SIZE, palette::touch(), log_at(i))?;
            stage.set_family_opacity(m, 0.0);
            stage.add_to_scene(m)?;
            log.push(m);
        }
        let starts: Vec<f64> = plan.slide_runs.iter().map(|r| r.from).collect();
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        let logged = log.clone();
        stage.add_updater(
            anchor,
            move |s, _| {
                let u = place.share(s);
                for (&m, &from) in logged.iter().zip(&starts) {
                    s.set_family_opacity(m, if u >= from { 1.0 } else { 0.0 });
                }
            },
            true,
        )?;
        place.make(stage, &plan.slide, &plan.slide_legs(), linear)?;
        stop(stage, anchor);

        // 04_2: the counts, named as the log shows them; the coast itself
        // never changed
        let line = say(&kit, stage, "04_2")?;
        line.wait_for(stage, "dips to eight")?;
        let eight = ring(stage, log_at(2), RING_R);
        fade_in_all(stage, &[eight], 0.3)?;
        line.wait_for(stage, "The coast hasn't changed")?;
        fade_out_all(stage, &[eight], 0.3)?;
        together(stage, widen(&pieces, PICKED_W, WIDEN_S)?)?;
        together(stage, widen(&pieces, COAST_W, WIDEN_S)?)?;

        // 04_3: the slide's log goes (it read that slide only); an eighth
        // of a turn about the card's middle, then the slide that keeps the
        // lines clearest of the coast
        let line = say(&kit, stage, "04_3")?;
        fade_out_all(stage, &log, FADE_S)?;
        line.wait_for(stage, "Give it an eighth")?;
        let middle = placement::pivot(stage, [0.0, 0.0]);
        fade_in_all(stage, &[middle], 0.3)?;
        place.make(stage, &plan.eighth, &[(1.0, TURN_S)], smooth)?;
        place.make(stage, &plan.settle, &[(1.0, SETTLE_S)], smooth)?;
        fade_out_all(stage, &[middle], 0.3)?;

        // 04_4: eight
        say(&kit, stage, "04_4")?;

        // 04_5: a placement is two choices; the controls, bound to the
        // sheet, and each choice nudged and put back
        let line = say(&kit, stage, "04_5")?;
        line.wait_for(stage, "two choices")?;
        let control_backs = [
            backing(stage, v(-4.25, -2.68), 4.5, 1.0)?,
            backing(stage, v(2.65, -2.42), 3.9, 1.4)?,
        ];
        fade_in_all(stage, &control_backs, 0.2)?;
        let controls = Controls::new(&kit, stage, place)?;
        at_once(
            stage,
            &controls.fixed,
            &[],
            vec![
                tween(controls.knob_alpha, 1.0, FADE_S)?,
                tween(controls.needle_alpha, 1.0, FADE_S)?,
            ],
            FADE_S,
        )?;
        line.wait_for(stage, "How far the sheet is slid")?;
        let track = placement::halo_along(stage, placement::slider_track());
        fade_in_all(stage, &[track], 0.3)?;
        place.make(stage, &plan.nudges[0], &[(1.0, NUDGE_S)], smooth)?;
        place.make(stage, &plan.nudges[1], &[(1.0, NUDGE_S)], smooth)?;
        line.wait_for(stage, "which way its lines run")?;
        let arc = placement::halo_along(stage, placement::dial_arc());
        at_once(stage, &[arc], &[track], vec![], 0.3)?;
        place.make(stage, &plan.nudges[2], &[(1.0, NUDGE_S)], smooth)?;
        place.make(stage, &plan.nudges[3], &[(1.0, NUDGE_S)], smooth)?;
        fade_out_all(stage, &[arc], 0.3)?;

        // 04_6: one gap's slide, marked on one line; the count hidden while
        // the viewer predicts
        let line = say(&kit, stage, "04_6")?;
        line.wait_for(stage, "slide the sheet by exactly one gap")?;
        let (ticks, label_at) = bracket(plan.gap.from);
        let mut marks: Vec<Mob> = ticks.iter().map(|seg| path(stage, seg, Palette::ink(), 2.0)).collect();
        marks.push(kit.text(stage, &words.gap, LABEL_SIZE, Palette::dim(), card_point(label_at))?);
        at_once(stage, &marks, &[], vec![tween(halo_alpha, 1.0, FADE_S)?], FADE_S)?;
        line.wait_for(stage, "what will the count be")?;
        stage.play(tween(count_alpha, 0.0, 0.3)?)?;
        let think_back = backing(stage, v(THINK_AT[0], 2.65), 2.8, 2.0)?;
        fade_in_all(stage, &[think_back], 0.2)?;
        let q = think(&kit, stage, THINK_AT)?;

        // the slide by one gap, in silence, the count shown throughout:
        // the lines leave their footprints and land on them
        let mut reveal = vec![tween(count_alpha, 1.0, 0.3)?];
        for &m in &steps {
            reveal.push(fade_to(m, 1.0, 0.3)?);
        }
        at_once(stage, &[], &[q, think_back], reveal, 0.3)?;
        place.make(stage, &plan.gap, &[(1.0, GAP_S)], linear)?;

        // 04_7: eight again; only slides within one gap are different
        let line = say(&kit, stage, "04_7")?;
        line.wait_for(stage, "Only slides within one gap")?;
        let track = placement::halo_along(stage, placement::slider_track());
        at_once(stage, &[track], &marks, vec![tween(halo_alpha, 0.0, FADE_S)?], FADE_S)?;
        place.settle(stage)?;

        // 04_8: half a turn about the point midway between two lines, the
        // knob hidden (half a turn renames the slide), the footprints kept
        let line = say(&kit, stage, "04_8")?;
        line.wait_for(stage, "about this point")?;
        let point = placement::pivot(stage, plan.half.pin);
        at_once(
            stage,
            &[point],
            &[track],
            vec![tween(controls.knob_alpha, 0.0, 0.3)?],
            0.3,
        )?;
        place.make(stage, &plan.half, &[(1.0, HALF_S)], smooth)?;
        place.settle(stage)?;
        stage.play(tween(controls.knob_alpha, 1.0, 0.3)?)?;

        // 04_9: the same places; only directions within half a turn
        let line = say(&kit, stage, "04_9")?;
        line.wait_for(stage, "Only directions within half a turn")?;
        let arc = placement::halo_along(stage, placement::dial_arc());
        let mut gone = steps.clone();
        gone.push(point);
        at_once(stage, &[arc], &gone, vec![], FADE_S)?;

        // back to the first placement in silence, a choice at a time, the
        // count shown throughout
        kit.hold(stage)?;
        fade_out_all(stage, &[arc], 0.3)?;
        place.make(stage, &plan.back[0], &[(1.0, BACK_TURN_S)], smooth)?;
        place.make(stage, &plan.back[1], &[(1.0, BACK_SLIDE_S)], smooth)?;

        // 04_10: its seven crossings, ringed, one reading among many
        let line = say(&kit, stage, "04_10")?;
        line.wait_for(stage, "Our first count")?;
        let first: Vec<Mob> = sheet::crossings(&coast, plan.first.beta, plan.first.slide)
            .iter()
            .map(|c| ring(stage, card_point(c.at), CROSSING_RING_R))
            .collect();
        fade_in_all(stage, &first, 0.4)?;
        line.wait_for(stage, "one placement among many")?;
        fade_out_all(stage, &first, 0.4)?;

        // 04_11: the plan; every slide, then every direction (about the
        // point on the middle line, the knob hidden), then the question
        let line = say(&kit, stage, "04_11")?;
        line.wait_for(stage, "Try every slide")?;
        place.make(stage, &plan.survey[0], &[(1.0, SURVEY_SLIDE_S)], smooth)?;
        place.settle(stage)?;
        let point = placement::pivot(stage, plan.survey[1].pin);
        at_once(stage, &[point], &[], vec![tween(controls.knob_alpha, 0.0, 0.3)?], 0.3)?;
        place.make(stage, &plan.survey[1], &[(1.0, SURVEY_TURN_S)], smooth)?;
        place.settle(stage)?;
        let mut out = controls.fixed.clone();
        out.extend(control_backs);
        out.push(point);
        at_once(
            stage,
            &[],
            &out,
            vec![
                tween(count_alpha, 0.0, FADE_S)?,
                tween(controls.needle_alpha, 0.0, FADE_S)?,
            ],
            FADE_S,
        )?;
        let question = kit.text_t2c(
            stage,
            QUESTION,
            COUNT_SIZE,
            Palette::ink(),
            &[(ASK, palette::touch())],
            QUESTION_AT,
        )?;
        fade_in_all(stage, &[question], 0.5)?;
        let up = now(stage);
        stand(&kit, stage, up)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{CARD_CENTRE, CARD_UNIT};

    fn shown() -> (Curve, Plan) {
        let coast = model::coast();
        let plan = plan(&coast);
        (coast, plan)
    }

    fn count(coast: &Curve, p: Pose) -> usize {
        sheet::crossings(coast, p.beta, p.slide).len()
    }

    fn dist(a: P2, b: P2) -> f64 {
        (a[0] - b[0]).hypot(a[1] - b[1])
    }

    fn seg_dist(p: P2, a: P2, b: P2) -> f64 {
        let (ax, ay) = (b[0] - a[0], b[1] - a[1]);
        let t = (((p[0] - a[0]) * ax + (p[1] - a[1]) * ay) / (ax * ax + ay * ay)).clamp(0.0, 1.0);
        dist(p, [a[0] + t * ax, a[1] + t * ay])
    }

    /// The coast at every 0.02 cm of its length, straight runs included
    /// (`Curve::upto` keeps only a straight run's ends), for distances.
    fn along(coast: &Curve) -> Vec<P2> {
        let n = (coast.length / 0.02).ceil() as usize;
        (0..=n).map(|i| coast.at(coast.length * i as f64 / n as f64)).collect()
    }

    /// The words on screen say what the model and the facts say: 03's
    /// "7 crossings" (`open.crossings`), the log 7, 9, 8, 7
    /// (`slide.sequence`), "2 cm" (`sheet.gap`).
    #[test]
    fn the_screen_words_are_built_from_the_model() {
        let (coast, plan) = shown();
        let w = words(&coast, &plan).unwrap();
        assert_eq!(w.first, "7 crossings");
        assert_eq!(w.log, ["7", "9", "8", "7"]);
        assert_eq!(w.gap, "2 cm");
        assert_eq!(QUESTION, "average count = ?");
    }

    /// The voice's counts are refused when the model disagrees: sliding
    /// down instead of up reads 7, 9, 7, 8, 9; stopping after 1 cm leaves
    /// the count at 8; half a turn about the card's middle ends at 11.
    #[test]
    fn a_plan_the_voice_does_not_describe_is_refused() {
        for (by, seen) in [(-SLIDE_BY, vec![7, 9, 7, 8, 9]), (1.0, vec![7, 9, 8])] {
            let (coast, mut plan) = shown();
            plan.slide = Move::slide(plan.first, by);
            plan.slide_runs = placement::runs(&coast, &plan.slide, 3000);
            assert_eq!(plan.slide_runs.iter().map(|r| r.count).collect::<Vec<_>>(), seen);
            assert!(words(&coast, &plan).is_err());
        }
        let (coast, mut plan) = shown();
        plan.half = Move::turn(plan.half.from, PI, [0.0, 0.0]);
        assert_eq!(count(&coast, plan.half.end()), 11);
        assert!(words(&coast, &plan).is_err());
    }

    /// The true length stays sealed: no word on screen gives it away.
    #[test]
    fn the_true_length_is_not_on_screen() {
        let f = facts::facts().unwrap();
        let (coast, plan) = shown();
        let w = words(&coast, &plan).unwrap();
        let mut on_screen = vec![
            w.first,
            w.gap,
            ARROW.into(),
            QUESTION.into(),
            CROSSINGS.into(),
            TITLE.into(),
            placement::SEALED.into(),
            placement::SLIDE_NAME.into(),
            placement::DIRECTION_NAME.into(),
            super::super::draw::PAUSE_NOTE.into(),
        ];
        on_screen.extend(w.log);
        for key in ["coast.length", "coast.length_2dp", "coast.length_3dp"] {
            let length = f.get(key).unwrap();
            for text in &on_screen {
                assert!(!text.contains(length), "{text} shows {length}");
            }
        }
    }

    /// Each move starts where the last ended, and the scene's settles
    /// name the same lines.
    #[test]
    fn the_moves_chain() {
        let (_, plan) = shown();
        let moves = plan.moves();
        for pair in moves.windows(2) {
            let (end, next) = (pair[0].end(), pair[1].from);
            assert!(end.same_lines(next, 1e-9), "{end:?} then {next:?}");
        }
        assert_eq!(moves.len(), 13);
        // the settles: after the gap, the half turn and both survey moves
        assert_eq!(plan.half.from, plan.gap.end().canonical());
        assert_eq!(plan.back[0].from, plan.half.end().canonical());
        assert!(plan.back[1].end().same_lines(plan.first, 1e-12));
        assert!(plan.survey[1].end().same_lines(plan.first, 1e-12));
    }

    /// When the slide, run through `legs` at a steady pace each, reaches
    /// share `u`, seconds.
    fn seconds_at(legs: &[(f64, f64)], u: f64) -> f64 {
        let (mut from, mut t) = (0.0, 0.0);
        for &(to, seconds) in legs {
            if u <= to {
                return t + seconds * (u - from) / (to - from);
            }
            (from, t) = (to, t + seconds);
        }
        t
    }

    /// 04_1's slide crawls through the brief 8: it lies inside the slow
    /// stretch and stays on screen 0.8 s, 48 frames at 60 fps; every count
    /// the slide reads lasts at least that long.
    #[test]
    fn the_brief_eight_has_time_to_be_read() {
        let (_, plan) = shown();
        let [a, z] = plan.slow;
        let eight = plan.slide_runs[2];
        assert_eq!(eight.count, 8);
        assert!(a < eight.from && eight.to < z);
        let seconds = (eight.to - eight.from) * SLIDE_BY / SLOW;
        assert!(seconds > 0.78, "{seconds}");
        let legs = plan.slide_legs();
        let total: f64 = legs.iter().map(|l| l.1).sum();
        assert!((total - 6.07).abs() < 0.01, "{total}");
        assert!(legs.iter().all(|l| l.1 > 1.0));
        for r in &plan.slide_runs {
            let shown = seconds_at(&legs, r.to.min(1.0)) - seconds_at(&legs, r.from.max(0.0));
            assert!(shown > 0.78, "{} lasts {shown} s", r.count);
        }
        // a steady slide at the fast pace would flash it for a fifth of that
        assert!((eight.to - eight.from) * SLIDE_BY / FAST < 0.2);
    }

    /// Every count any move passes through is more than one, so
    /// "crossings" is always the word; and the plan's moves keep the
    /// counts the voice gives at their ends.
    #[test]
    fn every_count_on_the_way_is_plural() {
        let (coast, plan) = shown();
        for mv in plan.moves() {
            let r = placement::runs(&coast, &mv, 4000);
            assert!(
                r.iter().all(|x| x.count >= 2),
                "{mv:?}: {:?}",
                r.iter().map(|x| x.count).collect::<Vec<_>>()
            );
        }
        assert_eq!(count(&coast, plan.settle.end()), 8);
        assert_eq!(count(&coast, plan.nudges[3].end()), 8);
        // the nudges move the knob and the needle and leave the count at 8
        for mv in plan.nudges {
            let r = placement::runs(&coast, &mv, 4000);
            assert_eq!(r.iter().map(|x| x.count).collect::<Vec<_>>(), [8]);
        }
    }

    /// 04_6's bracket runs from line 1 to where line 2 lies, one gap at
    /// right angles to the lines, over empty card: more than 2 cm from
    /// the coast, its label too, inside the card, and above the controls.
    #[test]
    fn the_bracket_is_one_gap_over_empty_card() {
        let (coast, plan) = shown();
        let at = plan.gap.from;
        assert_eq!(facts::fixed(at.beta.to_degrees(), 6), "45.000000");
        let (segs, label) = bracket(at);
        let [p, q] = segs[0];
        assert!((dist(p, q) - GAP).abs() < 1e-12);
        let n = [-at.beta.sin(), at.beta.cos()];
        let height = |x: P2| x[0] * n[0] + x[1] * n[1];
        assert!((height(p) - (at.slide + GAP)).abs() < 1e-9 && (height(q) - (at.slide + 2.0 * GAP)).abs() < 1e-9);
        let pts = along(&coast);
        for x in &pts {
            assert!(seg_dist(*x, p, q) > 2.0 && dist(*x, label) > 2.0);
        }
        for x in [p, q, label] {
            let f = card_point(x);
            assert!(
                x[0].abs() < 7.0 && x[1].abs() < 4.0 && f[1] > placement::DIAL_AT[1] + 1.0,
                "{x:?}"
            );
        }
        // the haloed line is the one the bracket starts from, and after the
        // gap it lies where the bracket ends
        let after = plan.gap.end();
        assert!((after.slide + BRACKET_LINE as f64 * GAP - height(q)).abs() < 1e-9);
    }

    /// The log and its rings stay in the band over the coast, between the
    /// count (whose word, 30 points, is taken as at most 1.8 wide) and the
    /// think ring; the points turned about sit off the coast; 04_10's
    /// rings round the crossings do not touch each other.
    #[test]
    fn the_layout_keeps_its_rooms() {
        use super::super::draw::THINK_R;
        const {
            assert!(LOG_X - RING_R > COUNT_RIGHT[0] + WORD_GAP + 1.8);
            assert!(LOG_X + 3.0 * LOG_STEP + RING_R + 0.3 < THINK_AT[0] - THINK_R);
        }
        let (coast, plan) = shown();
        let band_lo = CARD_CENTRE[1] + CARD_UNIT * sheet::lines_on_card(plan.first.slide)[3];
        assert!(TOP_Y - RING_R > band_lo + 0.05);
        let pts = along(&coast);
        for pin in [[0.0, 0.0], plan.half.pin, plan.survey[1].pin] {
            let near = pts.iter().map(|&x| dist(x, pin)).fold(f64::INFINITY, f64::min);
            assert!(near > 0.35, "{pin:?} is {near} cm from the coast");
        }
        let at: Vec<P2> = sheet::crossings(&coast, plan.first.beta, plan.first.slide)
            .iter()
            .map(|c| c.at)
            .collect();
        assert_eq!(at.len(), 7);
        for (i, a) in at.iter().enumerate() {
            for b in &at[i + 1..] {
                assert!(CARD_UNIT * dist(*a, *b) > 2.0 * CROSSING_RING_R + 0.05, "{a:?} {b:?}");
            }
        }
    }

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("04_1", "the count itself"),
            ("04_2", "dips to eight"),
            ("04_2", "The coast hasn't changed"),
            ("04_3", "Give it an eighth"),
            ("04_5", "two choices"),
            ("04_5", "How far the sheet is slid"),
            ("04_5", "which way its lines run"),
            ("04_6", "slide the sheet by exactly one gap"),
            ("04_6", "what will the count be"),
            ("04_7", "Only slides within one gap"),
            ("04_8", "about this point"),
            ("04_9", "Only directions within half a turn"),
            ("04_10", "Our first count"),
            ("04_10", "one placement among many"),
            ("04_11", "Try every slide"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
        assert!(script.text("04_4").is_some());
    }
}
