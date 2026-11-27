//! Start from the group operations, then put those operations on a body.

use std::f64::consts::{FRAC_PI_2, TAU};

use explainer_kit::{Kit, Palette, Timed, add, fade_all, lerp, play, tween, v};
use fmn::prelude::*;

use crate::model::{Arm2R, Se2, hexapod_d6_elements};
use crate::palette;

pub(super) fn reveal(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut fades = Vec::new();
    for &mob in mobs {
        let fade = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?.rt(0.42);
        fades.push(stage.prepare(fade)?);
    }
    stage.play_prepared(fades)?;
    Ok(())
}

pub(super) fn stroke(points: Vec<Vec3>, color: Srgb, width: f64) -> VMobject {
    polyline(points).style(palette::line_style(color, width)).build()
}

pub(super) fn dot(point: Vec3, color: Srgb, radius: f64) -> VMobject {
    Dot::new().point(point).color(color).radius(radius).build()
}

pub(super) fn place(origin: Vec3, scale: f64, [x, y]: [f64; 2]) -> Vec3 {
    add(origin, v(scale * x, scale * y))
}

pub(super) fn heading(kit: &Kit, stage: &mut Stage<'_>, words: &str) -> fmn::Result<()> {
    let title = kit.text(stage, words, 37.0, Palette::ink(), v(0.0, 3.35))?;
    reveal(stage, &[title])
}

fn caption(kit: &Kit, stage: &mut Stage<'_>, words: &str, x: f64, y: f64) -> fmn::Result<()> {
    let label = kit.text(stage, words, 25.0, Palette::ink(), v(x, y))?;
    reveal(stage, &[label])
}

pub(super) fn pose(centre: Vec3, scale: f64, g: Se2) -> VMobject {
    let p = |point| place(centre, scale, g.apply(point));
    let hull = Polygon::new([p([-0.25, -0.19]), p([0.25, -0.19]), p([0.25, 0.19]), p([-0.25, 0.19])])
        .style(palette::flat(palette::steel(), 0.85))
        .build();
    let axle = stroke(vec![p([0.0, -0.34]), p([0.0, 0.34])], palette::rotation(), 4.0);
    let nose = stroke(vec![p([0.11, 0.0]), p([0.64, 0.0])], palette::translation(), 5.0);
    let tip = dot(p([0.64, 0.0]), palette::translation(), 0.055);
    v_group(vec![hull, axle, nose, tip])
}

fn arm(centre: Vec3, shoulder: f64, elbow: f64) -> VMobject {
    let at = |p| place(centre, 0.91, p);
    let base = at([0.0, 0.0]);
    let middle = at(Arm2R::UNIT.elbow(shoulder));
    let hand = at(Arm2R::UNIT.hand(shoulder, elbow));
    v_group(vec![
        stroke(vec![base, middle], palette::rotation(), 8.0),
        stroke(vec![middle, hand], palette::translation(), 8.0),
        dot(base, palette::steel(), 0.10),
        dot(middle, palette::steel(), 0.09),
        dot(hand, palette::hero(), 0.09),
    ])
}

fn hexagon(centre: Vec3, n: usize, angle: f64, mirror_angle: f64, mirror_progress: f64) -> VMobject {
    let point = |index: usize| {
        let a = TAU * index as f64 / n as f64;
        add(centre, v(a.cos(), a.sin()))
    };
    let active = add(
        centre,
        v(
            lerp(angle.cos(), mirror_angle.cos(), mirror_progress),
            lerp(angle.sin(), mirror_angle.sin(), mirror_progress),
        ),
    );
    let mut limbs = vec![dot(centre, palette::steel(), 0.16)];
    for i in 0..n {
        limbs.push(stroke(vec![centre, point(i)], palette::steel(), 4.0));
        limbs.push(dot(point(i), palette::steel(), 0.055));
    }
    limbs.push(dot(active, palette::symmetry(), 0.16));
    v_group(limbs)
}

fn orbit(centre: Vec3, radius: f64) -> Vec<Vec3> {
    (0..=80)
        .map(|i| {
            let a = TAU * f64::from(i) / 80.0;
            add(centre, v(radius * a.cos(), radius * a.sin()))
        })
        .collect()
}

pub fn opening(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    kit.say(stage, "01_1")?;
    heading(kit, stage, "Three robots. Three ways to multiply.")?;
    let arm_centre = v(-5.38, 0.16);
    let rover_centre = v(-0.67, -0.53);
    let hex_centre = v(4.32, 0.35);
    let shoulder = stage.add_value_tracker(-0.30);
    let elbow = stage.add_value_tracker(0.2);
    let arm_view = stage.always_redraw(move |scene| {
        let q1 = scene.tracker_value(shoulder).unwrap_or(0.0);
        let q2 = scene.tracker_value(elbow).unwrap_or(0.0);
        scene.add(arm(arm_centre, q1, q2))
    });
    stage.add_to_scene(arm_view)?;
    caption(kit, stage, "Arm", -4.35, 2.29)?;
    play!(stage; tween(shoulder, 0.0, 1.0)?, tween(elbow, FRAC_PI_2, 1.0)?);

    kit.say(stage, "01_2")?;
    caption(kit, stage, "Rover", 0.0, 2.29)?;
    let rover = stage.add_value_tracker(0.0);
    let rover_view = stage.always_redraw(move |scene| {
        let u = scene.tracker_value(rover).unwrap_or(0.0);
        let g = Se2::rotation(u.min(1.0) * FRAC_PI_2).compose(Se2::translation((u - 1.0).max(0.0), 0.0));
        scene.add(pose(rover_centre, 1.03, g))
    });
    stage.add_to_scene(rover_view)?;
    play!(stage; tween(rover, 1.0, 0.90)?);
    play!(stage; tween(rover, 2.0, 0.90)?);
    let other = Se2::translation(1.0, 0.0).compose(Se2::rotation(FRAC_PI_2));
    let alternative = stage.arena_mut().add(pose(rover_centre, 1.03, other));
    reveal(stage, &[alternative])?;

    kit.say(stage, "01_3")?;
    caption(kit, stage, "Hexapod", 4.32, 2.29)?;
    let elements = hexapod_d6_elements();
    let count = elements[0].len();
    let slot_angle = TAU / count as f64;
    let starting_slot = 1;
    let rotation = elements[2][starting_slot] as f64 * slot_angle;
    let reflection = elements[3][starting_slot] as f64 * slot_angle;
    let marked = stage.add_value_tracker(slot_angle);
    let flip = stage.add_value_tracker(0.0);
    let hex_view = stage.always_redraw(move |scene| {
        let angle = scene.tracker_value(marked).unwrap_or(slot_angle);
        let progress = scene.tracker_value(flip).unwrap_or(0.0);
        scene.add(hexagon(hex_centre, count, angle, reflection, progress))
    });
    stage.add_to_scene(hex_view)?;
    play!(stage; tween(marked, rotation, 1.15)?);
    let mirror = stage.arena_mut().add(stroke(
        vec![add(hex_centre, v(-1.30, 0.0)), add(hex_centre, v(1.30, 0.0))],
        palette::symmetry(),
        2.1,
    ));
    reveal(stage, &[mirror])?;
    play!(stage; tween(flip, 1.0, 1.25)?);

    kit.say(stage, "01_4")?;
    let groups = [
        kit.tex(
            stage,
            r"SO(2)\times SO(2)",
            32.0,
            palette::rotation(),
            &[],
            v(-4.35, -1.22),
        )?,
        kit.tex(stage, r"SE(2)", 32.0, palette::translation(), &[], v(0.0, -1.22))?,
        kit.tex(stage, r"D_6", 32.0, palette::symmetry(), &[], v(4.32, -1.22))?,
    ];
    reveal(stage, &groups)?;
    caption(kit, stage, "Turns two joint circles", -4.35, -1.91)?;
    caption(kit, stage, "Moves a floor frame", 0.0, -1.91)?;
    caption(kit, stage, "Permutes leg commands", 4.32, -1.91)?;

    kit.say(stage, "01_5")?;
    let composition = kit.tex(
        stage,
        r"\rho(gh)=\rho(g)\rho(h)",
        34.0,
        Palette::ink(),
        &[],
        v(0.0, -2.73),
    )?;
    reveal(stage, &[composition])?;
    caption(
        kit,
        stage,
        "Composition in the group becomes matrix multiplication",
        0.0,
        -3.34,
    )?;
    kit.hold(stage)?;
    fade_all(stage, 0.48)?;

    kit.say(stage, "01_6")?;
    heading(kit, stage, "A hinge and a slider")?;
    let hinge = v(-3.31, 0.36);
    let rail = v(3.10, 0.36);
    let radius = 1.40;
    let circle = stroke(orbit(hinge, radius), palette::rotation(), 3.6);
    let rail_line = stroke(
        vec![add(rail, v(-2.42, 0.0)), add(rail, v(2.42, 0.0))],
        palette::translation(),
        3.6,
    );
    let hinge_mark = stage.add_value_tracker(0.0);
    let slide = stage.add_value_tracker(0.0);
    let turn_view = stage.always_redraw(move |scene| {
        let theta = scene.tracker_value(hinge_mark).unwrap_or(0.0);
        let point = place(hinge, 1.0, Se2::rotation(theta).apply([radius, 0.0]));
        scene.add(v_group(vec![
            stroke(vec![hinge, point], palette::steel(), 5.0),
            dot(point, palette::hero(), 0.115),
        ]))
    });
    let slide_view = stage.always_redraw(move |scene| {
        let x = scene.tracker_value(slide).unwrap_or(0.0);
        scene.add(dot(add(rail, v(x, 0.0)), palette::hero(), 0.115))
    });
    let line_mobs = [stage.arena_mut().add(circle), stage.arena_mut().add(rail_line)];
    reveal(stage, &line_mobs)?;
    stage.add_to_scene(turn_view)?;
    stage.add_to_scene(slide_view)?;

    kit.say(stage, "01_7")?;
    play!(stage; tween(hinge_mark, FRAC_PI_2, 0.95)?, tween(slide, 0.72, 0.95)?);
    play!(stage; tween(hinge_mark, 2.0 * FRAC_PI_2, 0.95)?, tween(slide, 1.44, 0.95)?);
    let sum = kit.tex(
        stage,
        r"\frac14+\frac14=\frac12\ \mathrm{turn}",
        29.0,
        palette::rotation(),
        &[],
        v(-3.31, -1.64),
    )?;
    reveal(stage, &[sum])?;

    kit.say(stage, "01_8")?;
    play!(stage; tween(hinge_mark, FRAC_PI_2, 0.95)?, tween(slide, 0.72, 0.95)?);
    caption(kit, stage, "The inverse undoes a move", 0.0, -2.65)?;

    kit.say(stage, "01_9")?;
    let question = kit.text(
        stage,
        "How far forward to get back to zero?",
        26.0,
        palette::quiz(),
        v(0.0, -3.25),
    )?;
    reveal(stage, &[question])?;
    kit.listen(stage, 4.0, "hinge_inverse_forward")?;

    kit.say(stage, "01_10")?;
    let remove_question = fade_out(stage.arena_mut(), question, ORIGIN, 1.0)?.rt(0.32);
    stage.play(remove_question)?;
    play!(stage; tween(hinge_mark, TAU, 2.0)?);
    let wrap = kit.tex(
        stage,
        r"\frac14+\frac34=1\equiv 0\ \mathrm{turns}",
        29.0,
        palette::hero(),
        &[],
        v(-3.31, -2.19),
    )?;
    reveal(stage, &[wrap])?;

    kit.say(stage, "01_11")?;
    let two = [
        kit.tex(stage, r"SO(2)", 32.0, palette::rotation(), &[], v(-3.31, 2.19))?,
        kit.tex(
            stage,
            r"(\mathbb R,+)",
            32.0,
            palette::translation(),
            &[],
            v(3.10, 2.19),
        )?,
    ];
    reveal(stage, &two)?;
    play!(stage; tween(slide, -1.62, 1.85)?);

    kit.say(stage, "01_12")?;
    caption(kit, stage, "Compose the parts to build the body", 0.0, -3.22)?;
    kit.finish(stage)
}

/// Path of each composition when moves are appended in the robot's local frame.
fn pair_pose(first_turn: bool, phase: f64) -> Se2 {
    let first = phase.clamp(0.0, 1.0);
    let second = (phase - 1.0).clamp(0.0, 1.0);
    if first_turn {
        Se2::rotation(FRAC_PI_2 * first).compose(Se2::translation(second, 0.0))
    } else {
        Se2::translation(first, 0.0).compose(Se2::rotation(FRAC_PI_2 * second))
    }
}

fn axes(stage: &mut Stage<'_>, origin: Vec3) -> Mob {
    let frames = [-1.0, 0.0, 1.0];
    let mut lines = Vec::new();
    for t in frames {
        lines.push(stroke(
            vec![add(origin, v(-1.0, t)), add(origin, v(2.0, t))],
            palette::panel(),
            2.5,
        ));
        lines.push(stroke(
            vec![add(origin, v(t, -1.0)), add(origin, v(t, 1.75))],
            palette::panel(),
            2.5,
        ));
    }
    lines.push(dot(origin, palette::steel(), 0.075));
    stage.arena_mut().add(v_group(lines))
}

fn matrix(g: Se2) -> String {
    let clean = |n: f64| if n.abs() < 1e-12 { 0.0 } else { n };
    let (s, c) = g.theta.sin_cos();
    let [c, s, ns, x, y] = [c, s, -s, g.x, g.y].map(clean);
    format!(r"\begin{{pmatrix}}{c:.0}&{ns:.0}&{x:.0}\\{s:.0}&{c:.0}&{y:.0}\\0&0&1\end{{pmatrix}}")
}

pub fn rigid_poses(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    kit.say(stage, "03_1")?;
    heading(kit, stage, "A frame moves with the metal")?;
    let origins = [v(-4.03, -0.45), v(1.70, -0.45)];
    let grids = [axes(stage, origins[0]), axes(stage, origins[1])];
    reveal(stage, &grids)?;
    let phases = [stage.add_value_tracker(0.0), stage.add_value_tracker(0.0)];
    for (i, (origin, phase)) in origins.into_iter().zip(phases).enumerate() {
        let view = stage.always_redraw(move |scene| {
            let u = scene.tracker_value(phase).unwrap_or(0.0);
            let g = pair_pose(i == 0, u);
            let translation = stroke(
                vec![origin, place(origin, 1.0, [g.x, g.y])],
                palette::translation(),
                3.0,
            );
            scene.add(v_group(vec![translation, pose(origin, 1.0, g)]))
        });
        stage.add_to_scene(view)?;
    }

    kit.say(stage, "03_2")?;
    caption(kit, stage, "R turns a quarter left. T steps forward.", 0.0, 2.56)?;
    kit.say(stage, "03_3")?;
    caption(
        kit,
        stage,
        "The blue axle and cyan nose turn with the frame",
        0.0,
        -2.30,
    )?;

    kit.say(stage, "03_4")?;
    caption(kit, stage, "Turn, then drive", -3.53, 1.69)?;
    play!(stage; tween(phases[0], 1.0, 1.10)?);
    play!(stage; tween(phases[0], 2.0, 1.10)?);
    let [rt, tr] = [pair_pose(true, 2.0), pair_pose(false, 2.0)];
    let left = kit.tex(
        stage,
        &format!(r"RT\to ({:.0},{:.0})", rt.x, rt.y),
        30.0,
        palette::hero(),
        &[],
        v(-3.53, -1.76),
    )?;
    reveal(stage, &[left])?;

    kit.say(stage, "03_5")?;
    caption(kit, stage, "Drive, then turn", 2.20, 1.69)?;
    play!(stage; tween(phases[1], 1.0, 1.10)?);
    kit.listen(stage, 4.0, "reverse_pose_order")?;

    kit.say(stage, "03_6")?;
    play!(stage; tween(phases[1], 2.0, 1.10)?);
    let right = kit.tex(
        stage,
        &format!(r"TR\to ({:.0},{:.0})", tr.x, tr.y),
        30.0,
        palette::hero(),
        &[],
        v(2.20, -1.76),
    )?;
    reveal(stage, &[right])?;
    kit.hold(stage)?;
    fade_all(stage, 0.45)?;

    kit.say(stage, "03_7")?;
    heading(kit, stage, "How a pose moves a point")?;
    let action = kit.tex(
        stage,
        r"\begin{pmatrix}R&p\\0&1\end{pmatrix}\binom{x}{1}=\binom{Rx+p}{1}",
        38.0,
        palette::steel(),
        &[],
        v(0.0, 1.47),
    )?;
    reveal(stage, &[action])?;

    kit.say(stage, "03_8")?;
    let product = kit.tex(stage, r"\begin{pmatrix}R&p\\0&1\end{pmatrix}\begin{pmatrix}S&q\\0&1\end{pmatrix}=\begin{pmatrix}RS&p+Rq\\0&1\end{pmatrix}", 35.0, palette::hero(), &[], v(0.0, -0.12))?;
    reveal(stage, &[product])?;
    caption(
        kit,
        stage,
        "The first rotation also turns the second displacement",
        0.0,
        -1.36,
    )?;
    kit.hold(stage)?;

    kit.say(stage, "03_9")?;
    fade_all(stage, 0.38)?;
    heading(kit, stage, "One product law. Two positions.")?;
    let products = [
        kit.tex(
            stage,
            &format!("RT={}", matrix(rt)),
            33.0,
            palette::rotation(),
            &[],
            v(-3.38, 1.0),
        )?,
        kit.tex(
            stage,
            &format!("TR={}", matrix(tr)),
            33.0,
            palette::translation(),
            &[],
            v(3.38, 1.0),
        )?,
    ];
    reveal(stage, &products)?;

    kit.say(stage, "03_10")?;
    let [x, y] = Arm2R::UNIT.hand(0.0, FRAC_PI_2);
    let fk = kit.tex(
        stage,
        &format!(r"R(q_1)T(1)R(q_2)T(1)\cdot (0,0)\ \longrightarrow\ ({x:.0},{y:.0})"),
        30.0,
        palette::hero(),
        &[],
        v(0.0, -0.99),
    )?;
    reveal(stage, &[fk])?;

    kit.say(stage, "03_11")?;
    let spatial = kit.tex(
        stage,
        r"SE(3):\quad R\in SO(3),\quad p\in\mathbb R^3,\quad \det(R)=+1",
        30.0,
        palette::steel(),
        &[],
        v(0.0, -2.41),
    )?;
    reveal(stage, &[spatial])?;
    kit.finish(stage)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_local_orderings_have_the_same_heading_and_distinct_positions() {
        let rt = pair_pose(true, 2.0);
        let tr = pair_pose(false, 2.0);
        assert!((rt.x.abs() + (rt.y - 1.0).abs()) < 1e-12);
        assert_eq!(
            tr,
            Se2 {
                theta: FRAC_PI_2,
                x: 1.0,
                y: 0.0
            }
        );
        assert_eq!(rt.theta, tr.theta);
        assert_ne!(matrix(rt), matrix(tr));
    }
}
