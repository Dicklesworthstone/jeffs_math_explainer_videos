//! The thumbnail (one still, 1280×720): the video's opening question and the
//! machine that has to answer it. On the left, the book of 01_1 and 01_2 in
//! its two orders, tilt then roll and roll then tilt, each with the arrow out
//! of its cover: the covers face two ways, and a `≠` stands between them. On
//! the right, the arm of 01_5 (KUKA's LBR iiwa 14, `Arm::iiwa14`) with its
//! gripper at the pose 01_5 asks for (`model::goal`, solved by Newton's method
//! from the start of 13_5, `poses::START`), over the small book on its table,
//! its seven joints ringed in the screws' gold. Over both, the title in two
//! lines. No number is shown.

use std::f64::consts::FRAC_PI_2;

use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{ArmRig, BOOK_HALF, PartSpec, Pin, Shape, View, World, book, mix, still, table, turns_in_order};
use super::glyphs::TITLE;
use crate::model::{self, Arm, GOAL_P, M3, Pose, Q, V3, X, Y, degrees, poses};
use crate::palette;

/// The title's two lines: the break falls before "in Lie Groups".
const TITLE_BREAK: &str = " in ";
/// The title's height (both lines the same size) and the widest a line may be.
const TITLE_SIZE: f64 = 120.0;
const TITLE_MAX_W: f64 = 13.2;
const TITLE_Y: [f64; 2] = [3.05, 1.85];
/// The books' world: where its origin sits on the screen, and its scale.
const BOOKS_AT: [f64; 2] = [-3.7, -2.0];
const BOOKS_SCALE: f64 = 0.8;
/// The two copies' centres along x, and the height they are turned at
/// (chapter 01's `LIFT_Z`; the copies stand a little farther apart than
/// chapter 01's 3.5, to leave the sign room).
const BOOK_X: [f64; 2] = [-2.1, 2.1];
const LIFT_Z: f64 = 1.25;
const TABLE_LO: [f64; 2] = [-3.4, -1.6];
const TABLE_HI: [f64; 2] = [3.4, 1.6];
/// How long the arrow out of each cover is (world units).
const COVER_ARROW: f64 = 1.5;
/// The `≠` between the books: its height on the screen.
const NOT_EQUAL_H: f64 = 1.0;
/// The arm's world: its origin on the screen, and scene units per metre.
const ARM_AT: [f64; 2] = [3.25, -2.9];
const ARM_VIEW_SCALE: f64 = 4.1;
/// The arm's table (metres), round its foot and the small book.
const ARM_TABLE_LO: [f64; 2] = [-0.3, -0.3];
const ARM_TABLE_HI: [f64; 2] = [0.6, 0.55];
/// The small book under the gripper (chapter 01's, half sizes in metres).
const SMALL_BOOK: V3 = [0.07, 0.10, 0.016];
/// Where the pictures must stay below: under the title's second line.
#[cfg(test)]
const PICTURES_TOP: f64 = 1.2;

/// The thumbnail's scene, for the explainer's `thumbnail` field.
pub fn make(kit: Kit) -> Box<dyn SceneConstruct> {
    Box::new(Thumbnail { kit })
}

pub struct Thumbnail {
    pub kit: Kit,
}

/// The book's orientation after both quarter turns, about the room's lines
/// `first` then `second` (chapter 01's `turned` at the end of both turns).
fn after_both(first: V3, second: V3) -> M3 {
    turns_in_order(&[(first, FRAC_PI_2), (second, FRAC_PI_2)])
}

/// The two copies: tilt (about x) then roll (about y), and the other order.
fn orders() -> [M3; 2] {
    [after_both(X, Y), after_both(Y, X)]
}

/// The arm's joint angles with the gripper at 01_5's goal: Newton's method
/// from 13_5's start, as chapter 13 runs it.
fn at_the_goal(arm: &Arm) -> Q {
    let goal = model::goal(arm);
    arm.solve(&degrees(poses::START), &goal, model::TOL_SPIN, model::TOL_SLIDE)
        .q
}

/// An arrow out of the book's cover, along its own up (chapter 01's).
fn cover_arrow() -> Vec<Shape> {
    let up = BOOK_HALF[2];
    vec![Shape::Arrow {
        from: [0.0, 0.0, up],
        to: [0.0, 0.0, up + COVER_ARROW],
        color: mix(palette::turn(), Palette::ink(), 0.45),
        width: 7.0,
        head: 0.3,
    }]
}

/// The title's two lines.
fn title_lines() -> [&'static str; 2] {
    let at = TITLE.find(TITLE_BREAK).expect("the title has 'in'");
    [&TITLE[..at], &TITLE[at + 1..]]
}

impl SceneConstruct for Thumbnail {
    fn name(&self) -> &str {
        "thumbnail"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();

        // the books: one table, the two orders side by side
        let mut specs = vec![PartSpec::new(table(TABLE_LO, TABLE_HI, 0.0, 0.8), still(Pose::NOTHING)).layer(-2)];
        for (x, r) in BOOK_X.into_iter().zip(orders()) {
            let pose = Pose { r, p: [x, 0.0, LIFT_Z] };
            specs.push(PartSpec::new(book(BOOK_HALF), still(pose)));
            specs.push(PartSpec::new(cover_arrow(), still(pose)));
        }
        let books_view = View::new(BOOKS_SCALE, v(BOOKS_AT[0], BOOKS_AT[1]));
        let books = World::build(stage, books_view, 5.0, 0.2, specs, Vec::<Pin>::new())?;
        stage.set_tracker_value(books.shown, 1.0)?;
        books.add_to_scene(stage)?;

        // the sign between them, at the height of their centres
        let mid = books_view.project([0.0, 0.0, LIFT_Z]);
        let sign = kit.text_styled(stage, "≠", 150.0, palette::screw(), mid, true, false)?;
        let h = stage.get_height(sign);
        stage.scale(sign, NOT_EQUAL_H / h);
        stage.add_to_scene(sign)?;

        // the arm at the goal over the small book, its seven joints ringed in gold
        let arm = Arm::iiwa14();
        let q = at_the_goal(&arm);
        let (rig, _) = ArmRig::along(stage, arm, vec![q]);
        let mut specs = rig.parts(palette::steel());
        specs.push(PartSpec::new(table(ARM_TABLE_LO, ARM_TABLE_HI, 0.0, 0.15), still(Pose::NOTHING)).layer(-2));
        specs.push(PartSpec::new(
            book(SMALL_BOOK),
            still(Pose {
                r: model::I3,
                p: [GOAL_P[0], GOAL_P[1], SMALL_BOOK[2]],
            }),
        ));
        let arm_world = World::build(
            stage,
            View::new(ARM_VIEW_SCALE, v(ARM_AT[0], ARM_AT[1])),
            1.4,
            0.25,
            specs,
            Vec::<Pin>::new(),
        )?;
        stage.set_tracker_value(arm_world.shown, 1.0)?;
        arm_world.add_to_scene(stage)?;

        // the title, both lines the same size
        let mut lines = Vec::new();
        for (s, (y, color)) in title_lines()
            .into_iter()
            .zip(TITLE_Y.into_iter().zip([Palette::ink(), palette::screw()]))
        {
            lines.push(kit.text_styled(stage, s, TITLE_SIZE, color, v(0.0, y), true, false)?);
        }
        let widest = lines.iter().map(|&m| stage.get_width(m)).fold(0.0, f64::max);
        for m in lines {
            if widest > TITLE_MAX_W {
                stage.scale(m, TITLE_MAX_W / widest);
            }
            stage.add_to_scene(m)?;
        }

        // one second at the thumbnail's 1 fps: the one captured frame
        stage.wait(1.0)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{DEG, LIMITS_DEG, Z};

    /// Where a turned cover faces: the book's own up, carried by `r`.
    fn facing(r: &M3) -> V3 {
        [r[0][2], r[1][2], r[2][2]]
    }

    /// The two orders leave the covers facing two different ways, and those
    /// are the ways 01_2 says: tilt first, the cover faces you (-y, the
    /// viewer's side); roll first, the wall on your right (+x).
    #[test]
    fn the_two_orders_face_the_two_ways_01_2_says() {
        let [tilt_first, roll_first] = orders().map(|r| facing(&r));
        let near = |a: V3, b: V3| (0..3).all(|k| (a[k] - b[k]).abs() < 1e-9);
        assert!(near(tilt_first, [0.0, -1.0, 0.0]), "{tilt_first:?}");
        assert!(near(roll_first, [1.0, 0.0, 0.0]), "{roll_first:?}");
        assert!(!near(tilt_first, roll_first));
        // and neither still faces up: both turns did something
        assert!(!near(tilt_first, Z) && !near(roll_first, Z));
    }

    /// The arm's pose is the goal's: the tip is where 01_5 puts it, within
    /// the solver's tolerance, and the joints are inside KUKA's limits.
    #[test]
    fn the_gripper_is_at_the_goal_and_the_joints_within_their_limits() {
        let arm = Arm::iiwa14();
        let q = at_the_goal(&arm);
        let tip = arm.tip(&q);
        let goal = model::goal(&arm);
        let miss = (0..3).map(|k| (tip.p[k] - goal.p[k]).powi(2)).sum::<f64>().sqrt();
        assert!(miss < model::TOL_SLIDE, "the tip misses the goal by {miss} m");
        assert!(q.iter().zip(LIMITS_DEG).all(|(x, lim)| x.abs() / DEG <= lim), "{q:?}");
    }

    /// Both pictures stand inside the frame's safe area and under the title:
    /// the books' table, their corners and the tips of the cover arrows; the
    /// arm's table, its joints and its gripper.
    #[test]
    fn both_pictures_fit_the_frame_under_the_title() {
        use super::super::draw::SAFE;
        use crate::model::{add, apply};
        let corners = |lo: [f64; 2], hi: [f64; 2]| {
            [
                [lo[0], lo[1], 0.0],
                [lo[0], hi[1], 0.0],
                [hi[0], lo[1], 0.0],
                [hi[0], hi[1], 0.0],
            ]
        };
        let mut books: Vec<V3> = corners(TABLE_LO, TABLE_HI).to_vec();
        for (x, r) in BOOK_X.into_iter().zip(orders()) {
            let c = [x, 0.0, LIFT_Z];
            for sx in [-1.0, 1.0] {
                for sy in [-1.0, 1.0] {
                    for sz in [-1.0, 1.0] {
                        let corner = [sx * BOOK_HALF[0], sy * BOOK_HALF[1], sz * BOOK_HALF[2]];
                        books.push(add(c, apply(&r, corner)));
                    }
                }
            }
            books.push(add(c, apply(&r, [0.0, 0.0, BOOK_HALF[2] + COVER_ARROW])));
        }
        let arm = Arm::iiwa14();
        let q = at_the_goal(&arm);
        let mut reach: Vec<V3> = corners(ARM_TABLE_LO, ARM_TABLE_HI).to_vec();
        reach.extend((0..q.len()).map(|i| arm.joint(&q, i).0));
        reach.push(arm.tip(&q).p);
        for (name, view, pts) in [
            ("books", View::new(BOOKS_SCALE, v(BOOKS_AT[0], BOOKS_AT[1])), books),
            ("arm", View::new(ARM_VIEW_SCALE, v(ARM_AT[0], ARM_AT[1])), reach),
        ] {
            let (lo, hi) = view.box_of(pts);
            assert!(
                lo[0] > -SAFE[0] && hi[0] < SAFE[0] && lo[1] > -SAFE[1],
                "the {name} leave the frame: {lo:?} {hi:?}"
            );
            assert!(hi[1] < PICTURES_TOP, "the {name} reach {:.2}, into the title", hi[1]);
        }
    }

    #[test]
    fn the_title_breaks_into_two_lines_that_keep_every_word() {
        let [a, b] = title_lines();
        assert_eq!(format!("{a} {b}"), TITLE);
        assert!(b.starts_with("in "), "{b}");
    }
}
