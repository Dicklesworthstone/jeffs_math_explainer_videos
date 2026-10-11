//! The law of the catch and the careful range (chapter 12). For a lake of
//! N fish with M marked and a second catch of n, `Law` is P_N(K = k) for
//! every possible caught-twice count k (the hypergeometric law, computed in
//! logs by the ratio of neighbours and normalised, as the oracle's
//! `ch12_coverage_over_N` does). A catch k lands in a lake's middle 95%
//! when neither tail beyond it is under 2.5%; the careful range for a
//! catch is every lake size where it does (the exact 95% range by test
//! inversion, oracle `exact_ci`).
//!
//! Floats decide every tail comparison here, so each decision must clear
//! 2.5% by more than float error: a tail within `GUARD` of it is refused
//! (an error), never decided by rounding. The oracle decides the same
//! comparisons in exact integers.

/// The tail cut: 2.5% on each side.
pub const ALPHA: f64 = 1.0 / 40.0;
/// Feller's own range for his catch, recorded (1st ed. 1950, p. 38: "These
/// figures would justify a bet that the true number n of fish lies
/// somewhere between 8500 and 12,000").
pub const FELLER_BET: (u64, u64) = (8_500, 12_000);

/// The one-sided chances behind a range for a catch: P(K ≤ k) at its low
/// end and P(K ≥ k) at its high end (for Feller's bet, oracle A-F14).
pub fn range_tails(marked: u64, caught: u64, caught_twice: u64, (lo, hi): (u64, u64)) -> (f64, f64) {
    (
        Law::new(lo, marked, caught).le(caught_twice),
        Law::new(hi, marked, caught).ge(caught_twice),
    )
}
/// Closer to the cut than this and a float comparison is not trusted.
pub const GUARD: f64 = 1e-9;

/// P_N(K = k) over the possible k, from `first` up.
#[derive(Clone, Debug)]
pub struct Law {
    pub lake: u64,
    pub first: u64,
    pub p: Vec<f64>,
    /// P(K ≤ first + i) and P(K ≥ first + i), summed from each tail inward.
    below: Vec<f64>,
    above: Vec<f64>,
}

impl Law {
    /// The law of the caught-twice count for lake size `lake`, `marked`
    /// marked and a second catch of `caught`.
    pub fn new(lake: u64, marked: u64, caught: u64) -> Self {
        assert!(marked <= lake && caught <= lake, "catches larger than the lake");
        let first = (marked + caught).saturating_sub(lake);
        let last = marked.min(caught);
        // log P(k + 1) − log P(k) = log((M − k)(n − k)) − log((k + 1)(N − M − n + k + 1))
        let mut lp = Vec::with_capacity((last - first + 1) as usize);
        let mut acc = 0.0f64;
        lp.push(acc);
        for k in first..last {
            let (m, n, k, big) = (marked as f64, caught as f64, k as f64, lake as f64);
            acc += ((m - k) * (n - k)).ln() - ((k + 1.0) * (big - m - n + k + 1.0)).ln();
            lp.push(acc);
        }
        let top = lp.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let mut p: Vec<f64> = lp.iter().map(|&l| (l - top).exp()).collect();
        let total: f64 = p.iter().sum();
        for x in &mut p {
            *x /= total;
        }
        let below: Vec<f64> = p
            .iter()
            .scan(0.0, |acc, &x| {
                *acc += x;
                Some(*acc)
            })
            .collect();
        let mut above: Vec<f64> = p
            .iter()
            .rev()
            .scan(0.0, |acc, &x| {
                *acc += x;
                Some(*acc)
            })
            .collect();
        above.reverse();
        Self {
            lake,
            first,
            p,
            below,
            above,
        }
    }

    pub fn last(&self) -> u64 {
        self.first + self.p.len() as u64 - 1
    }

    /// P(K = k).
    pub fn at(&self, k: u64) -> f64 {
        if k < self.first || k > self.last() {
            0.0
        } else {
            self.p[(k - self.first) as usize]
        }
    }

    /// P(K ≤ k).
    pub fn le(&self, k: u64) -> f64 {
        if k < self.first {
            return 0.0;
        }
        self.below[((k - self.first) as usize).min(self.p.len() - 1)]
    }

    /// P(K ≥ k).
    pub fn ge(&self, k: u64) -> f64 {
        if k > self.last() {
            return 0.0;
        }
        self.above[k.saturating_sub(self.first) as usize]
    }

    /// Whether catch `k` lands in this lake's middle 95%: both tails from
    /// it hold more than 2.5%.
    pub fn keeps(&self, k: u64) -> Result<bool, String> {
        let (le, ge) = (self.le(k), self.ge(k));
        for (side, tail) in [("P(K ≤ k)", le), ("P(K ≥ k)", ge)] {
            if (tail - ALPHA).abs() < GUARD {
                return Err(format!(
                    "lake {}, catch {k}: {side} = {tail} is within {GUARD} of 2.5%; decide it exactly",
                    self.lake
                ));
            }
        }
        Ok(le > ALPHA && ge > ALPHA)
    }

    /// The catches this lake keeps: an interval [lo, hi] of k.
    pub fn middle(&self) -> Result<(u64, u64), String> {
        let mut kept = Vec::new();
        for k in self.first..=self.last() {
            if self.keeps(k)? {
                kept.push(k);
            }
        }
        let (lo, hi) = (kept[0], kept[kept.len() - 1]);
        if kept.len() as u64 != hi - lo + 1 {
            return Err(format!("lake {}: the kept catches are not an interval", self.lake));
        }
        Ok((lo, hi))
    }

    /// The chance the catch lands in the middle 95%.
    pub fn middle_mass(&self) -> Result<f64, String> {
        let (lo, hi) = self.middle()?;
        Ok((lo..=hi).map(|k| self.at(k)).sum())
    }

    /// The caught-twice counts behind the soup guess's 2.5% and 97.5%
    /// points in this lake (given someone is caught twice; oracle
    /// `plain_estimate_law`): walking k down from the largest, the first k
    /// whose share of the runs at or above it reaches 2.5%, then 97.5%.
    pub fn estimate_quantiles(&self) -> Result<(u64, u64), String> {
        let any = 1.0 - self.at(0);
        let mut cum = 0.0;
        let (mut q025, mut q975) = (None, None);
        for k in (self.first.max(1)..=self.last()).rev() {
            cum += self.at(k);
            for (cut, slot) in [(ALPHA, &mut q025), (1.0 - ALPHA, &mut q975)] {
                if slot.is_none() {
                    let share = cum / any;
                    if (share - cut).abs() < GUARD {
                        return Err(format!("lake {}: a quantile sits within {GUARD} of its cut", self.lake));
                    }
                    if share >= cut {
                        *slot = Some(k);
                    }
                }
            }
        }
        q025.zip(q975)
            .ok_or_else(|| format!("lake {}: no quantiles", self.lake))
    }
}

/// The careful range for a catch: every lake size whose middle 95% keeps
/// `caught_twice`. P_N(K ≤ k) only rises with N and P_N(K ≥ k) only falls,
/// so each end is found by bisection (as the oracle's `exact_ci`) and then
/// checked on both sides.
pub fn careful_range(marked: u64, caught: u64, caught_twice: u64) -> Result<(u64, u64), String> {
    assert!(caught_twice > 0, "no range from nobody caught twice");
    let fewest = marked + caught - caught_twice;
    let le_ok = |lake: u64| -> Result<bool, String> {
        let tail = Law::new(lake, marked, caught).le(caught_twice);
        guarded(lake, tail)
    };
    let ge_ok = |lake: u64| -> Result<bool, String> {
        let tail = Law::new(lake, marked, caught).ge(caught_twice);
        guarded(lake, tail)
    };
    // lower end: the smallest lake where the catch is not too few
    let lo = if le_ok(fewest)? {
        fewest
    } else {
        let (mut bad, mut good) = (fewest, fewest + 1);
        while !le_ok(good)? {
            bad = good;
            good *= 2;
        }
        while good - bad > 1 {
            let mid = (good + bad) / 2;
            if le_ok(mid)? {
                good = mid;
            } else {
                bad = mid;
            }
        }
        good
    };
    // upper end: the largest lake where the catch is not too many
    let mut good = fewest.max(marked * caught / caught_twice);
    if !ge_ok(good)? {
        return Err(format!("catch {caught_twice}: the soup guess's lake rejects it"));
    }
    let mut bad = good * 2;
    while ge_ok(bad)? {
        good = bad;
        bad *= 2;
    }
    while bad - good > 1 {
        let mid = (good + bad) / 2;
        if ge_ok(mid)? {
            good = mid;
        } else {
            bad = mid;
        }
    }
    let hi = good;
    let edges = (lo == fewest || !le_ok(lo - 1)?) && le_ok(lo)? && ge_ok(hi)? && !ge_ok(hi + 1)?;
    if !edges {
        return Err(format!("catch {caught_twice}: the range's ends do not check"));
    }
    Ok((lo, hi))
}

fn guarded(lake: u64, tail: f64) -> Result<bool, String> {
    if (tail - ALPHA).abs() < GUARD {
        return Err(format!("lake {lake}: a tail of {tail} is too close to 2.5% for floats"));
    }
    Ok(tail > ALPHA)
}

#[cfg(test)]
mod tests {
    use super::*;
    use explainer_kit::choose;

    /// P_N(K = k) exactly, as a float, for small lakes.
    fn exact(lake: u64, m: u64, n: u64, k: u64) -> f64 {
        if k > m || n < k || n - k > lake - m {
            return 0.0;
        }
        (choose(m, k) * choose(lake - m, n - k)) as f64 / choose(lake, n) as f64
    }

    #[test]
    fn the_law_is_the_hypergeometric_on_small_lakes() {
        for lake in 1..=30u64 {
            for m in 0..=lake {
                for n in 0..=lake {
                    let law = Law::new(lake, m, n);
                    for k in 0..=lake {
                        assert!(
                            (law.at(k) - exact(lake, m, n, k)).abs() < 1e-12,
                            "N {lake} M {m} n {n} k {k}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn fellers_careful_range_is_the_oracles() {
        assert_eq!(
            careful_range(1_000, 1_000, 100),
            Ok((8_400, 12_073)),
            "A-F13.ci95_lo/hi"
        );
        assert_eq!(careful_range(30, 24, 12), Ok((47, 94)), "A-F24.ci95_lo/hi");
        assert_eq!(
            careful_range(500, 500, 25),
            Ok((6_936, 15_210)),
            "A-CH12.slider.k25.ci95"
        );
        assert_eq!(
            careful_range(2_000, 2_000, 400),
            Ok((9_254, 10_851)),
            "A-CH12.slider.k400.ci95"
        );
    }

    #[test]
    fn the_true_lakes_middle_is_the_ringing_catches() {
        let law = Law::new(10_000, 1_000, 1_000);
        assert_eq!(law.middle(), Ok((83, 118)), "A-CH12.ring_toss.ringing_k");
        let mass = law.middle_mass().unwrap();
        assert!(
            (mass - 0.954_528_062_358_332).abs() < 1e-12,
            "A-CH12.ring_toss.exact_coverage_N10000"
        );
        // a range rings the true lake exactly when its catch is in that middle
        for k in [82, 83, 100, 118, 119] {
            let (lo, hi) = careful_range(1_000, 1_000, k).unwrap();
            assert_eq!((lo..=hi).contains(&10_000), (83..=118).contains(&k), "catch {k}");
        }
    }

    #[test]
    fn catching_the_whole_lake_leaves_one_size() {
        // M = n = N = 10,000: a lake of 10,001 gives that catch with chance 1/10,001
        let law = Law::new(10_001, 10_000, 10_000);
        assert!((law.ge(10_000) - 1.0 / 10_001.0).abs() < 1e-15);
        assert_eq!(careful_range(10_000, 10_000, 10_000), Ok((10_000, 10_000)));
    }
}
