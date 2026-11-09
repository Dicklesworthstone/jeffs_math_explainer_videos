//! The Proscenium Scene state machine (§13.1, §1.4, fm-5xm).
//!
//! This layer deliberately owns orchestration, not mechanisms already owned
//! below it:
//!
//! - Marionette's [`Stage`] owns family membership, stable z-ordering, and CoW
//!   snapshots.
//! - Choreo's [`fmn_anim::play_segment`] / [`fmn_anim::wait_segment`] own the
//!   six-step frame order and immutable [`FramePacket`] boundary.
//! - Scribe plugs into [`Scene::set_preflight_hook`] and sees the complete
//!   constructed root walk plus the first play's animation closure exactly
//!   once before the first captured frame.
//! - Lumen/Reel/Studio plug into [`SceneSink`]. `fmn-scene` cannot depend on
//!   `fmn-output` without inverting §19's crate DAG, so the composition root
//!   converts packets to pixels and submits those pixels to Reel.
//!
//! The sink also receives typed lifecycle events. They are the structured,
//! assertion-friendly surface for “preflight happened before frame zero” and
//! “pre/play/post ran in order”; human decoration belongs above this crate.

use std::collections::VecDeque;
use std::fmt;
use std::path::{Path, PathBuf};

mod idle;
pub use idle::IdleFrame;

use fmn_anim::{
    AnimError, Animation, AnimationBoundary, FramePacket, ImpureEffect, OpenSegment, OpenWait,
    Purity, RateFunc, RationalFrameClock, RationalTime, SceneUpdaterBoundary, SegmentKind,
    SegmentReport, abort_open_play, abort_open_wait, complete_play_frame, complete_wait_frame,
    finish_open_play, finish_open_play_animations, finish_open_wait, open_play_with_mode,
    open_wait, open_wait_after_updaters, play_segment_with_boundary, prepare_play_animation,
    prepare_play_frame, prepare_wait_frame, validate_play, wait_segment_with_boundary,
};
use fmn_core::rng::{Pcg64Dxsm, RngRoot};
use fmn_hash::SerialError;
use fmn_mobject::{
    Mob, Mobject, PersistError, SceneState, Stage, StageError, UpdaterFn, UpdaterId,
    UpdaterKindTag, UpdaterManifest,
};
use fmn_render::{
    CameraConfig as RenderCameraConfig, CameraError, CameraFrame, THREE_D_CAMERA_SAMPLES,
    ThreeDCamera,
};

use crate::events::{EventDispatcher, EventError, EventInbox, EventPayload, InputEvent};

const DEFAULT_FPS: u32 = 30;
const DEFAULT_WAIT_TIME: f64 = 1.0;
const DEFAULT_MAX_PENDING_EVENTS: usize = 4096;
const THREE_D_THETA_DEGREES: f64 = -30.0;
const THREE_D_PHI_DEGREES: f64 = 70.0;

/// Runtime configuration after CLI/front-door flags have been resolved.
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeConfig {
    /// Requested frame rate. A windowed scene overrides this to 30.
    pub fps: u32,
    /// Whether an interactive window/Studio preview is attached.
    pub windowed: bool,
    /// Initial `-s` status.
    pub skip_animations: bool,
    /// First play index to render (`-n START`).
    pub start_at_play: Option<u64>,
    /// Exclusive play index at which to raise [`EndScene`] (`-n START,END`).
    pub end_at_play: Option<u64>,
    /// Force one current-state preview at skipped segment boundaries.
    pub preview_while_skipping: bool,
    /// Presenter waits are driven by the installed [`HoldController`].
    pub presenter_mode: bool,
    /// Maximum host payloads awaiting one serial Scene boundary.
    ///
    /// The limit is shared by every clone of [`EventInbox`]. Zero explicitly
    /// disables host event ingress without affecting queued replay events.
    pub max_pending_events: usize,
    /// `wait(None)` duration.
    pub default_wait_time: f64,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            fps: DEFAULT_FPS,
            windowed: false,
            skip_animations: false,
            start_at_play: None,
            end_at_play: None,
            preview_while_skipping: true,
            presenter_mode: false,
            max_pending_events: DEFAULT_MAX_PENDING_EVENTS,
            default_wait_time: DEFAULT_WAIT_TIME,
        }
    }
}

impl RuntimeConfig {
    /// Build the semantic defaults from the resolved project configuration.
    ///
    /// CLI-only state (`windowed`, skip, range, presenter) remains explicit on
    /// this type and defaults off.
    #[must_use]
    pub fn from_config(config: &fmn_config::Config) -> Self {
        Self {
            fps: config.camera.fps,
            preview_while_skipping: config.scene.preview_while_skipping,
            default_wait_time: config.scene.default_wait_time,
            ..Self::default()
        }
    }

    /// Frame rate selected by the native runtime, including the windowed override.
    #[must_use]
    pub fn effective_fps(&self) -> u32 {
        if self.windowed { 30 } else { self.fps }
    }

    fn validate(&self) -> Result<(), SceneError> {
        if self.effective_fps() == 0 {
            return Err(SceneError::InvalidConfig(
                "fps must be nonzero (unless windowed, which fixes it at 30)",
            ));
        }
        if !self.default_wait_time.is_finite() || self.default_wait_time < 0.0 {
            return Err(SceneError::InvalidConfig(
                "default_wait_time must be finite and non-negative",
            ));
        }
        if let (Some(start), Some(end)) = (self.start_at_play, self.end_at_play)
            && end < start
        {
            return Err(SceneError::InvalidConfig(
                "end_at_play must not precede start_at_play",
            ));
        }
        Ok(())
    }
}

/// Play-level timing overrides. `None` preserves each animation's own value.
#[derive(Clone, Default)]
pub struct PlayOverrides {
    /// Override every member's run time.
    pub run_time: Option<f64>,
    /// Override every member's rate function.
    pub rate_func: Option<RateFunc>,
    /// Override every member's lag ratio.
    pub lag_ratio: Option<f64>,
}

impl fmt::Debug for PlayOverrides {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PlayOverrides")
            .field("run_time", &self.run_time)
            .field("has_rate_func", &self.rate_func.is_some())
            .field("lag_ratio", &self.lag_ratio)
            .finish()
    }
}

/// A named integration boundary failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrationError {
    point: &'static str,
    message: String,
}

impl IntegrationError {
    /// Construct an error at a stable integration point (`sink`, `preflight`,
    /// `presenter`, …).
    #[must_use]
    pub fn new(point: &'static str, message: impl Into<String>) -> Self {
        Self {
            point,
            message: message.into(),
        }
    }

    /// Stable machine-readable integration point.
    #[must_use]
    pub fn point(&self) -> &'static str {
        self.point
    }

    /// Human-readable detail supplied by the adapter.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for IntegrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} integration failed: {}", self.point, self.message)
    }
}

impl std::error::Error for IntegrationError {}

/// The Rust spelling of the Reference's control-flow exception.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndScene;

impl fmt::Display for EndScene {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("scene ended")
    }
}

impl std::error::Error for EndScene {}

/// Errors surfaced by the Scene state machine.
#[derive(Debug)]
pub enum SceneError {
    /// Invalid semantic configuration.
    InvalidConfig(&'static str),
    /// An operation is illegal in the current lifecycle state.
    InvalidLifecycle(&'static str),
    /// A restored state cannot be represented on this scene's clock.
    InvalidState(&'static str),
    /// A durable restore contains updater identities whose callables have not
    /// yet been explicitly rebound.
    UnboundUpdaters,
    /// Marionette ownership/family failure.
    Stage(StageError),
    /// Choreo lifecycle/clock failure.
    Animation(AnimError),
    /// Lumen camera construction failure.
    Camera(CameraError),
    /// Durable-state decode failure.
    Persist(PersistError),
    /// Durable-state encode failure.
    Serialize(SerialError),
    /// Typed input validation/replay failure.
    Event(EventError),
    /// Scribe/Reel/Studio/presenter adapter failure.
    Integration(IntegrationError),
    /// Normal early scene termination, caught by [`Scene::run`].
    EndScene(EndScene),
}

impl fmt::Display for SceneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig(message) => write!(f, "invalid scene configuration: {message}"),
            Self::InvalidLifecycle(message) => write!(f, "invalid scene lifecycle: {message}"),
            Self::InvalidState(message) => write!(f, "invalid scene state: {message}"),
            Self::UnboundUpdaters => f.write_str(
                "decoded scene state carries updater identities whose callables are not rebound",
            ),
            Self::Stage(error) => write!(f, "scene stage failed: {error}"),
            Self::Animation(error) => write!(f, "scene animation failed: {error}"),
            Self::Camera(error) => write!(f, "scene camera failed: {error}"),
            Self::Persist(error) => write!(f, "scene-state decode failed: {error}"),
            Self::Serialize(error) => write!(f, "scene-state encode failed: {error}"),
            Self::Event(error) => write!(f, "scene event failed: {error}"),
            Self::Integration(error) => error.fmt(f),
            Self::EndScene(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for SceneError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Stage(error) => Some(error),
            Self::Animation(error) => Some(error),
            Self::Camera(error) => Some(error),
            Self::Persist(error) => Some(error),
            Self::Serialize(error) => Some(error),
            Self::Event(error) => Some(error),
            Self::Integration(error) => Some(error),
            Self::EndScene(error) => Some(error),
            Self::InvalidConfig(_)
            | Self::InvalidLifecycle(_)
            | Self::InvalidState(_)
            | Self::UnboundUpdaters => None,
        }
    }
}

impl From<StageError> for SceneError {
    fn from(error: StageError) -> Self {
        Self::Stage(error)
    }
}

impl From<AnimError> for SceneError {
    fn from(error: AnimError) -> Self {
        Self::Animation(error)
    }
}

impl From<CameraError> for SceneError {
    fn from(error: CameraError) -> Self {
        Self::Camera(error)
    }
}

impl From<PersistError> for SceneError {
    fn from(error: PersistError) -> Self {
        Self::Persist(error)
    }
}

impl From<SerialError> for SceneError {
    fn from(error: SerialError) -> Self {
        Self::Serialize(error)
    }
}

impl From<EventError> for SceneError {
    fn from(error: EventError) -> Self {
        Self::Event(error)
    }
}

impl From<IntegrationError> for SceneError {
    fn from(error: IntegrationError) -> Self {
        Self::Integration(error)
    }
}

impl From<EndScene> for SceneError {
    fn from(error: EndScene) -> Self {
        Self::EndScene(error)
    }
}

/// Why an immutable packet crossed the Scene boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureReason {
    /// An ordinary sampled play/wait frame.
    Segment,
    /// An explicit [`Scene::show`] request.
    Show,
    /// The configured windowed preview at the end of a skipped segment.
    SkippedPreview,
    /// A frame emitted at a live idle boundary or while presenter mode holds.
    PresenterHold,
}

/// A structured Scene lifecycle phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecyclePhase {
    /// `Scene::run` began.
    SceneBegin,
    /// The program's setup hook is about to run.
    Setup,
    /// The program's construct hook is about to run.
    Construct,
    /// The program's interactive tail is about to run.
    Interact,
    /// The program's tear-down hook is about to run.
    TearDown,
    /// `Scene::run` completed.
    SceneEnd,
    /// The one-shot constructed-scene preflight completed.
    Preflight,
    /// Range/skip/presenter preparation completed.
    PrePlay,
    /// Choreo is about to begin/progress/finish the segment.
    DriveSegment,
    /// Choreo completed the segment.
    FinishSegment,
    /// Writer/preview bookkeeping is running.
    PostPlay,
    /// A presenter hold loop began.
    HoldBegin,
    /// A presenter hold loop released.
    HoldEnd,
}

/// Stable structured metadata for a lifecycle phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LifecycleEvent {
    /// Phase.
    pub phase: LifecyclePhase,
    /// Current completed-play count (the index about to run during pre-play).
    pub play_index: u64,
    /// Exact scene time.
    pub time: RationalTime,
    /// Segment kind when the phase belongs to a play/wait.
    pub segment: Option<SegmentKind>,
    /// Effective skip status at this point.
    pub skipping: bool,
}

/// One insertion-ordered request from `Scene.add_sound`.
///
/// Reel converts [`Self::time`] from the rational frame grid to its output
/// sample grid exactly, then applies [`Self::time_offset`] once. Keeping those
/// values separate prevents float accumulation from entering A/V placement
/// (BN-02, BN-14).
#[derive(Debug, Clone, PartialEq)]
pub struct SoundRequest {
    /// User asset path; the composition root resolves/decodes it.
    pub sound_file: PathBuf,
    /// Exact Scene time at the call site.
    pub time: RationalTime,
    /// Reference-compatible relative offset in seconds.
    pub time_offset: f64,
    /// Foreground gain in dB.
    pub gain: Option<f64>,
    /// Existing-background gain in dB during the overlay.
    pub gain_to_background: Option<f64>,
}

/// The lower-boundary adapter implemented by the CLI/Studio composition root.
///
/// `capture` receives immutable front-end state, not pixels. An implementation
/// synchronizes Lumen from the packet, renders a frame, then submits that frame
/// to Reel's ordered emitter. This preserves the downward crate DAG.
pub trait SceneSink {
    /// Observe a structured lifecycle event.
    fn event(&mut self, event: LifecycleEvent) -> Result<(), IntegrationError> {
        let _ = event;
        Ok(())
    }

    /// Consume one immutable capture.
    fn capture(
        &mut self,
        reason: CaptureReason,
        packet: FramePacket,
    ) -> Result<(), IntegrationError>;
}

/// A headless sink that discards captures and events.
#[derive(Debug, Default)]
pub struct NullSceneSink;

impl SceneSink for NullSceneSink {
    fn capture(
        &mut self,
        _reason: CaptureReason,
        _packet: FramePacket,
    ) -> Result<(), IntegrationError> {
        Ok(())
    }
}

trait PreflightHook {
    fn preflight(&mut self, stage: &Stage, anchors: &[Mob]) -> Result<(), IntegrationError>;
}

impl<F> PreflightHook for F
where
    F: FnMut(&Stage, &[Mob]) -> Result<(), IntegrationError>,
{
    fn preflight(&mut self, stage: &Stage, anchors: &[Mob]) -> Result<(), IntegrationError> {
        self(stage, anchors)
    }
}

#[derive(Debug)]
struct NoPreflight;

impl PreflightHook for NoPreflight {
    fn preflight(&mut self, _stage: &Stage, _roots: &[Mob]) -> Result<(), IntegrationError> {
        Ok(())
    }
}

/// Which presenter pause is being driven.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoldKind {
    /// The Reference's first-pre-play presenter pause.
    Initial,
    /// A presenter-mode `wait`.
    Wait,
}

/// One deterministic poll result from a presenter host.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoldDecision {
    /// Advance exactly one rational-clock frame, capture, and poll again.
    Continue,
    /// Release this hold.
    Release,
    /// Terminate the scene normally.
    EndScene,
}

/// Host input for presenter mode.
///
/// The callback is polled only in the serial front-end, before each hold frame.
/// A Studio/window implementation folds its queued input into this decision;
/// tests can provide a counter. The headless default releases immediately, so
/// accidentally enabling presenter mode in a batch cannot hang forever.
pub trait HoldController {
    /// Decide whether the current hold advances, releases, or ends the scene.
    fn poll(
        &mut self,
        kind: HoldKind,
        stage: &Stage,
        time: RationalTime,
    ) -> Result<HoldDecision, IntegrationError>;
}

impl<F> HoldController for F
where
    F: FnMut(HoldKind, &Stage, RationalTime) -> Result<HoldDecision, IntegrationError>,
{
    fn poll(
        &mut self,
        kind: HoldKind,
        stage: &Stage,
        time: RationalTime,
    ) -> Result<HoldDecision, IntegrationError> {
        self(kind, stage, time)
    }
}

#[derive(Debug)]
struct ImmediateRelease;

impl HoldController for ImmediateRelease {
    fn poll(
        &mut self,
        _kind: HoldKind,
        _stage: &Stage,
        _time: RationalTime,
    ) -> Result<HoldDecision, IntegrationError> {
        Ok(HoldDecision::Release)
    }
}

/// A Scene program's lifecycle hooks.
///
/// Passing the sink explicitly keeps the program able to call `play`, `wait`,
/// and `show` without storing a self-referential adapter inside [`Scene`].
pub trait SceneProgram {
    /// Stable scene name used by output naming and registry surfaces.
    fn name(&self) -> &str {
        "Scene"
    }

    /// Common subclass setup.
    fn setup(&mut self, _scene: &mut Scene, _sink: &mut dyn SceneSink) -> Result<(), SceneError> {
        Ok(())
    }

    /// Where scene construction and playback occur.
    fn construct(&mut self, scene: &mut Scene, sink: &mut dyn SceneSink) -> Result<(), SceneError>;

    /// Interactive tail; W9 events/Studio override this later.
    fn interact(
        &mut self,
        _scene: &mut Scene,
        _sink: &mut dyn SceneSink,
    ) -> Result<(), SceneError> {
        Ok(())
    }

    /// Common subclass teardown.
    fn tear_down(
        &mut self,
        _scene: &mut Scene,
        _sink: &mut dyn SceneSink,
    ) -> Result<(), SceneError> {
        Ok(())
    }
}

/// Summary of one [`Scene::run`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SceneRunReport {
    /// Whether [`EndScene`] ended construct/interact early.
    pub ended_early: bool,
    /// Completed play/wait calls.
    pub play_count: u64,
    /// Exact final time.
    pub time: RationalTime,
}

/// Result metadata from a durable state restore.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneStateRestore {
    /// Identities whose callables durable bytes cannot contain.
    pub updaters: UpdaterManifest,
}

enum QueuedEvent {
    Live {
        sequence: u64,
        payload: EventPayload,
    },
    Replay(InputEvent),
}

/// The Scene state machine.
pub struct Scene {
    idle_owner: std::rc::Rc<()>,
    idle_sequence: u64,
    config: RuntimeConfig,
    stage: Stage,
    clock: RationalFrameClock,
    rng_root: RngRoot,
    scene_rng: Pcg64Dxsm,
    play_count: u64,
    skipping: bool,
    original_skipping: bool,
    preflight_done: bool,
    preflight: Box<dyn PreflightHook>,
    hold_controller: Box<dyn HoldController>,
    unbound_updaters: Option<UpdaterManifest>,
    sound_requests: Vec<SoundRequest>,
    event_dispatcher: EventDispatcher,
    event_inbox: EventInbox,
    queued_events: VecDeque<QueuedEvent>,
    recorded_events: Vec<InputEvent>,
    next_event_sequence: Option<u64>,
    event_error: Option<EventError>,
}

/// One play whose frame loop is intentionally split around the step-4
/// host-language updater boundary.
///
/// The session owns its animations and Choreo cursor while [`Scene`] keeps
/// the arena, clock, journal, and sink lifecycle. Front ends call
/// [`Scene::prepare_stepped_play_frame`], release their Scene borrow to run
/// callbacks, then call [`Scene::complete_stepped_play_frame`] for the same
/// frame. Ordinary Rust scenes continue to use [`Scene::play`].
pub struct SteppedPlay {
    animations: Vec<Box<dyn Animation>>,
    open: OpenSegment,
    sink_error: Option<IntegrationError>,
}

/// One wait split around the same host-language scene-updater boundary as
/// [`SteppedPlay`].
pub struct SteppedWait {
    open: OpenWait,
    sink_error: Option<IntegrationError>,
}

impl SteppedPlay {
    /// The unborrowed host completed all scene updater work for this frame.
    /// Native completion still drains events and captures exactly once.
    ///
    /// # Errors
    /// No prepared frame exists, or its updater phase is already complete.
    pub fn mark_scene_updaters_complete(&mut self) -> Result<(), SceneError> {
        self.open.mark_scene_updaters_complete().map_err(Into::into)
    }

    /// The host completed the final zero-dt phase after animation cleanup.
    ///
    /// # Errors
    /// Cleanup is incomplete, a frame is pending, or this phase is complete.
    pub fn mark_final_scene_updaters_complete(&mut self) -> Result<(), SceneError> {
        self.open
            .mark_final_scene_updaters_complete()
            .map_err(Into::into)
    }
}

impl SteppedWait {
    /// The unborrowed host completed all scene updater work for this frame.
    ///
    /// # Errors
    /// No prepared frame exists, or its updater phase is already complete.
    pub fn mark_scene_updaters_complete(&mut self) -> Result<(), SceneError> {
        self.open.mark_scene_updaters_complete().map_err(Into::into)
    }
}

impl fmt::Debug for SteppedWait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SteppedWait")
            .field("open", &self.open)
            .field("has_sink_error", &self.sink_error.is_some())
            .finish()
    }
}

impl fmt::Debug for SteppedPlay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SteppedPlay")
            .field("animation_count", &self.animations.len())
            .field("stepped", &self.open.stepped())
            .field("frames", &self.open.segment().n_frames())
            .field("has_sink_error", &self.sink_error.is_some())
            .finish()
    }
}

impl fmt::Debug for Scene {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Scene")
            .field("config", &self.config)
            .field("time", &self.clock.now())
            .field("play_count", &self.play_count)
            .field("skipping", &self.skipping)
            .field("preflight_done", &self.preflight_done)
            .field("has_unbound_updaters", &self.unbound_updaters.is_some())
            .field("sound_requests", &self.sound_requests.len())
            .field("event_listeners", &self.event_dispatcher.listener_count())
            .field("queued_events", &self.queued_events.len())
            .field("recorded_events", &self.recorded_events.len())
            .finish_non_exhaustive()
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new(RuntimeConfig::default(), 0).expect("default scene config is valid")
    }
}

impl Scene {
    /// Construct a scene from resolved runtime semantics and a deterministic
    /// root seed.
    pub fn new(config: RuntimeConfig, seed: u64) -> Result<Self, SceneError> {
        config.validate()?;
        let fps = config.effective_fps();
        let rng_root = RngRoot::from_seed(seed);
        let scene_rng = rng_root.substream("scene").sequential();
        let original_skipping = config.skip_animations;
        let skipping = original_skipping || config.start_at_play.is_some();
        let event_inbox = EventInbox::new(config.max_pending_events)?;
        Ok(Self {
            idle_owner: std::rc::Rc::new(()),
            idle_sequence: 0,
            config,
            stage: Stage::new(),
            clock: RationalFrameClock::new(fps).map_err(AnimError::Clock)?,
            rng_root,
            scene_rng,
            play_count: 0,
            skipping,
            original_skipping,
            preflight_done: false,
            preflight: Box::new(NoPreflight),
            hold_controller: Box::new(ImmediateRelease),
            unbound_updaters: None,
            sound_requests: Vec::new(),
            event_dispatcher: EventDispatcher::new(),
            event_inbox,
            queued_events: VecDeque::new(),
            recorded_events: Vec::new(),
            next_event_sequence: Some(0),
            event_error: None,
        })
    }

    /// Resolved runtime config. `fps()` is the effective value after the
    /// windowed override.
    #[must_use]
    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    /// Effective frame rate.
    #[must_use]
    pub fn fps(&self) -> u32 {
        self.clock.fps()
    }

    /// Exact scene time.
    #[must_use]
    pub fn time(&self) -> RationalTime {
        self.clock.now()
    }

    /// Completed segment count.
    #[must_use]
    pub fn play_count(&self) -> u64 {
        self.play_count
    }

    /// Current effective skip status.
    #[must_use]
    pub fn is_skipping(&self) -> bool {
        self.skipping
    }

    /// Scene arena.
    #[must_use]
    pub fn stage(&self) -> &Stage {
        &self.stage
    }

    /// Mutable scene arena for construction and direct mobject operations.
    pub fn stage_mut(&mut self) -> &mut Stage {
        &mut self.stage
    }

    /// The scene-serial named RNG substream. Frame work must use
    /// [`FramePacket::rng_fork`] instead.
    pub fn rng_mut(&mut self) -> &mut Pcg64Dxsm {
        &mut self.scene_rng
    }

    /// Deterministic input dispatcher. Listeners should be installed during
    /// construction, before the event stream they consume is queued.
    #[must_use]
    pub fn event_dispatcher(&self) -> &EventDispatcher {
        &self.event_dispatcher
    }

    /// Mutable input dispatcher for listener registration/removal.
    pub fn event_dispatcher_mut(&mut self) -> &mut EventDispatcher {
        &mut self.event_dispatcher
    }

    /// Cloneable Studio/browser/TUI submission handle.
    ///
    /// Payloads arriving through this inbox are assigned sequence ids and exact
    /// rational timestamps only when the Scene drains them at a serial
    /// pre-capture boundary.
    #[must_use]
    pub fn event_inbox(&self) -> EventInbox {
        self.event_inbox.clone()
    }

    /// Queue one already-available live event for the next serial boundary.
    ///
    /// Returns the stable sequence id reserved for it.
    pub fn queue_event(&mut self, payload: EventPayload) -> Result<u64, EventError> {
        let canonical = InputEvent::new(0, RationalTime::zero(self.fps()), payload)?.payload;
        let sequence = self.allocate_event_sequence()?;
        self.queued_events.push_back(QueuedEvent::Live {
            sequence,
            payload: canonical,
        });
        Ok(sequence)
    }

    /// Queue a complete recorded stream for deterministic timed replay.
    ///
    /// Replay starts from an empty input queue/inbox, uses the Scene's exact fps,
    /// and requires nondecreasing timestamps plus globally increasing sequence
    /// ids. Events become eligible at the first serial boundary whose clock
    /// time reaches their timestamp.
    pub fn queue_replay_events(&mut self, events: &[InputEvent]) -> Result<(), EventError> {
        if !self.queued_events.is_empty() || !self.event_inbox.is_empty() {
            return Err(EventError::ReplayOutOfOrder);
        }
        let mut canonical = Vec::with_capacity(events.len());
        let mut previous_timestamp = None;
        let mut candidate_sequence = self.next_event_sequence;
        for event in events {
            let event = InputEvent::new(event.sequence, event.timestamp, event.payload.clone())?;
            if event.timestamp.fps() != self.fps() {
                return Err(EventError::TimestampGridMismatch {
                    expected: self.fps(),
                    found: event.timestamp.fps(),
                });
            }
            if previous_timestamp.is_some_and(|prior| event.timestamp < prior) {
                return Err(EventError::ReplayOutOfOrder);
            }
            let minimum_sequence = candidate_sequence.ok_or(EventError::SequenceExhausted)?;
            if event.sequence < minimum_sequence {
                return Err(EventError::ReplayOutOfOrder);
            }
            candidate_sequence = event.sequence.checked_add(1);
            previous_timestamp = Some(event.timestamp);
            canonical.push(event);
        }
        self.next_event_sequence = candidate_sequence;
        for event in canonical {
            self.queued_events.push_back(QueuedEvent::Replay(event));
        }
        Ok(())
    }

    /// Drain all inputs eligible at the current rational-clock instant.
    ///
    /// This is the explicit paused/manual serial boundary. `play`, `wait`,
    /// `show`, skipped previews, and presenter holds call the same machinery.
    pub fn dispatch_pending_events(&mut self) -> Result<usize, EventError> {
        let time = self.clock.now();
        let dispatched = drain_event_queue(
            &mut self.event_dispatcher,
            &self.event_inbox,
            &mut self.queued_events,
            &mut self.recorded_events,
            &mut self.next_event_sequence,
            &mut self.event_error,
            &mut self.stage,
            time,
        );
        match self.event_error.take() {
            Some(error) => Err(error),
            None => Ok(dispatched),
        }
    }

    pub fn seek_interactive_frame(&mut self, frame: i64) -> Result<(), SceneError> {
        self.ensure_ready()?;
        if frame < 0 {
            return Err(SceneError::InvalidState(
                "interactive frame must be non-negative",
            ));
        }
        let mut clock = RationalFrameClock::new(self.fps()).map_err(AnimError::Clock)?;
        clock.advance_frames(frame).map_err(AnimError::Clock)?;
        // A discontinuous seek supersedes a yielded live frame even when the
        // caller seeks back to precisely the same rational time afterwards.
        self.idle_owner = std::rc::Rc::new(());
        self.clock = clock;
        self.sync_stage_time();
        Ok(())
    }

    /// Events in the exact order and at the exact times they were dispatched.
    #[must_use]
    pub fn recorded_events(&self) -> &[InputEvent] {
        &self.recorded_events
    }

    /// Move the current event record into a journal writer.
    pub fn take_recorded_events(&mut self) -> Vec<InputEvent> {
        std::mem::take(&mut self.recorded_events)
    }

    /// Install Scribe's constructed-scene preflight.
    ///
    /// The anchors are every current scene root plus any otherwise-unrooted
    /// mobjects in the first play's animation closure. This ensures a static
    /// typeset target cannot first appear after the one-shot walk.
    ///
    /// It must be installed before the first segment/show so an adapter cannot
    /// silently miss frame zero.
    pub fn set_preflight_hook(
        &mut self,
        hook: impl FnMut(&Stage, &[Mob]) -> Result<(), IntegrationError> + 'static,
    ) -> Result<(), SceneError> {
        if self.preflight_done {
            return Err(SceneError::InvalidLifecycle(
                "preflight hook cannot change after preflight has run",
            ));
        }
        self.preflight = Box::new(hook);
        Ok(())
    }

    /// Install the presenter/window event adapter.
    pub fn set_hold_controller(&mut self, controller: impl HoldController + 'static) {
        self.hold_controller = Box::new(controller);
    }

    /// Add a detached mobject to the arena and root it in one operation.
    pub fn add_mobject(&mut self, mobject: impl Into<Mobject>) -> Result<Mob, SceneError> {
        let mob = self.stage.add(mobject);
        self.stage.add_to_scene(mob)?;
        Ok(mob)
    }

    /// Root live handles using Marionette's stable `(z_index, position)` rule.
    pub fn add(&mut self, mobs: &[Mob]) -> Result<&mut Self, SceneError> {
        self.stage.add_many_to_scene(mobs)?;
        Ok(self)
    }

    /// Remove handles from the draw list with rooted-family splice semantics.
    pub fn remove(&mut self, mobs: &[Mob]) -> &mut Self {
        self.stage.remove_many_from_scene(mobs);
        self
    }

    /// Re-add handles, which is exactly the Reference's promotion operation.
    pub fn bring_to_front(&mut self, mobs: &[Mob]) -> Result<&mut Self, SceneError> {
        self.add(mobs)
    }

    /// Prepend handles without re-sorting.
    pub fn bring_to_back(&mut self, mobs: &[Mob]) -> Result<&mut Self, SceneError> {
        self.stage.bring_many_to_back(mobs)?;
        Ok(self)
    }

    /// Splice replacements into one exact top-level member's draw-list slot.
    pub fn replace(&mut self, mob: Mob, replacements: &[Mob]) -> Result<&mut Self, SceneError> {
        self.stage.replace_in_scene(mob, replacements)?;
        Ok(self)
    }

    /// Replace the complete draw list with one batch using ordinary add order.
    ///
    /// This is the Reference's `remove_all_except`: despite the name, handles
    /// need not already be scene roots. Validate the complete batch before
    /// clearing so a stale native handle cannot destroy the current scene.
    pub fn remove_all_except(&mut self, mobs: &[Mob]) -> Result<&mut Self, SceneError> {
        if mobs.iter().any(|&mob| !self.stage.contains(mob)) {
            return Err(StageError::StaleHandle.into());
        }
        self.clear();
        self.add(mobs)
    }

    /// Clear the scene draw list without deleting arena entries.
    pub fn clear(&mut self) -> &mut Self {
        for mob in self.stage.roots().to_vec() {
            self.stage.remove_from_scene(mob);
        }
        self
    }

    /// The current top-level draw list.
    #[must_use]
    pub fn mobjects(&self) -> &[Mob] {
        self.stage.roots()
    }

    /// Insertion-ordered sound requests for Reel's end-of-scene mix.
    #[must_use]
    pub fn sound_requests(&self) -> &[SoundRequest] {
        &self.sound_requests
    }

    /// Queue a sound with the Reference's four arguments.
    ///
    /// `gain` and `gain_to_background` are dB. Skip mode is an early no-op,
    /// matching the Reference. The rational call-site time remains separate
    /// from the relative offset so Reel can place frame time with exact integer
    /// arithmetic; a resulting pre-zero start is clipped rather than rejected
    /// (BN-14).
    ///
    /// # Errors
    /// A non-finite offset or gain is rejected before it can enter output
    /// provenance. Skip mode returns before validation, matching the
    /// Reference's early-return semantics.
    pub fn add_sound(
        &mut self,
        sound_file: impl Into<PathBuf>,
        time_offset: f64,
        gain: Option<f64>,
        gain_to_background: Option<f64>,
    ) -> Result<&mut Self, SceneError> {
        if self.skipping {
            return Ok(self);
        }
        if !time_offset.is_finite() {
            return Err(SceneError::InvalidConfig(
                "sound time_offset must be finite",
            ));
        }
        if gain.is_some_and(|value| !value.is_finite()) {
            return Err(SceneError::InvalidConfig("sound gain must be finite"));
        }
        if gain_to_background.is_some_and(|value| !value.is_finite()) {
            return Err(SceneError::InvalidConfig(
                "sound gain_to_background must be finite",
            ));
        }
        self.sound_requests.push(SoundRequest {
            sound_file: sound_file.into(),
            time: self.clock.now(),
            time_offset,
            gain,
            gain_to_background,
        });
        Ok(self)
    }

    /// Begin a play whose scene-updater phase is driven through an explicit
    /// borrow-release window.
    ///
    /// This performs the same overrides, validation, preflight, lifecycle
    /// events, and animation prologue as [`Self::play`], but stops before the
    /// first frame. An empty animation list remains a warning-only no-op,
    /// represented as `Ok(None)`.
    pub fn begin_stepped_play(
        &mut self,
        mut animations: Vec<Box<dyn Animation>>,
        overrides: PlayOverrides,
        sink: &mut dyn SceneSink,
    ) -> Result<Option<SteppedPlay>, SceneError> {
        if animations.is_empty() {
            return Ok(None);
        }
        for animation in &mut animations {
            animation.update_rate_info(
                overrides.run_time,
                overrides.rate_func.clone(),
                overrides.lag_ratio,
            );
        }
        validate_play(&self.stage, &self.clock, &animations)?;
        let animation_anchors: Vec<Mob> = animations
            .iter()
            .flat_map(|animation| animation.preflight_mobjects())
            .collect();
        self.pre_play(SegmentKind::Play, &animation_anchors, sink)?;
        self.emit_event(sink, LifecyclePhase::DriveSegment, Some(SegmentKind::Play))?;
        let open =
            open_play_with_mode(&mut self.stage, &self.clock, &mut animations, self.skipping)?;
        Ok(Some(SteppedPlay {
            animations,
            open,
            sink_error: None,
        }))
    }

    /// Advance a stepped play through animation interpolation and rational
    /// clock step 3, then yield before any scene updater runs.
    ///
    /// The caller must release its Scene borrow, execute host-language scene
    /// updaters using the returned `dt`, and pair a successful boundary with
    /// exactly one [`Self::complete_stepped_play_frame`] call.
    pub fn prepare_stepped_play_frame(
        &mut self,
        play: &mut SteppedPlay,
    ) -> Result<Option<SceneUpdaterBoundary>, SceneError> {
        prepare_play_frame(
            &mut self.stage,
            &mut self.clock,
            &mut play.animations,
            &mut play.open,
        )
        .map_err(Into::into)
    }

    /// Run one top-level animation's step-1/2 pair and yield before the next.
    ///
    /// Host-language animation slots use this earlier release point for
    /// callbacks whose mutations must be visible to subsequent animations
    /// and to the later scene-updater phase.
    pub fn prepare_stepped_play_animation(
        &mut self,
        play: &mut SteppedPlay,
    ) -> Result<Option<AnimationBoundary>, SceneError> {
        prepare_play_animation(
            &mut self.stage,
            &self.clock,
            &mut play.animations,
            &mut play.open,
        )
        .map_err(Into::into)
    }

    /// Complete step 4 and capture one frame previously returned by
    /// [`Self::prepare_stepped_play_frame`].
    ///
    /// Sink failures are retained and surfaced after deterministic segment
    /// completion, matching [`Self::play`].
    pub fn complete_stepped_play_frame(
        &mut self,
        play: &mut SteppedPlay,
        sink: &mut dyn SceneSink,
    ) -> Result<(), SceneError> {
        let capture = !self.skipping;
        let Scene {
            stage,
            clock,
            rng_root,
            event_dispatcher,
            event_inbox,
            queued_events,
            recorded_events,
            next_event_sequence,
            event_error,
            ..
        } = self;
        let mut boundary = |stage: &mut Stage, time: RationalTime| {
            drain_event_queue(
                event_dispatcher,
                event_inbox,
                queued_events,
                recorded_events,
                next_event_sequence,
                event_error,
                stage,
                time,
            );
        };
        let mut emit = |packet| {
            if play.sink_error.is_none()
                && let Err(error) = sink.capture(CaptureReason::Segment, packet)
            {
                play.sink_error = Some(error);
            }
        };
        complete_play_frame(
            stage,
            clock,
            rng_root,
            &mut play.open,
            capture,
            &mut boundary,
            &mut emit,
        )?;
        Ok(())
    }

    /// Finish animation lifecycles while retaining the stepped-play handle.
    ///
    /// A host-language portal calls this after the last frame and before its
    /// zero-`dt` updater dispatch.  In particular, this resumes mobjects whose
    /// animation requested updater suspension.  [`Self::finish_stepped_play`]
    /// then runs the native updater half and consumes the play normally.
    pub fn finish_stepped_play_animations(
        &mut self,
        play: &mut SteppedPlay,
    ) -> Result<Vec<Mob>, SceneError> {
        let mut resumed = Vec::new();
        for animation in &play.animations {
            animation.collect_resumed_updater_mobjects(&mut resumed);
        }
        finish_open_play_animations(&mut self.stage, &mut play.animations, &mut play.open)?;
        Ok(resumed)
    }

    /// Finish a stepped play through the ordinary animation cleanup,
    /// zero-dt updater pass, lifecycle notifications, and play-count update.
    pub fn finish_stepped_play(
        &mut self,
        mut play: SteppedPlay,
        sink: &mut dyn SceneSink,
    ) -> Result<SegmentReport, SceneError> {
        let report = finish_open_play(&mut self.stage, &mut play.animations, play.open)?;
        self.sync_stage_time();
        let event_error = self.event_error.take();
        let finish_result = if play.sink_error.is_none() && event_error.is_none() {
            self.emit_event(sink, LifecyclePhase::FinishSegment, Some(SegmentKind::Play))
        } else {
            Ok(())
        };
        let post_result = self.post_play(
            SegmentKind::Play,
            sink,
            play.sink_error.is_none() && event_error.is_none(),
        );
        if let Some(error) = play.sink_error {
            return Err(error.into());
        }
        if let Some(error) = event_error {
            return Err(error.into());
        }
        finish_result?;
        post_result?;
        Ok(report)
    }

    /// Consume a stepped play after a host-language callback exception.
    ///
    /// The original exception remains the caller's primary error. This
    /// infallible cleanup clears animation lifecycle state and advances the
    /// play counter without publishing successful finish/post-play events.
    pub fn abort_stepped_play(&mut self, mut play: SteppedPlay, sink: &mut dyn SceneSink) {
        let _report = abort_open_play(&mut self.stage, &mut play.animations, play.open);
        self.sync_stage_time();
        self.event_error.take();
        self.post_play(SegmentKind::Play, sink, false)
            .expect("non-notifying post-play is infallible");
    }

    /// Drive one play through `prepare overrides → pre_play →
    /// begin/progress/finish → post_play`.
    ///
    /// An empty list is the Reference's warning-only no-op, represented as
    /// `Ok(None)`. A sink failure during capture is surfaced at the segment
    /// boundary after Choreo has deterministically finished the segment and
    /// post-play has incremented the counter; later captures are suppressed.
    pub fn play(
        &mut self,
        mut animations: Vec<Box<dyn Animation>>,
        overrides: PlayOverrides,
        sink: &mut dyn SceneSink,
    ) -> Result<Option<SegmentReport>, SceneError> {
        if animations.is_empty() {
            return Ok(None);
        }
        for animation in &mut animations {
            animation.update_rate_info(
                overrides.run_time,
                overrides.rate_func.clone(),
                overrides.lag_ratio,
            );
        }
        validate_play(&self.stage, &self.clock, &animations)?;
        let animation_anchors: Vec<Mob> = animations
            .iter()
            .flat_map(|animation| animation.preflight_mobjects())
            .collect();
        self.pre_play(SegmentKind::Play, &animation_anchors, sink)?;
        self.emit_event(sink, LifecyclePhase::DriveSegment, Some(SegmentKind::Play))?;

        let mut sink_error = None;
        let skipping = self.skipping;
        let report = {
            let Scene {
                stage,
                clock,
                rng_root,
                event_dispatcher,
                event_inbox,
                queued_events,
                recorded_events,
                next_event_sequence,
                event_error,
                ..
            } = self;
            let mut boundary = |stage: &mut Stage, time: RationalTime| {
                drain_event_queue(
                    event_dispatcher,
                    event_inbox,
                    queued_events,
                    recorded_events,
                    next_event_sequence,
                    event_error,
                    stage,
                    time,
                );
            };
            let mut emit = |packet| {
                if sink_error.is_none()
                    && let Err(error) = sink.capture(CaptureReason::Segment, packet)
                {
                    sink_error = Some(error);
                }
            };
            play_segment_with_boundary(
                stage,
                clock,
                rng_root,
                &mut animations,
                skipping,
                &mut boundary,
                &mut emit,
            )?
        };
        self.sync_stage_time();
        let event_error = self.event_error.take();

        let finish_result = if sink_error.is_none() && event_error.is_none() {
            self.emit_event(sink, LifecyclePhase::FinishSegment, Some(SegmentKind::Play))
        } else {
            Ok(())
        };
        let post_result = self.post_play(
            SegmentKind::Play,
            sink,
            sink_error.is_none() && event_error.is_none(),
        );
        if let Some(error) = sink_error {
            return Err(error.into());
        }
        if let Some(error) = event_error {
            return Err(error.into());
        }
        finish_result?;
        post_result?;
        Ok(Some(report))
    }

    /// Begin a declarative wait whose per-frame scene updater phase can run
    /// after the caller releases its Scene borrow.
    pub fn begin_stepped_wait(
        &mut self,
        duration: Option<f64>,
        sink: &mut dyn SceneSink,
    ) -> Result<SteppedWait, SceneError> {
        self.begin_stepped_wait_with_updates(duration, sink, false)
    }

    /// Begin after the unborrowed host completed the whole initial zero-dt
    /// scene-updater phase, including native slots. Do not run them again.
    pub fn begin_stepped_wait_after_updaters(
        &mut self,
        duration: Option<f64>,
        sink: &mut dyn SceneSink,
    ) -> Result<SteppedWait, SceneError> {
        self.begin_stepped_wait_with_updates(duration, sink, true)
    }

    fn begin_stepped_wait_with_updates(
        &mut self,
        duration: Option<f64>,
        sink: &mut dyn SceneSink,
        updaters_completed: bool,
    ) -> Result<SteppedWait, SceneError> {
        let duration = duration.unwrap_or(self.config.default_wait_time);
        if !duration.is_finite() || duration < 0.0 {
            return Err(SceneError::InvalidConfig(
                "wait duration must be finite and non-negative",
            ));
        }
        self.clock.segment(duration).map_err(AnimError::Clock)?;
        self.pre_play(SegmentKind::Wait, &[], sink)?;
        self.emit_event(sink, LifecyclePhase::DriveSegment, Some(SegmentKind::Wait))?;
        let open = if updaters_completed {
            open_wait_after_updaters(&mut self.stage, &self.clock, duration, self.skipping)?
        } else {
            open_wait(&mut self.stage, &self.clock, duration, self.skipping)?
        };
        Ok(SteppedWait {
            open,
            sink_error: None,
        })
    }

    /// Advance a stepped wait's rational clock, yielding before scene
    /// updaters exactly like [`Self::prepare_stepped_play_frame`].
    pub fn prepare_stepped_wait_frame(
        &mut self,
        wait: &mut SteppedWait,
    ) -> Result<Option<SceneUpdaterBoundary>, SceneError> {
        prepare_wait_frame(&mut self.stage, &mut self.clock, &mut wait.open).map_err(Into::into)
    }

    /// Complete native scene updaters, input dispatch, and capture for one
    /// prepared wait frame.
    pub fn complete_stepped_wait_frame(
        &mut self,
        wait: &mut SteppedWait,
        sink: &mut dyn SceneSink,
    ) -> Result<(), SceneError> {
        let capture = !self.skipping;
        let Scene {
            stage,
            clock,
            rng_root,
            event_dispatcher,
            event_inbox,
            queued_events,
            recorded_events,
            next_event_sequence,
            event_error,
            ..
        } = self;
        let mut boundary = |stage: &mut Stage, time: RationalTime| {
            drain_event_queue(
                event_dispatcher,
                event_inbox,
                queued_events,
                recorded_events,
                next_event_sequence,
                event_error,
                stage,
                time,
            );
        };
        let mut emit = |packet| {
            if wait.sink_error.is_none()
                && let Err(error) = sink.capture(CaptureReason::Segment, packet)
            {
                wait.sink_error = Some(error);
            }
        };
        complete_wait_frame(
            stage,
            clock,
            rng_root,
            &mut wait.open,
            capture,
            &mut boundary,
            &mut emit,
        )?;
        Ok(())
    }

    /// Finish a stepped wait through the ordinary lifecycle and play-count
    /// transition.
    pub fn finish_stepped_wait(
        &mut self,
        wait: SteppedWait,
        sink: &mut dyn SceneSink,
    ) -> Result<SegmentReport, SceneError> {
        let report = finish_open_wait(wait.open)?;
        self.sync_stage_time();
        let event_error = self.event_error.take();
        let finish_result = if wait.sink_error.is_none() && event_error.is_none() {
            self.emit_event(sink, LifecyclePhase::FinishSegment, Some(SegmentKind::Wait))
        } else {
            Ok(())
        };
        let post_result = self.post_play(
            SegmentKind::Wait,
            sink,
            wait.sink_error.is_none() && event_error.is_none(),
        );
        if let Some(error) = wait.sink_error {
            return Err(error.into());
        }
        if let Some(error) = event_error {
            return Err(error.into());
        }
        finish_result?;
        post_result?;
        Ok(report)
    }

    /// Consume a stepped wait after a host-language updater exception.
    pub fn abort_stepped_wait(&mut self, wait: SteppedWait, sink: &mut dyn SceneSink) {
        let _report = abort_open_wait(wait.open);
        self.sync_stage_time();
        self.event_error.take();
        self.post_play(SegmentKind::Wait, sink, false)
            .expect("non-notifying post-play is infallible");
    }

    /// Wait for `duration`, or the configured default when `None`.
    pub fn wait(
        &mut self,
        duration: Option<f64>,
        sink: &mut dyn SceneSink,
    ) -> Result<SegmentReport, SceneError> {
        self.wait_impl(duration, None, false, sink)
    }

    /// Wait until `stop_condition` turns true, or `max_time` elapses.
    pub fn wait_until(
        &mut self,
        max_time: f64,
        stop_condition: &mut dyn FnMut(&Stage) -> bool,
        sink: &mut dyn SceneSink,
    ) -> Result<SegmentReport, SceneError> {
        self.wait_impl(Some(max_time), Some(stop_condition), false, sink)
    }

    /// Wait while explicitly bypassing presenter mode.
    pub fn wait_ignoring_presenter(
        &mut self,
        duration: Option<f64>,
        sink: &mut dyn SceneSink,
    ) -> Result<SegmentReport, SceneError> {
        self.wait_impl(duration, None, true, sink)
    }

    fn wait_impl(
        &mut self,
        duration: Option<f64>,
        stop_condition: Option<&mut dyn FnMut(&Stage) -> bool>,
        ignore_presenter: bool,
        sink: &mut dyn SceneSink,
    ) -> Result<SegmentReport, SceneError> {
        let duration = duration.unwrap_or(self.config.default_wait_time);
        if !duration.is_finite() || duration < 0.0 {
            return Err(SceneError::InvalidConfig(
                "wait duration must be finite and non-negative",
            ));
        }
        self.clock.segment(duration).map_err(AnimError::Clock)?;
        self.pre_play(SegmentKind::Wait, &[], sink)?;
        self.emit_event(sink, LifecyclePhase::DriveSegment, Some(SegmentKind::Wait))?;

        let report = if self.config.presenter_mode && !self.skipping && !ignore_presenter {
            self.presenter_wait(sink)?
        } else {
            let mut sink_error = None;
            let skipping = self.skipping;
            let report = {
                let Scene {
                    stage,
                    clock,
                    rng_root,
                    event_dispatcher,
                    event_inbox,
                    queued_events,
                    recorded_events,
                    next_event_sequence,
                    event_error,
                    ..
                } = self;
                let mut boundary = |stage: &mut Stage, time: RationalTime| {
                    drain_event_queue(
                        event_dispatcher,
                        event_inbox,
                        queued_events,
                        recorded_events,
                        next_event_sequence,
                        event_error,
                        stage,
                        time,
                    );
                };
                let mut emit = |packet| {
                    if sink_error.is_none()
                        && let Err(error) = sink.capture(CaptureReason::Segment, packet)
                    {
                        sink_error = Some(error);
                    }
                };
                wait_segment_with_boundary(
                    stage,
                    clock,
                    rng_root,
                    duration,
                    stop_condition,
                    skipping,
                    &mut boundary,
                    &mut emit,
                )?
            };
            self.sync_stage_time();
            if let Some(error) = sink_error {
                self.post_play(SegmentKind::Wait, sink, false)?;
                return Err(error.into());
            }
            if let Some(error) = self.event_error.take() {
                self.post_play(SegmentKind::Wait, sink, false)?;
                return Err(error.into());
            }
            report
        };

        let finish_result =
            self.emit_event(sink, LifecyclePhase::FinishSegment, Some(SegmentKind::Wait));
        let post_result = self.post_play(SegmentKind::Wait, sink, finish_result.is_ok());
        finish_result?;
        post_result?;
        Ok(report)
    }

    /// Force a current-state capture without advancing time.
    pub fn show(&mut self, sink: &mut dyn SceneSink) -> Result<(), SceneError> {
        self.ensure_preflight(&[], sink)?;
        for root in self.stage.roots().to_vec() {
            self.stage.update_mobject(root, 0.0);
        }
        self.sync_stage_time();
        self.dispatch_pending_events()?;
        sink.capture(
            CaptureReason::Show,
            FramePacket::freeze_barrier(&self.stage, &self.clock, &self.rng_root),
        )?;
        Ok(())
    }

    /// Capture the in-memory SceneState. Updater callables remain shared in
    /// the CoW snapshot.
    ///
    /// # Errors
    /// [`SceneError::UnboundUpdaters`] while a durable manifest is pending;
    /// capturing that callable-free intermediate state would create an
    /// apparent escape from the replay barrier.
    pub fn state(&mut self) -> Result<SceneState, SceneError> {
        self.ensure_ready()?;
        self.sync_stage_time();
        let time = self.clock.now();
        Ok(SceneState::capture(
            &self.stage,
            time.frames(),
            time.fps(),
            self.play_count,
            &self.scene_rng,
        ))
    }

    /// Capture canonical durable SceneState bytes.
    pub fn state_bytes(&mut self) -> Result<Vec<u8>, SceneError> {
        Ok(self.state()?.to_bytes()?)
    }

    /// Restore an in-memory state, including updater callables.
    pub fn restore_state(&mut self, state: &SceneState) -> Result<(), SceneError> {
        self.apply_state(
            state.frames_elapsed,
            state.fps,
            state.play_count,
            state.rng_state,
            &state.snapshot,
        )?;
        self.unbound_updaters = None;
        Ok(())
    }

    /// Decode and restore durable SceneState bytes.
    ///
    /// Updater callables are intentionally absent from the format. When the
    /// returned manifest is non-empty, playback fails closed with
    /// [`SceneError::UnboundUpdaters`] until
    /// [`Scene::rebind_updaters`] resolves every original identity.
    pub fn restore_state_bytes(&mut self, bytes: &[u8]) -> Result<SceneStateRestore, SceneError> {
        let decoded = SceneState::from_bytes(bytes, &self.stage)?;
        self.apply_state(
            decoded.frames_elapsed,
            decoded.fps,
            decoded.play_count,
            decoded.rng_state,
            &decoded.snapshot,
        )?;
        let updaters = decoded.updaters;
        self.unbound_updaters = (!updaters.entries.is_empty()).then(|| updaters.clone());
        Ok(SceneStateRestore { updaters })
    }

    /// Reinstall every callable named by a decoded updater manifest.
    ///
    /// Resolution completes before Marionette mutates any updater list. The
    /// callback kind is checked against the durable manifest and the original
    /// [`UpdaterId`] is retained, so replay cannot silently substitute a
    /// different execution order or removal token.
    ///
    /// # Errors
    /// [`SceneError::InvalidLifecycle`] when no manifest is pending,
    /// [`SceneError::InvalidState`] when a resolver returns the wrong updater
    /// kind, or the resolver's [`IntegrationError`].
    pub fn rebind_updaters(
        &mut self,
        mut resolve: impl FnMut(Mob, UpdaterId, UpdaterKindTag) -> Result<UpdaterFn, IntegrationError>,
    ) -> Result<(), SceneError> {
        let manifest = self
            .unbound_updaters
            .as_ref()
            .ok_or(SceneError::InvalidLifecycle(
                "no decoded updater manifest is pending",
            ))?;
        let identities = manifest.identities(&self.stage)?;
        let mut bindings = Vec::with_capacity(identities.len());
        for identity in identities {
            let func = resolve(identity.mob, identity.id, identity.kind)?;
            let matches = matches!(
                (identity.kind, &func),
                (UpdaterKindTag::NonDt, UpdaterFn::NonDt(_))
                    | (UpdaterKindTag::Dt, UpdaterFn::Dt(_))
            );
            if !matches {
                return Err(SceneError::InvalidState(
                    "rebound updater kind does not match the durable manifest",
                ));
            }
            bindings.push((identity, func));
        }
        self.stage.restore_updater_bindings(bindings)?;
        self.unbound_updaters = None;
        Ok(())
    }

    /// Set effective playback skipping without changing the saved revert state.
    /// Host-language front doors call this only between segments, after their
    /// range/skip hooks and before Choreo opens the next sampled segment.
    pub fn set_skipping(&mut self, skipping: bool) -> &mut Self {
        self.skipping = skipping;
        self
    }

    /// Force skip mode while remembering the status to which revert returns.
    pub fn force_skipping(&mut self) -> &mut Self {
        self.original_skipping = self.skipping;
        self.skipping = true;
        self
    }

    /// Restore the skip status remembered by [`Scene::force_skipping`].
    pub fn revert_to_original_skipping_status(&mut self) -> &mut Self {
        self.skipping = self.original_skipping;
        self
    }

    /// Produce normal scene-termination control flow.
    pub fn end<T>(&mut self) -> Result<T, SceneError> {
        Err(EndScene.into())
    }

    /// Run setup → construct → interact, catch [`EndScene`], and always run
    /// teardown/end notifications.
    pub fn run(
        &mut self,
        program: &mut dyn SceneProgram,
        sink: &mut dyn SceneSink,
    ) -> Result<SceneRunReport, SceneError> {
        let body = (|| {
            self.emit_event(sink, LifecyclePhase::SceneBegin, None)?;
            self.emit_event(sink, LifecyclePhase::Setup, None)?;
            program.setup(self, sink)?;
            self.emit_event(sink, LifecyclePhase::Construct, None)?;
            program.construct(self, sink)?;
            self.emit_event(sink, LifecyclePhase::Interact, None)?;
            program.interact(self, sink)
        })();

        let (ended_early, body_error) = match body {
            Ok(()) => (false, None),
            Err(SceneError::EndScene(_)) => (true, None),
            Err(error) => (false, Some(error)),
        };

        // Cleanup is best-effort in the strong sense: neither a lifecycle
        // adapter failure nor a program tear-down failure suppresses the
        // remaining cleanup phases. Error precedence still preserves the
        // original body failure, then the earliest cleanup failure.
        let teardown_event = self.emit_event(sink, LifecyclePhase::TearDown, None);
        let teardown_body = program.tear_down(self, sink);
        let scene_end = self.emit_event(sink, LifecyclePhase::SceneEnd, None);

        if let Some(error) = body_error {
            return Err(error);
        }
        teardown_event?;
        teardown_body?;
        scene_end?;
        Ok(SceneRunReport {
            ended_early,
            play_count: self.play_count,
            time: self.clock.now(),
        })
    }

    fn ensure_ready(&self) -> Result<(), SceneError> {
        if self.unbound_updaters.is_some() {
            Err(SceneError::UnboundUpdaters)
        } else {
            Ok(())
        }
    }

    fn allocate_event_sequence(&mut self) -> Result<u64, EventError> {
        let sequence = self
            .next_event_sequence
            .ok_or(EventError::SequenceExhausted)?;
        self.next_event_sequence = sequence.checked_add(1);
        Ok(sequence)
    }

    fn ensure_preflight(
        &mut self,
        animation_anchors: &[Mob],
        sink: &mut dyn SceneSink,
    ) -> Result<(), SceneError> {
        self.ensure_ready()?;
        if self.preflight_done {
            return Ok(());
        }
        let mut anchors = self.stage.roots().to_vec();
        let mut covered: Vec<Mob> = anchors
            .iter()
            .flat_map(|&root| self.stage.family(root))
            .collect();
        for &anchor in animation_anchors {
            if self.stage.contains(anchor) && !covered.contains(&anchor) {
                anchors.push(anchor);
                for member in self.stage.family(anchor) {
                    if !covered.contains(&member) {
                        covered.push(member);
                    }
                }
            }
        }
        self.preflight.preflight(&self.stage, &anchors)?;
        self.emit_event(sink, LifecyclePhase::Preflight, None)?;
        self.preflight_done = true;
        Ok(())
    }

    fn pre_play(
        &mut self,
        kind: SegmentKind,
        animation_anchors: &[Mob],
        sink: &mut dyn SceneSink,
    ) -> Result<(), SceneError> {
        if self.play_count == u64::MAX {
            return Err(SceneError::InvalidState("play count is exhausted"));
        }
        self.ensure_preflight(animation_anchors, sink)?;
        if self.config.presenter_mode && self.play_count == 0 {
            self.hold_loop(HoldKind::Initial, kind, sink)?;
        }
        self.update_skipping_status()?;
        self.emit_event(sink, LifecyclePhase::PrePlay, Some(kind))
    }

    fn post_play(
        &mut self,
        kind: SegmentKind,
        sink: &mut dyn SceneSink,
        notify_sink: bool,
    ) -> Result<(), SceneError> {
        let result = if notify_sink {
            (|| {
                if self.config.preview_while_skipping && self.skipping && self.config.windowed {
                    self.sync_stage_time();
                    self.dispatch_pending_events()?;
                    sink.capture(
                        CaptureReason::SkippedPreview,
                        FramePacket::freeze_barrier(&self.stage, &self.clock, &self.rng_root),
                    )?;
                }
                self.emit_event(sink, LifecyclePhase::PostPlay, Some(kind))
            })()
        } else {
            Ok(())
        };
        self.play_count += 1;
        result
    }

    fn update_skipping_status(&mut self) -> Result<(), SceneError> {
        if self.config.start_at_play == Some(self.play_count) && !self.original_skipping {
            self.skipping = false;
        }
        if self
            .config
            .end_at_play
            .is_some_and(|end| self.play_count >= end)
        {
            return Err(EndScene.into());
        }
        Ok(())
    }

    fn presenter_wait(&mut self, sink: &mut dyn SceneSink) -> Result<SegmentReport, SceneError> {
        let base_frame = self.clock.now().frames();
        for root in self.stage.roots().to_vec() {
            self.stage.update_mobject(root, 0.0);
        }
        self.hold_loop(HoldKind::Wait, SegmentKind::Wait, sink)?;
        let n_frames = self.clock.now().frames() - base_frame;
        Ok(SegmentReport {
            kind: SegmentKind::Wait,
            purity: Purity::Stateful(vec![ImpureEffect::StopCondition]),
            begin_state: None,
            base_frame,
            n_frames,
            run_time: (RationalTime::zero(self.clock.fps()) + n_frames).to_f64(),
        })
    }

    fn hold_loop(
        &mut self,
        kind: HoldKind,
        segment: SegmentKind,
        sink: &mut dyn SceneSink,
    ) -> Result<(), SceneError> {
        self.emit_event(sink, LifecyclePhase::HoldBegin, Some(segment))?;
        loop {
            self.dispatch_pending_events()?;
            match self
                .hold_controller
                .poll(kind, &self.stage, self.clock.now())?
            {
                HoldDecision::Release => break,
                HoldDecision::EndScene => return Err(EndScene.into()),
                HoldDecision::Continue => {
                    self.clock.advance_frames(1).map_err(AnimError::Clock)?;
                    self.stage
                        .update_at_time(self.clock.dt().to_f64(), self.clock.now().to_f64());
                    self.dispatch_pending_events()?;
                    if !self.skipping {
                        sink.capture(
                            CaptureReason::PresenterHold,
                            FramePacket::freeze_barrier(&self.stage, &self.clock, &self.rng_root),
                        )?;
                    }
                }
            }
        }
        self.emit_event(sink, LifecyclePhase::HoldEnd, Some(segment))
    }

    fn emit_event(
        &self,
        sink: &mut dyn SceneSink,
        phase: LifecyclePhase,
        segment: Option<SegmentKind>,
    ) -> Result<(), SceneError> {
        sink.event(LifecycleEvent {
            phase,
            play_index: self.play_count,
            time: self.clock.now(),
            segment,
            skipping: self.skipping,
        })?;
        Ok(())
    }

    fn sync_stage_time(&mut self) {
        self.stage.set_time_from_clock(self.clock.now().to_f64());
    }

    fn apply_state(
        &mut self,
        frames_elapsed: i64,
        fps: u32,
        play_count: u64,
        rng_state: ([u64; 2], [u64; 2]),
        snapshot: &fmn_mobject::Snapshot,
    ) -> Result<(), SceneError> {
        if frames_elapsed < 0 {
            return Err(SceneError::InvalidState(
                "elapsed frame count must be non-negative",
            ));
        }
        if fps != self.clock.fps() {
            return Err(SceneError::InvalidState(
                "state frame grid does not match this scene",
            ));
        }
        let mut restored_clock = RationalFrameClock::new(fps).map_err(AnimError::Clock)?;
        restored_clock
            .advance_frames(frames_elapsed)
            .map_err(AnimError::Clock)?;
        if !self.stage.can_restore(snapshot) {
            return Err(SceneError::InvalidState(
                "an in-memory SceneState belongs to a different scene",
            ));
        }
        // A restored snapshot is a new execution boundary, not completion of
        // the host-updater window belonging to the previous live state.
        self.idle_owner = std::rc::Rc::new(());
        self.stage.restore(snapshot);
        self.clock = restored_clock;
        self.sync_stage_time();
        self.play_count = play_count;
        self.scene_rng = Pcg64Dxsm::restore(rng_state.0, rng_state.1);
        self.preflight_done = false;
        self.event_dispatcher.reset_state();
        self.queued_events.clear();
        self.event_inbox.clear();
        self.recorded_events.clear();
        self.next_event_sequence = Some(0);
        self.event_error = None;
        self.skipping = self.original_skipping
            || self
                .config
                .start_at_play
                .is_some_and(|start| play_count < start);
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
fn drain_event_queue(
    dispatcher: &mut EventDispatcher,
    inbox: &EventInbox,
    queued: &mut VecDeque<QueuedEvent>,
    recorded: &mut Vec<InputEvent>,
    next_sequence: &mut Option<u64>,
    event_error: &mut Option<EventError>,
    stage: &mut Stage,
    time: RationalTime,
) -> usize {
    if event_error.is_some() {
        return 0;
    }

    let mut candidate_sequence = *next_sequence;
    let sequence_capacity =
        candidate_sequence.map_or(0, |sequence| u128::from(u64::MAX - sequence) + 1);
    let payloads = match inbox.drain_if_at_most(sequence_capacity) {
        Ok(Some(payloads)) => payloads,
        Ok(None) => {
            *event_error = Some(EventError::SequenceExhausted);
            return 0;
        }
        Err(error) => {
            *event_error = Some(error);
            return 0;
        }
    };
    let mut host_events = Vec::new();
    for payload in payloads {
        let Some(sequence) = candidate_sequence else {
            *event_error = Some(EventError::SequenceExhausted);
            return 0;
        };
        candidate_sequence = sequence.checked_add(1);
        host_events.push(QueuedEvent::Live { sequence, payload });
    }
    *next_sequence = candidate_sequence;
    queued.extend(host_events);

    let mut dispatched = 0;
    loop {
        let due = match queued.front() {
            Some(QueuedEvent::Live { .. }) => true,
            Some(QueuedEvent::Replay(event)) => event.timestamp <= time,
            None => false,
        };
        if !due {
            break;
        }
        let event = match queued.pop_front() {
            Some(QueuedEvent::Live { sequence, payload }) => InputEvent {
                sequence,
                timestamp: time,
                payload,
            },
            Some(QueuedEvent::Replay(event)) => event,
            None => break,
        };
        dispatcher.dispatch(&event, stage);
        recorded.push(event);
        dispatched += 1;
    }
    dispatched
}

/// Camera-frame orientation, stored in radians.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraOrientation {
    /// ZXZ theta.
    pub theta: f64,
    /// ZXZ phi.
    pub phi: f64,
    /// ZXZ gamma.
    pub gamma: f64,
}

impl CameraOrientation {
    /// Construct from the Reference's degree-facing convenience surface.
    #[must_use]
    pub fn from_degrees(theta: f64, phi: f64, gamma: f64) -> Self {
        Self {
            theta: theta.to_radians(),
            phi: phi.to_radians(),
            gamma: gamma.to_radians(),
        }
    }
}

/// Options on [`ThreeDScene::add`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThreeDAddOptions {
    /// Apply depth testing to non-fixed objects when the scene default allows.
    pub set_depth_test: bool,
    /// Make stroked vector objects camera-facing (`flat_stroke = false`).
    pub perpendicular_stroke: bool,
}

impl Default for ThreeDAddOptions {
    fn default() -> Self {
        Self {
            set_depth_test: true,
            perpendicular_stroke: true,
        }
    }
}

/// Scene plus the Reference's ThreeDScene defaults.
///
/// The camera frame is represented by one rooted empty mobject with three
/// tracker children. That makes orientation and ambient rotation ordinary,
/// snapshotted Marionette state today; the future projection layer can consume
/// these handles without replacing the Scene semantics.
pub struct ThreeDScene {
    scene: Scene,
    camera_root: Mob,
    theta: Mob,
    phi: Mob,
    gamma: Mob,
    samples: u8,
    always_depth_test: bool,
}

impl fmt::Debug for ThreeDScene {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ThreeDScene")
            .field("scene", &self.scene)
            .field("orientation", &self.orientation())
            .field("samples", &self.samples)
            .field("always_depth_test", &self.always_depth_test)
            .finish()
    }
}

impl ThreeDScene {
    /// Construct at the defaults: 4 samples, `(-30°, 70°, 0°)`, depth test
    /// enabled.
    pub fn new(config: RuntimeConfig, seed: u64) -> Result<Self, SceneError> {
        let mut scene = Scene::new(config, seed)?;
        let defaults =
            CameraOrientation::from_degrees(THREE_D_THETA_DEGREES, THREE_D_PHI_DEGREES, 0.0);
        let theta = scene.stage.add_value_tracker(defaults.theta);
        let phi = scene.stage.add_value_tracker(defaults.phi);
        let gamma = scene.stage.add_value_tracker(defaults.gamma);
        let camera_root = scene.stage.add(Mobject::new());
        for child in [theta, phi, gamma] {
            scene.stage.attach(camera_root, child)?;
        }
        scene.stage.add_to_scene(camera_root)?;
        Ok(Self {
            scene,
            camera_root,
            theta,
            phi,
            gamma,
            samples: THREE_D_CAMERA_SAMPLES,
            always_depth_test: true,
        })
    }

    /// Shared Scene surface.
    #[must_use]
    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    /// Mutable shared Scene surface.
    pub fn scene_mut(&mut self) -> &mut Scene {
        &mut self.scene
    }

    /// Camera frame root handle.
    #[must_use]
    pub fn camera_root(&self) -> Mob {
        self.camera_root
    }

    /// Multisample default.
    #[must_use]
    pub fn samples(&self) -> u8 {
        self.samples
    }

    /// Whether ordinary 3D additions receive depth testing.
    #[must_use]
    pub fn always_depth_test(&self) -> bool {
        self.always_depth_test
    }

    /// Set the scene-level depth default.
    pub fn set_always_depth_test(&mut self, value: bool) {
        self.always_depth_test = value;
    }

    /// Current camera orientation.
    ///
    /// # Errors
    /// Returns [`StageError::StaleHandle`] if any camera handle is dead or no
    /// longer names a scalar tracker.
    pub fn orientation(&self) -> Result<CameraOrientation, SceneError> {
        Ok(CameraOrientation {
            theta: self
                .scene
                .stage
                .tracker_value(self.theta)
                .ok_or(StageError::StaleHandle)?,
            phi: self
                .scene
                .stage
                .tracker_value(self.phi)
                .ok_or(StageError::StaleHandle)?,
            gamma: self
                .scene
                .stage
                .tracker_value(self.gamma)
                .ok_or(StageError::StaleHandle)?,
        })
    }

    /// Materialize the tracker-backed orientation as a Lumen camera frame.
    ///
    /// Trackers remain the authoritative snapshotted scene state. The returned
    /// value is an immutable capture input, so render workers never read the
    /// live Stage while projecting.
    pub fn camera_frame(&self) -> Result<CameraFrame, SceneError> {
        let orientation = self.orientation()?;
        let mut frame = CameraFrame::default();
        frame.set_euler_angles(
            Some(orientation.theta),
            Some(orientation.phi),
            Some(orientation.gamma),
        )?;
        Ok(frame)
    }

    /// Default Lumen capture configuration for this ThreeDScene.
    ///
    /// This is the unambiguous path by which ThreeDScene's four-sample default
    /// feeds ThreeDCamera. Callers may then edit the returned value, including
    /// setting `samples = 0` to disable edge supersampling explicitly.
    pub fn three_d_camera_config(&self) -> Result<RenderCameraConfig, SceneError> {
        Ok(RenderCameraConfig {
            samples: self.samples,
            frame: self.camera_frame()?,
            ..RenderCameraConfig::default()
        })
    }

    /// Build a ThreeDCamera from caller capture configuration and current
    /// tracker-backed orientation.
    ///
    /// Resolution, background, field of view, frame shape/center, and movable
    /// light remain caller-configurable. `samples = 0` remains an explicit
    /// request; use [`ThreeDScene::three_d_camera_config`] when the scene's kept
    /// four-sample default is wanted.
    pub fn three_d_camera(
        &self,
        mut config: RenderCameraConfig,
    ) -> Result<ThreeDCamera, SceneError> {
        let orientation = self.orientation()?;
        config.frame.set_euler_angles(
            Some(orientation.theta),
            Some(orientation.phi),
            Some(orientation.gamma),
        )?;
        Ok(ThreeDCamera::new(config)?)
    }

    /// Set all three Euler angles.
    pub fn set_orientation(
        &mut self,
        orientation: CameraOrientation,
    ) -> Result<&mut Self, SceneError> {
        if !orientation.theta.is_finite()
            || !orientation.phi.is_finite()
            || !orientation.gamma.is_finite()
        {
            return Err(SceneError::InvalidConfig(
                "camera orientation angles must be finite",
            ));
        }
        self.orientation()?;
        self.scene
            .stage
            .set_tracker_value(self.theta, orientation.theta)?;
        self.scene
            .stage
            .set_tracker_value(self.phi, orientation.phi)?;
        self.scene
            .stage
            .set_tracker_value(self.gamma, orientation.gamma)?;
        Ok(self)
    }

    /// Add the Reference's ambient theta rotation as a dt updater.
    pub fn add_ambient_rotation(&mut self, angular_speed: f64) -> Result<UpdaterId, SceneError> {
        if !angular_speed.is_finite() {
            return Err(SceneError::InvalidConfig(
                "ambient angular speed must be finite",
            ));
        }
        Ok(self.scene.stage.add_dt_updater(
            self.theta,
            move |stage, theta, dt| {
                if let Some(current) = stage.tracker_value(theta) {
                    let _ = stage.set_tracker_value(theta, current + angular_speed * dt);
                }
            },
            false,
        )?)
    }

    /// Remove a previously installed ambient-rotation updater.
    pub fn remove_ambient_rotation(&mut self, updater: UpdaterId) {
        self.scene.stage.remove_updater(self.theta, updater);
    }

    /// Add 3D mobjects with depth/perpendicular-stroke defaults, then use the
    /// ordinary stable Scene ordering.
    pub fn add(
        &mut self,
        mobs: &[Mob],
        options: ThreeDAddOptions,
    ) -> Result<&mut Self, SceneError> {
        if mobs.iter().any(|&mob| !self.scene.stage.contains(mob)) {
            return Err(StageError::StaleHandle.into());
        }
        for &mob in mobs {
            let fixed = self
                .scene
                .stage
                .uniforms(mob)
                .is_some_and(|uniforms| uniforms.is_fixed_in_frame == 1.0);
            let family = self.scene.stage.family(mob);
            if options.set_depth_test && self.always_depth_test && !fixed {
                for member in &family {
                    if let Some(uniforms) = self.scene.stage.uniforms_mut(*member) {
                        uniforms.depth_test = true;
                    }
                }
            }
            if options.perpendicular_stroke {
                for member in family {
                    if has_stroke(&self.scene.stage, member)
                        && let Some(uniforms) = self.scene.stage.uniforms_mut(member)
                    {
                        uniforms.flat_stroke = false;
                    }
                }
            }
        }
        self.scene.add(mobs)?;
        Ok(self)
    }
}

impl Default for ThreeDScene {
    fn default() -> Self {
        Self::new(RuntimeConfig::default(), 0).expect("default 3D scene config is valid")
    }
}

impl std::ops::Deref for ThreeDScene {
    type Target = Scene;

    fn deref(&self) -> &Self::Target {
        &self.scene
    }
}

impl std::ops::DerefMut for ThreeDScene {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scene
    }
}

fn has_stroke(stage: &Stage, mob: Mob) -> bool {
    let Some(entry) = stage.get(mob) else {
        return false;
    };
    let widths = entry.buffer.read_column("stroke_width");
    let rgba = entry.buffer.read_column("stroke_rgba");
    match (widths, rgba) {
        (Some(widths), Some(rgba)) => {
            let (colors, _) = rgba.as_chunks::<4>();
            widths
                .iter()
                .zip(colors)
                .any(|(width, color)| *width > 0.0 && color[3] > 0.0)
        }
        _ => false,
    }
}

/// Blank fallback used when no source module/registry is supplied.
#[derive(Debug, Default)]
pub struct BlankScene;

impl SceneProgram for BlankScene {
    fn name(&self) -> &str {
        "BlankScene"
    }

    fn construct(
        &mut self,
        _scene: &mut Scene,
        _sink: &mut dyn SceneSink,
    ) -> Result<(), SceneError> {
        Ok(())
    }
}

type SceneFactory = fn() -> Box<dyn SceneProgram>;

fn scene_factory<P>() -> Box<dyn SceneProgram>
where
    P: SceneProgram + Default + 'static,
{
    Box::new(P::default())
}

/// One named Rust scene registration.
#[derive(Clone)]
pub struct SceneRegistration {
    name: String,
    factory: SceneFactory,
}

impl fmt::Debug for SceneRegistration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SceneRegistration")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

impl SceneRegistration {
    /// Registered scene name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Construct a fresh program.
    #[must_use]
    pub fn instantiate(&self) -> Box<dyn SceneProgram> {
        (self.factory)()
    }
}

/// Ordered Rust scene registry mirroring `SCENES_IN_ORDER`.
#[derive(Debug, Clone, Default)]
pub struct SceneRegistry {
    scenes: Vec<SceneRegistration>,
}

impl SceneRegistry {
    /// No-module fallback.
    #[must_use]
    pub fn blank() -> Self {
        let mut registry = Self::default();
        registry
            .register::<BlankScene>("BlankScene")
            .expect("fresh registry");
        registry
    }

    /// Register a scene in explicit source order.
    pub fn register<P>(&mut self, name: impl Into<String>) -> Result<&mut Self, SceneSelectionError>
    where
        P: SceneProgram + Default + 'static,
    {
        let name = name.into();
        if name.is_empty() {
            return Err(SceneSelectionError::EmptyName);
        }
        if self.scenes.iter().any(|scene| scene.name == name) {
            return Err(SceneSelectionError::DuplicateName(name));
        }
        self.scenes.push(SceneRegistration {
            name,
            factory: scene_factory::<P>,
        });
        Ok(self)
    }

    /// Registrations in explicit order.
    #[must_use]
    pub fn scenes(&self) -> &[SceneRegistration] {
        &self.scenes
    }

    /// Reference selection semantics:
    ///
    /// - `write_all`, or a registry containing one class, selects all;
    /// - requested names are selected in request order and missing names are
    ///   reported without hiding valid selections;
    /// - multiple classes with no valid selection return `SelectionRequired`
    ///   instead of launching a blocking prompt inside the library.
    pub fn select<'a>(
        &'a self,
        requested: &[&str],
        write_all: bool,
    ) -> Result<SceneSelection<'a>, SceneSelectionError> {
        if self.scenes.is_empty() {
            return Err(SceneSelectionError::NoScenes);
        }
        if write_all || self.scenes.len() == 1 {
            return Ok(SceneSelection {
                scenes: self.scenes.iter().collect(),
                missing: Vec::new(),
            });
        }
        let mut scenes = Vec::new();
        let mut missing = Vec::new();
        for &name in requested {
            match self.scenes.iter().find(|scene| scene.name == name) {
                Some(scene) => scenes.push(scene),
                None => missing.push(name.to_owned()),
            }
        }
        if scenes.is_empty() {
            return Err(SceneSelectionError::SelectionRequired {
                choices: self.scenes.iter().map(|scene| scene.name.clone()).collect(),
                missing,
            });
        }
        Ok(SceneSelection { scenes, missing })
    }
}

/// A resolved scene selection plus names the registry did not contain.
#[derive(Debug)]
pub struct SceneSelection<'a> {
    /// Selected registrations, in selection order.
    pub scenes: Vec<&'a SceneRegistration>,
    /// Requested names that were not found.
    pub missing: Vec<String>,
}

/// Scene registration/selection refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneSelectionError {
    /// Empty registration name.
    EmptyName,
    /// Duplicate registration.
    DuplicateName(String),
    /// Registry had no scenes.
    NoScenes,
    /// A noninteractive caller must select from these choices.
    SelectionRequired {
        /// Available names in registry order.
        choices: Vec<String>,
        /// Requested names that did not exist.
        missing: Vec<String>,
    },
}

impl fmt::Display for SceneSelectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyName => f.write_str("scene registration name cannot be empty"),
            Self::DuplicateName(name) => write!(f, "duplicate scene registration {name:?}"),
            Self::NoScenes => f.write_str("no scenes are registered"),
            Self::SelectionRequired { choices, missing } => write!(
                f,
                "scene selection required; choices={choices:?}, missing={missing:?}"
            ),
        }
    }
}

impl std::error::Error for SceneSelectionError {}

/// Pure output naming/path semantics owned by Scene and consumed by Reel/CLI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputNaming {
    /// Base output directory.
    pub output_directory: PathBuf,
    /// Explicit `--file_name`, if any.
    pub file_name: Option<PathBuf>,
    /// Range suffix start.
    pub start_at_play: Option<u64>,
    /// Range suffix end.
    pub end_at_play: Option<u64>,
    /// Host request corresponding to `-o`; no subprocess is invoked here (D2).
    pub open_on_completion: bool,
}

impl Default for OutputNaming {
    fn default() -> Self {
        Self {
            output_directory: PathBuf::from("."),
            file_name: None,
            start_at_play: None,
            end_at_play: None,
            open_on_completion: false,
        }
    }
}

impl OutputNaming {
    /// Root name before an output format's extension is applied.
    #[must_use]
    pub fn root(&self, scene_name: &str) -> PathBuf {
        let name = self.file_name.clone().unwrap_or_else(|| {
            let mut name = scene_name.to_owned();
            if let Some(start) = self.start_at_play {
                name.push('_');
                name.push_str(&start.to_string());
            }
            if let Some(end) = self.end_at_play {
                name.push('_');
                name.push_str(&end.to_string());
            }
            PathBuf::from(name)
        });
        self.output_directory.join(name)
    }

    /// Final artifact path. A leading dot on `extension` is accepted.
    #[must_use]
    pub fn artifact(&self, scene_name: &str, extension: &str) -> PathBuf {
        with_extension(self.root(scene_name), extension)
    }

    /// Subdivided output directory (the extensionless root).
    #[must_use]
    pub fn partial_directory(&self, scene_name: &str) -> PathBuf {
        self.root(scene_name)
    }

    /// `00000.ext`, `00001.ext`, … inside the subdivided root.
    #[must_use]
    pub fn partial_artifact(&self, scene_name: &str, play_index: u64, extension: &str) -> PathBuf {
        let file = format!("{play_index:05}");
        with_extension(self.partial_directory(scene_name).join(file), extension)
    }

    /// Host-facing completion request. D2 forbids spawning an OS opener; the
    /// CLI/Studio host may honor this request through its own capability.
    #[must_use]
    pub fn completion_request(&self, scene_name: &str, extension: &str) -> CompletionRequest {
        CompletionRequest {
            artifact: self.artifact(scene_name, extension),
            open: self.open_on_completion,
        }
    }
}

fn with_extension(mut path: PathBuf, extension: &str) -> PathBuf {
    path.set_extension(extension.strip_prefix('.').unwrap_or(extension));
    path
}

/// Output completion metadata consumed by a host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionRequest {
    /// Published artifact.
    pub artifact: PathBuf,
    /// Whether the host was asked to open/present it.
    pub open: bool,
}

impl CompletionRequest {
    /// Borrow the artifact as a path.
    #[must_use]
    pub fn artifact(&self) -> &Path {
        &self.artifact
    }
}
