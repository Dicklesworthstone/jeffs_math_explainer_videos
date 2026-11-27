//! Joint trees, repeated limbs and nested revolute actions from the gallery.

use std::f64::consts::{FRAC_PI_6, TAU};

use explainer_kit::{Kit, Palette, Timed, add, fade_all, play, tween, v};
use fmn::prelude::*;

use crate::{
    model::{
        STAIR_AXLE_OFFSET_M, STAIR_IDLER_TEETH, STAIR_SUN_TEETH, STAIR_TIRE_RADIUS_M,
        STAIR_WHEEL_TEETH, balancer_wheel_relative, stair_cluster_angles,
    },
    palette,
};

fn stroke(points: Vec<Vec3>, color: Srgb, width: f64) -> VMobject {
    polyline(points).style(palette::line_style(color, width)).build()
}

fn dot(at: Vec3, color: Srgb, radius: f64) -> VMobject {
    Dot::new().point(at).radius(radius).color(color).build()
}

fn show(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut animations = Vec::new();
    for &mob in mobs {
        let fade = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?.rt(0.46);
        animations.push(stage.prepare(fade)?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

fn title(kit: &Kit, stage: &mut Stage<'_>, label: &str) -> fmn::Result<Mob> {
    kit.text(stage, label, 37.0, Palette::ink(), v(0.0, 3.4))
}

fn clear(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    kit.hold(stage)?;
    fade_all(stage, 0.45)
}

fn ring(center: Vec3, radius: f64, color: Srgb) -> VMobject {
    let points = (0..=96)
        .map(|k| {
            let angle = TAU * f64::from(k) / 96.0;
            add(center, v(radius * angle.cos(), radius * angle.sin()))
        })
        .collect();
    stroke(points, color, 2.65)
}

fn tree(stage: &mut Stage<'_>, points: &[Vec3], edges: &[(usize, usize)], path: &[usize]) -> Mob {
    let mut shapes = Vec::new();
    for &(parent, child) in edges {
        let active = path.windows(2).any(|pair| pair == [parent, child]);
        let color = if active { palette::hero() } else { palette::steel() };
        shapes.push(stroke(
            vec![points[parent], points[child]],
            color,
            if active { 5.4 } else { 2.5 },
        ));
    }
    for (index, &at) in points.iter().enumerate() {
        let color = if path.contains(&index) {
            palette::hero()
        } else {
            palette::rotation()
        };
        shapes.push(dot(at, color, 0.105));
    }
    stage.arena_mut().add(v_group(shapes))
}

pub fn construct(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    kit.say(stage, "11_1")?;
    let title_card = title(kit, stage, "The tree tells us which matrices to multiply")?;
    let points = [
        v(-3.5, 1.8),
        v(-5.45, 0.85),
        v(-4.15, 0.85),
        v(-2.85, 0.85),
        v(-1.55, 0.85),
        v(-5.45, -0.05),
        v(-4.15, -0.05),
        v(-2.85, -0.05),
        v(-1.55, -0.05),
        v(-5.45, -0.95),
        v(-4.15, -0.95),
        v(-2.85, -0.95),
        v(-1.55, -0.95),
        v(-5.45, -1.65),
        v(-4.15, -1.65),
        v(-2.85, -1.65),
        v(-1.55, -1.65),
    ];
    let joints = [
        (0, 1),
        (0, 2),
        (0, 3),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
        (4, 8),
        (5, 9),
        (6, 10),
        (7, 11),
        (8, 12),
        (9, 13),
        (10, 14),
        (11, 15),
        (12, 16),
    ];
    let graph = tree(stage, &points, &joints, &[0, 2, 6, 10, 14]);
    let formula = kit.tex(
        stage,
        r"T_{\mathrm{foot}}=T_{\mathrm{torso}}T_{\mathrm{roll}}T_{\mathrm{pitch}}T_{\mathrm{knee}}T_{\mathrm{offset}}",
        25.0,
        palette::hero(),
        &[],
        v(2.85, 0.9),
    )?;
    let text = kit.text(
        stage,
        "Each hip leads to one foot",
        24.0,
        Palette::ink(),
        v(2.9, -0.28),
    )?;
    show(stage, &[title_card, graph, formula, text])?;
    clear(kit, stage)?;

    kit.say(stage, "11_2")?;
    let title_card = title(kit, stage, "Same joint count, different placement")?;
    let bend = stage.add_value_tracker(0.0);
    let profiles = stage.always_redraw(move |s| {
        let angle = s.tracker_value(bend).unwrap_or(0.0);
        let mut shapes = Vec::new();
        for (x, tilt) in [(-3.25, 0.23_f64), (3.25, 0.92_f64)] {
            let top = v(x, 1.10);
            shapes.push(stroke(
                vec![add(top, v(-0.88, 0.0)), add(top, v(0.88, 0.0))],
                palette::steel(),
                9.0,
            ));
            for sign in [-1.0, 1.0] {
                let hip = add(top, v(0.79 * sign, 0.0));
                let knee = add(hip, v(0.9 * sign * tilt.sin(), -0.9 * tilt.cos()));
                let follow = tilt - angle;
                let foot = add(knee, v(1.18 * sign * follow.sin(), -1.18 * follow.cos()));
                shapes.push(stroke(vec![hip, knee], palette::rotation(), 7.0));
                shapes.push(stroke(vec![knee, foot], palette::translation(), 6.0));
                shapes.push(dot(foot, palette::hero(), 0.10));
            }
        }
        s.add(v_group(shapes))
    });
    stage.add_to_scene(profiles)?;
    let dog = kit.text(stage, "Under the torso", 26.0, Palette::ink(), v(-3.25, 2.06))?;
    let spider = kit.text(stage, "Sprawled layout", 26.0, Palette::ink(), v(3.25, 2.06))?;
    let count = kit.tex(
        stage,
        r"12\ \text{rotary joints each}\qquad C_2\times C_2\ \text{for ideal leg slots}",
        24.0,
        palette::symmetry(),
        &[],
        v(0.0, -2.65),
    )?;
    show(stage, &[title_card, dog, spider, count])?;
    play!(stage; tween(bend, 0.5, 2.3)?);
    clear(kit, stage)?;

    kit.say(stage, "11_3")?;
    let title_card = title(kit, stage, "Rotation acts on eight command slots")?;
    let center = v(-3.25, 0.0);
    let mut background = vec![title_card];
    for i in 0..8 {
        let angle = TAU * f64::from(i) / 8.0;
        let unit = v(angle.cos(), angle.sin());
        background.push(stage.arena_mut().add(stroke(
            vec![center, add(center, unit.map(|x| x * 1.82))],
            palette::steel(),
            2.4,
        )));
        background.push(kit.text(
            stage,
            &i.to_string(),
            24.0,
            Palette::ink(),
            add(center, unit.map(|x| x * 2.21)),
        )?);
    }
    let group = kit.tex(
        stage,
        r"D_8:\quad r(i)=i+1\pmod{8}",
        28.0,
        palette::symmetry(),
        &[],
        v(2.95, 1.05),
    )?;
    background.push(group);
    show(stage, &background)?;
    let step = stage.add_value_tracker(0.0);
    let channels = stage.always_redraw(move |s| {
        let turns = s.tracker_value(step).unwrap_or(0.0);
        let shapes: Vec<_> = [1, 0, -1, 0, 1, 0, -1, 0]
            .into_iter()
            .enumerate()
            .map(|(index, sign)| {
                let angle = TAU * (index as f64 + turns) / 8.0;
                let color = match sign {
                    1 => palette::hero(),
                    -1 => palette::translation(),
                    _ => Palette::faint(),
                };
                dot(
                    add(center, v(1.68 * angle.cos(), 1.68 * angle.sin())),
                    color,
                    if sign == 0 { 0.08 } else { 0.17 },
                )
            })
            .collect();
        s.add(v_group(shapes))
    });
    stage.add_to_scene(channels)?;
    play!(stage; tween(step, 1.0, 1.4)?);
    let shift = kit.text(
        stage,
        "One step moves the signs to odd slots",
        21.0,
        Palette::ink(),
        v(2.95, -0.15),
    )?;
    show(stage, &[shift])?;
    play!(stage; tween(step, 2.0, 1.4)?);
    let neg = kit.tex(stage, r"P_r^2 f=-f", 34.0, palette::hero(), &[], v(2.95, -1.18))?;
    show(stage, &[neg])?;
    clear(kit, stage)?;

    kit.say(stage, "11_4")?;
    let title_card = title(kit, stage, "A mirror exchanges the stance leg")?;
    let mut poses = vec![title_card];
    for (x, planted) in [(-3.25, 1_usize), (3.25, 2_usize)] {
        let lifted = if planted == 1 { [0.0, 0.36] } else { [0.36, 0.0] };
        let points = [
            v(x, 1.7),
            v(x - 0.47, 0.7),
            v(x + 0.47, 0.7),
            v(x - 0.86, -0.39 + lifted[0]),
            v(x + 0.86, -0.39 + lifted[1]),
            v(x - 0.99, -1.35 + lifted[0]),
            v(x + 0.99, -1.35 + lifted[1]),
        ];
        let foot = if planted == 1 { 5 } else { 6 };
        poses.push(tree(
            stage,
            &points,
            &[(0, 1), (0, 2), (1, 3), (2, 4), (3, 5), (4, 6)],
            &[0, planted, foot - 2, foot],
        ));
        poses.push(stage.arena_mut().add(stroke(
            vec![v(x - 1.42, -1.38), v(x + 1.42, -1.38)],
            palette::steel(),
            2.0,
        )));
    }
    let mirror = kit.tex(
        stage,
        r"\text{left stance}\ \leftrightarrow\ \text{right stance}",
        26.0,
        palette::symmetry(),
        &[],
        v(0.0, -2.51),
    )?;
    poses.push(mirror);
    show(stage, &poses)?;
    clear(kit, stage)?;

    kit.say(stage, "11_5")?;
    let title_card = title(kit, stage, "Three wheels, one geared drive")?;
    let center = v(-3.35, 0.0);
    let progress = stage.add_value_tracker(0.0);
    let carrier_end = FRAC_PI_6;
    let sun_end = 5.0_f64.to_radians();
    let pitch_radius = |teeth: u32| 0.0215 * f64::from(teeth);
    let sun_radius = pitch_radius(STAIR_SUN_TEETH);
    let idler_radius = pitch_radius(STAIR_IDLER_TEETH);
    let wheel_radius = pitch_radius(STAIR_WHEEL_TEETH);
    let axle_offset = sun_radius + 2.0 * idler_radius + wheel_radius;
    let tire_radius = STAIR_TIRE_RADIUS_M / STAIR_AXLE_OFFSET_M * axle_offset;
    let cluster = stage.always_redraw(move |s| {
        let t = s.tracker_value(progress).unwrap_or(0.0);
        let carrier = t * carrier_end;
        let sun = t * sun_end;
        let (idler, _, rim) = stair_cluster_angles(carrier, sun);
        let radial = |radius: f64, angle: f64| v(radius * angle.cos(), radius * angle.sin());
        let mut shapes = vec![
            ring(center, sun_radius, palette::hero()),
            stroke(vec![center, add(center, radial(0.81 * sun_radius, sun))], palette::hero(), 3.8),
            dot(center, palette::hero(), 0.08),
        ];
        for index in 0..3 {
            let angle = carrier + TAU * f64::from(index) / 3.0;
            let wheel = add(center, radial(axle_offset, angle));
            let idle_gear = add(center, radial(sun_radius + idler_radius, angle));
            shapes.push(stroke(
                vec![add(center, radial(sun_radius, angle)), wheel], palette::rotation(), 3.4,
            ));
            shapes.push(ring(idle_gear, idler_radius, palette::rotation()));
            shapes.push(stroke(
                vec![idle_gear, add(idle_gear, radial(0.74 * idler_radius, idler))], palette::rotation(), 2.8,
            ));
            shapes.push(ring(wheel, tire_radius, palette::steel()));
            shapes.push(ring(wheel, wheel_radius, palette::translation()));
            shapes.push(dot(wheel, palette::translation(), 0.07));
            shapes.push(stroke(
                vec![wheel, add(wheel, radial(0.91 * tire_radius, rim))], palette::translation(), 4.0,
            ));
        }
        s.add(v_group(shapes))
    });
    stage.add_to_scene(cluster)?;
    show(stage, &[title_card])?;
    kit.until_sentence(stage, "11_5", 2)?;
    let pose = kit.tex(
        stage,
        &format!(r"R(c)\,T({STAIR_AXLE_OFFSET_M:.2},0)\,R(w)"),
        27.0,
        palette::hero(),
        &[],
        v(2.85, 2.06),
    )?;
    let axes = kit.text(
        stage, "Parallel axles · angles add · the hub moves", 23.0, Palette::ink(), v(2.85, 1.36),
    )?;
    show(stage, &[pose, axes])?;
    kit.until_sentence(stage, "11_5", 5)?;
    let gears = kit.text(
        stage,
        &format!("{STAIR_SUN_TEETH}-tooth sun → {STAIR_IDLER_TEETH}-tooth idler → {STAIR_WHEEL_TEETH}-tooth wheel gear"),
        22.0,
        palette::rotation(),
        v(2.85, 0.57),
    )?;
    let ratio = kit.tex(
        stage,
        &format!(r"w=\frac{{{STAIR_SUN_TEETH}}}{{{STAIR_WHEEL_TEETH}}}(s-c)"),
        27.0,
        palette::translation(),
        &[],
        v(2.85, -0.23),
    )?;
    show(stage, &[gears, ratio])?;
    kit.until_sentence(stage, "11_5", 7)?;
    play!(stage; tween(progress, 1.0, 2.4)?);
    let (_, relative, net) = stair_cluster_angles(carrier_end, sun_end);
    let cancellation = kit.tex(
        stage,
        &format!(
            r"w=\frac{{{STAIR_SUN_TEETH}}}{{{STAIR_WHEEL_TEETH}}}({:.0}^\circ-{:.0}^\circ)={:.0}^\circ,\quad c+w={:.0}^\circ",
            sun_end.to_degrees(), carrier_end.to_degrees(), relative.to_degrees(),
            net.to_degrees().round() + 0.0,
        ),
        26.0,
        palette::symmetry(),
        &[],
        v(2.85, -1.25),
    )?;
    let guide = kit.text(
        stage, "The three wheel marks keep their direction as the carrier swings", 21.0,
        Palette::ink(), v(0.0, -2.80),
    )?;
    show(stage, &[cancellation, guide])?;
    clear(kit, stage)?;

    kit.say(stage, "11_6")?;
    let title_card = title(kit, stage, "A wheel inherits the passive joints above it")?;
    let points = [
        v(-3.15, 1.94),
        v(-3.15, 0.82),
        v(-4.37, -0.28),
        v(-1.93, -0.28),
        v(-5.22, -1.54),
        v(-3.55, -1.54),
    ];
    let rig = tree(stage, &points, &[(0, 1), (1, 2), (1, 3), (2, 4), (2, 5)], &[0, 1, 2, 4]);
    let mut parts = vec![title_card, rig];
    for (name, i, offset) in [
        ("Torso", 0, v(0.0, 0.35)),
        ("Rocker", 1, v(-0.72, 0.06)),
        ("Bogie", 2, v(-0.67, 0.06)),
        ("Front wheel", 3, v(0.79, 0.02)),
        ("Rear wheel", 4, v(0.0, -0.37)),
    ] {
        parts.push(kit.text(stage, name, 20.0, Palette::ink(), add(points[i], offset))?);
    }
    let rear = kit.tex(
        stage,
        r"T_{\mathrm{rear}}=T_{\mathrm{torso}}T_{\mathrm{rocker}}T_{\mathrm{bogie}}T_{\mathrm{spin}}",
        25.0,
        palette::hero(),
        &[],
        v(2.9, 1.1),
    )?;
    let front = kit.tex(
        stage,
        r"T_{\mathrm{front}}=T_{\mathrm{torso}}T_{\mathrm{rocker}}T_{\mathrm{spin}}",
        25.0,
        Palette::ink(),
        &[],
        v(2.9, -0.1),
    )?;
    parts.extend([rear, front]);
    show(stage, &parts)?;
    clear(kit, stage)?;

    kit.say(stage, "11_7")?;
    let title_card = title(kit, stage, "The balancer adds pitch at the axle")?;
    let advance = stage.add_value_tracker(0.0);
    let radius = 0.82;
    let travel = 0.25;
    let backward_pitch = 0.10;
    let base = stage.always_redraw(move |s| {
        let t = s.tracker_value(advance).unwrap_or(0.0);
        let axle = v(-3.6 + radius * travel * t, -1.12);
        let pitch = backward_pitch * t;
        let torso = v(-2.32 * pitch.sin(), 2.32 * pitch.cos());
        let roll = -travel * t;
        let spoke = v(0.80 * radius * roll.cos(), 0.80 * radius * roll.sin());
        s.add(v_group([
            ring(axle, radius, palette::steel()),
            stroke(vec![axle, add(axle, torso)], palette::rotation(), 11.0),
            stroke(vec![axle, add(axle, spoke)], palette::hero(), 4.7),
            dot(axle, Palette::ink(), 0.11),
        ]))
    });
    stage.add_to_scene(base)?;
    let rule = kit.tex(
        stage,
        r"\phi_{\mathrm{wheel\ relative}}=-x/r-\theta_{\mathrm{pitch}}",
        26.0,
        palette::hero(),
        &[],
        v(2.65, 1.04),
    )?;
    let answer = kit.tex(
        stage,
        &format!(r"-{travel:.2}-{backward_pitch:.2}={:.2}\ \text{{radians}}",
            balancer_wheel_relative(travel, backward_pitch)),
        27.0,
        palette::good(),
        &[],
        v(2.65, -0.16),
    )?;
    show(stage, &[title_card, rule])?;
    play!(stage; tween(advance,1.0,2.1)?);
    show(stage, &[answer])?;
    clear(kit, stage)?;

    kit.say(stage, "11_8")?;
    let title_card = title(kit, stage, "A mobile arm adds a chain to the chassis")?;
    let path = [
        v(-5.3, -1.05),
        v(-5.0, -0.31),
        v(-4.4, 0.44),
        v(-3.77, 0.70),
        v(-3.17, 0.61),
        v(-2.72, 1.03),
        v(-2.30, 1.42),
        v(-1.87, 1.32),
    ];
    let edges: Vec<_> = (0..7).map(|i| (i, i + 1)).collect();
    let graph = tree(stage, &path, &edges, &[0, 1, 2, 3, 4, 5, 6, 7]);
    let formula = kit.tex(
        stage,
        r"T_{\mathrm{hand}}=T_{\mathrm{base}}\,T_1\cdots T_7",
        32.0,
        palette::hero(),
        &[],
        v(2.7, 1.14),
    )?;
    let group = kit.tex(
        stage,
        r"\text{ideal body symmetry}:\ \{e\}",
        27.0,
        palette::symmetry(),
        &[],
        v(2.7, -0.25),
    )?;
    show(stage, &[title_card, graph, formula, group])?;
    clear(kit, stage)?;

    kit.say(stage, "11_9")?;
    let title_card = title(kit, stage, "Straight sliders give an additive group")?;
    let shape = tree(
        stage,
        &[v(-5.2, -1.1), v(-2.15, -1.1), v(-2.15, 1.25), v(-3.58, 0.48)],
        &[(0, 1), (1, 2), (2, 3)],
        &[0, 1, 2, 3],
    );
    let formula = kit.tex(
        stage,
        r"T(a)T(b)=T(a+b)=T(b)T(a)",
        32.0,
        palette::translation(),
        &[],
        v(2.78, 0.78),
    )?;
    let group = kit.tex(stage, r"a,b\in\mathbb{R}^3", 26.0, palette::hero(), &[], v(2.78, -0.45))?;
    show(stage, &[title_card, shape, formula, group])?;
    clear(kit, stage)?;

    kit.say(stage, "11_10")?;
    let title_card = title(kit, stage, "Three layers of the model")?;
    show(stage, &[title_card])?;
    for (i, (label, color)) in [
        ("Rigid transformations place the parts", palette::rotation()),
        ("Joints and contacts restrict motion", palette::translation()),
        ("Symmetry acts on the channels", palette::symmetry()),
    ]
    .into_iter()
    .enumerate()
    {
        let part = kit.text(stage, label, 29.0, color, v(0.0, 1.4 - i as f64 * 1.16))?;
        show(stage, &[part])?;
    }
    kit.finish(stage)
}
