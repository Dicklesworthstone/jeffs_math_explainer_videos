//! Topology-aware acceptance for the certified flatten-and-clip boolean.

use fmn_geom::{
    BooleanError, BooleanLimits, BooleanOperation, BooleanOptions, BooleanPhase, BooleanRoute,
    FillRule, QuadPath, path_boolean, path_boolean_flattened,
};

type Point = [f64; 2];

fn path_from_contours(contours: &[&[Point]]) -> QuadPath {
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
    path_from_contours(&[&[[x0, y0], [x1, y0], [x1, y1], [x0, y1]]])
}

fn v3(point: Point) -> [f64; 3] {
    [point[0], point[1], 0.0]
}

fn run(
    subject: &QuadPath,
    clip: &QuadPath,
    operation: BooleanOperation,
) -> fmn_geom::BooleanResult {
    path_boolean(subject, clip, operation, BooleanOptions::default())
        .expect("acceptance fixture must produce a boolean")
}

fn run_flattened(
    subject: &QuadPath,
    clip: &QuadPath,
    operation: BooleanOperation,
) -> fmn_geom::BooleanResult {
    path_boolean_flattened(subject, clip, operation, BooleanOptions::default())
        .expect("forced fallback acceptance fixture must produce a boolean")
}

fn winding(path: &QuadPath, point: Point) -> i64 {
    let mut winding = 0_i64;
    for subpath in path.subpaths() {
        let mut flattened = vec![[subpath[0][0], subpath[0][1]]];
        for index in (0..subpath.len() - 2).step_by(2) {
            let p0 = subpath[index];
            let handle = subpath[index + 1];
            let p2 = subpath[index + 2];
            let midpoint = [
                p0[0] + (p2[0] - p0[0]) * 0.5,
                p0[1] + (p2[1] - p0[1]) * 0.5,
                p0[2] + (p2[2] - p0[2]) * 0.5,
            ];
            let steps = if handle == midpoint { 1 } else { 64 };
            for step in 1..=steps {
                let sample = fmn_geom::bezier::quadratic_point(
                    p0,
                    handle,
                    p2,
                    f64::from(step) / f64::from(steps),
                );
                flattened.push([sample[0], sample[1]]);
            }
        }
        if flattened.last() != flattened.first() {
            flattened.push(flattened[0]);
        }
        for segment in flattened.windows(2) {
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

fn contains(path: &QuadPath, point: Point) -> bool {
    winding(path, point) != 0
}

fn expected(operation: BooleanOperation, subject: bool, clip: bool) -> bool {
    match operation {
        BooleanOperation::Union => subject || clip,
        BooleanOperation::Intersection => subject && clip,
        BooleanOperation::Difference => subject && !clip,
        BooleanOperation::Exclusion => subject ^ clip,
    }
}

fn signed_area(path: &QuadPath) -> f64 {
    path.subpaths()
        .iter()
        .map(|subpath| {
            let anchors: Vec<Point> = subpath.iter().step_by(2).map(|p| [p[0], p[1]]).collect();
            0.5 * anchors
                .windows(2)
                .map(|edge| edge[0][0] * edge[1][1] - edge[0][1] * edge[1][0])
                .sum::<f64>()
        })
        .sum()
}

fn component_hole_counts(path: &QuadPath) -> (usize, usize) {
    let mut components = 0;
    let mut holes = 0;
    for subpath in path.subpaths() {
        let anchors: Vec<Point> = subpath.iter().step_by(2).map(|p| [p[0], p[1]]).collect();
        let area = 0.5
            * anchors
                .windows(2)
                .map(|edge| edge[0][0] * edge[1][1] - edge[0][1] * edge[1][0])
                .sum::<f64>();
        if area > 0.0 {
            components += 1;
        } else if area < 0.0 {
            holes += 1;
        }
    }
    (components, holes)
}

fn assert_grid_truth(
    result: &QuadPath,
    subject: &QuadPath,
    clip: &QuadPath,
    operation: BooleanOperation,
    samples: &[Point],
) {
    for &point in samples {
        assert_eq!(
            contains(result, point),
            expected(operation, contains(subject, point), contains(clip, point)),
            "fill mismatch at {point:?} for {operation:?}"
        );
    }
}

#[test]
fn overlapping_rectangles_match_boolean_truth_and_area() {
    let subject = rectangle(0.0, 0.0, 2.0, 2.0);
    let clip = rectangle(1.0, -1.0, 3.0, 1.0);
    let samples = [
        [-0.25, 0.5],
        [0.5, 0.5],
        [1.5, 0.5],
        [2.5, 0.5],
        [1.5, 1.5],
        [2.5, 1.5],
    ];
    let cases = [
        (BooleanOperation::Union, 7.0),
        (BooleanOperation::Intersection, 1.0),
        (BooleanOperation::Difference, 3.0),
        (BooleanOperation::Exclusion, 6.0),
    ];
    for (operation, area) in cases {
        let result = run(&subject, &clip, operation);
        assert_eq!(result.route, BooleanRoute::FlattenClip);
        assert_grid_truth(&result.path, &subject, &clip, operation, &samples);
        assert!((signed_area(&result.path) - area).abs() < 1.0e-12);
        assert!(result.stats.pair_tests > 0);
    }
}

#[test]
fn commutativity_is_topological_not_vertex_sequence_based() {
    let a = rectangle(-2.0, -1.0, 1.0, 2.0);
    let b = rectangle(-1.0, -2.0, 2.0, 1.0);
    let samples: Vec<Point> = (-10..=10)
        .flat_map(|y| {
            (-10..=10).map(move |x| [f64::from(x) * 0.23 + 0.017, f64::from(y) * 0.23 + 0.031])
        })
        .collect();
    for operation in [BooleanOperation::Union, BooleanOperation::Intersection] {
        let ab = run(&a, &b, operation);
        let ba = run(&b, &a, operation);
        for &point in &samples {
            assert_eq!(
                contains(&ab.path, point),
                contains(&ba.path, point),
                "commutativity failed at {point:?}"
            );
        }
        assert_eq!(
            component_hole_counts(&ab.path),
            component_hole_counts(&ba.path)
        );
    }
}

#[test]
fn difference_cannot_overlap_its_clip() {
    let a = rectangle(-2.0, -2.0, 2.0, 2.0);
    let b = rectangle(-0.5, -3.0, 0.75, 3.0);
    let difference = run(&a, &b, BooleanOperation::Difference);
    let overlap = run(&difference.path, &b, BooleanOperation::Intersection);
    assert!(!overlap.path.has_points());
    assert_eq!(overlap.stats.output_contours, 0);
}

#[test]
fn nested_opposite_contours_preserve_hole_and_euler_characteristic() {
    let outer = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]];
    let inner_clockwise = [[1.0, 1.0], [1.0, 3.0], [3.0, 3.0], [3.0, 1.0]];
    let donut = path_from_contours(&[&outer, &inner_clockwise]);
    let result = run(&donut, &QuadPath::new(), BooleanOperation::Union);
    assert!(contains(&result.path, [0.5, 0.5]));
    assert!(!contains(&result.path, [2.0, 2.0]));
    let (components, holes) = component_hole_counts(&result.path);
    assert_eq!((components, holes), (1, 1));
    assert_eq!(components as isize - holes as isize, 0);
    assert!((signed_area(&result.path) - 12.0).abs() < 1.0e-12);
}

#[test]
fn even_odd_and_nonzero_fill_rules_are_distinct() {
    let outer = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]];
    let inner_same_winding = [[1.0, 1.0], [3.0, 1.0], [3.0, 3.0], [1.0, 3.0]];
    let nested = path_from_contours(&[&outer, &inner_same_winding]);

    let nonzero = run(&nested, &QuadPath::new(), BooleanOperation::Union);
    let even_odd = path_boolean(
        &nested,
        &QuadPath::new(),
        BooleanOperation::Union,
        BooleanOptions {
            subject_fill_rule: FillRule::EvenOdd,
            ..BooleanOptions::default()
        },
    )
    .expect("even-odd fixture must produce a boolean");
    assert!(contains(&nonzero.path, [2.0, 2.0]));
    assert!(!contains(&even_odd.path, [2.0, 2.0]));
    assert_eq!(component_hole_counts(&even_odd.path), (1, 1));
}

#[test]
fn shared_edges_partial_overlaps_and_touches_do_not_emit_slivers() {
    let left = rectangle(0.0, 0.0, 1.0, 1.0);
    let right = rectangle(1.0, 0.0, 2.0, 1.0);
    let union = run(&left, &right, BooleanOperation::Union);
    let intersection = run(&left, &right, BooleanOperation::Intersection);
    assert!((signed_area(&union.path) - 2.0).abs() < 1.0e-12);
    assert_eq!(component_hole_counts(&union.path), (1, 0));
    assert!(!intersection.path.has_points());

    let overlap = rectangle(0.5, 1.0, 1.5, 2.0);
    let touch = run(&left, &overlap, BooleanOperation::Intersection);
    assert!(!touch.path.has_points());

    let partial = rectangle(0.5, 0.0, 1.5, 1.0);
    let exclusion = run(&left, &partial, BooleanOperation::Exclusion);
    assert!((signed_area(&exclusion.path) - 1.0).abs() < 1.0e-12);
}

#[test]
fn exact_backtracks_cancel_as_zero_area() {
    let backtrack = path_from_contours(&[&[[0.0, 0.0], [2.0, 0.0], [0.0, 0.0]]]);
    let result = run(&backtrack, &QuadPath::new(), BooleanOperation::Union);
    assert!(!result.path.has_points());
}

#[test]
fn self_intersection_is_resolved_by_winding_not_input_vertex_order() {
    let bow_tie = path_from_contours(&[&[[-2.0, -1.0], [2.0, 1.0], [-2.0, 1.0], [2.0, -1.0]]]);
    let normalized = run(&bow_tie, &QuadPath::new(), BooleanOperation::Union);
    let samples = [
        [-1.5, 0.75],
        [-1.5, -0.75],
        [0.0, 0.5],
        [0.0, -0.5],
        [0.0, 1.5],
    ];
    for point in samples {
        assert_eq!(
            contains(&normalized.path, point),
            contains(&bow_tie, point),
            "self-intersection fill changed at {point:?}"
        );
    }
}

#[test]
fn curved_inputs_flatten_deterministically_and_drop_z() {
    let mut circle = QuadPath::try_arc(0.0, fmn_core::constants::TAU, 2.0, [0.0, 0.0, 9.0], None)
        .expect("valid arc");
    for point in circle.points().iter().copied() {
        assert_eq!(point[2], 9.0);
    }
    circle.set_tolerance_for_point_equality(1.0e-9);
    let a = run(&circle, &QuadPath::new(), BooleanOperation::Union);
    let b = run(&circle, &QuadPath::new(), BooleanOperation::Union);
    assert_eq!(a, b);
    assert!(a.stats.flattened_segments > circle.num_curves());
    assert!(contains(&a.path, [0.0, 0.0]));
    assert!(!contains(&a.path, [2.1, 0.0]));
    assert!(a.path.points().iter().all(|point| point[2] == 0.0));
}

#[test]
fn every_explosive_phase_has_a_precise_budget_error() {
    let a = rectangle(0.0, 0.0, 2.0, 2.0);
    let b = rectangle(1.0, 1.0, 3.0, 3.0);
    let defaults = BooleanLimits::DEFAULT;
    let cases = [
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
    for (limits, phase) in cases {
        let error = path_boolean(
            &a,
            &b,
            BooleanOperation::Union,
            BooleanOptions {
                limits,
                ..BooleanOptions::default()
            },
        )
        .expect_err("zero budget must refuse work");
        assert_eq!(error, BooleanError::ResourceLimit { phase, limit: 0 });
    }
}

#[test]
fn invalid_numbers_are_refused_before_work() {
    let square = rectangle(0.0, 0.0, 1.0, 1.0);
    for tolerance in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        let error = path_boolean(
            &square,
            &QuadPath::new(),
            BooleanOperation::Union,
            BooleanOptions {
                tolerance,
                ..BooleanOptions::default()
            },
        )
        .expect_err("invalid tolerance must be rejected");
        assert_eq!(error, BooleanError::InvalidTolerance);
    }

    let invalid =
        QuadPath::from_points(vec![[0.0, 0.0, 0.0], [0.5, 0.0, 0.0], [f64::NAN, 0.0, 0.0]])
            .expect("shared-anchor shape is structurally valid");
    assert!(matches!(
        path_boolean(
            &invalid,
            &QuadPath::new(),
            BooleanOperation::Union,
            BooleanOptions::default()
        ),
        Err(BooleanError::InvalidCoordinate { point: 2, .. })
    ));
}

#[test]
fn adversarial_rectangle_corpus_matches_point_in_fill_oracle() {
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;
    for case in 0..96 {
        let mut next = || {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            f64::from((state >> 40) as u32) / f64::from(1_u32 << 24)
        };
        let ax = next() * 6.0 - 3.0;
        let ay = next() * 6.0 - 3.0;
        let bx = next() * 6.0 - 3.0;
        let by = next() * 6.0 - 3.0;
        let a = rectangle(ax, ay, ax + 0.2 + next() * 2.0, ay + 0.2 + next() * 2.0);
        let b = rectangle(bx, by, bx + 0.2 + next() * 2.0, by + 0.2 + next() * 2.0);
        let samples: Vec<Point> = (0..17)
            .map(|sample| {
                let x = -3.75 + f64::from(sample) * 0.47 + f64::from(case) * 1.0e-7;
                let y =
                    -3.25 + f64::from((sample * 7 + case) % 17) * 0.43 + f64::from(case) * 3.0e-7;
                [x, y]
            })
            .collect();
        for operation in [
            BooleanOperation::Union,
            BooleanOperation::Intersection,
            BooleanOperation::Difference,
            BooleanOperation::Exclusion,
        ] {
            let result = run_flattened(&a, &b, operation);
            assert_grid_truth(&result.path, &a, &b, operation, &samples);
        }
    }
}

#[test]
fn concave_rotated_and_self_crossing_polygon_corpus_matches_oracle() {
    let shapes = [
        path_from_contours(&[&[[-2.0, -1.0], [1.7, -0.6], [0.4, 2.1]]]),
        path_from_contours(&[&[[0.0, -2.4], [2.2, 0.0], [0.0, 2.4], [-2.2, 0.0]]]),
        path_from_contours(&[&[
            [-2.4, -2.0],
            [0.3, -2.0],
            [0.3, -0.2],
            [2.2, -0.2],
            [2.2, 2.0],
            [-2.4, 2.0],
        ]]),
        path_from_contours(&[&[
            [0.0, 2.5],
            [0.7, -1.8],
            [-2.3, 0.8],
            [2.3, 0.8],
            [-0.7, -1.8],
        ]]),
        path_from_contours(&[&[
            [-2.5, -0.7],
            [-0.8, -2.2],
            [1.9, -1.4],
            [2.4, 1.3],
            [-0.4, 2.3],
        ]]),
    ];
    let samples: Vec<Point> = (0..15)
        .flat_map(|y| {
            (0..17).map(move |x| [-2.83 + 0.347 * f64::from(x), -2.61 + 0.389 * f64::from(y)])
        })
        .collect();
    for (left, subject) in shapes.iter().enumerate() {
        for (right, clip) in shapes.iter().enumerate().skip(left + 1) {
            for operation in [
                BooleanOperation::Union,
                BooleanOperation::Intersection,
                BooleanOperation::Difference,
                BooleanOperation::Exclusion,
            ] {
                let result = run_flattened(subject, clip, operation);
                assert_grid_truth(&result.path, subject, clip, operation, &samples);
                assert!(
                    result.stats.output_segments <= BooleanLimits::DEFAULT.max_output_segments,
                    "shape pair {left}/{right} escaped the output budget"
                );
            }
        }
    }
}

#[test]
fn curve_flattening_is_raster_equivalent_across_scales_away_from_boundary() {
    let circle = QuadPath::try_arc(0.0, fmn_core::constants::TAU, 2.0, [0.0, 0.0, 0.0], None)
        .expect("valid arc");
    let clip = rectangle(-1.0, -3.0, 1.0, 3.0);
    for operation in [
        BooleanOperation::Union,
        BooleanOperation::Intersection,
        BooleanOperation::Difference,
        BooleanOperation::Exclusion,
    ] {
        let mut segment_counts = Vec::new();
        for tolerance in [0.05, 0.01, 0.002] {
            let result = path_boolean(
                &circle,
                &clip,
                operation,
                BooleanOptions {
                    tolerance,
                    ..BooleanOptions::default()
                },
            )
            .expect("multi-scale curve fixture must produce a boolean");
            segment_counts.push(result.stats.flattened_segments);
            for y in 0..23 {
                for x in 0..25 {
                    let point = [-2.91 + 0.241 * f64::from(x), -2.67 + 0.257 * f64::from(y)];
                    let radius_squared = point[0] * point[0] + point[1] * point[1];
                    if (radius_squared - 4.0).abs() < 0.18 || (point[0].abs() - 1.0).abs() < 0.06 {
                        continue;
                    }
                    assert_eq!(
                        contains(&result.path, point),
                        expected(
                            operation,
                            radius_squared < 4.0,
                            point[0] > -1.0 && point[0] < 1.0 && point[1] > -3.0 && point[1] < 3.0,
                        ),
                        "{operation:?} drifted at tolerance {tolerance} and point {point:?}"
                    );
                }
            }
        }
        assert!(
            segment_counts.windows(2).all(|pair| pair[0] <= pair[1]),
            "tighter tolerances must not reduce flattening work"
        );
    }
}

#[test]
fn separated_control_hulls_preserve_curves_and_match_forced_fallback() {
    let subject = QuadPath::try_arc(0.0, fmn_core::constants::TAU, 1.5, [-4.0, 0.0, 7.0], None)
        .expect("valid arc");
    let clip = QuadPath::try_arc(0.0, fmn_core::constants::TAU, 1.25, [4.0, 0.0, -3.0], None)
        .expect("valid arc");
    let samples: Vec<Point> = (0..17)
        .flat_map(|y| {
            (0..31).map(move |x| [-7.13 + 0.47 * f64::from(x), -3.11 + 0.39 * f64::from(y)])
        })
        .collect();

    for operation in [
        BooleanOperation::Union,
        BooleanOperation::Intersection,
        BooleanOperation::Difference,
        BooleanOperation::Exclusion,
    ] {
        let selected = path_boolean(&subject, &clip, operation, BooleanOptions::default())
            .expect("separated curve class must be admitted");
        let fallback =
            path_boolean_flattened(&subject, &clip, operation, BooleanOptions::default())
                .expect("forced fallback must remain available");
        assert_eq!(selected.route, BooleanRoute::CurveAwareSeparated);
        assert_eq!(fallback.route, BooleanRoute::FlattenClip);
        assert_eq!(selected.stats.flattened_segments, 0);
        assert!(fallback.stats.flattened_segments > 0);
        for &point in &samples {
            assert_eq!(
                contains(&selected.path, point),
                contains(&fallback.path, point),
                "{operation:?}: separated curve route differs at {point:?}"
            );
        }
        assert!(selected.path.points().iter().all(|point| point[2] == 0.0));
    }

    let union = path_boolean(
        &subject,
        &clip,
        BooleanOperation::Union,
        BooleanOptions::default(),
    )
    .expect("separated union is admitted");
    assert_eq!(union.path.subpaths().len(), 2);
    assert!(
        union
            .path
            .subpaths()
            .iter()
            .all(|subpath| subpath.first() == subpath.last())
    );
    assert!(
        union.path.bezier_tuples().any(|[p0, handle, p2]| {
            let midpoint = [
                p0[0] + (p2[0] - p0[0]) * 0.5,
                p0[1] + (p2[1] - p0[1]) * 0.5,
                0.0,
            ];
            handle != midpoint
        }),
        "curve-aware output must retain non-linear quadratic handles"
    );

    let touching = rectangle(-1.0, -1.0, 0.0, 1.0);
    let touched = rectangle(0.0, -1.0, 1.0, 1.0);
    assert_eq!(
        run(&touching, &touched, BooleanOperation::Union).route,
        BooleanRoute::FlattenClip,
        "bound equality is the written tangency refusal"
    );
    let separated_rect = rectangle(3.0, -1.0, 4.0, 1.0);
    let even_odd = path_boolean(
        &touching,
        &separated_rect,
        BooleanOperation::Union,
        BooleanOptions {
            subject_fill_rule: FillRule::EvenOdd,
            ..BooleanOptions::default()
        },
    )
    .expect("unsupported fill-rule class falls back");
    assert_eq!(even_odd.route, BooleanRoute::FlattenClip);
}

fn parse_fixture_contours(encoded: &str) -> Vec<Vec<Point>> {
    if encoded == "-" {
        return Vec::new();
    }
    encoded
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
        .collect()
}

fn fixture_path(encoded: &str) -> QuadPath {
    let contours = parse_fixture_contours(encoded);
    let borrowed: Vec<&[Point]> = contours.iter().map(Vec::as_slice).collect();
    path_from_contours(&borrowed)
}

fn fixture_operation(encoded: &str) -> Option<BooleanOperation> {
    match encoded {
        "union" => Some(BooleanOperation::Union),
        "intersection" => Some(BooleanOperation::Intersection),
        "difference" => Some(BooleanOperation::Difference),
        "exclusion" => Some(BooleanOperation::Exclusion),
        _ => None,
    }
}

#[test]
fn captured_skia_pathops_fixtures_match_topology() {
    const FIXTURES: &str = include_str!("../fixtures/path_booleans.txt");
    let mut cases = 0;
    for line in FIXTURES
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(fields.len(), 8, "malformed fixture row");
        let name = fields[0];
        let operation = fixture_operation(fields[1]);
        assert!(
            operation.is_some(),
            "{name}: unknown fixture operation {}",
            fields[1]
        );
        let operation = operation.unwrap_or(BooleanOperation::Union);
        let subject = fixture_path(fields[2]);
        let clip = fixture_path(fields[3]);
        let expected_area: f64 = fields[4].parse().expect("fixture area is numeric");
        let expected_contours: usize = fields[5].parse().expect("fixture count is numeric");
        let expected_fill = fields[6].as_bytes();
        assert_eq!(expected_fill.len(), 19 * 17);
        assert!(!fields[7].is_empty(), "Skia command audit trail is present");

        let result = run_flattened(&subject, &clip, operation);
        assert!(
            (signed_area(&result.path) - expected_area).abs() < 1.0e-10,
            "{name}: area differs from captured skia-pathops topology"
        );
        // A zero-area point tangency may be serialized as one self-touching
        // contour (Skia) or two closed contours sharing a vertex (ours).
        // The filled set, area, and connectivity are identical; §7.4 rejects
        // point-wise command-stream identity as an acceptance criterion.
        assert_eq!(
            result.path.has_points(),
            expected_contours != 0,
            "{name}: empty/nonempty topology differs from skia-pathops"
        );
        for y in 0..17 {
            for x in 0..19 {
                let point = [-3.137 + 0.371 * f64::from(x), -3.083 + 0.409 * f64::from(y)];
                let expected = expected_fill[y as usize * 19 + x as usize] == b'1';
                assert_eq!(
                    contains(&result.path, point),
                    expected,
                    "{name}: captured point-in-fill mismatch at {point:?}"
                );
            }
        }
        cases += 1;
    }
    assert_eq!(cases, 10);
}

#[test]
fn fuzz_probe_accepts_arbitrary_byte_shapes_under_tight_budgets() {
    for length in 0..=257 {
        let bytes: Vec<u8> = (0..length)
            .map(|index| (index as u8).wrapping_mul(73).wrapping_add(length as u8))
            .collect();
        assert!(fmn_geom::boolean::fuzz_probe(&bytes));
    }
    assert!(fmn_geom::boolean::fuzz_probe(&[0xff; 1_024]));
}
