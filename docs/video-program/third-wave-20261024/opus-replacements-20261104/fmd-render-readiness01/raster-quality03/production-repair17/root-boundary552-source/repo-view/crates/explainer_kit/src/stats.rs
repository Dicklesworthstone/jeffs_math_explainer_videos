//! Topic-agnostic data helpers (ported from the explainer skill's template).
//!
//! All randomness goes through franken_manim's `Pcg64Dxsm`, bit-exact to
//! NumPy's `Generator(PCG64DXSM(seed))` for uniform draws, so a Python oracle
//! regenerates exactly the data a video shows:
//! `Data::new(s).uniform(lo, hi) == lo + (hi - lo) * Generator(PCG64DXSM(s)).random()`.
//! `normal()` is Box–Muller on two uniforms (not NumPy's `standard_normal`);
//! its `ln`/`cos` may differ by an ulp across operating systems.

use fmn::prelude::Pcg64Dxsm;

/// A seeded stream of numbers. One stream per purpose, created in a fixed
/// order: reordering draws changes every later value (and the oracle's).
pub struct Data {
    rng: Pcg64Dxsm,
}

impl Data {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: Pcg64Dxsm::from_seed(seed),
        }
    }

    /// One raw draw (NumPy: `PCG64DXSM(seed).random_raw()`).
    pub fn raw(&mut self) -> u64 {
        self.rng.next_u64()
    }

    /// Uniform in [0, 1) with 53-bit resolution (NumPy: `Generator.random()`).
    pub fn unit(&mut self) -> f64 {
        self.rng.next_f64()
    }

    pub fn uniform(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.unit()
    }

    /// Box–Muller standard normal (two uniform draws).
    pub fn normal(&mut self) -> f64 {
        let u1 = self.unit().max(1e-300);
        let u2 = self.unit();
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }
}

/// Fisher–Yates permutation of 0..n.
pub fn permutation(n: usize, data: &mut Data) -> Vec<usize> {
    let mut p: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        let j = (data.unit() * (i + 1) as f64) as usize;
        p.swap(i, j.min(i));
    }
    p
}

/// 1-based ranks; ties share the average of the places they span.
pub fn ranks(values: &[f64]) -> Vec<f64> {
    let mut order: Vec<usize> = (0..values.len()).collect();
    order.sort_by(|&a, &b| values[a].total_cmp(&values[b]));
    let mut out = vec![0.0; values.len()];
    let mut i = 0;
    while i < order.len() {
        let mut j = i;
        while j + 1 < order.len() && values[order[j + 1]] == values[order[i]] {
            j += 1;
        }
        let avg = (i + j) as f64 / 2.0 + 1.0;
        for &k in &order[i..=j] {
            out[k] = avg;
        }
        i = j + 1;
    }
    out
}

/// Point-cloud shapes in about [-1, 1]². Point `i` sits at parameter
/// t = (i + 0.5) / n on every shape, so one dot keeps its identity when a
/// cloud morphs from shape to shape. Symmetric shapes carry no linear trend,
/// so a correlation honestly reads ≈ 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    Line,
    Monotone,
    Parabola,
    Ring,
    Cross,
    Wave,
    Noise,
}

impl Shape {
    pub fn label(self) -> &'static str {
        match self {
            Shape::Line => "a line",
            Shape::Monotone => "a monotone curve",
            Shape::Parabola => "a parabola",
            Shape::Ring => "a ring",
            Shape::Cross => "an X shape",
            Shape::Wave => "a wave",
            Shape::Noise => "pure noise",
        }
    }
}

/// `n` points of `shape` with Gaussian jitter `jitter` (0.05 is a good start).
pub fn shape_points(shape: Shape, n: usize, seed: u64, jitter: f64) -> Vec<(f64, f64)> {
    let mut d = Data::new(seed);
    (0..n)
        .map(|i| {
            let t = (i as f64 + 0.5) / n as f64; // stable per-dot parameter
            let s = 2.0 * t - 1.0;
            let (x, y) = match shape {
                Shape::Line => (s, 0.9 * s),
                Shape::Monotone => (s, (2.5 * s).tanh() * 0.9),
                Shape::Parabola => (s, 0.9 - 1.6 * s * s),
                Shape::Ring => {
                    let a = std::f64::consts::TAU * t;
                    (0.9 * a.cos(), 0.9 * a.sin())
                }
                Shape::Cross => {
                    if i % 2 == 0 {
                        (s, 0.9 * s)
                    } else {
                        (s, -0.9 * s)
                    }
                }
                // two full cosine periods: symmetric about x = 0, no linear trend
                Shape::Wave => (s, 0.8 * (2.0 * std::f64::consts::TAU * t).cos()),
                Shape::Noise => (d.uniform(-0.95, 0.95), d.uniform(-0.95, 0.95)),
            };
            if shape == Shape::Noise {
                (x, y)
            } else {
                (x + jitter * d.normal(), y + jitter * d.normal())
            }
        })
        .collect()
}

/// The `q` quantile of sorted `values`, nearest rank, float rounding
/// included: identical to NumPy's `np.quantile(v, q, method="inverted_cdf")`
/// (the template checked every n < 600 at q = k/1000, 597k cases). Do not add
/// an epsilon before `ceil()`: at n = 100, q = 0.07 the product is
/// 7.000000000000001 and NumPy answers the 8th value, not the 7th.
///
/// # Panics
/// On an empty slice or `q` outside [0, 1].
pub fn inverted_cdf(sorted: &[f64], q: f64) -> f64 {
    assert!(
        !sorted.is_empty() && (0.0..=1.0).contains(&q),
        "need values and q in [0, 1]"
    );
    let idx = ((sorted.len() as f64 * q).ceil() as usize).clamp(1, sorted.len()) - 1;
    sorted[idx]
}

/// The "noise ceiling" of a statistic on these points: the `q` quantile of
/// |stat| over `trials` shuffles of y (pairing destroyed, marginals kept).
/// Divide the observed |stat| by it to put rival statistics on one scale.
pub fn null_quantile(
    xs: &[f64],
    ys: &[f64],
    stat: impl Fn(&[f64], &[f64]) -> f64,
    trials: usize,
    q: f64,
    seed: u64,
) -> f64 {
    assert!(
        trials >= 2 && (0.0..=1.0).contains(&q),
        "need ≥ 2 trials and q in [0, 1]"
    );
    let mut d = Data::new(seed);
    let mut values: Vec<f64> = (0..trials)
        .map(|_| {
            let p = permutation(ys.len(), &mut d);
            let shuffled: Vec<f64> = p.iter().map(|&k| ys[k]).collect();
            stat(xs, &shuffled).abs()
        })
        .collect();
    values.sort_by(f64::total_cmp);
    inverted_cdf(&values, q)
}

/// Pearson correlation (0 when either side has no variance).
///
/// # Panics
/// Unless the samples are non-empty and of equal length.
pub fn pearson(x: &[f64], y: &[f64]) -> f64 {
    assert!(
        x.len() == y.len() && !x.is_empty(),
        "pearson needs two non-empty samples of equal length"
    );
    let n = x.len() as f64;
    let (mx, my) = (x.iter().sum::<f64>() / n, y.iter().sum::<f64>() / n);
    let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
    for (a, b) in x.iter().zip(y) {
        sxy += (a - mx) * (b - my);
        sxx += (a - mx) * (a - mx);
        syy += (b - my) * (b - my);
    }
    if sxx == 0.0 || syy == 0.0 {
        0.0
    } else {
        sxy / (sxx * syy).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rng_matches_numpy_pcg64dxsm() {
        // numpy 2.5.3: Generator(PCG64DXSM(42)).bit_generator.random_raw() and .random()
        let mut d = Data::new(42);
        assert_eq!(d.raw(), 0xab1c_5033_8e63_481d);
        let mut d = Data::new(42);
        assert_eq!(d.unit().to_bits(), 0x3fe5_638a_0671_cc69);
    }

    #[test]
    fn ranks_average_ties() {
        assert_eq!(ranks(&[3.0, 1.0, 3.0, 2.0]), [3.5, 1.0, 3.5, 2.0]);
        assert_eq!(ranks(&[5.0, 5.0, 5.0]), [2.0, 2.0, 2.0]);
        assert!(ranks(&[]).is_empty());
    }

    #[test]
    fn permutation_is_a_permutation() {
        let mut p = permutation(50, &mut Data::new(7));
        p.sort_unstable();
        assert_eq!(p, (0..50).collect::<Vec<_>>());
    }

    #[test]
    fn quantiles_match_recorded_numpy_inverted_cdf() {
        // recorded with numpy 2.5.3: np.quantile(v, q, method="inverted_cdf")
        let mut v = vec![
            0.11, 0.42, 0.42, 0.07, 0.93, 0.55, 0.31, 0.68, 0.24, 0.79, 0.16, 0.88, 0.5,
        ];
        v.sort_by(f64::total_cmp);
        for (q, want) in [
            (0.0, 0.07),
            (0.01, 0.07),
            (0.1, 0.11),
            (0.25, 0.24),
            (0.3, 0.24),
            (0.5, 0.42),
            (0.7, 0.68),
            (0.9, 0.88),
            (0.95, 0.93),
            (0.99, 0.93),
            (1.0, 0.93),
        ] {
            assert_eq!(inverted_cdf(&v, q), want, "q = {q}");
        }
        // the float traps: n·q lands a hair off an integer
        let w: Vec<f64> = (0..100).map(f64::from).collect();
        for (q, want) in [(0.07, 7.0), (0.29, 28.0), (0.57, 56.0), (0.58, 57.0), (0.99, 98.0)] {
            assert_eq!(inverted_cdf(&w, q), want, "n = 100, q = {q}");
        }
    }

    #[test]
    fn symmetric_shapes_fool_pearson_and_noise_ceiling_is_sane() {
        let pts = shape_points(Shape::Ring, 150, 41, 0.05);
        let (x, y): (Vec<f64>, Vec<f64>) = pts.into_iter().unzip();
        let r = pearson(&x, &y);
        let ceiling = null_quantile(&x, &y, pearson, 300, 0.99, 99);
        assert!(
            r.abs() < ceiling,
            "a ring must read as noise to Pearson: r={r}, ceiling={ceiling}"
        );
        assert!(ceiling > 0.05 && ceiling < 0.5);
    }
}
