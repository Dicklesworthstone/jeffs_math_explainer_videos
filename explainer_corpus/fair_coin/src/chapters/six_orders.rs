//! Chapter 09, six orders: three rolls need the cube again, cut by the
//! loaded die's chances along every edge (09_1); the six orders of a four, a
//! five and a six are six bricks in it (09_2), and they are one brick turned
//! six ways (09_3); so the order three different rolls came in is a fair
//! choice among six (09_4), six thousand seeded rolls say so (09_5), and it
//! keeps going at a rising price (09_6).

use explainer_kit::props::{lower_third, predict_pause};
use explainer_kit::{Kit, Palette, Timed, fade_all, hex, play, thousands, tween, v};
use fmn::prelude::*;

use super::draw::{
    BoxSpec, Cube, CubePose, CubeView, SQUARE_AT, SQUARE_SIDE, alice, bob, column, cut_boxes, die_face, fade_in_all,
    fade_in_lagged, fade_out_all, faded_line, fail, flat, gold, line_style, panel, say, thrown, thrown_lit,
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
const A_MATCH: &str = "two rolls match:";
const ROLL_FRESH: &str = "roll three fresh ones";
const ROLLS: &str = "rolls";
const FAIR_ONES: &str = "fair ones";
const FAIR_DIE: &str = "a fair die:";
const MODEL: &str = "(model)";
const ORDERS: &str = "orders";
const WITH_THIS_DIE: &str = "with this die:";
const THREE_DIFFER: &str = "three rolls all differ";
const FOUR_DIFFER: &str = "four all differ";
const IN_100: &str = "times in 100, about";
const FOR_EACH: &str = "rolls for each fair one";

/// Where the cube stands (its front face's lower-left corner) and its side.
const CUBE_AT: Vec3 = [-5.2, -2.1, 0.0];
const CUBE_SIDE: f64 = 3.0;
/// Where the bricks stand once out of the cube, three to a row: the first
/// column's centre on screen and from one column to the next, the two rows'
/// heights, and how deep the bricks' centres sit (in sides).
const OUT_X: (f64, f64) = (0.55, 2.3);
const OUT_Y: [f64; 2] = [1.35, -0.95];
const OUT_DEPTH: f64 = 0.3;
/// The six columns of the fair die: the first one's centre and from one to
/// the next; where a die face, its order card and the foot of its bar sit;
/// how much room the bars have; the cube's side when its bricks stand there.
const COLUMN: (f64, f64) = (-0.9, 1.35);
const DIE_Y: f64 = -1.1;
const DIE_SIZE: f64 = 0.7;
const CARD_Y: f64 = -2.1;
const CARD_H: f64 = 1.0;
const BAR_FOOT: f64 = -0.55;
const BAR_ROOM: f64 = 3.1;
const SMALL_SIDE: f64 = 1.5;
/// The middle of the left-hand part, beside the columns.
const LEFT_MID: f64 = -4.5;

/// Where a brick's label sits while the brick is in the cube: an offset in
/// sides from the brick's centre on screen, by the brick's place in the row
/// (the six bricks stand in a ring, so each label goes outwards).
const LABEL_OFF: [(f64, f64); 6] = [
    (0.42, 0.02),
    (0.38, 0.1),
    (0.02, 0.29),
    (0.335, -0.475),
    (-0.06, 0.27),
    (-0.68, -0.02),
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

/// A brick's fill: one neutral, lit; its edges are gold.
fn brick_fill() -> Srgb {
    hex("#8191AE")
}

/// The cube's three back faces: the back, the left side, the floor.
fn wall(which: usize) -> Srgb {
    hex(["#1B2332", "#161D29", "#222C3D"][which])
}

fn mix(a: Srgb, b: Srgb, t: f64) -> Srgb {
    Srgb {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
    }
}

fn column_x(j: usize) -> f64 {
    COLUMN.0 + COLUMN.1 * j as f64
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

// ------------------------------------------------------------ solid faces

/// What a solid face's opacity is at full presence: full to the eye, and a
/// value the cube never writes.
const SOLID: f64 = 0.999;
/// The opacity of a face that is placed but not yet shown: nothing to the
/// eye, and not the 0 the cube gives a face that points away.
const UNSEEN: f64 = 0.002;

fn fill_alpha(s: &fmn::mobject::Stage, m: Mob) -> f64 {
    s.get(m)
        .and_then(|e| e.buffer.read_column("fill_rgba"))
        .and_then(|c| c.get(3).copied())
        .map_or(0.0, f64::from)
}

/// Solid faces for a cube, and a presence for each of its boxes. `draw.rs`
/// shades a face through its opacity, so a face shows whatever is behind it;
/// here every face the cube has just placed takes its shade as a colour
/// instead (`fill` mixed towards the background) at full opacity, with an
/// edge of `edge` (colour and width). Returns one tracker per box, in the
/// cube's order: 0 hides the box, 1 shows it. Call it right after
/// `Cube::install`, so that it runs after the cube's own updater.
fn solid_faces(stage: &mut Stage<'_>, cube: &Cube, fill: Srgb, edge: (Srgb, f64)) -> fmn::Result<Vec<Mob>> {
    let presence: Vec<Mob> = cube.boxes.iter().map(|_| stage.add_value_tracker(0.0)).collect();
    let boxes = cube.boxes.clone();
    let trackers = presence.clone();
    let mut shade = vec![[0.0_f64; 6]; boxes.len()];
    let mut wrote = vec![[f64::NAN; 6]; boxes.len()];
    let mut shown = vec![f64::NAN; boxes.len()];
    stage.add_updater(
        cube.anchor,
        move |s, _| {
            for (k, b) in boxes.iter().enumerate() {
                let here = s.tracker_value(trackers[k]).unwrap_or(0.0).clamp(0.0, 1.0);
                let changed = shown[k] != here;
                for (f, &m) in b.faces.iter().enumerate() {
                    let alpha = fill_alpha(s, m);
                    let placed = wrote[k][f].is_nan() || (alpha - wrote[k][f]).abs() > 1e-4;
                    if placed {
                        shade[k][f] = alpha;
                    }
                    if !(placed || changed) {
                        continue;
                    }
                    // a face that points away keeps the cube's own nothing
                    let (a, line) = if shade[k][f] > 0.0 {
                        ((SOLID * here).max(UNSEEN), 0.9 * here)
                    } else {
                        (0.0, 0.0)
                    };
                    s.set_fill(m, Some(mix(ground(), fill, shade[k][f])), Some(a), None, false);
                    s.set_stroke(m, Some(edge.0), Some(edge.1), Some(line), None, false);
                    wrote[k][f] = a;
                }
                shown[k] = here;
            }
        },
        true,
    )?;
    Ok(presence)
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

// ---------------------------------------------------------------- the room

/// The cube as a room seen from its open front: the square of two rolls is
/// its back face (thirty-six tiles), and the left side and the floor come
/// forward from it, each with its cuts; the three edges nearest the viewer
/// are drawn faintly. All of it follows the cube's trackers.
struct Room {
    anchor: Mob,
    tiles: Vec<Mob>,
    sides: Vec<Mob>,
    cuts: Vec<Mob>,
    edges: Vec<Mob>,
    near: Vec<Mob>,
}

impl Room {
    /// `big` are the cuts along the first two rolls (the six's stretch
    /// first), `small` along the third (the six's stretch last, at the
    /// back); `colors` gives each tile's colour by its place.
    fn build(
        stage: &mut Stage<'_>,
        view: CubeView,
        big: &[f64],
        small: &[f64],
        colors: impl Fn(usize, usize) -> Srgb,
    ) -> fmn::Result<Self> {
        let n = big.len() - 1;
        let mut tile_quads = Vec::new();
        let mut tiles = Vec::new();
        for i in 0..n {
            for j in 0..n {
                let (x0, x1, y0, y1) = (big[i], big[i + 1], big[j], big[j + 1]);
                tile_quads.push([[x0, y0, 1.0], [x1, y0, 1.0], [x1, y1, 1.0], [x0, y1, 1.0]]);
                let tile = Polygon::new([v(0.0, 0.0), v(1.0, 0.0), v(1.0, 1.0), v(0.0, 1.0)])
                    .style(flat(colors(i, j), 1.0))
                    .build();
                tiles.push(stage.arena_mut().add(tile));
            }
        }
        let side_quads = [
            [[0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 1.0], [0.0, 1.0, 0.0]],
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 0.0, 1.0]],
        ];
        let sides: Vec<Mob> = (1..=2)
            .map(|which| {
                let side = Polygon::new([v(0.0, 0.0), v(1.0, 0.0), v(1.0, 1.0), v(0.0, 1.0)])
                    .style(flat(wall(which), 1.0))
                    .build();
                stage.arena_mut().add(side)
            })
            .collect();
        // the cuts on the back, on the left side and on the floor
        let mut cut_ends: Vec<[[f64; 3]; 2]> = Vec::new();
        for &c in &big[1..n] {
            cut_ends.push([[c, 0.0, 1.0], [c, 1.0, 1.0]]);
            cut_ends.push([[0.0, c, 1.0], [1.0, c, 1.0]]);
            cut_ends.push([[0.0, c, 0.0], [0.0, c, 1.0]]);
            cut_ends.push([[c, 0.0, 0.0], [c, 0.0, 1.0]]);
        }
        for &c in &small[1..n] {
            cut_ends.push([[0.0, 0.0, c], [0.0, 1.0, c]]);
            cut_ends.push([[0.0, 0.0, c], [1.0, 0.0, c]]);
        }
        // the nine edges of the three back faces, then the three nearest
        let corner = |k: usize| [0, 1, 2].map(|a| f64::from(u8::from(k >> a & 1 == 1)));
        let mut edge_ends: Vec<[[f64; 3]; 2]> = Vec::new();
        let mut near_ends: Vec<[[f64; 3]; 2]> = Vec::new();
        for k in 0..8usize {
            for a in 0..3 {
                if k >> a & 1 == 1 {
                    continue;
                }
                let ends = [corner(k), corner(k | 1 << a)];
                // an edge of the back, the left side or the floor has both
                // ends on that face
                let on_back = ends.iter().all(|p| p[2] == 1.0);
                let on_side = ends.iter().all(|p| p[0] == 0.0);
                let on_floor = ends.iter().all(|p| p[1] == 0.0);
                if on_back || on_side || on_floor {
                    edge_ends.push(ends);
                } else {
                    near_ends.push(ends);
                }
            }
        }
        let mut lines = |ends: &[[[f64; 3]; 2]], style: Style| -> Vec<Mob> {
            ends.iter()
                .map(|_| {
                    stage
                        .arena_mut()
                        .add(polyline(vec![v(0.0, 0.0), v(1.0, 0.0)]).style(style).build())
                })
                .collect()
        };
        let cuts = lines(&cut_ends, line_style(hex("#465269"), 1.5));
        let edges = lines(&edge_ends, faded_line(Palette::dim(), 2.0, 0.85));
        let near = lines(&near_ends, faded_line(Palette::dim(), 1.5, 0.35));
        let anchor = stage.arena_mut().add(VMobject::new());
        let all_lines: Vec<(Mob, [[f64; 3]; 2])> = cuts
            .iter()
            .chain(&edges)
            .chain(&near)
            .copied()
            .zip(cut_ends.into_iter().chain(edge_ends).chain(near_ends))
            .collect();
        let quads: Vec<(Mob, [[f64; 3]; 4])> = tiles
            .iter()
            .chain(&sides)
            .copied()
            .zip(tile_quads.into_iter().chain(side_quads))
            .collect();
        let mut last: Option<CubePose> = None;
        stage.add_updater(
            anchor,
            move |s, _| {
                let val = |m: Mob| s.tracker_value(m).unwrap_or(0.0);
                // never quite flat: a side with no width is not a shape
                let pose = CubePose {
                    left: val(view.left),
                    bottom: val(view.bottom),
                    side: val(view.side),
                    depth: val(view.depth).max(0.004),
                };
                if last == Some(pose) {
                    return;
                }
                for &(m, quad) in &quads {
                    let _ = s.set_points(m, Polygon::new(quad.map(|p| pose.project(p))).build().points());
                }
                for &(m, ends) in &all_lines {
                    let line = polyline(ends.map(|p| pose.project(p)).to_vec()).build();
                    let _ = s.set_points(m, line.points());
                }
                last = Some(pose);
            },
            true,
        )?;
        Ok(Self {
            anchor,
            tiles,
            sides,
            cuts,
            edges,
            near,
        })
    }

    /// Everything to bring into the scene, back to front.
    fn mobs(&self) -> Vec<Mob> {
        let mut out = vec![self.anchor];
        out.extend(self.tiles.iter().chain(&self.sides).chain(&self.cuts));
        out.extend(self.edges.iter().chain(&self.near));
        out
    }
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

/// Each roll's rank among the three: 0 the lowest.
fn ranks(rolls: &[usize]) -> Vec<usize> {
    rolls.iter().map(|r| rolls.iter().filter(|o| *o < r).count()).collect()
}

/// An order as a small card: three bars, low, middle and high, in the order
/// the rolls came. Detached parts, the back first.
fn order_card(stage: &mut Stage<'_>, order: &[usize], at: Vec3) -> fmn::Result<Vec<Mob>> {
    let back = Rectangle::new()
        .width(1.16)
        .height(CARD_H)
        .corner_radius(0.1)
        .style(Style::default().fill(panel(), 0.94).stroke(Palette::faint(), 1.5, 1.0))
        .build()?
        .moved_to(at);
    let mut parts = vec![stage.arena_mut().add(back)];
    for (i, rank) in ranks(order).into_iter().enumerate() {
        let foot = v(at[0] + 0.3 * (i as f64 - 1.0), at[1] - 0.08);
        let bar = column(foot, 0.17, 0.12 + 0.15 * rank as f64, Palette::dim());
        parts.push(stage.arena_mut().add(bar));
    }
    Ok(parts)
}

/// Three rolls as dice with, when they all differ, a small chart that ranks
/// them: a bar and a word under each. Detached.
fn example(kit: &Kit, stage: &mut Stage<'_>, rolls: &[usize], top: f64) -> fmn::Result<Vec<Mob>> {
    let differ = order_of_rolls(rolls).is_some();
    let back = if differ { Palette::ink() } else { thrown_lit() };
    let mut out = Vec::new();
    for (i, (&roll, rank)) in rolls.iter().zip(ranks(rolls)).enumerate() {
        let x = LEFT_MID + 1.3 * (i as f64 - 1.0);
        out.push(die_face(stage, roll, v(x, top), 0.9, back, ground())?);
        if differ {
            let bar = column(v(x, top - 2.2), 0.5, 0.4 + 0.45 * rank as f64, Palette::dim());
            out.push(stage.arena_mut().add(bar));
            out.push(kit.text(stage, RANKS[rank], 24.0, Palette::ink(), v(x, top - 2.5))?);
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

        // 09_1: the six-way square tips back into a cube
        let line = say(&kit, stage, "09_1")?;
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
            }
        }
        let specs: Vec<(BoxSpec, Srgb)> = bricks.iter().map(|b| (b.spec, brick_fill())).collect();
        let mut cube = Cube::build(stage, SQUARE_AT, SQUARE_SIDE, 0.0, &specs)?;
        for b in &mut bricks {
            b.index = cube
                .find(b.spec)
                .ok_or_else(|| fail("a brick of the cube is missing".into()))?;
            // out of the cube every brick stands the same way: the five's
            // side across, the six's up, the four's back
            let side_of = |face: usize| b.order.iter().position(|&r| r == face).unwrap_or(0);
            cube.set_turn(b.index, [side_of(trio[1]), side_of(trio[2]), side_of(trio[0])]);
        }
        bricks.sort_by_key(|b| b.slot);
        cube.install(stage)?;
        let presence = solid_faces(stage, &cube, brick_fill(), (gold(), 2.5))?;
        let room = Room::build(stage, cube.view, &big, &small, |i, j| match i.cmp(&j) {
            std::cmp::Ordering::Less => alice(),
            std::cmp::Ordering::Greater => bob(),
            std::cmp::Ordering::Equal => thrown(),
        })?;
        fade_in_all(stage, &room.mobs(), 0.7)?;
        stage.scene_mut().add(&cube.mobs())?;
        line.wait_for(stage, "and three need the cube")?;
        let mut plain: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &room.tiles {
            let a = m.animate().set_anim_args(over(0.5))?.set_fill(Some(wall(0)), None)?;
            plain.push(stage.prepare(a)?);
        }
        stage.play_prepared(plain)?;
        let tip = [
            (cube.view.left, CUBE_AT[0]),
            (cube.view.bottom, CUBE_AT[1]),
            (cube.view.side, CUBE_SIDE),
            (cube.view.depth, 1.0),
        ];
        let tip = tweens(stage, &tip, 1.6)?;
        stage.play_prepared(tip)?;
        let pose = CubePose {
            left: CUBE_AT[0],
            bottom: CUBE_AT[1],
            side: CUBE_SIDE,
            depth: 1.0,
        };
        // each edge takes the three stretches the chapter's rolls live in,
        // and then its roll's name
        line.wait_for(stage, "cut by the die's chances")?;
        let stretch = |cuts: &[f64], face: usize, big_first: bool| {
            let k = if big_first { faces - face } else { face - 1 };
            (cuts[k] + cuts[k + 1]) / 2.0
        };
        let mut edge_marks = Vec::new();
        for &face in &trio {
            let text = face.to_string();
            let x = pose.project([stretch(&big, face, true), 0.0, 0.0]);
            let y = pose.project([0.0, stretch(&big, face, true), 0.0]);
            let z = pose.project([1.0, 0.0, stretch(&small, face, false)]);
            for at in [v(x[0], x[1] - 0.24), v(y[0] - 0.22, y[1]), v(z[0] + 0.2, z[1] - 0.14)] {
                edge_marks.push(kit.text(stage, &text, 22.0, Palette::dim(), at)?);
            }
        }
        fade_in_lagged(stage, &edge_marks, 1.0, 0.1)?;
        line.wait_for(stage, "along every edge")?;
        let corner = pose.project([0.0, 0.0, 0.0]);
        let far = pose.project([1.0, 0.0, 0.5]);
        let name_at = [
            v(corner[0] + CUBE_SIDE / 2.0, corner[1] - 0.6),
            v(corner[0] - 0.95, corner[1] + CUBE_SIDE + 0.3),
            v(far[0] + 1.25, far[1] - 0.45),
        ];
        let mut names = Vec::new();
        for (name, at) in ROLL_NAMES.into_iter().zip(name_at) {
            names.push(kit.text(stage, name, 22.0, Palette::dim(), at)?);
        }
        fade_in_all(stage, &names, 0.6)?;
        edge_marks.extend(names);

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
        line.wait_for(stage, "Each order lands")?;
        let mut appear = Vec::new();
        for b in &bricks {
            appear.extend(tweens(stage, &[(presence[b.index], 1.0)], 1.0)?);
        }
        let appear = AnimationGroup::with_lag_ratio(stage.arena_mut(), appear, 0.3)?.with_run_time(1.5);
        stage.play(appear)?;
        // the cube turns a little, and back, so the bricks' places can be read
        let turn = cube
            .view
            .depth
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(2.0),
                rate_func: Some(fmn::core::rate::there_and_back),
                ..AnimateArgs::default()
            })?
            .set_value(0.7)?;
        stage.play(turn)?;
        line.wait_for(stage, "How do the bricks compare")?;
        let mut labels = Vec::new();
        for b in &bricks {
            let c = pose.project(b.spec.center());
            let (dx, dy) = LABEL_OFF[b.slot];
            let text = b.order.map(|r| r.to_string()).join(" ");
            let at = v(c[0] + CUBE_SIDE * dx, c[1] + CUBE_SIDE * dy);
            let label = kit.text(stage, &text, 24.0, Palette::ink(), at)?;
            stage.set_z_index(label, 2, true);
            labels.push(label);
        }
        fade_in_lagged(stage, &labels, 1.0, 0.2)?;
        // a lagged reveal leaves its members in one group: the labels go
        // back to the top of the picture on their own
        stage.scene_mut().bring_to_front(&labels)?;
        let q = predict_pause(&kit, stage, v(3.6, -0.6))?;

        // 09_3: one brick, turned six ways
        let line = say(&kit, stage, "09_3")?;
        let mut gone = edge_marks.clone();
        gone.extend(&rolls_shown);
        gone.push(q);
        fade_out_all(stage, &gone, 0.3)?;
        // out of the cube, each still lying its own way
        let mut out: Vec<Box<dyn Animation>> = Vec::new();
        for (b, &label) in bricks.iter().zip(&labels) {
            let to = out_at(b.slot);
            let shift = shift_to(&pose, b.spec.center(), to, OUT_DEPTH);
            let moves: Vec<(Mob, f64)> = cube.boxes[b.index].shift.into_iter().zip(shift).collect();
            out.extend(tweens(stage, &moves, 1.3)?);
            let under = v(to[0], to[1] - 1.1);
            out.push(stage.prepare(label.animate().set_anim_args(over(1.3))?.move_to(under, ORIGIN)?)?);
        }
        stage.play_prepared(out)?;
        // then each turns where it stands, until all six stand alike
        line.wait_for(stage, "turned six ways")?;
        let turns: Vec<(Mob, f64)> = bricks.iter().map(|b| (cube.boxes[b.index].turn, 1.0)).collect();
        let turns = tweens(stage, &turns, 1.5)?;
        stage.play_prepared(turns)?;
        let out_mid = OUT_X.0 + OUT_X.1;
        line.wait_for(stage, "A tenth")?;
        let sides = trio.map(|face| chances[face - 1]);
        let volume = sides.iter().fold(Q::ONE, |acc, &x| acc * x);
        let product = sides.map(frac).join(r" \times ");
        let product = format!("{product} = {}", frac(volume));
        let product = kit.tex(stage, &product, 34.0, Palette::ink(), &[], v(out_mid, -3.05))?;
        fade_in_all(stage, &[product], 0.6)?;
        line.wait_for(stage, "The same three rolls")?;
        let same = kit.text(stage, SAME_ROLLS, 30.0, gold(), v(out_mid, 3.1))?;
        fade_in_all(stage, &[same], 0.6)?;
        line.wait_for(stage, "in any order")?;
        let mut wave: Vec<Box<dyn Animation>> = Vec::new();
        for &label in &labels {
            wave.push(stage.prepare(swell(label, 1.3, 0.6)?)?);
        }
        let wave = AnimationGroup::with_lag_ratio(stage.arena_mut(), wave, 0.25)?.with_run_time(1.0);
        stage.play(wave)?;
        stage.scene_mut().bring_to_front(&labels)?;
        kit.music_breath(stage, explainer_kit::animatic::HOLD_S)?;

        // 09_4: the order three different rolls came in is a fair die
        let line = say(&kit, stage, "09_4")?;
        let mut gone = room.mobs();
        gone.extend([product, same]);
        fade_out_all(stage, &gone, 0.4)?;
        // the bricks, half the size, stand where the die's faces will
        let small_pose = CubePose {
            side: SMALL_SIDE,
            ..pose
        };
        let mut settle = tweens(stage, &[(cube.view.side, SMALL_SIDE)], 1.0)?;
        for (b, &label) in bricks.iter().zip(&labels) {
            let to = v(column_x(b.slot), DIE_Y);
            let shift = shift_to(&small_pose, b.spec.center(), to, OUT_DEPTH);
            let moves: Vec<(Mob, f64)> = cube.boxes[b.index].shift.into_iter().zip(shift).collect();
            settle.extend(tweens(stage, &moves, 1.0)?);
            let under = v(column_x(b.slot), CARD_Y - 0.31);
            settle.push(stage.prepare(label.animate().set_anim_args(over(1.0))?.move_to(under, ORIGIN)?)?);
        }
        stage.play_prepared(settle)?;
        // each takes its order's card
        line.wait_for(stage, "the order they came in")?;
        let mut cards: Vec<Mob> = Vec::new();
        for b in &bricks {
            let parts = order_card(stage, &b.order, v(column_x(b.slot), CARD_Y))?;
            stage.set_z_index(parts[0], -1, true);
            cards.extend(parts);
        }
        fade_in_lagged(stage, &cards, 1.0, 0.08)?;
        // and becomes a face of a fair die
        line.wait_for(stage, "a fair choice among six")?;
        let mut turn_into: Vec<Box<dyn Animation>> = Vec::new();
        for b in &bricks {
            turn_into.extend(tweens(stage, &[(presence[b.index], 0.0)], 0.8)?);
            let at = v(column_x(b.slot), DIE_Y);
            let d = die_face(stage, b.slot + 1, at, DIE_SIZE, Palette::ink(), ground())?;
            turn_into.extend(fades_in(stage, &[d], 0.8)?);
        }
        stage.play_prepared(turn_into)?;
        // two orders, then a match, each shown as three dice
        let top = 1.6;
        let ring = |stage: &mut Stage<'_>, slot: usize| -> fmn::Result<Mob> {
            let (high, low) = (DIE_Y + DIE_SIZE / 2.0 + 0.08, CARD_Y - CARD_H / 2.0 - 0.08);
            let around = Rectangle::new()
                .width(1.3)
                .height(high - low)
                .corner_radius(0.14)
                .style(line_style(gold(), 3.0))
                .build()?
                .moved_to(v(column_x(slot), (high + low) / 2.0));
            Ok(stage.arena_mut().add(around))
        };
        // low, high, middle; then high, low, middle
        let spoken = [
            ([trio[0], trio[2], trio[1]], "Low, high, middle"),
            ([trio[2], trio[0], trio[1]], "High, low, middle"),
        ];
        let mut showing: Vec<Mob> = Vec::new();
        for (rolls, phrase) in spoken {
            line.wait_for(stage, phrase)?;
            let slot = order_of_rolls(&rolls).ok_or_else(|| fail("the example's rolls match".into()))?;
            let mut next = example(&kit, stage, &rolls, top)?;
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
        let mut next = example(&kit, stage, matching, top)?;
        next.push(kit.text(stage, A_MATCH, 26.0, Palette::dim(), v(LEFT_MID, top - 1.2))?);
        next.push(kit.text(stage, ROLL_FRESH, 26.0, Palette::dim(), v(LEFT_MID, top - 1.7))?);
        let mut swap = fades_out(stage, &showing, 0.4)?;
        swap.extend(fades_in(stage, &next, 0.5)?);
        stage.play_prepared(swap)?;
        let showing = next;

        // 09_5: six thousand rolls made six hundred fifty fair ones
        say(&kit, stage, "09_5")?;
        fade_out_all(stage, &showing, 0.4)?;
        let made = blocks(&run, trio.len(), order_of_rolls);
        let counts = tally(&made, bricks.len());
        let (low, high) = two_sigma_band(made.len() as u32, Q::new(1, bricks.len() as i128));
        let unit = BAR_ROOM / f64::from(high);
        let caption = format!("{} {ROLLS}, {} {FAIR_ONES}", thousands(run.len() as u64), made.len());
        let caption = kit.text(stage, &caption, 28.0, Palette::ink(), v(LEFT_MID, 0.6))?;
        let counters: Vec<Mob> = counts.iter().map(|_| stage.add_value_tracker(0.0)).collect();
        let bars: Vec<Mob> = (0..counts.len())
            .map(|j| {
                let bar = column(v(column_x(j), BAR_FOOT), 0.8, 0.02, Palette::dim());
                stage.arena_mut().add(bar)
            })
            .collect();
        let bar_anchor = stage.arena_mut().add(VMobject::new());
        {
            let (bars, counters) = (bars.clone(), counters.clone());
            let mut last: Option<Vec<f64>> = None;
            stage.add_updater(
                bar_anchor,
                move |s, _| {
                    let now: Vec<f64> = counters.iter().map(|&t| s.tracker_value(t).unwrap_or(0.0)).collect();
                    if last.as_ref() == Some(&now) {
                        return;
                    }
                    for (j, (&m, &count)) in bars.iter().zip(&now).enumerate() {
                        let bar = column(v(column_x(j), BAR_FOOT), 0.8, unit * count, Palette::dim());
                        let _ = s.set_points(m, bar.points());
                    }
                    last = Some(now);
                },
                true,
            )?;
        }
        let mut ins = vec![caption, bar_anchor];
        ins.extend(&bars);
        fade_in_all(stage, &ins, 0.4)?;
        // the counts stand in a row above the bars' room, clear of the band
        for (j, &t) in counters.iter().enumerate() {
            let at = v(column_x(j), BAR_FOOT + BAR_ROOM + 0.14);
            kit.live_number(stage, t, 0, false, 26.0, Palette::ink(), at, DOWN)?;
        }
        let rise: Vec<(Mob, f64)> = counters
            .iter()
            .zip(&counts)
            .map(|(&t, &count)| (t, count as f64))
            .collect();
        let rise = tweens(stage, &rise, 2.0)?;
        stage.play_prepared(rise)?;
        // where a fair die's six counts land about 19 times in 20
        let (band_left, band_right) = (column_x(0) - 0.6, column_x(counts.len() - 1) + 0.6);
        let (band_low, band_high) = (BAR_FOOT + unit * f64::from(low), BAR_FOOT + unit * f64::from(high));
        let strip = Rectangle::new()
            .width(band_right - band_left)
            .height(band_high - band_low)
            .style(flat(Palette::ink(), 0.13))
            .build()?
            .moved_to(v((band_left + band_right) / 2.0, (band_low + band_high) / 2.0));
        let mut band = vec![stage.arena_mut().add(strip)];
        for y in [band_low, band_high] {
            let rule = Line::new(v(band_left, y), v(band_right, y))
                .style(faded_line(Palette::ink(), 1.5, 0.55))
                .build()?;
            band.push(stage.arena_mut().add(rule));
        }
        let band_words = format!("{FAIR_DIE} {low} to {high} {MODEL}");
        let band_words = kit.text(stage, &band_words, 22.0, Palette::dim(), v(0.0, 0.0))?;
        stage.move_to(band_words, v(band_left - 0.2, (band_low + band_high) / 2.0), RIGHT);
        band.push(band_words);
        fade_in_all(stage, &band, 0.5)?;

        // 09_6: it keeps going, and each step costs more
        let line = say(&kit, stage, "09_6")?;
        let factorial = |n: usize| (1..=n).product::<usize>();
        let three = format!("{} {ROLLS}: {} {ORDERS}", trio.len(), factorial(trio.len()));
        let three = kit.text(stage, &three, 26.0, Palette::dim(), v(LEFT_MID, -0.4))?;
        fade_in_all(stage, &[three], 0.5)?;
        line.wait_for(stage, "Four different rolls")?;
        let more = trio.len() + 1;
        let four = format!("{more} {ROLLS}:");
        let four = kit.text(stage, &four, 26.0, Palette::ink(), v(LEFT_MID, -1.15))?;
        // the product, a factor at a time, then what it comes to
        let factors: Vec<String> = (1..=more).rev().map(|k| k.to_string()).collect();
        let mut pieces = vec![factors[0].clone()];
        pieces.extend(factors[1..].iter().map(|f| format!("× {f}")));
        let answer = format!("= {} {ORDERS}", factorial(more));
        let mut count_up = Vec::new();
        for text in pieces.iter().chain([&answer]) {
            count_up.push(kit.text(stage, text, 30.0, Palette::ink(), v(0.0, -1.75))?);
        }
        let gap = 0.16;
        let widths: Vec<f64> = count_up.iter().map(|&m| stage.get_width(m)).collect();
        let mut x = LEFT_MID - (widths.iter().sum::<f64>() + gap * (widths.len() as f64 - 1.0)) / 2.0;
        for (&m, w) in count_up.iter().zip(&widths) {
            stage.move_to(m, v(x, -1.75), LEFT);
            x += w + gap;
        }
        let mut product = vec![four];
        product.extend(&count_up[..pieces.len()]);
        fade_in_lagged(stage, &product, 1.6, 0.4)?;
        line.wait_for(stage, "twenty-four orders")?;
        fade_in_all(stage, &count_up[pieces.len()..], 0.5)?;
        line.wait_for(stage, "But rolls that all differ")?;
        let cost = |n: usize| -> fmn::Result<(String, String)> {
            let works = all_different(&chances, n);
            if works.is_zero() {
                return Err(fail(format!("{n} rolls of this die never all differ")));
            }
            Ok((works.percent(0), (Q::int(n as i128) / works).decimal(0)))
        };
        let (three_works, three_rolls) = cost(trio.len())?;
        let (four_works, four_rolls) = cost(more)?;
        let note = format!(
            "{THREE_DIFFER} {three_works} {IN_100} {three_rolls} {FOR_EACH}; {FOUR_DIFFER} {four_works} {IN_100} {four_rolls} {ROLLS} {MODEL}"
        );
        let note = lower_third(&kit, stage, WITH_THIS_DIE, &note)?;
        play!(stage; fade_in(stage.arena_mut(), note, ORIGIN, 1.0)?.rt(0.5));
        kit.music_breath(stage, explainer_kit::animatic::HOLD_S)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}
