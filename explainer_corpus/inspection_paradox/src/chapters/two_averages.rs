//! Chapter 06, two averages. Chapter 02's two ways of counting, now with
//! numbers, on the same timetable, tags, gaps and passengers:
//! - 06_1: averaged over buses. One bar per bus, born at its tag and as
//!   tall as the tag says; the bars level off at six minutes, and the sum
//!   is written out.
//! - 06_2: each passenger answers how long its gap is: a small answer under
//!   every dot, and one bar per passenger, born at its answer. Ten say two,
//!   fifty say ten. The two rows are equally wide, so before any formula
//!   the passengers' row is visibly mostly tall.
//! - 06_3: the passengers' average gap? (a prediction: their tally is on
//!   screen, their level is not.)
//! - 06_4: eight minutes forty. The passengers' bars level off there; the
//!   sum with a sixth and five sixths; two boxes, two honest averages.
//! - 06_5: the rows go. In your long gap a moment slides through, the part
//!   of the gap still ahead of it drawn over the line: half the gap on
//!   average. Half of eight minutes forty is four minutes twenty; the exact
//!   version (K03).
//! - 06_6: the two boxes; the long gaps last longer and are landed in more;
//!   the names to search.
//! - 06_7: an average gap from inside, to scale, you in the middle: four
//!   twenty back to the last bus, four twenty on to the next, together the
//!   eight forty.
//!
//! Every number on screen is read from the facts, and `averages` checks
//! each against the timetable before anything is drawn. The bars' heights
//! and the levels come from the same model, so a level drawn is the average
//! printed.

use explainer_kit::props::{names_to_search, precisely};
use explainer_kit::{Facts, Kit, Palette, Timed, fade_all, rect, tween, v};
use fmn::prelude::*;

use super::draw::{
    LINE_Y, bus_marker, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_lagged, flat, fly, group, line_style,
    now, pause, read_s, say, shrink_to, stand, steady, think, wait_until, x_of, you,
};
use super::timetable::{
    self, DOT_Y, Gap, fade_in_with, fade_out_with, gap_color, play_lagged, recolor, tag_at, x_at, your_moment,
};
use crate::facts;
use crate::model::{self, OFFSET, Q};
use crate::palette;

pub struct TwoAverages {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are read from the facts)
const OVER_BUSES_SUM: &str = "averaged over buses:";
const OVER_PASSENGERS_SUM: &str = "averaged over passengers:";
const TIMES: &str = "×";
const DIVIDED: &str = "÷";
const MIN: &str = "min";
const SAY: &str = "say";
const BUS_ROW: &str = "one bar per bus";
const PASSENGER_ROW: &str = "one bar per passenger";
const ASK: &str = "the passengers' average gap?";
const OVER_BUSES: &str = "over buses:";
const OVER_PASSENGERS: &str = "over passengers:";
const HALF_THE_GAP: &str = "average: half the gap";
const NAMES: [&str; 2] = ["inspection paradox", "length-biased sampling"];
const SINCE: &str = "since the last bus:";
const UNTIL: &str = "until the next:";
const YOUR_GAP: &str = "your gap:";
const ON_AVERAGE: &str = "on average";
/// The exact version of 06_5's claim (K03), around its three numbers: the
/// share of a gap still ahead (g/2), the passengers' average gap and the
/// average wait.
const K03_WAITS: &str = "a moment chosen uniformly within a gap of length g waits g/";
const K03_GAP: &str = "on average; averaged over passengers the gap is";
const K03_WAIT: &str = "min, so the average wait is";

// ------------------------------------------------------------------ layout

/// The top rows: 06_1's sum, then the question and 06_4's sum in one slot
/// (the question leaves before the sum comes), then the two boxes; the
/// think ring right of them, its note clear of the tags.
const SUM_YS: [f64; 2] = [3.35, 2.7];
const SUM_SIZE: f64 = 24.0;
const THINK_AT: Vec3 = [5.7, 2.65, 0.0];
const BOX_X: f64 = 3.2;
const BOX_Y: f64 = 2.05;
const BOX_SIZE: f64 = 22.0;
const BOX_PAD: [f64; 2] = [0.2, 0.15];
const BOX_W: f64 = 2.0;
/// 06_5: the sums are gone and the boxes move up into their row.
const BOX_UP_Y: f64 = 3.3;
/// The two rows under the passengers, equally wide: the buses' (one bar a
/// bus) above, the passengers' (one bar a passenger) below, each gold first
/// as chapter 02 sorted them, a bar as tall as the gap it reports. Each
/// row's caption right-aligned left of it, its level's label right of it.
const ROWS_X0: f64 = -3.0;
const ROWS_W: f64 = 7.0;
const BAR_S: f64 = 0.09;
const BUS_BASE: f64 = -2.0;
const PERSON_BASE: f64 = -3.45;
const BUS_GAP: f64 = 0.06;
const PERSON_GAP: f64 = 0.02;
const BAR_FILL: f64 = 0.85;
/// A bar is born at its tag or answer this small, then grows into its slot.
const BORN: f64 = 0.25;
const CAPTION_X: f64 = -3.2;
const CAPTION_DYS: [f64; 2] = [0.6, 0.3];
const CAPTION_SIZE: f64 = 20.0;
const LEVEL_OVER: f64 = 0.05;
const LEVEL_W: f64 = 3.0;
const LEVEL_X: f64 = 4.15;
const LEVEL_SIZE: f64 = 20.0;
/// 06_2: each passenger's answer, small, under its dot.
const ANSWER_DY: f64 = 0.25;
const ANSWER_SIZE: f64 = 13.0;
/// 06_5: the moment rides from clear of one bus to clear of the next (its
/// ring would cross a marker nearer), then back to the middle; the label
/// over your gap, the answer under the passengers' box.
const RIDE_CLEAR: f64 = 1.2;
const SLIDE_RT: f64 = 2.6;
const MIDDLE_RT: f64 = 0.8;
const YOU_Z: i32 = 2;
const HALF_Y: f64 = 1.85;
const HALF_SIZE: f64 = 20.0;
const CARD_Y: f64 = 2.5;
const CARD_SIZE: f64 = 24.0;
/// Emphasis: an outline around a text or a box, faded in, held, faded out.
const HALO_PAD: f64 = 0.1;
const HALO_W: f64 = 3.0;
const HALO_RT: f64 = 0.9;
/// 06_7: an average gap from inside, a minute this wide, you at its middle;
/// the label behind you over the arrows, the label ahead under them, then
/// the bracket of the whole gap and its label.
const INSIDE_X: f64 = 0.0;
const INSIDE_Y: f64 = -1.75;
const INSIDE_S: f64 = 0.6;
const ARROW_FROM_YOU: f64 = 0.21;
const ARROW_TO_BUS: f64 = 0.1;
const BEHIND_DY: f64 = 0.75;
const AHEAD_DY: f64 = -0.35;
const WHOLE_DY: f64 = -0.7;
const WHOLE_TICK: f64 = 0.1;
const WHOLE_LABEL_DY: f64 = -0.95;
const INSIDE_SIZE: f64 = 20.0;

// ------------------------------------------------------------- the numbers

/// A fact's printed fraction ("26/3", "6") as a number.
fn parse_q(key: &str, printed: &str) -> Result<Q, String> {
    let int = |s: &str| s.parse::<i64>().map_err(|e| format!("{key} = {printed}: {e}"));
    match printed.split_once('/') {
        Some((n, d)) => Ok(Q::new(int(n)?, int(d)?)),
        None => Ok(Q::int(int(printed)?)),
    }
}

/// A number as the screen writes it: whole numbers bare, a fraction as one
/// glyph after the whole ("⅙", "⅚", "8⅔", "½"), as `twelve_minutes::mixed`
/// writes a wait, with the sixths and two thirds this chapter's sums need.
/// Any other fraction is refused.
pub fn written(q: Q) -> Result<String, String> {
    if q < Q::int(0) {
        return Err(format!("{q} is negative"));
    }
    let whole = q.num() / q.den();
    let part = match (q.num() % q.den(), q.den()) {
        (0, _) => "",
        (1, 2) => "½",
        (1, 3) => "⅓",
        (2, 3) => "⅔",
        (1, 6) => "⅙",
        (5, 6) => "⅚",
        _ => return Err(format!("{q} has no glyph for its fraction")),
    };
    Ok(match (whole, part) {
        (w, "") => w.to_string(),
        (0, p) => p.to_owned(),
        (w, p) => format!("{w}{p}"),
    })
}

/// Every number chapter 06 prints, as the facts print it, with the model's
/// values the picture draws.
pub struct Averages {
    /// The gaps' two lengths and how many tags say each; the buses.
    short: String,
    long: String,
    short_count: String,
    long_count: String,
    buses: String,
    /// The tag on each bus, in bus order, printed and drawn.
    tags: Vec<String>,
    tag_values: Vec<i64>,
    /// The average over buses, bare.
    bus_mean: String,
    /// The passengers in short gaps and in long, their shares as glyphs.
    people_short: String,
    people_long: String,
    short_share: String,
    long_share: String,
    /// The gap each passenger stands in, by length, in passenger order.
    people_gaps: Vec<i64>,
    /// The average over passengers, as a mixed number and spelt out.
    person_gap: String,
    person_gap_s: String,
    /// The share of a gap still ahead on average, as a glyph and as K03's
    /// divisor; the average wait spelt out.
    half: String,
    halves: String,
    wait_s: String,
    /// 06_7: the average time since the last bus, and it with the wait.
    age_s: String,
    together_s: String,
    /// K03's two fractions, as the facts print them.
    person_gap_q: String,
    wait_q: String,
    /// The levels the two rows of bars come to, and 06_7's two arrows, in
    /// minutes.
    pub bus_level: Q,
    pub person_level: Q,
    pub behind: Q,
    pub ahead: Q,
}

impl Averages {
    /// A gap's length as printed.
    fn printed(&self, minutes: i64) -> &str {
        if minutes.to_string() == self.long {
            &self.long
        } else {
            &self.short
        }
    }

    /// Which tags, and which passengers, report a long gap.
    fn long_tags(&self) -> Vec<bool> {
        self.tag_values.iter().map(|&t| t.to_string() == self.long).collect()
    }

    fn long_people(&self) -> Vec<bool> {
        self.people_gaps.iter().map(|&g| g.to_string() == self.long).collect()
    }

    /// 06_1: "averaged over buses: (5 × 2 + 5 × 10) ÷ 10 = 6 min".
    fn bus_sum(&self) -> String {
        format!(
            "{OVER_BUSES_SUM} ({} {TIMES} {} + {} {TIMES} {}) {DIVIDED} {} = {} {MIN}",
            self.short_count, self.short, self.long_count, self.long, self.buses, self.bus_mean
        )
    }

    /// 06_4: "averaged over passengers: ⅙ × 2 + ⅚ × 10 = 8⅔ min = 8 min 40 s".
    fn person_sum(&self) -> String {
        format!(
            "{OVER_PASSENGERS_SUM} {} {TIMES} {} + {} {TIMES} {} = {} {MIN} = {}",
            self.short_share, self.short, self.long_share, self.long, self.person_gap, self.person_gap_s
        )
    }

    /// The rows' tallies: "5 say 2, 5 say 10" and "10 say 2, 50 say 10".
    fn bus_tally(&self) -> String {
        format!(
            "{} {SAY} {}, {} {SAY} {}",
            self.short_count, self.short, self.long_count, self.long
        )
    }

    fn person_tally(&self) -> String {
        format!(
            "{} {SAY} {}, {} {SAY} {}",
            self.people_short, self.short, self.people_long, self.long
        )
    }

    /// The levels' labels: "6 min" and "8 min 40 s".
    fn bus_level_label(&self) -> String {
        format!("{} {MIN}", self.bus_mean)
    }

    fn person_level_label(&self) -> String {
        self.person_gap_s.clone()
    }

    /// The boxes: "over buses: 6 min", "over passengers: 8 min 40 s".
    fn bus_box(&self) -> String {
        format!("{OVER_BUSES} {}", self.bus_level_label())
    }

    fn person_box(&self) -> String {
        format!("{OVER_PASSENGERS} {}", self.person_gap_s)
    }

    /// 06_5's answer: "½ × 8 min 40 s = 4 min 20 s".
    fn card(&self) -> String {
        format!("{} {TIMES} {} = {}", self.half, self.person_gap_s, self.wait_s)
    }

    /// K03, the storyboard's exact version of 06_5, with the facts' numbers.
    fn k03(&self) -> String {
        format!(
            "{K03_WAITS}{} {K03_GAP} {} {K03_WAIT} {} {MIN}",
            self.halves, self.person_gap_q, self.wait_q
        )
    }

    /// 06_7's labels.
    fn behind_label(&self) -> String {
        format!("{SINCE} {} {ON_AVERAGE}", self.age_s)
    }

    fn ahead_label(&self) -> String {
        format!("{UNTIL} {} {ON_AVERAGE}", self.wait_s)
    }

    fn whole_label(&self) -> String {
        format!("{YOUR_GAP} {} {ON_AVERAGE}", self.together_s)
    }
}

/// "8:40" as the screen spells it, "8 min 40 s" (`draw::min_s`, here
/// without the scene error).
fn spelt(clock: &str) -> Result<String, String> {
    super::draw::min_s(clock).map_err(|e| e.to_string())
}

/// The chapter's numbers, refused when a fact the picture prints no longer
/// matches the timetable it draws.
pub fn averages(f: &Facts) -> Result<Averages, String> {
    let get = |key: &str| f.get(key).map(str::to_owned).ok_or_else(|| format!("no fact {key}"));
    let (short, long) = facts::short_and_long();
    let hour = model::hour_gaps(OFFSET);
    let tags = model::gap_before_each_bus(OFFSET);
    let printed: Vec<String> = get("F-S1.bus_gap_tags")?.split(',').map(str::to_owned).collect();
    if printed != tags.iter().map(i64::to_string).collect::<Vec<_>>() {
        return Err("the printed tags are not the gap before each bus".into());
    }
    let gaps = timetable::gaps(OFFSET);
    let gap_of = timetable::gap_of_each(&gaps)?;
    let people_gaps: Vec<i64> = gap_of.iter().map(|&g| gaps[g].minutes()).collect();
    if people_gaps.iter().any(|&g| g != short && g != long) {
        return Err("a passenger stands in a gap neither short nor long".into());
    }
    let count = |xs: &[i64], n: i64| xs.iter().filter(|&&x| x == n).count() as i64;
    let (tags_short, tags_long) = (count(&tags, short), count(&tags, long));
    let buses = tags.len() as i64;
    let people = people_gaps.len() as i64;
    let (in_short, in_long) = (count(&people_gaps, short), count(&people_gaps, long));
    let bus_level = model::mean_gap(&tags);
    let bus_written = Q::new(tags_short * short + tags_long * long, buses);
    let (short_share, long_share) = (Q::new(in_short, people), Q::new(in_long, people));
    let person_level = model::passengers_gap(&hour);
    let person_written = short_share * Q::int(short) + long_share * Q::int(long);
    let half = parse_q("F-S1.wait_share_of_gap", &get("F-S1.wait_share_of_gap")?)?;
    if half.num() != 1 {
        return Err(format!("K03 writes the share ahead as g/n, not {half}"));
    }
    let ahead = model::mean_wait(&hour);
    let behind = model::mean_since_last(&hour);
    let distinct = if bus_level == person_level { 1 } else { 2 };
    let checks = [
        ("the short gaps' length", short.to_string(), "F-T1.short"),
        ("the long gaps' length", long.to_string(), "F-T1.long"),
        ("tags saying short", tags_short.to_string(), "F-T1.short_count"),
        ("tags saying long", tags_long.to_string(), "F-T1.long_count"),
        ("buses", buses.to_string(), "F-T1.buses_per_hour"),
        ("the average over buses", bus_level.to_string(), "F-S1.bus_mean_gap"),
        ("the average over buses", bus_level.to_string(), "F-T1.mean_gap"),
        ("06_1's sum", bus_written.to_string(), "F-S1.bus_mean_gap"),
        ("passengers", people.to_string(), "F-S1.passengers"),
        (
            "passengers in short gaps",
            in_short.to_string(),
            "F-S1.passengers_in_short",
        ),
        (
            "passengers in long gaps",
            in_long.to_string(),
            "F-S1.passengers_in_long",
        ),
        ("the short share", short_share.to_string(), "F-S1.person_short_share"),
        ("the long share", long_share.to_string(), "F-S1.person_long_share"),
        (
            "the average over passengers",
            person_level.to_string(),
            "F-S1.person_gap",
        ),
        ("06_4's sum", person_written.to_string(), "F-S1.person_gap"),
        (
            "the average over passengers",
            model::clock(person_level)?,
            "F-S1.person_gap_clock",
        ),
        (
            "half the passengers' gap",
            (half * person_level).to_string(),
            "F-W1.mean_wait",
        ),
        ("the average wait", ahead.to_string(), "F-W1.mean_wait"),
        (
            "half the passengers' gap",
            model::clock(half * person_level)?,
            "F-S1.half_person_gap_clock",
        ),
        ("the time since the last bus", behind.to_string(), "F-S1.age"),
        ("the time since the last bus", model::clock(behind)?, "F-S1.age_clock"),
        (
            "behind and ahead together",
            model::clock(behind + ahead)?,
            "F-S1.age_plus_wait_clock",
        ),
        (
            "behind and ahead together",
            model::clock(behind + ahead)?,
            "F-S1.person_gap_clock",
        ),
        ("different averages", distinct.to_string(), "F-S1.averages"),
        ("the buses' bars' level", level(&tags).to_string(), "F-S1.bus_mean_gap"),
        (
            "the passengers' bars' level",
            level(&people_gaps).to_string(),
            "F-S1.person_gap",
        ),
    ];
    for (what, drawn, key) in checks {
        let shown = get(key)?;
        if drawn != shown {
            return Err(format!("{what}: the picture draws {drawn}, but {key} is {shown}"));
        }
    }
    Ok(Averages {
        short: get("F-T1.short")?,
        long: get("F-T1.long")?,
        short_count: get("F-T1.short_count")?,
        long_count: get("F-T1.long_count")?,
        buses: get("F-T1.buses_per_hour")?,
        tags: printed,
        tag_values: tags,
        bus_mean: get("F-S1.bus_mean_gap")?,
        people_short: get("F-S1.passengers_in_short")?,
        people_long: get("F-S1.passengers_in_long")?,
        short_share: written(short_share)?,
        long_share: written(long_share)?,
        people_gaps,
        person_gap: written(person_level)?,
        person_gap_s: spelt(&get("F-S1.person_gap_clock")?)?,
        half: written(half)?,
        halves: half.den().to_string(),
        wait_s: spelt(&get("F-S1.half_person_gap_clock")?)?,
        age_s: spelt(&get("F-S1.age_clock")?)?,
        together_s: spelt(&get("F-S1.age_plus_wait_clock")?)?,
        person_gap_q: get("F-S1.person_gap")?,
        wait_q: get("F-W1.mean_wait")?,
        bus_level,
        person_level,
        behind,
        ahead,
    })
}

// ----------------------------------------------------------------- picture

/// A row's slots: equal slots across the rows' width, the long ones first
/// (in their order), then the short ones, as chapter 02 sorts its rows.
/// Each is (the thing's index, its slot's centre x).
pub fn sorted_row(longs: &[bool]) -> Vec<(usize, f64)> {
    let w = ROWS_W / longs.len() as f64;
    let n = longs.len();
    let order: Vec<usize> = (0..n)
        .filter(|&i| longs[i])
        .chain((0..n).filter(|&i| !longs[i]))
        .collect();
    order
        .into_iter()
        .enumerate()
        .map(|(slot, i)| (i, ROWS_X0 + w * (slot as f64 + 0.5)))
        .collect()
}

/// Where a row of bars levels off: the height every bar would have if the
/// row held the same area evenly. Its slots are equal, so the plain average
/// of the heights.
pub fn level(heights: &[i64]) -> Q {
    Q::new(heights.iter().sum(), heights.len() as i64)
}

/// A bar `w` wide and `h` tall, born at `born` at [`BORN`] of its size, in
/// `color`, detached.
fn born_bar(stage: &mut Stage<'_>, born: Vec3, w: f64, h: f64, color: Srgb) -> fmn::Result<Mob> {
    let bar = rect(
        v(born[0] - w * BORN / 2.0, born[1] - h * BORN / 2.0),
        w * BORN,
        h * BORN,
        flat(color, BAR_FILL),
    )?;
    Ok(stage.arena_mut().add(bar))
}

/// A row's level line, `minutes` up from its `base`, a little wider than
/// the row; detached.
fn level_line(stage: &mut Stage<'_>, base: f64, minutes: Q) -> fmn::Result<Mob> {
    let y = base + minutes.to_f64() * BAR_S;
    let line = Line::new(v(ROWS_X0 - LEVEL_OVER, y), v(ROWS_X0 + ROWS_W + LEVEL_OVER, y))
        .style(line_style(Palette::ink(), LEVEL_W))
        .build()?;
    Ok(stage.arena_mut().add(line))
}

/// A row's caption, right-aligned left of the row: what a bar is (dim),
/// then the tally (ink). Detached.
fn caption(kit: &Kit, stage: &mut Stage<'_>, lines: [&str; 2], base: f64) -> fmn::Result<Vec<Mob>> {
    let mut out = Vec::with_capacity(2);
    for (i, text) in lines.into_iter().enumerate() {
        let color = if i == 0 { Palette::dim() } else { Palette::ink() };
        let at = v(CAPTION_X, base + CAPTION_DYS[i]);
        let m = kit.text(stage, text, CAPTION_SIZE, color, at)?;
        stage.move_to(m, at, RIGHT);
        out.push(m);
    }
    Ok(out)
}

/// A comparison box: its words in `color`, outlined in it, as one group;
/// detached.
fn boxed(kit: &Kit, stage: &mut Stage<'_>, text: &str, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
    let words = kit.text(stage, text, BOX_SIZE, color, at)?;
    let w = stage.get_width(words) + 2.0 * BOX_PAD[0];
    let h = stage.get_height(words) + 2.0 * BOX_PAD[1];
    let outline = rect(v(at[0] - w / 2.0, at[1] - h / 2.0), w, h, line_style(color, BOX_W))?;
    let outline = stage.arena_mut().add(outline);
    group(stage, &[outline, words])
}

/// Emphasis for texts or boxes: an outline just around each, stroked only,
/// that fades in, holds and fades out over `rt`. The words themselves are
/// left as they are (a flash would scale and recolor them).
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

/// An updater keeping you at the tracker `at`'s moment and the part of your
/// gap still ahead of you, from you to the gap's `end`, drawn over the
/// line. Returns its anchor, already in the scene.
fn ride(stage: &mut Stage<'_>, at: Mob, me: Mob, ahead: Mob, end: f64) -> fmn::Result<Mob> {
    let anchor = stage.arena_mut().add(VMobject::new());
    stage.add_updater(
        anchor,
        move |s, _| {
            let t = s.tracker_value(at).unwrap_or(0.0);
            s.move_to(me, v(x_at(t), LINE_Y), ORIGIN);
            let path = polyline(timetable::span_points(t, end)).build();
            let _ = s.set_points(ahead, path.points());
        },
        true,
    )?;
    stage.add_to_scene(anchor)?;
    Ok(anchor)
}

/// 06_7's average gap from inside: where its last bus and its next bus
/// stand, each its average away from you, a minute [`INSIDE_S`] wide.
pub fn inside_ends(a: &Averages) -> (f64, f64) {
    (
        INSIDE_X - a.behind.to_f64() * INSIDE_S,
        INSIDE_X + a.ahead.to_f64() * INSIDE_S,
    )
}

/// An arrow along 06_7's gap from `from` to `to`, in `color`; detached.
fn arrow(stage: &mut Stage<'_>, from: f64, to: f64, color: Srgb) -> fmn::Result<Mob> {
    let a = Arrow::new(v(from, INSIDE_Y), v(to, INSIDE_Y))
        .buff(0.0)
        .thickness(3.0)
        .style(Style::default().fill(color, 1.0).stroke(color, 3.0, 1.0))
        .build()?;
    Ok(stage.arena_mut().add(a))
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

impl SceneConstruct for TwoAverages {
    fn name(&self) -> &str {
        "06_two_averages"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let f = facts::facts().map_err(fail)?;
        let a = averages(&f).map_err(fail)?;
        let (_, long) = facts::short_and_long();
        let gaps = timetable::gaps(OFFSET);
        let times = model::bus_times(OFFSET);
        let people = model::passengers();
        let long_tags = a.long_tags();
        let long_people = a.long_people();
        let yours: Gap = *gaps
            .iter()
            .find(|g| g.holds(your_moment()))
            .ok_or_else(|| fail("your moment is in no gap".into()))?;
        if yours.minutes() != long {
            return Err(fail("your gap is not a long one".into()));
        }
        let (teal, gold) = (timetable::short_gap(), palette::long_gap());

        // 06_1: the hour as chapter 03 left it, without the sign: markers,
        // brackets, tags and the passengers in their gaps' colors. Averaged
        // over buses: a bar per bus born at its tag, as tall as it says;
        // the bars level off at the tags' average; the sum
        let line = say(&kit, stage, "06_1")?;
        let table = timetable::timetable(&kit, stage, &f, OFFSET)?;
        let mut brackets = Vec::with_capacity(gaps.len());
        for g in &gaps {
            brackets.push(timetable::bracket(
                &kit,
                stage,
                g,
                a.printed(g.minutes()),
                gap_color(g.minutes()),
            )?);
        }
        let tags: Vec<Mob> = times
            .iter()
            .zip(&a.tags)
            .map(|(&t, value)| timetable::tag(&kit, stage, x_of(Q::int(t)), value))
            .collect::<fmn::Result<_>>()?;
        let dots: Vec<Mob> = people
            .iter()
            .zip(&a.people_gaps)
            .map(|(&t, &g)| timetable::passenger(stage, t, gap_color(g)))
            .collect();
        let markers: Vec<Mob> = table.blue.iter().chain(&table.red).copied().collect();
        let mut opening = table.hour.mobs();
        opening.extend(&markers);
        for b in &brackets {
            opening.extend([b.line, b.label]);
        }
        opening.extend(&tags);
        opening.extend(&dots);
        fade_in_all(stage, &opening, 0.8)?;
        line.wait_for(stage, "Averaged over buses")?;
        let bus_w = ROWS_W / a.tag_values.len() as f64 - BUS_GAP;
        let mut births = Vec::with_capacity(a.tag_values.len());
        let mut bus_bars = Vec::with_capacity(a.tag_values.len());
        for (i, x) in sorted_row(&long_tags) {
            let h = a.tag_values[i] as f64 * BAR_S;
            let born = tag_at(x_of(Q::int(times[i])));
            let bar = born_bar(stage, born, bus_w, h, gap_color(a.tag_values[i]))?;
            births.push(shrink_to(bar, 1.0 / BORN, v(x, BUS_BASE + h / 2.0), 1.0)?);
            bus_bars.push(bar);
        }
        play_lagged(stage, births, 1.4, 0.15)?;
        let bus_caption = caption(&kit, stage, [BUS_ROW, &a.bus_tally()], BUS_BASE)?;
        fade_in_all(stage, &bus_caption, 0.4)?;
        line.wait_for(stage, "the tags say six minutes")?;
        let bus_level = level_line(stage, BUS_BASE, a.bus_level)?;
        let bus_level_y = BUS_BASE + a.bus_level.to_f64() * BAR_S;
        let bus_label = kit.text_left(
            stage,
            &a.bus_level_label(),
            LEVEL_SIZE,
            Palette::ink(),
            v(LEVEL_X, bus_level_y),
        )?;
        grow_with(stage, bus_level, LEFT, &[bus_label], 0.6)?;
        line.wait_for(stage, "the number on the sign")?;
        let bus_sum = kit.text(stage, &a.bus_sum(), SUM_SIZE, Palette::ink(), v(0.0, SUM_YS[0]))?;
        fade_in_all(stage, &[bus_sum], 0.5)?;
        halo(stage, &[bus_label], Palette::ink(), HALO_RT)?;

        // 06_2: each passenger's answer under its dot; the short gaps'
        // answers take their color and their bars grow from them, then the
        // long gaps'; the row's caption and tally
        let line = say(&kit, stage, "06_2")?;
        let answers: Vec<Mob> = people
            .iter()
            .zip(&a.people_gaps)
            .map(|(&t, &g)| {
                kit.text(
                    stage,
                    a.printed(g),
                    ANSWER_SIZE,
                    Palette::dim(),
                    v(x_of(t), DOT_Y - ANSWER_DY),
                )
            })
            .collect::<fmn::Result<_>>()?;
        let person_w = ROWS_W / people.len() as f64 - PERSON_GAP;
        let (mut short_births, mut long_births) = (Vec::new(), Vec::new());
        let mut person_bars = Vec::with_capacity(people.len());
        for (i, x) in sorted_row(&long_people) {
            let g = a.people_gaps[i];
            let h = g as f64 * BAR_S;
            let bar = born_bar(stage, v(x_of(people[i]), DOT_Y - ANSWER_DY), person_w, h, gap_color(g))?;
            let birth = shrink_to(bar, 1.0 / BORN, v(x, PERSON_BASE + h / 2.0), 1.0)?;
            if long_people[i] {
                long_births.push(birth);
            } else {
                short_births.push(birth);
            }
            person_bars.push(bar);
        }
        line.wait_for(stage, "how long the gap they landed in is")?;
        fade_in_lagged(stage, &answers, 1.6, 0.05)?;
        for (word, is_long, color, births, rt) in [
            ("One in six", false, teal, short_births, 1.0),
            ("Five in six", true, gold, long_births, 1.6),
        ] {
            line.wait_for(stage, word)?;
            let painted = answers
                .iter()
                .zip(&long_people)
                .filter(|&(_, &l)| l == is_long)
                .map(|(&m, _)| recolor(m, color, 0.4))
                .collect::<fmn::Result<Vec<_>>>()?;
            fade_in_with(stage, &[], painted, 0.4)?;
            play_lagged(stage, births, rt, if is_long { 0.03 } else { 0.1 })?;
        }
        let person_caption = caption(&kit, stage, [PASSENGER_ROW, &a.person_tally()], PERSON_BASE)?;
        fade_in_all(stage, &person_caption, 0.4)?;

        // 06_3: the question; think time (the tally on screen, no level)
        let line = say(&kit, stage, "06_3")?;
        line.wait_for(stage, "what's the average gap")?;
        let ask = kit.text(stage, ASK, SUM_SIZE, Palette::ink(), v(0.0, SUM_YS[1]))?;
        fade_in_all(stage, &[ask], 0.4)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 06_4: eight forty, the passengers' bars level off there; the sum
        // with the two shares; same buses and gaps; the two boxes
        let line = say(&kit, stage, "06_4")?;
        fade_out_all(stage, &[q, ask], 0.3)?;
        let person_level = level_line(stage, PERSON_BASE, a.person_level)?;
        let person_label = kit.text_left(
            stage,
            &a.person_level_label(),
            LEVEL_SIZE,
            gold,
            v(LEVEL_X, PERSON_BASE + a.person_level.to_f64() * BAR_S),
        )?;
        grow_with(stage, person_level, LEFT, &[person_label], 0.6)?;
        line.wait_for(stage, "A sixth of the passengers")?;
        let person_sum = kit.text(stage, &a.person_sum(), SUM_SIZE, gold, v(0.0, SUM_YS[1]))?;
        fade_in_all(stage, &[person_sum], 0.5)?;
        line.wait_for(stage, "Same buses, same gaps")?;
        flash_lagged(stage, &markers, 0.8, 0.05)?;
        line.wait_for(stage, "two honest averages")?;
        let boxes = [
            boxed(&kit, stage, &a.bus_box(), Palette::dim(), v(-BOX_X, BOX_Y))?,
            boxed(&kit, stage, &a.person_box(), gold, v(BOX_X, BOX_Y))?,
        ];
        fade_in_all(stage, &boxes, 0.5)?;

        // 06_5: the rows and sums go, the boxes move up; a moment rides
        // through your long gap, the part still ahead drawn over it, and
        // comes to the middle: half the gap; half of eight forty; K03
        let line = say(&kit, stage, "06_5")?;
        let mut leaving = bus_bars;
        leaving.extend(person_bars);
        leaving.extend(&answers);
        leaving.extend(&bus_caption);
        leaving.extend(&person_caption);
        leaving.extend([bus_level, bus_label, person_level, person_label, bus_sum, person_sum]);
        let up = vec![
            fly(boxes[0], v(-BOX_X, BOX_UP_Y), 0.6)?,
            fly(boxes[1], v(BOX_X, BOX_UP_Y), 0.6)?,
        ];
        fade_out_with(stage, &leaving, up, 0.6)?;
        line.wait_for(stage, "turns up")?;
        let (from, to) = (yours.from as f64 + RIDE_CLEAR, yours.to as f64);
        let at = stage.add_value_tracker(from);
        let span = timetable::span(stage, yours.from as f64, to, gold);
        let ahead = timetable::span(stage, from, to, palette::you());
        let me = you(stage, v(x_at(from), LINE_Y))?;
        stage.set_z_index(me, YOU_Z, true);
        fade_in_all(stage, &[span, ahead, me], 0.4)?;
        let rider = ride(stage, at, me, ahead, to)?;
        stage.play(steady(at, to - RIDE_CLEAR, SLIDE_RT)?)?;
        line.wait_for(stage, "waits half that gap")?;
        let middle = (yours.from + yours.to) as f64 / 2.0;
        stage.play(tween(at, middle, MIDDLE_RT)?)?;
        let half = kit.text(stage, HALF_THE_GAP, HALF_SIZE, palette::you(), v(x_at(middle), HALF_Y))?;
        let k03 = a.k03();
        let card03 = precisely(&kit, stage, &k03)?;
        fade_in_all(stage, &[half, card03], 0.5)?;
        let card_up = now(stage);
        line.wait_for(stage, "Half of eight minutes forty")?;
        halo(stage, &[boxes[1]], gold, HALO_RT)?;
        line.wait_for(stage, "four minutes twenty")?;
        let card = kit.text(stage, &a.card(), CARD_SIZE, palette::verdict(), v(BOX_X, CARD_Y))?;
        fade_in_all(stage, &[card], 0.5)?;
        line.wait_for(stage, "The same answer")?;
        halo(stage, &[card], palette::verdict(), HALO_RT)?;
        kit.hold(stage)?;
        wait_until(stage, card_up + read_s(&k03))?;
        stage.clear_updaters(rider, true);

        // 06_6: the ride and K03 go; the two boxes; the long gaps last
        // longer and hold more passengers; the names to search
        let line = say(&kit, stage, "06_6")?;
        fade_out_all(stage, &[card03, card, half, me, span, ahead], 0.5)?;
        line.wait_for(stage, "Averaging over buses")?;
        halo(stage, &[boxes[0]], Palette::ink(), HALO_RT)?;
        line.wait_for(stage, "averaging over moments")?;
        halo(stage, &[boxes[1]], gold, HALO_RT)?;
        line.wait_for(stage, "long gaps last longer")?;
        let long_brackets: Vec<Mob> = brackets
            .iter()
            .zip(&gaps)
            .filter(|(_, g)| g.minutes() == long)
            .map(|(b, _)| b.line)
            .collect();
        flash_lagged(stage, &long_brackets, 1.0, 0.15)?;
        line.wait_for(stage, "get picked more")?;
        let long_dots: Vec<Mob> = dots
            .iter()
            .zip(&long_people)
            .filter(|&(_, &l)| l)
            .map(|(&m, _)| m)
            .collect();
        flash_lagged(stage, &long_dots, 1.2, 0.03)?;
        line.wait_for(stage, "the inspection paradox")?;
        let names = names_to_search(&kit, stage, &NAMES)?;
        fade_in_all(stage, &[names], 0.5)?;
        let names_up = now(stage);
        kit.hold(stage)?;
        wait_until(stage, names_up + read_s(&NAMES.join(" · ")))?;

        // 06_7: an average gap from inside, to scale: you in the middle,
        // its last bus and its next at the ends; the arrow back, the arrow
        // on, each four twenty; together the eight forty
        let line = say(&kit, stage, "06_7")?;
        fade_out_all(stage, &[names], 0.4)?;
        let (last_x, next_x) = inside_ends(&a);
        let mut ends = Vec::with_capacity(2);
        for x in [last_x, next_x] {
            let m = bus_marker(stage, 0, Palette::dim())?;
            stage.move_to(m, v(x, INSIDE_Y + 0.02), DOWN);
            ends.push(m);
        }
        let centre = you(stage, v(INSIDE_X, INSIDE_Y))?;
        ends.push(centre);
        fade_in_all(stage, &ends, 0.5)?;
        line.wait_for(stage, "the last bus left")?;
        let back = arrow(
            stage,
            INSIDE_X - ARROW_FROM_YOU,
            last_x + ARROW_TO_BUS,
            palette::verdict(),
        )?;
        let behind = kit.text(
            stage,
            &a.behind_label(),
            INSIDE_SIZE,
            palette::verdict(),
            v((INSIDE_X + last_x) / 2.0, INSIDE_Y + BEHIND_DY),
        )?;
        grow_with(stage, back, RIGHT, &[behind], 0.8)?;
        line.wait_for(stage, "ahead of you")?;
        let on = arrow(
            stage,
            INSIDE_X + ARROW_FROM_YOU,
            next_x - ARROW_TO_BUS,
            palette::verdict(),
        )?;
        let ahead_label = kit.text(
            stage,
            &a.ahead_label(),
            INSIDE_SIZE,
            palette::verdict(),
            v((INSIDE_X + next_x) / 2.0, INSIDE_Y + AHEAD_DY),
        )?;
        grow_with(stage, on, LEFT, &[ahead_label], 0.8)?;
        line.wait_for(stage, "Together")?;
        let y = INSIDE_Y + WHOLE_DY;
        let whole = stage.arena_mut().add(
            polyline(vec![
                v(last_x, y + WHOLE_TICK),
                v(last_x, y),
                v(next_x, y),
                v(next_x, y + WHOLE_TICK),
            ])
            .style(line_style(gold, 3.0))
            .build(),
        );
        let whole_label = kit.text(
            stage,
            &a.whole_label(),
            INSIDE_SIZE,
            gold,
            v((last_x + next_x) / 2.0, INSIDE_Y + WHOLE_LABEL_DY),
        )?;
        fade_in_all(stage, &[whole, whole_label], 0.6)?;
        halo(stage, &[boxes[1]], gold, HALO_RT)?;
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

    /// Needs chapter 06's lines in script.tsv (the integration step copies
    /// them from the draft). The whole chapter runs headless at 10 fps with
    /// the scratch voice: every phrase it waits for is in its line, every
    /// fact it reads exists and matches the timetable, and its seven lines
    /// are said in order. It ends with `kit.finish`, whose timeline runs
    /// past the end of the last line.
    #[test]
    fn the_chapter_runs_and_says_its_lines_in_order() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let narrator = Narrator::estimate(&script);
        let kit = Kit::new().unwrap().with_narrator(Rc::clone(&narrator));
        let mut scene = TwoAverages { kit };
        let config = fmn::scene::RuntimeConfig {
            fps: 10,
            ..fmn::scene::RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut fmn::scene::NullSceneSink).unwrap();
        let said_at = narrator.said();
        let said: Vec<&str> = said_at.iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(said, ["06_1", "06_2", "06_3", "06_4", "06_5", "06_6", "06_7"]);
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

    /// What the chapter prints is the draft's picture, word for word: the
    /// two sums, the passengers' tally, the question, the boxes, the
    /// half-gap card and its answer, the names and 06_7's three labels.
    #[test]
    fn the_printed_words_are_the_drafts() {
        let a = averages(&facts::facts().unwrap()).unwrap();
        let board = draft();
        for shown in [
            a.bus_sum(),
            a.person_tally(),
            ASK.to_owned(),
            a.person_sum(),
            a.bus_box(),
            a.person_box(),
            HALF_THE_GAP.to_owned(),
            a.card(),
            format!("names to search: {}", NAMES.join(" · ")),
            a.behind_label(),
            a.ahead_label(),
            a.whole_label(),
        ] {
            assert!(board.contains(&format!("\"{shown}\"")), "not in the draft: {shown}");
        }
        assert_eq!(a.bus_sum(), "averaged over buses: (5 × 2 + 5 × 10) ÷ 10 = 6 min");
        assert_eq!(
            a.person_sum(),
            "averaged over passengers: ⅙ × 2 + ⅚ × 10 = 8⅔ min = 8 min 40 s"
        );
    }

    /// K03, built from the facts, is the draft's exact version of 06_5
    /// word for word (and so the storyboard's, which holds the draft
    /// verbatim once chapter 06 is integrated).
    #[test]
    fn the_card_is_the_drafts_exact_version() {
        let a = averages(&facts::facts().unwrap()).unwrap();
        let k03 = a.k03();
        assert!(draft().contains(&format!("\"{k03}\"")), "{k03}");
        assert!(k03.contains("g/2") && k03.contains("26/3 min") && k03.contains("13/3 min"));
    }

    /// The numbers are the timetable's, and a fact that stops matching it
    /// is refused: the passengers' average gap changed.
    #[test]
    fn the_numbers_are_the_timetables_and_a_wrong_fact_is_refused() {
        let f = facts::facts().unwrap();
        let a = averages(&f).unwrap();
        assert_eq!(a.bus_level, Q::int(6));
        assert_eq!(a.person_level, Q::new(26, 3));
        assert_eq!((a.behind, a.ahead), (Q::new(13, 3), Q::new(13, 3)));
        assert_eq!(a.behind + a.ahead, a.person_level);
        assert_eq!(a.bus_tally(), "5 say 2, 5 say 10");
        assert_eq!(a.person_tally(), "10 say 2, 50 say 10");
        assert_eq!(
            (a.bus_level_label(), a.person_level_label()),
            ("6 min".to_owned(), "8 min 40 s".to_owned())
        );
        for (key, bad) in [
            ("F-S1.person_gap", "9"),
            ("F-S1.passengers_in_long", "49"),
            ("F-S1.age_clock", "4:30"),
        ] {
            let mut wrong = Facts::new();
            for row in f.print().lines() {
                let (k, value) = row.split_once('\t').unwrap();
                wrong.add(k, if k == key { bad } else { value }).unwrap();
            }
            let refused = averages(&wrong).err().expect("a wrong fact must be refused");
            assert!(refused.contains(key), "{refused}");
        }
    }

    /// The two rows draw the same rule, one bar per thing, a bar as tall as
    /// its gap, across the same width: the buses' bars level off at the
    /// average over buses and the passengers' at the average over
    /// passengers. Each row has the facts' counts of short and long bars,
    /// gold first, every slot inside the rows' width.
    #[test]
    fn each_row_levels_off_at_its_average() {
        let a = averages(&facts::facts().unwrap()).unwrap();
        assert_eq!(level(&a.tag_values), a.bus_level);
        assert_eq!(level(&a.people_gaps), a.person_level);
        assert_ne!(level(&a.tag_values), level(&a.people_gaps));
        let (short, long) = facts::short_and_long();
        for (heights, longs, counts) in [
            (&a.tag_values, a.long_tags(), [&a.short_count, &a.long_count]),
            (&a.people_gaps, a.long_people(), [&a.people_short, &a.people_long]),
        ] {
            let n = |g: i64| heights.iter().filter(|&&h| h == g).count().to_string();
            assert_eq!([n(short), n(long)], [counts[0].clone(), counts[1].clone()]);
            let row = sorted_row(&longs);
            assert_eq!(row.len(), heights.len());
            let w = ROWS_W / heights.len() as f64;
            for (slot, &(i, x)) in row.iter().enumerate() {
                assert!((x - (ROWS_X0 + w * (slot as f64 + 0.5))).abs() < 1e-12);
                assert!(x - w / 2.0 >= ROWS_X0 - 1e-12 && x + w / 2.0 <= ROWS_X0 + ROWS_W + 1e-12);
                let gold_slots = longs.iter().filter(|&&l| l).count();
                assert_eq!(longs[i], slot < gold_slots, "gold first");
            }
            let mut seen: Vec<usize> = row.iter().map(|&(i, _)| i).collect();
            seen.sort_unstable();
            assert_eq!(seen, (0..heights.len()).collect::<Vec<_>>());
        }
    }

    /// 06_7's arrows are each their average long and meet at you; the
    /// bracket over both is the passengers' average gap, to scale.
    #[test]
    fn the_gap_from_inside_is_to_scale() {
        let a = averages(&facts::facts().unwrap()).unwrap();
        let (last, next) = inside_ends(&a);
        assert!((INSIDE_X - last - a.behind.to_f64() * INSIDE_S).abs() < 1e-12);
        assert!((next - INSIDE_X - a.ahead.to_f64() * INSIDE_S).abs() < 1e-12);
        assert!((next - last - a.person_level.to_f64() * INSIDE_S).abs() < 1e-12);
        assert!(INSIDE_X - ARROW_FROM_YOU > last + ARROW_TO_BUS);
    }

    #[test]
    fn numbers_print_as_the_screen_writes_them() {
        assert_eq!(written(Q::new(1, 6)).unwrap(), "⅙");
        assert_eq!(written(Q::new(5, 6)).unwrap(), "⅚");
        assert_eq!(written(Q::new(26, 3)).unwrap(), "8⅔");
        assert_eq!(written(Q::new(1, 2)).unwrap(), "½");
        assert_eq!(written(Q::new(13, 3)).unwrap(), "4⅓");
        assert_eq!(written(Q::int(6)).unwrap(), "6");
        assert!(written(Q::new(1, 5)).is_err());
        assert!(written(Q::new(-1, 6)).is_err());
        assert_eq!(parse_q("k", "26/3").unwrap(), Q::new(26, 3));
        assert_eq!(parse_q("k", "6").unwrap(), Q::int(6));
        assert!(parse_q("k", "6 min").is_err());
    }

    /// The texts up together stay apart and on screen, the rows' bars clear
    /// of the passengers' answers and of each other, the lower-thirds clear
    /// of everything up with them. Widths are estimated, not measured: 0.0045
    /// units a point a character, a line 0.6 of its size tall; a box's
    /// outline adds its padding.
    #[test]
    fn the_picture_stays_apart_and_on_screen() {
        let a = averages(&facts::facts().unwrap()).unwrap();
        let w = |s: &str, pt: f64| s.chars().count() as f64 * pt * 0.0045;
        let h = |pt: f64| pt * 0.0045 / 0.55 * 0.6;
        type Box4 = (f64, f64, f64, f64);
        let centred = |s: &str, pt: f64, x: f64, y: f64| -> Box4 {
            (x - w(s, pt) / 2.0, x + w(s, pt) / 2.0, y - h(pt), y + h(pt))
        };
        let left = |s: &str, pt: f64, x: f64, y: f64| -> Box4 { (x, x + w(s, pt), y - h(pt), y + h(pt)) };
        let right = |s: &str, pt: f64, x: f64, y: f64| -> Box4 { (x - w(s, pt), x, y - h(pt), y + h(pt)) };
        let boxed = |s: &str, x: f64, y: f64| -> Box4 {
            let b = centred(s, BOX_SIZE, x, y);
            (b.0 - BOX_PAD[0], b.1 + BOX_PAD[0], b.2 - BOX_PAD[1], b.3 + BOX_PAD[1])
        };
        let apart = |a: Box4, b: Box4| a.1 < b.0 || b.1 < a.0 || a.3 < b.2 || b.3 < a.2;
        let on_screen = |a: Box4| a.0 > -7.1 && a.1 < 7.1 && a.2 > -4.0 && a.3 < 4.0;
        // the hour's rows: tags over the markers, answers under the dots
        let tag_row = (-6.2, 6.2, tag_at(0.0)[1] - h(18.0), tag_at(0.0)[1] + h(18.0));
        let answer_row = (
            -6.2,
            6.2,
            DOT_Y - ANSWER_DY - h(ANSWER_SIZE),
            DOT_Y - ANSWER_DY + h(ANSWER_SIZE),
        );
        let bus_row = (ROWS_X0, ROWS_X0 + ROWS_W, BUS_BASE, BUS_BASE + 10.0 * BAR_S);
        let person_row = (ROWS_X0, ROWS_X0 + ROWS_W, PERSON_BASE, PERSON_BASE + 10.0 * BAR_S);
        let captions = [
            right(BUS_ROW, CAPTION_SIZE, CAPTION_X, BUS_BASE + CAPTION_DYS[0]),
            right(&a.bus_tally(), CAPTION_SIZE, CAPTION_X, BUS_BASE + CAPTION_DYS[1]),
            right(PASSENGER_ROW, CAPTION_SIZE, CAPTION_X, PERSON_BASE + CAPTION_DYS[0]),
            right(&a.person_tally(), CAPTION_SIZE, CAPTION_X, PERSON_BASE + CAPTION_DYS[1]),
        ];
        let levels = [
            left(
                &a.bus_level_label(),
                LEVEL_SIZE,
                LEVEL_X,
                BUS_BASE + a.bus_level.to_f64() * BAR_S,
            ),
            left(
                &a.person_level_label(),
                LEVEL_SIZE,
                LEVEL_X,
                PERSON_BASE + a.person_level.to_f64() * BAR_S,
            ),
        ];
        let sums = [
            centred(&a.bus_sum(), SUM_SIZE, 0.0, SUM_YS[0]),
            centred(&a.person_sum(), SUM_SIZE, 0.0, SUM_YS[1]),
        ];
        let ask = centred(ASK, SUM_SIZE, 0.0, SUM_YS[1]);
        let ring = (
            THINK_AT[0] - 0.62,
            THINK_AT[0] + 0.62,
            THINK_AT[1] - 0.62,
            THINK_AT[1] + 0.62,
        );
        let note = centred("pause for more time", 18.0, THINK_AT[0], THINK_AT[1] - 0.62 - 0.3);
        let boxes = [boxed(&a.bus_box(), -BOX_X, BOX_Y), boxed(&a.person_box(), BOX_X, BOX_Y)];
        let boxes_up = [
            boxed(&a.bus_box(), -BOX_X, BOX_UP_Y),
            boxed(&a.person_box(), BOX_X, BOX_UP_Y),
        ];
        let half = centred(HALF_THE_GAP, HALF_SIZE, x_at(7.0), HALF_Y);
        let card = centred(&a.card(), CARD_SIZE, BOX_X, CARD_Y);
        // a lower-third's band (`props::lower_third`): 0.4 in from each
        // side, its foot 0.2 off the frame's, 0.3 + 0.36 a line tall; K03
        // wraps at 92 characters
        let lines = |s: &str| s.chars().count().div_ceil(92) as f64;
        let third = |s: &str| (-6.71, 6.71, -3.8, -3.8 + 0.3 + 0.36 * lines(s));
        let k03 = third(&a.k03());
        let names = third(&NAMES.join(" · "));
        let (last, next) = inside_ends(&a);
        let inside = [
            centred(
                &a.behind_label(),
                INSIDE_SIZE,
                (INSIDE_X + last) / 2.0,
                INSIDE_Y + BEHIND_DY,
            ),
            centred(
                &a.ahead_label(),
                INSIDE_SIZE,
                (INSIDE_X + next) / 2.0,
                INSIDE_Y + AHEAD_DY,
            ),
            centred(
                &a.whole_label(),
                INSIDE_SIZE,
                (last + next) / 2.0,
                INSIDE_Y + WHOLE_LABEL_DY,
            ),
            (last - 0.07, next + 0.07, INSIDE_Y - 0.1, INSIDE_Y + 0.46),
            (last, next, INSIDE_Y + WHOLE_DY, INSIDE_Y + WHOLE_DY + WHOLE_TICK),
        ];
        let moments: Vec<Vec<Box4>> = vec![
            // 06_3: the rows, captions, the bus level, the bus sum, the
            // question and its ring
            [
                vec![
                    tag_row, answer_row, bus_row, person_row, sums[0], ask, ring, note, levels[0],
                ],
                captions.to_vec(),
            ]
            .concat(),
            // 06_4: the levels, both sums, the boxes
            [
                vec![tag_row, answer_row, bus_row, person_row, sums[0], sums[1]],
                captions.to_vec(),
                levels.to_vec(),
                boxes.to_vec(),
            ]
            .concat(),
            // 06_5: the boxes up, the half-gap label, the answer, K03
            [vec![tag_row, half, card, k03], boxes_up.to_vec()].concat(),
            // 06_6, 06_7
            [vec![tag_row, names], boxes_up.to_vec()].concat(),
            [vec![tag_row], boxes_up.to_vec(), inside.to_vec()].concat(),
        ];
        for (m, up) in moments.iter().enumerate() {
            for (i, &p) in up.iter().enumerate() {
                assert!(on_screen(p), "moment {m}: item {i} off screen: {p:?}");
                for (j, &q) in up.iter().enumerate().skip(i + 1) {
                    assert!(apart(p, q), "moment {m}: items {i} and {j} overlap: {p:?} {q:?}");
                }
            }
        }
    }

    /// The chapter's fixed words carry no digits: every number on screen is
    /// read from the facts.
    #[test]
    fn the_fixed_words_carry_no_numbers() {
        for w in [
            OVER_BUSES_SUM,
            OVER_PASSENGERS_SUM,
            TIMES,
            DIVIDED,
            MIN,
            SAY,
            BUS_ROW,
            PASSENGER_ROW,
            ASK,
            OVER_BUSES,
            OVER_PASSENGERS,
            HALF_THE_GAP,
            NAMES[0],
            NAMES[1],
            SINCE,
            UNTIL,
            YOUR_GAP,
            ON_AVERAGE,
            K03_WAITS,
            K03_GAP,
            K03_WAIT,
        ] {
            assert!(!w.chars().any(|c| c.is_ascii_digit()), "{w}");
        }
    }
}
