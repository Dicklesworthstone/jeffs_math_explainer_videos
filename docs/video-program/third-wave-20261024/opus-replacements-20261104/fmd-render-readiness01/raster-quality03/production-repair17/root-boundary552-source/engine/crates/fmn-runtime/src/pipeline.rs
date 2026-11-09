//! Bounded, ordered, cancellation-safe frame-stage overlap.
//!
//! The caller is the scene/update stage: advancing the supplied iterator freezes
//! the next semantic frame. Prepared, rasterized, and converted values are
//! required to be owned and `Send`; this is the seam that keeps Marionette's
//! live stage and Python callbacks on their serial owner while immutable work
//! crosses render-team boundaries.

use crate::plan::{ExecutionPlan, TeamPlan};
use fmn_platform::clock::{Clock, StdClock};
use fmn_platform::profile::{
    ProfileCounter, ProfileLane, ProfilePath, ProfilePhase, ProfileRecorder,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::{
    Receiver, SendError, Sender, SyncSender, TryRecvError, TrySendError, channel, sync_channel,
};
use std::time::Duration;

trait ScopedSpawner {
    fn spawn<'scope, 'env: 'scope, F>(
        &self,
        scope: &'scope std::thread::Scope<'scope, 'env>,
        work: F,
    ) -> std::io::Result<std::thread::ScopedJoinHandle<'scope, ()>>
    where
        F: FnOnce() + Send + 'scope;
}

struct NativeScopedSpawner;

impl ScopedSpawner for NativeScopedSpawner {
    fn spawn<'scope, 'env: 'scope, F>(
        &self,
        scope: &'scope std::thread::Scope<'scope, 'env>,
        work: F,
    ) -> std::io::Result<std::thread::ScopedJoinHandle<'scope, ()>>
    where
        F: FnOnce() + Send + 'scope,
    {
        std::thread::Builder::new().spawn_scoped(scope, work)
    }
}

/// One source item: a frozen frame or an effect-model barrier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineEvent<F, B> {
    /// A semantic frame, in strictly increasing scene order.
    Frame {
        /// Durable frame/sequence number.
        sequence: u64,
        /// Caller-owned frozen work.
        frame: F,
    },
    /// Drain every earlier frame, then run the barrier callback.
    Barrier(B),
}

impl<F, B> PipelineEvent<F, B> {
    /// Construct a frame event.
    #[must_use]
    pub const fn frame(sequence: u64, frame: F) -> Self {
        Self::Frame { sequence, frame }
    }

    /// Construct a pipeline barrier.
    #[must_use]
    pub const fn barrier(barrier: B) -> Self {
        Self::Barrier(barrier)
    }
}

/// A pipeline callback's location.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineStage {
    /// Iterating scene/update work on the caller thread.
    Scene,
    /// Render-plan synchronization and binning.
    Prepare,
    /// Frame rasterization on a render team.
    Raster,
    /// Color conversion / sink preparation.
    Convert,
    /// Ordered handoff to W8's emitter seam.
    Emit,
    /// Pixel-observing/effect barrier callback.
    Barrier,
}

impl fmt::Display for PipelineStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Scene => "scene",
            Self::Prepare => "prepare",
            Self::Raster => "raster",
            Self::Convert => "convert",
            Self::Emit => "emit",
            Self::Barrier => "barrier",
        })
    }
}

/// Caller implementation of the three worker-owned stages.
///
/// The methods receive the exact advisory team plan. A Lumen adapter passes
/// `team.threads()` to `FrameJob::render_into`; engines with owned pools can
/// additionally honor the locality lanes and scratch sizes.
pub trait PipelineStages: Sync {
    /// Frozen source item produced by scene/update.
    type Frame: Send;
    /// Immutable synchronized IR + bins.
    type Prepared: Send;
    /// Raw raster result.
    type Rasterized: Send;
    /// Converted result ready for ordered emission.
    type Output: Send;
    /// Domain error shared by stage, barrier, and emitter callbacks.
    type Error: Send;

    /// Synchronize retained mirrors and build bins.
    fn prepare(
        &self,
        frame: Self::Frame,
        scene_team: &TeamPlan,
    ) -> Result<Self::Prepared, Self::Error>;

    /// Rasterize one frame. Several calls may run concurrently on independent
    /// render-team leaders.
    fn rasterize(
        &self,
        prepared: Self::Prepared,
        render_team: &TeamPlan,
    ) -> Result<Self::Rasterized, Self::Error>;

    /// Convert a raw frame into the negotiated sink surface.
    fn convert(
        &self,
        rasterized: Self::Rasterized,
        output_team: &TeamPlan,
    ) -> Result<Self::Output, Self::Error>;
}

/// Cloneable cooperative stop request.
///
/// Synchronous stage callbacks are not force-killed. Cancellation is observed
/// before and after every stage, all queued jobs become terminal, and every
/// in-flight slot is released before [`FramePipeline::run`] returns.
#[derive(Debug, Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    /// A live token.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Request cancellation.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    /// Whether cancellation has been requested.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}

/// Snapshot handed to a barrier after the drain has completed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BarrierContext {
    /// Frames submitted before this point.
    pub submitted: u64,
    /// Frames already handed to the ordered emitter seam.
    pub emitted: u64,
    /// Always zero at a correctly executed barrier.
    pub outstanding_slots: usize,
}

/// Per-stage work and occupancy counters (§17.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageUtilization {
    /// Successful callback invocations.
    pub jobs: u64,
    /// Sum of callback wall times.
    pub busy: Duration,
    /// Concurrent stage leaders (render has one per team).
    pub workers: usize,
}

impl StageUtilization {
    /// Busy fraction over the whole run, clamped to `[0, 1]`.
    #[must_use]
    pub fn fraction(&self, elapsed: Duration) -> f64 {
        let capacity = elapsed.as_secs_f64() * self.workers.max(1) as f64;
        if capacity == 0.0 {
            return 0.0;
        }
        (self.busy.as_secs_f64() / capacity).clamp(0.0, 1.0)
    }
}

/// One completed pipeline run's structured counters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineStats {
    /// Total caller-thread wall time.
    pub elapsed: Duration,
    /// Time from start to the first ordered emission.
    pub first_output_latency: Option<Duration>,
    /// Frames accepted.
    pub submitted: u64,
    /// Frames successfully prepared.
    pub prepared: u64,
    /// Frames successfully rasterized.
    pub rasterized: u64,
    /// Frames successfully converted.
    pub converted: u64,
    /// Frames handed off in sequence order.
    pub emitted: u64,
    /// Explicit barriers executed.
    pub barriers: u64,
    /// Times the source had to await a global frame slot.
    pub backpressure_waits: u64,
    /// Largest observed global slot occupancy.
    pub max_in_flight: usize,
    /// Outstanding permits after all workers joined (must be zero).
    pub outstanding_slots: usize,
    /// Caller-owned source generation.
    pub scene: StageUtilization,
    /// IR synchronization/binning.
    pub prepare: StageUtilization,
    /// All render teams combined.
    pub raster: StageUtilization,
    /// Conversion.
    pub convert: StageUtilization,
    /// Ordered emission.
    pub emit: StageUtilization,
    /// Drained effect-model barriers.
    pub barrier: StageUtilization,
    /// Successful frames per render team.
    pub render_team_frames: Vec<u64>,
    /// Callback wall time per render team.
    pub render_team_busy: Vec<Duration>,
}

/// Pipeline failure kind.
#[derive(Debug)]
pub enum PipelineError<E> {
    /// Source frame numbers moved backward or repeated.
    NonMonotonicSequence {
        /// Last accepted sequence.
        previous: u64,
        /// Rejected sequence.
        next: u64,
    },
    /// A user callback returned an error.
    Stage {
        /// Frame sequence, or `None` for a barrier.
        sequence: Option<u64>,
        /// Callback location.
        stage: PipelineStage,
        /// Domain error.
        source: E,
    },
    /// A caller callback panicked; the panic was contained and the pipeline drained.
    CallbackPanicked {
        /// Frame sequence, when the callback had accepted one.
        sequence: Option<u64>,
        /// Callback location.
        stage: PipelineStage,
    },
    /// The host refused a worker before the complete pipeline topology existed.
    WorkerSpawnFailed {
        /// Worker stage whose start was refused.
        stage: PipelineStage,
        /// Total workers required by the plan.
        requested: usize,
        /// Workers started before the refusal.
        spawned: usize,
    },
    /// A stage queue disappeared before accepting its work.
    StageDisconnected {
        /// Frame sequence, when one was in hand.
        sequence: Option<u64>,
        /// Failed destination stage.
        stage: PipelineStage,
    },
    /// Cooperative cancellation completed.
    Cancelled,
    /// An internal permit survived worker shutdown.
    LeakedSlots {
        /// Outstanding permits.
        outstanding: usize,
    },
}

impl<E: fmt::Display> fmt::Display for PipelineError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonMonotonicSequence { previous, next } => {
                write!(f, "frame sequence {next} did not follow {previous}")
            }
            Self::Stage {
                sequence,
                stage,
                source,
            } => {
                if let Some(sequence) = sequence {
                    write!(f, "{stage} stage failed for frame {sequence}: {source}")
                } else {
                    write!(f, "{stage} callback failed: {source}")
                }
            }
            Self::CallbackPanicked { sequence, stage } => {
                if let Some(sequence) = sequence {
                    write!(f, "{stage} callback panicked for frame {sequence}")
                } else {
                    write!(f, "{stage} callback panicked")
                }
            }
            Self::WorkerSpawnFailed {
                stage,
                requested,
                spawned,
            } => write!(
                f,
                "{stage} worker startup failed after {spawned} of {requested} pipeline workers started"
            ),
            Self::StageDisconnected { sequence, stage } => {
                if let Some(sequence) = sequence {
                    write!(f, "{stage} stage disconnected for frame {sequence}")
                } else {
                    write!(f, "{stage} stage disconnected")
                }
            }
            Self::Cancelled => f.write_str("frame pipeline cancelled"),
            Self::LeakedSlots { outstanding } => {
                write!(f, "frame pipeline leaked {outstanding} in-flight slots")
            }
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for PipelineError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Stage { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Error plus the counters collected while draining.
#[derive(Debug)]
pub struct PipelineFailure<E> {
    /// Root failure.
    pub error: PipelineError<E>,
    /// Final counters; `outstanding_slots` is still required to be zero.
    pub stats: Box<PipelineStats>,
}

impl<E: fmt::Display> fmt::Display for PipelineFailure<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error.fmt(f)
    }
}

impl<E: std::error::Error + 'static> std::error::Error for PipelineFailure<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// Four-stage bounded executor.
pub struct FramePipeline<'a, S: PipelineStages> {
    plan: &'a ExecutionPlan,
    stages: &'a S,
    cancellation: CancellationToken,
    clock: Arc<dyn Clock>,
    profile: ProfileRecorder,
    profile_path: ProfilePath,
}

impl<S: PipelineStages> fmt::Debug for FramePipeline<'_, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FramePipeline")
            .field("plan", self.plan)
            .field("cancellation", &self.cancellation)
            .field("profile", &self.profile)
            .field("profile_path", &self.profile_path)
            .finish_non_exhaustive()
    }
}

impl<'a, S: PipelineStages> FramePipeline<'a, S> {
    /// Construct a pipeline with a fresh cancellation token.
    #[must_use]
    pub fn new(plan: &'a ExecutionPlan, stages: &'a S) -> Self {
        Self {
            plan,
            stages,
            cancellation: CancellationToken::new(),
            clock: Arc::new(StdClock::new()),
            profile: ProfileRecorder::disabled(),
            profile_path: ProfilePath::scene(0),
        }
    }

    /// Construct a pipeline controlled by an existing token.
    #[must_use]
    pub fn with_cancellation(
        plan: &'a ExecutionPlan,
        stages: &'a S,
        cancellation: CancellationToken,
    ) -> Self {
        Self {
            plan,
            stages,
            cancellation,
            clock: Arc::new(StdClock::new()),
            profile: ProfileRecorder::disabled(),
            profile_path: ProfilePath::scene(0),
        }
    }

    /// Construct a pipeline with explicit cancellation and clock capabilities.
    ///
    /// Deterministic scheduler tests hand in `FakeClock`; production ordinarily
    /// uses [`Self::new`] or [`Self::with_cancellation`].
    #[must_use]
    pub fn with_clock(
        plan: &'a ExecutionPlan,
        stages: &'a S,
        cancellation: CancellationToken,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            plan,
            stages,
            cancellation,
            clock,
            profile: ProfileRecorder::disabled(),
            profile_path: ProfilePath::scene(0),
        }
    }

    /// Construct a pipeline with explicit clock and profiling capabilities.
    ///
    /// The recorder is disabled by default in every other constructor. A
    /// profiled composition root hands the same clock to this pipeline and to
    /// any nested subsystem spans, producing one comparable monotonic epoch.
    #[must_use]
    pub fn with_clock_and_profile(
        plan: &'a ExecutionPlan,
        stages: &'a S,
        cancellation: CancellationToken,
        clock: Arc<dyn Clock>,
        profile: ProfileRecorder,
    ) -> Self {
        Self {
            plan,
            stages,
            cancellation,
            clock,
            profile,
            profile_path: ProfilePath::scene(0),
        }
    }

    /// Set the scene/play prefix inherited by every frame and phase record.
    #[must_use]
    pub const fn with_profile_path(mut self, path: ProfilePath) -> Self {
        self.profile_path = path;
        self
    }

    /// A clone of this run's stop handle.
    #[must_use]
    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancellation.clone()
    }

    /// Execute source events with ordered output and drained barrier callbacks.
    ///
    /// Advancing `events` is the serial scene/update stage. `emit` is called on
    /// the caller thread in submitted order, regardless of worker completion
    /// order. A barrier callback runs only after every earlier `emit` returned.
    ///
    /// # Errors
    /// Returns [`PipelineFailure`] after all worker threads have joined and all
    /// frame-slot permits have been released.
    pub fn run<I, B, Emit, Barrier>(
        &self,
        events: I,
        emit: Emit,
        barrier: Barrier,
    ) -> Result<PipelineStats, PipelineFailure<S::Error>>
    where
        I: IntoIterator<Item = PipelineEvent<S::Frame, B>>,
        Emit: FnMut(u64, S::Output) -> Result<(), S::Error>,
        Barrier: FnMut(B, BarrierContext) -> Result<(), S::Error>,
    {
        self.run_with_spawner(events, emit, barrier, &NativeScopedSpawner)
    }

    fn run_with_spawner<I, B, Emit, Barrier, Spawner>(
        &self,
        events: I,
        mut emit: Emit,
        mut barrier: Barrier,
        spawner: &Spawner,
    ) -> Result<PipelineStats, PipelineFailure<S::Error>>
    where
        I: IntoIterator<Item = PipelineEvent<S::Frame, B>>,
        Emit: FnMut(u64, S::Output) -> Result<(), S::Error>,
        Barrier: FnMut(B, BarrierContext) -> Result<(), S::Error>,
        Spawner: ScopedSpawner,
    {
        let clock = Arc::clone(&self.clock);
        let start = clock.monotonic();
        let counters = Arc::new(Counters::new(
            self.plan.render_teams.len(),
            self.profile.clone(),
            self.profile_path,
        ));
        let slots = Arc::new(SlotTracker::new(self.plan.frames_in_flight.max(1)));
        let cancellation = self.cancellation.clone();

        let mut coordinator = std::thread::scope(|scope| {
            let requested = self.plan.render_teams.len().saturating_add(2);
            let mut spawned = 0;
            let mut state = Coordinator::new();
            let (completion_tx, completion_rx) = channel();
            let (raster_tx, raster_rx) = sync_channel(self.plan.frames_in_flight.max(1));

            let output_completion = completion_tx.clone();
            let output_counters = Arc::clone(&counters);
            let output_cancel = cancellation.clone();
            let output_clock = Arc::clone(&clock);
            let output_team = &self.plan.output_team;
            if spawner
                .spawn(scope, move || {
                    output_worker(
                        self.stages,
                        output_team,
                        raster_rx,
                        output_completion,
                        output_counters,
                        output_cancel,
                        output_clock,
                    );
                })
                .is_err()
            {
                state.fail(
                    PipelineError::WorkerSpawnFailed {
                        stage: PipelineStage::Convert,
                        requested,
                        spawned,
                    },
                    &cancellation,
                );
                return state;
            }
            spawned += 1;

            let mut render_senders = Vec::with_capacity(self.plan.render_teams.len());
            for (team_index, team) in self.plan.render_teams.iter().enumerate() {
                let (sender, receiver) = sync_channel(1);
                render_senders.push(sender);
                let team_raster = raster_tx.clone();
                let team_completion = completion_tx.clone();
                let team_counters = Arc::clone(&counters);
                let team_cancel = cancellation.clone();
                let team_clock = Arc::clone(&clock);
                if spawner
                    .spawn(scope, move || {
                        render_worker(
                            self.stages,
                            team,
                            team_index,
                            receiver,
                            team_raster,
                            team_completion,
                            team_counters,
                            team_cancel,
                            team_clock,
                        );
                    })
                    .is_err()
                {
                    state.fail(
                        PipelineError::WorkerSpawnFailed {
                            stage: PipelineStage::Raster,
                            requested,
                            spawned,
                        },
                        &cancellation,
                    );
                    return state;
                }
                spawned += 1;
            }
            drop(raster_tx);

            let (prepare_tx, prepare_rx) = sync_channel(self.plan.frames_in_flight.max(1));
            let prepare_completion = completion_tx.clone();
            let prepare_counters = Arc::clone(&counters);
            let prepare_cancel = cancellation.clone();
            let prepare_clock = Arc::clone(&clock);
            let scene_team = &self.plan.scene_team;
            if spawner
                .spawn(scope, move || {
                    prepare_worker(
                        self.stages,
                        scene_team,
                        prepare_rx,
                        render_senders,
                        prepare_completion,
                        prepare_counters,
                        prepare_cancel,
                        prepare_clock,
                    );
                })
                .is_err()
            {
                state.fail(
                    PipelineError::WorkerSpawnFailed {
                        stage: PipelineStage::Prepare,
                        requested,
                        spawned,
                    },
                    &cancellation,
                );
                return state;
            }
            drop(completion_tx);

            let mut events = match catch_unwind(AssertUnwindSafe(|| events.into_iter())) {
                Ok(events) => events,
                Err(_) => {
                    state.fail(
                        PipelineError::CallbackPanicked {
                            sequence: None,
                            stage: PipelineStage::Scene,
                        },
                        &cancellation,
                    );
                    drop(prepare_tx);
                    return state;
                }
            };
            let mut previous = None;

            loop {
                if cancellation.is_cancelled() || state.failed() {
                    break;
                }

                // A slot covers the frozen source value as well as every worker
                // queue it subsequently traverses. Waiting before `next()`
                // prevents the scene iterator from materializing an untracked
                // `(limit + 1)`th frame.
                while state.outstanding_len() >= self.plan.frames_in_flight.max(1)
                    && !state.failed()
                {
                    counters.backpressure_waits.fetch_add(1, Ordering::Relaxed);
                    state.receive_one(
                        &completion_rx,
                        &mut emit,
                        &counters,
                        &slots,
                        start,
                        clock.as_ref(),
                        &cancellation,
                    );
                }
                if state.failed() || cancellation.is_cancelled() {
                    break;
                }

                let scene_start = clock.monotonic();
                let event = catch_unwind(AssertUnwindSafe(|| events.next()));
                let scene_elapsed = elapsed_since(clock.as_ref(), scene_start);
                add_duration(&counters.scene_ns, scene_elapsed);
                let event = match event {
                    Ok(event) => event,
                    Err(_) => {
                        state.fail(
                            PipelineError::CallbackPanicked {
                                sequence: None,
                                stage: PipelineStage::Scene,
                            },
                            &cancellation,
                        );
                        break;
                    }
                };
                let Some(event) = event else {
                    break;
                };
                let path = match &event {
                    PipelineEvent::Frame { sequence, .. } => {
                        counters.profile_path.with_frame(*sequence)
                    }
                    PipelineEvent::Barrier(_) => counters.profile_path,
                };
                counters.profile.record_span(
                    path,
                    ProfilePhase::SceneUpdate,
                    ProfileLane::caller(),
                    scene_start,
                    scene_elapsed,
                );
                counters.scene_jobs.fetch_add(1, Ordering::Relaxed);

                match event {
                    PipelineEvent::Frame { sequence, frame } => {
                        if previous.is_some_and(|last| sequence <= last) {
                            state.fail(
                                PipelineError::NonMonotonicSequence {
                                    previous: previous.unwrap_or(sequence),
                                    next: sequence,
                                },
                                &cancellation,
                            );
                            break;
                        }
                        previous = Some(sequence);

                        let permit = SlotPermit::new(Arc::clone(&slots));
                        counters.submitted.fetch_add(1, Ordering::Relaxed);
                        state.submit(sequence);
                        let work = Work {
                            sequence,
                            value: frame,
                            permit,
                        };
                        if let Err(SendError(work)) = prepare_tx.send(work) {
                            state.accept(
                                Completion::Disconnected {
                                    sequence: work.sequence,
                                    stage: PipelineStage::Prepare,
                                    permit: work.permit,
                                },
                                &mut emit,
                                &counters,
                                &slots,
                                start,
                                clock.as_ref(),
                                &cancellation,
                            );
                        }
                        state.receive_ready(
                            &completion_rx,
                            &mut emit,
                            &counters,
                            &slots,
                            start,
                            clock.as_ref(),
                            &cancellation,
                        );
                    }
                    PipelineEvent::Barrier(payload) => {
                        state.drain(
                            &completion_rx,
                            &mut emit,
                            &counters,
                            &slots,
                            start,
                            clock.as_ref(),
                            &cancellation,
                        );
                        if state.failed() || cancellation.is_cancelled() {
                            break;
                        }
                        let context = BarrierContext {
                            submitted: counters.submitted.load(Ordering::Relaxed),
                            emitted: counters.emitted.load(Ordering::Relaxed),
                            outstanding_slots: slots.outstanding(),
                        };
                        debug_assert_eq!(context.outstanding_slots, 0);
                        let barrier_start = clock.monotonic();
                        let barrier_result =
                            catch_unwind(AssertUnwindSafe(|| barrier(payload, context)));
                        let barrier_elapsed = elapsed_since(clock.as_ref(), barrier_start);
                        add_duration(&counters.barrier_ns, barrier_elapsed);
                        counters.profile.record_span(
                            counters.profile_path,
                            ProfilePhase::Barrier,
                            ProfileLane::caller(),
                            barrier_start,
                            barrier_elapsed,
                        );
                        match barrier_result {
                            Ok(Ok(())) => {
                                counters.barriers.fetch_add(1, Ordering::Relaxed);
                            }
                            Ok(Err(source)) => {
                                state.fail(
                                    PipelineError::Stage {
                                        sequence: None,
                                        stage: PipelineStage::Barrier,
                                        source,
                                    },
                                    &cancellation,
                                );
                                break;
                            }
                            Err(_) => {
                                state.fail(
                                    PipelineError::CallbackPanicked {
                                        sequence: None,
                                        stage: PipelineStage::Barrier,
                                    },
                                    &cancellation,
                                );
                                break;
                            }
                        }
                    }
                }
            }

            drop(prepare_tx);
            state.drain(
                &completion_rx,
                &mut emit,
                &counters,
                &slots,
                start,
                clock.as_ref(),
                &cancellation,
            );
            if cancellation.is_cancelled() {
                state.fail_if_empty(PipelineError::Cancelled, &cancellation);
            }
            state
        });

        let elapsed = elapsed_since(clock.as_ref(), start);
        let stats = counters.snapshot(elapsed, &slots, self.plan);
        if stats.outstanding_slots != 0 {
            coordinator.fail_without_cancel(PipelineError::LeakedSlots {
                outstanding: stats.outstanding_slots,
            });
        }
        if let Some(error) = coordinator.error {
            Err(PipelineFailure {
                error,
                stats: Box::new(stats),
            })
        } else {
            Ok(stats)
        }
    }
}

struct Work<T> {
    sequence: u64,
    value: T,
    permit: SlotPermit,
}

enum Completion<O, E> {
    Output(Work<O>),
    Error {
        sequence: u64,
        stage: PipelineStage,
        source: E,
        permit: SlotPermit,
    },
    Panicked {
        sequence: u64,
        stage: PipelineStage,
        permit: SlotPermit,
    },
    Disconnected {
        sequence: u64,
        stage: PipelineStage,
        permit: SlotPermit,
    },
    Cancelled {
        sequence: u64,
        permit: SlotPermit,
    },
}

#[allow(clippy::too_many_arguments)]
fn prepare_worker<S: PipelineStages>(
    stages: &S,
    scene_team: &TeamPlan,
    receiver: Receiver<Work<S::Frame>>,
    render_senders: Vec<SyncSender<Work<S::Prepared>>>,
    completion: Sender<Completion<S::Output, S::Error>>,
    counters: Arc<Counters>,
    cancellation: CancellationToken,
    clock: Arc<dyn Clock>,
) {
    let mut next_team = 0;
    for work in receiver {
        if cancellation.is_cancelled() {
            send_cancelled(&completion, work.sequence, work.permit);
            continue;
        }
        let Work {
            sequence,
            value,
            permit,
        } = work;
        let started = clock.monotonic();
        let result = catch_unwind(AssertUnwindSafe(|| stages.prepare(value, scene_team)));
        let elapsed = elapsed_since(clock.as_ref(), started);
        add_duration(&counters.prepare_ns, elapsed);
        counters.profile.record_span(
            counters.profile_path.with_frame(sequence),
            ProfilePhase::Prepare,
            ProfileLane::prepare(),
            started,
            elapsed,
        );
        match result {
            Ok(Ok(prepared)) => {
                counters.prepared.fetch_add(1, Ordering::Relaxed);
                let work = Work {
                    sequence,
                    value: prepared,
                    permit,
                };
                if cancellation.is_cancelled() {
                    send_cancelled(&completion, sequence, work.permit);
                } else {
                    dispatch_to_render(
                        work,
                        &render_senders,
                        &mut next_team,
                        &completion,
                        &cancellation,
                    );
                }
            }
            Ok(Err(source)) => {
                cancellation.cancel();
                let _ = completion.send(Completion::Error {
                    sequence,
                    stage: PipelineStage::Prepare,
                    source,
                    permit,
                });
            }
            Err(_) => {
                cancellation.cancel();
                let _ = completion.send(Completion::Panicked {
                    sequence,
                    stage: PipelineStage::Prepare,
                    permit,
                });
            }
        }
    }
}

fn dispatch_to_render<O, E, P>(
    mut work: Work<P>,
    senders: &[SyncSender<Work<P>>],
    next_team: &mut usize,
    completion: &Sender<Completion<O, E>>,
    cancellation: &CancellationToken,
) {
    if senders.is_empty() {
        let _ = completion.send(Completion::Disconnected {
            sequence: work.sequence,
            stage: PipelineStage::Raster,
            permit: work.permit,
        });
        cancellation.cancel();
        return;
    }

    loop {
        if cancellation.is_cancelled() {
            send_cancelled(completion, work.sequence, work.permit);
            return;
        }
        let mut disconnected = 0;
        for offset in 0..senders.len() {
            let index = (*next_team + offset) % senders.len();
            match senders[index].try_send(work) {
                Ok(()) => {
                    *next_team = (index + 1) % senders.len();
                    return;
                }
                Err(TrySendError::Full(returned)) => work = returned,
                Err(TrySendError::Disconnected(returned)) => {
                    work = returned;
                    disconnected += 1;
                }
            }
        }
        if disconnected == senders.len() {
            cancellation.cancel();
            let _ = completion.send(Completion::Disconnected {
                sequence: work.sequence,
                stage: PipelineStage::Raster,
                permit: work.permit,
            });
            return;
        }
        std::thread::yield_now();
    }
}

#[allow(clippy::too_many_arguments)]
fn render_worker<S: PipelineStages>(
    stages: &S,
    team: &TeamPlan,
    team_index: usize,
    receiver: Receiver<Work<S::Prepared>>,
    raster_sender: SyncSender<Work<S::Rasterized>>,
    completion: Sender<Completion<S::Output, S::Error>>,
    counters: Arc<Counters>,
    cancellation: CancellationToken,
    clock: Arc<dyn Clock>,
) {
    for work in receiver {
        if cancellation.is_cancelled() {
            send_cancelled(&completion, work.sequence, work.permit);
            continue;
        }
        let Work {
            sequence,
            value,
            permit,
        } = work;
        let started = clock.monotonic();
        let result = catch_unwind(AssertUnwindSafe(|| stages.rasterize(value, team)));
        let elapsed = elapsed_since(clock.as_ref(), started);
        add_duration(&counters.raster_ns, elapsed);
        counters.profile.record_span(
            counters.profile_path.with_frame(sequence),
            ProfilePhase::Raster,
            ProfileLane::render(u16::try_from(team_index).unwrap_or(u16::MAX)),
            started,
            elapsed,
        );
        if let Some(team_ns) = counters.render_team_ns.get(team_index) {
            add_duration(team_ns, elapsed);
        }
        match result {
            Ok(Ok(rasterized)) => {
                counters.rasterized.fetch_add(1, Ordering::Relaxed);
                if let Some(team_frames) = counters.render_team_frames.get(team_index) {
                    team_frames.fetch_add(1, Ordering::Relaxed);
                }
                let work = Work {
                    sequence,
                    value: rasterized,
                    permit,
                };
                if cancellation.is_cancelled() {
                    send_cancelled(&completion, sequence, work.permit);
                } else if let Err(SendError(work)) = raster_sender.send(work) {
                    cancellation.cancel();
                    let _ = completion.send(Completion::Disconnected {
                        sequence: work.sequence,
                        stage: PipelineStage::Convert,
                        permit: work.permit,
                    });
                }
            }
            Ok(Err(source)) => {
                cancellation.cancel();
                let _ = completion.send(Completion::Error {
                    sequence,
                    stage: PipelineStage::Raster,
                    source,
                    permit,
                });
            }
            Err(_) => {
                cancellation.cancel();
                let _ = completion.send(Completion::Panicked {
                    sequence,
                    stage: PipelineStage::Raster,
                    permit,
                });
            }
        }
    }
}

fn output_worker<S: PipelineStages>(
    stages: &S,
    output_team: &TeamPlan,
    receiver: Receiver<Work<S::Rasterized>>,
    completion: Sender<Completion<S::Output, S::Error>>,
    counters: Arc<Counters>,
    cancellation: CancellationToken,
    clock: Arc<dyn Clock>,
) {
    for work in receiver {
        if cancellation.is_cancelled() {
            send_cancelled(&completion, work.sequence, work.permit);
            continue;
        }
        let Work {
            sequence,
            value,
            permit,
        } = work;
        let started = clock.monotonic();
        let result = catch_unwind(AssertUnwindSafe(|| stages.convert(value, output_team)));
        let elapsed = elapsed_since(clock.as_ref(), started);
        add_duration(&counters.convert_ns, elapsed);
        counters.profile.record_span(
            counters.profile_path.with_frame(sequence),
            ProfilePhase::ColorConversion,
            ProfileLane::output(),
            started,
            elapsed,
        );
        match result {
            Ok(Ok(output)) => {
                counters.converted.fetch_add(1, Ordering::Relaxed);
                if cancellation.is_cancelled() {
                    send_cancelled(&completion, sequence, permit);
                } else {
                    let _ = completion.send(Completion::Output(Work {
                        sequence,
                        value: output,
                        permit,
                    }));
                }
            }
            Ok(Err(source)) => {
                cancellation.cancel();
                let _ = completion.send(Completion::Error {
                    sequence,
                    stage: PipelineStage::Convert,
                    source,
                    permit,
                });
            }
            Err(_) => {
                cancellation.cancel();
                let _ = completion.send(Completion::Panicked {
                    sequence,
                    stage: PipelineStage::Convert,
                    permit,
                });
            }
        }
    }
}

fn send_cancelled<O, E>(completion: &Sender<Completion<O, E>>, sequence: u64, permit: SlotPermit) {
    let _ = completion.send(Completion::Cancelled { sequence, permit });
}

struct Coordinator<O, E> {
    order: VecDeque<u64>,
    outstanding: BTreeSet<u64>,
    outputs: BTreeMap<u64, Work<O>>,
    terminal: BTreeSet<u64>,
    error: Option<PipelineError<E>>,
}

impl<O, E> Coordinator<O, E> {
    fn new() -> Self {
        Self {
            order: VecDeque::new(),
            outstanding: BTreeSet::new(),
            outputs: BTreeMap::new(),
            terminal: BTreeSet::new(),
            error: None,
        }
    }

    fn submit(&mut self, sequence: u64) {
        self.order.push_back(sequence);
        self.outstanding.insert(sequence);
    }

    fn failed(&self) -> bool {
        self.error.is_some()
    }

    fn outstanding_len(&self) -> usize {
        self.outstanding.len()
    }

    fn fail(&mut self, error: PipelineError<E>, cancellation: &CancellationToken) {
        if self.error.is_none() {
            self.error = Some(error);
            cancellation.cancel();
        }
    }

    fn fail_without_cancel(&mut self, error: PipelineError<E>) {
        if self.error.is_none() {
            self.error = Some(error);
        }
    }

    fn fail_if_empty(&mut self, error: PipelineError<E>, cancellation: &CancellationToken) {
        if self.error.is_none() {
            self.fail(error, cancellation);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn receive_one<Emit>(
        &mut self,
        receiver: &Receiver<Completion<O, E>>,
        emit: &mut Emit,
        counters: &Counters,
        slots: &SlotTracker,
        start: Duration,
        clock: &dyn Clock,
        cancellation: &CancellationToken,
    ) where
        Emit: FnMut(u64, O) -> Result<(), E>,
    {
        match receiver.recv() {
            Ok(completion) => {
                self.accept(
                    completion,
                    emit,
                    counters,
                    slots,
                    start,
                    clock,
                    cancellation,
                );
            }
            Err(_) => {
                self.fail(
                    PipelineError::StageDisconnected {
                        sequence: self.order.front().copied(),
                        stage: PipelineStage::Convert,
                    },
                    cancellation,
                );
                self.abandon();
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn receive_ready<Emit>(
        &mut self,
        receiver: &Receiver<Completion<O, E>>,
        emit: &mut Emit,
        counters: &Counters,
        slots: &SlotTracker,
        start: Duration,
        clock: &dyn Clock,
        cancellation: &CancellationToken,
    ) where
        Emit: FnMut(u64, O) -> Result<(), E>,
    {
        loop {
            match receiver.try_recv() {
                Ok(completion) => {
                    self.accept(
                        completion,
                        emit,
                        counters,
                        slots,
                        start,
                        clock,
                        cancellation,
                    );
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    if !self.outstanding.is_empty() {
                        self.fail(
                            PipelineError::StageDisconnected {
                                sequence: self.order.front().copied(),
                                stage: PipelineStage::Convert,
                            },
                            cancellation,
                        );
                        self.abandon();
                    }
                    break;
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn drain<Emit>(
        &mut self,
        receiver: &Receiver<Completion<O, E>>,
        emit: &mut Emit,
        counters: &Counters,
        slots: &SlotTracker,
        start: Duration,
        clock: &dyn Clock,
        cancellation: &CancellationToken,
    ) where
        Emit: FnMut(u64, O) -> Result<(), E>,
    {
        while !self.outstanding.is_empty() {
            self.receive_one(receiver, emit, counters, slots, start, clock, cancellation);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn accept<Emit>(
        &mut self,
        completion: Completion<O, E>,
        emit: &mut Emit,
        counters: &Counters,
        _slots: &SlotTracker,
        start: Duration,
        clock: &dyn Clock,
        cancellation: &CancellationToken,
    ) where
        Emit: FnMut(u64, O) -> Result<(), E>,
    {
        match completion {
            Completion::Output(work) => {
                if self.failed() || cancellation.is_cancelled() {
                    self.terminal.insert(work.sequence);
                    self.outstanding.remove(&work.sequence);
                    drop(work);
                } else {
                    self.outputs.insert(work.sequence, work);
                }
            }
            Completion::Error {
                sequence,
                stage,
                source,
                permit,
            } => {
                drop(permit);
                self.terminal.insert(sequence);
                self.outstanding.remove(&sequence);
                self.fail(
                    PipelineError::Stage {
                        sequence: Some(sequence),
                        stage,
                        source,
                    },
                    cancellation,
                );
            }
            Completion::Panicked {
                sequence,
                stage,
                permit,
            } => {
                drop(permit);
                self.terminal.insert(sequence);
                self.outstanding.remove(&sequence);
                self.fail(
                    PipelineError::CallbackPanicked {
                        sequence: Some(sequence),
                        stage,
                    },
                    cancellation,
                );
            }
            Completion::Disconnected {
                sequence,
                stage,
                permit,
            } => {
                drop(permit);
                self.terminal.insert(sequence);
                self.outstanding.remove(&sequence);
                self.fail(
                    PipelineError::StageDisconnected {
                        sequence: Some(sequence),
                        stage,
                    },
                    cancellation,
                );
            }
            Completion::Cancelled { sequence, permit } => {
                drop(permit);
                self.terminal.insert(sequence);
                self.outstanding.remove(&sequence);
            }
        }
        self.emit_ready(emit, counters, start, clock, cancellation);
    }

    fn emit_ready<Emit>(
        &mut self,
        emit: &mut Emit,
        counters: &Counters,
        start: Duration,
        clock: &dyn Clock,
        cancellation: &CancellationToken,
    ) where
        Emit: FnMut(u64, O) -> Result<(), E>,
    {
        while let Some(&sequence) = self.order.front() {
            if self.terminal.remove(&sequence) {
                self.order.pop_front();
                continue;
            }
            let Some(work) = self.outputs.remove(&sequence) else {
                break;
            };
            self.order.pop_front();
            if self.failed() || cancellation.is_cancelled() {
                self.outstanding.remove(&sequence);
                drop(work);
                continue;
            }

            let Work { value, permit, .. } = work;
            let started = clock.monotonic();
            let result = catch_unwind(AssertUnwindSafe(|| emit(sequence, value)));
            let elapsed = elapsed_since(clock, started);
            add_duration(&counters.emit_ns, elapsed);
            counters.profile.record_span(
                counters.profile_path.with_frame(sequence),
                ProfilePhase::Emit,
                ProfileLane::caller(),
                started,
                elapsed,
            );
            drop(permit);
            self.outstanding.remove(&sequence);
            match result {
                Ok(Ok(())) => {
                    counters.emitted.fetch_add(1, Ordering::Relaxed);
                    let first = duration_nanos(elapsed_since(clock, start)).max(1);
                    let _ = counters.first_output_ns.compare_exchange(
                        0,
                        first,
                        Ordering::Relaxed,
                        Ordering::Relaxed,
                    );
                }
                Ok(Err(source)) => {
                    self.fail(
                        PipelineError::Stage {
                            sequence: Some(sequence),
                            stage: PipelineStage::Emit,
                            source,
                        },
                        cancellation,
                    );
                }
                Err(_) => {
                    self.fail(
                        PipelineError::CallbackPanicked {
                            sequence: Some(sequence),
                            stage: PipelineStage::Emit,
                        },
                        cancellation,
                    );
                }
            }
        }
    }

    fn abandon(&mut self) {
        self.outputs.clear();
        self.terminal.clear();
        self.outstanding.clear();
        self.order.clear();
    }
}

struct SlotTracker {
    limit: usize,
    outstanding: AtomicUsize,
    max: AtomicUsize,
}

impl SlotTracker {
    const fn new(limit: usize) -> Self {
        Self {
            limit,
            outstanding: AtomicUsize::new(0),
            max: AtomicUsize::new(0),
        }
    }

    fn outstanding(&self) -> usize {
        self.outstanding.load(Ordering::Acquire)
    }

    fn observe(&self, current: usize) {
        debug_assert!(current <= self.limit);
        let mut seen = self.max.load(Ordering::Relaxed);
        while current > seen {
            match self.max.compare_exchange_weak(
                seen,
                current,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => seen = actual,
            }
        }
    }
}

struct SlotPermit {
    tracker: Arc<SlotTracker>,
}

impl SlotPermit {
    fn new(tracker: Arc<SlotTracker>) -> Self {
        let current = tracker.outstanding.fetch_add(1, Ordering::AcqRel) + 1;
        tracker.observe(current);
        Self { tracker }
    }
}

impl Drop for SlotPermit {
    fn drop(&mut self) {
        let previous = self.tracker.outstanding.fetch_sub(1, Ordering::AcqRel);
        debug_assert!(previous > 0);
    }
}

struct Counters {
    submitted: AtomicU64,
    prepared: AtomicU64,
    rasterized: AtomicU64,
    converted: AtomicU64,
    emitted: AtomicU64,
    barriers: AtomicU64,
    backpressure_waits: AtomicU64,
    scene_jobs: AtomicU64,
    scene_ns: AtomicU64,
    prepare_ns: AtomicU64,
    raster_ns: AtomicU64,
    convert_ns: AtomicU64,
    emit_ns: AtomicU64,
    barrier_ns: AtomicU64,
    first_output_ns: AtomicU64,
    render_team_frames: Vec<AtomicU64>,
    render_team_ns: Vec<AtomicU64>,
    profile: ProfileRecorder,
    profile_path: ProfilePath,
}

impl Counters {
    fn new(render_teams: usize, profile: ProfileRecorder, profile_path: ProfilePath) -> Self {
        Self {
            submitted: AtomicU64::new(0),
            prepared: AtomicU64::new(0),
            rasterized: AtomicU64::new(0),
            converted: AtomicU64::new(0),
            emitted: AtomicU64::new(0),
            barriers: AtomicU64::new(0),
            backpressure_waits: AtomicU64::new(0),
            scene_jobs: AtomicU64::new(0),
            scene_ns: AtomicU64::new(0),
            prepare_ns: AtomicU64::new(0),
            raster_ns: AtomicU64::new(0),
            convert_ns: AtomicU64::new(0),
            emit_ns: AtomicU64::new(0),
            barrier_ns: AtomicU64::new(0),
            first_output_ns: AtomicU64::new(0),
            render_team_frames: (0..render_teams).map(|_| AtomicU64::new(0)).collect(),
            render_team_ns: (0..render_teams).map(|_| AtomicU64::new(0)).collect(),
            profile,
            profile_path,
        }
    }

    fn snapshot(
        &self,
        elapsed: Duration,
        slots: &SlotTracker,
        plan: &ExecutionPlan,
    ) -> PipelineStats {
        let first = self.first_output_ns.load(Ordering::Relaxed);
        let stats = PipelineStats {
            elapsed,
            first_output_latency: (first != 0).then(|| Duration::from_nanos(first)),
            submitted: self.submitted.load(Ordering::Relaxed),
            prepared: self.prepared.load(Ordering::Relaxed),
            rasterized: self.rasterized.load(Ordering::Relaxed),
            converted: self.converted.load(Ordering::Relaxed),
            emitted: self.emitted.load(Ordering::Relaxed),
            barriers: self.barriers.load(Ordering::Relaxed),
            backpressure_waits: self.backpressure_waits.load(Ordering::Relaxed),
            max_in_flight: slots.max.load(Ordering::Relaxed),
            outstanding_slots: slots.outstanding(),
            scene: StageUtilization {
                jobs: self.scene_jobs.load(Ordering::Relaxed),
                busy: Duration::from_nanos(self.scene_ns.load(Ordering::Relaxed)),
                // `events.next()` runs on exactly one caller thread. The
                // advisory scene team may help work *inside* a callback, but
                // using its width here understated the measured occupancy.
                workers: 1,
            },
            prepare: StageUtilization {
                jobs: self.prepared.load(Ordering::Relaxed),
                busy: Duration::from_nanos(self.prepare_ns.load(Ordering::Relaxed)),
                workers: 1,
            },
            raster: StageUtilization {
                jobs: self.rasterized.load(Ordering::Relaxed),
                busy: Duration::from_nanos(self.raster_ns.load(Ordering::Relaxed)),
                workers: plan.render_teams.len().max(1),
            },
            convert: StageUtilization {
                jobs: self.converted.load(Ordering::Relaxed),
                busy: Duration::from_nanos(self.convert_ns.load(Ordering::Relaxed)),
                workers: 1,
            },
            emit: StageUtilization {
                jobs: self.emitted.load(Ordering::Relaxed),
                busy: Duration::from_nanos(self.emit_ns.load(Ordering::Relaxed)),
                workers: 1,
            },
            barrier: StageUtilization {
                jobs: self.barriers.load(Ordering::Relaxed),
                busy: Duration::from_nanos(self.barrier_ns.load(Ordering::Relaxed)),
                workers: 1,
            },
            render_team_frames: self
                .render_team_frames
                .iter()
                .map(|counter| counter.load(Ordering::Relaxed))
                .collect(),
            render_team_busy: self
                .render_team_ns
                .iter()
                .map(|counter| Duration::from_nanos(counter.load(Ordering::Relaxed)))
                .collect(),
        };
        self.record_profile_counters(&stats);
        stats
    }

    fn record_profile_counters(&self, stats: &PipelineStats) {
        let path = self.profile_path;
        let caller = ProfileLane::caller();
        for (counter, value) in [
            (ProfileCounter::SubmittedFrames, stats.submitted),
            (ProfileCounter::EmittedFrames, stats.emitted),
            (ProfileCounter::BackpressureWaits, stats.backpressure_waits),
            (
                ProfileCounter::MaxInFlight,
                u64::try_from(stats.max_in_flight).unwrap_or(u64::MAX),
            ),
        ] {
            self.profile.record_counter(path, counter, caller, value);
        }

        let capacity = duration_nanos(stats.elapsed);
        for (index, busy) in stats.render_team_busy.iter().copied().enumerate() {
            let lane = ProfileLane::render(u16::try_from(index).unwrap_or(u16::MAX));
            self.profile.record_counter(
                path,
                ProfileCounter::RenderTeamBusyNs,
                lane,
                duration_nanos(busy),
            );
            self.profile
                .record_counter(path, ProfileCounter::RenderTeamCapacityNs, lane, capacity);
        }
    }
}

fn add_duration(counter: &AtomicU64, duration: Duration) {
    counter.fetch_add(duration_nanos(duration), Ordering::Relaxed);
}

fn elapsed_since(clock: &dyn Clock, start: Duration) -> Duration {
    clock.monotonic().saturating_sub(start)
}

fn duration_nanos(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::{ExecutionPlan, OutputPixelFormat, PlanRequest, RenderIntent, SurfaceSpec};
    use fmn_platform::clock::FakeClock;
    use fmn_platform::topology::HardwareTopology;

    #[derive(Debug)]
    struct Arithmetic;

    impl PipelineStages for Arithmetic {
        type Frame = u64;
        type Prepared = u64;
        type Rasterized = Vec<u8>;
        type Output = Vec<u8>;
        type Error = &'static str;

        fn prepare(&self, frame: u64, _team: &TeamPlan) -> Result<u64, Self::Error> {
            Ok(frame.wrapping_mul(17).wrapping_add(3))
        }

        fn rasterize(&self, prepared: u64, team: &TeamPlan) -> Result<Vec<u8>, Self::Error> {
            // Fixed work with schedule-dependent yielding: output deliberately
            // ignores team identity.
            for _ in 0..(prepared % 5) {
                std::thread::yield_now();
            }
            let mut output = prepared.to_le_bytes().to_vec();
            output.push(u8::from(team.threads() > 0));
            Ok(output)
        }

        fn convert(
            &self,
            mut rasterized: Vec<u8>,
            _team: &TeamPlan,
        ) -> Result<Vec<u8>, Self::Error> {
            rasterized.reverse();
            Ok(rasterized)
        }
    }

    fn plan(depth: usize, intent: RenderIntent) -> ExecutionPlan {
        let mut topology = HardwareTopology::fallback(96);
        topology.total_memory_bytes = Some(128 * 1024 * 1024 * 1024);
        ExecutionPlan::derive(
            PlanRequest::standard(intent, SurfaceSpec::lumen(64, 64), OutputPixelFormat::Rgba8)
                .with_max_frames_in_flight(depth),
            &topology,
            None,
        )
        .expect("plan")
    }

    fn expected(frame: u64) -> Vec<u8> {
        let prepared = frame.wrapping_mul(17).wrapping_add(3);
        let mut output = prepared.to_le_bytes().to_vec();
        output.push(1);
        output.reverse();
        output
    }

    #[derive(Default)]
    struct StartupObservedStages {
        calls: AtomicUsize,
    }

    impl PipelineStages for StartupObservedStages {
        type Frame = u64;
        type Prepared = u64;
        type Rasterized = u64;
        type Output = u64;
        type Error = &'static str;

        fn prepare(&self, frame: u64, _team: &TeamPlan) -> Result<u64, Self::Error> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            Ok(frame)
        }

        fn rasterize(&self, frame: u64, _team: &TeamPlan) -> Result<u64, Self::Error> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            Ok(frame)
        }

        fn convert(&self, frame: u64, _team: &TeamPlan) -> Result<u64, Self::Error> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            Ok(frame)
        }
    }

    struct RefusingScopedSpawner {
        refuse_at: usize,
        attempts: AtomicUsize,
    }

    impl RefusingScopedSpawner {
        const fn new(refuse_at: usize) -> Self {
            Self {
                refuse_at,
                attempts: AtomicUsize::new(0),
            }
        }

        fn attempts(&self) -> usize {
            self.attempts.load(Ordering::Relaxed)
        }
    }

    impl ScopedSpawner for RefusingScopedSpawner {
        fn spawn<'scope, 'env: 'scope, F>(
            &self,
            scope: &'scope std::thread::Scope<'scope, 'env>,
            work: F,
        ) -> std::io::Result<std::thread::ScopedJoinHandle<'scope, ()>>
        where
            F: FnOnce() + Send + 'scope,
        {
            let attempt = self.attempts.fetch_add(1, Ordering::Relaxed);
            if attempt == self.refuse_at {
                return Err(std::io::ErrorKind::WouldBlock.into());
            }
            NativeScopedSpawner.spawn(scope, work)
        }
    }

    fn assert_startup_refusal(
        refuse_at: usize,
        expected_stage: PipelineStage,
        expected_spawned: usize,
    ) {
        let plan = plan(3, RenderIntent::Offline);
        assert!(plan.render_teams.len() >= 2);
        let requested = plan.render_teams.len().saturating_add(2);
        let stages = StartupObservedStages::default();
        let pipeline = FramePipeline::new(&plan, &stages);
        let source_calls = AtomicUsize::new(0);
        let emit_calls = AtomicUsize::new(0);
        let barrier_calls = AtomicUsize::new(0);
        let events = std::iter::from_fn(|| {
            source_calls.fetch_add(1, Ordering::Relaxed);
            None::<PipelineEvent<u64, ()>>
        });
        let spawner = RefusingScopedSpawner::new(refuse_at);

        let failure = pipeline
            .run_with_spawner(
                events,
                |_, _| {
                    emit_calls.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                },
                |(), _| {
                    barrier_calls.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                },
                &spawner,
            )
            .expect_err("worker startup must fail");

        assert_eq!(spawner.attempts(), expected_spawned.saturating_add(1));
        assert_eq!(source_calls.load(Ordering::Relaxed), 0);
        assert_eq!(emit_calls.load(Ordering::Relaxed), 0);
        assert_eq!(barrier_calls.load(Ordering::Relaxed), 0);
        assert_eq!(stages.calls.load(Ordering::Relaxed), 0);
        assert_eq!(failure.stats.submitted, 0);
        assert_eq!(failure.stats.prepared, 0);
        assert_eq!(failure.stats.rasterized, 0);
        assert_eq!(failure.stats.converted, 0);
        assert_eq!(failure.stats.emitted, 0);
        assert_eq!(failure.stats.barriers, 0);
        assert_eq!(failure.stats.outstanding_slots, 0);
        assert_eq!(
            failure.to_string(),
            format!(
                "{expected_stage} worker startup failed after {expected_spawned} of {requested} pipeline workers started"
            )
        );
        assert!(matches!(
            failure.error,
            PipelineError::WorkerSpawnFailed {
                stage,
                requested: actual_requested,
                spawned,
            } if stage == expected_stage
                && actual_requested == requested
                && spawned == expected_spawned
        ));
        assert!(pipeline.cancellation_token().is_cancelled());
    }

    #[test]
    fn output_worker_start_refusal_is_typed_and_atomic() {
        assert_startup_refusal(0, PipelineStage::Convert, 0);
    }

    #[test]
    fn intermediate_raster_worker_start_refusal_is_typed_and_atomic() {
        assert_startup_refusal(2, PipelineStage::Raster, 2);
    }

    #[test]
    fn final_prepare_worker_start_refusal_is_typed_and_atomic() {
        let plan = plan(3, RenderIntent::Offline);
        let requested = plan.render_teams.len().saturating_add(2);
        assert_startup_refusal(
            requested.saturating_sub(1),
            PipelineStage::Prepare,
            requested.saturating_sub(1),
        );
    }

    #[test]
    fn every_queue_depth_matches_serial_bytes_and_order() {
        for depth in 1..=6 {
            let plan = plan(depth, RenderIntent::Offline);
            let stages = Arithmetic;
            let pipeline = FramePipeline::new(&plan, &stages);
            let events = (0..32).map(|sequence| PipelineEvent::<_, ()>::frame(sequence, sequence));
            let mut emitted = Vec::new();
            let stats = pipeline
                .run(
                    events,
                    |sequence, output| {
                        emitted.push((sequence, output));
                        Ok(())
                    },
                    |(), _| Ok(()),
                )
                .expect("pipeline");
            let serial: Vec<_> = (0..32).map(|frame| (frame, expected(frame))).collect();
            assert_eq!(emitted, serial, "queue depth {depth}");
            assert_eq!(stats.emitted, 32);
            assert!(stats.max_in_flight <= depth);
            assert_eq!(stats.outstanding_slots, 0);
        }
    }

    struct TrackedFrame {
        live: Arc<AtomicUsize>,
    }

    impl TrackedFrame {
        fn new(live: Arc<AtomicUsize>, maximum: &AtomicUsize) -> Self {
            let current = live.fetch_add(1, Ordering::AcqRel) + 1;
            maximum.fetch_max(current, Ordering::AcqRel);
            Self { live }
        }
    }

    impl Drop for TrackedFrame {
        fn drop(&mut self) {
            self.live.fetch_sub(1, Ordering::AcqRel);
        }
    }

    struct RetainsTrackedFrame;

    impl PipelineStages for RetainsTrackedFrame {
        type Frame = TrackedFrame;
        type Prepared = TrackedFrame;
        type Rasterized = TrackedFrame;
        type Output = TrackedFrame;
        type Error = &'static str;

        fn prepare(
            &self,
            frame: TrackedFrame,
            _team: &TeamPlan,
        ) -> Result<TrackedFrame, Self::Error> {
            // Keep the first frozen source value alive long enough that an
            // eager `(limit + 1)`th iterator advance is deterministic.
            std::thread::sleep(Duration::from_millis(15));
            Ok(frame)
        }

        fn rasterize(
            &self,
            frame: TrackedFrame,
            _team: &TeamPlan,
        ) -> Result<TrackedFrame, Self::Error> {
            Ok(frame)
        }

        fn convert(
            &self,
            frame: TrackedFrame,
            _team: &TeamPlan,
        ) -> Result<TrackedFrame, Self::Error> {
            Ok(frame)
        }
    }

    #[test]
    fn frozen_source_values_are_inside_the_global_slot_budget() {
        let plan = plan(1, RenderIntent::Preview);
        let live = Arc::new(AtomicUsize::new(0));
        let maximum = Arc::new(AtomicUsize::new(0));
        let events = (0..3).map({
            let live = Arc::clone(&live);
            let maximum = Arc::clone(&maximum);
            move |sequence| {
                PipelineEvent::<_, ()>::frame(
                    sequence,
                    TrackedFrame::new(Arc::clone(&live), &maximum),
                )
            }
        });
        let stages = RetainsTrackedFrame;
        let stats = FramePipeline::new(&plan, &stages)
            .run(events, |_, _| Ok(()), |(), _| Ok(()))
            .expect("pipeline");
        assert_eq!(stats.max_in_flight, 1);
        assert_eq!(maximum.load(Ordering::Acquire), 1);
        assert_eq!(live.load(Ordering::Acquire), 0);
    }

    #[test]
    fn pixel_observing_barrier_sees_a_complete_drain() {
        let plan = plan(3, RenderIntent::Offline);
        let stages = Arithmetic;
        let pipeline = FramePipeline::new(&plan, &stages);
        let emitted = AtomicU64::new(0);
        let events = vec![
            PipelineEvent::frame(0, 0),
            PipelineEvent::frame(1, 1),
            PipelineEvent::barrier("pixels"),
            PipelineEvent::frame(2, 2),
        ];
        let stats = pipeline
            .run(
                events,
                |_, _| {
                    emitted.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                },
                |name, context| {
                    assert_eq!(name, "pixels");
                    assert_eq!(context.submitted, 2);
                    assert_eq!(context.emitted, 2);
                    assert_eq!(context.outstanding_slots, 0);
                    assert_eq!(emitted.load(Ordering::Relaxed), 2);
                    Ok(())
                },
            )
            .expect("pipeline");
        assert_eq!(stats.barriers, 1);
        assert_eq!(stats.emitted, 3);
    }

    struct Cancels {
        token: CancellationToken,
    }

    impl PipelineStages for Cancels {
        type Frame = u64;
        type Prepared = u64;
        type Rasterized = u64;
        type Output = u64;
        type Error = &'static str;

        fn prepare(&self, frame: u64, _team: &TeamPlan) -> Result<u64, Self::Error> {
            Ok(frame)
        }

        fn rasterize(&self, frame: u64, _team: &TeamPlan) -> Result<u64, Self::Error> {
            if frame == 3 {
                self.token.cancel();
            }
            Ok(frame)
        }

        fn convert(&self, frame: u64, _team: &TeamPlan) -> Result<u64, Self::Error> {
            Ok(frame)
        }
    }

    #[test]
    fn cancellation_drains_and_releases_every_slot() {
        let plan = plan(4, RenderIntent::Offline);
        let token = CancellationToken::new();
        let stages = Cancels {
            token: token.clone(),
        };
        let pipeline = FramePipeline::with_cancellation(&plan, &stages, token);
        let events = (0..64).map(|sequence| PipelineEvent::<_, ()>::frame(sequence, sequence));
        let failure = pipeline
            .run(events, |_, _| Ok(()), |(), _| Ok(()))
            .expect_err("cancelled");
        assert!(matches!(failure.error, PipelineError::Cancelled));
        assert_eq!(failure.stats.outstanding_slots, 0);
        assert!(failure.stats.submitted < 64);
    }

    #[test]
    fn cancellation_after_emit_skips_the_following_barrier() {
        let plan = plan(2, RenderIntent::Offline);
        let token = CancellationToken::new();
        let stages = Arithmetic;
        let pipeline = FramePipeline::with_cancellation(&plan, &stages, token.clone());
        let barrier_ran = AtomicBool::new(false);
        let events = [
            PipelineEvent::frame(0, 0),
            PipelineEvent::barrier("must-not-run"),
        ];
        let failure = pipeline
            .run(
                events,
                |_, _| {
                    token.cancel();
                    Ok(())
                },
                |_, _| {
                    barrier_ran.store(true, Ordering::Relaxed);
                    Ok(())
                },
            )
            .expect_err("cancelled");
        assert!(matches!(failure.error, PipelineError::Cancelled));
        assert!(!barrier_ran.load(Ordering::Relaxed));
        assert_eq!(failure.stats.barriers, 0);
        assert_eq!(failure.stats.outstanding_slots, 0);
    }

    struct Panics;

    impl PipelineStages for Panics {
        type Frame = u64;
        type Prepared = u64;
        type Rasterized = u64;
        type Output = u64;
        type Error = &'static str;

        fn prepare(&self, frame: u64, _team: &TeamPlan) -> Result<u64, Self::Error> {
            Ok(frame)
        }

        fn rasterize(&self, frame: u64, _team: &TeamPlan) -> Result<u64, Self::Error> {
            assert_ne!(frame, 2, "deliberate fixture panic");
            Ok(frame)
        }

        fn convert(&self, frame: u64, _team: &TeamPlan) -> Result<u64, Self::Error> {
            Ok(frame)
        }
    }

    #[test]
    fn worker_panic_is_contained_and_slots_still_drain() {
        let plan = plan(3, RenderIntent::Offline);
        let stages = Panics;
        let pipeline = FramePipeline::new(&plan, &stages);
        let events = (0..8).map(|sequence| PipelineEvent::<_, ()>::frame(sequence, sequence));
        let failure = pipeline
            .run(events, |_, _| Ok(()), |(), _| Ok(()))
            .expect_err("panic");
        assert!(matches!(
            failure.error,
            PipelineError::CallbackPanicked {
                sequence: Some(2),
                stage: PipelineStage::Raster
            }
        ));
        assert_eq!(failure.stats.outstanding_slots, 0);
    }

    #[test]
    fn source_emit_and_barrier_panics_are_structured_and_drained() {
        let plan = plan(2, RenderIntent::Offline);
        let stages = Arithmetic;

        let mut source_step = 0;
        let source = std::iter::from_fn(move || {
            source_step += 1;
            match source_step {
                1 => Some(PipelineEvent::<_, ()>::frame(0, 0)),
                2 => panic!("deliberate source panic"),
                _ => None,
            }
        });
        let source_failure = FramePipeline::new(&plan, &stages)
            .run(source, |_, _| Ok(()), |(), _| Ok(()))
            .expect_err("source panic");
        assert!(matches!(
            source_failure.error,
            PipelineError::CallbackPanicked {
                sequence: None,
                stage: PipelineStage::Scene
            }
        ));
        assert_eq!(source_failure.stats.outstanding_slots, 0);

        let emit_failure = FramePipeline::new(&plan, &stages)
            .run(
                [PipelineEvent::<_, ()>::frame(0, 0)],
                |_, _| panic!("deliberate emit panic"),
                |(), _| Ok(()),
            )
            .expect_err("emit panic");
        assert!(matches!(
            emit_failure.error,
            PipelineError::CallbackPanicked {
                sequence: Some(0),
                stage: PipelineStage::Emit
            }
        ));
        assert_eq!(emit_failure.stats.outstanding_slots, 0);

        let barrier_failure = FramePipeline::new(&plan, &stages)
            .run(
                [PipelineEvent::frame(0, 0), PipelineEvent::barrier("pixels")],
                |_, _| Ok(()),
                |_, _| panic!("deliberate barrier panic"),
            )
            .expect_err("barrier panic");
        assert!(matches!(
            barrier_failure.error,
            PipelineError::CallbackPanicked {
                sequence: None,
                stage: PipelineStage::Barrier
            }
        ));
        assert_eq!(barrier_failure.stats.outstanding_slots, 0);
        assert_eq!(barrier_failure.stats.barriers, 0);
    }

    #[test]
    fn non_monotonic_sequences_fail_closed() {
        let plan = plan(2, RenderIntent::Offline);
        let stages = Arithmetic;
        let pipeline = FramePipeline::new(&plan, &stages);
        let events = vec![
            PipelineEvent::<_, ()>::frame(7, 7),
            PipelineEvent::frame(7, 8),
        ];
        let failure = pipeline
            .run(events, |_, _| Ok(()), |(), _| Ok(()))
            .expect_err("duplicate");
        assert!(matches!(
            failure.error,
            PipelineError::NonMonotonicSequence {
                previous: 7,
                next: 7
            }
        ));
        assert_eq!(failure.stats.outstanding_slots, 0);
    }

    #[test]
    fn utilization_and_preview_latency_are_measured() {
        for depth in 1..=2 {
            let plan = plan(depth, RenderIntent::Preview);
            let stages = Arithmetic;
            let pipeline = FramePipeline::new(&plan, &stages);
            let events = (0..24).map(|sequence| PipelineEvent::<_, ()>::frame(sequence, sequence));
            let stats = pipeline
                .run(events, |_, _| Ok(()), |(), _| Ok(()))
                .expect("pipeline");
            assert!(stats.first_output_latency.is_some());
            assert!(stats.max_in_flight <= depth);
            assert_eq!(
                stats.render_team_frames.iter().sum::<u64>(),
                stats.rasterized
            );
            assert_eq!(stats.raster.jobs, 24);
            assert!((0.0..=1.0).contains(&stats.raster.fraction(stats.elapsed)));
        }
    }

    struct Timed {
        clock: Arc<FakeClock>,
    }

    impl PipelineStages for Timed {
        type Frame = ();
        type Prepared = ();
        type Rasterized = ();
        type Output = ();
        type Error = &'static str;

        fn prepare(&self, (): (), _team: &TeamPlan) -> Result<(), Self::Error> {
            self.clock.advance(Duration::from_millis(1));
            Ok(())
        }

        fn rasterize(&self, (): (), _team: &TeamPlan) -> Result<(), Self::Error> {
            self.clock.advance(Duration::from_millis(2));
            Ok(())
        }

        fn convert(&self, (): (), _team: &TeamPlan) -> Result<(), Self::Error> {
            self.clock.advance(Duration::from_millis(3));
            Ok(())
        }
    }

    #[test]
    fn handed_in_clock_makes_latency_metrics_replayable() {
        let plan = plan(1, RenderIntent::Preview);
        let clock = Arc::new(FakeClock::new());
        let stages = Timed {
            clock: Arc::clone(&clock),
        };
        let clock_capability: Arc<dyn Clock> = clock;
        let profile = ProfileRecorder::enabled();
        let pipeline = FramePipeline::with_clock_and_profile(
            &plan,
            &stages,
            CancellationToken::new(),
            clock_capability,
            profile.clone(),
        )
        .with_profile_path(ProfilePath::scene(7).with_play(2));
        let stats = pipeline
            .run(
                [
                    PipelineEvent::<_, ()>::frame(0, ()),
                    PipelineEvent::barrier(()),
                ],
                |_, ()| {
                    stages.clock.advance(Duration::from_millis(4));
                    Ok(())
                },
                |(), _| {
                    stages.clock.advance(Duration::from_millis(5));
                    Ok(())
                },
            )
            .expect("pipeline");
        assert_eq!(stats.prepare.busy, Duration::from_millis(1));
        assert_eq!(stats.raster.busy, Duration::from_millis(2));
        assert_eq!(stats.convert.busy, Duration::from_millis(3));
        assert_eq!(stats.emit.busy, Duration::from_millis(4));
        assert_eq!(stats.barrier.busy, Duration::from_millis(5));
        assert_eq!(stats.barrier.jobs, 1);
        assert_eq!(stats.first_output_latency, Some(Duration::from_millis(10)));
        assert_eq!(stats.elapsed, Duration::from_millis(15));
        assert_eq!(stats.scene.workers, 1);

        let snapshot = profile.snapshot();
        let phases = snapshot
            .records()
            .iter()
            .filter_map(|record| match record {
                fmn_platform::profile::ProfileRecord::Span(span) => Some(span.phase),
                fmn_platform::profile::ProfileRecord::Counter(_) => None,
            })
            .collect::<BTreeSet<_>>();
        assert!(phases.contains(&ProfilePhase::SceneUpdate));
        assert!(phases.contains(&ProfilePhase::Prepare));
        assert!(phases.contains(&ProfilePhase::Raster));
        assert!(phases.contains(&ProfilePhase::ColorConversion));
        assert!(phases.contains(&ProfilePhase::Emit));
        assert!(phases.contains(&ProfilePhase::Barrier));
        assert!(
            snapshot
                .to_ndjson()
                .contains("\"schema\":\"fmn-profile/1\"")
        );
        assert!(
            snapshot
                .to_folded()
                .contains("scene:7;play:2;frame:0;phase:raster")
        );
    }
}
