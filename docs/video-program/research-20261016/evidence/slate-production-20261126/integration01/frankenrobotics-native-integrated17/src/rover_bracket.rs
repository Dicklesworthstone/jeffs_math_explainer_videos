//! The four-move rover loop first as a group product, then as a Lie bracket.

use std::f64::consts::FRAC_PI_2;

use explainer_kit::{Kit, Palette, Timed, add, fade_all, play, tween, v};
use fmn::prelude::*;

use crate::{
    group_foundations::{dot, heading, place, pose, reveal, stroke},
    model::{Se2, four_move_wheel_travel},
    palette,
};

type Matrix = [[i8; 3]; 3];

const TURN: Matrix = [[0, -1, 0], [1, 0, 0], [0, 0, 0]];
const DRIVE: Matrix = [[0, 0, 1], [0, 0, 0], [0, 0, 0]];

fn bracket(a: Matrix, b: Matrix) -> Matrix {
    std::array::from_fn(|i| std::array::from_fn(|j| (0..3).map(|k| a[i][k] * b[k][j] - b[i][k] * a[k][j]).sum()))
}

fn tex_matrix(rows: Matrix) -> String {
    format!(
        r"\begin{{pmatrix}}{}\end{{pmatrix}}",
        rows.map(|row| row.map(|entry| entry.to_string()).join("&")).join(r"\\")
    )
}

fn line_label(kit: &Kit, stage: &mut Stage<'_>, words: &str, point: Vec3) -> fmn::Result<()> {
    let mob = kit.text(stage, words, 25.0, Palette::ink(), point)?;
    reveal(stage, &[mob])
}

pub fn construct(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    kit.say(stage, "05_1")?;
    heading(kit, stage, "Can these wheels go sideways?")?;
    let origin = v(-3.3, -0.7);
    let scale = 1.34;
    let track = [
        stroke(
            vec![place(origin, scale, [-1.56, 0.0]), place(origin, scale, [1.28, 0.0])],
            palette::panel(),
            3.8,
        ),
        stroke(
            vec![place(origin, scale, [0.0, -0.68]), place(origin, scale, [0.0, 1.7])],
            palette::panel(),
            3.8,
        ),
        dot(origin, palette::steel(), 0.077),
    ];
    let grid = stage.arena_mut().add(v_group(track));
    reveal(stage, &[grid])?;
    let clock = stage.add_value_tracker(0.0);
    let view = stage.always_redraw(move |scene| {
        let phase = scene.tracker_value(clock).unwrap_or(0.0);
        let g = Se2::rover_commutator_at(FRAC_PI_2, 1.0, phase);
        let mut geometry = vec![pose(origin, scale, g)];
        if phase > 1.0 {
            let path = (0..=72)
                .map(|i| {
                    let g = Se2::rover_commutator_at(FRAC_PI_2, 1.0, phase * f64::from(i) / 72.0);
                    place(origin, scale, [g.x, g.y])
                })
                .collect();
            geometry.insert(0, stroke(path, palette::translation(), 4.0));
        }
        scene.add(v_group(geometry))
    });
    stage.add_to_scene(view)?;

    kit.say(stage, "05_2")?;
    for (step, label) in ["Turn left", "Drive forward", "Turn right", "Drive backward"]
        .into_iter()
        .enumerate()
    {
        let at = v(2.63, 1.66 - step as f64 * 0.79);
        line_label(kit, stage, label, at)?;
        if step < 3 {
            play!(stage; tween(clock, (step + 1) as f64, 1.05)?);
        }
    }

    kit.say(stage, "05_3")?;
    let prompt = kit.text(
        stage,
        "Where does the axle finish?",
        27.0,
        palette::quiz(),
        v(0.0, -2.40),
    )?;
    reveal(stage, &[prompt])?;
    kit.listen(stage, 4.0, "rover_commutator_finish")?;

    kit.say(stage, "05_4")?;
    let remove_prompt = fade_out(stage.arena_mut(), prompt, ORIGIN, 1.0)?.rt(0.30);
    stage.play(remove_prompt)?;
    play!(stage; tween(clock, 4.0, 1.50)?);
    let result = Se2::rover_commutator(FRAC_PI_2, 1.0);
    let answer = kit.tex(
        stage,
        &format!(
            r"({:.0},{:.0}),\quad \theta_\mathrm{{final}}={:.0}",
            result.x, result.y, result.theta
        ),
        28.0,
        palette::hero(),
        &[],
        v(-3.3, -2.42),
    )?;
    reveal(stage, &[answer])?;

    kit.say(stage, "05_5")?;
    let product = kit.tex(
        stage,
        r"R_\theta T_a R_{-\theta}T_{-a}=T_{a(\cos\theta-1),\,a\sin\theta}",
        30.0,
        palette::hero(),
        &[],
        v(0.0, -3.23),
    )?;
    reveal(stage, &[product])?;
    kit.hold(stage)?;
    fade_all(stage, 0.4)?;

    kit.say(stage, "05_6")?;
    heading(kit, stage, "Shrink the loop and watch the direction")?;
    let frame_origin = v(-2.9, -0.75);
    let theta = stage.add_value_tracker(FRAC_PI_2);
    let path = stage.always_redraw(move |scene| {
        let alpha = scene.tracker_value(theta).unwrap_or(FRAC_PI_2);
        let end = Se2::rover_commutator(alpha, 1.0);
        let rotation = Se2::rotation(alpha);
        let arm = rotation.apply([1.0, 0.0]);
        let points = [[0.0, 0.0], arm, [end.x, end.y]].map(|p| place(frame_origin, 2.48, p));
        scene.add(v_group(vec![
            stroke(points.to_vec(), palette::translation(), 3.5),
            stroke(vec![points[0], points[2]], palette::hero(), 4.2),
            dot(points[2], palette::hero(), 0.09),
        ]))
    });
    stage.add_to_scene(path)?;
    let circle_arrow = stroke(
        vec![frame_origin, add(frame_origin, v(0.0, 2.62))],
        palette::steel(),
        2.2,
    );
    let baseline = stage.arena_mut().add(circle_arrow);
    reveal(stage, &[baseline])?;
    for power in 1..=3 {
        play!(stage; tween(theta, FRAC_PI_2 / 2_f64.powi(power), 1.15)?);
    }
    let terms = kit.tex(
        stage,
        r"\begin{array}{rcl}x&=&a(\cos\theta-1)\sim-a\theta^2/2\\y&=&a\sin\theta\sim a\theta\end{array}",
        30.0,
        palette::hero(),
        &[],
        v(2.57, 1.03),
    )?;
    reveal(stage, &[terms])?;

    kit.say(stage, "05_7")?;
    let small = 0.25;
    let large_move = Se2::rover_commutator(small, small);
    let small_move = Se2::rover_commutator(small / 2.0, small / 2.0);
    let measurements = kit.tex(
        stage,
        &format!(
            r"y({small},{small})={:.4},\quad y({},{})={:.4}",
            large_move.y,
            small / 2.0,
            small / 2.0,
            small_move.y
        ),
        27.0,
        palette::translation(),
        &[],
        v(0.0, -1.70),
    )?;
    reveal(stage, &[measurements])?;

    kit.say(stage, "05_8")?;
    let ratio = kit.tex(
        stage,
        &format!(
            r"\frac{{y(a,\theta)}}{{y(a/2,\theta/2)}}={:.2}\longrightarrow4",
            large_move.y / small_move.y
        ),
        30.0,
        palette::hero(),
        &[],
        v(0.0, -2.81),
    )?;
    reveal(stage, &[ratio])?;
    kit.hold(stage)?;
    fade_all(stage, 0.45)?;

    kit.say(stage, "05_9")?;
    heading(kit, stage, "Subtract the two velocity products")?;
    let matrices = kit.tex(
        stage,
        &format!(r"A={}\qquad B={}", tex_matrix(TURN), tex_matrix(DRIVE)),
        33.0,
        palette::rotation(),
        &[],
        v(0.0, 1.72),
    )?;
    reveal(stage, &[matrices])?;
    let commutator = kit.tex(
        stage,
        &format!(r"[A,B]=AB-BA={}", tex_matrix(bracket(TURN, DRIVE))),
        33.0,
        palette::hero(),
        &[],
        v(0.0, -0.06),
    )?;
    reveal(stage, &[commutator])?;

    kit.say(stage, "05_10")?;
    line_label(
        kit,
        stage,
        "Turn, forward, sideways span the planar velocity space",
        v(0.0, -1.36),
    )?;
    kit.hold(stage)?;

    kit.say(stage, "05_11")?;
    let distance_m = 0.01;
    let half_track_m = 0.20;
    let (angle, travel_m) = four_move_wheel_travel(distance_m, half_track_m);
    let travel_formula = kit.tex(
        stage,
        &format!(
            r"\mathrm{{Four\ moves:}}\quad 2b\alpha+d\cot(\alpha/2),\qquad \alpha\approx {:.0}^\circ",
            angle.to_degrees()
        ),
        24.0,
        Palette::ink(),
        &[],
        v(0.0, -2.03),
    )?;
    let cost = kit.tex(
        stage,
        &format!(
            r"{:.0}\ \mathrm{{cm\ sideways}}\quad\longrightarrow\quad {:.1}\ \mathrm{{cm\ per\ wheel}}",
            100.0 * distance_m,
            100.0 * travel_m
        ),
        29.0,
        palette::translation(),
        &[],
        v(0.0, -2.72),
    )?;
    reveal(stage, &[travel_formula, cost])?;
    kit.finish(stage)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bracket_is_the_tangent_of_the_finite_rover_loop() {
        let matrix = bracket(TURN, DRIVE);
        assert_eq!(matrix, [[0, 0, 0], [0, 0, 1], [0, 0, 0]]);
        let h = 1e-4;
        let g = Se2::rover_commutator(h, h);
        assert!((g.x / (h * h) - f64::from(matrix[0][2])).abs() < 1e-4);
        assert!((g.y / (h * h) - f64::from(matrix[1][2])).abs() < 1e-8);
    }
}
