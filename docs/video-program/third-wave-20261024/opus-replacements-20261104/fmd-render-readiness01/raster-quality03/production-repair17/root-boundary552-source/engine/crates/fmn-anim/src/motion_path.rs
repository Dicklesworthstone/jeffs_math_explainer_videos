//! The shared point-path law and its translation-independent affine lift.
use fmn_core::types::Vec3;
use fmn_mobject::Placement;

use crate::animation::AnimError;

/// Arc angles below this threshold use the straight path (the Reference rule).
pub const STRAIGHT_PATH_THRESHOLD: f64 = 0.01;

fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn unit_axis(axis: Vec3) -> Vec3 {
    let scale = axis
        .iter()
        .fold(0.0_f64, |largest, value| largest.max(value.abs()));
    if scale == 0.0 {
        return [0.0, 0.0, 1.0];
    }
    // Scale before squaring: every finite nonzero axis has the same direction
    // at 1e-300 and 1e300, without underflow to the OUT fallback or overflow.
    let scaled = axis.map(|value| value / scale);
    let norm = dot(scaled, scaled).sqrt();
    scaled.map(|value| value / norm)
}

fn nan_to_num(value: f64) -> f64 {
    if value.is_nan() {
        0.0
    } else if value == f64::INFINITY {
        f64::MAX
    } else if value == f64::NEG_INFINITY {
        f64::MIN
    } else {
        value
    }
}

/// A point path as composable, journal-able data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathFunc {
    /// `(1-alpha) * start + alpha * end`.
    Straight,
    /// A circular path perpendicular to `axis`, with linear axial motion.
    /// Unequal axial coordinates therefore trace a helix, not a broken arc.
    Arc {
        /// Signed arc angle in radians.
        angle: f64,
        /// Finite rotation axis, normalized at eval; zero means OUT.
        axis: Vec3,
    },
}

impl PathFunc {
    /// Preserve the Reference's small-angle straight-path threshold.
    #[must_use]
    pub fn from_path_arc(angle: f64, axis: Vec3) -> Self {
        if angle.abs() < STRAIGHT_PATH_THRESHOLD {
            Self::Straight
        } else {
            Self::Arc { angle, axis }
        }
    }

    /// Reject non-finite arc parameters before an animation mutates its Stage.
    ///
    /// # Errors
    /// [`AnimError::InvalidPath`] for a non-finite angle or axis component.
    pub fn validate(&self) -> Result<(), AnimError> {
        if let Self::Arc { angle, axis } = self {
            if !angle.is_finite() {
                return Err(AnimError::InvalidPath("arc angle must be finite"));
            }
            if axis.iter().any(|value| !value.is_finite()) {
                return Err(AnimError::InvalidPath("arc axis must be finite"));
            }
        }
        Ok(())
    }

    /// Evaluate one path. Arc endpoints are exact; its axial component lerps
    /// independently of its perpendicular rotation (BN-12).
    #[must_use]
    pub fn eval(&self, start: Vec3, end: Vec3, alpha: f64) -> Vec3 {
        match *self {
            Self::Straight => std::array::from_fn(|i| (1.0 - alpha) * start[i] + alpha * end[i]),
            Self::Arc { angle, axis } => {
                if angle.abs() < STRAIGHT_PATH_THRESHOLD {
                    return Self::Straight.eval(start, end, alpha);
                }
                if alpha == 0.0 || start == end {
                    return start;
                }
                if alpha == 1.0 {
                    return end;
                }
                let unit = unit_axis(axis);
                let half: Vec3 = std::array::from_fn(|i| (end[i] - start[i]) / 2.0);
                let tan_half = fmn_dmath::tan(angle / 2.0);
                let c = cross(unit, half);
                let center: Vec3 =
                    std::array::from_fn(|i| start[i] + half[i] + nan_to_num(c[i] / tan_half));
                let radius: Vec3 = std::array::from_fn(|i| start[i] - center[i]);
                let perpendicular = cross(unit, radius);
                let sin_a = fmn_dmath::sin(alpha * angle);
                let cos_a = fmn_dmath::cos(alpha * angle);
                // Complete Rodrigues rotation, then translate along the axis.
                // The old two-term rotation dropped this projection, so a 3D
                // arc could fail to reach its requested endpoint.
                let axial = (1.0 - cos_a) * dot(unit, radius) + 2.0 * alpha * dot(unit, half);
                std::array::from_fn(|i| {
                    center[i] + cos_a * radius[i] + sin_a * perpendicular[i] + unit[i] * axial
                })
            }
        }
    }
}

/// Lift the point-path law to an affine map without subtracting two large
/// translated points. For fixed path/alpha the law is linear in its endpoint
/// vectors, so each matrix column can be evaluated independently of origin.
pub(super) fn interpolate_placement(
    from: Placement,
    to: Placement,
    alpha: f64,
    path: PathFunc,
) -> Placement {
    let origin = path.eval(from.translation(), to.translation(), alpha);
    let a = from.linear();
    let b = to.linear();
    let mut linear = [[0.0; 3]; 3];
    for column in 0..3 {
        let value = path.eval(
            [a[0][column], a[1][column], a[2][column]],
            [b[0][column], b[1][column], b[2][column]],
            alpha,
        );
        for row in 0..3 {
            linear[row][column] = value[row];
        }
    }
    Placement::new(linear, origin)
}
