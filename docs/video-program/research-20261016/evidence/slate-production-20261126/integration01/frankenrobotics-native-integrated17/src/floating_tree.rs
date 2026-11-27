//! A floating quadruped and the exact point constraint on one branch.

use explainer_kit::{Kit, Palette, add, fade_all, play, tween, v};
use fmn::prelude::*;

use crate::{
    group_foundations::{dot, heading, reveal, stroke},
    palette,
};

const HIPS: [Vec3; 4] = [
    [1.02, 0.54, 0.0],
    [1.02, -0.54, 0.0],
    [-1.02, 0.54, 0.0],
    [-1.02, -0.54, 0.0],
];
const JOINTS_PER_LEG: usize = 3;
const POSE_DIMENSION: usize = 6;

fn roll([x, y, z]: Vec3, q: f64) -> Vec3 {
    let (s, c) = q.sin_cos();
    [x, c * y - s * z, s * y + c * z]
}

fn pitch([x, y, z]: Vec3, q: f64) -> Vec3 {
    let (s, c) = q.sin_cos();
    [c * x + s * z, y, -s * x + c * z]
}

fn yaw([x, y, z]: Vec3, q: f64) -> Vec3 {
    let (s, c) = q.sin_cos();
    [c * x - s * y, s * x + c * y, z]
}

/// Idealized hinge chain about a forward hip axis and two lateral axes in its child frame.
fn leg(hip: Vec3, [q_roll, q_hip, q_knee]: Vec3) -> [Vec3; 3] {
    let upper = roll(pitch([0.0, 0.0, -0.77], q_hip), q_roll);
    let lower = roll(pitch([0.0, 0.0, -0.77], q_hip + q_knee), q_roll);
    let knee = add(hip, upper);
    [hip, knee, add(knee, lower)]
}

fn local_legs(active_leg: usize, active_roll: f64) -> [[Vec3; 3]; 4] {
    std::array::from_fn(|i| {
        let active = i == active_leg;
        let relative = if active {
            [active_roll, -0.31, 0.62]
        } else {
            [0.0, -0.63, 1.26]
        };
        leg(HIPS[i], relative)
    })
}

fn pin_translation(local_foot: Vec3, heading: f64, target: Vec3) -> Vec3 {
    let rotated = yaw(local_foot, heading);
    std::array::from_fn(|i| target[i] - rotated[i])
}

fn project([x, y, z]: Vec3) -> Vec3 {
    v(-3.25 + 0.92 * x - 0.83 * y, -1.82 + 0.26 * x + 0.42 * y + 1.34 * z)
}

fn dog(world: Vec3, yaw_angle: f64, legs: [[Vec3; 3]; 4], active_leg: usize) -> VMobject {
    let to_screen = |p| project(add(world, yaw(p, yaw_angle)));
    let hull = Polygon::new([HIPS[0], HIPS[1], HIPS[3], HIPS[2]].map(to_screen))
        .style(palette::flat(palette::panel(), 1.0))
        .build();
    let mut geometry = Vec::new();
    for i in [2, 3, 1, 0] {
        let points = legs[i].map(to_screen);
        let color = if i == active_leg {
            palette::hero()
        } else {
            palette::steel()
        };
        geometry.push(stroke(points.to_vec(), color, 6.5));
        geometry.push(dot(points[2], color, 0.10));
        geometry.push(dot(points[1], palette::rotation(), 0.075));
    }
    geometry.push(hull);
    for (a, b) in [(0, 1), (1, 3), (3, 2), (2, 0)] {
        geometry.push(stroke(
            vec![to_screen(HIPS[a]), to_screen(HIPS[b])],
            palette::steel(),
            3.6,
        ));
    }
    geometry.push(stroke(
        vec![to_screen([0.0, 0.0, 0.0]), to_screen([0.69, 0.0, 0.0])],
        palette::translation(),
        4.0,
    ));
    v_group(geometry)
}

fn ground_grid(stage: &mut Stage<'_>) -> Mob {
    let mut floor = Vec::new();
    for i in -2..=2 {
        let a = f64::from(i) * 0.64;
        floor.push(stroke(
            vec![project([a, -1.35, 0.0]), project([a, 1.35, 0.0])],
            palette::panel(),
            2.0,
        ));
        floor.push(stroke(
            vec![project([-1.35, a, 0.0]), project([1.35, a, 0.0])],
            palette::panel(),
            2.0,
        ));
    }
    stage.arena_mut().add(v_group(floor))
}

fn text(kit: &Kit, stage: &mut Stage<'_>, content: &str, at: Vec3) -> fmn::Result<()> {
    let label = kit.text(stage, content, 25.0, Palette::ink(), at)?;
    reveal(stage, &[label])
}

pub fn construct(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    kit.say(stage, "06_1")?;
    heading(kit, stage, "Build a body from a tree of motions")?;
    let floor = ground_grid(stage);
    reveal(stage, &[floor])?;
    let base = stage.add_value_tracker(0.0);
    let roll_value = stage.add_value_tracker(0.0);
    let view = stage.always_redraw(move |scene| {
        let u = scene.tracker_value(base).unwrap_or(0.0);
        let q = scene.tracker_value(roll_value).unwrap_or(0.0);
        scene.add(dog(
            [0.50 * u, -0.23 * u, 1.75 + 0.19 * u],
            0.33 * u,
            local_legs(0, q),
            0,
        ))
    });
    stage.add_to_scene(view)?;

    kit.say(stage, "06_2")?;
    let factors = kit.tex(
        stage,
        r"R_x(q_{\mathrm{roll}}),\ R_y(q_{\mathrm{hip}}),\ R_y(q_{\mathrm{knee}})",
        26.0,
        palette::rotation(),
        &[],
        v(2.88, 1.47),
    )?;
    reveal(stage, &[factors])?;
    text(kit, stage, "Three relative rotations per leg", v(2.88, 0.83))?;

    kit.say(stage, "06_3")?;
    let joints = HIPS.len() * JOINTS_PER_LEG;
    let ambient = kit.tex(
        stage,
        &format!(
            r"SE(3)\times SO(2)^{{{joints}}},\quad \mathrm{{dim}}={}",
            POSE_DIMENSION + joints
        ),
        27.0,
        palette::hero(),
        &[],
        v(2.88, -0.03),
    )?;
    reveal(stage, &[ambient])?;
    play!(stage; tween(base, 1.0, 2.4)?);

    kit.say(stage, "06_4")?;
    play!(stage; tween(roll_value, 0.46, 1.65)?);
    text(kit, stage, "The other branches stay put", v(2.88, -0.80))?;
    kit.hold(stage)?;

    kit.say(stage, "06_5")?;
    let catalogue = kit.text(
        stage,
        "Humanoid  ·  Hexapod  ·  Wheel-leg hybrid",
        25.0,
        Palette::ink(),
        v(0.0, -2.93),
    )?;
    reveal(stage, &[catalogue])?;
    kit.hold(stage)?;
    fade_all(stage, 0.45)?;

    kit.say(stage, "06_6")?;
    heading(kit, stage, "Pin the tip of one branch")?;
    let floor = ground_grid(stage);
    let supporting_foot = [1.02, 0.54, 0.0];
    let anchor = stage
        .arena_mut()
        .add(dot(project(supporting_foot), palette::good(), 0.145));
    reveal(stage, &[floor, anchor])?;
    let stance = stage.add_value_tracker(0.0);
    let constrained = stage.always_redraw(move |scene| {
        let u = scene.tracker_value(stance).unwrap_or(0.0);
        let legs = local_legs(0, 0.34 * u);
        let heading = 0.28 * u;
        let world = pin_translation(legs[0][2], heading, supporting_foot);
        scene.add(dog(world, heading, legs, 0))
    });
    stage.add_to_scene(constrained)?;
    text(kit, stage, "How many freedoms remain?", v(2.84, 1.62))?;
    kit.listen(stage, 4.0, "one_foot_constraint")?;

    kit.say(stage, "06_7")?;
    let contact = kit.tex(
        stage,
        r"R\,p_{\mathrm{foot}}(q)+t=a",
        35.0,
        palette::hero(),
        &[],
        v(2.84, 0.76),
    )?;
    reveal(stage, &[contact])?;
    play!(stage; tween(stance, 1.0, 2.45)?);
    let fixed = supporting_foot.len();
    let rank = kit.tex(
        stage,
        &format!(
            r"{}-{fixed}={}",
            POSE_DIMENSION + joints,
            POSE_DIMENSION + joints - fixed
        ),
        35.0,
        palette::good(),
        &[],
        v(2.84, -0.19),
    )?;
    reveal(stage, &[rank])?;

    kit.hold(stage)?;
    fade_all(stage, 0.43)?;
    kit.say(stage, "06_8")?;
    heading(kit, stage, "Change the contact and the equation follows")?;
    let floor = ground_grid(stage);
    let other_contact = [HIPS[1][0], HIPS[1][1], 0.0];
    let anchor = stage
        .arena_mut()
        .add(dot(project(other_contact), palette::good(), 0.145));
    reveal(stage, &[floor, anchor])?;
    let swing = stage.add_value_tracker(0.0);
    let second_configuration = stage.always_redraw(move |scene| {
        let phase = scene.tracker_value(swing).unwrap_or(0.0);
        let legs = local_legs(1, -0.26 * phase);
        let heading = -0.26 * phase;
        let world = pin_translation(legs[1][2], heading, other_contact);
        scene.add(dog(world, heading, legs, 1))
    });
    stage.add_to_scene(second_configuration)?;
    let right_contact = kit.tex(
        stage,
        r"R\,p_{\mathrm{right\ front}}(q)+t=b",
        29.0,
        palette::hero(),
        &[],
        v(2.84, 0.77),
    )?;
    reveal(stage, &[right_contact])?;
    text(kit, stage, "Each contact set picks a slice", v(2.84, -1.19))?;
    play!(stage; tween(swing, 1.0, 2.1)?);
    kit.finish(stage)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changing_one_joint_leaves_other_branches_unchanged() {
        let a = local_legs(0, 0.0);
        let b = local_legs(0, 0.43);
        assert_ne!(a[0][2], b[0][2]);
        assert_eq!(a[1..], b[1..]);
    }

    #[test]
    fn pinned_point_is_invariant_under_compensating_base_motion() {
        let target = [1.02, 0.54, 0.0];
        for step in -10..=10 {
            let u = f64::from(step) / 10.0;
            let [leg, ..] = local_legs(0, 0.34 * u);
            let angle = 0.28 * u;
            let world = pin_translation(leg[2], angle, target);
            let point = add(world, yaw(leg[2], angle));
            assert!(point.into_iter().zip(target).all(|(a, b)| (a - b).abs() < 1e-12));
        }
    }
}
