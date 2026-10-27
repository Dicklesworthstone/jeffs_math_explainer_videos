//! Chapter 07, no bus was late. The same timetable, buses and passengers:
//! - 07_1: every bus on time (chapter 01's green marks), every time written
//!   in the timetable, a row per route over the markers, and the gaps
//!   bracketed as chapter 02 measured them.
//! - 07_2: real buses bunch: a small dim inset, two markers drifting
//!   together on a loop, a different story (K04). Here the gaps are only
//!   uneven.
//! - 07_3: the even timetable: the red bus slides to six minutes after the
//!   blue, as in 03_1, and every gap is six. The average wait there? (a
//!   prediction; no triangles yet.)
//! - 07_4: three minutes. Ten equal triangles over the line; half of six,
//!   the sign's number; over buses and over passengers the gap is six.
//! - 07_5: the red bus slides again, the triangles reshaping with it, while
//!   a plot traces the average wait against how long the red bus runs after
//!   the blue, and a readout gives it to the second. Ten buses an hour and a
//!   six-minute average gap throughout. Then five seconds of picture with no
//!   voice: the sweep, to the end and back to our timetable.
//! - 07_6: the U's points at two minutes, six and none, each as it is said,
//!   the red bus jumping there.
//! - 07_7: same buses, same sign, three minutes to six; the red bus goes
//!   back to our timetable.
//!
//! One tracker, the red bus's offset, drives 07_3 to 07_7: updaters keep the
//! red markers, the passengers' colors, the triangles, the plot and the
//! readout true to it between whole minutes. The readout rounds the exact
//! wait to the second, the video's one rounding; every other number on
//! screen is read from the facts, and `plan` checks each against the model
//! at every whole offset before anything is drawn.

use std::f64::consts::{PI, TAU};
use std::rc::Rc;

use explainer_kit::props::precisely;
use explainer_kit::{Facts, Kit, Palette, TextKind, fade_all, ink_style, rect, tween, v};
use fmn::prelude::*;

use super::draw::{
    LINE_X0, LINE_X1, LINE_Y, PASSENGER_R, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_lagged, flat,
    line_style, marker_top, now, pause, read_s, say, stand, steady, think, wait_until, x_of,
};
use super::timetable::{self, gap_color, x_at};
use crate::facts;
use crate::model::{self, EVEN_OFFSET, HOUR, OFFSET, PERIOD, Q};
use crate::palette;

pub struct NoBusLate {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are read from the facts)
const ON_TIME: &str = "every bus on time";
const INSET_LINES: [&str; 3] = ["bus bunching:", "delays feed on themselves", "(a different story)"];
/// The exact version of 07_2's claim (K04).
const K04: &str = "the effect needs only uneven gaps, from any cause; bus bunching is one cause, a feedback between \
                   delays and boarding, outside this video";
const ASK: &str = "average wait now?";
const OVER_BUSES: &str = "over buses:";
const OVER_PASSENGERS: &str = "over passengers:";
const AVERAGE_WAIT: &str = "average wait:";
const MIN: &str = "min";
const SLIDER: &str = "red bus after blue (min)";

// ------------------------------------------------------------------ layout

/// 07_1: a green mark over each marker and chapter 01's words; the
/// timetable's rows over them, each time over its own marker.
const ON_TIME_DY: f64 = 0.18;
const ON_TIME_AT: Vec3 = [0.0, 1.55, 0.0];
const ON_TIME_SIZE: f64 = 20.0;
const BLUE_ROW_Y: f64 = 2.0;
const RED_ROW_Y: f64 = 2.4;
const TIME_SIZE: f64 = 18.0;
/// 07_2: the inset at the top right, clear of the timetable's rows: its
/// card, the loop at its left, its three lines beside the loop.
const INSET_DL: Vec3 = [4.3, 2.75, 0.0];
const INSET_W: f64 = 2.7;
const INSET_H: f64 = 1.1;
const LOOP_AT: Vec3 = [4.75, 3.3, 0.0];
const LOOP_R: f64 = 0.28;
const INSET_TEXT_X: f64 = 5.15;
const INSET_YS: [f64; 3] = [3.6, 3.3, 3.0];
const INSET_SIZE: f64 = 14.0;
const INSET_MARKER: [f64; 2] = [0.06, 0.16];
/// The two markers on the loop: the one behind starts half a loop back and
/// closes to this angle behind the one ahead, never passing it, while they
/// go round this many times.
const LOOP_START: f64 = 0.0;
const LOOP_TURNS: f64 = 1.25;
const LOOP_CLOSEST: f64 = 0.45;
const BUNCH_RT: f64 = 4.0;
/// 07_3: the question right of the sign, the think ring right of it.
const ASK_AT: Vec3 = [1.2, -1.2, 0.0];
const ASK_SIZE: f64 = 24.0;
const THINK_AT: Vec3 = [4.6, -1.5, 0.0];
const SLIDE_RT: f64 = 1.6;
/// 07_4: the triangles over the line, a minute up as long as a minute
/// across; a triangle with no width has nothing to draw, so the drawing
/// keeps the red bus this far (in minutes) inside the ends.
const TRI_FILL: f64 = 0.35;
const TRI_W: f64 = 3.0;
const TRI_MIN: f64 = 0.01;
/// 07_4's boxes under the line, right of the sign.
const BOX_ATS: [Vec3; 3] = [[0.0, -1.3, 0.0], [3.0, -1.3, 0.0], [1.5, -2.05, 0.0]];
const BOX_SIZE: f64 = 22.0;
const BOX_PAD: [f64; 2] = [0.2, 0.15];
const BOX_W: f64 = 2.0;
/// 07_5's plot, lower right: the red bus's offset across (0 to 12 minutes;
/// its axis is the slider's track) and the average wait up (0 to 6
/// minutes); the readout over it.
const PLOT_O: Vec3 = [1.2, -3.3, 0.0];
const PLOT_XS: f64 = 0.35;
const PLOT_YS: f64 = 0.33;
const PLOT_SIZE: f64 = 16.0;
const PLOT_TICK_DY: f64 = 0.22;
const PLOT_NAME_DY: f64 = 0.48;
const CURVE_W: f64 = 4.0;
const CURVE_STEP: f64 = 0.05;
const CURVE_SEED: f64 = 0.05;
const KNOB_R: f64 = 0.08;
const POINT_R: f64 = 0.07;
const READOUT_AT: Vec3 = [3.0, -1.0, 0.0];
const READOUT_SIZE: f64 = 24.0;
/// 07_5's sweep after "Only the wait changes": to the buses together, on to
/// twelve minutes apart, back to our timetable, each leg (to, seconds)
/// easing in and out; the silent window it plays on through.
const SWEEP: [(f64, f64); 3] = [(0.0, 1.6), (PERIOD as f64, 3.8), (OFFSET as f64, 1.6)];
const LISTEN_S: f64 = 5.0;
/// 07_6: each spoken point's label, placed off the curve: two minutes up
/// and right of its point, six under it, none left of the wait axis.
const MARK_SIZE: f64 = 18.0;
const MARK_D2: [f64; 2] = [0.3, 0.22];
const MARK_D6_DY: f64 = -0.25;
const MARK_D0_DX: f64 = 0.12;
const JUMP_RT: f64 = 0.8;
/// Emphasis: an outline around a text, faded in, held, faded out.
const HALO_PAD: f64 = 0.1;
const HALO_W: f64 = 3.0;
const HALO_RT: f64 = 0.9;

// ------------------------------------------------------------- the numbers

/// What chapter 07 prints, read from the facts and checked against the
/// model at every whole offset the slider passes.
pub struct Plan {
    /// The timetable's rows as printed (":00"), each with its minute.
    blue: Vec<(i64, String)>,
    red: Vec<(i64, String)>,
    /// Our gaps' two lengths, as their brackets print them.
    short: String,
    long: String,
    /// The even timetable: its offset, its gap, its wait spelt out.
    even: i64,
    even_gap: String,
    even_wait_s: String,
    /// 07_6's three points in the order spoken (two minutes, six, none),
    /// each with its clock.
    marked: [(i64, String); 3],
    /// The offset axis's ticks: six and twelve.
    ticks: [(i64, String); 2],
}

impl Plan {
    /// 07_4's boxes: "over buses: 6 min", "over passengers: 6 min",
    /// "average wait: 3 min".
    fn boxes(&self) -> [String; 3] {
        [
            format!("{OVER_BUSES} {} {MIN}", self.even_gap),
            format!("{OVER_PASSENGERS} {} {MIN}", self.even_gap),
            format!("{AVERAGE_WAIT} {}", self.even_wait_s),
        ]
    }
}

/// A fact's comma list of minutes ("0,12,24").
fn minutes(key: &str, printed: &str) -> Result<Vec<i64>, String> {
    printed
        .split(',')
        .map(|s| s.parse::<i64>().map_err(|e| format!("{key} = {printed}: {e}")))
        .collect()
}

/// The slider's offset `d` (minutes, whole or not) as an exact number, to
/// the nearest tenth of a second, kept within 0 to 12 minutes (a tracker
/// eased onto an end can land a hair past it).
fn tenths(d: f64) -> Q {
    Q::new((d.clamp(0.0, PERIOD as f64) * 600.0).round() as i64, 600)
}

/// The average wait with the red bus `d` minutes after the blue, for
/// drawing (`model::offset_wait_rational`).
pub fn wait_of(d: f64) -> f64 {
    model::offset_wait_rational(tenths(d)).map_or(0.0, Q::to_f64)
}

/// The readout in 07_5 to 07_7: the exact average wait at the slider's
/// offset, rounded to the nearest second, halves up (`model::clock_rounded`):
/// "average wait: 4:01" at two and a half minutes.
pub fn readout(d: f64) -> Result<String, String> {
    let wait = model::offset_wait_rational(tenths(d))?;
    Ok(format!("{AVERAGE_WAIT} {}", model::clock_rounded(wait)?))
}

/// The plan, refused when a fact the picture prints disagrees with the
/// model at any whole offset.
pub fn plan(f: &Facts) -> Result<Plan, String> {
    let get = |key: &str| f.get(key).map(str::to_owned).ok_or_else(|| format!("no fact {key}"));
    let blue = minutes("F-T1.blue_times", &get("F-T1.blue_times")?)?;
    let red = minutes("F-T1.red_times", &get("F-T1.red_times")?)?;
    if blue != model::route(0) || red != model::route(OFFSET) {
        return Err("the timetable's rows are not the two routes".into());
    }
    let (short, long) = facts::short_and_long();
    let even: i64 = get("F-O1.even_offset")?
        .parse()
        .map_err(|e| format!("F-O1.even_offset: {e}"))?;
    if even != EVEN_OFFSET {
        return Err(format!("the even timetable's offset is {even}, not {EVEN_OFFSET}"));
    }
    let even_gap = get("F-O1.even_gap")?;
    let even_gaps = model::hour_gaps(even);
    if even_gaps.iter().any(|g| g.to_string() != even_gap) {
        return Err(format!("the even timetable's gaps are not all {even_gap}"));
    }
    let clocks: Vec<String> = get("F-O1.wait_clocks")?.split(',').map(str::to_owned).collect();
    if clocks.len() != PERIOD as usize + 1 {
        return Err("F-O1.wait_clocks does not give one clock per whole offset".into());
    }
    let waits: Vec<Q> = (0..=PERIOD).map(model::offset_wait).collect();
    let buses = get("F-G1.buses_every_offset")?;
    let mean_gap = get("F-G1.mean_gap_every_offset")?;
    for d in 0..=PERIOD {
        let i = d as usize;
        if model::offset_wait_rational(Q::int(d))? != waits[i] {
            return Err(format!("offset {d}: the slider's wait is not the timetable's"));
        }
        if model::clock(waits[i])? != clocks[i] {
            return Err(format!("offset {d}: the wait is {}, not {}", waits[i], clocks[i]));
        }
        if readout(d as f64)? != format!("{AVERAGE_WAIT} {}", clocks[i]) {
            return Err(format!("offset {d}: the readout rounds a whole-second wait"));
        }
        if model::bus_times(d).len().to_string() != buses {
            return Err(format!("offset {d}: not {buses} buses an hour"));
        }
        if model::mean_gap(&model::hour_gaps(d)).to_string() != mean_gap {
            return Err(format!("offset {d}: the average gap over buses is not {mean_gap}"));
        }
    }
    let min = *waits.iter().min().ok_or("no offsets")?;
    let max = *waits.iter().max().ok_or("no offsets")?;
    let best = waits.iter().position(|&w| w == min).ok_or("no minimum")? as i64;
    let together = get("F-O1.together_period")?;
    let mut apart = model::hour_gaps(0);
    apart.retain(|&g| g > 0);
    let between = parse_q("F-O1.between_offset", &get("F-O1.between_offset")?)?;
    let checks = [
        ("the short gaps' length", short.to_string(), "F-T1.short"),
        ("the long gaps' length", long.to_string(), "F-T1.long"),
        (
            "the even gap over buses",
            model::mean_gap(&even_gaps).to_string(),
            "F-O1.even_gap",
        ),
        (
            "the even gap over passengers",
            model::passengers_gap(&even_gaps).to_string(),
            "F-O1.even_gap",
        ),
        (
            "the even wait",
            model::mean_wait(&even_gaps).to_string(),
            "F-O1.even_wait",
        ),
        (
            "half the even gap",
            waits[even as usize].to_string(),
            "F-T1.half_mean_gap",
        ),
        (
            "the even wait",
            model::clock(waits[even as usize])?,
            "F-O1.even_wait_clock",
        ),
        ("the shortest wait", min.to_string(), "F-O1.min_wait"),
        ("the longest wait", max.to_string(), "F-O1.max_wait"),
        ("the best offset", best.to_string(), "F-O1.best_offset"),
        ("the best offset", best.to_string(), "F-O1.even_offset"),
        ("our timetable's wait", clocks[OFFSET as usize].clone(), "F-O1.wait.d2"),
        ("the even wait", clocks[best as usize].clone(), "F-O1.wait.d6"),
        ("the buses together", clocks[0].clone(), "F-O1.wait.d0"),
        (
            "our timetable's wait",
            clocks[OFFSET as usize].clone(),
            "F-W1.mean_wait_clock",
        ),
        (
            "the buses together, waiting for a bus every",
            if apart.iter().all(|&g| g == apart[0]) {
                apart[0].to_string()
            } else {
                "uneven".into()
            },
            "F-O1.together_period",
        ),
        (
            "half the together period",
            (parse_q("F-O1.together_period", &together)? / Q::int(2)).to_string(),
            "F-O1.max_wait",
        ),
        (
            "the readout between whole minutes",
            readout(between.to_f64())?,
            "F-O1.between_readout",
        ),
        ("the offset axis's end", PERIOD.to_string(), "F-T1.period"),
        (
            "the sign's buses",
            model::bus_times(OFFSET).len().to_string(),
            "F-T1.buses_per_hour",
        ),
    ];
    for (what, drawn, key) in checks {
        let shown = get(key)?;
        let shown = if key == "F-O1.between_readout" {
            format!("{AVERAGE_WAIT} {shown}")
        } else {
            shown
        };
        if drawn != shown {
            return Err(format!("{what}: the picture draws {drawn}, but {key} is {shown}"));
        }
    }
    let row = |times: &[i64]| -> Vec<(i64, String)> { times.iter().map(|&t| (t, format!(":{t:02}"))).collect() };
    Ok(Plan {
        blue: row(&blue),
        red: row(&red),
        short: get("F-T1.short")?,
        long: get("F-T1.long")?,
        even,
        even_gap,
        even_wait_s: super::draw::min_s(&get("F-O1.even_wait_clock")?).map_err(|e| e.to_string())?,
        marked: [
            (OFFSET, get("F-O1.wait.d2")?),
            (best, get("F-O1.wait.d6")?),
            (0, get("F-O1.wait.d0")?),
        ],
        ticks: [(even, get("F-O1.even_offset")?), (PERIOD, get("F-T1.period")?)],
    })
}

/// A fact's printed fraction ("5/2", "12") as a number.
fn parse_q(key: &str, printed: &str) -> Result<Q, String> {
    let int = |s: &str| s.parse::<i64>().map_err(|e| format!("{key} = {printed}: {e}"));
    match printed.split_once('/') {
        Some((n, d)) => Ok(Q::new(int(n)?, int(d)?)),
        None => Ok(Q::int(int(printed)?)),
    }
}

// ----------------------------------------------------------------- picture

/// Where the red marker of the `k`-th twelve minutes stands with the red bus
/// `d` minutes after the blue: at its moment, or a marker's width (`beside`)
/// from a blue marker it would cover, the one it leaves with the buses
/// together or the next one it reaches twelve minutes on. Chapter 03's
/// rule, kept clear at both ends.
pub fn red_x(k: usize, d: f64, beside: f64) -> f64 {
    let base = (k as i64 * PERIOD) as f64;
    let twelve = x_at(PERIOD as f64) - x_at(0.0);
    x_at(base) + (x_at(d) - x_at(0.0)).clamp(beside, twelve - beside)
}

/// A minute up over the line, as long as a minute across it.
pub fn minute_up() -> f64 {
    (LINE_X1 - LINE_X0) / HOUR as f64
}

/// The hour's triangles with the red bus `d` minutes after the blue: in
/// each twelve minutes the gap from the blue bus to the red, then from the
/// red to the next blue, each as tall as it is wide, as (moment, wait)
/// corners in minutes.
pub fn triangles_at(d: f64) -> Vec<[(f64, f64); 3]> {
    let mut out = Vec::new();
    for k in 0..model::route(0).len() {
        let base = (k as i64 * PERIOD) as f64;
        for (a, b) in [(base, base + d), (base + d, base + PERIOD as f64)] {
            out.push([(a, 0.0), (a, b - a), (b, 0.0)]);
        }
    }
    out
}

/// A triangle's corners on screen, over the line.
fn over_line(c: [(f64, f64); 3]) -> Vec<Vec3> {
    c.iter().map(|&(t, w)| v(x_at(t), LINE_Y + w * minute_up())).collect()
}

/// Where a point of the plot sits: the red bus `d` minutes after the blue,
/// an average wait of `wait` minutes.
pub fn plot_at(d: f64, wait: f64) -> Vec3 {
    v(PLOT_O[0] + PLOT_XS * d, PLOT_O[1] + PLOT_YS * wait)
}

/// The U from offset `lo` to `hi`, as points on the plot, at least two.
pub fn curve(lo: f64, hi: f64) -> Vec<Vec3> {
    let n = (((hi - lo) / CURVE_STEP).ceil() as usize).max(1);
    (0..=n)
        .map(|i| {
            let d = lo + (hi - lo) * i as f64 / n as f64;
            plot_at(d, wait_of(d))
        })
        .collect()
}

/// 07_5's sweep, `s` seconds after it starts from offset `start`.
pub fn sweep_at(start: f64, s: f64) -> f64 {
    let (mut from, mut left) = (start, s.max(0.0));
    for (to, seconds) in SWEEP {
        if left < seconds {
            let u = left / seconds;
            return from + (to - from) * u * u * (3.0 - 2.0 * u);
        }
        left -= seconds;
        from = to;
    }
    from
}

/// The sweep's length in seconds.
pub fn sweep_s() -> f64 {
    SWEEP.iter().map(|&(_, s)| s).sum()
}

/// 07_2's two markers on the loop, as angles (the one ahead, the one
/// behind) once the tracker has run to `s` of the way: the one behind goes
/// round steadily and closes from half a loop back to [`LOOP_CLOSEST`].
pub fn loop_angles(s: f64) -> (f64, f64) {
    let behind = LOOP_START + LOOP_TURNS * TAU * s;
    (behind + PI - (PI - LOOP_CLOSEST) * s, behind)
}

/// A filled triangle with its outline, over the line; detached.
fn triangle(stage: &mut Stage<'_>, c: [(f64, f64); 3], color: Srgb) -> Mob {
    let style = Style::default().fill(color, TRI_FILL).stroke(color, TRI_W, 1.0);
    stage.arena_mut().add(Polygon::new(over_line(c)).style(style).build())
}

/// A comparison box: its words in `color`, outlined in it, as one group;
/// detached.
fn boxed(kit: &Kit, stage: &mut Stage<'_>, text: &str, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
    let words = kit.text(stage, text, BOX_SIZE, color, at)?;
    let w = stage.get_width(words) + 2.0 * BOX_PAD[0];
    let h = stage.get_height(words) + 2.0 * BOX_PAD[1];
    let outline = rect(v(at[0] - w / 2.0, at[1] - h / 2.0), w, h, line_style(color, BOX_W))?;
    let outline = stage.arena_mut().add(outline);
    super::draw::group(stage, &[outline, words])
}

/// Emphasis for texts: an outline just around each, stroked only, faded
/// in, held and faded out over `rt`, the words left as they are.
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

/// A small dot in `color` at `at`; detached.
fn spot(stage: &mut Stage<'_>, at: Vec3, r: f64, color: Srgb) -> Mob {
    stage
        .arena_mut()
        .add(Dot::new().point(at).radius(r).style(flat(color, 1.0)).build())
}

/// A fresh anchor for an updater, in the scene (clear its updaters to stop
/// the updater).
fn anchor(stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    let anchor = stage.arena_mut().add(VMobject::new());
    stage.add_to_scene(anchor)?;
    Ok(anchor)
}

/// Keep each red marker the tracker `d`'s minutes after its blue bus.
/// Returns the updater's anchor.
fn move_reds(stage: &mut Stage<'_>, d: Mob, reds: &[Mob], beside: f64) -> fmn::Result<Mob> {
    let reds = reds.to_vec();
    let a = anchor(stage)?;
    stage.add_updater(
        a,
        move |s, _| {
            let dv = s.tracker_value(d).unwrap_or(0.0);
            for (k, &m) in reds.iter().enumerate() {
                s.set_x(m, red_x(k, dv, beside));
            }
        },
        true,
    )?;
    Ok(a)
}

/// Keep each passenger gold while it stands in a gap that starts at a red
/// bus (the red marker has passed it in its twelve minutes), teal in a gap
/// that starts at a blue bus: our long and short gaps at two minutes, the
/// triangles' colors at any offset. Returns the updater's anchor.
fn paint(stage: &mut Stage<'_>, d: Mob, dots: &[Mob], people: &[Q]) -> fmn::Result<Mob> {
    let dots = dots.to_vec();
    let places: Vec<f64> = people.iter().map(|t| t.to_f64().rem_euclid(PERIOD as f64)).collect();
    let mut shown: Vec<Option<bool>> = vec![None; dots.len()];
    let (gold, teal) = (palette::long_gap(), timetable::short_gap());
    let a = anchor(stage)?;
    stage.add_updater(
        a,
        move |s, _| {
            let dv = s.tracker_value(d).unwrap_or(0.0);
            for (i, &m) in dots.iter().enumerate() {
                let now_gold = places[i] >= dv;
                if shown[i] != Some(now_gold) {
                    s.set_color(m, if now_gold { gold } else { teal }, None, true);
                    shown[i] = Some(now_gold);
                }
            }
        },
        true,
    )?;
    Ok(a)
}

/// Keep the triangles the tracker `d`'s shape. Returns the updater's
/// anchor.
fn reshape(stage: &mut Stage<'_>, d: Mob, shapes: &[Mob]) -> fmn::Result<Mob> {
    let shapes = shapes.to_vec();
    let a = anchor(stage)?;
    stage.add_updater(
        a,
        move |s, _| {
            let dv = s
                .tracker_value(d)
                .unwrap_or(0.0)
                .clamp(TRI_MIN, PERIOD as f64 - TRI_MIN);
            for (&m, c) in shapes.iter().zip(triangles_at(dv)) {
                let _ = s.set_points(m, Polygon::new(over_line(c)).build().points());
            }
        },
        true,
    )?;
    Ok(a)
}

/// Keep the plot's point on the U at the tracker `d`'s offset, the knob
/// under it on the offset axis with a guide up to it, and the trace over
/// every offset the slider has reached since `start`. Returns the
/// updater's anchor.
fn follow(stage: &mut Stage<'_>, d: Mob, parts: [Mob; 4], start: f64) -> fmn::Result<Mob> {
    let [point, knob, guide, trace] = parts;
    let (mut lo, mut hi) = (start - CURVE_SEED, start + CURVE_SEED);
    let a = anchor(stage)?;
    stage.add_updater(
        a,
        move |s, _| {
            let dv = s.tracker_value(d).unwrap_or(start).clamp(0.0, PERIOD as f64);
            (lo, hi) = (lo.min(dv), hi.max(dv));
            let (foot, top) = (plot_at(dv, 0.0), plot_at(dv, wait_of(dv)));
            s.move_to(point, top, ORIGIN);
            s.move_to(knob, foot, ORIGIN);
            let _ = s.set_points(guide, polyline(vec![foot, top]).build().points());
            let _ = s.set_points(trace, polyline(curve(lo, hi)).build().points());
        },
        true,
    )?;
    Ok(a)
}

/// Run the tracker `d` along the sweep from `start` by the scene clock, so
/// the sweep plays on through a silent window. Returns the updater's
/// anchor.
fn drive(stage: &mut Stage<'_>, d: Mob, start: f64) -> fmn::Result<Mob> {
    let a = anchor(stage)?;
    let mut elapsed = 0.0;
    stage.add_dt_updater(
        a,
        move |s, _, dt| {
            elapsed += dt;
            let _ = s.set_tracker_value(d, sweep_at(start, elapsed));
        },
        true,
    )?;
    Ok(a)
}

/// The readout following the tracker `d`, left-aligned at `left` as
/// `Kit::text_left` places a text. Detached (out of the scene) until added.
fn live_readout(kit: &Kit, stage: &mut Stage<'_>, d: Mob, left: Vec3) -> Mob {
    let book = Rc::clone(&kit.book);
    let m = stage.always_redraw(move |s| {
        let text = readout(s.tracker_value(d).unwrap_or(0.0)).expect("the slider's wait is never negative");
        explainer_kit::glyphs::record(TextKind::Text, &text);
        let shown = Text::new(&text)
            .font_size(READOUT_SIZE)
            .style(ink_style(palette::verdict()))
            .build(&book)
            .expect("the bundled face lays out digits");
        let m = s.add(shown);
        s.move_to(m, left, LEFT);
        m
    });
    stage.remove_from_scene(m);
    m
}

/// The plot's frame: the offset axis (the slider's track), the wait axis,
/// the offset ticks and the slider's name.
fn plot_frame(kit: &Kit, stage: &mut Stage<'_>, p: &Plan) -> fmn::Result<Vec<Mob>> {
    let x_end = plot_at(PERIOD as f64, 0.0);
    let y_end = plot_at(0.0, wait_of(0.0));
    let mut out = Vec::new();
    for end in [x_end, y_end] {
        out.push(
            stage
                .arena_mut()
                .add(Line::new(PLOT_O, end).style(line_style(Palette::dim(), 2.0)).build()?),
        );
    }
    for (d, label) in &p.ticks {
        let at = plot_at(*d as f64, 0.0);
        out.push(kit.text(stage, label, PLOT_SIZE, Palette::dim(), v(at[0], at[1] - PLOT_TICK_DY))?);
    }
    out.push(kit.text(
        stage,
        SLIDER,
        PLOT_SIZE,
        Palette::dim(),
        v((PLOT_O[0] + x_end[0]) / 2.0, PLOT_O[1] - PLOT_NAME_DY),
    )?);
    Ok(out)
}

/// Where 07_6's label for the point at offset `d` goes, and how it aligns
/// there.
fn mark_at(p: &Plan, d: i64) -> (Vec3, Vec3) {
    let at = plot_at(d as f64, wait_of(d as f64));
    if d == 0 {
        (v(PLOT_O[0] - MARK_D0_DX, at[1]), RIGHT)
    } else if d == p.marked[1].0 {
        (v(at[0], at[1] + MARK_D6_DY), ORIGIN)
    } else {
        (v(at[0] + MARK_D2[0], at[1] + MARK_D2[1]), ORIGIN)
    }
}

/// 07_6's `i`-th spoken point: a dot on the U and its clock beside it,
/// faded in. Returns the label.
fn mark_point(kit: &Kit, stage: &mut Stage<'_>, p: &Plan, i: usize) -> fmn::Result<Mob> {
    let (d, clock) = &p.marked[i];
    let dot = spot(
        stage,
        plot_at(*d as f64, wait_of(*d as f64)),
        POINT_R,
        palette::verdict(),
    );
    let (at, edge) = mark_at(p, *d);
    let label = kit.text(stage, clock, MARK_SIZE, palette::verdict(), at)?;
    stage.move_to(label, at, edge);
    fade_in_all(stage, &[dot, label], 0.4)?;
    Ok(label)
}

impl SceneConstruct for NoBusLate {
    fn name(&self) -> &str {
        "07_no_bus_late"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let f = facts::facts().map_err(fail)?;
        let p = plan(&f).map_err(fail)?;
        let (_, long) = facts::short_and_long();
        let gaps = timetable::gaps(OFFSET);
        let people = model::passengers();
        let gap_of = timetable::gap_of_each(&gaps).map_err(fail)?;
        let (teal, gold, green) = (timetable::short_gap(), palette::long_gap(), palette::verdict());

        // 07_1: the timetable and the passengers in their gaps' colors; a
        // green mark on every bus; the timetable's rows over the markers,
        // and our gaps' brackets: every gap written in advance
        let line = say(&kit, stage, "07_1")?;
        let table = timetable::timetable(&kit, stage, &f, OFFSET)?;
        let dots: Vec<Mob> = people
            .iter()
            .zip(&gap_of)
            .map(|(&t, &g)| timetable::passenger(stage, t, gap_color(gaps[g].minutes())))
            .collect();
        let mut opening = table.mobs();
        opening.extend(&dots);
        fade_in_all(stage, &opening, 0.8)?;
        line.wait_for(stage, "No bus broke down")?;
        let mut checks: Vec<Mob> = p
            .blue
            .iter()
            .chain(&p.red)
            .map(|&(t, _)| {
                let top = marker_top(t);
                spot(stage, v(top[0], top[1] + ON_TIME_DY), PASSENGER_R, green)
            })
            .collect();
        checks.push(kit.text(stage, ON_TIME, ON_TIME_SIZE, green, ON_TIME_AT)?);
        fade_in_lagged(stage, &checks, 0.8, 0.1)?;
        line.wait_for(stage, "written into the timetable")?;
        let mut rows = Vec::new();
        for (times, y, color) in [
            (&p.blue, BLUE_ROW_Y, palette::blue_bus()),
            (&p.red, RED_ROW_Y, palette::red_bus()),
        ] {
            for (t, label) in times {
                rows.push(kit.text(stage, label, TIME_SIZE, color, v(x_of(Q::int(*t)), y))?);
            }
        }
        fade_in_lagged(stage, &rows, 1.2, 0.08)?;
        let mut ours = Vec::new();
        for g in &gaps {
            let label = if g.minutes() == long { &p.long } else { &p.short };
            let b = timetable::bracket(&kit, stage, g, label, gap_color(g.minutes()))?;
            ours.extend([b.line, b.label]);
        }
        fade_in_lagged(stage, &ours, 1.2, 0.1)?;

        // 07_2: the inset, two buses on a loop, the one behind closing on
        // the late one; a different story (K04); our gaps are simply
        // uneven, and the long ones stretch the wait
        let line = say(&kit, stage, "07_2")?;
        let card = stage
            .arena_mut()
            .add(rect(INSET_DL, INSET_W, INSET_H, line_style(Palette::dim(), 2.0))?);
        let ring = stage.arena_mut().add(
            Circle::new()
                .radius(LOOP_R)
                .arc_center(LOOP_AT)
                .style(line_style(Palette::faint(), 2.0))
                .build(),
        );
        let mut inset = vec![card, ring];
        let pair: Vec<Mob> = (0..2)
            .map(|_| {
                let [w, h] = INSET_MARKER;
                rect(v(-w / 2.0, -h / 2.0), w, h, flat(palette::blue_bus(), 1.0)).map(|r| stage.arena_mut().add(r))
            })
            .collect::<fmn::Result<_>>()?;
        inset.extend(&pair);
        for (text, y) in INSET_LINES.iter().zip(INSET_YS) {
            inset.push(kit.text_left(stage, text, INSET_SIZE, Palette::dim(), v(INSET_TEXT_X, y))?);
        }
        let bunch = stage.add_value_tracker(0.0);
        let on_loop = anchor(stage)?;
        let riding = pair.clone();
        stage.add_updater(
            on_loop,
            move |s, _| {
                let (ahead, behind) = loop_angles(s.tracker_value(bunch).unwrap_or(0.0));
                for (&m, a) in riding.iter().zip([ahead, behind]) {
                    s.move_to(
                        m,
                        v(LOOP_AT[0] + LOOP_R * a.cos(), LOOP_AT[1] + LOOP_R * a.sin()),
                        ORIGIN,
                    );
                }
            },
            true,
        )?;
        line.wait_for(stage, "Real buses do bunch")?;
        fade_in_all(stage, &inset, 0.6)?;
        line.wait_for(stage, "A late bus")?;
        stage.play(steady(bunch, 1.0, BUNCH_RT)?)?;
        line.wait_for(stage, "a different story")?;
        let k04 = precisely(&kit, stage, K04)?;
        fade_in_all(stage, &[k04], 0.5)?;
        let card_up = now(stage);
        line.wait_for(stage, "simply uneven")?;
        let long_lines: Vec<Mob> = ours
            .chunks(2)
            .zip(&gaps)
            .filter(|(_, g)| g.minutes() == long)
            .map(|(b, _)| b[0])
            .collect();
        flash_lagged(stage, &long_lines, 1.0, 0.15)?;
        kit.hold(stage)?;
        wait_until(stage, card_up + read_s(K04))?;
        stage.clear_updaters(on_loop, true);

        // 07_3: the inset, K04, the marks, the rows and our brackets go; the
        // red bus slides to six minutes after the blue, the passengers'
        // colors following; every gap six; the question; think time
        let line = say(&kit, stage, "07_3")?;
        let mut leaving = inset;
        leaving.push(k04);
        leaving.extend(&checks);
        leaving.extend(&rows);
        leaving.extend(&ours);
        fade_out_all(stage, &leaving, 0.5)?;
        let d = stage.add_value_tracker(OFFSET as f64);
        let beside = stage.get_width(table.blue[0]);
        move_reds(stage, d, &table.red, beside)?;
        paint(stage, d, &dots, &people)?;
        line.wait_for(stage, "Go back to the even timetable")?;
        stage.play(tween(d, p.even as f64, SLIDE_RT)?)?;
        line.wait_for(stage, "Every gap is six minutes")?;
        let mut evens = Vec::new();
        for g in &timetable::gaps(p.even) {
            let b = timetable::bracket(&kit, stage, g, &p.even_gap, gap_color(g.minutes()))?;
            evens.push((b.line, b.label));
        }
        let even_mobs: Vec<Mob> = evens.iter().flat_map(|&(l, t)| [l, t]).collect();
        fade_in_lagged(stage, &even_mobs, 1.0, 0.1)?;
        line.wait_for(stage, "What's the average wait")?;
        let ask = kit.text(stage, ASK, ASK_SIZE, Palette::ink(), ASK_AT)?;
        fade_in_all(stage, &[ask], 0.4)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 07_4: three minutes: ten equal triangles over the line and the
        // wait; half of six, the sign's number; the buses and the
        // passengers agree
        let line = say(&kit, stage, "07_4")?;
        fade_out_all(stage, &[q, ask], 0.3)?;
        let shapes: Vec<Mob> = triangles_at(p.even as f64)
            .into_iter()
            .enumerate()
            .map(|(i, c)| triangle(stage, c, if i % 2 == 0 { teal } else { gold }))
            .collect();
        let labels = p.boxes();
        let wait_box = boxed(&kit, stage, &labels[2], green, BOX_ATS[2])?;
        fade_in_lagged(stage, &shapes, 1.0, 0.1)?;
        fade_in_all(stage, &[wait_box], 0.4)?;
        reshape(stage, d, &shapes)?;
        line.wait_for(stage, "Half of six")?;
        let even_lines: Vec<Mob> = evens.iter().map(|&(l, _)| l).collect();
        flash_lagged(stage, &even_lines, 0.8, 0.08)?;
        line.wait_for(stage, "what the sign suggests")?;
        halo(stage, &[table.sign.every], Palette::ink(), HALO_RT)?;
        line.wait_for(stage, "the buses and the passengers agree")?;
        let agree = [
            boxed(&kit, stage, &labels[0], Palette::dim(), BOX_ATS[0])?,
            boxed(&kit, stage, &labels[1], gold, BOX_ATS[1])?,
        ];
        fade_in_all(stage, &agree, 0.5)?;

        // 07_5: the boxes and the even brackets go; the plot, its point,
        // the knob and the readout at six minutes; ten buses, a six-minute
        // average gap; then the sweep, on through five silent seconds
        let line = say(&kit, stage, "07_5")?;
        let mut leaving = agree.to_vec();
        leaving.push(wait_box);
        leaving.extend(&even_mobs);
        fade_out_all(stage, &leaving, 0.5)?;
        line.wait_for(stage, "watching the wait")?;
        let mut plot = plot_frame(&kit, stage, &p)?;
        let start = p.even as f64;
        let point = spot(stage, plot_at(start, wait_of(start)), POINT_R, green);
        let knob = spot(stage, plot_at(start, 0.0), KNOB_R, palette::you());
        let guide = stage.arena_mut().add(
            polyline(vec![plot_at(start, 0.0), plot_at(start, wait_of(start))])
                .style(line_style(Palette::faint(), 2.0))
                .build(),
        );
        let trace = stage.arena_mut().add(
            polyline(curve(start - CURVE_SEED, start + CURVE_SEED))
                .style(line_style(green, CURVE_W))
                .build(),
        );
        let still = kit.text_left(stage, &readout(start).map_err(fail)?, READOUT_SIZE, green, READOUT_AT)?;
        plot.extend([guide, trace, point, knob, still]);
        fade_in_all(stage, &plot, 0.6)?;
        let live = live_readout(&kit, stage, d, READOUT_AT);
        stage.remove_from_scene(still);
        stage.add_to_scene(live)?;
        let follower = follow(stage, d, [point, knob, guide, trace], start)?;
        line.wait_for(stage, "The same ten buses")?;
        halo(stage, &[table.sign.buses], Palette::ink(), HALO_RT)?;
        line.wait_for(stage, "the same average gap")?;
        halo(stage, &[table.sign.every], Palette::ink(), HALO_RT)?;
        line.wait_for(stage, "Only the wait changes")?;
        let swept = now(stage);
        let driver = drive(stage, d, start)?;
        kit.music_breath(stage, LISTEN_S)?;
        wait_until(stage, swept + sweep_s())?;
        stage.clear_updaters(driver, true);
        stage.set_tracker_value(d, OFFSET as f64)?;

        // 07_6: the U's point at each offset as it is said, the red bus
        // jumping there: two minutes, six, none (a bus every twelve
        // minutes: the blue markers, each with its red beside it)
        let line = say(&kit, stage, "07_6")?;
        line.wait_for(stage, "it's four minutes twenty")?;
        let mut marks = vec![mark_point(&kit, stage, &p, 0)?];
        line.wait_for(stage, "Six apart")?;
        stage.play(tween(d, p.marked[1].0 as f64, JUMP_RT)?)?;
        line.wait_for(stage, "it's three")?;
        marks.push(mark_point(&kit, stage, &p, 1)?);
        line.wait_for(stage, "come together")?;
        stage.play(tween(d, p.marked[2].0 as f64, JUMP_RT)?)?;
        line.wait_for(stage, "every twelve minutes")?;
        flash_lagged(stage, &table.blue, 0.8, 0.1)?;
        line.wait_for(stage, "the average is six")?;
        marks.push(mark_point(&kit, stage, &p, 2)?);

        // 07_7: same buses, same sign; three minutes to six; the red bus
        // back to our timetable
        let line = say(&kit, stage, "07_7")?;
        line.wait_for(stage, "same sign")?;
        halo(stage, &[table.sign.card], Palette::ink(), HALO_RT)?;
        line.wait_for(stage, "from three minutes to six")?;
        halo(stage, &[marks[1], marks[2]], green, HALO_RT)?;
        line.wait_for(stage, "Only the spacing changed")?;
        stage.play(tween(d, OFFSET as f64, SLIDE_RT)?)?;
        stage.clear_updaters(follower, true);
        let up = now(stage);
        stand(&kit, stage, up)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use explainer_kit::{Narrator, Script};

    use super::*;

    /// The full draft with its line breaks and indents as single spaces.
    fn draft() -> String {
        crate::DRAFT.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// Needs chapter 07's lines in script.tsv (the integration step copies
    /// them from the draft). The whole chapter runs headless at 10 fps with
    /// the scratch voice: every phrase it waits for is in its line, the plan
    /// holds at every whole offset, and its seven lines are said in order.
    /// It ends with `kit.finish`, whose timeline runs past the end of the
    /// last line.
    #[test]
    fn the_chapter_runs_and_says_its_lines_in_order() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let narrator = Narrator::estimate(&script);
        let kit = Kit::new().unwrap().with_narrator(Rc::clone(&narrator));
        let mut scene = NoBusLate { kit };
        let config = fmn::scene::RuntimeConfig {
            fps: 10,
            ..fmn::scene::RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut fmn::scene::NullSceneSink).unwrap();
        let said_at = narrator.said();
        let said: Vec<&str> = said_at.iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(said, ["07_1", "07_2", "07_3", "07_4", "07_5", "07_6", "07_7"]);
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

    /// K04 is the draft's exact version of 07_2 word for word, and what the
    /// chapter prints is the draft's picture: the inset, the question, the
    /// three boxes, the readout's form and the U's two labelled ends.
    #[test]
    fn the_printed_words_are_the_drafts() {
        let board = draft();
        assert!(board.contains(&format!("\"{K04}\"")), "{K04}");
        let p = plan(&facts::facts().unwrap()).unwrap();
        let mut shown = vec![INSET_LINES.join(" "), ASK.to_owned()];
        shown.extend(p.boxes());
        for s in shown {
            assert!(board.contains(&format!("\"{s}\"")), "not in the draft: {s}");
        }
        assert!(board.contains(&format!("\"{AVERAGE_WAIT} m:ss\"")));
        for (d, clock) in &p.marked {
            assert!(board.contains(&format!("d = {d} ({clock})")), "{d} {clock}");
        }
        assert_eq!(
            p.boxes(),
            ["over buses: 6 min", "over passengers: 6 min", "average wait: 3 min"]
        );
    }

    /// The plan holds, and a fact that stops matching the model is refused:
    /// our timetable's wait, the even wait and the red route, each changed.
    #[test]
    fn the_plan_is_the_models_and_a_wrong_fact_is_refused() {
        let f = facts::facts().unwrap();
        let p = plan(&f).unwrap();
        assert_eq!(p.even, EVEN_OFFSET);
        assert_eq!(
            p.marked,
            [(2, "4:20".to_owned()), (6, "3:00".to_owned()), (0, "6:00".to_owned())]
        );
        assert_eq!(p.blue[1], (12, ":12".to_owned()));
        assert_eq!(p.red[0], (2, ":02".to_owned()));
        for (key, bad) in [
            ("F-O1.wait.d2", "4:30"),
            ("F-O1.even_wait", "4"),
            ("F-T1.red_times", "3,15,27,39,51"),
            ("F-O1.between_readout", "4:02"),
        ] {
            let mut wrong = Facts::new();
            for row in f.print().lines() {
                let (k, value) = row.split_once('\t').unwrap();
                wrong.add(k, if k == key { bad } else { value }).unwrap();
            }
            assert!(plan(&wrong).is_err(), "a wrong {key} must be refused");
        }
    }

    /// The readout is the model's wait to the second: exact at every whole
    /// offset, rounded between them (4:01 at two and a half minutes), and
    /// never outside the slider's ends.
    #[test]
    fn the_readout_rounds_only_between_whole_seconds() {
        let f = facts::facts().unwrap();
        let clocks: Vec<&str> = f.get("F-O1.wait_clocks").unwrap().split(',').collect();
        assert_eq!(clocks.len(), PERIOD as usize + 1);
        for (d, clock) in clocks.iter().enumerate() {
            assert_eq!(readout(d as f64).unwrap(), format!("{AVERAGE_WAIT} {clock}"));
        }
        assert_eq!(
            readout(2.5).unwrap(),
            format!("{AVERAGE_WAIT} {}", f.get("F-O1.between_readout").unwrap())
        );
        assert_eq!(readout(12.000_000_1).unwrap(), readout(12.0).unwrap());
        assert_eq!(readout(-1e-9).unwrap(), readout(0.0).unwrap());
    }

    /// The red marker never covers a blue marker, at any offset: beside the
    /// blue it leaves with the buses together, beside the next blue twelve
    /// minutes on, at its moment between.
    #[test]
    fn the_red_bus_never_covers_a_blue_one() {
        let beside = 0.14;
        for k in 0..model::route(0).len() {
            let base = (k as i64 * PERIOD) as f64;
            for step in 0..=1200 {
                let d = f64::from(step) / 100.0;
                let x = red_x(k, d, beside);
                assert!(x - x_at(base) >= beside - 1e-12, "{k} {d}");
                assert!(x_at(base + PERIOD as f64) - x >= beside - 1e-12, "{k} {d}");
                if (1.0..=11.0).contains(&d) {
                    assert!((x - x_at(base + d)).abs() < 1e-12, "{k} {d}");
                }
            }
        }
    }

    /// The triangles are drawn a minute up as long as a minute across, each
    /// as tall as its gap, and at every tenth of a minute their area over
    /// the hour is the model's average wait; at six minutes all ten are the
    /// same.
    #[test]
    fn the_triangles_hold_the_average_wait() {
        assert!((minute_up() - (x_at(1.0) - x_at(0.0))).abs() < 1e-12);
        for tenth in 0_i32..=120 {
            let d = f64::from(tenth) / 10.0;
            let ts = triangles_at(d);
            assert_eq!(ts.len(), model::bus_times(OFFSET).len());
            let mut area = 0.0;
            for [a, top, b] in &ts {
                assert_eq!(a.1, 0.0);
                assert!((top.1 - (b.0 - a.0)).abs() < 1e-12 && top.0 == a.0);
                area += (b.0 - a.0) * top.1 / 2.0;
            }
            let wait = model::offset_wait_rational(Q::new(i64::from(tenth), 10))
                .unwrap()
                .to_f64();
            assert!((area / HOUR as f64 - wait).abs() < 1e-9, "{d}");
            assert!((wait_of(d) - wait).abs() < 1e-12, "{d}");
        }
        let even = triangles_at(EVEN_OFFSET as f64);
        assert!(
            even.iter()
                .all(|t| (t[2].0 - t[0].0 - EVEN_OFFSET as f64).abs() < 1e-12)
        );
    }

    /// The U is the model's: its lowest point at the even timetable, its
    /// ends at the buses together, and every traced point on it.
    #[test]
    fn the_plot_traces_the_models_u() {
        let p = plan(&facts::facts().unwrap()).unwrap();
        let points = curve(0.0, PERIOD as f64);
        assert!(points.len() > 200);
        for pt in &points {
            let d = (pt[0] - PLOT_O[0]) / PLOT_XS;
            assert!((pt[1] - plot_at(d, wait_of(d))[1]).abs() < 1e-9);
        }
        let lowest = points.iter().min_by(|a, b| a[1].partial_cmp(&b[1]).unwrap()).unwrap();
        assert!(((lowest[0] - PLOT_O[0]) / PLOT_XS - p.even as f64).abs() < 1e-9);
        assert!((wait_of(0.0) - wait_of(PERIOD as f64)).abs() < 1e-12);
        assert_eq!(curve(6.0, 6.0).len(), 2);
    }

    /// The sweep starts at the even timetable, reaches the buses together
    /// and twelve minutes apart, ends at our timetable, never leaves the
    /// slider's ends, never jumps, and outlasts the silent window.
    #[test]
    fn the_sweep_runs_end_to_end_and_home() {
        let start = EVEN_OFFSET as f64;
        let near = |a: f64, b: f64| (a - b).abs() < 1e-9;
        assert_eq!(sweep_at(start, 0.0), start);
        assert!(near(sweep_at(start, SWEEP[0].1), 0.0));
        assert!(near(sweep_at(start, SWEEP[0].1 + SWEEP[1].1), PERIOD as f64));
        assert!(near(sweep_at(start, sweep_s()), OFFSET as f64));
        assert_eq!(sweep_at(start, sweep_s() + 5.0), OFFSET as f64);
        let (mut lo, mut hi, mut last) = (start, start, start);
        for i in 1..=10_000 {
            let d = sweep_at(start, sweep_s() * f64::from(i) / 10_000.0);
            assert!((0.0..=PERIOD as f64).contains(&d), "{d}");
            assert!((d - last).abs() < 0.02, "a jump at {i}");
            (lo, hi, last) = (lo.min(d), hi.max(d), d);
        }
        assert!(lo < 1e-6 && hi > PERIOD as f64 - 1e-6, "{lo} {hi}");
        assert!(sweep_s() > LISTEN_S);
    }

    /// On the loop the bus behind closes on the one ahead, from half a loop
    /// back to its closest, and never passes it.
    #[test]
    fn the_bus_behind_closes_but_never_passes() {
        let (a, b) = loop_angles(0.0);
        assert!((a - b - PI).abs() < 1e-12);
        let (a, b) = loop_angles(1.0);
        assert!((a - b - LOOP_CLOSEST).abs() < 1e-12);
        let mut last = PI;
        for i in 1..=100 {
            let (a, b) = loop_angles(f64::from(i) / 100.0);
            let gap = a - b;
            assert!(gap < last && gap >= LOOP_CLOSEST - 1e-12);
            last = gap;
        }
    }

    /// The texts up together stay apart and on screen; the inset clear of
    /// the timetable's rows, the boxes and the plot clear of the sign, the
    /// lower-third under everything up with it. Widths are estimated, not
    /// measured: 0.0045 units a point a character, a line 0.6 of its size
    /// tall.
    #[test]
    fn the_picture_stays_apart_and_on_screen() {
        let p = plan(&facts::facts().unwrap()).unwrap();
        let w = |s: &str, pt: f64| s.chars().count() as f64 * pt * 0.0045;
        let h = |pt: f64| pt * 0.0045 / 0.55 * 0.6;
        type Box4 = (f64, f64, f64, f64);
        let centred = |s: &str, pt: f64, at: Vec3| -> Box4 {
            (
                at[0] - w(s, pt) / 2.0,
                at[0] + w(s, pt) / 2.0,
                at[1] - h(pt),
                at[1] + h(pt),
            )
        };
        let left = |s: &str, pt: f64, at: Vec3| -> Box4 { (at[0], at[0] + w(s, pt), at[1] - h(pt), at[1] + h(pt)) };
        let boxed = |s: &str, at: Vec3| -> Box4 {
            let b = centred(s, BOX_SIZE, at);
            (b.0 - BOX_PAD[0], b.1 + BOX_PAD[0], b.2 - BOX_PAD[1], b.3 + BOX_PAD[1])
        };
        let apart = |a: Box4, b: Box4| a.1 < b.0 || b.1 < a.0 || a.3 < b.2 || b.3 < a.2;
        let on_screen = |a: Box4| a.0 > -7.1 && a.1 < 7.1 && a.2 > -4.0 && a.3 < 4.0;
        let sign = (-6.15, -2.45, -2.25, -0.85);
        let dots_row = (-6.1, 6.1, -0.66, -0.54);
        let time_rows: Vec<Box4> = p
            .blue
            .iter()
            .map(|(t, s)| centred(s, TIME_SIZE, v(x_of(Q::int(*t)), BLUE_ROW_Y)))
            .chain(
                p.red
                    .iter()
                    .map(|(t, s)| centred(s, TIME_SIZE, v(x_of(Q::int(*t)), RED_ROW_Y))),
            )
            .collect();
        let on_time = centred(ON_TIME, ON_TIME_SIZE, ON_TIME_AT);
        let inset = (INSET_DL[0], INSET_DL[0] + INSET_W, INSET_DL[1], INSET_DL[1] + INSET_H);
        let inset_text: Vec<Box4> = INSET_LINES
            .iter()
            .zip(INSET_YS)
            .map(|(s, y)| left(s, INSET_SIZE, v(INSET_TEXT_X, y)))
            .collect();
        // K04's band (`props::lower_third`): 0.4 in from each side, its
        // foot 0.2 off the frame's, 0.3 + 0.36 a line tall, wrapped at 92
        // characters
        let lines = |s: &str| s.chars().count().div_ceil(92) as f64;
        let k04 = (-6.71, 6.71, -3.8, -3.8 + 0.3 + 0.36 * lines(K04));
        let ask = centred(ASK, ASK_SIZE, ASK_AT);
        let ring = (
            THINK_AT[0] - 0.62,
            THINK_AT[0] + 0.62,
            THINK_AT[1] - 0.62,
            THINK_AT[1] + 0.62,
        );
        let note = centred("pause for more time", 18.0, v(THINK_AT[0], THINK_AT[1] - 0.62 - 0.3));
        let labels = p.boxes();
        let boxes: Vec<Box4> = labels.iter().zip(BOX_ATS).map(|(s, at)| boxed(s, at)).collect();
        let x_end = plot_at(PERIOD as f64, 0.0);
        let plot = (PLOT_O[0], x_end[0], PLOT_O[1], plot_at(0.0, wait_of(0.0))[1]);
        let mut plot_texts: Vec<Box4> = p
            .ticks
            .iter()
            .map(|(d, s)| {
                let at = plot_at(*d as f64, 0.0);
                centred(s, PLOT_SIZE, v(at[0], at[1] - PLOT_TICK_DY))
            })
            .collect();
        plot_texts.push(centred(
            SLIDER,
            PLOT_SIZE,
            v((PLOT_O[0] + x_end[0]) / 2.0, PLOT_O[1] - PLOT_NAME_DY),
        ));
        let readout_box = left(&readout(2.5).unwrap(), READOUT_SIZE, READOUT_AT);
        let marks: Vec<Box4> = p
            .marked
            .iter()
            .map(|(d, s)| {
                let (at, edge) = mark_at(&p, *d);
                let b = centred(s, MARK_SIZE, at);
                if edge == RIGHT {
                    (b.0 - w(s, MARK_SIZE) / 2.0, b.1 - w(s, MARK_SIZE) / 2.0, b.2, b.3)
                } else {
                    b
                }
            })
            .collect();
        // a mark stays off the curve: its box holds no point of the U
        for (m, (d, _)) in marks.iter().zip(&p.marked) {
            for pt in curve(0.0, PERIOD as f64) {
                let inside = pt[0] > m.0 && pt[0] < m.1 && pt[1] > m.2 && pt[1] < m.3;
                assert!(!inside, "the label at {d} min sits on the U");
            }
        }
        // the triangles at their tallest (the buses together) stay under
        // the frame's top
        assert!(LINE_Y + PERIOD as f64 * minute_up() < 4.0);
        let moments: Vec<Vec<Box4>> = vec![
            // 07_1 and 07_2: the rows, the marks' words, the inset, K04
            [vec![sign, dots_row, on_time, inset, k04], time_rows.clone(), inset_text].concat(),
            // 07_3: the question and its ring
            vec![sign, dots_row, ask, ring, note],
            // 07_4: the boxes
            [vec![sign, dots_row], boxes].concat(),
            // 07_5 to 07_7: the plot, its words, the readout, the marks
            [vec![sign, dots_row, readout_box], plot_texts, marks].concat(),
        ];
        for (m, up) in moments.iter().enumerate() {
            for (i, &a) in up.iter().enumerate() {
                assert!(on_screen(a), "moment {m}: item {i} off screen: {a:?}");
                for (j, &b) in up.iter().enumerate().skip(i + 1) {
                    // the inset's words sit inside its card by design
                    if m == 0 && i == 3 && j >= 5 + time_rows.len() {
                        continue;
                    }
                    assert!(apart(a, b), "moment {m}: items {i} and {j} overlap: {a:?} {b:?}");
                }
            }
        }
        for t in moments[0][5 + time_rows.len()..].iter() {
            assert!(
                t.0 > inset.0 && t.1 < inset.1 && t.2 > inset.2 && t.3 < inset.3,
                "{t:?}"
            );
        }
        assert!(apart(plot, sign) && apart(plot, readout_box) && apart(plot, dots_row));
    }

    /// The chapter's fixed words carry no digits: every number on screen is
    /// read from the facts.
    #[test]
    fn the_fixed_words_carry_no_numbers() {
        for w in [
            ON_TIME,
            INSET_LINES[0],
            INSET_LINES[1],
            INSET_LINES[2],
            K04,
            ASK,
            OVER_BUSES,
            OVER_PASSENGERS,
            AVERAGE_WAIT,
            MIN,
            SLIDER,
        ] {
            assert!(!w.chars().any(|c| c.is_ascii_digit()), "{w}");
        }
    }
}
