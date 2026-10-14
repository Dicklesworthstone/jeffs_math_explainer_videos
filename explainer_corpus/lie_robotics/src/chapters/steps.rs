//! Chapter 13, steps: the question again, the pose wanted (13_1); angles
//! subtracted walk into both traps, two orientations a degree and a half
//! apart with ring angles ninety apart (13_2); so compare the poses on the
//! flat piece, where one screw joins them and its twist is the error
//! (13_3); run the Jacobian backwards and move (13_4); how many steps
//! (13_5); and three, each gaining more than the one before: Newton's
//! method on a curved space (13_6).

use explainer_kit::props::CodePanel;
use explainer_kit::props::lower_third;
use explainer_kit::{Kit, Palette, Timed, play, tween, v};
use fmn::prelude::*;

use super::draw::{
    ARM_SCALE, ArmRig, Ask, BADGE_CORNER, BOOK_HALF, Badge, Choice, PartSpec, QuizCard, QuizLook, Shape, View, World,
    book, circle_points, dashed_rect, fact, fact_f64, fact_list, fade_in_all, fade_in_lagged, ground, leave, mix,
    perp_basis, say, screw_shapes, still, table, tag,
};
use crate::model::{self, Arm, DEG, M3, Pose, V3, X, Y, Z, add, dot, poses, scale};
use crate::palette;

pub struct Steps {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const WHICH_ANGLES: &str = "q = ?";
const THE_GOAL: &str = "the pose I want";
const TWO_BOOKS_NOTE: &str = "ring angles:";
const DIFFER: &str = "differ by:";
const HOW_MANY: &str = "how many steps?";
const LOG_LABEL: &str = "the logarithm:";
const LOG_TEXT: &str = "the twist that would reach the goal in one second";
const SMALLEST_LABEL: &str = "many joint speeds make that twist:";
const SMALLEST_TEXT: &str = "the smallest is taken";
const A_TWIST: &str = "# a twist, read in the gripper's frame";
const THE_SMALLEST: &str = "# the smallest joint step that makes it";
const PROGRAMMERS: &str = "for programmers";
const START: &str = "start";
const AFTER: &str = "after";
const EACH_GAINS: &str = "each step gains more than the one before";
const POOR_START_LABEL: &str = "from a poor start:";
const POOR_START_TEXT: &str =
    "it can take many more steps, or fail; real solvers cap, damp, respect joint limits and retry";
const SPIN_SLIDE: &str = "spin | slide";

/// The beats of the chapter, in order (storyboard ids).
const BEATS: [&str; 6] = ["13_1", "13_2", "13_3", "13_4", "13_5", "13_6"];

/// 13_5's quiz: three, thirty or three hundred (`G-Q.13_5.answer` is the
/// number of steps Newton's method took).
pub const ASK: Ask = Ask {
    line: BEATS[4],
    question: HOW_MANY,
    choices: &[
        Choice { key: "3", label: "3" },
        Choice { key: "30", label: "30" },
        Choice {
            key: "300",
            label: "300",
        },
    ],
};

/// The arm's place (the foot's). The world stands a unit higher than the
/// frame's bottom would put it, so that the screws' lowest ends clear the
/// lower third that three beats show over them.
const ARM_AT: [f64; 2] = [-3.2, -1.6];
/// How far a remaining-error screw's axis runs each way out of its point
/// (metres of the arm's world).
const SCREW_REACH: f64 = 0.5;
/// Where the card sits: under the badge's run in 13_3 and clear of the
/// badge's corner for 13_5.
const CARD_AT: [f64; 2] = [4.45, -0.2];
const CARD_SCALE: f64 = 0.92;
/// The two books in their rings: where each gimbal's centre is, and its
/// radii (outer, middle, inner), in the picture's own units.
const GIMBALS: [[f64; 2]; 2] = [[-3.55, 0.55], [3.55, 0.55]];
const RADII: [f64; 3] = [1.9, 1.5, 1.1];
// the rings nest, and the two gimbals sit side by side with room between
// them for the arrow
const _: () = assert!(RADII[0] > RADII[1] && RADII[1] > RADII[2]);
const _: () = assert!(GIMBALS[1][0] - RADII[0] > 1.2 && GIMBALS[0][0] + RADII[0] < -1.2);
// the captions (2.45 under a centre) stand clear under the outer ring
const _: () = assert!(RADII[0] + 0.3 < 2.45);
const BOOK_FIT: f64 = 0.7;
/// Where the badge runs once, and how much bigger it is there.
const BADGE_RUN_AT: [f64; 2] = [3.6, 2.45];
const BADGE_RUN_SCALE: f64 = 2.4;
/// The table of steps.
const TABLE_X: f64 = 1.3;
const TABLE_Y: f64 = 2.0;
const TABLE_ROW: f64 = 0.62;
const TABLE_PT: f64 = 32.0;

/// The parts this chapter adds to the arm's own 23: the table is 23 and the
/// book 24.
mod part {
    pub const GOAL: usize = 25;
    /// The remaining error as a gold screw, before the first step and after
    /// each of the three: parts 26 to 29.
    pub const SCREWS: std::ops::Range<usize> = 26..30;
}

/// The distance the way the voice says it: whole centimeters from a
/// centimeter up, whole millimeters from one, hundredths of one below.
pub fn distance(mm: f64) -> String {
    if mm >= 10.0 {
        format!("{} cm", (mm / 10.0).round() as i64)
    } else if mm >= 1.0 {
        format!("{} mm", mm.round() as i64)
    } else {
        format!("{mm:.2} mm")
    }
}

/// An angle the way the table prints it: whole from ten degrees up, a place
/// from a tenth, two places below.
pub fn turn_label(deg: f64) -> String {
    if deg >= 10.0 {
        format!("{}°", deg.round() as i64)
    } else if deg >= 0.1 {
        format!("{deg:.1}°")
    } else {
        format!("{deg:.2}°")
    }
}

/// A number of the six as printed, with a true minus sign.
fn six(text: &str) -> String {
    text.replace('-', "−")
}

/// A pose's marker: a dot at its position and the three axes of its
/// orientation, `len` long, in `color`.
pub fn pose_frame(pose: &Pose, len: f64, color: Srgb) -> Vec<Shape> {
    let mut frame = vec![Shape::Dot {
        at: pose.p,
        color,
        radius: 0.13,
    }];
    for k in 0..3 {
        let c = [pose.r[0][k], pose.r[1][k], pose.r[2][k]];
        frame.push(Shape::Arrow {
            from: pose.p,
            to: add(pose.p, scale(c, len)),
            color,
            width: 5.0,
            head: 0.2,
        });
    }
    frame
}

/// The goal's marker, in the goal's pink.
pub fn goal_frame(goal: &Pose, len: f64) -> Vec<Shape> {
    pose_frame(goal, len, palette::goal())
}

/// The gimbal's three frames for ring angles (degrees): the outer ring turns
/// about the vertical, the middle about the outer's own y, the inner about the
/// middle's own x; the book sits in the inner.
pub fn gimbal_frames(angles: [f64; 3]) -> [M3; 3] {
    let outer = model::turn(Z, angles[0] * DEG);
    let middle = model::mul(&outer, &model::turn(Y, angles[1] * DEG));
    let inner = model::mul(&middle, &model::turn(X, angles[2] * DEG));
    [outer, middle, inner]
}

/// A ring as its near half and its far half (about the origin), so that the
/// book sits between them: the half toward the viewer is drawn after it.
fn ring_halves(normal: V3, radius: f64, color: Srgb, width: f64) -> [Shape; 2] {
    let (_, _, toward) = View::basis();
    let (e1, e2) = perp_basis(normal);
    // dot(point, toward) = R cos(theta - phi): positive on (phi - pi/2, phi + pi/2)
    let phi = dot(e2, toward).atan2(dot(e1, toward));
    let half = std::f64::consts::FRAC_PI_2;
    let arc = |from: f64, to: f64| Shape::Path {
        pts: circle_points([0.0; 3], normal, radius, from, to, 28),
        color,
        width,
        closed: false,
        bias: 0.0,
    };
    [arc(phi - half, phi + half), arc(phi + half, phi + 3.0 * half)]
}

/// One book in its three rings, as parts about `centre`: the rings' halves
/// (their normals follow the frames) and the book turned by the inner frame.
fn gimbal_parts(centre: V3, angles: [f64; 3]) -> Vec<PartSpec> {
    let [outer, middle, inner] = gimbal_frames(angles);
    let blue = palette::turn();
    let colour = [mix(blue, ground(), 0.15), blue, mix(blue, Palette::ink(), 0.35)];
    let normals = [
        model::apply(&outer, X),
        model::apply(&middle, Z),
        model::apply(&inner, Y),
    ];
    let at = still(Pose {
        r: model::I3,
        p: centre,
    });
    let mut shapes = Vec::new();
    for ((&n, &r), c) in normals.iter().zip(&RADII).zip(colour) {
        shapes.extend(ring_halves(n, r, c, 4.0));
    }
    let half = [
        BOOK_HALF[0] * BOOK_FIT,
        BOOK_HALF[1] * BOOK_FIT,
        BOOK_HALF[2] * BOOK_FIT,
    ];
    vec![
        PartSpec::new(shapes, at),
        PartSpec::new(book(half), still(Pose { r: inner, p: centre })),
    ]
}

/// The ring angles of the two books and their differences, from the facts.
fn near() -> fmn::Result<([[i64; 3]; 2], [i64; 3])> {
    let triples = fact("G-E.near.triples")?;
    let parse = |s: &str| -> Vec<i64> { s.split(',').filter_map(|x| x.trim().parse().ok()).collect() };
    let rows: Vec<Vec<i64>> = triples.split(';').map(parse).collect();
    let diff = parse(fact("G-E.near.differ")?);
    if rows.len() != 2 || rows.iter().any(|r| r.len() != 3) || diff.len() != 3 {
        return Err(super::draw::fail(
            "G-E.near: not two triples and three differences".into(),
        ));
    }
    Ok((
        [
            [rows[0][0], rows[0][1], rows[0][2]],
            [rows[1][0], rows[1][1], rows[1][2]],
        ],
        [diff[0], diff[1], diff[2]],
    ))
}

impl SceneConstruct for Steps {
    fn name(&self) -> &str {
        "13_steps"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let arm = Arm::iiwa14();
        let (orange, red) = (palette::flat(), palette::warn());
        let q_start = model::degrees(poses::START);
        let goal = model::goal(&arm);
        let run = arm.solve(&q_start, &goal, model::TOL_SPIN, model::TOL_SLIDE);
        let view = View::new(ARM_SCALE, v(ARM_AT[0], ARM_AT[1]));
        let steps = run.steps();

        // ---- the arm world: the arm walks Newton's iterates, the goal,
        // the book and the remaining error as a gold screw at each of them
        let (rig, walk) = ArmRig::along(stage, arm.clone(), run.path.clone());
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
        for q in &run.path {
            let tip = arm.tip(q);
            let s = model::screw_of(&goal.then(&tip.undo()));
            specs.push(PartSpec::new(screw_shapes(&s, SCREW_REACH, 0.14), still(Pose::NOTHING)).layer(3));
        }
        let world = World::build(stage, view, 1.4, 0.25, specs, Vec::new())?;
        let screws: Vec<usize> = part::SCREWS.take(run.path.len()).collect();
        for i in ArmRig::GOLD_COLLARS.chain(screws.iter().copied()).chain([part::GOAL]) {
            stage.set_tracker_value(world.parts[i].shown, 0.0)?;
        }
        world.add_to_scene(stage)?;
        let shown = |i: usize| world.parts[i].shown;

        // ---- the two books in their rings (a world of their own)
        let (pair, differ) = near()?;
        // each gimbal's centre is a point of the screen (the view is the
        // isometric one, so the world point is what projects there)
        let (right, up, _) = View::basis();
        let mut ring_specs = Vec::new();
        for (centre, angles) in GIMBALS.iter().zip(pair) {
            let at = add(scale(right, centre[0]), scale(up, centre[1]));
            ring_specs.extend(gimbal_parts(at, angles.map(|a| a as f64)));
        }
        let rings = World::build(stage, View::new(1.0, v(0.0, 0.0)), 5.0, 0.0, ring_specs, Vec::new())?;
        rings.add_to_scene(stage)?;

        // 13_1: back to the question; the pose I want; which joint angles
        let line = say(&kit, stage, BEATS[0])?;
        world.show(stage, 0.8)?;
        line.wait_for(stage, "Here is the pose I want")?;
        let goal_pos = view.project(goal.p);
        let goal_words = kit.text(
            stage,
            THE_GOAL,
            30.0,
            palette::goal(),
            v(goal_pos[0] + 1.35, goal_pos[1] + 0.55),
        )?;
        play!(stage;
            tween(shown(part::GOAL), 1.0, 0.7)?,
            fade_in(stage.arena_mut(), goal_words, ORIGIN, 1.0)?.rt(0.7),
        );
        line.wait_for(stage, "Which joint angles")?;
        let which = kit.text(stage, WHICH_ANGLES, 44.0, Palette::ink(), v(3.8, 1.9))?;
        fade_in_all(stage, &[which], 0.5)?;
        line.wait_end(stage)?;

        // 13_2: subtracting angles walks into both traps
        let line = say(&kit, stage, BEATS[1])?;
        play!(stage; tween(world.shown, 0.0, 0.6)?);
        leave(stage, &[which, goal_words], &[], 0.3)?;
        line.wait_for(stage, "Subtracting angles")?;
        rings.show(stage, 0.8)?;
        let apart = fact_f64("G-E.near.apart")?;
        let between = Arrow::new(v(-1.15, 0.55), v(1.15, 0.55))
            .thickness(4.0)
            .color(orange)
            .build()?;
        let between = stage.arena_mut().add(between);
        let between_label = kit.text(stage, &format!("{apart:.1}°"), 36.0, orange, v(0.0, 1.0))?;
        line.wait_for(stage, "one point four degrees apart")?;
        fade_in_all(stage, &[between, between_label], 0.6)?;
        line.wait_for(stage, "Yet two of their three ring angles")?;
        let mut shown_texts = Vec::new();
        for (centre, angles) in GIMBALS.iter().zip(pair) {
            let words = format!("({}°, {}°, {}°)", angles[0], angles[1], angles[2]);
            shown_texts.push(kit.text(stage, &words, 32.0, Palette::ink(), v(centre[0], centre[1] - 2.45))?);
        }
        let note = kit.text(stage, TWO_BOOKS_NOTE, 24.0, Palette::dim(), v(0.0, -1.9))?;
        shown_texts.push(note);
        fade_in_all(stage, &shown_texts, 0.6)?;
        let differ_label = kit.text(stage, DIFFER, 28.0, Palette::dim(), v(-1.55, -3.3))?;
        let mut diffs = vec![differ_label];
        for (k, d) in differ.iter().enumerate() {
            let at = v(0.35 + 1.45 * k as f64, -3.3);
            let colour = if *d != 0 { red } else { Palette::ink() };
            diffs.push(kit.text(stage, &format!("{d}°"), 34.0, colour, at)?);
            if *d != 0 {
                diffs.push(dashed_rect(stage, at, 1.15, 0.7, red, 3.0)?);
            }
        }
        fade_in_lagged(stage, &diffs, 1.0, 0.5)?;
        line.wait_end(stage)?;

        // 13_3: do it on the flat piece: one screw joins the poses, and read
        // as a twist it is the error
        let line = say(&kit, stage, BEATS[2])?;
        let mut away = shown_texts.clone();
        away.extend(&diffs);
        away.extend([between, between_label]);
        leave(stage, &away, &[rings.shown], 0.6)?;
        world.show(stage, 0.8)?;
        let badge = Badge::build(stage, v(BADGE_CORNER[0], BADGE_CORNER[1]), 1.0)?;
        line.wait_for(stage, "So do it on the flat piece")?;
        fade_in_all(stage, &[badge.body], 0.5)?;
        line.wait_for(stage, "Compare the pose the arm has")?;
        play!(stage; tween(shown(part::GOAL), 1.0, 0.4)?);
        line.wait_for(stage, "One screw joins them")?;
        play!(stage; tween(shown(screws[0]), 1.0, 0.8)?);
        let moves = badge.move_to(stage, BADGE_RUN_SCALE, v(BADGE_RUN_AT[0], BADGE_RUN_AT[1]), 0.9)?;
        stage.play_prepared(moves)?;
        fade_in_all(stage, &[badge.arrow], 0.5)?;
        line.wait_for(stage, "Read as a twist")?;
        let numbers = fact("G-I.error.twist")?.split(',').map(six).collect::<Vec<_>>();
        let spin_row = format!("{}   {}   {}", numbers[0], numbers[1], numbers[2]);
        let slide_row = format!("{}   {}   {}", numbers[3], numbers[4], numbers[5]);
        let six_head = kit.text(
            stage,
            SPIN_SLIDE,
            24.0,
            Palette::dim(),
            v(BADGE_RUN_AT[0], BADGE_RUN_AT[1] - 0.95),
        )?;
        let spin_text = kit.text(
            stage,
            &spin_row,
            34.0,
            orange,
            v(BADGE_RUN_AT[0], BADGE_RUN_AT[1] - 1.5),
        )?;
        let slide_text = kit.text(
            stage,
            &slide_row,
            34.0,
            orange,
            v(BADGE_RUN_AT[0], BADGE_RUN_AT[1] - 2.05),
        )?;
        fade_in_all(stage, &[six_head, spin_text, slide_text], 0.7)?;
        let log = lower_third(&kit, stage, LOG_LABEL, LOG_TEXT)?;
        fade_in_all(stage, &[log], 0.5)?;
        line.wait_end(stage)?;

        // 13_4: run the Jacobian backwards, and move; look again; repeat
        let line = say(&kit, stage, BEATS[3])?;
        let tol = fact("G-I.ik.tolerances")?;
        let (tol_spin, tol_slide) = tol
            .split_once(',')
            .ok_or_else(|| super::draw::fail("G-I.ik.tolerances".into()))?;
        let tol_slide_mm = format!("{}", tol_slide.parse::<f64>().unwrap_or(0.0) * 1000.0);
        let code = [
            A_TWIST.to_owned(),
            "error = log(inverse(T(q)) * T_goal)".to_owned(),
            THE_SMALLEST.to_owned(),
            "q = q + pinv(J_body(q)) * error".to_owned(),
            format!("repeat until error < {tol_spin} rad and {tol_slide_mm} mm"),
        ];
        let lines: Vec<&str> = code.iter().map(String::as_str).collect();
        let panel = CodePanel::build(&kit, stage, &lines, 26.0, 6.2)?;
        // the numbers and the lower third go, and the badge shrinks back to
        // its corner (where it was for the rest of the video)
        let run_badge = Badge {
            at: v(BADGE_RUN_AT[0], BADGE_RUN_AT[1]),
            scale: BADGE_RUN_SCALE,
            ..badge
        };
        let mut anims = run_badge.move_to(stage, 1.0 / BADGE_RUN_SCALE, v(BADGE_CORNER[0], BADGE_CORNER[1]), 0.6)?;
        for m in [six_head, spin_text, slide_text, log] {
            stage.clear_updaters(m, true);
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.4))?);
        }
        stage.play_prepared(anims)?;
        line.wait_for(stage, "run the Jacobian backwards")?;
        stage.add_to_scene(panel.group)?;
        stage.play(panel.slide_in(0.7)?)?;
        let smallest = lower_third(&kit, stage, SMALLEST_LABEL, SMALLEST_TEXT)?;
        fade_in_all(stage, &[smallest], 0.5)?;
        line.wait_for(stage, "move the joints that way")?;
        play!(stage;
            tween(shown(screws[0]), 0.0, 0.4)?,
            tween(walk, 1.0 / steps as f64, 1.6)?,
        );
        play!(stage; tween(shown(screws[1]), 1.0, 0.4)?);
        line.wait_end(stage)?;

        // 13_5: how many steps from the start, to a tenth of a millimeter?
        let line = say(&kit, stage, BEATS[4])?;
        play!(stage;
            tween(shown(screws[1]), 0.0, 0.4)?,
            tween(walk, 0.0, 0.9)?,
        );
        play!(stage; tween(shown(screws[0]), 1.0, 0.4)?);
        leave(stage, &[panel.group, smallest], &[], 0.5)?;
        let card = QuizCard::build(&kit, stage, ASK, QuizLook::at(v(CARD_AT[0], CARD_AT[1]), CARD_SCALE))?;
        line.wait_last(stage)?;
        card.show(stage, 0.5)?;
        card.count(&kit, stage)?;

        // 13_6: from here, three
        let line = say(&kit, stage, BEATS[5])?;
        let dist = fact_list("G-I.ik.dist_mm")?;
        let angle = fact_list("G-I.ik.angle_deg")?;
        let mut rows = Vec::new();
        for (k, (d, a)) in dist.iter().zip(&angle).enumerate() {
            let label = if k == 0 {
                START.to_owned()
            } else {
                format!("{AFTER} {k}")
            };
            rows.push(kit.text_left(
                stage,
                &format!("{label}: {} and {}", distance(*d), turn_label(*a)),
                TABLE_PT,
                Palette::ink(),
                v(TABLE_X, TABLE_Y - TABLE_ROW * k as f64),
            )?);
        }
        leave(stage, &card.mobs(), &[], 0.5)?;
        // the rows land as the voice says them: half a meter, nine
        // centimeters, four millimeters, three hundredths of a millimeter
        let cues = [
            "Half a meter off",
            "Then nine centimeters",
            "Four millimeters",
            "Three hundredths of a millimeter",
        ];
        for (k, cue) in cues.iter().enumerate().take(dist.len()) {
            line.wait_for(stage, cue)?;
            if k > 0 {
                play!(stage;
                    tween(shown(screws[k - 1]), 0.0, 0.3)?,
                    tween(shown(screws[k]), 1.0, 0.3)?,
                    tween(walk, k as f64 / steps as f64, 1.0)?,
                );
            }
            fade_in_all(stage, &[rows[k]], 0.4)?;
        }
        line.wait_for(stage, "It is Newton's method")?;
        let gains = kit.text(
            stage,
            EACH_GAINS,
            28.0,
            Palette::dim(),
            v(TABLE_X + 2.3, TABLE_Y - TABLE_ROW * 4.2),
        )?;
        let programmers = tag(
            &kit,
            stage,
            PROGRAMMERS,
            v(TABLE_X + 0.8, TABLE_Y - TABLE_ROW * 5.4),
            22.0,
        )?;
        let poor = lower_third(&kit, stage, POOR_START_LABEL, POOR_START_TEXT)?;
        fade_in_all(stage, &[gains, programmers, poor], 0.6)?;
        line.wait_end(stage)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{angle_axis, mul, norm, sub, transpose};

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
        let chapter = board.iter().find(|c| c.key == "13_steps").expect("chapter 13");
        let ids: Vec<&str> = chapter.beats.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(ids, BEATS);
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for beat in &chapter.beats {
            let words = beat.spoken().expect("every beat of chapter 13 is spoken");
            assert_eq!(script.text(&beat.id), Some(words.as_str()), "{}", beat.id);
        }
    }

    #[test]
    fn every_cue_phrase_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("13_1", "Here is the pose I want"),
            ("13_1", "Which joint angles"),
            ("13_2", "Subtracting angles"),
            ("13_2", "one point four degrees apart"),
            ("13_2", "Yet two of their three ring angles"),
            ("13_3", "So do it on the flat piece"),
            ("13_3", "Compare the pose the arm has"),
            ("13_3", "One screw joins them"),
            ("13_3", "Read as a twist"),
            ("13_4", "run the Jacobian backwards"),
            ("13_4", "move the joints that way"),
            ("13_6", "From here, three"),
            ("13_6", "Half a meter off"),
            ("13_6", "Then nine centimeters"),
            ("13_6", "Four millimeters"),
            ("13_6", "Three hundredths of a millimeter"),
            ("13_6", "It is Newton's method"),
        ] {
            let text = script.text(id).unwrap_or_default();
            assert!(text.contains(phrase), "{id} does not say {phrase:?}");
        }
    }

    /// 13_5 and 13_6: the quiz is for three steps (computed), and the table
    /// the scene prints says what the voice says, row by row, each step
    /// gaining more than the one before.
    #[test]
    fn three_steps_and_the_table_is_what_the_voice_says() {
        assert_eq!(ASK.seconds().unwrap(), 5);
        assert_eq!(ASK.choices[ASK.answer_index().unwrap()].label, "3");
        let arm = Arm::iiwa14();
        let run = arm.solve(
            &model::degrees(poses::START),
            &model::goal(&arm),
            model::TOL_SPIN,
            model::TOL_SLIDE,
        );
        assert_eq!(run.steps(), 3);
        assert_eq!(fact("G-I.ik.steps").unwrap(), "3");
        let (dist, angle) = (
            fact_list("G-I.ik.dist_mm").unwrap(),
            fact_list("G-I.ik.angle_deg").unwrap(),
        );
        let rows: Vec<String> = dist
            .iter()
            .zip(&angle)
            .map(|(d, a)| format!("{} and {}", distance(*d), turn_label(*a)))
            .collect();
        assert_eq!(
            rows,
            ["50 cm and 90°", "9 cm and 8.6°", "4 mm and 0.7°", "0.03 mm and 0.01°"]
        );
        // each step gains more than the one before
        let gains: Vec<f64> = dist.windows(2).map(|w| w[0] / w[1]).collect();
        assert!(gains.windows(2).all(|g| g[1] > g[0]), "{gains:?}");
        // the facts are the run the arm walks
        for (k, d) in dist.iter().enumerate() {
            assert!((run.dist_mm[k] - d).abs() < 0.005, "step {k}");
        }
        // within a tenth of a millimeter at the end
        assert!(run.dist_mm[3] < 0.1);
        assert_eq!(run.path.len(), 4, "start and one pose after each of three steps");
    }

    /// 13_3: the six numbers are the twist, in the gripper's frame, that
    /// reaches the goal: its spin is a quarter turn (the first row of the
    /// table), and the world-frame screw has the same angle.
    #[test]
    fn the_error_is_one_twist_a_quarter_turn_long() {
        let arm = Arm::iiwa14();
        let (start, goal) = (arm.tip(&model::degrees(poses::START)), model::goal(&arm));
        let twist = model::log6(&start.undo().then(&goal));
        let printed: Vec<f64> = fact("G-I.error.twist")
            .unwrap()
            .split(',')
            .map(|x| x.parse().unwrap())
            .collect();
        assert_eq!(printed.len(), 6);
        for (a, b) in printed.iter().zip(twist) {
            assert!((a - b).abs() < 5e-4, "{a} against {b}");
        }
        let spin = norm([twist[0], twist[1], twist[2]]);
        let first = fact_list("G-I.ik.angle_deg").unwrap()[0];
        assert!((spin / DEG - first).abs() < 1e-3, "{} against {first}", spin / DEG);
        let screw = model::screw_of(&goal.then(&start.undo()));
        assert!(
            (screw.angle - spin).abs() < 1e-9,
            "the same quarter turn in either frame"
        );
        assert_eq!(six("-1.571"), "−1.571");
    }

    /// 13_2: the two orientations are the facts': ring angles as printed,
    /// the angle between them is the fact's 1.4 degrees, and the ring
    /// frames the scene draws multiply to `model::rings`.
    #[test]
    fn the_two_books_are_a_degree_and_a_half_apart_with_rings_ninety_apart() {
        let (pair, differ) = near().unwrap();
        assert_eq!(pair, [[0, 89, 0], [90, 89, 90]]);
        assert_eq!(differ, [90, 0, 90]);
        for k in 0..3 {
            assert_eq!((pair[1][k] - pair[0][k]).abs(), differ[k]);
        }
        let ra = gimbal_frames(pair[0].map(|a| a as f64))[2];
        let rb = gimbal_frames(pair[1].map(|a| a as f64))[2];
        assert!(model::gap(&ra, &model::rings(0.0, 89.0 * DEG, 0.0)) < 1e-12);
        assert!(model::gap(&rb, &model::rings(90.0 * DEG, 89.0 * DEG, 90.0 * DEG)) < 1e-12);
        let between = angle_axis(&mul(&transpose(&ra), &rb)).0 / DEG;
        assert!(
            (between - fact_f64("G-E.near.apart").unwrap()).abs() < 1e-4,
            "{between}"
        );
        assert_eq!(format!("{between:.1}"), "1.4");
        // the big book has room in its rings
        let half = [
            BOOK_HALF[0] * BOOK_FIT,
            BOOK_HALF[1] * BOOK_FIT,
            BOOK_HALF[2] * BOOK_FIT,
        ];
        assert!(norm(half) < RADII[2], "the book fits inside the inner ring");
    }

    /// 13_4: the code panel's stopping rule is the facts' tolerances.
    #[test]
    fn the_stopping_rule_is_the_facts_tolerances() {
        let tol = fact("G-I.ik.tolerances").unwrap();
        assert_eq!(tol, "0.001,0.0001");
        let (spin, slide) = tol.split_once(',').unwrap();
        assert_eq!(format!("{}", slide.parse::<f64>().unwrap() * 1000.0), "0.1");
        assert_eq!(spin.parse::<f64>().unwrap(), model::TOL_SPIN);
        assert_eq!(slide.parse::<f64>().unwrap(), model::TOL_SLIDE);
    }

    /// The goal's marker is the goal's position and orientation.
    #[test]
    fn the_goal_frame_is_the_goal() {
        let arm = Arm::iiwa14();
        let goal = model::goal(&arm);
        let frame = goal_frame(&goal, 0.17);
        assert_eq!(frame.len(), 4);
        let Shape::Dot { at, .. } = &frame[0] else {
            panic!("a dot first");
        };
        assert_eq!(*at, goal.p);
        for (k, s) in frame[1..].iter().enumerate() {
            let Shape::Arrow { from, to, .. } = s else {
                panic!("an arrow");
            };
            let col = [goal.r[0][k], goal.r[1][k], goal.r[2][k]];
            assert!(norm(sub(sub(*to, *from), scale(col, 0.17))) < 1e-12);
        }
    }

    /// The points a list of shapes is drawn through.
    fn points_of(shapes: Vec<Shape>) -> Vec<V3> {
        let mut pts = Vec::new();
        for shape in shapes {
            match shape {
                Shape::Path { pts: p, .. } | Shape::Face { pts: p, .. } => pts.extend(p),
                Shape::Arrow { from, to, .. } => pts.extend([from, to]),
                Shape::Dashed { a, b, .. } => pts.extend([a, b]),
                Shape::Dot { at, .. } => pts.push(at),
            }
        }
        pts
    }

    /// The remaining error's screw at every iterate, with the axis and the
    /// curl and the slide arrow, stays inside the margin and above the lower
    /// third, which 13_3, 13_4 and 13_6 show over it.
    #[test]
    fn the_screws_stay_above_the_lower_third() {
        use super::super::draw::{LOWER_THIRD_TOP, SAFE};
        let arm = Arm::iiwa14();
        let goal = model::goal(&arm);
        let run = arm.solve(&model::degrees(poses::START), &goal, model::TOL_SPIN, model::TOL_SLIDE);
        let view = View::new(ARM_SCALE, v(ARM_AT[0], ARM_AT[1]));
        for (k, q) in run.path.iter().enumerate() {
            let s = model::screw_of(&goal.then(&arm.tip(q).undo()));
            let (lo, hi) = view.box_of(points_of(screw_shapes(&s, SCREW_REACH, 0.14)));
            assert!(
                lo[1] > LOWER_THIRD_TOP + 0.05,
                "iterate {k}'s screw reaches down to {:.2}",
                lo[1]
            );
            assert!(hi[1] < SAFE[1] && lo[0] > -SAFE[0] && hi[0] < SAFE[0], "{lo:?} {hi:?}");
        }
    }

    /// 13_2: the two gimbals' centres are the screen points GIMBALS names,
    /// so they stand at one height, and their captions clear the rings.
    #[test]
    fn the_gimbals_stand_level_with_their_captions_below() {
        let (right, up, _) = View::basis();
        let view = View::new(1.0, v(0.0, 0.0));
        let ys: Vec<f64> = GIMBALS
            .iter()
            .map(|c| {
                let at = add(scale(right, c[0]), scale(up, c[1]));
                let s = view.project(at);
                assert!((s[0] - c[0]).abs() < 1e-12, "the centre is where GIMBALS puts it");
                s[1]
            })
            .collect();
        assert!((ys[0] - ys[1]).abs() < 1e-12, "the two stand at one height");
    }

    #[test]
    fn the_quiz_and_the_table_fit_the_frame() {
        run(|stage| {
            let kit = Kit::new()?;
            let card = QuizCard::build(&kit, stage, ASK, QuizLook::at(v(CARD_AT[0], CARD_AT[1]), CARD_SCALE))?;
            let right = stage.get_right(card.back)[0];
            assert!(right < 7.1);
            let widest = kit.text_left(
                stage,
                &format!("{AFTER} 3: {} and {}", distance(0.03), turn_label(0.0076)),
                TABLE_PT,
                Palette::ink(),
                v(TABLE_X, 0.0),
            )?;
            assert!(stage.get_right(widest)[0] < 7.0, "a table row runs off the frame");
            let numbers = fact("G-I.error.twist")?.split(',').map(six).collect::<Vec<_>>();
            let row = kit.text(
                stage,
                &format!("{}   {}   {}", numbers[0], numbers[1], numbers[2]),
                34.0,
                Palette::ink(),
                v(BADGE_RUN_AT[0], 0.0),
            )?;
            assert!(
                stage.get_right(row)[0] < 7.0 && stage.get_left(row)[0] > 0.2,
                "the six numbers fit"
            );
            Ok(())
        });
    }
}
