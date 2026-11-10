//! §10.2's analytic fill, and the question of whether it maps onto a GPU
//! (fm-orn — the G0-8 follow-on named in that report's §5).
//!
//! ## Why this module exists and [`crate::fill`] does not answer the question
//!
//! [`crate::fill`] is G0-6's *defined stand-in*: coverage exact in x and
//! supersampled in y, chosen because a determinism spike needs an algorithm two
//! platforms can agree on, not the algorithm W5 will ship. It says so in its own
//! first paragraph. This module is the other one — **§10.2 as written**:
//!
//! > Tiled scanline nonzero-winding coverage evaluated analytically on the
//! > curves: quadratic segments y-monotonized by splitting at the
//! > vertical-tangent parameter; per scanline, exact segment intersections from
//! > the closed-form quadratic root; signed trapezoidal area accumulation per
//! > cell — no triangulation, no signed-alpha tricks, no orientation
//! > bookkeeping.
//!
//! The reason it is spiked before fm-gw7 rather than during fm-5oi is the one
//! G0-8 gave: the stroke stage is per-*pixel* independent, which is why it fell
//! out as one-thread-per-pixel with no synchronization and no atomics. A fill is
//! per-*scanline*, and its accumulation is order-dependent. An accumulation
//! whose ordering matters is exactly the shape that does not fall out of the
//! stroke kernel's structure, so the IR could freeze around a layout that cannot
//! carry it.
//!
//! ## Two split axes, not one
//!
//! The plan says "y-monotonized by splitting at the vertical-tangent parameter",
//! and those are two different parameters. y-monotone — y monotone along the
//! piece, so a scanline meets it at most once — is the **horizontal**-tangent
//! parameter, `dy/dt = 0`. The vertical-tangent parameter, `dx/dt = 0`, gives
//! x-monotone pieces.
//!
//! This spike splits at **both**, and the second split is not decoration: the
//! per-cell trapezoid needs the exact parameter at which the piece crosses each
//! *column* boundary, and "the closed-form root" is only a well-defined answer
//! when x is monotone too. Splitting at one axis and solving as if both held
//! would silently pick one of two roots. So the finding for fm-gw7 is that
//! §10.2's sentence names one split and the algorithm needs two — see
//! [`MonoTable`].
//!
//! ## The tile carry, which is the part a tiled fill cannot skip
//!
//! Coverage inside a tile is not a function of the geometry inside that tile. A
//! scanline entering the tile's left edge already carries whatever winding the
//! path deposited to the left of it, so every row needs
//!
//! ```text
//! carry(row) = Σ over pieces of (signed dy of the part of the piece in this
//!              row lying left of the tile)
//! ```
//!
//! before the first cell is read. This is computed in closed form (one root
//! solve per piece), never by walking cells from the frame's left edge — and it
//! is why the fill loops over **every** piece of a path, not only the pieces
//! whose bounds meet the tile. Binning is unaffected: a path that encloses any
//! pixel of a tile has hull bounds containing that tile, so the existing
//! slab-keyed [`crate::ir::RenderIr::bin`] already lists it.
//!
//! ## The two dispatch shapes this module defines
//!
//! Both are built from one routine, [`accumulate_piece_row`], because the
//! per-pixel form is the scanline form with a one-cell window:
//!
//! - [`fill_row`] — the scanline shape. One accumulator of `tile + 1` cells per
//!   row, one serial prefix sum along x. This is §10.2 literally, and on the GPU
//!   it is one thread per scanline.
//! - [`coverage_at_cell`] — the per-pixel shape. Each pixel sums, over the
//!   pieces, "the winding that passed to my left" plus "my own cell's trapezoid"
//!   and needs no accumulator and no scan. On the GPU it is one thread per
//!   pixel: the *same* dispatch shape as the stroke kernel.
//!
//! In exact arithmetic the two agree term for term; they differ only in the
//! association of the sum, which is why both are measured rather than assumed
//! equal.

use crate::cpu::Precision;
use crate::ir::{DrawKind, PathHeader, RenderIr, Style};

// ---------------------------------------------------------------- arithmetic

/// The scalar operations the fill needs, so the reference (`f64`) and the
/// annex's arithmetic floor (`f32`) run **one** expression tree rather than two
/// hand-kept transcriptions.
///
/// G0-8 wrote `distance_to_quadratic` twice, once per width, and the two copies
/// were correct — but the mirror rule survived that only because a human held
/// it. Here the two instantiations are the same source by construction, which
/// leaves exactly one hand-mirrored copy to worry about: the MSL kernel.
pub trait Real:
    Copy
    + PartialOrd
    + core::ops::Add<Output = Self>
    + core::ops::Sub<Output = Self>
    + core::ops::Mul<Output = Self>
    + core::ops::Div<Output = Self>
    + core::ops::Neg<Output = Self>
{
    /// `0`.
    const ZERO: Self;
    /// `1`.
    const ONE: Self;
    /// `2`.
    const TWO: Self;
    /// `4`.
    const FOUR: Self;
    /// `0.5`.
    const HALF: Self;
    /// The root solver's degeneracy tolerance, **relative to the polynomial's
    /// own scale and sized to this width** — G0-8's finding F8, which cost a
    /// day when a single absolute constant was shared between an f64 engine and
    /// an f32 one.
    const DEGENERATE_REL: Self;

    /// Widen an `f32` (the IR's storage width) into the working width.
    fn from_f32(v: f32) -> Self;
    /// Widen a pixel index.
    fn from_u32(v: u32) -> Self;
    /// Narrow to `f64` for the surface and the tests.
    fn to_f64(self) -> f64;
    /// Truncate toward negative infinity.
    fn to_i64(self) -> i64;
    /// Absolute value.
    fn abs(self) -> Self;
    /// Square root.
    fn sqrt(self) -> Self;
    /// Largest integer not greater than `self`.
    fn floor(self) -> Self;
    /// Smallest integer not less than `self`.
    fn ceil(self) -> Self;
}

impl Real for f64 {
    const ZERO: f64 = 0.0;
    const ONE: f64 = 1.0;
    const TWO: f64 = 2.0;
    const FOUR: f64 = 4.0;
    const HALF: f64 = 0.5;
    const DEGENERATE_REL: f64 = 1e-14;

    fn from_f32(v: f32) -> f64 {
        v as f64
    }
    fn from_u32(v: u32) -> f64 {
        v as f64
    }
    fn to_f64(self) -> f64 {
        self
    }
    fn to_i64(self) -> i64 {
        self as i64
    }
    fn abs(self) -> f64 {
        f64::abs(self)
    }
    fn sqrt(self) -> f64 {
        f64::sqrt(self)
    }
    fn floor(self) -> f64 {
        f64::floor(self)
    }
    fn ceil(self) -> f64 {
        f64::ceil(self)
    }
}

impl Real for f32 {
    const ZERO: f32 = 0.0;
    const ONE: f32 = 1.0;
    const TWO: f32 = 2.0;
    const FOUR: f32 = 4.0;
    const HALF: f32 = 0.5;
    const DEGENERATE_REL: f32 = 1e-6;

    fn from_f32(v: f32) -> f32 {
        v
    }
    fn from_u32(v: u32) -> f32 {
        v as f32
    }
    fn to_f64(self) -> f64 {
        self as f64
    }
    fn to_i64(self) -> i64 {
        self as i64
    }
    fn abs(self) -> f32 {
        f32::abs(self)
    }
    fn sqrt(self) -> f32 {
        f32::sqrt(self)
    }
    fn floor(self) -> f32 {
        f32::floor(self)
    }
    fn ceil(self) -> f32 {
        f32::ceil(self)
    }
}

/// The smaller of two values, written as a comparison rather than `f64::min`.
///
/// `f64::min` and MSL's `min` agree on ordinary values and disagree about NaN
/// and signed zero; after G0-8's `sign(0)` incident (finding F7) the house rule
/// is that any predicate the shader also has to spell gets spelled the same way
/// on both sides.
fn fmin<T: Real>(a: T, b: T) -> T {
    if a < b { a } else { b }
}

/// The larger of two values. See [`fmin`].
fn fmax<T: Real>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

/// `+1` for zero and positives, `-1` for negatives — G0-8's `sign_or_positive`,
/// at the working width.
fn sign_or_positive<T: Real>(x: T) -> T {
    if x >= T::ZERO { T::ONE } else { -T::ONE }
}

/// Real roots of `a2 t² + a1 t + a0`, falling through to the linear case.
///
/// A transcription of [`crate::sdf::solve_quadratic`] at an arbitrary width,
/// including its stable pairing and its relative degeneracy test. The f64
/// instantiation is asserted equal to the original in this module's tests, so
/// the two can never drift apart unnoticed.
fn solve_quadratic<T: Real>(a2: T, a1: T, a0: T, out: &mut [T; 2]) -> usize {
    let scale = fmax(fmax(a2.abs(), a1.abs()), a0.abs());
    if scale <= T::ZERO {
        return 0;
    }
    let tol = T::DEGENERATE_REL * scale;
    if a2.abs() <= tol {
        if a1.abs() <= tol {
            return 0;
        }
        out[0] = -a0 / a1;
        return 1;
    }
    let disc = a1 * a1 - T::FOUR * a2 * a0;
    if disc < T::ZERO {
        return 0;
    }
    let s = disc.sqrt();
    let q = -T::HALF * (a1 + sign_or_positive(a1) * s);
    if q == T::ZERO {
        out[0] = T::ZERO;
        out[1] = T::ZERO;
        return 2;
    }
    out[0] = q / a2;
    out[1] = a0 / q;
    2
}

// ------------------------------------------------------------- monotone table

/// A quadratic piece that is monotone in **both** axes.
///
/// Control points in screen-space pixels, stored at the IR's `f32` width so the
/// CPU reference and the Metal engine read the identical bytes and any
/// divergence between them is provably arithmetic rather than input.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MonoPiece {
    /// Start anchor.
    pub p0: [f32; 2],
    /// Control handle.
    pub p1: [f32; 2],
    /// End anchor.
    pub p2: [f32; 2],
}

/// Below this parameter distance from an endpoint a split is skipped.
///
/// A split at `t = 1e-12` yields a piece whose y-extent is smaller than any
/// scanline band, so it can never contribute coverage — it only costs a table
/// row and a per-row rejection. The threshold is on the *parameter*, which is
/// dimensionless, so it needs no relation to screen scale.
const SPLIT_EPS: f64 = 1e-9;

/// The fill's derived geometry: every path's segments cut into doubly-monotone
/// pieces, plus the per-path ranges.
///
/// **This is a second table, not a replacement for the IR's `SegmentTable`** —
/// which is the layout verdict fm-gw7 wants from this spike. Strokes read
/// unsplit segments and would be wrong if they did not: §10.3 interpolates width
/// and colour by the normalized arc-length span `(s0, s1)` each segment carries,
/// and splitting a segment invalidates that span unless it is recomputed, while
/// the stroke's nearest-point search over more, shorter pieces is strictly more
/// work for the same answer. So the fill gets its own derived table, keyed — like
/// every other derived artifact in §10.8 — by the **geometry** revision alone: a
/// colour change or a camera move must not rebuild it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MonoTable {
    /// All pieces of all paths, contiguous, grouped by path.
    pub pieces: Vec<MonoPiece>,
    /// `(first_piece, piece_count)` for path `i`, parallel to
    /// [`RenderIr::paths`].
    pub ranges: Vec<(u32, u32)>,
}

impl MonoTable {
    /// Derive the table from an IR's segments.
    pub fn build(ir: &RenderIr) -> MonoTable {
        let mut pieces = Vec::with_capacity(ir.segments.len() * 2);
        let mut ranges = Vec::with_capacity(ir.paths.len());
        for path in &ir.paths {
            let first = pieces.len() as u32;
            for i in path.first_segment..(path.first_segment + path.segment_count) {
                let seg = &ir.segments[i as usize];
                split_monotone(seg.p0, seg.p1, seg.p2, &mut pieces);
            }
            ranges.push((first, pieces.len() as u32 - first));
        }
        MonoTable { pieces, ranges }
    }

    /// The pieces belonging to path `index`.
    pub fn pieces_of(&self, index: usize) -> &[MonoPiece] {
        let (first, count) = self.ranges[index];
        &self.pieces[first as usize..(first + count) as usize]
    }
}

/// The parameter of a quadratic component's extremum, if it lies strictly
/// inside the segment.
///
/// `v(t) = a + b t + c t²` has `v'(t) = b + 2 c t`, so the extremum is at
/// `-b / 2c`. A vanishing `c` is a straight component with no extremum at all,
/// which is the common case (every line, every glyph stem) and must not divide.
fn extremum(v0: f64, v1: f64, v2: f64) -> Option<f64> {
    let b = 2.0 * (v1 - v0);
    let c = (v2 - v1) - (v1 - v0);
    let scale = v0.abs().max(v1.abs()).max(v2.abs()).max(1.0);
    if c.abs() <= 1e-14 * scale {
        return None;
    }
    let t = -b / (2.0 * c);
    if t > SPLIT_EPS && t < 1.0 - SPLIT_EPS {
        Some(t)
    } else {
        None
    }
}

/// Split one segment at its y- and x-extrema, appending doubly-monotone pieces.
///
/// A quadratic has at most one extremum per axis, so this appends at most three
/// pieces — a bound the device layout can rely on when sizing the table.
fn split_monotone(p0: [f32; 2], p1: [f32; 2], p2: [f32; 2], out: &mut Vec<MonoPiece>) {
    let d = |p: [f32; 2]| [p[0] as f64, p[1] as f64];
    let (a, h, b) = (d(p0), d(p1), d(p2));

    let mut ts: [f64; 2] = [0.0; 2];
    let mut n = 0;
    if let Some(t) = extremum(a[1], h[1], b[1]) {
        ts[n] = t;
        n += 1;
    }
    if let Some(t) = extremum(a[0], h[0], b[0]) {
        ts[n] = t;
        n += 1;
    }
    if n == 2 && ts[0] > ts[1] {
        ts.swap(0, 1);
    }

    // Successive de Casteljau splits, each reparameterized into the remaining
    // right-hand piece.
    let mut cur = [a, h, b];
    let mut t_base = 0.0;
    for &t in ts.iter().take(n) {
        let local = (t - t_base) / (1.0 - t_base);
        if !(local > SPLIT_EPS && local < 1.0 - SPLIT_EPS) {
            continue;
        }
        let (left, right) = split_at(cur, local);
        out.push(piece(left));
        cur = right;
        t_base = t;
    }
    out.push(piece(cur));
}

/// de Casteljau's split of a quadratic at `t`.
fn split_at(p: [[f64; 2]; 3], t: f64) -> ([[f64; 2]; 3], [[f64; 2]; 3]) {
    let lerp = |a: [f64; 2], b: [f64; 2]| [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
    let q0 = lerp(p[0], p[1]);
    let q1 = lerp(p[1], p[2]);
    let r = lerp(q0, q1);
    ([p[0], q0, r], [r, q1, p[2]])
}

fn piece(p: [[f64; 2]; 3]) -> MonoPiece {
    let n = |v: [f64; 2]| [v[0] as f32, v[1] as f32];
    MonoPiece {
        p0: n(p[0]),
        p1: n(p[1]),
        p2: n(p[2]),
    }
}

// --------------------------------------------------------------- the fill core

/// One piece's component polynomials, `v(t) = a + b t + c t²`, at the working
/// width.
///
/// `c` is formed as `(v2 - v1) - (v1 - v0)` rather than `v0 - 2 v1 + v2`:
/// algebraically the same, numerically not, and G0-8 measured the difference on
/// near-straight screen-space curves at `f32`.
#[derive(Debug, Clone, Copy)]
struct Coeffs<T: Real> {
    ax: T,
    bx: T,
    cx: T,
    ay: T,
    by: T,
    cy: T,
}

impl<T: Real> Coeffs<T> {
    fn of(piece: &MonoPiece) -> Coeffs<T> {
        let f = T::from_f32;
        let (x0, x1, x2) = (f(piece.p0[0]), f(piece.p1[0]), f(piece.p2[0]));
        let (y0, y1, y2) = (f(piece.p0[1]), f(piece.p1[1]), f(piece.p2[1]));
        Coeffs {
            ax: x0,
            bx: T::TWO * (x1 - x0),
            cx: (x2 - x1) - (x1 - x0),
            ay: y0,
            by: T::TWO * (y1 - y0),
            cy: (y2 - y1) - (y1 - y0),
        }
    }

    fn x(&self, t: T) -> T {
        self.ax + self.bx * t + self.cx * t * t
    }

    fn y(&self, t: T) -> T {
        self.ay + self.by * t + self.cy * t * t
    }

    /// The parameter in `[t_lo, t_hi]` at which `x` reaches `target`.
    fn t_at_x(&self, target: T, t_lo: T, t_hi: T) -> T {
        invert(
            self.ax,
            self.bx,
            self.cx,
            target,
            t_lo,
            t_hi,
            self.x(t_lo),
            self.x(t_hi),
        )
    }

    /// The parameter in `[t_lo, t_hi]` at which `y` reaches `target`.
    fn t_at_y(&self, target: T, t_lo: T, t_hi: T) -> T {
        invert(
            self.ay,
            self.by,
            self.cy,
            target,
            t_lo,
            t_hi,
            self.y(t_lo),
            self.y(t_hi),
        )
    }
}

/// Invert a component that is monotone on `[t_lo, t_hi]`.
///
/// Total by construction: a target outside the endpoint values clamps to the
/// endpoint that attains the nearer extreme, and a root solve that finds nothing
/// in range falls back to the secant through the endpoints. Neither fallback is
/// a fudge — monotonicity guarantees the answer exists, and the fallbacks only
/// decide *which* representable parameter to name when the closed form has run
/// out of precision to distinguish them.
#[allow(clippy::too_many_arguments)]
fn invert<T: Real>(a: T, b: T, c: T, target: T, t_lo: T, t_hi: T, v_lo: T, v_hi: T) -> T {
    let ascending = v_hi >= v_lo;
    let vmin = fmin(v_lo, v_hi);
    let vmax = fmax(v_lo, v_hi);
    if target <= vmin {
        return if ascending { t_lo } else { t_hi };
    }
    if target >= vmax {
        return if ascending { t_hi } else { t_lo };
    }

    let mut roots = [T::ZERO; 2];
    let n = solve_quadratic(c, b, a - target, &mut roots);
    for &r in roots.iter().take(n) {
        if r >= t_lo && r <= t_hi {
            return r;
        }
    }
    // The secant fallback. `vmax > vmin` here, so the division is safe.
    t_lo + (t_hi - t_lo) * ((target - v_lo) / (v_hi - v_lo))
}

/// Deposit one sub-span's signed trapezoid into the row.
///
/// `cells` is `x_hi - x_lo + 1` wide: the extra entry catches the spill from the
/// last in-tile cell and is never read, which keeps the deposit branch-free at
/// the tile's right edge.
#[allow(clippy::too_many_arguments)]
fn deposit<T: Real>(
    cells: &mut [T],
    carry: &mut T,
    x_lo: u32,
    x_hi: u32,
    x_a: T,
    x_b: T,
    y_a: T,
    y_b: T,
) {
    let d = y_b - y_a;
    if d == T::ZERO {
        return;
    }
    let xm = T::HALF * (x_a + x_b);
    let cell_f = xm.floor();
    let cell = cell_f.to_i64();
    if cell < x_lo as i64 {
        *carry = *carry + d;
        return;
    }
    if cell >= x_hi as i64 {
        return;
    }
    let xmf = xm - cell_f;
    let i = (cell - x_lo as i64) as usize;
    cells[i] = cells[i] + d * (T::ONE - xmf);
    cells[i + 1] = cells[i + 1] + d * xmf;
}

/// Accumulate one piece's contribution to one pixel row of one tile.
///
/// This is the whole algorithm, and both dispatch shapes are built from it:
/// [`fill_row`] calls it once per piece with the tile's full width, and
/// [`coverage_at_cell`] calls it once per piece with a one-cell window. The
/// per-cell work is bounded by `x_hi - x_lo + 2` iterations, which is what lets
/// the MSL twin write a counted loop instead of a `while`.
pub fn accumulate_piece_row<T: Real>(
    piece: &MonoPiece,
    row_y: u32,
    x_lo: u32,
    x_hi: u32,
    cells: &mut [T],
    carry: &mut T,
) {
    let c = Coeffs::<T>::of(piece);
    let row = T::from_u32(row_y);
    let row_end = row + T::ONE;

    // The piece's y-extent, and the part of it inside this scanline band.
    // Monotone in y, so the endpoints are the extremes.
    let y0 = c.y(T::ZERO);
    let y1 = c.y(T::ONE);
    let band_lo = fmax(fmin(y0, y1), row);
    let band_hi = fmin(fmax(y0, y1), row_end);
    if band_hi <= band_lo {
        return;
    }
    let u = c.t_at_y(band_lo, T::ZERO, T::ONE);
    let v = c.t_at_y(band_hi, T::ZERO, T::ONE);
    let ta = fmin(u, v);
    let tb = fmax(u, v);
    if tb <= ta {
        return;
    }

    let xa = c.x(ta);
    let xb = c.x(tb);
    let increasing = xb >= xa;
    let left = T::from_u32(x_lo);
    let right = T::from_u32(x_hi);

    // Where the piece crosses the tile's vertical edges. Monotone in x, so each
    // crossing is a single closed-form root and the clamp inside `invert`
    // handles a piece that never reaches an edge.
    let t_left = c.t_at_x(left, ta, tb);
    let t_right = c.t_at_x(right, ta, tb);

    // Everything left of the tile contributes its full signed dy to the row's
    // carry — in one subtraction, never by walking cells from the frame edge.
    let (l0, l1) = if increasing {
        (ta, t_left)
    } else {
        (t_left, tb)
    };
    if l1 > l0 {
        *carry = *carry + (c.y(l1) - c.y(l0));
    }

    // The in-tile span, walked column boundary by column boundary.
    let (mut t_prev, t_end) = if increasing {
        (t_left, t_right)
    } else {
        (t_right, t_left)
    };
    if t_end <= t_prev {
        return;
    }
    // Clamped into the tile, and this is load-bearing rather than tidy. The
    // walk's span is by construction the part of the piece inside the tile, so
    // its endpoints cannot lie outside `[x_lo, x_hi]` — but `x(t_prev)` is a
    // root solve's answer re-evaluated, so it lands an ulp *outside* about half
    // the time. Without the clamp, an entry at exactly the tile's right edge
    // computes as `208.0009`, the next boundary comes out as `ceil(x) - 1 = 208`
    // — the column the walk is already in — the step makes no progress, and the
    // fallback below deposits the entire remaining span, which crosses three
    // columns, as a single trapezoid in whichever column its midpoint lands in.
    //
    // That bug was real and it was in the f64 reference, where the f32
    // transcription happened to round the other way and be right. It is the
    // reason the per-pixel form ([`coverage_at_cell`]) is worth taking
    // seriously: it never walks, so it has no stepping predicate to get wrong.
    let left = T::from_u32(x_lo);
    let right = T::from_u32(x_hi);
    let clamp_x = |v: T| {
        if v < left {
            left
        } else if v > right {
            right
        } else {
            v
        }
    };
    let mut x_prev = clamp_x(c.x(t_prev));
    let mut y_prev = c.y(t_prev);
    let x_end = clamp_x(c.x(t_end));
    let y_end = c.y(t_end);

    let steps = (x_hi - x_lo) as usize + 2;
    for _ in 0..steps {
        // The next integer column boundary strictly beyond `x_prev` in the
        // direction of travel.
        let boundary = if increasing {
            x_prev.floor() + T::ONE
        } else {
            x_prev.ceil() - T::ONE
        };
        let past = if increasing {
            boundary >= x_end
        } else {
            boundary <= x_end
        };
        let (mut t_next, mut x_next) = if past {
            (t_end, x_end)
        } else {
            // The crossing's x is the boundary *by construction*; using the
            // exact integer rather than re-evaluating `x(t)` keeps the cell
            // index unambiguous when the root solve lands an ulp to one side.
            (c.t_at_x(boundary, t_prev, t_end), boundary)
        };
        if t_next <= t_prev && !past {
            // The closed form could not separate two adjacent column
            // boundaries. Fall back to the secant in x — monotone, and it
            // advances unless the span is degenerate. Advancing to the boundary
            // matters more than the parameter's exactness here: what must never
            // happen is a multi-column span deposited as one trapezoid.
            let denom = x_end - x_prev;
            if denom != T::ZERO {
                t_next = t_prev + (t_end - t_prev) * ((boundary - x_prev) / denom);
            }
        }
        if t_next <= t_prev {
            // Genuinely degenerate: the remaining span cannot be subdivided, so
            // it fits in one column and finishing it here is exact.
            t_next = t_end;
            x_next = x_end;
        }
        let y_next = if t_next >= t_end { y_end } else { c.y(t_next) };
        deposit(cells, carry, x_lo, x_hi, x_prev, x_next, y_prev, y_next);
        if t_next >= t_end {
            break;
        }
        t_prev = t_next;
        x_prev = x_next;
        y_prev = y_next;
    }
}

/// Nonzero-winding coverage of one path over one pixel row of one tile.
///
/// `cells` is scratch of length `x_hi - x_lo + 1`; `out` is `x_hi - x_lo` wide
/// and receives coverage in `[0, 1]`. Both are caller-owned so the frame
/// allocates once (§17.2's zero-steady-state-allocation habit).
///
/// The absolute value is the nonzero rule: a region wound twice accumulates to
/// `2` and clamps to full coverage, and a region wound `+1` then `-1` cancels to
/// nothing. No orientation bookkeeping, exactly as §10.2 promises.
pub fn fill_row<T: Real>(
    pieces: &[MonoPiece],
    row_y: u32,
    x_lo: u32,
    x_hi: u32,
    cells: &mut [T],
    out: &mut [T],
) {
    let w = (x_hi - x_lo) as usize;
    debug_assert_eq!(out.len(), w);
    debug_assert_eq!(cells.len(), w + 1);
    for c in cells.iter_mut() {
        *c = T::ZERO;
    }
    let mut carry = T::ZERO;
    for piece in pieces {
        accumulate_piece_row(piece, row_y, x_lo, x_hi, cells, &mut carry);
    }
    let mut running = carry;
    for i in 0..w {
        running = running + cells[i];
        let a = running.abs();
        out[i] = if a > T::ONE { T::ONE } else { a };
    }
}

/// Nonzero-winding coverage of one path at one pixel, computed without an
/// accumulator or a scan.
///
/// The same terms as [`fill_row`] in a different association: per piece, the
/// winding that passed to this cell's left plus this cell's own trapezoid.
/// That makes the per-pixel dispatch shape available to the annex — the same
/// one-thread-per-pixel shape the stroke kernel already uses — at the cost of
/// re-deriving each piece's scanline band once per pixel instead of once per
/// row.
pub fn coverage_at_cell<T: Real>(pieces: &[MonoPiece], row_y: u32, cell: u32) -> T {
    let mut acc = T::ZERO;
    let mut window = [T::ZERO; 2];
    for piece in pieces {
        window[0] = T::ZERO;
        window[1] = T::ZERO;
        let mut carry = T::ZERO;
        accumulate_piece_row(piece, row_y, cell, cell + 1, &mut window, &mut carry);
        acc = acc + carry + window[0];
    }
    let a = acc.abs();
    if a > T::ONE { T::ONE } else { a }
}

/// [`fill_row`] at the precision the caller asked for, writing `f64` out.
///
/// The `f32` instantiation is the annex's arithmetic without the annex's
/// hardware — the same controlled experiment G0-8 ran for strokes, so "how much
/// of the Metal engine's divergence is just `f32`?" stays answerable on a
/// machine with no GPU.
pub fn fill_row_at(
    pieces: &[MonoPiece],
    row_y: u32,
    x_lo: u32,
    x_hi: u32,
    precision: Precision,
    scratch: &mut RowScratch,
    out: &mut [f64],
) {
    let w = (x_hi - x_lo) as usize;
    match precision {
        Precision::Reference => {
            scratch.cells64.resize(w + 1, 0.0);
            scratch.out64.resize(w, 0.0);
            fill_row(
                pieces,
                row_y,
                x_lo,
                x_hi,
                &mut scratch.cells64[..w + 1],
                &mut scratch.out64[..w],
            );
            out[..w].copy_from_slice(&scratch.out64[..w]);
        }
        Precision::AnnexF32 => {
            scratch.cells32.resize(w + 1, 0.0);
            scratch.out32.resize(w, 0.0);
            fill_row(
                pieces,
                row_y,
                x_lo,
                x_hi,
                &mut scratch.cells32[..w + 1],
                &mut scratch.out32[..w],
            );
            for (o, v) in out[..w].iter_mut().zip(&scratch.out32[..w]) {
                *o = *v as f64;
            }
        }
    }
}

/// Per-frame scratch for [`fill_row_at`], allocated once and reused.
#[derive(Debug, Clone, Default)]
pub struct RowScratch {
    cells64: Vec<f64>,
    out64: Vec<f64>,
    cells32: Vec<f32>,
    out32: Vec<f32>,
}

impl RowScratch {
    /// Scratch sized for a tile `tile` pixels wide.
    pub fn for_tile(tile: u32) -> RowScratch {
        let w = tile as usize;
        RowScratch {
            cells64: vec![0.0; w + 1],
            out64: vec![0.0; w],
            cells32: vec![0.0; w + 1],
            out32: vec![0.0; w],
        }
    }
}

// ------------------------------------------------------------ the colour field

/// The fill's colour at a screen point.
///
/// Delegates to [`crate::fill::gradient_at`] rather than restating it: the
/// interior field is a *defined stand-in* in both modules and §10.2's real one
/// (arc-length boundary interpolation with mean-value coordinates) is fm-5oi's
/// to design. Duplicating a placeholder would have made it look like a decision.
pub fn gradient_at(style: &Style, p: [f64; 2]) -> [f64; 4] {
    crate::fill::gradient_at(style, p)
}

// ------------------------------------------------- tile classification (§10.4)

/// The command touches the tile's edge: evaluate coverage.
pub const CLASS_PARTIAL: u32 = 0;
/// The tile lies wholly inside the path: coverage is `1` everywhere in it.
pub const CLASS_INTERIOR: u32 = 1;

/// §10.4's per-tile classification, one word per *command* rather than per tile.
///
/// §10.4 classifies tiles as "empty, fully covered, simple-edge, or
/// complex-edge" so that "interiors fill as vectorized spans" and coverage is
/// evaluated "only near edges". Empty is already handled by binning (an unbinned
/// path emits no command), so what is left for a fill is the interior/edge
/// distinction — and it is a property of a **(path, tile) pair**, not of a tile,
/// which is why the flags are parallel to [`crate::ir::CommandLists::draws`]
/// rather than to the tiles.
///
/// **The verdict for fm-gw7:** this wants to be a flag word *inside* the
/// per-tile command list — `(path_index, flags)` rather than `path_index` — at a
/// cost of four bytes per command. Keeping it in a side table here is a spike
/// convenience that avoids reshaping buffers G0-8 already published
/// measurements against.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TileClasses {
    /// One entry per entry of `ir.tiles.draws`.
    pub flags: Vec<u32>,
}

impl TileClasses {
    /// How many commands were classified interior.
    pub fn interior_count(&self) -> usize {
        self.flags.iter().filter(|f| **f == CLASS_INTERIOR).count()
    }
}

/// Classify every command in every tile.
///
/// A command is [`CLASS_INTERIOR`] when the tile lies wholly inside a closed,
/// convex fill. Two consequences, and the second is the one that matters:
///
/// 1. The whole per-pixel evaluation disappears for that command — the dominant
///    cost of any large filled shape, since most of a big shape is interior.
/// 2. Coverage there becomes **exactly `1`** instead of an accumulation that
///    lands within an ulp or two of it. That is not a rounding nicety: it is the
///    precondition that makes occlusion pruning bit-exact, and discovering it is
///    what tied these two optimizations together (see [`prune_occluded`]).
///
/// Writing `1` is also the *more* accurate answer. Analytically the coverage of
/// an interior cell is exactly one; the `1 - ε` an accumulation produces is the
/// numerical error, not the truth.
pub fn classify(ir: &RenderIr, mono: &MonoTable) -> TileClasses {
    let cols = ir.grid.cols();
    let tile = ir.grid.tile;
    let mut flags = vec![CLASS_PARTIAL; ir.tiles.draws.len()];
    for t in 0..ir.grid.count() {
        let lo = ir.tiles.offsets[t] as usize;
        let hi = ir.tiles.offsets[t + 1] as usize;
        if lo == hi {
            continue;
        }
        let rect = tile_rect(ir, t as u32, cols, tile);
        for (flag, &draw) in flags[lo..hi].iter_mut().zip(&ir.tiles.draws[lo..hi]) {
            if tile_is_interior(ir, mono, draw as usize, rect) {
                *flag = CLASS_INTERIOR;
            }
        }
    }
    TileClasses { flags }
}

/// The pixel rectangle a tile covers, clipped to the surface.
fn tile_rect(ir: &RenderIr, t: u32, cols: u32, tile: u32) -> [f64; 4] {
    let tx = (t % cols) * tile;
    let ty = (t / cols) * tile;
    [
        tx as f64,
        ty as f64,
        (tx + tile).min(ir.grid.width) as f64,
        (ty + tile).min(ir.grid.height) as f64,
    ]
}

/// How far outside the tile the interiority test has to hold.
///
/// Geometrically zero would do — tiles are pixel-aligned, so a tile inside a
/// convex region has every one of its cells inside it. The margin is against the
/// *test's* own fragility rather than the geometry's: a corner sitting exactly on
/// an edge is the one place [`winding_at`]'s half-open band could answer either
/// way, and growing the rectangle moves every probe off the boundary. It can only
/// cost pruning opportunities, never soundness.
const INTERIOR_MARGIN: f64 = 1.0;

/// Is this tile wholly inside this path's interior?
fn tile_is_interior(ir: &RenderIr, mono: &MonoTable, index: usize, rect: [f64; 4]) -> bool {
    let path: &PathHeader = &ir.paths[index];
    if path.kind != DrawKind::Fill || !is_convex_closed(ir, path) {
        return false;
    }
    let g = INTERIOR_MARGIN;
    let pieces = mono.pieces_of(index);
    for &(x, y) in &[
        (rect[0] - g, rect[1] - g),
        (rect[2] + g, rect[1] - g),
        (rect[0] - g, rect[3] + g),
        (rect[2] + g, rect[3] + g),
    ] {
        if winding_at(pieces, x, y) == 0 {
            return false;
        }
    }
    true
}

// --------------------------------------------------------- occlusion pruning

/// What a pruning pass removed, for the report and for the tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PruneReport {
    /// Draw commands in the tile lists before pruning.
    pub before: usize,
    /// Draw commands after pruning.
    pub after: usize,
    /// Tiles in which at least one command was dropped.
    pub tiles_touched: usize,
}

impl PruneReport {
    /// The fraction of commands the pass removed.
    pub fn removed_fraction(&self) -> f64 {
        if self.before == 0 {
            return 0.0;
        }
        (self.before - self.after) as f64 / self.before as f64
    }
}

/// Drop commands that are provably invisible, keeping painter order intact.
///
/// §10.8 asks for "a conservative back-to-front opaque-coverage mask per tile"
/// that skips hidden commands "when the result is provably unchanged". This is
/// that mask at its simplest sound form: walk each tile's run from the front,
/// find the **last** command that provably paints the whole tile opaque, and
/// drop everything behind it. It consumes [`classify`]'s flags rather than
/// recomputing interiority, and rewrites them alongside the command list.
///
/// The soundness argument, whose third clause is the one this spike learned:
///
/// 1. The command is [`CLASS_INTERIOR`]: the tile lies wholly inside a closed,
///    convex fill, so every cell of it is interior. (Convexity: a quadratic
///    Bézier lies in its control polygon's hull and bulges toward its handle, so
///    a convex, consistently-turning control polygon bounds a convex region;
///    four interior corners then put the whole rectangle inside.)
/// 2. Both gradient endpoints are opaque, so the composite is `src` everywhere
///    in the tile regardless of what was under it.
/// 3. **Coverage there is exactly `1`, not `1 - ε`** — which is true only
///    because [`classify`] short-circuits the accumulation for interior
///    commands. Without that, the covering fill lets the layer beneath show
///    through by an ulp, the pruned and unpruned frames differ in the last bit
///    of a channel, and an "optimization" has silently changed the picture. The
///    first draft of this pass had exactly that bug, and it is why the two
///    mechanisms ship together.
///
/// Anything that fails a clause is simply not pruned. The frames before and
/// after are asserted bit-identical in the tests — an occlusion optimization
/// that changes a pixel is a bug, not a trade-off.
pub fn prune_occluded(ir: &mut RenderIr, classes: &mut TileClasses) -> PruneReport {
    let ntiles = ir.grid.count();
    let mut report = PruneReport {
        before: ir.tiles.draws.len(),
        after: 0,
        tiles_touched: 0,
    };

    let mut offsets = Vec::with_capacity(ntiles + 1);
    let mut draws = Vec::with_capacity(ir.tiles.draws.len());
    let mut flags = Vec::with_capacity(ir.tiles.draws.len());
    offsets.push(0u32);

    for t in 0..ntiles {
        let lo = ir.tiles.offsets[t] as usize;
        let hi = ir.tiles.offsets[t + 1] as usize;

        let mut start = lo;
        for k in lo..hi {
            if classes.flags[k] == CLASS_INTERIOR && is_opaque(ir, ir.tiles.draws[k] as usize) {
                start = k;
            }
        }
        if start > lo {
            report.tiles_touched += 1;
        }
        draws.extend_from_slice(&ir.tiles.draws[start..hi]);
        flags.extend_from_slice(&classes.flags[start..hi]);
        offsets.push(draws.len() as u32);
    }

    report.after = draws.len();
    ir.tiles.offsets = offsets;
    ir.tiles.draws = draws;
    classes.flags = flags;
    report
}

/// Does this path composite as pure source wherever it has full coverage?
fn is_opaque(ir: &RenderIr, index: usize) -> bool {
    let style = &ir.styles[ir.paths[index].style as usize];
    style.rgba[3] == 1.0 && style.rgba_end[3] == 1.0
}

/// Is the path closed, with a convex control polygon and consistent turning?
///
/// Conservative in the right direction: a false answer costs a pruning
/// opportunity, and there is no input for which a true answer admits a
/// non-convex region.
fn is_convex_closed(ir: &RenderIr, path: &PathHeader) -> bool {
    let first = path.first_segment as usize;
    let count = path.segment_count as usize;
    if count < 2 {
        return false;
    }
    let segs = &ir.segments[first..first + count];
    let close = |a: [f32; 2], b: [f32; 2]| {
        (a[0] - b[0]).abs() as f64 <= 1e-4 && (a[1] - b[1]).abs() as f64 <= 1e-4
    };
    if !close(segs[count - 1].p2, segs[0].p0) {
        return false;
    }

    // The control polygon of the whole closed path: anchor, handle, anchor, …
    let mut poly: Vec<[f64; 2]> = Vec::with_capacity(count * 2);
    for s in segs {
        poly.push([s.p0[0] as f64, s.p0[1] as f64]);
        poly.push([s.p1[0] as f64, s.p1[1] as f64]);
    }

    let n = poly.len();
    let mut sign = 0i32;
    for i in 0..n {
        let a = poly[i];
        let b = poly[(i + 1) % n];
        let c = poly[(i + 2) % n];
        let cross = (b[0] - a[0]) * (c[1] - b[1]) - (b[1] - a[1]) * (c[0] - b[0]);
        // Collinear triples are fine — a polyline edge split in two is still
        // convex — so only a genuine sign flip disqualifies.
        let s = if cross > 1e-9 {
            1
        } else if cross < -1e-9 {
            -1
        } else {
            0
        };
        if s != 0 {
            if sign == 0 {
                sign = s;
            } else if sign != s {
                return false;
            }
        }
    }
    sign != 0
}

/// The winding number of `pieces` around the point `(x, y)`.
///
/// Crossings of the horizontal line through `y`, counted to the left of `x`,
/// with the band half-open in y so a crossing at a shared anchor is counted once
/// — the same convention [`crate::fill`] uses, and the one that stops a closed
/// path leaking winding at every joint.
pub fn winding_at(pieces: &[MonoPiece], x: f64, y: f64) -> i32 {
    let mut w = 0i32;
    for piece in pieces {
        let c = Coeffs::<f64>::of(piece);
        let y0 = c.y(0.0);
        let y1 = c.y(1.0);
        let (lo, hi) = (fmin(y0, y1), fmax(y0, y1));
        if y < lo || y >= hi {
            continue;
        }
        let t = c.t_at_y(y, 0.0, 1.0);
        if c.x(t) < x {
            w += if y1 > y0 { 1 } else { -1 };
        }
    }
    w
}

// ------------------------------------------------- the Metal-shaped derivation

/// Scalars per [`MonoPiece`] in [`FlatFill::pieces`].
pub const PIECE_STRIDE: usize = 6;
/// Scalars per path in [`FlatFill::path_u32`].
pub const FILL_PATH_U32_STRIDE: usize = 4;
/// Scalars per path in [`FlatFill::path_f32`].
pub const FILL_PATH_F32_STRIDE: usize = 4;
/// Scalars per style in [`FlatFill::styles`].
pub const FILL_STYLE_STRIDE: usize = 12;

/// The fill stage's device layout: flat, single-typed arrays, one per table.
///
/// A separate derivation from [`crate::ir::FlatIr`] rather than an extension of
/// it, for two reasons the report states as verdicts:
///
/// - the geometry table is different (monotone pieces, no arc-length span —
///   §10.2 never asks where along the path a pixel is), and
/// - the style table is different (fills read `rgba_end` and `gradient_axis`,
///   which strokes do not, and strokes read the width taper, which fills do
///   not).
///
/// One union-shaped `StyleTable` serving both stages would waste a third of
/// every row or need a bitcast in the inner loop, which is finding F1 again.
#[derive(Debug, Clone, PartialEq)]
pub struct FlatFill {
    /// `[width, height, tile, cols]`.
    pub params_u32: Vec<u32>,
    /// Background RGBA, linear.
    pub params_f32: Vec<f32>,
    /// [`PIECE_STRIDE`] scalars per monotone piece.
    pub pieces: Vec<f32>,
    /// `[first_piece, piece_count, style, 0]` per path.
    pub path_u32: Vec<u32>,
    /// The conservative slab per path.
    pub path_f32: Vec<f32>,
    /// `[rgba, rgba_end, gradient_axis]` per style.
    pub styles: Vec<f32>,
    /// CSR offsets, `tiles + 1` entries.
    pub tile_offsets: Vec<u32>,
    /// CSR path indices.
    pub tile_draws: Vec<u32>,
}

impl FlatFill {
    /// Total bytes crossing the boundary for one frame (§17.2's PG-A counter).
    pub fn upload_bytes(&self) -> usize {
        4 * (self.params_u32.len()
            + self.params_f32.len()
            + self.pieces.len()
            + self.path_u32.len()
            + self.path_f32.len()
            + self.styles.len()
            + self.tile_offsets.len()
            + self.tile_draws.len())
    }
}

/// Derive the fill stage's device layout from the IR and its monotone table.
///
/// **No padding for empty tables happens here, deliberately.** Metal rejects a
/// zero-length buffer binding, so an empty scene needs one dummy element — but
/// putting that in the derivation makes `FlatFill` disagree with its own
/// declared strides, which is exactly how the empty-scene dispatch-floor
/// measurement blew up an assertion the first time it ran. The derivation stays
/// faithful and [`upload_bytes`] stays an honest count; `annex::render_fill`
/// pads at the binding, where the constraint actually lives.
///
/// [`upload_bytes`]: FlatFill::upload_bytes
pub fn flatten_fill(ir: &RenderIr, mono: &MonoTable) -> FlatFill {
    let mut pieces = Vec::with_capacity(mono.pieces.len() * PIECE_STRIDE);
    for p in &mono.pieces {
        pieces.extend_from_slice(&[p.p0[0], p.p0[1], p.p1[0], p.p1[1], p.p2[0], p.p2[1]]);
    }
    let mut path_u32 = Vec::with_capacity(ir.paths.len() * FILL_PATH_U32_STRIDE);
    let mut path_f32 = Vec::with_capacity(ir.paths.len() * FILL_PATH_F32_STRIDE);
    for (i, p) in ir.paths.iter().enumerate() {
        let (first, count) = mono.ranges[i];
        path_u32.extend_from_slice(&[first, count, p.style, 0]);
        path_f32.extend_from_slice(&p.slab);
    }
    let mut styles = Vec::with_capacity(ir.styles.len() * FILL_STYLE_STRIDE);
    for s in &ir.styles {
        styles.extend_from_slice(&s.rgba);
        styles.extend_from_slice(&s.rgba_end);
        styles.extend_from_slice(&s.gradient_axis);
    }
    FlatFill {
        params_u32: vec![ir.grid.width, ir.grid.height, ir.grid.tile, ir.grid.cols()],
        params_f32: ir.background.to_vec(),
        pieces,
        path_u32,
        path_f32,
        styles,
        tile_offsets: ir.tiles.offsets.clone(),
        tile_draws: ir.tiles.draws.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{Style, TileGrid};
    use fmn_geom::quadpath::QuadPath;

    fn grid(n: u32) -> TileGrid {
        TileGrid {
            width: n,
            height: n,
            tile: 16,
        }
    }

    fn rect_path(x0: f64, y0: f64, x1: f64, y1: f64, reversed: bool) -> Vec<[f64; 3]> {
        let pts = if reversed {
            [[x0, y0], [x0, y1], [x1, y1], [x1, y0], [x0, y0]]
        } else {
            [[x0, y0], [x1, y0], [x1, y1], [x0, y1], [x0, y0]]
        };
        pts.iter().map(|p| [p[0], p[1], 0.0]).collect()
    }

    fn fill_ir(paths: &[(Vec<[f64; 3]>, [f32; 4])], n: u32) -> RenderIr {
        let mut ir = RenderIr::new(grid(n), [0.0, 0.0, 0.0, 1.0]);
        for (pts, rgba) in paths {
            let mut p = QuadPath::default();
            p.start_new_path(pts[0]);
            for q in &pts[1..] {
                p.add_line_to(*q, false).unwrap();
            }
            ir.compile_path(&p, Style::flat(*rgba, 0.0, 1.5), DrawKind::Fill)
                .unwrap();
        }
        ir.bin();
        ir
    }

    fn row_coverage(mono: &MonoTable, path: usize, py: u32, w: u32) -> Vec<f64> {
        let mut cells = vec![0.0f64; w as usize + 1];
        let mut out = vec![0.0f64; w as usize];
        fill_row(mono.pieces_of(path), py, 0, w, &mut cells, &mut out);
        out
    }

    #[test]
    fn the_generic_solver_matches_the_stroke_stages_solver_exactly() {
        // One expression tree, two widths, is only an advantage if the f64
        // instantiation is still the same solver the rest of the spike uses.
        for (a2, a1, a0) in [
            (1.0, -3.0, 2.0),
            (0.0, 4.0, -8.0),
            (1e-20, 1.0, 1.0),
            (2.0, 0.0, -8.0),
            (1.0, 2.0, 5.0),
            (0.0, 0.0, 0.0),
        ] {
            let mut mine = [0.0f64; 2];
            let mut theirs = [0.0f64; 3];
            let n = solve_quadratic(a2, a1, a0, &mut mine);
            let m = crate::sdf::solve_quadratic(a2, a1, a0, &mut theirs);
            assert_eq!(n, m, "root count differs for {a2} {a1} {a0}");
            for i in 0..n {
                assert_eq!(mine[i], theirs[i], "root {i} differs for {a2} {a1} {a0}");
            }
        }
    }

    #[test]
    fn splitting_makes_every_piece_monotone_in_both_axes() {
        // A curve chosen to have both extrema strictly inside: it must become
        // three pieces, each monotone in x and in y.
        let mut p = QuadPath::default();
        p.start_new_path([0.0, 0.0, 0.0]);
        p.add_quadratic_bezier_curve_to([40.0, 30.0, 0.0], [10.0, 5.0, 0.0], false)
            .unwrap();
        let mut ir = RenderIr::new(grid(64), [0.0; 4]);
        ir.compile_path(&p, Style::flat([1.0; 4], 0.0, 1.5), DrawKind::Fill)
            .unwrap();
        let mono = MonoTable::build(&ir);
        assert_eq!(mono.pieces.len(), 3, "both extrema should split");

        for piece in &mono.pieces {
            let c = Coeffs::<f64>::of(piece);
            let mut prev = (c.x(0.0), c.y(0.0));
            let end = (c.x(1.0), c.y(1.0));
            let (dx, dy) = (end.0 - prev.0, end.1 - prev.1);
            for i in 1..=64 {
                let t = i as f64 / 64.0;
                let cur = (c.x(t), c.y(t));
                assert!(
                    (cur.0 - prev.0) * dx >= -1e-9,
                    "x is not monotone in {piece:?}"
                );
                assert!(
                    (cur.1 - prev.1) * dy >= -1e-9,
                    "y is not monotone in {piece:?}"
                );
                prev = cur;
            }
        }
    }

    #[test]
    fn a_straight_segment_is_never_split() {
        // The common case — every line, every glyph stem — must not pay for a
        // split it does not need, and must not divide by a vanishing extremum.
        let ir = fill_ir(&[(rect_path(4.0, 4.0, 28.0, 28.0, false), [1.0; 4])], 32);
        let mono = MonoTable::build(&ir);
        assert_eq!(mono.pieces.len(), ir.segments.len());
    }

    #[test]
    fn a_pixel_aligned_rectangle_is_exact_in_both_axes() {
        // The property the supersampled stand-in cannot have: the horizontal
        // edges are exact too, so a half-pixel row is exactly half covered.
        let ir = fill_ir(&[(rect_path(8.0, 8.5, 24.0, 24.0, false), [1.0; 4])], 32);
        let mono = MonoTable::build(&ir);

        let inside = row_coverage(&mono, 0, 16, 32);
        for (px, c) in inside.iter().enumerate().take(24).skip(8) {
            assert!((c - 1.0).abs() < 1e-12, "interior pixel {px} = {c}");
        }
        assert!(inside[7] < 1e-12, "left of the edge: {}", inside[7]);
        assert!(inside[24] < 1e-12, "right of the edge: {}", inside[24]);

        // Row 8 is half covered vertically by the y = 8.5 edge.
        let half = row_coverage(&mono, 0, 8, 32);
        assert!((half[16] - 0.5).abs() < 1e-12, "got {}", half[16]);
        // Row 7 is entirely outside.
        let out = row_coverage(&mono, 0, 7, 32);
        assert!(out.iter().all(|c| *c == 0.0), "{out:?}");
    }

    #[test]
    fn total_coverage_integrates_to_the_analytic_area() {
        // The oracle a fill owes. Fractional in *both* axes this time, which the
        // supersampled stand-in could only approximate in y.
        //
        // The corner values are quarter-integers on purpose: the IR stores
        // segments at `f32`, so a rectangle at y = 12.4 is really a rectangle at
        // 12.399999618530273 and this assertion would be measuring the storage
        // width rather than the fill. (It was, in the first draft, and the
        // 3.8e-5 shortfall it reported is exactly that rounding — worth
        // recording, because the same effect bounds how exact any fill over this
        // IR can ever be.)
        let ir = fill_ir(
            &[(rect_path(8.25, 12.25, 41.75, 44.75, false), [1.0; 4])],
            64,
        );
        let mono = MonoTable::build(&ir);
        let total: f64 = (0..64)
            .map(|py| row_coverage(&mono, 0, py, 64).iter().sum::<f64>())
            .sum();
        let want = (41.75 - 8.25) * (44.75 - 12.25);
        assert!(
            (total - want).abs() < 1e-9,
            "coverage {total} vs analytic area {want}"
        );
    }

    #[test]
    fn a_circle_integrates_to_pi_r_squared() {
        // The curved case, where the monotone splits and the closed-form
        // crossings are doing the work. Quadratics approximate a circle to a
        // small but nonzero error, so the bar is 0.5 % of the area.
        let r = 22.0;
        let p = crate::arc(0.0, std::f64::consts::TAU, r, [32.0, 32.0, 0.0], None);
        let mut ir = RenderIr::new(grid(64), [0.0; 4]);
        ir.compile_path(&p, Style::flat([1.0; 4], 0.0, 1.5), DrawKind::Fill)
            .unwrap();
        let mono = MonoTable::build(&ir);
        let total: f64 = (0..64)
            .map(|py| row_coverage(&mono, 0, py, 64).iter().sum::<f64>())
            .sum();
        let want = std::f64::consts::PI * r * r;
        assert!(
            (total - want).abs() / want < 5e-3,
            "coverage {total} vs pi r^2 {want}"
        );
    }

    #[test]
    fn a_hole_is_a_hole_under_the_nonzero_rule() {
        // Outer square one way, inner square the other: the winding cancels, so
        // the middle must read as empty with no orientation bookkeeping at all.
        let mut p = QuadPath::default();
        for (a, b, rev) in [(8.0, 56.0, false), (24.0, 40.0, true)] {
            let pts = rect_path(a, a, b, b, rev);
            p.start_new_path(pts[0]);
            for q in &pts[1..] {
                p.add_line_to(*q, false).unwrap();
            }
        }
        let mut ir = RenderIr::new(grid(64), [0.0; 4]);
        ir.compile_path(&p, Style::flat([1.0; 4], 0.0, 1.5), DrawKind::Fill)
            .unwrap();
        let mono = MonoTable::build(&ir);
        let row = row_coverage(&mono, 0, 32, 64);
        assert!(
            (row[16] - 1.0).abs() < 1e-9,
            "ring should be solid: {}",
            row[16]
        );
        assert!(row[32] < 1e-9, "hole should be empty: {}", row[32]);
    }

    #[test]
    fn the_tile_carry_makes_tiling_invisible() {
        // The load-bearing property of a tiled fill: a scanline that enters a
        // tile already inside the shape must arrive with its winding. Rendering
        // the same row one tile at a time must equal rendering it whole.
        let ir = fill_ir(&[(rect_path(3.5, 3.5, 60.5, 60.5, false), [1.0; 4])], 64);
        let mono = MonoTable::build(&ir);
        let whole = row_coverage(&mono, 0, 32, 64);

        let mut stitched = Vec::new();
        for t in 0..4u32 {
            let (lo, hi) = (t * 16, t * 16 + 16);
            let mut cells = vec![0.0f64; 17];
            let mut out = vec![0.0f64; 16];
            fill_row(mono.pieces_of(0), 32, lo, hi, &mut cells, &mut out);
            stitched.extend_from_slice(&out);
        }
        for (i, (a, b)) in whole.iter().zip(stitched.iter()).enumerate() {
            assert!((a - b).abs() < 1e-12, "pixel {i}: whole {a} vs tiled {b}");
        }
        // And the middle tiles are solid, which is only true if the carry works.
        assert!((stitched[24] - 1.0).abs() < 1e-12, "{}", stitched[24]);
    }

    #[test]
    fn the_per_pixel_order_agrees_with_the_scanline_order() {
        // The two dispatch shapes are the same sum in a different association.
        // In f64 they must agree to rounding; if they ever do not, one of the
        // two kernels is computing something else.
        let r = 20.0;
        let p = crate::arc(0.0, std::f64::consts::TAU, r, [32.0, 30.0, 0.0], None);
        let mut ir = RenderIr::new(grid(64), [0.0; 4]);
        ir.compile_path(&p, Style::flat([1.0; 4], 0.0, 1.5), DrawKind::Fill)
            .unwrap();
        let mono = MonoTable::build(&ir);
        let pieces = mono.pieces_of(0);

        let mut worst = 0.0f64;
        for py in 0..64u32 {
            let row = row_coverage(&mono, 0, py, 64);
            for (px, want) in row.iter().enumerate() {
                let got: f64 = coverage_at_cell(pieces, py, px as u32);
                worst = worst.max((got - want).abs());
            }
        }
        assert!(worst < 1e-9, "the two orders diverged by {worst}");
    }

    /// A piece that enters its tile at exactly the tile's right edge.
    ///
    /// Lifted verbatim from the fill frame's lobed blob (path 12, piece 8, row
    /// 454, tile `[192, 208)`), where the walk's stepping predicate stalled: the
    /// entry parameter re-evaluated to `208.0009`, `ceil(x) - 1` named the
    /// column the walk was already in, and a three-column span was deposited as
    /// one trapezoid. It was wrong in **f64** and right in `f32` purely because
    /// the two rounded opposite ways, which is exactly the shape of bug a
    /// two-width transcription is supposed to be able to see and this one nearly
    /// hid.
    fn edge_entry_piece() -> MonoPiece {
        MonoPiece {
            p0: [224.83124, 457.48788],
            p1: [192.70052, 457.48788],
            p2: [185.06674, 426.62854],
        }
    }

    #[test]
    fn a_piece_entering_at_the_tile_edge_is_split_across_every_column() {
        let pieces = [edge_entry_piece()];
        for (x_lo, x_hi) in [(192u32, 208u32), (176, 192), (200, 216), (192, 256)] {
            let w = (x_hi - x_lo) as usize;
            for py in 450..458u32 {
                let mut cells = vec![0.0f64; w + 1];
                let mut out = vec![0.0f64; w];
                fill_row(&pieces, py, x_lo, x_hi, &mut cells, &mut out);
                for (i, got) in out.iter().enumerate() {
                    let want: f64 = coverage_at_cell(&pieces, py, x_lo + i as u32);
                    assert!(
                        (got - want).abs() < 1e-9,
                        "tile [{x_lo},{x_hi}) row {py} px {}: scanline {got} vs per-pixel {want}",
                        x_lo + i as u32
                    );
                }
            }
        }
    }

    #[test]
    fn the_two_orders_agree_on_every_tile_alignment() {
        // The generalization of the bug above: the scanline walk and the
        // per-pixel form must agree for *every* window a tiling could impose,
        // not only for a window wide enough that no piece enters at its edge.
        // The original version of this test used one full-width window and saw
        // nothing.
        let p = crate::arc(0.0, std::f64::consts::TAU, 19.5, [31.5, 30.25, 0.0], None);
        let mut ir = RenderIr::new(grid(64), [0.0; 4]);
        ir.compile_path(&p, Style::flat([1.0; 4], 0.0, 1.5), DrawKind::Fill)
            .unwrap();
        let mono = MonoTable::build(&ir);
        let pieces = mono.pieces_of(0);

        let mut worst = 0.0f64;
        for tile in [4u32, 8, 16] {
            for x_lo in (0..64).step_by(tile as usize) {
                let x_hi = x_lo + tile;
                let w = tile as usize;
                for py in 0..64u32 {
                    let mut cells = vec![0.0f64; w + 1];
                    let mut out = vec![0.0f64; w];
                    fill_row(pieces, py, x_lo, x_hi, &mut cells, &mut out);
                    for (i, got) in out.iter().enumerate() {
                        let want: f64 = coverage_at_cell(pieces, py, x_lo + i as u32);
                        worst = worst.max((got - want).abs());
                    }
                }
            }
        }
        assert!(worst < 1e-9, "tiling changed the answer by {worst}");
    }

    #[test]
    fn the_f32_transcription_tracks_the_reference_but_is_not_it() {
        // The arithmetic floor, exactly as the stroke stage measures it: same
        // algorithm, same geometry, only the scalar width changes.
        let r = 21.0;
        let p = crate::arc(0.0, std::f64::consts::TAU, r, [32.0, 32.0, 0.0], None);
        let mut ir = RenderIr::new(grid(64), [0.0; 4]);
        ir.compile_path(&p, Style::flat([1.0; 4], 0.0, 1.5), DrawKind::Fill)
            .unwrap();
        let mono = MonoTable::build(&ir);
        let pieces = mono.pieces_of(0);

        let mut worst = 0.0f64;
        let mut any_difference = false;
        for py in 0..64u32 {
            let mut c64 = vec![0.0f64; 65];
            let mut o64 = vec![0.0f64; 64];
            fill_row(pieces, py, 0, 64, &mut c64, &mut o64);
            let mut c32 = vec![0.0f32; 65];
            let mut o32 = vec![0.0f32; 64];
            fill_row(pieces, py, 0, 64, &mut c32, &mut o32);
            for (a, b) in o64.iter().zip(o32.iter()) {
                let d = (a - *b as f64).abs();
                worst = worst.max(d);
                any_difference |= d > 0.0;
            }
        }
        assert!(
            worst < 1e-3,
            "f32 alone should barely move the fill: {worst}"
        );
        assert!(
            any_difference,
            "f32 and f64 agreeing bit-for-bit means the f32 path is not running"
        );
    }

    #[test]
    fn convexity_is_recognized_and_concavity_is_not() {
        let square = fill_ir(&[(rect_path(4.0, 4.0, 28.0, 28.0, false), [1.0; 4])], 32);
        assert!(is_convex_closed(&square, &square.paths[0]));

        // An L: closed, but with one reflex corner.
        let l: Vec<[f64; 3]> = [
            [4.0, 4.0],
            [28.0, 4.0],
            [28.0, 12.0],
            [12.0, 12.0],
            [12.0, 28.0],
            [4.0, 28.0],
            [4.0, 4.0],
        ]
        .iter()
        .map(|p| [p[0], p[1], 0.0])
        .collect();
        let bent = fill_ir(&[(l, [1.0; 4])], 32);
        assert!(!is_convex_closed(&bent, &bent.paths[0]));

        // An open path is never a cover, however convex it looks.
        let open: Vec<[f64; 3]> = [[4.0, 4.0], [28.0, 4.0], [28.0, 28.0]]
            .iter()
            .map(|p| [p[0], p[1], 0.0])
            .collect();
        let arc = fill_ir(&[(open, [1.0; 4])], 32);
        assert!(!is_convex_closed(&arc, &arc.paths[0]));
    }

    #[test]
    fn classification_finds_the_interior_and_is_not_a_semantics() {
        // A big square: its central tiles are interior, its edge tiles are not,
        // and turning the classification on must not move the picture by more
        // than the accumulation's own error — that is the whole claim.
        let ir = fill_ir(&[(rect_path(4.0, 4.0, 60.0, 60.0, false), [1.0; 4])], 64);
        let mono = MonoTable::build(&ir);
        let classes = classify(&ir, &mono);
        assert!(classes.interior_count() > 0, "nothing classified interior");
        assert!(
            classes.interior_count() < classes.flags.len(),
            "the edge tiles cannot be interior"
        );

        let plain = crate::cpu::render_with(
            &ir,
            Precision::Reference,
            crate::cpu::FillKernel::Analytic {
                mono: &mono,
                classes: None,
            },
        );
        let classified = crate::cpu::render_with(
            &ir,
            Precision::Reference,
            crate::cpu::FillKernel::Analytic {
                mono: &mono,
                classes: Some(&classes),
            },
        );
        let d = crate::compare::diverge(&plain, &classified);
        assert!(
            d.max_abs < 1e-9,
            "classification changed the picture: {}",
            d.summary()
        );
        assert_eq!(d.max_u8, 0, "and it must be invisible: {}", d.summary());
    }

    #[test]
    fn pruning_removes_hidden_draws_and_changes_no_pixel() {
        // A small opaque square under a large opaque one. Every tile the small
        // square touches is fully inside the large square, so every one of its
        // commands is provably invisible.
        let mut ir = fill_ir(
            &[
                (
                    rect_path(20.0, 20.0, 40.0, 40.0, false),
                    [1.0, 0.0, 0.0, 1.0],
                ),
                (rect_path(2.0, 2.0, 62.0, 62.0, false), [0.0, 1.0, 0.0, 1.0]),
            ],
            64,
        );
        let mono = MonoTable::build(&ir);
        let mut classes = classify(&ir, &mono);

        let before = crate::cpu::render_with(
            &ir,
            Precision::Reference,
            crate::cpu::FillKernel::Analytic {
                mono: &mono,
                classes: Some(&classes),
            },
        );
        let report = prune_occluded(&mut ir, &mut classes);
        let after = crate::cpu::render_with(
            &ir,
            Precision::Reference,
            crate::cpu::FillKernel::Analytic {
                mono: &mono,
                classes: Some(&classes),
            },
        );

        assert!(
            report.removed_fraction() > 0.0,
            "nothing was pruned: {report:?}"
        );
        assert_eq!(
            before.pixels, after.pixels,
            "occlusion pruning changed a pixel — that is a bug, not a trade-off"
        );
    }

    #[test]
    fn a_transparent_cover_is_never_pruned_behind() {
        // The same geometry with the top square half transparent: nothing may be
        // dropped, because the result is emphatically not unchanged.
        let mut ir = fill_ir(
            &[
                (
                    rect_path(20.0, 20.0, 40.0, 40.0, false),
                    [1.0, 0.0, 0.0, 1.0],
                ),
                (rect_path(2.0, 2.0, 62.0, 62.0, false), [0.0, 1.0, 0.0, 0.5]),
            ],
            64,
        );
        let mono = MonoTable::build(&ir);
        let mut classes = classify(&ir, &mono);
        let report = prune_occluded(&mut ir, &mut classes);
        assert_eq!(report.before, report.after, "{report:?}");
    }

    #[test]
    fn a_concave_cover_is_never_pruned_behind() {
        // The convexity clause is load-bearing: an L-shaped opaque fill covers
        // some of its bounding tiles and not others, and the corner test alone
        // cannot tell which without convexity.
        let l: Vec<[f64; 3]> = [
            [2.0, 2.0],
            [62.0, 2.0],
            [62.0, 30.0],
            [30.0, 30.0],
            [30.0, 62.0],
            [2.0, 62.0],
            [2.0, 2.0],
        ]
        .iter()
        .map(|p| [p[0], p[1], 0.0])
        .collect();
        let mut ir = fill_ir(
            &[
                (
                    rect_path(35.0, 35.0, 55.0, 55.0, false),
                    [1.0, 0.0, 0.0, 1.0],
                ),
                (l, [0.0, 1.0, 0.0, 1.0]),
            ],
            64,
        );
        let mono = MonoTable::build(&ir);
        let mut classes = classify(&ir, &mono);
        let report = prune_occluded(&mut ir, &mut classes);
        assert_eq!(report.before, report.after, "{report:?}");
    }

    #[test]
    fn the_device_layout_preserves_every_table_at_its_declared_stride() {
        let ir = fill_ir(&[(rect_path(4.0, 4.0, 28.0, 28.0, false), [1.0; 4])], 32);
        let mono = MonoTable::build(&ir);
        let flat = flatten_fill(&ir, &mono);
        assert_eq!(flat.pieces.len(), mono.pieces.len() * PIECE_STRIDE);
        assert_eq!(flat.path_u32.len(), ir.paths.len() * FILL_PATH_U32_STRIDE);
        assert_eq!(flat.path_f32.len(), ir.paths.len() * FILL_PATH_F32_STRIDE);
        assert_eq!(flat.styles.len(), ir.styles.len() * FILL_STYLE_STRIDE);
        assert_eq!(flat.tile_offsets.len(), ir.grid.count() + 1);
        assert!(flat.upload_bytes() > 0);
    }
}
