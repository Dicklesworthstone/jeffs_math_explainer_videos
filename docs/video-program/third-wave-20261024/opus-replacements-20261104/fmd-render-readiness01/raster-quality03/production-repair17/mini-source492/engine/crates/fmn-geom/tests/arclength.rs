//! fm-xci acceptance: arc-length oracles against closed forms, the
//! inverse-LUT property, constant-speed metamorphic behavior, and the
//! geometry-revision-only cache invalidation.

use fmn_core::constants::TAU;
use fmn_geom::arclength::quadratic_arc_length;
use fmn_geom::{ArcLengthTable, CachedArcLength, QuadPath};

fn p(x: f64, y: f64) -> [f64; 3] {
    [x, y, 0.0]
}

/// Reference numeric integration: fine composite Simpson on |B'(t)| —
/// the independent oracle the closed form must agree with.
fn simpson_length(a0: [f64; 3], h: [f64; 3], a1: [f64; 3]) -> f64 {
    let speed = |t: f64| -> f64 {
        let d = [
            2.0 * ((1.0 - t) * (h[0] - a0[0]) + t * (a1[0] - h[0])),
            2.0 * ((1.0 - t) * (h[1] - a0[1]) + t * (a1[1] - h[1])),
            2.0 * ((1.0 - t) * (h[2] - a0[2]) + t * (a1[2] - h[2])),
        ];
        (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
    };
    let n = 20_000;
    let step = 1.0 / n as f64;
    let mut sum = speed(0.0) + speed(1.0);
    for i in 1..n {
        let weight = if i % 2 == 1 { 4.0 } else { 2.0 };
        sum += weight * speed(i as f64 * step);
    }
    sum * step / 3.0
}

#[test]
fn closed_form_matches_numeric_oracle() {
    let cases = [
        (p(0.0, 0.0), p(1.0, 1.0), p(2.0, 0.0)),    // parabolic arc
        (p(0.0, 0.0), p(0.5, 0.0), p(1.0, 0.0)),    // straight line
        (p(-1.0, 2.0), p(3.0, 5.0), p(0.5, -4.0)),  // generic
        (p(0.0, 0.0), p(1e-3, 1e-3), p(2e-3, 0.0)), // tiny
        ([0.0, 0.0, 1.0], [1.0, 2.0, -1.0], [2.0, 0.0, 3.0]), // spatial
    ];
    for (a0, h, a1) in cases {
        let exact = quadratic_arc_length(a0, h, a1);
        let numeric = simpson_length(a0, h, a1);
        assert!(
            (exact - numeric).abs() <= 1e-9 * (1.0 + numeric),
            "closed form {exact} vs Simpson {numeric} for {a0:?} {h:?} {a1:?}"
        );
    }
}

#[test]
fn line_and_degenerate_lengths_are_exact() {
    // Straight line (handle at midpoint): exactly the chord.
    assert_eq!(
        quadratic_arc_length(p(0.0, 0.0), p(1.5, 0.0), p(3.0, 0.0)),
        3.0
    );
    // Point (all coincident): zero.
    assert_eq!(
        quadratic_arc_length(p(2.0, 2.0), p(2.0, 2.0), p(2.0, 2.0)),
        0.0
    );
    // Interior cusp: out 0.5 and back — total length exactly 1.
    let cusp = quadratic_arc_length(p(0.0, 0.0), p(1.0, 0.0), p(0.0, 0.0));
    assert!((cusp - 1.0).abs() < 1e-15, "cusp length {cusp}");
    // Collinear but forward (handle off-midpoint): still the chord.
    let skewed = quadratic_arc_length(p(0.0, 0.0), p(0.25, 0.0), p(1.0, 0.0));
    assert!((skewed - 1.0).abs() < 1e-12, "skewed-line length {skewed}");
}

#[test]
fn near_straight_curves_do_not_cancel_to_zero() {
    // The regression this guards: a segment sampled at two nearby points
    // and scaled up (a tangent line, a trimmed dash, any zoom) has a
    // handle that is the midpoint to within rounding. Its quadratic term
    // is ~1e-33 of its linear one, and the general antiderivative's two
    // terms — each ~1e16 — used to cancel to exactly zero. Now the
    // near-straight case is detected relatively and integrated as the
    // linear-speed curve it is.
    let cases = [
        (
            p(1.099_550_882_214_647_8, 4.095_488_941_355_138),
            p(-1.500_235_056_200_356, 2.598_454_180_763_514_6),
            p(-4.100_020_994_615_36, 1.101_419_420_171_891_2),
        ),
        // Deliberately perturbed midpoints across several magnitudes.
        (p(0.0, 0.0), p(500.0 + 1e-9, 1e-9), p(1000.0, 0.0)),
        (p(0.0, 0.0), p(0.5, 1e-12), p(1.0, 0.0)),
        (p(-1e6, 0.0), p(0.0, 1e-7), p(1e6, 0.0)),
    ];
    for (a0, h, a1) in cases {
        let closed = quadratic_arc_length(a0, h, a1);
        let numeric = simpson_length(a0, h, a1);
        assert!(
            closed > 0.0,
            "near-straight curve reported zero length: {a0:?} {h:?} {a1:?}"
        );
        assert!(
            (closed - numeric).abs() <= 1e-9 * numeric.max(1.0),
            "closed {closed} vs numeric {numeric}"
        );
    }
}

#[test]
fn the_near_straight_branch_meets_the_general_one() {
    // Sweep the quadratic term down through the branch threshold: the
    // reported length has to stay continuous across it, or a curve would
    // change length as it is scaled.
    let mut previous = f64::NAN;
    for k in 4..24 {
        let bend = 10f64.powi(-k);
        let length = quadratic_arc_length(p(0.0, 0.0), p(0.5, bend), p(1.0, 0.0));
        assert!(length >= 1.0 - 1e-12, "bend {bend}: length {length}");
        if previous.is_finite() {
            assert!(
                (length - previous).abs() < 1e-3,
                "discontinuity at bend {bend}: {previous} -> {length}"
            );
        }
        previous = length;
    }
    assert!((previous - 1.0).abs() < 1e-12, "flat limit {previous}");
}

#[test]
fn parabola_matches_analytic_formula() {
    // (0,0) h(1,1) (2,0) is y = x(2−x)/… as a Bézier: B(t) = (2t, 2t(1−t)).
    // Speed = 2√(1 + (1−2t)²·…) — use the standard parabola arc-length
    // closed form via substitution u = 1−2t: L = ∫₀¹ 2√(1+(2−4t+…)) …
    // Simpler: independently, L = √5 + asinh(2)/… — compute with the
    // textbook formula for y = f(x): ∫₀² √(1 + (1−x)²) dx
    //   = [ (x−1)/2·√(1+(x−1)²) + asinh(x−1)/2 ]₀²
    //   = √2 + asinh(1).
    let expected = 2.0f64.sqrt() + 1.0f64.asinh();
    let got = quadratic_arc_length(p(0.0, 0.0), p(1.0, 1.0), p(2.0, 0.0));
    assert!(
        (got - expected).abs() < 1e-12,
        "parabola: {got} vs analytic {expected}"
    );
}

#[test]
fn full_circle_approximation_length_is_close_to_tau() {
    // The BN-09 16-component circle: the quadratic spline's true length
    // is close to (and distinct from) the ideal circumference.
    let circle = QuadPath::try_arc(0.0, TAU, 1.0, [0.0; 3], None).expect("valid arc");
    let len = circle.get_arc_length();
    assert!((len - TAU).abs() < 2e-3, "spline circumference {len}");
    // The Reference's get_arc_length (chord/handle blend) would land
    // farther out; ours integrates the actual curves.
}

#[test]
fn inverse_lut_round_trips() {
    let mut path = QuadPath::new();
    path.set_points_as_corners(&[p(0.0, 0.0), p(3.0, 0.0), p(3.0, 1.0), p(7.0, 1.0)])
        .unwrap();
    let table = ArcLengthTable::for_path(&path);
    let total = table.total();
    assert!((total - 8.0).abs() < 1e-12); // 3 + 1 + 4

    // point_from_proportion(L(t)/total) lands back at parameter ≈ t.
    for i in 0..=20 {
        let alpha = i as f64 / 20.0;
        let (index, t) = table.curve_and_t_at(&path, alpha).unwrap();
        // Reconstruct accumulated length at (index, t) and compare.
        let mut acc: f64 = table.curve_lengths()[..index].iter().sum();
        let [a0, h, a1] = path.nth_curve_points(index).unwrap();
        let sub = fmn_geom::bezier::partial_quadratic(&[a0, h, a1], 0.0, t);
        acc += quadratic_arc_length(sub[0], sub[1], sub[2]);
        assert!(
            (acc - alpha * total).abs() < 1e-9 * (1.0 + total),
            "alpha {alpha}: accumulated {acc} vs target {}",
            alpha * total
        );
    }
}

#[test]
fn constant_speed_metamorphic() {
    // Equal proportion steps ⇒ equal arc steps, even on a wildly uneven
    // path (a short curve followed by a long one).
    let mut path = QuadPath::new();
    path.start_new_path(p(0.0, 0.0));
    path.add_quadratic_bezier_curve_to(p(0.05, 0.1), p(0.1, 0.0), true)
        .unwrap();
    path.add_quadratic_bezier_curve_to(p(5.0, 6.0), p(10.0, 0.0), true)
        .unwrap();
    let table = ArcLengthTable::for_path(&path);
    let n = 64;
    let points: Vec<[f64; 3]> = (0..=n)
        .map(|i| {
            path.point_from_proportion_with(&table, i as f64 / n as f64)
                .unwrap()
        })
        .collect();
    // Measure each step's arc length with a fine polyline (chord sums on
    // proportion substeps), and compare steps against the mean.
    let step_arc = |i: usize| -> f64 {
        let sub = 64;
        let mut sum = 0.0;
        let mut prev = points[i];
        for k in 1..=sub {
            let alpha = (i as f64 + k as f64 / sub as f64) / n as f64;
            let q = path.point_from_proportion_with(&table, alpha).unwrap();
            sum += ((q[0] - prev[0]).powi(2) + (q[1] - prev[1]).powi(2) + (q[2] - prev[2]).powi(2))
                .sqrt();
            prev = q;
        }
        sum
    };
    let steps: Vec<f64> = (0..n).map(step_arc).collect();
    let mean = steps.iter().sum::<f64>() / steps.len() as f64;
    for (i, s) in steps.iter().enumerate() {
        assert!(
            (s - mean).abs() < 0.02 * mean,
            "step {i}: arc {s} deviates from mean {mean}"
        );
    }
    // And the quick approximation demonstrably does NOT have this
    // property on this path (it is the labeled heuristic, not the truth).
    let quick_first = path.quick_point_from_proportion(0.5).unwrap();
    let true_first = path.point_from_proportion_with(&table, 0.5).unwrap();
    assert!(
        (quick_first[0] - true_first[0]).abs() > 1.0,
        "quick and true proportions should diverge on uneven paths"
    );
}

#[test]
fn cache_rebuilds_on_geometry_revision_only() {
    let mut path = QuadPath::new();
    path.set_points_as_corners(&[p(0.0, 0.0), p(1.0, 0.0), p(2.0, 1.0)])
        .unwrap();
    let mut cache = CachedArcLength::new();

    // Geometry revision 1: first build.
    let total_before = cache.get(1, &path).total();
    assert_eq!(cache.rebuilds(), 1);
    // Transform/style revisions do not touch the geometry key: no rebuild.
    let _ = cache.get(1, &path);
    let _ = cache.get(1, &path);
    assert_eq!(cache.rebuilds(), 1);

    // A geometry edit bumps the revision: rebuild reflects new lengths.
    path.add_line_to(p(2.0, 5.0), true).unwrap();
    let total_after = cache.get(2, &path).total();
    assert_eq!(cache.rebuilds(), 2);
    assert!(total_after > total_before);
}

#[test]
fn degenerate_paths() {
    // Empty path.
    let empty = QuadPath::new();
    assert_eq!(empty.get_arc_length(), 0.0);
    assert_eq!(empty.point_from_proportion(0.5), None);
    // Single point: proportion returns the point.
    let point = QuadPath::from_points(vec![p(1.0, 2.0)]).unwrap();
    assert_eq!(point.get_arc_length(), 0.0);
    assert_eq!(point.point_from_proportion(0.7), Some(p(1.0, 2.0)));
    // Path with a null curve: zero-length curve contributes nothing and
    // never traps the inverse.
    let mut path = QuadPath::new();
    path.start_new_path(p(0.0, 0.0));
    path.add_line_to(p(0.0, 0.0), true).unwrap();
    path.add_line_to(p(4.0, 0.0), true).unwrap();
    assert!((path.get_arc_length() - 4.0).abs() < 1e-12);
    let mid = path.point_from_proportion(0.5).unwrap();
    assert!((mid[0] - 2.0).abs() < 1e-9);
}
