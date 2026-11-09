//! The array arithmetic under the geometry kernel.
//!
//! These are the operations the Reference gets from NumPy itself —
//! elementwise addition, scaling, interpolation, the row-vector/matrix
//! product, `np.isclose` — as opposed to the ones it defines in
//! `manimlib/utils/space_ops.py`, which live in [`crate::space_ops`] and
//! are public API (fm-ngx). Nothing here but [`Mat3`] is public: it is
//! plumbing, and every caller is inside this crate.

use fmn_core::types::Vec3;

/// Row-major 3×3 matrix: `m[row][col]`.
pub type Mat3 = [[f64; 3]; 3];

pub(crate) const IDENTITY: Mat3 = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

#[inline]
pub(crate) fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
pub(crate) fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
pub(crate) fn scale(a: Vec3, s: f64) -> Vec3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

#[inline]
pub(crate) fn lerp(a: Vec3, b: Vec3, t: f64) -> Vec3 {
    [
        (1.0 - t) * a[0] + t * b[0],
        (1.0 - t) * a[1] + t * b[1],
        (1.0 - t) * a[2] + t * b[2],
    ]
}

/// `p @ m` in numpy terms: treat `p` as a row vector.
#[inline]
pub(crate) fn mul_point_mat(p: Vec3, m: &Mat3) -> Vec3 {
    [
        p[0] * m[0][0] + p[1] * m[1][0] + p[2] * m[2][0],
        p[0] * m[0][1] + p[1] * m[1][1] + p[2] * m[2][1],
        p[0] * m[0][2] + p[1] * m[1][2] + p[2] * m[2][2],
    ]
}

#[inline]
pub(crate) fn transpose(m: &Mat3) -> Mat3 {
    [
        [m[0][0], m[1][0], m[2][0]],
        [m[0][1], m[1][1], m[2][1]],
        [m[0][2], m[1][2], m[2][2]],
    ]
}

/// `np.isclose` with numpy's default tolerances (`rtol=1e-5`, `atol=1e-8`),
/// applied per component — the closure test several Reference formulas
/// depend on.
pub(crate) fn np_isclose_all(a: Vec3, b: Vec3) -> bool {
    a.iter()
        .zip(b.iter())
        .all(|(x, y)| (x - y).abs() <= 1e-8 + 1e-5 * y.abs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mul_point_mat_is_the_row_vector_product() {
        // Row-vector convention: p @ m sums over the matrix's ROWS.
        let m: Mat3 = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];
        assert_eq!(mul_point_mat([1.0, 0.0, 0.0], &m), [1.0, 2.0, 3.0]);
        assert_eq!(mul_point_mat([0.0, 1.0, 0.0], &m), [4.0, 5.0, 6.0]);
        assert_eq!(mul_point_mat([1.0, 1.0, 1.0], &m), [12.0, 15.0, 18.0]);
    }

    #[test]
    fn transpose_is_an_involution() {
        let m: Mat3 = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];
        assert_eq!(transpose(&transpose(&m)), m);
        assert_eq!(transpose(&IDENTITY), IDENTITY);
    }

    #[test]
    fn lerp_hits_both_ends() {
        let (a, b) = ([0.0, 1.0, 2.0], [4.0, 5.0, 6.0]);
        assert_eq!(lerp(a, b, 0.0), a);
        assert_eq!(lerp(a, b, 1.0), b);
        assert_eq!(lerp(a, b, 0.5), [2.0, 3.0, 4.0]);
    }

    #[test]
    fn np_isclose_uses_numpy_tolerances() {
        assert!(np_isclose_all([1.0, 0.0, 0.0], [1.0 + 1e-9, 0.0, 0.0]));
        assert!(!np_isclose_all([1.0, 0.0, 0.0], [1.0 + 1e-4, 0.0, 0.0]));
        // Relative term: large magnitudes get proportionally more slack.
        assert!(np_isclose_all([1e6, 0.0, 0.0], [1e6 + 1.0, 0.0, 0.0]));
    }
}
