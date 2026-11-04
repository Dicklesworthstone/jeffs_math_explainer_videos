//! The mathematics every scene draws from: the ratio ruler, the folded
//! decade, complex numbers with their principal and continued logarithms,
//! and 2×2 real matrices with their exponential (a scaled Taylor series,
//! deliberately a different algorithm from oracle.py's closed form).
//!
//! Rebuild candidate for `mx-video-program-program-87do.63`; uncompiled here.

use std::f64::consts::{PI, TAU};

// ------------------------------------------------------------- the ruler

/// Position of `x` on a ruler whose step multiplies by `base` (x > 0).
pub fn position(x: f64, base: f64) -> f64 {
    x.ln() / base.ln()
}

/// Equal temperament with A4 = 440 Hz: the frequency `n` keys above A4.
pub fn key_hz(n: i32) -> f64 {
    440.0 * 2f64.powf(f64::from(n) / 12.0)
}

/// The decade fold of a positive number: (digits in [1, 10), laps), with
/// `digits × 10^laps = x`. The dial shows only the digits.
pub fn fold_decade(x: f64) -> (f64, i32) {
    assert!(x > 0.0, "only positive numbers sit on the ruler");
    let laps = x.log10().floor() as i32;
    (x / 10f64.powi(laps), laps)
}

// ------------------------------------------------------- complex numbers

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct C {
    pub re: f64,
    pub im: f64,
}

impl C {
    pub const fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }
    pub fn polar(r: f64, theta: f64) -> Self {
        Self::new(r * theta.cos(), r * theta.sin())
    }
    pub fn abs(self) -> f64 {
        self.re.hypot(self.im)
    }
    /// The principal angle, in (−π, π] (DLMF 4.2.3's convention on the cut).
    pub fn arg(self) -> f64 {
        self.im.atan2(self.re)
    }
    pub fn mul(self, o: Self) -> Self {
        Self::new(self.re * o.re - self.im * o.im, self.re * o.im + self.im * o.re)
    }
    pub fn add(self, o: Self) -> Self {
        Self::new(self.re + o.re, self.im + o.im)
    }
    pub fn sub(self, o: Self) -> Self {
        Self::new(self.re - o.re, self.im - o.im)
    }
    pub fn scale(self, k: f64) -> Self {
        Self::new(self.re * k, self.im * k)
    }
    /// e^(x + iy) = e^x (cos y + i sin y).
    pub fn exp(self) -> Self {
        Self::polar(self.re.exp(), self.im)
    }
    /// The principal logarithm, ln|z| + i Arg z (z ≠ 0).
    pub fn log(self) -> Self {
        Self::new(self.abs().ln(), self.arg())
    }
    /// The logarithm on lap `k`: ln|z| + i(Arg z + 2πk).
    pub fn log_on_lap(self, k: i32) -> Self {
        Self::new(self.abs().ln(), self.arg() + TAU * f64::from(k))
    }
    /// The principal square root, e^(½ Log z).
    pub fn sqrt(self) -> Self {
        self.log().scale(0.5).exp()
    }
}

/// Bring an angle into the principal window (−π, π].
pub fn wrap(theta: f64) -> f64 {
    let w = theta - TAU * ((theta + PI) / TAU).floor();
    if w <= -PI { w + TAU } else { w }
}

/// Carry the logarithm continuously along a sampled path that avoids zero:
/// start on the principal value and add each step's small angle change.
/// Consecutive samples must turn by less than a half turn about zero.
pub fn lift(path: &[C]) -> Vec<C> {
    let mut out = Vec::with_capacity(path.len());
    let Some(&first) = path.first() else { return out };
    out.push(first.log());
    for pair in path.windows(2) {
        let (p, q) = (pair[0], pair[1]);
        let step = q.mul(C::new(p.re, -p.im)).arg();
        let last = *out.last().expect("pushed above");
        out.push(C::new(q.abs().ln(), last.im + step));
    }
    out
}

/// How many times a closed sampled loop winds counter-clockwise about zero.
pub fn winding(path: &[C]) -> i32 {
    let l = lift(path);
    match (l.first(), l.last()) {
        (Some(a), Some(b)) => ((b.im - a.im) / TAU).round() as i32,
        _ => 0,
    }
}

/// A circle of `n` samples (n + 1 points, closed) about `centre`, radius `r`,
/// walked `laps` times counter-clockwise.
pub fn circle_path(centre: C, r: f64, laps: f64, n: usize) -> Vec<C> {
    (0..=n)
        .map(|k| centre.add(C::polar(r, TAU * laps * k as f64 / n as f64)))
        .collect()
}

/// Exact angle arithmetic in units of π/q: the principal representative of
/// an angle `p` (units of π/q) in the window (−q, q].
pub fn principal_units(p: i64, q: i64) -> i64 {
    let r = (p + q).rem_euclid(2 * q) - q;
    if r == -q { q } else { r }
}

// -------------------------------------------------------------- matrices

/// A real 2×2 matrix, rows first.
pub type M2 = [[f64; 2]; 2];

pub const I2: M2 = [[1.0, 0.0], [0.0, 1.0]];
/// The quarter-turn generator: J p turns p a quarter turn counter-clockwise.
pub const J: M2 = [[0.0, -1.0], [1.0, 0.0]];

pub fn mmul(a: M2, b: M2) -> M2 {
    let mut c = [[0.0; 2]; 2];
    for i in 0..2 {
        for j in 0..2 {
            c[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j];
        }
    }
    c
}
pub fn madd(a: M2, b: M2) -> M2 {
    [
        [a[0][0] + b[0][0], a[0][1] + b[0][1]],
        [a[1][0] + b[1][0], a[1][1] + b[1][1]],
    ]
}
pub fn mscale(a: M2, k: f64) -> M2 {
    [[a[0][0] * k, a[0][1] * k], [a[1][0] * k, a[1][1] * k]]
}
pub fn det(a: M2) -> f64 {
    a[0][0] * a[1][1] - a[0][1] * a[1][0]
}
pub fn trace(a: M2) -> f64 {
    a[0][0] + a[1][1]
}
pub fn inverse(a: M2) -> M2 {
    let d = det(a);
    [[a[1][1] / d, -a[0][1] / d], [-a[1][0] / d, a[0][0] / d]]
}
pub fn apply(a: M2, p: [f64; 2]) -> [f64; 2] {
    [a[0][0] * p[0] + a[0][1] * p[1], a[1][0] * p[0] + a[1][1] * p[1]]
}
pub fn rotation(theta: f64) -> M2 {
    let (s, c) = theta.sin_cos();
    [[c, -s], [s, c]]
}
/// The largest entry difference.
pub fn mdist(a: M2, b: M2) -> f64 {
    let mut m: f64 = 0.0;
    for i in 0..2 {
        for j in 0..2 {
            m = m.max((a[i][j] - b[i][j]).abs());
        }
    }
    m
}

/// e^X by scaling and squaring a Taylor series: halve X until its entries
/// are small, sum thirty terms, then square back.
pub fn mexp(x: M2) -> M2 {
    let norm = x.iter().flatten().fold(0.0_f64, |m, v| m.max(v.abs()));
    let mut squarings = 0;
    while norm / 2f64.powi(squarings) > 0.25 {
        squarings += 1;
    }
    let small = mscale(x, 1.0 / 2f64.powi(squarings));
    let (mut sum, mut term) = (I2, I2);
    for k in 1..=30 {
        term = mscale(mmul(term, small), 1.0 / f64::from(k));
        sum = madd(sum, term);
    }
    for _ in 0..squarings {
        sum = mmul(sum, sum);
    }
    sum
}

/// The flow of velocity `x` after time `t`: e^(tX).
pub fn flow(x: M2, t: f64) -> M2 {
    mexp(mscale(x, t))
}

/// The identity's logarithm that carries every point once around an ellipse
/// with semi-axes `a`, `b`, tilted by `tilt` radians: 2π P J P⁻¹ with
/// P = R(tilt) diag(a, b).
pub fn ellipse_generator(a: f64, b: f64, tilt: f64) -> M2 {
    let p = mmul(rotation(tilt), [[a, 0.0], [0.0, b]]);
    mscale(mmul(mmul(p, J), inverse(p)), TAU)
}

/// The rotation-and-scale matrix of a complex number: a + bi ↦ aI + bJ.
pub fn as_matrix(z: C) -> M2 {
    [[z.re, -z.im], [z.im, z.re]]
}

/// The real matrices X with XA = AX, for a diagonal A = diag(p, q) with
/// p ≠ q, have zero off-diagonal entries: XA − AX = [[0, (q−p)x01],
/// [(p−q)x10, 0]]. Returns that commutator for a test.
pub fn commutator(x: M2, a: M2) -> M2 {
    madd(mmul(x, a), mscale(mmul(a, x), -1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() < tol
    }

    #[test]
    fn the_ruler_adds_where_numbers_multiply() {
        for (a, b) in [(2.5, 3.2), (4.0, 6.0), (0.5, 2.0), (3.0, 3.0)] {
            assert!(close(
                position(a * b, 10.0),
                position(a, 10.0) + position(b, 10.0),
                1e-12
            ));
            assert!(close(position(a * b, 2.0), position(a, 2.0) + position(b, 2.0), 1e-12));
        }
        assert!(close(position(2f64.sqrt(), 2.0), 0.5, 1e-12));
        assert!(close(position(0.5, 2.0), -1.0, 1e-12));
        // a change of base is a stretch: log_b x = ln x / ln b
        assert!(close(position(3.0, 2.0), 3f64.ln() / 2f64.ln(), 1e-12));
    }

    #[test]
    fn the_decade_fold_keeps_digits_and_counts_laps() {
        for (x, digits, laps) in [(0.24, 2.4, -1), (2.4, 2.4, 0), (24.0, 2.4, 1), (240.0, 2.4, 2)] {
            let (d, l) = fold_decade(x);
            assert!(close(d, digits, 1e-12), "{x}");
            assert_eq!(l, laps, "{x}");
        }
        // exact powers of ten sit at the top of the dial on their own lap
        assert_eq!(fold_decade(10.0), (1.0, 1));
        assert_eq!(fold_decade(1.0), (1.0, 0));
    }

    #[test]
    fn principal_angles_live_in_minus_pi_to_pi() {
        assert!(close(wrap(1.5 * PI), -0.5 * PI, 1e-12));
        assert!(close(wrap(PI), PI, 1e-12));
        assert!(close(wrap(-PI), PI, 1e-12));
        assert!(close(wrap(5.0 * TAU + 0.25), 0.25, 1e-9));
        assert_eq!(principal_units(6, 4), -2);
        assert_eq!(principal_units(4, 4), 4);
        assert_eq!(principal_units(-4, 4), 4);
        assert_eq!(principal_units(3, 4), 3);
        assert!(close(C::new(-1.0, 0.0).arg(), PI, 1e-15));
    }

    #[test]
    fn every_lap_of_a_logarithm_exponentiates_to_the_same_number() {
        let z = C::polar(2.0, 0.6 * PI);
        for k in -3..=3 {
            let back = z.log_on_lap(k).exp();
            assert!(back.sub(z).abs() < 1e-12, "lap {k}");
        }
        // the column over -1: odd multiples of pi times i
        for m in [-3.0, -1.0, 1.0, 3.0] {
            assert!(C::new(0.0, m * PI).exp().add(C::new(1.0, 0.0)).abs() < 1e-12);
        }
    }

    #[test]
    fn carrying_the_logarithm_once_around_zero_climbs_one_lap() {
        let unit = circle_path(C::new(0.0, 0.0), 1.0, 1.0, 720);
        let l = lift(&unit);
        assert!(close(l[180].im, PI / 2.0, 1e-9));
        assert!(close(l[360].im, PI, 1e-9));
        assert!(close(l[720].im, TAU, 1e-9));
        assert_eq!(winding(&unit), 1);
        assert_eq!(winding(&circle_path(C::new(0.0, 0.0), 0.5, -2.0, 1440)), -2);
        let side = circle_path(C::new(1.7, 0.0), 0.8, 1.0, 720);
        assert_eq!(winding(&side), 0);
        let ls = lift(&side);
        assert!(close(ls[0].im, ls[720].im, 1e-9), "an off-centre loop returns");
    }

    #[test]
    fn the_product_rule_drops_a_lap_past_the_window() {
        let z = C::polar(1.0, 0.75 * PI);
        let sum = z.log().add(z.log());
        let product = z.mul(z).log();
        assert!(close(sum.im, 1.5 * PI, 1e-12));
        assert!(close(product.im, -0.5 * PI, 1e-12));
        assert!(close(sum.im - product.im, TAU, 1e-12));
        // inside the window the rule holds with no lap
        let w = C::polar(1.0, 0.25 * PI);
        assert!(close(w.mul(w).log().im, w.log().im * 2.0, 1e-12));
    }

    #[test]
    fn the_square_root_puzzle_breaks_at_combining_the_roots() {
        let minus_one = C::new(-1.0, 0.0);
        let i = minus_one.sqrt();
        assert!(i.sub(C::new(0.0, 1.0)).abs() < 1e-12);
        assert!(i.mul(i).sub(minus_one).abs() < 1e-12);
        let combined = minus_one.mul(minus_one).sqrt();
        assert!(combined.sub(C::new(1.0, 0.0)).abs() < 1e-12);
    }

    #[test]
    fn the_series_exponential_matches_rotations_and_determinants() {
        for theta in [0.3, PI / 2.0, PI, 2.5 * PI] {
            assert!(mdist(mexp(mscale(J, theta)), rotation(theta)) < 1e-12);
        }
        // det(e^X) = e^(tr X) for a real X
        let x = [[0.3, -1.0], [0.8, -0.1]];
        for t in [0.5, 1.0, 3.0] {
            assert!(close(det(flow(x, t)), (t * trace(x)).exp(), 1e-10));
        }
        // complex numbers and their matrices exponentiate alike
        let z = C::new(0.4, 1.3);
        assert!(mdist(mexp(as_matrix(z)), as_matrix(z.exp())) < 1e-12);
    }

    #[test]
    fn three_speeds_reach_the_same_quarter_turn() {
        let quarter = rotation(PI / 2.0);
        for turns in [0.25, 1.25, -0.75] {
            assert!(mdist(mexp(mscale(J, TAU * turns)), quarter) < 1e-12, "{turns}");
        }
    }

    #[test]
    fn ellipse_flows_bring_the_plane_home() {
        for (a, b, tilt) in [(2.0, 1.0, 0.0), (1.6, 0.7, PI / 6.0), (3.0, 0.4, 1.0)] {
            let x = ellipse_generator(a, b, tilt);
            assert!(mdist(mexp(x), I2) < 1e-9, "{a} {b} {tilt}");
            // and it is not the zero velocity, nor a plain rotation, when a ≠ b
            assert!(mdist(x, mscale(J, TAU)) > 0.1);
            // halfway, every point is at its own opposite: e^(X/2) = −I
            assert!(mdist(flow(x, 0.5), mscale(I2, -1.0)) < 1e-9);
        }
    }

    #[test]
    fn flows_never_reach_zero_area() {
        let x = [[0.3, -1.0], [0.8, -0.1]];
        for k in 0..=300 {
            let t = 3.0 * f64::from(k) / 300.0;
            assert!(det(flow(x, t)) > 0.0);
        }
    }

    #[test]
    fn whatever_commutes_with_the_flip_pair_keeps_its_axes() {
        let a = [[-2.0, 0.0], [0.0, -0.5]];
        // the commutator is linear in X; on the basis it reads off the two
        // off-diagonal entries scaled by ±(q − p) = ±1.5, so XA = AX forces them to 0
        let e01 = [[0.0, 1.0], [0.0, 0.0]];
        let e10 = [[0.0, 0.0], [1.0, 0.0]];
        assert!(close(commutator(e01, a)[0][1], 1.5, 1e-12));
        assert!(close(commutator(e10, a)[1][0], -1.5, 1e-12));
        assert!(mdist(commutator([[0.7, 0.0], [0.0, -3.0]], a), [[0.0; 2]; 2]) < 1e-15);
        // and a real diagonal X exponentiates to positive diagonal entries only
        for (p, q) in [(0.7, -3.0), (-5.0, 2.0), (0.0, 0.0)] {
            let e = mexp([[p, 0.0], [0.0, q]]);
            assert!(e[0][0] > 0.0 && e[1][1] > 0.0);
        }
        // the half turn has a real logarithm, the plain rotation by pi
        assert!(mdist(mexp(mscale(J, PI)), mscale(I2, -1.0)) < 1e-12);
    }
}
