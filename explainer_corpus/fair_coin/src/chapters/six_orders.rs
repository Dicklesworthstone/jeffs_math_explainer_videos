//! Chapter 09, six orders: three rolls can make a fair die; the six-way
//! square grows into the cube where it stands, cut by the loaded die's
//! chances along every edge (09_1); the six orders of a four, a five and a
//! six are six bricks in it, lit one at a time (09_2), and they are one brick
//! turned six ways: out of the cube each still lies its own way, then each
//! turns into the same pose (09_3); so the order three different rolls came
//! in, low, middle and high, is a fair choice among six and picks a face
//! (09_4), six thousand seeded rolls say so (09_5), and it keeps going at a
//! rising price (09_6).

use super::draw::predict_pause;
use explainer_kit::{Kit, Palette, Timed, fade_all, hex, play, thousands, tween, v};
use fmn::prelude::*;

use super::draw::{
    BoxSpec, CUBE_SIDE, Cube, CubePose, CubeRoom, RulerEdge, SQUARE_AT, SQUARE_SIDE, alice, bob, column, cube_ruler,
    cut_boxes, die_face, fade_in_all, fade_in_lagged, fade_out_all, faded_line, fail, flat, gold, group, line_style,
    panel, piece_grey, say, thrown, thrown_lit,
};
use crate::facts::loaded_run;
use crate::model::{LOADED, Q, all_different, blocks, die, order_index, order_of_rolls, tally, two_sigma_band};

pub struct SixOrders {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const ROLL_NAMES: [&str; 3] = ["first roll", "second roll", "third roll"];
const IN_SOME_ORDER: &str = "in some order";
const SAME_ROLLS: &str = "the same rolls, any order: the same chance";
const RANKS: [&str; 3] = ["low", "middle", "high"];
const A_MATCH: &str = "two rolls match: roll three fresh ones";
const ROLLS: &str = "rolls";
const THREES: &str = "threes of rolls:";
const ALL_DIFFER: &str = "with three different rolls,";
const BY_ORDER: &str = "sorted by their order";
const EACH_FACE: &str = "each face:";
const TO: &str = "to";
const NINETEEN: &str = "about 19 times in 20 (model)";
const ORDERS: &str = "orders";
const ABOUT: &str = "about";
const ROLLS_EACH: &str = "rolls each";

/// Where the bricks stand once out of the cube, three to a row: the first
/// column's centre on screen and from one column to the next, the two rows'
/// heights, and how deep the bricks' centres sit (in sides).
const OUT_X: (f64, f64) = (0.3, 2.45);
const OUT_Y: [f64; 2] = [1.5, -1.3];
const OUT_DEPTH: f64 = 0.3;
/// The six order cards while they are the subject (09_4): from one to the
/// next, their size and height, where the fair die's faces stand over them,
/// and the cube's side when its bricks stand on them.
const CARD_PITCH: f64 = 2.1;
const BIG_CARD: (f64, f64) = (1.9, 1.45);
const BIG_CARD_Y: f64 = -1.25;
const BIG_DIE: f64 = 0.85;
const BIG_DIE_Y: f64 = 0.05;
const SMALL_SIDE: f64 = 1.5;
/// The examples of 09_4: their dice's height, and the foot of their charts.
const EXAMPLE_Y: f64 = 2.75;
const EXAMPLE_FOOT: f64 = 1.15;
/// The six columns of the fair die under its bars (09_5): the first one's
/// centre and from one to the next; where a die face and its card sit; the
/// foot of the bars and how much room they have.
const COLUMN: (f64, f64) = (-0.9, 1.35);
const DIE_Y: f64 = -1.1;
const DIE_SIZE: f64 = 0.7;
const CARD_Y: f64 = -2.1;
const BAR_FOOT: f64 = -0.55;
const BAR_ROOM: f64 = 3.1;
/// The left-hand column beside the bars: its middle and its left edge.
const LEFT_MID: f64 = -4.5;
const LEFT_EDGE: f64 = -6.6;

/// Where each brick's label stands while the brick is in the cube (by its
/// place among the six orders), outside the pieces where the room allows; a
/// short leader runs to the brick.
const LABEL_AT: [(f64, f64); 6] = [
    (-1.0, 1.3),
    (-1.3, -0.75),
    (-2.95, 2.85),
    (-5.1, -0.85),
    (-5.35, 2.35),
    (-6.55, 0.5),
];

fn over(run_time: f64) -> AnimateArgs {
    AnimateArgs {
        run_time: Some(run_time),
        ..AnimateArgs::default()
    }
}

fn ground() -> Srgb {
    hex("#0E1116")
}

fn column_x(j: usize) -> f64 {
    COLUMN.0 + COLUMN.1 * j as f64
}

fn card_x(j: usize) -> f64 {
    CARD_PITCH * (j as f64 - 2.5)
}

/// Where the brick with place `slot` among the six stands once out of the
/// cube.
fn out_at(slot: usize) -> Vec3 {
    let per_row = OUT_Y.len() + 1;
    v(OUT_X.0 + OUT_X.1 * (slot % per_row) as f64, OUT_Y[slot / per_row])
}

/// A quick swell and back, for a thing that keeps its colours.
fn swell(m: Mob, factor: f64, run_time: f64) -> fmn::Result<AnimBuilder> {
    Ok(m.animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(run_time),
            rate_func: Some(fmn::core::rate::there_and_back),
            ..AnimateArgs::default()
        })?
        .scale(factor)?)
}

/// The shift (in sides) that puts a box whose own centre is `centre` at the
/// screen point `to`, `depth` sides deep.
fn shift_to(pose: &CubePose, centre: [f64; 3], to: Vec3, depth: f64) -> [f64; 3] {
    let flat = pose.project([0.0, 0.0, depth]);
    [
        (to[0] - flat[0]) / pose.side - centre[0],
        (to[1] - flat[1]) / pose.side - centre[1],
        depth - centre[2],
    ]
}

/// One brick: its place in `Cube::boxes`, its box, the three rolls it
/// stands for in the order they came, and its place among the six orders
/// (the fair die's face, less one).
#[derive(Clone, Copy)]
struct Brick {
    index: usize,
    spec: BoxSpec,
    order: [usize; 3],
    slot: usize,
}

/// Where a brick's three sides show on screen, standing with its centre at
/// `centre` (on screen) and sides `size` (scene units, across, up, back):
/// under its front's bottom edge, beside its front's left edge, beside its
/// floor's right edge going back, and over its top.
fn side_spots(centre: Vec3, size: [f64; 3]) -> [Vec3; 4] {
    let (dx, dy) = (0.46 * size[2] / 2.0, 0.34 * size[2] / 2.0);
    let front = v(centre[0] - dx, centre[1] - dy);
    let back_right = v(front[0] + size[0] / 2.0 + dx, front[1] - size[1] / 2.0 + dy);
    [
        v(front[0], front[1] - size[1] / 2.0 - 0.2),
        v(front[0] - size[0] / 2.0 - 0.32, front[1]),
        v(back_right[0] + 0.3, back_right[1] - 0.18),
        v(centre[0], centre[1] + size[1] / 2.0 + dy + 0.28),
    ]
}

/// Each roll's rank among the three: 0 the lowest.
fn ranks(rolls: &[usize]) -> Vec<usize> {
    rolls.iter().map(|r| rolls.iter().filter(|o| *o < r).count()).collect()
}

/// An order as a card: three bars, low, middle and high, in the order the
/// rolls came, and no digits (the face comes from the order alone). One
/// group, the back first.
fn order_card(stage: &mut Stage<'_>, order: &[usize], at: Vec3) -> fmn::Result<Mob> {
    let (w, h) = BIG_CARD;
    let back = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.14)
        .style(Style::default().fill(panel(), 0.94).stroke(Palette::faint(), 1.5, 1.0))
        .build()?
        .moved_to(at);
    let mut parts = vec![stage.arena_mut().add(back)];
    for (i, rank) in ranks(order).into_iter().enumerate() {
        let foot = v(at[0] + 0.5 * (i as f64 - 1.0), at[1] - 0.48);
        let bar = column(foot, 0.3, 0.26 + 0.29 * rank as f64, Palette::dim());
        parts.push(stage.arena_mut().add(bar));
    }
    group(stage, &parts)
}

/// Three rolls as dice with, when they all differ, a small chart that ranks
/// them: a bar and a word under each. Detached.
fn example(kit: &Kit, stage: &mut Stage<'_>, rolls: &[usize]) -> fmn::Result<Vec<Mob>> {
    let differ = order_of_rolls(rolls).is_some();
    let back = if differ { Palette::ink() } else { thrown_lit() };
    let mut out = Vec::new();
    for (i, (&roll, rank)) in rolls.iter().zip(ranks(rolls)).enumerate() {
        let x = 1.3 * (i as f64 - 1.0);
        out.push(die_face(stage, roll, v(x, EXAMPLE_Y), 0.8, back, ground())?);
        if differ {
            let bar = column(v(x, EXAMPLE_FOOT), 0.4, 0.3 + 0.28 * rank as f64, Palette::dim());
            out.push(stage.arena_mut().add(bar));
            out.push(kit.text(stage, RANKS[rank], 22.0, Palette::ink(), v(x, EXAMPLE_FOOT - 0.25))?);
        }
    }
    Ok(out)
}

/// A chance as TeX: `\frac{1}{5}`.
fn frac(q: Q) -> String {
    match q.exact().split_once('/') {
        Some((n, d)) => format!(r"\frac{{{n}}}{{{d}}}"),
        None => q.exact(),
    }
}

/// The running totals of some chances, from 0 to 1.
fn running<'a>(chances: impl Iterator<Item = &'a Q>) -> Vec<f64> {
    let mut total = Q::ZERO;
    let mut out = vec![0.0];
    for &c in chances {
        total = total + c;
        out.push(total.f64());
    }
    out
}

/// Words laid left to right from `left` at height `y`, a small gap apart;
/// detached.
fn words_in_a_line(
    kit: &Kit,
    stage: &mut Stage<'_>,
    parts: &[String],
    size: f64,
    color: Srgb,
    left: f64,
    y: f64,
) -> fmn::Result<Vec<Mob>> {
    let mut out = Vec::new();
    let mut x = left;
    for p in parts {
        let m = kit.text(stage, p, size, color, v(0.0, y))?;
        let w = stage.get_width(m);
        stage.move_to(m, v(x, y), LEFT);
        x += w + 0.13;
        out.push(m);
    }
    Ok(out)
}

fn tweens(stage: &mut Stage<'_>, moves: &[(Mob, f64)], run_time: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
    let mut out: Vec<Box<dyn Animation>> = Vec::new();
    for &(t, value) in moves {
        out.push(stage.prepare(tween(t, value, run_time)?)?);
    }
    Ok(out)
}

fn fades_in(stage: &mut Stage<'_>, mobs: &[Mob], run_time: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
    let mut out: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        out.push(stage.prepare(a.rt(run_time))?);
    }
    Ok(out)
}

fn fades_out(stage: &mut Stage<'_>, mobs: &[Mob], run_time: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
    let mut out: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        out.push(stage.prepare(a.rt(run_time))?);
    }
    Ok(out)
}

/// Move `m` to `to` (its centre) and scale it by `factor`, `rt` seconds.
fn move_scaled(stage: &mut Stage<'_>, m: Mob, to: Vec3, factor: f64, rt: f64) -> fmn::Result<Box<dyn Animation>> {
    stage.prepare(
        m.animate()
            .set_anim_args(over(rt))?
            .scale(factor)?
            .move_to(to, ORIGIN)?,
    )
}

impl SceneConstruct for SixOrders {
    fn name(&self) -> &str {
        "09_six_orders"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let chances = die(&LOADED);
        let faces = LOADED.len();
        // the three rolls of the chapter: the three biggest faces
        let trio = [faces - 2, faces - 1, faces];
        // along the first two rolls a bigger roll's stretch is nearer the
        // corner, as in the square; along the third it is nearer the back,
        // where the three drawn faces meet, so that no brick hides another
        let big = running(chances.iter().rev());
        let small = running(chances.iter());
        let along = |cuts: &[f64], face: usize, big_first: bool| -> (f64, f64) {
            let k = if big_first { faces - face } else { face - 1 };
            (cuts[k], cuts[k + 1])
        };

        // the six-way square of chapter 08, in colour: here a cube with no
        // depth yet, its thirty-six columns coloured as the square's blocks
        let line = say(&kit, stage, "09_1")?;
        let whole = [0.0, 1.0];
        let mut specs: Vec<(BoxSpec, Srgb)> = cut_boxes([&big, &big, &whole])
            .into_iter()
            .map(|(spec, place)| {
                let color = match place[0].cmp(&place[1]) {
                    std::cmp::Ordering::Less => alice(),
                    std::cmp::Ordering::Greater => bob(),
                    std::cmp::Ordering::Equal => thrown(),
                };
                (spec, color)
            })
            .collect();
        let columns: Vec<BoxSpec> = specs.iter().map(|&(s, _)| s).collect();
        let mut bricks = Vec::new();
        for (spec, place) in cut_boxes([&big, &big, &small]) {
            let order = [faces - place[0], faces - place[1], place[2] + 1];
            let mut sorted = order;
            sorted.sort_unstable();
            if sorted == trio {
                bricks.push(Brick {
                    index: 0,
                    spec,
                    order,
                    slot: order_index(&order),
                });
                specs.push((spec, piece_grey()));
            }
        }
        let mut cube = Cube::build(stage, SQUARE_AT, SQUARE_SIDE, 0.0, &specs)?;
        for b in &mut bricks {
            b.index = cube
                .find(b.spec)
                .ok_or_else(|| fail("a brick of the cube is missing".into()))?;
            // out of the cube every brick turns to stand the same way: the
            // five's side across, the six's up, the four's back
            let side_of = |face: usize| b.order.iter().position(|&r| r == face).unwrap_or(0);
            cube.set_turn(b.index, [side_of(trio[1]), side_of(trio[2]), side_of(trio[0])]);
        }
        bricks.sort_by_key(|b| b.slot);
        let column_boxes: Vec<usize> = columns.iter().filter_map(|&s| cube.find(s)).collect();
        for b in &bricks {
            stage.set_tracker_value(cube.boxes[b.index].light, 1.0)?;
            stage.set_tracker_value(cube.boxes[b.index].shown, 0.0)?;
        }
        cube.install(stage)?;
        let room = CubeRoom::build(stage, cube.view, [&big, &big, &small])?;
        let digit_edge = |big_first: bool, cuts: &[f64], named: &[usize], name: &str| RulerEdge {
            stretches: (1..=faces)
                .map(|f| {
                    let (a, b) = along(cuts, f, big_first);
                    (a, b, Palette::ink())
                })
                .collect(),
            words: (1..=faces).map(|f| named.contains(&f).then(|| f.to_string())).collect(),
            word_color: Some(Palette::ink()),
            word_size: 20.0,
            word_out: 0.38,
            name: Some(name.into()),
        };
        let all: Vec<usize> = (1..=faces).collect();
        let ruler = cube_ruler(
            &kit,
            stage,
            cube.view,
            [
                digit_edge(true, &big, &all, ROLL_NAMES[0]),
                digit_edge(true, &big, &all, ROLL_NAMES[1]),
                digit_edge(false, &small, &trio, ROLL_NAMES[2]),
            ],
            0.022,
        )?;
        let mut shown = room.mobs();
        shown.extend(cube.mobs());
        stage.scene_mut().add(&shown)?;
        let mut flat_parts = vec![ruler.anchor];
        flat_parts.extend(ruler.stretches[0].iter().chain(&ruler.stretches[1]));
        flat_parts.extend(ruler.words[0].iter().chain(&ruler.words[1]));
        flat_parts.extend(&ruler.names[..2]);
        fade_in_all(stage, &flat_parts, 0.7)?;
        // the square grows into the cube where it stands, as in chapter 06
        line.wait_for(stage, "Three need the cube again")?;
        stage.scene_mut().add(&ruler.stretches[2])?;
        let mut grow = tweens(stage, &[(cube.view.side, CUBE_SIDE), (cube.view.depth, 1.0)], 1.5)?;
        let mut third = ruler.words[2].clone();
        third.push(ruler.names[2]);
        grow.extend(fades_in(stage, &third, 1.5)?);
        // the one and the two would crowd each other at the cube's size:
        // their stretches stay, their digits go
        let crowded: Vec<Mob> = ruler.words[..2].iter().flat_map(|w| w[..2].to_vec()).collect();
        grow.extend(fades_out(stage, &crowded, 0.6)?);
        stage.play_prepared(grow)?;
        let pose = cube.view.pose(stage);

        // 09_2: a four, a five and a six, in some order: six bricks
        let line = say(&kit, stage, "09_2")?;
        let dice_at = |i: usize| v(3.6 + 1.2 * (i as f64 - 1.0), 2.1);
        let mut rolls_shown = Vec::new();
        for (i, (&face, phrase)) in trio.iter().zip(["a four", "a five", "a six"]).enumerate() {
            line.wait_for(stage, phrase)?;
            let d = die_face(stage, face, dice_at(i), 0.85, Palette::ink(), ground())?;
            play!(stage; fade_in(stage.arena_mut(), d, v(0.0, -0.2), 1.0)?.rt(0.4));
            rolls_shown.push(d);
        }
        line.wait_for(stage, "in some order")?;
        let some_order = kit.text(stage, IN_SOME_ORDER, 26.0, Palette::dim(), v(3.6, 1.25))?;
        fade_in_all(stage, &[some_order], 0.4)?;
        rolls_shown.push(some_order);
        // the bricks light one at a time, each with its order on a leader;
        // the colours leave the cube as the first lights, and the tick
        // digits with them
        line.wait_for(stage, "Each order lands")?;
        let mut labels = Vec::new();
        let mut leaders = Vec::new();
        let mut label_spots = Vec::new();
        for b in &bricks {
            let text = b.order.map(|r| r.to_string()).join(" ");
            let (lx, ly) = LABEL_AT[b.slot];
            let label = kit.text(stage, &text, 24.0, Palette::ink(), v(lx, ly))?;
            stage.set_z_index(label, 2, true);
            labels.push(label);
            let leader = stage.arena_mut().add(
                polyline(vec![v(lx, ly), v(lx + 0.1, ly)])
                    .style(faded_line(gold(), 2.0, 0.9))
                    .build(),
            );
            stage.set_z_index(leader, 2, true);
            leaders.push(leader);
            label_spots.push(v(lx, ly));
        }
        // the leaders follow their bricks while the cube rocks
        let leader_anchor = stage.arena_mut().add(VMobject::new());
        {
            let (view, leaders, label_spots) = (cube.view, leaders.clone(), label_spots.clone());
            let ends: Vec<[f64; 3]> = bricks.iter().map(|b| b.spec.center()).collect();
            stage.add_updater(
                leader_anchor,
                move |s, _| {
                    let val = |m: Mob| s.tracker_value(m).unwrap_or(0.0);
                    let pose = CubePose {
                        left: val(view.left),
                        bottom: val(view.bottom),
                        side: val(view.side),
                        depth: val(view.depth),
                    };
                    for ((&m, &from), &end) in leaders.iter().zip(&label_spots).zip(&ends) {
                        let to = pose.project(end);
                        let (dx, dy) = (to[0] - from[0], to[1] - from[1]);
                        let len = dx.hypot(dy).max(1e-6);
                        let start = v(from[0] + dx / len * 0.36, from[1] + dy / len * 0.2);
                        let _ = s.set_points(m, polyline(vec![start, to]).build().points());
                    }
                },
                true,
            )?;
        }
        stage.add_to_scene(leader_anchor)?;
        let mut digits: Vec<Mob> = ruler
            .words
            .iter()
            .flatten()
            .copied()
            .filter(|m| !crowded.contains(m))
            .collect();
        digits.extend([leader_anchor]);
        for (k, b) in bricks.iter().enumerate() {
            let mut light = tweens(stage, &[(cube.boxes[b.index].shown, 1.0)], 0.4)?;
            light.extend(fades_in(stage, &[labels[k], leaders[k]], 0.4)?);
            if k == 0 {
                let gone: Vec<(Mob, f64)> = column_boxes.iter().map(|&i| (cube.boxes[i].shown, 0.0)).collect();
                light.extend(tweens(stage, &gone, 0.6)?);
                light.extend(fades_out(stage, &digits[..digits.len() - 1], 0.4)?);
            }
            stage.play_prepared(light)?;
        }
        // the colour columns are gone: off the stage
        let column_faces: Vec<Mob> = column_boxes.iter().flat_map(|&i| cube.faces(i)).collect();
        stage.scene_mut().remove(&column_faces);
        // the cube rocks a little and back, so the bricks' places can be read
        let rock = cube
            .view
            .depth
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(1.4),
                rate_func: Some(fmn::core::rate::there_and_back),
                ..AnimateArgs::default()
            })?
            .set_value(0.72)?;
        stage.play(rock)?;
        let q = predict_pause(&kit, stage, v(3.6, -0.6))?;

        // 09_3: one brick, turned six ways: out of the cube each still lies
        // its own way, its three sides marked; then each turns to the same pose
        let line = say(&kit, stage, "09_3")?;
        let mut gone = rolls_shown.clone();
        gone.extend(leaders.iter().chain(&ruler.names));
        gone.extend([q, leader_anchor]);
        fade_out_all(stage, &gone, 0.3)?;
        let side = pose.side;
        let size_of = |b: &Brick| b.spec.size.map(|x| x * side);
        let turned_size = [1, 2, 0].map(|i| chances[trio[i] - 1].f64() * side);
        let mut out: Vec<Box<dyn Animation>> = Vec::new();
        for (b, &label) in bricks.iter().zip(&labels) {
            let to = out_at(b.slot);
            let shift = shift_to(&pose, b.spec.center(), to, OUT_DEPTH);
            let moves: Vec<(Mob, f64)> = cube.boxes[b.index].shift.into_iter().zip(shift).collect();
            out.extend(tweens(stage, &moves, 1.3)?);
            out.push(move_scaled(stage, label, side_spots(to, size_of(b))[3], 1.0, 1.3)?);
        }
        stage.play_prepared(out)?;
        let brick_faces: Vec<Mob> = bricks.iter().flat_map(|b| cube.faces(b.index)).collect();
        stage.scene_mut().bring_to_front(&brick_faces)?;
        stage.scene_mut().bring_to_front(&labels)?;
        // the same three numbers on every brick, in different places
        line.wait_for(stage, "A tenth")?;
        let side_text = |face: usize| chances[face - 1].exact();
        let mut marks = Vec::new();
        for b in &bricks {
            let spots = side_spots(out_at(b.slot), size_of(b));
            for (i, &face) in b.order.iter().enumerate() {
                marks.push(kit.text(stage, &side_text(face), 22.0, Palette::dim(), spots[i])?);
            }
        }
        fade_in_all(stage, &marks, 0.5)?;
        // then one after another they turn, never folding flat, to stand
        // the same way
        line.wait_for(stage, "The same three rolls")?;
        fade_out_all(stage, &marks, 0.3)?;
        let mut turns: Vec<Box<dyn Animation>> = Vec::new();
        for (b, &label) in bricks.iter().zip(&labels) {
            let mut both = tweens(stage, &[(cube.boxes[b.index].turn, 1.0)], 1.0)?;
            both.push(move_scaled(
                stage,
                label,
                side_spots(out_at(b.slot), turned_size)[3],
                1.0,
                1.0,
            )?);
            turns.push(Box::new(AnimationGroup::new(stage.arena_mut(), both)?));
        }
        let turns = AnimationGroup::with_lag_ratio(stage.arena_mut(), turns, 0.28)?.with_run_time(2.4);
        stage.play(turns)?;
        stage.scene_mut().bring_to_front(&labels)?;
        // now the same numbers in the same places, and only then the product
        let mut same_marks = Vec::new();
        for b in &bricks {
            let spots = side_spots(out_at(b.slot), turned_size);
            for (i, &face) in [trio[1], trio[2], trio[0]].iter().enumerate() {
                same_marks.push(kit.text(stage, &side_text(face), 22.0, Palette::dim(), spots[i])?);
            }
        }
        let sides = trio.map(|face| chances[face - 1]);
        let volume = sides.iter().fold(Q::ONE, |acc, &x| acc * x);
        let product = sides.map(frac).join(r" \times ");
        let product = format!("{product} = {}", frac(volume));
        let out_mid = OUT_X.0 + OUT_X.1;
        let product = kit.tex(stage, &product, 34.0, Palette::ink(), &[], v(out_mid, -3.3))?;
        let same = kit.text(stage, SAME_ROLLS, 30.0, gold(), v(out_mid, 3.35))?;
        let mut ins = same_marks.clone();
        ins.extend([product, same]);
        fade_in_all(stage, &ins, 0.6)?;
        kit.music_breath(stage, explainer_kit::animatic::HOLD_S)?;

        // 09_4: the order three different rolls came in is a fair choice
        // among six, and it picks a face
        let line = say(&kit, stage, "09_4")?;
        let mut gone = room.mobs();
        gone.extend(ruler.stretches.iter().flatten());
        gone.extend(same_marks.iter().chain([&product, &same]));
        fade_out_all(stage, &gone, 0.4)?;
        // the bricks, small, stand in a row where the cards will be
        let small_pose = CubePose {
            side: SMALL_SIDE,
            ..pose
        };
        let card_top = BIG_CARD_Y + BIG_CARD.1 / 2.0;
        let stand_size = turned_size.map(|x| x * SMALL_SIDE / side);
        let mut settle = tweens(stage, &[(cube.view.side, SMALL_SIDE), (cube.rim, 0.6)], 1.0)?;
        for (b, &label) in bricks.iter().zip(&labels) {
            let to = v(card_x(b.slot), card_top + stand_size[1] / 2.0 + 0.2);
            let shift = shift_to(&small_pose, b.spec.center(), to, OUT_DEPTH);
            let moves: Vec<(Mob, f64)> = cube.boxes[b.index].shift.into_iter().zip(shift).collect();
            settle.extend(tweens(stage, &moves, 1.0)?);
            settle.push(move_scaled(stage, label, v(card_x(b.slot), to[1] + 0.85), 1.0, 1.0)?);
        }
        stage.play_prepared(settle)?;
        // each takes its order's card: three bars, low, middle and high, in
        // the order the rolls came; the digits go, since the face never
        // depends on which numbers came up
        line.wait_for(stage, "the order they came in")?;
        let mut cards = Vec::new();
        for b in &bricks {
            cards.push(order_card(stage, &b.order, v(card_x(b.slot), BIG_CARD_Y))?);
        }
        let mut swap = fades_out(stage, &labels, 0.5)?;
        let lagged_cards = {
            let mut each: Vec<Box<dyn Animation>> = Vec::new();
            for &c in &cards {
                let a = fade_in(stage.arena_mut(), c, ORIGIN, 1.0)?;
                each.push(stage.prepare(a)?);
            }
            AnimationGroup::with_lag_ratio(stage.arena_mut(), each, 0.15)?.with_run_time(1.0)
        };
        swap.push(Box::new(lagged_cards));
        stage.play_prepared(swap)?;
        stage.scene_mut().bring_to_front(&cards)?;
        // and each brick becomes a face of a fair die
        line.wait_for(stage, "and it picks a face")?;
        let mut turn_into: Vec<Box<dyn Animation>> = Vec::new();
        let mut fair_dice = Vec::new();
        for b in &bricks {
            turn_into.extend(tweens(stage, &[(cube.boxes[b.index].shown, 0.0)], 0.8)?);
            let at = v(card_x(b.slot), BIG_DIE_Y);
            let d = die_face(stage, b.slot + 1, at, BIG_DIE, Palette::ink(), ground())?;
            turn_into.extend(fades_in(stage, &[d], 0.8)?);
            fair_dice.push(d);
        }
        stage.play_prepared(turn_into)?;
        // two orders, then a match, each shown as three dice
        let ring = |stage: &mut Stage<'_>, slot: usize| -> fmn::Result<Mob> {
            let high = BIG_DIE_Y + BIG_DIE / 2.0 + 0.1;
            let low = BIG_CARD_Y - BIG_CARD.1 / 2.0 - 0.1;
            let around = Rectangle::new()
                .width(BIG_CARD.0 + 0.2)
                .height(high - low)
                .corner_radius(0.16)
                .style(line_style(gold(), 3.0))
                .build()?
                .moved_to(v(card_x(slot), (high + low) / 2.0));
            Ok(stage.arena_mut().add(around))
        };
        // low, high, middle with the chapter's three rolls; high, low,
        // middle with three others
        let spoken = [
            ([trio[0], trio[2], trio[1]], "Low, high, middle"),
            ([faces, 1, 3], "High, low, middle"),
        ];
        let mut showing: Vec<Mob> = Vec::new();
        for (rolls, phrase) in spoken {
            line.wait_for(stage, phrase)?;
            let slot = order_of_rolls(&rolls).ok_or_else(|| fail("the example's rolls match".into()))?;
            let mut next = example(&kit, stage, &rolls)?;
            next.push(ring(stage, slot)?);
            let mut swap = fades_out(stage, &showing, 0.4)?;
            swap.extend(fades_in(stage, &next, 0.5)?);
            stage.play_prepared(swap)?;
            showing = next;
        }
        line.wait_for(stage, "If any two match")?;
        // the run's first three rolls in a row that do not all differ
        let run = loaded_run();
        let matching = run
            .chunks_exact(trio.len())
            .find(|b| order_of_rolls(b).is_none())
            .ok_or_else(|| fail("no three rolls of the run match".into()))?;
        let mut next = example(&kit, stage, matching)?;
        next.push(kit.text(stage, A_MATCH, 24.0, Palette::dim(), v(0.0, EXAMPLE_Y - 0.95))?);
        let mut swap = fades_out(stage, &showing, 0.4)?;
        swap.extend(fades_in(stage, &next, 0.5)?);
        stage.play_prepared(swap)?;
        let showing = next;

        // 09_5: six thousand rolls made six hundred fifty fair ones; the
        // cards and faces step down under six bars
        say(&kit, stage, "09_5")?;
        let mut step: Vec<Box<dyn Animation>> = fades_out(stage, &showing, 0.4)?;
        for (j, (&c, &d)) in cards.iter().zip(&fair_dice).enumerate() {
            step.push(move_scaled(stage, c, v(column_x(j), CARD_Y), 0.6, 1.0)?);
            step.push(move_scaled(stage, d, v(column_x(j), DIE_Y), DIE_SIZE / BIG_DIE, 1.0)?);
        }
        stage.play_prepared(step)?;
        let made = blocks(&run, trio.len(), order_of_rolls);
        let counts = tally(&made, bricks.len());
        let (low, high) = two_sigma_band(made.len() as u32, Q::new(1, bricks.len() as i128));
        let unit = BAR_ROOM / f64::from(high);
        let caption = [
            format!("{} {THREES}", thousands((run.len() / trio.len()) as u64)),
            format!("{} {ALL_DIFFER}", made.len()),
            BY_ORDER.to_owned(),
        ];
        let caption: Vec<Mob> = caption
            .iter()
            .enumerate()
            .map(|(i, l)| kit.text(stage, l, 26.0, Palette::ink(), v(LEFT_MID, 0.75 - 0.42 * i as f64)))
            .collect::<fmn::Result<_>>()?;
        // the bars are tallies of the run as its fair rolls land, in order
        let landed = stage.add_value_tracker(0.0);
        let mut prefix = vec![vec![0usize; counts.len()]];
        for &face in &made {
            let mut next = prefix[prefix.len() - 1].clone();
            next[face] += 1;
            prefix.push(next);
        }
        let prefix = std::rc::Rc::new(prefix);
        let bars: Vec<Mob> = (0..counts.len())
            .map(|j| {
                let bar = column(v(column_x(j), BAR_FOOT), 0.8, 0.004, Palette::dim());
                stage.arena_mut().add(bar)
            })
            .collect();
        let bar_anchor = stage.arena_mut().add(VMobject::new());
        {
            let (bars, prefix) = (bars.clone(), std::rc::Rc::clone(&prefix));
            let mut last = None;
            stage.add_updater(
                bar_anchor,
                move |s, _| {
                    let n = (s.tracker_value(landed).unwrap_or(0.0).floor().max(0.0) as usize).min(prefix.len() - 1);
                    if last == Some(n) {
                        return;
                    }
                    for (j, &m) in bars.iter().enumerate() {
                        let h = unit * prefix[n][j] as f64;
                        let bar = column(v(column_x(j), BAR_FOOT), 0.8, h, Palette::dim());
                        let _ = s.set_points(m, bar.points());
                    }
                    last = Some(n);
                },
                true,
            )?;
        }
        let mut ins = caption.clone();
        ins.push(bar_anchor);
        ins.extend(&bars);
        fade_in_all(stage, &ins, 0.4)?;
        let mut tallies = Vec::new();
        for j in 0..counts.len() {
            let book = std::rc::Rc::clone(&kit.book);
            let prefix = std::rc::Rc::clone(&prefix);
            let at = v(column_x(j), BAR_FOOT + BAR_ROOM + 0.14);
            let m = stage.always_redraw(move |s| {
                let n = (s.tracker_value(landed).unwrap_or(0.0).floor().max(0.0) as usize).min(prefix.len() - 1);
                let d = DecimalNumber::new(prefix[n][j] as f64)
                    .num_decimal_places(0)
                    .font_size(26.0)
                    .color(Palette::ink())
                    .build(&book)
                    .expect("bundled digits always lay out");
                let m = s.add(d.into_vmob());
                s.move_to(m, at, DOWN);
                m
            });
            stage.add_to_scene(m)?;
            tallies.push(m);
        }
        let pour = landed
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(2.2),
                rate_func: Some(fmn::core::rate::linear),
                ..AnimateArgs::default()
            })?
            .set_value(made.len() as f64)?;
        stage.play(pour)?;
        // where one face of a fair die lands about 19 times in 20
        let (band_left, band_right) = (column_x(0) - 0.6, column_x(counts.len() - 1) + 0.6);
        let (band_low, band_high) = (BAR_FOOT + unit * f64::from(low), BAR_FOOT + unit * f64::from(high));
        let strip = Rectangle::new()
            .width(band_right - band_left)
            .height(band_high - band_low)
            .style(flat(Palette::ink(), 0.08))
            .build()?
            .moved_to(v((band_left + band_right) / 2.0, (band_low + band_high) / 2.0));
        // the tint lies behind the bars; only its two thin rules cross them
        let strip = stage.arena_mut().add(strip);
        stage.set_z_index(strip, -1, true);
        let mut band = vec![strip];
        for y in [band_low, band_high] {
            let rule = Line::new(v(band_left, y), v(band_right, y))
                .style(faded_line(Palette::ink(), 1.5, 0.55))
                .build()?;
            band.push(stage.arena_mut().add(rule));
        }
        let mid = (band_low + band_high) / 2.0;
        let first = format!("{EACH_FACE} {low} {TO} {high},");
        for (text, y) in [(first.as_str(), mid + 0.17), (NINETEEN, mid - 0.17)] {
            let m = kit.text(stage, text, 22.0, Palette::dim(), v(0.0, y))?;
            stage.move_to(m, v(band_left - 0.2, y), RIGHT);
            band.push(m);
        }
        fade_in_all(stage, &band, 0.5)?;

        // 09_6: it keeps going, and each step costs more
        let line = say(&kit, stage, "09_6")?;
        let factors = |n: usize| -> Vec<String> {
            let mut out = vec![format!("{n} {ROLLS}:"), n.to_string()];
            out.extend((1..n).rev().map(|k| format!("× {k}")));
            out
        };
        let orders = |n: usize| format!("= {} {ORDERS}", (1..=n).product::<usize>());
        let cost = |n: usize| -> fmn::Result<String> {
            let works = all_different(&chances, n);
            if works.is_zero() {
                return Err(fail(format!("{n} rolls of this die never all differ")));
            }
            Ok(format!(
                "{ABOUT} {} {ROLLS_EACH}",
                (Q::int(n as i128) / works).decimal(0)
            ))
        };
        let blocks_y = [-1.0, -2.3];
        let mut lines_of = Vec::new();
        for (i, n) in [trio.len(), trio.len() + 1].into_iter().enumerate() {
            let mut parts = factors(n);
            parts.push(orders(n));
            let mut row = words_in_a_line(&kit, stage, &parts, 26.0, Palette::ink(), LEFT_EDGE, blocks_y[i])?;
            let result = row.pop().ok_or_else(|| fail("an empty line of orders".into()))?;
            let price = kit.text_left(stage, &cost(n)?, 24.0, Palette::dim(), v(LEFT_EDGE, blocks_y[i] - 0.45))?;
            lines_of.push((row, result, price));
        }
        // three rolls: what has just been shown, then four
        fade_in_lagged(stage, &lines_of[0].0, 0.8, 0.2)?;
        fade_in_all(stage, &[lines_of[0].1], 0.2)?;
        line.wait_for(stage, "Four different rolls")?;
        fade_in_lagged(stage, &lines_of[1].0, 1.2, 0.3)?;
        line.wait_for(stage, "twenty-four orders")?;
        fade_in_all(stage, &[lines_of[1].1], 0.2)?;
        line.wait_for(stage, "But rolls that all differ")?;
        fade_in_lagged(stage, &[lines_of[0].2, lines_of[1].2], 1.0, 0.5)?;
        let swells: Vec<Box<dyn Animation>> = vec![
            stage.prepare(swell(lines_of[0].2, 1.12, 0.8)?)?,
            stage.prepare(swell(lines_of[1].2, 1.12, 0.8)?)?,
        ];
        let swells = AnimationGroup::with_lag_ratio(stage.arena_mut(), swells, 0.5)?;
        stage.play(swells)?;
        kit.music_breath(stage, explainer_kit::animatic::HOLD_S)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}
