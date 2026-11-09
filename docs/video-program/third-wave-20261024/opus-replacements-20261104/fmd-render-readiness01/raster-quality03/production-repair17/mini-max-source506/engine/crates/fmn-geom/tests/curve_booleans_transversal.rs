//! The fm-qjy proof battery for the `transversal-interiors` (C1)
//! curve-aware boolean class: every acceptance item of
//! `docs/geometry/curve_booleans.md` §6 as executable tests.
//!
//! - differential point-in-fill proof against forced Stage 1 on jittered
//!   grids, excluding only samples inside the documented band around the
//!   flattened input boundary;
//! - winding equivalence (NonZero and EvenOdd);
//! - components/holes/Euler-characteristic invariants;
//! - boolean identities, including nested evaluations;
//! - multi-scale raster agreement between both routes off boundary cells;
//! - captured skia-pathops fixtures stay on the certified fallback;
//! - deterministic resource-budget outcomes under zeroed phases;
//! - adversarial deterministic fuzzing (2048 cases): never panic, budgets
//!   respected, route discipline, differential fill agreement.

use fmn_geom::{
    BooleanError, BooleanLimits, BooleanOperation, BooleanOptions, BooleanPhase, BooleanResult,
    BooleanRoute, FillRule, QuadPath, path_boolean, path_boolean_flattened,
};

/// Fail with a message. ubs bans `panic!`/`unreachable!` and clippy bans
/// `assert!(false, …)`; a non-constant failing assertion is the compliant
/// form.
#[track_caller]
fn fail(message: String) {
    assert!(message.is_empty(), "{message}");
}

type Point = [f64; 2];

const OPERATIONS: [BooleanOperation; 4] = [
    BooleanOperation::Union,
    BooleanOperation::Intersection,
    BooleanOperation::Difference,
    BooleanOperation::Exclusion,
];

/// Test-side curve sampling density for winding/raster oracles. At 128
/// steps per piece the oracle flattening deviates ~1e-4 on battery-scale
/// scenes, far inside the differential band.
const ORACLE_STEPS: u32 = 128;

// ---------------------------------------------------------------- paths

fn v3(point: Point) -> [f64; 3] {
    [point[0], point[1], 0.0]
}

fn line_path(contours: &[&[Point]]) -> QuadPath {
    let mut path = QuadPath::new();
    for contour in contours {
        if contour.is_empty() {
            continue;
        }
        path.start_new_path(v3(contour[0]));
        for &point in &contour[1..] {
            path.add_line_to(v3(point), true)
                .expect("a started path accepts line segments");
        }
        if contour.last() != contour.first() {
            path.add_line_to(v3(contour[0]), true)
                .expect("a started path closes");
        }
    }
    path
}

fn rectangle(x0: f64, y0: f64, x1: f64, y1: f64) -> QuadPath {
    line_path(&[&[[x0, y0], [x1, y0], [x1, y1], [x0, y1]]])
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

/// An axis-aligned ellipse built by anisotropically scaling a unit arc,
/// then rotating and translating.
fn ellipse(rx: f64, ry: f64, rotation: f64, center: Point) -> QuadPath {
    let unit = circle(1.0, [0.0, 0.0]);
    let (sin, cos) = rotation.sin_cos();
    let points = unit
        .points()
        .iter()
        .map(|p| {
            let x = p[0] * rx;
            let y = p[1] * ry;
            [
                cos * x - sin * y + center[0],
                sin * x + cos * y + center[1],
                0.0,
            ]
        })
        .collect();
    QuadPath::from_points(points).expect("an ellipse is a valid path")
}

/// A two-piece lens: both arcs share both anchors.
fn lune(center: Point, half_width: f64, bulge: f64) -> QuadPath {
    QuadPath::from_points(vec![
        v3([center[0] - half_width, center[1]]),
        v3([center[0], center[1] + bulge]),
        v3([center[0] + half_width, center[1]]),
        v3([center[0], center[1] - bulge]),
        v3([center[0] - half_width, center[1]]),
    ])
    .expect("a lune is a valid path")
}

/// A ring: outer CCW circle plus an inner CW circle (a hole under both
/// fill rules when wound oppositely).
fn donut(outer: f64, inner: f64, center: Point) -> QuadPath {
    let mut path = circle(outer, center);
    let hole = circle(inner, center);
    let mut reversed = hole.points().to_vec();
    reversed.reverse();
    path.add_subpath(&reversed)
        .expect("the hole subpath is valid");
    path
}

// ---------------------------------------------------------------- oracles

/// Test-side flattening of every subpath at a fixed sampling density.
fn flattened_contours(path: &QuadPath) -> Vec<Vec<Point>> {
    let mut contours = Vec::new();
    for subpath in path.subpaths() {
        if subpath.len() < 3 {
            continue;
        }
        let mut points = vec![[subpath[0][0], subpath[0][1]]];
        for index in (0..subpath.len() - 2).step_by(2) {
            let (p0, handle, p2) = (subpath[index], subpath[index + 1], subpath[index + 2]);
            for step in 1..=ORACLE_STEPS {
                let t = f64::from(step) / f64::from(ORACLE_STEPS);
                let mt = 1.0 - t;
                points.push([
                    mt * mt * p0[0] + 2.0 * mt * t * handle[0] + t * t * p2[0],
                    mt * mt * p0[1] + 2.0 * mt * t * handle[1] + t * t * p2[1],
                ]);
            }
        }
        if points.last() != points.first() {
            let first = points[0];
            points.push(first);
        }
        contours.push(points);
    }
    contours
}

/// A cached point-in-fill oracle: the path flattened once at the test
/// oracle density, then wound per sample without re-flattening.
struct Fill {
    contours: Vec<Vec<Point>>,
}

impl Fill {
    fn of(path: &QuadPath) -> Self {
        Self {
            contours: flattened_contours(path),
        }
    }

    fn winding(&self, point: Point) -> i64 {
        let mut winding = 0_i64;
        for contour in &self.contours {
            for segment in contour.windows(2) {
                let [a, b] = [segment[0], segment[1]];
                let side = (b[0] - a[0]) * (point[1] - a[1]) - (b[1] - a[1]) * (point[0] - a[0]);
                if a[1] <= point[1] && b[1] > point[1] && side > 0.0 {
                    winding += 1;
                } else if a[1] > point[1] && b[1] <= point[1] && side < 0.0 {
                    winding -= 1;
                }
            }
        }
        winding
    }

    fn contains(&self, point: Point) -> bool {
        self.winding(point) != 0
    }

    fn contains_rule(&self, point: Point, rule: FillRule) -> bool {
        let winding = self.winding(point);
        match rule {
            FillRule::NonZero => winding != 0,
            FillRule::EvenOdd => winding.rem_euclid(2) == 1,
        }
    }
}

fn contains(path: &QuadPath, point: Point) -> bool {
    Fill::of(path).contains(point)
}

fn expected(operation: BooleanOperation, subject: bool, clip: bool) -> bool {
    match operation {
        BooleanOperation::Union => subject || clip,
        BooleanOperation::Intersection => subject && clip,
        BooleanOperation::Difference => subject && !clip,
        BooleanOperation::Exclusion => subject ^ clip,
    }
}

/// The exact quadratic boundary integral, including each subpath's
/// implicit closing chord.
fn exact_area(path: &QuadPath) -> f64 {
    let cross = |a: Point, b: Point| a[0] * b[1] - a[1] * b[0];
    path.subpaths()
        .iter()
        .map(|subpath| {
            let pieces: f64 = subpath
                .windows(3)
                .step_by(2)
                .map(|piece| {
                    let (p0, h, p2) = (
                        [piece[0][0], piece[0][1]],
                        [piece[1][0], piece[1][1]],
                        [piece[2][0], piece[2][1]],
                    );
                    (cross(p0, h) + 0.5 * cross(p0, p2) + cross(h, p2)) / 3.0
                })
                .sum();
            let first = [subpath[0][0], subpath[0][1]];
            let last = [subpath[subpath.len() - 1][0], subpath[subpath.len() - 1][1]];
            pieces + 0.5 * cross(last, first)
        })
        .sum()
}

/// Components (positively wound contours) and holes (negatively wound),
/// signed by the exact curve area.
fn component_hole_counts(path: &QuadPath) -> (usize, usize) {
    let mut components = 0;
    let mut holes = 0;
    for subpath in path.subpaths() {
        let mut single = QuadPath::new();
        single
            .add_subpath(subpath)
            .expect("a subpath re-adds cleanly");
        let area = exact_area(&single);
        if area > 0.0 {
            components += 1;
        } else if area < 0.0 {
            holes += 1;
        }
    }
    (components, holes)
}

/// Distance from a sample to the flattened input boundary — the
/// differential band of spec §6.1.
fn boundary_distance(point: Point, contours: &[Vec<Point>]) -> f64 {
    let mut best = f64::INFINITY;
    for contour in contours {
        for segment in contour.windows(2) {
            let (a, b) = (segment[0], segment[1]);
            let direction = [b[0] - a[0], b[1] - a[1]];
            let denominator = direction[0].powi(2) + direction[1].powi(2);
            let t = if denominator == 0.0 {
                0.0
            } else {
                (((point[0] - a[0]) * direction[0] + (point[1] - a[1]) * direction[1])
                    / denominator)
                    .clamp(0.0, 1.0)
            };
            let dx = point[0] - (a[0] + direction[0] * t);
            let dy = point[1] - (a[1] + direction[1] * t);
            best = best.min(dx.hypot(dy));
        }
    }
    best
}

/// The documented differential band: Stage-1 flatten tolerance plus the
/// curve-route vertex-snap residual, with headroom for the test-side
/// oracle flattening (spec §6.1).
fn differential_band(options: BooleanOptions) -> f64 {
    8.0 * options.tolerance + 1.0e-6
}

/// Jittered grid samples over a box.
fn grid(x0: f64, y0: f64, dx: f64, dy: f64, nx: usize, ny: usize) -> Vec<Point> {
    (0..ny)
        .flat_map(|y| {
            (0..nx).map(move |x| {
                [
                    x0 + dx * f64::from(x as u32) + 0.013 * f64::from((x * 7 + y * 3) as u32),
                    y0 + dy * f64::from(y as u32) + 0.017 * f64::from((x * 5 + y * 11) as u32),
                ]
            })
        })
        .collect()
}

/// Samples outside the differential band around both flattened inputs.
fn kept_samples(
    samples: &[Point],
    subject: &QuadPath,
    clip: &QuadPath,
    options: BooleanOptions,
) -> Vec<Point> {
    let mut input_contours = flattened_contours(subject);
    input_contours.extend(flattened_contours(clip));
    let band = differential_band(options);
    samples
        .iter()
        .copied()
        .filter(|&point| boundary_distance(point, &input_contours) > band)
        .collect()
}

fn run(
    subject: &QuadPath,
    clip: &QuadPath,
    operation: BooleanOperation,
    options: BooleanOptions,
) -> BooleanResult {
    path_boolean(subject, clip, operation, options).expect("battery fixture must produce a boolean")
}

fn run_flattened(
    subject: &QuadPath,
    clip: &QuadPath,
    operation: BooleanOperation,
    options: BooleanOptions,
) -> BooleanResult {
    path_boolean_flattened(subject, clip, operation, options)
        .expect("forced fallback fixture must produce a boolean")
}

/// The differential core (spec §6.1, §6.2): routed vs forced Stage 1 on
/// kept grid samples, plus winding equivalence against the operands.
fn assert_differential(
    subject: &QuadPath,
    clip: &QuadPath,
    options: BooleanOptions,
    samples: &[Point],
) {
    let kept = kept_samples(samples, subject, clip, options);
    assert!(
        kept.len() >= samples.len() / 2,
        "the band must not eat the grid ({} of {})",
        kept.len(),
        samples.len()
    );
    let subject_fill = Fill::of(subject);
    let clip_fill = Fill::of(clip);
    for operation in OPERATIONS {
        let routed = run(subject, clip, operation, options);
        let fallback = run_flattened(subject, clip, operation, options);
        let routed_fill = Fill::of(&routed.path);
        let fallback_fill = Fill::of(&fallback.path);
        for &point in &kept {
            let truth = expected(
                operation,
                subject_fill.contains_rule(point, options.subject_fill_rule),
                clip_fill.contains_rule(point, options.clip_fill_rule),
            );
            let routed_winding = routed_fill.winding(point);
            assert_eq!(
                routed_winding != 0,
                truth,
                "{operation:?}: winding equivalence failed at {point:?} (route {:?})",
                routed.route
            );
            assert_eq!(
                routed_winding.rem_euclid(2) == 1,
                truth,
                "{operation:?}: winding parity failed at {point:?} (route {:?})",
                routed.route
            );
            assert_eq!(
                fallback_fill.contains(point),
                truth,
                "{operation:?}: forced Stage 1 disagrees with the operands at {point:?}"
            );
        }
    }
}

// ---------------------------------------------------------- 1. admission

/// The absorbed reproduction probe: the transversal route must admit,
/// preserve genuine quadratic handles, and flatten nothing.
#[test]
fn overlapping_circles_admit_transversal_route() {
    let subject = circle(2.0, [0.0, 0.0]);
    let clip = circle(1.5, [1.0, 0.0]);
    let samples = grid(-2.6, -2.4, 0.31, 0.29, 21, 17);
    for operation in OPERATIONS {
        let result = run(&subject, &clip, operation, BooleanOptions::default());
        assert_eq!(
            result.route,
            BooleanRoute::CurveAwareTransversal,
            "{operation:?} declined"
        );
        assert_eq!(result.stats.flattened_segments, 0);
        assert!(result.stats.intersections > 0);
        assert!(
            result.path.bezier_tuples().any(|[p0, handle, p2]| {
                handle[..2] != [p0[0] + (p2[0] - p0[0]) * 0.5, p0[1] + (p2[1] - p0[1]) * 0.5]
            }),
            "curve-aware output must retain non-linear quadratic handles"
        );
        assert!(result.path.points().iter().all(|point| point[2] == 0.0));
        assert!(
            result
                .path
                .subpaths()
                .iter()
                .all(|subpath| subpath.first() == subpath.last()),
            "curve-aware contours must be explicitly closed"
        );
    }
    assert_differential(&subject, &clip, BooleanOptions::default(), &samples);
}

/// Admitted scenes: crossings, containment (vacuous C1), and disjoint
/// operands whose control hulls overlap but whose curves never meet.
#[test]
fn admitted_scenes_match_forced_fallback_and_keep_topology() {
    let scenes: Vec<(QuadPath, QuadPath, &str)> = vec![
        (
            circle(2.0, [0.0, 0.0]),
            circle(1.5, [1.0, 0.0]),
            "circle×circle",
        ),
        (
            circle(2.0, [0.0, 0.0]),
            rectangle(-1.0, -3.0, 1.0, 3.0),
            "circle×rectangle",
        ),
        (
            ellipse(2.5, 1.2, 0.4, [0.3, -0.2]),
            circle(1.6, [-0.5, 0.4]),
            "ellipse×circle",
        ),
        (
            lune([0.0, 0.0], 2.0, 1.6),
            ellipse(1.7, 0.9, -0.3, [0.4, 0.1]),
            "lune×ellipse",
        ),
        (
            circle(3.0, [0.0, 0.0]),
            circle(1.0, [0.4, -0.3]),
            "nested circles (vacuous C1)",
        ),
        (
            ellipse(2.0, 0.6, 0.0, [0.0, 0.0]),
            ellipse(2.0, 0.6, 0.0, [2.8, 0.9]),
            "disjoint, overlapping hulls",
        ),
    ];
    for (subject, clip, name) in &scenes {
        let samples = grid(-4.3, -2.9, 0.37, 0.29, 23, 19);
        for operation in OPERATIONS {
            let result = run(subject, clip, operation, BooleanOptions::default());
            assert_eq!(
                result.route,
                BooleanRoute::CurveAwareTransversal,
                "{name}: {operation:?} declined"
            );
            let fallback = run_flattened(subject, clip, operation, BooleanOptions::default());
            assert_eq!(
                component_hole_counts(&result.path),
                component_hole_counts(&fallback.path),
                "{name}: {operation:?} component/hole topology differs"
            );
        }
        assert_differential(subject, clip, BooleanOptions::default(), &samples);
    }
}

/// EvenOdd operands are admitted and winding-equivalent under parity.
#[test]
fn even_odd_fill_rule_is_winding_equivalent_on_the_curve_route() {
    let subject = donut(2.6, 1.3, [0.0, 0.0]);
    let clip = ellipse(1.5, 0.8, 0.2, [1.7, 0.2]);
    let options = BooleanOptions {
        subject_fill_rule: FillRule::EvenOdd,
        clip_fill_rule: FillRule::EvenOdd,
        ..BooleanOptions::default()
    };
    let samples = grid(-3.2, -2.8, 0.33, 0.31, 21, 17);
    for operation in OPERATIONS {
        let result = run(&subject, &clip, operation, options);
        assert_eq!(
            result.route,
            BooleanRoute::CurveAwareTransversal,
            "{operation:?}: even-odd transversal scene declined"
        );
    }
    assert_differential(&subject, &clip, options, &samples);
}

// ---------------------------------------------------------- 2. topology

/// Nested-hole scenes: components/holes and the Euler characteristic
/// agree with forced Stage 1 (spec §6.3).
#[test]
fn nested_holes_preserve_components_holes_and_euler() {
    let subject = donut(2.5, 1.2, [0.0, 0.0]);
    let clip = ellipse(1.4, 0.9, 0.2, [1.7, 0.0]);
    let samples = grid(-3.1, -2.7, 0.29, 0.31, 23, 19);
    for operation in OPERATIONS {
        let routed = run(&subject, &clip, operation, BooleanOptions::default());
        assert_eq!(routed.route, BooleanRoute::CurveAwareTransversal);
        let fallback = run_flattened(&subject, &clip, operation, BooleanOptions::default());
        let (rc, rh) = component_hole_counts(&routed.path);
        let (fc, fh) = component_hole_counts(&fallback.path);
        assert_eq!((rc, rh), (fc, fh), "{operation:?}: topology differs");
        assert_eq!(
            rc as isize - rh as isize,
            fc as isize - fh as isize,
            "{operation:?}: Euler characteristic differs"
        );
    }
    assert_differential(&subject, &clip, BooleanOptions::default(), &samples);
}

/// Exact-area differential: curve-route exact boundary integrals match
/// forced Stage 1 within the flatten band (spec §6.1 area corollary).
#[test]
fn exact_area_matches_fallback_within_flatten_band() {
    let subject = circle(2.0, [0.0, 0.0]);
    let clip = ellipse(1.6, 1.1, -0.4, [0.8, 0.2]);
    let options = BooleanOptions::default();
    let perimeter_scale = 2.0 * fmn_core::constants::TAU * 2.0;
    for operation in OPERATIONS {
        let routed = run(&subject, &clip, operation, options);
        assert_eq!(routed.route, BooleanRoute::CurveAwareTransversal);
        let fallback = run_flattened(&subject, &clip, operation, options);
        let drift = (exact_area(&routed.path) - exact_area(&fallback.path)).abs();
        assert!(
            drift < perimeter_scale * options.tolerance * 4.0,
            "{operation:?}: area drift {drift:e} exceeds the flatten band"
        );
    }
}

// -------------------------------------------------------- 3. identities

/// Boolean identities as point-in-fill equalities, including nested
/// evaluations feeding boolean outputs back as inputs (spec §6.4).
#[test]
fn boolean_identities_hold_on_the_curve_route() {
    let a = circle(2.0, [0.0, 0.0]);
    let b = ellipse(1.7, 1.0, 0.5, [1.1, 0.3]);
    let options = BooleanOptions::default();
    let samples = grid(-3.4, -2.6, 0.31, 0.27, 23, 19);

    let ab_union = run(&a, &b, BooleanOperation::Union, options);
    let ba_union = run(&b, &a, BooleanOperation::Union, options);
    let intersection = run(&a, &b, BooleanOperation::Intersection, options);
    let difference = run(&a, &b, BooleanOperation::Difference, options);
    let repartitioned = run(&a, &difference.path, BooleanOperation::Difference, options);
    let union_minus_a = run(&ab_union.path, &a, BooleanOperation::Difference, options);
    let b_minus_a = run(&b, &a, BooleanOperation::Difference, options);

    let kept = kept_samples(&samples, &a, &b, options);
    let fills = [
        Fill::of(&ab_union.path),
        Fill::of(&ba_union.path),
        Fill::of(&intersection.path),
        Fill::of(&difference.path),
        Fill::of(&repartitioned.path),
        Fill::of(&union_minus_a.path),
        Fill::of(&b_minus_a.path),
        Fill::of(&a),
    ];
    let [
        ab_fill,
        ba_fill,
        intersection_fill,
        difference_fill,
        repartitioned_fill,
        union_minus_a_fill,
        b_minus_a_fill,
        a_fill,
    ] = &fills;
    for &point in &kept {
        // A∪B == B∪A.
        assert_eq!(
            ab_fill.contains(point),
            ba_fill.contains(point),
            "commutativity failed at {point:?}"
        );
        // (A∩B) ⊔ (A−B) == A.
        assert_eq!(
            intersection_fill.contains(point) ^ difference_fill.contains(point),
            a_fill.contains(point),
            "partition failed at {point:?}"
        );
        // A∩B == A − (A−B).
        assert_eq!(
            intersection_fill.contains(point),
            repartitioned_fill.contains(point),
            "nested intersection identity failed at {point:?}"
        );
        // (A∪B) − A == B − A.
        assert_eq!(
            union_minus_a_fill.contains(point),
            b_minus_a_fill.contains(point),
            "nested union identity failed at {point:?}"
        );
    }
    assert_eq!(
        component_hole_counts(&ab_union.path),
        component_hole_counts(&ba_union.path),
        "commutativity must be topological, not vertex-order"
    );
}

// ----------------------------------------------------------- 4. raster

/// Multi-scale raster agreement (spec §6.5): at several resolutions the
/// two routes' bitmaps may differ only on boundary cells — cells whose
/// 3×3 neighborhood in the fallback bitmap contains both fill values.
#[test]
fn multi_scale_rasters_agree_off_boundary_cells() {
    let subject = circle(2.0, [0.0, 0.0]);
    let clip = ellipse(1.6, 1.1, -0.4, [0.8, 0.2]);
    let options = BooleanOptions::default();
    for operation in OPERATIONS {
        let routed = run(&subject, &clip, operation, options);
        assert_eq!(routed.route, BooleanRoute::CurveAwareTransversal);
        let fallback = run_flattened(&subject, &clip, operation, options);
        for resolution in [32_usize, 48, 64] {
            let (x0, y0, span) = (-2.55, -2.35, 5.4);
            let cell = span / resolution as f64;
            let bitmap = |fill: &Fill| -> Vec<bool> {
                (0..resolution * resolution)
                    .map(|index| {
                        let x = index % resolution;
                        let y = index / resolution;
                        fill.contains([
                            x0 + (f64::from(x as u32) + 0.5) * cell,
                            y0 + (f64::from(y as u32) + 0.5) * cell,
                        ])
                    })
                    .collect()
            };
            let routed_bitmap = bitmap(&Fill::of(&routed.path));
            let fallback_bitmap = bitmap(&Fill::of(&fallback.path));
            let mut mismatches = 0_usize;
            for y in 0..resolution {
                for x in 0..resolution {
                    let index = y * resolution + x;
                    if routed_bitmap[index] == fallback_bitmap[index] {
                        continue;
                    }
                    mismatches += 1;
                    let boundary_cell =
                        (y.saturating_sub(1)..=(y + 1).min(resolution - 1)).any(|ny| {
                            (x.saturating_sub(1)..=(x + 1).min(resolution - 1)).any(|nx| {
                                fallback_bitmap[ny * resolution + nx] != fallback_bitmap[index]
                            })
                        });
                    assert!(
                        boundary_cell,
                        "{operation:?} at {resolution}³: off-boundary raster mismatch at cell ({x}, {y})"
                    );
                }
            }
            assert!(
                mismatches * 100 <= resolution * resolution * 3,
                "{operation:?} at {resolution}: {mismatches} boundary mismatches exceed 3%"
            );
        }
    }
}

// -------------------------------------------- 5. unproved degeneracy classes

/// Spec §7: C2–C5 stay on the certified fallback permanently, and the
/// fallback result stays correct (spec §6.1 applied to the routing rows).
#[test]
fn unproved_classes_route_to_fallback_and_stay_correct() {
    let tangent = (
        circle(1.0, [0.0, 0.0]),
        rectangle(1.0, -1.0, 3.0, 1.0),
        "C3 tangency",
    );
    let shared_endpoint = (
        lune([0.0, 0.0], 2.0, 1.5),
        lune([4.0, 0.0], 2.0, -1.5),
        "C2 shared endpoint",
    );
    let identical = (
        circle(1.8, [0.2, -0.1]),
        circle(1.8, [0.2, -0.1]),
        "C4 overlap (identical operands)",
    );
    let partial_overlap = (
        lune([0.0, 0.0], 2.0, 1.4),
        lune([0.0, 0.0], 2.0, 1.4),
        "C4 overlap (coincident arcs)",
    );
    let backtrack = (
        QuadPath::from_points(vec![
            v3([0.0, 0.0]),
            v3([1.0, 1.0]),
            v3([0.0, 0.0]),
            v3([-1.0, -1.0]),
            v3([0.0, 0.0]),
        ])
        .expect("a backtracking path is structurally valid"),
        circle(1.5, [0.5, 0.5]),
        "C5 zero-area backtrack",
    );
    let self_crossing = (
        QuadPath::from_points(vec![
            v3([-2.0, -1.0]),
            v3([0.0, 2.0]),
            v3([2.0, -1.0]),
            v3([0.0, -2.0]),
            v3([-2.0, 1.0]),
            v3([2.0, 1.0]),
            v3([-2.0, -1.0]),
        ])
        .expect("a self-crossing path is structurally valid"),
        circle(1.2, [0.5, 0.0]),
        "C2/C5 self-crossing contour",
    );
    let all_line = (
        rectangle(0.0, 0.0, 2.0, 2.0),
        rectangle(1.0, -1.0, 3.0, 1.0),
        "all-line inputs (exact under Stage 1)",
    );
    let slanted_all_line = (
        line_path(&[&[[0.0, 0.0], [1.5, 0.2], [0.3, 1.4]]]),
        line_path(&[&[[0.8, -0.5], [1.6, 0.5], [0.7, 1.6], [-0.2, 0.6]]]),
        "slanted all-line inputs (ulp-off-chord handles are still lines)",
    );
    let cases = [
        tangent,
        shared_endpoint,
        identical,
        partial_overlap,
        backtrack,
        self_crossing,
        all_line,
        slanted_all_line,
    ];
    for (subject, clip, name) in &cases {
        let samples = grid(-4.1, -3.3, 0.41, 0.37, 19, 17);
        for operation in OPERATIONS {
            let result = run(subject, clip, operation, BooleanOptions::default());
            assert_eq!(
                result.route,
                BooleanRoute::FlattenClip,
                "{name}: {operation:?} must route to the certified fallback permanently"
            );
        }
        assert_differential(subject, clip, BooleanOptions::default(), &samples);
    }
}

/// The captured skia-pathops fixtures are all-line inputs: they must
/// keep routing to Stage 1 and keep matching their captured topology
/// (spec §6.6).
#[test]
fn captured_skia_fixtures_stay_on_the_certified_fallback() {
    const FIXTURES: &str = include_str!("../fixtures/path_booleans.txt");
    let mut cases = 0;
    for line in FIXTURES
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(fields.len(), 8, "malformed fixture row");
        let name = fields[0];
        let operation = match fields[1] {
            "union" => BooleanOperation::Union,
            "intersection" => BooleanOperation::Intersection,
            "difference" => BooleanOperation::Difference,
            "exclusion" => BooleanOperation::Exclusion,
            other => {
                fail(format!("{name}: unknown fixture operation {other}"));
                BooleanOperation::Union
            }
        };
        let subject = fixture_path(fields[2]);
        let clip = fixture_path(fields[3]);
        let expected_area: f64 = fields[4].parse().expect("fixture area is numeric");
        let expected_contours: usize = fields[5].parse().expect("fixture count is numeric");

        let result = run(&subject, &clip, operation, BooleanOptions::default());
        assert_ne!(
            result.route,
            BooleanRoute::CurveAwareTransversal,
            "{name}: polygonal fixtures must never enter the transversal class"
        );
        assert!(
            (exact_area(&result.path) - expected_area).abs() < 1.0e-10,
            "{name}: area differs from captured skia-pathops topology"
        );
        assert_eq!(
            result.path.has_points(),
            expected_contours != 0,
            "{name}: empty/nonempty topology differs from skia-pathops"
        );
        cases += 1;
    }
    assert_eq!(cases, 10);
}

fn fixture_path(encoded: &str) -> QuadPath {
    if encoded == "-" {
        return QuadPath::new();
    }
    let contours: Vec<Vec<Point>> = encoded
        .split(';')
        .map(|contour| {
            contour
                .split('|')
                .map(|point| {
                    let (x, y) = point
                        .split_once(',')
                        .expect("fixture point has an x,y pair");
                    [
                        x.parse().expect("fixture x is numeric"),
                        y.parse().expect("fixture y is numeric"),
                    ]
                })
                .collect()
        })
        .collect();
    let borrowed: Vec<&[Point]> = contours.iter().map(Vec::as_slice).collect();
    line_path(&borrowed)
}

// ----------------------------------------------------------- 6. budgets

/// Zeroed phases produce deterministic outcomes; every decline falls
/// through to the fallback, which surfaces its typed error identically
/// (spec §6.7).
#[test]
fn zeroed_budgets_are_deterministic_and_match_forced_fallback() {
    let subject = circle(2.0, [0.0, 0.0]);
    let clip = circle(1.5, [1.0, 0.0]);
    let defaults = BooleanLimits::DEFAULT;
    let phases = [
        (
            BooleanLimits {
                max_flattened_segments: 0,
                ..defaults
            },
            BooleanPhase::Flatten,
        ),
        (
            BooleanLimits {
                max_pair_tests: 0,
                ..defaults
            },
            BooleanPhase::PairTests,
        ),
        (
            BooleanLimits {
                max_intersections: 0,
                ..defaults
            },
            BooleanPhase::Intersections,
        ),
        (
            BooleanLimits {
                max_classification_tests: 0,
                ..defaults
            },
            BooleanPhase::Classification,
        ),
        (
            BooleanLimits {
                max_output_segments: 0,
                ..defaults
            },
            BooleanPhase::Output,
        ),
    ];
    let samples = grid(-2.6, -2.4, 0.31, 0.29, 21, 17);
    for (limits, phase) in phases {
        let options = BooleanOptions {
            limits,
            ..BooleanOptions::default()
        };
        for operation in OPERATIONS {
            let first = path_boolean(&subject, &clip, operation, options);
            let second = path_boolean(&subject, &clip, operation, options);
            assert_eq!(
                first, second,
                "{phase:?}/{operation:?}: budget outcome must be deterministic"
            );
            let forced = path_boolean_flattened(&subject, &clip, operation, options);
            match (first, forced) {
                (Err(error), Err(forced_error)) => {
                    assert_eq!(
                        error,
                        BooleanError::ResourceLimit { phase, limit: 0 },
                        "{phase:?}/{operation:?}: typed budget error must name its phase"
                    );
                    assert_eq!(
                        error, forced_error,
                        "{phase:?}/{operation:?}: decline must surface the fallback's typed error"
                    );
                }
                (Ok(result), Err(_)) => {
                    // The zeroed phase governs work the admitted curve
                    // route never performs (flattening): a curve-aware
                    // success is legitimate — but it must still be the
                    // correct filled set.
                    assert_eq!(
                        result.route,
                        BooleanRoute::CurveAwareTransversal,
                        "{phase:?}/{operation:?}: only the curve route may bypass a fallback budget"
                    );
                    let kept = kept_samples(&samples, &subject, &clip, options);
                    for &point in &kept {
                        assert_eq!(
                            contains(&result.path, point),
                            expected(operation, contains(&subject, point), contains(&clip, point)),
                            "{phase:?}/{operation:?}: budget-bypassing result is wrong at {point:?}"
                        );
                    }
                }
                (Ok(result), Ok(forced_result)) => {
                    let kept = kept_samples(&samples, &subject, &clip, options);
                    for &point in &kept {
                        assert_eq!(
                            contains(&result.path, point),
                            contains(&forced_result.path, point),
                            "{phase:?}/{operation:?}: budget outcome diverges at {point:?}"
                        );
                    }
                }
                (Err(error), Ok(_)) => {
                    fail(format!(
                        "{phase:?}/{operation:?}: routed entry errored ({error:?}) where the fallback succeeds"
                    ));
                }
            }
        }
    }
}

// ------------------------------------------------------------- 7. fuzz

struct SplitMix64(u64);

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn unit(&mut self) -> f64 {
        f64::from((self.next() >> 40) as u32) / f64::from(1_u32 << 24)
    }

    fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.unit()
    }
}

/// A random rotated ellipse (families 1–2).
fn random_ellipse(rng: &mut SplitMix64, scale: f64) -> QuadPath {
    ellipse(
        rng.range(0.5, 2.2) * scale,
        rng.range(0.5, 2.2) * scale,
        rng.range(-1.5, 1.5),
        [rng.range(-1.5, 1.5) * scale, rng.range(-1.5, 1.5) * scale],
    )
}

/// A random polar closed loop of quadratics: anchors on jittered polar
/// rings, handles pushed tangentially — smooth star-like contours.
fn random_smooth_loop(rng: &mut SplitMix64, scale: f64) -> QuadPath {
    let pieces = 3 + (rng.next() % 5) as usize;
    let center = [rng.range(-1.0, 1.0) * scale, rng.range(-1.0, 1.0) * scale];
    let mut anchors = Vec::with_capacity(pieces);
    for index in 0..pieces {
        let angle =
            fmn_core::constants::TAU * (index as f64 + rng.range(-0.2, 0.2)) / pieces as f64;
        let radius = rng.range(0.8, 2.0) * scale;
        anchors.push([
            center[0] + radius * angle.cos(),
            center[1] + radius * angle.sin(),
        ]);
    }
    let mut points = vec![v3(anchors[0])];
    for index in 0..pieces {
        let next = anchors[(index + 1) % pieces];
        let mid = [
            (anchors[index][0] + next[0]) * 0.5,
            (anchors[index][1] + next[1]) * 0.5,
        ];
        let bulge = rng.range(0.8, 1.6);
        let handle = [
            center[0] + (mid[0] - center[0]) * bulge,
            center[1] + (mid[1] - center[1]) * bulge,
        ];
        points.push(v3(handle));
        points.push(v3(next));
    }
    QuadPath::from_points(points).expect("a polar loop is structurally valid")
}

/// A random closed walk, usually self-crossing (adversarial C2/C5).
fn random_walk_loop(rng: &mut SplitMix64, scale: f64) -> QuadPath {
    let pieces = 4 + (rng.next() % 4) as usize;
    let mut anchors = Vec::with_capacity(pieces);
    for _ in 0..pieces {
        anchors.push([rng.range(-2.0, 2.0) * scale, rng.range(-2.0, 2.0) * scale]);
    }
    let mut points = vec![v3(anchors[0])];
    for index in 0..pieces {
        let next = anchors[(index + 1) % pieces];
        points.push(v3([
            (anchors[index][0] + next[0]) * 0.5 + rng.range(-0.5, 0.5) * scale,
            (anchors[index][1] + next[1]) * 0.5 + rng.range(-0.5, 0.5) * scale,
        ]));
        points.push(v3(next));
    }
    QuadPath::from_points(points).expect("a walk loop is structurally valid")
}

/// A random all-line polygon (Stage-1-exact family).
fn random_polygon(rng: &mut SplitMix64, scale: f64) -> QuadPath {
    let vertices = 3 + (rng.next() % 4) as usize;
    let center = [rng.range(-1.0, 1.0) * scale, rng.range(-1.0, 1.0) * scale];
    let contour: Vec<Point> = (0..vertices)
        .map(|index| {
            let angle = fmn_core::constants::TAU * index as f64 / vertices as f64;
            let radius = rng.range(0.6, 1.8) * scale;
            [
                center[0] + radius * angle.cos(),
                center[1] + radius * angle.sin(),
            ]
        })
        .collect();
    line_path(&[&contour])
}

/// Adversarial fuzzing (spec §6.8): 2048 deterministic cases across
/// admissible, degenerate, and Stage-1-exact families. Never panic,
/// budgets respected, route discipline held, differential fill agreement
/// outside the documented band, deterministic reruns. The corpus is split
/// by operation into four tests so the harness runs them concurrently;
/// every test regenerates the identical corpus from the same stream.
#[test]
fn adversarial_curve_corpus_differential_fuzz_union() {
    adversarial_fuzz(BooleanOperation::Union, 0);
}

#[test]
fn adversarial_curve_corpus_differential_fuzz_intersection() {
    adversarial_fuzz(BooleanOperation::Intersection, 1);
}

#[test]
fn adversarial_curve_corpus_differential_fuzz_difference() {
    adversarial_fuzz(BooleanOperation::Difference, 2);
}

#[test]
fn adversarial_curve_corpus_differential_fuzz_exclusion() {
    adversarial_fuzz(BooleanOperation::Exclusion, 3);
}

fn adversarial_fuzz(operation: BooleanOperation, op_index: usize) {
    let mut rng = SplitMix64(0xf00d_9e17_5eed_c0de_u64);
    let mut route_counts = [0_usize; 3];
    for case in 0..2048_u32 {
        let family = rng.next() % 10;
        let scale = rng.range(0.5, 2.0);
        let (subject, clip) = match family {
            0..=3 => (
                random_ellipse(&mut rng, scale),
                random_ellipse(&mut rng, scale),
            ),
            4..=5 => (
                random_smooth_loop(&mut rng, scale),
                random_ellipse(&mut rng, scale),
            ),
            6..=7 => (
                random_walk_loop(&mut rng, scale),
                random_ellipse(&mut rng, scale),
            ),
            8 => (
                random_polygon(&mut rng, scale),
                random_polygon(&mut rng, scale),
            ),
            _ => {
                // Degenerate family: a backtracking out-and-back contour
                // (zero area, C5).
                let anchor = [rng.range(-1.0, 1.0), rng.range(-1.0, 1.0)];
                let out = [
                    anchor[0] + rng.range(0.5, 2.0),
                    anchor[1] + rng.range(-1.0, 1.0),
                ];
                let degenerate = QuadPath::from_points(vec![
                    v3(anchor),
                    v3([(anchor[0] + out[0]) * 0.5, (anchor[1] + out[1]) * 0.5 + 0.4]),
                    v3(anchor),
                ])
                .expect("a degenerate path is structurally valid");
                (degenerate, random_ellipse(&mut rng, scale))
            }
        };
        let samples = grid(-4.7, -4.3, 0.53, 0.47, 7, 7);
        let kept = kept_samples(&samples, &subject, &clip, BooleanOptions::default());
        let subject_fill = Fill::of(&subject);
        let clip_fill = Fill::of(&clip);
        {
            let routed = path_boolean(&subject, &clip, operation, BooleanOptions::default());
            let forced =
                path_boolean_flattened(&subject, &clip, operation, BooleanOptions::default());
            match (routed, forced) {
                (Err(routed_error), Err(forced_error)) => {
                    assert_eq!(
                        routed_error, forced_error,
                        "case {case}/{operation:?}: typed refusal must match forced Stage 1"
                    );
                }
                (Ok(result), Ok(fallback)) => {
                    let route_index = match result.route {
                        BooleanRoute::CurveAwareSeparated => 0,
                        BooleanRoute::CurveAwareTransversal => 1,
                        BooleanRoute::FlattenClip => 2,
                    };
                    route_counts[route_index] += 1;
                    if family == 9 {
                        assert_eq!(
                            result.route,
                            BooleanRoute::FlattenClip,
                            "case {case}/{operation:?}: degenerate family left Stage 1"
                        );
                    } else if family == 8 {
                        assert_ne!(
                            result.route,
                            BooleanRoute::CurveAwareTransversal,
                            "case {case}/{operation:?}: all-line family entered C1"
                        );
                    }
                    if result.route == BooleanRoute::CurveAwareTransversal {
                        assert_eq!(
                            result.stats.flattened_segments, 0,
                            "case {case}/{operation:?}: curve route flattened"
                        );
                    }
                    let limits = BooleanLimits::DEFAULT;
                    assert!(result.stats.pair_tests <= limits.max_pair_tests);
                    assert!(result.stats.intersections <= limits.max_intersections);
                    assert!(result.stats.classification_tests <= limits.max_classification_tests);
                    assert!(result.stats.output_segments <= limits.max_output_segments);
                    // Determinism probe on this operation's share of the
                    // corpus; the four operations partition the cases.
                    if case as usize % OPERATIONS.len() == op_index {
                        let rerun =
                            path_boolean(&subject, &clip, operation, BooleanOptions::default());
                        assert_eq!(
                            &result,
                            rerun.as_ref().expect("deterministic rerun must succeed"),
                            "case {case}/{operation:?}: routed boolean is not deterministic"
                        );
                    }
                    let result_fill = Fill::of(&result.path);
                    let fallback_fill = Fill::of(&fallback.path);
                    for &point in &kept {
                        let truth = expected(
                            operation,
                            subject_fill.contains(point),
                            clip_fill.contains(point),
                        );
                        assert_eq!(
                            result_fill.contains(point),
                            truth,
                            "case {case}/{operation:?} ({:?}): winding equivalence failed at {point:?}",
                            result.route
                        );
                        assert_eq!(
                            result_fill.contains(point),
                            fallback_fill.contains(point),
                            "case {case}/{operation:?} ({:?} vs {:?}): fill mismatch at {point:?}",
                            result.route,
                            fallback.route
                        );
                    }
                }
                (result, forced) => {
                    fail(format!(
                        "case {case}/{operation:?}: routed/fallback outcome kinds diverge: {:?} vs {:?}",
                        result.map(|r| r.route),
                        forced.map(|r| r.route)
                    ));
                }
            }
        }
    }
    assert!(
        route_counts[1] > 0,
        "the fuzz corpus must actually exercise the transversal route"
    );
    assert!(
        route_counts[2] > 0,
        "the fuzz corpus must actually exercise the fallback"
    );
}
