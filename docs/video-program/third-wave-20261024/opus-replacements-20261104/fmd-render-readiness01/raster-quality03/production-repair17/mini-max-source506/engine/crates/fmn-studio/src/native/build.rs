//! Deferred authoring at a completed native segment boundary.
//!
//! The context exposes scene data, its one RNG and listener registration, but
//! not mutable Scene playback. Construction cannot consume unreported play/wait
//! frames through this API.

use fmn_core::rng::Pcg64Dxsm;
use fmn_render::CameraConfig;
use fmn_scene::studio_bridge::{Animation, Stage};
use fmn_scene::{CameraRig, EventDispatcher, PlayOverrides, Scene, SceneError};

use crate::SpanRegistry;

use super::NativeSegment;

/// One-shot construction over the state left by all preceding segments.
///
/// Invocations belong to the disposable native worker. A replay factory must
/// create a fresh closure and fresh mutable captures on each reconstruction.
/// The callback itself is not serialized, cloned, or claimed pure.
pub type NativeBuild =
    Box<dyn FnOnce(&mut NativeBuildContext<'_>) -> Result<Vec<NativeSegment>, SceneError>>;

/// Native construction access at an ordered boundary between segments.
///
/// Returned segments execute before the previously queued continuation. Scene
/// time and the playback driver are read-only here. Stage mutations and newly
/// registered callbacks take effect at the next ordinary native capture.
pub struct NativeBuildContext<'a> {
    pub(super) scene: &'a mut Scene,
    pub(super) spans: &'a mut SpanRegistry,
}

impl NativeBuildContext<'_> {
    /// Read completed scene time, play count, runtime policy, and native state.
    #[must_use]
    pub fn scene(&self) -> &Scene {
        self.scene
    }

    /// Read the original native arena, including all earlier completed edits.
    #[must_use]
    pub fn stage(&self) -> &Stage {
        self.scene.stage()
    }

    /// Build mobjects, alter membership, register updaters, and construct native
    /// animation targets in the original arena. No runtime clock is advanced.
    pub fn stage_mut(&mut self) -> &mut Stage {
        self.scene.stage_mut()
    }

    /// The existing scene-serial PCG64DXSM substream. Deferred random choices
    /// consume the same snapshotted RNG as ordinary Scene construction; no
    /// ambient entropy, independent seed or renderer-thread stream is introduced.
    pub fn rng_mut(&mut self) -> &mut Pcg64Dxsm {
        self.scene.rng_mut()
    }

    /// Register/remove native listeners for content constructed at this point.
    /// Input still runs through the Scene's single dispatcher and capture seam;
    /// obtaining this handle neither queues nor dispatches an event.
    pub fn event_dispatcher_mut(&mut self) -> &mut EventDispatcher {
        self.scene.event_dispatcher_mut()
    }

    /// Attach native source/type-setting spans to newly constructed handles.
    pub fn spans_mut(&mut self) -> &mut SpanRegistry {
        self.spans
    }

    /// Record an authored sound cue at this completed segment boundary.
    ///
    /// The ordinary Scene validation and rational call-site clock apply. This
    /// records the request for output composition; it does not start playback,
    /// decode an asset, or consume a preview frame.
    pub fn add_sound(
        &mut self,
        sound_file: impl Into<std::path::PathBuf>,
        time_offset: f64,
        gain: Option<f64>,
        gain_to_background: Option<f64>,
    ) -> Result<(), SceneError> {
        self.scene
            .add_sound(sound_file, time_offset, gain, gain_to_background)?;
        Ok(())
    }

    /// Construct a camera transition from the *current* completed rig state.
    /// The existing Transform remains responsible for interpolation/lifecycle.
    pub fn camera_to(
        &mut self,
        rig: CameraRig,
        target: &CameraConfig,
    ) -> Result<Box<dyn Animation>, SceneError> {
        Ok(Box::new(rig.animate_to(self.scene, target)?))
    }
}

impl NativeSegment {
    /// Defer construction until preceding segments have finished, including
    /// their final-alpha, remover and updater-resume epilogues. The returned
    /// segments are prepended in order; nested deferrals execute depth first.
    ///
    /// Merely constructing a program, reading a paused frame, or seeking frame
    /// zero does not invoke this callback. Failure poisons the program instead
    /// of retrying a one-shot callback or pretending its side effects rolled back.
    pub fn defer(
        build: impl FnOnce(&mut NativeBuildContext<'_>) -> Result<Vec<Self>, SceneError> + 'static,
    ) -> Self {
        Self::Build {
            build: Box::new(build),
        }
    }

    /// Apply a scene-data edit between segments without emitting an extra frame
    /// or incrementing the native play count. Add/remove/style operations belong
    /// here rather than being applied while the whole schedule is constructed.
    pub fn edit(
        edit: impl FnOnce(&mut NativeBuildContext<'_>) -> Result<(), SceneError> + 'static,
    ) -> Self {
        Self::defer(move |context| {
            edit(context)?;
            Ok(Vec::new())
        })
    }

    /// Build animation objects from the preceding segment's completed state.
    /// Ordinary timing, easing, suspension and compositions remain native.
    pub fn play_with(
        overrides: PlayOverrides,
        build: impl FnOnce(&mut NativeBuildContext<'_>) -> Result<Vec<Box<dyn Animation>>, SceneError>
        + 'static,
    ) -> Self {
        Self::defer(move |context| {
            Ok(vec![Self::Play {
                animations: build(context)?,
                overrides,
            }])
        })
    }
}
