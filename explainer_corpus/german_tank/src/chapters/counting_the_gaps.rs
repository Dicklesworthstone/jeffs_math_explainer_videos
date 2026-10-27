//! Chapter 03, "Counting what nobody saw": back on the friends' line from
//! 1 to 84 with the badge in the corner, the unseen numbers below each box
//! are counted dot by dot, one gap at a time, each with its own live count
//! (03_1 to 03_3); the five counts slide into the panel's sum and its
//! average (03_4); the top gap, dashed off the faded end with its "?"
//! (03_5); a fair handful from seventy, whose count is known, so its top
//! gap can be seen and is just one more gap (03_6); the prediction (03_7);
//! the top gap guessed as one average gap, the bracket shortening to the
//! estimate (03_8); the recipe and its condition (03_9); and what "right on
//! average" promises, and what it doesn't (03_10).
//!
//! The counts are trackers: a gap's number under the line is the kit's live
//! count of its tracker, and its blue dots light by the same rounding, so
//! the number shown and the dots lit always agree. The top gap's bracket is
//! bound to one tracker, its right end, so it shortens rather than being
//! redrawn. Every number on the screen is a fact.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, Timed, fade_all, tween, v};
use fmn::prelude::*;

use super::draw::{
    BRACKET_DY, BRACKET_W, DOT_R, LABEL_DY, OFF, STRIP_X1, STRIP_Y, TICK_W, bracket_points, dash_spans, fade_in_all,
    fade_in_lagged, fade_out_all, fail, flash_all, flash_lagged, flat, line_style, move_all, pause, say, stroke, think,
    wait_until, x_at,
};
use super::friends_line::{
    ARC_H, ARC_LIFT, ARC_W, BADGE_AT, FriendsLine, PANEL_SIZE, PANEL_X, PANEL_Y, Start, THINK_AT, arc_points, at,
    badge, bind_to, gap_spans, hidden, is_handful, list, now, panel, read_s, reshape, ring, thin, whole,
};
use crate::facts;
use crate::model::LINE_DOTS;
use crate::palette;

pub struct CountingTheGaps {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const UNKNOWN: &str = "?";
const RECIPE: &str = "estimate = largest + average gap";
const CLAIM: &str = "right on average";
const INSET_CAPTION: &str = "a fair handful from";

/// A lit dot: blue, a little larger than the faint dot it covers.
const LIT_R: f64 = DOT_R * 1.5;
/// The counting: seconds per dot, the same for the first two gaps (03_1,
/// 03_2), then a little faster for each of the last three (03_3).
const COUNT_DOT_S: f64 = 0.12;
const FASTER: f64 = 0.85;
/// An arc closing over its counted dots; the counts' slide into the panel;
/// the gap between the pieces of a panel sum; the top gap's bracket
/// shortening to the estimate.
const CLOSE_S: f64 = 0.4;
const SLIDE_S: f64 = 1.0;
const SUM_GAP: f64 = 0.14;
const SHRINK_S: f64 = 1.6;
/// The tail's "?", under its dashed bracket near the strip's right end (as
/// chapter 01 places it), and the guessed top gap's count, under its green
/// bracket.
const TAIL_X: f64 = 5.4;
const UNDER_BRACKET: f64 = 0.35;
/// The estimate's tick across the line, from just under it to above it,
/// and its label over the tick.
const TICK_DOWN: f64 = 0.12;
const TICK_UP: f64 = 0.3;
const ESTIMATE_DY: f64 = 0.55;
/// The inset (03_6): its own small line from `INSET_X[0]` to `INSET_X[1]`
/// at `INSET_Y`, the counts over its arcs, its caption above, its top
/// gap's dashed bracket under it.
const INSET_X: [f64; 2] = [-6.3, -0.3];
const INSET_Y: f64 = 1.45;
const INSET_ARC_H: f64 = 0.16;
const INSET_SIZE: f64 = 20.0;
const INSET_COUNT_DY: f64 = 0.17;
const INSET_CAPTION_DY: f64 = 0.7;
const INSET_BRACKET_DY: f64 = 0.28;
const INSET_END_DX: f64 = 0.3;
const INSET_LINE_W: f64 = 2.0;
/// The badge beside the claim it conditions (03_9); how far the estimate
/// dims when the words say it is not what is promised (03_10).
const BADGE_GAP: f64 = 0.3;
const DIM_O: f64 = 0.4;
const END_BEAT: f64 = 0.6;

/// What chapter 03 shows, from the facts: the friends' five, their gaps,
/// the unseen total, the gap count and the average gap, the estimate; the
/// handful from the small count with its gaps and its top gap.
#[derive(Clone, Debug)]
struct Picture {
    numbers: Vec<u64>,
    first: u64,
    largest: u64,
    k: usize,
    gaps: Vec<u64>,
    unseen: u64,
    gap_count: usize,
    average_gap: u64,
    estimate: u64,
    small: u64,
    known: Vec<u64>,
    known_gaps: Vec<u64>,
    known_top_gap: u64,
}

fn picture() -> Result<Picture, String> {
    let f = facts::facts()?;
    checked(Picture {
        numbers: list(&f, "G-F1.numbers")?,
        first: whole(&f, "G-F1.first")?,
        largest: whole(&f, "G-F1.largest")?,
        k: whole(&f, "G-F1.handful")? as usize,
        gaps: list(&f, "G-F3.gaps")?,
        unseen: whole(&f, "G-F3.unseen")?,
        gap_count: whole(&f, "G-F3.gap_count")? as usize,
        average_gap: whole(&f, "G-F3.average_gap")?,
        estimate: whole(&f, "G-F3.estimate")?,
        small: whole(&f, "G-F2.small")?,
        known: list(&f, "G-F3.known_handful")?,
        known_gaps: list(&f, "G-F3.known_gaps")?,
        known_top_gap: whole(&f, "G-F3.known_top_gap")?,
    })
}

/// The sizes of a sorted handful's non-empty gaps, below the smallest and
/// between neighbours, smallest first.
fn span_sizes(hand: &[u64]) -> Vec<u64> {
    gap_spans(hand).iter().map(|&(a, b)| b - a + 1).collect()
}

/// The picture only if the scene can draw it truthfully: the friends' five
/// are a handful on the line, the largest last; each gap the facts list is
/// one stretch of unseen dots the scene counts, so none is empty and there
/// are as many as numbers; the panel's sum and its division are exact; the
/// estimate is the largest plus one average gap, and its tick is on the
/// line; the inset's handful is a handful from the small count whose gaps
/// and top gap fill that count exactly.
fn checked(pic: Picture) -> Result<Picture, String> {
    if !is_handful(&pic.numbers, pic.k, LINE_DOTS) || pic.numbers.last() != Some(&pic.largest) || pic.first != 1 {
        return Err(format!(
            "the friends' {:?} must be {} numbers in order on the line from 1, the largest {}",
            pic.numbers, pic.k, pic.largest
        ));
    }
    if span_sizes(&pic.numbers) != pic.gaps || pic.gaps.len() != pic.gap_count || pic.gap_count != pic.k {
        return Err(format!(
            "the gaps {:?} must be the {} non-empty stretches below and between {:?}",
            pic.gaps, pic.k, pic.numbers
        ));
    }
    let sum: u64 = pic.gaps.iter().sum();
    if sum != pic.unseen || pic.average_gap * pic.gap_count as u64 != pic.unseen {
        return Err(format!(
            "the panel's {sum} = {} and {} ÷ {} = {} must be exact",
            pic.unseen, pic.unseen, pic.gap_count, pic.average_gap
        ));
    }
    if pic.estimate != pic.largest + pic.average_gap || pic.estimate >= LINE_DOTS {
        return Err(format!(
            "the estimate {} must be {} plus one average gap, {}, on the line",
            pic.estimate, pic.largest, pic.average_gap
        ));
    }
    let top = pic.known.last().map_or(0, |&b| pic.small.saturating_sub(b));
    if !is_handful(&pic.known, pic.k, pic.small)
        || span_sizes(&pic.known) != pic.known_gaps
        || pic.known_top_gap != top
        || pic.known_top_gap == 0
        || pic.known_gaps.iter().sum::<u64>() + pic.known_top_gap + pic.k as u64 != pic.small
    {
        return Err(format!(
            "the inset's {:?}, gaps {:?} and top gap {} must fill its {} boxes",
            pic.known, pic.known_gaps, pic.known_top_gap, pic.small
        ));
    }
    Ok(pic)
}

/// Card A, the exact version of "right on average" (03_10).
fn card_a(pic: &Picture) -> String {
    format!(
        "boxes {} to N with no skips; a fair handful of {}. Right on average: over all handfuls, the estimates \
         average exactly N, for every N of at least {}. {} is one estimate, not a count.",
        pic.first, pic.k, pic.k, pic.estimate
    )
}

/// The panel's sum of the gaps (03_4), piece by piece: each gap's count,
/// a "+" between them, then "=" and the unseen total.
fn sum_pieces(pic: &Picture) -> Vec<String> {
    let mut out = Vec::new();
    for (j, g) in pic.gaps.iter().enumerate() {
        if j > 0 {
            out.push("+".to_owned());
        }
        out.push(g.to_string());
    }
    out.push("=".to_owned());
    out.push(pic.unseen.to_string());
    out
}

/// Whether piece `j` of a sum of `len` pieces is a gap's count (every
/// other piece, from the first, up to the "=").
fn is_count(j: usize, len: usize) -> bool {
    j.is_multiple_of(2) && j + 2 < len
}

/// The panel's average gap (03_4) and the estimate (03_8).
fn average_line(pic: &Picture) -> String {
    format!("{} ÷ {} = {}", pic.unseen, pic.gap_count, pic.average_gap)
}

fn estimate_line(pic: &Picture) -> String {
    format!("{} + {} = {}", pic.largest, pic.average_gap, pic.estimate)
}

/// What a live count shows for its tracker's value: `Kit::live_count`
/// rounds to the nearest whole number, halves away from zero. The lit dots
/// follow this same rule, so they always agree with the number shown.
fn counted(value: f64) -> u64 {
    value.round().max(0.0) as u64
}

/// Whether the `j`th dot (from 0) of a gap is lit while its count shows
/// `shown`.
fn lit(j: usize, shown: u64) -> bool {
    (j as u64) < shown
}

/// Seconds per dot for gap `j` (from 0).
fn per_dot_s(j: usize) -> f64 {
    COUNT_DOT_S * FASTER.powi(j.saturating_sub(1) as i32)
}

/// Where gap `a..=b`'s count sits: under the line, centred on its stretch.
fn count_spot(a: u64, b: u64, end: f64) -> Vec3 {
    v(
        (x_at(a as f64 - 0.5, end) + x_at(b as f64 + 0.5, end)) / 2.0,
        STRIP_Y - LABEL_DY,
    )
}

/// A number's place on the inset's own line, from 1 to `small`.
fn inset_x(n: f64, small: u64) -> f64 {
    INSET_X[0] + (n - 1.0) * (INSET_X[1] - INSET_X[0]) / (small as f64 - 1.0)
}

/// The inset's gaps, the top one last: (first unseen, last unseen) pairs.
fn inset_spans(pic: &Picture) -> Vec<(u64, u64)> {
    let mut out = gap_spans(&pic.known);
    if let Some(&b) = pic.known.last() {
        out.push((b + 1, pic.small));
    }
    out
}

/// One gap on the friends' line: its unseen numbers `a..=b`, its count's
/// tracker, its blue dots, its arc and the label its count becomes.
struct Gap {
    a: u64,
    b: u64,
    count: Mob,
    dots: Vec<Mob>,
    arc: Mob,
    label: Mob,
}

impl Gap {
    /// Gap `a..=b` on the line shown to `end`: its pieces detached, its
    /// dots bound to its count (each off the frame until the count shows
    /// it, then on its number).
    fn new(kit: &Kit, stage: &mut Stage<'_>, a: u64, b: u64, end: f64) -> fmn::Result<Gap> {
        let count = stage.add_value_tracker(0.0);
        let dots: Vec<Mob> = (a..=b)
            .map(|_| {
                let d = Dot::new()
                    .point(v(OFF[0], OFF[1]))
                    .radius(LIT_R)
                    .style(flat(palette::unseen(), 1.0))
                    .build();
                stage.arena_mut().add(d)
            })
            .collect();
        let placed = dots.clone();
        bind_to(stage, &[count], move |s, now| {
            let shown = counted(now[0]);
            for (j, (&d, n)) in placed.iter().zip(a..=b).enumerate() {
                let spot = if lit(j, shown) {
                    v(x_at(n as f64, end), STRIP_Y)
                } else {
                    v(OFF[0], OFF[1])
                };
                s.move_to(d, spot, ORIGIN);
            }
        })?;
        let arc = stroke(
            stage,
            arc_points(x_at(a as f64 - 0.5, end), x_at(b as f64 + 0.5, end), STRIP_Y, ARC_H),
            line_style(palette::unseen(), ARC_W, 1.0),
        );
        let label = kit.text(
            stage,
            &(b - a + 1).to_string(),
            PANEL_SIZE,
            palette::unseen(),
            count_spot(a, b, end),
        )?;
        Ok(Gap {
            a,
            b,
            count,
            dots,
            arc,
            label,
        })
    }

    /// Count the gap up, dot by dot, `per_dot` seconds each, its live count
    /// under it.
    fn count_up(&self, kit: &Kit, stage: &mut Stage<'_>, end: f64, per_dot: f64) -> fmn::Result<Mob> {
        for &d in &self.dots {
            stage.add_to_scene(d)?;
        }
        let live = kit.live_count(
            stage,
            self.count,
            PANEL_SIZE,
            palette::unseen(),
            count_spot(self.a, self.b, end),
        )?;
        let size = (self.b - self.a + 1) as f64;
        let run = self
            .count
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(per_dot * size),
                rate_func: Some(fmn::core::rate::linear),
                ..AnimateArgs::default()
            })?
            .set_value(size)?;
        stage.play(run)?;
        Ok(live)
    }

    /// The arc closes over the counted dots, and the live count gives way
    /// to the label in its place, which can move.
    fn close(&self, stage: &mut Stage<'_>, live: Mob) -> fmn::Result<()> {
        swap(stage, &[live], &[self.arc, self.label], CLOSE_S)
    }
}

/// Fade `out` away and `into` in, together, `rt` seconds (updaters on
/// `out` cleared first, so a live number fades as it stands).
fn swap(stage: &mut Stage<'_>, out: &[Mob], into: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in out {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    for &m in into {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Dim text and open strokes together, keeping their colors. The pinned
/// engine's set_opacity also changes fill alpha, so open paths must change
/// only their stroke alpha or the gap bracket becomes a filled band.
fn dim_all(stage: &mut Stage<'_>, text: &[Mob], strokes: &[Mob], o: f64, rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for (&m, stroke_only) in text.iter().map(|m| (m, false)).chain(strokes.iter().map(|m| (m, true))) {
        let a = m.animate().set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?;
        let a = if stroke_only {
            a.set_stroke(None, None, Some(o))?
        } else {
            a.set_opacity(o)?
        };
        anims.push(stage.prepare(a)?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// The centres of `pieces` laid left to right from `x`, `SUM_GAP` apart,
/// on height `y`.
fn row_of(stage: &Stage<'_>, pieces: &[Mob], x: f64, y: f64) -> Vec<Vec3> {
    let mut left = x;
    pieces
        .iter()
        .map(|&m| {
            let w = stage.get_width(m);
            let centre = v(left + w / 2.0, y);
            left += w + SUM_GAP;
            centre
        })
        .collect()
}

/// The inset's pieces, in the order they come in, and its arcs among them.
struct Inset {
    all: Vec<Mob>,
    arcs: Vec<Mob>,
}

/// The inset (03_6): a fair handful from the small count on its own line,
/// its numbers as rings, every gap (the top one too) with its arc and
/// count, the top gap's dashed bracket, the count at the line's end, and
/// the caption. Detached.
fn inset(kit: &Kit, stage: &mut Stage<'_>, pic: &Picture) -> fmn::Result<Inset> {
    let ix = |n: f64| inset_x(n, pic.small);
    let mut arcs = Vec::new();
    let mut all = vec![
        kit.text_left(
            stage,
            &format!("{INSET_CAPTION} {}", pic.small),
            INSET_SIZE,
            Palette::dim(),
            v(INSET_X[0], INSET_Y + INSET_CAPTION_DY),
        )?,
        stroke(
            stage,
            vec![v(ix(pic.first as f64), INSET_Y), v(ix(pic.small as f64), INSET_Y)],
            line_style(palette::line(), INSET_LINE_W, 1.0),
        ),
        kit.text(
            stage,
            &pic.small.to_string(),
            INSET_SIZE,
            palette::outline(),
            v(ix(pic.small as f64) + INSET_END_DX, INSET_Y),
        )?,
    ];
    for &n in &pic.known {
        all.push(ring(stage, v(ix(n as f64), INSET_Y)));
    }
    for (a, b) in inset_spans(pic) {
        let (x0, x1) = (ix(a as f64 - 0.5), ix(b as f64 + 0.5));
        let arc = stroke(
            stage,
            arc_points(x0, x1, INSET_Y, INSET_ARC_H),
            line_style(palette::unseen(), ARC_W, 1.0),
        );
        arcs.push(arc);
        all.push(arc);
        all.push(kit.text(
            stage,
            &(b - a + 1).to_string(),
            INSET_SIZE,
            palette::unseen(),
            v((x0 + x1) / 2.0, INSET_Y + ARC_LIFT + INSET_ARC_H + INSET_COUNT_DY),
        )?);
    }
    if let Some(&b) = pic.known.last() {
        let y = INSET_Y - INSET_BRACKET_DY;
        for (x0, x1) in dash_spans(ix(b as f64 + 1.0), ix(pic.small as f64)) {
            all.push(stroke(
                stage,
                vec![v(x0, y), v(x1, y)],
                line_style(palette::top_gap(), BRACKET_W, 1.0),
            ));
        }
    }
    Ok(Inset { all, arcs })
}

impl SceneConstruct for CountingTheGaps {
    fn name(&self) -> &str {
        "03_counting_the_gaps"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let pic = picture().map_err(fail)?;
        let opening = LINE_DOTS as f64;

        // the friends' line as chapter 02 left it, and the badge
        let strip = FriendsLine::build(
            &kit,
            stage,
            pic.first,
            LINE_DOTS,
            &pic.numbers,
            &[],
            Start {
                end: opening,
                fan: 0.0,
                rise: 1.0,
                widest: opening,
            },
        )?;
        let corner = badge(&kit, stage, at(BADGE_AT))?;
        let mut opened = strip.opening();
        opened.push(corner);
        fade_in_all(stage, &opened, 0.8)?;
        let largest = pic.numbers.len() - 1;
        let sixty = [strip.bodies[largest], strip.labels[largest]];

        // every gap's pieces, bound to its count; the top gap's dashes,
        // bound to how far they reach
        let mut gaps = Vec::new();
        for (a, b) in gap_spans(&pic.numbers) {
            gaps.push(Gap::new(&kit, stage, a, b, opening)?);
        }
        let reach = stage.add_value_tracker(opening);
        let from = x_at(pic.largest as f64 + 1.0, opening);
        let dash_y = STRIP_Y - BRACKET_DY;
        let dashes: Vec<Mob> = (0..dash_spans(from, STRIP_X1).len())
            .map(|_| thin(stage, palette::top_gap(), BRACKET_W, 1.0))
            .collect();
        let placed = dashes.clone();
        bind_to(stage, &[reach], move |s, now| {
            let spans = dash_spans(from, x_at(now[0], opening).min(STRIP_X1));
            for (j, &m) in placed.iter().enumerate() {
                let points = spans
                    .get(j)
                    .map_or_else(hidden, |&(x0, x1)| vec![v(x0, dash_y), v(x1, dash_y)]);
                reshape(s, m, points);
            }
        })?;

        // 03_1: below fourteen: its dots light one at a time as the count
        // under them climbs, then its arc closes over them
        let line = say(&kit, stage, "03_1")?;
        line.wait_for(stage, "Look at the numbers")?;
        let live = gaps[0].count_up(&kit, stage, opening, per_dot_s(0))?;
        line.wait_for(stage, "there are thirteen")?;
        gaps[0].close(stage, live)?;

        // 03_2: the next gap, the same way
        let line = say(&kit, stage, "03_2")?;
        line.wait_for(stage, "Between fourteen")?;
        let live = gaps[1].count_up(&kit, stage, opening, per_dot_s(1))?;
        line.wait_for(stage, "twelve more")?;
        gaps[1].close(stage, live)?;

        // 03_3: the last three, each a little faster, each closing as it
        // lands
        let line = say(&kit, stage, "03_3")?;
        for (j, phrase) in [(2, "Then five"), (3, "then seventeen"), (4, "then eight")] {
            line.wait_for(stage, phrase)?;
            let live = gaps[j].count_up(&kit, stage, opening, per_dot_s(j))?;
            gaps[j].close(stage, live)?;
        }

        // 03_4: the gaps; their counts slide into the panel's sum, then
        // the average gap
        let line = say(&kit, stage, "03_4")?;
        line.wait_for(stage, "These stretches are the gaps")?;
        let arcs: Vec<Mob> = gaps.iter().map(|g| g.arc).collect();
        flash_lagged(stage, &arcs, 1.0, 0.2)?;
        line.wait_for(stage, "Five gaps hold")?;
        // the sum's pieces: a gap's label where its count is, the signs and
        // the total new (the total blue, an unseen count)
        let pieces = sum_pieces(&pic);
        let mut labels = gaps.iter().map(|g| g.label);
        let mut sum: Vec<(Mob, bool)> = Vec::new();
        for (j, piece) in pieces.iter().enumerate() {
            if is_count(j, pieces.len()) {
                let label = labels
                    .next()
                    .ok_or_else(|| fail("a sum piece with no gap".to_owned()))?;
                sum.push((label, true));
            } else {
                let color = if j + 1 == pieces.len() {
                    palette::unseen()
                } else {
                    Palette::ink()
                };
                sum.push((kit.text(stage, piece, PANEL_SIZE, color, panel(0))?, false));
            }
        }
        let sum_mobs: Vec<Mob> = sum.iter().map(|&(m, _)| m).collect();
        let spots = row_of(stage, &sum_mobs, PANEL_X, PANEL_Y[0]);
        let (mut slides, mut signs) = (Vec::new(), Vec::new());
        for (&(m, slides_in), &spot) in sum.iter().zip(&spots) {
            if slides_in {
                slides.push((m, spot));
            } else {
                stage.move_to(m, spot, ORIGIN);
                signs.push(m);
            }
        }
        move_all(stage, &slides, 1.0, SLIDE_S)?;
        fade_in_all(stage, &signs, 0.4)?;
        line.wait_for(stage, "That's eleven per gap")?;
        let blue = [
            (pic.unseen.to_string(), palette::unseen()),
            (pic.average_gap.to_string(), palette::unseen()),
        ];
        let t2c: Vec<(&str, Srgb)> = blue.iter().map(|(s, c)| (s.as_str(), *c)).collect();
        let average = kit.text_t2c(stage, &average_line(&pic), PANEL_SIZE, Palette::ink(), &t2c, panel(1))?;
        stage.move_to(average, panel(1), LEFT);
        fade_in_all(stage, &[average], 0.5)?;

        // 03_5: one more gap, above sixty: the dashed bracket from 61 off
        // the faded end, and its "?"
        let line = say(&kit, stage, "03_5")?;
        line.wait_for(stage, "one more gap")?;
        let mut tail: Vec<Mob> = dashes.iter().rev().copied().collect();
        fade_in_lagged(stage, &tail, 0.8, 0.2)?;
        line.wait_for(stage, "It's the top gap")?;
        let tail_q = kit.text(
            stage,
            UNKNOWN,
            PANEL_SIZE,
            palette::top_gap(),
            v(TAIL_X, dash_y - UNDER_BRACKET),
        )?;
        fade_in_all(stage, &[tail_q], 0.4)?;
        line.wait_for(stage, "nobody can see it")?;
        flash_all(stage, &[tail_q], 0.6)?;
        tail.push(tail_q);

        // 03_6: in a fair handful the top gap is nothing special: a fair
        // handful from seventy, whose top gap can be seen, and is one more
        // gap among the others
        let line = say(&kit, stage, "03_6")?;
        line.wait_for(stage, "In a fair handful")?;
        flash_all(stage, &[corner], 0.6)?;
        line.wait_for(stage, "It's just one more space")?;
        let shown = inset(&kit, stage, &pic)?;
        fade_in_lagged(stage, &shown.all, 1.2, 0.1)?;
        flash_lagged(stage, &shown.arcs, 1.2, 0.2)?;
        line.wait_for(stage, "out of sight")?;
        flash_all(stage, &tail, 0.6)?;

        // 03_7: the prediction; the inset goes and the countdown runs
        say(&kit, stage, "03_7")?;
        fade_out_all(stage, &shown.all, 0.5)?;
        let q = think(&kit, stage, at(THINK_AT))?;

        // 03_8: about seventy-one: the "?"s go, the green tick lands; then
        // the reason: sixty, plus one average gap, the bracket shortening
        // to it and turning green
        let line = say(&kit, stage, "03_8")?;
        let x_est = x_at(pic.estimate as f64, opening);
        let tick = stroke(
            stage,
            vec![v(x_est, STRIP_Y - TICK_DOWN), v(x_est, STRIP_Y + TICK_UP)],
            line_style(palette::estimate(), TICK_W, 1.0),
        );
        let about = kit.text(
            stage,
            &format!("≈ {}", pic.estimate),
            PANEL_SIZE,
            palette::estimate(),
            v(x_est, STRIP_Y + ESTIMATE_DY),
        )?;
        swap(stage, &[q, tail_q], &[tick, about], 0.5)?;
        line.wait_for(stage, "Sixty, plus")?;
        flash_all(stage, &sixty, 0.6)?;
        line.wait_for(stage, "plus one average gap")?;
        stage.play(tween(reach, pic.estimate as f64, SHRINK_S)?)?;
        let guess = stroke(
            stage,
            bracket_points(pic.largest as f64 + 1.0, pic.estimate as f64, opening),
            line_style(palette::estimate(), BRACKET_W, 1.0),
        );
        let guess_x = (from + x_est) / 2.0;
        let guess_count = kit.text(
            stage,
            &pic.average_gap.to_string(),
            PANEL_SIZE,
            palette::estimate(),
            v(guess_x, dash_y - UNDER_BRACKET),
        )?;
        swap(stage, &dashes, &[guess, guess_count], 0.6)?;
        line.wait_for(stage, "of eleven")?;
        let roles = [
            (pic.largest.to_string(), palette::seen()),
            (pic.average_gap.to_string(), palette::unseen()),
            (pic.estimate.to_string(), palette::estimate()),
        ];
        let t2c: Vec<(&str, Srgb)> = roles.iter().map(|(s, c)| (s.as_str(), *c)).collect();
        let instance = kit.text_t2c(stage, &estimate_line(&pic), PANEL_SIZE, Palette::ink(), &t2c, panel(2))?;
        stage.move_to(instance, panel(2), LEFT);
        fade_in_all(stage, &[instance], 0.5)?;

        // 03_9: the recipe: the sums give way to it, the friends' case
        // moves under it; its two ingredients flash; then the condition,
        // the badge moving beside the claim
        let line = say(&kit, stage, "03_9")?;
        line.wait_for(stage, "That's a recipe")?;
        let mut sums = sum_mobs.clone();
        sums.push(average);
        fade_out_all(stage, &sums, 0.4)?;
        let names = [
            ("estimate", palette::estimate()),
            ("largest", palette::seen()),
            ("average gap", palette::unseen()),
        ];
        let recipe = kit.text_t2c(stage, RECIPE, PANEL_SIZE, Palette::ink(), &names, panel(0))?;
        stage.move_to(recipe, panel(0), LEFT);
        let w = stage.get_width(instance);
        fade_in_all(stage, &[recipe], 0.5)?;
        move_all(stage, &[(instance, v(PANEL_X + w / 2.0, PANEL_Y[1]))], 1.0, 0.6)?;
        line.wait_for(stage, "Take the largest number")?;
        flash_all(stage, &sixty, 0.6)?;
        line.wait_for(stage, "add the average gap")?;
        flash_all(stage, &[guess, guess_count], 0.6)?;
        line.wait_for(stage, "If the handful is fair")?;
        let claim = kit.text_left(stage, CLAIM, PANEL_SIZE, Palette::ink(), panel(3))?;
        fade_in_all(stage, &[claim], 0.4)?;
        let beside = PANEL_X + stage.get_width(claim) + BADGE_GAP + stage.get_width(corner) / 2.0;
        move_all(stage, &[(corner, v(beside, PANEL_Y[3]))], 1.0, 0.8)?;
        line.wait_for(stage, "right on average")?;
        flash_all(stage, &[claim, corner], 0.6)?;

        // 03_10: a promise about the recipe, not about seventy-one: the
        // estimate dims while the recipe and its condition stay bright;
        // card A
        let line = say(&kit, stage, "03_10")?;
        let card_text = card_a(&pic);
        let card = precisely(&kit, stage, &card_text)?;
        fade_in_all(stage, &[card], 0.4)?;
        let card_up = now(stage);
        line.wait_for(stage, "a promise about the recipe")?;
        flash_all(stage, &[recipe], 0.6)?;
        line.wait_for(stage, "not about seventy-one")?;
        dim_all(stage, &[instance, about, guess_count], &[tick, guess], DIM_O, 0.6)?;
        wait_until(stage, line.start + line.len)?;
        pause(stage, END_BEAT)?;
        // card A stays until it can be read
        wait_until(stage, card_up + read_s(&card_text))?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use explainer_kit::Script;
    use explainer_kit::animatic::wrap;
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::*;
    use crate::chapters::draw::{BOX_H, BOX_RISE, BOX_W, SMALL, THINK_R};
    use crate::chapters::friends_line::{BADGE, BADGE_PAD, BADGE_SIZE, RING_R, est_width};
    use crate::model::{FRIENDS, doubling_recipe, gap_recipe, gaps};

    const NEAR: f64 = 1e-9;
    /// The rings of the inset's row are this far apart at least, outline
    /// to outline (as chapter 02's rows).
    const RING_GAP: f64 = 0.02;
    /// A "precisely:" card's line, beside its label: the kit starts at 92
    /// characters and narrows by 4 until the widest line fits.
    const CARD_CHARS_WIDEST: usize = 92;
    const CARD_CHARS_NARROWEST: usize = 64;

    fn good() -> Picture {
        picture().unwrap()
    }

    /// The scene draws the facts: the friends' five and their gaps, as the
    /// model computes them; the estimate the model's recipe gives; the
    /// inset's handful is chapter 02's first row from seventy.
    #[test]
    fn the_picture_is_the_facts() {
        let pic = good();
        assert_eq!(pic.numbers, FRIENDS);
        assert_eq!(pic.gaps, gaps(&FRIENDS));
        assert_eq!(pic.known, facts::small_handfuls()[0]);
        assert_eq!(gap_recipe(&FRIENDS).whole(), Some(pic.estimate as i128));
        assert_eq!(
            sum_pieces(&pic).join(" "),
            format!(
                "{} = {}",
                pic.gaps.iter().map(u64::to_string).collect::<Vec<_>>().join(" + "),
                pic.unseen
            )
        );
        assert_eq!(
            average_line(&pic),
            format!("{} ÷ {} = {}", pic.unseen, pic.k, pic.average_gap)
        );
        assert_eq!(
            estimate_line(&pic),
            format!("{} + {} = {}", pic.largest, pic.average_gap, pic.estimate)
        );
        // the sum's pieces: the ones the scene fills with the gaps' labels
        // are the gaps' counts, in order, and only those
        let pieces = sum_pieces(&pic);
        let counts: Vec<&str> = (0..pieces.len())
            .filter(|&j| is_count(j, pieces.len()))
            .map(|j| pieces[j].as_str())
            .collect();
        let shown: Vec<String> = pic.gaps.iter().map(u64::to_string).collect();
        assert_eq!(counts, shown.iter().map(String::as_str).collect::<Vec<_>>());
        assert!(
            !is_count(pieces.len() - 1, pieces.len()),
            "the total is new, not a gap's label"
        );
    }

    /// The negative controls: each picture the words or the scene would
    /// contradict is refused, one wrong fact at a time, for its reason.
    #[test]
    fn a_picture_the_words_contradict_is_refused() {
        let pic = good();
        let refused = |p: Picture, why: &str| {
            let e = checked(p).expect_err(why);
            assert!(e.contains(why), "refused for {e:?}, not {why:?}");
        };
        assert!(checked(pic.clone()).is_ok());
        // a handful with neighbours has an empty gap, which nothing counts
        let mut touching = pic.clone();
        touching.numbers = vec![14, 15, 33, 51, 60];
        touching.gaps = vec![13, 0, 17, 17, 8];
        refused(touching, "non-empty stretches");
        refused(
            Picture {
                gaps: vec![13, 12, 6, 16, 8],
                ..pic.clone()
            },
            "non-empty stretches",
        );
        refused(
            Picture {
                unseen: pic.unseen + 1,
                ..pic.clone()
            },
            "must be exact",
        );
        refused(
            Picture {
                average_gap: pic.average_gap + 1,
                estimate: pic.estimate + 1,
                ..pic.clone()
            },
            "must be exact",
        );
        // the rival recipe, double the average (chapter 10), is on the line
        // but is not this chapter's estimate
        let doubled = doubling_recipe(&FRIENDS).nearest() as u64;
        assert!(doubled < LINE_DOTS && doubled != pic.estimate);
        refused(
            Picture {
                estimate: doubled,
                ..pic.clone()
            },
            "one average gap",
        );
        refused(
            Picture {
                estimate: pic.largest,
                ..pic.clone()
            },
            "one average gap",
        );
        refused(
            Picture {
                known_top_gap: pic.known_top_gap + 1,
                ..pic.clone()
            },
            "fill",
        );
        refused(
            Picture {
                known: vec![30, 33, 50, 53, 70],
                ..pic.clone()
            },
            "fill",
        );
        refused(Picture { small: 90, ..pic }, "fill");
    }

    /// Every phrase the scene waits for is in its line of script.tsv, once,
    /// and a line's phrases come in the order the scene waits for them.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let waits: &[(&str, &[&str])] = &[
            ("03_1", &["Look at the numbers", "there are thirteen"]),
            ("03_2", &["Between fourteen", "twelve more"]),
            ("03_3", &["Then five", "then seventeen", "then eight"]),
            (
                "03_4",
                &[
                    "These stretches are the gaps",
                    "Five gaps hold",
                    "That's eleven per gap",
                ],
            ),
            ("03_5", &["one more gap", "It's the top gap", "nobody can see it"]),
            (
                "03_6",
                &["In a fair handful", "It's just one more space", "out of sight"],
            ),
            ("03_8", &["Sixty, plus", "plus one average gap", "of eleven"]),
            (
                "03_9",
                &[
                    "That's a recipe",
                    "Take the largest number",
                    "add the average gap",
                    "If the handful is fair",
                    "right on average",
                ],
            ),
            ("03_10", &["a promise about the recipe", "not about seventy-one"]),
        ];
        for &(id, phrases) in waits {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            let mut last = 0;
            for p in phrases {
                assert_eq!(text.matches(p).count(), 1, "{id}: {p:?}");
                let i = text.find(p).unwrap();
                assert!(i >= last, "{id}: {p:?} comes after the phrase before it");
                last = i;
            }
        }
    }

    /// The count shown and the dots lit agree on every frame: both round
    /// the tracker as `Kit::live_count` does, so the first dot lights as
    /// the number turns to 1 and the last as it turns to the gap's size.
    /// The negative control: lighting by the whole part (floor) would show
    /// "1" over no lit dot halfway to the first.
    #[test]
    fn the_lit_dots_agree_with_the_count_shown() {
        for (value, shown) in [
            (0.0, 0),
            (0.49, 0),
            (0.5, 1),
            (1.5, 2),
            (12.49, 12),
            (12.5, 13),
            (13.0, 13),
        ] {
            assert_eq!(counted(value), shown, "{value}");
            assert_eq!(counted(value), value.round() as u64, "the kit's rule at {value}");
            let lights = (0..13).filter(|&j| lit(j, counted(value))).count() as u64;
            assert_eq!(lights, shown, "{value}");
        }
        assert!(lit(0, 1) && !lit(1, 1) && !lit(0, 0));
        let floor = |x: f64| x.floor() as u64;
        assert_ne!(floor(0.5), counted(0.5), "a floor rule disagrees with the number shown");
        // the last three gaps count faster, each a little more
        assert!((per_dot_s(0) - per_dot_s(1)).abs() < NEAR);
        assert!(per_dot_s(2) < per_dot_s(1) && per_dot_s(3) < per_dot_s(2) && per_dot_s(4) < per_dot_s(3));
    }

    /// The top gap's bracket shortens from the faded end to the estimate:
    /// at the start it is chapter 01's tail (from 61 to the strip's end),
    /// every dash stays inside it while it shortens and the dashes only
    /// ever go, and at the end it runs from 61 to the estimate, which is
    /// one average gap of numbers, where the green bracket takes over.
    #[test]
    fn the_top_gap_shortens_to_one_average_gap() {
        let pic = good();
        let end = LINE_DOTS as f64;
        let from = x_at(pic.largest as f64 + 1.0, end);
        let full = dash_spans(from, x_at(end, end).min(STRIP_X1));
        assert!((full[0].1 - STRIP_X1).abs() < NEAR && full.last().unwrap().0 >= from - NEAR);
        let mut before = full.len();
        for s in 0..=400 {
            let reach = end - (end - pic.estimate as f64) * s as f64 / 400.0;
            let to = x_at(reach, end).min(STRIP_X1);
            let spans = dash_spans(from, to);
            assert!(
                spans.iter().all(|&(a, b)| from - NEAR <= a && a < b && b <= to + NEAR),
                "{reach}"
            );
            assert!(spans.len() <= before, "{reach}: a dash came back");
            before = spans.len();
        }
        let short = dash_spans(from, x_at(pic.estimate as f64, end));
        assert!((short[0].1 - x_at(pic.estimate as f64, end)).abs() < NEAR);
        let guess = bracket_points(pic.largest as f64 + 1.0, pic.estimate as f64, end);
        assert!((guess[1][0] - from).abs() < NEAR && (guess[2][0] - x_at(pic.estimate as f64, end)).abs() < NEAR);
        assert_eq!(
            pic.estimate - pic.largest,
            pic.average_gap,
            "the guess covers one average gap of numbers"
        );
        assert!(short.len() < full.len());
    }

    /// The gaps' counts under the line keep apart and clear the boxes;
    /// their dots and the line's neighbouring dots keep apart.
    #[test]
    fn the_counts_on_the_line_keep_apart() {
        let pic = good();
        let end = LINE_DOTS as f64;
        let spans = gap_spans(&pic.numbers);
        let spots: Vec<(f64, f64)> = spans
            .iter()
            .map(|&(a, b)| {
                (
                    count_spot(a, b, end)[0],
                    est_width(&(b - a + 1).to_string(), PANEL_SIZE),
                )
            })
            .collect();
        for w in spots.windows(2) {
            assert!(w[1].0 - w[0].0 > (w[0].1 + w[1].1) / 2.0 + 0.3);
        }
        const {
            assert!(
                STRIP_Y - LABEL_DY + 0.2 < STRIP_Y + BOX_RISE - SMALL * BOX_H / 2.0,
                "the counts under the boxes"
            )
        };
        const {
            assert!(
                STRIP_Y - LABEL_DY - 0.2 > STRIP_Y - BRACKET_DY + 0.05,
                "the counts over the bracket"
            )
        };
        // a lit dot covers its faint dot and stays clear of its neighbour
        let step = x_at(2.0, end) - x_at(1.0, end);
        assert!(LIT_R > DOT_R && 2.0 * LIT_R < step);
        // the estimate's tick and label clear the largest box
        let x_est = x_at(pic.estimate as f64, end);
        let label_left = x_est - est_width(&format!("≈ {}", pic.estimate), PANEL_SIZE) / 2.0;
        let box_right = x_at(pic.largest as f64, end) + SMALL * BOX_W / 2.0;
        assert!(label_left > box_right + 0.2, "{label_left} beside {box_right}");
        assert!(x_est < TAIL_X - 0.4, "the tick left of where the \"?\" was");
    }

    /// The inset: its rings, its counts and its caption keep apart, it
    /// stays on the frame left of the panel, over the boxes and under the
    /// badge; every gap of its handful, the top one too, has an arc. The
    /// negative control: at the inset's scale, two neighbouring numbers
    /// would put their rings on top of each other, so the check can fail.
    #[test]
    fn the_inset_keeps_its_pieces_apart() {
        let pic = good();
        let ix = |n: f64| inset_x(n, pic.small);
        let apart = |hand: &[u64]| {
            hand.windows(2)
                .all(|w| ix(w[1] as f64) - ix(w[0] as f64) >= 2.0 * RING_R + RING_GAP)
        };
        assert!(apart(&pic.known));
        assert!(!apart(&[1, 2, 3, 4, 5]), "neighbours at the inset's scale");
        let spans = inset_spans(&pic);
        assert_eq!(spans.len(), pic.known_gaps.len() + 1);
        assert_eq!(spans.last().map(|&(a, b)| b - a + 1), Some(pic.known_top_gap));
        let counts: Vec<(f64, f64)> = spans
            .iter()
            .map(|&(a, b)| {
                let x = (ix(a as f64 - 0.5) + ix(b as f64 + 0.5)) / 2.0;
                (x, est_width(&(b - a + 1).to_string(), INSET_SIZE))
            })
            .collect();
        for w in counts.windows(2) {
            assert!(w[1].0 - w[0].0 > (w[0].1 + w[1].1) / 2.0 + 0.05, "{w:?}");
        }
        let safe_x = FRAME_X_RADIUS - 0.25;
        let end_label = ix(pic.small as f64) + INSET_END_DX + est_width(&pic.small.to_string(), INSET_SIZE) / 2.0;
        assert!(INSET_X[0] > -safe_x && end_label < PANEL_X);
        let caption = format!("{INSET_CAPTION} {}", pic.small);
        assert!(INSET_X[0] + est_width(&caption, INSET_SIZE) < PANEL_X);
        const {
            assert!(
                INSET_Y - INSET_BRACKET_DY - 0.1 > STRIP_Y + BOX_RISE + SMALL * BOX_H / 2.0,
                "the inset over the boxes"
            )
        };
        const {
            assert!(
                INSET_Y + INSET_CAPTION_DY + 0.15 < BADGE_AT[1] - 0.15 - BADGE_PAD[1],
                "the inset under the badge"
            )
        };
        // the top gap's dashed bracket fits under its own gap
        let b = *pic.known.last().unwrap();
        let dashes = dash_spans(ix(b as f64 + 1.0), ix(pic.small as f64));
        assert!(!dashes.is_empty() && dashes.iter().all(|&(x0, _)| x0 >= ix(b as f64 + 1.0) - NEAR));
        // the countdown's ring, when it comes, is right of the inset
        const { assert!(THINK_AT[0] - THINK_R > INSET_X[1] + 1.0) };
    }

    /// The panel's lines, the claim and the badge beside it stay in the
    /// title-safe area, and the badge's height fits between the friends'
    /// case and the bottom of the panel.
    #[test]
    fn the_panel_fits_the_frame() {
        let pic = good();
        let safe_x = FRAME_X_RADIUS - 0.25;
        let sum = sum_pieces(&pic);
        let sum_w: f64 = sum.iter().map(|p| est_width(p, PANEL_SIZE)).sum::<f64>() + SUM_GAP * (sum.len() - 1) as f64;
        for w in [
            sum_w,
            est_width(&average_line(&pic), PANEL_SIZE),
            est_width(&estimate_line(&pic), PANEL_SIZE),
            est_width(RECIPE, PANEL_SIZE),
        ] {
            assert!(PANEL_X + w < safe_x, "{w}");
        }
        let badge_w = est_width(BADGE, BADGE_SIZE) + 2.0 * BADGE_PAD[0];
        assert!(PANEL_X + est_width(CLAIM, PANEL_SIZE) + BADGE_GAP + badge_w < safe_x);
        // the badge's half height, its words' (0.15) and its pad; a panel
        // line's half height, 0.16
        const {
            assert!(
                PANEL_Y[3] + 0.15 + BADGE_PAD[1] < PANEL_Y[1] - 0.16,
                "over the friends' case"
            )
        };
        const {
            assert!(
                PANEL_Y[3] - 0.15 - BADGE_PAD[1] > THINK_AT[1] + 0.2,
                "over where the countdown was"
            )
        };
        const { assert!(PANEL_Y[0] + 0.2 < FRAME_Y_RADIUS - 0.15) };
    }

    /// Card A says the exact condition and what "right on average" means
    /// in at most three lines at every width the lower third might narrow
    /// to, with the facts' numbers, and is up long enough to be read. The
    /// negative control: the card's first, longer draft would not fit.
    #[test]
    fn card_a_fits_and_is_read() {
        let pic = good();
        let text = card_a(&pic);
        assert!(text.contains(&format!("{} is one estimate, not a count", pic.estimate)));
        for chars in (CARD_CHARS_NARROWEST..=CARD_CHARS_WIDEST).step_by(4) {
            assert!(wrap(&text, chars, 4).len() <= 3, "{chars}: {text}");
        }
        let draft = format!(
            "boxes numbered {} to N with no skips, and a fair handful of {}. Right on average: over every \
             possible handful, the estimates average exactly N, for every N of at least {}. {} is one estimate, \
             not a count.",
            pic.first, pic.k, pic.k, pic.estimate
        );
        assert!(wrap(&draft, CARD_CHARS_NARROWEST, 4).len() > 3);
        assert!(read_s(&text) < 15.0);
    }
}
