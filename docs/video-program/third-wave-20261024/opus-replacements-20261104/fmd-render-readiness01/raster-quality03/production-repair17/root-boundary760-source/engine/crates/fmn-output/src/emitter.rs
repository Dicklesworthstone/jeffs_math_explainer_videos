//! Bounded, frame-index-ordered sink emission (§14.1, D-19).
//!
//! Dispatch reserves slots in sequence order before render work begins.
//! A reservation owns its [`FrameBuffer`] exclusively while a worker fills it;
//! publishing consumes that reservation, which is the visibility boundary that
//! makes torn frames impossible. Completion may happen in any order. A
//! dedicated drain thread exposes only the contiguous ready prefix to sinks.
//!
//! The ring owns exactly `capacity` preallocated frame buffers. It never grows
//! on the hot path. When every slot is outstanding, the next dispatch blocks
//! before receiving a buffer, so sink pressure propagates without letting later
//! frames occupy the slot needed by the drain's next sequence.
//!
//! Finalization prepares every sink without publication, then takes one commit
//! grant linearized against cancellation and commits in binding order. A
//! prepare failure aborts every sink. A commit failure aborts the failing and
//! later sinks; earlier sinks may already have published, so this is ordered
//! multi-sink delivery, not a cross-sink filesystem transaction. Composition
//! that needs one durable artifact plus previews should bind the durable
//! publisher last.

use fmn_frame::{FrameBuffer, FrameLayout, FramePool};
use fmn_platform::process::ProcessCancellation;
use std::fmt;
use std::mem;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::thread::{self, JoinHandle};

/// Result of offering one frame to a sink.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SinkWrite {
    /// The sink consumed the frame.
    Consumed,
    /// A non-blocking preview sink is busy.
    ///
    /// Reliable sinks must wait internally and return [`Self::Consumed`].
    WouldBlock,
}

/// A sink-owned failure, normalized at the Reel boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SinkFailure {
    code: Arc<str>,
    message: Arc<str>,
}

impl SinkFailure {
    /// Construct a failure with a stable human-readable message.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self::with_code("sink.failure", message)
    }

    /// Construct a failure with a stable machine-readable category.
    #[must_use]
    pub fn with_code(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: Arc::from(code.into()),
            message: Arc::from(message.into()),
        }
    }

    /// Stable machine-readable failure category.
    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }

    /// The sink-provided message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for SinkFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for SinkFailure {}

impl From<std::io::Error> for SinkFailure {
    fn from(value: std::io::Error) -> Self {
        Self::with_code("sink.io", value.to_string())
    }
}

/// One ordered frame consumer.
///
/// `write_frame` runs on the emitter's dedicated drain thread. A reliable sink
/// applies backpressure by blocking inside that call. A preview sink configured
/// for explicit dropping may instead return [`SinkWrite::WouldBlock`].
pub trait FrameSink: Send + 'static {
    /// Attach the emitter's cooperative cancellation token before the drain
    /// thread starts.
    ///
    /// Process-backed sinks pass this token into their long-lived subprocess
    /// supervisor so cancellation can interrupt a blocked stdin write.
    fn set_cancellation(&mut self, _cancellation: ProcessCancellation) {}

    /// Consume one complete immutable frame.
    fn write_frame(&mut self, sequence: u64, frame: &FrameBuffer)
    -> Result<SinkWrite, SinkFailure>;

    /// Flush/end the stream after every accepted frame was drained.
    ///
    /// This is the only flush point in the abstraction; no per-frame
    /// synchronous flush is implied.
    fn finish(&mut self) -> Result<(), SinkFailure> {
        Ok(())
    }

    /// Complete expensive/private work without publishing a destination.
    ///
    /// The emitter runs this phase while cancellation may still win. The
    /// default has no preparation; legacy one-shot sinks run entirely in
    /// [`Self::commit_finish`] under the commit grant.
    fn prepare_finish(&mut self) -> Result<(), SinkFailure> {
        Ok(())
    }

    /// Publish prepared work after the emitter has granted commit.
    ///
    /// The default delegates to [`Self::finish`]. The emitter linearizes
    /// cancellation before calling this method.
    fn commit_finish(&mut self) -> Result<(), SinkFailure> {
        self.finish()
    }

    /// Abandon buffered work after the emitter has failed or been cancelled.
    ///
    /// This hook must not publish an artifact. It exists separately from
    /// [`Self::finish`] so a buffered durable sink cannot mistake a partial
    /// stream for a successful render. Panics are contained by the emitter,
    /// and the stream's original terminal failure remains authoritative.
    fn abort(&mut self) {}
}

/// Outcome of a cooperative cancellation request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CancelOutcome {
    /// Cancellation won before publication was granted.
    Won,
    /// The stream was already cancelled or had already failed.
    AlreadyTerminal,
    /// Publication had already been granted; cancellation did not take effect.
    TooLate,
}

impl<F> FrameSink for F
where
    F: FnMut(u64, &FrameBuffer) -> Result<SinkWrite, SinkFailure> + Send + 'static,
{
    fn write_frame(
        &mut self,
        sequence: u64,
        frame: &FrameBuffer,
    ) -> Result<SinkWrite, SinkFailure> {
        self(sequence, frame)
    }
}

/// Backpressure/drop behavior for one sink.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SinkMode {
    /// Every frame must be consumed. `WouldBlock` is a contract violation.
    Reliable,
    /// A preview-class sink may explicitly decline a busy frame.
    PreviewDrop,
}

/// A named sink and its delivery policy.
pub struct SinkBinding {
    name: Arc<str>,
    mode: SinkMode,
    sink: Box<dyn FrameSink>,
}

impl SinkBinding {
    /// Bind a durable sink. It must consume every offered frame.
    #[must_use]
    pub fn reliable(name: impl Into<String>, sink: impl FrameSink) -> Self {
        Self {
            name: Arc::from(name.into()),
            mode: SinkMode::Reliable,
            sink: Box::new(sink),
        }
    }

    /// Bind an explicitly lossy preview-class sink.
    #[must_use]
    pub fn preview_dropping(name: impl Into<String>, sink: impl FrameSink) -> Self {
        Self {
            name: Arc::from(name.into()),
            mode: SinkMode::PreviewDrop,
            sink: Box::new(sink),
        }
    }

    /// Stable sink name used in reports and errors.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Delivery policy.
    #[must_use]
    pub const fn mode(&self) -> SinkMode {
        self.mode
    }
}

impl fmt::Debug for SinkBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SinkBinding")
            .field("name", &self.name)
            .field("mode", &self.mode)
            .finish_non_exhaustive()
    }
}

/// Preallocation and sequence contract for an [`OrderedEmitter`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmitterConfig {
    layout: FrameLayout,
    capacity: usize,
    first_sequence: u64,
    ring_bytes: usize,
}

impl EmitterConfig {
    /// Validate a real, finite ring budget.
    ///
    /// # Errors
    ///
    /// Returns [`EmitterError::ZeroCapacity`] for an empty ring and
    /// [`EmitterError::RingBudgetOverflow`] when the byte budget cannot be
    /// represented.
    pub fn new(
        layout: FrameLayout,
        capacity: usize,
        first_sequence: u64,
    ) -> Result<Self, EmitterError> {
        if capacity == 0 {
            return Err(EmitterError::ZeroCapacity);
        }
        let ring_bytes =
            layout
                .total_bytes()
                .checked_mul(capacity)
                .ok_or(EmitterError::RingBudgetOverflow {
                    frame_bytes: layout.total_bytes(),
                    capacity,
                })?;
        Ok(Self {
            layout,
            capacity,
            first_sequence,
            ring_bytes,
        })
    }

    /// Layout shared by every slot.
    #[must_use]
    pub const fn layout(&self) -> &FrameLayout {
        &self.layout
    }

    /// Maximum number of outstanding frames.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    /// First accepted frame index.
    #[must_use]
    pub const fn first_sequence(&self) -> u64 {
        self.first_sequence
    }

    /// Bytes in one frame slot.
    #[must_use]
    pub const fn frame_bytes(&self) -> usize {
        self.layout.total_bytes()
    }

    /// Exact preallocated frame-storage budget.
    #[must_use]
    pub const fn ring_bytes(&self) -> usize {
        self.ring_bytes
    }
}

/// Typed emitter refusal or terminal failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmitterError {
    /// A ring must hold at least one frame.
    ZeroCapacity,
    /// `frame_bytes * capacity` overflowed `usize`.
    RingBudgetOverflow {
        /// Bytes in one frame.
        frame_bytes: usize,
        /// Requested slots.
        capacity: usize,
    },
    /// A render with no sink would silently discard its output.
    NoSinks,
    /// Sink names are required for actionable diagnostics.
    EmptySinkName {
        /// Position in the configured sink list.
        index: usize,
    },
    /// Reservations must be made in contiguous frame order.
    UnexpectedSequence {
        /// Next sequence the ring can reserve.
        expected: u64,
        /// Sequence requested by the caller.
        actual: u64,
    },
    /// The sequence domain has no representable successor.
    SequenceExhausted {
        /// Refused terminal sequence.
        sequence: u64,
    },
    /// No new reservations are accepted after finish begins.
    Closed,
    /// Cooperative cancellation stopped the stream.
    Cancelled,
    /// A worker discarded an unpublished reservation.
    ReservationAbandoned {
        /// Frame whose exclusive write never published.
        sequence: u64,
    },
    /// A reliable sink returned the preview-only busy signal.
    ReliableSinkWouldBlock {
        /// Configured sink name.
        sink: Arc<str>,
        /// Frame being delivered.
        sequence: u64,
    },
    /// A sink returned an error.
    SinkFailed {
        /// Configured sink name.
        sink: Arc<str>,
        /// Frame sequence, or `None` while finishing the stream.
        sequence: Option<u64>,
        /// Sink-owned detail.
        source: SinkFailure,
    },
    /// A sink callback panicked; the panic was contained.
    SinkPanicked {
        /// Configured sink name.
        sink: Arc<str>,
        /// Frame sequence, or `None` while finishing the stream.
        sequence: Option<u64>,
    },
    /// The drain thread could not be created.
    ThreadSpawn {
        /// Operating-system diagnostic.
        message: Arc<str>,
    },
    /// The drain thread panicked outside a contained sink callback.
    WorkerPanicked,
    /// A private slot-state invariant failed closed.
    InternalInvariant {
        /// Static invariant description.
        message: &'static str,
        /// Related frame when known.
        sequence: Option<u64>,
    },
}

impl fmt::Display for EmitterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroCapacity => f.write_str("ordered emitter capacity must be nonzero"),
            Self::RingBudgetOverflow {
                frame_bytes,
                capacity,
            } => write!(
                f,
                "ordered emitter ring budget overflows: {capacity} slots at {frame_bytes} bytes"
            ),
            Self::NoSinks => f.write_str("ordered emitter requires at least one sink"),
            Self::EmptySinkName { index } => {
                write!(f, "ordered emitter sink {index} has an empty name")
            }
            Self::UnexpectedSequence { expected, actual } => write!(
                f,
                "ordered emitter expected reservation {expected}, got {actual}"
            ),
            Self::SequenceExhausted { sequence } => {
                write!(
                    f,
                    "frame sequence {sequence} has no representable successor"
                )
            }
            Self::Closed => f.write_str("ordered emitter is closed"),
            Self::Cancelled => f.write_str("ordered emitter was cancelled"),
            Self::ReservationAbandoned { sequence } => {
                write!(
                    f,
                    "frame {sequence} reservation was abandoned before publication"
                )
            }
            Self::ReliableSinkWouldBlock { sink, sequence } => write!(
                f,
                "reliable sink {sink} returned WouldBlock for frame {sequence}"
            ),
            Self::SinkFailed {
                sink,
                sequence,
                source,
            } => {
                if let Some(sequence) = sequence {
                    write!(f, "sink {sink} failed for frame {sequence}: {source}")
                } else {
                    write!(f, "sink {sink} failed while finishing: {source}")
                }
            }
            Self::SinkPanicked { sink, sequence } => {
                if let Some(sequence) = sequence {
                    write!(f, "sink {sink} panicked for frame {sequence}")
                } else {
                    write!(f, "sink {sink} panicked while finishing")
                }
            }
            Self::ThreadSpawn { message } => {
                write!(f, "ordered emitter drain thread could not start: {message}")
            }
            Self::WorkerPanicked => f.write_str("ordered emitter drain thread panicked"),
            Self::InternalInvariant { message, sequence } => {
                if let Some(sequence) = sequence {
                    write!(
                        f,
                        "ordered emitter invariant failed for frame {sequence}: {message}"
                    )
                } else {
                    write!(f, "ordered emitter invariant failed: {message}")
                }
            }
        }
    }
}

impl std::error::Error for EmitterError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::SinkFailed { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Live ring counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmitterStats {
    /// Configured slot count.
    pub capacity: usize,
    /// Bytes in one frame.
    pub frame_bytes: usize,
    /// Exact bytes preallocated for frame storage.
    pub ring_bytes: usize,
    /// Slots reserved, ready, or currently visible to sinks.
    pub outstanding: usize,
    /// Reusable slots.
    pub available: usize,
    /// Largest observed occupancy.
    pub max_outstanding: usize,
    /// Reservations granted.
    pub reserved: u64,
    /// Reservations atomically published.
    pub published: u64,
    /// Frames successfully processed by every sink policy.
    pub emitted: u64,
    /// Dispatches that had to wait for a free slot.
    pub backpressure_waits: u64,
    /// Whether finish has stopped new reservations.
    pub closed: bool,
    /// Whether a terminal error is recorded.
    pub failed: bool,
}

/// Per-sink final delivery counters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SinkReport {
    /// Configured sink name.
    pub name: Arc<str>,
    /// Delivery policy.
    pub mode: SinkMode,
    /// Frames consumed by the sink.
    pub accepted: u64,
    /// Busy frames explicitly dropped by a preview-class sink.
    pub dropped: u64,
}

/// Successful final emitter report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmitterReport {
    /// Ring-level counters.
    pub stats: EmitterStats,
    /// Delivery counters in configured sink order.
    pub sinks: Vec<SinkReport>,
}

/// Terminal failure plus all counters available after drain shutdown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmitterFailure {
    /// First/root failure.
    pub error: EmitterError,
    /// Final ring and per-sink counters.
    pub report: Box<EmitterReport>,
}

impl fmt::Display for EmitterFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error.fmt(f)
    }
}

impl std::error::Error for EmitterFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// Cloneable dispatch-side access to one emitter ring.
#[derive(Clone)]
pub struct EmitterHandle {
    shared: Arc<Shared>,
}

impl fmt::Debug for EmitterHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EmitterHandle")
            .field("stats", &self.stats())
            .finish()
    }
}

impl EmitterHandle {
    /// Reserve the next frame slot, blocking only when the bounded ring is full.
    ///
    /// Callers must reserve in contiguous sequence order. That rule is what
    /// guarantees a full ring always contains the frame the ordered drain is
    /// waiting for.
    ///
    /// # Errors
    ///
    /// Returns a typed terminal/configuration error. Sink pressure itself is
    /// not an error; it increments `backpressure_waits` and waits.
    pub fn reserve(&self, sequence: u64) -> Result<FrameReservation, EmitterError> {
        let mut state = self.shared.lock();
        let mut counted_wait = false;

        loop {
            if let Some(error) = state.failure.clone() {
                return Err(error);
            }
            if state.closed {
                return Err(EmitterError::Closed);
            }
            if sequence != state.next_reservation_sequence {
                return Err(EmitterError::UnexpectedSequence {
                    expected: state.next_reservation_sequence,
                    actual: sequence,
                });
            }
            if sequence == u64::MAX {
                return Err(EmitterError::SequenceExhausted { sequence });
            }
            if state.outstanding < state.config.capacity {
                break;
            }
            if !counted_wait {
                state.backpressure_waits = state.backpressure_waits.saturating_add(1);
                counted_wait = true;
            }
            state = self.shared.wait_for_slot(state);
        }

        let slot_index = state.reserve_slot;
        let slot = mem::replace(&mut state.slots[slot_index], Slot::Vacant);
        let buffer = match slot {
            Slot::Free(buffer) => buffer,
            other => {
                state.slots[slot_index] = other;
                let error = EmitterError::InternalInvariant {
                    message: "reservation cursor did not address a free slot",
                    sequence: Some(sequence),
                };
                state.fail(error.clone());
                self.shared.notify_all();
                return Err(error);
            }
        };

        state.slots[slot_index] = Slot::Writing { sequence };
        state.reserve_slot = advance_slot(slot_index, state.config.capacity);
        state.next_reservation_sequence += 1;
        state.outstanding += 1;
        state.max_outstanding = state.max_outstanding.max(state.outstanding);
        state.reserved = state.reserved.saturating_add(1);
        drop(state);

        Ok(FrameReservation {
            sequence,
            buffer,
            guard: ReservationGuard {
                shared: Arc::clone(&self.shared),
                slot_index,
                sequence,
                armed: true,
            },
        })
    }

    /// Request cooperative cancellation and wake every waiter.
    pub fn cancel(&self) -> CancelOutcome {
        self.shared.cancel()
    }

    /// Snapshot counters without changing emitter state.
    #[must_use]
    pub fn stats(&self) -> EmitterStats {
        self.shared.lock().snapshot()
    }
}

/// An exclusively writable preallocated frame slot.
///
/// The sink side cannot access this buffer until [`Self::publish`] consumes
/// the reservation. Dropping it unpublished fails the stream closed.
pub struct FrameReservation {
    sequence: u64,
    buffer: FrameBuffer,
    guard: ReservationGuard,
}

impl fmt::Debug for FrameReservation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FrameReservation")
            .field("sequence", &self.sequence)
            .field("layout", self.buffer.layout())
            .finish_non_exhaustive()
    }
}

impl FrameReservation {
    /// Reserved frame index.
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Immutable access to the private buffer.
    #[must_use]
    pub const fn frame(&self) -> &FrameBuffer {
        &self.buffer
    }

    /// Exclusive fill access. No sink can observe these writes.
    pub fn frame_mut(&mut self) -> &mut FrameBuffer {
        &mut self.buffer
    }

    /// Atomically publish the complete buffer to the ordered drain.
    ///
    /// # Errors
    ///
    /// Returns the stream's root failure if cancellation or a sink failure won
    /// the race. The slot is released before the error returns.
    pub fn publish(self) -> Result<(), EmitterError> {
        let Self { buffer, guard, .. } = self;
        guard.publish(buffer)
    }
}

/// Controller owning the dedicated sink thread.
pub struct OrderedEmitter {
    handle: EmitterHandle,
    worker: Option<JoinHandle<Vec<SinkReport>>>,
}

impl fmt::Debug for OrderedEmitter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OrderedEmitter")
            .field("stats", &self.handle.stats())
            .finish_non_exhaustive()
    }
}

impl OrderedEmitter {
    /// Preallocate the ring and start its drain thread.
    ///
    /// # Errors
    ///
    /// Refuses an empty/unnamed sink set and reports thread creation failure.
    pub fn new(config: EmitterConfig, mut sinks: Vec<SinkBinding>) -> Result<Self, EmitterError> {
        if sinks.is_empty() {
            return Err(EmitterError::NoSinks);
        }
        if let Some(index) = sinks.iter().position(|sink| sink.name().is_empty()) {
            return Err(EmitterError::EmptySinkName { index });
        }

        let mut pool = FramePool::new(config.layout.clone(), config.capacity);
        let mut slots = Vec::with_capacity(config.capacity);
        for _ in 0..config.capacity {
            let Some(buffer) = pool.try_acquire() else {
                return Err(EmitterError::InternalInvariant {
                    message: "new frame pool did not contain its declared capacity",
                    sequence: None,
                });
            };
            slots.push(Slot::Free(buffer));
        }

        let first_sequence = config.first_sequence;
        let cancellation = ProcessCancellation::new();
        for sink in &mut sinks {
            sink.sink.set_cancellation(cancellation.clone());
        }
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                config,
                slots: slots.into_boxed_slice(),
                reserve_slot: 0,
                emit_slot: 0,
                next_reservation_sequence: first_sequence,
                next_emission_sequence: first_sequence,
                outstanding: 0,
                max_outstanding: 0,
                reserved: 0,
                published: 0,
                emitted: 0,
                backpressure_waits: 0,
                closed: false,
                failure: None,
                cancellation,
            }),
            finalization: Mutex::new(FinalizationState::Open),
            ready: Condvar::new(),
            slot_available: Condvar::new(),
        });

        let worker_shared = Arc::clone(&shared);
        let worker = thread::Builder::new()
            .name("fmn-ordered-emitter".into())
            .spawn(move || drain(worker_shared, sinks))
            .map_err(|error| EmitterError::ThreadSpawn {
                message: Arc::from(error.to_string()),
            })?;

        Ok(Self {
            handle: EmitterHandle { shared },
            worker: Some(worker),
        })
    }

    /// Clone a dispatch-side reservation handle.
    #[must_use]
    pub fn handle(&self) -> EmitterHandle {
        self.handle.clone()
    }

    /// Reserve through the controller when dispatch is single-owner.
    pub fn reserve(&self, sequence: u64) -> Result<FrameReservation, EmitterError> {
        self.handle.reserve(sequence)
    }

    /// Snapshot live counters.
    #[must_use]
    pub fn stats(&self) -> EmitterStats {
        self.handle.stats()
    }

    /// Request cooperative cancellation.
    pub fn cancel(&self) -> CancelOutcome {
        self.handle.cancel()
    }

    /// Close dispatch, drain every published/reserved frame, finish sinks, and
    /// join the worker.
    ///
    /// Existing reservations may still publish after close begins. The caller
    /// should join render workers before calling this method; otherwise this
    /// method correctly waits for their reservations. Sinks finalize in
    /// binding order; a failure aborts the failing and remaining sinks.
    pub fn finish(mut self) -> Result<EmitterReport, EmitterFailure> {
        self.handle.shared.close();
        let sink_reports = match self.worker.take() {
            Some(worker) => match worker.join() {
                Ok(reports) => reports,
                Err(_) => {
                    self.handle.shared.fail(EmitterError::WorkerPanicked);
                    Vec::new()
                }
            },
            None => Vec::new(),
        };
        let state = self.handle.shared.lock();
        let report = EmitterReport {
            stats: state.snapshot(),
            sinks: sink_reports,
        };
        if let Some(error) = state.failure.clone() {
            Err(EmitterFailure {
                error,
                report: Box::new(report),
            })
        } else {
            Ok(report)
        }
    }
}

impl Drop for OrderedEmitter {
    fn drop(&mut self) {
        if let Some(worker) = self.worker.take() {
            self.handle.shared.cancel();
            drop(worker);
        }
    }
}

struct ReservationGuard {
    shared: Arc<Shared>,
    slot_index: usize,
    sequence: u64,
    armed: bool,
}

impl ReservationGuard {
    fn publish(mut self, buffer: FrameBuffer) -> Result<(), EmitterError> {
        let mut state = self.shared.lock();
        let slot = mem::replace(&mut state.slots[self.slot_index], Slot::Vacant);
        match slot {
            Slot::Writing { sequence } if sequence == self.sequence => {
                if let Some(error) = state.failure.clone() {
                    state.slots[self.slot_index] = Slot::Free(buffer);
                    state.outstanding = state.outstanding.saturating_sub(1);
                    self.armed = false;
                    drop(state);
                    self.shared.notify_all();
                    return Err(error);
                }
                state.slots[self.slot_index] = Slot::Ready {
                    sequence: self.sequence,
                    buffer,
                };
                state.published = state.published.saturating_add(1);
                self.armed = false;
                drop(state);
                self.shared.ready.notify_one();
                Ok(())
            }
            other => {
                state.slots[self.slot_index] = other;
                let error = EmitterError::InternalInvariant {
                    message: "publication did not own its writing slot",
                    sequence: Some(self.sequence),
                };
                state.fail(error.clone());
                self.armed = false;
                drop(state);
                self.shared.notify_all();
                Err(error)
            }
        }
    }
}

impl Drop for ReservationGuard {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let mut state = self.shared.lock();
        let slot = mem::replace(&mut state.slots[self.slot_index], Slot::Vacant);
        match slot {
            Slot::Writing { sequence } if sequence == self.sequence => {
                state.outstanding = state.outstanding.saturating_sub(1);
                state.fail(EmitterError::ReservationAbandoned {
                    sequence: self.sequence,
                });
            }
            other => {
                state.slots[self.slot_index] = other;
                state.fail(EmitterError::InternalInvariant {
                    message: "abandoned reservation did not own its writing slot",
                    sequence: Some(self.sequence),
                });
            }
        }
        drop(state);
        self.shared.notify_all();
    }
}

struct Shared {
    state: Mutex<State>,
    finalization: Mutex<FinalizationState>,
    ready: Condvar,
    slot_available: Condvar,
}

impl Shared {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn wait_for_ready<'a>(&self, state: MutexGuard<'a, State>) -> MutexGuard<'a, State> {
        self.ready
            .wait(state)
            .unwrap_or_else(PoisonError::into_inner)
    }

    fn wait_for_slot<'a>(&self, state: MutexGuard<'a, State>) -> MutexGuard<'a, State> {
        self.slot_available
            .wait(state)
            .unwrap_or_else(PoisonError::into_inner)
    }

    fn fail(&self, error: EmitterError) {
        let mut state = self.lock();
        state.fail(error);
        drop(state);
        self.notify_all();
    }

    fn cancel(&self) -> CancelOutcome {
        let mut finalization = self
            .finalization
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        match *finalization {
            FinalizationState::Committing | FinalizationState::Complete => CancelOutcome::TooLate,
            FinalizationState::Cancelled => CancelOutcome::AlreadyTerminal,
            FinalizationState::Open => {
                let mut state = self.lock();
                if state.failure.is_some() {
                    *finalization = FinalizationState::Cancelled;
                    return CancelOutcome::AlreadyTerminal;
                }
                *finalization = FinalizationState::Cancelled;
                state.fail(EmitterError::Cancelled);
                drop(state);
                drop(finalization);
                self.notify_all();
                CancelOutcome::Won
            }
        }
    }

    fn close(&self) {
        let mut state = self.lock();
        state.closed = true;
        drop(state);
        self.notify_all();
    }

    fn notify_all(&self) {
        self.ready.notify_all();
        self.slot_available.notify_all();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FinalizationState {
    Open,
    Cancelled,
    Committing,
    Complete,
}

struct State {
    config: EmitterConfig,
    slots: Box<[Slot]>,
    reserve_slot: usize,
    emit_slot: usize,
    next_reservation_sequence: u64,
    next_emission_sequence: u64,
    outstanding: usize,
    max_outstanding: usize,
    reserved: u64,
    published: u64,
    emitted: u64,
    backpressure_waits: u64,
    closed: bool,
    failure: Option<EmitterError>,
    cancellation: ProcessCancellation,
}

impl State {
    fn fail(&mut self, error: EmitterError) {
        if self.failure.is_none() {
            self.failure = Some(error);
            self.cancellation.cancel();
        }
    }

    fn snapshot(&self) -> EmitterStats {
        let available = self
            .slots
            .iter()
            .filter(|slot| matches!(slot, Slot::Free(_)))
            .count();
        EmitterStats {
            capacity: self.config.capacity,
            frame_bytes: self.config.frame_bytes(),
            ring_bytes: self.config.ring_bytes,
            outstanding: self.outstanding,
            available,
            max_outstanding: self.max_outstanding,
            reserved: self.reserved,
            published: self.published,
            emitted: self.emitted,
            backpressure_waits: self.backpressure_waits,
            closed: self.closed,
            failed: self.failure.is_some(),
        }
    }
}

enum Slot {
    Free(FrameBuffer),
    Writing { sequence: u64 },
    Ready { sequence: u64, buffer: FrameBuffer },
    Emitting { sequence: u64 },
    Vacant,
}

struct SinkWorker {
    binding: SinkBinding,
    accepted: u64,
    dropped: u64,
}

impl SinkWorker {
    fn new(binding: SinkBinding) -> Self {
        Self {
            binding,
            accepted: 0,
            dropped: 0,
        }
    }

    fn report(self) -> SinkReport {
        SinkReport {
            name: self.binding.name,
            mode: self.binding.mode,
            accepted: self.accepted,
            dropped: self.dropped,
        }
    }
}

enum DrainStep {
    Frame {
        sequence: u64,
        slot_index: usize,
        buffer: FrameBuffer,
    },
    Complete,
    Failed,
}

fn drain(shared: Arc<Shared>, sinks: Vec<SinkBinding>) -> Vec<SinkReport> {
    let mut sinks = sinks.into_iter().map(SinkWorker::new).collect::<Vec<_>>();

    loop {
        let step = next_drain_step(&shared);
        let DrainStep::Frame {
            sequence,
            slot_index,
            buffer,
        } = step
        else {
            break;
        };

        let delivery = deliver_frame(sequence, &buffer, &mut sinks);
        let mut state = shared.lock();
        let slot = mem::replace(&mut state.slots[slot_index], Slot::Vacant);
        let owned = matches!(slot, Slot::Emitting { sequence: active } if active == sequence);
        if owned {
            state.slots[slot_index] = Slot::Free(buffer);
            state.outstanding = state.outstanding.saturating_sub(1);
        } else {
            state.slots[slot_index] = slot;
            state.fail(EmitterError::InternalInvariant {
                message: "drain completion did not own its emitting slot",
                sequence: Some(sequence),
            });
        }

        if let Err(error) = delivery {
            state.fail(error);
        } else if owned {
            state.emit_slot = advance_slot(slot_index, state.config.capacity);
            state.next_emission_sequence += 1;
            state.emitted = state.emitted.saturating_add(1);
        }
        let failed = state.failure.is_some();
        drop(state);
        shared.notify_all();
        if failed {
            break;
        }
    }

    finalize_sinks(&shared, &mut sinks);
    sinks.into_iter().map(SinkWorker::report).collect()
}

fn next_drain_step(shared: &Shared) -> DrainStep {
    let mut state = shared.lock();
    loop {
        if state.failure.is_some() {
            return DrainStep::Failed;
        }

        let slot_index = state.emit_slot;
        let slot = mem::replace(&mut state.slots[slot_index], Slot::Vacant);
        match slot {
            Slot::Ready { sequence, buffer } => {
                if sequence != state.next_emission_sequence {
                    state.slots[slot_index] = Slot::Ready { sequence, buffer };
                    state.fail(EmitterError::InternalInvariant {
                        message: "ready slot sequence did not match ordered drain cursor",
                        sequence: Some(sequence),
                    });
                    drop(state);
                    shared.notify_all();
                    return DrainStep::Failed;
                }
                state.slots[slot_index] = Slot::Emitting { sequence };
                return DrainStep::Frame {
                    sequence,
                    slot_index,
                    buffer,
                };
            }
            other => {
                state.slots[slot_index] = other;
                if state.closed && state.outstanding == 0 {
                    return DrainStep::Complete;
                }
                state = shared.wait_for_ready(state);
            }
        }
    }
}

fn deliver_frame(
    sequence: u64,
    frame: &FrameBuffer,
    sinks: &mut [SinkWorker],
) -> Result<(), EmitterError> {
    for worker in sinks {
        let result = catch_unwind(AssertUnwindSafe(|| {
            worker.binding.sink.write_frame(sequence, frame)
        }));
        match result {
            Ok(Ok(SinkWrite::Consumed)) => {
                worker.accepted = worker.accepted.saturating_add(1);
            }
            Ok(Ok(SinkWrite::WouldBlock)) if worker.binding.mode == SinkMode::PreviewDrop => {
                worker.dropped = worker.dropped.saturating_add(1);
            }
            Ok(Ok(SinkWrite::WouldBlock)) => {
                return Err(EmitterError::ReliableSinkWouldBlock {
                    sink: Arc::clone(&worker.binding.name),
                    sequence,
                });
            }
            Ok(Err(source)) => {
                return Err(EmitterError::SinkFailed {
                    sink: Arc::clone(&worker.binding.name),
                    sequence: Some(sequence),
                    source,
                });
            }
            Err(_) => {
                return Err(EmitterError::SinkPanicked {
                    sink: Arc::clone(&worker.binding.name),
                    sequence: Some(sequence),
                });
            }
        }
    }
    Ok(())
}

fn finalize_sinks(shared: &Shared, sinks: &mut [SinkWorker]) {
    if shared.lock().failure.is_some() {
        abort_sinks(sinks);
        return;
    }

    for index in 0..sinks.len() {
        let error = {
            let worker = &mut sinks[index];
            let result = catch_unwind(AssertUnwindSafe(|| worker.binding.sink.prepare_finish()));
            finalization_error(worker, result)
        };
        if let Some(error) = error {
            shared.fail(error);
            abort_sinks(sinks);
            return;
        }
    }

    let mut finalization = shared
        .finalization
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    if *finalization != FinalizationState::Open || shared.lock().failure.is_some() {
        *finalization = FinalizationState::Cancelled;
        drop(finalization);
        abort_sinks(sinks);
        return;
    }
    *finalization = FinalizationState::Committing;

    for index in 0..sinks.len() {
        let error = {
            let worker = &mut sinks[index];
            let result = catch_unwind(AssertUnwindSafe(|| worker.binding.sink.commit_finish()));
            finalization_error(worker, result)
        };
        if let Some(error) = error {
            *finalization = FinalizationState::Complete;
            shared.fail(error);
            drop(finalization);
            abort_sinks(&mut sinks[index..]);
            return;
        }
    }
    *finalization = FinalizationState::Complete;
}

fn finalization_error(
    worker: &SinkWorker,
    result: Result<Result<(), SinkFailure>, Box<dyn std::any::Any + Send>>,
) -> Option<EmitterError> {
    match result {
        Ok(Ok(())) => None,
        Ok(Err(source)) => Some(EmitterError::SinkFailed {
            sink: Arc::clone(&worker.binding.name),
            sequence: None,
            source,
        }),
        Err(_) => Some(EmitterError::SinkPanicked {
            sink: Arc::clone(&worker.binding.name),
            sequence: None,
        }),
    }
}

fn abort_sinks(sinks: &mut [SinkWorker]) {
    for worker in sinks {
        let _ = catch_unwind(AssertUnwindSafe(|| worker.binding.sink.abort()));
    }
}

const fn advance_slot(slot: usize, capacity: usize) -> usize {
    if slot + 1 == capacity { 0 } else { slot + 1 }
}
