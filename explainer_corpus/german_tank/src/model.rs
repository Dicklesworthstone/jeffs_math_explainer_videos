//! The model: boxes numbered 1, 2, …, N with no skips; a handful is k
//! different boxes; in a fair handful every set of k is equally likely (a
//! uniform draw without replacement). A recipe turns a handful into an
//! estimate of N.
//!
//! Everything is exact: rationals over i128. Distributions come from closed
//! forms, never from enumerating handfuls (the oracle enumerates instead):
//! P(largest = m) = C(m − 1, k − 1)/C(N, k); the handfuls with spread s
//! (largest minus smallest) number (N − s)·C(s − 1, k − 2); the handfuls of
//! each sum come from a sum-of-subsets table, and those of each largest
//! number and sum from the same table under each largest. Only the small
//! cases a scene shows whole (the tiny company's pairs, the sufficiency
//! check's sets) are listed one by one.
//!
//! The handfuls the video draws "by computer" come from [`SplitMix64`], one
//! stream per experiment; `oracle.py` implements the same generator
//! separately and must print the same handfuls.

use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};

use explainer_kit::choose;

// ---------------------------------------------------------------- fractions

/// An exact fraction in lowest terms, the denominator positive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Q {
    n: i128,
    d: i128,
}

/// Overflow is a bug in the model (every value here is a small fraction),
/// not a failure.
const OVERFLOW: &str = "a fraction overflowed i128";

fn gcd(a: i128, b: i128) -> i128 {
    let (mut a, mut b) = (a.checked_abs().expect(OVERFLOW), b.checked_abs().expect(OVERFLOW));
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

fn times(a: i128, b: i128) -> i128 {
    a.checked_mul(b).expect(OVERFLOW)
}

fn plus(a: i128, b: i128) -> i128 {
    a.checked_add(b).expect(OVERFLOW)
}

fn negated(a: i128) -> i128 {
    a.checked_neg().expect(OVERFLOW)
}

impl Q {
    pub const ZERO: Q = Q { n: 0, d: 1 };
    pub const ONE: Q = Q { n: 1, d: 1 };

    /// # Panics
    /// If `d` is zero, or a step overflows i128.
    pub fn new(n: i128, d: i128) -> Self {
        assert!(d != 0, "a fraction needs a non-zero denominator");
        let g = gcd(n, d).max(1);
        let s = if d < 0 { -1 } else { 1 };
        Q {
            n: times(s, n) / g,
            d: times(s, d) / g,
        }
    }

    pub const fn int(n: i128) -> Self {
        Q { n, d: 1 }
    }

    /// A count as a fraction.
    pub fn count(n: u128) -> Self {
        Q::int(i128::try_from(n).expect(OVERFLOW))
    }

    /// The whole number this is, if it is one.
    pub fn whole(self) -> Option<i128> {
        (self.d == 1).then_some(self.n)
    }

    pub fn abs(self) -> Q {
        if self.n < 0 { -self } else { self }
    }

    /// The nearest whole number, halves away from zero (what the voice says
    /// for "about").
    pub fn nearest(self) -> i128 {
        let half = Q::new(1, 2);
        let up = |x: Q| {
            let y = x + half;
            y.n.div_euclid(y.d)
        };
        if self.n >= 0 { up(self) } else { -up(-self) }
    }

    /// Exactly one decimal place ("28.6", "71.0"), only when that is exact.
    pub fn decimal1(self) -> Result<String, String> {
        let tenths = self * Q::int(10);
        let t = tenths
            .whole()
            .ok_or_else(|| format!("{self} has no exact one-decimal form"))?;
        let sign = if t < 0 { "-" } else { "" };
        Ok(format!("{sign}{}.{}", t.abs() / 10, t.abs() % 10))
    }

    /// Exactly two decimal places ("4.25"), only when that is exact and the
    /// value is not negative.
    pub fn decimal2(self) -> Result<String, String> {
        let t = (self * Q::int(100))
            .whole()
            .filter(|&t| t >= 0)
            .ok_or_else(|| format!("{self} has no exact positive two-decimal form"))?;
        Ok(format!("{}.{:02}", t / 100, t % 100))
    }

    /// The square root, when this is the square of a fraction (9/4 → 3/2).
    pub fn exact_sqrt(self) -> Option<Q> {
        let root = |x: i128| {
            let r = i128::try_from(u128::try_from(x).ok()?.isqrt()).ok()?;
            (r * r == x).then_some(r)
        };
        Some(Q::new(root(self.n)?, root(self.d)?))
    }

    /// A positive value rounded half up to one decimal place (a panel's
    /// "≈ 166.8").
    pub fn about1(self) -> Result<String, String> {
        if self.n < 0 {
            return Err(format!("{self}: about1 is for positive values"));
        }
        let t = (self * Q::int(10)).nearest();
        Ok(format!("{}.{}", t / 10, t % 10))
    }

    /// The square root of a positive value rounded half up to one decimal
    /// place, found exactly: the tenths t with ((2t − 1)/20)² ≤ self <
    /// ((2t + 1)/20)², from the integer square root of 400 × self (a
    /// panel's "≈ 11.7").
    pub fn sqrt_about1(self) -> Result<String, String> {
        if self.n < 0 {
            return Err(format!("{self}: sqrt_about1 is for positive values"));
        }
        let floor = u128::try_from(times(self.n, 400).div_euclid(self.d)).expect(OVERFLOW);
        let r = floor.isqrt();
        let odd = if r % 2 == 1 { r } else { r.saturating_sub(1) };
        let t = odd.div_ceil(2);
        Ok(format!("{}.{}", t / 10, t % 10))
    }
}

/// `n/d`, or `n` when whole: as Python's `str(Fraction)` prints it.
impl fmt::Display for Q {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.d == 1 {
            write!(f, "{}", self.n)
        } else {
            write!(f, "{}/{}", self.n, self.d)
        }
    }
}

impl Add for Q {
    type Output = Q;
    fn add(self, o: Q) -> Q {
        let g = gcd(self.d, o.d);
        Q::new(
            plus(times(self.n, o.d / g), times(o.n, self.d / g)),
            times(self.d / g, o.d),
        )
    }
}

impl Neg for Q {
    type Output = Q;
    fn neg(self) -> Q {
        Q {
            n: negated(self.n),
            d: self.d,
        }
    }
}

impl Sub for Q {
    type Output = Q;
    fn sub(self, o: Q) -> Q {
        self + -o
    }
}

impl Mul for Q {
    type Output = Q;
    fn mul(self, o: Q) -> Q {
        let (g1, g2) = (gcd(self.n, o.d).max(1), gcd(o.n, self.d).max(1));
        Q::new(times(self.n / g1, o.n / g2), times(self.d / g2, o.d / g1))
    }
}

impl Div for Q {
    type Output = Q;
    /// # Panics
    /// On division by zero.
    fn div(self, o: Q) -> Q {
        assert!(o.n != 0, "division by a zero fraction");
        self * Q::new(o.d, o.n)
    }
}

impl Ord for Q {
    fn cmp(&self, o: &Q) -> Ordering {
        times(self.n, o.d).cmp(&times(o.n, self.d))
    }
}

impl PartialOrd for Q {
    fn partial_cmp(&self, o: &Q) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

/// Values joined by commas, as the facts print a list.
pub fn csv<T: fmt::Display>(values: &[T]) -> String {
    values.iter().map(T::to_string).collect::<Vec<_>>().join(",")
}

/// Handfuls joined by semicolons, each a comma list.
pub fn rows(hands: &[Vec<u64>]) -> String {
    hands.iter().map(|h| csv(h)).collect::<Vec<_>>().join(";")
}

fn c(n: u64, k: u64) -> Q {
    Q::count(choose(n, k))
}

// ------------------------------------------------------------- the example

/// The five friends' numbers, in order.
pub const FRIENDS: [u64; 5] = [14, 27, 33, 51, 60];

/// The sixth friend's number (chapter 16).
pub const SIXTH: u64 = 90;

/// The companies chapter 02 compares, and the one chapter 08 fixes.
pub const BIG: u64 = 1000;
pub const SMALL: u64 = 70;
pub const KNOWN: u64 = 71;

/// The dots of the line as first drawn: every number from 1 to 84.
pub const LINE_DOTS: u64 = 84;

/// The zoomed line's marks divide its extent into this many equal parts.
pub const ZOOM_PARTS: u64 = 4;

/// The marks of a line shown from `first` to `end`: `first`, then every
/// `end / parts` up to but not including `end` (the end gets its own
/// label). Refused unless `parts` divides `end` evenly and the first mark
/// past `first` lies beyond it, so the marks are whole numbers in order.
pub fn marks(first: u64, end: u64, parts: u64) -> Result<Vec<u64>, String> {
    if parts == 0 || !end.is_multiple_of(parts) {
        return Err(format!("{parts} equal parts of a line to {end} are not whole numbers"));
    }
    let step = end / parts;
    if step <= first {
        return Err(format!(
            "a mark every {step} does not lie past the first number, {first}"
        ));
    }
    Ok(std::iter::once(first).chain((1..parts).map(|j| j * step)).collect())
}

// ------------------------------------------------------------------ recipes

/// The unseen numbers below the smallest and between neighbours, smallest
/// first. `hand` must be sorted, different numbers, all at least 1.
pub fn gaps(hand: &[u64]) -> Vec<u64> {
    let mut prev = 0;
    hand.iter()
        .map(|&x| {
            let g = x - prev - 1;
            prev = x;
            g
        })
        .collect()
}

/// The gaps strictly between neighbours (chapter 15).
pub fn inner_gaps(hand: &[u64]) -> Vec<u64> {
    hand.windows(2).map(|w| w[1] - w[0] - 1).collect()
}

/// The gap recipe as a function of the largest number m and the handful
/// size k: m plus the average gap, m + (m − k)/k.
pub fn gap_of(m: u64, k: u64) -> Q {
    Q::int(m as i128) + Q::new(m as i128 - k as i128, k as i128)
}

/// The gap recipe: the largest number plus the average of the gaps below it.
pub fn gap_recipe(hand: &[u64]) -> Q {
    let g = gaps(hand);
    let largest = *hand.last().expect("a handful has a box");
    Q::int(largest as i128) + Q::new(g.iter().sum::<u64>() as i128, g.len() as i128)
}

/// Twice the average, minus one.
pub fn doubling_recipe(hand: &[u64]) -> Q {
    Q::new(2 * hand.iter().sum::<u64>() as i128, hand.len() as i128) - Q::ONE
}

/// The doubling recipe from a handful's sum.
pub fn doubling_of(sum: u64, k: u64) -> Q {
    Q::new(2 * sum as i128, k as i128) - Q::ONE
}

/// With the first number unknown: s(1 + 2/(k − 1)) − 1 for spread s, the
/// largest minus the smallest (Clark, Gonye and Miller 2021, p. 3).
pub fn spread_of(s: u64, k: u64) -> Q {
    Q::int(s as i128) * (Q::ONE + Q::new(2, k as i128 - 1)) - Q::ONE
}

// ---------------------------------------------------------- distributions

/// How many handfuls of k from 1..n have largest number m.
pub fn with_largest(m: u64, k: u64) -> Q {
    if m < k { Q::ZERO } else { c(m - 1, k - 1) }
}

/// The chance that every number of a fair handful of k from 1..n is at most
/// m: C(m, k)/C(n, k).
pub fn all_at_most(m: u64, n: u64, k: u64) -> Q {
    c(m.min(n), k) / c(n, k)
}

/// Average of `f(m)` over the fair handfuls of k from 1..n, weighting each
/// largest number m by how many handfuls it tops.
pub fn mean_by_largest(n: u64, k: u64, f: impl Fn(u64) -> Q) -> Q {
    (k..=n).fold(Q::ZERO, |s, m| s + with_largest(m, k) * f(m)) / c(n, k)
}

/// The share of fair handfuls of k from 1..n whose largest number `keep`s.
pub fn share_by_largest(n: u64, k: u64, keep: impl Fn(u64) -> bool) -> Q {
    (k..=n)
        .filter(|&m| keep(m))
        .fold(Q::ZERO, |s, m| s + with_largest(m, k))
        / c(n, k)
}

/// The top of the one-sided range: the largest count n for which a fair
/// handful of k still tops out at m or less with chance at least `level`.
pub fn upper_bound(m: u64, k: u64, level: Q) -> u64 {
    let mut n = m;
    while all_at_most(m, n + 1, k) >= level {
        n += 1;
    }
    n
}

/// The smallest chance, over the counts `k..=last`, that the range built
/// from a fair handful holds the true count.
pub fn worst_coverage(k: u64, last: u64, level: Q) -> Q {
    let tops: Vec<u64> = (0..=last)
        .map(|m| if m < k { 0 } else { upper_bound(m, k, level) })
        .collect();
    (k..=last)
        .map(|n| share_by_largest(n, k, |m| tops[m as usize] >= n))
        .min()
        .expect("at least one count")
}

/// How many handfuls of k from 1..n have each sum (index = sum), by the
/// sum-of-subsets table: ways[j][s] handfuls of j boxes with sum s.
pub fn handfuls_by_sum(n: u64, k: usize) -> Vec<u128> {
    let top = (n as usize) * k + 1;
    let mut ways = vec![vec![0u128; top]; k + 1];
    ways[0][0] = 1;
    for b in 1..=n as usize {
        for j in (1..=k).rev() {
            for s in (b..top).rev() {
                ways[j][s] += ways[j - 1][s - b];
            }
        }
    }
    ways.swap_remove(k)
}

/// Average of `f(sum)` over the fair handfuls of k from 1..n.
pub fn mean_by_sum(n: u64, k: usize, f: impl Fn(u64) -> Q) -> Q {
    let ways = handfuls_by_sum(n, k);
    let total = ways.iter().sum::<u128>();
    ways.iter()
        .enumerate()
        .filter(|&(_, &w)| w > 0)
        .fold(Q::ZERO, |acc, (s, &w)| acc + Q::count(w) * f(s as u64))
        / Q::count(total)
}

/// The share of fair handfuls of k from 1..n whose largest number m and sum
/// s `keep`: the handfuls topped by m are m with any k − 1 of 1..m − 1,
/// counted by their sum with the sum-of-subsets table.
pub fn share_by_largest_and_sum(n: u64, k: usize, keep: impl Fn(u64, u64) -> bool) -> Q {
    let kept: u128 = (k as u64..=n)
        .map(|m| {
            handfuls_by_sum(m - 1, k - 1)
                .iter()
                .enumerate()
                .filter(|&(t, _)| keep(m, m + t as u64))
                .map(|(_, &w)| w)
                .sum::<u128>()
        })
        .sum();
    Q::count(kept) / c(n, k as u64)
}

/// Average of `f(s)` over the fair handfuls of k from n consecutive numbers,
/// by spread s: (n − s)·C(s − 1, k − 2) handfuls have spread s.
pub fn mean_by_spread(n: u64, k: u64, f: impl Fn(u64) -> Q) -> Q {
    (k - 1..n).fold(Q::ZERO, |acc, s| acc + Q::int((n - s) as i128) * c(s - 1, k - 2) * f(s)) / c(n, k)
}

/// The only recipe of the largest number that is unbiased at every count
/// from k to `top`, solved count by count: at count n, the handfuls topped by
/// n must make the average n.
pub fn forced_values(k: u64, top: u64) -> Vec<Q> {
    let mut g: Vec<Q> = Vec::new();
    for n in k..=top {
        let known = (k..n).fold(Q::ZERO, |s, m| s + with_largest(m, k) * g[(m - k) as usize]);
        g.push((Q::int(n as i128) * c(n, k) - known) / with_largest(n, k));
    }
    g
}

// ----------------------------------------------------- handfuls one by one

/// Every handful of k from 1..n, in lexicographic order (small cases only:
/// the tiny company, the sufficiency check).
pub fn all_handfuls(n: u64, k: usize) -> Vec<Vec<u64>> {
    fn grow(from: u64, n: u64, k: usize, cur: &mut Vec<u64>, out: &mut Vec<Vec<u64>>) {
        if cur.len() == k {
            out.push(cur.clone());
            return;
        }
        for x in from..=n {
            cur.push(x);
            grow(x + 1, n, k, cur, out);
            cur.pop();
        }
    }
    let mut out = Vec::new();
    grow(1, n, k, &mut Vec::new(), &mut out);
    out
}

/// Every way to split `total` unseen boxes into `parts` gaps, in
/// lexicographic order (stars and bars).
pub fn splits(total: u64, parts: usize) -> Vec<Vec<u64>> {
    if parts == 1 {
        return vec![vec![total]];
    }
    (0..=total)
        .flat_map(|first| {
            splits(total - first, parts - 1).into_iter().map(move |mut rest| {
                rest.insert(0, first);
                rest
            })
        })
        .collect()
}

// ------------------------------------------------- handfuls drawn by computer

/// The experiments' base seed; experiment `stream` uses `SEED + stream`.
pub const SEED: u64 = 20_261_024;

/// The streams: the five that leave the bin, ten handfuls from a thousand,
/// ten from seventy, twenty from the 71-box company.
pub const BIN_STREAM: u64 = 0;
pub const BIG_STREAM: u64 = 1;
pub const SMALL_STREAM: u64 = 2;
pub const AGAIN_STREAM: u64 = 3;

/// SplitMix64 (Steele, Lea and Flood's mixing of a Weyl sequence).
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A whole number in 0..n, every one equally likely: values from the
    /// incomplete last block of n are thrown away and drawn again.
    pub fn below(&mut self, n: u64) -> u64 {
        assert!(n > 0, "a draw needs at least one value");
        // 2^64 mod n, and the largest value kept is u64::MAX minus it
        let rest = (u64::MAX % n + 1) % n;
        loop {
            let x = self.next_u64();
            if x <= u64::MAX - rest {
                return x % n;
            }
        }
    }
}

/// `count` fair handfuls of k different boxes from 1..n, each drawn one box
/// at a time (a repeat is drawn again), sorted; the first ones, never chosen.
pub fn drawn_handfuls(stream: u64, n: u64, k: usize, count: usize) -> Vec<Vec<u64>> {
    // k different boxes exist only among at least k: past that the redraw
    // below would never end
    assert!(
        k as u64 <= n,
        "a handful of {k} different boxes needs at least {k} boxes, not {n}"
    );
    let mut rng = SplitMix64::new(SEED + stream);
    (0..count)
        .map(|_| {
            let mut seen: Vec<u64> = Vec::with_capacity(k);
            while seen.len() < k {
                let x = 1 + rng.below(n);
                if !seen.contains(&x) {
                    seen.push(x);
                }
            }
            seen.sort_unstable();
            seen
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Brute force over every handful of k from 1..n: the average of `f`.
    fn brute(n: u64, k: usize, f: impl Fn(&[u64]) -> Q) -> Q {
        let hs = all_handfuls(n, k);
        hs.iter().fold(Q::ZERO, |s, h| s + f(h.as_slice())) / Q::int(hs.len() as i128)
    }

    #[test]
    fn fractions_print_and_round_like_the_oracle() {
        assert_eq!(Q::new(13, 2).to_string(), "13/2");
        assert_eq!(Q::new(-4, 2).to_string(), "-2");
        assert_eq!(Q::new(143, 5).decimal1().unwrap(), "28.6");
        assert_eq!(Q::int(71).decimal1().unwrap(), "71.0");
        assert!(Q::new(1808, 25).decimal1().is_err());
        assert_eq!(Q::new(1001, 6).about1().unwrap(), "166.8");
        assert_eq!(Q::new(71, 6).about1().unwrap(), "11.8");
        assert_eq!(Q::new(5, 2).nearest(), 3);
        assert_eq!(Q::new(-5, 2).nearest(), -3);
        assert_eq!(Q::new(1808, 25).nearest(), 72);
        // square roots at their edges: exact, a half rounding up, just under
        // a half rounding down, and a root under a twentieth
        assert_eq!(Q::new(121, 100).sqrt_about1().unwrap(), "1.1");
        assert_eq!(Q::new(529, 400).sqrt_about1().unwrap(), "1.2");
        assert_eq!(Q::new(528, 400).sqrt_about1().unwrap(), "1.1");
        assert_eq!(Q::new(1, 1000).sqrt_about1().unwrap(), "0.0");
        assert_eq!(Q::new(4752, 35).sqrt_about1().unwrap(), "11.7");
        assert!(Q::new(-1, 2).sqrt_about1().is_err());
        // two decimals only when exact and not negative; square roots only
        // of squares
        assert_eq!(Q::new(17, 4).decimal2().unwrap(), "4.25");
        assert_eq!(Q::new(201, 100).decimal2().unwrap(), "2.01");
        assert_eq!(Q::int(3).decimal2().unwrap(), "3.00");
        assert!(Q::new(1, 3).decimal2().is_err());
        assert!(Q::new(-1, 4).decimal2().is_err());
        assert_eq!(Q::new(9, 4).exact_sqrt(), Some(Q::new(3, 2)));
        assert_eq!(Q::ZERO.exact_sqrt(), Some(Q::ZERO));
        assert_eq!(Q::int(3).exact_sqrt(), None);
        assert_eq!(Q::new(9, 8).exact_sqrt(), None);
        assert_eq!(Q::new(-9, 4).exact_sqrt(), None);
    }

    /// The friends' example: gaps 13, 12, 5, 17, 8 and the estimate 71, by
    /// the gaps and by the closed form m(1 + 1/k) − 1 (Clark, Gonye and
    /// Miller 2021, p. 3).
    #[test]
    fn the_friends_estimate_is_seventy_one_both_ways() {
        assert_eq!(gaps(&FRIENDS), vec![13, 12, 5, 17, 8]);
        assert_eq!(gap_recipe(&FRIENDS), Q::int(71));
        assert_eq!(gap_of(60, 5), Q::int(60) * (Q::ONE + Q::new(1, 5)) - Q::ONE);
        assert_eq!(doubling_recipe(&FRIENDS), Q::int(73));
    }

    /// Unbiased at every count, by the closed form and by brute force, and
    /// the gap recipe never below the largest number (equal exactly at 1..k).
    #[test]
    fn the_gap_recipe_is_unbiased_and_never_below_the_largest() {
        for k in 1..=4u64 {
            for n in k..=12 {
                let closed = mean_by_largest(n, k, |m| gap_of(m, k));
                assert_eq!(closed, Q::int(n as i128), "k {k}, n {n}");
                assert_eq!(brute(n, k as usize, gap_recipe), closed);
                for h in all_handfuls(n, k as usize) {
                    let m = *h.last().unwrap();
                    assert!(gap_recipe(&h) >= Q::int(m as i128));
                    let edge: Vec<u64> = (1..=k).collect();
                    assert_eq!(gap_recipe(&h) == Q::int(m as i128), h == edge);
                }
            }
        }
    }

    /// The sum table and the spread count agree with brute force.
    #[test]
    fn the_closed_forms_match_brute_force() {
        for (n, k) in [(9u64, 3usize), (11, 4), (12, 5)] {
            assert_eq!(
                mean_by_sum(n, k, |s| doubling_of(s, k as u64)),
                brute(n, k, doubling_recipe)
            );
            let ways = handfuls_by_sum(n, k);
            assert_eq!(ways.iter().sum::<u128>(), choose(n, k as u64));
            let below = |m: u64, s: u64| doubling_of(s, k as u64) < Q::int(m as i128);
            let brute_below = all_handfuls(n, k)
                .iter()
                .filter(|h| below(h[k - 1], h.iter().sum()))
                .count();
            assert_eq!(
                share_by_largest_and_sum(n, k, below),
                Q::new(brute_below as i128, choose(n, k as u64) as i128)
            );
            assert_eq!(
                mean_by_spread(n, k as u64, |s| spread_of(s, k as u64)),
                brute(n, k, |h| spread_of(h[k - 1] - h[0], k as u64))
            );
            for m in 0..=n {
                let at_most = all_handfuls(n, k).iter().filter(|h| h[k - 1] <= m).count();
                assert_eq!(
                    all_at_most(m, n, k as u64),
                    Q::new(at_most as i128, choose(n, k as u64) as i128)
                );
            }
        }
    }

    /// The forced values are the gap recipe's, and the staircase for pairs is
    /// 2, 7/2, 5, 13/2.
    #[test]
    fn the_forced_recipe_is_the_gap_recipe() {
        assert_eq!(
            forced_values(2, 5),
            vec![Q::int(2), Q::new(7, 2), Q::int(5), Q::new(13, 2)]
        );
        for k in 1..=5 {
            for (i, g) in forced_values(k, 30).into_iter().enumerate() {
                assert_eq!(g, gap_of(k + i as u64, k));
            }
        }
    }

    /// Ninety-five percent range: 60 to 107 for five, 60 to 79 for ten, and
    /// the chance one count past the top is below one in twenty.
    #[test]
    fn the_range_tops_out_at_one_hundred_and_seven() {
        let level = Q::new(1, 20);
        assert_eq!(upper_bound(60, 5, level), 107);
        assert!(all_at_most(60, 107, 5) >= level && all_at_most(60, 108, 5) < level);
        assert_eq!(upper_bound(60, 10, level), 79);
        assert!(worst_coverage(5, 40, level) >= Q::new(19, 20));
    }

    /// Splits are the compositions, once each, and the pairs from five boxes
    /// match them one to one through their gap lists.
    #[test]
    fn handfuls_and_gap_lists_match_one_to_one() {
        let lists: Vec<Vec<u64>> = all_handfuls(5, 2)
            .iter()
            .map(|h| {
                let mut g = gaps(h);
                g.push(5 - h[1]);
                g
            })
            .collect();
        let mut sorted = lists.clone();
        sorted.sort();
        assert_eq!(sorted, splits(3, 3));
        assert_eq!(splits(3, 3).len(), 10);
    }

    /// The generator keeps its draws in range, every handful is five
    /// different sorted boxes, and the streams differ; `below` rejects the
    /// incomplete block (n = 2^63 + 1 keeps fewer than half the values).
    #[test]
    fn drawn_handfuls_are_fair_handful_shaped() {
        for (stream, n) in [(BIG_STREAM, BIG), (SMALL_STREAM, SMALL), (AGAIN_STREAM, KNOWN), (9, 7)] {
            for h in drawn_handfuls(stream, n, 5, 50) {
                assert_eq!(h.len(), 5);
                assert!(h.windows(2).all(|w| w[0] < w[1]));
                assert!(h[0] >= 1 && h[4] <= n);
            }
        }
        let mut rng = SplitMix64::new(SEED);
        for n in [1, 2, 3, 70, 71, 1000, (1u64 << 63) + 1] {
            for _ in 0..200 {
                assert!(rng.below(n) < n);
            }
        }
        assert_ne!(
            drawn_handfuls(BIG_STREAM, BIG, 5, 3),
            drawn_handfuls(SMALL_STREAM, BIG, 5, 3)
        );
        // the same stream twice is the same draws
        assert_eq!(
            drawn_handfuls(BIG_STREAM, BIG, 5, 3),
            drawn_handfuls(BIG_STREAM, BIG, 5, 3)
        );
        // at the edge, a handful of every box made is every box
        assert_eq!(drawn_handfuls(9, 5, 5, 2), [[1, 2, 3, 4, 5], [1, 2, 3, 4, 5]]);
    }

    /// One box more than the company made is refused at once, not redrawn
    /// for ever.
    #[test]
    #[should_panic(expected = "needs at least 6 boxes, not 5")]
    fn a_handful_larger_than_the_company_is_refused() {
        drawn_handfuls(9, 5, 6, 1);
    }

    /// Negative controls: a recipe without the −1 is high by one; dividing
    /// the gaps by k + 1 is low; equal chances per box without equal
    /// handfuls (two deliveries) average 8, not 10.
    #[test]
    fn the_wrong_recipes_and_the_unfair_draw_fail() {
        for k in [2u64, 3, 5] {
            for n in k + 1..12 {
                let no_minus_one = mean_by_largest(n, k, |m| Q::int(m as i128) * (Q::ONE + Q::new(1, k as i128)));
                assert_eq!(no_minus_one, Q::int(n as i128 + 1));
                let k_plus_one = mean_by_largest(n, k, |m| {
                    Q::int(m as i128) + Q::new(m as i128 - k as i128, k as i128 + 1)
                });
                assert!(k_plus_one < Q::int(n as i128));
            }
        }
        let deliveries = [[1u64, 2, 3, 4, 5], [6, 7, 8, 9, 10]];
        let average = (gap_recipe(&deliveries[0]) + gap_recipe(&deliveries[1])) / Q::int(2);
        assert_eq!(average, Q::int(8));
    }

    /// The zoomed line's marks: the first number, then equal steps short of
    /// the end; refused at the edges where they would not be whole numbers
    /// in order.
    #[test]
    fn the_marks_divide_the_line_evenly() {
        assert_eq!(marks(1, BIG, ZOOM_PARTS).unwrap(), [1, 250, 500, 750]);
        assert_eq!(marks(1, 8, 2).unwrap(), [1, 4]);
        assert_eq!(marks(1, 2, 1).unwrap(), [1], "one part: the first number only");
        assert!(marks(1, 1002, 4).is_err(), "a quarter of 1002 is not whole");
        assert!(marks(1, BIG, 0).is_err(), "no parts");
        assert!(marks(1, 4, 4).is_err(), "a step of 1 does not lie past 1");
        assert!(
            marks(250, BIG, 4).is_err(),
            "the first mark past 250 would be 250 again"
        );
        for parts in 1..=10 {
            if let Ok(m) = marks(1, BIG, parts) {
                assert!(m.windows(2).all(|w| w[0] < w[1]) && *m.last().unwrap() < BIG);
                assert_eq!(m.len() as u64, parts);
            }
        }
    }
}
