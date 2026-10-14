//! Chapter 02, one turn: where tilt then roll left the book is one turn
//! about one axis (02_1), a third of a turn (02_2, 02_3), and any turn after
//! any turn is one turn (02_4); turns can be undone, which makes them a group
//! where order usually matters (02_5); and to compute, track where the three
//! edges point: nine numbers in a grid, and one grid times another (02_6).

use std::f64::consts::FRAC_PI_2;
use std::rc::Rc;

use explainer_kit::props::{CodePanel, lower_third, precisely};
use explainer_kit::{Kit, Palette, Timed, play, tween, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::draw::{
    Ask, BOOK_HALF, Choice, PartSpec, Pin, QuizLook, Shape, View, World, arc_between, book, edge_arrows, edge_colors,
    fact, fact_f64, fade_in_all, fail, grid_brackets, grid_spots, group, leave, line_style, pause, point_fn, pose_fn,
    say, screen_entry, screw_shapes, static_grid, still, table, turns_in_order, tween_all, tween_linear, value,
    your_turn,
};
use crate::model::{self, DEG, M3, Pose, Screw, V3, X, Y, Z, dot, scale, unit};
use crate::palette;

pub struct OneTurn {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const GROUP: &str = "group";
const ASSOCIATIVE: &str = "and combining is associative";
const ONE_AXIS: &str = "about one axis, order never matters";
const AXIS_WORD: &str = "axis";
const TWO_TURNS_ONE_AXIS: &str = "two turns about one axis";
const TILT_THEN_ROLL: &str = "tilt, then roll";
const ROLL_THEN_TILT: &str = "roll, then tilt";
const TILT: &str = "tilt";
const ROLL: &str = "roll";
const TIMES: &str = "×";
const EQUALS: &str = "=";
const ORDER_DEGREE: &str = "°";
const EULER_NOTE: &str = "however a sphere is turned about its centre, some diameter ends where it began";
const CODE: [&str; 4] = [
    "R = roll @ tilt",
    "# turns about the room's lines:",
    "# the later turn goes on the left",
    "# columns: where the edges point",
];

/// The beats of the chapter, in order (storyboard ids).
const BEATS: [&str; 6] = ["02_1", "02_2", "02_3", "02_4", "02_5", "02_6"];

/// 02_2's quiz: how big is the single turn? The keys are the angle in
/// degrees, as `G-Q.02_2.answer` prints it.
pub const ASK: Ask = Ask {
    line: BEATS[1],
    question: "How big is that\nsingle turn?",
    choices: &[
        Choice {
            key: "90",
            label: r"\frac{1}{4}\text{ turn}",
        },
        Choice {
            key: "120",
            label: r"\frac{1}{3}\text{ turn}",
        },
        Choice {
            key: "180",
            label: r"\frac{1}{2}\text{ turn}",
        },
    ],
};

/// How far the gold axis runs each way from the book's centre, and where its
/// word stands against the upper end (scene units).
const AXIS_HALF: f64 = 2.1;
const AXIS_WORD_SHIFT: [f64; 2] = [0.55, 0.38];

/// The end of the gold line that is higher on the screen: the word "axis"
/// stands beside it. The line runs through the book's centre, held at
/// `CENTRE` (the tip was once taken about the origin, 1.25 under the line:
/// the word stood on the book's crossbar instead of at the line's end).
fn axis_word_tip(view: &View, axis: V3) -> V3 {
    let (a, b) = (
        model::add(CENTRE, scale(axis, AXIS_HALF)),
        model::add(CENTRE, scale(axis, -AXIS_HALF)),
    );
    if view.project(a)[1] >= view.project(b)[1] { a } else { b }
}

/// 02_3's readout of the turn so far: its right edge, and its degree sign.
const READOUT_AT: [f64; 2] = [-3.55, 2.9];
const DEGREE_AT: [f64; 2] = [-3.43, 2.98];

/// The height the book is held at while it turns.
const TURN_Z: f64 = 1.25;
const CENTRE: V3 = [0.0, 0.0, TURN_Z];
/// Where the book's world sits (its table ends inside the safe area on the
/// left and clear of the quiz card on the right) and where the card stands.
const BOOKS_AT: [f64; 2] = [-3.1, -0.7];
const CARD_AT: [f64; 2] = [4.35, 0.2];
const CARD_SCALE: f64 = 0.92;
/// The live grid of 02_6, above the code panel that slides in at the end
/// (the panel stands at the right, centred on the middle of the frame).
const GRID_AT: [f64; 2] = [4.3, 2.45];
const GRID_PITCH: (f64, f64) = (1.25, 0.72);
const TABLE_LO: [f64; 2] = [-3.0, -2.2];
const TABLE_HI: [f64; 2] = [3.0, 2.6];

/// 02_4's three more pairs of turns: (axis, degrees) then (axis, degrees).
/// Arbitrary, generic ones: the claim they illustrate is Euler's, tested on
/// 192 products in `G-X1.one_turn`.
const PAIRS: [((V3, f64), (V3, f64)); 3] = [
    ((X, 60.0), (Y, 100.0)),
    ((Z, 130.0), (X, 50.0)),
    (([1.0, 1.0, 0.0], 80.0), (Z, 120.0)),
];

/// The book after the turn about `axis` (`deg` degrees of it), then a tilt
/// and a roll (`ta` and `tb` of a quarter turn each): the picture's matrix.
fn book_turn(axis: V3, deg: f64, ta: f64, tb: f64) -> M3 {
    turns_in_order(&[(axis, deg * DEG), (X, ta * FRAC_PI_2), (Y, tb * FRAC_PI_2)])
}

/// The single turn two quarter turns come to: its angle (degrees) and axis.
fn single_turn() -> (f64, V3) {
    let (angle, axis) = model::angle_axis(&model::tilt_then_roll(FRAC_PI_2));
    (angle / DEG, axis)
}

/// The two 3 by 3 grids' cells as strings, from a `G-B.*.R`-style fact
/// (`a,b,c;d,e,f;g,h,i`).
fn parse_grid(text: &str) -> fmn::Result<[[String; 3]; 3]> {
    let rows: Vec<&str> = text.split(';').collect();
    if rows.len() != 3 {
        return Err(fail(format!("a grid has three rows: {text}")));
    }
    let mut out: [[String; 3]; 3] = Default::default();
    for (i, row) in rows.iter().enumerate() {
        let cells: Vec<&str> = row.split(',').collect();
        if cells.len() != 3 {
            return Err(fail(format!("a grid row has three cells: {row}")));
        }
        for (j, cell) in cells.iter().enumerate() {
            let x: f64 = cell.parse().map_err(|_| fail(format!("not a number: {cell}")))?;
            out[i][j] = screen_entry(x);
        }
    }
    Ok(out)
}

/// A grid of a turn's matrix, rounded as the picture prints it.
fn grid_of(r: &M3) -> [[String; 3]; 3] {
    std::array::from_fn(|i| std::array::from_fn(|j| screen_entry(r[i][j])))
}

/// Two strokes for an equals sign, centred at `at`, in `color`.
fn equals_sign(stage: &mut Stage<'_>, at: Vec3, color: Srgb) -> fmn::Result<Mob> {
    let w = 0.3;
    let bars: Vec<Mob> = [0.1, -0.1]
        .into_iter()
        .map(|dy| {
            let bar = polyline(vec![v(at[0] - w, at[1] + dy), v(at[0] + w, at[1] + dy)])
                .style(line_style(color, 6.0))
                .build();
            stage.arena_mut().add(bar)
        })
        .collect();
    group(stage, &bars)
}

/// A red cross over `at`, as strokes.
fn cross_out(stage: &mut Stage<'_>, at: Vec3) -> fmn::Result<Mob> {
    let r = 0.28;
    let strokes: Vec<Mob> = [(1.0, 1.0), (1.0, -1.0)]
        .into_iter()
        .map(|(sx, sy)| {
            let bar = polyline(vec![
                v(at[0] - sx * r, at[1] - sy * r),
                v(at[0] + sx * r, at[1] + sy * r),
            ])
            .style(line_style(palette::warn(), 7.0))
            .build();
            stage.arena_mut().add(bar)
        })
        .collect();
    group(stage, &strokes)
}

impl SceneConstruct for OneTurn {
    fn name(&self) -> &str {
        "02_one_turn"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let gold = palette::screw();
        let (_, axis) = single_turn();
        let facts_deg = fact_f64("G-B.tilt_then_roll.angle")?;

        // ---- the book, held at turn height: its ghost, the book itself with
        // its three edge arrows, and (hidden until needed) the gold axis and
        // the angle each edge makes with it
        let (deg, ta, tb) = (
            stage.add_value_tracker(facts_deg),
            stage.add_value_tracker(0.0),
            stage.add_value_tracker(0.0),
        );
        let moving = pose_fn(move |s| Pose {
            r: book_turn(axis, value(s, deg), value(s, ta), value(s, tb)),
            p: CENTRE,
        });
        let at_rest = still(Pose {
            r: model::I3,
            p: CENTRE,
        });
        let arcs: Vec<Shape> = (0..3)
            .map(|k| {
                let mut edge = [0.0; 3];
                edge[k] = 1.0;
                let s = if dot(axis, edge) < 0.0 { -1.0 } else { 1.0 };
                Shape::Path {
                    pts: arc_between([axis[0] * s, axis[1] * s, axis[2] * s], edge, 0.8, 14),
                    color: Palette::ink(),
                    width: 3.0,
                    closed: false,
                    bias: 0.0,
                }
            })
            .collect();
        let specs = vec![
            PartSpec::new(table(TABLE_LO, TABLE_HI, 0.0, 0.8), still(Pose::NOTHING)).layer(-2),
            PartSpec::new(book(BOOK_HALF), at_rest.clone()),
            PartSpec::new(book(BOOK_HALF), moving.clone()),
            PartSpec::new(edge_arrows(1.35), moving.clone()),
            PartSpec::new(
                vec![Shape::line(
                    [-AXIS_HALF * axis[0], -AXIS_HALF * axis[1], -AXIS_HALF * axis[2]],
                    [AXIS_HALF * axis[0], AXIS_HALF * axis[1], AXIS_HALF * axis[2]],
                    gold,
                    5.0,
                )],
                at_rest,
            ),
            PartSpec::new(arcs, moving),
        ];
        const GHOST: usize = 1;
        const AXIS: usize = 4;
        const ARCS: usize = 5;
        // the word "axis" stands by the gold line's upper end and comes in
        // with the line (R2 C-01, M-03)
        let view = View::new(1.0, v(BOOKS_AT[0], BOOKS_AT[1]));
        let axis_word = kit.text(stage, AXIS_WORD, 28.0, gold, ORIGIN)?;
        let word_at = axis_word_tip(&view, axis);
        let pins = vec![Pin {
            mob: axis_word,
            at: point_fn(move |_| word_at),
            shift: v(AXIS_WORD_SHIFT[0], AXIS_WORD_SHIFT[1]),
        }];
        let books = World::build(stage, view, 4.0, 0.2, specs, pins)?;
        stage.set_tracker_value(books.parts[GHOST].grey, 1.0)?;
        stage.set_tracker_value(books.parts[AXIS].shown, 0.0)?;
        stage.set_tracker_value(books.parts[ARCS].shown, 0.0)?;
        books.add_to_scene(stage)?;

        // 02_1: it could have got there in a single turn about a single axis
        let line = say(&kit, stage, BEATS[0])?;
        books.show(stage, 0.8)?;
        line.wait_for(stage, "a single turn")?;
        play!(stage;
            tween(books.parts[AXIS].shown, 1.0, 0.5)?,
            fade_in(stage.arena_mut(), axis_word, ORIGIN, 1.0)?.rt(0.5),
        );
        play!(stage; tween_linear(deg, 0.0, 0.9)?);
        line.wait_for(stage, "about a single axis")?;
        play!(stage; tween(deg, facts_deg, 1.8)?);

        // 02_2: how big is that single turn?
        let card = your_turn(&kit, stage, ASK, QuizLook::at(v(CARD_AT[0], CARD_AT[1]), CARD_SCALE))?;

        // 02_3: a third of a turn, three of them bring it home; the axis
        // leans equally from the three edges
        let line = say(&kit, stage, BEATS[2])?;
        play!(stage; tween(deg, 0.0, 0.7)?);
        let readout = kit.live_number(
            stage,
            deg,
            0,
            false,
            54.0,
            Palette::ink(),
            v(READOUT_AT[0], READOUT_AT[1]),
            RIGHT,
        )?;
        let degree = kit.text(stage, ORDER_DEGREE, 54.0, Palette::ink(), v(DEGREE_AT[0], DEGREE_AT[1]))?;
        fade_in_all(stage, &[degree], 0.2)?;
        line.wait_for(stage, "A third of a turn")?;
        play!(stage; tween(deg, facts_deg, 1.2)?);
        line.wait_for(stage, "Three of them")?;
        play!(stage; tween(deg, 2.0 * facts_deg, 1.0)?);
        play!(stage; tween(deg, 3.0 * facts_deg, 1.0)?);
        line.wait_for(stage, "Its axis leans equally")?;
        play!(stage; tween(books.parts[ARCS].shown, 1.0, 0.6)?);

        // 02_4: that always works: three more pairs, each one axis and one arc
        let line = say(&kit, stage, BEATS[3])?;
        let mut leaving = card.mobs();
        leaving.extend([readout, degree, axis_word]);
        leave(stage, &leaving, &[books.shown], 0.6)?;
        stage.set_tracker_value(deg, 0.0)?;
        let small: V3 = BOOK_HALF.map(|x| 0.62 * x);
        let view_small = View::new(0.9, v(0.0, -0.4));
        let mut pair_specs = Vec::new();
        let mut firsts = Vec::new();
        let mut seconds = Vec::new();
        let mut axes_at = Vec::new();
        for (k, &((ax1, d1), (ax2, d2))) in PAIRS.iter().enumerate() {
            let centre = [(k as f64 - 1.0) * 5.4, 0.0, 1.0];
            let (a, b) = (stage.add_value_tracker(0.0), stage.add_value_tracker(0.0));
            let (ax1, ax2) = (unit(ax1), unit(ax2));
            firsts.push(a);
            seconds.push(b);
            pair_specs.push(PartSpec::new(
                book(small),
                pose_fn(move |s| Pose {
                    r: turns_in_order(&[(ax1, d1 * DEG * value(s, a)), (ax2, d2 * DEG * value(s, b))]),
                    p: centre,
                }),
            ));
            let both = turns_in_order(&[(ax1, d1 * DEG), (ax2, d2 * DEG)]);
            let (angle, one_axis) = model::angle_axis(&both);
            let one = Screw {
                angle,
                axis: one_axis,
                point: centre,
                slide: 0.0,
            };
            axes_at.push(pair_specs.len());
            pair_specs.push(PartSpec::new(screw_shapes(&one, 1.3, 0.85), still(Pose::NOTHING)).layer(1));
        }
        let pairs = World::build(stage, view_small, 7.0, 0.1, pair_specs, Vec::<Pin>::new())?;
        for &i in &axes_at {
            stage.set_tracker_value(pairs.parts[i].shown, 0.0)?;
        }
        pairs.add_to_scene(stage)?;
        pairs.show(stage, 0.5)?;
        line.wait_for(stage, "Any turn followed by any turn")?;
        let ones: Vec<(Mob, f64)> = firsts.iter().map(|&t| (t, 1.0)).collect();
        tween_all(stage, &ones, 0.6)?;
        let twos: Vec<(Mob, f64)> = seconds.iter().map(|&t| (t, 1.0)).collect();
        tween_all(stage, &twos, 0.6)?;
        let shown: Vec<(Mob, f64)> = axes_at.iter().map(|&i| (pairs.parts[i].shown, 1.0)).collect();
        tween_all(stage, &shown, 0.6)?;
        // the exact version comes in with the three axes, not with the
        // sentence's last words, so that it stays up for several seconds
        let year = fact("G-L.euler.year")?;
        let note = lower_third(&kit, stage, &format!("Euler, {year}:"), EULER_NOTE)?;
        play!(stage; fade_in(stage.arena_mut(), note, ORIGIN, 1.0)?.rt(0.5));

        // 02_5: a turn can be undone; a collection with both is a group
        let line = say(&kit, stage, BEATS[4])?;
        leave(stage, &[note], &[pairs.shown], 0.5)?;
        books.show(stage, 0.5)?;
        stage.set_tracker_value(books.parts[AXIS].shown, 0.0)?;
        stage.set_tracker_value(books.parts[ARCS].shown, 0.0)?;
        line.wait_for(stage, "And any turn can be undone")?;
        play!(stage; tween(deg, facts_deg, 1.0)?);
        play!(stage; tween(deg, 0.0, 1.0)?);
        line.wait_for(stage, "A collection where")?;
        let word = kit.text(stage, GROUP, 78.0, gold, v(4.0, 1.5))?;
        let precise = kit.text(stage, ASSOCIATIVE, 26.0, Palette::dim(), v(4.0, 0.7))?;
        fade_in_all(stage, &[word, precise], 0.6)?;
        line.wait_for(stage, "In this one, order usually matters")?;
        leave(stage, &[word, precise], &[books.shown], 0.5)?;
        // the two panels: tilt, roll and roll, tilt end apart; two turns about
        // one axis end together
        let one_axis = model::turn(X, (40.0 + 70.0) * DEG);
        let panel_books: [(M3, f64); 4] = [
            (model::tilt_then_roll(FRAC_PI_2), -6.55),
            (model::roll_then_tilt(FRAC_PI_2), -3.15),
            (one_axis, 3.15),
            (one_axis, 6.55),
        ];
        let panel_specs: Vec<PartSpec> = panel_books
            .iter()
            .map(|&(r, x)| PartSpec::new(book(small), still(Pose { r, p: [x, 0.0, 1.0] })))
            .collect();
        let panels = World::build(stage, view_small, 7.0, 0.1, panel_specs, Vec::<Pin>::new())?;
        panels.add_to_scene(stage)?;
        panels.show(stage, 0.6)?;
        let mid = |x_world: f64| view_small.project([x_world, 0.0, 1.0]);
        let apart = equals_sign(stage, mid(-4.85), palette::warn())?;
        let crossed = cross_out(stage, mid(-4.85))?;
        let same = equals_sign(stage, mid(4.85), palette::good())?;
        let left_cap = kit.text(stage, TILT_THEN_ROLL, 26.0, Palette::dim(), v(-5.0, -1.9))?;
        let right_cap = kit.text(stage, ROLL_THEN_TILT, 26.0, Palette::dim(), v(-2.4, -1.9))?;
        let axis_cap = kit.text(stage, TWO_TURNS_ONE_AXIS, 26.0, Palette::dim(), v(3.7, -1.9))?;
        fade_in_all(stage, &[apart, crossed, left_cap, right_cap], 0.5)?;
        fade_in_all(stage, &[same, axis_cap], 0.5)?;
        let order = precisely(&kit, stage, ONE_AXIS)?;
        play!(stage; fade_in(stage.arena_mut(), order, ORIGIN, 1.0)?.rt(0.5));
        kit.hold(stage)?;

        // 02_6: track where the three edges point: nine numbers in a grid
        let line = say(&kit, stage, BEATS[5])?;
        leave(
            stage,
            &[apart, crossed, same, left_cap, right_cap, axis_cap, order],
            &[panels.shown],
            0.5,
        )?;
        books.show(stage, 0.6)?;
        let r_now: Rc<dyn Fn(&Arena) -> M3> =
            Rc::new(move |s: &Arena| book_turn(axis, value(s, deg), value(s, ta), value(s, tb)));
        let pitch = GRID_PITCH;
        let grid_at = v(GRID_AT[0], GRID_AT[1]);
        let colors = edge_colors();
        line.wait_for(stage, "track where the book's three edges point")?;
        play!(stage; tween(ta, 1.0, 1.3)?);
        line.wait_for(stage, "Three arrows, three numbers each")?;
        let spots = grid_spots(grid_at, pitch);
        for (j, &color) in colors.iter().enumerate() {
            for (i, row) in spots.iter().enumerate() {
                let (f, at) = (Rc::clone(&r_now), row[j]);
                kit.live_value(
                    stage,
                    move |s| {
                        let x = f(s)[i][j];
                        (x * 100.0).round() / 100.0 + 0.0
                    },
                    2,
                    false,
                    34.0,
                    color,
                    at,
                    ORIGIN,
                )?;
            }
            if j + 1 < 3 {
                pause(stage, 0.5)?;
            }
        }
        line.wait_for(stage, "Nine numbers in a grid")?;
        let brackets = grid_brackets(stage, grid_at, pitch, Palette::dim());
        fade_in_all(stage, &brackets, 0.4)?;
        play!(stage; tween(tb, 1.0, 1.3)?);
        line.wait_for(stage, "One turn after another")?;
        // the grids of tilt and roll make the grid of tilt then roll
        let first = kit.text(stage, TILT, 26.0, Palette::dim(), v(-5.4, 2.55))?;
        let second = kit.text(stage, ROLL, 26.0, Palette::dim(), v(-2.5, 2.55))?;
        let result = kit.text(stage, TILT_THEN_ROLL, 26.0, Palette::dim(), v(0.5, 2.55))?;
        let small_pitch = (0.72, 0.5);
        let rows = [
            (grid_of(&model::roll_turn(FRAC_PI_2)), v(-5.4, 1.5)),
            (grid_of(&model::tilt_turn(FRAC_PI_2)), v(-2.5, 1.5)),
            (parse_grid(fact("G-B.tilt_then_roll.R")?)?, v(0.5, 1.5)),
        ];
        let mut row_mobs = Vec::new();
        for (cells, at) in rows {
            row_mobs.extend(static_grid(&kit, stage, cells, at, small_pitch, 30.0, colors)?);
        }
        let times = kit.text(stage, TIMES, 40.0, Palette::ink(), v(-3.95, 1.5))?;
        let equals = kit.text(stage, EQUALS, 40.0, Palette::ink(), v(-1.05, 1.5))?;
        row_mobs.extend([first, second, result, times, equals]);
        fade_in_all(stage, &row_mobs, 0.7)?;
        let panel = CodePanel::build(&kit, stage, &CODE, 26.0, 5.6)?;
        play!(stage; panel.slide_in(0.8)?);
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::{CARD_W, QuizCard, SAFE, entry, probe};
    use super::*;

    /// Chapter 02 plays exactly the storyboard's beats, and every beat is the
    /// script's own line.
    #[test]
    fn the_beats_are_the_storyboards_and_the_scripts() {
        let board = explainer_kit::storyboard::parse(crate::STORYBOARD).unwrap();
        let chapter = board.iter().find(|c| c.key == "02_one_turn").expect("chapter 02");
        let ids: Vec<&str> = chapter.beats.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(ids, BEATS);
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for beat in &chapter.beats {
            let words = beat.spoken().expect("every beat of chapter 02 is spoken");
            assert_eq!(script.text(&beat.id), Some(words.as_str()), "{}", beat.id);
        }
    }

    /// 02_2: a countdown of the storyboard's seconds, three choices, and the
    /// computed answer is the third of a turn.
    #[test]
    fn the_quiz_asks_for_a_third_of_a_turn() {
        assert_eq!(ASK.seconds().unwrap(), 5);
        assert_eq!(ASK.answer_index().unwrap(), 1);
        assert_eq!(fact("G-Q.02_2.answer").unwrap(), "120");
        let keys: Vec<f64> = ASK.choices.iter().map(|c| c.key.parse().unwrap()).collect();
        assert_eq!(keys, vec![90.0, 120.0, 180.0], "a quarter, a third, a half");
    }

    /// 02_1 and 02_3: tilt then roll is one turn of 120 degrees about an axis
    /// that makes the same angle with all three edges, and three of them
    /// bring the book home.
    #[test]
    fn two_quarter_turns_are_a_third_of_a_turn_about_the_diagonal() {
        let (deg, axis) = single_turn();
        assert!((deg - fact_f64("G-B.tilt_then_roll.angle").unwrap()).abs() < 1e-3);
        assert!((deg - 120.0).abs() < 1e-9);
        // the single turn, and the two quarter turns, are the same matrix
        let both = model::tilt_then_roll(FRAC_PI_2);
        assert!(model::gap(&book_turn(axis, deg, 0.0, 0.0), &both) < 1e-12);
        assert!(model::gap(&book_turn(axis, 0.0, 1.0, 1.0), &both) < 1e-12);
        // equally inclined to the three edges
        let cos_each: Vec<f64> = [X, Y, Z].iter().map(|e| dot(axis, *e).abs()).collect();
        for c in &cos_each {
            assert!((c - 1.0 / 3f64.sqrt()).abs() < 1e-12, "{c}");
        }
        // three of them: home
        assert!(model::gap(&book_turn(axis, 3.0 * deg, 0.0, 0.0), &model::I3) < 1e-12);
        assert!(model::gap(&book_turn(axis, 2.0 * deg, 0.0, 0.0), &model::I3) > 0.5);
    }

    /// 02_4: each of the three pairs of turns is one turn about one axis, as
    /// Euler's theorem says (`G-X1.one_turn` tests 192 more).
    #[test]
    fn every_pair_of_turns_is_one_turn() {
        assert_eq!(fact("G-X1.one_turn").unwrap(), "192/192");
        assert_eq!(fact("G-L.euler.year").unwrap(), "1775");
        for &((ax1, d1), (ax2, d2)) in &PAIRS {
            let r = turns_in_order(&[(unit(ax1), d1 * DEG), (unit(ax2), d2 * DEG)]);
            let (angle, axis) = model::angle_axis(&r);
            assert!(angle > 1e-3, "a pair that does something");
            assert!(
                model::norm(model::sub(model::apply(&r, axis), axis)) < 1e-12,
                "the axis stays put"
            );
            assert!(model::gap(&model::turn(axis, angle), &r) < 1e-12, "one turn about it");
            // and not the sum of the two angles: the turns are about two lines
            assert!((angle / DEG - (d1 + d2)).abs() > 1.0);
        }
    }

    /// 02_5: tilt then roll is not roll then tilt; two turns about one axis
    /// are the same in both orders; a turn is undone by its opposite.
    #[test]
    fn order_matters_except_about_one_axis() {
        assert_eq!(fact("G-B.turns.same").unwrap(), "0");
        assert_eq!(fact("G-X2.same_axis_swap").unwrap(), "1");
        let (a, b) = (model::turn(X, 40.0 * DEG), model::turn(X, 70.0 * DEG));
        assert!(model::gap(&model::mul(&a, &b), &model::mul(&b, &a)) < 1e-12);
        assert!(model::gap(&model::mul(&a, &b), &model::turn(X, 110.0 * DEG)) < 1e-12);
        let undo = model::transpose(&a);
        assert!(model::gap(&model::mul(&undo, &a), &model::I3) < 1e-12);
    }

    /// 02_2: the fractions' two storeys keep to their rows (they once ran
    /// into each other), and every row ends inside the card, before the tick.
    #[test]
    fn the_quiz_rows_keep_apart_and_inside_the_card() {
        probe(|stage| {
            let kit = Kit::new()?;
            let look = QuizLook::at(v(CARD_AT[0], CARD_AT[1]), CARD_SCALE);
            let card = QuizCard::build(&kit, stage, ASK, look)?;
            let right = stage.get_right(card.back)[0];
            for pair in card.rows.windows(2) {
                let (upper, lower) = (stage.get_bottom(pair[0])[1], stage.get_top(pair[1])[1]);
                assert!(upper > lower, "two rows share height: {upper:.2} against {lower:.2}");
            }
            for &row in &card.rows {
                assert!(stage.get_right(row)[0] < right - 0.7 * CARD_SCALE);
            }
            assert!(stage.get_bottom(*card.rows.last().unwrap())[1] > stage.get_top(card.track)[1]);
            Ok(())
        });
    }

    /// The whole chapter constructs, headless: every phrase it waits for is
    /// in its line, and every label is built and placed.
    #[test]
    fn the_chapter_constructs_end_to_end() {
        let mut scene = OneTurn {
            kit: Kit::new().expect("the kit"),
        };
        fmn::run_scene(
            &mut scene,
            fmn::scene::RuntimeConfig::default(),
            7,
            &mut fmn::scene::NullSceneSink,
        )
        .expect("the chapter's scene runs");
    }

    /// 02_1 to 02_3: the word "axis" stands by the gold line's upper end,
    /// right of it, inside the safe area, off the quiz card of 02_2 and off
    /// the degree readout of 02_3.
    #[test]
    fn the_word_axis_stands_clear() {
        let view = View::new(1.0, v(BOOKS_AT[0], BOOKS_AT[1]));
        let (_, axis) = single_turn();
        let tip = axis_word_tip(&view, axis);
        // the line's two ends, as the picture draws them: about the book's
        // centre, which is held at turn height
        let (a, b) = (
            model::add(CENTRE, scale(axis, AXIS_HALF)),
            model::add(CENTRE, scale(axis, -AXIS_HALF)),
        );
        assert!(view.project(tip)[1] >= view.project(a)[1].min(view.project(b)[1]));
        assert!(
            tip == a || tip == b,
            "the word is set at an end of the gold line, not 1.25 under it"
        );
        probe(|stage| {
            let kit = Kit::new()?;
            let word = kit.text(stage, AXIS_WORD, 28.0, palette::screw(), ORIGIN)?;
            let p = view.project(tip);
            let centre = (p[0] + AXIS_WORD_SHIFT[0], p[1] + AXIS_WORD_SHIFT[1]);
            let (w, h) = (stage.get_width(word), stage.get_height(word));
            let (left, right) = (centre.0 - 0.5 * w, centre.0 + 0.5 * w);
            let (bottom, top) = (centre.1 - 0.5 * h, centre.1 + 0.5 * h);
            assert!(
                left > p[0] + 0.1,
                "the word starts {:.2} right of the line's end",
                left - p[0]
            );
            assert!(right < SAFE[0] && top < SAFE[1] && bottom > -SAFE[1]);
            let card_left = CARD_AT[0] - 0.5 * CARD_W * CARD_SCALE;
            assert!(
                right < card_left - 0.2,
                "the word ends at {right:.2}, the card at {card_left:.2}"
            );
            // clear of the book as it swings from flat to the turned pose
            // (the word stood on its crossbar at the tip taken about the
            // origin)
            let end_deg = fact_f64("G-B.tilt_then_roll.angle")?;
            for k in 0..=8 {
                let deg = end_deg * f64::from(k) / 8.0;
                let r = book_turn(axis, deg, 0.0, 0.0);
                let mut pts = Vec::new();
                for sx in [-1.0, 1.0] {
                    for sy in [-1.0, 1.0] {
                        for sz in [-1.0, 1.0] {
                            let c = model::apply(&r, [sx * BOOK_HALF[0], sy * BOOK_HALF[1], sz * BOOK_HALF[2]]);
                            pts.push(model::add(CENTRE, c));
                        }
                    }
                }
                let (lo, hi) = view.box_of(pts);
                let apart = right < lo[0] - 0.05 || left > hi[0] + 0.05 || top < lo[1] - 0.05 || bottom > hi[1] + 0.05;
                assert!(
                    apart,
                    "at {deg:.0} degrees the word ({left:.2}..{right:.2}, {bottom:.2}..{top:.2}) meets the book {lo:?} {hi:?}"
                );
            }
            // 02_3's readout: three digits at 54 points ending at READOUT_AT
            // and the degree sign after them
            let readout_left = READOUT_AT[0] - 1.8;
            let readout_right = DEGREE_AT[0] + 0.4;
            let (readout_bottom, readout_top) = (READOUT_AT[1] - 0.5, READOUT_AT[1] + 0.5);
            let apart = right < readout_left || left > readout_right || top < readout_bottom || bottom > readout_top;
            assert!(
                apart,
                "the word at ({left:.2}..{right:.2}, {bottom:.2}..{top:.2}) meets the readout"
            );
            Ok(())
        });
    }

    /// 02_1, 02_2: the table ends inside the safe area on the left and clear
    /// of the card on the right.
    #[test]
    fn the_table_fits_beside_the_card() {
        let view = View::new(1.0, v(BOOKS_AT[0], BOOKS_AT[1]));
        let corners = [
            TABLE_LO,
            [TABLE_HI[0], TABLE_LO[1]],
            TABLE_HI,
            [TABLE_LO[0], TABLE_HI[1]],
        ]
        .map(|c| [c[0], c[1], 0.0]);
        let (lo, hi) = view.box_of(corners);
        assert!(lo[0] > -SAFE[0] && hi[1] < SAFE[1] && lo[1] > -SAFE[1], "{lo:?} {hi:?}");
        let card_left = CARD_AT[0] - 0.5 * CARD_W * CARD_SCALE;
        assert!(
            hi[0] < card_left - 0.5,
            "the table ends at {:.2}, the card begins at {card_left:.2}",
            hi[0]
        );
    }

    /// 02_6: the live grid and its brackets stand above the code panel that
    /// slides in at the end (they once sat behind its first line).
    #[test]
    fn the_grid_stands_above_the_code_panel() {
        probe(|stage| {
            let kit = Kit::new()?;
            let panel = CodePanel::build(&kit, stage, &CODE, 26.0, 5.6)?;
            let (left, top) = (
                stage.get_left(panel.group)[0] - (panel.width + 0.45),
                stage.get_top(panel.group)[1],
            );
            let grid_bottom = GRID_AT[1] - 1.5 * GRID_PITCH.1;
            let grid_right = GRID_AT[0] + 1.62 * GRID_PITCH.0;
            assert!(grid_right > left, "the panel really is under the grid's columns");
            assert!(
                grid_bottom - 0.3 > top,
                "the grid's last row ends at {grid_bottom:.2}, the panel's top is {top:.2}"
            );
            assert!(GRID_AT[1] + 1.5 * GRID_PITCH.1 < SAFE[1]);
            Ok(())
        });
    }

    /// 02_6: the grid the live cells show after tilt and roll is the fact's
    /// grid, cell for cell; the columns are where the edges point.
    #[test]
    fn the_grid_after_two_turns_is_the_facts_grid() {
        let (_, axis) = single_turn();
        let live = grid_of(&book_turn(axis, 0.0, 1.0, 1.0));
        let want = parse_grid(fact("G-B.tilt_then_roll.R").unwrap()).unwrap();
        assert_eq!(live, want);
        // the columns are the images of the three edges
        let r = book_turn(axis, 0.0, 1.0, 1.0);
        for (j, e) in [X, Y, Z].iter().enumerate() {
            let col = model::apply(&r, *e);
            assert_eq!([r[0][j], r[1][j], r[2][j]], col);
        }
        // before any turn the grid is the identity
        let id = grid_of(&book_turn(axis, 0.0, 0.0, 0.0));
        assert_eq!(id[0], ["1".to_owned(), "0".to_owned(), "0".to_owned()]);
        // the product of the two grids is the third, in the order the tag says
        let product = model::mul(&model::roll_turn(FRAC_PI_2), &model::tilt_turn(FRAC_PI_2));
        assert_eq!(grid_of(&product), want);
    }

    #[test]
    fn entries_print_with_the_engines_minus_and_no_negative_zero() {
        // on screen beside live numbers: the engine's minus, U+2013
        assert_eq!(screen_entry(-1.0), "–1");
        assert_eq!(screen_entry(-0.7213), "–0.72");
        assert_eq!(screen_entry(-0.0), "0");
        assert_eq!(screen_entry(-1e-17), "0");
        assert_eq!(screen_entry(0.5), "0.50");
        // the plain entry keeps a true minus sign
        assert_eq!(entry(-1.0), "−1");
        assert_eq!(entry(-0.7213), "−0.72");
    }
}
