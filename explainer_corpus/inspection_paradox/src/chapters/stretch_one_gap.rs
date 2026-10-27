//! Chapter 03, stretch one gap. A controlled experiment on chapter 02's
//! timetable, the same buses, tags and passengers:
//! - 03_1: a timetable with no surprise, the red bus six minutes after the
//!   blue, every gap six minutes.
//! - 03_2: the gaps from each red bus to the next blue bus turn gold. Half
//!   the buses end one, and half the passengers stand in one: the two
//!   counts agree, the negative control.
//! - 03_3, 03_4: only the red bus moves, a minute at a time. There are
//!   still ten buses and a six-minute average gap. The buses' count stays
//!   at a half while the passengers' count follows the gold gap's length,
//!   7, 8, 9, 10 of every 12, a point on a straight line each step.
//! - 03_5: what share stands in the long gap with the buses together? (a
//!   prediction; nothing moves.)
//! - 03_6: all of them. With two buses at each moment there is no bus share
//!   to show, only the sign's average gap over buses.
//! - 03_7: back to our timetable. Each gap is counted once, but your
//!   mornings land in it in proportion to its length.
//!
//! One tracker, the red bus's offset, drives the experiment. Updaters keep
//! the red markers, the gold gaps and the passengers' colors true to it
//! between whole minutes. Every number on screen is read from the facts,
//! and the scene refuses to start if the model disagrees with a fact it
//! shows at any offset.

use explainer_kit::{Facts, Kit, Palette, fade_all, tween, v};
use fmn::prelude::*;

use super::draw::{
    LINE_Y, PASSENGER_R, fact, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, flash_lagged, flat,
    line_style, now, say, stand, think, x_of, you,
};
use super::timetable::{self, Gap, drop_in_lagged, fade_in_with, fade_out_with, gap_color, recolor, x_at, your_moment};
use crate::facts;
use crate::model::{self, EVEN_OFFSET, OFFSET, PERIOD, Q};
use crate::palette;

pub struct StretchOneGap {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are read from the facts)
const ENDS_IT: &str = "tags on buses that end it:";
const IN_IT: &str = "passengers in it:";
const OF_EVERY: &str = "of every";
const ASK: &str = "passengers in the long gap, buses together?";
const OVER_BUSES: &str = "average gap over buses:";
const MIN: &str = "min";
const AND_SO_ON: &str = "...";
const PLOT_X: &str = "gap length";
const PLOT_Y: &str = "share of passengers";

/// 03_4's spoken steps: the word the readout changes on, and the step (its
/// place in F-G1.offsets) whose passengers of every twelve it says. "Seven"
/// is already on screen (03_3's first step); the voice names it.
const STEPS_SPOKEN: [(usize, &str); 4] = [(1, "Seven"), (2, "eight"), (3, "nine"), (4, "ten.")];

// ------------------------------------------------------------------ layout

/// The readouts, right of the sign's card (its right edge is near
/// x = −2.45): words from READ_X, values in a column from VALUE_X.
const READ_X: f64 = -1.9;
const VALUE_X: f64 = 0.9;
const UPPER_Y: f64 = -1.2;
const LOWER_Y: f64 = -1.65;
const READ_SIZE: f64 = 22.0;
/// 03_5: the question under the readouts, the think ring to its right.
const ASK_AT: Vec3 = [0.0, -2.4, 0.0];
const ASK_SIZE: f64 = 22.0;
const THINK_AT: Vec3 = [3.3, -2.7, 0.0];
/// 03_4's plot, lower right: the passengers' share against the gold gap's
/// length, from 0 to 12 minutes and from none to all.
const PLOT_O: Vec3 = [4.5, -3.2, 0.0];
const PLOT_XS: f64 = 0.18;
const PLOT_YS: f64 = 2.0;
const PLOT_SIZE: f64 = 16.0;
const PLOT_TICK_DY: f64 = 0.22;
const PLOT_TICK_DX: f64 = 0.1;
const PLOT_NAME_DY: f64 = 0.5;
const PLOT_Y_NAME_DX: f64 = 0.7;
const PLOT_Y_NAME_DY: f64 = 0.25;
/// Timing: the red bus's first slide, a minute's step, a step on a spoken
/// number (03_4's "eight, nine, ten" fall about half a second apart), 03_4's
/// visual hold (the storyboard's "listen 3 s": no sound teaches, so music
/// only).
const SLIDE_RT: f64 = 1.6;
const STEP_RT: f64 = 0.6;
const SAID_STEP_RT: f64 = 0.4;
const BREATH_S: f64 = 3.0;
/// You step down onto the line (03_7).
const STEP_ON: f64 = 0.45;

// ------------------------------------------------------------- the numbers

/// The experiment's numbers, read from the facts and checked against the
/// model at every offset the experiment visits.
struct Plan {
    /// The red bus's offsets, six down to none (F-G1.offsets), as numbers
    /// and as printed.
    offsets: Vec<i64>,
    printed: Vec<String>,
    /// The passengers of every twelve in the gold gap at each offset
    /// (F-G1.passengers_of_12): also the gold gap's length, the blue
    /// bus's tag.
    twelfths: Vec<String>,
    period: String,
    bus_share: String,
    mean_gap: String,
    /// The hour's two gaps with the buses together: none, and twelve.
    zero: String,
    twelve: String,
}

impl Plan {
    /// The lower readout at step `i`: "7 of every 12".
    fn readout(&self, i: usize) -> String {
        format!("{} {OF_EVERY} {}", self.twelfths[i], self.period)
    }

    /// The upper readout with the buses together: "6 min (0, 12, 0, 12, ...)".
    fn together(&self) -> String {
        format!(
            "{} {MIN} ({z}, {t}, {z}, {t}, {AND_SO_ON})",
            self.mean_gap,
            z = self.zero,
            t = self.twelve
        )
    }
}

/// Whether the passenger at `t` stands in a gap from a red bus to the next
/// blue bus, with the red bus `d` minutes after the blue: the last bus at
/// or before `t` is a red one. With the buses together, every bus is both,
/// so every passenger does.
fn in_growing_gap(t: Q, d: i64) -> bool {
    let last = t - model::since_last(d, t);
    (last.num() - d).rem_euclid(PERIOD) == 0
}

/// A passenger's place in its twelve minutes.
fn within(t: Q) -> f64 {
    t.to_f64().rem_euclid(PERIOD as f64)
}

/// The live rule while the red bus slides (`d` any offset, whole or not): a
/// passenger is gold once the red marker has passed it, at or after the red
/// bus in its twelve minutes.
fn gold_at(within: f64, d: f64) -> bool {
    within >= d
}

/// The experiment's plan, refused when a fact it shows disagrees with the
/// model, or with the picture's own count of gold passengers, at any
/// offset.
fn plan(f: &Facts) -> Result<Plan, String> {
    let get = |key: &str| f.get(key).map(str::to_owned).ok_or_else(|| format!("no fact {key}"));
    let printed: Vec<String> = get("F-G1.offsets")?.split(',').map(str::to_owned).collect();
    let offsets: Vec<i64> = printed
        .iter()
        .map(|s| s.parse::<i64>().map_err(|e| format!("offset {s}: {e}")))
        .collect::<Result<_, _>>()?;
    if offsets != facts::stretch_offsets() {
        return Err("F-G1.offsets is not the model's six down to none".into());
    }
    let twelfths: Vec<String> = get("F-G1.passengers_of_12")?.split(',').map(str::to_owned).collect();
    if twelfths.len() != offsets.len() {
        return Err("F-G1.passengers_of_12 does not give one count per offset".into());
    }
    let period = get("F-T1.period")?;
    let bus_share = get("F-G1.bus_share_every_offset")?;
    let buses = get("F-G1.buses_every_offset")?;
    let mean_gap = get("F-G1.mean_gap_every_offset")?;
    let people = model::passengers();
    for (&d, n) in offsets.iter().zip(&twelfths) {
        let share = model::share_after_red(d);
        if (share * Q::int(PERIOD)).to_string() != *n {
            return Err(format!(
                "offset {d}: {n} of every {period}, but the model's share is {share}"
            ));
        }
        let gold = people.iter().filter(|&&t| in_growing_gap(t, d)).count() as i64;
        if Q::new(gold, people.len() as i64) != share {
            return Err(format!(
                "offset {d}: the picture colors {gold} passengers gold, not the share {share}"
            ));
        }
        if (PERIOD - d).to_string() != *n {
            return Err(format!("offset {d}: the gold gap is not {n} minutes long"));
        }
        if d > 0 {
            let blue_share = Q::new(model::route(0).len() as i64, model::bus_times(d).len() as i64);
            if blue_share.to_string() != bus_share {
                return Err(format!(
                    "offset {d}: the buses ending the gold gap are {blue_share}, not {bus_share}"
                ));
            }
            for (&t, &tag) in model::bus_times(d).iter().zip(&model::gap_before_each_bus(d)) {
                let want = if t % PERIOD == 0 { PERIOD - d } else { d };
                if tag != want {
                    return Err(format!("offset {d}: the bus at :{t:02} carries {tag}, not {want}"));
                }
            }
        }
        if model::bus_times(d).len().to_string() != buses {
            return Err(format!("offset {d}: not {buses} buses an hour"));
        }
        if model::mean_gap(&model::hour_gaps(d)).to_string() != mean_gap {
            return Err(format!("offset {d}: the average gap over buses is not {mean_gap}"));
        }
    }
    let spoken: Vec<String> = STEPS_SPOKEN.iter().map(|&(i, _)| twelfths[i].clone()).collect();
    if get("F-G1.spoken_of_12")? != spoken.join(",") {
        return Err("03_4's spoken steps are not F-G1.spoken_of_12".into());
    }
    let zero = printed.last().ok_or("no offsets")?.clone();
    let twelve = get("F-O1.together_period")?;
    let together = model::hour_gaps(0);
    let alternates = together
        .chunks(2)
        .all(|p| p.len() == 2 && p[0].to_string() == zero && p[1].to_string() == twelve);
    if !alternates {
        return Err(format!(
            "with the buses together the gaps are not {zero}, {twelve}, ..."
        ));
    }
    Ok(Plan {
        offsets,
        printed,
        twelfths,
        period,
        bus_share,
        mean_gap,
        zero,
        twelve,
    })
}

// ----------------------------------------------------------------- picture

/// Where the red marker of the `k`-th twelve minutes stands with the red bus
/// `d` minutes after the blue: at its moment, or one marker width
/// (`beside`) from the blue marker when they would overlap, so that with
/// the buses together it stands beside the blue (03_6).
fn red_x(k: usize, d: f64, beside: f64) -> f64 {
    let base = (k as i64 * PERIOD) as f64;
    x_at(base) + (x_at(d) - x_at(0.0)).max(beside)
}

/// Where a point of the plot sits: the gold gap `minutes` long, `share` of
/// the passengers in it.
fn plot_at(minutes: i64, share: Q) -> Vec3 {
    v(
        PLOT_O[0] + PLOT_XS * minutes as f64,
        PLOT_O[1] + PLOT_YS * share.to_f64(),
    )
}

/// Our timetable's marks, as chapter 02 drew them.
struct Marks {
    /// Each gap's bracket and its length.
    brackets: Vec<Mob>,
    /// The tag over each bus, the gap before it.
    tags: Vec<Mob>,
}

fn our_marks(kit: &Kit, stage: &mut Stage<'_>, f: &Facts, gaps: &[Gap]) -> fmn::Result<Marks> {
    let (_, long) = facts::short_and_long();
    let mut brackets = Vec::new();
    for g in gaps {
        let key = if g.minutes() == long { "F-T1.long" } else { "F-T1.short" };
        let b = timetable::bracket(kit, stage, g, &fact(f, key)?, gap_color(g.minutes()))?;
        brackets.extend([b.line, b.label]);
    }
    let printed = fact(f, "F-S1.bus_gap_tags")?;
    let values: Vec<&str> = printed.split(',').collect();
    let model_tags: Vec<String> = model::gap_before_each_bus(OFFSET).iter().map(i64::to_string).collect();
    if values != model_tags {
        return Err(fail("the printed tags are not the gap before each bus".into()));
    }
    let mut tags = Vec::new();
    for (&t, value) in model::bus_times(OFFSET).iter().zip(values) {
        tags.push(timetable::tag(kit, stage, x_of(Q::int(t)), value)?);
    }
    Ok(Marks { brackets, tags })
}

/// The tags at step `i`: over each blue marker the gold gap's length, over
/// each red marker the offset (both checked against the model by `plan`).
fn tags_at(kit: &Kit, stage: &mut Stage<'_>, plan: &Plan, i: usize, beside: f64) -> fmn::Result<Vec<Mob>> {
    let d = plan.offsets[i] as f64;
    let mut out = Vec::new();
    for k in 0..model::route(0).len() {
        let base = (k as i64 * PERIOD) as f64;
        out.push(timetable::tag(kit, stage, x_at(base), &plan.twelfths[i])?);
        out.push(timetable::tag(kit, stage, red_x(k, d, beside), &plan.printed[i])?);
    }
    Ok(out)
}

/// The plot's point for step `i`, gold.
fn plot_point(stage: &mut Stage<'_>, plan: &Plan, i: usize) -> Mob {
    let d = plan.offsets[i];
    stage.arena_mut().add(
        Dot::new()
            .point(plot_at(PERIOD - d, model::share_after_red(d)))
            .radius(PASSENGER_R)
            .style(flat(palette::long_gap(), 1.0))
            .build(),
    )
}

/// The plot's frame: two axes, their ticks (facts, checked against the
/// model) and their names.
fn plot_frame(kit: &Kit, stage: &mut Stage<'_>, f: &Facts) -> fmn::Result<Vec<Mob>> {
    let x_end = plot_at(PERIOD, Q::int(0));
    let y_end = plot_at(0, Q::int(1));
    let mut out = vec![
        stage.arena_mut().add(
            Line::new(PLOT_O, x_end)
                .style(line_style(Palette::dim(), 2.0))
                .build()?,
        ),
        stage.arena_mut().add(
            Line::new(PLOT_O, y_end)
                .style(line_style(Palette::dim(), 2.0))
                .build()?,
        ),
    ];
    for (minutes, key) in [(EVEN_OFFSET, "F-O1.even_gap"), (PERIOD, "F-T1.period")] {
        let label = fact(f, key)?;
        if label != minutes.to_string() {
            return Err(fail(format!("{key} is {label}, not the tick at {minutes} min")));
        }
        let at = plot_at(minutes, Q::int(0));
        out.push(kit.text(stage, &label, PLOT_SIZE, Palette::dim(), v(at[0], at[1] - PLOT_TICK_DY))?);
    }
    for (d, key) in [(EVEN_OFFSET, "F-G1.share.d6"), (0, "F-G1.share.d0")] {
        let share = model::share_after_red(d);
        let label = fact(f, key)?;
        if label != share.to_string() {
            return Err(fail(format!("{key} is {label}, not the tick at {share}")));
        }
        let at = plot_at(0, share);
        let tick = kit.text(stage, &label, PLOT_SIZE, Palette::dim(), at)?;
        stage.move_to(tick, v(at[0] - PLOT_TICK_DX, at[1]), RIGHT);
        out.push(tick);
    }
    out.push(kit.text(
        stage,
        PLOT_X,
        PLOT_SIZE,
        Palette::dim(),
        v((PLOT_O[0] + x_end[0]) / 2.0, PLOT_O[1] - PLOT_NAME_DY),
    )?);
    out.push(kit.text(
        stage,
        PLOT_Y,
        PLOT_SIZE,
        Palette::dim(),
        v(PLOT_O[0] + PLOT_Y_NAME_DX, y_end[1] + PLOT_Y_NAME_DY),
    )?);
    Ok(out)
}

/// The straight line the points lie on, from the model's two ends: no gold
/// gap and no passengers (the red bus a whole period after the blue), the
/// whole twelve minutes and every passenger (the buses together).
fn trend_line(stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    let line = Line::new(
        plot_at(0, model::share_after_red(PERIOD)),
        plot_at(PERIOD, model::share_after_red(0)),
    )
    .style(line_style(Palette::faint(), 2.0))
    .build()?;
    Ok(stage.arena_mut().add(line))
}

/// An updater keeping each red marker the tracker `d`'s minutes after its
/// blue bus.
fn move_reds(stage: &mut Stage<'_>, d: Mob, reds: &[Mob], beside: f64) -> fmn::Result<()> {
    let anchor = stage.arena_mut().add(VMobject::new());
    let reds = reds.to_vec();
    stage.add_updater(
        anchor,
        move |s, _| {
            let dv = s.tracker_value(d).unwrap_or(0.0);
            for (k, &m) in reds.iter().enumerate() {
                s.set_x(m, red_x(k, dv, beside));
            }
        },
        true,
    )?;
    stage.add_to_scene(anchor)?;
    Ok(())
}

/// An updater keeping the gold gaps (`spans`, one per twelve minutes) from
/// each red bus to the next blue bus, and each passenger gold once the red
/// marker has passed it, ink before. Returns its anchor; clear its
/// updaters to let the passengers go back to their gaps' colors.
fn paint_stretch(stage: &mut Stage<'_>, d: Mob, spans: &[Mob], dots: &[Mob], people: &[Q]) -> fmn::Result<Mob> {
    let anchor = stage.arena_mut().add(VMobject::new());
    let spans = spans.to_vec();
    let dots = dots.to_vec();
    let places: Vec<f64> = people.iter().map(|&t| within(t)).collect();
    let mut shown: Vec<Option<bool>> = vec![None; dots.len()];
    let (gold, ink) = (palette::long_gap(), Palette::ink());
    stage.add_updater(
        anchor,
        move |s, _| {
            let dv = s.tracker_value(d).unwrap_or(0.0);
            for (k, &m) in spans.iter().enumerate() {
                let base = (k as i64 * PERIOD) as f64;
                let path = polyline(timetable::span_points(base + dv, base + PERIOD as f64)).build();
                let _ = s.set_points(m, path.points());
            }
            for (i, &m) in dots.iter().enumerate() {
                let now_gold = gold_at(places[i], dv);
                if shown[i] != Some(now_gold) {
                    s.set_color(m, if now_gold { gold } else { ink }, None, true);
                    shown[i] = Some(now_gold);
                }
            }
        },
        true,
    )?;
    stage.add_to_scene(anchor)?;
    Ok(anchor)
}

/// The experiment's moving parts: the offset tracker, the tags and the
/// lower readout's value on screen, the plot's points so far, and a
/// marker's width.
struct Stretch {
    d: Mob,
    tags: Vec<Mob>,
    value: Mob,
    points: Vec<Mob>,
    beside: f64,
}

/// One step of the experiment, over `rt` seconds: the red bus to offset `i`
/// of the plan (the updaters move its markers, the gold gaps and the
/// passengers' colors), the tags and the lower readout changing with it,
/// and the plot gaining its point. With `tags` false the old tags leave and
/// none come (the buses together: two buses at one moment).
fn step(
    kit: &Kit,
    stage: &mut Stage<'_>,
    s: &mut Stretch,
    plan: &Plan,
    i: usize,
    rt: f64,
    tags: bool,
) -> fmn::Result<()> {
    let new_tags = if tags {
        tags_at(kit, stage, plan, i, s.beside)?
    } else {
        Vec::new()
    };
    let value = kit.text_left(
        stage,
        &plan.readout(i),
        READ_SIZE,
        palette::long_gap(),
        v(VALUE_X, LOWER_Y),
    )?;
    let point = plot_point(stage, plan, i);
    let mut old = s.tags.clone();
    old.push(s.value);
    // Discrete counts describe the endpoints. Crossfading them at the same
    // anchor superimposes different digits (notably 9 and 10). Clear the old
    // counts, move the actual experiment, then reveal its new counts. The
    // total duration remains rt, preserving the authored spoken step timing.
    fade_out_all(stage, &old, rt * 0.25)?;
    stage.play(tween(s.d, plan.offsets[i] as f64, rt * 0.5)?)?;
    let mut coming = new_tags.clone();
    coming.extend([value, point]);
    fade_in_all(stage, &coming, rt * 0.25)?;
    s.tags = new_tags;
    s.value = value;
    s.points.push(point);
    Ok(())
}

impl SceneConstruct for StretchOneGap {
    fn name(&self) -> &str {
        "03_stretch_one_gap"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let f = facts::facts().map_err(fail)?;
        let plan = plan(&f).map_err(fail)?;
        let (short, _) = facts::short_and_long();
        let people = model::passengers();
        let gaps = timetable::gaps(OFFSET);
        let gap_of = timetable::gap_of_each(&gaps).map_err(fail)?;
        let even = plan.offsets[0];
        let together = plan.offsets.len() - 1;

        // 03_1: our timetable as chapter 02 left it; the marks of our gaps
        // leave and the passengers go back to ink; the red bus slides to six
        // minutes after the blue, and every gap is six
        let line = say(&kit, stage, "03_1")?;
        let table = timetable::timetable(&kit, stage, &f, OFFSET)?;
        let ours = our_marks(&kit, stage, &f, &gaps)?;
        let dots: Vec<Mob> = people
            .iter()
            .zip(&gap_of)
            .map(|(&t, &g)| timetable::passenger(stage, t, gap_color(gaps[g].minutes())))
            .collect();
        let mut opening = table.mobs();
        opening.extend(&ours.brackets);
        opening.extend(&ours.tags);
        opening.extend(&dots);
        fade_in_all(stage, &opening, 0.8)?;
        line.wait_for(stage, "Move the red bus")?;
        let ink: Vec<AnimBuilder> = dots
            .iter()
            .map(|&m| recolor(m, Palette::ink(), 0.5))
            .collect::<fmn::Result<_>>()?;
        let mut marks = ours.brackets.clone();
        marks.extend(&ours.tags);
        fade_out_with(stage, &marks, ink, 0.5)?;
        let d = stage.add_value_tracker(OFFSET as f64);
        let beside = stage.get_width(table.blue[0]);
        move_reds(stage, d, &table.red, beside)?;
        stage.play(tween(d, even as f64, SLIDE_RT)?)?;
        line.wait_for(stage, "Now every gap")?;
        let even_label = fact(&f, "F-O1.even_gap")?;
        let mut evens = Vec::new();
        for g in &timetable::gaps(even) {
            let b = timetable::bracket(&kit, stage, g, &even_label, gap_color(g.minutes()))?;
            evens.extend([b.line, b.label]);
        }
        let tags = tags_at(&kit, stage, &plan, 0, beside)?;
        let mut coming = evens.clone();
        coming.extend(&tags);
        fade_in_all(stage, &coming, 0.6)?;

        // 03_2: the gaps from each red bus to the next blue bus turn gold
        // with the passengers in them; half the buses end one, half the
        // passengers stand in one
        let line = say(&kit, stage, "03_2")?;
        line.wait_for(stage, "from each red bus")?;
        let spans: Vec<Mob> = (0..table.red.len())
            .map(|k| {
                let base = (k as i64 * PERIOD) as f64;
                timetable::span(stage, base + even as f64, base + PERIOD as f64, palette::long_gap())
            })
            .collect();
        let gold: Vec<AnimBuilder> = (0..dots.len())
            .filter(|&i| in_growing_gap(people[i], even))
            .map(|i| recolor(dots[i], palette::long_gap(), 0.6))
            .collect::<fmn::Result<_>>()?;
        fade_in_with(stage, &spans, gold, 0.6)?;
        let painter = paint_stretch(stage, d, &spans, &dots, &people)?;
        line.wait_for(stage, "Half the buses")?;
        let upper = [
            kit.text_left(stage, ENDS_IT, READ_SIZE, Palette::dim(), v(READ_X, UPPER_Y))?,
            kit.text_left(stage, &plan.bus_share, READ_SIZE, Palette::dim(), v(VALUE_X, UPPER_Y))?,
        ];
        fade_in_all(stage, &upper, 0.5)?;
        flash_lagged(stage, &table.blue, 1.0, 0.15)?;
        line.wait_for(stage, "half the passengers")?;
        let lower = kit.text_left(stage, IN_IT, READ_SIZE, palette::long_gap(), v(READ_X, LOWER_Y))?;
        let value = kit.text_left(
            stage,
            &plan.readout(0),
            READ_SIZE,
            palette::long_gap(),
            v(VALUE_X, LOWER_Y),
        )?;
        fade_in_all(stage, &[lower, value], 0.5)?;
        line.wait_for(stage, "The two counts agree")?;
        flash_all(stage, &[upper[1], value], 0.6)?;
        let mut s = Stretch {
            d,
            tags,
            value,
            points: Vec::new(),
            beside,
        };

        // 03_3: the even brackets leave (the gaps are about to change); the
        // plot and its first point; the first step; the sign still holds
        let line = say(&kit, stage, "03_3")?;
        fade_out_all(stage, &evens, 0.4)?;
        let mut plot = plot_frame(&kit, stage, &f)?;
        let first = plot_point(stage, &plan, 0);
        plot.push(first);
        fade_in_all(stage, &plot, 0.6)?;
        s.points.push(first);
        line.wait_for(stage, "a minute at a time")?;
        step(&kit, stage, &mut s, &plan, 1, STEP_RT, true)?;
        line.wait_for(stage, "The same ten buses")?;
        flash_all(stage, &[table.sign.buses, table.sign.every], 0.8)?;

        // 03_4: the buses' count stays, the passengers' count follows the
        // gold gap's length, a step on each number said; then the straight
        // line, and a hold to compare the two readouts
        let line = say(&kit, stage, "03_4")?;
        line.wait_for(stage, "never moves")?;
        flash_all(stage, &[upper[1]], 0.6)?;
        line.wait_for(stage, "follows its length")?;
        flash_all(stage, &[s.value], 0.6)?;
        for (i, word) in STEPS_SPOKEN {
            line.wait_for(stage, word)?;
            if i == s.points.len() - 1 {
                let newest = s.points[i];
                flash_all(stage, &[s.value, newest], 0.5)?;
            } else {
                step(&kit, stage, &mut s, &plan, i, SAID_STEP_RT, true)?;
            }
        }
        let trend = trend_line(stage)?;
        fade_in_all(stage, &[trend], 0.5)?;
        kit.music_breath(stage, BREATH_S)?;

        // 03_5: the question; think time; nothing moves
        let line = say(&kit, stage, "03_5")?;
        line.wait_for(stage, "What share")?;
        let ask = kit.text(stage, ASK, ASK_SIZE, Palette::ink(), ASK_AT)?;
        fade_in_all(stage, &[ask], 0.4)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 03_6: all of them; the red bus steps on to join the blue, the plot
        // gaining a point at each offset passed, every passenger gold, the
        // tags gone; the upper readout becomes the sign's average gap over
        // buses, zero gaps included
        let line = say(&kit, stage, "03_6")?;
        fade_out_all(stage, &[q, ask], 0.3)?;
        for i in s.points.len()..=together {
            step(&kit, stage, &mut s, &plan, i, STEP_RT, false)?;
        }
        line.wait_for(stage, "And the sign")?;
        let over = [
            kit.text_left(stage, OVER_BUSES, READ_SIZE, Palette::dim(), v(READ_X, UPPER_Y))?,
            kit.text_left(stage, &plan.together(), READ_SIZE, Palette::dim(), v(VALUE_X, UPPER_Y))?,
        ];
        fade_out_all(stage, &upper, 0.3)?;
        fade_in_all(stage, &over, 0.4)?;
        flash_all(stage, &[table.sign.every, over[1]], 0.8)?;

        // 03_7: back to our timetable, the readouts gone and the plot kept;
        // a tag on each bus, one per gap; the passengers back in their gaps'
        // colors, and you, in a long gap
        let line = say(&kit, stage, "03_7")?;
        let back_rt = STEP_RT * (OFFSET - plan.offsets[together]) as f64;
        let mut leaving = vec![lower, s.value];
        leaving.extend(over);
        fade_out_with(stage, &leaving, vec![tween(d, OFFSET as f64, back_rt)?], back_rt)?;
        stage.clear_updaters(painter, true);
        line.wait_for(stage, "counts each gap once")?;
        let ours = our_marks(&kit, stage, &f, &gaps)?;
        fade_in_lagged(stage, &ours.tags, 1.2, 0.2)?;
        line.wait_for(stage, "in proportion to its length")?;
        let teal: Vec<AnimBuilder> = (0..dots.len())
            .filter(|&i| gaps[gap_of[i]].minutes() == short)
            .map(|i| recolor(dots[i], timetable::short_gap(), 0.6))
            .collect::<fmn::Result<_>>()?;
        fade_out_with(stage, &spans, teal, 0.6)?;
        fade_in_all(stage, &ours.brackets, 0.5)?;
        flash_all(stage, &[trend], 0.8)?;
        line.wait_for(stage, "Stretch a gap")?;
        let me = you(stage, v(x_of(your_moment()), LINE_Y))?;
        drop_in_lagged(stage, &[me], STEP_ON, 0.6, 0.0)?;
        let up = now(stage);
        stand(&kit, stage, up)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use explainer_kit::{Narrator, Script};

    use super::*;

    /// A marker's width as `draw.rs` builds it (private there); the scene
    /// measures it from the blue marker instead.
    const MARKER_W: f64 = 0.14;

    /// Needs chapter 03's lines in script.tsv (the integration step moves
    /// them from the draft). The whole chapter runs headless at 10 fps with
    /// the scratch voice: every phrase it waits for is in its line, every
    /// fact it reads exists, the plan holds at every offset, and its seven
    /// lines are said in order. It ends with `kit.finish`, whose timeline
    /// runs past the end of the last line.
    #[test]
    fn the_chapter_runs_and_says_its_lines_in_order() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let narrator = Narrator::estimate(&script);
        let kit = Kit::new().unwrap().with_narrator(Rc::clone(&narrator));
        let mut scene = StretchOneGap { kit };
        let config = fmn::scene::RuntimeConfig {
            fps: 10,
            ..fmn::scene::RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut fmn::scene::NullSceneSink).unwrap();
        let said_at = narrator.said();
        let said: Vec<&str> = said_at.iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(said, ["03_1", "03_2", "03_3", "03_4", "03_5", "03_6", "03_7"]);
        let (last, at) = said_at.last().unwrap();
        let last_ends = at + narrator.length(last).unwrap();
        let end = scene
            .kit
            .finished
            .borrow()
            .as_ref()
            .expect("no timeline: the chapter must end with kit.finish")
            .seconds();
        assert!(
            end > last_ends,
            "the timeline ends at {end:.2} s, before {last} ends at {last_ends:.2} s"
        );
    }

    /// The plan holds at every offset, and a fact that stops matching the
    /// model is refused: a wrong count at offset 1 (12 of every 12 instead
    /// of 11).
    #[test]
    fn the_plan_is_the_model_at_every_offset_and_a_wrong_fact_is_refused() {
        let f = facts::facts().unwrap();
        let p = plan(&f).unwrap();
        assert_eq!(p.offsets[0], EVEN_OFFSET);
        assert_eq!(p.offsets[p.offsets.len() - 1], 0);
        assert!(p.offsets.contains(&OFFSET));
        let mut counts: Vec<String> = p.twelfths.clone();
        let one = p.offsets.iter().position(|&d| d == 1).unwrap();
        counts[one] = PERIOD.to_string();
        let changed = counts.join(",");
        let mut wrong = Facts::new();
        for row in f.print().lines() {
            let (key, value) = row.split_once('\t').unwrap();
            wrong
                .add(
                    key,
                    if key == "F-G1.passengers_of_12" {
                        changed.as_str()
                    } else {
                        value
                    },
                )
                .unwrap();
        }
        let refused = plan(&wrong).err().expect("a wrong count must be refused");
        assert!(refused.contains("offset 1"), "{refused}");
    }

    /// The live rule the updater colors by agrees with the model's rule at
    /// every whole offset from twelve down to none, passenger by passenger,
    /// and the gold share is the model's share.
    #[test]
    fn the_live_rule_is_the_models_rule_at_every_whole_offset() {
        let people = model::passengers();
        for d in 0..=PERIOD {
            let mut gold = 0;
            for &t in &people {
                let live = gold_at(within(t), d as f64);
                if d < PERIOD {
                    assert_eq!(live, in_growing_gap(t, d), "offset {d}, passenger at {t}");
                }
                gold += i64::from(live);
            }
            assert_eq!(
                Q::new(gold, people.len() as i64),
                model::share_after_red(d),
                "offset {d}"
            );
        }
    }

    /// Gold means standing between a red marker and the next blue marker
    /// on screen, at every offset from six down to one.
    #[test]
    fn gold_is_between_a_red_marker_and_the_next_blue() {
        for d in 1..=EVEN_OFFSET {
            for &t in &model::passengers() {
                let x = x_of(t);
                let between = (0..model::route(0).len()).any(|k| {
                    let next_blue = x_at(((k as i64 + 1) * PERIOD) as f64);
                    red_x(k, d as f64, MARKER_W) <= x && x < next_blue
                });
                assert_eq!(between, in_growing_gap(t, d), "offset {d}, passenger at {t}");
            }
        }
    }

    /// The red marker never covers the blue: it is at its moment whenever
    /// the two stand a marker's width apart, and beside the blue marker
    /// otherwise, down to the buses together.
    #[test]
    fn the_red_marker_never_covers_the_blue() {
        for k in 0..model::route(0).len() {
            let blue = x_at((k as i64 * PERIOD) as f64);
            for hundredth in 0..=600 {
                let d = f64::from(hundredth) / 100.0;
                let x = red_x(k, d, MARKER_W);
                assert!(x - blue >= MARKER_W - 1e-12, "k {k}, offset {d}");
                if x_at(d) - x_at(0.0) >= MARKER_W {
                    assert!((x - x_at((k as i64 * PERIOD) as f64 + d)).abs() < 1e-12);
                }
            }
        }
    }

    /// Every point of the plot lies on the straight line from the model's
    /// two ends, inside the frame, one per offset.
    #[test]
    fn the_points_lie_on_the_straight_line_inside_the_frame() {
        let f = facts::facts().unwrap();
        let p = plan(&f).unwrap();
        let a = plot_at(0, model::share_after_red(PERIOD));
        let b = plot_at(PERIOD, model::share_after_red(0));
        assert_eq!(a, PLOT_O);
        for &d in &p.offsets {
            let c = plot_at(PERIOD - d, model::share_after_red(d));
            let cross = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
            assert!(cross.abs() < 1e-12, "offset {d}");
            assert!(c[0] >= a[0] && c[0] <= b[0] && c[1] >= a[1] && c[1] <= b[1]);
        }
    }

    /// The numbers 03_4 says are the readouts on screen as it says them.
    #[test]
    fn the_spoken_steps_are_the_readouts() {
        let f = facts::facts().unwrap();
        let p = plan(&f).unwrap();
        let words = ["seven", "eight", "nine", "ten"];
        for (n, (i, word)) in STEPS_SPOKEN.into_iter().enumerate() {
            let said = word.trim_end_matches('.').to_lowercase();
            assert_eq!(said, words[n]);
            assert_eq!(p.twelfths[i], (7 + n).to_string(), "{word}");
        }
    }

    /// The readouts, the question, the think ring and the plot stay apart
    /// and on screen, right of the sign's card. Widths are estimated, not
    /// measured: 0.0045 units a point a character (bayesian_search's
    /// measured face), a line 0.6 of its size tall.
    #[test]
    fn the_panel_stays_apart_and_on_screen() {
        let f = facts::facts().unwrap();
        let p = plan(&f).unwrap();
        let w = |s: &str, pt: f64| s.chars().count() as f64 * pt * 0.0045;
        let h = |pt: f64| pt * 0.0045 / 0.55 * 0.6;
        let left = |s: &str, pt: f64, x: f64, y: f64| (x, x + w(s, pt), y - h(pt), y + h(pt));
        let centred = |s: &str, pt: f64, x: f64, y: f64| (x - w(s, pt) / 2.0, x + w(s, pt) / 2.0, y - h(pt), y + h(pt));
        let apart = |a: (f64, f64, f64, f64), b: (f64, f64, f64, f64)| a.1 < b.0 || b.1 < a.0 || a.3 < b.2 || b.3 < a.2;
        let on_screen = |a: (f64, f64, f64, f64)| a.0 > -7.1 && a.1 < 7.1 && a.2 > -4.0 && a.3 < 4.0;
        let sign_right = -4.3 + 3.7 / 2.0;
        let widest = (0..p.offsets.len())
            .map(|i| p.readout(i))
            .max_by_key(|s| s.chars().count())
            .unwrap();
        let ring = (
            THINK_AT[0] - 0.62,
            THINK_AT[0] + 0.62,
            THINK_AT[1] - 0.62,
            THINK_AT[1] + 0.62,
        );
        let note = centred("pause for more time", 18.0, THINK_AT[0], THINK_AT[1] - 0.62 - 0.3);
        let frame = (
            PLOT_O[0] - PLOT_TICK_DX - w("1/2", PLOT_SIZE),
            plot_at(PERIOD, Q::int(1))[0] + PASSENGER_R,
            PLOT_O[1] - PLOT_NAME_DY - h(PLOT_SIZE),
            PLOT_O[1] + PLOT_YS + PLOT_Y_NAME_DY + h(PLOT_SIZE),
        );
        let boxes = [
            left(ENDS_IT, READ_SIZE, READ_X, UPPER_Y),
            left(&p.together(), READ_SIZE, VALUE_X, UPPER_Y),
            left(IN_IT, READ_SIZE, READ_X, LOWER_Y),
            left(&widest, READ_SIZE, VALUE_X, LOWER_Y),
            centred(ASK, ASK_SIZE, ASK_AT[0], ASK_AT[1]),
            ring,
            note,
            frame,
        ];
        assert!(left(OVER_BUSES, READ_SIZE, READ_X, UPPER_Y).1 < VALUE_X);
        for (i, &a) in boxes.iter().enumerate() {
            assert!(on_screen(a) && a.0 > sign_right, "{a:?}");
            for &b in &boxes[i + 1..] {
                assert!(apart(a, b), "{a:?} {b:?}");
            }
        }
        // the plot's name over its frame stays under the passengers' row
        assert!(frame.3 < timetable::DOT_Y - PASSENGER_R);
    }

    /// The chapter's fixed words carry no digits: every number on screen is
    /// read from the facts.
    #[test]
    fn the_fixed_words_carry_no_numbers() {
        for w in [
            ENDS_IT, IN_IT, OF_EVERY, ASK, OVER_BUSES, MIN, AND_SO_ON, PLOT_X, PLOT_Y,
        ] {
            assert!(!w.chars().any(|c| c.is_ascii_digit()), "{w}");
        }
    }
}
