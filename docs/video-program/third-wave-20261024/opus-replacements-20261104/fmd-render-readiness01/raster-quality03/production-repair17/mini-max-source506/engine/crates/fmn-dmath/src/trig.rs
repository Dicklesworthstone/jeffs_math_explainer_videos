//! sin / cos / tan for f64: FDLIBM-structure, fixed coefficients.
//!
//! Kernels are straight ports of FDLIBM `__kernel_sin`, `__kernel_cos`,
//! and `__kernel_tan` (as carried by musl), with the published minimax
//! coefficient sets (S1..S6, C1..C6, T[0..=12]) pinned by their exact bit
//! patterns and evaluated in the exact published operation order. Argument
//! reduction is [`crate::rem_pio2`]. Because every operation is plain IEEE
//! f64 arithmetic in a fixed order, results are bit-identical on every
//! target.
//!
//! Design accuracy bounds (verified downstream against mpmath vectors):
//! * `sin`, `cos`: < 1 ulp over the full finite domain.
//! * `tan`: < 2 ulp over the full finite domain.
//!
//! §17.3 audit ruling: these three entry points remain scalar. Their
//! representative workload is inseparable from the branchy Cody–Waite /
//! Payne–Hanek range reducer, while vectorizing only the primary-range
//! polynomial would benchmark a non-representative subset. A future retained
//! route needs a measured bulk range-reduction contract, ideally shared by a
//! paired sine/cosine API, before adding another public surface.

use crate::bits::{hi, zero_lo};
use crate::rem_pio2::rem_pio2;

// ---------------------------------------------------------------------------
// __kernel_sin coefficients (FDLIBM): sin(x) ~ x + S1*x^3 + ... + S6*x^13
// on |x| <= pi/4. Decimal values in comments are the FDLIBM printouts.
// ---------------------------------------------------------------------------
const S1: f64 = f64::from_bits(0xBFC5_5555_5555_5549); // -1.66666666666666324348e-01
const S2: f64 = f64::from_bits(0x3F81_1111_1110_F8A6); //  8.33333333332248946124e-03
const S3: f64 = f64::from_bits(0xBF2A_01A0_19C1_61D5); // -1.98412698298579493134e-04
const S4: f64 = f64::from_bits(0x3EC7_1DE3_57B1_FE7D); //  2.75573137070700676789e-06
const S5: f64 = f64::from_bits(0xBE5A_E5E6_8A2B_9CEB); // -2.50507602534068634195e-08
const S6: f64 = f64::from_bits(0x3DE5_D93A_5ACF_D57C); //  1.58969099521155010221e-10

// ---------------------------------------------------------------------------
// __kernel_cos coefficients (FDLIBM): cos(x) ~ 1 - x^2/2 + C1*x^4 + ...
// ---------------------------------------------------------------------------
const C1: f64 = f64::from_bits(0x3FA5_5555_5555_554C); //  4.16666666666666019037e-02
const C2: f64 = f64::from_bits(0xBF56_C16C_16C1_5177); // -1.38888888888741095749e-03
const C3: f64 = f64::from_bits(0x3EFA_01A0_19CB_1590); //  2.48015872894767294178e-05
const C4: f64 = f64::from_bits(0xBE92_7E4F_809C_52AD); // -2.75573143513906633035e-07
const C5: f64 = f64::from_bits(0x3E21_EE9E_BDB4_B1C4); //  2.08757232129817482790e-09
const C6: f64 = f64::from_bits(0xBDA8_FAE9_BE88_38D4); // -1.13596475577881948265e-11

// ---------------------------------------------------------------------------
// __kernel_tan coefficients (FDLIBM): odd polynomial on |x| < 0.6744,
// tan(x) ~ x + T[0]*x^3 + T[1]*x^5 + ... + T[12]*x^27.
// ---------------------------------------------------------------------------
const T: [f64; 13] = [
    f64::from_bits(0x3FD5_5555_5555_5563), //  3.33333333333334091986e-01
    f64::from_bits(0x3FC1_1111_1110_FE7A), //  1.33333333333201242699e-01
    f64::from_bits(0x3FAB_A1BA_1BB3_41FE), //  5.39682539762260521377e-02
    f64::from_bits(0x3F96_64F4_8406_D637), //  2.18694882948595424599e-02
    f64::from_bits(0x3F82_26E3_E96E_8493), //  8.86323982359930005737e-03
    f64::from_bits(0x3F6D_6D22_C956_0328), //  3.59207910759131235356e-03
    f64::from_bits(0x3F57_DBC8_FEE0_8315), //  1.45620945432529025516e-03
    f64::from_bits(0x3F43_44D8_F2F2_6501), //  5.88041240820264096874e-04
    f64::from_bits(0x3F30_26F7_1A8D_1068), //  2.46463134818469906812e-04
    f64::from_bits(0x3F14_7E88_A037_92A6), //  7.81794442939557092300e-05
    f64::from_bits(0x3F12_B80F_32F0_A7E9), //  7.14072491382608190305e-05
    f64::from_bits(0xBEF3_75CB_DB60_5373), // -1.85586374855275456654e-05
    f64::from_bits(0x3EFB_2A70_74BF_7AD4), //  2.59073051863633712884e-05
];
/// High part of pi/4. 7.85398163397448278999e-01
const PIO4: f64 = f64::from_bits(0x3FE9_21FB_5444_2D18);
/// Low part of pi/4. 3.06161699786838301793e-17
const PIO4_LO: f64 = f64::from_bits(0x3C81_A626_3314_5C07);

/// FDLIBM `__kernel_sin`: sine on |x| <= pi/4, where `y` is the low part of
/// the reduced argument and `iy == 0` means y is exactly zero (unreduced
/// input). The correction term folds `y` in as sin(x+y) ~ sin x + y*cos x.
fn k_sin(x: f64, y: f64, iy: i32) -> f64 {
    let z = x * x;
    let w = z * z;
    // r = x^4 part of the polynomial, in FDLIBM's exact split/nesting.
    let r = S2 + z * (S3 + z * S4) + z * w * (S5 + z * S6);
    let v = z * x;
    if iy == 0 {
        x + v * (S1 + z * r)
    } else {
        x - ((z * (0.5 * y - v * r) - y) - v * S1)
    }
}

/// FDLIBM `__kernel_cos`: cosine on |x| <= pi/4 with low part `y`.
/// 1 - x^2/2 is formed as w + (((1-w)-hz) + ...) to keep the rounding of
/// the leading terms exact; the x*y term is the cos(x+y) cross-correction.
fn k_cos(x: f64, y: f64) -> f64 {
    let z = x * x;
    let w = z * z;
    let r = z * (C1 + z * (C2 + z * C3)) + w * w * (C4 + z * (C5 + z * C6));
    let hz = 0.5 * z;
    let w2 = 1.0 - hz;
    w2 + (((1.0 - w2) - hz) + (z * r - x * y))
}

/// FDLIBM `__kernel_tan` (musl `__tan`): tangent on |x| <= pi/4 with low
/// part `y`; `odd` selects -1/tan (the odd-quadrant continuation).
///
/// For |x| >= 0.6744 the argument is reflected as tan(x) = 1/tan(pi/4 - x)
/// via the two-part (PIO4, PIO4_LO) constant. The final -1/(x+r) for the
/// odd case is computed with a split-word correction step to stay within
/// the accuracy bound.
fn k_tan(x: f64, y: f64, odd: bool) -> f64 {
    let hx = hi(x);
    let big = (hx & 0x7FFF_FFFF) >= 0x3FE5_9428; // |x| >= 0.6744
    let mut x = x;
    let mut y = y;
    let mut sign = false;
    if big {
        sign = hx >> 31 != 0;
        if sign {
            x = -x;
            y = -y;
        }
        x = (PIO4 - x) + (PIO4_LO - y);
        y = 0.0;
    }
    let z = x * x;
    let w = z * z;
    // Break x^5*(T[1]+x^2*T[2]+...) into two interleaved even/odd chains,
    // exactly as FDLIBM does (this is the published operation order).
    let r = T[1] + w * (T[3] + w * (T[5] + w * (T[7] + w * (T[9] + w * T[11]))));
    let v = z * (T[2] + w * (T[4] + w * (T[6] + w * (T[8] + w * (T[10] + w * T[12])))));
    let s = z * x;
    let r = y + z * (s * (r + v) + y) + s * T[0];
    let w = x + r; // tan(x) ~ x + r
    if big {
        // tan(pi/4 - x') reflected back: 1 - 2*(x - (w^2/(w+s) - r)).
        let s2 = if odd { -1.0 } else { 1.0 };
        let v2 = s2 - 2.0 * (x + (r - w * w / (w + s2)));
        return if sign { -v2 } else { v2 };
    }
    if !odd {
        return w;
    }
    // Compute -1.0/(x+r) accurately: a plain division is up to 2 ulp off,
    // so refine with hi/lo split words (SET_LOW_WORD(_, 0) in FDLIBM).
    let w0 = zero_lo(w);
    let v2 = r - (w0 - x); // w0 + v2 = r + x
    let a = -1.0 / w;
    let a0 = zero_lo(a);
    a0 + a * (1.0 + a0 * w0 + a0 * v2)
}

/// Deterministic sine (FDLIBM `sin` structure).
///
/// Bit-identical on every target; design accuracy < 1 ulp versus the
/// correctly rounded result. `sin(±0)` is `±0` (sign preserved),
/// `sin(±inf)` and `sin(NaN)` are NaN.
#[must_use]
pub fn sin(x: f64) -> f64 {
    let ix = hi(x) & 0x7FFF_FFFF;

    // |x| ~<= pi/4: no reduction.
    if ix <= 0x3FE9_21FB {
        if ix < 0x3E50_0000 {
            // |x| < 2^-26: sin(x) rounds to x (also preserves ±0).
            return x;
        }
        return k_sin(x, 0.0, 0);
    }

    // sin(inf or NaN) is NaN.
    if ix >= 0x7FF0_0000 {
        // FDLIBM's `x - x` idiom: inf becomes NaN, NaN payloads propagate.
        #[allow(clippy::eq_op)]
        return x - x;
    }

    // Reduce and dispatch on the quadrant.
    let (n, y0, y1) = rem_pio2(x);
    match n {
        0 => k_sin(y0, y1, 1),
        1 => k_cos(y0, y1),
        2 => -k_sin(y0, y1, 1),
        _ => -k_cos(y0, y1),
    }
}

/// Deterministic cosine (FDLIBM `cos` structure).
///
/// Bit-identical on every target; design accuracy < 1 ulp versus the
/// correctly rounded result. `cos(±0)` is `1`, `cos(±inf)` and `cos(NaN)`
/// are NaN.
#[must_use]
pub fn cos(x: f64) -> f64 {
    let ix = hi(x) & 0x7FFF_FFFF;

    // |x| ~<= pi/4: no reduction.
    if ix <= 0x3FE9_21FB {
        if ix < 0x3E46_A09E {
            // |x| < 2^-27 * sqrt(2): cos(x) rounds to 1.
            return 1.0;
        }
        return k_cos(x, 0.0);
    }

    // cos(inf or NaN) is NaN.
    if ix >= 0x7FF0_0000 {
        // FDLIBM's `x - x` idiom: inf becomes NaN, NaN payloads propagate.
        #[allow(clippy::eq_op)]
        return x - x;
    }

    // Reduce and dispatch on the quadrant.
    let (n, y0, y1) = rem_pio2(x);
    match n {
        0 => k_cos(y0, y1),
        1 => -k_sin(y0, y1, 1),
        2 => -k_cos(y0, y1),
        _ => k_sin(y0, y1, 1),
    }
}

/// Deterministic tangent (FDLIBM `tan` structure).
///
/// Bit-identical on every target; design accuracy < 2 ulp versus the
/// correctly rounded result. `tan(±0)` is `±0` (sign preserved),
/// `tan(±inf)` and `tan(NaN)` are NaN.
#[must_use]
pub fn tan(x: f64) -> f64 {
    let ix = hi(x) & 0x7FFF_FFFF;

    // |x| ~<= pi/4: no reduction.
    if ix <= 0x3FE9_21FB {
        if ix < 0x3E40_0000 {
            // |x| < 2^-27: tan(x) rounds to x (also preserves ±0).
            return x;
        }
        return k_tan(x, 0.0, false);
    }

    // tan(inf or NaN) is NaN.
    if ix >= 0x7FF0_0000 {
        // FDLIBM's `x - x` idiom: inf becomes NaN, NaN payloads propagate.
        #[allow(clippy::eq_op)]
        return x - x;
    }

    // Reduce; tan has period pi, so only the parity of n matters.
    let (n, y0, y1) = rem_pio2(x);
    k_tan(y0, y1, n & 1 == 1)
}

#[cfg(test)]
mod tests {
    use super::{cos, sin, tan};

    /// Map an f64 onto the monotone integer line (sign-magnitude order) so
    /// ulp distance is a plain subtraction; ±0 both map to 0.
    fn ordered(x: f64) -> i64 {
        let i = x.to_bits() as i64;
        if i < 0 { i64::MIN - i } else { i }
    }

    /// Ulp distance between two finite f64s (handles sign crossings).
    fn ulp_diff(a: f64, b: f64) -> u64 {
        assert!(!a.is_nan() && !b.is_nan(), "NaN in ulp comparison: {a} {b}");
        (i128::from(ordered(a)) - i128::from(ordered(b))).unsigned_abs() as u64
    }

    /// Deterministic sweep: 10_000 points on [-100, 100).
    fn sweep() -> impl Iterator<Item = f64> {
        (0..10_000).map(|i| -100.0 + f64::from(i) * 0.02)
    }

    #[test]
    fn sin_matches_std_within_2_ulp_on_sweep() {
        let mut max = 0;
        for x in sweep() {
            let d = ulp_diff(sin(x), x.sin());
            max = max.max(d);
            assert!(d <= 2, "sin({x}): {d} ulp from std");
        }
        // The design bound is < 1 ulp from exact; std is within ~1 ulp of
        // exact, so the observed max should be small. Keep for diagnostics.
        assert!(max <= 2, "max sin deviation {max} ulp");
    }

    #[test]
    fn cos_matches_std_within_2_ulp_on_sweep() {
        for x in sweep() {
            let d = ulp_diff(cos(x), x.cos());
            assert!(d <= 2, "cos({x}): {d} ulp from std");
        }
    }

    #[test]
    fn tan_matches_std_within_2_ulp_on_sweep() {
        for x in sweep() {
            let d = ulp_diff(tan(x), x.tan());
            assert!(d <= 2, "tan({x}): {d} ulp from std");
        }
    }

    #[test]
    fn large_args_match_std_within_2_ulp() {
        let big = [
            1e10,
            1e16,
            1e22,
            f64::from_bits((1000 + 1023_u64) << 52), // 2^1000
            f64::from_bits(((1000 + 1023_u64) << 52) | 0x000F_5678_9ABC_DEF0),
            1e300,
            -1e10,
            -1e16,
            -1e22,
        ];
        for &x in &big {
            assert!(ulp_diff(sin(x), x.sin()) <= 2, "sin({x})");
            assert!(ulp_diff(cos(x), x.cos()) <= 2, "cos({x})");
            assert!(ulp_diff(tan(x), x.tan()) <= 2, "tan({x})");
        }
    }

    #[test]
    fn sin_is_odd_and_cos_is_even_bitwise() {
        for x in sweep() {
            assert_eq!(
                sin(-x).to_bits(),
                (-sin(x)).to_bits(),
                "sin not bitwise odd at {x}"
            );
            assert_eq!(
                cos(-x).to_bits(),
                cos(x).to_bits(),
                "cos not bitwise even at {x}"
            );
            assert_eq!(
                tan(-x).to_bits(),
                (-tan(x)).to_bits(),
                "tan not bitwise odd at {x}"
            );
        }
    }

    #[test]
    fn pythagorean_identity_holds() {
        for x in sweep() {
            let s = sin(x);
            let c = cos(x);
            let err = (s * s + c * c - 1.0).abs();
            assert!(err < 4e-16, "sin^2+cos^2 off by {err} at {x}");
        }
    }

    #[test]
    fn tan_consistent_with_sin_over_cos_away_from_poles() {
        for x in sweep() {
            let c = cos(x);
            if c.abs() < 1e-2 {
                continue; // near a pole: the quotient itself is ill-conditioned
            }
            let d = ulp_diff(tan(x), sin(x) / c);
            // tan < 2 ulp, sin/cos each < 1 ulp plus the division's 0.5:
            // allow a small combined budget.
            assert!(d <= 4, "tan({x}) vs sin/cos: {d} ulp");
        }
    }

    #[test]
    fn special_values() {
        // Signed zero preserved by the odd functions.
        assert_eq!(sin(0.0).to_bits(), 0.0_f64.to_bits());
        assert_eq!(sin(-0.0).to_bits(), (-0.0_f64).to_bits());
        assert_eq!(tan(0.0).to_bits(), 0.0_f64.to_bits());
        assert_eq!(tan(-0.0).to_bits(), (-0.0_f64).to_bits());
        // cos(±0) = 1 exactly.
        assert_eq!(cos(0.0).to_bits(), 1.0_f64.to_bits());
        assert_eq!(cos(-0.0).to_bits(), 1.0_f64.to_bits());
        // NaN propagates; infinities produce NaN.
        for f in [sin as fn(f64) -> f64, cos, tan] {
            assert!(f(f64::NAN).is_nan());
            assert!(f(f64::INFINITY).is_nan());
            assert!(f(f64::NEG_INFINITY).is_nan());
        }
        // Tiny arguments: sin(x) == x, cos(x) == 1 below the thresholds.
        let tiny = f64::from_bits(0x3E3F_FFFF_FFFF_FFFF); // just under 2^-28
        assert_eq!(sin(tiny).to_bits(), tiny.to_bits());
        assert_eq!(tan(tiny).to_bits(), tiny.to_bits());
        assert_eq!(cos(tiny).to_bits(), 1.0_f64.to_bits());
    }
}
