//! Apply the representations already taught to three calculations from the site.

use std::f64::consts::FRAC_PI_2;

use explainer_kit::{Kit, Palette, Timed, fade_all, play, tween, v};
use fmn::prelude::*;

use crate::{
    group_foundations::{dot, heading, place, reveal, stroke},
    model::{
        QUAD_MODES, corridor_balanced_pose, corridor_exact_limit, dock_cart_ride, stance_apply_diagonal_shift,
        stance_equal_springs,
    },
    palette,
};

const HALL: f64 = 0.90;
const BODY_WIDTH: f64 = 0.60;
const BODY_LENGTH: f64 = 1.30;
const STANCE_HALF_WIDTH: f64 = 0.25;
const LIMIT: f64 = 160.0;

fn text(kit: &Kit, stage: &mut Stage<'_>, words: &str, at: Vec3) -> fmn::Result<Mob> {
    let label = kit.text(stage, words, 25.0, Palette::ink(), at)?;
    reveal(stage, &[label])?;
    Ok(label)
}

fn formula(kit: &Kit, stage: &mut Stage<'_>, words: &str, at: Vec3) -> fmn::Result<Mob> {
    let label = kit.tex(stage, words, 30.0, palette::hero(), &[], at)?;
    reveal(stage, &[label])?;
    Ok(label)
}

fn introduction(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    kit.say(stage, "12_1")?;
    heading(kit, stage, "Put the algebra to work")?;
    text(
        kit,
        stage,
        "Pose product   ·   Symmetry basis   ·   Affine maps",
        v(0.0, 0.44),
    )?;
    kit.hold(stage)?;
    fade_all(stage, 0.41)
}

fn corridor(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    kit.say(stage, "12_2")?;
    heading(kit, stage, "Send the entire footprint around the corner")?;
    let at = v(-5.92, -2.43);
    let metres = 2.64;
    let p = move |p| place(at, metres, p);
    let hall = Polygon::new(
        [
            [0.0, 0.0],
            [1.75, 0.0],
            [1.75, HALL],
            [HALL, HALL],
            [HALL, 1.75],
            [0.0, 1.75],
        ]
        .map(p),
    )
    .style(palette::flat(palette::panel(), 1.0))
    .build();
    let wall = stroke(
        vec![p([1.75, HALL]), p([HALL, HALL]), p([HALL, 1.75])],
        palette::steel(),
        4.2,
    );
    let outline = stroke(
        vec![p([1.75, 0.0]), p([0.0, 0.0]), p([0.0, 1.75])],
        palette::steel(),
        4.2,
    );
    let base = stage.arena_mut().add(v_group(vec![hall, wall, outline]));
    reveal(stage, &[base])?;
    let phi = stage.add_value_tracker(0.0);
    let body = stage.always_redraw(move |scene| {
        let angle = scene.tracker_value(phi).unwrap_or(0.0);
        let (g, _) = corridor_balanced_pose(BODY_LENGTH, BODY_WIDTH, HALL, angle);
        let [a, b] = [BODY_LENGTH / 2.0, BODY_WIDTH / 2.0];
        let corners = [[-a, -b], [a, -b], [a, b], [-a, b]].map(|q| p(g.apply(q)));
        scene.add(v_group(vec![
            Polygon::new(corners)
                .style(palette::flat(palette::translation(), 0.67))
                .build(),
            stroke(
                vec![corners[0], corners[1], corners[2], corners[3], corners[0]],
                palette::translation(),
                4.1,
            ),
            dot(p(g.apply([0.0, 0.0])), palette::hero(), 0.105),
        ]))
    });
    stage.add_to_scene(body)?;
    formula(
        kit,
        stage,
        &format!(r"a={HALL:.2}\,\mathrm{{m}},\quad b={BODY_WIDTH:.2}\,\mathrm{{m}}"),
        v(2.82, 1.75),
    )?;

    kit.say(stage, "12_3")?;
    let diameter = BODY_LENGTH.hypot(BODY_WIDTH);
    text(kit, stage, &format!("Inner circle: {BODY_WIDTH:.2} m"), v(2.82, 0.79))?;
    text(kit, stage, &format!("Outer circle: {diameter:.2} m"), v(2.82, 0.15))?;
    kit.hold(stage)?;

    kit.say(stage, "12_4")?;
    play!(stage; tween(phi, FRAC_PI_2 / 2.0, 1.95)?);
    let limit = corridor_exact_limit(HALL, BODY_WIDTH);
    formula(
        kit,
        stage,
        &format!(r"L_{{\mathrm{{max}}}}=2\sqrt{{2}}\,a-2b={limit:.3}\,\mathrm{{m}}"),
        v(2.82, -0.84),
    )?;
    text(kit, stage, "The tightest angle is halfway through", v(2.82, -1.65))?;
    kit.hold(stage)?;
    play!(stage; tween(phi, FRAC_PI_2, 1.80)?);
    kit.hold(stage)?;
    fade_all(stage, 0.42)
}

fn vector(values: [f64; 4]) -> String {
    values
        .map(|x| format!("{:.0}", if x.abs() < 1e-12 { 0.0 } else { x }))
        .join(r",\ ")
}

fn stance(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    kit.say(stage, "12_5")?;
    heading(kit, stage, "Change a force pattern without tipping")?;
    let columns = [-5.36, -3.94, -2.52, -1.10];
    let y_at = |force: f64| -1.55 + force / 88.0;
    let cap = stage.arena_mut().add(stroke(
        vec![v(-6.02, y_at(LIMIT)), v(-0.47, y_at(LIMIT))],
        palette::warn(),
        2.7,
    ));
    reveal(stage, &[cap])?;
    for (column, name) in columns.into_iter().zip(["FL", "FR", "BL", "BR"]) {
        text(kit, stage, name, v(column, -1.95))?;
    }
    let totals = formula(
        kit,
        stage,
        r"(W,M_\mathrm{pitch},M_\mathrm{roll})=(400,60,30)",
        v(2.83, 1.72),
    )?;
    text(kit, stage, "Force in N, moments in N·m", v(2.83, 1.02))?;

    kit.say(stage, "12_6")?;
    let old = stance_equal_springs(STANCE_HALF_WIDTH);
    let delta = stage.add_value_tracker(0.0);
    let half_width = stage.add_value_tracker(STANCE_HALF_WIDTH);
    let bars = stage.always_redraw(move |scene| {
        let width = scene.tracker_value(half_width).unwrap_or(STANCE_HALF_WIDTH);
        let equal_springs = stance_equal_springs(width);
        let requested = scene.tracker_value(delta).unwrap_or(0.0);
        // The back-right foot limits how far the diagonal can unload.
        let forces = stance_apply_diagonal_shift(equal_springs, requested.max(-equal_springs[3]));
        let geometry = columns
            .into_iter()
            .zip(forces)
            .map(|(x, force)| {
                let color = if force > LIMIT + 1e-9 {
                    palette::warn()
                } else {
                    palette::translation()
                };
                if force.abs() < 1e-9 {
                    dot(v(x, -1.55), palette::steel(), 0.055)
                } else {
                    stroke(vec![v(x, -1.55), v(x, y_at(force))], color, 14.0)
                }
            })
            .collect::<Vec<_>>();
        scene.add(v_group(geometry))
    });
    stage.add_to_scene(bars)?;
    let before = formula(kit, stage, &format!("({})", vector(old)), v(-3.25, -2.77))?;

    kit.say(stage, "12_7")?;
    let prompt = text(kit, stage, "Which pattern leaves all three fixed?", v(2.77, -0.84))?;
    kit.listen(stage, 5.0, "stance_null_mode")?;

    kit.say(stage, "12_8")?;
    let remove = fade_out(stage.arena_mut(), prompt, ORIGIN, 1.0)?.rt(0.35);
    stage.play(remove)?;
    let [a, b, c, d] = QUAD_MODES[3];
    formula(kit, stage, &format!(r"({a:+},{b:+},{c:+},{d:+})"), v(2.82, -0.63))?;

    kit.say(stage, "12_9")?;
    let hide_before = fade_out(stage.arena_mut(), before, ORIGIN, 1.0)?.rt(0.35);
    stage.play(hide_before)?;
    let shift = -20.0;
    play!(stage; tween(delta, shift, 2.28)?);
    let new = stance_apply_diagonal_shift(old, shift);
    let active = formula(kit, stage, &format!("({})", vector(new)), v(-3.25, -2.77))?;
    text(kit, stage, "The load and both moments stay fixed", v(2.84, -1.50))?;
    kit.hold(stage)?;

    kit.say(stage, "12_10")?;
    let hide_totals = fade_out(stage.arena_mut(), totals, ORIGIN, 1.0)?.rt(0.34);
    let hide_active = fade_out(stage.arena_mut(), active, ORIGIN, 1.0)?.rt(0.34);
    play!(stage; hide_totals, hide_active);
    let edge_width = 3.0 / 14.0;
    let narrow = stance_equal_springs(edge_width);
    let edge = stance_apply_diagonal_shift(narrow, -narrow[3]);
    play!(stage; tween(half_width, edge_width, 1.35)?);
    let edge_label = formula(
        kit,
        stage,
        r"b_{\mathrm{half}}=\frac{3}{14}\,\mathrm{m}",
        v(2.85, 1.71),
    )?;
    let edge_vector = formula(kit, stage, &format!("({})", vector(edge)), v(-3.25, -2.77))?;
    kit.until_sentence(stage, "12_10", 2)?;
    let hide_label = fade_out(stage.arena_mut(), edge_label, ORIGIN, 1.0)?.rt(0.34);
    let hide_vector = fade_out(stage.arena_mut(), edge_vector, ORIGIN, 1.0)?.rt(0.34);
    play!(stage; hide_label, hide_vector);
    let tight_width = 0.2;
    let very_narrow = stance_equal_springs(tight_width);
    let best = stance_apply_diagonal_shift(very_narrow, -very_narrow[3]);
    let min_peak = best[0];
    play!(stage; tween(half_width, tight_width, 1.35)?);
    formula(kit, stage, r"b_{\mathrm{half}}=\frac{1}{5}\,\mathrm{m}", v(2.85, 1.71))?;
    formula(
        kit,
        stage,
        &format!(r"F_{{\mathrm{{max}}}}\geq {min_peak:.0}\,\mathrm{{N}}"),
        v(2.87, -2.25),
    )?;
    formula(kit, stage, &format!("({})", vector(best)), v(-3.25, -2.77))?;
    kit.hold(stage)?;
    fade_all(stage, 0.42)
}

fn cart_at(gain: f64, pushes: [f64; 3], time: f64) -> [f64; 2] {
    let mut x = 0.0;
    let mut speed = 0.0;
    for (i, u) in pushes.into_iter().enumerate() {
        let dt = (time - i as f64).clamp(0.0, 1.0);
        x += speed * dt + gain * u * dt * dt / 2.0;
        speed += gain * u * dt;
    }
    [x, speed]
}

fn carts(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    kit.say(stage, "12_11")?;
    heading(kit, stage, "Docking composes three affine maps")?;
    formula(kit, stage, r"(x,v)\mapsto(x+v+gu/2,\ v+gu)", v(0.0, 2.11))?;
    let levels = [0.14, -1.06];
    let gains = [1.0, 2.0];
    let pushes = gains.map(|gain| dock_cart_ride(gain, 0.25).0);
    let time = stage.add_value_tracker(0.0);
    for (i, y) in levels.into_iter().enumerate() {
        let lane = stage
            .arena_mut()
            .add(stroke(vec![v(-4.23, y), v(4.58, y)], palette::steel(), 3.2));
        reveal(stage, &[lane])?;
        text(kit, stage, &format!("Gain {:.0}", gains[i]), v(-5.50, y))?;
    }
    let dots = stage.always_redraw(move |scene| {
        let t = scene.tracker_value(time).unwrap_or(0.0);
        let geometry = gains
            .into_iter()
            .enumerate()
            .map(|(i, gain)| {
                let [x, _] = cart_at(gain, pushes[i], t);
                dot(
                    v(-4.20 + 8.77 * x, levels[i]),
                    if i == 0 {
                        palette::translation()
                    } else {
                        palette::rotation()
                    },
                    0.14,
                )
            })
            .collect::<Vec<_>>();
        scene.add(v_group(geometry))
    });
    stage.add_to_scene(dots)?;

    kit.say(stage, "12_12")?;
    play!(stage; tween(time,1.0,1.72)?);
    let speeds = gains.map(|g| cart_at(g, dock_cart_ride(g, 0.25).0, 1.0)[1]);
    let threshold = (speeds[1] - speeds[0]) / 2.0;
    formula(
        kit,
        stage,
        &format!(
            r"v_1={:.2}\ \mathrm{{or}}\ {:.2},\quad \varepsilon<{threshold:.3}",
            speeds[0], speeds[1]
        ),
        v(0.0, -2.44),
    )?;
    play!(stage; tween(time,3.0,2.30)?);
    kit.hold(stage)?;
    fade_all(stage, 0.44)
}

pub fn construct(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    introduction(kit, stage)?;
    corridor(kit, stage)?;
    stance(kit, stage)?;
    carts(kit, stage)?;

    kit.say(stage, "12_13")?;
    heading(kit, stage, "Start with the algebra of the body")?;
    formula(
        kit,
        stage,
        r"R(q_1)T(1)R(q_2)T(1),\qquad SE(2),\qquad C_2\times C_2",
        v(0.0, 1.35),
    )?;
    text(
        kit,
        stage,
        "Locate the hand   ·   Move the rover   ·   Shift the load",
        v(0.0, 0.37),
    )?;

    kit.say(stage, "12_14")?;
    text(
        kit,
        stage,
        "Which transformations? What do they act on?",
        v(0.0, -1.20),
    )?;
    text(
        kit,
        stage,
        "Compose in order. Apply the joint and contact constraints.",
        v(0.0, -1.90),
    )?;
    kit.finish(stage)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cart_paths_share_the_desired_terminal_state() {
        for g in [1.0, 2.0] {
            let (moves, target) = dock_cart_ride(g, 0.25);
            assert_eq!(cart_at(g, moves, 3.0), target);
            assert_eq!(target, [1.0, 0.0]);
        }
    }
}
