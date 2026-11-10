//! The engine-equivalence suite (§16.3 plane 4, fm-t1v W10): certified CPU
//! versus fast CPU under the versioned v1 visual budget — **the
//! engine-change blocker**.
//!
//! Budget v1 (see `src/equivalence.rs`'s module docs for the contract):
//!
//! - max linear channel error ≤ `FAST_VISUAL_BUDGET_V1_MAX_CHANNEL_ERROR`
//! - RMS linear channel error ≤ `FAST_VISUAL_BUDGET_V1_RMS_CHANNEL_ERROR`
//! - global sRGB-luma SSIM ≥ `FAST_VISUAL_BUDGET_V1_MIN_SSIM`
//!
//! A violation fails this suite. Loosening a bound is an adjudicated edit
//! to the `fmn-render` constant with a re-measurement in its doc comment —
//! never a scene edit. There is nothing to bless here: the certified
//! engine's pixels are the reference, and the fast engine must meet them.
//!
//! The pixel walk lives in this target because `fmn-frame` is a documented
//! dev-only edge of this crate (see Cargo.toml); `src/equivalence.rs` holds
//! the frame-free budget contract.

use fmn_conformance::equivalence::{Divergence, budget_v1_failures};
use fmn_conformance::scene_goldens::{
    EQUIVALENCE_SUBSET, TILING, corpus, frame_config, scene_named,
};
use fmn_frame::{FrameBuffer, FrameLayout, PixelFormat};
use fmn_mobject::Stage;
use fmn_render::bin::Binning;
use fmn_render::engine::{EngineIdentity, FrameJob, Tier};
use fmn_render::fill::MonoTable;
use fmn_render::plan::RenderPlan;

/// How many threads the fast route renders with. Thread count is not part
/// of an engine identity (C10), so the blocker exercises a multithreaded
/// fast render: scheduling must not spend budget a single-threaded run
/// would keep.
const FAST_THREADS: usize = 4;

/// Render a stage through one explicitly journaled engine identity into
/// the raw certified frame layout. The same derivation lives in
/// `tests/scene_goldens.rs` — keep the two in step.
fn render_frame(stage: &Stage, identity: EngineIdentity, threads: usize) -> FrameBuffer {
    let config = frame_config();
    let mut plan = RenderPlan::new();
    plan.sync(stage, 0)
        .expect("valid engine-equivalence fixture");
    let mono = MonoTable::build(&plan, config.map).expect("bounded equivalence monotone table");
    let mut binning = Binning::build(&plan, config.viewport, TILING, config.map)
        .expect("bounded conformance binning");
    binning
        .prune_occluded(&plan)
        .expect("occlusion pruning matches the plan");
    FrameJob::with_identity(&plan, &mono, &binning, config, identity)
        .expect("frame artifacts match the plan")
        .render(threads)
        .expect("the engine renders the frame")
}

/// Compare two equally laid-out raw linear-light frames under budget v1's
/// three metrics — the formula `tests/certified_engine.rs` established,
/// kept verbatim so all three lanes measure identically.
fn divergence(reference: &FrameBuffer, candidate: &FrameBuffer) -> Divergence {
    assert_eq!(
        reference.layout(),
        candidate.layout(),
        "engine-equivalence layouts differ"
    );
    let mut maximum = 0.0_f64;
    let mut maximum_at = (0, 0, 0);
    let mut reference_at_maximum = 0.0;
    let mut candidate_at_maximum = 0.0;
    let mut squared = 0.0;
    let mut channels = 0_u64;
    for y in 0..reference.layout().height() {
        for x in 0..reference.layout().width() {
            for (channel, (a, b)) in read_pixel(reference, x, y)
                .into_iter()
                .zip(read_pixel(candidate, x, y))
                .enumerate()
            {
                let error = (a - b).abs();
                if error > maximum {
                    maximum = error;
                    maximum_at = (x, y, channel);
                    reference_at_maximum = a;
                    candidate_at_maximum = b;
                }
                squared += error * error;
                channels += 1;
            }
        }
    }
    assert!(channels > 0, "a frame comparison must contain channels");
    Divergence {
        maximum,
        maximum_at,
        reference_at_maximum,
        candidate_at_maximum,
        rms: (squared / channels as f64).sqrt(),
        ssim: ssim_luma(reference, candidate),
    }
}

/// Decode one `Rgba16F` pixel into exact `f64` values.
fn read_pixel(frame: &FrameBuffer, x: u32, y: u32) -> [f64; 4] {
    let base = y as usize * frame.layout().stride(0) + x as usize * 8;
    let pixel = &frame.plane(0)[base..base + 8];
    let mut decoded = [0.0; 4];
    for (channel, value) in decoded.iter_mut().enumerate() {
        *value = fmn_frame::half::f16_to_f64(u16::from_le_bytes([
            pixel[channel * 2],
            pixel[channel * 2 + 1],
        ]));
    }
    decoded
}

/// Global SSIM over the canonical sRGB8 Rec. 709 luma plane.
///
/// The spike deliberately selected the global form as a stable smoke alarm
/// rather than shipping an unreviewed windowed metric. The production
/// engine-equivalence lane keeps that exact ruling and pairs it with the
/// hard linear-channel bounds above.
fn ssim_luma(reference: &FrameBuffer, candidate: &FrameBuffer) -> f64 {
    let luma = |frame: &FrameBuffer| {
        let layout = FrameLayout::tight(
            PixelFormat::Rgba8,
            frame.layout().width(),
            frame.layout().height(),
        )
        .expect("the comparison layout is valid");
        let mut encoded = FrameBuffer::new(layout);
        fmn_frame::convert::rgba16f_to_rgba8(frame, &mut encoded)
            .expect("the raw frame converts canonically");
        let mut values =
            Vec::with_capacity(frame.layout().width() as usize * frame.layout().height() as usize);
        let width_bytes = frame.layout().width() as usize * 4;
        for y in 0..frame.layout().height() as usize {
            let row = &encoded.plane(0)
                [y * encoded.layout().stride(0)..y * encoded.layout().stride(0) + width_bytes];
            values.extend(row.as_chunks::<4>().0.iter().map(|pixel| {
                0.2126 * f64::from(pixel[0])
                    + 0.7152 * f64::from(pixel[1])
                    + 0.0722 * f64::from(pixel[2])
            }));
        }
        values
    };
    let reference = luma(reference);
    let candidate = luma(candidate);
    assert_eq!(
        reference.len(),
        candidate.len(),
        "SSIM planes differ in length"
    );
    assert!(!reference.is_empty(), "SSIM requires at least one pixel");
    let count = reference.len() as f64;
    let reference_mean = reference.iter().sum::<f64>() / count;
    let candidate_mean = candidate.iter().sum::<f64>() / count;
    let mut reference_variance = 0.0;
    let mut candidate_variance = 0.0;
    let mut covariance = 0.0;
    for (&reference, &candidate) in reference.iter().zip(&candidate) {
        reference_variance += (reference - reference_mean) * (reference - reference_mean);
        candidate_variance += (candidate - candidate_mean) * (candidate - candidate_mean);
        covariance += (reference - reference_mean) * (candidate - candidate_mean);
    }
    let divisor = (count - 1.0).max(1.0);
    reference_variance /= divisor;
    candidate_variance /= divisor;
    covariance /= divisor;

    let c1 = (0.01 * 255.0_f64).powi(2);
    let c2 = (0.03 * 255.0_f64).powi(2);
    ((2.0 * reference_mean * candidate_mean + c1) * (2.0 * covariance + c2))
        / ((reference_mean * reference_mean + candidate_mean * candidate_mean + c1)
            * (reference_variance + candidate_variance + c2))
}

#[test]
fn fast_cpu_stays_inside_the_versioned_v1_budget() {
    let corpus = corpus();
    let mut failures = Vec::new();
    for &name in EQUIVALENCE_SUBSET {
        let case = scene_named(name).expect("the subset names real scenes");
        let built = (case.build)(corpus);
        let certified = render_frame(&built.stage, EngineIdentity::certified(), 1);
        for &tier in Tier::ALL {
            let identity = EngineIdentity {
                tier,
                ..EngineIdentity::fast()
            };
            let fast = render_frame(&built.stage, identity, FAST_THREADS);
            let measured = divergence(&certified, &fast);
            failures.extend(budget_v1_failures(name, tier, &measured));
        }
    }
    assert!(
        failures.is_empty(),
        "engine-equivalence budget v1 violated:\n{}",
        failures.join("\n")
    );
}

/// fm-sq8.11: content the Python portal authors. A pure-2D portal scene
/// renders on Lumen's retained planar route, and its frames equal native
/// replay of the scene's FMTL/1 export (e2e
/// `render_matrix.python_portal_planar_route.v1`). Replaying that export's
/// stages here holds every fast tier to budget v1 on what the portal
/// produces (a Transform mid-flight, BLUE fill under a thick stroke), not
/// only on the native corpus.
#[test]
fn portal_planar_content_stays_inside_the_versioned_v1_budget() {
    let root = std::env::temp_dir().join(format!(
        "fmn-engine-equivalence-portal-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).expect("portal scratch directory");
    let report =
        manimlib::run_portal_gauntlet_planar_route(&root).expect("the portal planar route renders");
    let bytes = std::fs::read(&report.bundle).expect("the portal's FMTL/1 export");
    let bundle = fmn_scene::timeline_bundle::TimelineBundle::from_bytes(&bytes)
        .expect("the portal's FMTL/1 export decodes");
    assert!(!bundle.requires_camera(), "a planar scene needs no camera");
    let frames = u32::try_from(report.frame_count).expect("a small frame count");
    assert!(frames >= 3, "the scene animates over {frames} frames");
    let mut failures = Vec::new();
    for index in [0, frames / 2, frames - 1] {
        let stage = bundle
            .stage_at(index)
            .expect("every exported frame reconstructs");
        let certified = render_frame(&stage, EngineIdentity::certified(), 1);
        for &tier in Tier::ALL {
            let identity = EngineIdentity {
                tier,
                ..EngineIdentity::fast()
            };
            let fast = render_frame(&stage, identity, FAST_THREADS);
            let measured = divergence(&certified, &fast);
            failures.extend(budget_v1_failures(
                &format!("portal.flat@{index}"),
                tier,
                &measured,
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "engine-equivalence budget v1 violated on portal content:\n{}",
        failures.join("\n")
    );
}

#[test]
fn the_equivalence_subset_is_not_empty_and_is_stable() {
    // The blocker only blocks while it covers the engine's stress families;
    // silently shrinking the subset must itself fail.
    assert!(
        EQUIVALENCE_SUBSET.len() >= 10,
        "the equivalence subset shrank to {}",
        EQUIVALENCE_SUBSET.len()
    );
    for &name in EQUIVALENCE_SUBSET {
        assert!(
            scene_named(name).is_some(),
            "equivalence subset names unknown scene {name}"
        );
    }
}

#[test]
fn the_blocker_actually_blocks() {
    // Sensitivity: the suite is worthless if a visibly wrong frame passes.
    // Perturb one pixel past the max-channel bound and the verdict must
    // fire; leave it alone and it must not.
    let corpus = corpus();
    let case = scene_named("circle_tex_label.v1").expect("scene exists");
    let built = (case.build)(corpus);
    let certified = render_frame(&built.stage, EngineIdentity::certified(), 1);

    let mut altered = FrameBuffer::new(certified.layout().clone());
    altered.as_bytes_mut().copy_from_slice(certified.as_bytes());
    // The whole first row, every channel, forced to half-float 1.0 — far
    // past any plausible engine-arithmetic drift, and large enough to move
    // all three metrics.
    let row = certified.layout().width() as usize * 8;
    for bytes in altered.plane_mut(0)[..row].as_chunks_mut::<2>().0 {
        bytes.copy_from_slice(&0x3C00_u16.to_le_bytes());
    }

    let measured = divergence(&certified, &altered);
    let failures = budget_v1_failures(case.name, Tier::Scalar, &measured);
    assert!(
        !failures.is_empty(),
        "a perturbed frame passed budget v1: {measured:?}"
    );
    assert!(
        measured.maximum > 0.09,
        "the perturbation must move the max metric: {measured:?}"
    );
}

#[test]
fn identical_frames_measure_zero_divergence() {
    // The measurement's own floor: a frame against itself must measure
    // exactly zero max/RMS error and SSIM 1, or the budget arithmetic is
    // broken rather than the engine.
    let corpus = corpus();
    let case = scene_named("circle_tex_label.v1").expect("scene exists");
    let built = (case.build)(corpus);
    let frame = render_frame(&built.stage, EngineIdentity::certified(), 1);
    let measured = divergence(&frame, &frame);
    assert_eq!(measured.maximum, 0.0);
    assert_eq!(measured.rms, 0.0);
    assert_eq!(measured.ssim, 1.0);
    assert!(budget_v1_failures(case.name, Tier::Scalar, &measured).is_empty());
}
