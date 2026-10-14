//! Chapter 08, seven screws: the arm stands up, 1.3 m tall, seven hinges
//! (08_1); each is a screw with no slide (08_2); turn only the elbow and what
//! moves is everything past it (08_3, 08_4); the recipe applies the last
//! joint's screw first, each axis still where it stood at zero (08_5); and the
//! gripper's pose is a product of seven exponentials (08_6).

use explainer_kit::props::{CodePanel, lower_third};
use explainer_kit::{Kit, Palette, Timed, play, tween, v};
use fmn::prelude::*;

use super::draw::{
    ARM_SCALE, ArmRig, Ask, Choice, PartSpec, Pin, QSource, QuizCard, QuizLook, Shape, View, World, fact, fact_f64,
    fact_list, fade_in_all, fade_in_lagged, fail, leave, line_style, say, screw_shapes, still, table, tag, tween_all,
};
use crate::model::{self, Arm, DEG, ELBOW, N, Pose, Screw, V3, poses};
use crate::palette;

pub struct SevenScrews {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const SHOULDER: &str = "shoulder";
const ELBOW_WORD: &str = "elbow";
const WRIST: &str = "wrist";
const M_WORD: &str = "M";
const ABOUT_A_LINE: &str = "turning about a line through c: x goes to c + R(x − c)";
const ORDER_OF_SUMS: &str = "the order of the sums, not of the motion";
const PRODUCT_LABEL: &str = "the product of exponentials ·";
/// The line the video has been earning, term by term (TeX), left to right.
const TERMS: [&str; 6] = [
    r"T(q) =",
    r"e^{[S_1]q_1}",
    r"e^{[S_2]q_2}",
    r"\cdots",
    r"e^{[S_7]q_7}",
    r"M",
];
/// Plain labels under the line's parts (`None` where there is none).
const TERM_LABELS: [&str; 6] = [
    "the gripper's pose",
    "joint 1's twist,\nheld for its angle",
    "",
    "…",
    "",
    "the pose at zero",
];

/// The beats of the chapter, in order (storyboard ids).
const BEATS: [&str; 6] = ["08_1", "08_2", "08_3", "08_4", "08_5", "08_6"];

/// 08_3's quiz: only joint four turns: which parts move? The keys are what
/// `G-Q.08_3.answer` prints (`past`) and two wrong readings.
pub const ASK: Ask = Ask {
    line: BEATS[2],
    question: "only the elbow turns:\nwhat moves?",
    choices: &[
        Choice {
            key: "all",
            label: "everything",
        },
        Choice {
            key: "past",
            label: "only what is past the elbow",
        },
        Choice {
            key: "forearm",
            label: "only the forearm",
        },
    ],
};

/// The joint angle every joint is turned to, one after another, in 08_5 and
/// held in 08_6 (degrees): any pose would do for the recipe.
const RECIPE_DEG: f64 = 40.0;
/// Where the arm is drawn: the foot's screen position.
const ARM_AT: [f64; 2] = [-3.7, -2.95];

/// The part order after the arm's 23: its seven axis lines, its seven
/// screws, the gripper's home pose, and the floor.
const AXES_AT: usize = 23;
const SCREWS_AT: usize = AXES_AT + N;
const HOME_AT: usize = SCREWS_AT + N;

/// An axis direction as a code line, from its fact (`0,-1,0`): `(0, -1, 0)`.
/// The panel is set in the typewriter face, where a code minus is the ASCII
/// hyphen.
fn direction_text(fact_value: &str) -> String {
    format!("({})", fact_value.replace(',', ", "))
}

/// The seven code lines of the model as data (08_1's panel): each joint's
/// axis and its height at zero, then the gripper's pose M.
fn model_lines() -> fmn::Result<Vec<String>> {
    let mut lines = Vec::new();
    for i in 1..=N {
        let axis = fact(&format!("G-A.axis.{i}"))?;
        let z = fact_f64(&format!("G-A.axis_height.{i}"))?;
        lines.push(format!("S{i}: axis {} at z = {z:.4}", direction_text(axis)));
    }
    let p = fact_list("G-A.home.p")?;
    lines.push(format!("M: ({:.1}, {:.1}, {:.4}), R = I", p[0], p[1], p[2]));
    Ok(lines)
}

/// A pair of small arrows' worth of frame at `at` (the gripper's pose M at
/// zero), three short arrows along x, y and z.
fn frame_shapes(at: V3, len: f64, color: Srgb) -> Vec<Shape> {
    (0..3)
        .map(|k| {
            let mut to = at;
            to[k] += len;
            Shape::Arrow {
                from: at,
                to,
                color,
                width: 4.0,
                head: 0.18,
            }
        })
        .collect()
}

impl SceneConstruct for SevenScrews {
    fn name(&self) -> &str {
        "08_seven_screws"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let gold = palette::screw();
        let arm = Arm::iiwa14();
        let rig = ArmRig::joints(stage, arm.clone(), poses::SWING);
        let QSource::Joints(q) = rig.source.clone() else {
            return Err(fail("the arm of chapter 08 is held by seven trackers".into()));
        };

        // ---- the arm world: the arm's 23 parts, then each joint's axis line
        // and screw, then the home pose of the gripper, and the floor
        let mut specs = rig.parts(palette::steel());
        for i in 0..N {
            let a = model::scale(arm.axis[i], 0.34);
            specs.push(PartSpec::new(
                vec![Shape::line(
                    model::sub(arm.origin[i], a),
                    model::add(arm.origin[i], a),
                    gold,
                    3.5,
                )],
                still(Pose::NOTHING),
            ));
        }
        for i in 0..N {
            let one = Screw {
                angle: 1.1,
                axis: arm.axis[i],
                point: arm.origin[i],
                slide: 0.0,
            };
            specs.push(PartSpec::new(screw_shapes(&one, 0.4, 0.17), still(Pose::NOTHING)).layer(2));
        }
        specs.push(PartSpec::new(frame_shapes(arm.home.p, 0.14, Palette::ink()), still(Pose::NOTHING)).layer(2));
        specs.push(PartSpec::new(table([-0.8, -0.8], [0.9, 0.8], 0.0, 0.2), still(Pose::NOTHING)).layer(-2));
        let view = View::new(ARM_SCALE, v(ARM_AT[0], ARM_AT[1]));
        let world = World::build(stage, view, 1.6, 0.25, specs, Vec::<Pin>::new())?;
        for i in ArmRig::STEEL_COLLARS.chain(AXES_AT..=HOME_AT) {
            stage.set_tracker_value(world.parts[i].shown, 0.0)?;
        }
        world.add_to_scene(stage)?;

        // 08_1: standing straight up, 1.3 m tall, seven hinges, each a turn
        // about its own axis
        let line = say(&kit, stage, BEATS[0])?;
        world.show(stage, 0.8)?;
        line.wait_for(stage, "Standing straight up")?;
        let to_zero: Vec<(Mob, f64)> = q.iter().map(|&t| (t, 0.0)).collect();
        tween_all(stage, &to_zero, 1.8)?;
        line.wait_for(stage, "one point three meters tall")?;
        let height = fact_f64("G-A.height")?;
        let (foot, top) = (view.project([0.5, 0.0, 0.0]), view.project([0.5, 0.0, height]));
        let mark = stage.arena_mut().add(
            polyline(vec![v(foot[0] - 0.1, foot[1]), foot, top, v(top[0] - 0.1, top[1])])
                .style(line_style(Palette::dim(), 3.0))
                .build(),
        );
        let mark_words = kit.text(
            stage,
            &format!("{height:.1} m"),
            30.0,
            Palette::ink(),
            v(top[0] + 0.55, (foot[1] + top[1]) / 2.0),
        )?;
        fade_in_all(stage, &[mark, mark_words], 0.5)?;
        line.wait_for(stage, "Each of its seven joints is a hinge")?;
        let mut numbers = Vec::new();
        let mut lines_on = Vec::new();
        for i in 0..N {
            let sign = if i % 2 == 0 { 1.0 } else { -1.0 };
            let end = model::add(arm.origin[i], model::scale(arm.axis[i], sign * 0.48));
            let at = view.project(end);
            numbers.push(kit.text(stage, &format!("{}", i + 1), 26.0, gold, at)?);
            lines_on.push(world.parts[AXES_AT + i].shown);
        }
        let ons: Vec<(Mob, f64)> = lines_on.iter().map(|&t| (t, 1.0)).collect();
        tween_all(stage, &ons, 0.8)?;
        fade_in_lagged(stage, &numbers, 0.9, 0.4)?;
        line.wait_for(stage, "All it can do")?;
        let named = [(SHOULDER, model::SHOULDER), (ELBOW_WORD, ELBOW), (WRIST, model::WRIST)];
        let mut names = Vec::new();
        for (word, joint) in named {
            let p = view.project(arm.origin[joint]);
            names.push(kit.text_left(stage, word, 26.0, Palette::ink(), v(p[0] + 0.95, p[1]))?);
        }
        fade_in_all(stage, &names, 0.5)?;
        let code_lines = model_lines()?;
        let code_refs: Vec<&str> = code_lines.iter().map(String::as_str).collect();
        let panel = CodePanel::build(&kit, stage, &code_refs, 22.0, 6.0)?;
        play!(stage; panel.slide_in(0.8)?);

        // 08_2: a turn about an axis is a screw with no slide: seven, stacked
        let line = say(&kit, stage, BEATS[1])?;
        leave(stage, &[panel.group, mark, mark_words], &[], 0.5)?;
        line.wait_for(stage, "So the arm is seven screws")?;
        for i in 0..N {
            tween_all(stage, &[(world.parts[SCREWS_AT + i].shown, 1.0)], 0.35)?;
        }
        let about = tag(&kit, stage, ABOUT_A_LINE, v(3.2, -1.8), 24.0)?;
        fade_in_all(stage, &[about], 0.5)?;

        // 08_3: only the elbow turns: what moves?
        let line = say(&kit, stage, BEATS[2])?;
        let mut gone = vec![about];
        gone.extend(&names);
        gone.extend(&numbers);
        let mut zero: Vec<Mob> = (0..N).map(|i| world.parts[SCREWS_AT + i].shown).collect();
        zero.extend((0..N).map(|i| world.parts[AXES_AT + i].shown));
        zero.extend(ArmRig::GOLD_COLLARS.map(|i| world.parts[i].shown));
        leave(stage, &gone, &zero, 0.5)?;
        let elbow_gold = world.parts[ArmRig::GOLD_COLLARS.start + ELBOW].shown;
        let steel: Vec<(Mob, f64)> = ArmRig::STEEL_COLLARS.map(|i| (world.parts[i].shown, 1.0)).collect();
        tween_all(stage, &steel, 0.4)?;
        play!(stage; tween(elbow_gold, 1.0, 0.5)?);
        let card = QuizCard::build(&kit, stage, ASK, QuizLook::at(v(4.35, 0.3), 0.92))?;
        line.wait_last(stage)?;
        card.show(stage, 0.5)?;
        card.count(&kit, stage)?;

        // 08_4: everything past the elbow, as one rigid piece
        let line = say(&kit, stage, BEATS[3])?;
        leave(stage, &card.mobs(), &[], 0.4)?;
        line.wait_for(stage, "Everything past the elbow")?;
        // the parts before the elbow (the foot, the links up to it, their
        // collars) go grey; then the elbow turns a quarter turn
        let mut still_parts: Vec<usize> = vec![0];
        still_parts.extend((0..=ELBOW).map(|k| ArmRig::LINKS.start + k));
        still_parts.extend((0..=ELBOW).map(|i| ArmRig::STEEL_COLLARS.start + i));
        let greys: Vec<(Mob, f64)> = still_parts.iter().map(|&i| (world.parts[i].grey, 1.0)).collect();
        tween_all(stage, &greys, 0.5)?;
        let quarter = poses::ELBOW_ONLY[ELBOW];
        play!(stage; tween(q[ELBOW], quarter * DEG, 1.6)?);

        // 08_5: the recipe: the last joint's screw first, each axis where it
        // stood at zero
        let line = say(&kit, stage, BEATS[4])?;
        play!(stage; tween(q[ELBOW], 0.0, 0.7)?);
        let ungrey: Vec<(Mob, f64)> = still_parts.iter().map(|&i| (world.parts[i].grey, 0.0)).collect();
        tween_all(stage, &ungrey, 0.4)?;
        let gold_collars: Vec<Mob> = ArmRig::GOLD_COLLARS.map(|i| world.parts[i].shown).collect();
        let steel_mobs: Vec<(Mob, f64)> = ArmRig::STEEL_COLLARS.map(|i| (world.parts[i].shown, 1.0)).collect();
        let mut off: Vec<(Mob, f64)> = gold_collars.iter().map(|&t| (t, 0.0)).collect();
        off.extend(steel_mobs);
        tween_all(stage, &off, 0.3)?;
        line.wait_for(stage, "Start with the gripper's pose")?;
        play!(stage; tween(world.parts[HOME_AT].shown, 1.0, 0.5)?);
        let m_label = {
            let p = view.project(arm.home.p);
            kit.text(stage, M_WORD, 30.0, Palette::ink(), v(p[0] + 0.45, p[1] + 0.15))?
        };
        fade_in_all(stage, &[m_label], 0.3)?;
        line.wait_for(stage, "Apply the screw of the last joint")?;
        for i in (0..N).rev() {
            play!(stage;
                tween(gold_collars[i], 1.0, 0.2)?,
                tween(world.parts[AXES_AT + i].shown, 1.0, 0.2)?,
            );
            play!(stage; tween(q[i], RECIPE_DEG * DEG, 0.55)?);
        }
        let sums = tag(&kit, stage, ORDER_OF_SUMS, v(3.4, -1.8), 24.0)?;
        fade_in_all(stage, &[sums], 0.5)?;

        // 08_6: the product of seven exponentials, built from the right
        let line = say(&kit, stage, BEATS[5])?;
        leave(stage, &[sums], &[], 0.4)?;
        let mut parts = Vec::new();
        for term in TERMS {
            parts.push(kit.tex(stage, term, 46.0, Palette::ink(), &[], ORIGIN)?);
        }
        let gap = 0.2;
        let widths: Vec<f64> = parts.iter().map(|&m| stage.get_width(m)).collect();
        let total: f64 = widths.iter().sum::<f64>() + gap * (parts.len() as f64 - 1.0);
        let mut x = 2.4 - total / 2.0;
        let mut centres = Vec::new();
        for (&m, &w) in parts.iter().zip(&widths) {
            let c = v(x + w / 2.0, 2.95);
            stage.move_to(m, c, ORIGIN);
            centres.push(c);
            x += w + gap;
        }
        let mut under = Vec::new();
        for (words, c) in TERM_LABELS.iter().zip(&centres) {
            if !words.is_empty() {
                under.push(kit.text(stage, words, 22.0, Palette::dim(), v(c[0], 2.2))?);
            }
        }
        line.wait_for(stage, "So the gripper's pose is a product")?;
        // right to left, as the screws lit
        for &m in parts.iter().rev() {
            play!(stage; fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?.rt(0.35));
        }
        fade_in_all(stage, &under, 0.5)?;
        let year = fact("G-L.brockett.year")?;
        let cite = lower_third(&kit, stage, PRODUCT_LABEL, &format!("Brockett, {year}"))?;
        line.wait_for(stage, "Roger Brockett")?;
        play!(stage; fade_in(stage.arena_mut(), cite, ORIGIN, 1.0)?.rt(0.5));
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::degrees;

    /// The piece that rides the first `link` joints, by the recipe: the
    /// screws of joints `link` down to 1 applied to its home place, the
    /// last joint's first (08_5).
    fn recipe_pose(arm: &Arm, q: &model::Q, link: usize) -> Pose {
        let mut p = Pose::NOTHING;
        for s in (0..link).rev() {
            p = model::exp6(arm.screw(s).map(|x| x * q[s])).then(&p);
        }
        p
    }

    /// The same piece joint by joint, as the arm computes it.
    fn chain_pose(arm: &Arm, q: &model::Q, link: usize) -> Pose {
        arm.carried(q, link)
    }

    #[test]
    fn the_beats_are_the_storyboards_and_the_scripts() {
        let board = explainer_kit::storyboard::parse(crate::STORYBOARD).unwrap();
        let chapter = board.iter().find(|c| c.key == "08_seven_screws").expect("chapter 08");
        let ids: Vec<&str> = chapter.beats.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(ids, BEATS);
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for beat in &chapter.beats {
            let words = beat.spoken().expect("every beat of chapter 08 is spoken");
            assert_eq!(script.text(&beat.id), Some(words.as_str()), "{}", beat.id);
        }
    }

    /// 08_3: four seconds; the computed answer is that everything past the
    /// elbow moves.
    #[test]
    fn the_quiz_asks_what_moves_past_the_elbow() {
        assert_eq!(ASK.seconds().unwrap(), 4);
        assert_eq!(fact("G-Q.08_3.answer").unwrap(), "past");
        assert_eq!(ASK.answer_index().unwrap(), 1);
    }

    /// 08_1: seven joints, alternately vertical and level (the facts' axes),
    /// the arm 1.3 m tall, the shoulder, elbow and wrist the second, fourth
    /// and sixth.
    #[test]
    fn seven_hinges_one_point_three_metres_tall() {
        assert_eq!(fact("G-A.joints").unwrap(), "7");
        let want = ["0,0,1", "0,1,0", "0,0,1", "0,-1,0", "0,0,1", "0,1,0", "0,0,1"];
        for (i, w) in want.iter().enumerate() {
            assert_eq!(fact(&format!("G-A.axis.{}", i + 1)).unwrap(), *w);
        }
        assert_eq!(format!("{:.1}", fact_f64("G-A.height").unwrap()), "1.3");
        assert_eq!((model::SHOULDER + 1, ELBOW + 1, model::WRIST + 1), (2, 4, 6));
        // the code panel's lines: seven axes with heights, then M
        let lines = model_lines().unwrap();
        assert_eq!(lines.len(), 8);
        assert_eq!(lines[0], "S1: axis (0, 0, 1) at z = 0.1575");
        assert!(lines[3].contains("(0, -1, 0)"));
        assert_eq!(lines[7], "M: (0.0, 0.0, 1.3060), R = I");
    }

    /// 08_2 and 08_4: a hinge is a screw with no slide; turn only the elbow
    /// and exactly the pieces past it move.
    #[test]
    fn only_what_is_past_the_elbow_moves() {
        let arm = Arm::iiwa14();
        for i in 0..N {
            let s = model::screw_of(&model::exp6(arm.screw(i).map(|x| x * 0.8)));
            assert!(s.slide.abs() < 1e-9, "joint {i} has no slide");
            assert!(model::norm(model::sub(s.axis, arm.axis[i])) < 1e-9);
        }
        let q = degrees(poses::ELBOW_ONLY);
        for link in 0..=N {
            let moved = chain_pose(&arm, &q, link).gap(&Pose::NOTHING) > 1e-9;
            assert_eq!(moved, link > ELBOW, "link {link}");
        }
        assert_eq!(fact("G-K.elbow.moves").unwrap(), "5,6,7,tip");
    }

    /// 08_5: applying the screws from the last joint down to the first, each
    /// about its axis at zero, builds the same pose every piece has
    /// joint by joint; and the whole is the product of exponentials of the
    /// gripper's home pose.
    #[test]
    fn the_recipe_is_the_chain() {
        let arm = Arm::iiwa14();
        let q: model::Q = [0.4, -0.7, 0.2, 1.1, -0.3, 0.6, 0.9];
        for link in 0..=N {
            let by_recipe = recipe_pose(&arm, &q, link);
            assert!(by_recipe.gap(&chain_pose(&arm, &q, link)) < 1e-9, "link {link}");
        }
        let tip = (0..N)
            .rev()
            .fold(arm.home, |t, s| model::exp6(arm.screw(s).map(|x| x * q[s])).then(&t));
        assert!(tip.gap(&arm.tip(&q)) < 1e-9, "T(q) = e^(S1 q1) ... e^(S7 q7) M");
        // at zero the gripper is at M
        assert!(arm.tip(&[0.0; N]).gap(&arm.home) < 1e-12);
    }

    /// 08_6: the line is six parts, with a label under three of them, and
    /// Brockett's year is the fact's.
    #[test]
    fn the_line_has_six_parts_and_three_labels() {
        assert_eq!(TERMS.len(), TERM_LABELS.len());
        let labelled = TERM_LABELS.iter().filter(|s| !s.is_empty()).count();
        assert_eq!(labelled, 4);
        assert_eq!(fact("G-L.brockett.year").unwrap(), "1983");
        assert_eq!(RECIPE_DEG, 40.0);
    }
}
