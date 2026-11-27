//! Native picture for all thirteen cues of the rolling-constraints chapter.

use explainer_kit::{Kit, Palette, Timed, fade_all, play, tween, v};
use fmn::prelude::*;

use crate::model::{Omni3, Se2, track_lateral_velocities};
use crate::palette;

#[derive(Clone, Copy)]
enum Base {
    Differential,
    Omni,
    Tracks,
}

fn chassis(base: Base, pose: Se2, scale: f64) -> VMobject {
    let point = |x: f64, y: f64| {
        let p = pose.apply([scale * x, scale * y]);
        v(p[0], p[1])
    };
    let stroke = palette::line_style(palette::steel(), 4.0);
    let mut parts = Vec::new();
    match base {
        Base::Omni => {
            let mounts = [[1.0, 0.0], [-0.5, 3.0_f64.sqrt() / 2.0], [-0.5, -3.0_f64.sqrt() / 2.0]];
            parts.push(Polygon::new(mounts.map(|[x, y]| point(x, y))).style(stroke).build());
            for ([px, py], [tx, ty]) in mounts.into_iter().zip(Omni3::tangents()) {
                let ends = [
                    point(px - 0.24 * tx, py - 0.24 * ty),
                    point(px + 0.24 * tx, py + 0.24 * ty),
                ];
                parts.push(
                    polyline(ends.to_vec())
                        .style(palette::line_style(palette::velocity(), 12.0))
                        .build(),
                );
            }
        }
        Base::Differential | Base::Tracks => {
            parts.push(
                Polygon::new([
                    point(-0.63, -0.38),
                    point(0.63, -0.38),
                    point(0.63, 0.38),
                    point(-0.63, 0.38),
                ])
                .style(stroke)
                .build(),
            );
            let half_span = if matches!(base, Base::Tracks) { 0.69 } else { 0.23 };
            for y in [-0.54, 0.54] {
                parts.push(
                    polyline(vec![point(-half_span, y), point(half_span, y)])
                        .style(palette::line_style(palette::velocity(), 12.0))
                        .build(),
                );
            }
        }
    }
    parts.push(
        polyline(vec![point(0.0, 0.0), point(0.45, 0.0)])
            .style(palette::line_style(palette::rotation(), 4.0))
            .build(),
    );
    parts.push(
        Dot::new()
            .point(point(0.0, 0.0))
            .radius(0.055)
            .color(palette::hero())
            .build(),
    );
    v_group(parts)
}

fn draw_body(stage: &mut Stage<'_>, base: Base, x: f64, y: f64, scale: f64) -> Mob {
    stage.arena_mut().add(chassis(base, Se2::translation(x, y), scale))
}

fn arrow(stage: &mut Stage<'_>, from: Vec3, to: Vec3, color: Srgb) -> fmn::Result<Mob> {
    Ok(stage
        .arena_mut()
        .add(Arrow::new(from, to).thickness(4.0).color(color).build()?))
}

fn show(stage: &mut Stage<'_>, mobs: &[Mob], seconds: f64) -> fmn::Result<()> {
    let mut batch = Vec::new();
    for &mob in mobs {
        let fade = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?.rt(seconds);
        batch.push(stage.prepare(fade)?);
    }
    stage.play_prepared(batch)?;
    Ok(())
}

fn title(kit: &Kit, stage: &mut Stage<'_>, text: &str) -> fmn::Result<Mob> {
    kit.text(stage, text, 37.0, Palette::ink(), v(0.0, 3.4))
}

fn wipe(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    kit.hold(stage)?;
    fade_all(stage, 0.48)
}

pub fn construct(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    kit.say(stage, "04_1")?;
    let opening = title(kit, stage, "One pose, three ways to move")?;
    let mut set = vec![opening];
    let bases = [
        (-4.1, Base::Differential, "Differential drive"),
        (0.0, Base::Omni, "Omni triangle"),
        (4.1, Base::Tracks, "Rigid tracks"),
    ];
    for (x, base, name) in bases {
        let frame = kit.text(stage, name, 25.0, Palette::ink(), v(x, 1.62))?;
        let pose = kit.tex(
            stage,
            r"(x,y,\theta)\in SE(2)",
            20.0,
            palette::rotation(),
            &[],
            v(x, -1.72),
        )?;
        set.extend([draw_body(stage, base, x, 0.0, 1.19), frame, pose]);
    }
    show(stage, &set, 0.75)?;
    kit.say(stage, "04_2")?;
    let prompt = kit.text(
        stage,
        "Which instantaneous velocity is available?",
        29.0,
        palette::hero(),
        v(0.0, -2.77),
    )?;
    show(stage, &[prompt], 0.5)?;
    wipe(kit, stage)?;

    kit.say(stage, "04_3")?;
    let diff = title(kit, stage, "Fixed wheels")?;
    let clock = stage.add_value_tracker(0.0);
    let body = stage.always_redraw(move |s| {
        let turn = s.tracker_value(clock).unwrap_or(0.0);
        let pose = Se2::translation(-3.3, 0.05).compose(Se2::rotation(turn));
        s.add(chassis(Base::Differential, pose, 1.8))
    });
    stage.add_to_scene(body)?;
    let forward = arrow(stage, v(-3.22, 0.08), v(-0.92, 0.08), palette::translation())?;
    let cross = arrow(stage, v(-3.29, 0.2), v(-3.29, 1.98), palette::warn())?;
    let label = kit.text(
        stage,
        "Sideways speed at the axle = 0",
        26.0,
        palette::warn(),
        v(-3.25, -1.58),
    )?;
    show(stage, &[diff, forward, cross, label], 0.55)?;

    kit.say(stage, "04_4")?;
    let normal = kit.tex(
        stage,
        r"-\sin\theta\,\dot x+\cos\theta\,\dot y=0",
        31.0,
        palette::velocity(),
        &[],
        v(2.9, 1.16),
    )?;
    show(stage, &[normal], 0.55)?;
    let forward_out = fade_out(stage.arena_mut(), forward, ORIGIN, 1.0)?.rt(0.25);
    let cross_out = fade_out(stage.arena_mut(), cross, ORIGIN, 1.0)?.rt(0.25);
    play!(stage; forward_out, cross_out);
    play!(stage; tween(clock,0.66,1.4)?);
    let degrees = kit.text(
        stage,
        "Two controls   ·   forward speed and yaw",
        23.0,
        Palette::ink(),
        v(2.78, 0.35),
    )?;
    show(stage, &[degrees], 0.4)?;
    wipe(kit, stage)?;

    kit.say(stage, "04_5")?;
    let omni = title(kit, stage, "Three rolling directions")?;
    let center = [-3.55, -0.05];
    let scale = 1.65;
    let triangle = draw_body(stage, Base::Omni, center[0], center[1], scale);
    show(stage, &[omni, triangle], 0.6)?;
    let mut directions = Vec::new();
    let mut mounts = Vec::new();
    for [tx, ty] in Omni3::tangents() {
        let from = v(center[0] + scale * ty, center[1] - scale * tx);
        directions.push(arrow(stage, from, v(from[0] + tx, from[1] + ty), palette::velocity())?);
        mounts.push(from);
    }
    show(stage, &directions, 0.5)?;
    kit.say(stage, "04_6")?;
    let projection = kit.tex(
        stage,
        r"u_i=t_i\cdot(v_x,v_y)+\omega",
        34.0,
        palette::translation(),
        &[],
        v(2.7, 1.75),
    )?;
    let note = kit.text(
        stage,
        "Rolling tangent plus rotation (unit radius)",
        23.0,
        Palette::ink(),
        v(2.7, 1.05),
    )?;
    show(stage, &[projection, note], 0.45)?;
    kit.say(stage, "04_7")?;
    let command = kit.tex(
        stage,
        r"(v_x,v_y,\omega)=(0,2,0)",
        31.0,
        palette::translation(),
        &[],
        v(2.72, 0.14),
    )?;
    let slide = arrow(
        stage,
        v(center[0], center[1] + 0.04),
        v(center[0], center[1] + 1.38),
        palette::translation(),
    )?;
    show(stage, &[command, slide], 0.55)?;
    let question = explainer_kit::props::predict_pause(kit, stage, v(2.75, -1.15))?;
    kit.say(stage, "04_8")?;
    let dismiss = fade_out(stage.arena_mut(), question, ORIGIN, 1.0)?.rt(0.3);
    stage.play(dismiss)?;
    let speeds = Omni3::rim_speeds(0.0, 2.0, 0.0);
    let mut answers = Vec::new();
    for (i, (&speed, [tx, ty])) in speeds.iter().zip(Omni3::tangents()).enumerate() {
        let from = mounts[i];
        let to = v(from[0] + 0.6 * speed * tx, from[1] + 0.6 * speed * ty);
        let motion = arrow(stage, from, to, palette::translation())?;
        let sign = kit.text(
            stage,
            &format!("{speed:+.0}"),
            28.0,
            palette::translation(),
            v(to[0], to[1] + 0.34),
        )?;
        answers.extend([motion, sign]);
    }
    show(stage, &answers, 0.5)?;
    kit.say(stage, "04_9")?;
    let where_to = kit.tex(
        stage,
        r"M=\begin{pmatrix}0&1&1\\-\sqrt{3}/2&-1/2&1\\\sqrt{3}/2&-1/2&1\end{pmatrix}",
        27.0,
        Palette::ink(),
        &[],
        v(2.7, -1.30),
    )?;
    let det = kit.text(
        stage,
        &format!("det M ≈ {:.2}   ·   unique rim speeds", Omni3::unscaled_determinant()),
        23.0,
        palette::good(),
        v(2.7, -2.56),
    )?;
    show(stage, &[where_to, det], 0.64)?;
    wipe(kit, stage)?;

    kit.say(stage, "04_10")?;
    let tracks = title(kit, stage, "A rigid belt has two contact constraints")?;
    let tractor = draw_body(stage, Base::Tracks, -3.4, 0.0, 2.1);
    let a = 0.5;
    let contact_x = [-3.4 + 2.1 * a, -3.4 - 2.1 * a];
    let contact_y = -1.1;
    let mut dots = vec![tracks, tractor];
    for x in contact_x {
        dots.push(
            stage.arena_mut().add(
                Dot::new()
                    .point(v(x, contact_y))
                    .radius(0.10)
                    .color(palette::hero())
                    .build(),
            ),
        );
    }
    show(stage, &dots, 0.6)?;

    kit.say(stage, "04_11")?;
    let contact = track_lateral_velocities(0.0, 1.0, a);
    let mut slips = Vec::new();
    for (x, speed) in contact_x.into_iter().zip(contact) {
        let displacement = 1.5 * speed;
        let arrow = arrow(stage, v(x, contact_y), v(x, contact_y + displacement), palette::warn())?;
        let sign = kit.text(
            stage,
            &format!("{speed:+.1}"),
            25.0,
            palette::warn(),
            v(x + 0.78 * speed.signum(), contact_y + displacement),
        )?;
        slips.extend([arrow, sign]);
    }
    show(stage, &slips, 0.55)?;
    kit.say(stage, "04_12")?;
    let constraint = kit.tex(
        stage,
        r"\begin{pmatrix}1&1/2\\1&-1/2\end{pmatrix}\binom{v_y}{\omega}=\binom{0}{0}",
        31.0,
        Palette::ink(),
        &[],
        v(2.95, 0.5),
    )?;
    let no_turn = kit.tex(
        stage,
        r"\det=-1 \qquad v_y=\omega=0",
        27.0,
        palette::warn(),
        &[],
        v(2.95, -0.65),
    )?;
    show(stage, &[constraint, no_turn], 0.55)?;
    kit.say(stage, "04_13")?;
    let hinges = kit.text(
        stage,
        "Four flipper hinges add four joint angles",
        22.0,
        Palette::ink(),
        v(3.2, -1.10),
    )?;
    let outcome = kit.text(
        stage,
        "Model the slip to model the turn",
        25.0,
        palette::hero(),
        v(0.0, -2.77),
    )?;
    show(stage, &[hinges, outcome], 0.45)?;
    wipe(kit, stage)?;
    kit.finish(stage)
}
