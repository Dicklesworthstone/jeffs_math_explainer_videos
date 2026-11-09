//! `pow` — a faithful port of FDLIBM `e_pow`, plus the C99 `pow(1, y)`
//! amendment — and fixed-order integer `powi`, bit-identical on every
//! certified target.
//!
//! Method (FDLIBM stages, kept in order):
//! 1. IEEE/C99 special cases: `x^0`, `1^y`, NaN propagation, the
//!    odd/even-integer classification of `y` for negative bases, `±inf`
//!    and `±0` in either slot, `y ∈ {±1, 2, 0.5}` shortcuts.
//! 2. `|y| >= 2^31` shortcuts (forced over/underflow unless `|x|` is
//!    within 2^-20 of 1, which gets a short log series).
//! 3. `log2(|x|)` in extended precision: `bp`/`dp_h`/`dp_l` interval
//!    tables, the `ss = u/v` split arithmetic, L1..L6 polynomial.
//! 4. `y * log2(|x|)` as a split multiply with the overflow (`z > 1024`)
//!    / underflow (`z <= -1075`) `o_threshold`/`u_threshold` logic.
//! 5. `2^(p_h + p_l)` reconstruction with the P1..P5 polynomial and
//!    exponent-bit scaling (denormal outputs via `scalbn`).
//!
//! Design accuracy bound (verified downstream against the committed
//! mpmath vectors): < 2 ulp overall; the IEEE special-case table is
//! exact. Known, inherited FDLIBM caveat: in the sliver `|y| > 2^31`
//! with `|x - 1| < 2^-20`, the short log series carries only ~53 bits
//! of `log2(x)`, so the error grows with `|y*log2(x)|` (worst case a
//! few hundred ulp near the overflow/underflow bounds). All magic
//! constants are exact bit patterns; each comment gives the published
//! FDLIBM decimal.
//!
//! §17.3 audit ruling: `pow` remains scalar. Its five stages are dominated
//! by per-lane IEEE/C99 classification and split reconstruction, and the
//! current high-volume color consumers already use exact transfer tables.
//! No production caller presents a coherent two-input batch that justifies
//! eagerly evaluating mutually exclusive branches.

use crate::bits::{from_parts, hi, horner, lo, with_hi, zero_lo};
use crate::exp_log::scalbn;

const HUGE: f64 = 1.0e300;
const TINY: f64 = 1.0e-300;

/// Interval endpoints for the log2 reduction: `|x|` is scaled against 1
/// or 1.5.
const BP: [f64; 2] = [1.0, 1.5];
/// `log2(1.5)` head, 5.84962487220764160156e-01 (26 bits).
const DP_H: [f64; 2] = [0.0, f64::from_bits(0x3FE2_B803_4000_0000)];
/// `log2(1.5)` tail, 1.35003920212974897128e-08.
const DP_L: [f64; 2] = [0.0, f64::from_bits(0x3E4C_FDEB_43CF_D006)];

/// 2^53 (FDLIBM `two53`), denormal pre-scale.
const TWO53: f64 = f64::from_bits(0x4340_0000_0000_0000);

// Polynomial for (3/2)*(log(x) - 2s - 2/3*s^3), L1..L6.
const L1: f64 = f64::from_bits(0x3FE3_3333_3333_3303); // 5.99999999999994648725e-01
const L2: f64 = f64::from_bits(0x3FDB_6DB6_DB6F_ABFF); // 4.28571428578550184252e-01
const L3: f64 = f64::from_bits(0x3FD5_5555_518F_264D); // 3.33333329818377432918e-01
const L4: f64 = f64::from_bits(0x3FD1_7460_A91D_4101); // 2.72728123808534006489e-01
const L5: f64 = f64::from_bits(0x3FCD_864A_93C9_DB65); // 2.30660745775561754067e-01
const L6: f64 = f64::from_bits(0x3FCA_7E28_4A45_4EEF); // 2.06975017800338417784e-01

// exp2 minimax polynomial P1..P5 (same table as e_exp).
const P1: f64 = f64::from_bits(0x3FC5_5555_5555_553E); //  1.66666666666666019037e-01
const P2: f64 = f64::from_bits(0xBF66_C16C_16BE_BD93); // -2.77777777770155933842e-03
const P3: f64 = f64::from_bits(0x3F11_566A_AF25_DE2C); //  6.61375632143793436117e-05
const P4: f64 = f64::from_bits(0xBEBB_BD41_C5D2_6BF1); // -1.65339022054652515390e-06
const P5: f64 = f64::from_bits(0x3E66_3769_72BE_A4D0); //  4.13813679705723846039e-08

/// ln2 = 6.93147180559945286227e-01 and its head/tail split.
const LG2: f64 = f64::from_bits(0x3FE6_2E42_FEFA_39EF);
const LG2_H: f64 = f64::from_bits(0x3FE6_2E43_0000_0000); //  6.93147182464599609375e-01
const LG2_L: f64 = f64::from_bits(0xBE20_5C61_0CA8_6C39); // -1.90465429995776804525e-09

/// `-(1024 - log2(ovfl + 0.5ulp))`, 8.0085662595372944372e-17: the
/// borderline-overflow guard.
const OVT: f64 = f64::from_bits(0x3C97_1547_652B_82FE);

/// `cp = 2/(3 ln2)` = 9.61796693925975554329e-01 and its head/tail.
const CP: f64 = f64::from_bits(0x3FEE_C709_DC3A_03FD);
const CP_H: f64 = f64::from_bits(0x3FEE_C709_E000_0000); //  9.61796700954437255859e-01
const CP_L: f64 = f64::from_bits(0xBE3E_2FE0_145B_01F5); // -7.02846165095275826516e-09

/// `1/ln2` = 1.44269504088896338700e+00 and its 24-bit head/tail.
const IVLN2: f64 = f64::from_bits(0x3FF7_1547_652B_82FE);
const IVLN2_H: f64 = f64::from_bits(0x3FF7_1547_6000_0000); // 1.44269502162933349609e+00
const IVLN2_L: f64 = f64::from_bits(0x3E54_AE0B_F85D_DF44); // 1.92596299112661746887e-08

/// Nearest double to 1/3 (FDLIBM's literal 0.3333333333333333333333).
const THIRD: f64 = f64::from_bits(0x3FD5_5555_5555_5555);

/// `x^n` with an owned, fixed multiplication order.
///
/// The standard library's `f64::powi` lowers through an
/// unspecified-precision intrinsic on the pinned nightly. This definition uses
/// exponentiation by squaring, consumes exponent bits least-significant first,
/// and performs one reciprocal after the positive power for a negative
/// exponent. `i32::MIN` is handled without signed overflow through
/// [`i32::unsigned_abs`].
#[must_use]
pub fn powi(mut x: f64, n: i32) -> f64 {
    let reciprocal = n < 0;
    let mut exponent = n.unsigned_abs();
    let mut result = 1.0;
    while exponent != 0 {
        if exponent & 1 != 0 {
            result *= x;
        }
        exponent >>= 1;
        if exponent != 0 {
            x *= x;
        }
    }
    if reciprocal { 1.0 / result } else { result }
}

/// x^y — FDLIBM `e_pow` with the C99 `pow(1, y) = 1` amendment.
///
/// All IEEE/C99 special cases hold exactly: `pow(x, 0) = 1` (even for
/// NaN x), `pow(1, y) = 1` (even for NaN y), the `±0`/`±inf` sign rules
/// via the odd/even-integer classification of `y`, and
/// `pow(negative, non-integer) = NaN`. Design accuracy < 2 ulp.
#[must_use]
pub fn pow(x: f64, y: f64) -> f64 {
    let hx = hi(x) as i32;
    let lx = lo(x);
    let hy = hi(y) as i32;
    let ly = lo(y);
    let ix = hx & 0x7fff_ffff;
    let iy = hy & 0x7fff_ffff;

    // ---- stage 1: special cases -------------------------------------

    // x^0 = 1 for every x, including NaN.
    if (iy as u32 | ly) == 0 {
        return 1.0;
    }

    // 1^y = 1 for every y, including NaN (C99; FDLIBM predates this).
    if hx == 0x3ff0_0000 && lx == 0 {
        return 1.0;
    }

    // NaN in either argument (now that x^0 and 1^y are out of the way).
    if ix > 0x7ff0_0000
        || (ix == 0x7ff0_0000 && lx != 0)
        || iy > 0x7ff0_0000
        || (iy == 0x7ff0_0000 && ly != 0)
    {
        return x + y;
    }

    // Classify y for negative x:
    //   yisint = 0 — y is not an integer
    //   yisint = 1 — y is an odd integer
    //   yisint = 2 — y is an even integer
    let mut yisint: i32 = 0;
    if hx < 0 {
        if iy >= 0x4340_0000 {
            yisint = 2; // |y| >= 2^52: always an even integer
        } else if iy >= 0x3ff0_0000 {
            let k = (iy >> 20) - 0x3ff; // exponent of y
            if k > 20 {
                let j = ly >> (52 - k);
                if (j << (52 - k)) == ly {
                    yisint = 2 - (j & 1) as i32;
                }
            } else if ly == 0 {
                let j = iy >> (20 - k);
                if (j << (20 - k)) == iy {
                    yisint = 2 - (j & 1);
                }
            }
        }
    }

    // Special values of y.
    if ly == 0 {
        if iy == 0x7ff0_0000 {
            // y is ±inf
            if ((ix - 0x3ff0_0000) as u32 | lx) == 0 {
                return 1.0; // (±1)^±inf = 1 (C99)
            } else if ix >= 0x3ff0_0000 {
                // (|x| > 1)^±inf = inf, 0
                return if hy >= 0 { y } else { 0.0 };
            }
            // (|x| < 1)^∓inf = inf, 0
            return if hy < 0 { -y } else { 0.0 };
        }
        if iy == 0x3ff0_0000 {
            // y is ±1
            return if hy < 0 { 1.0 / x } else { x };
        }
        if hy == 0x4000_0000 {
            return x * x; // y is 2
        }
        if hy == 0x3fe0_0000 && hx >= 0 {
            return x.sqrt(); // y is 0.5, x >= +0
        }
    }

    let mut ax = x.abs();
    // Special values of x: ±0, ±inf, -1 (+1 handled above).
    if lx == 0 && (ix == 0x7ff0_0000 || ix == 0 || ix == 0x3ff0_0000) {
        let mut z = ax;
        if hy < 0 {
            z = 1.0 / z; // z = (1/|x|)^|y|
        }
        if hx < 0 {
            if ((ix - 0x3ff0_0000) | yisint) == 0 {
                z = f64::NAN; // (-1)^non-int is NaN
            } else if yisint == 1 {
                z = -z; // (x < 0)^odd = -(|x|^odd)
            }
        }
        return z;
    }

    let n_sign = (hx >> 31) + 1; // 0 for x < 0, 1 for x > 0

    // (x < 0)^(non-integer) is NaN.
    if (n_sign | yisint) == 0 {
        return f64::NAN;
    }

    // Sign of the result: -1 only for (negative)^(odd integer).
    let s = if (n_sign | (yisint - 1)) == 0 {
        -1.0
    } else {
        1.0
    };

    // ---- stage 2/3: t1 + t2 = log2(|x|) in extended precision --------

    let (t1, t2) = if iy > 0x41e0_0000 {
        // |y| > 2^31
        if iy > 0x43f0_0000 {
            // |y| > 2^64: must over/underflow
            if ix <= 0x3fef_ffff {
                return if hy < 0 { HUGE * HUGE } else { TINY * TINY };
            }
            if ix >= 0x3ff0_0000 {
                return if hy > 0 { HUGE * HUGE } else { TINY * TINY };
            }
        }
        // Over/underflow if x is not close to one.
        if ix < 0x3fef_ffff {
            return if hy < 0 {
                s * HUGE * HUGE
            } else {
                s * TINY * TINY
            };
        }
        if ix > 0x3ff0_0000 {
            return if hy > 0 {
                s * HUGE * HUGE
            } else {
                s * TINY * TINY
            };
        }
        // |1 - x| <= 2^-20: log(x) by x - x^2/2 + x^3/3 - x^4/4.
        let t = ax - 1.0; // t has 20 trailing zeros
        let w = (t * t) * (0.5 - t * (THIRD - t * 0.25));
        let u = IVLN2_H * t; // ivln2_h has 21 significant bits
        let v = t * IVLN2_L - w * IVLN2;
        let t1 = zero_lo(u + v);
        (t1, v - (t1 - u))
    } else {
        let mut n: i32 = 0;
        let mut ix2 = ix;
        // Take care of subnormal |x|.
        if ix2 < 0x0010_0000 {
            ax *= TWO53;
            n -= 53;
            ix2 = hi(ax) as i32;
        }
        n += (ix2 >> 20) - 0x3ff;
        let j = ix2 & 0x000f_ffff;
        // Determine the interval: bp[k] = 1 or 1.5.
        ix2 = j | 0x3ff0_0000; // normalize ix
        let k: usize;
        if j <= 0x3_988e {
            k = 0; // |x| < sqrt(3/2)
        } else if j < 0xb_b67a {
            k = 1; // |x| < sqrt(3)
        } else {
            k = 0;
            n += 1;
            ix2 -= 0x0010_0000;
        }
        ax = with_hi(ax, ix2 as u32);

        // ss = s_h + s_l = (ax - bp[k]) / (ax + bp[k]), split.
        let u = ax - BP[k];
        let v = 1.0 / (ax + BP[k]);
        let ss = u * v;
        let s_h = zero_lo(ss);
        // t_h = ax + bp[k], high part built directly from the bits.
        let t_h = from_parts(
            (((ix2 >> 1) | 0x2000_0000) + 0x0008_0000 + ((k as i32) << 18)) as u32,
            0,
        );
        let t_l = ax - (t_h - BP[k]);
        let s_l = v * ((u - s_h * t_h) - s_h * t_l);
        // Compute log(ax): L1..L6 polynomial in ss^2.
        let s2 = ss * ss;
        let mut r = s2 * s2 * horner(s2, &[L6, L5, L4, L3, L2, L1]);
        r += s_l * (s_h + ss);
        let s2 = s_h * s_h;
        let t_h = zero_lo(3.0 + s2 + r);
        let t_l = r - ((t_h - 3.0) - s2);
        // u + v = ss*(1 + ...), split.
        let u = s_h * t_h;
        let v = s_l * t_h + t_l * ss;
        // 2/(3 log2) * (ss + ...): p_h + p_l.
        let p_h = zero_lo(u + v);
        let p_l = v - (p_h - u);
        let z_h = CP_H * p_h; // cp_h + cp_l = 2/(3*log2)
        let z_l = CP_L * p_h + p_l * CP + DP_L[k];
        // log2(ax) = (ss + ...)*2/(3*log2) = n + dp_h + z_h + z_l.
        let t = f64::from(n);
        let t1 = zero_lo(((z_h + z_l) + DP_H[k]) + t);
        (t1, z_l - (((t1 - t) - DP_H[k]) - z_h))
    };

    // ---- stage 4: split y into y1 + y2, compute (y1+y2)*(t1+t2) ------

    let y1 = zero_lo(y);
    let p_l = (y - y1) * t1 + y * t2;
    let mut p_h = y1 * t1;
    let z = p_l + p_h;
    let j = hi(z) as i32;
    let i = lo(z);
    if j >= 0x4090_0000 {
        // z >= 1024
        if ((j - 0x4090_0000) as u32 | i) != 0 {
            return s * HUGE * HUGE; // z > 1024: overflow
        }
        if p_l + OVT > z - p_h {
            return s * HUGE * HUGE; // borderline overflow
        }
    } else if (j & 0x7fff_ffff) >= 0x4090_cc00 {
        // |z| >= 1075
        if (j.wrapping_sub(0xc090_cc00u32 as i32) as u32 | i) != 0 {
            return s * TINY * TINY; // z < -1075: underflow
        }
        if p_l <= z - p_h {
            return s * TINY * TINY; // borderline underflow
        }
    }

    // ---- stage 5: 2^(p_h + p_l) --------------------------------------

    let i = j & 0x7fff_ffff;
    let k = (i >> 20) - 0x3ff;
    let mut n: i32 = 0;
    if i > 0x3fe0_0000 {
        // |z| > 0.5: set n = [z + 0.5] and reduce p_h by n.
        n = j + (0x0010_0000 >> (k + 1));
        let k = ((n & 0x7fff_ffff) >> 20) - 0x3ff; // new k for n
        let t = from_parts((n & !(0x000f_ffff >> k)) as u32, 0);
        n = ((n & 0x000f_ffff) | 0x0010_0000) >> (20 - k);
        if j < 0 {
            n = -n;
        }
        p_h -= t;
    }
    let t = zero_lo(p_l + p_h);
    let u = t * LG2_H;
    let v = (p_l - (t - p_h)) * LG2 + t * LG2_L;
    let z = u + v;
    let w = v - (z - u);
    let t = z * z;
    let t1 = z - t * horner(t, &[P5, P4, P3, P2, P1]);
    let r = (z * t1) / (t1 - 2.0) - (w + z * w);
    let z = 1.0 - (r - z);
    let j = (hi(z) as i32) + (n << 20);
    if (j >> 20) <= 0 {
        s * scalbn(z, n) // subnormal output
    } else {
        s * with_hi(z, hi(z).wrapping_add((n as u32) << 20))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn powi_has_fixed_order_and_complete_integer_domain() {
        let x = f64::from_bits(0x3ff0_0000_0000_0001);
        assert_eq!(powi(x, 3).to_bits(), (x * (x * x)).to_bits());
        assert_eq!(powi(x, 5).to_bits(), (x * ((x * x) * (x * x))).to_bits());

        assert_eq!(powi(f64::NAN, 0).to_bits(), 1.0f64.to_bits());
        assert!(powi(f64::NAN, 1).is_nan());
        assert_eq!(powi(-2.0, 3).to_bits(), (-8.0f64).to_bits());
        assert_eq!(powi(-2.0, -3).to_bits(), (-0.125f64).to_bits());
        assert_eq!(powi(-0.0, 3).to_bits(), (-0.0f64).to_bits());
        assert_eq!(powi(-0.0, -3).to_bits(), f64::NEG_INFINITY.to_bits());
        assert_eq!(powi(1.0, i32::MIN).to_bits(), 1.0f64.to_bits());
        assert_eq!(powi(-1.0, i32::MIN).to_bits(), 1.0f64.to_bits());
    }

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
    fn pow_grid_vs_std() {
        // 100 log-spaced positive bases x 100 exponents in [-40, 40]
        // (integers and fractions both land in the grid).
        let b0 = 0.001f64.to_bits();
        let b1 = 1000.0f64.to_bits();
        let step = (b1 - b0) / 99;
        let mut max = 0u128;
        for i in 0..100u64 {
            let x = f64::from_bits(b0 + step * i);
            for j in 0..100i32 {
                let y = -40.0 + f64::from(j) * 80.0 / 99.0;
                let d = ulp_diff(pow(x, y), x.powf(y));
                if d > max {
                    max = d;
                }
            }
        }
        println!("pow positive-grid max ulp vs std: {max}");
        assert!(max <= 4, "pow grid max ulp {max}");
    }

    #[test]
    fn pow_negative_base_grid_vs_std() {
        // Negative bases: integer y must follow the parity sign rules,
        // non-integer y must be NaN — both match std bitwise/NaN-wise.
        let mut max = 0u128;
        for i in 0..100i32 {
            let x = -10.0 + f64::from(i) * 9.75 / 99.0; // [-10, -0.25]
            for j in -12..=12i32 {
                let y = f64::from(j);
                let d = ulp_diff(pow(x, y), x.powf(y));
                if d > max {
                    max = d;
                }
                // Non-integer exponent: NaN on both sides.
                let yf = y + 0.5;
                assert!(pow(x, yf).is_nan());
                assert!(x.powf(yf).is_nan());
            }
        }
        println!("pow negative-base grid max ulp vs std: {max}");
        assert!(max <= 4, "pow negative-base grid max ulp {max}");
    }

    #[test]
    fn pow_power_of_two_bases_exact() {
        // 2^k and 0.5^k round-trip through the log2/exp2 stages exactly.
        for j in -10..=10i32 {
            let y = f64::from(j);
            let expect = f64::from_bits(u64::from((j + 1023) as u32) << 52); // 2^j
            assert_eq!(pow(2.0, y).to_bits(), expect.to_bits(), "2^{j}");
            let expect_neg = f64::from_bits(u64::from((1023 - j) as u32) << 52); // 2^-j
            assert_eq!(pow(0.5, y).to_bits(), expect_neg.to_bits(), "0.5^{j}");
        }
        // Negative base, integer exponents: exact with parity sign.
        assert_eq!(pow(-2.0, 3.0).to_bits(), (-8.0f64).to_bits());
        assert_eq!(pow(-2.0, 4.0).to_bits(), 16.0f64.to_bits());
        assert_eq!(pow(-2.0, -3.0).to_bits(), (-0.125f64).to_bits());
        assert_eq!(pow(-2.0, -4.0).to_bits(), 0.0625f64.to_bits());
    }

    #[test]
    fn pow_special_case_table() {
        let inf = f64::INFINITY;
        let nan = f64::NAN;
        // Each row: (x, y, expected). Bit-exact, and std must agree.
        let table: &[(f64, f64, f64)] = &[
            (nan, 0.0, 1.0),
            (nan, -0.0, 1.0),
            (1.0, nan, 1.0),
            (1.0, inf, 1.0),
            (1.0, -inf, 1.0),
            (-1.0, inf, 1.0),
            (-1.0, -inf, 1.0),
            (0.0, -1.0, inf),
            (-0.0, -1.0, -inf),
            (0.0, -2.0, inf),
            (-0.0, -2.0, inf),
            (0.0, 3.0, 0.0),
            (-0.0, 3.0, -0.0),
            (-0.0, 4.0, 0.0),
            (0.0, inf, 0.0),
            (0.0, -inf, inf),
            (2.0, inf, inf),
            (2.0, -inf, 0.0),
            (0.5, inf, 0.0),
            (0.5, -inf, inf),
            (inf, 2.0, inf),
            (inf, -2.0, 0.0),
            (-inf, 3.0, -inf),
            (-inf, -3.0, -0.0),
            (-inf, 2.0, inf),
            (-2.0, 3.0, -8.0),
            (10.0, 400.0, inf),
            (10.0, -400.0, 0.0),
            (-10.0, 401.0, -inf),
            (-10.0, -401.0, -0.0),
        ];
        for &(x, y, expect) in table {
            let got = pow(x, y);
            assert_eq!(
                got.to_bits(),
                expect.to_bits(),
                "pow({x:?}, {y:?}) = {got:?}, want {expect:?}"
            );
            assert_eq!(
                x.powf(y).to_bits(),
                expect.to_bits(),
                "std disagrees on pow({x:?}, {y:?})"
            );
        }
        // NaN rows (result NaN, payload not compared).
        assert!(pow(nan, 1.0).is_nan());
        assert!(pow(2.0, nan).is_nan());
        assert!(pow(-2.0, 0.5).is_nan());
        assert!(pow(-1.5, 2.5).is_nan());
        assert!(pow(-1.0, 0.5).is_nan());
    }

    #[test]
    fn pow_huge_y_paths() {
        // |y| > 2^31 and > 2^64 shortcut paths, incl. the near-1 series.
        let y31 = 3.0e9f64;
        let y64 = 3.0e19f64;
        for &(x, y) in &[
            (2.0, y31),
            (2.0, -y31),
            (0.5, y31),
            (0.5, -y31),
            (2.0, y64),
            (0.5, y64),
            (1.0 + 2.0e-10, y31),
            (1.0 + 2.0e-10, -y31),
        ] {
            let d = ulp_diff(pow(x, y), x.powf(y));
            assert!(d <= 4, "pow({x}, {y}) ulp {d}");
        }
        // Known FDLIBM limitation, ported faithfully: in the sliver
        // |y| > 2^31 AND |x-1| < 2^-20, the short log series rounds
        // u = ivln2_h*t once (t can carry 33 bits), so log2(x) holds
        // only ~53 bits and the error scales with |y*log2(x)|. Bound:
        // |z| <= 1024 gives at most ~355 ulp; modern std pow is nearly
        // correctly rounded, so the observed gap is real FDLIBM error.
        for &(x, y) in &[(1.0 + 1.0e-7, y31), (1.0 - 1.0e-7, y31)] {
            let d = ulp_diff(pow(x, y), x.powf(y));
            assert!(d <= 512, "pow({x}, {y}) ulp {d}");
        }
    }

    #[test]
    fn pow_subnormal_results() {
        // Results deep in the denormal range still match std closely.
        for &(x, y) in &[(2.0, -1070.5), (10.0, -320.7), (0.5, 1071.3)] {
            let d = ulp_diff(pow(x, y), x.powf(y));
            assert!(d <= 4, "pow({x}, {y}) ulp {d}");
        }
    }
}
