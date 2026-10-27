//! Chapter 09, no timetable at all. A fresh hour line:
//! - 09_1: buses at random moments, a seeded picture labelled
//!   "illustration" with no numbers on it. A cursor runs along it: the time
//!   since the last bus grows and starts again at every bus, while the
//!   chance that a bus comes this moment stays the same.
//! - 09_2: the model keeps the average, a bus every six minutes ("model");
//!   the wait's triangles rise over the line, a few tiny and a few huge, and
//!   the huge ones pulse.
//! - 09_3: the average wait now? (a prediction; no wait on screen.)
//! - 09_4: six minutes, beside the model's average gap; K06.
//! - 09_5: you, at your moment of chapters 02 and 03; the past dims; the
//!   wait ahead of you, 6 min on average, against a new gap from a bus, the
//!   same 6 min: the buses have no memory.
//! - 09_6: chapter 08's formula with the model's numbers: the spread 36, and
//!   3 min + 36 min² ÷ 12 min = 6 min.
//! - 09_7: back from you, the last bus left 6 min before, on average; your
//!   gap, 12 min on average. The names to search.
//! - 09_8: a scale of the average wait for a bus every six minutes on
//!   average: even gaps 3 min, completely random 6 min, our timetable 4 min
//!   20 s, and further out gaps of 0, 0 and 18 minutes at 9 min. The scale
//!   runs on past them, unlabelled; K07.
//!
//! The illustration comes from a fixed seed and is checked ([`fits`]) so it
//! reads: no two markers touching, the tallest triangle on screen, some gaps
//! tiny and some long, your moment clear of both buses of a gap at least the
//! model's average (where a passenger usually lands, 09_7). Its own average
//! gap is not six, and nothing says it is: every number on screen is the
//! model's, read from the facts and checked by [`random`] against `model`.

use explainer_kit::props::{names_to_search, precisely};
use explainer_kit::{Facts, Kit, Palette, fade_all, rect, v};
use fmn::prelude::*;

use super::draw::{
    LINE_X0, LINE_X1, LINE_Y, bus_marker, fade_in_all, fade_in_lagged, fade_out_all, fade_to, fail, flat, hour_line,
    line_style, min_s, now, read_s, say, stand, steady, think, wait_until, you,
};
use super::the_formula::{self, Role, Row, frame_around, halo, play_all, pulse, show_row, spread_color, text_box};
use super::timetable::{self, x_at};
use super::twelve_minutes::MIN;
use crate::facts::{self, csv};
use crate::model::{self, CLUMPED_GAPS, EVEN_OFFSET, HOUR, OFFSET, Q, RANDOM_MEAN};
use crate::palette;

pub struct AtRandom {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are read from the facts)
const ILLUSTRATION: &str = "illustration";
const SINCE_LAST: &str = "time since the last bus";
const CHANCE: &str = "chance a bus comes this moment";
const AVERAGE_GAP: &str = "average gap:";
const MODEL: &str = "(model)";
const ASK: &str = "average wait now?";
const AVERAGE_WAIT: &str = "average wait:";
const AHEAD: &str = "wait ahead of you:";
const FRESH: &str = "a new gap:";
const ON_AVERAGE: &str = "on average";
const SINCE: &str = "since the last bus:";
const YOUR_GAP: &str = "your gap:";
const SPREAD_IS: &str = "spread =";
const AREA: &str = "min²";
const SQUARED: &str = "²";
const PLUS: &str = "+";
const DIVIDE: &str = "÷";
const EQUALS: &str = "=";
const SCALE_NAME: &str = "average wait, a bus every";
const EVEN: &str = "even gaps:";
const RANDOM: &str = "completely random:";
const OURS: &str = "our timetable:";
const GAPS: &str = "gaps";
const AVERAGE: &str = "average";
const MORE_SPREAD: &str = "more spread";
const NAMES: [&str; 2] = ["waiting-time paradox", "bus paradox"];
/// The exact version of 09_4's answer (K06), the draft's, in pieces around
/// the model's numbers.
const K06_MODEL: &str = "model: buses as a Poisson process, one every";
const K06_MEAN: &str = "minutes on average; gaps exponential, mean";
const K06_VARIANCE: &str = "variance";
const K06_REST: &str = "passengers turn up independently of the buses; long-run averages";
/// The exact version of 09_8's claim (K07), the draft's word for word: both
/// conditions on turning up, and both on the gaps.
const K07: &str = "the formula holds for passengers who turn up independently of the buses and spread evenly in time, \
                   when the gaps have a long-run average and spread (finite mean and finite mean square)";

// ------------------------------------------------------------------ layout

/// A minute up over the line, as long as a minute across it.
const MINUTE_UP: f64 = (LINE_X1 - LINE_X0) / HOUR as f64;
/// The label under the line's right end.
const ILLUSTRATION_AT: Vec3 = [LINE_X1, 0.25, 0.0];
const SMALL: f64 = 16.0;
/// 09_1: the "now" cursor over the line; the time since the last bus drawn
/// on the line from that bus to the cursor; the two gauges top right, their
/// names left-aligned, their bars from one edge (the time since the last
/// bus this many units a minute; the chance a fixed length, for it never
/// changes).
const CURSOR_DY: [f64; 2] = [-0.15, 0.6];
const CURSOR_W: f64 = 2.0;
const SPAN_W: f64 = 7.0;
const GAUGE_NAME_X: f64 = 0.6;
const GAUGE_BAR_X: f64 = 4.4;
const GAUGE_YS: [f64; 2] = [3.2, 2.6];
const GAUGE_SIZE: f64 = 18.0;
const GAUGE_W: f64 = 10.0;
const SINCE_U: f64 = 0.12;
const CHANCE_LEN: f64 = 0.9;
/// 09_1's sweep: from the first bus to this minute, steadily, in this many
/// seconds.
const SWEEP_TO: f64 = 33.0;
const SWEEP_RT: f64 = 5.0;
/// The triangles' fill, and how far 09_5 and 09_8 dim the past.
const TRI_FILL: f64 = 0.35;
const DIM_FILL: f64 = 0.12;
const DIM_MARKER: f64 = 0.3;
/// Under the line: the model's box and the answer's, one row; the question
/// and its ring on the same row before the answer.
const BOX_ROW: f64 = -1.5;
const MODEL_AT: Vec3 = [-3.6, BOX_ROW, 0.0];
const ANSWER_AT: Vec3 = [1.6, BOX_ROW, 0.0];
const BOX_SIZE: f64 = 22.0;
const ASK_SIZE: f64 = 24.0;
const THINK_AT: Vec3 = [5.0, BOX_ROW, 0.0];
/// 09_5 and 09_7: the comparison, on its own scale (units a minute) from
/// your moment: row A, the wait ahead of you (and in 09_7 the time back to
/// the last bus); row B, a new gap from a bus; your gap's bracket under row
/// A once row B has gone.
const DIAGRAM_U: f64 = 0.3;
const ROW_A_Y: f64 = -0.25;
const ROW_B_Y: f64 = -0.8;
const ARROW_W: f64 = 3.0;
const LABEL_DX: f64 = 0.15;
const LABEL_SIZE: f64 = 18.0;
const BACK_LABEL_DY: f64 = 0.25;
const BUS_MARK: [f64; 2] = [0.1, 0.24];
const BRACKET_DY: f64 = -0.3;
const BRACKET_TICK: f64 = 0.1;
const BRACKET_LABEL_DY: f64 = -0.55;
/// 09_6's rows, left-aligned under the boxes: chapter 08's formula, dim;
/// the spread; the numbers, boxed.
const ROWS_X: f64 = -4.9;
const ROW_YS: [f64; 3] = [-2.2, -2.75, -3.3];
const FORMULA_SIZE: f64 = 18.0;
const ROW_SIZE: f64 = 20.0;
/// 09_8's scale: a minute a unit from its zero, running on past its last
/// mark to [`SCALE_MAX`] minutes and an arrow tip; each mark's label above
/// or below it; the scale's name over its left end; the "more spread"
/// arrow over the marks, from the even gaps' wait onward.
const SCALE_X0: f64 = -5.0;
const SCALE_Y: f64 = -2.0;
const SCALE_U: f64 = 1.0;
const SCALE_MAX: f64 = 10.0;
const SCALE_TIP: f64 = 0.6;
const MARK_TICK: f64 = 0.12;
const MARK_DY: [f64; 2] = [0.35, -0.35];
const SCALE_NAME_AT: Vec3 = [SCALE_X0, -0.95, 0.0];
const SPREAD_ARROW_Y: f64 = -1.25;
const SPREAD_ARROW_END: f64 = 9.6;
const SPREAD_LABEL_DY: f64 = 0.25;
const MARK_SIZE: f64 = 18.0;
/// Emphasis; 09_6's boxed six held silent before the rows go.
const HALO_RT: f64 = 0.9;
const VERDICT_LOOK_S: f64 = 1.5;

// --------------------------------------------------------- the illustration

/// The illustration's seed: the first from 1 that [`fits`], found with a
/// Python mirror of [`illustration`]; the tests pin what it draws.
pub const SEED: u64 = 358;
/// What the picture needs to read (see [`fits`]). Markers are 0.14 wide, a
/// minute 0.2 across, so buses [`APART`] minutes apart never touch.
const FIRST_MIN: f64 = 0.5;
const LAST_MAX: f64 = 59.3;
const APART: f64 = 0.8;
const TALLEST: f64 = 15.0;
const TINY: f64 = 1.5;
const LONG: f64 = 12.0;
const CLEAR: f64 = 1.2;
const COUNT: [usize; 2] = [7, 13];

/// splitmix64: the next of a fixed sequence of 64-bit numbers.
fn splitmix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// The illustration's buses from `seed`: from :00, gaps drawn as the model
/// draws them (exponential, the model's average), each moment rounded to a
/// hundredth of a minute. Returns the buses in the hour, and the first bus
/// after it, whose gap the last triangle runs into.
pub fn illustration(seed: u64) -> (Vec<f64>, f64) {
    let mut state = seed;
    let mut t = 0.0;
    let mut out = Vec::new();
    loop {
        let u = ((splitmix(&mut state) >> 11) as f64 + 0.5) / (1u64 << 53) as f64;
        t += -(RANDOM_MEAN as f64) * u.ln();
        let at = (t * 100.0 + 0.5).floor() / 100.0;
        if at > HOUR as f64 {
            return (out, at);
        }
        out.push(at);
    }
}

/// Whether the illustration reads: between seven and thirteen buses, the
/// first and last clear of the line's ends, no two markers touching, the
/// tallest triangle (the first wait, every gap, the last gap) on screen,
/// at least one gap tiny and one long, and moment `me` clear of both buses
/// of a gap at least the model's average.
pub fn fits(buses: &[f64], after: f64, me: f64) -> Result<(), String> {
    let (Some(&first), Some(&last)) = (buses.first(), buses.last()) else {
        return Err("no buses".into());
    };
    if buses.len() < COUNT[0] || buses.len() > COUNT[1] {
        return Err(format!("{} buses", buses.len()));
    }
    if first < FIRST_MIN || last > LAST_MAX {
        return Err(format!("a bus at {first} or {last} min crowds the line's end"));
    }
    let gaps: Vec<f64> = buses.windows(2).map(|w| w[1] - w[0]).collect();
    let shortest = gaps.iter().copied().fold(f64::INFINITY, f64::min);
    let longest = gaps.iter().copied().fold(0.0, f64::max);
    if shortest < APART {
        return Err(format!("two markers {shortest:.2} min apart touch"));
    }
    let tallest = longest.max(first).max(after - last);
    if tallest > TALLEST {
        return Err(format!("a triangle {tallest:.2} min tall runs off the screen"));
    }
    if shortest > TINY || longest < LONG {
        return Err(format!(
            "gaps from {shortest:.2} to {longest:.2} min: none tiny or none long"
        ));
    }
    let Some(&before) = buses.iter().rev().find(|&&b| b < me) else {
        return Err(format!("no bus before your moment {me} min"));
    };
    let ahead = buses.iter().copied().chain([after]).find(|&b| b > me).unwrap_or(after);
    if me - before < CLEAR || ahead - me < CLEAR {
        return Err(format!("your moment {me} min is too close to a bus"));
    }
    if ahead - before < RANDOM_MEAN as f64 {
        return Err(format!("your gap, {:.2} min, is shorter than average", ahead - before));
    }
    Ok(())
}

/// The wait of someone who turns up at moment `t`: until the next bus.
#[cfg(test)]
fn wait_at(buses: &[f64], after: f64, t: f64) -> f64 {
    buses.iter().copied().chain([after]).find(|&b| b > t).unwrap_or(after) - t
}

/// A piece of the waiting picture: (moment, wait) corners in minutes, the
/// length of the gap it belongs to, and whether it lies wholly behind you.
#[derive(Clone, Debug, PartialEq)]
pub struct Piece {
    pub corners: Vec<(f64, f64)>,
    pub gap: f64,
    pub behind: bool,
}

/// The waiting from moment `a` to `end` with the next bus at `b` (no
/// earlier than `end`): a triangle if the bus comes by `end`, cut off at
/// `end` otherwise.
fn waiting(a: f64, end: f64, b: f64) -> Vec<(f64, f64)> {
    if end < b {
        vec![(a, 0.0), (a, b - a), (end, b - end), (end, 0.0)]
    } else {
        vec![(a, 0.0), (a, b - a), (b, 0.0)]
    }
}

/// The hour's waiting as pieces in time order: before the first bus (the
/// bus before :00 is off the line, so this piece's gap is the part shown),
/// each gap's triangle, the last gap cut off at the hour's end; the gap
/// holding moment `me` split there, so the part behind you can dim.
pub fn pieces(buses: &[f64], after: f64, me: f64) -> Vec<Piece> {
    let hour = HOUR as f64;
    let mut out = Vec::new();
    if let Some(&first) = buses.first() {
        out.push(Piece {
            corners: waiting(0.0, first, first),
            gap: first,
            behind: first <= me,
        });
    }
    let mut stops = buses.to_vec();
    stops.push(after);
    for w in stops.windows(2) {
        let (a, b) = (w[0], w[1]);
        let end = b.min(hour);
        if a < me && me < b {
            out.push(Piece {
                corners: vec![(a, 0.0), (a, b - a), (me, b - me), (me, 0.0)],
                gap: b - a,
                behind: true,
            });
            out.push(Piece {
                corners: waiting(me, end, b),
                gap: b - a,
                behind: false,
            });
        } else {
            out.push(Piece {
                corners: waiting(a, end, b),
                gap: b - a,
                behind: b <= me,
            });
        }
    }
    out
}

/// (moment, wait) corners over the hour line.
fn over_line(corners: &[(f64, f64)]) -> Vec<Vec3> {
    corners
        .iter()
        .map(|&(t, w)| v(x_at(t), LINE_Y + w * MINUTE_UP))
        .collect()
}

// ------------------------------------------------------------- the numbers

/// What chapter 09 prints, all the model's, read from the facts and checked
/// against `model`.
pub struct Random {
    pub mean_gap: String,
    pub spread: String,
    pub half_mean: String,
    pub twice_mean: String,
    pub mean_wait: String,
    pub mean_age: String,
    pub person_gap: String,
    /// 09_8's marks in the order spoken (even gaps, completely random, our
    /// timetable, then gaps 0, 0, 18): each wait in minutes, for its place,
    /// and its label.
    pub marks: [(Q, String); 4],
}

/// A fact's printed fraction ("13/3", "6") as a number.
fn parse_q(key: &str, printed: &str) -> Result<Q, String> {
    let int = |s: &str| s.parse::<i64>().map_err(|e| format!("{key} = {printed}: {e}"));
    match printed.split_once('/') {
        Some((n, d)) => Ok(Q::new(int(n)?, int(d)?)),
        None => Ok(Q::int(int(printed)?)),
    }
}

/// The chapter's numbers, refused when a fact disagrees with the model.
pub fn random(f: &Facts) -> Result<Random, String> {
    let get = |key: &str| f.get(key).map(str::to_owned).ok_or_else(|| format!("no fact {key}"));
    let drawn = |key: &str, value: String| -> Result<String, String> {
        let printed = get(key)?;
        if printed == value {
            Ok(printed)
        } else {
            Err(format!("{key} is {printed}, but the model has {value}"))
        }
    };
    // completely random buses: exponential gaps, the model's first two
    // moments
    let (m1, m2) = model::exponential_moments(RANDOM_MEAN);
    let mean_gap = drawn("F-R1.mean_gap", m1.to_string())?;
    let spread = m2 - m1 * m1;
    let spread_s = drawn("F-R1.spread", spread.to_string())?;
    if spread != m1 * m1 {
        return Err("the random gaps' spread is not the average gap squared".into());
    }
    // chapter 08's formula: half the average gap, plus the spread over
    // twice the average gap (the formula's ½ and 2 from F-F1.area_factor)
    let factor = parse_q("F-F1.area_factor", &get("F-F1.area_factor")?)?;
    let half_mean = drawn("F-R1.half_mean", (m1 * factor).to_string())?;
    let twice_mean = drawn("F-R1.twice_mean", (m1 / factor).to_string())?;
    let wait = m1 * factor + spread / (m1 / factor);
    let mean_wait = drawn("F-R1.mean_wait", wait.to_string())?;
    // no memory: the wait ahead is a new gap's average, the time back the
    // same, and the gap you land in both together
    if wait != m1 {
        return Err("the wait ahead is not a new gap's average".into());
    }
    drawn("F-R1.wait_over_mean", (wait / m1).to_string())?;
    let mean_age = drawn("F-R1.mean_age", wait.to_string())?;
    let person = m2 / m1;
    let person_gap = drawn("F-R1.person_gap", person.to_string())?;
    if person != wait + wait {
        return Err("your gap is not the time back plus the wait ahead".into());
    }
    drawn("F-R1.person_gap_ratio", (person / m1).to_string())?;

    // 09_8's comparison: every timetable on it has the same average gap
    let ours = model::hour_gaps(OFFSET);
    let even = model::hour_gaps(EVEN_OFFSET);
    for (what, gaps) in [
        ("our timetable", &ours[..]),
        ("even gaps", &even[..]),
        ("0, 0, 18", &CLUMPED_GAPS[..]),
    ] {
        if model::mean_gap(gaps) != m1 {
            return Err(format!("{what} does not average a bus every {m1} minutes"));
        }
    }
    let even_wait = drawn("F-O1.even_wait", model::mean_wait(&even).to_string())?;
    let ours_wait = model::mean_wait(&ours);
    drawn("F-W1.mean_wait", ours_wait.to_string())?;
    let ours_clock = drawn("F-W1.mean_wait_clock", model::clock(ours_wait)?)?;
    let clumped_gaps = drawn("F-R2.gaps", csv(CLUMPED_GAPS))?;
    let clumped_mean = drawn("F-R2.mean_gap", model::mean_gap(&CLUMPED_GAPS).to_string())?;
    let clumped = model::mean_wait(&CLUMPED_GAPS);
    let clumped_wait = drawn("F-R2.mean_wait", clumped.to_string())?;
    drawn(
        "F-R2.exceeds_random",
        if clumped > wait { "true" } else { "false" }.to_owned(),
    )?;
    // more spread, a longer wait: the spreads in the scale's order
    let spreads = [
        model::spread(&even),
        model::spread(&ours),
        spread,
        model::spread(&CLUMPED_GAPS),
    ];
    let waits = [model::mean_wait(&even), ours_wait, wait, clumped];
    let rising = |xs: &[Q]| xs.windows(2).all(|w| w[0] < w[1]);
    if !rising(&spreads) || !rising(&waits) {
        return Err("the scale's marks are not in order of spread and of wait".into());
    }
    if Q::int(SCALE_MAX as i64) <= clumped {
        return Err(format!("the scale ends at {SCALE_MAX} min, before its last mark"));
    }
    let spaced = clumped_gaps.split(',').collect::<Vec<_>>().join(", ");
    Ok(Random {
        mean_gap,
        spread: spread_s,
        half_mean,
        twice_mean,
        mean_wait: mean_wait.clone(),
        mean_age,
        person_gap,
        marks: [
            (waits[0], format!("{EVEN} {even_wait} {MIN}")),
            (waits[2], format!("{RANDOM} {mean_wait} {MIN}")),
            (
                waits[1],
                format!("{OURS} {}", min_s(&ours_clock).map_err(|e| e.to_string())?),
            ),
            (
                waits[3],
                format!("{GAPS} {spaced} ({AVERAGE} {clumped_mean}): {clumped_wait} {MIN}"),
            ),
        ],
    })
}

impl Random {
    /// "average gap: 6 min (model)".
    pub fn model_box(&self) -> String {
        format!("{AVERAGE_GAP} {} {MIN} {MODEL}", self.mean_gap)
    }

    /// "average wait: 6 min".
    pub fn answer(&self) -> String {
        format!("{AVERAGE_WAIT} {} {MIN}", self.mean_wait)
    }

    /// 09_5's two rows, and 09_7's time back and your gap.
    pub fn ahead(&self) -> String {
        format!("{AHEAD} {} {MIN} {ON_AVERAGE}", self.mean_wait)
    }

    pub fn fresh(&self) -> String {
        format!("{FRESH} {} {MIN} {ON_AVERAGE}", self.mean_gap)
    }

    pub fn since(&self) -> String {
        format!("{SINCE} {} {MIN} {ON_AVERAGE}", self.mean_age)
    }

    pub fn your_gap(&self) -> String {
        format!("{YOUR_GAP} {} {MIN} {ON_AVERAGE}", self.person_gap)
    }

    /// 09_6: "spread = 6² = 36 min² (model)".
    pub fn spread_row(&self) -> Row {
        Row::of(
            Role::Spread,
            vec![
                SPREAD_IS.into(),
                format!("{}{SQUARED}", self.mean_gap),
                EQUALS.into(),
                format!("{} {AREA}", self.spread),
                MODEL.into(),
            ],
        )
        .with(4, Role::Dim)
    }

    /// 09_6: "3 min + 36 min² ÷ 12 min = 6 min", the half dim, the spread's
    /// share violet, the wait green.
    pub fn numbers_row(&self) -> Row {
        Row::of(
            Role::Ink,
            vec![
                format!("{} {MIN}", self.half_mean),
                PLUS.into(),
                format!("{} {AREA} {DIVIDE} {} {MIN}", self.spread, self.twice_mean),
                format!("{EQUALS} {} {MIN}", self.mean_wait),
            ],
        )
        .with(0, Role::Dim)
        .with(2, Role::Spread)
        .with(3, Role::Verdict)
    }

    /// 09_8's scale's name.
    pub fn scale_name(&self) -> String {
        format!("{SCALE_NAME} {} {MIN} {ON_AVERAGE}", self.mean_gap)
    }

    /// K06, built from the model's numbers.
    pub fn k06(&self) -> String {
        format!(
            "{K06_MODEL} {m} {K06_MEAN} {m}, {K06_VARIANCE} {}; {K06_REST}",
            self.spread,
            m = self.mean_gap
        )
    }
}

/// Where a wait of `w` minutes sits on 09_8's scale.
pub fn scale_x(w: f64) -> f64 {
    SCALE_X0 + SCALE_U * w
}

// ----------------------------------------------------------------- picture

/// A text in a box of its color: the two as one group, and the box (whose
/// own outline a halo measures); detached.
fn boxed(kit: &Kit, stage: &mut Stage<'_>, text: &str, color: Srgb, at: Vec3) -> fmn::Result<(Mob, Mob)> {
    let words = kit.text(stage, text, BOX_SIZE, color, at)?;
    let outline = frame_around(stage, words, color)?;
    Ok((super::draw::group(stage, &[outline, words])?, outline))
}

/// An arrow from `from` to `to` in `color`, one filled shape with no
/// outline; detached.
fn arrow(stage: &mut Stage<'_>, from: Vec3, to: Vec3, color: Srgb) -> fmn::Result<Mob> {
    let a = Arrow::new(from, to).buff(0.0).thickness(ARROW_W).color(color).build()?;
    Ok(stage.arena_mut().add(a))
}

/// A thick stroke through `points`, detached.
fn bar(stage: &mut Stage<'_>, points: Vec<Vec3>, color: Srgb, w: f64) -> Mob {
    stage
        .arena_mut()
        .add(polyline(points).style(line_style(color, w)).build())
}

/// A fresh anchor for an updater, in the scene.
fn anchor(stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    let a = stage.arena_mut().add(VMobject::new());
    stage.add_to_scene(a)?;
    Ok(a)
}

/// The time since the last bus at moment `t` of the illustration: from the
/// latest bus at or before `t` (0 before the first bus; the sweep starts
/// at it).
pub fn since_last(buses: &[f64], t: f64) -> f64 {
    buses.iter().rev().find(|&&b| b <= t).map_or(0.0, |&b| t - b)
}

/// 09_1's sweep: keep the cursor at the tracker `clock`'s moment, the span
/// from the last bus to it, and the gauge's bar as long as the time since
/// that bus. Returns the updater's anchor.
fn follow(stage: &mut Stage<'_>, clock: Mob, parts: [Mob; 3], buses: Vec<f64>) -> fmn::Result<Mob> {
    let [cursor, span, gauge] = parts;
    let first = buses.first().copied().unwrap_or(0.0);
    let a = anchor(stage)?;
    stage.add_updater(
        a,
        move |s, _| {
            let t = s.tracker_value(clock).unwrap_or(first);
            let since = since_last(&buses, t).max(1e-3);
            let x = x_at(t);
            s.move_to(cursor, v(x, LINE_Y + (CURSOR_DY[0] + CURSOR_DY[1]) / 2.0), ORIGIN);
            let back = x - since * (x_at(1.0) - x_at(0.0));
            let _ = s.set_points(span, polyline(vec![v(back, LINE_Y), v(x, LINE_Y)]).build().points());
            let y = GAUGE_YS[0];
            let _ = s.set_points(
                gauge,
                polyline(vec![v(GAUGE_BAR_X, y), v(GAUGE_BAR_X + since * SINCE_U, y)])
                    .build()
                    .points(),
            );
        },
        true,
    )?;
    Ok(a)
}

impl SceneConstruct for AtRandom {
    fn name(&self) -> &str {
        "09_at_random"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let f = facts::facts().map_err(fail)?;
        let r = random(&f).map_err(fail)?;
        let formula = the_formula::formula(&f).map_err(fail)?;
        let me_t = timetable::your_moment().to_f64();
        let (buses, after) = illustration(SEED);
        fits(&buses, after, me_t).map_err(fail)?;
        let parts = pieces(&buses, after, me_t);
        let mean = RANDOM_MEAN as f64;
        let twice: f64 = r
            .twice_mean
            .parse()
            .map_err(|e| fail(format!("F-R1.twice_mean: {e}")))?;
        let (green, violet, gold) = (palette::verdict(), spread_color(), palette::long_gap());

        // 09_1: a fresh hour line; buses at random moments, an illustration;
        // the time since the last bus grows and starts again, the chance of
        // a bus stays the same
        let hour = hour_line(&kit, stage, &[])?;
        fade_in_all(stage, &hour.mobs(), 0.6)?;
        let line = say(&kit, stage, "09_1")?;
        line.wait_for(stage, "Suppose buses come completely at random")?;
        let markers: Vec<Mob> = buses
            .iter()
            .map(|&t| -> fmn::Result<Mob> {
                let m = bus_marker(stage, 0, palette::blue_bus())?;
                stage.set_x(m, x_at(t));
                Ok(m)
            })
            .collect::<fmn::Result<_>>()?;
        let label = kit.text(stage, ILLUSTRATION, SMALL, Palette::dim(), ILLUSTRATION_AT)?;
        stage.move_to(label, ILLUSTRATION_AT, RIGHT);
        fade_in_lagged(stage, &markers, 1.4, 0.15)?;
        fade_in_all(stage, &[label], 0.4)?;
        line.wait_for(stage, "At every moment")?;
        let first = buses[0];
        let clock = stage.add_value_tracker(first);
        let x0 = x_at(first);
        let cursor = stage.arena_mut().add(
            Line::new(v(x0, LINE_Y + CURSOR_DY[0]), v(x0, LINE_Y + CURSOR_DY[1]))
                .style(line_style(Palette::ink(), CURSOR_W))
                .build()?,
        );
        let span = bar(stage, vec![v(x0 - 1e-3, LINE_Y), v(x0, LINE_Y)], Palette::ink(), SPAN_W);
        let since_bar = bar(
            stage,
            vec![v(GAUGE_BAR_X, GAUGE_YS[0]), v(GAUGE_BAR_X + 1e-3, GAUGE_YS[0])],
            Palette::ink(),
            GAUGE_W,
        );
        let chance_bar = bar(
            stage,
            vec![v(GAUGE_BAR_X, GAUGE_YS[1]), v(GAUGE_BAR_X + CHANCE_LEN, GAUGE_YS[1])],
            Palette::ink(),
            GAUGE_W,
        );
        let names = [
            kit.text_left(
                stage,
                SINCE_LAST,
                GAUGE_SIZE,
                Palette::dim(),
                v(GAUGE_NAME_X, GAUGE_YS[0]),
            )?,
            kit.text_left(stage, CHANCE, GAUGE_SIZE, Palette::dim(), v(GAUGE_NAME_X, GAUGE_YS[1]))?,
        ];
        let mut gauges = vec![cursor, span, since_bar, chance_bar];
        gauges.extend(names);
        fade_in_all(stage, &gauges, 0.5)?;
        let sweeping = follow(stage, clock, [cursor, span, since_bar], buses.clone())?;
        line.wait_for(stage, "just as likely to come")?;
        let b = text_box(stage, names[1]);
        halo(stage, &[b], Palette::ink(), HALO_RT)?;
        line.wait_for(stage, "however long it's been")?;
        stage.play(steady(clock, SWEEP_TO, SWEEP_RT)?)?;
        kit.hold(stage)?;
        stage.clear_updaters(sweeping, true);

        // 09_2: the gauges go; the model keeps a bus every six minutes on
        // average; the triangles rise, a few tiny and a few huge
        let line = say(&kit, stage, "09_2")?;
        fade_out_all(stage, &gauges, 0.4)?;
        line.wait_for(stage, "a bus every six minutes")?;
        let (model_box, model_frame) = boxed(&kit, stage, &r.model_box(), Palette::dim(), MODEL_AT)?;
        fade_in_all(stage, &[model_box], 0.5)?;
        line.wait_for(stage, "some gaps are tiny")?;
        let fills: Vec<Mob> = parts
            .iter()
            .map(|p| {
                let c = if p.gap < mean { timetable::short_gap() } else { gold };
                stage
                    .arena_mut()
                    .add(Polygon::new(over_line(&p.corners)).style(flat(c, TRI_FILL)).build())
            })
            .collect();
        fade_in_lagged(stage, &fills, 2.0, 0.12)?;
        line.wait_for(stage, "the long ones hold huge triangles")?;
        let huge: Vec<Mob> = parts
            .iter()
            .zip(&fills)
            .filter(|(p, _)| p.gap >= twice)
            .map(|(_, &m)| m)
            .collect();
        pulse(stage, &huge, TRI_FILL)?;

        // 09_3: the question; think time; no wait on screen
        let line = say(&kit, stage, "09_3")?;
        line.wait_for(stage, "How long is the average wait now")?;
        let ask = kit.text(stage, ASK, ASK_SIZE, Palette::ink(), ANSWER_AT)?;
        fade_in_all(stage, &[ask], 0.4)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 09_4: six minutes, beside the model's average gap; K06
        let line = say(&kit, stage, "09_4")?;
        fade_out_all(stage, &[q, ask], 0.3)?;
        let (answer, answer_frame) = boxed(&kit, stage, &r.answer(), green, ANSWER_AT)?;
        fade_in_all(stage, &[answer], 0.4)?;
        line.wait_for(stage, "The whole average gap")?;
        let both = [text_box(stage, model_frame), text_box(stage, answer_frame)];
        halo(stage, &both, Palette::ink(), HALO_RT)?;
        line.wait_for(stage, "not half of it")?;
        let k06_text = r.k06();
        let k06 = precisely(&kit, stage, &k06_text)?;
        fade_in_all(stage, &[k06], 0.5)?;
        let card_up = now(stage);
        kit.hold(stage)?;
        wait_until(stage, card_up + read_s(&k06_text))?;

        // 09_5: K06 goes; you, at your moment; the past dims; the wait
        // ahead of you against a new gap from a bus, the same on average
        let line = say(&kit, stage, "09_5")?;
        fade_out_all(stage, &[k06], 0.4)?;
        line.wait_for(stage, "So from the moment you turn up")?;
        let x_me = x_at(me_t);
        let me = you(stage, v(x_me, LINE_Y))?;
        let drop = bar(
            stage,
            vec![v(x_me, LINE_Y - 0.2), v(x_me, ROW_A_Y + 0.1)],
            Palette::faint(),
            2.0,
        );
        fade_in_all(stage, &[me, drop], 0.4)?;
        let mut dimming = Vec::new();
        for (p, &m) in parts.iter().zip(&fills) {
            if p.behind {
                dimming.push(fade_to(m, DIM_FILL, 0.6)?);
            }
        }
        for (&t, &m) in buses.iter().zip(&markers) {
            if t < me_t {
                dimming.push(fade_to(m, DIM_MARKER, 0.6)?);
            }
        }
        play_all(stage, dimming)?;
        line.wait_for(stage, "the wait ahead of you")?;
        let reach = mean * DIAGRAM_U;
        let ahead_arrow = arrow(stage, v(x_me, ROW_A_Y), v(x_me + reach, ROW_A_Y), green)?;
        let ahead = kit.text_left(
            stage,
            &r.ahead(),
            LABEL_SIZE,
            green,
            v(x_me + reach + LABEL_DX, ROW_A_Y),
        )?;
        fade_in_all(stage, &[ahead_arrow, ahead], 0.5)?;
        line.wait_for(stage, "just like a brand new gap")?;
        let [w, h] = BUS_MARK;
        let new_bus = stage.arena_mut().add(rect(
            v(x_me - w / 2.0, ROW_B_Y - h / 2.0),
            w,
            h,
            flat(palette::blue_bus(), 1.0),
        )?);
        let fresh_arrow = arrow(stage, v(x_me, ROW_B_Y), v(x_me + reach, ROW_B_Y), Palette::ink())?;
        let fresh = kit.text_left(
            stage,
            &r.fresh(),
            LABEL_SIZE,
            Palette::ink(),
            v(x_me + reach + LABEL_DX, ROW_B_Y),
        )?;
        fade_in_all(stage, &[new_bus, fresh_arrow, fresh], 0.5)?;
        line.wait_for(stage, "a gap averages six minutes")?;
        let labels = [text_box(stage, ahead), text_box(stage, fresh)];
        halo(stage, &labels, Palette::ink(), HALO_RT)?;

        // 09_6: chapter 08's formula, dim, then the model's numbers: the
        // spread, and three minutes plus thirty-six over twelve
        let line = say(&kit, stage, "09_6")?;
        line.wait_for(stage, "The formula agrees")?;
        let letters = show_row(
            &kit,
            stage,
            &Row::of(Role::Dim, vec![formula.formula_row().text()]),
            v(ROWS_X, ROW_YS[0]),
            FORMULA_SIZE,
        )?;
        fade_in_all(stage, &[letters.mob], 0.5)?;
        line.wait_for(stage, "the spread is the average gap squared")?;
        let spread_row = show_row(&kit, stage, &r.spread_row(), v(ROWS_X, ROW_YS[1]), ROW_SIZE)?;
        fade_in_all(stage, &[spread_row.mob], 0.5)?;
        line.wait_for(stage, "Three minutes, plus thirty-six over twelve")?;
        let numbers = show_row(&kit, stage, &r.numbers_row(), v(ROWS_X, ROW_YS[2]), ROW_SIZE)?;
        fade_in_all(stage, &[numbers.mob], 0.5)?;
        line.wait_for(stage, "Six")?;
        let verdict = frame_around(stage, numbers.mob, green)?;
        fade_in_all(stage, &[verdict], 0.4)?;
        kit.music_breath(stage, VERDICT_LOOK_S)?;

        // 09_7: before the line, the rows, the new gap and the drop from
        // you go (the time back is labelled across it); back from you, the
        // last bus six minutes before on average; your gap, twelve; the
        // names
        fade_out_all(
            stage,
            &[
                letters.mob,
                spread_row.mob,
                numbers.mob,
                verdict,
                new_bus,
                fresh_arrow,
                fresh,
                drop,
            ],
            0.4,
        )?;
        let line = say(&kit, stage, "09_7")?;
        line.wait_for(stage, "Look back too")?;
        let back_arrow = arrow(stage, v(x_me, ROW_A_Y), v(x_me - reach, ROW_A_Y), Palette::dim())?;
        let since = kit.text(
            stage,
            &r.since(),
            SMALL,
            Palette::dim(),
            v(x_me - reach / 2.0, ROW_A_Y + BACK_LABEL_DY),
        )?;
        fade_in_all(stage, &[back_arrow, since], 0.5)?;
        // the gap is marked as it is named, and its length comes with the
        // spoken twelve, not before
        line.wait_for(stage, "So the gap you land in")?;
        let y = ROW_A_Y + BRACKET_DY;
        let (a, b) = (x_me - reach, x_me + reach);
        let bracket = bar(
            stage,
            vec![v(a, y + BRACKET_TICK), v(a, y), v(b, y), v(b, y + BRACKET_TICK)],
            gold,
            3.0,
        );
        fade_in_all(stage, &[bracket], 0.4)?;
        line.wait_for(stage, "averages twelve minutes")?;
        let your_gap = kit.text(
            stage,
            &r.your_gap(),
            LABEL_SIZE,
            gold,
            v(x_me, ROW_A_Y + BRACKET_LABEL_DY),
        )?;
        fade_in_all(stage, &[your_gap], 0.4)?;
        kit.hold(stage)?;
        let names = names_to_search(&kit, stage, &NAMES)?;
        fade_in_all(stage, &[names], 0.5)?;
        let names_up = now(stage);
        wait_until(stage, names_up + read_s(&NAMES.join(" · ")))?;

        // 09_8: before the line (its first words are the first mark), the
        // comparison goes, the picture dims and the empty scale of average
        // waits for a bus every six minutes on average comes in; each mark
        // as it is said; more spread; past six; K07
        let leaving = [
            names,
            me,
            ahead_arrow,
            ahead,
            back_arrow,
            since,
            bracket,
            your_gap,
            model_box,
            answer,
        ];
        fade_out_all(stage, &leaving, 0.4)?;
        let mut dimming = Vec::new();
        for (p, &m) in parts.iter().zip(&fills) {
            if !p.behind {
                dimming.push(fade_to(m, DIM_FILL, 0.6)?);
            }
        }
        for (&t, &m) in buses.iter().zip(&markers) {
            if t >= me_t {
                dimming.push(fade_to(m, DIM_MARKER, 0.6)?);
            }
        }
        play_all(stage, dimming)?;
        let axis = arrow(
            stage,
            v(scale_x(0.0), SCALE_Y),
            v(scale_x(SCALE_MAX) + SCALE_TIP, SCALE_Y),
            Palette::dim(),
        )?;
        let scale_name = kit.text_left(stage, &r.scale_name(), MARK_SIZE, Palette::dim(), SCALE_NAME_AT)?;
        fade_in_all(stage, &[axis, scale_name], 0.5)?;
        let line = say(&kit, stage, "09_8")?;
        let colors = [Palette::dim(), Palette::ink(), green, Palette::ink()];
        let phrases = [
            "Even gaps wait three minutes",
            "random buses wait six",
            "our timetable sits at four twenty",
            "even past six",
        ];
        for (i, phrase) in phrases.into_iter().enumerate() {
            if i == 3 {
                line.wait_for(stage, "More spread means a longer wait")?;
                let from = scale_x(r.marks[0].0.to_f64());
                let more = arrow(
                    stage,
                    v(from, SPREAD_ARROW_Y),
                    v(scale_x(SPREAD_ARROW_END), SPREAD_ARROW_Y),
                    violet,
                )?;
                let middle = (from + scale_x(SPREAD_ARROW_END)) / 2.0;
                let more_label = kit.text(
                    stage,
                    MORE_SPREAD,
                    MARK_SIZE,
                    violet,
                    v(middle, SPREAD_ARROW_Y + SPREAD_LABEL_DY),
                )?;
                fade_in_all(stage, &[more, more_label], 0.5)?;
            }
            line.wait_for(stage, phrase)?;
            let (w, text) = &r.marks[i];
            let x = scale_x(w.to_f64());
            let tick = bar(
                stage,
                vec![v(x, SCALE_Y - MARK_TICK), v(x, SCALE_Y + MARK_TICK)],
                colors[i],
                3.0,
            );
            let dy = if i < 2 { MARK_DY[0] } else { MARK_DY[1] };
            let mark = kit.text(stage, text, MARK_SIZE, colors[i], v(x, SCALE_Y + dy))?;
            fade_in_all(stage, &[tick, mark], 0.4)?;
        }
        line.wait_for(stage, "The formula needs only")?;
        let k07 = precisely(&kit, stage, K07)?;
        fade_in_all(stage, &[k07], 0.5)?;
        let card_up = now(stage);
        kit.hold(stage)?;
        wait_until(stage, card_up + read_s(K07))?;
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

    /// The full draft with its line breaks and indents as single spaces.
    fn draft() -> String {
        crate::DRAFT.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// A copy of the facts with `key` printed as `bad`.
    fn with_fact(f: &Facts, key: &str, bad: &str) -> Facts {
        let mut wrong = Facts::new();
        for row in f.print().lines() {
            let (k, value) = row.split_once('\t').unwrap();
            wrong.add(k, if k == key { bad } else { value }).unwrap();
        }
        wrong
    }

    /// Chapter 08's formula in letters, as 09_6 shows it again.
    fn the_formula_letters() -> String {
        the_formula::formula(&facts::facts().unwrap())
            .unwrap()
            .formula_row()
            .text()
    }

    /// Every text on screen before 09_4 answers the question, in order.
    fn before_the_answer(r: &Random) -> Vec<String> {
        vec![
            ILLUSTRATION.into(),
            SINCE_LAST.into(),
            CHANCE.into(),
            r.model_box(),
            ASK.into(),
        ]
    }

    /// Every text on screen before 09_7 says how long your gap is, in order
    /// (09_4 to 09_6 added to those before the answer).
    fn before_your_gap(r: &Random) -> Vec<String> {
        let mut out = before_the_answer(r);
        out.extend([r.answer(), r.k06(), r.ahead(), r.fresh()]);
        out.push(the_formula_letters());
        out.push(r.spread_row().text());
        out.push(r.numbers_row().text());
        out
    }

    /// Needs chapter 09's lines in script.tsv (the integration step copies
    /// them from the draft). The whole chapter runs headless at 10 fps with
    /// the scratch voice: every phrase it waits for is in its line, the
    /// numbers match the facts, the illustration fits, and its eight lines
    /// are said in order. It ends with `kit.finish`, whose timeline runs
    /// past the end of the last line.
    #[test]
    fn the_chapter_runs_and_says_its_lines_in_order() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let narrator = Narrator::estimate(&script);
        let kit = Kit::new().unwrap().with_narrator(Rc::clone(&narrator));
        let mut scene = AtRandom { kit };
        let config = fmn::scene::RuntimeConfig {
            fps: 10,
            ..fmn::scene::RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut fmn::scene::NullSceneSink).unwrap();
        let said_at = narrator.said();
        let said: Vec<&str> = said_at.iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(said, ["09_1", "09_2", "09_3", "09_4", "09_5", "09_6", "09_7", "09_8"]);
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

    /// K06 (built from the model's numbers) and K07 are the draft's exact
    /// versions word for word, and what the chapter prints is the draft's
    /// picture: the label, the model's box, the question, the answer, the
    /// new gap, your gap, the three marks the draft names, "more spread"
    /// and the names to search. By the kit's wrap at 92 characters each
    /// card is two lines.
    #[test]
    fn the_printed_words_are_the_drafts() {
        let board = draft();
        let r = random(&facts::facts().unwrap()).unwrap();
        for card in [r.k06(), K07.to_owned()] {
            assert!(board.contains(&format!("\"{card}\"")), "{card}");
            assert_eq!(explainer_kit::animatic::wrap(&card, 92, 3).len(), 2, "{card}");
        }
        let mut shown = vec![
            ILLUSTRATION.to_owned(),
            r.model_box(),
            ASK.to_owned(),
            r.answer(),
            r.fresh(),
            r.your_gap(),
            MORE_SPREAD.to_owned(),
        ];
        shown.extend(r.marks[..3].iter().map(|(_, s)| s.clone()));
        for s in shown {
            assert!(board.contains(&format!("\"{s}\"")), "not in the draft: {s}");
        }
        assert!(board.contains(&format!("names to search: {}", NAMES.join(", "))));
        assert_eq!(r.k06().matches("Poisson").count(), 1);
    }

    /// The numbers are the model's, and a fact that stops matching it is
    /// refused: the random spread, the random wait, your gap, the clumped
    /// wait and the clumped gaps (still a six-minute average), each changed.
    #[test]
    fn the_numbers_are_the_models_and_a_wrong_fact_is_refused() {
        let f = facts::facts().unwrap();
        let r = random(&f).unwrap();
        assert_eq!((r.mean_gap.as_str(), r.spread.as_str()), ("6", "36"));
        assert_eq!((r.half_mean.as_str(), r.twice_mean.as_str()), ("3", "12"));
        assert_eq!((r.mean_wait.as_str(), r.mean_age.as_str()), ("6", "6"));
        assert_eq!(r.person_gap, "12");
        let labels: Vec<&str> = r.marks.iter().map(|(_, s)| s.as_str()).collect();
        assert_eq!(
            labels,
            [
                "even gaps: 3 min",
                "completely random: 6 min",
                "our timetable: 4 min 20 s",
                "gaps 0, 0, 18 (average 6): 9 min",
            ]
        );
        assert_eq!(r.spread_row().text(), "spread = 6² = 36 min² (model)");
        assert_eq!(r.numbers_row().text(), "3 min + 36 min² ÷ 12 min = 6 min");
        for row in [r.spread_row(), r.numbers_row()] {
            for (piece, _) in row.accents() {
                assert_eq!(row.text().matches(piece).count(), 1, "{piece}");
            }
        }
        for (key, bad) in [
            ("F-R1.spread", "30"),
            ("F-R1.mean_wait", "3"),
            ("F-R1.person_gap", "6"),
            ("F-R2.mean_wait", "8"),
            ("F-R2.gaps", "0,6,12"),
        ] {
            assert!(
                random(&with_fact(&f, key, bad)).is_err(),
                "a wrong {key} must be refused"
            );
        }
    }

    /// The illustration is the seed's, fixed: nine buses, the first at 4.39
    /// min, the next after the hour at 60.62 min, as the Python mirror drew
    /// it; it fits; the seeds before it do not (seed 271 fits all but the
    /// last rule: you would land 1.29 min after a bus in a gap of 4.77 min).
    /// Its own average gap is not the model's six (it is a sample), so
    /// nothing may print it as the model's.
    #[test]
    fn the_illustration_is_fixed_and_fits() {
        let me = timetable::your_moment().to_f64();
        let (buses, after) = illustration(SEED);
        assert_eq!(buses, [4.39, 15.43, 16.53, 29.25, 31.18, 43.53, 45.74, 50.58, 56.61]);
        assert_eq!(after, 60.62);
        assert!(fits(&buses, after, me).is_ok());
        for seed in 1..SEED {
            let (b, a) = illustration(seed);
            assert!(fits(&b, a, me).is_err(), "seed {seed} fits too");
        }
        let (b, a) = illustration(271);
        assert!(fits(&b, a, me).unwrap_err().contains("shorter than average"));
        let sample = (buses[buses.len() - 1] - buses[0]) / (buses.len() - 1) as f64;
        assert!((sample - RANDOM_MEAN as f64).abs() > 0.1, "{sample}");
        // a different stream is a different picture
        assert_ne!(illustration(SEED + 1).0, buses);
    }

    /// The pieces draw the wait at every moment of the hour: each piece's
    /// top edge is the wait there, the pieces meet without gaps or overlap,
    /// their area is the hour's waiting, the gap holding your moment is
    /// split there with the part behind you marked, and nothing rises past
    /// the screen's top.
    #[test]
    fn the_pieces_draw_the_wait() {
        let me = timetable::your_moment().to_f64();
        let (buses, after) = illustration(SEED);
        let ps = pieces(&buses, after, me);
        assert_eq!(ps.len(), buses.len() + 2);
        let mut edge = 0.0;
        let mut area = 0.0;
        for p in &ps {
            let c = &p.corners;
            assert!((c[0].0 - edge).abs() < 1e-12 && c[0].1 == 0.0, "{p:?}");
            edge = c[c.len() - 1].0;
            // the top edge: the wait just after its left end, and (a cut
            // piece) just before its right end
            let top = &c[1..c.len() - 1];
            assert!(
                (top[0].1 - wait_at(&buses, after, top[0].0 + 1e-9)).abs() < 1e-6,
                "{p:?}"
            );
            if let [_, (t, w)] = top {
                assert!((w - wait_at(&buses, after, t - 1e-9)).abs() < 1e-6, "{p:?}");
            }
            assert!(LINE_Y + c[1].1 * MINUTE_UP < 4.0, "{p:?}");
            let k = c.len();
            area += (0..k)
                .map(|i| c[i].0 * c[(i + 1) % k].1 - c[(i + 1) % k].0 * c[i].1)
                .sum::<f64>()
                .abs()
                / 2.0;
            let right = c[c.len() - 1].0;
            assert_eq!(p.behind, right <= me + 1e-12, "{p:?}");
        }
        assert!((edge - HOUR as f64).abs() < 1e-12);
        let steps = 600_000;
        let dt = HOUR as f64 / steps as f64;
        let waiting: f64 = (0..steps)
            .map(|i| wait_at(&buses, after, (i as f64 + 0.5) * dt) * dt)
            .sum();
        assert!((area - waiting).abs() < 1e-3, "{area} {waiting}");
        assert_eq!(ps.iter().filter(|p| p.corners.iter().any(|&(t, _)| t == me)).count(), 2);
    }

    /// No memory, in the model: the wait ahead is a new gap's average, the
    /// time back the same, and your gap is both, twice the average gap;
    /// chapter 08's formula with the exponential's spread gives the same
    /// wait. A timetable has memory: with our gaps the wait ahead is 4⅓
    /// min, not a new gap's 6, so the comparison is the model's, not any
    /// picture's.
    #[test]
    fn no_memory_and_the_formula_agree() {
        let (m1, m2) = model::exponential_moments(RANDOM_MEAN);
        let spread = m2 - m1 * m1;
        let two = Q::int(2);
        let wait = m1 / two + spread / (two * m1);
        assert_eq!(wait, m1);
        assert_eq!(m2 / m1, wait + wait);
        assert_eq!(m2 / m1, two * m1);
        let ours = model::hour_gaps(OFFSET);
        assert_ne!(model::mean_wait(&ours), model::mean_gap(&ours));
        assert_eq!(model::mean_wait(&ours), Q::new(13, 3));
    }

    /// The scale's marks run in order of spread and of wait, from even gaps
    /// to gaps of 0, 0 and 18; the scale runs past its last mark; no label
    /// calls any end a worst case or a maximum.
    #[test]
    fn the_scale_runs_on_past_six() {
        let r = random(&facts::facts().unwrap()).unwrap();
        let mut by_place: Vec<Q> = r.marks.iter().map(|(w, _)| *w).collect();
        let spoken = by_place.clone();
        by_place.sort();
        assert_eq!(by_place, [spoken[0], spoken[2], spoken[1], spoken[3]]);
        assert!(spoken[3] > spoken[1], "a timetable waits longer than random buses");
        assert!(scale_x(SCALE_MAX) > scale_x(spoken[3].to_f64()));
        for (_, s) in &r.marks {
            for word in ["worst", "max", "most", "limit"] {
                assert!(!s.contains(word), "{s}");
            }
        }
        assert!(!r.scale_name().contains("max"));
    }

    /// Nothing on screen gives the answer before 09_4 or your gap before
    /// 09_7: before the answer no "average wait:", and the only "6 min" is
    /// the model's average gap, given; before your gap no "your gap" and no
    /// "12 min on average". (09_6's "12 min" is the formula's twice the
    /// average gap, as the voice says it.)
    #[test]
    fn nothing_is_revealed_before_its_line() {
        let r = random(&facts::facts().unwrap()).unwrap();
        let six = format!("{} {MIN}", r.mean_wait);
        for s in before_the_answer(&r) {
            assert!(!s.contains(AVERAGE_WAIT), "{s}");
            assert!(!s.contains(&six) || s == r.model_box(), "{s}");
        }
        let twelve = format!("{} {MIN} {ON_AVERAGE}", r.person_gap);
        for s in before_your_gap(&r) {
            assert!(!s.contains(YOUR_GAP) && !s.contains(&twelve), "{s}");
        }
        assert!(before_your_gap(&r).contains(&r.answer()));
        assert!(r.your_gap().contains(&twelve));
    }

    /// The since-last clock starts again at every bus and grows a minute a
    /// minute between them.
    #[test]
    fn the_time_since_the_last_bus_starts_again_at_each_bus() {
        let (buses, _) = illustration(SEED);
        for &b in &buses {
            assert!(since_last(&buses, b).abs() < 1e-12);
            assert!((since_last(&buses, b + 0.5) - 0.5).abs() < 1e-9);
        }
        let first = buses[0];
        assert!(since_last(&buses, first + 0.25) > 0.0);
    }

    /// The texts up together stay apart and on screen: the gauges, the
    /// boxes, the question and its ring, the comparison rows, the formula
    /// rows, the scale's labels and the lower-thirds. Widths are estimated,
    /// not measured: 0.0045 units a point a character, a line 0.6 of its
    /// size tall.
    #[test]
    fn the_picture_stays_apart_and_on_screen() {
        let r = random(&facts::facts().unwrap()).unwrap();
        let formula = the_formula_letters();
        let me = x_at(timetable::your_moment().to_f64());
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
            (b.0 - 0.2, b.1 + 0.2, b.2 - 0.15, b.3 + 0.15)
        };
        let apart = |a: Box4, b: Box4| a.1 < b.0 || b.1 < a.0 || a.3 < b.2 || b.3 < a.2;
        let on_screen = |a: Box4| a.0 > -7.1 && a.1 < 7.1 && a.2 > -4.0 && a.3 < 4.0;
        let band = |s: &str| {
            let lines = explainer_kit::animatic::wrap(s, 92, 3).len() as f64;
            (-6.71, 6.71, -3.8, -3.8 + 0.3 + 0.36 * lines)
        };
        let label = {
            let b = left(ILLUSTRATION, SMALL, ILLUSTRATION_AT);
            (b.0 - w(ILLUSTRATION, SMALL), b.1 - w(ILLUSTRATION, SMALL), b.2, b.3)
        };
        // the line and its markers, and the triangles over them
        let line_row = (LINE_X0, LINE_X1, LINE_Y - 0.05, LINE_Y + 0.46);
        let gauges = vec![
            left(SINCE_LAST, GAUGE_SIZE, v(GAUGE_NAME_X, GAUGE_YS[0])),
            left(CHANCE, GAUGE_SIZE, v(GAUGE_NAME_X, GAUGE_YS[1])),
            (
                GAUGE_BAR_X,
                GAUGE_BAR_X + LONG.max(TALLEST) * SINCE_U,
                GAUGE_YS[0] - 0.05,
                GAUGE_YS[0] + 0.05,
            ),
            (
                GAUGE_BAR_X,
                GAUGE_BAR_X + CHANCE_LEN,
                GAUGE_YS[1] - 0.05,
                GAUGE_YS[1] + 0.05,
            ),
        ];
        let model_box = boxed(&r.model_box(), MODEL_AT);
        let ask = centred(ASK, ASK_SIZE, ANSWER_AT);
        let ring = (
            THINK_AT[0] - 0.62,
            THINK_AT[0] + 0.62,
            THINK_AT[1] - 0.62,
            THINK_AT[1] + 0.62,
        );
        let note = centred("pause for more time", 18.0, v(THINK_AT[0], THINK_AT[1] - 0.62 - 0.3));
        let answer = boxed(&r.answer(), ANSWER_AT);
        let reach = RANDOM_MEAN as f64 * DIAGRAM_U;
        let ahead = left(&r.ahead(), LABEL_SIZE, v(me + reach + LABEL_DX, ROW_A_Y));
        let fresh = left(&r.fresh(), LABEL_SIZE, v(me + reach + LABEL_DX, ROW_B_Y));
        // row A's arrow ahead (from your drop line), then in 09_7 with the
        // arrow back, both meeting at your moment: one row
        let row_a = (me - 0.1, me + reach, ROW_A_Y - 0.08, ROW_A_Y + 0.08);
        let row_b = (me - 0.1, me + reach, ROW_B_Y - 0.12, ROW_B_Y + 0.12);
        let both_ways = (me - reach, me + reach, ROW_A_Y - 0.08, ROW_A_Y + 0.08);
        let since = centred(&r.since(), SMALL, v(me - reach / 2.0, ROW_A_Y + BACK_LABEL_DY));
        let bracket = (
            me - reach,
            me + reach,
            ROW_A_Y + BRACKET_DY,
            ROW_A_Y + BRACKET_DY + BRACKET_TICK,
        );
        let your_gap = centred(&r.your_gap(), LABEL_SIZE, v(me, ROW_A_Y + BRACKET_LABEL_DY));
        let rows = vec![
            left(&formula, FORMULA_SIZE, v(ROWS_X, ROW_YS[0])),
            left(&r.spread_row().text(), ROW_SIZE, v(ROWS_X, ROW_YS[1])),
            {
                let b = left(&r.numbers_row().text(), ROW_SIZE, v(ROWS_X, ROW_YS[2]));
                (b.0 - 0.2, b.1 + 0.2, b.2 - 0.15, b.3 + 0.15)
            },
        ];
        let scale = (
            scale_x(0.0),
            scale_x(SCALE_MAX) + SCALE_TIP,
            SCALE_Y - 0.12,
            SCALE_Y + 0.12,
        );
        let scale_name = left(&r.scale_name(), MARK_SIZE, SCALE_NAME_AT);
        let from = scale_x(r.marks[0].0.to_f64());
        let more = (
            from,
            scale_x(SPREAD_ARROW_END),
            SPREAD_ARROW_Y - 0.08,
            SPREAD_ARROW_Y + 0.08,
        );
        let more_label = centred(
            MORE_SPREAD,
            MARK_SIZE,
            v(
                (from + scale_x(SPREAD_ARROW_END)) / 2.0,
                SPREAD_ARROW_Y + SPREAD_LABEL_DY,
            ),
        );
        let marks: Vec<Box4> = r
            .marks
            .iter()
            .enumerate()
            .map(|(i, (q, s))| {
                let dy = if i < 2 { MARK_DY[0] } else { MARK_DY[1] };
                centred(s, MARK_SIZE, v(scale_x(q.to_f64()), SCALE_Y + dy))
            })
            .collect();
        let moments: Vec<Vec<Box4>> = vec![
            // 09_1
            [vec![line_row, label], gauges].concat(),
            // 09_3
            vec![line_row, label, model_box, ask, ring, note],
            // 09_4: K06
            vec![line_row, label, model_box, answer, band(&r.k06())],
            // 09_5 and 09_6
            [
                vec![line_row, label, model_box, answer, row_a, row_b, ahead, fresh],
                rows.clone(),
            ]
            .concat(),
            // 09_7
            vec![
                line_row,
                label,
                model_box,
                answer,
                both_ways,
                ahead,
                since,
                bracket,
                your_gap,
                band(&NAMES.join(" · ")),
            ],
            // 09_8
            [
                vec![line_row, label, scale, scale_name, more, more_label, band(K07)],
                marks,
            ]
            .concat(),
        ];
        for (k, up) in moments.iter().enumerate() {
            for (i, &a) in up.iter().enumerate() {
                assert!(on_screen(a), "moment {k}: item {i} off screen: {a:?}");
                for (j, &b) in up.iter().enumerate().skip(i + 1) {
                    assert!(apart(a, b), "moment {k}: items {i} and {j} overlap: {a:?} {b:?}");
                }
            }
        }
    }

    /// The chapter's fixed words carry no digits: every number on screen is
    /// read from the facts.
    #[test]
    fn the_fixed_words_carry_no_numbers() {
        for w in [
            ILLUSTRATION,
            SINCE_LAST,
            CHANCE,
            AVERAGE_GAP,
            MODEL,
            ASK,
            AVERAGE_WAIT,
            AHEAD,
            FRESH,
            ON_AVERAGE,
            SINCE,
            YOUR_GAP,
            SPREAD_IS,
            AREA,
            SQUARED,
            PLUS,
            DIVIDE,
            EQUALS,
            SCALE_NAME,
            EVEN,
            RANDOM,
            OURS,
            GAPS,
            AVERAGE,
            MORE_SPREAD,
            NAMES[0],
            NAMES[1],
            K06_MODEL,
            K06_MEAN,
            K06_VARIANCE,
            K06_REST,
            K07,
        ] {
            assert!(!w.chars().any(|c| c.is_ascii_digit()), "{w}");
        }
    }
}
