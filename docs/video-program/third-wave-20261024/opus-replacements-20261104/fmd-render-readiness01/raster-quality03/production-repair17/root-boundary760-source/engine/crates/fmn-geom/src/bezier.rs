//! Quadratic-Bézier primitives: evaluation, restriction to a subinterval,
//! and the unit-circle arc construction — ports of `manimlib/utils/bezier.py`
//! (`3b1b/manim` @ `6199a00d`), computed in f64 per §6.1.

use crate::GeomError;
use crate::scalar;
#[cfg(test)]
use crate::space_ops;
use crate::vec;
use fmn_core::constants::TAU;
use fmn_core::types::Vec3;

/// Evaluate the quadratic Bézier `(p0, p1, p2)` at `t`, using the Reference's
/// exact expression `p0(1-t)² + 2 p1 t(1-t) + p2 t²`.
#[must_use]
pub fn quadratic_point(p0: Vec3, p1: Vec3, p2: Vec3, t: f64) -> Vec3 {
    let u = 1.0 - t;
    let mut out = [0.0; 3];
    for i in 0..3 {
        out[i] = p0[i] * u * u + 2.0 * p1[i] * t * u + p2[i] * t * t;
    }
    out
}

/// Evaluate the cubic Bézier `(a0, h0, h1, a1)` at `t` (Bernstein form).
#[must_use]
pub fn cubic_point(a0: Vec3, h0: Vec3, h1: Vec3, a1: Vec3, t: f64) -> Vec3 {
    let u = 1.0 - t;
    let mut out = [0.0; 3];
    for i in 0..3 {
        out[i] = a0[i] * u * u * u
            + 3.0 * h0[i] * t * u * u
            + 3.0 * h1[i] * t * t * u
            + a1[i] * t * t * t;
    }
    out
}

/// `partial_quadratic_bezier_points`: control points for the restriction of a
/// quadratic to the parameter interval `[a, b]`, matching the Reference's
/// formulas (including the `a == 1` triple-endpoint special case).
#[must_use]
pub fn partial_quadratic(points: &[Vec3; 3], a: f64, b: f64) -> [Vec3; 3] {
    if a == 1.0 {
        return [points[2], points[2], points[2]];
    }
    let curve = |t: f64| quadratic_point(points[0], points[1], points[2], t);
    let h0 = if a > 0.0 { curve(a) } else { points[0] };
    let h2 = if b < 1.0 { curve(b) } else { points[2] };
    let h1_prime = vec::lerp(points[1], points[2], a);
    let end_prop = (b - a) / (1.0 - a);
    let h1 = vec::lerp(h0, h1_prime, end_prop);
    [h0, h1, h2]
}

/// `np.linspace(a, b, n)` with the endpoint included, matching numpy's
/// `start + i * step` evaluation.
#[must_use]
pub(crate) fn linspace(a: f64, b: f64, n: usize) -> Vec<f64> {
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![a];
    }
    let step = (b - a) / (n as f64 - 1.0);
    let mut out: Vec<f64> = (0..n).map(|i| a + i as f64 * step).collect();
    // numpy pins the endpoint exactly.
    out[n - 1] = b;
    out
}

/// The declared arc-component budget (fm-4tb.1): no arc may request more
/// quadratic components than this (131,073 points, about 3 MB). Real scenes
/// do ask for dense arcs explicitly: 3b1b's `_2022/visual_proofs/lies.py`
/// draws `Circle(n_components=2**14)`, which the earlier 4096 refused. A
/// request far past this, such as the ~5.6e7 components an unbounded angle
/// implies, is still a caller error or hostile input.
pub const MAX_ARC_COMPONENTS: usize = 65_536;

/// Validate an arc request and return its exact shared-anchor point count.
///
/// This is the one component contract used by Chisel and every higher-level
/// builder (fm-4tb.1): finite angle, nonzero count, representable `2·n + 1`
/// point arithmetic, then the declared component budget. No allocation is
/// performed.
///
/// # Errors
/// The corresponding typed [`GeomError`] for each rejected precondition.
pub fn arc_point_count(angle: f64, n_components: usize) -> Result<usize, GeomError> {
    if !angle.is_finite() {
        return Err(GeomError::NonFiniteArcAngle);
    }
    if n_components == 0 {
        return Err(GeomError::ZeroArcComponents);
    }
    let point_count = n_components
        .checked_mul(2)
        .and_then(|count| count.checked_add(1))
        .ok_or(GeomError::ArcComponentOverflow {
            count: n_components,
        })?;
    if n_components > MAX_ARC_COMPONENTS {
        return Err(GeomError::ArcComponentsAboveBudget {
            count: n_components,
            budget: MAX_ARC_COMPONENTS,
        });
    }
    Ok(point_count)
}

/// `quadratic_bezier_points_for_arc`: `2n+1` shared-anchor points tracing the
/// unit-circle arc from angle 0 to `angle`, handles pushed out by
/// `1/cos(θ/2)` so each quadratic component interpolates its arc chord.
///
/// # Errors
/// [`GeomError::NonFiniteArcAngle`] for NaN/infinite angles,
/// [`GeomError::ZeroArcComponents`] for a zero count,
/// [`GeomError::ArcComponentOverflow`] when `2·n + 1` is not representable,
/// [`GeomError::ArcComponentsAboveBudget`] above
/// [`MAX_ARC_COMPONENTS`].
pub fn quadratic_points_for_arc(angle: f64, n_components: usize) -> Result<Vec<Vec3>, GeomError> {
    let n_points = arc_point_count(angle, n_components)?;
    let angles = linspace(0.0, angle, n_points);
    let mut points: Vec<Vec3> = angles
        .iter()
        .map(|&a| [scalar::cos(a), scalar::sin(a), 0.0])
        .collect();
    let theta = angle / n_components as f64;
    let handle_scale = 1.0 / scalar::cos(theta / 2.0);
    for point in points.iter_mut().skip(1).step_by(2) {
        *point = vec::scale(*point, handle_scale);
    }
    Ok(points)
}

/// The one arc-density rule (Behavior Note BN-09): the number of quadratic
/// components used to trace an arc of the given subtended angle.
///
/// The Reference ships three inconsistent conventions
/// (`int(15·|θ|/TAU) + 1`, `ceil(8·|θ|/TAU)`, and a fixed 8); FrankenManim
/// uses `max(1, ceil(16·|θ|/TAU))` everywhere — 16 components for a full
/// circle, matching the Reference's `Arc`/`Circle` quality (its finest
/// convention) at every common angle and never coarser than it.
///
/// # Errors
/// [`GeomError::NonFiniteArcAngle`] for NaN/infinite angles;
/// [`GeomError::ArcComponentsAboveBudget`] when the rule's count exceeds
/// [`MAX_ARC_COMPONENTS`]; or [`GeomError::ArcComponentOverflow`] when an
/// extreme finite angle saturates the host component count.
pub fn arc_n_components(angle: f64) -> Result<usize, GeomError> {
    if !angle.is_finite() {
        return Err(GeomError::NonFiniteArcAngle);
    }
    let n = (16.0 * angle.abs() / TAU).ceil() as usize;
    let n = n.max(1);
    let _point_count = arc_point_count(angle, n)?;
    Ok(n)
}

/// A refusal from [`integer_interpolate`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IntegerInterpolationError {
    /// A half-open integer range must contain at least one sample.
    InvalidRange {
        /// Inclusive lower bound.
        start: i64,
        /// Exclusive upper bound.
        end: i64,
    },
    /// NaN and infinities have no ordered position to clamp.
    NonFiniteAlpha {
        /// The rejected interpolant.
        alpha: f64,
    },
}

impl core::fmt::Display for IntegerInterpolationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::InvalidRange { start, end } => write!(
                f,
                "integer interpolation needs a nonempty half-open range (got {start}..{end})"
            ),
            Self::NonFiniteAlpha { alpha } => {
                write!(
                    f,
                    "integer interpolation alpha must be finite (got {alpha})"
                )
            }
        }
    }
}

impl std::error::Error for IntegerInterpolationError {}

/// `integer_interpolate`: an integer in `[start, end)` plus the residue
/// toward the next one. Finite alpha outside `[0, 1]` keeps the
/// Reference's exact endpoint clamping; an empty/reversed range or a
/// non-finite alpha refuses before arithmetic.
///
/// The span is formed in `f64`, never by subtracting signed endpoints, so
/// the complete `i64::MIN..i64::MAX` domain cannot overflow before
/// conversion. The interior result is clamped back to the validated
/// half-open range to absorb endpoint rounding at spans wider than f64's
/// exact-integer range.
///
/// # Errors
/// [`IntegerInterpolationError::InvalidRange`] when `start >= end`, or
/// [`IntegerInterpolationError::NonFiniteAlpha`] for NaN/infinite alpha.
pub fn integer_interpolate(
    start: i64,
    end: i64,
    alpha: f64,
) -> Result<(i64, f64), IntegerInterpolationError> {
    if start >= end {
        return Err(IntegerInterpolationError::InvalidRange { start, end });
    }
    if !alpha.is_finite() {
        return Err(IntegerInterpolationError::NonFiniteAlpha { alpha });
    }
    if alpha >= 1.0 {
        return Ok((end - 1, 1.0));
    }
    if alpha <= 0.0 {
        return Ok((start, 0.0));
    }
    let span = end as f64 - start as f64;
    let offset = alpha * span;
    let value = (start as f64 + offset).floor() as i128;
    let value = value.clamp(i128::from(start), i128::from(end) - 1);
    #[allow(clippy::cast_possible_truncation)]
    let value = value as i64;
    Ok((value, offset.rem_euclid(1.0)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmn_core::constants::PI;

    #[test]
    fn partial_quadratic_endpoints_land_on_curve() {
        let pts = [[0.0, 0.0, 0.0], [1.0, 2.0, 0.0], [2.0, 0.0, 0.0]];
        let sub = partial_quadratic(&pts, 0.25, 0.75);
        let expect_start = quadratic_point(pts[0], pts[1], pts[2], 0.25);
        let expect_end = quadratic_point(pts[0], pts[1], pts[2], 0.75);
        for i in 0..3 {
            assert!((sub[0][i] - expect_start[i]).abs() < 1e-15);
            assert!((sub[2][i] - expect_end[i]).abs() < 1e-15);
        }
        // The restriction agrees with the original at its own midpoint.
        let mid_sub = quadratic_point(sub[0], sub[1], sub[2], 0.5);
        let mid_full = quadratic_point(pts[0], pts[1], pts[2], 0.5);
        for i in 0..3 {
            assert!((mid_sub[i] - mid_full[i]).abs() < 1e-12);
        }
    }

    #[test]
    fn partial_quadratic_a_equals_one() {
        let pts = [[0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [2.0, 0.0, 0.0]];
        assert_eq!(partial_quadratic(&pts, 1.0, 1.0), [pts[2], pts[2], pts[2]]);
    }

    #[test]
    fn arc_density_rule() {
        assert_eq!(arc_n_components(TAU), Ok(16));
        assert_eq!(arc_n_components(PI), Ok(8));
        assert_eq!(arc_n_components(TAU / 4.0), Ok(4));
        assert_eq!(arc_n_components(-TAU / 4.0), Ok(4));
        assert_eq!(arc_n_components(1e-9), Ok(1));
        assert_eq!(arc_n_components(0.0), Ok(1));
    }

    #[test]
    fn arc_component_contract_refusals_are_typed() {
        // fm-4tb.1: zero, non-finite, above-budget, and point-count
        // overflow are typed errors before any allocation.
        assert_eq!(
            quadratic_points_for_arc(1.0, 0),
            Err(GeomError::ZeroArcComponents)
        );
        assert_eq!(
            quadratic_points_for_arc(f64::NAN, 2),
            Err(GeomError::NonFiniteArcAngle)
        );
        assert_eq!(
            quadratic_points_for_arc(f64::INFINITY, 2),
            Err(GeomError::NonFiniteArcAngle)
        );
        assert_eq!(
            arc_n_components(f64::NEG_INFINITY),
            Err(GeomError::NonFiniteArcAngle)
        );
        // The budget boundary itself works; one past it refuses.
        let at = quadratic_points_for_arc(TAU / 4.0, MAX_ARC_COMPONENTS)
            .expect("budget boundary admits");
        assert_eq!(at.len(), 2 * MAX_ARC_COMPONENTS + 1);
        assert_eq!(
            quadratic_points_for_arc(TAU / 4.0, MAX_ARC_COMPONENTS + 1),
            Err(GeomError::ArcComponentsAboveBudget {
                count: MAX_ARC_COMPONENTS + 1,
                budget: MAX_ARC_COMPONENTS
            })
        );
        assert_eq!(
            quadratic_points_for_arc(TAU / 4.0, usize::MAX),
            Err(GeomError::ArcComponentOverflow { count: usize::MAX })
        );
        // The rule's own count for an extreme finite angle is a budget
        // refusal, never a wrap.
        assert_eq!(
            arc_n_components(1e18),
            Err(GeomError::ArcComponentsAboveBudget {
                count: (16.0_f64 * 1e18 / TAU).ceil() as usize,
                budget: MAX_ARC_COMPONENTS
            })
        );
    }

    #[test]
    fn arc_points_interpolate_unit_circle() {
        let pts = quadratic_points_for_arc(PI / 2.0, 2).expect("valid arc");
        assert_eq!(pts.len(), 5);
        // Anchors sit on the unit circle.
        for p in [pts[0], pts[2], pts[4]] {
            assert!((space_ops::get_norm(p) - 1.0).abs() < 1e-12);
        }
        assert!((pts[4][0]).abs() < 1e-12 && (pts[4][1] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn integer_interpolate_hardens_boundaries_and_keeps_reference_clamping() {
        let (value, residue) = integer_interpolate(0, 10, 0.46).expect("valid interpolation");
        assert_eq!(value, 4);
        assert!((residue - 0.6).abs() < 1e-12);
        assert_eq!(integer_interpolate(0, 10, 1.5), Ok((9, 1.0)));
        assert_eq!(integer_interpolate(0, 10, -0.5), Ok((0, 0.0)));
        assert_eq!(
            integer_interpolate(4, 4, 0.5),
            Err(IntegerInterpolationError::InvalidRange { start: 4, end: 4 })
        );
        assert_eq!(
            integer_interpolate(5, 4, 0.5),
            Err(IntegerInterpolationError::InvalidRange { start: 5, end: 4 })
        );
        assert!(matches!(
            integer_interpolate(0, 10, f64::NAN),
            Err(IntegerInterpolationError::NonFiniteAlpha { alpha }) if alpha.is_nan()
        ));
        for alpha in [f64::NEG_INFINITY, f64::INFINITY] {
            assert_eq!(
                integer_interpolate(0, 10, alpha),
                Err(IntegerInterpolationError::NonFiniteAlpha { alpha })
            );
        }
        assert_eq!(integer_interpolate(i64::MIN, i64::MAX, 0.5), Ok((0, 0.0)));
    }
}
