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
    for d in [guess, guess.saturating_sub(1), guess + 1] {
        if d == 0 || d >= scale {
            continue;
        }
        let lower_ok = half_height_vs(n, 2 * d - 1, 2 * scale) != Ordering::Less;
        if lower_ok && half_height_vs(n, 2 * d + 1, 2 * scale) == Ordering::Less {
            return Ok(format!("0.{d:0width$}", width = places as usize));
        }
    }
    Err(format!(
        "2^(-1/{n}) has no certified {places}-place rounding below 1 (near {guess})"
    ))
}

/// Where the moving point x_n lies against the fraction num/den, exactly:
/// x_n < num/den exactly when 1/2 < (num/den)^n, that is den^n < 2·num^n
/// (both sides positive, t ↦ t^n increasing). Equal only when num/den is
/// x_n itself, which a fraction can be only at n = 1.
pub fn half_height_vs(n: u32, num: u32, den: u32) -> Ordering {
    Nat::pow(den, n).cmp(&Nat::pow(num, n).mul_small(2))
}

// ------------------------------------------------ chapter 02: the strip

/// The powers chapter 02 raises the bright curve through after the
/// opening's last, 50 (fact `C2.powers`).
pub const MORE_POWERS: [u32; 4] = [100, 200, 500, 1000];

/// Chapter 02's sideways stretches: at m the zoom window is [1 − 1/m, 1]
/// drawn as wide as the inset (fact `C2.zoom.*`); each is ten times the last.
pub const ZOOMS: [u32; 3] = [10, 100, 1000];

/// The width 1 − x_n of the strip where the n-th curve is above one half
/// (and of the strip where its gap is at least one half), in floating
/// point: 1 − e^(−ln 2/n), computed without cancellation.
pub fn width(n: u32) -> Result<f64, String> {
    check_power(n)?;
    Ok(-(-std::f64::consts::LN_2 / f64::from(n)).exp_m1())
}

/// Bounds on the width for every n, from t − t²/2 < 1 − e^(−t) < t with
/// t = ln 2/n. So n·(1 − x_n) climbs toward ln 2 as n grows: an asymptotic
/// statement. At no n is the width ln 2/n, and at none is it zero. (Chapter
/// 02's card shows only the exact width, its positivity and the limit; these
/// bounds support it in the tests, the oracle and the README, and nothing
/// drawn is computed from them.)
#[cfg(test)]
pub fn width_bounds(n: u32) -> Result<(f64, f64), String> {
    check_power(n)?;
    let t = std::f64::consts::LN_2 / f64::from(n);
    Ok((t - t * t / 2.0, t))
}

/// m·(1 − x_n), the n-th strip's width stretched m times, rounded half up
/// to `places` places (1 to 9) and certified like the witness. With
/// A = 2·10^p·m, the digits W returned satisfy
/// (W − ½)/10^p ≤ m(1 − x_n) < (W + ½)/10^p, that is
/// (A − 2W − 1)/A < x_n ≤ (A − 2W + 1)/A. m = 1 gives the width itself;
/// m = n gives the share of a window 1/n wide that the strip fills.
pub fn width_rounded(n: u32, m: u32, places: u32) -> Result<String, String> {
    check_power(n)?;
    if m == 0 || !(1..=9).contains(&places) {
        return Err(format!(
            "a stretch of {m} at {places} places: stretch at least 1, show 1 to 9"
        ));
    }
    let scale = 10u32.pow(places);
    let a = scale
        .checked_mul(2)
        .and_then(|s| s.checked_mul(m))
        .ok_or_else(|| format!("{places} places at a stretch of {m} overflow"))?;
    let stretched = f64::from(m) * width(n)?;
    if stretched >= 1.0 {
        return Err(format!("{m}·(1 − x_{n}) = {stretched} is not below 1"));
    }
    let guess = (stretched * f64::from(scale)).round() as u32;
    for d in [guess, guess.saturating_sub(1), guess + 1] {
        if d >= scale {
            continue;
        }
        // at least (2d − 1)/(2·10^p), so x_n ≤ (A − 2d + 1)/A; below
        // (2d + 1)/(2·10^p), so x_n > (A − 2d − 1)/A
        let lower_ok = d == 0 || half_height_vs(n, a - 2 * d + 1, a) != Ordering::Greater;
        if lower_ok && half_height_vs(n, a - 2 * d - 1, a) == Ordering::Greater {
            return Ok(format!("0.{d:0width$}", width = places as usize));
        }
    }
    Err(format!(
        "{m}·(1 − x_{n}) has no certified {places}-place rounding (near {guess})"
    ))
}

/// Whether the n-th curve is above one half at the point x of [0, 1],
/// exactly: x^n > 1/2. This region is (x_n, 1]: it contains x = 1, where
/// the curve is 1, and not x_n, where the curve is exactly 1/2.
pub fn above_half(x: Ratio, n: u32) -> bool {
    Nat::pow(x.num, n).mul_small(2) > Nat::pow(x.den, n)
}

/// Whether the n-th curve's gap to the limit is at least one half at the
/// point x of [0, 1], exactly: x < 1 and x^n ≥ 1/2. This region is
/// [x_n, 1): it contains x_n, where the gap is exactly 1/2, and not x = 1,
/// where the curve and the limit are both 1 and the gap is zero.
pub fn gap_at_least_half(x: Ratio, n: u32) -> bool {
    x.num < x.den && Nat::pow(x.num, n).mul_small(2) >= Nat::pow(x.den, n)
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

// ------------------------------------------ chapter 03: the old objection

/// A signed exact fraction num/den in lowest terms, den > 0: chapter 03's
/// increments h and what the square does with them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frac {
    pub num: i64,
    pub den: i64,
}

impl Frac {
    /// num/den in lowest terms; a zero denominator is refused. (The tests
    /// build fractions with it; the chapter's are constants.)
    #[cfg(test)]
    pub fn new(num: i64, den: i64) -> Result<Self, String> {
        Self::reduced(i128::from(num), i128::from(den))
    }

    pub const fn whole(n: i64) -> Self {
        Self { num: n, den: 1 }
    }

    fn reduced(num: i128, den: i128) -> Result<Self, String> {
        if den == 0 {
            return Err(format!("{num}/0 is not a number"));
        }
        // den ≠ 0, so g ≥ 1
        let g = i128::try_from(gcd(num.unsigned_abs(), den.unsigned_abs())).map_err(|_| "a fraction overflows")?;
        let (num, den) = (den.signum() * num / g, den.signum() * den / g);
        let overflow = || format!("{num}/{den} overflows");
        Ok(Self {
            num: i64::try_from(num).map_err(|_| overflow())?,
            den: i64::try_from(den).map_err(|_| overflow())?,
        })
    }

    fn wide(self) -> (i128, i128) {
        (i128::from(self.num), i128::from(self.den))
    }

    pub fn plus(self, o: Self) -> Result<Self, String> {
        let ((a, b), (c, d)) = (self.wide(), o.wide());
        Self::reduced(a * d + c * b, b * d)
    }

    pub fn minus(self, o: Self) -> Result<Self, String> {
        let ((a, b), (c, d)) = (self.wide(), o.wide());
        Self::reduced(a * d - c * b, b * d)
    }

    pub fn times(self, o: Self) -> Result<Self, String> {
        let ((a, b), (c, d)) = (self.wide(), o.wide());
        Self::reduced(a * c, b * d)
    }

    /// self / o; dividing by zero is refused, never approximated.
    pub fn over(self, o: Self) -> Result<Self, String> {
        if o.num == 0 {
            return Err(format!("{self}/0 is not a number"));
        }
        let ((a, b), (c, d)) = (self.wide(), o.wide());
        Self::reduced(a * d, b * c)
    }

    pub fn negated(self) -> Result<Self, String> {
        let (a, b) = self.wide();
        Self::reduced(-a, b)
    }

    pub fn is_zero(self) -> bool {
        self.num == 0
    }

    /// Whether self is within `tol` of `centre`, strictly: |self − centre| < tol.
    pub fn within(self, centre: Self, tol: Self) -> Result<bool, String> {
        let d = self.minus(centre)?;
        Ok(d < tol && d > tol.negated()?)
    }

    pub fn f64(self) -> f64 {
        self.num as f64 / self.den as f64
    }
}

impl Ord for Frac {
    fn cmp(&self, other: &Self) -> Ordering {
        // both denominators are positive
        let ((a, b), (c, d)) = (self.wide(), other.wide());
        (a * d).cmp(&(c * b))
    }
}

impl PartialOrd for Frac {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Frac {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.den == 1 {
            write!(f, "{}", self.num)
        } else {
            write!(f, "{}/{}", self.num, self.den)
        }
    }
}

/// Chapter 03's example, ours and modern, not Berkeley's: his §§XIII–XIV
/// treat any power x^n and write o for the increment. Here the square, at
/// x = 1/2 (fact `C3.base`).
pub const BASE: Frac = Frac { num: 1, den: 2 };
/// The square's power (fact `C3.power`).
pub const SQUARE: u32 = 2;

/// The increments chapter 03 shrinks, from above and then from below (facts
/// `C3.h.*`). None is zero, so every chord has two distinct ends, and each
/// keeps 1/2 + h inside the plot's [0, 1].
pub const NUDGES: [Frac; 6] = [
    Frac { num: 1, den: 2 },
    Frac { num: 1, den: 4 },
    Frac { num: 1, den: 10 },
    Frac { num: -1, den: 2 },
    Frac { num: -1, den: 4 },
    Frac { num: -1, den: 10 },
];

/// The tolerances chapter 03 draws around the limit (facts `C3.eps.*`).
pub const TOLERANCES: [Frac; 2] = [Frac { num: 1, den: 10 }, Frac { num: 1, den: 20 }];

/// (a + h)² − a², the square's change from a to a + h, exactly. At h = 0
/// it is zero.
pub fn square_change(a: Frac, h: Frac) -> Result<Frac, String> {
    let moved = a.plus(h)?;
    moved.times(moved)?.minus(a.times(a)?)
}

/// The slope of the chord of y = x² from x = a to x = a + h,
/// ((a + h)² − a²)/h, for h ≠ 0 only. At h = 0 the chord's two ends are one
/// point and the quotient is 0/0, which names no number: it is refused,
/// never evaluated.
pub fn chord_slope(a: Frac, h: Frac) -> Result<Frac, String> {
    if h.is_zero() {
        return Err(format!(
            "at h = 0 the chord's slope is (({a})² − ({a})²)/0 = 0/0, which is no number"
        ));
    }
    square_change(a, h)?.over(h)
}

/// The change and the slope as polynomials in h, coefficients lowest power
/// first: (a + h)² − a² = 0 + 2a·h + h², and for h ≠ 0 only, dividing by h
/// leaves the slope 2a + h. The tests hold [`chord_slope`] to it.
pub fn chord_formula(a: Frac) -> Result<([Frac; 3], [Frac; 2]), String> {
    let two_a = a.times(Frac::whole(2))?;
    let one = Frac::whole(1);
    Ok(([Frac::whole(0), two_a, one], [two_a, one]))
}

/// Where the chord slopes go as h → 0 through nonzero h: 2a. The slope is
/// 2a + h exactly, so its distance from 2a is |h|: for every tolerance t > 0
/// the window 0 < |h| < t ([`window`]) keeps it within t, and no wider window
/// does, since at |h| = t the distance is t itself.
pub fn chord_limit(a: Frac) -> Result<Frac, String> {
    a.times(Frac::whole(2))
}

/// The half-width of the window of nonzero h that keeps every chord slope
/// within the tolerance t of [`chord_limit`]: t itself, for t > 0.
pub fn window(t: Frac) -> Result<Frac, String> {
    if t <= Frac::whole(0) {
        Err(format!("a tolerance is strictly positive, not {t}"))
    } else {
        Ok(t)
    }
}

/// The line through (x0, y0) with slope s, clipped to the square [0, 1]²:
/// its ends, left to right. Floating point: it is drawn, never printed.
pub fn clip_unit(x0: f64, y0: f64, s: f64) -> Result<[(f64, f64); 2], String> {
    let y = |x: f64| y0 + s * (x - x0);
    let (mut lo, mut hi) = (0.0f64, 1.0f64);
    if s == 0.0 {
        if !(0.0..=1.0).contains(&y0) {
            return Err(format!("the flat line y = {y0} misses [0, 1]²"));
        }
    } else {
        let (at0, at1) = (x0 - y0 / s, x0 + (1.0 - y0) / s);
        lo = lo.max(at0.min(at1));
        hi = hi.min(at0.max(at1));
    }
    if lo < hi {
        Ok([(lo, y(lo)), (hi, y(hi))])
    } else {
        Err(format!("the line through ({x0}, {y0}) with slope {s} misses [0, 1]²"))
    }
}

// ------------------------------------------- chapter 04: the challenge game

/// The challenger's tolerance in chapter 04's honest round, and the
/// defender's wider answer, which loses (facts `C4.eps`, `C4.wide`).
pub const CHALLENGE: Frac = Frac { num: 1, den: 10 };
pub const WIDE: Frac = Frac { num: 1, den: 5 };
/// The challenger's picks against the honest window, from both sides, and
/// the pick that happens to land against the wide one (facts `C4.pick.*`,
/// `C4.lucky`): none is zero, and each sits clear of the hollow mark.
pub const PICKS: [Frac; 3] = [
    Frac { num: -2, den: 25 },
    Frac { num: 1, den: 25 },
    Frac { num: 9, den: 100 },
];
pub const LUCKY: Frac = Frac { num: 1, den: 20 };
/// The values chapter 04 fills the hole at h = 0 with: the limit itself,
/// and another (facts `C4.fill.*`).
pub const FILLS: [Frac; 2] = [Frac { num: 1, den: 1 }, Frac { num: 3, den: 2 }];

/// The symmetric safe radius around the base point a, for a inside (0, 1):
/// every h with |h| ≤ min(a, 1 − a) keeps the input a + h in the square's
/// domain [0, 1]. On one side the domain may reach farther; this is the
/// radius that is safe on both. Every pick chapter 04 plays or tests stays
/// within it.
pub fn reach(a: Frac) -> Result<Frac, String> {
    let rest = Frac::whole(1).minus(a)?;
    if a <= Frac::whole(0) || rest <= Frac::whole(0) {
        return Err(format!("the base point {a} is not inside (0, 1)"));
    }
    Ok(if a < rest { a } else { rest })
}

fn least(x: Frac, y: Frac) -> Frac {
    if x < y { x } else { y }
}

fn size(x: Frac) -> Result<Frac, String> {
    if x < Frac::whole(0) { x.negated() } else { Ok(x) }
}

/// A pick, inside the domain, that beats the window 0 < |h| < delta
/// against the tolerance t around the slopes' limit at a, if there is one.
/// The slope is the limit plus h exactly ([`chord_slope`]), so the window
/// works when delta ≤ t (None). When delta > t, the pick
/// h = min((t + delta)/2, [`reach`]) has t < h < delta, since the reach is
/// more than t, and a + h stays in [0, 1]; its slope is h away from the
/// limit.
///
/// Only tolerances below the safe radius are supported (t < reach), and a
/// larger one is refused. That is a limit of this function, not a claim
/// that no pick misses: at a = 1/2 and t = 1/2, the legal pick h = 1/2 is
/// exactly 1/2 away and misses the strict tolerance for any delta > 1/2.
pub fn window_failure(a: Frac, t: Frac, delta: Frac) -> Result<Option<Frac>, String> {
    let t = window(t)?;
    if delta <= Frac::whole(0) {
        return Err(format!("a window is strictly positive, not {delta}"));
    }
    let far = reach(a)?;
    if t >= far {
        return Err(format!(
            "a tolerance of {t} is outside the supported range: it must be below the safe radius {far}"
        ));
    }
    if delta <= t {
        return Ok(None);
    }
    Ok(Some(least(t.plus(delta)?.over(Frac::whole(2))?, far)))
}

/// The slope's graph with its hole filled: g(h) is the chord's slope for
/// h ≠ 0 and g(0) = fill. Continuity at 0 against the tolerance t, for the
/// window |h| < delta, aims at g(0). The pick h = 0 is in every window and
/// always passes, since g(0) is within every tolerance of itself; only a
/// nonzero pick can lose.
/// - fill is the limit L: the rest is the limit's game, so the window works
///   when delta ≤ t and otherwise loses at [`window_failure`]'s pick.
/// - |L − fill| > t: every window loses at
///   h = min(delta, |L − fill| − t, [`reach`])/2, which is in the window and
///   the domain, with |g(h) − fill| ≥ |L − fill| − h > t.
///
/// A fill closer to L than t is refused: that tolerance does not tell the
/// two apart, and the chapter never plays it.
pub fn fill_failure(a: Frac, fill: Frac, t: Frac, delta: Frac) -> Result<Option<Frac>, String> {
    let t = window(t)?;
    if delta <= Frac::whole(0) {
        return Err(format!("a window is strictly positive, not {delta}"));
    }
    let limit = chord_limit(a)?;
    if fill == limit {
        return window_failure(a, t, delta);
    }
    let apart = size(limit.minus(fill)?)?;
    if apart <= t {
        return Err(format!(
            "a tolerance of {t} does not tell {fill} from the limit {limit}"
        ));
    }
    let room = apart.minus(t)?;
    Ok(Some(least(least(delta, room), reach(a)?).over(Frac::whole(2))?))
}

/// The window a defender who saw the challenger's nonzero pick h first
/// could answer with: |h|/2, too narrow to hold h. In the continuity game
/// h = 0 is also a pick, but it is in every window and passes anyway,
/// since g(0) is within any tolerance of itself: only a nonzero pick can
/// be shut out, and none is refused here but 0.
pub fn blind_window(h: Frac) -> Result<Frac, String> {
    if h.is_zero() {
        return Err("no window shuts out h = 0, which passes anyway".to_owned());
    }
    size(h)?.over(Frac::whole(2))
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

    /// The two strips differ exactly at their ends. The curve is above 1/2
    /// on (x_n, 1], which holds x = 1; the gap is at least 1/2 on [x_n, 1),
    /// which holds x_n and not x = 1, where the gap is zero. At n = 1 the
    /// crossing x_1 = 1/2 is a fraction, so both ends are tested exactly;
    /// for larger n, at fractions just either side of the crossing.
    #[test]
    fn the_two_strips_differ_only_at_their_ends() {
        let (zero, one) = (r(0, 1), r(1, 1));
        for n in [1, 2, 50, 1000] {
            assert!(above_half(one, n), "n = {n}: the curve is 1 at x = 1");
            assert!(!gap_at_least_half(one, n), "n = {n}: the gap is 0 at x = 1");
            assert_eq!(gap(1.0, n).unwrap(), 0.0);
            assert!(!above_half(zero, n) && !gap_at_least_half(zero, n));
        }
        assert_eq!(half_height_vs(1, 1, 2), Ordering::Equal);
        assert!(!above_half(HALF, 1), "at x_1 the curve is exactly 1/2, not above it");
        assert!(gap_at_least_half(HALF, 1), "at x_1 the gap is exactly 1/2");
        for n in [2, 10, 50, 100, 1000] {
            let d: u32 = half_height_rounded(n, 6).unwrap()[2..].parse().unwrap();
            let (left, right) = (r(2 * d - 1, 2_000_000), r(2 * d + 1, 2_000_000));
            assert_eq!(half_height_vs(n, left.num, left.den), Ordering::Greater, "n = {n}");
            assert_eq!(half_height_vs(n, right.num, right.den), Ordering::Less, "n = {n}");
            assert!(!above_half(left, n) && !gap_at_least_half(left, n), "n = {n}");
            assert!(above_half(right, n) && gap_at_least_half(right, n), "n = {n}");
        }
    }

    /// The width 1 − x_n lies strictly between ln 2/n − (ln 2)²/(2n²) and
    /// ln 2/n, falls as n grows, never reaches zero, and n times it climbs
    /// toward ln 2 from below: a limit, not a value any n attains.
    #[test]
    fn the_strip_narrows_like_ln2_over_n_and_is_never_empty() {
        let ln2 = std::f64::consts::LN_2;
        let mut last = (f64::INFINITY, 0.0);
        for n in [1, 2, 10, 50, 100, 200, 500, 897, 898, 1000, 10_000, 1_000_000] {
            let w = width(n).unwrap();
            let (lo, hi) = width_bounds(n).unwrap();
            assert!(lo < w && w < hi, "n = {n}: {lo} < {w} < {hi}");
            assert!((w - (1.0 - half_height(n).unwrap())).abs() < 1e-12, "n = {n}");
            let scaled = f64::from(n) * w;
            assert!(w < last.0 && scaled > last.1 && scaled < ln2, "n = {n}");
            assert!(ln2 - scaled < ln2 * ln2 / (2.0 * f64::from(n)), "n = {n}");
            last = (w, scaled);
        }
        let w = width(u32::MAX).unwrap();
        assert!(w > 0.0 && w < width_bounds(u32::MAX).unwrap().1);
        assert!(width(0).is_err() && width_bounds(0).is_err());
    }

    /// The printed widths are certified, and they are 1 minus the printed
    /// witness: no crossing is exactly halfway, so the two roundings agree.
    #[test]
    fn widths_are_printed_to_certified_digits() {
        let want = [
            (50, "0.013767"),
            (100, "0.006908"),
            (200, "0.003460"),
            (500, "0.001385"),
            (1000, "0.000693"),
        ];
        for (n, s) in want {
            assert_eq!(width_rounded(n, 1, 6).unwrap(), s, "n = {n}");
        }
        for (m, s) in [(10, "0.670"), (100, "0.691"), (1000, "0.693")] {
            assert_eq!(width_rounded(m, m, 3).unwrap(), s, "m = {m}");
        }
        for n in 1..=60 {
            let w: u32 = width_rounded(n, 1, 4).unwrap()[2..].parse().unwrap();
            let x: u32 = half_height_rounded(n, 4).unwrap()[2..].parse().unwrap();
            assert_eq!(w + x, 10_000, "n = {n}");
        }
        assert_eq!(width_rounded(1, 1, 1).unwrap(), "0.5");
        assert!(width_rounded(1, 3, 3).is_err(), "3 × 1/2 is not below 1");
        assert!(width_rounded(5, 0, 3).is_err() && width_rounded(5, 1, 0).is_err() && width_rounded(0, 1, 3).is_err());
    }

    fn q(num: i64, den: i64) -> Frac {
        Frac::new(num, den).unwrap()
    }

    #[test]
    fn signed_fractions_are_exact_and_refuse_zero_denominators() {
        assert_eq!(q(2, -4), q(-1, 2));
        assert_eq!(q(-3, -6).to_string(), "1/2");
        assert_eq!(q(4, 2).to_string(), "2");
        assert_eq!(q(-1, 10).to_string(), "-1/10");
        assert_eq!(q(1, 3).plus(q(1, 6)).unwrap(), q(1, 2));
        assert_eq!(q(1, 3).minus(q(1, 2)).unwrap(), q(-1, 6));
        assert_eq!(q(-2, 3).times(q(3, 4)).unwrap(), q(-1, 2));
        assert_eq!(q(1, 2).over(q(-1, 4)).unwrap(), q(-2, 1));
        assert_eq!(q(1, 4).negated().unwrap(), q(-1, 4));
        assert!(Frac::new(1, 0).is_err());
        assert!(q(1, 2).over(q(0, 5)).is_err(), "no division by zero");
        assert!(q(-1, 2) < q(-1, 3) && q(1, 3) < q(1, 2) && q(0, 1) == Frac::whole(0));
        assert!(q(11, 10).within(q(1, 1), q(1, 5)).unwrap());
        assert!(!q(6, 5).within(q(1, 1), q(1, 5)).unwrap(), "strictly within");
        assert!(!q(4, 5).within(q(1, 1), q(1, 5)).unwrap(), "strictly within, below too");
    }

    /// The modern step on the square: for every nonzero h tried, from either
    /// side, the change is 2a·h + h² and the chord's slope is exactly 2a + h;
    /// at h = 0 the change is zero, so the quotient would be 0/0, and it is
    /// refused.
    #[test]
    fn the_chord_slope_is_two_a_plus_h_for_every_nonzero_h() {
        for a in [BASE, q(0, 1), q(3, 4), q(-2, 3)] {
            let (change, slope) = chord_formula(a).unwrap();
            assert_eq!(change, [q(0, 1), a.times(q(2, 1)).unwrap(), q(1, 1)]);
            assert_eq!(slope, [chord_limit(a).unwrap(), q(1, 1)]);
            assert!(square_change(a, q(0, 1)).unwrap().is_zero(), "nothing changes at h = 0");
            assert!(chord_slope(a, q(0, 1)).is_err(), "0/0 is refused at a = {a}");
            for k in (-60..=60).filter(|&k| k != 0) {
                for den in [1, 7, 40, 1000] {
                    let h = q(k, den);
                    let want_change = change[1].times(h).unwrap().plus(h.times(h).unwrap()).unwrap();
                    assert_eq!(square_change(a, h).unwrap(), want_change, "a = {a}, h = {h}");
                    let want = slope[0].plus(h).unwrap();
                    assert_eq!(chord_slope(a, h).unwrap(), want, "a = {a}, h = {h}");
                }
            }
        }
        assert_eq!(chord_limit(BASE).unwrap(), q(1, 1));
        let slopes: Vec<String> = NUDGES
            .iter()
            .map(|&h| chord_slope(BASE, h).unwrap().to_string())
            .collect();
        assert_eq!(slopes, ["3/2", "5/4", "11/10", "1/2", "3/4", "9/10"]);
    }

    /// The limit, exactly: for each tolerance t, every rational h with
    /// 0 < |h| < t tried puts the slope strictly within t of 1, and at
    /// |h| = t it is exactly t away, so no wider window works.
    #[test]
    fn the_window_keeps_every_nonzero_h_within_the_tolerance() {
        let limit = chord_limit(BASE).unwrap();
        for t in TOLERANCES.into_iter().chain([q(1, 2), q(3, 1000), q(7, 3)]) {
            let w = window(t).unwrap();
            assert_eq!(w, t);
            for k in (-999..=999).filter(|&k| k != 0) {
                let h = w.times(q(k, 1000)).unwrap();
                assert!(
                    chord_slope(BASE, h).unwrap().within(limit, t).unwrap(),
                    "t = {t}, h = {h}"
                );
            }
            for edge in [w, w.negated().unwrap()] {
                let d = chord_slope(BASE, edge).unwrap().minus(limit).unwrap();
                assert!(d == t || d == t.negated().unwrap(), "at |h| = t the slope is t away");
            }
            let wider = w.times(q(1001, 1000)).unwrap();
            let outside = w.plus(wider).unwrap().over(q(2, 1)).unwrap();
            assert!(!chord_slope(BASE, outside).unwrap().within(limit, t).unwrap());
        }
        assert!(window(q(0, 1)).is_err() && window(q(-1, 10)).is_err());
    }

    /// Every chord drawn passes through both of its points and ends on the
    /// edge of the square; so does the line of slope 1, and the fan of
    /// lines through the base point.
    #[test]
    fn the_chords_are_clipped_to_the_square_through_their_points() {
        let (x0, y0) = (BASE.f64(), BASE.f64() * BASE.f64());
        let on_edge = |(x, y): (f64, f64)| {
            let near = |a: f64, b: f64| (a - b).abs() < 1e-12;
            (-1e-12..=1.0 + 1e-12).contains(&x)
                && (-1e-12..=1.0 + 1e-12).contains(&y)
                && (near(x, 0.0) || near(x, 1.0) || near(y, 0.0) || near(y, 1.0))
        };
        let mut slopes: Vec<f64> = NUDGES.iter().map(|&h| chord_slope(BASE, h).unwrap().f64()).collect();
        slopes.extend([0.75, 1.0, 1.25]);
        for (i, &s) in slopes.iter().enumerate() {
            let [a, b] = clip_unit(x0, y0, s).unwrap();
            assert!(a.0 < b.0 && on_edge(a) && on_edge(b), "slope {s}");
            assert!(a.0 <= x0 && x0 <= b.0);
            if let Some(&h) = NUDGES.get(i) {
                let x = BASE.plus(h).unwrap().f64();
                assert!((0.0..=1.0).contains(&x), "1/2 + {h} is in the plot");
                assert!(
                    a.0 - 1e-12 <= x && x <= b.0 + 1e-12,
                    "h = {h}: the far point is on the drawn chord"
                );
                assert!((y0 + s * (x - x0) - x * x).abs() < 1e-12);
            }
        }
        assert!(clip_unit(0.5, 2.0, 0.0).is_err());
        assert!(clip_unit(5.0, 0.0, 1.0).is_err());
    }

    /// The honest window loses to no pick; a wider one loses to the pick
    /// min((t + delta)/2, 1/2), which is inside it, keeps 1/2 + h in [0, 1]
    /// and whose slope misses the band: up to delta = 9/10 it is the
    /// midpoint, exactly 1/2 there, and from then on the domain's edge.
    #[test]
    fn a_wider_window_loses_at_a_pick_inside_the_domain() {
        let limit = chord_limit(BASE).unwrap();
        let t = CHALLENGE;
        let far = reach(BASE).unwrap();
        assert_eq!(far, q(1, 2));
        for k in 1..=10 {
            assert_eq!(window_failure(BASE, t, q(k, 100)).unwrap(), None, "delta = {k}/100");
        }
        for delta in (11..=300).map(|k| q(k, 100)).chain([q(101, 1000), q(7, 3), q(1000, 1)]) {
            let h = window_failure(BASE, t, delta).unwrap().unwrap();
            assert!(t < h && h < delta, "delta = {delta}: pick {h} inside the window");
            assert!(
                h <= far && (0.0..=1.0).contains(&BASE.plus(h).unwrap().f64()),
                "delta = {delta}: in [0, 1]"
            );
            assert!(
                !chord_slope(BASE, h).unwrap().within(limit, t).unwrap(),
                "delta = {delta}"
            );
        }
        assert_eq!(window_failure(BASE, t, WIDE).unwrap(), Some(q(3, 20)));
        assert_eq!(
            window_failure(BASE, t, q(9, 10)).unwrap(),
            Some(far),
            "the midpoint meets the edge"
        );
        assert_eq!(
            window_failure(BASE, t, q(91, 100)).unwrap(),
            Some(far),
            "past it, the edge"
        );
        assert_eq!(window_failure(BASE, t, q(89, 100)).unwrap(), Some(q(99, 200)));
        assert!(
            window_failure(BASE, far, q(1, 1)).is_err(),
            "t = 1/2 is outside the supported range t < 1/2"
        );
        // refused as unsupported, not impossible: h = 1/2 is a legal pick
        // exactly 1/2 away, so it misses the strict tolerance 1/2
        assert!(!chord_slope(BASE, far).unwrap().within(limit, far).unwrap());
        assert!(reach(q(0, 1)).is_err() && reach(q(1, 1)).is_err());
        assert_eq!(reach(q(1, 4)).unwrap(), q(1, 4));
        for h in PICKS {
            assert!(
                h.within(q(0, 1), t).unwrap() && !h.is_zero(),
                "{h} is a pick of the window"
            );
            assert!(chord_slope(BASE, h).unwrap().within(limit, t).unwrap(), "{h}");
        }
        assert!(LUCKY < WIDE && chord_slope(BASE, LUCKY).unwrap().within(limit, t).unwrap());
        assert!(window_failure(BASE, q(0, 1), WIDE).is_err() && window_failure(BASE, t, q(0, 1)).is_err());
    }

    /// Filled with the limit, the graph is continuous at 0: the honest
    /// window holds h = 0 and every pick. Filled with 3/2, every window
    /// loses at h = min(delta, 2/5)/2, inside it and the domain and more
    /// than t away. (These are checks at chosen windows and picks; the
    /// argument for every window is the triangle inequality in
    /// `fill_failure`'s doc.)
    #[test]
    fn a_hole_filled_wrong_loses_every_window() {
        let t = CHALLENGE;
        let [good, bad] = FILLS;
        assert_eq!(good, chord_limit(BASE).unwrap());
        assert_eq!(fill_failure(BASE, good, t, t).unwrap(), None);
        assert!(good.within(good, t).unwrap(), "g(0) is within t of itself");
        for k in (-99..=99).filter(|&k| k != 0) {
            let h = t.times(q(k, 100)).unwrap();
            assert!(chord_slope(BASE, h).unwrap().within(good, t).unwrap(), "h = {h}");
        }
        assert_eq!(fill_failure(BASE, good, t, WIDE).unwrap(), Some(q(3, 20)));
        // h = 0 is in every window, and passes against any fill: only
        // nonzero picks lose
        assert!(bad.within(bad, t).unwrap());
        let gap = q(3, 10);
        for delta in [
            q(1, 1000),
            q(1, 40),
            q(1, 20),
            t,
            q(2, 5),
            q(1, 2),
            q(1, 1),
            q(7, 3),
            q(1000, 1),
        ] {
            let h = fill_failure(BASE, bad, t, delta).unwrap().unwrap();
            assert!(q(0, 1) < h && h < delta && h <= q(1, 5), "delta = {delta}: pick {h}");
            let miss = chord_slope(BASE, h).unwrap().minus(bad).unwrap();
            assert!(
                miss <= gap.negated().unwrap(),
                "delta = {delta}: at least 3/10 below 1.5"
            );
        }
        assert_eq!(fill_failure(BASE, bad, t, t).unwrap(), Some(q(1, 20)));
        assert_eq!(
            fill_failure(BASE, bad, t, q(2, 5)).unwrap(),
            Some(q(1, 5)),
            "the widest pick, g = 1.2"
        );
        // a fill far off is capped by the domain: min(7/3, 19/10, 1/2)/2
        assert_eq!(fill_failure(BASE, q(3, 1), t, q(7, 3)).unwrap(), Some(q(1, 4)));
        assert!(
            fill_failure(BASE, q(11, 10), t, t).is_err(),
            "t does not tell 11/10 from 1"
        );
    }

    /// A defender who saw the pick first could answer with a window too
    /// narrow to hold it.
    #[test]
    fn a_blind_window_never_holds_the_pick() {
        for h in PICKS.into_iter().chain([LUCKY, q(-3, 7)]) {
            let w = blind_window(h).unwrap();
            assert!(
                q(0, 1) < w && !h.within(q(0, 1), w).unwrap(),
                "{h} is outside 0 < |h| < {w}"
            );
        }
        assert_eq!(blind_window(LUCKY).unwrap(), q(1, 40));
        assert!(blind_window(q(0, 1)).is_err());
    }

    /// Continuity of x² at x = 1 on [0, 1] against 1/2: the window is
    /// 2^(−1/2) < x ≤ 1. Rational points just inside it have x² > 1/2,
    /// just outside do not, and x = 1 itself does; x² ≤ 1 on the domain, so
    /// |x² − 1| < 1/2 exactly on the window.
    #[test]
    fn the_square_is_continuous_at_the_end_of_its_domain() {
        assert_eq!(half_height_rounded(2, 4).unwrap(), "0.7071");
        assert_eq!(width_rounded(2, 1, 4).unwrap(), "0.2929");
        let d: u32 = half_height_rounded(2, 6).unwrap()[2..].parse().unwrap();
        let (outside, inside) = (r(2 * d - 1, 2_000_000), r(2 * d + 1, 2_000_000));
        assert!(!above_half(outside, 2) && above_half(inside, 2));
        assert!(above_half(r(1, 1), 2), "x = 1 is in the window: 1² = 1");
        // across the window, from just inside its edge up to just below 1
        let start = 2 * d + 1;
        for k in 0..=100 {
            let x = r(start + k * (1_999_999 - start) / 100, 2_000_000);
            assert!(above_half(x, 2), "{x}");
        }
    }

    /// The plot is 1296 pixels wide in the 4K picture; the strip is narrower
    /// than one of them exactly when x_n > 1295/1296, from n = 898 on.
    #[test]
    fn the_strip_goes_below_a_pixel_at_898() {
        let pixel = r(1295, 1296);
        assert_eq!(first_within(pixel, HALF).unwrap(), 898);
        assert_eq!(half_height_vs(898, 1295, 1296), Ordering::Greater);
        assert_eq!(half_height_vs(897, 1295, 1296), Ordering::Less);
        assert!(width(898).unwrap() < 1.0 / 1296.0 && width(897).unwrap() > 1.0 / 1296.0);
    }
}
