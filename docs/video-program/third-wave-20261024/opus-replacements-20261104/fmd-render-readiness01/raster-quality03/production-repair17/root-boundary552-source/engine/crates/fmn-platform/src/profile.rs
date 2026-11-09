//! Owned, capability-clocked profiling records (§17.1, §18).
//!
//! Profiling is observability, not scene semantics: these records never enter
//! the certified input closure or a frame digest. The clock is still an
//! explicit [`Clock`] capability rather than an ambient `Instant`, so tests
//! and replay tools can produce exact timing fixtures.
//!
//! [`ProfileRecorder::disabled`] is the production default. Its hot-path cost
//! is one `Option` branch: it reads no clock, allocates nothing, and takes no
//! lock. An enabled recorder stores a stable scene → play → frame → phase →
//! tile path, exports versioned line-oriented JSON, and emits folded stacks
//! suitable for flame-summary tooling. Filesystem publication belongs to the
//! CLI/composition root; this module only turns records into owned text.

use crate::clock::Clock;
use std::collections::BTreeMap;
use std::fmt::{self, Write as _};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

/// Stable schema carried by every NDJSON record.
pub const PROFILE_SCHEMA: &str = "fmn-profile/1";

/// The fixed hierarchy owned by §18.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProfilePath {
    /// Scene identity within this profiled run.
    pub scene: u64,
    /// Play identity within the scene, when one exists.
    pub play: Option<u64>,
    /// Durable frame sequence, when the record belongs to one frame.
    pub frame: Option<u64>,
    /// Fine-tile index, only for tile-local work.
    pub tile: Option<u64>,
}

impl ProfilePath {
    /// Start a scene-level path.
    #[must_use]
    pub const fn scene(scene: u64) -> Self {
        Self {
            scene,
            play: None,
            frame: None,
            tile: None,
        }
    }

    /// Refine this path to one play.
    #[must_use]
    pub const fn with_play(mut self, play: u64) -> Self {
        self.play = Some(play);
        self
    }

    /// Refine this path to one frame.
    #[must_use]
    pub const fn with_frame(mut self, frame: u64) -> Self {
        self.frame = Some(frame);
        self.tile = None;
        self
    }

    /// Refine this path to one tile.
    #[must_use]
    pub const fn with_tile(mut self, tile: u64) -> Self {
        self.tile = Some(tile);
        self
    }
}

/// Version-1 phase taxonomy.
///
/// The first eleven variants are the mandatory §17.1 breakdown. Composite
/// pipeline phases stay named honestly until their adapters split them into
/// the more precise records.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProfilePhase {
    /// Scene state advance and updater work.
    SceneUpdate,
    /// A callback crossing into Python.
    PythonCallback,
    /// Object-space geometry compilation.
    GeometryCompile,
    /// Retained render-IR synchronization.
    RenderIrSync,
    /// Macro/fine-tile command-list construction.
    Binning,
    /// Lumen coverage and compositing.
    Raster,
    /// Raw-frame color/pixel-format conversion.
    ColorConversion,
    /// Accelerator resource upload.
    AnnexUpload,
    /// Accelerator result readback.
    AnnexReadback,
    /// Ordered bytes written to the ffmpeg pipe.
    FfmpegFeed,
    /// Encoder work after input submission.
    Encode,
    /// A not-yet-split render preparation callback.
    Prepare,
    /// Ordered emitter handoff.
    Emit,
    /// A drained effect-model barrier.
    Barrier,
}

impl ProfilePhase {
    /// Stable robot-facing spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::SceneUpdate => "scene_update",
            Self::PythonCallback => "python_callback",
            Self::GeometryCompile => "geometry_compile",
            Self::RenderIrSync => "render_ir_sync",
            Self::Binning => "binning",
            Self::Raster => "raster",
            Self::ColorConversion => "color_conversion",
            Self::AnnexUpload => "annex_upload",
            Self::AnnexReadback => "annex_readback",
            Self::FfmpegFeed => "ffmpeg_feed",
            Self::Encode => "encode",
            Self::Prepare => "prepare",
            Self::Emit => "emit",
            Self::Barrier => "barrier",
        }
    }
}

impl fmt::Display for ProfilePhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Stable worker-lane role; host thread IDs are deliberately not serialized.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProfileLaneRole {
    /// The composition-root caller.
    Caller,
    /// Scene/update worker.
    Scene,
    /// Retained-plan preparation worker.
    Prepare,
    /// One CPU render team.
    Render,
    /// Color conversion / output worker.
    Output,
    /// Accelerator queue.
    Annex,
    /// Encoder worker.
    Encoder,
}

impl ProfileLaneRole {
    /// Stable robot-facing spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Caller => "caller",
            Self::Scene => "scene",
            Self::Prepare => "prepare",
            Self::Render => "render",
            Self::Output => "output",
            Self::Annex => "annex",
            Self::Encoder => "encoder",
        }
    }
}

/// A stable logical lane inside a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProfileLane {
    /// Lane role.
    pub role: ProfileLaneRole,
    /// Role-local lane index.
    pub index: u16,
}

impl ProfileLane {
    /// A lane with a stable role and role-local index.
    #[must_use]
    pub const fn new(role: ProfileLaneRole, index: u16) -> Self {
        Self { role, index }
    }

    /// The serial composition-root caller.
    #[must_use]
    pub const fn caller() -> Self {
        Self::new(ProfileLaneRole::Caller, 0)
    }

    /// The single retained-plan preparation worker.
    #[must_use]
    pub const fn prepare() -> Self {
        Self::new(ProfileLaneRole::Prepare, 0)
    }

    /// One render-team leader.
    #[must_use]
    pub const fn render(index: u16) -> Self {
        Self::new(ProfileLaneRole::Render, index)
    }

    /// The output/conversion worker.
    #[must_use]
    pub const fn output() -> Self {
        Self::new(ProfileLaneRole::Output, 0)
    }
}

/// Version-1 counter taxonomy carried beside phase timings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProfileCounter {
    /// Frames accepted by the pipeline.
    SubmittedFrames,
    /// Frames handed to the ordered emitter seam.
    EmittedFrames,
    /// Source waits for a bounded in-flight slot.
    BackpressureWaits,
    /// Maximum observed in-flight frames.
    MaxInFlight,
    /// Retained-artifact reuse hits.
    ReuseHits,
    /// Retained-artifact reuse misses.
    ReuseMisses,
    /// Content-addressed cache hits.
    CacheHits,
    /// Content-addressed cache misses.
    CacheMisses,
    /// Dirty tiles in the observed frame/run.
    DirtyTiles,
    /// Total classified tiles in the observed frame/run.
    TotalTiles,
    /// Heap allocations in the observed frame/run.
    Allocations,
    /// Summed render-team busy nanoseconds.
    RenderTeamBusyNs,
    /// Available wall-clock nanoseconds for one render-team leader.
    RenderTeamCapacityNs,
    /// Bytes uploaded to an annex device.
    BytesUploaded,
    /// Bytes read back from an annex device.
    BytesReadBack,
    /// Maximum queued frames at the encoder boundary.
    EncodeQueueDepth,
}

impl ProfileCounter {
    /// Stable robot-facing spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::SubmittedFrames => "submitted_frames",
            Self::EmittedFrames => "emitted_frames",
            Self::BackpressureWaits => "backpressure_waits",
            Self::MaxInFlight => "max_in_flight",
            Self::ReuseHits => "reuse_hits",
            Self::ReuseMisses => "reuse_misses",
            Self::CacheHits => "cache_hits",
            Self::CacheMisses => "cache_misses",
            Self::DirtyTiles => "dirty_tiles",
            Self::TotalTiles => "total_tiles",
            Self::Allocations => "allocations",
            Self::RenderTeamBusyNs => "render_team_busy_ns",
            Self::RenderTeamCapacityNs => "render_team_capacity_ns",
            Self::BytesUploaded => "bytes_uploaded",
            Self::BytesReadBack => "bytes_read_back",
            Self::EncodeQueueDepth => "encode_queue_depth",
        }
    }
}

/// One completed timing span.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProfileSpanRecord {
    /// Monotonic insertion order within the recorder.
    pub ordinal: u64,
    /// Hierarchical owner.
    pub path: ProfilePath,
    /// Work classification.
    pub phase: ProfilePhase,
    /// Stable logical lane.
    pub lane: ProfileLane,
    /// Nanoseconds from the handed-in clock's epoch.
    pub start_ns: u64,
    /// Inclusive wall duration.
    pub duration_ns: u64,
}

/// One point counter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProfileCounterRecord {
    /// Monotonic insertion order within the recorder.
    pub ordinal: u64,
    /// Hierarchical owner.
    pub path: ProfilePath,
    /// Stable logical lane.
    pub lane: ProfileLane,
    /// Counter classification.
    pub counter: ProfileCounter,
    /// Unsigned measured value.
    pub value: u64,
}

/// One version-1 profile record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileRecord {
    /// Timed work.
    Span(ProfileSpanRecord),
    /// Point counter.
    Counter(ProfileCounterRecord),
}

impl ProfileRecord {
    const fn ordinal(self) -> u64 {
        match self {
            Self::Span(record) => record.ordinal,
            Self::Counter(record) => record.ordinal,
        }
    }
}

/// An immutable export snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileSnapshot {
    records: Vec<ProfileRecord>,
}

impl ProfileSnapshot {
    /// Records in recorder ordinal order.
    #[must_use]
    pub fn records(&self) -> &[ProfileRecord] {
        &self.records
    }

    /// Versioned, line-oriented JSON with no human decoration.
    #[must_use]
    pub fn to_ndjson(&self) -> String {
        let mut out = String::new();
        for record in &self.records {
            match *record {
                ProfileRecord::Span(span) => {
                    let _ = write!(
                        out,
                        "{{\"schema\":\"{PROFILE_SCHEMA}\",\"kind\":\"span\",\
                         \"ordinal\":{},\"scene\":{},\"play\":",
                        span.ordinal, span.path.scene
                    );
                    push_optional_u64(&mut out, span.path.play);
                    out.push_str(",\"frame\":");
                    push_optional_u64(&mut out, span.path.frame);
                    out.push_str(",\"tile\":");
                    push_optional_u64(&mut out, span.path.tile);
                    let _ = writeln!(
                        out,
                        ",\"phase\":\"{}\",\"lane_role\":\"{}\",\"lane_index\":{},\
                         \"start_ns\":{},\"duration_ns\":{}}}",
                        span.phase.name(),
                        span.lane.role.name(),
                        span.lane.index,
                        span.start_ns,
                        span.duration_ns
                    );
                }
                ProfileRecord::Counter(counter) => {
                    let _ = write!(
                        out,
                        "{{\"schema\":\"{PROFILE_SCHEMA}\",\"kind\":\"counter\",\
                         \"ordinal\":{},\"scene\":{},\"play\":",
                        counter.ordinal, counter.path.scene
                    );
                    push_optional_u64(&mut out, counter.path.play);
                    out.push_str(",\"frame\":");
                    push_optional_u64(&mut out, counter.path.frame);
                    out.push_str(",\"tile\":");
                    push_optional_u64(&mut out, counter.path.tile);
                    let _ = writeln!(
                        out,
                        ",\"counter\":\"{}\",\"lane_role\":\"{}\",\"lane_index\":{},\
                         \"value\":{}}}",
                        counter.counter.name(),
                        counter.lane.role.name(),
                        counter.lane.index,
                        counter.value
                    );
                }
            }
        }
        out
    }

    /// Aggregate spans into deterministic folded stacks for flame summaries.
    #[must_use]
    pub fn to_folded(&self) -> String {
        let mut totals = BTreeMap::<(ProfilePath, ProfilePhase, ProfileLane), u128>::new();
        for record in &self.records {
            if let ProfileRecord::Span(span) = *record {
                let total = totals
                    .entry((span.path, span.phase, span.lane))
                    .or_default();
                *total = total.saturating_add(u128::from(span.duration_ns));
            }
        }

        let mut out = String::new();
        for ((path, phase, lane), duration) in totals {
            let _ = write!(out, "scene:{}", path.scene);
            if let Some(play) = path.play {
                let _ = write!(out, ";play:{play}");
            }
            if let Some(frame) = path.frame {
                let _ = write!(out, ";frame:{frame}");
            }
            if let Some(tile) = path.tile {
                let _ = write!(out, ";tile:{tile}");
            }
            let _ = writeln!(
                out,
                ";phase:{};lane:{}:{} {duration}",
                phase.name(),
                lane.role.name(),
                lane.index
            );
        }
        out
    }
}

fn push_optional_u64(out: &mut String, value: Option<u64>) {
    if let Some(value) = value {
        let _ = write!(out, "{value}");
    } else {
        out.push_str("null");
    }
}

#[derive(Debug)]
struct ProfileInner {
    next_ordinal: AtomicU64,
    records: Mutex<Vec<ProfileRecord>>,
}

impl ProfileInner {
    fn ordinal(&self) -> u64 {
        self.next_ordinal.fetch_add(1, Ordering::Relaxed)
    }

    fn push(&self, record: ProfileRecord) {
        self.records
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(record);
    }
}

/// Cloneable profile capability.
///
/// Disabled recorders perform no clock reads, allocations, atomics, or locks.
#[derive(Clone, Default)]
pub struct ProfileRecorder {
    inner: Option<Arc<ProfileInner>>,
}

impl fmt::Debug for ProfileRecorder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProfileRecorder")
            .field("enabled", &self.is_enabled())
            .finish_non_exhaustive()
    }
}

impl ProfileRecorder {
    /// A recorder that discards everything at one branch per call.
    #[must_use]
    pub const fn disabled() -> Self {
        Self { inner: None }
    }

    /// A live in-memory recorder.
    #[must_use]
    pub fn enabled() -> Self {
        Self {
            inner: Some(Arc::new(ProfileInner {
                next_ordinal: AtomicU64::new(0),
                records: Mutex::new(Vec::new()),
            })),
        }
    }

    /// Whether timing/counter collection is active.
    #[must_use]
    #[inline]
    pub const fn is_enabled(&self) -> bool {
        self.inner.is_some()
    }

    /// Begin a capability-clocked span.
    ///
    /// Hold the returned guard for exactly the work being measured. Dropping
    /// it, including during unwinding, records the span.
    #[must_use]
    #[inline]
    pub fn span<'a>(
        &self,
        clock: &'a dyn Clock,
        path: ProfilePath,
        phase: ProfilePhase,
        lane: ProfileLane,
    ) -> Option<ProfileSpan<'a>> {
        let inner = Arc::clone(self.inner.as_ref()?);
        let start = clock.monotonic();
        Some(ProfileSpan {
            inner,
            clock,
            path,
            phase,
            lane,
            start,
            finished: false,
        })
    }

    /// Record a span already measured with the same handed-in clock.
    #[inline]
    pub fn record_span(
        &self,
        path: ProfilePath,
        phase: ProfilePhase,
        lane: ProfileLane,
        start: Duration,
        duration: Duration,
    ) {
        let Some(inner) = &self.inner else {
            return;
        };
        let ordinal = inner.ordinal();
        inner.push(ProfileRecord::Span(ProfileSpanRecord {
            ordinal,
            path,
            phase,
            lane,
            start_ns: duration_nanos(start),
            duration_ns: duration_nanos(duration),
        }));
    }

    /// Record one unsigned counter value.
    #[inline]
    pub fn record_counter(
        &self,
        path: ProfilePath,
        counter: ProfileCounter,
        lane: ProfileLane,
        value: u64,
    ) {
        let Some(inner) = &self.inner else {
            return;
        };
        let ordinal = inner.ordinal();
        inner.push(ProfileRecord::Counter(ProfileCounterRecord {
            ordinal,
            path,
            lane,
            counter,
            value,
        }));
    }

    /// Take an immutable, ordinal-sorted snapshot.
    #[must_use]
    pub fn snapshot(&self) -> ProfileSnapshot {
        let mut records = self
            .inner
            .as_ref()
            .map(|inner| {
                inner
                    .records
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .clone()
            })
            .unwrap_or_default();
        records.sort_by_key(|record| record.ordinal());
        ProfileSnapshot { records }
    }
}

/// RAII span returned by [`ProfileRecorder::span`].
#[must_use = "hold the span guard until the measured work is complete"]
pub struct ProfileSpan<'a> {
    inner: Arc<ProfileInner>,
    clock: &'a dyn Clock,
    path: ProfilePath,
    phase: ProfilePhase,
    lane: ProfileLane,
    start: Duration,
    finished: bool,
}

impl fmt::Debug for ProfileSpan<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProfileSpan")
            .field("path", &self.path)
            .field("phase", &self.phase)
            .field("lane", &self.lane)
            .field("start", &self.start)
            .finish_non_exhaustive()
    }
}

impl ProfileSpan<'_> {
    /// Finish immediately and return the recorded duration.
    pub fn finish(mut self) -> Duration {
        let duration = self.record();
        self.finished = true;
        duration
    }

    fn record(&self) -> Duration {
        let duration = self.clock.monotonic().saturating_sub(self.start);
        let ordinal = self.inner.ordinal();
        self.inner.push(ProfileRecord::Span(ProfileSpanRecord {
            ordinal,
            path: self.path,
            phase: self.phase,
            lane: self.lane,
            start_ns: duration_nanos(self.start),
            duration_ns: duration_nanos(duration),
        }));
        duration
    }
}

impl Drop for ProfileSpan<'_> {
    fn drop(&mut self) {
        if !self.finished {
            self.record();
        }
    }
}

fn duration_nanos(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::FakeClock;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn disabled_recorders_do_not_touch_the_clock() {
        #[derive(Default)]
        struct ProbeClock {
            reads: AtomicUsize,
        }

        impl Clock for ProbeClock {
            fn monotonic(&self) -> Duration {
                self.reads.fetch_add(1, Ordering::Relaxed);
                Duration::ZERO
            }

            fn wall(&self) -> std::time::SystemTime {
                self.reads.fetch_add(1, Ordering::Relaxed);
                std::time::SystemTime::UNIX_EPOCH
            }
        }

        let recorder = ProfileRecorder::disabled();
        let clock = ProbeClock::default();
        assert!(
            recorder
                .span(
                    &clock,
                    ProfilePath::scene(0),
                    ProfilePhase::SceneUpdate,
                    ProfileLane::caller()
                )
                .is_none()
        );
        recorder.record_counter(
            ProfilePath::scene(0),
            ProfileCounter::Allocations,
            ProfileLane::caller(),
            0,
        );
        assert!(recorder.snapshot().records().is_empty());
        assert_eq!(clock.reads.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn ndjson_and_folded_exports_are_stable_and_hierarchical() {
        let clock = FakeClock::new();
        let recorder = ProfileRecorder::enabled();
        let path = ProfilePath::scene(7)
            .with_play(2)
            .with_frame(11)
            .with_tile(3);
        let span = recorder
            .span(&clock, path, ProfilePhase::Raster, ProfileLane::render(1))
            .expect("enabled");
        clock.advance(Duration::from_micros(250));
        assert_eq!(span.finish(), Duration::from_micros(250));
        recorder.record_counter(path, ProfileCounter::DirtyTiles, ProfileLane::render(1), 4);

        let snapshot = recorder.snapshot();
        assert_eq!(snapshot.records().len(), 2);
        assert_eq!(
            snapshot.to_ndjson(),
            concat!(
                "{\"schema\":\"fmn-profile/1\",\"kind\":\"span\",\"ordinal\":0,",
                "\"scene\":7,\"play\":2,\"frame\":11,\"tile\":3,\"phase\":\"raster\",",
                "\"lane_role\":\"render\",\"lane_index\":1,\"start_ns\":0,",
                "\"duration_ns\":250000}\n",
                "{\"schema\":\"fmn-profile/1\",\"kind\":\"counter\",\"ordinal\":1,",
                "\"scene\":7,\"play\":2,\"frame\":11,\"tile\":3,",
                "\"counter\":\"dirty_tiles\",\"lane_role\":\"render\",",
                "\"lane_index\":1,\"value\":4}\n"
            )
        );
        assert_eq!(
            snapshot.to_folded(),
            "scene:7;play:2;frame:11;tile:3;phase:raster;lane:render:1 250000\n"
        );
    }
}
