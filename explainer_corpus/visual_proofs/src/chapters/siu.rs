//! Siu's box on the cube kit (`iso`), for chapters 07 to 09: the three stacks
//! of `model::stacks`, the turns that put the second and third where the
//! model puts them (as quarter turns, found by search and checked), the box's
//! cells by level and by stack, and the one view the three chapters share.
//!
//! The camera never moves: the view is fixed, and the picture is fitted to it
//! by where things stand (a stack stands at the left, a row of five slabs
//! runs to the right).

use super::draw::copy;
use super::iso::{Axis, Cell, P3, ViewAt, compose, quarter_turns};
use crate::model::stacks;
use fmn::library::line_brace;
use fmn::prelude::*;

/// A turn of the lattice, exactly (a rotation matrix of whole numbers).
pub type Turn = [[i64; 3]; 3];

/// Where the lattice origin is on screen and how big a lattice unit is: a
/// stack of four stands at the left under the shelf's row, the copies wait
/// to its right on the screen's own horizontal (a lattice step of 5 in x and
/// 3 in y is a step to the right of about 3.5 and none up), and the five
/// slabs of 07_4 stand in two rows over the same ground.
pub const VIEW: ViewAt = ViewAt::classic((-5.9, -1.9), 0.6);
/// Where each slab of the block stands when the block is taken apart (07_4):
/// the lowest stays, the next two in front, the last two behind.
pub const ROW: [(f64, f64); 5] = [(0.0, 0.0), (4.5, 0.0), (9.0, 2.0), (0.0, 6.5), (4.5, 6.5)];
/// Where the second and the third copy of the stack wait, as lattice points
/// of their lowest corner, before they are turned and slid in.
pub const WAIT: [P3; 3] = [[0.0, 0.0, 0.0], [5.0, 3.0, 0.0], [10.0, 5.0, 0.0]];

/// The `which`-th stack (0 the first, in blue; 1 the second, in copper; 2 the
/// third, in pink) at its place in the box of `n` levels: the model's cells.
pub fn placed(n: i64, which: usize) -> Vec<Cell> {
    stacks::placed(n, which)
}

/// The stacks' colours: first, second, third copy.
pub fn colour(which: usize) -> Srgb {
    copy(which)
}

/// The least corner of a set of cells: the lowest x, y and z, as a point.
pub fn min_corner(cells: &[Cell]) -> P3 {
    let m = |f: fn(&Cell) -> i64| cells.iter().map(f).min().unwrap_or(0) as f64;
    [m(|c| c.0), m(|c| c.1), m(|c| c.2)]
}

/// The most distant corner of a set of cells: the highest x, y and z, plus
/// one.
#[cfg(test)]
pub fn max_corner(cells: &[Cell]) -> P3 {
    let m = |f: fn(&Cell) -> i64| cells.iter().map(f).max().unwrap_or(0) as f64 + 1.0;
    [m(|c| c.0), m(|c| c.1), m(|c| c.2)]
}

/// The cells of stack `which` at level `z` of the box.
pub fn level(n: i64, which: usize, z: i64) -> Vec<Cell> {
    placed(n, which).into_iter().filter(|c| c.2 == z).collect()
}

/// The quarter turns (one, two or three, each about a lattice axis) whose
/// composition, applied in order, is exactly `turn`; the shortest found.
pub fn quarters_for(turn: Turn) -> Option<Vec<(Axis, i32)>> {
    let steps: Vec<(Axis, i32)> = [Axis::X, Axis::Y, Axis::Z]
        .into_iter()
        .flat_map(|a| [1, -1, 2].into_iter().map(move |q| (a, q)))
        .collect();
    let identity = quarter_turns(Axis::X, 0);
    let apply = |seq: &[(Axis, i32)]| {
        seq.iter()
            .fold(identity, |acc, &(a, q)| compose(quarter_turns(a, q), acc))
    };
    for len in 0..=3 {
        let mut idx = vec![0usize; len];
        loop {
            let seq: Vec<(Axis, i32)> = idx.iter().map(|&i| steps[i]).collect();
            if apply(&seq) == turn {
                return Some(seq);
            }
            // next sequence of this length
            let mut at = len;
            while at > 0 {
                idx[at - 1] += 1;
                if idx[at - 1] < steps.len() {
                    break;
                }
                idx[at - 1] = 0;
                at -= 1;
            }
            if at == 0 {
                break;
            }
        }
    }
    None
}

/// Whether the stacks can be dropped one after another from straight above,
/// in the order 0, 1, 2, and never meet a cube of another: in every column of
/// the box the first stack's cubes lie under the second's and the second's
/// under the third's.
#[cfg(test)]
pub fn drops_from_above(n: i64) -> bool {
    use std::collections::HashMap;
    type Span = Option<(i64, i64)>;
    let mut tops: HashMap<(i64, i64), [Span; 3]> = HashMap::new();
    for which in 0..3 {
        for (x, y, z) in placed(n, which) {
            let e = tops.entry((x, y)).or_insert([None; 3]);
            let slot = &mut e[which];
            *slot = Some(match *slot {
                Some((lo, hi)) => (lo.min(z), hi.max(z)),
                None => (z, z),
            });
        }
    }
    tops.values().all(|e| {
        let spans: Vec<(i64, i64)> = e.iter().flatten().copied().collect();
        spans.windows(2).all(|w| w[0].1 < w[1].0)
    })
}

/// A brace along the edge from `a` to `b` on screen, bulging away to the side
/// `out` (a screen direction, not nec. unit), with `words` at its tip. The
/// words are built by the caller, so any text or TeX will do. Detached.
pub fn edge_brace(stage: &mut Stage<'_>, a: Vec3, b: Vec3, out: Vec3, color: Srgb) -> fmn::Result<(Mob, f64)> {
    // the brace is built along a flat line and then turned with it, so its
    // direction is given in the flat line's frame: it bulges to the side of
    // the line (a to b) that `out` is on
    let along = [b[0] - a[0], b[1] - a[1]];
    let side = along[0] * out[1] - along[1] * out[0];
    let flat_dir = if side >= 0.0 { UP } else { DOWN };
    let buff = 0.08;
    let brace = line_brace(a, b, flat_dir).buff(buff).color(color);
    let depth = brace.depth();
    let m = stage.arena_mut().add(brace.build());
    Ok((m, buff + depth))
}

/// The words of the legend and their colours, for the three copies.
pub fn legend() -> [(&'static str, Srgb); 3] {
    [("first", colour(0)), ("second", colour(1)), ("third", colour(2))]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The quarter turns found are exactly the model's turns, and the first
    /// is the identity.
    #[test]
    fn the_models_turns_are_quarter_turns() {
        assert_eq!(quarters_for(stacks::TURNS[0]).unwrap(), vec![]);
        for which in 1..3 {
            let seq = quarters_for(stacks::TURNS[which]).unwrap();
            assert!(!seq.is_empty() && seq.len() <= 2, "stack {which}: {seq:?}");
            let got = seq.iter().fold(quarter_turns(Axis::X, 0), |acc, &(a, q)| {
                compose(quarter_turns(a, q), acc)
            });
            assert_eq!(got, stacks::TURNS[which]);
            assert_eq!(stacks::determinant(got), 1, "a turn, never a mirror image");
        }
    }

    /// The box is filled once and by the three stacks only, and the stacks
    /// can be dropped in from above one after another (the first stack
    /// stands, the second goes over it, the third over both).
    #[test]
    fn the_stacks_drop_in_from_above_in_order() {
        for n in 1..=6 {
            assert!(drops_from_above(n), "n = {n}");
            let all: Vec<Cell> = (0..3).flat_map(|w| placed(n, w)).collect();
            let mut sorted = all.clone();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(sorted.len(), all.len(), "no cube twice");
            assert_eq!(all.len() as i64, 3 * (n * (n + 1) * (2 * n + 1) / 6));
        }
    }

    /// The box of four levels: 4 wide, 5 deep, 4 tall, with the step of ten
    /// on top, and each stack's bounds a cube of side four.
    #[test]
    fn the_box_of_four_levels() {
        let n = 4;
        for which in 0..3 {
            let cells = placed(n, which);
            let (lo, hi) = (min_corner(&cells), max_corner(&cells));
            assert_eq!(
                [hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]],
                [4.0, 4.0, 4.0],
                "stack {which}"
            );
        }
        let all: Vec<Cell> = (0..3).flat_map(|w| placed(n, w)).collect();
        assert_eq!(min_corner(&all), [0.0, 0.0, 0.0]);
        assert_eq!(max_corner(&all), [4.0, 5.0, 5.0]);
        let step = all.iter().filter(|c| c.2 == 4).count();
        assert_eq!(step, 10);
        for z in 0..4 {
            assert_eq!(all.iter().filter(|c| c.2 == z).count(), 20, "level {z}");
        }
    }
}
