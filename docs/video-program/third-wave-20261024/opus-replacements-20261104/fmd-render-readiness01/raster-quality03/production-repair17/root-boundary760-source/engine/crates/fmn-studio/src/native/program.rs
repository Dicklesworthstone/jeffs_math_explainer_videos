//! A resumable native program over Proscenium's existing stepped drivers.

use std::collections::VecDeque;

use fmn_render::CameraConfig;
use fmn_scene::studio_bridge::{Animation, FramePacket};
use fmn_scene::{
    CameraRig, CaptureReason, EventPayload, IntegrationError, InteractiveScene, NullSceneSink,
    PlayOverrides, Scene, SceneError, SceneSink, SteppedPlay, SteppedWait,
};

use crate::{InteractiveDispatch, InteractivePreview, ServiceError, SpanRegistry, WorkerErrorCode};

use super::{NativeBuild, NativeBuildContext};

/// Maximum cumulative segments admitted by one native program, including
/// deferred construction and every segment it generates, not just queue length.
pub const MAX_NATIVE_SEGMENTS: usize = 65_536;

/// A native animation, wait, or deferred authoring step.
///
/// Animation values and callback closures belong to one program instance. A
/// replay factory must construct fresh values, not clone their mutable captures.
pub enum NativeSegment {
    /// Play the supplied animations together with the ordinary timing overrides.
    Play {
        /// Owned native animations.
        animations: Vec<Box<dyn Animation>>,
        /// Scene-level timing overrides.
        overrides: PlayOverrides,
    },
    /// Wait for the supplied duration (or the Scene's configured default).
    Wait {
        /// Duration in seconds; sampling stays on the Scene's rational grid.
        duration: Option<f64>,
    },
    /// Run one-shot native construction after preceding segment epilogues.
    /// Prefer `defer`, `edit`, or `play_with` for ergonomic construction.
    Build {
        /// Owned callback; returned segments run before the queued continuation.
        build: NativeBuild,
    },
}

enum ActiveSegment {
    Play(SteppedPlay),
    Wait(SteppedWait),
}

impl ActiveSegment {
    fn advance(&mut self, scene: &mut Scene, sink: &mut CaptureOne) -> Result<bool, SceneError> {
        match self {
            Self::Play(play) => {
                if scene.prepare_stepped_play_frame(play)?.is_none() {
                    return Ok(false);
                }
                scene.complete_stepped_play_frame(play, sink)?;
            }
            Self::Wait(wait) => {
                if scene.prepare_stepped_wait_frame(wait)?.is_none() {
                    return Ok(false);
                }
                scene.complete_stepped_wait_frame(wait, sink)?;
            }
        }
        Ok(true)
    }

    fn finish(self, scene: &mut Scene) -> Result<(), SceneError> {
        match self {
            Self::Play(play) => {
                scene.finish_stepped_play(play, &mut NullSceneSink)?;
            }
            Self::Wait(wait) => {
                scene.finish_stepped_wait(wait, &mut NullSceneSink)?;
            }
        }
        Ok(())
    }

    fn abort(self, scene: &mut Scene) {
        match self {
            Self::Play(play) => scene.abort_stepped_play(play, &mut NullSceneSink),
            Self::Wait(wait) => scene.abort_stepped_wait(wait, &mut NullSceneSink),
        }
    }
}

#[derive(Default)]
struct CaptureOne {
    packet: Option<FramePacket>,
    invalid: bool,
}

impl SceneSink for CaptureOne {
    fn capture(
        &mut self,
        reason: CaptureReason,
        packet: FramePacket,
    ) -> Result<(), IntegrationError> {
        if reason != CaptureReason::Segment || self.packet.is_some() {
            self.invalid = true;
            return Err(IntegrationError::new(
                "studio",
                "expected one native segment capture",
            ));
        }
        self.packet = Some(packet);
        Ok(())
    }
}

/// A live, pausable native scene with its animations, callbacks and input owner.
///
/// Frame zero is the factory's constructed state; leading deferred steps run
/// only when advancing toward the first capture. Subsequent indices are actual
/// rational clock indices, not clock relabels. At most one capture is retained
/// per step. Pausing never runs later construction or a segment epilogue early.
/// An optional CameraRig lives in the same snapshotted Stage as the geometry.
pub struct NativeSceneProgram {
    preview: InteractivePreview,
    segments: VecDeque<NativeSegment>,
    active: Option<ActiveSegment>,
    frame: u64,
    frame_limit: u64,
    failed: bool,
    spans: SpanRegistry,
    camera_rig: Option<CameraRig>,
    admitted_segments: usize,
    started_segments: usize,
    segment_limit: usize,
}

impl NativeSceneProgram {
    /// Adopt an initial native Scene and a bounded schedule.
    ///
    /// Skip/range/presenter modes are refused: they do not have the one-capture
    /// per rational-clock-step contract required by interactive seeking.
    pub fn new(
        scene: Scene,
        segments: Vec<NativeSegment>,
        frame_limit: u64,
    ) -> Result<Self, ServiceError> {
        Self::from_interactive(
            InteractiveScene::new(scene).map_err(execution_error)?,
            segments,
            frame_limit,
        )
    }

    /// Adopt an existing editor without installing duplicate input listeners.
    pub fn from_interactive(
        scene: InteractiveScene,
        segments: Vec<NativeSegment>,
        frame_limit: u64,
    ) -> Result<Self, ServiceError> {
        let config = scene.config();
        if scene.time().frames() != 0
            || scene.play_count() != 0
            || config.skip_animations
            || config.start_at_play.is_some()
            || config.end_at_play.is_some()
            || config.presenter_mode
        {
            return Err(invalid(
                "native program requires a fresh non-skipping, non-presenter Scene",
            ));
        }
        if segments.len() > MAX_NATIVE_SEGMENTS || frame_limit > i64::MAX.cast_unsigned() {
            return Err(invalid("native program exceeds its segment or clock limit"));
        }
        let preview = InteractivePreview::from_interactive(scene).map_err(execution_error)?;
        let admitted_segments = segments.len();
        Ok(Self {
            preview,
            segments: segments.into(),
            active: None,
            frame: 0,
            frame_limit,
            failed: false,
            spans: SpanRegistry::new(),
            camera_rig: None,
            admitted_segments,
            started_segments: 0,
            segment_limit: MAX_NATIVE_SEGMENTS,
        })
    }

    /// Tighten cumulative schedule admission before execution. Every deferred
    /// step and every returned segment spends one slot for the program's entire
    /// lifetime. Recursive expansion cannot evade the limit by keeping a short
    /// pending queue. Zero permits only an initially empty schedule.
    pub fn with_segment_limit(mut self, limit: usize) -> Result<Self, ServiceError> {
        self.require_healthy()?;
        if self.started_segments != 0
            || limit > MAX_NATIVE_SEGMENTS
            || limit < self.admitted_segments
        {
            return Err(invalid(
                "native segment limit must cover the initial schedule and be set before execution",
            ));
        }
        self.segment_limit = limit;
        Ok(self)
    }

    /// Bind one native camera family before the first segment starts. The rig
    /// must belong to this Scene. Binding never executes an updater, and cannot
    /// be replaced partway through a program. Select CameraConfig on the worker
    /// as well; its resolution, aspect and capture policy remain authoritative.
    pub fn with_camera_rig(mut self, rig: CameraRig) -> Result<Self, ServiceError> {
        self.require_healthy()?;
        if self.frame != 0
            || self.started_segments != 0
            || self.active.is_some()
            || self.camera_rig.is_some()
            || self.preview.scene().play_count() != 0
        {
            return Err(invalid("bind a camera rig once, before native playback"));
        }
        rig.sample(self.preview.stage(), &CameraConfig::default())
            .map_err(execution_error)?;
        self.camera_rig = Some(rig);
        Ok(self)
    }

    /// Original tracker handles for the optional camera. Values are sampled
    /// from captured native state, never from a renderer-side timing callback.
    #[must_use]
    pub const fn camera_rig(&self) -> Option<CameraRig> {
        self.camera_rig
    }

    pub(super) fn camera_binding_index(&self) -> Result<Option<u64>, ServiceError> {
        self.camera_rig
            .map(|rig| {
                rig.binding_index(self.preview.stage())
                    .map_err(execution_error)
            })
            .transpose()
    }

    fn validate_camera(&self) -> Result<(), ServiceError> {
        if let Some(rig) = self.camera_rig {
            rig.sample(self.preview.stage(), &CameraConfig::default())
                .map_err(execution_error)?;
        }
        Ok(())
    }

    /// Latest completed capture, or zero for the initial constructed state.
    #[must_use]
    pub const fn frame_index(&self) -> u64 {
        self.frame
    }

    /// Highest admitted capture index (zero is always available initially).
    #[must_use]
    pub const fn frame_limit(&self) -> u64 {
        self.frame_limit
    }

    /// The actual Scene/editor owner; its clock and callbacks are never rebuilt
    /// from a callable-free durable snapshot.
    #[must_use]
    pub fn preview(&self) -> &InteractivePreview {
        &self.preview
    }

    /// Source spans bound to this instance's original handles.
    #[must_use]
    pub fn spans(&self) -> &SpanRegistry {
        &self.spans
    }

    /// Bind source spans while constructing the program. Deferred constructors
    /// can register their spans through NativeBuildContext as well.
    pub fn spans_mut(&mut self) -> &mut SpanRegistry {
        &mut self.spans
    }

    /// Dispatch native editing or application input at the paused boundary.
    /// Failure or unwinding poisons the program until its owner rebuilds it.
    pub fn dispatch(&mut self, event: EventPayload) -> Result<InteractiveDispatch, ServiceError> {
        self.require_healthy()?;
        event
            .validate()
            .map_err(|error| invalid(error.to_string()))?;
        self.failed = true;
        let result = self
            .preview
            .dispatch(event)
            .map_err(execution_error)
            .and_then(|receipt| {
                self.validate_camera()?;
                Ok(receipt)
            });
        self.failed = result.is_err();
        result
    }

    /// Capture the actual Scene clock, RNG and Stage, including rig channels.
    /// Invalid camera values cannot become successful checkpoint receipts.
    pub fn state_bytes(&mut self) -> Result<Vec<u8>, ServiceError> {
        self.require_healthy()?;
        self.validate_camera()?;
        self.preview
            .scene_mut()
            .state_bytes()
            .map_err(execution_error)
    }

    /// Execute the next ordinary native capture, retaining executable state for
    /// the next call. At the frame limit, refusal happens before any epilogue or
    /// deferred callback. Errors and panics poison the cursor: a caller catching
    /// an unwind cannot accidentally reuse partially consumed authoring work.
    pub fn next_frame(&mut self) -> Result<Option<FramePacket>, ServiceError> {
        self.require_healthy()?;
        if self.frame >= self.frame_limit {
            return Err(invalid("native execution reached its frame budget"));
        }
        self.failed = true;
        let result = self.next_frame_inner().and_then(|packet| {
            self.validate_camera()?;
            Ok(packet)
        });
        self.failed = result.is_err();
        result
    }

    /// Advance by real execution only. Backward movement requires a fresh
    /// factory instance; changing the clock alone is never treated as replay.
    pub fn advance_to(&mut self, target: u64) -> Result<(), ServiceError> {
        self.require_healthy()?;
        if target < self.frame || target > self.frame_limit {
            return Err(invalid(
                "native seek requires a forward target within the frame limit",
            ));
        }
        while self.frame < target {
            if self.next_frame()?.is_none() {
                return Err(invalid("native program ended before the requested frame"));
            }
        }
        Ok(())
    }

    fn require_healthy(&self) -> Result<(), ServiceError> {
        if self.failed {
            Err(execution_error(
                "native program failed; reconstruct it from its factory",
            ))
        } else {
            Ok(())
        }
    }

    fn prepend(&mut self, segments: Vec<NativeSegment>) -> Result<(), ServiceError> {
        let admitted = self
            .admitted_segments
            .checked_add(segments.len())
            .filter(|count| *count <= self.segment_limit)
            .ok_or_else(|| {
                invalid("deferred native construction exceeded its cumulative segment budget")
            })?;
        self.segments
            .try_reserve(segments.len())
            .map_err(execution_error)?;
        for segment in segments.into_iter().rev() {
            self.segments.push_front(segment);
        }
        self.admitted_segments = admitted;
        Ok(())
    }

    fn next_frame_inner(&mut self) -> Result<Option<FramePacket>, ServiceError> {
        loop {
            if self.active.is_none() {
                let Some(segment) = self.segments.pop_front() else {
                    return Ok(None);
                };
                self.started_segments += 1;
                self.active = match segment {
                    NativeSegment::Play {
                        animations,
                        overrides,
                    } => self
                        .preview
                        .scene_mut()
                        .begin_stepped_play(animations, overrides, &mut NullSceneSink)
                        .map_err(execution_error)?
                        .map(ActiveSegment::Play),
                    NativeSegment::Wait { duration } => Some(ActiveSegment::Wait(
                        self.preview
                            .scene_mut()
                            .begin_stepped_wait(duration, &mut NullSceneSink)
                            .map_err(execution_error)?,
                    )),
                    NativeSegment::Build { build } => {
                        let generated = build(&mut NativeBuildContext {
                            scene: self.preview.scene_mut(),
                            spans: &mut self.spans,
                        })
                        .map_err(execution_error)?;
                        self.validate_camera()?;
                        self.prepend(generated)?;
                        None
                    }
                };
                continue;
            }
            let Some(mut active) = self.active.take() else {
                return Err(execution_error("native segment cursor disappeared"));
            };
            let mut capture = CaptureOne::default();
            match active.advance(self.preview.scene_mut(), &mut capture) {
                Ok(false) => active
                    .finish(self.preview.scene_mut())
                    .map_err(execution_error)?,
                Ok(true) => {
                    self.active = Some(active);
                    let expected = self.frame + 1;
                    let packet = capture
                        .packet
                        .ok_or_else(|| execution_error("native step omitted its capture"))?;
                    if capture.invalid
                        || u64::try_from(packet.frame_index()).ok() != Some(expected)
                        || self.preview.scene().time() != packet.time()
                    {
                        return Err(execution_error(
                            "native capture disagreed with its rational clock",
                        ));
                    }
                    self.frame = expected;
                    return Ok(Some(packet));
                }
                Err(error) => {
                    active.abort(self.preview.scene_mut());
                    return Err(execution_error(error));
                }
            }
        }
    }
}

pub(super) fn execution_error(error: impl std::fmt::Display) -> ServiceError {
    ServiceError::new(WorkerErrorCode::ExecutionFailed, error.to_string())
}

pub(super) fn invalid(message: impl Into<String>) -> ServiceError {
    ServiceError::new(WorkerErrorCode::InvalidRequest, message)
}
