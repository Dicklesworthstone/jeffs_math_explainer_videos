//! Chapter 09, by hand: the arm's four real lengths (09_1); two joints a
//! quarter turn each, the elbow's screw first (09_2) and then the shoulder's
//! (09_3); where does joint one's quarter turn take the gripper (09_4,
//! answered in 09_5); and what the recipe never needed, where an axis is now
//! (09_6). The arm moves by its seven joint trackers, and the tip's numbers
//! are read off the moving arm every frame.

use std::f64::consts::FRAC_PI_2;

use explainer_kit::{Kit, Palette, Timed, play, tween, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::draw::{
    ARM_SCALE, ArmRig, Ask, Choice, PartSpec, Pin, QSource, QuizCard, QuizLook, Shape, View, World, fact_f64,
    fact_list, fade_in_all, fade_in_lagged, fail, greyed, leave, point_fn, say, still, table,
};
use crate::model::{self, Arm, DEG, ELBOW, N, Pose, SHOULDER, V3, add, poses, scale};
use crate::palette;

pub struct ByHand {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const TIP_HEAD: &str = "the gripper's tip, in centimeters";
const FORWARD: &str = "forward";
const SIDEWAYS: &str = "sideways";
const UP: &str = "up";
const TO_THE_SIDE: &str = "to the side";
const HIGHER: &str = "higher";
const NOWHERE: &str = "nowhere";
const WHERE_DOES_IT_GO: &str = "where does the gripper go?";
const NOW: &str = "now";
const AT_ZERO: &str = "at zero";
const SHOULDER_FIRST: [&str; 3] = ["shoulder first:", "then you must find", "where the elbow's axis went"];
const JOINT_ONE: &str = "joint 1";
const SHOULDER_NAME: &str = "shoulder";
const ELBOW_NAME: &str = "elbow";

/// The beats of the chapter, in order (storyboard ids).
const BEATS: [&str; 6] = ["09_1", "09_2", "09_3", "09_4", "09_5", "09_6"];

/// 09_4's quiz: the card's words, and the choices with what the computed
/// answer calls them (`G-Q.09_4.answer`).
pub const ASK: Ask = Ask {
    line: BEATS[3],
    question: WHERE_DOES_IT_GO,
    choices: &[
        Choice {
            key: "higher",
            label: HIGHER,
        },
        Choice {
            key: "side",
            label: TO_THE_SIDE,
        },
        Choice {
            key: "nowhere",
            label: NOWHERE,
        },
    ],
};

/// The angle of every bend in the chapter: a quarter turn.
const QUARTER: f64 = FRAC_PI_2;

/// The arm's place on the screen (the foot's), and the quiz card's: its top
/// sits under the readout's captions (`CAPTION_Y`), its bottom inside the
/// margin.
const ARM_AT: [f64; 2] = [-3.1, -2.55];
const CARD_AT: [f64; 2] = [4.45, -1.4];
const CARD_SCALE: f64 = 0.9;
/// Where each pinned label stands against its point: the joints' names (the
/// elbow's to its right, where the forearm never goes), and 09_6's "now" and
/// "at zero" (one right-below its axis, the other left-above).
const JOINT_SHIFT: [f64; 2] = [-0.95, 0.25];
const ELBOW_SHIFT: [f64; 2] = [0.95, 0.25];
const NOW_SHIFT: [f64; 2] = [0.8, -0.2];
const ZERO_SHIFT: [f64; 2] = [-0.95, 0.15];
/// The crossed-out road of 09_6: the left of its three lines and their size,
/// small enough that the longest ends inside the frame's margin.
const ALT_AT: [f64; 2] = [3.6, -1.2];
const ALT_PT: f64 = 26.0;
/// The readout's heading (centred) and the sum written out under it (left).
const HEAD_AT: [f64; 2] = [4.8, READ_Y + 0.72];
const SUM_AT: [f64; 2] = [2.15, 0.6];
/// The recipe's row, and its factors' gap.
const ROW_Y: f64 = 3.2;
const ROW_GAP: f64 = 0.55;
const ROW_PT: f64 = 44.0;
/// The tip readout: the right edge of each number's column, and its height.
const READ_X: [f64; 3] = [3.15, 4.8, 6.45];
const READ_Y: f64 = 1.85;
const READ_PT: f64 = 40.0;
/// How far the length brackets stand from the arm, and the axes' half
/// length (metres).
const BRACKET_OFF: f64 = 0.19;
const AXIS_REACH: f64 = 0.32;

/// The parts this chapter adds to the arm's own 23 and the table.
mod part {
    /// Four brackets: foot to shoulder, upper arm, forearm, wrist to flange.
    pub const BRACKETS: std::ops::Range<usize> = 24..28;
    /// Joint axes carried by the arm: joints 1, 2 and 4.
    pub const AXIS_BASE: usize = 28;
    pub const AXIS_SHOULDER: usize = 29;
    pub const AXIS_ELBOW: usize = 30;
    /// 09_6: the elbow's axis where it is now, and where the recipe used it.
    pub const ELBOW_NOW: usize = 31;
    pub const ELBOW_AT_ZERO: usize = 32;
}

/// The four lengths the arm is made of, foot to flange, as the heights at
/// which each begins and ends at home (metres), from the facts.
fn lengths() -> fmn::Result<[(f64, f64); 4]> {
    let shoulder = fact_f64("G-A.shoulder_height")?;
    let upper = fact_f64("G-A.upper_arm")?;
    let forearm = fact_f64("G-A.forearm")?;
    let tip = fact_f64("G-A.wrist_to_tip")?;
    let (a, b, c) = (shoulder, shoulder + upper, shoulder + upper + forearm);
    Ok([(0.0, a), (a, b), (b, c), (c, c + tip)])
}

/// Centimeters as the video writes them: whole when whole, one place
/// otherwise (36, 42, 40, 12.6).
fn cm(metres: f64) -> String {
    let c = metres * 100.0;
    if (c - c.round()).abs() < 1e-6 {
        format!("{}", c.round() as i64)
    } else {
        format!("{c:.1}")
    }
}

/// A tip coordinate for the readout, in centimeters at one place, never a
/// negative zero.
fn readout_cm(metres: f64) -> f64 {
    ((metres * 1000.0).round() / 10.0) + 0.0
}

/// The seven joint trackers of a rig that holds its joints.
fn trackers(rig: &ArmRig) -> fmn::Result<[Mob; N]> {
    match &rig.source {
        QSource::Joints(t) => Ok(*t),
        QSource::Path { .. } => Err(fail("by hand: the rig walks a path, not seven joints".into())),
    }
}

/// An axis as the world draws it: a line through `point` along `dir`,
/// `reach` each way.
fn axis_line(point: V3, dir: V3, reach: f64, color: Srgb, width: f64) -> Shape {
    Shape::line(
        add(point, scale(dir, -reach)),
        add(point, scale(dir, reach)),
        color,
        width,
    )
}

/// Animate a mobject's colour (a factor glowing, then resting again).
fn tint(mob: Mob, color: Srgb, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .set_color(color)?)
}

/// Shift a mobject sideways, `rt` seconds.
fn slide(mob: Mob, dx: f64, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .shift([dx, 0.0, 0.0])?)
}

/// One factor of the recipe: `e^{[S_j] 90°}` for joint `j` (from 1).
fn factor_tex(joint: usize) -> String {
    format!(r"e^{{[S_{joint}]\,{}^\circ}}", (QUARTER / DEG).round() as i64)
}

impl SceneConstruct for ByHand {
    fn name(&self) -> &str {
        "09_by_hand"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let arm = Arm::iiwa14();
        let rig = ArmRig::joints(stage, arm.clone(), poses::HOME);
        let q = trackers(&rig)?;
        let gold = palette::screw();

        // ---- the world: the arm, a table, the brackets, three axes, and
        // the elbow's axis now and at zero
        let mut specs = rig.parts(palette::steel());
        specs.push(PartSpec::new(table([-0.7, -0.7], [0.9, 0.9], 0.0, 0.2), still(Pose::NOTHING)).layer(-2));
        // the table is part 23; the brackets follow it
        let (right, _, _) = View::basis();
        let off = scale(right, BRACKET_OFF);
        let tick = scale(off, -0.55);
        let dim = Palette::dim();
        for (z0, z1) in lengths()? {
            let (a, b) = (add([0.0, 0.0, z0], off), add([0.0, 0.0, z1], off));
            let shape = Shape::Path {
                pts: vec![add(a, tick), a, b, add(b, tick)],
                color: dim,
                width: 3.0,
                closed: false,
                bias: 0.0,
            };
            specs.push(PartSpec::new(vec![shape], still(Pose::NOTHING)).layer(1));
        }
        for joint in [0, SHOULDER, ELBOW] {
            specs.push(
                PartSpec::new(
                    vec![axis_line(arm.origin[joint], arm.axis[joint], AXIS_REACH, gold, 3.5)],
                    rig.carried(joint),
                )
                .layer(1),
            );
        }
        specs.push(
            PartSpec::new(
                vec![axis_line(arm.origin[ELBOW], arm.axis[ELBOW], AXIS_REACH, greyed(), 4.5)],
                rig.carried(ELBOW),
            )
            .layer(1),
        );
        specs.push(
            PartSpec::new(
                vec![Shape::Dashed {
                    a: add(arm.origin[ELBOW], scale(arm.axis[ELBOW], -AXIS_REACH)),
                    b: add(arm.origin[ELBOW], scale(arm.axis[ELBOW], AXIS_REACH)),
                    color: gold,
                    width: 3.5,
                    dashes: 9,
                }],
                still(Pose::NOTHING),
            )
            .layer(1),
        );

        // ---- labels that follow the arm: the lengths, and the joints in turn
        let segs = lengths()?;
        let mut pins = Vec::new();
        let mut length_labels = Vec::new();
        for &(z0, z1) in &segs {
            let words = format!("{} cm", cm(z1 - z0));
            let m = kit.text(stage, &words, 30.0, Palette::ink(), v(0.0, 0.0))?;
            let mid = [off[0], off[1], (z0 + z1) / 2.0];
            pins.push(Pin {
                mob: m,
                at: point_fn(move |_: &Arena| mid),
                shift: v(0.72, 0.0),
            });
            length_labels.push(m);
        }
        let mut joint_labels = Vec::new();
        for (joint, words, shift) in [
            (0, JOINT_ONE, JOINT_SHIFT),
            (SHOULDER, SHOULDER_NAME, JOINT_SHIFT),
            (ELBOW, ELBOW_NAME, ELBOW_SHIFT),
        ] {
            let m = kit.text(stage, words, 28.0, gold, v(0.0, 0.0))?;
            let (at, _) = rig.axis_now(joint);
            pins.push(Pin {
                mob: m,
                at,
                shift: v(shift[0], shift[1]),
            });
            joint_labels.push(m);
        }
        let view = View::new(ARM_SCALE, v(ARM_AT[0], ARM_AT[1]));
        let world = World::build(stage, view, 1.4, 0.25, specs, pins)?;
        let hidden: Vec<usize> = ArmRig::GOLD_COLLARS
            .chain(part::BRACKETS)
            .chain([
                part::AXIS_BASE,
                part::AXIS_SHOULDER,
                part::AXIS_ELBOW,
                part::ELBOW_NOW,
                part::ELBOW_AT_ZERO,
            ])
            .collect();
        for i in hidden {
            stage.set_tracker_value(world.parts[i].shown, 0.0)?;
        }
        world.add_to_scene(stage)?;
        let collar = |joint: usize| world.parts[ArmRig::GOLD_COLLARS.start + joint].shown;

        // ---- the recipe's row: factors side by side (the base joint's slot
        // empty until 09_5)
        let src = [
            factor_tex(1),
            factor_tex(SHOULDER + 1),
            factor_tex(ELBOW + 1),
            "M".to_owned(),
        ];
        let mut factors = Vec::new();
        for s in &src {
            factors.push(kit.tex(stage, s, ROW_PT, Palette::ink(), &[], v(0.0, ROW_Y))?);
        }
        let widths: Vec<f64> = factors.iter().map(|&f| stage.get_width(f)).collect();
        let total: f64 = widths.iter().sum::<f64>() + ROW_GAP * (widths.len() as f64 - 1.0);
        let mut at = -total / 2.0 + 1.2;
        for (&f, w) in factors.iter().zip(&widths) {
            stage.move_to(f, v(at + w / 2.0, ROW_Y), ORIGIN);
            at += w + ROW_GAP;
        }
        let [f1, f2, f4, m] = [factors[0], factors[1], factors[2], factors[3]];
        // while the base joint's factor is absent the other three sit centred
        let held = (widths[0] + ROW_GAP) / 2.0;
        for &f in &[f2, f4, m] {
            stage.shift(f, v(-held, 0.0));
        }

        // 09_1: the arm with its four real lengths
        let line = say(&kit, stage, BEATS[0])?;
        world.show(stage, 0.8)?;
        // the tip readout: three numbers read off the moving arm every
        // frame (a live number redraws at full opacity, so it comes in whole)
        let head = kit.text(stage, TIP_HEAD, 28.0, Palette::dim(), v(HEAD_AT[0], HEAD_AT[1]))?;
        let mut captions = Vec::new();
        for (x, word) in READ_X.iter().zip([FORWARD, SIDEWAYS, UP]) {
            captions.push(kit.text(stage, word, 26.0, Palette::dim(), v(*x - 0.62, READ_Y - 0.58))?);
        }
        fade_in_all(stage, &[head], 0.5)?;
        for (k, x) in READ_X.iter().enumerate() {
            let probe = rig.clone();
            kit.live_value(
                stage,
                move |s: &Arena| readout_cm(probe.arm.tip(&probe.q(s)).p[k]),
                1,
                false,
                READ_PT,
                Palette::ink(),
                v(*x, READ_Y),
                RIGHT,
            )?;
        }
        fade_in_all(stage, &captions, 0.4)?;
        line.wait_for(stage, "with the real arm's lengths")?;
        let braces: Vec<Mob> = part::BRACKETS.map(|i| world.parts[i].shown).collect();
        for (&b, &label) in braces.iter().zip(&length_labels) {
            play!(stage; tween(b, 1.0, 0.45)?, fade_in(stage.arena_mut(), label, ORIGIN, 1.0)?.rt(0.45));
        }

        // 09_2: elbow first
        let line = say(&kit, stage, BEATS[1])?;
        line.wait_for(stage, "Bend two joints")?;
        play!(stage;
            tween(braces[0], 0.0, 0.5)?,
            tween(braces[1], 0.0, 0.5)?,
            tween(braces[2], 0.0, 0.5)?,
            tween(braces[3], 0.0, 0.5)?,
        );
        let gone: Vec<Mob> = length_labels.clone();
        leave(stage, &gone, &[], 0.4)?;
        let row = [f2, f4, m];
        fade_in_lagged(stage, &row, 1.6, 0.4)?;
        line.wait_for(stage, "The recipe says")?;
        let elbow_label = joint_labels[2];
        play!(stage;
            tint(f4, gold, 0.5)?,
            tween(collar(ELBOW), 1.0, 0.5)?,
            tween(world.parts[part::AXIS_ELBOW].shown, 1.0, 0.5)?,
            fade_in(stage.arena_mut(), elbow_label, ORIGIN, 1.0)?.rt(0.5),
        );
        line.wait_for(stage, "the screw at the elbow swings")?;
        play!(stage; tween(q[ELBOW], QUARTER, 2.6)?);

        // 09_3: then the shoulder's, about the axis where it stood at zero
        let line = say(&kit, stage, BEATS[2])?;
        let shoulder_label = joint_labels[1];
        play!(stage;
            tint(f4, Palette::ink(), 0.4)?,
            tint(f2, gold, 0.5)?,
            tween(collar(ELBOW), 0.0, 0.4)?,
            tween(world.parts[part::AXIS_ELBOW].shown, 0.0, 0.4)?,
            tween(collar(SHOULDER), 1.0, 0.5)?,
            tween(world.parts[part::AXIS_SHOULDER].shown, 1.0, 0.5)?,
            fade_in(stage.arena_mut(), shoulder_label, ORIGIN, 1.0)?.rt(0.5),
        );
        line.wait_for(stage, "swings everything above it")?;
        play!(stage; tween(q[SHOULDER], QUARTER, 4.2)?);
        line.wait_for(stage, "and the forearm points straight up")?;
        let p = fact_list("G-K.ell.p")?;
        let (shoulder_h, forearm, flange) = (
            fact_f64("G-A.shoulder_height")?,
            fact_f64("G-A.forearm")?,
            fact_f64("G-A.wrist_to_tip")?,
        );
        let sum_forward = format!("{} {FORWARD}", cm(p[0]));
        let sum_up = format!(
            "{} + {} + {} = {} {UP}",
            cm(shoulder_h),
            cm(forearm),
            cm(flange),
            cm(p[2])
        );
        let s1 = kit.text_left(stage, &sum_forward, 30.0, Palette::ink(), v(SUM_AT[0], SUM_AT[1]))?;
        let s2 = kit.text_left(stage, &sum_up, 30.0, Palette::ink(), v(SUM_AT[0], SUM_AT[1] - 0.45))?;
        fade_in_lagged(stage, &[s1, s2], 1.0, 0.6)?;

        // 09_4: joint one, at the base: where does the gripper go?
        let line = say(&kit, stage, BEATS[3])?;
        let card = QuizCard::build(&kit, stage, ASK, QuizLook::at(v(CARD_AT[0], CARD_AT[1]), CARD_SCALE))?;
        let base_label = joint_labels[0];
        play!(stage;
            tint(f2, Palette::ink(), 0.4)?,
            tween(collar(SHOULDER), 0.0, 0.4)?,
            tween(world.parts[part::AXIS_SHOULDER].shown, 0.0, 0.4)?,
            tween(collar(0), 1.0, 0.5)?,
            tween(world.parts[part::AXIS_BASE].shown, 1.0, 0.5)?,
            fade_in(stage.arena_mut(), base_label, ORIGIN, 1.0)?.rt(0.5),
        );
        leave(stage, &[s1, s2, shoulder_label, elbow_label], &[], 0.4)?;
        line.wait_last(stage)?;
        card.show(stage, 0.5)?;
        card.count(&kit, stage)?;

        // 09_5: round to the side, at the same height
        let line = say(&kit, stage, BEATS[4])?;
        play!(stage; tween(q[0], QUARTER, 3.0)?);
        line.wait_for(stage, "Now it is")?;
        play!(stage;
            slide(f2, held, 0.6)?,
            slide(f4, held, 0.6)?,
            slide(m, held, 0.6)?,
            tint(f1, gold, 0.6)?,
            fade_in(stage.arena_mut(), f1, ORIGIN, 1.0)?.rt(0.6),
        );
        line.wait_end(stage)?;
        play!(stage; tint(f1, Palette::ink(), 0.5)?);

        // 09_6: the recipe never asked where the elbow's axis is now
        let line = say(&kit, stage, BEATS[5])?;
        let mut leaving = card.mobs();
        leaving.push(base_label);
        leave(stage, &leaving, &[collar(0), world.parts[part::AXIS_BASE].shown], 0.5)?;
        line.wait_for(stage, "where the elbow's axis is now")?;
        // the axis is drawn grey; its word is the ordinary dim, which reads
        let now = kit.text(stage, NOW, 30.0, Palette::dim(), v(0.0, 0.0))?;
        let (now_pt, _) = arm.joint(&model::degrees(poses::QUARTER), ELBOW);
        let now_pos = view.project(now_pt);
        stage.move_to(now, v(now_pos[0] + NOW_SHIFT[0], now_pos[1] + NOW_SHIFT[1]), ORIGIN);
        play!(stage;
            tween(world.parts[part::ELBOW_NOW].shown, 1.0, 0.6)?,
            fade_in(stage.arena_mut(), now, ORIGIN, 1.0)?.rt(0.6),
        );
        line.wait_for(stage, "Each screw is written down once")?;
        let zero = kit.text(stage, AT_ZERO, 30.0, gold, v(0.0, 0.0))?;
        let zero_pos = view.project(arm.origin[ELBOW]);
        stage.move_to(
            zero,
            v(zero_pos[0] + ZERO_SHIFT[0], zero_pos[1] + ZERO_SHIFT[1]),
            ORIGIN,
        );
        play!(stage;
            tween(world.parts[part::ELBOW_AT_ZERO].shown, 1.0, 0.6)?,
            fade_in(stage.arena_mut(), zero, ORIGIN, 1.0)?.rt(0.6),
        );
        // the road not taken, crossed out
        let mut alt = Vec::new();
        for (i, words) in SHOULDER_FIRST.iter().enumerate() {
            alt.push(kit.text_left(
                stage,
                words,
                ALT_PT,
                Palette::dim(),
                v(ALT_AT[0], ALT_AT[1] - 0.45 * i as f64),
            )?);
        }
        let (a, b) = (v(3.15, -1.0), v(3.45, -1.7));
        let cross = [
            polyline(vec![a, b])
                .style(super::draw::line_style(palette::warn(), 6.0))
                .build(),
            polyline(vec![v(a[0], b[1]), v(b[0], a[1])])
                .style(super::draw::line_style(palette::warn(), 6.0))
                .build(),
        ];
        for c in cross {
            alt.push(stage.arena_mut().add(c));
        }
        line.wait_end(stage)?;
        fade_in_all(stage, &alt, 0.6)?;
        kit.hold(stage)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{exp6, norm, sub};

    /// A scene run headless, for the checks that need real text and real
    /// mobjects.
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

    /// Chapter 09 plays exactly the storyboard's beats, in its order, and
    /// every one is the script's own line.
    #[test]
    fn the_beats_are_the_storyboards_and_the_scripts() {
        let board = explainer_kit::storyboard::parse(crate::STORYBOARD).unwrap();
        let chapter = board.iter().find(|c| c.key == "09_by_hand").expect("chapter 09");
        let ids: Vec<&str> = chapter.beats.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(ids, BEATS);
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for beat in &chapter.beats {
            let words = beat.spoken().expect("every beat of chapter 09 is spoken");
            assert_eq!(script.text(&beat.id), Some(words.as_str()), "{}", beat.id);
        }
    }

    /// Every phrase the scene waits for is in its line (a missing phrase is
    /// a scene error at render time, so it is caught here).
    #[test]
    fn every_cue_phrase_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("09_1", "with the real arm's lengths"),
            ("09_2", "Bend two joints"),
            ("09_2", "The recipe says"),
            ("09_2", "the screw at the elbow swings"),
            ("09_3", "swings everything above it"),
            ("09_3", "and the forearm points straight up"),
            ("09_5", "Now it is"),
            ("09_6", "where the elbow's axis is now"),
            ("09_6", "Each screw is written down once"),
        ] {
            let text = script.text(id).unwrap_or_default();
            assert!(text.contains(phrase), "{id} does not say {phrase:?}");
        }
    }

    /// 09_1: the four lengths are the facts', they follow one another from
    /// the foot to the flange, and together they are the arm's height.
    #[test]
    fn the_four_lengths_add_up_to_the_arms_height() {
        let l = lengths().unwrap();
        let said: Vec<String> = l.iter().map(|(a, b)| cm(b - a)).collect();
        assert_eq!(said, ["36", "42", "40", "12.6"]);
        for w in l.windows(2) {
            assert!((w[0].1 - w[1].0).abs() < 1e-12, "a length starts where the last ended");
        }
        assert_eq!(l[0].0, 0.0);
        let height = fact_f64("G-A.height").unwrap();
        assert!(
            (l[3].1 - height).abs() < 1e-12,
            "the last length ends at the arm's height"
        );
        assert_eq!(cm(height), "130.6");
        // and the first sits on the drawn arm: the brackets stand beside it
        let arm = Arm::iiwa14();
        assert!((arm.home.p[2] - height).abs() < 1e-12);
    }

    /// The readout never shows a negative zero and rounds to one place.
    #[test]
    fn the_readout_rounds_to_a_place_and_never_shows_minus_zero() {
        assert_eq!(readout_cm(-1.0e-17), 0.0);
        assert!(readout_cm(-1.0e-17).is_sign_positive());
        assert_eq!(readout_cm(0.886), 88.6);
        assert_eq!(readout_cm(-0.526), -52.6);
        assert_eq!(readout_cm(1.306), 130.6);
    }

    /// The tip's numbers at each stop of the story are the pinned facts':
    /// home, the elbow bent, the shoulder bent too, joint one turned.
    #[test]
    fn the_tip_lands_where_the_facts_pinned_it() {
        let arm = Arm::iiwa14();
        let tip = |q: [f64; N]| arm.tip(&model::degrees(q)).p;
        let want = |key: &str| -> V3 {
            let l = fact_list(key).unwrap();
            [l[0], l[1], l[2]]
        };
        assert!(close(tip(poses::HOME), want("G-A.home.p")));
        assert!(close(tip(poses::ELBOW_ONLY), want("G-K.ell.after_elbow")));
        assert!(close(tip(poses::ELL), want("G-K.ell.p")));
        assert!(close(tip(poses::QUARTER), want("G-K.quarter.p")));
        // and as the readout will print them
        let say = |p: V3| p.map(readout_cm);
        assert_eq!(say(tip(poses::HOME)), [0.0, 0.0, 130.6]);
        assert_eq!(say(tip(poses::ELBOW_ONLY)), [-52.6, 0.0, 78.0]);
        assert_eq!(say(tip(poses::ELL)), [42.0, 0.0, 88.6]);
        assert_eq!(say(tip(poses::QUARTER)), [0.0, 42.0, 88.6]);
    }

    /// The scene bends joint 4, then joint 2, then joint 1, a quarter turn
    /// each: those stops are the named poses.
    #[test]
    fn the_scene_walks_the_named_poses_in_order() {
        let mut q = [0.0; N];
        q[ELBOW] = QUARTER;
        assert_eq!(q.map(|x| (x / DEG).round()), poses::ELBOW_ONLY);
        q[SHOULDER] = QUARTER;
        assert_eq!(q.map(|x| (x / DEG).round()), poses::ELL);
        q[0] = QUARTER;
        assert_eq!(q.map(|x| (x / DEG).round()), poses::QUARTER);
    }

    /// The recipe: each screw is written down once, with the arm at zero,
    /// and the factors multiply in the order the sums run. Done by hand from
    /// the home screws it lands where the arm does; done shoulder first it
    /// does not (the crossed-out road of 09_6).
    #[test]
    fn the_recipe_with_home_screws_lands_on_the_arm_and_the_wrong_order_does_not() {
        let arm = Arm::iiwa14();
        let turned = |joint: usize| exp6(arm.screw(joint).map(|x| x * QUARTER));
        let recipe = turned(0).then(&turned(SHOULDER)).then(&turned(ELBOW)).then(&arm.home);
        let q = model::degrees(poses::QUARTER);
        assert!(close(recipe.p, arm.tip(&q).p));
        assert!(model::gap(&recipe.r, &arm.tip(&q).r) < 1e-9);
        let want = fact_list("G-K.quarter.p").unwrap();
        assert!(close(recipe.p, [want[0], want[1], want[2]]));
        // the other way round, with the same home screws
        let wrong = turned(0).then(&turned(ELBOW)).then(&turned(SHOULDER)).then(&arm.home);
        assert!(
            norm(sub(wrong.p, recipe.p)) > 0.1,
            "shoulder first with home screws ends {:.3} m from the arm",
            norm(sub(wrong.p, recipe.p))
        );
    }

    /// 09_4's quiz: four seconds, and the right choice is the one the model
    /// computes: a quarter turn of joint one keeps the height and moves the
    /// tip round to the side, by the whole of its distance from the base.
    #[test]
    fn the_quiz_answer_is_what_joint_one_does() {
        assert_eq!(ASK.seconds().unwrap(), 4);
        assert_eq!(ASK.choices[ASK.answer_index().unwrap()].label, TO_THE_SIDE);
        let arm = Arm::iiwa14();
        let before = arm.tip(&model::degrees(poses::ELL)).p;
        let after = arm.tip(&model::degrees(poses::QUARTER)).p;
        assert!((after[2] - before[2]).abs() < 1e-12, "not higher");
        assert!(norm(sub(after, before)) > 0.5, "not nowhere");
        assert!(
            after[0].abs() < 1e-12 && (after[1] - before[0]).abs() < 1e-12,
            "round to the side"
        );
        // 09_5: forty-two in front, then forty-two to the side
        assert_eq!(cm(before[0]), cm(after[1]));
        assert_eq!(cm(before[0]), "42");
    }

    /// 09_6: the elbow's axis is not where the recipe used it once the
    /// other joints have turned, and is exactly there at zero.
    #[test]
    fn the_elbows_axis_has_moved_and_the_recipe_never_asked() {
        let arm = Arm::iiwa14();
        let (home_at, home_dir) = arm.joint(&[0.0; N], ELBOW);
        assert!(close(home_at, arm.origin[ELBOW]) && close(home_dir, arm.axis[ELBOW]));
        let (now_at, now_dir) = arm.joint(&model::degrees(poses::QUARTER), ELBOW);
        assert!(norm(sub(now_at, home_at)) > 0.3, "the elbow has travelled");
        assert!(norm(sub(now_dir, home_dir)) > 0.5, "its axis points elsewhere");
    }

    /// The glow marks the joint the recipe is working on, and the picture's
    /// axes are the arm's own: through the origin, along the axis, carried.
    #[test]
    fn the_axes_drawn_are_the_joints_own() {
        let arm = Arm::iiwa14();
        for joint in [0, SHOULDER, ELBOW] {
            let Shape::Path { pts, .. } =
                axis_line(arm.origin[joint], arm.axis[joint], AXIS_REACH, Palette::ink(), 3.0)
            else {
                panic!("an axis is a path");
            };
            let mid = scale(add(pts[0], pts[1]), 0.5);
            assert!(close(mid, arm.origin[joint]));
            let along = model::unit(sub(pts[1], pts[0]));
            assert!(close(along, arm.axis[joint]));
            assert!((norm(sub(pts[1], pts[0])) - 2.0 * AXIS_REACH).abs() < 1e-12);
        }
        // the part table of this chapter follows the arm's 23 and the table
        assert_eq!(part::BRACKETS.start, 23 + 1);
    }

    /// The quiz card's rows fit inside the card, with real text.
    #[test]
    fn the_quiz_rows_fit_the_card() {
        run(|stage| {
            let kit = Kit::new()?;
            let card = QuizCard::build(&kit, stage, ASK, QuizLook::at(v(CARD_AT[0], CARD_AT[1]), CARD_SCALE))?;
            let (left, right) = (stage.get_left(card.back)[0], stage.get_right(card.back)[0]);
            for &row in &card.rows {
                assert!(
                    stage.get_right(row)[0] < right - 0.15,
                    "a row runs past the card's edge"
                );
                assert!(stage.get_left(row)[0] > left, "a row starts outside the card");
            }
            assert!(right < 7.1, "the card is inside the frame: {right}");
            // the tick sits at the right of the answer's row, clear of its words
            let tick = card.verdict[1];
            let answer = card.rows[ASK.answer_index().unwrap()];
            assert!(
                stage.get_right(answer)[0] < stage.get_left(tick)[0] - 0.1,
                "the tick runs into the answer's words"
            );
            // the card stands under the readout's captions and inside the margin
            assert!(
                stage.get_top(card.back)[1] < READ_Y - 0.58 - 0.2,
                "the card's top reaches the readout's captions"
            );
            assert!(stage.get_bottom(card.back)[1] > -3.85, "the card runs off the bottom");
            Ok(())
        });
    }

    /// Where the arm's links are on the screen at joint angles `q` (degrees):
    /// a dozen points along each link, from the base to the tip.
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

    /// The joints' names stand clear of the arm everywhere they are on show:
    /// the elbow's through its swing (09_2) and the shoulder's (09_3), the
    /// shoulder's with it, and joint one's through its quarter turn (09_5).
    #[test]
    fn the_joint_names_stand_clear_of_the_arm() {
        run(|stage| {
            let kit = Kit::new()?;
            let arm = Arm::iiwa14();
            let view = View::new(ARM_SCALE, v(ARM_AT[0], ARM_AT[1]));
            let at = |angles: [(usize, f64); 3]| {
                let mut q = [0.0; N];
                for (joint, deg) in angles {
                    q[joint] = deg;
                }
                q
            };
            let swing = |k: usize| f64::from(k as u32) / 4.0 * 90.0;
            // (the name, its joint, its shift, the poses it is on show at)
            type Case = (&'static str, usize, [f64; 2], Vec<[f64; N]>);
            let mut cases: Vec<Case> = Vec::new();
            let mut elbow_poses = Vec::new();
            let mut shoulder_poses = Vec::new();
            let mut base_poses = Vec::new();
            for k in 0..=4 {
                elbow_poses.push(at([(ELBOW, swing(k)), (SHOULDER, 0.0), (0, 0.0)]));
                shoulder_poses.push(at([(SHOULDER, swing(k)), (ELBOW, 90.0), (0, 0.0)]));
                base_poses.push(at([(0, swing(k)), (SHOULDER, 90.0), (ELBOW, 90.0)]));
            }
            cases.push((
                ELBOW_NAME,
                ELBOW,
                ELBOW_SHIFT,
                [elbow_poses.clone(), shoulder_poses.clone()].concat(),
            ));
            cases.push((SHOULDER_NAME, SHOULDER, JOINT_SHIFT, shoulder_poses));
            cases.push((JOINT_ONE, 0, JOINT_SHIFT, base_poses));
            for (name, joint, shift, poses_shown) in cases {
                let label = kit.text(stage, name, 28.0, Palette::dim(), v(0.0, 0.0))?;
                let (half_w, half_h) = (stage.get_width(label) / 2.0, stage.get_height(label) / 2.0);
                for q in poses_shown {
                    let here = view.project(arm.joint(&model::degrees(q), joint).0);
                    let centre = [here[0] + shift[0], here[1] + shift[1]];
                    for p in links_on_screen(&arm, &view, q) {
                        let inside = (p[0] - centre[0]).abs() < half_w + 0.1 && (p[1] - centre[1]).abs() < half_h + 0.1;
                        assert!(!inside, "{name:?} sits on the arm at {q:?}: link point {p:?}");
                    }
                }
            }
            Ok(())
        });
    }

    /// 09_6's "now" and "at zero" stand apart.
    #[test]
    fn now_and_at_zero_do_not_touch() {
        run(|stage| {
            let kit = Kit::new()?;
            let arm = Arm::iiwa14();
            let view = View::new(ARM_SCALE, v(ARM_AT[0], ARM_AT[1]));
            let now_pos = view.project(arm.joint(&model::degrees(poses::QUARTER), ELBOW).0);
            let zero_pos = view.project(arm.origin[ELBOW]);
            let now = kit.text(
                stage,
                NOW,
                30.0,
                Palette::dim(),
                v(now_pos[0] + NOW_SHIFT[0], now_pos[1] + NOW_SHIFT[1]),
            )?;
            let zero = kit.text(
                stage,
                AT_ZERO,
                30.0,
                palette::screw(),
                v(zero_pos[0] + ZERO_SHIFT[0], zero_pos[1] + ZERO_SHIFT[1]),
            )?;
            let boxed = |m: Mob| {
                (
                    stage.get_left(m)[0],
                    stage.get_right(m)[0],
                    stage.get_bottom(m)[1],
                    stage.get_top(m)[1],
                )
            };
            let (a, b) = (boxed(now), boxed(zero));
            let apart = a.1 + 0.05 < b.0 || b.1 + 0.05 < a.0 || a.3 + 0.05 < b.2 || b.3 + 0.05 < a.2;
            assert!(apart, "\"now\" {a:?} touches \"at zero\" {b:?}");
            assert!(a.1 < 6.96 && b.0 > -6.96);
            Ok(())
        });
    }

    /// The recipe's row fits across the frame with all four factors.
    #[test]
    fn the_recipe_row_fits_the_frame() {
        run(|stage| {
            let kit = Kit::new()?;
            let mut width = 0.0;
            for s in [
                factor_tex(1),
                factor_tex(SHOULDER + 1),
                factor_tex(ELBOW + 1),
                "M".to_owned(),
            ] {
                let f = kit.tex(stage, &s, ROW_PT, Palette::ink(), &[], v(0.0, ROW_Y))?;
                width += stage.get_width(f) + ROW_GAP;
            }
            assert!(width - ROW_GAP < 13.6, "the row is {width:.2} wide");
            Ok(())
        });
    }

    /// The long words stay inside the frame's margin (the lint's 0.15 inside
    /// the 7.11 half width), measured with the real face where the scene
    /// puts them: the readout's heading, the sum written out, and the three
    /// lines of the road not taken.
    #[test]
    fn the_long_words_end_inside_the_margin() {
        const EDGE: f64 = 7.11 - 0.15;
        run(|stage| {
            let kit = Kit::new()?;
            let up = format!("36 + 40 + 12.6 = 88.6 {UP}");
            let head = kit.text(stage, TIP_HEAD, 28.0, Palette::dim(), v(HEAD_AT[0], HEAD_AT[1]))?;
            let sum = kit.text_left(stage, &up, 30.0, Palette::ink(), v(SUM_AT[0], SUM_AT[1] - 0.45))?;
            assert!(
                stage.get_right(head)[0] < EDGE,
                "the heading ends at {}",
                stage.get_right(head)[0]
            );
            assert!(stage.get_right(sum)[0] < EDGE);
            for (i, words) in SHOULDER_FIRST.iter().enumerate() {
                let m = kit.text_left(
                    stage,
                    words,
                    ALT_PT,
                    Palette::dim(),
                    v(ALT_AT[0], ALT_AT[1] - 0.45 * i as f64),
                )?;
                assert!(
                    stage.get_right(m)[0] < EDGE,
                    "{words:?} ends at {}",
                    stage.get_right(m)[0]
                );
            }
            Ok(())
        });
    }
}
