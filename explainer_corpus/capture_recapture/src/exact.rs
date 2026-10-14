//! Exact ratios as correctly rounded floats, so a float fact equals its
//! oracle key (Python's `float(Fraction)`, spelled by `facts::py`) string
//! for string: `./x capture_recapture facts-check` compares strings.
//!
//! The chances chapters 09–11 show are ratios of integers far beyond u128
//! (C(10,000, 1,000) has 4,700 bits). [`Dd`] carries a double-double, about
//! 106 bits, built from error-free transformations (Dekker and Knuth, the
//! product's error by FMA), with its own power-of-two exponent, so a long
//! product of small ratios neither loses precision nor overflows nor
//! underflows. [`Dd::to_f64`] rounds once, to nearest, into the normal or
//! subnormal range: the same float Python's `float(Fraction)` gives,
//! unless the exact value lies within about 2⁻⁹⁰ of a rounding midpoint
//! (the facts check would catch that).

use std::cmp::Ordering;

/// 2^e as a float, for −1022 ≤ e ≤ 1023 (exact).
fn pow2(e: i32) -> f64 {
    debug_assert!((-1022..=1023).contains(&e));
    f64::from_bits(((e + 1023) as u64) << 52)
}

/// x · 2^e for any e, in steps that stay exact wherever the result is
/// representable (0 below the subnormals, infinity above the floats).
fn ldexp(mut x: f64, mut e: i32) -> f64 {
    while e > 1023 {
        x *= pow2(1023);
        e -= 1023;
    }
    while e < -1022 {
        x *= pow2(-1022);
        e += 1022;
        if x == 0.0 {
            return x;
        }
    }
    x * pow2(e)
}

/// The binary exponent of a nonzero finite float: |x| = m · 2^e with
/// 1 ≤ m < 2 (subnormals included).
fn exponent_of(x: f64) -> i32 {
    let bits = x.to_bits();
    let biased = ((bits >> 52) & 0x7ff) as i32;
    if biased == 0 {
        // subnormal: the leading one is somewhere in the 52 fraction bits
        let frac = bits & ((1u64 << 52) - 1);
        -1011 - frac.leading_zeros() as i32
    } else {
        biased - 1023
    }
}

/// a + b = s + e exactly (Knuth's two-sum).
fn two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    let bb = s - a;
    (s, (a - (s - bb)) + (b - bb))
}

/// a + b = s + e exactly when |a| ≥ |b| (Dekker's fast two-sum).
fn quick_two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    (s, b - (s - a))
}

/// a · b = p + e exactly (the error by one fused multiply-add).
fn two_prod(a: f64, b: f64) -> (f64, f64) {
    let p = a * b;
    (p, a.mul_add(b, -p))
}

/// A real as (hi + lo) · 2^exp with 1 ≤ |hi| < 2 (or zero) and hi + lo in
/// canonical form (hi is hi + lo rounded to nearest). Public values are
/// never negative; intermediate remainders may be.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dd {
    hi: f64,
    lo: f64,
    exp: i32,
}

impl Dd {
    pub const ZERO: Dd = Dd {
        hi: 0.0,
        lo: 0.0,
        exp: 0,
    };

    /// An integer, exactly (a u64 needs at most 64 bits; hi holds 53, the
    /// remainder fits lo exactly).
    pub fn int(n: u64) -> Self {
        let hi = n as f64;
        let lo = (i128::from(n) - hi as i128) as f64;
        Self { hi, lo, exp: 0 }.norm()
    }

    /// A u128 to about 106 bits (C(100, 50) has 97, so exactly).
    pub fn int128(n: u128) -> Self {
        let hi = n as f64;
        let lo = (n as i128 - hi as i128) as f64;
        Self { hi, lo, exp: 0 }.norm()
    }

    /// Canonical form, with the exponent of hi moved into `exp`.
    fn norm(self) -> Self {
        let (hi, lo) = quick_two_sum(self.hi, self.lo);
        if hi == 0.0 {
            return Self::ZERO;
        }
        let e = exponent_of(hi);
        Self {
            hi: ldexp(hi, -e),
            lo: ldexp(lo, -e),
            exp: self.exp + e,
        }
    }

    pub fn is_zero(self) -> bool {
        self.hi == 0.0
    }

    /// 2^e, exactly, for any e.
    pub fn pow2(e: i32) -> Self {
        Self {
            hi: 1.0,
            lo: 0.0,
            exp: e,
        }
    }

    /// 10^e, to about 106 bits.
    pub fn pow10(e: u32) -> Self {
        (0..e).fold(Self::int(1), |acc, _| acc.mul_int(10))
    }

    /// self ≥ o (both non-negative).
    pub fn ge(self, o: Self) -> bool {
        if o.is_zero() {
            return true;
        }
        if self.is_zero() {
            return false;
        }
        (self.exp, self.hi, self.lo) >= (o.exp, o.hi, o.lo)
    }

    /// self · b for a float b (an integer below 2⁵³ is exact).
    pub fn mul_f(self, b: f64) -> Self {
        let (p, e) = two_prod(self.hi, b);
        Self {
            hi: p,
            lo: e + self.lo * b,
            exp: self.exp,
        }
        .norm()
    }

    /// self / b for a nonzero float b.
    pub fn div_f(self, b: f64) -> Self {
        let q1 = self.hi / b;
        let (p, e) = two_prod(q1, b);
        let (s, f) = two_sum(self.hi, -p);
        let q2 = (s + ((f - e) + self.lo)) / b;
        Self {
            hi: q1,
            lo: q2,
            exp: self.exp,
        }
        .norm()
    }

    pub fn mul_int(self, n: u64) -> Self {
        assert!(n < 1 << 53, "mul_int takes integers a float holds exactly");
        self.mul_f(n as f64)
    }

    pub fn div_int(self, n: u64) -> Self {
        assert!(
            n > 0 && n < 1 << 53,
            "div_int takes nonzero integers a float holds exactly"
        );
        self.div_f(n as f64)
    }

    pub fn mul(self, o: Self) -> Self {
        let (p, e) = two_prod(self.hi, o.hi);
        Self {
            hi: p,
            lo: e + (self.hi * o.lo + self.lo * o.hi),
            exp: self.exp + o.exp,
        }
        .norm()
    }

    pub fn div(self, o: Self) -> Self {
        assert!(!o.is_zero(), "Dd division by zero");
        if self.is_zero() {
            return Self::ZERO;
        }
        // divide the mantissas: q1 from the leading parts, q2 from the
        // remainder a − q1 · b computed to double-double precision
        let a = Self { exp: 0, ..self };
        let b = Self { exp: 0, ..o };
        let q1 = a.hi / b.hi;
        let r = a.add_signed(b.mul_f(q1), -1.0);
        let q2 = ldexp(r.hi + r.lo, r.exp) / b.hi;
        Self {
            hi: q1,
            lo: q2,
            exp: self.exp - o.exp,
        }
        .norm()
    }

    pub fn add(self, o: Self) -> Self {
        self.add_signed(o, 1.0)
    }

    /// self − o, which must not be negative.
    pub fn sub(self, o: Self) -> Self {
        let r = self.add_signed(o, -1.0);
        assert!(r.hi >= 0.0, "Dd::sub went negative");
        r
    }

    /// self + sign · o, aligned to the larger exponent.
    fn add_signed(self, o: Self, sign: f64) -> Self {
        if o.is_zero() {
            return self;
        }
        if self.is_zero() {
            return Self {
                hi: sign * o.hi,
                lo: sign * o.lo,
                exp: o.exp,
            };
        }
        let base = self.exp.max(o.exp);
        let (ah, al) = (ldexp(self.hi, self.exp - base), ldexp(self.lo, self.exp - base));
        let (bh, bl) = (sign * ldexp(o.hi, o.exp - base), sign * ldexp(o.lo, o.exp - base));
        let (s, e) = two_sum(ah, bh);
        let (t, f) = two_sum(al, bl);
        let (s, e) = quick_two_sum(s, e + t);
        Self {
            hi: s,
            lo: e + f,
            exp: base,
        }
        .norm()
    }

    /// The nearest float (ties to even), normal or subnormal; 0 below half
    /// the smallest subnormal, infinity past the largest float.
    pub fn to_f64(self) -> f64 {
        assert!(self.hi >= 0.0, "Dd values are never negative");
        if self.is_zero() {
            return 0.0;
        }
        if self.exp > 1023 {
            return f64::INFINITY;
        }
        if self.exp >= -1022 {
            // hi already is hi + lo rounded to nearest (canonical form);
            // scaling by a power of two in the normal range is exact
            return self.hi * pow2(self.exp);
        }
        // subnormal: count units of 2^-1074 and round that count once
        let shift = self.exp + 1074;
        if shift < -1 {
            return 0.0;
        }
        let units = ldexp(self.hi, shift);
        let floor = units.floor();
        let frac = (units - floor) + ldexp(self.lo, shift);
        let n = match frac.partial_cmp(&0.5) {
            Some(Ordering::Greater) => floor + 1.0,
            Some(Ordering::Less) => floor,
            _ if floor % 2.0 == 0.0 => floor,
            _ => floor + 1.0,
        };
        f64::from_bits(n as u64)
    }

    /// The decimal scientific form m · 10^e with 1 ≤ m < 10, m to about
    /// 13 significant digits (for numbers like 5.35 × 10⁻⁴³⁰).
    pub fn sci10(self) -> (f64, i32) {
        assert!(self.hi > 0.0, "sci10 of a positive value");
        let log10 = (self.hi.log10() + self.lo / (self.hi * std::f64::consts::LN_10))
            + f64::from(self.exp) * std::f64::consts::LOG10_2;
        let mut e = log10.floor() as i32;
        let mut m = 10f64.powf(log10 - f64::from(e));
        if m >= 10.0 {
            m /= 10.0;
            e += 1;
        }
        (m, e)
    }
}

/// C(n, k) as a double-double (exact to about 106 bits, any size).
pub fn choose_dd(n: u64, k: u64) -> Dd {
    if k > n {
        return Dd::ZERO;
    }
    let k = k.min(n - k);
    (0..k).fold(Dd::int(1), |acc, i| acc.mul_int(n - i).div_int(i + 1))
}

/// P_N(K = k) = C(M, k) C(N − M, n − k) / C(N, n), the netful count over
/// all netfuls (chapter 04's rule), as a double-double.
pub fn hypergeometric(lake: u64, (m, n, k): (u64, u64, u64)) -> Dd {
    if k > m || k > n || n - k > lake.saturating_sub(m) || n > lake {
        return Dd::ZERO;
    }
    choose_dd(m, k).mul(choose_dd(lake - m, n - k)).div(choose_dd(lake, n))
}

/// Every P_N(K = k), k = 0 … min(M, n), by Feller's step from one count to
/// the next: P(k + 1)/P(k) = (M − k)(n − k) / ((k + 1)(N − M − n + k + 1)).
pub fn hypergeometric_pmf(lake: u64, m: u64, n: u64) -> Vec<Dd> {
    let top = m.min(n);
    let low = n.saturating_sub(lake - m);
    let mut out = vec![Dd::ZERO; top as usize + 1];
    let mut p = hypergeometric(lake, (m, n, low));
    for k in low..=top {
        out[k as usize] = p;
        if k < top {
            p = p.mul_int((m - k) * (n - k)).div_int((k + 1) * (lake - m - n + k + 1));
        }
    }
    out
}

/// The plain soup guess and Chapman's, exactly, for one experiment design
/// (chapter 10; oracle `af21_small_study_50`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EstimateLaw {
    /// P(K = 0): the second catch holds no marked fish.
    pub p0: f64,
    /// 1 / P(K = 0): "one time in …".
    pub one_in: f64,
    /// E[Mn/K | K > 0]: the plain guess, over the experiments that caught any.
    pub plain_mean: f64,
    pub plain_bias_pct: f64,
    /// E[(M + 1)(n + 1)/(K + 1) − 1]: Chapman's guess, over every experiment.
    pub chapman_mean: f64,
    pub chapman_bias_pct: f64,
}

/// The exact law of both guesses at lake size N with M marked and n caught,
/// from the netful chances (every term exact to about 106 bits, each
/// result rounded once).
pub fn estimate_law(lake: u64, m: u64, n: u64) -> EstimateLaw {
    let pmf = hypergeometric_pmf(lake, m, n);
    let p0 = pmf[0];
    let any = Dd::int(1).sub(p0);
    let plain = (1..pmf.len())
        .fold(Dd::ZERO, |s, k| s.add(pmf[k].mul_int(m * n).div_int(k as u64)))
        .div(any);
    let a = (m + 1) * (n + 1);
    // (M + 1)(n + 1)/(k + 1) − 1 = (a − k − 1)/(k + 1), positive for every k ≤ n
    let chapman = (0..pmf.len()).fold(Dd::ZERO, |s, k| {
        s.add(pmf[k].mul_int(a - k as u64 - 1).div_int(k as u64 + 1))
    });
    let truth = Dd::int(lake);
    let pct = |mean: Dd| -> f64 {
        if mean.ge(truth) {
            mean.sub(truth).mul_int(100).div_int(lake).to_f64()
        } else {
            -truth.sub(mean).mul_int(100).div_int(lake).to_f64()
        }
    };
    EstimateLaw {
        p0: p0.to_f64(),
        one_in: Dd::int(1).div(p0).to_f64(),
        plain_mean: plain.to_f64(),
        plain_bias_pct: pct(plain),
        chapman_mean: chapman.to_f64(),
        chapman_bias_pct: pct(chapman),
    }
}

/// Feller's comparison of a lake size with the one just below it:
/// L(N)/L(N − 1) = (N − M)(N − n) / (N(N − M − n + k)), as the two exact
/// integers (07_2's 529 and 528).
pub fn feller_ratio(lake: u64, (m, n, k): (u64, u64, u64)) -> (u64, u64) {
    ((lake - m) * (lake - n), lake * (lake + k - m - n))
}

/// Scores relative to the top one, L(N)/L(top), for N in `lo ..= hi`,
/// walked from the top in logs with Feller's ratio, one step per lake size
/// (07_1's lower-third: how code walks the curve). Zero below the smallest
/// possible lake.
pub fn relative_scores((m, n, k): (u64, u64, u64), lo: u64, hi: u64) -> Vec<f64> {
    assert!(k > 0 && lo <= hi);
    let smallest = m + n - k;
    let top = (m * n / k).max(smallest);
    let mut log = std::collections::HashMap::new();
    log.insert(top, 0.0f64);
    let mut l = 0.0;
    for lake in top + 1..=hi.max(top) {
        let (a, b) = feller_ratio(lake, (m, n, k));
        l += (a as f64).ln() - (b as f64).ln();
        log.insert(lake, l);
    }
    let mut l = 0.0;
    for lake in (lo.max(smallest + 1)..=top).rev() {
        // L(N − 1) = L(N) · den/num
        let (a, b) = feller_ratio(lake, (m, n, k));
        l += (b as f64).ln() - (a as f64).ln();
        log.insert(lake - 1, l);
    }
    (lo..=hi)
        .map(|lake| if lake < smallest { 0.0 } else { log[&lake].exp() })
        .collect()
}

#[cfg(test)]
mod tests {
    use explainer_kit::choose;

    use super::*;
    use crate::facts::py;

    #[test]
    fn the_facts_spell_floats_as_python_does() {
        // facts::py against what CPython's repr() prints
        let cases: [(f64, &str); 14] = [
            (10_088.586_031_204_479, "10088.586031204479"),
            (10_000.0, "10000.0"),
            (1e-4, "0.0001"),
            (1e-5, "1e-05"),
            (1.5e-7, "1.5e-07"),
            (1e16, "1e+16"),
            (999_999_999_999_999.9, "999999999999999.9"),
            (5e-324, "5e-324"),
            (2.225_073_858_507_201_4e-308, "2.2250738585072014e-308"),
            (-7.275_957_614_183_426e-12, "-7.275957614183426e-12"),
            (123.0, "123.0"),
            (0.071_976_991_884_843_12, "0.07197699188484312"),
            (8.960_120_782_668_056, "8.960120782668056"),
            (f64::INFINITY, "inf"),
        ];
        for (x, want) in cases {
            assert_eq!(py(x), want, "{x:e}");
        }
    }

    #[test]
    fn integers_and_small_ratios_round_exactly() {
        for n in [0u64, 1, 3, 1 << 53, (1 << 53) + 1, u64::MAX, 12_345_678_901_234_567_890] {
            let d = Dd::int(n);
            assert_eq!(d.to_f64(), n as f64, "{n}");
        }
        assert_eq!(Dd::int128(choose(100, 50)).to_f64(), choose(100, 50) as f64);
        // a ratio of exact floats is correctly rounded by IEEE division
        for (a, b) in [(1u64, 3u64), (90_000, 1_111), (2, 7), (1_000_000, 118), (22, 7)] {
            assert_eq!(Dd::int(a).div_int(b).to_f64(), a as f64 / b as f64, "{a}/{b}");
            assert_eq!(Dd::int(a).div(Dd::int(b)).to_f64(), a as f64 / b as f64, "{a}/{b} dd");
        }
    }

    #[test]
    fn exact_binomial_ratios_match_u128_arithmetic() {
        // within u128, every netful chance by Dd equals the exact ratio
        // rounded by an independent route (two exact u128 halves)
        for (lake, m, n) in [(20u64, 5u64, 4u64), (40, 10, 10), (60, 10, 10), (33, 10, 10)] {
            let total = choose(lake, n);
            let pmf = hypergeometric_pmf(lake, m, n);
            let mut sum = Dd::ZERO;
            for (k, p) in pmf.iter().enumerate() {
                let count = choose(m, k as u64) * choose(lake - m, n - k as u64);
                let want = Dd::int128(count).div(Dd::int128(total)).to_f64();
                assert_eq!(p.to_f64(), want, "N {lake} M {m} n {n} k {k}");
                assert_eq!(hypergeometric(lake, (m, n, k as u64)).to_f64(), want);
                sum = sum.add(*p);
            }
            assert!((sum.to_f64() - 1.0).abs() < 1e-15, "the chances add to one");
        }
        // A-F3: the toy lake's five chances, exactly as the fact sheet prints them
        let p: Vec<f64> = hypergeometric_pmf(20, 5, 4).iter().map(|d| d.to_f64()).collect();
        assert_eq!(p[0], 91.0 / 323.0);
        assert_eq!(p[1], 455.0 / 969.0);
        assert_eq!(p[4], 1.0 / 969.0);
    }

    #[test]
    fn tiny_values_keep_their_digits_and_round_into_subnormals() {
        // P_1900(K = 100) = 5.3515… × 10⁻⁴³⁰ (A-F11): far below any float
        let p = hypergeometric(1_900, (1_000, 1_000, 100));
        let (m, e) = p.sci10();
        assert_eq!(e, -430);
        assert!((m - 5.3515).abs() < 1e-4, "{m}");
        assert_eq!(p.to_f64(), 0.0);
        // the smallest subnormal survives, half of it rounds to even (zero)
        let tiny = Dd::pow2(-1074);
        assert_eq!(tiny.to_f64(), 5e-324);
        assert_eq!(tiny.div_int(2).to_f64(), 0.0);
        assert_eq!(tiny.mul_int(3).div_int(2).to_f64(), 1e-323); // 1.5 units → 2
        assert_eq!(Dd::int(1).div(choose_dd(2_000, 1_000)).to_f64(), 0.0);
    }

    #[test]
    fn the_small_study_law_matches_the_fact_sheet() {
        // A-F21 (N = 1,000, M = n = 50): 7.20% (1 in 13.9), Chapman 935.2
        // (−6.48%), the plain guess given any caught twice 1,241 (+24.1%)
        let law = estimate_law(1_000, 50, 50);
        assert_eq!(py(law.p0), "0.07197699188484312");
        assert_eq!(py(law.one_in), "13.893328601449646");
        assert_eq!(py(law.chapman_mean), "935.2207073036412");
        assert_eq!(py(law.chapman_bias_pct), "-6.477929269635881");
        assert_eq!(py(law.plain_mean), "1241.2389574266258");
        assert_eq!(py(law.plain_bias_pct), "24.12389574266259");
        // Chapman's mean by the A-F18 closed form, N − (N+1) C(N−M, n+1)/C(N+1, n+1)
        let bias = Dd::int(1_001).mul(choose_dd(950, 51)).div(choose_dd(1_001, 51));
        assert_eq!(Dd::int(1_000).sub(bias).to_f64(), law.chapman_mean);
    }

    #[test]
    fn feller_ratio_walks_the_score_curve() {
        // 07_2: 529/528 up, 576/578 down, each the exact ratio of scores
        assert_eq!(feller_ratio(33, (10, 10, 3)), (529, 528));
        assert_eq!(feller_ratio(34, (10, 10, 3)), (576, 578));
        for lake in 18..=60u64 {
            let (a, b) = feller_ratio(lake, (10, 10, 3));
            let exact = hypergeometric(lake, (10, 10, 3)).div(hypergeometric(lake - 1, (10, 10, 3)));
            assert_eq!(exact.to_f64(), a as f64 / b as f64, "N {lake}");
        }
    }

    #[test]
    fn the_walk_from_the_top_matches_exact_scores() {
        let rel = relative_scores((10, 10, 3), 15, 60);
        let top = hypergeometric(33, (10, 10, 3)).to_f64();
        for (i, r) in rel.iter().enumerate() {
            let lake = 15 + i as u64;
            let want = hypergeometric(lake, (10, 10, 3)).to_f64() / top;
            assert!((r - want).abs() < 1e-12, "N {lake}: {r} vs {want}");
        }
        assert_eq!(rel[0], 0.0);
        assert_eq!(rel[1], 0.0, "16 < 17, the smallest lake");
        // Feller's: the tie at 9,999 / 10,000, falling on both sides
        let f = relative_scores((1_000, 1_000, 100), 9_998, 10_001);
        assert_eq!(f[1], 1.0);
        assert_eq!(f[2], 1.0);
        assert!(f[0] < 1.0 && f[3] < 1.0);
        // and far out, against exact double-doubles (12,000: A-F12's 0.152)
        let wide = relative_scores((1_000, 1_000, 100), 1_900, 12_000);
        let exact = hypergeometric(12_000, (1_000, 1_000, 100))
            .div(hypergeometric(10_000, (1_000, 1_000, 100)))
            .to_f64();
        let got = wide[12_000 - 1_900];
        assert!((got / exact - 1.0).abs() < 1e-9, "{got} vs {exact}");
        assert_eq!(wide[0], 0.0, "the smallest lake's score underflows (5.4e-430)");
    }

    #[test]
    fn subtraction_and_addition_agree_with_exact_integers() {
        let a = Dd::int(1_000_000_007).mul_int(1_000_000_009);
        let b = Dd::int(999_999_937).mul_int(1_000_000_000);
        let want = 1_000_000_007u128 * 1_000_000_009 - 999_999_937 * 1_000_000_000;
        assert_eq!(a.sub(b).to_f64(), want as f64);
        assert_eq!(a.add(b).sub(b).sub(a).to_f64(), 0.0);
    }
}
