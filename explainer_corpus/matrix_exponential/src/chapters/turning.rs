//! Chapter 06, turning: the number i as a quarter turn, twice making minus
//! one (06_1); a rate of i times your position is a push at right angles to
//! where you stand, as long as your distance from the centre (06_2); the
//! quiz, what path tiny pushes trace (06_3); a circle: four big steps drift
//! outward, a hundred hug it, and in the limit you stay on it moving at speed
//! one, so a time t is an arc of length t, an angle of t radians (06_4); and
//! that point is e to the i t, which at a time pi lands on minus one (06_5).
//!
//! The plane is the number plane, drawn with `PLANE`. Every number is a
//! fact: where four and a hundred steps end are `F-V.06_4.*`, where a time
//! pi lands `F-N.turn.pi`, Euler's year `F-B.euler.year`, the answer of the
//! quiz `F-Q.06_3.answer`.

use std::f64::consts::{FRAC_PI_2, PI};

use explainer_kit::props::lower_third;
use explainer_kit::{Kit, Palette, fade_all, play, tween, v};
use fmn::prelude::*;

use super::draw::{
    Plane, QuizCard, arrow, arrow_points, fact, fade_in_all, fade_out_all, faded_line, flat, minus, music_hold, result,
    rule, say, turning, tween_linear, warn,
};

pub struct Turning {
    pub kit: Kit,
}

/// The number plane: where its origin is on the screen and how big a unit is.
const PLANE: Plane = Plane {
    centre: [-2.2, -0.3, 0.0],
    unit: 1.4,
};
/// The right panel's centre.
const PANEL_X: f64 = 3.9;
/// The quiz card.
const CARD_AT: Vec3 = [3.9, -0.2, 0.0];
const CARD_SIZE: (f64, f64) = (5.4, 4.8);
/// Steps in the two paths of 06_4.
const BIG_STEPS: u32 = 4;
const MANY_STEPS: u32 = 100;
/// How many samples the highlighted arc is drawn from.
const ARC_SAMPLES: usize = 64;

// the chapter's fixed words (numbers are built from facts)
const TWO_TURNS: &str = "two quarter turns";
const THE_RULE: &str = "rate = i \u{d7} position";
const ONE_STEP_WORDS: &str = "one small push sideways";
const FOLLOW: &str = "follow the rule";
const IN_TINY_STEPS: &str = "in tiny steps";
const SPEED_ONE: &str = "speed 1";
const ARC_T: &str = "arc of length t";
const ONE_RADIAN: &str = "1 radian";
const ARC_AS_RADIUS: &str = "an arc as long as the radius";
const WHO_AND_WHEN: &str = "who and when:";
const POINT_IS: &str = "e^{it}";

/// `steps` straight steps of the rule "a push at right angles" over a time
/// `t`, from the point 1: the points, first to last.
pub fn euler_path(steps: u32, t: f64) -> Vec<[f64; 2]> {
    let h = t / f64::from(steps);
    let mut p = [1.0, 0.0];
    let mut out = vec![p];
    for _ in 0..steps {
        p = [p[0] - h * p[1], p[1] + h * p[0]];
        out.push(p);
    }
    out
}

/// The arc of the unit circle from angle `a0` to `a1` on `PLANE`, in
/// `samples` + 1 points (at least two distinct ones).
fn arc(a0: f64, a1: f64, radius: f64, samples: usize) -> Vec<Vec3> {
    let mut pts: Vec<Vec3> = (0..=samples)
        .map(|k| {
            let a = a0 + (a1 - a0) * k as f64 / samples as f64;
            PLANE.p(radius * a.cos(), radius * a.sin())
        })
        .collect();
    if (a1 - a0).abs() < 1e-3 {
        pts.truncate(1);
        let at = pts[0];
        pts.push(v(at[0] + 1e-4, at[1]));
    }
    pts
}

/// A curved turn from angle `a0` to `a1` at `radius`, as one polyline with
/// its head at the end.
fn turn_arrow(a0: f64, a1: f64, radius: f64) -> Vec<Vec3> {
    let mut pts = arc(a0, a1, radius, 24);
    let tip = pts.pop().unwrap_or([0.0; 3]);
    let from = pts.last().copied().unwrap_or(tip);
    pts.extend(arrow_points(from, tip, 0.2));
    pts
}

impl SceneConstruct for Turning {
    fn name(&self) -> &str {
        "06_turning"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let ink = Palette::ink();
        let origin = PLANE.p(0.0, 0.0);
        let axes = PLANE.axes(stage, 3.0, 2.2);
        let mk_line = |stage: &mut Stage<'_>, pts: Vec<Vec3>, color: Srgb, width: f64, o: f64| {
            stage
                .arena_mut()
                .add(polyline(pts).style(faded_line(color, width, o)).build())
        };

        // 06_1: feed it i: a quarter turn; twice, minus one
        let line = say(&kit, stage, "06_1")?;
        let one_arrow = arrow(stage, origin, PLANE.p(1.0, 0.0), ink, 5.0)?;
        let i_arrow = arrow(stage, origin, PLANE.p(0.0, 1.0), turning(), 5.0)?;
        let minus_one = arrow(stage, origin, PLANE.p(-1.0, 0.0), turning(), 5.0)?;
        let turn_one = mk_line(stage, turn_arrow(0.1, FRAC_PI_2 - 0.1, 0.62), turning(), 4.0, 1.0);
        let turn_two = mk_line(stage, turn_arrow(FRAC_PI_2 + 0.1, PI - 0.1, 0.62), turning(), 4.0, 1.0);
        let times_a = kit.tex(stage, "\\times\\, i", 50.0, turning(), &[], PLANE.p(0.78, 0.78))?;
        let times_b = kit.tex(stage, "\\times\\, i", 50.0, turning(), &[], PLANE.p(-0.78, 0.78))?;
        let names = [
            kit.tex(stage, "1", 48.0, ink, &[], PLANE.p(1.18, -0.22))?,
            kit.tex(stage, "i", 48.0, turning(), &[], PLANE.p(0.26, 1.15))?,
            kit.tex(stage, "-1", 48.0, turning(), &[], PLANE.p(-1.25, -0.25))?,
        ];
        let product = kit.tex(stage, "i \\times i = -1", 90.0, ink, &[], v(PANEL_X, 1.3))?;
        let two_turns = kit.text(stage, TWO_TURNS, 34.0, Palette::dim(), v(PANEL_X, 0.1))?;
        let mut first: Vec<Mob> = axes.clone();
        first.extend([one_arrow, names[0]]);
        fade_in_all(stage, &first, 0.6)?;
        line.wait_for(stage, "You may know it")?;
        fade_in_all(stage, &[names[1]], 0.4)?;
        line.wait_for(stage, "multiplying by i")?;
        fade_in_all(stage, &[i_arrow, turn_one, times_a], 0.6)?;
        line.wait_for(stage, "Turn a quarter turn")?;
        fade_in_all(stage, &[minus_one, turn_two, times_b, names[2]], 0.6)?;
        kit.hold(stage)?;
        fade_in_all(stage, &[product, two_turns], 0.6)?;
        kit.hold(stage)?;

        // 06_2: a rate of i times your position: a push at right angles, as
        // long as your distance from the centre
        let line = say(&kit, stage, "06_2")?;
        let mut leaving = vec![
            one_arrow, i_arrow, minus_one, turn_one, turn_two, times_a, times_b, product, two_turns,
        ];
        leaving.extend(names);
        fade_out_all(stage, &leaving, 0.5)?;
        let phi = stage.add_value_tracker(0.0);
        let rho = stage.add_value_tracker(1.0);
        let position = mk_line(stage, arrow_points(origin, PLANE.p(1.0, 0.0), 0.2), ink, 3.0, 1.0);
        let push = mk_line(
            stage,
            arrow_points(PLANE.p(1.0, 0.0), PLANE.p(1.0, 1.0), 0.2),
            rule(),
            6.0,
            1.0,
        );
        let dot_template: Vec<Vec3> = Dot::new().radius(0.11).build().points().to_vec();
        let you = stage.arena_mut().add(
            Dot::new()
                .point(PLANE.p(1.0, 0.0))
                .radius(0.11)
                .style(flat(result(), 1.0))
                .build(),
        );
        let anchor = stage.arena_mut().add(VMobject::new());
        let mut last = (f64::NAN, f64::NAN);
        stage.add_updater(
            anchor,
            move |s, _| {
                let (a, r) = (s.tracker_value(phi).unwrap_or(0.0), s.tracker_value(rho).unwrap_or(1.0));
                if (a, r) == last {
                    return;
                }
                let p = (r * a.cos(), r * a.sin());
                // the rule: i times the position is the position turned a
                // quarter and left as long
                let q = (p.0 - r * a.sin(), p.1 + r * a.cos());
                let (at, to) = (PLANE.p(p.0, p.1), PLANE.p(q.0, q.1));
                let _ = s.set_points(position, &arrow_points(origin, at, 0.2));
                let _ = s.set_points(push, &arrow_points(at, to, 0.2));
                let moved: Vec<Vec3> = dot_template
                    .iter()
                    .map(|d| [d[0] + at[0], d[1] + at[1], d[2]])
                    .collect();
                let _ = s.set_points(you, &moved);
                last = (a, r);
            },
            true,
        )?;
        stage.add_to_scene(anchor)?;
        let rate_rule = kit.tex(stage, "y' = i\\, y", 90.0, rule(), &[], v(PANEL_X, 1.4))?;
        let rule_words = kit.text(stage, THE_RULE, 34.0, Palette::dim(), v(PANEL_X, 0.2))?;
        line.wait_for(stage, "So let your rate be")?;
        fade_in_all(stage, &[position, push, you, rate_rule, rule_words], 0.7)?;
        line.wait_for(stage, "That's a push sideways")?;
        play!(stage;
            tween(phi, 2.6, 3.6)?,
            tween(rho, 1.6, 3.6)?,
        );
        line.wait_for(stage, "as long as your distance")?;
        play!(stage; tween(rho, 0.8, 1.6)?);
        kit.hold(stage)?;

        // 06_3: your turn: follow that rule in tiny steps: what path?
        let line = say(&kit, stage, "06_3")?;
        fade_out_all(stage, &[position, push, you, rate_rule, rule_words], 0.4)?;
        // one step drawn, the rest to come
        let start_dot = stage.arena_mut().add(
            Dot::new()
                .point(PLANE.p(1.0, 0.0))
                .radius(0.11)
                .style(flat(result(), 1.0))
                .build(),
        );
        let one_step = mk_line(
            stage,
            arrow_points(PLANE.p(1.0, 0.0), PLANE.p(1.0, 0.35), 0.12),
            rule(),
            6.0,
            1.0,
        );
        let card = QuizCard::build(&kit, stage, "06_3", CARD_AT, CARD_SIZE)?;
        let (centre, _, _) = card.content();
        let lines = [
            kit.text(stage, FOLLOW, 34.0, ink, v(centre[0], centre[1] + 0.8))?,
            kit.text(stage, IN_TINY_STEPS, 34.0, ink, v(centre[0], centre[1] + 0.2))?,
            kit.text(stage, ONE_STEP_WORDS, 28.0, rule(), v(centre[0], centre[1] - 0.45))?,
        ];
        let mut ins = vec![start_dot, one_step];
        ins.extend(lines);
        card.show_frame(stage, &ins, 0.6)?;
        line.wait_last(stage)?;
        card.arm(stage)?;
        card.count(&kit, stage)?;

        // 06_4: a circle: four big steps drift outward, a hundred hug it,
        // in the limit you stay on it at speed one: arc length t, t radians
        let line = say(&kit, stage, "06_4")?;
        let answer = kit.text(
            stage,
            fact("F-Q.06_3.answer")?,
            30.0,
            result(),
            v(CARD_AT[0], card.ring_at()[1]),
        )?;
        card.land(stage, &[answer], 0.3)?;
        let ring = mk_line(stage, arc(0.0, std::f64::consts::TAU, 1.0, 96), result(), 5.0, 1.0);
        let four = euler_path(BIG_STEPS, PI);
        let many = euler_path(MANY_STEPS, PI);
        let to_screen = |path: &[[f64; 2]]| -> Vec<Vec3> { path.iter().map(|p| PLANE.p(p[0], p[1])).collect() };
        let four_line = mk_line(stage, to_screen(&four), rule(), 4.0, 1.0);
        let many_line = mk_line(stage, to_screen(&many), rule(), 3.0, 1.0);
        // where they end, as the facts print it: "-2.32,1.20" as "−2.32, 1.20"
        let ends = |key: &str| -> fmn::Result<String> { Ok(minus(&fact(key)?.replace(',', ", "))) };
        let four_words = kit.text(
            stage,
            &format!("{BIG_STEPS} steps end at {}", ends("F-V.06_4.four_steps")?),
            30.0,
            warn(),
            v(PANEL_X, 1.9),
        )?;
        let many_words = kit.text(
            stage,
            &format!("{MANY_STEPS} steps end at {}", ends("F-V.06_4.hundred_steps")?),
            30.0,
            rule(),
            v(PANEL_X, 1.1),
        )?;
        let mut leaving = card.mobs();
        leaving.extend(lines);
        leaving.push(answer);
        leaving.extend([start_dot, one_step]);
        fade_out_all(stage, &leaving, 0.5)?;
        fade_in_all(stage, &[ring], 0.6)?;
        line.wait_for(stage, "Four big steps drift outward")?;
        fade_in_all(stage, &[four_line, four_words], 0.7)?;
        line.wait_for(stage, "A hundred hug it")?;
        fade_in_all(stage, &[many_line, many_words], 0.7)?;
        line.wait_for(stage, "Shrink the steps to nothing")?;
        fade_out_all(stage, &[four_line, many_line, four_words, many_words], 0.5)?;
        // the point at arc length s, the arc behind it, and the radius to it
        let s = stage.add_value_tracker(0.0);
        let arc_mob = mk_line(stage, arc(0.0, 1e-4, 1.0, ARC_SAMPLES), result(), 9.0, 1.0);
        let spoke = mk_line(stage, vec![origin, PLANE.p(1.0, 0.0)], Palette::dim(), 3.0, 1.0);
        let point = stage.arena_mut().add(
            Dot::new()
                .point(PLANE.p(1.0, 0.0))
                .radius(0.13)
                .style(flat(result(), 1.0))
                .build(),
        );
        let point_template: Vec<Vec3> = Dot::new().radius(0.13).build().points().to_vec();
        let mover = stage.arena_mut().add(VMobject::new());
        let mut shown = f64::NAN;
        stage.add_updater(
            mover,
            move |st, _| {
                let t = st.tracker_value(s).unwrap_or(0.0);
                if t.to_bits() == shown.to_bits() {
                    return;
                }
                let _ = st.set_points(arc_mob, &arc(0.0, t, 1.0, ARC_SAMPLES));
                let at = PLANE.p(t.cos(), t.sin());
                let _ = st.set_points(spoke, &[origin, at]);
                let moved: Vec<Vec3> = point_template
                    .iter()
                    .map(|d| [d[0] + at[0], d[1] + at[1], d[2]])
                    .collect();
                let _ = st.set_points(point, &moved);
                shown = t;
            },
            true,
        )?;
        stage.add_to_scene(mover)?;
        let speed = kit.text(stage, SPEED_ONE, 34.0, ink, v(PANEL_X, 1.9))?;
        let arc_words = kit.text(stage, ARC_T, 34.0, result(), v(PANEL_X, 1.1))?;
        fade_in_all(stage, &[arc_mob, spoke, point, speed], 0.5)?;
        play!(stage; tween_linear(s, 2.0, 2.6)?);
        line.wait_for(stage, "So after a time t")?;
        fade_in_all(stage, &[arc_words], 0.5)?;
        // an arc as long as the radius is one radian
        line.wait_for(stage, "an angle of t radians")?;
        let radian = mk_line(stage, arc(0.0, 1.0, 1.28, 32), turning(), 7.0, 1.0);
        let radius_end = mk_line(stage, vec![origin, PLANE.p(1.28, 0.0)], turning(), 3.0, 0.8);
        let radian_words = kit.text(stage, ONE_RADIAN, 34.0, turning(), v(PANEL_X, 0.1))?;
        let as_long = kit.text(stage, ARC_AS_RADIUS, 26.0, Palette::dim(), v(PANEL_X, -0.55))?;
        fade_in_all(stage, &[radian, radius_end, radian_words, as_long], 0.6)?;
        let who = lower_third(
            &kit,
            stage,
            WHO_AND_WHEN,
            &format!(
                "Euler, {}: e^(v\u{221a}\u{2212}1) = cos v + \u{221a}\u{2212}1 sin v",
                fact("F-B.euler.year")?
            ),
        )?;
        fade_in_all(stage, &[who], 0.5)?;
        kit.hold(stage)?;

        // 06_5: that point is e to the i t; a time pi lands on minus one
        let line = say(&kit, stage, "06_5")?;
        fade_out_all(
            stage,
            &[speed, arc_words, radian, radius_end, radian_words, as_long, who],
            0.5,
        )?;
        let name = kit.tex(stage, POINT_IS, 76.0, result(), &[], PLANE.p(0.0, 1.72))?;
        let lands = kit.tex(stage, "e^{i\\pi} = -1", 90.0, result(), &[], v(PANEL_X, 1.2))?;
        let pi_words = kit.text(
            stage,
            "a time \u{3c0}: half a circle",
            30.0,
            Palette::dim(),
            v(PANEL_X, 0.1),
        )?;
        play!(stage; tween(s, 0.0, 0.6)?);
        line.wait_for(stage, "That point is called")?;
        fade_in_all(stage, &[name], 0.5)?;
        line.wait_for(stage, "Run it for a time pi")?;
        fade_in_all(stage, &[pi_words], 0.4)?;
        play!(stage; tween_linear(s, PI, 3.0)?);
        line.wait_for(stage, "you land on minus one")?;
        fade_in_all(stage, &[lands], 0.6)?;
        music_hold(&kit, stage)?;

        kit.hold(stage)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::{numbers, show, testing};
    use super::*;
    use crate::model::{quarter_turn, tiny_steps};

    /// 06_4: the four-step and hundred-step paths end where the facts say
    /// the voice's "drift outward" and "hug it" end, and they are the model's
    /// tiny steps of the quarter turn over a time pi.
    #[test]
    fn four_steps_drift_outward_and_a_hundred_hug_the_circle() {
        let j = quarter_turn();
        for (steps, key) in [
            (BIG_STEPS, "F-V.06_4.four_steps"),
            (MANY_STEPS, "F-V.06_4.hundred_steps"),
        ] {
            let path = euler_path(steps, PI);
            let end = path[path.len() - 1];
            let model = tiny_steps(&j, PI, steps).column(0);
            assert!(
                (end[0] - model[0]).abs() < 1e-12 && (end[1] - model[1]).abs() < 1e-12,
                "{steps}"
            );
            let said = numbers(key).unwrap();
            assert_eq!(show(end[0], 2).unwrap(), format!("{:.2}", said[0]), "{key}");
            assert_eq!(show(end[1], 2).unwrap(), format!("{:.2}", said[1]), "{key}");
            // every step leaves the circle a little further out: the radius
            // after k steps is (1 + h^2)^(k/2)
            let h = PI / f64::from(steps);
            for (k, p) in path.iter().enumerate() {
                let want = (1.0 + h * h).powf(k as f64 / 2.0);
                assert!((p[0].hypot(p[1]) - want).abs() < 1e-9, "{steps}: step {k}");
            }
        }
        // four steps end well off the circle, a hundred near it
        let (far, near) = (euler_path(BIG_STEPS, PI), euler_path(MANY_STEPS, PI));
        let radius = |p: &[f64; 2]| p[0].hypot(p[1]);
        assert!(radius(&far[BIG_STEPS as usize]) > 2.5);
        assert!(radius(&near[MANY_STEPS as usize]) < 1.06);
    }

    /// 06_2: the push is at right angles to where you stand and exactly as
    /// long as your distance from the centre, wherever you are.
    #[test]
    fn the_push_is_at_right_angles_and_as_long_as_the_distance() {
        for a in [0.0_f64, 0.7, 2.6, 4.1] {
            for r in [0.3_f64, 0.8, 1.0, 1.6] {
                let p = (r * a.cos(), r * a.sin());
                let q = (-r * a.sin(), r * a.cos());
                assert!((p.0 * q.0 + p.1 * q.1).abs() < 1e-12, "right angles");
                assert!((q.0.hypot(q.1) - p.0.hypot(p.1)).abs() < 1e-12, "as long");
            }
        }
    }

    /// 06_4 and 06_5: at speed one, a time t is an arc of length t (the
    /// polyline drawn for it is that long), a radian is an arc as long as
    /// the radius, and a time pi is half a circle that lands on minus one.
    #[test]
    fn a_time_t_is_an_arc_of_length_t_and_pi_lands_on_minus_one() {
        for t in [0.5_f64, 1.0, 2.0, PI] {
            let pts = arc(0.0, t, 1.0, ARC_SAMPLES);
            let unit = PLANE.unit;
            let length: f64 = pts
                .windows(2)
                .map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]))
                .sum::<f64>()
                / unit;
            assert!((length - t).abs() < 1e-3, "t = {t}: {length}");
        }
        let landed = numbers("F-N.turn.pi").unwrap();
        assert!((landed[0] + 1.0).abs() < 1e-6 && landed[1].abs() < 1e-6, "{landed:?}");
        assert_eq!(fact("F-B.euler.year").unwrap(), "1748");
        assert_eq!(fact("F-N.turn.radian_degrees").unwrap(), "57.2958");
        // the rule's own motion: the point's speed is 1
        let h = 1e-6;
        let t = 0.9_f64;
        let v = (((t + h).cos() - (t - h).cos()) / (2.0 * h)).hypot(((t + h).sin() - (t - h).sin()) / (2.0 * h));
        assert!((v - 1.0).abs() < 1e-6);
        assert_eq!(fact("F-Q.06_3.answer").unwrap(), "a circle of radius 1.000");
    }

    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("06_turning").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            "i \\times i = -1",
            "y' = i\\, y",
            "e^{it}",
            "e^{i\\pi} = -1",
            "4 steps end at",
            "100 steps end at",
            "Euler, 1748",
            "a circle of radius 1.000",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never typesets {want:?}: {words:?}"
            );
        }
        assert!(ran.seconds > 35.0 && ran.seconds < 85.0, "{} s", ran.seconds);
    }
}
