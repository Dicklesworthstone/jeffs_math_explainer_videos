//! Chapter 09, every height: Siu's box at the next size. The block of four
//! levels, taken apart into its five slabs (09_1, the question: would five
//! fit?); each slab gains an L-shaped layer, the step gains a column, and one
//! more slab goes underneath, and the slabs close up into the block of five
//! levels (09_2); those gold pieces are three new slabs of five by five, one
//! under, one behind, one beside, nothing overlapping, as the L-shaped layer
//! is what the flat square gains (09_3); the first case, three single cubes in
//! a box one by two by one and a half, and a chain of blocks, each growing
//! into the next (09_4); and the card, with the three turns on the ring
//! (09_5).
//!
//! The solids are `model::stacks` cube for cube: the block of four levels by
//! slab and stack, and the pieces it gains (`stacks::new_base`) by the level
//! of the new block they stand at. The scene fails if the old pieces, lifted
//! one cube, and the new pieces are not exactly the three stacks of five
//! levels where the model puts them. The slabs are taken apart straight up
//! (a gap between levels) so that every new cube is seen, which a lift and a
//! slide from one viewpoint hides.

use explainer_kit::{Kit, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    Board, CardLook, CardRow, LENS_AT, LENS_R, ProofCard, THUMB, Words, breath, dim, fact, fact_int, fade_in_all,
    fade_out_all, fail, flash, hero, ink, lens, lens_mark, lower_third, pause, question_card, question_pause, say,
    shelf_slot, tex_row,
};
use super::iso::{Anchor, Block, Cell, Iso, P3, Solid, ViewAt};
use super::kinetics::{Moves, arrive, leave, play, play_with, prepared};
use super::siu::{colour, edge_brace, level, placed};
use crate::facts::shown;
use crate::model::{layers, stacks};

pub struct EveryHeight {
    pub kit: Kit,
}

/// The narration lines this chapter speaks, in order.
pub const LINES: [&str; 5] = ["09_1", "09_2", "09_3", "09_4", "09_5"];

// ------------------------------------------------------------------ choices

/// How much air there is between two levels while the block is apart, in
/// cubes.
pub const GAP: f64 = 0.6;
/// How far right (lattice x) the base slab waits before it slides in.
pub const WAIT: P3 = [7.0, 4.2, 0.0];
/// The view while the block is apart: a block of five levels with six
/// slabs and their gaps, standing under the shelf.
pub const APART: ViewAt = ViewAt::classic((-5.6, -2.3), 0.5);
/// The block of five levels closed up, in the left half, with its labels'
/// and the inset's room beside it.
pub const SETTLED_AT: (f64, f64) = (-4.0, 0.4);
pub const SETTLED_ROOM: (f64, f64) = (4.6, 3.8);
/// The same block small in the top left corner, while the chain and the
/// card are on the screen, and then on the shelf.
pub const CORNER_AT: (f64, f64) = (-5.0, 1.2);
pub const CORNER_ROOM: (f64, f64) = (3.4, 2.6);
/// Where the question stands in 09_1.
pub const QUESTION_AT: Vec3 = [1.8, 0.9, 0.0];
/// The inset of 09_3: the flat square and its L, and the three slabs.
pub const FLAT_AT: Vec3 = [1.0, 0.7, 0.0];
pub const FLAT_CELL: f64 = 0.34;
pub const SLABS_AT: Vec3 = [0.9, -1.9, 0.0];
pub const SLAB_CELL: f64 = 0.26;
pub const SLAB_GAP: f64 = 0.6;
/// The chain of 09_4: its centre and the room it has; the base case's
/// view; the gap between two blocks of the chain, in units of the chain's
/// scale.
pub const CHAIN_AT: (f64, f64) = (1.8, 1.5);
pub const CHAIN_ROOM: (f64, f64) = (9.0, 2.5);
pub const CHAIN_GAP: f64 = 1.0;
pub const BASE_VIEW: ViewAt = ViewAt::classic((-2.9, -2.1), 0.8);
/// Where the three turns wait in the base case's lattice, far off its box.
pub const RING_AT: P3 = [30.0, 0.0, 0.0];
/// The card, a little higher and smaller than its home so five lines fit.
pub const CARD_LOOK: CardLook = CardLook {
    x: 0.0,
    top: -0.9,
    scale: 0.78,
};
/// The shelf's slot this proof takes: after chapter 06's.
pub const SLOT: usize = 3;
/// Where the staircase's count stands in 09_2, left edge.
pub const STAIRCASE_AT: Vec3 = [-1.2, 1.0, 0.0];
/// The type size of the numbers beside the slabs.
const LABEL_PT: f64 = 30.0;

// the chapter's fixed words
const UNDER: &str = "under";
const BEHIND: &str = "behind";
const BESIDE: &str = "beside";
const CUBES_WORD: &str = "cubes";
const STAIRCASE_WORDS: &str = "staircase";
const LEVELS_WORD: &str = "levels";
const INDUCTION: &str = "mathematical induction:";
const INDUCTION_TEXT: &str =
    "the first case, and each case leads to the next. Not guessing from cases, like the circle";
const SHOWN_TEX: &str = r"\text{for every whole number } n \text{ from 1, three stacks of } n \text{ levels, the top step sliced in half and turned, fill a box } n \times (n + 1) \times \left(n + \frac{1}{2}\right)";
const ASSUMED_TURNED: &str = "each stack is the first one, turned";
const ASSUMED_TOP: &str = "the step and its turned copy fill the top";
const REASON_TOP: &str = "two staircases make a rectangle";
const ASSUMED_NEXT: &str = "the next height fits";
const REASON_NEXT: &str = "three new slabs";

// ---------------------------------------------------------------- the model

/// A piece of one stack at one level of a block, as a solid on stage.
#[derive(Clone, Copy, Debug)]
pub struct Piece {
    pub level: i64,
    pub which: usize,
    pub solid: Solid,
}

/// What the chapter reads from the facts, checked against the model.
pub struct Reads {
    pub n: i64,
    pub next: i64,
    /// The block of the next height: wide, deep and tall in whole cubes
    /// (the staircase is on top of that), and the staircase's cubes.
    pub grown_box: [i64; 3],
    pub grown_top: i64,
    pub grown_slab: i64,
    pub one_cubes: i64,
    /// The first case's box as words for its three sides (`1`, `2`, `1½`).
    pub one_box: [String; 3],
}

/// `a/2` as `{a/2}½` (`3/2` is `1½`), a whole number as it is.
pub fn mixed(s: &str) -> String {
    match s.split_once('/') {
        Some((a, "2")) => match a.parse::<i64>() {
            Ok(a) => format!("{}½", a / 2),
            Err(_) => s.to_owned(),
        },
        _ => s.to_owned(),
    }
}

fn want(key: &str, got: i64, model: i64) -> fmn::Result<i64> {
    if got == model {
        Ok(got)
    } else {
        Err(fail(format!("{key} is {got} but the model has {model}")))
    }
}

fn reads() -> fmn::Result<Reads> {
    let n = shown::STACK;
    let next = n + 1;
    let grown = fact("F-S.grown.box")?;
    let want_box = format!("{next}x{}x{next}", next + 1);
    if grown != want_box {
        return Err(fail(format!(
            "F-S.grown.box is {grown:?}, the model's block {want_box:?}"
        )));
    }
    // the step on top of the grown block: the old step and a column
    let grown_top = want(
        "F-S.grown.top",
        fact_int("F-S.grown.top")?,
        stacks::top(n).len() as i64 + next,
    )?;
    let grown_slab = want(
        "F-S.grown.slab",
        fact_int("F-S.grown.slab")?,
        stacks::new_base(next, 0).len() as i64,
    )?;
    let one_cubes = want(
        "F-S.one.cubes",
        fact_int("F-S.one.cubes")?,
        (0..3).map(|w| stacks::placed(1, w).len() as i64).sum(),
    )?;
    let one = fact("F-S.one.box")?;
    let sides: Vec<&str> = one.split('x').collect();
    let [a, b, c] = sides.as_slice() else {
        return Err(fail(format!("F-S.one.box = {one:?} is not three sizes")));
    };
    Ok(Reads {
        n,
        next,
        grown_box: [next, next + 1, next],
        grown_top,
        grown_slab,
        one_cubes,
        one_box: [mixed(a), mixed(b), mixed(c)],
    })
}

/// The cells of `which`'s new base slab at level `j` of the new block.
pub fn new_at(next: i64, which: usize, j: i64) -> Vec<Cell> {
    stacks::new_base(next, which).into_iter().filter(|c| c.2 == j).collect()
}

/// The half cubes of the first case's box, by stack: each whole cube is two
/// halves one over the other, except the top step's, whose upper half is
/// turned into the empty half (`stacks::turned_half`). Cells in half-height
/// units.
pub fn first_case(which: usize) -> Vec<Cell> {
    let n = 1;
    let mut out = Vec::new();
    for (x, y, z) in placed(n, which) {
        if z < n {
            out.extend([(x, y, 2 * z), (x, y, 2 * z + 1)]);
        } else {
            let (tx, ty) = stacks::turned_half(n, (x, y));
            out.extend([(x, y, 2 * n), (tx, ty, 2 * n)]);
        }
    }
    out
}

/// The half-cube boxes of a cell list in half-height units.
fn halves(cells: &[Cell]) -> Vec<Block> {
    cells.iter().map(|&c| Block::sized(c, [1.0, 1.0, 0.5])).collect()
}

/// Where the blocks of the chain stand, as lattice points along the line
/// that runs level across the screen (a step of 5 in x and 3 in y is a step
/// to the right and none up), each a box `n` wide, `n + 1` deep and
/// `n + ½` tall, `gap` of the chain's unit apart on screen. Returns the
/// points and the lattice length the chain spans along that line.
pub fn chain_points(top: i64, gap: f64) -> (Vec<P3>, f64) {
    // a step t along the line moves a point right on screen by 5.83 t (of the
    // chain's scale); a block n lattice units wide is about 1.357 n + 0.48
    let per_t = 5.0 * ViewAt::YAW.cos() + 3.0 * ViewAt::YAW.sin();
    let wide = |m: i64| 1.357 * m as f64 + 0.48;
    let mut at = Vec::new();
    let mut x = 0.0;
    for m in 1..=top {
        at.push([5.0 * x / per_t, 3.0 * x / per_t, 0.0]);
        x += wide(m) + gap;
    }
    (at, x - gap)
}

// ----------------------------------------------------------- the drawings

/// Moves of the cube kit and arrivals together, started `lag` apart.
fn with(stage: &mut Stage<'_>, moves: Vec<AnimBuilder>, extra: Moves, lag: f64) -> fmn::Result<()> {
    let mut all = prepared(stage, moves)?;
    all.extend(extra);
    play(stage, all, lag)
}

/// The words `s` in `colour`, pinned to a point of the lattice, `offset`
/// from it, hidden until it arrives.
fn pinned(kit: &Kit, stage: &mut Stage<'_>, iso: &Iso, s: &str, at: P3, offset: Vec3, color: Srgb) -> fmn::Result<Mob> {
    let m = kit.text(stage, s, LABEL_PT, color, ORIGIN)?;
    iso.pin(stage, m, Anchor::World(at), offset)?;
    Ok(m)
}

/// A flat board of `side` by `side` cells in `colour`, with its cells in
/// `lit` in `hero`, one mobject each. Detached.
fn flat_square(
    stage: &mut Stage<'_>,
    board: &Board,
    side: i64,
    lit: &[(i64, i64)],
    colour_of: Srgb,
) -> fmn::Result<(Mob, Option<Mob>)> {
    let cells: Vec<(i64, i64)> = (0..side)
        .flat_map(|x| (0..side).map(move |y| (x, y)))
        .filter(|c| !lit.contains(c))
        .collect();
    let rest = board.tiles(stage, &cells, colour_of)?;
    let lit_mob = if lit.is_empty() {
        None
    } else {
        Some(board.tiles(stage, lit, hero())?)
    };
    Ok((rest, lit_mob))
}

impl SceneConstruct for EveryHeight {
    fn name(&self) -> &str {
        "09_every_height"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let r = reads()?;
        let (n, next) = (r.n, r.next);
        let iso = Iso::new(stage, APART)?;

        // the block of n levels, one solid a slab and a stack, taken apart
        // straight up: slab z stands at z (1 + GAP)
        let mut old: Vec<Piece> = Vec::new();
        for z in 0..=n {
            for which in 0..3 {
                let cells = level(n, which, z);
                if cells.is_empty() {
                    continue;
                }
                let at = [0.0, 0.0, z as f64 * GAP];
                let solid = iso.solid(stage, &cells, colour(which), at)?;
                old.push(Piece { level: z, which, solid });
            }
        }
        // what the next height adds, by the level of the new block it stands
        // at: level j beside the old slab j - 1; level 0 waits at the right
        let mut fresh: Vec<Piece> = Vec::new();
        for which in 0..3 {
            for j in 0..=next {
                let cells = new_at(next, which, j);
                if cells.is_empty() {
                    continue;
                }
                let at = if j == 0 {
                    WAIT
                } else {
                    [0.0, 0.0, (j - 1) as f64 * GAP - 1.0]
                };
                let solid = iso.solid(stage, &cells, colour(which), at)?;
                fresh.push(Piece { level: j, which, solid });
            }
        }

        // ---------------------------------------------------------- 09_1
        let line = say(&kit, stage, LINES[0])?;
        let mut shown_old = Vec::new();
        for p in &old {
            shown_old.push(iso.fade(p.solid, 1.0, 0.8)?);
        }
        iso.play(stage, shown_old, 0.0)?;
        line.wait_last(stage)?;
        let words = line
            .sentence_texts()
            .pop()
            .ok_or_else(|| fail("09_1 has no sentences".into()))?;
        let question = question_card(&kit, stage, &words, QUESTION_AT)?;
        play_with(stage, 0.0, |st| Ok(vec![arrive(st, question, 0.5)?]))?;
        question_pause(&kit, stage)?;
        play_with(stage, 0.0, |st| Ok(vec![leave(st, question, 0.4)?]))?;

        // ---------------------------------------------------------- 09_2
        let line = say(&kit, stage, LINES[1])?;
        // an L round each full slab, the first with its arms counted
        let right_arm = new_at(next, 2, 1).len() as i64;
        let corner = new_at(next, 1, 1)
            .iter()
            .filter(|c| (c.0, c.1) == (next - 1, next))
            .count() as i64;
        let back_arm = new_at(next, 1, 1).len() as i64 - corner;
        let top_of_first = 1.0;
        let arms = [
            (
                right_arm,
                [next as f64 - 0.5, (next as f64) / 2.0, top_of_first],
                v(0.4, 0.0),
            ),
            (
                back_arm,
                [(next as f64 - 1.0) / 2.0, next as f64 + 0.5, top_of_first],
                v(0.0, 0.32),
            ),
            (
                corner,
                [next as f64 - 0.5, next as f64 + 0.5, top_of_first],
                v(0.35, 0.3),
            ),
        ];
        let mut counts: Vec<Mob> = Vec::new();
        for (count, at, offset) in arms {
            counts.push(pinned(&kit, stage, &iso, &count.to_string(), at, offset, hero())?);
        }
        line.wait_sentence(stage, 1)?;
        for j in 1..=n {
            let mut moves = Vec::new();
            for p in fresh.iter().filter(|p| p.level == j) {
                moves.push(iso.fade(p.solid, 1.0, 0.45)?);
                moves.push(iso.glow(p.solid, hero(), 1.0, 0.45)?);
            }
            if j == 1 {
                let extra: Moves = counts
                    .iter()
                    .map(|&c| arrive(stage, c, 0.45))
                    .collect::<fmn::Result<Moves>>()?;
                with(stage, moves, extra, 0.0)?;
                pause(stage, 0.6)?;
            } else {
                iso.play(stage, moves, 0.0)?;
            }
        }
        // the counts have said their piece
        let gone: Moves = counts
            .iter()
            .map(|&c| leave(stage, c, 0.3))
            .collect::<fmn::Result<Moves>>()?;
        for &c in &counts {
            iso.unpin(c);
        }
        play(stage, gone, 0.0)?;
        // the step gains a column
        line.wait_sentence(stage, 2)?;
        let mut moves = Vec::new();
        for p in fresh.iter().filter(|p| p.level == next) {
            moves.push(iso.fade(p.solid, 1.0, 0.5)?);
            moves.push(iso.glow(p.solid, hero(), 1.0, 0.5)?);
        }
        iso.play(stage, moves, 0.0)?;
        // one more slab goes underneath: everything makes room, the base
        // slides in from the right, and the slabs close up
        line.wait_sentence(stage, 3)?;
        let lift = 1.0 + GAP;
        let mut up = Vec::new();
        for p in &old {
            up.push(iso.slide_by(stage, p.solid, [0.0, 0.0, lift], 0.55)?);
        }
        for p in fresh.iter().filter(|p| p.level > 0) {
            up.push(iso.slide_by(stage, p.solid, [0.0, 0.0, lift], 0.55)?);
        }
        iso.play(stage, up, 0.0)?;
        let mut base = Vec::new();
        // the slab is gold, its far row of five already in the copper of the
        // stack behind
        for p in fresh.iter().filter(|p| p.level == 0) {
            base.push(iso.fade(p.solid, 1.0, 0.3)?);
            if p.which == 0 {
                base.push(iso.glow(p.solid, hero(), 1.0, 0.3)?);
            }
            base.push(iso.slide(stage, p.solid, [0.0, 0.0, 0.0], 0.8)?);
        }
        iso.play(stage, base, 0.0)?;
        let mut close = Vec::new();
        for p in &old {
            close.push(iso.slide_by(stage, p.solid, [0.0, 0.0, -((p.level + 1) as f64) * GAP], 0.7)?);
        }
        for p in fresh.iter().filter(|p| p.level > 0) {
            close.push(iso.slide_by(stage, p.solid, [0.0, 0.0, -(p.level as f64) * GAP], 0.7)?);
            close.push(iso.glow(p.solid, hero(), 0.0, 0.7)?);
        }
        for p in fresh.iter().filter(|p| p.level == 0 && p.which == 0) {
            close.push(iso.glow(p.solid, hero(), 0.0, 0.7)?);
        }
        iso.play(stage, close, 0.0)?;
        // the model's cells, stack by stack
        for which in 0..3 {
            let mut got: Vec<Cell> = Vec::new();
            for p in old.iter().chain(&fresh).filter(|p| p.which == which) {
                let cells = iso
                    .cells(stage, p.solid)
                    .ok_or_else(|| fail(format!("a piece of stack {which} is off the lattice")))?;
                got.extend(cells);
            }
            got.sort_unstable();
            let mut want_cells = placed(next, which);
            want_cells.sort_unstable();
            if got != want_cells {
                return Err(fail(format!("stack {which} did not end on the model's cells")));
            }
        }
        // the block of the next height: braces 5, 6, 5 and its staircase
        let [bw, bd, bh] = r.grown_box.map(|x| x as f64);
        let edges = [
            (
                iso.project(stage, [0.0, 0.0, 0.0]),
                iso.project(stage, [bw, 0.0, 0.0]),
                v(0.0, -1.0),
                r.grown_box[0],
            ),
            (
                iso.project(stage, [bw, 0.0, 0.0]),
                iso.project(stage, [bw, bd, 0.0]),
                v(1.0, -0.6),
                r.grown_box[1],
            ),
            (
                iso.project(stage, [0.0, 0.0, bh]),
                iso.project(stage, [0.0, 0.0, 0.0]),
                v(-1.0, 0.0),
                r.grown_box[2],
            ),
        ];
        let mut braces: Vec<Mob> = Vec::new();
        for (a, b, out, size_of_edge) in edges {
            let (brace, reach) = edge_brace(stage, a, b, out, ink())?;
            let mid = v((a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0);
            let len = (out[0] * out[0] + out[1] * out[1]).sqrt();
            let label = kit.text(
                stage,
                &size_of_edge.to_string(),
                36.0,
                ink(),
                v(
                    mid[0] + (reach + 0.3) * out[0] / len,
                    mid[1] + (reach + 0.3) * out[1] / len,
                ),
            )?;
            braces.extend([brace, label]);
        }
        let staircase = kit.text_left(
            stage,
            &format!("{STAIRCASE_WORDS}: {} {CUBES_WORD}", r.grown_top),
            30.0,
            ink(),
            v(STAIRCASE_AT[0], STAIRCASE_AT[1]),
        )?;
        braces.push(staircase);
        let braces_in: Moves = braces
            .iter()
            .map(|&m| arrive(stage, m, 0.5))
            .collect::<fmn::Result<Moves>>()?;
        play(stage, braces_in, 0.25)?;
        line.wait_end(stage)?;
        breath(&kit, stage)?;

        // ---------------------------------------------------------- 09_3
        let line = say(&kit, stage, LINES[2])?;
        // the old block steps back and the three new slabs stand out; the
        // block settles to the left half
        let settled = APART.fitted(
            [0.0; 3],
            [next as f64, (next + 1) as f64, (next + 1) as f64],
            SETTLED_AT,
            SETTLED_ROOM,
        );
        let mut moves = iso.view_to(settled, 0.8)?;
        for p in &old {
            moves.push(iso.glow(p.solid, dim(), 0.85, 0.6)?);
        }
        let braces_out: Moves = braces
            .iter()
            .map(|&m| leave(stage, m, 0.4))
            .collect::<fmn::Result<Moves>>()?;
        with(stage, moves, braces_out, 0.0)?;
        // the inset: the three slabs flat, each a five by five board
        let mut slab_mobs: Vec<Mob> = Vec::new();
        let mut slab_names: Vec<Mob> = Vec::new();
        for (which, word) in [UNDER, BEHIND, BESIDE].iter().enumerate() {
            let x = SLABS_AT[0] + which as f64 * (next as f64 * SLAB_CELL + SLAB_GAP);
            let board = Board::new([x, SLABS_AT[1], 0.0], SLAB_CELL);
            let (m, _) = flat_square(stage, &board, next, &[], colour(which))?;
            slab_mobs.push(m);
            slab_names.push(kit.text(
                stage,
                word,
                24.0,
                colour(which),
                v(x + next as f64 * SLAB_CELL / 2.0, SLABS_AT[1] - 0.35),
            )?);
        }
        let slab_total = kit.text(
            stage,
            &format!("{0} + {0} + {0} = {1}", r.grown_slab, 3 * r.grown_slab),
            34.0,
            ink(),
            v(SLABS_AT[0] + 2.4, SLABS_AT[1] - 1.05),
        )?;
        let mut inset: Vec<Mob> = slab_mobs.clone();
        inset.extend(&slab_names);
        inset.push(slab_total);
        line.wait_for(stage, "gold pieces")?;
        let arrivals: Moves = inset
            .iter()
            .map(|&m| arrive(stage, m, 0.5))
            .collect::<fmn::Result<Moves>>()?;
        play(stage, arrivals, 0.15)?;
        // the new slabs pulse, in gold and back
        let mut pulse_up = Vec::new();
        let mut pulse_down = Vec::new();
        for p in &fresh {
            pulse_up.push(iso.glow(p.solid, hero(), 1.0, 0.35)?);
            pulse_down.push(iso.glow(p.solid, hero(), 0.0, 0.5)?);
        }
        iso.play(stage, pulse_up, 0.0)?;
        iso.play(stage, pulse_down, 0.0)?;
        // the flat square gaining its L
        line.wait_for(stage, "Like the L-shaped layer")?;
        let board = Board::new(FLAT_AT, FLAT_CELL);
        let side = n + 1;
        let l_cells = layers::layer(side);
        let (square, l_mob) = flat_square(stage, &board, side, &l_cells, dim())?;
        let l_mob = l_mob.ok_or_else(|| fail("the L-shaped layer has no cells".into()))?;
        let square_sum = tex_row(
            &kit,
            stage,
            &[
                (&format!("{}", n * n), dim()),
                ("+", ink()),
                (&format!("{}", l_cells.len()), hero()),
                ("=", ink()),
                (&format!("{}", side * side), ink()),
            ],
            40.0,
            v(
                FLAT_AT[0] + 2.0 * (side as f64 * FLAT_CELL) + 0.2,
                FLAT_AT[1] + side as f64 * FLAT_CELL / 2.0,
            ),
            0.2,
            2.4,
        )?;
        let mut flat_in: Vec<Mob> = vec![square, l_mob];
        flat_in.extend(&square_sum);
        let arrivals: Moves = flat_in
            .iter()
            .map(|&m| arrive(stage, m, 0.5))
            .collect::<fmn::Result<Moves>>()?;
        play(stage, arrivals, 0.0)?;
        // the two growths pulse together
        let mut up = Vec::new();
        for p in &fresh {
            up.push(iso.glow(p.solid, hero(), 1.0, 0.35)?);
        }
        iso.play(stage, up, 0.0)?;
        flash(stage, &[l_mob], hero(), 1.25, 0.5)?;
        let mut down = Vec::new();
        for p in &fresh {
            down.push(iso.glow(p.solid, hero(), 0.0, 0.5)?);
        }
        iso.play(stage, down, 0.0)?;
        line.wait_end(stage)?;

        // ---------------------------------------------------------- 09_4
        let line = say(&kit, stage, LINES[3])?;
        // the block goes to the corner; the inset goes; the chain comes
        let corner_view = APART.fitted(
            [0.0; 3],
            [next as f64, (next + 1) as f64, (next + 1) as f64],
            CORNER_AT,
            CORNER_ROOM,
        );
        let mut moves = iso.view_to(corner_view, 0.8)?;
        for p in &fresh {
            moves.push(iso.glow(p.solid, hero(), 0.0, 0.1)?);
        }
        for p in &old {
            moves.push(iso.glow(p.solid, dim(), 0.0, 0.6)?);
        }
        let mut leaving: Vec<Mob> = inset.clone();
        leaving.extend(&flat_in);
        let outs: Moves = leaving
            .iter()
            .map(|&m| leave(stage, m, 0.4))
            .collect::<fmn::Result<Moves>>()?;
        with(stage, moves, outs, 0.0)?;
        // the chain: block n is n wide, n + 1 deep and n + 1/2 tall
        let (points, _) = chain_points(next, CHAIN_GAP);
        let hi = [
            points.last().map_or(0.0, |p| p[0]) + next as f64,
            points.last().map_or(0.0, |p| p[1]) + (next + 1) as f64,
            next as f64 + 0.5,
        ];
        let chain_view = ViewAt::classic((0.0, 0.0), 1.0).fitted([0.0; 3], hi, CHAIN_AT, CHAIN_ROOM);
        let chain = Iso::new(stage, chain_view)?;
        let mut boxes: Vec<Solid> = Vec::new();
        for (i, at) in points.iter().enumerate() {
            let m = (i + 1) as f64;
            boxes.push(chain.block(stage, [m, m + 1.0, m + 0.5], dim(), *at)?);
        }
        let mut level_labels: Vec<Mob> = Vec::new();
        for (i, at) in points.iter().enumerate() {
            let m = (i + 1) as f64;
            let label = kit.text(stage, &format!("{}", i + 1), 26.0, dim(), ORIGIN)?;
            chain.pin(
                stage,
                label,
                Anchor::World([at[0] + m / 2.0, at[1], 0.0]),
                [0.0, -0.3, 0.0],
            )?;
            level_labels.push(label);
        }
        let levels_caption = kit.text(
            stage,
            LEVELS_WORD,
            24.0,
            dim(),
            v(CHAIN_AT.0, CHAIN_AT.1 - CHAIN_ROOM.1 / 2.0 - 0.55),
        )?;
        // the first case, big: the three cubes as a box
        let base = Iso::new(stage, BASE_VIEW)?;
        let mut case: Vec<Solid> = Vec::new();
        for which in 0..3 {
            let cells = first_case(which);
            case.push(base.solid_of(stage, &halves(&cells), colour(which), [0.0; 3], true)?);
        }
        let box_words = r.one_box.clone();
        let (bw, bd, bh) = (1.0, 2.0, 1.5);
        let corners = |p: P3| base.project(stage, p);
        let braces_src = [
            (
                corners([0.0, 0.0, 0.0]),
                corners([bw, 0.0, 0.0]),
                v(0.0, -1.0),
                &box_words[0],
            ),
            (
                corners([bw, 0.0, 0.0]),
                corners([bw, bd, 0.0]),
                v(1.0, -0.6),
                &box_words[1],
            ),
            (
                corners([0.0, 0.0, bh]),
                corners([0.0, 0.0, 0.0]),
                v(-1.0, 0.0),
                &box_words[2],
            ),
        ];
        let mut braces: Vec<Mob> = Vec::new();
        for (a, b, out, word) in braces_src {
            let (brace, reach) = edge_brace(stage, a, b, out, ink())?;
            let mid = v((a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0);
            let len = (out[0] * out[0] + out[1] * out[1]).sqrt();
            let label = kit.text(
                stage,
                word,
                32.0,
                ink(),
                v(
                    mid[0] + (reach + 0.3) * out[0] / len,
                    mid[1] + (reach + 0.3) * out[1] / len,
                ),
            )?;
            braces.extend([brace, label]);
        }
        let cubes_words = kit.text(
            stage,
            &format!("{} {CUBES_WORD}", r.one_cubes),
            30.0,
            hero(),
            v(-2.4, 0.6),
        )?;
        // the chain grows into each next block, lagged
        let mut grow = Vec::new();
        for &b in &boxes {
            grow.push(chain.fade(b, 1.0, 0.5)?);
        }
        let label_in: Moves = level_labels
            .iter()
            .chain([&levels_caption])
            .map(|&m| arrive(stage, m, 0.5))
            .collect::<fmn::Result<Moves>>()?;
        let mut all = prepared(stage, grow)?;
        all.extend(label_in);
        play(stage, all, 0.35)?;
        // height one: three cubes make the box one by two by one and a half
        line.wait_for(stage, "And at height one")?;
        let mut moves = Vec::new();
        for &s in &case {
            moves.push(base.fade(s, 1.0, 0.6)?);
        }
        let mut extra: Moves = Vec::new();
        for &m in braces.iter().chain([&cubes_words]) {
            extra.push(arrive(stage, m, 0.5)?);
        }
        let mut all = prepared(stage, moves)?;
        all.extend(extra);
        play(stage, all, 0.0)?;
        // that covers every height
        line.wait_for(stage, "That covers every height")?;
        let induction = lower_third(&kit, stage, INDUCTION, INDUCTION_TEXT)?;
        play_with(stage, 0.0, |st| Ok(vec![arrive(st, induction, 0.5)?]))?;
        line.wait_end(stage)?;
        pause(stage, 0.8)?;

        // ---------------------------------------------------------- 09_5
        let line = say(&kit, stage, LINES[4])?;
        // the chain and the base case go; the card comes with the three
        // turns on the ring
        let mut away = vec![chain.fade_all(0.0, 0.5)?];
        for &s in &case {
            away.push(base.fade(s, 0.0, 0.5)?);
        }
        let mut outs: Moves = Vec::new();
        for &m in level_labels
            .iter()
            .chain(&braces)
            .chain([&levels_caption, &cubes_words, &induction])
        {
            chain.unpin(m);
            outs.push(leave(stage, m, 0.4)?);
        }
        with(stage, away, outs, 0.0)?;
        let mut turns: Vec<Solid> = Vec::new();
        for which in 0..3 {
            turns.push(base.solid(stage, &placed(2, which), colour(which), RING_AT)?);
        }
        let ring_view = BASE_VIEW.fitted(
            RING_AT,
            [RING_AT[0] + 2.0, RING_AT[1] + 3.0, RING_AT[2] + 3.0],
            (LENS_AT[0], LENS_AT[1]),
            (1.5 * LENS_R, 1.5 * LENS_R),
        );
        base.set_view(stage, ring_view)?;
        let one_cubes = r.one_cubes;
        let mut proof = ProofCard::build_at(
            &kit,
            stage,
            &[
                CardRow::Shown(Words::Tex(SHOWN_TEX)),
                CardRow::Assumed(Words::Text(ASSUMED_TURNED)),
                CardRow::Assumed(Words::Text(ASSUMED_TOP)),
                CardRow::Assumed(Words::Text(&format!(
                    "height one is {one_cubes} cubes in a {} by {} by {} box",
                    r.one_box[0], r.one_box[1], r.one_box[2]
                ))),
                CardRow::Assumed(Words::Text(ASSUMED_NEXT)),
            ],
            CARD_LOOK,
        )?;
        line.wait_for(stage, "passes the card")?;
        proof.show(stage, 0.6)?;
        let ring = lens(stage);
        fade_in_all(stage, &[ring], 0.4)?;
        let mut moves = Vec::new();
        for &s in &turns {
            moves.push(base.fade(s, 1.0, 0.5)?);
        }
        iso.play(stage, moves, 0.0)?;
        proof.tick(&kit, stage, 1, "")?;
        let tick = lens_mark(stage, v(LENS_AT[0], LENS_AT[1]), 0.7, true);
        fade_in_all(stage, &[tick], 0.3)?;
        proof.tick(&kit, stage, 2, REASON_TOP)?;
        proof.tick(&kit, stage, 3, "")?;
        proof.tick(&kit, stage, 4, REASON_NEXT)?;
        line.wait_end(stage)?;
        // the card holds; then the picture goes to the shelf
        breath(&kit, stage)?;
        let mut done = proof.mobs();
        done.extend([ring, tick]);
        fade_out_all(stage, &done, 0.5)?;
        let mut away = Vec::new();
        for &s in &turns {
            away.push(base.fade(s, 0.0, 0.4)?);
        }
        let slot = shelf_slot(SLOT);
        let thumb_view = APART.fitted(
            [0.0; 3],
            [next as f64, (next + 1) as f64, (next + 1) as f64],
            (slot[0], slot[1]),
            (THUMB, THUMB),
        );
        away.extend(iso.view_to(thumb_view, 1.0)?);
        iso.play(stage, away, 0.0)?;

        kit.hold(stage)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::{SHELF_FLOOR, testing};
    use super::*;
    use crate::model::stacks;

    /// The pieces the chapter builds are exactly what the model says: the
    /// block of four levels lifted one cube, and three new slabs, are the
    /// three stacks of five levels, stack by stack, and no cube is in two
    /// pieces.
    #[test]
    fn the_old_block_lifted_and_three_new_slabs_are_the_next_height() {
        let (n, next) = (shown::STACK, shown::STACK + 1);
        for which in 0..3 {
            let mut got: Vec<Cell> = (0..=n)
                .flat_map(|z| level(n, which, z))
                .map(|(x, y, z)| (x, y, z + 1))
                .collect();
            for j in 0..=next {
                got.extend(new_at(next, which, j));
            }
            let mut want = placed(next, which);
            want.sort_unstable();
            got.sort_unstable();
            assert_eq!(got, want, "stack {which}");
        }
        // the three new slabs are flat boards of next by next
        let r = reads().unwrap();
        for which in 0..3 {
            assert_eq!(stacks::new_base(next, which).len() as i64, r.grown_slab);
        }
        assert_eq!(r.grown_slab, next * next);
        // the new cells by level: the base is a five by six board, the four
        // lifted slabs each gain an L of ten, the step a column of five
        let count = |j: i64| (0..3).map(|w| new_at(next, w, j).len() as i64).sum::<i64>();
        assert_eq!(count(0), next * (next + 1));
        for j in 1..=n {
            assert_eq!(count(j), 2 * next);
        }
        assert_eq!(count(next), next);
        assert_eq!((0..=next).map(count).sum::<i64>(), 3 * r.grown_slab);
        // the whole is the old block (90) and the new (75): the five levels
        // of three stacks of the next height
        let all: i64 = (0..3).map(|w| placed(next, w).len() as i64).sum();
        let old: i64 = (0..3).map(|w| placed(n, w).len() as i64).sum();
        assert_eq!(all - old, 3 * r.grown_slab);
        let grown_top = fact_int("F-S.grown.top").unwrap();
        assert_eq!(grown_top, stacks::top(next).len() as i64);
        assert_eq!(grown_top, stacks::top(n).len() as i64 + next);
    }

    /// The L round each lifted slab is the right arm of the beside slab and
    /// the back arm and corner of the behind slab: five and four and one,
    /// and with the old slab's four by five it is a five by six board.
    #[test]
    fn each_slab_gains_an_l_of_two_arms_and_a_corner() {
        let (n, next) = (shown::STACK, shown::STACK + 1);
        for j in 1..=n {
            let beside = new_at(next, 2, j);
            let behind = new_at(next, 1, j);
            assert!(new_at(next, 0, j).is_empty(), "nothing of the first stack at level {j}");
            // the right arm: a column at x = next - 1 across the old depth
            assert_eq!(beside.len() as i64, next);
            assert!(beside.iter().all(|c| c.0 == next - 1 && (0..next).contains(&c.1)));
            // the back arm and the corner: a row at y = next across the new width
            assert_eq!(behind.len() as i64, next);
            assert!(behind.iter().all(|c| c.1 == next && (0..next).contains(&c.0)));
            assert!(
                behind.iter().any(|c| (c.0, c.1) == (next - 1, next)),
                "the corner is in the row"
            );
            // the old slab and the L tile the new board
            let old_slab: Vec<(i64, i64)> = (0..3).flat_map(|w| level(n, w, j - 1)).map(|c| (c.0, c.1)).collect();
            let mut board: Vec<(i64, i64)> = old_slab;
            board.extend(beside.iter().chain(&behind).map(|c| (c.0, c.1)));
            board.sort_unstable();
            let before = board.len();
            board.dedup();
            assert_eq!(before, board.len(), "no cell twice");
            assert_eq!(board.len() as i64, next * (next + 1));
        }
    }

    /// The first case: three single cubes are a box one by two by one and a
    /// half, as the model's half cells.
    #[test]
    fn height_one_is_three_cubes_in_a_box_one_by_two_by_one_and_a_half() {
        let r = reads().unwrap();
        assert_eq!(r.one_cubes, 3);
        assert_eq!(r.one_box, ["1".to_owned(), "2".to_owned(), "1½".to_owned()]);
        let mut cells: Vec<Cell> = (0..3).flat_map(first_case).collect();
        cells.sort_unstable();
        let mut want = stacks::half_cells(1);
        want.sort_unstable();
        assert_eq!(cells, want);
        // six half cubes, each once, in a one by two by three block of half
        // heights
        assert_eq!(cells.len(), 6);
        assert!(
            cells
                .iter()
                .all(|c| c.0 == 0 && (0..2).contains(&c.1) && (0..3).contains(&c.2))
        );
        assert_eq!(mixed("9/2"), "4½");
        assert_eq!(mixed("5"), "5");
    }

    /// The chain: the box of n levels holds three stacks of n levels for
    /// every n, and the blocks of the chain stand apart on the screen.
    #[test]
    fn the_chain_is_the_boxes_of_one_to_five_levels() {
        for m in 1..=shown::STACKS_THROUGH {
            let volume = m * (m + 1) * (2 * m + 1) / 2; // m (m + 1) (m + 1/2) × 2 ÷ 2
            let stack: i64 = (1..=m).map(|k| k * k).sum();
            assert_eq!(volume, 3 * stack, "height {m}");
        }
        let next = shown::STACK + 1;
        let (points, length) = chain_points(next, CHAIN_GAP);
        assert_eq!(points.len() as i64, next);
        // each block starts to the right of the one before, on screen
        let view = ViewAt::classic((0.0, 0.0), 1.0);
        let mut right_edge = f64::NEG_INFINITY;
        for (i, p) in points.iter().enumerate() {
            let m = (i + 1) as f64;
            let left = view.project(*p)[0].min(view.project([p[0], p[1] + m + 1.0, 0.0])[0]);
            assert!(
                left > right_edge,
                "block {m} starts at {left}, the one before ends at {right_edge}"
            );
            let end = view.project([p[0] + m, p[1], 0.0])[0];
            right_edge = view.project([p[0] + m, p[1] + m + 1.0, 0.0])[0].max(end);
        }
        assert!(length > 0.0);
        // and they stay level: no block is higher on screen than the first
        for p in &points {
            let up = view.project(*p)[1] - view.project([0.0; 3])[1];
            assert!(up.abs() < 0.9, "a block stands {up} off the line");
        }
    }

    /// Everything stands on the screen: the block taken apart in the apart
    /// view (the base waiting at the right), settled, in the corner, on the
    /// shelf, the chain, the first case and the three turns on the ring.
    #[test]
    fn the_pictures_stand_on_the_screen() {
        let (n, next) = (shown::STACK, shown::STACK + 1);
        let check = |view: ViewAt, lo: P3, hi: P3, what: &str, low: f64| {
            for m in 0..8usize {
                let p = [
                    if m & 1 == 1 { hi[0] } else { lo[0] },
                    if m & 2 == 2 { hi[1] } else { lo[1] },
                    if m & 4 == 4 { hi[2] } else { lo[2] },
                ];
                let s = view.project(p);
                assert!(
                    s[0] > -FRAME_X_RADIUS + 0.2 && s[0] < FRAME_X_RADIUS - 0.2,
                    "{what}: x {} out of the frame",
                    s[0]
                );
                assert!(
                    s[1] > low && s[1] < SHELF_FLOOR + 1e-6,
                    "{what}: y {} out of its room",
                    s[1]
                );
            }
        };
        let low = -FRAME_Y_RADIUS + 0.15;
        let (w, d) = (next as f64, (next + 1) as f64);
        // taken apart: six slabs of the new block, and the base at the right
        let top = (next as f64) * (1.0 + GAP) + 1.0;
        check(APART, [0.0; 3], [w, d, top], "the block taken apart", low);
        check(
            APART,
            WAIT,
            [WAIT[0] + w, WAIT[1] + d, WAIT[2] + 1.0],
            "the base waiting",
            low,
        );
        // the first block of four levels, apart, is inside the same room
        check(
            APART,
            [0.0; 3],
            [n as f64, d, (n as f64 + 1.0) * (1.0 + GAP)],
            "four levels apart",
            low,
        );
        let settled = APART.fitted([0.0; 3], [w, d, d], SETTLED_AT, SETTLED_ROOM);
        check(settled, [0.0; 3], [w, d, d], "settled", low);
        let corner = APART.fitted([0.0; 3], [w, d, d], CORNER_AT, CORNER_ROOM);
        check(corner, [0.0; 3], [w, d, d], "in the corner", low);
        let slot = shelf_slot(SLOT);
        let thumb = APART.fitted([0.0; 3], [w, d, d], (slot[0], slot[1]), (THUMB, THUMB));
        // the thumbnail fills no more than its slot, centred in it
        let (mut lo, mut hi) = ((f64::INFINITY, f64::INFINITY), (f64::NEG_INFINITY, f64::NEG_INFINITY));
        for m in 0..8usize {
            let p = [
                if m & 1 == 1 { w } else { 0.0 },
                if m & 2 == 2 { d } else { 0.0 },
                if m & 4 == 4 { d } else { 0.0 },
            ];
            let s = thumb.project(p);
            lo = (lo.0.min(s[0]), lo.1.min(s[1]));
            hi = (hi.0.max(s[0]), hi.1.max(s[1]));
        }
        assert!(
            hi.0 - lo.0 <= THUMB + 1e-6 && hi.1 - lo.1 <= THUMB + 1e-6,
            "the thumbnail is too big"
        );
        assert!(((lo.0 + hi.0) / 2.0 - slot[0]).abs() < 1e-6 && ((lo.1 + hi.1) / 2.0 - slot[1]).abs() < 1e-6);
        // the chain sits right of the corner block and above the lower-third
        let (points, _) = chain_points(next, CHAIN_GAP);
        let hi = [
            points.last().map_or(0.0, |p| p[0]) + w,
            points.last().map_or(0.0, |p| p[1]) + d,
            w + 0.5,
        ];
        let chain = ViewAt::classic((0.0, 0.0), 1.0).fitted([0.0; 3], hi, CHAIN_AT, CHAIN_ROOM);
        check(chain, [0.0; 3], hi, "the chain", -2.85);
        let left = chain.project([0.0; 3])[0].min(chain.project([0.0, d, 0.0])[0]);
        let right_of_block = corner.project([w, 0.0, 0.0])[0].max(corner.project([w, d, 0.0])[0]);
        assert!(
            left > right_of_block,
            "the chain starts at {left}, the block ends at {right_of_block}"
        );
        // the first case and the three turns
        check(
            BASE_VIEW,
            [0.0; 3],
            [1.0, 2.0, 1.5],
            "the first case",
            -FRAME_Y_RADIUS + 0.9,
        );
        let ring = BASE_VIEW.fitted(
            RING_AT,
            [RING_AT[0] + 2.0, RING_AT[1] + 3.0, RING_AT[2] + 3.0],
            (LENS_AT[0], LENS_AT[1]),
            (1.5 * LENS_R, 1.5 * LENS_R),
        );
        for m in 0..8usize {
            let p = [
                RING_AT[0] + if m & 1 == 1 { 2.0 } else { 0.0 },
                RING_AT[1] + if m & 2 == 2 { 3.0 } else { 0.0 },
                RING_AT[2] + if m & 4 == 4 { 3.0 } else { 0.0 },
            ];
            let s = ring.project(p);
            let off = ((s[0] - LENS_AT[0]).powi(2) + (s[1] - LENS_AT[1]).powi(2)).sqrt();
            assert!(off < LENS_R, "the turns reach {off} from the ring's middle");
        }
        // the inset: flat square and slabs stay right of the settled block
        let block_right = settled.project([w, 0.0, 0.0])[0].max(settled.project([w, d, 0.0])[0]);
        assert!(FLAT_AT[0] > block_right, "the flat square meets the block");
        assert!(SLABS_AT[0] > block_right, "the slabs meet the block");
        let slabs_right = SLABS_AT[0] + 3.0 * (w * SLAB_CELL) + 2.0 * SLAB_GAP;
        assert!(slabs_right < FRAME_X_RADIUS - 0.15, "the slabs reach {slabs_right}");
        assert!(FLAT_AT[1] + w * FLAT_CELL <= SHELF_FLOOR + 1e-9);
    }

    /// With the scratch voice placed each line starts within two seconds,
    /// and a breath and a question's pause, of the end of the one before.
    #[test]
    fn no_line_waits_long_for_its_picture() {
        let v = testing::run_voiced("09_every_height").unwrap();
        let mut lines = v.lines.clone();
        lines.sort_by(|a, b| a.1.total_cmp(&b.1));
        let mut prev: Option<(String, f64)> = None;
        let mut late: Vec<String> = Vec::new();
        for (id, at, len) in &lines {
            if let Some((before, end)) = &prev {
                let gap = at - end;
                eprintln!("{id}: starts {at:.1} after a gap of {gap:.1} (since {before})");
                let allowed = 2.0 + explainer_kit::animatic::HOLD_S + super::super::draw::QUESTION_HOLD_S;
                if gap > allowed {
                    late.push(format!("{id} starts {gap:.1} s after {before} ends"));
                }
            }
            prev = Some((id.clone(), at + len));
        }
        eprintln!("chapter {:.1} s", v.ran.seconds);
        assert!(late.is_empty(), "{late:?}");
    }

    /// The chapter speaks exactly the script's 09_ lines, in order.
    #[test]
    fn the_chapter_speaks_the_scripts_lines_in_order() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let ids: Vec<&str> = script
            .lines()
            .iter()
            .map(|(id, _)| id.as_str())
            .filter(|id| id.starts_with("09_"))
            .collect();
        assert_eq!(ids, LINES);
    }

    /// The scene constructs end to end: the pieces end on the model's cells
    /// (the scene fails otherwise), every phrase is in its line, the layout
    /// lint finds nothing.
    #[test]
    fn the_chapter_runs_and_the_layout_lint_is_clean() {
        let ran = testing::run("09_every_height").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            "Why would five fit?",
            "under",
            "behind",
            "beside",
            "25 + 25 + 25 = 75",
            "16",
            "9",
            "staircase: 15 cubes",
            "3 cubes",
            "1½",
            "mathematical induction:",
            "Not guessing from cases, like the circle",
            "each stack is the first one, turned",
            "two staircases make a rectangle",
            "height one is 3 cubes in a 1 by 2 by 1½ box",
            "three new slabs",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never typesets {want:?}: {words:?}"
            );
        }
        assert!(ran.seconds > 40.0 && ran.seconds < 110.0, "{} s", ran.seconds);
    }
}
