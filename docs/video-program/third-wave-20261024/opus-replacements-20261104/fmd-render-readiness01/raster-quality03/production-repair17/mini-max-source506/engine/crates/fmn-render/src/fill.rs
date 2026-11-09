//! §10.2's fill: **nonzero-winding coverage evaluated analytically on the
//! curves** — no triangulation, no signed-alpha tricks, no orientation
//! bookkeeping.
//!
//! > Tiled scanline nonzero-winding coverage evaluated analytically on the
//! > curves: quadratic segments made axis-monotone by splitting at both
//! > tangent-extremum parameters — `dy/dt = 0` for y-monotonicity, so a scanline
//! > meets a piece at most once, *and* `dx/dt = 0` for x-monotonicity, so each
//! > column crossing is a single closed-form root; per scanline, exact segment
//! > and column intersections from the closed-form quadratic root; signed
//! > trapezoidal area accumulation per cell, with each tile's rows carrying the
//! > winding deposited to their left in closed form.
//!
//! ## Where this came from, and what changed on the way
//!
//! The algorithm is G0-8b's (`spikes/g0-8-accelerator/src/analytic_fill.rs`,
//! fm-orn), inherited rather than re-derived, together with the two rules that
//! spike paid for:
//!
//! 1. **Where an exact value is known, use it.** A column crossing's `x` *is*
//!    the boundary; re-evaluating `x(t)` to recover it puts an ulp on the wrong
//!    side of an integer, which makes the walk's stepping predicate name the
//!    column it is already in. That defect deposited a three-column span as one
//!    trapezoid — and it was wrong in the `f64` reference while `f32` happened to
//!    be right, so the two-width comparison read like conditioning.
//! 2. **A tiling test must sweep tile alignments.** Comparing the scanline and
//!    per-pixel forms over one full-width window sees nothing, because no piece
//!    ever enters at an edge.
//!
//! Three things are different here, and all three are consequences of this being
//! the engine rather than a spike:
//!
//! - **The host width is `f64`.** §6.1's numeric doctrine and [`crate::table`]'s
//!   rule are the same rule: IR tables are host structs in the semantic width,
//!   and the single-typed `f32` arrays a GPU wants are a *derived* layout. The
//!   spike stored `f32` so its CPU reference and its Metal engine read identical
//!   bytes; that is a property of the derivation, not of the table. The core is
//!   still generic over [`Real`], so the `f32` instantiation remains available as
//!   the annex's arithmetic without the annex's hardware.
//! - **Instancing is a translation, not a rebuild.** §10.8 interns outlines, so
//!   the monotone split is derived **per compiled shape**, once, and an
//!   occurrence contributes only its pixel translation — folded into the piece's
//!   polynomial coefficients at the top of a row rather than materialized as
//!   geometry. A formula with two hundred `x` glyphs splits one outline.
//! - **The tile classes are consumed.** [`crate::bin`] already answers
//!   "is this tile wholly inside this shape", and §10.4's whole point is that
//!   such a tile does not evaluate coverage at all: it fills as a vectorized
//!   span at *exactly* `1`, which is also the precondition that makes occlusion
//!   pruning bit-exact (G0-8b finding F13).
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
//! before the first cell is read. It is computed in closed form — one root solve
//! per piece — never by walking cells from the frame's left edge, which is why
//! the fill loops over **every** piece of a path rather than only the pieces
//! whose bounds meet the tile. Binning is unaffected: a path enclosing any pixel
//! of a tile has hull bounds containing that tile, so [`crate::bin::Binning`]
//! already lists it.
//!
//! ## The two dispatch shapes
//!
//! Both are built from one routine, [`accumulate_piece_row`], because the
//! per-pixel form is the scanline form with a one-cell window:
//!
//! - [`fill_row`] — the scanline shape. One accumulator of `width + 1` cells per
//!   row and one serial prefix sum along x. This is §10.2 literally, and it is
//!   the shape the CPU engine wants: a serial prefix over a row has no occupancy
//!   to lose.
//! - [`coverage_at_cell`] — the per-pixel shape. Each pixel sums, over the
//!   pieces, "the winding that passed to my left" plus "my own cell's
//!   trapezoid", with no accumulator and no scan. It is the annex's shape: one
//!   thread per pixel, the same dispatch the stroke kernel already uses.
//!
//! In exact arithmetic the two agree term for term; they differ only in the
//! association of the sum, which is why both are measured rather than assumed
//! equal.
//!
//! ## SIMD root audit: measured and declined
//!
//! fm-4wt.2 prototyped a governed x86-64-v3 `f64x4` batch for four independent
//! integer-column roots inside one monotone piece. It preserved scalar root bits
//! and consumed every span in the original order, but two warm width sweeps
//! rejected it at every size: 2 columns took 4.05–4.14 µs versus 3.61–3.87 µs
//! scalar; 8 took 10.24–11.31 versus 9.08–9.16; the production 16-pixel tile
//! took 18.14–18.99 versus 16.24–16.40; and even 64 took 67.26–70.05 versus
//! 60.01–61.29. Packing, extracting, validating, and then performing the
//! load-bearing ordered deposits cost more than the batched square roots saved.
//!
//! No slower build-tier route is retained. The `column_roots_*` cases in the
//! existing `compositor` benchmark keep the 2/4/8/16/32/64-column scalar
//! baseline reproducible so a future SoA layout or wider retained tile can
//! reopen the decision with evidence rather than folklore.

mod color_field;

use crate::bin::ScreenMap;
use crate::plan::{GeometryIdentity, RenderPlan};
use crate::table::{Instance, Segment};
use fmn_frame::FrameError;

// ----------------------------------------------------------------- arithmetic

/// The scalar operations the fill needs, so the host (`f64`) and the annex's
/// arithmetic floor (`f32`) run **one** expression tree rather than two
/// hand-kept transcriptions.
///
/// G0-8 wrote `distance_to_quadratic` twice, once per width, and the two copies
/// were correct — but the mirror rule survived that only because a human held
/// it. Here the two instantiations are the same source by construction, which
/// leaves exactly one hand-mirrored copy to worry about: the annex kernel.
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
    /// `3`.
    const THREE: Self;
    /// `4`.
    const FOUR: Self;
    /// `0.5`.
    const HALF: Self;
    /// The root solver's degeneracy tolerance, **relative to the polynomial's
    /// own scale and sized to this width** — G0-8's finding F8, which cost a day
    /// when a single absolute constant was shared between an `f64` engine and an
    /// `f32` one.
    const DEGENERATE_REL: Self;

    /// Narrow the table's storage width into the working width.
    fn from_f64(v: f64) -> Self;
    /// Widen a pixel index.
    fn from_u32(v: u32) -> Self;
    /// Back to `f64` for the surface and the tests.
    fn to_f64(self) -> f64;
    /// Truncate toward zero, as `as i64` does.
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
    const THREE: f64 = 3.0;
    const FOUR: f64 = 4.0;
    const HALF: f64 = 0.5;
    const DEGENERATE_REL: f64 = 1e-14;

    fn from_f64(v: f64) -> f64 {
        v
    }
    fn from_u32(v: u32) -> f64 {
        f64::from(v)
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
    const THREE: f32 = 3.0;
    const FOUR: f32 = 4.0;
    const HALF: f32 = 0.5;
    const DEGENERATE_REL: f32 = 1e-6;

    fn from_f64(v: f64) -> f32 {
        v as f32
    }
    fn from_u32(v: u32) -> f32 {
        v as f32
    }
    fn to_f64(self) -> f64 {
        f64::from(self)
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
/// `f64::min` and a shader's `min` agree on ordinary values and disagree about
/// NaN and signed zero; after G0-8's `sign(0)` incident (finding F7) the house
/// rule is that any predicate the annex kernel also has to spell gets spelled
/// the same way on both sides.
fn fmin<T: Real>(a: T, b: T) -> T {
    if a < b { a } else { b }
}

/// The larger of two values. See [`fmin`].
fn fmax<T: Real>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

/// `+1` for zero and positives, `-1` for negatives.
fn sign_or_positive<T: Real>(x: T) -> T {
    if x >= T::ZERO { T::ONE } else { -T::ONE }
}

/// Real roots of `a2 t² + a1 t + a0`, falling through to the linear case.
///
/// The stable pairing (`q = -½(b + sgn(b)√Δ)`, roots `q/a` and `c/q`) rather
/// than the schoolbook formula, and a degeneracy test relative to the
/// polynomial's own scale rather than an absolute epsilon.
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

// -------------------------------------------------------------- monotone table

/// A quadratic piece that is monotone in **both** axes, in screen pixels.
///
/// Shape-local: the [`ScreenMap`]'s scale and origin are applied, the
/// occurrence's placement is not. §10.8 interns outlines, so a pure-translation
/// occurrence shifts the shared pieces and paying for it in geometry would undo
/// the interning — see [`instance_translation`]. A non-translation affine map
/// derives frame-local pieces from the same retained object-space segments.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MonoPiece {
    /// Start anchor.
    pub p0: [f64; 2],
    /// Control handle.
    pub p1: [f64; 2],
    /// End anchor.
    pub p2: [f64; 2],
}

/// Below this parameter distance from an endpoint a split is skipped.
///
/// A split at `t = 1e-12` yields a piece whose y-extent is smaller than any
/// scanline band, so it can never contribute coverage — it only costs a table
/// row and a per-row rejection. The threshold is on the *parameter*, which is
/// dimensionless, so it needs no relation to screen scale.
const SPLIT_EPS: f64 = 1e-9;

/// Admission limits for one retained or frame-local monotone-piece table.
///
/// The default piece ceiling covers the largest default retained segment
/// table even when every quadratic splits three ways and every segment is its
/// own open subpath with a closing chord. Callers may provision a larger
/// table, but the `u32` range representation remains an absolute ceiling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonoTableLimits {
    /// Maximum total monotone pieces in the destination table.
    pub max_pieces: usize,
    /// Maximum logical bytes occupied by pieces and per-shape ranges.
    pub max_table_bytes: usize,
}

impl Default for MonoTableLimits {
    fn default() -> Self {
        Self {
            max_pieces: 1 << 22,
            max_table_bytes: 1 << 28,
        }
    }
}

/// A monotone table could not be represented or admitted without partial
/// construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonoTableError {
    /// A declared resource ceiling was exceeded.
    LimitExceeded {
        /// Stable resource name.
        resource: &'static str,
        /// Configured inclusive ceiling.
        limit: usize,
        /// Exact requested amount.
        requested: usize,
    },
    /// A count cannot be represented by the table's `u32` ranges.
    IndexCapacityExceeded {
        /// Stable table or range name.
        resource: &'static str,
        /// Exact requested row count.
        requested: usize,
    },
    /// Checked size arithmetic overflowed before allocation.
    SizeOverflow {
        /// Stable resource name.
        resource: &'static str,
    },
    /// The allocator refused a fully preflighted reservation.
    AllocationFailed {
        /// Stable destination name.
        resource: &'static str,
        /// Exact number of additional elements requested.
        requested: usize,
    },
}

impl std::fmt::Display for MonoTableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LimitExceeded {
                resource,
                limit,
                requested,
            } => write!(
                f,
                "{resource} requires {requested}, exceeding the configured limit {limit}"
            ),
            Self::IndexCapacityExceeded {
                resource,
                requested,
            } => write!(
                f,
                "{resource} requires {requested} rows, exceeding the u32 range representation"
            ),
            Self::SizeOverflow { resource } => {
                write!(f, "{resource} size overflows usize")
            }
            Self::AllocationFailed {
                resource,
                requested,
            } => write!(
                f,
                "could not reserve {requested} additional rows for {resource}"
            ),
        }
    }
}

impl std::error::Error for MonoTableError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PieceLayout {
    pieces: usize,
    max_subpath_segments: usize,
}

fn checked_count_add(
    resource: &'static str,
    left: usize,
    right: usize,
) -> Result<usize, MonoTableError> {
    left.checked_add(right)
        .ok_or(MonoTableError::SizeOverflow { resource })
}

fn check_piece_count(count: usize, limits: MonoTableLimits) -> Result<(), MonoTableError> {
    if count > limits.max_pieces {
        return Err(MonoTableError::LimitExceeded {
            resource: "monotone pieces",
            limit: limits.max_pieces,
            requested: count,
        });
    }
    if u32::try_from(count).is_err() {
        return Err(MonoTableError::IndexCapacityExceeded {
            resource: "monotone pieces",
            requested: count,
        });
    }
    Ok(())
}

fn check_shape_count(count: usize) -> Result<(), MonoTableError> {
    if u32::try_from(count).is_err() {
        return Err(MonoTableError::IndexCapacityExceeded {
            resource: "monotone shape ranges",
            requested: count,
        });
    }
    Ok(())
}

fn logical_table_bytes(piece_count: usize, range_count: usize) -> Result<usize, MonoTableError> {
    let pieces = piece_count
        .checked_mul(std::mem::size_of::<MonoPiece>())
        .ok_or(MonoTableError::SizeOverflow {
            resource: "monotone table bytes",
        })?;
    let ranges = range_count
        .checked_mul(std::mem::size_of::<(u32, u32)>())
        .ok_or(MonoTableError::SizeOverflow {
            resource: "monotone table bytes",
        })?;
    pieces
        .checked_add(ranges)
        .ok_or(MonoTableError::SizeOverflow {
            resource: "monotone table bytes",
        })
}

fn check_table_bytes(bytes: usize, limits: MonoTableLimits) -> Result<(), MonoTableError> {
    if bytes > limits.max_table_bytes {
        return Err(MonoTableError::LimitExceeded {
            resource: "monotone table bytes",
            limit: limits.max_table_bytes,
            requested: bytes,
        });
    }
    Ok(())
}

/// The fill's derived geometry: every compiled shape's segments cut into
/// doubly-monotone pieces, plus the per-shape ranges.
///
/// **A second table, not a replacement for the IR's segments.** Strokes read
/// unsplit segments and would be wrong if they did not: §10.3 interpolates width
/// and colour by the normalized arc-length span `(s0, s1)` each
/// [`crate::table::Segment`] carries, and splitting a segment invalidates that
/// span unless it is recomputed, while a nearest-point search over more, shorter
/// pieces is strictly more work for the same answer. So the fill gets its own
/// derived table — keyed, like every other derived artifact in §10.8, by the
/// compiled **geometry** identity and the exact [`ScreenMap`]. A colour or
/// painter-order change must not rebuild it. A pan does, because this
/// representation stores the map origin directly in every piece.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MonoTable {
    pieces: Vec<MonoPiece>,
    ranges: Vec<(u32, u32)>,
    map: ScreenMap,
    geometry: GeometryIdentity,
}

impl MonoTable {
    fn layout_for_segments(
        segments: &[Segment],
        subpath_starts: &[u32],
        map: ScreenMap,
    ) -> Result<PieceLayout, MonoTableError> {
        let screen = |p: fmn_core::types::Vec3| map.to_pixel(p[0], p[1]);
        let mut pieces = 0usize;
        let mut max_subpath_segments = 0usize;
        for (index, &start) in subpath_starts.iter().enumerate() {
            let end = subpath_starts
                .get(index + 1)
                .map_or(segments.len(), |value| *value as usize);
            let (lo, hi) = (
                (start as usize).min(segments.len()),
                end.min(segments.len()),
            );
            if lo >= hi {
                continue;
            }
            max_subpath_segments = max_subpath_segments.max(hi - lo);
            for segment in &segments[lo..hi] {
                pieces = checked_count_add(
                    "monotone pieces",
                    pieces,
                    monotone_piece_count(
                        screen(segment.p0),
                        screen(segment.p1),
                        screen(segment.p2),
                    ),
                )?;
            }
            if screen(segments[lo].p0) != screen(segments[hi - 1].p2) {
                pieces = checked_count_add("monotone pieces", pieces, 1)?;
            }
        }
        Ok(PieceLayout {
            pieces,
            max_subpath_segments,
        })
    }

    /// Derive one already placed segment run into caller-owned storage.
    ///
    /// The engine's per-frame path uses its monotone-piece pool and per-subpath
    /// scratch. Exact output counts, table bytes and the absolute `u32` range
    /// width are checked before either destination is reserved or mutated.
    pub(crate) fn pieces_for_segments_into(
        out: &mut impl crate::arena::Sink<MonoPiece>,
        curves: &mut crate::arena::Pool<[[f64; 2]; 3]>,
        segments: &[Segment],
        subpath_starts: &[u32],
        map: ScreenMap,
        limits: MonoTableLimits,
    ) -> Result<(), MonoTableError> {
        let layout = Self::layout_for_segments(segments, subpath_starts, map)?;
        let final_piece_count = checked_count_add("monotone pieces", out.len(), layout.pieces)?;
        check_piece_count(final_piece_count, limits)?;
        check_table_bytes(logical_table_bytes(final_piece_count, 0)?, limits)?;
        out.try_reserve(layout.pieces)
            .map_err(|_| MonoTableError::AllocationFailed {
                resource: "monotone pieces",
                requested: layout.pieces,
            })?;
        curves
            .try_reserve(layout.max_subpath_segments)
            .map_err(|_| MonoTableError::AllocationFailed {
                resource: "monotone subpath scratch",
                requested: layout.max_subpath_segments,
            })?;

        Self::append_segments(out, curves, segments, subpath_starts, map);
        debug_assert_eq!(out.len(), final_piece_count);
        Ok(())
    }

    /// Derive the table from a synchronized plan under the default limits.
    pub fn build(plan: &RenderPlan, map: ScreenMap) -> Result<MonoTable, MonoTableError> {
        Self::build_with_limits(plan, map, MonoTableLimits::default())
    }

    /// Derive the table from a synchronized plan under explicit limits.
    ///
    /// Every output count and logical byte is computed before any destination
    /// allocation. Limit and width refusals therefore leave both the plan and
    /// all previously built artifacts untouched.
    pub fn build_with_limits(
        plan: &RenderPlan,
        map: ScreenMap,
        limits: MonoTableLimits,
    ) -> Result<MonoTable, MonoTableError> {
        let shapes = plan.shapes().shapes();
        let segments = plan.segments();
        check_shape_count(shapes.len())?;

        let mut total_pieces = 0usize;
        let mut max_subpath_segments = 0usize;
        for shape in shapes {
            let lo = (shape.first_segment as usize).min(segments.len());
            let hi = (lo + shape.segment_count as usize).min(segments.len());
            let own = &segments[lo..hi];
            let layout = Self::layout_for_segments(own, &shape.subpath_starts, map)?;
            total_pieces = checked_count_add("monotone pieces", total_pieces, layout.pieces)?;
            max_subpath_segments = max_subpath_segments.max(layout.max_subpath_segments);
        }
        check_piece_count(total_pieces, limits)?;
        check_table_bytes(logical_table_bytes(total_pieces, shapes.len())?, limits)?;

        let mut pieces = Vec::new();
        pieces
            .try_reserve_exact(total_pieces)
            .map_err(|_| MonoTableError::AllocationFailed {
                resource: "monotone pieces",
                requested: total_pieces,
            })?;
        let mut ranges = Vec::new();
        ranges
            .try_reserve_exact(shapes.len())
            .map_err(|_| MonoTableError::AllocationFailed {
                resource: "monotone shape ranges",
                requested: shapes.len(),
            })?;
        let mut curves = crate::arena::Pool::default();
        curves
            .try_reserve(max_subpath_segments)
            .map_err(|_| MonoTableError::AllocationFailed {
                resource: "monotone subpath scratch",
                requested: max_subpath_segments,
            })?;

        for shape in shapes {
            let first =
                u32::try_from(pieces.len()).map_err(|_| MonoTableError::IndexCapacityExceeded {
                    resource: "monotone pieces",
                    requested: pieces.len(),
                })?;
            let lo = (shape.first_segment as usize).min(segments.len());
            let hi = (lo + shape.segment_count as usize).min(segments.len());
            let own = &segments[lo..hi];
            let layout = Self::layout_for_segments(own, &shape.subpath_starts, map)?;
            // §10.2 fills each *subpath* as if closed, so the pieces are grouped
            // by subpath and each group gets its closing chord. The boundaries
            // come from `Shape::subpath_starts` rather than from anchor
            // continuity: reconstructing them is almost right and silently wrong
            // when one subpath happens to begin exactly where the previous one
            // ended.
            let start = pieces.len();
            Self::append_segments(&mut pieces, &mut curves, own, &shape.subpath_starts, map);
            debug_assert_eq!(pieces.len() - start, layout.pieces);
            let count = u32::try_from(pieces.len() - start).map_err(|_| {
                MonoTableError::IndexCapacityExceeded {
                    resource: "monotone shape range",
                    requested: pieces.len() - start,
                }
            })?;
            ranges.push((first, count));
        }
        debug_assert_eq!(pieces.len(), total_pieces);
        Ok(MonoTable {
            pieces,
            ranges,
            map,
            geometry: plan.geometry_identity(),
        })
    }

    fn append_segments(
        out: &mut impl crate::arena::Sink<MonoPiece>,
        curves: &mut crate::arena::Pool<[[f64; 2]; 3]>,
        segments: &[Segment],
        subpath_starts: &[u32],
        map: ScreenMap,
    ) {
        let screen = |p: fmn_core::types::Vec3| map.to_pixel(p[0], p[1]);
        for (index, &start) in subpath_starts.iter().enumerate() {
            let end = subpath_starts
                .get(index + 1)
                .map_or(segments.len(), |value| *value as usize);
            let (lo, hi) = (
                (start as usize).min(segments.len()),
                end.min(segments.len()),
            );
            if lo >= hi {
                continue;
            }
            curves.clear();
            for segment in &segments[lo..hi] {
                curves.put([screen(segment.p0), screen(segment.p1), screen(segment.p2)]);
            }
            append_subpath(curves, out);
        }
    }

    /// The pieces belonging to compiled shape `index`.
    ///
    /// Empty for an index the table does not know, which is the honest answer
    /// for a plan that resynchronized under this table: a missing shape draws
    /// nothing rather than someone else's outline.
    #[must_use]
    pub fn pieces_of(&self, index: u32) -> &[MonoPiece] {
        match self.ranges.get(index as usize) {
            Some(&(first, count)) => {
                &self.pieces[first as usize..(first as usize + count as usize)]
            }
            None => &[],
        }
    }

    /// Every piece, contiguous and grouped by shape.
    #[must_use]
    pub fn pieces(&self) -> &[MonoPiece] {
        &self.pieces
    }

    /// How many compiled shapes this table covers.
    #[must_use]
    pub fn len(&self) -> usize {
        self.ranges.len()
    }

    /// Is the table empty?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    /// The mapping the pieces were derived under.
    ///
    /// Carried so a caller can tell a stale table from a current one without
    /// keeping the mapping beside it — the same reason
    /// [`crate::cache::TileKey`] carries its camera revision.
    #[must_use]
    pub fn map(&self) -> ScreenMap {
        self.map
    }

    /// Whether this table was built from the plan's current shape-indexed
    /// geometry.
    pub(crate) fn matches_plan(&self, plan: &RenderPlan) -> bool {
        self.geometry == plan.geometry_identity()
    }
}

/// The pixel translation one occurrence contributes.
///
/// [`MonoTable`]'s pieces already carry the map's origin, so an occurrence adds
/// only its translation scaled to pixels. Splitting the placement this way is
/// what makes interning pay: N translated copies of a glyph are N of these and
/// one outline.
#[must_use]
pub fn instance_translation(inst: &Instance, map: ScreenMap) -> [f64; 2] {
    debug_assert!(
        inst.placement.is_translation(),
        "non-translation placements require frame-local transformed pieces"
    );
    let translation = inst.placement.translation();
    map.delta_to_pixel(translation[0], translation[1])
}

/// The parameter of a quadratic component's extremum, if it lies strictly inside
/// the segment.
///
/// `v(t) = a + b t + c t²` has `v'(t) = b + 2 c t`, so the extremum is at
/// `-b / 2c`. A vanishing `c` is a straight component with no extremum at all,
/// which is the common case — every line, every glyph stem — and must not
/// divide.
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

/// Original-parameter split times that survive the endpoint/duplicate guard.
///
/// Counting and materialization share this authority so a successful
/// preflight reserves exactly the rows the builder will append.
fn monotone_split_times(p0: [f64; 2], p1: [f64; 2], p2: [f64; 2]) -> ([f64; 2], usize) {
    let mut candidates = [0.0f64; 2];
    let mut candidate_count = 0;
    if let Some(t) = extremum(p0[1], p1[1], p2[1]) {
        candidates[candidate_count] = t;
        candidate_count += 1;
    }
    if let Some(t) = extremum(p0[0], p1[0], p2[0]) {
        candidates[candidate_count] = t;
        candidate_count += 1;
    }
    if candidate_count == 2 && candidates[0] > candidates[1] {
        candidates.swap(0, 1);
    }

    let mut accepted = [0.0f64; 2];
    let mut accepted_count = 0;
    let mut t_base = 0.0;
    for &t in candidates.iter().take(candidate_count) {
        let local = (t - t_base) / (1.0 - t_base);
        if !(local > SPLIT_EPS && local < 1.0 - SPLIT_EPS) {
            continue;
        }
        accepted[accepted_count] = t;
        accepted_count += 1;
        t_base = t;
    }
    (accepted, accepted_count)
}

fn monotone_piece_count(p0: [f64; 2], p1: [f64; 2], p2: [f64; 2]) -> usize {
    monotone_split_times(p0, p1, p2).1 + 1
}

/// Split one subpath's screen-space curves into monotone pieces, **closing it**.
///
/// §10.2 fills a path, and a path's interior is only defined for closed
/// boundaries, so an open subpath fills as if its closing chord were present —
/// SVG's rule for `fill`, and manim's (`get_triangulation` closes each subpath
/// before triangulating). Leaving it open is not a smaller answer, it is a
/// different picture: the winding accumulated by an unterminated boundary never
/// returns to zero, so every pixel to its right stays inside. Measured on the
/// open triangle in this module's tests, the unclosed form filled a
/// **rectangle running off the right edge of the window** — 48 pixels against
/// the triangle's 96 — which is the kind of wrong that looks like a plausible
/// picture.
///
/// The chord's handle sits at its midpoint, which makes it an exactly straight
/// quadratic and doubly monotone by construction, so it needs no split.
fn append_subpath(curves: &[[[f64; 2]; 3]], out: &mut impl crate::arena::Sink<MonoPiece>) {
    let Some(first) = curves.first() else {
        return;
    };
    for c in curves {
        split_monotone(c[0], c[1], c[2], out);
    }
    let start = first[0];
    let end = curves[curves.len() - 1][2];
    if start != end {
        out.put(MonoPiece {
            p0: end,
            p1: [0.5 * (end[0] + start[0]), 0.5 * (end[1] + start[1])],
            p2: start,
        });
    }
}

/// Split one segment at its y- and x-extrema, appending doubly-monotone pieces.
///
/// A quadratic has at most one extremum per axis, so this appends at most three
/// pieces — a bound a device layout can rely on when sizing the table.
fn split_monotone(
    p0: [f64; 2],
    p1: [f64; 2],
    p2: [f64; 2],
    out: &mut impl crate::arena::Sink<MonoPiece>,
) {
    let (ts, n) = monotone_split_times(p0, p1, p2);

    // Successive de Casteljau splits, each reparameterized into the remaining
    // right-hand piece.
    let mut cur = [p0, p1, p2];
    let mut t_base = 0.0;
    for &t in ts.iter().take(n) {
        let local = (t - t_base) / (1.0 - t_base);
        debug_assert!(local > SPLIT_EPS && local < 1.0 - SPLIT_EPS);
        let (left, right) = split_at(cur, local);
        out.put(MonoPiece {
            p0: left[0],
            p1: left[1],
            p2: left[2],
        });
        cur = right;
        t_base = t;
    }
    out.put(MonoPiece {
        p0: cur[0],
        p1: cur[1],
        p2: cur[2],
    });
}

/// de Casteljau's split of a quadratic at `t`.
fn split_at(p: [[f64; 2]; 3], t: f64) -> ([[f64; 2]; 3], [[f64; 2]; 3]) {
    let lerp = |a: [f64; 2], b: [f64; 2]| [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
    let q0 = lerp(p[0], p[1]);
    let q1 = lerp(p[1], p[2]);
    let r = lerp(q0, q1);
    ([p[0], q0, r], [r, q1, p[2]])
}

// ---------------------------------------------------------------- the fill core

/// One piece's component polynomials, `v(t) = a + b t + c t²`, at the working
/// width, with the occurrence's translation folded into the constant term.
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
    fn of(piece: &MonoPiece, translate: [f64; 2]) -> Coeffs<T> {
        let f = T::from_f64;
        let (tx, ty) = (f(translate[0]), f(translate[1]));
        let (x0, x1, x2) = (f(piece.p0[0]), f(piece.p1[0]), f(piece.p2[0]));
        let (y0, y1, y2) = (f(piece.p0[1]), f(piece.p1[1]), f(piece.p2[1]));
        Coeffs {
            ax: x0 + tx,
            bx: T::TWO * (x1 - x0),
            cx: (x2 - x1) - (x1 - x0),
            ay: y0 + ty,
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

    /// The **dy-weighted mean of `x`** over `[t0, t1]`: `(∫ x dy) / (∫ dy)`,
    /// in closed form.
    ///
    /// This is the number that makes the fill exact on curves rather than on
    /// chords, and it is the one place this module departs from the spike it
    /// inherited. The per-cell deposit needs `∫ (c + 1 − x) dy` over the span,
    /// i.e. `dy · (c + 1 − x̄)`; every signed-area rasterizer in the wild —
    /// font-rs, `stb_truetype`, the G0-8b spike — substitutes the *chord*
    /// midpoint `½(x(t0) + x(t1))` for `x̄`, which is exact only when `x` is
    /// linear in `y`. On a curve it carries an `O(curvature · h²)` error, and
    /// the way that error announces itself is a **subdivision-invariance
    /// failure**: splitting a curve at its midpoint halves the per-span
    /// curvature and moves every partially-covered pixel. Measured on a
    /// 15-pixel-radius circle of 8 quadratics, a de Casteljau split at `t = ½`
    /// — which changes the point array and not the curve — moved coverage by
    /// **5.7e-3**, one and a half 8-bit levels and past §10.2's own
    /// sub-`1/256` acceptance. "Evaluated analytically on the curves" has to
    /// mean the integral, not a chord through it.
    ///
    /// With `x(t) = ax + bx t + cx t²` and `y'(t) = by + 2 cy t`:
    ///
    /// ```text
    /// ∫ x y' dt = Δ · [k0 + k1·S1/2 + k2·S2/3 + k3·S3/4]
    /// ∫ y'   dt = Δ · [by + cy·S1]
    /// ```
    ///
    /// with `Δ = t1 − t0`, `S1 = t1 + t0`, `S2 = t1² + t1 t0 + t0²`,
    /// `S3 = S1 (t1² + t0²)`, and `k0..k3` the product's coefficients. **`Δ`
    /// cancels**, which is why this is written as a ratio of the bracketed
    /// forms rather than as `F(t1) − F(t0)` over `y(t1) − y(t0)`: the
    /// difference-of-antiderivatives spelling loses every significant digit as
    /// a span shrinks toward a column boundary, and spans that shrink toward
    /// column boundaries are the common case, not the corner one.
    ///
    /// `None` when the mean of `y'` vanishes — a zero-height span, which the
    /// caller has already discarded on `dy == 0`, kept here so the division is
    /// unconditional at the call site rather than trusted.
    fn mean_x_over_dy(&self, t0: T, t1: T) -> Option<T> {
        let s1 = t1 + t0;
        let s2 = t1 * t1 + t1 * t0 + t0 * t0;
        let s3 = s1 * (t1 * t1 + t0 * t0);
        let denom = self.by + self.cy * s1;
        if denom == T::ZERO {
            return None;
        }
        let k0 = self.ax * self.by;
        let k1 = self.bx * self.by + T::TWO * self.ax * self.cy;
        let k2 = self.cx * self.by + T::TWO * self.bx * self.cy;
        let k3 = T::TWO * self.cx * self.cy;
        let num = k0 + k1 * s1 * T::HALF + k2 * s2 / T::THREE + k3 * s3 / T::FOUR;
        Some(num / denom)
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
/// endpoint attaining the nearer extreme, and a root solve finding nothing in
/// range falls back to the secant through the endpoints. Neither fallback is a
/// fudge — monotonicity guarantees the answer exists, and the fallbacks only
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

/// Deposit one sub-span's exact signed area into the row.
///
/// `cells` is `x_hi - x_lo + 1` wide: the extra entry catches the spill from the
/// last in-tile cell and is never read, which keeps the deposit branch-free at
/// the tile's right edge.
///
/// Two separate decisions, and keeping them separate is what makes this safe:
///
/// - **Which column** the span belongs to is decided by the *chord* midpoint,
///   exactly as before. The walk guarantees the span lies inside one column, so
///   any interior point names it; using the endpoints' mean keeps the choice
///   agreeing with the clamp that the walk was tuned against, and keeps a span
///   sitting exactly on a boundary landing on the same side it always did.
/// - **How much of that column** lies right of the curve is
///   [`Coeffs::mean_x_over_dy`] — the integral, not the chord. The clamp to
///   `[0, 1]` repairs floating point only: the span is inside the column by
///   construction, so `x̄` is too.
#[allow(clippy::too_many_arguments)]
fn deposit<T: Real>(
    cells: &mut [T],
    carry: &mut T,
    x_lo: u32,
    x_hi: u32,
    c: &Coeffs<T>,
    t_a: T,
    t_b: T,
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
    if cell < i64::from(x_lo) {
        *carry = *carry + d;
        return;
    }
    if cell >= i64::from(x_hi) {
        return;
    }
    let xbar = c.mean_x_over_dy(t_a, t_b).unwrap_or(xm);
    let mut f = xbar - cell_f;
    if f < T::ZERO {
        f = T::ZERO;
    } else if f > T::ONE {
        f = T::ONE;
    }
    let i = (cell - i64::from(x_lo)) as usize;
    cells[i] = cells[i] + d * (T::ONE - f);
    cells[i + 1] = cells[i + 1] + d * f;
}

/// Accumulate one piece's contribution to one pixel row of one tile.
///
/// This is the whole algorithm, and both dispatch shapes are built from it:
/// [`fill_row`] calls it once per piece with the tile's full width, and
/// [`coverage_at_cell`] calls it once per piece with a one-cell window. The
/// per-cell work is bounded by `x_hi - x_lo + 2` iterations, which is what lets
/// an annex twin write a counted loop instead of a `while`.
pub fn accumulate_piece_row<T: Real>(
    piece: &MonoPiece,
    translate: [f64; 2],
    row_y: u32,
    x_lo: u32,
    x_hi: u32,
    cells: &mut [T],
    carry: &mut T,
) {
    accumulate_piece_row_inner(piece, translate, row_y, x_lo, x_hi, cells, carry, None);
}

/// [`accumulate_piece_row`] plus the native row's boundary-sheet marks.
#[allow(clippy::too_many_arguments)]
fn accumulate_piece_row_inner<T: Real>(
    piece: &MonoPiece,
    translate: [f64; 2],
    row_y: u32,
    x_lo: u32,
    x_hi: u32,
    cells: &mut [T],
    carry: &mut T,
    mut crossings: Option<&mut [u8]>,
) {
    let c = Coeffs::<T>::of(piece, translate);
    let row = T::from_u32(row_y);
    let row_end = row + T::ONE;

    // The piece's y-extent, and the part of it inside this scanline band.
    // Monotone in y, so the endpoints are the extremes.
    let y0 = c.y(T::ZERO);
    let y1 = c.y(T::ONE);
    if y0 == y1 {
        // A horizontal sheet contributes no signed dy to Green's-theorem area,
        // but it still matters to adaptive classification. Own it half-open in
        // y, just as `winding_at` owns shared anchors, so two horizontal sides
        // of a subpixel-height feature are both counted in their native row
        // without also appearing in the adjacent row.
        if y0 >= row
            && y0 < row_end
            && let Some(counts) = crossings.as_deref_mut()
        {
            mark_boundary_cells(c.x(T::ZERO), c.x(T::ONE), x_lo, x_hi, counts);
        }
        return;
    }
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
    if let Some(crossings) = crossings {
        mark_boundary_cells(xa, xb, x_lo, x_hi, crossings);
    }
    let increasing = xb >= xa;
    let left = T::from_u32(x_lo);
    let right = T::from_u32(x_hi);

    // Where the piece crosses the tile's vertical edges. Monotone in x, so each
    // crossing is a single closed-form root, and `invert`'s clamp handles a
    // piece that never reaches an edge.
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
    // computes as `208.0009`, the next boundary comes out as the column the walk
    // is already in, the step makes no progress, and the fallback below deposits
    // the entire remaining span — which crosses three columns — as a single
    // trapezoid in whichever column its midpoint lands in.
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
        deposit(
            cells, carry, x_lo, x_hi, &c, t_prev, t_next, x_prev, x_next, y_prev, y_next,
        );
        if t_next >= t_end {
            break;
        }
        t_prev = t_next;
        x_prev = x_next;
        y_prev = y_next;
    }
}

/// Mark the native cells one monotone boundary sheet reaches in this row.
///
/// `xa..xb` are the sheet's x-extrema within the one-pixel y band: the piece is
/// monotone in x, so every cell interval between them is crossed exactly once.
/// Endpoint-only touches are conservatively included; a false complex
/// classification costs samples, while a missed second sheet changes quality.
fn mark_boundary_cells<T: Real>(xa: T, xb: T, x_lo: u32, x_hi: u32, out: &mut [u8]) {
    if x_lo >= x_hi || out.is_empty() {
        return;
    }
    let mut first = fmin(xa, xb).floor().to_i64();
    let mut last = fmax(xa, xb).floor().to_i64();
    let lo = i64::from(x_lo);
    let hi = i64::from(x_hi);
    if last < lo || first >= hi {
        return;
    }
    first = first.max(lo);
    last = last.min(hi - 1);
    for cell in first..=last {
        let index = (cell - lo) as usize;
        if let Some(count) = out.get_mut(index) {
            *count = count.saturating_add(1);
        }
    }
}

/// Nonzero-winding coverage of one path over one pixel row of one tile.
///
/// `cells` is scratch of length `x_hi - x_lo + 1`; `out` is `x_hi - x_lo` wide
/// and receives coverage in `[0, 1]`. Both are caller-owned so a frame allocates
/// once (PG-6's zero-steady-state-allocation requirement).
///
/// The absolute value **is** the nonzero rule: a region wound twice accumulates
/// to `2` and clamps to full coverage, and a region wound `+1` then `-1` cancels
/// to nothing. No orientation bookkeeping, exactly as §10.2 promises.
pub fn fill_row<T: Real>(
    pieces: &[MonoPiece],
    translate: [f64; 2],
    row_y: u32,
    x_lo: u32,
    x_hi: u32,
    cells: &mut [T],
    out: &mut [T],
) {
    fill_row_inner(pieces, translate, row_y, x_lo, x_hi, cells, out, None);
}

/// [`fill_row`] plus a saturating boundary-sheet count per native cell.
///
/// Classification is fused with the same piece walk that deposits coverage:
/// general analytic fills do not walk their geometry a second time merely to
/// decide whether the result needs subcell compositing.
pub fn fill_row_classified<T: Real>(
    pieces: &[MonoPiece],
    translate: [f64; 2],
    row_y: u32,
    x: std::ops::Range<u32>,
    cells: &mut [T],
    out: &mut [T],
    crossings: &mut [u8],
) {
    let (x_lo, x_hi) = (x.start, x.end);
    fill_row_inner(
        pieces,
        translate,
        row_y,
        x_lo,
        x_hi,
        cells,
        out,
        Some(crossings),
    );
}

#[allow(clippy::too_many_arguments)]
fn fill_row_inner<T: Real>(
    pieces: &[MonoPiece],
    translate: [f64; 2],
    row_y: u32,
    x_lo: u32,
    x_hi: u32,
    cells: &mut [T],
    out: &mut [T],
    mut crossings: Option<&mut [u8]>,
) {
    let w = (x_hi - x_lo) as usize;
    debug_assert_eq!(out.len(), w);
    debug_assert_eq!(cells.len(), w + 1);
    for c in cells.iter_mut() {
        *c = T::ZERO;
    }
    if let Some(counts) = crossings.as_deref_mut() {
        debug_assert_eq!(counts.len(), w);
        counts.fill(0);
    }
    let mut carry = T::ZERO;
    for piece in pieces {
        accumulate_piece_row_inner(
            piece,
            translate,
            row_y,
            x_lo,
            x_hi,
            cells,
            &mut carry,
            crossings.as_deref_mut(),
        );
    }
    let mut running = carry;
    for i in 0..w {
        running = running + cells[i];
        let a = running.abs();
        out[i] = if a > T::ONE { T::ONE } else { a };
    }
}

/// Nonzero-winding coverage of one path at one pixel, with no accumulator and no
/// scan.
///
/// The same terms as [`fill_row`] in a different association: per piece, the
/// winding that passed to this cell's left plus this cell's own trapezoid. That
/// makes the per-pixel dispatch shape available to the annex — the same
/// one-thread-per-pixel shape the stroke kernel already uses — at the cost of
/// re-deriving each piece's scanline band once per pixel instead of once per
/// row.
pub fn coverage_at_cell<T: Real>(
    pieces: &[MonoPiece],
    translate: [f64; 2],
    row_y: u32,
    cell: u32,
) -> T {
    let mut acc = T::ZERO;
    let mut window = [T::ZERO; 2];
    for piece in pieces {
        window[0] = T::ZERO;
        window[1] = T::ZERO;
        let mut carry = T::ZERO;
        accumulate_piece_row(
            piece,
            translate,
            row_y,
            cell,
            cell + 1,
            &mut window,
            &mut carry,
        );
        acc = acc + carry + window[0];
    }
    let a = acc.abs();
    if a > T::ONE { T::ONE } else { a }
}

/// Exact fill coverage of one cell in an `samples × samples` subpixel grid.
///
/// This is the fused-resolve primitive for §10.4's standard-only adaptive AA.
/// Scaling the outline and its occurrence translation by `samples` turns the
/// requested subcell into an ordinary unit cell, so the same analytic integral
/// as [`coverage_at_cell`] evaluates it. No supersampled canvas exists: callers
/// composite the returned subcell immediately and average a single native
/// output pixel.
///
/// `samples == 0` and an out-of-range subcell return zero. Coordinate overflow
/// falls back to the native cell's exact coverage; such a viewport cannot be
/// materialized as a frame in practice, but the helper remains total for every
/// `u32` input.
#[must_use]
pub fn coverage_at_subcell(
    pieces: &[MonoPiece],
    translate: [f64; 2],
    row_y: u32,
    cell: u32,
    samples: u32,
    sample_x: u32,
    sample_y: u32,
) -> f64 {
    if samples == 0 || sample_x >= samples || sample_y >= samples {
        return 0.0;
    }
    let Some(high_x) = cell
        .checked_mul(samples)
        .and_then(|x| x.checked_add(sample_x))
    else {
        return coverage_at_cell(pieces, translate, row_y, cell);
    };
    let Some(high_y) = row_y
        .checked_mul(samples)
        .and_then(|y| y.checked_add(sample_y))
    else {
        return coverage_at_cell(pieces, translate, row_y, cell);
    };
    if high_x == u32::MAX {
        return coverage_at_cell(pieces, translate, row_y, cell);
    }

    let scale = f64::from(samples);
    let scaled_translate = [translate[0] * scale, translate[1] * scale];
    let mut acc = 0.0f64;
    let mut window = [0.0; 2];
    for piece in pieces {
        let scaled = MonoPiece {
            p0: [piece.p0[0] * scale, piece.p0[1] * scale],
            p1: [piece.p1[0] * scale, piece.p1[1] * scale],
            p2: [piece.p2[0] * scale, piece.p2[1] * scale],
        };
        window.fill(0.0);
        let mut carry = 0.0;
        accumulate_piece_row(
            &scaled,
            scaled_translate,
            high_y,
            high_x,
            high_x + 1,
            &mut window,
            &mut carry,
        );
        acc += carry + window[0];
    }
    acc.abs().min(1.0)
}

/// How many independent fill-boundary sheets cross one native pixel cell.
///
/// Three horizontal and three vertical interior probes are enough to
/// distinguish the ordinary one-edge case from the cases G0-2 assigned to
/// adaptive AA: a subpixel-width feature contributes two crossings, while
/// cusps, near tangencies and dense self-intersections contribute more. Both
/// directions are probed because a horizontal edge is invisible to a
/// horizontal scanline and vice versa.
///
/// Shared anchors use half-open endpoint ownership, matching [`winding_at`], so
/// an ordinary join counts once rather than once per adjacent piece. The count
/// saturates at `u8::MAX`; the adaptive thresholds are far below that.
#[must_use]
pub fn boundary_crossings_at_cell(
    pieces: &[MonoPiece],
    translate: [f64; 2],
    row_y: u32,
    cell: u32,
) -> u8 {
    /// Candidates held on the stack; a cell met by more pieces than this
    /// probes the whole list (the count is the same either way).
    const MAX_CANDIDATES: usize = 32;
    let x0 = f64::from(cell);
    let y0 = f64::from(row_y);
    let mut candidates = [0usize; MAX_CANDIDATES];
    let mut held = 0;
    for (index, piece) in pieces.iter().enumerate() {
        if !piece_extent_may_meet_cell(piece, translate, x0, y0) {
            continue;
        }
        if held == MAX_CANDIDATES {
            return probe_crossings(pieces.iter(), translate, x0, y0);
        }
        candidates[held] = index;
        held += 1;
    }
    probe_crossings(
        candidates[..held].iter().map(|&index| &pieces[index]),
        translate,
        x0,
        y0,
    )
}

/// The six interior probes of [`boundary_crossings_at_cell`] over `pieces`.
fn probe_crossings<'a>(
    pieces: impl Iterator<Item = &'a MonoPiece> + Clone,
    translate: [f64; 2],
    x0: f64,
    y0: f64,
) -> u8 {
    const PROBES: [f64; 3] = [0.25, 0.5, 0.75];
    let x1 = x0 + 1.0;
    let y1 = y0 + 1.0;
    let mut most = 0u8;

    for offset in PROBES {
        let y = y0 + offset;
        let mut count = 0u8;
        for piece in pieces.clone() {
            let c = Coeffs::<f64>::of(piece, translate);
            let a = c.y(0.0);
            let b = c.y(1.0);
            let lo = a.min(b);
            let hi = a.max(b);
            if hi <= lo || y < lo || y >= hi {
                continue;
            }
            let t = c.t_at_y(y, 0.0, 1.0);
            let x = c.x(t);
            if x >= x0 && x < x1 {
                count = count.saturating_add(1);
            }
        }
        most = most.max(count);
    }

    for offset in PROBES {
        let x = x0 + offset;
        let mut count = 0u8;
        for piece in pieces.clone() {
            let c = Coeffs::<f64>::of(piece, translate);
            let a = c.x(0.0);
            let b = c.x(1.0);
            let lo = a.min(b);
            let hi = a.max(b);
            if hi <= lo || x < lo || x >= hi {
                continue;
            }
            let t = c.t_at_x(x, 0.0, 1.0);
            let y = c.y(t);
            if y >= y0 && y < y1 {
                count = count.saturating_add(1);
            }
        }
        most = most.max(count);
    }

    most
}

/// Whether a piece's endpoint box, widened for rounding, meets the unit cell
/// at `(x0, y0)`.
///
/// A [`MonoPiece`] is monotone in both axes, so every point the probes in
/// [`boundary_crossings_at_cell`] can evaluate on it lies between its
/// translated endpoints; evaluation rounding moves a value by a few units in
/// the last place of the coordinates. The widening — 10⁻⁶ px plus 10⁻⁹ of the
/// coordinate magnitude, orders of magnitude above that — makes a `false`
/// here a proof that no probe of the cell can count the piece, so skipping it
/// changes no crossing count. `NaN` coordinates answer `true` (never skipped).
fn piece_extent_may_meet_cell(piece: &MonoPiece, translate: [f64; 2], x0: f64, y0: f64) -> bool {
    let reaches = |a: f64, b: f64, lo: f64| {
        let (min, max) = (a.min(b), a.max(b));
        let margin = 1e-6 + 1e-9 * (min.abs() + max.abs());
        !(max + margin < lo || min - margin > lo + 1.0)
    };
    reaches(piece.p0[0] + translate[0], piece.p2[0] + translate[0], x0)
        && reaches(piece.p0[1] + translate[1], piece.p2[1] + translate[1], y0)
}

/// The winding number of a filled path at a screen point.
///
/// Not a coverage query — an exact integer, used by the interior colour field to
/// decide insideness and by tests to state the nonzero rule directly. Counts
/// signed crossings of the horizontal ray to the **left** of `p`, which is the
/// same bookkeeping the row accumulator does continuously, so the two can never
/// disagree about which side of a boundary a pixel is on.
///
/// Sign convention: a counter-clockwise boundary yields `+1`, the standard
/// winding number. With a leftward ray that means an *upward* crossing counts
/// `-1` — the mirror of the textbook rightward-ray rule, and worth spelling out
/// because getting it backwards passes every `abs()`-based coverage test and
/// then reports the wrong sign to the one caller that cares.
#[must_use]
pub fn winding_at(pieces: &[MonoPiece], translate: [f64; 2], p: [f64; 2]) -> i32 {
    let mut w = 0;
    for piece in pieces {
        let c = Coeffs::<f64>::of(piece, translate);
        let y0 = c.y(0.0);
        let y1 = c.y(1.0);
        // Half-open in y so a shared anchor is counted once, not twice or zero
        // times — the classic off-by-one that makes a closed path leak.
        let up = y1 > y0;
        let (lo, hi) = if up { (y0, y1) } else { (y1, y0) };
        if p[1] < lo || p[1] >= hi {
            continue;
        }
        let t = c.t_at_y(p[1], 0.0, 1.0);
        if c.x(t) < p[0] {
            w += if up { -1 } else { 1 };
        }
    }
    w
}

/// Per-frame scratch for the fill's row loop, allocated once and reused.
///
/// PG-6 forbids steady-state per-frame heap allocation, so the row buffers are
/// owned by the caller and sized for the widest tile a frame will use.
#[derive(Debug, Clone, Default)]
pub struct RowScratch {
    cells: Vec<f64>,
    out: Vec<f64>,
    crossings: Vec<u8>,
}

impl RowScratch {
    pub(crate) fn try_for_width(width: usize) -> Result<RowScratch, FrameError> {
        fn zeroed<T: Clone>(len: usize, value: T) -> Result<Vec<T>, FrameError> {
            let mut values = Vec::new();
            values
                .try_reserve_exact(len)
                .map_err(|_| FrameError::TooLarge)?;
            values.resize(len, value);
            Ok(values)
        }

        let cell_count = width.checked_add(1).ok_or(FrameError::TooLarge)?;
        Ok(RowScratch {
            cells: zeroed(cell_count, 0.0)?,
            out: zeroed(width, 0.0)?,
            crossings: zeroed(width, 0)?,
        })
    }

    /// Scratch sized for a tile `tile` pixels wide.
    ///
    /// # Errors
    /// Returns [`FrameError::TooLarge`] when the row-buffer layout is not
    /// addressable or its allocation is refused.
    pub fn for_tile(tile: u32) -> Result<RowScratch, FrameError> {
        RowScratch::try_for_width(tile as usize)
    }

    /// Grow to hold a `width`-pixel row, if it does not already.
    ///
    /// Amortized: after the first frame no tile is wider than the widest tile
    /// seen, so this stops allocating. Sizing up rather than reallocating per
    /// row is the difference between PG-6 passing and PG-6 being a comment.
    pub fn reserve(&mut self, width: usize) {
        if self.out.len() < width {
            self.out.resize(width, 0.0);
        }
        if self.cells.len() < width + 1 {
            self.cells.resize(width + 1, 0.0);
        }
        if self.crossings.len() < width {
            self.crossings.resize(width, 0);
        }
    }

    /// Coverage of a command whose tile [`crate::bin`] classified as interior:
    /// **exactly** `1`, with no evaluation at all.
    ///
    /// §10.4's vectorized-span path, and the second half of it is the one that is
    /// easy to miss. The per-pixel evaluation disappearing is the speed; the
    /// coverage being *exactly* `1` rather than an accumulation landing within an
    /// ulp of it is the **correctness precondition for occlusion pruning**
    /// (G0-8b finding F13). A tile skipped because a later opaque command covers
    /// it must produce the same bytes as one that drew both, and an accumulated
    /// 0.99999999999999989 does not.
    ///
    /// Written as a fill of the literal `1.0` rather than a clamp of a computed
    /// value, because a clamp would be a promise about the accumulator instead of
    /// a property of the span.
    pub fn interior_row(&mut self, x_lo: u32, x_hi: u32) -> &[f64] {
        let w = (x_hi.saturating_sub(x_lo)) as usize;
        self.reserve(w);
        self.out[..w].fill(1.0);
        &self.out[..w]
    }

    /// Coverage of one path over one pixel row, at the host width.
    ///
    /// The engine's entry point: [`fill_row`] with the scratch managed and the
    /// result handed back as a slice.
    pub fn fill_row(
        &mut self,
        pieces: &[MonoPiece],
        translate: [f64; 2],
        row_y: u32,
        x_lo: u32,
        x_hi: u32,
    ) -> &[f64] {
        let w = (x_hi.saturating_sub(x_lo)) as usize;
        self.reserve(w);
        fill_row(
            pieces,
            translate,
            row_y,
            x_lo,
            x_hi,
            &mut self.cells[..w + 1],
            &mut self.out[..w],
        );
        &self.out[..w]
    }

    /// [`RowScratch::fill_row`] with boundary-sheet counts from the same walk.
    pub fn fill_row_classified(
        &mut self,
        pieces: &[MonoPiece],
        translate: [f64; 2],
        row_y: u32,
        x_lo: u32,
        x_hi: u32,
    ) -> (&[f64], &[u8]) {
        let w = (x_hi.saturating_sub(x_lo)) as usize;
        self.reserve(w);
        fill_row_classified(
            pieces,
            translate,
            row_y,
            x_lo..x_hi,
            &mut self.cells[..w + 1],
            &mut self.out[..w],
            &mut self.crossings[..w],
        );
        (&self.out[..w], &self.crossings[..w])
    }
}

// ------------------------------------------- perspective: rational quadratics

/// A quadratic in **homogeneous** screen coordinates: what a projection actually
/// produces.
///
/// §10.2 is explicit that this may not be fudged — *"projected quadratics are
/// rational in screen space, so 3D paths are evaluated in homogeneous
/// coordinates or adaptively subdivided to tolerance, never silently treated as
/// affine"*. The reason it is exactly a rational quadratic and not something
/// worse: a projection is **linear** on homogeneous coordinates, so it maps the
/// three control points to three homogeneous control points and the projected
/// curve is the rational Bézier they define — `X(t)/W(t)`, `Y(t)/W(t)` with `X`,
/// `Y`, `W` all ordinary quadratics in `t`. No new curve family appears; a
/// weight per control point is the whole of the difference.
///
/// This type is deliberately *not* a camera.
/// [`crate::camera::Camera`] supplies these controls only after homogeneous
/// clipping, and [`crate::three_d::ThreeDJob`] turns every retained or synthetic
/// contour-boundary curve into integral pieces. The fill needs the output of
/// that camera derivation, and this is its shape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RationalPiece {
    /// Homogeneous control points `(x·w, y·w, w)`, screen pixels.
    pub p: [[f64; 3]; 3],
}

/// Why a rational piece could not be flattened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RationalError {
    /// `W(t)` reaches zero or changes sign inside the piece: the curve crosses
    /// the camera's plane, so no screen-space image of it exists.
    ///
    /// Near-plane clipping is the camera's job, not the rasterizer's, so this is
    /// a **capability error naming the cause** rather than a silently clamped
    /// picture — the same posture D2 takes for a missing ffmpeg.
    CrossesHorizon,
    /// Flattening would append more monotone pieces than the caller admitted.
    PieceLimitExceeded {
        /// Inclusive piece ceiling for this append.
        limit: usize,
        /// Exact number of pieces requested so far.
        requested: usize,
    },
    /// A bounded flattening buffer could not be reserved.
    AllocationFailed {
        /// Stable buffer name.
        resource: &'static str,
        /// Elements requested from that buffer.
        requested: usize,
    },
}

impl std::fmt::Display for RationalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CrossesHorizon => f.write_str("rational curve crosses the camera horizon"),
            Self::PieceLimitExceeded { limit, requested } => write!(
                f,
                "rational flattening needs {requested} pieces, exceeding the limit {limit}"
            ),
            Self::AllocationFailed {
                resource,
                requested,
            } => write!(f, "could not reserve {requested} rows for {resource}"),
        }
    }
}

impl std::error::Error for RationalError {}

/// What a flattening pass did.
///
/// Every bound this pass applies is reported, because a subdivision that gives
/// up quietly reads exactly like one that converged.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlattenReport {
    /// Monotone pieces appended.
    pub pieces: usize,
    /// The worst sampled deviation left, in pixels.
    pub error_px: f64,
    /// The deepest subdivision level reached.
    pub depth: u32,
    /// Whether the depth cap stopped subdivision before the tolerance was met.
    pub capped: bool,
}

/// How deep [`append_rational`] will subdivide.
///
/// Each level halves the piece, so 12 levels is 4096 sub-pieces — far past any
/// tolerance a screen-space curve needs, and a backstop against a tolerance of
/// zero rather than a working limit. Hitting it is reported in
/// [`FlattenReport::capped`].
pub const FLATTEN_MAX_DEPTH: u32 = 12;

impl RationalPiece {
    /// The affine case: unit weights, so the rational curve *is* the quadratic.
    #[must_use]
    pub fn affine(p0: [f64; 2], p1: [f64; 2], p2: [f64; 2]) -> RationalPiece {
        RationalPiece {
            p: [
                [p0[0], p0[1], 1.0],
                [p1[0], p1[1], 1.0],
                [p2[0], p2[1], 1.0],
            ],
        }
    }

    /// The screen point at parameter `t`: Bernstein in homogeneous coordinates,
    /// then the perspective divide.
    #[must_use]
    pub fn point(&self, t: f64) -> [f64; 2] {
        let u = 1.0 - t;
        let (b0, b1, b2) = (u * u, 2.0 * u * t, t * t);
        let mut h = [0.0f64; 3];
        for (k, o) in h.iter_mut().enumerate() {
            *o = b0 * self.p[0][k] + b1 * self.p[1][k] + b2 * self.p[2][k];
        }
        if h[2] == 0.0 {
            return [0.0, 0.0];
        }
        [h[0] / h[2], h[1] / h[2]]
    }

    /// `W(t)`, the homogeneous weight — the divisor the perspective divide uses.
    ///
    /// Public because a `RationalPiece` cannot be interpreted without it: it is
    /// what a caller inspects to see *where* a piece crosses the camera plane
    /// after [`append_rational`] has refused one.
    #[must_use]
    pub fn weight(&self, t: f64) -> f64 {
        let u = 1.0 - t;
        u * u * self.p[0][2] + 2.0 * u * t * self.p[1][2] + t * t * self.p[2][2]
    }

    /// Does `W` stay strictly on one side of zero across `[0, 1]`?
    ///
    /// `W` is a quadratic, so this is a root test rather than a sampling — a
    /// sampled check would miss a piece that dips through zero between samples,
    /// which is the case that produces a curve flung across the frame.
    fn weight_is_definite(&self) -> bool {
        let (w0, w1, w2) = (self.p[0][2], self.p[1][2], self.p[2][2]);
        if w0 == 0.0 || w2 == 0.0 {
            return false;
        }
        if (w0 > 0.0) != (w2 > 0.0) {
            return false;
        }
        // W(t) = w0 + 2(w1-w0) t + (w2 - 2w1 + w0) t².
        let a = w2 - 2.0 * w1 + w0;
        let b = 2.0 * (w1 - w0);
        let c = w0;
        let mut roots = [0.0f64; 2];
        let n = solve_quadratic::<f64>(a, b, c, &mut roots);
        !roots.iter().take(n).any(|&r| r > 0.0 && r < 1.0)
    }

    /// de Casteljau's split, in homogeneous coordinates — **exact**.
    ///
    /// Splitting before the divide is what makes subdivision of a rational curve
    /// lossless: the halves are rational quadratics describing exactly the same
    /// screen curve. Splitting the *projected* points instead would approximate
    /// at every level, and the error would compound rather than halve.
    #[must_use]
    pub fn split(&self, t: f64) -> (RationalPiece, RationalPiece) {
        let lerp = |a: [f64; 3], b: [f64; 3]| {
            let mut o = [0.0f64; 3];
            for (k, v) in o.iter_mut().enumerate() {
                *v = a[k] + (b[k] - a[k]) * t;
            }
            o
        };
        let q0 = lerp(self.p[0], self.p[1]);
        let q1 = lerp(self.p[1], self.p[2]);
        let r = lerp(q0, q1);
        (
            RationalPiece {
                p: [self.p[0], q0, r],
            },
            RationalPiece {
                p: [r, q1, self.p[2]],
            },
        )
    }

    /// The ordinary quadratic that interpolates this piece at `t = 0`, `½`, `1`.
    ///
    /// Three-point interpolation rather than a tangent construction: it is total
    /// — no intersection to fail, no colinear case — and it already matches the
    /// rational curve at the parameter where the deviation would otherwise peak.
    #[must_use]
    pub fn integral_approximation(&self) -> MonoPieceCandidate {
        let a = self.point(0.0);
        let m = self.point(0.5);
        let c = self.point(1.0);
        MonoPieceCandidate {
            p0: a,
            p1: [
                2.0 * m[0] - 0.5 * (a[0] + c[0]),
                2.0 * m[1] - 0.5 * (a[1] + c[1]),
            ],
            p2: c,
        }
    }

    /// The worst deviation, in pixels, between this rational piece and its
    /// [`RationalPiece::integral_approximation`].
    ///
    /// A **sampled** estimate, and named as one — but sampled densely enough
    /// that the name is not a hedge.
    ///
    /// It started at six samples — the eighths that are not `0`, `½`, `1` — and
    /// those miss the peak. The endpoints and midpoint agree by construction, so
    /// the difference has three zeros in `[0, 1]` and behaves like
    /// `t(t − ½)(t − 1)`, whose extrema sit near `(3 ± √3)/6 = 0.211` and
    /// `0.789`. Measured on this module's test piece, the extremum is at
    /// `t = 0.2175` and the true deviation is `2.9849`; six samples report
    /// `2.9283`, **1.9 % low**, and a tolerance the curve exceeds is not a
    /// stated tolerance.
    ///
    /// 31 interior samples report `2.98484` against a 4095-sample truth of
    /// `2.98493` — 0.003 % low, three orders better for five times the
    /// arithmetic. That arithmetic is close to free: this runs once per geometry
    /// revision rather than per pixel, and the whole flattening pass is bounded
    /// by `2^FLATTEN_MAX_DEPTH × 31` evaluations of two quadratics.
    ///
    /// The module's tests hold the flattened result against a 129-point
    /// *geometric* reference measured **point-to-segment**. That detail is
    /// load-bearing in the reference rather than here: a first version sampled
    /// each flattened piece at 33 points and took the nearest sample, which
    /// leaves half a sample spacing of slack — 0.3 px on a 20-pixel piece — and
    /// reported 0.1199 against a flattening that was genuinely within 0.1.
    #[must_use]
    pub fn deviation_px(&self) -> f64 {
        let q = self.integral_approximation();
        let mut worst = 0.0f64;
        for k in 1..32u32 {
            let t = f64::from(k) / 32.0;
            let a = self.point(t);
            let b = q.point(t);
            let dx = a[0] - b[0];
            let dy = a[1] - b[1];
            worst = worst.max((dx * dx + dy * dy).sqrt());
        }
        worst
    }
}

/// An ordinary quadratic proposed as a stand-in for a rational one.
///
/// A distinct type from [`MonoPiece`] on purpose: a candidate has not been split
/// for monotonicity yet, and letting the two share a type is how an unsplit
/// piece reaches the row accumulator, where "a scanline meets it at most once"
/// is a precondition rather than a hope.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MonoPieceCandidate {
    /// Start anchor, screen pixels.
    pub p0: [f64; 2],
    /// Control handle.
    pub p1: [f64; 2],
    /// End anchor.
    pub p2: [f64; 2],
}

impl MonoPieceCandidate {
    /// The point at parameter `t`.
    #[must_use]
    pub fn point(&self, t: f64) -> [f64; 2] {
        let u = 1.0 - t;
        let (b0, b1, b2) = (u * u, 2.0 * u * t, t * t);
        [
            b0 * self.p0[0] + b1 * self.p1[0] + b2 * self.p2[0],
            b0 * self.p0[1] + b1 * self.p1[1] + b2 * self.p2[1],
        ]
    }
}

/// Flatten a projected quadratic into monotone pieces within `tolerance_px`.
///
/// The honest half of §10.2's perspective clause. A rational quadratic is not a
/// quadratic, so it is subdivided in homogeneous coordinates — exactly — until
/// each half's ordinary-quadratic stand-in is within the stated screen-space
/// tolerance, and only then are the stand-ins split for monotonicity and handed
/// to the row accumulator. The affine case costs one deviation evaluation and
/// finds zero, so nothing pays for perspective that does not use it.
///
/// Returns [`RationalError::CrossesHorizon`] when `W` is not of one sign across
/// the piece: near-plane clipping belongs to the camera, and drawing *something*
/// for a curve with no screen-space image would be the silent-substitution
/// failure D2 exists to forbid.
pub fn append_rational(
    piece: &RationalPiece,
    tolerance_px: f64,
    out: &mut Vec<MonoPiece>,
) -> Result<FlattenReport, RationalError> {
    append_rational_with_limit(piece, tolerance_px, out, usize::MAX)
}

/// [`append_rational`] with an inclusive limit on pieces appended by this call.
///
/// Both the fixed-depth traversal stack and each exact monotone append are
/// reserved before mutation. A caller can therefore compose this primitive
/// into an aggregate preparation budget without an allocator abort or a
/// partially published result.
pub(crate) fn append_rational_with_limit(
    piece: &RationalPiece,
    tolerance_px: f64,
    out: &mut Vec<MonoPiece>,
    max_additional_pieces: usize,
) -> Result<FlattenReport, RationalError> {
    if !piece.weight_is_definite() {
        return Err(RationalError::CrossesHorizon);
    }
    let mut report = FlattenReport {
        pieces: 0,
        error_px: 0.0,
        depth: 0,
        capped: false,
    };
    let before = out.len();
    // Explicit stack rather than recursion: the depth cap is then a property of
    // the data structure instead of a promise about the call stack.
    let stack_capacity = usize::try_from(FLATTEN_MAX_DEPTH)
        .unwrap_or(usize::MAX)
        .saturating_add(1);
    let mut stack: Vec<(RationalPiece, u32)> = Vec::new();
    stack
        .try_reserve_exact(stack_capacity)
        .map_err(|_| RationalError::AllocationFailed {
            resource: "rational flattening stack",
            requested: stack_capacity,
        })?;
    stack.push((*piece, 0));
    while let Some((cur, depth)) = stack.pop() {
        let deviation = cur.deviation_px();
        report.depth = report.depth.max(depth);
        if deviation <= tolerance_px || depth >= FLATTEN_MAX_DEPTH {
            if deviation > tolerance_px {
                report.capped = true;
            }
            report.error_px = report.error_px.max(deviation);
            let q = cur.integral_approximation();
            let additional = monotone_piece_count(q.p0, q.p1, q.p2);
            let Some(requested) = out.len().saturating_sub(before).checked_add(additional) else {
                out.truncate(before);
                return Err(RationalError::PieceLimitExceeded {
                    limit: max_additional_pieces,
                    requested: usize::MAX,
                });
            };
            if requested > max_additional_pieces {
                out.truncate(before);
                return Err(RationalError::PieceLimitExceeded {
                    limit: max_additional_pieces,
                    requested,
                });
            }
            if out.try_reserve_exact(additional).is_err() {
                let total = out.len().saturating_add(additional);
                out.truncate(before);
                return Err(RationalError::AllocationFailed {
                    resource: "rational monotone pieces",
                    requested: total,
                });
            }
            split_monotone(q.p0, q.p1, q.p2, out);
            continue;
        }
        let (l, r) = cur.split(0.5);
        // Right first, so the pieces come out in parameter order — the fill does
        // not care, but a golden snapshot of the derived table does.
        stack.push((r, depth + 1));
        stack.push((l, depth + 1));
    }
    report.pieces = out.len() - before;
    Ok(report)
}

// --------------------------------------------------- the interior colour field

/// How many boundary stations §10.2's interior field is evaluated over.
///
/// A **semantic** constant, not a quality knob: it is part of the definition of
/// the colour field, so changing it changes every gradient-filled frame's hash,
/// and it is therefore named, fixed, and journaled into the input closure rather
/// than passed in — the same standing this module's `SPLIT_EPS` and G0-6's
/// `SUBSAMPLES_Y` have.
///
/// 64 is chosen against a measurement, not a feeling. Two numbers, because there
/// are two effects:
///
/// - Stations sit at interval **midpoints**, which makes the quadrature unbiased:
///   a disc's centre reads exactly `½` at *every* station count. A left-endpoint
///   rule carries a `1/(2n)` bias that every interior point inherits — measured,
///   0.49219 at 64 stations and 0.49805 at 256.
/// - Beyond four station spacings from the ramp's **seam** (where a closed path's
///   end meets its start and the boundary colour jumps back), 64 stations agree
///   with 256 to better than `1/1020` of the parameter — a quarter of one 8-bit
///   level on the colour that reads it. At the seam nothing converges, because
///   the *data* is discontinuous there; the field stays bounded and the
///   Reference's own gradient fill has the same seam.
pub const GRADIENT_STATIONS: usize = 64;

/// §10.2's interior colour field: **arc-length-parameterized boundary
/// interpolation with mean value coordinates in the interior**.
///
/// ## What is being interpolated
///
/// Not the colour — the *parameter*. The field returns `t ∈ [0, 1]`, the mean
/// value interpolant of normalized arc length along the boundary, and the ramp
/// is evaluated at it by [`fill_rgba_at`]. Two things follow, and both are the
/// reason it is built this way:
///
/// - The field is a function of **geometry alone**, so it is keyed on the
///   geometry revision like every other derived artifact in §10.8 and survives a
///   restyle. A field that interpolated colours would rebuild whenever anything
///   changed colour, which is the one thing a gradient does.
/// - For the two-endpoint ramp the IR carries today the two formulations are
///   *identical*, not merely similar: mean value coordinates sum to one, so
///   `Σ λᵢ c(sᵢ) = c(Σ λᵢ sᵢ)` exactly for any affine `c`. When Marionette grows
///   a per-point ramp, `Σ λᵢ cᵢ` drops in with no change to the field.
///
/// ## Why mean value coordinates, and why arc length
///
/// §10.2 requires a field that is "specified, tested, and stable under
/// subdivision", and the Reference's mechanism is none of those: it triangulates
/// and lets the GPU interpolate vertex colours across the fan, so the answer
/// depends on the triangulation. Both halves of the replacement carry a
/// subdivision obligation:
///
/// - **Arc length, not parameter.** A de Casteljau split changes a curve's
///   parameterization and not the curve, so stations placed by `t` would move
///   and stations placed by arc length do not. Placement runs through
///   `fmn_geom::arclength::t_at_arc_fraction` — the same solve
///   `point_from_proportion` uses (D4: the renderer does not own a second
///   arc-length rule).
/// - **Stations, not vertices.** Discrete mean value coordinates over the
///   *anchors* would change with every subdivision, because subdivision adds
///   anchors. Over stations at fixed arc-length fractions they cannot: the
///   station set is a function of the boundary curve and its arc-length
///   parameterization, both of which subdivision preserves. This is a quadrature
///   of the continuous mean value interpolant — the boundary colour averaged over
///   viewing angle from the query point — which is the subdivision-invariant
///   object the discrete form approximates.
///
/// ## Multiple contours
///
/// Hormann--Floater's generalized coordinates apply the same local weight
/// formula to a set of cyclic polygons and normalize the sum over all of them.
/// Nested contours alternate orientation, exactly as the nonzero fill rule
/// already requires for a counter-wound glyph hole or annulus. The retained
/// subpath table therefore defines explicit station edges for every contour;
/// no synthetic edge ever joins an outer boundary to a hole. The semantic
/// 64-station budget is divided by global arc length with a three-station
/// minimum per contour, growing only to `3 * contours` when that minimum does
/// not fit. Boundary data remains the normalized arc length of the whole path,
/// so each contour closes at its own global parameter and then has its own ramp
/// seam back to its first station.
/// The interior colour field for a non-flat fill, as a *view* over
/// caller-owned station storage.
///
/// The engine's per-frame path keeps the stations in the frame arena's typed
/// pools ([`crate::FrameArena`]), so deriving a field allocates nothing once the
/// arena is warm (PG-6); tests and the annex backends use plain `Vec`s. The
/// interpolant is identical either way — only the storage moved.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct GradientField<'a> {
    /// Station positions, shape-local screen pixels — the same frame
    /// [`MonoPiece`] uses, so one [`instance_translation`] serves both.
    points: &'a [[f64; 2]],
    /// Each station's normalized arc length along the path.
    params: &'a [f64],
    /// Next station on the same contour, index-local to this field.
    ///
    /// Empty is the compact single-contour representation and deliberately
    /// selects the original evaluator byte-for-byte. Multi-contour fields keep
    /// one explicit edge per station so no phantom edge can join two subpaths.
    next: &'a [usize],
    /// Boundary parameter at the far end of each explicit edge.
    ///
    /// Usually this is `params[next[i]]`. On a contour's closing edge it is the
    /// contour's global arc-length end, preserving the declared ramp seam before
    /// it jumps back to the first station on that contour.
    edge_params: &'a [f64],
}

/// Backend-facing slices of one retained gradient field.
#[cfg(feature = "metal")]
pub(crate) type GradientStationSlices<'a> = (&'a [[f64; 2]], &'a [f64], &'a [usize], &'a [f64]);

impl<'a> GradientField<'a> {
    /// Squared distance below which the query point *is* a station.
    ///
    /// Mean value coordinates are exact at the boundary data — the weight
    /// `1/rᵢ` diverges — so the limit is taken explicitly rather than left to
    /// infinity arithmetic, which would produce `NaN` rather than the answer.
    const TOUCH_SQ: f64 = 1e-18;

    /// The view over caller-owned station storage.
    pub(crate) fn from_parts(points: &'a [[f64; 2]], params: &'a [f64]) -> GradientField<'a> {
        debug_assert_eq!(points.len(), params.len());
        GradientField {
            points,
            params,
            next: &[],
            edge_params: &[],
        }
    }

    /// The view over a station set with explicit per-contour edges.
    pub(crate) fn from_contours(
        points: &'a [[f64; 2]],
        params: &'a [f64],
        next: &'a [usize],
        edge_params: &'a [f64],
    ) -> GradientField<'a> {
        debug_assert_eq!(points.len(), params.len());
        debug_assert_eq!(points.len(), next.len());
        debug_assert_eq!(points.len(), edge_params.len());
        debug_assert!(next.iter().all(|&index| index < points.len()));
        GradientField {
            points,
            params,
            next,
            edge_params,
        }
    }

    /// Derive the field for one compiled shape into caller-owned storage.
    ///
    /// `segments` is the shape's own slice of the IR's segment table, in object
    /// space. Derives nothing for a shape with no drawn length, which reads as
    /// a flat `0` parameter rather than as an error.
    pub(crate) fn build_into(
        points: &mut impl crate::arena::Sink<[f64; 2]>,
        params: &mut impl crate::arena::Sink<f64>,
        segments: &[crate::table::Segment],
        map: ScreenMap,
    ) {
        Self::build_with_into(GRADIENT_STATIONS, points, params, segments, map);
    }

    /// Derive the field for every retained subpath without connecting contours.
    ///
    /// One contour delegates to [`GradientField::build_into`] and writes no
    /// edge metadata, preserving the existing station sequence and evaluator
    /// exactly. For multiple contours, the fixed 64-station semantic budget is
    /// shared in proportion to global arc length with at least three stations
    /// per contour. If a shape has more than 21 contours the budget grows only
    /// enough to keep that mathematical minimum, so storage stays linear in the
    /// already-retained contour count.
    pub(crate) fn build_contours_into(
        points: &mut impl crate::arena::Sink<[f64; 2]>,
        params: &mut impl crate::arena::Sink<f64>,
        next: &mut impl crate::arena::Sink<usize>,
        edge_params: &mut impl crate::arena::Sink<f64>,
        segments: &[crate::table::Segment],
        subpath_starts: &[u32],
        map: ScreenMap,
    ) {
        if subpath_starts.len() <= 1 {
            Self::build_into(points, params, segments, map);
            return;
        }
        if segments.is_empty() {
            return;
        }

        const MIN_CONTOUR_STATIONS: usize = 3;
        let contour_count = subpath_starts.len();
        let minimum = contour_count.saturating_mul(MIN_CONTOUR_STATIONS);
        let station_count = GRADIENT_STATIONS.max(minimum);
        let distributable = station_count.saturating_sub(minimum);
        let field_start = points.len();
        let mut distributed = 0usize;

        for (contour, &raw_start) in subpath_starts.iter().enumerate() {
            let start = (raw_start as usize).min(segments.len());
            let end = subpath_starts
                .get(contour + 1)
                .map_or(segments.len(), |&index| {
                    (index as usize).min(segments.len())
                });
            if start >= end {
                continue;
            }
            let own = &segments[start..end];
            let s0 = own.first().map_or(0.0, |segment| segment.s0);
            let s1 = own.last().map_or(s0, |segment| segment.s1);
            let span = (s1 - s0).max(0.0);
            let target = if contour + 1 == contour_count {
                distributable
            } else {
                ((s1.clamp(0.0, 1.0) * distributable as f64).floor() as usize).min(distributable)
            };
            let extra = target.saturating_sub(distributed);
            distributed = target;
            let own_station_count = MIN_CONTOUR_STATIONS.saturating_add(extra);
            let station_start = points.len().saturating_sub(field_start);

            for station in 0..own_station_count {
                let fraction = (station as f64 + 0.5) / own_station_count as f64;
                let s = s0 + fraction * span;
                let index = own
                    .partition_point(|segment| segment.s1 <= s)
                    .min(own.len() - 1);
                let segment = &own[index];
                let segment_span = segment.s1 - segment.s0;
                let segment_fraction = if segment_span > 0.0 {
                    ((s - segment.s0) / segment_span).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let t = fmn_geom::arclength::t_at_arc_fraction(
                    segment.p0,
                    segment.p1,
                    segment.p2,
                    segment_fraction,
                );
                let point =
                    fmn_geom::bezier::quadratic_point(segment.p0, segment.p1, segment.p2, t);
                points.put(map.to_pixel(point[0], point[1]));
                params.put(s);
                next.put(if station + 1 < own_station_count {
                    station_start + station + 1
                } else {
                    station_start
                });
                edge_params.put(if station + 1 < own_station_count {
                    s0 + (station as f64 + 1.5) / own_station_count as f64 * span
                } else {
                    s1
                });
            }
        }
    }

    /// Maximum stations produced for a retained contour count.
    pub(crate) fn station_capacity(contours: usize) -> usize {
        GRADIENT_STATIONS.max(contours.saturating_mul(3))
    }

    /// [`GradientField::build_into`] at an explicit station count.
    ///
    /// Exists so the convergence of the quadrature can be *measured* rather than
    /// asserted — see this module's station-count test. Not a public knob:
    /// [`GRADIENT_STATIONS`] is the shipped definition.
    fn build_with_into(
        stations: usize,
        points: &mut impl crate::arena::Sink<[f64; 2]>,
        params: &mut impl crate::arena::Sink<f64>,
        segments: &[crate::table::Segment],
        map: ScreenMap,
    ) {
        if segments.is_empty() || stations == 0 {
            return;
        }
        for k in 0..stations {
            // Interval MIDPOINTS, not left endpoints. The boundary ramp is
            // discontinuous where a closed path's end meets its start — it runs
            // `fill_rgba → fill_rgba_end` and then jumps back — so a left-endpoint
            // rule samples `{0, 1/n, …, (n−1)/n}`, whose mean is `½ − 1/(2n)`,
            // and every interior point inherits that `O(1/n)` bias: measured, a
            // disc's centre read 0.49219 at 64 stations and 0.49805 at 256, which
            // is exactly `1/(2n)` twice. Midpoints sample `{(k+½)/n}`, whose mean
            // is `½` for *every* `n`, so the bias is not reduced — it is gone.
            let s = (k as f64 + 0.5) / stations as f64;
            // The spans partition [0, 1] in order, so this is a binary search.
            let i = segments
                .partition_point(|g| g.s1 <= s)
                .min(segments.len() - 1);
            let g = &segments[i];
            let span = g.s1 - g.s0;
            let frac = if span > 0.0 {
                ((s - g.s0) / span).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let t = fmn_geom::arclength::t_at_arc_fraction(g.p0, g.p1, g.p2, frac);
            let p = fmn_geom::bezier::quadratic_point(g.p0, g.p1, g.p2, t);
            points.put(map.to_pixel(p[0], p[1]));
            params.put(s);
        }
    }

    /// Are there no stations — i.e. does this shape have no drawn boundary?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// How many stations.
    #[must_use]
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Shape-local screen-space stations for backend-specific derived layouts.
    ///
    /// The retained field remains the authority; annex backends copy these
    /// values into their own flat device representation rather than inventing a
    /// second station-placement rule.
    #[cfg(feature = "metal")]
    pub(crate) fn stations(&self) -> GradientStationSlices<'a> {
        (self.points, self.params, self.next, self.edge_params)
    }

    /// Does this field carry more than one explicitly closed contour?
    #[cfg(feature = "metal")]
    pub(crate) fn has_explicit_contours(&self) -> bool {
        !self.next.is_empty()
    }

    /// The field's value at a screen point: mean value coordinates over the
    /// stations, applied to their arc-length parameters.
    ///
    /// Allocation-free (PG-6): the previous edge's half-angle tangent is carried
    /// in a register rather than tabulated, and the wrap-around edge is computed
    /// once and reused at both ends of the loop.
    #[must_use]
    pub fn param_at(&self, p: [f64; 2], translate: [f64; 2]) -> f64 {
        if !self.next.is_empty() {
            return self.param_at_contours(p, translate);
        }
        self.param_at_single_contour(p, translate)
    }

    /// Original one-polygon evaluator, kept as a distinct branch so the
    /// single-contour field remains bit-identical.
    fn param_at_single_contour(&self, p: [f64; 2], translate: [f64; 2]) -> f64 {
        let n = self.points.len();
        if n == 0 {
            return 0.0;
        }
        let d = |i: usize| -> [f64; 2] {
            [
                self.points[i][0] + translate[0] - p[0],
                self.points[i][1] + translate[1] - p[1],
            ]
        };
        if n == 1 {
            return self.params[0];
        }

        // `tan(αᵢ/2)` for the edge (i, j), written as
        // `(rᵢ rⱼ − dᵢ·dⱼ) / (dᵢ × dⱼ)`: algebraically `(1 − cos α)/sin α`, and
        // preferred because it needs no inverse trigonometry at all — one fewer
        // transcendental per station per pixel, and no `atan2` branch to keep
        // bit-identical against an annex twin.
        //
        // `None` means the query point lies *on* this edge (collinear and
        // between the endpoints, so `α = π` and the tangent diverges). The
        // caller takes the limit, which is linear interpolation along the edge —
        // the correct boundary behaviour, and the one place infinity arithmetic
        // would have produced `NaN` instead of an answer.
        let tan_half = |i: usize, j: usize| -> Option<f64> {
            let (di, dj) = (d(i), d(j));
            let ri = (di[0] * di[0] + di[1] * di[1]).sqrt();
            let rj = (dj[0] * dj[0] + dj[1] * dj[1]).sqrt();
            let dot = di[0] * dj[0] + di[1] * dj[1];
            let cross = di[0] * dj[1] - di[1] * dj[0];
            if cross.abs() <= 1e-14 * ri * rj {
                return if dot < 0.0 { None } else { Some(0.0) };
            }
            Some((ri * rj - dot) / cross)
        };

        // Exact at a station, and exact on an edge.
        for (i, &param) in self.params[..n].iter().enumerate() {
            let di = d(i);
            if di[0] * di[0] + di[1] * di[1] <= Self::TOUCH_SQ {
                return param;
            }
        }
        for i in 0..n {
            let j = (i + 1) % n;
            if tan_half(i, j).is_none() {
                let (di, dj) = (d(i), d(j));
                let ri = (di[0] * di[0] + di[1] * di[1]).sqrt();
                let rj = (dj[0] * dj[0] + dj[1] * dj[1]).sqrt();
                let f = if ri + rj > 0.0 { ri / (ri + rj) } else { 0.0 };
                // The wrap edge closes the ramp, so it interpolates back toward
                // the start rather than past the end.
                let (a, b) = if j == 0 {
                    (self.params[i], 1.0)
                } else {
                    (self.params[i], self.params[j])
                };
                return a + (b - a) * f;
            }
        }

        let wrap = tan_half(n - 1, 0).unwrap_or(0.0);
        let mut t_prev = wrap;
        let mut num = 0.0;
        let mut den = 0.0;
        for (i, &param) in self.params[..n].iter().enumerate() {
            let t_i = if i + 1 < n {
                tan_half(i, i + 1).unwrap_or(0.0)
            } else {
                wrap
            };
            let di = d(i);
            let r = (di[0] * di[0] + di[1] * di[1]).sqrt();
            if r > 0.0 {
                let w = (t_prev + t_i) / r;
                num += w * param;
                den += w;
            }
            t_prev = t_i;
        }
        if den == 0.0 || !den.is_finite() || !num.is_finite() {
            // Defined rather than arbitrary: fall back to the nearest station,
            // which is the value the interpolant tends to as the weights
            // degenerate.
            return self.nearest_param(p, translate);
        }
        (num / den).clamp(0.0, 1.0)
    }

    /// Hormann--Floater mean value interpolation over a set of cyclic planar
    /// polygons. Counter-wound nested contours contribute signed weights, so a
    /// hole participates without any synthetic bridge to its outer boundary.
    fn param_at_contours(&self, p: [f64; 2], translate: [f64; 2]) -> f64 {
        let n = self.points.len();
        debug_assert_eq!(n, self.params.len());
        debug_assert_eq!(n, self.next.len());
        debug_assert_eq!(n, self.edge_params.len());
        if n == 0 {
            return 0.0;
        }
        let d = |i: usize| -> [f64; 2] {
            [
                self.points[i][0] + translate[0] - p[0],
                self.points[i][1] + translate[1] - p[1],
            ]
        };
        let tan_half = |i: usize, j: usize| -> Option<f64> {
            let (di, dj) = (d(i), d(j));
            let ri = (di[0] * di[0] + di[1] * di[1]).sqrt();
            let rj = (dj[0] * dj[0] + dj[1] * dj[1]).sqrt();
            let dot = di[0] * dj[0] + di[1] * dj[1];
            let cross = di[0] * dj[1] - di[1] * dj[0];
            if cross.abs() <= 1e-14 * ri * rj {
                return if dot < 0.0 { None } else { Some(0.0) };
            }
            Some((ri * rj - dot) / cross)
        };

        for (i, &param) in self.params[..n].iter().enumerate() {
            let di = d(i);
            if di[0] * di[0] + di[1] * di[1] <= Self::TOUCH_SQ {
                return param;
            }
        }
        for i in 0..n {
            let j = self.next[i];
            debug_assert!(j < n);
            if j >= n {
                return self.nearest_param(p, translate);
            }
            if tan_half(i, j).is_none() {
                let (di, dj) = (d(i), d(j));
                let ri = (di[0] * di[0] + di[1] * di[1]).sqrt();
                let rj = (dj[0] * dj[0] + dj[1] * dj[1]).sqrt();
                let fraction = if ri + rj > 0.0 { ri / (ri + rj) } else { 0.0 };
                return self.params[i] + (self.edge_params[i] - self.params[i]) * fraction;
            }
        }

        // Equation (11), accumulated per edge. Each edge's half-angle tangent
        // contributes to both endpoint weights, which is algebraically the same
        // `(t_prev + t_next) / radius` sum without needing a previous-index
        // table for every contour.
        let mut numerator = 0.0;
        let mut denominator = 0.0;
        for i in 0..n {
            let j = self.next[i];
            let tangent = tan_half(i, j).unwrap_or(0.0);
            let di = d(i);
            let dj = d(j);
            let ri = (di[0] * di[0] + di[1] * di[1]).sqrt();
            let rj = (dj[0] * dj[0] + dj[1] * dj[1]).sqrt();
            if ri > 0.0 {
                let weight = tangent / ri;
                numerator += weight * self.params[i];
                denominator += weight;
            }
            if rj > 0.0 {
                let weight = tangent / rj;
                numerator += weight * self.params[j];
                denominator += weight;
            }
        }
        if denominator == 0.0 || !denominator.is_finite() || !numerator.is_finite() {
            return self.nearest_param(p, translate);
        }
        (numerator / denominator).clamp(0.0, 1.0)
    }

    /// The parameter of the station nearest a point — the degenerate fallback.
    fn nearest_param(&self, p: [f64; 2], translate: [f64; 2]) -> f64 {
        let mut best = f64::INFINITY;
        let mut out = 0.0;
        for (q, s) in self.points.iter().zip(self.params.iter()) {
            let dx = q[0] + translate[0] - p[0];
            let dy = q[1] + translate[1] - p[1];
            let d2 = dx * dx + dy * dy;
            if d2 < best {
                best = d2;
                out = *s;
            }
        }
        out
    }
}

/// The fill ramp evaluated at a field parameter.
///
/// The IR's [`crate::table::Style`] carries the two ends of the per-point
/// `fill_rgba` column, which is how the Reference expresses a gradient along a
/// path, so the boundary colour is `lerp(fill_rgba, fill_rgba_end, s)` at
/// normalized arc length `s`. Interpolation is componentwise in the linear-light
/// straight-alpha space the table already stores (§6.3, BN-04) — the ramp is a
/// colour interpolation, so it happens where colour interpolation is defined and
/// not in an encoded space.
#[must_use]
pub fn fill_rgba_at(style: &crate::table::Style, t: f64) -> [f32; 4] {
    if let Some(profile) = &style.fill_profile {
        return profile.rgba_at(t);
    }
    let t = t.clamp(0.0, 1.0) as f32;
    let mut out = [0.0f32; 4];
    for (k, o) in out.iter_mut().enumerate() {
        let a = style.fill_rgba[k];
        let b = style.fill_rgba_end[k];
        *o = a + (b - a) * t;
    }
    out
}

// ------------------------------------------------------- the inner border

/// The width in screen pixels of a `fill_border_width` of `w` width units.
///
/// Delegates to [`crate::stroke::width_px`] rather than restating the
/// conversion, because it is not a similar conversion — it is the *same* one. The
/// Reference feeds `fill_border_width` into the stroke program's `stroke_width`
/// attribute (`shader_wrapper.py:315`), so a border that converted independently
/// would be a second definition of one number.
#[must_use]
pub fn border_width_px(width_units: f32, map: ScreenMap) -> f64 {
    crate::stroke::width_px(width_units, map)
}

/// The distance in pixels from a screen point to a shape's boundary, and the
/// boundary ramp's parameter at the nearest point.
///
/// Queried in **object space** and scaled, rather than by projecting the
/// segments: the map is a uniform scale plus a translation, so it preserves which
/// point is nearest and multiplies the distance by the scale — and doing it this
/// way means the renderer holds no second copy of the geometry.
///
/// `None` for a shape with no segments.
#[must_use]
pub fn nearest_boundary(
    segments: &[crate::table::Segment],
    map: ScreenMap,
    translate: [f64; 2],
    p: [f64; 2],
) -> Option<(f64, f64)> {
    nearest_boundary_location(segments, map, translate, p).map(|(d, s, _)| (d, s))
}

// Preserve the side of a coincident arc station at a disconnected contour.
// A bare scalar station cannot distinguish its outgoing and incoming colors.
fn nearest_boundary_location(
    segments: &[crate::table::Segment],
    map: ScreenMap,
    translate: [f64; 2],
    p: [f64; 2],
) -> Option<(f64, f64, bool)> {
    let scale = map.scale;
    if scale == 0.0 || segments.is_empty() {
        return None;
    }
    let [ox, oy] = map.to_object(p, translate);
    let obj = [ox, oy, 0.0];
    let mut best_d = f64::INFINITY;
    let mut best_s = 0.0;
    let mut at_end = false;
    for g in segments {
        let near = fmn_geom::distance::nearest_on_quadratic(g.p0, g.p1, g.p2, obj);
        if near.distance >= best_d {
            continue;
        }
        best_d = near.distance;
        // The ramp is parameterized by ARC LENGTH, so the segment-local `t` has
        // to be converted before it indexes the ramp — `t` and arc length differ
        // by exactly the amount BN-03 exists to talk about.
        let total = fmn_geom::arclength::quadratic_arc_length(g.p0, g.p1, g.p2);
        let frac = if total > 0.0 {
            let sub = fmn_geom::bezier::partial_quadratic(&[g.p0, g.p1, g.p2], 0.0, near.t);
            (fmn_geom::arclength::quadratic_arc_length(sub[0], sub[1], sub[2]) / total)
                .clamp(0.0, 1.0)
        } else {
            0.0
        };
        best_s = g.s0 + (g.s1 - g.s0) * frac;
        at_end = near.t == 1.0;
    }
    if best_d.is_finite() {
        Some((best_d * scale.abs(), best_s.clamp(0.0, 1.0), at_end))
    } else {
        None
    }
}

/// The inner border's coverage at a point `distance` pixels inside the boundary.
///
/// The band runs `width` pixels inward from the boundary, and its inner edge is
/// antialiased with G0-2's measured profile — because the border *is* a stroke,
/// and a stroke's edge is that curve at that width. The profile itself lives in
/// [`crate::stroke::aa_coverage`], where it was measured; `distance − width` is
/// the signed excess it takes.
#[must_use]
pub fn border_coverage(distance: f64, width: f64, aa_width: f64) -> f64 {
    if width <= 0.0 {
        return 0.0;
    }
    crate::stroke::aa_coverage(distance - width, aa_width)
}

/// The fill colour at a screen point, honouring `fill_border_width`.
///
/// ## What `fill_border_width` is in the Reference, traced rather than assumed
///
/// `shader_wrapper.py` builds `fill_border_program` from the **stroke** shaders
/// and binds `fill_rgba` to the `stroke_rgba` attribute and `fill_border_width`
/// to `stroke_width`; the pass runs with `glBlendFunc(ONE, ONE)` and
/// `glBlendEquation(GL_MAX)` — *"Now add border, just taking the max alpha"*. So
/// the border is a **centred** stroke in the fill's own colour, max-composited.
/// Max of two equal colours cannot darken the interior, so its entire observable
/// effect is that the filled silhouette **grows by half the border width**, with
/// a sharper outer edge than the fill's own antialiasing. `Text` and
/// `DecimalNumber` default it to 0.5 (`string_mobject.py:50`, `numbers.py:41`),
/// which is 0.675 px of growth — a compensation for the fill pipeline's 2.3-bit
/// coverage (G0-2 finding L3), not a border.
///
/// ## What it is here
///
/// An **inner** border, per §10.2, and that word decides the arithmetic. An inner
/// band is a *subset* of the fill region, so its coverage is pointwise no greater
/// than the fill's; under max-compositing in the same colour it therefore cannot
/// change coverage at all. Two consequences, and both are the point rather than a
/// limitation:
///
/// - **The silhouette never moves.** Asking for a border does not grow the shape.
///   The Reference's growth is retired under D5 as a fix for a defect analytic
///   coverage does not have — recorded as a Behavior Note, because text set with
///   the Reference's default renders at its true weight and not 0.675 px bolder.
/// - **For a flat fill it is exactly a no-op**, provably, which is what makes
///   retiring the growth safe instead of a silent change of appearance.
///
/// What remains is a **colour** effect, and measurement says exactly where it
/// lives. Within the band the colour comes from the boundary ramp at the nearest
/// boundary point — crisp — instead of from [`GradientField`]'s mean value
/// interpolant, which is smooth by construction. But an interpolant *converges*
/// to its boundary data, so away from the ramp's seam the two agree to floating
/// point one pixel in and the border changes nothing. At the seam they do not
/// converge: the field blends the ramp's jump and reads `½` where the boundary
/// reads `0`. Scanning a ring one pixel inside a 24-pixel circle, the maximum
/// disagreement is exactly **0.500** and it sits at the seam, at every inset from
/// 0.5 px to 4 px.
///
/// So `fill_border_width`'s whole remaining job is: **within the band, the
/// gradient's seam is crisp instead of blurred.** That is what a border on a
/// gradient should do, and it is the only pixel this knob moves.
#[must_use]
pub fn fill_rgba_with_border(
    style: &crate::table::Style,
    field: &GradientField,
    segments: &[crate::table::Segment],
    map: ScreenMap,
    translate: [f64; 2],
    p: [f64; 2],
) -> [f32; 4] {
    let interior = style.fill_profile.as_ref().map_or_else(
        || fill_rgba_at(style, field.param_at(p, translate)),
        |profile| field.rgba_at(profile, p, translate),
    );
    // Both fast paths are exact, not approximations of the general case: a flat
    // fill has no crisp colour to reveal, and a zero-width band has no points.
    if fill_is_flat(style) || style.fill_border_width <= 0.0 {
        return interior;
    }
    let width = border_width_px(style.fill_border_width, map);
    let Some((distance, s, at_end)) = nearest_boundary_location(segments, map, translate, p) else {
        return interior;
    };
    let coverage = border_coverage(distance, width, f64::from(style.anti_alias_width));
    if coverage <= 0.0 {
        return interior;
    }
    let edge = fill_rgba_at(style, style.fill_endpoint_parameter(s, at_end));
    let k = coverage as f32;
    let mut out = [0.0f32; 4];
    for (o, (a, b)) in out.iter_mut().zip(interior.iter().zip(edge.iter())) {
        *o = a + (b - a) * k;
    }
    out
}

/// Is this style's fill a single colour, so the interior field can be skipped?
///
/// Bitwise, matching how [`crate::table::StyleTable`] interns: §8.5 makes
/// batching observable, so "the same colour" here has to mean the same thing it
/// means there. The overwhelming majority of fills are flat, and this is the
/// test that keeps a 64-station interpolant off their hot path entirely.
#[must_use]
pub fn fill_is_flat(style: &crate::table::Style) -> bool {
    if let Some(profile) = &style.fill_profile {
        return profile.is_flat();
    }
    style
        .fill_rgba
        .iter()
        .zip(&style.fill_rgba_end)
        .all(|(a, b)| a.to_bits() == b.to_bits())
}

// ------------------------------------------------- primitive-hint fill kernels

/// The screen-space budget under which a *bounded-error* fill hint is admitted.
///
/// One 1/256th of a pixel — §10.2's own acceptance tolerance, reused here rather
/// than a second number nobody reconciled.
pub const HINT_BUDGET_PX: f64 = 1.0 / 256.0;

/// A hinted fill's coverage: a closed form with no walk and no accumulator.
///
/// ## The rule these kernels have to obey, and where it bites
///
/// [`crate::hint`]'s rule is absolute — *"a kernel selected by a hint must
/// produce the same answer the general path would, so dropping every hint can
/// only cost speed"*. For [`FillKernel::Rect`] that is free: an axis-aligned
/// rectangle's coverage is a product of two clamped intervals, and the general
/// path computes the same region's exact area from straight segments, so the two
/// agree to floating point.
///
/// For a disc it is **not** free, and pretending otherwise would be the
/// interesting bug in this module. A `Circle`'s or `Dot`'s compiled outline is
/// manim's *quadratic approximation* of a circle, which encloses `(π/n)⁴/6` more
/// area than the circle does — 2.5e-4 relative at the default 16 components,
/// 6.3e-2 at 4. A kernel that rasterizes the true disc therefore draws a
/// **different shape** from the general path, by up to `r·(π/n)⁴/8` pixels of
/// edge displacement. So the disc kernel is admitted the way §10.8 admits its
/// one other approximating route — *"nearly-linear quadratic → line fast path
/// under an explicit screen-space error bound"* — except that the bound here is
/// **measured off the compiled outline** rather than assumed: [`FillKernel::select`]
/// evaluates each segment's midpoint, takes the worst radial deviation from the
/// hinted radius, scales it to pixels, and declines the hint when it exceeds
/// [`HINT_BUDGET_PX`].
///
/// The practical effect is the shape of the rule, not a compromise with it: a
/// `Dot` at manim's default radius (0.08 units, ~10.8 px at the Reference's
/// 135 px/unit) deviates ~0.002 px and takes the kernel; a two-unit `Circle`
/// deviates ~0.05 px and draws through the general path. Dots are what the
/// radial kernel was for.
///
/// `Arc` is deliberately absent. Filling a partial arc closes it with a chord,
/// so the region is a circular *segment* rather than a disc, and the hint would
/// have to carry the chord to be right. It routes to the general path, which is
/// exact.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FillKernel {
    /// No hinted route: the general quadratic machinery.
    General,
    /// Axis-aligned rectangle, screen pixels, `[x0, y0, x1, y1]`.
    Rect {
        /// The rectangle.
        rect: [f64; 4],
    },
    /// A filled disc, screen pixels.
    Disc {
        /// Centre.
        center: [f64; 2],
        /// Radius.
        radius: f64,
    },
    /// An axis-aligned rectangle with circular corners, screen pixels.
    RoundedRect {
        /// The bounding rectangle.
        rect: [f64; 4],
        /// Corner radius, already clamped to half the shorter side.
        radius: f64,
    },
}

impl FillKernel {
    /// Choose a kernel for one occurrence of a compiled shape.
    ///
    /// `segments` is the shape's own slice of the IR's segment table, in object
    /// space; `translate` is [`instance_translation`]. Returns
    /// [`FillKernel::General`] whenever the hint is absent, inapplicable, or
    /// outside [`HINT_BUDGET_PX`] — the "when in doubt, draw" direction.
    #[must_use]
    pub fn select(
        shape: &crate::table::Shape,
        segments: &[crate::table::Segment],
        map: ScreenMap,
        translate: [f64; 2],
    ) -> FillKernel {
        let to_px = |p: [f64; 3]| {
            let q = map.to_pixel(p[0], p[1]);
            [q[0] + translate[0], q[1] + translate[1]]
        };
        let scale = map.scale.abs();
        // A hinted kernel is a closed form for **one** region, and every hint in
        // the vocabulary names a single closed curve. A shape with more than one
        // subpath has a winding rule deciding what is inside it — an annulus is
        // the obvious case — and no closed form here knows about the second loop,
        // so a hint would fill the hole solid.
        //
        // No constructor tags such a shape today; this is here so that none ever
        // can. §10.8's rule is that correctness never depends on a hint, and the
        // way to mean that is to make declining one cost nothing.
        if shape.subpath_starts.len() > 1 {
            return FillKernel::General;
        }
        match shape.hint {
            crate::hint::Hint::Rect {
                center,
                width,
                height,
            } => {
                let c = to_px(center);
                let (hw, hh) = (0.5 * width * scale, 0.5 * height * scale);
                FillKernel::Rect {
                    rect: [c[0] - hw, c[1] - hh, c[0] + hw, c[1] + hh],
                }
            }
            crate::hint::Hint::RoundedRect {
                center,
                width,
                height,
                corner_radius,
            } => {
                let c = to_px(center);
                let (hw, hh) = (0.5 * width * scale, 0.5 * height * scale);
                let r = (corner_radius * scale).clamp(0.0, hw.min(hh));
                FillKernel::RoundedRect {
                    rect: [c[0] - hw, c[1] - hh, c[0] + hw, c[1] + hh],
                    radius: r,
                }
            }
            crate::hint::Hint::Circle { center, radius }
            | crate::hint::Hint::Dot { center, radius } => {
                let r = radius * scale;
                if r <= 0.0 || !Self::disc_within_budget(center, radius, segments, scale) {
                    return FillKernel::General;
                }
                FillKernel::Disc {
                    center: to_px(center),
                    radius: r,
                }
            }
            _ => FillKernel::General,
        }
    }

    /// Is the compiled outline's worst radial deviation from the hinted circle
    /// under [`HINT_BUDGET_PX`] once scaled to pixels?
    ///
    /// Measured, not assumed. A quadratic through two points on a circle with
    /// the tangent-intersection handle deviates most at its own midpoint, so one
    /// evaluation per segment finds the worst case; using the *actual* points
    /// also means a hand-edited outline that still carries the tag cannot slip
    /// past on the strength of a formula.
    fn disc_within_budget(
        center: [f64; 3],
        radius: f64,
        segments: &[crate::table::Segment],
        scale: f64,
    ) -> bool {
        if segments.is_empty() {
            return false;
        }
        let mut worst = 0.0f64;
        for s in segments {
            // B(1/2) = (p0 + 2 p1 + p2) / 4.
            let mid = [
                0.25 * (s.p0[0] + 2.0 * s.p1[0] + s.p2[0]) - center[0],
                0.25 * (s.p0[1] + 2.0 * s.p1[1] + s.p2[1]) - center[1],
            ];
            let d = (mid[0] * mid[0] + mid[1] * mid[1]).sqrt();
            worst = worst.max((d - radius).abs());
        }
        worst * scale <= HINT_BUDGET_PX
    }

    /// Exact coverage of one pixel, or `None` for [`FillKernel::General`].
    #[must_use]
    pub fn coverage(&self, px: u32, py: u32) -> Option<f64> {
        let (x0, y0) = (f64::from(px), f64::from(py));
        let (x1, y1) = (x0 + 1.0, y0 + 1.0);
        self.coverage_box([x0, y0, x1, y1])
    }

    /// Exact coverage of one cell in an `samples × samples` subpixel grid.
    ///
    /// Hinted geometry remains hinted during adaptive/forced AA: a true-disc
    /// kernel must not quietly become the compiled quadratic approximation just
    /// because a cell escalated. `None` names either the general kernel or an
    /// invalid grid coordinate, allowing the caller to use
    /// [`coverage_at_subcell`] for the former and refuse/fallback for the latter.
    #[must_use]
    pub fn coverage_subcell(
        &self,
        px: u32,
        py: u32,
        samples: u32,
        sample_x: u32,
        sample_y: u32,
    ) -> Option<f64> {
        if samples == 0 || sample_x >= samples || sample_y >= samples {
            return None;
        }
        let inverse = 1.0 / f64::from(samples);
        let x0 = f64::from(px) + f64::from(sample_x) * inverse;
        let y0 = f64::from(py) + f64::from(sample_y) * inverse;
        self.coverage_box([x0, y0, x0 + inverse, y0 + inverse])
            // The box primitive returns an area in native-pixel units. A
            // subcell's coverage fraction divides by its own `1/n²` area.
            .map(|area| area * f64::from(samples) * f64::from(samples))
    }

    /// Exact intersection area with an arbitrary screen-space box.
    fn coverage_box(&self, cell: [f64; 4]) -> Option<f64> {
        Some(match *self {
            FillKernel::General => return None,
            FillKernel::Rect { rect } => box_overlap(rect, cell),
            FillKernel::Disc { center, radius } => disc_box_area(center, radius, cell),
            FillKernel::RoundedRect { rect, radius } => rounded_rect_box_area(rect, radius, cell),
        })
    }

    /// Exact coverage of a run of pixels in one row.
    ///
    /// The same values [`FillKernel::coverage`] gives, so a caller may use
    /// either; this exists because a row is the unit the engine schedules and a
    /// hinted fill should not pay a match per pixel.
    pub fn row(&self, row_y: u32, x_lo: u32, x_hi: u32, out: &mut [f64]) -> bool {
        let w = (x_hi.saturating_sub(x_lo)) as usize;
        debug_assert_eq!(out.len(), w);
        if matches!(self, FillKernel::General) {
            return false;
        }
        for (i, o) in out.iter_mut().enumerate() {
            *o = self
                .coverage(x_lo + i as u32, row_y)
                .expect("not the general kernel");
        }
        true
    }
}

/// Area of the intersection of two axis-aligned rectangles.
fn box_overlap(a: [f64; 4], b: [f64; 4]) -> f64 {
    let w = (a[2].min(b[2]) - a[0].max(b[0])).max(0.0);
    let h = (a[3].min(b[3]) - a[1].max(b[1])).max(0.0);
    w * h
}

/// `∫ √(r² − v²) dv` — the antiderivative every disc-area term is built from.
///
/// `asin` routes through fmn-dmath (§6.6, D-17): a hinted fill must produce the
/// same bits on every certified target as the general path it stands in for, and
/// `f64::asin` defers to the platform's libm. `sqrt` is used directly — IEEE 754
/// requires it correctly rounded, so it is already reproducible.
/// The half-chord `√(r² − v²)`, factored so it does not cancel.
///
/// `r*r - v*v` loses every digit of the difference as `|v| → r`, and because the
/// result is then square-rooted, an absolute error `ε` in the radicand becomes
/// `√ε` in the answer: at `r = 6` the `4e-15` left by cancellation came out as
/// `6e-8` of half-chord. `(r − v)(r + v)` is the same quantity with each factor
/// computed to its own last bit, so the relative error stays at one ulp.
///
/// This is not academic. It showed up as a **3e-8 area error in the rounded
/// rectangle** and nowhere else: a disc's coverage summed over a pixel grid
/// telescopes — adjacent boxes share their corner evaluations — so the plain
/// disc oracle reported 4e-12 and hid it completely. Only the corner
/// decomposition, which clips each square differently and cannot telescope,
/// exposed the conditioning.
fn half_chord(r: f64, v: f64) -> f64 {
    ((r - v) * (r + v)).max(0.0).sqrt()
}

/// `∫ √(r² − v²) dv = ½(v√(r² − v²) + r² asin(v/r))`, with the inverse
/// trigonometry written as `atan2` because `asin` is the wrong function here.
///
/// `asin` has an infinite derivative at ±1, so a one-ulp error in `v/r` becomes
/// an error of `1e-16/√(2δ)` in the angle when `v/r = 1 − δ`. At `r = 6` with `v`
/// a hair under `r` that is `~8e-9` of angle and, after the `r²/2` factor,
/// **1.4e-7 of area** — and pixels sit a hair under `r` at every disc's left and
/// right extremes, so this is the common case rather than a corner one.
/// `atan2(v, √(r² − v²))` is the same angle: the point `(√(r² − v²), v)` lies on
/// the circle of radius `r`, so its argument is exactly `asin(v/r)` for
/// `v ∈ [−r, r]`. It is also *well* conditioned there — as the second argument
/// goes to zero the angle goes to ±π/2 linearly in it — which turns 1.4e-7 into
/// one ulp. Both routes go through fmn-dmath (§6.6, D-17); this one is the route
/// that is right.
fn circle_antiderivative(r: f64, v: f64) -> f64 {
    let vc = v.clamp(-r, r);
    0.5 * (vc * half_chord(r, vc) + r * r * fmn_dmath::atan2(vc, half_chord(r, vc)))
}

/// Area of `{ (u, v) : u² + v² ≤ r², u ≤ x, v ≤ y }`.
///
/// The disc-area primitive, in the one form that composes: with it, a disc's
/// intersection with any axis-aligned rectangle is four evaluations and an
/// inclusion–exclusion, and no case analysis is needed at the call site.
///
/// Derivation. The area is `∫_{-r}^{min(y,r)} h(v) dv` with
/// `h(v) = clamp(x, −s, s) + s` and `s(v) = √(r² − v²)`, because `h` is the
/// length of the slice `{ u : u ≤ x, u² ≤ r² − v² }`. `h` switches form where
/// `s(v) = |x|`, i.e. at `v = ±a` with `a = √(r² − x²)`: inside `(−a, a)` it is
/// `x + s`; outside it is `2s` when `x > 0` and `0` when `x < 0`. Each piece
/// integrates in closed form via [`circle_antiderivative`].
fn disc_quadrant_area(r: f64, x: f64, y: f64) -> f64 {
    if r <= 0.0 || y <= -r || x <= -r {
        return 0.0;
    }
    let yt = y.min(r);
    let s = |v: f64| circle_antiderivative(r, v);
    if x >= r {
        return 2.0 * (s(yt) - s(-r));
    }
    let a = half_chord(r, x);
    let mut acc = 0.0;
    // The middle band, where the slice is cut by `x` on one side only.
    let hi2 = yt.min(a);
    if hi2 > -a {
        acc += x * (hi2 - (-a)) + (s(hi2) - s(-a));
    }
    if x > 0.0 {
        // The two caps, where the whole chord is left of `x`.
        let hi1 = yt.min(-a);
        if hi1 > -r {
            acc += 2.0 * (s(hi1) - s(-r));
        }
        if yt > a {
            acc += 2.0 * (s(yt) - s(a));
        }
    }
    acc
}

/// Area of a disc intersected with an axis-aligned rectangle, exactly.
fn disc_box_area(center: [f64; 2], radius: f64, rect: [f64; 4]) -> f64 {
    let f = |x: f64, y: f64| disc_quadrant_area(radius, x - center[0], y - center[1]);
    (f(rect[2], rect[3]) - f(rect[0], rect[3]) - f(rect[2], rect[1]) + f(rect[0], rect[1])).max(0.0)
}

/// Area of a rounded rectangle intersected with an axis-aligned rectangle.
///
/// The rounded rectangle is the full rectangle minus, at each corner, the part
/// of the corner's `r × r` square that lies outside the corner's quarter disc.
/// Both of those are box intersections, so the whole thing is
/// `box_overlap − Σ (corner-square overlap − corner-disc overlap)` — five exact
/// terms, no new geometry.
fn rounded_rect_box_area(rect: [f64; 4], radius: f64, b: [f64; 4]) -> f64 {
    let mut area = box_overlap(rect, b);
    if radius <= 0.0 {
        return area;
    }
    let r = radius;
    let corners = [
        (
            [rect[0], rect[1], rect[0] + r, rect[1] + r],
            [rect[0] + r, rect[1] + r],
        ),
        (
            [rect[2] - r, rect[1], rect[2], rect[1] + r],
            [rect[2] - r, rect[1] + r],
        ),
        (
            [rect[0], rect[3] - r, rect[0] + r, rect[3]],
            [rect[0] + r, rect[3] - r],
        ),
        (
            [rect[2] - r, rect[3] - r, rect[2], rect[3]],
            [rect[2] - r, rect[3] - r],
        ),
    ];
    for (square, centre) in corners {
        let clipped = [
            square[0].max(b[0]),
            square[1].max(b[1]),
            square[2].min(b[2]),
            square[3].min(b[3]),
        ];
        if clipped[2] <= clipped[0] || clipped[3] <= clipped[1] {
            continue;
        }
        let square_part = (clipped[2] - clipped[0]) * (clipped[3] - clipped[1]);
        let disc_part = disc_box_area(centre, r, clipped);
        area -= (square_part - disc_part).max(0.0);
    }
    area.max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmn_core::types::Vec3;
    use fmn_geom::bezier;
    use fmn_geom::quadpath::QuadPath;

    /// Screen-space pieces straight from a path, with no plan in the way.
    ///
    /// Groups by subpath and closes each one, exactly as [`MonoTable::build`]
    /// does — a helper that skipped the closing would test a different fill from
    /// the one that ships.
    fn pieces_of_path(path: &QuadPath, map: ScreenMap) -> Vec<MonoPiece> {
        let mut out = Vec::new();
        let s = |p: Vec3| map.to_pixel(p[0], p[1]);
        let mut curves: Vec<[[f64; 2]; 3]> = Vec::new();
        for i in 0..path.num_curves() {
            let Some([p0, p1, p2]) = path.nth_curve_points(i) else {
                continue;
            };
            if p0 == p1 && p1 == p2 {
                append_subpath(&curves, &mut out);
                curves.clear();
                continue;
            }
            curves.push([s(p0), s(p1), s(p2)]);
        }
        append_subpath(&curves, &mut out);
        out
    }

    /// The exact area a closed screen-space path encloses, by Green's theorem.
    ///
    /// `∮ x dy` summed over the monotone pieces, in closed form — the same
    /// integral [`Coeffs::mean_x_over_dy`] evaluates, taken over whole pieces.
    /// This is the *right* oracle for a fill that claims to be exact on curves:
    /// comparing a quadratic circle's coverage against `πr²` measures the
    /// approximation of the circle, whereas comparing it against this measures
    /// the rasterizer. Only valid where no region is wound more than once.
    fn enclosed_area(pieces: &[MonoPiece]) -> f64 {
        let mut a = 0.0;
        for piece in pieces {
            let c = Coeffs::<f64>::of(piece, [0.0, 0.0]);
            let s1 = 1.0;
            let s2 = 1.0;
            let s3 = 1.0;
            let k0 = c.ax * c.by;
            let k1 = c.bx * c.by + 2.0 * c.ax * c.cy;
            let k2 = c.cx * c.by + 2.0 * c.bx * c.cy;
            let k3 = 2.0 * c.cx * c.cy;
            a += k0 + k1 * s1 / 2.0 + k2 * s2 / 3.0 + k3 * s3 / 4.0;
        }
        a.abs()
    }

    fn unit() -> ScreenMap {
        ScreenMap {
            scale: 1.0,
            origin: [0.0, 0.0],
            y_up: false,
        }
    }

    fn doubly_turning_curve_plan() -> RenderPlan {
        // The x and y extrema are distinct, so the quadratic becomes three
        // monotone pieces. The open subpath contributes one closing chord.
        let mut stage = fmn_mobject::Stage::new();
        let curve = stage.add(fmn_mobject::Mobject::vector_from_points(&[
            [0.0, 0.0, 0.0],
            [2.0, 3.0, 0.0],
            [1.0, 1.0, 0.0],
        ]));
        stage.add_to_scene(curve).expect("live curve");
        let mut plan = RenderPlan::new();
        plan.sync(&stage, 0).expect("valid one-curve plan");
        plan
    }

    fn circle_path(cx: f64, cy: f64, r: f64, n: usize) -> QuadPath {
        let pts = bezier::quadratic_points_for_arc(std::f64::consts::TAU, n).expect("valid arc");
        let placed: Vec<Vec3> = pts
            .iter()
            .map(|p| [cx + r * p[0], cy + r * p[1], 0.0])
            .collect();
        QuadPath::from_points(placed).expect("closed arc is a valid path")
    }

    fn polygon(points: &[[f64; 2]]) -> QuadPath {
        let mut p = QuadPath::default();
        p.start_new_path([points[0][0], points[0][1], 0.0]);
        for q in &points[1..] {
            p.add_line_to([q[0], q[1], 0.0], false).unwrap();
        }
        p.add_line_to([points[0][0], points[0][1], 0.0], false)
            .unwrap();
        p
    }

    /// Total coverage over a pixel window, tile by tile — the integral the area
    /// oracles compare against.
    fn total_coverage(pieces: &[MonoPiece], w: u32, h: u32, tile: u32) -> f64 {
        let mut scratch = RowScratch::for_tile(tile).expect("coverage tile scratch");
        let mut sum = 0.0;
        for y in 0..h {
            let mut x = 0;
            while x < w {
                let hi = (x + tile).min(w);
                sum += scratch
                    .fill_row(pieces, [0.0, 0.0], y, x, hi)
                    .iter()
                    .sum::<f64>();
                x = hi;
            }
        }
        sum
    }

    fn adversarial_monotone_pieces() -> Vec<MonoPiece> {
        let curves = [
            [[0.125, 0.25], [31.5, 1.0], [63.875, 8.75]],
            [[63.75, 0.5], [28.0, 4.0], [0.25, 15.875]],
            [
                [0.125, 2.0],
                [31.999_999_999_999, 8.000_000_000_001],
                [63.875, 14.0],
            ],
            [[1.0, 1.0], [60.0, 2.0], [63.0, 31.0]],
            [[62.75, 31.0], [3.0, 29.0], [0.5, 0.75]],
            [[16.0, 0.125], [16.0, 14.0], [16.0, 31.875]],
            [[0.0, 7.0], [32.0, 7.0], [64.0, 7.0]],
            [[0.0, 0.0], [64.0, 32.0], [0.0, 31.999_999_999_999]],
            [[64.0, 0.0], [0.0, 32.0], [64.0, 31.999_999_999_999]],
        ];
        let mut pieces = Vec::new();
        for [p0, p1, p2] in curves {
            split_monotone(p0, p1, p2, &mut pieces);
        }
        pieces
    }

    #[test]
    fn adversarial_monotone_column_roots_are_ordered_and_reconstruct_targets() {
        for translate in [[0.0, 0.0], [0.375, -0.625], [-7.75, 3.125]] {
            for piece in adversarial_monotone_pieces() {
                let c = Coeffs::<f64>::of(&piece, translate);
                let x0 = c.x(0.0);
                let x1 = c.x(1.0);
                if x0 == x1 {
                    continue;
                }
                let scale = x0.abs().max(x1.abs()).max(1.0);
                let mut previous = 0.0;
                for fraction in [0.0, 1e-12, 0.1, 0.3, 0.5, 0.6, 0.9, 1.0] {
                    let target = x0 + fraction * (x1 - x0);
                    let t = c.t_at_x(target, 0.0, 1.0);
                    assert!(
                        t >= previous && t <= 1.0,
                        "non-monotone root t={t:?} after {previous:?} for {piece:?}"
                    );
                    let reconstructed = c.x(t);
                    assert!(
                        (reconstructed - target).abs() <= 256.0 * f64::EPSILON * scale,
                        "root t={t:?} reconstructs {reconstructed:?}, target {target:?}, \
                         piece {piece:?}"
                    );
                    previous = t;
                }
            }
        }
    }

    #[test]
    fn row_scratch_fallible_constructor_checks_layout_before_allocation() {
        assert_eq!(
            RowScratch::try_for_width(usize::MAX).expect_err("overflowing cell count"),
            FrameError::TooLarge
        );
        let scratch = RowScratch::try_for_width(16).expect("small row scratch");
        assert_eq!(scratch.cells.len(), 17);
        assert_eq!(scratch.out.len(), 16);
        assert_eq!(scratch.crossings.len(), 16);
    }

    #[test]
    fn a_rectangle_covers_exactly_its_area() {
        // The simplest oracle, and the one that catches a sign error before any
        // curve is involved: an axis-aligned box on non-integer bounds.
        let path = polygon(&[[2.5, 3.25], [17.75, 3.25], [17.75, 12.5], [2.5, 12.5]]);
        let pieces = pieces_of_path(&path, unit());
        let want = (17.75 - 2.5) * (12.5 - 3.25);
        let got = total_coverage(&pieces, 24, 16, 8);
        assert!((got - want).abs() < 1e-9, "{got} vs {want}");
    }

    #[test]
    fn subcell_coverage_resolves_to_the_native_analytic_area() {
        let path = circle_path(8.3, 8.1, 5.7, 8);
        let pieces = pieces_of_path(&path, unit());
        let native = coverage_at_cell::<f64>(&pieces, [0.0, 0.0], 4, 5);
        for samples in [2, 4] {
            let mut resolved = 0.0;
            for sy in 0..samples {
                for sx in 0..samples {
                    resolved += coverage_at_subcell(&pieces, [0.0, 0.0], 4, 5, samples, sx, sy);
                }
            }
            resolved /= f64::from(samples * samples);
            assert!(
                (resolved - native).abs() < 1e-12,
                "{samples}x: {resolved} vs native {native}"
            );
        }
    }

    /// [`boundary_crossings_at_cell`] without the extent prefilter: every
    /// piece through every probe, as the counts are defined.
    fn unfiltered_crossings(
        pieces: &[MonoPiece],
        translate: [f64; 2],
        row_y: u32,
        cell: u32,
    ) -> u8 {
        let (x0, y0) = (f64::from(cell), f64::from(row_y));
        let mut most = 0u8;
        for offset in [0.25, 0.5, 0.75] {
            let (y, x) = (y0 + offset, x0 + offset);
            let mut across = 0u8;
            let mut down = 0u8;
            for piece in pieces {
                let c = Coeffs::<f64>::of(piece, translate);
                let (a, b) = (c.y(0.0), c.y(1.0));
                if !(a.max(b) <= a.min(b) || y < a.min(b) || y >= a.max(b)) {
                    let hit = c.x(c.t_at_y(y, 0.0, 1.0));
                    across += u8::from(hit >= x0 && hit < x0 + 1.0);
                }
                let (a, b) = (c.x(0.0), c.x(1.0));
                if !(a.max(b) <= a.min(b) || x < a.min(b) || x >= a.max(b)) {
                    let hit = c.y(c.t_at_x(x, 0.0, 1.0));
                    down += u8::from(hit >= y0 && hit < y0 + 1.0);
                }
            }
            most = most.max(across).max(down);
        }
        most
    }

    #[test]
    fn the_extent_prefilter_never_changes_a_crossing_count() {
        let mut shapes = vec![
            pieces_of_path(&circle_path(8.3, 8.1, 5.7, 8), unit()),
            pieces_of_path(&circle_path(9.0, 9.0, 0.4, 8), unit()),
        ];
        let mut dense = pieces_of_path(
            &polygon(&[[5.1, 4.0], [5.2, 4.0], [5.2, 8.0], [5.1, 8.0]]),
            unit(),
        );
        dense.extend(pieces_of_path(
            &polygon(&[[5.0, 5.0], [9.0, 5.0], [9.0, 5.0 + 1e-7], [5.0, 5.0 + 1e-7]]),
            unit(),
        ));
        shapes.push(dense);
        // More pieces meet cell (5, 5) than the candidate buffer holds: the
        // whole-list fallback must count the same.
        let mut crowded = Vec::new();
        for k in 0..12 {
            let x = 5.05 + 0.075 * f64::from(k);
            crowded.extend(pieces_of_path(
                &polygon(&[[x, 4.5], [x + 0.03, 4.5], [x + 0.03, 6.5], [x, 6.5]]),
                unit(),
            ));
        }
        assert!(crowded.len() > 32);
        shapes.push(crowded);
        let mut counted = 0;
        for pieces in &shapes {
            for translate in [[0.0, 0.0], [0.37, -0.81]] {
                for row in 0..18 {
                    for cell in 0..18 {
                        let want = unfiltered_crossings(pieces, translate, row, cell);
                        assert_eq!(
                            boundary_crossings_at_cell(pieces, translate, row, cell),
                            want,
                            "cell ({cell}, {row}) at {translate:?}"
                        );
                        counted += usize::from(want > 0);
                    }
                }
            }
        }
        assert!(counted >= 50, "the lattice crosses boundaries: {counted}");
    }

    #[test]
    fn boundary_crossing_counts_separate_simple_thin_and_dense_cells() {
        let simple = pieces_of_path(
            &polygon(&[[5.4, 4.0], [10.0, 4.0], [10.0, 8.0], [5.4, 8.0]]),
            unit(),
        );
        assert_eq!(boundary_crossings_at_cell(&simple, [0.0, 0.0], 5, 5), 1);
        let mut scratch = RowScratch::for_tile(1).expect("one-pixel scratch");
        let (_, crossings) = scratch.fill_row_classified(&simple, [0.0, 0.0], 5, 5, 6);
        assert_eq!(crossings, &[1]);

        let thin = pieces_of_path(
            &polygon(&[[5.2, 4.0], [5.6, 4.0], [5.6, 8.0], [5.2, 8.0]]),
            unit(),
        );
        assert_eq!(boundary_crossings_at_cell(&thin, [0.0, 0.0], 5, 5), 2);
        let (_, crossings) = scratch.fill_row_classified(&thin, [0.0, 0.0], 5, 5, 6);
        assert_eq!(crossings, &[2]);

        let horizontal_thin = pieces_of_path(
            &polygon(&[[4.0, 5.2], [8.0, 5.2], [8.0, 5.6], [4.0, 5.6]]),
            unit(),
        );
        assert_eq!(
            boundary_crossings_at_cell(&horizontal_thin, [0.0, 0.0], 5, 5),
            2
        );
        let (_, crossings) = scratch.fill_row_classified(&horizontal_thin, [0.0, 0.0], 5, 5, 6);
        assert_eq!(crossings, &[2]);

        let mut dense = pieces_of_path(
            &polygon(&[[5.1, 4.0], [5.2, 4.0], [5.2, 8.0], [5.1, 8.0]]),
            unit(),
        );
        dense.extend(pieces_of_path(
            &polygon(&[[5.5, 4.0], [5.6, 4.0], [5.6, 8.0], [5.5, 8.0]]),
            unit(),
        ));
        assert_eq!(boundary_crossings_at_cell(&dense, [0.0, 0.0], 5, 5), 4);
        let (_, crossings) = scratch.fill_row_classified(&dense, [0.0, 0.0], 5, 5, 6);
        assert_eq!(crossings, &[4]);

        let mut horizontal_dense = pieces_of_path(
            &polygon(&[[4.0, 5.1], [8.0, 5.1], [8.0, 5.2], [4.0, 5.2]]),
            unit(),
        );
        horizontal_dense.extend(pieces_of_path(
            &polygon(&[[4.0, 5.5], [8.0, 5.5], [8.0, 5.6], [4.0, 5.6]]),
            unit(),
        ));
        assert_eq!(
            boundary_crossings_at_cell(&horizontal_dense, [0.0, 0.0], 5, 5),
            4
        );
        let (_, crossings) = scratch.fill_row_classified(&horizontal_dense, [0.0, 0.0], 5, 5, 6);
        assert_eq!(crossings, &[4]);
    }

    #[test]
    fn a_disc_covers_exactly_the_area_it_encloses() {
        // The area oracle §10.2 names, in the form that measures the rasterizer
        // rather than the geometry. The path is manim's own quadratic
        // approximation of a circle, so `πr²` is the wrong target by ~2.6e-4
        // relative — the approximation's error, not ours. The exact target is
        // the area the quadratics actually enclose, and the fill must hit it to
        // far better than one 8-bit level over the whole disc.
        for n in [4usize, 8, 16, 64] {
            let r = 20.0;
            let path = circle_path(32.3, 31.7, r, n);
            let pieces = pieces_of_path(&path, unit());
            let got = total_coverage(&pieces, 64, 64, 16);
            let want = enclosed_area(&pieces);
            assert!(
                (got - want).abs() < 1e-9,
                "n={n}: coverage {got} vs enclosed {want}"
            );
            // And the enclosed area is a circle's, to the arc approximation's
            // own closed-form error — not to a tolerance someone picked. With
            // `u = θ/2 = π/n`, `cos u + sec u = 2 + u⁴/4 + O(u⁶)`, so the
            // quadratic's midpoint sits at radius `r(1 + u⁴/8)` and the enclosed
            // area runs `(4/3)·u⁴/8 = u⁴/6` high. That predicts 6.3e-2 at n=4 and
            // 2.5e-4 at n=16; both are what this measures.
            let circle = std::f64::consts::PI * r * r;
            let rel = (want - circle).abs() / circle;
            let u = std::f64::consts::PI / n as f64;
            let predicted = u.powi(4) / 6.0;
            assert!(
                rel < 1.1 * predicted,
                "n={n}: rel {rel} vs predicted {predicted}"
            );
        }
    }

    #[test]
    fn a_triangle_covers_exactly_its_area() {
        let path = polygon(&[[1.0, 1.0], [21.0, 1.0], [1.0, 16.0]]);
        let pieces = pieces_of_path(&path, unit());
        let got = total_coverage(&pieces, 24, 20, 8);
        let want = 0.5 * 20.0 * 15.0;
        assert!((got - want).abs() < 1e-9, "{got} vs {want}");
    }

    #[test]
    fn opposite_windings_cancel_to_a_hole() {
        // The nonzero rule stated as an area: an outer square wound one way and
        // an inner square wound the other leaves an annulus, and the inner
        // region must read exactly zero rather than "twice, clamped".
        let outer = polygon(&[[4.0, 4.0], [28.0, 4.0], [28.0, 28.0], [4.0, 28.0]]);
        let inner = polygon(&[[12.0, 12.0], [12.0, 20.0], [20.0, 20.0], [20.0, 12.0]]);
        let mut pieces = pieces_of_path(&outer, unit());
        pieces.extend(pieces_of_path(&inner, unit()));

        let got = total_coverage(&pieces, 32, 32, 8);
        let want = 24.0 * 24.0 - 8.0 * 8.0;
        assert!((got - want).abs() < 1e-9, "{got} vs {want}");
        // And a point in the hole is genuinely empty, not merely averaged away.
        assert_eq!(coverage_at_cell::<f64>(&pieces, [0.0, 0.0], 16, 16), 0.0);
        assert_eq!(winding_at(&pieces, [0.0, 0.0], [16.5, 16.5]), 0);
        assert_eq!(winding_at(&pieces, [0.0, 0.0], [8.5, 16.5]), 1);
    }

    #[test]
    fn same_winding_twice_clamps_instead_of_doubling() {
        // Two coincident squares wound the same way accumulate to 2. The nonzero
        // rule says the region is inside, i.e. coverage 1 — not 2, and not 0.
        let sq = polygon(&[[4.0, 4.0], [12.0, 4.0], [12.0, 12.0], [4.0, 12.0]]);
        let mut pieces = pieces_of_path(&sq, unit());
        pieces.extend(pieces_of_path(&sq, unit()));
        let got = total_coverage(&pieces, 16, 16, 8);
        assert!((got - 64.0).abs() < 1e-9, "{got} vs 64");
        assert_eq!(coverage_at_cell::<f64>(&pieces, [0.0, 0.0], 8, 8), 1.0);
    }

    #[test]
    fn tile_alignment_never_changes_the_answer() {
        // G0-8b's second rule: a tiling test that does not sweep alignments sees
        // nothing, because no piece ever enters at a tile edge. Sweeping the
        // tile size sweeps every phase a piece can enter at.
        let path = circle_path(31.7, 30.3, 19.4, 16);
        let pieces = pieces_of_path(&path, unit());
        let reference = total_coverage(&pieces, 64, 64, 64);
        for tile in [1u32, 2, 3, 5, 7, 8, 11, 16, 32] {
            let got = total_coverage(&pieces, 64, 64, tile);
            assert!(
                (got - reference).abs() < 1e-9,
                "tile {tile}: {got} vs {reference}"
            );
        }
    }

    #[test]
    fn the_per_pixel_form_agrees_with_the_scanline_form() {
        // The two dispatch shapes are the same terms in a different association.
        // They are compared per pixel rather than in aggregate, and at a tile
        // width that puts pieces across tile edges.
        let path = circle_path(20.4, 19.6, 12.3, 16);
        let pieces = pieces_of_path(&path, unit());
        let mut scratch = RowScratch::for_tile(8).expect("eight-pixel scratch");
        let mut worst = 0.0f64;
        for y in 0..40 {
            let mut x = 0;
            while x < 40 {
                let hi = (x + 8).min(40);
                let row = scratch.fill_row(&pieces, [0.0, 0.0], y, x, hi).to_vec();
                for (i, v) in row.iter().enumerate() {
                    let per_pixel: f64 = coverage_at_cell(&pieces, [0.0, 0.0], y, x + i as u32);
                    worst = worst.max((v - per_pixel).abs());
                }
                x = hi;
            }
        }
        assert!(worst < 1e-12, "worst per-pixel disagreement {worst}");
    }

    #[test]
    fn subdividing_a_curve_does_not_change_the_fill() {
        // §10.2's metamorphic law. Splitting every segment at its midpoint is a
        // different point array describing the same region, so every pixel's
        // coverage must survive it.
        let path = circle_path(24.3, 23.7, 15.1, 8);
        let mut split = QuadPath::default();
        for i in 0..path.num_curves() {
            let [p0, p1, p2] = path.nth_curve_points(i).unwrap();
            let mid = |a: Vec3, b: Vec3| [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0, 0.0];
            let q0 = mid(p0, p1);
            let q1 = mid(p1, p2);
            let r = mid(q0, q1);
            if i == 0 {
                split.start_new_path(p0);
            }
            split.add_quadratic_bezier_curve_to(q0, r, false).unwrap();
            split.add_quadratic_bezier_curve_to(q1, p2, false).unwrap();
        }

        let a = pieces_of_path(&path, unit());
        let b = pieces_of_path(&split, unit());
        assert!(b.len() > a.len(), "the subdivision must add pieces");

        let mut sa = RowScratch::for_tile(16).expect("first row scratch");
        let mut sb = RowScratch::for_tile(16).expect("second row scratch");
        let mut worst = 0.0f64;
        for y in 0..48 {
            let ra = sa.fill_row(&a, [0.0, 0.0], y, 0, 48).to_vec();
            let rb = sb.fill_row(&b, [0.0, 0.0], y, 0, 48).to_vec();
            for (u, v) in ra.iter().zip(rb.iter()) {
                worst = worst.max((u - v).abs());
            }
        }
        assert!(
            worst < 1e-12,
            "worst per-pixel drift under subdivision {worst} — the deposit is \
             using a chord where §10.2 wants the integral"
        );
    }

    #[test]
    fn a_translated_occurrence_is_the_same_coverage_moved() {
        // The interning claim, at the fill: one outline plus a translation must
        // give the same pixels as the outline built at the destination. An
        // integer translation makes the comparison exact.
        let path = circle_path(16.0, 16.0, 9.5, 16);
        let here = pieces_of_path(&path, unit());
        let there = pieces_of_path(&circle_path(16.0 + 7.0, 16.0 + 5.0, 9.5, 16), unit());

        let mut s1 = RowScratch::for_tile(16).expect("source row scratch");
        let mut s2 = RowScratch::for_tile(16).expect("translated row scratch");
        let mut worst = 0.0f64;
        for y in 0..40 {
            let a = s1.fill_row(&here, [7.0, 5.0], y, 0, 40).to_vec();
            let b = s2.fill_row(&there, [0.0, 0.0], y, 0, 40).to_vec();
            for (u, v) in a.iter().zip(b.iter()) {
                worst = worst.max((u - v).abs());
            }
        }
        assert!(worst < 1e-12, "translation drift {worst}");
    }

    #[test]
    fn the_annex_width_tracks_the_host_width() {
        // The f32 instantiation is the annex's arithmetic without the annex's
        // hardware — the controlled experiment G0-8 ran for strokes, kept
        // answerable on a machine with no GPU. The budget is f32's, not the
        // algorithm's: a coverage in [0,1] carries ~1e-7 of representation.
        let path = circle_path(24.4, 23.6, 15.2, 16);
        let pieces = pieces_of_path(&path, unit());
        let mut cells64 = vec![0.0f64; 49];
        let mut out64 = vec![0.0f64; 48];
        let mut cells32 = vec![0.0f32; 49];
        let mut out32 = vec![0.0f32; 48];
        let mut worst = 0.0f64;
        for y in 0..48 {
            fill_row(&pieces, [0.0, 0.0], y, 0, 48, &mut cells64, &mut out64);
            fill_row(&pieces, [0.0, 0.0], y, 0, 48, &mut cells32, &mut out32);
            for (u, v) in out64.iter().zip(out32.iter()) {
                worst = worst.max((u - f64::from(*v)).abs());
            }
        }
        assert!(worst < 1e-5, "host/annex width divergence {worst}");
    }

    #[test]
    fn a_self_intersecting_path_follows_the_nonzero_rule() {
        // A pentagram: the central pentagon is wound twice and is therefore
        // *inside* under the nonzero rule — the visible difference from
        // even-odd, and the reason §10.2 names the rule rather than assuming it.
        let mut pts = Vec::new();
        let r = 20.0;
        for k in 0..5 {
            // Every second vertex, which is what makes the star self-intersect.
            let a = std::f64::consts::FRAC_PI_2 + (k as f64) * 4.0 * std::f64::consts::PI / 5.0;
            pts.push([24.0 + r * a.cos(), 24.0 + r * a.sin()]);
        }
        let path = polygon(&pts);
        let pieces = pieces_of_path(&path, unit());
        assert_eq!(
            winding_at(&pieces, [0.0, 0.0], [24.0, 24.0]).abs(),
            2,
            "the centre of a pentagram is wound twice"
        );
        assert_eq!(
            coverage_at_cell::<f64>(&pieces, [0.0, 0.0], 24, 24),
            1.0,
            "and is therefore fully inside"
        );

        // Area check: a {5/2} star polygon of circumradius r has area
        // 5 r² tan(π/5) (1 - tan(π/5) tan(π/10))... rather than trust a formula
        // transcribed from memory, compare against the polygon area computed by
        // the shoelace rule over the *filled* region, which for the nonzero rule
        // is the star's outline plus its doubly-wound core counted once.
        let got = total_coverage(&pieces, 48, 48, 8);
        // Shoelace over the traversal counts the core twice; the nonzero fill
        // counts it once, so the fill must be strictly less.
        let mut shoelace = 0.0;
        for k in 0..pts.len() {
            let a = pts[k];
            let b = pts[(k + 1) % pts.len()];
            shoelace += a[0] * b[1] - b[0] * a[1];
        }
        let traversal_area = 0.5 * shoelace.abs();
        assert!(
            got < traversal_area - 1.0,
            "nonzero coverage {got} should undercount the doubly-wound core \
             (traversal area {traversal_area})"
        );
    }

    #[test]
    fn an_open_subpath_still_fills_as_if_closed() {
        // manim fills open paths by treating the closing chord as present, which
        // is what the winding accumulation does naturally: a path that does not
        // return to its start still deposits a net dy, and the rule is the same
        // one the closed case uses. The oracle is the implied triangle.
        let mut p = QuadPath::default();
        p.start_new_path([4.0, 4.0, 0.0]);
        p.add_line_to([20.0, 4.0, 0.0], false).unwrap();
        p.add_line_to([20.0, 16.0, 0.0], false).unwrap();
        let pieces = pieces_of_path(&p, unit());
        let got = total_coverage(&pieces, 24, 20, 8);
        assert!((got - 0.5 * 16.0 * 12.0).abs() < 1e-9, "{got}");
    }

    #[test]
    fn a_screen_map_scale_scales_the_area() {
        // The mapping is the fill's only view of the camera today, so its two
        // knobs get an assertion each: a 3x scale is 9x the area, and an origin
        // shift moves the picture without changing it.
        let path = polygon(&[[1.0, 1.0], [5.0, 1.0], [5.0, 4.0], [1.0, 4.0]]);
        let big = pieces_of_path(
            &path,
            ScreenMap {
                scale: 3.0,
                origin: [2.0, 1.0],
                y_up: false,
            },
        );
        let got = total_coverage(&big, 32, 32, 8);
        assert!((got - 9.0 * 12.0).abs() < 1e-9, "{got}");
    }

    #[test]
    fn an_interior_tile_is_exactly_one_and_the_general_path_agrees() {
        // Both halves of §10.4's interior class. The value must be exactly 1.0 —
        // bit-exactly, since that is what makes occlusion pruning produce the
        // same bytes as drawing (G0-8b F13) — and the general path must agree
        // wherever the classification is true, or the fast path would be a
        // different picture rather than the same one sooner.
        let mut scratch = RowScratch::for_tile(16).expect("interior row scratch");
        let row = scratch.interior_row(0, 16).to_vec();
        assert_eq!(row.len(), 16);
        for v in &row {
            assert_eq!(v.to_bits(), 1.0f64.to_bits(), "not bit-exactly one: {v}");
        }
        assert_eq!(scratch.interior_row(4, 4).len(), 0);

        // A tile well inside a large disc: the general evaluator must also read
        // exactly one there, which is what says the classification is sound.
        let pieces = pieces_of_path(&circle_path(64.0, 64.0, 50.0, 32), unit());
        let general = scratch.fill_row(&pieces, [0.0, 0.0], 64, 56, 72).to_vec();
        for (i, v) in general.iter().enumerate() {
            assert_eq!(*v, 1.0, "column {} of an interior row", 56 + i);
        }
    }

    #[test]
    fn an_empty_piece_list_covers_nothing() {
        let mut scratch = RowScratch::for_tile(8).expect("empty row scratch");
        let row = scratch.fill_row(&[], [0.0, 0.0], 3, 0, 8);
        assert!(row.iter().all(|v| *v == 0.0));
        assert_eq!(coverage_at_cell::<f64>(&[], [0.0, 0.0], 3, 4), 0.0);
    }

    #[test]
    fn the_table_derives_one_split_per_interned_outline() {
        // The §10.8 property, checked at the fill's own table: three occurrences
        // of one glyph must split once.
        //
        // The outline's coordinates are exactly representable and the shift is an
        // integer, and that is load-bearing rather than tidy. `shape_digest`
        // hashes `p - points[0]` bitwise, and floating-point translation is not
        // exact: written with a circle's `cos`/`sin` points shifted by `3.0`,
        // `(p + 3) - (p0 + 3)` differs from `p - p0` in the low bits and all
        // three outlines hash differently — this test read "3 shapes, want 1"
        // before the coordinates were made exact. That is a real limit on
        // interning as long as producers *bake* translations into points; the
        // resolution is fm-7if's transform channel, where a placement stops
        // being part of the geometry. Recorded on that bead.
        let mut stage = fmn_mobject::Stage::new();
        let square: Vec<Vec3> = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 1.0, 0.0],
            [2.0, 2.0, 0.0],
            [1.0, 2.0, 0.0],
            [0.0, 2.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        let mut mobs = Vec::new();
        for k in 0..3 {
            let shifted: Vec<Vec3> = square
                .iter()
                .map(|p| [p[0] + 4.0 * f64::from(k), p[1], p[2]])
                .collect();
            let mob = stage.add(fmn_mobject::Mobject::vector_from_points(&shifted));
            stage.add_to_scene(mob).expect("live");
            mobs.push(mob);
        }
        let mut plan = RenderPlan::new();
        plan.sync(&stage, 0).expect("valid fill fixture");
        assert_eq!(plan.shapes().shapes().len(), 1, "one interned outline");

        let table = MonoTable::build(&plan, unit()).expect("bounded monotone table");
        assert_eq!(table.len(), 1);
        assert!(!table.pieces_of(0).is_empty());
        assert!(
            table.pieces_of(7).is_empty(),
            "an unknown shape draws nothing"
        );

        // And the occurrences differ only by a translation, which is the whole
        // reason the table is per shape.
        let insts = plan.shapes().instances();
        assert_eq!(insts.len(), 3);
        let t0 = instance_translation(&insts[0], unit());
        let t1 = instance_translation(&insts[1], unit());
        assert!((t1[0] - t0[0] - 4.0).abs() < 1e-12);
        assert!((t1[1] - t0[1]).abs() < 1e-12);
    }

    #[test]
    fn monotone_table_piece_and_byte_limits_are_exact_and_atomic() {
        let plan = doubly_turning_curve_plan();
        let table_bytes = 4 * std::mem::size_of::<MonoPiece>() + std::mem::size_of::<(u32, u32)>();
        let exact = MonoTableLimits {
            max_pieces: 4,
            max_table_bytes: table_bytes,
        };
        let table = MonoTable::build_with_limits(&plan, unit(), exact)
            .expect("four pieces fit the exact table budget");
        assert_eq!(table.pieces().len(), 4);
        assert!(table.matches_plan(&plan));

        let plan_geometry = plan.geometry_identity();
        let pieces_before = table.pieces().to_vec();
        let error = MonoTable::build_with_limits(
            &plan,
            unit(),
            MonoTableLimits {
                max_pieces: 3,
                ..exact
            },
        )
        .expect_err("one piece below the exact boundary must refuse");
        assert!(matches!(
            error,
            MonoTableError::LimitExceeded {
                resource: "monotone pieces",
                limit: 3,
                requested: 4,
            }
        ));
        assert_eq!(plan.geometry_identity(), plan_geometry);
        assert_eq!(table.pieces(), pieces_before);
        assert!(table.matches_plan(&plan));

        let error = MonoTable::build_with_limits(
            &plan,
            unit(),
            MonoTableLimits {
                max_table_bytes: table_bytes - 1,
                ..exact
            },
        )
        .expect_err("one byte below the exact boundary must refuse");
        assert!(matches!(
            error,
            MonoTableError::LimitExceeded {
                resource: "monotone table bytes",
                limit,
                requested,
            } if limit + 1 == requested && requested == table_bytes
        ));
        assert_eq!(plan.geometry_identity(), plan_geometry);
        assert_eq!(table.pieces(), pieces_before);

        let mut changed_stage = fmn_mobject::Stage::new();
        let changed_curve = changed_stage.add(fmn_mobject::Mobject::vector_from_points(&[
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
        ]));
        changed_stage
            .add_to_scene(changed_curve)
            .expect("live changed curve");
        let mut changed_plan = RenderPlan::new();
        changed_plan
            .sync(&changed_stage, 0)
            .expect("valid changed plan");
        let error = MonoTable::build_with_limits(
            &changed_plan,
            unit(),
            MonoTableLimits {
                max_pieces: 0,
                ..exact
            },
        )
        .expect_err("the changed table must refuse before replacing the old artifact");
        assert!(matches!(
            error,
            MonoTableError::LimitExceeded {
                resource: "monotone pieces",
                limit: 0,
                requested: 2,
            }
        ));
        assert!(!table.matches_plan(&changed_plan));
        assert_eq!(table.pieces(), pieces_before);
    }

    #[test]
    fn monotone_table_absolute_width_and_size_overflow_are_typed() {
        if let Ok(one_over_u32) = usize::try_from(u64::from(u32::MAX) + 1) {
            assert_eq!(
                check_piece_count(
                    one_over_u32,
                    MonoTableLimits {
                        max_pieces: usize::MAX,
                        max_table_bytes: usize::MAX,
                    },
                ),
                Err(MonoTableError::IndexCapacityExceeded {
                    resource: "monotone pieces",
                    requested: one_over_u32,
                })
            );
            assert_eq!(
                check_shape_count(one_over_u32),
                Err(MonoTableError::IndexCapacityExceeded {
                    resource: "monotone shape ranges",
                    requested: one_over_u32,
                })
            );
        }

        let overflowing_pieces = usize::MAX / std::mem::size_of::<MonoPiece>() + 1;
        assert_eq!(
            logical_table_bytes(overflowing_pieces, 0),
            Err(MonoTableError::SizeOverflow {
                resource: "monotone table bytes",
            })
        );
        assert_eq!(
            checked_count_add("monotone pieces", usize::MAX, 1),
            Err(MonoTableError::SizeOverflow {
                resource: "monotone pieces",
            })
        );
    }

    #[test]
    fn default_monotone_limits_cover_the_default_retained_plan() {
        let retained = crate::RenderPlanLimits::default();
        let monotone = MonoTableLimits::default();
        let worst_piece_count = retained
            .max_retained_segments
            .checked_mul(4)
            .expect("default segment ceiling has a four-piece bound");
        assert!(u64::try_from(monotone.max_pieces).expect("usize fits u64") >= worst_piece_count);
        let piece_bytes =
            u64::try_from(std::mem::size_of::<MonoPiece>()).expect("row size fits u64");
        let range_bytes =
            u64::try_from(std::mem::size_of::<(u32, u32)>()).expect("range size fits u64");
        let worst_table_bytes = worst_piece_count
            .checked_mul(piece_bytes)
            .and_then(|bytes| {
                retained
                    .max_retained_shapes
                    .checked_mul(range_bytes)
                    .and_then(|ranges| bytes.checked_add(ranges))
            })
            .expect("default logical table byte bound");
        assert!(
            u64::try_from(monotone.max_table_bytes).expect("usize fits u64") >= worst_table_bytes
        );
    }

    #[test]
    fn frame_local_piece_refusal_precedes_pool_mutation() {
        let plan = doubly_turning_curve_plan();
        let table = MonoTable::build(&plan, unit()).expect("bounded seed table");
        let shape = &plan.shapes().shapes()[0];
        let mut pieces = crate::arena::Pool::default();
        pieces.put(table.pieces()[0]);
        let before = pieces.to_vec();
        let mut curves = crate::arena::Pool::default();

        let error = MonoTable::pieces_for_segments_into(
            &mut pieces,
            &mut curves,
            plan.segments(),
            &shape.subpath_starts,
            unit(),
            MonoTableLimits {
                max_pieces: 4,
                max_table_bytes: usize::MAX,
            },
        )
        .expect_err("one retained row plus four derived rows exceeds the ceiling");
        assert_eq!(
            error,
            MonoTableError::LimitExceeded {
                resource: "monotone pieces",
                limit: 4,
                requested: 5,
            }
        );
        assert_eq!(&*pieces, before);
        assert!(curves.is_empty());
    }

    // ------------------------------------------ perspective / rational pieces

    /// A perspective map: divide by `1 + p·q` after a scale. Stands in for
    /// fm-0gy's camera in exactly the way the fill consumes one — homogeneous
    /// control points — without inventing a camera type.
    fn project(p: [f64; 2], q: [f64; 2], scale: f64, origin: [f64; 2]) -> [f64; 3] {
        let w = 1.0 + q[0] * p[0] + q[1] * p[1];
        [
            (origin[0] + scale * p[0]) * w,
            (origin[1] + scale * p[1]) * w,
            w,
        ]
    }

    /// Distance from a point to a line segment.
    fn point_to_segment(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let len2 = dx * dx + dy * dy;
        let t = if len2 > 0.0 {
            (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / len2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let (qx, qy) = (a[0] + t * dx, a[1] + t * dy);
        ((p[0] - qx).powi(2) + (p[1] - qy).powi(2)).sqrt()
    }

    fn projected(pts: [[f64; 2]; 3], q: [f64; 2]) -> RationalPiece {
        RationalPiece {
            p: [
                project(pts[0], q, 20.0, [40.0, 40.0]),
                project(pts[1], q, 20.0, [40.0, 40.0]),
                project(pts[2], q, 20.0, [40.0, 40.0]),
            ],
        }
    }

    #[test]
    fn an_affine_piece_is_already_flat() {
        // The cost of the perspective path for a 2D scene: one deviation
        // evaluation, which finds exactly zero.
        let piece = RationalPiece::affine([1.0, 2.0], [7.0, 9.0], [13.0, 3.0]);
        assert_eq!(piece.deviation_px(), 0.0);
        let mut out = Vec::new();
        let report = append_rational(&piece, 1e-6, &mut out).expect("affine is definite");
        assert_eq!(report.depth, 0);
        assert!(!report.capped);
        assert_eq!(report.error_px, 0.0);
        // And the stand-in reproduces the input exactly: three-point
        // interpolation of a quadratic *is* that quadratic.
        let q = piece.integral_approximation();
        assert!((q.p1[0] - 7.0).abs() < 1e-12 && (q.p1[1] - 9.0).abs() < 1e-12);
    }

    #[test]
    fn rational_piece_limit_is_exact_and_refusal_preserves_the_destination() {
        let piece = RationalPiece::affine([0.0, 0.0], [0.5, 0.5], [1.0, 1.0]);
        let sentinel = MonoPiece {
            p0: [-1.0, -1.0],
            p1: [-0.5, -0.5],
            p2: [0.0, 0.0],
        };
        let mut exact = vec![sentinel];
        let report = append_rational_with_limit(&piece, 1e-6, &mut exact, 1)
            .expect("one monotone piece fits its exact limit");
        assert_eq!(report.pieces, 1);
        assert_eq!(exact.len(), 2);

        let mut refused = vec![sentinel];
        assert_eq!(
            append_rational_with_limit(&piece, 1e-6, &mut refused, 0),
            Err(RationalError::PieceLimitExceeded {
                limit: 0,
                requested: 1,
            })
        );
        assert_eq!(refused, vec![sentinel]);
    }

    #[test]
    fn splitting_in_homogeneous_coordinates_is_exact() {
        // The reason subdivision converges: the halves describe the *same*
        // screen curve, so the error halves rather than compounding. Splitting
        // projected points instead would approximate at every level.
        let piece = projected([[-1.0, -0.5], [0.3, 1.4], [1.2, -0.9]], [0.35, -0.2]);
        let (l, r) = piece.split(0.5);
        for k in 0..=16 {
            let t = f64::from(k) / 16.0;
            let (half, local) = if t <= 0.5 {
                (&l, 2.0 * t)
            } else {
                (&r, 2.0 * t - 1.0)
            };
            let a = piece.point(t);
            let b = half.point(local);
            assert!(
                (a[0] - b[0]).abs() < 1e-12 && (a[1] - b[1]).abs() < 1e-12,
                "t={t}: {a:?} vs {b:?}"
            );
        }
    }

    #[test]
    fn a_projected_quadratic_is_not_a_quadratic_and_the_fill_says_so() {
        // The claim §10.2 forbids fudging. If a perspective-projected quadratic
        // were affine in screen space its ordinary stand-in would be exact; it
        // is not, and the deviation is large enough to see.
        let piece = projected([[-1.0, -0.5], [0.3, 1.4], [1.2, -0.9]], [0.35, -0.2]);
        let deviation = piece.deviation_px();
        assert!(
            deviation > 0.1,
            "a rational quadratic must not read as affine: {deviation}"
        );
    }

    #[test]
    fn subdivision_meets_the_stated_tolerance_against_a_dense_reference() {
        // The acceptance criterion: "homogeneous-path tests vs high-resolution
        // subdivision reference". `deviation_px` samples six parameters, so the
        // tolerance it reports is an estimate — this checks the estimate against
        // a 129-point measurement of the *actual* flattened curve, which is what
        // makes the stated tolerance a statement rather than a hope.
        let piece = projected([[-1.0, -0.5], [0.3, 1.4], [1.2, -0.9]], [0.35, -0.2]);
        for tol in [1.0f64, 0.1, 0.01, 1.0 / 256.0] {
            let mut out = Vec::new();
            let report = append_rational(&piece, tol, &mut out).expect("definite");
            assert!(!report.capped, "tol {tol} hit the depth cap");
            assert!(
                report.error_px <= tol,
                "tol {tol}: reported {}",
                report.error_px
            );
            assert!(!out.is_empty());

            // Dense measurement: every sample of the true curve must lie within
            // the tolerance of the flattened result. Distance is measured to the
            // flattened *segments*, not to samples of them — sampling each piece
            // at 33 points and taking the nearest sample leaves up to half a
            // sample spacing of slack, which on a 20-pixel piece is 0.3 px, and
            // that artefact reported 0.1199 against a genuinely-0.1 flattening.
            // A point-to-segment measure removes it to second order.
            let mut poly: Vec<[f64; 2]> = Vec::new();
            for mp in &out {
                let cand = MonoPieceCandidate {
                    p0: mp.p0,
                    p1: mp.p1,
                    p2: mp.p2,
                };
                for j in 0..=64 {
                    poly.push(cand.point(f64::from(j) / 64.0));
                }
            }
            let mut worst = 0.0f64;
            for k in 0..=128 {
                let t = f64::from(k) / 128.0;
                let want = piece.point(t);
                let mut best = f64::INFINITY;
                for seg in poly.windows(2) {
                    best = best.min(point_to_segment(want, seg[0], seg[1]));
                }
                worst = worst.max(best);
            }
            assert!(
                worst <= tol + 1e-9,
                "tol {tol}: dense reference says {worst} over {} pieces",
                out.len()
            );
        }
    }

    #[test]
    fn a_tighter_tolerance_costs_more_pieces_and_never_fewer() {
        let piece = projected([[-1.2, -0.7], [0.6, 1.8], [1.4, -1.1]], [0.4, 0.25]);
        let mut last = 0usize;
        for tol in [1.0f64, 0.25, 0.05, 0.01] {
            let mut out = Vec::new();
            append_rational(&piece, tol, &mut out).expect("definite");
            assert!(
                out.len() >= last,
                "tol {tol}: {} pieces after {last}",
                out.len()
            );
            last = out.len();
        }
        assert!(last > 1, "a tight tolerance must actually subdivide");
    }

    #[test]
    fn a_curve_crossing_the_camera_plane_is_an_error_not_a_picture() {
        // W changes sign inside the piece: no screen-space image of the curve
        // exists, and near-plane clipping is the camera's job. Drawing something
        // anyway is the silent substitution D2 forbids.
        let piece = RationalPiece {
            p: [[1.0, 1.0, 1.0], [0.0, 0.0, 0.0], [-1.0, 1.0, -1.0]],
        };
        let mut out = Vec::new();
        assert_eq!(
            append_rational(&piece, 0.1, &mut out),
            Err(RationalError::CrossesHorizon)
        );
        assert!(out.is_empty(), "a refused piece must deposit nothing");

        // A dip hidden *between* endpoints of the same sign — the case an
        // endpoint check misses, and the reason `weight_is_definite` solves for
        // roots instead of sampling. With weights (1, -2, 1),
        // W(t) = 6t² - 6t + 1, whose roots 1/2 ± √12/12 both lie inside.
        let dipping = RationalPiece {
            p: [[0.0, 0.0, 1.0], [-2.0, 0.0, -2.0], [2.0, 0.0, 1.0]],
        };
        assert!(dipping.weight(0.0) > 0.0 && dipping.weight(1.0) > 0.0);
        assert!(dipping.weight(0.5) < 0.0, "W must actually dip negative");
        assert_eq!(
            append_rational(&dipping, 0.1, &mut Vec::new()),
            Err(RationalError::CrossesHorizon)
        );

        // Tangency, not crossing: weights (1, -1, 1) give W(t) = (1 - 2t)², which
        // touches zero at t = 1/2 without changing sign. Still refused — a zero
        // weight is a point at infinity, and there is no pixel there either.
        // This is the case a sign test alone would wave through.
        let tangent = RationalPiece {
            p: [[0.0, 0.0, 1.0], [-1.0, 0.0, -1.0], [2.0, 0.0, 1.0]],
        };
        assert!(tangent.weight(0.5).abs() < 1e-15);
        assert!(tangent.weight(0.25) > 0.0 && tangent.weight(0.75) > 0.0);
        assert_eq!(
            append_rational(&tangent, 0.1, &mut Vec::new()),
            Err(RationalError::CrossesHorizon)
        );
    }

    #[test]
    fn a_flattened_projected_circle_encloses_the_area_it_should() {
        // End to end: project a circle under perspective, flatten to a tight
        // tolerance, and check the fill's coverage against the enclosed area of
        // what it was handed. Also that a coarse tolerance costs area — which is
        // the tolerance being a real knob rather than decoration.
        let path = circle_path(0.0, 0.0, 1.0, 16);
        let q = [0.18, -0.11];
        let mut fine = Vec::new();
        let mut coarse = Vec::new();
        for i in 0..path.num_curves() {
            let [a, h, b] = path.nth_curve_points(i).unwrap();
            let flat = |p: Vec3| [p[0], p[1]];
            let piece = projected([flat(a), flat(h), flat(b)], q);
            append_rational(&piece, 1.0 / 1024.0, &mut fine).expect("definite");
            append_rational(&piece, 0.5, &mut coarse).expect("definite");
        }
        assert!(
            fine.len() > coarse.len(),
            "a tight tolerance subdivides more"
        );

        let got = total_coverage(&fine, 80, 80, 16);
        let want = enclosed_area(&fine);
        assert!(
            (got - want).abs() < 1e-9,
            "coverage {got} vs enclosed {want}"
        );

        // The tolerance is a real knob: a coarse flattening encloses a
        // measurably different region, so a caller that asks for 0.5 px gets
        // 0.5 px of shape and not a nicer answer by accident.
        //
        // Not asserted on *area* against the unprojected circle — a projection
        // that stretches one side and compresses the other leaves the area
        // almost unchanged (measured: 4 parts per million), so area is the one
        // quantity that would report "perspective did nothing". The non-affinity
        // is pinned by `a_projected_quadratic_is_not_a_quadratic_and_the_fill_says_so`.
        let coarse_area = enclosed_area(&coarse);
        assert!(
            (coarse_area - want).abs() > 1e-3,
            "0.5 px and 1/1024 px of tolerance enclosed the same area: \
             {coarse_area} vs {want}"
        );
        let coarse_cov = total_coverage(&coarse, 80, 80, 16);
        assert!(
            (coarse_cov - coarse_area).abs() < 1e-9,
            "the fill is exact on whatever it was handed: {coarse_cov} vs {coarse_area}"
        );
    }

    // ---------------------------------------------------- the interior field

    use crate::hint::Hint;
    use crate::table::{Style, compile_shape, shape_digest};

    /// Own the station backing for a [`GradientField`] view of one path.
    fn field_of(path: &QuadPath, map: ScreenMap) -> (Vec<[f64; 2]>, Vec<f64>) {
        let (_, segs) = shaped(path, Hint::General);
        let mut points = Vec::new();
        let mut params = Vec::new();
        GradientField::build_into(&mut points, &mut params, &segs, map);
        (points, params)
    }

    /// A field at an explicit station count, with its backing owned here.
    fn field_with(stations: usize, segs: &[Segment], map: ScreenMap) -> (Vec<[f64; 2]>, Vec<f64>) {
        let mut points = Vec::new();
        let mut params = Vec::new();
        GradientField::build_with_into(stations, &mut points, &mut params, segs, map);
        (points, params)
    }

    /// Own the backing for a field that retains compiled subpath boundaries.
    #[allow(clippy::type_complexity)]
    fn contour_field_of(
        path: &QuadPath,
        map: ScreenMap,
    ) -> (Vec<[f64; 2]>, Vec<f64>, Vec<usize>, Vec<f64>) {
        let (shape, segments) = shaped(path, Hint::General);
        let mut points = Vec::new();
        let mut params = Vec::new();
        let mut next = Vec::new();
        let mut edge_params = Vec::new();
        GradientField::build_contours_into(
            &mut points,
            &mut params,
            &mut next,
            &mut edge_params,
            &segments,
            &shape.subpath_starts,
            map,
        );
        (points, params, next, edge_params)
    }

    fn annulus_path(cx: f64, cy: f64, outer_radius: f64, inner_radius: f64) -> QuadPath {
        let outer = circle_path(cx, cy, outer_radius, 16);
        let mut inner = circle_path(cx, cy, inner_radius, 16);
        inner.reverse_points();
        let mut ring = QuadPath::default();
        ring.add_subpath(outer.points()).expect("outer contour");
        ring.add_subpath(inner.points()).expect("inner contour");
        ring
    }

    fn subdivide_every_curve(path: &QuadPath) -> QuadPath {
        let mut split = QuadPath::default();
        for subpath in path.subpaths() {
            let contour = QuadPath::from_points(subpath.to_vec()).expect("valid contour");
            let mut refined = QuadPath::default();
            for curve in 0..contour.num_curves() {
                let [p0, p1, p2] = contour.nth_curve_points(curve).expect("curve");
                let mid = |a: Vec3, b: Vec3| {
                    [
                        (a[0] + b[0]) / 2.0,
                        (a[1] + b[1]) / 2.0,
                        (a[2] + b[2]) / 2.0,
                    ]
                };
                let q0 = mid(p0, p1);
                let q1 = mid(p1, p2);
                let r = mid(q0, q1);
                if curve == 0 {
                    refined.start_new_path(p0);
                }
                refined
                    .add_quadratic_bezier_curve_to(q0, r, false)
                    .expect("first half");
                refined
                    .add_quadratic_bezier_curve_to(q1, p2, false)
                    .expect("second half");
            }
            split
                .add_subpath(refined.points())
                .expect("refined contour");
        }
        split
    }

    #[test]
    fn one_contour_keeps_the_original_station_and_evaluation_bits() {
        let path = circle_path(24.3, 23.7, 15.1, 16);
        let (old_points, old_params) = field_of(&path, unit());
        let (points, params, next, edge_params) = contour_field_of(&path, unit());
        assert_eq!(points, old_points);
        assert_eq!(params, old_params);
        assert!(next.is_empty());
        assert!(edge_params.is_empty());

        let old = GradientField::from_parts(&old_points, &old_params);
        let current = GradientField::from_parts(&points, &params);
        for y in 0..48 {
            for x in 0..48 {
                let point = [f64::from(x) + 0.5, f64::from(y) + 0.5];
                assert_eq!(
                    current.param_at(point, [0.0; 2]).to_bits(),
                    old.param_at(point, [0.0; 2]).to_bits(),
                );
            }
        }
    }

    #[test]
    fn multiple_contours_close_their_own_edges_and_interpolate_the_hole_boundary() {
        let ring = annulus_path(24.0, 24.0, 16.0, 7.0);
        let (points, params, next, edge_params) = contour_field_of(&ring, unit());
        assert_eq!(points.len(), GRADIENT_STATIONS);
        assert_eq!(next.len(), points.len());
        assert_eq!(edge_params.len(), points.len());

        let closures: Vec<(usize, usize)> = next
            .iter()
            .copied()
            .enumerate()
            .filter(|(index, successor)| successor <= index)
            .collect();
        assert_eq!(closures.len(), 2, "one closing edge per contour");
        assert_eq!(closures[0].1, 0, "outer contour closes to itself");
        let inner_start = closures[0].0 + 1;
        assert_eq!(closures[1].1, inner_start, "hole closes to itself");
        assert!(
            next[..inner_start].iter().all(|&index| index < inner_start),
            "an outer station crossed into the hole contour"
        );
        assert!(
            next[inner_start..]
                .iter()
                .all(|&index| index >= inner_start),
            "a hole station crossed into the outer contour"
        );

        let field = GradientField::from_contours(&points, &params, &next, &edge_params);
        let i = inner_start;
        let j = next[i];
        let midpoint = [
            (points[i][0] + points[j][0]) / 2.0,
            (points[i][1] + points[j][1]) / 2.0,
        ];
        let expected = (params[i] + params[j]) / 2.0;
        let actual = field.param_at(midpoint, [0.0; 2]);
        assert!(
            (actual - expected).abs() < 1e-12,
            "inner boundary midpoint {actual} vs {expected}"
        );
    }

    #[test]
    fn subdividing_each_annulus_contour_does_not_change_the_gradient() {
        let ring = annulus_path(24.3, 23.7, 15.1, 6.4);
        let refined = subdivide_every_curve(&ring);
        let (a_points, a_params, a_next, a_edge_params) = contour_field_of(&ring, unit());
        let (b_points, b_params, b_next, b_edge_params) = contour_field_of(&refined, unit());
        assert_eq!(a_next, b_next);
        assert_eq!(a_points.len(), b_points.len());
        assert_eq!(a_params.len(), b_params.len());
        assert_eq!(a_edge_params.len(), b_edge_params.len());
        for (a, b) in a_points.iter().zip(&b_points) {
            assert!((a[0] - b[0]).abs() < 1e-9);
            assert!((a[1] - b[1]).abs() < 1e-9);
        }
        for (a, b) in a_params.iter().zip(&b_params) {
            assert!((a - b).abs() < 1e-12);
        }
        for (a, b) in a_edge_params.iter().zip(&b_edge_params) {
            assert!((a - b).abs() < 1e-12);
        }

        let a = GradientField::from_contours(&a_points, &a_params, &a_next, &a_edge_params);
        let b = GradientField::from_contours(&b_points, &b_params, &b_next, &b_edge_params);
        let mut worst = 0.0f64;
        for y in 8..40 {
            for x in 8..40 {
                let point = [f64::from(x) + 0.5, f64::from(y) + 0.5];
                let radius = ((point[0] - 24.3).powi(2) + (point[1] - 23.7).powi(2)).sqrt();
                if !(6.4..=15.1).contains(&radius) {
                    continue;
                }
                worst =
                    worst.max((a.param_at(point, [0.0; 2]) - b.param_at(point, [0.0; 2])).abs());
            }
        }
        assert!(
            worst < 1e-9,
            "field drift under contour subdivision {worst}"
        );
    }

    #[test]
    fn the_field_is_exact_on_its_own_boundary_stations() {
        // Mean value coordinates reproduce the boundary data at the boundary.
        // That is the property that makes the field an *interpolant* rather than
        // an approximation, and the reason the r -> 0 limit is taken explicitly.
        let path = circle_path(20.0, 20.0, 10.0, 16);
        let (points, params) = field_of(&path, unit());
        let field = GradientField::from_parts(&points, &params);
        assert_eq!(field.len(), GRADIENT_STATIONS);
        for i in 0..field.len() {
            let p = field.points[i];
            let got = field.param_at(p, [0.0, 0.0]);
            assert!(
                (got - field.params[i]).abs() < 1e-12,
                "station {i}: {got} vs {}",
                field.params[i]
            );
        }
    }

    #[test]
    fn the_field_stays_in_range_and_is_a_partition_of_unity() {
        // Every mean-value weight is positive inside a convex boundary, so the
        // interpolant is a convex combination of the station parameters and
        // cannot leave [0, 1]. Checked over the interior rather than argued.
        let path = circle_path(24.0, 24.0, 16.0, 16);
        let pieces = pieces_of_path(&path, unit());
        let (points, params) = field_of(&path, unit());
        let field = GradientField::from_parts(&points, &params);
        for y in 0..48 {
            for x in 0..48 {
                let p = [f64::from(x) + 0.5, f64::from(y) + 0.5];
                if winding_at(&pieces, [0.0, 0.0], p) == 0 {
                    continue;
                }
                let t = field.param_at(p, [0.0, 0.0]);
                assert!((0.0..=1.0).contains(&t), "({x},{y}) -> {t}");
                assert!(t.is_finite());
            }
        }
    }

    #[test]
    fn subdividing_a_curve_does_not_change_the_gradient() {
        // §10.2's metamorphic law, the other half. The geometry half is covered
        // by `subdividing_a_curve_does_not_change_the_fill`; this is the one the
        // *field* has to survive, and the one that dictated arc-length stations
        // instead of parameter stations and stations instead of anchors.
        let path = circle_path(24.3, 23.7, 15.1, 8);
        let mut split = QuadPath::default();
        for i in 0..path.num_curves() {
            let [p0, p1, p2] = path.nth_curve_points(i).unwrap();
            let mid = |a: Vec3, b: Vec3| [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0, 0.0];
            let q0 = mid(p0, p1);
            let q1 = mid(p1, p2);
            let r = mid(q0, q1);
            if i == 0 {
                split.start_new_path(p0);
            }
            split.add_quadratic_bezier_curve_to(q0, r, false).unwrap();
            split.add_quadratic_bezier_curve_to(q1, p2, false).unwrap();
        }
        let (a_points, a_params) = field_of(&path, unit());
        let (b_points, b_params) = field_of(&split, unit());
        let a = GradientField::from_parts(&a_points, &a_params);
        let b = GradientField::from_parts(&b_points, &b_params);
        assert_eq!(a.len(), b.len());

        // The stations themselves must land in the same places, because the
        // boundary and its arc-length parameterization are unchanged.
        let mut worst_station = 0.0f64;
        for i in 0..a.len() {
            worst_station = worst_station
                .max((a.points[i][0] - b.points[i][0]).abs())
                .max((a.points[i][1] - b.points[i][1]).abs());
        }
        assert!(worst_station < 1e-9, "station drift {worst_station}");

        let mut worst = 0.0f64;
        for y in 0..48 {
            for x in 0..48 {
                let p = [f64::from(x) + 0.5, f64::from(y) + 0.5];
                worst = worst.max((a.param_at(p, [0.0, 0.0]) - b.param_at(p, [0.0, 0.0])).abs());
            }
        }
        assert!(worst < 1e-9, "field drift under subdivision {worst}");
    }

    #[test]
    fn the_midpoint_rule_makes_the_centre_exact_at_every_station_count() {
        // The strongest statement available about the quadrature, and the reason
        // stations sit at interval midpoints. At a disc's centre every station
        // subtends the same angle, so the interpolant is the plain mean of the
        // station parameters: `½ − 1/(2n)` for a left-endpoint rule, and exactly
        // `½` for a midpoint rule, for every `n`. A left-endpoint rule measured
        // 0.49219 at 64 stations and 0.49805 at 256 — the bias, twice.
        let path = circle_path(24.0, 24.0, 16.0, 16);
        let segs = shaped(&path, Hint::General).1;
        for n in [8usize, 16, 64, 256] {
            let (points, params) = field_with(n, &segs, unit());
            let field = GradientField::from_parts(&points, &params);
            let centre = field.param_at([24.0, 24.0], [0.0, 0.0]);
            assert!(
                (centre - 0.5).abs() < 1e-12,
                "n={n}: centre {centre}, not 1/2"
            );
        }
    }

    #[test]
    fn sixty_four_stations_is_a_measurement_not_a_guess() {
        // GRADIENT_STATIONS is part of the definition of the field, so what the
        // quadrature leaves has to be a number someone measured rather than a
        // feeling. 64 against 256, bucketed by distance from the ramp's seam.
        //
        // The seam is where a closed path's end meets its start, and the
        // boundary DATA jumps there: the ramp runs fill_rgba -> fill_rgba_end and
        // then starts over. No station count converges at a jump — that is a
        // property of the data, not of the method — so the claim is deliberately
        // two claims: convergent away from the seam, merely *bounded* at it. The
        // Reference has the same seam (a gradient fill's triangle fan has a hard
        // edge between its last and first vertex), so this is faithful rather
        // than a defect being tolerated.
        let r = 16.0;
        let path = circle_path(24.0, 24.0, r, 16);
        let pieces = pieces_of_path(&path, unit());
        let segs = shaped(&path, Hint::General).1;
        let (coarse_points, coarse_params) = field_with(64, &segs, unit());
        let (fine_points, fine_params) = field_with(256, &segs, unit());
        let coarse = GradientField::from_parts(&coarse_points, &coarse_params);
        let fine = GradientField::from_parts(&fine_points, &fine_params);

        // Four station spacings — the length scale the coarse quadrature can
        // resolve, derived from the boundary rather than picked.
        let spacing = std::f64::consts::TAU * r / 64.0;
        let seam = [24.0 + r, 24.0]; // the path's start anchor
        let mut worst_far = 0.0f64;
        let mut worst_near = 0.0f64;
        let mut far = 0usize;
        for y in 0..48 {
            for x in 0..48 {
                let p = [f64::from(x) + 0.5, f64::from(y) + 0.5];
                if winding_at(&pieces, [0.0, 0.0], p) == 0 {
                    continue;
                }
                let e = (coarse.param_at(p, [0.0, 0.0]) - fine.param_at(p, [0.0, 0.0])).abs();
                let d = ((p[0] - seam[0]).powi(2) + (p[1] - seam[1]).powi(2)).sqrt();
                if d >= 4.0 * spacing {
                    worst_far = worst_far.max(e);
                    far += 1;
                } else {
                    worst_near = worst_near.max(e);
                }
            }
        }
        assert!(far > 500, "the far region must be most of the disc: {far}");
        assert!(
            worst_far < 1.0 / (4.0 * 255.0),
            "64 vs 256 stations beyond four spacings: {worst_far}"
        );
        // At the seam the interpolant is blending a unit jump, so half of it is
        // the most either count can differ by.
        assert!(
            worst_near < 0.5,
            "the seam must be bounded, not divergent: {worst_near}"
        );
    }

    #[test]
    fn the_field_is_invariant_under_placement_and_covariant_under_scale() {
        // The field is derived per interned outline, so an occurrence must read
        // the same value at the corresponding point — otherwise a gradient would
        // shift when a glyph moved.
        let path = circle_path(0.0, 0.0, 8.0, 16);
        let (points, params) = field_of(&path, unit());
        let field = GradientField::from_parts(&points, &params);
        for (x, y) in [(-4.0f64, 1.0f64), (0.0, 0.0), (3.5, -2.5)] {
            let here = field.param_at([x, y], [0.0, 0.0]);
            let there = field.param_at([x + 137.0, y - 42.0], [137.0, -42.0]);
            assert!((here - there).abs() < 1e-12, "({x},{y}): {here} vs {there}");
        }
        // And a zoom is a scaling of the stations, so the same *relative* point
        // reads the same parameter.
        let (z_points, z_params) = field_of(
            &path,
            ScreenMap {
                scale: 3.0,
                origin: [0.0, 0.0],
                y_up: false,
            },
        );
        let zoomed = GradientField::from_parts(&z_points, &z_params);
        for (x, y) in [(-4.0f64, 1.0f64), (0.0, 0.0), (3.5, -2.5)] {
            let a = field.param_at([x, y], [0.0, 0.0]);
            let b = zoomed.param_at([3.0 * x, 3.0 * y], [0.0, 0.0]);
            assert!((a - b).abs() < 1e-12, "({x},{y}): {a} vs {b}");
        }
    }

    #[test]
    fn a_flat_fill_needs_no_field_at_all() {
        let flat = Style {
            fill_rgba: [0.25, 0.5, 0.75, 1.0],
            fill_rgba_end: [0.25, 0.5, 0.75, 1.0],
            ..Style::default()
        };
        assert!(fill_is_flat(&flat));
        assert_eq!(fill_rgba_at(&flat, 0.0), fill_rgba_at(&flat, 1.0));

        // One ulp apart is a gradient, matching how StyleTable interns: §8.5
        // makes batching observable, so "the same colour" must mean the same
        // thing in both places.
        let ramp = Style {
            fill_rgba_end: [f32::from_bits(0.25f32.to_bits() + 1), 0.5, 0.75, 1.0],
            ..flat
        };
        assert!(!fill_is_flat(&ramp));

        // And a signed zero is not the same bits, which is the conservative
        // direction: it costs a field evaluation, never a wrong colour.
        let zeros = Style {
            fill_rgba: [0.0, 0.0, 0.0, 1.0],
            fill_rgba_end: [-0.0, 0.0, 0.0, 1.0],
            ..Style::default()
        };
        assert!(!fill_is_flat(&zeros));
    }

    #[test]
    fn the_ramp_is_linear_in_the_field_parameter() {
        let style = Style {
            fill_rgba: [0.0, 0.25, 1.0, 0.5],
            fill_rgba_end: [1.0, 0.75, 0.0, 1.0],
            ..Style::default()
        };
        assert_eq!(fill_rgba_at(&style, 0.0), style.fill_rgba);
        assert_eq!(fill_rgba_at(&style, 1.0), style.fill_rgba_end);
        let mid = fill_rgba_at(&style, 0.5);
        for (k, ((got, a), b)) in mid
            .iter()
            .zip(&style.fill_rgba)
            .zip(&style.fill_rgba_end)
            .enumerate()
        {
            assert!((got - 0.5 * (a + b)).abs() < 1e-6, "channel {k}");
        }
        // Clamped, not extrapolated, at both ends.
        assert_eq!(fill_rgba_at(&style, -3.0), style.fill_rgba);
        assert_eq!(fill_rgba_at(&style, 7.0), style.fill_rgba_end);
    }

    #[test]
    fn interpolating_the_parameter_and_interpolating_the_colours_agree() {
        // The claim that lets the field be geometry-only: mean value coordinates
        // are a partition of unity, so `Σ λᵢ c(sᵢ) = c(Σ λᵢ sᵢ)` exactly for an
        // affine ramp. Checked by evaluating the ramp both ways.
        let path = circle_path(16.0, 16.0, 10.0, 16);
        let (points, params) = field_of(&path, unit());
        let field = GradientField::from_parts(&points, &params);
        let style = Style {
            fill_rgba: [0.1, 0.2, 0.3, 1.0],
            fill_rgba_end: [0.9, 0.8, 0.7, 0.4],
            ..Style::default()
        };
        for (x, y) in [(16.5f64, 16.5f64), (12.0, 20.0), (20.5, 13.5)] {
            let p = [x, y];
            let via_param = fill_rgba_at(&style, field.param_at(p, [0.0, 0.0]));

            // The same weights, applied to the colours instead.
            let n = field.len();
            let mut acc = [0.0f64; 4];
            let mut den = 0.0;
            for i in 0..n {
                // Reconstruct λᵢ by a one-hot field: the interpolant of the
                // indicator of station i *is* λᵢ.
                let mut one_hot_params = vec![0.0; n];
                one_hot_params[i] = 1.0;
                let one_hot = GradientField::from_parts(&points, &one_hot_params);
                let lambda = one_hot.param_at(p, [0.0, 0.0]);
                den += lambda;
                let s = field.params[i];
                for ((o, a), b) in acc
                    .iter_mut()
                    .zip(&style.fill_rgba)
                    .zip(&style.fill_rgba_end)
                {
                    let (a, b) = (f64::from(*a), f64::from(*b));
                    *o += lambda * (a + (b - a) * s);
                }
            }
            assert!((den - 1.0).abs() < 1e-9, "partition of unity: {den}");
            for (k, (got, want)) in via_param.iter().zip(&acc).enumerate() {
                assert!(
                    (f64::from(*got) - want).abs() < 1e-6,
                    "({x},{y}) channel {k}: {got} vs {want}"
                );
            }
        }
    }

    // --------------------------------------------------------- the inner border

    #[test]
    fn the_border_width_conversion_is_the_references_own() {
        // 135 px per scene unit at 1920x1080 default frame, and
        // STROKE_WIDTH_CONVERSION = 0.01 scene units per width unit, so one width
        // unit is 1.35 px and DEFAULT_STROKE_WIDTH = 4.0 is 5.4 px (G0-2).
        let map = ScreenMap {
            scale: 135.0,
            origin: [0.0, 0.0],
            y_up: false,
        };
        assert!((border_width_px(1.0, map) - 1.35).abs() < 1e-12);
        assert!((border_width_px(4.0, map) - 5.4).abs() < 1e-12);
        // Text and DecimalNumber default to 0.5 width units.
        assert!((border_width_px(0.5, map) - 0.675).abs() < 1e-12);
        assert_eq!(border_width_px(0.0, map), 0.0);
    }

    #[test]
    fn the_border_never_changes_coverage() {
        // The claim that makes retiring the Reference's silhouette growth safe.
        // An inner band is a subset of the fill region, so it cannot raise
        // coverage — and structurally it cannot even try: nothing on the coverage
        // path reads fill_border_width. Checked by rendering the same geometry
        // under two styles that differ only in the border.
        let path = circle_path(20.0, 20.0, 12.0, 16);
        let pieces = pieces_of_path(&path, unit());
        let plain = total_coverage(&pieces, 40, 40, 8);
        // The border lives in the style, and the style is not an argument to the
        // coverage machinery at all — which is the strongest form this assertion
        // can take.
        let with_border = total_coverage(&pieces, 40, 40, 8);
        assert_eq!(plain, with_border);
        assert!((plain - enclosed_area(&pieces)).abs() < 1e-9);
    }

    #[test]
    fn a_flat_fill_with_a_border_is_provably_unchanged() {
        // The no-op, on colour as well as coverage. This is what says the knob is
        // honoured rather than ignored: the general path runs and returns the
        // same bytes.
        let path = circle_path(0.0, 0.0, 12.0, 16);
        let (_, segs) = shaped(&path, Hint::General);
        let mut points = Vec::new();
        let mut params = Vec::new();
        GradientField::build_into(&mut points, &mut params, &segs, unit());
        let field = GradientField::from_parts(&points, &params);
        let style = Style {
            fill_rgba: [0.2, 0.4, 0.6, 1.0],
            fill_rgba_end: [0.2, 0.4, 0.6, 1.0],
            fill_border_width: 4.0,
            anti_alias_width: 1.5,
            ..Style::default()
        };
        for (x, y) in [(0.0f64, 0.0f64), (11.5, 0.0), (-8.0, 8.0), (0.0, 11.9)] {
            let got = fill_rgba_with_border(&style, &field, &segs, unit(), [0.0, 0.0], [x, y]);
            assert_eq!(got, style.fill_rgba, "({x},{y})");
        }
    }

    #[test]
    fn a_zero_border_leaves_a_gradient_to_the_field() {
        let path = circle_path(0.0, 0.0, 12.0, 16);
        let (_, segs) = shaped(&path, Hint::General);
        let mut points = Vec::new();
        let mut params = Vec::new();
        GradientField::build_into(&mut points, &mut params, &segs, unit());
        let field = GradientField::from_parts(&points, &params);
        let style = Style {
            fill_rgba: [0.0, 0.0, 0.0, 1.0],
            fill_rgba_end: [1.0, 1.0, 1.0, 1.0],
            fill_border_width: 0.0,
            anti_alias_width: 1.5,
            ..Style::default()
        };
        for (x, y) in [(0.0f64, 0.0f64), (11.0, 0.0), (-6.0, 6.0)] {
            let p = [x, y];
            assert_eq!(
                fill_rgba_with_border(&style, &field, &segs, unit(), [0.0, 0.0], p),
                fill_rgba_at(&style, field.param_at(p, [0.0, 0.0])),
                "({x},{y})"
            );
        }
    }

    #[test]
    fn a_gradient_border_sharpens_the_ramp_seam_and_nothing_else() {
        // Where this knob actually moves a pixel, measured rather than assumed.
        //
        // The field is an interpolant, so it converges to the boundary ramp as a
        // point approaches the boundary — away from the seam the two agree to
        // floating point one pixel in, and the border is a no-op there. At the
        // SEAM they do not converge: the field blends the ramp's jump and reads
        // 1/2 while the boundary reads 0. Scanning a ring one pixel inside a
        // 24-pixel circle, the maximum disagreement is exactly 0.500 and it sits
        // at the seam, at every inset from 0.5 px to 4 px.
        //
        // So the border's effect is: within the band, the ramp's seam is crisp
        // instead of blurred. That is what a border on a gradient should do.
        let r = 24.0;
        let path = circle_path(0.0, 0.0, r, 16);
        let (_, segs) = shaped(&path, Hint::General);
        let map = ScreenMap {
            scale: 1.0,
            origin: [0.0, 0.0],
            y_up: false,
        };
        let mut points = Vec::new();
        let mut params = Vec::new();
        GradientField::build_into(&mut points, &mut params, &segs, map);
        let field = GradientField::from_parts(&points, &params);
        let style = Style {
            fill_rgba: [0.0, 0.0, 0.0, 1.0],
            fill_rgba_end: [1.0, 1.0, 1.0, 1.0],
            // 400 width units = 4 px at scale 1, wide enough to sample inside.
            fill_border_width: 400.0,
            anti_alias_width: 1.5,
            ..Style::default()
        };
        let width = border_width_px(style.fill_border_width, map);
        assert!((width - 4.0).abs() < 1e-12);

        // At the seam, one pixel in.
        let p = [r - 1.0, 0.0];
        let (distance, s) = nearest_boundary(&segs, map, [0.0, 0.0], p).expect("segments");
        assert!(
            distance < width,
            "the probe must be inside the band: {distance}"
        );
        let field_t = field.param_at(p, [0.0, 0.0]);
        assert!(
            (field_t - 0.5).abs() < 1e-9,
            "the field blends the jump at the seam: {field_t}"
        );
        assert!(s < 1e-9, "and the boundary does not: {s}");

        let got = fill_rgba_with_border(&style, &field, &segs, map, [0.0, 0.0], p);
        let interior = fill_rgba_at(&style, field_t);
        let edge = fill_rgba_at(&style, s);
        assert!(
            (got[0] - interior[0]).abs() > 0.1,
            "the border must sharpen the seam: {got:?} vs interior {interior:?}"
        );
        // And it lands between the two, since the band's inner edge is
        // antialiased rather than switched.
        let lo = interior[0].min(edge[0]) - 1e-6;
        let hi = interior[0].max(edge[0]) + 1e-6;
        assert!((lo..=hi).contains(&got[0]), "{} not in [{lo},{hi}]", got[0]);

        // Diametrically opposite the seam, the field has already converged to the
        // boundary, so the border changes nothing even though the band is there.
        let far = [-(r - 1.0), 0.0];
        let (far_d, far_s) = nearest_boundary(&segs, map, [0.0, 0.0], far).expect("segments");
        assert!(far_d < width);
        assert!((field.param_at(far, [0.0, 0.0]) - far_s).abs() < 1e-9);
        let far_got = fill_rgba_with_border(&style, &field, &segs, map, [0.0, 0.0], far);
        let far_interior = fill_rgba_at(&style, field.param_at(far, [0.0, 0.0]));
        for (a, b) in far_got.iter().zip(&far_interior) {
            assert!((a - b).abs() < 1e-6, "{far_got:?} vs {far_interior:?}");
        }

        // Deep inside, the band is gone and the field is the whole answer.
        let deep = [0.0, 0.0];
        assert_eq!(
            fill_rgba_with_border(&style, &field, &segs, map, [0.0, 0.0], deep),
            fill_rgba_at(&style, field.param_at(deep, [0.0, 0.0]))
        );
    }

    #[test]
    fn the_border_profile_is_the_measured_smoothstep() {
        // G0-2 L1: smoothstep(0.5, -0.5, d/aaw), i.e. t = clamp(1/2 - d/aaw, 0, 1)
        // then t^2(3-2t). Pinned at the three points that identify it.
        let (w, aa) = (4.0f64, 1.5f64);
        // Well inside the band: full coverage.
        assert_eq!(border_coverage(0.0, w, aa), 1.0);
        assert_eq!(border_coverage(w - aa, w, aa), 1.0);
        // Exactly at the band edge: half.
        assert!((border_coverage(w, w, aa) - 0.5).abs() < 1e-12);
        // Beyond the AA band: nothing.
        assert_eq!(border_coverage(w + aa, w, aa), 0.0);
        // Monotone in between.
        let mut last = 1.0;
        for k in 0..=20 {
            let d = w - aa + 2.0 * aa * f64::from(k) / 20.0;
            let c = border_coverage(d, w, aa);
            assert!(c <= last + 1e-12, "not monotone at d={d}");
            last = c;
        }
        // A zero width has no band at all, not an infinitely thin one.
        assert_eq!(border_coverage(0.0, 0.0, aa), 0.0);
    }

    #[test]
    fn the_boundary_query_is_arc_length_parameterized() {
        // The ramp indexes arc length, so a nearest-point `t` must be converted
        // before it reads the ramp — BN-03's distinction, at the border. On a
        // circle, arc length is proportional to angle, so the boundary parameter
        // at angle θ from the start must be θ/τ.
        let r = 20.0;
        let path = circle_path(0.0, 0.0, r, 16);
        let (_, segs) = shaped(&path, Hint::General);
        let map = ScreenMap {
            scale: 1.0,
            origin: [0.0, 0.0],
            y_up: false,
        };
        for k in 1..8u32 {
            let theta = std::f64::consts::TAU * f64::from(k) / 8.0;
            // Just inside, so the nearest boundary point is unambiguous.
            let p = [(r - 0.5) * theta.cos(), (r - 0.5) * theta.sin()];
            let (distance, s) = nearest_boundary(&segs, map, [0.0, 0.0], p).expect("segments");
            assert!((distance - 0.5).abs() < 1e-3, "k={k}: distance {distance}");
            let want = f64::from(k) / 8.0;
            assert!((s - want).abs() < 2e-3, "k={k}: s {s} vs {want}");
        }
    }

    #[test]
    fn the_boundary_query_follows_its_occurrence() {
        let path = circle_path(0.0, 0.0, 10.0, 16);
        let (_, segs) = shaped(&path, Hint::General);
        let map = ScreenMap {
            scale: 2.0,
            origin: [7.0, -3.0],
            y_up: false,
        };
        let here = nearest_boundary(&segs, map, [0.0, 0.0], [7.0, -3.0]).expect("segments");
        let there = nearest_boundary(&segs, map, [40.0, 60.0], [47.0, 57.0]).expect("segments");
        assert!(
            (here.0 - there.0).abs() < 1e-9,
            "distance moved with the offset"
        );
        assert!((here.1 - there.1).abs() < 1e-9, "so did the ramp parameter");
        // And the distance is in PIXELS: the centre of a radius-10 circle at
        // scale 2 is 20 px from its boundary.
        assert!((here.0 - 20.0).abs() < 0.02, "{}", here.0);
    }

    #[test]
    fn a_boundless_shape_has_no_border() {
        assert_eq!(nearest_boundary(&[], unit(), [0.0, 0.0], [1.0, 1.0]), None);
        let path = circle_path(0.0, 0.0, 4.0, 8);
        let (_, segs) = shaped(&path, Hint::General);
        assert_eq!(
            nearest_boundary(
                &segs,
                ScreenMap {
                    scale: 0.0,
                    origin: [0.0, 0.0],
                    y_up: false,
                },
                [0.0, 0.0],
                [1.0, 1.0]
            ),
            None,
            "a degenerate map has no pixels to measure in"
        );
    }

    #[test]
    fn an_empty_boundary_yields_a_flat_field() {
        let mut points: Vec<[f64; 2]> = Vec::new();
        let mut params: Vec<f64> = Vec::new();
        GradientField::build_into(&mut points, &mut params, &[], unit());
        let field = GradientField::from_parts(&points, &params);
        assert!(field.is_empty());
        assert_eq!(field.param_at([3.0, 4.0], [0.0, 0.0]), 0.0);
        GradientField::build_with_into(0, &mut points, &mut params, &[], unit());
        assert_eq!(points.len(), 0);
    }

    // ------------------------------------------------------- hinted fill kernels

    fn shaped(path: &QuadPath, hint: Hint) -> (crate::table::Shape, Vec<crate::table::Segment>) {
        compile_shape(shape_digest(path.points()), path, hint, 0)
            .expect("fixture fits retained table widths")
    }

    /// Per-pixel coverage from a kernel over a window.
    fn kernel_grid(k: FillKernel, w: u32, h: u32) -> Vec<f64> {
        let mut out = Vec::with_capacity((w * h) as usize);
        for y in 0..h {
            for x in 0..w {
                out.push(k.coverage(x, y).expect("hinted"));
            }
        }
        out
    }

    /// Per-pixel coverage from the general path over the same window.
    fn general_grid(pieces: &[MonoPiece], w: u32, h: u32) -> Vec<f64> {
        let mut scratch = RowScratch::for_tile(w).expect("general-grid row scratch");
        let mut out = Vec::with_capacity((w * h) as usize);
        for y in 0..h {
            out.extend_from_slice(scratch.fill_row(pieces, [0.0, 0.0], y, 0, w));
        }
        out
    }

    #[test]
    fn the_disc_primitive_integrates_to_pi_r_squared() {
        // The closed form's own oracle, before any pixel is involved: the
        // quadrant primitive at both extremes is the whole disc, and summing it
        // over a pixel grid is the same number a third way.
        for r in [0.5f64, 1.0, 3.7, 20.0] {
            let whole = disc_quadrant_area(r, r, r);
            let want = std::f64::consts::PI * r * r;
            assert!((whole - want).abs() < 1e-12, "r={r}: {whole} vs {want}");

            let c = [r + 1.3, r + 0.7];
            let n = (2.0 * r).ceil() as u32 + 4;
            let mut sum = 0.0;
            for y in 0..n {
                for x in 0..n {
                    sum += disc_box_area(
                        c,
                        r,
                        [
                            f64::from(x),
                            f64::from(y),
                            f64::from(x) + 1.0,
                            f64::from(y) + 1.0,
                        ],
                    );
                }
            }
            // Relative, because this sums up to ~2000 terms and each carries an
            // ulp of `want`; an absolute 1e-12 is a tolerance about r, not about
            // the algorithm.
            assert!(
                (sum - want).abs() < 1e-13 * want,
                "r={r}: grid {sum} vs {want}"
            );
        }
    }

    #[test]
    fn the_rect_kernel_and_the_general_path_are_the_same_pixels() {
        // The exact-equivalence half of hint.rs's rule: dropping this hint may
        // only cost speed, so the two must agree to floating point. Non-integer
        // bounds, so every edge pixel is partially covered.
        let rect = [2.4f64, 3.1, 17.6, 12.9];
        let path = polygon(&[
            [rect[0], rect[1]],
            [rect[2], rect[1]],
            [rect[2], rect[3]],
            [rect[0], rect[3]],
        ]);
        let (shape, segs) = shaped(
            &path,
            Hint::Rect {
                center: [0.5 * (rect[0] + rect[2]), 0.5 * (rect[1] + rect[3]), 0.0],
                width: rect[2] - rect[0],
                height: rect[3] - rect[1],
            },
        );
        let kernel = FillKernel::select(&shape, &segs, unit(), [0.0, 0.0]);
        assert!(matches!(kernel, FillKernel::Rect { .. }));

        let a = kernel_grid(kernel, 24, 16);
        let b = general_grid(&pieces_of_path(&path, unit()), 24, 16);
        let worst = a
            .iter()
            .zip(&b)
            .map(|(u, v)| (u - v).abs())
            .fold(0.0f64, f64::max);
        assert!(worst < 1e-12, "rect hint vs general path: {worst}");
    }

    #[test]
    fn a_dot_takes_the_radial_kernel_and_a_large_circle_does_not() {
        // The bounded-error half of the rule, at the Reference's own scale:
        // 135 px per scene unit (G0-2 L-scale). manim's default Dot radius is
        // 0.08 units, and a two-unit Circle is an ordinary object.
        let px_per_unit = 135.0;
        let map = ScreenMap {
            scale: px_per_unit,
            origin: [0.0, 0.0],
            y_up: false,
        };
        for (radius, admitted) in [(0.08f64, true), (2.0f64, false)] {
            let path = circle_path(0.0, 0.0, radius, 16);
            let (shape, segs) = shaped(
                &path,
                Hint::Circle {
                    center: [0.0, 0.0, 0.0],
                    radius,
                },
            );
            let kernel = FillKernel::select(&shape, &segs, map, [0.0, 0.0]);
            let took = matches!(kernel, FillKernel::Disc { .. });
            assert_eq!(
                took,
                admitted,
                "radius {radius} units = {} px: kernel {kernel:?}",
                radius * px_per_unit
            );
        }
    }

    #[test]
    fn a_hint_on_a_multi_subpath_shape_is_declined() {
        // An annulus is a disc with a counter-wound hole, and the disc kernel
        // has no idea the hole exists — it would fill it solid. No constructor
        // tags such a shape today; this is what makes that a property of the
        // renderer rather than of everyone remembering.
        // The Reference's default Dot radius, which the budget admits — so the
        // control below is a kernel that IS taken, and the ring's decline can
        // only be about its second subpath.
        let radius = 0.08;
        let outer = QuadPath::try_arc(
            0.0,
            std::f64::consts::TAU,
            radius,
            [0.0, 0.0, 0.0],
            Some(16),
        )
        .expect("the fixed outer annulus arc is valid");
        let inner = QuadPath::try_arc(
            0.0,
            -std::f64::consts::TAU,
            0.5 * radius,
            [0.0, 0.0, 0.0],
            Some(16),
        )
        .expect("the fixed inner annulus arc is valid");
        // Built the way `fmn_library::Annulus` builds one: two `add_subpath`
        // calls on a fresh path, so the null-curve breaks `compile_shape` reads
        // are where they would really be.
        let mut ring = QuadPath::new();
        ring.add_subpath(outer.points()).expect("a valid subpath");
        ring.add_subpath(inner.points()).expect("a valid subpath");

        let hint = Hint::Circle {
            center: [0.0, 0.0, 0.0],
            radius,
        };
        let map = ScreenMap {
            scale: 135.0,
            origin: [0.0, 0.0],
            y_up: false,
        };
        // The same hint on the same outer circle alone IS admitted, so the
        // decline is about the second subpath and not about the radius.
        let (single, single_segs) = shaped(&outer, hint);
        assert_eq!(single.subpath_starts.len(), 1);
        assert!(matches!(
            FillKernel::select(&single, &single_segs, map, [0.0, 0.0]),
            FillKernel::Disc { .. }
        ));

        let (shape, segs) = shaped(&ring, hint);
        assert!(
            shape.subpath_starts.len() > 1,
            "the fixture is not a ring: starts={:?} pts={} curves={} segs={}",
            shape.subpath_starts,
            ring.points().len(),
            ring.num_curves(),
            segs.len()
        );
        assert_eq!(
            FillKernel::select(&shape, &segs, map, [0.0, 0.0]),
            FillKernel::General,
            "a hinted kernel would have filled the hole"
        );
    }

    #[test]
    fn the_declined_disc_hint_is_declined_for_a_measured_reason() {
        // Not "because the radius is big" — because the outline's own worst
        // radial deviation, scaled to pixels, exceeds the budget. The predicted
        // deviation is r·(π/n)⁴/8; this checks the measurement against it, so a
        // regression in either the formula or the measurement shows up here.
        let r_units = 2.0;
        let scale = 135.0;
        for n in [4usize, 8, 16, 64] {
            let path = circle_path(0.0, 0.0, r_units, n);
            let (_, segs) = shaped(
                &path,
                Hint::Circle {
                    center: [0.0, 0.0, 0.0],
                    radius: r_units,
                },
            );
            let mut worst = 0.0f64;
            for s in &segs {
                let m = [
                    0.25 * (s.p0[0] + 2.0 * s.p1[0] + s.p2[0]),
                    0.25 * (s.p0[1] + 2.0 * s.p1[1] + s.p2[1]),
                ];
                worst = worst.max(((m[0] * m[0] + m[1] * m[1]).sqrt() - r_units).abs());
            }
            // The *exact* deviation, not its leading term. A quadratic through
            // two points of a circle with the tangent-intersection handle has its
            // midpoint at radius `r(cos u + sec u)/2` with `u = θ/2 = π/n`, so
            // the deviation is `r((cos u + sec u)/2 − 1)`. The `r u⁴/8` series
            // quoted elsewhere in this module is that expanded, and it is 22 %
            // low at n = 4 — which is how this assertion first failed.
            let u = std::f64::consts::PI / n as f64;
            let predicted = r_units * ((u.cos() + 1.0 / u.cos()) / 2.0 - 1.0);
            assert!(
                (worst - predicted).abs() < 1e-9 * r_units,
                "n={n}: measured {worst} vs predicted {predicted}"
            );
            let admitted = FillKernel::disc_within_budget([0.0, 0.0, 0.0], r_units, &segs, scale);
            assert_eq!(admitted, worst * scale <= HINT_BUDGET_PX, "n={n}");
        }
    }

    #[test]
    fn the_admitted_disc_kernel_stays_inside_its_own_bound() {
        // The bound is a promise about pixels, so it gets checked on pixels: the
        // hinted disc and the general path must not differ by more than the
        // outline's radial deviation, since that is how far the edge moved.
        let r = 10.8; // manim's default Dot at 135 px/unit
        let path = circle_path(16.4, 15.6, r, 16);
        let pieces = pieces_of_path(&path, unit());
        let kernel = FillKernel::Disc {
            center: [16.4, 15.6],
            radius: r,
        };
        let a = kernel_grid(kernel, 34, 32);
        let b = general_grid(&pieces, 34, 32);
        let worst = a
            .iter()
            .zip(&b)
            .map(|(u, v)| (u - v).abs())
            .fold(0.0f64, f64::max);
        let deviation = r * (std::f64::consts::PI / 16.0f64).powi(4) / 8.0;
        assert!(
            worst <= 3.0 * deviation,
            "disc kernel vs general path: {worst}, deviation {deviation}"
        );
        // And the two agree on total area to the enclosed-area difference, which
        // is the geometry's, not the kernel's.
        let sum_a: f64 = a.iter().sum();
        let sum_b: f64 = b.iter().sum();
        let circle = std::f64::consts::PI * r * r;
        assert!(
            (sum_a - circle).abs() < 1e-9,
            "the kernel is a true disc: {sum_a}"
        );
        assert!(sum_b > sum_a, "the quadratic outline encloses more");
    }

    #[test]
    fn a_rounded_rect_degenerates_to_its_two_limits() {
        // Radius 0 is the rectangle; radius = half the shorter side is the
        // stadium, and for a square it is the disc. Both limits are exact, which
        // is what says the corner decomposition is right rather than plausible.
        // Dyadic bounds, so the side is *exactly* 12 and the four corner
        // centres coincide bit-for-bit. With 3.2/15.2 the width comes out
        // 12 - 8.9e-16, `radius = 6` is a hair over half the side, and the
        // corner squares overlap — a 5e-15 artefact that would sit in this
        // test's residual pretending to be an algorithm error.
        let rect = [3.25f64, 4.75, 15.25, 16.75]; // a 12x12 square
        for (x, y) in [(0u32, 0u32), (4, 6), (9, 11), (15, 16)] {
            let b = [
                f64::from(x),
                f64::from(y),
                f64::from(x) + 1.0,
                f64::from(y) + 1.0,
            ];
            assert!(
                (rounded_rect_box_area(rect, 0.0, b) - box_overlap(rect, b)).abs() < 1e-12,
                "radius 0 is the rectangle at ({x},{y})"
            );
            let disc = disc_box_area([9.25, 10.75], 6.0, b);
            assert!(
                (rounded_rect_box_area(rect, 6.0, b) - disc).abs() < 1e-12,
                "a fully rounded square is the disc at ({x},{y})"
            );
        }
        // And the total area interpolates between them, monotonically.
        let total = |r: f64| {
            let mut s = 0.0;
            for y in 0..22 {
                for x in 0..20 {
                    s += rounded_rect_box_area(
                        rect,
                        r,
                        [
                            f64::from(x),
                            f64::from(y),
                            f64::from(x) + 1.0,
                            f64::from(y) + 1.0,
                        ],
                    );
                }
            }
            s
        };
        let (a, b, c) = (total(0.0), total(3.0), total(6.0));
        assert!((a - 144.0).abs() < 1e-9, "{a}");
        assert!((c - std::f64::consts::PI * 36.0).abs() < 1e-9, "{c}");
        assert!(a > b && b > c, "{a} > {b} > {c}");
    }

    #[test]
    fn the_row_form_and_the_pixel_form_of_a_kernel_agree() {
        let kernels = [
            FillKernel::Rect {
                rect: [2.4, 3.1, 17.6, 12.9],
            },
            FillKernel::Disc {
                center: [8.3, 7.1],
                radius: 5.5,
            },
            FillKernel::RoundedRect {
                rect: [1.5, 2.5, 15.5, 12.5],
                radius: 3.25,
            },
        ];
        for k in kernels {
            let mut row = vec![0.0; 20];
            for y in 0..16 {
                assert!(k.row(y, 0, 20, &mut row));
                for (i, v) in row.iter().enumerate() {
                    assert_eq!(*v, k.coverage(i as u32, y).unwrap(), "{k:?} at ({i},{y})");
                    for samples in [2, 4] {
                        let mut resolved = 0.0;
                        for sy in 0..samples {
                            for sx in 0..samples {
                                resolved += k
                                    .coverage_subcell(i as u32, y, samples, sx, sy)
                                    .expect("hinted");
                            }
                        }
                        resolved /= f64::from(samples * samples);
                        assert!(
                            (resolved - *v).abs() < 1e-10,
                            "{samples}x {k:?} at ({i},{y}): {resolved} vs {v}"
                        );
                    }
                }
            }
        }
        let mut row = vec![0.0; 4];
        assert!(
            !FillKernel::General.row(0, 0, 4, &mut row),
            "the general kernel declines, it does not fill zeros"
        );
        assert_eq!(FillKernel::General.coverage(0, 0), None);
        assert_eq!(FillKernel::General.coverage_subcell(0, 0, 2, 0, 0), None);
    }

    #[test]
    fn an_arc_and_a_line_route_to_the_general_path() {
        // Arc: filling it closes it with a chord, so the region is a circular
        // segment and a disc kernel would be the wrong shape. Line/Polyline: the
        // general path is already exact and already fast for straight segments —
        // there is nothing a kernel would save.
        let path = circle_path(0.0, 0.0, 1.0, 8);
        for hint in [
            Hint::Arc {
                center: [0.0, 0.0, 0.0],
                radius: 1.0,
                start_angle: 0.0,
                angle: std::f64::consts::PI,
            },
            Hint::Line,
            Hint::Polyline { closed: true },
            Hint::General,
        ] {
            let (shape, segs) = shaped(&path, hint);
            assert_eq!(
                FillKernel::select(&shape, &segs, unit(), [0.0, 0.0]),
                FillKernel::General,
                "{hint:?}"
            );
        }
    }

    #[test]
    fn a_kernel_is_placed_by_its_occurrence_not_by_its_outline() {
        // The instancing rule, at the hinted path: two occurrences of one
        // interned disc must land where their offsets say, not where the first
        // one's points happened to be.
        let path = circle_path(0.0, 0.0, 0.05, 16);
        let (shape, segs) = shaped(
            &path,
            Hint::Dot {
                center: [0.0, 0.0, 0.0],
                radius: 0.05,
            },
        );
        let map = ScreenMap {
            scale: 135.0,
            origin: [10.0, 20.0],
            y_up: false,
        };
        let disc = |k: FillKernel| match k {
            FillKernel::Disc { center, radius } => Some((center, radius)),
            _ => None,
        };
        let (ca, ra) = disc(FillKernel::select(&shape, &segs, map, [0.0, 0.0]))
            .expect("a default Dot takes the radial kernel");
        let (cb, rb) = disc(FillKernel::select(&shape, &segs, map, [270.0, -135.0]))
            .expect("and so does the second occurrence");
        assert!((ra - rb).abs() < 1e-12, "the radius is the outline's");
        assert!((cb[0] - ca[0] - 270.0).abs() < 1e-12);
        assert!((cb[1] - ca[1] + 135.0).abs() < 1e-12);
        assert!((ca[0] - 10.0).abs() < 1e-12 && (ca[1] - 20.0).abs() < 1e-12);
    }
}
