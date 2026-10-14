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
    fact_list, fade_in_all, fade_in_lagged, fail, group, leave, lie_winter, line_style, say, screw_shapes, still,
    table, tag, tween_all,
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
/// Plain labels under the line's parts (empty where there is none; the first
/// exponential's is built from lines of text and TeX by [`joint_label`]).
const TERM_LABELS: [&str; 6] = ["the gripper's\npose", "", "", "…", "", "the pose at zero"];
/// 08_6's label under the first exponential (R2 B-08), top to bottom: plain
/// lines, then lines with the angle q₁ in TeX (the subscript is TeX's).
const JOINT_LABEL_PLAIN: [&str; 3] = ["joint 1's twist", "at one radian", "a second, held"];
const JOINT_LABEL_TEX: [&str; 2] = [r"\text{for } q_1 \text{ seconds}", r"(q_1 \text{ in radians})"];
/// How far apart the label's lines stand.
const JOINT_LABEL_PITCH: f64 = 0.33;
/// How many of the arm's joints each named point carries, from the base to
/// the gripper: a ball joint's three at the shoulder, the elbow's hinge, a
/// ball joint's three at the wrist (the usual kinematic model, `G-L.kuka.axes`
/// in all).
const JOINTS_AT_POINT: [usize; 3] = [3, 1, 3];
/// The words of 08_1's points' labels and their size.
const POINT_LABEL_PT: f64 = 22.0;
/// 08_6's lower third's end: whose groups the line is built on.
const BUILT_ON: &str = "built on Lie's groups, begun";

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
/// Where the arm is drawn: the foot's screen position. High enough that the
/// floor's front corner ends above a lower third (at -2.95 it ran off the
/// frame's bottom edge).
const ARM_AT: [f64; 2] = [-3.7, -2.0];
/// The floor under the arm (metres, at the world's origin): small enough
/// that its front corner stays above 08_6's two-line lower third.
const FLOOR_LO: [f64; 2] = [-0.7, -0.5];
const FLOOR_HI: [f64; 2] = [0.6, 0.5];
/// The equation's row: its centre's x and its y, and the labels' y under it.
const ROW_X: f64 = 1.9;
const ROW_Y: f64 = 2.95;
const UNDER_Y: f64 = 2.0;
/// Where the quiz card stands.
const CARD_AT: [f64; 2] = [4.35, 0.3];
const CARD_SCALE: f64 = 0.92;

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

/// The joint numbers (1-based, as the screen numbers them) at each named
/// point, base to gripper: `[1, 2, 3]`, `[4]`, `[5, 6, 7]`.
fn joint_numbers() -> [Vec<usize>; 3] {
    let mut next = 1;
    JOINTS_AT_POINT.map(|n| {
        let here: Vec<usize> = (next..next + n).collect();
        next += n;
        here
    })
}

/// A named point's label: `shoulder · joints 1 2 3`, `elbow · joint 4`
/// (R2 C-11).
fn point_label(word: &str, joints: &[usize]) -> String {
    let numbers: Vec<String> = joints.iter().map(usize::to_string).collect();
    let noun = if joints.len() == 1 { "joint" } else { "joints" };
    format!("{word} · {noun} {}", numbers.join(" "))
}

/// 08_1: the three named points come in with their joint numbers, and the
/// height mark (its bracket and "1.3 m") goes as they come, in one beat of
/// half a second. The labels begin at the bracket's own line, and "1.3 m"
/// stood between "elbow" and "shoulder" with the line through the first
/// letter of each (the dense read of the scored draft): the mark has said its
/// piece by "All it can do". Returns the labels.
fn name_the_points(kit: &Kit, stage: &mut Stage<'_>, view: &View, arm: &Arm, mark: &[Mob]) -> fmn::Result<Vec<Mob>> {
    let named = [(SHOULDER, model::SHOULDER), (ELBOW_WORD, ELBOW), (WRIST, model::WRIST)];
    let groups = joint_numbers();
    let mut names = Vec::new();
    for ((word, joint), joints) in named.into_iter().zip(&groups) {
        let p = view.project(arm.origin[joint]);
        let label = point_label(word, joints);
        names.push(kit.text_left(stage, &label, POINT_LABEL_PT, Palette::ink(), v(p[0] + 0.95, p[1]))?);
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mark {
        stage.clear_updaters(m, true);
        let gone = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(gone.rt(0.5))?);
    }
    for &m in &names {
        let come = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(come.rt(0.5))?);
    }
    stage.play_prepared(anims)?;
    Ok(names)
}

/// 08_6's lower third's text: who presented it, when, and whose groups it is
/// built on, begun when (the facts' years; 1873–74 is the winter).
fn brockett_words() -> fmn::Result<String> {
    Ok(format!(
        "Brockett, {} · {BUILT_ON} {}",
        fact("G-L.brockett.year")?,
        lie_winter()?
    ))
}

/// The label under the first exponential, built from its lines (centred at
/// `x`, the first line at `y`): plain text and TeX in one group.
fn joint_label(kit: &Kit, stage: &mut Stage<'_>, x: f64, y: f64) -> fmn::Result<Mob> {
    let mut lines = Vec::new();
    for (i, words) in JOINT_LABEL_PLAIN.iter().enumerate() {
        let at = v(x, y - JOINT_LABEL_PITCH * i as f64);
        lines.push(kit.text(stage, words, 22.0, Palette::dim(), at)?);
    }
    for (k, src) in JOINT_LABEL_TEX.iter().enumerate() {
        let at = v(x, y - JOINT_LABEL_PITCH * (JOINT_LABEL_PLAIN.len() + k) as f64);
        lines.push(kit.tex(stage, src, 22.0, Palette::dim(), &[], at)?);
    }
    group(stage, &lines)
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

/// The line of 08_6: its six terms side by side on one row, detached, and the
/// plain labels under them (the terms and the labels, in order).
fn equation_row(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<(Vec<Mob>, Vec<Mob>)> {
    let mut parts = Vec::new();
    for term in TERMS {
        parts.push(kit.tex(stage, term, 46.0, Palette::ink(), &[], ORIGIN)?);
    }
    let gap = 0.35;
    let widths: Vec<f64> = parts.iter().map(|&m| stage.get_width(m)).collect();
    let total: f64 = widths.iter().sum::<f64>() + gap * (parts.len() as f64 - 1.0);
    let mut x = ROW_X - total / 2.0;
    let mut centres = Vec::new();
    for (&m, &w) in parts.iter().zip(&widths) {
        let c = v(x + w / 2.0, ROW_Y);
        stage.move_to(m, c, ORIGIN);
        centres.push(c);
        x += w + gap;
    }
    let mut under = Vec::new();
    for (i, (words, c)) in TERM_LABELS.iter().zip(&centres).enumerate() {
        if i == 1 {
            // the first exponential: its twist, its speed, its time
            under.push(joint_label(kit, stage, c[0], UNDER_Y)?);
        } else if !words.is_empty() {
            under.push(kit.text(stage, words, 22.0, Palette::dim(), v(c[0], UNDER_Y))?);
        }
    }
    Ok((parts, under))
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
        specs.push(PartSpec::new(table(FLOOR_LO, FLOOR_HI, 0.0, 0.2), still(Pose::NOTHING)).layer(-2));
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
        let names = name_the_points(&kit, stage, &view, &arm, &[mark, mark_words])?;
        let code_lines = model_lines()?;
        let code_refs: Vec<&str> = code_lines.iter().map(String::as_str).collect();
        let panel = CodePanel::build(&kit, stage, &code_refs, 22.0, 6.0)?;
        play!(stage; panel.slide_in(0.8)?);

        // 08_2: a turn about an axis is a screw with no slide: seven, stacked
        let line = say(&kit, stage, BEATS[1])?;
        // the model's code panel (eight lines) stays through the line's first
        // sentence, so that it can be read, and goes as the screws light
        line.wait_for(stage, "So the arm is seven screws")?;
        leave(stage, &[panel.group], &[], 0.4)?;
        for i in 0..N {
            tween_all(stage, &[(world.parts[SCREWS_AT + i].shown, 1.0)], 0.35)?;
        }
        let about = tag(&kit, stage, ABOUT_A_LINE, v(3.2, -1.8), 24.0)?;
        fade_in_all(stage, &[about], 0.5)?;

        // 08_3: only the elbow turns: what moves?
        let line = say(&kit, stage, BEATS[2])?;
        // ("about" stays up until the question's card takes its place)
        let mut gone = names.clone();
        gone.extend(&numbers);
        let mut zero: Vec<Mob> = (0..N).map(|i| world.parts[SCREWS_AT + i].shown).collect();
        zero.extend((0..N).map(|i| world.parts[AXES_AT + i].shown));
        zero.extend(ArmRig::GOLD_COLLARS.map(|i| world.parts[i].shown));
        leave(stage, &gone, &zero, 0.5)?;
        let elbow_gold = world.parts[ArmRig::GOLD_COLLARS.start + ELBOW].shown;
        let steel: Vec<(Mob, f64)> = ArmRig::STEEL_COLLARS.map(|i| (world.parts[i].shown, 1.0)).collect();
        tween_all(stage, &steel, 0.4)?;
        play!(stage; tween(elbow_gold, 1.0, 0.5)?);
        let card = QuizCard::build(&kit, stage, ASK, QuizLook::at(v(CARD_AT[0], CARD_AT[1]), CARD_SCALE))?;
        line.wait_last(stage)?;
        leave(stage, &[about], &[], 0.3)?;
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
        let (parts, under) = equation_row(&kit, stage)?;
        line.wait_for(stage, "So the gripper's pose is a product")?;
        // ("the order of the sums" has stood since the last line's end: it
        // goes as the line it explains appears)
        leave(stage, &[sums], &[], 0.4)?;
        // right to left, as the screws lit
        for &m in parts.iter().rev() {
            play!(stage; fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?.rt(0.35));
        }
        fade_in_all(stage, &under, 0.5)?;
        let cite = lower_third(&kit, stage, PRODUCT_LABEL, &brockett_words()?)?;
        line.wait_for(stage, "Roger Brockett")?;
        play!(stage; fade_in(stage.arena_mut(), cite, ORIGIN, 1.0)?.rt(0.5));
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::{CARD_W, LOWER_THIRD_TOP, SAFE, cut_texts, probe, run_chapter};
    use super::*;
    use crate::model::degrees;

    /// 08_1 to 08_6: the floor's corners, the standing arm and its mark stay
    /// inside the frame (the floor once ran off the bottom edge) and above a
    /// lower third; the quiz card clears the arm's picture.
    #[test]
    fn the_arm_and_its_floor_fit_the_frame() {
        let view = View::new(ARM_SCALE, v(ARM_AT[0], ARM_AT[1]));
        let arm = Arm::iiwa14();
        let mut pts: Vec<V3> = [
            FLOOR_LO,
            [FLOOR_HI[0], FLOOR_LO[1]],
            FLOOR_HI,
            [FLOOR_LO[0], FLOOR_HI[1]],
        ]
        .iter()
        .map(|c| [c[0], c[1], 0.0])
        .collect();
        pts.push(arm.home.p);
        let (lo, hi) = view.box_of(pts);
        assert!(lo[1] > LOWER_THIRD_TOP, "the floor's front corner is at {:.2}", lo[1]);
        assert!(lo[0] > -SAFE[0] && hi[1] < SAFE[1]);
        let card_left = CARD_AT[0] - 0.5 * CARD_W * CARD_SCALE;
        // the names and the code panel start right of the floor; the card is
        // never on screen with the equation
        assert!(hi[0] < card_left - 0.5);
        // the equation's labels (about 22 px of text) end below the row and
        // above the arm's gripper
        let top = view.project(arm.home.p)[1] + 0.45;
        assert!(top < UNDER_Y - 0.4, "the gripper's frame reaches {top:.2}");
    }

    /// 08_6: the six terms of the line fit the safe area on one row, and the
    /// plain labels under them stand apart (they overlapped).
    #[test]
    fn the_equation_row_and_its_labels_fit() {
        probe(|stage| {
            let kit = Kit::new()?;
            let (parts, under) = equation_row(&kit, stage)?;
            let left = stage.get_left(parts[0])[0];
            let right = stage.get_right(*parts.last().unwrap())[0];
            assert!(
                left > -SAFE[0] && right < SAFE[0],
                "the line runs {left:.2} to {right:.2}"
            );
            for pair in under.windows(2) {
                let (r, l) = (stage.get_right(pair[0])[0], stage.get_left(pair[1])[0]);
                assert!(l - r > 0.15, "two labels are {:.2} apart", l - r);
            }
            for &m in &under {
                assert!(stage.get_top(m)[1] < ROW_Y - 0.4, "a label touches the line");
            }
            Ok(())
        });
    }

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
        // three plain labels, and the first exponential's built from lines
        let labelled = TERM_LABELS.iter().filter(|s| !s.is_empty()).count();
        assert_eq!(labelled + 1, 4);
        assert!(TERM_LABELS[1].is_empty());
        assert_eq!(fact("G-L.brockett.year").unwrap(), "1983");
        assert_eq!(RECIPE_DEG, 40.0);
    }

    /// The whole chapter constructs, headless.
    #[test]
    fn the_chapter_constructs_end_to_end() {
        run_chapter(&mut SevenScrews {
            kit: Kit::new().expect("the kit"),
        });
    }

    /// 08_1: the three named points carry their joints, 1 2 3, 4 and 5 6 7,
    /// seven in all (`G-L.kuka.axes`), each group's middle joint the model's
    /// shoulder, elbow and wrist; the labels are the storyboard's words.
    #[test]
    fn the_points_carry_their_joint_numbers() {
        let groups = joint_numbers();
        assert_eq!(groups, [vec![1, 2, 3], vec![4], vec![5, 6, 7]]);
        assert_eq!(
            groups.iter().map(Vec::len).sum::<usize>(),
            fact("G-L.kuka.axes").unwrap().parse::<usize>().unwrap()
        );
        assert_eq!(N, 7);
        // 0-based, the middle of each group is the model's named joint
        let middles: Vec<usize> = groups.iter().map(|g| g[g.len() / 2] - 1).collect();
        assert_eq!(middles, [model::SHOULDER, ELBOW, model::WRIST]);
        let labels = [
            point_label(SHOULDER, &groups[0]),
            point_label(ELBOW_WORD, &groups[1]),
            point_label(WRIST, &groups[2]),
        ];
        assert_eq!(
            labels,
            [
                "shoulder · joints 1 2 3".to_owned(),
                "elbow · joint 4".to_owned(),
                "wrist · joints 5 6 7".to_owned(),
            ]
        );
        for l in &labels {
            assert!(
                crate::STORYBOARD.contains(l.as_str()),
                "the storyboard does not say {l:?}"
            );
        }
    }

    /// 08_1: the three labels, where the scene stands them, end before the
    /// code panel's left edge once it has slid in, stand clear of each other
    /// and inside the safe area.
    #[test]
    fn the_point_labels_clear_the_code_panel() {
        let arm = Arm::iiwa14();
        let view = View::new(ARM_SCALE, v(ARM_AT[0], ARM_AT[1]));
        let groups = joint_numbers();
        let named = [(SHOULDER, model::SHOULDER), (ELBOW_WORD, ELBOW), (WRIST, model::WRIST)];
        probe(|stage| {
            let kit = Kit::new()?;
            let lines = model_lines()?;
            let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
            let panel = CodePanel::build(&kit, stage, &refs, 22.0, 6.0)?;
            let panel_left = stage.get_left(panel.group)[0] - (panel.width + 0.45);
            let mut boxes = Vec::new();
            for ((word, joint), joints) in named.into_iter().zip(&groups) {
                let p = view.project(arm.origin[joint]);
                let label = kit.text_left(
                    stage,
                    &point_label(word, joints),
                    POINT_LABEL_PT,
                    Palette::ink(),
                    v(p[0] + 0.95, p[1]),
                )?;
                let (l, r) = (stage.get_left(label)[0], stage.get_right(label)[0]);
                let (b, t) = (stage.get_bottom(label)[1], stage.get_top(label)[1]);
                assert!(l > -SAFE[0] && r < SAFE[0] && t < SAFE[1] && b > -SAFE[1]);
                assert!(
                    r < panel_left - 0.1,
                    "{word}: ends at {r:.2}, the panel starts at {panel_left:.2}"
                );
                boxes.push((b, t));
            }
            // shoulder, elbow, wrist: each higher than the one before
            for pair in boxes.windows(2) {
                assert!(pair[1].0 > pair[0].1 + 0.05, "two labels share height: {pair:?}");
            }
            Ok(())
        });
    }

    /// 08_1 (the dense read of the scored draft): the height mark goes in the
    /// beat the point labels come in, half a second, and is gone when it is
    /// over (the labels begin at the bracket's own line, with "1.3 m" between
    /// two of them), and the three labels are in the scene.
    #[test]
    fn the_height_mark_goes_as_the_points_are_named() {
        let arm = Arm::iiwa14();
        let view = View::new(ARM_SCALE, v(ARM_AT[0], ARM_AT[1]));
        // where the scene puts the bracket: the foot and top of the arm at
        // half a unit out
        let height = fact_f64("G-A.height").unwrap();
        let (foot, top) = (view.project([0.5, 0.0, 0.0]), view.project([0.5, 0.0, height]));
        probe(move |stage| {
            let kit = Kit::new()?;
            let bracket = stage
                .arena_mut()
                .add(polyline(vec![foot, top]).style(line_style(Palette::dim(), 3.0)).build());
            let words = kit.text(
                stage,
                "1.3 m",
                30.0,
                Palette::ink(),
                v(top[0] + 0.55, 0.5 * (foot[1] + top[1])),
            )?;
            stage.add_to_scene(bracket)?;
            stage.add_to_scene(words)?;
            assert!(stage.scene().mobjects().contains(&bracket), "the bracket starts up");
            let start = stage.scene().time().to_f64();
            let names = name_the_points(&kit, stage, &view, &arm, &[bracket, words])?;
            let took = stage.scene().time().to_f64() - start;
            assert!((took - 0.5).abs() < 0.1, "the beat took {took:.2} s");
            // a fade out ends by taking the mobject out of the scene
            assert!(
                !stage.scene().mobjects().contains(&bracket) && !stage.scene().mobjects().contains(&words),
                "the height mark is still up when the labels are"
            );
            assert_eq!(names.len(), 3);
            Ok(())
        });
    }

    /// 08_6: the lower third is the storyboard's words, years from the
    /// facts; it fits its lines (the kit cuts a longer text), and the floor's
    /// front corner stands above it.
    #[test]
    fn the_lower_third_is_built_on_lies_groups_and_clears_the_floor() {
        let words = brockett_words().unwrap();
        assert_eq!(words, "Brockett, 1983 · built on Lie's groups, begun 1873–74");
        let line = format!("{PRODUCT_LABEL} {words}");
        assert!(
            crate::STORYBOARD.contains(&line),
            "the storyboard does not say {line:?}"
        );
        assert_eq!(fact("G-L.lie.year").unwrap(), "1873");
        let view = View::new(ARM_SCALE, v(ARM_AT[0], ARM_AT[1]));
        let front = view
            .box_of([
                [FLOOR_LO[0], FLOOR_LO[1], 0.0],
                [FLOOR_HI[0], FLOOR_LO[1], 0.0],
                [FLOOR_HI[0], FLOOR_HI[1], 0.0],
                [FLOOR_LO[0], FLOOR_HI[1], 0.0],
            ])
            .0[1];
        probe(|stage| {
            let kit = Kit::new()?;
            let band = lower_third(&kit, stage, PRODUCT_LABEL, &words)?;
            assert!(cut_texts(&kit).is_empty(), "cut: {:?}", cut_texts(&kit));
            assert!(stage.get_left(band)[0] > -SAFE[0] && stage.get_right(band)[0] < SAFE[0]);
            let top = stage.get_top(band)[1];
            assert!(
                front > top + 0.05,
                "the floor's front corner is at {front:.2}, the band's top at {top:.2}"
            );
            Ok(())
        });
    }

    /// 08_6: the first exponential's label is the storyboard's words in
    /// five lines, inside the safe area, under its term, clear of the arm
    /// (every joint, in the pose of the recipe, with the links' width).
    #[test]
    fn the_first_exponentials_label_is_clear_of_the_arm() {
        let said = format!(
            "{} {} {}",
            JOINT_LABEL_PLAIN.join(" "),
            "for q_1 seconds",
            "(q_1 in radians)"
        );
        assert_eq!(
            said,
            "joint 1's twist at one radian a second, held for q_1 seconds (q_1 in radians)"
        );
        // the storyboard's sentence, with its subscripts
        let storyboard = "joint 1's twist at one radian a second, held for q₁ seconds (q₁ in radians)";
        assert!(crate::STORYBOARD.contains(storyboard));
        let arm = Arm::iiwa14();
        let view = View::new(ARM_SCALE, v(ARM_AT[0], ARM_AT[1]));
        probe(|stage| {
            let kit = Kit::new()?;
            let (parts, under) = equation_row(&kit, stage)?;
            let label = under[1];
            let (l, r) = (stage.get_left(label)[0], stage.get_right(label)[0]);
            let (b, t) = (stage.get_bottom(label)[1], stage.get_top(label)[1]);
            assert!(l > -SAFE[0] && r < SAFE[0] && b > -SAFE[1], "{l:.2}..{r:.2}, {b:.2}");
            // under its own term, centred on it
            let term = parts[1];
            let mid = 0.5 * (stage.get_left(term)[0] + stage.get_right(term)[0]);
            assert!((0.5 * (l + r) - mid).abs() < 0.05, "the label is not under its term");
            assert!(t < ROW_Y - 0.4);
            // the arm in the recipe's pose: its joints and tip, each with the
            // links' half width, stay off the label's box
            let q = degrees([RECIPE_DEG; N]);
            let mut points: Vec<V3> = (0..N).map(|i| arm.joint(&q, i).0).collect();
            points.push(arm.tip(&q).p);
            for p in points {
                let s = view.project(p);
                let near_x = s[0] > l - 0.25 && s[0] < r + 0.25;
                let near_y = s[1] > b - 0.25 && s[1] < t + 0.25;
                assert!(!(near_x && near_y), "the arm's point {s:?} is in the label's box");
            }
            Ok(())
        });
    }
}
