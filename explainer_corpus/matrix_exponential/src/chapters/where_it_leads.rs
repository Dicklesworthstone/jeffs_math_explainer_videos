//! Chapter 16, where it leads: a robot arm's pose is a product of
//! exponentials, one per joint, which is the companion video (16_1); the
//! Standard Model's forces are built on three Lie groups with eight, three
//! and one independent tiny steps, twelve in all, and there are twelve
//! particles that carry them (16_2). One last time, your turn: e raised to
//! the quarter turn J (16_3). A rotation by one radian: the rule J moves
//! every point at a speed equal to its distance from the centre, so in one
//! unit of time each point travels an arc as long as its radius, and a line
//! of tiny steps wraps onto a circle of rotations (16_4). The matrix is a
//! rule for tiny steps, its exponential is where the rule takes you, and its
//! rate is the rule applied to itself (16_5).
//!
//! Every number is a fact: the arm's product of exponentials is checked
//! against its forward kinematics in the tests, the Lie dimensions are
//! `F-V.16_2.steps`, the carriers `F-B.bosons`, Lie's years `F-B.lie.*`, the
//! answer `F-Q.16_3.answer` (`F-H.J.true`).

use std::f64::consts::TAU;

use explainer_kit::props::{lower_third, names_to_search, receipts_card};
use explainer_kit::{Kit, Palette, fade_all, play, tween, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::draw::{
    Flow, FlowSpec, MatrixGrid, Plane, QuizCard, chip, entries, fade_in_all, fade_out_all, faded_line, flat, lattice,
    music_hold, numbers, panel, result, rotation_map, rule, say, turning, tween_linear, whole,
};
use super::flow::{Levels, fade_in_anim, fade_out_anim};
use super::follow_the_arrows::lay_row;
use super::the_doorway::{Cube, VIEW_TILT, VIEW_YAW};
use crate::facts::{cited, spin_axes, spiral_rule};
use crate::model::{Mat, expm_t};

// ------------------------------------------------------------------ choices

/// The arm of 16_1: where its base sits, the three link lengths (scene
/// units) and the angles of its joints at home.
const BASE: [f64; 2] = [-4.6, -2.2];
const LINKS: [f64; 3] = [1.75, 1.45, 1.05];
const HOME: [f64; 3] = [0.9, -0.5, -0.3];
/// 16_4: the circle's radius on screen, where its centre sits, and how many
/// points a ruler is drawn from.
const RADIUS: f64 = 1.85;
const CIRCLE_AT: Vec3 = [-3.2, 0.3, 0.0];
const RULER_POINTS: usize = 140;
/// How long a ruler is before it wraps, in tiny steps, and how far it is
/// let out afterwards (more than a full turn of 2π).
const RULER_SHORT: f64 = 1.7;
const RULER_LONG: f64 = 7.0;
const _: () = assert!(RULER_LONG > TAU, "the ruler must be let out past a full turn");
/// 16_5: the centres of the four panes, and their common height.
const PANE_X: [f64; 4] = [-5.0, -1.7, 1.6, 4.9];
const PANE_Y: f64 = -0.3;

// ------------------------------------------------------------- the geometry

/// The joints of the planar arm: its base, then the point after each link,
/// for joint angles `q` (each relative to the link before).
pub fn arm_points(base: [f64; 2], links: &[f64; 3], q: &[f64; 3]) -> [[f64; 2]; 4] {
    let mut out = [base; 4];
    let mut angle = 0.0;
    for k in 0..3 {
        angle += q[k];
        out[k + 1] = [out[k][0] + links[k] * angle.cos(), out[k][1] + links[k] * angle.sin()];
    }
    out
}

/// The twist of a revolute joint at `p` (a unit turn in a unit of "time"),
/// as a 3 by 3 matrix acting on (x, y, 1): the rule of turning about `p`.
pub fn twist(p: [f64; 2]) -> Mat {
    Mat::from_rows(&[&[0.0, -1.0, p[1]], &[1.0, 0.0, -p[0]], &[0.0, 0.0, 0.0]])
}

/// Every point of the arm as a product of exponentials, one per joint,
/// applied to that point's home place: joint k sits where
/// e^{[S1]q1} … e^{[Sk]qk} takes its home place, and the hand where all
/// three take M. The twists sit at the home arm's joints; `q` are the joint
/// angles measured from home.
pub fn by_exponentials(base: [f64; 2], links: &[f64; 3], home: &[f64; 3], q: &[f64; 3]) -> [[f64; 2]; 4] {
    let at_home = arm_points(base, links, home);
    let mut pose = Mat::identity(3);
    let mut out = [base; 4];
    for (k, (&p, slot)) in at_home.iter().zip(out.iter_mut()).enumerate() {
        let moved = pose.apply(&[p[0], p[1], 1.0]);
        *slot = [moved[0], moved[1]];
        if k < 3 {
            pose = pose.mul(&expm_t(&twist(p), q[k]));
        }
    }
    out
}

/// A point of the ruler `s` tiny steps from where it touches the circle,
/// when it has wrapped to curvature `kappa` (0 straight, 1 the unit
/// circle): its length is kept all along, so it never stretches. Plane
/// coordinates: the circle is the unit circle, touched at (1, 0).
pub fn wrap_point(kappa: f64, s: f64) -> [f64; 2] {
    if kappa < 1e-9 {
        return [1.0, s];
    }
    let r = 1.0 / kappa;
    [1.0 - r + r * (kappa * s).cos(), r * (kappa * s).sin()]
}

/// The ruler's points from 0 to `reach` tiny steps, on screen.
pub fn ruler_points(kappa: f64, reach: f64, n: usize) -> Vec<Vec3> {
    (0..=n)
        .map(|k| {
            let p = wrap_point(kappa, reach * k as f64 / n as f64);
            v(CIRCLE_AT[0] + RADIUS * p[0], CIRCLE_AT[1] + RADIUS * p[1])
        })
        .collect()
}

/// A short stroke across the ruler at step `s`, `half` long each way.
pub fn tick(kappa: f64, s: f64, half: f64) -> [Vec3; 2] {
    let (a, b) = (wrap_point(kappa, s - 1e-4), wrap_point(kappa, s + 1e-4));
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len = dx.hypot(dy).max(1e-12);
    let (nx, ny) = (-dy / len, dx / len);
    let c = wrap_point(kappa, s);
    let at = |sign: f64| {
        v(
            CIRCLE_AT[0] + RADIUS * c[0] + sign * half * nx,
            CIRCLE_AT[1] + RADIUS * c[1] + sign * half * ny,
        )
    };
    [at(-1.0), at(1.0)]
}

/// The moving parts of 16_5's panes at clock `t` (0 to 2π), as places on
/// screen: the growing amount on its curve, the point going round, the point
/// carried by a matrix.
pub fn pane_places(t: f64) -> [Vec3; 3] {
    // a share: the amount e^{0.9 u} along its curve, u running 0 to 1.5
    let u = 1.5 * t / TAU;
    let growth = v(PANE_X[0] - 1.1 + 1.45 * u, PANE_Y - 0.9 + 0.5 * ((0.9 * u).exp() - 1.0));
    // i: round a circle of radius 0.9, at speed one
    let round = v(PANE_X[1] + 0.9 * t.cos(), PANE_Y + 0.9 * t.sin());
    // a matrix: the spiral rule, a time of 0.35 per unit of clock
    let p = expm_t(&spiral_rule(), 0.35 * t).apply(&[1.0, 0.0]);
    let flow = v(PANE_X[2] + 0.9 * p[0], PANE_Y + 0.9 * p[1]);
    [growth, round, flow]
}

// -------------------------------------------------------------- the scene

pub struct WhereItLeads {
    pub kit: Kit,
}

/// One tile of 16_2's twelve: a rounded card with its letter, edged in the
/// colour of the group it belongs to.
fn boson_tile(
    kit: &Kit,
    stage: &mut Stage<'_>,
    label: &str,
    at: Vec3,
    edge: Srgb,
    as_tex: bool,
) -> fmn::Result<Vec<Mob>> {
    let r = Rectangle::new()
        .width(0.95)
        .height(0.8)
        .corner_radius(0.1)
        .style(Style::default().fill(panel(), 0.96).stroke(edge, 3.0, 1.0))
        .build()?
        .moved_to(at);
    let back = stage.arena_mut().add(r);
    let text = if as_tex {
        kit.tex(stage, label, 40.0, Palette::ink(), &[], at)?
    } else {
        kit.text(stage, label, 34.0, Palette::ink(), at)?
    };
    Ok(vec![back, text])
}

impl SceneConstruct for WhereItLeads {
    fn name(&self) -> &str {
        "16_where_it_leads"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (ink, teal, gold) = (Palette::ink(), rule(), result());

        // ---- 16_1: robots use this: a product of exponentials, a joint each
        let line = say(&kit, stage, "16_1")?;
        // each tracker is a joint's angle measured from home
        let qs = [
            stage.add_value_tracker(0.0),
            stage.add_value_tracker(0.0),
            stage.add_value_tracker(0.0),
        ];
        let start = by_exponentials(BASE, &LINKS, &HOME, &[0.0; 3]);
        let mut arm_links = Vec::new();
        for k in 0..3 {
            let l = polyline(vec![v(start[k][0], start[k][1]), v(start[k + 1][0], start[k + 1][1])])
                .style(faded_line(ink, 9.0, 1.0))
                .build();
            arm_links.push(stage.arena_mut().add(l));
        }
        let mut joints = Vec::new();
        for p in start.iter().take(3) {
            let d = Dot::new()
                .point(v(p[0], p[1]))
                .radius(0.16)
                .style(flat(teal, 1.0))
                .build();
            joints.push(stage.arena_mut().add(d));
        }
        let hand = stage.arena_mut().add(
            Dot::new()
                .point(v(start[3][0], start[3][1]))
                .radius(0.14)
                .style(flat(gold, 1.0))
                .build(),
        );
        let arm_anchor = stage.arena_mut().add(VMobject::new());
        let (links_m, joints_m) = (arm_links.clone(), joints.clone());
        let mut last = [f64::NAN; 3];
        stage.add_updater(
            arm_anchor,
            move |s, _| {
                let q = qs.map(|m| s.tracker_value(m).unwrap_or(0.0));
                if q.iter().zip(&last).all(|(a, b)| a.to_bits() == b.to_bits()) {
                    return;
                }
                last = q;
                let pts = by_exponentials(BASE, &LINKS, &HOME, &q);
                for (k, &m) in links_m.iter().enumerate() {
                    let seg = vec![v(pts[k][0], pts[k][1]), v(pts[k + 1][0], pts[k + 1][1])];
                    let _ = s.set_points(m, polyline(seg).build().points());
                }
                for (k, &m) in joints_m.iter().enumerate() {
                    s.move_to(m, v(pts[k][0], pts[k][1]), ORIGIN);
                }
                s.move_to(hand, v(pts[3][0], pts[3][1]), ORIGIN);
            },
            true,
        )?;
        stage.add_to_scene(arm_anchor)?;
        // the formula, one tag per joint, then the home pose M
        let mut tags = Vec::new();
        for k in 1..=3 {
            tags.push(kit.tex(
                stage,
                &format!(r"e^{{[S_{k}]q_{k}}}"),
                48.0,
                teal,
                &[("S", teal), ("q", turning())],
                ORIGIN,
            )?);
        }
        let home_pose = kit.tex(stage, "M", 48.0, gold, &[], ORIGIN)?;
        let mut row = tags.clone();
        row.push(home_pose);
        let total: f64 = row.iter().map(|&m| stage.get_width(m)).sum::<f64>() + 0.2 * (row.len() as f64 - 1.0);
        lay_row(stage, &row, 1.2 - total / 2.0 + 2.0, 3.0, 0.2);
        let tag_levels = Levels::build(stage, &tags, 0.35)?;
        let pointer = chip(&kit, stage, "the companion video: robot arms", 30.0, gold, v(3.9, 0.9))?;
        let mut first = arm_links.clone();
        first.extend(&joints);
        first.extend([hand, home_pose, pointer]);
        first.extend(&tags);
        line.wait_for(stage, "A robot arm's pose is a product of exponentials")?;
        fade_in_all(stage, &first, 0.8)?;
        // each joint turns in its turn, and its tag lights
        for (k, &target) in [0.4, -0.6, 0.9].iter().enumerate() {
            let go = vec![
                stage.prepare(tag_levels.tween(k, 1.0, 0.3)?)?,
                stage.prepare(tween(qs[k], target, 1.1)?)?,
            ];
            stage.play_prepared(go)?;
        }
        line.wait_for(stage, "That's the companion video")?;
        let who = lower_third(
            &kit,
            stage,
            "who and when:",
            &format!("Sophus Lie, {}\u{2013}{}", cited::LIE_BORN, cited::LIE_DIED),
        )?;
        fade_in_all(stage, &[who], 0.5)?;
        let swap = vec![fade_out_anim(stage, who, 0.4)?];
        stage.play_prepared(swap)?;
        let also = lower_third(
            &kit,
            stage,
            "also:",
            "spacecraft attitude, computer graphics, quantum gates",
        )?;
        fade_in_all(stage, &[also], 0.5)?;
        kit.hold(stage)?;

        // ---- 16_2: three Lie groups: eight, three and one tiny steps
        let line = say(&kit, stage, "16_2")?;
        let mut away = first.clone();
        away.push(also);
        stage.clear_updaters(arm_anchor, true);
        stage.clear_updaters(tag_levels.anchor, true);
        fade_out_all(stage, &away, 0.5)?;
        let groups = kit.tex(
            stage,
            r"SU(3)\ \times\ SU(2)\ \times\ U(1)",
            56.0,
            ink,
            &[("SU(3)", teal), ("SU(2)", teal), ("U(1)", teal)],
            v(0.0, 3.0),
        )?;
        let steps = numbers("F-V.16_2.steps")?;
        let group_x = [-3.3, 0.5, 3.2];
        let mut counts = Vec::new();
        for (i, &x) in group_x.iter().enumerate() {
            counts.push(kit.text(stage, &whole(steps[i]), 50.0, gold, v(x, 2.1))?);
        }
        line.wait_for(stage, "built on three Lie groups")?;
        fade_in_all(stage, &[groups], 0.7)?;
        line.wait_for(stage, "with eight, three, and one")?;
        fade_in_all(stage, &counts, 0.7)?;
        // twelve tiles: eight gluons, then the four carriers of SU(2) x U(1)
        let bosons = numbers("F-B.bosons")?;
        let mut tiles = Vec::new();
        for k in 0..bosons[0] as usize {
            let at = v(-4.9 + 1.1 * (k % 4) as f64, 0.55 - 1.0 * (k / 4) as f64);
            tiles.extend(boson_tile(&kit, stage, "g", at, teal, false)?);
        }
        for (k, label) in [r"W^{+}", r"W^{-}", "Z", r"\gamma"].iter().enumerate() {
            let at = v(0.6 + 1.1 * k as f64, -0.45);
            tiles.extend(boson_tile(&kit, stage, label, at, gold, true)?);
        }
        line.wait_for(stage, "That makes twelve")?;
        fade_in_all(stage, &tiles, 0.8)?;
        let carriers = steps[1] + steps[2];
        let brace_label = kit.tex(
            stage,
            &format!(r"SU(2)\times U(1):\ {}+{}={}", steps[1], steps[2], carriers),
            36.0,
            ink,
            &[],
            v(2.4, 0.6),
        )?;
        let gluon_label = kit.text(
            stage,
            &format!("{} gluons", whole(bosons[0])),
            30.0,
            Palette::dim(),
            v(-3.5, -1.6),
        )?;
        let twelve = kit.text(
            stage,
            &format!("{} + {} = {}", whole(steps[0]), whole(carriers), whole(steps[3])),
            44.0,
            gold,
            v(0.0, -2.5),
        )?;
        fade_in_all(stage, &[brace_label, gluon_label, twelve], 0.6)?;
        let fine = lower_third(&kit, stage, "precisely:", "the fine print is in the description")?;
        fade_in_all(stage, &[fine], 0.5)?;
        kit.hold(stage)?;

        // ---- 16_3: one last time, your turn: e to the quarter turn
        let line = say(&kit, stage, "16_3")?;
        let mut off = vec![groups, brace_label, gluon_label, twelve, fine];
        off.extend(&counts);
        off.extend(&tiles);
        fade_out_all(stage, &off, 0.5)?;
        let card = QuizCard::build(&kit, stage, "16_3", [0.0, 0.3, 0.0], (8.4, 5.0))?;
        let (middle, _, _) = card.content();
        let base_e = kit.tex(stage, "e", 130.0, ink, &[], v(middle[0] - 2.1, middle[1] - 0.2))?;
        let power = MatrixGrid::build(
            &kit,
            stage,
            &entries("F-J.power.k1", None)?,
            48.0,
            teal,
            v(middle[0] - 0.7, middle[1] + 0.55),
        )?;
        let equals = kit.tex(stage, "=", 90.0, ink, &[], v(middle[0] + 0.6, middle[1] - 0.2))?;
        let ask = kit.text(stage, "?", 110.0, gold, v(middle[0] + 1.8, middle[1] - 0.2))?;
        line.wait_last(stage)?;
        card.show(stage, &[base_e, power.group, equals, ask], 0.6)?;
        card.count(&kit, stage)?;
        let answer_grid = MatrixGrid::build(
            &kit,
            stage,
            &entries("F-H.J.true", Some(3))?,
            48.0,
            gold,
            v(middle[0] + 2.3, middle[1] - 0.2),
        )?;
        let words = kit.text(
            stage,
            "a rotation by 1 radian",
            32.0,
            gold,
            v(middle[0], middle[1] - 1.0),
        )?;
        let gone = vec![fade_out_anim(stage, ask, 0.3)?];
        stage.play_prepared(gone)?;
        card.land(stage, &[answer_grid.group, words], 0.4)?;

        // ---- 16_4: a line of tiny steps wraps onto a circle of rotations
        let line = say(&kit, stage, "16_4")?;
        let mut leave = card.mobs();
        leave.extend([base_e, power.group, equals, answer_grid.group, words]);
        fade_out_all(stage, &leave, 0.5)?;
        let spec = FlowSpec {
            plane: Plane {
                centre: v(CIRCLE_AT[0], CIRCLE_AT[1]),
                unit: RADIUS,
            },
            starts: lattice(0.5, 1.6),
            dot_radius: 0.045,
            dot_color: ink,
            dot_level: 0.0,
            velocity: None,
            trails: Vec::new(),
            trail_color: gold,
            trail_width: 3.0,
        };
        let turning_plane = Flow::build(stage, spec)?;
        let clock = stage.add_value_tracker(0.0);
        turning_plane.drive(stage, clock, rotation_map)?;
        stage.add_to_scene(turning_plane.group)?;
        let circle = Circle::new()
            .radius(RADIUS)
            .arc_center(v(CIRCLE_AT[0], CIRCLE_AT[1]))
            .style(faded_line(gold, 5.0, 1.0))
            .build();
        let circle = stage.arena_mut().add(circle);
        let curl = stage.add_value_tracker(0.0);
        let reach = stage.add_value_tracker(RULER_SHORT);
        let ruler = polyline(ruler_points(0.0, RULER_SHORT, RULER_POINTS))
            .style(faded_line(teal, 7.0, 1.0))
            .build();
        let ruler = stage.arena_mut().add(ruler);
        let mut ticks = Vec::new();
        for k in 0..=RULER_LONG as usize {
            let (c, w) = if k == 1 { (gold, 7.0) } else { (teal, 4.0) };
            let [a, b] = tick(0.0, k as f64, 0.16);
            let m = polyline(vec![a, b]).style(faded_line(c, w, 1.0)).build();
            ticks.push(stage.arena_mut().add(m));
        }
        let ruler_anchor = stage.arena_mut().add(VMobject::new());
        let ticks_m = ticks.clone();
        let mut last_state = (f64::NAN, f64::NAN);
        stage.add_updater(
            ruler_anchor,
            move |s, _| {
                let kappa = s.tracker_value(curl).unwrap_or(0.0);
                let r = s.tracker_value(reach).unwrap_or(RULER_SHORT);
                if (kappa.to_bits(), r.to_bits()) == (last_state.0.to_bits(), last_state.1.to_bits()) {
                    return;
                }
                last_state = (kappa, r);
                let _ = s.set_points(ruler, polyline(ruler_points(kappa, r, RULER_POINTS)).build().points());
                for (k, &m) in ticks_m.iter().enumerate() {
                    let on = (k as f64) <= r + 1e-9;
                    let [a, b] = tick(kappa, k as f64, 0.16);
                    let _ = s.set_points(m, polyline(if on { vec![a, b] } else { vec![a, a] }).build().points());
                    s.set_stroke(m, None, None, Some(if on { 1.0 } else { 0.0 }), None, false);
                }
            },
            true,
        )?;
        stage.add_to_scene(ruler_anchor)?;
        let line_name = kit.text(stage, "a line of tiny steps:", 30.0, teal, v(4.3, 2.4))?;
        let line_name2 = kit.text(stage, "the Lie algebra", 30.0, teal, v(4.3, 1.9))?;
        let circle_name = kit.text(stage, "a circle of rotations:", 30.0, gold, v(4.3, 0.5))?;
        let circle_name2 = kit.text(stage, "the Lie group", 30.0, gold, v(4.3, 0.0))?;
        line.wait_for(stage, "A rotation by one radian")?;
        let mut enter: Vec<Box<dyn Animation>> = vec![stage.prepare(tween(turning_plane.levels[0], 0.8, 0.6)?)?];
        let mut shown_first = vec![circle, ruler, line_name, line_name2];
        shown_first.extend(&ticks);
        for &m in &shown_first {
            enter.push(fade_in_anim(stage, m, 0.6)?);
        }
        stage.play_prepared(enter)?;
        line.wait_for(stage, "each point travels an arc as long as its radius")?;
        // the line wraps onto the circle, and the plane turns one radian under it
        play!(stage;
            tween_linear(curl, 1.0, 2.6)?,
            tween_linear(clock, 1.0, 2.6)?,
        );
        let radian = chip(&kit, stage, "1 radian", 32.0, gold, v(4.3, -1.2))?;
        fade_in_all(stage, &[circle_name, circle_name2, radian], 0.5)?;
        line.wait_for(stage, "One radian")?;
        play!(stage; tween_linear(reach, RULER_LONG, 3.0)?);

        // ---- 16_5: the rule, its result, and its rate
        let line = say(&kit, stage, "16_5")?;
        let mut clear = vec![circle, ruler, line_name, line_name2, circle_name, circle_name2, radian];
        clear.extend(&ticks);
        clear.push(turning_plane.group);
        stage.clear_updaters(ruler_anchor, true);
        fade_out_all(stage, &clear, 0.6)?;
        let pane_t = stage.add_value_tracker(0.0);
        let mut pane_mobs: Vec<Mob> = Vec::new();
        // the four rules, in teal: a share, i, a matrix, a spin
        let titles = [
            kit.tex(stage, "r", 44.0, teal, &[], v(PANE_X[0], 1.9))?,
            kit.tex(stage, "i", 44.0, teal, &[], v(PANE_X[1], 1.9))?,
            kit.tex(stage, "A", 44.0, teal, &[], v(PANE_X[2], 1.9))?,
            kit.text(stage, "a spin", 36.0, teal, v(PANE_X[3], 1.9))?,
        ];
        pane_mobs.extend(titles);
        let subtitles = ["a share", "the number i", "a matrix", ""];
        for (k, words) in subtitles.iter().enumerate() {
            if !words.is_empty() {
                pane_mobs.push(kit.text(stage, words, 24.0, Palette::dim(), v(PANE_X[k], 1.35))?);
            }
        }
        // their results, in gold: growth, turning, flow, rotation
        let curve = polyline(
            (0..=30)
                .map(|k| {
                    let u = 1.5 * k as f64 / 30.0;
                    v(PANE_X[0] - 1.1 + 1.45 * u, PANE_Y - 0.9 + 0.5 * ((0.9 * u).exp() - 1.0))
                })
                .collect::<Vec<_>>(),
        )
        .style(faded_line(Palette::dim(), 3.0, 0.8))
        .build();
        pane_mobs.push(stage.arena_mut().add(curve));
        let ring = Circle::new()
            .radius(0.9)
            .arc_center(v(PANE_X[1], PANE_Y))
            .style(faded_line(Palette::dim(), 3.0, 0.8))
            .build();
        pane_mobs.push(stage.arena_mut().add(ring));
        let spiral_way = polyline(
            (0..=40)
                .map(|k| {
                    let p = expm_t(&spiral_rule(), 0.35 * TAU * k as f64 / 40.0).apply(&[1.0, 0.0]);
                    v(PANE_X[2] + 0.9 * p[0], PANE_Y + 0.9 * p[1])
                })
                .collect::<Vec<_>>(),
        )
        .style(faded_line(Palette::dim(), 3.0, 0.8))
        .build();
        pane_mobs.push(stage.arena_mut().add(spiral_way));
        let start_places = pane_places(0.0);
        let mut movers = Vec::new();
        for p in start_places {
            movers.push(
                stage
                    .arena_mut()
                    .add(Dot::new().point(p).radius(0.13).style(flat(gold, 1.0)).build()),
            );
        }
        let pane_anchor = stage.arena_mut().add(VMobject::new());
        let movers_m = movers.clone();
        let mut last_t = f64::NAN;
        stage.add_updater(
            pane_anchor,
            move |s, _| {
                let t = s.tracker_value(pane_t).unwrap_or(0.0);
                if t.to_bits() == last_t.to_bits() {
                    return;
                }
                last_t = t;
                for (&m, p) in movers_m.iter().zip(pane_places(t)) {
                    s.move_to(m, p, ORIGIN);
                }
            },
            true,
        )?;
        stage.add_to_scene(pane_anchor)?;
        pane_mobs.extend(&movers);
        let lz = spin_axes()[2].clone();
        let cube = Cube::build(
            stage,
            v(PANE_X[3], PANE_Y),
            0.6,
            (VIEW_YAW, VIEW_TILT),
            move |s: &Arena| expm_t(&lz, s.tracker_value(pane_t).unwrap_or(0.0)),
        )?;
        pane_mobs.extend(cube.mobs());
        let rate = kit.tex(
            stage,
            r"\frac{d}{dt}\,e^{tA}=A\,e^{tA}",
            52.0,
            ink,
            &[("e", gold), ("tA", teal)],
            v(0.0, -2.6),
        )?;
        pane_mobs.push(rate);
        fade_in_all(stage, &pane_mobs, 0.7)?;
        line.wait_for(stage, "E to the matrix is where that rule takes you")?;
        play!(stage; tween_linear(pane_t, TAU, 5.0)?);
        music_hold(&kit, stage)?;
        // the receipts, and the names to search
        let receipts = receipts_card(
            &kit,
            stage,
            &[
                "every number in this video was computed by code;",
                "every dot was placed by exp(tA).",
                "github.com/Dicklesworthstone/jeffs_math_explainer_videos",
            ],
        )?;
        let names = names_to_search(
            &kit,
            stage,
            &[
                "matrix exponential",
                "Baker\u{2013}Campbell\u{2013}Hausdorff",
                "Lie algebra",
            ],
        )?;
        stage.clear_updaters(pane_anchor, true);
        stage.clear_updaters(cube.anchor, true);
        fade_out_all(stage, &pane_mobs, 0.5)?;
        fade_in_all(stage, &[receipts, names], 0.7)?;
        kit.hold(stage)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::{fact, testing};
    use super::*;
    use crate::model::{quarter_turn, su_dim};

    /// 16_1: the arm's pose as a product of exponentials, one per joint, is
    /// its forward kinematics: every joint and the hand land where the links,
    /// turned by the summed joint angles, put them, for any joint angles; the
    /// three exponentials never move the base.
    #[test]
    fn a_product_of_exponentials_is_the_arms_forward_kinematics() {
        let angles: [[f64; 3]; 5] = [
            [0.0, 0.0, 0.0],
            [0.4, -0.7, 0.2],
            [-1.3, 0.9, 1.6],
            [3.0, -2.5, 0.3],
            [0.0, 2.0, -2.0],
        ];
        for q in angles {
            let got = by_exponentials(BASE, &LINKS, &HOME, &q);
            let moved = [HOME[0] + q[0], HOME[1] + q[1], HOME[2] + q[2]];
            let want = arm_points(BASE, &LINKS, &moved);
            for (k, (g, w)) in got.iter().zip(&want).enumerate() {
                assert!(
                    (g[0] - w[0]).abs() < 1e-9 && (g[1] - w[1]).abs() < 1e-9,
                    "q = {q:?}, point {k}: {g:?} against {w:?}"
                );
            }
            assert_eq!(got[0], BASE, "the base stays put");
        }
        // the twist is skew in its turning part and fixes its own joint
        assert_eq!(arm_points(BASE, &LINKS, &HOME)[0], BASE);
        let t = twist([2.0, 1.0]);
        assert_eq!((t.get(0, 1), t.get(1, 0)), (-1.0, 1.0));
        assert_eq!(
            t.apply(&[2.0, 1.0, 1.0]),
            vec![0.0, 0.0, 0.0],
            "the joint itself does not move"
        );
    }

    /// 16_2: eight, three and one tiny steps make twelve, the carriers of
    /// the three groups are twelve, and the strips' numbers are the facts'.
    #[test]
    fn eight_three_and_one_make_twelve_and_so_do_the_carriers() {
        let steps = numbers("F-V.16_2.steps").unwrap();
        assert_eq!(steps, [8.0, 3.0, 1.0, 12.0]);
        assert_eq!((su_dim(3), su_dim(2)), (8, 3));
        assert_eq!(su_dim(3) + su_dim(2) + 1, 12);
        let bosons = numbers("F-B.bosons").unwrap();
        assert_eq!(bosons, [8.0, 2.0, 1.0, 1.0]);
        assert_eq!(bosons.iter().sum::<f64>(), steps[3]);
        // the four under the brace are SU(2) x U(1): 3 + 1 = 4 = W+, W-, Z, photon
        assert_eq!(steps[1] + steps[2], bosons[1] + bosons[2] + bosons[3]);
        assert_eq!(fact("F-L.standard_model.steps").unwrap(), "12");
        assert_eq!((cited::LIE_BORN, cited::LIE_DIED), (1842, 1899));
    }

    /// 16_4: the ruler is the circle at curvature one (its points are the
    /// first column of e^{sJ}), and it never stretches however far it has
    /// wrapped; the circle is touched at (1, 0) with the ruler going up.
    #[test]
    fn the_ruler_wraps_onto_the_circle_without_stretching() {
        let j = quarter_turn();
        for s in [0.0, 0.5, 1.0, 2.0, std::f64::consts::PI, 5.0, 7.0] {
            let p = wrap_point(1.0, s);
            let c = expm_t(&j, s).column(0);
            assert!((p[0] - c[0]).abs() < 1e-12 && (p[1] - c[1]).abs() < 1e-12, "s = {s}");
        }
        assert_eq!(wrap_point(0.0, 1.0), [1.0, 1.0]);
        for kappa in [0.0, 0.05, 0.3, 0.7, 1.0] {
            let n = 2000;
            let pts: Vec<[f64; 2]> = (0..=n)
                .map(|k| wrap_point(kappa, RULER_LONG * k as f64 / n as f64))
                .collect();
            let length: f64 = pts
                .windows(2)
                .map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]))
                .sum();
            assert!((length - RULER_LONG).abs() < 1e-3, "kappa = {kappa}: length {length}");
            assert_eq!(pts[0], [1.0, 0.0], "it always starts where it touches");
        }
        // one tiny step is one radian: the tick at 1 lands at the angle one
        let at = wrap_point(1.0, 1.0);
        assert!((at[1].atan2(at[0]) - 1.0).abs() < 1e-12);
        // and a ruler of seven wraps past a full turn: its far end is back
        // round the circle, past where it started
        let far = wrap_point(1.0, RULER_LONG);
        assert!(far[1].atan2(far[0]) > 0.0 && far[1].atan2(far[0]) < 1.0, "{far:?}");
    }

    /// 16_5: the panes' moving parts stay where they say: the share's amount
    /// grows, the point goes round at distance 0.9 from the centre, and the
    /// matrix spirals inward.
    #[test]
    fn the_panes_move_as_their_rules_say() {
        let [g0, r0, f0] = pane_places(0.0);
        assert!((g0[0] - (PANE_X[0] - 1.1)).abs() < 1e-12 && (g0[1] - (PANE_Y - 0.9)).abs() < 1e-12);
        assert!((r0[0] - (PANE_X[1] + 0.9)).abs() < 1e-12 && (r0[1] - PANE_Y).abs() < 1e-12);
        assert!((f0[0] - (PANE_X[2] + 0.9)).abs() < 1e-12);
        let mut radius = f64::INFINITY;
        let mut height = f64::NEG_INFINITY;
        for k in 0..=40 {
            let t = TAU * f64::from(k) / 40.0;
            let [g, r, f] = pane_places(t);
            assert!(g[1] >= height - 1e-12, "the amount only grows");
            height = g[1];
            let round = (r[0] - PANE_X[1]).hypot(r[1] - PANE_Y);
            assert!((round - 0.9).abs() < 1e-12, "going round at a steady distance");
            let away = (f[0] - PANE_X[2]).hypot(f[1] - PANE_Y);
            assert!(away <= radius + 1e-12, "the spiral only comes in");
            radius = away;
        }
    }

    /// The chapter runs on the storyboard with the layout lint clean, types
    /// the strings it should, and stays inside the chapter budget.
    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("16_where_it_leads").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            "e^{[S_1]q_1}",
            "the companion video: robot arms",
            "Sophus Lie, 1842\u{2013}1899",
            "SU(3)\\ \\times\\ SU(2)\\ \\times\\ U(1)",
            "8 + 4 = 12",
            "a rotation by 1 radian",
            "a line of tiny steps:",
            "1 radian",
            "\\frac{d}{dt}\\,e^{tA}=A\\,e^{tA}",
            "every dot was placed by exp(tA).",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never typesets {want:?}: {words:?}"
            );
        }
        assert!(ran.seconds > 45.0 && ran.seconds < 85.0, "{} s", ran.seconds);
    }
}
