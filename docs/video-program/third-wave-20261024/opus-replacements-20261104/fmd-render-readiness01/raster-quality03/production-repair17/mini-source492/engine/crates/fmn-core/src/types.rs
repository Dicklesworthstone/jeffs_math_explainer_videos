//! Numeric doctrine hooks (§6.1): the f64-semantic / f32-record split and
//! canonicalization at serialization boundaries.

/// Semantic scalar: geometry, time, alphas — engine mathematics is f64.
pub type Semantic = f64;

/// Record scalar: the Reference's f32 record dtypes, kept as API surface
/// (RecordBuffer views hand these to NumPy unchanged).
pub type Record = f32;

/// A point or direction in scene space (§6.2 coordinate conventions:
/// y up, z out of the screen, FRAME_HEIGHT scene units tall).
pub type Vec3 = [Semantic; 3];

/// The single canonical quiet-NaN bit pattern used wherever a NaN must be
/// serialized or hashed.
pub const CANONICAL_NAN_F64: u64 = 0x7ff8_0000_0000_0000;

/// The f32 counterpart of [`CANONICAL_NAN_F64`].
pub const CANONICAL_NAN_F32: u32 = 0x7fc0_0000;

/// Canonicalize a semantic scalar at a serialization/hash boundary:
/// `-0.0` becomes `+0.0` and every NaN becomes the one canonical quiet NaN,
/// so equal values hash equally on every platform.
#[must_use]
pub fn canonicalize_f64(x: f64) -> f64 {
    if x == 0.0 {
        0.0
    } else if x.is_nan() {
        f64::from_bits(CANONICAL_NAN_F64)
    } else {
        x
    }
}

/// [`canonicalize_f64`] for record scalars.
#[must_use]
pub fn canonicalize_f32(x: f32) -> f32 {
    if x == 0.0 {
        0.0
    } else if x.is_nan() {
        f32::from_bits(CANONICAL_NAN_F32)
    } else {
        x
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalize_folds_negative_zero_and_nans() {
        assert_eq!(canonicalize_f64(-0.0).to_bits(), 0.0_f64.to_bits());
        assert_eq!(canonicalize_f64(f64::NAN).to_bits(), CANONICAL_NAN_F64);
        assert_eq!(
            canonicalize_f64(f64::from_bits(0x7ff8_dead_beef_0001)).to_bits(),
            CANONICAL_NAN_F64
        );
        assert_eq!(canonicalize_f64(1.5), 1.5);
        assert_eq!(canonicalize_f32(-0.0).to_bits(), 0.0_f32.to_bits());
        assert_eq!(canonicalize_f32(f32::NAN).to_bits(), CANONICAL_NAN_F32);
    }
}
