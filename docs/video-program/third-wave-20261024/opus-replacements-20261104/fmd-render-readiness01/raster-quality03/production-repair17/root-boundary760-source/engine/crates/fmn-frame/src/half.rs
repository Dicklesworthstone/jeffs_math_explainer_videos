//! IEEE 754 binary16 ("half") conversion, in safe integer arithmetic.
//!
//! RGBA16F intermediates (§14.1) store channel samples as raw binary16
//! bits in little-endian `u16`s. This module owns the bit-exact
//! conversions: decode is exact (every binary16 value is exactly
//! representable in f32/f64), encode is round-to-nearest-even. No
//! platform float intrinsics are involved — the same bits come out on
//! every certified platform.

/// 2⁻²⁴ as an exact f32 constant (the binary16 subnormal quantum).
const F16_SUBNORMAL_QUANTUM: f32 = f32::from_bits(0x3380_0000);

/// Decode binary16 bits to the exactly-equal f32.
#[must_use]
pub fn f16_to_f32(bits: u16) -> f32 {
    let sign = u32::from(bits >> 15) << 31;
    let exp = (bits >> 10) & 0x1f;
    let frac = u32::from(bits & 0x3ff);
    match exp {
        0 => {
            // Zero or subnormal: magnitude = frac × 2⁻²⁴ (exact in f32).
            let mag = frac as f32 * F16_SUBNORMAL_QUANTUM;
            if sign != 0 { -mag } else { mag }
        }
        0x1f => f32::from_bits(sign | 0x7f80_0000 | (frac << 13)),
        _ => f32::from_bits(sign | ((u32::from(exp) + 112) << 23) | (frac << 13)),
    }
}

/// Decode binary16 bits to the exactly-equal f64.
#[must_use]
pub fn f16_to_f64(bits: u16) -> f64 {
    f64::from(f16_to_f32(bits))
}

/// Shift `mant` right by `shift` bits, rounding to nearest, ties to even.
fn round_rne(mant: u32, shift: u32) -> u32 {
    let half = 1u32 << (shift - 1);
    let rem = mant & ((1u32 << shift) - 1);
    let truncated = mant >> shift;
    if rem > half || (rem == half && truncated & 1 == 1) {
        truncated + 1
    } else {
        truncated
    }
}

/// Encode an f32 as binary16 bits, rounding to nearest, ties to even.
///
/// Overflow saturates to the like-signed infinity; NaN stays NaN (quiet
/// bit forced, top payload bits kept); underflow flushes to the
/// like-signed zero exactly where IEEE rounding says it must.
#[must_use]
pub fn f16_from_f32(x: f32) -> u16 {
    let bits = x.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let abs = bits & 0x7fff_ffff;

    if abs >= 0x7f80_0000 {
        // Infinity or NaN.
        let mantissa = if abs > 0x7f80_0000 {
            (((abs >> 13) & 0x3ff) as u16) | 0x200
        } else {
            0
        };
        return sign | 0x7c00 | mantissa;
    }
    if abs >= 0x477f_f000 {
        // |x| ≥ 65520 rounds past the largest finite half (65504).
        return sign | 0x7c00;
    }

    let exp = (abs >> 23) as i32;
    let mant = (abs & 0x007f_ffff) | 0x0080_0000; // 24-bit significand
    if exp >= 113 {
        // Normal half. `rounded` carries the implicit bit at 0x400; a
        // mantissa overflow into 0x800 lands as +1 on the exponent field
        // through plain addition, which is exactly the rounding rule.
        let rounded = round_rne(mant, 13);
        let half_exp = (exp - 112) as u32;
        sign | ((half_exp << 10) + (rounded - 0x400)) as u16
    } else if exp >= 102 {
        // Subnormal half (a result of exactly 0x400 has rounded up into
        // the smallest normal, and the bit pattern is again correct).
        let shift = (126 - exp) as u32;
        sign | round_rne(mant, shift) as u16
    } else {
        sign
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_values() {
        assert_eq!(f16_from_f32(0.0), 0x0000);
        assert_eq!(f16_from_f32(-0.0), 0x8000);
        assert_eq!(f16_from_f32(1.0), 0x3c00);
        assert_eq!(f16_from_f32(-2.0), 0xc000);
        assert_eq!(f16_from_f32(0.5), 0x3800);
        assert_eq!(f16_from_f32(65504.0), 0x7bff);
        assert_eq!(f16_from_f32(65520.0), 0x7c00); // rounds to +inf
        assert_eq!(f16_from_f32(f32::INFINITY), 0x7c00);
        assert_eq!(f16_from_f32(F16_SUBNORMAL_QUANTUM), 0x0001);
        // 0.1 → 1.6 × 2⁻⁴, mantissa 614.4 rounds to 614 = 0x266.
        assert_eq!(f16_from_f32(0.1), 0x2e66);
        assert!(f16_to_f32(f16_from_f32(f32::NAN)).is_nan());
    }

    #[test]
    fn decode_encode_is_identity_on_all_bit_patterns() {
        for bits in 0..=u16::MAX {
            let x = f16_to_f32(bits);
            if x.is_nan() {
                let back = f16_from_f32(x);
                assert!(back & 0x7c00 == 0x7c00 && back & 0x3ff != 0);
            } else {
                assert_eq!(f16_from_f32(x), bits, "bits {bits:#06x}");
            }
        }
    }

    #[test]
    fn rne_ties_go_to_even() {
        // 2⁻²⁵ is exactly halfway between 0 and the smallest subnormal.
        let tie = f32::from_bits(0x3300_0000);
        assert_eq!(f16_from_f32(tie), 0x0000);
        // Just above the tie rounds up.
        assert_eq!(f16_from_f32(f32::from_bits(0x3300_0001)), 0x0001);
    }
}
