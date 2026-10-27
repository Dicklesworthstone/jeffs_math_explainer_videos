//! Chapter 11, back to the stop. Chapter 01's picture comes back, and the
//! model meets its limit with the same objects:
//! - 11_1: the street strip with you at the stop, the hour, both routes'
//!   buses and the sign, rebuilt as 01_3 left them. Every promise holds.
//! - 11_2: the tags over the buses, then the sixty passengers, as in 02;
//!   five sixths of them stand in long gaps. The wait triangles rise over
//!   the whole hour, to scale.
//! - 11_3: one twelve-minute stretch, its triangles' area over the time
//!   they cover: four minutes twenty, boxed.
//! - 11_4: you, out of the strip, one minute before the red bus at :26:
//!   your wait? (a prediction.)
//! - 11_5: one minute, the triangle's own height at your moment. You no
//!   longer turn up at random, so the triangles' average is not yours.
//! - 11_6: every passenger plans: each dot slides forward, never back, to
//!   one minute before the first bus it can make with a minute to spare
//!   (`model::planned_arrivals`). Ten stack before each blue bus and two
//!   before each red, every dot in its new gap's color.
//! - 11_7: still five in six in long gaps, but every wait is one minute:
//!   the share of long gaps alone never gave the wait. The exact version
//!   (K09).
//!
//! The street is chapter 01's own drawing (`the_street_is_chapter_01s`
//! checks the copy against `the_stop.rs`). The stacks come from the dots
//! and the timetable alone, and `stop` refuses a fact that stops matching
//! them. Every number on screen is read from the facts.

use explainer_kit::props::precisely;
use explainer_kit::{Facts, Kit, Palette, Timed, fade_all, live_bar, rect, tween, v};
use fmn::prelude::*;

use super::draw::{
    LINE_X0, LINE_X1, LINE_Y, PASSENGER_R, STAND_UP, fade_in_all, fade_in_lagged, fade_out_all, fade_to, fail,
    flash_all, flash_lagged, flat, group, line_style, marker_top, min_s, now, pause, read_s, say, shrink_to, steady,
    think, wait_until, x_of, you,
};
use super::timetable::{self, DOT_Y, Gap, fade_in_with, fade_out_with, gap_color, recolor, stroke_to, x_at};
use super::triangles;
use super::twelve_minutes::grow_lagged;
use super::two_averages::written;
use crate::facts;
use crate::model::{self, EARLY, HOUR, OFFSET, PERIOD, Q};
use crate::palette;

pub struct BackToTheStop {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are read from the facts)
const HOME: &str = "home";
const STOP: &str = "bus stop";
const STATION: &str = "station";
const ON_TIME: &str = "every bus on time";
const AREA: &str = "min²";
const MIN: &str = "min";
const TIMES: &str = "×";
const DIVIDED: &str = "÷";
const THE_SAME: &str = "the same";
const AVERAGE_WAIT: &str = "average wait:";
const ASK: &str = "your wait?";
const WAIT: &str = "wait";
const SHARE_NAME: &str = "share of passengers in long gaps";
const WAIT_NAME: &str = "average wait";
/// The exact version of 11_5 and 11_7's claim (K09), around the 2 of its
/// formula (the halves of F-S1.wait_share_of_gap, as 06's K03 writes g/2).
const K09_HEAD: &str = "the average wait E[G²]/(";
const K09_TAIL: &str = "E[G]) is for arrival moments independent of the timetable and spread evenly in time; \
                        passengers who plan, or who arrive with other passengers' timing, need a model of how they \
                        arrive";

// ------------------------------------------------------------------ layout

/// The street of 01_1 and its strip, exactly as `the_stop.rs` draws them.
const STREET_Y: f64 = 0.0;
const HOME_X: f64 = -5.0;
const STOP_X: f64 = 0.0;
const STATION_X: f64 = 5.0;
const GROUND_HALF: f64 = 6.5;
const STREET_LABEL_DY: f64 = -0.8;
const STREET_LABEL_SIZE: f64 = 20.0;
const STRIP_SCALE: f64 = 0.5;
const STRIP_AT: Vec3 = [0.0, 3.05, 0.0];
/// 01_6's green dot over every bus, and its words.
const ON_TIME_DY: f64 = 0.18;
const ON_TIME_AT: Vec3 = [0.0, 1.55, 0.0];
/// The triangles over the hour: a minute up as long as a minute across
/// (the hour's own scale), see-through so the markers show, behind them.
const TRI_FILL: f64 = 0.35;
const TRI_BRIGHT: f64 = 0.6;
const TRI_DIM: f64 = 0.12;
const TRI_GONE: f64 = 0.05;
const TRI_Z: i32 = -1;
const GROW_RT: f64 = 2.0;
/// 11_3: the dimmed stretches' label, the bright triangles' areas (the
/// short one's left of it, the long one's at its centroid), the stretch's
/// bracket under the line, and the verdict box, lower right.
const SAME_AT: Vec3 = [5.3, 2.2, 0.0];
const SAME_SIZE: f64 = 20.0;
const AREA_SIZE: f64 = 18.0;
const SHORT_AREA_AT: Vec3 = [-6.12, 0.85, 0.0];
const STRETCH_Y: f64 = 0.0;
const STRETCH_TICK: f64 = 0.1;
const STRETCH_INSET: f64 = 0.04;
const STRETCH_LABEL_Y: f64 = -0.22;
const STRETCH_SIZE: f64 = 20.0;
const BOX_AT: Vec3 = [3.6, -1.55, 0.0];
const BOX_SIZE: f64 = 24.0;
const BOX_PAD: [f64; 2] = [0.2, 0.15];
const BOX_W: f64 = 2.0;
const BOX_DIM: f64 = 0.35;
/// 11_4, 11_5: you under the passengers' row (at the line your ring would
/// cross the markers a minute away), a thin line up to your moment; the
/// question, then your wait, right of you, its segment over the triangles
/// (set before it joins the scene: a z-index set later does not re-sort);
/// the think ring top right.
const YOU_Y: f64 = -0.95;
const YOU_Z: i32 = 3;
const YOU_RT: f64 = 1.2;
const GUIDE_W: f64 = 2.0;
const ASK_DX: f64 = 0.3;
const ASK_SIZE: f64 = 24.0;
const THINK_AT: Vec3 = [6.0, 2.4, 0.0];
const SEGMENT_W: f64 = 5.0;
/// 11_6, 11_7: the panel at the top once the strip has gone: each row's
/// name right-aligned left of its bar, its value right of it.
const NAME_X: f64 = -0.3;
const BAR_X0: f64 = 0.0;
const SHARE_W: f64 = 3.0;
const BAR_H: f64 = 0.26;
const ROW_YS: [f64; 2] = [3.45, 2.95];
const PANEL_SIZE: f64 = 22.0;
const VALUE_X: f64 = 3.2;
const WAIT_UNIT: f64 = 0.6;
const WAIT_RT: f64 = 2.0;
/// The planners' stacks hang down from the passengers' row, a dot this far
/// under the one above; each stack's wait under it.
const STACK_DY: f64 = 0.125;
const STACK_LABEL_DY: f64 = 0.23;
const STACK_LABEL_SIZE: f64 = 12.0;
const SLIDE_RT: f64 = 3.5;
/// Emphasis: an outline around a text, faded in, held, faded out.
const HALO_PAD: f64 = 0.1;
const HALO_W: f64 = 3.0;
const HALO_RT: f64 = 0.9;

// ------------------------------------------------------------- the numbers

/// A fact's printed fraction ("5/6", "1") as a number.
fn parse_q(key: &str, printed: &str) -> Result<Q, String> {
    let int = |s: &str| s.parse::<i64>().map_err(|e| format!("{key} = {printed}: {e}"));
    match printed.split_once('/') {
        Some((n, d)) => Ok(Q::new(int(n)?, int(d)?)),
        None => Ok(Q::int(int(printed)?)),
    }
}

/// Moment `t` brought into the hour: the timetable repeats, so a planner
/// who stands at 61 minutes stands at :01.
pub fn in_hour(t: Q) -> Q {
    let h = Q::int(HOUR);
    let hours = (t / h).num().div_euclid((t / h).den());
    t - h * Q::int(hours)
}

/// The red bus you plan for in 11_4: the red bus nearest the middle of the
/// hour, clear of the sign and of the clock labels.
pub fn your_bus() -> i64 {
    model::route(OFFSET)
        .into_iter()
        .min_by_key(|&b| (2 * b - HOUR).abs())
        .unwrap_or(OFFSET)
}

/// Your moment in 11_4: [`EARLY`] before your bus.
pub fn your_moment() -> Q {
    Q::int(your_bus() - EARLY)
}

/// Each planner's place in its stack (0 at the top, on the passengers'
/// row): the planners who stand at the same moment of the hour, the one
/// with least far to slide on top.
pub fn ranks(ready: &[Q], planned: &[Q]) -> Vec<usize> {
    let far = |j: usize| planned[j] - ready[j];
    (0..ready.len())
        .map(|i| {
            (0..ready.len())
                .filter(|&j| in_hour(planned[j]) == in_hour(planned[i]))
                .filter(|&j| far(j) < far(i) || (far(j) == far(i) && j < i))
                .count()
        })
        .collect()
}

/// The stacks: each moment of the hour where planners stand, with how many
/// stand there, in time order.
pub fn stacks(planned: &[Q]) -> Vec<(Q, usize)> {
    let mut spots: Vec<Q> = planned.iter().map(|&t| in_hour(t)).collect();
    spots.sort();
    spots.dedup();
    spots
        .into_iter()
        .map(|s| (s, planned.iter().filter(|&&t| in_hour(t) == s).count()))
        .collect()
}

/// The passengers ready less than [`EARLY`] before a bus: they cannot make
/// it with a minute to spare, so they slide on to the next one.
pub fn too_late(ready: &[Q]) -> Vec<usize> {
    (0..ready.len())
        .filter(|&i| model::wait_at(OFFSET, ready[i]) < Q::int(EARLY))
        .collect()
}

/// Every number chapter 11 prints, read from the facts and checked against
/// the timetable, the triangles, your moment and the planners.
pub struct Stop {
    /// The tag on each bus, in bus order.
    tags: Vec<String>,
    /// 11_3: the bright stretch's two triangles' areas, all of them, the
    /// time they cover, how many such stretches, and the average wait.
    areas: [String; 2],
    total_area: String,
    period: String,
    cycles: String,
    wait_s: String,
    /// 11_5: your wait.
    your_wait: String,
    /// 11_6, 11_7: the planners' share of long gaps (as drawn and as a
    /// glyph), each planner's wait, and their average spelt out.
    pub long_share: Q,
    share: String,
    planned_wait: String,
    planned_wait_s: String,
    /// The 2 of K09's formula.
    halves: String,
    /// Where each planner stands (minutes past the hour, before wrapping),
    /// in passenger order, and its place in its stack.
    pub planned: Vec<Q>,
    pub ranks: Vec<usize>,
    /// The average waits the bar falls between, in minutes.
    pub random_wait: Q,
    pub planned_mean: Q,
}

impl Stop {
    /// "2 min²", "50 min²".
    fn area_labels(&self) -> [String; 2] {
        [format!("{} {AREA}", self.areas[0]), format!("{} {AREA}", self.areas[1])]
    }

    /// "12 min", under the bright stretch.
    fn stretch_label(&self) -> String {
        format!("{} {MIN}", self.period)
    }

    /// "the same × 5".
    fn same_label(&self) -> String {
        format!("{THE_SAME} {TIMES} {}", self.cycles)
    }

    /// "average wait: 52 min² ÷ 12 min = 4 min 20 s".
    fn verdict(&self) -> String {
        format!(
            "{AVERAGE_WAIT} {} {AREA} {DIVIDED} {} {MIN} = {}",
            self.total_area, self.period, self.wait_s
        )
    }

    /// "wait: 1 min", beside you.
    fn your_label(&self) -> String {
        format!("{WAIT}: {} {MIN}", self.your_wait)
    }

    /// "wait 1 min", under each stack.
    fn stack_label(&self) -> String {
        format!("{WAIT} {} {MIN}", self.planned_wait)
    }

    /// K09, the storyboard's exact version of 11_5 and 11_7.
    fn k09(&self) -> String {
        format!("{K09_HEAD}{}{K09_TAIL}", self.halves)
    }
}

/// The chapter's numbers, refused when a fact the picture prints or the
/// voice says stops matching what is drawn.
pub fn stop(f: &Facts) -> Result<Stop, String> {
    let get = |key: &str| f.get(key).map(str::to_owned).ok_or_else(|| format!("no fact {key}"));
    let (short, long) = facts::short_and_long();
    let hour = model::hour_gaps(OFFSET);
    let tag_values = model::gap_before_each_bus(OFFSET);
    let tags: Vec<String> = get("F-S1.bus_gap_tags")?.split(',').map(str::to_owned).collect();
    if tags != tag_values.iter().map(i64::to_string).collect::<Vec<_>>() {
        return Err("the printed tags are not the gap before each bus".into());
    }
    // 11_3: the first stretch's triangles, their area over the time
    let stretch: Vec<Gap> = timetable::gaps(OFFSET).into_iter().filter(|g| g.to <= PERIOD).collect();
    let lengths: Vec<i64> = stretch.iter().map(Gap::minutes).collect();
    if lengths != [short, long] {
        return Err(format!(
            "the first stretch's gaps are {lengths:?}, not the short then the long"
        ));
    }
    let area: Q = lengths.iter().map(|&g| model::triangle_area(g)).sum();
    let random_wait = area / Q::int(PERIOD);
    if random_wait != model::mean_wait(&hour) {
        return Err("one stretch's area over its time is not the hour's average wait".into());
    }
    // 11_4, 11_5: you, the timetable reader
    let yours = your_moment();
    let your_wait = model::wait_at(OFFSET, yours);
    // 11_6, 11_7: the planners
    let ready = model::passengers();
    let planned = model::planned_arrivals(OFFSET);
    let ranks = ranks(&ready, &planned);
    let n = Q::int(planned.len() as i64);
    let waits: Vec<Q> = planned.iter().map(|&t| model::wait_at(OFFSET, t)).collect();
    let planned_mean = waits.iter().copied().sum::<Q>() / n;
    let mut distinct = waits.clone();
    distinct.sort();
    distinct.dedup();
    let in_long = planned.iter().filter(|&&t| model::gap_at(OFFSET, t) == long).count();
    let share = Q::new(in_long as i64, planned.len() as i64);
    let before = |route: i64| -> Vec<usize> {
        let mut sizes: Vec<usize> = model::route(route)
            .into_iter()
            .map(|b| {
                let spot = in_hour(Q::int(b - EARLY));
                stacks(&planned)
                    .iter()
                    .find(|&&(s, _)| s == spot)
                    .map_or(0, |&(_, k)| k)
            })
            .collect();
        sizes.sort_unstable();
        sizes.dedup();
        sizes
    };
    let mut spots: Vec<i64> = stacks(&planned)
        .iter()
        .map(|&(s, _)| s.num().rem_euclid(PERIOD))
        .collect();
    spots.sort_unstable();
    spots.dedup();
    let ahead = planned
        .iter()
        .map(|&t| model::wait_at(OFFSET, t) / Q::int(model::gap_at(OFFSET, t)))
        .sum::<Q>()
        / n;
    let truth = |b: bool| if b { "true" } else { "false" }.to_owned();
    let checks = [
        ("buses", tag_values.len().to_string(), "F-T1.buses_per_hour"),
        ("the average gap", model::mean_gap(&hour).to_string(), "F-T1.mean_gap"),
        (
            "the long gaps' share of the time",
            model::time_share(&hour, |g| g == long).to_string(),
            "F-T1.long_time_share",
        ),
        (
            "the short triangle's area",
            model::triangle_area(short).to_string(),
            "F-A1.short_area",
        ),
        (
            "the long triangle's area",
            model::triangle_area(long).to_string(),
            "F-A1.long_area",
        ),
        ("the stretch's area", area.to_string(), "F-A1.total_area"),
        ("the stretch's time", PERIOD.to_string(), "F-T1.period"),
        (
            "stretches in the hour",
            (HOUR / PERIOD).to_string(),
            "F-T1.cycles_per_hour",
        ),
        ("the average wait", model::clock(random_wait)?, "F-W1.mean_wait_clock"),
        ("how early you turn up", EARLY.to_string(), "F-K1.early"),
        ("your wait", your_wait.to_string(), "F-K1.wait"),
        ("planners", planned.len().to_string(), "F-K2.passengers"),
        ("planners before each blue bus", facts::csv(before(0)), "F-K2.per_blue"),
        (
            "planners before each red bus",
            facts::csv(before(OFFSET)),
            "F-K2.per_red",
        ),
        (
            "every planner ready before it stands",
            truth(planned.iter().zip(&ready).all(|(p, r)| p >= r)),
            "F-K2.never_early",
        ),
        ("the planners' waits", facts::csv(&distinct), "F-K2.waits"),
        (
            "the planners' average wait",
            model::clock(planned_mean)?,
            "F-K2.mean_wait_clock",
        ),
        ("the planners in long gaps", share.to_string(), "F-K2.in_long_share"),
        (
            "the planners in long gaps, the time share",
            share.to_string(),
            "F-T1.long_time_share",
        ),
        (
            "the planners in short gaps",
            (Q::int(1) - share).to_string(),
            "F-K2.in_short_share",
        ),
        (
            "where in its stretch each stack stands",
            facts::csv(&spots),
            "F-K2.stand_at",
        ),
        (
            "the share of its gap ahead of a planner",
            ahead.to_string(),
            "F-K2.wait_share_of_gap",
        ),
    ];
    for (what, drawn, key) in checks {
        let shown = get(key)?;
        if drawn != shown {
            return Err(format!("{what}: the picture draws {drawn}, but {key} is {shown}"));
        }
    }
    // the planners' even share of long gaps never gave their wait: a random
    // moment has half its gap ahead, a planner much less
    let random_ahead = parse_q("F-S1.wait_share_of_gap", &get("F-S1.wait_share_of_gap")?)?;
    if random_ahead == ahead {
        return Err("the planners have as much of their gap ahead as random moments".into());
    }
    if random_ahead.num() != 1 {
        return Err(format!("K09 writes the share ahead as 1/n, not {random_ahead}"));
    }
    let long_share = parse_q("F-K2.in_long_share", &get("F-K2.in_long_share")?)?;
    let spell = |key: &str| -> Result<String, String> { min_s(&get(key)?).map_err(|e| e.to_string()) };
    Ok(Stop {
        tags,
        areas: [get("F-A1.short_area")?, get("F-A1.long_area")?],
        total_area: get("F-A1.total_area")?,
        period: get("F-T1.period")?,
        cycles: get("F-T1.cycles_per_hour")?,
        wait_s: spell("F-W1.mean_wait_clock")?,
        your_wait: get("F-K1.wait")?,
        long_share,
        share: written(long_share)?,
        planned_wait: get("F-K2.waits")?,
        planned_wait_s: spell("F-K2.mean_wait_clock")?,
        halves: random_ahead.den().to_string(),
        planned,
        ranks,
        random_wait,
        planned_mean,
    })
}

// ----------------------------------------------------------------- picture

/// The street of 01_1, detached: the ground, the house, the stop with its
/// sign, the station with its train and clock, and their labels. Chapter
/// 01's function, copied word for word (it is private there).
fn street(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let y = STREET_Y;
    let ground = Line::new(v(-GROUND_HALF, y - 0.4), v(GROUND_HALF, y - 0.4))
        .style(line_style(Palette::faint(), 2.0))
        .build()?;
    let house = rect(v(HOME_X - 0.5, y - 0.4), 1.0, 0.8, flat(Palette::faint(), 1.0))?;
    let roof = Polygon::new([
        v(HOME_X - 0.65, y + 0.4),
        v(HOME_X + 0.65, y + 0.4),
        v(HOME_X, y + 0.95),
    ])
    .style(flat(Palette::dim(), 1.0))
    .build();
    let post = Line::new(v(STOP_X, y - 0.4), v(STOP_X, y + 0.85))
        .style(line_style(Palette::dim(), 3.0))
        .build()?;
    let plate = rect(v(STOP_X - 0.32, y + 0.85), 0.64, 0.36, flat(Palette::dim(), 1.0))?;
    let station = rect(v(STATION_X - 1.0, y - 0.4), 2.0, 1.1, flat(Palette::faint(), 1.0))?;
    let train = rect(v(STATION_X - 0.8, y - 0.32), 1.6, 0.34, flat(Palette::dim(), 1.0))?;
    let clock = Circle::new()
        .radius(0.2)
        .arc_center(v(STATION_X, y + 0.98))
        .style(line_style(Palette::dim(), 2.5))
        .build();
    let hand = Line::new(v(STATION_X, y + 0.98), v(STATION_X, y + 1.12))
        .style(line_style(Palette::dim(), 2.5))
        .build()?;
    let mut parts: Vec<Mob> = [ground, house, roof, post, plate, station, train, clock, hand]
        .into_iter()
        .map(|m| stage.arena_mut().add(m))
        .collect();
    for (word, x) in [(HOME, HOME_X), (STOP, STOP_X), (STATION, STATION_X)] {
        parts.push(kit.text(
            stage,
            word,
            STREET_LABEL_SIZE,
            Palette::dim(),
            v(x, y + STREET_LABEL_DY),
        )?);
    }
    Ok(parts)
}

/// Where you stand at the stop in the street, full size (01_1's walk ends
/// here).
fn at_the_stop() -> Vec3 {
    v(STOP_X - 0.35, STREET_Y - 0.25)
}

/// A minute on the hour line, across and up alike.
pub fn hour_unit() -> f64 {
    (LINE_X1 - LINE_X0) / HOUR as f64
}

/// Moment `t` with a wait of `w` minutes above it, over the hour line.
pub fn over_hour(t: f64, w: f64) -> Vec3 {
    v(x_at(t), LINE_Y + w * hour_unit())
}

/// Each gap's triangle over the hour, in its gap's color, behind the
/// markers; detached.
fn hour_triangles(stage: &mut Stage<'_>, gaps: &[Gap]) -> Vec<Mob> {
    gaps.iter()
        .map(|g| {
            let corners = triangles::triangle(g).map(|(t, w)| over_hour(t, w));
            let m = stage.arena_mut().add(
                Polygon::new(corners)
                    .style(flat(gap_color(g.minutes()), TRI_FILL))
                    .build(),
            );
            stage.set_z_index(m, TRI_Z, true);
            m
        })
        .collect()
}

/// The length of the gap moment `t` (minutes into the hour) falls in, 0
/// outside the hour.
fn minutes_at(gaps: &[Gap], t: f64) -> i64 {
    gaps.iter()
        .find(|g| g.from as f64 <= t && t < g.to as f64)
        .map_or(0, Gap::minutes)
}

/// Where a planner is, `s` of the way through the slide: its moment,
/// `ready` moving forward towards `planned` and brought into the hour, and
/// its height, `s` of the way down to its place in its stack.
pub fn slide_at(ready: Q, planned: Q, rank: usize, s: f64) -> (f64, f64) {
    let t = ready.to_f64() + s * (planned - ready).to_f64();
    (t.rem_euclid(HOUR as f64), DOT_Y - s * rank as f64 * STACK_DY)
}

/// An updater that slides the passengers as the tracker `go` runs from 0
/// to 1, each in the color of the gap it is in. Returns its anchor,
/// already in the scene; clear the anchor's updaters when the slide is
/// over.
fn slide(stage: &mut Stage<'_>, dots: &[Mob], ready: &[Q], s: &Stop, go: Mob) -> fmn::Result<Mob> {
    let gaps = timetable::gaps(OFFSET);
    let paths: Vec<(Q, Q, usize)> = (0..dots.len()).map(|i| (ready[i], s.planned[i], s.ranks[i])).collect();
    let mut painted: Vec<i64> = ready.iter().map(|&r| minutes_at(&gaps, r.to_f64())).collect();
    let dots = dots.to_vec();
    let anchor = stage.arena_mut().add(VMobject::new());
    stage.add_updater(
        anchor,
        move |st, _| {
            let at = st.tracker_value(go).unwrap_or(0.0);
            for (i, &(r, p, k)) in paths.iter().enumerate() {
                let (t, y) = slide_at(r, p, k, at);
                st.move_to(dots[i], v(x_at(t), y), ORIGIN);
                let g = minutes_at(&gaps, t);
                if g != painted[i] {
                    st.set_color(dots[i], gap_color(g), None, true);
                    painted[i] = g;
                }
            }
        },
        true,
    )?;
    stage.add_to_scene(anchor)?;
    Ok(anchor)
}

/// The bracket under the line over minutes `from` to `to`, detached.
fn stretch_bracket(stage: &mut Stage<'_>, from: f64, to: f64) -> Mob {
    let (a, b) = (x_at(from) + STRETCH_INSET, x_at(to) - STRETCH_INSET);
    stage.arena_mut().add(
        polyline(vec![
            v(a, STRETCH_Y + STRETCH_TICK),
            v(a, STRETCH_Y),
            v(b, STRETCH_Y),
            v(b, STRETCH_Y + STRETCH_TICK),
        ])
        .style(line_style(Palette::ink(), 2.5))
        .build(),
    )
}

/// The verdict box: its words and its outline, apart, so each can be
/// dimmed by what it paints with; detached.
fn verdict_box(kit: &Kit, stage: &mut Stage<'_>, text: &str) -> fmn::Result<[Mob; 2]> {
    let color = palette::verdict();
    let words = kit.text(stage, text, BOX_SIZE, color, BOX_AT)?;
    let w = stage.get_width(words) + 2.0 * BOX_PAD[0];
    let h = stage.get_height(words) + 2.0 * BOX_PAD[1];
    let outline = rect(
        v(BOX_AT[0] - w / 2.0, BOX_AT[1] - h / 2.0),
        w,
        h,
        line_style(color, BOX_W),
    )?;
    Ok([stage.arena_mut().add(outline), words])
}

/// Emphasis for texts: an outline just around each, stroked only, that
/// fades in, holds and fades out over `rt`.
fn halo(stage: &mut Stage<'_>, mobs: &[Mob], color: Srgb, rt: f64) -> fmn::Result<()> {
    let mut rings = Vec::with_capacity(mobs.len());
    for &m in mobs {
        let c = stage.get_center(m);
        let w = stage.get_width(m) + 2.0 * HALO_PAD;
        let h = stage.get_height(m) + 2.0 * HALO_PAD;
        let ring = rect(v(c[0] - w / 2.0, c[1] - h / 2.0), w, h, line_style(color, HALO_W))?;
        rings.push(stage.arena_mut().add(ring));
    }
    fade_in_all(stage, &rings, rt * 0.3)?;
    pause(stage, rt * 0.4)?;
    fade_out_all(stage, &rings, rt * 0.3)
}

/// Builders taking each of `mobs` to opacity `o` over `rt`.
fn all_to(mobs: &[Mob], o: f64, rt: f64) -> fmn::Result<Vec<AnimBuilder>> {
    mobs.iter().map(|&m| fade_to(m, o, rt)).collect()
}

/// A line grows from `edge` while texts fade in, together over `rt`.
fn grow_with(stage: &mut Stage<'_>, line: Mob, edge: Vec3, texts: &[Mob], rt: f64) -> fmn::Result<()> {
    let grow = fmn::animation::grow_from_edge(stage.arena_mut(), line, edge, None)?;
    let mut anims: Vec<Box<dyn Animation>> = vec![stage.prepare(grow.rt(rt))?];
    for &m in texts {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

impl SceneConstruct for BackToTheStop {
    fn name(&self) -> &str {
        "11_back_to_the_stop"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let f = facts::facts().map_err(fail)?;
        let s = stop(&f).map_err(fail)?;
        let gaps = timetable::gaps(OFFSET);
        let times = model::bus_times(OFFSET);
        let ready = model::passengers();
        let (_, long) = facts::short_and_long();
        let gold = palette::long_gap();

        // 11_1: chapter 01's picture as 01_3 left it: the strip with you at
        // the stop, the hour, the buses, the sign; the buses pulse; every
        // bus on time; the sign's average; all of it true
        let line = say(&kit, stage, "11_1")?;
        let parts = street(&kit, stage)?;
        let me = you(stage, at_the_stop())?;
        let mut members = parts;
        members.push(me);
        let strip = group(stage, &members)?;
        stage.scale(strip, STRIP_SCALE);
        stage.move_to(strip, STRIP_AT, ORIGIN);
        let table = timetable::timetable(&kit, stage, &f, OFFSET)?;
        let mut opening = vec![strip];
        opening.extend(table.mobs());
        fade_in_all(stage, &opening, 0.8)?;
        let mut all_buses: Vec<(i64, Mob)> = model::route(0).into_iter().zip(table.blue.iter().copied()).collect();
        all_buses.extend(model::route(OFFSET).into_iter().zip(table.red.iter().copied()));
        all_buses.sort_by_key(|&(t, _)| t);
        let in_order: Vec<Mob> = all_buses.iter().map(|&(_, m)| m).collect();
        line.wait_for(stage, "Ten buses an hour")?;
        flash_lagged(stage, &in_order, 1.4, 0.15)?;
        line.wait_for(stage, "all on time")?;
        let mut on_time: Vec<Mob> = all_buses
            .iter()
            .map(|&(t, _)| {
                let at = marker_top(t);
                stage.arena_mut().add(
                    Dot::new()
                        .point(v(at[0], at[1] + ON_TIME_DY))
                        .radius(PASSENGER_R)
                        .style(flat(palette::verdict(), 1.0))
                        .build(),
                )
            })
            .collect();
        on_time.push(kit.text(stage, ON_TIME, 20.0, palette::verdict(), ON_TIME_AT)?);
        fade_in_lagged(stage, &on_time, 0.8, 0.1)?;
        line.wait_for(stage, "a bus every six minutes on average")?;
        halo(stage, &[table.sign.every], Palette::ink(), HALO_RT)?;
        line.wait_for(stage, "All of that is true")?;
        halo(
            stage,
            &[table.sign.buses, table.sign.every],
            palette::verdict(),
            HALO_RT,
        )?;

        // 11_2: the tags over the buses; the sixty passengers; the long
        // gaps shade gold and each passenger takes its gap's color; the
        // tags go and the triangles rise over the whole hour
        let line = say(&kit, stage, "11_2")?;
        fade_out_all(stage, &on_time, 0.4)?;
        line.wait_for(stage, "you don't sample buses")?;
        let tags: Vec<Mob> = times
            .iter()
            .zip(&s.tags)
            .map(|(&t, value)| timetable::tag(&kit, stage, x_of(Q::int(t)), value))
            .collect::<fmn::Result<_>>()?;
        fade_in_lagged(stage, &tags, 1.0, 0.1)?;
        line.wait_for(stage, "You sample a moment")?;
        let dots: Vec<Mob> = ready
            .iter()
            .map(|&t| timetable::passenger(stage, t, Palette::ink()))
            .collect();
        fade_in_lagged(stage, &dots, 1.6, 0.05)?;
        line.wait_for(stage, "Five times in six")?;
        let shades: Vec<Mob> = gaps
            .iter()
            .filter(|g| g.minutes() == long)
            .map(|g| timetable::span(stage, g.from as f64, g.to as f64, gold))
            .collect();
        let paint = dots
            .iter()
            .zip(&ready)
            .map(|(&d, &t)| recolor(d, gap_color(model::gap_at(OFFSET, t)), 0.8))
            .collect::<fmn::Result<Vec<_>>>()?;
        fade_in_with(stage, &shades, paint, 0.8)?;
        line.wait_for(stage, "where the waits are long")?;
        let tris = hour_triangles(stage, &gaps);
        let mut leaving = tags;
        leaving.extend(&shades);
        fade_out_all(stage, &leaving, 0.4)?;
        grow_lagged(stage, &tris, GROW_RT, 0.1)?;

        // 11_3: the first stretch brightens, the rest dim; the triangles'
        // areas; the time they cover; the verdict box
        let line = say(&kit, stage, "11_3")?;
        let bright: Vec<usize> = (0..gaps.len()).filter(|&i| gaps[i].to <= PERIOD).collect();
        let dim: Vec<usize> = (0..gaps.len()).filter(|&i| gaps[i].to > PERIOD).collect();
        let pick = |idx: &[usize]| -> Vec<Mob> { idx.iter().map(|&i| tris[i]).collect() };
        line.wait_for(stage, "The area of the triangles")?;
        let same = kit.text(stage, &s.same_label(), SAME_SIZE, Palette::dim(), SAME_AT)?;
        let [short_area, long_area] = s.area_labels();
        let short_label = kit.text(stage, &short_area, AREA_SIZE, timetable::short_gap(), SHORT_AREA_AT)?;
        stage.move_to(short_label, SHORT_AREA_AT, RIGHT);
        let long_one = gaps[bright[1]];
        let centroid = over_hour(
            long_one.from as f64 + long_one.minutes() as f64 / 3.0,
            long_one.minutes() as f64 / 3.0,
        );
        let long_label = kit.text(stage, &long_area, AREA_SIZE, Palette::ink(), centroid)?;
        let mut light = all_to(&pick(&bright), TRI_BRIGHT, 0.6)?;
        light.extend(all_to(&pick(&dim), TRI_DIM, 0.6)?);
        fade_in_with(stage, &[same, short_label, long_label], light, 0.6)?;
        line.wait_for(stage, "divided by the time they cover")?;
        let bracket = stretch_bracket(stage, 0.0, PERIOD as f64);
        let bracket_label = kit.text(
            stage,
            &s.stretch_label(),
            STRETCH_SIZE,
            Palette::ink(),
            v(x_at(PERIOD as f64 / 2.0), STRETCH_LABEL_Y),
        )?;
        grow_with(stage, bracket, LEFT, &[bracket_label], 0.6)?;
        line.wait_for(stage, "Fifty-two square minutes")?;
        let [outline, verdict] = verdict_box(&kit, stage, &s.verdict())?;
        fade_in_all(stage, &[outline, verdict], 0.5)?;
        line.wait_for(stage, "Four minutes and twenty seconds")?;
        halo(stage, &[verdict], palette::verdict(), HALO_RT)?;
        let verdict_up = now(stage);
        kit.hold(stage)?;
        wait_until(stage, verdict_up + read_s(&s.verdict()))?;

        // 11_4: the stretches back to one brightness; you leave the stop
        // for a minute before the red bus at :26, a line up to your moment;
        // the question; think time
        let line = say(&kit, stage, "11_4")?;
        let even = all_to(&tris, TRI_FILL, 0.6)?;
        fade_out_with(
            stage,
            &[same, short_label, long_label, bracket, bracket_label],
            even,
            0.6,
        )?;
        line.wait_for(stage, "turn up one minute before your bus")?;
        let your_x = x_of(your_moment());
        let your_at = v(your_x, YOU_Y);
        stage.play(shrink_to(me, 1.0 / STRIP_SCALE, your_at, YOU_RT)?)?;
        let guide = stage.arena_mut().add(
            Line::new(v(your_x, YOU_Y + 0.2), v(your_x, LINE_Y))
                .style(line_style(palette::you(), GUIDE_W))
                .build()?,
        );
        let your_marker: Vec<Mob> = all_buses
            .iter()
            .filter(|&&(t, _)| t == your_bus())
            .map(|&(_, m)| m)
            .collect();
        fade_in_all(stage, &[guide], 0.4)?;
        flash_all(stage, &your_marker, 0.6)?;
        line.wait_for(stage, "How long do you wait")?;
        let ask = kit.text_left(stage, ASK, ASK_SIZE, Palette::ink(), v(your_x + ASK_DX, YOU_Y))?;
        fade_in_all(stage, &[ask], 0.4)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 11_5: one minute, the triangle's height at your moment; the
        // triangles and the box dim
        let line = say(&kit, stage, "11_5")?;
        fade_out_all(stage, &[q, ask], 0.3)?;
        let wait_w = model::wait_at(OFFSET, your_moment()).to_f64();
        let segment = stage.arena_mut().add(
            Line::new(
                over_hour(your_moment().to_f64(), 0.0),
                over_hour(your_moment().to_f64(), wait_w),
            )
            .style(line_style(palette::verdict(), SEGMENT_W))
            .build()?,
        );
        stage.set_z_index(segment, YOU_Z, true);
        let answer = kit.text_left(
            stage,
            &s.your_label(),
            ASK_SIZE,
            palette::verdict(),
            v(your_x + ASK_DX, YOU_Y),
        )?;
        grow_with(stage, segment, DOWN, &[answer], 0.6)?;
        line.wait_for(stage, "the triangles' average doesn't describe you")?;
        let mut dimming = all_to(&tris, TRI_DIM, 0.8)?;
        dimming.push(stroke_to(outline, BOX_DIM, 0.8)?);
        dimming.push(fade_to(verdict, BOX_DIM, 0.8)?);
        fade_in_with(stage, &[], dimming, 0.8)?;

        // 11_6: you, the sign, the strip and the box go; the panel; the
        // passengers who cannot make the next bus with a minute to spare;
        // the slide; the stacks before the blue buses, then the red
        let line = say(&kit, stage, "11_6")?;
        // you go with the strip, your family
        let mut gone = vec![guide, segment, answer, strip, outline, verdict];
        gone.extend(table.sign.mobs());
        fade_out_all(stage, &gone, 0.6)?;
        let share_name = kit.text(stage, SHARE_NAME, PANEL_SIZE, Palette::ink(), v(NAME_X, ROW_YS[0]))?;
        stage.move_to(share_name, v(NAME_X, ROW_YS[0]), RIGHT);
        let wait_name = kit.text(stage, WAIT_NAME, PANEL_SIZE, Palette::ink(), v(NAME_X, ROW_YS[1]))?;
        stage.move_to(wait_name, v(NAME_X, ROW_YS[1]), RIGHT);
        let share_frame = stage.arena_mut().add(rect(
            v(BAR_X0, ROW_YS[0] - BAR_H / 2.0),
            SHARE_W,
            BAR_H,
            line_style(Palette::dim(), 2.0),
        )?);
        let share_fill = stage.arena_mut().add(rect(
            v(BAR_X0, ROW_YS[0] - BAR_H / 2.0),
            SHARE_W * s.long_share.to_f64(),
            BAR_H,
            flat(gold, 0.85),
        )?);
        let share_value = kit.text_left(stage, &s.share, PANEL_SIZE, gold, v(VALUE_X, ROW_YS[0]))?;
        let random_value = kit.text_left(stage, &s.wait_s, PANEL_SIZE, Palette::ink(), v(VALUE_X, ROW_YS[1]))?;
        fade_in_all(
            stage,
            &[
                share_name,
                wait_name,
                share_frame,
                share_fill,
                share_value,
                random_value,
            ],
            0.5,
        )?;
        let wait_now = stage.add_value_tracker(0.0);
        let wait_bar = live_bar(stage, wait_now, v(BAR_X0, ROW_YS[1]), WAIT_UNIT, BAR_H, Palette::ink())?;
        stage.play(tween(wait_now, s.random_wait.to_f64(), 0.6)?)?;
        line.wait_for(stage, "with a minute to spare")?;
        let late: Vec<Mob> = too_late(&ready).into_iter().map(|i| dots[i]).collect();
        flash_lagged(stage, &late, 1.0, 0.1)?;
        line.wait_for(stage, "and stand at the stop")?;
        let go = stage.add_value_tracker(0.0);
        let slider = slide(stage, &dots, &ready, &s, go)?;
        stage.play(tween(go, 1.0, SLIDE_RT)?)?;
        stage.clear_updaters(slider, true);
        let before = |route: i64| -> Vec<Mob> {
            let spots: Vec<Q> = model::route(route)
                .into_iter()
                .map(|b| in_hour(Q::int(b - EARLY)))
                .collect();
            (0..dots.len())
                .filter(|&i| spots.contains(&in_hour(s.planned[i])))
                .map(|i| dots[i])
                .collect()
        };
        line.wait_for(stage, "Ten would wait for each blue bus")?;
        let mut blue_stacks = before(0);
        blue_stacks.extend(&table.blue);
        flash_all(stage, &blue_stacks, 0.8)?;
        line.wait_for(stage, "two for each red")?;
        let mut red_stacks = before(OFFSET);
        red_stacks.extend(&table.red);
        flash_all(stage, &red_stacks, 0.8)?;

        // 11_7: the share stays at five sixths; the average wait falls to a
        // minute, and each stack's wait; the last minute of every gap; the
        // gold triangles dim further; K09, held until read
        let line = say(&kit, stage, "11_7")?;
        halo(stage, &[share_value], gold, HALO_RT)?;
        line.wait_for(stage, "every wait would be one minute")?;
        let planned_value = kit.text_left(
            stage,
            &s.planned_wait_s,
            PANEL_SIZE,
            palette::verdict(),
            v(VALUE_X, ROW_YS[1]),
        )?;
        fade_out_all(stage, &[random_value], 0.3)?;
        let mut stack_labels = Vec::new();
        for (spot, size) in stacks(&s.planned) {
            let y = DOT_Y - (size - 1) as f64 * STACK_DY - STACK_LABEL_DY;
            stack_labels.push(kit.text(
                stage,
                &s.stack_label(),
                STACK_LABEL_SIZE,
                Palette::ink(),
                v(x_at(spot.to_f64()), y),
            )?);
        }
        // the new wait and every stack's come in while the bar falls
        let mut wait_words = vec![planned_value];
        wait_words.extend(&stack_labels);
        let fall = steady(wait_now, s.planned_mean.to_f64(), WAIT_RT)?;
        fade_in_with(stage, &wait_words, vec![fall], WAIT_RT)?;
        // the bar stands still from here, so the chapter's fade can take it
        stage.clear_updaters(wait_bar, true);
        line.wait_for(stage, "They'd all stand in the last minute of a gap")?;
        let last_minutes: Vec<Mob> = gaps
            .iter()
            .map(|g| timetable::span(stage, (g.to - EARLY) as f64, g.to as f64, gap_color(g.minutes())))
            .collect();
        fade_in_all(stage, &last_minutes, 0.6)?;
        line.wait_for(stage, "the triangles' average wouldn't describe them")?;
        let gold_tris: Vec<Mob> = (0..gaps.len())
            .filter(|&i| gaps[i].minutes() == long)
            .map(|i| tris[i])
            .collect();
        fade_in_with(stage, &[], all_to(&gold_tris, TRI_GONE, 0.8)?, 0.8)?;
        line.wait_for(stage, "Change how passengers arrive")?;
        let k09_text = s.k09();
        let k09 = precisely(&kit, stage, &k09_text)?;
        fade_in_all(stage, &[k09], 0.5)?;
        let k09_up = now(stage);
        kit.hold(stage)?;
        wait_until(stage, k09_up + read_s(&k09_text).max(STAND_UP))?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use explainer_kit::{Narrator, Script};

    use super::super::draw::PAUSE_NOTE;
    use super::*;

    /// The full draft with its line breaks and indents as single spaces.
    fn draft() -> String {
        crate::DRAFT.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    fn the_stop() -> Stop {
        stop(&facts::facts().unwrap()).unwrap()
    }

    /// A fact's printed fraction, as a number.
    fn fact_q(key: &str) -> Q {
        let f = facts::facts().unwrap();
        parse_q(key, f.get(key).unwrap()).unwrap()
    }

    /// The facts with one key's printed value replaced.
    fn with(key: &str, bad: &str) -> Facts {
        let f = facts::facts().unwrap();
        let mut wrong = Facts::new();
        for row in f.print().lines() {
            let (k, value) = row.split_once('\t').unwrap();
            wrong.add(k, if k == key { bad } else { value }).unwrap();
        }
        wrong
    }

    /// Needs chapter 11's lines in script.tsv (the integration step adds
    /// them from the storyboard). The whole chapter runs headless at 10 fps
    /// with the scratch voice: every phrase it waits for is in its line,
    /// every fact it reads exists and matches the timetable and the
    /// planners, and its seven lines are said in order. It ends with
    /// `kit.finish`, whose timeline runs past the end of the last line.
    #[test]
    fn the_chapter_runs_and_says_its_lines_in_order() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let narrator = Narrator::estimate(&script);
        let kit = Kit::new().unwrap().with_narrator(Rc::clone(&narrator));
        let mut scene = BackToTheStop { kit };
        let config = fmn::scene::RuntimeConfig {
            fps: 10,
            ..fmn::scene::RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut fmn::scene::NullSceneSink).unwrap();
        let said_at = narrator.said();
        let said: Vec<&str> = said_at.iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(said, ["11_1", "11_2", "11_3", "11_4", "11_5", "11_6", "11_7"]);
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

    /// The street is chapter 01's: `street` and every constant it and the
    /// strip use are copied word for word from `the_stop.rs`, where 01_1
    /// walks you to [`at_the_stop`] and 01_3 shrinks the strip to
    /// [`STRIP_SCALE`] at [`STRIP_AT`].
    #[test]
    fn the_street_is_chapter_01s() {
        let ours = include_str!("back_to_the_stop.rs");
        let theirs = include_str!("the_stop.rs");
        let body = |src: &'static str| -> &'static str {
            let from = src.find("\nfn street(").expect("a street function");
            let to = from + src[from..].find("\n}\n").expect("its end");
            &src[from..to]
        };
        assert_eq!(body(ours), body(theirs));
        for name in [
            "STREET_Y",
            "HOME_X",
            "STOP_X",
            "STATION_X",
            "GROUND_HALF",
            "STREET_LABEL_DY",
            "STREET_LABEL_SIZE",
            "STRIP_SCALE",
            "STRIP_AT",
            "HOME",
            "STOP",
            "STATION",
            "ON_TIME",
            "ON_TIME_DY",
            "ON_TIME_AT",
        ] {
            let line = |src: &'static str| -> Vec<&'static str> {
                src.lines()
                    .filter(|l| l.starts_with(&format!("const {name}:")))
                    .collect()
            };
            assert_eq!(line(ours).len(), 1, "{name}");
            assert_eq!(line(ours), line(theirs), "{name}");
        }
        assert!(theirs.contains("fly(me, v(STOP_X - 0.35, STREET_Y - 0.25)"));
        assert_eq!(at_the_stop(), v(STOP_X - 0.35, STREET_Y - 0.25));
        assert!(theirs.contains("shrink_to(strip, STRIP_SCALE, STRIP_AT"));
    }

    /// Every number the chapter prints is the model's, and a fact that
    /// stops matching the timetable, the triangles, you or the planners is
    /// refused: the negative controls.
    #[test]
    fn the_numbers_are_the_models_and_a_wrong_fact_is_refused() {
        let s = the_stop();
        assert_eq!(s.area_labels(), ["2 min²".to_owned(), "50 min²".to_owned()]);
        assert_eq!(s.stretch_label(), "12 min");
        assert_eq!(s.same_label(), "the same × 5");
        assert_eq!(s.verdict(), "average wait: 52 min² ÷ 12 min = 4 min 20 s");
        assert_eq!(s.your_label(), "wait: 1 min");
        assert_eq!(s.stack_label(), "wait 1 min");
        assert_eq!(s.share, "⅚");
        assert_eq!(s.planned_wait_s, "1 min");
        assert_eq!(s.random_wait, model::mean_wait(&model::hour_gaps(OFFSET)));
        assert_eq!(s.random_wait, Q::new(13, 3));
        assert_eq!(s.planned_mean, Q::int(EARLY));
        assert_eq!(
            s.tags,
            model::gap_before_each_bus(OFFSET)
                .iter()
                .map(i64::to_string)
                .collect::<Vec<_>>()
        );
        for (key, bad) in [
            ("F-K2.per_blue", "6"),
            ("F-K2.per_red", "6"),
            ("F-K2.in_long_share", "1/2"),
            ("F-K2.waits", "13/3"),
            ("F-K2.never_early", "false"),
            ("F-K2.stand_at", "0,10"),
            ("F-K2.wait_share_of_gap", "1/2"),
            ("F-K1.wait", "13/3"),
            ("F-A1.total_area", "50"),
            ("F-W1.mean_wait_clock", "6:00"),
        ] {
            let refused = stop(&with(key, bad)).err().expect("a wrong fact must be refused");
            assert!(refused.contains(key), "{key}: {refused}");
        }
        // random moments with as little of their gap ahead as the planners
        let refused = stop(&with("F-S1.wait_share_of_gap", "1/6")).err().unwrap();
        assert!(refused.contains("as much of their gap ahead"), "{refused}");
    }

    /// The timetable reader waits [`EARLY`] before any bus of the hour, the
    /// triangle's own height at that moment; the triangles' average, four
    /// minutes twenty, is not that wait. (Where you stand is checked with
    /// the picture.)
    #[test]
    fn the_timetable_reader_waits_one_minute() {
        let s = the_stop();
        let early = Q::int(EARLY);
        assert!(model::route(OFFSET).contains(&your_bus()), "your bus is red");
        for b in model::bus_times(OFFSET) {
            assert_eq!(model::wait_at(OFFSET, Q::int(b) - early), early, "the bus at {b}");
        }
        let t = your_moment();
        let gaps = timetable::gaps(OFFSET);
        let g = gaps.iter().find(|g| g.holds(t)).unwrap();
        // the triangle's corners: its first bus, the apex over it, its next
        let [(a, _), (_, top), (b, _)] = triangles::triangle(g);
        let height = top * (b - t.to_f64()) / (b - a);
        assert!((height - model::wait_at(OFFSET, t).to_f64()).abs() < 1e-12);
        assert_eq!(model::wait_at(OFFSET, t), fact_q("F-K1.wait"));
        assert_ne!(model::wait_at(OFFSET, t), s.random_wait);
    }

    /// The planners, by brute force over every bus from an hour before to
    /// two after: each stands [`EARLY`] before the first bus at or after
    /// its dot plus [`EARLY`], never before it is ready, in the last minute
    /// of its gap. The two dots of each stretch ready less than a minute
    /// before a bus slide on past it. So the stacks are as F-K2 says, every
    /// planner ends in the color of the gap it stands in, and in each stack
    /// the dot with least far to slide is on top.
    #[test]
    fn the_planners_stand_a_minute_before_the_first_bus_they_can_make() {
        let s = the_stop();
        let f = facts::facts().unwrap();
        let ready = model::passengers();
        let early = Q::int(EARLY);
        let mut buses: Vec<Q> = (-1..=2)
            .flat_map(|h| model::bus_times(OFFSET).into_iter().map(move |b| Q::int(b + h * HOUR)))
            .collect();
        buses.sort();
        for (i, &r) in ready.iter().enumerate() {
            let first = buses.iter().copied().find(|&b| b >= r + early).unwrap();
            assert_eq!(s.planned[i], first - early, "passenger {i}");
            assert!(s.planned[i] >= r, "passenger {i} turns up before it is ready");
            assert_eq!(model::wait_at(OFFSET, s.planned[i]), early, "passenger {i}");
        }
        assert_eq!(in_hour(Q::int(HOUR + 1)), Q::int(1));
        assert_eq!(in_hour(Q::int(-1)), Q::int(HOUR - 1));
        assert_eq!(in_hour(*ready.last().unwrap()), *ready.last().unwrap());
        let late = too_late(&ready);
        assert_eq!(late.len(), model::bus_times(OFFSET).len(), "one for each bus");
        for (i, &r) in ready.iter().enumerate() {
            let next = r + model::wait_at(OFFSET, r);
            if late.contains(&i) {
                assert!(s.planned[i] + early > next, "passenger {i} slides past its next bus");
            } else {
                assert_eq!(s.planned[i] + early, next, "passenger {i} takes its next bus");
            }
        }
        let per = |key: &str| f.get(key).unwrap().parse::<usize>().unwrap();
        let (short, long) = facts::short_and_long();
        let gaps = timetable::gaps(OFFSET);
        let all = stacks(&s.planned);
        assert_eq!(all.len(), model::bus_times(OFFSET).len());
        assert_eq!(all.iter().map(|&(_, k)| k).sum::<usize>(), ready.len());
        for &(spot, k) in &all {
            let bus = in_hour(spot + early);
            let blue = model::route(0).iter().any(|&b| Q::int(b) == bus);
            assert_eq!(k, per(if blue { "F-K2.per_blue" } else { "F-K2.per_red" }), "at {spot}");
            let members: Vec<usize> = (0..ready.len()).filter(|&i| in_hour(s.planned[i]) == spot).collect();
            for &i in &members {
                let gap = model::gap_at(OFFSET, s.planned[i]);
                assert_eq!(gap, if blue { long } else { short }, "passenger {i}");
                assert_eq!(minutes_at(&gaps, in_hour(s.planned[i]).to_f64()), gap, "passenger {i}");
            }
            let mut by_slide: Vec<(Q, usize)> =
                members.iter().map(|&i| (s.planned[i] - ready[i], s.ranks[i])).collect();
            by_slide.sort();
            let order: Vec<usize> = by_slide.iter().map(|&(_, rank)| rank).collect();
            assert_eq!(order, (0..k).collect::<Vec<_>>(), "the stack at {spot}");
        }
    }

    /// The same share of long gaps can come with different waits. Random
    /// moments and planners both stand five times in six in a long gap,
    /// the long gaps' share of the time; the dots' own average wait is the
    /// triangles' four minutes twenty, the planners' one minute, because a
    /// random moment has half its gap ahead and a planner a sixth. The
    /// control the film leaves out, six planners a minute before every
    /// bus (some passing up a bus they could make), waits a minute too but
    /// stands only half the time in a long gap: the share alone never gave
    /// the wait.
    #[test]
    fn the_same_share_of_long_gaps_can_wait_differently() {
        let s = the_stop();
        let ready = model::passengers();
        let (_, long) = facts::short_and_long();
        let share = |at: &[Q]| {
            let in_long = at.iter().filter(|&&t| model::gap_at(OFFSET, t) == long).count();
            Q::new(in_long as i64, at.len() as i64)
        };
        let mean_wait = |at: &[Q]| at.iter().map(|&t| model::wait_at(OFFSET, t)).sum::<Q>() / Q::int(at.len() as i64);
        let ahead = |at: &[Q]| {
            at.iter()
                .map(|&t| model::wait_at(OFFSET, t) / Q::int(model::gap_at(OFFSET, t)))
                .sum::<Q>()
                / Q::int(at.len() as i64)
        };
        let time_share = model::time_share(&model::hour_gaps(OFFSET), |g| g == long);
        assert_eq!(share(&ready), time_share);
        assert_eq!(share(&s.planned), time_share);
        assert_eq!(s.long_share, time_share);
        assert_eq!(mean_wait(&ready), s.random_wait);
        assert_eq!(mean_wait(&s.planned), s.planned_mean);
        assert_ne!(s.random_wait, s.planned_mean);
        assert_eq!(ahead(&ready), fact_q("F-S1.wait_share_of_gap"));
        assert_eq!(ahead(&s.planned), fact_q("F-K2.wait_share_of_gap"));
        let buses = model::bus_times(OFFSET);
        let per_bus = ready.len() / buses.len();
        let shared: Vec<Q> = buses
            .iter()
            .flat_map(|&b| std::iter::repeat_n(Q::int(b - EARLY), per_bus))
            .collect();
        assert_eq!(shared.len(), ready.len());
        assert_eq!(mean_wait(&shared), Q::int(EARLY));
        // the blue buses close the long gaps
        assert_eq!(share(&shared), Q::new(model::route(0).len() as i64, buses.len() as i64));
        assert_ne!(share(&shared), time_share);
    }

    /// The slide moves every dot forward, never back (the dot at :59½ runs
    /// on past :60 and comes in again at :00, as the timetable repeats),
    /// and only down, from the passengers' row to its place in its stack.
    /// It starts where 11_2 left the dots and ends where the planners
    /// stand.
    #[test]
    fn the_slide_never_goes_back() {
        let s = the_stop();
        let ready = model::passengers();
        let steps = 200;
        for (i, &r) in ready.iter().enumerate() {
            let (p, k) = (s.planned[i], s.ranks[i]);
            assert_eq!(slide_at(r, p, k, 0.0), (r.to_f64(), DOT_Y), "passenger {i}");
            let (t, y) = slide_at(r, p, k, 1.0);
            assert!((t - in_hour(p).to_f64()).abs() < 1e-9, "passenger {i}");
            assert!((y - (DOT_Y - k as f64 * STACK_DY)).abs() < 1e-9, "passenger {i}");
            let mut was = slide_at(r, p, k, 0.0);
            for j in 1..=steps {
                let at = slide_at(r, p, k, j as f64 / steps as f64);
                let forward = (at.0 - was.0).rem_euclid(HOUR as f64);
                assert!(forward < 0.1, "passenger {i} moves back at step {j}");
                assert!(at.1 <= was.1 + 1e-12, "passenger {i} moves up at step {j}");
                was = at;
            }
        }
    }

    /// The triangles over the hour are the model's, to scale: a minute up
    /// as long as a minute across, so each apex stands as high as its base
    /// is wide, and each area is the model's area in square minutes.
    #[test]
    fn the_triangles_are_the_models_to_scale() {
        let unit = hour_unit();
        assert!((x_at(1.0) - x_at(0.0) - unit).abs() < 1e-12);
        let (_, long) = facts::short_and_long();
        for g in timetable::gaps(OFFSET) {
            let [p, q, r] = triangles::triangle(&g).map(|(t, w)| over_hour(t, w));
            assert!((p[1] - LINE_Y).abs() < 1e-12 && (r[1] - LINE_Y).abs() < 1e-12);
            assert!(
                ((q[1] - p[1]) - (r[0] - p[0])).abs() < 1e-12,
                "apex as high as the base is wide"
            );
            let area = (r[0] - p[0]) * (q[1] - p[1]) / 2.0 / (unit * unit);
            assert!((area - model::triangle_area(g.minutes()).to_f64()).abs() < 1e-9);
        }
        assert!((over_hour(0.0, long as f64)[1] - (LINE_Y + long as f64 * unit)).abs() < 1e-12);
    }

    /// What the chapter prints is the storyboard's picture, word for word.
    #[test]
    fn the_printed_words_are_the_drafts() {
        let s = the_stop();
        let board = draft();
        let [short_area, long_area] = s.area_labels();
        for shown in [
            ON_TIME.to_owned(),
            short_area,
            long_area,
            s.stretch_label(),
            s.same_label(),
            s.verdict(),
            ASK.to_owned(),
            s.your_label(),
            SHARE_NAME.to_owned(),
            WAIT_NAME.to_owned(),
            s.share.clone(),
            s.wait_s.clone(),
            s.planned_wait_s.clone(),
            s.stack_label(),
            s.k09(),
        ] {
            assert!(board.contains(&format!("\"{shown}\"")), "not in the draft: {shown}");
        }
    }

    /// The fixed words carry no digits: every number on screen is read
    /// from the facts or counted from what is drawn (K09's 2 is the halves
    /// of F-S1.wait_share_of_gap).
    #[test]
    fn the_fixed_words_carry_no_numbers() {
        for w in [
            HOME,
            STOP,
            STATION,
            ON_TIME,
            AREA,
            MIN,
            TIMES,
            DIVIDED,
            THE_SAME,
            AVERAGE_WAIT,
            ASK,
            WAIT,
            SHARE_NAME,
            WAIT_NAME,
            K09_HEAD,
            K09_TAIL,
        ] {
            assert!(!w.chars().any(|c| c.is_ascii_digit()), "{w}");
        }
        assert_eq!(the_stop().halves, "2");
    }

    /// The texts up together stay apart and on screen, and nothing printed
    /// over the line reaches into the triangles. Widths are estimated, not
    /// measured: 0.0045 units a point a character, a line 0.6 of its size
    /// tall. The strip is chapter 01's street, from its labels' foot to its
    /// plate's top, halved about its centre at [`STRIP_AT`].
    #[test]
    fn the_picture_stays_apart_and_on_screen() {
        let s = the_stop();
        type Box4 = (f64, f64, f64, f64);
        let w = |t: &str, pt: f64| t.chars().count() as f64 * pt * 0.0045;
        let h = |pt: f64| pt * 0.0045 / 0.55 * 0.6;
        let centred = |t: &str, pt: f64, x: f64, y: f64| -> Box4 {
            (x - w(t, pt) / 2.0, x + w(t, pt) / 2.0, y - h(pt), y + h(pt))
        };
        let left = |t: &str, pt: f64, x: f64, y: f64| -> Box4 { (x, x + w(t, pt), y - h(pt), y + h(pt)) };
        let right = |t: &str, pt: f64, x: f64, y: f64| -> Box4 { (x - w(t, pt), x, y - h(pt), y + h(pt)) };
        let dot = |x: f64, y: f64, r: f64| -> Box4 { (x - r, x + r, y - r, y + r) };
        let apart = |a: Box4, b: Box4| a.1 < b.0 || b.1 < a.0 || a.3 < b.2 || b.3 < a.2;
        let on_screen = |a: Box4| a.0 > -7.1 && a.1 < 7.1 && a.2 > -4.0 && a.3 < 4.0;
        // private elsewhere: draw's clock labels (LABEL_DY, LABEL_SIZE),
        // markers' half width and your ring; timetable's tags (TAG_SIZE)
        // and sign (SIGN_AT, SIGN_W, SIGN_H)
        let (clock_dy, clock_size, marker_half, ring_r, tag_size) = (0.32, 18.0, 0.07, 0.16, 18.0);
        let sign: Box4 = (-4.3 - 1.85, -4.3 + 1.85, -1.55 - 0.7, -1.55 + 0.7);
        // the strip
        let (sy0, sy1) = (
            STREET_Y + STREET_LABEL_DY - h(STREET_LABEL_SIZE),
            STREET_Y + 0.85 + 0.36,
        );
        let (cx, cy) = (0.0, (sy0 + sy1) / 2.0);
        let k = STRIP_SCALE;
        let scaled = |x: f64, y: f64| (STRIP_AT[0] + k * (x - cx), STRIP_AT[1] + k * (y - cy));
        let strip: Box4 = (
            scaled(-GROUND_HALF, sy0).0,
            scaled(GROUND_HALF, sy1).0,
            scaled(0.0, sy0).1,
            scaled(0.0, sy1).1,
        );
        let ground_y = scaled(0.0, STREET_Y - 0.4).1;
        let ground: Box4 = (strip.0, strip.1, ground_y, ground_y);
        let street_labels: Vec<Box4> = [(HOME, HOME_X), (STOP, STOP_X), (STATION, STATION_X)]
            .iter()
            .map(|&(word, x)| {
                let (lx, ly) = scaled(x, STREET_Y + STREET_LABEL_DY);
                let (hw, hh) = (k * w(word, STREET_LABEL_SIZE) / 2.0, k * h(STREET_LABEL_SIZE));
                (lx - hw, lx + hw, ly - hh, ly + hh)
            })
            .collect();
        let (ux, uy) = scaled(at_the_stop()[0], at_the_stop()[1]);
        let you_in_strip = dot(ux, uy, k * ring_r);
        // the hour, its buses, the passengers
        let markers: Box4 = (
            LINE_X0 - marker_half,
            LINE_X1 + marker_half,
            LINE_Y - 0.01,
            marker_top(0)[1],
        );
        let clock_y = LINE_Y - clock_dy;
        let clocks: Box4 = (
            LINE_X0 - w(":00", clock_size) / 2.0,
            LINE_X1 + w(":60", clock_size) / 2.0,
            clock_y - h(clock_size),
            clock_y + h(clock_size),
        );
        let ready = model::passengers();
        let r = PASSENGER_R;
        let row: Box4 = (
            x_of(ready[0]) - r,
            x_of(*ready.last().unwrap()) + r,
            DOT_Y - r,
            DOT_Y + r,
        );
        let times = model::bus_times(OFFSET);
        let (first, last) = (Q::int(times[0]), Q::int(*times.last().unwrap()));
        let tag_y = timetable::tag_at(0.0)[1];
        let tags_row: Box4 = (
            x_of(first) - w("10", tag_size) / 2.0,
            x_of(last) + w("10", tag_size) / 2.0,
            tag_y - h(tag_size),
            tag_y + h(tag_size),
        );
        let on_time_y = marker_top(0)[1] + ON_TIME_DY;
        let on_time_row: Box4 = (x_of(first) - r, x_of(last) + r, on_time_y - r, on_time_y + r);
        let on_time_words = centred(ON_TIME, 20.0, ON_TIME_AT[0], ON_TIME_AT[1]);
        // 11_3
        let [short_area, long_area] = s.area_labels();
        let same = centred(&s.same_label(), SAME_SIZE, SAME_AT[0], SAME_AT[1]);
        let short_label = right(&short_area, AREA_SIZE, SHORT_AREA_AT[0], SHORT_AREA_AT[1]);
        let stretch: Vec<Gap> = timetable::gaps(OFFSET).into_iter().filter(|g| g.to <= PERIOD).collect();
        let long_one = stretch[1];
        let centroid = over_hour(
            long_one.from as f64 + long_one.minutes() as f64 / 3.0,
            long_one.minutes() as f64 / 3.0,
        );
        let long_label = centred(&long_area, AREA_SIZE, centroid[0], centroid[1]);
        let bracket: Box4 = (
            x_at(0.0) + STRETCH_INSET,
            x_at(PERIOD as f64) - STRETCH_INSET,
            STRETCH_Y,
            STRETCH_Y + STRETCH_TICK,
        );
        let bracket_label = centred(
            &s.stretch_label(),
            STRETCH_SIZE,
            x_at(PERIOD as f64 / 2.0),
            STRETCH_LABEL_Y,
        );
        let words = centred(&s.verdict(), BOX_SIZE, BOX_AT[0], BOX_AT[1]);
        let verdict: Box4 = (
            words.0 - BOX_PAD[0],
            words.1 + BOX_PAD[0],
            words.2 - BOX_PAD[1],
            words.3 + BOX_PAD[1],
        );
        // 11_4, 11_5
        let your_x = x_of(your_moment());
        let you_out = dot(your_x, YOU_Y, ring_r);
        let guide: Box4 = (your_x - 0.01, your_x + 0.01, YOU_Y + 0.2, LINE_Y);
        let ask = left(ASK, ASK_SIZE, your_x + ASK_DX, YOU_Y);
        let answer = left(&s.your_label(), ASK_SIZE, your_x + ASK_DX, YOU_Y);
        let ring: Box4 = (
            THINK_AT[0] - 0.62,
            THINK_AT[0] + 0.62,
            THINK_AT[1] - 0.62,
            THINK_AT[1] + 0.62,
        );
        let note = centred(PAUSE_NOTE, 18.0, THINK_AT[0], THINK_AT[1] - 0.62 - 0.3);
        // 11_6, 11_7: the panel, the stacks, K09
        let share_name = right(SHARE_NAME, PANEL_SIZE, NAME_X, ROW_YS[0]);
        let wait_name = right(WAIT_NAME, PANEL_SIZE, NAME_X, ROW_YS[1]);
        let share_bar: Box4 = (
            BAR_X0,
            BAR_X0 + SHARE_W,
            ROW_YS[0] - BAR_H / 2.0,
            ROW_YS[0] + BAR_H / 2.0,
        );
        let wait_bar: Box4 = (
            BAR_X0,
            BAR_X0 + WAIT_UNIT * s.random_wait.to_f64(),
            ROW_YS[1] - BAR_H / 2.0,
            ROW_YS[1] + BAR_H / 2.0,
        );
        let share_value = left(&s.share, PANEL_SIZE, VALUE_X, ROW_YS[0]);
        let random_value = left(&s.wait_s, PANEL_SIZE, VALUE_X, ROW_YS[1]);
        let planned_value = left(&s.planned_wait_s, PANEL_SIZE, VALUE_X, ROW_YS[1]);
        assert!(STACK_DY > 2.0 * r, "stacked dots overlap");
        let mut columns = Vec::new();
        let mut stack_labels = Vec::new();
        for (spot, size) in stacks(&s.planned) {
            let (x, foot) = (x_at(spot.to_f64()), DOT_Y - (size - 1) as f64 * STACK_DY);
            columns.push((x - r, x + r, foot - r, DOT_Y + r));
            stack_labels.push(centred(&s.stack_label(), STACK_LABEL_SIZE, x, foot - STACK_LABEL_DY));
        }
        // K09's band (`props::lower_third`): its foot 0.2 off the frame's,
        // 0.3 + 0.36 a line tall, wrapped word by word at 92 characters
        let mut lines: Vec<String> = Vec::new();
        for word in s.k09().split_whitespace() {
            match lines.last_mut() {
                Some(l) if l.chars().count() + 1 + word.chars().count() <= 92 => {
                    l.push(' ');
                    l.push_str(word);
                }
                _ => lines.push(word.to_owned()),
            }
        }
        assert!(lines.len() <= 3, "K09 is cut short");
        let k09: Box4 = (-6.71, 6.71, -3.8, -3.8 + 0.3 + 0.36 * lines.len() as f64);
        let strip_parts = [strip, markers, clocks, sign];
        let panel = [share_name, wait_name, share_bar, wait_bar, share_value];
        let moments: Vec<Vec<Box4>> = vec![
            // 11_1: the opening picture, every bus on time
            [strip_parts.to_vec(), vec![on_time_row, on_time_words]].concat(),
            // 11_2: the tags and the passengers
            [strip_parts.to_vec(), vec![tags_row, row]].concat(),
            // 11_3: the stretch, its areas, its time and the verdict
            [
                strip_parts.to_vec(),
                vec![row, same, short_label, long_label, bracket, bracket_label, verdict],
            ]
            .concat(),
            // 11_4: you, the question and its ring
            [strip_parts.to_vec(), vec![row, verdict, you_out, ask, ring, note]].concat(),
            // 11_5: your wait
            [strip_parts.to_vec(), vec![row, verdict, you_out, answer]].concat(),
            // 11_6: the panel and the stacks
            [vec![markers, clocks, random_value], panel.to_vec(), columns.clone()].concat(),
            // 11_7: each stack's wait, K09
            [
                vec![markers, clocks, planned_value, k09],
                panel.to_vec(),
                columns.clone(),
                stack_labels,
            ]
            .concat(),
        ];
        for (m, up) in moments.iter().enumerate() {
            for (i, &a) in up.iter().enumerate() {
                assert!(on_screen(a), "moment {m}: item {i} off screen: {a:?}");
                for (j, &b) in up.iter().enumerate().skip(i + 1) {
                    assert!(apart(a, b), "moment {m}: items {i} and {j} overlap: {a:?} {b:?}");
                }
            }
        }
        // the highest a triangle reaches over [x0, x1]: its slant falls a
        // unit for a unit across
        let tri_top = |x0: f64, x1: f64| -> f64 {
            timetable::gaps(OFFSET)
                .iter()
                .filter(|g| x_at(g.from as f64) <= x1 && x0 <= x_at(g.to as f64))
                .map(|g| LINE_Y + x_at(g.to as f64) - x0.max(x_at(g.from as f64)))
                .fold(LINE_Y, f64::max)
        };
        let over_the_line = [
            ground,
            street_labels[0],
            street_labels[1],
            street_labels[2],
            you_in_strip,
            same,
            short_label,
            ring,
            note,
            share_name,
            wait_name,
            share_bar,
            wait_bar,
            share_value,
            random_value,
            planned_value,
        ];
        for (i, &a) in over_the_line.iter().enumerate() {
            let top = tri_top(a.0, a.1);
            assert!(a.2 > top, "item {i} reaches into the triangles: {a:?} under {top:.3}");
        }
        // 50 min² sits inside its own triangle, clear of its bus marker
        let (xa, xb) = (x_at(long_one.from as f64), x_at(long_one.to as f64));
        assert!(long_label.0 > xa + marker_half && long_label.2 > marker_top(0)[1]);
        assert!(long_label.3 < LINE_Y + xb - long_label.1);
        // the line up to your moment passes between the passengers and the
        // clock labels, clear of you and your words
        let mut near: Vec<Box4> = ready.iter().map(|&t| dot(x_of(t), DOT_Y, r)).collect();
        for m in (0..=HOUR).step_by(PERIOD as usize) {
            near.push(centred(&format!(":{m:02}"), clock_size, x_at(m as f64), clock_y));
        }
        near.extend([you_out, ask, answer]);
        for (i, &a) in near.iter().enumerate() {
            assert!(apart(guide, a), "the guide crosses item {i}: {a:?}");
        }
    }
}
