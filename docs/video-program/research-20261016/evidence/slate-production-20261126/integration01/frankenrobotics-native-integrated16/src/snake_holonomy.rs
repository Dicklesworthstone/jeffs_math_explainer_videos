//! Chapter 9's three-link demonstration, bound to the sampled rolling model.

use std::rc::Rc;

use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{Frame2, Kit, Palette, Timed, play, v};
use fmn::prelude::*;

use crate::model::Se2;
use crate::palette;

#[derive(Clone, Copy, Debug)]
struct Sample {
    angles: [f64; 2],
    pose: Se2,
}

fn trace(direction: &str) -> Rc<[Sample]> {
    include_str!("pose-trace01.tsv")
        .lines()
        .skip(1)
        .filter_map(|line| {
            let mut fields = line.split('\t');
            if fields.next()? != direction {
                return None;
            }
            let values: [f64; 6] = fields
                .map(|field| field.parse().expect("verified trace number"))
                .collect::<Vec<_>>()
                .try_into()
                .expect("verified trace row");
            Some(Sample {
                angles: [values[1], values[2]],
                pose: Se2 {
                    x: values[3],
                    y: values[4],
                    theta: values[5],
                },
            })
        })
        .collect()
}

fn at(samples: &[Sample], progress: f64) -> Sample {
    let last = samples.len() - 1;
    let position = progress.clamp(0.0, 1.0) * last as f64;
    let i = (position.floor() as usize).min(last - 1);
    let t = position - i as f64;
    let (p, q) = (samples[i], samples[i + 1]);
    let mix = |a: f64, b: f64| a + t * (b - a);
    Sample {
        angles: [mix(p.angles[0], q.angles[0]), mix(p.angles[1], q.angles[1])],
        pose: Se2 {
            x: mix(p.pose.x, q.pose.x),
            y: mix(p.pose.y, q.pose.y),
            theta: mix(p.pose.theta, q.pose.theta),
        },
    }
}

fn segment(a: Vec3, b: Vec3, color: Srgb, width: f64) -> VMobject {
    polyline(vec![a, b]).style(palette::line_style(color, width)).build()
}

fn marker(point: Vec3, color: Srgb, radius: f64) -> VMobject {
    Dot::new()
        .point(point)
        .radius(radius)
        .style(palette::flat(color, 1.0))
        .build()
}

fn point(frame: Frame2, p: [f64; 2]) -> Vec3 {
    frame.p(p[0], p[1])
}

/// Each outer link has unit length, and its wheel sits at the midpoint.
/// The gold line on each wheel is its rolling direction.
fn body(frame: Frame2, sample: Sample, start: Se2, ghost: bool) -> VMobject {
    let pose = start.compose(sample.pose);
    let hinges = [pose.apply([-0.5, 0.0]), pose.apply([0.5, 0.0])];
    let headings = [pose.theta + sample.angles[0], pose.theta, pose.theta + sample.angles[1]];
    let endpoint = |hinge: [f64; 2], heading: f64, length: f64| {
        [hinge[0] + length * heading.cos(), hinge[1] + length * heading.sin()]
    };
    let rear = endpoint(hinges[0], headings[0], -1.0);
    let nose = endpoint(hinges[1], headings[2], 1.0);
    let links = [rear, hinges[0], hinges[1], nose];
    let steel = if ghost { Palette::faint() } else { palette::steel() };
    let mut shapes = Vec::with_capacity(8);
    for link in links.windows(2) {
        shapes.push(segment(point(frame, link[0]), point(frame, link[1]), steel, 8.0));
    }
    for hinge in hinges {
        shapes.push(marker(
            point(frame, hinge),
            if ghost { Palette::faint() } else { palette::rotation() },
            0.055,
        ));
    }
    if !ghost {
        shapes.push(marker(point(frame, nose), palette::translation(), 0.065));
        let wheels = [
            endpoint(hinges[0], headings[0], -0.5),
            [pose.x, pose.y],
            endpoint(hinges[1], headings[2], 0.5),
        ];
        for (wheel, heading) in wheels.into_iter().zip(headings) {
            let a = endpoint(wheel, heading, -0.12);
            let b = endpoint(wheel, heading, 0.12);
            shapes.push(segment(point(frame, a), point(frame, b), palette::velocity(), 12.0));
        }
    }
    v_group(shapes)
}

fn fade(stage: &mut Stage<'_>, mob: Mob, seconds: f64) -> fmn::Result<()> {
    let animation = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?.rt(seconds);
    stage.play(animation)?;
    Ok(())
}

fn move_clock(clock: Mob, to: f64, seconds: f64) -> fmn::Result<AnimBuilder> {
    Ok(clock
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(seconds),
            rate_func: Some(fmn::core::rate::linear),
            ..AnimateArgs::default()
        })?
        .set_value(to)?)
}

fn path(frame: Frame2, samples: &[Sample], project: impl Fn(Sample) -> [f64; 2], color: Srgb) -> VMobject {
    polyline(
        samples
            .iter()
            .map(|&sample| point(frame, project(sample)))
            .collect::<Vec<_>>(),
    )
    .style(palette::line_style(color, 3.0))
    .build()
}

/// The clockwise prediction and the inverse motion show the joint and pose
/// representations side by side. Call before the curvature and nine-link acts.
pub fn construct_loop(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    let forward = trace("clockwise");
    let reverse = trace("counterclockwise");
    assert_eq!((forward.len(), reverse.len()), (201, 201));
    let displacement = at(&forward, 1.0).pose;
    let world = Frame2 {
        center: v(-3.65, 0.15),
        width: 5.55,
        height: 4.62,
        x: (-1.9, 1.7),
        y: (-1.5, 1.5),
    };
    let joints = Frame2 {
        center: v(3.65, 0.15),
        width: 4.05,
        height: 4.05,
        x: (0.35, 1.45),
        y: (0.35, 1.45),
    };
    kit.say(stage, "09_1")?;
    let name = kit.text(stage, "One shape cycle", 39.0, Palette::ink(), v(0.0, 3.42))?;
    let initial = stage.arena_mut().add(body(world, forward[0], Se2::IDENTITY, false));
    fade(stage, name, 0.45)?;
    fade(stage, initial, 0.65)?;

    kit.say(stage, "09_2")?;
    let floor_title = kit.text(
        stage,
        "Floor pose   SE(2)",
        28.0,
        palette::translation(),
        v(-3.65, 2.77),
    )?;
    let joint_title = kit.text(
        stage,
        "Joint angles   SO(2) × SO(2)",
        28.0,
        palette::rotation(),
        v(3.65, 2.77),
    )?;
    let floor_border = stage.arena_mut().add(world.border(Palette::faint())?);
    let joint_border = stage.arena_mut().add(joints.border(Palette::faint())?);
    let x_axis = kit.tex(stage, r"\alpha_1", 26.0, palette::rotation(), &[], v(3.65, -2.2))?;
    let y_axis = kit.tex(stage, r"\alpha_2", 26.0, palette::rotation(), &[], v(1.24, 0.15))?;
    for mob in [floor_title, joint_title, floor_border, joint_border, x_axis, y_axis] {
        stage.add_to_scene(mob)?;
    }
    let clock = stage.add_value_tracker(0.0);
    let unmasked = stage.add_value_tracker(0.0);
    let shape_trace = Rc::clone(&forward);
    let cursor = stage.always_redraw(move |s| {
        let value = s.tracker_value(clock).unwrap_or(0.0);
        let sample = if value <= 1.0 {
            at(&shape_trace, value)
        } else {
            at(&reverse, value - 1.0)
        };
        s.add(marker(point(joints, sample.angles), palette::hero(), 0.085))
    });
    stage.add_to_scene(cursor)?;

    kit.say(stage, "09_3")?;
    let circle = stage
        .arena_mut()
        .add(path(joints, &forward, |p| p.angles, palette::rotation()));
    fade(stage, circle, 0.6)?;
    stage.scene_mut().bring_to_front(&[cursor])?;

    kit.say(stage, "09_4")?;
    let hide_initial = fade_out(stage.arena_mut(), initial, ORIGIN, 1.0)?.rt(0.4);
    stage.play(hide_initial)?;
    let hidden = kit.text(stage, "Predict the final pose", 30.0, palette::quiz(), v(-3.65, 0.6))?;
    fade(stage, hidden, 0.4)?;
    play!(stage; move_clock(clock, 1.0, 5.0)?);
    kit.hold(stage)?;
    kit.mark(stage, "predict");
    let question = kit.text(stage, "?", 88.0, palette::quiz(), v(-3.65, -0.4))?;
    fade(stage, question, 0.3)?;
    let prediction_frames = frames_covering(5.0, stage.scene().time().fps());
    wait_frames(stage, prediction_frames)?;

    kit.say(stage, "09_5")?;
    let ghost_body = stage.arena_mut().add(body(world, forward[0], Se2::IDENTITY, true));
    stage.add_to_scene(ghost_body)?;
    let initial_curve = stage
        .arena_mut()
        .add(path(world, &forward, |p| [p.pose.x, p.pose.y], palette::translation()));
    let fwd = Rc::clone(&forward);
    let back = trace("counterclockwise");
    let return_pose = at(&back, 1.0).pose;
    let route = stage.always_redraw(move |s| {
        let value = s.tracker_value(unmasked).unwrap_or(0.0);
        let geometry = if value <= 1.0 {
            body(world, at(&fwd, value), Se2::IDENTITY, false)
        } else {
            body(world, at(&back, value - 1.0), displacement, false)
        };
        s.add(geometry)
    });
    let hide_question = fade_out(stage.arena_mut(), question, ORIGIN, 1.0)?.rt(0.35);
    let hide_label = fade_out(stage.arena_mut(), hidden, ORIGIN, 1.0)?.rt(0.35);
    stage.set_tracker_value(clock, 0.0)?;
    play!(stage; hide_question, hide_label);
    stage.add_to_scene(initial_curve)?;
    stage.add_to_scene(route)?;
    play!(stage; move_clock(clock, 1.0, 3.0)?, move_clock(unmasked, 1.0, 3.0)?);
    let amount = format!(
        "Forward loop   Δx {:+.3}   Δy {:+.3} links",
        displacement.x, displacement.y
    );
    let shift = kit.text(stage, &amount, 27.0, palette::translation(), v(-3.65, -2.73))?;
    fade(stage, shift, 0.4)?;

    kit.say(stage, "09_6")?;
    let inverse = kit.tex(stage, r"g\,g^{-1}=I", 34.0, palette::hero(), &[], v(3.65, -2.82))?;
    let dismiss = fade_out(stage.arena_mut(), shift, ORIGIN, 1.0)?.rt(0.35);
    play!(stage; dismiss);
    play!(stage; move_clock(clock, 2.0, 3.5)?, move_clock(unmasked, 2.0, 3.5)?);
    let reverse = kit.text(
        stage,
        &format!(
            "Reverse loop   Δx {:+.3}   Δy {:+.3} links",
            return_pose.x, return_pose.y
        ),
        27.0,
        palette::translation(),
        v(-3.65, -2.73),
    )?;
    fade(stage, reverse, 0.4)?;
    fade(stage, inverse, 0.45)?;

    kit.say(stage, "09_7")?;
    let balance = kit.text(
        stage,
        "Sideways speed in the body frame = 0",
        26.0,
        palette::velocity(),
        v(0.0, -3.43),
    )?;
    fade(stage, balance, 0.4)?;
    kit.hold(stage)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_closes_in_joint_space_and_reverses_in_pose_space() {
        let f = trace("clockwise");
        let r = trace("counterclockwise");
        let start = at(&f, 0.0);
        let end = at(&f, 1.0);
        assert!((start.angles[0] - end.angles[0]).abs() < 1e-14);
        assert!((start.angles[1] - end.angles[1]).abs() < 1e-14);
        assert!((end.pose.x - 0.20293).abs() < 1e-5);
        assert!((end.pose.y - 0.12723).abs() < 1e-5);
        let product = end.pose.compose(at(&r, 1.0).pose);
        assert!(product.x.abs().max(product.y.abs()).max(product.theta.abs()) < 1e-8);
    }
}
