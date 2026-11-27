//! Track the same two joint rotations on the arm and its configuration torus.

use std::f64::consts::{FRAC_PI_2, TAU};

use explainer_kit::{Kit, Palette, Timed, add, fade_all, play, tween, v};
use fmn::prelude::*;

use crate::{model::Arm2R, palette};

fn line(points: Vec<Vec3>, color: Srgb, thickness: f64) -> VMobject {
    polyline(points).style(palette::line_style(color, thickness)).build()
}

fn marker(point: Vec3, color: Srgb, radius: f64) -> VMobject {
    Dot::new().point(point).color(color).radius(radius).build()
}

fn title(kit: &Kit, stage: &mut Stage<'_>, text: &str) -> fmn::Result<Mob> {
    kit.text(stage, text, 37.0, Palette::ink(), v(0.0, 3.4))
}

fn reveal(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut animations = Vec::new();
    for &mob in mobs {
        let fade = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?.rt(0.48);
        animations.push(stage.prepare(fade)?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

fn arm_point([x, y]: [f64; 2], scale: f64) -> Vec3 {
    v(-3.7 + scale * x, -0.19 + scale * y)
}

fn arm(links: Arm2R, shoulder: f64, elbow: f64, scale: f64) -> Vec<VMobject> {
    let base = arm_point([0.0, 0.0], scale);
    let hinge = arm_point(links.elbow(shoulder), scale);
    let tip = arm_point(links.hand(shoulder, elbow), scale);
    vec![
        line(vec![base, hinge], palette::rotation(), 9.0),
        line(vec![hinge, tip], palette::translation(), 9.0),
        marker(base, palette::rotation(), 0.14),
        marker(hinge, palette::translation(), 0.12),
        marker(tip, palette::hero(), 0.1),
    ]
}

const SQUARE_LEFT: f64 = 0.50;
const SQUARE_BOTTOM: f64 = -1.31;
const SQUARE_SIDE: f64 = 2.36;

fn angle_square(shoulder: f64, elbow: f64) -> Vec3 {
    v(
        SQUARE_LEFT + SQUARE_SIDE * shoulder.rem_euclid(TAU) / TAU,
        SQUARE_BOTTOM + SQUARE_SIDE * elbow.rem_euclid(TAU) / TAU,
    )
}

fn torus_point(shoulder: f64, elbow: f64) -> Vec3 {
    let (sin_shoulder, cos_shoulder) = shoulder.sin_cos();
    let (sin_elbow, cos_elbow) = elbow.sin_cos();
    let radius = 1.22 + 0.47 * cos_elbow;
    v(
        4.86 + radius * cos_shoulder,
        -0.10 + 0.49 * radius * sin_shoulder + 0.87 * 0.47 * sin_elbow,
    )
}

fn ring(color: Srgb, thickness: f64, at: impl Fn(f64) -> Vec3) -> VMobject {
    line(
        (0..=64).map(|i| at(TAU * f64::from(i) / 64.0)).collect(),
        color,
        thickness,
    )
}

fn torus_mesh() -> VMobject {
    let rings = (0..8).flat_map(|i| {
        let fixed = TAU * f64::from(i) / 8.0;
        [
            ring(Palette::dim(), 0.75, |shoulder| torus_point(shoulder, fixed)),
            ring(Palette::dim(), 0.75, |elbow| torus_point(fixed, elbow)),
        ]
    });
    v_group(rings.collect::<Vec<_>>())
}

fn traced_arm(shoulder: f64, elbow: f64) -> VMobject {
    let links = Arm2R::UNIT;
    let mut shapes = Vec::new();
    let arc = if shoulder > 0.001 {
        Some((shoulder.min(TAU), palette::rotation(), true))
    } else if elbow > FRAC_PI_2 + 0.001 {
        Some(((elbow - FRAC_PI_2).min(TAU), palette::translation(), false))
    } else {
        None
    };
    if let Some((sweep, color, move_shoulder)) = arc {
        let points = (0..=72)
            .map(|i| {
                let t = sweep * f64::from(i) / 72.0;
                let hand = if move_shoulder {
                    links.hand(t, FRAC_PI_2)
                } else {
                    links.hand(0.0, FRAC_PI_2 + t)
                };
                arm_point(hand, 1.04)
            })
            .collect();
        shapes.push(line(points, color, 2.4));
    }
    shapes.extend(arm(links, shoulder, elbow, 1.04));
    v_group(shapes)
}

pub fn construct(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    kit.say(stage, "02_1")?;
    let heading = title(kit, stage, "Two hinges make a torus")?;
    let desc = kit.text(stage, "A physical arm", 27.0, Palette::ink(), v(-3.7, 2.67))?;
    reveal(stage, &[heading, desc])?;
    let shoulder = stage.add_value_tracker(0.0);
    let elbow = stage.add_value_tracker(0.0);
    let arm_view = stage.always_redraw(move |scene| {
        let q1 = scene.tracker_value(shoulder).unwrap_or(0.0);
        let q2 = scene.tracker_value(elbow).unwrap_or(0.0);
        scene.add(traced_arm(q1, q2))
    });
    stage.add_to_scene(arm_view)?;

    kit.say(stage, "02_2")?;
    let pair = kit.tex(
        stage,
        r"(q_1,q_2)\in SO(2)\times SO(2)",
        29.0,
        palette::hero(),
        &[],
        v(3.17, 2.68),
    )?;
    reveal(stage, &[pair])?;

    kit.say(stage, "02_3")?;
    let right = SQUARE_LEFT + SQUARE_SIDE;
    let top = SQUARE_BOTTOM + SQUARE_SIDE;
    let center = SQUARE_LEFT + SQUARE_SIDE / 2.0;
    let corners = [
        v(SQUARE_LEFT, SQUARE_BOTTOM),
        v(right, SQUARE_BOTTOM),
        v(right, top),
        v(SQUARE_LEFT, top),
    ];
    let mut board = Vec::new();
    for (index, color) in [
        palette::translation(),
        palette::rotation(),
        palette::translation(),
        palette::rotation(),
    ]
    .into_iter()
    .enumerate()
    {
        let edge = line(vec![corners[index], corners[(index + 1) % 4]], color, 3.1);
        board.push(stage.arena_mut().add(edge));
    }
    let q1 = kit.tex(
        stage,
        r"q_1\;\mathrm{shoulder}",
        23.0,
        palette::rotation(),
        &[],
        v(center, -1.66),
    )?;
    let q2 = kit.tex(stage, r"q_2", 23.0, palette::translation(), &[], v(0.08, -0.13))?;
    let wrap = kit.text(stage, "Match the colored edges", 20.0, Palette::ink(), v(center, -2.16))?;
    let mesh = stage.arena_mut().add(torus_mesh());
    let same = kit.text(stage, "Same two angles", 20.0, Palette::ink(), v(4.86, -2.16))?;
    board.extend([q1, q2, wrap, mesh, same]);
    reveal(stage, &board)?;
    let point = stage.always_redraw(move |scene| {
        let q1 = scene.tracker_value(shoulder).unwrap_or(0.0);
        let q2 = scene.tracker_value(elbow).unwrap_or(0.0);
        scene.add(v_group(vec![
            ring(palette::rotation(), 2.5, |theta| torus_point(theta, q2)),
            ring(palette::translation(), 2.5, |theta| torus_point(q1, theta)),
            marker(angle_square(q1, q2), palette::hero(), 0.10),
            marker(torus_point(q1, q2), palette::hero(), 0.10),
        ]))
    });
    stage.add_to_scene(point)?;

    kit.say(stage, "02_4")?;
    play!(stage; tween(elbow, FRAC_PI_2, 1.55)?);
    let [x, y] = Arm2R::UNIT.hand(0.0, FRAC_PI_2);
    let endpoint = kit.tex(
        stage,
        &format!(r"\mathrm{{hand}}=({x:.0},{y:.0})"),
        26.0,
        palette::hero(),
        &[],
        v(-3.7, -2.7),
    )?;
    reveal(stage, &[endpoint])?;
    kit.hold(stage)?;
    let fade_endpoint = fade_out(stage.arena_mut(), endpoint, ORIGIN, 1.0)?.rt(0.32);
    stage.play(fade_endpoint)?;

    kit.say(stage, "02_5")?;
    let fixed = kit.text(stage, "Shoulder fixed", 24.0, palette::translation(), v(-3.7, -2.7))?;
    reveal(stage, &[fixed])?;
    play!(stage; tween(elbow, FRAC_PI_2 + TAU, 4.8)?);

    kit.say(stage, "02_6")?;
    let fade_fixed = fade_out(stage.arena_mut(), fixed, ORIGIN, 1.0)?.rt(0.32);
    stage.play(fade_fixed)?;
    let prompt = kit.text(
        stage,
        "Elbow fixed. What path does the hand draw?",
        22.0,
        palette::quiz(),
        v(-3.7, -2.7),
    )?;
    reveal(stage, &[prompt])?;
    kit.hold(stage)?;
    kit.listen(stage, 5.0, "02_6_predict")?;
    let fade_prompt = fade_out(stage.arena_mut(), prompt, ORIGIN, 1.0)?.rt(0.32);
    stage.play(fade_prompt)?;

    kit.say(stage, "02_7")?;
    play!(stage; tween(shoulder, TAU, 4.8)?);
    let radius = kit.tex(
        stage,
        r"\mathrm{radius}=\sqrt{1^2+1^2}=\sqrt{2}",
        26.0,
        palette::rotation(),
        &[],
        v(-3.7, -2.7),
    )?;
    reveal(stage, &[radius])?;
    kit.hold(stage)?;
    fade_all(stage, 0.5)?;

    kit.say(stage, "02_8")?;
    let heading = title(kit, stage, "From the hand back to the motors")?;
    let bench = Arm2R::BENCH;
    let bench_pose = stage.arena_mut().add(v_group(arm(bench, 0.0, FRAC_PI_2, 5.3)));
    let limits = kit.text(
        stage,
        "Joint stops restrict each circle to an arc",
        23.0,
        Palette::ink(),
        v(2.8, 1.7),
    )?;
    let hand = arm_point(bench.hand(0.0, FRAC_PI_2), 5.3);
    let force = Arrow::new(hand, add(hand, v(1.45, 0.0)))
        .color(palette::velocity())
        .thickness(5.0)
        .build()?;
    let force_arrow = stage.arena_mut().add(force);
    reveal(stage, &[heading, bench_pose, limits])?;

    kit.say(stage, "02_9")?;
    let force = [10.0, 0.0];
    let [shoulder_torque, elbow_torque] = bench.torques(0.0, FRAC_PI_2, force);
    let load = kit.tex(
        stage,
        r"\mathbf F=(10,0)\,\mathrm N",
        25.0,
        palette::velocity(),
        &[],
        v(-2.8, 2.04),
    )?;
    let rule = kit.tex(
        stage,
        r"\delta p=J\delta q,\quad \mathbf F\cdot\delta p=(J^T\mathbf F)\cdot\delta q",
        24.0,
        Palette::ink(),
        &[],
        v(2.8, 0.76),
    )?;
    let torques = kit.tex(
        stage,
        &format!(r"\tau=J^T\mathbf F=({shoulder_torque:.0},{elbow_torque:.0})\,\mathrm{{N\,m}}"),
        26.0,
        palette::hero(),
        &[],
        v(2.8, -0.16),
    )?;
    reveal(stage, &[force_arrow, load, rule, torques])?;

    kit.say(stage, "02_10")?;
    let rates = [0.2, -0.1];
    let [vx, vy] = bench.tip_velocity(0.0, FRAC_PI_2, rates);
    let velocity = kit.tex(
        stage,
        &format!(r"\dot p=({vx:.2},{vy:.2})\,\mathrm{{m/s}}"),
        25.0,
        palette::translation(),
        &[],
        v(-3.7, -2.39),
    )?;
    let power = force[0] * vx + force[1] * vy;
    let power_line = kit.tex(
        stage,
        &format!(r"\mathbf F\cdot\dot p=\tau\cdot\dot q={power:.1}\,\mathrm W"),
        26.0,
        palette::good(),
        &[],
        v(2.8, -1.44),
    )?;
    reveal(stage, &[velocity, power_line])?;
    kit.finish(stage)
}
