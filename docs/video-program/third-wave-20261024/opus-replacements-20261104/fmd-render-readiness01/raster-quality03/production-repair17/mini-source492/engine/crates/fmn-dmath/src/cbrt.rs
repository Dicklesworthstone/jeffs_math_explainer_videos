//! Cube root: `cbrt`.
//!
//! Algorithm and coefficients are FDLIBM `s_cbrt.c` (Sun Microsystems,
//! with the W. Kahan-attributed polynomial refinement as carried by
//! FreeBSD/musl): a bit-hack initial estimate good to ~5 bits, a degree-4
//! polynomial in `r = x/t^3` lifting it to ~23 bits, a round-away-to-
//! 23-bits step that makes the following division safe, and one Newton
//! iteration to full precision. All constants are materialized from
//! their published bit patterns; the only operations are +, -, *, / and
//! bit moves, so the result is bit-identical on every target.
//!
//! Design accuracy bound: < 1 ulp (the FreeBSD source documents the
//! final error as < 0.667 ulp); verified downstream against mpmath
//! ground truth by `tests/vectors.rs`. Special cases: `cbrt(+-0) = +-0`,
//! `cbrt(+-inf) = +-inf`, NaN propagates, and the sign is carried
//! through the bit pipeline so `cbrt(-x) == -cbrt(x)` bitwise.

use crate::bits::{F64x, SIMD_LANES, U64x};
use std::simd::cmp::SimdPartialOrd;
use std::simd::num::SimdFloat;

/// B1 = (1023 - 1023/3 - 0.03306235651) * 2^20 — exponent-bias fixup for
/// the normal-range initial estimate (s_cbrt.c).
const B1: u32 = 715_094_163;
/// B2 = (1023 - 1023/3 - 54/3 - 0.03306235651) * 2^20 — same fixup after
/// scaling a subnormal input by 2^54 (s_cbrt.c).
const B2: u32 = 696_219_795;

/// 2^54, the subnormal prescale.
const TWO54: f64 = f64::from_bits(0x4350_0000_0000_0000);

// |1/cbrt(x) - p(x)| < 2^-23.5 (s_cbrt.c P0..P4).
const P0: f64 = f64::from_bits(0x3FFE_03E6_0F61_E692); //  1.87595182427177009643
const P1: f64 = f64::from_bits(0xBFFE_28E0_92F0_2420); // -1.88497979543377169875
const P2: f64 = f64::from_bits(0x3FF9_F160_4A49_D6C2); //  1.62142972010535446614
const P3: f64 = f64::from_bits(0xBFE8_44CB_BEE7_51D9); // -0.758397934778766047437
const P4: f64 = f64::from_bits(0x3FC2_B000_D4E4_EDD7); //  0.145996192886612446982

/// Cube root, FDLIBM `s_cbrt.c`, bit-reproducible.
///
/// Handles the full f64 range including subnormals (scaled by 2^54 and
/// compensated through the `B2` exponent fixup). `cbrt(+-0) = +-0`,
/// `cbrt(+-inf) = +-inf`, `cbrt(NaN) = NaN`. Design bound: < 1 ulp
/// (documented < 0.667 ulp in the FreeBSD source).
#[must_use]
pub fn cbrt(x: f64) -> f64 {
    let mut ui = x.to_bits();
    let mut hx = ((ui >> 32) as u32) & 0x7fff_ffff;

    // s_cbrt.c: cbrt(NaN) is NaN, cbrt(+-inf) is +-inf.
    if hx >= 0x7ff0_0000 {
        return x + x;
    }

    // Rough cbrt to 5 bits:
    //   cbrt(2^e * (1+m)) ~= 2^(e/3) * (1 + (e%3 + m)/3)
    // realized by dividing the biased-exponent-plus-mantissa high word by
    // 3 and adding the precomputed bias correction B1 (or B2 after the
    // 2^54 prescale for subnormals).
    if hx < 0x0010_0000 {
        // Zero or subnormal.
        ui = (x * TWO54).to_bits();
        hx = ((ui >> 32) as u32) & 0x7fff_ffff;
        if hx == 0 {
            return x; // cbrt(+-0) = +-0
        }
        hx = hx / 3 + B2;
    } else {
        hx = hx / 3 + B1;
    }
    ui &= 1_u64 << 63; // keep only the sign of x
    ui |= u64::from(hx) << 32;
    let mut t = f64::from_bits(ui);

    // New cbrt to 23 bits (s_cbrt.c):
    //   cbrt(x) = t * cbrt(x/t^3) ~= t * P(r), r = x/t^3 = (t/x * t * t)^-1
    // with r computed as (t*t)*(t/x) so that both terms stay < 2 and the
    // product overflows/underflows nowhere t does not.
    let r = (t * t) * (t / x);
    t *= (P0 + r * (P1 + r * P2)) + ((r * r) * r) * (P3 + r * P4);

    // Round t away from zero to 23 bits (add half of bit 30, clear the
    // low 30 bits): t now exceeds cbrt(x) by less than ~2 23-bit ulps,
    // t*t is exact in double, and the Newton denominator below is safe.
    ui = t.to_bits();
    ui = (ui + 0x8000_0000) & 0xffff_ffff_c000_0000;
    t = f64::from_bits(ui);

    // One Newton step to 53 bits with error < 0.667 ulp (s_cbrt.c):
    let s = t * t; // t*t is exact
    let r = x / s; // error <= 0.5 ulp; |r| < |t|
    let w = t + t; // t+t is exact
    let r = (r - t) / (w + r); // r-t is exact; w+r ~= 3t
    t + t * r // error <= 0.5 + 0.5/3 + epsilon
}

/// Apply [`cbrt`] to a slice in place, grouping independent normal inputs
/// into build-tier SIMD lanes.
///
/// A chunk containing zero, a subnormal, infinity, or NaN takes the scalar
/// definition as a unit. That keeps every special-value and payload rule
/// exact while the overwhelmingly common normal finite path executes the
/// polynomial and Newton refinement lane-wise. There is no horizontal
/// reduction, so lane count cannot affect any result.
pub fn cbrt_slice(values: &mut [f64]) {
    let mut index = 0;
    while index + SIMD_LANES <= values.len() {
        let input = F64x::from_slice(&values[index..]);
        let high = (input.to_bits() >> U64x::splat(32)) & U64x::splat(0x7fff_ffff);
        let normal =
            high.simd_ge(U64x::splat(0x0010_0000)) & high.simd_lt(U64x::splat(0x7ff0_0000));
        if normal.all() {
            cbrt_normal_lanes(input).copy_to_slice(&mut values[index..index + SIMD_LANES]);
        } else {
            for value in &mut values[index..index + SIMD_LANES] {
                *value = cbrt(*value);
            }
        }
        index += SIMD_LANES;
    }
    for value in &mut values[index..] {
        *value = cbrt(*value);
    }
}

#[inline]
fn cbrt_normal_lanes(x: F64x) -> F64x {
    let bits = x.to_bits();
    let high = (bits >> U64x::splat(32)) & U64x::splat(0x7fff_ffff);
    let initial_high = high / U64x::splat(3) + U64x::splat(u64::from(B1));
    let sign = bits & U64x::splat(1_u64 << 63);
    let mut t = F64x::from_bits(sign | (initial_high << U64x::splat(32)));

    let r = (t * t) * (t / x);
    t *= (F64x::splat(P0) + r * (F64x::splat(P1) + r * F64x::splat(P2)))
        + ((r * r) * r) * (F64x::splat(P3) + r * F64x::splat(P4));

    let rounded = (t.to_bits() + U64x::splat(0x8000_0000)) & U64x::splat(0xffff_ffff_c000_0000);
    t = F64x::from_bits(rounded);

    let s = t * t;
    let r = x / s;
    let w = t + t;
    let r = (r - t) / (w + r);
    t + t * r
}

#[cfg(test)]
mod tests {
    use super::*;

    fn monotone(x: f64) -> i64 {
        #[allow(clippy::cast_possible_wrap)]
        let b = x.to_bits() as i64;
        if b < 0 { i64::MIN - b } else { b }
    }

    fn ulp_diff(a: f64, b: f64) -> u64 {
        assert_eq!(a.is_nan(), b.is_nan(), "NaN mismatch: {a:?} vs {b:?}");
        if a.is_nan() {
            return 0;
        }
        monotone(a).abs_diff(monotone(b))
    }

    /// Deterministic sweep: fine linear steps over [-4, 4], log-spaced
    /// magnitudes across the whole normal range, and subnormals.
    fn sweep_points() -> Vec<f64> {
        let mut pts = Vec::new();
        for i in 0..=10_000_u32 {
            pts.push(-4.0 + f64::from(i) * (8.0 / 10_000.0));
        }
        let mut m = 1.0e-300;
        while m <= 1.0e300 {
            pts.push(m);
            pts.push(-m);
            m *= 10.0;
        }
        // Subnormals, including the very smallest.
        for m in [5e-324, 1.5e-323, 1.0e-320, 3.7e-310, 2.0e-308] {
            pts.push(m);
            pts.push(-m);
        }
        pts
    }

    #[test]
    fn cbrt_exact_cubes_and_special_values() {
        assert_eq!(cbrt(8.0).to_bits(), 2.0_f64.to_bits());
        assert_eq!(cbrt(-27.0).to_bits(), (-3.0_f64).to_bits());
        assert_eq!(cbrt(1.0).to_bits(), 1.0_f64.to_bits());
        assert_eq!(cbrt(0.0).to_bits(), 0.0_f64.to_bits());
        assert_eq!(cbrt(-0.0).to_bits(), (-0.0_f64).to_bits());
        assert_eq!(cbrt(f64::INFINITY).to_bits(), f64::INFINITY.to_bits());
        assert_eq!(
            cbrt(f64::NEG_INFINITY).to_bits(),
            f64::NEG_INFINITY.to_bits()
        );
        assert!(cbrt(f64::NAN).is_nan());
        // Exact cube of a power of two deep in the subnormal range:
        // cbrt(2^-1074) = 2^-358.
        assert_eq!(cbrt(5e-324).to_bits(), (-358.0_f64).exp2().to_bits());
    }

    #[test]
    fn cbrt_cubes_back_within_4_ulp() {
        let mut max = 0;
        for x in sweep_points() {
            if x == 0.0 {
                continue;
            }
            let c = cbrt(x);
            let cubed = c * c * c;
            let d = ulp_diff(cubed, x);
            max = max.max(d);
            assert!(d <= 4, "cbrt({x:e})^3 = {cubed:e}: {d} ulp from x");
        }
        println!("cbrt cube-back max ulp: {max}");
    }

    #[test]
    fn cbrt_matches_std_within_2_ulp() {
        let mut max = 0;
        for x in sweep_points() {
            let d = ulp_diff(cbrt(x), x.cbrt());
            max = max.max(d);
            assert!(d <= 2, "cbrt({x:e}): {d} ulp");
        }
        println!("cbrt max ulp vs std: {max}");
    }

    #[test]
    fn cbrt_sign_symmetry_is_bitwise() {
        for x in sweep_points() {
            assert_eq!(cbrt(-x).to_bits(), (-cbrt(x)).to_bits(), "x = {x:e}");
        }
    }
}
