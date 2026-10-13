//! The mathematics of Video J, as the video works it: small real square
//! matrices, the exponential by scaling and squaring (and, where the video
//! shows them, by the raw power series and by tiny steps), walks on a small
//! network, the 32-bit runs of the computing chapter, Padé approximants of
//! e, and the skew matrices whose exponentials are rotations.
//!
//! Pure math with unit tests; the chapters only draw. `oracle.py` gets the
//! same numbers by other routes (exact rational series, closed forms, walk
//! enumeration, SciPy's `expm`).

// The chapters are animatics until P4; `facts.rs` already uses all of this.
#![allow(dead_code)]

/// A real square matrix, row-major.
#[derive(Clone, Debug, PartialEq)]
pub struct Mat {
    n: usize,
    a: Vec<f64>,
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

    /// From rows; every row has as many entries as there are rows.
    pub fn from_rows(rows: &[&[f64]]) -> Self {
        let n = rows.len();
        assert!(rows.iter().all(|r| r.len() == n), "a square matrix");
        Self {
            n,
            a: rows.iter().flat_map(|r| r.iter().copied()).collect(),
        }
    }

    pub fn n(&self) -> usize {
        self.n
    }

    pub fn get(&self, i: usize, j: usize) -> f64 {
        self.a[i * self.n + j]
    }

    pub fn set(&mut self, i: usize, j: usize, v: f64) {
        self.a[i * self.n + j] = v;
    }

    pub fn entries(&self) -> &[f64] {
        &self.a
    }

    pub fn row(&self, i: usize) -> &[f64] {
        &self.a[i * self.n..(i + 1) * self.n]
    }

    pub fn column(&self, j: usize) -> Vec<f64> {
        (0..self.n).map(|i| self.get(i, j)).collect()
    }

    pub fn mul(&self, b: &Mat) -> Mat {
        assert_eq!(self.n, b.n);
        let n = self.n;
        let mut out = Mat::zeros(n);
        for i in 0..n {
            for j in 0..n {
                out.a[i * n + j] = (0..n).map(|k| self.get(i, k) * b.get(k, j)).sum();
            }
        }
        out
    }

    pub fn add(&self, b: &Mat) -> Mat {
        assert_eq!(self.n, b.n);
        Mat {
            n: self.n,
            a: self.a.iter().zip(&b.a).map(|(x, y)| x + y).collect(),
        }
    }

    pub fn sub(&self, b: &Mat) -> Mat {
        assert_eq!(self.n, b.n);
        Mat {
            n: self.n,
            a: self.a.iter().zip(&b.a).map(|(x, y)| x - y).collect(),
        }
    }

    pub fn scale(&self, c: f64) -> Mat {
        Mat {
            n: self.n,
            a: self.a.iter().map(|x| x * c).collect(),
        }
    }

    pub fn transpose(&self) -> Mat {
        let n = self.n;
        let mut out = Mat::zeros(n);
        for i in 0..n {
            for j in 0..n {
                out.a[j * n + i] = self.get(i, j);
            }
        }
        out
    }

    /// The matrix applied to an arrow.
    pub fn apply(&self, v: &[f64]) -> Vec<f64> {
        assert_eq!(v.len(), self.n);
        (0..self.n)
            .map(|i| self.row(i).iter().zip(v).map(|(a, x)| a * x).sum())
            .collect()
    }

    /// The k-th power by repeated multiplication (k = 0 is the identity).
    pub fn pow(&self, k: u32) -> Mat {
        (0..k).fold(Mat::identity(self.n), |p, _| p.mul(self))
    }

    /// The largest entry in absolute value.
    pub fn max_abs(&self) -> f64 {
        self.a.iter().fold(0.0, |m, x| m.max(x.abs()))
    }

    /// The largest absolute row sum (the ∞-norm), used to choose the scaling.
    pub fn norm_inf(&self) -> f64 {
        (0..self.n)
            .map(|i| self.row(i).iter().map(|x| x.abs()).sum::<f64>())
            .fold(0.0, f64::max)
    }

    pub fn trace(&self) -> f64 {
        (0..self.n).map(|i| self.get(i, i)).sum()
    }

    /// The determinant, by cofactors (the video's matrices are 4 by 4 at most).
    pub fn det(&self) -> f64 {
        let n = self.n;
        if n == 1 {
            return self.a[0];
        }
        (0..n)
            .map(|j| {
                let rows: Vec<Vec<f64>> = (1..n)
                    .map(|i| (0..n).filter(|&c| c != j).map(|c| self.get(i, c)).collect())
                    .collect();
                let refs: Vec<&[f64]> = rows.iter().map(Vec::as_slice).collect();
                let sign = if j % 2 == 0 { 1.0 } else { -1.0 };
                sign * self.get(0, j) * Mat::from_rows(&refs).det()
            })
            .sum()
    }

    /// AB − BA: zero exactly when the order of A and B does not matter.
    pub fn commutator(&self, b: &Mat) -> Mat {
        self.mul(b).sub(&b.mul(self))
    }

    /// The video's "size" of a matrix: the most it can stretch any arrow
    /// (the operator 2-norm, the square root of the largest eigenvalue of
    /// AᵀA).
    pub fn size(&self) -> f64 {
        let gram = self.transpose().mul(self);
        symmetric_eigenvalues(&gram)
            .last()
            .copied()
            .unwrap_or(0.0)
            .max(0.0)
            .sqrt()
    }

    /// e raised to each entry: the obvious guess the video opens with.
    pub fn entrywise_exp(&self) -> Mat {
        Mat {
            n: self.n,
            a: self.a.iter().map(|x| x.exp()).collect(),
        }
    }
}

/// The eigenvalues of a symmetric matrix, ascending, by cyclic Jacobi
/// rotations.
pub fn symmetric_eigenvalues(m: &Mat) -> Vec<f64> {
    let n = m.n();
    let mut a = m.clone();
    for _sweep in 0..64 {
        let off: f64 = (0..n)
            .flat_map(|i| (0..n).filter(move |&j| j != i).map(move |j| (i, j)))
            .map(|(i, j)| a.get(i, j).powi(2))
            .sum();
        if off < 1e-30 {
            break;
        }
        for p in 0..n {
            for q in p + 1..n {
                let apq = a.get(p, q);
                if apq.abs() < 1e-300 {
                    continue;
                }
                let theta = (a.get(q, q) - a.get(p, p)) / (2.0 * apq);
                let t = theta.signum() / (theta.abs() + (theta * theta + 1.0).sqrt());
                let t = if theta == 0.0 { 1.0 } else { t };
                let c = 1.0 / (t * t + 1.0).sqrt();
                let s = t * c;
                let mut r = Mat::identity(n);
                r.set(p, p, c);
                r.set(q, q, c);
                r.set(p, q, s);
                r.set(q, p, -s);
                a = r.transpose().mul(&a).mul(&r);
            }
        }
    }
    let mut eig: Vec<f64> = (0..n).map(|i| a.get(i, i)).collect();
    eig.sort_by(f64::total_cmp);
    eig
}

/// The Taylor degree and the scaled size the exponential is computed at.
const EXPM_DEGREE: u32 = 24;
const EXPM_SCALED_NORM: f64 = 0.5;

/// e^A by scaling and squaring: halve A until it is small, sum the series
/// there, square back up. (The oracle sums the exact series instead.)
pub fn expm(a: &Mat) -> Mat {
    let norm = a.norm_inf();
    let mut halvings = 0_i32;
    while norm * 0.5_f64.powi(halvings) > EXPM_SCALED_NORM {
        halvings += 1;
    }
    let x = a.scale(0.5_f64.powi(halvings));
    let mut sum = series(&x, EXPM_DEGREE).pop().expect("a last term").total;
    for _ in 0..halvings {
        sum = sum.mul(&sum);
    }
    sum
}

/// e^{tA}.
pub fn expm_t(a: &Mat, t: f64) -> Mat {
    expm(&a.scale(t))
}

/// One step of the power series: the term A^k/k! and the running total
/// through it.
#[derive(Clone, Debug)]
pub struct SeriesStep {
    pub k: u32,
    pub term: Mat,
    pub total: Mat,
}

/// The series I + A + A²/2! + … through the term A^kmax/kmax!.
pub fn series(a: &Mat, kmax: u32) -> Vec<SeriesStep> {
    let mut term = Mat::identity(a.n());
    let mut total = term.clone();
    let mut steps = vec![SeriesStep {
        k: 0,
        term: term.clone(),
        total: total.clone(),
    }];
    for k in 1..=kmax {
        term = term.mul(a).scale(1.0 / f64::from(k));
        total = total.add(&term);
        steps.push(SeriesStep {
            k,
            term: term.clone(),
            total: total.clone(),
        });
    }
    steps
}

/// n tiny steps of the rule A over a time t: (I + (t/n)A)^n. For a 1 by 1
/// matrix this is compound interest; for a system it is Euler's method.
pub fn tiny_steps(a: &Mat, t: f64, n: u32) -> Mat {
    Mat::identity(a.n()).add(&a.scale(t / f64::from(n))).pow(n)
}

/// (1 + x/n)^n: one unit growing at rate x, in n tiny steps.
pub fn compound(x: f64, n: u32) -> f64 {
    (1.0 + x / f64::from(n)).powi(i32::try_from(n).expect("a step count that fits"))
}

/// 1 + x + x²/2! + … + x^k/k!.
pub fn exp_partial(x: f64, k: u32) -> f64 {
    let mut term = 1.0;
    let mut total = 1.0;
    for j in 1..=k {
        term *= x / f64::from(j);
        total += term;
    }
    total
}

/// k!, exactly.
pub fn factorial(k: u32) -> u128 {
    (1..=u128::from(k)).product()
}

/// The quarter turn: (1, 0) goes to (0, 1).
pub fn quarter_turn() -> Mat {
    Mat::from_rows(&[&[0.0, -1.0], &[1.0, 0.0]])
}

/// The rotation of the plane by `t` radians, from cosine and sine.
pub fn rotation(t: f64) -> Mat {
    Mat::from_rows(&[&[t.cos(), -t.sin()], &[t.sin(), t.cos()]])
}

/// Two tanks of `litres` each, with `pumped` litres a minute going each way:
/// the rule for the grams of salt in each (per minute).
pub fn tanks(litres: f64, pumped: f64) -> Mat {
    let r = pumped / litres;
    Mat::from_rows(&[&[-r, r], &[r, -r]])
}

/// A small network: nodes 0..n and its links (each joins two different
/// nodes, once).
#[derive(Clone, Copy, Debug)]
pub struct Network {
    pub n: usize,
    pub links: &'static [(usize, usize)],
}

/// The video's network: a triangle (nodes 1, 2, 3) with a tail (node 4).
pub const KITE: Network = Network {
    n: 4,
    links: &[(0, 1), (0, 2), (1, 2), (2, 3)],
};

/// Two nodes and one link: the two tanks again.
pub const PAIR: Network = Network { n: 2, links: &[(0, 1)] };

impl Network {
    /// 1 where two nodes are linked, 0 elsewhere.
    pub fn adjacency(&self) -> Mat {
        let mut a = Mat::zeros(self.n);
        for &(i, j) in self.links {
            a.set(i, j, 1.0);
            a.set(j, i, 1.0);
        }
        a
    }

    /// How many links meet at a node.
    pub fn degree(&self, node: usize) -> usize {
        self.links.iter().filter(|&&(i, j)| i == node || j == node).count()
    }

    /// The Laplacian L = D − A (degrees on the diagonal, −1 for each link).
    pub fn laplacian(&self) -> Mat {
        let mut l = self.adjacency().scale(-1.0);
        for i in 0..self.n {
            l.set(i, i, self.degree(i) as f64);
        }
        l
    }

    /// The number of walks of exactly k links between every pair of nodes
    /// (a walk may revisit nodes and links), by adding one link at a time.
    pub fn walks(&self, k: u32) -> Vec<Vec<u64>> {
        let mut count: Vec<Vec<u64>> = (0..self.n)
            .map(|i| (0..self.n).map(|j| u64::from(i == j)).collect())
            .collect();
        for _ in 0..k {
            let mut next = vec![vec![0_u64; self.n]; self.n];
            for (from, row) in count.iter().enumerate() {
                for (mid, &ways) in row.iter().enumerate() {
                    for &(a, b) in self.links {
                        if a == mid {
                            next[from][b] += ways;
                        } else if b == mid {
                            next[from][a] += ways;
                        }
                    }
                }
            }
            count = next;
        }
        count
    }
}

/// A 2 by 2 matrix in 32-bit floating point, for the computing chapter.
pub type M2f = [[f32; 2]; 2];

fn mul_f32(a: &M2f, b: &M2f) -> M2f {
    let mut out = [[0.0_f32; 2]; 2];
    for i in 0..2 {
        for j in 0..2 {
            out[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j];
        }
    }
    out
}

/// The power series summed in 32-bit arithmetic until a term no longer
/// changes the running total: the total, and the number of the first term
/// that changed nothing. Each entry of a term is
/// `(t[i][0]·a[0][j] + t[i][1]·a[1][j]) / k`, every operation rounded to 32
/// bits (the oracle repeats it operation for operation).
pub fn series_f32(a: &M2f) -> (M2f, u32) {
    let mut term: M2f = [[1.0, 0.0], [0.0, 1.0]];
    let mut total = term;
    let mut k = 0_u32;
    loop {
        k += 1;
        let product = mul_f32(&term, a);
        let mut next = total;
        for i in 0..2 {
            for j in 0..2 {
                term[i][j] = product[i][j] / k as f32;
                next[i][j] = total[i][j] + term[i][j];
            }
        }
        if next == total {
            return (total, k);
        }
        total = next;
    }
}

/// Scaling and squaring in 32-bit arithmetic: halve the matrix `halvings`
/// times, sum the series through the term of that `degree`, square back.
pub fn scale_square_f32(a: &M2f, halvings: u32, degree: u32) -> M2f {
    let scale = 0.5_f32.powi(i32::try_from(halvings).expect("a small count"));
    let x: M2f = [[a[0][0] * scale, a[0][1] * scale], [a[1][0] * scale, a[1][1] * scale]];
    let mut term: M2f = [[1.0, 0.0], [0.0, 1.0]];
    let mut total = term;
    for k in 1..=degree {
        let product = mul_f32(&term, &x);
        for i in 0..2 {
            for j in 0..2 {
                term[i][j] = product[i][j] / k as f32;
                total[i][j] += term[i][j];
            }
        }
    }
    for _ in 0..halvings {
        total = mul_f32(&total, &total);
    }
    total
}

fn gcd(a: u128, b: u128) -> u128 {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// The [m/m] Padé approximant of e^x at x = 1, as a fraction in lowest
/// terms: the ratio of two degree-m polynomials whose series agrees with
/// e^x through x^{2m}. Coefficient of x^j, up to a common factor:
/// (2m − j)! / (j! (m − j)!), with alternating signs below.
pub fn pade_at_one(m: u32) -> (u128, u128) {
    let coeff = |j: u32| factorial(2 * m - j) / (factorial(j) * factorial(m - j));
    let top: u128 = (0..=m).map(coeff).sum();
    let plus: u128 = (0..=m).step_by(2).map(coeff).sum();
    let minus: u128 = (1..=m).step_by(2).map(coeff).sum();
    let bottom = plus - minus;
    let g = gcd(top, bottom);
    (top / g, bottom / g)
}

/// The skew matrix of a spin: `hat(v)` applied to an arrow is the cross
/// product v × arrow, so its exponential turns space about the axis v by
/// the angle |v|.
pub fn hat(v: [f64; 3]) -> Mat {
    let [a, b, c] = v;
    Mat::from_rows(&[&[0.0, -c, b], &[c, 0.0, -a], &[-b, a, 0.0]])
}

/// How far a matrix is from being a rotation: the largest entry of
/// RᵀR − I, and its determinant.
pub fn rotation_defect(r: &Mat) -> (f64, f64) {
    (r.transpose().mul(r).sub(&Mat::identity(r.n())).max_abs(), r.det())
}

/// Independent spins in n dimensions: one for each pair of axes.
pub fn spins(n: u32) -> u32 {
    n * (n - 1) / 2
}

/// The dimension of su(n): the independent tiny steps of SU(n).
pub fn su_dim(n: u32) -> u32 {
    n * n - 1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: &Mat, b: &Mat, tol: f64) -> bool {
        a.sub(b).max_abs() <= tol
    }

    #[test]
    fn the_exponential_of_the_quarter_turn_is_the_rotation() {
        let j = quarter_turn();
        for t in [0.0, 0.3, 1.0, 2.5, std::f64::consts::PI, 5.0, 40.0] {
            assert!(close(&expm_t(&j, t), &rotation(t), 1e-12), "t = {t}");
        }
    }

    #[test]
    fn scaling_and_squaring_agrees_with_the_raw_series_and_with_tiny_steps() {
        let a = Mat::from_rows(&[&[0.3, -1.2], &[0.7, 0.1]]);
        let by_series = series(&a, 40).pop().unwrap().total;
        assert!(close(&expm(&a), &by_series, 1e-13));
        // tiny steps approach it at the rate 1/n
        let e4 = tiny_steps(&a, 1.0, 10_000).sub(&expm(&a)).max_abs();
        let e5 = tiny_steps(&a, 1.0, 100_000).sub(&expm(&a)).max_abs();
        assert!(e5 < e4 / 9.0 && e5 < 2e-5, "{e4} {e5}");
    }

    #[test]
    fn the_exponential_of_a_one_by_one_matrix_is_the_number_exponential() {
        for x in [-3.0, -0.5, 0.0, 1.0, 2.0, 7.5_f64] {
            let m = Mat::from_rows(&[&[x]]);
            assert!((expm(&m).get(0, 0) - x.exp()).abs() <= 1e-12 * x.exp().max(1.0));
        }
    }

    #[test]
    fn sizes_are_what_the_closed_forms_say() {
        assert!((quarter_turn().size() - 1.0).abs() < 1e-12);
        let shear = Mat::from_rows(&[&[1.0, 1.0], &[0.0, 1.0]]);
        assert!((shear.size() - (1.0 + 5.0_f64.sqrt()) / 2.0).abs() < 1e-12);
        assert!((shear.pow(2).size() - (1.0 + 2.0_f64.sqrt())).abs() < 1e-12);
        let nil = Mat::from_rows(&[&[0.0, 1.0], &[0.0, 0.0]]);
        assert!((nil.size() - 1.0).abs() < 1e-12);
        assert_eq!(nil.pow(2), Mat::zeros(2));
    }

    #[test]
    fn walk_counts_are_the_powers_of_the_adjacency_matrix() {
        let a = KITE.adjacency();
        for k in 0..=8 {
            let power = a.pow(k);
            for (i, row) in KITE.walks(k).iter().enumerate() {
                for (j, &count) in row.iter().enumerate() {
                    assert_eq!(count as f64, power.get(i, j), "k = {k}, {i} to {j}");
                }
            }
        }
    }

    #[test]
    fn the_laplacian_spreads_without_creating_or_destroying() {
        let l = KITE.laplacian();
        for t in [0.1, 1.0, 7.0] {
            let heat = expm_t(&l, -t);
            for j in 0..4 {
                let total: f64 = heat.column(j).iter().sum();
                assert!((total - 1.0).abs() < 1e-12);
                assert!(heat.column(j).iter().all(|&x| x > 0.0));
            }
        }
    }

    #[test]
    fn pade_fractions() {
        assert_eq!(pade_at_one(1), (3, 1));
        assert_eq!(pade_at_one(2), (19, 7));
        assert_eq!(pade_at_one(3), (193, 71));
        assert_eq!(pade_at_one(4), (2721, 1001));
        let (p, q) = pade_at_one(13);
        assert!((p as f64 / q as f64 - std::f64::consts::E).abs() < 1e-15);
    }

    #[test]
    fn the_exponential_of_a_spin_is_a_rotation_about_its_axis() {
        let v = [0.3, -1.1, 0.7];
        let r = expm(&hat(v));
        let (defect, det) = rotation_defect(&r);
        assert!(defect < 1e-13 && (det - 1.0).abs() < 1e-13);
        // the axis does not move
        let moved = r.apply(&v);
        assert!(moved.iter().zip(&v).all(|(a, b)| (a - b).abs() < 1e-13));
        // the angle is the length of v: trace = 1 + 2 cos(angle)
        let angle = v.iter().map(|x| x * x).sum::<f64>().sqrt();
        assert!((r.trace() - (1.0 + 2.0 * angle.cos())).abs() < 1e-13);
    }

    #[test]
    fn the_determinant_of_an_exponential_is_e_to_the_trace() {
        let a = Mat::from_rows(&[&[0.2, 1.0, -0.5], &[0.0, -0.7, 0.3], &[1.1, 0.4, 0.1]]);
        assert!((expm(&a).det() - a.trace().exp()).abs() < 1e-12);
    }
}
