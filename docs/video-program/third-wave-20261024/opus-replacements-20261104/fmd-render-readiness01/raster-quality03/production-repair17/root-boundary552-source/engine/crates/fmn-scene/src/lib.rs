//! Proscenium — the Scene runtime and replay foundations (§13.1, §13.4).
//!
//! The [`runtime`] module owns the Scene state machine on Choreo's rational
//! clock: lifecycle orchestration, frame/preflight/presenter integration
//! boundaries, stable Stage membership operations, skip/range semantics,
//! SceneState restore, 3D defaults, discovery/selection, and output naming.
//! [`events`] is the typed, insertion-ordered dispatcher and production input
//! inbox; [`interactive`] layers Reference-compatible selection/editing
//! behavior over that one serial pre-capture seam.
//! The [`journal`] module is the replay journal + effect model — the one
//! record with three consumers: supervisor edit-replay, the purity
//! classifier's journaled evidence, and the pipeline's barrier vocabulary.
#![forbid(unsafe_code)]

pub mod camera_rig;
pub mod events;
pub mod interactive;
pub mod journal;
pub mod recording;
pub mod runtime;
pub mod timeline_bundle;

pub use camera_rig::CameraRig;

/// Lower-layer scene types exposed through Proscenium for Studio and
/// portal adapters.
///
/// `fmn-studio` and `fmn-python` sit above `fmn-scene` in the governed
/// crate DAG. Keeping this narrow facade here lets those consumers reach
/// the state that Scene already owns — including the camera frame the
/// scene presents to every front door — without adding undeclared direct
/// edges to Marionette, Choreo, or Lumen.
pub mod studio_bridge {
    pub use fmn_anim::{AnimError, Animation, FramePacket, Timeline};
    pub use fmn_mobject::{Mob, SceneState, Snapshot, Stage, Uniforms};
    pub use fmn_render::{CameraError, CameraFrame};
}

pub use events::{
    DispatchState, EventDispatcher, EventError, EventInbox, EventListener, EventListenerId,
    EventPayload, EventPropagation, EventTarget, EventType, InputEvent, Key, Modifiers,
    MouseButton,
};
pub use interactive::{
    InteractiveAction, InteractiveClipboard, InteractiveScene, KeyBinding, REFERENCE_KEYBOARD_MAP,
    SelectionHighlight, SelectionRectangle,
};
pub use journal::{
    AssetRead, BundleDivergence, CommandKind, CommandRecord, EffectClass, Entry, ImpureEffectTag,
    InvalidationReason, Journal, JournalError, MAX_RENDER_BACKEND_IDENTITY_BYTES,
    RenderBackendRecord, RenderBackendRole, ReplayAudit, ReplayPlan, ReproBundle, SubprocessRecord,
    plan_replay,
};
pub use runtime::{
    BlankScene, CameraOrientation, CaptureReason, CompletionRequest, EndScene, HoldController,
    HoldDecision, HoldKind, IntegrationError, LifecycleEvent, LifecyclePhase, NullSceneSink,
    OutputNaming, PlayOverrides, RuntimeConfig, Scene, SceneError, SceneProgram, SceneRegistration,
    SceneRegistry, SceneRunReport, SceneSelection, SceneSelectionError, SceneSink,
    SceneStateRestore, SoundRequest, SteppedPlay, SteppedWait, ThreeDAddOptions, ThreeDScene,
};
pub use timeline_bundle::{
    BundleError, BundleExportLimits, BundleReadError, BundleSegmentKind, DEFAULT_MAX_BUNDLE_BYTES,
    DEFAULT_MAX_BUNDLE_EXPORT_FRAMES, TIMELINE_BUNDLE_SCHEMA, TimelineBundle,
    bundle_engine_version, export_timeline_bundle, export_timeline_bundle_with_limits,
};
