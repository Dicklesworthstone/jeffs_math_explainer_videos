//! The engine-equivalence suite (§16.3 plane 4, fm-t1v W10): certified CPU
//! versus fast CPU under the **versioned v1 visual budget**, wired as the
//! engine-change blocker.
//!
//! §10.1 and §16.3: the fast engine shares Lumen's semantics but not its
//! arithmetic width. Every fast route therefore answers to the certified
//! CPU engine's pixels, and the answer is numeric, not rhetorical.
//!
//! ## Budget v1 (this is the versioned contract — bump it deliberately)
//!
//! | Metric | Bound | Source of truth |
//! |---|---|---|
//! | max linear channel error | ≤ `FAST_VISUAL_BUDGET_V1_MAX_CHANNEL_ERROR` (`0.092`) | `fmn_render::engine` |
//! | RMS linear channel error | ≤ `FAST_VISUAL_BUDGET_V1_RMS_CHANNEL_ERROR` (`0.00042`) | `fmn_render::engine` |
//! | global sRGB-luma SSIM | ≥ `FAST_VISUAL_BUDGET_V1_MIN_SSIM` (`0.99999`) | `fmn_render::engine` |
//!
//! The constants live in `fmn-render` beside the engine they gate. Max and
//! RMS are the local-error tripwires over the raw `Rgba16F` linear-light
//! frame; the global SSIM over the canonical sRGB8 Rec. 709 luma plane is
//! the perceptual smoke alarm (the spike's ruling, kept: a stable global
//! form rather than an unreviewed windowed metric).
//!
//! ## Where the pieces live
//!
//! This module is the **budget contract**: the measured [`Divergence`]
//! record and the [`budget_v1_failures`] verdict. The pixel walk that
//! produces a `Divergence` — per-channel linear error over the raw frame
//! plus the luma SSIM — lives in `tests/engine_equivalence.rs`, because
//! `fmn-frame` is a documented dev-only edge of this crate (see
//! Cargo.toml): the conformance library holds the versioned semantics, the
//! test target holds the frame access. The formula is the one
//! `tests/certified_engine.rs` established for its own corpus.
//!
//! ## Blocking semantics
//!
//! [`budget_v1_failures`] returns a human-legible message per violated
//! bound; the test asserts the list is empty. **A budget violation fails
//! the suite — this is the merge blocker for engine arithmetic changes.**
//! Non-finite values and values outside a metric's mathematical domain are
//! failures before threshold comparison, so invalid arithmetic cannot make
//! every floating-point comparison false and pass the blocker.
//! Loosening a bound means editing the `fmn-render` constant with an
//! adjudicated re-measurement in its doc comment, never editing a scene to
//! fit. Thread count is not part of an engine identity (C10): the suite
//! renders the fast route multithreaded, so scheduling cannot quietly spend
//! budget the scalar-threaded reviewer run would not see.
//!
//! The scene set is [`crate::scene_goldens::EQUIVALENCE_SUBSET`], rendered
//! in the test target through the same plan/mono/binning derivation shape
//! the self-goldens use, so the equivalence lane and the bit-locked lane
//! cannot silently diverge in plan construction.

use fmn_render::engine::{
    FAST_VISUAL_BUDGET_V1_MAX_CHANNEL_ERROR, FAST_VISUAL_BUDGET_V1_MIN_SSIM,
    FAST_VISUAL_BUDGET_V1_RMS_CHANNEL_ERROR, Tier,
};

/// The measured divergence between a candidate frame and the certified
/// reference, under budget v1's three metrics. Pure data: the pixel walk
/// that fills it lives in the test target (see the module docs).
#[derive(Debug, Clone, Copy)]
pub struct Divergence {
    /// Maximum absolute linear channel error over the frame.
    pub maximum: f64,
    /// `(x, y, channel)` of the maximum error.
    pub maximum_at: (u32, u32, usize),
    /// The reference frame's channel value at the maximum.
    pub reference_at_maximum: f64,
    /// The candidate frame's channel value at the maximum.
    pub candidate_at_maximum: f64,
    /// Root-mean-square linear channel error over the frame.
    pub rms: f64,
    /// Global sRGB-luma SSIM (1.0 is identical).
    pub ssim: f64,
}

/// The budget-v1 verdict for one measured divergence: one human-legible
/// message per violated bound or invalid metric domain, empty when the
/// candidate passes. The test fails on any message — this is the
/// engine-change blocker.
#[must_use]
pub fn budget_v1_failures(scene: &str, tier: Tier, measured: &Divergence) -> Vec<String> {
    let mut failures = Vec::new();
    if !measured.maximum.is_finite() || measured.maximum < 0.0 {
        failures.push(format!(
            "{scene} {} max={} is not a finite non-negative channel error",
            tier.name(),
            measured.maximum,
        ));
    } else if measured.maximum > FAST_VISUAL_BUDGET_V1_MAX_CHANNEL_ERROR {
        failures.push(format!(
            "{scene} {} max={} at {:?} (certified {}, fast {}) exceeds \
             FAST_VISUAL_BUDGET_V1_MAX_CHANNEL_ERROR={FAST_VISUAL_BUDGET_V1_MAX_CHANNEL_ERROR}",
            tier.name(),
            measured.maximum,
            measured.maximum_at,
            measured.reference_at_maximum,
            measured.candidate_at_maximum,
        ));
    }
    if !measured.rms.is_finite() || measured.rms < 0.0 {
        failures.push(format!(
            "{scene} {} rms={} is not a finite non-negative channel error",
            tier.name(),
            measured.rms,
        ));
    } else if measured.rms > FAST_VISUAL_BUDGET_V1_RMS_CHANNEL_ERROR {
        failures.push(format!(
            "{scene} {} rms={} exceeds \
             FAST_VISUAL_BUDGET_V1_RMS_CHANNEL_ERROR={FAST_VISUAL_BUDGET_V1_RMS_CHANNEL_ERROR}",
            tier.name(),
            measured.rms,
        ));
    }
    if !measured.ssim.is_finite() || !(-1.0..=1.0).contains(&measured.ssim) {
        failures.push(format!(
            "{scene} {} ssim={} is outside the finite [-1, 1] metric domain",
            tier.name(),
            measured.ssim,
        ));
    } else if measured.ssim < FAST_VISUAL_BUDGET_V1_MIN_SSIM {
        failures.push(format!(
            "{scene} {} ssim={} is below \
             FAST_VISUAL_BUDGET_V1_MIN_SSIM={FAST_VISUAL_BUDGET_V1_MIN_SSIM}",
            tier.name(),
            measured.ssim,
        ));
    }
    failures
}

#[cfg(test)]
mod tests {
    use super::{Divergence, budget_v1_failures};
    use fmn_render::engine::Tier;

    fn identical() -> Divergence {
        Divergence {
            maximum: 0.0,
            maximum_at: (0, 0, 0),
            reference_at_maximum: 0.0,
            candidate_at_maximum: 0.0,
            rms: 0.0,
            ssim: 1.0,
        }
    }

    #[test]
    fn finite_identical_measurement_passes() {
        assert!(budget_v1_failures("identical", Tier::Scalar, &identical()).is_empty());
    }

    fn assert_invalid(case: &str, measured: Divergence) {
        assert!(
            !budget_v1_failures(case, Tier::Scalar, &measured).is_empty(),
            "invalid metric passed budget v1: {case}"
        );
    }

    #[test]
    fn invalid_metric_domains_fail_closed() {
        for (case, maximum) in [
            ("nan maximum", f64::NAN),
            ("infinite maximum", f64::INFINITY),
            ("negative maximum", -0.25),
        ] {
            assert_invalid(
                case,
                Divergence {
                    maximum,
                    ..identical()
                },
            );
        }
        for (case, rms) in [
            ("nan rms", f64::NAN),
            ("infinite rms", f64::INFINITY),
            ("negative rms", -0.25),
        ] {
            assert_invalid(case, Divergence { rms, ..identical() });
        }
        for (case, ssim) in [
            ("nan ssim", f64::NAN),
            ("infinite ssim", f64::INFINITY),
            ("ssim above one", 1.01),
            ("ssim below minus one", -1.01),
        ] {
            assert_invalid(
                case,
                Divergence {
                    ssim,
                    ..identical()
                },
            );
        }
    }
}
