//! What chapter 09 shows of the 09_6 code panel (oracle `ch09_panel`, keys
//! `A-CH09.panel.*`), each number computed the way the panel's Python
//! computes it, so the facts check can compare them string for string.
//!
//! The panel (numpy 2.5):
//!
//! ```text
//! k = np.array([marked[rng.choice(N, n, replace=False)].sum() for _ in range(10_000)])
//! est = M * n / k
//! p = [comb(M, j) * comb(N - M, n - j) / comb(N, n) for j in range(n + 1)]
//! print(est.mean(), sum(M * n / j * p[j] for j in range(1, n + 1)))
//! ```
//!
//! The experiments themselves are [`crate::runs::panel`] (numpy's stream,
//! draw for draw). Here: numpy's pairwise summation (so `mean` and `std`),
//! `percentile`, and the last line, whose p[j] are Python's correctly
//! rounded `int / int` and whose sum is CPython's compensated `sum()`.

use crate::exact::{Dd, hypergeometric_pmf};
use crate::runs::{self, CAUGHT, LAKE, MARKED};

/// The panel's experiments.
pub struct Panel {
    /// Fish caught twice in each experiment, in run order.
    pub k: Vec<u32>,
    /// Each experiment's soup guess, `M * n / k`.
    pub est: Vec<f64>,
}

/// Run the panel's 10,000 experiments (well under a second).
pub fn run() -> Panel {
    let k = runs::panel();
    let mn = (MARKED * CAUGHT) as f64;
    let est = k.iter().map(|&k| mn / f64::from(k)).collect();
    Panel { k, est }
}

/// What the chapter shows and says, each number as numpy computes it.
pub struct Stats {
    pub k_mean: f64,
    pub k_sd: f64,
    pub k_min: u32,
    pub k_max: u32,
    /// Runs with 91 ≤ k ≤ 109 ("most land within about nine of it").
    pub k_within_9_share: f64,
    pub mean: f64,
    pub median: f64,
    pub p2_5: f64,
    pub p97_5: f64,
    /// `est.std(ddof=1) / sqrt(runs)`.
    pub standard_error: f64,
    /// The panel's last line: the exact mean the check compares with.
    pub exact_mean: f64,
    pub gap_in_standard_errors: f64,
}

impl Panel {
    /// The statistics, or why a percentile could not be read exactly.
    pub fn stats(&self) -> Result<Stats, String> {
        let kf: Vec<f64> = self.k.iter().map(|&k| f64::from(k)).collect();
        let mut sorted = self.est.clone();
        sorted.sort_by(f64::total_cmp);
        let (k_min, k_max) = (
            self.k.iter().copied().min().unwrap_or(0),
            self.k.iter().copied().max().unwrap_or(0),
        );
        let within = self.k.iter().filter(|&&k| (91..=109).contains(&k)).count();
        let est_mean = mean(&self.est);
        let standard_error = std_ddof1(&self.est) / (self.est.len() as f64).sqrt();
        let exact_mean = exact_line_mean();
        let tie = |q: u64| {
            percentile_at_a_tie(&sorted, q).ok_or_else(|| {
                format!(
                    "the panel's {}% point falls between different estimates",
                    q as f64 / 10.0
                )
            })
        };
        Ok(Stats {
            k_mean: mean(&kf),
            k_sd: std(&kf),
            k_min,
            k_max,
            k_within_9_share: within as f64 / self.k.len() as f64,
            mean: est_mean,
            median: runs::median(&self.est),
            p2_5: tie(25)?,
            p97_5: tie(975)?,
            standard_error,
            exact_mean,
            gap_in_standard_errors: (est_mean - exact_mean) / standard_error,
        })
    }
}

/// numpy's pairwise summation of a contiguous float64 array (what
/// `np.add.reduce`, so `mean` and `std`, run on one block).
pub fn pairwise_sum(a: &[f64]) -> f64 {
    let n = a.len();
    if n < 8 {
        a.iter().fold(0.0, |s, &x| s + x)
    } else if n <= 128 {
        let mut r = [0.0; 8];
        r.copy_from_slice(&a[..8]);
        let whole = n - n % 8;
        let (chunks, _) = a[8..whole].as_chunks::<8>();
        for chunk in chunks {
            for (acc, &x) in r.iter_mut().zip(chunk) {
                *acc += x;
            }
        }
        let mut res = ((r[0] + r[1]) + (r[2] + r[3])) + ((r[4] + r[5]) + (r[6] + r[7]));
        for &x in &a[whole..] {
            res += x;
        }
        res
    } else {
        let mut n2 = n / 2;
        n2 -= n2 % 8;
        pairwise_sum(&a[..n2]) + pairwise_sum(&a[n2..])
    }
}

/// `np.mean`.
pub fn mean(a: &[f64]) -> f64 {
    pairwise_sum(a) / a.len() as f64
}

/// `np.std` with `ddof` degrees of freedom removed.
fn std_with(a: &[f64], ddof: usize) -> f64 {
    let m = mean(a);
    let sq: Vec<f64> = a.iter().map(|&x| (x - m) * (x - m)).collect();
    (pairwise_sum(&sq) / (a.len() - ddof) as f64).sqrt()
}

/// `np.std` (ddof = 0).
pub fn std(a: &[f64]) -> f64 {
    std_with(a, 0)
}

/// `np.std(ddof=1)`.
pub fn std_ddof1(a: &[f64]) -> f64 {
    std_with(a, 1)
}

/// `np.percentile(a, q)` (linear) where the two order statistics around
/// the q-th position are equal, as at the panel's 2.5% and 97.5% points:
/// then every interpolation gives that value. `None` otherwise (numpy's
/// interpolation arithmetic is not ported). `q_tenths` is q in tenths of a
/// percent.
pub fn percentile_at_a_tie(sorted: &[f64], q_tenths: u64) -> Option<f64> {
    // the position q(n − 1)/100, exactly
    let n1 = (sorted.len() - 1) as u64;
    let lo = (q_tenths * n1 / 1_000) as usize;
    let hi = if (q_tenths * n1).is_multiple_of(1_000) {
        lo
    } else {
        lo + 1
    };
    (sorted[lo] == sorted[hi]).then_some(sorted[lo])
}

/// CPython's `sum()` of floats (3.12 and later: Neumaier's compensated
/// summation, the compensation added once at the end), started from 0.
pub fn python_sum(xs: impl IntoIterator<Item = f64>) -> f64 {
    let mut it = xs.into_iter();
    let Some(first) = it.next() else {
        return 0.0;
    };
    let (mut f, mut c) = (first, 0.0f64);
    for x in it {
        let t = f + x;
        if f.abs() >= x.abs() {
            c += (f - t) + x;
        } else {
            c += (x - t) + f;
        }
        f = t;
    }
    if c != 0.0 && c.is_finite() { f + c } else { f }
}

/// The panel's last line, `sum(M * n / j * p[j] for j in range(1, n + 1))`:
/// each p[j] is Python's `int / int`, the exact netful count over all
/// netfuls correctly rounded, then the products and CPython's sum.
pub fn exact_line_mean() -> f64 {
    let (m, n, lake) = (MARKED as u64, CAUGHT as u64, LAKE as u64);
    let p: Vec<f64> = hypergeometric_pmf(lake, m, n).into_iter().map(Dd::to_f64).collect();
    python_sum((1..=n).map(|j| (m * n) as f64 / j as f64 * p[j as usize]))
}

/// A polynomial hash of the whole k stream, Σ kᵢ · 31^i mod 2⁶¹ − 1 (the
/// oracle computes the same), so one key pins all 10,000 experiments.
pub fn polyhash(k: &[u32]) -> u64 {
    const P: u128 = (1 << 61) - 1;
    let (mut h, mut w) = (0u128, 1u128);
    for &x in k {
        h = (h + u128::from(x) * w) % P;
        w = w * 31 % P;
    }
    h as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facts::py;

    #[test]
    fn the_panel_prints_its_own_line() {
        // A-CH09.panel.print_line (numpy 2.5.4, CPython 3.14), and the
        // statistics chapter 09 shows, each as the oracle prints it
        let s = run().stats().unwrap();
        assert_eq!(
            format!("{} {}", py(s.mean), py(s.exact_mean)),
            "10088.586031204479 10082.498763370464"
        );
        assert_eq!(py(s.k_sd), "8.960120782668056");
        assert_eq!(py(s.p2_5), "8474.57627118644");
        assert_eq!(py(s.p97_5), "12048.192771084337");
        assert_eq!(py(s.median), "10000.0");
        assert_eq!(py(s.standard_error), "9.212051620287479");
    }

    #[test]
    fn pairwise_summation_follows_numpys_blocks() {
        // below 8: left to right; 8 … 128: eight lanes; above: halves
        let a: Vec<f64> = (0..300).map(|i| 1.0 / (1.0 + i as f64)).collect();
        let lanes = |s: &[f64]| {
            let mut r = [0.0; 8];
            r.copy_from_slice(&s[..8]);
            let whole = s.len() - s.len() % 8;
            for i in (8..whole).step_by(8) {
                for j in 0..8 {
                    r[j] += s[i + j];
                }
            }
            let mut res = ((r[0] + r[1]) + (r[2] + r[3])) + ((r[4] + r[5]) + (r[6] + r[7]));
            for &x in &s[whole..] {
                res += x;
            }
            res
        };
        assert_eq!(pairwise_sum(&a[..5]), (((a[0] + a[1]) + a[2]) + a[3]) + a[4]);
        assert_eq!(pairwise_sum(&a[..100]), lanes(&a[..100]));
        // 300 → 144 + 156 → (72 + 72) + (72 + 84)
        let want = (lanes(&a[..72]) + lanes(&a[72..144])) + (lanes(&a[144..216]) + lanes(&a[216..300]));
        assert_eq!(pairwise_sum(&a), want);
    }

    #[test]
    fn compensated_sum_matches_cpython_on_a_cancelling_series() {
        // CPython ≥ 3.12: sum([1e16, 1.0, -1e16]) == 1.0 (plain addition gives 0.0)
        assert_eq!(python_sum([1e16, 1.0, -1e16]), 1.0);
        assert_eq!(python_sum([0.1; 10]), 1.0);
        assert_eq!(python_sum(std::iter::empty()), 0.0);
    }

    #[test]
    fn percentiles_only_answer_at_ties() {
        let s = [1.0, 2.0, 2.0, 3.0, 4.0];
        // q = 37.5: position 1.5 → between 2.0 and 2.0
        assert_eq!(percentile_at_a_tie(&s, 375), Some(2.0));
        assert_eq!(percentile_at_a_tie(&s, 500), Some(2.0));
        assert_eq!(percentile_at_a_tie(&s, 625), None);
    }

    #[test]
    fn the_hash_sees_every_run_and_its_order() {
        assert_eq!(polyhash(&[]), 0);
        assert_eq!(polyhash(&[1, 2]), 1 + 2 * 31);
        assert_ne!(polyhash(&[1, 2, 3]), polyhash(&[1, 3, 2]));
    }
}
