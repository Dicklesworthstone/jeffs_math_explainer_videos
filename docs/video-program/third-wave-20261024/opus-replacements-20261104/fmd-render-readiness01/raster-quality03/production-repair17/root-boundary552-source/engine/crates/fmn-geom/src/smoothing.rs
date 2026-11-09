//! Anchor-mode smoothing: the approximate local handle rule and the true
//! spline solve — ports of `approx_smooth_quadratic_bezier_handles`,
//! `get_smooth_cubic_bezier_handle_points`, and `smooth_quadratic_path`
//! (`manimlib/utils/bezier.py` @ `6199a00d`), computed in f64.
//!
//! The linear solves route through fsci-linalg per doctrine D4: banded
//! elimination for open paths, dense solve for closed ones, both in Strict
//! mode so refusal semantics match scipy's defaults (`check_finite`,
//! singular systems refuse rather than return garbage). The public functions
//! here kept their signatures across the migration.

use crate::GeomError;
use crate::cubic;
use crate::space_ops;
use crate::vec;
use fmn_core::types::Vec3;

/// Maximum dimension of the dense system used for a closed spline.
///
/// A closed path with `a` anchors (including the repeated endpoint) has
/// `2 * (a - 1)` unknown handles. Capping that dimension at 256 admits up to
/// 129 anchors while bounding the dense solves to a fixed work budget.
/// Raising it requires an equally explicit resource contract.
pub const MAX_CLOSED_SMOOTHING_DIMENSION: usize = 256;

/// Maximum number of `f64` cells in a closed-smoothing dense matrix.
///
/// The solver retains the matrix and one per-coordinate clone concurrently,
/// so this 65,536-cell ceiling bounds their raw payload to 1 MiB in total,
/// before row-vector metadata.
pub const MAX_CLOSED_SMOOTHING_MATRIX_CELLS: usize =
    MAX_CLOSED_SMOOTHING_DIMENSION * MAX_CLOSED_SMOOTHING_DIMENSION;

fn smoothing_dimension(anchor_count: usize) -> Result<usize, GeomError> {
    anchor_count
        .saturating_sub(1)
        .checked_mul(2)
        .ok_or(GeomError::SmoothingSizeOverflow {
            anchors: anchor_count,
        })
}

fn validate_closed_smoothing_budget(
    anchor_count: usize,
    dimension: usize,
) -> Result<usize, GeomError> {
    let cells = dimension
        .checked_mul(dimension)
        .ok_or(GeomError::SmoothingSizeOverflow {
            anchors: anchor_count,
        })?;
    if dimension > MAX_CLOSED_SMOOTHING_DIMENSION || cells > MAX_CLOSED_SMOOTHING_MATRIX_CELLS {
        return Err(GeomError::ClosedSmoothingBudgetExceeded { dimension, cells });
    }
    Ok(cells)
}

/// `approx_smooth_quadratic_bezier_handles`: local (solver-free) handles that
/// make each quadratic part of a parabola through its neighbor anchors.
/// Returns one handle per gap (`anchors.len() - 1`), or a single point for a
/// one-anchor input, matching the Reference's degenerate returns.
#[must_use]
pub fn approx_smooth_quadratic_handles(anchors: &[Vec3]) -> Vec<Vec3> {
    let n = anchors.len();
    if n == 1 {
        return vec![anchors[0]];
    }
    if n == 2 {
        return vec![space_ops::midpoint(anchors[0], anchors[1])];
    }
    // smooth_to_right[i] = ¼ p[i] + p[i+1] − ¼ p[i+2] on the forward points;
    // smooth_to_left is the same rule on the reversed sequence.
    let str_at = |i: usize| -> Vec3 {
        vec::sub(
            vec::add(vec::scale(anchors[i], 0.25), anchors[i + 1]),
            vec::scale(anchors[i + 2], 0.25),
        )
    };
    let stl_at = |i: usize| -> Vec3 {
        // reversed points: rp[k] = p[n-1-k]
        vec::sub(
            vec::add(vec::scale(anchors[n - 1 - i], 0.25), anchors[n - 2 - i]),
            vec::scale(anchors[n - 3 - i], 0.25),
        )
    };
    let closed = vec::np_isclose_all(anchors[0], anchors[n - 1]);
    let (last_str, last_stl) = if closed {
        (
            vec::sub(
                vec::add(vec::scale(anchors[n - 2], 0.25), anchors[n - 1]),
                vec::scale(anchors[1], 0.25),
            ),
            vec::sub(
                vec::add(vec::scale(anchors[1], 0.25), anchors[0]),
                vec::scale(anchors[n - 2], 0.25),
            ),
        )
    } else {
        (stl_at(0), str_at(0))
    };
    (0..n - 1)
        .map(|i| {
            let first = if i < n - 2 { str_at(i) } else { last_str };
            let second = if i == 0 { last_stl } else { stl_at(n - 2 - i) };
            vec::scale(vec::add(first, second), 0.5)
        })
        .collect()
}

/// `get_smooth_cubic_bezier_handle_points`: the two cubic handle sequences
/// making a C² spline through `anchors` — the banded system of
/// particleincell.com/2012/bezier-splines for open paths, with the
/// Reference's first/second-derivative row replacements for closed ones.
pub fn smooth_cubic_handles(anchors: &[Vec3]) -> Result<(Vec<Vec3>, Vec<Vec3>), GeomError> {
    let n_pts = anchors.len();
    if n_pts < 2 {
        return Ok((Vec::new(), Vec::new()));
    }
    let n = smoothing_dimension(n_pts)?;
    let closed = vec::np_isclose_all(anchors[0], anchors[n_pts - 1]);
    if closed {
        validate_closed_smoothing_budget(n_pts, n)?;
    }
    let (l, u) = SMOOTHING_BANDS;
    let ab = smoothing_band(n);

    let mut b = vec![[0.0f64; 3]; n];
    for (k, anchor) in anchors.iter().enumerate().skip(1) {
        b[2 * k - 1] = vec::scale(*anchor, 2.0);
    }
    b[0] = anchors[0];
    b[n - 1] = anchors[n_pts - 1];

    let mut solution = vec![[0.0f64; 3]; n];
    if closed {
        let matrix = closed_smoothing_matrix(&ab, n);
        b[0] = vec::scale(anchors[0], 2.0);
        b[n - 1] = [0.0; 3];
        for dim in 0..3 {
            let mut rhs: Vec<f64> = b.iter().map(|row| row[dim]).collect();
            solve_dense(&matrix, &mut rhs)?;
            for (row, value) in solution.iter_mut().zip(rhs) {
                row[dim] = value;
            }
        }
    } else {
        for dim in 0..3 {
            let mut rhs: Vec<f64> = b.iter().map(|row| row[dim]).collect();
            solve_banded(l, u, &ab, &mut rhs)?;
            for (row, value) in solution.iter_mut().zip(rhs) {
                row[dim] = value;
            }
        }
    }

    let h1 = solution.iter().step_by(2).copied().collect();
    let h2 = solution.iter().skip(1).step_by(2).copied().collect();
    Ok((h1, h2))
}

/// `smooth_quadratic_path`: a smooth quadratic spline through `anchors`, in
/// shared-anchor layout. Non-flat inputs are rotated to a plane, smoothed,
/// and rotated back, exactly as the Reference does.
///
/// Every solved cubic routes through the one error-bounded converter (§7.2).
/// `tolerance` is the global scene-unit ceiling; each source segment also
/// keeps the predecessor contract `0.1 * chord_length`, whichever is tighter.
pub fn smooth_quadratic_path(anchors: &[Vec3], tolerance: f64) -> Result<Vec<Vec3>, GeomError> {
    if !tolerance.is_finite() || tolerance <= 0.0 {
        return Err(GeomError::InvalidTolerance);
    }
    if anchors.len() < 2 {
        return Ok(anchors.to_vec());
    }
    if anchors.len() == 2 {
        let mean = space_ops::midpoint(anchors[0], anchors[1]);
        return Ok(vec![anchors[0], mean, anchors[1]]);
    }

    let is_flat = anchors.iter().all(|p| p[2] == 0.0);
    let mut working: Vec<Vec3> = anchors.to_vec();
    let mut rot = vec::IDENTITY;
    let mut shift = 0.0;
    if !is_flat {
        let normal = space_ops::cross(
            vec::sub(anchors[2], anchors[1]),
            vec::sub(anchors[1], anchors[0]),
        );
        rot = space_ops::z_to_vector(normal);
        for p in working.iter_mut() {
            *p = vec::mul_point_mat(*p, &rot);
        }
        shift = working[0][2];
        for p in working.iter_mut() {
            p[2] -= shift;
        }
    }

    let (h1s, h2s) = smooth_cubic_handles(&working)?;
    // Work in the xy-plane like the Reference (it collects 2D rows and lifts
    // back at the end); z is zero here by construction.
    let mut quads: Vec<Vec3> = vec![[working[0][0], working[0][1], 0.0]];
    for i in 0..working.len() - 1 {
        let chord_tolerance = 0.1 * space_ops::get_norm(vec::sub(working[i + 1], working[i]));
        let segment_tolerance = if chord_tolerance.is_finite() && chord_tolerance > 0.0 {
            tolerance.min(chord_tolerance)
        } else {
            tolerance
        };
        let approx = cubic::cubic_to_quadratics(
            working[i],
            h1s[i],
            h2s[i],
            working[i + 1],
            segment_tolerance,
        )?;
        for p in &approx[1..] {
            quads.push([p[0], p[1], 0.0]);
        }
    }

    if !is_flat {
        let rot_t = vec::transpose(&rot);
        for p in quads.iter_mut() {
            p[2] += shift;
            *p = vec::mul_point_mat(*p, &rot_t);
        }
    }
    Ok(quads)
}

/// Lower and upper bandwidths of the smoothing system.
const SMOOTHING_BANDS: (usize, usize) = (2, 1);

/// The smoothing system of dimension `n` in LAPACK band storage,
/// `ab[u + i - j][j] = A[i][j]`. It depends on `n` alone.
fn smoothing_band(n: usize) -> Vec<Vec<f64>> {
    let (l, u) = SMOOTHING_BANDS;
    let mut ab = vec![vec![0.0; n]; l + u + 1];
    for j in (1..n).step_by(2) {
        ab[0][j] = -1.0;
    }
    for j in (2..n).step_by(2) {
        ab[0][j] = 1.0;
    }
    for j in (0..n).step_by(2) {
        ab[1][j] = 2.0;
    }
    for j in (1..n).step_by(2) {
        ab[1][j] = 1.0;
    }
    if n >= 2 {
        for j in (1..n.saturating_sub(2)).step_by(2) {
            ab[2][j] = -2.0;
        }
        for j in (0..n.saturating_sub(3)).step_by(2) {
            ab[3][j] = 1.0;
        }
        ab[2][n - 2] = -1.0;
        ab[1][n - 1] = 2.0;
    }
    ab
}

/// The dense closed-path system: the band matrix with its first and last
/// rows replaced by the seam conditions. Like the band, it depends on `n`
/// alone.
fn closed_smoothing_matrix(ab: &[Vec<f64>], n: usize) -> Vec<Vec<f64>> {
    let (l, u) = SMOOTHING_BANDS;
    let mut matrix = band_to_dense(l, u, ab, n);
    // Last row relates second derivatives across the seam,
    // first row relates first derivatives.
    for x in matrix[n - 1].iter_mut() {
        *x = 0.0;
    }
    matrix[n - 1][0] = 2.0;
    matrix[n - 1][1] = -1.0;
    matrix[n - 1][n - 2] = 1.0;
    matrix[n - 1][n - 1] = -2.0;
    for x in matrix[0].iter_mut() {
        *x = 0.0;
    }
    matrix[0][0] = 1.0;
    matrix[0][n - 1] = 1.0;
    matrix
}

/// Expand LAPACK band storage into a dense matrix
/// (`bezier.diag_to_matrix`).
fn band_to_dense(l: usize, u: usize, ab: &[Vec<f64>], n: usize) -> Vec<Vec<f64>> {
    let mut m = vec![vec![0.0; n]; n];
    for (r, band_row) in ab.iter().enumerate().take(l + u + 1) {
        for (j, &value) in band_row.iter().enumerate() {
            // A[i][j] with i = r + j - u.
            let i = r as isize + j as isize - u as isize;
            if (0..n as isize).contains(&i) {
                m[i as usize][j] = value;
            }
        }
    }
    m
}

/// Banded solve of `A x = b` through fsci-linalg, `A` given in band storage
/// `ab[u + i - j][j] = A[i][j]`, the solution written into `b`.
fn solve_banded(l: usize, u: usize, ab: &[Vec<f64>], b: &mut [f64]) -> Result<(), GeomError> {
    let solved = fsci_linalg::solve_banded((l, u), ab, b, fsci_linalg::SolveOptions::default())
        .map_err(solver_error)?;
    b.copy_from_slice(&solved.x);
    Ok(())
}

/// Dense solve of `A x = b` through fsci-linalg, the solution written into
/// `b`.
fn solve_dense(m: &[Vec<f64>], b: &mut [f64]) -> Result<(), GeomError> {
    let solved =
        fsci_linalg::solve(m, b, fsci_linalg::SolveOptions::default()).map_err(solver_error)?;
    b.copy_from_slice(&solved.x);
    Ok(())
}

/// Maps fsci-linalg refusals onto the smoothing error surface: a singular
/// system keeps its established variant; every other refusal (non-finite
/// coordinates above all) must not masquerade as singularity.
fn solver_error(error: fsci_linalg::LinalgError) -> GeomError {
    match error {
        fsci_linalg::LinalgError::SingularMatrix => GeomError::SingularSystem,
        _ => GeomError::SolverRefused,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AnchorMode, QuadPath};

    fn zigzag_anchors(count: usize, closed: bool) -> Vec<Vec3> {
        assert!(count >= 2);
        let mut anchors: Vec<Vec3> = (0..count)
            .map(|index| [index as f64, (index % 2) as f64, 0.0])
            .collect();
        if closed {
            anchors[count - 1] = anchors[0];
        }
        anchors
    }

    #[test]
    fn closed_smoothing_matrix_is_never_symmetric() {
        // fm-bp82: fsci-linalg's only FMA kernels (its blocked Cholesky)
        // run for symmetric systems of dimension >= 128, and solve_dense
        // passes no assume_a, so only a detected-symmetric matrix could
        // reach them. The closed system depends on n alone, so checking
        // every admissible dimension proves the certified path never does.
        // (n = 2, the degenerate two-anchor loop, is symmetric but 64x below
        // that dimension floor.)
        for n in (4..=MAX_CLOSED_SMOOTHING_DIMENSION).step_by(2) {
            let m = closed_smoothing_matrix(&smoothing_band(n), n);
            let symmetric = (0..n).all(|i| (0..n).all(|j| m[i][j] == m[j][i]));
            assert!(
                !symmetric,
                "the closed system of dimension {n} is symmetric"
            );
        }
        let m = closed_smoothing_matrix(&smoothing_band(2), 2);
        assert_eq!(m, vec![vec![1.0, 1.0], vec![1.0, -2.0]]);
    }

    #[test]
    fn closed_smoothing_budget_checks_dimension_cells_and_overflow() {
        let exact_anchors = MAX_CLOSED_SMOOTHING_DIMENSION / 2 + 1;
        let exact_dimension = smoothing_dimension(exact_anchors).unwrap();
        assert_eq!(exact_dimension, MAX_CLOSED_SMOOTHING_DIMENSION);
        assert_eq!(
            validate_closed_smoothing_budget(exact_anchors, exact_dimension),
            Ok(MAX_CLOSED_SMOOTHING_MATRIX_CELLS)
        );

        let over_anchors = exact_anchors + 1;
        let over_dimension = smoothing_dimension(over_anchors).unwrap();
        let over_cells = over_dimension * over_dimension;
        assert_eq!(
            validate_closed_smoothing_budget(over_anchors, over_dimension),
            Err(GeomError::ClosedSmoothingBudgetExceeded {
                dimension: over_dimension,
                cells: over_cells,
            })
        );

        assert_eq!(
            smoothing_dimension(usize::MAX),
            Err(GeomError::SmoothingSizeOverflow {
                anchors: usize::MAX,
            })
        );
        let square_overflow_anchors = usize::MAX / 2;
        let square_overflow_dimension = smoothing_dimension(square_overflow_anchors).unwrap();
        assert_eq!(
            validate_closed_smoothing_budget(square_overflow_anchors, square_overflow_dimension),
            Err(GeomError::SmoothingSizeOverflow {
                anchors: square_overflow_anchors,
            })
        );
    }

    #[test]
    fn closed_smoothing_admits_the_boundary_and_refuses_one_over() {
        let exact_anchors = MAX_CLOSED_SMOOTHING_DIMENSION / 2 + 1;
        let exact = zigzag_anchors(exact_anchors, true);
        let (h1, h2) = smooth_cubic_handles(&exact).unwrap();
        assert_eq!(h1.len(), exact_anchors - 1);
        assert_eq!(h2.len(), exact_anchors - 1);

        let over_anchors = exact_anchors + 1;
        let over_dimension = 2 * (over_anchors - 1);
        let expected = GeomError::ClosedSmoothingBudgetExceeded {
            dimension: over_dimension,
            cells: over_dimension * over_dimension,
        };
        let closed = zigzag_anchors(over_anchors, true);
        assert_eq!(smooth_cubic_handles(&closed).unwrap_err(), expected);
        assert_eq!(
            smooth_quadratic_path(&closed, cubic::DEFAULT_TOLERANCE_SCENE).unwrap_err(),
            expected
        );

        let open = zigzag_anchors(over_anchors, false);
        let (h1, h2) = smooth_cubic_handles(&open).unwrap();
        assert_eq!(h1.len(), over_anchors - 1);
        assert_eq!(h2.len(), over_anchors - 1);
    }

    #[test]
    fn quadpath_propagates_closed_smoothing_refusal_without_mutation() {
        let anchor_count = MAX_CLOSED_SMOOTHING_DIMENSION / 2 + 2;
        let anchors = zigzag_anchors(anchor_count, true);
        let mut path = QuadPath::new();
        path.set_points_as_corners(&anchors).unwrap();
        let before = path.points().to_vec();

        let dimension = 2 * (anchor_count - 1);
        assert_eq!(
            path.change_anchor_mode(AnchorMode::TrueSmooth).unwrap_err(),
            GeomError::ClosedSmoothingBudgetExceeded {
                dimension,
                cells: dimension * dimension,
            }
        );
        assert_eq!(path.points(), before);
    }

    #[test]
    fn two_anchor_handles_sit_at_thirds() {
        // Closed form: the natural cubic through two points is the straight
        // line, handles at ⅓ and ⅔.
        let anchors = [[0.0, 0.0, 0.0], [3.0, 0.0, 0.0]];
        let (h1, h2) = smooth_cubic_handles(&anchors).unwrap();
        assert_eq!(h1.len(), 1);
        assert!((h1[0][0] - 1.0).abs() < 1e-12);
        assert!((h2[0][0] - 2.0).abs() < 1e-12);
    }

    #[test]
    fn collinear_anchors_yield_collinear_handles() {
        let anchors = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
        ];
        let (h1, h2) = smooth_cubic_handles(&anchors).unwrap();
        for h in h1.iter().chain(h2.iter()) {
            assert!(h[1].abs() < 1e-12 && h[2].abs() < 1e-12);
        }
        // Handles are ordered along the line within each segment.
        for i in 0..3 {
            assert!(h1[i][0] > i as f64 && h2[i][0] < (i + 1) as f64 + 1.0);
        }
    }

    #[test]
    fn smooth_path_passes_through_anchors() {
        let anchors = [
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 1.0, 0.0],
        ];
        let path = smooth_quadratic_path(&anchors, cubic::DEFAULT_TOLERANCE_SCENE).unwrap();
        assert_eq!(path.len() % 2, 1);
        // Every input anchor appears as an anchor of the output spline.
        for a in anchors {
            assert!(
                path.iter()
                    .step_by(2)
                    .any(|p| space_ops::get_norm(vec::sub(*p, a)) < 1e-9),
                "anchor {a:?} missing from smoothed path"
            );
        }
    }

    #[test]
    fn approx_handles_degenerate_inputs() {
        let single = approx_smooth_quadratic_handles(&[[1.0, 2.0, 0.0]]);
        assert_eq!(single, vec![[1.0, 2.0, 0.0]]);
        let pair = approx_smooth_quadratic_handles(&[[0.0, 0.0, 0.0], [2.0, 2.0, 0.0]]);
        assert_eq!(pair, vec![[1.0, 1.0, 0.0]]);
    }
}
