//! Chapter 10, two tanks: a real system. Two tanks of a hundred litres, with
//! pumps swapping ten litres a minute, and a hundred grams of salt stirred
//! into the left one (10_1). Salt leaves each tank at a rate of one tenth of
//! what it holds, per minute, and arrives in the other: two equations, four
//! numbers, one matrix; on the plane of chapter 09 (grams left across, grams
//! right up) the pair of readings is one dot, and the rule's arrows all point
//! along one diagonal, toward the line of equal salt (10_2). One big step of
//! ten minutes throws the salt clean through the balance line (10_3). Ten
//! steps do better, a thousand better still, and e^{10A} is where those steps
//! are heading (10_4). Your turn: where does the salt end up (10_5)? Fifty
//! grams in each, and the exponential says how fast (10_6).
//!
//! Every number is a fact: the rule is `F-T.rule`, the start `F-T.start`,
//! the one big step `F-V.10_3.one_step`, the steps and their errors
//! `F-V.10_4.*` and `F-T.off_by.*`, the answer `F-Q.10_5.answer`, the time
//! `F-V.10_6.minutes`, the total `F-T.total`.

use explainer_kit::{Kit, Palette, Timed, fade_all, play, thousands, tween, v};
use fmn::prelude::*;

use super::draw::{
    Flow, FlowSpec, MatrixGrid, Plane, QuizCard, Velocity, arrow_points, chip, entries, fact, fade_in_all, faded_line,
    fail, fit_width, flat, map2, music_hold, numbers, result, rule, say, substance, tween_linear, warn, whole,
};
use super::flow::{fade_in_anim, fade_out_anim};
use crate::facts::shown;
use crate::model::{Mat, expm_t, tanks};

// ------------------------------------------------------------------ choices

/// The tanks: centres, size, and how high the picture sits.
const TANK_X: [f64; 2] = [-5.4, -3.2];
const TANK_Y: f64 = 2.3;
const TANK_W: f64 = 1.7;
const TANK_H: f64 = 1.9;
/// The plane of grams: left across, right up, its origin at the lower left.
/// A hundred grams is 3.4 units.
const GRAMS: Plane = Plane {
    centre: [3.1, -1.9, 0.0],
    unit: 0.034,
};
/// The left edge of the rows of 10_4 and 10_6, under the tanks.
const TABLE_X: f64 = -6.7;
/// How far the plane's axes reach (grams).
const AXIS_G: f64 = 108.0;
/// An arrow's length in minutes of its own velocity.
const GAIN: f64 = 1.3;
/// How far the clock runs at the end (hours, as the question says).
const LONG_RUN_MIN: f64 = 60.0;
/// The quiz card, over the plane's side of the frame.
const CARD_AT: Vec3 = [4.4, 0.0, 0.0];
const CARD_SIZE: (f64, f64) = (5.0, 3.9);

// ------------------------------------------------------------- the geometry

/// The rule for the grams in each tank, per minute (`F-T.rule`).
fn tank_rule() -> Mat {
    tanks(shown::TANK_LITRES, shown::TANK_PUMPED)
}

/// Grams in the left and right tank after `t` minutes.
pub fn readings(t: f64) -> [f64; 2] {
    let g = expm_t(&tank_rule(), t).apply(&shown::TANK_START);
    [g[0], g[1]]
}

/// Where `n` straight steps of `minutes` in all end up, one point per step
/// (from the start): each step follows the arrow it stands on.
pub fn steps_path(n: u32, minutes: f64) -> Vec<[f64; 2]> {
    let step = Mat::identity(2).add(&tank_rule().scale(minutes / f64::from(n)));
    let mut here = shown::TANK_START;
    let mut out = vec![here];
    for _ in 0..n {
        let next = step.apply(&here);
        here = [next[0], next[1]];
        out.push(here);
    }
    out
}

/// How full the water in a tank looks with `grams` of salt in it (an
/// opacity).
pub fn water_level(grams: f64) -> f64 {
    (0.1 + 0.78 * grams / shown::TANK_START[0]).clamp(0.0, 0.9)
}

/// A fact that is a whole number of grams, as it is printed.
fn whole_fact(key: &str) -> fmn::Result<String> {
    Ok(whole(
        fact(key)?
            .parse::<f64>()
            .map_err(|e| fail(format!("fact {key}: {e}")))?,
    ))
}

// ----------------------------------------------------------------- the tanks

/// The two tanks, the pumps between them, and the salt each holds, which the
/// water's colour and two readouts follow.
struct Tanks {
    /// Everything that is drawn once and stays (frames, pumps, words).
    still: Vec<Mob>,
    /// The water of each tank (its opacity follows the grams).
    water: [Mob; 2],
    /// Grams in each tank, as shown. While `follow` reads above a half they
    /// are the exact solution at the clock's `minutes`; otherwise they are
    /// whatever the scene tweens them to.
    grams: [Mob; 2],
    follow: Mob,
    anchor: Mob,
}

impl Tanks {
    fn build(kit: &Kit, stage: &mut Stage<'_>, minutes: Mob) -> fmn::Result<Self> {
        let ink = Palette::ink();
        let mut still = Vec::new();
        let mut water = Vec::new();
        for &cx in &TANK_X {
            let w = Rectangle::new()
                .width(TANK_W - 0.2)
                .height(TANK_H - 0.2)
                .style(flat(substance(), 0.1))
                .build()?
                .moved_to(v(cx, TANK_Y));
            water.push(stage.arena_mut().add(w));
            let frame = Rectangle::new()
                .width(TANK_W)
                .height(TANK_H)
                .style(Style::default().fill(ink, 0.0).stroke(ink, 3.5, 1.0))
                .build()?
                .moved_to(v(cx, TANK_Y));
            still.push(stage.arena_mut().add(frame));
        }
        // two pumps between the tanks, one each way
        let (x0, x1) = (TANK_X[0] + TANK_W / 2.0, TANK_X[1] - TANK_W / 2.0);
        for (y, to_right) in [(TANK_Y + 0.3, true), (TANK_Y - 0.3, false)] {
            let (a, b) = if to_right {
                (x0 + 0.05, x1 - 0.05)
            } else {
                (x1 - 0.05, x0 + 0.05)
            };
            let p = polyline(arrow_points(v(a, y), v(b, y), 0.18))
                .style(faded_line(Palette::dim(), 4.0, 1.0))
                .build();
            still.push(stage.arena_mut().add(p));
        }
        let litres = format!("{} L", shown::TANK_LITRES);
        for &cx in &TANK_X {
            still.push(kit.text(stage, &litres, 24.0, Palette::dim(), v(cx, TANK_Y + TANK_H / 2.0 + 0.3))?);
        }
        let pumped = format!("{} L/min each way", shown::TANK_PUMPED);
        still.push(kit.text(
            stage,
            &pumped,
            24.0,
            Palette::dim(),
            v((TANK_X[0] + TANK_X[1]) / 2.0, TANK_Y - TANK_H / 2.0 - 1.0),
        )?);
        // the salt in each, in grams
        let follow = stage.add_value_tracker(1.0);
        let grams = [
            stage.add_value_tracker(shown::TANK_START[0]),
            stage.add_value_tracker(shown::TANK_START[1]),
        ];
        // the readouts are live: the kit puts them on the scene as it builds them
        for (i, &cx) in TANK_X.iter().enumerate() {
            let g = grams[i];
            let y = TANK_Y - TANK_H / 2.0 - 0.45;
            kit.live_value(
                stage,
                move |s| s.tracker_value(g).unwrap_or(0.0),
                0,
                false,
                40.0,
                substance(),
                v(cx + 0.3, y),
                RIGHT,
            )?;
            still.push(kit.text(stage, "g", 40.0, substance(), v(cx + 0.5, y))?);
        }
        let anchor = stage.arena_mut().add(VMobject::new());
        let waters = [water[0], water[1]];
        let mut last = (f64::NAN, f64::NAN);
        stage.add_updater(
            anchor,
            move |s, _| {
                if s.tracker_value(follow).unwrap_or(0.0) > 0.5 {
                    let g = readings(s.tracker_value(minutes).unwrap_or(0.0));
                    let _ = s.set_tracker_value(grams[0], g[0]);
                    let _ = s.set_tracker_value(grams[1], g[1]);
                }
                let (a, b) = (
                    s.tracker_value(grams[0]).unwrap_or(0.0),
                    s.tracker_value(grams[1]).unwrap_or(0.0),
                );
                if (a.to_bits(), b.to_bits()) == (last.0.to_bits(), last.1.to_bits()) {
                    return;
                }
                s.set_fill(waters[0], None, Some(water_level(a)), None, false);
                s.set_fill(waters[1], None, Some(water_level(b)), None, false);
                last = (a, b);
            },
            true,
        )?;
        Ok(Self {
            still,
            water: waters,
            grams,
            follow,
            anchor,
        })
    }

    /// What fades in: the water, then everything drawn once. (The readouts
    /// are live and put themselves on the scene when they are built.)
    fn picture(&self) -> Vec<Mob> {
        let mut out = vec![self.water[0], self.water[1]];
        out.extend(&self.still);
        out
    }
}

// -------------------------------------------------------------- the scene

pub struct TwoTanks {
    pub kit: Kit,
}

impl SceneConstruct for TwoTanks {
    fn name(&self) -> &str {
        "10_two_tanks"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (ink, teal, gold) = (Palette::ink(), rule(), result());
        let a = tank_rule();
        let start = shown::TANK_START;
        let ten = shown::TANK_STEP_TIME;

        // the clock, in minutes: the tanks and the dot on the plane read it
        let clock = stage.add_value_tracker(0.0);

        // the plane of grams, with the rule's own arrow on the one dot
        let spec = FlowSpec {
            plane: GRAMS,
            starts: vec![start],
            dot_radius: 0.1,
            dot_color: gold,
            dot_level: 0.0,
            velocity: Some(Velocity {
                rule: map2(&a),
                scale: GAIN,
                color: teal,
                width: 4.0,
                head: 0.16,
            }),
            trails: vec![0],
            trail_color: gold,
            trail_width: 4.5,
        };
        let flow = Flow::build(stage, spec)?;
        let map_a = {
            let a = a.clone();
            move |t: f64| map2(&expm_t(&a, t))
        };
        flow.drive(stage, clock, map_a)?;
        let [dot_level, arrow_level, trail_level] = flow.levels;

        // ---- 10_1: two tanks, a hundred grams in the left one
        let line = say(&kit, stage, "10_1")?;
        let tanks_picture = Tanks::build(&kit, stage, clock)?;
        stage.add_to_scene(tanks_picture.anchor)?;
        let [grams_left, grams_right] = tanks_picture.grams;
        fade_in_all(stage, &tanks_picture.picture(), 0.8)?;
        line.wait_for(stage, "Stir a hundred grams")?;
        kit.hold(stage)?;

        // ---- 10_2: the rule, four numbers, one matrix; the plane
        let line = say(&kit, stage, "10_2")?;
        let eq_words = [
            r"\text{rate of left}=-0.1\times\text{left}+0.1\times\text{right}",
            r"\text{rate of right}=0.1\times\text{left}-0.1\times\text{right}",
        ];
        let mut words = Vec::new();
        for (i, src) in eq_words.iter().enumerate() {
            let m = kit.tex(stage, src, 34.0, ink, &[], v(-3.75, -0.45 - 0.75 * i as f64))?;
            fit_width(stage, m, 6.0);
            words.push(m);
        }
        fade_in_all(stage, &words, 0.7)?;
        line.wait_for(stage, "That is two equations")?;
        let grid_rows = entries("F-T.rule", Some(1))?;
        let grid = MatrixGrid::build(&kit, stage, &grid_rows, 56.0, teal, v(-4.2, -2.75))?;
        let grid_name = kit.tex(stage, "A=", 56.0, teal, &[], v(-6.2, -2.75))?;
        fade_in_all(stage, &[grid.group, grid_name], 0.6)?;
        // the plane: axes, the line of equal salt, the arrows of the rule
        let across = polyline(vec![GRAMS.p(0.0, 0.0), GRAMS.p(AXIS_G, 0.0)])
            .style(faded_line(Palette::dim(), 2.5, 1.0))
            .build();
        let up = polyline(vec![GRAMS.p(0.0, 0.0), GRAMS.p(0.0, AXIS_G)])
            .style(faded_line(Palette::dim(), 2.5, 1.0))
            .build();
        let balance = polyline(vec![GRAMS.p(0.0, 0.0), GRAMS.p(start[0], start[0])])
            .style(faded_line(Palette::faint(), 3.0, 1.0))
            .build();
        let (across, up, balance) = (
            stage.arena_mut().add(across),
            stage.arena_mut().add(up),
            stage.arena_mut().add(balance),
        );
        let x_name = kit.text(stage, "grams left", 26.0, Palette::dim(), GRAMS.p(54.0, -11.0))?;
        let y_name = kit.text(stage, "grams right", 26.0, Palette::dim(), GRAMS.p(8.0, AXIS_G + 9.0))?;
        let even = kit.text(stage, "equal salt", 24.0, Palette::dim(), GRAMS.p(84.0, 99.0))?;
        // five by five arrows of the rule, long enough to read
        let mut field = Vec::new();
        for i in 0..5 {
            for j in 0..5 {
                let p = [10.0 + 20.0 * f64::from(i), 10.0 + 20.0 * f64::from(j)];
                let w = a.apply(&p);
                let tip = [p[0] + GAIN * w[0], p[1] + GAIN * w[1]];
                let m = polyline(arrow_points(GRAMS.p(p[0], p[1]), GRAMS.p(tip[0], tip[1]), 0.12))
                    .style(faded_line(teal, 2.5, 0.85))
                    .build();
                field.push(stage.arena_mut().add(m));
            }
        }
        let mut plane_in = vec![across, up, balance, x_name, y_name, even];
        plane_in.extend(&field);
        stage.add_to_scene(flow.group)?;
        let mut enter: Vec<Box<dyn Animation>> = vec![
            stage.prepare(tween(dot_level, 1.0, 0.7)?)?,
            stage.prepare(tween(arrow_level, 1.0, 0.7)?)?,
            stage.prepare(tween(trail_level, 1.0, 0.7)?)?,
        ];
        for &m in &plane_in {
            enter.push(fade_in_anim(stage, m, 0.7)?);
        }
        stage.play_prepared(enter)?;

        // ---- 10_3: one big step of ten minutes goes clean through the line
        let line = say(&kit, stage, "10_3")?;
        let one = steps_path(1, ten);
        let jump = polyline(vec![GRAMS.p(one[0][0], one[0][1]), GRAMS.p(one[1][0], one[1][1])])
            .style(faded_line(teal, 5.0, 1.0))
            .build();
        let jump = stage.arena_mut().add(jump);
        line.wait_for(stage, "Take one big step")?;
        stage.set_tracker_value(tanks_picture.follow, 0.0)?;
        play!(stage;
            show_creation(jump).rt(0.9),
            tween(grams_left, one[1][0], 0.9)?,
            tween(grams_right, one[1][1], 0.9)?,
        );
        line.wait_for(stage, "That is absurd")?;
        let cross_at = GRAMS.p(one[1][0], one[1][1]);
        let r = 0.28;
        let strokes = [
            (v(cross_at[0] - r, cross_at[1] - r), v(cross_at[0] + r, cross_at[1] + r)),
            (v(cross_at[0] - r, cross_at[1] + r), v(cross_at[0] + r, cross_at[1] - r)),
        ];
        let mut cross = Vec::new();
        for (p, q) in strokes {
            let m = polyline(vec![p, q]).style(faded_line(warn(), 6.0, 1.0)).build();
            cross.push(stage.arena_mut().add(m));
        }
        fade_in_all(stage, &cross, 0.4)?;
        kit.hold(stage)?;

        // ---- 10_4: ten steps, a thousand, and e^{10A}
        let line = say(&kit, stage, "10_4")?;
        let mut back: Vec<Box<dyn Animation>> = vec![
            stage.prepare(tween(grams_left, start[0], 0.6)?)?,
            stage.prepare(tween(grams_right, start[1], 0.6)?)?,
        ];
        let mut leaving = vec![jump, grid.group, grid_name];
        leaving.extend(&cross);
        leaving.extend(&words);
        for &m in &leaving {
            back.push(fade_out_anim(stage, m, 0.5)?);
        }
        stage.play_prepared(back)?;
        stage.set_tracker_value(tanks_picture.follow, 1.0)?;
        let n10 = shown::TANK_STEPS[3];
        let n1000 = shown::TANK_STEPS[5];
        let ten_path = polyline(
            steps_path(n10, ten)
                .iter()
                .map(|p| GRAMS.p(p[0], p[1]))
                .collect::<Vec<_>>(),
        )
        .style(faded_line(teal, 4.0, 1.0))
        .build();
        let ten_path = stage.arena_mut().add(ten_path);
        line.wait_for(stage, "Ten steps do better")?;
        let row_ten = kit.text_left(
            stage,
            &format!(
                "{n10} steps: {} g, off by {} g",
                fact("F-V.10_4.ten_steps")?,
                fact("F-T.off_by.n10")?
            ),
            30.0,
            teal,
            v(TABLE_X, -0.7),
        )?;
        play!(stage; show_creation(ten_path).rt(1.4), fade_in(stage.arena_mut(), row_ten, ORIGIN, 1.0)?.rt(0.6));
        line.wait_for(stage, "A thousand, better still")?;
        let row_thousand = kit.text_left(
            stage,
            &format!(
                "{} steps: {} g, off by {} g",
                thousands(n1000),
                fact("F-V.10_4.thousand_steps")?,
                fact("F-T.off_by.n1000")?
            ),
            30.0,
            teal,
            v(TABLE_X, -1.4),
        )?;
        fade_in_all(stage, &[row_thousand], 0.6)?;
        line.wait_for(stage, "It's the bank's trick again")?;
        play!(stage; tween_linear(clock, ten, 2.4)?);
        let exact_tex = format!(r"e^{{{ten}A}}:\ {}\text{{ g}}", fact("F-V.10_4.exact")?);
        let row_exact = kit.tex_left(stage, &exact_tex, 40.0, gold, &[("A", teal)], v(TABLE_X, -2.2))?;
        fade_in_all(stage, &[row_exact], 0.6)?;

        // ---- 10_5: your turn: where does the salt end up?
        let line = say(&kit, stage, "10_5")?;
        let card = QuizCard::build(&kit, stage, "10_5", CARD_AT, CARD_SIZE)?;
        let (middle, room, _) = card.content();
        let problem = kit.text(stage, "after a long time?", 40.0, ink, v(middle[0], middle[1] + 0.25))?;
        fit_width(stage, problem, room);
        // the plane steps back while the card has its side of the frame
        let mut hide: Vec<Box<dyn Animation>> = vec![
            stage.prepare(tween(dot_level, 0.0, 0.4)?)?,
            stage.prepare(tween(arrow_level, 0.0, 0.4)?)?,
            stage.prepare(tween(trail_level, 0.0, 0.4)?)?,
        ];
        for &m in plane_in.iter().chain(&[ten_path]) {
            hide.push(fade_out_anim(stage, m, 0.4)?);
        }
        stage.play_prepared(hide)?;
        line.wait_last(stage)?;
        card.show(stage, &[problem], 0.5)?;
        card.count(&kit, stage)?;
        let settled = numbers("F-Q.10_5.answer")?;
        let answer_words = format!("{} g and {} g", whole(settled[0]), whole(settled[1]));
        let answer = kit.text(stage, &answer_words, 44.0, gold, v(middle[0], middle[1] - 0.5))?;
        card.land(stage, &[answer], 0.4)?;

        // ---- 10_6: fifty each, and how fast
        let line = say(&kit, stage, "10_6")?;
        let mut leave = card.mobs();
        leave.extend([problem, answer]);
        let mut back_in: Vec<Box<dyn Animation>> = vec![
            stage.prepare(tween(dot_level, 1.0, 0.5)?)?,
            stage.prepare(tween(arrow_level, 1.0, 0.5)?)?,
            stage.prepare(tween(trail_level, 1.0, 0.5)?)?,
        ];
        for &m in &plane_in {
            back_in.push(fade_in_anim(stage, m, 0.5)?);
        }
        for &m in &leave {
            back_in.push(fade_out_anim(stage, m, 0.5)?);
        }
        stage.play_prepared(back_in)?;
        line.wait_for(stage, "And the exponential says how fast")?;
        let minutes: f64 = fact("F-V.10_6.minutes")?
            .parse()
            .map_err(|e| fail(format!("fact F-V.10_6.minutes: {e}")))?;
        play!(stage; tween_linear(clock, minutes, 2.6)?);
        let marker = chip(
            &kit,
            stage,
            &format!("{} min: within 1 g", fact("F-V.10_6.minutes")?),
            28.0,
            gold,
            v(GRAMS.centre[0] + 1.9, GRAMS.centre[1] - 0.9),
        )?;
        fade_in_all(stage, &[marker], 0.5)?;
        line.wait_for(stage, "Each tank is within a gram")?;
        play!(stage; tween_linear(clock, LONG_RUN_MIN, 1.8)?);
        let total = kit.text_left(
            stage,
            &format!("total: {} g, always", whole_fact("F-T.total")?),
            30.0,
            Palette::dim(),
            v(TABLE_X, -3.3),
        )?;
        fade_in_all(stage, &[total], 0.5)?;
        music_hold(&kit, stage)?;
        kit.hold(stage)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::{fact, numbers, testing};
    use super::*;

    fn close(a: [f64; 2], b: &[f64], tol: f64) -> bool {
        (a[0] - b[0]).abs() < tol && (a[1] - b[1]).abs() < tol
    }

    /// 10_1 and 10_6: the readings are the facts' at every time they are
    /// quoted, and the total is a hundred grams always (an always sentence,
    /// tested from the start to far past where the readings stop moving).
    #[test]
    fn the_readings_are_the_facts_and_the_total_never_changes() {
        for t in shown::TANK_TIMES {
            let want = numbers(&format!("F-T.at.t{}", t as u32)).unwrap();
            assert!(close(readings(t), &want, 1e-3), "t = {t}");
        }
        // from the start to a thousand minutes (the video runs to LONG_RUN_MIN)
        for t in [0.0, 1e-9, 0.5, 7.0, 19.56, LONG_RUN_MIN, 1000.0] {
            let [l, r] = readings(t);
            assert!((l + r - 100.0).abs() < 1e-9, "t = {t}: {l} + {r}");
            assert!(
                l >= 50.0 - 1e-9 && (-1e-9..=50.0 + 1e-9).contains(&r),
                "t = {t}: {l}, {r}"
            );
        }
        assert_eq!(fact("F-T.total").unwrap(), "100.000000");
        assert_eq!(whole_fact("F-T.total").unwrap(), "100");
    }

    /// 10_3 and 10_4: one step throws all the salt to the other tank, ten
    /// steps and a thousand end where the facts say, and each is closer to
    /// the exponential than the last.
    #[test]
    fn the_steps_end_where_the_facts_say_and_close_in() {
        let one = *steps_path(1, 10.0).last().unwrap();
        assert!(close(one, &numbers("F-V.10_3.one_step").unwrap(), 1e-9), "{one:?}");
        let exact = readings(10.0);
        let mut off = Vec::new();
        for n in [1_u32, 2, 5, 10, 100, 1000] {
            let end = *steps_path(n, 10.0).last().unwrap();
            let want = numbers(&format!("F-T.steps.n{n}")).unwrap();
            assert!(close(end, &want, 1e-3), "n = {n}");
            off.push(exact[0] - end[0]);
        }
        assert!(off.windows(2).all(|w| w[1].abs() < w[0].abs()), "{off:?}");
        // the table's numbers, as the screen prints them
        let left = |n| steps_path(n, 10.0).last().unwrap()[0];
        assert_eq!(format!("{:.2}", left(10)), fact("F-V.10_4.ten_steps").unwrap());
        assert_eq!(format!("{:.2}", left(1000)), fact("F-V.10_4.thousand_steps").unwrap());
        assert_eq!(format!("{:.2}", exact[0]), fact("F-V.10_4.exact").unwrap());
        assert_eq!(format!("{:.2}", off[3]), fact("F-T.off_by.n10").unwrap());
        assert_eq!(format!("{:.3}", off[5]), fact("F-T.off_by.n1000").unwrap());
        // the one big step crosses the balance line (left ends under right)
        assert!(one[0] < one[1], "the step overshoots the line of equal salt");
        // the closed form of n steps: 50 (1 + (1 − 2/n)^n) in the left tank
        for n in [1_u32, 10, 1000] {
            let formula = 50.0 * (1.0 + (1.0 - 2.0 / f64::from(n)).powi(n as i32));
            assert!((left(n) - formula).abs() < 1e-9, "n = {n}");
        }
    }

    /// 10_5 and 10_6: the salt ends at fifty and fifty, and the left tank is
    /// within a gram of fifty from the minute the narration says.
    #[test]
    fn it_settles_at_fifty_each_and_within_a_gram_after_the_minutes_said() {
        assert_eq!(fact("F-Q.10_5.answer").unwrap(), "50,50");
        let far = readings(LONG_RUN_MIN);
        assert!(close(far, &[50.0, 50.0], 1e-3), "{far:?}");
        let minutes: f64 = fact("F-V.10_6.minutes").unwrap().parse().unwrap();
        assert!((readings(minutes + 0.05)[0] - 50.0).abs() < 1.0);
        assert!((readings(minutes - 0.05)[0] - 50.0).abs() > 1.0);
        assert_eq!(fact("F-V.10_6.minutes.said").unwrap(), "20");
        // the water's colour rises with the salt, and is never solid
        assert!(water_level(0.0) < water_level(50.0) && water_level(50.0) < water_level(100.0));
        assert!(water_level(100.0) <= 0.9 && water_level(0.0) >= 0.1);
    }

    /// The rule's arrows all point along one diagonal, toward the line of
    /// equal salt, and have no length on it.
    #[test]
    fn the_rules_arrows_point_along_one_diagonal() {
        let a = tank_rule();
        for i in 0..5 {
            for j in 0..5 {
                let p = [10.0 + 20.0 * f64::from(i), 10.0 + 20.0 * f64::from(j)];
                let w = a.apply(&p);
                assert!((w[0] + w[1]).abs() < 1e-12, "the two tanks' rates are opposite");
                assert!((w[0] - 0.1 * (p[1] - p[0])).abs() < 1e-12);
                if p[0] == p[1] {
                    assert!(w[0].abs() < 1e-12, "no push on the line of equal salt");
                }
            }
        }
    }

    /// The chapter runs on the storyboard with the layout lint clean, types
    /// the strings it should, and stays inside the chapter budget.
    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("10_two_tanks").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            "10 L/min each way",
            "10 steps: 55.37 g, off by 1.40 g",
            "1,000 steps: 56.75 g, off by 0.014 g",
            "e^{10A}:\\ 56.77\\text{ g}",
            "50 g and 50 g",
            "19.6 min: within 1 g",
            "total: 100 g, always",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never typesets {want:?}: {words:?}"
            );
        }
        assert!(ran.seconds > 45.0 && ran.seconds < 85.0, "{} s", ran.seconds);
    }
}
