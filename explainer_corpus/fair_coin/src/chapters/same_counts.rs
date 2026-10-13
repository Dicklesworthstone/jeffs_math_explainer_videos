//! Chapter 06, same counts: the two deciding pairs hold the same results in
//! a different order (06_1); a third toss turns the square into a cube of
//! eight pieces (06_2); three of them are the patterns with two heads and a
//! tail (06_3), and they are one slab turned three ways (06_4); so patterns
//! sort by their counts into groups of 1, 2, 1 (06_5), 1, 3, 3, 1 and
//! 1, 4, 6, 4, 1, Pascal's triangle (06_6); and the coin's unfairness only
//! decides which group a run of tosses lands in (06_7).

use std::rc::Rc;

use explainer_kit::props::{glow, predict_pause};
use explainer_kit::{Kit, Palette, Timed, fade_all, hex, play, tween, v};
use fmn::animation::indicate;
use fmn::prelude::*;

use super::draw::{
    BLOCKS, Block, BoxSpec, Cube, CubePose, SQUARE_AT, SQUARE_SIDE, Square, alice, bob, column, cut_boxes, disc_row,
    fade_in_all, fade_in_lagged, fade_out_all, fail, flat, gold, group, heads, line_style, pair_tile, panel, rgb, say,
    tag, tails, thrown,
};
use crate::model::{Face, Q, bent, pascal_row, pattern_chance, patterns};

pub struct SameCounts {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
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
const HEADS: &str = "heads";
const HEAD: &str = "head";
const COUNTS_CARRY: &str = "The counts carry the unfairness.";
const ORDER_IS_FAIR: &str = "The order is perfectly fair.";

/// Where the cube stands when it first appears and once its slabs come out
/// (its front face's lower-left corner), and its side.
const CUBE_FIRST: Vec3 = [-3.2, -1.6, 0.0];
const CUBE_AT: Vec3 = [-6.4, -1.6, 0.0];
const CUBE_SIDE: f64 = 2.5;
/// How far a slab first steps straight out of the cube, in sides.
const STEP_OUT: f64 = 0.25;
/// Where the three slabs stand once out: their centres on screen.
const SLOTS: [f64; 3] = [-0.95, 1.95, 4.85];
const SLOT_Y: f64 = 0.05;
/// How deep a slab's centre sits there, in sides.
const SLOT_DEPTH: f64 = 0.4;
/// The cube's side once its pieces are a row of the triangle.
const SMALL_SIDE: f64 = 0.72;

/// The triangle of groups: from one group's centre to the next, a group's
/// panel, and each row's height on screen with its panel's height (the rows
/// for two, three and four tosses).
const PITCH: f64 = 2.6;
const PANEL_W: f64 = 2.5;
const ROWS: [(f64, f64); 3] = [(2.2, 1.3), (0.3, 1.1), (-1.65, 1.2)];
/// The small square of the row for two tosses.
const MINI: f64 = 1.15;
/// The bars over the row for four tosses: where they stand, how wide, and
/// how tall a chance of one would be.
const BAR_FOOT: f64 = -0.55;
const BAR_W: f64 = 1.3;
const BAR_UNIT: f64 = 7.4;
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

/// A piece of the cube nobody has named yet, and one that has.
fn piece_grey() -> Srgb {
    hex("#3E4A60")
}

fn piece_lit() -> Srgb {
    hex("#8191AE")
}

fn mix(a: Srgb, b: Srgb, t: f64) -> Srgb {
    Srgb {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
    }
}

/// The x of group `k` (how many tails; all heads on the left) in the row
/// for `n` tosses.
fn group_x(n: usize, k: usize) -> f64 {
    PITCH * (k as f64 - n as f64 / 2.0)
}

/// Where a row's group sizes are written.
fn sizes_y(row: usize) -> f64 {
    ROWS[row].0 - ROWS[row].1 / 2.0 - 0.3
}

// ------------------------------------------------------------ solid faces

/// What a solid face's opacity is set to: full to the eye, and a value the
/// cube never writes, so a face the cube has just placed is told from one
/// already made solid.
const SOLID: f64 = 0.999;

fn fill_alpha(s: &fmn::mobject::Stage, m: Mob) -> f64 {
    s.get(m)
        .and_then(|e| e.buffer.read_column("fill_rgba"))
        .and_then(|c| c.get(3).copied())
        .map_or(0.0, f64::from)
}

/// The lights of a cube with solid faces: one tracker per box, in the cube's
/// order (0 is the plain fill, 1 the lit one), and one for the gold edge a
/// lit box wears (1 gold, 0 the plain dark edge).
struct Lights {
    each: Vec<Mob>,
    rim: Mob,
}

/// Solid faces for a cube, and a light for each of its boxes. `draw.rs`
/// shades a face through its opacity, so a face shows the boxes behind it;
/// here every face the cube has just placed takes its shade as a colour
/// instead (its fill mixed towards the background) at full opacity. Call it
/// right after `Cube::install`, so that it runs after the cube's own updater.
fn solid_faces(stage: &mut Stage<'_>, cube: &Cube, fill: Srgb, lit: Srgb) -> fmn::Result<Lights> {
    let each: Vec<Mob> = cube.boxes.iter().map(|_| stage.add_value_tracker(0.0)).collect();
    let rim = stage.add_value_tracker(1.0);
    let boxes = cube.boxes.clone();
    let trackers = each.clone();
    let mut shade = vec![[0.0_f64; 6]; boxes.len()];
    let mut shown = vec![(f64::NAN, f64::NAN); boxes.len()];
    stage.add_updater(
        cube.anchor,
        move |s, _| {
            let val = |s: &fmn::mobject::Stage, m: Mob| s.tracker_value(m).unwrap_or(0.0).clamp(0.0, 1.0);
            let rim = val(s, rim);
            for (k, b) in boxes.iter().enumerate() {
                let light = val(s, trackers[k]);
                let relit = shown[k] != (light, rim);
                let (paint, edge) = (mix(fill, lit, light), mix(ground(), gold(), light * rim));
                for (f, &m) in b.faces.iter().enumerate() {
                    let alpha = fill_alpha(s, m);
                    let placed = (alpha - SOLID).abs() > 1e-4;
                    if placed {
                        shade[k][f] = alpha;
                    }
                    if shade[k][f] <= 0.0 || !(placed || relit) {
                        continue;
                    }
                    s.set_fill(m, Some(mix(ground(), paint, shade[k][f])), Some(SOLID), None, false);
                    s.set_stroke(m, Some(edge), Some(1.6 + 1.4 * light * rim), None, None, false);
                }
                shown[k] = (light, rim);
            }
        },
        true,
    )?;
    Ok(Lights { each, rim })
}

/// The cube's pose from its trackers.
fn pose_of(stage: &Stage<'_>, cube: &Cube) -> CubePose {
    let val = |m: Mob| stage.tracker_value(m).unwrap_or(0.0);
    CubePose {
        left: val(cube.view.left),
        bottom: val(cube.view.bottom),
        side: val(cube.view.side),
        depth: val(cube.view.depth),
    }
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

/// A pattern of four tosses as a small tile: its discs on a back.
fn tile(stage: &mut Stage<'_>, pattern: &[Face], at: Vec3) -> fmn::Result<Mob> {
    let back = Rectangle::new()
        .width(1.04)
        .height(0.3)
        .corner_radius(0.07)
        .style(flat(hex("#2A3447"), 1.0))
        .build()?
        .moved_to(at);
    let mut parts = vec![stage.arena_mut().add(back)];
    let left = v(at[0] - 0.23 * 1.5, at[1]);
    parts.extend(disc_row(stage, pattern, left, 0.09, 0.23));
    group(stage, &parts)
}

/// A group's panel, drawn behind whatever stands in it.
fn group_panel(stage: &mut Stage<'_>, at: Vec3, height: f64) -> fmn::Result<Mob> {
    let back = Rectangle::new()
        .width(PANEL_W)
        .height(height)
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

/// A live percentage ("41%"), rebuilt every frame from `read` (the value in
/// percent and where its `edge` sits); one decimal only under 3%.
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
        // (a half read off a bar's height can sit a hair under itself)
        let (value, places) = if value < 2.95 {
            (value, 1)
        } else {
            ((value + 1e-6).round(), 0)
        };
        let d = DecimalNumber::new(value)
            .num_decimal_places(places)
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

impl SceneConstruct for SameCounts {
    fn name(&self) -> &str {
        "06_same_counts"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let coin = bent();
        let p = coin.f64();
        let chance = |f: Face| if f == Face::H { coin } else { coin.other() };

        // 06_1: the two deciding pairs hold the same results
        let line = say(&kit, stage, "06_1")?;
        let square = Square::build(stage, SQUARE_AT, SQUARE_SIDE, p)?;
        square.paint(stage, Block::HT, alice(), 1.0);
        square.paint(stage, Block::TH, bob(), 1.0);
        square.paint(stage, Block::HH, thrown(), 1.0);
        square.paint(stage, Block::TT, thrown(), 1.0);
        let pose = square.pose(stage);
        let (small, big) = (0.4, 1.4);
        let mut tiles = Vec::new();
        for (b, pair) in [(Block::HT, (Face::H, Face::T)), (Block::TH, (Face::T, Face::H))] {
            let parts = pair_tile(&kit, stage, pair, pose.center(b), small)?;
            tiles.push(group(stage, &parts)?);
        }
        let mut ins = square.mobs();
        ins.extend(&tiles);
        fade_in_all(stage, &ins, 0.7)?;
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

        // 06_2: three tosses need a cube; the square is its front
        let line = say(&kit, stage, "06_2")?;
        let (wide, thin) = ([0.0, p, 1.0], [0.0, 1.0 - p, 1.0]);
        let boxes = cut_boxes([&wide, &wide, &thin]);
        let specs: Vec<(BoxSpec, Srgb)> = boxes.iter().map(|&(spec, _)| (spec, piece_grey())).collect();
        let mut cube = Cube::build(stage, CUBE_FIRST, CUBE_SIDE, 0.0, &specs)?;
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
            // stands its long side up: a third of a turn about the corner
            if let Some(a) = piece.odd() {
                let turned = if piece.tails() == 1 {
                    [(a + 1) % 3, (a + 2) % 3, a]
                } else {
                    [(a + 2) % 3, a, (a + 1) % 3]
                };
                cube.set_turn(index, turned);
            }
            pieces.push(piece);
        }
        cube.install(stage)?;
        let front = mix(ground(), piece_grey(), fill_alpha(stage.arena(), cube.faces(0)[4]));
        let lights = solid_faces(stage, &cube, piece_grey(), piece_lit())?;
        // the three slabs in the order they will be named: the tail last, in
        // the middle, first
        let mut slabs: Vec<Piece> = pieces.iter().copied().filter(|x| x.tails() == 1).collect();
        slabs.sort_by_key(|x| std::cmp::Reverse(x.odd()));
        let mut clear: Vec<Box<dyn Animation>> = Vec::new();
        for &m in tiles.iter().chain(&links).chain(&lift_words) {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            clear.push(stage.prepare(a.rt(0.4))?);
        }
        for b in BLOCKS {
            clear.push(stage.prepare(square.repaint(b, front, 0.4)?)?);
        }
        stage.play_prepared(clear)?;
        let to_cube = [
            (square.view.left, CUBE_FIRST[0]),
            (square.view.bottom, CUBE_FIRST[1]),
            (square.view.side, CUBE_SIDE),
        ];
        let moves = tweens(stage, &to_cube, 0.9)?;
        stage.play_prepared(moves)?;
        line.wait_for(stage, "so three need a cube")?;
        stage.scene_mut().add(&cube.mobs())?;
        fade_out_all(stage, &square.mobs(), 0.25)?;
        play!(stage; tween(cube.view.depth, 1.0, 1.4)?);

        // 06_3: three patterns, three slabs
        let line = say(&kit, stage, "06_3")?;
        let first_pose = pose_of(stage, &cube);
        let named = ["Two heads then a tail", "Heads, tails, heads", "A tail then two heads"];
        let mut strips = Vec::new();
        let mut word = Vec::new();
        for (k, (slab, phrase)) in slabs.iter().zip(named).enumerate() {
            line.wait_for(stage, phrase)?;
            let at = first_pose.project(slab.outer(0.0));
            let discs = strip(stage, &slab.pattern, at, 0.13, 0.29)?;
            stage.set_z_index(discs, 2, true);
            let mut ins = tweens(stage, &[(lights.each[slab.index], 1.0)], 0.5)?;
            ins.extend(fades_in(stage, &[discs], 0.5)?);
            if k == 0 {
                let w = tag(&kit, stage, A_PATTERN, v(at[0], at[1] - 0.5), 22.0)?;
                stage.set_z_index(w, 2, true);
                ins.extend(fades_in(stage, &[w], 0.5)?);
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
        let q = predict_pause(&kit, stage, v(3.8, 0.1))?;

        // 06_4: one slab, turned three ways
        let line = say(&kit, stage, "06_4")?;
        word.push(q);
        fade_out_all(stage, &word, 0.3)?;
        line.wait_for(stage, "Each is four fifths")?;
        // the cube makes room, and each slab takes a first step straight out
        // along its own thin side; from there no slab stands behind another
        // piece, so it may be drawn last
        let big_pose = CubePose {
            left: CUBE_AT[0],
            bottom: CUBE_AT[1],
            ..first_pose
        };
        let mut step = tweens(
            stage,
            &[(cube.view.left, big_pose.left), (cube.view.bottom, big_pose.bottom)],
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
            fly.push(
                stage.prepare(
                    discs
                        .animate()
                        .set_anim_args(over(1.3))?
                        .scale(1.5)?
                        .move_to(to, ORIGIN)?,
                )?,
            );
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

        // 06_5: sort every pattern by its counts; two tosses first
        let line = say(&kit, stage, "06_5")?;
        let mut gone = products.clone();
        gone.extend(percents.iter().chain(&equals).chain(&strips));
        gone.extend([fence_a, fence_b]);
        fade_out_all(stage, &gone, 0.4)?;
        // the cube and its slabs shrink to where the row for three tosses
        // will be, the slabs already in their group; above them, the square
        let ratio = SMALL_SIDE / CUBE_SIDE;
        let small_pose = CubePose {
            left: group_x(3, 1) - (SLOTS[1] - CUBE_AT[0]) * ratio,
            bottom: ROWS[1].0 - (SLOT_Y - CUBE_AT[1]) * ratio,
            side: SMALL_SIDE,
            depth: 1.0,
        };
        let (mx, my) = (-MINI / 2.0, ROWS[0].0 - MINI / 2.0);
        let q = 1.0 - p;
        // each block of the square: its colour, its corner and size in the
        // square, its group, and its place in the group
        let blocks = [
            (thrown(), (0.0, 0.0), (p, p), 0, 0.0),
            (alice(), (0.0, p), (p, q), 1, -(MINI * q + 0.25) / 2.0),
            (bob(), (p, 0.0), (q, p), 1, (MINI * p + 0.25) / 2.0),
            (thrown(), (p, p), (q, q), 2, 0.0),
        ];
        let mut minis = Vec::new();
        for &(color, (u, w), (du, dw), ..) in &blocks {
            let at = v(mx + MINI * (u + du / 2.0), my + MINI * (w + dw / 2.0));
            let block = Rectangle::new()
                .width(MINI * du)
                .height(MINI * dw)
                .style(flat(color, 1.0))
                .build()?
                .moved_to(at);
            minis.push(stage.arena_mut().add(block));
        }
        let shrink = [
            (cube.view.left, small_pose.left),
            (cube.view.bottom, small_pose.bottom),
            (cube.view.side, small_pose.side),
        ];
        let mut arrive = tweens(stage, &shrink, 1.0)?;
        arrive.extend(fades_in(stage, &minis, 1.0)?);
        stage.play_prepared(arrive)?;
        line.wait_for(stage, "how many heads")?;
        let mut two_panels = Vec::new();
        let mut two_labels = Vec::new();
        for (k, words) in COUNTS.into_iter().enumerate() {
            let at = v(group_x(2, k), ROWS[0].0);
            two_panels.push(group_panel(stage, at, ROWS[0].1)?);
            let y = at[1] + ROWS[0].1 / 2.0 + 0.24;
            two_labels.push(kit.text(stage, words, 22.0, Palette::dim(), v(at[0], y))?);
        }
        let mut sort = fades_in(stage, &two_panels, 0.8)?;
        sort.extend(fades_in(stage, &two_labels, 0.8)?);
        for (&m, &(.., k, dx)) in minis.iter().zip(&blocks) {
            let to = v(group_x(2, k) + dx, ROWS[0].0);
            sort.push(stage.prepare(m.animate().set_anim_args(over(1.0))?.move_to(to, ORIGIN)?)?);
        }
        stage.play_prepared(sort)?;
        line.wait_for(stage, "Same counts, same chance")?;
        let title = kit.text(stage, SAME_COUNTS, 32.0, gold(), v(0.0, 3.58))?;
        fade_in_all(stage, &[title], 0.6)?;
        // group sizes, a row at a time
        let size_row = |stage: &mut Stage<'_>, row: usize| -> fmn::Result<(Vec<Mob>, Mob)> {
            let n = row + 2;
            let sizes = pascal_row(n as u32);
            let mut out = Vec::new();
            for k in 0..=n {
                let text = sizes[n - k].to_string();
                out.push(kit.text(stage, &text, 40.0, Palette::ink(), v(group_x(n, k), sizes_y(row)))?);
            }
            let name = format!("{n} {TOSSES}");
            let name = kit.text(stage, &name, 22.0, Palette::dim(), v(0.0, sizes_y(row)))?;
            stage.move_to(name, v(group_x(n, 0) - 0.45, sizes_y(row)), RIGHT);
            Ok((out, name))
        };
        let (two_sizes, two_name) = size_row(stage, 0)?;
        line.wait_for(stage, "one, two and one")?;
        let mut ins = vec![two_name];
        ins.extend(&two_sizes);
        fade_in_lagged(stage, &ins, 1.3, 0.4)?;
        line.wait_for(stage, "and the middle group")?;
        play!(stage;
            two_panels[1].animate().set_anim_args(over(0.6))?.set_stroke(Some(gold()), Some(3.0), None)?,
            swell(minis[1], 1.15, 0.9)?,
            swell(minis[2], 1.15, 0.9)?,
        );

        // 06_6: three tosses, four tosses: Pascal's triangle
        let line = say(&kit, stage, "06_6")?;
        let mut three_panels = Vec::new();
        for k in 0..=3 {
            three_panels.push(group_panel(stage, v(group_x(3, k), ROWS[1].0), ROWS[1].1)?);
        }
        let mut sort = fades_in(stage, &three_panels, 0.8)?;
        // every piece is lit now, and none wears the gold edge
        sort.extend(tweens(stage, &[(lights.rim, 0.0)], 1.0)?);
        let mut raised = Vec::new();
        let mut rods = 0;
        for piece in &pieces {
            let (shift, turn) = (cube.boxes[piece.index].shift, cube.boxes[piece.index].turn);
            let centre = piece.spec.center();
            sort.extend(tweens(stage, &[(lights.each[piece.index], 1.0)], 1.0)?);
            let x = match piece.tails() {
                // the slabs are in their group already
                1 => continue,
                2 => {
                    rods += 1;
                    sort.extend(tweens(stage, &[(turn, 1.0)], 1.2)?);
                    group_x(3, 2) + 0.5 * (f64::from(rods) - 2.0)
                }
                k => group_x(3, k),
            };
            let to = shift_to(&small_pose, centre, v(x, ROWS[1].0), centre[2]);
            let moves: Vec<(Mob, f64)> = shift.into_iter().zip(to).collect();
            sort.extend(tweens(stage, &moves, 1.2)?);
            if piece.tails() >= 2 {
                raised.extend(cube.faces(piece.index));
            }
        }
        // the rods and the small corner cross in front of the slabs
        for &m in &raised {
            stage.set_z_index(m, 2, false);
        }
        stage.scene_mut().bring_to_front(&raised)?;
        stage.play_prepared(sort)?;
        let (three_sizes, three_name) = size_row(stage, 1)?;
        line.wait_for(stage, "one, three, three, one")?;
        let mut ins = vec![three_name];
        ins.extend(&three_sizes);
        fade_in_lagged(stage, &ins, 1.3, 0.4)?;
        // four tosses have no picture: their patterns arrive already sorted
        let (four_sizes, four_name) = size_row(stage, 2)?;
        let mut four_panels = Vec::new();
        let mut four_tiles: Vec<Mob> = Vec::new();
        let mut arriving = vec![four_name];
        for (k, &size) in four_sizes.iter().enumerate() {
            let at = v(group_x(4, k), ROWS[2].0);
            let back = group_panel(stage, at, ROWS[2].1)?;
            four_panels.push(back);
            arriving.push(back);
            let group_patterns = patterns(4, 4 - k as u32);
            let cols = group_patterns.len().min(2);
            let rows = group_patterns.len().div_ceil(cols);
            for (i, pattern) in group_patterns.iter().enumerate() {
                let dx = 1.12 * ((i % cols) as f64 - (cols as f64 - 1.0) / 2.0);
                let dy = 0.36 * ((rows as f64 - 1.0) / 2.0 - (i / cols) as f64);
                let t = tile(stage, pattern, v(at[0] + dx, at[1] + dy))?;
                four_tiles.push(t);
                arriving.push(t);
            }
            arriving.push(size);
        }
        line.wait_for(stage, "Four tosses give")?;
        fade_in_lagged(stage, &arriving, 2.8, 0.12)?;
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
        let pascal = kit.text(stage, PASCAL, 30.0, gold(), v(0.0, -3.4))?;
        let six = four_sizes[2];
        let ring = Circle::new()
            .radius(0.34)
            .style(line_style(gold(), 2.5))
            .build()
            .moved_to(v(group_x(4, 2), sizes_y(2)));
        let halo = stage.arena_mut().add(glow(&ring, gold(), 10, 16.0));
        let ring = stage.arena_mut().add(ring);
        play!(stage;
            fade_in(stage.arena_mut(), pascal, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), halo, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), ring, ORIGIN, 1.0)?.rt(0.6),
            six.animate().set_anim_args(over(0.6))?.set_color(gold())?,
        );
        kit.listen(stage, 2.0, "06_same_counts:06_6")?;

        // 06_7: the unfairness only decides the group
        let line = say(&kit, stage, "06_7")?;
        let mut gone = vec![title, pascal, halo, ring, two_name, three_name];
        gone.extend(two_panels.iter().chain(&two_labels).chain(&minis));
        gone.extend(two_sizes.iter().chain(&three_sizes).chain(&three_panels));
        gone.extend(cube.mobs());
        fade_out_all(stage, &gone, 0.5)?;
        // over each group a bar for the chance of landing in it, on a slider
        let fair = Q::HALF.f64();
        let slider = stage.add_value_tracker(fair);
        let ways: Vec<f64> = pascal_row(4).iter().rev().map(|&w| w as f64).collect();
        let landing = move |c: f64, k: usize| ways[k] * c.powi(4 - k as i32) * (1.0 - c).powi(k as i32);
        let bars: Vec<Mob> = (0..=4)
            .map(|k| {
                let bar = column(v(group_x(4, k), BAR_FOOT), BAR_W, 0.1, Palette::dim());
                stage.arena_mut().add(bar)
            })
            .collect();
        let bar_anchor = stage.arena_mut().add(VMobject::new());
        {
            let (bars, landing) = (bars.clone(), landing.clone());
            let mut last = None;
            stage.add_updater(
                bar_anchor,
                move |s, _| {
                    let c = s.tracker_value(slider).unwrap_or(0.0);
                    if last == Some(c) {
                        return;
                    }
                    for (k, &m) in bars.iter().enumerate() {
                        let bar = column(
                            v(group_x(4, k), BAR_FOOT),
                            BAR_W,
                            BAR_UNIT * landing(c, k),
                            Palette::dim(),
                        );
                        let _ = s.set_points(m, bar.points());
                    }
                    last = Some(c);
                },
                true,
            )?;
        }
        let mut ins = vec![bar_anchor];
        ins.extend(&bars);
        for k in 0..=4usize {
            let n = 4 - k;
            let words = format!("{n} {}", if n == 1 { HEAD } else { HEADS });
            ins.push(kit.text(stage, &words, 20.0, Palette::dim(), v(group_x(4, k), -0.82))?);
        }
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
        for k in 0..=4 {
            let landing = landing.clone();
            live_percent(&kit, stage, 24.0, Palette::ink(), DOWN, move |s| {
                let share = landing(s.tracker_value(slider).unwrap_or(0.0), k);
                (100.0 * share, v(group_x(4, k), BAR_FOOT + BAR_UNIT * share + 0.1))
            })?;
        }
        live_percent(&kit, stage, 26.0, Palette::ink(), LEFT, move |s| {
            let c = s.tracker_value(slider).unwrap_or(0.0);
            (100.0 * c, v(TRACK.1 + 0.3, SLIDER_Y))
        })?;
        line.wait_for(stage, "only decides")?;
        play!(stage; tween(slider, p, 2.2)?);
        // inside a group nothing moved: every tile is as likely as its neighbours
        line.wait_for(stage, "Inside a group")?;
        let mut pulse: Vec<Box<dyn Animation>> = Vec::new();
        for &t in &four_tiles {
            pulse.push(stage.prepare(swell(t, 1.12, 0.9)?)?);
        }
        stage.play_prepared(pulse)?;
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
        kit.music_breath(stage, explainer_kit::animatic::HOLD_S)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}
