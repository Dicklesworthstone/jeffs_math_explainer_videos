//! Chapter 10, velocities add: the robot's real question runs the other way
//! round, and the answer starts with speeds (10_1); every axis counts where
//! it is right now, so a joint turning alone is a twist, and twists on the
//! flat piece add (10_2); seven twists side by side are the Jacobian's
//! columns (10_3); which joint moves the tip fastest, asked (10_4) and
//! answered by how far the tip is from each axis (10_5).

use std::f64::consts::{PI, TAU};

use explainer_kit::props::lower_third;
use explainer_kit::{Kit, Palette, Timed, play, tween, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::draw::{
    ARM_SCALE, ArmRig, Ask, BOOK_HALF, Choice, PartSpec, Pin, QuizCard, QuizLook, Shape, View, World, book,
    circle_points, entry, fact, fade_in_all, fade_in_lagged, faded_line, leave, line_style, point_fn, say, still,
    table, tag, turned_about,
};
use crate::model::{self, Arm, ELBOW, Pose, SHOULDER, V3, add, cross, poses, scale, slide_of, spin_of, sub};
use crate::palette;

pub struct VelocitiesAdd {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const GOAL_WORDS: &str = "the pose I want";
const ANGLES_WORDS: &str = "which joint angles?";
const SPIN_ROWS: &str = "spin";
const SLIDE_ROWS: &str = "slide";
const JACOBIAN_EQ: &str = r"V = J(q)\,\dot q";
const PROGRAMMERS: &str = "for programmers";
const TWIST_SLIDE_NOTE: &str = "a twist's slide is the velocity of the point at the room's origin;";
/// The second line of the note, set in TeX so that its cross is the math
/// font's (the text face's × is drawn in a heavier fallback face).
const TIP_VELOCITY_TEX: &str = r"\text{the tip's velocity is slide } + \text{ spin } \times \text{ tip position}";
const JACOBIAN_LABEL: &str = "the Jacobian:";
const JACOBIAN_TEXT: &str = "the map from joint speeds to the gripper's twist, at the pose the arm is in";
const WHICH_FASTEST: &str = "which joint moves the tip fastest?";
/// The three joints the question names, as the arm labels them.
const ASKED_NAMES: [&str; 3] = ["1 · the base", "2 · the shoulder", "7 · the wrist"];
const ZERO_CM: &str = "0 cm";
const ONLY_SPINS: &str = "only spins it";

/// The beats of the chapter, in order (storyboard ids).
const BEATS: [&str; 5] = ["10_1", "10_2", "10_3", "10_4", "10_5"];

/// 10_4's quiz: the three joints the question names, with the one the
/// computed answer calls the fastest (`G-Q.10_4.answer`: its number).
pub const ASK: Ask = Ask {
    line: BEATS[3],
    question: WHICH_FASTEST,
    choices: &[
        Choice {
            key: "1",
            label: "joint 1",
        },
        Choice {
            key: "2",
            label: "joint 2",
        },
        Choice {
            key: "7",
            label: "joint 7",
        },
    ],
};

/// The joints of the question (0-based): the base, the shoulder, the wrist's
/// last.
const ASKED: [usize; 3] = [0, SHOULDER, 6];

/// Where the arm stands, the card sits, and the grid's columns run.
const ARM_AT: [f64; 2] = [-3.1, -2.55];
const CARD_AT: [f64; 2] = [4.45, -0.85];
const CARD_SCALE: f64 = 0.9;
const GRID_X0: f64 = 1.0;
const GRID_PITCH: f64 = 0.88;
const GRID_Y0: f64 = 2.35;
const GRID_ROW: f64 = 0.48;
const GRID_PT: f64 = 27.0;
/// Metres drawn per metre a second of tip speed, so the arrows read.
const ARROW_PER_SPEED: f64 = 0.5;
/// The goal marker's opacity once its first moment has passed.
const GOAL_DIMMED: f64 = 0.4;
const LEVER_WIDTH: f64 = 4.0;
/// A spin's curl: three quarters of a circle, then its head.
const CURL_SWEEP: f64 = 1.5 * PI;

/// The parts this chapter adds to the arm's own 23: the table is 23 and the
/// book 24.
mod part {
    pub const GOAL: usize = 25;
    /// Small curled arrows that turn at the shoulder and the elbow (10_1).
    pub const TURNING_SHOULDER: usize = 26;
    pub const TURNING_ELBOW: usize = 27;
    /// The twist at the tip of one joint turning alone (10_2, 10_5).
    pub const TWIST_SHOULDER: usize = 28;
    pub const TWIST_BASE: usize = 29;
    /// Both together, and the base's arrow laid tip to tail (10_2).
    pub const TWIST_SUM: usize = 30;
    pub const TWIST_TAIL: usize = 31;
    /// Joint seven's: a spin and no slide (10_5).
    pub const TWIST_WRIST: usize = 32;
    /// The tip's distance from the base's, the shoulder's and the wrist's
    /// axes (10_5), and the right triangle under the shoulder's.
    pub const LEVER_BASE: usize = 33;
    pub const LEVER_SHOULDER: usize = 34;
    pub const LEVER_WRIST: usize = 35;
    pub const TRIANGLE: usize = 36;
}

/// A spin as the pictures draw it: an arc about `axis` through `centre`,
/// `sweep` radians a right-handed turn, ending in an arrow head.
pub fn curl(centre: V3, axis: V3, radius: f64, sweep: f64, color: Srgb, width: f64) -> Vec<Shape> {
    let n = 24;
    let arc = circle_points(centre, axis, radius, 0.0, sweep, n);
    vec![
        Shape::Path {
            pts: arc[..n].to_vec(),
            color,
            width,
            closed: false,
            bias: 0.0,
        },
        Shape::Arrow {
            from: arc[n - 1],
            to: arc[n],
            color,
            width,
            head: 0.2,
        },
    ]
}

/// The tip's velocity when joint `i` alone turns at one radian a second:
/// the twist's slide plus its spin crossed with the tip's position (the
/// "for programmers" tag).
pub fn tip_velocity(j: &model::Jac, i: usize, tip: V3) -> V3 {
    let col = model::column(j, i);
    add(slide_of(col), cross(spin_of(col), tip))
}

/// The point of joint `i`'s axis nearest to `p`.
fn foot(point: V3, dir: V3, p: V3) -> V3 {
    add(point, scale(dir, model::dot(sub(p, point), dir)))
}

/// Whole centimeters (67, 42, 0), as the voice says them.
fn cm_whole(metres: f64) -> String {
    format!("{}", (metres * 100.0).round() as i64)
}

/// Centimeters to one place when not whole (52.6).
fn cm(metres: f64) -> String {
    let c = metres * 100.0;
    if (c - c.round()).abs() < 1e-6 {
        format!("{}", c.round() as i64)
    } else {
        format!("{c:.1}")
    }
}

/// The L shape's Jacobian from its fact: six rows of seven entries, printed
/// as the video prints a matrix entry (whole numbers plain, the rest at two
/// places, a true minus sign, never "−0").
fn grid_entries() -> fmn::Result<Vec<Vec<String>>> {
    fact("G-J.ell.J")?
        .split(';')
        .map(|row| {
            row.split(',')
                .map(|x| {
                    x.trim()
                        .parse::<f64>()
                        .map(entry)
                        .map_err(|_| super::draw::fail(format!("G-J.ell.J has {x:?}, not a number")))
                })
                .collect()
        })
        .collect()
}

fn arrow(from: V3, velocity: V3, color: Srgb, width: f64) -> Shape {
    Shape::Arrow {
        from,
        to: add(from, scale(velocity, ARROW_PER_SPEED)),
        color,
        width,
        head: 0.22,
    }
}

impl SceneConstruct for VelocitiesAdd {
    fn name(&self) -> &str {
        "10_velocities_add"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let arm = Arm::iiwa14();
        let rig = ArmRig::joints(stage, arm.clone(), poses::ELL);
        let q = model::degrees(poses::ELL);
        let jac = arm.jacobian(&q);
        let tip = arm.tip(&q).p;
        let goal = model::goal(&arm);
        let (orange, gold, pink) = (palette::flat(), palette::screw(), palette::goal());
        let view = View::new(ARM_SCALE, v(ARM_AT[0], ARM_AT[1]));

        // ---- the world: the arm in its L shape, the table, the book and the
        // goal; the turning curls; the twists at the tip; the levers
        let mut specs = rig.parts(palette::steel());
        specs.push(PartSpec::new(table([-0.9, -0.7], [1.0, 0.9], 0.0, 0.2), still(Pose::NOTHING)).layer(-2));
        let small = [BOOK_HALF[0] / 11.0, BOOK_HALF[1] / 11.0, BOOK_HALF[2] / 9.0];
        specs.push(PartSpec::new(
            book(small),
            still(Pose {
                r: model::I3,
                p: [model::GOAL_P[0], model::GOAL_P[1], small[2]],
            }),
        ));
        let mut frame = vec![Shape::Dot {
            at: goal.p,
            color: pink,
            radius: 0.13,
        }];
        for k in 0..3 {
            let c = [goal.r[0][k], goal.r[1][k], goal.r[2][k]];
            frame.push(Shape::Arrow {
                from: goal.p,
                to: add(goal.p, scale(c, 0.17)),
                color: pink,
                width: 5.0,
                head: 0.2,
            });
        }
        specs.push(PartSpec::new(frame, still(Pose::NOTHING)).layer(2));
        // the small curls that start to turn (10_1): one tracker turns both
        let turn = stage.add_value_tracker(0.0);
        for joint in [SHOULDER, ELBOW] {
            let (centre, axis) = arm.joint(&q, joint);
            specs.push(
                PartSpec::new(
                    curl([0.0; 3], axis, 0.11, CURL_SWEEP, orange, 4.5),
                    turned_about(centre, vec![(axis, turn, TAU)]),
                )
                .layer(2),
            );
        }
        // the twists at the tip of joints 2 and 1 alone, their sum, and the
        // base's arrow laid tip to tail on the shoulder's
        let v_sh = tip_velocity(&jac, SHOULDER, tip);
        let v_base = tip_velocity(&jac, 0, tip);
        let both = add(v_sh, v_base);
        let (axis_sh, axis_base, axis_wrist) =
            (arm.joint(&q, SHOULDER).1, arm.joint(&q, 0).1, arm.joint(&q, ASKED[2]).1);
        let with_curl = |axis: V3, velocity: Option<V3>, color: Srgb| {
            let mut shapes = curl(tip, axis, 0.13, CURL_SWEEP, color, 4.0);
            if let Some(vel) = velocity {
                shapes.push(arrow(tip, vel, color, 5.0));
            }
            shapes
        };
        specs.push(PartSpec::new(with_curl(axis_sh, Some(v_sh), orange), still(Pose::NOTHING)).layer(2));
        specs.push(PartSpec::new(with_curl(axis_base, Some(v_base), orange), still(Pose::NOTHING)).layer(2));
        specs.push(PartSpec::new(vec![arrow(tip, both, orange, 6.5)], still(Pose::NOTHING)).layer(3));
        let head_of_first = add(tip, scale(v_sh, ARROW_PER_SPEED));
        specs.push(
            PartSpec::new(
                vec![Shape::Dashed {
                    a: head_of_first,
                    b: add(head_of_first, scale(v_base, ARROW_PER_SPEED)),
                    color: orange,
                    width: 4.0,
                    dashes: 7,
                }],
                still(Pose::NOTHING),
            )
            .layer(2),
        );
        specs.push(PartSpec::new(with_curl(axis_wrist, None, orange), still(Pose::NOTHING)).layer(2));
        // the levers: from each axis, the shortest line to the tip
        let lever_to = |joint: usize| -> Shape {
            let (point, dir) = arm.joint(&q, joint);
            Shape::line(foot(point, dir, tip), tip, gold, LEVER_WIDTH)
        };
        specs.push(PartSpec::new(vec![lever_to(0)], still(Pose::NOTHING)).layer(3));
        specs.push(PartSpec::new(vec![lever_to(SHOULDER)], still(Pose::NOTHING)).layer(3));
        specs.push(
            PartSpec::new(
                vec![Shape::Dot {
                    at: tip,
                    color: gold,
                    radius: 0.09,
                }],
                still(Pose::NOTHING),
            )
            .layer(3),
        );
        // the triangle: the shoulder's height, across to the tip, up to it
        let shoulder_pt = arm.joint(&q, SHOULDER).0;
        let corner = [tip[0], tip[1], shoulder_pt[2]];
        specs.push(
            PartSpec::new(
                vec![
                    Shape::Dashed {
                        a: shoulder_pt,
                        b: corner,
                        color: Palette::dim(),
                        width: 3.0,
                        dashes: 6,
                    },
                    Shape::Dashed {
                        a: corner,
                        b: tip,
                        color: Palette::dim(),
                        width: 3.0,
                        dashes: 7,
                    },
                ],
                still(Pose::NOTHING),
            )
            .layer(2),
        );

        // ---- labels that follow world points
        let mut pins = Vec::new();
        let mut asked_labels = Vec::new();
        for ((&joint, words), shift) in ASKED
            .iter()
            .zip(ASKED_NAMES)
            .zip([v(-1.15, -0.1), v(-1.3, 0.35), v(1.25, 0.4)])
        {
            let m = kit.text(stage, words, 26.0, gold, v(0.0, 0.0))?;
            let (at, _) = rig.axis_now(joint);
            pins.push(Pin { mob: m, at, shift });
            asked_labels.push(m);
        }
        let goal_label = kit.text(stage, GOAL_WORDS, 30.0, pink, v(0.0, 0.0))?;
        let goal_at = goal.p;
        pins.push(Pin {
            mob: goal_label,
            at: point_fn(move |_: &Arena| goal_at),
            shift: v(1.35, 0.5),
        });
        let lever_mid = |joint: usize| {
            let (point, dir) = arm.joint(&q, joint);
            let f = foot(point, dir, tip);
            scale(add(f, tip), 0.5)
        };
        let mut lever_labels = Vec::new();
        for (joint, words, shift) in [
            (0, format!("{} cm", cm_whole(arm.lever(&q, 0))), v(0.0, 0.42)),
            (
                SHOULDER,
                format!("{} cm", cm_whole(arm.lever(&q, SHOULDER))),
                v(-0.75, -0.35),
            ),
        ] {
            let m = kit.text(stage, &words, 30.0, gold, v(0.0, 0.0))?;
            let at = lever_mid(joint);
            pins.push(Pin {
                mob: m,
                at: point_fn(move |_: &Arena| at),
                shift,
            });
            lever_labels.push(m);
        }
        let wrist_label = kit.text(stage, ZERO_CM, 30.0, gold, v(0.0, 0.0))?;
        pins.push(Pin {
            mob: wrist_label,
            at: point_fn(move |_: &Arena| tip),
            shift: v(0.0, 0.5),
        });
        lever_labels.push(wrist_label);
        let leg_across = kit.text(stage, &cm(tip[0]), 26.0, Palette::dim(), v(0.0, 0.0))?;
        let leg_up = kit.text(stage, &cm(tip[2] - shoulder_pt[2]), 26.0, Palette::dim(), v(0.0, 0.0))?;
        let across_mid = scale(add(shoulder_pt, corner), 0.5);
        let up_mid = scale(add(corner, tip), 0.5);
        pins.push(Pin {
            mob: leg_across,
            at: point_fn(move |_: &Arena| across_mid),
            shift: v(0.0, -0.32),
        });
        pins.push(Pin {
            mob: leg_up,
            at: point_fn(move |_: &Arena| up_mid),
            shift: v(0.5, 0.0),
        });
        let spins_only = kit.text(stage, ONLY_SPINS, 26.0, Palette::dim(), v(0.0, 0.0))?;
        pins.push(Pin {
            mob: spins_only,
            at: point_fn(move |_: &Arena| tip),
            shift: v(1.05, -0.4),
        });

        let world = World::build(stage, view, 1.4, 0.25, specs, pins)?;
        let hidden: Vec<usize> = ArmRig::GOLD_COLLARS
            .chain([
                part::GOAL,
                part::TURNING_SHOULDER,
                part::TURNING_ELBOW,
                part::TWIST_SHOULDER,
                part::TWIST_BASE,
                part::TWIST_SUM,
                part::TWIST_TAIL,
                part::TWIST_WRIST,
                part::LEVER_BASE,
                part::LEVER_SHOULDER,
                part::LEVER_WRIST,
                part::TRIANGLE,
            ])
            .collect();
        for i in hidden {
            stage.set_tracker_value(world.parts[i].shown, 0.0)?;
        }
        world.add_to_scene(stage)?;
        let shown = |i: usize| world.parts[i].shown;
        let collar = |joint: usize| world.parts[ArmRig::GOLD_COLLARS.start + joint].shown;

        // 10_1: the pose I want; which joint angles?; the answer starts with
        // speeds
        let line = say(&kit, stage, BEATS[0])?;
        world.show(stage, 0.8)?;
        line.wait_for(stage, "This is the pose I want")?;
        play!(stage;
            tween(shown(part::GOAL), 1.0, 0.6)?,
            fade_in(stage.arena_mut(), goal_label, ORIGIN, 1.0)?.rt(0.6),
        );
        line.wait_for(stage, "Which joint angles")?;
        let unknown = kit.text(stage, ANGLES_WORDS, 32.0, Palette::ink(), v(3.4, 1.9))?;
        fade_in_all(stage, &[unknown], 0.5)?;
        line.wait_for(stage, "The answer starts with speeds")?;
        play!(stage;
            tween(shown(part::TURNING_SHOULDER), 1.0, 0.4)?,
            tween(shown(part::TURNING_ELBOW), 1.0, 0.4)?,
        );
        // the curls turn while the goal steps back (it stays, dimmed, through
        // chapters 10 to 12)
        play!(stage;
            tween(turn, 1.0, 2.2)?,
            tween(shown(part::GOAL), GOAL_DIMMED, 1.0)?,
        );
        leave(stage, &[unknown, goal_label], &[], 0.3)?;

        // 10_2: each axis counts where it is right now; a joint turning
        // alone is a twist; twists add
        let line = say(&kit, stage, BEATS[1])?;
        play!(stage;
            tween(shown(part::TURNING_SHOULDER), 0.0, 0.4)?,
            tween(shown(part::TURNING_ELBOW), 0.0, 0.4)?,
        );
        line.wait_for(stage, "A joint turning alone")?;
        play!(stage;
            tween(collar(SHOULDER), 1.0, 0.4)?,
            tween(shown(part::TWIST_SHOULDER), 1.0, 0.6)?,
        );
        line.wait_for(stage, "That is a twist")?;
        play!(stage;
            tween(collar(SHOULDER), 0.0, 0.3)?,
            tween(collar(0), 1.0, 0.4)?,
            tween(shown(part::TWIST_BASE), 1.0, 0.6)?,
        );
        line.wait_for(stage, "so they add")?;
        play!(stage;
            tween(collar(SHOULDER), 1.0, 0.4)?,
            tween(shown(part::TWIST_TAIL), 1.0, 0.6)?,
        );
        play!(stage; tween(shown(part::TWIST_SUM), 1.0, 0.8)?);
        line.wait_end(stage)?;

        // 10_3: seven twists side by side: the Jacobian
        let line = say(&kit, stage, BEATS[2])?;
        play!(stage;
            tween(shown(part::TWIST_SHOULDER), 0.0, 0.5)?,
            tween(shown(part::TWIST_BASE), 0.0, 0.5)?,
            tween(shown(part::TWIST_SUM), 0.0, 0.5)?,
            tween(shown(part::TWIST_TAIL), 0.0, 0.5)?,
            tween(collar(SHOULDER), 0.0, 0.4)?,
            tween(collar(0), 0.0, 0.4)?,
        );
        let entries = grid_entries()?;
        let (rows, cols) = (entries.len(), entries[0].len());
        let mut columns: Vec<Vec<Mob>> = Vec::new();
        let mut ties = Vec::new();
        for c in 0..cols {
            let x = GRID_X0 + GRID_PITCH * c as f64;
            let mut col = Vec::new();
            for (r, row) in entries.iter().enumerate() {
                col.push(kit.text(stage, &row[c], GRID_PT, orange, v(x, GRID_Y0 - GRID_ROW * r as f64))?);
            }
            let joint_at = view.project(arm.joint(&q, c).0);
            let tie = polyline(vec![joint_at, v(x, GRID_Y0 + 0.4)])
                .style(faded_line(orange, 1.6, 0.55))
                .build();
            ties.push(stage.arena_mut().add(tie));
            columns.push(col);
        }
        line.wait_for(stage, "Seven joints")?;
        for (col, &tie) in columns.iter().zip(&ties) {
            let mut ins = col.clone();
            ins.push(tie);
            fade_in_all(stage, &ins, 0.35)?;
        }
        line.wait_for(stage, "A grid, six rows by seven columns")?;
        let spin_label = kit.text(
            stage,
            SPIN_ROWS,
            24.0,
            Palette::dim(),
            v(GRID_X0 - 0.95, GRID_Y0 - GRID_ROW),
        )?;
        let slide_label = kit.text(
            stage,
            SLIDE_ROWS,
            24.0,
            Palette::dim(),
            v(GRID_X0 - 0.95, GRID_Y0 - GRID_ROW * 4.0),
        )?;
        let (top, bottom) = (GRID_Y0 + 0.3, GRID_Y0 - GRID_ROW * (rows as f64 - 1.0) - 0.3);
        let (left_x, right_x) = (GRID_X0 - 0.5, GRID_X0 + GRID_PITCH * (cols as f64 - 1.0) + 0.5);
        let bracket = |x: f64, inward: f64| {
            polyline(vec![v(x + inward, top), v(x, top), v(x, bottom), v(x + inward, bottom)])
                .style(line_style(Palette::ink(), 3.0))
                .build()
        };
        let brackets = [
            stage.arena_mut().add(bracket(left_x, 0.18)),
            stage.arena_mut().add(bracket(right_x, -0.18)),
        ];
        leave(stage, &ties, &[], 0.5)?;
        let mut frame = vec![spin_label, slide_label];
        frame.extend(brackets);
        fade_in_all(stage, &frame, 0.6)?;
        line.wait_for(stage, "Joint speeds go in")?;
        let eq = kit.tex(stage, JACOBIAN_EQ, 54.0, Palette::ink(), &[], v(4.0, -0.75))?;
        fade_in_all(stage, &[eq], 0.6)?;
        let programmers = tag(&kit, stage, PROGRAMMERS, v(2.2, -1.55), 24.0)?;
        let notes = vec![
            kit.text_left(stage, TWIST_SLIDE_NOTE, 22.0, Palette::dim(), v(0.6, -2.0))?,
            kit.tex_left(stage, TIP_VELOCITY_TEX, 26.0, Palette::dim(), &[], v(0.6, -2.42))?,
        ];
        let mut ins = vec![programmers];
        ins.extend(&notes);
        fade_in_all(stage, &ins, 0.6)?;
        let third = lower_third(&kit, stage, JACOBIAN_LABEL, JACOBIAN_TEXT)?;
        line.wait_for(stage, "The gripper's twist comes out")?;
        fade_in_all(stage, &[third], 0.5)?;
        line.wait_end(stage)?;

        // 10_4: which joint moves the tip fastest?
        let line = say(&kit, stage, BEATS[3])?;
        let mut away: Vec<Mob> = columns.into_iter().flatten().collect();
        away.extend([eq, programmers, third, spin_label, slide_label]);
        away.extend(&notes);
        away.extend(brackets);
        leave(stage, &away, &[], 0.5)?;
        let card = QuizCard::build(&kit, stage, ASK, QuizLook::at(v(CARD_AT[0], CARD_AT[1]), CARD_SCALE))?;
        // the three joints light in turn as the question names them
        for (k, (&joint, label)) in ASKED.iter().zip(&asked_labels).enumerate() {
            line.wait_for(stage, ["Joint one", "two or", "seven"][k])?;
            if k > 0 {
                stage.play(tween(collar(ASKED[k - 1]), 0.0, 0.25)?)?;
            }
            play!(stage;
                tween(collar(joint), 1.0, 0.3)?,
                fade_in(stage.arena_mut(), *label, ORIGIN, 1.0)?.rt(0.3),
            );
        }
        line.wait_last(stage)?;
        card.show(stage, 0.5)?;
        card.count(&kit, stage)?;

        // 10_5: joint two: what counts is how far the tip is from each axis
        let line = say(&kit, stage, BEATS[4])?;
        leave(
            stage,
            &asked_labels,
            &[collar(ASKED[0]), collar(ASKED[1]), collar(ASKED[2])],
            0.3,
        )?;
        // the answer, then what it is that counts
        play!(stage; tween(collar(SHOULDER), 1.0, 0.4)?);
        line.wait_for(stage, "What counts is how far")?;
        play!(stage; tween(shown(part::TRIANGLE), 1.0, 0.6)?);
        line.wait_for(stage, "Sixty-seven centimeters")?;
        play!(stage;
            tween(shown(part::LEVER_SHOULDER), 1.0, 0.6)?,
            fade_in(stage.arena_mut(), lever_labels[1], ORIGIN, 1.0)?.rt(0.6),
        );
        let legs = [leg_across, leg_up];
        fade_in_lagged(stage, &legs, 0.8, 0.5)?;
        line.wait_for(stage, "Forty-two from the base's")?;
        play!(stage;
            tween(collar(SHOULDER), 0.0, 0.3)?,
            tween(collar(0), 1.0, 0.4)?,
            tween(shown(part::LEVER_BASE), 1.0, 0.6)?,
            fade_in(stage.arena_mut(), lever_labels[0], ORIGIN, 1.0)?.rt(0.6),
        );
        line.wait_for(stage, "And the tip sits right on")?;
        play!(stage;
            tween(collar(0), 0.0, 0.3)?,
            tween(collar(ASKED[2]), 1.0, 0.4)?,
            tween(shown(part::LEVER_WRIST), 1.0, 0.6)?,
            fade_in(stage.arena_mut(), lever_labels[2], ORIGIN, 1.0)?.rt(0.6),
        );
        line.wait_for(stage, "so that joint only spins it")?;
        // the three speeds at the tip in proportion: the shoulder's, the
        // base's, and the wrist's spin alone
        play!(stage;
            tween(shown(part::TWIST_SHOULDER), 1.0, 0.7)?,
            tween(shown(part::TWIST_BASE), 1.0, 0.7)?,
            tween(shown(part::TWIST_WRIST), 1.0, 0.7)?,
            fade_in(stage.arena_mut(), spins_only, ORIGIN, 1.0)?.rt(0.7),
        );
        line.wait_end(stage)?;
        kit.hold(stage)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::fact_list;
    use super::*;
    use crate::model::{DEG, N, norm};

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

    fn close(a: V3, b: V3) -> bool {
        norm(sub(a, b)) < 1e-9
    }

    /// The distance from `p` to the line through `a` along the unit `d`,
    /// done independently of the model's `lever`.
    fn point_to_line(p: V3, a: V3, d: V3) -> f64 {
        let r = sub(p, a);
        let along = model::dot(r, d);
        norm(sub(r, scale(d, along)))
    }

    #[test]
    fn the_beats_are_the_storyboards_and_the_scripts() {
        let board = explainer_kit::storyboard::parse(crate::STORYBOARD).unwrap();
        let chapter = board.iter().find(|c| c.key == "10_velocities_add").expect("chapter 10");
        let ids: Vec<&str> = chapter.beats.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(ids, BEATS);
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for beat in &chapter.beats {
            let words = beat.spoken().expect("every beat of chapter 10 is spoken");
            assert_eq!(script.text(&beat.id), Some(words.as_str()), "{}", beat.id);
        }
    }

    #[test]
    fn every_cue_phrase_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("10_1", "This is the pose I want"),
            ("10_1", "Which joint angles"),
            ("10_1", "The answer starts with speeds"),
            ("10_2", "A joint turning alone"),
            ("10_2", "That is a twist"),
            ("10_2", "so they add"),
            ("10_3", "Seven joints"),
            ("10_3", "A grid, six rows by seven columns"),
            ("10_3", "Joint speeds go in"),
            ("10_3", "The gripper's twist comes out"),
            ("10_4", "Joint one"),
            ("10_4", "two or"),
            ("10_4", "seven"),
            ("10_5", "What counts is how far"),
            ("10_5", "Sixty-seven centimeters"),
            ("10_5", "Forty-two from the base's"),
            ("10_5", "And the tip sits right on"),
            ("10_5", "so that joint only spins it"),
        ] {
            let text = script.text(id).unwrap_or_default();
            assert!(text.contains(phrase), "{id} does not say {phrase:?}");
        }
    }

    /// 10_3: the grid is six rows by seven columns, it is the arm's own
    /// Jacobian at the L shape to the two places printed, and its second
    /// column is the pinned shoulder screw.
    #[test]
    fn the_grid_is_the_jacobian_of_the_l_shape() {
        let arm = Arm::iiwa14();
        let j = arm.jacobian(&model::degrees(poses::ELL));
        let printed = grid_entries().unwrap();
        assert_eq!(fact("G-J.shape").unwrap(), "6x7");
        assert_eq!((printed.len(), printed[0].len()), (6, N));
        for (r, row) in printed.iter().enumerate() {
            assert_eq!(row.len(), N);
            for (c, text) in row.iter().enumerate() {
                // the printed minus is the true one
                assert!(!text.contains('-'), "{text:?} has a hyphen for a minus");
                let shown: f64 = text.replace('−', "-").parse().unwrap();
                assert!(
                    (shown - j[r][c]).abs() < 0.005 + 1e-9,
                    "[{r}][{c}] {text} against {}",
                    j[r][c]
                );
            }
        }
        let col2: Vec<f64> = fact_list("G-J.ell.col2").unwrap();
        let from_grid: Vec<f64> = (0..6)
            .map(|r| printed[r][1].replace('−', "-").parse().unwrap())
            .collect();
        for (a, b) in col2.iter().zip(&from_grid) {
            assert!((a - b).abs() < 0.005, "{a} against {b}");
        }
    }

    /// Entries are printed whole or at two places, zero as 0, and a negative
    /// with the true minus sign (U+2212), as every matrix of the video is.
    #[test]
    fn entries_print_whole_or_at_two_places_with_a_true_minus() {
        let rows = grid_entries().unwrap();
        let all: Vec<&str> = rows.iter().flatten().map(String::as_str).collect();
        assert_eq!(all.len(), 6 * N);
        for want in ["0", "1", "−1", "−0.36", "0.36", "−0.42", "0.42", "−0.76"] {
            assert!(all.contains(&want), "the grid has no {want:?}");
        }
        assert!(all.iter().all(|e| !e.contains('-') && *e != "−0" && *e != "0.00"));
    }

    /// The tag's claim: the tip's velocity is the slide plus the spin
    /// crossed with the tip's position, for every column of the Jacobian,
    /// against a finite difference of the arm's own forward kinematics.
    #[test]
    fn the_tip_velocity_is_slide_plus_spin_times_position() {
        let arm = Arm::iiwa14();
        let q = model::degrees(poses::ELL);
        let j = arm.jacobian(&q);
        let tip = arm.tip(&q).p;
        let h = 1e-6;
        for i in 0..N {
            let (mut up, mut down) = (q, q);
            up[i] += h;
            down[i] -= h;
            let numeric = scale(sub(arm.tip(&up).p, arm.tip(&down).p), 0.5 / h);
            assert!(
                norm(sub(tip_velocity(&j, i, tip), numeric)) < 1e-6,
                "joint {} tip speed",
                i + 1
            );
        }
    }

    /// 10_2: twists live on the flat piece, so they add: both joints turning
    /// gives the sum of the two tip velocities, and the speed of the tip is
    /// the Jacobian times the joint speeds.
    #[test]
    fn two_joints_turning_together_add_their_twists() {
        let arm = Arm::iiwa14();
        let q = model::degrees(poses::ELL);
        let j = arm.jacobian(&q);
        let tip = arm.tip(&q).p;
        let (v_sh, v_base) = (tip_velocity(&j, SHOULDER, tip), tip_velocity(&j, 0, tip));
        let mut speeds = [0.0; N];
        speeds[SHOULDER] = 1.0;
        speeds[0] = 1.0;
        let twist = model::twist_from(&j, &speeds);
        let together = add(slide_of(twist), cross(spin_of(twist), tip));
        assert!(close(together, add(v_sh, v_base)));
        // and the picture's sum arrow is exactly that
        let Shape::Arrow { from, to, .. } = arrow(tip, add(v_sh, v_base), palette::flat(), 6.0) else {
            panic!("an arrow");
        };
        assert!(close(sub(to, from), scale(together, ARROW_PER_SPEED)));
    }

    /// 10_4 and 10_5: the answer is joint two, by the distance from the tip
    /// to each axis (done here without the model's `lever`), the quiz asks
    /// the three joints the words name, and the voice's numbers are the
    /// facts'.
    #[test]
    fn joint_two_is_fastest_by_the_tips_distance_from_each_axis() {
        let arm = Arm::iiwa14();
        let q = model::degrees(poses::ELL);
        let tip = arm.tip(&q).p;
        let dist = |joint: usize| {
            let (point, dir) = arm.joint(&q, joint);
            point_to_line(tip, point, dir)
        };
        let keys: Vec<usize> = ASK
            .choices
            .iter()
            .map(|c| c.key.parse::<usize>().unwrap() - 1)
            .collect();
        assert_eq!(keys, ASKED);
        let best = keys
            .iter()
            .copied()
            .max_by(|&a, &b| dist(a).total_cmp(&dist(b)))
            .unwrap();
        assert_eq!(best + 1, 2);
        assert_eq!(ASK.choices[ASK.answer_index().unwrap()].key, "2");
        assert_eq!(ASK.seconds().unwrap(), 5);
        // the three numbers of 10_5, in the order the voice says them
        assert_eq!(cm_whole(dist(SHOULDER)), "67");
        assert_eq!(cm_whole(dist(0)), "42");
        assert!(dist(6) < 1e-12, "the tip is on joint seven's axis");
        assert_eq!(cm_whole(dist(6)), "0");
        for (joint, key) in [
            (0, "G-J.ell.lever.1"),
            (SHOULDER, "G-J.ell.lever.2"),
            (6, "G-J.ell.lever.7"),
        ] {
            let want: f64 = fact(key).unwrap().parse().unwrap();
            assert!(
                (dist(joint) - want).abs() < 5e-5,
                "{key}: {} against {want}",
                dist(joint)
            );
            assert!((arm.lever(&q, joint) - want).abs() < 5e-5);
        }
        // no other joint beats the shoulder (the quiz is not a trick)
        for other in [0, 2, 3, 4, 5, 6] {
            assert!(
                dist(other) < dist(SHOULDER),
                "joint {} is nearer than the shoulder's",
                other + 1
            );
        }
    }

    /// 10_5's triangle: legs 42 and 52.6 at a right angle, and its long side
    /// is the shoulder's distance.
    #[test]
    fn the_triangle_under_the_first_lever_is_right_angled() {
        let arm = Arm::iiwa14();
        let q = model::degrees(poses::ELL);
        let (tip, shoulder) = (arm.tip(&q).p, arm.joint(&q, SHOULDER).0);
        let corner = [tip[0], tip[1], shoulder[2]];
        let (across, up) = (sub(corner, shoulder), sub(tip, corner));
        assert!(model::dot(across, up).abs() < 1e-12, "a right angle");
        assert_eq!((cm(norm(across)), cm(norm(up))), ("42".to_owned(), "52.6".to_owned()));
        let hyp = norm(sub(tip, shoulder));
        assert!((hyp - (norm(across).powi(2) + norm(up).powi(2)).sqrt()).abs() < 1e-12);
        assert!((hyp - arm.lever(&q, SHOULDER)).abs() < 1e-12);
    }

    /// The arrows of 10_5 stand in proportion: the shoulder's longer than the
    /// base's by the ratio of the two distances, the wrist's no arrow at all.
    #[test]
    fn the_arrows_are_in_proportion_to_the_levers() {
        let arm = Arm::iiwa14();
        let q = model::degrees(poses::ELL);
        let j = arm.jacobian(&q);
        let tip = arm.tip(&q).p;
        let speed = |i: usize| norm(tip_velocity(&j, i, tip));
        assert!((speed(SHOULDER) / speed(0) - arm.lever(&q, SHOULDER) / arm.lever(&q, 0)).abs() < 1e-12);
        assert!(speed(6) < 1e-12);
        assert!(
            (speed(SHOULDER) - arm.lever(&q, SHOULDER)).abs() < 1e-12,
            "m/s at one radian a second"
        );
        // the quarter-turn of ELL's joint angles is the L shape
        assert_eq!(q.map(|x| (x / DEG).round()), poses::ELL);
    }

    /// The grid fits the frame: its right edge, with real text, is inside it
    /// and clear of the quiz card's column.
    #[test]
    fn the_grid_fits_the_frame() {
        run(|stage| {
            let kit = Kit::new()?;
            let entries = grid_entries()?;
            let mut right: f64 = -10.0;
            for (c, col) in (0..entries[0].len()).enumerate() {
                for (r, row) in entries.iter().enumerate() {
                    let m = kit.text(
                        stage,
                        &row[col],
                        GRID_PT,
                        Palette::ink(),
                        v(GRID_X0 + GRID_PITCH * c as f64, GRID_Y0 - GRID_ROW * r as f64),
                    )?;
                    right = right.max(stage.get_right(m)[0]);
                }
            }
            assert!(right < 6.9, "the grid's right edge is at {right:.2}");
            Ok(())
        });
    }

    #[test]
    fn the_quiz_rows_fit_the_card() {
        run(|stage| {
            let kit = Kit::new()?;
            let card = QuizCard::build(&kit, stage, ASK, QuizLook::at(v(CARD_AT[0], CARD_AT[1]), CARD_SCALE))?;
            let right = stage.get_right(card.back)[0];
            for &row in &card.rows {
                assert!(stage.get_right(row)[0] < right - 0.15);
            }
            assert!(right < 7.1);
            Ok(())
        });
    }

    /// The two lines of the "for programmers" note end inside the frame's
    /// margin (0.15 inside the 7.11 half width), the second set in TeX.
    #[test]
    fn the_programmers_note_ends_inside_the_margin() {
        run(|stage| {
            let kit = Kit::new()?;
            let first = kit.text_left(stage, TWIST_SLIDE_NOTE, 22.0, Palette::dim(), v(0.6, -2.0))?;
            let second = kit.tex_left(stage, TIP_VELOCITY_TEX, 26.0, Palette::dim(), &[], v(0.6, -2.42))?;
            for m in [first, second] {
                assert!(stage.get_right(m)[0] < 7.11 - 0.15, "ends at {}", stage.get_right(m)[0]);
            }
            Ok(())
        });
    }

    #[test]
    fn a_curl_is_an_arc_and_a_head_about_its_axis() {
        let shapes = curl([1.0, 2.0, 3.0], [0.0, 0.0, 1.0], 0.2, CURL_SWEEP, Palette::ink(), 4.0);
        assert_eq!(shapes.len(), 2);
        let Shape::Path { pts, .. } = &shapes[0] else {
            panic!("an arc");
        };
        for p in pts {
            assert!((norm(sub(*p, [1.0, 2.0, 3.0])) - 0.2).abs() < 1e-12, "on the circle");
            assert!((p[2] - 3.0).abs() < 1e-12, "in the plane");
        }
    }
}
