//! Chapter 12, the spare joint: seven joints for six directions leaves one
//! spare (12_1); lay your palm on a desk and keep your shoulder still, can
//! your elbow move (12_2, 12_3: it swings on an arc, and counted the way a
//! roboticist counts your arm has seven joints too); the robot does the
//! same, its gripper holding one pose while its elbow swings on a circle
//! twenty-nine centimeters in radius (12_4); and that is what the spare
//! joint buys, the elbow out of the way of an obstacle while the gripper
//! keeps working (12_5).

use explainer_kit::props::lower_third;
use explainer_kit::{Kit, Palette, Timed, play, tween, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::draw::{
    ARM_SCALE, ArmRig, Ask, Choice, PartSpec, Pin, QuizCard, QuizLook, Shape, View, World, cuboid, fact, fact_f64,
    fade_in_all, greyed, ground, leave, mix, paper, plate, point_fn, pose_fn, say, still, table, turned_about, value,
};
use crate::model::{
    self, Arm, ELBOW, N, Pose, SHOULDER, V3, WRIST, add, apply, cross, dot, norm, poses, scale, sub, unit,
};
use crate::palette;

pub struct TheSpareJoint {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const CAN_YOUR_ELBOW: &str = "can your elbow still move?";
const IT_CAN: &str = "it can";
const IT_CANT: &str = "it can't";
const GRIPPER_MOVED: &str = "gripper moved:";
const CM: &str = "cm";
const SHOULDER_NAME: &str = "shoulder";
const ELBOW_NAME: &str = "elbow";
const WRIST_NAME: &str = "wrist";
const NULL_SPACE_LABEL: &str = "the null space of the Jacobian:";
const NULL_SPACE_TEXT: &str = "joint speeds whose twists add up to nothing";

/// The beats of the chapter, in order (storyboard ids).
const BEATS: [&str; 5] = ["12_1", "12_2", "12_3", "12_4", "12_5"];

/// 12_2's quiz (`G-Q.12_2.answer` is `moves`).
pub const ASK: Ask = Ask {
    line: BEATS[1],
    question: CAN_YOUR_ELBOW,
    choices: &[
        Choice {
            key: "moves",
            label: IT_CAN,
        },
        Choice {
            key: "fixed",
            label: IT_CANT,
        },
    ],
};

/// A person's arm, counted the way a roboticist counts: the joints at the
/// shoulder, the elbow and the wrist (0-based ranges into the robot's seven).
const COUNTED: [(&str, std::ops::Range<usize>); 3] = [(SHOULDER_NAME, 0..3), (ELBOW_NAME, 3..4), (WRIST_NAME, 4..7)];

const ARM_AT: [f64; 2] = [-3.1, -2.55];
const CARD_AT: [f64; 2] = [4.45, 0.0];
const CARD_SCALE: f64 = 0.92;
/// 12_4's readout: its label's left edge and height, the right edge of the
/// live number, and where "cm" is centred, left to right without overlap.
const MOVED_AT: [f64; 2] = [0.75, 1.9];
const MOVED_NUMBER_RIGHT: f64 = 3.7;
const MOVED_CM_X: f64 = 4.15;
/// The pinned shoulder and the palm on the desk, in the person's picture
/// (metres, the picture's own frame).
const SHOULDER_AT: V3 = [0.0, 0.0, 0.62];
const PALM_AT: V3 = [0.35, 0.0, 0.0];
/// How far the person's elbow swings each way, radians.
const PERSON_SWING: f64 = 0.8;
/// The null-space walk: radians a step, and how many steps at most.
const STEP: f64 = 0.01;
const MAX_STEPS: usize = 2000;
/// An arrow's drawn length per metre a second of tip speed.
const ARROW_PER_SPEED: f64 = 1.1;
/// Half the obstacle's side, and how far outside the elbow it starts (m).
const OBSTACLE_HALF: f64 = 0.075;
const OBSTACLE_FROM: f64 = 0.9;

/// The parts this chapter adds to the arm's own 23 (the table is 23).
mod part {
    pub const OBSTACLE: usize = 24;
    /// The circle the elbow can swing on, and the part of it the joints
    /// reach, traced.
    pub const CIRCLE: usize = 25;
    pub const TRACE: usize = 26;
    /// The seven arrows at the gripper: 27 to 33.
    pub const ARROWS: std::ops::Range<usize> = 27..34;
    /// The circle's radius, a line from its centre to the elbow.
    pub const RADIUS: usize = 34;
}

/// The person's parts that are singled out: the elbow's circle, after the
/// desk, the palm, the two bones and the shoulder's pin.
mod person {
    pub const CIRCLE: usize = 5;
}

/// Where the elbow stands between a pinned shoulder and a hand on a desk,
/// for upper arm `a` and forearm `b`: its place at its lowest, the centre
/// and radius of the circle it can swing on, and the axis of that circle.
#[derive(Clone, Copy, Debug)]
pub struct Triangle {
    pub elbow: V3,
    pub centre: V3,
    pub radius: f64,
    pub axis: V3,
}

/// The elbow of a two-bone arm with the far ends at `s` and `p`, hanging
/// below the line from one to the other.
pub fn triangle(s: V3, p: V3, a: f64, b: f64) -> Triangle {
    let (axis, d) = (unit(sub(p, s)), norm(sub(p, s)));
    let along = (a * a - b * b + d * d) / (2.0 * d);
    let radius = (a * a - along * along).max(0.0).sqrt();
    let centre = add(s, scale(axis, along));
    let down = [0.0, 0.0, -1.0];
    let n = unit(sub(down, scale(axis, dot(down, axis))));
    Triangle {
        elbow: add(centre, scale(n, radius)),
        centre,
        radius,
        axis,
    }
}

/// The whole centimeters the voice says.
fn cm_whole(metres: f64) -> String {
    format!("{}", (metres * 100.0).round() as i64)
}

/// The tally of 12_1 from the facts: joints, directions, and the spare.
fn tally() -> fmn::Result<(String, String, String)> {
    Ok((
        fact("G-A.joints")?.to_owned(),
        fact("G-J.rank.swing")?.to_owned(),
        fact("G-J.spare")?.to_owned(),
    ))
}

/// The joint angles along the elbow's whole swing, from one end of the
/// reachable arc to the other, and where the start pose sits among them
/// (0 to 1).
pub fn swing_keys(arm: &Arm, start: &model::Q) -> (Vec<model::Q>, f64) {
    let up = arm.swing(start, STEP, 1.0, MAX_STEPS);
    let down = arm.swing(start, STEP, -1.0, MAX_STEPS);
    let mut keys: Vec<model::Q> = down.iter().rev().copied().collect();
    keys.extend(up.iter().skip(1).copied());
    let at = (down.len() - 1) as f64 / (keys.len() - 1) as f64;
    (keys, at)
}

impl SceneConstruct for TheSpareJoint {
    fn name(&self) -> &str {
        "12_the_spare_joint"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let arm = Arm::iiwa14();
        let (orange, gold, steel) = (palette::flat(), palette::screw(), palette::steel());
        let q0 = model::degrees(poses::SWING);
        let view = View::new(ARM_SCALE, v(ARM_AT[0], ARM_AT[1]));
        let upper = fact_f64("G-A.upper_arm")?;
        let forearm = fact_f64("G-A.forearm")?;

        // ---- the person: a pinned shoulder, a palm on the desk, an elbow
        // that swings about the line between them
        let t = triangle(SHOULDER_AT, PALM_AT, upper, forearm);
        let swing = stage.add_value_tracker(0.0);
        let local = |p: V3| sub(p, SHOULDER_AT);
        let carried = turned_about(SHOULDER_AT, vec![(t.axis, swing, 1.0)]);
        let edge = mix(steel, ground(), 0.7);
        let bone = |a: V3, b: V3| -> Vec<Shape> { vec![Shape::line(a, b, edge, 17.0), Shape::line(a, b, steel, 12.0)] };
        let faint_gold = mix(gold, ground(), 0.35);
        let person_specs = vec![
            PartSpec::new(table([-0.5, -0.5], [0.95, 0.5], 0.0, 0.2), still(Pose::NOTHING)).layer(-2),
            PartSpec::new(
                vec![plate(
                    [PALM_AT[0] - 0.1, PALM_AT[1] - 0.07],
                    [PALM_AT[0] + 0.12, PALM_AT[1] + 0.07],
                    0.004,
                    paper(),
                    0.02,
                )],
                still(Pose::NOTHING),
            ),
            PartSpec::new(bone([0.0; 3], local(t.elbow)), carried.clone()),
            PartSpec::new(bone(local(t.elbow), local(PALM_AT)), carried.clone()),
            PartSpec::new(
                vec![
                    Shape::Dot {
                        at: SHOULDER_AT,
                        color: gold,
                        radius: 0.11,
                    },
                    Shape::circle(SHOULDER_AT, [0.0, 1.0, 0.0], 0.1, 20, Palette::ink(), 3.0),
                ],
                still(Pose::NOTHING),
            )
            .layer(2),
            PartSpec::new(
                vec![Shape::circle(t.centre, t.axis, t.radius, 56, faint_gold, 3.0)],
                still(Pose::NOTHING),
            )
            .layer(1),
            PartSpec::new(
                vec![Shape::Dot {
                    at: local(t.elbow),
                    color: gold,
                    radius: 0.09,
                }],
                carried,
            )
            .layer(2),
        ];
        // where the elbow is this frame: its place at rest turned about the
        // line from the shoulder to the hand
        let elbow_at = {
            let (elbow, axis) = (t.elbow, t.axis);
            point_fn(move |a: &Arena| {
                add(
                    SHOULDER_AT,
                    apply(&model::turn(axis, value(a, swing)), sub(elbow, SHOULDER_AT)),
                )
            })
        };
        let mut person_pins = Vec::new();
        let mut counts = Vec::new();
        for ((name, joints), shift) in COUNTED.iter().zip([v(-0.75, 0.35), v(0.7, 0.45), v(0.1, 0.55)]) {
            let m = kit.text(stage, &format!("{}", joints.len()), 40.0, gold, v(0.0, 0.0))?;
            let at = match *name {
                SHOULDER_NAME => {
                    let s = SHOULDER_AT;
                    point_fn(move |_: &Arena| s)
                }
                ELBOW_NAME => elbow_at.clone(),
                _ => {
                    let p = PALM_AT;
                    point_fn(move |_: &Arena| p)
                }
            };
            person_pins.push(Pin { mob: m, at, shift });
            counts.push(m);
        }
        let person_view = View::new(ARM_SCALE, v(ARM_AT[0] + 0.2, ARM_AT[1]));
        let person_world = World::build(stage, person_view, 1.0, 0.2, person_specs, person_pins)?;
        stage.set_tracker_value(person_world.parts[person::CIRCLE].shown, 0.0)?;
        person_world.add_to_scene(stage)?;

        // ---- the robot: its own swing, walked along a table of poses
        let (keys, start_at) = swing_keys(&arm, &q0);
        let (rig, walk) = ArmRig::along(stage, arm.clone(), keys.clone());
        stage.set_tracker_value(walk, start_at)?;
        let mut specs = rig.parts(steel);
        specs.push(PartSpec::new(table([-0.9, -0.7], [1.0, 0.9], 0.0, 0.2), still(Pose::NOTHING)).layer(-2));
        let (shoulder_pt, wrist_pt) = (arm.joint(&q0, SHOULDER).0, arm.joint(&q0, WRIST).0);
        let elbow0 = arm.joint(&q0, ELBOW).0;
        let axis = unit(sub(wrist_pt, shoulder_pt));
        let radius = arm.elbow_radius(&q0);
        let centre = {
            let r = sub(elbow0, shoulder_pt);
            add(shoulder_pt, scale(axis, dot(r, axis)))
        };
        // the obstacle comes in from outside the elbow's circle, along the
        // direction the elbow is on
        let out = unit(sub(elbow0, centre));
        let obstacle_tracker = stage.add_value_tracker(0.0);
        let obstacle_pose = pose_fn(move |a: &Arena| {
            let k = value(a, obstacle_tracker);
            Pose {
                r: model::I3,
                p: add(elbow0, scale(out, OBSTACLE_FROM * (1.0 - k))),
            }
        });
        specs.push(
            PartSpec::new(
                cuboid([0.0; 3], [OBSTACLE_HALF; 3], [greyed(); 6], Some((Palette::dim(), 2.0))),
                obstacle_pose,
            )
            .layer(2),
        );
        specs.push(
            PartSpec::new(
                vec![Shape::circle(centre, axis, radius, 64, faint_gold, 3.0)],
                still(Pose::NOTHING),
            )
            .layer(1),
        );
        let trace: Vec<V3> = keys.iter().map(|q| arm.joint(q, ELBOW).0).collect();
        specs.push(
            PartSpec::new(
                vec![Shape::Path {
                    pts: trace,
                    color: gold,
                    width: 5.0,
                    closed: false,
                    bias: 0.0,
                }],
                still(Pose::NOTHING),
            )
            .layer(2),
        );
        // the seven arrows at the gripper: what each joint's share of the
        // spare joint's own motion does to the tip, laid out tip to tail
        let j0 = arm.jacobian(&q0);
        let null = model::still_speeds(&j0);
        let tip = arm.tip(&q0).p;
        let pushes: Vec<V3> = (0..N)
            .map(|i| {
                let col = model::column(&j0, i);
                let tip_velocity = add(model::slide_of(col), cross(model::spin_of(col), tip));
                scale(tip_velocity, null[i])
            })
            .collect();
        let chain = stage.add_value_tracker(0.0);
        let mut offset = [0.0; 3];
        for push in &pushes {
            let o = scale(offset, ARROW_PER_SPEED);
            specs.push(
                PartSpec::new(
                    vec![Shape::Arrow {
                        from: tip,
                        to: add(tip, scale(*push, ARROW_PER_SPEED)),
                        color: orange,
                        width: 5.0,
                        head: 0.2,
                    }],
                    pose_fn(move |a: &Arena| Pose {
                        r: model::I3,
                        p: scale(o, value(a, chain)),
                    }),
                )
                .layer(3),
            );
            offset = add(offset, *push);
        }
        specs.push(
            PartSpec::new(
                vec![Shape::Dashed {
                    a: centre,
                    b: elbow0,
                    color: gold,
                    width: 3.0,
                    dashes: 6,
                }],
                still(Pose::NOTHING),
            )
            .layer(2),
        );
        let mut pins = Vec::new();
        let radius_label = kit.text(stage, &format!("{} {CM}", cm_whole(radius)), 32.0, gold, v(0.0, 0.0))?;
        let mid = scale(add(centre, elbow0), 0.5);
        pins.push(Pin {
            mob: radius_label,
            at: point_fn(move |_: &Arena| mid),
            shift: v(0.7, 0.0),
        });
        let world = World::build(stage, view, 1.4, 0.25, specs, pins)?;
        let hidden: Vec<usize> = ArmRig::GOLD_COLLARS
            .chain([part::OBSTACLE, part::CIRCLE, part::TRACE, part::RADIUS])
            .chain(part::ARROWS)
            .collect();
        for i in hidden {
            stage.set_tracker_value(world.parts[i].shown, 0.0)?;
        }
        world.add_to_scene(stage)?;
        let shown = |i: usize| world.parts[i].shown;

        // 12_1: seven joints for six directions leaves one spare
        let line = say(&kit, stage, BEATS[0])?;
        world.show(stage, 0.8)?;
        let (joints, rank, spare) = tally()?;
        let sum = kit.tex(
            stage,
            &format!("{joints} - {rank} = {spare}"),
            72.0,
            Palette::ink(),
            &[(spare.as_str(), gold)],
            v(4.3, 2.4),
        )?;
        line.wait_for(stage, "Seven joints")?;
        fade_in_all(stage, &[sum], 0.7)?;
        line.wait_end(stage)?;

        // 12_2: try it on yourself; the card; the person at the desk
        let line = say(&kit, stage, BEATS[1])?;
        let card = QuizCard::build(&kit, stage, ASK, QuizLook::at(v(CARD_AT[0], CARD_AT[1]), CARD_SCALE))?;
        play!(stage; tween(world.shown, 0.0, 0.7)?);
        leave(stage, &[sum], &[], 0.3)?;
        line.wait_for(stage, "Lay your palm flat on a desk")?;
        person_world.show(stage, 0.8)?;
        line.wait_for(stage, "keep your shoulder still")?;
        line.wait_last(stage)?;
        card.show(stage, 0.5)?;
        card.count(&kit, stage)?;

        // 12_3: it can; it swings on an arc while the hand stays put
        let line = say(&kit, stage, BEATS[2])?;
        play!(stage; tween(person_world.parts[person::CIRCLE].shown, 1.0, 0.5)?);
        line.wait_for(stage, "It swings on an arc")?;
        play!(stage; tween(swing, PERSON_SWING, 1.0)?);
        play!(stage; tween(swing, -PERSON_SWING, 2.0)?);
        play!(stage; tween(swing, 0.0, 1.0)?);
        line.wait_for(stage, "Counted the way a roboticist counts")?;
        fade_in_all(stage, &counts, 0.8)?;
        let counted: Vec<String> = COUNTED.iter().map(|(_, r)| r.len().to_string()).collect();
        let total = kit.tex(
            stage,
            &format!("{} = {}", counted.join(" + "), fact("G-A.joints")?),
            60.0,
            Palette::ink(),
            &[],
            v(4.3, 2.4),
        )?;
        line.wait_for(stage, "your arm has seven joints too")?;
        fade_in_all(stage, &[total], 0.6)?;
        line.wait_end(stage)?;

        // 12_4: the robot does the same
        let line = say(&kit, stage, BEATS[3])?;
        let mut away = vec![total];
        away.extend(&counts);
        leave(
            stage,
            &away,
            &[person_world.shown, person_world.parts[person::CIRCLE].shown],
            0.6,
        )?;
        world.show(stage, 0.8)?;
        let moved_label = kit.text_left(stage, GRIPPER_MOVED, 30.0, Palette::dim(), v(MOVED_AT[0], MOVED_AT[1]))?;
        let cm_label = kit.text(stage, CM, 30.0, Palette::dim(), v(MOVED_CM_X, MOVED_AT[1]))?;
        line.wait_for(stage, "Its gripper holds one pose")?;
        fade_in_all(stage, &[moved_label, cm_label], 0.4)?;
        {
            let (probe, arm_for) = (rig.clone(), arm.clone());
            let held = arm.tip(&q0).p;
            kit.live_value(
                stage,
                move |a: &Arena| 100.0 * norm(sub(arm_for.tip(&probe.q(a)).p, held)),
                1,
                false,
                40.0,
                Palette::ink(),
                v(MOVED_NUMBER_RIGHT, MOVED_AT[1]),
                RIGHT,
            )?;
        }
        play!(stage;
            tween(shown(part::TRACE), 1.0, 0.6)?,
            tween(shown(part::CIRCLE), 1.0, 0.6)?,
        );
        line.wait_for(stage, "while its elbow swings")?;
        play!(stage; tween(walk, 1.0, 2.4)?);
        play!(stage; tween(walk, 0.0, 3.4)?);
        line.wait_for(stage, "twenty-nine centimeters in radius")?;
        play!(stage;
            tween(walk, start_at, 1.6)?,
            tween(shown(part::RADIUS), 1.0, 0.6)?,
            fade_in(stage.arena_mut(), radius_label, ORIGIN, 1.0)?.rt(0.6),
        );
        line.wait_end(stage)?;

        // 12_5: what the spare joint buys
        let line = say(&kit, stage, BEATS[4])?;
        play!(stage;
            tween(shown(part::RADIUS), 0.0, 0.3)?,
            tween(shown(part::OBSTACLE), 1.0, 0.4)?,
        );
        leave(stage, &[radius_label], &[], 0.2)?;
        line.wait_for(stage, "The elbow can get out of the way")?;
        // the end of the arc farthest from where the obstacle comes to rest
        let far_end = {
            let d = |k: usize| norm(sub(arm.joint(&keys[k], ELBOW).0, elbow0));
            if d(0) > d(keys.len() - 1) { 0.0 } else { 1.0 }
        };
        play!(stage;
            tween(obstacle_tracker, 1.0, 3.0)?,
            tween(walk, far_end, 3.0)?,
        );
        line.wait_for(stage, "while the gripper keeps working")?;
        for i in part::ARROWS {
            play!(stage; tween(shown(i), 1.0, 0.12)?);
        }
        play!(stage; tween(chain, 1.0, 1.8)?);
        let nothing = lower_third(&kit, stage, NULL_SPACE_LABEL, NULL_SPACE_TEXT)?;
        fade_in_all(stage, &[nothing], 0.6)?;
        line.wait_end(stage)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{DEG, LIMITS_DEG};

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
            .find(|c| c.key == "12_the_spare_joint")
            .expect("chapter 12");
        let ids: Vec<&str> = chapter.beats.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(ids, BEATS);
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for beat in &chapter.beats {
            let words = beat.spoken().expect("every beat of chapter 12 is spoken");
            assert_eq!(script.text(&beat.id), Some(words.as_str()), "{}", beat.id);
        }
    }

    #[test]
    fn every_cue_phrase_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("12_1", "Seven joints"),
            ("12_1", "leaves one joint spare"),
            ("12_2", "Lay your palm flat on a desk"),
            ("12_2", "keep your shoulder still"),
            ("12_3", "It swings on an arc"),
            ("12_3", "Counted the way a roboticist counts"),
            ("12_3", "your arm has seven joints too"),
            ("12_4", "Its gripper holds one pose"),
            ("12_4", "while its elbow swings"),
            ("12_4", "twenty-nine centimeters in radius"),
            ("12_5", "The elbow can get out of the way"),
            ("12_5", "while the gripper keeps working"),
        ] {
            let text = script.text(id).unwrap_or_default();
            assert!(text.contains(phrase), "{id} does not say {phrase:?}");
        }
    }

    /// 12_1: seven joints, six directions, one spare, all from the facts,
    /// and the spare is what is left.
    #[test]
    fn the_tally_is_seven_minus_six_is_one() {
        let (joints, rank, spare) = tally().unwrap();
        assert_eq!((joints.as_str(), rank.as_str(), spare.as_str()), ("7", "6", "1"));
        let (j, r, s): (usize, usize, usize) = (joints.parse().unwrap(), rank.parse().unwrap(), spare.parse().unwrap());
        assert_eq!(j - r, s);
        let arm = Arm::iiwa14();
        assert_eq!(model::rank(&arm.jacobian(&model::degrees(poses::SWING))), r);
    }

    /// 12_2: six seconds, and "it can" is the computed answer: the person's
    /// elbow has a whole circle, and so has the robot's.
    #[test]
    fn the_quiz_answer_is_that_the_elbow_moves() {
        assert_eq!(ASK.seconds().unwrap(), 6);
        assert_eq!(ASK.choices[ASK.answer_index().unwrap()].label, IT_CAN);
    }

    /// 12_3: shoulder 3, elbow 1, wrist 3: seven, every joint counted once.
    #[test]
    fn counted_the_roboticists_way_a_person_has_seven_joints() {
        let counts: Vec<usize> = COUNTED.iter().map(|(_, r)| r.len()).collect();
        assert_eq!(counts, [3, 1, 3]);
        assert_eq!(counts.iter().sum::<usize>(), N);
        let mut seen = [0; N];
        for (_, r) in &COUNTED {
            for i in r.clone() {
                seen[i] += 1;
            }
        }
        assert_eq!(seen, [1; N]);
        // the elbow is the robot's elbow joint, the wrist its joints 5 to 7
        assert_eq!(COUNTED[1].1, ELBOW..ELBOW + 1);
        assert_eq!(COUNTED[2].1.start, WRIST - 1);
    }

    /// 12_3: the person's elbow swings on a circle while shoulder and hand
    /// stay put: both bone lengths hold at every angle of the swing.
    #[test]
    fn the_persons_elbow_swings_with_both_ends_fixed() {
        let (a, b) = (fact_f64("G-A.upper_arm").unwrap(), fact_f64("G-A.forearm").unwrap());
        let t = triangle(SHOULDER_AT, PALM_AT, a, b);
        assert!((norm(sub(t.elbow, SHOULDER_AT)) - a).abs() < 1e-12);
        assert!((norm(sub(t.elbow, PALM_AT)) - b).abs() < 1e-12);
        assert!(t.radius > 0.1, "a circle worth swinging on");
        assert!(t.elbow[2] < SHOULDER_AT[2], "the elbow hangs below the line");
        for k in -8..=8 {
            let phi = PERSON_SWING * f64::from(k) / 8.0;
            let rotated = add(SHOULDER_AT, apply(&model::turn(t.axis, phi), sub(t.elbow, SHOULDER_AT)));
            assert!(
                (norm(sub(rotated, SHOULDER_AT)) - a).abs() < 1e-12,
                "upper arm at {phi}"
            );
            assert!((norm(sub(rotated, PALM_AT)) - b).abs() < 1e-12, "forearm at {phi}");
            assert!(
                (norm(sub(rotated, t.centre)) - t.radius).abs() < 1e-12,
                "on the circle at {phi}"
            );
        }
        // the hand and the shoulder are on the axis the swing turns about
        let on_axis = |p: V3| norm(cross(t.axis, sub(p, SHOULDER_AT)));
        assert!(on_axis(PALM_AT) < 1e-12 && on_axis(SHOULDER_AT) < 1e-12);
        // the palm is flat on the desk and the swing stays above it
        assert!(PALM_AT[2].abs() < 1e-12);
        for k in -8..=8 {
            let phi = PERSON_SWING * f64::from(k) / 8.0;
            let e = add(SHOULDER_AT, apply(&model::turn(t.axis, phi), sub(t.elbow, SHOULDER_AT)));
            assert!(e[2] > 0.0, "the elbow stays above the desk at {phi}");
        }
    }

    /// 12_4: along the robot's walk the gripper holds its pose, the elbow
    /// stays on its circle, every joint is inside its limit, and the radius
    /// is the fact's twenty-nine centimeters.
    #[test]
    fn the_robots_gripper_holds_while_its_elbow_swings_on_a_circle() {
        let arm = Arm::iiwa14();
        let q0 = model::degrees(poses::SWING);
        let (keys, start_at) = swing_keys(&arm, &q0);
        assert!(keys.len() > 200, "{} poses", keys.len());
        assert!((0.0..1.0).contains(&start_at));
        let hold = arm.tip(&q0);
        let r = arm.elbow_radius(&q0);
        assert_eq!(cm_whole(r), "29");
        assert!((r - fact_f64("G-J.elbow.radius").unwrap()).abs() < 5e-5);
        let (s0, w0) = (arm.joint(&q0, SHOULDER).0, arm.joint(&q0, WRIST).0);
        let axis = unit(sub(w0, s0));
        let mut worst: f64 = 0.0;
        for q in &keys {
            worst = worst.max(norm(sub(arm.tip(q).p, hold.p)));
            assert!(arm.tip(q).gap(&hold) < 1e-6, "the gripper holds its pose");
            let (s, e, w) = (arm.joint(q, SHOULDER).0, arm.joint(q, ELBOW).0, arm.joint(q, WRIST).0);
            assert!(norm(sub(s, s0)) < 1e-9, "the shoulder stays");
            assert!(norm(sub(w, w0)) < 1e-6, "the wrist stays");
            let off = sub(e, s);
            let radial = norm(sub(off, scale(axis, dot(off, axis))));
            assert!(
                (radial - r).abs() < 1e-6,
                "the elbow is {radial} from the line, not {r}"
            );
            for (x, lim) in q.iter().zip(LIMITS_DEG) {
                assert!(x.abs() / DEG <= lim + 1e-9, "inside the limits");
            }
        }
        // the readout prints 0.0
        assert_eq!(format!("{:.1}", 100.0 * worst), "0.0");
        // the walk starts where the poses stand: the first sample of the key
        // table at `start_at` is the swing pose
        let f = start_at * (keys.len() - 1) as f64;
        let k = f.round() as usize;
        assert!(norm(sub(arm.tip(&keys[k]).p, hold.p)) < 1e-6);
        assert!(keys[k].iter().zip(q0).all(|(a, b)| (a - b).abs() < 1e-12));
        assert_eq!(fact("G-J.swing.null_twist").unwrap(), "0.0000");
    }

    /// 12_5: the seven arrows are each joint's tip velocity for the spare
    /// joint's own speeds; laid tip to tail they close, and so do the spins.
    #[test]
    fn the_seven_arrows_cancel_to_nothing() {
        let arm = Arm::iiwa14();
        let q0 = model::degrees(poses::SWING);
        let j = arm.jacobian(&q0);
        let null = model::still_speeds(&j);
        assert!(
            (null.iter().map(|x| x * x).sum::<f64>().sqrt() - 1.0).abs() < 1e-12,
            "unit speeds"
        );
        let tip = arm.tip(&q0).p;
        let mut sum = [0.0; 3];
        let mut spin = [0.0; 3];
        let mut biggest: f64 = 0.0;
        for (i, &speed) in null.iter().enumerate() {
            let col = model::column(&j, i);
            let v_i = scale(add(model::slide_of(col), cross(model::spin_of(col), tip)), speed);
            biggest = biggest.max(norm(v_i));
            sum = add(sum, v_i);
            spin = add(spin, scale(model::spin_of(col), speed));
        }
        assert!(norm(sum) < 1e-9, "the tip velocities add to {sum:?}");
        assert!(norm(spin) < 1e-9, "the spins add to {spin:?}");
        assert!(biggest > 0.05, "the arrows are not all nothing");
        // a joint speed that is not the spare's does move the tip
        let mut other = [0.0; N];
        other[0] = 1.0;
        assert!(model::twist_from(&j, &other).iter().any(|x| x.abs() > 0.1));
    }

    /// 12_5: the obstacle comes to rest where the elbow was, and the end of
    /// the arc the walk goes to is the one farther from it.
    #[test]
    fn the_elbow_gets_out_of_the_obstacles_way() {
        let arm = Arm::iiwa14();
        let q0 = model::degrees(poses::SWING);
        let (keys, _) = swing_keys(&arm, &q0);
        let elbow0 = arm.joint(&q0, ELBOW).0;
        let (first, last) = (arm.joint(&keys[0], ELBOW).0, arm.joint(keys.last().unwrap(), ELBOW).0);
        let far = norm(sub(first, elbow0)).max(norm(sub(last, elbow0)));
        assert!(
            far > 2.0 * OBSTACLE_HALF * 3.0_f64.sqrt() + 0.1,
            "the elbow ends clear of the obstacle"
        );
        assert!(OBSTACLE_FROM > far, "the obstacle starts outside everything");
    }

    #[test]
    fn the_quiz_rows_fit_and_the_tally_fits() {
        run(|stage| {
            let kit = Kit::new()?;
            let card = QuizCard::build(&kit, stage, ASK, QuizLook::at(v(CARD_AT[0], CARD_AT[1]), CARD_SCALE))?;
            let right = stage.get_right(card.back)[0];
            for &row in &card.rows {
                assert!(stage.get_right(row)[0] < right - 0.15);
            }
            assert!(right < 7.1);
            let (j, r, s) = tally()?;
            let sum = kit.tex(
                stage,
                &format!("{j} - {r} = {s}"),
                72.0,
                Palette::ink(),
                &[],
                v(4.3, 2.4),
            )?;
            assert!(stage.get_right(sum)[0] < 7.0, "the tally fits");
            Ok(())
        });
    }

    /// 12_4's readout reads left to right without touching itself: the
    /// label, the live number (right-aligned, at its widest "0.0"), "cm".
    #[test]
    fn the_gripper_moved_readout_does_not_overlap_itself() {
        run(|stage| {
            let kit = Kit::new()?;
            let label = kit.text_left(stage, GRIPPER_MOVED, 30.0, Palette::dim(), v(MOVED_AT[0], MOVED_AT[1]))?;
            let number = kit.text(stage, "0.0", 40.0, Palette::ink(), v(0.0, 0.0))?;
            let cm = kit.text(stage, CM, 30.0, Palette::dim(), v(MOVED_CM_X, MOVED_AT[1]))?;
            let number_left = MOVED_NUMBER_RIGHT - stage.get_width(number);
            assert!(
                stage.get_right(label)[0] + 0.1 < number_left,
                "the label ends at {}, the number starts at {number_left}",
                stage.get_right(label)[0]
            );
            assert!(MOVED_NUMBER_RIGHT + 0.1 < stage.get_left(cm)[0]);
            assert!(stage.get_right(cm)[0] < 7.11 - 0.15);
            Ok(())
        });
    }
}
