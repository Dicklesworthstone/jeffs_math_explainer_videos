//! The picture chapters 04 and 05 share: one twelve-minute stretch of our
//! timetable, zoomed in (04_1) until a minute is half a unit, so that each
//! passenger's wait can stand on the line as a bar (04) and the plot of the
//! wait can be drawn over the same line, a minute up as long as a minute
//! across (05):
//! - the stretch's line, its three buses and their clock labels;
//! - a faint tick at every minute;
//! - the two gaps' brackets and lengths under the line;
//! - the twelve passengers as dots on the line, in their gaps' colors;
//! - a bar standing on each passenger, as tall as its wait.
//!
//! The buses, gaps, passengers and waits all come from `model` (the gaps
//! through `timetable`), and [`numbers`] checks every number the two
//! chapters print against them: counts, waits, sums, averages, areas,
//! ratios and the tiling. Builders return detached mobjects, like
//! `draw.rs`.

use explainer_kit::{Facts, Frame2, Kit, Palette, Timed, rect, v};
use fmn::prelude::*;

use super::draw::{LINE_Y, PASSENGER_R, bus_marker, fail, flat, line_style};
use super::timetable::{self, Bracket, Gap, gap_color};
use crate::facts::{csv, short_and_long};
use crate::model::{self, HOUR, OFFSET, PERIOD, Q};
use crate::palette;

// ------------------------------------------------------------------ layout

/// A minute in scene units, across and up alike: chapter 05's triangles are
/// then as tall on screen as they are wide, as they are in minutes.
pub const UNIT: f64 = 0.5;
/// The stretch's first moment, on its line: chapter 05's origin, left of
/// the panel's column.
pub const ORIGIN_X: f64 = -6.2;
pub const AXIS_Y: f64 = -1.6;
/// Under the line: the clock labels, then each gap's bracket and its length
/// (the hour's rows in `draw.rs` and `timetable.rs`, moved down with the
/// line).
const CLOCK_DY: f64 = 0.32;
const CLOCK_SIZE: f64 = 18.0;
const TICK_H: f64 = 0.1;
const BRACKET_DY: f64 = 0.6;
const BRACKET_TICK: f64 = 0.1;
const BRACKET_INSET: f64 = 0.04;
const BRACKET_W: f64 = 3.0;
const BRACKET_LABEL_DY: f64 = 0.82;
const BRACKET_LABEL_SIZE: f64 = 20.0;
/// A passenger's bar before 05_4 widens it to its whole minute: thin, so
/// the twelve read as separate waits.
pub const BAR_W: f64 = 0.1;
const BAR_O: f64 = 0.75;
/// Drawing order: the bars over chapter 05's triangles, the passengers over
/// their bars.
const BAR_Z: i32 = 1;
const DOT_Z: i32 = 2;
/// The panel right of the stretch: a column of rows from the top.
pub const PANEL_X: f64 = 3.9;
const PANEL_TOP: f64 = 3.1;
const ROW_H: f64 = 0.65;

// the fixed words (numbers are read from the facts)
pub const MIN: &str = "min";

/// The stretch as a data window: across, the moment you turn up, from the
/// blue bus to the next; up, a wait, up to the longest (the long gap). Both
/// scales are [`UNIT`].
pub fn frame() -> Frame2 {
    let (across, up) = (PERIOD as f64, model::longest_wait(OFFSET) as f64);
    Frame2 {
        center: v(ORIGIN_X + UNIT * across / 2.0, AXIS_Y + UNIT * up / 2.0),
        width: UNIT * across,
        height: UNIT * up,
        x: (0.0, across),
        y: (0.0, up),
    }
}

/// Moment `t` of the stretch (minutes after its first bus) with a wait of
/// `w` minutes above it, on screen.
pub fn at(t: f64, w: f64) -> Vec3 {
    frame().p(t, w)
}

/// A frame's scale in scene units a minute, refused unless a minute up is
/// as long as a minute across.
pub fn equal_scale(fr: &Frame2) -> Result<f64, String> {
    let across = fr.width / (fr.x.1 - fr.x.0);
    let up = fr.height / (fr.y.1 - fr.y.0);
    if (across - up).abs() > 1e-9 {
        return Err(format!("a minute is {across} across but {up} up"));
    }
    Ok(across)
}

/// The panel's row `i`, from the top.
pub fn row(i: usize) -> f64 {
    PANEL_TOP - ROW_H * i as f64
}

// ------------------------------------------------------------- the stretch

/// A bus's color by its route: blue for the route from :00, red otherwise.
pub fn bus_color(t: i64) -> Srgb {
    if model::route(0).contains(&t) {
        palette::blue_bus()
    } else {
        palette::red_bus()
    }
}

/// The stretch's buses in order, the blue bus at its start, the red bus and
/// the next blue bus at its end, by their minutes.
pub fn buses() -> Vec<i64> {
    model::bus_times(OFFSET).into_iter().filter(|&t| t <= PERIOD).collect()
}

/// The stretch's two gaps in order, the short gap and the long.
pub fn gaps() -> Vec<Gap> {
    timetable::gaps(OFFSET).into_iter().filter(|g| g.to <= PERIOD).collect()
}

/// A passenger of the stretch: when they turn up, how long they wait, and
/// the gap they stand in (its place in [`gaps`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Passenger {
    pub t: Q,
    pub wait: Q,
    pub gap: usize,
}

/// The stretch's passengers in order: the hour's passengers before its
/// next blue bus, one in the middle of every minute.
pub fn passengers() -> Result<Vec<Passenger>, String> {
    let gs = gaps();
    model::passengers()
        .into_iter()
        .filter(|&t| t < Q::int(PERIOD))
        .map(|t| {
            let gap = gs
                .iter()
                .position(|g| g.holds(t))
                .ok_or_else(|| format!("the passenger at {t} min stands in no gap of the stretch"))?;
            Ok(Passenger {
                t,
                wait: model::wait_at(OFFSET, t),
                gap,
            })
        })
        .collect()
}

/// The places (in [`passengers`] order) of the passengers in gap `g`.
pub fn in_gap(people: &[Passenger], g: usize) -> Vec<usize> {
    (0..people.len()).filter(|&i| people[i].gap == g).collect()
}

/// The mobjects at `places`, in that order.
pub fn pick(mobs: &[Mob], places: &[usize]) -> Vec<Mob> {
    places.iter().map(|&i| mobs[i]).collect()
}

/// Minutes as the screen writes a wait: whole minutes bare, a half or a
/// third as one glyph after the whole ("1½", "½", "4⅓"). Any other fraction
/// is refused: the glyph set (`glyphs.rs`) has no other the waits need.
pub fn mixed(q: Q) -> Result<String, String> {
    if q < Q::int(0) {
        return Err(format!("{q} min is negative"));
    }
    let whole = q.num() / q.den();
    let part = match (q.num() % q.den(), q.den()) {
        (0, _) => "",
        (1, 2) => "½",
        (1, 3) => "⅓",
        _ => return Err(format!("{q} has no glyph for its fraction")),
    };
    Ok(match (whole, part) {
        (w, "") => w.to_string(),
        (0, p) => p.to_owned(),
        (w, p) => format!("{w}{p}"),
    })
}

// ------------------------------------------------------------- the numbers

/// Every number chapters 04 and 05 print, read from the facts and checked
/// against the stretch it describes. A list runs in the gaps' order, or in
/// the passengers' order.
pub struct Numbers {
    /// The stretch's twelve minutes come five times in the hour.
    pub cycles: String,
    /// Under each bus, its minute: 0, 2, 12.
    pub clocks: Vec<String>,
    /// Each gap's length: 2, 10.
    pub gap_minutes: Vec<String>,
    /// 04: the passengers, all of them and in each gap, and each one's wait.
    pub people: String,
    pub gap_people: Vec<String>,
    pub waits: Vec<String>,
    /// Each gap's waits added up, and their average.
    pub gap_totals: Vec<String>,
    pub gap_means: Vec<String>,
    /// Your two mornings of 01_4 (the long gap's first and last passengers,
    /// by their places in [`passengers`]) and their waits.
    pub mornings: [usize; 2],
    pub morning_waits: [String; 2],
    /// All the waiting, and its average over the passengers in minutes and
    /// as a clock.
    pub total: String,
    pub mean: String,
    pub mean_clock: String,
    /// The guess of 01_5, half of the average gap, and the other way to the
    /// same number: the gaps' average waits added and divided by the number
    /// of gaps.
    pub mean_gap: String,
    pub half_mean_gap: String,
    pub gap_count: String,
    /// The long gap's passengers over the short gap's.
    pub people_ratio: String,
    /// 05: a triangle's area is half its width times its height; each gap's
    /// triangle, and both.
    pub half: String,
    pub areas: Vec<String>,
    pub total_area: String,
    /// The long triangle over the short: how much wider, and how much more
    /// waiting.
    pub width_ratio: String,
    pub area_ratio: String,
    /// The small triangles that tile the large one, upright and upside down.
    pub tiles: [String; 2],
    /// A gap doubled, and how many times the waiting it then holds.
    pub double: String,
    pub double_area_ratio: String,
}

/// The two chapters' numbers, refused when a fact disagrees with the
/// stretch: its buses and gaps, its passengers and their waits (the facts'
/// midpoint waits), the sums and averages, the triangles' areas (equal to
/// the bars' waiting, gap by gap), the ratios and the tiling counts.
pub fn numbers(f: &Facts) -> Result<Numbers, String> {
    let get = |key: &str| f.get(key).map(str::to_owned).ok_or_else(|| format!("no fact {key}"));
    // a fact that must be what the stretch has
    let drawn = |key: &str, value: String| -> Result<String, String> {
        let printed = get(key)?;
        if printed == value {
            Ok(printed)
        } else {
            Err(format!("{key} is {printed}, but the stretch has {value}"))
        }
    };
    let gs = gaps();
    let people = passengers()?;
    let lengths: Vec<i64> = gs.iter().map(Gap::minutes).collect();
    let (short, long) = short_and_long();
    if lengths != [short, long] {
        return Err(format!(
            "the stretch's gaps are {lengths:?}, not the short gap then the long"
        ));
    }

    // the stretch
    let period = drawn("F-T1.period", PERIOD.to_string())?;
    let cycles = drawn("F-T1.cycles_per_hour", (HOUR / PERIOD).to_string())?;
    let first_blue = get("F-T1.blue_times")?.split(',').next().unwrap_or_default().to_owned();
    let clocks = vec![first_blue, get("F-T1.offset")?, period.clone()];
    let minutes: Vec<String> = buses().iter().map(i64::to_string).collect();
    if clocks != minutes {
        return Err(format!(
            "the clock labels {clocks:?} are not the stretch's buses {minutes:?}"
        ));
    }
    let gap_minutes = vec![
        drawn("F-T1.short", short.to_string())?,
        drawn("F-T1.long", long.to_string())?,
    ];
    let printed_gaps = get("F-T1.gaps")?;
    if printed_gaps != csv(&lengths) {
        return Err(format!("F-T1.gaps is {printed_gaps}, not the stretch's gaps"));
    }

    // 04: the passengers and their waits
    let count = |g: usize| in_gap(&people, g).len();
    let waits_in = |g: usize| -> Vec<Q> { in_gap(&people, g).into_iter().map(|i| people[i].wait).collect() };
    for (g, &m) in lengths.iter().enumerate() {
        if waits_in(g) != model::midpoint_waits(m) {
            return Err(format!(
                "the waits in the {m}-minute gap are not one at the middle of each minute"
            ));
        }
    }
    let people_n = drawn("F-W2.people", people.len().to_string())?;
    let gap_people = vec![
        drawn("F-W2.short_people", count(0).to_string())?,
        drawn("F-W2.long_people", count(1).to_string())?,
    ];
    drawn("F-W2.short_waits", csv(waits_in(0)))?;
    let long_waits = waits_in(1);
    let (Some(&first), Some(&last)) = (long_waits.first(), long_waits.last()) else {
        return Err("the long gap holds no passenger".into());
    };
    drawn("F-W2.long_waits_range", csv([first, last]))?;
    let waits: Vec<String> = people.iter().map(|p| mixed(p.wait)).collect::<Result<_, _>>()?;
    let total_in = |g: usize| waits_in(g).into_iter().sum::<Q>();
    let mean_in = |g: usize| total_in(g) / Q::int(count(g) as i64);
    let gap_totals = vec![
        drawn("F-W2.short_total", total_in(0).to_string())?,
        drawn("F-W2.long_total", total_in(1).to_string())?,
    ];
    let gap_means = vec![
        drawn("F-W1.short_mean_wait", mean_in(0).to_string())?,
        drawn("F-W1.long_mean_wait", mean_in(1).to_string())?,
    ];
    let in_long = in_gap(&people, 1);
    let (Some(&unlucky), Some(&lucky)) = (in_long.first(), in_long.last()) else {
        return Err("the long gap holds no passenger".into());
    };
    drawn("F-M1.unlucky_at", people[unlucky].t.to_string())?;
    drawn("F-M1.lucky_at", people[lucky].t.to_string())?;
    let morning_waits = [
        drawn("F-M1.unlucky_wait_clock", model::clock(people[unlucky].wait)?)?,
        drawn("F-M1.lucky_wait_clock", model::clock(people[lucky].wait)?)?,
    ];

    // 04: all the waiting, its average, and the guess
    let all: Q = people.iter().map(|p| p.wait).sum();
    let total = drawn("F-W2.total", all.to_string())?;
    let average = all / Q::int(people.len() as i64);
    drawn("F-W2.mean_wait", average.to_string())?;
    if model::mean_wait(&lengths) != average {
        return Err(format!(
            "the passengers' average, {average}, is not the average over moments"
        ));
    }
    drawn("F-W1.mean_wait", average.to_string())?;
    let mean = mixed(average)?;
    let mean_clock = drawn("F-W1.mean_wait_clock", model::clock(average)?)?;
    let half = Q::new(1, 2);
    let gap_average = model::mean_gap(&lengths);
    let mean_gap = drawn("F-T1.mean_gap", gap_average.to_string())?;
    let half_mean_gap = drawn("F-T1.half_mean_gap", (half * gap_average).to_string())?;
    let naive = (mean_in(0) + mean_in(1)) / Q::int(gs.len() as i64);
    drawn("F-W1.naive_mean_of_waits", naive.to_string())?;
    if naive != half * gap_average {
        return Err("the average of the gaps' average waits is not half the average gap".into());
    }
    let people_ratio = drawn(
        "F-W2.people_ratio",
        Q::new(count(1) as i64, count(0) as i64).to_string(),
    )?;

    // 05: the triangles hold the bars' waiting, gap by gap
    for (g, (gap, &m)) in gs.iter().zip(&lengths).enumerate() {
        let area = model::triangle_area(m);
        if area != half * Q::int(m) * Q::int(m) {
            return Err(format!("the {m}-minute triangle is not half of {m} times {m}"));
        }
        if total_in(g) != area {
            return Err(format!("the {m}-minute gap's bars do not hold its triangle's area"));
        }
        // as tall as it is wide: the wait just after the gap's first bus
        if model::wait_at(OFFSET, Q::int(gap.from)) != Q::int(m) {
            return Err(format!("the {m}-minute triangle is not {m} minutes tall"));
        }
    }
    if get("F-A1.bars_equal_triangles")? != "true" {
        return Err("F-A1.bars_equal_triangles is not true".into());
    }
    let areas = vec![
        drawn("F-A1.short_area", model::triangle_area(short).to_string())?,
        drawn("F-A1.long_area", model::triangle_area(long).to_string())?,
    ];
    let total_area = drawn(
        "F-A1.total_area",
        (model::triangle_area(short) + model::triangle_area(long)).to_string(),
    )?;
    let wider = Q::new(long, short);
    let width_ratio = drawn("F-A1.width_ratio", wider.to_string())?;
    drawn("F-A1.height_ratio", wider.to_string())?;
    let more = model::triangle_area(long) / model::triangle_area(short);
    let area_ratio = drawn("F-A1.area_ratio", more.to_string())?;
    if more != wider * wider {
        return Err("the long triangle's waiting is not its width ratio times its height ratio".into());
    }
    if long % short != 0 {
        return Err(format!("the {short}-minute triangle cannot tile the {long}-minute one"));
    }
    let (up, down) = model::tiling((long / short) as u64);
    let tiles = [
        drawn("F-A1.tiles.upright", up.to_string())?,
        drawn("F-A1.tiles.inverted", down.to_string())?,
    ];
    if Q::int((up + down) as i64) != more {
        return Err("the tiles are not the long triangle's waiting over the short's".into());
    }
    let double = get("F-A1.double")?;
    let k: i64 = double.parse().map_err(|e| format!("F-A1.double {double}: {e}"))?;
    if k <= 0 || long % k != 0 {
        return Err(format!("the {long}-minute gap is not {double} equal gaps"));
    }
    let quadruple = model::triangle_area(long) / model::triangle_area(long / k);
    let double_area_ratio = drawn("F-A1.double_area_ratio", quadruple.to_string())?;
    let (up, down) = model::tiling(k as u64);
    if Q::int((up + down) as i64) != quadruple {
        return Err("the halves' tiles are not the doubled gap's waiting".into());
    }

    Ok(Numbers {
        cycles,
        clocks,
        gap_minutes,
        people: people_n,
        gap_people,
        waits,
        gap_totals,
        gap_means,
        mornings: [unlucky, lucky],
        morning_waits,
        total,
        mean,
        mean_clock,
        mean_gap,
        half_mean_gap,
        gap_count: gs.len().to_string(),
        people_ratio,
        half: mixed(half)?,
        areas,
        total_area,
        width_ratio,
        area_ratio,
        tiles,
        double,
        double_area_ratio,
    })
}

// ---------------------------------------------------------------- builders

/// A bracket under the stretch's gap from minute `from` to `to`, drawn as
/// `timetable::bracket_points` draws one under the hour.
pub fn bracket_points(from: f64, to: f64) -> Vec<Vec3> {
    let y = AXIS_Y - BRACKET_DY;
    let (a, b) = (at(from, 0.0)[0] + BRACKET_INSET, at(to, 0.0)[0] - BRACKET_INSET);
    vec![v(a, y + BRACKET_TICK), v(a, y), v(b, y), v(b, y + BRACKET_TICK)]
}

/// The bracket of the stretch's `gap`, labelled `label` (a fact's printed
/// value) in the gap's color. A label that is not the gap's length is
/// refused.
pub fn bracket(kit: &Kit, stage: &mut Stage<'_>, gap: &Gap, label: &str) -> fmn::Result<Bracket> {
    if label != gap.minutes().to_string() {
        return Err(fail(format!(
            "the gap from {} min is {} min, not {label}",
            gap.from,
            gap.minutes()
        )));
    }
    let color = gap_color(gap.minutes());
    let line = stage.arena_mut().add(
        polyline(bracket_points(gap.from as f64, gap.to as f64))
            .style(line_style(color, BRACKET_W))
            .build(),
    );
    let middle = at((gap.from + gap.to) as f64 / 2.0, 0.0)[0];
    let label = kit.text(
        stage,
        label,
        BRACKET_LABEL_SIZE,
        color,
        v(middle, AXIS_Y - BRACKET_LABEL_DY),
    )?;
    Ok(Bracket { line, label })
}

/// The bus at minute `t` of the stretch: `draw.rs`'s marker in its route's
/// color, standing on the stretch's line as it stands on the hour's.
pub fn marker(stage: &mut Stage<'_>, t: i64) -> fmn::Result<Mob> {
    let m = bus_marker(stage, t, bus_color(t))?;
    let lift = stage.get_bounding_box_point(m, DOWN)[1] - LINE_Y;
    stage.move_to(m, v(at(t as f64, 0.0)[0], AXIS_Y + lift), DOWN);
    Ok(m)
}

/// The stretch, detached.
pub struct Stretch {
    pub axis: Mob,
    /// The buses' markers, in [`buses`] order.
    pub markers: Vec<Mob>,
    /// Under each marker its minute, the last with the unit: "12 min".
    pub clocks: Vec<Mob>,
    /// A faint tick at every whole minute.
    pub ticks: Vec<Mob>,
    /// Each gap's bracket and length, in [`gaps`] order.
    pub brackets: Vec<Bracket>,
    /// The passengers on the line, in [`passengers`] order.
    pub dots: Vec<Mob>,
}

impl Stretch {
    /// The brackets' lines and labels.
    pub fn bracket_mobs(&self) -> Vec<Mob> {
        self.brackets.iter().flat_map(|b| [b.line, b.label]).collect()
    }

    /// Everything, for a chapter that opens on the stretch.
    pub fn mobs(&self) -> Vec<Mob> {
        let mut out = vec![self.axis];
        out.extend(&self.ticks);
        out.extend(&self.markers);
        out.extend(&self.clocks);
        out.extend(self.bracket_mobs());
        out.extend(&self.dots);
        out
    }
}

pub fn stretch(kit: &Kit, stage: &mut Stage<'_>, n: &Numbers, people: &[Passenger]) -> fmn::Result<Stretch> {
    let axis = stage.arena_mut().add(
        Line::new(at(0.0, 0.0), at(PERIOD as f64, 0.0))
            .style(line_style(Palette::dim(), 2.0))
            .build()?,
    );
    let times = buses();
    if n.clocks.len() != times.len() {
        return Err(fail(format!(
            "{} clock labels for {} buses",
            n.clocks.len(),
            times.len()
        )));
    }
    let mut markers = Vec::new();
    let mut clocks = Vec::new();
    for (i, (&t, label)) in times.iter().zip(&n.clocks).enumerate() {
        markers.push(marker(stage, t)?);
        let text = if i + 1 == times.len() {
            format!("{label} {MIN}")
        } else {
            label.clone()
        };
        let x = at(t as f64, 0.0)[0];
        clocks.push(kit.text(stage, &text, CLOCK_SIZE, Palette::dim(), v(x, AXIS_Y - CLOCK_DY))?);
    }
    let mut ticks = Vec::new();
    for m in 0..=PERIOD {
        let x = at(m as f64, 0.0)[0];
        ticks.push(
            stage.arena_mut().add(
                Line::new(v(x, AXIS_Y), v(x, AXIS_Y - TICK_H))
                    .style(line_style(Palette::faint(), 1.5))
                    .build()?,
            ),
        );
    }
    let gs = gaps();
    let mut brackets = Vec::new();
    for (g, label) in gs.iter().zip(&n.gap_minutes) {
        brackets.push(bracket(kit, stage, g, label)?);
    }
    let mut dots = Vec::new();
    for p in people {
        let d = stage.arena_mut().add(
            Dot::new()
                .point(at(p.t.to_f64(), 0.0))
                .radius(PASSENGER_R)
                .style(flat(gap_color(gs[p.gap].minutes()), 1.0))
                .build(),
        );
        stage.set_z_index(d, DOT_Z, true);
        dots.push(d);
    }
    Ok(Stretch {
        axis,
        markers,
        clocks,
        ticks,
        brackets,
        dots,
    })
}

/// The top of a passenger's bar: its moment, as high as its wait.
pub fn bar_top(p: &Passenger) -> Vec3 {
    at(p.t.to_f64(), p.wait.to_f64())
}

/// Each passenger's bar, standing on the line under its dot, [`BAR_W`]
/// wide and as tall as the wait, in its gap's color.
pub fn bars(stage: &mut Stage<'_>, people: &[Passenger]) -> fmn::Result<Vec<Mob>> {
    let gs = gaps();
    let mut out = Vec::new();
    for p in people {
        let foot = at(p.t.to_f64(), 0.0);
        let r = rect(
            v(foot[0] - BAR_W / 2.0, foot[1]),
            BAR_W,
            UNIT * p.wait.to_f64(),
            flat(gap_color(gs[p.gap].minutes()), BAR_O),
        )?;
        let m = stage.arena_mut().add(r);
        stage.set_z_index(m, BAR_Z, true);
        out.push(m);
    }
    Ok(out)
}

// --------------------------------------------------------------- playback

/// Bars rising from the line one after another, the whole taking `rt`.
pub fn grow_lagged(stage: &mut Stage<'_>, bars: &[Mob], rt: f64, lag: f64) -> fmn::Result<()> {
    if bars.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in bars {
        let a = fmn::animation::grow_from_edge(stage.arena_mut(), m, DOWN, None)?;
        anims.push(stage.prepare(a)?);
    }
    let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), anims, lag)?.with_run_time(rt);
    stage.play(g)?;
    Ok(())
}

/// Detached mobjects (`show`) fade in while others on screen (`flash`)
/// flash, all together over `rt` seconds: a label arriving with what it
/// names.
pub fn fade_in_flashing(stage: &mut Stage<'_>, show: &[Mob], flash: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in show {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    for &m in flash {
        let a = fmn::animation::indicate(stage.arena_mut(), m, 1.25, None)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    if !anims.is_empty() {
        stage.play_prepared(anims)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facts;

    /// A minute across is a minute up, and the frame runs from the
    /// stretch's first moment on its line to its end and to the longest
    /// wait. A frame stretched up is refused: the negative control.
    #[test]
    fn a_minute_up_is_a_minute_across() {
        let fr = frame();
        assert!((equal_scale(&fr).unwrap() - UNIT).abs() < 1e-12);
        assert_eq!(at(0.0, 0.0), v(ORIGIN_X, AXIS_Y));
        let corner = at(PERIOD as f64, model::longest_wait(OFFSET) as f64);
        assert!((corner[0] - (ORIGIN_X + UNIT * PERIOD as f64)).abs() < 1e-12);
        assert!((corner[1] - (AXIS_Y + UNIT * model::longest_wait(OFFSET) as f64)).abs() < 1e-12);
        let taller = Frame2 {
            height: fr.height * 1.2,
            ..fr
        };
        assert!(equal_scale(&taller).is_err());
    }

    /// The stretch is the first twelve minutes of our timetable: a blue
    /// bus, the red bus, the next blue bus; the short gap and the long; one
    /// passenger in the middle of each minute, each waiting until the next
    /// bus.
    #[test]
    fn the_stretch_is_the_timetables_first_twelve_minutes() {
        assert_eq!(buses(), vec![0, OFFSET, PERIOD]);
        assert_eq!(bus_color(0), palette::blue_bus());
        assert_eq!(bus_color(OFFSET), palette::red_bus());
        assert_eq!(bus_color(PERIOD), palette::blue_bus());
        let (short, long) = short_and_long();
        let lengths: Vec<i64> = gaps().iter().map(Gap::minutes).collect();
        assert_eq!(lengths, vec![short, long]);
        let people = passengers().unwrap();
        assert_eq!(people.len() as i64, PERIOD * model::PER_MINUTE);
        let buses = buses();
        for (k, p) in people.iter().enumerate() {
            assert_eq!(p.t, Q::new(2 * k as i64 + 1, 2 * model::PER_MINUTE));
            let next = buses.iter().map(|&b| Q::int(b)).find(|&b| b > p.t).unwrap();
            assert_eq!(p.wait, next - p.t, "{}", p.t);
            assert!(gaps()[p.gap].holds(p.t));
        }
    }

    /// Every number the two chapters print is the stretch's, and a fact
    /// that stops matching it is refused: the long gap's total waiting
    /// changed by a minute.
    #[test]
    fn the_numbers_are_the_stretches_and_a_wrong_fact_is_refused() {
        let f = facts::facts().unwrap();
        let n = numbers(&f).unwrap();
        assert_eq!(n.clocks, ["0", "2", "12"]);
        assert_eq!(n.gap_minutes, ["2", "10"]);
        assert_eq!(
            n.waits,
            ["1½", "½", "9½", "8½", "7½", "6½", "5½", "4½", "3½", "2½", "1½", "½"]
        );
        assert_eq!(n.gap_totals, ["2", "50"]);
        assert_eq!(n.gap_means, ["1", "5"]);
        assert_eq!(n.mornings, [2, 11]);
        assert_eq!(n.morning_waits, ["9:30", "0:30"]);
        assert_eq!(
            (n.total.as_str(), n.mean.as_str(), n.mean_clock.as_str()),
            ("52", "4⅓", "4:20")
        );
        assert_eq!(
            (n.mean_gap.as_str(), n.half_mean_gap.as_str(), n.gap_count.as_str()),
            ("6", "3", "2")
        );
        assert_eq!(n.half, "½");
        assert_eq!(n.areas, ["2", "50"]);
        assert_eq!(n.tiles, ["15", "10"]);
        assert_eq!((n.area_ratio.as_str(), n.double_area_ratio.as_str()), ("25", "4"));
        let mut wrong = Facts::new();
        for row in f.print().lines() {
            let (key, value) = row.split_once('\t').unwrap();
            wrong
                .add(key, if key == "F-W2.long_total" { "49" } else { value })
                .unwrap();
        }
        let refused = numbers(&wrong).err().expect("a wrong total must be refused");
        assert!(refused.contains("F-W2.long_total"), "{refused}");
    }

    #[test]
    fn waits_print_as_the_screen_writes_them() {
        assert_eq!(mixed(Q::new(3, 2)).unwrap(), "1½");
        assert_eq!(mixed(Q::new(1, 2)).unwrap(), "½");
        assert_eq!(mixed(Q::new(19, 2)).unwrap(), "9½");
        assert_eq!(mixed(Q::new(13, 3)).unwrap(), "4⅓");
        assert_eq!(mixed(Q::int(5)).unwrap(), "5");
        assert_eq!(mixed(Q::int(0)).unwrap(), "0");
        assert!(mixed(Q::new(2, 3)).is_err());
        assert!(mixed(Q::new(1, 6)).is_err());
        assert!(mixed(Q::new(-1, 2)).is_err());
    }

    /// Every bar stands inside the frame, and so does each widened to its
    /// whole minute (05_4): the minutes from one bus to the next.
    #[test]
    fn every_bar_stands_inside_the_frame() {
        let fr = frame();
        for p in passengers().unwrap() {
            let top = bar_top(&p);
            assert!(top[1] <= fr.top() + 1e-12, "{}", p.t);
            let half_minute = 1.0 / (2 * model::PER_MINUTE) as f64;
            assert!(at(p.t.to_f64() - half_minute, 0.0)[0] >= fr.left() - 1e-12);
            assert!(at(p.t.to_f64() + half_minute, 0.0)[0] <= fr.right() + 1e-12);
        }
        // a bar widens to its minute, never narrows
        assert!(at(1.0, 0.0)[0] - at(0.0, 0.0)[0] > BAR_W);
    }

    /// Under the line, top to bottom: the minute ticks, the clock labels
    /// (18 pt, about 0.06 each side of centre), the brackets and their
    /// lengths (20 pt), all above a two-line lower-third (its top near
    /// y = −2.78), and each bracket inside its own gap.
    #[test]
    fn the_rows_under_the_line_keep_apart() {
        const {
            assert!(AXIS_Y - TICK_H > AXIS_Y - CLOCK_DY + 0.06);
            assert!(AXIS_Y - CLOCK_DY - 0.06 > AXIS_Y - BRACKET_DY + BRACKET_TICK);
            assert!(AXIS_Y - BRACKET_DY > AXIS_Y - BRACKET_LABEL_DY + 0.07);
            assert!(AXIS_Y - BRACKET_LABEL_DY - 0.07 > -2.78);
        }
        for g in gaps() {
            let p = bracket_points(g.from as f64, g.to as f64);
            assert!(p[0][0] > at(g.from as f64, 0.0)[0] && p[3][0] < at(g.to as f64, 0.0)[0]);
        }
    }

    #[test]
    fn the_fixed_words_carry_no_numbers() {
        assert!(!MIN.chars().any(|c| c.is_ascii_digit()));
    }
}
