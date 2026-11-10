//! Ear-clip polygon triangulation with hole support (fm-81u).
//!
//! A deterministic, dependency-free triangulator for simple polygons with
//! holes — Chisel's utility-tier answer to mapbox-earcut. It exists for the
//! boolean engine's flatten fallback (§7.4), for mesh export, and for future
//! mesh work. The live fill path does **not** triangulate: §10.2 rasterizes
//! winding coverage analytically, and that independence is a design feature.
//! This module promises *validity* — an exact-cover triangulation of valid
//! input — and makes no Delaunay or mesh-quality pretensions.
//!
//! ## Input contract
//!
//! The outer boundary and each hole are rings of y-up scene-space points
//! (`f64`, §6.1) passed as `&[[f64; 2]]`. Rings may be wound either way —
//! winding is normalized internally, outer to counter-clockwise and holes to
//! clockwise — and the first point may or may not be repeated at the end (a
//! repeated closing point is dropped). Valid input is a simple outer ring
//! plus simple hole rings lying **strictly** inside it — never touching or
//! crossing its boundary — and not interfering with each other. Hole–hole
//! interference **is** validated: overlapping, touching, or nested holes are
//! refused with [`EarClipError::HoleOverlapsHole`].
//!
//! ## Output contract
//!
//! The result is a list of triangles, each three indices into the flattened
//! input order: the outer ring's vertices first (in input order), then hole
//! 0's vertices, then hole 1's, and so on. Every triangle is
//! counter-clockwise in the y-up input frame regardless of input winding.
//! For valid input the triangles exactly cover the region: their areas sum
//! to `area(outer) − Σ area(holes)`, none is flipped, and none is
//! degenerate.
//!
//! The exact-cover guarantee is enforced by **strict validity rules** on
//! every ear: no reflex-or-collinear vertex inside-or-on the candidate
//! triangle (mapbox-earcut's rule), no original boundary vertex contained
//! either, the closing diagonal crossing no original boundary edge and no
//! alive non-channel edge, and the centroid inside the material. A topology
//! those rules cannot decompose — a pathological channel configuration — is
//! a named refusal, [`EarClipError::NoValidTriangulation`], never a
//! degraded triangle. (mapbox-earcut emits signed even-odd output in those
//! cases; this module's acceptance contract is stricter.)
//!
//! ## Determinism contract
//!
//! Same input ⇒ byte-identical output, on every platform. The algorithm uses
//! only IEEE-754 `f64` add/subtract/multiply/divide and the
//! correctly-rounded `sqrt` — no transcendentals, no hashing, no unordered
//! iteration — and every choice point has a fixed tie-break:
//!
//! * a hole's bridge vertex is its rightmost vertex: greatest x, then least
//!   y, then least ring position;
//! * the bridge ray is cast from that vertex in +x; among boundary edges it
//!   crosses, the nearest intersection wins, ties broken by earliest edge in
//!   ring order (strict comparisons, first seen kept);
//! * the bridge endpoint candidate is the crossed edge's endpoint with the
//!   greater x (tie: smaller ring position), overridden by a reflex vertex
//!   lying strictly inside the bridge triangle when one exists — the nearest
//!   to the hole vertex by squared distance (tie: smallest ring position);
//! * the ear scan walks the ring from the last clip position and clips the
//!   first ear found; if a full lap finds none, the clipper splits along
//!   the first valid split diagonal found in index order
//!   (mapbox-earcut's `splitEarcut`), and only when no split exists is the
//!   named refusal returned.
//!
//! ## Degeneracy rules
//!
//! Defined behavior, never UB, on every input:
//!
//! * a non-finite coordinate (NaN, ±∞) ⇒
//!   [`EarClipError::NonFiniteCoordinate`];
//! * consecutive duplicate vertices (within tolerance) are removed, as is a
//!   repeated closing vertex; a ring left with fewer than three vertices ⇒
//!   [`EarClipError::TooFewPoints`];
//! * collinear runs are shortened: a vertex that lies *on* the segment
//!   between its neighbors (corner area within tolerance, and between them)
//!   is removed; collinear *spikes* (the vertex lies beyond a neighbor) are
//!   kept, because removing them would change the region;
//! * a ring whose signed area is zero within tolerance — including fully
//!   collinear rings ⇒ [`EarClipError::ZeroAreaRing`];
//! * a hole vertex outside the outer ring, or a hole edge properly crossing
//!   the outer boundary ⇒ [`EarClipError::HoleOutsideOuter`];
//! * a hole vertex on the outer boundary, or an outer vertex on a hole edge
//!   (within tolerance) ⇒ [`EarClipError::HoleTouchesOuter`] — the bridge
//!   seam arithmetic requires a strictly interior hole;
//! * holes that overlap, contain one another, or touch within tolerance ⇒
//!   [`EarClipError::HoleOverlapsHole`];
//! * a channel topology the strict validity rules cannot decompose ⇒
//!   [`EarClipError::NoValidTriangulation`] — subdivide the input and retry;
//! * more than [`MAX_EARCLIP_VERTICES`] flattened vertices ⇒
//!   [`EarClipError::TooManyVertices`]; a non-positive or non-finite
//!   tolerance ⇒ [`EarClipError::InvalidTolerance`].
//!
//! Zero-area sliver triangles (reachable at consumed hole-bridge seams) are
//! filtered from the output; the filter cannot remove a real triangle of
//! valid input, whose triangles all have area strictly above the
//! tolerance-scaled threshold by construction. A leftover ring reduced to a
//! consumed hole remnant or a zero-area sliver is dropped — it holds no
//! material.
//!
//! ## Complexity honesty
//!
//! Ear clipping is quadratic in the usual accounting — *n* clips, each an
//! O(*n*) scan whose O(*n*) containment test runs only for convex
//! candidates — and degrades toward O(*n*³) on adversarial rings where most
//! candidates fail late. Hole validation adds O(V_outer × V_holes). This is
//! the utility tier, not the hot path: no z-order curve, no heroic data
//! structures.

use std::error::Error;
use std::fmt;

/// Hard input-work cap for the utility-tier triangulator.
///
/// The algorithm is quadratic in ordinary cases and can approach cubic
/// work on adversarial rings. A `u32` index ceiling is therefore not a
/// meaningful resource bound. Callers with larger flattened paths must
/// partition them before triangulation.
pub const MAX_EARCLIP_VERTICES: usize = 4_096;

/// Tunables for [`triangulate_with_options`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EarClipOptions {
    /// Snap tolerance, relative to the outer ring's bounding-box diagonal.
    ///
    /// Two vertices closer than `tolerance × diagonal` are duplicates; a
    /// vertex whose corner area is below that scale is collinear; a hole
    /// feature within that distance of the outer boundary is "touching".
    /// Must be positive and finite. The default is `1e-12`.
    pub tolerance: f64,
}

impl Default for EarClipOptions {
    fn default() -> Self {
        Self { tolerance: 1e-12 }
    }
}

/// Which ring of the input a failure refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RingRole {
    /// The outer boundary (`ring_index` is always 0).
    Outer,
    /// One of the hole rings (`ring_index` is its position in `holes`).
    Hole,
}

impl fmt::Display for RingRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Outer => write!(f, "outer"),
            Self::Hole => write!(f, "hole"),
        }
    }
}

/// Failure modes of [`triangulate`] and [`triangulate_with_options`].
///
/// Every variant is a defined answer to degenerate input; no input makes the
/// triangulator panic, hang, or invoke undefined behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EarClipError {
    /// Fewer than three usable vertices remain after duplicate and collinear
    /// cleaning (or the ring arrived shorter than that).
    TooFewPoints {
        /// Outer boundary or hole.
        role: RingRole,
        /// Which ring (0 for the outer ring, the hole's position otherwise).
        ring_index: usize,
        /// How many vertices remained.
        points: usize,
    },
    /// A coordinate was NaN or infinite.
    NonFiniteCoordinate {
        /// Outer boundary or hole.
        role: RingRole,
        /// Which ring (0 for the outer ring, the hole's position otherwise).
        ring_index: usize,
        /// Which vertex in the input ring.
        vertex: usize,
    },
    /// The ring's signed area is zero within tolerance — includes fully
    /// collinear rings.
    ZeroAreaRing {
        /// Outer boundary or hole.
        role: RingRole,
        /// Which ring (0 for the outer ring, the hole's position otherwise).
        ring_index: usize,
    },
    /// A hole vertex lies outside the outer ring, or a hole edge properly
    /// crosses the outer boundary.
    HoleOutsideOuter {
        /// The hole's position in `holes`.
        hole: usize,
    },
    /// A hole vertex lies on the outer boundary, or an outer vertex lies on
    /// a hole edge (within tolerance). Hole bridging requires a strictly
    /// interior hole.
    HoleTouchesOuter {
        /// The hole's position in `holes`.
        hole: usize,
    },
    /// Two holes overlap: edges cross, one contains the other, or they
    /// touch within tolerance. Bridges are spliced sequentially, so an
    /// overlap would make the merged ring self-intersect.
    HoleOverlapsHole {
        /// The later hole's position in `holes`.
        hole: usize,
        /// The earlier hole it overlaps.
        other: usize,
    },
    /// No valid ear decomposition exists for the input's topology:
    /// the strict validity rules deadlocked. This is a named refusal,
    /// never a degraded output — it can fire on pathological inputs
    /// (deeply nested channel topologies); subdivide the input and
    /// retry.
    NoValidTriangulation,
    /// The flattened vertex count exceeds [`MAX_EARCLIP_VERTICES`].
    TooManyVertices {
        /// The total number of input vertices.
        total: usize,
    },
    /// The requested tolerance was not a positive, finite number.
    InvalidTolerance,
}

impl fmt::Display for EarClipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooFewPoints {
                role,
                ring_index,
                points,
            } => write!(
                f,
                "{role} ring {ring_index} has fewer than three usable vertices ({points})"
            ),
            Self::NonFiniteCoordinate {
                role,
                ring_index,
                vertex,
            } => write!(
                f,
                "{role} ring {ring_index} vertex {vertex} has a non-finite coordinate"
            ),
            Self::ZeroAreaRing { role, ring_index } => {
                write!(f, "{role} ring {ring_index} has zero area within tolerance")
            }
            Self::HoleOutsideOuter { hole } => {
                write!(f, "hole {hole} is not contained in the outer ring")
            }
            Self::HoleTouchesOuter { hole } => {
                write!(f, "hole {hole} touches the outer boundary")
            }
            Self::HoleOverlapsHole { hole, other } => {
                write!(f, "hole {hole} overlaps hole {other}")
            }
            Self::NoValidTriangulation => write!(
                f,
                "no valid triangulation exists under the strict validity rules \
                 (a pathological channel topology — subdivide the input and retry)"
            ),
            Self::TooManyVertices { total } => {
                write!(
                    f,
                    "flattened vertex count {total} exceeds the declared cap {MAX_EARCLIP_VERTICES}"
                )
            }
            Self::InvalidTolerance => {
                write!(f, "tolerance must be a positive, finite number")
            }
        }
    }
}

impl Error for EarClipError {}

/// Triangulate `outer` with `holes` using the default options.
///
/// See the module documentation for the input/output, determinism, and
/// degeneracy contracts.
///
/// # Errors
///
/// Returns [`EarClipError`] for the degenerate inputs enumerated in the
/// module documentation.
pub fn triangulate(
    outer: &[[f64; 2]],
    holes: &[&[[f64; 2]]],
) -> Result<Vec<[u32; 3]>, EarClipError> {
    triangulate_with_options(outer, holes, &EarClipOptions::default())
}

/// Triangulate `outer` with `holes` under explicit `options`.
///
/// # Errors
///
/// Returns [`EarClipError`] for the degenerate inputs enumerated in the
/// module documentation, plus [`EarClipError::InvalidTolerance`] for a bad
/// `options.tolerance`.
pub fn triangulate_with_options(
    outer: &[[f64; 2]],
    holes: &[&[[f64; 2]]],
    options: &EarClipOptions,
) -> Result<Vec<[u32; 3]>, EarClipError> {
    if !options.tolerance.is_finite() || options.tolerance <= 0.0 {
        return Err(EarClipError::InvalidTolerance);
    }
    let mut total = outer.len();
    for hole in holes {
        total = total
            .checked_add(hole.len())
            .ok_or(EarClipError::TooManyVertices { total: usize::MAX })?;
    }
    if total > MAX_EARCLIP_VERTICES {
        return Err(EarClipError::TooManyVertices { total });
    }
    check_finite(outer, RingRole::Outer, 0)?;
    for (index, hole) in holes.iter().enumerate() {
        check_finite(hole, RingRole::Hole, index)?;
    }

    // All topology predicates run in one deterministic affine space.
    // This preserves orientation, containment, flattened indices, and
    // relative tolerance semantics while preventing absolute scene
    // offsets/scales from overflowing or cancelling the predicates.
    let PredicateRings {
        outer: predicate_outer,
        holes: predicate_holes,
    } = normalize_predicate_space(outer, holes)?;
    let tol_abs = options.tolerance * bbox_diagonal(&predicate_outer);
    if !tol_abs.is_finite() {
        return Err(EarClipError::InvalidTolerance);
    }
    let cleaned_outer = clean_ring(&predicate_outer, 0, RingRole::Outer, 0, tol_abs, true)?;
    let mut cleaned_holes = Vec::with_capacity(holes.len());
    let mut base = outer.len();
    for (index, hole) in predicate_holes.iter().enumerate() {
        let cleaned = clean_ring(hole, base, RingRole::Hole, index, tol_abs, false)?;
        check_hole(&cleaned_outer, &cleaned, tol_abs, index)?;
        for (other_index, other) in cleaned_holes.iter().enumerate() {
            check_holes_disjoint(&cleaned, other, tol_abs, index, other_index)?;
        }
        cleaned_holes.push(cleaned);
        base = base
            .checked_add(holes[index].len())
            .ok_or(EarClipError::TooManyVertices { total: usize::MAX })?;
    }

    let mut points = cleaned_outer.points.clone();
    let mut flat = cleaned_outer.flat.clone();
    // Slit-edge flags, parallel to `points`: `slit_from[i]` marks the
    // edge `i -> next[i]` as part of the artificial channel cut a hole
    // bridge made (versus a real boundary edge). Bridge splices mark
    // their two seam edges; ear clipping propagates flags through
    // merges. The diagonal test ignores crossings with slit edges —
    // crossing the zero-width channel is harmless.
    let mut slit_from = vec![false; points.len()];
    let material = MaterialBoundary {
        outer: &cleaned_outer.points,
        holes: &cleaned_holes,
    };
    for (index, hole) in cleaned_holes.iter().enumerate() {
        bridge_hole(&mut points, &mut flat, &mut slit_from, hole, index)?;
    }
    clip_ears(&points, &flat, &slit_from, &material, tol_abs * tol_abs)
}

/// Twice the signed area of a triangle corner: `(b − a) × (c − a)`.
#[inline]
fn cross(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

/// Squared distance between two points.
#[inline]
fn dist2(a: [f64; 2], b: [f64; 2]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    dx * dx + dy * dy
}

/// Overflow-safe midpoint. Predicate-space coordinates are already
/// bounded, but keeping the primitive stable prevents future callers
/// from reintroducing same-sign overflow.
fn midpoint(a: f64, b: f64) -> f64 {
    a + (b - a) * 0.5
}

fn centroid3(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> [f64; 2] {
    [
        a[0] + (b[0] - a[0]) / 3.0 + (c[0] - a[0]) / 3.0,
        a[1] + (b[1] - a[1]) / 3.0 + (c[1] - a[1]) / 3.0,
    ]
}

/// Signed area of a ring (positive for counter-clockwise winding).
fn signed_area(points: &[[f64; 2]]) -> f64 {
    let n = points.len();
    if n < 3 {
        return 0.0;
    }
    let origin = points[0];
    let mut acc = 0.0;
    for (i, &p) in points.iter().enumerate() {
        let q = points[(i + 1) % n];
        acc += cross(origin, p, q);
    }
    0.5 * acc
}

/// Bounding-box diagonal of a ring (0 for an empty ring).
fn bbox_diagonal(ring: &[[f64; 2]]) -> f64 {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for p in ring {
        min_x = min_x.min(p[0]);
        min_y = min_y.min(p[1]);
        max_x = max_x.max(p[0]);
        max_y = max_y.max(p[1]);
    }
    if ring.is_empty() {
        return 0.0;
    }
    let dx = max_x - min_x;
    let dy = max_y - min_y;
    (dx * dx + dy * dy).sqrt()
}

fn check_finite(ring: &[[f64; 2]], role: RingRole, ring_index: usize) -> Result<(), EarClipError> {
    for (vertex, p) in ring.iter().enumerate() {
        if !p[0].is_finite() || !p[1].is_finite() {
            return Err(EarClipError::NonFiniteCoordinate {
                role,
                ring_index,
                vertex,
            });
        }
    }
    Ok(())
}

/// Map every ring through one positive affine transform into a bounded
/// predicate space.
///
/// Dividing by the largest absolute input component first avoids
/// overflow even when finite coordinates straddle the full f64 range.
/// The second pass recentres and scales the outer bounds. Every hole
/// uses that same transform, so orientation, containment, intersections,
/// and the tolerance relative to the outer diagonal are preserved.
struct PredicateRings {
    outer: Vec<[f64; 2]>,
    holes: Vec<Vec<[f64; 2]>>,
}

fn normalize_predicate_space(
    outer: &[[f64; 2]],
    holes: &[&[[f64; 2]]],
) -> Result<PredicateRings, EarClipError> {
    let mut magnitude = 0.0_f64;
    for point in outer {
        magnitude = magnitude.max(point[0].abs()).max(point[1].abs());
    }
    if magnitude == 0.0 {
        magnitude = 1.0;
    }

    let mut min = [f64::INFINITY; 2];
    let mut max = [f64::NEG_INFINITY; 2];
    for point in outer {
        let scaled = [point[0] / magnitude, point[1] / magnitude];
        for axis in 0..2 {
            min[axis] = min[axis].min(scaled[axis]);
            max[axis] = max[axis].max(scaled[axis]);
        }
    }
    let has_points = min[0].is_finite();
    let center = if has_points {
        [midpoint(min[0], max[0]), midpoint(min[1], max[1])]
    } else {
        [0.0; 2]
    };
    let mut span = if has_points {
        (max[0] - min[0]).max(max[1] - min[1])
    } else {
        1.0
    };
    if span == 0.0 {
        span = 1.0;
    }

    let normalize = |point: [f64; 2]| {
        [
            (point[0] / magnitude - center[0]) / span,
            (point[1] / magnitude - center[1]) / span,
        ]
    };
    let predicate_outer = outer.iter().copied().map(normalize).collect();
    let mut predicate_holes = Vec::with_capacity(holes.len());
    for (hole, ring) in holes.iter().enumerate() {
        let mut predicate_ring = Vec::with_capacity(ring.len());
        for &point in *ring {
            let normalized = normalize(point);
            if normalized.iter().any(|component| !component.is_finite()) {
                return Err(EarClipError::HoleOutsideOuter { hole });
            }
            predicate_ring.push(normalized);
        }
        predicate_holes.push(predicate_ring);
    }
    Ok(PredicateRings {
        outer: predicate_outer,
        holes: predicate_holes,
    })
}

/// A ring after cleaning and winding normalization.
struct CleanRing {
    /// Vertices in normalized winding (outer CCW, holes CW).
    points: Vec<[f64; 2]>,
    /// `flat[i]` is the index of `points[i]` in the flattened input order.
    flat: Vec<u32>,
}

/// Remove consecutive duplicates, collinear runs, and normalize winding.
///
/// `base` is the ring's starting index in the flattened input order.
/// `want_ccw` selects the normalized winding.
fn clean_ring(
    ring: &[[f64; 2]],
    base: usize,
    role: RingRole,
    ring_index: usize,
    tol_abs: f64,
    want_ccw: bool,
) -> Result<CleanRing, EarClipError> {
    let tol2 = tol_abs * tol_abs;
    let mut points: Vec<[f64; 2]> = Vec::with_capacity(ring.len());
    let mut flat: Vec<u32> = Vec::with_capacity(ring.len());
    for (i, &p) in ring.iter().enumerate() {
        let duplicate = points.last().is_some_and(|&q| dist2(p, q) <= tol2);
        if !duplicate {
            points.push(p);
            let flat_index = base
                .checked_add(i)
                .and_then(|index| u32::try_from(index).ok())
                .ok_or(EarClipError::TooManyVertices {
                    total: base.saturating_add(ring.len()),
                })?;
            flat.push(flat_index);
        }
    }
    // A repeated closing vertex is a duplicate too.
    while points.len() > 1 {
        let first = points[0];
        let last = points[points.len() - 1];
        if dist2(first, last) <= tol2 {
            points.pop();
            flat.pop();
        } else {
            break;
        }
    }
    if points.len() < 3 {
        return Err(EarClipError::TooFewPoints {
            role,
            ring_index,
            points: points.len(),
        });
    }
    let area = signed_area(&points);
    if area.abs() <= tol2 {
        return Err(EarClipError::ZeroAreaRing { role, ring_index });
    }
    remove_collinear(&mut points, &mut flat, tol_abs);
    if points.len() < 3 {
        return Err(EarClipError::TooFewPoints {
            role,
            ring_index,
            points: points.len(),
        });
    }
    if (area > 0.0) != want_ccw {
        points.reverse();
        flat.reverse();
    }
    Ok(CleanRing { points, flat })
}

/// Remove vertices that lie on the segment between their neighbors, to a
/// fixed point. Spikes (collinear but not between) are kept.
fn remove_collinear(points: &mut Vec<[f64; 2]>, flat: &mut Vec<u32>, tol_abs: f64) {
    loop {
        if points.len() < 3 {
            return;
        }
        let mut removed = false;
        let mut i = 0;
        while i < points.len() {
            let n = points.len();
            let a = points[(i + n - 1) % n];
            let v = points[i];
            let b = points[(i + 1) % n];
            let e1 = dist2(a, v).sqrt();
            let e2 = dist2(v, b).sqrt();
            let corner = cross(a, v, b).abs();
            let between = (v[0] - a[0]) * (b[0] - v[0]) + (v[1] - a[1]) * (b[1] - v[1]) >= 0.0;
            if corner <= tol_abs * e1.max(e2) && between {
                points.remove(i);
                flat.remove(i);
                removed = true;
            } else {
                i += 1;
            }
        }
        if !removed {
            return;
        }
    }
}

/// Is `p` strictly inside `ring` (even-odd, boundary excluded)?
fn strictly_inside_ring(ring: &[[f64; 2]], p: [f64; 2]) -> bool {
    let n = ring.len();
    let mut inside = false;
    for (i, &a) in ring.iter().enumerate() {
        let b = ring[(i + 1) % n];
        if (a[1] > p[1]) != (b[1] > p[1]) {
            let x = a[0] + (p[1] - a[1]) * (b[0] - a[0]) / (b[1] - a[1]);
            if x > p[0] {
                inside = !inside;
            }
        }
    }
    inside
}

/// Is `p` within `tol_abs` of segment `a`–`b` (perpendicular and along)?
fn point_near_segment(p: [f64; 2], a: [f64; 2], b: [f64; 2], tol_abs: f64) -> bool {
    let ex = b[0] - a[0];
    let ey = b[1] - a[1];
    let len2 = ex * ex + ey * ey;
    let len = len2.sqrt();
    if cross(a, b, p).abs() > tol_abs * len {
        return false;
    }
    let along = (p[0] - a[0]) * ex + (p[1] - a[1]) * ey;
    along >= -tol_abs * len && along <= len2 + tol_abs * len
}

/// Do the open segments `p1`–`p2` and `q1`–`q2` properly cross?
///
/// Endpoint contact and collinear overlap are *not* crossings; those are
/// caught by the tolerance-based near checks instead.
fn segments_cross(p1: [f64; 2], p2: [f64; 2], q1: [f64; 2], q2: [f64; 2]) -> bool {
    let d1 = cross(q1, q2, p1);
    let d2 = cross(q1, q2, p2);
    let d3 = cross(p1, p2, q1);
    let d4 = cross(p1, p2, q2);
    ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0))
        && ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0))
}

/// Validate that `hole` lies strictly inside `outer`.
fn check_hole(
    outer: &CleanRing,
    hole: &CleanRing,
    tol_abs: f64,
    hole_index: usize,
) -> Result<(), EarClipError> {
    for &p in &hole.points {
        let near = outer.points.iter().enumerate().any(|(i, &a)| {
            point_near_segment(p, a, outer.points[(i + 1) % outer.points.len()], tol_abs)
        });
        if near {
            return Err(EarClipError::HoleTouchesOuter { hole: hole_index });
        }
        if !strictly_inside_ring(&outer.points, p) {
            return Err(EarClipError::HoleOutsideOuter { hole: hole_index });
        }
    }
    let hn = hole.points.len();
    let on = outer.points.len();
    for i in 0..hn {
        let h1 = hole.points[i];
        let h2 = hole.points[(i + 1) % hn];
        for j in 0..on {
            let o1 = outer.points[j];
            let o2 = outer.points[(j + 1) % on];
            if segments_cross(h1, h2, o1, o2) {
                return Err(EarClipError::HoleOutsideOuter { hole: hole_index });
            }
            if point_near_segment(o1, h1, h2, tol_abs) {
                return Err(EarClipError::HoleTouchesOuter { hole: hole_index });
            }
        }
    }
    Ok(())
}

/// Two holes must be disjoint: no edge crossing, no containment either
/// way, and no near-touch within tolerance (bridges splice sequentially,
/// so an overlap would make the merged ring self-intersect).
fn check_holes_disjoint(
    hole: &CleanRing,
    other: &CleanRing,
    tol_abs: f64,
    hole_index: usize,
    other_index: usize,
) -> Result<(), EarClipError> {
    let overlap = || EarClipError::HoleOverlapsHole {
        hole: hole_index,
        other: other_index,
    };
    for &p in &hole.points {
        if strictly_inside_ring(&other.points, p) {
            return Err(overlap());
        }
    }
    for &p in &other.points {
        if strictly_inside_ring(&hole.points, p) {
            return Err(overlap());
        }
    }
    let an = hole.points.len();
    let bn = other.points.len();
    for i in 0..an {
        let a1 = hole.points[i];
        let a2 = hole.points[(i + 1) % an];
        for j in 0..bn {
            let b1 = other.points[j];
            let b2 = other.points[(j + 1) % bn];
            if segments_cross(a1, a2, b1, b2) {
                return Err(overlap());
            }
            if point_near_segment(b1, a1, a2, tol_abs) || point_near_segment(a1, b1, b2, tol_abs) {
                return Err(overlap());
            }
        }
    }
    Ok(())
}

/// The hole's bridge vertex: greatest x, then least y, then least position.
fn rightmost(points: &[[f64; 2]]) -> usize {
    let mut best = 0;
    for (i, p) in points.iter().enumerate().skip(1) {
        let b = points[best];
        if p[0] > b[0] || (p[0] == b[0] && p[1] < b[1]) {
            best = i;
        }
    }
    best
}

/// Is `p` strictly inside triangle `a b c` (boundary excluded)?
fn strictly_inside_triangle(p: [f64; 2], a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> bool {
    let area = cross(a, b, c);
    if area > 0.0 {
        cross(a, b, p) > 0.0 && cross(b, c, p) > 0.0 && cross(c, a, p) > 0.0
    } else if area < 0.0 {
        cross(a, b, p) < 0.0 && cross(b, c, p) < 0.0 && cross(c, a, p) < 0.0
    } else {
        false
    }
}

/// Splice `hole` into the merged ring with an earcut-style visible-vertex
/// bridge (Eberly's rightmost-vertex ray cast with the reflex override).
fn bridge_hole(
    points: &mut Vec<[f64; 2]>,
    flat: &mut Vec<u32>,
    slit_from: &mut Vec<bool>,
    hole: &CleanRing,
    hole_index: usize,
) -> Result<(), EarClipError> {
    let m = rightmost(&hole.points);
    let h = hole.points[m];
    let n = points.len();

    // Nearest edge crossing the ray h + t·(1, 0), t ≥ 0; ties keep the
    // earliest edge in ring order.
    let mut best_edge = usize::MAX;
    let mut best_x = f64::INFINITY;
    for e in 0..n {
        let p = points[e];
        let q = points[(e + 1) % n];
        if (p[1] > h[1]) != (q[1] > h[1]) {
            let x = p[0] + (h[1] - p[1]) * (q[0] - p[0]) / (q[1] - p[1]);
            if x >= h[0] && x < best_x {
                best_x = x;
                best_edge = e;
            }
        }
    }
    if best_edge == usize::MAX {
        // The containment validation passed, so a miss means the merged
        // boundary was already compromised by invalid input.
        return Err(EarClipError::HoleOutsideOuter { hole: hole_index });
    }
    let e2 = (best_edge + 1) % n;
    let p = points[best_edge];
    let q = points[e2];
    let intersection = [best_x, h[1]];

    // Endpoint candidate: the crossed edge's endpoint with the greater x
    // (tie: smaller ring position), or the endpoint the ray hit exactly.
    let mut bridge = if intersection == p {
        best_edge
    } else if intersection == q {
        e2
    } else if p[0] > q[0] {
        best_edge
    } else if q[0] > p[0] {
        e2
    } else {
        best_edge.min(e2)
    };

    // Reflex override: the reflex vertex strictly inside the bridge triangle
    // nearest to the hole vertex (tie: smallest ring position).
    let target = points[bridge];
    let mut best_d2 = f64::INFINITY;
    for i in 0..n {
        let v = points[i];
        let a = points[(i + n - 1) % n];
        let b = points[(i + 1) % n];
        if cross(a, v, b) < 0.0 && strictly_inside_triangle(v, h, intersection, target) {
            let d2 = dist2(v, h);
            if d2 < best_d2 {
                best_d2 = d2;
                bridge = i;
            }
        }
    }

    // Splice: outer up to and including the bridge vertex, the hole from its
    // bridge vertex all the way around and back to it, then the rest of the
    // outer ring starting at the bridge vertex again. The two seam edges —
    // bridge -> hole and hole -> bridge — are the artificial channel cut;
    // flag them so the ear test's diagonal rule can ignore crossings with
    // them (the channel has zero width; crossing it is harmless).
    let hole_len = hole.points.len();
    let new_len = n + hole_len + 2;
    let mut new_points = Vec::with_capacity(new_len);
    let mut new_flat = Vec::with_capacity(new_len);
    let mut new_slit = vec![false; new_len];
    // Outer head (through the bridge vertex); the edge from `bridge`
    // becomes the seam out to the hole.
    for k in 0..=bridge {
        new_points.push(points[k]);
        new_flat.push(flat[k]);
        new_slit[k] = slit_from[k];
    }
    new_slit[bridge] = true;
    // The hole loop from `m` all the way around and back to `m` (real
    // boundary edges, unflagged), ...
    for k in 0..=hole_len {
        let p = hole.points[(m + k) % hole_len];
        let f = hole.flat[(m + k) % hole_len];
        new_points.push(p);
        new_flat.push(f);
    }
    // ... then the outer tail starting at the bridge vertex again (the
    // seam-duplicate). The edge from the hole loop's last vertex to the
    // tail is the seam back.
    new_slit[bridge + hole_len + 1] = true;
    for k in bridge..n {
        new_points.push(points[k]);
        new_flat.push(flat[k]);
        new_slit[k + hole_len + 2] = slit_from[k];
    }
    debug_assert_eq!(new_points.len(), new_len);
    debug_assert_eq!(new_flat.len(), new_len);
    *points = new_points;
    *flat = new_flat;
    *slit_from = new_slit;
    Ok(())
}

/// Is `p` inside or on triangle `a b c` (boundary included)?
fn inside_or_on_triangle(p: [f64; 2], a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> bool {
    let area = cross(a, b, c);
    if area > 0.0 {
        cross(a, b, p) >= 0.0 && cross(b, c, p) >= 0.0 && cross(c, a, p) >= 0.0
    } else if area < 0.0 {
        cross(a, b, p) <= 0.0 && cross(b, c, p) <= 0.0 && cross(c, a, p) <= 0.0
    } else {
        false
    }
}

/// The original material boundary: the cleaned outer ring and holes.
/// The airtight ghost rule needs it — the current ring's even-odd
/// interior goes blind over consumed channel regions, so ear triangles
/// are checked against the ORIGINAL boundary instead.
struct MaterialBoundary<'a> {
    outer: &'a [[f64; 2]],
    holes: &'a [CleanRing],
}

/// Is alive vertex `v` an ear of the merged ring?
///
/// The validity rule, completed for sewn rings. A candidate ear is
/// valid iff (1) it is convex; (2) no other alive reflex-or-collinear
/// vertex lies inside or on its triangle (mapbox-earcut's rule, which
/// strict containment alone gets wrong at T-junctions); (3) its closing
/// diagonal crosses no alive **real** edge and no **original hole
/// edge** — slit edges (the zero-width bridge channels) are ignored;
/// and (4) its centroid lies inside the original outer ring and outside
/// every original hole. (3)+(4) together are airtight: a triangle with
/// no boundary crossing and one interior point is inside the material,
/// which is exactly the ghost case the alive-vertex rules cannot see
/// (already-consumed regions look alive-geometric to a spanning ear).
fn is_ear(
    v: usize,
    points: &[[f64; 2]],
    prev: &[usize],
    next: &[usize],
    alive: &[bool],
    slit_from: &[bool],
    material: &MaterialBoundary,
) -> bool {
    let a = prev[v];
    let b = next[v];
    let pa = points[a];
    let pv = points[v];
    let pb = points[b];
    if cross(pa, pv, pb) <= 0.0 {
        return false;
    }
    for w in 0..points.len() {
        if !alive[w] || w == a || w == v || w == b {
            continue;
        }
        // Positional duplicates of the ear's own vertices (hole-bridge
        // seam copies) are part of the ear itself — the
        // boundary-inclusive containment below would otherwise count a
        // seam copy sitting exactly on the ear's corner.
        if points[w] == pa || points[w] == pv || points[w] == pb {
            continue;
        }
        let reflex_or_collinear = cross(points[prev[w]], points[w], points[next[w]]) <= 0.0;
        if reflex_or_collinear && inside_or_on_triangle(points[w], pa, pv, pb) {
            return false;
        }
    }
    // The closing diagonal must not cross the ring's REAL boundary.
    // Vertex containment alone misses a ring edge clipping the
    // triangle's corner with all vertices outside (a hole's edge
    // slicing through the ear); with (a, v) and (v, b) inside, an
    // uncrossed diagonal puts the whole triangle inside. Slit edges —
    // the artificial channel cuts the hole bridges made — are ignored:
    // the channel has zero width, so crossing it is harmless. Proper
    // crossings only: endpoint contact and collinear overlap are handled
    // by the vertex rule above.
    for w in 0..points.len() {
        if !alive[w] || slit_from[w] {
            continue;
        }
        let w2 = next[w];
        if w == a || w == b || w2 == a || w2 == b {
            continue;
        }
        if segments_cross(pa, pb, points[w], points[w2]) {
            return false;
        }
    }
    // (2b) The triangle contains no ORIGINAL boundary vertex (position-
    // distinct from the ear's own corners, which hole-bridge copies may
    // coincide with). A triangle can swallow a whole region without
    // crossing its edges — vertex containment sees only the alive ring.
    for hole in material.holes {
        for &w in &hole.points {
            if w == pa || w == pv || w == pb {
                continue;
            }
            if inside_or_on_triangle(w, pa, pv, pb) {
                return false;
            }
        }
    }
    for &w in material.outer {
        if w == pa || w == pv || w == pb {
            continue;
        }
        if inside_or_on_triangle(w, pa, pv, pb) {
            return false;
        }
    }
    // (3b) The diagonal crosses no ORIGINAL boundary edge — neither a
    // hole's nor the outer's. Consumed boundary edges leave no alive
    // edges behind, so rule (3a) alone cannot see a diagonal cutting
    // through an already-tiled boundary region.
    for hole in material.holes {
        let hn = hole.points.len();
        for w in 0..hn {
            if segments_cross(pa, pb, hole.points[w], hole.points[(w + 1) % hn]) {
                return false;
            }
        }
    }
    let on = material.outer.len();
    for w in 0..on {
        if segments_cross(pa, pb, material.outer[w], material.outer[(w + 1) % on]) {
            return false;
        }
    }
    // (4) The airtight ghost rule: the centroid must lie inside the
    // original material — the outer ring, minus every hole.
    let centroid = centroid3(pa, pv, pb);
    if !strictly_inside_ring(material.outer, centroid) {
        return false;
    }
    for hole in material.holes {
        if strictly_inside_ring(&hole.points, centroid) {
            return false;
        }
    }
    true
}

/// Emit triangle `(a, b, c)` unless it is a zero-area sliver.
fn push_triangle(
    out: &mut Vec<[u32; 3]>,
    points: &[[f64; 2]],
    flat: &[u32],
    a: usize,
    b: usize,
    c: usize,
    sliver_eps: f64,
) {
    if cross(points[a], points[b], points[c]).abs() > sliver_eps {
        out.push([flat[a], flat[b], flat[c]]);
    }
}

/// Is the diagonal `(a, b)` inside the ring at `a`'s corner?
/// (mapbox-earcut's `locallyInside`, sign conventions ported verbatim.)
fn locally_inside(a: usize, b: usize, points: &[[f64; 2]], prev: &[usize], next: &[usize]) -> bool {
    let pa = points[prev[a]];
    let va = points[a];
    let na = points[next[a]];
    let pb = points[b];
    if cross(pa, va, na) < 0.0 {
        cross(va, pb, na) >= 0.0 && cross(va, pa, pb) >= 0.0
    } else {
        cross(va, pb, pa) < 0.0 || cross(va, na, pb) < 0.0
    }
}

/// A valid split diagonal (mapbox-earcut's `isValidDiagonal`): not
/// adjacent, locally inside at both ends, midpoint inside the ring, and
/// crossing no **real** (non-slit) ring edge. Crossing slit edges — the
/// zero-width channels the hole bridges cut — is harmless: the split
/// partitions the material either way.
struct SplitCheck<'a> {
    points: &'a [[f64; 2]],
    prev: &'a [usize],
    next: &'a [usize],
    alive: &'a [bool],
    slit_from: &'a [bool],
    ring: &'a [[f64; 2]],
}

fn is_valid_split(a: usize, b: usize, check: &SplitCheck) -> bool {
    let SplitCheck {
        points,
        prev,
        next,
        alive,
        slit_from,
        ring,
    } = *check;
    if a == b || next[a] == b || next[b] == a {
        return false;
    }
    if !locally_inside(a, b, points, prev, next) || !locally_inside(b, a, points, prev, next) {
        return false;
    }
    let mid = [
        midpoint(points[a][0], points[b][0]),
        midpoint(points[a][1], points[b][1]),
    ];
    if !strictly_inside_ring(ring, mid) {
        return false;
    }
    for w in 0..points.len() {
        if !alive[w] || slit_from[w] {
            continue;
        }
        let w2 = next[w];
        if w == a || w == b || w2 == a || w2 == b {
            continue;
        }
        if segments_cross(points[a], points[b], points[w], points[w2]) {
            return false;
        }
    }
    true
}

/// The earless escape (mapbox-earcut's `splitEarcut`): find a valid
/// split diagonal among alive vertices, in deterministic index order.
fn find_split(
    points: &[[f64; 2]],
    prev: &[usize],
    next: &[usize],
    alive: &[bool],
    slit_from: &[bool],
) -> Option<(usize, usize)> {
    let ring: Vec<[f64; 2]> = (0..points.len())
        .filter(|&i| alive[i])
        .map(|i| points[i])
        .collect();
    for a in 0..points.len() {
        if !alive[a] {
            continue;
        }
        for b in 0..points.len() {
            if !alive[b] || b == a {
                continue;
            }
            if is_valid_split(
                a,
                b,
                &SplitCheck {
                    points,
                    prev,
                    next,
                    alive,
                    slit_from,
                    ring: &ring,
                },
            ) {
                return Some((a, b));
            }
        }
    }
    None
}

/// Clip ears off the merged ring until one triangle remains. On an
/// earless ring (reachable through sewn seams) the clipper splits along
/// a valid diagonal and recurses — the mapbox-earcut architecture;
/// `most_convex` remains only for genuinely invalid input. A ring whose
/// every vertex lies inside or on an original hole is a consumed hole
/// remnant: it contains no material and is dropped.
fn clip_ears(
    points: &[[f64; 2]],
    flat: &[u32],
    slit_from: &[bool],
    material: &MaterialBoundary,
    sliver_eps: f64,
) -> Result<Vec<[u32; 3]>, EarClipError> {
    clip_ears_at(points, flat, slit_from, material, sliver_eps, 0)
}

/// Does `p` lie inside `hole` or coincide with one of its vertices?
fn inside_or_on_hole_vertex(hole: &CleanRing, p: [f64; 2]) -> bool {
    hole.points.contains(&p) || strictly_inside_ring(&hole.points, p)
}

#[allow(clippy::too_many_lines)]
fn clip_ears_at(
    points: &[[f64; 2]],
    flat: &[u32],
    slit_from: &[bool],
    material: &MaterialBoundary,
    sliver_eps: f64,
    depth: u32,
) -> Result<Vec<[u32; 3]>, EarClipError> {
    let n = points.len();
    let ring_area = |alive: &[bool], next: &[usize], start: usize, count: usize| {
        let mut s = 0.0;
        let mut i = start;
        let origin = points[start];
        for _ in 0..count.max(1) {
            let j = if alive.is_empty() {
                (i + 1) % n.max(1)
            } else {
                next[i]
            };
            s += cross(origin, points[i], points[j]);
            i = j;
        }
        s / 2.0
    };
    // A remnant holding only hole interiors contains no material; a
    // zero-area remnant (a degenerate out-and-back sliver) is already
    // fully tiled by the sliver rule.
    let initial_area = ring_area(&[], &[], 0, n).abs();
    if initial_area <= sliver_eps {
        return Ok(Vec::new());
    }
    if !material.holes.is_empty()
        && points.iter().all(|&p| {
            material
                .holes
                .iter()
                .any(|h| inside_or_on_hole_vertex(h, p))
        })
    {
        return Ok(Vec::new());
    }
    let mut prev: Vec<usize> = (0..n).map(|i| (i + n - 1) % n).collect();
    let mut next: Vec<usize> = (0..n).map(|i| (i + 1) % n).collect();
    let mut alive = vec![true; n];
    let mut slit_from = slit_from.to_vec();
    let mut remaining = n;
    let mut out = Vec::with_capacity(n.saturating_sub(2));
    let mut cursor = 0;

    while remaining > 3 {
        let mut ear = usize::MAX;
        let mut v = cursor;
        for _ in 0..remaining {
            if is_ear(v, points, &prev, &next, &alive, &slit_from, material) {
                ear = v;
                break;
            }
            v = next[v];
        }
        if ear == usize::MAX {
            // A remnant holding only hole interiors contains no
            // material; a zero-area remnant is already fully tiled.
            let alive_all_in_holes = !material.holes.is_empty()
                && (0..points.len()).filter(|&i| alive[i]).all(|i| {
                    material
                        .holes
                        .iter()
                        .any(|h| inside_or_on_hole_vertex(h, points[i]))
                });
            if alive_all_in_holes || ring_area(&alive, &next, cursor, remaining).abs() <= sliver_eps
            {
                return Ok(out);
            }
            if depth < 32
                && let Some((a, b)) = find_split(points, &prev, &next, &alive, &slit_from)
            {
                // Materialize the alive ring, split at (a, b) into two
                // sub-rings, and recurse: each half is a simple polygon
                // whose triangulation tiles its share of the material.
                let mut seq: Vec<([f64; 2], u32, bool)> = Vec::with_capacity(remaining);
                let mut i = cursor;
                for _ in 0..remaining {
                    seq.push((points[i], flat[i], slit_from[i]));
                    i = next[i];
                }
                let pos_a = seq
                    .iter()
                    .position(|s| s.0 == points[a] && s.1 == flat[a])
                    .ok_or(EarClipError::NoValidTriangulation)?;
                let pos_b = seq
                    .iter()
                    .position(|s| s.0 == points[b] && s.1 == flat[b])
                    .ok_or(EarClipError::NoValidTriangulation)?;
                let (lo, hi) = if pos_a <= pos_b {
                    (pos_a, pos_b)
                } else {
                    (pos_b, pos_a)
                };
                let mut ring1 = seq[lo..=hi].to_vec();
                let mut ring2: Vec<([f64; 2], u32, bool)> = seq[hi..].to_vec();
                ring2.extend_from_slice(&seq[..=lo]);
                // The shared diagonal edges are real boundaries of the
                // sub-rings, not channel: unflag them at the seam
                // positions of both halves.
                if let Some(last) = ring1.last_mut() {
                    last.2 = false;
                }
                if let Some(last) = ring2.last_mut() {
                    last.2 = false;
                }
                let unpack =
                    |ring: Vec<([f64; 2], u32, bool)>| -> (Vec<[f64; 2]>, Vec<u32>, Vec<bool>) {
                        let mut p = Vec::with_capacity(ring.len());
                        let mut f = Vec::with_capacity(ring.len());
                        let mut s = Vec::with_capacity(ring.len());
                        for (pt, fl, sl) in ring {
                            p.push(pt);
                            f.push(fl);
                            s.push(sl);
                        }
                        (p, f, s)
                    };
                if ring1.len() >= 3 {
                    let (p, f, s) = unpack(ring1);
                    out.extend(clip_ears_at(&p, &f, &s, material, sliver_eps, depth + 1)?);
                }
                if ring2.len() >= 3 {
                    let (p, f, s) = unpack(ring2);
                    out.extend(clip_ears_at(&p, &f, &s, material, sliver_eps, depth + 1)?);
                }
                return Ok(out);
            }
            // The strict validity rules deadlocked. A named refusal —
            // never a degraded triangle.
            return Err(EarClipError::NoValidTriangulation);
        }
        let a = prev[ear];
        let b = next[ear];
        push_triangle(&mut out, points, flat, a, ear, b, sliver_eps);
        // The merged edge (a -> b) lies in the channel only if the two
        // edges it replaces were both channel edges.
        slit_from[a] = slit_from[a] && slit_from[ear];
        next[a] = b;
        prev[b] = a;
        alive[ear] = false;
        remaining -= 1;
        cursor = b;
    }
    let a = cursor;
    let b = next[a];
    let c = next[b];
    push_triangle(&mut out, points, flat, a, b, c, sliver_eps);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// xorshift64* — small deterministic PRNG, no external crates.
    struct XorShift(u64);

    impl XorShift {
        fn next_u64(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x.wrapping_mul(0x2545_F491_4F6C_DD1D)
        }

        /// Uniform in `[0, 1)`.
        fn next_f64(&mut self) -> f64 {
            (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
        }

        fn range(&mut self, lo: f64, hi: f64) -> f64 {
            lo + (hi - lo) * self.next_f64()
        }

        fn usize_below(&mut self, n: usize) -> usize {
            (self.next_u64() % n as u64) as usize
        }
    }

    fn flatten(outer: &[[f64; 2]], holes: &[Vec<[f64; 2]>]) -> Vec<[f64; 2]> {
        let mut v: Vec<[f64; 2]> = outer.to_vec();
        for h in holes {
            v.extend_from_slice(h);
        }
        v
    }

    fn hole_refs(holes: &[Vec<[f64; 2]>]) -> Vec<&[[f64; 2]]> {
        holes.iter().map(Vec::as_slice).collect()
    }

    /// A star with `spikes` tips: simple and star-shaped about its center,
    /// CCW. Angle jitter stays under half the angular step so the vertex
    /// order (and hence simplicity) is guaranteed.
    fn random_star(rng: &mut XorShift, spikes: usize) -> Vec<[f64; 2]> {
        let tau = std::f64::consts::TAU;
        let phase = rng.range(0.0, tau);
        let cx = rng.range(-5.0, 5.0);
        let cy = rng.range(-5.0, 5.0);
        let r_out = rng.range(1.0, 4.0);
        let mut pts = Vec::with_capacity(2 * spikes);
        for k in 0..(2 * spikes) {
            let step = tau / (2.0 * spikes as f64);
            let ang = phase + step * k as f64 + rng.range(-0.3 * step, 0.3 * step);
            let r = if k % 2 == 0 {
                r_out * rng.range(0.8, 1.2)
            } else {
                r_out * rng.range(0.25, 0.6)
            };
            pts.push([cx + r * ang.cos(), cy + r * ang.sin()]);
        }
        pts
    }

    /// Scaled copy of `ring` about `center`; strictly inside for a
    /// star-shaped ring and `s < 1`.
    fn scaled_copy(ring: &[[f64; 2]], center: [f64; 2], s: f64) -> Vec<[f64; 2]> {
        ring.iter()
            .map(|p| {
                [
                    center[0] + s * (p[0] - center[0]),
                    center[1] + s * (p[1] - center[1]),
                ]
            })
            .collect()
    }

    fn centroid_of(ring: &[[f64; 2]]) -> [f64; 2] {
        let mut c = [0.0; 2];
        for p in ring {
            c[0] += p[0];
            c[1] += p[1];
        }
        c[0] /= ring.len() as f64;
        c[1] /= ring.len() as f64;
        c
    }

    /// Full validity audit of a triangulation: triangle count, orientation,
    /// non-degeneracy, area preservation, and centroid containment.
    fn check_valid(outer: &[[f64; 2]], holes: &[Vec<[f64; 2]>], tris: &[[u32; 3]]) {
        let verts = flatten(outer, holes);
        let mut want = signed_area(outer).abs();
        for h in holes {
            want -= signed_area(h).abs();
        }
        let total_vertices: usize = outer.len() + holes.iter().map(Vec::len).sum::<usize>();
        // A hole bridge splices two duplicated seam positions into the
        // ring; the zero-area ears they can form are sliver-skipped (the
        // no-degenerate rule below), costing at most two triangles per
        // hole. Area preservation is the real coverage check.
        let expected = total_vertices + 2 * holes.len() - 2;
        let floor = expected.saturating_sub(2 * holes.len());
        assert!(
            tris.len() <= expected && tris.len() >= floor,
            "triangle count {} outside [{floor}, {expected}]",
            tris.len()
        );
        let mut got = 0.0;
        for &t in tris {
            for &idx in &t {
                assert!((idx as usize) < verts.len(), "index out of range");
            }
            let a = verts[t[0] as usize];
            let b = verts[t[1] as usize];
            let c = verts[t[2] as usize];
            let area2 = cross(a, b, c);
            assert!(area2 > 0.0, "flipped triangle {t:?}");
            assert!(area2 / 2.0 > 1e-12, "degenerate triangle {t:?}");
            got += area2 / 2.0;
            let centroid = [(a[0] + b[0] + c[0]) / 3.0, (a[1] + b[1] + c[1]) / 3.0];
            assert!(
                strictly_inside_ring(outer, centroid),
                "centroid {centroid:?} outside outer ring"
            );
            for h in holes {
                assert!(
                    !strictly_inside_ring(h, centroid),
                    "centroid {centroid:?} inside a hole"
                );
            }
        }
        let rel = (got - want).abs() / want.abs().max(1e-300);
        assert!(rel < 1e-9, "area not preserved: got {got}, want {want}");
    }

    #[test]
    fn square_two_triangles() {
        let outer = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let tris = triangulate(&outer, &[]).expect("valid square");
        assert_eq!(tris.len(), 2);
        let holes: Vec<Vec<[f64; 2]>> = Vec::new();
        check_valid(&outer, &holes, &tris);
    }

    #[test]
    fn clockwise_input_is_normalized() {
        let outer = [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]];
        let tris = triangulate(&outer, &[]).expect("valid square");
        assert_eq!(tris.len(), 2);
        let holes: Vec<Vec<[f64; 2]>> = Vec::new();
        check_valid(&outer, &holes, &tris);
        // Output winding is CCW regardless of the CW input.
        for &t in &tris {
            let area2 = cross(
                outer[t[0] as usize],
                outer[t[1] as usize],
                outer[t[2] as usize],
            );
            assert!(area2 > 0.0);
        }
    }

    #[test]
    fn affine_translation_and_scale_preserve_triangle_indices() {
        let unit = [[0.0, 0.0], [2.0, 0.0], [2.0, 1.0], [0.0, 1.0]];
        let expected = triangulate(&unit, &[]).expect("unit rectangle");

        let base = 1e150;
        let step = 1e140;
        let translated = [
            [base, base],
            [base + 2.0 * step, base],
            [base + 2.0 * step, base + step],
            [base, base + step],
        ];
        assert_eq!(
            triangulate(&translated, &[]).expect("translated rectangle"),
            expected
        );

        let tiny = unit.map(|p| [p[0] * 1e-150, p[1] * 1e-150]);
        assert_eq!(triangulate(&tiny, &[]).expect("tiny rectangle"), expected);
    }

    #[test]
    fn closing_vertex_repeat_is_dropped() {
        let outer = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [0.0, 0.0]];
        let tris = triangulate(&outer, &[]).expect("valid square");
        assert_eq!(tris.len(), 2);
        assert!(tris.iter().flatten().all(|&i| i < 5));
    }

    #[test]
    fn duplicates_and_collinear_runs_are_cleaned() {
        // A 2x2 square with a consecutive duplicate and a collinear midpoint.
        let outer = [
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 0.0],
            [2.0, 0.0],
            [2.0, 2.0],
            [0.0, 2.0],
        ];
        let tris = triangulate(&outer, &[]).expect("valid square");
        assert_eq!(tris.len(), 2);
        let mut got = 0.0;
        for &t in &tris {
            assert!(t.iter().all(|&i| (i as usize) < outer.len()));
            got += cross(
                outer[t[0] as usize],
                outer[t[1] as usize],
                outer[t[2] as usize],
            ) / 2.0;
        }
        assert!((got - 4.0).abs() < 1e-12);
    }

    #[test]
    fn square_with_square_hole() {
        let outer = [[-2.0, -2.0], [2.0, -2.0], [2.0, 2.0], [-2.0, 2.0]];
        let hole = vec![[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]];
        let holes = vec![hole];
        let tris = triangulate(&outer, &hole_refs(&holes)).expect("valid donut");
        assert_eq!(tris.len(), 8);
        check_valid(&outer, &holes, &tris);
    }

    #[test]
    fn two_square_holes() {
        let outer = [[-4.0, -4.0], [4.0, -4.0], [4.0, 4.0], [-4.0, 4.0]];
        let h1 = vec![[-3.0, -3.0], [-1.0, -3.0], [-1.0, -1.0], [-3.0, -1.0]];
        let h2 = vec![[1.0, 1.0], [3.0, 1.0], [3.0, 3.0], [1.0, 3.0]];
        let holes = vec![h1, h2];
        let tris = triangulate(&outer, &hole_refs(&holes)).expect("valid two-hole");
        assert_eq!(tris.len(), 4 + 8 + 4 - 2);
        check_valid(&outer, &holes, &tris);
    }

    #[test]
    fn concave_l_shape() {
        let outer = [
            [0.0, 0.0],
            [2.0, 0.0],
            [2.0, 1.0],
            [1.0, 1.0],
            [1.0, 2.0],
            [0.0, 2.0],
        ];
        let tris = triangulate(&outer, &[]).expect("valid L");
        assert_eq!(tris.len(), 4);
        let holes: Vec<Vec<[f64; 2]>> = Vec::new();
        check_valid(&outer, &holes, &tris);
    }

    #[test]
    fn five_point_star() {
        let tau = std::f64::consts::TAU;
        let mut outer = Vec::with_capacity(10);
        for k in 0..10 {
            let ang = tau * k as f64 / 10.0;
            let r = if k % 2 == 0 { 2.0 } else { 0.9 };
            outer.push([r * ang.cos(), r * ang.sin()]);
        }
        let tris = triangulate(&outer, &[]).expect("valid star");
        assert_eq!(tris.len(), 8);
        let holes: Vec<Vec<[f64; 2]>> = Vec::new();
        check_valid(&outer, &holes, &tris);
    }

    #[test]
    fn star_with_scaled_star_hole() {
        let mut rng = XorShift(0x1234_5678_9ABC_DEF0);
        let outer = random_star(&mut rng, 7);
        let center = centroid_of(&outer);
        let holes = vec![scaled_copy(&outer, center, 0.3)];
        let tris = triangulate(&outer, &hole_refs(&holes)).expect("valid star donut");
        check_valid(&outer, &holes, &tris);
    }

    #[test]
    fn hole_touching_outer_is_refused() {
        let outer = [[-2.0, -2.0], [2.0, -2.0], [2.0, 2.0], [-2.0, 2.0]];
        // The vertex (2, 0) lies exactly on the outer ring's right edge.
        let holes = vec![vec![[2.0, 0.0], [1.5, -0.4], [1.5, 0.4]]];
        let result = triangulate(&outer, &hole_refs(&holes));
        assert_eq!(result, Err(EarClipError::HoleTouchesOuter { hole: 0 }));
    }

    #[test]
    fn hole_crossing_outer_is_refused() {
        let outer = [[-2.0, -2.0], [2.0, -2.0], [2.0, 2.0], [-2.0, 2.0]];
        // Vertices inside, but the right edge of the hole pokes through the
        // outer boundary.
        let holes = vec![vec![[1.0, -0.5], [3.0, -0.5], [3.0, 0.5], [1.0, 0.5]]];
        let result = triangulate(&outer, &hole_refs(&holes));
        assert_eq!(result, Err(EarClipError::HoleOutsideOuter { hole: 0 }));
    }

    #[test]
    fn degenerate_inputs_are_typed_errors() {
        let empty: [[f64; 2]; 0] = [];
        assert_eq!(
            triangulate(&empty, &[]),
            Err(EarClipError::TooFewPoints {
                role: RingRole::Outer,
                ring_index: 0,
                points: 0,
            })
        );
        let two = [[0.0, 0.0], [1.0, 0.0]];
        assert!(matches!(
            triangulate(&two, &[]),
            Err(EarClipError::TooFewPoints {
                role: RingRole::Outer,
                ..
            })
        ));
        let nan = [[0.0, 0.0], [f64::NAN, 0.0], [1.0, 1.0]];
        assert_eq!(
            triangulate(&nan, &[]),
            Err(EarClipError::NonFiniteCoordinate {
                role: RingRole::Outer,
                ring_index: 0,
                vertex: 1,
            })
        );
        let collinear = [[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]];
        assert_eq!(
            triangulate(&collinear, &[]),
            Err(EarClipError::ZeroAreaRing {
                role: RingRole::Outer,
                ring_index: 0,
            })
        );
        let spike = [[0.0, 0.0], [1.0, 0.0], [0.0, 0.0]];
        assert!(matches!(
            triangulate(&spike, &[]),
            Err(EarClipError::TooFewPoints {
                role: RingRole::Outer,
                ..
            })
        ));
    }

    #[test]
    fn hole_outside_outer_is_refused() {
        let outer = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let holes = vec![vec![[10.0, 10.0], [11.0, 10.0], [11.0, 11.0]]];
        assert_eq!(
            triangulate(&outer, &hole_refs(&holes)),
            Err(EarClipError::HoleOutsideOuter { hole: 0 })
        );
    }

    #[test]
    fn invalid_tolerance_is_refused() {
        let outer = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let options = EarClipOptions { tolerance: 0.0 };
        assert_eq!(
            triangulate_with_options(&outer, &[], &options),
            Err(EarClipError::InvalidTolerance)
        );
        let options = EarClipOptions {
            tolerance: f64::NAN,
        };
        assert_eq!(
            triangulate_with_options(&outer, &[], &options),
            Err(EarClipError::InvalidTolerance)
        );
    }

    #[test]
    fn oversized_vertex_work_is_refused_before_ring_processing() {
        let outer = [[0.0, 0.0], [2.0, 0.0], [0.0, 2.0]];
        let hole = [[0.25, 0.25], [0.5, 0.25], [0.25, 0.5]];
        let hole_count = (MAX_EARCLIP_VERTICES - outer.len()) / hole.len() + 1;
        let holes: Vec<&[[f64; 2]]> = vec![&hole; hole_count];
        let total = outer.len() + hole_count * hole.len();
        assert!(matches!(
            triangulate(&outer, &holes),
            Err(EarClipError::TooManyVertices { total: got }) if got == total
        ));
    }

    #[test]
    fn error_display_is_populated() {
        let err = EarClipError::HoleOutsideOuter { hole: 2 };
        let text = format!("{err}");
        assert!(text.contains("hole 2"));
        let err: &dyn std::error::Error = &err;
        assert!(err.source().is_none());
    }

    #[test]
    fn proptest_star_polygons() {
        let mut rng = XorShift(0xC0FF_EE42_0000_0001);
        for _ in 0..300 {
            let spikes = 3 + rng.usize_below(10);
            let outer = random_star(&mut rng, spikes);
            let tris = triangulate(&outer, &[]).expect("fuzz stars are valid");
            let holes: Vec<Vec<[f64; 2]>> = Vec::new();
            check_valid(&outer, &holes, &tris);
        }
    }

    #[test]
    fn normalized_channel_topology_is_valid_or_a_named_refusal() {
        // A star hole and a second hole tucked into the pocket between
        // it and the outer ring's reflex notch. This was the original
        // named-refusal regression. Predicate-space normalization may
        // make its valid split numerically visible; either a verified
        // exact cover or the named refusal is acceptable, never a
        // degraded triangle. (Found by the fuzzer.)
        let outer: Vec<[f64; 2]> = vec![
            [1.0098742902019824, 2.676915570348665],
            [1.811513848312949, 1.4447121406645853],
            [1.4915838183885142, 0.4565094380716197],
            [2.614755087379471, 1.398345839393615],
            [3.846312363926858, 2.4713495582592686],
            [2.4037983398591565, 2.297110395603676],
        ];
        let hole0: Vec<[f64; 2]> = vec![
            [1.8626, 2.0401],
            [2.0881, 1.6935],
            [1.9981, 1.4155],
            [2.3140, 1.6804],
            [2.6604, 1.9822],
            [2.2547, 1.9332],
        ];
        let hole1: Vec<[f64; 2]> = vec![
            [1.7472, 2.1756],
            [1.9506, 2.1756],
            [1.9506, 2.3789],
            [1.7472, 2.3789],
        ];
        let holes = vec![hole0, hole1];
        match triangulate(&outer, &hole_refs(&holes)) {
            Ok(tris) => check_valid(&outer, &holes, &tris),
            Err(err) => {
                assert!(matches!(err, EarClipError::NoValidTriangulation));
                assert!(err.to_string().contains("no valid triangulation"));
            }
        }
    }

    /// Distance between two segments (0 when they cross): the minimum
    /// endpoint-to-segment distance.
    fn segment_distance(a1: [f64; 2], a2: [f64; 2], b1: [f64; 2], b2: [f64; 2]) -> f64 {
        if segments_cross(a1, a2, b1, b2) {
            return 0.0;
        }
        let pt = |p: [f64; 2], a: [f64; 2], b: [f64; 2]| {
            let ex = b[0] - a[0];
            let ey = b[1] - a[1];
            let len2 = ex * ex + ey * ey;
            if len2 == 0.0 {
                return dist2(p, a).sqrt();
            }
            let t = (((p[0] - a[0]) * ex + (p[1] - a[1]) * ey) / len2).clamp(0.0, 1.0);
            dist2(p, [a[0] + t * ex, a[1] + t * ey]).sqrt()
        };
        pt(a1, b1, b2)
            .min(pt(a2, b1, b2))
            .min(pt(b1, a1, a2))
            .min(pt(b2, a1, a2))
    }

    /// Minimum edge-to-edge distance between two rings.
    fn ring_distance(a: &[[f64; 2]], b: &[[f64; 2]]) -> f64 {
        let mut best = f64::INFINITY;
        for i in 0..a.len() {
            for j in 0..b.len() {
                best = best.min(segment_distance(
                    a[i],
                    a[(i + 1) % a.len()],
                    b[j],
                    b[(j + 1) % b.len()],
                ));
            }
        }
        best
    }

    #[test]
    fn proptest_star_polygons_with_holes() {
        let mut rng = XorShift(0xC0FF_EE42_0000_0002);
        for round in 0..200 {
            let spikes = 3 + rng.usize_below(10);
            let outer = random_star(&mut rng, spikes);
            let center = centroid_of(&outer);
            let mut holes = vec![scaled_copy(&outer, center, rng.range(0.15, 0.35))];
            // Every third round adds a second, tiny square hole placed in
            // the region with the same validators the triangulator uses.
            if round % 3 == 0 {
                let tol_abs = EarClipOptions::default().tolerance * bbox_diagonal(&outer);
                let cleaned_outer =
                    clean_ring(&outer, 0, RingRole::Outer, 0, tol_abs, true).expect("clean outer");
                for _ in 0..64 {
                    let hx = center[0] + rng.range(-0.5, 0.5);
                    let hy = center[1] + rng.range(-0.5, 0.5);
                    let d = rng.range(0.05, 0.12);
                    let candidate = vec![
                        [hx - d, hy - d],
                        [hx + d, hy - d],
                        [hx + d, hy + d],
                        [hx - d, hy + d],
                    ];
                    let cleaned = clean_ring(&candidate, 0, RingRole::Hole, 1, tol_abs, false)
                        .expect("clean candidate");
                    if check_hole(&cleaned_outer, &cleaned, tol_abs, 1).is_err() {
                        continue;
                    }
                    // The same disjointness the triangulator requires:
                    // no corner containment, no edge crossings, no
                    // near-touches against the star hole.
                    let star = clean_ring(&holes[0].clone(), 0, RingRole::Hole, 0, tol_abs, false)
                        .expect("clean star hole");
                    if check_holes_disjoint(&cleaned, &star, tol_abs, 1, 0).is_err() {
                        continue;
                    }
                    // The fuzz family stays inside the strict-tileable
                    // region: real clearance from the outer boundary and
                    // the first hole (5% of the outer's bbox diagonal).
                    let clearance = 0.05 * bbox_diagonal(&outer);
                    if ring_distance(&candidate, &outer) < clearance
                        || ring_distance(&candidate, &holes[0]) < clearance
                    {
                        continue;
                    }
                    holes.push(candidate);
                    break;
                }
            }
            let tris = triangulate(&outer, &hole_refs(&holes)).expect("fuzz donuts are valid");
            let verts = flatten(&outer, &holes);
            for (ti, t) in tris.iter().enumerate() {
                let a = verts[t[0] as usize];
                let b = verts[t[1] as usize];
                let c = verts[t[2] as usize];
                let cent = [(a[0] + b[0] + c[0]) / 3.0, (a[1] + b[1] + c[1]) / 3.0];
                assert!(
                    strictly_inside_ring(&outer, cent),
                    "round {round} tri {ti} {t:?} centroid {cent:.4?} outside outer ({}) holes {}",
                    outer.len(),
                    holes.len()
                );
            }
            check_valid(&outer, &holes, &tris);
        }
    }

    #[test]
    fn proptest_determinism() {
        let mut rng = XorShift(0xC0FF_EE42_0000_0003);
        for _ in 0..50 {
            let spikes = 3 + rng.usize_below(8);
            let outer = random_star(&mut rng, spikes);
            let center = centroid_of(&outer);
            let holes = vec![scaled_copy(&outer, center, 0.25)];
            let first = triangulate(&outer, &hole_refs(&holes)).expect("valid");
            let second = triangulate(&outer, &hole_refs(&holes)).expect("valid");
            assert_eq!(first, second, "same input must give identical triangles");
        }
    }
}
