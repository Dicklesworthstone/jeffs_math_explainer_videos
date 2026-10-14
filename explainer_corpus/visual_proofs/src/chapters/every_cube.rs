//! Chapter 06, every cube: why a sixth cube fits round the square the first
//! five made, and why every cube does (storyboard 06_1 to 06_6).
//!
//! The pieces are the model's: `cubes::square(5)` is the picture of chapter
//! 05, `cubes::layer(6)` the sixth cube's layer (a corner, two whole slabs and
//! half a slab on each arm), and the arms' five pieces are the five smaller
//! cubes' sides laid end to end (`1, 2, 3, 4, 5`), paired as the model pairs
//! the slabs: the middle piece of an even cube alone at the arm's foot, then
//! `j` and `k − j` closing into a slab. Every count on screen is a fact
//! (`F-K.*`, `F-X.*`) checked against the model before anything is shown.

use explainer_kit::{Kit, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    Board, CardRow, Cell, LENS_AT, ProofCard, Side, Words, dim, fact, fact_ints, fade_to, fail, faint, file_on_shelf,
    flash, fly, formula, ground, group, hero, ink, lens, lens_mark, line_style, pause, question_card, question_pause,
    repaint, say, size, slide, source_line, tex_row, write_formula,
};
use super::kinetics::{Moves, arrive, fit_in, leave, play_with, rectangle, together};
use crate::model::{Rect, cubes};

pub struct EveryCube {
    pub kit: Kit,
}

/// The narration lines this chapter speaks, in order.
pub const LINES: [&str; 6] = ["06_1", "06_2", "06_3", "06_4", "06_5", "06_6"];

// ---------------------------------------------------------------- geometry

/// The five cubes' square, and the sixth cube's layer round it.
pub const FIVE: i64 = 5;
pub const SIXTH: i64 = 6;
/// The lower left corner of cell (0, 0) and the size of a cell: the 21 by 21
/// square of six cubes is 5.25 tall, under the shelf's floor.
pub const BOARD_AT: Vec3 = [-6.5, -2.3, 0.0];
pub const CELL: f64 = 0.25;
/// The dark between neighbouring pieces, on each side.
pub const SEAM: f64 = 0.02;
/// 06_4: the picture shrinks and rises to leave the card the foot of the
/// frame; the point it holds is the board's lower left corner, and the
/// shelf slot the proof takes (03, 04, 06, 09, 10, 11, ...).
pub const CARD_AT: Vec3 = [-6.5, -1.25, 0.0];
pub const CARD_K: f64 = 0.8;
pub const SLOT: usize = 2;
/// The middle of the column to the right of the board.
pub const COLUMN_X: f64 = 3.0;
/// The widest a line of the column may be.
pub const COLUMN_ROOM: f64 = 7.4;
/// How far a piece slides out of its arm, over the square, in cells: more
/// than the arm is wide, so it is clear of the arm.
pub const OUT: i64 = 7;
/// The slab strips of 06_3 beside the picture: a cell of a strip, the left
/// edge, and the heights of the 7 and the 8 strip.
pub const STRIP_CELL: f64 = 0.13;
pub const STRIP_X: f64 = 3.0;
pub const STRIP_H: f64 = 0.36;
/// The timeline of 06_6: its left end, its length for the years 0 to 2000.
pub const TIME_X: f64 = -0.6;
pub const TIME_W: f64 = 6.2;
pub const TIME_Y: f64 = 0.4;

/// The scene point of board coordinates (cells, fractions allowed).
pub fn at(x: f64, y: f64) -> Vec3 {
    v(BOARD_AT[0] + x * CELL, BOARD_AT[1] + y * CELL)
}

/// A flat piece of the board: the rectangle `r` in `color`, with a seam of
/// dark all round. Detached.
fn piece(stage: &mut Stage<'_>, r: Rect, color: Srgb) -> Mob {
    let corner = at(r.x as f64, r.y as f64);
    rectangle(stage, corner, r.w as f64 * CELL, r.h as f64 * CELL, SEAM, color, 1.0)
}

/// An L-shaped layer: the cells of the square `b` on a side that the square
/// `a` on a side does not have, as one polygon with a seam. Detached.
fn l_layer(stage: &mut Stage<'_>, a: i64, b: i64, color: Srgb) -> Mob {
    let (a, b, g) = (a as f64, b as f64, SEAM / CELL);
    let corners: Vec<Vec3> = if a == 0.0 {
        vec![at(g, g), at(b - g, g), at(b - g, b - g), at(g, b - g)]
    } else {
        vec![
            at(a + g, g),
            at(b - g, g),
            at(b - g, b - g),
            at(g, b - g),
            at(g, a + g),
            at(a + g, a + g),
        ]
    };
    stage
        .arena_mut()
        .add(Polygon::new(corners).style(super::draw::flat(color, 1.0)).build())
}

/// Which arm of the layer a piece is on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arm {
    /// The upright arm, to the right of the square.
    Right,
    /// The arm along the top.
    Top,
}

/// The rectangle of an arm of cube `k`'s layer from `from` to `to` cells
/// along it, round a square of side `s` (the arms are `k` wide).
pub fn arm_rect(arm: Arm, s: i64, k: i64, from: i64, to: i64) -> Rect {
    match arm {
        Arm::Right => Rect::new(s, from, k, to - from),
        Arm::Top => Rect::new(from, s, to - from, k),
    }
}

/// Where cube `k`'s arm is cut into pieces: the five smaller cubes' sides
/// end to end, `1, 2, …, k − 1`, as (piece, from, to) along the arm.
pub fn pieces_of_arm(k: i64) -> Vec<(i64, i64, i64)> {
    let mut from = 0;
    (1..k)
        .map(|j| {
            let p = (j, from, from + j);
            from += j;
            p
        })
        .collect()
}

/// Where each piece goes when the arm is paired as the model's slabs are:
/// the middle piece of an even cube alone at the foot (half a slab), then
/// `j` and `k − j` side by side as slabs. The same pieces as
/// [`pieces_of_arm`], (piece, from, to).
#[cfg(test)]
pub fn paired_arm(k: i64) -> Vec<(i64, i64, i64)> {
    let foot = if k % 2 == 0 { k / 2 } else { 0 };
    let mut out = Vec::new();
    if k % 2 == 0 {
        out.push((k / 2, 0, k / 2));
    }
    for j in 1..=((k - 1) / 2) {
        let start = foot + (j - 1) * k;
        out.push((j, start, start + j));
        out.push((k - j, start + j, start + k));
    }
    out.sort_unstable();
    out
}

/// The slabs the paired arm makes, as (from, to) along the arm: the half slab
/// at the foot if the cube is even, then whole slabs of length `k`.
pub fn slabs_of_arm(k: i64) -> Vec<(i64, i64)> {
    let foot = if k % 2 == 0 { k / 2 } else { 0 };
    let mut out = Vec::new();
    if k % 2 == 0 {
        out.push((0, k / 2));
    }
    for i in 0..((k - 1) / 2) {
        out.push((foot + i * k, foot + (i + 1) * k));
    }
    out
}

/// The sides of the five smaller cubes laid end to end, 0 first: the feet of
/// the arm's pieces (0, 1, 3, 6, 10, 15).
pub fn feet(through: i64) -> Vec<i64> {
    (0..=through).map(cubes::side).collect()
}

/// A colour a step lighter or darker than `c`, `t` from −1 (much darker) to 1
/// (much lighter).
pub fn shade(c: Srgb, t: f64) -> Srgb {
    let toward = if t >= 0.0 { 1.0 } else { 0.0 };
    let a = t.abs();
    Srgb {
        r: c.r + (toward - c.r) * a,
        g: c.g + (toward - c.g) * a,
        b: c.b + (toward - c.b) * a,
    }
}

/// The shade of the `i`-th (from 0) of `n` layers of one cube: the first the
/// lightest, the last the darkest, one layer the colour itself.
pub fn step_shade(c: Srgb, i: usize, n: usize) -> Srgb {
    if n <= 1 {
        c
    } else {
        shade(c, 0.35 - 0.7 * i as f64 / (n as f64 - 1.0))
    }
}

// the strip of eight cubes' arm (28 cells) is inside the frame, and a piece
// 6 wide slid out of the arm by OUT cells is clear of it and still on the
// old square
const _: () = assert!(STRIP_X + 28.0 * STRIP_CELL < FRAME_X_RADIUS - 0.3);
const _: () = assert!(OUT >= 6 && OUT + 6 <= 15 + 6);

/// A mixed number as TeX from a fact's fraction: `5/2` is `2\frac{1}{2}`,
/// `3` is `3`.
pub fn mixed(fraction: &str) -> String {
    match fraction.split_once('/') {
        Some((n, d)) => match (n.parse::<i64>(), d.parse::<i64>()) {
            (Ok(n), Ok(d)) if n > d => format!("{}\\frac{{{}}}{{{}}}", n / d, n % d, d),
            _ => format!("\\frac{{{n}}}{{{d}}}"),
        },
        None => fraction.to_owned(),
    }
}

/// An arc over a strip from `x0` to `x1` at height `y`, `rise` high, as an
/// open path of points. Detached.
fn arc(stage: &mut Stage<'_>, x0: f64, x1: f64, y: f64, rise: f64, color: Srgb) -> Mob {
    let n = 18;
    let pts: Vec<Vec3> = (0..=n)
        .map(|i| {
            let t = i as f64 / n as f64;
            let a = std::f64::consts::PI * t;
            v(x0 + (x1 - x0) * (1.0 - a.cos()) / 2.0, y + rise * a.sin())
        })
        .collect();
    stage
        .arena_mut()
        .add(polyline(pts).style(line_style(color, 2.5)).build())
}

// -------------------------------------------------------------- the facts

/// What the chapter reads from the facts, checked against the model.
struct Reads {
    cubes: Vec<i64>,
    side: i64,
    sixth_cube: i64,
    sixth_side: i64,
    /// The sixth cube's arm: its width and its length, and its slabs.
    arm: (i64, i64),
    arm_slabs: String,
    /// The strips: the 7 cube's and the 8 cube's arm, (k, length, slabs).
    strips: [(i64, i64, String); 2],
    runs: Vec<Vec<i64>>,
    /// The years of Nicomachus, al-Karaji and Fry.
    years: [i64; 3],
}

/// The year a source fact starts with (`100,II.20.5` is 100).
fn year_of(key: &str) -> fmn::Result<i64> {
    let value = fact(key)?;
    let first = value.split(',').next().unwrap_or_default();
    first
        .parse()
        .map_err(|_| fail(format!("fact {key} = {value:?} does not start with a year")))
}

fn reads() -> fmn::Result<Reads> {
    let want = |key: &str, got: i64, model: i64| -> fmn::Result<i64> {
        if got == model {
            Ok(got)
        } else {
            Err(fail(format!("{key} is {got} but the model's is {model}")))
        }
    };
    let cubes_f = fact_ints("F-K.cubes")?;
    let sides = fact_ints("F-K.sides")?;
    let side = want("F-K.side", super::draw::fact_int("F-K.side")?, cubes::side(FIVE))?;
    for (i, &s) in sides.iter().enumerate() {
        want("F-K.sides", s, cubes::side(i as i64 + 1))?;
    }
    let sixth_cube = want(
        "F-K.sixth.cube",
        super::draw::fact_int("F-K.sixth.cube")?,
        cubes::layer(SIXTH).iter().map(|p| p.rect.area()).sum(),
    )?;
    let sixth_side = want(
        "F-K.sixth.side",
        super::draw::fact_int("F-K.sixth.side")?,
        cubes::side(SIXTH),
    )?;
    let arm = fact_ints(&format!("F-K.arm.{SIXTH}"))?;
    let arm = (arm[0], arm[1]);
    want("F-K.arm.6 (long)", arm.1, side)?;
    let arm_slabs = fact(&format!("F-K.arm.{SIXTH}.slabs"))?;
    let mut strips = Vec::new();
    for k in [SIXTH + 1, SIXTH + 2] {
        let a = fact_ints(&format!("F-K.arm.{k}"))?;
        want(&format!("F-K.arm.{k} (long)"), a[1], cubes::side(k - 1))?;
        strips.push((k, a[1], fact(&format!("F-K.arm.{k}.slabs"))?));
    }
    let strips: [(i64, i64, String); 2] = strips.try_into().map_err(|_| fail("two strips".into()))?;
    let runs: Vec<Vec<i64>> = fact("F-K.odd_runs")?
        .split('|')
        .map(|r| {
            r.split(',')
                .map(|n| n.parse().map_err(|_| fail(format!("odd run {r:?}"))))
                .collect()
        })
        .collect::<fmn::Result<_>>()?;
    for (i, run) in runs.iter().enumerate() {
        if *run != cubes::odd_run(i as i64 + 1) {
            return Err(fail(format!("F-K.odd_runs: run {} is not the model's", i + 1)));
        }
    }
    Ok(Reads {
        cubes: cubes_f,
        side,
        sixth_cube,
        sixth_side,
        arm,
        arm_slabs,
        strips,
        runs,
        years: [
            year_of("F-X.nicomachus")?,
            year_of("F-X.al_karaji")?,
            year_of("F-X.fry")?,
        ],
    })
}

// ------------------------------------------------------------- the pieces

/// The orders the arm's pieces take, foot to end: as laid (the five smaller
/// cubes' sides end to end), after the piece 5 has hopped next to the piece
/// 1, and paired (the middle piece hops to the foot, so that 2 and 4 are
/// neighbours too). The last is the model's: [`paired_arm`].
pub const ORDERS: [[i64; 5]; 3] = [[1, 2, 3, 4, 5], [1, 5, 2, 3, 4], [3, 1, 5, 2, 4]];
/// The piece that hops in each of the two moves.
pub const HOPPERS: [i64; 2] = [5, 3];

/// Where piece `j` starts along the arm when the pieces lie in `order`, each
/// as long as its number.
pub fn start_in(order: &[i64], j: i64) -> i64 {
    order.iter().take_while(|&&o| o != j).sum()
}

/// One of the arm's five pieces: its group (the piece and its numeral), the
/// piece alone, and where it lies along the arm in each of [`ORDERS`].
struct ArmPiece {
    j: i64,
    arm: Arm,
    group: Mob,
    fill: Mob,
    /// Where it starts along the arm in each order, in cells from the foot.
    at: [i64; 3],
}

/// A displacement of `cells` along `arm`, in scene units.
fn along_arm(arm: Arm, cells: i64) -> Vec3 {
    let d = cells as f64 * CELL;
    match arm {
        Arm::Right => v(0.0, d),
        Arm::Top => v(d, 0.0),
    }
}

/// The slide that takes a piece out of its arm over the square, and its
/// reverse (the way back).
fn out_of(arm: Arm) -> Vec3 {
    match arm {
        Arm::Right => v(-(OUT as f64) * CELL, 0.0),
        Arm::Top => v(0.0, -(OUT as f64) * CELL),
    }
}

/// One hop of an arm: piece `hopper` slides out of the arm, over the square,
/// to its new place; the pieces between shift along the arm to close and
/// open the gap; and it slides back in. `to` is the order after the hop.
fn hop(stage: &mut Stage<'_>, pieces: &[&ArmPiece], hopper: i64, from: usize, to: usize) -> fmn::Result<()> {
    let arm = pieces.first().map(|p| p.arm).ok_or_else(|| fail("no pieces".into()))?;
    let out = out_of(arm);
    let back = v(-out[0], -out[1]);
    let moving = pieces
        .iter()
        .find(|p| p.j == hopper)
        .ok_or_else(|| fail(format!("no piece {hopper}")))?;
    together(stage, vec![slide(moving.group, out, 0.35)?], 0.0)?;
    let mut along = Vec::new();
    for p in pieces {
        let d = p.at[to] - p.at[from];
        if d != 0 {
            along.push(slide(p.group, along_arm(arm, d), 0.45)?);
        }
    }
    together(stage, along, 0.0)?;
    together(stage, vec![slide(moving.group, back, 0.35)?], 0.0)
}

/// The numeral `j`, in the dark of the ground, in the middle of rectangle
/// `r`. Detached.
fn numeral(kit: &Kit, stage: &mut Stage<'_>, j: i64, r: Rect) -> fmn::Result<Mob> {
    let centre = at(r.x as f64 + r.w as f64 / 2.0, r.y as f64 + r.h as f64 / 2.0);
    let m = kit.text(stage, &j.to_string(), 22.0, ground(), centre)?;
    fit_in(stage, m, r.w as f64 * CELL * 0.6, r.h as f64 * CELL * 0.8);
    Ok(m)
}

/// A strip of cube `k`'s arm for the pairing drawn small: its pieces `1 …
/// k − 1` laid end to end from `x0` at height `y`, a bar each. Returns the
/// pieces and the x of each piece's middle.
fn strip(stage: &mut Stage<'_>, k: i64, x0: f64, y: f64) -> (Vec<Mob>, Vec<f64>) {
    let mut mobs = Vec::new();
    let mut mids = Vec::new();
    for (j, from, to) in pieces_of_arm(k) {
        let w = (to - from) as f64 * STRIP_CELL;
        let left = x0 + from as f64 * STRIP_CELL;
        let c = if j == k / 2 && k % 2 == 0 { dim() } else { size(4) };
        mobs.push(rectangle(stage, v(left, y - STRIP_H / 2.0), w, STRIP_H, 0.012, c, 1.0));
        mids.push(left + w / 2.0);
    }
    (mobs, mids)
}

impl SceneConstruct for EveryCube {
    fn name(&self) -> &str {
        "06_every_cube"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let r = reads()?;
        let board = Board::new(BOARD_AT, CELL);
        let (s, k) = (r.side, SIXTH);

        // --------------------------------------------- the five cubes' square
        let five = cubes::square(FIVE);
        let square: Vec<Mob> = five
            .iter()
            .map(|p| piece(stage, p.rect, size(p.cube as usize)))
            .collect();
        // the sixth layer's outline: the L of cells the sixth cube takes
        let sixth_layer = cubes::layer(SIXTH);
        let sixth_cells: Vec<Cell> = sixth_layer.iter().flat_map(|p| p.rect.cells()).collect();
        let outline = board.outline(stage, &sixth_cells, super::draw::faded_line(dim(), 2.5, 1.0))?;

        // ---------------------------------------------------------- 06_1
        let line = say(&kit, stage, LINES[0])?;
        for &m in &square {
            stage.add_to_scene(m)?;
        }
        let cube_tex = kit.tex(
            stage,
            &format!("{k}^{{3}} = {}\\text{{ tiles}}", r.sixth_cube),
            52.0,
            ink(),
            &[],
            v(COLUMN_X, 3.0),
        )?;
        line.wait_sentence(stage, 1)?;
        play_with(stage, 0.0, |st| {
            Ok(vec![arrive(st, outline, 0.6)?, arrive(st, cube_tex, 0.6)?])
        })?;
        line.wait_last(stage)?;
        let words = line
            .sentence_texts()
            .pop()
            .ok_or_else(|| fail("06_1 has no sentences".into()))?;
        let question = question_card(&kit, stage, &words, v(COLUMN_X, 0.9))?;
        play_with(stage, 0.0, |st| Ok(vec![arrive(st, question, 0.5)?]))?;
        question_pause(&kit, stage)?;
        play_with(stage, 0.0, |st| Ok(vec![leave(st, question, 0.4)?]))?;

        // ---------------------------------------------------------- 06_2
        let line = say(&kit, stage, LINES[1])?;
        // the layer is six wide: a brace under the right arm
        let wide = board.dimension(
            &kit,
            stage,
            Rect::new(s, 0, k, s),
            Side::Below,
            Words::Text(&k.to_string()),
            30.0,
            hero(),
        )?;
        play_with(stage, 0.0, |st| {
            Ok(vec![arrive(st, wide.brace, 0.5)?, arrive(st, wide.label, 0.5)?])
        })?;
        // the pieces of the sixth layer, in gold: the corner, then the arms
        let layer_mobs: Vec<(Mob, Rect)> = sixth_layer
            .iter()
            .map(|p| (piece(stage, p.rect, hero()), p.rect))
            .collect();
        let corner_mob = layer_mobs
            .iter()
            .find(|(_, rect)| rect.x == s && rect.y == s)
            .map(|&(m, _)| m)
            .ok_or_else(|| fail("the sixth layer has no corner".into()))?;
        line.wait_sentence(stage, 1)?;
        play_with(stage, 0.0, |st| Ok(vec![arrive(st, corner_mob, 0.6)?]))?;
        line.wait_sentence(stage, 2)?;
        let long = board.dimension(
            &kit,
            stage,
            Rect::new(s, 0, k, s),
            Side::Right,
            Words::Text(&r.arm.1.to_string()),
            30.0,
            hero(),
        )?;
        let slabs_tex = kit.tex(
            stage,
            &format!(
                "{} \\div {} = {}\\text{{ slabs}}",
                r.arm.1,
                r.arm.0,
                mixed(&r.arm_slabs)
            ),
            48.0,
            ink(),
            &[],
            v(COLUMN_X, 1.9),
        )?;
        play_with(stage, 0.0, |st| {
            Ok(vec![
                arrive(st, long.brace, 0.5)?,
                arrive(st, long.label, 0.5)?,
                arrive(st, slabs_tex, 0.6)?,
            ])
        })?;
        line.wait_sentence(stage, 3)?;
        let arms: Vec<Mob> = layer_mobs
            .iter()
            .filter(|(_, rect)| !(rect.x == s && rect.y == s))
            .map(|&(m, _)| m)
            .collect();
        play_with(stage, 0.0, |st| {
            let mut m: Moves = Vec::new();
            for &a in &arms {
                m.push(arrive(st, a, 0.5)?);
            }
            Ok(m)
        })?;
        let five_plus_one = kit.tex(stage, "5 + 1 = 6", 60.0, ink(), &[], v(COLUMN_X, 0.7))?;
        line.wait_sentence(stage, 4)?;
        play_with(stage, 0.0, |st| Ok(vec![arrive(st, five_plus_one, 0.5)?]))?;
        line.wait_sentence(stage, 5)?;
        // the row of five for five returns, and its question mark is answered
        let totals: Vec<i64> = r
            .cubes
            .iter()
            .scan(0, |sum, &c| {
                *sum += c;
                Some(*sum)
            })
            .collect();
        let mut parts: Vec<(String, Srgb)> = totals.iter().map(|t| (t.to_string(), ink())).collect();
        parts.push(("?".to_owned(), dim()));
        let part_refs: Vec<(&str, Srgb)> = parts.iter().map(|(t, c)| (t.as_str(), *c)).collect();
        let row = tex_row(&kit, stage, &part_refs, 44.0, v(COLUMN_X, -0.7), 0.55, COLUMN_ROOM)?;
        let answer = kit.tex(
            stage,
            &format!(
                "{} = {} \\times {}",
                r.sixth_side * r.sixth_side,
                r.sixth_side,
                r.sixth_side
            ),
            54.0,
            hero(),
            &[],
            v(COLUMN_X, -1.8),
        )?;
        let question_mark = *row.last().ok_or_else(|| fail("no row".into()))?;
        play_with(stage, 0.0, |st| {
            let mut m: Moves = Vec::new();
            for &t in &row {
                m.push(arrive(st, t, 0.5)?);
            }
            Ok(m)
        })?;
        flash(
            stage,
            &arms.iter().copied().chain([corner_mob]).collect::<Vec<_>>(),
            hero(),
            1.03,
            0.8,
        )?;
        play_with(stage, 0.0, |st| {
            Ok(vec![leave(st, question_mark, 0.4)?, arrive(st, answer, 0.6)?])
        })?;
        // the whole square: twenty-one on a side
        line.wait_sentence(stage, 6)?;
        let whole = board.dimension(
            &kit,
            stage,
            Rect::new(0, 0, r.sixth_side, r.sixth_side),
            Side::Below,
            Words::Text(&r.sixth_side.to_string()),
            30.0,
            hero(),
        )?;
        play_with(stage, 0.0, |st| {
            Ok(vec![
                leave(st, wide.brace, 0.3)?,
                leave(st, wide.label, 0.3)?,
                arrive(st, whole.brace, 0.5)?,
                arrive(st, whole.label, 0.5)?,
            ])
        })?;
        line.wait_end(stage)?;

        // ---------------------------------------------------------- 06_3
        let line = say(&kit, stage, LINES[2])?;
        // the right column goes; the old square dims and is ruled
        let mut clear: Vec<Mob> = vec![slabs_tex, five_plus_one, answer, cube_tex];
        clear.extend(row.iter().copied().filter(|&m| m != question_mark));
        clear.extend([whole.brace, whole.label, long.brace, long.label]);
        let mut feet_marks: Vec<Mob> = Vec::new();
        let ruled = feet(FIVE);
        for &f in &ruled[1..] {
            // marks on the left edge and the bottom edge, and the faint
            // lines from them across to the arms
            let left = stage.arena_mut().add(
                polyline(vec![at(0.0, f as f64), at(r.sixth_side as f64, f as f64)])
                    .style(super::draw::faded_line(ink(), 1.5, 0.35))
                    .build(),
            );
            let bottom = stage.arena_mut().add(
                polyline(vec![at(f as f64, 0.0), at(f as f64, r.sixth_side as f64)])
                    .style(super::draw::faded_line(ink(), 1.5, 0.35))
                    .build(),
            );
            feet_marks.extend([left, bottom]);
        }
        play_with(stage, 0.0, |st| {
            let mut m: Moves = Vec::new();
            for &c in &clear {
                m.push(leave(st, c, 0.4)?);
            }
            Ok(m)
        })?;
        let mut dimming = Vec::new();
        for &m in &square {
            dimming.push(fade_to(m, 0.3, 0.6)?);
        }
        together(stage, dimming, 0.0)?;
        play_with(stage, 0.0, |st| {
            let mut m: Moves = Vec::new();
            for &f in &feet_marks {
                m.push(arrive(st, f, 0.6)?);
            }
            Ok(m)
        })?;
        // the arms' five pieces, in the sizes' colours, each with its numeral
        let mut arm_pieces: Vec<ArmPiece> = Vec::new();
        for arm in [Arm::Right, Arm::Top] {
            for (j, from, to) in pieces_of_arm(k) {
                let rect = arm_rect(arm, s, k, from, to);
                let fill = piece(stage, rect, size(j as usize));
                let num = numeral(&kit, stage, j, rect)?;
                let g = group(stage, &[fill, num])?;
                let at = ORDERS.map(|o| start_in(&o, j));
                arm_pieces.push(ArmPiece {
                    j,
                    arm,
                    group: g,
                    fill,
                    at,
                });
            }
        }
        line.wait_sentence(stage, 0)?;
        // the gold arms give way to the arms' pieces
        let gold_arms: Vec<Mob> = arms.clone();
        play_with(stage, 0.0, |st| {
            let mut m: Moves = Vec::new();
            for &a in &gold_arms {
                m.push(leave(st, a, 0.5)?);
            }
            for p in &arm_pieces {
                m.push(arrive(st, p.group, 0.5)?);
            }
            Ok(m)
        })?;
        // the pairing in two hops of each arm: the 5 next to the 1 (so one and
        // five make six), then the 3 to the foot (so two and four are
        // neighbours and make six, and the 3 is half a slab)
        let pair_label = |kit: &Kit, stage: &mut Stage<'_>, text: &str, y: f64| -> fmn::Result<Mob> {
            kit.tex(stage, text, 40.0, ink(), &[], v(1.3, y))
        };
        let one_five = pair_label(&kit, stage, "1 + 5 = 6", 2.9)?;
        let two_four = pair_label(&kit, stage, "2 + 4 = 6", 2.2)?;
        let three = pair_label(&kit, stage, "3 = \\frac{1}{2} \\times 6", 1.5)?;
        let labels = [one_five, two_four, three];
        for n in 0..2 {
            line.wait_sentence(stage, n + 1)?;
            for arm in [Arm::Right, Arm::Top] {
                let of_arm: Vec<&ArmPiece> = arm_pieces.iter().filter(|p| p.arm == arm).collect();
                hop(stage, &of_arm, HOPPERS[n], n, n + 1)?;
            }
            play_with(stage, 0.0, |st| Ok(vec![arrive(st, labels[n], 0.4)?]))?;
        }
        // the two greens, one in each arm, are half a slab each
        line.wait_sentence(stage, 3)?;
        let greens: Vec<Mob> = arm_pieces.iter().filter(|p| p.j == 3).map(|p| p.fill).collect();
        flash(stage, &greens, size(3), 1.15, 0.8)?;
        play_with(stage, 0.0, |st| Ok(vec![arrive(st, labels[2], 0.4)?]))?;
        line.wait_sentence(stage, 4)?;
        // the slabs closed: a gold outline round each whole one
        let mut slab_outlines: Vec<Mob> = Vec::new();
        for arm in [Arm::Right, Arm::Top] {
            for (from, to) in slabs_of_arm(k) {
                if to - from == k {
                    let rect = arm_rect(arm, s, k, from, to);
                    slab_outlines.push(board.rect_outline(stage, rect, line_style(hero(), 3.0))?);
                }
            }
        }
        play_with(stage, 0.0, |st| {
            let mut m: Moves = Vec::new();
            for &o in &slab_outlines {
                m.push(arrive(st, o, 0.5)?);
            }
            Ok(m)
        })?;
        // the strips of 7 and of 8 cubes pair themselves beside it
        let mut strip_mobs: Vec<Mob> = Vec::new();
        let mut arcs: Vec<Vec<Mob>> = Vec::new();
        let ys = [0.5, -1.0];
        for (i, (kk, _long, _slabs)) in r.strips.iter().enumerate() {
            let (bars, mids) = strip(stage, *kk, STRIP_X, ys[i]);
            let n = bars.len();
            let mut these = Vec::new();
            for p in 1..=((kk - 1) / 2) as usize {
                let (a, b) = (mids[p - 1], mids[n - p]);
                let rise = 0.22 + 0.18 * (((kk - 1) / 2) as usize - p) as f64;
                these.push(arc(stage, a, b, ys[i] + STRIP_H / 2.0 + 0.05, rise, hero()));
            }
            strip_mobs.extend(bars);
            arcs.push(these);
        }
        play_with(stage, 0.0, |st| {
            let mut m: Moves = Vec::new();
            for &b in &strip_mobs {
                m.push(arrive(st, b, 0.5)?);
            }
            Ok(m)
        })?;
        for these in &arcs {
            let these = these.clone();
            play_with(stage, 0.15, |st| {
                let mut m: Moves = Vec::new();
                for &a in &these {
                    m.push(arrive(st, a, 0.6)?);
                }
                Ok(m)
            })?;
        }
        let letters = kit.tex(
            stage,
            "\\text{pieces } 1, 2, \\ldots, k - 1 \\text{: each, with its partner, makes } k",
            34.0,
            ink(),
            &[],
            v(COLUMN_X, -2.2),
        )?;
        fit_in(stage, letters, COLUMN_ROOM, 1.0);
        play_with(stage, 0.0, |st| Ok(vec![arrive(st, letters, 0.6)?]))?;
        line.wait_end(stage)?;

        // ---------------------------------------------------------- 06_4
        let line = say(&kit, stage, LINES[3])?;
        // the picture wakes: the sixth layer in gold again, the rest bright
        let mut wake: Vec<AnimBuilder> = Vec::new();
        for &m in &square {
            wake.push(fade_to(m, 1.0, 0.6)?);
        }
        for p in &arm_pieces {
            wake.push(repaint(p.fill, hero(), 1.0, 0.6)?);
        }
        let mut gone: Vec<Mob> = Vec::new();
        gone.extend(&feet_marks);
        gone.extend(&strip_mobs);
        gone.extend(arcs.iter().flatten());
        gone.extend([letters, one_five, two_four, three]);
        together(stage, wake, 0.0)?;
        play_with(stage, 0.0, |st| {
            let mut m: Moves = Vec::new();
            for &g in &gone {
                m.push(leave(st, g, 0.5)?);
            }
            Ok(m)
        })?;
        // every mobject of the picture, to shrink and rise
        let mut picture: Vec<Mob> = square.clone();
        picture.extend(arm_pieces.iter().map(|p| p.group));
        picture.push(corner_mob);
        picture.extend(&slab_outlines);
        picture.push(outline);
        line.wait_sentence(stage, 1)?;
        relocate(stage, &picture, BOARD_AT, CARD_AT, CARD_K, 0.8)?;
        // the formula, boxed
        let formula_src = "1^{3} + 2^{3} + \\cdots + n^{3} = (1 + 2 + \\cdots + n)^{2}";
        let sum = formula(&kit, stage, formula_src, 46.0, ink(), v(COLUMN_X, 2.7))?;
        write_formula(stage, &sum, 1.4)?;
        let pad = 0.3;
        let (fw, fh) = (stage.get_width(sum.mob), stage.get_height(sum.mob));
        let boxed = stage.arena_mut().add(
            Rectangle::new()
                .width(fw + 2.0 * pad)
                .height(fh + 2.0 * pad)
                .corner_radius(0.12)
                .style(line_style(ink(), 2.5))
                .build()?
                .moved_to(v(COLUMN_X, 2.7)),
        );
        play_with(stage, 0.0, |st| Ok(vec![arrive(st, boxed, 0.4)?]))?;
        line.wait_end(stage)?;
        // the card
        let mut card = ProofCard::build(
            &kit,
            stage,
            &[
                CardRow::Shown(Words::Tex(SHOWN_TEX)),
                CardRow::Assumed(Words::Text(ASSUMED_WRAPS)),
                CardRow::Assumed(Words::Text(ASSUMED_PAIRS)),
            ],
        )?;
        card.show(stage, 0.6)?;
        pause(stage, 0.4)?;
        // the close-up of the arm on its ruler: whole slabs and the half slab
        let ring = lens(stage);
        let (lx, ly) = (LENS_AT[0], LENS_AT[1]);
        let unit = 1.7 / r.arm.1 as f64;
        let bar_h = unit * k as f64;
        let mut close: Vec<Mob> = vec![ring];
        for (from, to) in slabs_of_arm(k) {
            let c = if to - from == k { hero() } else { size(3) };
            close.push(rectangle(
                stage,
                v(lx - 0.85 + from as f64 * unit, ly - bar_h / 2.0),
                (to - from) as f64 * unit,
                bar_h,
                0.025,
                c,
                1.0,
            ));
        }
        play_with(stage, 0.0, |st| {
            let mut m: Moves = Vec::new();
            for &c in &close {
                m.push(arrive(st, c, 0.5)?);
            }
            Ok(m)
        })?;
        card.tick(&kit, stage, 1, REASON_WRAPS)?;
        card.tick(&kit, stage, 2, REASON_PAIRS)?;
        let tick = lens_mark(stage, LENS_AT, 0.7, true);
        play_with(stage, 0.0, |st| Ok(vec![arrive(st, tick, 0.3)?]))?;
        pause(stage, 0.5)?;
        // the card has passed: its picture goes to the shelf
        let mut leaving: Vec<Mob> = close.clone();
        leaving.extend([tick, sum.mob, boxed]);
        leaving.extend(card.mobs());
        play_with(stage, 0.0, |st| {
            leaving
                .iter()
                .map(|&m| leave(st, m, 0.4))
                .collect::<fmn::Result<Moves>>()
        })?;
        let board_mid = v(
            CARD_AT[0] + r.sixth_side as f64 * CELL * CARD_K / 2.0,
            CARD_AT[1] + r.sixth_side as f64 * CELL * CARD_K / 2.0,
        );
        file_on_shelf(
            stage,
            &picture,
            board_mid,
            r.sixth_side as f64 * CELL * CARD_K,
            SLOT,
            1.0,
        )?;

        // ---------------------------------------------------------- 06_5
        let line = say(&kit, stage, LINES[4])?;
        // the square of five cubes again: one L-shaped band a cube, then each
        // band cut into thin layers
        let mut thick: Vec<Mob> = Vec::new();
        for c in 1..=FIVE {
            thick.push(l_layer(stage, cubes::side(c - 1), cubes::side(c), size(c as usize)));
        }
        let mut thin: Vec<(Mob, i64, usize, usize)> = Vec::new();
        for c in 1..=FIVE {
            let n = c as usize;
            for i in 0..n {
                let m = cubes::side(c - 1) + 1 + i as i64;
                let mob = l_layer(stage, m - 1, m, size(c as usize));
                thin.push((mob, c, i, n));
            }
        }
        play_with(stage, 0.12, |st| {
            let mut m: Moves = Vec::new();
            for &t in &thick {
                m.push(arrive(st, t, 0.5)?);
            }
            Ok(m)
        })?;
        // the bands are cut: the thin layers take their places, a shade each
        play_with(stage, 0.0, |st| {
            let mut m: Moves = Vec::new();
            for &t in &thick {
                m.push(leave(st, t, 0.4)?);
            }
            for &(mob, _, _, _) in &thin {
                m.push(arrive(st, mob, 0.4)?);
            }
            Ok(m)
        })?;
        let mut shades = Vec::new();
        for &(mob, c, i, n) in &thin {
            shades.push(repaint(mob, step_shade(size(c as usize), i, n), 1.0, 0.8)?);
        }
        together(stage, shades, 0.08)?;
        // the odd numbers in runs, one row a cube, each term its layer's shade
        let mut rows: Vec<Vec<Mob>> = Vec::new();
        for (idx, run) in r.runs.iter().enumerate() {
            let c = idx as i64 + 1;
            let mut parts: Vec<(String, Srgb)> = vec![(format!("{c}^{{3}} ="), ink())];
            for (i, o) in run.iter().enumerate() {
                if i > 0 {
                    parts.push(("+".to_owned(), dim()));
                }
                parts.push((o.to_string(), step_shade(size(c as usize), i, run.len())));
            }
            let refs: Vec<(&str, Srgb)> = parts.iter().map(|(t, c)| (t.as_str(), *c)).collect();
            rows.push(tex_row(
                &kit,
                stage,
                &refs,
                38.0,
                v(COLUMN_X, 3.1 - 0.62 * idx as f64),
                0.22,
                COLUMN_ROOM,
            )?);
        }
        for (n, row) in rows.iter().enumerate() {
            if n == 1 {
                line.wait_sentence(stage, 1)?;
            }
            // the line is three sentences (the first cube, the two cube, and
            // "so this square is the first proof's layers again"): the rows
            // from the three cube on arrive with the last
            if n == 2 {
                line.wait_sentence(stage, 2)?;
            }
            let row = row.clone();
            play_with(stage, 0.05, |st| {
                let mut m: Moves = Vec::new();
                for &t in &row {
                    m.push(arrive(st, t, 0.5)?);
                }
                Ok(m)
            })?;
        }
        let rule = kit.tex(
            stage,
            "\\text{cube } k \\text{ is } k \\text{ odd numbers from } k(k - 1) + 1",
            34.0,
            ink(),
            &[],
            v(COLUMN_X, -0.15),
        )?;
        fit_in(stage, rule, COLUMN_ROOM, 1.0);
        let centred = kit.tex(
            stage,
            "\\text{centred on } k^{2}\\text{: } k \\times k^{2} = k^{3}",
            34.0,
            ink(),
            &[],
            v(COLUMN_X, -0.85),
        )?;
        let last_odd = r.runs.last().and_then(|run| run.last()).copied().unwrap_or(0);
        let total = kit.tex(
            stage,
            &format!("1 + 3 + 5 + \\cdots + {last_odd} = {} \\times {}", r.side, r.side),
            44.0,
            hero(),
            &[],
            v(COLUMN_X, -1.75),
        )?;
        play_with(stage, 0.2, |st| {
            Ok(vec![
                arrive(st, rule, 0.5)?,
                arrive(st, centred, 0.5)?,
                arrive(st, total, 0.6)?,
            ])
        })?;
        line.wait_end(stage)?;

        // ---------------------------------------------------------- 06_6
        let line = say(&kit, stage, LINES[5])?;
        let mut away: Vec<Mob> = vec![rule, centred, total];
        for row in &rows {
            away.extend(row);
        }
        play_with(stage, 0.0, |st| {
            let mut m: Moves = Vec::new();
            for &a in &away {
                m.push(leave(st, a, 0.4)?);
            }
            Ok(m)
        })?;
        // a line of years: the dots, and who and what
        let names = [
            ("Nicomachus", "runs of odd numbers", r.years[0], true),
            ("al-Karaji", "L-shaped layers", r.years[1], true),
            ("Fry", "slabs cut in half", r.years[2], false),
        ];
        let timeline = stage.arena_mut().add(
            polyline(vec![v(TIME_X, TIME_Y), v(TIME_X + TIME_W, TIME_Y)])
                .style(line_style(faint(), 3.0))
                .build(),
        );
        play_with(stage, 0.0, |st| Ok(vec![arrive(st, timeline, 0.4)?]))?;
        let mut marks: Vec<Vec<Mob>> = Vec::new();
        for &(name, what, year, above) in &names {
            let x = TIME_X + TIME_W * year as f64 / 2000.0;
            let dot = stage.arena_mut().add(
                Circle::new()
                    .radius(0.09)
                    .arc_center(v(x, TIME_Y))
                    .style(super::draw::flat(hero(), 1.0))
                    .build(),
            );
            let dy = if above { 0.55 } else { -0.55 };
            let who = kit.text(stage, name, 30.0, ink(), v(x, TIME_Y + dy))?;
            let year_text = if year == r.years[0] || year == r.years[1] {
                format!("about {year}")
            } else {
                year.to_string()
            };
            let when = kit.text(stage, &year_text, 24.0, dim(), v(x, TIME_Y + 1.7 * dy))?;
            let did = kit.text(stage, what, 24.0, dim(), v(x, TIME_Y + 2.5 * dy))?;
            marks.push(vec![dot, who, when, did]);
        }
        line.wait_sentence(stage, 0)?;
        for (n, m) in marks.iter().take(2).enumerate() {
            if n == 1 {
                line.wait_for(stage, "Al-Karaji")?;
            }
            let m = m.clone();
            play_with(stage, 0.1, |st| {
                let mut a: Moves = Vec::new();
                for &x in &m {
                    a.push(arrive(st, x, 0.5)?);
                }
                Ok(a)
            })?;
        }
        line.wait_sentence(stage, 1)?;
        let fry = marks[2].clone();
        play_with(stage, 0.1, |st| {
            let mut a: Moves = Vec::new();
            for &x in &fry {
                a.push(arrive(st, x, 0.5)?);
            }
            Ok(a)
        })?;
        let source_words = format!("the cut slabs: A. L. Fry, Mathematics Magazine, {}", r.years[2]);
        let source = source_line(&kit, stage, &source_words, false)?;
        play_with(stage, 0.0, |st| Ok(vec![arrive(st, source, 0.5)?]))?;
        line.wait_end(stage)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

// the card's words
const SHOWN_TEX: &str = r"\text{for every whole number } n \text{ from 1, the cubes up to } n^{3} \text{ add up to the square of } 1 + 2 + \cdots + n";
const ASSUMED_WRAPS: &str = "each cube's layer wraps the square the cubes before it made";
const REASON_WRAPS: &str = "the arm is the sides laid end to end";
const ASSUMED_PAIRS: &str = "each arm is k − 1 half slabs, the corner the last";
const REASON_PAIRS: &str = "pairs make k; the first cube is the corner tile";

/// Shrink a picture by `k` about `from` and put that point at `to`: every
/// mobject of `mobs` (on the screen, not grouped) moves with it, rigidly.
fn relocate(stage: &mut Stage<'_>, mobs: &[Mob], from: Vec3, to: Vec3, k: f64, rt: f64) -> fmn::Result<()> {
    let mut moves = Vec::new();
    for &m in mobs {
        let c = stage.get_center(m);
        let target = v(to[0] + k * (c[0] - from[0]), to[1] + k * (c[1] - from[1]));
        moves.push(fly(m, target, k, rt)?);
    }
    together(stage, moves, 0.0)
}

#[cfg(test)]
mod tests {
    use super::super::draw::testing;
    use super::*;
    use crate::model::layers::layer;
    use crate::model::tiles;

    /// The chapter speaks exactly the script's 06_ lines, in order.
    #[test]
    fn the_chapter_speaks_the_scripts_lines_in_order() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let ids: Vec<&str> = script
            .lines()
            .iter()
            .map(|(id, _)| id.as_str())
            .filter(|id| id.starts_with("06_"))
            .collect();
        assert_eq!(ids, LINES);
    }

    /// What the chapter reads is the model's: the cubes, the sides, the sixth
    /// cube's 216 tiles and 21 on a side, its arm six by fifteen with two and
    /// a half slabs, the strips of seven and eight cubes, the runs of odd
    /// numbers and the four years.
    #[test]
    fn the_facts_are_the_models() {
        let r = reads().unwrap();
        assert_eq!(r.cubes, [1, 8, 27, 64, 125]);
        assert_eq!(fact_ints("F-K.sides").unwrap(), [1, 3, 6, 10, 15]);
        assert_eq!((r.side, r.sixth_cube, r.sixth_side), (15, 216, 21));
        assert_eq!((r.arm, r.arm_slabs.as_str()), ((6, 15), "5/2"));
        assert_eq!(r.strips[0], (7, 21, "3".to_owned()));
        assert_eq!(r.strips[1], (8, 28, "7/2".to_owned()));
        assert_eq!(r.runs.len(), 5);
        assert_eq!(r.runs[4], [21, 23, 25, 27, 29]);
        assert_eq!(r.years, [100, 1000, 1985]);
        // the totals row: each is the square of the side, and the sixth is 441
        let mut sum = 0;
        for (c, side) in r.cubes.iter().zip(fact_ints("F-K.sides").unwrap()) {
            sum += c;
            assert_eq!(sum, side * side);
        }
        assert_eq!(sum + r.sixth_cube, r.sixth_side * r.sixth_side);
        assert_eq!(r.sixth_side * r.sixth_side, 441);
    }

    /// An arm's pieces are the smaller cubes' sides laid end to end, and the
    /// pairing is the model's slabs: the pieces tile the arm before and after,
    /// every pair makes one slab, and the slabs of the pairing sit exactly
    /// where `cubes::layer` puts the arm's slabs and halves.
    #[test]
    fn the_arm_is_the_cubes_sides_and_the_pairing_is_the_models_slabs() {
        for k in 2..=12 {
            let arm_len = cubes::side(k - 1);
            for plan in [pieces_of_arm(k), paired_arm(k)] {
                let mut spans: Vec<(i64, i64)> = plan.iter().map(|&(_, a, b)| (a, b)).collect();
                spans.sort_unstable();
                let mut at = 0;
                for (a, b) in spans {
                    assert_eq!(a, at, "cube {k}: a gap or an overlap");
                    at = b;
                }
                assert_eq!(at, arm_len, "cube {k}: the pieces fill the arm");
                // each piece keeps its length
                for &(j, a, b) in &plan {
                    assert_eq!(b - a, j);
                }
            }
            // the slabs: whole ones of length k, a half of k/2 for an even cube
            let slabs = slabs_of_arm(k);
            assert_eq!(slabs.iter().map(|&(a, b)| b - a).sum::<i64>(), arm_len);
            assert_eq!(slabs.len() as i64, (k - 1) / 2 + i64::from(k % 2 == 0));
            // pairs: j and k − j close into one slab of length k
            let paired = paired_arm(k);
            for j in 1..=((k - 1) / 2) {
                let a = paired.iter().find(|p| p.0 == j).unwrap();
                let b = paired.iter().find(|p| p.0 == k - j).unwrap();
                assert_eq!(a.2, b.1, "{j} and {} are side by side", k - j);
                assert_eq!(b.2 - a.1, k, "and make one slab of {k}");
            }
            // against the model: the right arm's rectangles are the slabs
            let layer = cubes::layer(k);
            for (from, to) in slabs {
                let rect = arm_rect(Arm::Right, cubes::side(k - 1), k, from, to);
                assert!(layer.iter().any(|p| p.rect == rect), "cube {k}: slab {from}..{to}");
                let top = arm_rect(Arm::Top, cubes::side(k - 1), k, from, to);
                assert!(layer.iter().any(|p| p.rect == top), "cube {k}: top slab {from}..{to}");
            }
        }
    }

    /// The three orders of the arm's pieces are one pairing: the first is the
    /// sides end to end, each hop moves one piece, and the last is the model's
    /// pairing, piece by piece.
    #[test]
    fn two_hops_take_the_sides_to_the_paired_arm() {
        assert_eq!(ORDERS[0], [1, 2, 3, 4, 5]);
        for (n, hopper) in HOPPERS.iter().enumerate() {
            // removing the hopper from either order leaves the same order of the rest
            let rest = |o: &[i64; 5]| o.iter().copied().filter(|j| j != hopper).collect::<Vec<_>>();
            assert_eq!(rest(&ORDERS[n]), rest(&ORDERS[n + 1]), "only piece {hopper} hops");
            assert_ne!(ORDERS[n], ORDERS[n + 1]);
        }
        // the paired order puts each piece where the model's pairing does
        for &(j, from, _) in &paired_arm(SIXTH) {
            assert_eq!(start_in(&ORDERS[2], j), from, "piece {j}");
        }
        for o in ORDERS {
            let mut sorted = o;
            sorted.sort_unstable();
            assert_eq!(sorted, [1, 2, 3, 4, 5]);
            // every order fills the arm: its pieces are 1 + 2 + 3 + 4 + 5 long
            assert_eq!(o.iter().sum::<i64>(), cubes::side(FIVE));
        }
        // a hop's shifts are along the arm, and the hopper's way out and back
        // are the same slide, opposite
        for arm in [Arm::Right, Arm::Top] {
            let out = out_of(arm);
            assert!(out[0] * out[1] == 0.0 && out[0] + out[1] < 0.0);
            let d = along_arm(arm, 3);
            assert!(d[0] * d[1] == 0.0 && d[0] + d[1] > 0.0);
        }
    }

    /// The pictures tile: the five cubes' pieces fill the 15 by 15 square,
    /// the sixth layer round it fills the 21 by 21 exactly, and the two arms'
    /// pieces fill the L of cells the outline draws.
    #[test]
    fn the_pieces_tile_the_square_and_the_sixth_layer_closes_it() {
        let five = cubes::square(FIVE);
        tiles(five.iter().flat_map(|p| p.rect.cells()), Rect::new(0, 0, 15, 15), &[]).unwrap();
        let all = cubes::square(SIXTH);
        tiles(all.iter().flat_map(|p| p.rect.cells()), Rect::new(0, 0, 21, 21), &[]).unwrap();
        // the arms' pieces and the corner are the sixth layer's cells
        let mut cells: Vec<Cell> = Vec::new();
        for arm in [Arm::Right, Arm::Top] {
            for (_, a, b) in pieces_of_arm(SIXTH) {
                cells.extend(arm_rect(arm, 15, SIXTH, a, b).cells());
            }
        }
        cells.extend(Rect::new(15, 15, 6, 6).cells());
        let layer_cells: Vec<Cell> = cubes::layer(SIXTH).iter().flat_map(|p| p.rect.cells()).collect();
        let mut a = cells.clone();
        let mut b = layer_cells;
        a.sort_unstable();
        b.sort_unstable();
        assert_eq!(a, b);
        // the thin layers are chapter 03's L-shaped layers: layer m has 2m − 1
        // tiles, and the thin layers of a cube's band are its band exactly
        for c in 1..=FIVE {
            let band: i64 = (cubes::side(c - 1) + 1..=cubes::side(c))
                .map(|m| layer(m).len() as i64)
                .sum();
            assert_eq!(band, c * c * c, "the band of cube {c} has {} tiles", c * c * c);
        }
    }

    /// Where things stand: the 21 square is under the shelf's floor and the
    /// frame's left edge, the shrunk picture is above the card's top, the
    /// strips and the timeline are inside the frame, and a piece slid out
    /// over the square lands inside the old square.
    #[test]
    fn the_picture_the_strips_and_the_timeline_fit_the_frame() {
        let top = at(0.0, 21.0)[1];
        assert!(top <= super::super::draw::SHELF_FLOOR + 1e-9, "top {top}");
        assert!(at(0.0, 0.0)[0] > -FRAME_X_RADIUS + 0.4);
        // 06_4: the shrunk picture is above the card (top −1.6) and under the shelf
        let lo = CARD_AT[1];
        let hi = CARD_AT[1] + 21.0 * CELL * CARD_K;
        assert!(lo > -1.6 && hi <= super::super::draw::SHELF_FLOOR + 1e-9, "{lo} {hi}");
        assert!((CELL * CARD_K - 0.2).abs() < 1e-12);
        // the years put the four dots left to right in the frame
        let xs: Vec<f64> = [100.0, 1000.0, 1985.0]
            .iter()
            .map(|y| TIME_X + TIME_W * y / 2000.0)
            .collect();
        assert!(xs.windows(2).all(|w| w[1] > w[0] + 1.0) && xs[2] < FRAME_X_RADIUS - 0.3);
    }

    /// The shades are steps of the cube's colour: the first lightest, the last
    /// darkest, a single layer the colour itself; and a mixed number is
    /// written as the fact's fraction.
    #[test]
    fn shades_and_mixed_numbers() {
        let c = size(4);
        assert_eq!(step_shade(c, 0, 1), c);
        let (first, last) = (step_shade(c, 0, 4), step_shade(c, 3, 4));
        assert!(first.r > c.r && first.g > c.g && first.b > c.b);
        assert!(last.r < c.r && last.g < c.g && last.b < c.b);
        assert_eq!(mixed("5/2"), "2\\frac{1}{2}");
        assert_eq!(mixed("3"), "3");
        assert_eq!(mixed("7/2"), "3\\frac{1}{2}");
    }

    /// The scene constructs end to end (every phrase it waits for is in its
    /// line, every fact it reads is there, every string typesets), the layout
    /// lint of the kit finds nothing, and the chapter is inside the
    /// storyboard's budget.
    #[test]
    fn the_chapter_runs_and_the_layout_lint_is_clean() {
        let ran = testing::run("06_every_cube").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        assert!(ran.seconds > 40.0 && ran.seconds < 90.0, "{} s", ran.seconds);
    }
}
