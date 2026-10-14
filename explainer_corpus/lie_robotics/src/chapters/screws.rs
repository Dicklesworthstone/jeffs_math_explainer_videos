//! Chapter 07, screws: leave the floor, and a pose can change in six ways
//! (07_1, 07_2); a velocity in space is a twist, and held steady it is a
//! screw (07_3); a door, a drawer, a bottle cap (07_4); any displacement of a
//! rigid body is one screw (07_5); and who found it (07_6).

use explainer_kit::animatic::wrap;
use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, Timed, play, tween, v};
use fmn::prelude::*;

use super::draw::{
    Ask, BOOK_HALF, Choice, PartSpec, Pin, QuizCard, QuizLook, Shape, VIEW_AZIMUTH_DEG, View, World, cuboid, fact,
    fact_f64, fade_in_all, four_words, leave, mix, plate, pose_fn, say, screen_entry, screw_shapes, spin_arrow, still,
    table, tag, tween_linear, value,
};
use crate::facts::shown::{TOSS_SLIDE, TOSS_TURN_DEG};
use crate::model::{
    self, CUBE_AXES, DEG, M3, Pose, SWEEP_DEG, V3, Z, add, along_screw, dot, log6, norm, scale, screw_of, sub, turn,
    unit,
};
use crate::palette;

pub struct Screws {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const TWIST: &str = "twist";
const SPIN_WORD: &str = "spin";
const SLIDE_WORD: &str = "slide";
const DOOR: &str = "door";
const DRAWER: &str = "drawer";
const BOTTLE_CAP: &str = "bottle cap";
const TURN_WORD: &str = "turn";
const SLIDE_LABEL: &str = "slide";
const METRES: &str = "m";
const DEGREE: &str = "°";
const FOR_PROGRAMMERS: &str = "for programmers";
/// 07_2's tag (R2 C-05): what the slide arrow of a twist is, in few words.
const TWIST_NOTE: &str = "a twist's slide arrow is the velocity of the point at the room's origin";
/// The tag's text wraps at this many characters; it stands in the right
/// column, under the six numbers (its centre).
const TAG_WRAP: usize = 30;
const TAG_AT: [f64; 2] = [4.6, -1.5];
const TAG_PT: f64 = 22.0;
const DIFFERENT_POSES: &str = "different poses,";
const SCREWS: &str = "screws";
const MOZZI_NOTE: &str = "(for motion at an instant)";
const CHASLES_NOTE: &str = "every displacement of a rigid body is a screw displacement";

/// The beats of the chapter, in order (storyboard ids).
const BEATS: [&str; 6] = ["07_1", "07_2", "07_3", "07_4", "07_5", "07_6"];

/// The words of 07_2's tag: the header, then the note on lines of at most
/// `TAG_WRAP` characters.
fn twist_tag_words() -> String {
    let mut lines = vec![FOR_PROGRAMMERS.to_owned()];
    lines.extend(wrap(TWIST_NOTE, TAG_WRAP, usize::MAX));
    lines.join("\n")
}

/// 07_1's quiz: in how many ways can a pose change in space? The keys are
/// the number as `G-Q.07_1.answer` prints it.
pub const ASK: Ask = Ask {
    line: BEATS[0],
    question: "how many ways can\na pose change?",
    choices: &[
        Choice { key: "3", label: "3" },
        Choice { key: "6", label: "6" },
        Choice { key: "9", label: "9" },
    ],
};

/// The platform's six-second path (chapter 06's numbers): half a metre a
/// second at thirty degrees a second, in metres.
const PATH_SPEED: f64 = 0.5;
const PATH_SECONDS: f64 = 6.0;
/// How far the helix rises over that path: the slide added in 07_3.
const HELIX_RISE: f64 = 1.5;
/// The book's rest place on the floor (metres) and how high it rises.
const BOOK_AT: V3 = [1.9, 0.9, 0.04];
const BOOK_RISE: f64 = 0.9;
const FLOOR_LO: [f64; 2] = [-1.2, -1.0];
const FLOOR_HI: [f64; 2] = [3.4, 2.4];

/// How the floor lies at `t` (0 face-on to the viewer, as in chapter 06's top
/// view; 1 flat in the fixed view): a turn about the screen's horizontal and
/// a turn about the vertical, each taken `1 − t` of the way back.
fn floor_turn(t: f64) -> M3 {
    let (_, _, toward) = View::basis();
    let tilt_axis = unit(model::cross(Z, toward));
    let tilt = dot(Z, toward).clamp(-1.0, 1.0).acos();
    let yaw = VIEW_AZIMUTH_DEG * DEG;
    let back = 1.0 - t;
    model::mul(&turn(tilt_axis, back * tilt), &turn(Z, back * yaw))
}

/// The twist of a motion held for one second: the spin arrow and the slide
/// arrow (six numbers).
fn toss_pose() -> Pose {
    Pose {
        r: turn(Z, TOSS_TURN_DEG * DEG),
        p: TOSS_SLIDE,
    }
}

/// A door swinging `angle` about a vertical hinge at `hinge`, as the motion
/// of the room: a screw with no slide.
fn door_motion(hinge: V3, angle: f64) -> Pose {
    let r = turn(Z, angle);
    Pose {
        r,
        p: sub(hinge, model::apply(&r, hinge)),
    }
}

/// A drawer pulled `d` along x: a screw with no turn.
fn drawer_motion(d: f64) -> Pose {
    Pose {
        r: model::I3,
        p: [d, 0.0, 0.0],
    }
}

/// A cap turned `angle` about a vertical line through `at` as it rises `h`:
/// a screw that does both.
fn cap_motion(at: V3, angle: f64, h: f64) -> Pose {
    let r = turn(Z, angle);
    Pose {
        r,
        p: add(sub(at, model::apply(&r, at)), [0.0, 0.0, h]),
    }
}

/// The turns of the wall of books: the thirteen axes of a cube's faces, edges
/// and corners at the five angles that are not nothing, and no turn once:
/// sixty-six different turns (`G-X3.one_screw` counts them four slides each).
fn wall_turns() -> Vec<(V3, f64)> {
    let mut out = vec![(Z, 0.0)];
    for axis in CUBE_AXES {
        for &deg in &SWEEP_DEG[1..] {
            out.push((unit(axis), deg));
        }
    }
    out
}

/// The part order of the floor world.
mod floor {
    pub const PLATFORM: usize = 1;
    pub const ARROWS: usize = 2;
    pub const ARC: usize = 3;
    pub const BOOK: usize = 5;
    pub const AXIS: usize = 6;
    pub const HELIX: usize = 7;
    pub const TWIST: usize = 8;
}

/// The platform's body and wheels, flat on the floor, in metres.
fn platform_faces() -> Vec<Shape> {
    let z = 0.012;
    let body = vec![[0.40, 0.21, z], [-0.40, 0.21, z], [-0.40, -0.21, z], [0.40, -0.21, z]];
    let mut out = vec![Shape::Face {
        pts: body,
        fill: mix(palette::steel(), super::draw::ground(), 0.55),
        edge: Some((palette::steel(), 2.5)),
        shaded: false,
        bias: 0.0,
    }];
    for (sf, sl) in [(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)] {
        let (cf, cl) = (sf * 0.25, sl * 0.28);
        out.push(Shape::Face {
            pts: vec![
                [cf + 0.1, cl + 0.045, z],
                [cf - 0.1, cl + 0.045, z],
                [cf - 0.1, cl - 0.045, z],
                [cf + 0.1, cl - 0.045, z],
            ],
            fill: palette::steel(),
            edge: None,
            shaded: false,
            bias: 0.01,
        });
    }
    out
}

/// The platform's three velocity arrows on the floor: forward, sideways and
/// the curl of the spin.
fn floor_arrows() -> Vec<Shape> {
    let z = 0.03;
    let orange = palette::flat();
    let mut out = vec![
        Shape::Arrow {
            from: [0.0, 0.0, z],
            to: [0.55, 0.0, z],
            color: orange,
            width: 5.0,
            head: 0.2,
        },
        Shape::Arrow {
            from: [0.0, 0.0, z],
            to: [0.0, 0.55, z],
            color: orange,
            width: 5.0,
            head: 0.2,
        },
    ];
    let arc = super::draw::circle_points([0.0, 0.0, z], Z, 0.22, 0.6, 5.0, 16);
    let n = arc.len();
    out.push(Shape::Path {
        pts: arc[..n - 1].to_vec(),
        color: orange,
        width: 4.0,
        closed: false,
        bias: 0.0,
    });
    out.push(Shape::Arrow {
        from: arc[n - 2],
        to: arc[n - 1],
        color: orange,
        width: 4.0,
        head: 0.18,
    });
    out
}

/// The platform's six seconds as a path of points (metres), flat or
/// rising by `rise` over its length.
fn path_shape(rise: f64, color: Srgb) -> Shape {
    let spin = 30.0 * DEG;
    let n = 60;
    let pts: Vec<V3> = (0..=n)
        .map(|k| {
            let tau = PATH_SECONDS * k as f64 / n as f64;
            let (x, y, _) = model::planar_exp(PATH_SPEED, 0.0, spin, tau);
            [x, y, 0.03 + rise * k as f64 / n as f64]
        })
        .collect();
    Shape::Path {
        pts,
        color,
        width: 5.0,
        closed: false,
        bias: 0.0,
    }
}

/// Three numbers as text, to two places, with the engine's minus (they sit
/// where live numbers would).
fn triple(values: [f64; 3]) -> [String; 3] {
    values.map(screen_entry)
}

impl SceneConstruct for Screws {
    fn name(&self) -> &str {
        "07_screws"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (orange, gold, teal) = (palette::flat(), palette::screw(), palette::place());
        let toss = toss_pose();
        let twist6 = log6(&toss);
        let (spin_v, slide_v) = ([twist6[0], twist6[1], twist6[2]], [twist6[3], twist6[4], twist6[5]]);
        let radius = PATH_SPEED / (30.0 * DEG);

        // ---- the floor world: chapter 06's floor, its platform and arrows,
        // its centre dot, the book that will rise off it, the screw's axis,
        // the helix, and the book's two arrows
        let (tilt, lift) = (stage.add_value_tracker(0.0), stage.add_value_tracker(0.0));
        let tilted = pose_fn(move |s| Pose {
            r: floor_turn(value(s, tilt)),
            p: [0.0; 3],
        });
        let book_half: V3 = [0.17, 0.24, 0.04];
        let book_pose = pose_fn(move |s| Pose {
            r: model::I3,
            p: [BOOK_AT[0], BOOK_AT[1], BOOK_AT[2] + BOOK_RISE * value(s, lift)],
        });
        let centre_up = [BOOK_AT[0], BOOK_AT[1], BOOK_AT[2] + BOOK_RISE];
        let slide_arrow = Shape::Arrow {
            from: centre_up,
            to: add(centre_up, scale(slide_v, 0.55)),
            color: orange,
            width: 5.0,
            head: 0.2,
        };
        let mut twist_shapes = spin_arrow(centre_up, unit(spin_v), 0.4 * norm(spin_v), 0.28);
        twist_shapes.push(slide_arrow);
        let specs = vec![
            PartSpec::new(table(FLOOR_LO, FLOOR_HI, 0.0, 0.5), tilted.clone()).layer(-2),
            PartSpec::new(platform_faces(), tilted.clone()).layer(-1),
            PartSpec::new(floor_arrows(), tilted.clone()).layer(-1),
            PartSpec::new(vec![path_shape(0.0, teal)], tilted.clone()).layer(-1),
            PartSpec::new(
                vec![Shape::Dot {
                    at: [0.0, radius, 0.03],
                    color: gold,
                    radius: 0.1,
                }],
                tilted,
            )
            .layer(-1),
            PartSpec::new(super::draw::book(book_half), book_pose),
            PartSpec::new(
                vec![Shape::line([0.0, radius, 0.0], [0.0, radius, 2.7], gold, 5.0)],
                still(Pose::NOTHING),
            ),
            PartSpec::new(vec![path_shape(HELIX_RISE, teal)], still(Pose::NOTHING)),
            PartSpec::new(twist_shapes, still(Pose::NOTHING)).layer(2),
        ];
        let view = View::new(1.35, v(-3.3, -0.6));
        let world = World::build(stage, view, 3.4, 0.2, specs, Vec::<Pin>::new())?;
        for i in [floor::BOOK, floor::AXIS, floor::HELIX, floor::TWIST] {
            stage.set_tracker_value(world.parts[i].shown, 0.0)?;
        }
        world.add_to_scene(stage)?;

        // 07_1: leave the floor; in how many ways can a pose change in space?
        let line = say(&kit, stage, BEATS[0])?;
        world.show(stage, 0.6)?;
        line.wait_for(stage, "Now leave the floor")?;
        play!(stage; tween(tilt, 1.0, 1.4)?);
        line.wait_for(stage, "In how many ways")?;
        play!(stage; tween(world.parts[floor::BOOK].shown, 1.0, 0.3)?);
        play!(stage; tween(lift, 1.0, 0.9)?);
        let card = QuizCard::build(&kit, stage, ASK, QuizLook::at(v(4.35, 0.3), 0.92))?;
        line.wait_last(stage)?;
        card.show(stage, 0.5)?;
        card.count(&kit, stage)?;

        // 07_2: six: a slide and a spin, six numbers, a twist
        let line = say(&kit, stage, BEATS[1])?;
        leave(stage, &card.mobs(), &[], 0.4)?;
        play!(stage;
            tween(world.parts[floor::TWIST].shown, 1.0, 0.7)?,
            tween(world.parts[floor::PLATFORM].shown, 0.0, 0.5)?,
            tween(world.parts[floor::ARROWS].shown, 0.0, 0.5)?,
        );
        let head = kit.text(stage, TWIST, 48.0, orange, v(4.2, 2.5))?;
        let spin_head = kit.text(stage, SPIN_WORD, 28.0, Palette::dim(), v(3.2, 1.7))?;
        let slide_head = kit.text(stage, SLIDE_WORD, 28.0, Palette::dim(), v(5.4, 1.7))?;
        let (spin_text, slide_text) = (triple(spin_v), triple(slide_v));
        let mut numbers = Vec::new();
        for i in 0..3 {
            let y = 1.05 - 0.6 * i as f64;
            numbers.push(kit.text(stage, &spin_text[i], 36.0, Palette::ink(), v(3.2, y))?);
            numbers.push(kit.text(stage, &slide_text[i], 36.0, Palette::ink(), v(5.4, y))?);
        }
        line.wait_for(stage, "So a velocity in space is two arrows")?;
        fade_in_all(stage, &[head], 0.4)?;
        line.wait_for(stage, "Six numbers")?;
        let mut group = vec![spin_head, slide_head];
        group.extend(&numbers);
        fade_in_all(stage, &group, 0.6)?;
        // what the slide arrow is comes in with the numbers, as a small tag
        // under them, and stays through the line's end
        let note = tag(&kit, stage, &twist_tag_words(), v(TAG_AT[0], TAG_AT[1]), TAG_PT)?;
        play!(stage; fade_in(stage.arena_mut(), note, ORIGIN, 1.0)?.rt(0.5));

        // 07_3: hold a twist steady: a screw; the centre stands up into an
        // axis, the arc lifts into a helix; the four words
        let line = say(&kit, stage, BEATS[2])?;
        let mut gone = vec![head, spin_head, slide_head];
        gone.extend(&numbers);
        leave(
            stage,
            &gone,
            &[world.parts[floor::TWIST].shown, world.parts[floor::BOOK].shown],
            0.5,
        )?;
        play!(stage;
            tween(world.parts[floor::PLATFORM].shown, 1.0, 0.5)?,
            tween(world.parts[floor::ARROWS].shown, 1.0, 0.5)?,
        );
        line.wait_for(stage, "The body turns about an axis")?;
        leave(stage, &[note], &[], 0.4)?;
        play!(stage; tween(world.parts[floor::AXIS].shown, 1.0, 0.8)?);
        play!(stage;
            tween(world.parts[floor::HELIX].shown, 1.0, 1.0)?,
            tween(world.parts[floor::ARC].shown, 0.0, 1.0)?,
        );
        let words = four_words(&kit, stage, v(4.1, 1.2), 0.95)?;
        fade_in_all(stage, &words, 0.6)?;

        // 07_4: a door is a screw with no slide, a drawer one with no turn,
        // a bottle cap does both
        let line = say(&kit, stage, BEATS[3])?;
        leave(stage, &words, &[world.shown], 0.6)?;
        let hinge = [-4.9, 0.0, 0.0];
        let (door_t, drawer_t, cap_t) = (
            stage.add_value_tracker(0.0),
            stage.add_value_tracker(0.0),
            stage.add_value_tracker(0.0),
        );
        let door_start = Pose { r: model::I3, p: hinge };
        let drawer_start = Pose {
            r: model::I3,
            p: [-0.5, 0.0, 0.35],
        };
        let cap_at = [4.6, 0.0, 0.2];
        let cap_start = Pose {
            r: model::I3,
            p: cap_at,
        };
        // each part's pose is the room's motion applied to where it started
        let door = pose_fn(move |s| door_motion(hinge, 75.0 * DEG * value(s, door_t)).then(&door_start));
        let drawer = pose_fn(move |s| drawer_motion(1.0 * value(s, drawer_t)).then(&drawer_start));
        let cap = pose_fn(move |s| {
            let a = value(s, cap_t);
            cap_motion(cap_at, 2.0 * std::f64::consts::PI * a, 0.7 * a).then(&cap_start)
        });
        let wood = mix(palette::steel(), super::draw::ground(), 0.35);
        let darker = mix(palette::steel(), super::draw::ground(), 0.6);
        let door_box = cuboid(
            [0.5, 0.0, 0.8],
            [0.5, 0.05, 0.8],
            [wood, wood, darker, darker, wood, wood],
            Some((super::draw::ground(), 1.4)),
        );
        let drawer_box = cuboid(
            [0.0; 3],
            [0.5, 0.4, 0.18],
            [wood, wood, darker, wood, wood, palette::steel()],
            Some((super::draw::ground(), 1.4)),
        );
        let cap_box = cuboid(
            [0.0; 3],
            [0.34, 0.34, 0.14],
            [wood, wood, wood, wood, darker, palette::steel()],
            Some((super::draw::ground(), 1.4)),
        );
        let slide_axis = Shape::line([-1.6, 0.0, 0.35], [1.6, 0.0, 0.35], gold, 5.0);
        let specs = vec![
            PartSpec::new(
                vec![plate([-6.0, -1.2], [6.0, 1.4], 0.0, super::draw::table_top(), -1.0)],
                still(Pose::NOTHING),
            )
            .layer(-2),
            PartSpec::new(door_box, door),
            PartSpec::new(
                vec![Shape::line([hinge[0], 0.0, 0.0], [hinge[0], 0.0, 2.1], gold, 5.0)],
                still(Pose::NOTHING),
            ),
            PartSpec::new(drawer_box, drawer),
            PartSpec::new(vec![slide_axis], still(Pose::NOTHING)),
            PartSpec::new(cap_box, cap),
            PartSpec::new(
                vec![Shape::line([cap_at[0], 0.0, 0.0], [cap_at[0], 0.0, 1.5], gold, 5.0)],
                still(Pose::NOTHING),
            ),
        ];
        let small = World::build(stage, View::new(1.1, v(-0.4, -1.0)), 7.0, 0.1, specs, Vec::<Pin>::new())?;
        for i in [2, 4, 6] {
            stage.set_tracker_value(small.parts[i].shown, 0.0)?;
        }
        small.add_to_scene(stage)?;
        small.show(stage, 0.5)?;
        let labels = [
            kit.text(stage, DOOR, 30.0, Palette::dim(), v(-5.3, -2.2))?,
            kit.text(stage, DRAWER, 30.0, Palette::dim(), v(-0.4, -2.2))?,
            kit.text(stage, BOTTLE_CAP, 30.0, Palette::dim(), v(4.2, -2.2))?,
        ];
        line.wait_for(stage, "A door is a screw with no slide")?;
        play!(stage; tween(small.parts[2].shown, 1.0, 0.4)?);
        fade_in_all(stage, &[labels[0]], 0.3)?;
        play!(stage; tween(door_t, 1.0, 1.4)?);
        line.wait_for(stage, "A drawer is a screw with no turn")?;
        play!(stage; tween(small.parts[4].shown, 1.0, 0.4)?);
        fade_in_all(stage, &[labels[1]], 0.3)?;
        play!(stage; tween(drawer_t, 1.0, 1.4)?);
        line.wait_for(stage, "A bottle cap does both")?;
        play!(stage; tween(small.parts[6].shown, 1.0, 0.4)?);
        fade_in_all(stage, &[labels[2]], 0.3)?;
        play!(stage; tween(cap_t, 1.0, 1.8)?);

        // 07_5: pick up the book and put it down anywhere: one screw
        let line = say(&kit, stage, BEATS[4])?;
        leave(stage, &labels, &[small.shown], 0.5)?;
        let screw = screw_of(&toss);
        let at_rest = Pose {
            r: model::I3,
            p: [0.0, 0.0, 0.14],
        };
        let walk = stage.add_value_tracker(0.0);
        let half: V3 = BOOK_HALF.map(|x| 0.55 * x);
        let ghost_at = along_screw(&screw, 1.0).then(&at_rest);
        let walker = pose_fn(move |s| along_screw(&screw, value(s, walk)).then(&at_rest));
        let toss_specs = vec![
            PartSpec::new(table([-2.0, -2.0], [3.0, 3.0], 0.0, 0.5), still(Pose::NOTHING)).layer(-2),
            PartSpec::new(super::draw::book(half), still(ghost_at)),
            PartSpec::new(super::draw::book(half), walker),
            PartSpec::new(screw_shapes(&screw, 2.0, 0.7), still(Pose::NOTHING)).layer(2),
        ];
        let tossed = World::build(
            stage,
            View::new(1.3, v(-3.3, -0.4)),
            4.0,
            0.2,
            toss_specs,
            Vec::<Pin>::new(),
        )?;
        stage.set_tracker_value(tossed.parts[3].shown, 0.0)?;
        stage.set_tracker_value(tossed.parts[1].grey, 1.0)?;
        tossed.add_to_scene(stage)?;
        tossed.show(stage, 0.5)?;
        line.wait_for(stage, "Put it down anywhere")?;
        play!(stage; tween(tossed.parts[3].shown, 1.0, 0.6)?);
        line.wait_for(stage, "One screw would have carried it there")?;
        play!(stage; tween_linear(walk, 1.0, 2.0)?);
        let turn_deg = fact_f64("G-S.toss.turn_deg")?;
        let slide = fact_f64("G-S.toss.slide")?;
        let readout = kit.text_left(
            stage,
            &format!("{TURN_WORD} {turn_deg:.0}{DEGREE}    {SLIDE_LABEL} {slide:.1} {METRES}"),
            34.0,
            gold,
            v(-6.2, 2.9),
        )?;
        fade_in_all(stage, &[readout], 0.4)?;
        // a wall of small books, each with its own screw axis
        let turns = wall_turns();
        let want = 24usize;
        // one book per cell, 0.95 of a scene unit apart across and up
        let (across, up) = (View::basis().0[0], View::basis().1[2]);
        let mut wall_specs = Vec::new();
        for k in 0..want {
            let (axis, deg) = turns[1 + k * (turns.len() - 1) / want];
            let (col, row) = ((k % 6) as f64, (k / 6) as f64);
            let at = [(col - 2.5) * 0.95 / across, 0.0, (1.5 - row) * 0.95 / up];
            let r = turn(axis, deg * DEG);
            wall_specs.push(PartSpec::new(
                super::draw::book(BOOK_HALF.map(|x| 0.3 * x)),
                still(Pose { r, p: at }),
            ));
            wall_specs.push(PartSpec::new(
                vec![Shape::line(
                    add(at, scale(axis, -0.5)),
                    add(at, scale(axis, 0.5)),
                    gold,
                    3.0,
                )],
                still(Pose::NOTHING),
            ));
        }
        let wall = World::build(
            stage,
            View::new(1.0, v(2.9, 0.4)),
            4.5,
            0.1,
            wall_specs,
            Vec::<Pin>::new(),
        )?;
        wall.add_to_scene(stage)?;
        wall.show(stage, 0.8)?;
        let count = fact("G-X3.one_screw")?
            .split('/')
            .next()
            .and_then(|n| n.parse::<u32>().ok())
            .unwrap_or(0);
        let caption = kit.text(
            stage,
            &format!("{count} {DIFFERENT_POSES} {count} {SCREWS}"),
            30.0,
            Palette::ink(),
            v(3.0, -2.3),
        )?;
        fade_in_all(stage, &[caption], 0.5)?;

        // 07_6: Chasles, and Mozzi before him
        let line = say(&kit, stage, BEATS[5])?;
        let (mozzi, chasles) = (fact("G-L.mozzi.year")?, fact("G-L.chasles.year")?);
        let history = precisely(
            &kit,
            stage,
            &format!("Mozzi {mozzi} {MOZZI_NOTE} · Chasles {chasles}: {CHASLES_NOTE}"),
        )?;
        line.wait_for(stage, "Michel Chasles")?;
        play!(stage; fade_in(stage.arena_mut(), history, ORIGIN, 1.0)?.rt(0.6));
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::FRAC_PI_2;

    use super::super::draw::{LOWER_THIRD_TOP, SAFE, fact_list, probe, run_chapter};
    use super::*;
    use crate::model::X;

    #[test]
    fn the_beats_are_the_storyboards_and_the_scripts() {
        let board = explainer_kit::storyboard::parse(crate::STORYBOARD).unwrap();
        let chapter = board.iter().find(|c| c.key == "07_screws").expect("chapter 07");
        let ids: Vec<&str> = chapter.beats.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(ids, BEATS);
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for beat in &chapter.beats {
            let words = beat.spoken().expect("every beat of chapter 07 is spoken");
            assert_eq!(script.text(&beat.id), Some(words.as_str()), "{}", beat.id);
        }
    }

    /// 07_1: four seconds; three slides and three turns make six.
    #[test]
    fn the_quiz_asks_for_six() {
        assert_eq!(ASK.seconds().unwrap(), 4);
        assert_eq!(fact("G-Q.07_1.answer").unwrap(), "6");
        assert_eq!(ASK.answer_index().unwrap(), 1);
        assert_eq!(fact("G-S.numbers.pose").unwrap(), "6");
        assert_eq!(fact("G-S.numbers.twist").unwrap(), "6");
    }

    /// 07_1: the floor, tilted, is face-on to the viewer with chapter 06's x
    /// to the right; let go, it lies flat.
    #[test]
    fn the_floor_tilts_from_face_on_to_flat() {
        let (right, _, toward) = View::basis();
        let face_on = floor_turn(0.0);
        assert!(
            norm(sub(model::apply(&face_on, Z), toward)) < 1e-9,
            "its normal faces the viewer"
        );
        assert!(
            norm(sub(model::apply(&face_on, X), right)) < 1e-9,
            "x runs to the right"
        );
        assert!(model::gap(&floor_turn(1.0), &model::I3) < 1e-12, "flat at the end");
    }

    /// The whole chapter constructs, headless.
    #[test]
    fn the_chapter_constructs_end_to_end() {
        run_chapter(&mut Screws {
            kit: Kit::new().expect("the kit"),
        });
    }

    /// 07_2: the "for programmers" tag says, in the storyboard's words, what
    /// the slide arrow is; it stands in the right column under the six
    /// numbers, inside the safe area, and off the floor's edges.
    #[test]
    fn the_programmers_tag_stands_under_the_numbers_and_off_the_floor() {
        assert!(crate::STORYBOARD.contains(TWIST_NOTE), "the storyboard's tag differs");
        let words = twist_tag_words();
        assert!(words.starts_with(FOR_PROGRAMMERS));
        assert_eq!(words.lines().skip(1).collect::<Vec<_>>().join(" "), TWIST_NOTE);
        probe(|stage| {
            let kit = Kit::new()?;
            let t = tag(&kit, stage, &words, v(TAG_AT[0], TAG_AT[1]), TAG_PT)?;
            let (left, right) = (stage.get_left(t)[0], stage.get_right(t)[0]);
            let (bottom, top) = (stage.get_bottom(t)[1], stage.get_top(t)[1]);
            assert!(
                left > -SAFE[0] && right < SAFE[0],
                "the tag runs {left:.2} to {right:.2}"
            );
            assert!(bottom > LOWER_THIRD_TOP + 0.1, "the tag's foot is at {bottom:.2}");
            // the six numbers' lowest row stands at y = -0.15 (36 points)
            assert!(top < -0.15 - 0.3, "the tag's top is at {top:.2}");
            // the floor, flat, in this chapter's view: no edge crosses the tag
            let view = View::new(1.35, v(-3.3, -0.6));
            let corners = [
                [FLOOR_LO[0], FLOOR_LO[1], 0.0],
                [FLOOR_HI[0], FLOOR_LO[1], 0.0],
                [FLOOR_HI[0], FLOOR_HI[1], 0.0],
                [FLOOR_LO[0], FLOOR_HI[1], 0.0],
            ];
            for k in 0..4 {
                let (a, b) = (view.project(corners[k]), view.project(corners[(k + 1) % 4]));
                for s in 0..=60 {
                    let f = f64::from(s) / 60.0;
                    let p = (a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f);
                    let inside = p.0 > left - 0.05 && p.0 < right + 0.05 && p.1 > bottom - 0.05 && p.1 < top + 0.05;
                    assert!(!inside, "the floor's edge passes through the tag at {p:?}");
                }
            }
            Ok(())
        });
    }

    /// 07_2: a twist is two arrows, six numbers, and held for a second it
    /// is the pose it came from.
    #[test]
    fn a_twist_is_six_numbers_that_make_the_pose() {
        let t = log6(&toss_pose());
        assert_eq!(t.len(), 6);
        let back = model::exp6(t);
        assert!(back.gap(&toss_pose()) < 1e-9);
        // the spin arrow is a quarter turn a second about the vertical
        assert!((t[0].abs() + t[1].abs()) < 1e-9 && (t[2] - FRAC_PI_2).abs() < 1e-9);
        assert_eq!(
            triple([1.0, -0.0, 0.5]),
            ["1".to_owned(), "0".to_owned(), "0.50".to_owned()]
        );
    }

    /// 07_3: the screw of the platform's path is vertical through its centre,
    /// the helix is that arc lifted by a slide, and the centre dot is the
    /// fact's centre.
    #[test]
    fn the_axis_stands_up_through_the_centre() {
        let radius = PATH_SPEED / (30.0 * DEG);
        assert!((radius - fact_f64("G-P.radius").unwrap()).abs() < 1e-3);
        let helix = path_shape(HELIX_RISE, palette::place());
        let pts = match &helix {
            Shape::Path { pts, .. } => pts.clone(),
            _ => Vec::new(),
        };
        assert!(!pts.is_empty(), "the helix is a path");
        let (first, last) = (pts[0], pts[pts.len() - 1]);
        assert!(norm(sub(first, [0.0, 0.0, 0.03])) < 1e-9);
        assert!((last[2] - 0.03 - HELIX_RISE).abs() < 1e-9, "it rises by the slide");
        // every point is the radius from the vertical axis through (0, r)
        for p in pts {
            assert!((p[0].hypot(p[1] - radius) - radius).abs() < 1e-9);
        }
    }

    /// 07_4: a door is a screw with no slide, a drawer one with no turn, a
    /// cap both.
    #[test]
    fn a_door_a_drawer_and_a_cap_are_screws() {
        let hinge = [-4.9, 0.0, 0.0];
        let door = screw_of(&door_motion(hinge, 75.0 * DEG));
        assert!((door.angle / DEG - 75.0).abs() < 1e-9 && door.slide.abs() < 1e-9);
        assert!(norm(sub(door.axis, Z)) < 1e-9 && (door.point[0] - hinge[0]).abs() < 1e-9);
        let drawer = screw_of(&drawer_motion(1.0));
        assert!(drawer.angle.abs() < 1e-12 && (drawer.slide - 1.0).abs() < 1e-12);
        let cap = screw_of(&cap_motion([4.6, 0.0, 0.2], 1.0, 0.7));
        assert!((cap.angle - 1.0).abs() < 1e-9 && (cap.slide - 0.7).abs() < 1e-9);
    }

    /// 07_5: the tossed book is one screw: a quarter turn about the vertical
    /// through (0, 1) with a half-metre slide (the facts); replayed along it,
    /// the book lands a metre right, a metre away and half a metre up.
    #[test]
    fn the_tossed_book_is_one_corkscrew() {
        let s = screw_of(&toss_pose());
        assert!((s.angle / DEG - fact_f64("G-S.toss.turn_deg").unwrap()).abs() < 1e-3);
        assert!((s.slide - fact_f64("G-S.toss.slide").unwrap()).abs() < 1e-3);
        let point = fact_list("G-S.toss.point").unwrap();
        assert!(norm(sub(s.point, [point[0], point[1], point[2]])) < 1e-3);
        assert!(norm(sub(s.axis, Z)) < 1e-9);
        let rest = Pose {
            r: model::I3,
            p: [0.0, 0.0, 0.14],
        };
        let there = along_screw(&s, 1.0).then(&rest);
        assert!(norm(sub(there.p, [1.0, 1.0, 0.14 + 0.5])) < 1e-9);
        assert!(model::gap(&there.r, &turn(Z, FRAC_PI_2)) < 1e-9);
    }

    /// 07_5: the wall's sixty-six turns times four slides are the 264 poses
    /// of the fact, and each pose of the wall is rebuilt by its own screw.
    #[test]
    fn a_wall_of_poses_each_one_screw() {
        let turns = wall_turns();
        assert_eq!(turns.len(), 66);
        let slides = 4;
        assert_eq!(
            fact("G-X3.one_screw").unwrap(),
            format!("{0}/{0}", turns.len() * slides)
        );
        for (axis, deg) in turns.iter().skip(1).step_by(3) {
            let t = Pose {
                r: turn(*axis, deg * DEG),
                p: [0.2, -0.1, 0.4],
            };
            let rebuilt = along_screw(&screw_of(&t), 1.0);
            assert!(rebuilt.gap(&t) < 1e-9, "{axis:?} {deg}");
            // the screw's axis is the turn's own axis
            let s = screw_of(&t);
            assert!(norm(sub(s.axis, *axis)) < 1e-9 || norm(add(s.axis, *axis)) < 1e-9);
        }
    }

    /// 07_6: the years the lower third states.
    #[test]
    fn mozzi_before_chasles() {
        let (m, c) = (
            fact_f64("G-L.mozzi.year").unwrap(),
            fact_f64("G-L.chasles.year").unwrap(),
        );
        assert_eq!((m, c), (1763.0, 1830.0));
    }
}
