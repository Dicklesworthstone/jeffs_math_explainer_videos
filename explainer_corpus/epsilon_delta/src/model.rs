//! The mathematics of "Epsilon and Delta: Why Getting Closer Wasn't
//! Enough": pure functions with tests; the chapters only draw.
//!
//! Chapter 01 is about the curves f_n(x) = x^n on the closed interval
//! [0, 1], for whole n ≥ 1. Every one of them is continuous. Their
//! pointwise limit is f(x) = 0 for 0 ≤ x < 1 and f(1) = 1, which jumps at
//! the right end (approached from inside the domain, so from the left). At
//! a fixed x the gap |x^n − f(x)| shrinks to zero: below 1 each new power
//! multiplies it by the same x < 1, and at x = 1 it is zero from the start.
//! At the moving point x_n = 2^(−1/n), which lies strictly inside [0, 1),
//! the n-th curve stands at exactly one half while f is zero, so that gap is
//! one half for every n, and no single n brings every gap below one half.
//! The witness is enough without being the worst point: the gap's supremum
//! over [0, 1] is 1 for every n, approached near x = 1 and never reached
//! (`sup_gap_approach`, which the tests use).
//!
//! Floating point draws the pictures. Every number the video prints is
//! certified with exact integers ([`Nat`]): the decimal expansions of
//! (a/10^k)^n, the first n from which a fixed point's gap stays below a
//! rational tolerance, and the rounding of each irrational number to the
//! places it is shown. The continuation's constructive deltas (the Sprague
//! and Davis notes, brief §3–4) are here too, with the naive guess that
//! fails.

use std::cmp::Ordering;
use std::fmt;

/// The curves the opening draws: x^n for these n (fact `C1.powers`).
pub const POWERS: [u32; 8] = [1, 2, 3, 4, 5, 10, 20, 50];

/// One half: the height the moving point keeps, and the gap it leaves.
pub const HALF: Ratio = Ratio { num: 1, den: 2 };

/// How many straight segments each drawn curve is made of.
pub const SEGMENTS: usize = 240;

fn check_point(x: f64) -> Result<(), String> {
    if (0.0..=1.0).contains(&x) {
        Ok(())
    } else {
        Err(format!("x = {x} is not in the domain [0, 1]"))
    }
}

fn check_power(n: u32) -> Result<(), String> {
    if n == 0 {
        Err("the curves are x^n for whole n ≥ 1; n = 0 is not one of them".to_owned())
    } else {
        Ok(())
    }
}

/// The n-th curve's height at x: x^n, for x in [0, 1] and n ≥ 1.
pub fn power(x: f64, n: u32) -> Result<f64, String> {
    check_point(x)?;
    check_power(n)?;
    Ok(x.powf(f64::from(n)))
}

/// The pointwise limit f of the curves: 0 on [0, 1), and 1 at x = 1.
pub fn limit(x: f64) -> Result<f64, String> {
    check_point(x)?;
    Ok(if x < 1.0 { 0.0 } else { 1.0 })
}

/// The gap |x^n − f(x)| between the n-th curve and the limit, at x.
pub fn gap(x: f64, n: u32) -> Result<f64, String> {
    Ok((power(x, n)? - limit(x)?).abs())
}

/// The moving point x_n = 2^(−1/n), where the n-th curve crosses one half.
/// It lies strictly inside [0, 1) because 2^(1/n) > 1, so the limit is zero
/// there and the gap is exactly one half.
pub fn half_height(n: u32) -> Result<f64, String> {
    check_power(n)?;
    let x = HALF.f64().powf(1.0 / f64::from(n));
    if (0.0..1.0).contains(&x) {
        Ok(x)
    } else {
        Err(format!("2^(-1/{n}) is {x} in floating point, not below 1"))
    }
}

/// A point of [0, 1) where the n-th curve's gap exceeds `target` (in
/// [0, 1)). The gap's supremum over [0, 1] is 1 for every n and is never
/// reached, so the half-height point is a sufficient witness, not the worst
/// one. (No chapter shows it yet; the tests hold the claim.)
#[cfg(test)]
pub fn sup_gap_approach(n: u32, target: f64) -> Result<f64, String> {
    check_power(n)?;
    if !(0.0..1.0).contains(&target) {
        return Err(format!("target {target} is not in [0, 1)"));
    }
    let x = f64::midpoint(1.0, target).powf(1.0 / f64::from(n));
    if x < 1.0 {
        Ok(x)
    } else {
        Err(format!("no floating-point x below 1 has x^{n} above {target}"))
    }
}

/// The sample abscissae every drawn curve shares, from 0 to 1, crowded
/// toward 1 where the high powers climb (x = 1 − (1 − k/K)²). Every curve
/// has the same x at each sample, so morphing one curve into the next moves
/// each point straight down, as squaring every height does.
pub fn sample_xs() -> Vec<f64> {
    (0..=SEGMENTS)
        .map(|k| {
            let t = 1.0 - k as f64 / SEGMENTS as f64;
            1.0 - t * t
        })
        .collect()
}

/// The n-th curve's samples (x, x^n) at [`sample_xs`].
pub fn curve(n: u32) -> Result<Vec<(f64, f64)>, String> {
    sample_xs().into_iter().map(|x| Ok((x, power(x, n)?))).collect()
}

// ------------------------------------------------------------ exact numbers

/// An exact fraction num/den of small whole numbers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ratio {
    pub num: u32,
    pub den: u32,
}

impl Ratio {
    /// num/den; a zero denominator is refused.
    pub fn new(num: u32, den: u32) -> Result<Self, String> {
        if den == 0 {
            Err(format!("{num}/0 is not a number"))
        } else {
            Ok(Self { num, den })
        }
    }

    /// A point of the domain [0, 1].
    pub fn point(num: u32, den: u32) -> Result<Self, String> {
        let r = Self::new(num, den)?;
        if num > den {
            Err(format!("{r} is not in the domain [0, 1]"))
        } else {
            Ok(r)
        }
    }

    /// A tolerance: strictly positive.
    pub fn tolerance(num: u32, den: u32) -> Result<Self, String> {
        let r = Self::new(num, den)?;
        if num == 0 {
            Err("a tolerance is strictly positive".to_owned())
        } else {
            Ok(r)
        }
    }

    pub fn f64(self) -> f64 {
        f64::from(self.num) / f64::from(self.den)
    }
}

impl fmt::Display for Ratio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.num, self.den)
    }
}

/// A natural number of any size: little-endian base-2^32 limbs with no
/// leading zero limb (zero is no limbs). Only what the certificates need:
/// products by small factors, powers, comparison and decimal digits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Nat(Vec<u32>);

impl Nat {
    pub fn small(v: u32) -> Self {
        if v == 0 { Self(Vec::new()) } else { Self(vec![v]) }
    }

    /// self × m.
    pub fn mul_small(&self, m: u32) -> Self {
        if m == 0 || self.0.is_empty() {
            return Self(Vec::new());
        }
        let mut out = Vec::with_capacity(self.0.len() + 1);
        let mut carry = 0u64;
        for &limb in &self.0 {
            let t = u64::from(limb) * u64::from(m) + carry;
            out.push(t as u32);
            carry = t >> 32;
        }
        if carry > 0 {
            out.push(carry as u32);
        }
        Self(out)
    }

    /// base^e (base^0 = 1).
    pub fn pow(base: u32, e: u32) -> Self {
        (0..e).fold(Self::small(1), |acc, _| acc.mul_small(base))
    }

    /// (self / d, self mod d), for d > 0.
    fn div_small(&self, d: u32) -> (Self, u32) {
        let mut q = vec![0u32; self.0.len()];
        let mut rem = 0u64;
        for (qi, &limb) in q.iter_mut().zip(&self.0).rev() {
            let cur = (rem << 32) | u64::from(limb);
            *qi = (cur / u64::from(d)) as u32;
            rem = cur % u64::from(d);
        }
        while q.last() == Some(&0) {
            q.pop();
        }
        (Self(q), rem as u32)
    }

    /// The decimal digits, most significant first ("0" for zero).
    pub fn decimal(&self) -> String {
        let mut chunks = Vec::new();
        let mut n = self.clone();
        while !n.0.is_empty() {
            let (q, r) = n.div_small(1_000_000_000);
            chunks.push(r);
            n = q;
        }
        let Some(top) = chunks.pop() else {
            return "0".to_owned();
        };
        let rest: String = chunks.iter().rev().map(|c| format!("{c:09}")).collect();
        format!("{top}{rest}")
    }
}

impl Ord for Nat {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .len()
            .cmp(&other.0.len())
            .then_with(|| self.0.iter().rev().cmp(other.0.iter().rev()))
    }
}

impl PartialOrd for Nat {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// The exact decimal expansion of (a/10^k)^n, for a point a/10^k of
/// [0, 1]: "0.729" for (9/10)^3, "1" when a = 10^k, "0" when a = 0; no
/// trailing zeros.
pub fn decimal_power(a: u32, k: u32, n: u32) -> Result<String, String> {
    check_power(n)?;
    let scale = 10u32.checked_pow(k).ok_or_else(|| format!("10^{k} is too large"))?;
    if a > scale {
        return Err(format!("{a}/10^{k} is not in the domain [0, 1]"));
    }
    if a == scale {
        return Ok("1".to_owned());
    }
    if a == 0 {
        return Ok("0".to_owned());
    }
    let places = k
        .checked_mul(n)
        .ok_or_else(|| format!("(a/10^{k})^{n} has too many places"))? as usize;
    // a < 10^k, so a^n < 10^(kn) has at most kn digits: pad on the left
    let digits = format!("{:0>places$}", Nat::pow(a, n).decimal());
    Ok(format!("0.{}", digits.trim_end_matches('0')))
}

/// A non-negative decimal string rounded half up to `places` places, with
/// trailing zeros kept: "0.59049" → "0.5905", "0.5" → "0.5000", "0.99996"
/// → "1.0000".
pub fn round_half_up(s: &str, places: usize) -> Result<String, String> {
    let (int, frac) = s.split_once('.').unwrap_or((s, ""));
    let digits_only = |t: &str| t.bytes().all(|b| b.is_ascii_digit());
    if int.is_empty() || !digits_only(int) || !digits_only(frac) {
        return Err(format!("{s:?} is not a non-negative decimal"));
    }
    let mut digits: Vec<u8> = int
        .bytes()
        .chain(frac.bytes().chain(std::iter::repeat(b'0')).take(places))
        .map(|b| b - b'0')
        .collect();
    if frac.as_bytes().get(places).is_some_and(|&b| b >= b'5') {
        let mut i = digits.len();
        loop {
            if i == 0 {
                digits.insert(0, 1);
                break;
            }
            i -= 1;
            if digits[i] == 9 {
                digits[i] = 0;
            } else {
                digits[i] += 1;
                break;
            }
        }
    }
    let text: String = digits.iter().map(|&d| char::from(b'0' + d)).collect();
    let int_len = text.len() - places;
    Ok(if places == 0 {
        text
    } else {
        format!("{}.{}", &text[..int_len], &text[int_len..])
    })
}

/// Whether the n-th curve's gap at the point x is below the tolerance eps,
/// exactly: x^n < eps when x < 1 (num^n · eps.den < eps.num · den^n), and
/// always at x = 1, where the gap is zero.
pub fn gap_below(x: Ratio, eps: Ratio, n: u32) -> bool {
    if x.num == x.den {
        return true;
    }
    Nat::pow(x.num, n).mul_small(eps.den) < Nat::pow(x.den, n).mul_small(eps.num)
}

/// The first n ≥ 1 from which the gap at the fixed point x stays below eps
/// (x in [0, 1], eps > 0): how long a fixed point has to wait. Below 1 the
/// gap falls strictly as n grows, and at 1 it is zero, so the first n that
/// works keeps working. Found by floating point, then certified exactly:
/// n works and n − 1 does not.
pub fn first_within(x: Ratio, eps: Ratio) -> Result<u32, String> {
    let x = Ratio::point(x.num, x.den)?;
    let eps = Ratio::tolerance(eps.num, eps.den)?;
    if x.num == x.den || x.num == 0 || eps.num >= eps.den {
        return Ok(1);
    }
    // x^n < eps exactly when n > ln eps / ln x (both logarithms negative)
    let guess = (eps.f64().ln() / x.f64().ln()).floor() + 1.0;
    let mut n = guess.clamp(1.0, f64::from(u32::MAX)) as u32;
    for _ in 0..64 {
        match (gap_below(x, eps, n), n > 1 && gap_below(x, eps, n - 1)) {
            (true, false) => return Ok(n),
            (true, true) => n -= 1,
            (false, _) => {
                n = n
                    .checked_add(1)
                    .ok_or_else(|| format!("x = {x} never gets within {eps} in u32"))?;
            }
        }
    }
    Err(format!("no certified first n for x = {x} within {eps} near {guess}"))
}

/// The moving point x_n = 2^(−1/n) rounded half up to `places` places (1 to
/// 9), certified with exact integers: the digits D returned satisfy
/// (D − ½)/10^p ≤ x_n < (D + ½)/10^p (a value exactly halfway rounds up),
/// that is 2·(2D − 1)^n ≤ (2·10^p)^n < 2·(2D + 1)^n, because x_n^n = 1/2
/// exactly. (No x_n is exactly halfway: it is irrational for n ≥ 2, and
/// x_1 = 1/2 is no midpoint (2D ± 1)/(2·10^p); the interval is half-up all
/// the same.) A point that would print as 1.000… is refused: x_n is below 1.
pub fn half_height_rounded(n: u32, places: u32) -> Result<String, String> {
    if !(1..=9).contains(&places) {
        return Err(format!("{places} places: show 1 to 9"));
    }
    let x = half_height(n)?;
    let scale = 10u32.pow(places);
    let guess = (x * f64::from(scale)).round() as u32;
    let mut mid: Option<Nat> = None;
    for d in [guess, guess.saturating_sub(1), guess + 1] {
        if d == 0 || d >= scale {
            continue;
        }
        let mid = mid.get_or_insert_with(|| Nat::pow(2 * scale, n));
        let low = Nat::pow(2 * d - 1, n).mul_small(2);
        let high = Nat::pow(2 * d + 1, n).mul_small(2);
        if low <= *mid && *mid < high {
            return Ok(format!("0.{d:0width$}", width = places as usize));
        }
    }
    Err(format!(
        "2^(-1/{n}) has no certified {places}-place rounding below 1 (near {guess})"
    ))
}

// ------------------------------------------- the continuation's deltas (§3–4)

fn gcd(a: u128, b: u128) -> u128 {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// Sprague's delta for x² at a = 2 (brief §3): the t ≥ 0 with t² + 4t = ε,
/// that is √(4 + ε) − 2, for ε below 5 (so δ < 1), rounded half up to
/// `places` places (1 to 12) and certified: (D − ½)/10^p ≤ δ < (D + ½)/10^p,
/// so a δ exactly halfway rounds up, which for the increasing
/// g(t) = t² + 4t is g((D − ½)/10^p) ≤ ε < g((D + ½)/10^p), in exact
/// integers. For |h| < δ, |x² − 4| = |h² + 4h| ≤ |h|² + 4|h| < δ² + 4δ = ε.
pub fn sprague_delta_rounded(eps: Ratio, places: u32) -> Result<String, String> {
    let eps = Ratio::tolerance(eps.num, eps.den)?;
    if eps.num >= 5 * eps.den {
        return Err(format!("ε = {eps}: Sprague's δ is 1 or more"));
    }
    if !(1..=12).contains(&places) {
        return Err(format!("{places} places: show 1 to 12"));
    }
    let scale = 10u128.pow(places);
    let s = 2 * scale;
    let (en, ed) = (u128::from(eps.num), u128::from(eps.den));
    // g(t/s) against ε, exactly: ed·(t² + 4ts) against en·s²
    let against = |t: u128| -> Result<Ordering, String> {
        let overflow = || format!("ε = {eps} at {places} places overflows");
        let g = t
            .checked_mul(t)
            .and_then(|tt| t.checked_mul(4 * s).and_then(|ts| tt.checked_add(ts)))
            .and_then(|v| v.checked_mul(ed))
            .ok_or_else(overflow)?;
        let e = s
            .checked_mul(s)
            .and_then(|ss| ss.checked_mul(en))
            .ok_or_else(overflow)?;
        Ok(g.cmp(&e))
    };
    let delta = (4.0 + eps.f64()).sqrt() - 2.0;
    let guess = (delta * scale as f64).round() as u128;
    for d in [guess, guess.saturating_sub(1), guess + 1] {
        if d >= scale {
            continue;
        }
        // the lower end may equal δ (an exact half belongs to D); the upper
        // end must lie strictly above it
        let lower_ok = d == 0 || against(2 * d - 1)? != Ordering::Greater;
        if lower_ok && against(2 * d + 1)? == Ordering::Greater {
            return Ok(format!("0.{d:0width$}", width = places as usize));
        }
    }
    Err(format!(
        "no certified rounding of Sprague's δ for ε = {eps} near {guess}"
    ))
}

/// The coefficients of P(a + h) in powers of h, lowest first, from P's
/// coefficients lowest first: the Taylor shift by repeated synthetic
/// division by (x − a), in exact integers.
pub fn recenter(p: &[i64], a: i64) -> Result<Vec<i128>, String> {
    let mut c: Vec<i128> = p.iter().map(|&v| i128::from(v)).collect();
    let a = i128::from(a);
    let deg = c.len().saturating_sub(1);
    for i in 0..deg {
        for j in (i..deg).rev() {
            let shifted = a
                .checked_mul(c[j + 1])
                .and_then(|t| c[j].checked_add(t))
                .ok_or("recentring overflows i128")?;
            c[j] = shifted;
        }
    }
    Ok(c)
}

/// Davis's bound C = Σ_{k≥1} |c_k| over the recentred coefficients: when
/// |h| ≤ 1 every |h|^k ≤ |h|, so |P(a + h) − P(a)| ≤ C·|h|.
pub fn local_bound(shifted: &[i128]) -> u128 {
    shifted.iter().skip(1).map(|c| c.unsigned_abs()).sum()
}

/// δ = min(1, ε/C) as a reduced fraction (num, den): with |h| < δ ≤ 1,
/// |P(a + h) − P(a)| ≤ C·|h| < C·δ ≤ ε. A constant polynomial (C = 0) is
/// within every ε everywhere; then any δ works and this returns 1.
pub fn davis_delta(bound: u128, eps: Ratio) -> (u128, u128) {
    let (num, den) = (u128::from(eps.num), u128::from(eps.den) * bound);
    if num >= den {
        return (1, 1);
    }
    let g = gcd(num, den);
    (num / g, den / g)
}

/// P(a + h) − P(a) = Σ_{k≥1} c_k h^k at the rational h = hn/hd (hd > 0),
/// exactly, as a reduced fraction with a positive denominator.
pub fn change_at(shifted: &[i128], hn: i128, hd: i128) -> Result<(i128, i128), String> {
    if hd <= 0 {
        return Err(format!("h = {hn}/{hd}: the denominator must be positive"));
    }
    let overflow = || "the change overflows i128".to_owned();
    let deg = u32::try_from(shifted.len().saturating_sub(1)).map_err(|_| overflow())?;
    let mut num: i128 = 0;
    for (k, &c) in (0u32..).zip(shifted).skip(1) {
        let term = hn
            .checked_pow(k)
            .and_then(|h| hd.checked_pow(deg - k).and_then(|d| h.checked_mul(d)))
            .and_then(|t| t.checked_mul(c))
            .ok_or_else(overflow)?;
        num = num.checked_add(term).ok_or_else(overflow)?;
    }
    let den = hd.checked_pow(deg).ok_or_else(overflow)?;
    let g = gcd(num.unsigned_abs(), den.unsigned_abs()).max(1);
    let g = i128::try_from(g).map_err(|_| overflow())?;
    Ok((num / g, den / g))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(num: u32, den: u32) -> Ratio {
        Ratio::new(num, den).unwrap()
    }

    #[test]
    fn the_domain_is_the_closed_interval_and_the_powers_start_at_one() {
        for bad in [-0.1, 1.0 + 1e-12, f64::NAN, f64::INFINITY] {
            assert!(power(bad, 2).is_err(), "{bad}");
            assert!(limit(bad).is_err(), "{bad}");
        }
        assert!(power(0.5, 0).is_err(), "n = 0 is not a curve of the sequence");
        assert!(half_height(0).is_err());
        for n in [1, 2, 50, u32::MAX] {
            assert_eq!(power(0.0, n).unwrap(), 0.0);
            assert_eq!(power(1.0, n).unwrap(), 1.0, "1^n = 1");
            assert_eq!(gap(1.0, n).unwrap(), 0.0, "the endpoint never moves");
        }
        assert_eq!(limit(0.0).unwrap(), 0.0);
        assert_eq!(limit(1.0 - f64::EPSILON).unwrap(), 0.0, "every x below 1 goes to 0");
        assert_eq!(limit(1.0).unwrap(), 1.0);
    }

    /// The witness: strictly inside [0, 1), the limit zero there, the curve
    /// at one half, so the gap is one half; it climbs toward 1 like
    /// 1 − ln 2/n and never reaches it, even at the largest n.
    #[test]
    fn the_moving_point_keeps_a_gap_of_one_half() {
        let mut last = 0.0;
        for n in 1..=200 {
            let x = half_height(n).unwrap();
            assert!(x > last && x < 1.0, "n = {n}: x = {x}");
            assert_eq!(limit(x).unwrap(), 0.0);
            assert!((power(x, n).unwrap() - 0.5).abs() < 1e-12, "n = {n}");
            assert!((gap(x, n).unwrap() - 0.5).abs() < 1e-12, "n = {n}");
            let t = std::f64::consts::LN_2 / f64::from(n);
            assert!(1.0 - x < t && 1.0 - x > t - t * t / 2.0, "n = {n}");
            last = x;
        }
        let x = half_height(u32::MAX).unwrap();
        assert!(x < 1.0 && (gap(x, u32::MAX).unwrap() - 0.5).abs() < 1e-5);
    }

    /// The half-height point suffices and is not the worst: every curve has
    /// gaps as close to 1 as you like, just below x = 1.
    #[test]
    fn the_witness_is_sufficient_not_optimal() {
        for n in POWERS {
            for target in [0.5, 0.9, 0.999] {
                let x = sup_gap_approach(n, target).unwrap();
                assert!(x < 1.0 && gap(x, n).unwrap() > target, "n = {n}, target {target}");
            }
            assert!(x_bigger_gap(n) > 0.5);
        }
        assert!(sup_gap_approach(3, 1.0).is_err());
        assert!(sup_gap_approach(3, -0.1).is_err());
    }

    fn x_bigger_gap(n: u32) -> f64 {
        let x = half_height(n).unwrap();
        gap(f64::midpoint(x, 1.0), n).unwrap()
    }

    /// Fixed points wait a finite time, longer the nearer they sit to 1:
    /// with x fixed first, every gap gets below one half.
    #[test]
    fn a_fixed_point_waits_longer_the_nearer_it_is_to_one() {
        let half = r(1, 2);
        assert_eq!(first_within(r(9, 10), half).unwrap(), 7);
        assert_eq!(first_within(r(99, 100), half).unwrap(), 69);
        assert_eq!(first_within(r(999, 1000), half).unwrap(), 693);
        assert_eq!(first_within(r(1, 1), half).unwrap(), 1, "the endpoint's gap is 0");
        assert_eq!(first_within(r(0, 7), half).unwrap(), 1);
        assert_eq!(first_within(r(9, 10), r(3, 2)).unwrap(), 1, "a tolerance above 1");
        assert!(first_within(r(11, 10), half).is_err(), "off the domain");
        assert!(first_within(r(9, 10), r(0, 1)).is_err(), "a zero tolerance");
        // the gap falls strictly at a fixed point below 1
        for n in 1..60 {
            assert!(gap(0.9, n + 1).unwrap() < gap(0.9, n).unwrap());
        }
    }

    /// The certified first n agrees with an exact count from n = 1 up.
    #[test]
    fn first_within_matches_counting_up() {
        for den in [7, 20] {
            for num in 0..=den {
                for eps in [r(1, 2), r(1, 10), r(3, 4)] {
                    let x = r(num, den);
                    let counted = (1..2000).find(|&n| gap_below(x, eps, n)).unwrap();
                    assert_eq!(first_within(x, eps).unwrap(), counted, "{x}, {eps}");
                }
            }
        }
    }

    /// No single n serves every x: a rational point just past each curve's
    /// crossing, still below 1, has a gap above one half, exactly. (The
    /// certificate's upper end is that point.)
    #[test]
    fn no_single_n_brings_every_gap_below_one_half() {
        for n in 1..=60 {
            let shown = half_height_rounded(n, 4).unwrap();
            let d: u32 = shown.strip_prefix("0.").unwrap().parse().unwrap();
            let q = Ratio::point(2 * d + 1, 20_000).unwrap();
            assert!(q.num < q.den, "the point is below 1");
            assert!(!gap_below(q, HALF, n), "n = {n}: the gap at {q} is above 1/2");
            assert!(first_within(q, HALF).unwrap() > n, "n = {n}");
        }
    }

    #[test]
    fn exact_decimals_of_the_fixed_points_heights() {
        let want = [
            (1, "0.9"),
            (2, "0.81"),
            (3, "0.729"),
            (4, "0.6561"),
            (5, "0.59049"),
            (10, "0.3486784401"),
        ];
        for (n, s) in want {
            assert_eq!(decimal_power(9, 1, n).unwrap(), s);
        }
        assert_eq!(decimal_power(99, 2, 2).unwrap(), "0.9801");
        assert_eq!(decimal_power(5, 1, 2).unwrap(), "0.25");
        assert_eq!(decimal_power(10, 1, 3).unwrap(), "1");
        assert_eq!(decimal_power(0, 1, 3).unwrap(), "0");
        assert!(decimal_power(11, 1, 2).is_err());
        assert!(decimal_power(9, 1, 0).is_err());
        let fifty = decimal_power(9, 1, 50).unwrap();
        assert_eq!(fifty.len(), 2 + 50, "9^50 needs all fifty places");
        assert_eq!(round_half_up(&fifty, 4).unwrap(), "0.0052");
        assert_eq!(round_half_up(&decimal_power(9, 1, 20).unwrap(), 4).unwrap(), "0.1216");
    }

    #[test]
    fn rounding_half_up_on_exact_decimals() {
        let cases = [
            ("0.59049", 4, "0.5905"),
            ("0.5", 4, "0.5000"),
            ("0.99996", 4, "1.0000"),
            ("0.12345", 4, "0.1235"),
            ("0.123449", 4, "0.1234"),
            ("1", 2, "1.00"),
            ("0.9", 0, "1"),
            ("9.995", 2, "10.00"),
        ];
        for (s, p, want) in cases {
            assert_eq!(round_half_up(s, p).unwrap(), want, "{s} to {p}");
        }
        for bad in ["", ".5", "-0.5", "1e3", "0.5.1"] {
            assert!(round_half_up(bad, 2).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn the_witness_is_printed_to_certified_digits() {
        assert_eq!(half_height_rounded(1, 4).unwrap(), "0.5000");
        assert_eq!(half_height_rounded(2, 4).unwrap(), "0.7071");
        assert_eq!(half_height_rounded(10, 4).unwrap(), "0.9330");
        assert_eq!(half_height_rounded(50, 4).unwrap(), "0.9862");
        // x_1 = 1/2 exactly, a printed value itself at every precision
        for places in 1..=9u32 {
            let zeros = "0".repeat(places as usize - 1);
            assert_eq!(half_height_rounded(1, places).unwrap(), format!("0.5{zeros}"));
        }
        for n in 1..=100 {
            let f = half_height(n).unwrap();
            let shown: f64 = half_height_rounded(n, 6).unwrap().parse().unwrap();
            assert!((shown - f).abs() <= 5e-7 + 1e-12, "n = {n}");
        }
        // x_20000 = 0.99997…: four places would print 1.0000 for a point below 1
        assert!(half_height_rounded(20_000, 4).is_err());
        assert_eq!(half_height_rounded(20_000, 6).unwrap(), "0.999965");
        assert!(half_height_rounded(5, 0).is_err());
        assert!(half_height_rounded(0, 4).is_err());
    }

    #[test]
    fn big_naturals_multiply_compare_and_print() {
        assert_eq!(Nat::pow(2, 100).decimal(), "1267650600228229401496703205376");
        assert_eq!(Nat::pow(10, 30).decimal(), format!("1{}", "0".repeat(30)));
        assert_eq!(Nat::small(0).decimal(), "0");
        assert_eq!(Nat::pow(7, 0).decimal(), "1");
        assert_eq!(Nat::pow(3, 5).mul_small(0), Nat::small(0));
        assert!(Nat::pow(9, 7).mul_small(2) < Nat::pow(10, 7), "0.9^7 < 1/2");
        assert!(Nat::pow(9, 6).mul_small(2) > Nat::pow(10, 6), "0.9^6 > 1/2");
        assert!(Nat::pow(2, 64) > Nat::small(u32::MAX), "more limbs is bigger");
        assert_eq!(Nat::pow(1_000_000_000, 2).decimal(), format!("1{}", "0".repeat(18)));
    }

    #[test]
    fn the_curves_are_drawn_finely_enough() {
        let xs = sample_xs();
        assert_eq!(xs.len(), SEGMENTS + 1);
        assert_eq!((xs[0], xs[SEGMENTS]), (0.0, 1.0));
        assert!(xs.windows(2).all(|w| w[1] > w[0]));
        for n in POWERS {
            let c = curve(n).unwrap();
            for w in c.windows(2) {
                let ((x0, y0), (x1, y1)) = (w[0], w[1]);
                assert!((x1 - x0).hypot(y1 - y0) < 0.03, "n = {n}: a chord at x = {x0}");
                // a straight piece sags from the curve by at most this much
                let m = f64::midpoint(x0, x1);
                assert!((power(m, n).unwrap() - f64::midpoint(y0, y1)).abs() < 1e-3, "n = {n}");
            }
        }
    }

    /// The two worked examples of Davis's note (brief §4), the square of
    /// Sprague's, and a constant: recentring is exact.
    #[test]
    fn recentring_and_davis_bounds() {
        let cubic = recenter(&[-6, 0, 5, 1], 1).unwrap();
        assert_eq!(cubic, [0, 13, 8, 1]);
        assert_eq!(local_bound(&cubic), 22);
        let quartic = recenter(&[2, -8, 0, 0, -3], -2).unwrap();
        assert_eq!(quartic, [-30, 88, -72, 24, -3]);
        assert_eq!(local_bound(&quartic), 187);
        let square = recenter(&[0, 0, 1], 2).unwrap();
        assert_eq!(square, [4, 4, 1]);
        assert_eq!(local_bound(&square), 5);
        assert_eq!(recenter(&[7], 3).unwrap(), [7]);
        assert_eq!(local_bound(&[7]), 0);
        // P(a + h) is the recentred polynomial at h, for every whole h tried
        let eval = |c: &[i128], x: i128| c.iter().rev().fold(0i128, |acc, &k| acc * x + k);
        let p: Vec<i128> = [2i128, -8, 0, 0, -3].into();
        for h in -5..=5 {
            assert_eq!(eval(&p, -2 + h), eval(&quartic, h), "h = {h}");
        }
        assert_eq!(davis_delta(22, r(1, 10)), (1, 220));
        assert_eq!(davis_delta(5, r(1, 10)), (1, 50));
        assert_eq!(davis_delta(0, r(1, 10)), (1, 1), "a constant: any δ");
        assert_eq!(davis_delta(22, r(30, 1)), (1, 1), "δ never exceeds 1");
    }

    /// Davis's δ keeps the cubic within ε at rational points across its
    /// whole interval, right up to its edges (which |h| < δ excludes).
    #[test]
    fn davis_delta_keeps_the_cubic_within_epsilon() {
        let cubic = recenter(&[-6, 0, 5, 1], 1).unwrap();
        let (dn, dd) = davis_delta(local_bound(&cubic), r(1, 10));
        let (dn, dd) = (i128::try_from(dn).unwrap(), i128::try_from(dd).unwrap());
        for m in -1000..=1000 {
            let (num, den) = change_at(&cubic, dn * m, dd * 1000).unwrap();
            // |change| < 1/10  ⟺  10·|num| < den
            assert!(10 * num.abs() < den, "h = {m}/1000 of δ: {num}/{den}");
        }
    }

    /// The guess δ = ε/4 (the slope at 2, ignoring h²) fails for x² at 2:
    /// at ε = 1, h = 6/25 is inside |h| < 1/4 and moves x² by 636/625 > 1.
    /// Sprague's δ and the simpler min(1, ε/5) both work at ε = 1/10; the
    /// simpler one is smaller, and smaller is fine.
    #[test]
    fn a_naive_delta_fails_and_two_honest_ones_work() {
        let square = recenter(&[0, 0, 1], 2).unwrap();
        // the naive δ is ε/c₁ = 1/4, and h = 6/25 is inside it: 6 · 4 < 25 · 1
        assert_eq!(square[1], 4);
        assert!(6 * square[1] < 25, "6/25 is inside |h| < 1/4");
        assert_eq!(change_at(&square, 6, 25).unwrap(), (636, 625));
        let sprague: f64 = sprague_delta_rounded(r(1, 10), 10).unwrap().parse().unwrap();
        assert_eq!(sprague_delta_rounded(r(1, 10), 10).unwrap(), "0.0248456731");
        assert!((sprague - (4.1f64.sqrt() - 2.0)).abs() < 1e-10);
        let simple = 1.0 / 50.0;
        assert!(simple < sprague);
        for delta in [simple, sprague - 1e-10] {
            for h in [delta, -delta] {
                assert!((h * h + 4.0 * h).abs() < 0.1, "|h| = {delta}");
            }
        }
        // at the simple δ's own edge, exactly: (1/50)² + 4/50 = 201/2500 < 1/10
        assert_eq!(change_at(&square, 1, 50).unwrap(), (201, 2500));
        assert!(sprague_delta_rounded(r(5, 1), 4).is_err(), "δ would be 1");
        assert!(sprague_delta_rounded(r(1, 10), 0).is_err());
    }

    /// A δ exactly halfway between two printed values rounds up. The
    /// reviewer's counterexample (FuchsiaHawk, Mail 3738): at ε = 81/400,
    /// δ = √(1681/400) − 2 = 41/20 − 2 = 1/20 exactly, halfway between 0.0
    /// and 0.1, and it once printed 0.0. Each rational δ = num/den below
    /// comes from ε = δ² + 4δ = (num² + 4·num·den)/den², and its printed
    /// digits must equal its exact decimal rounded half up, ties included
    /// (0.05 → 0.1, 0.025 → 0.03, 0.125 → 0.13, 0.0075 → 0.008).
    #[test]
    fn an_exact_half_rounds_up() {
        assert_eq!(sprague_delta_rounded(r(81, 400), 1).unwrap(), "0.1");
        let deltas = [
            (1, 20, "0.05"),
            (1, 10, "0.1"),
            (1, 40, "0.025"),
            (1, 8, "0.125"),
            (3, 400, "0.0075"),
        ];
        for (num, den, exact) in deltas {
            let eps = r(num * num + 4 * num * den, den * den);
            for places in 1..=4u32 {
                let want = round_half_up(exact, places as usize).unwrap();
                assert_eq!(
                    sprague_delta_rounded(eps, places).unwrap(),
                    want,
                    "δ = {num}/{den} = {exact} at {places} places"
                );
            }
        }
        assert_eq!(sprague_delta_rounded(r(161, 1600), 2).unwrap(), "0.03");
        assert_eq!(sprague_delta_rounded(r(4809, 160_000), 3).unwrap(), "0.008");
    }
}
