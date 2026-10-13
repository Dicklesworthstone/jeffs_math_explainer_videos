//! Chapter 06, same counts: it opens on the resting square; the two deciding
//! pairs hold the same results in a different order (06_1); a third toss
//! grows the square into a cube of eight pieces where it stands (06_2);
//! three of them are the patterns with two heads and a tail (06_3), and they
//! are one slab turned three ways (06_4); so patterns sort by their counts:
//! the square's four blocks fall into three bins, 1, 2, 1 (06_5), the cube's
//! eight pieces into 1, 3, 3, 1, and four tosses into 1, 4, 6, 4, 1, Pascal's
//! triangle (06_6); the coin's unfairness only decides which group a run of
//! tosses lands in (06_7). It ends on the row for four tosses, which chapter
//! 07 opens on.

use std::rc::Rc;

use super::draw::predict_pause;
use explainer_kit::props::glow;
use explainer_kit::{Kit, Palette, Timed, hex, play, tween, v};
use fmn::animation::{indicate, transform_from_copy};
use fmn::prelude::*;

use super::draw::{
    Block, BoxSpec, CUBE_SIDE, Cube, CubePose, CubeRoom, FOUR_ROW_BAR_FOOT, FOUR_ROW_BAR_W, FourRow, ROW_BOX_W,
    RowBars, RulerEdge, SQUARE_AT, SQUARE_SIDE, Square, SquarePose, SquareView, alice, bob, cube_ruler, cut_boxes,
    disc_row, fade_all_but, fade_in_all, fade_in_lagged, fade_out_all, fail, flat, gold, group, group_box, heads,
    heads_count, line_style, pair_tile, panel, rgb, row_x, say, tag, tails, thrown,
};
use crate::model::{Face, Q, bent, pascal_row, pattern_chance};

pub struct SameCounts {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const HEADS: &str = "heads";
const TAILS: &str = "tails";
const HEADS_THEN_TAILS: &str = "heads then tails";
const TAILS_THEN_HEADS: &str = "tails then heads";
const A_PATTERN: &str = "a pattern";
const SAME_EVERY_TIME: &str = "for tosses with the same chance every time";
const NO_MEMORY: &str = "and no memory";
const COUNTS: [&str; 3] = ["two heads", "one head and one tail", "two tails"];
const SAME_COUNTS: &str = "same counts, same chance";
const TOSSES: &str = "tosses";
const PASCAL: &str = "Pascal's triangle";
const CHANCE_OF_HEADS: &str = "chance of heads";
const COUNTS_CARRY: &str = "The counts carry the unfairness.";
const ORDER_IS_FAIR: &str = "The order is perfectly fair.";

/// The cube's side while its slabs stand beside it (06_4): its front corner
/// stays at home, and the slabs need room.
const ROOMY_SIDE: f64 = 2.5;
/// How far a slab first steps straight out of the cube, in sides.
const STEP_OUT: f64 = 0.25;
/// Where the three slabs stand once out: their centres on screen.
const SLOTS: [f64; 3] = [-0.2, 2.45, 5.1];
const SLOT_Y: f64 = 0.1;
/// How deep a slab's centre sits there, in sides.
const SLOT_DEPTH: f64 = 0.4;
/// The cube's side once its pieces are a row of the triangle, and how far
/// below that row it waits while the square's blocks are sorted (06_5).
const SMALL_SIDE: f64 = 0.72;
const PARK_DROP: f64 = 3.9;

/// One row of the triangle above the row for four tosses: its boxes' centre
/// height and height, and where its counts and its group sizes sit.
struct Row {
    y: f64,
    h: f64,
    count_y: f64,
    size_y: f64,
}

const ROW2: Row = Row {
    y: 2.48,
    h: 0.8,
    count_y: 3.06,
    size_y: 1.84,
};
const ROW3: Row = Row {
    y: 0.86,
    h: 0.85,
    count_y: 1.43,
    size_y: 0.16,
};
/// The square's side once its blocks stand in the row for two tosses, and
/// the gap between the green and the violet block there.
const MINI: f64 = 0.85;
const MINI_GAP: f64 = 0.12;
/// The bins of 06_5, at the square's own size: the padding round their
/// blocks, the gap between the green and violet block, between bins, and
/// the bins' centre height (the square's).
const BIN_PAD: f64 = 0.2;
const BIN_INNER: f64 = 0.25;
const BIN_GAP: f64 = 0.25;
const BIN_Y: f64 = 0.6;
/// The slider's track (the chances 0 and 1) and its height.
const TRACK: (f64, f64) = (-1.6, 1.6);
const SLIDER_Y: f64 = 3.45;

fn over(run_time: f64) -> AnimateArgs {
    AnimateArgs {
        run_time: Some(run_time),
        ..AnimateArgs::default()
    }
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

fn ground() -> Srgb {
    hex("#0E1116")
}

/// Keep the square's four edge words (under the bottom edge's heads and
/// tails stretches, then beside the left edge's) at the middle of their
/// stretches as the square changes size. Returns the anchor that carries the
/// updater: it must be on stage.
fn follow_words(stage: &mut Stage<'_>, view: SquareView, words: [Mob; 4]) -> fmn::Result<Mob> {
    let anchor = stage.arena_mut().add(VMobject::new());
    let mut last: Option<SquarePose> = None;
    stage.add_updater(
        anchor,
        move |s, _| {
            let pose = view.pose(s);
            if last == Some(pose) {
                return;
            }
            let (below, beside) = (pose.bottom - 0.38, pose.left - 0.55);
            let spots = [
                v(pose.point(pose.x / 2.0, 0.0)[0], below),
                v(pose.point((1.0 + pose.x) / 2.0, 0.0)[0], below),
                v(beside, pose.point(0.0, pose.y_h / 2.0)[1]),
                v(beside, pose.point(0.0, (1.0 + pose.y_h) / 2.0)[1]),
            ];
            for (&m, at) in words.iter().zip(spots) {
                s.move_to(m, at, ORIGIN);
            }
            last = Some(pose);
        },
        true,
    )?;
    Ok(anchor)
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

/// One piece of the cube: its place in `Cube::boxes`, its box, and the
/// pattern of three tosses it stands for.
#[derive(Clone, Copy)]
struct Piece {
    index: usize,
    spec: BoxSpec,
    pattern: [Face; 3],
}

impl Piece {
    fn tails(&self) -> usize {
        self.pattern.iter().filter(|&&f| f == Face::T).count()
    }

    /// The toss that differs from the other two (a slab's thin side, a
    /// rod's long one), if one does.
    fn odd(&self) -> Option<usize> {
        let lone = match self.tails() {
            1 => Face::T,
            2 => Face::H,
            _ => return None,
        };
        self.pattern.iter().position(|&f| f == lone)
    }

    /// The middle of a slab's outer face, `out` sides further out. The
    /// tails end of the first two tosses is the far end of its side; of the
    /// third it is the near end, so that all three slabs can be seen.
    fn outer(&self, out: f64) -> [f64; 3] {
        let mut c = self.spec.center();
        if let Some(a) = self.odd() {
            c[a] += outward(a) * (self.spec.size[a] / 2.0 + out);
        }
        c
    }
}

/// Which way is out of the cube for a slab that is thin along `axis`.
fn outward(axis: usize) -> f64 {
    if axis == 2 { -1.0 } else { 1.0 }
}

/// The pattern a box of the cube stands for, from its place along each side.
fn pattern_of(place: [usize; 3]) -> [Face; 3] {
    let face = |heads: bool| if heads { Face::H } else { Face::T };
    [face(place[0] == 0), face(place[1] == 0), face(place[2] == 1)]
}

/// A pattern as a row of discs centred at `at`, one group.
fn strip(stage: &mut Stage<'_>, pattern: &[Face], at: Vec3, radius: f64, step: f64) -> fmn::Result<Mob> {
    let left = v(at[0] - step * (pattern.len() as f64 - 1.0) / 2.0, at[1]);
    let discs = disc_row(stage, pattern, left, radius, step);
    group(stage, &discs)
}

/// A group's box in a row of the triangle, behind what stands in it.
fn row_box(stage: &mut Stage<'_>, at: Vec3, height: f64) -> fmn::Result<Mob> {
    let m = group_box(stage, at, height)?;
    stage.set_z_index(m, -1, true);
    Ok(m)
}

/// A box of `w` by `h` like a group's, for a bin of 06_5. Detached.
fn bin_box(stage: &mut Stage<'_>, at: Vec3, w: f64, h: f64) -> fmn::Result<Mob> {
    let back = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.12)
        .style(Style::default().fill(panel(), 0.94).stroke(Palette::faint(), 1.5, 1.0))
        .build()?
        .moved_to(at);
    let m = stage.arena_mut().add(back);
    stage.set_z_index(m, -1, true);
    Ok(m)
}

/// A chance as TeX: `\frac{4}{5}`.
fn frac(q: Q) -> String {
    match q.exact().split_once('/') {
        Some((n, d)) => format!(r"\frac{{{n}}}{{{d}}}"),
        None => q.exact(),
    }
}

/// A live percentage ("80%"), rebuilt every frame from `read` (the value in
/// percent and where its `edge` sits), rounded to a whole percent.
fn live_percent(
    kit: &Kit,
    stage: &mut Stage<'_>,
    size: f64,
    color: Srgb,
    edge: Vec3,
    read: impl Fn(&fmn::mobject::Stage) -> (f64, Vec3) + 'static,
) -> fmn::Result<Mob> {
    let book = Rc::clone(&kit.book);
    let m = stage.always_redraw(move |s| {
        let (value, at) = read(s);
        // with no decimals the number is cut, not rounded: round it first
        let d = DecimalNumber::new((value + 1e-6).round())
            .num_decimal_places(0)
            .unit("%")
            .font_size(size)
            .color(color)
            .build(&book)
            .expect("bundled digits always lay out");
        let m = s.add(d.into_vmob());
        s.move_to(m, at, edge);
        m
    });
    stage.add_to_scene(m)?;
    Ok(m)
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

/// The four blocks of a square of side `side` laid out as groups by counts:
/// each block's centre, given each group's centre (two heads, one of each,
/// two tails) and the gap between the green and the violet block. In
/// `Block` order: HH, HT, TH, TT.
fn grouped(side: f64, p: f64, centres: [Vec3; 3], gap: f64) -> [Vec3; 4] {
    let q = 1.0 - p;
    let total = side + gap;
    let mid = centres[1];
    [
        centres[0],
        v(mid[0] - total / 2.0 + p * side / 2.0, mid[1]),
        v(mid[0] + total / 2.0 - q * side / 2.0, mid[1]),
        centres[2],
    ]
}

impl SceneConstruct for SameCounts {
    fn name(&self) -> &str {
        "06_same_counts"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let coin = bent();
        let p = coin.f64();
        let q = 1.0 - p;
        let chance = |f: Face| if f == Face::H { coin } else { coin.other() };

        // the resting square, as chapter 05 leaves it, on stage from the
        // first frame so the picture stays through the cut
        let square = Square::build(stage, SQUARE_AT, SQUARE_SIDE, p)?;
        square.paint(stage, Block::HT, alice(), 1.0);
        square.paint(stage, Block::TH, bob(), 1.0);
        square.paint(stage, Block::HH, thrown(), 1.0);
        square.paint(stage, Block::TT, thrown(), 1.0);
        let edges = square.edges(&kit, stage, HEADS, TAILS)?;
        let mut resting = square.mobs();
        resting.extend(edges.mobs());
        for &m in &resting {
            stage.add_to_scene(m)?;
        }
        let words_follow = follow_words(stage, square.view, edges.words)?;
        stage.add_to_scene(words_follow)?;

        // 06_1: the two deciding pairs hold the same results
        let line = say(&kit, stage, "06_1")?;
        let pose = square.pose(stage);
        let (small, big) = (0.4, 1.4);
        let mut tiles = Vec::new();
        for (b, pair) in [(Block::HT, (Face::H, Face::T)), (Block::TH, (Face::T, Face::H))] {
            let parts = pair_tile(&kit, stage, pair, pose.center(b), small)?;
            tiles.push(group(stage, &parts)?);
        }
        fade_in_all(stage, &tiles, 0.5)?;
        // each pair lifts out of its block
        let lifted = [v(2.6, 1.35), v(2.6, -0.95)];
        let words = [
            (HEADS_THEN_TAILS, alice(), 0.95, "Heads then tails"),
            (TAILS_THEN_HEADS, bob(), -0.95, "tails then heads hold"),
        ];
        let mut lift_words = Vec::new();
        for (i, (text, color, dy, phrase)) in words.into_iter().enumerate() {
            line.wait_for(stage, phrase)?;
            let word = kit.text(stage, text, 28.0, color, v(lifted[i][0], lifted[i][1] + dy))?;
            play!(stage;
                tiles[i].animate().set_anim_args(over(0.9))?.scale(big / small)?.move_to(lifted[i], ORIGIN)?,
                fade_in(stage.arena_mut(), word, ORIGIN, 1.0)?.rt(0.9),
            );
            lift_words.push(word);
        }
        // a gold link from each disc to its twin in the other pair
        let (gap, r) = (0.32 * big, 0.26 * big);
        let mut links = Vec::new();
        for (phrase, side) in [("one head", -1.0), ("and one tail", 1.0)] {
            line.wait_for(stage, phrase)?;
            let from = v(lifted[0][0] + side * gap, lifted[0][1] - r);
            let to = v(lifted[1][0] - side * gap, lifted[1][1] + r);
            let link = Line::new(from, to).style(line_style(gold(), 5.0)).build()?;
            let link = stage.arena_mut().add(link);
            play!(stage; show_creation(link).rt(0.5));
            links.push(link);
        }

        // 06_2: three tosses need a cube; the square grows into one where it
        // stands, its blocks running back as the third toss's side grows
        let line = say(&kit, stage, "06_2")?;
        let mut clear = tiles.clone();
        clear.extend(links.iter().chain(&lift_words));
        fade_out_all(stage, &clear, 0.4)?;
        let (wide, thin) = ([0.0, p, 1.0], [0.0, q, 1.0]);
        let boxes = cut_boxes([&wide, &wide, &thin]);
        // each piece in its column's colour: the square's block, run back
        let column_color = |place: [usize; 3]| match (place[0], place[1]) {
            (0, 1) => alice(),
            (1, 0) => bob(),
            _ => thrown(),
        };
        let specs: Vec<(BoxSpec, Srgb)> = boxes.iter().map(|&(spec, place)| (spec, column_color(place))).collect();
        let mut cube = Cube::build(stage, SQUARE_AT, SQUARE_SIDE, 0.0, &specs)?;
        let mut pieces = Vec::new();
        for &(spec, place) in &boxes {
            let index = cube
                .find(spec)
                .ok_or_else(|| fail("a piece of the cube is missing".into()))?;
            let piece = Piece {
                index,
                spec,
                pattern: pattern_of(place),
            };
            // a slab turns its thin side to the third direction, a rod
            // stands its long side up: a quarter turn each
            if let Some(a) = piece.odd() {
                let to = if piece.tails() == 1 { 2 } else { 1 };
                let mut turned = [0, 1, 2];
                turned.swap(a, to);
                cube.set_turn(index, turned);
            }
            pieces.push(piece);
        }
        cube.install(stage)?;
        let room = CubeRoom::build(stage, cube.view, [&wide, &wide, &thin])?;
        let none = || RulerEdge {
            stretches: Vec::new(),
            words: Vec::new(),
            word_color: None,
            word_size: 24.0,
            word_out: 0.55,
            name: None,
        };
        let third = RulerEdge {
            stretches: vec![(0.0, q, tails()), (q, 1.0, heads())],
            words: vec![Some(TAILS.into()), Some(HEADS.into())],
            ..none()
        };
        let ruler = cube_ruler(&kit, stage, cube.view, [none(), none(), third], 0.0)?;
        // the three slabs in the order they will be named: the tail last, in
        // the middle, first
        let mut slabs: Vec<Piece> = pieces.iter().copied().filter(|x| x.tails() == 1).collect();
        slabs.sort_by_key(|x| std::cmp::Reverse(x.odd()));
        line.wait_for(stage, "so three need a cube")?;
        // with no depth the cube is exactly the square: it takes the
        // square's place, and the square's edges stay with its front face
        let mut grown = room.mobs();
        grown.extend(cube.mobs());
        grown.push(ruler.anchor);
        grown.extend(&ruler.stretches[2]);
        stage.scene_mut().add(&grown)?;
        let mut grow = tweens(
            stage,
            &[
                (cube.view.side, CUBE_SIDE),
                (cube.view.depth, 1.0),
                (square.view.side, CUBE_SIDE),
            ],
            1.5,
        )?;
        grow.extend(fades_in(stage, &ruler.words[2], 1.5)?);
        stage.play_prepared(grow)?;

        // 06_3: three patterns, three slabs; the colours go as the first lights
        let line = say(&kit, stage, "06_3")?;
        let first_pose = cube.view.pose(stage);
        let named = ["Two heads then a tail", "Heads, tails, heads", "A tail then two heads"];
        let mut strips = Vec::new();
        let mut word = Vec::new();
        for (k, (slab, phrase)) in slabs.iter().zip(named).enumerate() {
            line.wait_for(stage, phrase)?;
            let at = first_pose.project(slab.outer(0.0));
            let discs = strip(stage, &slab.pattern, at, 0.13, 0.29)?;
            stage.set_z_index(discs, 2, true);
            let mut ins = tweens(stage, &[(cube.boxes[slab.index].light, 1.0)], 0.6)?;
            ins.extend(fades_in(stage, &[discs], 0.6)?);
            if k == 0 {
                ins.extend(tweens(stage, &[(cube.tint, 0.0)], 0.6)?);
                let w = tag(&kit, stage, A_PATTERN, v(at[0], at[1] - 0.5), 22.0)?;
                stage.set_z_index(w, 2, true);
                ins.extend(fades_in(stage, &[w], 0.6)?);
                word.push(w);
            }
            stage.play_prepared(ins)?;
            strips.push(discs);
        }
        line.wait_for(stage, "Which of these three")?;
        let mut ask: Vec<Box<dyn Animation>> = Vec::new();
        for &discs in &strips {
            ask.push(stage.prepare(swell(discs, 1.25, 0.9)?)?);
        }
        stage.play_prepared(ask)?;
        let q_mark = predict_pause(&kit, stage, v(3.8, 0.1))?;

        // 06_4: one slab, turned three ways
        let line = say(&kit, stage, "06_4")?;
        word.push(q_mark);
        fade_out_all(stage, &word, 0.3)?;
        line.wait_for(stage, "Each is four fifths")?;
        // the cube makes room where it stands, and each slab takes a first
        // step straight out along its own thin side; from there no slab
        // stands behind another piece, so it may be drawn last
        let big_pose = CubePose {
            side: ROOMY_SIDE,
            ..first_pose
        };
        let mut step = tweens(
            stage,
            &[(cube.view.side, ROOMY_SIDE), (square.view.side, ROOMY_SIDE)],
            0.8,
        )?;
        for (slab, &discs) in slabs.iter().zip(&strips) {
            let Some(a) = slab.odd() else { continue };
            let out = outward(a) * STEP_OUT;
            step.extend(tweens(stage, &[(cube.boxes[slab.index].shift[a], out)], 0.8)?);
            let to = big_pose.project(slab.outer(STEP_OUT));
            step.push(stage.prepare(discs.animate().set_anim_args(over(0.8))?.move_to(to, ORIGIN)?)?);
        }
        stage.play_prepared(step)?;
        let mut front_faces = Vec::new();
        for slab in slabs.iter().rev() {
            for m in cube.faces(slab.index) {
                stage.set_z_index(m, 1, false);
                front_faces.push(m);
            }
        }
        stage.scene_mut().bring_to_front(&front_faces)?;
        // then to their places side by side, each still lying its own way:
        // wide by tall by deep is its pattern's three chances in order
        let mut fly: Vec<Box<dyn Animation>> = Vec::new();
        for (k, (slab, &discs)) in slabs.iter().zip(&strips).enumerate() {
            let to = v(SLOTS[k], SLOT_Y);
            let shift = shift_to(&big_pose, slab.spec.center(), to, SLOT_DEPTH);
            let moves: Vec<(Mob, f64)> = cube.boxes[slab.index].shift.into_iter().zip(shift).collect();
            fly.extend(tweens(stage, &moves, 1.3)?);
            fly.push(move_scaled(stage, discs, to, 1.5, 1.3)?);
        }
        stage.play_prepared(fly)?;
        let (head_tex, tail_tex) = (frac(coin), frac(coin.other()));
        let t2c = [(head_tex.as_str(), heads()), (tail_tex.as_str(), tails())];
        let mut products = Vec::new();
        for (k, slab) in slabs.iter().enumerate() {
            let factors: Vec<String> = slab.pattern.iter().map(|&f| frac(chance(f))).collect();
            let src = factors.join(r" \times ");
            products.push(kit.tex(stage, &src, 34.0, Palette::ink(), &t2c, v(SLOTS[k], -1.95))?);
        }
        fade_in_lagged(stage, &products, 1.2, 0.3)?;
        line.wait_for(stage, "The same slab")?;
        let turns: Vec<(Mob, f64)> = slabs.iter().map(|s| (cube.boxes[s.index].turn, 1.0)).collect();
        let turn = tweens(stage, &turns, 1.6)?;
        stage.play_prepared(turn)?;
        let each = format!("{}%", pattern_chance(coin, 3, 2).percent(1));
        let mut percents = Vec::new();
        for &x in &SLOTS {
            percents.push(kit.text(stage, &each, 34.0, Palette::ink(), v(x, -2.85))?);
        }
        fade_in_all(stage, &percents, 0.5)?;
        // the fence: what kind of tosses this is true of
        let fence_y = 2.35;
        let fence_a = kit.text(stage, SAME_EVERY_TIME, 24.0, Palette::dim(), v(0.0, fence_y))?;
        let fence_b = kit.text(stage, NO_MEMORY, 24.0, Palette::dim(), v(0.0, fence_y))?;
        let (wa, wb) = (stage.get_width(fence_a), stage.get_width(fence_b));
        let fence_left = SLOTS[1] - (wa + 0.14 + wb) / 2.0;
        stage.move_to(fence_a, v(fence_left, fence_y), LEFT);
        stage.move_to(fence_b, v(fence_left + wa + 0.14, fence_y), LEFT);
        line.wait_for(stage, "with the same chance")?;
        fade_in_all(stage, &[fence_a], 0.5)?;
        line.wait_for(stage, "and no memory")?;
        fade_in_all(stage, &[fence_b], 0.5)?;
        line.wait_for(stage, "order never changes")?;
        let mut equals = Vec::new();
        for pair in SLOTS.windows(2) {
            let at = v((pair[0] + pair[1]) / 2.0, -2.85);
            equals.push(kit.text_styled(stage, "=", 44.0, gold(), at, true, false)?);
        }
        let mut same = fades_in(stage, &equals, 0.6)?;
        for &m in &percents {
            let a = indicate(stage.arena_mut(), m, 1.2, Some(rgb(gold())))?;
            same.push(stage.prepare(a.rt(0.9))?);
        }
        stage.play_prepared(same)?;

        // 06_5: sort every pattern by its counts; two tosses first, at the
        // square's own size
        let line = say(&kit, stage, "06_5")?;
        let mut gone = products.clone();
        gone.extend(percents.iter().chain(&equals).chain(&strips));
        gone.extend([fence_a, fence_b]);
        gone.extend(room.mobs());
        gone.extend(ruler.mobs());
        fade_out_all(stage, &gone, 0.4)?;
        // the cube and its slabs step aside, small, below; the square they
        // stood on is back at its own size
        let ratio = SMALL_SIDE / ROOMY_SIDE;
        let small_pose = CubePose {
            left: row_x(3, 1) - (SLOTS[1] - SQUARE_AT[0]) * ratio,
            bottom: ROW3.y - (SLOT_Y - SQUARE_AT[1]) * ratio,
            side: SMALL_SIDE,
            depth: 1.0,
        };
        let aside = tweens(
            stage,
            &[
                (cube.view.left, small_pose.left),
                (cube.view.bottom, small_pose.bottom - PARK_DROP),
                (cube.view.side, SMALL_SIDE),
                (square.view.side, SQUARE_SIDE),
            ],
            1.0,
        )?;
        stage.play_prepared(aside)?;
        // the four blocks slide apart into three bins by their counts
        let s = SQUARE_SIDE;
        let bin_h = p * s + 2.0 * BIN_PAD;
        let bin_w = [
            p * s + 2.0 * BIN_PAD,
            s + BIN_INNER + 2.0 * BIN_PAD,
            q * s + 2.0 * BIN_PAD,
        ];
        let total = bin_w.iter().sum::<f64>() + 2.0 * BIN_GAP;
        let mut bin_x = [0.0; 3];
        let mut x = -total / 2.0;
        for k in 0..3 {
            bin_x[k] = x + bin_w[k] / 2.0;
            x += bin_w[k] + BIN_GAP;
        }
        let bin_centres = bin_x.map(|x| v(x, BIN_Y));
        let block_at = grouped(s, p, bin_centres, BIN_INNER);
        let blocks = [Block::HH, Block::HT, Block::TH, Block::TT].map(|b| square.block(b));
        let mut bins = Vec::new();
        let mut bin_words = Vec::new();
        for k in 0..3 {
            bins.push(bin_box(stage, bin_centres[k], bin_w[k], bin_h)?);
            let at = v(bin_x[k], BIN_Y + bin_h / 2.0 + 0.3);
            bin_words.push(kit.text(stage, COUNTS[k], 24.0, Palette::dim(), at)?);
        }
        line.wait_for(stage, "how many heads")?;
        let mut sort = fades_in(stage, &bins, 0.8)?;
        sort.extend(fades_in(stage, &bin_words, 0.8)?);
        let mut square_rest = vec![square.frame];
        square_rest.extend(square.cuts);
        square_rest.extend(edges.mobs());
        square_rest.push(words_follow);
        sort.extend(fades_out(stage, &square_rest, 0.5)?);
        for (&m, &to) in blocks.iter().zip(&block_at) {
            sort.push(stage.prepare(m.animate().set_anim_args(over(1.0))?.move_to(to, ORIGIN)?)?);
        }
        stage.play_prepared(sort)?;
        line.wait_for(stage, "Same counts, same chance")?;
        let title_y = BIN_Y + bin_h / 2.0 + 0.85;
        let title = kit.text(stage, SAME_COUNTS, 32.0, gold(), v(0.0, title_y))?;
        fade_in_all(stage, &[title], 0.6)?;
        let two_sizes: Vec<Mob> = pascal_row(2)
            .iter()
            .rev()
            .zip(bin_x)
            .map(|(n, x)| {
                kit.text(
                    stage,
                    &n.to_string(),
                    40.0,
                    Palette::ink(),
                    v(x, BIN_Y - bin_h / 2.0 - 0.38),
                )
            })
            .collect::<fmn::Result<_>>()?;
        let two_name = kit.text(stage, &format!("2 {TOSSES}"), 22.0, Palette::dim(), v(0.0, 0.0))?;
        stage.move_to(two_name, v(bin_x[0] - 0.45, BIN_Y - bin_h / 2.0 - 0.38), RIGHT);
        line.wait_for(stage, "one, two and one")?;
        let mut ins = vec![two_name];
        ins.extend(&two_sizes);
        fade_in_lagged(stage, &ins, 1.2, 0.4)?;
        stage.scene_mut().bring_to_front(&ins)?;
        line.wait_for(stage, "and the middle group")?;
        play!(stage;
            bins[1].animate().set_anim_args(over(0.6))?.set_stroke(Some(gold()), Some(3.0), None)?,
            swell(blocks[1], 1.08, 0.9)?,
            swell(blocks[2], 1.08, 0.9)?,
        );

        // 06_6: the bins shrink into the top row as the cube's eight pieces
        // sort into the second; four tosses arrive sorted; Pascal's triangle
        let line = say(&kit, stage, "06_6")?;
        let run = 0.9;
        let mut moves: Vec<Box<dyn Animation>> = Vec::new();
        let row2_centres = [0, 1, 2].map(|k| v(row_x(2, k), ROW2.y));
        let mini_at = grouped(MINI, p, row2_centres, MINI_GAP);
        let shrink = MINI / s;
        for k in 0..3 {
            let target = bin_box(stage, row2_centres[k], ROW_BOX_W, ROW2.h)?;
            moves.push(stage.prepare(Transform::new(bins[k], target).rt(run))?);
            moves.push(move_scaled(
                stage,
                bin_words[k],
                v(row_x(2, k), ROW2.count_y),
                20.0 / 24.0,
                run,
            )?);
            moves.push(move_scaled(stage, two_sizes[k], v(row_x(2, k), ROW2.size_y), 1.0, run)?);
        }
        for (&m, &to) in blocks.iter().zip(&mini_at) {
            moves.push(move_scaled(stage, m, to, shrink, run)?);
        }
        let name_w = stage.get_width(two_name);
        let name_at = v(row_x(2, 0) - 0.45 - name_w / 2.0, ROW2.size_y);
        moves.push(move_scaled(stage, two_name, name_at, 1.0, run)?);
        // the row for three tosses: its boxes and counts, and the pieces
        let mut three_boxes = Vec::new();
        let mut three_counts = Vec::new();
        for k in 0..=3 {
            three_boxes.push(row_box(stage, v(row_x(3, k), ROW3.y), ROW3.h)?);
            let count = heads_count(3 - k);
            three_counts.push(kit.text(stage, &count, 20.0, Palette::dim(), v(row_x(3, k), ROW3.count_y))?);
        }
        moves.extend(tweens(
            stage,
            &[(cube.view.bottom, small_pose.bottom), (cube.rim, 0.0)],
            run,
        )?);
        let mut raised = Vec::new();
        let mut rods = 0;
        for piece in &pieces {
            let (shift, turn) = (cube.boxes[piece.index].shift, cube.boxes[piece.index].turn);
            let centre = piece.spec.center();
            moves.extend(tweens(stage, &[(cube.boxes[piece.index].light, 1.0)], run)?);
            let x = match piece.tails() {
                // the slabs are in their group already
                1 => continue,
                2 => {
                    rods += 1;
                    moves.extend(tweens(stage, &[(turn, 1.0)], run)?);
                    row_x(3, 2) + 0.5 * (f64::from(rods) - 2.0)
                }
                k => row_x(3, k),
            };
            let to = shift_to(&small_pose, centre, v(x, ROW3.y), centre[2]);
            let shifts: Vec<(Mob, f64)> = shift.into_iter().zip(to).collect();
            moves.extend(tweens(stage, &shifts, run)?);
            if piece.tails() >= 2 {
                raised.extend(cube.faces(piece.index));
            }
        }
        // the rods and the small corner cross in front of the slabs
        for &m in &raised {
            stage.set_z_index(m, 2, false);
        }
        stage.scene_mut().bring_to_front(&raised)?;
        stage.play_prepared(moves)?;
        // the row's boxes and counts once the bins have left its place
        let mut row3 = three_boxes.clone();
        row3.extend(&three_counts);
        fade_in_all(stage, &row3, 0.35)?;
        let three_sizes: Vec<Mob> = pascal_row(3)
            .iter()
            .rev()
            .enumerate()
            .map(|(k, n)| kit.text(stage, &n.to_string(), 40.0, Palette::ink(), v(row_x(3, k), ROW3.size_y)))
            .collect::<fmn::Result<_>>()?;
        let three_name = kit.text(stage, &format!("3 {TOSSES}"), 22.0, Palette::dim(), v(0.0, 0.0))?;
        stage.move_to(three_name, v(row_x(3, 0) - 0.45, ROW3.size_y), RIGHT);
        line.wait_for(stage, "one, three, three, one")?;
        let mut ins = vec![three_name];
        ins.extend(&three_sizes);
        fade_in_lagged(stage, &ins, 1.0, 0.4)?;
        stage.scene_mut().bring_to_front(&ins)?;
        // four tosses have no picture: their patterns arrive already sorted
        let row4 = FourRow::build(&kit, stage)?;
        let four_sizes = row4.sizes;
        let mut arriving = vec![row4.name];
        for k in 0..row4.boxes.len() {
            arriving.push(row4.boxes[k]);
            arriving.extend(&row4.tiles[k]);
            arriving.extend([row4.counts[k], row4.sizes[k]]);
        }
        line.wait_for(stage, "Four tosses give")?;
        fade_in_lagged(stage, &arriving, 2.2, 0.12)?;
        // a lagged reveal leaves its members in one group: the row's parts
        // stand on their own again, so the row can stay through the cut
        stage.scene_mut().bring_to_front(&row4.mobs())?;
        line.wait_for(stage, "That's Pascal's")?;
        let mut all_sizes = two_sizes.clone();
        all_sizes.extend(three_sizes.iter().chain(&four_sizes));
        let mut pulse: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &all_sizes {
            let a = indicate(stage.arena_mut(), m, 1.25, Some(rgb(gold())))?;
            pulse.push(stage.prepare(a)?);
        }
        let pulse = AnimationGroup::with_lag_ratio(stage.arena_mut(), pulse, 0.12)?.with_run_time(0.9);
        stage.play(pulse)?;
        stage.scene_mut().bring_to_front(&all_sizes)?;
        // each number is the two above it added: the two threes come down
        // beside the six
        let six = four_sizes[2];
        let six_at = stage.get_center(six);
        let parts = ["=", "3", "+", "3"];
        let mut sum = Vec::new();
        let mut x = six_at[0] + 0.5;
        for text in parts {
            let m = kit.text(stage, text, 40.0, gold(), v(0.0, six_at[1]))?;
            let w = stage.get_width(m);
            stage.move_to(m, v(x, six_at[1]), LEFT);
            x += w + 0.14;
            sum.push(m);
        }
        let mut add: Vec<Box<dyn Animation>> = Vec::new();
        for (from, to) in [(three_sizes[1], sum[1]), (three_sizes[2], sum[3])] {
            let a = transform_from_copy(stage.arena_mut(), from, to)?;
            add.push(stage.prepare(a.rt(0.8))?);
        }
        add.extend(fades_in(stage, &[sum[0], sum[2]], 0.8)?);
        add.push(stage.prepare(six.animate().set_anim_args(over(0.8))?.set_color(gold())?)?);
        stage.play_prepared(add)?;
        let pascal = kit.text(stage, PASCAL, 30.0, gold(), v(0.0, -3.35))?;
        let ring = Circle::new()
            .radius(0.34)
            .style(line_style(gold(), 2.5))
            .build()
            .moved_to(six_at);
        let halo = stage.arena_mut().add(glow(&ring, gold(), 10, 16.0));
        let ring = stage.arena_mut().add(ring);
        fade_in_all(stage, &[pascal, halo, ring], 0.5)?;
        kit.listen(stage, 2.0, "06_same_counts:06_6")?;

        // 06_7: the unfairness only decides the group
        let line = say(&kit, stage, "06_7")?;
        let mut gone = vec![title, pascal, halo, ring, two_name, three_name];
        gone.extend(&sum);
        gone.extend(bins.iter().chain(&bin_words).chain(&blocks).chain(&two_sizes));
        gone.extend(three_boxes.iter().chain(&three_counts).chain(&three_sizes));
        gone.extend(cube.mobs());
        gone.push(square.anchor);
        let mut clear = fades_out(stage, &gone, 0.5)?;
        clear.push(stage.prepare(six.animate().set_anim_args(over(0.5))?.set_color(Palette::ink())?)?);
        stage.play_prepared(clear)?;
        // over each group a bar for the chance of landing in it, on a slider
        let fair = Q::HALF.f64();
        let slider = stage.add_value_tracker(fair);
        let bars = row4.bars(&kit, stage, slider)?;
        let mut ins = bars.mobs();
        let track = Line::new(v(TRACK.0, SLIDER_Y), v(TRACK.1, SLIDER_Y))
            .style(line_style(Palette::faint(), 4.0))
            .build()?;
        ins.push(stage.arena_mut().add(track));
        let knob_x = |c: f64| TRACK.0 + (TRACK.1 - TRACK.0) * c;
        let knob = Dot::new()
            .point(v(knob_x(fair), SLIDER_Y))
            .radius(0.12)
            .style(flat(Palette::ink(), 1.0))
            .build();
        let knob = stage.arena_mut().add(knob);
        stage.add_updater(
            knob,
            move |s, me| {
                let c = s.tracker_value(slider).unwrap_or(0.0);
                s.move_to(me, v(knob_x(c), SLIDER_Y), ORIGIN);
            },
            true,
        )?;
        ins.push(knob);
        let slider_name = kit.text(stage, CHANCE_OF_HEADS, 24.0, Palette::dim(), v(0.0, SLIDER_Y))?;
        stage.move_to(slider_name, v(TRACK.0 - 0.3, SLIDER_Y), RIGHT);
        ins.push(slider_name);
        fade_in_all(stage, &ins, 0.5)?;
        for &label in &bars.labels {
            stage.add_to_scene(label)?;
        }
        live_percent(&kit, stage, 26.0, Palette::ink(), LEFT, move |s| {
            let c = s.tracker_value(slider).unwrap_or(0.0);
            (100.0 * c, v(TRACK.1 + 0.3, SLIDER_Y))
        })?;
        line.wait_for(stage, "only decides")?;
        play!(stage; tween(slider, p, 2.2)?);
        // inside a group every pattern is as likely as the next: the middle
        // bar splits into one equal slice per pattern, each labelled
        line.wait_for(stage, "Inside a group")?;
        let middle = 2;
        let (foot, top) = (FOUR_ROW_BAR_FOOT, RowBars::top(middle, p));
        let shown = row4.tiles[middle].len();
        let each = format!("{}%", pattern_chance(coin, 4, 2).percent(2));
        let half = FOUR_ROW_BAR_W / 2.0;
        let mut slices = Vec::new();
        for i in 1..shown {
            let y = foot + (top - foot) * i as f64 / shown as f64;
            let cut = Line::new(v(-half, y), v(half, y))
                .style(line_style(ground(), 3.0))
                .build()?;
            slices.push(stage.arena_mut().add(cut));
        }
        // the labels fan out beside the bar, each on a leader to its slice
        let label_pitch = 0.31;
        let label_x = half + 0.55;
        let mid_y = (foot + top) / 2.0;
        for i in 0..shown {
            let y = foot + (top - foot) * (i as f64 + 0.5) / shown as f64;
            let ly = mid_y + label_pitch * (i as f64 - (shown as f64 - 1.0) / 2.0);
            let leader = Line::new(v(half + 0.04, y), v(label_x - 0.06, ly))
                .style(line_style(Palette::dim(), 1.5))
                .build()?;
            slices.push(stage.arena_mut().add(leader));
            slices.push(kit.text_left(stage, &each, 20.0, Palette::ink(), v(label_x, ly))?);
        }
        let mut split = fades_in(stage, &slices, 0.6)?;
        for &t in &row4.tiles[middle] {
            split.push(stage.prepare(swell(t, 1.12, 0.9)?)?);
        }
        stage.play_prepared(split)?;
        // the act's last two sentences, as a card
        let card_y = -3.38;
        let card_a = kit.text(stage, COUNTS_CARRY, 28.0, Palette::ink(), v(0.0, card_y))?;
        let card_b = kit.text(stage, ORDER_IS_FAIR, 28.0, gold(), v(0.0, card_y))?;
        let (wa, wb) = (stage.get_width(card_a), stage.get_width(card_b));
        let left = -(wa + 0.3 + wb) / 2.0;
        stage.move_to(card_a, v(left, card_y), LEFT);
        stage.move_to(card_b, v(left + wa + 0.3, card_y), LEFT);
        let back = Rectangle::new()
            .width(wa + wb + 0.9)
            .height(0.62)
            .corner_radius(0.12)
            .style(Style::default().fill(panel(), 0.94).stroke(Palette::faint(), 1.5, 1.0))
            .build()?
            .moved_to(v(0.0, card_y));
        let back = stage.arena_mut().add(back);
        line.wait_for(stage, "The counts carry")?;
        fade_in_all(stage, &[back, card_a], 0.5)?;
        line.wait_for(stage, "The order is")?;
        fade_in_all(stage, &[card_b], 0.5)?;
        // the bars, the slider and the card go; the row stays through the
        // cut, and chapter 07 opens on it
        kit.hold(stage)?;
        stage.scene_mut().bring_to_front(&row4.mobs())?;
        fade_all_but(stage, &row4.mobs(), 0.6)?;
        kit.music_breath(stage, explainer_kit::animatic::HOLD_S)?;
        kit.finish(stage)
    }
}
