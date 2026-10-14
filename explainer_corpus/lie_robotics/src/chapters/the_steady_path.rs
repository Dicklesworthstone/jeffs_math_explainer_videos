//! Chapter 14, the steady path: what should the gripper do between the
//! start and the goal (14_1); blend the two orientations number by number,
//! is the result halfway there (14_2); it is not even an orientation, a
//! gripper turned by it is squashed to half its volume (14_3); the steady
//! path is the screw, half the turn and half the slide (14_4); and the tip
//! may go straight instead, seven point six centimeters off (14_5).

use explainer_kit::props::{lower_third, precisely};
use explainer_kit::{Kit, Palette, Timed, play, tween, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::draw::{
    ARM_SCALE, ArmRig, Ask, BOOK_HALF, Choice, PartSpec, Pin, QuizCard, QuizLook, Shape, View, World, along_a_screw,
    book, cuboid, entry, fact, fact_f64, fade_in_all, fade_in_lagged, faded_line, leave, mix, point_fn, say,
    screw_shapes, static_grid, still, table, tag,
};
use super::steps::{goal_frame, pose_frame};
use crate::model::{self, Arm, DEG, M3, Pose, V3, add, angle_axis, det, mul, norm, poses, scale, sub, transpose};
use crate::palette;

pub struct TheSteadyPath {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const START_WORDS: &str = "start";
const GOAL_WORDS: &str = "goal";
const ON_THE_WAY: &str = "on the way?";
const IS_IT_HALFWAY: &str = "is the result halfway there?";
const HALFWAY: &str = "halfway";
const NOT_ORIENTATION: &str = "not even an orientation";
const PLUS: &str = "+";
const HALF_OF_EACH: &str = "half of each:";
const VOLUME: &str = "volume";
const NEVER_A_TURN: &str = "a blend of two different turns is never a turn";
const PROGRAMMERS: &str = "for programmers";
const RENORMALISE: &str = "re-normalising repairs the shape, not the timing:";
const A_QUARTER_OF_THE_WAY: &str = "a quarter of the way it has turned";
const NOT: &str = "not";
const TURNED: &str = "turned";
const CM_DOWN: &str = "cm down";
const STEADY_LABEL: &str = "'steady' here means one constant twist.";
const STEADY_TEXT: &str =
    "Splitting the job, a straight line for the tip and the turn on its own, is common in practice";

/// The beats of the chapter, in order (storyboard ids).
const BEATS: [&str; 5] = ["14_1", "14_2", "14_3", "14_4", "14_5"];

/// 14_2's quiz: halfway, or not even an orientation (`G-Q.14_2.answer`).
pub const ASK: Ask = Ask {
    line: BEATS[1],
    question: IS_IT_HALFWAY,
    choices: &[
        Choice {
            key: "pose",
            label: HALFWAY,
        },
        Choice {
            key: "not",
            label: NOT_ORIENTATION,
        },
    ],
};

const ARM_AT: [f64; 2] = [-1.9, -2.45];
const CARD_AT: [f64; 2] = [4.5, -0.1];
const CARD_SCALE: f64 = 0.86;
/// The three grids of 14_2: their centres across, their height, their pitch.
const GRID_X: [f64; 3] = [-5.55, -2.95, -0.3];
const GRID_Y: f64 = 0.9;
const GRID_PITCH: (f64, f64) = (0.62, 0.52);
const GRID_PT: f64 = 26.0;
/// The five grippers of 14_3: from the left edge, their spacing and height.
const BLEND_X0: f64 = -5.8;
const BLEND_DX: f64 = 1.62;
const BLEND_Y: f64 = -1.0;
/// The circle picture of 14_3: its centre and radius.
const CIRCLE_AT: [f64; 2] = [-4.9, 2.55];
const CIRCLE_R: f64 = 1.0;
/// A gripper's half sizes, in picture units.
const GRIPPER_HALF: V3 = [0.46, 0.3, 0.2];
/// The steps at which the blend is shown: 0, 1/4, 1/2, 3/4, 1.
const BLEND_STEPS: [f64; 5] = [0.0, 0.25, 0.5, 0.75, 1.0];
/// How many poses the corkscrew is solved at.
const SCREW_SAMPLES: usize = 24;
/// The "for programmers" tag of 14_3, and the gap before the note it labels.
const TAG_AT: [f64; 2] = [-5.35, -2.75];
const NOTE_GAP: f64 = 0.2;
// the three grids end before the quiz card begins, the row of grippers ends
// before it too, and both start inside the frame
const _: () = assert!(GRID_X[2] + 1.62 * GRID_PITCH.0 < CARD_AT[0] - 2.2);
const _: () = assert!(GRID_X[0] - 1.62 * GRID_PITCH.0 > -7.0);
const _: () = assert!(BLEND_X0 - GRIPPER_HALF[0] * 1.4 > -7.0);
const _: () = assert!(BLEND_X0 + BLEND_DX * 4.0 + GRIPPER_HALF[0] * 1.4 < 3.0);

/// The parts this chapter adds to the arm's own 23: the table is 23 and the
/// book 24.
mod part {
    pub const GOAL: usize = 25;
    pub const START: usize = 26;
    /// The screw from start to goal, with its thread (27).
    pub const SCREW: usize = 27;
    /// The tip's path along the screw, the straight line, and the gap.
    pub const HELIX: usize = 28;
    pub const LINE: usize = 29;
    pub const GAP: usize = 30;
}

/// The blend `(1 - s) a + s b` of two matrices, number by number.
pub fn blend(a: &M3, b: &M3, s: f64) -> M3 {
    std::array::from_fn(|i| std::array::from_fn(|k| (1.0 - s) * a[i][k] + s * b[i][k]))
}

/// Two places, ties away from zero, so that 0.625 reads 0.63 everywhere.
pub fn two_places(x: f64) -> String {
    format!("{:.2}", (x * 100.0).round() / 100.0 + 0.0)
}

/// A matrix as the grid prints it.
fn rows_of(m: &M3) -> [[String; 3]; 3] {
    std::array::from_fn(|i| std::array::from_fn(|k| entry(m[i][k])))
}

/// The start and goal orientations the chapter's facts print.
fn start_and_goal() -> fmn::Result<(M3, M3)> {
    let parse = |key: &str| -> fmn::Result<M3> {
        let rows: Vec<Vec<f64>> = fact(key)?
            .split(';')
            .map(|r| r.split(',').filter_map(|x| x.trim().parse().ok()).collect())
            .collect();
        if rows.len() != 3 || rows.iter().any(|r| r.len() != 3) {
            return Err(super::draw::fail(format!("{key} is not a 3 by 3 grid")));
        }
        Ok(std::array::from_fn(|i| std::array::from_fn(|k| rows[i][k])))
    };
    Ok((parse("G-I.start.R")?, parse("G-I.goal.R")?))
}

/// The world points of the tip along the screw (`steps` + 1 of them) and
/// along the straight line between the two tip positions.
pub fn two_paths(start: &Pose, screw: &model::Screw, goal_p: V3, steps: usize) -> (Vec<V3>, Vec<V3>) {
    let helix = (0..=steps)
        .map(|k| model::along_screw(screw, k as f64 / steps as f64).then(start).p)
        .collect();
    let line = (0..=steps)
        .map(|k| add(start.p, scale(sub(goal_p, start.p), k as f64 / steps as f64)))
        .collect();
    (helix, line)
}

/// Where the two paths are farthest apart: the index along them and the gap.
pub fn widest_gap(helix: &[V3], line: &[V3]) -> (usize, f64) {
    helix
        .iter()
        .zip(line)
        .enumerate()
        .map(|(k, (a, b))| (k, norm(sub(*a, *b))))
        .fold((0, 0.0), |best, now| if now.1 > best.1 { now } else { best })
}

impl SceneConstruct for TheSteadyPath {
    fn name(&self) -> &str {
        "14_the_steady_path"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let arm = Arm::iiwa14();
        let (orange, gold, red, blue, teal) = (
            palette::flat(),
            palette::screw(),
            palette::warn(),
            palette::turn(),
            palette::place(),
        );
        let q0 = model::degrees(poses::START);
        let start = arm.tip(&q0);
        let goal = model::goal(&arm);
        let screw = model::screw_of(&goal.then(&start.undo()));
        let view = View::new(ARM_SCALE, v(ARM_AT[0], ARM_AT[1]));

        // ---- the arm world: the arm walks the corkscrew; the poses at
        // each end, the screw, the tip's two paths and their widest gap
        let keys = along_a_screw(&arm, &q0, &start, &screw, SCREW_SAMPLES);
        let (rig, walk) = ArmRig::along(stage, arm.clone(), keys);
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
        specs.push(PartSpec::new(goal_frame(&goal, 0.17), still(Pose::NOTHING)).layer(2));
        specs.push(PartSpec::new(pose_frame(&start, 0.17, blue), still(Pose::NOTHING)).layer(2));
        specs.push(PartSpec::new(screw_shapes(&screw, 0.9, 0.16), still(Pose::NOTHING)).layer(3));
        let (helix, line) = two_paths(&start, &screw, goal.p, 60);
        specs.push(
            PartSpec::new(
                vec![Shape::Path {
                    pts: helix.clone(),
                    color: gold,
                    width: 5.0,
                    closed: false,
                    bias: 0.0,
                }],
                still(Pose::NOTHING),
            )
            .layer(3),
        );
        specs.push(
            PartSpec::new(
                vec![Shape::Path {
                    pts: line.clone(),
                    color: teal,
                    width: 5.0,
                    closed: false,
                    bias: 0.0,
                }],
                still(Pose::NOTHING),
            )
            .layer(3),
        );
        let (widest, gap) = widest_gap(&helix, &line);
        specs.push(
            PartSpec::new(
                vec![Shape::Dashed {
                    a: helix[widest],
                    b: line[widest],
                    color: red,
                    width: 4.0,
                    dashes: 5,
                }],
                still(Pose::NOTHING),
            )
            .layer(4),
        );
        let mut pins = Vec::new();
        let start_label = kit.text(stage, START_WORDS, 30.0, blue, v(0.0, 0.0))?;
        let goal_label = kit.text(stage, GOAL_WORDS, 30.0, palette::goal(), v(0.0, 0.0))?;
        for (mob, p, shift) in [(start_label, start.p, v(-0.9, 0.55)), (goal_label, goal.p, v(1.0, 0.5))] {
            pins.push(Pin {
                mob,
                at: point_fn(move |_: &Arena| p),
                shift,
            });
        }
        let gap_label = kit.text(stage, &format!("{:.1} cm", gap * 100.0), 32.0, red, v(0.0, 0.0))?;
        let gap_at = scale(add(helix[widest], line[widest]), 0.5);
        pins.push(Pin {
            mob: gap_label,
            at: point_fn(move |_: &Arena| gap_at),
            shift: v(1.0, 0.0),
        });
        let world = World::build(stage, view, 1.4, 0.25, specs, pins)?;
        for i in ArmRig::GOLD_COLLARS.chain([part::GOAL, part::START, part::SCREW, part::HELIX, part::LINE, part::GAP])
        {
            stage.set_tracker_value(world.parts[i].shown, 0.0)?;
        }
        // the arm is faint behind the two poses until it is the subject
        for i in ArmRig::LINKS.chain(ArmRig::STEEL_COLLARS) {
            stage.set_tracker_value(world.parts[i].grey, 0.9)?;
        }
        world.add_to_scene(stage)?;
        let shown = |i: usize| world.parts[i].shown;

        // 14_1: one more question: between the start and the goal
        let line_14_1 = say(&kit, stage, BEATS[0])?;
        world.show(stage, 0.8)?;
        line_14_1.wait_for(stage, "Between the start and the goal")?;
        play!(stage;
            tween(shown(part::START), 1.0, 0.6)?,
            tween(shown(part::GOAL), 1.0, 0.6)?,
            fade_in(stage.arena_mut(), start_label, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), goal_label, ORIGIN, 1.0)?.rt(0.6),
        );
        line_14_1.wait_for(stage, "what should the gripper do")?;
        let way = kit.text(stage, ON_THE_WAY, 36.0, Palette::ink(), v(3.4, 1.8))?;
        fade_in_all(stage, &[way], 0.5)?;
        line_14_1.wait_end(stage)?;

        // 14_2: blend the two orientations, number by number
        let line_14_2 = say(&kit, stage, BEATS[1])?;
        let card = QuizCard::build(&kit, stage, ASK, QuizLook::at(v(CARD_AT[0], CARD_AT[1]), CARD_SCALE))?;
        leave(stage, &[way, start_label, goal_label], &[world.shown], 0.6)?;
        let (rs, rg) = start_and_goal()?;
        let mid = blend(&rs, &rg, 0.5);
        let blue_cols = [blue, blue, blue];
        let pink_cols = [palette::goal(); 3];
        line_14_2.wait_for(stage, "Blend the two orientations")?;
        let g_start = static_grid(
            &kit,
            stage,
            rows_of(&rs),
            v(GRID_X[0], GRID_Y),
            GRID_PITCH,
            GRID_PT,
            blue_cols,
        )?;
        let g_goal = static_grid(
            &kit,
            stage,
            rows_of(&rg),
            v(GRID_X[1], GRID_Y),
            GRID_PITCH,
            GRID_PT,
            pink_cols,
        )?;
        fade_in_lagged(stage, &[g_start.clone(), g_goal.clone()].concat(), 1.0, 0.5)?;
        line_14_2.wait_for(stage, "Half of one matrix")?;
        let half = kit.text(stage, HALF_OF_EACH, 28.0, Palette::dim(), v(GRID_X[2], GRID_Y + 1.35))?;
        let plus = kit.text(
            stage,
            PLUS,
            40.0,
            Palette::ink(),
            v((GRID_X[0] + GRID_X[1]) / 2.0, GRID_Y),
        )?;
        let g_mid = static_grid(
            &kit,
            stage,
            rows_of(&mid),
            v(GRID_X[2], GRID_Y),
            GRID_PITCH,
            GRID_PT,
            [orange; 3],
        )?;
        let mut ins = vec![plus, half];
        ins.extend(&g_mid);
        fade_in_all(stage, &ins, 0.8)?;
        line_14_2.wait_last(stage)?;
        card.show(stage, 0.5)?;
        card.count(&kit, stage)?;

        // 14_3: not even an orientation: squashed to half its volume
        let line_14_3 = say(&kit, stage, BEATS[2])?;
        // the badge's circle: two arrows a quarter turn apart, and their
        // average, which stops short of the circle
        let c = v(CIRCLE_AT[0], CIRCLE_AT[1]);
        let rim = stage.arena_mut().add(
            Circle::new()
                .radius(CIRCLE_R)
                .arc_center(c)
                .style(faded_line(blue, 3.0, 0.8))
                .build(),
        );
        let (u1, u2) = ([1.0, 0.0], [0.0, 1.0]);
        let tip_of = |u: [f64; 2], len: f64| v(c[0] + CIRCLE_R * len * u[0], c[1] + CIRCLE_R * len * u[1]);
        let a1 = stage
            .arena_mut()
            .add(Arrow::new(c, tip_of(u1, 1.0)).thickness(4.0).color(blue).build()?);
        let a2 = stage
            .arena_mut()
            .add(Arrow::new(c, tip_of(u2, 1.0)).thickness(4.0).color(blue).build()?);
        let mean = [(u1[0] + u2[0]) / 2.0, (u1[1] + u2[1]) / 2.0];
        let short = (mean[0].hypot(mean[1])).abs();
        let a3 = stage.arena_mut().add(
            DashedLine::new(c, tip_of(mean, 1.0))
                .dash_length(0.14)
                .style(faded_line(red, 4.0, 1.0))
                .build()?,
        );
        let short_label = kit.text(stage, &two_places(short), 30.0, red, v(c[0] + 0.55, c[1] + 0.7))?;
        line_14_3.wait_for(stage, "It is not even an orientation")?;
        fade_in_all(stage, &[rim, a1, a2, a3, short_label], 0.7)?;
        // the five grippers through the blend, and what they weigh
        let mut gripper_parts = Vec::new();
        for (k, &s) in BLEND_STEPS.iter().enumerate() {
            gripper_parts.push(PartSpec::new(
                cuboid(
                    [0.0; 3],
                    GRIPPER_HALF,
                    [mix(blue, Palette::ink(), 0.3); 6],
                    Some((super::draw::ground(), 1.4)),
                ),
                still(Pose {
                    r: blend(&rs, &rg, s),
                    p: [BLEND_X0 + BLEND_DX * k as f64, BLEND_Y, 0.0],
                }),
            ));
        }
        let blend_world = World::build(stage, View::new(1.0, v(0.0, 0.0)), 6.0, 0.0, gripper_parts, Vec::new())?;
        blend_world.add_to_scene(stage)?;
        line_14_3.wait_for(stage, "A gripper turned by that blend")?;
        blend_world.show(stage, 0.8)?;
        let mut volumes = Vec::new();
        for (k, &s) in BLEND_STEPS.iter().enumerate() {
            let words = format!("{VOLUME} {}", two_places(det(&blend(&rs, &rg, s))));
            volumes.push(kit.text(
                stage,
                &words,
                24.0,
                if (s - 0.5).abs() < 1e-9 { red } else { Palette::dim() },
                v(BLEND_X0 + BLEND_DX * k as f64, BLEND_Y - 1.2),
            )?);
        }
        fade_in_lagged(stage, &volumes, 1.4, 0.4)?;
        let never = precisely(&kit, stage, NEVER_A_TURN)?;
        let programmers = tag(&kit, stage, PROGRAMMERS, v(TAG_AT[0], TAG_AT[1]), 22.0)?;
        let quarter_blend = fact_f64("G-I.blend.quarter_turned")?;
        let quarter_screw = fact_f64("G-I.screw.quarter_turned")?;
        // the note starts a little right of the tag, wherever the tag ends
        let note_left = stage.get_right(programmers)[0] + NOTE_GAP;
        let renorm = kit.text_left(
            stage,
            &format!(
                "{RENORMALISE} {A_QUARTER_OF_THE_WAY} {:.1}°, {NOT} {:.1}°",
                quarter_blend, quarter_screw
            ),
            22.0,
            Palette::dim(),
            v(note_left, TAG_AT[1]),
        )?;
        line_14_3.wait_end(stage)?;
        fade_in_all(stage, &[never, programmers, renorm], 0.6)?;
        kit.music_breath(stage, 1.0)?;

        // 14_4: the steady path is the screw
        let line_14_4 = say(&kit, stage, BEATS[3])?;
        let mut away = vec![rim, a1, a2, a3, short_label, never, programmers, renorm];
        away.extend(&g_start);
        away.extend(&g_goal);
        away.extend(&g_mid);
        away.extend([plus, half]);
        away.extend(&volumes);
        away.extend(card.mobs());
        leave(stage, &away, &[blend_world.shown], 0.6)?;
        for i in ArmRig::LINKS.chain(ArmRig::STEEL_COLLARS) {
            stage.set_tracker_value(world.parts[i].grey, 0.0)?;
        }
        world.show(stage, 0.8)?;
        play!(stage;
            tween(shown(part::START), 1.0, 0.4)?,
            tween(shown(part::GOAL), 1.0, 0.4)?,
        );
        line_14_4.wait_for(stage, "Take the twist that joins the two poses")?;
        play!(stage; tween(shown(part::SCREW), 1.0, 0.7)?);
        // the readouts: how far it has turned, and the volume of the
        // gripper, read off the arm as it corkscrews
        let turned_label = kit.text_left(stage, TURNED, 30.0, Palette::dim(), v(2.2, 2.4))?;
        let volume_label = kit.text_left(stage, VOLUME, 30.0, Palette::dim(), v(2.2, 1.7))?;
        fade_in_all(stage, &[turned_label, volume_label], 0.4)?;
        let (probe, arm_for) = (rig.clone(), arm.clone());
        let r0 = start.r;
        let turned_now = kit.live_value(
            stage,
            move |a: &Arena| angle_axis(&mul(&transpose(&r0), &arm_for.tip(&probe.q(a)).r)).0 / DEG,
            0,
            false,
            38.0,
            Palette::ink(),
            v(5.4, 2.4),
            RIGHT,
        )?;
        let (probe, arm_for) = (rig.clone(), arm.clone());
        let volume_now = kit.live_value(
            stage,
            move |a: &Arena| det(&arm_for.tip(&probe.q(a)).r),
            2,
            false,
            38.0,
            Palette::ink(),
            v(5.4, 1.7),
            RIGHT,
        )?;
        let degrees_word = kit.text(stage, "°", 38.0, Palette::dim(), v(5.65, 2.4))?;
        fade_in_all(stage, &[degrees_word], 0.2)?;
        line_14_4.wait_for(stage, "hold it for half a second")?;
        play!(stage; tween(walk, 1.0, 4.4)?);
        let slide_cm = format!(
            "{} {CM_DOWN}",
            (fact_f64("G-I.screw.slide")?.abs() * 100.0 * 10.0).round() / 10.0
        );
        let slide_text = kit.text(stage, &slide_cm, 30.0, gold, v(4.0, 0.9))?;
        fade_in_all(stage, &[slide_text], 0.5)?;
        line_14_4.wait_end(stage)?;
        kit.music_breath(stage, 1.0)?;

        // 14_5: the tip may go straight, because positions do add
        let line_14_5 = say(&kit, stage, BEATS[4])?;
        leave(
            stage,
            &[
                turned_label,
                volume_label,
                degrees_word,
                slide_text,
                turned_now,
                volume_now,
            ],
            &[],
            0.4,
        )?;
        play!(stage; tween(walk, 0.0, 1.0)?);
        line_14_5.wait_for(stage, "The tip may go straight instead")?;
        play!(stage;
            tween(shown(part::LINE), 1.0, 0.6)?,
            tween(shown(part::HELIX), 1.0, 0.6)?,
        );
        line_14_5.wait_for(stage, "Here the two paths differ")?;
        play!(stage;
            tween(shown(part::GAP), 1.0, 0.5)?,
            fade_in(stage.arena_mut(), gap_label, ORIGIN, 1.0)?.rt(0.5),
        );
        let steady = lower_third(&kit, stage, STEADY_LABEL, STEADY_TEXT)?;
        fade_in_all(stage, &[steady], 0.6)?;
        line_14_5.wait_end(stage)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            .find(|c| c.key == "14_the_steady_path")
            .expect("chapter 14");
        let ids: Vec<&str> = chapter.beats.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(ids, BEATS);
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for beat in &chapter.beats {
            let words = beat.spoken().expect("every beat of chapter 14 is spoken");
            assert_eq!(script.text(&beat.id), Some(words.as_str()), "{}", beat.id);
        }
    }

    #[test]
    fn every_cue_phrase_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("14_1", "Between the start and the goal"),
            ("14_1", "what should the gripper do"),
            ("14_2", "Blend the two orientations"),
            ("14_2", "Half of one matrix"),
            ("14_3", "It is not even an orientation"),
            ("14_3", "A gripper turned by that blend"),
            ("14_4", "Take the twist that joins the two poses"),
            ("14_4", "hold it for half a second"),
            ("14_5", "The tip may go straight instead"),
            ("14_5", "Here the two paths differ"),
        ] {
            let text = script.text(id).unwrap_or_default();
            assert!(text.contains(phrase), "{id} does not say {phrase:?}");
        }
    }

    /// 14_2: the three grids are the two orientations and their blend; the
    /// blend is half of each and is not an orientation (its determinant is
    /// the fact's one half, its columns are not unit length); the quiz
    /// answer is that.
    #[test]
    fn the_blend_is_half_of_each_and_is_not_a_turn() {
        let (rs, rg) = start_and_goal().unwrap();
        // both are turns
        for r in [&rs, &rg] {
            assert!((det(r) - 1.0).abs() < 1e-12);
            assert!(model::gap(&mul(&transpose(r), r), &model::I3) < 1e-12);
        }
        let mid = blend(&rs, &rg, 0.5);
        for i in 0..3 {
            for k in 0..3 {
                assert!((mid[i][k] - (rs[i][k] + rg[i][k]) / 2.0).abs() < 1e-15);
            }
        }
        assert_eq!(format!("{:.4}", det(&mid)), fact("G-I.blend.det").unwrap());
        assert!(
            model::gap(&mul(&transpose(&mid), &mid), &model::I3) > 0.1,
            "not orthogonal"
        );
        // the printed grids
        assert_eq!(rows_of(&mid)[0], ["−0.50", "−0.50", "0"]);
        assert_eq!(rows_of(&mid)[1], ["−0.50", "0.50", "0"]);
        assert_eq!(rows_of(&mid)[2], ["0", "0", "−1"]);
        assert_eq!(rows_of(&rs)[0], ["−1", "0", "0"]);
        // the quiz
        assert_eq!(ASK.seconds().unwrap(), 4);
        assert_eq!(ASK.choices[ASK.answer_index().unwrap()].label, NOT_ORIENTATION);
    }

    /// 14_3: the volume through the blend falls to one half at the middle
    /// and recovers: 1, 0.62.., 0.5, ... against cos squared of half the
    /// angle, and "a blend of two different turns is never a turn" is the
    /// facts' 66 of 66.
    #[test]
    fn the_volume_halves_in_the_middle_and_recovers() {
        let (rs, rg) = start_and_goal().unwrap();
        let dets: Vec<f64> = BLEND_STEPS.iter().map(|&s| det(&blend(&rs, &rg, s))).collect();
        assert!((dets[0] - 1.0).abs() < 1e-12 && (dets[4] - 1.0).abs() < 1e-12);
        assert!((dets[2] - 0.5).abs() < 1e-12);
        assert!((dets[1] - dets[3]).abs() < 1e-12, "symmetric");
        assert!(dets[1] > dets[2] && dets[1] < dets[0]);
        // 1 - 2 s (1 - s) (1 - cos a), for a quarter turn
        for (&s, d) in BLEND_STEPS.iter().zip(&dets) {
            assert!((d - (1.0 - 2.0 * s * (1.0 - s))).abs() < 1e-12);
        }
        assert_eq!(
            BLEND_STEPS.map(|s| two_places(det(&blend(&rs, &rg, s)))),
            ["1.00", "0.63", "0.50", "0.63", "1.00"]
        );
        assert_eq!(fact("G-X4.blend").unwrap(), "66/66");
        // the circle's picture: two unit arrows a quarter turn apart average
        // to 0.71, short of the circle
        let mean = (0.5_f64.powi(2) + 0.5_f64.powi(2)).sqrt();
        assert_eq!(two_places(mean), "0.71");
        // and the repaired blend a quarter of the way is 18.4, the screw 22.5
        assert_eq!(format!("{:.1}", fact_f64("G-I.blend.quarter_turned").unwrap()), "18.4");
        assert_eq!(format!("{:.1}", fact_f64("G-I.screw.quarter_turned").unwrap()), "22.5");
    }

    /// 14_4: the screw is a quarter turn about the vertical and 34.8 cm
    /// down; half of it is 45 degrees and a true pose; and the arm's own
    /// poses along it do the same within a millimeter.
    #[test]
    fn the_screw_is_a_quarter_turn_and_thirty_five_centimeters_down() {
        let arm = Arm::iiwa14();
        let q0 = model::degrees(poses::START);
        let (start, goal) = (arm.tip(&q0), model::goal(&arm));
        let screw = model::screw_of(&goal.then(&start.undo()));
        assert!((screw.angle / DEG - 90.0).abs() < 1e-9);
        assert!(norm(sub(screw.axis, [0.0, 0.0, 1.0])) < 1e-9, "a vertical axis");
        assert_eq!(
            format!("{:.1}", fact_f64("G-I.screw.slide").unwrap().abs() * 100.0),
            "34.8"
        );
        assert!((screw.slide - fact_f64("G-I.screw.slide").unwrap()).abs() < 5e-5);
        let mid = model::along_screw(&screw, 0.5).then(&start);
        assert!((det(&mid.r) - 1.0).abs() < 1e-12, "a true pose at halfway");
        assert!((angle_axis(&mul(&transpose(&start.r), &mid.r)).0 / DEG - 45.0).abs() < 1e-9);
        assert_eq!(fact("G-I.screw.mid.turned").unwrap(), "45.0000");
        assert_eq!(fact("G-I.screw.mid.det").unwrap(), "1.0000");
        let keys = along_a_screw(&arm, &q0, &start, &screw, SCREW_SAMPLES);
        for (k, q) in keys.iter().enumerate() {
            let want = model::along_screw(&screw, k as f64 / SCREW_SAMPLES as f64).then(&start);
            assert!(norm(sub(arm.tip(q).p, want.p)) < 1e-3, "sample {k}");
        }
    }

    /// 14_5: the screw's helix and the straight line meet at both ends and
    /// are widest apart by the fact's seven point six centimeters.
    #[test]
    fn the_helix_and_the_line_part_by_seven_point_six_centimeters() {
        let arm = Arm::iiwa14();
        let (start, goal) = (arm.tip(&model::degrees(poses::START)), model::goal(&arm));
        let screw = model::screw_of(&goal.then(&start.undo()));
        let (helix, line) = two_paths(&start, &screw, goal.p, 100);
        assert!(norm(sub(helix[0], line[0])) < 1e-12);
        assert!(norm(sub(helix[100], line[100])) < 1e-9, "both end on the goal");
        let (at, gap) = widest_gap(&helix, &line);
        assert!((gap - fact_f64("G-I.bow").unwrap()).abs() < 5e-5, "{gap}");
        assert_eq!(format!("{:.1}", gap * 100.0), "7.6");
        assert!(at > 10 && at < 90, "widest in the middle, not at an end");
        // positions add: the straight line is the sum of start and a fraction
        // of the difference
        let third = add(start.p, scale(sub(goal.p, start.p), 1.0 / 3.0));
        assert!(norm(sub(third, line[33])) < 0.01 * norm(sub(goal.p, start.p)));
    }

    #[test]
    fn the_quiz_card_and_the_long_note_fit_the_frame() {
        run(|stage| {
            let kit = Kit::new()?;
            let card = QuizCard::build(&kit, stage, ASK, QuizLook::at(v(CARD_AT[0], CARD_AT[1]), CARD_SCALE))?;
            let right = stage.get_right(card.back)[0];
            for &row in &card.rows {
                assert!(stage.get_right(row)[0] < right - 0.15, "a row runs past the card");
            }
            assert!(right < 7.1);
            // the note sits right of its tag, inside the frame's margin
            let tagged = tag(&kit, stage, PROGRAMMERS, v(TAG_AT[0], TAG_AT[1]), 22.0)?;
            let note_left = stage.get_right(tagged)[0] + NOTE_GAP;
            let long = kit.text_left(
                stage,
                &format!("{RENORMALISE} {A_QUARTER_OF_THE_WAY} 18.4°, {NOT} 22.5°"),
                22.0,
                Palette::dim(),
                v(note_left, TAG_AT[1]),
            )?;
            assert!(
                stage.get_left(tagged)[0] > -7.11 + 0.15,
                "the tag starts inside the margin"
            );
            assert!(
                stage.get_right(long)[0] < 7.11 - 0.15,
                "the re-normalising note ends at {}",
                stage.get_right(long)[0]
            );
            Ok(())
        });
    }
}
