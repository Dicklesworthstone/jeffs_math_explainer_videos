//! Chapters 08 and 09's devices (storyboard-draft, "08_the_line_on_the_dial"
//! and "09_a_curve_for_three"): the dial board, positions across and the
//! dial's marks 0 to 10 up, where every point sits on a mark and nothing
//! lies between marks; the pieces a straight rise or fall breaks into when
//! it runs off the board's top and comes back in at the bottom; and the
//! coil a height winds round a dial face, one loop for every whole turn,
//! before it lands on its mark.
//!
//! The faces are `two_keys.rs`'s, the plot `plot.rs`'s and the outlines
//! `envelope.rs`'s, unchanged. Builders return detached mobjects, as there.
//! What a board, a coil or a wrapped line shows is computed here from the
//! numbers the scenes pass in, and the tests check it against the dial's
//! own arithmetic (`Dial::wrap`), so a picture cannot drift from the model.
//!
//! As in the other chapters, emphasis never restyles or scales typeset
//! text (Mail 6866): a row label or a point is pointed at by a ring or an
//! outline of its own ([`ring`], `envelope::outline`), and only shapes
//! swell or change color.

use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{disc, fade_in_all, line_style, stroke};
use super::two_keys::Face;

// ------------------------------------------------------------------ layout

/// How far the board's edges run past the first and last positions, in
/// steps of position.
pub const EDGE_RUN: f64 = 0.4;
/// The faint dot at every (position, mark), and a point's radius.
pub const LATTICE_R: f64 = 0.025;
pub const BOARD_POINT_R: f64 = 0.09;
/// The row labels' size and how far left of position 0 they sit; the
/// position labels' size and how far under mark 0 (in rows).
pub const ROW_LABEL_SIZE: f64 = 16.0;
pub const ROW_LABEL_DX: f64 = 0.32;
pub const POSITION_LABEL_SIZE: f64 = 16.0;
pub const POSITION_LABEL_DK: f64 = 1.3;
/// The title's size and how far over the top edge it sits.
pub const TITLE_SIZE: f64 = 20.0;
pub const TITLE_GAP: f64 = 0.36;
/// The edges' line width.
const EDGE_LINE: f64 = 2.0;
/// A wrapped line's width.
pub const WRAPPED_W: f64 = 3.0;

/// A coil's first loop, just outside the rim, and how much further out each
/// whole turn stands, as fractions of the face's radius; its samples a mark
/// and its width.
pub const COIL_IN: f64 = 1.08;
pub const COIL_STEP: f64 = 0.12;
const COIL_SAMPLES: f64 = 12.0;
pub const COIL_W: f64 = 3.0;

/// A ring's line width.
const RING_LINE: f64 = 3.0;

// -------------------------------------------------------------- the board

/// Where a board sits: position 0's place across, mark 0's place up, one
/// position's and one mark's steps, the dial's marks (rows 0 to marks − 1)
/// and the last position.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grid {
    pub x0: f64,
    pub y0: f64,
    pub dx: f64,
    pub dy: f64,
    pub marks: u32,
    pub last: u32,
}

impl Grid {
    /// The scene point at position `x`, mark (row) `k`.
    pub fn at(&self, x: f64, k: f64) -> Vec3 {
        v(self.x0 + self.dx * x, self.y0 + self.dy * k)
    }

    /// The bottom edge, half a row under mark 0, and the top edge, half a
    /// row over the top mark: where a wrapped line leaves and comes back.
    pub fn bottom(&self) -> f64 {
        self.at(0.0, -0.5)[1]
    }

    pub fn top(&self) -> f64 {
        self.at(0.0, f64::from(self.marks) - 0.5)[1]
    }

    /// The edges' left and right ends.
    pub fn left(&self) -> f64 {
        self.x0 - EDGE_RUN * self.dx
    }

    pub fn right(&self) -> f64 {
        self.x0 + self.dx * (f64::from(self.last) + EDGE_RUN)
    }

    /// The centre of mark `k`'s row label, left of position 0.
    pub fn row_label_at(&self, k: u32) -> Vec3 {
        v(self.x0 - ROW_LABEL_DX, self.at(0.0, f64::from(k))[1])
    }

    /// The centre of position `x`'s label, under the bottom edge.
    pub fn position_label_at(&self, x: u32) -> Vec3 {
        self.at(f64::from(x), -POSITION_LABEL_DK)
    }

    /// The title's centre, over the top edge, across the board's middle.
    pub fn title_at(&self) -> Vec3 {
        v((self.left() + self.right()) / 2.0, self.top() + TITLE_GAP)
    }
}

/// The board: a faint dot at every position and mark, each mark's label up
/// the left, the two edges, a title over them and, if asked, each
/// position's number under them.
pub struct Board {
    pub grid: Grid,
    parts: Vec<Mob>,
    rows: Vec<Mob>,
}

impl Board {
    pub fn build(kit: &Kit, stage: &mut Stage<'_>, grid: Grid, title: &str, numbered: bool) -> fmn::Result<Self> {
        let mut parts = Vec::new();
        for x in 0..=grid.last {
            for k in 0..grid.marks {
                parts.push(disc(
                    stage,
                    Palette::faint(),
                    grid.at(f64::from(x), f64::from(k)),
                    LATTICE_R,
                ));
            }
        }
        let mut rows = Vec::with_capacity(grid.marks as usize);
        for k in 0..grid.marks {
            let label = kit.text(
                stage,
                &k.to_string(),
                ROW_LABEL_SIZE,
                Palette::dim(),
                grid.row_label_at(k),
            )?;
            parts.push(label);
            rows.push(label);
        }
        let edges = [grid.bottom(), grid.top()].map(|y| {
            stroke(
                stage,
                vec![v(grid.left(), y), v(grid.right(), y)],
                Palette::dim(),
                EDGE_LINE,
            )
        });
        parts.extend(edges);
        parts.push(kit.text(stage, title, TITLE_SIZE, Palette::dim(), grid.title_at())?);
        if numbered {
            for x in 0..=grid.last {
                parts.push(kit.text(
                    stage,
                    &x.to_string(),
                    POSITION_LABEL_SIZE,
                    Palette::dim(),
                    grid.position_label_at(x),
                )?);
            }
        }
        Ok(Self { grid, parts, rows })
    }

    /// Everything the board is drawn with.
    pub fn parts(&self) -> &[Mob] {
        &self.parts
    }

    /// Mark `k`'s row label.
    pub fn row_label(&self, k: u32) -> Option<Mob> {
        self.rows.get(k as usize).copied()
    }

    /// Bring the board in, over `rt`.
    pub fn show(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        fade_in_all(stage, &self.parts, rt)
    }

    /// A point at position `x`, mark `k`, in `color`. Detached.
    pub fn point(&self, stage: &mut Stage<'_>, x: u32, k: u32, color: Srgb) -> Mob {
        disc(stage, color, self.grid.at(f64::from(x), f64::from(k)), BOARD_POINT_R)
    }

    /// The straight line from `from` to `to` (position, unwrapped height),
    /// wrapped onto the board ([`wrapped`]): one stroke per piece, in
    /// order, in `color`, `w` wide; each grows from its start. Detached.
    pub fn line(&self, stage: &mut Stage<'_>, from: (f64, f64), to: (f64, f64), color: Srgb, w: f64) -> Vec<Mob> {
        wrapped(from, to, self.grid.marks)
            .into_iter()
            .map(|[a, b]| stroke(stage, vec![self.grid.at(a.0, a.1), self.grid.at(b.0, b.1)], color, w))
            .collect()
    }
}

// --------------------------------------------------------------- wrapping

/// The band an unwrapped height lies in, on `marks` marks: band j holds the
/// heights from j·marks − ½ to (j + 1)·marks − ½, so each mark sits in the
/// middle of its row.
fn band(h: f64, marks: u32) -> f64 {
    ((h + 0.5) / f64::from(marks)).floor()
}

/// The pieces, on a board of `marks` rows, of the straight line from
/// `from` to `to` (position, unwrapped height), as (position, row) ends,
/// in order from `from`. Each piece runs between the board's edges, half a
/// row outside marks 0 and marks − 1; where the line crosses an edge it
/// leaves there and comes back in at the other edge, at the same position.
/// A whole height lands on its own mark: at position x the row is the
/// height wrapped, as the dial wraps it.
pub fn wrapped(from: (f64, f64), to: (f64, f64), marks: u32) -> Vec<[(f64, f64); 2]> {
    let ((x0, h0), (x1, h1)) = (from, to);
    let m = f64::from(marks);
    let (j0, j1) = (band(h0, marks) as i64, band(h1, marks) as i64);
    // where along the line (0 to 1) each edge is crossed, in order
    let edge = |j: i64| (j as f64 * m - 0.5 - h0) / (h1 - h0);
    let mut cuts = vec![0.0];
    if j1 > j0 {
        cuts.extend((j0 + 1..=j1).map(edge));
    } else {
        cuts.extend((j1 + 1..=j0).rev().map(edge));
    }
    cuts.push(1.0);
    cuts.windows(2)
        .map(|w| {
            let mid = h0 + (h1 - h0) * (w[0] + w[1]) / 2.0;
            let shift = band(mid, marks) * m;
            let end = |t: f64| (x0 + (x1 - x0) * t, h0 + (h1 - h0) * t - shift);
            [end(w[0]), end(w[1])]
        })
        .collect()
}

/// The whole turns an ordinary height `h` makes round `marks` marks, and
/// the mark it lands on.
pub fn turns_and_mark(h: i64, marks: u32) -> (i64, u32) {
    let m = i64::from(marks);
    // rem_euclid of a positive modulus is in 0..marks, so it fits
    (h.div_euclid(m), h.rem_euclid(m) as u32)
}

/// What wrapping `h` takes off: "9 stays 9", "13 − 11 = 2", "25 − 11 −
/// 11 = 3", one "− 11" for each whole turn the coil shows.
pub fn taken_off(h: i64, marks: u32) -> String {
    let (turns, k) = turns_and_mark(h, marks);
    if turns == 0 {
        return format!("{h} stays {h}");
    }
    let minus: String = (0..turns).map(|_| format!(" \u{2212} {marks}")).collect();
    format!("{h}{minus} = {k}")
}

/// The same, as whole turns and what is left: "47 = 4 × 11 + 3".
pub fn turns_and_more(h: i64, marks: u32) -> String {
    let (turns, k) = turns_and_mark(h, marks);
    format!("{h} = {turns} \u{d7} {marks} + {k}")
}

/// The mark 0 nearest to unwrapped mark `u` (a multiple of `marks`; the
/// one forward on a tie): where a needle that has landed goes back to the
/// top the short way, ready for the next height.
pub fn nearest_top(u: i64, marks: u32) -> i64 {
    let m = i64::from(marks);
    let below = u.div_euclid(m) * m;
    if 2 * (u - below) < m { below } else { below + m }
}

// ------------------------------------------------------------------ coils

/// A coil's distance from the centre at `k` marks along it, as a fraction
/// of the face's radius: one loop a turn, each a step further out.
pub fn coil_radius(k: f64, marks: u32) -> f64 {
    COIL_IN + COIL_STEP * k / f64::from(marks)
}

/// The coil that height `h` winds round `face` (of `marks` marks): from
/// mark 0 forward `h` marks, just outside the rim and a little further out
/// with every whole turn, so each turn shows, in `color`; it grows from
/// mark 0. Detached.
pub fn coil(stage: &mut Stage<'_>, face: &Face, marks: u32, h: u32, color: Srgb) -> Mob {
    let n = ((f64::from(h) * COIL_SAMPLES).ceil() as usize).max(2);
    let points: Vec<Vec3> = (0..=n)
        .map(|i| {
            let k = f64::from(h) * i as f64 / n as f64;
            face.point(k, face.r * coil_radius(k, marks))
        })
        .collect();
    stroke(stage, points, color, COIL_W)
}

// --------------------------------------------------------------- emphasis

/// A ring of radius `r` round `at`, in `color`. Detached.
pub fn ring(stage: &mut Stage<'_>, at: Vec3, r: f64, color: Srgb) -> Mob {
    let c = Circle::new()
        .radius(r)
        .style(line_style(color, RING_LINE))
        .build()
        .moved_to(at);
    stage.arena_mut().add(c)
}

/// Strokes growing one after another, the whole taking `rt`.
pub fn create_lagged(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64, lag: f64) -> fmn::Result<()> {
    if mobs.is_empty() {
        return Ok(());
    }
    let mut anims = Vec::with_capacity(mobs.len());
    for &m in mobs {
        anims.push(stage.prepare(show_creation(m))?);
    }
    let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), anims, lag)?.with_run_time(rt);
    stage.play(g)?;
    Ok(())
}

/// The scene's time now, in seconds.
pub fn now(stage: &Stage<'_>) -> f64 {
    stage.scene().time().to_f64()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Dial, LINE_SLOPE, MARKS, Recipe, SECRET, TRUSTEES};

    /// The row a list of pieces passes through at position `x`, if any
    /// piece covers it.
    fn row_at(pieces: &[[(f64, f64); 2]], x: f64) -> Option<f64> {
        pieces.iter().find_map(|&[(xa, ka), (xb, kb)]| {
            let (lo, hi) = if xa <= xb { (xa, xb) } else { (xb, xa) };
            ((lo - 1e-9..=hi + 1e-9).contains(&x) && (xb - xa).abs() > 1e-12)
                .then(|| ka + (kb - ka) * (x - xa) / (xb - xa))
        })
    }

    /// Every piece stays between the edges, consecutive pieces meet at the
    /// same position on opposite edges, the first starts at `from`'s row
    /// wrapped and the last ends at `to`'s.
    fn check_pieces(from: (f64, f64), to: (f64, f64), marks: u32) {
        let pieces = wrapped(from, to, marks);
        let m = f64::from(marks);
        let wrap = |h: f64| h - band(h, marks) * m;
        assert!((pieces[0][0].0 - from.0).abs() < 1e-9 && (pieces[0][0].1 - wrap(from.1)).abs() < 1e-9);
        let last = pieces.last().unwrap()[1];
        assert!((last.0 - to.0).abs() < 1e-9 && (last.1 - wrap(to.1)).abs() < 1e-9);
        for p in &pieces {
            for &(_, k) in p {
                assert!((-0.5 - 1e-9..=m - 0.5 + 1e-9).contains(&k), "{from:?} {to:?}: row {k}");
            }
        }
        for w in pieces.windows(2) {
            let (a, b) = (w[0][1], w[1][0]);
            assert!((a.0 - b.0).abs() < 1e-9, "the line jumps across at a wrap");
            assert!((a.1 - b.1).abs() > m - 1.0 - 1e-9, "a wrap goes from edge to edge");
        }
        // a whole number of turns: the pieces' rises add up to the line's
        let rise: f64 = pieces.iter().map(|p| p[1].1 - p[0].1).sum();
        assert!((rise - (to.1 - from.1)).abs() < 1e-9);
    }

    /// The line from the secret climbing 4 a step, wrapped, passes through
    /// every trustee's share at its position: the same points as the dial's
    /// arithmetic, nothing between marks.
    #[test]
    fn the_wrapped_line_meets_every_share() {
        let dial = Dial::vault();
        let line = Recipe::new(SECRET, &[LINE_SLOPE]);
        let top = line.height(5) as f64;
        let pieces = wrapped((0.0, f64::from(SECRET)), (5.0, top), MARKS);
        assert_eq!(pieces.len(), 3, "25 is two whole turns and 3: three pieces");
        for x in 0..=5 {
            let k = row_at(&pieces, f64::from(x)).unwrap();
            assert!((k - f64::from(line.share(dial, x))).abs() < 1e-9, "{x}: {k}");
        }
        check_pieces((0.0, f64::from(SECRET)), (5.0, top), MARKS);
        // the planted negative: drawn unwrapped, trustees 3 to 5 fall off
        // the board
        let flat = [[(0.0, f64::from(SECRET)), (5.0, top)]];
        assert!(
            TRUSTEES
                .iter()
                .filter(|&&t| row_at(&flat, f64::from(t)).unwrap() > 10.5)
                .count()
                >= 3
        );
    }

    /// Lines up, down, flat and across several turns wrap continuously.
    #[test]
    fn wrapped_lines_meet_at_the_edges() {
        for (from, to) in [
            ((0.0, 5.0), (5.0, 25.0)),
            ((2.0, 2.0), (1.0, -2.0)),
            ((1.0, 9.0), (0.0, 5.0)),
            ((2.0, 2.0), (0.0, 2.0)),
            ((2.0, 2.0), (0.0, -18.0)),
            ((0.0, 5.0), (5.0, 215.0)),
            ((3.0, 0.0), (4.0, 10.0)),
        ] {
            check_pieces(from, to, MARKS);
        }
        // a flat line is one piece
        assert_eq!(wrapped((2.0, 2.0), (0.0, 2.0), MARKS).len(), 1);
        // two back from 2 on eleven marks: out at the bottom, in at the
        // top, on to 9
        let back = wrapped((2.0, 2.0), (1.0, -2.0), MARKS);
        assert_eq!(back.len(), 2);
        assert!((back[0][1].1 + 0.5).abs() < 1e-9 && (back[1][0].1 - 10.5).abs() < 1e-9);
        assert!((back[1][1].1 - 9.0).abs() < 1e-9);
    }

    /// Turns and marks are the dial's own: wrapping by turns lands where
    /// `Dial::wrap` does, and the texts say so.
    #[test]
    fn turns_land_where_the_dial_wraps() {
        let dial = Dial::vault();
        for h in -30..=250 {
            let (turns, k) = turns_and_mark(h, MARKS);
            assert_eq!(k, dial.wrap(h));
            assert_eq!(turns * 11 + i64::from(k), h);
        }
        assert_eq!(turns_and_mark(47, MARKS), (4, 3));
        assert_eq!(taken_off(9, MARKS), "9 stays 9");
        assert_eq!(taken_off(13, MARKS), "13 \u{2212} 11 = 2");
        assert_eq!(taken_off(25, MARKS), "25 \u{2212} 11 \u{2212} 11 = 3");
        assert_eq!(turns_and_more(47, MARKS), "47 = 4 \u{d7} 11 + 3");
        assert_eq!(turns_and_more(215, MARKS), "215 = 19 \u{d7} 11 + 6");
    }

    /// The needle goes back to the top the short way.
    #[test]
    fn the_nearest_top_is_the_short_way_back() {
        assert_eq!(nearest_top(9, 11), 11);
        assert_eq!(nearest_top(13, 11), 11);
        // 33 is five marks away; 22 is six. The forward zero is nearer.
        assert_eq!(nearest_top(28, 11), 33);
        assert_eq!(nearest_top(22, 11), 22);
        assert_eq!(nearest_top(0, 11), 0);
        for u in -40..=60 {
            let t = nearest_top(u, 11);
            assert_eq!(t.rem_euclid(11), 0);
            assert!((u - t).abs() <= 5, "{u} goes back {}", u - t);
        }
    }

    /// Each whole turn of a coil stands clear of the last by more than the
    /// line's width, on the smallest face the chapters draw (radius 0.45);
    /// chapter 08's longest coil (25, two turns and three) stays inside
    /// where an arc's label would sit, and chapter 09's (47, four turns and
    /// three) inside one and a half radii.
    #[test]
    fn a_coils_turns_stand_apart() {
        use fmn::core::constants::STROKE_WIDTH_CONVERSION;
        let gap = 0.45 * (coil_radius(11.0, MARKS) - coil_radius(0.0, MARKS));
        assert!(gap > 1.5 * COIL_W * STROKE_WIDTH_CONVERSION, "{gap}");
        assert!(coil_radius(25.0, MARKS) < super::super::two_keys::ARC_LABEL_IN);
        assert!(coil_radius(47.0, MARKS) < 1.6);
        assert!(coil_radius(0.0, MARKS) > 1.0, "outside the rim");
    }

    /// A board's rows, edges and labels are where they say.
    #[test]
    fn a_board_is_where_it_says() {
        let g = Grid {
            x0: -5.6,
            y0: -0.12,
            dx: 0.85,
            dy: 0.29,
            marks: MARKS,
            last: 5,
        };
        assert!((g.top() - g.at(0.0, 10.0)[1] - 0.145).abs() < 1e-9);
        assert!((g.at(0.0, 0.0)[1] - g.bottom() - 0.145).abs() < 1e-9);
        assert!(g.left() < g.x0 && g.right() > g.at(5.0, 0.0)[0]);
        assert!(g.row_label_at(3)[0] < g.left() + 0.1);
        assert!(g.position_label_at(2)[1] < g.bottom());
        assert!(g.title_at()[1] > g.top());
    }
}
