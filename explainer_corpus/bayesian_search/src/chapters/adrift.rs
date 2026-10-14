//! Chapter 10, adrift: a wreck stays where it is and a person in the water
//! does not (10_1); every dot moves with its own guess at the wind and the
//! current, so the map drifts and spreads (10_2, a question to carry); a
//! person is hard to see, a tenth of a mile of strip from a helicopter
//! (10_3); and twice the radius is four times the area, so the same flight
//! finds the person ninety-three times in a hundred an hour in and thirteen
//! in a hundred twelve hours in (10_4).
//!
//! The cloud is `model::drift`: 2,000 seeded dots, each keeping one error
//! for the whole drift. Every number shown is a fact (`F-D.*`) or a
//! `model::drift` value; each is marked "model" on the screen.

use explainer_kit::animatic::HOLD_S;
use explainer_kit::props::{Pose, Swarm, lower_third};
use explainer_kit::{Frame2, Kit, Palette, Timed, fade_all, play, tween, v};
use fmn::prelude::*;

use super::draw::{
    CHAR_UNITS_PER_POINT, QUIZ_LINES_Y, Spoken, carry_pause, fade_in_all, fade_out_all, faded_line, fail, flat,
    line_style, map as lit_blue, say, search as teal, tag, text_lines, tween_linear,
};
use crate::facts::facts;
use crate::model::{Q, drift, koopman};

pub struct Adrift {
    pub kit: Kit,
}

/// The narration lines this chapter speaks, in order.
pub const LINES: [&str; 4] = ["10_1", "10_2", "10_3", "10_4"];

// the chapter's fixed words (numbers are built from facts)
const LAST_KNOWN: &str = "last\nknown\nposition";
/// How far left of the diamond the words end: past the cloud's first circle
/// (a mile, `START_RADIUS`, drawn at [`UNITS_PER_MILE`]) and its edge's
/// stroke, which no later circle comes back across.
const LAST_KNOWN_GAP: f64 = 0.6;
const WIND: &str = "wind";
const CURRENT: &str = "current";
const LEEWAY: &str = "leeway:";
const LEEWAY_TEXT: &str = "the push of the wind on what floats";
const MODEL: &str = "model";
const SWEEP_LABEL: &str = "sweep width:";
/// 10_3's lower-third (R2: S2, R9, D13): sweep width is an effective width,
/// and the strip is not a hard edge.
const EFFECTIVE_WIDTH: &str =
    "an effective width, not a hard edge. It sees some outside the strip and misses some inside";
/// 10_3's label of the strip, from Table H-15: the numbers are the facts'.
const SWEEP_TABLE_HEAD: &str = "sweep width";
const SWEEP_TABLE_TAIL: &str = "person in the water, helicopter at";
const SWEEP_SOURCE: &str = "US Coast Guard, Table H-15";
/// 10_3's flight beside the track (R2: M5, D6), so that "the same flight" of
/// 10_4 has an antecedent.
const ONE_FLIGHT: &str = "one flight:";
const TRACK_WORDS: &str = "sea miles of track";
const HOUR: &str = "h";
const HOUR_WORD: &str = "hour";
const COVERAGE_LABEL: &str = "coverage = strip × track ÷ area";
/// 10_4: what the quartering holds for (R2: R1), beside the "× ¼".
const ANY_DOUBLING: &str = "for any doubling";
const X_LABEL: &str = "coverage";
const Y_LABEL: &str = "chance one flight sees the person";
/// 10_4's model label (R2: S11, D6, S4): the flight, how it is spread, and
/// what the cloud's edge is.
const SAME_TRACK_HEAD: &str = "the same";
const SAME_TRACK_TAIL: &str = "sea miles of track spread evenly over the circle as random strips (a cautious floor);";
const MODEL_FLIGHT: &str = "one flight taken as instant; the cloud a uniform disc whose edge holds every dot:";
const EDGE_TAIL: &str = "of the distance drifted";
/// 10_4's hold (R2: S10).
const NOT_MODELLED: &str = "not in this model: how long a person survives; that clock runs too";
const SQ_SEA_MILES: &str = "sq sea miles";

// ------------------------------------------------------------------ geometry

/// Scene units a sea mile is drawn as.
const UNITS_PER_MILE: f64 = 0.40;
/// Where the person was last known.
const START: Vec3 = [-5.4, 0.6, 0.0];
/// The hour line along the bottom: its ends, and its height (its labels sit
/// above the question's line in the caption band, a clear 0.13 over it: at
/// -2.35 the "6 h" touched the carried question).
const LINE_X: (f64, f64) = (-6.2, 6.2);
const LINE_Y: f64 = -2.2;
/// Hours at the line's right end: the last the chapter shows.
const HOURS_MAX: f64 = 12.0;
/// How long the drift still runs after the question is asked, seconds.
const DRIFT_AFTER_ASK_S: f64 = 2.4;
/// How opaque a dot of the cloud is (the cloud is thick near its middle).
const CLOUD_OPACITY: f64 = 0.9;

/// 10_4's y label: inside the plot, ending this far from its right edge and
/// this far over its bottom; an hour's words keep this far from the point
/// and stay this far inside the plot's edge.
const Y_LABEL_INSET: f64 = 0.2;
const Y_LABEL_RISE: f64 = 0.4;
const HOUR_WORDS_GAP: f64 = 0.2;
const HOUR_WORDS_EDGE: f64 = 0.15;
/// An hour's words stand this far under their point (right of it), or this
/// far under it when they stand left of it (the curve is nearly flat there).
const HOUR_WORDS_DROP: f64 = 0.32;
const HOUR_WORDS_DROP_FAR: f64 = 0.5;
/// The largest small cloud's radius over the plot (twelve hours): small
/// enough that the twelfth hour's and the sixth's, at their coverages, do
/// not meet.
const SMALL_CLOUD_R: f64 = 0.42;

/// 10_3's labels (R2: M5, S2): the table's words stand over the held cloud,
/// the flight's length by the strip's left end (a little above it, clear of
/// the last known position's words); type sizes in points.
const SWEEP_TAG_Y: f64 = 3.5;
const SWEEP_TAG_PT: f64 = 20.0;
const FLIGHT_TAG_PT: f64 = 20.0;
const FLIGHT_TAG_RISE: f64 = 0.32;
const FLIGHT_TAG_GAP: f64 = 0.12;
/// How much wider a tag's back is than its words, scene units.
const TAG_PAD: f64 = 0.3;
/// 10_4's list of the circles' radius and area (R2: M5, D6, R1): the left
/// end of its rows, the top row, the rows' spacing, and the type size. It
/// stands left of the plot, between the labels of 100% and 50%.
const CIRCLES_X: f64 = -6.8;
const CIRCLES_TOP: f64 = 1.55;
const CIRCLES_STEP: f64 = 0.36;
const CIRCLES_PT: f64 = 17.0;
/// 10_4's model label: the lines' type size and spacing under the plot's
/// axis label, and the line of the hold under them.
const MODEL_PT: f64 = 18.0;
const MODEL_STEP: f64 = 0.25;
/// The first model line stands this far under the plot's floor.
const MODEL_DROP: f64 = 0.9;
const SURVIVAL_Y: f64 = -3.62;
/// "For any doubling" starts here, right of the "× ¼" (whose middle is at
/// 3.6 and which is about a unit wide).
const DOUBLING_X: f64 = 4.25;

/// Where the hour line puts hour `h`.
fn hour_x(h: f64) -> f64 {
    LINE_X.0 + (LINE_X.1 - LINE_X.0) * h / HOURS_MAX
}

/// The centre of the cloud after `h` hours: the mean drift carries it east.
fn centre(h: f64) -> Vec3 {
    v(START[0] + drift::DRIFT.f64() * h * UNITS_PER_MILE, START[1])
}

/// The cloud's radius after `h` hours, in miles: the starting radius plus
/// the drift error, which grows with the distance drifted (the model's
/// `drift::radius`, for any `h`).
fn radius_miles(h: f64) -> f64 {
    drift::START_RADIUS.f64() + drift::ERROR_SHARE.f64() * drift::DRIFT.f64() * h
}

/// Where one dot is after `h` hours.
fn place(d: &drift::Dot, h: f64) -> Vec3 {
    let (ox, oy) = drift::offset(d, h);
    let c = centre(h);
    v(c[0] + ox * UNITS_PER_MILE, c[1] + oy * UNITS_PER_MILE)
}

/// The zigzag of `passes` straight passes across a square of `side` centred
/// at `at`: the same pass length whatever the square, so the total length
/// of track is `passes × side`.
fn zigzag(at: Vec3, side: f64, passes: usize) -> Vec<Vec3> {
    let mut out = Vec::with_capacity(2 * passes);
    for k in 0..passes {
        let y = at[1] + side / 2.0 - side * (k as f64 + 0.5) / passes as f64;
        let (a, b) = (at[0] - side / 2.0, at[0] + side / 2.0);
        if k % 2 == 0 {
            out.extend([v(a, y), v(b, y)]);
        } else {
            out.extend([v(b, y), v(a, y)]);
        }
    }
    out
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

/// The curve's window of 10_4: coverage across, the chance of seeing up.
fn curve_window() -> Frame2 {
    Frame2 {
        center: v(0.9, 0.1),
        width: 8.4,
        height: 4.0,
        x: (0.0, 3.0),
        y: (0.0, 100.0),
    }
}

/// The four hours the chapter compares, with the coverage one flight of
/// the model's track has over the cloud then, and the chance it sees.
struct Hour {
    h: Q,
    coverage: f64,
    seeing: f64,
}

fn hours() -> Vec<Hour> {
    drift::hours()
        .into_iter()
        .map(|h| Hour {
            h,
            coverage: drift::coverage(h),
            seeing: drift::seeing(h),
        })
        .collect()
}

/// A number of hours as the screen words it ("1 h").
fn hours_label(h: Q) -> String {
    format!("{} {HOUR}", h.fixed(0))
}

/// The last sentence of `line` as a question to carry: set in ink in the
/// caption band like the quiz's question, no ring. Detached.
fn carry_last(kit: &Kit, stage: &mut Stage<'_>, line: &Spoken) -> fmn::Result<Mob> {
    let words = line
        .sentence_texts()
        .pop()
        .ok_or_else(|| fail(format!("{} has no sentences", line.id)))?;
    kit.text(stage, &words, 26.0, Palette::ink(), v(0.0, QUIZ_LINES_Y[0]))
}

/// A closed outline of an open diamond (a last known position) at `at`.
fn diamond(at: Vec3, r: f64) -> Vec<Vec3> {
    vec![
        v(at[0], at[1] + r),
        v(at[0] + r, at[1]),
        v(at[0], at[1] - r),
        v(at[0] - r, at[1]),
        v(at[0], at[1] + r),
    ]
}

/// The two edges of a fan of uncertainty `spread` radians either side of
/// the heading `angle`, `len` long, from `from`.
fn fan(from: Vec3, angle: f64, spread: f64, len: f64) -> Vec<Vec3> {
    let tip = |a: f64| v(from[0] + len * a.cos(), from[1] + len * a.sin());
    vec![from, tip(angle - spread), tip(angle + spread)]
}

/// How wide a tag of `words` at `pt` points runs, its back included.
fn tag_width(words: &str, pt: f64) -> f64 {
    words.chars().count() as f64 * pt * CHAR_UNITS_PER_POINT + TAG_PAD
}

/// 10_3's label of the strip, as Table H-15 gives it, from the facts' strip
/// (in sea miles, feet and metres) and altitude (in feet).
fn sweep_table_words(strip: &str, feet: &str, metres: &str, altitude: &str) -> String {
    format!(
        "{SWEEP_TABLE_HEAD} {strip} sea mile, {feet} ft, {metres} m: {SWEEP_TABLE_TAIL} {altitude} feet. {SWEEP_SOURCE}"
    )
}

/// 10_3's "one flight: 180 sea miles of track", from the facts' track.
fn flight_words(track: &str) -> String {
    format!("{ONE_FLIGHT} {track} {TRACK_WORDS}")
}

/// Where the flight's tag stands: its right end a little left of the strip's
/// left end and a little above the track.
fn flight_tag_at(words: &str) -> Vec3 {
    let c = centre(HOURS_MAX);
    let strip_left = c[0] - radius_miles(HOURS_MAX) * UNITS_PER_MILE;
    v(
        strip_left - FLIGHT_TAG_GAP - tag_width(words, FLIGHT_TAG_PT) / 2.0,
        c[1] + FLIGHT_TAG_RISE,
    )
}

/// 10_4's rows of the circles' sizes, one per hour in the order the markers
/// land: the first with its words ("1 hour: radius 1.45, area 6.6 sq sea
/// miles"), the rest bare ("3: 2.35, 17.3"). Each row is (hours, radius, area).
fn circle_rows(rows: &[(String, String, String)]) -> Vec<String> {
    rows.iter()
        .enumerate()
        .map(|(i, (h, r, a))| {
            if i == 0 {
                format!("{h} {HOUR_WORD}: radius {r}, area {a} {SQ_SEA_MILES}")
            } else {
                format!("{h}: {r}, {a}")
            }
        })
        .collect()
}

/// 10_4's model label in three lines: the flight and how it is spread, the
/// flight as an instant and the cloud a disc, and the disc's edge (the
/// model's starting radius plus its share of the distance drifted).
fn model_lines(track: &str, start: &str, share: &str) -> [String; 3] {
    [
        format!("{MODEL}: {SAME_TRACK_HEAD} {track} {SAME_TRACK_TAIL}"),
        MODEL_FLIGHT.to_owned(),
        format!("{start} mile plus {share} {EDGE_TAIL}"),
    ]
}

impl SceneConstruct for Adrift {
    fn name(&self) -> &str {
        "10_adrift"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let all = facts().map_err(fail)?;
        let fact = |key: &str| -> fmn::Result<String> {
            all.get(key)
                .map(str::to_owned)
                .ok_or_else(|| fail(format!("no fact {key}")))
        };

        // ------------------------------------------------------- 10_1
        // open water, a last known position, the wind and the current, and
        // how little anyone knows of either
        let line = say(&kit, stage, LINES[0])?;
        let last_known = stage.arena_mut().add(
            polyline(diamond(START, 0.16))
                .style(line_style(Palette::dim(), 3.0))
                .build(),
        );
        // the words stand LEFT of the diamond, in three short lines: the
        // cloud begins on the diamond and sweeps the water to its right and
        // below for seven of the twelve hours, and the words under the
        // diamond were under its dots (and its edge) for most of 10_2
        let last_label = kit.text(stage, LAST_KNOWN, 24.0, Palette::dim(), START)?;
        stage.move_to(last_label, v(START[0] - LAST_KNOWN_GAP, START[1]), RIGHT);
        fade_in_all(stage, &[last_known, last_label], 0.6)?;
        line.wait_for(stage, "Wind and current")?;
        let mut arrows: Vec<Mob> = Vec::new();
        for (name, angle, y) in [(WIND, 0.12, 0.85), (CURRENT, -0.20, -0.85)] {
            let from = v(START[0] + 0.3, START[1] + y);
            let tip = v(from[0] + 2.0 * f64::cos(angle), from[1] + 2.0 * f64::sin(angle));
            let wedge = stage.arena_mut().add(
                Polygon::new(fan(from, angle, 0.22, 2.2))
                    .style(flat(Palette::dim(), 0.18))
                    .build(),
            );
            let arrow = Arrow::new(from, tip)
                .buff(0.0)
                .thickness(4.0)
                .color(Palette::dim())
                .build()?;
            let arrow = stage.arena_mut().add(arrow);
            let word = kit.text(stage, name, 24.0, Palette::dim(), v(tip[0] + 0.55, tip[1]))?;
            arrows.extend([wedge, arrow, word]);
        }
        fade_in_all(stage, &arrows, 0.7)?;
        let leeway = lower_third(&kit, stage, LEEWAY, LEEWAY_TEXT)?;
        line.wait_end(stage)?;
        fade_in_all(stage, &[leeway], 0.5)?;

        // ------------------------------------------------------- 10_2
        // every dot moves with its own guess: the map drifts and spreads
        let line = say(&kit, stage, LINES[1])?;
        fade_out_all(stage, &[leeway], 0.3)?;
        let cloud: Vec<drift::Dot> = drift::cloud();
        let poses: Vec<Pose> = cloud
            .iter()
            .map(|d| {
                let p = place(d, 0.0);
                Pose {
                    x: p[0],
                    y: p[1],
                    heading: 0.0,
                }
            })
            .collect();
        let template = Dot::new().radius(0.028).style(flat(lit_blue(), CLOUD_OPACITY)).build();
        let swarm = Swarm::build(stage, &template, &poses, None)?;
        // the cloud comes in through its dots' own opacity (a fade_in over a
        // group of two thousand would copy them all)
        for &m in &swarm.members {
            stage.set_opacity(m, 0.0, false);
        }
        let shown = stage.add_value_tracker(0.0);
        // the hour line, with the four hours the chapter compares marked
        let hour_line = stage.arena_mut().add(
            polyline(vec![v(LINE_X.0, LINE_Y), v(LINE_X.1, LINE_Y)])
                .style(line_style(Palette::dim(), 2.5))
                .build(),
        );
        let mut line_mobs: Vec<Mob> = vec![hour_line];
        for h in drift::hours() {
            let x = hour_x(h.f64());
            line_mobs.push(
                stage.arena_mut().add(
                    polyline(vec![v(x, LINE_Y - 0.1), v(x, LINE_Y + 0.1)])
                        .style(line_style(Palette::dim(), 2.5))
                        .build(),
                ),
            );
            line_mobs.push(kit.text(stage, &hours_label(h), 22.0, Palette::dim(), v(x, LINE_Y - 0.32))?);
        }
        let model_tag = kit.text_left(
            stage,
            &format!(
                "{MODEL}: a start known to {} mile, a drift of {} miles an hour, an error of {} of the distance drifted",
                fact("F-D.start_radius")?,
                fact("F-D.drift")?,
                fact("F-D.error_share")?
            ),
            20.0,
            Palette::dim(),
            v(-6.9, 3.55),
        )?;
        line_mobs.push(model_tag);
        let time = stage.add_value_tracker(0.0);
        // the circle that follows the cloud's edge, and the moving mark
        let edge = stage.always_redraw(move |s| {
            let h = s.tracker_value(time).unwrap_or(0.0);
            let c = Circle::new()
                .radius(radius_miles(h) * UNITS_PER_MILE)
                .arc_center(centre(h))
                .style(faded_line(teal(), 3.0, 0.9))
                .build();
            s.add(c)
        });
        let mark = stage.always_redraw(move |s| {
            let h = s.tracker_value(time).unwrap_or(0.0);
            let m = Dot::new()
                .point(v(hour_x(h), LINE_Y))
                .radius(0.1)
                .style(flat(Palette::ink(), 1.0))
                .build();
            s.add(m)
        });
        // each member keeps its own guess: placed from the one clock
        let members = swarm.members.clone();
        let mut at: Vec<Vec3> = poses.iter().map(|p| v(p.x, p.y)).collect();
        let walkers = cloud.clone();
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        let mut last = (f64::NAN, f64::NAN);
        stage.add_updater(
            anchor,
            move |s, _| {
                let now = (
                    s.tracker_value(time).unwrap_or(0.0),
                    s.tracker_value(shown).unwrap_or(0.0),
                );
                if now == last {
                    return;
                }
                if now.1 != last.1 {
                    for &m in &members {
                        s.set_opacity(m, now.1 * CLOUD_OPACITY, false);
                    }
                }
                if now.0 != last.0 {
                    for ((&m, d), cur) in members.iter().zip(&walkers).zip(at.iter_mut()) {
                        let p = place(d, now.0);
                        if p != *cur {
                            s.shift(m, [p[0] - cur[0], p[1] - cur[1], 0.0]);
                            *cur = p;
                        }
                    }
                }
                last = now;
            },
            true,
        )?;
        stage.add_to_scene(swarm.group)?;
        fade_out_all(stage, &arrows, 0.4)?;
        fade_in_all(stage, &line_mobs, 0.7)?;
        stage.play(tween(shown, 1.0, 0.7)?)?;
        stage.add_to_scene(edge)?;
        stage.add_to_scene(mark)?;
        line.wait_for(stage, "The map drifts")?;
        // the drift keeps one steady rate through the question: the cloud is
        // still spreading as it is asked, and has settled by its hold
        let lead = (line.sentence(2)? - stage.scene().time().to_f64()).max(0.5);
        let total = lead + DRIFT_AFTER_ASK_S;
        stage.play(tween_linear(time, HOURS_MAX * lead / total, lead)?)?;
        let question = carry_last(&kit, stage, &line)?;
        play!(stage;
            fade_in(stage.arena_mut(), question, ORIGIN, 1.0)?.rt(0.4),
            tween_linear(time, HOURS_MAX, DRIFT_AFTER_ASK_S)?,
        );
        carry_pause(&kit, stage)?;
        fade_out_all(stage, &[question], 0.3)?;

        // ------------------------------------------------------- 10_3
        // a person is hard to see: the strip across the held cloud, to scale
        let line = say(&kit, stage, LINES[2])?;
        // the hour line has done its work, and the lower-third needs its room
        let mut done = line_mobs.clone();
        done.push(mark);
        fade_out_all(stage, &done, 0.5)?;
        let c = centre(HOURS_MAX);
        let half = radius_miles(HOURS_MAX) * UNITS_PER_MILE;
        let strip_h = drift::STRIP.f64() * UNITS_PER_MILE;
        let strip = stage.arena_mut().add(
            Polygon::new(vec![
                v(c[0] - half, c[1] - strip_h / 2.0),
                v(c[0] + half, c[1] - strip_h / 2.0),
                v(c[0] + half, c[1] + strip_h / 2.0),
                v(c[0] - half, c[1] + strip_h / 2.0),
            ])
            .style(flat(teal(), 1.0))
            .build(),
        );
        line.wait_for(stage, "From a helicopter")?;
        fade_in_all(stage, &[strip], 0.8)?;
        // the table's own words over the cloud, and beside the track the
        // flight it is the track of: "the same flight" of 10_4 has its
        // antecedent here (R2: M5, D6)
        let table_words = sweep_table_words(
            &fact("F-D.strip")?,
            &fact("F-D.strip.feet")?,
            &fact("F-D.strip.metres")?,
            &fact("F-D.strip.altitude.feet")?,
        );
        let table = tag(&kit, stage, &table_words, v(0.0, SWEEP_TAG_Y), SWEEP_TAG_PT)?;
        let one_flight = flight_words(&fact("F-D.track")?);
        let flight = tag(&kit, stage, &one_flight, flight_tag_at(&one_flight), FLIGHT_TAG_PT)?;
        fade_in_all(stage, &[table, flight], 0.6)?;
        // in the hold: what sweep width is (R2: S2, R9, D13), said once, not
        // as a hard edge
        let sweep = lower_third(&kit, stage, SWEEP_LABEL, EFFECTIVE_WIDTH)?;
        line.wait_for(stage, "It is a tenth")?;
        fade_in_all(stage, &[sweep], 0.5)?;

        // ------------------------------------------------------- 10_4
        // twice the radius is four times the area; the same flight over a
        // cloud that has grown buys less every hour
        let line = say(&kit, stage, LINES[3])?;
        // the cloud, its circle and the hour line leave; two outline
        // circles, one twice as wide, each in its square, take the stage
        let leaving = vec![edge, strip, sweep, table, flight, last_known, last_label];
        // the cloud goes through its dots' own opacity, the rest by fades
        let mut anims: Vec<Box<dyn Animation>> = vec![stage.prepare(tween(shown, 0.0, 0.6)?)?];
        for &m in &leaving {
            stage.clear_updaters(m, true);
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.6))?);
        }
        stage.play_prepared(anims)?;
        stage.clear_updaters(anchor, true);
        stage.remove_from_scene(swarm.group);
        let (small_side, big_side) = (2.0, 4.0);
        let small_at = v(-3.6, 0.7);
        let big_at = v(1.2, 0.7);
        let mut pair: Vec<Mob> = Vec::new();
        for (at, side, passes) in [(small_at, small_side, 8), (big_at, big_side, 4)] {
            let square = stage.arena_mut().add(
                polyline(square_outline(at, side))
                    .style(line_style(Palette::faint(), 2.5))
                    .build(),
            );
            let circle = stage.arena_mut().add(
                Circle::new()
                    .radius(side / 2.0)
                    .arc_center(at)
                    .style(line_style(Palette::dim(), 3.0))
                    .build(),
            );
            let track = stage.arena_mut().add(
                polyline(zigzag(at, side, passes))
                    .style(line_style(teal(), 5.0))
                    .build(),
            );
            pair.extend([square, circle, track]);
        }
        fade_in_all(stage, &pair, 0.7)?;
        // four of the small squares tile the large one
        let mut tiling: Vec<Mob> = Vec::new();
        for (a, b) in [
            (
                v(big_at[0], big_at[1] - big_side / 2.0),
                v(big_at[0], big_at[1] + big_side / 2.0),
            ),
            (
                v(big_at[0] - big_side / 2.0, big_at[1]),
                v(big_at[0] + big_side / 2.0, big_at[1]),
            ),
        ] {
            tiling.push(
                stage
                    .arena_mut()
                    .add(polyline(vec![a, b]).style(faded_line(Palette::dim(), 2.0, 0.7)).build()),
            );
        }
        let factor = fact("F-D.double.coverage_factor")?;
        let (num, den) = factor
            .split_once('/')
            .ok_or_else(|| fail(format!("F-D.double.coverage_factor {factor} is not a fraction")))?;
        let equation = kit.text(stage, COVERAGE_LABEL, 28.0, Palette::ink(), v(-1.2, -1.95))?;
        let times = kit.tex(
            stage,
            &format!(r"\times \frac{{{num}}}{{{den}}}"),
            44.0,
            Palette::ink(),
            &[],
            v(3.6, -1.95),
        )?;
        let area_words = kit.text(
            stage,
            &format!("{} × the area", fact("F-D.double.area_factor")?),
            24.0,
            Palette::dim(),
            v(big_at[0], big_at[1] + big_side / 2.0 + 0.35),
        )?;
        // the quartering holds for every doubling, not for this pair alone
        // (R2: R1: the cloud's circle is not twice as wide at twelve hours as
        // at one)
        let doubling = kit.text_left(stage, ANY_DOUBLING, 24.0, Palette::ink(), v(DOUBLING_X, -1.95))?;
        line.wait_for(stage, "so each square mile")?;
        fade_in_all(stage, &tiling, 0.5)?;
        fade_in_all(stage, &[equation, times, area_words, doubling], 0.6)?;

        // chapter 09's curve, and four markers sliding down it as the hours pass
        line.wait_for(stage, "In this model")?;
        let mut away = pair.clone();
        away.extend(&tiling);
        away.extend([equation, times, area_words, doubling]);
        fade_out_all(stage, &away, 0.5)?;
        let w = curve_window();
        let frame = stage.arena_mut().add(w.border(Palette::faint())?);
        let mut furniture: Vec<Mob> = vec![frame];
        for c in 0..=3u32 {
            let p = w.p(f64::from(c), 0.0);
            furniture.push(kit.text(stage, &c.to_string(), 24.0, Palette::dim(), v(p[0], p[1] - 0.28))?);
        }
        for pct in [0u32, 50, 100] {
            let p = w.p(0.0, f64::from(pct));
            furniture.push(kit.text(stage, &format!("{pct}%"), 24.0, Palette::dim(), v(p[0] - 0.42, p[1]))?);
        }
        furniture.push(kit.text(stage, X_LABEL, 24.0, Palette::dim(), v(w.center[0], w.bottom() - 0.5))?);
        // the y label stands inside the plot, under the curve at the right:
        // above the plot the small clouds of the hours (up to 0.55 across)
        // stood on it
        let y_label = kit.text(stage, Y_LABEL, 24.0, Palette::dim(), ORIGIN)?;
        let half = stage.get_width(y_label) / 2.0;
        stage.move_to(
            y_label,
            v(w.right() - Y_LABEL_INSET - half, w.bottom() + Y_LABEL_RISE),
            ORIGIN,
        );
        furniture.push(y_label);
        let curve_pts: Vec<Vec3> = (0..=120)
            .map(|i| {
                let c = 3.0 * f64::from(i) / 120.0;
                w.p(c, 100.0 * koopman::random_search(c))
            })
            .collect();
        furniture.push(
            stage
                .arena_mut()
                .add(polyline(curve_pts).style(line_style(Palette::ink(), 3.0)).build()),
        );
        // what the model is, and what its cloud's edge is (R2: S11, D6, S4)
        let lines = model_lines(
            &fact("F-D.track")?,
            &fact("F-D.start_radius")?,
            &fact("F-D.error_share")?,
        );
        let same = text_lines(
            &kit,
            stage,
            &lines,
            MODEL_PT,
            Palette::dim(),
            w.center[0],
            w.bottom() - MODEL_DROP,
            MODEL_STEP,
        )?;
        furniture.push(same);
        fade_in_all(stage, &furniture, 0.6)?;
        // each hour's circle has its own radius and area (R2: M5, D6, R1):
        // a row for each, brought in with its marker
        let mut sizes: Vec<(String, String, String)> = Vec::new();
        for h in drift::hours() {
            let key = format!("F-D.h{h}");
            sizes.push((
                h.fixed(0),
                fact(&format!("{key}.radius"))?,
                fact(&format!("{key}.area"))?,
            ));
        }
        let mut size_rows: Vec<Mob> = Vec::new();
        for (i, row) in circle_rows(&sizes).iter().enumerate() {
            size_rows.push(kit.text_left(
                stage,
                row,
                CIRCLES_PT,
                Palette::ink(),
                v(CIRCLES_X, CIRCLES_TOP - CIRCLES_STEP * i as f64),
            )?);
        }
        // one marker rides the curve down as the hours pass, leaving each
        // hour's chance and a small cloud of that hour above it
        let spans = hours();
        let cov = stage.add_value_tracker(spans[0].coverage);
        let rider = stage.always_redraw(move |s| {
            let c = s.tracker_value(cov).unwrap_or(0.0);
            let d = Dot::new()
                .point(w.p(c, 100.0 * koopman::random_search(c)))
                .radius(0.13)
                .style(flat(Palette::ink(), 1.0))
                .build();
            s.add(d)
        });
        let mut stops: Vec<Mob> = Vec::new();
        let mark_hour = |stage: &mut Stage<'_>, hr: &Hour, row: Mob, stops: &mut Vec<Mob>| -> fmn::Result<()> {
            let at = w.p(hr.coverage, 100.0 * hr.seeing);
            let cloud_r = SMALL_CLOUD_R * radius_miles(hr.h.f64()) / radius_miles(HOURS_MAX);
            let small_cloud = stage.arena_mut().add(
                Circle::new()
                    .radius(cloud_r.max(0.06))
                    .arc_center(v(at[0], w.top() + 0.75))
                    .style(line_style(teal(), 3.0))
                    .build(),
            );
            // under the curve, right of its point; left of it, lower still,
            // when there is no room before the plot's edge (over the curve
            // the line struck through the words)
            let words = kit.text(
                stage,
                &format!("{} {:.0}%", hours_label(hr.h), 100.0 * hr.seeing),
                24.0,
                Palette::ink(),
                ORIGIN,
            )?;
            let half = stage.get_width(words) / 2.0;
            let (x, y) = if at[0] + HOUR_WORDS_GAP + 2.0 * half > w.right() - HOUR_WORDS_EDGE {
                (at[0] - HOUR_WORDS_GAP - half, at[1] - HOUR_WORDS_DROP_FAR)
            } else {
                (at[0] + HOUR_WORDS_GAP + half, at[1] - HOUR_WORDS_DROP)
            };
            stage.move_to(words, v(x, y), ORIGIN);
            fade_in_all(stage, &[small_cloud, words, row], 0.4)?;
            stops.extend([small_cloud, words, row]);
            Ok(())
        };
        line.wait_for(stage, "one hour in")?;
        stage.add_to_scene(rider)?;
        mark_hour(stage, &spans[0], size_rows[0], &mut stops)?;
        line.wait_for(stage, "Twelve hours in")?;
        for (hr, &row) in spans[1..].iter().zip(&size_rows[1..]) {
            stage.play(tween_linear(cov, hr.coverage, 0.9)?)?;
            mark_hour(stage, hr, row, &mut stops)?;
        }
        line.wait_end(stage)?;
        // in the hold: what this model leaves out (R2: S10)
        let survival = kit.text(stage, NOT_MODELLED, 22.0, Palette::ink(), v(w.center[0], SURVIVAL_Y))?;
        fade_in_all(stage, &[survival], 0.5)?;
        kit.hold(stage)?;
        kit.music_breath(stage, HOLD_S)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::q;

    /// The chapter speaks exactly the script's 10_ lines, in order.
    #[test]
    fn the_chapter_speaks_the_scripts_lines_in_order() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let ids: Vec<&str> = script
            .lines()
            .iter()
            .map(|(id, _)| id.as_str())
            .filter(|id| id.starts_with("10_"))
            .collect();
        assert_eq!(ids, LINES);
    }

    /// The radius drawn at any hour is the model's, and no dot is ever
    /// outside it: the circle follows the edge of the cloud.
    #[test]
    fn the_circle_is_the_models_radius_and_holds_every_dot() {
        let cloud = drift::cloud();
        assert_eq!(cloud.len(), drift::DOTS);
        for h in drift::hours() {
            assert!((radius_miles(h.f64()) - drift::radius(h).f64()).abs() < 1e-12);
        }
        for tenths in 0..=120 {
            let h = f64::from(tenths) / 10.0;
            assert!(drift::farthest(&cloud, h) <= radius_miles(h) + 1e-12, "hour {h}");
        }
    }

    /// The cloud drifts east at the model's rate and the dots' places are
    /// the model's offsets about it: the centre of mass of the cloud stays
    /// near the mean drift.
    #[test]
    fn the_cloud_drifts_at_the_models_rate_and_every_dot_keeps_its_error() {
        let cloud = drift::cloud();
        for h in [1.0, 3.0, 6.0, 12.0] {
            let c = centre(h);
            let travelled = (c[0] - START[0]) / UNITS_PER_MILE;
            assert!((travelled - 1.5 * h).abs() < 1e-9, "{h} h: {travelled} miles");
            let mean = cloud
                .iter()
                .map(|d| place(d, h))
                .fold((0.0, 0.0), |a, p| (a.0 + p[0], a.1 + p[1]));
            let n = cloud.len() as f64;
            assert!(
                (mean.0 / n - c[0]).abs() < 0.05 * radius_miles(h) * UNITS_PER_MILE + 0.02,
                "hour {h}"
            );
        }
        // one error for the whole drift: a dot's offset is linear in the
        // hours (so it keeps its direction)
        let d = cloud[7];
        let (a, b) = (drift::offset(&d, 2.0), drift::offset(&d, 4.0));
        let step = (b.0 - a.0, b.1 - a.1);
        let again = drift::offset(&d, 6.0);
        assert!(((again.0 - b.0) - step.0).abs() < 1e-12 && ((again.1 - b.1) - step.1).abs() < 1e-12);
    }

    /// 10_3 and 10_4: the strip drawn is a tenth of a mile at the drawn
    /// scale; at twice the radius the area is four times and the coverage a
    /// quarter; the four hours' chances are the facts' 93, 65, 34 and 13.
    #[test]
    fn the_strip_the_doubling_and_the_four_hours_are_the_facts() {
        let f = facts().unwrap();
        assert_eq!(
            format!("{:.1}", drift::STRIP.f64()),
            f.get("F-D.strip").unwrap(),
            "the strip is the fact's tenth of a mile"
        );
        assert!((drift::STRIP.f64() * UNITS_PER_MILE - 0.04).abs() < 1e-12);
        let r = drift::radius(q(1, 1));
        let twice = r * Q::int(2);
        assert_eq!((twice * twice) / (r * r), Q::int(4));
        assert_eq!(f.get("F-D.double.coverage_factor"), Some("1/4"));
        assert_eq!(f.get("F-D.double.area_factor"), Some("4"));
        let want = ["93", "65", "34", "13"];
        for (hr, w) in hours().iter().zip(want) {
            assert_eq!(format!("{:.0}", 100.0 * hr.seeing), w);
            let key = format!("F-D.h{}.seeing", hr.h);
            assert_eq!(f.get(&key), Some(w), "{key}");
            let cover = format!("F-D.h{}.coverage", hr.h);
            assert_eq!(f.get(&cover).unwrap(), format!("{:.2}", hr.coverage));
        }
        // the same track over a growing cloud: the chance only falls
        let s: Vec<f64> = hours().iter().map(|h| h.seeing).collect();
        assert!(s.windows(2).all(|w| w[1] < w[0]));
    }

    /// The hour line puts the four hours where its ends say, in order, and
    /// the zigzag is the same track length in both squares.
    #[test]
    fn the_hour_line_and_the_zigzags_are_laid_true() {
        assert_eq!(hour_x(0.0), LINE_X.0);
        assert_eq!(hour_x(HOURS_MAX), LINE_X.1);
        let xs: Vec<f64> = drift::hours().iter().map(|h| hour_x(h.f64())).collect();
        assert!(xs.windows(2).all(|w| w[0] < w[1]));
        let length = |pts: &[Vec3]| -> f64 {
            pts.chunks(2)
                .map(|c| (c[1][0] - c[0][0]).hypot(c[1][1] - c[0][1]))
                .sum()
        };
        let small = length(&zigzag(v(0.0, 0.0), 2.0, 8));
        let big = length(&zigzag(v(0.0, 0.0), 4.0, 4));
        assert!((small - big).abs() < 1e-12, "{small} against {big}");
        // 8 passes in a square of side 2 are 4 times as dense as 4 passes
        // in a square of side 4: spacing is area over length
        let spacing = |side: f64, passes: f64| side / passes;
        assert!((spacing(4.0, 4.0) / spacing(2.0, 8.0) - 4.0).abs() < 1e-12);
    }

    /// 10_4's hours on the curve: the small clouds above the plot never meet
    /// each other, each hour's words stand under the curve and inside the
    /// plot, and the last hour's (nearest the right edge) end before it.
    #[test]
    fn the_hours_on_the_curve_do_not_touch() {
        let w = curve_window();
        let hs = hours();
        let r_of = |hr: &Hour| SMALL_CLOUD_R * radius_miles(hr.h.f64()) / radius_miles(HOURS_MAX);
        let mut by_x: Vec<(f64, f64)> = hs
            .iter()
            .map(|hr| (w.p(hr.coverage, 0.0)[0], r_of(hr).max(0.06)))
            .collect();
        by_x.sort_by(|a, b| a.0.total_cmp(&b.0));
        for pair in by_x.windows(2) {
            let gap = pair[1].0 - pair[0].0 - pair[0].1 - pair[1].1;
            assert!(gap > 0.05, "two hours' clouds are {gap} apart");
        }
        for hr in &hs {
            let at = w.p(hr.coverage, 100.0 * hr.seeing);
            let width = format!("{} {:.0}%", hours_label(hr.h), 100.0 * hr.seeing)
                .chars()
                .count() as f64
                * 24.0
                * super::super::draw::CHAR_UNITS_PER_POINT;
            let fits_right = at[0] + HOUR_WORDS_GAP + width <= w.right() - HOUR_WORDS_EDGE;
            let (drop, right_end) = if fits_right {
                (HOUR_WORDS_DROP, at[0] + HOUR_WORDS_GAP + width)
            } else {
                (HOUR_WORDS_DROP_FAR, at[0] - HOUR_WORDS_GAP)
            };
            assert!(right_end <= w.right() - HOUR_WORDS_EDGE + 1e-9);
            // 24-point words are about 0.24 high: under the point, over the plot's floor
            assert!(
                at[1] - drop - 0.12 > w.bottom(),
                "{}: {}",
                hours_label(hr.h),
                at[1] - drop
            );
        }
        assert!(
            hs.iter().any(|hr| {
                let at = w.p(hr.coverage, 100.0 * hr.seeing);
                at[0] + HOUR_WORDS_GAP + 0.9 > w.right() - HOUR_WORDS_EDGE
            }),
            "one hour (the first) needs the words on its left"
        );
    }

    /// The narrated draft's overlaps in 10_2 and 10_3: the cloud at its
    /// largest (twelve hours) stands between the model tag and the hour
    /// line, and the hour line's labels stand over the carried question
    /// without touching it ("6 h" did).
    #[test]
    fn the_cloud_fits_between_the_model_tag_and_the_hour_line() {
        let r = radius_miles(HOURS_MAX) * UNITS_PER_MILE;
        assert!(START[1] + r < 3.55 - 0.3, "the cloud's top is {}", START[1] + r);
        assert!(START[1] - r > LINE_Y + 0.1, "the cloud's bottom is {}", START[1] - r);
        // a label is 22 points high: a fifth of a unit about its centre
        let labels_bottom = LINE_Y - 0.32 - 0.1;
        let question_top = QUIZ_LINES_Y[0] + 0.13;
        assert!(
            labels_bottom - question_top >= 0.05,
            "{labels_bottom} over {question_top}"
        );
    }

    /// R2 (M5, S2, R9, D13): 10_3's label of the strip is Table H-15's words
    /// with the facts' numbers, the flight's length stands by the strip's
    /// left end (clear of the cloud's edge and of the last known position's
    /// words), the table's label stands over the held cloud inside the safe
    /// area, and the lower-third says the strip is an effective width, in one
    /// line.
    #[test]
    fn the_sweep_width_labels_say_what_the_facts_say_and_stand_clear() {
        let f = facts().unwrap();
        let get = |k: &str| f.get(k).unwrap().to_owned();
        let table = sweep_table_words(
            &get("F-D.strip"),
            &get("F-D.strip.feet"),
            &get("F-D.strip.metres"),
            &get("F-D.strip.altitude.feet"),
        );
        assert_eq!(
            table,
            "sweep width 0.1 sea mile, 608 ft, 185 m: person in the water, helicopter at 300 feet. US Coast Guard, Table H-15"
        );
        let flight = flight_words(&get("F-D.track"));
        assert_eq!(flight, "one flight: 180 sea miles of track");
        // the table's label: over the cloud's top, inside the frame's safe
        // area on both sides and above
        let cloud_top = START[1] + radius_miles(HOURS_MAX) * UNITS_PER_MILE;
        let half = tag_width(&table, SWEEP_TAG_PT) / 2.0;
        assert!(half < FRAME_X_RADIUS - 0.15, "the table's label is {} wide", 2.0 * half);
        assert!(SWEEP_TAG_Y - 0.21 > cloud_top + 0.05, "{cloud_top}");
        const {
            assert!(SWEEP_TAG_Y + 0.21 < FRAME_Y_RADIUS - 0.15);
        }
        // the flight's tag: its right end clear of the strip's left end, its
        // left end clear of the diamond, and over the last known position's
        // words
        let at = flight_tag_at(&flight);
        let half = tag_width(&flight, FLIGHT_TAG_PT) / 2.0;
        let strip_left = centre(HOURS_MAX)[0] - radius_miles(HOURS_MAX) * UNITS_PER_MILE;
        assert!(at[0] + half <= strip_left - FLIGHT_TAG_GAP + 1e-9);
        assert!(at[0] - half > START[0] + 0.16 + 0.3, "{}", at[0] - half);
        // (and right of the words, which stand left of the diamond)
        assert!(
            at[0] - half > START[0] - LAST_KNOWN_GAP,
            "the flight's tag meets the last known position's words"
        );
        // the lower-third is one line and says both halves of "effective"
        assert!(EFFECTIVE_WIDTH.chars().count() <= 92);
        for part in [
            "effective width",
            "not a hard edge",
            "outside the strip",
            "misses some inside",
        ] {
            assert!(EFFECTIVE_WIDTH.contains(part), "{part}");
        }
        assert_eq!(SWEEP_LABEL, "sweep width:");
    }

    /// 10_1, 10_2: the words "last known position" stand where the cloud
    /// never goes. Under the diamond they were under the dots and the edge's
    /// circle from half an hour to seven (a dense read of the narrated draft
    /// had the cloud over "known pos"): they stand left of the diamond, three
    /// short lines, past the first circle and inside the frame.
    #[test]
    fn the_last_known_positions_words_stand_where_the_cloud_never_goes() {
        use super::super::draw::CHAR_UNITS_PER_POINT;
        let lines: Vec<&str> = LAST_KNOWN.lines().collect();
        assert_eq!(lines.join(" "), "last known position");
        let w = lines.iter().map(|l| l.chars().count()).max().unwrap() as f64 * 24.0 * CHAR_UNITS_PER_POINT;
        // three 24 pt lines are about 0.75 tall, centred on the diamond
        let (right, tall) = (START[0] - LAST_KNOWN_GAP, 0.75);
        let (left, bottom, top) = (right - w, START[1] - tall / 2.0, START[1] + tall / 2.0);
        assert!(left > -FRAME_X_RADIUS + 0.15, "the words begin at {left}");
        // the cloud's disc and its edge's stroke, hour by hour and between
        for k in 0..=240 {
            let h = HOURS_MAX * f64::from(k) / 240.0;
            let c = centre(h);
            let reach = radius_miles(h) * UNITS_PER_MILE + 0.05;
            let near = ((c[0] - c[0].clamp(left, right)), (c[1] - c[1].clamp(bottom, top)));
            assert!(
                near.0.hypot(near.1) > reach,
                "at {h:.1} h the cloud (centre {c:?}, reach {reach:.2}) meets the words"
            );
        }
    }

    /// R2 (M5, D6, R1): the circles' rows are the facts' radius and area
    /// (1.45 and 6.6, 2.35 and 17.3, 3.70 and 43.0, 6.40 and 128.7), the model
    /// agrees with them (twelve hours is 4.4 times as wide and 19.5 times the
    /// area of one hour), and the list stands left of the plot between the
    /// labels of 100% and 50%.
    #[test]
    fn the_circles_rows_are_the_facts_and_stand_left_of_the_plot() {
        let f = facts().unwrap();
        let mut sizes = Vec::new();
        for h in drift::hours() {
            let key = format!("F-D.h{h}");
            sizes.push((
                h.fixed(0),
                f.get(&format!("{key}.radius")).unwrap().to_owned(),
                f.get(&format!("{key}.area")).unwrap().to_owned(),
            ));
        }
        let rows = circle_rows(&sizes);
        assert_eq!(
            rows,
            [
                "1 hour: radius 1.45, area 6.6 sq sea miles",
                "3: 2.35, 17.3",
                "6: 3.70, 43.0",
                "12: 6.40, 128.7",
            ]
        );
        // the model's own numbers: the fact is the model's radius and area
        for (h, (_, r, a)) in drift::hours().into_iter().zip(&sizes) {
            assert_eq!(&drift::radius(h).fixed(2), r);
            assert_eq!(&format!("{:.1}", drift::area(h)), a);
        }
        let (h1, h12) = (drift::hours()[0], drift::hours()[3]);
        assert_eq!(
            format!("{:.1}", drift::radius(h12).f64() / drift::radius(h1).f64()),
            "4.4"
        );
        assert_eq!(format!("{:.1}", drift::area(h12) / drift::area(h1)), "19.5");
        // room: the widest row ends before the plot's left edge and the
        // labels' column, and the rows stand between the 100% and 50% labels
        let w = curve_window();
        let widest = rows.iter().map(|r| r.chars().count()).max().unwrap_or(0) as f64;
        let right = CIRCLES_X + widest * CIRCLES_PT * CHAR_UNITS_PER_POINT;
        // the 100% and 50% labels are over and under the rows, so the rows
        // may stand in their column: they only have to end before the frame
        assert!(right < w.p(0.0, 0.0)[0] - 0.15, "the list ends at {right}");
        const {
            assert!(CIRCLES_X > -FRAME_X_RADIUS + 0.15);
        }
        let top = CIRCLES_TOP + 0.1;
        let bottom = CIRCLES_TOP - CIRCLES_STEP * (rows.len() as f64 - 1.0) - 0.1;
        assert!(top < w.p(0.0, 100.0)[1] - 0.12, "{top}");
        assert!(bottom > w.p(0.0, 50.0)[1] + 0.12, "{bottom}");
    }

    /// R2 (S11, D6, S4, S10, R1): the model label names the flight, how it is
    /// spread and the cloud's edge, from the facts; "for any doubling" stands
    /// right of the "× ¼" inside the frame; the three lines and the hold's
    /// line stack under the plot's axis label inside the frame's safe area.
    #[test]
    fn the_model_label_defines_the_clouds_edge_and_stacks_under_the_plot() {
        let f = facts().unwrap();
        let get = |k: &str| f.get(k).unwrap().to_owned();
        let lines = model_lines(&get("F-D.track"), &get("F-D.start_radius"), &get("F-D.error_share"));
        assert_eq!(
            lines[0],
            "model: the same 180 sea miles of track spread evenly over the circle as random strips (a cautious floor);"
        );
        assert_eq!(
            lines[1],
            "one flight taken as instant; the cloud a uniform disc whose edge holds every dot:"
        );
        assert_eq!(lines[2], "1 mile plus 0.3 of the distance drifted");
        // the model's edge is that: the starting radius plus the share of the
        // distance drifted
        assert_eq!(drift::START_RADIUS.f64(), 1.0);
        assert!((drift::ERROR_SHARE.f64() - 0.3).abs() < 1e-12);
        let w = curve_window();
        for l in &lines {
            let half = l.chars().count() as f64 * MODEL_PT * CHAR_UNITS_PER_POINT / 2.0;
            assert!(w.center[0] - half > -FRAME_X_RADIUS + 0.15, "{l}");
            assert!(w.center[0] + half < FRAME_X_RADIUS - 0.15, "{l}");
        }
        // vertical stack: the axis label, three lines, the hold's line
        let x_label_bottom = w.bottom() - 0.5 - 0.15;
        // (18 points are about 0.23 high, 22 points about 0.25)
        let first = w.bottom() - MODEL_DROP;
        let last = first - MODEL_STEP * 2.0;
        assert!(first + 0.115 < x_label_bottom, "the first line meets the axis label");
        assert!(
            last - 0.115 > SURVIVAL_Y + 0.125 + 0.03,
            "the last line meets the hold's"
        );
        const {
            assert!(SURVIVAL_Y - 0.125 > -FRAME_Y_RADIUS + 0.15);
        }
        let hold = NOT_MODELLED.chars().count() as f64 * 22.0 * CHAR_UNITS_PER_POINT / 2.0;
        assert!(w.center[0] + hold < FRAME_X_RADIUS - 0.15);
        assert!(NOT_MODELLED.starts_with("not in this model: how long a person survives"));
        // "for any doubling" follows the quartering and fits the frame
        let end = DOUBLING_X + ANY_DOUBLING.chars().count() as f64 * 24.0 * CHAR_UNITS_PER_POINT;
        const {
            assert!(DOUBLING_X > 3.6 + 0.5);
        }
        assert!(end < FRAME_X_RADIUS - 0.15, "{end}");
    }
}
