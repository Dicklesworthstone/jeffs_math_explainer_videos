//! The `transversal-interiors` (C1) curve-aware boolean class.
//!
//! The normative specification is `docs/geometry/curve_booleans.md`; this
//! module is its executable form. Every cross-operand piece intersection
//! must be a proper crossing strictly inside both pieces (spec §3.4),
//! every same-operand pair must meet only at shared contour endpoints,
//! and every numerical screen below is a *routing* decision: anything
//! unresolved, singular, tangent, endpoint-touching, overlapping, or
//! budget-exhausting declines the class and the caller falls through to
//! the certified flatten-and-clip Stage 1. Declining never fails the
//! operation and never coarsens output silently.
//!
//! Pipeline: piece collection (with Stage-1's closure snap) → fat-line
//! pruned Bézier clipping (bounded rounds, bounded subdivision) → bounded
//! Newton polish → transversal-interior screening → de Casteljau
//! splitting with shared-vertex snap → winding classification by ray
//! casting → the same angular-sort face traversal as Stage 1, stitched
//! over curve tangents. Nothing here claims coordinate exactness; the
//! double-double orientation and area signs shared with Stage 1 are used
//! so routing signs agree with the arrangement's own predicates.

use super::{
    BooleanOperation, BooleanOptions, BooleanResult, BooleanRoute, BooleanStats, Dd, add,
    angle_cmp, coordinate_resolution, cross_dd, dot, length, lerp, midpoint, operation_value,
    point_cmp, point_segment_distance, product_difference, scale, sub, to_vec3, xy,
};
use crate::QuadPath;

type Point = [f64; 2];

/// Open-interval margin for a transversal interior crossing (spec §3.4).
const INTERIOR_MARGIN: f64 = 1.0e-9;
/// Minimum crossing sine for a transversal crossing (spec §3.4).
const SINE_MIN: f64 = 1.0e-6;
/// Clip-interval width that counts as convergence (spec §3.2).
const CLIP_T_EPS: f64 = 1.0e-10;
/// Two roots on one piece closer than this are a near-double root (C3).
const ROOT_MERGE_EPS: f64 = 1.0e-9;
/// Documented clip-round bound per recursion node (spec §3.2).
const MAX_CLIP_ROUNDS: u32 = 24;
/// Documented bisection depth bound (spec §3.2).
const MAX_CLIP_DEPTH: u32 = 6;
/// Documented clip-node bound per piece pair (spec §3.2).
const MAX_CLIP_NODES: u32 = 512;
/// Documented Newton iteration bound (spec §3.3).
const MAX_NEWTON_ITERATIONS: u32 = 16;
/// Jacobian singularity threshold relative to derivative magnitudes (§3.3).
const JACOBIAN_SINGULAR: f64 = 1.0e-13;
/// Ray-crossing parameters within this of an endpoint make the sample ray
/// ambiguous; classification retries with a smaller offset (spec §4).
const RAY_PARAM_SNAP: f64 = 1.0e-9;
/// Classification sample retries with halved offsets (spec §4).
const SAMPLE_ATTEMPTS: u32 = 4;
/// Maximum midpoint-subdivision depth used to tighten a zero control-hull
/// clearance bound during classification (spec §4).
const MAX_CLEARANCE_DEPTH: u32 = 8;

/// One quadratic piece of an operand contour.
#[derive(Clone, Copy)]
struct Piece {
    /// 0 for the subject, 1 for the clip.
    operand: usize,
    quad: Quad,
    /// Vertex of the piece start anchor.
    start_vertex: usize,
    /// Vertex of the piece end anchor.
    end_vertex: usize,
    /// Whether the handle lies off the chord (a genuine curve).
    curved: bool,
}

/// A quadratic Bézier in the plane.
#[derive(Clone, Copy)]
struct Quad {
    p0: Point,
    p1: Point,
    p2: Point,
}

impl Quad {
    fn eval(self, t: f64) -> Point {
        let mt = 1.0 - t;
        add(
            add(scale(self.p0, mt * mt), scale(self.p1, 2.0 * mt * t)),
            scale(self.p2, t * t),
        )
    }

    fn deriv(self, t: f64) -> Point {
        let mt = 1.0 - t;
        add(
            scale(sub(self.p1, self.p0), 2.0 * mt),
            scale(sub(self.p2, self.p1), 2.0 * t),
        )
    }

    /// De Casteljau split at `t`: exact in the parameterization (spec §3.6).
    fn split(self, t: f64) -> (Self, Self) {
        let q0 = lerp(self.p0, self.p1, t);
        let q1 = lerp(self.p1, self.p2, t);
        let r = lerp(q0, q1, t);
        (
            Self {
                p0: self.p0,
                p1: q0,
                p2: r,
            },
            Self {
                p0: r,
                p1: q1,
                p2: self.p2,
            },
        )
    }

    /// The subcurve over `[a, b]` of the parameter interval.
    fn subcurve(self, a: f64, b: f64) -> Self {
        if b - a >= 1.0 {
            return self;
        }
        let (left, _) = self.split(b.clamp(0.0, 1.0));
        if b <= 0.0 {
            return left;
        }
        let (_, right) = left.split((a / b).clamp(0.0, 1.0));
        right
    }

    /// The tight quadratic fat line (spec §3.1): baseline point, unit
    /// normal, and the signed-distance band containing the curve.
    fn fat_line(self) -> (Point, Point, f64, f64) {
        let chord = sub(self.p2, self.p0);
        let chord_length = length(chord);
        if chord_length == 0.0 {
            return (self.p0, [0.0, 1.0], 0.0, 0.0);
        }
        let normal = [-chord[1] / chord_length, chord[0] / chord_length];
        let d1 = dot(normal, sub(self.p1, self.p0)) * 0.5;
        (self.p0, normal, d1.min(0.0), d1.max(0.0))
    }

    /// Signed area via the exact quadratic boundary integral (spec §5),
    /// scaled by 6: `2·cross(p0,p1) + cross(p0,p2) + 2·cross(p1,p2)`.
    fn area_times_six(self) -> Dd {
        let c01 = product_difference(self.p0[0], self.p1[1], self.p0[1], self.p1[0]);
        let c02 = product_difference(self.p0[0], self.p2[1], self.p0[1], self.p2[0]);
        let c12 = product_difference(self.p1[0], self.p2[1], self.p1[1], self.p2[0]);
        c01.add(c01).add(c02).add(c12).add(c12)
    }
}

/// Attempt the `transversal-interiors` class. `None` declines the class;
/// the caller then runs the certified fallback. Every refusal path is a
/// routing decision, never an error and never a coarsened result.
pub(super) fn try_transversal(
    subject: &QuadPath,
    clip: &QuadPath,
    operation: BooleanOperation,
    options: BooleanOptions,
) -> Option<BooleanResult> {
    if !has_nonzero_curved_area(subject) || !has_nonzero_curved_area(clip) {
        return None;
    }
    let mut vertices: Vec<Point> = Vec::new();
    let mut pieces: Vec<Piece> = Vec::new();
    for (operand, path) in [(0_usize, subject), (1_usize, clip)] {
        collect_operand(path, operand, options, &mut vertices, &mut pieces)?;
    }
    if !pieces.iter().any(|piece| piece.curved) {
        // All-line inputs are exact under Stage 1 and stay there (spec §1).
        return None;
    }
    let scene_scale = pieces
        .iter()
        .flat_map(|piece| [piece.quad.p0, piece.quad.p1, piece.quad.p2])
        .fold(1.0_f64, |acc, p| acc.max(p[0].abs()).max(p[1].abs()));

    let mut stats = BooleanStats {
        input_curves: subject.num_curves().saturating_add(clip.num_curves()),
        ..BooleanStats::default()
    };
    let intersections =
        solve_intersections(&pieces, scene_scale, options, &mut stats, &mut vertices)?;
    let edges = split_into_atomic_edges(&pieces, &intersections, &vertices)?;
    let graph = classify(&vertices, &edges, operation, options, &mut stats)?;
    let loops = graph.trace_loops(options, &mut stats)?;
    let path = emit_path(&loops)?;

    stats.output_contours = loops.len();
    Some(BooleanResult {
        path,
        route: BooleanRoute::CurveAwareTransversal,
        stats,
    })
}

/// C1's nonzero-area admission screen (spec §1): the exact quadratic
/// boundary integral per contour, double-double accumulated. The
/// anchor-shoelace proxy used by the separated class is insufficient
/// here: a two-anchor lens (both pieces curved, anchors collinear)
/// encloses genuine area the proxy reports as zero.
fn has_nonzero_curved_area(path: &QuadPath) -> bool {
    path.subpaths().into_iter().any(|subpath| {
        if subpath.len() < 3 {
            return false;
        }
        let mut area = Dd::ZERO;
        for index in (0..subpath.len().saturating_sub(2)).step_by(2) {
            area = area.add(
                Quad {
                    p0: xy(subpath[index]),
                    p1: xy(subpath[index + 1]),
                    p2: xy(subpath[index + 2]),
                }
                .area_times_six(),
            );
        }
        let first = subpath[0];
        let last = subpath[subpath.len() - 1];
        if first != last {
            // The implicit fill-closing chord contributes its straight
            // area (three sixths of the cross product), mirroring the
            // closing piece the collector appends.
            let chord = product_difference(first[0], last[1], first[1], last[0]);
            area = area.add(chord).add(chord).add(chord);
        }
        area.sign() != 0
    })
}

/// Collect an operand's contours into pieces and anchor vertices,
/// applying the closure snap and every C5 degenerate-piece refusal.
fn collect_operand(
    path: &QuadPath,
    operand: usize,
    options: BooleanOptions,
    vertices: &mut Vec<Point>,
    pieces: &mut Vec<Piece>,
) -> Option<()> {
    let operand_start = pieces.len();
    for subpath in path.subpaths() {
        if subpath.len() < 3 {
            continue;
        }
        let anchors: Vec<Point> = subpath.iter().step_by(2).map(|&p| xy(p)).collect();
        let handles: Vec<Point> = subpath.iter().skip(1).step_by(2).map(|&p| xy(p)).collect();
        let last = anchors.len() - 1;
        let snapped = length(sub(anchors[last], anchors[0])) <= options.tolerance;
        let cycle_len = if snapped { last } else { anchors.len() };
        if cycle_len == 0 {
            return None;
        }
        let cycle: Vec<usize> = (0..cycle_len)
            .map(|index| {
                vertices.push(anchors[index]);
                vertices.len() - 1
            })
            .collect();
        for index in 0..last {
            let end = (index + 1) % cycle_len;
            push_piece(
                operand,
                Quad {
                    p0: anchors[index],
                    p1: handles[index],
                    p2: if end == 0 {
                        anchors[0]
                    } else {
                        anchors[index + 1]
                    },
                },
                cycle[index],
                cycle[end],
                pieces,
            )?;
        }
        if !snapped {
            push_piece(
                operand,
                Quad {
                    p0: anchors[last],
                    p1: midpoint(anchors[last], anchors[0]),
                    p2: anchors[0],
                },
                cycle[last],
                cycle[0],
                pieces,
            )?;
        }
    }
    // Pinch points and self-touching contours are C2/C5: within one
    // operand every anchor must be distinct (the closure identification
    // is already folded into the cycle).
    let mut starts: Vec<Point> = pieces[operand_start..]
        .iter()
        .map(|piece| piece.quad.p0)
        .collect();
    starts.sort_by(|a, b| point_cmp(*a, *b));
    if starts.windows(2).any(|pair| pair[0] == pair[1]) {
        return None;
    }
    Some(())
}

fn push_piece(
    operand: usize,
    quad: Quad,
    start_vertex: usize,
    end_vertex: usize,
    pieces: &mut Vec<Piece>,
) -> Option<()> {
    if quad.p0 == quad.p1 || quad.p1 == quad.p2 || quad.p0 == quad.p2 {
        return None;
    }
    pieces.push(Piece {
        operand,
        quad,
        start_vertex,
        end_vertex,
        curved: is_genuinely_curved(quad),
    });
    Some(())
}

/// A curve must leave its chord by more than the controls' coordinate
/// resolution to satisfy C1's curved-piece admission screen. Line builders
/// and affine transforms can leave an ulp-scale off-chord residue; treating
/// that residue as a curve would make an all-line path's route depend on its
/// translation. The screen is conservative because a declined piece still
/// reaches the certified Stage 1 implementation.
fn is_genuinely_curved(quad: Quad) -> bool {
    let chord = sub(quad.p2, quad.p0);
    let chord_length = length(chord);
    if chord_length == 0.0 {
        return false;
    }
    let resolution = coordinate_resolution(quad.p0)
        .max(coordinate_resolution(quad.p1))
        .max(coordinate_resolution(quad.p2));
    cross_dd(sub(quad.p1, quad.p0), chord).value().abs() > resolution * chord_length
}

/// One proved transversal interior crossing, shared by both pieces.
#[derive(Clone, Copy)]
struct Intersection {
    piece_a: usize,
    t_a: f64,
    piece_b: usize,
    t_b: f64,
    vertex: usize,
}

/// Solve every piece pair: same-operand pairs must meet only at shared
/// endpoints; cross-operand roots must all pass the C1 screen. Any
/// solver failure, tangency, endpoint touch, overlap, or budget
/// exhaustion declines the class.
fn solve_intersections(
    pieces: &[Piece],
    scene_scale: f64,
    options: BooleanOptions,
    stats: &mut BooleanStats,
    vertices: &mut Vec<Point>,
) -> Option<Vec<Intersection>> {
    let mut intersections = Vec::new();
    for i in 0..pieces.len() {
        for j in i + 1..pieces.len() {
            if !boxes_overlap(pieces[i].quad, pieces[j].quad) {
                continue;
            }
            bump(&mut stats.pair_tests, options.limits.max_pair_tests)?;
            let roots = clip_pair(&pieces[i].quad, &pieces[j].quad, options, stats)?;
            if pieces[i].operand == pieces[j].operand {
                if roots
                    .iter()
                    .any(|&(t, u)| !at_shared_endpoint(&pieces[i].quad, &pieces[j].quad, t, u))
                {
                    return None;
                }
                continue;
            }
            for (t_seed, u_seed) in roots {
                let (t, u) = polish(
                    &pieces[i].quad,
                    &pieces[j].quad,
                    t_seed,
                    u_seed,
                    scene_scale,
                )?;
                if !is_transversal_interior(&pieces[i].quad, &pieces[j].quad, t, u) {
                    return None;
                }
                bump(&mut stats.intersections, options.limits.max_intersections)?;
                vertices.push(midpoint(pieces[i].quad.eval(t), pieces[j].quad.eval(u)));
                intersections.push(Intersection {
                    piece_a: i,
                    t_a: t,
                    piece_b: j,
                    t_b: u,
                    vertex: vertices.len() - 1,
                });
            }
        }
    }
    // A near-double root on one piece is C3 territory: any two roots on
    // the same piece closer than the merge band decline the class.
    for piece in 0..pieces.len() {
        let mut params: Vec<f64> = intersections
            .iter()
            .filter_map(|hit| {
                if hit.piece_a == piece {
                    Some(hit.t_a)
                } else if hit.piece_b == piece {
                    Some(hit.t_b)
                } else {
                    None
                }
            })
            .collect();
        params.sort_by(f64::total_cmp);
        if params
            .windows(2)
            .any(|pair| pair[1] - pair[0] < ROOT_MERGE_EPS)
        {
            return None;
        }
    }
    Some(intersections)
}

fn boxes_overlap(a: Quad, b: Quad) -> bool {
    let (a_min_x, a_max_x, a_min_y, a_max_y) = quad_box(a);
    let (b_min_x, b_max_x, b_min_y, b_max_y) = quad_box(b);
    a_min_x <= b_max_x && b_min_x <= a_max_x && a_min_y <= b_max_y && b_min_y <= a_max_y
}

fn quad_box(q: Quad) -> (f64, f64, f64, f64) {
    (
        q.p0[0].min(q.p1[0]).min(q.p2[0]),
        q.p0[0].max(q.p1[0]).max(q.p2[0]),
        q.p0[1].min(q.p1[1]).min(q.p2[1]),
        q.p0[1].max(q.p1[1]).max(q.p2[1]),
    )
}

/// Every root of a same-operand pair must sit at an endpoint the two
/// pieces share (a contour joint).
fn at_shared_endpoint(a: &Quad, b: &Quad, t: f64, u: f64) -> bool {
    [(0.0, a.p0), (1.0, a.p2)].iter().any(|&(ta, pa)| {
        [(0.0, b.p0), (1.0, b.p2)].iter().any(|&(tb, pb)| {
            pa == pb && (t - ta).abs() <= INTERIOR_MARGIN && (u - tb).abs() <= INTERIOR_MARGIN
        })
    })
}

/// The C1 acceptance predicate (spec §3.4): strict interior parameters,
/// nonzero derivatives, crossing sine bounded away from zero.
fn is_transversal_interior(a: &Quad, b: &Quad, t: f64, u: f64) -> bool {
    if t < INTERIOR_MARGIN
        || t > 1.0 - INTERIOR_MARGIN
        || u < INTERIOR_MARGIN
        || u > 1.0 - INTERIOR_MARGIN
    {
        return false;
    }
    let da = a.deriv(t);
    let db = b.deriv(u);
    let la = length(da);
    let lb = length(db);
    if la == 0.0 || lb == 0.0 {
        return false;
    }
    cross_dd(da, db).value().abs() >= SINE_MIN * la * lb
}

/// Fat-line pruned Bézier clipping (spec §3.1, §3.2): bounded rounds,
/// bounded bisection, bounded nodes. `None` means unresolved, which
/// declines the class.
fn clip_pair(
    a: &Quad,
    b: &Quad,
    options: BooleanOptions,
    stats: &mut BooleanStats,
) -> Option<Vec<(f64, f64)>> {
    let mut nodes = 0_u32;
    let mut roots = Vec::new();
    clip_recurse(
        a,
        b,
        (0.0, 1.0),
        (0.0, 1.0),
        0,
        &mut nodes,
        &mut roots,
        options,
        stats,
    )?;
    roots.sort_by(|x, y| x.0.total_cmp(&y.0).then_with(|| x.1.total_cmp(&y.1)));
    roots.dedup_by(|later, earlier| {
        if (later.0 - earlier.0).abs() <= ROOT_MERGE_EPS
            && (later.1 - earlier.1).abs() <= ROOT_MERGE_EPS
        {
            earlier.0 = (earlier.0 + later.0) * 0.5;
            earlier.1 = (earlier.1 + later.1) * 0.5;
            true
        } else {
            false
        }
    });
    Some(roots)
}

#[allow(clippy::too_many_arguments)]
fn clip_recurse(
    a: &Quad,
    b: &Quad,
    interval_a: (f64, f64),
    interval_b: (f64, f64),
    depth: u32,
    nodes: &mut u32,
    roots: &mut Vec<(f64, f64)>,
    options: BooleanOptions,
    stats: &mut BooleanStats,
) -> Option<()> {
    *nodes += 1;
    if *nodes > MAX_CLIP_NODES {
        return None;
    }
    bump(&mut stats.pair_tests, options.limits.max_pair_tests)?;
    let mut ta = interval_a;
    let mut tb = interval_b;
    let mut prev_width_a = ta.1 - ta.0;
    let mut prev_width_b = tb.1 - tb.0;
    let mut stalls = 0_u32;
    for _round in 0..MAX_CLIP_ROUNDS {
        let b_sub = b.subcurve(tb.0, tb.1);
        let (base, normal, dmin, dmax) = b_sub.fat_line();
        let Some(clip_a) = clip_to_band(&a.subcurve(ta.0, ta.1), base, normal, dmin, dmax) else {
            // This recursion branch provably misses the other curve's
            // fat line. It contributes no roots, but is not a solver
            // failure and must not decline the entire piece pair.
            return Some(());
        };
        ta = (
            ta.0 + clip_a.0 * (ta.1 - ta.0),
            ta.0 + clip_a.1 * (ta.1 - ta.0),
        );
        if ta.0 == ta.1 {
            // The clip collapsed to one parameter: resolve the contact
            // exactly instead of iterating a degenerate subcurve.
            let point = a.eval(ta.0);
            roots.extend(point_on_quad(b, point, tb).into_iter().map(|u| (ta.0, u)));
            return Some(());
        }
        let a_sub = a.subcurve(ta.0, ta.1);
        let (base, normal, dmin, dmax) = a_sub.fat_line();
        let Some(clip_b) = clip_to_band(&b.subcurve(tb.0, tb.1), base, normal, dmin, dmax) else {
            return Some(());
        };
        tb = (
            tb.0 + clip_b.0 * (tb.1 - tb.0),
            tb.0 + clip_b.1 * (tb.1 - tb.0),
        );
        if tb.0 == tb.1 {
            let point = b.eval(tb.0);
            roots.extend(point_on_quad(a, point, ta).into_iter().map(|t| (t, tb.0)));
            return Some(());
        }
        let width_a = ta.1 - ta.0;
        let width_b = tb.1 - tb.0;
        if width_a <= CLIP_T_EPS && width_b <= CLIP_T_EPS {
            roots.push(((ta.0 + ta.1) * 0.5, (tb.0 + tb.1) * 0.5));
            return Some(());
        }
        let shrank = width_a < 0.8 * prev_width_a && width_b < 0.8 * prev_width_b;
        if shrank {
            stalls = 0;
        } else {
            stalls += 1;
        }
        prev_width_a = width_a;
        prev_width_b = width_b;
        if stalls >= 2 {
            if depth >= MAX_CLIP_DEPTH {
                return None;
            }
            if width_a >= width_b {
                let mid = (ta.0 + ta.1) * 0.5;
                if mid <= ta.0 || mid >= ta.1 {
                    return None;
                }
                clip_recurse(
                    a,
                    b,
                    (ta.0, mid),
                    tb,
                    depth + 1,
                    nodes,
                    roots,
                    options,
                    stats,
                )?;
                return clip_recurse(
                    a,
                    b,
                    (mid, ta.1),
                    tb,
                    depth + 1,
                    nodes,
                    roots,
                    options,
                    stats,
                );
            }
            let mid = (tb.0 + tb.1) * 0.5;
            if mid <= tb.0 || mid >= tb.1 {
                return None;
            }
            clip_recurse(
                a,
                b,
                ta,
                (tb.0, mid),
                depth + 1,
                nodes,
                roots,
                options,
                stats,
            )?;
            return clip_recurse(
                a,
                b,
                ta,
                (mid, tb.1),
                depth + 1,
                nodes,
                roots,
                options,
                stats,
            );
        }
    }
    None
}

/// Append the numerically stable real roots of `a·t² + b·t + c`.
fn append_quadratic_roots(a: f64, b: f64, c: f64, roots: &mut Vec<f64>) {
    if a == 0.0 {
        if b != 0.0 {
            roots.push(-c / b);
        }
        return;
    }

    let discriminant = b * b - 4.0 * a * c;
    let discriminant_tolerance = 64.0 * f64::EPSILON * (b * b).abs().max((4.0 * a * c).abs());
    if discriminant < -discriminant_tolerance {
        return;
    }
    let root = fmn_dmath::sqrt(discriminant.max(0.0));
    if root == 0.0 {
        roots.push(-b / (2.0 * a));
        return;
    }

    // Choose the sum with like-signed terms, then obtain the other root
    // from their product. This avoids cancelling `-b` against `sqrt(D)`
    // when one root is much smaller than the other.
    let q = -0.5 * (b + root.copysign(b));
    if q == 0.0 {
        roots.push(-b / (2.0 * a));
    } else {
        roots.push(q / a);
        roots.push(c / q);
    }
}

/// The convex hull of `{ t ∈ [0,1] : dmin ≤ E(t) ≤ dmax }` for the
/// distance polynomial with Bernstein coefficients `(e0, e1, e2)`.
/// `None` means the band is missed entirely (fat-line pruning).
fn clip_to_band(q: &Quad, base: Point, normal: Point, dmin: f64, dmax: f64) -> Option<(f64, f64)> {
    let e0 = dot(normal, sub(q.p0, base));
    let e1 = dot(normal, sub(q.p1, base));
    let e2 = dot(normal, sub(q.p2, base));
    let band_scale = e0
        .abs()
        .max(e1.abs())
        .max(e2.abs())
        .max(dmin.abs())
        .max(dmax.abs())
        .max(f64::MIN_POSITIVE);
    let band_tolerance = 64.0 * f64::EPSILON * band_scale;
    let mut cuts = Vec::with_capacity(6);
    cuts.extend([0.0, 1.0]);
    for bound in [dmin, dmax] {
        let a = e0 - 2.0 * e1 + e2;
        let b = 2.0 * (e1 - e0);
        let c = e0 - bound;
        append_quadratic_roots(a, b, c, &mut cuts);
    }
    cuts.sort_by(f64::total_cmp);
    let mut interval: Option<(f64, f64)> = None;
    for pair in cuts.windows(2) {
        let mid = (pair[0] + pair[1]) * 0.5;
        if !(0.0..=1.0).contains(&mid) {
            continue;
        }
        let mt = 1.0 - mid;
        let e = e0 * mt * mt + 2.0 * e1 * mt * mid + e2 * mid * mid;
        if e >= dmin - band_tolerance && e <= dmax + band_tolerance {
            let lo = pair[0].max(0.0);
            let hi = pair[1].min(1.0);
            interval = Some(match interval {
                Some((l, h)) => (l.min(lo), h.max(hi)),
                None => (lo, hi),
            });
        }
    }
    // A contact at an isolated parameter (an endpoint touch or a
    // tangency against the band edge) has measure zero: no window
    // midpoint ever samples it. Pruning must stay conservative, so
    // in-band cut points themselves extend the hull (spec §3.2); the
    // collapsed interval is resolved exactly by the caller.
    for &cut in &cuts {
        // An exact endpoint root can round to just outside [0, 1]
        // (sqrt(b*b) is not exactly |b|); admit it within the root-merge
        // band and clamp, or endpoint contacts silently escape the hull.
        if !(-ROOT_MERGE_EPS..=1.0 + ROOT_MERGE_EPS).contains(&cut) {
            continue;
        }
        let cut = cut.clamp(0.0, 1.0);
        let mt = 1.0 - cut;
        let e = e0 * mt * mt + 2.0 * e1 * mt * cut + e2 * cut * cut;
        if e >= dmin - band_tolerance && e <= dmax + band_tolerance {
            interval = Some(match interval {
                Some((l, h)) => (l.min(cut), h.max(cut)),
                None => (cut, cut),
            });
        }
    }
    interval
}

/// Parameters in `range` where `q` evaluates to `point`: a bounded
/// quadratic solve used when clipping collapses an interval to a single
/// parameter (spec §3.2). The residual screen is conservative; any false
/// candidate still has to pass the stricter Newton and C1 screens.
fn point_on_quad(q: &Quad, point: Point, range: (f64, f64)) -> Vec<f64> {
    const PARAM_EPS: f64 = ROOT_MERGE_EPS;
    let scale = [q.p0, q.p1, q.p2, point]
        .into_iter()
        .fold(1.0_f64, |acc, p| acc.max(p[0].abs()).max(p[1].abs()));
    let tolerance = 256.0 * f64::EPSILON * scale;

    enum AxisRoots {
        Any,
        Finite(Vec<f64>),
        Miss,
    }

    // Roots of `q_axis(u) - point_axis = 0`. An identically-zero
    // coordinate imposes no constraint; a nonzero constant proves a miss.
    let solve_axis = |axis: usize| -> AxisRoots {
        let c0 = q.p0[axis] - point[axis];
        let c1 = q.p1[axis] - point[axis];
        let c2 = q.p2[axis] - point[axis];
        let a = c0 - 2.0 * c1 + c2;
        let b = 2.0 * (c1 - c0);
        if a == 0.0 && b == 0.0 {
            return if c0.abs() <= tolerance {
                AxisRoots::Any
            } else {
                AxisRoots::Miss
            };
        }
        let mut roots = Vec::with_capacity(2);
        append_quadratic_roots(a, b, c0, &mut roots);
        if roots.is_empty() {
            return AxisRoots::Miss;
        }
        AxisRoots::Finite(roots)
    };
    let residual = |u: f64| length(sub(q.eval(u), point));
    let mut candidates: Vec<f64> = Vec::new();
    let x_roots = solve_axis(0);
    let y_roots = solve_axis(1);
    match (&x_roots, &y_roots) {
        (AxisRoots::Miss, _) | (_, AxisRoots::Miss) => {}
        (AxisRoots::Any, AxisRoots::Any) => candidates.push((range.0 + range.1) * 0.5),
        (AxisRoots::Finite(xs), AxisRoots::Any) => candidates.extend(xs.iter().copied()),
        (AxisRoots::Any, AxisRoots::Finite(ys)) => candidates.extend(ys.iter().copied()),
        (AxisRoots::Finite(xs), AxisRoots::Finite(ys)) => {
            candidates.extend(xs.iter().chain(ys).copied());
        }
    }
    candidates.retain_mut(|u| {
        if *u >= -PARAM_EPS && *u <= 1.0 + PARAM_EPS {
            *u = u.clamp(0.0, 1.0);
            *u >= range.0 - PARAM_EPS && *u <= range.1 + PARAM_EPS && residual(*u) <= tolerance
        } else {
            false
        }
    });
    candidates.sort_by(f64::total_cmp);
    candidates.dedup_by(|later, earlier| {
        if (*later - *earlier).abs() <= ROOT_MERGE_EPS {
            *earlier = (*earlier + *later) * 0.5;
            true
        } else {
            false
        }
    });
    candidates
}

/// Bounded Newton polish on `F(t, u) = A(t) − B(u)` against the
/// original pieces (spec §3.3). `None` refuses singular, divergent, or
/// non-converging roots.
fn polish(a: &Quad, b: &Quad, t_seed: f64, u_seed: f64, scene_scale: f64) -> Option<(f64, f64)> {
    let tolerance = 4.0 * f64::EPSILON * scene_scale;
    let mut t = t_seed.clamp(0.0, 1.0);
    let mut u = u_seed.clamp(0.0, 1.0);
    for _iteration in 0..MAX_NEWTON_ITERATIONS {
        let f = sub(a.eval(t), b.eval(u));
        if length(f) <= tolerance {
            return Some((t, u));
        }
        let da = a.deriv(t);
        let db = b.deriv(u);
        let det = cross_dd(da, db).value();
        if det.abs() <= JACOBIAN_SINGULAR * length(da) * length(db) {
            return None;
        }
        let dt = -cross_dd(f, db).value() / det;
        let du = -cross_dd(f, da).value() / det;
        t += dt;
        u += du;
        if !(0.0..=1.0).contains(&t) || !(0.0..=1.0).contains(&u) {
            return None;
        }
        if dt == 0.0 && du == 0.0 {
            return (length(sub(a.eval(t), b.eval(u))) <= tolerance).then_some((t, u));
        }
    }
    (length(sub(a.eval(t), b.eval(u))) <= tolerance).then_some((t, u))
}

/// One curve edge of the arrangement: a split piece between two vertices.
#[derive(Clone, Copy)]
struct AtomicEdge {
    operand: usize,
    quad: Quad,
    from: usize,
    to: usize,
}

/// Split every piece at its intersection parameters; split anchors snap
/// to the shared vertex (spec §3.6).
fn split_into_atomic_edges(
    pieces: &[Piece],
    intersections: &[Intersection],
    vertices: &[Point],
) -> Option<Vec<AtomicEdge>> {
    let mut edges = Vec::new();
    for (index, piece) in pieces.iter().enumerate() {
        let mut marks: Vec<(f64, usize)> = intersections
            .iter()
            .filter_map(|hit| {
                if hit.piece_a == index {
                    Some((hit.t_a, hit.vertex))
                } else if hit.piece_b == index {
                    Some((hit.t_b, hit.vertex))
                } else {
                    None
                }
            })
            .collect();
        marks.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut t_prev = 0.0;
        let mut from = piece.start_vertex;
        for &(t, vertex) in &marks {
            push_atomic_edge(piece, t_prev, t, from, vertex, vertices, &mut edges)?;
            t_prev = t;
            from = vertex;
        }
        push_atomic_edge(
            piece,
            t_prev,
            1.0,
            from,
            piece.end_vertex,
            vertices,
            &mut edges,
        )?;
    }
    Some(edges)
}

fn push_atomic_edge(
    piece: &Piece,
    t0: f64,
    t1: f64,
    from: usize,
    to: usize,
    vertices: &[Point],
    edges: &mut Vec<AtomicEdge>,
) -> Option<()> {
    let mut quad = piece.quad.subcurve(t0, t1);
    quad.p0 = vertices[from];
    quad.p2 = vertices[to];
    if quad.p0 == quad.p1 || quad.p1 == quad.p2 || quad.p0 == quad.p2 {
        // Resolution collapse of a split interval (spec §3.6 refuses
        // what cannot be represented).
        return None;
    }
    edges.push(AtomicEdge {
        operand: piece.operand,
        quad,
        from,
        to,
    });
    Some(())
}

/// The boundary half-edge graph over curve edges.
struct CurveGraph<'a> {
    edges: &'a [AtomicEdge],
    boundary: Vec<bool>,
    outgoing: Vec<Vec<usize>>,
}

/// Winding classification (spec §4): left-face samples with hull
/// clearance, ray casting over atomic edges, boundary selection by the
/// operation's truth value on both sides.
fn classify<'a>(
    vertices: &[Point],
    edges: &'a [AtomicEdge],
    operation: BooleanOperation,
    options: BooleanOptions,
    stats: &mut BooleanStats,
) -> Option<CurveGraph<'a>> {
    let mut boundary = vec![false; edges.len() * 2];
    let mut outgoing = vec![Vec::new(); vertices.len()];
    for (index, edge) in edges.iter().enumerate() {
        let direction = edge.quad.deriv(0.5);
        let direction_length = length(direction);
        let edge_length = length(sub(edge.quad.p2, edge.quad.p0));
        if edge_length == 0.0 || direction_length == 0.0 {
            return None;
        }
        let mid = edge.quad.eval(0.5);
        let mut clearance = edge_length;
        for (other_index, other) in edges.iter().enumerate() {
            if index == other_index {
                continue;
            }
            bump(
                &mut stats.classification_tests,
                options.limits.max_classification_tests,
            )?;
            let distance = point_quad_hull_distance(mid, other.quad, MAX_CLEARANCE_DEPTH);
            clearance = clearance.min(distance);
        }
        let base_offset = options.tolerance.min(edge_length).min(clearance) * 0.25;
        if !base_offset.is_finite() || base_offset <= coordinate_resolution(mid) {
            return None;
        }
        let normal = [
            -direction[1] / direction_length,
            direction[0] / direction_length,
        ];
        let tangent = [
            direction[0] / direction_length,
            direction[1] / direction_length,
        ];
        let mut left_winding = None;
        let mut offset = base_offset;
        for attempt in 0..SAMPLE_ATTEMPTS {
            // Halving alone cannot move the ray height when the mid
            // normal is horizontal; a tiny deterministic tangential
            // nudge turns the retry into a new ray while staying on
            // the left face (the cross-product sign is unchanged) and
            // inside the clearance ball around the midpoint (spec §4).
            let nudge = f64::from(attempt) * 1.0e-3;
            let sample = add(
                mid,
                add(scale(normal, offset), scale(tangent, offset * nudge)),
            );
            if sample == mid || cross_dd(direction, sub(sample, mid)).sign() <= 0 {
                return None;
            }
            if let Some(winding) = winding_at(sample, edges, options, stats)? {
                left_winding = Some(winding);
                break;
            }
            offset *= 0.5;
        }
        let (left_subject, left_clip) = left_winding?;
        let (right_subject, right_clip) = if edge.operand == 0 {
            (left_subject - 1, left_clip)
        } else {
            (left_subject, left_clip - 1)
        };
        let left_filled = operation_value(
            operation,
            options.subject_fill_rule.contains(left_subject),
            options.clip_fill_rule.contains(left_clip),
        );
        let right_filled = operation_value(
            operation,
            options.subject_fill_rule.contains(right_subject),
            options.clip_fill_rule.contains(right_clip),
        );
        boundary[index * 2] = left_filled && !right_filled;
        boundary[index * 2 + 1] = right_filled && !left_filled;
        outgoing[edge.from].push(index * 2);
        outgoing[edge.to].push(index * 2 + 1);
    }
    for (vertex, list) in outgoing.iter_mut().enumerate() {
        list.sort_by(|&a, &b| {
            angle_cmp(
                departure_tangent(edges, a, vertex),
                departure_tangent(edges, b, vertex),
            )
            .then_with(|| a.cmp(&b))
        });
    }
    Some(CurveGraph {
        edges,
        boundary,
        outgoing,
    })
}

/// The direction in which a half-edge leaves `vertex`: the curve
/// tangent at the shared endpoint.
fn departure_tangent(edges: &[AtomicEdge], half_edge: usize, vertex: usize) -> Point {
    let edge = edges[half_edge / 2];
    if half_edge.is_multiple_of(2) {
        debug_assert!(edge.from == vertex);
        sub(edge.quad.p1, edge.quad.p0)
    } else {
        debug_assert!(edge.to == vertex);
        sub(edge.quad.p1, edge.quad.p2)
    }
}

/// Winding counts at a ray-cast sample (spec §4). Roots use the
/// half-open interval `[0, 1)`; `Ok(None)` means the ray is ambiguous
/// (on-edge crossing or zero derivative at a root) and the caller
/// retries with a new offset. Outer `None` declines the class (budget).
fn winding_at(
    sample: Point,
    edges: &[AtomicEdge],
    options: BooleanOptions,
    stats: &mut BooleanStats,
) -> Option<Option<(i64, i64)>> {
    let mut subject = 0_i64;
    let mut clip = 0_i64;
    let resolution = coordinate_resolution(sample);
    for edge in edges {
        bump(
            &mut stats.classification_tests,
            options.limits.max_classification_tests,
        )?;
        let y0 = edge.quad.p0[1] - sample[1];
        let y1 = edge.quad.p1[1] - sample[1];
        let y2 = edge.quad.p2[1] - sample[1];
        let a = y0 - 2.0 * y1 + y2;
        let b = 2.0 * (y1 - y0);
        let mut roots = Vec::new();
        if a == 0.0 {
            if b != 0.0 {
                roots.push(-y0 / b);
            }
        } else {
            let discriminant = b * b - 4.0 * a * y0;
            let discriminant_tolerance =
                64.0 * f64::EPSILON * (b * b).abs().max((4.0 * a * y0).abs());
            if discriminant < 0.0 && discriminant >= -discriminant_tolerance {
                // A ray grazing an edge apex within rounding is
                // ambiguous: never miscount it, retry elsewhere.
                return Some(None);
            }
            if discriminant > 0.0 {
                let root = fmn_dmath::sqrt(discriminant);
                let q = -0.5 * (b + root.copysign(b));
                if q == 0.0 {
                    roots.push(-b / (2.0 * a));
                } else {
                    roots.push(q / a);
                    roots.push(y0 / q);
                }
            } else if discriminant == 0.0 {
                roots.push(-b / (2.0 * a));
            }
        }
        for t in roots {
            if !(-RAY_PARAM_SNAP..=1.0 + RAY_PARAM_SNAP).contains(&t) {
                continue;
            }
            let t = t.clamp(0.0, 1.0);
            let x = edge.quad.eval(t)[0];
            if x < sample[0] - resolution {
                // Winding uses the ray from the sample toward +x. A
                // contact strictly behind its origin cannot affect the
                // count, so its endpoint/tangency status is irrelevant.
                continue;
            }
            if (x - sample[0]).abs() <= resolution {
                return Some(None);
            }
            if t > 1.0 - RAY_PARAM_SNAP {
                // The shared vertex belongs to the next atomic edge.
                continue;
            }
            let dy = edge.quad.deriv(t)[1];
            let dy_scale = (y1 - y0).abs() + (y2 - y1).abs();
            if dy.abs() <= 64.0 * f64::EPSILON * dy_scale {
                // A root with a vanishing vertical derivative is a
                // tangent ray: ambiguous, retry with a new offset.
                return Some(None);
            }
            if x > sample[0] {
                let crossing = if dy > 0.0 { 1 } else { -1 };
                if edge.operand == 0 {
                    subject += crossing;
                } else {
                    clip += crossing;
                }
            }
        }
    }
    Some(Some((subject, clip)))
}

/// Lower bound on the distance from `point` to a curve: the distance to
/// its control triangle (spec §4).
fn point_triangle_distance(point: Point, q: &Quad) -> f64 {
    let triangle_degenerate = cross_dd(sub(q.p1, q.p0), sub(q.p2, q.p0)).sign() == 0;
    let o01 = cross_dd(sub(q.p1, q.p0), sub(point, q.p0)).sign();
    let o12 = cross_dd(sub(q.p2, q.p1), sub(point, q.p1)).sign();
    let o20 = cross_dd(sub(q.p0, q.p2), sub(point, q.p2)).sign();
    if !triangle_degenerate
        && ((o01 >= 0 && o12 >= 0 && o20 >= 0) || (o01 <= 0 && o12 <= 0 && o20 <= 0))
    {
        return 0.0;
    }
    point_segment_distance(point, q.p0, q.p1)
        .min(point_segment_distance(point, q.p1, q.p2))
        .min(point_segment_distance(point, q.p2, q.p0))
}

/// Lower-bound the point-to-curve distance by recursively tightening its
/// control hull. A point can lie inside a broad quadratic control triangle
/// while remaining well clear of the curve; de Casteljau child hulls still
/// contain the complete curve and therefore preserve the lower-bound proof.
/// The fixed depth keeps the routing screen bounded, and an unresolved zero
/// simply makes classification decline.
fn point_quad_hull_distance(point: Point, q: Quad, depth: u32) -> f64 {
    let distance = point_triangle_distance(point, &q);
    if distance > 0.0 || depth == 0 {
        return distance;
    }
    let (left, right) = q.split(0.5);
    point_quad_hull_distance(point, left, depth - 1).min(point_quad_hull_distance(
        point,
        right,
        depth - 1,
    ))
}

/// A traced result loop: oriented curve pieces, end to start.
struct Loop {
    quads: Vec<Quad>,
    area: f64,
}

impl CurveGraph<'_> {
    /// Face traversal: the same next-before-the-twin rule as Stage 1
    /// (spec §5), over tangent-sorted outgoing half-edges.
    fn trace_loops(&self, options: BooleanOptions, stats: &mut BooleanStats) -> Option<Vec<Loop>> {
        let mut visited = vec![false; self.boundary.len()];
        let mut loops = Vec::new();
        let mut traced = 0_usize;
        for start in 0..self.boundary.len() {
            if !self.boundary[start] || visited[start] {
                continue;
            }
            let mut current = start;
            let mut quads = Vec::new();
            loop {
                if visited[current] {
                    if current != start {
                        return None;
                    }
                    break;
                }
                if traced == options.limits.max_output_segments {
                    return None;
                }
                traced += 1;
                visited[current] = true;
                let edge = self.edges[current / 2];
                quads.push(if current.is_multiple_of(2) {
                    edge.quad
                } else {
                    Quad {
                        p0: edge.quad.p2,
                        p1: edge.quad.p1,
                        p2: edge.quad.p0,
                    }
                });
                current = self.next_boundary(current)?;
            }
            if quads.len() < 2 {
                return None;
            }
            let area = quads
                .iter()
                .fold(Dd::ZERO, |acc, q| acc.add(q.area_times_six()));
            if area.sign() == 0 {
                return None;
            }
            loops.push(Loop {
                quads,
                area: area.value() / 6.0,
            });
        }
        stats.output_segments = loops.iter().map(|l| l.quads.len()).sum();
        loops.sort_by(|a, b| {
            point_cmp(a.quads[0].p0, b.quads[0].p0)
                .then_with(|| a.area.total_cmp(&b.area))
                .then_with(|| a.quads.len().cmp(&b.quads.len()))
        });
        Some(loops)
    }

    fn next_boundary(&self, current: usize) -> Option<usize> {
        let edge = self.edges[current / 2];
        let destination = if current.is_multiple_of(2) {
            edge.to
        } else {
            edge.from
        };
        let twin = current ^ 1;
        let outgoing = &self.outgoing[destination];
        let twin_position = outgoing.iter().position(|&edge| edge == twin)?;
        for step in 1..=outgoing.len() {
            let candidate = outgoing[(twin_position + outgoing.len() - step) % outgoing.len()];
            if self.boundary[candidate] {
                return Some(candidate);
            }
        }
        None
    }
}

/// Emit loops as closed quadratic subpaths (spec §5).
fn emit_path(loops: &[Loop]) -> Option<QuadPath> {
    let mut path = QuadPath::new();
    for contour in loops {
        // `QuadPath::add_subpath` deliberately treats a subpath whose first
        // anchor equals the current end as a continuation. Distinct boolean
        // contours can nevertheless meet at a C1 crossing (notably XOR
        // lobes), so rotate each later closed loop to a representably
        // different anchor before appending it. Rotation changes neither the
        // curve nor its orientation. If every anchor coincides, the separate
        // contours cannot be represented without mutating coordinates and
        // this route must decline.
        let start = if let Some(previous) = path.points().last().copied() {
            contour
                .quads
                .iter()
                .position(|quad| !path.consider_points_equal(to_vec3(quad.p0), previous))?
        } else {
            0
        };
        let mut points: Vec<super::Vec3> = Vec::with_capacity(contour.quads.len() * 2 + 1);
        for offset in 0..contour.quads.len() {
            let quad = &contour.quads[(start + offset) % contour.quads.len()];
            points.push(to_vec3(quad.p0));
            points.push(to_vec3(quad.p1));
        }
        let first = points[0];
        points.push(first);
        path.add_subpath(&points).ok()?;
    }
    Some(path)
}

/// Budget bump: exceeding a declared limit declines the class (the
/// fallback then surfaces its own deterministic typed error).
fn bump(counter: &mut usize, limit: usize) -> Option<()> {
    if *counter == limit {
        return None;
    }
    *counter += 1;
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quad(p0: Point, p1: Point, p2: Point) -> Quad {
        Quad { p0, p1, p2 }
    }

    #[test]
    fn clip_to_band_distinguishes_empty_branches_from_point_contacts() {
        let base = [0.0, 0.0];
        let normal = [0.0, 1.0];

        let separated = quad([0.0, 2.0], [0.5, 2.0], [1.0, 2.0]);
        assert_eq!(clip_to_band(&separated, base, normal, 0.0, 0.0), None);

        let endpoint = quad([0.0, 0.0], [0.5, 0.5], [1.0, 1.0]);
        assert_eq!(
            clip_to_band(&endpoint, base, normal, 0.0, 0.0),
            Some((0.0, 0.0))
        );

        let tangent = quad([0.0, 1.0], [0.5, -1.0], [1.0, 1.0]);
        assert_eq!(
            clip_to_band(&tangent, base, normal, 0.0, 0.0),
            Some((0.5, 0.5))
        );

        let cancellation_case = quad(
            [0.0, 5.292935891544152e-5],
            [0.5, -3.278838400812914e-5],
            [1.0, -1.184701795835019e-4],
        );
        let clipped = clip_to_band(&cancellation_case, base, normal, 0.0, 0.0)
            .expect("small stable root must survive cancellation");
        assert!((clipped.0 - 0.3087621308989503).abs() < 1.0e-12);
        assert_eq!(clipped.0, clipped.1);
    }

    #[test]
    fn clip_pair_prunes_misses_but_preserves_endpoint_and_tangent_contacts() {
        let options = BooleanOptions::default();

        let horizontal = quad([0.0, 0.0], [0.5, 0.0], [1.0, 0.0]);
        let separated = quad([0.0, 1.0], [0.5, 1.0], [1.0, 1.0]);
        let mut stats = BooleanStats::default();
        assert_eq!(
            clip_pair(&horizontal, &separated, options, &mut stats),
            Some(Vec::new())
        );

        let endpoint = quad([1.0, 0.0], [1.0, 0.5], [1.0, 1.0]);
        let mut stats = BooleanStats::default();
        let endpoint_roots = clip_pair(&horizontal, &endpoint, options, &mut stats)
            .expect("endpoint contact must be resolved");
        assert_eq!(endpoint_roots, vec![(1.0, 0.0)]);
        assert!(!is_transversal_interior(&horizontal, &endpoint, 1.0, 0.0));

        let tangent = quad([0.0, 1.0], [0.5, -1.0], [1.0, 1.0]);
        let mut stats = BooleanStats::default();
        let tangent_roots = clip_pair(&horizontal, &tangent, options, &mut stats)
            .expect("tangent contact must be retained for the C1 screen");
        assert_eq!(tangent_roots, vec![(0.5, 0.5)]);
        assert!(!is_transversal_interior(&horizontal, &tangent, 0.5, 0.5));
    }

    #[test]
    fn clip_pair_preserves_the_reflected_circle_crossing() {
        let subject = quad(
            [0.76536686473018, -1.8477590650225733],
            [1.1329089947010422, -1.6955181300451476],
            [1.4142135623730947, -1.4142135623730954],
        );
        let clip = quad(
            [0.9999999999999998, -1.5],
            [1.2983685510694871, -1.5],
            [1.5740251485476349, -1.38581929876693],
        );
        let mut stats = BooleanStats::default();
        let roots = clip_pair(&subject, &clip, BooleanOptions::default(), &mut stats)
            .expect("transversal circle pair must resolve");
        assert_eq!(roots.len(), 1, "reflected crossing was pruned: {roots:?}");
    }

    #[test]
    fn degenerate_control_triangle_uses_segment_clearance() {
        let line = quad([0.0, 0.0], [0.5, 0.0], [1.0, 0.0]);
        assert_eq!(point_triangle_distance([0.5, 0.0], &line), 0.0);
        assert_eq!(point_triangle_distance([2.0, 0.0], &line), 1.0);
    }

    #[test]
    fn subdivided_control_hulls_tighten_a_false_zero_clearance() {
        let arch = quad([-1.0, 0.0], [0.0, 2.0], [1.0, 0.0]);
        let below_arch = [0.0, 0.25];
        assert_eq!(point_triangle_distance(below_arch, &arch), 0.0);
        assert!(point_quad_hull_distance(below_arch, arch, 1) > 0.0);
        assert_eq!(point_quad_hull_distance([0.0, 1.0], arch, 8), 0.0);
    }

    #[test]
    fn translated_rounded_line_is_not_a_genuine_curve() {
        let p0 = [1_000_000.1, -999_999.7];
        let p2 = [1_000_001.3, -999_997.2];
        let rounded_midpoint = [0.5 * p0[0] + 0.5 * p2[0], 0.5 * p0[1] + 0.5 * p2[1]];
        let line = quad(p0, rounded_midpoint, p2);
        assert!(!is_genuinely_curved(line));

        let visible_arch = quad(p0, [rounded_midpoint[0], rounded_midpoint[1] + 1.0e-4], p2);
        assert!(is_genuinely_curved(visible_arch));
    }

    fn circle(radius: f64, center: Point) -> QuadPath {
        QuadPath::try_arc(
            0.0,
            fmn_core::constants::TAU,
            radius,
            [center[0], center[1], 0.0],
            None,
        )
        .expect("valid arc")
    }

    fn line_path(contour: &[Point]) -> QuadPath {
        let mut path = QuadPath::new();
        path.start_new_path(to_vec3(contour[0]));
        for &point in &contour[1..] {
            path.add_line_to(to_vec3(point), true)
                .expect("a started path accepts lines");
        }
        path.add_line_to(to_vec3(contour[0]), true)
            .expect("a started path closes");
        path
    }

    /// Exact quadratic boundary integral over a result path (spec §5).
    fn path_area(path: &QuadPath) -> f64 {
        path.subpaths()
            .iter()
            .map(|subpath| {
                let mut total = Dd::ZERO;
                for index in (0..subpath.len().saturating_sub(2)).step_by(2) {
                    total = total.add(
                        Quad {
                            p0: xy(subpath[index]),
                            p1: xy(subpath[index + 1]),
                            p2: xy(subpath[index + 2]),
                        }
                        .area_times_six(),
                    );
                }
                total.value() / 6.0
            })
            .sum()
    }

    fn admit(subject: &QuadPath, clip: &QuadPath, operation: BooleanOperation) -> BooleanResult {
        try_transversal(subject, clip, operation, BooleanOptions::default())
            .expect("genuinely transversal scenes must admit")
    }

    #[test]
    fn overlapping_circles_admit_with_exact_area_identities() {
        let subject = circle(2.0, [0.0, 0.0]);
        let clip = circle(1.5, [1.0, 0.0]);
        let subject_area = path_area(&subject);
        let clip_area = path_area(&clip);
        // The 16-piece quadratic circle is a slightly different set than
        // the analytic circle; the identities below are exact regardless.
        assert!((subject_area - 4.0 * std::f64::consts::PI).abs() < 5.0e-3);

        let union = admit(&subject, &clip, BooleanOperation::Union);
        let intersection = admit(&subject, &clip, BooleanOperation::Intersection);
        let difference = admit(&subject, &clip, BooleanOperation::Difference);
        let exclusion = admit(&subject, &clip, BooleanOperation::Exclusion);
        for result in [&union, &intersection, &difference, &exclusion] {
            assert_eq!(result.route, BooleanRoute::CurveAwareTransversal);
        }

        let union_area = path_area(&union.path);
        let intersection_area = path_area(&intersection.path);
        let difference_area = path_area(&difference.path);
        let exclusion_area = path_area(&exclusion.path);
        assert!(
            (union_area + intersection_area - subject_area - clip_area).abs() < 1.0e-9,
            "area(union) + area(intersection) != area(A) + area(B)"
        );
        assert!(
            (difference_area + intersection_area - subject_area).abs() < 1.0e-9,
            "area(difference) + area(intersection) != area(A)"
        );
        assert!(
            (exclusion_area + 2.0 * intersection_area - subject_area - clip_area).abs() < 1.0e-9,
            "area(exclusion) + 2*area(intersection) != area(A) + area(B)"
        );
        // Analytic lens for R=2, r=1.5, d=1, loosened for the quadratic
        // circle approximation.
        let lens = 5.9014758000050564;
        assert!((intersection_area - lens).abs() < 1.5e-2);
        assert!(intersection_area > 0.0 && difference_area > 0.0);

        // Determinism: identical inputs produce identical outputs.
        let again = admit(&subject, &clip, BooleanOperation::Union);
        assert_eq!(union, again);
    }

    #[test]
    fn circle_and_rectangle_admit_mixed_line_curve_scene() {
        let subject = circle(2.0, [0.0, 0.0]);
        let clip = line_path(&[[-1.0, -3.0], [1.0, -3.0], [1.0, 3.0], [-1.0, 3.0]]);
        let intersection = admit(&subject, &clip, BooleanOperation::Intersection);
        // Circle clipped to the slab |x| <= 1: 2*sqrt(3) + 4*pi/3 for the
        // analytic circle, loosened for the quadratic approximation.
        let slab = 2.0 * 3.0_f64.sqrt() + 4.0 * std::f64::consts::PI / 3.0;
        assert!((path_area(&intersection.path) - slab).abs() < 1.5e-2);
        let difference = admit(&subject, &clip, BooleanOperation::Difference);
        assert!(
            (path_area(&difference.path) + path_area(&intersection.path) - path_area(&subject))
                .abs()
                < 1.0e-9
        );
    }

    #[test]
    fn nested_and_disjoint_scenes_admit_the_vacuous_case() {
        // Nested: no cross-operand roots; classification picks contours.
        let outer = circle(2.0, [0.0, 0.0]);
        let inner = circle(1.0, [0.0, 0.0]);
        let intersection = admit(&outer, &inner, BooleanOperation::Intersection);
        assert!((path_area(&intersection.path) - path_area(&inner)).abs() < 1.0e-9);
        let difference = admit(&outer, &inner, BooleanOperation::Difference);
        assert_eq!(
            difference.path.subpaths().len(),
            2,
            "annulus = shell + hole"
        );
        assert!(
            (path_area(&difference.path) - (path_area(&outer) - path_area(&inner))).abs() < 1.0e-9
        );

        // Disjoint with overlapping control hulls (not the separated
        // class): intersection is admitted as the empty path.
        let a = circle(1.0, [0.0, 0.0]);
        let b = circle(0.5, [1.2, 1.2]);
        let empty = admit(&a, &b, BooleanOperation::Intersection);
        assert_eq!(empty.path.subpaths().len(), 0);
        let union = admit(&a, &b, BooleanOperation::Union);
        assert!((path_area(&union.path) - path_area(&a) - path_area(&b)).abs() < 1.0e-9);
    }

    #[test]
    fn even_odd_fill_rule_is_admitted() {
        // Subject = two concentric contours: EvenOdd fills the annulus.
        let mut subject = circle(2.0, [0.0, 0.0]);
        let inner = circle(1.0, [0.0, 0.0]);
        for subpath in inner.subpaths() {
            subject.add_subpath(subpath).expect("subpath appends");
        }
        let clip = circle(3.0, [10.0, 0.0]);
        let options = BooleanOptions {
            subject_fill_rule: crate::FillRule::EvenOdd,
            clip_fill_rule: crate::FillRule::EvenOdd,
            ..BooleanOptions::default()
        };
        let union = try_transversal(&subject, &clip, BooleanOperation::Union, options)
            .expect("EvenOdd is an admitted fill rule");
        let annulus = path_area(&circle(2.0, [0.0, 0.0])) - path_area(&circle(1.0, [0.0, 0.0]));
        assert!((path_area(&union.path) - annulus - path_area(&clip)).abs() < 1.0e-9);
        let intersection =
            try_transversal(&subject, &clip, BooleanOperation::Intersection, options)
                .expect("EvenOdd is an admitted fill rule");
        assert_eq!(intersection.path.subpaths().len(), 0);
    }

    #[test]
    fn degeneracy_classes_decline_to_the_fallback_permanently() {
        let options = BooleanOptions::default();
        // C3: external tangency (touching circles).
        assert!(
            try_transversal(
                &circle(1.0, [0.0, 0.0]),
                &circle(1.0, [2.0, 0.0]),
                BooleanOperation::Union,
                options
            )
            .is_none(),
            "tangent circles must decline"
        );
        // C3: internal tangency.
        assert!(
            try_transversal(
                &circle(2.0, [0.0, 0.0]),
                &circle(1.0, [1.0, 0.0]),
                BooleanOperation::Union,
                options
            )
            .is_none(),
            "internally tangent circles must decline"
        );
        // C4: coincident operands (piece overlaps everywhere).
        assert!(
            try_transversal(
                &circle(1.5, [0.5, 0.5]),
                &circle(1.5, [0.5, 0.5]),
                BooleanOperation::Union,
                options
            )
            .is_none(),
            "coincident operands must decline"
        );
        // C5: zero-area backtrack subject.
        let backtrack = line_path(&[[0.0, 0.0], [2.0, 0.0]]);
        assert!(
            try_transversal(
                &backtrack,
                &circle(1.0, [0.0, 0.0]),
                BooleanOperation::Union,
                options
            )
            .is_none(),
            "zero-area operands must decline"
        );
        // C5: pinch point (an anchor repeats inside one contour).
        let mut pinch = QuadPath::new();
        pinch.start_new_path(to_vec3([0.0, 0.0]));
        pinch
            .add_quadratic_bezier_curve_to(to_vec3([4.0, 4.0]), to_vec3([4.0, 2.0]), true)
            .expect("quad appends");
        pinch
            .add_quadratic_bezier_curve_to(to_vec3([2.0, -2.0]), to_vec3([0.0, 0.0]), true)
            .expect("quad appends");
        pinch
            .add_quadratic_bezier_curve_to(to_vec3([4.0, -4.0]), to_vec3([4.0, -2.0]), true)
            .expect("quad appends");
        pinch
            .add_quadratic_bezier_curve_to(to_vec3([3.0, 1.0]), to_vec3([2.0, 0.0]), true)
            .expect("quad appends");
        pinch
            .add_line_to(to_vec3([0.0, 0.0]), true)
            .expect("closes");
        assert!(
            try_transversal(
                &pinch,
                &circle(3.0, [8.0, 0.0]),
                BooleanOperation::Union,
                options
            )
            .is_none(),
            "pinch points must decline"
        );
        // All-line inputs stay on Stage 1 (exact there, spec §1).
        let rect_a = line_path(&[[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]]);
        let rect_b = line_path(&[[1.0, 1.0], [3.0, 1.0], [3.0, 3.0], [1.0, 3.0]]);
        assert!(try_transversal(&rect_a, &rect_b, BooleanOperation::Union, options).is_none());
        // Budget exhaustion declines (never errors, never coarsens).
        let starved = BooleanOptions {
            limits: crate::BooleanLimits {
                max_pair_tests: 0,
                ..crate::BooleanLimits::DEFAULT
            },
            ..options
        };
        assert!(
            try_transversal(
                &circle(2.0, [0.0, 0.0]),
                &circle(1.5, [1.0, 0.0]),
                BooleanOperation::Union,
                starved
            )
            .is_none(),
            "a zeroed budget must decline"
        );
        // The public route observes the fallback for declined classes.
        let tangent = crate::path_boolean(
            &circle(1.0, [0.0, 0.0]),
            &circle(1.0, [2.0, 0.0]),
            BooleanOperation::Union,
            options,
        )
        .expect("declined classes still produce a boolean via Stage 1");
        assert_eq!(tangent.route, BooleanRoute::FlattenClip);
        // Same-operand interior crossings are found by the pair solver
        // (the operand-self-intersection refusal path).
        let bow_a = quad([0.0, 0.0], [4.0, 4.0], [4.0, 0.0]);
        let bow_b = quad([4.0, 0.0], [0.0, 4.0], [0.0, 0.0]);
        let mut stats = BooleanStats::default();
        let roots = clip_pair(&bow_a, &bow_b, options, &mut stats).expect("bowtie pair resolves");
        assert!(
            roots
                .iter()
                .any(|&(t, u)| { !at_shared_endpoint(&bow_a, &bow_b, t, u) }),
            "the interior self-crossing must be found: {roots:?}"
        );
    }
}
