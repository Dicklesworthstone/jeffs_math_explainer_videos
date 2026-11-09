//! Inverse trigonometric kernels: `atan`, `atan2`, `asin`, `acos`.
//!
//! Algorithms and coefficient sets are the classical FDLIBM routines
//! (`s_atan.c`, `e_atan2.c`, `e_asin.c`, `e_acos.c`, Sun Microsystems,
//! as carried by musl/FreeBSD), transcribed with the exact operation
//! order preserved. Every constant is materialized with `f64::from_bits`
//! from the word pairs published in the FDLIBM sources, so no decimal
//! parsing is in the loop and the computed bits are a pure function of
//! the input on every target.
//!
//! Design accuracy bounds (verified downstream against mpmath ground
//! truth by `tests/vectors.rs`):
//!
//! * `atan`, `asin`, `acos`: < 1 ulp worst case.
//! * `atan2`: < 2 ulp worst case (the `pi_lo` correction paths near the
//!   ±x axis accumulate one extra rounding).
//!
//! IEEE 754-specified special cases (signed zeros, infinities, the
//! quadrant table of `atan2`) are reproduced exactly; `f64::sqrt` is the
//! one delegation (correctly rounded per IEEE 754, hence bit-identical
//! everywhere), needed by the |x| >= 0.5 paths of `asin`/`acos`.
//!
//! A §17.3 independent-lane bulk prototype was measured and rejected:
//! interval masks plus eager evaluation of mutually exclusive rational and
//! square-root branches cost more than these compact scalar functions. The
//! scalar definitions remain the measured implementation.

use crate::bits::{hi, lo, zero_lo};

// ---------------------------------------------------------------------------
// Shared constants (FDLIBM e_asin.c / e_acos.c / e_atan2.c).
// ---------------------------------------------------------------------------

/// pi/2 high part, 1.57079632679489655800e+00 (0x3FF921FB, 0x54442D18).
const PIO2_HI: f64 = f64::from_bits(0x3FF9_21FB_5444_2D18);
/// pi/2 low part, 6.12323399573676603587e-17 (0x3C91A626, 0x33145C07).
const PIO2_LO: f64 = f64::from_bits(0x3C91_A626_3314_5C07);
/// pi/4 high part, 7.85398163397448278999e-01 (0x3FE921FB, 0x54442D18).
const PIO4_HI: f64 = f64::from_bits(0x3FE9_21FB_5444_2D18);
/// pi, 3.14159265358979311600e+00 (0x400921FB, 0x54442D18).
const PI: f64 = f64::from_bits(0x4009_21FB_5444_2D18);
/// pi low part, 1.22464679914735317722e-16 (0x3CA1A626, 0x33145C07).
const PI_LO: f64 = f64::from_bits(0x3CA1_A626_3314_5C07);
/// FDLIBM's `tiny`; added to force inexact, absorbed by rounding.
const TINY: f64 = 1.0e-300;

// ---------------------------------------------------------------------------
// atan — FDLIBM s_atan.c.
// ---------------------------------------------------------------------------

/// atan(0.5)/atan(1.0)/atan(1.5)/atan(inf), high parts (s_atan.c `atanhi`).
const ATANHI: [f64; 4] = [
    f64::from_bits(0x3FDD_AC67_0561_BB4F), // 4.63647609000806093515e-01
    f64::from_bits(0x3FE9_21FB_5444_2D18), // 7.85398163397448278999e-01
    f64::from_bits(0x3FEF_730B_D281_F69B), // 9.82793723247329054082e-01
    f64::from_bits(0x3FF9_21FB_5444_2D18), // 1.57079632679489655800e+00
];

/// Matching low parts (s_atan.c `atanlo`).
const ATANLO: [f64; 4] = [
    f64::from_bits(0x3C7A_2B7F_222F_65E2), // 2.26987774529616870924e-17
    f64::from_bits(0x3C81_A626_3314_5C07), // 3.06161699786838301793e-17
    f64::from_bits(0x3C70_0788_7AF0_CBBD), // 1.39033110312309984516e-17
    f64::from_bits(0x3C91_A626_3314_5C07), // 6.12323399573676603587e-17
];

/// Minimax polynomial coefficients for atan on the reduced interval
/// (s_atan.c `aT[0..=10]`; aT[i] is close to (-1)^i / (2i + 3)).
const AT: [f64; 11] = [
    f64::from_bits(0x3FD5_5555_5555_550D), //  3.33333333333329318027e-01
    f64::from_bits(0xBFC9_9999_9998_EBC4), // -1.99999999998764832476e-01
    f64::from_bits(0x3FC2_4924_9200_83FF), //  1.42857142725034663711e-01
    f64::from_bits(0xBFBC_71C6_FE23_1671), // -1.11111104054623557880e-01
    f64::from_bits(0x3FB7_45CD_C54C_206E), //  9.09088713343650656196e-02
    f64::from_bits(0xBFB3_B0F2_AF74_9A6D), // -7.69187620504482999495e-02
    f64::from_bits(0x3FB1_0D66_A0D0_3D51), //  6.66107313738753120669e-02
    f64::from_bits(0xBFAD_DE2D_52DE_FD9A), // -5.83357013379057348645e-02
    f64::from_bits(0x3FA9_7B4B_2476_0DEB), //  4.97687799461593236017e-02
    f64::from_bits(0xBFA2_B444_2C6A_6C2F), // -3.65315727442169155270e-02
    f64::from_bits(0x3F90_AD3A_E322_DA11), //  1.62858201153657823623e-02
];

/// Arc tangent, FDLIBM `s_atan.c`, bit-reproducible.
///
/// Four-interval argument reduction at 7/16, 11/16, 19/16, 39/16 and
/// 2^66 onto |t| < 7/16, then an 11-term odd/even-split polynomial in
/// t^2, recombined against the `ATANHI`/`ATANLO` table. Design bound:
/// < 1 ulp.
#[must_use]
pub fn atan(x: f64) -> f64 {
    let hx = hi(x);
    let ix = hx & 0x7fff_ffff;

    // s_atan.c: |x| >= 2^66 — atan saturates at +-pi/2 (NaN passes through).
    if ix >= 0x4410_0000 {
        if ix > 0x7ff0_0000 || (ix == 0x7ff0_0000 && lo(x) != 0) {
            return x + x; // NaN
        }
        if hx >> 31 == 0 {
            return ATANHI[3] + ATANLO[3];
        }
        return -ATANHI[3] - ATANLO[3];
    }

    // s_atan.c argument reduction: pick interval id and reduced argument.
    let id: i32;
    let mut t = x;
    if ix < 0x3fdc_0000 {
        // |x| < 7/16: no reduction.
        if ix < 0x3e40_0000 {
            // |x| < 2^-27: atan(x) = x to double precision.
            return x;
        }
        id = -1;
    } else {
        t = x.abs();
        if ix < 0x3ff3_0000 {
            if ix < 0x3fe6_0000 {
                // 7/16 <= |x| < 11/16: atan(x) = atan(1/2) + atan(f), f = (2x-1)/(2+x).
                id = 0;
                t = (2.0 * t - 1.0) / (2.0 + t);
            } else {
                // 11/16 <= |x| < 19/16: atan(x) = atan(1) + atan(f), f = (x-1)/(x+1).
                id = 1;
                t = (t - 1.0) / (t + 1.0);
            }
        } else if ix < 0x4003_8000 {
            // 19/16 <= |x| < 39/16: atan(x) = atan(3/2) + atan(f), f = (x-1.5)/(1+1.5x).
            id = 2;
            t = (t - 1.5) / (1.0 + 1.5 * t);
        } else {
            // 39/16 <= |x| < 2^66: atan(x) = pi/2 - atan(1/x).
            id = 3;
            t = -1.0 / t;
        }
    }

    // s_atan.c polynomial: sum aT[i] z^(i+1) split into odd/even halves,
    // accumulated in exactly this nesting.
    let z = t * t;
    let w = z * z;
    let s1 = z * (AT[0] + w * (AT[2] + w * (AT[4] + w * (AT[6] + w * (AT[8] + w * AT[10])))));
    let s2 = w * (AT[1] + w * (AT[3] + w * (AT[5] + w * (AT[7] + w * AT[9]))));
    if id < 0 {
        return t - t * (s1 + s2);
    }
    #[allow(clippy::cast_sign_loss)] // id in 0..=3 here
    let idx = id as usize;
    let z = ATANHI[idx] - ((t * (s1 + s2) - ATANLO[idx]) - t);
    if hx >> 31 == 0 { z } else { -z }
}

// ---------------------------------------------------------------------------
// atan2 — FDLIBM e_atan2.c.
// ---------------------------------------------------------------------------

/// Quadrant-aware arc tangent of `y/x`, FDLIBM `e_atan2.c`,
/// bit-reproducible.
///
/// Reproduces every IEEE 754-required special case exactly:
/// `atan2(+-0, +0) = +-0`, `atan2(+-0, -0) = +-pi`, `atan2(+-y, 0) =
/// +-pi/2`, the +-pi/4 and +-3pi/4 double-infinity results, and the
/// signed-zero/inf quadrant table. Uses the |y/x| > 2^60 and < 2^-60
/// shortcuts and the `PI_LO` correction terms of the original. Design
/// bound: < 2 ulp worst case (near the +-x axis); < 1 ulp elsewhere.
#[must_use]
pub fn atan2(y: f64, x: f64) -> f64 {
    // e_atan2.c: NaN in, NaN out.
    if x.is_nan() || y.is_nan() {
        return x + y;
    }
    let hx = hi(x);
    let lx = lo(x);
    let hy = hi(y);
    let ly = lo(y);
    let ix = hx & 0x7fff_ffff;
    let iy = hy & 0x7fff_ffff;

    // x == 1.0: reduce to atan(y).
    if hx == 0x3ff0_0000 && lx == 0 {
        return atan(y);
    }

    // m = 2*sign(x) + sign(y) selects the quadrant fixups below.
    let m = ((hy >> 31) & 1) | ((hx >> 30) & 2);

    // y == +-0.
    if (iy | ly) == 0 {
        return match m {
            0 | 1 => y,      // atan2(+-0, +anything) = +-0
            2 => PI + TINY,  // atan2(+0, -anything) = pi
            _ => -PI - TINY, // atan2(-0, -anything) = -pi
        };
    }
    // x == +-0.
    if (ix | lx) == 0 {
        return if hy >> 31 == 0 {
            PIO2_HI + TINY
        } else {
            -PIO2_HI - TINY
        };
    }
    // x == +-inf.
    if ix == 0x7ff0_0000 {
        if iy == 0x7ff0_0000 {
            return match m {
                0 => PIO4_HI + TINY,        // atan2(+inf, +inf) = pi/4
                1 => -PIO4_HI - TINY,       // atan2(-inf, +inf) = -pi/4
                2 => 3.0 * PIO4_HI + TINY,  // atan2(+inf, -inf) = 3pi/4
                _ => -3.0 * PIO4_HI - TINY, // atan2(-inf, -inf) = -3pi/4
            };
        }
        return match m {
            0 => 0.0,        // atan2(+finite, +inf) = +0
            1 => -0.0,       // atan2(-finite, +inf) = -0
            2 => PI + TINY,  // atan2(+finite, -inf) = pi
            _ => -PI - TINY, // atan2(-finite, -inf) = -pi
        };
    }
    // y == +-inf (x finite).
    if iy == 0x7ff0_0000 {
        return if hy >> 31 == 0 {
            PIO2_HI + TINY
        } else {
            -PIO2_HI - TINY
        };
    }

    // e_atan2.c: compute z = atan(|y/x|) with exponent-difference guards.
    #[allow(clippy::cast_possible_wrap)] // top bits are exponent fields, < 0x7ff1_0000
    let k = (iy as i32 - ix as i32) >> 20;
    let z = if k > 60 {
        // |y/x| > 2^60: z saturates at pi/2 (the 0.5*PI_LO threads through
        // the case 2/3 corrections below to keep them near +-pi/2).
        PIO2_HI + 0.5 * PI_LO
    } else if hx >> 31 != 0 && k < -60 {
        // |y|/x < -2^60: z underflows to 0 against a negative x.
        0.0
    } else {
        atan((y / x).abs())
    };
    match m {
        0 => z,                // atan2(+, +)
        1 => -z,               // atan2(-, +): exact sign flip
        2 => PI - (z - PI_LO), // atan2(+, -)
        _ => (z - PI_LO) - PI, // atan2(-, -)
    }
}

// ---------------------------------------------------------------------------
// asin / acos shared rational kernel — FDLIBM e_asin.c pS/qS.
// ---------------------------------------------------------------------------

/// pS0..pS5 numerator coefficients (e_asin.c / e_acos.c).
const PS0: f64 = f64::from_bits(0x3FC5_5555_5555_5555); //  1.66666666666666657415e-01
const PS1: f64 = f64::from_bits(0xBFD4_D612_03EB_6F7D); // -3.25565818622400915405e-01
const PS2: f64 = f64::from_bits(0x3FC9_C155_0E88_4455); //  2.01212532134862925881e-01
const PS3: f64 = f64::from_bits(0xBFA4_8228_B568_8F3B); // -4.00555345006794114027e-02
const PS4: f64 = f64::from_bits(0x3F49_EFE0_7501_B288); //  7.91534994289814532176e-04
const PS5: f64 = f64::from_bits(0x3F02_3DE1_0DFD_F709); //  3.47933107596021167570e-05
/// qS1..qS4 denominator coefficients (e_asin.c / e_acos.c).
const QS1: f64 = f64::from_bits(0xC003_3A27_1C8A_2D4B); // -2.40339491173441421878e+00
const QS2: f64 = f64::from_bits(0x4000_2AE5_9C59_8AC8); //  2.02094576023350569471e+00
const QS3: f64 = f64::from_bits(0xBFE6_066C_1B8D_0159); // -6.88283971605453293030e-01
const QS4: f64 = f64::from_bits(0x3FB3_B8C5_B12E_9282); //  7.70381505559019352791e-02

/// R(z) = p(z)/q(z): the rational approximation of (asin(x) - x)/x^3
/// shared by `asin` and `acos`, in the exact FDLIBM nesting.
fn r_kernel(z: f64) -> f64 {
    let p = z * (PS0 + z * (PS1 + z * (PS2 + z * (PS3 + z * (PS4 + z * PS5)))));
    let q = 1.0 + z * (QS1 + z * (QS2 + z * (QS3 + z * QS4)));
    p / q
}

/// Arc sine, FDLIBM `e_asin.c`, bit-reproducible.
///
/// |x| < 0.5 uses `x + x*R(x^2)` directly; 0.5 <= |x| < 1 goes through
/// `pi/2 - 2*asin(sqrt((1-|x|)/2))` with the low-word-zeroed split of
/// `sqrt` (via `zero_lo`) and its exact correction term; `asin(+-1)`
/// returns +-pi/2 exactly; |x| > 1 and NaN return NaN. Design bound:
/// < 1 ulp.
#[must_use]
pub fn asin(x: f64) -> f64 {
    let hx = hi(x);
    let ix = hx & 0x7fff_ffff;

    if ix >= 0x3ff0_0000 {
        // |x| >= 1.
        if ((ix - 0x3ff0_0000) | lo(x)) == 0 {
            // asin(+-1) = +-pi/2 (computed as in e_asin.c, with inexact).
            return x * PIO2_HI + x * PIO2_LO;
        }
        return f64::NAN; // asin(|x| > 1) and asin(NaN)
    }

    if ix < 0x3fe0_0000 {
        // |x| < 0.5.
        if ix < 0x3e40_0000 {
            // |x| < 2^-27: asin(x) = x to double precision.
            return x;
        }
        let t = x * x;
        let w = r_kernel(t);
        return x + x * w;
    }

    // 0.5 <= |x| < 1 (e_asin.c sqrt path).
    let w = 1.0 - x.abs();
    let t = w * 0.5;
    let r = r_kernel(t);
    let s = t.sqrt();
    let t = if ix >= 0x3FEF_3333 {
        // |x| > 0.975: single-branch correction.
        PIO2_HI - (2.0 * (s + s * r) - PIO2_LO)
    } else {
        // Split s into a 20-significant-bit head w (low word zeroed) and
        // recover the exact tail c = (t - w*w)/(s + w).
        let w = zero_lo(s);
        let c = (t - w * w) / (s + w);
        let p = 2.0 * s * r - (PIO2_LO - 2.0 * c);
        let q = PIO4_HI - 2.0 * w;
        PIO4_HI - (p - q)
    };
    if hx >> 31 == 0 { t } else { -t }
}

/// Arc cosine, FDLIBM `e_acos.c`, bit-reproducible.
///
/// Three ranges: |x| < 0.5 via `pi/2 - (x + x*R(x^2))`; x < -0.5 via
/// `pi - 2*asin(sqrt((1+x)/2))`; x > 0.5 via `2*asin(sqrt((1-x)/2))`
/// with the high-word-truncated `df` sqrt split (via `zero_lo`).
/// `acos(1) = 0` and `acos(-1) = pi` exactly; |x| > 1 and NaN return
/// NaN. Design bound: < 1 ulp.
#[must_use]
pub fn acos(x: f64) -> f64 {
    let hx = hi(x);
    let ix = hx & 0x7fff_ffff;

    if ix >= 0x3ff0_0000 {
        // |x| >= 1.
        if ((ix - 0x3ff0_0000) | lo(x)) == 0 {
            // |x| == 1: acos(1) = 0, acos(-1) = pi (with inexact).
            if hx >> 31 == 0 {
                return 0.0;
            }
            return PI + 2.0 * PIO2_LO;
        }
        return f64::NAN; // acos(|x| > 1) and acos(NaN)
    }

    if ix < 0x3fe0_0000 {
        // |x| < 0.5.
        if ix <= 0x3c60_0000 {
            // |x| < 2^-57: acos(x) = pi/2 to double precision.
            return PIO2_HI + PIO2_LO;
        }
        let z = x * x;
        let r = r_kernel(z);
        return PIO2_HI - (x - (PIO2_LO - x * r));
    }

    if hx >> 31 != 0 {
        // x < -0.5: acos(x) = pi - 2 asin(sqrt((1+x)/2)).
        let z = (1.0 + x) * 0.5;
        let s = z.sqrt();
        let r = r_kernel(z);
        let w = r * s - PIO2_LO;
        return PI - 2.0 * (s + w);
    }

    // x > 0.5: acos(x) = 2 asin(sqrt((1-x)/2)), with df = sqrt head
    // (low word zeroed) and exact tail c = (z - df*df)/(s + df).
    let z = (1.0 - x) * 0.5;
    let s = z.sqrt();
    let df = zero_lo(s);
    let c = (z - df * df) / (s + df);
    let r = r_kernel(z);
    let w = r * s + c;
    2.0 * (df + w)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::cast_precision_loss)]

    use super::*;

    /// Map an f64 onto a monotone i64 scale so ulp distance is a
    /// subtraction (+0 and -0 coincide at 0).
    fn monotone(x: f64) -> i64 {
        #[allow(clippy::cast_possible_wrap)]
        let b = x.to_bits() as i64;
        if b < 0 { i64::MIN - b } else { b }
    }

    /// Bit distance in ulps; NaN vs NaN counts as equal.
    fn ulp_diff(a: f64, b: f64) -> u64 {
        assert_eq!(a.is_nan(), b.is_nan(), "NaN mismatch: {a:?} vs {b:?}");
        if a.is_nan() {
            return 0;
        }
        monotone(a).abs_diff(monotone(b))
    }

    /// Deterministic sweep covering every atan reduction interval: fine
    /// linear steps over [-4, 4] (10_000 intervals), the exact reduction
    /// thresholds +- a nudge, and log-spaced magnitudes 1e-300..1e300.
    fn sweep_points() -> Vec<f64> {
        let mut pts = Vec::new();
        for i in 0..=10_000_u32 {
            pts.push(-4.0 + f64::from(i) * (8.0 / 10_000.0));
        }
        let boundaries = [
            7.0 / 16.0,
            11.0 / 16.0,
            19.0 / 16.0,
            39.0 / 16.0,
            73_786_976_294_838_206_464.0, // 2^66
        ];
        for b in boundaries {
            for d in [-1.0e-6, 0.0, 1.0e-6] {
                let v = b * (1.0 + d);
                pts.push(v);
                pts.push(-v);
            }
        }
        let mut m = 1.0e-300;
        while m <= 1.0e300 {
            pts.push(m);
            pts.push(-m);
            m *= 10.0;
        }
        pts
    }

    #[test]
    fn atan_matches_std_within_2_ulp() {
        let mut max = 0;
        for x in sweep_points() {
            let d = ulp_diff(atan(x), x.atan());
            max = max.max(d);
            assert!(d <= 2, "atan({x:e}): {d} ulp");
        }
        println!("atan max ulp vs std: {max}");
    }

    #[test]
    fn atan_special_values() {
        assert_eq!(atan(0.0).to_bits(), 0.0_f64.to_bits());
        assert_eq!(atan(-0.0).to_bits(), (-0.0_f64).to_bits());
        assert_eq!(
            atan(f64::INFINITY).to_bits(),
            f64::INFINITY.atan().to_bits()
        );
        assert_eq!(
            atan(f64::NEG_INFINITY).to_bits(),
            f64::NEG_INFINITY.atan().to_bits()
        );
        assert!(atan(f64::NAN).is_nan());
        // Denormals pass straight through the |x| < 2^-27 shortcut.
        assert_eq!(atan(5e-324).to_bits(), 5e-324_f64.to_bits());
    }

    #[test]
    fn atan_is_bitwise_odd() {
        for x in sweep_points() {
            assert_eq!(atan(-x).to_bits(), (-atan(x)).to_bits(), "x = {x:e}");
        }
    }

    #[test]
    fn atan2_special_grid_matches_std_exactly() {
        let specials = [
            0.0,
            -0.0,
            0.5,
            -0.5,
            1.0,
            -1.0,
            1.0e300,
            -1.0e300,
            1.0e-300,
            -1.0e-300,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NAN,
        ];
        let mut max = 0;
        for &y in &specials {
            for &x in &specials {
                let mine = atan2(y, x);
                let theirs = y.atan2(x);
                if y.is_nan() || x.is_nan() {
                    // NaN result bits (payload) are not IEEE-specified.
                    assert!(mine.is_nan() && theirs.is_nan(), "atan2({y:?}, {x:?})");
                } else if y == 0.0 || x == 0.0 || y.is_infinite() || x.is_infinite() {
                    // IEEE 754-specified cases: exact bit equality.
                    assert_eq!(
                        mine.to_bits(),
                        theirs.to_bits(),
                        "atan2({y:?}, {x:?}) = {mine:?} vs std {theirs:?}"
                    );
                } else {
                    let d = ulp_diff(mine, theirs);
                    max = max.max(d);
                    assert!(d <= 2, "atan2({y:e}, {x:e}): {d} ulp");
                }
            }
        }
        println!("atan2 special-grid max ulp vs std (general cells): {max}");
    }

    #[test]
    fn atan2_general_grid_matches_std_within_2_ulp() {
        let mut max = 0;
        for i in 0..=120_u32 {
            for j in 0..=120_u32 {
                let y = -3.0 + f64::from(i) * 0.05;
                let x = -3.0 + f64::from(j) * 0.05;
                if y == 0.0 || x == 0.0 {
                    continue; // exact cases covered above
                }
                let d = ulp_diff(atan2(y, x), y.atan2(x));
                max = max.max(d);
                assert!(d <= 2, "atan2({y}, {x}): {d} ulp");
            }
        }
        println!("atan2 general-grid max ulp vs std: {max}");
    }

    #[test]
    fn asin_matches_std_within_2_ulp() {
        let mut max = 0;
        for i in 0..=10_000_u32 {
            let x = -1.0 + f64::from(i) * (2.0 / 10_000.0);
            let d = ulp_diff(asin(x), x.asin());
            max = max.max(d);
            assert!(d <= 2, "asin({x:e}): {d} ulp");
        }
        // Range-boundary neighborhoods (0.5 and 0.975 splits).
        for x in [0.5, 0.975] {
            for d in [-1.0e-9, 0.0, 1.0e-9] {
                let v = x + d;
                for v in [v, -v] {
                    let d = ulp_diff(asin(v), v.asin());
                    max = max.max(d);
                    assert!(d <= 2, "asin({v:e}): {d} ulp");
                }
            }
        }
        println!("asin max ulp vs std: {max}");
    }

    #[test]
    fn asin_domain_edges() {
        // asin(+-1) = +-pi/2 exactly, bit-equal to std.
        assert_eq!(asin(1.0).to_bits(), 1.0_f64.asin().to_bits());
        assert_eq!(asin(-1.0).to_bits(), (-1.0_f64).asin().to_bits());
        assert_eq!(asin(1.0).to_bits(), std::f64::consts::FRAC_PI_2.to_bits());
        // |x| slightly above 1: NaN.
        assert!(asin(1.000_000_000_000_000_2).is_nan());
        assert!(asin(-1.000_000_000_000_000_2).is_nan());
        assert!(asin(f64::NAN).is_nan());
        assert!(asin(f64::INFINITY).is_nan());
        // Denormals pass through the |x| < 2^-27 shortcut.
        assert_eq!(asin(5e-324).to_bits(), 5e-324_f64.to_bits());
        assert_eq!(asin(-1.0e-310).to_bits(), (-1.0e-310_f64).to_bits());
    }

    #[test]
    fn acos_matches_std_within_2_ulp() {
        let mut max = 0;
        for i in 0..=10_000_u32 {
            let x = -1.0 + f64::from(i) * (2.0 / 10_000.0);
            let d = ulp_diff(acos(x), x.acos());
            max = max.max(d);
            assert!(d <= 2, "acos({x:e}): {d} ulp");
        }
        println!("acos max ulp vs std: {max}");
    }

    #[test]
    fn acos_domain_edges() {
        assert_eq!(acos(1.0).to_bits(), 0.0_f64.to_bits());
        assert_eq!(acos(-1.0).to_bits(), (-1.0_f64).acos().to_bits());
        assert_eq!(acos(-1.0).to_bits(), std::f64::consts::PI.to_bits());
        assert!(acos(1.000_000_000_000_000_2).is_nan());
        assert!(acos(-1.000_000_000_000_000_2).is_nan());
        assert!(acos(f64::NAN).is_nan());
        // Tiny and denormal inputs collapse to pi/2.
        assert_eq!(
            acos(5e-324).to_bits(),
            std::f64::consts::FRAC_PI_2.to_bits()
        );
        assert_eq!(
            acos(-1.0e-310).to_bits(),
            std::f64::consts::FRAC_PI_2.to_bits()
        );
    }

    #[test]
    fn acos_reflection_sums_to_pi() {
        // Design target: |acos(x) + acos(-x) - pi| within ~4e-16. One ulp
        // of pi is 4.44e-16, so the bit-meaningful form of that bound is
        // "at most 1 ulp from fl(pi)" — which is also what platform libm
        // achieves (the sum is one rounding away from exact).
        for i in 0..=2_000_u32 {
            let x = -1.0 + f64::from(i) * (2.0 / 2_000.0);
            let sum = acos(x) + acos(-x);
            let d = ulp_diff(sum, std::f64::consts::PI);
            assert!(d <= 1, "acos({x}) + acos({}) = {sum}: {d} ulp from pi", -x);
        }
    }
}
