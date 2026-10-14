//! Chapter 15, everywhere: the same moves run far beyond the arm: drones,
//! maps, a physics engine's step by the exponential, a humanoid's floating
//! body (15_1); the opening's quiz again with one degree (15_2); the
//! leftover between the two orders is a turn, shrinking a hundredfold as the
//! turns shrink tenfold (15_3, 15_4); and the arm multiplies its motions and
//! adds on the flat piece (15_5).

use std::f64::consts::TAU;
use std::rc::Rc;

use explainer_kit::props::{lower_third, names_to_search, receipts_card};
use explainer_kit::{Kit, Palette, play, tween, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::draw::{
    ARM_SCALE, ArmRig, Ask, BOOK_HALF, Choice, PartSpec, QuizCard, QuizLook, Shape, View, World, along_a_screw, book,
    fact, fade_in_all, fade_in_lagged, flat, ground, leave, line_style, mix, perp_basis, pose_fn, say, still, table,
    tag, tween_linear, value,
};
use super::steps::{goal_frame, pose_frame};
use crate::facts::shown;
use crate::model::{self, Arm, DEG, M3, Pose, V3, X, Y, add, angle_axis, apply, det, mul, norm, poses};
use crate::palette;

pub struct Everywhere {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const DRONES: &str = "drones steer poses";
const MAPS: &str = "mapping robots estimate them";
const ADD_THE_SPIN: &str = "add the spin to the nine numbers (forward Euler)";
const THE_EXPONENTIAL: &str = "the exponential";
const VOLUME: &str = "volume";
const WIDTH: &str = "width";
const OUR_TEST: &str = "our test:";
const ENGINE_TAG: &str = "FrankenSim steps bodies by the exponential (fs-time, se3_exp_step)";
const PROGRAMMERS: &str = "for programmers";
const HUMANOID_LABEL: &str = "the humanoid:";
const HUMANOID_NAME: &str = "Unitree G1";
const VELOCITY: &str = "velocity";
const STORED: &str = "stored state";
const NUMBERS: &str = "numbers";
const STILL_DIFFER: &str = "do the two results still differ?";
const THE_SAME: &str = "the same";
const STILL_DIFFERENT: &str = "still different";
const TURNS_WORD: &str = "both turns:";
const LEFTOVER_WORD: &str = "leftover:";
const LEFTOVER_NOTE_LABEL: &str = "leftover ≈";
const LEFTOVER_NOTE: &str = "a turn by angle × angle (in radians) about the arrow at right angles to roll's and tilt's, \
which is −z: the Lie bracket [y, x] = −z (x is tilt's line, y roll's, z up)";
const TABLE_HEAD: &str = "turn → leftover";
const GRIPPER_POSE: &str = "the gripper's pose";
const START_POSE: &str = "the pose at zero";
const FIRST_TWIST: &str = "joint 1's twist, held for its angle";
const RECEIPTS: [&str; 3] = [
    "every number we worked out was computed by code",
    "and checked by a second program; quoted numbers are cited",
    "github.com/Dicklesworthstone/jeffs_math_explainer_videos",
];
const NAMES: [&str; 4] = ["Lie group", "SE(3)", "screw theory", "product of exponentials"];

/// The beats of the chapter, in order (storyboard ids).
const BEATS: [&str; 5] = ["15_1", "15_2", "15_3", "15_4", "15_5"];

/// 15_2's quiz (`G-Q.15_2.answer` is `differ`).
pub const ASK: Ask = Ask {
    line: BEATS[1],
    question: STILL_DIFFER,
    choices: &[
        Choice {
            key: "same",
            label: THE_SAME,
        },
        Choice {
            key: "differ",
            label: STILL_DIFFERENT,
        },
    ],
};

/// The quiz card: its right edge (4.5 + 2.38) is inside the margin, and its
/// left edge (2.12) is clear of the books' world (`BOOKS_VIEW`).
const CARD_AT: [f64; 2] = [4.5, -0.2];
const CARD_SCALE: f64 = 0.88;
/// The opening's books' world: its scale and where its origin stands, so
/// that the right book, its axes and the table's far corner end before the
/// card begins and the table's near corner before 15_4's table of numbers.
const BOOKS_VIEW: (f64, [f64; 2]) = (0.9, [-0.6, -0.4]);
/// How far each book's two gold axes run out of its centre (world units).
const BOOK_AXIS: f64 = 1.4;
/// 15_3's readouts: their words' left edge, the right edge of their live
/// numbers, and their two heights: above the books, where no axis runs.
const READOUT_WORDS_X: f64 = -3.9;
const READOUT_NUMBER_RIGHT: f64 = -0.7;
const READOUT_Y: [f64; 2] = [3.5, 2.95];
/// 15_4's picture of the Lie bracket, above the card.
const BRACKET_AT: [f64; 2] = [2.8, 2.75];
/// The leftover's own axis is the last of `table_books`' parts: out of the
/// quiz's picture (it would say "something stands between them") and in at
/// 15_3.
const LEFTOVER_AXIS: usize = 5;
// the wedge ends before the readouts' words begin, and the bracket's picture
// (its line 1 either side, its third arrow 0.9 up) is inside the frame
const _: () = assert!(WEDGE_AT[0] + WEDGE_R + 0.3 < READOUT_WORDS_X);
const _: () = assert!(BRACKET_AT[1] + 0.9 < 3.85 && BRACKET_AT[0] + 1.0 < 6.96);
/// The two spinning books: where each is, and how big (picture units).
const SPIN_AT: [[f64; 2]; 2] = [[-3.6, -0.55], [0.2, -0.55]];
const SPIN_FIT: f64 = 0.62;
/// Seconds of the physics test, shown at once the picture can follow.
const SPIN_SECONDS: f64 = 10.0;
/// The two books of the opening, in the table's world.
const TABLE_BOOKS: [[f64; 2]; 2] = [[-1.9, 0.0], [1.9, 0.0]];
const TABLE_LO: [f64; 2] = [-3.2, -1.8];
const TABLE_HI: [f64; 2] = [3.2, 1.8];
const LIFT: f64 = 1.1;
/// Where the leftover's wedge is drawn, looking down its axis, and its size.
const WEDGE_AT: [f64; 2] = [-5.3, 2.5];
const WEDGE_R: f64 = 1.05;
/// The largest turn of the sweep, degrees, and the smallest.
const BIG_TURN: f64 = 90.0;
const SMALL_TURN: f64 = 1.0;

/// The turn size, in degrees, at `u` of the way through the sweep (0 to 1,
/// logarithmic from `BIG_TURN` down to `SMALL_TURN`).
pub fn turn_at(u: f64) -> f64 {
    BIG_TURN * (SMALL_TURN / BIG_TURN).powf(u.clamp(0.0, 1.0))
}

/// How far through the sweep a turn of `deg` degrees is.
pub fn sweep_of(deg: f64) -> f64 {
    (BIG_TURN / deg).ln() / (BIG_TURN / SMALL_TURN).ln()
}

/// The leftover's angle (degrees) and its axis, for turns of `deg` degrees.
pub fn leftover_of(deg: f64) -> (f64, V3) {
    let (a, axis) = angle_axis(&model::leftover(deg * DEG));
    (a / DEG, axis)
}

/// A leftover as the picture and the table print it: whole degrees from a
/// hundred, two places from one, four below.
pub fn leftover_label(deg: f64) -> String {
    if deg >= 100.0 {
        format!("{}°", deg.round() as i64)
    } else if deg >= 1.0 {
        format!("{deg:.2}°")
    } else {
        format!("{deg:.4}°")
    }
}

/// The body spun by the added spin and by the exponential, `steps` steps of
/// `dt`: the matrices after each step (index 0 is the start), as the facts'
/// `drift` computes their last.
pub fn spin_tables(steps: usize, dt: f64) -> (Vec<M3>, Vec<M3>) {
    let axis = model::unit([1.0, 1.0, 1.0]);
    let spin = model::hat(axis);
    let one = model::turn(axis, dt);
    let (mut added, mut stepped) = (model::I3, model::I3);
    let (mut a, mut s) = (vec![added], vec![stepped]);
    for _ in 0..steps {
        let d = mul(&spin, &added);
        added = std::array::from_fn(|i| std::array::from_fn(|k| added[i][k] + dt * d[i][k]));
        stepped = mul(&one, &stepped);
        a.push(added);
        s.push(stepped);
    }
    (a, s)
}

/// The humanoid as a stick figure: its bones, as pairs of points (metres),
/// and one dot per joint, grouped as the robot has them: each leg's six,
/// the waist's three, each arm's seven.
pub struct Figure {
    pub bones: Vec<(V3, V3)>,
    pub joints: Vec<V3>,
    pub pelvis: V3,
}

pub fn figure() -> Figure {
    let pelvis = [0.0, 0.0, 0.78];
    let mut bones = Vec::new();
    let mut joints = Vec::new();
    let around = |c: V3, k: usize, n: usize| -> V3 {
        // a cluster of `n` dots about `c`, so each joint of a group is seen
        let a = TAU * k as f64 / n as f64;
        add(c, [0.035 * a.cos(), 0.0, 0.035 * a.sin()])
    };
    for side in [-1.0, 1.0] {
        let hip = [side * 0.09, 0.0, 0.74];
        let knee = [side * 0.11, 0.02, 0.4];
        let ankle = [side * 0.11, 0.0, 0.06];
        bones.extend([
            (pelvis, hip),
            (hip, knee),
            (knee, ankle),
            (ankle, [side * 0.11, 0.12, 0.0]),
        ]);
        // three at the hip, one at the knee, two at the ankle
        joints.extend((0..3).map(|k| around(hip, k, 3)));
        joints.push(knee);
        joints.extend((0..2).map(|k| around(ankle, k, 2)));
    }
    let waist = [0.0, 0.0, 0.9];
    let chest = [0.0, 0.0, 1.08];
    let head = [0.0, 0.0, 1.22];
    bones.extend([(pelvis, waist), (waist, chest), (chest, head)]);
    joints.extend((0..3).map(|k| around(waist, k, 3)));
    for side in [-1.0, 1.0] {
        let shoulder = [side * 0.17, 0.0, 1.05];
        let elbow = [side * 0.22, 0.08, 0.82];
        let wrist = [side * 0.2, 0.2, 0.62];
        bones.extend([(chest, shoulder), (shoulder, elbow), (elbow, wrist)]);
        joints.extend((0..3).map(|k| around(shoulder, k, 3)));
        joints.push(elbow);
        joints.extend((0..3).map(|k| around(wrist, k, 3)));
    }
    Figure { bones, joints, pelvis }
}

/// A rotation whose third column is `axis`: it carries z onto the axis.
pub fn frame_with_axis(axis: V3) -> M3 {
    let (e1, e2) = perp_basis(axis);
    [
        [e1[0], e2[0], axis[0]],
        [e1[1], e2[1], axis[1]],
        [e1[2], e2[2], axis[2]],
    ]
}

impl SceneConstruct for Everywhere {
    fn name(&self) -> &str {
        "15_everywhere"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (gold, blue) = (palette::screw(), palette::turn());
        let steel = palette::steel();

        // ---- the two spinning books and their tables of matrices
        let (steps, dt) = shown::DRIFT;
        let (added, stepped) = spin_tables(steps, dt);
        let (added, stepped) = (Rc::new(added), Rc::new(stepped));
        let clock = stage.add_value_tracker(0.0);
        let at_step = move |a: &Arena| -> usize { ((value(a, clock) / dt).round().max(0.0) as usize).min(steps) };
        let half = [
            BOOK_HALF[0] * SPIN_FIT,
            BOOK_HALF[1] * SPIN_FIT,
            BOOK_HALF[2] * SPIN_FIT,
        ];
        let mut spin_specs = Vec::new();
        for (k, table_of) in [Rc::clone(&added), Rc::clone(&stepped)].into_iter().enumerate() {
            let centre = [SPIN_AT[k][0], SPIN_AT[k][1], 0.0];
            spin_specs.push(PartSpec::new(
                book(half),
                pose_fn(move |a: &Arena| Pose {
                    r: table_of[at_step(a)],
                    p: centre,
                }),
            ));
        }
        let spin_world = World::build(stage, View::new(1.0, v(0.0, 0.0)), 5.0, 0.0, spin_specs, Vec::new())?;
        spin_world.add_to_scene(stage)?;

        // ---- the humanoid
        let fig = figure();
        let mut g1_specs = Vec::new();
        let edge = mix(steel, ground(), 0.7);
        let mut bone_shapes = Vec::new();
        for (a, b) in &fig.bones {
            bone_shapes.push(Shape::line(*a, *b, edge, 12.0));
            bone_shapes.push(Shape::line(*a, *b, steel, 8.0));
        }
        g1_specs.push(PartSpec::new(bone_shapes, still(Pose::NOTHING)));
        g1_specs.push(
            PartSpec::new(
                fig.joints
                    .iter()
                    .map(|&p| Shape::Dot {
                        at: p,
                        color: gold,
                        radius: 0.055,
                    })
                    .collect(),
                still(Pose::NOTHING),
            )
            .layer(2),
        );
        let pelvis_pose = Pose {
            r: model::I3,
            p: fig.pelvis,
        };
        g1_specs.push(PartSpec::new(pose_frame(&pelvis_pose, 0.2, palette::goal()), still(Pose::NOTHING)).layer(3));
        // its feet stand above the lower third's top edge
        let g1_world = World::build(stage, View::new(3.0, v(4.3, -3.0)), 1.0, 0.1, g1_specs, Vec::new())?;
        g1_world.add_to_scene(stage)?;

        // 15_1: far beyond this arm
        let line = say(&kit, stage, BEATS[0])?;
        let drone = drone_icon(stage, v(-5.2, 2.45))?;
        let map = map_icon(stage, v(-1.9, 2.45))?;
        let drone_words = kit.text(stage, DRONES, 26.0, Palette::dim(), v(-5.2, 1.5))?;
        let map_words = kit.text(stage, MAPS, 26.0, Palette::dim(), v(-1.9, 1.5))?;
        line.wait_for(stage, "Drones steer poses")?;
        fade_in_all(stage, &[drone, drone_words], 0.6)?;
        line.wait_for(stage, "Mapping robots")?;
        fade_in_all(stage, &[map, map_words], 0.6)?;
        line.wait_for(stage, "A physics engine")?;
        let (add_label, exp_label) = (
            kit.text(
                stage,
                ADD_THE_SPIN,
                24.0,
                Palette::dim(),
                v(SPIN_AT[0][0], SPIN_AT[0][1] - 1.55),
            )?,
            kit.text(
                stage,
                THE_EXPONENTIAL,
                24.0,
                Palette::dim(),
                v(SPIN_AT[1][0], SPIN_AT[1][1] - 1.55),
            )?,
        );
        spin_world.show(stage, 0.7)?;
        fade_in_all(stage, &[add_label, exp_label], 0.5)?;
        // the readouts are read off the bodies as they spin; the test's note
        // is under them
        let across = model::unit([1.0, -1.0, 0.0]);
        let mut readouts = Vec::new();
        for (k, table_of) in [Rc::clone(&added), Rc::clone(&stepped)].into_iter().enumerate() {
            let (vol, wid) = (
                {
                    let t = Rc::clone(&table_of);
                    kit.live_value(
                        stage,
                        move |a: &Arena| det(&t[at_step(a)]),
                        3,
                        false,
                        30.0,
                        Palette::ink(),
                        v(SPIN_AT[k][0] + 0.9, SPIN_AT[k][1] - 2.05),
                        RIGHT,
                    )?
                },
                {
                    let t = Rc::clone(&table_of);
                    kit.live_value(
                        stage,
                        move |a: &Arena| norm(apply(&t[at_step(a)], across)),
                        3,
                        false,
                        30.0,
                        Palette::ink(),
                        v(SPIN_AT[k][0] + 0.9, SPIN_AT[k][1] - 2.55),
                        RIGHT,
                    )?
                },
            );
            readouts.extend([vol, wid]);
        }
        let mut words = Vec::new();
        for at in SPIN_AT {
            words.push(kit.text(stage, VOLUME, 26.0, Palette::dim(), v(at[0] - 0.45, at[1] - 2.05))?);
            words.push(kit.text(stage, WIDTH, 26.0, Palette::dim(), v(at[0] - 0.45, at[1] - 2.55))?);
        }
        let our_test = format!(
            "{OUR_TEST} {} steps of {} s at one radian a second. {ENGINE_TAG}",
            fact("G-C.drift.steps")?,
            fact("G-C.drift.dt")?
        );
        let engine = kit.text(stage, &our_test, 20.0, Palette::dim(), v(0.0, -3.55))?;
        fade_in_all(stage, &[words.clone(), vec![engine]].concat(), 0.5)?;
        play!(stage; tween_linear(clock, SPIN_SECONDS, 6.0)?);
        line.wait_for(stage, "And a humanoid's whole body floats")?;
        let mut away = vec![drone, map, drone_words, map_words, add_label, exp_label, engine];
        away.extend(&words);
        away.extend(&readouts);
        leave(stage, &away, &[spin_world.shown], 0.6)?;
        g1_world.show(stage, 0.8)?;
        let name = lower_third(
            &kit,
            stage,
            HUMANOID_LABEL,
            &format!("{HUMANOID_NAME}, {} joints", fact("G-C.g1.joints")?),
        )?;
        let six = fact("G-S.numbers.twist")?;
        let four = fact("G-S.numbers.quaternion")?;
        let three = model::X.len();
        let tally_v = format!(
            "{VELOCITY}: {six} + {} = {} {NUMBERS}",
            fact("G-C.g1.joints")?,
            fact("G-C.g1.velocity")?
        );
        let tally_s = format!(
            "{STORED}: {three} + {four} + {} = {} {NUMBERS}",
            fact("G-C.g1.joints")?,
            fact("G-C.g1.stored")?
        );
        let programmers = tag(&kit, stage, PROGRAMMERS, v(-4.0, 1.9), 22.0)?;
        let t1 = kit.text_left(stage, &tally_v, 28.0, Palette::ink(), v(-6.4, 1.2))?;
        let t2 = kit.text_left(stage, &tally_s, 28.0, Palette::ink(), v(-6.4, 0.6))?;
        fade_in_all(stage, &[name, programmers, t1, t2], 0.7)?;
        let names = names_to_search(&kit, stage, &NAMES)?;
        line.wait_end(stage)?;
        // the names to search take the humanoid's place at the foot
        leave(stage, &[name], &[], 0.3)?;
        fade_in_all(stage, &[names], 0.5)?;
        kit.music_breath(stage, 1.0)?;

        // 15_2: the opening's quiz, with one degree
        let line = say(&kit, stage, BEATS[1])?;
        leave(stage, &[programmers, t1, t2, names], &[g1_world.shown], 0.6)?;
        let u = stage.add_value_tracker(sweep_of(SMALL_TURN));
        let books = table_books(u, blue, gold);
        let books_world = World::build(
            stage,
            View::new(BOOKS_VIEW.0, v(BOOKS_VIEW.1[0], BOOKS_VIEW.1[1])),
            5.0,
            0.15,
            books,
            Vec::new(),
        )?;
        stage.set_tracker_value(books_world.parts[LEFTOVER_AXIS].shown, 0.0)?;
        books_world.add_to_scene(stage)?;
        let card = QuizCard::build(&kit, stage, ASK, QuizLook::at(v(CARD_AT[0], CARD_AT[1]), CARD_SCALE))?;
        books_world.show(stage, 0.8)?;
        line.wait_last(stage)?;
        card.show(stage, 0.5)?;
        card.count(&kit, stage)?;

        // 15_3: they do, by less than two hundredths of a degree
        let line = say(&kit, stage, BEATS[2])?;
        let rim = wedge(stage, u, blue)?;
        let (turns_label, leftover_label_m) = (
            kit.text_left(
                stage,
                TURNS_WORD,
                26.0,
                Palette::dim(),
                v(READOUT_WORDS_X, READOUT_Y[0]),
            )?,
            kit.text_left(
                stage,
                LEFTOVER_WORD,
                26.0,
                Palette::dim(),
                v(READOUT_WORDS_X, READOUT_Y[1]),
            )?,
        );
        fade_in_all(stage, &[turns_label, leftover_label_m], 0.4)?;
        let turned = kit.live_value(
            stage,
            move |a: &Arena| turn_at(value(a, u)),
            0,
            false,
            30.0,
            Palette::ink(),
            v(READOUT_NUMBER_RIGHT, READOUT_Y[0]),
            RIGHT,
        )?;
        let left_over = kit.live_value(
            stage,
            move |a: &Arena| leftover_of(turn_at(value(a, u))).0,
            4,
            false,
            30.0,
            blue,
            v(READOUT_NUMBER_RIGHT, READOUT_Y[1]),
            RIGHT,
        )?;
        fade_in_all(stage, &[rim], 0.5)?;
        line.wait_for(stage, "They do")?;
        play!(stage; tween(u, 0.0, 1.0)?);
        line.wait_for(stage, "And that leftover is itself a turn")?;
        // its axis stands between the books, drawn only now that it is named
        play!(stage; tween(books_world.parts[LEFTOVER_AXIS].shown, 1.0, 0.5)?);
        play!(stage; tween(u, sweep_of(10.0), 2.4)?);
        play!(stage; tween(u, 1.0, 2.0)?);
        line.wait_end(stage)?;

        // 15_4: the leftover shrinks a hundredfold; the Lie bracket
        let line = say(&kit, stage, BEATS[3])?;
        let mut rows = Vec::new();
        for (k, small) in shown::SMALL_TURNS.iter().rev().enumerate() {
            let (angle, _) = leftover_of(*small);
            rows.push(kit.text_left(
                stage,
                &format!("{}° → {}", *small as i64, leftover_label(angle)),
                30.0,
                Palette::ink(),
                v(-6.7, -0.45 - 0.5 * k as f64),
            )?);
        }
        let head = kit.text_left(stage, TABLE_HEAD, 24.0, Palette::dim(), v(-6.7, -0.05))?;
        line.wait_for(stage, "make both ten times smaller")?;
        fade_in_lagged(stage, &[vec![head], rows.clone()].concat(), 1.2, 0.5)?;
        let bracket = bracket_badge(stage, v(BRACKET_AT[0], BRACKET_AT[1]))?;
        line.wait_for(stage, "called the Lie bracket")?;
        fade_in_all(stage, &bracket, 0.7)?;
        let note = lower_third(&kit, stage, LEFTOVER_NOTE_LABEL, LEFTOVER_NOTE)?;
        line.wait_for(stage, "It measures how much order matters")?;
        fade_in_all(stage, &[note], 0.6)?;
        line.wait_end(stage)?;

        // 15_5: the arm multiplies, and adds on the flat piece
        let line = say(&kit, stage, BEATS[4])?;
        let mut gone = vec![rim, turns_label, leftover_label_m, turned, left_over, head, note];
        gone.extend(&rows);
        gone.extend(&bracket);
        gone.extend(card.mobs());
        leave(stage, &gone, &[books_world.shown], 0.6)?;
        let arm = Arm::iiwa14();
        let q0 = model::degrees(poses::START);
        let start = arm.tip(&q0);
        let goal = model::goal(&arm);
        let screw = model::screw_of(&goal.then(&start.undo()));
        let keys = along_a_screw(&arm, &q0, &start, &screw, 24);
        let (rig, walk) = ArmRig::along(stage, arm.clone(), keys);
        let mut specs = rig.parts(steel);
        specs.push(PartSpec::new(table([-0.9, -0.7], [1.0, 0.9], 0.0, 0.2), still(Pose::NOTHING)).layer(-2));
        specs.push(PartSpec::new(goal_frame(&goal, 0.17), still(Pose::NOTHING)).layer(2));
        let arm_world = World::build(stage, View::new(ARM_SCALE, v(-3.6, -2.5)), 1.4, 0.25, specs, Vec::new())?;
        for i in ArmRig::GOLD_COLLARS {
            stage.set_tracker_value(arm_world.parts[i].shown, 0.0)?;
        }
        arm_world.add_to_scene(stage)?;
        // the opening's two books, above it: turned a quarter turn each way
        stage.set_tracker_value(u, 0.0)?;
        let books_again = table_books(u, blue, gold);
        let books_top = World::build(stage, View::new(0.55, v(3.6, 1.5)), 5.0, 0.15, books_again, Vec::new())?;
        books_top.add_to_scene(stage)?;
        arm_world.show(stage, 0.8)?;
        books_top.show(stage, 0.8)?;
        play!(stage; tween_linear(walk, 1.0, 3.2)?);
        line.wait_for(stage, "So the arm multiplies its motions")?;
        let pieces = equation_pieces(&kit, stage)?;
        fade_in_lagged(stage, &pieces, 1.6, 0.5)?;
        line.wait_end(stage)?;

        // the receipts: where every number came from
        let receipts = receipts_card(&kit, stage, &RECEIPTS)?;
        leave(stage, &pieces, &[arm_world.shown, books_top.shown], 0.6)?;
        fade_in_all(stage, &[receipts], 0.8)?;
        kit.music_breath(stage, 1.5)?;
        kit.finish(stage)
    }
}

/// A quadrotor: a body, four arms and four rotors (drawn flat).
fn drone_icon(stage: &mut Stage<'_>, at: Vec3) -> fmn::Result<Mob> {
    let ink = Palette::ink();
    let mut parts = Vec::new();
    for (sx, sy) in [(-1.0, 1.0), (1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)] {
        let tip = v(at[0] + 0.7 * sx, at[1] + 0.45 * sy);
        parts.push(
            stage
                .arena_mut()
                .add(polyline(vec![at, tip]).style(line_style(ink, 4.0)).build()),
        );
        parts.push(
            stage.arena_mut().add(
                Circle::new()
                    .radius(0.22)
                    .arc_center(tip)
                    .style(line_style(ink, 3.0))
                    .build(),
            ),
        );
    }
    parts.push(
        stage
            .arena_mut()
            .add(Circle::new().radius(0.16).arc_center(at).style(flat(ink, 1.0)).build()),
    );
    super::draw::group(stage, &parts)
}

/// A map with three pose markers: a frame, and a small triangle each.
fn map_icon(stage: &mut Stage<'_>, at: Vec3) -> fmn::Result<Mob> {
    let ink = Palette::ink();
    let mut parts = Vec::new();
    let frame = Rectangle::new()
        .width(1.8)
        .height(1.2)
        .corner_radius(0.08)
        .style(line_style(ink, 3.0))
        .build()?
        .moved_to(at);
    parts.push(stage.arena_mut().add(frame));
    for (dx, dy, turn) in [(-0.55_f64, -0.25_f64, 0.4_f64), (0.0, 0.2, 1.9), (0.55, -0.15, -0.8)] {
        let c = v(at[0] + dx, at[1] + dy);
        let tri: Vec<Vec3> = [0.0, 2.4, -2.4]
            .iter()
            .map(|a| v(c[0] + 0.14 * (turn + a).cos(), c[1] + 0.14 * (turn + a).sin()))
            .collect();
        parts.push(
            stage
                .arena_mut()
                .add(Polygon::new(tri).style(flat(palette::goal(), 1.0)).build()),
        );
    }
    super::draw::group(stage, &parts)
}

/// The opening's table, books and gold lines, driven by the sweep tracker
/// `u`: the left book is tilted and then rolled by the turn at `u`, the right
/// one rolled and then tilted, and the leftover's axis stands between them.
fn table_books(u: Mob, axis_color: Srgb, gold: Srgb) -> Vec<PartSpec> {
    let mut specs = vec![PartSpec::new(table(TABLE_LO, TABLE_HI, 0.0, 0.8), still(Pose::NOTHING)).layer(-2)];
    for (k, centre) in TABLE_BOOKS.iter().enumerate() {
        let p = [centre[0], centre[1], LIFT];
        specs.push(PartSpec::new(
            book(BOOK_HALF),
            pose_fn(move |a: &Arena| {
                let theta = turn_at(value(a, u)) * DEG;
                Pose {
                    r: if k == 0 {
                        model::tilt_then_roll(theta)
                    } else {
                        model::roll_then_tilt(theta)
                    },
                    p,
                }
            }),
        ));
        let mut lines = Vec::new();
        for dir in [X, Y] {
            for sign in [-1.0, 1.0] {
                lines.push(Shape::line([0.0; 3], scale_v(dir, BOOK_AXIS * sign), gold, 3.0));
            }
        }
        specs.push(
            PartSpec::new(
                lines,
                still(Pose {
                    r: model::I3,
                    p: [centre[0], centre[1], 0.03],
                }),
            )
            .layer(-1),
        );
    }
    // the leftover's own axis, standing between the books: along the axis
    // of the leftover turn at this size
    specs.push(
        PartSpec::new(
            vec![Shape::line([0.0, 0.0, -1.3], [0.0, 0.0, 1.3], axis_color, 5.0)],
            pose_fn(move |a: &Arena| {
                let (_, axis) = leftover_of(turn_at(value(a, u)));
                Pose {
                    r: frame_with_axis(axis),
                    p: [0.0, 0.0, LIFT],
                }
            }),
        )
        .layer(3),
    );
    specs
}

fn scale_v(a: V3, k: f64) -> V3 {
    a.map(|x| x * k)
}

/// The leftover as a wedge seen looking down its axis: a sector of the
/// turn's angle (live from `u`), with a gold dot for the axis.
fn wedge(stage: &mut Stage<'_>, u: Mob, color: Srgb) -> fmn::Result<Mob> {
    let at = v(WEDGE_AT[0], WEDGE_AT[1]);
    let m = stage.always_redraw(move |s| {
        let phi = (leftover_of(turn_at(s.tracker_value(u).unwrap_or(0.0))).0 * DEG).max(1e-4);
        let n = 24;
        let mut pts = vec![at];
        for k in 0..=n {
            let a = phi * k as f64 / n as f64;
            pts.push(v(at[0] + WEDGE_R * a.cos(), at[1] + WEDGE_R * a.sin()));
        }
        let sector = Polygon::new(pts)
            .style(Style::default().fill(color, 0.45).stroke(color, 2.5, 1.0))
            .build();
        s.add(sector)
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// The badge's picture of the Lie bracket: two spin arrows laid on the
/// orange line and a third at right angles to both.
fn bracket_badge(stage: &mut Stage<'_>, at: Vec3) -> fmn::Result<Vec<Mob>> {
    let orange = palette::flat();
    let mut out = Vec::new();
    let line = polyline(vec![v(at[0] - 1.0, at[1]), v(at[0] + 1.0, at[1])])
        .style(line_style(orange, 4.0))
        .build();
    out.push(stage.arena_mut().add(line));
    for dx in [-0.45, 0.35] {
        let a = Arrow::new(v(at[0] + dx - 0.3, at[1] + 0.12), v(at[0] + dx + 0.3, at[1] + 0.12))
            .thickness(3.5)
            .color(orange)
            .build()?;
        out.push(stage.arena_mut().add(a));
    }
    let third = Arrow::new(v(at[0], at[1]), v(at[0], at[1] + 0.9))
        .thickness(4.0)
        .color(palette::turn())
        .build()?;
    out.push(stage.arena_mut().add(third));
    Ok(out)
}

/// The seven-factor line, in pieces with its plain labels under them.
fn equation_pieces(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let y = -0.3;
    let size = 40.0;
    let ink = Palette::ink();
    let mut out = Vec::new();
    for (src, x) in [
        (r"T(q) =", 1.2),
        (r"e^{[S_1]q_1}", 2.65),
        (r"\cdots", 4.0),
        (r"e^{[S_7]q_7}", 5.25),
        (r"M", 6.35),
    ] {
        out.push(kit.tex(stage, src, size, ink, &[], v(x, y))?);
    }
    for (words, x) in EQUATION_LABELS {
        out.push(kit.text(stage, words, 20.0, Palette::dim(), v(x, y - 0.7))?);
    }
    Ok(out)
}

/// The plain labels under the seven-factor line and where each is centred:
/// spaced so that the widest (the middle one) touches neither neighbour, and
/// the last ends inside the frame's margin.
const EQUATION_LABELS: [(&str, f64); 3] = [(GRIPPER_POSE, 1.2), (FIRST_TWIST, 3.65), (START_POSE, 6.1)];

#[cfg(test)]
mod tests {
    use super::super::draw::fact_f64;
    use super::*;
    use crate::facts::drift;
    use crate::model::Z;

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
        let chapter = board.iter().find(|c| c.key == "15_everywhere").expect("chapter 15");
        let ids: Vec<&str> = chapter.beats.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(ids, BEATS);
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for beat in &chapter.beats {
            let words = beat.spoken().expect("every beat of chapter 15 is spoken");
            assert_eq!(script.text(&beat.id), Some(words.as_str()), "{}", beat.id);
        }
    }

    #[test]
    fn every_cue_phrase_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("15_1", "Drones steer poses"),
            ("15_1", "Mapping robots"),
            ("15_1", "A physics engine"),
            ("15_1", "And a humanoid's whole body floats"),
            ("15_3", "They do"),
            ("15_3", "And that leftover is itself a turn"),
            ("15_4", "make both ten times smaller"),
            ("15_4", "called the Lie bracket"),
            ("15_4", "It measures how much order matters"),
            ("15_5", "So the arm multiplies its motions"),
            ("15_5", "does its adding on the flat piece"),
        ] {
            let text = script.text(id).unwrap_or_default();
            assert!(text.contains(phrase), "{id} does not say {phrase:?}");
        }
    }

    /// 15_1: the two spinning books are the facts' experiment: after the
    /// thousand steps the added spin has grown the volume to 1.105 and the
    /// width to 1.051, and the exponential's is exactly 1.
    #[test]
    fn the_physics_step_swells_the_added_body_and_keeps_the_exponential_rigid() {
        let (steps, dt) = shown::DRIFT;
        let (added, stepped) = spin_tables(steps, dt);
        assert_eq!((added.len(), stepped.len()), (steps + 1, steps + 1));
        let across = model::unit([1.0, -1.0, 0.0]);
        let (vol_a, vol_s, width_a) = drift();
        assert!((det(&added[steps]) - vol_a).abs() < 1e-12);
        assert!((det(&stepped[steps]) - vol_s).abs() < 1e-12);
        assert!((norm(apply(&added[steps], across)) - width_a).abs() < 1e-12);
        assert_eq!(format!("{:.3}", vol_a), "1.105");
        assert_eq!(format!("{:.3}", width_a), "1.051");
        assert_eq!(format!("{:.3}", vol_s), "1.000");
        assert_eq!(fact("G-C.drift.add").unwrap(), "1.1052");
        assert_eq!(fact("G-C.drift.steps").unwrap(), "1000");
        assert_eq!(fact("G-C.drift.dt").unwrap(), "0.01");
        // the swelling only grows; the exponential's turn never does
        assert!(added.windows(2).all(|w| det(&w[1]) >= det(&w[0]) - 1e-15));
        assert!(stepped.iter().all(|m| (det(m) - 1.0).abs() < 1e-9));
        // a test of an hour: one radian a second for ten seconds
        assert!((steps as f64 * dt - SPIN_SECONDS).abs() < 1e-9);
    }

    /// 15_1: twenty-nine joints, a velocity of 35 numbers and a stored
    /// state of 36, from the facts, with a figure that has them all.
    #[test]
    fn the_humanoid_has_twenty_nine_joints_and_the_tally_adds_up() {
        let f = figure();
        assert_eq!(f.joints.len(), 29);
        assert_eq!(f.joints.len().to_string(), fact("G-C.g1.joints").unwrap());
        let six: usize = fact("G-S.numbers.twist").unwrap().parse().unwrap();
        let four: usize = fact("G-S.numbers.quaternion").unwrap().parse().unwrap();
        assert_eq!(
            six + f.joints.len(),
            fact("G-C.g1.velocity").unwrap().parse::<usize>().unwrap()
        );
        assert_eq!(
            model::X.len() + four + f.joints.len(),
            fact("G-C.g1.stored").unwrap().parse::<usize>().unwrap()
        );
        assert_eq!(fact("G-C.g1.velocity").unwrap(), "35");
        assert_eq!(fact("G-C.g1.stored").unwrap(), "36");
        // every bone joins points of the figure, and the figure stands on the floor
        let lowest = f
            .bones
            .iter()
            .flat_map(|(a, b)| [a[2], b[2]])
            .fold(f64::INFINITY, f64::min);
        assert!(lowest.abs() < 1e-9);
    }

    /// 15_2 to 15_4: the leftover is a turn that is not nothing at one
    /// degree, under two hundredths of a degree, a hundredth part of itself
    /// from ten degrees to one, almost exactly the product of the angles in
    /// radians about −z, and the quarter turns' is a third of a turn.
    #[test]
    fn the_leftover_shrinks_a_hundredfold_and_stands_about_minus_z() {
        assert_eq!(ASK.seconds().unwrap(), 4);
        assert_eq!(ASK.choices[ASK.answer_index().unwrap()].label, STILL_DIFFERENT);
        let (big, _) = leftover_of(90.0);
        let (ten, axis10) = leftover_of(10.0);
        let (one, axis1) = leftover_of(1.0);
        assert!((big - 120.0).abs() < 1e-9);
        assert!(one > 0.0 && one < 0.02, "{one}");
        assert!((ten / one - 100.0).abs() < 1.0, "a hundredfold: {}", ten / one);
        for (deg, angle, key) in [(90.0, big, "90"), (10.0, ten, "10"), (1.0, one, "1")] {
            let fact_angle = fact_f64(&format!("G-B.leftover.{key}.angle")).unwrap();
            assert!((angle - fact_angle).abs() < 5e-5, "{deg}: {angle} against {fact_angle}");
        }
        // the angle times the angle, in radians, and the axis −z
        for (deg, angle, axis, tol_axis) in [(1.0, one, axis1, 1.0), (10.0, ten, axis10, 8.0)] {
            let product = (deg * DEG) * (deg * DEG) / DEG;
            assert!((angle / product - 1.0).abs() < 0.01, "{deg}: {angle} against {product}");
            let off = (-axis[2]).clamp(-1.0, 1.0).acos() / DEG;
            assert!(off < tol_axis, "{deg}°: the axis is {off}° from −z");
        }
        // the printed table and the live label
        assert_eq!(leftover_label(big), "120°");
        assert_eq!(leftover_label(ten), "1.74°");
        assert_eq!(leftover_label(one), "0.0175°");
        // the sweep runs from ninety down to one and passes ten
        assert!((turn_at(0.0) - 90.0).abs() < 1e-9 && (turn_at(1.0) - 1.0).abs() < 1e-9);
        assert!((turn_at(sweep_of(10.0)) - 10.0).abs() < 1e-9);
        assert!(shown::SMALL_TURNS == [1.0, 10.0, 90.0]);
    }

    /// The frame the axis part uses carries z onto the axis and is a turn.
    #[test]
    fn the_axis_frame_is_a_turn_that_carries_z_to_the_axis() {
        for axis in [
            model::unit([1.0, 2.0, -0.5]),
            [0.0, 0.0, -1.0],
            model::unit([-0.0087, 0.0087, -0.9999]),
        ] {
            let r = frame_with_axis(axis);
            assert!((det(&r) - 1.0).abs() < 1e-9);
            assert!(model::gap(&mul(&model::transpose(&r), &r), &model::I3) < 1e-9);
            assert!(norm(model::sub(apply(&r, Z), axis)) < 1e-9);
        }
    }

    /// The labels under the seven-factor line do not touch one another and
    /// the last ends inside the margin; neither do the line's own pieces.
    #[test]
    fn the_equation_and_its_labels_do_not_overlap() {
        run(|stage| {
            let kit = Kit::new()?;
            let pieces = equation_pieces(&kit, stage)?;
            let (math, labels) = pieces.split_at(pieces.len() - EQUATION_LABELS.len());
            let spans = |ms: &[Mob], stage: &Stage<'_>| -> Vec<(f64, f64)> {
                ms.iter()
                    .map(|&m| (stage.get_left(m)[0], stage.get_right(m)[0]))
                    .collect()
            };
            for group in [spans(math, stage), spans(labels, stage)] {
                for w in group.windows(2) {
                    assert!(w[0].1 + 0.1 < w[1].0, "{:?} runs into {:?}", w[0], w[1]);
                }
            }
            let last = labels.last().copied().expect("three labels");
            assert!(stage.get_right(last)[0] < 7.11 - 0.15);
            assert!(stage.get_left(math[0])[0] > -7.11 + 0.15);
            Ok(())
        });
    }

    /// 15_2 to 15_4: at the biggest turn of the sweep, the books, their
    /// axes and the leftover's axis end before the card begins, the table's
    /// near corner before the left-hand table of numbers; the card's right
    /// edge is inside the margin.
    #[test]
    fn the_books_world_clears_the_card_and_the_numbers() {
        use super::super::draw::SAFE;
        run(|stage| {
            let kit = Kit::new()?;
            let card = QuizCard::build(&kit, stage, ASK, QuizLook::at(v(CARD_AT[0], CARD_AT[1]), CARD_SCALE))?;
            let (card_left, card_right) = (stage.get_left(card.back)[0], stage.get_right(card.back)[0]);
            assert!(card_right < SAFE[0], "the card's right edge is {card_right:.3}");
            let view = View::new(BOOKS_VIEW.0, v(BOOKS_VIEW.1[0], BOOKS_VIEW.1[1]));
            let mut right_most = f64::MIN;
            let mut left_most = f64::MAX;
            for deg in [SMALL_TURN, 10.0, BIG_TURN] {
                let theta = deg * DEG;
                for (k, centre) in TABLE_BOOKS.iter().enumerate() {
                    let r = if k == 0 {
                        model::tilt_then_roll(theta)
                    } else {
                        model::roll_then_tilt(theta)
                    };
                    let mut pts: Vec<V3> = Vec::new();
                    for c in 0..8 {
                        let s = |bit: usize| if c >> bit & 1 == 1 { 1.0 } else { -1.0 };
                        let local = [s(0) * BOOK_HALF[0], s(1) * BOOK_HALF[1], s(2) * BOOK_HALF[2]];
                        pts.push(add([centre[0], centre[1], LIFT], apply(&r, local)));
                    }
                    // the book's two gold axes end at +-BOOK_AXIS
                    for dir in [X, Y] {
                        for sign in [-1.0, 1.0] {
                            pts.push([
                                centre[0] + dir[0] * BOOK_AXIS * sign,
                                centre[1] + dir[1] * BOOK_AXIS * sign,
                                0.03,
                            ]);
                        }
                    }
                    let (lo, hi) = view.box_of(pts);
                    right_most = right_most.max(hi[0]);
                    left_most = left_most.min(lo[0]);
                }
            }
            assert!(
                right_most + 0.05 < card_left,
                "the books' world reaches {right_most:.2}, the card begins at {card_left:.2}"
            );
            // the table of numbers (15_4) ends at -4.5 or so, at its widest
            let widest = kit.text_left(
                stage,
                &format!("{}° → {}", 90, leftover_label(120.0)),
                30.0,
                Palette::ink(),
                v(-6.7, -0.45),
            )?;
            let corner_x = view
                .box_of([[TABLE_LO[0], TABLE_LO[1], 0.0], [TABLE_LO[0], TABLE_HI[1], 0.0]])
                .0[0];
            assert!(
                stage.get_right(widest)[0] + 0.1 < corner_x,
                "the table of numbers ends at {:.2}, the table's corner is at {corner_x:.2}",
                stage.get_right(widest)[0]
            );
            assert!(left_most > -SAFE[0]);
            Ok(())
        });
    }

    /// 15_3's readouts: the words end before the number begins even at its
    /// widest ("120.0000"), inside the margin, right of the wedge, above the
    /// bracket's picture; the bracket's picture is above the card and inside
    /// the frame.
    #[test]
    fn the_readouts_and_the_bracket_stand_apart() {
        use super::super::draw::SAFE;
        run(|stage| {
            let kit = Kit::new()?;
            let card = QuizCard::build(&kit, stage, ASK, QuizLook::at(v(CARD_AT[0], CARD_AT[1]), CARD_SCALE))?;
            let words = [TURNS_WORD, LEFTOVER_WORD];
            let numbers = ["90", "120.0000"];
            for (k, (word, number)) in words.iter().zip(numbers).enumerate() {
                let w = kit.text_left(stage, word, 26.0, Palette::dim(), v(READOUT_WORDS_X, READOUT_Y[k]))?;
                let n = kit.text(stage, number, 30.0, Palette::ink(), v(0.0, READOUT_Y[k]))?;
                let n_left = READOUT_NUMBER_RIGHT - stage.get_width(n);
                assert!(
                    stage.get_right(w)[0] + 0.2 < n_left,
                    "{word:?} ends at {}, its number starts at {n_left}",
                    stage.get_right(w)[0]
                );
                assert!(stage.get_top(w)[1] < SAFE[1]);
            }
            // the badge: its line, two arrows and the third standing on it
            let (lo, hi) = (
                [BRACKET_AT[0] - 1.0, BRACKET_AT[1]],
                [BRACKET_AT[0] + 1.0, BRACKET_AT[1] + 0.9],
            );
            assert!(
                lo[1] > stage.get_top(card.back)[1] + 0.3,
                "the bracket is above the card"
            );
            assert!(hi[1] < SAFE[1] && hi[0] < SAFE[0] && lo[0] > READOUT_NUMBER_RIGHT);
            // the leftover's axis is the last part of the books' table
            let books = table_books(stage.add_value_tracker(0.0), palette::turn(), palette::screw());
            assert_eq!(books.len(), LEFTOVER_AXIS + 1);
            Ok(())
        });
    }

    /// 15_1: the humanoid's feet stand above the lower third that names it.
    #[test]
    fn the_humanoids_feet_clear_the_lower_third() {
        use super::super::draw::LOWER_THIRD_TOP;
        let fig = figure();
        let view = View::new(3.0, v(4.3, -3.0));
        let points = fig
            .bones
            .iter()
            .flat_map(|&(a, b)| [a, b])
            .chain(fig.joints.iter().copied());
        let (lo, _) = view.box_of(points);
        assert!(lo[1] > LOWER_THIRD_TOP + 0.1, "the feet reach down to {:.2}", lo[1]);
    }

    /// 15_4: the leftover's note, with the plain lead and the letters' key,
    /// is two lines at most and stands under the card; 15_1's two labels
    /// under the spinning books, the second now with "(forward Euler)", do
    /// not touch.
    #[test]
    fn the_leftover_note_and_the_spin_labels_fit() {
        run(|stage| {
            let kit = Kit::new()?;
            let card = QuizCard::build(&kit, stage, ASK, QuizLook::at(v(CARD_AT[0], CARD_AT[1]), CARD_SCALE))?;
            let note = lower_third(&kit, stage, LEFTOVER_NOTE_LABEL, LEFTOVER_NOTE)?;
            // a one-line band reaches up to -3.14, a two-line one to -2.78
            assert!(
                stage.get_top(note)[1] <= -2.78 + 1e-6,
                "the note is more than two lines: it reaches up to {}",
                stage.get_top(note)[1]
            );
            assert!(stage.get_top(note)[1] + 0.1 < stage.get_bottom(card.back)[1]);
            let add = kit.text(
                stage,
                ADD_THE_SPIN,
                24.0,
                Palette::dim(),
                v(SPIN_AT[0][0], SPIN_AT[0][1] - 1.55),
            )?;
            let exp = kit.text(
                stage,
                THE_EXPONENTIAL,
                24.0,
                Palette::dim(),
                v(SPIN_AT[1][0], SPIN_AT[1][1] - 1.55),
            )?;
            assert!(stage.get_left(add)[0] > -7.11 + 0.15);
            assert!(
                stage.get_right(add)[0] + 0.2 < stage.get_left(exp)[0],
                "the two labels run together: {} against {}",
                stage.get_right(add)[0],
                stage.get_left(exp)[0]
            );
            Ok(())
        });
    }

    #[test]
    fn the_cards_and_the_notes_fit_the_frame() {
        run(|stage| {
            let kit = Kit::new()?;
            let card = QuizCard::build(&kit, stage, ASK, QuizLook::at(v(CARD_AT[0], CARD_AT[1]), CARD_SCALE))?;
            let right = stage.get_right(card.back)[0];
            for &row in &card.rows {
                assert!(stage.get_right(row)[0] < right - 0.15);
            }
            assert!(right < 7.1);
            let engine = kit.text(
                stage,
                &format!("{OUR_TEST} 1000 steps of 0.01 s at one radian a second. {ENGINE_TAG}"),
                20.0,
                Palette::dim(),
                v(0.0, -3.55),
            )?;
            assert!(
                stage.get_left(engine)[0] > -7.1 && stage.get_right(engine)[0] < 7.1,
                "the test note fits"
            );
            let tally = kit.text_left(
                stage,
                &format!("{STORED}: 3 + 4 + 29 = 36 {NUMBERS}"),
                28.0,
                Palette::ink(),
                v(-6.4, 0.6),
            )?;
            assert!(stage.get_right(tally)[0] < -0.5, "the tally stays left of the humanoid");
            Ok(())
        });
    }
}
