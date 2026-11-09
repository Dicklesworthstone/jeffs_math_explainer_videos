//! The tolerance doctrine as reusable assertion helpers (§16.4, fm-xb3).
//!
//! Four comparison regimes, each with a precise definition and an explicit
//! NaN/−0 stance, so no test invents its own epsilon:
//!
//! 1. **Bit equality** — self-goldens. `to_bits` equality; distinguishes
//!    `−0.0` from `+0.0` and every NaN payload, because a golden locks the
//!    exact bytes an engine produced.
//! 2. **ULP-scaled** — owned f64 mathematics (fmn-dmath, fmn-geom oracles).
//!    Distance in units-in-the-last-place over the monotone integer mapping
//!    of IEEE-754 doubles; `+0.0` and `−0.0` are 1 ULP apart (adjacent under
//!    the mapping), and any NaN on either side is a defined outcome governed
//!    by [`NanPolicy`], never an accidental `false`.
//! 3. **Loose absolute f32** — cross-engine structural fixtures against the
//!    Reference (we compute in f64 over f32 records; the Reference computes
//!    in f64 over f32 records with different op ordering). Plain absolute
//!    finite, non-negative tolerance on values of frame-coordinate
//!    magnitude.
//! 4. **Explicit NaN/−0 handling** — [`NanPolicy`] is a required parameter
//!    wherever NaN can occur, and the −0 behavior of every regime is stated
//!    in its doc comment. Nothing here treats NaN or signed zero implicitly.
//!
//! All helpers return `Result<(), Mismatch>` rather than panicking: this
//! crate's library surface stays panic-free, and test code decides how to
//! fail (typically `.unwrap()` on the `Result`, whose `Display` carries the
//! full diagnostic).

use std::fmt;

/// How a comparison treats NaN operands.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NanPolicy {
    /// Two NaNs (any payloads) compare equal; NaN vs non-NaN is a mismatch.
    /// The usual choice for oracle tables that legitimately contain NaN rows.
    EqualNans,
    /// Any NaN on either side is a mismatch. The usual choice for geometry,
    /// where a NaN is always a bug.
    Reject,
}

/// A failed comparison. `Display` renders the full diagnostic: the element
/// index, both values (decimal and bit pattern), and the regime's verdict.
#[derive(Clone, PartialEq, Debug)]
pub enum Mismatch {
    /// The slices under comparison have different lengths; no elements were
    /// compared.
    Length {
        /// Expected element count.
        expected: usize,
        /// Actual element count.
        actual: usize,
    },
    /// One element pair failed its regime.
    Element {
        /// Flat element index (for point slices: `point_index * 3 + axis`).
        index: usize,
        /// The expected value, widened to f64 for reporting.
        expected: f64,
        /// The actual value, widened to f64 for reporting.
        actual: f64,
        /// The expected value's original bit pattern (f32 patterns are
        /// reported zero-extended).
        expected_bits: u64,
        /// The actual value's original bit pattern.
        actual_bits: u64,
        /// Which regime failed and by how much.
        verdict: Verdict,
    },
}

/// The per-regime failure detail inside [`Mismatch::Element`].
#[derive(Clone, PartialEq, Debug)]
pub enum Verdict {
    /// Bit-equality regime: the patterns differ.
    BitsDiffer,
    /// ULP regime: the measured distance exceeded the budget, or was
    /// unmeasurable (opposite signs / NaN under [`NanPolicy::Reject`]).
    UlpExceeded {
        /// Measured distance, if the pair was measurable.
        distance: Option<u64>,
        /// The permitted budget.
        max_ulps: u64,
    },
    /// Absolute regime: the requested tolerance was negative or non-finite,
    /// so it cannot define a comparison budget.
    InvalidTolerance {
        /// The rejected tolerance.
        tol: f64,
    },
    /// Absolute regime: `|expected − actual|` exceeded the tolerance (or was
    /// NaN under [`NanPolicy::Reject`], or infinities disagreed).
    AbsExceeded {
        /// The measured absolute error (`NaN` when unmeasurable).
        error: f64,
        /// The permitted tolerance.
        tol: f64,
    },
}

impl fmt::Display for Mismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Length { expected, actual } => {
                write!(
                    f,
                    "length mismatch: expected {expected} elements, got {actual}"
                )
            }
            Self::Element {
                index,
                expected,
                actual,
                expected_bits,
                actual_bits,
                verdict,
            } => {
                write!(
                    f,
                    "element {index}: expected {expected:e} (bits {expected_bits:#018x}), \
                     got {actual:e} (bits {actual_bits:#018x}): "
                )?;
                match verdict {
                    Verdict::BitsDiffer => write!(f, "bit patterns differ"),
                    Verdict::UlpExceeded { distance, max_ulps } => match distance {
                        Some(d) => write!(f, "{d} ULPs apart, budget {max_ulps}"),
                        None => write!(f, "not ULP-comparable (sign/NaN), budget {max_ulps}"),
                    },
                    Verdict::InvalidTolerance { tol } => write!(
                        f,
                        "invalid absolute tolerance {tol:e}; expected a finite non-negative value"
                    ),
                    Verdict::AbsExceeded { error, tol } => {
                        write!(f, "absolute error {error:e} exceeds tolerance {tol:e}")
                    }
                }
            }
        }
    }
}

impl std::error::Error for Mismatch {}

/// Map an f64 to the monotone signed-magnitude integer line: negative values
/// map below positive ones, ordering matches `<` on non-NaN doubles, and
/// adjacent representable doubles map to adjacent integers.
fn monotone_bits(x: f64) -> u64 {
    let b = x.to_bits();
    if b & (1 << 63) != 0 {
        !b
    } else {
        b | (1 << 63)
    }
}

/// The ULP distance between two doubles under the monotone mapping, or `None`
/// if either is NaN. `+0.0` and `−0.0` are adjacent (distance 1); values of
/// opposite sign measure across zero (so the distance is finite but huge,
/// which any sane budget rejects).
#[must_use]
pub fn ulp_distance(a: f64, b: f64) -> Option<u64> {
    if a.is_nan() || b.is_nan() {
        return None;
    }
    Some(monotone_bits(a).abs_diff(monotone_bits(b)))
}

/// Bit-equality on one f64 pair (regime 1). Distinguishes NaN payloads and
/// signed zeros by construction.
///
/// # Errors
/// [`Mismatch::Element`] when the bit patterns differ.
pub fn check_bits_f64(expected: f64, actual: f64) -> Result<(), Mismatch> {
    if expected.to_bits() == actual.to_bits() {
        Ok(())
    } else {
        Err(element(0, expected, actual, Verdict::BitsDiffer))
    }
}

/// Bit-equality across two f64 slices (regime 1).
///
/// # Errors
/// [`Mismatch::Length`] on length disagreement, else the first differing
/// element as [`Mismatch::Element`].
pub fn check_slice_bits_f64(expected: &[f64], actual: &[f64]) -> Result<(), Mismatch> {
    check_len(expected.len(), actual.len())?;
    for (i, (&e, &a)) in expected.iter().zip(actual).enumerate() {
        if e.to_bits() != a.to_bits() {
            return Err(element(i, e, a, Verdict::BitsDiffer));
        }
    }
    Ok(())
}

/// ULP-scaled comparison on one f64 pair (regime 2).
///
/// # Errors
/// [`Mismatch::Element`] when the distance exceeds `max_ulps`, or when a NaN
/// appears under [`NanPolicy::Reject`] (under [`NanPolicy::EqualNans`], two
/// NaNs pass and a single NaN fails).
pub fn check_ulp_f64(
    expected: f64,
    actual: f64,
    max_ulps: u64,
    nan: NanPolicy,
) -> Result<(), Mismatch> {
    match ulp_distance(expected, actual) {
        Some(d) if d <= max_ulps => Ok(()),
        Some(d) => Err(element(
            0,
            expected,
            actual,
            Verdict::UlpExceeded {
                distance: Some(d),
                max_ulps,
            },
        )),
        None => {
            if nan == NanPolicy::EqualNans && expected.is_nan() && actual.is_nan() {
                Ok(())
            } else {
                Err(element(
                    0,
                    expected,
                    actual,
                    Verdict::UlpExceeded {
                        distance: None,
                        max_ulps,
                    },
                ))
            }
        }
    }
}

/// ULP-scaled comparison across two f64 slices (regime 2).
///
/// # Errors
/// [`Mismatch::Length`] on length disagreement, else the first element
/// exceeding the budget as [`Mismatch::Element`].
pub fn check_slice_ulp_f64(
    expected: &[f64],
    actual: &[f64],
    max_ulps: u64,
    nan: NanPolicy,
) -> Result<(), Mismatch> {
    check_len(expected.len(), actual.len())?;
    for (i, (&e, &a)) in expected.iter().zip(actual).enumerate() {
        check_ulp_f64(e, a, max_ulps, nan).map_err(|m| reindex(m, i))?;
    }
    Ok(())
}

/// Loose absolute comparison on one value pair (regime 3): passes when
/// `|expected − actual| <= tol`, with equal infinities passing and signed
/// zeros compared by value (`−0.0 == +0.0` here — cross-engine fixtures do
/// not promise zero signs).
///
/// # Errors
/// [`Mismatch::Element`] when `tol` is negative or non-finite, the error
/// exceeds `tol`, infinities disagree, or a NaN appears under
/// [`NanPolicy::Reject`].
pub fn check_abs(expected: f64, actual: f64, tol: f64, nan: NanPolicy) -> Result<(), Mismatch> {
    if !tol.is_finite() || tol < 0.0 {
        return Err(element(
            0,
            expected,
            actual,
            Verdict::InvalidTolerance { tol },
        ));
    }
    let nan_case = expected.is_nan() || actual.is_nan();
    if nan_case {
        if nan == NanPolicy::EqualNans && expected.is_nan() && actual.is_nan() {
            return Ok(());
        }
        return Err(element(
            0,
            expected,
            actual,
            Verdict::AbsExceeded {
                error: f64::NAN,
                tol,
            },
        ));
    }
    // Equal infinities subtract to NaN; treat exact equality (covers them and
    // exact finite hits) as a pass before measuring.
    if expected == actual {
        return Ok(());
    }
    let error = (expected - actual).abs();
    if error <= tol {
        Ok(())
    } else {
        Err(element(
            0,
            expected,
            actual,
            Verdict::AbsExceeded { error, tol },
        ))
    }
}

/// Loose absolute comparison across two f64 slices (regime 3).
///
/// # Errors
/// [`Mismatch::Length`] on length disagreement, else the first element
/// exceeding the tolerance as [`Mismatch::Element`].
pub fn check_slice_abs(
    expected: &[f64],
    actual: &[f64],
    tol: f64,
    nan: NanPolicy,
) -> Result<(), Mismatch> {
    check_len(expected.len(), actual.len())?;
    for (i, (&e, &a)) in expected.iter().zip(actual).enumerate() {
        check_abs(e, a, tol, nan).map_err(|m| reindex(m, i))?;
    }
    Ok(())
}

/// Loose absolute comparison across two point slices (regime 3), the shape
/// cross-engine structural fixtures use. The reported flat index is
/// `point_index * 3 + axis`.
///
/// # Errors
/// [`Mismatch::Length`] on point-count disagreement, else the first failing
/// coordinate as [`Mismatch::Element`].
pub fn check_points_abs(
    expected: &[[f64; 3]],
    actual: &[[f64; 3]],
    tol: f64,
    nan: NanPolicy,
) -> Result<(), Mismatch> {
    check_len(expected.len(), actual.len())?;
    for (i, (e, a)) in expected.iter().zip(actual).enumerate() {
        for axis in 0..3 {
            check_abs(e[axis], a[axis], tol, nan).map_err(|m| reindex(m, i * 3 + axis))?;
        }
    }
    Ok(())
}

fn check_len(expected: usize, actual: usize) -> Result<(), Mismatch> {
    if expected == actual {
        Ok(())
    } else {
        Err(Mismatch::Length { expected, actual })
    }
}

fn element(index: usize, expected: f64, actual: f64, verdict: Verdict) -> Mismatch {
    Mismatch::Element {
        index,
        expected,
        actual,
        expected_bits: expected.to_bits(),
        actual_bits: actual.to_bits(),
        verdict,
    }
}

fn reindex(m: Mismatch, index: usize) -> Mismatch {
    match m {
        Mismatch::Element {
            expected,
            actual,
            expected_bits,
            actual_bits,
            verdict,
            ..
        } => Mismatch::Element {
            index,
            expected,
            actual,
            expected_bits,
            actual_bits,
            verdict,
        },
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bits_distinguish_signed_zero_and_nan_payloads() {
        assert!(check_bits_f64(0.0, 0.0).is_ok());
        assert!(check_bits_f64(0.0, -0.0).is_err());
        let quiet = f64::from_bits(0x7ff8_0000_0000_0000);
        let payload = f64::from_bits(0x7ff8_0000_0000_0001);
        assert!(check_bits_f64(quiet, quiet).is_ok());
        assert!(check_bits_f64(quiet, payload).is_err());
    }

    #[test]
    fn ulp_distance_is_monotone_and_zero_aware() {
        assert_eq!(ulp_distance(1.0, 1.0), Some(0));
        let next = f64::from_bits(1.0f64.to_bits() + 1);
        assert_eq!(ulp_distance(1.0, next), Some(1));
        assert_eq!(ulp_distance(0.0, -0.0), Some(1));
        assert_eq!(ulp_distance(f64::NAN, 1.0), None);
        // Across zero is finite but enormous.
        assert!(ulp_distance(-1.0, 1.0).unwrap() > u64::from(u32::MAX));
    }

    #[test]
    fn ulp_regime_honors_nan_policy() {
        assert!(check_ulp_f64(f64::NAN, f64::NAN, 0, NanPolicy::EqualNans).is_ok());
        assert!(check_ulp_f64(f64::NAN, f64::NAN, 0, NanPolicy::Reject).is_err());
        assert!(check_ulp_f64(f64::NAN, 1.0, u64::MAX, NanPolicy::EqualNans).is_err());
    }

    #[test]
    fn abs_regime_covers_infinities_and_zero_sign() {
        assert!(check_abs(f64::INFINITY, f64::INFINITY, 0.0, NanPolicy::Reject).is_ok());
        assert!(check_abs(f64::INFINITY, f64::NEG_INFINITY, 1e300, NanPolicy::Reject).is_err());
        // Cross-engine regime: zero signs are not promised.
        assert!(check_abs(0.0, -0.0, 0.0, NanPolicy::Reject).is_ok());
        assert!(check_abs(1.0, 1.0009, 1e-3, NanPolicy::Reject).is_ok());
        assert!(check_abs(1.0, 1.002, 1e-3, NanPolicy::Reject).is_err());
    }

    #[test]
    fn abs_regime_rejects_invalid_budgets_before_comparing_values() {
        for tol in [-1.0, f64::NEG_INFINITY, f64::INFINITY, f64::NAN] {
            let error = check_abs(1.0, 1.0, tol, NanPolicy::Reject)
                .expect_err("an invalid tolerance must not pass an exact match");
            assert!(
                matches!(
                    error,
                    Mismatch::Element {
                        verdict: Verdict::InvalidTolerance { .. },
                        ..
                    }
                ),
                "wrong invalid-tolerance verdict: {error}"
            );
        }

        assert!(
            check_abs(
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::INFINITY,
                NanPolicy::Reject
            )
            .is_err(),
            "an infinite budget must not admit disagreeing infinities"
        );
        assert!(check_slice_abs(&[1.0], &[1.0], f64::NAN, NanPolicy::Reject).is_err());
        assert!(check_points_abs(&[[0.0; 3]], &[[0.0; 3]], -1.0, NanPolicy::Reject).is_err());
    }

    #[test]
    fn slice_helpers_report_flat_indices() {
        let e = [[0.0, 1.0, 2.0], [3.0, 4.0, 5.0]];
        let mut a = e;
        a[1][2] = 5.01;
        let m = check_points_abs(&e, &a, 1e-3, NanPolicy::Reject).unwrap_err();
        assert!(matches!(m, Mismatch::Element { index: 5, .. }));
    }

    #[test]
    fn length_mismatch_reported_before_elements() {
        let m = check_slice_bits_f64(&[1.0], &[1.0, 2.0]).unwrap_err();
        assert_eq!(
            m,
            Mismatch::Length {
                expected: 1,
                actual: 2
            }
        );
    }
}
