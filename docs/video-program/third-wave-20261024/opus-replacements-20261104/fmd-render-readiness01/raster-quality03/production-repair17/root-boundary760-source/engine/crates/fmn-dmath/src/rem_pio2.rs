//! Argument reduction: x = n*(pi/2) + y0 + y1, |y0| <= pi/4.
//!
//! Structure: FDLIBM `__ieee754_rem_pio2` (as modernized in musl's
//! `__rem_pio2.c`) plus the integer reduction kernel `__kernel_rem_pio2`
//! (musl `__rem_pio2_large.c`, specialized to the double-precision jk = 4 /
//! prec = 1 configuration). All constants are pinned by their exact IEEE 754
//! bit patterns, so the reduction is bit-identical on every target.
//!
//! Reduction stages:
//! 1. |x| <= pi/4 — no reduction needed.
//! 2. |x| ~<= 9pi/4 — direct Cody–Waite subtraction of 1..4 multiples of
//!    pi/2 using a two-part (pio2_1, pio2_1t) split; k*pio2_1 is exact
//!    because pio2_1 carries 33 trailing zero bits.
//! 3. |x| < 2^20 * pi/2 — "medium": n = rint(x * 2/pi) via the round-to-int
//!    trick, then up to three Cody–Waite stages (pio2_1/1t, pio2_2/2t,
//!    pio2_3/3t), each stage extending the effective precision of pi/2 to
//!    85, 118, then 151 bits, escalating only when cancellation is detected
//!    from the exponent drop of the partial remainder.
//! 4. Anything larger — Payne–Hanek style: split |x| into 24-bit digits and
//!    multiply against the stored binary digits of 2/pi in the integer
//!    kernel, keeping only the fractional part that survives.

use crate::bits::hi;

// ---------------------------------------------------------------------------
// Constants (bit-exact; decimal values in comments are the FDLIBM printouts)
// ---------------------------------------------------------------------------

/// 2/pi, rounded. 6.36619772367581382433e-01
const INV_PIO2: f64 = f64::from_bits(0x3FE4_5F30_6DC9_C883);
/// First 33 bits of pi/2. 1.57079632673412561417e+00
const PIO2_1: f64 = f64::from_bits(0x3FF9_21FB_5440_0000);
/// pi/2 - PIO2_1, rounded. 6.07710050650619224932e-11
const PIO2_1T: f64 = f64::from_bits(0x3DD0_B461_1A62_6331);
/// Second 33-bit chunk of pi/2. 6.07710050630396597660e-11
const PIO2_2: f64 = f64::from_bits(0x3DD0_B461_1A60_0000);
/// pi/2 - (PIO2_1 + PIO2_2), rounded. 2.02226624879595063154e-21
const PIO2_2T: f64 = f64::from_bits(0x3BA3_198A_2E03_7073);
/// Third 33-bit chunk of pi/2. 2.02226624871116645580e-21
const PIO2_3: f64 = f64::from_bits(0x3BA3_198A_2E00_0000);
/// pi/2 - (PIO2_1 + PIO2_2 + PIO2_3), rounded. 8.47842766036889956997e-32
const PIO2_3T: f64 = f64::from_bits(0x397B_839A_2520_49C1);
/// pi/4, rounded. 0x1.921fb54442d18p-1 (guard bound in the medium path).
const PIO4: f64 = f64::from_bits(0x3FE9_21FB_5444_2D18);
/// 0x1.8p52: adding then subtracting rounds to the nearest integer
/// (round-to-nearest-even, which Rust guarantees).
const TOINT: f64 = f64::from_bits(0x4338_0000_0000_0000);
/// 2^24.
const TWO24: f64 = f64::from_bits(0x4170_0000_0000_0000);
/// 2^-24.
const TWOM24: f64 = f64::from_bits(0x3E70_0000_0000_0000);

/// The binary digits of 2/pi in 24-bit chunks: ipio2 from FDLIBM
/// `__kernel_rem_pio2`. 66 entries suffice for f64 (the full table extends
/// further only for ld128).
const IPIO2: [i32; 66] = [
    0x00A2_F983,
    0x006E_4E44,
    0x0015_29FC,
    0x0027_57D1,
    0x00F5_34DD,
    0x00C0_DB62,
    0x0095_993C,
    0x0043_9041,
    0x00FE_5163,
    0x00AB_DEBB,
    0x00C5_61B7,
    0x0024_6E3A,
    0x0042_4DD2,
    0x00E0_0649,
    0x002E_EA09,
    0x00D1_921C,
    0x00FE_1DEB,
    0x001C_B129,
    0x00A7_3EE8,
    0x0082_35F5,
    0x002E_BB44,
    0x0084_E99C,
    0x0070_26B4,
    0x005F_7E41,
    0x0039_91D6,
    0x0039_8353,
    0x0039_F49C,
    0x0084_5F8B,
    0x00BD_F928,
    0x003B_1FF8,
    0x0097_FFDE,
    0x0005_980F,
    0x00EF_2F11,
    0x008B_5A0A,
    0x006D_1F6D,
    0x0036_7ECF,
    0x0027_CB09,
    0x00B7_4F46,
    0x003F_669E,
    0x005F_EA2D,
    0x0075_27BA,
    0x00C7_EBE5,
    0x00F1_7B3D,
    0x0007_39F7,
    0x008A_5292,
    0x00EA_6BFB,
    0x005F_B11F,
    0x008D_5D08,
    0x0056_0330,
    0x0046_FC7B,
    0x006B_ABF0,
    0x00CF_BC20,
    0x009A_F436,
    0x001D_A9E3,
    0x0091_615E,
    0x00E6_1B08,
    0x0065_9985,
    0x005F_14A0,
    0x0068_408D,
    0x00FF_D880,
    0x004D_7327,
    0x0031_0606,
    0x0015_56CA,
    0x0073_A8C9,
    0x0060_E27B,
    0x00C0_8C6B,
];

/// pi/2 split into 24-bit-mantissa doubles: PIo2 from `__kernel_rem_pio2`.
const PIO2S: [f64; 8] = [
    f64::from_bits(0x3FF9_21FB_4000_0000), // 1.57079625129699707031e+00
    f64::from_bits(0x3E74_442D_0000_0000), // 7.54978941586159635335e-08
    f64::from_bits(0x3CF8_4698_8000_0000), // 5.39030252995776476554e-15
    f64::from_bits(0x3B78_CC51_6000_0000), // 3.28200341580791294123e-22
    f64::from_bits(0x39F0_1B83_8000_0000), // 1.27065575308067607349e-29
    f64::from_bits(0x387A_2520_4000_0000), // 1.22933308981111328932e-36
    f64::from_bits(0x36E3_8222_8000_0000), // 2.73370053816464559624e-44
    f64::from_bits(0x3569_F31D_0000_0000), // 2.16741683877804819444e-51
];

// ---------------------------------------------------------------------------
// Local scalbn / floor helpers (no libm calls anywhere in this crate)
// ---------------------------------------------------------------------------

/// x * 2^n with the exact musl `scalbn` staging (exponent-field multiply,
/// pre-scaled in steps so overflow/underflow round exactly once).
fn scalbn(x: f64, mut n: i32) -> f64 {
    /// 2^1023.
    const HUGE: f64 = f64::from_bits(0x7FE0_0000_0000_0000);
    /// 2^-1022 * 2^53 = 2^-969 (keeps the final step out of double rounding).
    const TINY: f64 = f64::from_bits(54u64 << 52);
    let mut y = x;
    if n > 1023 {
        y *= HUGE;
        n -= 1023;
        if n > 1023 {
            y *= HUGE;
            n -= 1023;
            if n > 1023 {
                n = 1023;
            }
        }
    } else if n < -1022 {
        y *= TINY;
        n += 1022 - 53;
        if n < -1022 {
            y *= TINY;
            n += 1022 - 53;
            if n < -1022 {
                n = -1022;
            }
        }
    }
    // 2^n as a bit pattern; n is now in [-1022, 1023] so this is normal.
    #[allow(clippy::cast_sign_loss)]
    let pow2 = f64::from_bits(((0x3ff + n) as u64) << 52);
    y * pow2
}

/// floor for the non-negative, comfortably-in-range values the kernel
/// produces (v >= 0, v < 2^62): truncation toward zero == floor.
fn floor_pos(v: f64) -> f64 {
    (v as i64) as f64
}

// ---------------------------------------------------------------------------
// __kernel_rem_pio2, double-precision configuration
// ---------------------------------------------------------------------------

/// Payne–Hanek integer kernel: FDLIBM `__kernel_rem_pio2`, fixed to the
/// double-precision configuration (jk = 4 initial terms of 2/pi, prec = 1
/// two-double result).
///
/// `tx` holds |x| broken into 1..=3 non-negative 24-bit "digits"
/// (`tx[i] = floor(scale) at 2^(e0 - 24i)`), `e0` is the exponent of the
/// leading digit (`x = sum tx[i] * 2^(e0 - 24i)`), matching the caller's
/// `scalbn(|x|, ilogb(x) + 23)` split.
///
/// Returns `(n, y0, y1)` with `n` the last 3 bits of the integer quotient
/// nearest x/(pi/2) and `y0 + y1` the remainder, |y0| <= pi/4.
fn kernel_rem_pio2(tx: &[f64], e0: i32) -> (i32, f64, f64) {
    /// Initial number of 2/pi terms carried (init_jk[prec] with prec = 1).
    const JK: usize = 4;
    /// Number of PIo2 terms used in the back-multiplication (jp = jk).
    const JP: usize = JK;

    let nx = tx.len();
    let jx = nx - 1; // index of last input digit
    // jv: first 2/pi digit needed; q0: binary exponent of the last kept
    // digit ("scale" of iq[jz-1]). Note q0 < 3 by construction.
    let jv = ((e0 - 3) / 24).max(0);
    let mut q0 = e0 - 24 * (jv + 1);
    let jv = jv as usize;

    // Working arrays sized as in FDLIBM (jz never exceeds these bounds for
    // the 66-entry table / f64 inputs).
    let mut f = [0.0_f64; 20]; // ipio2 digits promoted to f64
    let mut q = [0.0_f64; 20]; // partial products x[j] * f[..]
    let mut iq = [0_i32; 20]; // 24-bit integer digits of the product
    let mut fq = [0.0_f64; 20]; // final digits times pi/2

    // Set up f[0..=jx+JK]: the 2/pi digits each x-digit multiplies against.
    for (i, slot) in f.iter_mut().enumerate().take(jx + JK + 1) {
        let j = jv as i64 + i as i64 - jx as i64;
        *slot = if j < 0 {
            0.0
        } else {
            f64::from(IPIO2[j as usize])
        };
    }

    // Compute q[0..=JK]: q[i] = sum over input digits of x[j]*f[jx+i-j].
    // Each term is an exact product of 24-bit values; the sum of <= 3 such
    // terms is exact in f64.
    for i in 0..=JK {
        let mut fw = 0.0;
        for j in 0..=jx {
            fw += tx[j] * f[jx + i - j];
        }
        q[i] = fw;
    }

    let mut jz = JK;
    let mut z;
    let mut n;
    let mut ih;
    // "recompute" loop from FDLIBM: distill, then if the fraction cancelled
    // to zero, pull in more 2/pi digits and redo.
    loop {
        // Distill q[] into 24-bit integer digits iq[], LSD first.
        z = q[jz];
        let mut i = 0usize;
        let mut j = jz;
        while j > 0 {
            let fw = floor_pos(TWOM24 * z);
            iq[i] = (z - TWO24 * fw) as i32;
            z = q[j - 1] + fw;
            i += 1;
            j -= 1;
        }

        // Compute n: the integer part of the product (mod 8).
        z = scalbn(z, q0); // actual value of the leading part
        z -= 8.0 * floor_pos(z * 0.125); // trim off integer >= 8
        n = z as i32;
        z -= f64::from(n);
        // ih > 0 iff the fraction q > 0.5, i.e. round n up and negate.
        ih = 0;
        if q0 > 0 {
            // Need bits of iq[jz-1] to complete n.
            let i2 = iq[jz - 1] >> (24 - q0);
            n += i2;
            iq[jz - 1] -= i2 << (24 - q0);
            ih = iq[jz - 1] >> (23 - q0);
        } else if q0 == 0 {
            ih = iq[jz - 1] >> 23;
        } else if z >= 0.5 {
            ih = 2;
        }

        if ih > 0 {
            // q > 0.5: use n+1 and the complement 1-q instead.
            n += 1;
            let mut carry = 0;
            for digit in iq.iter_mut().take(jz) {
                let j2 = *digit;
                if carry == 0 {
                    if j2 != 0 {
                        carry = 1;
                        *digit = 0x0100_0000 - j2;
                    }
                } else {
                    *digit = 0x00FF_FFFF - j2;
                }
            }
            if q0 == 1 {
                // Rare: clear the bit already folded into n.
                iq[jz - 1] &= 0x007F_FFFF;
            } else if q0 == 2 {
                iq[jz - 1] &= 0x003F_FFFF;
            }
            if ih == 2 {
                z = 1.0 - z;
                if carry != 0 {
                    z -= scalbn(1.0, q0);
                }
            }
        }

        // If everything cancelled to zero we cannot tell how close x is to
        // a multiple of pi/2 yet: fetch more digits and recompute.
        if z == 0.0 {
            let mut j2 = 0;
            for &digit in iq.iter().take(jz).skip(JK) {
                j2 |= digit;
            }
            if j2 == 0 {
                // k = number of additional terms needed.
                let mut k = 1usize;
                while iq[JK - k] == 0 {
                    k += 1; // guaranteed to stop: x is not a multiple of pi/2
                }
                for i2 in (jz + 1)..=(jz + k) {
                    f[jx + i2] = f64::from(IPIO2[jv + i2]);
                    let mut fw = 0.0;
                    for j3 in 0..=jx {
                        fw += tx[j3] * f[jx + i2 - j3];
                    }
                    q[i2] = fw;
                }
                jz += k;
                continue;
            }
        }
        break;
    }

    // Chop off trailing zero digits, or split z into 24-bit digits.
    if z == 0.0 {
        jz -= 1;
        q0 -= 24;
        while iq[jz] == 0 {
            jz -= 1;
            q0 -= 24;
        }
    } else {
        z = scalbn(z, -q0);
        if z >= TWO24 {
            let fw = floor_pos(TWOM24 * z);
            iq[jz] = (z - TWO24 * fw) as i32;
            jz += 1;
            q0 += 24;
            iq[jz] = fw as i32;
        } else {
            iq[jz] = z as i32;
        }
    }

    // Convert the integer digits back to floating chunks q[i] = iq[i]*2^..
    let mut fw = scalbn(1.0, q0);
    for i in (0..=jz).rev() {
        q[i] = fw * f64::from(iq[i]);
        fw *= TWOM24;
    }

    // Multiply by pi/2 (in 24-bit pieces) and accumulate into fq[],
    // smallest terms computed first.
    for i in (0..=jz).rev() {
        let mut acc = 0.0;
        let mut k = 0usize;
        while k <= JP && k <= jz - i {
            acc += PIO2S[k] * q[i + k];
            k += 1;
        }
        fq[jz - i] = acc;
    }

    // Compress fq[] into (y0, y1): prec = 1 branch of FDLIBM.
    let mut sum = 0.0;
    for &v in fq[..=jz].iter().rev() {
        sum += v;
    }
    let y0 = if ih == 0 { sum } else { -sum };
    let mut tail = fq[0] - sum;
    for &v in &fq[1..=jz] {
        tail += v;
    }
    let y1 = if ih == 0 { tail } else { -tail };

    (n & 7, y0, y1)
}

// ---------------------------------------------------------------------------
// __ieee754_rem_pio2
// ---------------------------------------------------------------------------

/// One Cody–Waite stage for the small |x| fast paths: y = x - k*(pi/2) via
/// the (pio2_1, pio2_1t) split. `k*PIO2_1` is exact for |k| <= 4 because
/// pio2_1 has 33 trailing zero bits, so one stage is good to ~85 bits.
/// Negative k reproduces FDLIBM's mirrored `x + pio2_1` branches exactly
/// (IEEE +,-,* commute with negation).
fn cody_waite_small(x: f64, k: f64) -> (f64, f64) {
    let z = x - k * PIO2_1;
    let y0 = z - k * PIO2_1T;
    let y1 = (z - y0) - k * PIO2_1T;
    (y0, y1)
}

/// Medium path (|x| < 2^20 * pi/2): FDLIBM's three-stage Cody–Waite with
/// cancellation detection via the exponent gap between x and the remainder.
fn medium(x: f64, ix: u32) -> (i32, f64, f64) {
    // rint(x * 2/pi) via the add-magic-subtract trick; Rust guarantees
    // round-to-nearest-even, which is what the trick requires.
    let f_n = x * INV_PIO2 + TOINT - TOINT;
    let mut n = f_n as i32;
    let mut f_n = f_n;
    let mut r = x - f_n * PIO2_1;
    let mut w = f_n * PIO2_1T; // 1st round, good to 85 bits
    // musl carries a guard for directed rounding modes; Rust always runs
    // round-to-nearest, but the guard is kept for exact structural parity
    // (it also pins |y0| <= pi/4 at the boundary).
    if r - w < -PIO4 {
        n -= 1;
        f_n -= 1.0;
        r = x - f_n * PIO2_1;
        w = f_n * PIO2_1T;
    } else if r - w > PIO4 {
        n += 1;
        f_n += 1.0;
        r = x - f_n * PIO2_1;
        w = f_n * PIO2_1T;
    }
    let mut y0 = r - w;
    let ex = (ix >> 20) as i32;
    let mut ey = ((hi(y0) >> 20) & 0x7FF) as i32;
    if ex - ey > 16 {
        // 2nd round, good to 118 bits.
        let t = r;
        w = f_n * PIO2_2;
        r = t - w;
        w = f_n * PIO2_2T - ((t - r) - w);
        y0 = r - w;
        ey = ((hi(y0) >> 20) & 0x7FF) as i32;
        if ex - ey > 49 {
            // 3rd round, good to 151 bits; covers all medium cases.
            let t2 = r;
            w = f_n * PIO2_3;
            r = t2 - w;
            w = f_n * PIO2_3T - ((t2 - r) - w);
            y0 = r - w;
        }
    }
    let y1 = (r - y0) - w;
    (n, y0, y1)
}

/// Reduce `x` to `(n, y0, y1)` with `x = N*(pi/2) + y0 + y1`,
/// `n = N mod 4` (the quadrant, always in `0..=3`), `|y0| <= pi/4`, and
/// `y1` the low part of the remainder (`y0 + y1` carries ~33 extra bits).
///
/// FDLIBM `__ieee754_rem_pio2` structure. `|x| <= pi/4` returns `(0, x, 0)`
/// untouched; NaN/inf return `(0, NaN, 0)`.
#[must_use]
pub(crate) fn rem_pio2(x: f64) -> (i32, f64, f64) {
    let hx = hi(x);
    let ix = hx & 0x7FFF_FFFF;
    let sign = hx >> 31 != 0;

    // Stage 1: |x| ~<= pi/4 — already reduced.
    if ix <= 0x3FE9_21FB {
        return (0, x, 0.0);
    }

    // Stage 2: |x| ~<= 5pi/4 — subtract 1 or 2 multiples of pi/2 directly.
    if ix <= 0x400F_6A7A {
        if (ix & 0x000F_FFFF) == 0x0009_21FB {
            // |x| ~= pi/2 or pi: heavy cancellation against pio2_1 — take
            // the medium path, whose extra stages absorb it.
            let (n, y0, y1) = medium(x, ix);
            return (n & 3, y0, y1);
        }
        let k = if ix <= 0x4002_D97C { 1 } else { 2 }; // |x| ~<= 3pi/4 ?
        let k = if sign { -k } else { k };
        let (y0, y1) = cody_waite_small(x, f64::from(k));
        return (k & 3, y0, y1);
    }

    // Stage 2 continued: |x| ~<= 9pi/4 — 3 or 4 multiples of pi/2.
    if ix <= 0x401C_463B {
        if ix == 0x4012_D97C || ix == 0x4019_21FB {
            // |x| ~= 3pi/2 or 2pi: cancellation — medium path again.
            let (n, y0, y1) = medium(x, ix);
            return (n & 3, y0, y1);
        }
        let k = if ix <= 0x4015_FDBC { 3 } else { 4 }; // |x| ~<= 7pi/4 ?
        let k = if sign { -k } else { k };
        let (y0, y1) = cody_waite_small(x, f64::from(k));
        return (k & 3, y0, y1);
    }

    // Stage 3: |x| < 2^20 * pi/2 — medium multi-stage Cody–Waite.
    if ix < 0x4139_21FB {
        let (n, y0, y1) = medium(x, ix);
        return (n & 3, y0, y1);
    }

    // NaN / inf: quadrant 0, NaN remainder. FDLIBM's `x - x` idiom turns
    // inf into NaN and propagates NaN payloads.
    if ix >= 0x7FF0_0000 {
        #[allow(clippy::eq_op)]
        let y = x - x;
        return (0, y, 0.0);
    }

    // Stage 4: Payne–Hanek. Set z = scalbn(|x|, -ilogb(x) + 23) by forcing
    // the exponent field to 2^23, then peel three 24-bit digits.
    let z_bits = (x.to_bits() & (u64::MAX >> 12)) | ((0x3FF + 23u64) << 52);
    let mut z = f64::from_bits(z_bits);
    let mut tx = [0.0_f64; 3];
    for digit in tx.iter_mut().take(2) {
        *digit = f64::from(z as i32);
        z = (z - *digit) * TWO24;
    }
    tx[2] = z;
    // Skip trailing zero digits (the leading digit is never zero).
    let mut nx = 3;
    while nx > 1 && tx[nx - 1] == 0.0 {
        nx -= 1;
    }
    let e0 = ((ix >> 20) as i32) - (0x3FF + 23);
    let (n, y0, y1) = kernel_rem_pio2(&tx[..nx], e0);
    if sign {
        ((-n) & 3, -y0, -y1)
    } else {
        (n & 3, y0, y1)
    }
}

#[cfg(test)]
mod tests {
    use super::rem_pio2;

    const PI_2: f64 = std::f64::consts::FRAC_PI_2;
    const PI_4: f64 = std::f64::consts::FRAC_PI_4;

    /// Reconstruct sin(x) from the reduction using std as the oracle:
    /// sin(x) = sin(y0+y1 + n*pi/2).
    fn sin_via_reduction(x: f64) -> f64 {
        let (n, y0, y1) = rem_pio2(x);
        let y = y0 + y1;
        match n {
            0 => y.sin(),
            1 => y.cos(),
            2 => -y.sin(),
            _ => -y.cos(),
        }
    }

    #[test]
    fn small_args_pass_through() {
        for x in [0.0, -0.0, 0.5, -0.5, PI_4, -PI_4, 1e-300, -1e-300] {
            let (n, y0, y1) = rem_pio2(x);
            assert_eq!(n, 0);
            assert_eq!(y0.to_bits(), x.to_bits());
            assert_eq!(y1.to_bits(), 0.0_f64.to_bits());
        }
    }

    #[test]
    fn quadrant_and_remainder_match_direct_computation() {
        // Sweep the Cody-Waite and medium ranges. For each point, the
        // nearest-integer quotient and the remainder must agree with a
        // direct (test-only, std-based) computation.
        for i in 0..20_000 {
            let x = -1000.0 + f64::from(i) * 0.1 + 0.05; // avoid exact 0
            let (n, y0, y1) = rem_pio2(x);
            assert!((0..=3).contains(&n), "quadrant out of range for {x}");
            // |y0| <= pi/4 (+1 ulp of slack at the boundary).
            assert!(
                y0.abs() <= PI_4 * (1.0 + 1e-15),
                "y0 = {y0} too big for {x}"
            );
            assert!(y1.abs() < 1e-15, "tail not small: {y1} for {x}");
            let q = (x / PI_2).round();
            let expected_n = (q as i64).rem_euclid(4) as i32;
            assert_eq!(n, expected_n, "quadrant mismatch at {x}");
            // Remainder check in extended precision via the split constant.
            let rem = (x - q * PI_2) - q * 6.123_233_995_736_766e-17;
            assert!(
                (y0 + y1 - rem).abs() < 1e-9,
                "remainder mismatch at {x}: {} vs {rem}",
                y0 + y1
            );
        }
    }

    #[test]
    fn near_multiples_of_pi_over_2_stay_accurate() {
        // The cancellation-prone points that force the medium/3-stage path.
        for k in 1..=100_i32 {
            let x = f64::from(k) * PI_2;
            let recon = sin_via_reduction(x);
            let expect = x.sin();
            assert!(
                (recon - expect).abs() < 1e-16,
                "near k*pi/2, k={k}: {recon} vs {expect}"
            );
        }
    }

    #[test]
    fn large_args_agree_with_std_reduction() {
        // Payne-Hanek path vs glibc (itself doing correct reduction).
        let big = [
            1e10,
            1e16,
            1e22,
            f64::from_bits((1000 + 1023_u64) << 52), // 2^1000
            1e300,
            -1e22,
            -1e10,
        ];
        for &x in &big {
            let recon = sin_via_reduction(x);
            let expect = x.sin();
            let diff = (recon - expect).abs();
            assert!(
                diff <= 4.0 * expect.abs().max(f64::MIN_POSITIVE) * f64::EPSILON,
                "large-arg mismatch at {x}: {recon} vs {expect}"
            );
        }
    }

    #[test]
    fn negation_mirrors_exactly() {
        for i in 1..10_000 {
            let x = f64::from(i) * 0.37 + 1.0;
            let (n_p, y0_p, y1_p) = rem_pio2(x);
            let (n_m, y0_m, y1_m) = rem_pio2(-x);
            assert_eq!(y0_m.to_bits(), (-y0_p).to_bits(), "y0 not mirrored at {x}");
            assert_eq!(y1_m.to_bits(), (-y1_p).to_bits(), "y1 not mirrored at {x}");
            assert_eq!(n_m, (-n_p) & 3, "quadrant not mirrored at {x}");
        }
    }

    #[test]
    fn non_finite_inputs() {
        let (n, y0, y1) = rem_pio2(f64::NAN);
        assert_eq!(n, 0);
        assert!(y0.is_nan());
        assert_eq!(y1.to_bits(), 0.0_f64.to_bits());
        for x in [f64::INFINITY, f64::NEG_INFINITY] {
            let (n, y0, y1) = rem_pio2(x);
            assert_eq!(n, 0);
            assert!(y0.is_nan());
            assert_eq!(y1.to_bits(), 0.0_f64.to_bits());
        }
    }
}
