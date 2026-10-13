//! Chapter 03, three angles: nine numbers is a lot, three should do, and a
//! gimbal tries (03_1); turned a quarter turn, two rings do the same job
//! (03_2, 03_3: gimbal lock); Apollo had this problem and Collins asked for a
//! fourth ring (03_4, 03_5); and the fault is in the three numbers: software
//! stores nine, or four (03_6).

use std::f64::consts::TAU;

use explainer_kit::props::{lower_third, precisely};
use explainer_kit::{Kit, Palette, Timed, play, tween, v};
use fmn::prelude::*;

use super::draw::{
    Ask, BOOK_HALF, Choice, PartSpec, Pin, QuizCard, QuizLook, Shape, View, World, circle_points, dashed_rect,
    fact_f64, fade_in_all, flat, leave, line_style, live_bar_of, mix, panel, pause, pose_fn, say, still, tween_linear,
    value,
};
use crate::model::{self, DEG, M3, Pose, V3, X, Y, Z};
use crate::palette;

pub struct ThreeAngles {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const OUTER_RING: &str = "outer ring";
const MIDDLE_RING: &str = "middle ring";
const INNER_RING: &str = "inner ring";
const WAYS_TO_TURN: &str = "ways to turn:";
const GIMBAL_LOCK: &str = "gimbal lock";
const BAR_WORDS: &str = "of the third way to turn, this much is left";
const LAMP: &str = "lamp";
const QUOTE: &str = "How about sending me a fourth gimbal for Christmas.";
const QUOTE_BY: &str = "Michael Collins, Apollo 11,";
const QUOTE_WHEN: &str = "hours into the flight";
const THREE_ANGLES: &str = "three angles";
const ROTATION_MATRIX: &str = "rotation matrix";
const UNIT_QUATERNION: &str = "unit quaternion";
const A_BAD_SPOT: &str = "a bad spot";
const NUMBERS: &str = "numbers";
const RULES: &str = "rules";
const ONE_RULE: &str = "rule";
const NO_JUMP: &str = "three numbers cannot give every orientation one name without a jump somewhere; nine give one name each, four give two (q and −q), with no jump";
const OWN_MEASURE: &str = "The bar is this video's own measure";
const PERCENT: &str = "%";
const DEGREE: &str = "°";

/// The beats of the chapter, in order (storyboard ids).
const BEATS: [&str; 6] = ["03_1", "03_2", "03_3", "03_4", "03_5", "03_6"];

/// 03_2's quiz: do the two rings do different things any more?
pub const ASK: Ask = Ask {
    line: BEATS[1],
    question: "different, or\nthe same?",
    choices: &[
        Choice {
            key: "different",
            label: "different",
        },
        Choice {
            key: "same",
            label: "the same",
        },
    ],
};

/// What each ring turns about, outer to inner, as `G-E.rings.order` says:
/// the vertical, then the line pointing away, then the left-to-right line.
const RING_AXES: [V3; 3] = [Z, Y, X];
/// Each ring's radius, outer to inner, and the fourth ring Collins asked for.
const RADII: [f64; 3] = [1.95, 1.55, 1.15];
const FOURTH_RADIUS: f64 = 2.4;
// each ring is nested inside the one before, and the fourth round them all
const _: () = assert!(RADII[0] > RADII[1] && RADII[1] > RADII[2] && FOURTH_RADIUS > RADII[0]);
/// The book is drawn this fraction of full size inside the inner ring.
const BOOK_FIT: f64 = 0.76;
/// "A hundred and five hours": day 4, hour 9 of the flight, from the
/// transcript's stamp `04 09 00 30` (NASA air-to-ground transcript, tape
/// 67/11, p. 337).
const COLLINS_STAMP_DAYS_HOURS: (u32, u32) = (4, 9);

/// The three ring frames for yaw, pitch and roll (radians): the outer ring
/// turns about the vertical, the middle ring about the outer's line pointing
/// away, the inner about the middle's left-to-right line. The inner frame,
/// carrying the book, is `model::rings`.
fn frames(yaw: f64, pitch: f64, roll: f64) -> [M3; 3] {
    let outer = model::turn(Z, yaw);
    let middle = model::mul(&outer, &model::turn(Y, pitch));
    let inner = model::mul(&middle, &model::turn(X, roll));
    [outer, middle, inner]
}

/// How many ways the three rings can turn the book right now: three unless
/// the three axes lie in one plane (the volume they span vanishes).
fn ways_to_turn(pitch: f64) -> u32 {
    let span = model::rings_span(RING_AXES[0], RING_AXES[1], RING_AXES[2], pitch);
    if span < 1e-9 { 2 } else { 3 }
}

/// The share of the third way to turn that is left at middle angle `pitch`.
fn left_of_third(pitch: f64) -> f64 {
    model::rings_left(pitch)
}

/// A ring of `radius` about the origin in the plane perpendicular to
/// `normal`, cut into arcs so each is sorted by its own depth, with a dot
/// where it is joined to its neighbours.
fn ring(normal: V3, radius: f64, joins: &[V3], color: Srgb) -> Vec<Shape> {
    let n = 16;
    let mut out: Vec<Shape> = (0..n)
        .map(|k| {
            let (a, b) = (TAU * k as f64 / n as f64, TAU * (k + 1) as f64 / n as f64);
            Shape::Path {
                pts: circle_points([0.0; 3], normal, radius, a, b, 4),
                color,
                width: 7.0,
                closed: false,
                bias: 0.0,
            }
        })
        .collect();
    for dir in joins.iter().copied() {
        for sign in [-1.0, 1.0] {
            out.push(Shape::Dot {
                at: [sign * radius * dir[0], sign * radius * dir[1], sign * radius * dir[2]],
                color: Palette::ink(),
                radius: 0.075,
            });
        }
    }
    out
}

/// A flat card: a rounded back with a faint edge, centred at `at`.
fn card(stage: &mut Stage<'_>, at: Vec3, w: f64, h: f64) -> fmn::Result<Mob> {
    let back = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.16)
        .style(Style::default().fill(panel(), 0.96).stroke(Palette::faint(), 2.0, 1.0))
        .build()?
        .moved_to(at);
    Ok(stage.arena_mut().add(back))
}

impl SceneConstruct for ThreeAngles {
    fn name(&self) -> &str {
        "03_three_angles"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let blue = palette::turn();
        let warn = palette::warn();

        // ---- the gimbal: three rings, one inside the next, and the book in
        // the inner one; its angles are three trackers, in degrees
        let (yaw, pitch, roll) = (
            stage.add_value_tracker(0.0),
            stage.add_value_tracker(0.0),
            stage.add_value_tracker(0.0),
        );
        let frame = move |k: usize| {
            pose_fn(move |s| Pose {
                r: frames(value(s, yaw) * DEG, value(s, pitch) * DEG, value(s, roll) * DEG)[k],
                p: [0.0; 3],
            })
        };
        let half: V3 = BOOK_HALF.map(|x| BOOK_FIT * x);
        let lock_line = vec![Shape::Dashed {
            a: [0.0, 0.0, -2.9],
            b: [0.0, 0.0, 2.9],
            color: warn,
            width: 4.5,
            dashes: 14,
        }];
        let missing_line = vec![Shape::Dashed {
            a: [-2.9, 0.0, 0.0],
            b: [2.9, 0.0, 0.0],
            color: warn,
            width: 4.5,
            dashes: 14,
        }];
        let fourth = ring(Y, FOURTH_RADIUS, &[Z, X], mix(blue, Palette::ink(), 0.45));
        // part 0 is the book; the rings are the next three
        let specs = vec![
            PartSpec::new(super::draw::book(half), frame(2)),
            PartSpec::new(ring(X, RADII[0], &[Z, Y], blue), frame(0)),
            PartSpec::new(ring(Z, RADII[1], &[Y, X], blue), frame(1)),
            PartSpec::new(ring(Y, RADII[2], &[X], blue), frame(2)),
            PartSpec::new(lock_line, still(Pose::NOTHING)).layer(1),
            PartSpec::new(missing_line, frame(0)).layer(1),
            PartSpec::new(fourth, still(Pose::NOTHING)),
        ];
        const RINGS: [usize; 3] = [1, 2, 3];
        const LOCK: usize = 4;
        const MISSING: usize = 5;
        const FOURTH: usize = 6;
        let view = View::new(1.45, v(-4.3, 0.1));
        let gimbal = World::build(stage, view, 3.4, 0.25, specs, Vec::<Pin>::new())?;
        for i in RINGS.into_iter().chain([LOCK, MISSING, FOURTH]) {
            stage.set_tracker_value(gimbal.parts[i].shown, 0.0)?;
        }
        gimbal.add_to_scene(stage)?;

        // 03_1: nine numbers is a lot; a gimbal tries with three rings
        let line = say(&kit, stage, BEATS[0])?;
        gimbal.show(stage, 0.8)?;
        line.wait_for(stage, "Here is a machine")?;
        for i in RINGS {
            play!(stage; tween(gimbal.parts[i].shown, 1.0, 0.5)?);
        }
        line.wait_for(stage, "each turning about its own axis")?;
        // three live readouts, one per ring
        let rows = [
            (OUTER_RING, yaw, 2.2),
            (MIDDLE_RING, pitch, 1.4),
            (INNER_RING, roll, 0.6),
        ];
        let mut texts = Vec::new();
        for (words, tracker, y) in rows {
            let label = kit.text_left(stage, words, 30.0, blue, v(-1.5, y))?;
            kit.live_number(stage, tracker, 0, false, 44.0, Palette::ink(), v(1.05, y), RIGHT)?;
            let sign = kit.text(stage, DEGREE, 44.0, Palette::ink(), v(1.25, y + 0.04))?;
            texts.extend([label, sign]);
        }
        fade_in_all(stage, &texts, 0.4)?;
        for tracker in [yaw, pitch, roll] {
            play!(stage; tween(tracker, 40.0, 0.6)?);
            play!(stage; tween(tracker, 0.0, 0.6)?);
        }

        // 03_2: a quarter turn of the middle ring: do the outer and inner
        // rings still do different things?
        let line = say(&kit, stage, BEATS[1])?;
        let card2 = QuizCard::build(&kit, stage, ASK, QuizLook::at(v(4.35, 0.3), 0.92))?;
        line.wait_for(stage, "Turn the middle ring a quarter turn")?;
        play!(stage; tween(pitch, 90.0, 1.5)?);
        line.wait_for(stage, "Now the outer ring's axis")?;
        play!(stage; tween(gimbal.parts[LOCK].shown, 1.0, 0.5)?);
        line.wait_last(stage)?;
        card2.show(stage, 0.5)?;
        card2.count(&kit, stage)?;

        // 03_3: the same axis: nothing is stuck, but only two ways to turn
        let line = say(&kit, stage, BEATS[2])?;
        line.wait_for(stage, "They turn the book about the same axis")?;
        let (locked_a, locked_b) = (30.0, -30.0);
        play!(stage; tween(yaw, locked_a, 0.7)?);
        play!(stage; tween(yaw, 0.0, 0.7)?);
        play!(stage; tween(roll, locked_b, 0.7)?);
        play!(stage; tween(roll, 0.0, 0.7)?);
        line.wait_for(stage, "Nothing is stuck")?;
        // the readout appears at three, then drops to two as the ring locks
        let ways_label = kit.text_left(stage, WAYS_TO_TURN, 30.0, Palette::ink(), v(-1.5, -0.4))?;
        let ways = kit.live_value(
            stage,
            move |s| f64::from(ways_to_turn(value(s, pitch) * DEG)),
            0,
            false,
            48.0,
            Palette::ink(),
            v(1.05, -0.4),
            RIGHT,
        )?;
        fade_in_all(stage, &[ways_label], 0.3)?;
        play!(stage; tween(pitch, 0.0, 0.6)?, tween(gimbal.parts[LOCK].shown, 0.0, 0.6)?);
        pause(stage, 0.4)?;
        play!(stage; tween(pitch, 90.0, 1.0)?);
        line.wait_for(stage, "This is gimbal lock")?;
        let lock_word = kit.text(stage, GIMBAL_LOCK, 44.0, warn, v(-4.3, -3.2))?;
        play!(stage; tween(gimbal.parts[MISSING].shown, 1.0, 0.5)?);
        fade_in_all(stage, &[lock_word], 0.4)?;

        // 03_4: Apollo: the lamp above 70 degrees, the reference dropped
        // above 85
        let line = say(&kit, stage, BEATS[3])?;
        let mut leaving = card2.mobs();
        leaving.extend([ways_label, ways, lock_word]);
        leave(stage, &leaving, &[gimbal.parts[MISSING].shown], 0.5)?;
        play!(stage; tween(pitch, 0.0, 0.8)?);
        let (lamp_deg, lost_deg) = (fact_f64("G-L.apollo.lamp_deg")?, fact_f64("G-L.apollo.lost_deg")?);
        let unit = 2.4;
        let bar_at = v(-1.5, -1.3);
        let bar = live_bar_of(
            stage,
            move |s| left_of_third(value(s, pitch) * DEG),
            bar_at,
            unit,
            0.34,
            blue,
        )?;
        let words = kit.text_left(stage, BAR_WORDS, 24.0, Palette::dim(), v(-1.5, -1.85))?;
        let percent = kit.live_value(
            stage,
            move |s| (100.0 * left_of_third(value(s, pitch) * DEG)).round(),
            0,
            false,
            44.0,
            Palette::ink(),
            v(2.3, -0.5),
            RIGHT,
        )?;
        let percent_sign = kit.text(stage, PERCENT, 44.0, Palette::ink(), v(2.5, -0.46))?;
        fade_in_all(stage, &[words, percent_sign], 0.4)?;
        line.wait_for(stage, "Past seventy degrees")?;
        // the 70 and 85 degree marks on the bar: where the share is
        let at_lamp = left_of_third(lamp_deg * DEG);
        let at_lost = left_of_third(lost_deg * DEG);
        let mark_lamp = {
            let x = bar_at[0] + unit * at_lamp;
            let m = polyline(vec![v(x, bar_at[1] - 0.3), v(x, bar_at[1] + 0.3)])
                .style(line_style(Palette::ink(), 3.0))
                .build();
            stage.arena_mut().add(m)
        };
        let mark_lost = {
            let x = bar_at[0] + unit * at_lost;
            let m = polyline(vec![v(x, bar_at[1] - 0.3), v(x, bar_at[1] + 0.3)])
                .style(line_style(warn, 3.0))
                .build();
            stage.arena_mut().add(m)
        };
        let t_lamp = kit.text(
            stage,
            &format!("{lamp_deg:.0}{DEGREE}"),
            26.0,
            Palette::ink(),
            v(bar_at[0] + unit * at_lamp, bar_at[1] + 0.55),
        )?;
        let t_lost = kit.text(
            stage,
            &format!("{lost_deg:.0}{DEGREE}"),
            26.0,
            warn,
            v(bar_at[0] + unit * at_lost - 0.1, bar_at[1] - 0.55),
        )?;
        fade_in_all(stage, &[mark_lamp, mark_lost, t_lamp, t_lost], 0.4)?;
        // the lamp
        let lamp_at = v(2.0, -1.3);
        let lamp = stage.always_redraw(move |s| {
            let lit = value(s, pitch) >= lamp_deg;
            let c = Circle::new()
                .radius(0.2)
                .arc_center(lamp_at)
                .style(flat(warn, if lit { 1.0 } else { 0.18 }))
                .build();
            s.add(c)
        });
        stage.add_to_scene(lamp)?;
        let lamp_word = kit.text(stage, LAMP, 24.0, Palette::dim(), v(2.0, -1.75))?;
        fade_in_all(stage, &[lamp_word], 0.3)?;
        play!(stage; tween_linear(pitch, lamp_deg, 1.6)?);
        line.wait_for(stage, "Past eighty-five")?;
        play!(stage; tween_linear(pitch, lost_deg, 0.9)?);
        let apollo = lower_third(
            &kit,
            stage,
            "Apollo guidance computer:",
            &format!(
                "gimbal lock lamp above {lamp_deg:.0}{DEGREE}, reference dropped above {lost_deg:.0}{DEGREE}. {OWN_MEASURE}"
            ),
        )?;
        play!(stage; fade_in(stage.arena_mut(), apollo, ORIGIN, 1.0)?.rt(0.5));

        // 03_5: Collins's request, and a fourth ring
        kit.hold(stage)?;
        let line = say(&kit, stage, BEATS[4])?;
        let bar_mobs = [
            words,
            percent,
            percent_sign,
            mark_lamp,
            mark_lost,
            t_lamp,
            t_lost,
            lamp,
            lamp_word,
            bar,
            apollo,
        ];
        leave(stage, &bar_mobs, &[], 0.5)?;
        play!(stage; tween(pitch, 0.0, 0.7)?);
        let (days, hours) = COLLINS_STAMP_DAYS_HOURS;
        let into_flight = days * 24 + hours;
        let back = card(stage, v(2.2, 0.4), 8.6, 2.9)?;
        let quote = kit.text(stage, QUOTE, 36.0, Palette::ink(), v(2.2, 0.85))?;
        let by = kit.text(
            stage,
            &format!("{QUOTE_BY} {into_flight} {QUOTE_WHEN}"),
            26.0,
            Palette::dim(),
            v(2.2, -0.05),
        )?;
        line.wait_for(stage, "How about sending me")?;
        fade_in_all(stage, &[back, quote, by], 0.6)?;
        line.wait_for(stage, "A fourth ring")?;
        play!(stage; tween(gimbal.parts[FOURTH].shown, 1.0, 0.8)?);
        kit.music_breath(stage, 1.5)?;

        // 03_6: the fault is in the three numbers: nine, or four
        let line = say(&kit, stage, BEATS[5])?;
        leave(stage, &[back, quote, by], &[gimbal.shown], 0.6)?;
        let (matrix, quaternion, arrow) = (
            fact_f64("G-S.numbers.turn_matrix")? as u32,
            fact_f64("G-S.numbers.quaternion")? as u32,
            fact_f64("G-R.numbers.arrow")? as u32,
        );
        let boxes = [
            (THREE_ANGLES, format!("{arrow} {NUMBERS}"), false, -4.4),
            (
                ROTATION_MATRIX,
                format!("{matrix} {NUMBERS} − {} {RULES} = {arrow}", matrix - arrow),
                true,
                0.0,
            ),
            (
                UNIT_QUATERNION,
                format!("{quaternion} {NUMBERS} − {} {ONE_RULE} = {arrow}", quaternion - arrow),
                true,
                4.4,
            ),
        ];
        let mut all = Vec::new();
        for (title, sums, good, x) in boxes {
            let back = if good {
                card(stage, v(x, 0.7), 4.0, 2.2)?
            } else {
                dashed_rect(stage, v(x, 0.7), 4.0, 2.2, warn, 3.0)?
            };
            let head = kit.text(stage, title, 32.0, Palette::ink(), v(x, 1.3))?;
            let body = kit.text(stage, &sums, 26.0, Palette::dim(), v(x, 0.6))?;
            all.extend([back, head, body]);
            if good {
                let tick = polyline(vec![v(x - 0.2, 0.0), v(x - 0.05, -0.18), v(x + 0.25, 0.2)])
                    .style(line_style(palette::good(), 6.0))
                    .build();
                all.push(stage.arena_mut().add(tick));
            } else {
                let spot = kit.text(stage, A_BAD_SPOT, 26.0, warn, v(x, 0.05))?;
                all.push(spot);
            }
        }
        line.wait_for(stage, "So software stores")?;
        fade_in_all(stage, &all, 0.8)?;
        let exact = precisely(&kit, stage, NO_JUMP)?;
        play!(stage; fade_in(stage.arena_mut(), exact, ORIGIN, 1.0)?.rt(0.5));
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::FRAC_PI_2;

    use super::super::draw::fact;
    use super::*;

    #[test]
    fn the_beats_are_the_storyboards_and_the_scripts() {
        let board = explainer_kit::storyboard::parse(crate::STORYBOARD).unwrap();
        let chapter = board.iter().find(|c| c.key == "03_three_angles").expect("chapter 03");
        let ids: Vec<&str> = chapter.beats.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(ids, BEATS);
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for beat in &chapter.beats {
            let words = beat.spoken().expect("every beat of chapter 03 is spoken");
            assert_eq!(script.text(&beat.id), Some(words.as_str()), "{}", beat.id);
        }
    }

    /// 03_2: a four-second countdown; the computed answer is that the two
    /// rings now do the same thing.
    #[test]
    fn the_quiz_asks_what_the_rings_answer() {
        assert_eq!(ASK.seconds().unwrap(), 4);
        assert_eq!(fact("G-Q.03_2.answer").unwrap(), "same");
        assert_eq!(ASK.answer_index().unwrap(), 1);
    }

    /// 03_1: the rings are drawn in the order the facts say (outer about the
    /// vertical, then away, then left to right), and the inner frame is the
    /// model's own `rings` matrix.
    #[test]
    fn the_rings_turn_about_z_then_y_then_x() {
        let order: Vec<&str> = RING_AXES
            .iter()
            .map(|a| match *a {
                x if x == X => "x",
                y if y == Y => "y",
                _ => "z",
            })
            .collect();
        assert_eq!(order.join(","), fact("G-E.rings.order").unwrap());
        let (yaw, pitch, roll) = (0.3, 0.8, -0.5);
        let f = frames(yaw, pitch, roll);
        assert!(model::gap(&f[2], &model::rings(yaw, pitch, roll)) < 1e-12);
        assert!(model::gap(&f[0], &model::turn(Z, yaw)) < 1e-12);
        // the book fits inside the inner ring
        let fits = BOOK_FIT * (BOOK_HALF[0].hypot(BOOK_HALF[1]));
        assert!(fits < RADII[2], "the book fits inside the inner ring");
    }

    /// 03_2 and 03_3: at a quarter turn of the middle ring the outer and
    /// inner rings turn the book alike (yaw 30 with no roll, and roll −30
    /// with no yaw, are one orientation); off it they do not; and there are
    /// two ways to turn, not three.
    #[test]
    fn a_quarter_turn_of_the_middle_ring_locks_it() {
        let ypr = |a: f64, b: f64, c: f64| model::rings(a * DEG, b * DEG, c * DEG);
        assert!(model::gap(&ypr(30.0, 90.0, 0.0), &ypr(0.0, 90.0, -30.0)) < 1e-12);
        assert_eq!(fact("G-E.lock.same").unwrap(), "0.0000");
        assert!(
            model::gap(&ypr(30.0, 0.0, 0.0), &ypr(0.0, 0.0, -30.0)) > 0.1,
            "free of the lock they differ"
        );
        assert_eq!(ways_to_turn(0.0), 3);
        assert_eq!(ways_to_turn(FRAC_PI_2), 2);
        assert_eq!(ways_to_turn(70.0 * DEG), 3);
        // the dashed line the quiz shows is the outer axis, and the inner
        // ring's axis is on it
        let f = frames(0.0, FRAC_PI_2, 0.0);
        let inner_axis = model::apply(&f[1], X);
        assert!(model::norm(model::sub(inner_axis, [0.0, 0.0, -1.0])) < 1e-12);
        // the way to turn that is missing is the outer ring's left-to-right line
        let missing = model::apply(&f[0], X);
        assert!(model::dot(missing, inner_axis).abs() < 1e-12 && model::dot(missing, Z).abs() < 1e-12);
    }

    /// 03_4: the bar is the share of the third way to turn that is left:
    /// 34% at the lamp's angle and 9% where the reference is dropped (the
    /// facts' own tables).
    #[test]
    fn the_bar_reads_thirty_four_and_nine_percent() {
        let (lamp, lost) = (
            fact_f64("G-L.apollo.lamp_deg").unwrap(),
            fact_f64("G-L.apollo.lost_deg").unwrap(),
        );
        assert_eq!((lamp, lost), (70.0, 85.0));
        let percent = |deg: f64| (100.0 * left_of_third(deg * DEG)).round();
        assert_eq!(percent(lamp), 34.0);
        assert_eq!(percent(lost), 9.0);
        assert_eq!(percent(0.0), 100.0);
        assert!((left_of_third(lamp * DEG) - fact_f64("G-E.left.70").unwrap()).abs() < 1e-4);
        assert!((left_of_third(lost * DEG) - fact_f64("G-E.left.85").unwrap()).abs() < 1e-4);
        assert!(left_of_third(FRAC_PI_2) < 1e-12);
    }

    /// 03_5: a hundred and five hours is day 4, hour 9 of the stamp
    /// `04 09 00 30`; and the fourth ring is outside the three.
    #[test]
    fn collins_asked_a_hundred_and_five_hours_in() {
        let (days, hours) = COLLINS_STAMP_DAYS_HOURS;
        assert_eq!(days * 24 + hours, 105);
        assert_eq!(fact("G-L.apollo.gimbals").unwrap(), "3");
    }

    /// 03_6: nine numbers less six rules and four less one are three again,
    /// and no order of three rings is free of a bad spot.
    #[test]
    fn nine_and_four_numbers_are_three_with_their_rules() {
        let arrow = fact_f64("G-R.numbers.arrow").unwrap() as u32;
        let (matrix, quaternion) = (
            fact_f64("G-S.numbers.turn_matrix").unwrap() as u32,
            fact_f64("G-S.numbers.quaternion").unwrap() as u32,
        );
        assert_eq!((arrow, matrix, quaternion), (3, 9, 4));
        assert_eq!(matrix - arrow, 6);
        assert_eq!(quaternion - arrow, 1);
        assert_eq!(fact("G-X6.ring_orders").unwrap(), "12/12");
    }
}
