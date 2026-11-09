//! The tier-2 demo bundle builder (fm-oee): the player's demo scene,
//! authored as a declarative [`fmn_anim::timeline::Timeline`] over the same
//! primitive-corpus mobjects the tier-1 surface renders, and exported through
//! the scene-side FMTL/1 writer ([`fmn_scene::export_timeline_bundle`]).
//!
//! This is host-side tooling — bundle export is a scene-side operation,
//! and the wasm artifact never carries it (`#[cfg(not(target_arch =
//! "wasm32"))]`; the module simply does not exist there). The demo page
//! (`demo/wasm/player.html`) consumes the exported `bundle.fmtl`; the
//! exporter binary is `crates/fmn-wasm/examples/export_bundle.rs`.
//!
//! The scene: the corpus circle and Lissajous wave, one one-second play
//! (a dyadic shift of the circle and a dyadic scale of the wave — two
//! [`fmn_anim::Transform`]s at the Reference's default `smooth` rate, the shape
//! of segment the export-time proof passes as pure-reconstructible), then a
//! half-second hold so the terminal state is scrubbable too. Labels mark the
//! segment boundaries.

use fmn_anim::timeline::Timeline;
use fmn_anim::{AnimConfig, Transform, prepare_animation};
use fmn_core::rng::RngRoot;
use fmn_mobject::Stage;
use fmn_scene::BundleError;

use crate::{circle_mobject, parametric_mobject};

/// The demo's frame rate (the tier-1 surface's convention).
const FPS: u32 = 30;
/// Bundle construction takes no entropy; the seed is a named constant.
const SEED: u64 = 0;
/// One second of play, then a half-second hold.
const RUN_TIME_SECONDS: f64 = 1.0;
const WAIT_SECONDS: f64 = 0.5;

/// Build the demo scene: the stage and the authored (unrun) timeline.
/// Factored from [`demo_bundle`] so the scrub-correctness test can run the
/// same schedule through the native engine for ground truth.
///
/// # Errors
/// [`BundleError`] from animation preparation or timeline authoring.
pub fn demo_scene() -> Result<(Stage, Timeline), BundleError> {
    let mut stage = Stage::new();
    let circle = stage.add(circle_mobject([-1.6, 0.0, 0.0], 1.0));
    stage
        .add_to_scene(circle)
        .map_err(|_| BundleError::PlanDrift("a fresh circle handle failed to root"))?;
    let wave = stage.add(parametric_mobject());
    stage
        .add_to_scene(wave)
        .map_err(|_| BundleError::PlanDrift("a fresh wave handle failed to root"))?;

    // One play, two pure Transforms at the same rate: the circle shifts a
    // dyadic 2 units right, the wave scales a dyadic 5/4 about its center.
    // Dyadic parameters keep the end-state arithmetic exact, which is what
    // lets the export-time proof pass these as kind 0.
    let circle_target = stage
        .copy_family(circle)
        .map_err(|_| BundleError::PlanDrift("circle copy failed"))?;
    stage.shift(circle_target, [2.0, 0.0, 0.0]);
    let wave_target = stage
        .copy_family(wave)
        .map_err(|_| BundleError::PlanDrift("wave copy failed"))?;
    stage.scale(wave_target, 1.25);

    let config = AnimConfig {
        run_time: RUN_TIME_SECONDS,
        ..AnimConfig::default()
    };
    let shift = Transform::new(circle, circle_target).with_config(config.clone());
    let grow = Transform::new(wave, wave_target).with_config(config);
    let shift = prepare_animation(shift, &mut stage)?;
    let grow = prepare_animation(grow, &mut stage)?;

    let mut timeline = Timeline::new(FPS).map_err(BundleError::Anim)?;
    timeline.label("shift");
    timeline
        .play(vec![shift, grow])
        .map_err(BundleError::Anim)?;
    timeline.label("settle");
    timeline.wait(WAIT_SECONDS).map_err(BundleError::Anim)?;
    Ok((stage, timeline))
}

/// Build and export the demo timeline bundle.
///
/// # Errors
/// [`BundleError`] from the timeline run or the container write.
pub fn demo_bundle() -> Result<Vec<u8>, BundleError> {
    let (mut stage, timeline) = demo_scene()?;
    let rng = RngRoot::from_seed(SEED);
    fmn_scene::export_timeline_bundle(timeline, &mut stage, &rng)
}
