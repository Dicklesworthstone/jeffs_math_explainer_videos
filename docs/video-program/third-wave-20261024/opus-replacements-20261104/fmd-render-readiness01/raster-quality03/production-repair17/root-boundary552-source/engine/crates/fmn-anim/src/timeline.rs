//! The declarative `Timeline` (§9.4, §13.5, §10.7, fm-hfe): keyframes,
//! labels, and seek — **sugar over composed animations on the rational
//! clock, never a second engine**.
//!
//! A [`Timeline`] is an authored list of steps. Each step compiles to the
//! very primitive an imperative scene would have called —
//! [`play_segment`] or [`wait_segment`] — over the same
//! [`RationalFrameClock`], through the same six-step frame order, producing
//! the same [`FramePacket`]s and the same [`SegmentReport`]s. There is no
//! timeline-specific interpolation path anywhere in this module; seek uses
//! the drivers' partial forms ([`open_play`] / [`advance_play`] and
//! [`wait_segment_upto`]) precisely so a sought frame and a played frame are
//! the same frame by construction, not by agreement.
//!
//! **Three things a timeline adds over a straight-line scene.**
//!
//! 1. *A compiled schedule.* [`Timeline::compile`] derives every segment's
//!    frame range before anything runs, because a segment's length is a
//!    pure function of `(fps, run_time)` and `run_time` is known from the
//!    animations' constructor surface (`get_run_time`, `time_span` widening
//!    included). That schedule — [`TimelinePlan`] — is what a scrubber
//!    needs and what serializes.
//! 2. *Labels.* Named positions on the schedule (`"intro"`, `"the punch
//!    line"`), resolved to frames at compile time.
//! 3. *Seek.* [`Timeline::seek`] reconstructs the state at any frame
//!    deterministically: **pure segments** (§9.5) rebuild in O(1) from the
//!    begin-state snapshot plus alpha via
//!    [`reconstruct_pure_frame`];
//!    **stateful segments** replay forward from the nearest checkpoint,
//!    because their frames depend on accumulated per-frame state and there
//!    is no honest shortcut. Checkpoints are CoW arena snapshots taken at
//!    segment boundaries, so the cost of holding them is O(touched).
//!
//! **One consequence of authoring declaratively.** A timeline holds
//! *prepared* animations, so an `.animate` chain freezes its target when the
//! step is authored, not when the step runs. An imperative scene calling
//! `play(mob.animate().shift(…))` prepares at play time and therefore
//! captures whatever the previous segment left behind; the same two steps
//! authored up front capture the state at authoring time. This is inherent
//! to declaring a whole schedule before running it — it is also what lets
//! [`Timeline::compile`] derive the frame schedule without a stage — and it
//! is named here rather than papered over. Author a step against the state
//! it should start from, or use [`Succession`](crate::Succession), whose
//! members begin just in time precisely to chain off each other.
//!
//! **What serialization promises today.** [`TimelinePlan::to_bytes`] writes
//! the schedule — fps, every segment's kind, run time, and frame range, and
//! every label — in fmn-hash's canonical container (`FMNA/5`), so it
//! round-trips byte-identically and content-addresses. That is the schedule
//! substrate the Studio scrubber (§13.5) and the WASM tier-2 player (§10.7,
//! fm-oee) both sit on. It is deliberately **a subset of the final
//! abstraction, never a substitute**: replaying frames with no scene code in
//! the process additionally needs the per-frame content the player
//! serializes (fm-oee) or animations reconstructed from the one API schema
//! (fm-vn6). Neither exists yet, and neither is faked here — a plan carries
//! what a plan can prove.

use std::collections::BTreeMap;
use std::rc::Rc;

use fmn_core::rng::RngRoot;
use fmn_hash::serial::{Limits, Reader, Schema, UnknownPolicy, Writer};
use fmn_hash::{Digest, SerialError, sha256};
use fmn_mobject::{Snapshot, Stage};

use crate::animation::{AnimError, Animation};
use crate::clock::{ClockError, FrameSample, RationalFrameClock};
use crate::frame::{
    FramePacket, advance_play, open_play, play_segment, wait_segment, wait_segment_upto,
};
use crate::purity::{SegmentKind, SegmentReport, reconstruct_pure_frame};

/// The canonical container schema for a serialized [`TimelinePlan`].
pub const TIMELINE_SCHEMA: Schema = Schema::new(*b"FMNA", 5, 1, 0);

/// A timeline serialization failure.
#[derive(Debug)]
pub enum TimelineError {
    /// The canonical container refused the bytes (framing, version,
    /// checksum, limits).
    Serial(SerialError),
    /// The payload decoded but violates a schedule invariant.
    Malformed(&'static str),
}

impl std::fmt::Display for TimelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serial(e) => write!(f, "timeline container: {e}"),
            Self::Malformed(what) => write!(f, "malformed timeline: {what}"),
        }
    }
}

impl std::error::Error for TimelineError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Serial(e) => Some(e),
            Self::Malformed(_) => None,
        }
    }
}

impl From<SerialError> for TimelineError {
    fn from(e: SerialError) -> Self {
        Self::Serial(e)
    }
}

// -------------------------------------------------------------------- steps

/// One authored step of a timeline.
pub enum Step {
    /// A `play()` of one or more animations (composition operators are
    /// animations, so a step is a whole composed tree when you want one).
    Play(Vec<Box<dyn Animation>>),
    /// A `wait()` of `duration` seconds. There is no stop-condition form: a
    /// declarative timeline has no callbacks, and a segment whose length is
    /// decided at run time cannot be scheduled ahead of time.
    Wait(f64),
}

impl Step {
    /// The step's kind, as the journal vocabulary names it.
    #[must_use]
    pub fn kind(&self) -> SegmentKind {
        match self {
            Self::Play(_) => SegmentKind::Play,
            Self::Wait(_) => SegmentKind::Wait,
        }
    }

    /// The step's run time in seconds — for a play, the Reference's `np.max`
    /// over the members' `get_run_time` (the same number
    /// [`play_segment`] derives).
    #[must_use]
    pub fn run_time(&self) -> f64 {
        match self {
            Self::Play(animations) => animations
                .iter()
                .map(|a| a.get_run_time())
                .fold(f64::NEG_INFINITY, f64::max),
            Self::Wait(duration) => *duration,
        }
    }
}

// --------------------------------------------------------------- the plan

/// One segment of a compiled schedule.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlannedSegment {
    /// Play or wait.
    pub kind: SegmentKind,
    /// The segment's run time in seconds, as authored.
    pub run_time: f64,
    /// Frames elapsed before this segment starts (global frame *k* of the
    /// segment is `base_frame + k`, `k` being 1-based).
    pub base_frame: i64,
    /// Frames the segment covers on the grid.
    pub n_frames: i64,
}

impl PlannedSegment {
    /// The global frame index of this segment's last frame, or `None` when
    /// independently constructed public fields do not fit in the frame
    /// counter. Compiled and decoded plans validate this range up front.
    #[must_use]
    pub fn end_frame(&self) -> Option<i64> {
        self.base_frame.checked_add(self.n_frames)
    }
}

/// A named position on the schedule.
#[derive(Clone, Debug, PartialEq)]
pub struct Label {
    /// The label's name, as authored.
    pub name: String,
    /// The segment index the label marks.
    pub segment: usize,
    /// The global frame the label resolves to: the first emitted frame at
    /// or after its segment, or the timeline's existing end when no later
    /// segment emits a frame.
    pub frame: i64,
}

fn label_frame_at_or_after(
    segments: &[PlannedSegment],
    segment_index: usize,
    total_frames: i64,
) -> Option<i64> {
    let tail = segments.get(segment_index..)?;
    match tail.iter().find(|segment| segment.n_frames > 0) {
        Some(segment) => segment.base_frame.checked_add(1),
        None => Some(total_frames),
    }
}

fn decoded_clock_error(error: ClockError) -> TimelineError {
    match error {
        ClockError::ZeroFps => TimelineError::Malformed("fps"),
        ClockError::NonFiniteRunTime | ClockError::RunTimeTooLong => {
            TimelineError::Malformed("segment run time")
        }
        ClockError::NegativeFrameAdvance | ClockError::FrameCounterOverflow => {
            TimelineError::Malformed("segment frame range")
        }
    }
}

/// A compiled timeline: the frame schedule and its labels. Serializable,
/// content-addressable, and independent of the animations that produced it.
#[derive(Clone, Debug, PartialEq)]
pub struct TimelinePlan {
    fps: u32,
    segments: Vec<PlannedSegment>,
    labels: Vec<Label>,
}

impl TimelinePlan {
    /// The frame rate the schedule was compiled at.
    #[must_use]
    pub fn fps(&self) -> u32 {
        self.fps
    }

    /// The segments, in authored order.
    #[must_use]
    pub fn segments(&self) -> &[PlannedSegment] {
        &self.segments
    }

    /// The labels, in authored order.
    #[must_use]
    pub fn labels(&self) -> &[Label] {
        &self.labels
    }

    /// Total frames the timeline emits.
    #[must_use]
    pub fn total_frames(&self) -> i64 {
        self.segments
            .last()
            .and_then(PlannedSegment::end_frame)
            .unwrap_or(0)
    }

    /// The exact duration of the schedule on the frame grid, in seconds.
    #[must_use]
    pub fn duration(&self) -> f64 {
        self.total_frames() as f64 / f64::from(self.fps)
    }

    /// Which segment a global frame belongs to, and its 1-based position
    /// inside that segment. `None` outside `1..=total_frames`.
    #[must_use]
    pub fn locate(&self, frame: i64) -> Option<(usize, i64)> {
        if frame < 1 {
            return None;
        }
        self.segments
            .iter()
            .position(|s| frame > s.base_frame && s.end_frame().is_some_and(|end| frame <= end))
            .map(|index| (index, frame - self.segments[index].base_frame))
    }

    /// The frame covering `seconds` on the grid: sample *k* spans
    /// `((k-1)/fps, k/fps]`, so an instant maps to `ceil(seconds · fps)`,
    /// clamped into the timeline. Time `0` maps to the first frame; a
    /// non-finite request clamps rather than panicking (the mapping stays
    /// total, as the alpha pipeline does).
    #[must_use]
    pub fn frame_at_time(&self, seconds: f64) -> i64 {
        let total = self.total_frames();
        if total == 0 {
            return 0;
        }
        let raw = (seconds * f64::from(self.fps)).ceil();
        if raw.is_nan() {
            return 1;
        }
        // `as` saturates at the integer bounds for finite and infinite
        // inputs alike, and the clamp brings it into the schedule.
        (raw as i64).clamp(1, total)
    }

    /// The frame a label resolves to.
    #[must_use]
    pub fn frame_of_label(&self, name: &str) -> Option<i64> {
        self.labels
            .iter()
            .find(|label| label.name == name)
            .map(|label| label.frame)
    }

    /// Encode canonically (`FMNA/5`): fps, the segment table, the label
    /// table. Field order is fixed and floats canonicalize at the boundary,
    /// so equal schedules always produce equal bytes on every platform.
    ///
    /// # Errors
    /// [`SerialError`] when either table exceeds the `u32` count-field
    /// width, or the document exceeds the container's limits.
    pub fn to_bytes(&self) -> Result<Vec<u8>, SerialError> {
        // Both wire counts are validated up front, so an over-wide schedule
        // refuses whole instead of emitting bytes whose declared count
        // disagrees with their payload.
        let segment_count = wire_count(self.segments.len())?;
        let label_count = wire_count(self.labels.len())?;
        let mut writer = Writer::new(TIMELINE_SCHEMA);
        writer.put_u32(self.fps);
        writer.put_u32(segment_count);
        for segment in &self.segments {
            writer.put_u8(match segment.kind {
                SegmentKind::Play => 0,
                SegmentKind::Wait => 1,
            });
            writer.put_f64(segment.run_time);
            writer.put_i64(segment.base_frame);
            writer.put_i64(segment.n_frames);
        }
        writer.put_u32(label_count);
        for label in &self.labels {
            writer.put_str(&label.name);
            writer.put_u64(
                u64::try_from(label.segment).map_err(|_| SerialError::SizeLimit {
                    limit: usize::try_from(u64::MAX).unwrap_or(usize::MAX),
                    needed: label.segment,
                })?,
            );
            writer.put_i64(label.frame);
        }
        writer.finish()
    }

    /// Decode a canonical plan. Hostile bytes are a named error, never a
    /// panic and never a half-built schedule.
    ///
    /// # Errors
    /// [`TimelineError`] for any framing, version, integrity, field, or
    /// invariant failure.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, TimelineError> {
        let mut reader = Reader::open(
            bytes,
            TIMELINE_SCHEMA,
            Limits::DEFAULT,
            UnknownPolicy::Strict,
        )?;
        let fps = reader.get_u32()?;
        let mut clock = RationalFrameClock::new(fps).map_err(decoded_clock_error)?;
        let segment_count = usize::try_from(reader.get_u32()?)
            .map_err(|_| TimelineError::Malformed("segment count"))?;
        let mut segments = Vec::new();
        for _ in 0..segment_count {
            let kind = match reader.get_u8()? {
                0 => SegmentKind::Play,
                1 => SegmentKind::Wait,
                _ => return Err(TimelineError::Malformed("segment kind")),
            };
            let segment = PlannedSegment {
                kind,
                run_time: reader.get_f64()?,
                base_frame: reader.get_i64()?,
                n_frames: reader.get_i64()?,
            };
            if segment.base_frame != clock.now().frames() {
                return Err(TimelineError::Malformed("segment base frame"));
            }
            let expected = clock
                .segment(segment.run_time)
                .map_err(decoded_clock_error)?;
            if segment.n_frames != expected.n_frames() {
                return Err(TimelineError::Malformed("segment frame count"));
            }
            clock
                .advance_frames(segment.n_frames)
                .map_err(decoded_clock_error)?;
            segments.push(segment);
        }
        let total_frames = clock.now().frames();
        let label_count = usize::try_from(reader.get_u32()?)
            .map_err(|_| TimelineError::Malformed("label count"))?;
        let mut labels = Vec::new();
        let mut previous_label_segment = None;
        for _ in 0..label_count {
            let name = reader.get_str()?.to_owned();
            let segment = usize::try_from(reader.get_u64()?)
                .map_err(|_| TimelineError::Malformed("label segment index"))?;
            let frame = reader.get_i64()?;
            let expected_frame = label_frame_at_or_after(&segments, segment, total_frames)
                .ok_or(TimelineError::Malformed("label segment index"))?;
            if previous_label_segment.is_some_and(|previous| segment < previous) {
                return Err(TimelineError::Malformed("label order"));
            }
            if frame != expected_frame {
                return Err(TimelineError::Malformed("label frame"));
            }
            previous_label_segment = Some(segment);
            labels.push(Label {
                name,
                segment,
                frame,
            });
        }
        reader.finish()?;
        Ok(Self {
            fps,
            segments,
            labels,
        })
    }

    /// The schedule's content address — the digest of its canonical bytes.
    ///
    /// # Errors
    /// [`SerialError`] from [`TimelinePlan::to_bytes`].
    pub fn content_id(&self) -> Result<Digest, SerialError> {
        Ok(sha256(&self.to_bytes()?))
    }
}

// ---------------------------------------------------------------- timeline

/// An authored timeline: steps, labels, and the checkpoints seek rides on.
pub struct Timeline {
    fps: u32,
    steps: Vec<Step>,
    labels: Vec<(String, usize)>,
    /// Segment index → the arena state at that segment's *start*. Index `0`
    /// is the timeline's base state, captured the first time it runs.
    checkpoints: BTreeMap<usize, Rc<Snapshot>>,
}

impl Timeline {
    /// An empty timeline on `fps`.
    ///
    /// # Errors
    /// [`AnimError::Clock`] for an fps the rational clock refuses.
    pub fn new(fps: u32) -> Result<Self, AnimError> {
        RationalFrameClock::new(fps).map_err(AnimError::Clock)?;
        Ok(Self {
            fps,
            steps: Vec::new(),
            labels: Vec::new(),
            checkpoints: BTreeMap::new(),
        })
    }

    /// The frame rate.
    #[must_use]
    pub fn fps(&self) -> u32 {
        self.fps
    }

    /// The authored steps.
    #[must_use]
    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    /// How many steps are authored.
    #[must_use]
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// Whether nothing is authored yet.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Author a `play()` step.
    ///
    /// # Errors
    /// [`AnimError::EmptyComposition`] — a play with no animations has no
    /// run time to schedule.
    pub fn play(&mut self, animations: Vec<Box<dyn Animation>>) -> Result<&mut Self, AnimError> {
        if animations.is_empty() {
            return Err(AnimError::EmptyComposition);
        }
        self.invalidate();
        self.steps.push(Step::Play(animations));
        Ok(self)
    }

    /// Author a `wait()` step.
    ///
    /// # Errors
    /// [`AnimError::Clock`] for a duration the clock refuses (non-finite, or
    /// beyond the frame counter's range).
    pub fn wait(&mut self, duration: f64) -> Result<&mut Self, AnimError> {
        RationalFrameClock::new(self.fps)
            .and_then(|clock| clock.segment(duration))
            .map_err(AnimError::Clock)?;
        self.invalidate();
        self.steps.push(Step::Wait(duration));
        Ok(self)
    }

    /// Name the position *about to be* authored — the start of the next
    /// step (or the timeline's end, if nothing follows).
    pub fn label(&mut self, name: &str) -> &mut Self {
        self.labels.push((name.to_owned(), self.steps.len()));
        self
    }

    /// Compile the schedule. Pure: nothing is begun, no stage is touched.
    ///
    /// # Errors
    /// [`AnimError::Clock`] for a step whose run time the clock refuses.
    pub fn compile(&self) -> Result<TimelinePlan, AnimError> {
        let mut clock = RationalFrameClock::new(self.fps).map_err(AnimError::Clock)?;
        let mut segments = Vec::with_capacity(self.steps.len());
        for step in &self.steps {
            let run_time = step.run_time();
            let n_frames = clock
                .segment(run_time)
                .map_err(AnimError::Clock)?
                .n_frames();
            segments.push(PlannedSegment {
                kind: step.kind(),
                run_time,
                base_frame: clock.now().frames(),
                n_frames,
            });
            clock.advance_frames(n_frames).map_err(AnimError::Clock)?;
        }
        let total_frames = clock.now().frames();
        let labels = self
            .labels
            .iter()
            .map(|(name, index)| Label {
                name: name.clone(),
                segment: *index,
                frame: label_frame_at_or_after(&segments, *index, total_frames)
                    .unwrap_or(total_frames),
            })
            .collect();
        Ok(TimelinePlan {
            fps: self.fps,
            segments,
            labels,
        })
    }

    /// Play the whole timeline from the current stage state, emitting every
    /// frame in order and returning one [`SegmentReport`] per step (the
    /// replay journal's record, §13.4).
    ///
    /// Checkpoints are recorded at every segment boundary as it passes, so a
    /// subsequent [`Timeline::seek`] into an already-rendered region starts
    /// from the nearest one instead of the beginning.
    ///
    /// # Errors
    /// Whatever the drivers report — [`AnimError`] from `begin`, the clock,
    /// or a composition's deferred failure.
    pub fn render(
        &mut self,
        stage: &mut Stage,
        rng: &RngRoot,
        emit: &mut dyn FnMut(FramePacket),
    ) -> Result<Vec<SegmentReport>, AnimError> {
        let mut clock = RationalFrameClock::new(self.fps).map_err(AnimError::Clock)?;
        self.checkpoints.clear();
        self.checkpoints.insert(0, Rc::new(stage.snapshot()));
        let mut reports = Vec::with_capacity(self.steps.len());
        for index in 0..self.steps.len() {
            reports.push(run_step(
                &mut self.steps[index],
                stage,
                &mut clock,
                rng,
                emit,
            )?);
            self.checkpoints
                .insert(index + 1, Rc::new(stage.snapshot()));
        }
        Ok(reports)
    }

    /// Reconstruct the state at a global frame and return that frame's
    /// packet. Deterministic and repeatable: seeking to *f* twice, or
    /// seeking backwards and forwards, produces the same frame the straight
    /// play-through produced.
    ///
    /// The route: restore the nearest checkpoint at or before the target
    /// segment, replay whole segments up to it (recording checkpoints as it
    /// goes), then land inside the target — in O(1) from its begin-state
    /// snapshot if it is **pure** (§9.5), or by stepping its frames if it is
    /// **stateful**, because accumulated per-frame state has no shortcut.
    ///
    /// # Errors
    /// [`AnimError::SeekOutOfRange`] for a frame outside the schedule;
    /// otherwise whatever the drivers report.
    pub fn seek(
        &mut self,
        stage: &mut Stage,
        rng: &RngRoot,
        frame: i64,
    ) -> Result<FramePacket, AnimError> {
        let plan = self.compile()?;
        let (target, offset) = plan
            .locate(frame)
            .ok_or_else(|| AnimError::SeekOutOfRange {
                frame,
                total: plan.total_frames(),
            })?;
        self.checkpoints
            .entry(0)
            .or_insert_with(|| Rc::new(stage.snapshot()));
        // The nearest recorded checkpoint at or before the target segment.
        let (mut index, state) = self
            .checkpoints
            .range(..=target)
            .next_back()
            .map(|(index, state)| (*index, Rc::clone(state)))
            .expect("checkpoint 0 was just ensured");
        stage.restore(&state);
        let mut clock = RationalFrameClock::new(self.fps).map_err(AnimError::Clock)?;
        clock
            .advance_frames(plan.segments[index].base_frame)
            .map_err(AnimError::Clock)?;
        stage.set_time_from_clock(clock.now().to_f64());
        let mut discard = |_: FramePacket| {};
        while index < target {
            run_step(&mut self.steps[index], stage, &mut clock, rng, &mut discard)?;
            index += 1;
            self.checkpoints.insert(index, Rc::new(stage.snapshot()));
        }
        seek_within(&mut self.steps[target], stage, &mut clock, rng, offset)
    }

    /// Forget every recorded checkpoint (the next seek replays from the
    /// timeline's base state). Authoring calls this implicitly: a schedule
    /// that changed invalidates the states recorded under the old one.
    pub fn clear_checkpoints(&mut self) {
        let base = self.checkpoints.remove(&0);
        self.checkpoints.clear();
        if let Some(base) = base {
            self.checkpoints.insert(0, base);
        }
    }

    /// Authoring invalidates recorded mid-timeline checkpoints.
    fn invalidate(&mut self) {
        self.clear_checkpoints();
    }

    /// Decompose into the authored steps and label anchors (fm-oee's
    /// bundle exporter drives the public segment drivers itself so it can
    /// snapshot every segment boundary; callers [`Timeline::compile`]
    /// first for the schedule, then consume the timeline with this).
    #[must_use]
    pub fn into_steps(self) -> (Vec<Step>, Vec<(String, usize)>) {
        (self.steps, self.labels)
    }

    /// The segment indices with a recorded checkpoint (diagnostic; the
    /// Studio's scrub bar shows these as the cheap seek targets).
    #[must_use]
    pub fn checkpointed_segments(&self) -> Vec<usize> {
        self.checkpoints.keys().copied().collect()
    }
}

/// Run one whole step through the ordinary drivers.
fn run_step(
    step: &mut Step,
    stage: &mut Stage,
    clock: &mut RationalFrameClock,
    rng: &RngRoot,
    emit: &mut dyn FnMut(FramePacket),
) -> Result<SegmentReport, AnimError> {
    match step {
        Step::Play(animations) => play_segment(stage, clock, rng, animations, false, emit),
        Step::Wait(duration) => wait_segment(stage, clock, rng, *duration, None, false, emit),
    }
}

/// Land on frame `offset` (1-based) inside `step`, whose start the caller
/// has already positioned the stage and clock at.
fn seek_within(
    step: &mut Step,
    stage: &mut Stage,
    clock: &mut RationalFrameClock,
    rng: &RngRoot,
    offset: i64,
) -> Result<FramePacket, AnimError> {
    let mut last: Option<FramePacket> = None;
    let mut capture = |packet: FramePacket| last = Some(packet);
    match step {
        Step::Play(animations) => {
            let mut open = open_play(stage, clock, animations)?;
            if open.report().purity.is_pure() {
                // §9.5's contract: frame *k* is a function of (begin
                // snapshot, α(k), keyed RNG fork) — one interpolation, no
                // replay, whatever k is.
                let sample = nth_sample(clock, open.report().run_time, offset)?;
                return reconstruct_pure_frame(stage, animations, open.report(), rng, &sample);
            }
            // Stateful: the frames *are* the state, so the replay is the
            // answer. The segment stays open — the caller may seek forward
            // again from here, or restore over it.
            advance_play(
                stage,
                clock,
                rng,
                animations,
                &mut open,
                offset,
                &mut capture,
            )?;
            last.ok_or(AnimError::SeekOutOfRange {
                frame: offset,
                total: open.segment().n_frames(),
            })
        }
        Step::Wait(duration) => {
            wait_segment_upto(stage, clock, rng, *duration, offset, &mut capture)?;
            last.ok_or(AnimError::SeekOutOfRange {
                frame: offset,
                total: 0,
            })
        }
    }
}

/// The `n`th (1-based) sample of a segment of `run_time` on this clock's
/// grid.
fn nth_sample(clock: &RationalFrameClock, run_time: f64, n: i64) -> Result<FrameSample, AnimError> {
    let segment = clock.segment(run_time).map_err(AnimError::Clock)?;
    segment
        .samples()
        .nth(usize::try_from(n - 1).unwrap_or(usize::MAX))
        .ok_or(AnimError::SeekOutOfRange {
            frame: n,
            total: segment.n_frames(),
        })
}

/// A collection length as this format's `u32` count field: a typed refusal
/// when the schedule exceeds the wire width, never a saturated value.
fn wire_count(needed: usize) -> Result<u32, SerialError> {
    u32::try_from(needed).map_err(|_| SerialError::SizeLimit {
        limit: usize::try_from(u32::MAX).unwrap_or(usize::MAX),
        needed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_above_the_wire_width_are_a_typed_refusal() {
        let max = usize::try_from(u32::MAX).expect("32-bit width fits usize");
        assert_eq!(wire_count(max), Ok(u32::MAX));
        let one_over = max + 1;
        assert_eq!(
            wire_count(one_over),
            Err(SerialError::SizeLimit {
                limit: max,
                needed: one_over
            })
        );
    }
}
