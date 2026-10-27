//! Chapter 12, one more timetable. The film's last puzzle, worked with the
//! film's own picture, then the train:
//! - 12_1: chapter 11's planners, rebuilt in their stacks as 11_7 left them,
//!   slide back to the moments they were ready: arrivals at random again,
//!   one a minute, spread evenly over the hour.
//! - 12_2: the red bus slides to four minutes after the blue; gaps of four
//!   and eight. The average wait? (a prediction; no triangle, no area and no
//!   wait on screen.)
//! - 12_3: the triangles rise; each gap holds half its length squared, in
//!   square minutes.
//! - 12_4: forty square minutes over the twelve minutes they cover; square
//!   minutes over minutes leave minutes: 3⅓ minutes, three minutes twenty,
//!   boxed, and the step held until it can be read.
//! - 12_5: chapter 07's U, with the puzzle's point at four minutes between
//!   our timetable's and the even one's.
//! - 12_6: the train: chapter 01's street strip; back on our timetable, the
//!   triangles reshaping with the red bus; four minutes twenty, the average
//!   cost.
//! - 12_7: a deadline needs the longest wait: you, just as a red bus leaves,
//!   and chapter 01's wait bar over the whole long gap: allow ten minutes.
//! - 12_8: only with every bus exactly on time; the exact version (K10).
//! - 12_9: your bus isn't late; the names to search, then the receipts on a
//!   card alone, each held until it can be read.
//!
//! The street and its wait bar are chapter 01's drawing and the plot is
//! chapter 07's (`the_street_is_chapter_01s` and `the_plot_is_chapter_07s`
//! check the copies). One tracker, the red bus's offset, moves the red
//! markers and repaints the passengers, and the triangles follow it back to
//! our timetable. Every number on screen is read from the facts, and `last`
//! refuses a fact that stops matching the model.

use std::cmp::Ordering;

use explainer_kit::props::{names_to_search, precisely, receipts_card};
use explainer_kit::{Facts, Kit, Palette, Timed, fade_all, live_bar, rect, tween, v};
use fmn::prelude::*;

use super::back_to_the_stop::{self, slide_at};
use super::draw::{
    LINE_Y, PASSENGER_R, STAND_UP, fade_in_all, fade_in_lagged, fade_out_all, fade_to, fail, flash_all, flash_lagged,
    flat, group, line_style, marker_top, min_s, now, pause, read_s, say, shrink_to, think, wait_until, x_of, you,
};
use super::no_bus_late::{curve, minute_up, plot_at, red_x, triangles_at, wait_of};
use super::timetable::{self, Gap, fade_in_with, fade_out_with, gap_color, x_at};
use super::twelve_minutes::grow_lagged;
use super::two_averages::written;
use crate::facts;
use crate::model::{self, HOUR, OFFSET, PER_MINUTE, PERIOD, PUZZLE_OFFSET, Q};
use crate::palette;

pub struct OneMoreTimetable {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are read from the facts)
const HOME: &str = "home";
const STOP: &str = "bus stop";
const STATION: &str = "station";
const ON_TIME: &str = "every bus on time";
const AT_RANDOM: &str = "at random: no look at the timetable, spread evenly over the hour";
const ASK: &str = "average wait?";
const AREA: &str = "min²";
const MIN: &str = "min";
const SQUARED: &str = "²";
const ARROW: &str = "→";
const TIMES: &str = "×";
const PLUS: &str = "+";
const EQUALS: &str = "=";
const DIVIDED: &str = "÷";
const AVERAGE_WAIT: &str = "average wait";
const SLIDER: &str = "red bus after blue (min)";
const AVERAGE_COST: &str = "average cost:";
const ALLOW: &str = "allow";
/// The exact version of 12_8's claim (K10), around the allowance.
const K10_HEAD: &str = "with every bus exactly on time, the longest wait is the longest gap,";
const K10_TAIL: &str = ", whenever you turn up; it says nothing about late buses or the rest of the journey";
const NAMES: [&str; 5] = [
    "inspection paradox",
    "random incidence",
    "length-biased sampling",
    "renewal–reward theorem",
    "class size paradox",
];
/// The names' band's label, `props::names_to_search`'s, read with them.
const NAMES_LABEL: &str = "names to search:";
const RECEIPTS: [&str; 4] = [
    "Every number in this video comes from the code",
    "and is checked against an independent oracle",
    "github.com/Dicklesworthstone/jeffs_math_explainer_videos",
    "explainer_corpus/inspection_paradox",
];
/// The receipts card's title, drawn by `props::receipts_card`, read with
/// its lines.
const RECEIPTS_TITLE: &str = "Receipts";

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
const ON_TIME_SIZE: f64 = 20.0;
/// 01_4's wait bar under the line, its label under it.
const BAR_Y: f64 = LINE_Y - 0.78;
const BAR_H: f64 = 0.16;
const BAR_LABEL_DY: f64 = -0.32;
const BAR_RT: f64 = 1.2;
const ALLOW_SIZE: f64 = 20.0;
/// 12_1: the slide back, as long as 11's slide forward, and the caption
/// under the passengers.
const UNSTACK_RT: f64 = 3.5;
const CAPTION_AT: Vec3 = [0.0, -1.0, 0.0];
const CAPTION_SIZE: f64 = 20.0;
/// 12_2: the red bus's slide, the question and the think ring, under the
/// caption.
const SLIDE_RT: f64 = 1.6;
const ASK_AT: Vec3 = [1.2, -1.7, 0.0];
const ASK_SIZE: f64 = 24.0;
const THINK_AT: Vec3 = [4.6, -1.9, 0.0];
/// The triangles over the hour, as chapter 11 draws them: a minute up as
/// long as a minute across, see-through so the markers show, behind them
/// (set before they join the scene: a z-index set later does not re-sort).
const TRI_FILL: f64 = 0.35;
const TRI_BRIGHT: f64 = 0.6;
const TRI_DIM: f64 = 0.12;
const TRI_Z: i32 = -1;
const GROW_RT: f64 = 2.0;
/// 12_3, 12_4: the rows at the top, centred, over the tallest triangle; the
/// steps lower left, centred under each other; the stretch's bracket under
/// the line.
const ROW_X: f64 = 1.0;
const ROW_YS: [f64; 3] = [3.55, 3.05, 2.55];
const ROW_SIZE: f64 = 24.0;
const STEP_X: f64 = -3.4;
const STEP_YS: [f64; 4] = [-1.45, -1.95, -2.45, -3.05];
const STEP_SIZE: f64 = 24.0;
const UNIT_SIZE: f64 = 20.0;
const BOX_PAD: [f64; 2] = [0.2, 0.15];
const BOX_W: f64 = 2.0;
const STRETCH_Y: f64 = 0.0;
const STRETCH_TICK: f64 = 0.1;
const STRETCH_INSET: f64 = 0.04;
const STRETCH_LABEL_Y: f64 = -0.22;
const STRETCH_SIZE: f64 = 20.0;
/// After 12_4's line and its breath, the step stands at least this much
/// longer with no voice, and until it can be read.
const INSPECT_S: f64 = 2.0;
/// Then chapter 07's plot comes back, still with no voice, before 12_5
/// names it.
const CALLBACK_RT: f64 = 0.8;
/// Chapter 07's plot, at its place and scale (`no_bus_late.rs`).
const PLOT_O: Vec3 = [1.2, -3.3, 0.0];
const PLOT_SIZE: f64 = 16.0;
const PLOT_TICK_DY: f64 = 0.22;
const PLOT_NAME_DY: f64 = 0.41;
const CURVE_W: f64 = 4.0;
const KNOB_R: f64 = 0.08;
const POINT_R: f64 = 0.07;
/// 07_6's labels beside its points.
const MARK_SIZE: f64 = 18.0;
const MARK_D2: [f64; 2] = [0.3, 0.22];
const MARK_D6_DY: f64 = -0.25;
const MARK_D0_DX: f64 = 0.12;
/// 12_6: the average cost, lower right; 12_7: you, out of the strip.
const COST_AT: Vec3 = [3.6, -1.45, 0.0];
const COST_SIZE: f64 = 24.0;
const YOU_RT: f64 = 1.2;
/// Emphasis: an outline around a text, faded in, held, faded out.
const HALO_PAD: f64 = 0.1;
const HALO_W: f64 = 3.0;
const HALO_RT: f64 = 0.9;

// ------------------------------------------------------------- the numbers

/// A fact's printed fraction ("10/3", "4") as a number.
fn parse_q(key: &str, printed: &str) -> Result<Q, String> {
    let int = |s: &str| s.parse::<i64>().map_err(|e| format!("{key} = {printed}: {e}"));
    match printed.split_once('/') {
        Some((n, d)) => Ok(Q::new(int(n)?, int(d)?)),
        None => Ok(Q::int(int(printed)?)),
    }
}

/// Whether the average waits over the whole offsets (`waits[d]`, the red
/// bus `d` minutes after the blue) come nearer their least wherever the
/// gaps are more even: an offset nearer `even` waits less, and two offsets
/// as near wait the same. 12_5's "the more even the gaps, the closer the
/// wait comes to three minutes".
pub fn nearer_when_more_even(waits: &[Q], even: i64) -> bool {
    let far = |d: usize| (d as i64 - even).abs();
    (0..waits.len()).all(|a| {
        (0..waits.len()).all(|b| match far(a).cmp(&far(b)) {
            Ordering::Less => waits[a] < waits[b],
            Ordering::Equal => waits[a] == waits[b],
            Ordering::Greater => true,
        })
    })
}

/// Every number chapter 12 prints or its voice says, read from the facts
/// and checked against the model.
pub struct Last {
    /// The puzzle: the red bus's offset, its two gaps (drawn and as
    /// printed), the half of each gap's square, their areas, all of them
    /// and the time they cover.
    pub offset: i64,
    pub gaps: [i64; 2],
    short: String,
    long: String,
    half: String,
    areas: [String; 2],
    total_area: String,
    period: String,
    /// The puzzle's wait, as a mixed number of minutes, as a clock and
    /// spelt out.
    mean: String,
    wait_clock: String,
    wait_s: String,
    /// Chapter 07's points in the order 07_6 spoke them (two minutes, six,
    /// none) and its offset axis's ticks, as `no_bus_late::plan` reads them.
    marked: [(i64, String); 3],
    ticks: [(i64, String); 2],
    /// The train, on our timetable: the average wait spelt out, and the
    /// longest wait, drawn, as printed and spelt out.
    cost_s: String,
    pub allowance: i64,
    allowance_printed: String,
    allowance_s: String,
    /// 11's planners, where 12_1 finds them: each dot's moment and its
    /// place in its stack.
    pub planned: Vec<Q>,
    pub ranks: Vec<usize>,
}

impl Last {
    /// 12_3: "4 min → ½ × (4 min)² = 8 min²", "8 min → ½ × (8 min)² = 32
    /// min²", 08_1's row.
    fn area_row(&self, i: usize) -> String {
        let g = if i == 0 { &self.short } else { &self.long };
        format!(
            "{g} {MIN} {ARROW} {} {TIMES} ({g} {MIN}){SQUARED} {EQUALS} {} {AREA}",
            self.half, self.areas[i]
        )
    }

    /// 12_4: "8 min² + 32 min² = 40 min²".
    fn sum_row(&self) -> String {
        format!(
            "{} {AREA} {PLUS} {} {AREA} {EQUALS} {} {AREA}",
            self.areas[0], self.areas[1], self.total_area
        )
    }

    /// "12 min", under the stretch.
    fn stretch_label(&self) -> String {
        format!("{} {MIN}", self.period)
    }

    /// 12_4: "average wait = 40 min² ÷ 12 min".
    fn division(&self) -> String {
        format!(
            "{AVERAGE_WAIT} {EQUALS} {} {AREA} {DIVIDED} {} {MIN}",
            self.total_area, self.period
        )
    }

    /// 12_4: "= 3⅓ min".
    fn mean_step(&self) -> String {
        format!("{EQUALS} {} {MIN}", self.mean)
    }

    /// 12_4: "= 3 min 20 s", boxed.
    fn clock_step(&self) -> String {
        format!("{EQUALS} {}", self.wait_s)
    }

    /// 12_4's whole step, as it is read.
    fn step_text(&self) -> String {
        [self.division(), units(), self.mean_step(), self.clock_step()].join(" ")
    }

    /// 12_6: "average cost: 4 min 20 s".
    fn cost(&self) -> String {
        format!("{AVERAGE_COST} {}", self.cost_s)
    }

    /// 12_7: "allow 10 min".
    fn allow(&self) -> String {
        format!("{ALLOW} {}", self.allowance_s)
    }

    /// K10, the storyboard's exact version of 12_8.
    fn k10(&self) -> String {
        format!("{K10_HEAD} {} {MIN}{K10_TAIL}", self.allowance_printed)
    }
}

/// 12_4: "min² ÷ min = min", square minutes over minutes.
fn units() -> String {
    format!("{AREA} {DIVIDED} {MIN} {EQUALS} {MIN}")
}

/// The names to search, as their band joins them.
fn names() -> String {
    NAMES.join(" · ")
}

/// The names' band as it is read: its label and the names.
fn names_text() -> String {
    format!("{NAMES_LABEL} {}", names())
}

/// The receipts card as it is read: its title and its lines.
fn receipts_text() -> String {
    format!("{RECEIPTS_TITLE} {}", RECEIPTS.join(" "))
}

/// The chapter's numbers, refused when a fact the picture prints or the
/// voice says stops matching the model: the puzzle's gaps, areas and wait,
/// 07's U, the passengers, our timetable's average wait and its longest.
pub fn last(f: &Facts) -> Result<Last, String> {
    let get = |key: &str| f.get(key).map(str::to_owned).ok_or_else(|| format!("no fact {key}"));
    let whole = |key: &str| -> Result<i64, String> {
        let printed = get(key)?;
        printed.parse::<i64>().map_err(|e| format!("{key} = {printed}: {e}"))
    };
    // the puzzle: the red bus four minutes after the blue
    let offset = whole("F-Q1.offset")?;
    if offset != PUZZLE_OFFSET {
        return Err(format!("F-Q1.offset is {offset}, not the puzzle's {PUZZLE_OFFSET}"));
    }
    let hour = model::hour_gaps(offset);
    let short = *hour.iter().min().ok_or("the puzzle has no gaps")?;
    let long = *hour.iter().max().ok_or("the puzzle has no gaps")?;
    let stretch: Vec<i64> = timetable::gaps(offset)
        .into_iter()
        .filter(|g| g.to <= PERIOD)
        .map(|g| g.minutes())
        .collect();
    if stretch != [short, long] {
        return Err(format!(
            "the puzzle's first stretch is {stretch:?}, not the short gap then the long"
        ));
    }
    // 12_2: "four minutes after the blue, so the gaps are four and eight"
    if short != offset || short + long != PERIOD {
        return Err("the puzzle's gaps are not the offset and the rest of twelve minutes".into());
    }
    let half = parse_q("F-F1.area_factor", &get("F-F1.area_factor")?)?;
    let areas = [model::triangle_area(short), model::triangle_area(long)];
    for (g, a) in [(short, areas[0]), (long, areas[1])] {
        if a != half * Q::int(g * g) {
            return Err(format!("the gap of {g} does not hold {half} of its square"));
        }
    }
    let total = areas[0] + areas[1];
    // square minutes over the minutes they cover: a wait in minutes
    let mean = total / Q::int(PERIOD);
    if mean != model::offset_wait(offset) || mean != model::offset_wait_rational(Q::int(offset))? {
        return Err("the puzzle's area over its time is not the timetable's average wait".into());
    }
    // 07's U: a clock per whole offset, nearer three where the gaps are
    // more even
    let clocks: Vec<String> = get("F-O1.wait_clocks")?.split(',').map(str::to_owned).collect();
    if clocks.len() != PERIOD as usize + 1 {
        return Err("F-O1.wait_clocks does not give one clock per whole offset".into());
    }
    let waits: Vec<Q> = (0..=PERIOD).map(model::offset_wait).collect();
    for d in 0..=PERIOD {
        if model::clock(waits[d as usize])? != clocks[d as usize] {
            return Err(format!("offset {d}: the U's wait is not {}", clocks[d as usize]));
        }
    }
    let even = whole("F-O1.even_offset")?;
    if !nearer_when_more_even(&waits, even) {
        return Err("the U is not nearer its least wait wherever the gaps are more even".into());
    }
    let least = *waits.iter().min().ok_or("no offsets")?;
    let best = waits.iter().position(|&w| w == least).ok_or("no least wait")?;
    // the passengers: one in the middle of every minute, evenly spaced
    let ready = model::passengers();
    let apart = Q::new(1, PER_MINUTE);
    if ready.windows(2).any(|w| w[1] - w[0] != apart) || ready.len() as i64 != HOUR * PER_MINUTE {
        return Err("the passengers are not spread evenly, one a minute".into());
    }
    // the train, on our timetable: the average and the longest wait
    let cost = model::mean_wait(&model::hour_gaps(OFFSET));
    let allowance = model::longest_wait(OFFSET);
    let leaving = model::wait_at(OFFSET, Q::int(OFFSET));
    if leaving != Q::int(allowance) {
        return Err("turning up just as the red bus leaves does not wait the longest gap".into());
    }
    let worst = (0..HOUR * 60)
        .map(|s| model::wait_at(OFFSET, Q::new(2 * s + 1, 120)))
        .max()
        .ok_or("no seconds")?;
    if worst > Q::int(allowance) {
        return Err("some moment of the hour waits longer than the allowance".into());
    }
    if cost >= Q::int(allowance) {
        return Err("the average wait is not below the allowance".into());
    }
    let (_, our_long) = facts::short_and_long();
    let checks = [
        ("the puzzle's offset", offset.to_string(), "F-Q1.offset"),
        ("the puzzle's gaps", facts::csv(&stretch), "F-Q1.gaps"),
        ("the puzzle's short gap", short.to_string(), "F-Q1.short"),
        ("the puzzle's long gap", long.to_string(), "F-Q1.long"),
        ("the short triangle's area", areas[0].to_string(), "F-Q1.short_area"),
        ("the long triangle's area", areas[1].to_string(), "F-Q1.long_area"),
        ("the stretch's area", total.to_string(), "F-Q1.total_area"),
        ("the stretch's time", PERIOD.to_string(), "F-T1.period"),
        ("the puzzle's wait", mean.to_string(), "F-Q1.mean_wait"),
        ("the puzzle's wait", model::clock(mean)?, "F-Q1.mean_wait_clock"),
        (
            "the U at the puzzle's offset",
            clocks[offset as usize].clone(),
            "F-Q1.mean_wait_clock",
        ),
        ("our timetable's point", clocks[OFFSET as usize].clone(), "F-O1.wait.d2"),
        ("the even point", clocks[even as usize].clone(), "F-O1.wait.d6"),
        ("the buses together", clocks[0].clone(), "F-O1.wait.d0"),
        ("the least wait", least.to_string(), "F-O1.min_wait"),
        ("the least wait's offset", best.to_string(), "F-O1.even_offset"),
        ("the offset axis's end", PERIOD.to_string(), "F-T1.period"),
        (
            "passengers a minute",
            (ready.len() as i64 / HOUR).to_string(),
            "F-W2.per_minute",
        ),
        ("our timetable's wait", model::clock(cost)?, "F-W1.mean_wait_clock"),
        ("the longest wait", allowance.to_string(), "F-W1.allowance"),
        (
            "the longest wait",
            model::clock(Q::int(allowance))?,
            "F-W1.allowance_clock",
        ),
        ("our long gap", our_long.to_string(), "F-W1.allowance"),
    ];
    for (what, drawn, key) in checks {
        let shown = get(key)?;
        if drawn != shown {
            return Err(format!("{what}: the picture draws {drawn}, but {key} is {shown}"));
        }
    }
    let spell = |key: &str| -> Result<String, String> { min_s(&get(key)?).map_err(|e| e.to_string()) };
    let planners = back_to_the_stop::stop(f)?;
    Ok(Last {
        offset,
        gaps: [short, long],
        short: get("F-Q1.short")?,
        long: get("F-Q1.long")?,
        half: written(half)?,
        areas: [get("F-Q1.short_area")?, get("F-Q1.long_area")?],
        total_area: get("F-Q1.total_area")?,
        period: get("F-T1.period")?,
        mean: written(parse_q("F-Q1.mean_wait", &get("F-Q1.mean_wait")?)?)?,
        wait_clock: get("F-Q1.mean_wait_clock")?,
        wait_s: spell("F-Q1.mean_wait_clock")?,
        marked: [
            (OFFSET, get("F-O1.wait.d2")?),
            (best as i64, get("F-O1.wait.d6")?),
            (0, get("F-O1.wait.d0")?),
        ],
        ticks: [(even, get("F-O1.even_offset")?), (PERIOD, get("F-T1.period")?)],
        cost_s: spell("F-W1.mean_wait_clock")?,
        allowance,
        allowance_printed: get("F-W1.allowance")?,
        allowance_s: spell("F-W1.allowance_clock")?,
        planned: planners.planned,
        ranks: planners.ranks,
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

/// The strip of 01_3, detached: the street with you at the stop, halved at
/// the top. Returns the strip and you.
fn the_strip(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<(Mob, Mob)> {
    let mut members = street(kit, stage)?;
    let me = you(stage, at_the_stop())?;
    members.push(me);
    let strip = group(stage, &members)?;
    stage.scale(strip, STRIP_SCALE);
    stage.move_to(strip, STRIP_AT, ORIGIN);
    Ok((strip, me))
}

/// The length of the gap moment `t` (minutes into the hour) falls in, 0
/// outside the hour (chapter 11's rule for a sliding dot's color).
fn minutes_at(gaps: &[Gap], t: f64) -> i64 {
    gaps.iter()
        .find(|g| g.from as f64 <= t && t < g.to as f64)
        .map_or(0, Gap::minutes)
}

/// A fresh anchor for an updater, in the scene (clear its updaters to stop
/// the updater).
fn anchor(stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    let anchor = stage.arena_mut().add(VMobject::new());
    stage.add_to_scene(anchor)?;
    Ok(anchor)
}

/// An updater that takes the planners back as the tracker `back` runs from
/// 1 to 0: each dot back along chapter 11's own path
/// (`back_to_the_stop::slide_at`), out of its stack, to the moment it was
/// ready, in the color of the gap it is in. Returns its anchor; clear it
/// when the slide is over.
fn unstack(stage: &mut Stage<'_>, dots: &[Mob], ready: &[Q], n: &Last, back: Mob) -> fmn::Result<Mob> {
    let gaps = timetable::gaps(OFFSET);
    let paths: Vec<(Q, Q, usize)> = (0..dots.len()).map(|i| (ready[i], n.planned[i], n.ranks[i])).collect();
    let mut painted: Vec<i64> = paths
        .iter()
        .map(|&(r, p, k)| minutes_at(&gaps, slide_at(r, p, k, 1.0).0))
        .collect();
    let dots = dots.to_vec();
    let a = anchor(stage)?;
    stage.add_updater(
        a,
        move |st, _| {
            let at = st.tracker_value(back).unwrap_or(1.0);
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
    Ok(a)
}

/// Keep each red marker the tracker `d`'s minutes after its blue bus
/// (`no_bus_late::red_x`). Returns the updater's anchor.
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

/// Whether the passenger at moment `t` stands in a gap that starts at a red
/// bus, with the red bus `d` minutes after the blue: the long gap at both
/// timetables the chapter draws, gold; the rest teal (chapter 07's rule).
pub fn after_red(t: Q, d: f64) -> bool {
    t.to_f64().rem_euclid(PERIOD as f64) >= d
}

/// Keep each passenger in its gap's color as the tracker `d` moves the red
/// bus ([`after_red`]). Returns the updater's anchor.
fn paint(stage: &mut Stage<'_>, d: Mob, dots: &[Mob], people: &[Q]) -> fmn::Result<Mob> {
    let dots = dots.to_vec();
    let people = people.to_vec();
    let mut shown: Vec<Option<bool>> = vec![None; dots.len()];
    let (gold, teal) = (palette::long_gap(), timetable::short_gap());
    let a = anchor(stage)?;
    stage.add_updater(
        a,
        move |s, _| {
            let dv = s.tracker_value(d).unwrap_or(0.0);
            for (i, &m) in dots.iter().enumerate() {
                let now_gold = after_red(people[i], dv);
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

/// A triangle's corners on screen, over the line: a minute up as long as a
/// minute across.
fn over_line(c: [(f64, f64); 3]) -> Vec<Vec3> {
    c.iter().map(|&(t, w)| v(x_at(t), LINE_Y + w * minute_up())).collect()
}

/// The hour's triangles with the red bus `d` minutes after the blue
/// (`no_bus_late::triangles_at`), in each twelve minutes the gap after the
/// blue bus teal and the gap after the red gold; see-through, behind the
/// markers; detached.
fn hour_triangles(stage: &mut Stage<'_>, d: f64) -> Vec<Mob> {
    triangles_at(d)
        .into_iter()
        .enumerate()
        .map(|(i, c)| {
            let color = if i % 2 == 0 {
                timetable::short_gap()
            } else {
                palette::long_gap()
            };
            let m = stage
                .arena_mut()
                .add(Polygon::new(over_line(c)).style(flat(color, TRI_FILL)).build());
            stage.set_z_index(m, TRI_Z, true);
            m
        })
        .collect()
}

/// Keep the triangles the tracker `d`'s shape. Returns the updater's
/// anchor. (The chapter moves the red bus only between two and four
/// minutes, so no triangle closes up.)
fn reshape(stage: &mut Stage<'_>, d: Mob, shapes: &[Mob]) -> fmn::Result<Mob> {
    let shapes = shapes.to_vec();
    let a = anchor(stage)?;
    stage.add_updater(
        a,
        move |s, _| {
            let dv = s.tracker_value(d).unwrap_or(OFFSET as f64);
            for (&m, c) in shapes.iter().zip(triangles_at(dv)) {
                let _ = s.set_points(m, Polygon::new(over_line(c)).build().points());
            }
        },
        true,
    )?;
    Ok(a)
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

/// A verdict: its words in green and their outline, apart, centred at
/// `at`; detached.
fn boxed(kit: &Kit, stage: &mut Stage<'_>, text: &str, at: Vec3) -> fmn::Result<[Mob; 2]> {
    let color = palette::verdict();
    let words = kit.text(stage, text, STEP_SIZE, color, at)?;
    let w = stage.get_width(words) + 2.0 * BOX_PAD[0];
    let h = stage.get_height(words) + 2.0 * BOX_PAD[1];
    let outline = rect(v(at[0] - w / 2.0, at[1] - h / 2.0), w, h, line_style(color, BOX_W))?;
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

/// A small dot in `color` at `at`; detached.
fn spot(stage: &mut Stage<'_>, at: Vec3, r: f64, color: Srgb) -> Mob {
    stage
        .arena_mut()
        .add(Dot::new().point(at).radius(r).style(flat(color, 1.0)).build())
}

/// The plot's frame: the offset axis (the slider's track), the wait axis,
/// the offset ticks and the slider's name.
fn plot_frame(kit: &Kit, stage: &mut Stage<'_>, p: &Last) -> fmn::Result<Vec<Mob>> {
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
fn mark_at(p: &Last, d: i64) -> (Vec3, Vec3) {
    let at = plot_at(d as f64, wait_of(d as f64));
    if d == 0 {
        (v(PLOT_O[0] - MARK_D0_DX, at[1]), RIGHT)
    } else if d == p.marked[1].0 {
        (v(at[0], at[1] + MARK_D6_DY), ORIGIN)
    } else {
        (v(at[0] + MARK_D2[0], at[1] + MARK_D2[1]), ORIGIN)
    }
}

/// A point on the U at offset `d` and its clock beside it, as 07_6 marks
/// them: [dot, label]; detached.
fn mark(kit: &Kit, stage: &mut Stage<'_>, p: &Last, d: i64, clock: &str) -> fmn::Result<[Mob; 2]> {
    let dot = spot(stage, plot_at(d as f64, wait_of(d as f64)), POINT_R, palette::verdict());
    let (at, edge) = mark_at(p, d);
    let label = kit.text(stage, clock, MARK_SIZE, palette::verdict(), at)?;
    stage.move_to(label, at, edge);
    Ok([dot, label])
}

impl SceneConstruct for OneMoreTimetable {
    fn name(&self) -> &str {
        "12_one_more_timetable"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let f = facts::facts().map_err(fail)?;
        let n = last(&f).map_err(fail)?;
        let ready = model::passengers();
        let gaps = timetable::gaps(OFFSET);
        let (teal, gold, green) = (timetable::short_gap(), palette::long_gap(), palette::verdict());

        // 12_1: chapter 11's end, the planners in their stacks before the
        // buses, each in its gap's color; they slide back to the moments
        // they were ready; one a minute; spread evenly, the caption
        let line = say(&kit, stage, "12_1")?;
        let table = timetable::timetable(&kit, stage, &f, OFFSET)?;
        let dots: Vec<Mob> = (0..ready.len())
            .map(|i| {
                let (t, y) = slide_at(ready[i], n.planned[i], n.ranks[i], 1.0);
                let m = timetable::passenger(stage, ready[i], gap_color(minutes_at(&gaps, t)));
                stage.move_to(m, v(x_at(t), y), ORIGIN);
                m
            })
            .collect();
        let mut opening = table.hour.mobs();
        opening.extend(&table.blue);
        opening.extend(&table.red);
        opening.extend(&dots);
        fade_in_all(stage, &opening, 0.8)?;
        line.wait_for(stage, "Now forget the planners")?;
        let back = stage.add_value_tracker(1.0);
        let unstacking = unstack(stage, &dots, &ready, &n, back)?;
        stage.play(tween(back, 0.0, UNSTACK_RT)?)?;
        stage.clear_updaters(unstacking, true);
        line.wait_for(stage, "one a minute")?;
        flash_lagged(stage, &dots, 1.0, 0.05)?;
        line.wait_for(stage, "spread evenly over the hour")?;
        let caption = kit.text(stage, AT_RANDOM, CAPTION_SIZE, Palette::dim(), CAPTION_AT)?;
        fade_in_all(stage, &[caption], 0.5)?;

        // 12_2: the red bus to four minutes after the blue, the passengers'
        // colors following; every gap's bracket; the question; think time.
        // No triangle, area or wait until the countdown ends
        let line = say(&kit, stage, "12_2")?;
        let d = stage.add_value_tracker(OFFSET as f64);
        let beside = stage.get_width(table.blue[0]);
        let reds = move_reds(stage, d, &table.red, beside)?;
        let painter = paint(stage, d, &dots, &ready)?;
        line.wait_for(stage, "Move the red bus")?;
        stage.play(tween(d, n.offset as f64, SLIDE_RT)?)?;
        let mut brackets = Vec::new();
        for g in &timetable::gaps(n.offset) {
            let (label, color) = if g.minutes() == n.gaps[0] {
                (&n.short, teal)
            } else {
                (&n.long, gold)
            };
            let b = timetable::bracket(&kit, stage, g, label, color)?;
            brackets.push((b.line, b.label));
        }
        let bracket_mobs: Vec<Mob> = brackets.iter().flat_map(|&(l, t)| [l, t]).collect();
        fade_in_lagged(stage, &bracket_mobs, 1.0, 0.1)?;
        line.wait_for(stage, "What's the average wait")?;
        let ask = kit.text(stage, ASK, ASK_SIZE, Palette::ink(), ASK_AT)?;
        fade_in_all(stage, &[ask], 0.4)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 12_3: the question goes, and the brackets after the first twelve
        // minutes; the triangles rise; each gap of the first twelve minutes
        // brightens with its area, the rest dim
        let line = say(&kit, stage, "12_3")?;
        let mut later = vec![q, ask];
        later.extend(brackets[2..].iter().flat_map(|&(l, t)| [l, t]));
        fade_out_all(stage, &later, 0.4)?;
        let tris = hour_triangles(stage, n.offset as f64);
        grow_lagged(stage, &tris, GROW_RT, 0.1)?;
        line.wait_for(stage, "The gap of four")?;
        let rows = [
            kit.text(stage, &n.area_row(0), ROW_SIZE, teal, v(ROW_X, ROW_YS[0]))?,
            kit.text(stage, &n.area_row(1), ROW_SIZE, gold, v(ROW_X, ROW_YS[1]))?,
        ];
        let mut light = vec![fade_to(tris[0], TRI_BRIGHT, 0.6)?];
        light.extend(all_to(&tris[2..], TRI_DIM, 0.6)?);
        fade_in_with(stage, &[rows[0]], light, 0.6)?;
        flash_all(stage, &[brackets[0].1], 0.6)?;
        line.wait_for(stage, "The gap of eight")?;
        fade_in_with(stage, &[rows[1]], vec![fade_to(tris[1], TRI_BRIGHT, 0.6)?], 0.6)?;
        flash_all(stage, &[brackets[1].1], 0.6)?;

        // 12_4: the sum; the twelve minutes the two cover; the area over
        // the time and its units; the wait in minutes, then to the second,
        // boxed; the step stands with no voice until it can be read
        let line = say(&kit, stage, "12_4")?;
        let sum = kit.text(stage, &n.sum_row(), ROW_SIZE, Palette::ink(), v(ROW_X, ROW_YS[2]))?;
        fade_in_all(stage, &[sum], 0.5)?;
        line.wait_for(stage, "over the twelve minutes they cover")?;
        let first: Vec<Mob> = brackets[..2].iter().flat_map(|&(l, t)| [l, t]).collect();
        fade_out_all(stage, &first, 0.4)?;
        let stretch = stretch_bracket(stage, 0.0, PERIOD as f64);
        let stretch_label = kit.text(
            stage,
            &n.stretch_label(),
            STRETCH_SIZE,
            Palette::ink(),
            v(x_at(PERIOD as f64 / 2.0), STRETCH_LABEL_Y),
        )?;
        grow_with(stage, stretch, LEFT, &[stretch_label], 0.6)?;
        line.wait_for(stage, "Dividing square minutes by minutes")?;
        let division = kit.text(stage, &n.division(), STEP_SIZE, Palette::ink(), v(STEP_X, STEP_YS[0]))?;
        let per = kit.text(stage, &units(), UNIT_SIZE, Palette::dim(), v(STEP_X, STEP_YS[1]))?;
        fade_in_all(stage, &[division], 0.5)?;
        fade_in_all(stage, &[per], 0.4)?;
        line.wait_for(stage, "Forty over twelve")?;
        let mean = kit.text(stage, &n.mean_step(), STEP_SIZE, Palette::ink(), v(STEP_X, STEP_YS[2]))?;
        fade_in_all(stage, &[mean], 0.4)?;
        line.wait_for(stage, "Three minutes and twenty seconds")?;
        let [outline, clock] = boxed(&kit, stage, &n.clock_step(), v(STEP_X, STEP_YS[3]))?;
        fade_in_all(stage, &[outline, clock], 0.5)?;
        halo(stage, &[clock], green, HALO_RT)?;
        let step_up = now(stage);
        kit.hold(stage)?;
        let held = now(stage);
        wait_until(stage, (step_up + read_s(&n.step_text())).max(held + INSPECT_S))?;

        // 12_5: chapter 07's plot as 07_6 left it, back before the line
        // names it; the knob at four minutes, its guide and the puzzle's
        // point on the U; our timetable's point and the even one's; the even
        // one, nearest three
        let mut plot = plot_frame(&kit, stage, &n)?;
        plot.push(
            stage.arena_mut().add(
                polyline(curve(0.0, PERIOD as f64))
                    .style(line_style(green, CURVE_W))
                    .build(),
            ),
        );
        let mut marks = Vec::new();
        for (dm, clock_at) in n.marked.clone() {
            marks.push(mark(&kit, stage, &n, dm, &clock_at)?);
        }
        let mut callback = plot.clone();
        callback.extend(marks.iter().flatten());
        fade_in_all(stage, &callback, CALLBACK_RT)?;
        let line = say(&kit, stage, "12_5")?;
        line.wait_for(stage, "That's a point on the curve")?;
        let at = n.offset as f64;
        let knob = spot(stage, plot_at(at, 0.0), KNOB_R, palette::you());
        let guide = stage.arena_mut().add(
            polyline(vec![plot_at(at, 0.0), plot_at(at, wait_of(at))])
                .style(line_style(Palette::faint(), 2.0))
                .build(),
        );
        fade_in_all(stage, &[knob, guide], 0.4)?;
        let puzzle = mark(&kit, stage, &n, n.offset, &n.wait_clock)?;
        fade_in_all(stage, &puzzle, 0.4)?;
        line.wait_for(stage, "between our timetable and the even timetable")?;
        halo(stage, &[marks[0][1], marks[1][1]], green, HALO_RT)?;
        line.wait_for(stage, "The more even the gaps")?;
        flash_all(stage, &[marks[1][1]], 0.8)?;

        // 12_6: the puzzle's picture goes and the triangles return to one
        // brightness; the strip of 01; the red bus back to our timetable,
        // the triangles reshaping with it; the average cost
        let line = say(&kit, stage, "12_6")?;
        let mut gone = rows.to_vec();
        gone.extend([
            sum,
            stretch,
            stretch_label,
            division,
            per,
            mean,
            outline,
            clock,
            caption,
            knob,
            guide,
        ]);
        gone.extend(&plot);
        gone.extend(marks.iter().flatten());
        gone.extend(puzzle);
        gone.extend(&dots);
        fade_out_with(stage, &gone, all_to(&tris, TRI_FILL, 0.6)?, 0.6)?;
        let (strip, me) = the_strip(&kit, stage)?;
        fade_in_all(stage, &[strip], 0.6)?;
        line.wait_for(stage, "Back on our timetable")?;
        let shaper = reshape(stage, d, &tris)?;
        stage.play(tween(d, OFFSET as f64, SLIDE_RT)?)?;
        for a in [shaper, reds, painter] {
            stage.clear_updaters(a, true);
        }
        line.wait_for(stage, "four minutes twenty")?;
        let cost = kit.text(stage, &n.cost(), COST_SIZE, green, COST_AT)?;
        fade_in_all(stage, &[cost], 0.5)?;

        // 12_7: the long triangle after the red bus at :02 brightens; you
        // leave the strip for the moment that bus leaves; 01's wait bar
        // grows from the blue bus at :12 back to you; the allowance
        let line = say(&kit, stage, "12_7")?;
        line.wait_for(stage, "your worst morning")?;
        let mut light = vec![fade_to(tris[1], TRI_BRIGHT, 0.6)?];
        light.extend(all_to(&[&tris[..1], &tris[2..]].concat(), TRI_DIM, 0.6)?);
        fade_in_with(stage, &[], light, 0.6)?;
        line.wait_for(stage, "Turn up just as a red bus leaves")?;
        let home = stage.get_center(me);
        let leaves = model::route(OFFSET)[0];
        let bus = leaves + n.allowance;
        stage.play(shrink_to(
            me,
            1.0 / STRIP_SCALE,
            v(x_of(Q::int(leaves)), BAR_Y),
            YOU_RT,
        )?)?;
        let unit = x_of(Q::int(1)) - x_of(Q::int(0));
        let left = stage.add_value_tracker(0.0);
        let bar = live_bar(stage, left, v(x_of(Q::int(bus)), BAR_Y), unit, BAR_H, gold)?;
        stage.play(tween(left, -(n.allowance as f64), BAR_RT)?)?;
        // the bar stands still from here, so a fade can take it
        stage.clear_updaters(bar, true);
        line.wait_for(stage, "allow the full ten minutes")?;
        let middle = (x_of(Q::int(leaves)) + x_of(Q::int(bus))) / 2.0;
        let allow = kit.text(
            stage,
            &n.allow(),
            ALLOW_SIZE,
            palette::you(),
            v(middle, BAR_Y + BAR_LABEL_DY),
        )?;
        fade_in_all(stage, &[allow], 0.4)?;

        // 12_8: every bus on time, 01's green marks; the bound in this
        // model only, K10, held until read
        let line = say(&kit, stage, "12_8")?;
        line.wait_for(stage, "only because every bus runs exactly on time")?;
        let mut buses: Vec<i64> = model::route(0);
        buses.extend(model::route(OFFSET));
        buses.sort_unstable();
        let mut on_time: Vec<Mob> = buses
            .iter()
            .map(|&t| {
                let top = marker_top(t);
                spot(stage, v(top[0], top[1] + ON_TIME_DY), PASSENGER_R, green)
            })
            .collect();
        let on_time_words = kit.text(stage, ON_TIME, ON_TIME_SIZE, green, ON_TIME_AT)?;
        on_time.push(on_time_words);
        fade_in_lagged(stage, &on_time, 0.8, 0.1)?;
        line.wait_for(stage, "It's a bound in this model")?;
        let k10_text = n.k10();
        let k10 = precisely(&kit, stage, &k10_text)?;
        fade_in_all(stage, &[k10], 0.5)?;
        let k10_up = now(stage);
        kit.hold(stage)?;
        wait_until(stage, k10_up + read_s(&k10_text))?;

        // 12_9: before the line, K10, the bar and its label go while you go
        // back to the stop; every bus on time; the passengers return, the
        // long gaps gold; then the names, read; then the receipts alone, read
        let back_home = vec![shrink_to(me, STRIP_SCALE, home, YOU_RT)?];
        fade_out_with(stage, &[k10, bar, allow], back_home, YOU_RT)?;
        let line = say(&kit, stage, "12_9")?;
        line.wait_for(stage, "Your bus isn't late")?;
        halo(stage, &[on_time_words], green, HALO_RT)?;
        line.wait_for(stage, "You're just more likely")?;
        let shades: Vec<Mob> = gaps
            .iter()
            .filter(|g| g.minutes() == n.allowance)
            .map(|g| timetable::span(stage, g.from as f64, g.to as f64, gold))
            .collect();
        fade_in_lagged(stage, &dots, 1.2, 0.05)?;
        fade_in_all(stage, &shades, 0.6)?;
        kit.hold(stage)?;
        let named = names_to_search(&kit, stage, &NAMES)?;
        fade_in_all(stage, &[named], 0.5)?;
        let names_up = now(stage);
        wait_until(stage, names_up + read_s(&names_text()))?;
        fade_all(stage, 0.6)?;
        let receipts = receipts_card(&kit, stage, &RECEIPTS)?;
        fade_in_all(stage, &[receipts], 0.5)?;
        let receipts_up = now(stage);
        wait_until(stage, receipts_up + read_s(&receipts_text()).max(STAND_UP))?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use explainer_kit::{Narrator, Script};

    use super::super::draw::{LINE_X0, LINE_X1, PAUSE_NOTE};
    use super::*;

    /// The full draft with its line breaks and indents as single spaces.
    fn draft() -> String {
        crate::DRAFT.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    fn the_last() -> Last {
        last(&facts::facts().unwrap()).unwrap()
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

    /// Needs chapter 12's lines in script.tsv (the integration step adds
    /// them from the storyboard). The whole chapter runs headless at 10 fps
    /// with the scratch voice: every phrase it waits for is in its line,
    /// every fact it reads exists and matches the model, and its nine lines
    /// are said in order. The reading holds are real time on the timeline:
    /// 12_4's step stands past its line and breath, K10 is up for its
    /// reading time before 12_9 begins, and after the last line the names
    /// and then the receipts each stand for their own. It ends with
    /// `kit.finish`.
    #[test]
    fn the_chapter_runs_and_says_its_lines_in_order() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let narrator = Narrator::estimate(&script);
        let kit = Kit::new().unwrap().with_narrator(Rc::clone(&narrator));
        let mut scene = OneMoreTimetable { kit };
        let config = fmn::scene::RuntimeConfig {
            fps: 10,
            ..fmn::scene::RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut fmn::scene::NullSceneSink).unwrap();
        let said_at = narrator.said();
        let said: Vec<&str> = said_at.iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(
            said,
            ["12_1", "12_2", "12_3", "12_4", "12_5", "12_6", "12_7", "12_8", "12_9"]
        );
        let at = |id: &str| said_at.iter().find(|(i, _)| i == id).unwrap().1;
        let ends = |id: &str| at(id) + narrator.length(id).unwrap();
        assert!(
            at("12_5") - ends("12_4") >= INSPECT_S,
            "12_4's step stands {:.2} s past its line",
            at("12_5") - ends("12_4")
        );
        let n = the_last();
        assert!(at("12_9") - at("12_8") > read_s(&n.k10()));
        let (last, last_at) = said_at.last().unwrap();
        let last_ends = last_at + narrator.length(last).unwrap();
        let end = scene
            .kit
            .finished
            .borrow()
            .as_ref()
            .expect("no timeline: the chapter must end with kit.finish")
            .seconds();
        let reading = read_s(&names_text()) + read_s(&receipts_text());
        assert!(
            end > last_ends + reading,
            "the timeline ends at {end:.2} s, {:.2} s after {last}, short of the names' and receipts' {reading:.2} s",
            end - last_ends
        );
    }

    /// Every number the chapter prints is the model's, and a fact that
    /// stops matching the puzzle, the U, the passengers or the train is
    /// refused: the negative controls, among them the wrong models (40 read
    /// as the wait, the even guess of three, the allowance taken as the
    /// average, the puzzle's longest gap taken as our timetable's).
    #[test]
    fn the_numbers_are_the_models_and_a_wrong_fact_is_refused() {
        let n = the_last();
        assert_eq!(n.offset, PUZZLE_OFFSET);
        assert_eq!(n.gaps, [4, 8]);
        assert_eq!(n.area_row(0), "4 min → ½ × (4 min)² = 8 min²");
        assert_eq!(n.area_row(1), "8 min → ½ × (8 min)² = 32 min²");
        assert_eq!(n.sum_row(), "8 min² + 32 min² = 40 min²");
        assert_eq!(n.stretch_label(), "12 min");
        assert_eq!(n.division(), "average wait = 40 min² ÷ 12 min");
        assert_eq!(units(), "min² ÷ min = min");
        assert_eq!(n.mean_step(), "= 3⅓ min");
        assert_eq!(n.clock_step(), "= 3 min 20 s");
        assert_eq!(n.wait_clock, "3:20");
        assert_eq!(n.cost(), "average cost: 4 min 20 s");
        assert_eq!(n.allow(), "allow 10 min");
        assert_eq!(n.allowance, 10);
        assert_eq!(
            n.marked,
            [(2, "4:20".to_owned()), (6, "3:00".to_owned()), (0, "6:00".to_owned())]
        );
        assert_eq!(n.ticks, [(6, "6".to_owned()), (12, "12".to_owned())]);
        for (key, bad) in [
            ("F-Q1.offset", "2"),
            ("F-Q1.gaps", "2,10"),
            ("F-Q1.short", "2"),
            ("F-Q1.long", "10"),
            ("F-Q1.short_area", "16"),
            ("F-Q1.long_area", "64"),
            // the area read as the wait, and the even guess
            ("F-Q1.total_area", "52"),
            ("F-Q1.mean_wait", "40"),
            ("F-Q1.mean_wait", "3"),
            ("F-Q1.mean_wait_clock", "4:20"),
            ("F-O1.wait.d2", "3:20"),
            ("F-O1.min_wait", "10/3"),
            ("F-W2.per_minute", "2"),
            // the allowance taken as the average, or as the puzzle's long gap
            ("F-W1.allowance_clock", "4:20"),
            ("F-W1.allowance", "8"),
            ("F-W1.mean_wait_clock", "10:00"),
        ] {
            let refused = last(&with(key, bad)).err().expect("a wrong fact must be refused");
            assert!(refused.contains(key), "{key} = {bad}: {refused}");
        }
        // the puzzle's own longest gap is not our timetable's allowance:
        // the chapter slides back to our timetable before the train
        assert_eq!(model::longest_wait(PUZZLE_OFFSET), n.gaps[1]);
        assert_ne!(model::longest_wait(PUZZLE_OFFSET), n.allowance);
    }

    /// The area is counted in square minutes and divided by minutes, and the
    /// voice never calls it minutes of waiting. The draft's earlier 12_4,
    /// which did, is the negative control.
    #[test]
    fn the_area_is_square_minutes_over_minutes() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let says_area_is_waiting = |text: &str| text.contains("minutes of waiting");
        let old = "That's forty minutes of waiting every twelve minutes, and forty over twelve is three and a third.";
        assert!(says_area_is_waiting(old));
        for id in ["12_3", "12_4"] {
            let text = script.text(id).unwrap();
            assert!(!says_area_is_waiting(text), "{id}: {text}");
        }
        let three = script.text("12_3").unwrap();
        assert!(three.contains("eight square minutes") && three.contains("thirty-two square minutes"));
        let four = script.text("12_4").unwrap();
        for said in [
            "forty square minutes",
            "over the twelve minutes they cover",
            "Dividing square minutes by minutes leaves minutes",
            "three and a third minutes",
        ] {
            assert!(four.contains(said), "12_4 no longer says {said:?}");
        }
        // what is shown divides square minutes by minutes into minutes
        let n = the_last();
        assert!(n.division().contains(&format!("{AREA} {DIVIDED} {} {MIN}", n.period)));
        assert!(n.mean_step().ends_with(&format!(" {MIN}")) && !n.mean_step().contains(AREA));
        let f = facts::facts().unwrap();
        let total = parse_q("F-Q1.total_area", f.get("F-Q1.total_area").unwrap()).unwrap();
        let mean = parse_q("F-Q1.mean_wait", f.get("F-Q1.mean_wait").unwrap()).unwrap();
        assert_eq!(total / Q::int(PERIOD), mean);
        assert_eq!(mean, Q::new(10, 3));
    }

    /// The U is nearer three wherever the gaps are more even, at its edges
    /// too (six at none and at twelve, three at six), between whole minutes
    /// as well (tenths of a minute, `model::offset_wait_rational`), and the
    /// puzzle's point is on it. A U out of order is refused.
    #[test]
    fn the_u_is_nearer_three_where_the_gaps_are_more_even() {
        let waits: Vec<Q> = (0..=PERIOD).map(model::offset_wait).collect();
        assert!(nearer_when_more_even(&waits, 6));
        assert_eq!((waits[0], waits[6], waits[12]), (Q::int(6), Q::int(3), Q::int(6)));
        let tenths: Vec<Q> = (0..=10 * PERIOD)
            .map(|k| model::offset_wait_rational(Q::new(k, 10)).unwrap())
            .collect();
        assert!(nearer_when_more_even(&tenths, 60));
        let mut swapped = waits.clone();
        swapped.swap(4, 5);
        assert!(!nearer_when_more_even(&swapped, 6));
        assert!(!nearer_when_more_even(&waits, 5));
        assert!((wait_of(PUZZLE_OFFSET as f64) - 10.0 / 3.0).abs() < 1e-12);
        let p = plot_at(PUZZLE_OFFSET as f64, wait_of(PUZZLE_OFFSET as f64));
        assert!(
            curve(0.0, PERIOD as f64)
                .iter()
                .any(|c| (c[0] - p[0]).abs() < 1e-9 && (c[1] - p[1]).abs() < 1e-9)
        );
    }

    /// The longest wait on our timetable is the long gap, waited by whoever
    /// turns up just as a red bus leaves; no second of the hour waits
    /// longer, and the average is well below it. On the puzzle's timetable
    /// the longest wait would be its long gap, eight.
    #[test]
    fn the_longest_wait_is_the_long_gap() {
        let n = the_last();
        for red in model::route(OFFSET) {
            assert_eq!(model::wait_at(OFFSET, Q::int(red)), Q::int(n.allowance), ":{red:02}");
        }
        let worst = (0..HOUR * 60)
            .map(|s| model::wait_at(OFFSET, Q::new(2 * s + 1, 120)))
            .max()
            .unwrap();
        assert!(worst < Q::int(n.allowance) && worst > Q::int(n.allowance - 1));
        assert!(model::mean_wait(&model::hour_gaps(OFFSET)) < Q::int(n.allowance));
        let (_, long) = facts::short_and_long();
        assert_eq!(n.allowance, long);
        assert_eq!(model::longest_wait(PUZZLE_OFFSET), 8);
    }

    /// 12_1 is 11_6's slide run backwards: at the slide's start each dot is
    /// in its stack where 11 left it, at its end on the passengers' row at
    /// the moment it was ready, one a minute, and it only ever moves back.
    #[test]
    fn the_planners_go_back_to_where_they_were_ready() {
        let n = the_last();
        let ready = model::passengers();
        assert_eq!(ready.len() as i64, HOUR * PER_MINUTE);
        let ahead = |t: f64, r: Q| (t - r.to_f64()).rem_euclid(HOUR as f64);
        for (i, &r) in ready.iter().enumerate() {
            let (p, rank) = (n.planned[i], n.ranks[i]);
            let (t0, y0) = slide_at(r, p, rank, 0.0);
            assert!((t0 - r.to_f64()).abs() < 1e-12 && (y0 - timetable::DOT_Y).abs() < 1e-12);
            let (t1, _) = slide_at(r, p, rank, 1.0);
            assert!((t1 - back_to_the_stop::in_hour(p).to_f64()).abs() < 1e-12);
            // back from 1 to 0, the dot's distance ahead of its ready moment
            // only shrinks
            let mut was = f64::INFINITY;
            for k in (0..=20).rev() {
                let (t, _) = slide_at(r, p, rank, k as f64 / 20.0);
                let now_ahead = ahead(t, r);
                assert!(now_ahead <= was + 1e-12, "dot {i} moves forward at {k}/20");
                was = now_ahead;
            }
        }
    }

    /// The passengers' colors follow the red bus by chapter 07's rule, and
    /// at both timetables the chapter draws that is the gap each stands in:
    /// gold in the gap from a red bus, the long one; teal in the short. The
    /// triangles teal then gold in each twelve minutes are those gaps, to
    /// scale: their areas are the facts'.
    #[test]
    fn the_colors_and_triangles_are_the_gaps() {
        let n = the_last();
        for (d, long) in [(OFFSET, facts::short_and_long().1), (n.offset, n.gaps[1])] {
            for t in model::passengers() {
                assert_eq!(after_red(t, d as f64), model::gap_at(d, t) == long, "d = {d}, t = {t}");
            }
            let shoelace = |c: [(f64, f64); 3]| {
                let p = over_line(c);
                let twice = (0..3)
                    .map(|i| {
                        let (a, b) = (p[i], p[(i + 1) % 3]);
                        a[0] * b[1] - b[0] * a[1]
                    })
                    .sum::<f64>();
                twice.abs() / 2.0 / (minute_up() * minute_up())
            };
            let tris = triangles_at(d as f64);
            for (i, c) in tris.iter().enumerate() {
                let g = if i % 2 == 0 { d } else { PERIOD - d };
                assert!(
                    (shoelace(*c) - model::triangle_area(g).to_f64()).abs() < 1e-9,
                    "d = {d}, {i}"
                );
            }
        }
        assert!((minute_up() - (LINE_X1 - LINE_X0) / HOUR as f64).abs() < 1e-12);
    }

    /// The street is chapter 01's: `street` and every constant it, the strip
    /// and 01_4's wait bar use are copied word for word from `the_stop.rs`.
    #[test]
    fn the_street_is_chapter_01s() {
        let ours = include_str!("one_more_timetable.rs");
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
            "BAR_Y",
            "BAR_H",
            "BAR_LABEL_DY",
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
        assert!(theirs.contains("live_bar(stage, left, v(x_of(bus), BAR_Y), unit, BAR_H, color)"));
        assert!(theirs.contains("let color = palette::long_gap();"));
    }

    /// The plot is chapter 07's: its frame and its labels' places are
    /// copied from `no_bus_late.rs`, except the raised axis caption (there they read 07's
    /// plan, here the chapter's numbers), with every constant they use, and
    /// the chapter's numbers give the frame 07's ticks and points.
    #[test]
    fn the_plot_is_chapter_07s() {
        let ours = include_str!("one_more_timetable.rs");
        let theirs = include_str!("no_bus_late.rs");
        let body = |src: &'static str, name: &str| -> String {
            let from = src.find(&format!("\nfn {name}(")).expect("the function");
            let to = from + src[from..].find("\n}\n").expect("its end");
            src[from..to].replace("p: &Last", "p: &Plan")
        };
        for name in ["plot_frame", "mark_at", "spot"] {
            assert_eq!(body(ours, name), body(theirs, name), "{name}");
        }
        for name in [
            "PLOT_O",
            "PLOT_SIZE",
            "PLOT_TICK_DY",
            "CURVE_W",
            "KNOB_R",
            "POINT_R",
            "MARK_SIZE",
            "MARK_D2",
            "MARK_D6_DY",
            "MARK_D0_DX",
            "SLIDER",
        ] {
            let line = |src: &'static str| -> Vec<&'static str> {
                src.lines()
                    .filter(|l| l.starts_with(&format!("const {name}:")))
                    .collect()
            };
            assert_eq!(line(ours).len(), 1, "{name}");
            assert_eq!(line(ours), line(theirs), "{name}");
        }
        // Native12 preview placed the old caption below the safe margin.
        const {
            assert!(PLOT_O[1] - PLOT_NAME_DY - 0.09 > -3.85);
            assert!(PLOT_NAME_DY - PLOT_TICK_DY > 0.17);
        }
        // 07's own plot reads the same ticks and points from the facts
        assert!(theirs.contains("ticks: [(even, get(\"F-O1.even_offset\")?), (PERIOD, get(\"F-T1.period\")?)],"));
        assert!(theirs.contains("(OFFSET, get(\"F-O1.wait.d2\")?),"));
    }

    /// What the chapter prints is the storyboard's picture, word for word.
    #[test]
    fn the_printed_words_are_the_drafts() {
        let n = the_last();
        let board = draft();
        let mut shown = vec![
            AT_RANDOM.to_owned(),
            n.short.clone(),
            n.long.clone(),
            ASK.to_owned(),
            n.area_row(0),
            n.area_row(1),
            n.sum_row(),
            n.stretch_label(),
            n.division(),
            units(),
            n.mean_step(),
            n.clock_step(),
            SLIDER.to_owned(),
            n.cost(),
            n.allow(),
            ON_TIME.to_owned(),
            n.k10(),
            NAMES_LABEL.to_owned(),
            names(),
        ];
        shown.extend(RECEIPTS.iter().map(|r| (*r).to_owned()));
        for s in shown {
            assert!(board.contains(&format!("\"{s}\"")), "not in the draft: {s}");
        }
        let puzzle = (n.offset, n.wait_clock.clone());
        for (d, clock) in n.marked.iter().chain([&puzzle]) {
            assert!(board.contains(&format!("d = {d} ({clock})")), "{d} {clock}");
        }
    }

    /// The fixed words carry no digits: every number on screen is read from
    /// the facts.
    #[test]
    fn the_fixed_words_carry_no_numbers() {
        let mut fixed = vec![
            HOME,
            STOP,
            STATION,
            ON_TIME,
            AT_RANDOM,
            ASK,
            AREA,
            MIN,
            SQUARED,
            ARROW,
            TIMES,
            PLUS,
            EQUALS,
            DIVIDED,
            AVERAGE_WAIT,
            SLIDER,
            AVERAGE_COST,
            ALLOW,
            K10_HEAD,
            K10_TAIL,
            NAMES_LABEL,
            RECEIPTS_TITLE,
        ];
        fixed.extend(NAMES);
        fixed.extend(RECEIPTS);
        for w in fixed {
            assert!(!w.chars().any(|c| c.is_ascii_digit()), "{w}");
        }
    }

    /// The texts up together stay apart and on screen, nothing printed over
    /// the line reaches into the triangles, and every card fits its band.
    /// Widths are estimated, not measured, as chapter 11's test estimates
    /// them: 0.0045 units a point a character, a line 0.6 of its size tall.
    /// "every bus on time" (12_8, 12_9) stands at 01's place, over the
    /// triangles dimmed to [`TRI_DIM`]; it is checked against the texts, not
    /// the triangles.
    #[test]
    fn the_picture_stays_apart_and_on_screen() {
        let n = the_last();
        type Box4 = (f64, f64, f64, f64);
        let w = |t: &str, pt: f64| t.chars().count() as f64 * pt * 0.0045;
        let h = |pt: f64| pt * 0.0045 / 0.55 * 0.6;
        let centred = |t: &str, pt: f64, x: f64, y: f64| -> Box4 {
            (x - w(t, pt) / 2.0, x + w(t, pt) / 2.0, y - h(pt), y + h(pt))
        };
        let right = |t: &str, pt: f64, x: f64, y: f64| -> Box4 { (x - w(t, pt), x, y - h(pt), y + h(pt)) };
        let dot = |x: f64, y: f64, r: f64| -> Box4 { (x - r, x + r, y - r, y + r) };
        let apart = |a: Box4, b: Box4| a.1 < b.0 || b.1 < a.0 || a.3 < b.2 || b.3 < a.2;
        let on_screen = |a: Box4| a.0 > -7.1 && a.1 < 7.1 && a.2 > -4.0 && a.3 < 4.0;
        // private elsewhere: draw's clock labels (LABEL_DY, LABEL_SIZE),
        // markers' half width, your ring and the think ring's radius;
        // timetable's bracket labels (BRACKET_LABEL_Y, _SIZE)
        let (clock_dy, clock_size, marker_half, ring_r, think_r) = (0.32, 18.0, 0.07, 0.16, 0.62);
        let (bracket_label_y, bracket_size) = (-0.22, 20.0);
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
        // the hour, its buses, the passengers in a row or in 11's stacks
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
        let dot_y = timetable::DOT_Y;
        let row: Box4 = (
            x_of(ready[0]) - r,
            x_of(*ready.last().unwrap()) + r,
            dot_y - r,
            dot_y + r,
        );
        let deepest = n.ranks.iter().max().copied().unwrap_or(0) as f64;
        let stacks: Box4 = (row.0, row.1, dot_y - deepest * 0.125 - r, dot_y + r);
        let caption = centred(AT_RANDOM, CAPTION_SIZE, CAPTION_AT[0], CAPTION_AT[1]);
        let bracket_labels: Box4 = (
            LINE_X0,
            LINE_X1,
            bracket_label_y - h(bracket_size),
            bracket_label_y + h(bracket_size),
        );
        let ask = centred(ASK, ASK_SIZE, ASK_AT[0], ASK_AT[1]);
        let ring = dot(THINK_AT[0], THINK_AT[1], think_r);
        let note = centred(PAUSE_NOTE, 18.0, THINK_AT[0], THINK_AT[1] - think_r - 0.3);
        // 12_3, 12_4
        let rows: Vec<Box4> = [n.area_row(0), n.area_row(1), n.sum_row()]
            .iter()
            .zip(ROW_YS)
            .map(|(t, y)| centred(t, ROW_SIZE, ROW_X, y))
            .collect();
        let stretch_label = centred(
            &n.stretch_label(),
            STRETCH_SIZE,
            x_at(PERIOD as f64 / 2.0),
            STRETCH_LABEL_Y,
        );
        let steps = [
            centred(&n.division(), STEP_SIZE, STEP_X, STEP_YS[0]),
            centred(&units(), UNIT_SIZE, STEP_X, STEP_YS[1]),
            centred(&n.mean_step(), STEP_SIZE, STEP_X, STEP_YS[2]),
        ];
        let words = centred(&n.clock_step(), STEP_SIZE, STEP_X, STEP_YS[3]);
        let verdict: Box4 = (
            words.0 - BOX_PAD[0],
            words.1 + BOX_PAD[0],
            words.2 - BOX_PAD[1],
            words.3 + BOX_PAD[1],
        );
        // 12_5: 07's two axes (they meet at the plot's corner, so each is
        // checked against the texts, not against the other), its ticks, the
        // slider's name and the four labels
        let x_end = plot_at(PERIOD as f64, 0.0);
        let y_end = plot_at(0.0, wait_of(0.0));
        let axes: [Box4; 2] = [
            (PLOT_O[0], x_end[0], PLOT_O[1], PLOT_O[1]),
            (PLOT_O[0], PLOT_O[0], PLOT_O[1], y_end[1]),
        ];
        let mut plot = Vec::new();
        for (d, label) in &n.ticks {
            let at = plot_at(*d as f64, 0.0);
            plot.push(centred(label, PLOT_SIZE, at[0], at[1] - PLOT_TICK_DY));
        }
        plot.push(centred(
            SLIDER,
            PLOT_SIZE,
            (PLOT_O[0] + x_end[0]) / 2.0,
            PLOT_O[1] - PLOT_NAME_DY,
        ));
        let mut labels = Vec::new();
        let puzzle = (n.offset, n.wait_clock.clone());
        for (d, clock) in n.marked.iter().chain([&puzzle]) {
            let (at, edge) = mark_at(&n, *d);
            labels.push(if edge == RIGHT {
                right(clock, MARK_SIZE, at[0], at[1])
            } else {
                centred(clock, MARK_SIZE, at[0], at[1])
            });
        }
        // each label clear of the U: no curve point inside it
        for (i, &l) in labels.iter().enumerate() {
            for c in curve(0.0, PERIOD as f64) {
                assert!(
                    !(l.0 < c[0] && c[0] < l.1 && l.2 < c[1] && c[1] < l.3),
                    "label {i} sits on the U"
                );
            }
        }
        // 12_6 to 12_9
        let cost = centred(&n.cost(), COST_SIZE, COST_AT[0], COST_AT[1]);
        let leaves = model::route(OFFSET)[0];
        let bus = leaves + n.allowance;
        let you_out = dot(x_of(Q::int(leaves)), BAR_Y, ring_r);
        let bar: Box4 = (
            x_of(Q::int(leaves)),
            x_of(Q::int(bus)),
            BAR_Y - BAR_H / 2.0,
            BAR_Y + BAR_H / 2.0,
        );
        let allow = centred(
            &n.allow(),
            ALLOW_SIZE,
            (x_of(Q::int(leaves)) + x_of(Q::int(bus))) / 2.0,
            BAR_Y + BAR_LABEL_DY,
        );
        let on_time_y = marker_top(0)[1] + ON_TIME_DY;
        let on_time_row: Box4 = (LINE_X0 - r, LINE_X1 + r, on_time_y - r, on_time_y + r);
        let on_time_words = centred(ON_TIME, ON_TIME_SIZE, ON_TIME_AT[0], ON_TIME_AT[1]);
        // the bands (`props::lower_third`): foot 0.2 off the frame's, 0.3 +
        // 0.36 a line tall, wrapped word by word at 92 characters
        let band = |text: &str| -> Box4 {
            let mut lines: Vec<String> = Vec::new();
            for word in text.split_whitespace() {
                match lines.last_mut() {
                    Some(l) if l.chars().count() + 1 + word.chars().count() <= 92 => {
                        l.push(' ');
                        l.push_str(word);
                    }
                    _ => lines.push(word.to_owned()),
                }
            }
            assert!(lines.len() <= 3, "{text} is cut short");
            (-6.71, 6.71, -3.8, -3.8 + 0.3 + 0.36 * lines.len() as f64)
        };
        let k10 = band(&n.k10());
        let named = band(&names());
        // the receipts card (`props::receipts_card`), alone: every line fits
        let card_w = 2.0 * 7.111 - 2.0;
        for line in RECEIPTS {
            assert!(w(line, 26.0) < card_w - 0.8, "{line}");
        }
        // the street's ground, its labels and you in it, over the triangles
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
        let hour = [markers, clocks];
        let moments: Vec<Vec<Box4>> = vec![
            // 12_1: the stacks, as 11 left them
            [hour.to_vec(), vec![stacks]].concat(),
            // 12_2: the row, the caption, the brackets, the question, the ring
            [hour.to_vec(), vec![row, caption, bracket_labels, ask, ring, note]].concat(),
            // 12_3, 12_4: the rows, the stretch, the step
            [
                hour.to_vec(),
                vec![row, caption, stretch_label, verdict],
                rows.clone(),
                steps.to_vec(),
            ]
            .concat(),
            // 12_5: the plot with the step still up
            [
                hour.to_vec(),
                vec![row, caption, stretch_label, verdict],
                rows.clone(),
                steps.to_vec(),
                plot.clone(),
                labels.clone(),
            ]
            .concat(),
            // 12_6: the strip and the cost
            [hour.to_vec(), vec![strip, cost]].concat(),
            // 12_7: the bar and its label; you, at the bar's left end, are
            // checked below
            [hour.to_vec(), vec![strip, cost, bar, allow]].concat(),
            // 12_8: every bus on time, K10
            [
                hour.to_vec(),
                vec![strip, cost, bar, allow, on_time_row, on_time_words, k10],
            ]
            .concat(),
            // 12_9: the passengers back, the names
            [hour.to_vec(), vec![strip, cost, row, on_time_row, on_time_words, named]].concat(),
        ];
        for (m, up) in moments.iter().enumerate() {
            for (i, &a) in up.iter().enumerate() {
                assert!(on_screen(a), "moment {m}: item {i} off screen: {a:?}");
                for (j, &b) in up.iter().enumerate().skip(i + 1) {
                    assert!(apart(a, b), "moment {m}: items {i} and {j} overlap: {a:?} {b:?}");
                }
            }
        }
        // the highest a triangle reaches over [x0, x1] with the red bus `d`
        // minutes after the blue: its slant falls a unit for a unit across
        let tri_top = |d: i64, x0: f64, x1: f64| -> f64 {
            timetable::gaps(d)
                .iter()
                .filter(|g| x_at(g.from as f64) <= x1 && x0 <= x_at(g.to as f64))
                .map(|g| LINE_Y + x_at(g.to as f64) - x0.max(x_at(g.from as f64)))
                .fold(LINE_Y, f64::max)
        };
        for (i, &a) in rows.iter().enumerate() {
            let top = tri_top(n.offset, a.0, a.1);
            assert!(a.2 > top, "row {i} reaches into the triangles: {a:?} under {top:.3}");
        }
        let mut street = vec![ground, you_in_strip];
        street.extend(&street_labels);
        for (i, &a) in street.iter().enumerate() {
            let top = tri_top(OFFSET, a.0, a.1);
            assert!(
                a.2 > top,
                "street item {i} reaches into the triangles: {a:?} under {top:.3}"
            );
        }
        // the axes clear of every text up with them
        let mut texts = vec![row, caption, stretch_label, verdict];
        texts.extend(&rows);
        texts.extend(steps);
        texts.extend(&plot);
        texts.extend(&labels);
        for (i, &a) in texts.iter().enumerate() {
            for axis in axes {
                assert!(apart(a, axis), "text {i} crosses an axis: {a:?} {axis:?}");
            }
        }
        // you, out of the strip at the bar's left end, clear of its label,
        // the clock labels and the markers
        for b in [allow, clocks, markers, cost] {
            assert!(apart(you_out, b), "you overlap {b:?}");
        }
    }
}
