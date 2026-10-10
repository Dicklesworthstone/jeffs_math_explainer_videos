//! The statistics the video animates, computed for real (never hard-coded).
//!
//! `hoeffding` follows the reference algorithm in Jeffrey Emanuel's
//! "My Favorite Statistical Measure: Hoeffding's D" (and the
//! `fast_vector_similarity` crate): average ranks, the per-point weighted
//! bivariate count Q, the three sums D1/D2/D3, and Hoeffding's 1948
//! normalization. `self_check` pins it to the article's worked example.

use fmn::prelude::Pcg64Dxsm;

/// Average ranks (1-based), ties sharing the mean of the ranks they span.
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
        // Positions i..=j (0-based) share ranks i+1..=j+1.
        let shared = (i + j) as f64 / 2.0 + 1.0;
        for &k in &order[i..=j] {
            out[k] = shared;
        }
        i = j + 1;
    }
    out
}

/// Everything the formula chapter shows, for one dataset.
#[derive(Debug, Clone)]
pub struct Hoeffding {
    pub r: Vec<f64>,
    pub s: Vec<f64>,
    pub q: Vec<f64>,
    pub d1: f64,
    pub d2: f64,
    pub d3: f64,
    pub d: f64,
}

/// Hoeffding's D with the article's tie-aware Q (N >= 5).
pub fn hoeffding(x: &[f64], y: &[f64]) -> Hoeffding {
    let n = x.len();
    assert!(
        n >= 5 && y.len() == n,
        "Hoeffding's D needs N >= 5 paired samples"
    );
    let r = ranks(x);
    let s = ranks(y);
    let q: Vec<f64> = (0..n)
        .map(|i| {
            let mut qi = 1.0;
            let mut both_eq = 0.0;
            for k in 0..n {
                let (rl, re) = (r[k] < r[i], r[k] == r[i]);
                let (sl, se) = (s[k] < s[i], s[k] == s[i]);
                if rl && sl {
                    qi += 1.0;
                }
                if re && se {
                    both_eq += 1.0;
                }
                if re && sl {
                    qi += 0.5;
                }
                if rl && se {
                    qi += 0.5;
                }
            }
            qi + 0.25 * (both_eq - 1.0)
        })
        .collect();
    let d1: f64 = q.iter().map(|&qi| (qi - 1.0) * (qi - 2.0)).sum();
    let d2: f64 = r
        .iter()
        .zip(&s)
        .map(|(&ri, &si)| (ri - 1.0) * (ri - 2.0) * (si - 1.0) * (si - 2.0))
        .sum();
    let d3: f64 = (0..n)
        .map(|i| (r[i] - 2.0) * (s[i] - 2.0) * (q[i] - 1.0))
        .sum();
    let nf = n as f64;
    let d = 30.0 * ((nf - 2.0) * (nf - 3.0) * d1 + d2 - 2.0 * (nf - 2.0) * d3)
        / (nf * (nf - 1.0) * (nf - 2.0) * (nf - 3.0) * (nf - 4.0));
    Hoeffding {
        r,
        s,
        q,
        d1,
        d2,
        d3,
        d,
    }
}

pub fn pearson(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len() as f64;
    let mx = x.iter().sum::<f64>() / n;
    let my = y.iter().sum::<f64>() / n;
    let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
    for (&a, &b) in x.iter().zip(y) {
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

pub fn spearman(x: &[f64], y: &[f64]) -> f64 {
    pearson(&ranks(x), &ranks(y))
}

/// Kendall's tau-b (tie-corrected), as scipy's default.
pub fn kendall(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len();
    let (mut c, mut d, mut tx, mut ty) = (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64);
    for i in 0..n {
        for j in i + 1..n {
            let a = (x[i] - x[j]).signum() * f64::from(x[i] != x[j]);
            let b = (y[i] - y[j]).signum() * f64::from(y[i] != y[j]);
            if a == 0.0 && b == 0.0 {
            } else if a == 0.0 {
                tx += 1.0;
            } else if b == 0.0 {
                ty += 1.0;
            } else if a * b > 0.0 {
                c += 1.0;
            } else {
                d += 1.0;
            }
        }
    }
    let denom = ((c + d + tx) * (c + d + ty)).sqrt();
    if denom == 0.0 { 0.0 } else { (c - d) / denom }
}

/// The article's worked example: heights and weights of ten people.
pub const HEIGHTS: [f64; 10] = [55.0, 62.0, 68.0, 70.0, 72.0, 65.0, 67.0, 78.0, 78.0, 78.0];
pub const WEIGHTS: [f64; 10] = [
    125.0, 145.0, 160.0, 156.0, 190.0, 150.0, 165.0, 250.0, 250.0, 250.0,
];

/// Pin the implementation to the article (and to scipy, checked offline):
/// a wrong number must never reach a frame.
pub fn self_check() {
    let h = hoeffding(&HEIGHTS, &WEIGHTS);
    assert_eq!(h.r, [1.0, 2.0, 5.0, 6.0, 7.0, 3.0, 4.0, 9.0, 9.0, 9.0]);
    assert_eq!(h.s, [1.0, 2.0, 5.0, 4.0, 7.0, 3.0, 6.0, 9.0, 9.0, 9.0]);
    assert_eq!(h.q, [1.0, 2.0, 4.0, 4.0, 7.0, 3.0, 4.0, 8.5, 8.5, 8.5]);
    assert_eq!((h.d1, h.d2, h.d3), (196.25, 10696.0, 1329.5));
    assert!((h.d - 0.410_714_285_714_285_7).abs() < 1e-15, "D = {}", h.d);
    assert!((pearson(&HEIGHTS, &WEIGHTS) - 0.930_711_074_810_372_7).abs() < 1e-12);
    assert!((spearman(&HEIGHTS, &WEIGHTS) - 0.950_310_559_006_211_3).abs() < 1e-12);
    assert!((kendall(&HEIGHTS, &WEIGHTS) - 0.857_142_857_142_857_1).abs() < 1e-12);
}

/// Seeded data from the engine's one RNG (PCG64DXSM, bit-exact to NumPy).
pub struct Data {
    rng: Pcg64Dxsm,
}

impl Data {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: Pcg64Dxsm::from_seed(seed),
        }
    }

    pub fn uniform(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.rng.next_f64()
    }

    /// Box-Muller standard normal.
    pub fn normal(&mut self) -> f64 {
        let u1 = self.rng.next_f64().max(1e-300);
        let u2 = self.rng.next_f64();
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }
}

/// The shapes the morphing scatter walks through, as unit-square-ish
/// coordinates in [-1, 1]^2; point `i` keeps its identity across shapes so
/// the dots can glide from one to the next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Line,
    Parabola,
    Ring,
    Cross,
    Wave,
    Noise,
    Monotone,
}

impl Shape {
    pub fn label(self) -> &'static str {
        match self {
            Shape::Line => "Line",
            Shape::Parabola => "Parabola",
            Shape::Ring => "Ring",
            Shape::Cross => "Cross",
            Shape::Wave => "Wave",
            Shape::Noise => "Pure noise",
            Shape::Monotone => "Monotone",
        }
    }
}

/// `n` points of `shape`, deterministic per (shape, seed).
pub fn shape_points(shape: Shape, n: usize, seed: u64) -> Vec<(f64, f64)> {
    let mut g = Data::new(seed);
    let jitter = 0.05;
    (0..n)
        .map(|i| {
            let t = (i as f64 + 0.5) / n as f64; // even coverage, stable order
            let (x, y) = match shape {
                Shape::Line => {
                    let x = 2.0 * t - 1.0;
                    (x, 0.85 * x)
                }
                Shape::Parabola => {
                    let x = 2.0 * t - 1.0;
                    (x, 1.7 * x * x - 0.85)
                }
                Shape::Ring => {
                    let a = std::f64::consts::TAU * t;
                    (0.9 * a.cos(), 0.9 * a.sin())
                }
                Shape::Cross => {
                    let x = 2.0 * ((2.0 * t) % 1.0) - 1.0;
                    let sign = if i % 2 == 0 { 1.0 } else { -1.0 };
                    (x, sign * 0.85 * x)
                }
                Shape::Wave => {
                    let x = 2.0 * t - 1.0;
                    // Two full cosine periods: symmetric about x = 0, so it
                    // carries no linear trend for Pearson to latch onto.
                    (x, 0.8 * (2.0 * std::f64::consts::TAU * t).cos())
                }
                Shape::Noise => (g.uniform(-0.95, 0.95), g.uniform(-0.95, 0.95)),
                Shape::Monotone => {
                    let x = 2.0 * t - 1.0;
                    (x, 0.9 * (2.4 * x).tanh() / 2.4_f64.tanh())
                }
            };
            if shape == Shape::Noise {
                (x, y)
            } else {
                (x + jitter * g.normal(), y + jitter * g.normal())
            }
        })
        .collect()
}

/// The four measures for one point set.
#[derive(Debug, Clone, Copy)]
pub struct Measures {
    pub pearson: f64,
    pub spearman: f64,
    pub kendall: f64,
    pub hoeffding: f64,
}

pub fn measures(points: &[(f64, f64)]) -> Measures {
    let x: Vec<f64> = points.iter().map(|p| p.0).collect();
    let y: Vec<f64> = points.iter().map(|p| p.1).collect();
    Measures {
        pearson: pearson(&x, &y),
        spearman: spearman(&x, &y),
        kendall: kendall(&x, &y),
        hoeffding: hoeffding(&x, &y).d,
    }
}

/// n choose k as f64 (exact enough for the 2.6e13 headline).
pub fn choose(n: u64, k: u64) -> f64 {
    (0..k).fold(1.0, |acc, i| acc * (n - i) as f64 / (i + 1) as f64)
}

/// Fisher-Yates permutation of 0..n from the engine RNG.
pub fn permutation(n: usize, data: &mut Data) -> Vec<usize> {
    let mut p: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        let j = (data.uniform(0.0, 1.0) * (i + 1) as f64) as usize;
        p.swap(i, j.min(i));
    }
    p
}

/// The 99th percentile of |measure| over `trials` shuffles of y: what
/// "pure coincidence" looks like for this N (a permutation test).
pub fn null_q99(points: &[(f64, f64)], trials: usize, seed: u64) -> Measures {
    assert!(trials >= 2, "a noise ceiling needs at least 2 shuffles");
    let mut data = Data::new(seed);
    let x: Vec<f64> = points.iter().map(|p| p.0).collect();
    let y: Vec<f64> = points.iter().map(|p| p.1).collect();
    let mut cols: [Vec<f64>; 4] = Default::default();
    for _ in 0..trials {
        let perm = permutation(y.len(), &mut data);
        let ys: Vec<f64> = perm.iter().map(|&k| y[k]).collect();
        cols[0].push(pearson(&x, &ys).abs());
        cols[1].push(spearman(&x, &ys).abs());
        cols[2].push(kendall(&x, &ys).abs());
        cols[3].push(hoeffding(&x, &ys).d.abs());
    }
    let q = |c: &mut Vec<f64>| {
        c.sort_by(f64::total_cmp);
        c[((c.len() as f64) * 0.99) as usize - 1]
    };
    Measures {
        pearson: q(&mut cols[0]),
        spearman: q(&mut cols[1]),
        kendall: q(&mut cols[2]),
        hoeffding: q(&mut cols[3]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chapters::gallery;

    /// Seeded tie-free samples (uniform doubles never collide in practice).
    fn sample(n: usize, seed: u64) -> (Vec<f64>, Vec<f64>) {
        let mut g = Data::new(seed);
        let x: Vec<f64> = (0..n).map(|_| g.uniform(-1.0, 1.0)).collect();
        let y: Vec<f64> = x.iter().map(|&a| a * a + 0.3 * g.normal()).collect();
        (x, y)
    }

    /// Seeded samples on a coarse grid, so ties (and exact twins) are common.
    fn tied_sample(n: usize, seed: u64) -> (Vec<f64>, Vec<f64>) {
        let mut g = Data::new(seed);
        let x: Vec<f64> = (0..n).map(|_| (g.uniform(0.0, 5.0)).floor()).collect();
        let y: Vec<f64> = x
            .iter()
            .map(|&a| (a + g.uniform(0.0, 3.0)).floor())
            .collect();
        (x, y)
    }

    /// Hoeffding's (1948) definition, by brute force: the mean of the order-5
    /// kernel over all ordered 5-tuples of distinct points, times the SAS
    /// factor 30. Independent of ranks, Q and the D1/D2/D3 bookkeeping.
    fn u_statistic_d(x: &[f64], y: &[f64]) -> f64 {
        let psi = |a: f64, b: f64, c: f64| f64::from(a >= b) - f64::from(a >= c);
        let n = x.len();
        let mut total = 0.0;
        for a in 0..n {
            for b in 0..n {
                for c in 0..n {
                    for d in 0..n {
                        for e in 0..n {
                            let idx = [a, b, c, d, e];
                            if (0..5).any(|i| idx[i + 1..].contains(&idx[i])) {
                                continue;
                            }
                            total += psi(x[a], x[b], x[c])
                                * psi(x[a], x[d], x[e])
                                * psi(y[a], y[b], y[c])
                                * psi(y[a], y[d], y[e]);
                        }
                    }
                }
            }
        }
        let tuples: f64 = (0..5).map(|i| (n - i) as f64).product();
        30.0 * total / 4.0 / tuples
    }

    #[test]
    fn ranks_average_ties() {
        assert_eq!(ranks(&[3.0, 1.0, 3.0, 2.0]), [3.5, 1.0, 3.5, 2.0]);
        assert_eq!(ranks(&[7.0, 7.0, 7.0]), [2.0, 2.0, 2.0]);
        assert_eq!(ranks(&[4.0]), [1.0]);
        assert!(ranks(&[]).is_empty());
    }

    #[test]
    fn worked_example_matches_the_article_and_the_on_screen_fraction() {
        self_check();
        // Chapter 6 types this substitution into its TeX:
        // 30 * (8*7*196.25 + 10696 - 2*8*1329.5) / (10*9*8*7*6) = 30 * 414/30240.
        let h = hoeffding(&HEIGHTS, &WEIGHTS);
        let n = HEIGHTS.len() as f64;
        assert_eq!(n, 10.0);
        let numerator = (n - 2.0) * (n - 3.0) * h.d1 + h.d2 - 2.0 * (n - 2.0) * h.d3;
        let denominator = n * (n - 1.0) * (n - 2.0) * (n - 3.0) * (n - 4.0);
        assert_eq!((numerator, denominator), (414.0, 30240.0));
        assert_eq!(h.d, 30.0 * 414.0 / 30240.0);
    }

    #[test]
    fn rank_formula_equals_hoeffdings_u_statistic_without_ties() {
        for (n, seed) in [(5, 1), (6, 2), (7, 3), (8, 4), (9, 5), (9, 6)] {
            let (x, y) = sample(n, seed);
            let fast = hoeffding(&x, &y).d;
            let brute = u_statistic_d(&x, &y);
            assert!(
                (fast - brute).abs() < 1e-12,
                "n={n} seed={seed}: rank formula {fast} vs U-statistic {brute}"
            );
        }
    }

    #[test]
    fn d_is_invariant_under_strictly_monotone_and_reflecting_transforms() {
        for (x, y) in [sample(60, 11), tied_sample(60, 12)] {
            let d = hoeffding(&x, &y).d;
            let fx: Vec<f64> = x.iter().map(|&a| (3.0 * a).exp()).collect();
            let gy: Vec<f64> = y.iter().map(|&b| b * b * b + 7.0).collect();
            let neg_x: Vec<f64> = x.iter().map(|&a| -a).collect();
            assert_eq!(hoeffding(&fx, &gy).d, d, "increasing transforms");
            assert_eq!(hoeffding(&neg_x, &y).d, d, "reflecting X");
        }
    }

    #[test]
    fn d_is_symmetric_and_ignores_the_order_of_the_pairs() {
        for (x, y) in [sample(50, 21), tied_sample(50, 22)] {
            let d = hoeffding(&x, &y).d;
            assert_eq!(hoeffding(&y, &x).d, d, "swapping X and Y");
            let perm = permutation(x.len(), &mut Data::new(23));
            let px: Vec<f64> = perm.iter().map(|&k| x[k]).collect();
            let py: Vec<f64> = perm.iter().map(|&k| y[k]).collect();
            assert_eq!(hoeffding(&px, &py).d, d, "reordering the pairs");
        }
    }

    #[test]
    fn perfect_monotone_dependence_scores_exactly_one() {
        for n in [5, 6, 10, 37] {
            let x: Vec<f64> = (0..n).map(f64::from).collect();
            let up: Vec<f64> = x.iter().map(|&a| a.powi(3)).collect();
            let down: Vec<f64> = x.iter().map(|&a| -a).collect();
            assert_eq!(hoeffding(&x, &up).d, 1.0, "increasing, n={n}");
            assert_eq!(hoeffding(&x, &down).d, 1.0, "decreasing, n={n}");
        }
    }

    /// Every ordering of 0..n (Heap's algorithm).
    fn all_orderings(n: usize) -> Vec<Vec<f64>> {
        fn heap(k: usize, a: &mut Vec<f64>, out: &mut Vec<Vec<f64>>) {
            if k <= 1 {
                out.push(a.clone());
                return;
            }
            for i in 0..k {
                heap(k - 1, a, out);
                a.swap(if k.is_multiple_of(2) { i } else { 0 }, k - 1);
            }
        }
        let mut out = Vec::new();
        heap(n, &mut (0..n).map(|i| i as f64).collect(), &mut out);
        out
    }

    /// Chapter 6 (line f8): "it never drops below minus one half", and one is
    /// a perfectly monotone relationship. Checked over every ordering of
    /// small samples, where the extremes actually occur (-1/2 at N = 5).
    #[test]
    fn d_stays_within_the_narrated_range() {
        for n in [5, 6, 7] {
            let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
            let orderings = all_orderings(n);
            assert_eq!(orderings.len(), (1..=n).product::<usize>());
            let ds: Vec<f64> = orderings.iter().map(|y| hoeffding(&x, y).d).collect();
            let lo = ds.iter().copied().fold(f64::INFINITY, f64::min);
            let hi = ds.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            assert!(lo >= -0.5 && hi == 1.0, "n={n}: D in [{lo}, {hi}]");
            if n == 5 {
                assert_eq!(lo, -0.5, "the bound is attained");
            }
        }
    }

    /// The [-1/2, 1] scale belongs to tie-free data. With heavy ties at small
    /// N the tie-corrected estimator loses it: perfectly dependent two-level
    /// data reads as strongly negative. (The video's data is tie-free apart
    /// from the worked example, whose D is pinned above.)
    #[test]
    fn heavy_ties_at_small_n_leave_the_scale() {
        let x = [0.0, 0.0, 0.0, 1.0, 1.0];
        assert_eq!(hoeffding(&x, &x).d, -1.84375);
    }

    #[test]
    fn d_averages_to_zero_under_independence() {
        // D is an unbiased estimator of a quantity that is 0 under
        // independence; shuffling Y realizes independence exactly.
        let (x, y) = sample(40, 31);
        let mut data = Data::new(32);
        let trials = 400;
        let mean = (0..trials)
            .map(|_| {
                let perm = permutation(y.len(), &mut data);
                let ys: Vec<f64> = perm.iter().map(|&k| y[k]).collect();
                hoeffding(&x, &ys).d
            })
            .sum::<f64>()
            / f64::from(trials);
        assert!(mean.abs() < 0.005, "mean D under shuffles = {mean}");
    }

    #[test]
    fn small_correlation_cases() {
        assert_eq!(kendall(&[1.0, 2.0, 3.0], &[1.0, 3.0, 2.0]), 1.0 / 3.0);
        assert_eq!(pearson(&[1.0, 2.0, 3.0], &[5.0, 5.0, 5.0]), 0.0);
        let x = [1.0, 2.0, 3.0, 4.0, 5.0];
        let cubed = x.map(|a: f64| a.powi(3));
        assert!((spearman(&x, &cubed) - 1.0).abs() < 1e-15);
        assert!(
            pearson(&x, &cubed) < 0.99,
            "Pearson only sees the linear part"
        );
    }

    /// Chapter 2's narration makes a claim about every shape (lines g3-g9).
    /// Each must hold for the exact points and noise ceilings the video uses.
    #[test]
    fn the_gallery_narration_is_true_for_the_rendered_data() {
        let above = |v: f64, ceiling: f64| v.abs() > ceiling;
        for shape in [
            Shape::Line,
            Shape::Parabola,
            Shape::Ring,
            Shape::Cross,
            Shape::Wave,
            Shape::Noise,
        ] {
            let pts = shape_points(shape, gallery::N, gallery::seed(shape));
            let m = measures(&pts);
            let q = null_q99(&pts, gallery::TRIALS, 99);
            let correlations = [
                above(m.pearson, q.pearson),
                above(m.spearman, q.spearman),
                above(m.kendall, q.kendall),
            ];
            let d_above = above(m.hoeffding, q.hoeffding);
            let (want_correlations, want_d) = match shape {
                // "For a straight line, everyone agrees."
                Shape::Line => (true, true),
                // "Pure noise: every measure stays under its ceiling, D included."
                Shape::Noise => (false, false),
                // Parabola, ring, X, wave: "the correlations see nothing.
                // D sees it clearly."
                _ => (false, true),
            };
            assert_eq!(
                (correlations, d_above),
                ([want_correlations; 3], want_d),
                "{}: measures {m:?} vs ceilings {q:?}",
                shape.label()
            );
        }
    }

    #[test]
    fn counting_and_shuffling_helpers() {
        assert_eq!(choose(5000, 4), 26_010_428_123_750.0);
        assert_eq!(choose(10, 0), 1.0);
        for n in [0, 1, 2, 17, 150] {
            let mut p = permutation(n, &mut Data::new(5));
            assert_eq!(p, permutation(n, &mut Data::new(5)), "seeded");
            p.sort_unstable();
            assert_eq!(p, (0..n).collect::<Vec<_>>(), "a bijection on 0..{n}");
        }
        assert_ne!(
            permutation(50, &mut Data::new(5)),
            permutation(50, &mut Data::new(6))
        );
    }

    #[test]
    #[should_panic(expected = "at least 2 shuffles")]
    fn a_noise_ceiling_needs_shuffles() {
        null_q99(&shape_points(Shape::Noise, 10, 1), 1, 1);
    }
}
