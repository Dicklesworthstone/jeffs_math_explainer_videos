//! Three wheel mechanisms described by different maps from the same chassis twist.

use explainer_kit::{Facts, Kit, Palette, Timed, add, fade_all, play, tween, v};
use fmn::prelude::*;

use crate::palette;

const MECANUM_WHEELS: [(f64, f64, f64); 4] = [
    (0.25, -0.2, 1.0),
    (0.25, 0.2, -1.0),
    (-0.25, -0.2, -1.0),
    (-0.25, 0.2, 1.0),
];
const SWERVE_WHEELS: [(f64, f64); 4] = [(0.2, -0.2), (0.2, 0.2), (-0.2, -0.2), (-0.2, 0.2)];
const WHEELBASE: f64 = 0.50;
const HALF_AXLE: f64 = 0.14;

fn steering_angle(offset: f64) -> f64 {
    (WHEELBASE / (1.0 - offset)).atan()
}

fn point_velocity([vx, vz, yaw]: [f64; 3], x: f64, z: f64) -> (f64, f64) {
    (vx + yaw * z, vz - yaw * x)
}

fn mecanum_matrix() -> [[f64; 3]; 4] {
    MECANUM_WHEELS.map(|(x, z, tangent)| [-1.0, -tangent, -z + tangent * x])
}

pub fn facts() -> Result<Facts, String> {
    let mut facts = crate::facts::facts()?;
    let reference = "frankenrobotics-gallery01/scene-facts01.json";
    let inner = steering_angle(HALF_AXLE).to_degrees();
    let outer = steering_angle(-HALF_AXLE).to_degrees();
    facts.pin(
        "wheel_gallery.ackermann_ideal_angles",
        format!("{inner:.1},{outer:.1}"),
        "30.2,23.7",
        reference,
    )?;
    let rows = mecanum_matrix();
    let rates = rows.map(|row| format!("{:+.1}", row[1] * 0.32 / 0.10));
    facts.pin(
        "wheel_gallery.mecanum_side_roll_rates",
        rates.join(","),
        "-3.2,+3.2,+3.2,-3.2",
        reference,
    )?;
    let [a, b, c, d] = rows;
    let minor =
        a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0]) + a[2] * (b[0] * c[1] - b[1] * c[0]);
    facts.pin("wheel_gallery.mecanum_minor", format!("{minor:.2}"), "-1.80", reference)?;
    for i in 0..3 {
        let defect = (a[i] + b[i] - c[i] - d[i]).abs();
        if defect > 1e-12 {
            return Err(format!("Mecanum compatibility failed in column {i}: {defect}"));
        }
    }
    let (vx, vz) = point_velocity([0.0, 0.0, 1.0], 0.2, -0.2);
    facts.pin(
        "wheel_gallery.swerve_corner_spin",
        format!("{:.2}", vx.hypot(vz) / 0.075),
        "3.77",
        reference,
    )?;
    Ok(facts)
}

fn line(points: Vec<Vec3>, color: Srgb, width: f64) -> VMobject {
    polyline(points).style(palette::line_style(color, width)).build()
}

fn tire(center: Vec3, steer: f64, color: Srgb) -> VMobject {
    let forward = v(0.32 * steer.sin(), 0.32 * steer.cos());
    let normal = v(0.11 * steer.cos(), -0.11 * steer.sin());
    let front = add(center, forward);
    let rear = add(center, forward.map(|x| -x));
    let opposite = normal.map(|x| -x);
    Polygon::new([
        add(front, normal),
        add(front, opposite),
        add(rear, opposite),
        add(rear, normal),
    ])
    .style(palette::flat(color, 1.0))
    .build()
}

fn add_dot(stage: &mut Stage<'_>, at: Vec3, color: Srgb) -> Mob {
    stage
        .arena_mut()
        .add(Dot::new().point(at).radius(0.11).color(color).build())
}

fn show(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut animations = Vec::new();
    for &mob in mobs {
        let input = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?.rt(0.52);
        animations.push(stage.prepare(input)?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

fn heading(kit: &Kit, stage: &mut Stage<'_>, label: &str) -> fmn::Result<Mob> {
    kit.text(stage, label, 37.0, Palette::ink(), v(0.0, 3.4))
}

fn clean(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    kit.hold(stage)?;
    fade_all(stage, 0.5)
}

fn chassis(stage: &mut Stage<'_>, center: Vec3, w: f64, h: f64) -> fmn::Result<Mob> {
    let outline = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.16)
        .style(palette::line_style(palette::steel(), 2.6))
        .build()?
        .moved_to(center);
    Ok(stage.arena_mut().add(outline))
}

pub fn construct(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    kit.say(stage, "10_1")?;
    let title = heading(kit, stage, "Four drives share one pose group")?;
    let pose = kit.tex(
        stage,
        r"T=\begin{pmatrix}R&p\\0&1\end{pmatrix}\in SE(2)",
        37.0,
        palette::hero(),
        &[],
        v(0.0, 0.75),
    )?;
    let labels = kit.text(
        stage,
        "Differential    ·    Ackermann    ·    Mecanum    ·    Swerve",
        26.0,
        Palette::ink(),
        v(0.0, -1.1),
    )?;
    show(stage, &[title, pose, labels])?;
    clean(kit, stage)?;

    kit.say(stage, "10_2")?;
    let title = heading(kit, stage, "One steering input sets two angles")?;
    let back = v(-4.4, -1.04);
    let center = add(back, v(4.1, 0.0));
    let scale = 4.1;
    let outline = chassis(stage, add(back, v(0.0, scale * WHEELBASE * 0.5)), 1.03, 2.28)?;
    let pivot = add_dot(stage, center, palette::hero());
    let axle = stage
        .arena_mut()
        .add(line(vec![add(back, v(-0.88, 0.0)), center], palette::steel(), 2.0));
    let mut marked = vec![title, outline, pivot, axle];
    let mut angles = Vec::new();
    for (offset, color) in [(HALF_AXLE, palette::rotation()), (-HALF_AXLE, palette::translation())] {
        let rear = add(back, v(scale * offset, 0.0));
        let front = add(rear, v(0.0, scale * WHEELBASE));
        let steer = steering_angle(offset);
        angles.push(steer.to_degrees());
        marked.push(stage.arena_mut().add(tire(rear, 0.0, palette::steel())));
        marked.push(stage.arena_mut().add(tire(front, steer, color)));
        marked.push(
            stage
                .arena_mut()
                .add(line(vec![front, rear, center, front], color, 2.45)),
        );
    }
    show(stage, &marked)?;

    kit.say(stage, "10_3")?;
    let names = ["Inner wheel", "Outer wheel"];
    let offsets = [0.86, 1.14];
    for (i, &angle) in angles.iter().enumerate() {
        let y = 1.05 - i as f64 * 1.42;
        let color = if i == 0 {
            palette::rotation()
        } else {
            palette::translation()
        };
        let name = kit.text(stage, names[i], 22.0, color, v(3.35, y + 0.43))?;
        let equation = kit.tex(
            stage,
            &format!(
                r"\tan\delta=\frac{{0.50}}{{{:.2}}}\quad\Rightarrow\quad \delta\approx{angle:.1}^\circ",
                offsets[i]
            ),
            25.0,
            color,
            &[],
            v(3.35, y - 0.2),
        )?;
        show(stage, &[name, equation])?;
    }
    clean(kit, stage)?;

    kit.say(stage, "10_4")?;
    let title = heading(kit, stage, "Fixed rollers make a linear map")?;
    let body = v(-3.8, -0.2);
    let outline = chassis(stage, body, 2.17, 3.02)?;
    let mut parts = vec![title, outline];
    let mut mounts = Vec::new();
    for (x, z, lean) in MECANUM_WHEELS {
        let point = add(body, v(6.6 * z, 5.6 * x));
        let slope = v(lean * 0.18, 0.19);
        parts.push(stage.arena_mut().add(tire(point, 0.0, palette::steel())));
        parts.push(stage.arena_mut().add(line(
            vec![add(point, slope.map(|x| -x)), add(point, slope)],
            palette::rotation(),
            4.1,
        )));
        mounts.push(point);
    }
    let matrix = mecanum_matrix();
    let rows = matrix.map(|[vx, vz, w]| format!("{vx:.0}&{vz:.0}&{w:.2}"));
    let equation = format!(
        r"r\,\dot{{\phi}}=\begin{{pmatrix}}{}\end{{pmatrix}}\begin{{pmatrix}}v_x\\v_z\\\omega\end{{pmatrix}}",
        rows.join(r"\\")
    );
    let motor_map = kit.tex(stage, &equation, 26.0, palette::hero(), &[], v(2.86, 0.9))?;
    parts.push(motor_map);
    show(stage, &parts)?;

    kit.say(stage, "10_5")?;
    let sideways = Arrow::new(add(body, v(-0.72, 0.0)), add(body, v(0.83, 0.0)))
        .color(palette::translation())
        .thickness(5.0)
        .build()?;
    let arrow = stage.arena_mut().add(sideways);
    let text = kit.tex(
        stage,
        r"v_z=0.32\,\mathrm{m/s},\quad r=0.10\,\mathrm m",
        24.0,
        palette::translation(),
        &[],
        v(2.9, -0.42),
    )?;
    let mut rates = vec![arrow, text];
    for (i, point) in mounts.into_iter().enumerate() {
        let rate = matrix[i][1] * 0.32 / 0.1;
        let away = if i.is_multiple_of(2) { -0.65 } else { 0.65 };
        rates.push(kit.text(
            stage,
            &format!("{rate:+.1}"),
            23.0,
            palette::velocity(),
            add(point, v(away, 0.51)),
        )?);
    }
    show(stage, &rates)?;

    kit.say(stage, "10_6")?;
    let relation = kit.tex(
        stage,
        r"\dot\phi_{FL}+\dot\phi_{FR}=\dot\phi_{RL}+\dot\phi_{RR}",
        26.0,
        palette::good(),
        &[],
        v(2.86, -1.32),
    )?;
    let rank = kit.text(
        stage,
        "Three independent chassis speeds",
        22.0,
        Palette::ink(),
        v(2.86, -2.16),
    )?;
    show(stage, &[relation, rank])?;
    clean(kit, stage)?;

    kit.say(stage, "10_7")?;
    let title = heading(kit, stage, "Swerve sends each module along its own arrow")?;
    let body = v(-3.4, 0.08);
    let outline = chassis(stage, body, 2.80, 2.80)?;
    let yaw = stage.add_value_tracker(0.0);
    let modules = stage.always_redraw(move |s| {
        let speed = s.tracker_value(yaw).unwrap_or(0.0);
        let mut pieces = Vec::new();
        for (x, z) in SWERVE_WHEELS {
            let center = add(body, v(6.0 * z, 6.0 * x));
            let (vx, vz) = point_velocity([0.45 * (1.0 - speed), 0.0, speed], x, z);
            pieces.push(tire(center, vz.atan2(vx), palette::steel()));
            if vx.hypot(vz) > 0.002 {
                let end = add(center, v(3.0 * vz, 3.0 * vx));
                let arrow = Arrow::new(center, end)
                    .color(palette::translation())
                    .thickness(3.5)
                    .build()
                    .expect("nonzero module velocity");
                pieces.push(arrow);
            }
        }
        s.add(v_group(pieces))
    });
    stage.add_to_scene(modules)?;
    let rule = kit.tex(
        stage,
        r"(v_x+\omega z,\;v_z-\omega x)",
        33.0,
        palette::translation(),
        &[],
        v(2.56, 1.08),
    )?;
    show(stage, &[title, outline, rule])?;

    kit.say(stage, "10_8")?;
    play!(stage; tween(yaw, 1.0, 2.5)?);
    let (vx, vz) = point_velocity([0.0, 0.0, 1.0], 0.2, -0.2);
    let speed = vx.hypot(vz);
    let anchor = add_dot(stage, add(body, v(-1.2, 1.2)), palette::hero());
    let components = kit.tex(
        stage,
        &format!(r"V=({vx:.2},{vz:.2})\,\mathrm{{m/s}}"),
        26.0,
        palette::hero(),
        &[],
        v(2.56, 0.03),
    )?;
    let spin = kit.tex(
        stage,
        &format!(
            r"|\dot\phi|=\frac{{0.20\sqrt{{2}}}}{{0.075}}\approx{:.2}\,\mathrm{{rad/s}}",
            speed / 0.075
        ),
        25.0,
        palette::good(),
        &[],
        v(2.56, -1.09),
    )?;
    show(stage, &[anchor, components, spin])?;
    clean(kit, stage)?;

    kit.say(stage, "10_9")?;
    let title = heading(kit, stage, "Different mechanisms, one SE(2) pose")?;
    show(stage, &[title])?;
    for (i, (name, behavior)) in [
        ("Differential", "Forward and turn, zero lateral axle speed"),
        ("Ackermann", "Steering linkage couples forward speed and yaw"),
        ("Mecanum", "Fixed projections along diagonal rollers"),
        ("Swerve", "Set module steering from its local velocity"),
    ]
    .into_iter()
    .enumerate()
    {
        let y = 1.65 - i as f64 * 1.07;
        let kind = kit.text_left(stage, name, 25.0, palette::hero(), v(-5.95, y))?;
        let line = kit.text_left(stage, behavior, 23.0, Palette::ink(), v(-2.2, y))?;
        show(stage, &[kind, line])?;
    }
    kit.finish(stage)
}
