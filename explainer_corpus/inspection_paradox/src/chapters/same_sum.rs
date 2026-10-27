//! Chapter 10, the same sum elsewhere. The twelve passengers of chapter
//! 04's stretch, two in its short gap and ten in its long one, carry the
//! sum away from the stop:
//! - 10_1: the stretch, its passengers and its gaps. Picking a moment picks
//!   the long gap more often. The picture shrinks into a corner, and two
//!   empty rooms appear.
//! - 10_2: the same twelve dots fly out of the corner into a school, the
//!   two teal ones into the small class and the ten gold ones into the big
//!   one. One bar per class gives the school's average, six.
//! - 10_3: each student is asked in turn. A tag under it says 2 or 10, and
//!   a bar born at the tag joins the row of one bar per student. The asking
//!   runs on past the line, one student at a time.
//! - 10_4: the question: the students' average? Their bars are on screen;
//!   their level and their sum are not.
//! - 10_5: eight and two thirds. The students' bars level off there, and
//!   the sum is written out. Under the corner picture, the passengers' gap
//!   of 06. Ten of the twelve sit in the big class.
//! - 10_6: Feld and Grofman's article, held until it can be read. The
//!   students go back to the corner as passengers, and the school goes.
//! - 10_7: for programmers, a sampling profiler on three loops of a
//!   program. Checks evenly spaced and not in step with the loop land in
//!   the 10 ms call half the time, though it is a sixth of the calls.
//! - 10_8: the wrong clock. Checks in step with the loop land in the same
//!   function every time: all in it, or, started later, none. The exact
//!   version (K08).
//! - 10_9: the corner picture grows back, and Terada's essay of 1922.
//!
//! The class sizes are the stretch's gaps' passenger counts, so the
//! students' sum is the passengers' sum of 04 and 06, number for number.
//! `school` refuses a fact that breaks this, and `profile` checks the
//! profiler's shares against the program it draws and the checks that fall
//! on it. Every number on screen is read from the facts or counted from
//! what is drawn.

use explainer_kit::props::{lower_third, precisely};
use explainer_kit::{Facts, Kit, Palette, Timed, children, fade_all, rect, v};
use fmn::prelude::*;

use super::draw::{
    PASSENGER_R, STAND_UP, fade_in_all, fade_out_all, fail, flash_all, flash_lagged, flat, group, line_style, now,
    pause, read_s, say, shrink_to, think, wait_until,
};
use super::timetable::{self, drop_in_lagged, fade_in_with, fade_out_with, gap_color, play_lagged};
use super::twelve_minutes::{self, Passenger, fade_in_flashing, pick};
use super::two_averages::{level, written};
use crate::facts;
use crate::model::{self, CLASS_SIZES, LONG_CALL_MS, Q, SHORT_CALL_MS, SHORT_CALLS};
use crate::palette;

pub struct SameSum {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are read from the facts)
const SCHOOL: &str = "school's average:";
const STUDENTS: &str = "students' average:";
const ASK: &str = "students' average?";
const TIMES: &str = "×";
const DIVIDED: &str = "÷";
const CLASS_ROW: &str = "one bar per class";
const STUDENT_ROW: &str = "one bar per student";
const CLASSES_OF: &str = "classes of";
const AND: &str = "and";
const SAY: &str = "say";
const OVER_PASSENGERS: &str = "over passengers:";
const SOURCE: &str = "source:";
const FOR_PROGRAMMERS: &str = "for programmers";
const MS: &str = "ms";
const CALL: &str = "call:";
const CALLS: &str = "calls:";
const OF_THE_TIME: &str = "of the time,";
const OF_THE_CALLS: &str = "of the calls";
const LANDING: &str = "checks landing in the";
const IN_CALLS: &str = "calls";
const IN_STEP: &str = "checks in step with the loop:";
const LATER: &str = "started later:";
const IN_THE: &str = "in the";
const ALL: &str = "all";
const NONE: &str = "none";
const THE_LOOP: &str = "the loop";
/// The exact version of 10_7 and 10_8's claim (K08): the draft's, with its
/// two conditions said (random checks match the time share on average, even
/// ones over whole loops; no sampling clock is promised it exactly).
const K08: &str = "on average and over a long run, samples taken often and not in step with the program fall in \
                   each function in proportion to its running time";
/// Feld and Grofman's article as printed on its first page (brief.md),
/// around its year, a fact. The volume and the pages are the citation's
/// identifiers, not quantities, so they are the only digits in the
/// chapter's fixed words; `the_citations_are_the_drafts` checks them.
const FG_WORK: &str = "Feld & Grofman, 'Variation in class size, the class size paradox, and some consequences for \
                       students', Research in Higher Education";
const FG_VOLUME: &str = "6";
const FG_PAGES: &str = "215–222";
/// Terada's essay, as Masuda and Hiraoka's abstract describes it, around
/// its year (a fact), and their preprint's identifier.
const TERADA_WHO: &str = "Torahiko Terada, physicist, essay of";
const TERADA_READ: &str = "; English translation and discussion: N. Masuda & T. Hiraoka,";
const TERADA_ID: &str = "arXiv:2004.14081 (2020)";

// ------------------------------------------------------------------ layout

/// 10_1: chapter 04's stretch shrinks into the top-left corner, and the
/// passengers' gap of 06 later sits under it.
const CORNER_K: f64 = 0.42;
const CORNER_AT: Vec3 = [-5.5, 3.4, 0.0];
const CORNER_RT: f64 = 1.2;
const GHOST_AT: Vec3 = [-5.5, 2.75, 0.0];
const GHOST_SIZE: f64 = 20.0;
/// The two sums at the top, right of the corner: the school's, then the
/// question and the students' sum in one slot (the question leaves before
/// the sum comes); the think ring right of them.
const SUM_X: f64 = 0.9;
const SUM_YS: [f64; 2] = [3.35, 2.7];
const SUM_SIZE: f64 = 24.0;
const THINK_AT: Vec3 = [5.7, 2.45, 0.0];
/// The two rooms, outlined, side by side, and the seats in them: two in the
/// small room, five by two in the big one, a seat this far from the next.
const ROOM_Y: f64 = 1.4;
const ROOM_H: f64 = 1.3;
const ROOM_W: [f64; 2] = [1.3, 3.2];
const ROOM_X: [f64; 2] = [-2.6, 0.6];
const SEAT_DX: f64 = 0.6;
const SEAT_YS: [f64; 2] = [1.75, 1.15];
const BIG_COLS: usize = 5;
const STUDENT_R: f64 = 0.1;
const SEAT_RT: f64 = 1.8;
/// A student's tag, 2 or 10, small and dim under it, as the passengers'
/// answers sit under their dots in 06_2.
const TAG_DY: f64 = 0.25;
const TAG_SIZE: f64 = 16.0;
/// The two rows under the rooms, equally wide, as chapter 06 draws its
/// rows: one bar per class above, one bar per student below, gold first, a
/// bar as tall as the class it reports. Each row's caption right-aligned
/// left of it, its level's label right of it.
const ROWS_X0: f64 = -2.2;
const ROWS_W: f64 = 6.0;
const BAR_S: f64 = 0.09;
const CLASS_BASE: f64 = -0.55;
const STUDENT_BASE: f64 = -2.05;
const CLASS_GAP: f64 = 0.1;
const STUDENT_GAP: f64 = 0.04;
const BAR_FILL: f64 = 0.85;
/// A bar is born at its room or its tag this small, then grows into its
/// slot.
const BORN: f64 = 0.25;
const CAPTION_X: f64 = -2.4;
const CAPTION_DYS: [f64; 2] = [0.6, 0.3];
const CAPTION_SIZE: f64 = 20.0;
const LEVEL_OVER: f64 = 0.05;
const LEVEL_W: f64 = 3.0;
const LEVEL_X: f64 = 3.95;
const LEVEL_SIZE: f64 = 20.0;
/// 10_3: the first students are asked slowly, the rest a little faster;
/// each is still asked on its own, its tag, then its bar.
const ASK_SLOW_RT: f64 = 0.9;
const ASK_RT: f64 = 0.5;
const ASK_SLOW: usize = 3;
/// Emphasis: an outline around a text, faded in, held, faded out.
const HALO_PAD: f64 = 0.1;
const HALO_W: f64 = 3.0;
const HALO_RT: f64 = 0.9;
/// 10_7, 10_8: the profiler's panel, framed, between the corner picture and
/// a two-line lower-third (top near y = −2.78). Three loops of the program
/// along a line, a millisecond this wide; the checks fall onto the calls
/// from above; the tallies under them.
const PANEL_X: [f64; 2] = [-6.6, 6.6];
const PANEL_Y: [f64; 2] = [-2.4, 2.7];
const PANEL_SLIDE: f64 = 7.0;
const HEADER_AT: Vec3 = [-6.3, 2.35, 0.0];
const HEADER_SIZE: f64 = 24.0;
const LOOPS: i64 = 3;
const PROGRAM_X0: f64 = -5.4;
const MS_W: f64 = 0.18;
const CALL_Y: f64 = 0.95;
const CALL_H: f64 = 0.4;
const CALL_INSET: f64 = 0.02;
const NAME_Y: f64 = 0.5;
const NAME_SIZE: f64 = 18.0;
const CHECK_Y: [f64; 2] = [1.25, 1.6];
const CHECK_W: f64 = 3.0;
const CHECK_DROP: f64 = 0.3;
const LOOP_Y: f64 = 1.8;
const LOOP_TICK: f64 = 0.1;
const LOOP_LABEL_DY: f64 = 0.22;
const TALLY_YS: [f64; 3] = [-0.2, -0.75, -1.3];
const TALLY_SIZE: f64 = 22.0;
/// The evenly spaced checks: every 3 ms from 1.5 ms, never on a call's
/// edge, and 3 shares no factor with the loop's 20, so over the three loops
/// they meet every part of the loop equally often. The checks in step with
/// the loop: one a loop, in the middle of the long call, then the same
/// checks started a long call later.
fn even_first() -> Q {
    Q::new(3, 2)
}

fn even_every() -> Q {
    Q::int(3)
}

// ------------------------------------------------------------- the numbers

/// A fact's printed fraction ("26/3", "6") as a number.
fn parse_q(key: &str, printed: &str) -> Result<Q, String> {
    let int = |s: &str| s.parse::<i64>().map_err(|e| format!("{key} = {printed}: {e}"));
    match printed.split_once('/') {
        Some((n, d)) => Ok(Q::new(int(n)?, int(d)?)),
        None => Ok(Q::int(int(printed)?)),
    }
}

/// Every number of the school (10_1 to 10_6), as the facts print it, with
/// the values the picture draws.
pub struct School {
    /// The classes' sizes as printed, small then big, and as drawn.
    sizes: Vec<String>,
    pub size_values: Vec<i64>,
    classes: String,
    school_mean: String,
    students: String,
    /// The students' answers added: 2 × 2 + 10 × 10.
    squares: String,
    student_mean: String,
    /// Each student's class (its place in `size_values`), in passenger
    /// order: the gap the passenger stood in.
    pub class_of: Vec<usize>,
    pub school_level: Q,
    pub student_level: Q,
    /// The passengers' gap of 06, spelt out.
    ghost: String,
    /// The two lower-thirds, with the facts' years.
    fg: String,
    terada: String,
}

impl School {
    /// The size a student in class `c` reports, as printed.
    fn answer(&self, c: usize) -> &str {
        &self.sizes[c]
    }

    /// "school's average: (2 + 10) ÷ 2 = 6".
    fn school_sum(&self) -> String {
        format!(
            "{SCHOOL} ({} + {}) {DIVIDED} {} = {}",
            self.sizes[0], self.sizes[1], self.classes, self.school_mean
        )
    }

    /// "students' average: (2 × 2 + 10 × 10) ÷ 12 = 104 ÷ 12 = 8⅔": how
    /// many students say each size, times the size.
    fn student_sum(&self) -> String {
        let (a, b) = (&self.sizes[0], &self.sizes[1]);
        format!(
            "{STUDENTS} ({a} {TIMES} {a} + {b} {TIMES} {b}) {DIVIDED} {n} = {sq} {DIVIDED} {n} = {mean}",
            n = self.students,
            sq = self.squares,
            mean = self.student_mean
        )
    }

    /// The rows' tallies: "classes of 2 and 10", "2 say 2, 10 say 10".
    fn class_tally(&self) -> String {
        format!("{CLASSES_OF} {} {AND} {}", self.sizes[0], self.sizes[1])
    }

    fn student_tally(&self) -> String {
        let (a, b) = (&self.sizes[0], &self.sizes[1]);
        format!("{a} {SAY} {a}, {b} {SAY} {b}")
    }

    /// 06_4's gold box, word for word: "over passengers: 8 min 40 s".
    fn ghost(&self) -> String {
        format!("{OVER_PASSENGERS} {}", self.ghost)
    }
}

/// The school's numbers, refused when a fact the picture prints stops
/// matching the stretch it comes from: a class for each of the stretch's
/// gaps, as big as the passengers in it, so the students are the
/// passengers, their answers are the passengers' gaps, and their average is
/// the passengers' gap of 06.
pub fn school(f: &Facts, people: &[Passenger]) -> Result<School, String> {
    let get = |key: &str| f.get(key).map(str::to_owned).ok_or_else(|| format!("no fact {key}"));
    let gs = twelve_minutes::gaps();
    let counts: Vec<i64> = (0..gs.len())
        .map(|g| twelve_minutes::in_gap(people, g).len() as i64)
        .collect();
    if counts != CLASS_SIZES {
        return Err(format!(
            "the stretch's gaps hold {counts:?} passengers, not the classes {CLASS_SIZES:?}"
        ));
    }
    let sizes: Vec<String> = get("F-C1.sizes")?.split(',').map(str::to_owned).collect();
    if sizes != counts.iter().map(i64::to_string).collect::<Vec<_>>() {
        return Err(format!(
            "F-C1.sizes is {}, but the stretch's gaps hold {counts:?}",
            sizes.join(",")
        ));
    }
    let class_of: Vec<usize> = people.iter().map(|p| p.gap).collect();
    let answers: Vec<i64> = class_of.iter().map(|&c| counts[c]).collect();
    let students = people.len() as i64;
    let squares: i64 = answers.iter().sum();
    if squares != counts.iter().map(|c| c * c).sum::<i64>() {
        return Err("the students' answers do not add up to the classes' sizes squared".into());
    }
    let school_level = model::mean_gap(&counts);
    let student_level = Q::new(squares, students);
    let big = counts.iter().copied().max().unwrap_or_default();
    let checks = [
        ("classes", counts.len().to_string(), "F-C1.classes"),
        ("the school's average", school_level.to_string(), "F-C1.school_mean"),
        (
            "the school's average, the sign's",
            school_level.to_string(),
            "F-T1.mean_gap",
        ),
        ("students", students.to_string(), "F-C1.students"),
        (
            "students, the stretch's passengers",
            students.to_string(),
            "F-W2.people",
        ),
        ("the big class", big.to_string(), "F-C1.big_class_students"),
        (
            "the big class, the long gap's passengers",
            big.to_string(),
            "F-W2.long_people",
        ),
        ("the students' average", student_level.to_string(), "F-C1.student_mean"),
        (
            "the students' average, the passengers'",
            model::passengers_gap(&counts).to_string(),
            "F-C1.student_mean",
        ),
        (
            "the students' average, the passengers' gap",
            student_level.to_string(),
            "F-S1.person_gap",
        ),
        (
            "the students' average as a clock",
            model::clock(student_level)?,
            "F-S1.person_gap_clock",
        ),
        ("the class bars' level", level(&counts).to_string(), "F-C1.school_mean"),
        (
            "the student bars' level",
            level(&answers).to_string(),
            "F-C1.student_mean",
        ),
    ];
    for (what, drawn, key) in checks {
        let shown = get(key)?;
        if drawn != shown {
            return Err(format!("{what}: the picture draws {drawn}, but {key} is {shown}"));
        }
    }
    if get("F-H1.over_a_century")? != "true" {
        return Err("10_9 says over a century ago, but F-H1.over_a_century is not true".into());
    }
    let ghost = super::draw::min_s(&get("F-S1.person_gap_clock")?).map_err(|e| e.to_string())?;
    Ok(School {
        sizes,
        size_values: counts,
        classes: get("F-C1.classes")?,
        school_mean: get("F-C1.school_mean")?,
        students: get("F-C1.students")?,
        squares: squares.to_string(),
        student_mean: written(parse_q("F-C1.student_mean", &get("F-C1.student_mean")?)?)?,
        class_of,
        school_level,
        student_level,
        ghost,
        fg: format!("{FG_WORK} {FG_VOLUME} ({}) {FG_PAGES}", get("F-C1.year")?),
        terada: format!("{TERADA_WHO} {}{TERADA_READ} {TERADA_ID}", get("F-H1.year")?),
    })
}

// ------------------------------------------------------------ the profiler

/// The program's loop: one long call, then the short ones, as (start,
/// length) in milliseconds.
pub fn calls() -> Vec<(i64, i64)> {
    let mut out = vec![(0, LONG_CALL_MS)];
    for k in 0..SHORT_CALLS {
        out.push((LONG_CALL_MS + k * SHORT_CALL_MS, SHORT_CALL_MS));
    }
    out
}

/// How long the loop runs, in milliseconds.
pub fn loop_ms() -> i64 {
    calls().iter().map(|&(_, len)| len).sum()
}

/// Whether moment `t` (ms from the program's start) falls in the long call
/// of its loop.
pub fn in_long_call(t: Q) -> bool {
    let p = Q::int(loop_ms());
    let loops = (t / p).num().div_euclid((t / p).den());
    t - p * Q::int(loops) < Q::int(LONG_CALL_MS)
}

/// A clock's checks over the drawn loops: every `every` ms from `first`.
pub fn clock(first: Q, every: Q) -> Vec<Q> {
    let end = Q::int(LOOPS * loop_ms());
    let mut out = Vec::new();
    let mut t = first;
    while t < end {
        out.push(t);
        t = t + every;
    }
    out
}

/// The share of `checks` that land in a long call.
pub fn share_in_long(checks: &[Q]) -> Q {
    let n = checks.iter().filter(|&&t| in_long_call(t)).count();
    Q::new(n as i64, checks.len().max(1) as i64)
}

/// The checks in step with the loop: one a loop, in the middle of the long
/// call, or (`later`) a long call later.
pub fn in_step(later: bool) -> Vec<Q> {
    let first = Q::new(LONG_CALL_MS, 2) + if later { Q::int(LONG_CALL_MS) } else { Q::int(0) };
    clock(first, Q::int(loop_ms()))
}

/// Every number of the profiler (10_7, 10_8), as the facts print it, with
/// the shares the drawn checks give.
pub struct Profile {
    long_ms: String,
    short_ms: String,
    /// The long call's share of the time and of the calls, then the short
    /// calls' (each the rest), as glyphs.
    long_time: String,
    long_calls: String,
    short_time: String,
    short_calls: String,
    /// The share of the even checks that land in the long call.
    checks: String,
    /// The checks in step with the loop: all in the long call, then none.
    in_step: [&'static str; 2],
}

impl Profile {
    /// "10 ms call: ½ of the time, ⅙ of the calls".
    fn long_tally(&self) -> String {
        format!(
            "{} {MS} {CALL} {} {OF_THE_TIME} {} {OF_THE_CALLS}",
            self.long_ms, self.long_time, self.long_calls
        )
    }

    /// "2 ms calls: ½ of the time, ⅚ of the calls".
    fn short_tally(&self) -> String {
        format!(
            "{} {MS} {CALLS} {} {OF_THE_TIME} {} {OF_THE_CALLS}",
            self.short_ms, self.short_time, self.short_calls
        )
    }

    /// "checks landing in the 10 ms calls: ½".
    fn checks_line(&self) -> String {
        format!("{LANDING} {} {MS} {IN_CALLS}: {}", self.long_ms, self.checks)
    }

    /// "checks in step with the loop: all in the 10 ms calls", then
    /// "started later: none in the 10 ms calls".
    fn in_step_lines(&self) -> [String; 2] {
        [
            format!(
                "{IN_STEP} {} {IN_THE} {} {MS} {IN_CALLS}",
                self.in_step[0], self.long_ms
            ),
            format!("{LATER} {} {IN_THE} {} {MS} {IN_CALLS}", self.in_step[1], self.long_ms),
        ]
    }

    /// The calls' names under the first loop: "10 ms", "2 ms".
    fn names(&self) -> [String; 2] {
        [format!("{} {MS}", self.long_ms), format!("{} {MS}", self.short_ms)]
    }
}

/// A share as the in-step line says it: all, or none. Any other share is
/// refused (the line would need a number).
fn all_or_none(share: Q) -> Result<&'static str, String> {
    if share == Q::int(1) {
        Ok(ALL)
    } else if share == Q::int(0) {
        Ok(NONE)
    } else {
        Err(format!(
            "checks in step with the loop land in the long call {share} of the time"
        ))
    }
}

/// The profiler's numbers, refused when a fact stops matching the program
/// drawn: its calls, the long call's share of the time (what evenly spread
/// samples measure, F-P1.long_sample_share) and of the calls, and the share
/// of the drawn even checks that land in it.
pub fn profile(f: &Facts) -> Result<Profile, String> {
    let get = |key: &str| f.get(key).map(str::to_owned).ok_or_else(|| format!("no fact {key}"));
    let p = loop_ms();
    let n_calls = calls().len() as i64;
    let long_time = Q::new(LONG_CALL_MS, p);
    let long_calls = Q::new(1, n_calls);
    let short_time = Q::new(SHORT_CALLS * SHORT_CALL_MS, p);
    let short_calls = Q::new(SHORT_CALLS, n_calls);
    let one = Q::int(1);
    if long_time + short_time != one || long_calls + short_calls != one {
        return Err("the long call and the short calls are not the whole loop".into());
    }
    let checks = share_in_long(&clock(even_first(), even_every()));
    let checks_printed = [
        ("the long call's length", LONG_CALL_MS.to_string(), "F-P1.long_ms"),
        ("a short call's length", SHORT_CALL_MS.to_string(), "F-P1.short_ms"),
        ("the short calls", SHORT_CALLS.to_string(), "F-P1.short_calls"),
        (
            "the long call's share of the time",
            long_time.to_string(),
            "F-P1.long_sample_share",
        ),
        (
            "the even checks in the long call",
            checks.to_string(),
            "F-P1.long_sample_share",
        ),
        (
            "the long call's share of the calls",
            long_calls.to_string(),
            "F-P1.long_call_share",
        ),
    ];
    for (what, drawn, key) in checks_printed {
        let shown = get(key)?;
        if drawn != shown {
            return Err(format!("{what}: the picture draws {drawn}, but {key} is {shown}"));
        }
    }
    let printed = |key: &str| -> Result<Q, String> { parse_q(key, &get(key)?) };
    let long_time_f = printed("F-P1.long_sample_share")?;
    let long_calls_f = printed("F-P1.long_call_share")?;
    Ok(Profile {
        long_ms: get("F-P1.long_ms")?,
        short_ms: get("F-P1.short_ms")?,
        long_time: written(long_time_f)?,
        long_calls: written(long_calls_f)?,
        short_time: written(one - long_time_f)?,
        short_calls: written(one - long_calls_f)?,
        checks: written(checks)?,
        in_step: [
            all_or_none(share_in_long(&in_step(false)))?,
            all_or_none(share_in_long(&in_step(true)))?,
        ],
    })
}

// ----------------------------------------------------------------- picture

/// Where a point of the full-size stretch lands once the stretch, centred
/// at `centre`, has shrunk into the corner.
fn cornered(p: Vec3, centre: Vec3) -> Vec3 {
    v(
        CORNER_AT[0] + CORNER_K * (p[0] - centre[0]),
        CORNER_AT[1] + CORNER_K * (p[1] - centre[1]),
    )
}

/// Each student's seat, in passenger order: the small class's in a row,
/// the big class's five by two. A class with more students than seats is
/// refused.
pub fn seats(class_of: &[usize]) -> Result<Vec<Vec3>, String> {
    let mut taken = [0_usize; 2];
    class_of
        .iter()
        .map(|&c| {
            let j = *taken.get(c).ok_or_else(|| format!("no room for class {c}"))?;
            taken[c] += 1;
            let room = [CLASS_SIZES[0] as usize, BIG_COLS * SEAT_YS.len()];
            if j >= room[c] {
                return Err(format!("class {c} has more students than seats"));
            }
            let cols = if c == 0 { room[0] } else { BIG_COLS };
            let x0 = ROOM_X[c] - SEAT_DX * (cols - 1) as f64 / 2.0;
            Ok(v(x0 + SEAT_DX * (j % cols) as f64, SEAT_YS[j / cols]))
        })
        .collect()
}

/// A class's color: its gap's, teal for the small class (the short gap's
/// two passengers), gold for the big one (the long gap's ten).
fn class_color(c: usize) -> Srgb {
    gap_color(CLASS_SIZES[c])
}

/// A room's outline, detached.
fn room(stage: &mut Stage<'_>, c: usize) -> fmn::Result<Mob> {
    let r = rect(
        v(ROOM_X[c] - ROOM_W[c] / 2.0, ROOM_Y - ROOM_H / 2.0),
        ROOM_W[c],
        ROOM_H,
        line_style(Palette::dim(), 2.5),
    )?;
    Ok(stage.arena_mut().add(r))
}

/// A row's slots: equal slots across the rows' width, the big class's
/// first, as chapter 06 sorts its rows. Each is (the thing's index, its
/// slot's centre x).
pub fn slots(bigs: &[bool]) -> Vec<(usize, f64)> {
    let n = bigs.len();
    let w = ROWS_W / n as f64;
    (0..n)
        .filter(|&i| bigs[i])
        .chain((0..n).filter(|&i| !bigs[i]))
        .enumerate()
        .map(|(slot, i)| (i, ROWS_X0 + w * (slot as f64 + 0.5)))
        .collect()
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
fn level_line(stage: &mut Stage<'_>, base: f64, height: Q) -> fmn::Result<Mob> {
    let y = base + height.to_f64() * BAR_S;
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

/// Emphasis for texts: an outline around their union, stroked only, that
/// fades in, holds and fades out over `rt`. The words themselves are left
/// as they are (a flash would scale and recolor them).
fn halo(stage: &mut Stage<'_>, mobs: &[Mob], color: Srgb, rt: f64, reveal: &[Mob]) -> fmn::Result<()> {
    let mut bounds: Option<(f64, f64, f64, f64)> = None;
    for &m in mobs {
        let c = stage.get_center(m);
        let w = stage.get_width(m) + 2.0 * HALO_PAD;
        let h = stage.get_height(m) + 2.0 * HALO_PAD;
        let b = (c[0] - w / 2.0, c[0] + w / 2.0, c[1] - h / 2.0, c[1] + h / 2.0);
        bounds = Some(match bounds {
            None => b,
            Some(a) => (a.0.min(b.0), a.1.max(b.1), a.2.min(b.2), a.3.max(b.3)),
        });
    }
    let Some((left, right, bottom, top)) = bounds else {
        return Ok(());
    };
    let ring = rect(v(left, bottom), right - left, top - bottom, line_style(color, HALO_W))?;
    let rings = vec![stage.arena_mut().add(ring)];
    let mut entering = rings.clone();
    entering.extend_from_slice(reveal);
    fade_in_all(stage, &entering, rt * 0.3)?;
    pause(stage, rt * 0.4)?;
    fade_out_all(stage, &rings, rt * 0.3)
}

/// Keep the time share visible while withholding the contrasting call share.
/// Align both pieces to glyphs of an unshown whole-text reference. Separately
/// centring their different glyph bounds would shift their baselines and spacing.
fn profile_tally(kit: &Kit, stage: &mut Stage<'_>, text: &str, color: Srgb, at: Vec3) -> fmn::Result<[Mob; 2]> {
    let (time, calls) = text
        .split_once(',')
        .ok_or_else(|| fail("profiler tally has no contrast".into()))?;
    let whole = kit.text_left(stage, text, TALLY_SIZE, color, at)?;
    let time = kit.text_left(stage, time, TALLY_SIZE, color, at)?;
    let calls = kit.text_left(stage, &format!(",{calls}"), TALLY_SIZE, color, at)?;
    let (w, t, c) = (children(stage, whole), children(stage, time), children(stage, calls));
    if t.is_empty() || c.is_empty() || w.len() != t.len() + c.len() {
        return Err(fail("profiler tally glyph partition differs from whole text".into()));
    }
    for (piece, first, index) in [(time, t[0], 0), (calls, c[0], t.len())] {
        let (to, from) = (stage.get_center(w[index]), stage.get_center(first));
        stage.shift(piece, v(to[0] - from[0], to[1] - from[1]));
    }
    Ok([time, calls])
}

/// Detached mobjects slide in together from `dx` to the right of their
/// places, fading in, over `rt`.
fn slide_in(stage: &mut Stage<'_>, mobs: &[Mob], dx: f64, rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fade_in(stage.arena_mut(), m, v(-dx, 0.0), 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Move a mobject's centre to `to` and recolor it, in one animation over
/// `rt` (two animations on one mobject would fight).
fn move_recolor(mob: Mob, to: Vec3, color: Srgb, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .move_to(to, ORIGIN)?
        .set_color(color)?)
}

/// Moment `t` of the program (ms from its start) on the panel.
pub fn program_x(t: f64) -> f64 {
    PROGRAM_X0 + MS_W * t
}

/// A call's color by its length: gold for the long call, teal for a short
/// one, the gaps' roles (a long stretch of time, a short one).
fn call_color(long: bool) -> Srgb {
    if long {
        palette::long_gap()
    } else {
        timetable::short_gap()
    }
}

/// The profiler's panel, detached: its frame, its header, every call of
/// the drawn loops (the long ones and the short ones apart) and the names
/// of the first loop's two kinds of call under them.
struct Program {
    frame: Mob,
    header: Mob,
    long: Vec<Mob>,
    short: Vec<Mob>,
    names: Vec<Mob>,
}

impl Program {
    fn mobs(&self) -> Vec<Mob> {
        let mut out = vec![self.frame, self.header];
        out.extend(&self.long);
        out.extend(&self.short);
        out.extend(&self.names);
        out
    }
}

fn program(kit: &Kit, stage: &mut Stage<'_>, p: &Profile) -> fmn::Result<Program> {
    let frame = rect(
        v(PANEL_X[0], PANEL_Y[0]),
        PANEL_X[1] - PANEL_X[0],
        PANEL_Y[1] - PANEL_Y[0],
        line_style(Palette::dim(), 2.0),
    )?;
    let frame = stage.arena_mut().add(frame);
    let header = kit.text_left(stage, FOR_PROGRAMMERS, HEADER_SIZE, Palette::dim(), HEADER_AT)?;
    let (mut long, mut short) = (Vec::new(), Vec::new());
    for k in 0..LOOPS {
        for (i, &(start, len)) in calls().iter().enumerate() {
            let t = (k * loop_ms() + start) as f64;
            let r = rect(
                v(program_x(t) + CALL_INSET, CALL_Y - CALL_H / 2.0),
                MS_W * len as f64 - 2.0 * CALL_INSET,
                CALL_H,
                flat(call_color(i == 0), BAR_FILL),
            )?;
            let m = stage.arena_mut().add(r);
            if i == 0 { long.push(m) } else { short.push(m) }
        }
    }
    let first = calls();
    let mut names = Vec::with_capacity(2);
    for (name, &(start, len)) in p.names().iter().zip(&first) {
        let x = program_x(start as f64 + len as f64 / 2.0);
        names.push(kit.text(stage, name, NAME_SIZE, Palette::dim(), v(x, NAME_Y))?);
    }
    Ok(Program {
        frame,
        header,
        long,
        short,
        names,
    })
}

/// A check at moment `t` of the program: a short line falling onto the
/// calls, in the color of the call it lands in; detached.
fn check(stage: &mut Stage<'_>, t: Q) -> fmn::Result<Mob> {
    let x = program_x(t.to_f64());
    let line = Line::new(v(x, CHECK_Y[0]), v(x, CHECK_Y[1]))
        .style(line_style(call_color(in_long_call(t)), CHECK_W))
        .build()?;
    Ok(stage.arena_mut().add(line))
}

/// The bracket over the first loop and its name, detached.
fn loop_bracket(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<[Mob; 2]> {
    let (a, b) = (program_x(0.0), program_x(loop_ms() as f64));
    let line = stage.arena_mut().add(
        polyline(vec![
            v(a, LOOP_Y - LOOP_TICK),
            v(a, LOOP_Y),
            v(b, LOOP_Y),
            v(b, LOOP_Y - LOOP_TICK),
        ])
        .style(line_style(Palette::dim(), 2.5))
        .build(),
    );
    let label = kit.text(
        stage,
        THE_LOOP,
        NAME_SIZE,
        Palette::dim(),
        v((a + b) / 2.0, LOOP_Y + LOOP_LABEL_DY),
    )?;
    Ok([line, label])
}

impl SceneConstruct for SameSum {
    fn name(&self) -> &str {
        "10_same_sum"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let f = facts::facts().map_err(fail)?;
        let n = twelve_minutes::numbers(&f).map_err(fail)?;
        let people = twelve_minutes::passengers().map_err(fail)?;
        let s = school(&f, &people).map_err(fail)?;
        let p = profile(&f).map_err(fail)?;
        let seat_at = seats(&s.class_of).map_err(fail)?;
        let small = twelve_minutes::in_gap(&people, 0);
        let big = twelve_minutes::in_gap(&people, 1);

        // 10_1: chapter 04's stretch, its passengers and gaps; a moment
        // picked lands in the long gap more often; the picture shrinks into
        // the corner, and two empty rooms appear
        let line = say(&kit, stage, "10_1")?;
        let st = twelve_minutes::stretch(&kit, stage, &n, &people)?;
        fade_in_all(stage, &st.mobs(), 0.8)?;
        line.wait_for(stage, "Whenever you pick a person or a moment")?;
        flash_lagged(stage, &st.dots, 1.2, 0.08)?;
        line.wait_for(stage, "instead of a group")?;
        let bracket_lines: Vec<Mob> = st.brackets.iter().map(|b| b.line).collect();
        flash_all(stage, &bracket_lines, 0.6)?;
        line.wait_for(stage, "the big groups get picked more often")?;
        let mut long_group = pick(&st.dots, &big);
        long_group.push(st.brackets[1].line);
        flash_lagged(stage, &long_group, 1.0, 0.05)?;
        let parts: Vec<Mob> = st.mobs().into_iter().filter(|m| !st.dots.contains(m)).collect();
        let corner = group(stage, &parts)?;
        let centre = stage.get_center(corner);
        let home: Vec<Vec3> = st.dots.iter().map(|&d| stage.get_center(d)).collect();
        let mut moves = vec![shrink_to(corner, CORNER_K, CORNER_AT, CORNER_RT)?];
        for (&d, &h) in st.dots.iter().zip(&home) {
            moves.push(shrink_to(d, CORNER_K, cornered(h, centre), CORNER_RT)?);
        }
        fade_in_with(stage, &[], moves, CORNER_RT)?;
        let rooms = [room(stage, 0)?, room(stage, 1)?];
        fade_in_all(stage, &rooms, 0.6)?;

        // 10_2: the same twelve dots fly out of the corner into their
        // classes and grow into students; one bar per class, born in its
        // room; the school's average
        let line = say(&kit, stage, "10_2")?;
        line.wait_for(stage, "seat those twelve passengers")?;
        let grow = STUDENT_R / (PASSENGER_R * CORNER_K);
        let seated = st
            .dots
            .iter()
            .zip(&seat_at)
            .map(|(&d, &at)| shrink_to(d, grow, at, SEAT_RT * 0.4))
            .collect::<fmn::Result<Vec<_>>>()?;
        play_lagged(stage, seated, SEAT_RT, 0.1)?;
        line.wait_for(stage, "two classes")?;
        flash_all(stage, &rooms, 0.6)?;
        line.wait_for(stage, "one of two students")?;
        flash_lagged(stage, &pick(&st.dots, &small), 0.6, 0.3)?;
        line.wait_for(stage, "and one of ten")?;
        flash_lagged(stage, &pick(&st.dots, &big), 1.0, 0.05)?;
        line.wait_for(stage, "The school says")?;
        let class_w = ROWS_W / s.size_values.len() as f64 - CLASS_GAP;
        let class_bigs: Vec<bool> = (0..s.size_values.len()).map(|c| c == 1).collect();
        let mut class_bars = Vec::with_capacity(2);
        let mut births = Vec::with_capacity(2);
        for (c, x) in slots(&class_bigs) {
            let h = s.size_values[c] as f64 * BAR_S;
            let bar = born_bar(stage, v(ROOM_X[c], ROOM_Y), class_w, h, class_color(c))?;
            births.push(shrink_to(bar, 1.0 / BORN, v(x, CLASS_BASE + h / 2.0), 1.0)?);
            class_bars.push(bar);
        }
        play_lagged(stage, births, 1.4, 0.3)?;
        let class_caption = caption(&kit, stage, [CLASS_ROW, &s.class_tally()], CLASS_BASE)?;
        fade_in_all(stage, &class_caption, 0.4)?;
        line.wait_for(stage, "six students")?;
        let class_level = level_line(stage, CLASS_BASE, s.school_level)?;
        let class_label = kit.text_left(
            stage,
            &s.school_mean,
            LEVEL_SIZE,
            Palette::ink(),
            v(LEVEL_X, CLASS_BASE + s.school_level.to_f64() * BAR_S),
        )?;
        grow_with(stage, class_level, LEFT, &[class_label], 0.6)?;
        let school_sum = kit.text(stage, &s.school_sum(), SUM_SIZE, Palette::ink(), v(SUM_X, SUM_YS[0]))?;
        fade_in_all(stage, &[school_sum], 0.5)?;

        // 10_3: each student in turn: its tag, then its bar, born at the
        // tag; the row's caption. The asking runs on past the line, so the
        // question below waits for it as a line of its own
        let line = say(&kit, stage, "10_3")?;
        line.wait_for(stage, "Ask every student in turn")?;
        let student_w = ROWS_W / people.len() as f64 - STUDENT_GAP;
        let student_bigs: Vec<bool> = s.class_of.iter().map(|&c| c == 1).collect();
        let slot_of: Vec<f64> = {
            let mut xs = vec![0.0; people.len()];
            for (i, x) in slots(&student_bigs) {
                xs[i] = x;
            }
            xs
        };
        let mut tags = Vec::with_capacity(people.len());
        let mut student_bars = Vec::with_capacity(people.len());
        for (i, (&dot, &at)) in st.dots.iter().zip(&seat_at).enumerate() {
            let c = s.class_of[i];
            let rt = if i < ASK_SLOW { ASK_SLOW_RT } else { ASK_RT };
            let tag_at = v(at[0], at[1] - TAG_DY);
            let tag = kit.text(stage, s.answer(c), TAG_SIZE, Palette::dim(), tag_at)?;
            fade_in_flashing(stage, &[tag], &[dot], rt / 2.0)?;
            let h = s.size_values[c] as f64 * BAR_S;
            let bar = born_bar(stage, tag_at, student_w, h, class_color(c))?;
            stage.play(shrink_to(
                bar,
                1.0 / BORN,
                v(slot_of[i], STUDENT_BASE + h / 2.0),
                rt / 2.0,
            )?)?;
            tags.push(tag);
            student_bars.push(bar);
        }
        let student_caption = caption(&kit, stage, [STUDENT_ROW, &s.student_tally()], STUDENT_BASE)?;
        fade_in_all(stage, &student_caption, 0.4)?;

        // 10_4: the question; think time (the bars on screen, no level, no
        // sum)
        let line = say(&kit, stage, "10_4")?;
        line.wait_for(stage, "What's the average answer")?;
        let ask = kit.text(stage, ASK, SUM_SIZE, Palette::ink(), v(SUM_X, SUM_YS[1]))?;
        fade_in_all(stage, &[ask], 0.4)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 10_5: eight and two thirds: the students' bars level off there;
        // the sum; the passengers' gap of 06 under the corner picture, whose
        // gaps flash; the big class
        let line = say(&kit, stage, "10_5")?;
        fade_out_all(stage, &[q, ask], 0.3)?;
        let gold = palette::long_gap();
        let student_level = level_line(stage, STUDENT_BASE, s.student_level)?;
        let student_label = kit.text_left(
            stage,
            &s.student_mean,
            LEVEL_SIZE,
            gold,
            v(LEVEL_X, STUDENT_BASE + s.student_level.to_f64() * BAR_S),
        )?;
        grow_with(stage, student_level, LEFT, &[student_label], 0.6)?;
        let student_sum = kit.text(stage, &s.student_sum(), SUM_SIZE, gold, v(SUM_X, SUM_YS[1]))?;
        fade_in_all(stage, &[student_sum], 0.5)?;
        line.wait_for(stage, "It's the bus sum again")?;
        let ghost = kit.text(stage, &s.ghost(), GHOST_SIZE, gold, GHOST_AT)?;
        let corner_gaps: Vec<Mob> = st.bracket_mobs();
        fade_in_flashing(stage, &[ghost], &corner_gaps, 0.6)?;
        line.wait_for(stage, "students in place of minutes")?;
        flash_lagged(stage, &st.dots, 1.0, 0.05)?;
        line.wait_for(stage, "Ten of the twelve students")?;
        let mut the_big = pick(&st.dots, &big);
        the_big.extend(pick(&student_bars, &big));
        flash_lagged(stage, &the_big, 1.2, 0.04)?;
        halo(stage, &[student_label], gold, HALO_RT, &[])?;

        // 10_6: the source, held until read; the students go back to the
        // corner as passengers, and the school goes
        let line = say(&kit, stage, "10_6")?;
        let fg = lower_third(&kit, stage, SOURCE, &s.fg)?;
        fade_in_all(stage, &[fg], 0.5)?;
        let fg_up = now(stage);
        line.wait_for(stage, "the class size paradox")?;
        kit.hold(stage)?;
        wait_until(stage, fg_up + read_s(&s.fg))?;
        let back = st
            .dots
            .iter()
            .zip(&home)
            .map(|(&d, &h)| shrink_to(d, 1.0 / grow, cornered(h, centre), 1.0))
            .collect::<fmn::Result<Vec<_>>>()?;
        let mut school_mobs: Vec<Mob> = rooms.to_vec();
        school_mobs.extend(&tags);
        school_mobs.extend(&class_bars);
        school_mobs.extend(&student_bars);
        school_mobs.extend(&class_caption);
        school_mobs.extend(&student_caption);
        school_mobs.extend([
            class_level,
            class_label,
            student_level,
            student_label,
            school_sum,
            student_sum,
            ghost,
            fg,
        ]);
        fade_out_with(stage, &school_mobs, back, 1.0)?;

        // 10_7: for programmers: the program's loops slide in; even checks
        // fall on them in their calls' colors; the long call's share of the
        // time and of the calls, then the short calls'; the checks' share
        let line = say(&kit, stage, "10_7")?;
        let prog = program(&kit, stage, &p)?;
        slide_in(stage, &prog.mobs(), PANEL_SLIDE, 0.8)?;
        line.wait_for(stage, "checks what your program is doing")?;
        let even = clock(even_first(), even_every());
        let ticks = even.iter().map(|&t| check(stage, t)).collect::<fmn::Result<Vec<_>>>()?;
        drop_in_lagged(stage, &ticks, CHECK_DROP, 2.4, 0.1)?;
        let tally_at = |i: usize| v(PROGRAM_X0, TALLY_YS[i]);
        line.wait_for(stage, "one ten millisecond call")?;
        let [long_tally, long_calls] = profile_tally(&kit, stage, &p.long_tally(), gold, tally_at(0))?;
        fade_in_flashing(stage, &[long_tally], &prog.long, 0.6)?;
        line.wait_for(stage, "five two millisecond calls")?;
        let teal = timetable::short_gap();
        let [short_tally, short_calls] = profile_tally(&kit, stage, &p.short_tally(), teal, tally_at(1))?;
        fade_in_flashing(stage, &[short_tally], &prog.short, 0.6)?;
        line.wait_for(stage, "half the samples")?;
        let checks_line = kit.text_left(stage, &p.checks_line(), TALLY_SIZE, Palette::ink(), tally_at(2))?;
        let gold_ticks: Vec<Mob> = ticks
            .iter()
            .zip(&even)
            .filter(|&(_, &t)| in_long_call(t))
            .map(|(&m, _)| m)
            .collect();
        fade_in_flashing(stage, &[checks_line], &gold_ticks, 0.6)?;
        line.wait_for(stage, "a sixth of the calls")?;
        // Reveal the contrast inside the existing halo interval, adding no time.
        halo(
            stage,
            &[long_tally, long_calls],
            gold,
            HALO_RT,
            &[long_calls, short_calls],
        )?;

        // 10_8: the even checks go; the loop named; checks in step with it
        // all land in the long call, then, started a long call later, none;
        // K08, held until read
        let line = say(&kit, stage, "10_8")?;
        line.wait_for(stage, "aren't in step")?;
        let mut leaving = ticks;
        leaving.push(checks_line);
        fade_out_all(stage, &leaving, 0.5)?;
        line.wait_for(stage, "Checks that tick exactly with its loop")?;
        let bracket = loop_bracket(&kit, stage)?;
        fade_in_all(stage, &bracket, 0.4)?;
        let stepped = in_step(false);
        let step_ticks = stepped
            .iter()
            .map(|&t| check(stage, t))
            .collect::<fmn::Result<Vec<_>>>()?;
        drop_in_lagged(stage, &step_ticks, CHECK_DROP, 1.2, 0.3)?;
        line.wait_for(stage, "land in the same function every time")?;
        let [all_line, none_line] = p.in_step_lines();
        let in_step_text = kit.text_left(stage, &all_line, TALLY_SIZE, Palette::ink(), tally_at(2))?;
        fade_in_flashing(stage, &[in_step_text], &step_ticks, 0.6)?;
        line.wait_for(stage, "the shares come out completely wrong")?;
        let later_text = kit.text_left(stage, &none_line, TALLY_SIZE, Palette::ink(), tally_at(2))?;
        let mid = (CHECK_Y[0] + CHECK_Y[1]) / 2.0;
        let shifted = step_ticks
            .iter()
            .zip(in_step(true))
            .map(|(&m, t)| move_recolor(m, v(program_x(t.to_f64()), mid), call_color(in_long_call(t)), 1.0))
            .collect::<fmn::Result<Vec<_>>>()?;
        fade_out_all(stage, &[in_step_text], 0.3)?;
        fade_in_with(stage, &[later_text], shifted, 1.0)?;
        let k08 = precisely(&kit, stage, K08)?;
        fade_in_all(stage, &[k08], 0.5)?;
        let k08_up = now(stage);
        kit.hold(stage)?;
        wait_until(stage, k08_up + read_s(K08))?;

        // 10_9: the panel goes and the corner picture grows back to its
        // place, the twelve passengers on their line; Terada, held until
        // read; it stands, then the chapter fades
        let line = say(&kit, stage, "10_9")?;
        let mut panel = prog.mobs();
        panel.extend(step_ticks);
        panel.extend(bracket);
        panel.extend([long_tally, long_calls, short_tally, short_calls, later_text, k08]);
        let mut regrow = vec![shrink_to(corner, 1.0 / CORNER_K, centre, CORNER_RT)?];
        for (&d, &h) in st.dots.iter().zip(&home) {
            regrow.push(shrink_to(d, 1.0 / CORNER_K, h, CORNER_RT)?);
        }
        // Clear the loop label before the returning timetable crosses its row.
        fade_out_all(stage, &panel, 0.3)?;
        fade_out_with(stage, &[], regrow, CORNER_RT)?;
        line.wait_for(stage, "In nineteen twenty-two")?;
        let terada = lower_third(&kit, stage, SOURCE, &s.terada)?;
        fade_in_all(stage, &[terada], 0.5)?;
        let terada_up = now(stage);
        kit.hold(stage)?;
        wait_until(stage, terada_up + read_s(&s.terada).max(STAND_UP))?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use explainer_kit::{Narrator, Script};

    use super::*;

    #[test]
    fn split_tallies_keep_every_whole_text_glyph_position() {
        struct Geometry(Kit);
        impl SceneConstruct for Geometry {
            fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
                let p = profile(&facts::facts().unwrap()).unwrap();
                for (row, text) in [p.long_tally(), p.short_tally()].iter().enumerate() {
                    let at = v(PROGRAM_X0, TALLY_YS[row]);
                    let whole = self.0.text_left(stage, text, TALLY_SIZE, palette::long_gap(), at)?;
                    let split = profile_tally(&self.0, stage, text, palette::long_gap(), at)?;
                    let reference = children(stage, whole);
                    let actual: Vec<_> = split.iter().flat_map(|&m| children(stage, m)).collect();
                    assert_eq!(reference.len(), actual.len());
                    let mut max_error = 0.0_f64;
                    for (a, b) in reference.iter().zip(&actual) {
                        let (a, b) = (stage.get_center(*a), stage.get_center(*b));
                        for axis in 0..3 {
                            max_error = max_error.max((a[axis] - b[axis]).abs());
                        }
                    }
                    // Font geometry crosses f32 storage; 1e-6 scene units is
                    // below 0.0002 px at1080p, far below the old 2.7px step.
                    assert!(max_error < 1e-6, "glyph displacement {max_error}");
                    let prefix = text.split_once(',').unwrap().0;
                    let unaligned = self.0.text_left(stage, prefix, TALLY_SIZE, palette::long_gap(), at)?;
                    let wrong = stage.get_center(children(stage, unaligned)[0]);
                    let correct = stage.get_center(reference[0]);
                    assert!((wrong[1] - correct[1]).abs() > 1e-3, "old placement must fail");
                    eprintln!(
                        "tally row {row}: max glyph error {max_error}; old vertical error {}",
                        wrong[1] - correct[1]
                    );
                }
                Ok(())
            }
        }
        fmn::run_scene(
            &mut Geometry(Kit::new().unwrap()),
            fmn::scene::RuntimeConfig::default(),
            7,
            &mut fmn::scene::NullSceneSink,
        )
        .unwrap();
    }

    /// The full draft with its line breaks and indents as single spaces.
    fn draft() -> String {
        crate::DRAFT.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    fn the_school() -> School {
        let f = facts::facts().unwrap();
        school(&f, &twelve_minutes::passengers().unwrap()).unwrap()
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

    /// Needs chapter 10's lines in script.tsv (the integration step adds
    /// them from the storyboard). The whole chapter runs headless at 10 fps
    /// with the scratch voice: every phrase it waits for is in its line,
    /// every fact it reads exists and matches the stretch and the program,
    /// and its nine lines are said in order. It ends with `kit.finish`,
    /// whose timeline runs past the end of the last line.
    #[test]
    fn the_chapter_runs_and_says_its_lines_in_order() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let narrator = Narrator::estimate(&script);
        let kit = Kit::new().unwrap().with_narrator(Rc::clone(&narrator));
        let mut scene = SameSum { kit };
        let config = fmn::scene::RuntimeConfig {
            fps: 10,
            ..fmn::scene::RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut fmn::scene::NullSceneSink).unwrap();
        let said_at = narrator.said();
        let said: Vec<&str> = said_at.iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(
            said,
            ["10_1", "10_2", "10_3", "10_4", "10_5", "10_6", "10_7", "10_8", "10_9"]
        );
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

    /// The students are the stretch's passengers: each sits in the class of
    /// the gap it stood in, the classes are as big as the gaps hold
    /// passengers, every student has its own seat inside its room, and its
    /// answer is its gap. A school of other sizes is refused: the negative
    /// control.
    #[test]
    fn the_students_are_the_stretchs_passengers() {
        let people = twelve_minutes::passengers().unwrap();
        let s = the_school();
        assert_eq!(s.size_values, vec![2, 10]);
        assert_eq!(class_color(0), timetable::short_gap());
        assert_eq!(class_color(1), palette::long_gap());
        let gs = twelve_minutes::gaps();
        for (p, &c) in people.iter().zip(&s.class_of) {
            assert_eq!(c, p.gap);
            assert_eq!(s.size_values[c], gs[p.gap].minutes(), "a student's answer is its gap");
        }
        let at = seats(&s.class_of).unwrap();
        for (i, a) in at.iter().enumerate() {
            let c = s.class_of[i];
            assert!((a[0] - ROOM_X[c]).abs() + STUDENT_R < ROOM_W[c] / 2.0, "student {i}");
            assert!((a[1] - ROOM_Y).abs() + STUDENT_R < ROOM_H / 2.0, "student {i}");
            // its tag stays in the room too
            assert!(a[1] - TAG_DY - 0.05 > ROOM_Y - ROOM_H / 2.0, "student {i}");
            for b in &at[i + 1..] {
                assert!(
                    (a[0] - b[0]).hypot(a[1] - b[1]) > 2.0 * STUDENT_R + 0.2,
                    "seats overlap"
                );
            }
        }
        assert!(seats(&[0, 0, 0]).is_err(), "the small class has two seats");
        let refused = school(&with("F-C1.sizes", "3,9"), &people).err().unwrap();
        assert!(refused.contains("F-C1.sizes"), "{refused}");
    }

    /// Averaged over classes the school says six, the sign's number;
    /// averaged over students it is eight and two thirds, the passengers'
    /// gap of 06. Each row levels off at its own average, and the two
    /// differ. A wrong students' average is refused.
    #[test]
    fn the_two_averages_are_the_buses_and_the_passengers() {
        let s = the_school();
        assert_eq!(s.school_level, Q::int(6));
        assert_eq!(s.student_level, Q::new(26, 3));
        assert_ne!(s.school_level, s.student_level);
        let answers: Vec<i64> = s.class_of.iter().map(|&c| s.size_values[c]).collect();
        assert_eq!(level(&answers), s.student_level);
        assert_eq!(level(&s.size_values), s.school_level);
        assert_eq!(s.school_sum(), "school's average: (2 + 10) ÷ 2 = 6");
        assert_eq!(
            s.student_sum(),
            "students' average: (2 × 2 + 10 × 10) ÷ 12 = 104 ÷ 12 = 8⅔"
        );
        assert_eq!(s.ghost(), "over passengers: 8 min 40 s");
        let people = twelve_minutes::passengers().unwrap();
        for (key, bad) in [
            ("F-C1.student_mean", "6"),
            ("F-C1.big_class_students", "9"),
            ("F-S1.person_gap_clock", "8:30"),
        ] {
            let refused = school(&with(key, bad), &people)
                .err()
                .expect("a wrong fact must be refused");
            assert!(refused.contains(key), "{refused}");
        }
    }

    /// The profiler measures time, not calls: the long call is half the
    /// loop's time and a sixth of its calls, and the even checks, which
    /// share no factor with the loop and run over whole loops, land in it
    /// exactly half the time. Over the first loop alone they do not: the
    /// equality needs whole loops, not just any stretch of checks.
    #[test]
    fn the_profiler_measures_time_not_calls() {
        let f = facts::facts().unwrap();
        let p = profile(&f).unwrap();
        assert_eq!(loop_ms(), 20);
        let even = clock(even_first(), even_every());
        assert_eq!(even.len(), 20);
        assert_eq!(share_in_long(&even), Q::new(1, 2));
        assert_eq!(Q::new(LONG_CALL_MS, loop_ms()), Q::new(1, 2));
        assert_eq!(Q::new(1, calls().len() as i64), Q::new(1, 6));
        // no check on a call's edge, where its color would be a coin toss:
        // every call starts on a whole millisecond, every check half way
        // between two
        for &t in &even {
            assert_eq!(t.den(), 2, "{t}");
        }
        let gcd = |mut a: i64, mut b: i64| {
            while b != 0 {
                (a, b) = (b, a % b);
            }
            a
        };
        assert_eq!(gcd(even_every().num(), loop_ms()), 1);
        let first_loop: Vec<Q> = even.iter().copied().filter(|&t| t < Q::int(loop_ms())).collect();
        assert_ne!(share_in_long(&first_loop), Q::new(1, 2));
        assert_eq!(p.long_tally(), "10 ms call: ½ of the time, ⅙ of the calls");
        assert_eq!(p.short_tally(), "2 ms calls: ½ of the time, ⅚ of the calls");
        assert_eq!(p.checks_line(), "checks landing in the 10 ms calls: ½");
        let refused = profile(&with("F-P1.long_call_share", "1/2")).err().unwrap();
        assert!(refused.contains("F-P1.long_call_share"), "{refused}");
    }

    /// The wrong clocks. Checks in step with the loop land in the same
    /// function every time: all in the long call, or, started a long call
    /// later, none. A clock every 4 ms gives three fifths or two fifths by
    /// where it starts. So no clock is promised the true share; the
    /// chapter's even checks get it because they are not in step and run
    /// over whole loops.
    #[test]
    fn a_clock_in_step_with_the_loop_gets_the_shares_wrong() {
        let p = profile(&facts::facts().unwrap()).unwrap();
        assert_eq!(share_in_long(&in_step(false)), Q::int(1));
        assert_eq!(share_in_long(&in_step(true)), Q::int(0));
        assert_eq!(in_step(false).len() as i64, LOOPS);
        assert_eq!(
            p.in_step_lines(),
            [
                "checks in step with the loop: all in the 10 ms calls".to_owned(),
                "started later: none in the 10 ms calls".to_owned()
            ]
        );
        assert_eq!(share_in_long(&clock(Q::new(1, 2), Q::int(4))), Q::new(3, 5));
        assert_eq!(share_in_long(&clock(Q::new(5, 2), Q::int(4))), Q::new(2, 5));
        assert!(all_or_none(Q::new(3, 5)).is_err());
    }

    /// Checks at random moments (a fixed-seed xorshift, uniform over the
    /// drawn loops) land in the long call about half the time, never a
    /// sixth: on average the samples measure time. The tolerance is about
    /// five standard deviations for this many checks.
    #[test]
    fn random_checks_measure_time_on_average() {
        let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        let span = (LOOPS * loop_ms()) as u64;
        let draws = 20_000_u64;
        let checks: Vec<Q> = (0..draws)
            .map(|_| Q::new((next() % (span * 1000)) as i64, 1000))
            .collect();
        let share = share_in_long(&checks).to_f64();
        assert!((share - 0.5).abs() < 0.02, "{share}");
        assert!((share - 1.0 / 6.0).abs() > 0.2, "{share}");
    }

    /// What the chapter prints is the storyboard's picture, word for word:
    /// the sums, the tallies, the question, the passengers' gap, the
    /// profiler's tallies and in-step lines, the call names and K08.
    #[test]
    fn the_printed_words_are_the_drafts() {
        let s = the_school();
        let p = profile(&facts::facts().unwrap()).unwrap();
        let board = draft();
        let [all_line, none_line] = p.in_step_lines();
        let [long_name, short_name] = p.names();
        for shown in [
            s.school_sum(),
            s.student_sum(),
            s.class_tally(),
            s.student_tally(),
            ASK.to_owned(),
            s.ghost(),
            CLASS_ROW.to_owned(),
            STUDENT_ROW.to_owned(),
            FOR_PROGRAMMERS.to_owned(),
            p.long_tally(),
            p.short_tally(),
            p.checks_line(),
            all_line,
            none_line,
            long_name,
            short_name,
            THE_LOOP.to_owned(),
            K08.to_owned(),
        ] {
            assert!(board.contains(&format!("\"{shown}\"")), "not in the draft: {shown}");
        }
    }

    /// The two lower-thirds are the storyboard's words, built from the
    /// facts' years; each fits the band in two lines (92 characters each)
    /// and stands at least as long as it takes to read at fifteen
    /// characters a second. The volume, pages and preprint identifier are
    /// the only digits in the fixed words, and they are the draft's.
    #[test]
    fn the_citations_are_the_drafts() {
        let s = the_school();
        let board = draft();
        for (card, year) in [(&s.fg, "1977"), (&s.terada, "1922")] {
            assert!(board.contains(&format!("\"{card}\"")), "not in the draft: {card}");
            assert!(card.contains(year), "{card}");
            assert!(card.chars().count() <= 2 * 92, "{card}");
            assert!(read_s(card) > 5.0, "{card}");
        }
        assert_eq!(
            s.fg,
            "Feld & Grofman, 'Variation in class size, the class size paradox, and some consequences for \
             students', Research in Higher Education 6 (1977) 215–222"
        );
        assert_eq!(
            s.terada,
            "Torahiko Terada, physicist, essay of 1922; English translation and discussion: N. Masuda & T. \
             Hiraoka, arXiv:2004.14081 (2020)"
        );
    }

    /// The fixed words carry no digits, except the citations' identifiers
    /// (checked above): every number on screen is read from the facts or
    /// counted from what is drawn.
    #[test]
    fn the_fixed_words_carry_no_numbers() {
        for w in [
            SCHOOL,
            STUDENTS,
            ASK,
            TIMES,
            DIVIDED,
            CLASS_ROW,
            STUDENT_ROW,
            CLASSES_OF,
            AND,
            SAY,
            OVER_PASSENGERS,
            SOURCE,
            FOR_PROGRAMMERS,
            MS,
            CALL,
            CALLS,
            OF_THE_TIME,
            OF_THE_CALLS,
            LANDING,
            IN_CALLS,
            IN_STEP,
            LATER,
            IN_THE,
            ALL,
            NONE,
            THE_LOOP,
            K08,
            FG_WORK,
            TERADA_WHO,
            TERADA_READ,
        ] {
            assert!(!w.chars().any(|c| c.is_ascii_digit()), "{w}");
        }
    }

    /// The texts up together stay apart and on screen. Widths are
    /// estimated, not measured: 0.0045 units a point a character, a line
    /// 0.6 of its size tall. The corner picture is chapter 04's stretch
    /// (its box from `twelve_minutes`' layout) at [`CORNER_K`].
    #[test]
    fn the_picture_stays_apart_and_on_screen() {
        let s = the_school();
        let p = profile(&facts::facts().unwrap()).unwrap();
        let w = |t: &str, pt: f64| t.chars().count() as f64 * pt * 0.0045;
        let h = |pt: f64| pt * 0.0045 / 0.55 * 0.6;
        type Box4 = (f64, f64, f64, f64);
        let centred = |t: &str, pt: f64, x: f64, y: f64| -> Box4 {
            (x - w(t, pt) / 2.0, x + w(t, pt) / 2.0, y - h(pt), y + h(pt))
        };
        let left = |t: &str, pt: f64, x: f64, y: f64| -> Box4 { (x, x + w(t, pt), y - h(pt), y + h(pt)) };
        let right = |t: &str, pt: f64, x: f64, y: f64| -> Box4 { (x - w(t, pt), x, y - h(pt), y + h(pt)) };
        let apart = |a: Box4, b: Box4| a.1 < b.0 || b.1 < a.0 || a.3 < b.2 || b.3 < a.2;
        let on_screen = |a: Box4| a.0 > -7.1 && a.1 < 7.1 && a.2 > -4.0 && a.3 < 4.0;
        // the stretch, full size: from its first marker to the last, from
        // its bracket labels to its markers' tops (twelve_minutes)
        let (x0, x1) = (twelve_minutes::ORIGIN_X - 0.07, twelve_minutes::ORIGIN_X + 6.0 + 0.07);
        let (y0, y1) = (twelve_minutes::AXIS_Y - 0.82 - 0.07, twelve_minutes::AXIS_Y + 0.44);
        let c = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
        let corner = (
            CORNER_AT[0] - CORNER_K * (c.0 - x0),
            CORNER_AT[0] + CORNER_K * (x1 - c.0),
            CORNER_AT[1] - CORNER_K * (c.1 - y0),
            CORNER_AT[1] + CORNER_K * (y1 - c.1),
        );
        let full = (x0, x1, y0, y1);
        let rooms: Vec<Box4> = (0..2)
            .map(|c| {
                (
                    ROOM_X[c] - ROOM_W[c] / 2.0,
                    ROOM_X[c] + ROOM_W[c] / 2.0,
                    ROOM_Y - ROOM_H / 2.0,
                    ROOM_Y + ROOM_H / 2.0,
                )
            })
            .collect();
        let class_row = (ROWS_X0, ROWS_X0 + ROWS_W, CLASS_BASE, CLASS_BASE + 10.0 * BAR_S);
        let student_row = (ROWS_X0, ROWS_X0 + ROWS_W, STUDENT_BASE, STUDENT_BASE + 10.0 * BAR_S);
        let captions = [
            right(CLASS_ROW, CAPTION_SIZE, CAPTION_X, CLASS_BASE + CAPTION_DYS[0]),
            right(&s.class_tally(), CAPTION_SIZE, CAPTION_X, CLASS_BASE + CAPTION_DYS[1]),
            right(STUDENT_ROW, CAPTION_SIZE, CAPTION_X, STUDENT_BASE + CAPTION_DYS[0]),
            right(
                &s.student_tally(),
                CAPTION_SIZE,
                CAPTION_X,
                STUDENT_BASE + CAPTION_DYS[1],
            ),
        ];
        let levels = [
            left(
                &s.school_mean,
                LEVEL_SIZE,
                LEVEL_X,
                CLASS_BASE + s.school_level.to_f64() * BAR_S,
            ),
            left(
                &s.student_mean,
                LEVEL_SIZE,
                LEVEL_X,
                STUDENT_BASE + s.student_level.to_f64() * BAR_S,
            ),
        ];
        let sums = [
            centred(&s.school_sum(), SUM_SIZE, SUM_X, SUM_YS[0]),
            centred(&s.student_sum(), SUM_SIZE, SUM_X, SUM_YS[1]),
        ];
        let ask = centred(ASK, SUM_SIZE, SUM_X, SUM_YS[1]);
        let ring = (
            THINK_AT[0] - 0.62,
            THINK_AT[0] + 0.62,
            THINK_AT[1] - 0.62,
            THINK_AT[1] + 0.62,
        );
        let note = centred("pause for more time", 18.0, THINK_AT[0], THINK_AT[1] - 0.62 - 0.3);
        let ghost = centred(&s.ghost(), GHOST_SIZE, GHOST_AT[0], GHOST_AT[1]);
        // a lower-third's band (`props::lower_third`): its foot 0.2 off the
        // frame's, 0.3 + 0.36 a line tall, wrapped at 92 characters
        let lines = |t: &str| t.chars().count().div_ceil(92) as f64;
        let third = |t: &str| (-6.71, 6.71, -3.8, -3.8 + 0.3 + 0.36 * lines(t));
        let panel = (PANEL_X[0], PANEL_X[1], PANEL_Y[0], PANEL_Y[1]);
        let header = left(FOR_PROGRAMMERS, HEADER_SIZE, HEADER_AT[0], HEADER_AT[1]);
        let calls_row = (
            program_x(0.0),
            program_x((LOOPS * loop_ms()) as f64),
            CALL_Y - CALL_H / 2.0,
            CALL_Y + CALL_H / 2.0,
        );
        let checks_row = (calls_row.0, calls_row.1, CHECK_Y[0], CHECK_Y[1]);
        let [long_name, short_name] = p.names();
        let names = [
            centred(&long_name, NAME_SIZE, program_x(LONG_CALL_MS as f64 / 2.0), NAME_Y),
            centred(
                &short_name,
                NAME_SIZE,
                program_x(LONG_CALL_MS as f64 + SHORT_CALL_MS as f64 / 2.0),
                NAME_Y,
            ),
        ];
        let loop_row = (program_x(0.0), program_x(loop_ms() as f64), LOOP_Y - LOOP_TICK, LOOP_Y);
        let loop_label = centred(
            THE_LOOP,
            NAME_SIZE,
            program_x(loop_ms() as f64 / 2.0),
            LOOP_Y + LOOP_LABEL_DY,
        );
        let [all_line, none_line] = p.in_step_lines();
        let tallies = [
            left(&p.long_tally(), TALLY_SIZE, PROGRAM_X0, TALLY_YS[0]),
            left(&p.short_tally(), TALLY_SIZE, PROGRAM_X0, TALLY_YS[1]),
            left(&p.checks_line(), TALLY_SIZE, PROGRAM_X0, TALLY_YS[2]),
            left(&all_line, TALLY_SIZE, PROGRAM_X0, TALLY_YS[2]),
            left(&none_line, TALLY_SIZE, PROGRAM_X0, TALLY_YS[2]),
        ];
        // inside the panel: everything but the panel's own frame
        for (i, &t) in [header, calls_row, checks_row, names[0], names[1], loop_row, loop_label]
            .iter()
            .chain(&tallies)
            .enumerate()
        {
            assert!(
                t.0 > panel.0 && t.1 < panel.1 && t.2 > panel.2 && t.3 < panel.3,
                "panel item {i} outside the frame: {t:?}"
            );
        }
        let moments: Vec<Vec<Box4>> = vec![
            // 10_4: the corner, rooms, rows, captions, the school's sum and
            // level, the question and its ring
            [
                vec![
                    corner,
                    rooms[0],
                    rooms[1],
                    class_row,
                    student_row,
                    sums[0],
                    ask,
                    ring,
                    note,
                    levels[0],
                ],
                captions.to_vec(),
            ]
            .concat(),
            // 10_5 and 10_6: both levels and sums, the passengers' gap, the
            // source
            [
                vec![
                    corner,
                    rooms[0],
                    rooms[1],
                    class_row,
                    student_row,
                    sums[0],
                    sums[1],
                    ghost,
                    third(&s.fg),
                ],
                captions.to_vec(),
                levels.to_vec(),
            ]
            .concat(),
            // 10_7, 10_8: the corner, the panel, K08 under it
            vec![corner, panel, third(K08)],
            // the panel's rows
            vec![
                header, calls_row, checks_row, names[0], names[1], tallies[0], tallies[1], tallies[2],
            ],
            vec![loop_row, loop_label, header, calls_row, checks_row, tallies[3]],
            // 10_9: the stretch back at full size, Terada's source
            vec![full, third(&s.terada)],
        ];
        for (m, up) in moments.iter().enumerate() {
            for (i, &a) in up.iter().enumerate() {
                assert!(on_screen(a), "moment {m}: item {i} off screen: {a:?}");
                for (j, &b) in up.iter().enumerate().skip(i + 1) {
                    assert!(apart(a, b), "moment {m}: items {i} and {j} overlap: {a:?} {b:?}");
                }
            }
        }
        // the rooms' students and their tags stay clear of the rows below
        assert!(ROOM_Y - ROOM_H / 2.0 > class_row.3 + 0.2);
    }
}
