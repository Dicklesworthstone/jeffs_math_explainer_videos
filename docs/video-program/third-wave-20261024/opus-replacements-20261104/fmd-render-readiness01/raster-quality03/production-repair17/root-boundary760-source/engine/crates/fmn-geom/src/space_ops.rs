//! `space_ops`, signature for signature (§7.5).
//!
//! A port of `manimlib/utils/space_ops.py` (`3b1b/manim` @ `6199a00d`) —
//! the vector, rotation, and planar-geometry vocabulary every mobject, the
//! camera, and user scene code all speak. The port is faithful including
//! the degenerate cases, because scene code depends on what the Reference
//! *does* there (a near-parallel `find_intersection` returning its first
//! point, `normalize` of a zero vector returning zeros, `get_unit_normal`'s
//! `DOWN` fallback), not on what a textbook would prescribe.
//!
//! Three classes of deliberate difference, all typed rather than silent:
//!
//! * **Refusals are values.** Where the Reference raises (`line_intersection`
//!   on parallel lines) or lets NumPy produce a NaN (`center_of_mass` of
//!   nothing), we return `Option`/a defined value. No NaN escapes this
//!   module.
//! * **Rotation conventions are pinned.** Everything rotational routes
//!   through [`crate::rotation`], which reimplements scipy `Rotation`'s
//!   conventions exactly (§7.5, §2.2) — see that module.
//! * **NumPy-array signatures become slice signatures.** The Reference's
//!   vectorized overloads (`cross` over an `(n, 3)` array,
//!   `normalize_along_axis`) are documented at their Rust spellings below;
//!   the batching itself belongs to the caller.
//!
//! Numerics: f64 semantic math (§6.1) with every transcendental routed
//! through `crate::scalar` onto fmn-dmath, so certified renders are
//! bit-stable across the platform matrix. `sqrt` is used directly — IEEE
//! 754 requires correct rounding, so it is already reproducible.
//!
//! The one member of the Reference's module deliberately absent is
//! `earclip_triangulation`, which is triangulation, not space ops, and
//! lands with the ear-clipper (fm-81u).

use fmn_core::constants::{DOWN, OUT, RIGHT, TAU, UP};
use fmn_core::types::Vec3;

use crate::rotation::{self, Quat};
use crate::scalar;
use crate::vec::{self, IDENTITY, Mat3};

/// A complex number as `[re, im]` — the Reference passes Python `complex`
/// values through [`complex_to_r3`]/[`r3_to_complex`].
pub type Complex = [f64; 2];

/// `find_intersection`'s default near-parallel cutoff.
pub const DEFAULT_INTERSECTION_THRESHOLD: f64 = 1e-5;

/// `get_unit_normal`'s default degeneracy tolerance.
pub const DEFAULT_UNIT_NORMAL_TOL: f64 = 1e-6;

/// `rotation_between_vectors`' alignment tolerance.
const ROTATION_BETWEEN_ATOL: f64 = 1e-8;

/// Maximum number of vectors one [`compass_directions`] call may publish.
///
/// The cap matches the other geometry-family multiplicity limits: it is
/// ample for authored scenes while preventing one scalar argument from
/// requesting effectively unbounded rotation work and allocation.
pub const MAX_COMPASS_DIRECTIONS: usize = 4_096;

/// Maximum admitted dimension of a square [`thick_diagonal`] mask.
///
/// A 64-by-64 mask contains [`MAX_THICK_DIAGONAL_CELLS`] bytes. The helper
/// checks the square with checked arithmetic before applying this limit.
pub const MAX_THICK_DIAGONAL_DIMENSION: usize = 64;

/// Maximum number of cells one [`thick_diagonal`] call may publish.
pub const MAX_THICK_DIAGONAL_CELLS: usize =
    MAX_THICK_DIAGONAL_DIMENSION * MAX_THICK_DIAGONAL_DIMENSION;

/// A public `space_ops` construction refused before proportional work begins.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpaceOpsError {
    /// The requested compass family exceeds [`MAX_COMPASS_DIRECTIONS`].
    TooManyDirections {
        /// Requested number of output vectors.
        requested: usize,
        /// Declared maximum.
        max: usize,
    },
    /// Squaring a thick-diagonal dimension cannot be represented by the host.
    DiagonalSizeOverflow {
        /// Requested square dimension.
        dim: usize,
    },
    /// The requested thick-diagonal square exceeds [`MAX_THICK_DIAGONAL_CELLS`].
    TooManyDiagonalCells {
        /// Requested number of output cells.
        requested: usize,
        /// Declared maximum.
        max: usize,
    },
}

impl std::fmt::Display for SpaceOpsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooManyDirections { requested, max } => write!(
                f,
                "requested {requested} compass directions, above the {max} cap"
            ),
            Self::DiagonalSizeOverflow { dim } => {
                write!(f, "a {dim}-by-{dim} thick diagonal exceeds host capacity")
            }
            Self::TooManyDiagonalCells { requested, max } => write!(
                f,
                "requested {requested} thick-diagonal cells, above the {max} cap"
            ),
        }
    }
}

impl std::error::Error for SpaceOpsError {}

// ------------------------------------------------------------ vector basics

/// `space_ops.cross`.
#[must_use]
pub fn cross(v1: Vec3, v2: Vec3) -> Vec3 {
    [
        v1[1] * v2[2] - v1[2] * v2[1],
        v1[2] * v2[0] - v1[0] * v2[2],
        v1[0] * v2[1] - v1[1] * v2[0],
    ]
}

/// `space_ops.cross2d`: the z component of the cross product of the xy
/// projections.
#[must_use]
pub fn cross2d(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[1] - b[0] * a[1]
}

/// The Euclidean inner product. (NumPy's `dot` in the Reference; named
/// here because the call sites are ours.)
#[must_use]
pub fn dot(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// `space_ops.get_norm`. The summation order (x, then y, then z) is fixed,
/// because a reassociation would change the last bit.
#[must_use]
pub fn get_norm(v: Vec3) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// `space_ops.norm_squared`.
#[must_use]
pub fn norm_squared(v: Vec3) -> f64 {
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
}

/// `space_ops.get_dist`.
#[must_use]
pub fn get_dist(v1: Vec3, v2: Vec3) -> f64 {
    get_norm(vec::sub(v2, v1))
}

/// `space_ops.normalize` with no fallback: a zero vector normalizes to the
/// zero vector (the Reference's `np.zeros(len(vect))`), never to a NaN.
#[must_use]
pub fn normalize(v: Vec3) -> Vec3 {
    let n = get_norm(v);
    if n > 0.0 {
        vec::scale(v, 1.0 / n)
    } else {
        [0.0; 3]
    }
}

/// `space_ops.normalize(vect, fall_back)`: a zero vector yields `fall_back`
/// verbatim (it is *not* normalized first — the Reference returns it as
/// given).
#[must_use]
pub fn normalize_or(v: Vec3, fall_back: Vec3) -> Vec3 {
    let n = get_norm(v);
    if n > 0.0 {
        vec::scale(v, 1.0 / n)
    } else {
        fall_back
    }
}

/// `space_ops.normalize_along_axis(array, 1)`: normalize each row in place.
///
/// The Reference's axis-0 form has no consumer at the pin; the axis-1 form
/// is what `Surface` uses for its normals (`types/surface.py:102, :173`).
/// Zero-norm rows are left as zeros (the Reference's `norms[norms == 0] = 1`
/// divides them by one).
pub fn normalize_rows(rows: &mut [Vec3]) {
    for row in rows.iter_mut() {
        let n = get_norm(*row);
        if n != 0.0 {
            *row = vec::scale(*row, 1.0 / n);
        }
    }
}

/// The Reference's 2D→3D point promotion: a coordinate list shorter than
/// three is zero-padded, longer is truncated.
///
/// This is the rule `Line.pointify` applies to every user-supplied point
/// (`geometry.py:747-749` — `result = np.zeros(self.dim);
/// result[:len(point)] = point`), and the reason `Polygon(( -3, 0 ), …)`
/// works at all. It is stated once here so every constructor that accepts
/// loose coordinates agrees.
#[must_use]
pub fn promote_point(coords: &[f64]) -> Vec3 {
    let mut out = [0.0; 3];
    for (slot, value) in out.iter_mut().zip(coords) {
        *slot = *value;
    }
    out
}

/// `space_ops.poly_line_length`: the summed length of the segments between
/// adjacent points (not closed).
#[must_use]
pub fn poly_line_length(points: &[Vec3]) -> f64 {
    points
        .windows(2)
        .map(|w| get_norm(vec::sub(w[1], w[0])))
        .sum()
}

/// `space_ops.center_of_mass`.
///
/// The empty input is a defined zero rather than NumPy's NaN — this
/// function feeds bounding boxes and label placement, where one NaN
/// silently poisons a whole family.
#[must_use]
pub fn center_of_mass(points: &[Vec3]) -> Vec3 {
    if points.is_empty() {
        return [0.0; 3];
    }
    let mut sum = [0.0; 3];
    for p in points {
        sum = vec::add(sum, *p);
    }
    vec::scale(sum, 1.0 / points.len() as f64)
}

/// `space_ops.midpoint`.
#[must_use]
pub fn midpoint(p1: Vec3, p2: Vec3) -> Vec3 {
    center_of_mass(&[p1, p2])
}

// ---------------------------------------------------------------- rotations

/// `space_ops.quaternion_mult` over a slice, including its empty case (the
/// identity quaternion) — see [`crate::rotation::compose_quat`] for the
/// convention.
#[must_use]
pub fn quaternion_mult(quats: &[Quat]) -> Quat {
    let mut result = match quats.first() {
        Some(q) => *q,
        None => return rotation::IDENTITY_QUAT,
    };
    for next in &quats[1..] {
        result = rotation::compose_quat(result, *next);
    }
    result
}

/// `space_ops.quaternion_conjugate`.
#[must_use]
pub fn quaternion_conjugate(q: Quat) -> Quat {
    [-q[0], -q[1], -q[2], q[3]]
}

/// `space_ops.quaternion_from_angle_axis`.
#[must_use]
pub fn quaternion_from_angle_axis(angle: f64, axis: Vec3) -> Quat {
    rotation::quat_from_rotvec(vec::scale(normalize(axis), angle))
}

/// `space_ops.angle_axis_from_quaternion`.
///
/// The Reference divides the rotation vector by its own norm, so the
/// identity quaternion yields NaNs; here that is `None`. Otherwise the
/// angle is in `[0, π]` and the axis is a unit vector.
#[must_use]
pub fn angle_axis_from_quaternion(q: Quat) -> Option<(f64, Vec3)> {
    let rotvec = rotation::rotvec_from_quat(q)?;
    let norm = get_norm(rotvec);
    if norm == 0.0 {
        return None;
    }
    Some((norm, vec::scale(rotvec, 1.0 / norm)))
}

/// `space_ops.rotation_matrix_transpose_from_quaternion` — scipy's
/// `as_matrix`, i.e. the matrix that maps a column vector.
#[must_use]
pub fn rotation_matrix_transpose_from_quaternion(q: Quat) -> Option<Mat3> {
    rotation::matrix_from_quat(q)
}

/// `space_ops.rotation_matrix_from_quaternion`: the above, transposed.
#[must_use]
pub fn rotation_matrix_from_quaternion(q: Quat) -> Option<Mat3> {
    rotation::matrix_from_quat(q).map(|m| vec::transpose(&m))
}

/// `space_ops.rotation_matrix`: rotation in R³ about `axis` by `angle`.
///
/// Built through the quaternion exactly as the Reference does
/// (`Rotation.from_rotvec(angle * normalize(axis)).as_matrix()`), so the
/// small-angle series and the element ordering match scipy bit for bit.
/// A zero axis gives the identity (its `normalize` is the zero vector).
#[must_use]
pub fn rotation_matrix(angle: f64, axis: Vec3) -> Mat3 {
    rotation::matrix_from_unit_quat(quaternion_from_angle_axis(angle, axis))
}

/// `space_ops.rotation_matrix_transpose`.
#[must_use]
pub fn rotation_matrix_transpose(angle: f64, axis: Vec3) -> Mat3 {
    vec::transpose(&rotation_matrix(angle, axis))
}

/// `space_ops.rotation_about_z` — the closed form, not the quaternion
/// path, matching the Reference's own special case.
#[must_use]
pub fn rotation_about_z(angle: f64) -> Mat3 {
    let c = scalar::cos(angle);
    let s = scalar::sin(angle);
    [[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]]
}

/// `space_ops.rotation_between_vectors`, including its degenerate-axis
/// fallback chain: the cross product first, then `v1 × RIGHT`, then
/// `v1 × UP`.
#[must_use]
pub fn rotation_between_vectors(v1: Vec3, v2: Vec3) -> Mat3 {
    if get_norm(vec::sub(v1, v2)) < ROTATION_BETWEEN_ATOL {
        return IDENTITY;
    }
    let mut axis = cross(v1, v2);
    if get_norm(axis) < ROTATION_BETWEEN_ATOL {
        axis = cross(v1, RIGHT);
    }
    if get_norm(axis) < ROTATION_BETWEEN_ATOL {
        axis = cross(v1, UP);
    }
    rotation_matrix(angle_between_vectors(v1, v2), axis)
}

/// `space_ops.z_to_vector`.
#[must_use]
pub fn z_to_vector(vector: Vec3) -> Mat3 {
    rotation_between_vectors(OUT, vector)
}

/// `space_ops.rotate_vector`.
#[must_use]
pub fn rotate_vector(vector: Vec3, angle: f64, axis: Vec3) -> Vec3 {
    // `np.dot(vector, rot.as_matrix().T)` — the matrix applied to a column.
    let m = rotation_matrix(angle, axis);
    vec::mul_point_mat(vector, &vec::transpose(&m))
}

/// `space_ops.rotate_vector` with the Reference's default axis, `OUT`.
#[must_use]
pub fn rotate_vector_about_z(vector: Vec3, angle: f64) -> Vec3 {
    rotate_vector(vector, angle, OUT)
}

/// `space_ops.rotate_vector_2d`: the Reference multiplies by
/// `exp(i·angle)`, which is the plane rotation.
#[must_use]
pub fn rotate_vector_2d(vector: Complex, angle: f64) -> Complex {
    let (c, s) = (scalar::cos(angle), scalar::sin(angle));
    [vector[0] * c - vector[1] * s, vector[0] * s + vector[1] * c]
}

/// `space_ops.angle_of_vector`: the polar angle of the xy projection.
#[must_use]
pub fn angle_of_vector(v: Vec3) -> f64 {
    scalar::atan2(v[1], v[0])
}

/// `space_ops.angle_between_vectors`: always in `[0, π]`, and zero when
/// either vector is zero.
#[must_use]
pub fn angle_between_vectors(v1: Vec3, v2: Vec3) -> f64 {
    let n1 = get_norm(v1);
    let n2 = get_norm(v2);
    if n1 == 0.0 || n2 == 0.0 {
        return 0.0;
    }
    scalar::acos((dot(v1, v2) / (n1 * n2)).clamp(-1.0, 1.0))
}

/// `space_ops.get_unit_normal`, with the Reference's fallbacks: when the
/// two vectors align, a normal in the plane they share with the z axis;
/// when that degenerates too, `DOWN`.
#[must_use]
pub fn get_unit_normal(v1: Vec3, v2: Vec3, tol: f64) -> Vec3 {
    let u1 = normalize(v1);
    let u2 = normalize(v2);
    let cp = cross(u1, u2);
    let cp_norm = get_norm(cp);
    if cp_norm < tol {
        let new_cp = cross(cross(u1, OUT), u1);
        let new_cp_norm = get_norm(new_cp);
        if new_cp_norm < tol {
            return DOWN;
        }
        return vec::scale(new_cp, 1.0 / new_cp_norm);
    }
    vec::scale(cp, 1.0 / cp_norm)
}

/// `space_ops.project_along_vector`: the component of `point` orthogonal to
/// `vector` (which the Reference expects to be a unit vector).
#[must_use]
pub fn project_along_vector(point: Vec3, vector: Vec3) -> Vec3 {
    vec::sub(point, vec::scale(vector, dot(vector, point)))
}

// -------------------------------------------------------- planar geometry

/// `space_ops.find_intersection` (the single-point form): the intersection
/// of the line through `p0` along `v0` with the line through `p1` along
/// `v1`; in 3D, the point on the first ray closest to the second.
///
/// A configuration whose denominator falls under `threshold` returns `p0`,
/// exactly as the Reference's `denom → ∞` masking does — a defined answer,
/// never a NaN.
#[must_use]
pub fn find_intersection(p0: Vec3, v0: Vec3, p1: Vec3, v1: Vec3, threshold: f64) -> Vec3 {
    let is_3d = p0[2] != 0.0 || v0[2] != 0.0 || p1[2] != 0.0 || v1[2] != 0.0;
    let (numer, denom) = if is_3d {
        let cp1 = cross(v1, vec::sub(p1, p0));
        let cp2 = cross(v1, v0);
        (dot(cp1, cp1), dot(cp1, cp2))
    } else {
        (cross2d(v1, vec::sub(p1, p0)), cross2d(v1, v0))
    };
    let ratio = if denom.abs() < threshold {
        0.0
    } else {
        numer / denom
    };
    vec::add(p0, vec::scale(v0, ratio))
}

/// `space_ops.line_intersection`: the intersection of two lines, each given
/// by two points on it, in the xy plane.
///
/// The Reference raises `"Lines do not intersect"` on a zero determinant;
/// here that is `None`.
#[must_use]
pub fn line_intersection(line1: (Vec3, Vec3), line2: (Vec3, Vec3)) -> Option<Vec3> {
    let x_diff = (line1.0[0] - line1.1[0], line2.0[0] - line2.1[0]);
    let y_diff = (line1.0[1] - line1.1[1], line2.0[1] - line2.1[1]);
    let det = |a: (f64, f64), b: (f64, f64)| a.0 * b.1 - a.1 * b.0;
    let div = det(x_diff, y_diff);
    if div == 0.0 {
        return None;
    }
    let d = (
        det((line1.0[0], line1.0[1]), (line1.1[0], line1.1[1])),
        det((line2.0[0], line2.0[1]), (line2.1[0], line2.1[1])),
    );
    Some([det(d, x_diff) / div, det(d, y_diff) / div, 0.0])
}

/// `space_ops.line_intersects_path`: whether the segment `start`–`end`
/// crosses the polyline `path` (xy only, strict crossings — a touch is not
/// a crossing, matching the Reference's `< 0` tests).
#[must_use]
pub fn line_intersects_path(start: Vec3, end: Vec3, path: &[Vec3]) -> bool {
    if path.len() < 2 {
        return false;
    }
    let v1 = vec::sub(end, start);
    path.windows(2).any(|seg| {
        let (p2, q2) = (seg[0], seg[1]);
        let v2 = vec::sub(q2, p2);
        let mis1 = cross2d(v1, vec::sub(p2, start)) * cross2d(v1, vec::sub(q2, start)) < 0.0;
        let mis2 = cross2d(v2, vec::sub(start, p2)) * cross2d(v2, vec::sub(end, p2)) < 0.0;
        mis1 && mis2
    })
}

/// `space_ops.get_closest_point_on_line`: the point of segment `a`–`b`
/// nearest `p`, clamped to the segment.
///
/// A degenerate segment (`a == b`) returns `a` rather than the Reference's
/// `0/0`.
#[must_use]
pub fn get_closest_point_on_line(a: Vec3, b: Vec3, p: Vec3) -> Vec3 {
    let ab = vec::sub(a, b);
    let denom = dot(ab, ab);
    if denom == 0.0 {
        return a;
    }
    let t = (dot(vec::sub(p, b), ab) / denom).clamp(0.0, 1.0);
    vec::add(vec::scale(a, t), vec::scale(b, 1.0 - t))
}

/// `space_ops.get_winding_number`: the number of times the closed polyline
/// through `points` winds about the origin. The path is treated as cyclic
/// (the Reference's `adjacent_pairs` wraps).
#[must_use]
pub fn get_winding_number(points: &[Vec3]) -> f64 {
    use fmn_core::constants::PI;
    if points.is_empty() {
        return 0.0;
    }
    let mut total = 0.0;
    for (i, p1) in points.iter().enumerate() {
        let p2 = points[(i + 1) % points.len()];
        let d_angle = angle_of_vector(p2) - angle_of_vector(*p1);
        total += (d_angle + PI).rem_euclid(TAU) - PI;
    }
    total / TAU
}

/// `space_ops.tri_area`: the area of the triangle `abc`, xy only.
#[must_use]
pub fn tri_area(a: Vec3, b: Vec3, c: Vec3) -> f64 {
    0.5 * (a[0] * (b[1] - c[1]) + b[0] * (c[1] - a[1]) + c[0] * (a[1] - b[1])).abs()
}

/// `space_ops.is_inside_triangle`: strictly inside, either winding, xy
/// only. A point on an edge is outside (all three cross products must
/// share a sign).
#[must_use]
pub fn is_inside_triangle(p: Vec3, a: Vec3, b: Vec3, c: Vec3) -> bool {
    let crosses = [
        cross2d(vec::sub(p, a), vec::sub(b, p)),
        cross2d(vec::sub(p, b), vec::sub(c, p)),
        cross2d(vec::sub(p, c), vec::sub(a, p)),
    ];
    crosses.iter().all(|x| *x > 0.0) || crosses.iter().all(|x| *x < 0.0)
}

/// `space_ops.compass_directions`: `n` copies of `start_vect`, rotated by
/// successive `TAU/n` steps about the z axis.
///
/// # Errors
///
/// Returns [`SpaceOpsError::TooManyDirections`] before iteration or
/// allocation when `n` exceeds [`MAX_COMPASS_DIRECTIONS`].
pub fn compass_directions(n: usize, start_vect: Vec3) -> Result<Vec<Vec3>, SpaceOpsError> {
    if n > MAX_COMPASS_DIRECTIONS {
        return Err(SpaceOpsError::TooManyDirections {
            requested: n,
            max: MAX_COMPASS_DIRECTIONS,
        });
    }
    if n == 0 {
        return Ok(Vec::new());
    }
    let angle = TAU / n as f64;
    Ok((0..n)
        .map(|k| rotate_vector(start_vect, k as f64 * angle, OUT))
        .collect())
}

/// `space_ops.thick_diagonal`: a `dim × dim` 0/1 mask, set within
/// `thickness` of the diagonal.
///
/// # Errors
///
/// Returns [`SpaceOpsError::DiagonalSizeOverflow`] when `dim²` cannot be
/// represented, or [`SpaceOpsError::TooManyDiagonalCells`] when the square
/// exceeds [`MAX_THICK_DIAGONAL_CELLS`]. Both checks happen before iteration
/// or allocation.
pub fn thick_diagonal(dim: usize, thickness: usize) -> Result<Vec<Vec<u8>>, SpaceOpsError> {
    let cells = dim
        .checked_mul(dim)
        .ok_or(SpaceOpsError::DiagonalSizeOverflow { dim })?;
    if cells > MAX_THICK_DIAGONAL_CELLS {
        return Err(SpaceOpsError::TooManyDiagonalCells {
            requested: cells,
            max: MAX_THICK_DIAGONAL_CELLS,
        });
    }
    Ok((0..dim)
        .map(|r| {
            (0..dim)
                .map(|c| u8::from(r.abs_diff(c) < thickness))
                .collect()
        })
        .collect())
}

// --------------------------------------------------------- complex helpers

/// `space_ops.complex_to_R3`.
#[must_use]
pub fn complex_to_r3(z: Complex) -> Vec3 {
    [z[0], z[1], 0.0]
}

/// `space_ops.R3_to_complex` — the z coordinate is dropped.
#[must_use]
pub fn r3_to_complex(point: Vec3) -> Complex {
    [point[0], point[1]]
}

/// `space_ops.complex_func_to_R3_func`.
pub fn complex_func_to_r3_func(f: impl Fn(Complex) -> Complex) -> impl Fn(Vec3) -> Vec3 {
    move |p| complex_to_r3(f(r3_to_complex(p)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmn_core::constants::PI;

    fn close(a: Vec3, b: Vec3, tol: f64) {
        for i in 0..3 {
            assert!((a[i] - b[i]).abs() < tol, "{a:?} vs {b:?}");
        }
    }

    #[test]
    fn normalize_defines_the_zero_vector() {
        assert_eq!(normalize([0.0; 3]), [0.0; 3]);
        assert_eq!(normalize_or([0.0; 3], OUT), OUT);
        // A fallback is returned verbatim, not normalized.
        assert_eq!(normalize_or([0.0; 3], [0.0, 0.0, 5.0]), [0.0, 0.0, 5.0]);
        close(normalize([3.0, 4.0, 0.0]), [0.6, 0.8, 0.0], 1e-15);
    }

    #[test]
    fn normalize_rows_leaves_zero_rows_alone() {
        let mut rows = [[3.0, 4.0, 0.0], [0.0; 3], [0.0, 0.0, -2.0]];
        normalize_rows(&mut rows);
        close(rows[0], [0.6, 0.8, 0.0], 1e-15);
        assert_eq!(rows[1], [0.0; 3]);
        close(rows[2], [0.0, 0.0, -1.0], 1e-15);
    }

    #[test]
    fn rotation_matrix_agrees_with_the_z_closed_form() {
        for angle in [0.0, 0.3, PI / 2.0, PI, -1.7, 5.0 * TAU + 0.2] {
            let m = rotation_matrix(angle, OUT);
            let z = rotation_about_z(angle);
            for r in 0..3 {
                close(m[r], z[r], 1e-14);
            }
        }
    }

    #[test]
    fn rotation_matrix_of_a_zero_axis_is_the_identity() {
        assert_eq!(rotation_matrix(1.2, [0.0; 3]), IDENTITY);
    }

    #[test]
    fn rotate_vector_is_the_rotation_it_says_it_is() {
        close(rotate_vector(RIGHT, PI / 2.0, OUT), UP, 1e-15);
        close(rotate_vector(UP, PI / 2.0, OUT), [-1.0, 0.0, 0.0], 1e-15);
        close(rotate_vector(OUT, PI / 2.0, RIGHT), [0.0, -1.0, 0.0], 1e-15);
        // Composition: two half-turns about z are the identity.
        let v = [1.0, 2.0, 3.0];
        close(rotate_vector(rotate_vector(v, PI, OUT), PI, OUT), v, 1e-14);
    }

    #[test]
    fn rotate_vector_2d_matches_the_3d_form() {
        let v = [1.0, -2.0, 0.0];
        for angle in [0.0, 0.7, -2.4] {
            let a = rotate_vector_2d([v[0], v[1]], angle);
            let b = rotate_vector(v, angle, OUT);
            assert!((a[0] - b[0]).abs() < 1e-14 && (a[1] - b[1]).abs() < 1e-14);
        }
    }

    #[test]
    fn rotation_between_vectors_walks_its_fallback_chain() {
        // Identical vectors: early return.
        assert_eq!(rotation_between_vectors(OUT, OUT), IDENTITY);
        // Antiparallel along z: the cross product degenerates, RIGHT saves it.
        let m = rotation_between_vectors(OUT, [0.0, 0.0, -1.0]);
        close(
            vec::mul_point_mat(OUT, &vec::transpose(&m)),
            [0.0, 0.0, -1.0],
            1e-12,
        );
        // Antiparallel along RIGHT: RIGHT degenerates too, UP saves it.
        let m = rotation_between_vectors(RIGHT, [-1.0, 0.0, 0.0]);
        close(
            vec::mul_point_mat(RIGHT, &vec::transpose(&m)),
            [-1.0, 0.0, 0.0],
            1e-12,
        );
    }

    #[test]
    fn z_to_vector_sends_out_to_the_target_direction() {
        for target in [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 2.0, 3.0]] {
            let unit = normalize(target);
            let m = z_to_vector(unit);
            close(vec::mul_point_mat(OUT, &vec::transpose(&m)), unit, 1e-12);
        }
    }

    #[test]
    fn angle_helpers_are_defined_at_degeneracies() {
        assert_eq!(angle_of_vector([0.0; 3]), 0.0);
        assert_eq!(angle_between_vectors([0.0; 3], UP), 0.0);
        assert_eq!(angle_between_vectors(UP, [0.0; 3]), 0.0);
        // Antiparallel: exactly π, never a NaN from an out-of-range acos.
        assert!((angle_between_vectors(RIGHT, [-1.0, 0.0, 0.0]) - PI).abs() < 1e-15);
        assert_eq!(angle_between_vectors(RIGHT, RIGHT), 0.0);
    }

    #[test]
    fn unit_normal_falls_back_to_down() {
        assert_eq!(
            get_unit_normal([0.0; 3], [0.0; 3], DEFAULT_UNIT_NORMAL_TOL),
            DOWN
        );
        assert_eq!(get_unit_normal(OUT, OUT, DEFAULT_UNIT_NORMAL_TOL), DOWN);
        close(
            get_unit_normal(RIGHT, UP, DEFAULT_UNIT_NORMAL_TOL),
            OUT,
            1e-15,
        );
        // Aligned in the xy plane: the in-plane fallback is a real normal.
        let n = get_unit_normal(RIGHT, RIGHT, DEFAULT_UNIT_NORMAL_TOL);
        assert!((get_norm(n) - 1.0).abs() < 1e-15);
        assert!(dot(n, RIGHT).abs() < 1e-15);
    }

    #[test]
    fn quaternion_mult_of_nothing_is_the_identity() {
        assert_eq!(quaternion_mult(&[]), rotation::IDENTITY_QUAT);
        let q = quaternion_from_angle_axis(0.7, OUT);
        assert_eq!(quaternion_mult(&[q]), q);
    }

    #[test]
    fn conjugate_undoes_a_rotation() {
        let q = quaternion_from_angle_axis(1.1, [1.0, 2.0, 3.0]);
        let back = quaternion_mult(&[q, quaternion_conjugate(q)]);
        for lane in &back[..3] {
            assert!(lane.abs() < 1e-15);
        }
        assert!((back[3] - 1.0).abs() < 1e-15);
    }

    #[test]
    fn angle_axis_round_trips_and_refuses_the_identity() {
        assert!(angle_axis_from_quaternion(rotation::IDENTITY_QUAT).is_none());
        assert!(angle_axis_from_quaternion([0.0; 4]).is_none());
        let axis = normalize([1.0, -2.0, 0.5]);
        let (angle, back) = angle_axis_from_quaternion(quaternion_from_angle_axis(1.3, axis))
            .expect("non-identity");
        assert!((angle - 1.3).abs() < 1e-14);
        close(back, axis, 1e-14);
    }

    #[test]
    fn quaternion_matrices_are_transposes() {
        let q = quaternion_from_angle_axis(0.9, [0.3, 1.0, -0.2]);
        let m = rotation_matrix_from_quaternion(q).unwrap();
        let mt = rotation_matrix_transpose_from_quaternion(q).unwrap();
        for r in 0..3 {
            for c in 0..3 {
                assert!((m[r][c] - mt[c][r]).abs() < 1e-15);
            }
        }
        assert!(rotation_matrix_from_quaternion([0.0; 4]).is_none());
    }

    #[test]
    fn find_intersection_defines_the_parallel_case() {
        close(
            find_intersection(
                [0.0; 3],
                RIGHT,
                [1.0, -1.0, 0.0],
                UP,
                DEFAULT_INTERSECTION_THRESHOLD,
            ),
            [1.0, 0.0, 0.0],
            1e-12,
        );
        // Parallel: p0 verbatim, to the bit.
        assert_eq!(
            find_intersection(
                [3.0, 1.0, 0.0],
                RIGHT,
                [0.0, 1.0, 0.0],
                RIGHT,
                DEFAULT_INTERSECTION_THRESHOLD,
            ),
            [3.0, 1.0, 0.0]
        );
    }

    #[test]
    fn line_intersection_refuses_parallel_lines() {
        assert_eq!(
            line_intersection(
                ([0.0; 3], [1.0, 0.0, 0.0]),
                ([0.0, 1.0, 0.0], [1.0, 1.0, 0.0])
            ),
            None
        );
        let p = line_intersection(
            ([0.0; 3], [2.0, 2.0, 0.0]),
            ([0.0, 2.0, 0.0], [2.0, 0.0, 0.0]),
        )
        .unwrap();
        close(p, [1.0, 1.0, 0.0], 1e-12);
    }

    #[test]
    fn closest_point_clamps_and_survives_degeneracy() {
        let (a, b) = ([0.0; 3], [4.0, 0.0, 0.0]);
        close(
            get_closest_point_on_line(a, b, [2.0, 3.0, 0.0]),
            [2.0, 0.0, 0.0],
            1e-14,
        );
        close(get_closest_point_on_line(a, b, [-9.0, 1.0, 0.0]), a, 1e-14);
        close(get_closest_point_on_line(a, b, [99.0, 1.0, 0.0]), b, 1e-14);
        assert_eq!(get_closest_point_on_line(a, a, [1.0, 1.0, 1.0]), a);
    }

    #[test]
    fn winding_number_counts_turns() {
        let square = [
            [1.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0],
            [-1.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
        ];
        assert!((get_winding_number(&square) - 1.0).abs() < 1e-12);
        let mut reversed = square;
        reversed.reverse();
        assert!((get_winding_number(&reversed) + 1.0).abs() < 1e-12);
        let away = [
            [3.0, 1.0, 0.0],
            [4.0, 1.0, 0.0],
            [4.0, 2.0, 0.0],
            [3.0, 2.0, 0.0],
        ];
        assert!(get_winding_number(&away).abs() < 1e-12);
        assert_eq!(get_winding_number(&[]), 0.0);
    }

    #[test]
    fn triangle_predicates() {
        let (a, b, c) = ([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((tri_area(a, b, c) - 0.5).abs() < 1e-15);
        assert_eq!(tri_area(a, [1.0, 1.0, 0.0], [2.0, 2.0, 0.0]), 0.0);
        assert!(is_inside_triangle([0.25, 0.25, 0.0], a, b, c));
        assert!(!is_inside_triangle([2.0, 2.0, 0.0], a, b, c));
        // On an edge: not strictly inside.
        assert!(!is_inside_triangle([0.5, 0.5, 0.0], a, b, c));
        // Winding does not matter.
        assert!(is_inside_triangle([0.25, 0.25, 0.0], a, c, b));
    }

    #[test]
    fn compass_directions_close_the_circle() {
        let dirs = compass_directions(4, RIGHT).expect("four directions are within the cap");
        assert_eq!(dirs.len(), 4);
        close(dirs[0], RIGHT, 1e-15);
        close(dirs[1], UP, 1e-15);
        close(dirs[2], [-1.0, 0.0, 0.0], 1e-15);
        close(dirs[3], DOWN, 1e-15);
        assert!(
            compass_directions(0, RIGHT)
                .expect("zero directions are defined")
                .is_empty()
        );
    }

    #[test]
    fn compass_directions_enforce_the_exact_count_boundary() {
        assert_eq!(
            compass_directions(MAX_COMPASS_DIRECTIONS, RIGHT)
                .expect("the declared boundary is admitted")
                .len(),
            MAX_COMPASS_DIRECTIONS
        );
        assert_eq!(
            compass_directions(MAX_COMPASS_DIRECTIONS + 1, RIGHT),
            Err(SpaceOpsError::TooManyDirections {
                requested: MAX_COMPASS_DIRECTIONS + 1,
                max: MAX_COMPASS_DIRECTIONS,
            })
        );
        assert_eq!(
            compass_directions(usize::MAX, RIGHT),
            Err(SpaceOpsError::TooManyDirections {
                requested: usize::MAX,
                max: MAX_COMPASS_DIRECTIONS,
            })
        );
    }

    #[test]
    fn path_crossing_is_strict() {
        let path = [[0.0, -1.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]];
        assert!(line_intersects_path(
            [-2.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            &path
        ));
        assert!(!line_intersects_path(
            [-2.0, 5.0, 0.0],
            [2.0, 5.0, 0.0],
            &path
        ));
        // Collinear touch is not a crossing.
        assert!(!line_intersects_path(
            [0.0, -1.0, 0.0],
            [0.0, 1.0, 0.0],
            &path
        ));
        assert!(!line_intersects_path([0.0; 3], [1.0, 0.0, 0.0], &path[..1]));
    }

    #[test]
    fn lengths_and_centroids_define_the_empty_case() {
        assert_eq!(poly_line_length(&[]), 0.0);
        assert_eq!(poly_line_length(&[[3.0; 3]]), 0.0);
        assert_eq!(center_of_mass(&[]), [0.0; 3]);
        close(midpoint([0.0; 3], [2.0, 4.0, 6.0]), [1.0, 2.0, 3.0], 1e-15);
    }

    #[test]
    fn thick_diagonal_masks() {
        assert_eq!(
            thick_diagonal(3, 1).expect("three-by-three is within the cap"),
            vec![vec![1, 0, 0], vec![0, 1, 0], vec![0, 0, 1]]
        );
        assert_eq!(
            thick_diagonal(3, 2).expect("three-by-three is within the cap"),
            vec![vec![1, 1, 0], vec![1, 1, 1], vec![0, 1, 1]]
        );
        assert!(
            thick_diagonal(0, 2)
                .expect("zero dimension is defined")
                .is_empty()
        );
    }

    #[test]
    fn thick_diagonal_enforces_cell_budget_and_square_overflow() {
        let boundary = thick_diagonal(MAX_THICK_DIAGONAL_DIMENSION, usize::MAX)
            .expect("the declared boundary is admitted");
        assert_eq!(boundary.len(), MAX_THICK_DIAGONAL_DIMENSION);
        assert!(
            boundary
                .iter()
                .all(|row| row.len() == MAX_THICK_DIAGONAL_DIMENSION)
        );

        let one_over_dim = MAX_THICK_DIAGONAL_DIMENSION + 1;
        let one_over_cells = one_over_dim * one_over_dim;
        assert_eq!(
            thick_diagonal(one_over_dim, 1),
            Err(SpaceOpsError::TooManyDiagonalCells {
                requested: one_over_cells,
                max: MAX_THICK_DIAGONAL_CELLS,
            })
        );
        assert_eq!(
            thick_diagonal(usize::MAX, 1),
            Err(SpaceOpsError::DiagonalSizeOverflow { dim: usize::MAX })
        );
    }

    #[test]
    fn complex_helpers_round_trip() {
        assert_eq!(complex_to_r3([1.5, -2.0]), [1.5, -2.0, 0.0]);
        assert_eq!(r3_to_complex([1.5, -2.0, 9.0]), [1.5, -2.0]);
        let square = complex_func_to_r3_func(|z| [z[0] * z[0] - z[1] * z[1], 2.0 * z[0] * z[1]]);
        close(square([0.0, 1.0, 5.0]), [-1.0, 0.0, 0.0], 1e-15);
    }

    #[test]
    fn project_along_vector_removes_the_component() {
        let v = normalize([1.0, 1.0, 0.0]);
        let projected = project_along_vector([2.0, 0.0, 3.0], v);
        assert!(dot(projected, v).abs() < 1e-15);
        assert!((projected[2] - 3.0).abs() < 1e-15);
    }
}
