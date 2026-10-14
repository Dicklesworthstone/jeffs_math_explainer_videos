//! Chapter 11, twelve hours: the Coast Guard's planner moves thousands of
//! dots and puts out, after an empty search, the share that search would
//! have seen (11_1); a lobsterman falls off his boat at night, about forty
//! miles off Long Island (11_2); the searchers guessed the evening before,
//! so the first map put its dots almost thirty miles too far north (11_3,
//! and 11_4 a question to carry); evidence moves the time of the fall hours
//! later (11_5); the map is run again, the dots move south, and a
//! helicopter flying through them sees him (11_6).
//!
//! The cloud is chapter 10's (`model::drift`, 2,000 seeded dots, each keeping
//! one error), placed around the point where the boat's track says he went
//! in. Every number shown is a fact (`F-C.*`, `F-D.flight.h3.*`) or a
//! `model::drift` value; the times are the facts' 24-hour strings, worded by
//! a tested helper.

use explainer_kit::animatic::HOLD_S;
use explainer_kit::props::{Pose, Swarm, lower_third};
use explainer_kit::stats::{Data, permutation};
use explainer_kit::{Facts, Kit, Palette, Timed, lerp, play, tween, v};
use fmn::core::color::interpolate_color;
use fmn::prelude::*;

use super::draw::{
    CHAR_UNITS_PER_POINT, balanced_lines, carry_pause, carry_question, fade_in_all, fade_out_all, faded_line, fail,
    flat, found, last_known, line_style, map as lit_blue, out as slate, say, search as teal, tag, text_lines,
    true_place, tween_linear, wrong,
};
use super::sound::SEARCH_MARK;
use crate::facts::facts;
use crate::model::drift;

pub struct TwelveHours {
    pub kit: Kit,
}

/// The narration lines this chapter speaks, in order.
pub const LINES: [&str; 6] = ["11_1", "11_2", "11_3", "11_4", "11_5", "11_6"];

// the chapter's fixed words (numbers are built from facts)
const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const AM: &str = "a.m.";
const PM: &str = "p.m.";
const CASP: &str = "CASP,";
const SAROPS_TEXT: &str = "thousands of particles for each scenario";
const LONG_ISLAND: &str = "Long Island";
const MONTAUK: &str = "Montauk";
const MODEL: &str = "model: each dot keeps its own guess of the wind and the current";
const FIRST_GUESS: &str = "first guess:";
const TOO_FAR_NORTH: &str = "too far north";
const ALMOST: &str = "almost";
const MILES: &str = "miles";
const OPEN_HATCH: &str = "open hatch";
const PUMPS: &str = "pumps running";
const SEEN: &str = "seen about";
/// 11_1's note under the cloud (R2: D14; Brown 1980 is in the description):
/// the proof of chapter 08 is for a wreck that stays put.
const DRIFT_NOTE: &str = "the proof of chapter 08 is for a wreck that stays put; for a target that drifts, planners move and reweight the dots, and the simple rule is no longer proved best";
/// 11_3's two tags (R2: M19): the guess's window, and the earlier point of
/// fall on the boat's track.
const EVENING_BEFORE: &str = "the evening before";
const EARLIER_NORTH: &str = "the earlier he fell, the farther north";
/// 11_6's lower-third (R2: S3; Tough, 2014; fact sheet, finding 7).
const FROZE_LABEL: &str = "The software froze:";
const FROZE_TEXT: &str = "the last track was drawn by hand through the high-chance area";

// ------------------------------------------------------------------ geometry

/// 11_1: scene units a sea mile is drawn as, and where the cloud stands.
const CLOUD_MILE: f64 = 0.9;
const CLOUD_AT: Vec3 = [0.0, 0.45, 0.0];
/// 11_2 on: scene units a sea mile is drawn as (forty of them are 2.8).
const MILE: f64 = 0.07;
/// The coast, the boat's track south from Montauk, and where it stands.
const COAST_Y: f64 = 3.15;
const TRACK_X: f64 = 1.4;
/// The track runs this far past where he fell: a drawing length, in miles,
/// for the open hatch and the pumps the boat was found with.
const TRACK_MILES: f64 = 66.0;
const HATCH_MILES: f64 = 56.0;
const PUMPS_MILES: f64 = 62.0;
/// The hour line down the left side: its x, its top and its bottom.
const LINE_X: f64 = -6.0;
const LINE_TOP: f64 = 3.0;
const LINE_BOTTOM: f64 = -2.2;
/// The layout lint's title-safe margin inside the frame, scene units.
const SAFE_MARGIN: f64 = 0.15;
/// The model tag's height: above the caption band's question lines (the
/// carried question of 11_4 stands at -2.85), clear of the track's labels.
const MODEL_Y: f64 = -2.3;
/// How opaque a dot of the cloud is when lit, deepened, and put out.
const LIT_DIM: f64 = 0.55;
const LIT_DEEP: f64 = 0.95;
const OUT_OPACITY: f64 = 0.8;
/// Half the length of the helicopter's track, scene units.
const HELI_HALF: f64 = 1.2;
/// How much wider a tag's back is than its words, scene units.
const TAG_PAD: f64 = 0.3;
/// 11_1's note under the cloud: its first line's height, the lines' spacing,
/// their type size and how many characters a line holds.
const NOTE_Y: f64 = -2.1;
const NOTE_STEP: f64 = 0.3;
const NOTE_PT: f64 = 20.0;
const NOTE_CHARS: usize = 92;
/// 11_3's tags: type size; the guess's tag stands this far over its label;
/// the earlier point's ring has this radius and its tag stands this far
/// right of it.
const TAG_PT: f64 = 20.0;
const EVENING_RISE: f64 = 0.4;
const EARLIER_RADIUS: f64 = 0.09;
const EARLIER_TAG_GAP: f64 = 0.45;
/// 11_5's bracket over the gap between the guess's middle and the fall: its
/// x (right of the clock labels), how far its serifs reach, the words' type
/// size, and how far under the window's label it starts.
const GAP_X: f64 = -4.1;
const GAP_SERIF: f64 = 0.12;
const GAP_PT: f64 = 24.0;
const GAP_UNDER: f64 = 0.22;
/// 11_6's note about the buoys: type size, line spacing, how many characters
/// a line holds, how far under the track it stands, and its right end's
/// distance from the helicopter's track.
const BUOYS_PT: f64 = 18.0;
const BUOYS_STEP: f64 = 0.27;
const BUOYS_CHARS: usize = 42;
const BUOYS_DROP: f64 = 0.9;
const BUOYS_GAP: f64 = 0.15;

/// Where the boat's track stands `miles` from the coast.
fn track_point(miles: f64) -> Vec3 {
    v(TRACK_X, COAST_Y - miles * MILE)
}

/// How wide a tag of `words` at `pt` points runs, its back included.
fn tag_width(words: &str, pt: f64) -> f64 {
    words.chars().count() as f64 * pt * CHAR_UNITS_PER_POINT + TAG_PAD
}

/// Where the earlier point's tag stands (its middle): right of the ring on
/// the track at the guess's early end, and half way between the coast line
/// and the first map's point, so it meets neither.
fn earlier_tag_at(night: &Night) -> Vec3 {
    let early = track_point(night.miles_at(night.guess.0));
    let first_map = track_point(night.miles_at(night.middle));
    v(
        early[0] + EARLIER_TAG_GAP + tag_width(EARLIER_NORTH, TAG_PT) / 2.0,
        (COAST_Y + first_map[1]) / 2.0,
    )
}

/// How wide the widest of `lines` runs at `pt` points (no back).
fn lines_width(lines: &[String], pt: f64) -> f64 {
    lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as f64 * pt * CHAR_UNITS_PER_POINT
}

/// 11_5's words over the gap: "5 hours" ("1 hour" for one), the fact's number.
fn hours_words(hours: &str) -> String {
    format!("{hours} {}", if hours == "1" { "hour" } else { "hours" })
}

/// 11_6's note beside the green point (R2: M10, D12): he held on to two
/// buoys, so chapter 10's chance (`percent` in a hundred) is not about him.
fn buoys_words(percent: &str) -> String {
    format!(
        "held on to two lobster buoys: not a free drifter, so chapter 10's {percent} in 100 is not a number about him"
    )
}

/// The middle of 11_6's note about the buoys, from its lines: its right end
/// a little left of the helicopter's track.
fn buoys_centre_x(lines: &[String]) -> f64 {
    TRACK_X - HELI_HALF - BUOYS_GAP - lines_width(lines, BUOYS_PT) / 2.0
}

// ---------------------------------------------------------------- the times

/// Minutes after midnight of an "HH:MM" string.
fn clock_minutes(s: &str) -> Option<u32> {
    let (h, m) = s.split_once(':')?;
    let (h, m): (u32, u32) = (h.parse().ok()?, m.parse().ok()?);
    (h < 24 && m < 60).then_some(h * 60 + m)
}

/// The hour and the half of the day in a 12-hour clock.
fn twelve(minutes: u32) -> (u32, &'static str) {
    let h = minutes / 60 % 24;
    match h {
        0 => (12, AM),
        1..=11 => (h, AM),
        12 => (12, PM),
        _ => (h - 12, PM),
    }
}

/// A time as the screen words it: 21:30 is "9:30 p.m.", 03:30 "3:30 a.m.".
fn twelve_hour(minutes: u32) -> String {
    let (h, half) = twelve(minutes);
    format!("{h}:{:02} {half}", minutes % 60)
}

/// A whole hour on the hour line: "9 p.m.".
fn hour_word(minutes: u32) -> String {
    let (h, half) = twelve(minutes);
    format!("{h} {half}")
}

/// A window "21:30-23:30" as the screen words it: "9:30 to 11:30 p.m."
/// (the half of the day once, when both ends share it).
fn window_words(window: &str) -> Option<String> {
    let (a, b) = window.split_once('-')?;
    let (a, b) = (clock_minutes(a)?, clock_minutes(b)?);
    let (ta, tb) = (twelve_hour(a), twelve_hour(b));
    let (na, ha) = ta.rsplit_once(' ')?;
    let (_, hb) = tb.rsplit_once(' ')?;
    Some(if ha == hb {
        format!("{na} to {tb}")
    } else {
        format!("{ta} to {tb}")
    })
}

/// A date "2013-07-24" as the screen words it: "24 July 2013".
fn date_words(iso: &str) -> Option<String> {
    let mut parts = iso.split('-');
    let (y, m, d): (u32, usize, u32) = (
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    );
    let month = MONTHS.get(m.checked_sub(1)?)?;
    Some(format!("{d} {month} {y}"))
}

/// A stay "11h28" as the screen words it: "11 hours 28 minutes".
fn stay_words(s: &str) -> Option<String> {
    let (h, m) = s.split_once('h')?;
    let (h, m): (u32, u32) = (h.parse().ok()?, m.parse().ok()?);
    let unit = |n: u32, one: &'static str, many: &'static str| if n == 1 { one } else { many };
    Some(format!(
        "{h} {} {m} {}",
        unit(h, "hour", "hours"),
        unit(m, "minute", "minutes")
    ))
}

/// The night as the facts give it, in minutes since the hour the first
/// guess's window opens (the hour line's top): the window, its middle, the
/// fall, the re-run of the map; and in sea miles how far off the coast he
/// went in and how far north of that the first map put its dots.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Night {
    /// Clock minutes of the hour line's top.
    start: u32,
    guess: (f64, f64),
    middle: f64,
    fell: f64,
    rerun: f64,
    seen: f64,
    offshore: f64,
    south: f64,
}

impl Night {
    fn from_facts(f: &Facts) -> Result<Night, String> {
        let get = |key: &str| f.get(key).ok_or_else(|| format!("no fact {key}"));
        let clock = |key: &str| {
            let s = get(key)?;
            clock_minutes(s).ok_or_else(|| format!("{key}: {s} is not a time"))
        };
        let window = get("F-C.aldridge.guess")?;
        let (a, b) = window
            .split_once('-')
            .and_then(|(a, b)| Some((clock_minutes(a)?, clock_minutes(b)?)))
            .ok_or_else(|| format!("F-C.aldridge.guess: {window} is not a window"))?;
        // the hour line starts on the hour the window opens
        let start = a / 60 * 60;
        let since = |c: u32| f64::from((c + 24 * 60 - start) % (24 * 60));
        let number = |key: &str| -> Result<f64, String> {
            let s = get(key)?;
            s.split('-')
                .next()
                .and_then(|n| n.parse().ok())
                .ok_or_else(|| format!("{key}: {s} is not a number"))
        };
        Ok(Night {
            start,
            guess: (since(a), since(b)),
            middle: since(clock("F-C.aldridge.guess.middle")?),
            fell: since(clock("F-C.aldridge.fell")?),
            rerun: since(clock("F-C.aldridge.rerun")?),
            seen: since(clock("F-C.aldridge.seen")?),
            offshore: number("F-C.aldridge.miles_offshore")?,
            south: number("F-C.aldridge.miles_south")?,
        })
    }

    /// Hours from night-minute `a` to `b`.
    fn hours_between(&self, a: f64, b: f64) -> f64 {
        (b - a) / 60.0
    }

    /// The boat's speed, in miles an hour: the miles the first guess put the
    /// fall short of the truth, over the hours it was early.
    fn speed(&self) -> f64 {
        self.south / self.hours_between(self.middle, self.fell)
    }

    /// Where the boat's track stands at night-minute `t`, in miles off the
    /// coast (at a steady speed, never short of the coast).
    fn miles_at(&self, t: f64) -> f64 {
        (self.offshore - self.speed() * self.hours_between(t, self.fell)).max(0.0)
    }

    /// The hour line's last night-minute: the hour after the fall.
    fn bottom(&self) -> f64 {
        (self.fell / 60.0).ceil() * 60.0
    }

    /// Where the hour line puts night-minute `t`.
    fn line_y(&self, t: f64) -> f64 {
        LINE_TOP + (LINE_BOTTOM - LINE_TOP) * t / self.bottom()
    }

    /// The clock minutes of night-minute `t`.
    fn clock_of(&self, t: f64) -> u32 {
        (self.start + t.round() as u32) % (24 * 60)
    }
}

/// Where one dot of the Aldridge cloud stands when the map is run for a fall
/// at night-minute `t` and looked at at the re-run: around where the track
/// has him at `t`, spread by the model's drift over the hours between.
fn aldridge_place(d: &drift::Dot, night: &Night, t: f64) -> Vec3 {
    let c = track_point(night.miles_at(t));
    let (ox, oy) = drift::offset(d, night.hours_between(t, night.rerun));
    v(c[0] + ox * MILE, c[1] + oy * MILE)
}

/// Which dots of `cloud` a search of the square of `side` miles, centred on
/// the cloud `hours` in, puts out: of the dots inside it, exactly
/// `round(inside × seeing)`, picked by a seeded draw. A dot outside the
/// square is never put out.
fn put_out(cloud: &[drift::Dot], hours: f64, side: f64, seeing: f64, seed: u64) -> Vec<bool> {
    let half = side / 2.0;
    let inside: Vec<usize> = (0..cloud.len())
        .filter(|&i| {
            let (x, y) = drift::offset(&cloud[i], hours);
            x.abs() <= half && y.abs() <= half
        })
        .collect();
    let n = (inside.len() as f64 * seeing).round() as usize;
    let order = permutation(inside.len(), &mut Data::new(seed));
    let mut gone = vec![false; cloud.len()];
    for &k in order.iter().take(n) {
        gone[inside[k]] = true;
    }
    gone
}

/// The zigzag of `passes` straight passes across a square of `side` centred
/// at `at`.
fn zigzag(at: Vec3, side: f64, passes: usize) -> Vec<Vec3> {
    let mut pts = Vec::with_capacity(2 * passes);
    for k in 0..passes {
        let y = at[1] + side / 2.0 - side * (k as f64 + 0.5) / passes as f64;
        let (a, b) = (at[0] - side / 2.0, at[0] + side / 2.0);
        if k % 2 == 0 {
            pts.extend([v(a, y), v(b, y)]);
        } else {
            pts.extend([v(b, y), v(a, y)]);
        }
    }
    pts
}

/// A closed outline of a square of `side` centred at `at`.
fn square_outline(at: Vec3, side: f64) -> Vec<Vec3> {
    let h = side / 2.0;
    vec![
        v(at[0] - h, at[1] - h),
        v(at[0] + h, at[1] - h),
        v(at[0] + h, at[1] + h),
        v(at[0] - h, at[1] + h),
        v(at[0] - h, at[1] - h),
    ]
}

/// A closed outline of an open diamond at `at`.
fn diamond(at: Vec3, r: f64) -> Vec<Vec3> {
    vec![
        v(at[0], at[1] + r),
        v(at[0] + r, at[1]),
        v(at[0], at[1] - r),
        v(at[0] - r, at[1]),
        v(at[0], at[1] + r),
    ]
}

/// A swarm of the cloud's dots, every one at no opacity until its tracker
/// brings it in; `at` gives each dot's place.
fn cloud_swarm(
    stage: &mut Stage<'_>,
    cloud: &[drift::Dot],
    radius: f64,
    at: impl Fn(&drift::Dot) -> Vec3,
) -> fmn::Result<Swarm> {
    let poses: Vec<Pose> = cloud
        .iter()
        .map(|d| {
            let p = at(d);
            Pose {
                x: p[0],
                y: p[1],
                heading: 0.0,
            }
        })
        .collect();
    let template = Dot::new().radius(radius).style(flat(lit_blue(), 1.0)).build();
    let swarm = Swarm::build(stage, &template, &poses, None)?;
    for &m in &swarm.members {
        stage.set_opacity(m, 0.0, false);
    }
    Ok(swarm)
}

impl SceneConstruct for TwelveHours {
    fn name(&self) -> &str {
        "11_twelve_hours"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let all = facts().map_err(fail)?;
        let fact = |key: &str| -> fmn::Result<String> {
            all.get(key)
                .map(str::to_owned)
                .ok_or_else(|| fail(format!("no fact {key}")))
        };
        let night = Night::from_facts(&all).map_err(fail)?;
        let cloud = drift::cloud();

        // ------------------------------------------------------- 11_1
        // the planner's cloud: a search over part of it comes back empty, the
        // dots it would have seen go out, and the rest of the map deepens
        let line = say(&kit, stage, LINES[0])?;
        let hours = drift::hours()[1].f64();
        let side: f64 = fact("F-D.flight.h3.best_side")?
            .parse()
            .map_err(|_| fail("F-D.flight.h3.best_side is not a number".into()))?;
        let flight = drift::flight(&cloud, hours, side as u32);
        let gone = put_out(&cloud, hours, side, flight.seeing, drift::SEED + 11);
        let first = cloud_swarm(stage, &cloud, 0.025, |d| {
            let (ox, oy) = drift::offset(d, hours);
            v(CLOUD_AT[0] + ox * CLOUD_MILE, CLOUD_AT[1] + oy * CLOUD_MILE)
        })?;
        let (shown, out_t, deepen) = (
            stage.add_value_tracker(0.0),
            stage.add_value_tracker(0.0),
            stage.add_value_tracker(0.0),
        );
        // the dots' looks are recounted from three trackers, so no group of
        // two thousand is ever faded or recoloured as one
        let members = first.members.clone();
        let flags = gone.clone();
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        let mut last = [f64::NAN; 3];
        stage.add_updater(
            anchor,
            move |s, _| {
                let now = [
                    s.tracker_value(shown).unwrap_or(0.0),
                    s.tracker_value(out_t).unwrap_or(0.0),
                    s.tracker_value(deepen).unwrap_or(0.0),
                ];
                if now == last {
                    return;
                }
                for (&m, &out) in members.iter().zip(&flags) {
                    let (c, o) = if out {
                        (
                            interpolate_color(lit_blue(), slate(), now[1]),
                            lerp(LIT_DIM, OUT_OPACITY, now[1]),
                        )
                    } else {
                        (lit_blue(), lerp(LIT_DIM, LIT_DEEP, now[2]))
                    };
                    s.set_fill(m, Some(c), Some(now[0] * o), None, false);
                }
                last = now;
            },
            true,
        )?;
        stage.add_to_scene(first.group)?;
        stage.play(tween(shown, 1.0, 0.9)?)?;
        // the names stand under the picture from the year they are named, so
        // they can be read before the line is over
        let planner = lower_third(
            &kit,
            stage,
            &format!("{CASP} {}.", fact("F-C.casp")?),
            &format!("SAROPS, since {}: {SAROPS_TEXT}", fact("F-C.sarops")?),
        )?;
        line.wait_for(stage, "since nineteen")?;
        fade_in_all(stage, &[planner], 0.5)?;
        // for a target that drifts the simple rule is no longer proved best
        // (R2: D14): a small note under the cloud, over the lower-third, in
        // while the planner's dots are named
        let note_lines = balanced_lines(DRIFT_NOTE, NOTE_CHARS);
        let drift_note = text_lines(
            &kit,
            stage,
            &note_lines,
            NOTE_PT,
            Palette::dim(),
            CLOUD_AT[0],
            NOTE_Y,
            NOTE_STEP,
        )?;
        line.wait_for(stage, "It moves thousands")?;
        fade_in_all(stage, &[drift_note], 0.6)?;
        // the search: a square laid over part of the cloud, its track
        let square = stage.arena_mut().add(
            polyline(square_outline(CLOUD_AT, side * CLOUD_MILE))
                .style(line_style(teal(), 4.0))
                .build(),
        );
        let track = stage.arena_mut().add(
            polyline(zigzag(CLOUD_AT, side * CLOUD_MILE, 6))
                .style(faded_line(teal(), 2.5, 0.8))
                .build(),
        );
        line.wait_for(stage, "and updates them")?;
        fade_in_all(stage, &[square], 0.5)?;
        kit.mark(stage, SEARCH_MARK);
        play!(stage; show_creation(track).rt(0.9));
        play!(stage; tween(out_t, 1.0, 0.9)?, tween(deepen, 1.0, 0.9)?);
        line.wait_end(stage)?;

        // ------------------------------------------------------- 11_2
        // a lobsterman falls off his boat at night, about forty miles off
        // Long Island, and nobody sees him go
        let line = say(&kit, stage, LINES[1])?;
        let mut away: Vec<Box<dyn Animation>> = vec![stage.prepare(tween(shown, 0.0, 0.6)?)?];
        for m in [square, track, planner, drift_note] {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            away.push(stage.prepare(a.rt(0.6))?);
        }
        stage.play_prepared(away)?;
        stage.clear_updaters(anchor, true);
        stage.remove_from_scene(first.group);
        stage.remove_from_scene(anchor);

        // the land and its coast end at the safe area's edge: laid out to the
        // frame they were flagged as off the title-safe stage
        let (safe_x, safe_y) = (FRAME_X_RADIUS - SAFE_MARGIN, FRAME_Y_RADIUS - SAFE_MARGIN);
        let land = stage.arena_mut().add(
            Polygon::new(vec![
                v(-safe_x, COAST_Y),
                v(safe_x, COAST_Y),
                v(safe_x, safe_y),
                v(-safe_x, safe_y),
            ])
            .style(flat(Palette::faint(), 0.35))
            .build(),
        );
        let coast = stage.arena_mut().add(
            polyline(vec![v(-safe_x, COAST_Y), v(safe_x, COAST_Y)])
                .style(line_style(Palette::dim(), 3.0))
                .build(),
        );
        let montauk_dot = stage.arena_mut().add(
            Dot::new()
                .point(track_point(0.0))
                .radius(0.08)
                .style(flat(Palette::ink(), 1.0))
                .build(),
        );
        let places = [
            kit.text(stage, LONG_ISLAND, 26.0, Palette::dim(), v(-3.4, COAST_Y + 0.38))?,
            kit.text(stage, MONTAUK, 24.0, Palette::dim(), v(TRACK_X, COAST_Y + 0.38))?,
            kit.text(
                stage,
                &date_words(&fact("F-C.aldridge.date")?).ok_or_else(|| fail("the date does not parse".into()))?,
                26.0,
                Palette::ink(),
                v(5.5, COAST_Y + 0.38),
            )?,
        ];
        let mut scenery: Vec<Mob> = vec![land, coast, montauk_dot];
        scenery.extend(places);
        // the hour line down the side, an hour to a tick
        let hour_line = stage.arena_mut().add(
            polyline(vec![v(LINE_X, LINE_TOP), v(LINE_X, LINE_BOTTOM)])
                .style(line_style(Palette::dim(), 2.5))
                .build(),
        );
        scenery.push(hour_line);
        let mut tick_at = 0.0;
        while tick_at <= night.bottom() {
            let y = night.line_y(tick_at);
            scenery.push(
                stage.arena_mut().add(
                    polyline(vec![v(LINE_X - 0.1, y), v(LINE_X + 0.1, y)])
                        .style(line_style(Palette::dim(), 2.5))
                        .build(),
                ),
            );
            scenery.push(kit.text(
                stage,
                &hour_word(night.clock_of(tick_at)),
                20.0,
                Palette::dim(),
                v(LINE_X - 0.62, y),
            )?);
            tick_at += 60.0;
        }
        let model_tag = kit.text_left(stage, MODEL, 20.0, Palette::dim(), v(0.9, MODEL_Y))?;
        scenery.push(model_tag);
        fade_in_all(stage, &scenery, 0.9)?;

        // the boat goes down the track through the night, the clock's mark
        // beside it on the hour line; where he went in, a hollow point stays
        let clock = stage.add_value_tracker(0.0);
        let wake = stage.always_redraw(move |s| {
            let m = night.miles_at(s.tracker_value(clock).unwrap_or(0.0)).max(0.05);
            s.add(
                polyline(vec![track_point(0.0), track_point(m)])
                    .style(line_style(Palette::dim(), 3.0))
                    .build(),
            )
        });
        let boat = stage.always_redraw(move |s| {
            let m = night.miles_at(s.tracker_value(clock).unwrap_or(0.0));
            s.add(
                Dot::new()
                    .point(track_point(m))
                    .radius(0.07)
                    .style(flat(Palette::ink(), 1.0))
                    .build(),
            )
        });
        let clock_mark = stage.always_redraw(move |s| {
            let t = s.tracker_value(clock).unwrap_or(0.0);
            s.add(
                Dot::new()
                    .point(v(LINE_X, night.line_y(t)))
                    .radius(0.1)
                    .style(flat(Palette::ink(), 1.0))
                    .build(),
            )
        });
        for m in [wake, boat, clock_mark] {
            stage.add_to_scene(m)?;
        }
        line.wait_for(stage, "at night")?;
        stage.play(tween_linear(clock, night.fell, 5.0)?)?;
        let went_in = stage.arena_mut().add(
            Circle::new()
                .radius(0.14)
                .arc_center(track_point(night.offshore))
                .style(line_style(true_place(), 3.0))
                .build(),
        );
        fade_in_all(stage, &[went_in], 0.4)?;
        line.wait_end(stage)?;

        // ------------------------------------------------------- 11_3
        // the searchers guessed the evening before: the first map's dots sit
        // almost thirty miles too far north
        let line = say(&kit, stage, LINES[2])?;
        for m in [wake, boat, clock_mark] {
            stage.clear_updaters(m, true);
        }
        fade_out_all(stage, &[boat, clock_mark], 0.4)?;
        let (win_top, win_bottom) = (night.line_y(night.guess.0), night.line_y(night.guess.1));
        let window = stage.arena_mut().add(
            Polygon::new(vec![
                v(LINE_X + 0.06, win_top),
                v(LINE_X + 0.36, win_top),
                v(LINE_X + 0.36, win_bottom),
                v(LINE_X + 0.06, win_bottom),
            ])
            .style(flat(wrong(), 0.55))
            .build(),
        );
        let guess_words =
            window_words(&fact("F-C.aldridge.guess")?).ok_or_else(|| fail("the window does not parse".into()))?;
        let window_label = kit.text_left(
            stage,
            &format!("{FIRST_GUESS} {guess_words}"),
            24.0,
            wrong(),
            v(LINE_X + 0.6, night.line_y(night.middle)),
        )?;
        // the window is tagged in plain words (R2: M19): the evening before;
        // its back's left edge stands a little left of the label's words
        let evening = tag(
            &kit,
            stage,
            EVENING_BEFORE,
            v(
                LINE_X + 0.45 + tag_width(EVENING_BEFORE, TAG_PT) / 2.0,
                night.line_y(night.middle) + EVENING_RISE,
            ),
            TAG_PT,
        )?;
        let fall = stage.add_value_tracker(night.middle);
        let fall_mark = stage.always_redraw(move |s| {
            let t = s.tracker_value(fall).unwrap_or(0.0);
            s.add(
                Dot::new()
                    .point(v(LINE_X, night.line_y(t)))
                    .radius(0.1)
                    .style(flat(Palette::ink(), 1.0))
                    .build(),
            )
        });
        line.wait_for(stage, "guessed the evening before")?;
        fade_in_all(stage, &[window, window_label, evening], 0.6)?;
        stage.add_to_scene(fall_mark)?;
        // the cloud of the first map, and the point it is run from
        line.wait_for(stage, "So the first map")?;
        let map_time = stage.add_value_tracker(night.middle);
        let second = cloud_swarm(stage, &cloud, 0.012, |d| aldridge_place(d, &night, night.middle))?;
        let shown2 = stage.add_value_tracker(0.0);
        let members2 = second.members.clone();
        let walkers = cloud.clone();
        let mut at: Vec<Vec3> = walkers
            .iter()
            .map(|d| aldridge_place(d, &night, night.middle))
            .collect();
        let anchor2 = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor2)?;
        let mut last2 = (f64::NAN, f64::NAN);
        stage.add_updater(
            anchor2,
            move |s, _| {
                let now = (
                    s.tracker_value(map_time).unwrap_or(0.0),
                    s.tracker_value(shown2).unwrap_or(0.0),
                );
                if now == last2 {
                    return;
                }
                if now.1 != last2.1 {
                    for &m in &members2 {
                        s.set_opacity(m, now.1 * LIT_DEEP, false);
                    }
                }
                if now.0 != last2.0 {
                    for ((&m, d), cur) in members2.iter().zip(&walkers).zip(at.iter_mut()) {
                        let p = aldridge_place(d, &night, now.0);
                        if p != *cur {
                            s.shift(m, [p[0] - cur[0], p[1] - cur[1], 0.0]);
                            *cur = p;
                        }
                    }
                }
                last2 = now;
            },
            true,
        )?;
        stage.add_to_scene(second.group)?;
        let assumed = stage.always_redraw(move |s| {
            let t = s.tracker_value(map_time).unwrap_or(0.0);
            s.add(
                polyline(diamond(track_point(night.miles_at(t)), 0.12))
                    .style(line_style(last_known(), 3.0))
                    .build(),
            )
        });
        stage.add_to_scene(assumed)?;
        stage.play(tween(shown2, 1.0, 0.9)?)?;
        // the distance between the first map's point and where he went in
        let dx = TRACK_X + 0.9;
        let (north, south) = (
            track_point(night.miles_at(night.middle))[1],
            track_point(night.offshore)[1],
        );
        let span = stage.arena_mut().add(
            polyline(vec![v(dx - 0.1, north), v(dx, north), v(dx, south), v(dx - 0.1, south)])
                .style(line_style(wrong(), 3.0))
                .build(),
        );
        let span_label = kit.text_left(
            stage,
            &format!("{ALMOST} {} {MILES} {TOO_FAR_NORTH}", fact("F-C.aldridge.miles_south")?),
            24.0,
            wrong(),
            v(dx + 0.2, (north + south) / 2.0),
        )?;
        // an earlier fall puts the first map's point farther north still: the
        // guess's early end, on the track, is a second point of fall (R2: M19)
        let earlier_at = track_point(night.miles_at(night.guess.0));
        let earlier = stage.arena_mut().add(
            Circle::new()
                .radius(EARLIER_RADIUS)
                .arc_center(earlier_at)
                .style(line_style(wrong(), 3.0))
                .build(),
        );
        let earlier_tag = tag(&kit, stage, EARLIER_NORTH, earlier_tag_at(&night), TAG_PT)?;
        fade_in_all(stage, &[span, span_label, earlier, earlier_tag], 0.6)?;
        line.wait_end(stage)?;

        // ------------------------------------------------------- 11_4
        // a question to carry: what could fix a map whose story was wrong?
        let line = say(&kit, stage, LINES[3])?;
        let question = carry_question(&kit, stage, LINES[3])?;
        line.wait_last(stage)?;
        fade_in_all(stage, &[question], 0.4)?;
        carry_pause(&kit, stage)?;
        fade_out_all(stage, &[question], 0.3)?;

        // ------------------------------------------------------- 11_5
        // evidence: an open hatch and pumps running, work he left until the
        // boat was far offshore, so he had fallen hours later
        let line = say(&kit, stage, LINES[4])?;
        let far = stage.arena_mut().add(
            polyline(vec![
                track_point(night.miles_at(night.bottom())),
                track_point(TRACK_MILES),
            ])
            .style(line_style(Palette::dim(), 3.0))
            .build(),
        );
        line.wait_for(stage, "On deck")?;
        play!(stage; show_creation(far).rt(0.8));
        let mut evidence: Vec<Mob> = Vec::new();
        for (miles, words, phrase) in [
            (HATCH_MILES, OPEN_HATCH, "an open hatch"),
            (PUMPS_MILES, PUMPS, "pumps running"),
        ] {
            line.wait_for(stage, phrase)?;
            let at = track_point(miles);
            let mark = stage.arena_mut().add(
                Dot::new()
                    .point(at)
                    .radius(0.09)
                    .style(flat(Palette::ink(), 1.0))
                    .build(),
            );
            let label = kit.text_left(stage, words, 22.0, Palette::dim(), v(at[0] + 0.2, at[1]))?;
            fade_in_all(stage, &[mark, label], 0.4)?;
            evidence.extend([mark, label]);
        }
        // the red window is struck out and stays behind; the mark slides
        line.wait_for(stage, "He had fallen hours later")?;
        let strike = stage.arena_mut().add(
            polyline(vec![v(LINE_X + 0.0, win_top), v(LINE_X + 0.42, win_bottom)])
                .style(line_style(wrong(), 4.0))
                .build(),
        );
        let first_time = kit.text_left(
            stage,
            &twelve_hour(night.clock_of(night.middle)),
            22.0,
            wrong(),
            v(LINE_X + 0.6, night.line_y(night.middle) - 0.36),
        )?;
        let last_time = kit.text_left(
            stage,
            &twelve_hour(night.clock_of(night.fell)),
            22.0,
            Palette::ink(),
            v(LINE_X + 0.3, night.line_y(night.fell)),
        )?;
        play!(stage;
            fade_in(stage.arena_mut(), strike, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), first_time, ORIGIN, 1.0)?.rt(0.5),
            tween(fall, night.fell, 3.0)?,
        );
        fade_in_all(stage, &[last_time], 0.4)?;
        // where the slider stops: the gap from the guess's middle to the
        // fall is five hours, written over it (R2: M20), so that 14_3's "five
        // hours off" has its antecedent
        let (gap_top, gap_bottom) = (night.line_y(night.middle) - GAP_UNDER, night.line_y(night.fell));
        let gap = stage.arena_mut().add(
            polyline(vec![
                v(GAP_X - GAP_SERIF, gap_top),
                v(GAP_X, gap_top),
                v(GAP_X, gap_bottom),
                v(GAP_X - GAP_SERIF, gap_bottom),
            ])
            .style(line_style(Palette::ink(), 3.0))
            .build(),
        );
        let gap_words = kit.text_left(
            stage,
            &hours_words(&fact("F-C.aldridge.hours_off")?),
            GAP_PT,
            Palette::ink(),
            v(GAP_X + 0.25, (gap_top + gap_bottom) / 2.0),
        )?;
        fade_in_all(stage, &[gap, gap_words], 0.5)?;
        line.wait_end(stage)?;

        // ------------------------------------------------------- 11_6
        // the map is run again from the new time: the dots move south, a
        // helicopter flying through them sees him alive
        let line = say(&kit, stage, LINES[5])?;
        line.wait_for(stage, "ran the map again")?;
        stage.play(tween(map_time, night.fell, 3.0)?)?;
        line.wait_for(stage, "a helicopter")?;
        let y0 = track_point(night.offshore)[1];
        let west = stage.arena_mut().add(
            polyline(vec![v(TRACK_X - HELI_HALF, y0), v(TRACK_X, y0)])
                .style(line_style(teal(), 5.0))
                .build(),
        );
        let east = stage.arena_mut().add(
            polyline(vec![v(TRACK_X, y0), v(TRACK_X + HELI_HALF, y0)])
                .style(line_style(teal(), 5.0))
                .build(),
        );
        play!(stage; show_creation(west).rt(0.9));
        // the track reaches him: one plucked note, and the point turns green
        kit.mark(stage, "pizz:found");
        let saved = stage.arena_mut().add(
            Dot::new()
                .point(track_point(night.offshore))
                .radius(0.16)
                .style(flat(found(), 1.0))
                .build(),
        );
        play!(stage;
            show_creation(east).rt(0.9),
            fade_out(stage.arena_mut(), went_in, ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), saved, ORIGIN, 1.0)?.rt(0.4),
        );
        // the green point lies over both halves of the track and the cloud, so
        // the find is the one thing in the picture that reads as green
        stage.scene_mut().bring_to_front(&[saved])?;
        line.wait_for(stage, "He was alive")?;
        let in_water =
            stay_words(&fact("F-C.aldridge.in_water")?).ok_or_else(|| fail("the stay does not parse".into()))?;
        let seen_clock = twelve_hour(night.clock_of(night.seen));
        let seen = kit.text_left(
            stage,
            &format!("{SEEN} {seen_clock}: {in_water}"),
            22.0,
            Palette::ink(),
            v(TRACK_X + 0.5, y0 - 0.55),
        )?;
        fade_in_all(stage, &[seen], 0.5)?;
        // he held on to two buoys: not a free drifter, so chapter 10's chance
        // is not a number about him (R2: M10, D12); said as "clinging to two
        // lobster buoys" is
        let buoys_lines = balanced_lines(&buoys_words(&fact("F-D.h12.seeing")?), BUOYS_CHARS);
        let buoys = text_lines(
            &kit,
            stage,
            &buoys_lines,
            BUOYS_PT,
            Palette::ink(),
            buoys_centre_x(&buoys_lines),
            y0 - BUOYS_DROP,
            BUOYS_STEP,
        )?;
        line.wait_for(stage, "clinging to two lobster buoys")?;
        fade_in_all(stage, &[buoys], 0.5)?;
        line.wait_end(stage)?;
        // in the hold: the software did not find him (R2: S3)
        let froze = lower_third(&kit, stage, FROZE_LABEL, FROZE_TEXT)?;
        fade_in_all(stage, &[froze], 0.5)?;
        kit.hold(stage)?;
        kit.music_breath(stage, HOLD_S)?;
        // the cloud leaves through its dots' own opacity, the rest by fades
        let mut leaving: Vec<Box<dyn Animation>> = vec![stage.prepare(tween(shown2, 0.0, 0.8)?)?];
        let rest: Vec<Mob> = stage
            .scene()
            .mobjects()
            .iter()
            .copied()
            .filter(|&m| m != second.group && m != anchor2)
            .collect();
        for m in rest {
            stage.clear_updaters(m, true);
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            leaving.push(stage.prepare(a.rt(0.8))?);
        }
        stage.play_prepared(leaving)?;
        stage.clear_updaters(anchor2, true);
        stage.remove_from_scene(second.group);
        stage.remove_from_scene(anchor2);
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The cloud's radius after `h` hours, in miles: the starting radius plus
    /// the drift error, which grows with the distance drifted
    /// (`drift::radius`, for any `h`).
    fn radius_miles(h: f64) -> f64 {
        drift::START_RADIUS.f64() + drift::ERROR_SHARE.f64() * drift::DRIFT.f64() * h
    }

    /// The chapter speaks exactly the script's 11_ lines, in order.
    #[test]
    fn the_chapter_speaks_the_scripts_lines_in_order() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let ids: Vec<&str> = script
            .lines()
            .iter()
            .map(|(id, _)| id.as_str())
            .filter(|id| id.starts_with("11_"))
            .collect();
        assert_eq!(ids, LINES);
    }

    /// The times on the screen are the facts' 24-hour strings, worded: the
    /// guess's window and middle, the fall, the re-run, the sighting, the
    /// date and the stay; and the 12-hour clock's edges (midnight, noon).
    #[test]
    fn the_times_are_the_facts_worded() {
        let f = facts().unwrap();
        let get = |k: &str| f.get(k).unwrap().to_owned();
        assert_eq!(window_words(&get("F-C.aldridge.guess")).unwrap(), "9:30 to 11:30 p.m.");
        let mins = |k: &str| clock_minutes(&get(k)).unwrap();
        assert_eq!(twelve_hour(mins("F-C.aldridge.guess.middle")), "10:30 p.m.");
        assert_eq!(twelve_hour(mins("F-C.aldridge.fell")), "3:30 a.m.");
        assert_eq!(twelve_hour(mins("F-C.aldridge.rerun")), "8:30 a.m.");
        assert_eq!(twelve_hour(mins("F-C.aldridge.seen")), "2:58 p.m.");
        assert_eq!(date_words(&get("F-C.aldridge.date")).unwrap(), "24 July 2013");
        assert_eq!(
            stay_words(&get("F-C.aldridge.in_water")).unwrap(),
            "11 hours 28 minutes"
        );
        // the edges of the 12-hour clock
        assert_eq!(twelve_hour(0), "12:00 a.m.");
        assert_eq!(twelve_hour(5), "12:05 a.m.");
        assert_eq!(twelve_hour(12 * 60), "12:00 p.m.");
        assert_eq!(twelve_hour(12 * 60 + 30), "12:30 p.m.");
        assert_eq!(twelve_hour(23 * 60 + 59), "11:59 p.m.");
        assert_eq!(window_words("23:30-00:30").unwrap(), "11:30 p.m. to 12:30 a.m.");
        assert_eq!(window_words("11:30-00:30").unwrap(), "11:30 to 12:30 a.m.");
        assert_eq!(stay_words("1h01").unwrap(), "1 hour 1 minute");
        assert!(clock_minutes("24:00").is_none() && clock_minutes("9h30").is_none());
        assert!(date_words("2013-13-01").is_none());
    }

    /// The night is the facts': the track is at the fall's miles when he
    /// fell and `south` miles short of them at the guess's middle; the boat's
    /// steady speed makes both true; the hours between are the facts'.
    #[test]
    fn the_night_is_the_facts() {
        let f = facts().unwrap();
        let n = Night::from_facts(&f).unwrap();
        assert_eq!(n.start, 21 * 60, "the hour line opens on the hour the window does");
        assert_eq!(n.guess, (30.0, 150.0));
        assert_eq!(n.middle, 90.0);
        assert_eq!(n.fell, 390.0);
        assert_eq!(n.bottom(), 420.0);
        assert_eq!(f.get("F-C.aldridge.hours_off"), Some("5"));
        assert_eq!(n.hours_between(n.middle, n.fell), 5.0);
        assert_eq!(f.get("F-C.aldridge.rerun.hours_after_fall"), Some("5"));
        assert_eq!(n.hours_between(n.fell, n.rerun), 5.0);
        assert_eq!(n.offshore, 40.0);
        assert_eq!(n.south, 30.0);
        assert!((n.speed() - 6.0).abs() < 1e-12);
        assert!((n.miles_at(n.fell) - n.offshore).abs() < 1e-12);
        assert!((n.miles_at(n.middle) - (n.offshore - n.south)).abs() < 1e-12);
        // the first map's point is almost thirty miles north of the truth, on
        // the screen as in the facts
        let gap = (track_point(n.miles_at(n.middle))[1] - track_point(n.offshore)[1]).abs() / MILE;
        assert!((gap - n.south).abs() < 1e-9, "{gap}");
        // the track never starts behind the coast, and the boat leaves it
        // by the hour line's top
        assert!(n.miles_at(0.0) >= 0.0 && n.miles_at(0.0) < 3.0);
        // the sighting: eleven hours and twenty-eight minutes after the fall
        assert_eq!(n.seen - n.fell, 11.0 * 60.0 + 28.0);
        assert_eq!(f.get("F-C.aldridge.in_water"), Some("11h28"));
    }

    /// The hour line is laid true: its ends are the night's first and last
    /// minutes, every mark the chapter puts on it is inside, and the hours
    /// are ticked in order.
    #[test]
    fn the_hour_line_is_laid_true() {
        let n = Night::from_facts(&facts().unwrap()).unwrap();
        assert_eq!(n.line_y(0.0), LINE_TOP);
        assert!((n.line_y(n.bottom()) - LINE_BOTTOM).abs() < 1e-12);
        for t in [n.guess.0, n.guess.1, n.middle, n.fell] {
            let y = n.line_y(t);
            assert!(y < LINE_TOP && y > LINE_BOTTOM, "{t}: {y}");
        }
        assert!(n.line_y(n.guess.0) > n.line_y(n.guess.1), "time runs down the line");
        let hours: Vec<String> = (0..=(n.bottom() / 60.0) as u32)
            .map(|h| hour_word(n.clock_of(f64::from(h) * 60.0)))
            .collect();
        assert_eq!(
            hours,
            [
                "9 p.m.", "10 p.m.", "11 p.m.", "12 a.m.", "1 a.m.", "2 a.m.", "3 a.m.", "4 a.m."
            ]
        );
    }

    /// 11_1: of the dots inside the square a search of the facts' best side
    /// lays over the cloud, exactly the share it would see go out; none
    /// outside the square does; a search that sees nothing puts out nothing
    /// and one that sees everything puts out every dot inside and no other.
    #[test]
    fn a_search_puts_out_the_share_it_would_have_seen_and_only_inside() {
        let f = facts().unwrap();
        let cloud = drift::cloud();
        let hours = drift::hours()[1].f64();
        assert_eq!(hours, 3.0);
        let side: f64 = f.get("F-D.flight.h3.best_side").unwrap().parse().unwrap();
        let flight = drift::flight(&cloud, hours, side as u32);
        assert_eq!(flight.inside.to_string(), f.get("F-D.flight.h3.inside").unwrap());
        assert_eq!(
            format!("{:.0}", 100.0 * flight.seeing),
            f.get("F-D.flight.h3.seeing").unwrap()
        );
        assert_eq!(
            format!("{:.0}", 100.0 * flight.find),
            f.get("F-D.flight.h3.find").unwrap()
        );
        let gone = put_out(&cloud, hours, side, flight.seeing, drift::SEED + 11);
        let n = gone.iter().filter(|&&g| g).count();
        assert_eq!(n, (flight.inside as f64 * flight.seeing).round() as usize);
        let half = side / 2.0;
        for (d, &out) in cloud.iter().zip(&gone) {
            let (x, y) = drift::offset(d, hours);
            if out {
                assert!(x.abs() <= half && y.abs() <= half, "a dot outside the square went out");
            }
        }
        // the edges
        assert_eq!(put_out(&cloud, hours, side, 0.0, 1).iter().filter(|&&g| g).count(), 0);
        let all = put_out(&cloud, hours, side, 1.0, 1);
        assert_eq!(all.iter().filter(|&&g| g).count(), flight.inside);
        // the same draw, the same dots
        assert_eq!(gone, put_out(&cloud, hours, side, flight.seeing, drift::SEED + 11));
    }

    /// 11_1's lower-third: CASP's year and SAROPS's, and "thousands of
    /// particles" is true of every size the facts give.
    #[test]
    fn the_planners_words_are_the_facts() {
        let f = facts().unwrap();
        assert_eq!(f.get("F-C.casp"), Some("1974"));
        assert_eq!(f.get("F-C.sarops"), Some("2007"));
        let sizes: Vec<u32> = f
            .get("F-C.sarops.dots")
            .unwrap()
            .split(',')
            .map(|s| s.parse().unwrap())
            .collect();
        assert!(!sizes.is_empty() && sizes.iter().all(|&n| n >= 1000), "{sizes:?}");
    }

    /// 11_3 and 11_6: the Aldridge cloud is the model's. Every dot stays in
    /// its radius about the point the map is run from, the radius is the
    /// model's at integer hours, it shrinks as the fall moves later (less
    /// time to drift), and when the map is run from the true time its
    /// centre is where he went in.
    #[test]
    fn the_aldridge_cloud_is_the_models_and_settles_on_where_he_went_in() {
        let n = Night::from_facts(&facts().unwrap()).unwrap();
        let cloud = drift::cloud();
        for h in drift::hours() {
            assert!((radius_miles(h.f64()) - drift::radius(h).f64()).abs() < 1e-12);
        }
        let mut last = f64::INFINITY;
        let mut t = n.middle;
        while t <= n.fell {
            let h = n.hours_between(t, n.rerun);
            let r = radius_miles(h);
            assert!(r < last, "the radius only shrinks as the fall moves later");
            last = r;
            let c = track_point(n.miles_at(t));
            for d in &cloud {
                let p = aldridge_place(d, &n, t);
                let miles = (p[0] - c[0]).hypot(p[1] - c[1]) / MILE;
                assert!(miles <= r + 1e-9, "t {t}: {miles} outside {r}");
            }
            t += 30.0;
        }
        // run from the true time, the cloud's middle is where he went in
        let c = track_point(n.offshore);
        let mean = cloud
            .iter()
            .map(|d| aldridge_place(d, &n, n.fell))
            .fold((0.0, 0.0), |a, p| (a.0 + p[0], a.1 + p[1]));
        let (mx, my) = (mean.0 / cloud.len() as f64, mean.1 / cloud.len() as f64);
        assert!(
            ((mx - c[0]).hypot(my - c[1])) / MILE < 1.0,
            "the cloud's middle is within a mile of him"
        );
        // the first map's middle is almost thirty miles away
        let first = track_point(n.miles_at(n.middle));
        assert!(((first[1] - c[1]).abs() / MILE - n.south).abs() < 1e-9);
    }

    /// The scene's layout holds: the track and every label sit on the stage.
    #[test]
    fn the_layout_sits_on_the_stage() {
        let n = Night::from_facts(&facts().unwrap()).unwrap();
        assert!(track_point(TRACK_MILES)[1] > -FRAME_Y_RADIUS + 1.0);
        assert!(
            TRACK_MILES > n.offshore + 5.0,
            "the boat is found well past where he fell"
        );
        assert!(HATCH_MILES > n.offshore && PUMPS_MILES > HATCH_MILES && PUMPS_MILES < TRACK_MILES);
        const {
            assert!(COAST_Y + 0.38 < FRAME_Y_RADIUS - 0.3);
            assert!(LINE_X - 1.0 > -FRAME_X_RADIUS);
            assert!(TRACK_X + HELI_HALF + 0.5 < FRAME_X_RADIUS);
        }
        // the whole of the first cloud (3 h) stays inside the stage
        let r = radius_miles(drift::hours()[1].f64()) * CLOUD_MILE;
        assert!(CLOUD_AT[1] + r < FRAME_Y_RADIUS - 0.3 && CLOUD_AT[1] - r > -FRAME_Y_RADIUS + 1.0);
        // the model tag stands a clear half unit above the carried question's
        // first line, and below the track's far end
        const {
            assert!(MODEL_Y - super::super::draw::QUIZ_LINES_Y[0] >= 0.5);
        }
        assert!(MODEL_Y < track_point(TRACK_MILES)[1] - 0.3);
    }

    /// 11_1's lower-third comes in with the year it names, not after the
    /// line: the phrases the chapter waits on are in the line as scripted.
    #[test]
    fn the_planner_is_named_when_its_year_is_said() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let text = script.text("11_1").unwrap();
        let (since, updates) = (text.find("since nineteen seventy-four"), text.find("and updates them"));
        assert!(since.is_some() && updates.is_some(), "{text}");
        assert!(since < updates, "the year is said before the search is drawn");
        // the note about drift comes in as the dots are named
        assert!(text.contains("It moves thousands"));
    }

    /// R2 (D14): 11_1's note under the cloud says the proof is for a wreck
    /// that stays put, in at most two lines, inside the frame, under the
    /// cloud's lowest dot and over the planner's lower-third.
    #[test]
    fn the_drift_note_stands_under_the_cloud_and_over_the_lower_third() {
        let lines = balanced_lines(DRIFT_NOTE, NOTE_CHARS);
        assert!(lines.len() <= 2, "{lines:?}");
        assert_eq!(lines.join(" "), DRIFT_NOTE);
        for part in [
            "proof of chapter 08",
            "a wreck that stays put",
            "a target that drifts",
            "reweight the dots",
            "no longer proved best",
        ] {
            assert!(DRIFT_NOTE.contains(part), "{part}");
        }
        let half = lines_width(&lines, NOTE_PT) / 2.0;
        assert!(CLOUD_AT[0] - half > -FRAME_X_RADIUS + 0.15 && CLOUD_AT[0] + half < FRAME_X_RADIUS - 0.15);
        // a line of 20 points is about 0.26 high
        let cloud_bottom = CLOUD_AT[1] - radius_miles(drift::hours()[1].f64()) * CLOUD_MILE;
        assert!(NOTE_Y + 0.13 < cloud_bottom - 0.1, "{cloud_bottom}");
        // the planner's lower-third is one line: the band's floor 0.2 over
        // the frame's, a back 0.3 and a line of 0.36 high
        let third_top = -FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36;
        let last = NOTE_Y - NOTE_STEP * (lines.len() as f64 - 1.0);
        assert!(last - 0.13 > third_top + 0.1, "{last}");
    }

    /// R2 (M19): 11_3's guess is tagged "the evening before" over its label,
    /// inside the hour line's column and clear of the label; the earlier
    /// point of fall stands on the track north of the first map's point,
    /// with its tag between the coast and the span's bracket's top, inside
    /// the frame.
    #[test]
    fn the_evening_and_the_earlier_point_are_tagged_where_they_read() {
        let f = facts().unwrap();
        let n = Night::from_facts(&f).unwrap();
        // the window's label is at the middle's height, the tag over it
        let tag_bottom = n.line_y(n.middle) + EVENING_RISE - 0.21;
        assert!(tag_bottom > n.line_y(n.middle) + 0.13 + 0.05, "{tag_bottom}");
        let left = LINE_X + 0.45;
        let right = left + tag_width(EVENING_BEFORE, TAG_PT);
        assert!(left > LINE_X + 0.36, "it stands right of the window's band");
        assert!(right < TRACK_X - 0.5, "it ends before the track");
        // above the window label's top and under the hour line's top
        assert!(n.line_y(n.middle) + EVENING_RISE + 0.21 < LINE_TOP + 0.5);
        // the earlier point: the guess's early end, north of the middle's
        let early = track_point(n.miles_at(n.guess.0));
        let middle = track_point(n.miles_at(n.middle));
        assert!(early[1] > middle[1], "the earlier he fell, the farther north");
        assert!(n.miles_at(n.guess.0) < n.miles_at(n.middle));
        assert!(n.miles_at(n.guess.0) >= 0.0);
        // its tag (20 points: a back about 0.47 high): right of the ring,
        // between the coast line and the first map's point, inside the frame
        let at = earlier_tag_at(&n);
        let w = tag_width(EARLIER_NORTH, TAG_PT);
        assert!(at[0] - w / 2.0 > early[0] + EARLIER_RADIUS, "the tag meets its ring");
        assert!(at[0] + w / 2.0 < FRAME_X_RADIUS - 0.15, "{}", at[0] + w / 2.0);
        assert!(at[1] + 0.235 < COAST_Y - 0.05, "the tag meets the coast");
        // the first map's cloud (as it stands at the re-run) is round its
        // point at the middle: the tag starts right of it
        let cloud_r = radius_miles(n.hours_between(n.middle, n.rerun)) * MILE;
        assert!(
            at[0] - w / 2.0 > middle[0] + cloud_r + 0.05,
            "the tag meets the first map's cloud"
        );
        assert_eq!(EARLIER_NORTH, "the earlier he fell, the farther north");
        assert_eq!(EVENING_BEFORE, "the evening before");
    }

    /// R2 (M20): 11_5 writes the fact's "5 hours" over the gap from the
    /// guess's middle to the fall, right of the clock labels and inside the
    /// hour line's column, and the number is the model's hours between.
    #[test]
    fn the_gap_is_five_hours_and_stands_right_of_the_clock_labels() {
        let f = facts().unwrap();
        let n = Night::from_facts(&f).unwrap();
        let hours = f.get("F-C.aldridge.hours_off").unwrap();
        assert_eq!(hours_words(hours), "5 hours");
        assert_eq!(hours_words("1"), "1 hour");
        assert_eq!(n.hours_between(n.middle, n.fell).to_string(), *hours);
        // the clock label of the guess's middle ("10:30 p.m.", 22 points)
        // ends before the bracket's serifs; the bracket starts under the
        // window's label and ends at the fall's height
        let first_time_right = LINE_X + 0.6 + "10:30 p.m.".chars().count() as f64 * 22.0 * CHAR_UNITS_PER_POINT;
        assert!(GAP_X - GAP_SERIF > first_time_right + 0.1, "{first_time_right}");
        // the window's label (24 points, centred on the middle's height) is
        // above the bracket's top tick
        let top = n.line_y(n.middle) - GAP_UNDER;
        assert!(top < n.line_y(n.middle) - 0.13 - 0.05, "under the window's label");
        assert!(n.line_y(n.fell) > LINE_BOTTOM);
        // the words stand right of the bracket and before the track's labels
        let width = hours_words(hours).chars().count() as f64 * GAP_PT * CHAR_UNITS_PER_POINT;
        assert!(
            GAP_X + 0.25 + width < TRACK_X - 1.0,
            "the words reach the track's labels"
        );
    }

    /// R2 (M10, D12, S3): 11_6's note says he held on to two lobster buoys
    /// and that chapter 10's number (the fact's 13) is not about him, in two
    /// lines to the left of the helicopter's track, clear of the clock
    /// labels' column; the lower-third that ends the chapter is one line and
    /// says the software froze.
    #[test]
    fn the_buoys_and_the_frozen_software_are_said_where_they_read() {
        let f = facts().unwrap();
        let percent = f.get("F-D.h12.seeing").unwrap();
        assert_eq!(percent, "13");
        let words = buoys_words(percent);
        assert_eq!(
            words,
            "held on to two lobster buoys: not a free drifter, so chapter 10's 13 in 100 is not a number about him"
        );
        let lines = balanced_lines(&words, BUOYS_CHARS);
        assert!(lines.len() <= 3, "{lines:?}");
        assert_eq!(lines.join(" "), words);
        // left of the helicopter's track and inside the frame
        let half = lines_width(&lines, BUOYS_PT) / 2.0;
        let centre = buoys_centre_x(&lines);
        assert!(centre + half <= TRACK_X - HELI_HALF - BUOYS_GAP + 1e-9);
        // and clear of 11_5's bracket, whose line runs down the same rows
        assert!(
            centre - half > GAP_X + 0.3,
            "the note meets the bracket: {}",
            centre - half
        );
        // under the green point and its track and under 11_5's "5 hours"
        // (which it may stand below in the same columns), over the model tag
        // and the planner's band; 18 points are about 0.23 high
        let n = Night::from_facts(&f).unwrap();
        let y0 = track_point(n.offshore)[1];
        let top = y0 - BUOYS_DROP + 0.115;
        let bottom = y0 - BUOYS_DROP - BUOYS_STEP * (lines.len() as f64 - 1.0) - 0.115;
        assert!(top < y0 - 0.55 - 0.15, "the buoys' note meets the sighting's words");
        let gap_mid = (n.line_y(n.middle) - GAP_UNDER + n.line_y(n.fell)) / 2.0;
        assert!(top < gap_mid - 0.13 - 0.1, "the buoys' note meets the gap's words");
        assert!(bottom > MODEL_Y + 0.2, "{bottom}");
        let third_top = -FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36;
        assert!(bottom > third_top + 0.1, "{bottom}");
        // the lower-third: one line, the words as the sources have them
        assert_eq!(FROZE_LABEL, "The software froze:");
        assert!(FROZE_TEXT.contains("drawn by hand") && FROZE_TEXT.contains("high-chance area"));
        assert!(FROZE_TEXT.chars().count() <= 92);
    }
}
