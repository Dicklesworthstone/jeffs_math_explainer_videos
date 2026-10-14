//! Chapter 01, the book: two quarter turns in two orders leave the book
//! facing two ways (01_1, 01_2); the title (01_3); slides land on the same
//! spot and turns do not (01_4); the arm whose software has to do this
//! arithmetic (01_5); and the one line the video will earn, with the gripper
//! corkscrewing down along a gold screw (01_6).

use std::f64::consts::FRAC_PI_2;

use explainer_kit::props::{lower_third, precisely};
use explainer_kit::{Kit, Palette, Timed, play, tween, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::draw::{
    ARM_SCALE, ArmRig, Ask, BOOK_HALF, Choice, PartSpec, Pin, PoseFn, QuizCard, QuizLook, Shape, View, World,
    along_a_screw, book, fade_in_all, fade_in_lagged, leave, line_style, mix, pose_fn, say, screw_shapes, still, table,
    turns_in_order, tween_linear, value,
};
use super::glyphs::TITLE;
use crate::model::{self, Arm, GOAL_P, M3, Pose, Screw, V3, X, Y, add, degrees, poses, scale};
use crate::palette;

pub struct TheBook {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const SAME_OR_DIFFERENT: &str = "same or different?";
const SAME: &str = "same";
const DIFFERENT: &str = "different";
const BOOK_EDGES: &str = "turn about the book's own edges instead and the two answers trade places: still different";
const SAY_LEE: &str = "(say \"Lee\")";
const POSITIONS_ADD: &str = "positions add";
const ORIENTATIONS_DONT: &str = "orientations don't";
const THE_ARM: &str = "the arm:";
const ARM_NAME: &str = "KUKA LBR iiwa 14 R820";
const POSITION: &str = "position";
const ORIENTATION: &str = "orientation";
const POSE_IS_BOTH: &str = "pose = both";
/// The line the video will earn (chapter 08 writes it out term by term).
const THE_LINE: &str = r"T(q) = e^{[S_1]q_1} e^{[S_2]q_2} \cdots e^{[S_7]q_7} M";

/// The beats of the chapter, in order (storyboard ids). `01_3` is the title:
/// no narration, a listen window.
const BEATS: [&str; 6] = ["01_1", "01_2", "01_3", "01_4", "01_5", "01_6"];

/// 01_1's quiz: the card's words, and the choices with what the computed
/// answer calls them (`G-Q.01_1.answer`).
pub const ASK: Ask = Ask {
    line: BEATS[0],
    question: SAME_OR_DIFFERENT,
    choices: &[
        Choice {
            key: "same",
            label: SAME,
        },
        Choice {
            key: "different",
            label: DIFFERENT,
        },
    ],
};

/// The left copy's place on the table; the right copy stands `SEPARATION`
/// along x until chapter 01's slides bring it onto the left one.
const X_LEFT: f64 = -1.75;
const SEPARATION: f64 = 3.5;
/// The height the book is turned at (it rests at its own half thickness).
const LIFT_Z: f64 = 1.25;
/// The slides of 01_4: right, then away (or the other way round).
const SLIDE_X: f64 = 3.2;
const SLIDE_Y: f64 = 1.5;
const TABLE_LO: [f64; 2] = [-3.0, -2.0];
const TABLE_HI: [f64; 2] = [3.0, 2.8];
// the slid book stays on the table
const _: () = assert!(X_LEFT + SLIDE_X < TABLE_HI[0] && SLIDE_Y + BOOK_HALF[1] < TABLE_HI[1]);
/// How far each gold line of the room runs from the book's centre.
const LINE_REACH: f64 = 2.3;
/// Where the books' world sits on the screen: the table and the gold lines
/// end inside the safe area on the left and clear of the quiz card (centred
/// at `CARD_AT`) on the right.
const BOOKS_AT: [f64; 2] = [-3.1, -0.9];
const CARD_AT: [f64; 2] = [4.35, 0.3];
const CARD_SCALE: f64 = 0.92;
/// The arm's world: its origin on the screen, clear of the lower third.
const ARM_AT: [f64; 2] = [-1.9, -1.9];
/// How far above the floor the screw's gold axis runs.
const SCREW_TOP: f64 = 1.3;

/// The book's orientation after its two turns, `a` and `b` of the way
/// through the first and second (0 to 1, each a quarter turn), about the
/// room's lines `first` and `second`.
fn turned(first: V3, second: V3, a: f64, b: f64) -> M3 {
    turns_in_order(&[(first, a * FRAC_PI_2), (second, b * FRAC_PI_2)])
}

fn smooth(x: f64) -> f64 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// One copy of the book: the room's two lines it turns about, in order, and
/// the trackers that move it (how far through each turn, how far along each
/// slide, and for the right copy how far apart from the left one).
#[derive(Clone, Copy)]
struct Book {
    first: V3,
    second: V3,
    a: Mob,
    b: Mob,
    sx: Mob,
    sy: Mob,
    apart: Option<Mob>,
}

impl Book {
    /// Where the book's centre is: it rises to turn height as its first turn
    /// begins, and slides over the table at rest height.
    fn centre(&self, s: &Arena) -> V3 {
        let lift = smooth(value(s, self.a).max(value(s, self.b)) / 0.2);
        let apart = self.apart.map_or(0.0, |t| value(s, t));
        [
            X_LEFT + SEPARATION * apart + SLIDE_X * value(s, self.sx),
            SLIDE_Y * value(s, self.sy),
            BOOK_HALF[2] + (LIFT_Z - BOOK_HALF[2]) * lift,
        ]
    }

    fn pose(self) -> PoseFn {
        pose_fn(move |s| Pose {
            r: turned(self.first, self.second, value(s, self.a), value(s, self.b)),
            p: self.centre(s),
        })
    }

    /// The room's lines through the book's centre: they do not turn.
    fn lines_pose(self) -> PoseFn {
        pose_fn(move |s| Pose {
            r: model::I3,
            p: self.centre(s),
        })
    }
}

/// The two gold lines of the room through a book's centre, each in two
/// halves so that the half behind the book is hidden by it.
fn room_lines() -> Vec<Shape> {
    let gold = palette::screw();
    let mut out = Vec::new();
    for dir in [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]] {
        for sign in [-1.0, 1.0] {
            let end = [dir[0] * sign * LINE_REACH, dir[1] * sign * LINE_REACH, 0.0];
            out.push(Shape::line([0.0; 3], end, gold, 3.5));
        }
    }
    out
}

/// An arrow out of the book's cover, along its own up.
fn cover_arrow() -> Vec<Shape> {
    let up = BOOK_HALF[2];
    vec![Shape::Arrow {
        from: [0.0, 0.0, up],
        to: [0.0, 0.0, up + 1.5],
        color: mix(palette::turn(), Palette::ink(), 0.45),
        width: 5.0,
        head: 0.26,
    }]
}

/// The part order of the book world (parts 0 and 1 are the table and the
/// left copy, which are never singled out).
mod part {
    pub const RIGHT: usize = 2;
    pub const LEFT_LINES: usize = 3;
    pub const RIGHT_LINES: usize = 4;
    pub const LEFT_ARROW: usize = 5;
    pub const RIGHT_ARROW: usize = 6;
    pub const LEFT_TRAIL: usize = 7;
    pub const RIGHT_TRAIL: usize = 8;
    pub const LANDING: usize = 9;
}

/// A tick (green) or a cross (red), as strokes and never a glyph, centred at
/// `at`: ticks and equals signs are the only green, and a wrong path is always
/// crossed.
fn mark(stage: &mut Stage<'_>, ok: bool, at: Vec3) -> fmn::Result<Mob> {
    let r = 0.2;
    if ok {
        let tick = polyline(vec![
            v(at[0] - 0.9 * r, at[1]),
            v(at[0] - 0.25 * r, at[1] - 0.7 * r),
            v(at[0] + r, at[1] + 0.8 * r),
        ])
        .style(line_style(palette::good(), 6.0))
        .build();
        return Ok(stage.arena_mut().add(tick));
    }
    let a = polyline(vec![v(at[0] - r, at[1] - r), v(at[0] + r, at[1] + r)])
        .style(line_style(palette::warn(), 6.0))
        .build();
    let b = polyline(vec![v(at[0] - r, at[1] + r), v(at[0] + r, at[1] - r)])
        .style(line_style(palette::warn(), 6.0))
        .build();
    let (a, b) = (stage.arena_mut().add(a), stage.arena_mut().add(b));
    super::draw::group(stage, &[a, b])
}

/// A column of a matrix: where its `k`th axis points.
fn column(r: &M3, k: usize) -> V3 {
    [r[0][k], r[1][k], r[2][k]]
}

/// The screw with its point moved along its axis to half of `top` above the
/// floor: drawn `top / 2` out each way, the gold axis then runs from the floor
/// up to `top`, where the screw's own point (the axis' nearest to the origin)
/// would have put half of it below the table.
fn from_the_floor(s: &Screw, top: f64) -> Screw {
    let up = if s.axis[2] >= 0.0 { s.axis } else { scale(s.axis, -1.0) };
    if up[2] < 1e-9 {
        return *s;
    }
    let along = (0.5 * top - s.point[2]) / up[2];
    Screw {
        point: add(s.point, scale(up, along)),
        ..*s
    }
}

impl SceneConstruct for TheBook {
    fn name(&self) -> &str {
        "01_the_book"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();

        // ---- the books on the table: two copies, each with its own trackers
        let [la, lb, lsx, lsy, ra, rb, rsx, rsy] = std::array::from_fn(|_| stage.add_value_tracker(0.0));
        let apart = stage.add_value_tracker(1.0);
        let left = Book {
            first: X,
            second: Y,
            a: la,
            b: lb,
            sx: lsx,
            sy: lsy,
            apart: None,
        };
        let right = Book {
            first: Y,
            second: X,
            a: ra,
            b: rb,
            sx: rsx,
            sy: rsy,
            apart: Some(apart),
        };
        let z = 0.03;
        let teal = palette::place();
        let trail = |via: [f64; 2]| {
            vec![Shape::Path {
                pts: vec![[X_LEFT, 0.0, z], [via[0], via[1], z], [X_LEFT + SLIDE_X, SLIDE_Y, z]],
                color: teal,
                width: 5.0,
                closed: false,
                bias: 0.0,
            }]
        };
        let specs = vec![
            PartSpec::new(table(TABLE_LO, TABLE_HI, 0.0, 0.8), still(Pose::NOTHING)).layer(-2),
            PartSpec::new(book(BOOK_HALF), left.pose()),
            PartSpec::new(book(BOOK_HALF), right.pose()),
            PartSpec::new(room_lines(), left.lines_pose()),
            PartSpec::new(room_lines(), right.lines_pose()),
            PartSpec::new(cover_arrow(), left.pose()),
            PartSpec::new(cover_arrow(), right.pose()),
            PartSpec::new(trail([X_LEFT + SLIDE_X, 0.0]), still(Pose::NOTHING)).layer(-1),
            PartSpec::new(trail([X_LEFT, SLIDE_Y]), still(Pose::NOTHING)).layer(-1),
            PartSpec::new(
                vec![Shape::Dot {
                    at: [X_LEFT + SLIDE_X, SLIDE_Y, z],
                    color: teal,
                    radius: 0.2,
                }],
                still(Pose::NOTHING),
            )
            .layer(-1),
        ];
        let books = World::build(
            stage,
            View::new(1.0, v(BOOKS_AT[0], BOOKS_AT[1])),
            5.0,
            0.2,
            specs,
            Vec::<Pin>::new(),
        )?;
        // the right copy is greyed until it is turned; the arrows, trails and
        // the landing dot are not there yet
        stage.set_tracker_value(books.parts[part::RIGHT].grey, 1.0)?;
        for i in [
            part::LEFT_ARROW,
            part::RIGHT_ARROW,
            part::LEFT_TRAIL,
            part::RIGHT_TRAIL,
            part::LANDING,
        ] {
            stage.set_tracker_value(books.parts[i].shown, 0.0)?;
        }
        books.add_to_scene(stage)?;

        // 01_1: tilt then roll; swap the order; the same or different?
        let line = say(&kit, stage, BEATS[0])?;
        let card = QuizCard::build(&kit, stage, ASK, QuizLook::at(v(CARD_AT[0], CARD_AT[1]), CARD_SCALE))?;
        books.show(stage, 0.8)?;
        line.wait_for(stage, "Tilt a book")?;
        play!(stage; tween(la, 1.0, 1.3)?);
        line.wait_for(stage, "Then roll it")?;
        play!(stage; tween(lb, 1.0, 1.3)?);
        line.wait_last(stage)?;
        card.show(stage, 0.5)?;
        card.count(&kit, stage)?;

        // 01_2: different. Tilt first, the cover faces you; roll first, it
        // faces the wall on your right
        let line = say(&kit, stage, BEATS[1])?;
        line.wait_for(stage, "Tilt first")?;
        play!(stage; tween(books.parts[part::LEFT_ARROW].shown, 1.0, 0.5)?);
        line.wait_for(stage, "Roll first")?;
        // the exact version comes in with the second demonstration, so that
        // it has the rest of the line (about four seconds) to be read
        let edges = precisely(&kit, stage, BOOK_EDGES)?;
        play!(stage;
            tween(books.parts[part::RIGHT].grey, 0.0, 0.4)?,
            tween(ra, 1.0, 1.2)?,
            fade_in(stage.arena_mut(), edges, ORIGIN, 1.0)?.rt(0.5),
        );
        play!(stage; tween(rb, 1.0, 1.2)?);
        line.wait_for(stage, "it faces the wall")?;
        play!(stage; tween(books.parts[part::RIGHT_ARROW].shown, 1.0, 0.5)?);

        // 01_3: the title, then the motif alone
        kit.hold(stage)?;
        let mut leaving = card.mobs();
        leaving.push(edges);
        leave(stage, &leaving, &[books.shown], 0.6)?;
        let title = kit.text(stage, TITLE, 62.0, Palette::ink(), v(0.0, 0.5))?;
        let lee = kit.text(stage, SAY_LEE, 28.0, Palette::dim(), v(0.0, -0.45))?;
        fade_in_all(stage, &[title, lee], 0.6)?;
        kit.listen(stage, 3.0, &format!("01_the_book:{}", BEATS[2]))?;
        leave(stage, &[title, lee], &[], 0.5)?;

        // 01_4: slides land on the same spot; turns do not
        for t in [la, lb, ra, rb] {
            stage.set_tracker_value(t, 0.0)?;
        }
        stage.set_tracker_value(books.parts[part::RIGHT].grey, 0.0)?;
        for i in [part::LEFT_ARROW, part::RIGHT_ARROW, part::LEFT_LINES, part::RIGHT_LINES] {
            stage.set_tracker_value(books.parts[i].shown, 0.0)?;
        }
        let line = say(&kit, stage, BEATS[3])?;
        books.show(stage, 0.6)?;
        play!(stage; tween(apart, 0.0, 1.0)?);
        line.wait_for(stage, "Slide the book right")?;
        play!(stage; tween(books.parts[part::LEFT_TRAIL].shown, 1.0, 0.3)?);
        play!(stage; tween(lsx, 1.0, 1.1)?);
        play!(stage; tween(lsy, 1.0, 0.9)?);
        line.wait_for(stage, "or away, then right")?;
        play!(stage; tween(books.parts[part::RIGHT_TRAIL].shown, 1.0, 0.3)?);
        play!(stage; tween(rsy, 1.0, 0.9)?);
        play!(stage; tween(rsx, 1.0, 1.1)?);
        line.wait_for(stage, "it lands on the same spot")?;
        play!(stage; tween(books.parts[part::LANDING].shown, 1.0, 0.4)?);
        line.wait_for(stage, "So positions add")?;
        let adds = kit.text_left(stage, POSITIONS_ADD, 38.0, Palette::ink(), v(2.6, 0.9))?;
        let adds_mark = mark(stage, true, v(2.1, 0.9))?;
        fade_in_all(stage, &[adds, adds_mark], 0.4)?;
        line.wait_for(stage, "and orientations don't")?;
        let dont = kit.text_left(stage, ORIENTATIONS_DONT, 38.0, Palette::ink(), v(2.6, -0.1))?;
        let dont_mark = mark(stage, false, v(2.1, -0.1))?;
        fade_in_all(stage, &[dont, dont_mark], 0.4)?;

        // 01_5: the arm, and what it has to do
        kit.hold(stage)?;
        leave(stage, &[adds, adds_mark, dont, dont_mark], &[books.shown], 0.6)?;
        let line = say(&kit, stage, BEATS[4])?;
        let arm = Arm::iiwa14();
        let q_start = degrees(poses::START);
        let start = arm.tip(&q_start);
        let goal = model::goal(&arm);
        let screw = model::screw_of(&goal.then(&start.undo()));
        let keys = along_a_screw(&arm, &q_start, &start, &screw, 24);
        let (rig, walk) = ArmRig::along(stage, arm.clone(), keys);
        let mut specs = rig.parts(palette::steel());
        specs.push(PartSpec::new(table([-0.9, -0.7], [1.0, 0.9], 0.0, 0.2), still(Pose::NOTHING)).layer(-2));
        let small = [0.07, 0.10, 0.016];
        specs.push(PartSpec::new(
            book(small),
            still(Pose {
                r: model::I3,
                p: [GOAL_P[0], GOAL_P[1], small[2]],
            }),
        ));
        let pink = palette::goal();
        let mut frame = vec![Shape::Dot {
            at: goal.p,
            color: pink,
            radius: 0.13,
        }];
        for k in 0..3 {
            let c = column(&goal.r, k);
            frame.push(Shape::Arrow {
                from: goal.p,
                to: [
                    goal.p[0] + 0.17 * c[0],
                    goal.p[1] + 0.17 * c[1],
                    goal.p[2] + 0.17 * c[2],
                ],
                color: pink,
                width: 5.0,
                head: 0.2,
            });
        }
        let goal_at = specs.len();
        specs.push(PartSpec::new(frame, still(Pose::NOTHING)).layer(2));
        let screw_at = specs.len();
        specs.push(
            PartSpec::new(
                screw_shapes(&from_the_floor(&screw, SCREW_TOP), 0.5 * SCREW_TOP, 0.16),
                still(Pose::NOTHING),
            )
            .layer(2),
        );
        let arm_view = View::new(ARM_SCALE, v(ARM_AT[0], ARM_AT[1]));
        let world = World::build(stage, arm_view, 1.4, 0.25, specs, Vec::<Pin>::new())?;
        let hidden: Vec<usize> = ArmRig::GOLD_COLLARS.chain([goal_at, screw_at]).collect();
        for i in hidden {
            stage.set_tracker_value(world.parts[i].shown, 0.0)?;
        }
        world.add_to_scene(stage)?;
        line.wait_for(stage, "This is a KUKA robot arm")?;
        world.show(stage, 0.8)?;
        let name = lower_third(&kit, stage, THE_ARM, ARM_NAME)?;
        play!(stage; fade_in(stage.arena_mut(), name, ORIGIN, 1.0)?.rt(0.5));
        line.wait_for(stage, "Its job is to put")?;
        play!(stage; tween(world.parts[goal_at].shown, 1.0, 0.7)?);
        line.wait_for(stage, "a position and an orientation")?;
        let at = arm_view.project(GOAL_P);
        let words = [
            (POSITION, palette::place(), v(at[0] + 0.9, at[1] + 0.55)),
            (ORIENTATION, palette::turn(), v(at[0] + 1.15, at[1] - 0.25)),
            (POSE_IS_BOTH, Palette::dim(), v(at[0] + 1.0, at[1] - 0.95)),
        ];
        let mut labels = Vec::new();
        for (text, color, to) in words {
            labels.push(kit.text_left(stage, text, 32.0, color, to)?);
        }
        fade_in_lagged(stage, &labels, 1.2, 0.5)?;

        // 01_6: so the software multiplies; the line it will earn
        let line = say(&kit, stage, BEATS[5])?;
        play!(stage; tween(world.parts[screw_at].shown, 1.0, 0.6)?);
        play!(stage; tween_linear(walk, 1.0, 4.4)?);
        line.wait_for(stage, "The way out")?;
        let equation = kit.tex(stage, THE_LINE, 52.0, Palette::dim(), &[], v(1.4, 3.3))?;
        let writing = write(stage.arena(), equation).rt(2.6);
        stage.play(writing)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::{CARD_W, LOWER_THIRD_TOP, SAFE, fact};
    use super::*;
    use crate::model::{DEG, Z};

    /// 01_1, 01_2: the table and the room's gold lines, projected, end inside
    /// the safe area on the left (they once ran off the frame) and clear of
    /// the quiz card on the right.
    #[test]
    fn the_books_and_their_lines_fit_beside_the_card() {
        let view = View::new(1.0, v(BOOKS_AT[0], BOOKS_AT[1]));
        let mut pts = Vec::new();
        for x in [TABLE_LO[0], TABLE_HI[0]] {
            for y in [TABLE_LO[1], TABLE_HI[1]] {
                pts.push([x, y, 0.0]);
            }
        }
        // the lines of both copies, through their centres at turn height
        for cx in [X_LEFT, X_LEFT + SEPARATION] {
            for s in [-1.0, 1.0] {
                pts.push([cx + s * LINE_REACH, 0.0, LIFT_Z]);
                pts.push([cx, s * LINE_REACH, LIFT_Z]);
            }
        }
        let (lo, hi) = view.box_of(pts);
        assert!(lo[0] > -SAFE[0] && hi[1] < SAFE[1] && lo[1] > -SAFE[1], "{lo:?} {hi:?}");
        let card_left = CARD_AT[0] - 0.5 * CARD_W * CARD_SCALE;
        assert!(
            hi[0] < card_left - 0.5,
            "the table ends at {:.2}, the card begins at {card_left:.2}",
            hi[0]
        );
        // the slid book is still on that table
        let landing = view.box_of([[X_LEFT + SLIDE_X, SLIDE_Y, 0.0]]);
        assert!(landing.0[0] > lo[0] && landing.1[0] < hi[0]);
    }

    /// 01_5, 01_6: the arm's world, its table, its screw and the labels at the
    /// goal stand clear of the lower third and inside the frame, and the
    /// gold axis runs from the floor up, never below the table.
    #[test]
    fn the_arm_scene_clears_the_lower_third() {
        let arm = Arm::iiwa14();
        let view = View::new(ARM_SCALE, v(ARM_AT[0], ARM_AT[1]));
        let start = arm.tip(&degrees(poses::START));
        let goal = model::goal(&arm);
        let screw = model::screw_of(&goal.then(&start.undo()));
        let drawn = from_the_floor(&screw, SCREW_TOP);
        let axis: Vec<V3> = screw_shapes(&drawn, 0.5 * SCREW_TOP, 0.16)
            .iter()
            .flat_map(Shape::points)
            .collect();
        let (lo, hi) = view.box_of(axis.iter().copied());
        assert!(lo[1] > LOWER_THIRD_TOP, "the axis' foot is at {:.2}", lo[1]);
        assert!(hi[1] < SAFE[1] && lo[0] > -SAFE[0] && hi[0] < SAFE[0]);
        assert!(
            axis.iter().all(|p| p[2] > -1e-9),
            "no part of the axis is below the floor"
        );
        // the axis still is the screw's: parallel to it, through the same line
        let along = model::sub(drawn.point, screw.point);
        assert!(model::norm(model::cross(along, screw.axis)) < 1e-9);
        assert_eq!((drawn.angle, drawn.slide), (screw.angle, screw.slide));
        // the table of the arm's world and the arm's own corners
        let floor = view.box_of([[-0.9, -0.7, 0.0], [1.0, -0.7, 0.0], [1.0, 0.9, 0.0], [-0.9, 0.9, 0.0]]);
        assert!(
            floor.0[1] > LOWER_THIRD_TOP,
            "the table's front corner is at {:.2}",
            floor.0[1]
        );
        let tip = view.box_of([GOAL_P]);
        assert!(
            tip.0[1] - 0.95 > LOWER_THIRD_TOP,
            "the last label stands at {:.2}",
            tip.0[1] - 0.95
        );
    }

    /// Chapter 01 plays exactly the storyboard's beats, in its order, and
    /// every spoken one is the script's own line.
    #[test]
    fn the_beats_are_the_storyboards_and_the_scripts() {
        let board = explainer_kit::storyboard::parse(crate::STORYBOARD).unwrap();
        let chapter = board.iter().find(|c| c.key == "01_the_book").expect("chapter 01");
        let ids: Vec<&str> = chapter.beats.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(ids, BEATS);
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for beat in &chapter.beats {
            match beat.spoken() {
                Some(words) => assert_eq!(script.text(&beat.id), Some(words.as_str()), "{}", beat.id),
                None => assert_eq!(beat.id, "01_3", "only the title is silent"),
            }
        }
    }

    /// The quiz: a countdown of the storyboard's seconds, one choice that is
    /// the computed answer, and the book's two orders really do differ.
    #[test]
    fn the_quiz_asks_what_the_book_answers() {
        assert_eq!(ASK.seconds().unwrap(), 5);
        assert_eq!(ASK.answer_index().unwrap(), 1, "different");
        assert_eq!(fact("G-B.turns.same").unwrap(), "0");
        let (tw, wt) = (model::tilt_then_roll(FRAC_PI_2), model::roll_then_tilt(FRAC_PI_2));
        assert!(
            model::gap(&tw, &wt) >= 1.0 - 1e-12,
            "the two orders end in orientations that differ in a whole entry of the grid"
        );
    }

    /// 01_2: after tilt then roll the cover faces the viewer (0, -1, 0); after
    /// roll then tilt it faces the right (1, 0, 0): the book the scene turns,
    /// not just the table of facts.
    #[test]
    fn the_covers_end_facing_you_and_the_right() {
        let r = turned(X, Y, 1.0, 1.0);
        assert!(model::norm(model::sub(model::apply(&r, Z), [0.0, -1.0, 0.0])) < 1e-12);
        assert_eq!(fact("G-B.tilt_then_roll.cover").unwrap(), "0,-1,0");
        let r = turned(Y, X, 1.0, 1.0);
        assert!(model::norm(model::sub(model::apply(&r, Z), [1.0, 0.0, 0.0])) < 1e-12);
        assert_eq!(fact("G-B.roll_then_tilt.cover").unwrap(), "1,0,0");
        // half way through the first turn the book is half way
        let half = turned(X, Y, 0.5, 0.0);
        assert!(model::gap(&half, &model::turn(X, 45.0 * DEG)) < 1e-12);
    }

    /// 01_4: the two slides, in either order, land on one spot (positions
    /// add), the two turns do not.
    #[test]
    fn slides_land_on_one_spot_and_turns_do_not() {
        let right = [SLIDE_X, 0.0, 0.0];
        let away = [0.0, SLIDE_Y, 0.0];
        let (a, b) = (model::add(right, away), model::add(away, right));
        assert_eq!(a, b);
        assert_eq!(fact("G-B.slides.same").unwrap(), "1");
        // and the landing dot of the picture is that spot
        assert_eq!(a, [SLIDE_X, SLIDE_Y, 0.0]);
    }

    /// 01_6: the arm's tip follows the screw from its start pose to the goal:
    /// the first sample is the start, the last the goal, and every sample
    /// stands on the screw's path within a millimetre.
    #[test]
    fn the_gripper_corkscrews_from_start_to_goal() {
        let arm = Arm::iiwa14();
        let q0 = degrees(poses::START);
        let start = arm.tip(&q0);
        let goal = model::goal(&arm);
        let screw = model::screw_of(&goal.then(&start.undo()));
        assert!((screw.angle / DEG - 90.0).abs() < 1e-6, "a quarter turn");
        let n = 24;
        let keys = along_a_screw(&arm, &q0, &start, &screw, n);
        assert_eq!(keys.len(), n + 1);
        assert_eq!(keys[0], q0);
        for (k, q) in keys.iter().enumerate() {
            let want = model::along_screw(&screw, k as f64 / n as f64).then(&start);
            let got = arm.tip(q);
            assert!(
                model::norm(model::sub(got.p, want.p)) < 1e-3,
                "sample {k} is {:.2} mm off the screw",
                1000.0 * model::norm(model::sub(got.p, want.p))
            );
            assert!(model::gap(&got.r, &want.r) < 1e-2, "sample {k} turned wrong");
        }
        let end = arm.tip(keys.last().unwrap());
        assert!(model::norm(model::sub(end.p, goal.p)) < 1e-3);
    }

    /// 01_6: "about a hundred and fifty years old" is Lie's 1873 (the facts'
    /// `G-L.lie.year`) read from 2026, to the nearest ten years.
    #[test]
    fn about_a_hundred_and_fifty_years() {
        let year: i32 = fact("G-L.lie.year").unwrap().parse().unwrap();
        let age = 2026 - year;
        assert!((age - 150).abs() <= 5, "{age} years");
    }
}
