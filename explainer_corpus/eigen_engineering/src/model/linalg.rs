//! Small dense linear algebra, written out so that no number of the video
//! leans on a library: a square matrix, Gaussian elimination, and the
//! eigenpairs of a symmetric matrix by cyclic Jacobi rotations (the method
//! of FrankenSim's dense kernel, `crates/fs-la/src/eigen.rs`). The oracle
//! gets the same numbers from LAPACK through NumPy.

/// A square matrix, row by row.
#[derive(Clone, Debug, PartialEq)]
pub struct Mat {
    pub n: usize,
    pub a: Vec<f64>,
}

impl Mat {
    pub fn zeros(n: usize) -> Self {
        Self { n, a: vec![0.0; n * n] }
    }

    pub fn identity(n: usize) -> Self {
        let mut m = Self::zeros(n);
        for i in 0..n {
            m.a[i * n + i] = 1.0;
        }
        m
    }

    /// # Panics
    /// If the rows do not make a square.
    pub fn from_rows(rows: &[&[f64]]) -> Self {
        let n = rows.len();
        assert!(rows.iter().all(|r| r.len() == n), "a square matrix has n rows of n");
        Self {
            n,
            a: rows.iter().flat_map(|r| r.iter().copied()).collect(),
        }
    }

    pub fn at(&self, i: usize, j: usize) -> f64 {
        self.a[i * self.n + j]
    }

    pub fn add(&mut self, i: usize, j: usize, value: f64) {
        self.a[i * self.n + j] += value;
    }

    pub fn mul_vec(&self, x: &[f64]) -> Vec<f64> {
        (0..self.n)
            .map(|i| (0..self.n).map(|j| self.at(i, j) * x[j]).sum())
            .collect()
    }

    /// The rows and columns named by `keep`, in that order.
    pub fn sub(&self, keep: &[usize]) -> Self {
        let mut m = Self::zeros(keep.len());
        for (i, &r) in keep.iter().enumerate() {
            for (j, &c) in keep.iter().enumerate() {
                m.a[i * keep.len() + j] = self.at(r, c);
            }
        }
        m
    }

    /// The largest difference between an entry and its mirror image.
    pub fn asymmetry(&self) -> f64 {
        let mut worst = 0.0f64;
        for i in 0..self.n {
            for j in 0..i {
                worst = worst.max((self.at(i, j) - self.at(j, i)).abs());
            }
        }
        worst
    }
}

pub fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// The solution of `a x = b` by Gaussian elimination with partial pivoting,
/// or `None` when a pivot vanishes (a singular matrix).
pub fn solve(a: &Mat, b: &[f64]) -> Option<Vec<f64>> {
    let n = a.n;
    let mut m = a.a.clone();
    let mut x = b.to_vec();
    let scale = m.iter().fold(0.0f64, |s, v| s.max(v.abs()));
    for col in 0..n {
        let pivot = (col..n).max_by(|&i, &j| m[i * n + col].abs().total_cmp(&m[j * n + col].abs()))?;
        if m[pivot * n + col].abs() <= 1e-12 * scale {
            return None;
        }
        if pivot != col {
            for k in 0..n {
                m.swap(col * n + k, pivot * n + k);
            }
            x.swap(col, pivot);
        }
        for row in col + 1..n {
            let factor = m[row * n + col] / m[col * n + col];
            if factor != 0.0 {
                for k in col..n {
                    m[row * n + k] -= factor * m[col * n + k];
                }
                x[row] -= factor * x[col];
            }
        }
    }
    for row in (0..n).rev() {
        let tail: f64 = (row + 1..n).map(|k| m[row * n + k] * x[k]).sum();
        x[row] = (x[row] - tail) / m[row * n + row];
    }
    Some(x)
}

/// Eigenpairs, in ascending order of eigenvalue: `vectors[k]` goes with
/// `values[k]`.
#[derive(Clone, Debug)]
pub struct Eigen {
    pub values: Vec<f64>,
    pub vectors: Vec<Vec<f64>>,
}

/// The eigenpairs of a symmetric matrix by cyclic Jacobi: rotate away one
/// off-diagonal pair at a time until nothing is left off the diagonal. The
/// eigenvectors come out at right angles and of length one; each is given
/// the sign that makes its largest entry positive.
pub fn jacobi_eigh(m: &Mat) -> Eigen {
    let n = m.n;
    let mut a = m.a.clone();
    let mut v = Mat::identity(n).a;
    let total: f64 = a.iter().map(|x| x * x).sum();
    for _sweep in 0..100 {
        let mut off = 0.0;
        for i in 0..n {
            for j in 0..n {
                if i != j {
                    off += a[i * n + j] * a[i * n + j];
                }
            }
        }
        if off <= 1e-30 * total {
            break;
        }
        for p in 0..n.saturating_sub(1) {
            for q in p + 1..n {
                let apq = a[p * n + q];
                if apq == 0.0 {
                    continue;
                }
                let theta = (a[q * n + q] - a[p * n + p]) / (2.0 * apq);
                let t = theta.signum() / (theta.abs() + theta.hypot(1.0));
                let c = 1.0 / t.hypot(1.0);
                let s = t * c;
                for k in 0..n {
                    let (akp, akq) = (a[k * n + p], a[k * n + q]);
                    a[k * n + p] = c * akp - s * akq;
                    a[k * n + q] = s * akp + c * akq;
                }
                for k in 0..n {
                    let (apk, aqk) = (a[p * n + k], a[q * n + k]);
                    a[p * n + k] = c * apk - s * aqk;
                    a[q * n + k] = s * apk + c * aqk;
                }
                for k in 0..n {
                    let (vkp, vkq) = (v[k * n + p], v[k * n + q]);
                    v[k * n + p] = c * vkp - s * vkq;
                    v[k * n + q] = s * vkp + c * vkq;
                }
            }
        }
    }
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&i, &j| a[i * n + i].total_cmp(&a[j * n + j]));
    let values = order.iter().map(|&k| a[k * n + k]).collect();
    let vectors = order
        .iter()
        .map(|&k| {
            let mut col: Vec<f64> = (0..n).map(|i| v[i * n + k]).collect();
            let big = col.iter().fold(0.0f64, |s, x| s.max(x.abs()));
            // the first entry within a hair of the largest decides the sign,
            // so two machines never disagree over a near tie
            let lead = col
                .iter()
                .find(|x| x.abs() >= big * (1.0 - 1e-9))
                .copied()
                .unwrap_or(1.0);
            if lead < 0.0 {
                for x in &mut col {
                    *x = -*x;
                }
            }
            col
        })
        .collect();
    Eigen { values, vectors }
}

/// The eigenpairs of `k φ = λ m φ` for a symmetric `k` and a diagonal,
/// positive mass `m`: the standard problem for m^(-1/2) k m^(-1/2), mapped
/// back. The modes are scaled so that φᵀ m φ = 1.
pub fn generalized_eigh(k: &Mat, mass: &[f64]) -> Eigen {
    let n = k.n;
    let root: Vec<f64> = mass.iter().map(|m| m.sqrt()).collect();
    let mut reduced = Mat::zeros(n);
    for i in 0..n {
        for j in 0..n {
            reduced.a[i * n + j] = k.at(i, j) / (root[i] * root[j]);
        }
    }
    let e = jacobi_eigh(&reduced);
    let vectors = e
        .vectors
        .iter()
        .map(|y| y.iter().zip(&root).map(|(y, r)| y / r).collect())
        .collect();
    Eigen {
        values: e.values,
        vectors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jacobi_finds_the_two_carts_and_a_three_by_three_with_known_roots() {
        let e = jacobi_eigh(&Mat::from_rows(&[&[2.0, -1.0], &[-1.0, 2.0]]));
        assert!((e.values[0] - 1.0).abs() < 1e-14 && (e.values[1] - 3.0).abs() < 1e-14);
        let h = 0.5f64.sqrt();
        assert!((e.vectors[0][0] - h).abs() < 1e-14 && (e.vectors[0][1] - h).abs() < 1e-14);
        assert!((e.vectors[1][0] - h).abs() < 1e-14 && (e.vectors[1][1] + h).abs() < 1e-14);
        // tridiag(-1, 2, -1) of size 3: 2 - sqrt 2, 2, 2 + sqrt 2
        let e = jacobi_eigh(&Mat::from_rows(&[
            &[2.0, -1.0, 0.0],
            &[-1.0, 2.0, -1.0],
            &[0.0, -1.0, 2.0],
        ]));
        let want = [2.0 - 2f64.sqrt(), 2.0, 2.0 + 2f64.sqrt()];
        for (got, want) in e.values.iter().zip(want) {
            assert!((got - want).abs() < 1e-14, "{got} vs {want}");
        }
    }

    #[test]
    fn solve_inverts_and_refuses_a_singular_matrix() {
        let a = Mat::from_rows(&[&[0.0, 2.0, 1.0], &[1.0, 1.0, 0.0], &[3.0, 0.0, 1.0]]);
        let x = solve(&a, &[5.0, 3.0, 5.0]).unwrap();
        let back = a.mul_vec(&x);
        for (got, want) in back.iter().zip([5.0, 3.0, 5.0]) {
            assert!((got - want).abs() < 1e-13);
        }
        assert!(solve(&Mat::from_rows(&[&[1.0, 2.0], &[2.0, 4.0]]), &[1.0, 1.0]).is_none());
    }

    #[test]
    fn generalized_modes_are_at_right_angles_once_mass_is_counted() {
        let k = Mat::from_rows(&[&[2.0, -1.0], &[-1.0, 2.0]]);
        let mass = [1.0, 4.0];
        let e = generalized_eigh(&k, &mass);
        let m_dot = |a: &[f64], b: &[f64]| -> f64 { (0..2).map(|i| a[i] * mass[i] * b[i]).sum() };
        assert!(m_dot(&e.vectors[0], &e.vectors[1]).abs() < 1e-14);
        assert!((m_dot(&e.vectors[0], &e.vectors[0]) - 1.0).abs() < 1e-14);
        // and they are not at right angles in the plain sense
        assert!(dot(&e.vectors[0], &e.vectors[1]).abs() > 0.1);
        for (lambda, phi) in e.values.iter().zip(&e.vectors) {
            let left = k.mul_vec(phi);
            for i in 0..2 {
                assert!((left[i] - lambda * mass[i] * phi[i]).abs() < 1e-13);
            }
        }
    }
}
