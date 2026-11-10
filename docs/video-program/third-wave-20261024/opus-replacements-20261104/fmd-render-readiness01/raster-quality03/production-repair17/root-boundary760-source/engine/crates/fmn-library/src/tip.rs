//! The tip-attachment algebra (`TipableVMobject`, §12.1).
//!
//! The Reference's `TipableVMobject` adds a tip by rotating it to face
//! along the path, moving its point onto the path's end, and then pulling
//! the path back so it stops at the tip's *base* rather than running under
//! the head (`geometry.py:71-129`).
//!
//! # Placing by true length (BN-03)
//!
//! The Reference positions a tip from the last two control points, which
//! on a curved path is the last **component's** tangent, and shortens the
//! shaft with `put_start_and_end_on`, which measures the chord. On a
//! `CurvedArrow` that is visibly wrong at large angles: the head tilts off
//! the curve. Here the direction comes from the true tangent at the path's
//! end and the shaft is trimmed by **true arc length** (§7.3), so the tip
//! sits on the actual end of the actual curve and points the way the curve
//! is going. Behaviour Note BN-03 covers the difference.

use fmn_core::constants::PI;
use fmn_core::types::Vec3;
use fmn_geom::{ArcLengthTable, QuadPath, space_ops};

use crate::poly::{ArrowTip, tip_angle, tip_base, tip_point};
use crate::vmobject::VMobject;

/// Which end of a path a tip goes on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipEnd {
    /// The path's first point (`at_start=True`).
    Start,
    /// The path's last point.
    End,
}

/// Attach a tip to one end of `shape`, returning the shape with the tip as
/// its first-class child and its own path trimmed back to the tip's base.
///
/// Mirrors `TipableVMobject.add_tip`: create, position, re-end, add.
#[must_use]
pub fn attach_tip(shape: VMobject, tip: ArrowTip, end: TipEnd) -> VMobject {
    attach_built_tip(shape, tip.build(), end)
}

/// Attach an already-built tip value to one end of `shape`.
///
/// This is the object-model-neutral half of [`attach_tip`]: front doors that
/// already own a first-class tip object can reuse Atlas's one placement and
/// trimming implementation without rebuilding or replacing that object.
#[must_use]
pub fn attach_built_tip(shape: VMobject, tip: VMobject, end: TipEnd) -> VMobject {
    let positioned = position_tip(&shape, tip, end);
    let trimmed = trim_to_tip(shape, &positioned, end);
    trimmed.with_child(positioned)
}

/// Rotate and translate an already-built tip onto one end of `shape`.
///
/// Empty or malformed paths leave the tip unchanged, matching the total
/// behavior of [`attach_tip`] for degenerate shafts.
#[must_use]
pub fn position_tip(shape: &VMobject, tip: VMobject, end: TipEnd) -> VMobject {
    let Ok(path) = shape.path() else {
        return tip;
    };
    if !path.has_points() {
        return tip;
    }

    let anchor = match end {
        TipEnd::Start => path.points()[0],
        TipEnd::End => *path.points().last().expect("non-empty"),
    };
    let outward = outward_tangent(&path, end);
    position_tip_at(tip, anchor, outward)
}

/// Trim `shape` so its selected end stops at the positioned tip's base.
///
/// The cut uses the same true-arc-length rule as [`attach_tip`] (BN-03).
#[must_use]
pub fn trim_to_tip(shape: VMobject, tip: &VMobject, end: TipEnd) -> VMobject {
    trim_to(shape, tip_base(tip), end)
}

/// The unit tangent at one end of the path, pointing *out* of it.
///
/// This is the true tangent — the derivative of the terminal quadratic —
/// not the chord between the last two control points, so a tip on a curve
/// lines up with the curve. Empty and point-only paths have no direction and
/// return the zero vector.
#[must_use]
pub fn outward_tangent(path: &QuadPath, end: TipEnd) -> Vec3 {
    let points = path.points();
    if points.is_empty() {
        return [0.0; 3];
    }
    let fallback = |a: Vec3, b: Vec3| space_ops::normalize(sub(a, b));
    match end {
        TipEnd::End => {
            let n = points.len();
            if n < 3 {
                return fallback(points[n - 1], points[0]);
            }
            // B'(1) = 2 (a1 - h) for the last component.
            let t = sub(points[n - 1], points[n - 2]);
            if space_ops::get_norm(t) > 0.0 {
                space_ops::normalize(t)
            } else {
                fallback(points[n - 1], points[0])
            }
        }
        TipEnd::Start => {
            if points.len() < 3 {
                return fallback(points[0], points[points.len() - 1]);
            }
            // B'(0) = 2 (h - a0), reversed to point out of the path.
            let t = sub(points[0], points[1]);
            if space_ops::get_norm(t) > 0.0 {
                space_ops::normalize(t)
            } else {
                fallback(points[0], *points.last().expect("non-empty"))
            }
        }
    }
}

/// Rotate `tip` to point along `outward` and move its point onto `anchor`
/// (the Reference's `position_tip`).
fn position_tip_at(tip: VMobject, anchor: Vec3, outward: Vec3) -> VMobject {
    let angle = space_ops::angle_of_vector(outward);
    let center = tip.center_point();
    let current = tip_angle_of(&tip);
    let turned = tip.rotated_about(angle - current, fmn_core::constants::OUT, center);
    let point = tip_point(&turned);
    turned.shifted(sub(anchor, point))
}

fn tip_angle_of(tip: &VMobject) -> f64 {
    tip_angle(tip)
}

/// Trim the shaft so it ends at `base` instead of running under the head.
///
/// The Reference calls `put_start_and_end_on`, which rescales the whole
/// path by the ratio of chord lengths. That is right for a straight
/// segment and wrong for a curved one — it shrinks the curve toward its
/// chord. Ours cuts the path by **true arc length** at the proportion the
/// base sits at, so a curved shaft keeps its curvature and simply stops
/// earlier.
fn trim_to(shape: VMobject, base: Vec3, end: TipEnd) -> VMobject {
    let Ok(path) = shape.path() else {
        return shape;
    };
    let total = path.get_arc_length();
    if total <= 0.0 {
        return shape;
    }
    let anchor = match end {
        TipEnd::Start => path.points()[0],
        TipEnd::End => *path.points().last().expect("non-empty"),
    };
    // How far along the path the tip's base sits, as a length fraction.
    let cut = (space_ops::get_norm(sub(base, anchor)) / total).clamp(0.0, 1.0);
    let (a, b) = match end {
        TipEnd::Start => (cut, 1.0),
        TipEnd::End => (0.0, 1.0 - cut),
    };
    if b <= a {
        return shape;
    }
    let table = ArcLengthTable::for_path(&path);
    let n_curves = path.num_curves();
    let to_index = |alpha: f64| -> f64 {
        match table.curve_and_t_at(&path, alpha.clamp(0.0, 1.0)) {
            Some((curve, t)) => (curve as f64 + t) / n_curves as f64,
            None => alpha,
        }
    };
    match QuadPath::partial_points(path.points(), to_index(a), to_index(b)) {
        Some((points, _, _)) => {
            let style = shape.style();
            let children: Vec<VMobject> = shape.children().to_vec();
            VMobject::from_points(points)
                .with_style(style)
                .with_children(children)
        }
        None => shape,
    }
}

/// The Reference's `get_length`: the straight distance between the ends.
#[must_use]
pub fn end_to_end_length(shape: &VMobject) -> f64 {
    let points = shape.points();
    match (points.first(), points.last()) {
        (Some(a), Some(b)) => space_ops::get_norm(sub(*a, *b)),
        _ => 0.0,
    }
}

/// The angle a tip at `end` would be rotated to, for tests and for classes
/// that need the direction without building the tip.
#[must_use]
pub fn tip_direction(path: &QuadPath, end: TipEnd) -> f64 {
    let out = outward_tangent(path, end);
    space_ops::angle_of_vector(out)
}

/// The Reference's `PI`-offset convention, exposed so `Arrow`-family code
/// can reason about tip orientation without duplicating the constant.
#[must_use]
pub fn reversed(angle: f64) -> f64 {
    angle - PI
}

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ArcBetweenPoints;
    use crate::poly::tip_vector;
    use crate::style::Style;
    use fmn_core::constants::TAU;

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() < tol
    }

    fn straight(from: Vec3, to: Vec3) -> VMobject {
        let mut path = QuadPath::new();
        let _ = path.set_points_as_corners(&[from, to]);
        VMobject::from_path(&path)
    }

    #[test]
    fn a_tip_lands_on_the_end_and_points_along_it() {
        let shaft = straight([0.0; 3], [4.0, 0.0, 0.0]);
        let tipped = attach_tip(shaft, ArrowTip::new(), TipEnd::End);
        let tip = tipped.children().last().expect("tip attached");
        assert!(
            close(tip_point(tip)[0], 4.0, 1e-9),
            "tip point at {:?}",
            tip_point(tip)
        );
        let v = tip_vector(tip);
        assert!(v[0] > 0.0 && close(v[1], 0.0, 1e-9), "tip vector {v:?}");
    }

    #[test]
    fn the_shaft_stops_at_the_tip_base() {
        let shaft = straight([0.0; 3], [4.0, 0.0, 0.0]);
        let tipped = attach_tip(shaft, ArrowTip::new().length(0.5), TipEnd::End);
        let shaft_end = *tipped.points().last().expect("shaft points");
        // The shaft ends about one tip-length short of the arrow point.
        assert!(
            close(shaft_end[0], 3.5, 1e-6),
            "shaft ends at {shaft_end:?}, expected x ≈ 3.5"
        );
    }

    #[test]
    fn built_tip_placement_and_trim_are_reusable_without_replacing_the_tip() {
        let shaft = straight([0.0; 3], [4.0, 0.0, 0.0]);
        let tip = ArrowTip::new().length(0.5).build();
        let positioned = position_tip(&shaft, tip, TipEnd::End);
        assert!(close(tip_point(&positioned)[0], 4.0, 1e-9));

        let trimmed = trim_to_tip(shaft.clone(), &positioned, TipEnd::End);
        assert!(close(
            trimmed.points().last().expect("shaft end")[0],
            3.5,
            1e-6
        ));

        let attached = attach_built_tip(shaft, positioned, TipEnd::End);
        assert_eq!(attached.children().len(), 1);
        assert!(close(
            tip_point(attached.children().first().expect("tip"))[0],
            4.0,
            1e-9
        ));
    }

    #[test]
    fn a_start_tip_points_the_other_way() {
        let shaft = straight([0.0; 3], [4.0, 0.0, 0.0]);
        let tipped = attach_tip(shaft, ArrowTip::new(), TipEnd::Start);
        let tip = tipped.children().last().expect("tip attached");
        assert!(close(tip_point(tip)[0], 0.0, 1e-9));
        assert!(tip_vector(tip)[0] < 0.0, "start tip points backward");
        let shaft_start = tipped.points()[0];
        assert!(shaft_start[0] > 0.0, "shaft was pulled back from the start");
    }

    #[test]
    fn a_tip_on_a_curve_follows_the_curve_not_the_chord() {
        // A half-turn arc: the chord direction and the true tangent at the
        // end differ by a right angle, so this is exactly the case the
        // Reference's control-point heuristic gets wrong.
        let arc = ArcBetweenPoints::new([0.0; 3], [2.0, 0.0, 0.0])
            .angle(TAU / 2.0)
            .style(Style::default())
            .build()
            .expect("the fixture arc is valid");
        let path = arc.path().unwrap();
        let end_tangent = outward_tangent(&path, TipEnd::End);
        let tipped = attach_tip(arc, ArrowTip::new(), TipEnd::End);
        let tip = tipped.children().last().expect("tip attached");
        let v = space_ops::normalize(tip_vector(tip));
        assert!(
            space_ops::angle_between_vectors(v, end_tangent) < 1e-6,
            "tip {v:?} vs tangent {end_tangent:?}"
        );
    }

    #[test]
    fn trimming_a_curve_keeps_its_curvature() {
        // Trimming by true length must not rescale the curve toward its
        // chord: the trimmed arc's remaining points stay on the circle.
        let arc = ArcBetweenPoints::new([0.0; 3], [2.0, 0.0, 0.0])
            .angle(TAU / 2.0)
            .build()
            .expect("the fixture arc is valid");
        let center = crate::arc::arc_center_of(arc.points()).unwrap();
        let radius = space_ops::get_norm(sub(arc.points()[0], center));
        let tipped = attach_tip(arc, ArrowTip::new(), TipEnd::End);
        for anchor in tipped.points().iter().step_by(2) {
            let r = space_ops::get_norm(sub(*anchor, center));
            assert!(
                close(r, radius, 1e-3),
                "trimmed anchor off the circle: {r} vs {radius}"
            );
        }
    }

    #[test]
    fn tips_on_degenerate_shafts_do_not_panic() {
        let empty = VMobject::new();
        let tipped = attach_tip(empty, ArrowTip::new(), TipEnd::End);
        assert_eq!(tipped.children().len(), 1);
        let point = VMobject::from_points(vec![[1.0, 1.0, 0.0]; 3]);
        let tipped = attach_tip(point, ArrowTip::new(), TipEnd::End);
        assert_eq!(tipped.children().len(), 1);
    }

    #[test]
    fn tip_direction_queries_are_total_on_degenerate_paths() {
        let empty = QuadPath::new();
        let point = QuadPath::from_points(vec![[1.0, 1.0, 0.0]]).expect("valid point path");

        for end in [TipEnd::Start, TipEnd::End] {
            assert_eq!(outward_tangent(&empty, end), [0.0; 3]);
            assert_eq!(tip_direction(&empty, end), 0.0);
            assert_eq!(outward_tangent(&point, end), [0.0; 3]);
            assert_eq!(tip_direction(&point, end), 0.0);
        }
    }

    #[test]
    fn end_to_end_length_is_the_chord() {
        let shaft = straight([0.0; 3], [3.0, 4.0, 0.0]);
        assert!(close(end_to_end_length(&shaft), 5.0, 1e-12));
        assert_eq!(end_to_end_length(&VMobject::new()), 0.0);
    }
}
