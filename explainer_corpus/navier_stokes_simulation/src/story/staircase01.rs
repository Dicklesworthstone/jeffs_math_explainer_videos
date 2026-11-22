//! Exact staircase geometry for chapter 17, cues 17_1 to 17_7. Standard library only, so its
//! tests run with plain `rustc --test`, without the renderer.
//!
//! RubyRaven `ruby-boundary01`, Agent Mail 36212. Definitions follow BronzeJaguar's reviewed
//! `cl01.py` (navier-closing-scene02, run `cl01-run01.txt`):
//! - lengths are in cylinder diameters D, with the circle's centre at the origin, which is a
//!   grid corner;
//! - a grid has `n` cells per diameter, and cell (i, j) covers [i/n, (i+1)/n] × [j/n, (j+1)/n];
//! - a cell is solid when its middle ((2i+1)/(2n), (2j+1)/(2n)) lies strictly inside the circle.
//!
//! Membership is decided in exact integer arithmetic. Floating point is used only for
//! positions on screen, for the radial gap readings, and for the cut-cell areas of 17_5,
//! which have closed forms.

use std::f64::consts::TAU;

/// The true cylinder: radius 1/2 D.
pub const TRUE_RADIUS: (i64, i64) = (1, 2);
/// The typed cylinder of 17_6: a diameter five percent too small, radius 19/40 D.
pub const TYPED_RADIUS: (i64, i64) = (19, 40);
/// Cells per diameter, 17_1 to 17_4 (facts stair.n8 to stair.n128).
pub const LEVELS: [i64; 5] = [8, 16, 32, 64, 128];
/// Cells per diameter, 17_6 (facts wrong.n32 to wrong.n128).
pub const WRONG_LEVELS: [i64; 3] = [32, 64, 128];

/// Which side of the outline an edge faces, outward.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

/// One straight piece of the outline, a maximal run of cell edges, in cell units (multiply by
/// 1/n for diameters). `a` and `b` follow the counterclockwise outline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Edge {
    pub a: (i64, i64),
    pub b: (i64, i64),
    pub side: Side,
}

impl Edge {
    /// Length in cell widths.
    pub fn cells(&self) -> i64 {
        (self.b.0 - self.a.0).abs() + (self.b.1 - self.a.1).abs()
    }

    /// The outward unit normal: by construction it points straight up, down, left or right
    /// (fact stair.normals.axis).
    pub fn normal(&self) -> (i64, i64) {
        match self.side {
            Side::Top => (0, 1),
            Side::Bottom => (0, -1),
            Side::Left => (-1, 0),
            Side::Right => (1, 0),
        }
    }
}

/// The staircase of a circle of radius `r_num / r_den` D about a grid corner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Staircase {
    pub n: i64,
    pub r_num: i64,
    pub r_den: i64,
    /// For column i in [-n, n), stored at index i + n: the number t of solid cells above the
    /// axis. The column's solid cells are j in [-t, t), by the circle's symmetry.
    half: Vec<i64>,
}

impl Staircase {
    pub fn new(n: i64, (r_num, r_den): (i64, i64)) -> Self {
        assert!(n > 0 && n <= 1 << 12, "cells per diameter out of range");
        assert!(r_num > 0 && r_num < r_den, "the radius must lie in (0, 1) D");
        // Middle strictly inside: ((2i+1)^2 + (2j+1)^2) r_den^2 < 4 n^2 r_num^2.
        let limit = 4 * n * n * r_num * r_num;
        let den2 = r_den * r_den;
        let half = (-n..n)
            .map(|i| {
                let a = (2 * i + 1) * (2 * i + 1);
                let mut t = 0;
                while (a + (2 * t + 1) * (2 * t + 1)) * den2 < limit {
                    t += 1;
                }
                t
            })
            .collect();
        Self { n, r_num, r_den, half }
    }

    /// The radius in diameters.
    pub fn radius(&self) -> f64 {
        self.r_num as f64 / self.r_den as f64
    }

    /// The cell width in diameters.
    pub fn h(&self) -> f64 {
        1.0 / self.n as f64
    }

    /// Solid cells above the axis in column i (0 outside the grid).
    pub fn half_at(&self, i: i64) -> i64 {
        if i < -self.n || i >= self.n {
            0
        } else {
            self.half[(i + self.n) as usize]
        }
    }

    /// Whether cell (i, j) is solid.
    pub fn solid(&self, i: i64, j: i64) -> bool {
        let t = self.half_at(i);
        -t <= j && j < t
    }

    /// The number of solid cells (facts stair.n8 to stair.n128: 52, 208, 812, 3228, 12892).
    pub fn cells(&self) -> i64 {
        self.half.iter().map(|t| 2 * t).sum()
    }

    /// The first and last columns holding a solid cell.
    pub fn columns(&self) -> (i64, i64) {
        let first = (-self.n..self.n).find(|&i| self.half_at(i) > 0).expect("a solid cell");
        let last = (-self.n..self.n)
            .rev()
            .find(|&i| self.half_at(i) > 0)
            .expect("a solid cell");
        (first, last)
    }

    /// The bounding box of the solid cells, cell units: (x_min, x_max, y_min, y_max).
    pub fn bbox(&self) -> (i64, i64, i64, i64) {
        let (first, last) = self.columns();
        let top = (first..=last).map(|i| self.half_at(i)).max().unwrap_or(0);
        (first, last + 1, -top, top)
    }

    /// The outline as a counterclockwise polygon in cell units: no repeated and no collinear
    /// vertices, so each polygon side is one maximal straight piece.
    pub fn polygon(&self) -> Vec<(i64, i64)> {
        let (first, last) = self.columns();
        let mut raw = Vec::new();
        for i in first..=last {
            let t = self.half_at(i);
            raw.push((i, -t));
            raw.push((i + 1, -t));
        }
        for i in (first..=last).rev() {
            let t = self.half_at(i);
            raw.push((i + 1, t));
            raw.push((i, t));
        }
        let mut out: Vec<(i64, i64)> = Vec::new();
        for p in raw {
            if out.last() != Some(&p) {
                out.push(p);
            }
        }
        while out.len() > 1 && out.first() == out.last() {
            out.pop();
        }
        // Drop collinear vertices until none is left (cyclically).
        loop {
            let k = out.len();
            let drop = (0..k).find(|&m| {
                let (p, q, r) = (out[(m + k - 1) % k], out[m], out[(m + 1) % k]);
                (q.0 - p.0) * (r.1 - q.1) - (q.1 - p.1) * (r.0 - q.0) == 0
            });
            match drop {
                Some(m) => {
                    out.remove(m);
                }
                None => break,
            }
        }
        out
    }

    /// The outline's straight pieces, each labelled with the side it faces.
    pub fn edges(&self) -> Vec<Edge> {
        let p = self.polygon();
        let k = p.len();
        (0..k)
            .map(|m| {
                let (a, b) = (p[m], p[(m + 1) % k]);
                let side = if a.1 == b.1 {
                    if a.1 > 0 { Side::Top } else { Side::Bottom }
                } else if a.0 > 0 {
                    Side::Right
                } else {
                    Side::Left
                };
                Edge { a, b, side }
            })
            .collect()
    }

    /// The outline's length in cell widths (4n for the true circle: fact stair.perimeter.four).
    pub fn perimeter_cells(&self) -> i64 {
        self.edges().iter().map(Edge::cells).sum()
    }

    /// Where 17_4's slide puts `edge`: a horizontal piece moves vertically to the top or bottom
    /// of the bounding box, a vertical piece sideways to its left or right. Returns the shift in
    /// cell units.
    pub fn slide(&self, edge: &Edge) -> (i64, i64) {
        let (x0, x1, y0, y1) = self.bbox();
        match edge.side {
            Side::Top => (0, y1 - edge.a.1),
            Side::Bottom => (0, y0 - edge.a.1),
            Side::Left => (x0 - edge.a.0, 0),
            Side::Right => (x1 - edge.a.0, 0),
        }
    }

    /// The outline's corners in diameters. The outline's farthest points from the centre are
    /// among them, since a segment's largest distance is at an end.
    pub fn corners(&self) -> Vec<(f64, f64)> {
        let h = self.h();
        self.polygon()
            .iter()
            .map(|&(x, y)| (x as f64 * h, y as f64 * h))
            .collect()
    }

    /// The outline points where a radial gap can peak, D: every corner (once) and each piece's
    /// foot of the perpendicular from the centre, when it falls inside the piece.
    fn gap_candidates(&self) -> Vec<(f64, f64)> {
        let h = self.h();
        let mut out = Vec::new();
        for e in self.edges() {
            let (a, b) = (
                (e.a.0 as f64 * h, e.a.1 as f64 * h),
                (e.b.0 as f64 * h, e.b.1 as f64 * h),
            );
            out.push(a);
            let (dx, dy) = (b.0 - a.0, b.1 - a.1);
            let t = -(a.0 * dx + a.1 * dy) / (dx * dx + dy * dy);
            if t > 0.0 && t < 1.0 {
                out.push((a.0 + t * dx, a.1 + t * dy));
            }
        }
        out
    }

    /// The largest radial gap between outline and circle: (gap in D, the first outline point
    /// along the outline where it occurs). The circle's symmetry makes it occur at several points:
    /// see [`Staircase::largest_gap_points`].
    pub fn largest_gap(&self) -> (f64, (f64, f64)) {
        let r = self.radius();
        self.gap_candidates()
            .into_iter()
            .map(|p| ((p.0.hypot(p.1) - r).abs(), p))
            .fold((-1.0, (0.0, 0.0)), |best, c| if c.0 > best.0 { c } else { best })
    }

    /// Every outline point whose radial gap is within `tol` D of the largest.
    pub fn largest_gap_points(&self, tol: f64) -> Vec<(f64, f64)> {
        let (r, (best, _)) = (self.radius(), self.largest_gap());
        self.gap_candidates()
            .into_iter()
            .filter(|p| (best - (p.0.hypot(p.1) - r).abs()).abs() <= tol)
            .collect()
    }

    /// The outline's largest distance from the centre, D, and the first corner where it occurs.
    pub fn farthest(&self) -> (f64, (f64, f64)) {
        self.corners()
            .into_iter()
            .map(|p| (p.0.hypot(p.1), p))
            .fold((-1.0, (0.0, 0.0)), |best, c| if c.0 > best.0 { c } else { best })
    }

    /// Every corner within `tol` D of the outline's largest distance from the centre.
    pub fn farthest_points(&self, tol: f64) -> Vec<(f64, f64)> {
        let (far, _) = self.farthest();
        self.corners()
            .into_iter()
            .filter(|p| (far - p.0.hypot(p.1)).abs() <= tol)
            .collect()
    }

    /// Whether every outline point lies strictly inside the circle of radius `R_num / R_den`,
    /// decided exactly: every corner is, in integers.
    pub fn strictly_inside(&self, (big_num, big_den): (i64, i64)) -> bool {
        // corner (x, y) in cell units: (x^2 + y^2) / n^2 < (big_num / big_den)^2.
        let rhs = self.n * self.n * big_num * big_num;
        self.polygon()
            .iter()
            .all(|&(x, y)| (x * x + y * y) * big_den * big_den < rhs)
    }
}

// ------------------------------------------------------------ the magnifier

/// Of tied points, the one farthest right, then lowest: the scene's magnifier inset sits to the
/// right of the figure, so its lens takes the tied corner nearest the inset.
pub fn rightmost_lowest(points: &[(f64, f64)]) -> Option<(f64, f64)> {
    points
        .iter()
        .copied()
        .max_by(|a, b| a.0.total_cmp(&b.0).then(b.1.total_cmp(&a.1)))
}

/// A square window in diameters: centre and half side.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Window {
    pub cx: f64,
    pub cy: f64,
    pub half: f64,
}

impl Window {
    pub fn contains(&self, p: (f64, f64)) -> bool {
        (p.0 - self.cx).abs() <= self.half && (p.1 - self.cy).abs() <= self.half
    }
}

/// The part of an axis-aligned segment inside `w`, if any.
pub fn clip_axis_segment(a: (f64, f64), b: (f64, f64), w: &Window) -> Option<((f64, f64), (f64, f64))> {
    let (xl, xh, yl, yh) = (w.cx - w.half, w.cx + w.half, w.cy - w.half, w.cy + w.half);
    if a.1 == b.1 {
        let (lo, hi) = (a.0.min(b.0).max(xl), a.0.max(b.0).min(xh));
        (a.1 >= yl && a.1 <= yh && lo < hi).then_some(((lo, a.1), (hi, a.1)))
    } else if a.0 == b.0 {
        let (lo, hi) = (a.1.min(b.1).max(yl), a.1.max(b.1).min(yh));
        (a.0 >= xl && a.0 <= xh && lo < hi).then_some(((a.0, lo), (a.0, hi)))
    } else {
        None
    }
}

/// The arcs of the circle of radius `r` about the origin that lie inside `w`, as angle
/// intervals (a0, a1) with a0 < a1 and a1 − a0 ≤ 2π, from where the circle crosses the window's
/// four sides.
pub fn arc_runs(r: f64, w: &Window) -> Vec<(f64, f64)> {
    let (xl, xh, yl, yh) = (w.cx - w.half, w.cx + w.half, w.cy - w.half, w.cy + w.half);
    let mut angles = Vec::new();
    for x in [xl, xh] {
        if x.abs() < r {
            let y = (r * r - x * x).sqrt();
            for yy in [y, -y] {
                if yy >= yl && yy <= yh {
                    angles.push(yy.atan2(x).rem_euclid(TAU));
                }
            }
        }
    }
    for y in [yl, yh] {
        if y.abs() < r {
            let x = (r * r - y * y).sqrt();
            for xx in [x, -x] {
                if xx >= xl && xx <= xh {
                    angles.push(y.atan2(xx).rem_euclid(TAU));
                }
            }
        }
    }
    angles.sort_by(f64::total_cmp);
    angles.dedup_by(|a, b| (*a - *b).abs() < 1e-12);
    let on = |a: f64| w.contains((r * a.cos(), r * a.sin()));
    if angles.is_empty() {
        return if on(0.0) { vec![(0.0, TAU)] } else { Vec::new() };
    }
    let k = angles.len();
    (0..k)
        .filter_map(|m| {
            let a0 = angles[m];
            let a1 = if m + 1 < k { angles[m + 1] } else { angles[0] + TAU };
            on(0.5 * (a0 + a1)).then_some((a0, a1))
        })
        .collect()
}

/// Points along the circle of radius `r` from angle `a0` to `a1`, at most `step` radians apart,
/// both ends included.
pub fn arc_points(r: f64, a0: f64, a1: f64, step: f64) -> Vec<(f64, f64)> {
    let k = (((a1 - a0) / step).ceil() as usize).max(1);
    (0..=k)
        .map(|m| {
            let a = if m == k {
                a1
            } else {
                a0 + (a1 - a0) * m as f64 / k as f64
            };
            (r * a.cos(), r * a.sin())
        })
        .collect()
}

// ------------------------------------------------------------- cut cells

/// ∫ sqrt(r² − x²) dx, the antiderivative, for |x| ≤ r.
fn primitive(r: f64, x: f64) -> f64 {
    let x = x.clamp(-r, r);
    0.5 * (x * (r * r - x * x).max(0.0).sqrt() + r * r * (x / r).asin())
}

/// The exact area of the disc of radius `r` about the origin inside [x0, x1] × [y0, y1], by
/// splitting x where the disc's edge meets y0 or y1 and integrating each piece in closed form.
pub fn disc_rect_area(r: f64, x0: f64, x1: f64, y0: f64, y1: f64) -> f64 {
    let mut cuts = vec![x0, x1, -r, r];
    for y in [y0, y1] {
        if y.abs() < r {
            let x = (r * r - y * y).sqrt();
            cuts.extend([x, -x]);
        }
    }
    cuts.retain(|&x| x >= x0 && x <= x1);
    cuts.sort_by(f64::total_cmp);
    cuts.dedup();
    let mut area = 0.0;
    for pair in cuts.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let m = 0.5 * (a + b);
        if m.abs() >= r {
            continue;
        }
        let s = (r * r - m * m).sqrt();
        let upper_is_wall = y1 > s;
        let lower_is_wall = y0 < -s;
        let up = if upper_is_wall { s } else { y1 };
        let low = if lower_is_wall { -s } else { y0 };
        if up <= low {
            continue;
        }
        let wall = primitive(r, b) - primitive(r, a);
        let top = if upper_is_wall { wall } else { y1 * (b - a) };
        let bottom = if lower_is_wall { -wall } else { y0 * (b - a) };
        area += top - bottom;
    }
    area
}

/// A cell the true wall passes through, with the share of it that is fluid.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CutCell {
    pub i: i64,
    pub j: i64,
    /// Fluid area over cell area, strictly between 0 and 1.
    pub fluid: f64,
}

/// Every cell of an `n`-per-diameter grid that the circle of radius 1/2 D crosses. The test is
/// exact in integers: the circle passes through the cell's interior when the cell's nearest
/// point is strictly inside and its farthest corner strictly outside.
pub fn cut_cells(n: i64) -> Vec<CutCell> {
    let h = 1.0 / n as f64;
    let r2 = n * n; // (2 r n)^2 with r = 1/2, against 4 d^2 in cell units
    let mut out = Vec::new();
    for i in -n..n {
        for j in -n..n {
            let near = |lo: i64| {
                if lo > 0 {
                    lo
                } else if lo + 1 < 0 {
                    lo + 1
                } else {
                    0
                }
            };
            let (nx, ny) = (near(i), near(j));
            let far = |lo: i64| lo.abs().max((lo + 1).abs());
            let (fx, fy) = (far(i), far(j));
            if 4 * (nx * nx + ny * ny) < r2 && r2 < 4 * (fx * fx + fy * fy) {
                let (x0, y0) = (i as f64 * h, j as f64 * h);
                let solid = disc_rect_area(0.5, x0, x0 + h, y0, y0 + h);
                out.push(CutCell {
                    i,
                    j,
                    fluid: 1.0 - solid / (h * h),
                });
            }
        }
    }
    out
}

/// A cell face the true wall crosses, cell units, with the part of it that is open (fluid).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CutFace {
    pub a: (i64, i64),
    pub b: (i64, i64),
    /// The solid part, as (from, to) along the face in diameters; the rest of the face is open.
    pub solid: ((f64, f64), (f64, f64)),
    /// Open length over face length, strictly between 0 and 1.
    pub open: f64,
}

/// Every face of an `n`-per-diameter grid that the circle of radius 1/2 D crosses: one end
/// strictly inside, the other strictly outside, decided in integers. A face that only touches
/// the circle at an end (at 8 cells, the four lattice points (±4, 0) and (0, ±4)) is not cut.
pub fn cut_faces(n: i64) -> Vec<CutFace> {
    let h = 1.0 / n as f64;
    let r2 = n * n;
    let inside = |x: i64, y: i64| 4 * (x * x + y * y) < r2;
    let outside = |x: i64, y: i64| 4 * (x * x + y * y) > r2;
    let mut out = Vec::new();
    let mut push = |a: (i64, i64), b: (i64, i64)| {
        let crosses = (inside(a.0, a.1) && outside(b.0, b.1)) || (outside(a.0, a.1) && inside(b.0, b.1));
        if !crosses {
            return;
        }
        // The chord of the disc along this face's line.
        let (fa, fb) = ((a.0 as f64 * h, a.1 as f64 * h), (b.0 as f64 * h, b.1 as f64 * h));
        let solid = if a.1 == b.1 {
            let s = (0.25 - fa.1 * fa.1).sqrt();
            let (lo, hi) = (fa.0.max(-s), fb.0.min(s));
            ((lo, fa.1), (hi, fa.1))
        } else {
            let s = (0.25 - fa.0 * fa.0).sqrt();
            let (lo, hi) = (fa.1.max(-s), fb.1.min(s));
            ((fa.0, lo), (fa.0, hi))
        };
        let len = (solid.1.0 - solid.0.0) + (solid.1.1 - solid.0.1);
        out.push(CutFace {
            a,
            b,
            solid,
            open: 1.0 - len / h,
        });
    };
    for x in -n..=n {
        for y in -n..n {
            push((x, y), (x, y + 1));
        }
    }
    for y in -n..=n {
        for x in -n..n {
            push((x, y), (x + 1, y));
        }
    }
    out
}

// --------------------------------------------------------------- the channel

/// ST96 Fig. 1 and §2.2 (facts st.length, st.height.diameter, st.gaps; cl01 dfg.gap.*), metres.
pub mod channel {
    pub const LENGTH: f64 = 2.2;
    pub const HEIGHT: f64 = 0.41;
    pub const DIAMETER: f64 = 0.1;
    /// The cylinder's centre from the inlet's lower corner.
    pub const CENTRE: (f64, f64) = (0.2, 0.2);

    pub fn gap_below() -> f64 {
        CENTRE.1 - DIAMETER / 2.0
    }

    pub fn gap_above() -> f64 {
        HEIGHT - CENTRE.1 - DIAMETER / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::f64::consts::PI;

    fn cell_set(s: &Staircase) -> HashSet<(i64, i64)> {
        let n = s.n;
        let mut out = HashSet::new();
        for i in -n..n {
            for j in -n..n {
                if s.solid(i, j) {
                    out.insert((i, j));
                }
            }
        }
        out
    }

    /// Unit solid–fluid edges counted straight from the cell set, as cl01's `edges` does.
    fn unit_edges(cells: &HashSet<(i64, i64)>) -> usize {
        let mut k = 0;
        for &(i, j) in cells {
            for (di, dj) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                if !cells.contains(&(i + di, j + dj)) {
                    k += 1;
                }
            }
        }
        k
    }

    /// Membership without the column shortcut: the middle strictly inside, in integers.
    fn direct(n: i64, (p, q): (i64, i64), i: i64, j: i64) -> bool {
        ((2 * i + 1).pow(2) + (2 * j + 1).pow(2)) * q * q < 4 * n * n * p * p
    }

    #[test]
    fn cell_counts_match_cl01() {
        let want = [52, 208, 812, 3228, 12892];
        for (&n, &cells) in LEVELS.iter().zip(&want) {
            let s = Staircase::new(n, TRUE_RADIUS);
            assert_eq!(s.cells(), cells, "n = {n}");
            for i in -n..n {
                for j in -n..n {
                    assert_eq!(s.solid(i, j), direct(n, TRUE_RADIUS, i, j));
                }
            }
        }
        assert_eq!(Staircase::new(256, TRUE_RADIUS).cells(), 51468);
    }

    #[test]
    fn outline_is_four_diameters_at_every_level() {
        for n in LEVELS.into_iter().chain([256]) {
            let s = Staircase::new(n, TRUE_RADIUS);
            assert_eq!(s.perimeter_cells(), 4 * n, "n = {n}");
            assert_eq!(unit_edges(&cell_set(&s)) as i64, 4 * n, "n = {n}");
            assert_eq!(
                s.bbox(),
                (-n / 2, n / 2, -n / 2, n / 2),
                "n = {n}: the widest rows span D"
            );
        }
    }

    #[test]
    fn rows_and_columns_are_unbroken() {
        for n in LEVELS {
            let s = Staircase::new(n, TRUE_RADIUS);
            let cells = cell_set(&s);
            for k in -n..n {
                let row: Vec<i64> = (-n..n).filter(|&i| cells.contains(&(i, k))).collect();
                let col: Vec<i64> = (-n..n).filter(|&j| cells.contains(&(k, j))).collect();
                for run in [row, col] {
                    if let (Some(a), Some(b)) = (run.first(), run.last()) {
                        assert_eq!((b - a + 1) as usize, run.len(), "n = {n}, line {k}");
                    }
                }
            }
        }
    }

    #[test]
    fn polygon_is_the_outline() {
        for n in LEVELS {
            let s = Staircase::new(n, TRUE_RADIUS);
            let p = s.polygon();
            // shoelace area = solid cells
            let k = p.len();
            let twice: i64 = (0..k)
                .map(|m| p[m].0 * p[(m + 1) % k].1 - p[(m + 1) % k].0 * p[m].1)
                .sum();
            assert_eq!(twice, 2 * s.cells(), "n = {n}: counterclockwise, area = cells");
            for e in s.edges() {
                assert!(e.a.0 == e.b.0 || e.a.1 == e.b.1, "axis-aligned");
                let mid = (e.a.0 + e.b.0, e.a.1 + e.b.1); // twice the midpoint
                let (nx, ny) = e.normal();
                // the cell just outside the piece's midpoint is fluid, just inside solid
                // Twice-scaled probe points one half cell either side of the piece's midpoint;
                // halving with floor names the cell each one lies in.
                let probe = |sign: i64| {
                    let (x2, y2) = (mid.0 + sign * nx, mid.1 + sign * ny);
                    s.solid(x2.div_euclid(2), y2.div_euclid(2))
                };
                assert!(!probe(1) && probe(-1), "n = {n}: outward normal of {e:?}");
            }
        }
    }

    #[test]
    fn slide_makes_the_circumscribed_square() {
        for n in LEVELS {
            let s = Staircase::new(n, TRUE_RADIUS);
            let h = n / 2;
            let mut cover: Vec<Vec<(i64, i64)>> = vec![Vec::new(); 4];
            for e in s.edges() {
                let (dx, dy) = s.slide(&e);
                let (a, b) = ((e.a.0 + dx, e.a.1 + dy), (e.b.0 + dx, e.b.1 + dy));
                let (k, lo, hi, at) = match e.side {
                    Side::Top => (0, a.0.min(b.0), a.0.max(b.0), (a.1, h)),
                    Side::Bottom => (1, a.0.min(b.0), a.0.max(b.0), (a.1, -h)),
                    Side::Left => (2, a.1.min(b.1), a.1.max(b.1), (a.0, -h)),
                    Side::Right => (3, a.1.min(b.1), a.1.max(b.1), (a.0, h)),
                };
                assert_eq!(at.0, at.1, "n = {n}: {e:?} lands on its side of the square");
                cover[k].push((lo, hi));
            }
            for side in &mut cover {
                side.sort();
                let mut at = -h;
                for &(lo, hi) in side.iter() {
                    assert_eq!(lo, at, "n = {n}: no gap and no overlap");
                    at = hi;
                }
                assert_eq!(at, h, "n = {n}: the side is covered to its end");
            }
        }
    }

    #[test]
    fn gap_stays_within_half_a_cell_diagonal() {
        let want = [0.472, 0.602, 0.643, 0.650, 0.661]; // cl01 stair.n*, cell widths
        for (&n, &w) in LEVELS.iter().zip(&want) {
            let s = Staircase::new(n, TRUE_RADIUS);
            let (gap, at) = s.largest_gap();
            let cells = gap * n as f64;
            assert!(cells <= std::f64::consts::FRAC_1_SQRT_2, "n = {n}");
            assert!((cells - w).abs() < 5e-4, "n = {n}: {cells}");
            assert!(at.0.hypot(at.1) > 0.5, "n = {n}: the largest gap is at an outer corner");
        }
    }

    #[test]
    fn typed_circle_staircase_stays_inside_from_32_cells() {
        for n in WRONG_LEVELS {
            let s = Staircase::new(n, TYPED_RADIUS);
            assert!(s.strictly_inside(TRUE_RADIUS), "n = {n}");
            let (far, _) = s.farthest();
            assert!(far < 0.475 + std::f64::consts::FRAC_1_SQRT_2 / n as f64, "n = {n}");
        }
        assert!(
            !Staircase::new(16, TYPED_RADIUS).strictly_inside(TRUE_RADIUS),
            "16 crosses"
        );
    }

    #[test]
    fn clipping() {
        // binary-exact window: x in [0.125, 0.375], y in [-0.5, -0.25]
        let w = Window {
            cx: 0.25,
            cy: -0.375,
            half: 0.125,
        };
        assert_eq!(
            clip_axis_segment((0.0, -0.4375), (0.5, -0.4375), &w),
            Some(((0.125, -0.4375), (0.375, -0.4375)))
        );
        assert_eq!(clip_axis_segment((0.0, -0.125), (0.5, -0.125), &w), None);
        assert_eq!(
            clip_axis_segment((0.25, -0.75), (0.25, 0.0), &w),
            Some(((0.25, -0.5), (0.25, -0.25)))
        );
        let runs = arc_runs(0.5, &w);
        assert_eq!(runs.len(), 1);
        for &(a0, a1) in &runs {
            for p in arc_points(0.5, a0, a1, 0.01) {
                assert!((p.0 - w.cx).abs() <= w.half + 1e-12 && (p.1 - w.cy).abs() <= w.half + 1e-12);
            }
        }
        assert_eq!(
            arc_runs(
                0.5,
                &Window {
                    cx: 0.0,
                    cy: 0.0,
                    half: 0.6
                }
            ),
            vec![(0.0, TAU)]
        );
        assert!(
            arc_runs(
                0.5,
                &Window {
                    cx: 0.0,
                    cy: 0.0,
                    half: 0.1
                }
            )
            .is_empty()
        );
    }

    #[test]
    fn cut_cells_are_exact() {
        // whole-cell sums over the grid reproduce the disc's area, pi / 4
        let n = 8;
        let h = 1.0 / n as f64;
        let mut total = 0.0;
        for i in -n..n {
            for j in -n..n {
                total += disc_rect_area(0.5, i as f64 * h, (i + 1) as f64 * h, j as f64 * h, (j + 1) as f64 * h);
            }
        }
        assert!((total - PI / 4.0).abs() < 1e-12, "{total}");
        assert!((disc_rect_area(0.5, -1.0, 1.0, -1.0, 1.0) - PI / 4.0).abs() < 1e-14);
        assert!((disc_rect_area(0.5, 0.0, 1.0, 0.0, 1.0) - PI / 16.0).abs() < 1e-14);
        let cut = cut_cells(n);
        for c in &cut {
            assert!(c.fluid > 0.0 && c.fluid < 1.0, "{c:?}");
        }
        let faces = cut_faces(n);
        for f in &faces {
            assert!(f.open > 0.0 && f.open < 1.0, "{f:?}");
        }
        // independent count: check_boundary01.py
        assert_eq!(cut.len(), 28);
        assert_eq!(faces.len(), 24);
    }

    #[test]
    fn channel_gaps() {
        assert!((channel::gap_below() - 0.15).abs() < 1e-12);
        assert!((channel::gap_above() - 0.16).abs() < 1e-12);
        assert!((channel::gap_above() - channel::gap_below() - 0.01).abs() < 1e-12);
    }

    #[test]
    fn tied_extremes_and_the_lens_corners() {
        // independent counts and picks: check_boundary01.py L4
        for (n, ties) in LEVELS.into_iter().zip([8, 8, 8, 16, 16]) {
            let s = Staircase::new(n, TRUE_RADIUS);
            let pts = s.largest_gap_points(1e-12);
            assert_eq!(pts.len(), ties, "n = {n}");
            let (gap, _) = s.largest_gap();
            for p in &pts {
                assert!(((p.0.hypot(p.1) - 0.5).abs() - gap).abs() <= 1e-12);
            }
        }
        let s = Staircase::new(32, TRUE_RADIUS);
        assert_eq!(rightmost_lowest(&s.largest_gap_points(1e-12)), Some((0.4375, -0.28125)));
        for (n, ties) in WRONG_LEVELS.into_iter().zip([16, 8, 24]) {
            assert_eq!(
                Staircase::new(n, TYPED_RADIUS).farthest_points(1e-12).len(),
                ties,
                "n = {n}"
            );
        }
        let t = Staircase::new(32, TYPED_RADIUS);
        assert_eq!(rightmost_lowest(&t.farthest_points(1e-12)), Some((0.46875, -0.15625)));
        assert_eq!(rightmost_lowest(&[]), None);
    }
}
