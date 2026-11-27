//! Follow the website's prescribed nine-link snake and compare two contact laws.

use std::f64::consts::{PI, TAU};
use std::rc::Rc;

use explainer_kit::{Kit, Palette, add, fade_all, lerp, play, tween, v};
use fmn::prelude::*;

use crate::{
    group_foundations::{dot, heading, reveal, stroke},
    palette,
};

const LINKS: usize = 9;
const LENGTH: f64 = 0.19;
const ARC_PERIOD_M: f64 = 1.3;
const AMPLITUDE: f64 = 0.55;
const STEPS: usize = 512;
type Angles = [f64; LINKS];
type Chain = [[f64; 2]; LINKS + 1];

fn nose_heading(s: f64) -> f64 {
    AMPLITUDE * (TAU * s / ARC_PERIOD_M).sin()
}

fn rates(s: f64, angles: Angles) -> Angles {
    // Coordinates use x forward, y left. The site's lateral coordinate is -y.
    let (mut vy, mut vx) = nose_heading(s).sin_cos();
    std::array::from_fn(|i| {
        let (sin, cos) = angles[i].sin_cos();
        let turn = (-vx * sin + vy * cos) / (LENGTH / 2.0);
        vx += LENGTH * sin * turn;
        vy -= LENGTH * cos * turn;
        turn
    })
}

fn advance(s: f64, q: Angles, ds: f64) -> Angles {
    let add_rate = |k: Angles, factor: f64| std::array::from_fn(|i| q[i] + factor * k[i]);
    let k1 = rates(s, q);
    let k2 = rates(s + ds / 2.0, add_rate(k1, ds / 2.0));
    let k3 = rates(s + ds / 2.0, add_rate(k2, ds / 2.0));
    let k4 = rates(s + ds, add_rate(k3, ds));
    std::array::from_fn(|i| q[i] + ds * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]) / 6.0)
}

fn trace() -> Vec<Chain> {
    let ds = ARC_PERIOD_M / STEPS as f64;
    let mut q = [0.0; LINKS];
    for _ in 0..7 {
        for k in 0..STEPS {
            q = advance(k as f64 * ds, q, ds);
        }
    }
    let mut nose = [0.0, 0.0];
    let mut result = Vec::with_capacity(STEPS + 1);
    for k in 0..=STEPS {
        let mut point = nose;
        let joints = std::array::from_fn(|i| {
            let current = point;
            if i < LINKS {
                let (sin, cos) = q[i].sin_cos();
                point = [point[0] - LENGTH * cos, point[1] - LENGTH * sin];
            }
            current
        });
        result.push(joints);
        if k < STEPS {
            let s = k as f64 * ds;
            let (sin, cos) = nose_heading(s + ds / 2.0).sin_cos();
            nose = [nose[0] + ds * cos, nose[1] + ds * sin];
            q = advance(s, q, ds);
        }
    }
    result
}

fn project([x, y]: [f64; 2]) -> Vec3 {
    v(0.65 + 3.40 * x, 0.40 + 3.40 * y)
}

fn text(kit: &Kit, stage: &mut Stage<'_>, phrase: &str, where_at: Vec3) -> fmn::Result<()> {
    let label = kit.text(stage, phrase, 25.0, Palette::ink(), where_at)?;
    reveal(stage, &[label])
}

pub fn construct(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    kit.say(stage, "09_8")?;
    fade_all(stage, 0.42)?;
    heading(kit, stage, "The order of two shape changes matters")?;
    let equation = kit.tex(
        stage,
        r"V_i(q,g)=(e_i,\;g\widehat A_i(q))",
        33.0,
        palette::hero(),
        &[],
        v(0.0, 1.03),
    )?;
    reveal(stage, &[equation])?;
    text(kit, stage, "Changing one hinge also updates the base", v(0.0, 0.16))?;
    let small_loop = kit.tex(
        stage,
        r"\text{small loop}\quad\longrightarrow\quad [V_1,V_2]",
        31.0,
        palette::translation(),
        &[],
        v(0.0, -1.15),
    )?;
    reveal(stage, &[small_loop])?;
    text(
        kit,
        stage,
        "Reverse the loop and its leading displacement reverses",
        v(0.0, -2.24),
    )?;
    kit.hold(stage)?;
    fade_all(stage, 0.42)?;

    kit.say(stage, "09_9")?;
    heading(kit, stage, "Follow the nine-link snake for one wave")?;
    let chain = Rc::new(trace());
    let dx = chain[STEPS][0][0] - chain[0][0][0];
    let nose_route = chain.iter().map(|links| project(links[0])).collect();
    let line = stage.arena_mut().add(stroke(nose_route, palette::rotation(), 3.0));
    reveal(stage, &[line])?;
    let phase = stage.add_value_tracker(0.0);
    let snake = stage.always_redraw(move |scene| {
        let u = scene.tracker_value(phase).unwrap_or(0.0).clamp(0.0, 1.0) * STEPS as f64;
        let ix = (u.floor() as usize).min(STEPS - 1);
        let t = u - ix as f64;
        let points: [Vec3; LINKS + 1] = std::array::from_fn(|i| {
            let a = chain[ix][i];
            let b = chain[ix + 1][i];
            project([lerp(a[0], b[0], t), lerp(a[1], b[1], t)])
        });
        let mut geometry = Vec::new();
        for (i, pair) in points.windows(2).enumerate() {
            let color = if i.is_multiple_of(2) {
                palette::steel()
            } else {
                palette::translation()
            };
            geometry.push(stroke(pair.to_vec(), color, 8.0));
            geometry.push(dot(pair[0], palette::rotation(), 0.071));
        }
        geometry.push(dot(points[0], palette::hero(), 0.12));
        scene.add(v_group(geometry))
    });
    stage.add_to_scene(snake)?;
    play!(stage; tween(phase, 1.0, 4.3)?);
    let distances = kit.tex(
        stage,
        &format!(r"\text{{Arc travelled }}={ARC_PERIOD_M:.1}\,\mathrm{{m}},\quad\text{{floor advance }}={dx:.2}\,\mathrm{{m}}"),
        27.0,
        palette::hero(),
        &[],
        v(0.0, -1.55),
    )?;
    reveal(stage, &[distances])?;
    kit.hold(stage)?;
    fade_all(stage, 0.42)?;

    kit.say(stage, "09_10")?;
    heading(kit, stage, "One wheel-leg, two contact equations")?;
    let ground = stage
        .arena_mut()
        .add(stroke(vec![v(-6.0, -1.1), v(5.9, -1.1)], palette::steel(), 2.8));
    reveal(stage, &[ground])?;
    let progress = stage.add_value_tracker(0.0);
    let wheel_leg = stage.always_redraw(move |scene| {
        let u = scene.tracker_value(progress).unwrap_or(0.0).clamp(0.0, 3.0);
        let swing_fraction = (u - 1.0).clamp(0.0, 1.0);
        let foot_x = -4.6 + 3.10 * u.min(1.0) + 0.71 * swing_fraction;
        let radius = 0.37;
        let wheel = v(foot_x, -1.10 + radius + 0.45 * (PI * swing_fraction).sin());
        let lower_angle = 0.47_f64;
        let upper_angle = -0.72 + 1.23 * (u - 2.0).max(0.0);
        let bend = add(wheel, v(0.91 * lower_angle.sin(), 0.91 * lower_angle.cos()));
        let body = add(bend, v(1.08 * upper_angle.sin(), 1.08 * upper_angle.cos()));
        let ring = Circle::new()
            .radius(radius)
            .style(palette::line_style(palette::steel(), 4.8))
            .build();
        let radians = 3.10 * u.min(1.0) / radius;
        let spoke = add(wheel, v(radius * radians.cos(), -radius * radians.sin()));
        scene.add(v_group(vec![
            ring.moved_to(wheel),
            stroke(vec![wheel, spoke], palette::rotation(), 3.4),
            stroke(vec![body, bend, wheel], palette::steel(), 7.0),
            dot(body, palette::translation(), 0.11),
        ]))
    });
    stage.add_to_scene(wheel_leg)?;
    let rolling = kit.tex(
        stage,
        r"\text{rolling:}\quad\dot x=r\dot q",
        30.0,
        palette::rotation(),
        &[],
        v(3.32, 1.14),
    )?;
    reveal(stage, &[rolling])?;
    play!(stage; tween(progress, 1.0, 2.05)?);
    play!(stage; tween(progress, 2.0, 1.50)?);
    let locked = stage.arena_mut().add(dot(v(-0.79, -1.10), palette::good(), 0.12));
    reveal(stage, &[locked])?;
    let sticking = kit.tex(
        stage,
        r"\text{planted:}\quad\dot p_\mathrm{contact}=0",
        30.0,
        palette::good(),
        &[],
        v(3.32, 0.0),
    )?;
    reveal(stage, &[sticking])?;
    play!(stage; tween(progress, 3.0, 1.85)?);
    kit.finish(stage)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_nine_link_periodic_shape_closes_and_matches_the_site_nose_distance() {
        let points = trace();
        let begin = points[0];
        let end = points[STEPS];
        for (a, b) in begin.into_iter().zip(end) {
            // Remove the common advance before checking shape periodicity.
            assert!((b[0] - end[0][0] - a[0]).abs() < 1e-8);
            assert!((b[1] - end[0][1] - a[1]).abs() < 1e-8);
        }
        assert!((end[0][0] - 1.203_530_675_884_688_4).abs() < 5e-9);
        assert!(end[0][1].abs() < 1e-12);
    }
}
