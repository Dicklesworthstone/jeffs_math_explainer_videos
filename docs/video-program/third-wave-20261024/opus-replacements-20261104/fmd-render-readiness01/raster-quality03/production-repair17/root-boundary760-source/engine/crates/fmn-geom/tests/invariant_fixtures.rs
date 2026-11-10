//! The shared-anchor invariant fixtures — the W2 exit rule (§7.1, fm-e3f):
//! exact expectations for empty / one-point / one-curve / closed /
//! multi-subpath paths, plus the adversarial set (zero-length curves,
//! coincident anchors, single-point subpaths). These are FrankenManim's own
//! contracts; the Reference-derived structural fixtures live in
//! `reference_parity.rs`.

use fmn_core::constants::DEG;
use fmn_core::types::Vec3;
use fmn_geom::cubic;
use fmn_geom::{AnchorMode, GeomError, QuadPath};

fn p(x: f64, y: f64) -> Vec3 {
    [x, y, 0.0]
}

// ------------------------------------------------------------------ empty

#[test]
fn empty_path() {
    let path = QuadPath::new();
    assert_eq!(path.num_points(), 0);
    assert!(!path.has_points());
    assert_eq!(path.num_curves(), 0);
    assert!(path.subpath_end_indices().is_empty());
    assert!(path.subpaths().is_empty());
    assert!(!path.has_new_path_started());
    assert!(!path.is_closed());
    assert!(path.joint_angles().is_empty());
    assert_eq!(path.last_point(), None);
    assert!(path.points_without_null_curves(1e-9).is_empty());
}

#[test]
fn empty_path_operations_error() {
    let mut path = QuadPath::new();
    assert_eq!(
        path.add_line_to(p(1.0, 0.0), true).unwrap_err(),
        GeomError::EmptyPath
    );
    assert_eq!(
        path.add_quadratic_bezier_curve_to(p(0.5, 1.0), p(1.0, 0.0), true)
            .unwrap_err(),
        GeomError::EmptyPath
    );
    assert_eq!(
        path.add_cubic_bezier_curve_to(p(0.0, 1.0), p(1.0, 1.0), p(1.0, 0.0))
            .unwrap_err(),
        GeomError::EmptyPath
    );
    assert_eq!(
        path.add_arc_to(p(1.0, 0.0), 1.0, None).unwrap_err(),
        GeomError::EmptyPath
    );
    assert_eq!(path.close_path(false).unwrap_err(), GeomError::EmptyPath);
}

// ------------------------------------------------------- invariant checks

#[test]
fn even_point_runs_are_rejected() {
    assert_eq!(
        QuadPath::from_points(vec![p(0.0, 0.0), p(1.0, 0.0)]).unwrap_err(),
        GeomError::EvenPointCount { len: 2 }
    );
    let mut path = QuadPath::new();
    path.start_new_path(p(0.0, 0.0));
    assert!(matches!(
        path.append_points(&[p(1.0, 0.0)]).unwrap_err(),
        GeomError::EvenPointCount { .. }
    ));
    assert!(matches!(
        path.add_subpath(&[p(1.0, 0.0), p(2.0, 0.0)]).unwrap_err(),
        GeomError::EvenPointCount { .. }
    ));
    // Odd runs are accepted; zero-length set is the empty path.
    assert!(QuadPath::from_points(vec![p(0.0, 0.0)]).is_ok());
    assert!(QuadPath::from_points(Vec::new()).is_ok());
}

#[test]
fn anchors_and_handles_must_interleave() {
    let mut path = QuadPath::new();
    assert_eq!(
        path.set_anchors_and_handles(&[p(0.0, 0.0), p(1.0, 0.0)], &[])
            .unwrap_err(),
        GeomError::MismatchedAnchorsAndHandles {
            anchors: 2,
            handles: 0
        }
    );
    // Empty anchors clears the path.
    path.start_new_path(p(0.0, 0.0));
    path.set_anchors_and_handles(&[], &[]).unwrap();
    assert!(!path.has_points());
}

// -------------------------------------------------------------- one point

#[test]
fn one_point_path_degeneracies() {
    // The VectorizedPoint degeneracy: a legal single-point path.
    let mut path = QuadPath::from_points(vec![p(1.0, 2.0)]).unwrap();
    assert_eq!(path.num_points(), 1);
    assert_eq!(path.num_curves(), 0);
    assert!(path.has_new_path_started());
    assert_eq!(path.subpath_end_indices(), vec![0]);
    assert_eq!(path.subpaths().len(), 1);
    assert_eq!(path.subpaths()[0], &[p(1.0, 2.0)][..]);
    // A single point is its own closed subpath (Reference behavior).
    assert!(path.is_closed());
    assert_eq!(path.joint_angles(), vec![0.0]);
    // Smooth-curve continuation from a fresh path degrades to a line.
    path.add_smooth_curve_to(p(2.0, 2.0)).unwrap();
    assert_eq!(path.num_points(), 3);
    assert_eq!(path.points()[1], p(1.5, 2.0));
}

#[test]
fn one_point_insert_n_curves_repeats() {
    let points = QuadPath::insert_n_curves_to_point_list(2, &[p(3.0, 4.0)], 1e-8).unwrap();
    assert_eq!(points, vec![p(3.0, 4.0); 5]);
}

// -------------------------------------------------------------- one curve

#[test]
fn one_curve_path() {
    let mut path = QuadPath::new();
    path.start_new_path(p(0.0, 0.0));
    path.add_quadratic_bezier_curve_to(p(1.0, 1.0), p(2.0, 0.0), true)
        .unwrap();
    assert_eq!(path.num_points(), 3);
    assert_eq!(path.num_curves(), 1);
    assert_eq!(
        path.nth_curve_points(0).unwrap(),
        [p(0.0, 0.0), p(1.0, 1.0), p(2.0, 0.0)]
    );
    assert_eq!(path.nth_curve_points(1), None);
    assert_eq!(path.nth_curve_point(0, 0.5).unwrap(), p(1.0, 0.5));
    assert_eq!(path.subpath_end_indices(), vec![2]);
    assert!(!path.is_closed());
    assert_eq!(path.anchors(), vec![p(0.0, 0.0), p(2.0, 0.0)]);
    assert_eq!(path.start_anchors(), vec![p(0.0, 0.0)]);
    assert_eq!(path.end_anchors(), vec![p(2.0, 0.0)]);
}

// ------------------------------------------------------------------ closed

#[test]
fn closed_path_detection_and_seam() {
    let mut path = QuadPath::new();
    path.set_points_as_corners(&[p(0.0, 0.0), p(2.0, 0.0), p(1.0, 2.0)])
        .unwrap();
    assert!(!path.is_closed());
    path.close_path(false).unwrap();
    assert!(path.is_closed());
    // Closing an already-closed path is a no-op.
    let count = path.num_points();
    path.close_path(false).unwrap();
    assert_eq!(path.num_points(), count);
    // The seam joins tangents: every anchor of the closed triangle has a
    // nonzero turn, including the seam anchor.
    let angles = path.joint_angles();
    assert!(angles[0].abs() > 1.0 * DEG);
    assert!(angles[angles.len() - 1].abs() > 1.0 * DEG);
}

#[test]
fn closure_respects_tolerance() {
    let mut path = QuadPath::new();
    // End lands within 1e-8 of the start: closed.
    path.set_points_as_corners(&[p(0.0, 0.0), p(2.0, 0.0), p(1.0, 2.0), p(0.5e-8, 0.0)])
        .unwrap();
    assert!(path.is_closed());
    // End lands 1e-4 away: open.
    let mut path = QuadPath::new();
    path.set_points_as_corners(&[p(0.0, 0.0), p(2.0, 0.0), p(1.0, 2.0), p(1e-4, 0.0)])
        .unwrap();
    assert!(!path.is_closed());
}

// --------------------------------------------------------- multi-subpath

#[test]
fn multi_subpath_decomposition() {
    let mut path = QuadPath::new();
    path.start_new_path(p(0.0, 0.0));
    path.add_line_to(p(1.0, 0.0), true).unwrap();
    path.start_new_path(p(2.0, 0.0));
    path.add_line_to(p(3.0, 0.0), true).unwrap();
    path.start_new_path(p(4.0, 0.0));
    path.add_line_to(p(5.0, 0.0), true).unwrap();
    // Layout: 3 one-curve subpaths separated by null-curve break markers:
    // each start_new_path duplicates the previous anchor as a handle.
    assert_eq!(path.num_points(), 11);
    assert_eq!(path.subpath_end_indices(), vec![2, 6, 10]);
    let subpaths = path.subpaths();
    assert_eq!(subpaths.len(), 3);
    assert_eq!(subpaths[0], &[p(0.0, 0.0), p(0.5, 0.0), p(1.0, 0.0)][..]);
    assert_eq!(subpaths[2], &[p(4.0, 0.0), p(4.5, 0.0), p(5.0, 0.0)][..]);
}

#[test]
fn add_subpath_merges_or_splices() {
    // Starting exactly at the current end: continuation, no break marker.
    let mut path = QuadPath::new();
    path.set_points_as_corners(&[p(0.0, 0.0), p(1.0, 0.0)])
        .unwrap();
    path.add_subpath(&[p(1.0, 0.0), p(1.5, 0.5), p(2.0, 1.0)])
        .unwrap();
    assert_eq!(path.num_points(), 5);
    assert_eq!(path.subpaths().len(), 1);
    // Starting elsewhere: a break marker is inserted.
    let mut path = QuadPath::new();
    path.set_points_as_corners(&[p(0.0, 0.0), p(1.0, 0.0)])
        .unwrap();
    path.add_subpath(&[p(5.0, 5.0), p(5.5, 5.5), p(6.0, 6.0)])
        .unwrap();
    assert_eq!(path.num_points(), 7);
    assert_eq!(path.subpaths().len(), 2);
    // Empty subpath: no-op.
    let count = path.num_points();
    path.add_subpath(&[]).unwrap();
    assert_eq!(path.num_points(), count);
}

// ------------------------------------------------------------ adversarial

#[test]
fn zero_length_curves_are_not_breaks() {
    // A null LINE (anchor repeated with midpoint handle on top of it) is a
    // subpath break only when the handle sits on the anchor AND the next
    // anchor is distinct; a genuine null curve run must not split the path.
    let mut path = QuadPath::new();
    path.start_new_path(p(0.0, 0.0));
    path.add_line_to(p(0.0, 0.0), true).unwrap();
    // Handle and both anchors coincide -> next anchor NOT distinct -> the
    // null curve is not an end marker.
    assert_eq!(path.subpath_end_indices(), vec![2]);
    path.add_line_to(p(1.0, 0.0), true).unwrap();
    // Now the null curve's following anchor (via its own handle) is distinct
    // from the coincident pair only through the midpoint handle, which sits
    // at (0.5, 0) — the break test looks at handle-on-anchor, which no
    // longer holds. Single subpath throughout.
    assert_eq!(path.subpaths().len(), 1);
    // Null curves are dropped by points_without_null_curves.
    let cleaned = path.points_without_null_curves(1e-9);
    assert_eq!(cleaned.len(), 3);
}

#[test]
fn coincident_anchor_handle_nudges() {
    // A quadratic whose handle coincides with the current end would forge a
    // break marker; the engine nudges it to the segment midpoint.
    let mut path = QuadPath::new();
    path.start_new_path(p(0.0, 0.0));
    path.add_quadratic_bezier_curve_to(p(0.0, 0.0), p(2.0, 0.0), true)
        .unwrap();
    assert_eq!(path.points()[1], p(1.0, 0.0));
    assert_eq!(path.subpaths().len(), 1);
}

#[test]
fn cubic_api_appends_the_exact_admitted_chain() {
    let controls = [p(0.0, 0.0), p(-1.0, 2.0), p(3.0, -2.0), p(4.0, 1.0)];
    let tolerance = 0.005;
    let expected = cubic::cubic_to_quadratics(
        controls[0],
        controls[1],
        controls[2],
        controls[3],
        tolerance,
    )
    .unwrap();

    let mut path = QuadPath::new();
    path.start_new_path(controls[0]);
    path.add_cubic_bezier_curve_to_with_tolerance(controls[1], controls[2], controls[3], tolerance)
        .unwrap();
    assert_eq!(path.points(), expected);
    assert_eq!(path.subpaths().len(), 1);
}

#[test]
fn simple_cubic_compatibility_knob_cannot_lower_fidelity() {
    let mut canonical = QuadPath::new();
    canonical.start_new_path(p(0.0, 0.0));
    canonical
        .add_cubic_bezier_curve_to(p(1.0, 0.25), p(2.0, 0.5), p(3.0, 1.0))
        .unwrap();

    let mut requested_simple = QuadPath::new();
    requested_simple.set_use_simple_quadratic_approx(true);
    requested_simple.start_new_path(p(0.0, 0.0));
    requested_simple
        .add_cubic_bezier_curve_to(p(1.0, 0.25), p(2.0, 0.5), p(3.0, 1.0))
        .unwrap();

    assert_eq!(requested_simple, canonical);
}

#[test]
fn stationary_cubic_start_is_not_a_false_subpath_break() {
    // Degree elevation of Q(t)=t²: the mathematically exact quadratic handle
    // equals the first anchor. The converter separates it by a bounded ulp so
    // `[anchor, handle, next]` cannot be misread as a break marker.
    let mut path = QuadPath::new();
    path.start_new_path(p(0.0, 0.0));
    path.add_cubic_bezier_curve_to(p(0.0, 0.0), p(1.0 / 3.0, 0.0), p(1.0, 0.0))
        .unwrap();
    assert_ne!(path.points()[0], path.points()[1]);
    assert_eq!(path.subpaths().len(), 1);
    assert_eq!(path.subpath_end_indices(), vec![path.points().len() - 1]);
}

#[test]
fn stationary_cubic_end_keeps_a_distinct_final_handle() {
    // Degree elevation of Q(t)=1-(1-t)² has its exact quadratic handle on the
    // final anchor. Keep the stored handle distinct so downstream anchor-mode
    // cleanup cannot rewrite the converter-admitted chain.
    let mut path = QuadPath::new();
    path.start_new_path(p(0.0, 0.0));
    path.add_cubic_bezier_curve_to(p(2.0 / 3.0, 0.0), p(1.0, 0.0), p(1.0, 0.0))
        .unwrap();
    let points = path.points();
    assert_ne!(points[points.len() - 2], points[points.len() - 1]);
    assert_eq!(path.subpaths().len(), 1);
}

#[test]
fn independently_converted_smooth_cubics_keep_their_join() {
    let mut path = QuadPath::new();
    path.start_new_path(p(0.0, 0.0));
    path.add_cubic_bezier_curve_to(p(0.2, 0.8), p(0.7, 1.0), p(1.0, 1.0))
        .unwrap();
    path.add_smooth_cubic_curve_to(p(1.8, 0.2), p(2.0, 0.0))
        .unwrap();

    assert_eq!(path.subpaths().len(), 1);
    assert!(path.is_smooth(1e-7), "{:?}", path.joint_angles());
}

#[test]
fn rejected_cubic_tolerance_leaves_the_path_unchanged() {
    let mut path = QuadPath::new();
    path.start_new_path(p(0.0, 0.0));
    let before = path.clone();
    assert_eq!(
        path.add_cubic_bezier_curve_to_with_tolerance(p(0.0, 1.0), p(1.0, 1.0), p(1.0, 0.0), 0.0,)
            .unwrap_err(),
        GeomError::InvalidTolerance
    );
    assert_eq!(path, before);
}

#[test]
fn single_point_subpaths() {
    // Two consecutive start_new_path calls create a singleton subpath.
    let mut path = QuadPath::new();
    path.start_new_path(p(0.0, 0.0));
    path.start_new_path(p(1.0, 1.0));
    assert_eq!(path.num_points(), 3);
    assert_eq!(path.subpath_end_indices(), vec![0, 2]);
    let subpaths = path.subpaths();
    assert_eq!(subpaths.len(), 2);
    assert_eq!(subpaths[0], &[p(0.0, 0.0)][..]);
    assert_eq!(subpaths[1], &[p(1.0, 1.0)][..]);
}

#[test]
fn null_curve_disallowed_flags_skip() {
    let mut path = QuadPath::new();
    path.start_new_path(p(0.0, 0.0));
    path.add_line_to(p(0.0, 0.0), false).unwrap();
    assert_eq!(path.num_points(), 1);
    path.add_quadratic_bezier_curve_to(p(0.5, 0.5), p(0.5e-9, 0.0), false)
        .unwrap();
    assert_eq!(path.num_points(), 1);
}

#[test]
fn consider_points_equal_is_strict_per_component() {
    let path = QuadPath::new();
    assert!(path.consider_points_equal(p(0.0, 0.0), p(0.9e-8, 0.0)));
    // Exactly at tolerance: NOT equal (strict comparison).
    assert!(!path.consider_points_equal(p(0.0, 0.0), p(1e-8, 0.0)));
    assert!(!path.consider_points_equal(p(0.0, 0.0), p(0.0, 2e-8)));
}

// ------------------------------------------------------------ anchor modes

#[test]
fn jagged_mode_sets_midpoint_handles() {
    let mut path = QuadPath::new();
    path.start_new_path(p(0.0, 0.0));
    path.add_quadratic_bezier_curve_to(p(0.0, 5.0), p(2.0, 0.0), true)
        .unwrap();
    path.change_anchor_mode(AnchorMode::Jagged).unwrap();
    assert_eq!(path.points()[1], p(1.0, 0.0));
}

#[test]
fn true_smooth_passes_through_original_anchors() {
    let mut path = QuadPath::new();
    path.set_points_as_corners(&[p(0.0, 0.0), p(1.0, 1.0), p(2.0, 0.0), p(3.0, 1.0)])
        .unwrap();
    let original_anchors = path.anchors();
    path.change_anchor_mode(AnchorMode::TrueSmooth).unwrap();
    assert_eq!(path.num_points() % 2, 1);
    for a in original_anchors {
        assert!(
            path.anchors()
                .iter()
                .any(|q| (0..3).all(|k| (q[k] - a[k]).abs() < 1e-9)),
            "anchor {a:?} lost by true_smooth"
        );
    }
    // And the result is smooth at its anchors.
    assert!(path.is_smooth(1.0 * DEG));
}

#[test]
fn true_smooth_rotated_plane_stays_finite_and_smooth() {
    let c = std::f64::consts::FRAC_1_SQRT_2;
    let anchors = [[0.0, 0.0, 0.0], [1.0, c, c], [2.0, 0.0, 0.0], [3.0, c, c]];
    let mut path = QuadPath::new();
    path.set_points_as_corners(&anchors).unwrap();
    path.change_anchor_mode(AnchorMode::TrueSmooth).unwrap();

    assert!(
        path.points()
            .iter()
            .flatten()
            .all(|value| value.is_finite())
    );
    assert_eq!(path.subpaths().len(), 1);
    assert!(path.is_smooth(1.0 * DEG));
    for anchor in anchors {
        assert!(
            path.anchors()
                .iter()
                .any(|candidate| (0..3)
                    .all(|axis| { (candidate[axis] - anchor[axis]).abs() < 1e-9 })),
            "anchor {anchor:?} lost by rotated smoothing"
        );
    }
}

#[test]
fn make_smooth_skips_already_smooth_paths() {
    let mut path = QuadPath::new();
    path.set_points_as_corners(&[p(0.0, 0.0), p(1.0, 0.0), p(2.0, 0.0)])
        .unwrap();
    let before = path.points().to_vec();
    path.make_smooth(true).unwrap();
    assert_eq!(path.points(), &before[..]);
}

// -------------------------------------------------------------- reversal

#[test]
fn reverse_points_preserves_subpath_structure() {
    let mut path = QuadPath::new();
    path.start_new_path(p(0.0, 0.0));
    path.add_line_to(p(1.0, 0.0), true).unwrap();
    path.start_new_path(p(2.0, 0.0));
    path.add_line_to(p(3.0, 0.0), true).unwrap();
    let ends_before = path.subpath_end_indices().len();
    path.reverse_points();
    assert_eq!(path.subpath_end_indices().len(), ends_before);
    assert_eq!(path.last_point().unwrap(), p(0.0, 0.0));
    assert_eq!(path.points()[0], p(3.0, 0.0));
}
