//! Hyperbolic kernels — FDLIBM `s_sinh` / `s_cosh` / `s_tanh`, built on
//! this crate's own [`exp`]/[`expm1`], bit-identical on every certified
//! target.
//!
//! * [`sinh`] — `expm1`-based for `|x| < 22` (with the `E/(E+1)`
//!   correction), `0.5*exp(|x|)` up to `log(maxdouble)`, then the
//!   `half*exp(half*x)` squared form up to the overflow threshold
//!   (~710.4758600739439); odd, sign applied via the `±0.5` factor.
//! * [`cosh`] — `expm1`-based below `0.5*ln2`, `(exp+1/exp)/2` below 22,
//!   large-argument paths as `sinh`; even, `cosh(0) = 1` exactly.
//! * [`tanh`] — `expm1`-based branches below 22 (separate `|x| >= 1` and
//!   `|x| < 1` forms), `±1` beyond; odd bitwise.
//!
//! Design accuracy bounds (verified downstream against the committed
//! mpmath vectors): `sinh`, `cosh`, `tanh` < 2 ulp.

use crate::bits::{F64x, SIMD_LANES, U64x, hi, lo};
use crate::exp_log::{exp, exp_lanes, expm1, expm1_lanes};
use std::simd::Select;
use std::simd::cmp::{SimdPartialEq, SimdPartialOrd};
use std::simd::num::SimdFloat;

const HUGE: f64 = 1.0e300;
const SHUGE: f64 = 1.0e307;
const TINY: f64 = 1.0e-300;

/// Hyperbolic sine — FDLIBM `s_sinh`.
///
/// Special values: `sinh(±0) = ±0`, `sinh(±inf) = ±inf`,
/// `sinh(NaN) = NaN`; overflows to `±inf` above ~710.476. Odd. Design
/// accuracy < 2 ulp.
#[must_use]
pub fn sinh(x: f64) -> f64 {
    let jx = hi(x) as i32;
    let ix = jx & 0x7fff_ffff;

    // x is inf or NaN.
    if ix >= 0x7ff0_0000 {
        return x + x;
    }

    let h = if jx < 0 { -0.5 } else { 0.5 };

    // |x| in [0, 22): sign(x)*0.5*(E + E/(E+1)), E = expm1(|x|).
    if ix < 0x4036_0000 {
        if ix < 0x3e30_0000 {
            return x; // |x| < 2^-28: sinh(tiny) = tiny
        }
        let t = expm1(x.abs());
        if ix < 0x3ff0_0000 {
            return h * (2.0 * t - t * t / (t + 1.0));
        }
        return h * (t + t / (t + 1.0));
    }

    // |x| in [22, log(maxdouble)]: 0.5*exp(|x|).
    if ix < 0x4086_2e42 {
        return h * exp(x.abs());
    }

    // |x| in (log(maxdouble), overflow threshold 710.4758600739439...]:
    // h*exp(0.5|x|) * exp(0.5|x|).
    let lx = lo(x);
    if ix < 0x4086_33ce || (ix == 0x4086_33ce && lx <= 0x8fb9_f87d) {
        let w = exp(0.5 * x.abs());
        let t = h * w;
        return t * w;
    }

    // |x| > overflow threshold: sinh(x) overflows.
    x * SHUGE
}

/// Hyperbolic cosine — FDLIBM `s_cosh`.
///
/// Special values: `cosh(±0) = 1` exactly, `cosh(±inf) = +inf`,
/// `cosh(NaN) = NaN`; overflows to `+inf` above ~710.476. Even. Design
/// accuracy < 2 ulp.
#[must_use]
pub fn cosh(x: f64) -> f64 {
    let ix = (hi(x) as i32) & 0x7fff_ffff;

    // x is inf or NaN.
    if ix >= 0x7ff0_0000 {
        return x * x;
    }

    // |x| in [0, 0.5*ln2): 1 + expm1(|x|)^2 / (2*exp(|x|)).
    if ix < 0x3fd6_2e43 {
        let t = expm1(x.abs());
        let w = 1.0 + t;
        if ix < 0x3c80_0000 {
            return w; // cosh(tiny) = 1
        }
        return 1.0 + (t * t) / (w + w);
    }

    // |x| in [0.5*ln2, 22): (exp(|x|) + 1/exp(|x|)) / 2.
    if ix < 0x4036_0000 {
        let t = exp(x.abs());
        return 0.5 * t + 0.5 / t;
    }

    // |x| in [22, log(maxdouble)]: 0.5*exp(|x|).
    if ix < 0x4086_2e42 {
        return 0.5 * exp(x.abs());
    }

    // |x| in (log(maxdouble), overflow threshold].
    let lx = lo(x);
    if ix < 0x4086_33ce || (ix == 0x4086_33ce && lx <= 0x8fb9_f87d) {
        let w = exp(0.5 * x.abs());
        let t = 0.5 * w;
        return t * w;
    }

    // |x| > overflow threshold: cosh(x) overflows.
    HUGE * HUGE
}

/// Hyperbolic tangent — FDLIBM `s_tanh`.
///
/// Special values: `tanh(±0) = ±0`, `tanh(±inf) = ±1` exactly,
/// `tanh(NaN) = NaN`; saturates at `±1` beyond `|x| > 22`. Odd bitwise.
/// Design accuracy < 2 ulp.
#[must_use]
pub fn tanh(x: f64) -> f64 {
    let jx = hi(x) as i32;
    let ix = jx & 0x7fff_ffff;

    // x is inf or NaN: tanh(±inf) = ±1, tanh(NaN) = NaN.
    if ix >= 0x7ff0_0000 {
        if jx >= 0 {
            return 1.0 / x + 1.0;
        }
        return 1.0 / x - 1.0;
    }

    let z;
    if ix < 0x4036_0000 {
        // |x| < 22
        if ix < 0x3c80_0000 {
            return x * (1.0 + x); // |x| < 2^-55: tanh(small) = small
        }
        if ix >= 0x3ff0_0000 {
            // |x| >= 1: 1 - 2/(expm1(2|x|) + 2)
            let t = expm1(2.0 * x.abs());
            z = 1.0 - 2.0 / (t + 2.0);
        } else {
            // |x| < 1: -expm1(-2|x|) / (expm1(-2|x|) + 2)
            let t = expm1(-2.0 * x.abs());
            z = -t / (t + 2.0);
        }
    } else {
        // |x| >= 22: ±1 (up to rounding).
        z = 1.0 - TINY;
    }
    if jx >= 0 { z } else { -z }
}

/// Apply [`sinh`] to a slice in place.
///
/// Ordinary `|x| < 22` chunks use the same `expm1` reduction as the scalar
/// definition in independent lanes. Tiny, large, and exceptional chunks
/// remain scalar so overflow and signed-special behavior stays exact.
pub fn sinh_slice(values: &mut [f64]) {
    let mut index = 0;
    while index + SIMD_LANES <= values.len() {
        let input = F64x::from_slice(&values[index..]);
        let absolute_high = (input.to_bits() >> U64x::splat(32)) & U64x::splat(0x7fff_ffff);
        let ordinary = absolute_high.simd_ge(U64x::splat(0x3e30_0000))
            & absolute_high.simd_lt(U64x::splat(0x4036_0000));
        if ordinary.all() {
            sinh_lanes(input).copy_to_slice(&mut values[index..index + SIMD_LANES]);
        } else {
            for value in &mut values[index..index + SIMD_LANES] {
                *value = sinh(*value);
            }
        }
        index += SIMD_LANES;
    }
    for value in &mut values[index..] {
        *value = sinh(*value);
    }
}

#[inline]
fn sinh_lanes(input: F64x) -> F64x {
    let bits = input.to_bits();
    let absolute_high = (bits >> U64x::splat(32)) & U64x::splat(0x7fff_ffff);
    let negative = (bits >> U64x::splat(63)).simd_eq(U64x::splat(1));
    let h = negative.select(F64x::splat(-0.5), F64x::splat(0.5));
    let t = expm1_lanes(input.abs());
    let small = h * (F64x::splat(2.0) * t - t * t / (t + F64x::splat(1.0)));
    let large = h * (t + t / (t + F64x::splat(1.0)));
    absolute_high
        .simd_lt(U64x::splat(0x3ff0_0000))
        .select(small, large)
}

/// Apply [`cosh`] to a slice in place.
///
/// SIMD is retained only for chunks wholly inside one scalar formula:
/// `expm1` below `0.5*ln(2)` or `exp` from there to 22. Mixed-formula,
/// tiny, large, and exceptional chunks use [`cosh`] lane by lane.
pub fn cosh_slice(values: &mut [f64]) {
    let mut index = 0;
    while index + SIMD_LANES <= values.len() {
        let input = F64x::from_slice(&values[index..]);
        let absolute_high = (input.to_bits() >> U64x::splat(32)) & U64x::splat(0x7fff_ffff);
        let small = absolute_high.simd_ge(U64x::splat(0x3c90_0000))
            & absolute_high.simd_lt(U64x::splat(0x3fd6_2e43));
        let medium = absolute_high.simd_ge(U64x::splat(0x3fd6_2e43))
            & absolute_high.simd_lt(U64x::splat(0x4036_0000));
        let result = if small.all() {
            let t = expm1_lanes(input.abs());
            let w = F64x::splat(1.0) + t;
            Some(F64x::splat(1.0) + t * t / (w + w))
        } else if medium.all() {
            let t = exp_lanes(input.abs());
            Some(F64x::splat(0.5) * t + F64x::splat(0.5) / t)
        } else {
            None
        };
        if let Some(result) = result {
            result.copy_to_slice(&mut values[index..index + SIMD_LANES]);
        } else {
            for value in &mut values[index..index + SIMD_LANES] {
                *value = cosh(*value);
            }
        }
        index += SIMD_LANES;
    }
    for value in &mut values[index..] {
        *value = cosh(*value);
    }
}

/// Apply [`tanh`] to a slice in place.
///
/// Chunks wholly below or above `|x| = 1` use the corresponding scalar
/// `expm1` formula in independent lanes. Mixed, tiny, saturated, and
/// exceptional chunks stay scalar.
pub fn tanh_slice(values: &mut [f64]) {
    let mut index = 0;
    while index + SIMD_LANES <= values.len() {
        let input = F64x::from_slice(&values[index..]);
        let bits = input.to_bits();
        let absolute_high = (bits >> U64x::splat(32)) & U64x::splat(0x7fff_ffff);
        let ordinary = absolute_high.simd_ge(U64x::splat(0x3c80_0000))
            & absolute_high.simd_lt(U64x::splat(0x4036_0000));
        let below_one = absolute_high.simd_lt(U64x::splat(0x3ff0_0000));
        let result = if ordinary.all() && below_one.all() {
            Some(tanh_lanes(input, false))
        } else if ordinary.all() && (!below_one).all() {
            Some(tanh_lanes(input, true))
        } else {
            None
        };
        if let Some(result) = result {
            result.copy_to_slice(&mut values[index..index + SIMD_LANES]);
        } else {
            for value in &mut values[index..index + SIMD_LANES] {
                *value = tanh(*value);
            }
        }
        index += SIMD_LANES;
    }
    for value in &mut values[index..] {
        *value = tanh(*value);
    }
}

#[inline]
fn tanh_lanes(input: F64x, at_least_one: bool) -> F64x {
    let bits = input.to_bits();
    let negative = (bits >> U64x::splat(63)).simd_eq(U64x::splat(1));
    let absolute = input.abs();
    let magnitude = if at_least_one {
        let t = expm1_lanes(F64x::splat(2.0) * absolute);
        F64x::splat(1.0) - F64x::splat(2.0) / (t + F64x::splat(2.0))
    } else {
        let t = expm1_lanes(F64x::splat(-2.0) * absolute);
        -t / (t + F64x::splat(2.0))
    };
    negative.select(-magnitude, magnitude)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Monotone integer image of a double (±0 collapse to 0).
    fn ord(x: f64) -> i128 {
        let b = x.to_bits() as i64;
        i128::from(if b < 0 { i64::MIN.wrapping_sub(b) } else { b })
    }

    /// Bit distance in ulps; 0 when both are NaN, huge when only one is.
    fn ulp_diff(a: f64, b: f64) -> u128 {
        if a.is_nan() || b.is_nan() {
            return if a.is_nan() && b.is_nan() {
                0
            } else {
                u128::MAX
            };
        }
        ord(a).abs_diff(ord(b))
    }

    #[test]
    fn sinh_sweep_vs_std() {
        let mut max = 0u128;
        for i in 0..10_000i32 {
            let x = -30.0 + f64::from(i) * 60.0 / 9_999.0;
            let d = ulp_diff(sinh(x), x.sinh());
            if d > max {
                max = d;
            }
        }
        println!("sinh max ulp vs std: {max}");
        assert!(max <= 2, "sinh sweep max ulp {max}");
    }

    #[test]
    fn cosh_sweep_vs_std() {
        let mut max = 0u128;
        for i in 0..10_000i32 {
            let x = -30.0 + f64::from(i) * 60.0 / 9_999.0;
            let d = ulp_diff(cosh(x), x.cosh());
            if d > max {
                max = d;
            }
        }
        println!("cosh max ulp vs std: {max}");
        assert!(max <= 2, "cosh sweep max ulp {max}");
    }

    #[test]
    fn tanh_sweep_vs_std() {
        let mut max = 0u128;
        for i in 0..10_000i32 {
            let x = -30.0 + f64::from(i) * 60.0 / 9_999.0;
            let d = ulp_diff(tanh(x), x.tanh());
            if d > max {
                max = d;
            }
        }
        println!("tanh max ulp vs std: {max}");
        assert!(max <= 2, "tanh sweep max ulp {max}");
    }

    #[test]
    fn hyper_large_arguments_vs_std() {
        // exp path, half-exp path, and the overflow edge.
        let mut max = 0u128;
        for &x in &[
            25.0, -25.0, 100.0, -100.0, 700.0, -700.0, 709.9, -709.9, 710.2, -710.2, 710.47,
            -710.47,
        ] {
            let ds = ulp_diff(sinh(x), x.sinh());
            let dc = ulp_diff(cosh(x), x.cosh());
            max = max.max(ds).max(dc);
        }
        println!("sinh/cosh large-argument max ulp vs std: {max}");
        assert!(max <= 2, "large-argument max ulp {max}");
        // Beyond the threshold: overflow with the right sign.
        assert_eq!(sinh(711.0), f64::INFINITY);
        assert_eq!(sinh(-711.0), f64::NEG_INFINITY);
        assert_eq!(cosh(711.0), f64::INFINITY);
        assert_eq!(cosh(-711.0), f64::INFINITY);
    }

    #[test]
    fn hyper_special_values() {
        // sinh
        assert_eq!(sinh(0.0).to_bits(), 0.0f64.to_bits());
        assert_eq!(sinh(-0.0).to_bits(), (-0.0f64).to_bits());
        assert_eq!(sinh(f64::INFINITY), f64::INFINITY);
        assert_eq!(sinh(f64::NEG_INFINITY), f64::NEG_INFINITY);
        assert!(sinh(f64::NAN).is_nan());
        let tiny = 1.0e-300;
        assert_eq!(sinh(tiny).to_bits(), tiny.to_bits());
        // cosh
        assert_eq!(cosh(0.0).to_bits(), 1.0f64.to_bits());
        assert_eq!(cosh(-0.0).to_bits(), 1.0f64.to_bits());
        assert_eq!(cosh(f64::INFINITY), f64::INFINITY);
        assert_eq!(cosh(f64::NEG_INFINITY), f64::INFINITY);
        assert!(cosh(f64::NAN).is_nan());
        assert_eq!(cosh(tiny).to_bits(), 1.0f64.to_bits());
        // tanh
        assert_eq!(tanh(0.0).to_bits(), 0.0f64.to_bits());
        assert_eq!(tanh(-0.0).to_bits(), (-0.0f64).to_bits());
        assert_eq!(tanh(f64::INFINITY).to_bits(), 1.0f64.to_bits());
        assert_eq!(tanh(f64::NEG_INFINITY).to_bits(), (-1.0f64).to_bits());
        assert!(tanh(f64::NAN).is_nan());
        assert_eq!(tanh(tiny).to_bits(), tiny.to_bits());
        assert_eq!(tanh(30.0).to_bits(), 1.0f64.to_bits());
    }

    #[test]
    fn cosh_sinh_identity() {
        // cosh^2 - sinh^2 = 1 for |x| < 5, within the error budget of
        // the squares.
        for i in 0..10_000i32 {
            let x = -5.0 + f64::from(i) * 10.0 / 9_999.0;
            let c = cosh(x);
            let s = sinh(x);
            let err = (c * c - s * s - 1.0).abs();
            assert!(err < 1.0e-10, "identity error {err:e} at x = {x}");
        }
    }

    #[test]
    fn tanh_is_odd_bitwise() {
        for i in 0..10_000i32 {
            let x = f64::from(i) * 30.0 / 9_999.0;
            assert_eq!(
                tanh(-x).to_bits(),
                (-tanh(x)).to_bits(),
                "tanh not odd at {x}"
            );
        }
    }
}
