//! Object-to-world affine placement for Marionette entries (§10.8, fm-7if).
//!
//! The `RecordBuffer` owns object-space points. Positional operations compose
//! one of these maps instead of rewriting those points, so a translation,
//! rotation, scale, or stretch can invalidate screen-space derivations without
//! invalidating object-space curve coefficients and arc-length tables.

use fmn_core::types::Vec3;
use fmn_geom::Mat3;

/// A deterministic 3D affine object-to-world map.
///
/// The linear part is row-major and the translation is applied afterwards:
/// `world = linear * object + translation`. The type contains semantic state
/// only; [`crate::Entry`] owns the independent monotonic revision counter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    linear: Mat3,
    translation: Vec3,
}

impl Placement {
    /// Identity placement.
    pub const IDENTITY: Self = Self {
        linear: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        translation: [0.0; 3],
    };

    /// Build an affine map from its row-major linear part and translation.
    #[must_use]
    pub const fn new(linear: Mat3, translation: Vec3) -> Self {
        Self {
            linear,
            translation,
        }
    }

    /// A pure translation.
    #[must_use]
    pub const fn from_translation(translation: Vec3) -> Self {
        Self::new(Self::IDENTITY.linear, translation)
    }

    /// The row-major linear part.
    #[must_use]
    pub const fn linear(self) -> Mat3 {
        self.linear
    }

    /// The translation part.
    #[must_use]
    pub const fn translation(self) -> Vec3 {
        self.translation
    }

    /// Apply the affine map to one point.
    #[must_use]
    pub fn apply_point(self, point: Vec3) -> Vec3 {
        let mut out = self.apply_vector(point);
        for (slot, shift) in out.iter_mut().zip(self.translation) {
            *slot += shift;
        }
        out
    }

    /// Apply only the linear part to a vector.
    #[must_use]
    pub fn apply_vector(self, vector: Vec3) -> Vec3 {
        [
            self.linear[0][0] * vector[0]
                + self.linear[0][1] * vector[1]
                + self.linear[0][2] * vector[2],
            self.linear[1][0] * vector[0]
                + self.linear[1][1] * vector[1]
                + self.linear[1][2] * vector[2],
            self.linear[2][0] * vector[0]
                + self.linear[2][1] * vector[1]
                + self.linear[2][2] * vector[2],
        ]
    }

    /// Compose two placements as `self(other(point))`.
    #[must_use]
    pub fn compose(self, other: Self) -> Self {
        let mut linear = [[0.0; 3]; 3];
        for (row, out_row) in linear.iter_mut().enumerate() {
            for (column, slot) in out_row.iter_mut().enumerate() {
                *slot = self.linear[row][0] * other.linear[0][column]
                    + self.linear[row][1] * other.linear[1][column]
                    + self.linear[row][2] * other.linear[2][column];
            }
        }
        Self {
            linear,
            translation: self.apply_point(other.translation),
        }
    }

    /// Construct `linear * (point - pivot) + pivot`.
    #[must_use]
    pub fn about(linear: Mat3, pivot: Vec3) -> Self {
        let map = Self::new(linear, [0.0; 3]);
        let moved = map.apply_vector(pivot);
        Self::new(
            linear,
            [
                pivot[0] - moved[0],
                pivot[1] - moved[1],
                pivot[2] - moved[2],
            ],
        )
    }

    /// Whether this is bit-for-bit identity.
    #[must_use]
    pub fn is_identity(self) -> bool {
        self.same_bits(Self::IDENTITY)
    }

    /// Whether the linear part is bit-for-bit identity.
    #[must_use]
    pub fn is_translation(self) -> bool {
        self.linear
            .iter()
            .flatten()
            .zip(Self::IDENTITY.linear.iter().flatten())
            .all(|(a, b)| a.to_bits() == b.to_bits())
    }

    /// Coefficients in canonical row-major affine order.
    #[must_use]
    pub fn coefficients(self) -> [f64; 12] {
        [
            self.linear[0][0],
            self.linear[0][1],
            self.linear[0][2],
            self.translation[0],
            self.linear[1][0],
            self.linear[1][1],
            self.linear[1][2],
            self.translation[1],
            self.linear[2][0],
            self.linear[2][1],
            self.linear[2][2],
            self.translation[2],
        ]
    }

    /// Bitwise semantic equality, including signed zero and NaN payloads.
    #[must_use]
    pub fn same_bits(self, other: Self) -> bool {
        self.coefficients()
            .iter()
            .zip(other.coefficients())
            .all(|(a, b)| a.to_bits() == b.to_bits())
    }
}

impl Default for Placement {
    fn default() -> Self {
        Self::IDENTITY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composition_is_outer_after_inner() {
        let scale = Placement::new(
            [[2.0, 0.0, 0.0], [0.0, 3.0, 0.0], [0.0, 0.0, 4.0]],
            [0.0; 3],
        );
        let shift = Placement::from_translation([5.0, 7.0, 11.0]);
        let composed = shift.compose(scale);
        assert_eq!(composed.apply_point([1.0, 2.0, 3.0]), [7.0, 13.0, 23.0]);
    }

    #[test]
    fn pivoted_linear_map_keeps_the_pivot_fixed() {
        let map = Placement::about(
            [[0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
            [2.0, 3.0, 0.0],
        );
        assert_eq!(map.apply_point([2.0, 3.0, 0.0]), [2.0, 3.0, 0.0]);
        assert_eq!(map.apply_point([3.0, 3.0, 0.0]), [2.0, 4.0, 0.0]);
    }
}
