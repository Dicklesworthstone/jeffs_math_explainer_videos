//! Chapter 11, lost directions: six rows allow six directions (11_1); the
//! elbow straightens and the viewer is asked whether all six are still
//! there (11_2); every joint's push on the wrist is across the arm, so
//! nothing slides along it (11_3); a singularity, and what it costs near
//! one (11_4, 11_5); and the wrist's three axes meeting at a point, the
//! gimbal of chapter 03, with the third joint at the shoulder covering for
//! the lost one (11_6).

use std::f64::consts::PI;

use explainer_kit::props::lower_third;
use explainer_kit::{Frame2, Kit, Palette, Timed, play, thousands, tween, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::draw::{
    ARM_SCALE, ArmRig, Ask, Choice, PartSpec, Pin, QSource, QuizCard, QuizLook, Shape, View, World, fact, fact_f64,
    fade_in_all, fade_in_lagged, faded_line, fail, leave, line_style, point_fn, say, still, table, value,
};
use super::velocities_add::{GOAL_DIMMED, curl, goal_parts, standing_goal};
use crate::model::{self, Arm, DEG, ELBOW, N, Pose, SHOULDER, V3, WRIST, add, cross, dot, poses, scale, sub, unit};
use crate::palette;

pub struct LostDirections {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const DIRECTIONS: &str = "directions:";
const ALL_SIX: &str = "all six";
const ONE_IS_LOST: &str = "one is lost";
const STILL_SIX: &str = "can the gripper still move\nin all six directions?";
const SWING_SIDEWAYS: &str = "the wrist can only swing sideways";
const SWING_LABEL: &str = "at first:";
const SINGULARITY: &str = "singularity";
const NOT_ONLY_FULL_STRETCH: &str = "a property of the joint angles:";
const FROM_THE_GEOMETRY: &str = "from the geometry alone: the gripper does not turn";
const LIMIT_SOURCE: &str = "limit: KUKA's speed for the elbow, at rated payload";
const DIRECTIONS_LABEL: &str = "directions:";
const DIRECTIONS_TEXT: &str = "the independent ways the gripper can move; their number is the Jacobian's rank";
const ELBOW_SPEED: &str = "the elbow's speed to pull the tip back at";
const LIMIT: &str = "limit";
const DEG_PER_S: &str = "°/s";
const BEND: &str = "the elbow's bend, degrees";
const SPEED: &str = "its speed, °/s";
const TURNS_A_SECOND: &str = "turns a second";
const WITHOUT_THREE: &str = "without joint 3:";
const SHOULDER_TWO: &str = "joints 1 and 2, at the shoulder";
const THIRD_JOINT: &str = "joint 3";
const PUSH_NOTE: &str = "push on the wrist";

/// The beats of the chapter, in order (storyboard ids).
const BEATS: [&str; 6] = ["11_1", "11_2", "11_3", "11_4", "11_5", "11_6"];

/// 11_2's quiz: all six, or one is lost (`G-Q.11_2.answer`).
pub const ASK: Ask = Ask {
    line: BEATS[1],
    question: STILL_SIX,
    choices: &[
        Choice {
            key: "all",
            label: ALL_SIX,
        },
        Choice {
            key: "lost",
            label: ONE_IS_LOST,
        },
    ],
};

const ARM_AT: [f64; 2] = [-3.1, -2.55];
const CARD_AT: [f64; 2] = [4.45, 0.0];
const CARD_SCALE: f64 = 0.92;
/// Where "push on the wrist" stands against the wrist (right of the arrows
/// that leave it, which fan up and down-right) and "joint 3" against its
/// axis (above-left, clear of the shoulder).
const PUSH_NOTE_SHIFT: [f64; 2] = [1.7, -0.1];
const THIRD_JOINT_SHIFT: [f64; 2] = [-0.55, 0.75];
/// Where the readout of "directions" sits.
const READOUT_AT: [f64; 2] = [-5.0, 3.35];
/// The speed above which the plot is cut off, degrees a second.
const SPEED_CUT: f64 = 200.0;
/// The bend 11_4's gauge first settles at, degrees.
const FIRST_BEND_DEG: f64 = 30.0;
/// Where the label of the plot's ten-degree point begins, from its dot: to
/// its right and a little up, over the curve's lower side (the curve climbs
/// steeply to the left of the dot and runs through a label centred above it,
/// as the narrated draft's 11_5 had it).
const TEN_LABEL_OFFSET: [f64; 2] = [0.25, 0.1];
/// The bend plot's window: degrees across, degrees a second up.
const BEND_MAX: f64 = 40.0;
const PLOT: Frame2 = Frame2 {
    center: [3.55, -0.1, 0.0],
    width: 6.0,
    height: 3.5,
    x: (0.0, BEND_MAX),
    y: (0.0, SPEED_CUT),
};
/// The gauge's bar: where it starts, how long a degree a second is.
const GAUGE_AT: [f64; 2] = [0.9, 1.2];
const GAUGE_UNIT: f64 = 0.0175;
/// How far a joint wiggles when its push is shown, radians.
const WIGGLE: f64 = 0.09;
/// The close-up's scale and where the wrist sits on the screen.
const CLOSE_SCALE: f64 = 6.0;
const CLOSE_AT: [f64; 2] = [3.6, 0.2];
const CURL_SWEEP: f64 = 1.5 * PI;

/// The parts this chapter adds to the arm's 23 (the table is 23).
mod part {
    /// The six small arrows at the tip: three slides, three turns.
    pub const SIX: usize = 24;
    /// The pushes of joints 1, 2 and 4 on the wrist (three parts), and the
    /// lost direction along the arm.
    pub const PUSH_FIRST: usize = 25;
    pub const ALONG: usize = 28;
    /// The goal's book (29) and its marker (30), dimmed through the chapter.
    pub const GOAL: usize = 30;
}

/// The close-up world's parts after the arm's 23: the three axes of the
/// wrist, the three rings about them, and the point where they meet.
mod close {
    pub const AXES: [usize; 3] = [23, 24, 25];
    pub const RINGS: [usize; 3] = [26, 27, 28];
    pub const POINT: usize = 29;
}

/// The three wrist joints (0-based): 5, 6 and 7.
const WRIST_JOINTS: [usize; 3] = [4, 5, 6];
/// The joints that wiggle in turn in 11_3: 1 to 5.
const WIGGLERS: [usize; 5] = [0, 1, 2, 3, 4];
/// Of those, the joints whose push on the wrist the picture draws: 1, 2 and
/// 4. With the elbow straight the axes of joints 3 and 5 run through the
/// wrist (joint 3's lies along the arm), so turning them does not move it.
const PUSHERS: [usize; 3] = [0, 1, 3];

/// The elbow's speed (degrees a second) to pull the tip straight back at
/// `pull` m/s with the elbow bent `bend` degrees (the model's own).
fn elbow_speed(arm: &Arm, bend: f64, pull: f64) -> f64 {
    arm.elbow_speed_to_pull(bend, pull)
}

/// The curve of the plot: (bend, speed) at every quarter degree from the
/// first bend whose speed fits under the cut, to the right edge.
fn curve(arm: &Arm, pull: f64) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    let mut b = 0.25;
    while b <= BEND_MAX + 1e-9 {
        let s = elbow_speed(arm, b, pull);
        if s <= SPEED_CUT {
            out.push((b, s));
        }
        b += 0.25;
    }
    out
}

/// A whole number of degrees a second as the voice and the picture say it.
fn per_second(x: f64) -> String {
    thousands(x.round() as u64)
}

/// A count as a word, for the lower third.
fn number_word(n: usize) -> &'static str {
    ["none", "one", "two", "three", "four", "five", "six", "seven"]
        .get(n)
        .copied()
        .unwrap_or("many")
}

fn push_of(arm: &Arm, q: &model::Q, joint: usize, wrist: V3) -> V3 {
    let (point, axis) = arm.joint(q, joint);
    cross(axis, sub(wrist, point))
}

/// The direction along the arm: from the shoulder to the wrist.
fn along_the_arm(arm: &Arm, q: &model::Q) -> V3 {
    unit(sub(arm.joint(q, WRIST).0, arm.joint(q, SHOULDER).0))
}

fn trackers(rig: &ArmRig) -> fmn::Result<[Mob; N]> {
    match &rig.source {
        QSource::Joints(t) => Ok(*t),
        QSource::Path { .. } => Err(fail("lost directions: the rig walks a path".into())),
    }
}

/// How many independent directions the arm the rig draws can move its tip
/// in, as it stands this frame.
fn rank_now(arm: &Arm, rig: &ArmRig, s: &Arena) -> usize {
    model::rank(&arm.jacobian(&rig.q(s)))
}

/// The "directions:" readout: its label, faded in, and its number, read off
/// the moving arm every frame (a live number comes in whole).
fn directions_readout(kit: &Kit, stage: &mut Stage<'_>, rig: &ArmRig, arm: &Arm) -> fmn::Result<(Mob, Mob)> {
    let head = kit.text_left(stage, DIRECTIONS, 40.0, Palette::dim(), v(READOUT_AT[0], READOUT_AT[1]))?;
    fade_in_all(stage, &[head], 0.5)?;
    let (probe, arm_for) = (rig.clone(), arm.clone());
    let number = kit.live_value(
        stage,
        move |s: &Arena| rank_now(&arm_for, &probe, s) as f64,
        0,
        false,
        54.0,
        Palette::ink(),
        v(READOUT_AT[0] + 3.0, READOUT_AT[1]),
        LEFT,
    )?;
    Ok((head, number))
}

/// Move a tracker to a value over `rt` seconds, as several at once.
fn tween_joints(stage: &mut Stage<'_>, moves: &[(Mob, f64)], rt: f64) -> fmn::Result<()> {
    super::draw::tween_all(stage, moves, rt)
}

impl SceneConstruct for LostDirections {
    fn name(&self) -> &str {
        "11_lost_directions"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let arm = Arm::iiwa14();
        let pull = fact_f64("G-J.blowup.pull")?;
        let limit = fact_f64("G-J.blowup.limit")?;
        let (orange, gold, red, blue) = (palette::flat(), palette::screw(), palette::warn(), palette::turn());
        let rig = ArmRig::joints(stage, arm.clone(), poses::ELL);
        let q = trackers(&rig)?;
        let (ell, straight) = (model::degrees(poses::ELL), model::degrees(poses::ELBOW_STRAIGHT));
        let view = View::new(ARM_SCALE, v(ARM_AT[0], ARM_AT[1]));

        // ---- the world
        let mut specs = rig.parts(palette::steel());
        specs.push(PartSpec::new(table([-0.9, -0.7], [1.0, 0.9], 0.0, 0.2), still(Pose::NOTHING)).layer(-2));
        // the six directions at the tip of the L shape
        let tip = arm.tip(&ell).p;
        let mut six = Vec::new();
        for axis in [model::X, model::Y, model::Z] {
            six.push(Shape::Arrow {
                from: tip,
                to: add(tip, scale(axis, 0.24)),
                color: orange,
                width: 4.5,
                head: 0.2,
            });
            six.extend(curl(tip, axis, 0.13, CURL_SWEEP, orange, 3.5));
        }
        specs.push(PartSpec::new(six, still(Pose::NOTHING)).layer(2));
        // the pushes of joints 1 to 4 on the wrist of the straight elbow,
        // each across the arm, and the lost direction along it
        let wrist = arm.joint(&straight, WRIST).0;
        for joint in PUSHERS {
            let push = push_of(&arm, &straight, joint, wrist);
            specs.push(
                PartSpec::new(
                    vec![Shape::Arrow {
                        from: wrist,
                        to: add(wrist, scale(push, 0.9)),
                        color: orange,
                        width: 5.0,
                        head: 0.22,
                    }],
                    still(Pose::NOTHING),
                )
                .layer(3),
            );
        }
        let along = along_the_arm(&arm, &straight);
        let lost_end = add(wrist, scale(along, 0.34));
        let cross_at = add(lost_end, scale(along, 0.06));
        let (e1, e2) = super::draw::perp_basis(along);
        let mark = |k: f64| -> Shape {
            Shape::line(
                add(cross_at, add(scale(e1, 0.05 * k), scale(e2, 0.05))),
                add(cross_at, add(scale(e1, -0.05 * k), scale(e2, -0.05))),
                red,
                4.0,
            )
        };
        specs.push(
            PartSpec::new(
                vec![
                    Shape::Dashed {
                        a: wrist,
                        b: lost_end,
                        color: red,
                        width: 4.5,
                        dashes: 5,
                    },
                    mark(1.0),
                    mark(-1.0),
                ],
                still(Pose::NOTHING),
            )
            .layer(3),
        );
        // the goal, dimmed: the robot's real question stays in view
        specs.extend(goal_parts(&arm));
        // labels that follow the arm
        let mut pins = Vec::new();
        let push_label = kit.text(stage, PUSH_NOTE, 24.0, Palette::dim(), v(0.0, 0.0))?;
        pins.push(Pin {
            mob: push_label,
            at: point_fn(move |_: &Arena| wrist),
            shift: v(PUSH_NOTE_SHIFT[0], PUSH_NOTE_SHIFT[1]),
        });
        let third_label = kit.text(stage, THIRD_JOINT, 26.0, gold, v(0.0, 0.0))?;
        let (third_at, _) = rig.axis_now(2);
        pins.push(Pin {
            mob: third_label,
            at: third_at,
            shift: v(THIRD_JOINT_SHIFT[0], THIRD_JOINT_SHIFT[1]),
        });
        let world = World::build(stage, view, 1.4, 0.25, specs, pins)?;
        let hidden: Vec<usize> = ArmRig::GOLD_COLLARS
            .chain([part::SIX, part::ALONG])
            .chain(part::PUSH_FIRST..part::PUSH_FIRST + PUSHERS.len())
            .collect();
        for i in hidden {
            stage.set_tracker_value(world.parts[i].shown, 0.0)?;
        }
        stage.set_tracker_value(world.parts[part::GOAL].shown, GOAL_DIMMED)?;
        world.add_to_scene(stage)?;
        let shown = |i: usize| world.parts[i].shown;
        let collar = |joint: usize| world.parts[ArmRig::GOLD_COLLARS.start + joint].shown;

        // 11_1: six rows allow six directions; usually seven joints cover
        // all six
        let line = say(&kit, stage, BEATS[0])?;
        world.show(stage, 0.8)?;
        line.wait_for(stage, "Six rows")?;
        let (head, number) = directions_readout(&kit, stage, &rig, &arm)?;
        let standing = standing_goal(&kit, stage)?;
        fade_in_all(stage, &[standing], 0.5)?;
        line.wait_for(stage, "six independent directions")?;
        play!(stage; tween(shown(part::SIX), 1.0, 0.8)?);
        let what = lower_third(&kit, stage, DIRECTIONS_LABEL, DIRECTIONS_TEXT)?;
        fade_in_all(stage, &[what], 0.5)?;
        line.wait_end(stage)?;

        // 11_2: the elbow straightens: can the gripper still move in all six?
        let line = say(&kit, stage, BEATS[1])?;
        // the readout is hidden while the question stands
        leave(stage, &[head, number, what], &[shown(part::SIX)], 0.4)?;
        let card = QuizCard::build(&kit, stage, ASK, QuizLook::at(v(CARD_AT[0], CARD_AT[1]), CARD_SCALE))?;
        line.wait_for(stage, "Here the arm's elbow is straight")?;
        tween_joints(
            stage,
            &[
                (q[SHOULDER], straight[SHOULDER]),
                (q[ELBOW], straight[ELBOW]),
                (q[WRIST], straight[WRIST]),
            ],
            2.0,
        )?;
        line.wait_last(stage)?;
        card.show(stage, 0.5)?;
        card.count(&kit, stage)?;

        // 11_3: no; along the arm every joint's push on the wrist is zero
        let line = say(&kit, stage, BEATS[2])?;
        // the readout comes back: five of six
        let _ = directions_readout(&kit, stage, &rig, &arm)?;
        line.wait_for(stage, "Along the arm")?;
        // each joint wiggles in turn, its push on the wrist drawn (joints 3
        // and 5 turn about lines through the wrist, which stays where it is)
        for &joint in &WIGGLERS {
            let tracker = q[joint];
            let at = value(stage.arena(), tracker);
            let push = PUSHERS
                .iter()
                .position(|&p| p == joint)
                .map(|k| shown(part::PUSH_FIRST + k));
            match push {
                Some(arrow) => play!(stage;
                    tween(arrow, 1.0, 0.3)?,
                    tween(tracker, at + WIGGLE, 0.35)?,
                ),
                None => play!(stage; tween(tracker, at + WIGGLE, 0.35)?),
            }
            play!(stage; tween(tracker, at - WIGGLE, 0.5)?);
            match push {
                Some(arrow) => play!(stage;
                    tween(tracker, at, 0.35)?,
                    tween(arrow, 0.35, 0.35)?,
                ),
                None => play!(stage; tween(tracker, at, 0.35)?),
            }
        }
        // then all four stand together, none of them along the arm
        let all_pushes: Vec<Mob> = (0..PUSHERS.len()).map(|k| shown(part::PUSH_FIRST + k)).collect();
        let together: Vec<(Mob, f64)> = all_pushes.iter().map(|&t| (t, 1.0)).collect();
        super::draw::tween_all(stage, &together, 0.4)?;
        fade_in_all(stage, &[push_label], 0.4)?;
        line.wait_for(stage, "So the gripper can't slide further out")?;
        play!(stage; tween(shown(part::ALONG), 1.0, 0.6)?);
        let swing = lower_third(&kit, stage, SWING_LABEL, SWING_SIDEWAYS)?;
        line.wait_for(stage, "Five of the six directions")?;
        fade_in_all(stage, &[swing], 0.5)?;
        line.wait_end(stage)?;

        // 11_4: a singularity; near one, the lost direction costs huge speeds
        let line = say(&kit, stage, BEATS[3])?;
        let mut away = all_pushes.clone();
        away.push(shown(part::ALONG));
        // the card has done its work: the gauge and the plot take its place
        let mut gone = vec![swing, push_label];
        gone.extend(card.mobs());
        leave(stage, &gone, &away, 0.5)?;
        let word = kit.text(stage, SINGULARITY, 60.0, Palette::ink(), v(3.4, 2.7))?;
        line.wait_for(stage, "A pose like this")?;
        fade_in_all(stage, &[word], 0.6)?;
        // the family of poses the speeds are for: shoulder and wrist at 90
        let family = model::degrees([0.0, 90.0, 0.0, 0.0, 0.0, 90.0, 0.0]);
        tween_joints(
            stage,
            &[
                (q[SHOULDER], family[SHOULDER]),
                (q[ELBOW], family[ELBOW]),
                (q[WRIST], family[WRIST]),
            ],
            1.4,
        )?;
        let not_only = lower_third(
            &kit,
            stage,
            NOT_ONLY_FULL_STRETCH,
            &format!(
                "not only at full stretch. Straight up with the elbow bent, this arm also has {} directions; straight up, {}",
                number_word(fact("G-J.rank.elbow_only")?.parse().map_err(|_| fail("rank".into()))?),
                number_word(fact("G-J.rank.home")?.parse().map_err(|_| fail("rank".into()))?),
            ),
        )?;
        fade_in_all(stage, &[not_only], 0.5)?;
        line.wait_for(stage, "Close to one")?;
        // the gauge: the elbow's speed for the pull, from the bend the arm has
        let speed = stage.add_value_tracker(SPEED_CUT);
        let anchor = stage.arena_mut().add(VMobject::new());
        {
            let arm_for = arm.clone();
            let bend = q[ELBOW];
            stage.add_updater(
                anchor,
                move |s, _| {
                    let b = (value(s, bend) / DEG).max(0.2);
                    let _ = s.set_tracker_value(speed, elbow_speed(&arm_for, b, pull));
                },
                true,
            )?;
        }
        stage.add_to_scene(anchor)?;
        let gauge_title = kit.text_left(
            stage,
            &format!("{ELBOW_SPEED} {} cm/s", (pull * 100.0).round() as i64),
            24.0,
            Palette::dim(),
            v(GAUGE_AT[0], GAUGE_AT[1] + 0.75),
        )?;
        let limit_x = GAUGE_AT[0] + GAUGE_UNIT * limit;
        let limit_line = polyline(vec![v(limit_x, GAUGE_AT[1] - 0.3), v(limit_x, GAUGE_AT[1] + 0.3)])
            .style(line_style(red, 4.0))
            .build();
        let limit_line = stage.arena_mut().add(limit_line);
        let limit_word = kit.text(
            stage,
            &format!("{LIMIT} {}", limit as i64),
            24.0,
            red,
            v(limit_x, GAUGE_AT[1] - 0.6),
        )?;
        fade_in_all(stage, &[gauge_title, limit_line, limit_word], 0.5)?;
        let shown_speed = stage.add_value_tracker(SPEED_CUT);
        let clip = stage.arena_mut().add(VMobject::new());
        stage.add_updater(
            clip,
            move |s, _| {
                let x = value(s, speed).min(SPEED_CUT);
                let _ = s.set_tracker_value(shown_speed, x);
            },
            true,
        )?;
        stage.add_to_scene(clip)?;
        let bar = explainer_kit::live_bar(stage, shown_speed, v(GAUGE_AT[0], GAUGE_AT[1]), GAUGE_UNIT, 0.3, orange)?;
        let reading = kit.live_count(stage, speed, 34.0, Palette::ink(), v(GAUGE_AT[0] + 4.4, GAUGE_AT[1]))?;
        let unit_word = kit.text(
            stage,
            DEG_PER_S,
            28.0,
            Palette::dim(),
            v(GAUGE_AT[0] + 5.35, GAUGE_AT[1]),
        )?;
        fade_in_all(stage, &[unit_word], 0.3)?;
        play!(stage; tween(q[ELBOW], FIRST_BEND_DEG * DEG, 2.2)?);
        line.wait_end(stage)?;

        // 11_5: pull the gripper straight back at ten centimeters a second
        let line = say(&kit, stage, BEATS[4])?;
        for m in [anchor, clip] {
            stage.clear_updaters(m, true);
        }
        leave(
            stage,
            &[
                gauge_title,
                limit_line,
                limit_word,
                bar,
                reading,
                unit_word,
                word,
                not_only,
            ],
            &[],
            0.5,
        )?;
        let samples = curve(&arm, pull);
        let frame_border = PLOT.border(Palette::faint())?;
        let border = stage.arena_mut().add(frame_border);
        let mut axes = vec![border];
        for t in [0.0, 10.0, 20.0, 30.0, 40.0] {
            let p = PLOT.p(t, 0.0);
            let label = kit.text(
                stage,
                &format!("{}", t as i64),
                22.0,
                Palette::dim(),
                v(p[0], p[1] - 0.25),
            )?;
            axes.push(label);
        }
        for s in [0.0, 50.0, 100.0, 150.0, 200.0] {
            let p = PLOT.p(0.0, s);
            let label = kit.text(
                stage,
                &format!("{}", s as i64),
                22.0,
                Palette::dim(),
                v(p[0] - 0.32, p[1]),
            )?;
            axes.push(label);
        }
        let x_title = kit.text(
            stage,
            BEND,
            24.0,
            Palette::dim(),
            v(PLOT.center[0], PLOT.bottom() - 0.62),
        )?;
        let y_title = kit.text(
            stage,
            SPEED,
            24.0,
            Palette::dim(),
            v(PLOT.left() + 0.35, PLOT.top() + 0.3),
        )?;
        axes.extend([x_title, y_title]);
        let trace = polyline(samples.iter().map(|&(b, s)| PLOT.p(b, s)).collect::<Vec<_>>())
            .style(line_style(orange, 5.0))
            .build();
        let trace = stage.arena_mut().add(trace);
        let limit_at = PLOT.p(0.0, limit);
        let limit_end = PLOT.p(BEND_MAX, limit);
        let limit_dashed = stage.arena_mut().add(
            DashedLine::new(limit_at, limit_end)
                .dash_length(0.2)
                .style(faded_line(red, 3.5, 1.0))
                .build()?,
        );
        let limit_label = kit.text(
            stage,
            &format!("{LIMIT} {}", limit as i64),
            24.0,
            red,
            v(PLOT.right() - 0.8, limit_at[1] + 0.25),
        )?;
        line.wait_for(stage, "Pull the gripper straight back")?;
        fade_in_all(stage, &axes, 0.6)?;
        fade_in_all(stage, &[trace], 1.0)?;
        // the two bends the voice names, the arm bending with the dot
        let dot_at = |deg: f64| PLOT.p(deg, elbow_speed(&arm, deg, pull));
        let marker = stage.arena_mut().add(
            Dot::new()
                .point(dot_at(30.0))
                .radius(0.1)
                .style(super::draw::flat(gold, 1.0))
                .build(),
        );
        let first = format!("30° : {} °/s", elbow_speed(&arm, 30.0, pull).round() as i64);
        // under its dot: above it the limit's dashed line runs through the words
        let first_label = kit.text(
            stage,
            &first,
            26.0,
            Palette::ink(),
            v(dot_at(30.0)[0], dot_at(30.0)[1] - 0.45),
        )?;
        line.wait_for(stage, "With the elbow bent thirty degrees")?;
        play!(stage; tween(q[ELBOW], 30.0 * DEG, 0.8)?);
        fade_in_all(stage, &[marker, first_label], 0.5)?;
        line.wait_for(stage, "Bent ten degrees")?;
        let ten = format!("10° : {} °/s", elbow_speed(&arm, 10.0, pull).round() as i64);
        let ten_label = kit.text_left(
            stage,
            &ten,
            26.0,
            Palette::ink(),
            v(
                dot_at(10.0)[0] + TEN_LABEL_OFFSET[0],
                dot_at(10.0)[1] + TEN_LABEL_OFFSET[1],
            ),
        )?;
        let ten_dot = stage.arena_mut().add(
            Dot::new()
                .point(dot_at(10.0))
                .radius(0.1)
                .style(super::draw::flat(gold, 1.0))
                .build(),
        );
        play!(stage; tween(q[ELBOW], 10.0 * DEG, 1.2)?);
        fade_in_all(stage, &[ten_dot, ten_label], 0.5)?;
        line.wait_for(stage, "Its limit is seventy-five")?;
        let bend_limit = fact_f64("G-J.blowup.bend_at_limit")?;
        let cross_dot = stage.arena_mut().add(
            Dot::new()
                .point(PLOT.p(bend_limit, limit))
                .radius(0.11)
                .style(super::draw::flat(red, 1.0))
                .build(),
        );
        let cross_label = kit.text(
            stage,
            &format!("{bend_limit:.1}°"),
            26.0,
            red,
            v(PLOT.p(bend_limit, limit)[0], PLOT.p(bend_limit, limit)[1] - 0.4),
        )?;
        fade_in_all(stage, &[limit_dashed, limit_label, cross_dot, cross_label], 0.6)?;
        play!(stage; tween(q[ELBOW], bend_limit * DEG, 0.8)?);
        line.wait_end(stage)?;
        // above the cut: the two last values, not said
        let high = |bend: f64| -> fmn::Result<String> {
            Ok(format!(
                "{bend:.0}° : {} °/s",
                per_second(fact_f64(&format!("G-J.blowup.{}", bend as i64))?)
            ))
        };
        let (a, b) = (high(3.0)?, high(1.0)?);
        let cut_a = kit.text_left(stage, &a, 26.0, Palette::ink(), v(PLOT.left() + 0.4, PLOT.top() + 0.75))?;
        let cut_b = kit.text_left(stage, &b, 26.0, Palette::ink(), v(PLOT.left() + 0.4, PLOT.top() + 1.15))?;
        let turns = fact("G-J.blowup.1.turns_per_s")?;
        let turns_label = kit.text_left(
            stage,
            &format!("{turns} {TURNS_A_SECOND}"),
            24.0,
            Palette::dim(),
            v(PLOT.left() + 3.4, PLOT.top() + 1.15),
        )?;
        let geometry = kit.text(
            stage,
            FROM_THE_GEOMETRY,
            24.0,
            Palette::dim(),
            v(PLOT.center[0], PLOT.bottom() - 1.0),
        )?;
        let limit_source = kit.text(
            stage,
            LIMIT_SOURCE,
            24.0,
            Palette::dim(),
            v(PLOT.center[0], PLOT.bottom() - 1.4),
        )?;
        fade_in_lagged(stage, &[cut_a, cut_b, turns_label, geometry, limit_source], 1.2, 0.5)?;
        kit.music_breath(stage, 1.0)?;

        // 11_6: the wrist is three joints whose axes meet at one point
        let line = say(&kit, stage, BEATS[5])?;
        let mut done = axes.clone();
        done.extend([
            trace,
            limit_dashed,
            limit_label,
            marker,
            ten_dot,
            first_label,
            ten_label,
            cross_dot,
            cross_label,
        ]);
        done.extend([cut_a, cut_b, turns_label, geometry, limit_source]);
        leave(stage, &done, &[], 0.6)?;
        // the pose the story ends on: the L shape with its wrist bent, so
        // that straightening it lines two axes up
        tween_joints(
            stage,
            &[
                (q[SHOULDER], ell[SHOULDER]),
                (q[ELBOW], ell[ELBOW]),
                (q[WRIST], 60.0 * DEG),
            ],
            1.2,
        )?;
        let close_view = {
            let w = arm.joint(&ell, WRIST).0;
            let (right, up, _) = View::basis();
            View::new(
                CLOSE_SCALE,
                v(
                    CLOSE_AT[0] - CLOSE_SCALE * dot(w, right),
                    CLOSE_AT[1] - CLOSE_SCALE * dot(w, up),
                ),
            )
        };
        let mut close_specs = rig.parts(palette::steel());
        let w_home = arm.origin[WRIST];
        for joint in WRIST_JOINTS {
            close_specs.push(
                PartSpec::new(
                    vec![Shape::line(
                        add(arm.origin[joint], scale(arm.axis[joint], -0.24)),
                        add(arm.origin[joint], scale(arm.axis[joint], 0.24)),
                        gold,
                        4.5,
                    )],
                    rig.carried(joint),
                )
                .layer(2),
            );
        }
        for joint in WRIST_JOINTS {
            close_specs.push(
                PartSpec::new(
                    vec![Shape::circle(arm.origin[joint], arm.axis[joint], 0.15, 36, blue, 3.5)],
                    rig.carried(joint),
                )
                .layer(1),
            );
        }
        close_specs.push(
            PartSpec::new(
                vec![Shape::Dot {
                    at: w_home,
                    color: Palette::ink(),
                    radius: 0.06,
                }],
                rig.carried(WRIST),
            )
            .layer(3),
        );
        let near = World::build(stage, close_view, 1.4, 0.25, close_specs, Vec::new())?;
        let near_hidden: Vec<usize> = ArmRig::GOLD_COLLARS
            .chain(close::AXES)
            .chain(close::RINGS)
            .chain([close::POINT])
            .collect();
        for i in near_hidden {
            stage.set_tracker_value(near.parts[i].shown, 0.0)?;
        }
        near.add_to_scene(stage)?;
        line.wait_for(stage, "The wrist is three joints")?;
        near.show(stage, 0.8)?;
        for axis in close::AXES {
            play!(stage; tween(near.parts[axis].shown, 1.0, 0.5)?);
        }
        line.wait_for(stage, "whose axes meet at one point")?;
        play!(stage; tween(near.parts[close::POINT].shown, 1.0, 0.5)?);
        line.wait_for(stage, "a gimbal")?;
        for ring in close::RINGS {
            play!(stage; tween(near.parts[ring].shown, 1.0, 0.4)?);
        }
        line.wait_for(stage, "Straighten it")?;
        play!(stage; tween(q[WRIST], 0.0, 2.4)?);
        line.wait_for(stage, "A six-joint arm")?;
        play!(stage;
            tween(collar(0), 1.0, 0.4)?,
            tween(collar(SHOULDER), 1.0, 0.4)?,
        );
        let without = fact("G-J.rank.ell.without_3")?;
        let lose = kit.text(stage, &format!("{WITHOUT_THREE} {without}"), 30.0, red, v(-3.6, 1.7))?;
        let two_words = kit.text(stage, SHOULDER_TWO, 26.0, Palette::dim(), v(-3.6, 1.2))?;
        line.wait_for(stage, "would lose a direction here")?;
        fade_in_all(stage, &[lose, two_words], 0.5)?;
        line.wait_for(stage, "This arm has a third")?;
        play!(stage;
            tween(collar(0), 0.0, 0.3)?,
            tween(collar(SHOULDER), 0.0, 0.3)?,
            tween(collar(2), 1.0, 0.4)?,
            fade_in(stage.arena_mut(), third_label, ORIGIN, 1.0)?.rt(0.4),
        );
        line.wait_end(stage)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::norm;

    fn run(construct: impl FnMut(&mut Stage<'_>) -> fmn::Result<()>) {
        struct Probe<F>(F);
        impl<F: FnMut(&mut Stage<'_>) -> fmn::Result<()>> SceneConstruct for Probe<F> {
            fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
                (self.0)(stage)
            }
        }
        fmn::run_scene(
            &mut Probe(construct),
            fmn::scene::RuntimeConfig::default(),
            7,
            &mut fmn::scene::NullSceneSink,
        )
        .expect("the probe scene runs");
    }

    #[test]
    fn the_beats_are_the_storyboards_and_the_scripts() {
        let board = explainer_kit::storyboard::parse(crate::STORYBOARD).unwrap();
        let chapter = board
            .iter()
            .find(|c| c.key == "11_lost_directions")
            .expect("chapter 11");
        let ids: Vec<&str> = chapter.beats.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(ids, BEATS);
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for beat in &chapter.beats {
            let words = beat.spoken().expect("every beat of chapter 11 is spoken");
            assert_eq!(script.text(&beat.id), Some(words.as_str()), "{}", beat.id);
        }
    }

    #[test]
    fn every_cue_phrase_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("11_1", "Six rows"),
            ("11_1", "six independent directions"),
            ("11_2", "Here the arm's elbow is straight"),
            ("11_3", "Along the arm"),
            ("11_3", "So the gripper can't slide further out"),
            ("11_3", "Five of the six directions"),
            ("11_4", "A pose like this"),
            ("11_4", "Close to one"),
            ("11_5", "Pull the gripper straight back"),
            ("11_5", "With the elbow bent thirty degrees"),
            ("11_5", "Bent ten degrees"),
            ("11_5", "Its limit is seventy-five"),
            ("11_6", "The wrist is three joints"),
            ("11_6", "whose axes meet at one point"),
            ("11_6", "a gimbal"),
            ("11_6", "Straighten it"),
            ("11_6", "A six-joint arm"),
            ("11_6", "would lose a direction here"),
            ("11_6", "This arm has a third"),
        ] {
            let text = script.text(id).unwrap_or_default();
            assert!(text.contains(phrase), "{id} does not say {phrase:?}");
        }
    }

    /// The readout's numbers are the ranks the facts pinned, at the named
    /// poses the chapter visits.
    #[test]
    fn the_directions_readout_is_the_facts_rank_at_each_pose() {
        let arm = Arm::iiwa14();
        let rank = |p: [f64; N]| model::rank(&arm.jacobian(&model::degrees(p)));
        for (pose, key) in [
            (poses::ELL, "G-J.rank.ell"),
            (poses::ELBOW_STRAIGHT, "G-J.rank.elbow_straight"),
            (poses::ELBOW_ONLY, "G-J.rank.elbow_only"),
            (poses::HOME, "G-J.rank.home"),
        ] {
            assert_eq!(rank(pose).to_string(), fact(key).unwrap(), "{key}");
        }
        assert_eq!(rank(poses::ELL), 6);
        assert_eq!(rank(poses::ELBOW_STRAIGHT), 5);
        // 11_4's family of poses: elbow straight is singular, a little bend is not
        assert_eq!(rank([0.0, 90.0, 0.0, 0.0, 0.0, 90.0, 0.0]), 5);
        assert_eq!(rank([0.0, 90.0, 0.0, 30.0, 0.0, 90.0, 0.0]), 6);
    }

    /// 11_2's quiz: four seconds, and the lost direction is computed from
    /// the rank of the straight-elbow pose.
    #[test]
    fn the_quiz_answer_is_the_lost_direction() {
        assert_eq!(ASK.seconds().unwrap(), 4);
        assert_eq!(ASK.choices[ASK.answer_index().unwrap()].label, ONE_IS_LOST);
        let arm = Arm::iiwa14();
        let rank = model::rank(&arm.jacobian(&model::degrees(poses::ELBOW_STRAIGHT)));
        assert!(rank < 6, "a direction is lost");
    }

    /// 11_3: at the straight elbow, no joint's push on the wrist has any
    /// part along the arm, though each push is a real push across it; and at
    /// the bent L shape the same joints do push along the arm.
    #[test]
    fn no_joints_push_on_the_wrist_is_along_the_arm_when_the_elbow_is_straight() {
        let arm = Arm::iiwa14();
        let q = model::degrees(poses::ELBOW_STRAIGHT);
        let wrist = arm.joint(&q, WRIST).0;
        let along = along_the_arm(&arm, &q);
        for joint in PUSHERS {
            let push = push_of(&arm, &q, joint, wrist);
            assert!(
                dot(push, along).abs() < 1e-12,
                "joint {} pushes along the arm",
                joint + 1
            );
            assert!(norm(push) > 0.05, "joint {} pushes at all", joint + 1);
        }
        // joints 3 and 5 turn about lines through the wrist: no push at all,
        // so they are not drawn
        for joint in [2, 4] {
            let (p, a) = arm.joint(&q, joint);
            assert!(
                norm(cross(a, sub(wrist, p))) < 1e-12,
                "joint {} turns about a line through the wrist",
                joint + 1
            );
            assert!(!PUSHERS.contains(&joint));
        }
        assert!(WIGGLERS.iter().all(|j| PUSHERS.contains(j) || [2, 4].contains(j)));
        // against the facts
        assert_eq!(fact("G-J.elbow_straight.push_along_arm").unwrap(), "0.0000");
        assert!(arm.push_along_arm(&q) < 1e-12);
        let bent = model::degrees(poses::SWING);
        assert!(arm.push_along_arm(&bent) > 0.2, "bent, a joint does push along the arm");
        // and the arrows in the picture are the ones computed: the shapes
        // are built from `push_of`, scaled by 0.9
        let push = push_of(&arm, &q, 1, wrist);
        assert!(norm(push) > 0.3, "the shoulder's push is a long arrow");
    }

    /// 11_5's curve: its points at the voice's bends are the facts', it
    /// falls as the bend grows, the cut keeps everything above 200 out, and
    /// the crossing is where the fact says.
    #[test]
    fn the_curve_is_the_facts_and_crosses_the_limit_where_they_say() {
        let arm = Arm::iiwa14();
        let pull = fact_f64("G-J.blowup.pull").unwrap();
        for bend in [30.0, 10.0, 3.0, 1.0] {
            let want = fact_f64(&format!("G-J.blowup.{}", bend as i64)).unwrap();
            assert!((elbow_speed(&arm, bend, pull) - want).abs() < 0.005, "{bend}");
        }
        let c = curve(&arm, pull);
        assert!(
            c.windows(2).all(|w| w[1].1 < w[0].1),
            "the speed falls as the bend grows"
        );
        assert!(c.iter().all(|&(_, s)| s <= SPEED_CUT));
        assert!(c[0].0 > 3.0, "the cut hides the values above 200");
        let limit = fact_f64("G-J.blowup.limit").unwrap();
        let at = c
            .windows(2)
            .find(|w| w[0].1 > limit && w[1].1 <= limit)
            .expect("a crossing");
        let bend = fact_f64("G-J.blowup.bend_at_limit").unwrap();
        assert!((at[0].0 - bend).abs() < 0.3, "{} against {bend}", at[0].0);
        assert_eq!(per_second(fact_f64("G-J.blowup.1").unwrap()), "1,602");
        assert_eq!(per_second(fact_f64("G-J.blowup.3").unwrap()), "534");
        assert_eq!(per_second(fact_f64("G-J.blowup.30").unwrap()), "54");
        assert_eq!(per_second(fact_f64("G-J.blowup.10").unwrap()), "160");
        // "four and a half turns a second"
        assert_eq!(fact("G-J.blowup.1.turns_per_s").unwrap(), "4.45");
        assert!((fact_f64("G-J.blowup.1").unwrap() / 360.0 - 4.45).abs() < 0.005);
    }

    /// 11_6: three joints whose axes meet at one point, at every pose
    /// tried; and straightened (joint 6 at zero) two of them line up, for
    /// any angles of the joints before it.
    #[test]
    fn the_wrists_axes_meet_and_line_up_when_straight() {
        let arm = Arm::iiwa14();
        let poses_tried = [
            poses::HOME,
            poses::ELL,
            poses::SWING,
            poses::START,
            poses::ELBOW_STRAIGHT,
            [12.0, -33.0, 70.0, 41.0, -80.0, 25.0, 140.0],
            [-100.0, 55.0, -20.0, -75.0, 130.0, -50.0, 10.0],
        ];
        for p in poses_tried {
            let q = model::degrees(p);
            let w = arm.joint(&q, WRIST).0;
            for joint in WRIST_JOINTS {
                let (point, axis) = arm.joint(&q, joint);
                let off = norm(cross(axis, sub(w, point)));
                assert!(off < 1e-9, "axis {} misses the wrist by {off}", joint + 1);
            }
        }
        for p in [
            [0.0, 90.0, 0.0, 90.0, 0.0, 0.0, 0.0],
            [30.0, -40.0, 100.0, 20.0, -70.0, 0.0, 15.0],
            [-120.0, 10.0, -60.0, -90.0, 33.0, 0.0, 170.0],
        ] {
            let q = model::degrees(p);
            let (a5, a7) = (arm.joint(&q, 4).1, arm.joint(&q, 6).1);
            assert!(norm(cross(a5, a7)) < 1e-9, "5 and 7 line up when 6 is straight");
        }
        // bent, they do not
        let q = model::degrees([0.0, 90.0, 0.0, 90.0, 0.0, 60.0, 0.0]);
        assert!(norm(cross(arm.joint(&q, 4).1, arm.joint(&q, 6).1)) > 0.5);
    }

    /// 11_6: a six-joint arm that loses one of the shoulder's joints loses a
    /// direction here; losing the wrist's spare does not.
    #[test]
    fn the_third_shoulder_joint_covers_for_a_lost_direction() {
        assert_eq!(fact("G-J.rank.ell.without_3").unwrap(), "5");
        assert_eq!(fact("G-J.rank.ell.without_7").unwrap(), "6");
        let arm = Arm::iiwa14();
        let j = arm.jacobian(&model::degrees(poses::ELL));
        for joint in 0..3 {
            let mut without = j;
            for row in &mut without {
                row[joint] = 0.0;
            }
            assert_eq!(model::rank(&without), 5, "without joint {}", joint + 1);
        }
        assert_eq!(number_word(5), "five");
        assert_eq!(number_word(3), "three");
        assert_eq!(fact("G-J.rank.elbow_only").unwrap(), "5");
        assert_eq!(fact("G-J.rank.home").unwrap(), "3");
        let reach = fact_f64("G-L.kuka.reach_mm").unwrap() / 1000.0;
        assert!(
            fact_f64("G-J.elbow_only.reach").unwrap() < reach,
            "singular well inside the reach"
        );
    }

    /// Where the arm's links are on the screen at joint angles `q`
    /// (degrees): a dozen points along each link, base to tip.
    fn links_on_screen(arm: &Arm, view: &View, q: [f64; N]) -> Vec<[f64; 2]> {
        let q = model::degrees(q);
        let mut joints: Vec<V3> = (0..N).map(|j| arm.joint(&q, j).0).collect();
        joints.insert(0, [0.0, 0.0, 0.0]); // the foot under joint 1
        joints.push(arm.tip(&q).p);
        let mut out = Vec::new();
        for pair in joints.windows(2) {
            for k in 0..=12 {
                let t = f64::from(k) / 12.0;
                let p = view.project(add(scale(pair[0], 1.0 - t), scale(pair[1], t)));
                out.push([p[0], p[1]]);
            }
        }
        out
    }

    fn box_of(stage: &Stage<'_>, m: Mob) -> [f64; 4] {
        [
            stage.get_left(m)[0],
            stage.get_right(m)[0],
            stage.get_bottom(m)[1],
            stage.get_top(m)[1],
        ]
    }

    fn inside(p: [f64; 2], b: [f64; 4], margin: f64) -> bool {
        p[0] > b[0] - margin && p[0] < b[1] + margin && p[1] > b[2] - margin && p[1] < b[3] + margin
    }

    /// 11_3: "push on the wrist" stands clear of the pushes' arrows and the
    /// lost direction's dashes, which all leave the wrist; and 11_6's "joint
    /// 3" clear of the arm's links.
    #[test]
    fn the_pinned_words_stand_clear_of_what_they_name() {
        run(|stage| {
            let kit = Kit::new()?;
            let arm = Arm::iiwa14();
            let view = View::new(ARM_SCALE, v(ARM_AT[0], ARM_AT[1]));
            // 11_3's picture: the straight elbow, four arrows and the dashes
            let q = model::degrees(poses::ELBOW_STRAIGHT);
            let wrist = arm.joint(&q, WRIST).0;
            let here = view.project(wrist);
            let label = kit.text(
                stage,
                PUSH_NOTE,
                24.0,
                Palette::dim(),
                v(here[0] + PUSH_NOTE_SHIFT[0], here[1] + PUSH_NOTE_SHIFT[1]),
            )?;
            let b = box_of(stage, label);
            let mut ends: Vec<V3> = PUSHERS
                .iter()
                .map(|&j| add(wrist, scale(push_of(&arm, &q, j, wrist), 0.9)))
                .collect();
            ends.push(add(wrist, scale(along_the_arm(&arm, &q), 0.4)));
            for end in ends {
                let to = view.project(end);
                for k in 0..=40 {
                    let t = f64::from(k) / 40.0;
                    let p = [here[0] + (to[0] - here[0]) * t, here[1] + (to[1] - here[1]) * t];
                    assert!(!inside(p, b, 0.03), "'push on the wrist' sits on a push at {p:?}");
                }
            }
            assert!(b[1] < 7.11 - 0.15);
            // "joint 3" on the L shape, where it appears
            let third = kit.text(stage, THIRD_JOINT, 26.0, palette::screw(), v(0.0, 0.0))?;
            let (half_w, half_h) = (stage.get_width(third) / 2.0, stage.get_height(third) / 2.0);
            let ell_q = model::degrees(poses::ELL);
            let axis_at = view.project(arm.joint(&ell_q, 2).0);
            let centre = [axis_at[0] + THIRD_JOINT_SHIFT[0], axis_at[1] + THIRD_JOINT_SHIFT[1]];
            let words = [
                centre[0] - half_w,
                centre[0] + half_w,
                centre[1] - half_h,
                centre[1] + half_h,
            ];
            for p in links_on_screen(&arm, &view, poses::ELL) {
                assert!(!inside(p, words, 0.08), "'joint 3' sits on the arm at {p:?}");
            }
            Ok(())
        });
    }

    /// 11_1 to 11_5: the standing words under the dimmed goal stand clear of
    /// the readout and of 11_4's "singularity"; 11_5's two lines under the
    /// plot end inside the margin and above the frame's bottom; the shoulder
    /// words of 11_6 fit left of the close-up.
    #[test]
    fn the_added_words_fit() {
        run(|stage| {
            let kit = Kit::new()?;
            let stand = standing_goal(&kit, stage)?;
            let head = kit.text_left(stage, DIRECTIONS, 40.0, Palette::dim(), v(READOUT_AT[0], READOUT_AT[1]))?;
            assert!(
                stage.get_top(stand)[1] + 0.1 < stage.get_bottom(head)[1],
                "the standing words touch the readout's heading"
            );
            let singularity = kit.text(stage, SINGULARITY, 60.0, Palette::ink(), v(3.4, 2.7))?;
            assert!(stage.get_right(stand)[0] + 0.3 < stage.get_left(singularity)[0]);
            for (words, y) in [
                (FROM_THE_GEOMETRY, PLOT.bottom() - 1.0),
                (LIMIT_SOURCE, PLOT.bottom() - 1.4),
            ] {
                let m = kit.text(stage, words, 24.0, Palette::dim(), v(PLOT.center[0], y))?;
                assert!(
                    stage.get_right(m)[0] < 7.11 - 0.15,
                    "{words:?} ends at {}",
                    stage.get_right(m)[0]
                );
                assert!(
                    stage.get_left(m)[0] > PLOT.left() - 0.5,
                    "{words:?} starts at {}",
                    stage.get_left(m)[0]
                );
                assert!(stage.get_bottom(m)[1] > -3.85);
            }
            let two = kit.text(stage, SHOULDER_TWO, 26.0, Palette::dim(), v(-3.6, 1.2))?;
            assert!(stage.get_right(two)[0] < -1.0 && stage.get_left(two)[0] > -7.11 + 0.15);
            Ok(())
        });
    }

    /// 11_5: the words at the ten-degree point ("10° : 160 °/s") stand clear
    /// of the curve and of the point's own dot, and inside the plot (the
    /// narrated draft had the curve running through the "10°").
    #[test]
    fn the_ten_degree_label_stands_clear_of_the_curve() {
        let arm = Arm::iiwa14();
        let pull = fact_f64("G-J.blowup.pull").unwrap();
        let points: Vec<[f64; 2]> = curve(&arm, pull)
            .iter()
            .map(|&(b, s)| {
                let p = PLOT.p(b, s);
                [p[0], p[1]]
            })
            .collect();
        run(|stage| {
            let kit = Kit::new()?;
            let dot = PLOT.p(10.0, elbow_speed(&arm, 10.0, pull));
            let words = format!("10° : {} °/s", elbow_speed(&arm, 10.0, pull).round() as i64);
            let label = kit.text_left(
                stage,
                &words,
                26.0,
                Palette::ink(),
                v(dot[0] + TEN_LABEL_OFFSET[0], dot[1] + TEN_LABEL_OFFSET[1]),
            )?;
            let b = box_of(stage, label);
            for p in &points {
                assert!(!inside(*p, b, 0.04), "the curve runs through the label at {p:?}");
            }
            assert!(!inside([dot[0], dot[1]], b, 0.12), "the label sits on its own dot");
            assert!(b[1] < PLOT.right(), "the label ends at {:.2}, inside the plot", b[1]);
            assert!(b[3] < PLOT.top(), "the label's top is inside the plot");
            Ok(())
        });
    }

    /// The plot's frame sits inside the screen, clear of the arm, with room
    /// above for the two values above the cut.
    #[test]
    fn the_plot_fits_the_frame() {
        assert!(PLOT.right() < 7.0 && PLOT.left() > -1.0);
        assert!(PLOT.top() + 1.4 < 4.0, "the cut values fit above the plot");
        assert!(PLOT.bottom() - 1.1 > -4.0, "its titles fit below");
        run(|stage| {
            let kit = Kit::new()?;
            let card = QuizCard::build(&kit, stage, ASK, QuizLook::at(v(CARD_AT[0], CARD_AT[1]), CARD_SCALE))?;
            let right = stage.get_right(card.back)[0];
            for &row in &card.rows {
                assert!(stage.get_right(row)[0] < right - 0.15);
            }
            assert!(right < 7.1);
            // two lines at the card's full size, and the card inside the margin
            assert!(ASK.question.contains('\n'));
            assert!(stage.get_top(card.back)[1] < 3.85 && stage.get_bottom(card.back)[1] > -3.85);
            let asked = box_of(stage, card.question);
            assert!(
                asked[1] - asked[0] < stage.get_width(card.back) - 0.2,
                "the question fits the card"
            );
            let m = kit.text(
                stage,
                &format!("1° : {} °/s", per_second(1602.34)),
                26.0,
                Palette::ink(),
                v(0.0, 0.0),
            )?;
            assert!(stage.get_width(m) < 3.0);
            Ok(())
        });
    }
}
