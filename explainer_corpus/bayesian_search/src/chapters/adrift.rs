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
    QUIZ_LINES_Y, Spoken, carry_pause, fade_in_all, fade_out_all, faded_line, fail, flat, line_style, map as lit_blue,
    say, search as teal, tween_linear,
};
use crate::facts::facts;
use crate::model::{Q, drift, koopman};

pub struct Adrift {
    pub kit: Kit,
}

/// The narration lines this chapter speaks, in order.
pub const LINES: [&str; 4] = ["10_1", "10_2", "10_3", "10_4"];

// the chapter's fixed words (numbers are built from facts)
const LAST_KNOWN: &str = "last known position";
const WIND: &str = "wind";
const CURRENT: &str = "current";
const LEEWAY: &str = "leeway:";
const LEEWAY_TEXT: &str = "the push of the wind on what floats";
const MODEL: &str = "model";
const SWEEP_LABEL: &str = "sweep width:";
const HOUR: &str = "h";
const COVERAGE_LABEL: &str = "coverage = strip × track ÷ area";
const X_LABEL: &str = "coverage";
const Y_LABEL: &str = "chance one flight sees the person";
const SAME_TRACK_HEAD: &str = "the same";
const SAME_TRACK_TAIL: &str = "sea miles of track, spread evenly over the circle";

// ------------------------------------------------------------------ geometry

/// Scene units a sea mile is drawn as.
const UNITS_PER_MILE: f64 = 0.40;
/// Where the person was last known.
const START: Vec3 = [-5.4, 0.3, 0.0];
/// The hour line along the bottom: its ends, and its height (its labels sit
/// above the question's line in the caption band).
const LINE_X: (f64, f64) = (-6.2, 6.2);
const LINE_Y: f64 = -2.35;
/// Hours at the line's right end: the last the chapter shows.
const HOURS_MAX: f64 = 12.0;
/// How long the drift still runs after the question is asked, seconds.
const DRIFT_AFTER_ASK_S: f64 = 2.4;
/// How opaque a dot of the cloud is (the cloud is thick near its middle).
const CLOUD_OPACITY: f64 = 0.9;

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
        let last_label = kit.text_left(
            stage,
            LAST_KNOWN,
            24.0,
            Palette::dim(),
            v(START[0] + 0.3, START[1] - 0.45),
        )?;
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
        let sweep = lower_third(
            &kit,
            stage,
            SWEEP_LABEL,
            &format!(
                "the width of the strip: {} sea mile, {} m, for a person in the water seen from a helicopter at {} feet. US Coast Guard, Table H-15",
                fact("F-D.strip")?,
                fact("F-D.strip.metres")?,
                fact("F-D.strip.altitude.feet")?
            ),
        )?;
        line.wait_for(stage, "It is a tenth")?;
        fade_in_all(stage, &[sweep], 0.5)?;

        // ------------------------------------------------------- 10_4
        // twice the radius is four times the area; the same flight over a
        // cloud that has grown buys less every hour
        let line = say(&kit, stage, LINES[3])?;
        // the cloud, its circle and the hour line leave; two outline
        // circles, one twice as wide, each in its square, take the stage
        let leaving = vec![edge, strip, sweep, last_known, last_label];
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
        line.wait_for(stage, "so each square mile")?;
        fade_in_all(stage, &tiling, 0.5)?;
        fade_in_all(stage, &[equation, times, area_words], 0.6)?;

        // chapter 09's curve, and four markers sliding down it as the hours pass
        line.wait_for(stage, "In this model")?;
        let mut away = pair.clone();
        away.extend(&tiling);
        away.extend([equation, times, area_words]);
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
        furniture.push(kit.text_left(stage, Y_LABEL, 24.0, Palette::dim(), v(w.left(), w.top() + 0.3))?);
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
        let same = kit.text(
            stage,
            &format!("{MODEL}: {SAME_TRACK_HEAD} {} {SAME_TRACK_TAIL}", fact("F-D.track")?),
            22.0,
            Palette::dim(),
            v(w.center[0], w.bottom() - 0.95),
        )?;
        furniture.push(same);
        fade_in_all(stage, &furniture, 0.6)?;
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
        let mark_hour = |stage: &mut Stage<'_>, hr: &Hour, stops: &mut Vec<Mob>| -> fmn::Result<()> {
            let at = w.p(hr.coverage, 100.0 * hr.seeing);
            let cloud_r = 0.55 * radius_miles(hr.h.f64()) / radius_miles(HOURS_MAX);
            let small_cloud = stage.arena_mut().add(
                Circle::new()
                    .radius(cloud_r.max(0.06))
                    .arc_center(v(at[0], w.top() + 0.75))
                    .style(line_style(teal(), 3.0))
                    .build(),
            );
            let words = kit.text_left(
                stage,
                &format!("{} {:.0}%", hours_label(hr.h), 100.0 * hr.seeing),
                24.0,
                Palette::ink(),
                v(at[0] + 0.2, at[1] + 0.15),
            )?;
            fade_in_all(stage, &[small_cloud, words], 0.4)?;
            stops.extend([small_cloud, words]);
            Ok(())
        };
        line.wait_for(stage, "one hour in")?;
        stage.add_to_scene(rider)?;
        mark_hour(stage, &spans[0], &mut stops)?;
        line.wait_for(stage, "Twelve hours in")?;
        for hr in &spans[1..] {
            stage.play(tween_linear(cov, hr.coverage, 0.9)?)?;
            mark_hour(stage, hr, &mut stops)?;
        }
        line.wait_end(stage)?;
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
}
