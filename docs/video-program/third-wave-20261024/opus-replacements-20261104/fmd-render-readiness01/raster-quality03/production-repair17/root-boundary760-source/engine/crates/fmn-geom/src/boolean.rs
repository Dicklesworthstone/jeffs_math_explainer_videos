//! Certified planar path booleans (§7.4).
//!
//! This module is the permanent Stage-1 implementation: quadratic paths are
//! flattened under a caller-visible error bound, their line segments are
//! split into a planar arrangement, and boolean boundaries are selected from
//! the arrangement's winding transitions. It is intentionally also the
//! reference implementation and fuzz target for any future curve-aware
//! Stage-2 class.
//!
//! The topology rules are explicit:
//!
//! - proper crossings and tangencies become shared arrangement vertices;
//! - shared endpoints are one vertex, independent of input ownership;
//! - collinear overlaps split at every overlap endpoint, then signed winding
//!   multiplicities decide whether each atomic edge survives;
//! - exact backtracks and other zero-area edge pairs cancel by multiplicity;
//! - open subpaths are closed for fill, matching the Reference's fill
//!   semantics;
//! - the operation is two-dimensional: `x` and `y` participate and output
//!   `z` is zero, matching the Reference's `point[:2]` conversion.
//!
//! ## Admitted Stage-2 classes
//!
//! `separated-control-hulls` (fm-8dx) is the first curve-aware class. Both
//! operands must contain a nonzero-area contour and their complete quadratic
//! control-point bounds must be strictly separated on `x` or `y`. A quadratic
//! and its implicit fill-closing chord lie inside that control hull, so the
//! filled sets cannot meet. Union/exclusion may therefore concatenate the
//! original quadratic subpaths (projected to `xy` and explicitly closed),
//! difference may preserve the subject, and intersection is empty. Equality
//! at a bound is deliberately *not* admitted:
//! tangencies, shared endpoints, every overlap class, and zero-area-only paths
//! route to [`BooleanRoute::FlattenClip`]. Tests compare every admitted
//! operation against the forced Stage-1 result on a point-in-fill grid.
//!
//! `transversal-interiors` (fm-qjy) is the second admitted class: every
//! cross-operand piece intersection is a proper crossing strictly inside
//! both pieces, proved by fat-line-pruned Bézier clipping, bounded Newton
//! polish, and the crossing-sine screen; same-operand pieces may meet only
//! at shared contour endpoints. The arrangement is built from split
//! quadratics — exact in the parameterization, never claimed exact in
//! coordinates — classified by ray-cast winding, and stitched by the same
//! angular-sort face traversal as Stage 1. The degeneracy-class taxonomy,
//! admission predicates, numerical bounds, and per-class proof obligations
//! are specified in `docs/geometry/curve_booleans.md`; shared endpoints,
//! tangencies, overlaps, and zero-area cases remain specified-but-unproved
//! and route to [`BooleanRoute::FlattenClip`] permanently. All-line inputs
//! also stay on Stage 1 by construction: flattening is exact for polygons,
//! so the captured skia-pathops fixtures permanently exercise the fallback.
//!
//! Further curve-aware classes do not get a heuristic fast path. Each needs a
//! written degeneracy specification and topology-aware differential proof
//! against [`path_boolean_flattened`]. Resource exhaustion and numerically
//! unresolvable arrangements are typed refusals, never silently coarsened
//! output.

use crate::QuadPath;
use fmn_core::types::Vec3;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

mod curve_boolean;

type Point = [f64; 2];

/// Coordinates larger than this are refused before products are formed.
///
/// The bound leaves ample exponent headroom for the double-double
/// determinant and squared-distance calculations. It is a numerical-domain
/// guard, not a scene-size recommendation.
pub const MAX_BOOLEAN_COORDINATE: f64 = 1.0e100;

const PARAMETER_MERGE_EPSILON: f64 = 128.0 * f64::EPSILON;
const MAX_FLATTEN_DEPTH: u8 = 64;

/// Which filled-set operation to apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanOperation {
    /// Points contained by either operand.
    Union,
    /// Points contained by both operands.
    Intersection,
    /// Points contained by the subject and not the clip.
    Difference,
    /// Points contained by exactly one operand.
    Exclusion,
}

/// How contour winding maps to filledness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillRule {
    /// Any nonzero winding is filled.
    NonZero,
    /// Odd winding is filled.
    EvenOdd,
}

/// The implementation route used for a boolean result.
///
/// Consumers record this rather than inferring a fast path from output
/// shape. New variants are admitted only with the Stage-2 proof described in
/// the module documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanRoute {
    /// Exact quadratic preservation proved by strictly separated control
    /// hulls.
    CurveAwareSeparated,
    /// Quadratic preservation proved by the transversal-interiors class:
    /// fat-line-pruned intersection, bounded Newton polish, exact
    /// parameterization splitting, and curve stitching
    /// (`docs/geometry/curve_booleans.md`).
    CurveAwareTransversal,
    /// The certified quadratic-flattening and line-arrangement fallback.
    FlattenClip,
}

/// Work budgets for hostile or accidentally pathological paths (§16.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BooleanLimits {
    /// Maximum total line segments after flattening both inputs.
    pub max_flattened_segments: usize,
    /// Maximum bounding-box candidate pairs tested for intersection.
    pub max_pair_tests: usize,
    /// Maximum intersection/overlap split events admitted.
    pub max_intersections: usize,
    /// Maximum edge visits used to classify arrangement faces.
    pub max_classification_tests: usize,
    /// Maximum line segments in the returned path.
    pub max_output_segments: usize,
}

impl BooleanLimits {
    /// Defaults sized well above ordinary scene geometry while bounding every
    /// quadratic phase.
    pub const DEFAULT: Self = Self {
        max_flattened_segments: 16_384,
        max_pair_tests: 8_000_000,
        max_intersections: 262_144,
        max_classification_tests: 16_000_000,
        max_output_segments: 65_536,
    };
}

impl Default for BooleanLimits {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Configuration for [`path_boolean`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BooleanOptions {
    /// Maximum scene-space deviation of each flattened quadratic chord.
    pub tolerance: f64,
    /// Fill rule for the subject operand.
    pub subject_fill_rule: FillRule,
    /// Fill rule for the clip operand.
    pub clip_fill_rule: FillRule,
    /// Resource budgets declared before work begins.
    pub limits: BooleanLimits,
}

impl Default for BooleanOptions {
    fn default() -> Self {
        Self {
            tolerance: crate::cubic::DEFAULT_TOLERANCE_SCENE,
            subject_fill_rule: FillRule::NonZero,
            clip_fill_rule: FillRule::NonZero,
            limits: BooleanLimits::DEFAULT,
        }
    }
}

/// Counters exposing the bounded work actually performed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BooleanStats {
    /// Quadratic curves presented by both input paths.
    pub input_curves: usize,
    /// Line segments emitted by adaptive flattening.
    pub flattened_segments: usize,
    /// Candidate segment pairs tested after bounding-box pruning.
    pub pair_tests: usize,
    /// Crossing, touch, and overlap split events admitted.
    pub intersections: usize,
    /// Edge visits used for local-face clearance and winding queries.
    pub classification_tests: usize,
    /// Closed contours in the result.
    pub output_contours: usize,
    /// Boundary curves in the result (line quadratics on the fallback route).
    pub output_segments: usize,
}

/// A path boolean plus auditable route and work counters.
#[derive(Debug, Clone, PartialEq)]
pub struct BooleanResult {
    /// The result as closed quadratic subpaths.
    pub path: QuadPath,
    /// Which admitted implementation produced `path`.
    pub route: BooleanRoute,
    /// Bounded-work counters for diagnostics and tests.
    pub stats: BooleanStats,
}

/// Input side named by a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanOperand {
    /// The left/subject operand.
    Subject,
    /// The right/clip operand.
    Clip,
}

/// Budgeted phase named by [`BooleanError::ResourceLimit`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanPhase {
    /// Adaptive quadratic flattening.
    Flatten,
    /// Candidate segment-pair intersection.
    PairTests,
    /// Arrangement split-event creation.
    Intersections,
    /// Local-face clearance and winding classification.
    Classification,
    /// Result contour emission.
    Output,
}

/// Typed failures from the certified boolean.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanError {
    /// `BooleanOptions::tolerance` was not positive and finite.
    InvalidTolerance,
    /// An input coordinate was non-finite or outside the supported domain.
    InvalidCoordinate {
        /// Which input contained the coordinate.
        operand: BooleanOperand,
        /// Index in `QuadPath::points()`.
        point: usize,
    },
    /// A declared work budget was exhausted.
    ResourceLimit {
        /// Phase that exhausted its budget.
        phase: BooleanPhase,
        /// Declared maximum for that phase.
        limit: usize,
    },
    /// Distinct topology fell below representable floating-point resolution.
    NumericalResolution(&'static str),
    /// The constructed arrangement violated an internal topological law.
    InvalidTopology(&'static str),
}

impl fmt::Display for BooleanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTolerance => {
                write!(f, "boolean tolerance must be a positive, finite number")
            }
            Self::InvalidCoordinate { operand, point } => write!(
                f,
                "{operand:?} boolean point {point} is non-finite or outside ±{MAX_BOOLEAN_COORDINATE:e}"
            ),
            Self::ResourceLimit { phase, limit } => {
                write!(
                    f,
                    "boolean {phase:?} exceeded its declared {limit}-item budget"
                )
            }
            Self::NumericalResolution(what) => {
                write!(f, "boolean topology is below numerical resolution: {what}")
            }
            Self::InvalidTopology(what) => {
                write!(
                    f,
                    "boolean arrangement is topologically inconsistent: {what}"
                )
            }
        }
    }
}

impl std::error::Error for BooleanError {}

/// Apply a deterministic planar boolean to two paths.
///
/// A curve-aware class is used only when its admission proof succeeds;
/// otherwise this calls [`path_boolean_flattened`]. Inspect
/// [`BooleanResult::route`] when routing is operationally relevant.
pub fn path_boolean(
    subject: &QuadPath,
    clip: &QuadPath,
    operation: BooleanOperation,
    options: BooleanOptions,
) -> Result<BooleanResult, BooleanError> {
    validate_options_and_path(subject, BooleanOperand::Subject, options)?;
    validate_options_and_path(clip, BooleanOperand::Clip, options)?;
    if let Some(result) = try_curve_aware_separated(subject, clip, operation, options)? {
        return Ok(result);
    }
    if let Some(result) = curve_boolean::try_transversal(subject, clip, operation, options) {
        return Ok(result);
    }
    flatten_clip_validated(subject, clip, operation, options)
}

/// Force the permanent certified flatten-and-clip implementation.
///
/// Quadratics are flattened to `options.tolerance`; open subpaths are
/// implicitly closed for fill. This is the reference implementation and
/// primary fuzz target for every curve-aware class.
pub fn path_boolean_flattened(
    subject: &QuadPath,
    clip: &QuadPath,
    operation: BooleanOperation,
    options: BooleanOptions,
) -> Result<BooleanResult, BooleanError> {
    validate_options_and_path(subject, BooleanOperand::Subject, options)?;
    validate_options_and_path(clip, BooleanOperand::Clip, options)?;
    flatten_clip_validated(subject, clip, operation, options)
}

fn flatten_clip_validated(
    subject: &QuadPath,
    clip: &QuadPath,
    operation: BooleanOperation,
    options: BooleanOptions,
) -> Result<BooleanResult, BooleanError> {
    let mut stats = BooleanStats {
        input_curves: subject.num_curves().saturating_add(clip.num_curves()),
        ..BooleanStats::default()
    };
    let subject_contours = flatten_path(subject, options, &mut stats)?;
    let clip_contours = flatten_path(clip, options, &mut stats)?;

    let mut nodes = Nodes::default();
    let mut segments = Vec::with_capacity(stats.flattened_segments);
    append_segments(&subject_contours, 0, &mut nodes, &mut segments);
    append_segments(&clip_contours, 1, &mut nodes, &mut segments);

    split_at_intersections(&mut segments, &mut nodes, options, &mut stats)?;
    merge_coincident_split_marks(&mut segments, &mut nodes);
    let (vertices, edges) = build_atomic_edges(&segments, &mut nodes)?;
    let graph = classify_boundaries(vertices, edges, operation, options, &mut stats)?;
    let loops = graph.trace_loops(options, &mut stats)?;
    let path = loops_to_path(&loops)?;

    stats.output_contours = loops.len();
    Ok(BooleanResult {
        path,
        route: BooleanRoute::FlattenClip,
        stats,
    })
}

fn try_curve_aware_separated(
    subject: &QuadPath,
    clip: &QuadPath,
    operation: BooleanOperation,
    options: BooleanOptions,
) -> Result<Option<BooleanResult>, BooleanError> {
    if options.subject_fill_rule != FillRule::NonZero
        || options.clip_fill_rule != FillRule::NonZero
        || !has_nonzero_anchor_area(subject)
        || !has_nonzero_anchor_area(clip)
    {
        return Ok(None);
    }
    let Some(subject_bounds) = control_bounds(subject) else {
        return Ok(None);
    };
    let Some(clip_bounds) = control_bounds(clip) else {
        return Ok(None);
    };
    if !subject_bounds.strictly_separated(clip_bounds) {
        return Ok(None);
    }

    let path = match operation {
        BooleanOperation::Union | BooleanOperation::Exclusion => project_paths(&[subject, clip])?,
        BooleanOperation::Intersection => QuadPath::new(),
        BooleanOperation::Difference => project_paths(&[subject])?,
    };
    let stats = BooleanStats {
        input_curves: subject.num_curves().saturating_add(clip.num_curves()),
        output_contours: path.subpaths().len(),
        output_segments: path
            .subpaths()
            .iter()
            .map(|subpath| subpath.len().saturating_sub(1) / 2)
            .sum(),
        ..BooleanStats::default()
    };
    Ok(Some(BooleanResult {
        path,
        route: BooleanRoute::CurveAwareSeparated,
        stats,
    }))
}

#[derive(Clone, Copy)]
struct Bounds {
    min: Point,
    max: Point,
}

impl Bounds {
    fn strictly_separated(self, other: Self) -> bool {
        self.max[0] < other.min[0]
            || other.max[0] < self.min[0]
            || self.max[1] < other.min[1]
            || other.max[1] < self.min[1]
    }
}

fn control_bounds(path: &QuadPath) -> Option<Bounds> {
    let mut points = path.points().iter();
    let first = xy(*points.next()?);
    let mut bounds = Bounds {
        min: first,
        max: first,
    };
    for &point in points {
        let point = xy(point);
        for (axis, &coordinate) in point.iter().enumerate() {
            bounds.min[axis] = bounds.min[axis].min(coordinate);
            bounds.max[axis] = bounds.max[axis].max(coordinate);
        }
    }
    Some(bounds)
}

fn has_nonzero_anchor_area(path: &QuadPath) -> bool {
    path.subpaths().into_iter().any(|subpath| {
        if subpath.len() < 5 {
            return false;
        }
        let mut area = Dd::ZERO;
        let anchors: Vec<Point> = subpath.iter().step_by(2).copied().map(xy).collect();
        for pair in anchors.windows(2) {
            area = area.add(product_difference(
                pair[0][0], pair[1][1], pair[0][1], pair[1][0],
            ));
        }
        if anchors.first() != anchors.last() {
            let first = anchors[0];
            let last = anchors[anchors.len() - 1];
            area = area.add(product_difference(last[0], first[1], last[1], first[0]));
        }
        area.sign() != 0
    })
}

fn project_paths(paths: &[&QuadPath]) -> Result<QuadPath, BooleanError> {
    let mut output = QuadPath::new();
    for path in paths {
        for subpath in path.subpaths() {
            let mut projected: Vec<Vec3> = subpath
                .iter()
                .map(|point| [canonical_zero(point[0]), canonical_zero(point[1]), 0.0])
                .collect();
            if projected.len() >= 3 {
                let first = projected[0];
                let last_index = projected.len() - 1;
                let last = projected[last_index];
                let tolerance = path.tolerance_for_point_equality();
                let xy_equal = last[..2]
                    .iter()
                    .zip(&first[..2])
                    .all(|(last, first)| (last - first).abs() < tolerance);
                if xy_equal {
                    projected[last_index] = first;
                } else {
                    projected.push([
                        last[0] + (first[0] - last[0]) * 0.5,
                        last[1] + (first[1] - last[1]) * 0.5,
                        0.0,
                    ]);
                    projected.push(first);
                }
            }
            output
                .add_subpath(&projected)
                .map_err(|_| BooleanError::InvalidTopology("failed to project quadratic path"))?;
        }
    }
    Ok(output)
}

/// Bounded parser-and-execution probe for `cargo-fuzz` and corpus smoke tests.
///
/// Arbitrary bytes become at most two 32-curve paths. Every generated
/// operation uses tight limits, so the probe may return a typed refusal but
/// cannot request unbounded work. Both the routed entry point (which may
/// attempt a curve-aware class) and the forced fallback are exercised. The
/// boolean results are deliberately ignored: a fuzz harness is checking for
/// panics, hangs, and invariant violations.
#[must_use]
pub fn fuzz_probe(bytes: &[u8]) -> bool {
    let selector = bytes.first().copied().unwrap_or(0);
    let body = bytes.get(1..).unwrap_or_default();
    let body = &body[..body.len().min(258)];
    let split = body.len() / 2;
    let subject = fuzz_path(&body[..split]);
    let clip = fuzz_path(&body[split..]);
    let operation = match selector & 3 {
        0 => BooleanOperation::Union,
        1 => BooleanOperation::Intersection,
        2 => BooleanOperation::Difference,
        _ => BooleanOperation::Exclusion,
    };
    let fill_rule = if selector & 4 == 0 {
        FillRule::NonZero
    } else {
        FillRule::EvenOdd
    };
    let options = BooleanOptions {
        subject_fill_rule: fill_rule,
        clip_fill_rule: fill_rule,
        limits: BooleanLimits {
            max_flattened_segments: 2_048,
            max_pair_tests: 200_000,
            max_intersections: 8_192,
            max_classification_tests: 200_000,
            max_output_segments: 8_192,
        },
        ..BooleanOptions::default()
    };
    drop(path_boolean_flattened(&subject, &clip, operation, options));
    drop(path_boolean(&subject, &clip, operation, options));
    true
}

fn fuzz_path(bytes: &[u8]) -> QuadPath {
    let (chunks, _) = bytes.as_chunks::<4>();
    let Some((first, rest)) = chunks.split_first() else {
        return QuadPath::new();
    };
    let mut path = QuadPath::new();
    path.start_new_path(fuzz_point(first[0], first[1]));
    if path
        .add_quadratic_bezier_curve_to(
            fuzz_point(first[2], first[3]),
            fuzz_point(
                first[0].wrapping_add(first[2]),
                first[1].wrapping_add(first[3]),
            ),
            true,
        )
        .is_err()
    {
        return QuadPath::new();
    }
    for chunk in rest.iter().take(31) {
        if path
            .add_quadratic_bezier_curve_to(
                fuzz_point(chunk[0], chunk[1]),
                fuzz_point(chunk[2], chunk[3]),
                true,
            )
            .is_err()
        {
            return QuadPath::new();
        }
    }
    path
}

fn fuzz_point(x: u8, y: u8) -> Vec3 {
    [
        f64::from(i8::from_ne_bytes([x])) / 8.0,
        f64::from(i8::from_ne_bytes([y])) / 8.0,
        0.0,
    ]
}

fn validate_options_and_path(
    path: &QuadPath,
    operand: BooleanOperand,
    options: BooleanOptions,
) -> Result<(), BooleanError> {
    if !options.tolerance.is_finite() || options.tolerance <= 0.0 {
        return Err(BooleanError::InvalidTolerance);
    }
    for (point, value) in path.points().iter().enumerate() {
        if value[..2]
            .iter()
            .any(|coordinate| !coordinate.is_finite() || coordinate.abs() > MAX_BOOLEAN_COORDINATE)
        {
            return Err(BooleanError::InvalidCoordinate { operand, point });
        }
    }
    Ok(())
}

fn flatten_path(
    path: &QuadPath,
    options: BooleanOptions,
    stats: &mut BooleanStats,
) -> Result<Vec<Vec<Point>>, BooleanError> {
    let mut contours = Vec::new();
    for subpath in path.subpaths() {
        if subpath.len() < 3 {
            continue;
        }
        let mut points = vec![xy(subpath[0])];
        for index in (0..subpath.len() - 2).step_by(2) {
            flatten_quadratic(
                xy(subpath[index]),
                xy(subpath[index + 1]),
                xy(subpath[index + 2]),
                options,
                stats,
                &mut points,
            )?;
        }
        points.dedup();
        if points.len() < 2 {
            continue;
        }
        let first = points[0];
        let last_index = points.len() - 1;
        let last = points[last_index];
        if length(sub(last, first)) <= options.tolerance {
            points[last_index] = first;
        } else {
            bump_flattened_segments(options, stats)?;
            points.push(points[0]);
        }
        points.dedup();
        if points.len() >= 4 {
            contours.push(points);
        }
    }
    Ok(contours)
}

#[derive(Clone, Copy)]
struct FlatPiece {
    p0: Point,
    p1: Point,
    p2: Point,
    depth: u8,
}

fn flatten_quadratic(
    p0: Point,
    p1: Point,
    p2: Point,
    options: BooleanOptions,
    stats: &mut BooleanStats,
    output: &mut Vec<Point>,
) -> Result<(), BooleanError> {
    let mut stack = vec![FlatPiece {
        p0,
        p1,
        p2,
        depth: 0,
    }];
    while let Some(piece) = stack.pop() {
        let left_mid = midpoint(piece.p0, piece.p1);
        let right_mid = midpoint(piece.p1, piece.p2);
        let curve_mid = midpoint(left_mid, right_mid);
        let chord_mid = midpoint(piece.p0, piece.p2);
        let deviation = length(sub(curve_mid, chord_mid));

        let cannot_split = left_mid == piece.p0
            || left_mid == piece.p1
            || right_mid == piece.p1
            || right_mid == piece.p2
            || curve_mid == piece.p0
            || curve_mid == piece.p2;
        if deviation <= options.tolerance || cannot_split {
            if output.last().copied() != Some(piece.p2) {
                bump_flattened_segments(options, stats)?;
                output.push(piece.p2);
            }
            continue;
        }
        if piece.depth == MAX_FLATTEN_DEPTH {
            return Err(BooleanError::NumericalResolution(
                "quadratic flattening did not converge",
            ));
        }
        let depth = piece.depth + 1;
        stack.push(FlatPiece {
            p0: curve_mid,
            p1: right_mid,
            p2: piece.p2,
            depth,
        });
        stack.push(FlatPiece {
            p0: piece.p0,
            p1: left_mid,
            p2: curve_mid,
            depth,
        });
    }
    Ok(())
}

fn bump_flattened_segments(
    options: BooleanOptions,
    stats: &mut BooleanStats,
) -> Result<(), BooleanError> {
    if stats.flattened_segments == options.limits.max_flattened_segments {
        return Err(BooleanError::ResourceLimit {
            phase: BooleanPhase::Flatten,
            limit: options.limits.max_flattened_segments,
        });
    }
    stats.flattened_segments += 1;
    Ok(())
}

fn xy(point: Vec3) -> Point {
    [canonical_zero(point[0]), canonical_zero(point[1])]
}

fn canonical_zero(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
}

#[derive(Debug, Clone, Copy)]
struct Mark {
    t: f64,
    node: usize,
}

#[derive(Debug)]
struct SourceSegment {
    p: Point,
    q: Point,
    start_node: usize,
    end_node: usize,
    operand: usize,
    marks: Vec<Mark>,
}

fn append_segments(
    contours: &[Vec<Point>],
    operand: usize,
    nodes: &mut Nodes,
    output: &mut Vec<SourceSegment>,
) {
    for contour in contours {
        let mut contour_nodes = Vec::with_capacity(contour.len());
        for &point in &contour[..contour.len() - 1] {
            contour_nodes.push(nodes.push(point));
        }
        contour_nodes.push(contour_nodes[0]);
        for index in 0..contour.len() - 1 {
            let start_node = contour_nodes[index];
            let end_node = contour_nodes[index + 1];
            let p = contour[index];
            let q = contour[index + 1];
            if p == q {
                continue;
            }
            output.push(SourceSegment {
                p,
                q,
                start_node,
                end_node,
                operand,
                marks: vec![
                    Mark {
                        t: 0.0,
                        node: start_node,
                    },
                    Mark {
                        t: 1.0,
                        node: end_node,
                    },
                ],
            });
        }
    }
}

#[derive(Default)]
struct Nodes {
    parent: Vec<usize>,
    point: Vec<Point>,
}

impl Nodes {
    fn push(&mut self, point: Point) -> usize {
        let id = self.parent.len();
        self.parent.push(id);
        self.point.push(point);
        id
    }

    fn find(&mut self, id: usize) -> usize {
        let mut root = id;
        while self.parent[root] != root {
            root = self.parent[root];
        }
        let mut current = id;
        while self.parent[current] != current {
            let next = self.parent[current];
            self.parent[current] = root;
            current = next;
        }
        root
    }

    fn union(&mut self, a: usize, b: usize) -> usize {
        let a = self.find(a);
        let b = self.find(b);
        if a == b {
            return a;
        }
        let root = a.min(b);
        let child = a.max(b);
        self.parent[child] = root;
        root
    }
}

fn split_at_intersections(
    segments: &mut [SourceSegment],
    nodes: &mut Nodes,
    options: BooleanOptions,
    stats: &mut BooleanStats,
) -> Result<(), BooleanError> {
    let mut order: Vec<usize> = (0..segments.len()).collect();
    order.sort_by(|&a, &b| {
        bbox(segments[a].p, segments[a].q)
            .0
            .total_cmp(&bbox(segments[b].p, segments[b].q).0)
            .then_with(|| a.cmp(&b))
    });

    for left in 0..order.len() {
        let i = order[left];
        let (_, i_max_x, i_min_y, i_max_y) = bbox(segments[i].p, segments[i].q);
        for &j in &order[left + 1..] {
            let (j_min_x, _, j_min_y, j_max_y) = bbox(segments[j].p, segments[j].q);
            if j_min_x > i_max_x {
                break;
            }
            if stats.pair_tests == options.limits.max_pair_tests {
                return Err(BooleanError::ResourceLimit {
                    phase: BooleanPhase::PairTests,
                    limit: options.limits.max_pair_tests,
                });
            }
            stats.pair_tests += 1;
            if j_min_y > i_max_y || j_max_y < i_min_y {
                continue;
            }
            intersect_pair(i, j, segments, nodes, options, stats)?;
        }
    }
    Ok(())
}

fn intersect_pair(
    i: usize,
    j: usize,
    segments: &mut [SourceSegment],
    nodes: &mut Nodes,
    options: BooleanOptions,
    stats: &mut BooleanStats,
) -> Result<(), BooleanError> {
    let p = segments[i].p;
    let q = segments[j].p;
    let r = sub(segments[i].q, p);
    let s = sub(segments[j].q, q);
    let denominator = cross_dd(r, s);
    let q_minus_p = sub(q, p);

    if denominator.sign() != 0 {
        let t_numerator = cross_dd(q_minus_p, s);
        let u_numerator = cross_dd(q_minus_p, r);
        if !ratio_in_unit(t_numerator, denominator) || !ratio_in_unit(u_numerator, denominator) {
            return Ok(());
        }
        let t = ratio(t_numerator, denominator);
        let u = ratio(u_numerator, denominator);
        bump_intersections(options, stats)?;
        attach_crossing(i, j, t, u, segments, nodes);
        return Ok(());
    }

    if cross_dd(q_minus_p, r).sign() != 0 {
        return Ok(());
    }

    let i_endpoints = [
        (segments[i].p, segments[i].start_node),
        (segments[i].q, segments[i].end_node),
    ];
    let j_endpoints = [
        (segments[j].p, segments[j].start_node),
        (segments[j].q, segments[j].end_node),
    ];
    let mut attached = BTreeSet::new();
    for &(point, node) in &i_endpoints {
        if point_on_segment(point, segments[j].p, segments[j].q) {
            let parameter = segment_parameter(point, segments[j].p, segments[j].q);
            attach_existing_node(j, parameter, node, segments, nodes);
            attached.insert((0_u8, node));
        }
    }
    for &(point, node) in &j_endpoints {
        if point_on_segment(point, segments[i].p, segments[i].q) {
            let parameter = segment_parameter(point, segments[i].p, segments[i].q);
            attach_existing_node(i, parameter, node, segments, nodes);
            attached.insert((1_u8, node));
        }
    }
    for _ in attached {
        bump_intersections(options, stats)?;
    }
    Ok(())
}

fn attach_crossing(
    i: usize,
    j: usize,
    t: f64,
    u: f64,
    segments: &mut [SourceSegment],
    nodes: &mut Nodes,
) {
    let t_endpoint = endpoint_node(&segments[i], t);
    let u_endpoint = endpoint_node(&segments[j], u);
    let node = match (t_endpoint, u_endpoint) {
        (Some(a), Some(b)) => nodes.union(a, b),
        (Some(a), None) => a,
        (None, Some(b)) => b,
        (None, None) => {
            let from_i = lerp(segments[i].p, segments[i].q, t);
            let from_j = lerp(segments[j].p, segments[j].q, u);
            nodes.push(midpoint(from_i, from_j))
        }
    };
    segments[i].marks.push(Mark { t, node });
    segments[j].marks.push(Mark { t: u, node });
}

fn endpoint_node(segment: &SourceSegment, t: f64) -> Option<usize> {
    if t <= PARAMETER_MERGE_EPSILON {
        Some(segment.start_node)
    } else if 1.0 - t <= PARAMETER_MERGE_EPSILON {
        Some(segment.end_node)
    } else {
        None
    }
}

fn attach_existing_node(
    segment: usize,
    t: f64,
    node: usize,
    segments: &mut [SourceSegment],
    nodes: &mut Nodes,
) {
    if let Some(endpoint) = endpoint_node(&segments[segment], t) {
        nodes.union(endpoint, node);
    }
    segments[segment].marks.push(Mark { t, node });
}

fn bump_intersections(
    options: BooleanOptions,
    stats: &mut BooleanStats,
) -> Result<(), BooleanError> {
    if stats.intersections == options.limits.max_intersections {
        return Err(BooleanError::ResourceLimit {
            phase: BooleanPhase::Intersections,
            limit: options.limits.max_intersections,
        });
    }
    stats.intersections += 1;
    Ok(())
}

fn merge_coincident_split_marks(segments: &mut [SourceSegment], nodes: &mut Nodes) {
    for segment in segments {
        segment
            .marks
            .sort_by(|a, b| a.t.total_cmp(&b.t).then_with(|| a.node.cmp(&b.node)));
        for pair in segment.marks.windows(2) {
            if (pair[1].t - pair[0].t).abs() <= PARAMETER_MERGE_EPSILON {
                nodes.union(pair[0].node, pair[1].node);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct EdgeWinding {
    subject: i64,
    clip: i64,
}

#[derive(Debug, Clone, Copy)]
struct AtomicEdge {
    from: usize,
    to: usize,
    winding: EdgeWinding,
}

fn build_atomic_edges(
    segments: &[SourceSegment],
    nodes: &mut Nodes,
) -> Result<(Vec<Point>, Vec<AtomicEdge>), BooleanError> {
    let mut split_roots = Vec::with_capacity(segments.len());
    let mut roots = BTreeSet::new();
    for segment in segments {
        // `merge_coincident_split_marks` establishes this ordering before
        // construction; avoid cloning an intersection-heavy mark list merely
        // to repeat the sort.
        let mut run = Vec::with_capacity(segment.marks.len());
        for mark in &segment.marks {
            let root = nodes.find(mark.node);
            if run.last().is_none_or(|&(_, prior)| prior != root) {
                run.push((mark.t, root));
                roots.insert(root);
            }
        }
        split_roots.push(run);
    }

    let mut sorted_roots: Vec<usize> = roots.into_iter().collect();
    sorted_roots
        .sort_by(|&a, &b| point_cmp(nodes.point[a], nodes.point[b]).then_with(|| a.cmp(&b)));
    let mut compact = vec![None; nodes.parent.len()];
    let mut vertices = Vec::with_capacity(sorted_roots.len());
    for root in sorted_roots {
        compact[root] = Some(vertices.len());
        vertices.push(nodes.point[root]);
    }

    let mut accumulated: BTreeMap<(usize, usize), EdgeWinding> = BTreeMap::new();
    for (segment, marks) in segments.iter().zip(split_roots) {
        for pair in marks.windows(2) {
            let from = compact[pair[0].1].ok_or(BooleanError::InvalidTopology(
                "split vertex was not compacted",
            ))?;
            let to = compact[pair[1].1].ok_or(BooleanError::InvalidTopology(
                "split vertex was not compacted",
            ))?;
            if from == to {
                continue;
            }
            let (key, direction) = if from < to {
                ((from, to), 1_i64)
            } else {
                ((to, from), -1_i64)
            };
            let winding = accumulated.entry(key).or_default();
            if segment.operand == 0 {
                winding.subject += direction;
            } else {
                winding.clip += direction;
            }
        }
    }

    let edges = accumulated
        .into_iter()
        .filter_map(|((from, to), winding)| {
            (winding.subject != 0 || winding.clip != 0).then_some(AtomicEdge { from, to, winding })
        })
        .collect();
    Ok((vertices, edges))
}

struct BoundaryGraph {
    vertices: Vec<Point>,
    half_edges: Vec<HalfEdge>,
    outgoing: Vec<Vec<usize>>,
    boundary: Vec<bool>,
}

#[derive(Clone, Copy)]
struct HalfEdge {
    from: usize,
    to: usize,
}

fn classify_boundaries(
    vertices: Vec<Point>,
    edges: Vec<AtomicEdge>,
    operation: BooleanOperation,
    options: BooleanOptions,
    stats: &mut BooleanStats,
) -> Result<BoundaryGraph, BooleanError> {
    let mut half_edges = Vec::with_capacity(edges.len() * 2);
    let mut outgoing = vec![Vec::new(); vertices.len()];
    let mut boundary = Vec::with_capacity(edges.len() * 2);

    for (edge_index, edge) in edges.iter().enumerate() {
        let p = vertices[edge.from];
        let q = vertices[edge.to];
        let edge_length = length(sub(q, p));
        if edge_length == 0.0 {
            return Err(BooleanError::NumericalResolution(
                "an atomic edge has zero representable length",
            ));
        }
        let center = midpoint(p, q);
        let mut clearance = edge_length * 0.5;
        for (other_index, other) in edges.iter().enumerate() {
            if edge_index == other_index {
                continue;
            }
            bump_classification(options, stats)?;
            let distance = point_segment_distance(center, vertices[other.from], vertices[other.to]);
            clearance = clearance.min(distance);
        }
        if clearance == 0.0 {
            return Err(BooleanError::InvalidTopology(
                "an unsplit edge crosses an atomic-edge interior",
            ));
        }
        let offset = (options.tolerance * 0.25)
            .min(edge_length * 0.25)
            .min(clearance * 0.25);
        let resolution = coordinate_resolution(center);
        if !offset.is_finite() || offset <= resolution {
            return Err(BooleanError::NumericalResolution(
                "adjacent arrangement faces cannot be sampled separately",
            ));
        }
        let direction = sub(q, p);
        let normal = [-direction[1] / edge_length, direction[0] / edge_length];
        let left_sample = add(center, scale(normal, offset));
        if left_sample == center || cross_dd(direction, sub(left_sample, p)).sign() <= 0 {
            return Err(BooleanError::NumericalResolution(
                "left-face sample rounded onto its boundary",
            ));
        }

        let (left_subject_winding, left_clip_winding) =
            winding_at(left_sample, &vertices, &edges, options, stats)?;
        let right_subject_winding = left_subject_winding - edge.winding.subject;
        let right_clip_winding = left_clip_winding - edge.winding.clip;
        let left_filled = operation_value(
            operation,
            options.subject_fill_rule.contains(left_subject_winding),
            options.clip_fill_rule.contains(left_clip_winding),
        );
        let right_filled = operation_value(
            operation,
            options.subject_fill_rule.contains(right_subject_winding),
            options.clip_fill_rule.contains(right_clip_winding),
        );

        let forward = half_edges.len();
        half_edges.push(HalfEdge {
            from: edge.from,
            to: edge.to,
        });
        half_edges.push(HalfEdge {
            from: edge.to,
            to: edge.from,
        });
        outgoing[edge.from].push(forward);
        outgoing[edge.to].push(forward + 1);
        boundary.push(left_filled && !right_filled);
        boundary.push(right_filled && !left_filled);
    }

    for (vertex, list) in outgoing.iter_mut().enumerate() {
        list.sort_by(|&a, &b| {
            angle_cmp(
                sub(vertices[half_edges[a].to], vertices[vertex]),
                sub(vertices[half_edges[b].to], vertices[vertex]),
            )
            .then_with(|| half_edges[a].to.cmp(&half_edges[b].to))
        });
    }
    Ok(BoundaryGraph {
        vertices,
        half_edges,
        outgoing,
        boundary,
    })
}

impl FillRule {
    fn contains(self, winding: i64) -> bool {
        match self {
            Self::NonZero => winding != 0,
            Self::EvenOdd => winding.rem_euclid(2) == 1,
        }
    }
}

fn operation_value(operation: BooleanOperation, subject: bool, clip: bool) -> bool {
    match operation {
        BooleanOperation::Union => subject || clip,
        BooleanOperation::Intersection => subject && clip,
        BooleanOperation::Difference => subject && !clip,
        BooleanOperation::Exclusion => subject ^ clip,
    }
}

fn winding_at(
    point: Point,
    vertices: &[Point],
    edges: &[AtomicEdge],
    options: BooleanOptions,
    stats: &mut BooleanStats,
) -> Result<(i64, i64), BooleanError> {
    let mut subject = 0_i64;
    let mut clip = 0_i64;
    for edge in edges {
        bump_classification(options, stats)?;
        let p = vertices[edge.from];
        let q = vertices[edge.to];
        let side = orient(p, q, point);
        if side == 0 && point_in_bounds(point, p, q) {
            return Err(BooleanError::NumericalResolution(
                "face sample lies on another arrangement edge",
            ));
        }
        let crossing = if p[1] <= point[1] && q[1] > point[1] && side > 0 {
            1_i64
        } else if p[1] > point[1] && q[1] <= point[1] && side < 0 {
            -1_i64
        } else {
            0_i64
        };
        subject += crossing * edge.winding.subject;
        clip += crossing * edge.winding.clip;
    }
    Ok((subject, clip))
}

fn bump_classification(
    options: BooleanOptions,
    stats: &mut BooleanStats,
) -> Result<(), BooleanError> {
    if stats.classification_tests == options.limits.max_classification_tests {
        return Err(BooleanError::ResourceLimit {
            phase: BooleanPhase::Classification,
            limit: options.limits.max_classification_tests,
        });
    }
    stats.classification_tests += 1;
    Ok(())
}

impl BoundaryGraph {
    fn trace_loops(
        &self,
        options: BooleanOptions,
        stats: &mut BooleanStats,
    ) -> Result<Vec<Vec<Point>>, BooleanError> {
        let mut visited = vec![false; self.half_edges.len()];
        let mut loops = Vec::new();
        let mut traced_segments = 0_usize;
        for start in 0..self.half_edges.len() {
            if !self.boundary[start] || visited[start] {
                continue;
            }
            let mut current = start;
            let mut contour = Vec::new();
            loop {
                if visited[current] {
                    if current != start {
                        return Err(BooleanError::InvalidTopology(
                            "a boundary chain merged into a different loop",
                        ));
                    }
                    break;
                }
                if traced_segments == options.limits.max_output_segments {
                    return Err(BooleanError::ResourceLimit {
                        phase: BooleanPhase::Output,
                        limit: options.limits.max_output_segments,
                    });
                }
                traced_segments += 1;
                visited[current] = true;
                contour.push(self.vertices[self.half_edges[current].from]);
                current = self.next_boundary(current)?;
            }
            simplify_loop(&mut contour);
            if contour.len() >= 3 && signed_area(&contour) != 0.0 {
                canonicalize_loop(&mut contour);
                loops.push(contour);
            }
        }
        loops.sort_by(|a, b| {
            point_cmp(a[0], b[0])
                .then_with(|| signed_area(a).total_cmp(&signed_area(b)))
                .then_with(|| a.len().cmp(&b.len()))
        });
        stats.output_segments = loops.iter().map(Vec::len).sum();
        Ok(loops)
    }

    fn next_boundary(&self, current: usize) -> Result<usize, BooleanError> {
        let destination = self.half_edges[current].to;
        let twin = current ^ 1;
        let outgoing = &self.outgoing[destination];
        let twin_position =
            outgoing
                .iter()
                .position(|&edge| edge == twin)
                .ok_or(BooleanError::InvalidTopology(
                    "half-edge twin is absent from its destination",
                ))?;
        for step in 1..=outgoing.len() {
            let candidate = outgoing[(twin_position + outgoing.len() - step) % outgoing.len()];
            if self.boundary[candidate] {
                return Ok(candidate);
            }
        }
        Err(BooleanError::InvalidTopology(
            "a result boundary has an open endpoint",
        ))
    }
}

fn simplify_loop(points: &mut Vec<Point>) {
    if points.len() < 3 {
        return;
    }
    loop {
        let mut remove = vec![false; points.len()];
        let mut removed = false;
        for index in 0..points.len() {
            let prior = points[(index + points.len() - 1) % points.len()];
            let point = points[index];
            let next = points[(index + 1) % points.len()];
            if point == prior
                || point == next
                || (orient(prior, point, next) == 0 && point_in_bounds(point, prior, next))
            {
                remove[index] = true;
                removed = true;
            }
        }
        if !removed {
            break;
        }
        let mut index = 0;
        points.retain(|_| {
            let keep = !remove[index];
            index += 1;
            keep
        });
        if points.len() < 3 {
            break;
        }
    }
}

fn canonicalize_loop(points: &mut [Point]) {
    if let Some((start, _)) = points
        .iter()
        .enumerate()
        .min_by(|(a_index, a), (b_index, b)| {
            point_cmp(**a, **b).then_with(|| {
                let a_next = points[(*a_index + 1) % points.len()];
                let b_next = points[(*b_index + 1) % points.len()];
                point_cmp(a_next, b_next)
            })
        })
    {
        points.rotate_left(start);
    }
}

fn loops_to_path(loops: &[Vec<Point>]) -> Result<QuadPath, BooleanError> {
    let mut path = QuadPath::new();
    for contour in loops {
        path.start_new_path(to_vec3(contour[0]));
        for &point in &contour[1..] {
            path.add_line_to(to_vec3(point), true)
                .map_err(|_| BooleanError::InvalidTopology("failed to append result contour"))?;
        }
        path.add_line_to(to_vec3(contour[0]), true)
            .map_err(|_| BooleanError::InvalidTopology("failed to close result contour"))?;
    }
    Ok(path)
}

fn to_vec3(point: Point) -> Vec3 {
    [point[0], point[1], 0.0]
}

fn bbox(p: Point, q: Point) -> (f64, f64, f64, f64) {
    (
        p[0].min(q[0]),
        p[0].max(q[0]),
        p[1].min(q[1]),
        p[1].max(q[1]),
    )
}

fn point_cmp(a: Point, b: Point) -> Ordering {
    a[0].total_cmp(&b[0]).then_with(|| a[1].total_cmp(&b[1]))
}

fn angle_cmp(a: Point, b: Point) -> Ordering {
    let a_half = angle_half(a);
    let b_half = angle_half(b);
    a_half.cmp(&b_half).then_with(|| {
        let cross = cross_dd(a, b).sign();
        if cross > 0 {
            Ordering::Less
        } else if cross < 0 {
            Ordering::Greater
        } else {
            squared_length(a).total_cmp(&squared_length(b))
        }
    })
}

fn angle_half(vector: Point) -> u8 {
    u8::from(vector[1] < 0.0 || (vector[1] == 0.0 && vector[0] < 0.0))
}

fn signed_area(points: &[Point]) -> f64 {
    let mut sum = Dd::ZERO;
    for index in 0..points.len() {
        let p = points[index];
        let q = points[(index + 1) % points.len()];
        sum = sum.add(product_difference(p[0], q[1], p[1], q[0]));
    }
    0.5 * sum.value()
}

fn point_segment_distance(point: Point, p: Point, q: Point) -> f64 {
    let direction = sub(q, p);
    let denominator = squared_length(direction);
    if denominator == 0.0 {
        return length(sub(point, p));
    }
    let t = (dot(sub(point, p), direction) / denominator).clamp(0.0, 1.0);
    length(sub(point, lerp(p, q, t)))
}

fn point_on_segment(point: Point, p: Point, q: Point) -> bool {
    orient(p, q, point) == 0 && point_in_bounds(point, p, q)
}

fn point_in_bounds(point: Point, p: Point, q: Point) -> bool {
    point[0] >= p[0].min(q[0])
        && point[0] <= p[0].max(q[0])
        && point[1] >= p[1].min(q[1])
        && point[1] <= p[1].max(q[1])
}

fn segment_parameter(point: Point, p: Point, q: Point) -> f64 {
    let direction = sub(q, p);
    let axis = usize::from(direction[1].abs() > direction[0].abs());
    canonical_parameter((point[axis] - p[axis]) / direction[axis])
}

fn ratio_in_unit(numerator: Dd, denominator: Dd) -> bool {
    if denominator.sign() > 0 {
        numerator.sign() >= 0 && denominator.sub(numerator).sign() >= 0
    } else {
        numerator.sign() <= 0 && denominator.sub(numerator).sign() <= 0
    }
}

fn ratio(numerator: Dd, denominator: Dd) -> f64 {
    canonical_parameter(numerator.value() / denominator.value())
}

fn canonical_parameter(value: f64) -> f64 {
    if value <= PARAMETER_MERGE_EPSILON {
        0.0
    } else if 1.0 - value <= PARAMETER_MERGE_EPSILON {
        1.0
    } else {
        value.clamp(0.0, 1.0)
    }
}

fn orient(a: Point, b: Point, c: Point) -> i8 {
    cross_dd(sub(b, a), sub(c, a)).sign()
}

fn coordinate_resolution(point: Point) -> f64 {
    let scale = point[0].abs().max(point[1].abs());
    if scale == 0.0 {
        f64::MIN_POSITIVE
    } else {
        scale * f64::EPSILON * 256.0
    }
}

fn add(a: Point, b: Point) -> Point {
    [a[0] + b[0], a[1] + b[1]]
}

fn sub(a: Point, b: Point) -> Point {
    [a[0] - b[0], a[1] - b[1]]
}

fn scale(point: Point, factor: f64) -> Point {
    [point[0] * factor, point[1] * factor]
}

fn midpoint(a: Point, b: Point) -> Point {
    [a[0] + (b[0] - a[0]) * 0.5, a[1] + (b[1] - a[1]) * 0.5]
}

fn lerp(a: Point, b: Point, t: f64) -> Point {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}

fn dot(a: Point, b: Point) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}

fn squared_length(point: Point) -> f64 {
    dot(point, point)
}

fn length(point: Point) -> f64 {
    let scale = point[0].abs().max(point[1].abs());
    if scale == 0.0 {
        0.0
    } else {
        scale
            * fmn_dmath::sqrt(
                fmn_dmath::powi(point[0] / scale, 2) + fmn_dmath::powi(point[1] / scale, 2),
            )
    }
}

#[derive(Debug, Clone, Copy)]
struct Dd {
    hi: f64,
    lo: f64,
}

impl Dd {
    const ZERO: Self = Self { hi: 0.0, lo: 0.0 };

    fn value(self) -> f64 {
        self.hi + self.lo
    }

    fn sign(self) -> i8 {
        if self.hi > 0.0 || (self.hi == 0.0 && self.lo > 0.0) {
            1
        } else if self.hi < 0.0 || (self.hi == 0.0 && self.lo < 0.0) {
            -1
        } else {
            0
        }
    }

    fn add(self, other: Self) -> Self {
        let (sum, error) = two_sum(self.hi, other.hi);
        let (hi, lo) = two_sum(sum, self.lo + other.lo + error);
        Self { hi, lo }
    }

    fn sub(self, other: Self) -> Self {
        self.add(Self {
            hi: -other.hi,
            lo: -other.lo,
        })
    }
}

fn cross_dd(a: Point, b: Point) -> Dd {
    product_difference(a[0], b[1], a[1], b[0])
}

fn product_difference(a: f64, b: f64, c: f64, d: f64) -> Dd {
    let (ab, ab_error) = two_product(a, b);
    let (cd, cd_error) = two_product(c, d);
    let (difference, difference_error) = two_sum(ab, -cd);
    let (hi, lo) = two_sum(difference, ab_error - cd_error + difference_error);
    Dd { hi, lo }
}

fn two_sum(a: f64, b: f64) -> (f64, f64) {
    let sum = a + b;
    let b_virtual = sum - a;
    let error = (a - (sum - b_virtual)) + (b - b_virtual);
    (sum, error)
}

fn two_product(a: f64, b: f64) -> (f64, f64) {
    const SPLITTER: f64 = 134_217_729.0;
    let product = a * b;
    let a_split = SPLITTER * a;
    let a_hi = a_split - (a_split - a);
    let a_lo = a - a_hi;
    let b_split = SPLITTER * b;
    let b_hi = b_split - (b_split - b);
    let b_lo = b - b_hi;
    let error = ((a_hi * b_hi - product) + a_hi * b_lo + a_lo * b_hi) + a_lo * b_lo;
    (product, error)
}
