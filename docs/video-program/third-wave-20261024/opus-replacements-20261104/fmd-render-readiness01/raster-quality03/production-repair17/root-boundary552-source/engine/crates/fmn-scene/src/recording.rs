//! Bounded recording of imperative scenes into the existing FMTL/1 format.
//!
//! Unlike the declarative timeline exporter, this sink never runs an animation
//! or updater. It records the real Scene's captures as kind-1 snapshots. A
//! callback's observed output is replayable; its callable is not serialized and
//! its purity is not inferred. The ordinary bundle reader and native/WASM
//! players remain authoritative.
//!
//! Segment durations describe the *captured frame grid*, not the original
//! floating-point play argument (which a SceneSink does not receive). Explicit
//! `show()` captures are one-frame holds. Skipped previews and presenter input
//! are refused rather than silently retimed. Minor 1 can record the exact camera
//! sampled by a composition root; no rig binding or camera callback is inferred.
//! Audio remains unsupported and must be refused by composition roots.

use fmn_anim::timeline::{TIMELINE_SCHEMA, TimelineError, TimelinePlan};
use fmn_anim::{FramePacket, RationalFrameClock, SegmentKind};
use fmn_hash::serial::{Limits, Writer};
use fmn_hash::{Digest, SerialError, sha256};
use fmn_mobject::{RenderSnapshotError, Snapshot, Stage};
use fmn_render::Camera;
use fmn_render::camera::CameraSample;

use crate::timeline_bundle::{
    BundleError, BundleExportLimits, BundleReadError, CAMERA_TIMELINE_BUNDLE_SCHEMA,
    TIMELINE_BUNDLE_SCHEMA, TimelineBundle, bundle_engine_version,
};
use crate::{CaptureReason, IntegrationError, LifecycleEvent, LifecyclePhase, SceneSink};

/// A failed recording cannot be finalized into a successful partial artifact.
#[derive(Debug)]
pub enum RecordingError {
    /// Capture, allocation, frame-work or container-size refusal.
    Bundle(BundleError),
    /// A render-only frame could not be projected safely.
    Projection(RenderSnapshotError),
    /// The recorded schedule did not satisfy the shared clock/plan contract.
    Plan(TimelineError),
    /// The finished artifact did not satisfy the production reader.
    Decode(BundleReadError),
    /// Encoding was refused before growing beyond the artifact budget.
    OutputLimit {
        /// Size of the next canonical append that would exceed the cap.
        needed: usize,
        /// Effective output cap, no larger than the production reader limit.
        limit: usize,
    },
    /// A capture protocol or capability that FMTL/1 cannot represent.
    Unsupported(&'static str),
}

impl std::fmt::Display for RecordingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bundle(error) => error.fmt(f),
            Self::Projection(error) => error.fmt(f),
            Self::Plan(error) => error.fmt(f),
            Self::Decode(error) => error.fmt(f),
            Self::OutputLimit { needed, limit } => write!(
                f,
                "scene bundle needs {needed} bytes, exceeding the {limit}-byte output budget"
            ),
            Self::Unsupported(message) => write!(f, "scene bundle recording: {message}"),
        }
    }
}

impl std::error::Error for RecordingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Bundle(error) => Some(error),
            Self::Projection(error) => Some(error),
            Self::Plan(error) => Some(error),
            Self::Decode(error) => Some(error),
            Self::OutputLimit { .. } | Self::Unsupported(_) => None,
        }
    }
}

impl From<BundleError> for RecordingError {
    fn from(error: BundleError) -> Self {
        Self::Bundle(error)
    }
}

impl From<SerialError> for RecordingError {
    fn from(error: SerialError) -> Self {
        Self::Bundle(BundleError::Serial(error))
    }
}

/// A validated, code-free artifact from a completed recording.
#[derive(Debug)]
pub struct RecordedSceneBundle {
    /// Canonical FMTL/1 bytes accepted by TimelineBundle and its shared player.
    pub bytes: Vec<u8>,
    /// Content address of exactly `bytes`.
    pub digest: Digest,
    /// Number of output frames, including explicit still captures.
    pub frame_count: u32,
    /// Number of captured play/wait segments and explicit still holds.
    pub segment_count: usize,
}

struct RecordedSegment {
    kind: SegmentKind,
    frames: Vec<Vec<u8>>,
}

struct ActiveSegment {
    recorded: RecordedSegment,
    base_frame: i64,
    play_index: u64,
}

/// A SceneSink for arbitrary imperative native or host-language scene programs.
///
/// Feed it the real lifecycle events and captures, then call [`Self::finish`]
/// only after the whole program has succeeded. Any sink failure is sticky.
/// Capture storage is charged cumulatively, including destination tables; frame
/// admission is checked before serializing the next snapshot. As with Scene's
/// other sinks, an error stops capture, not the deterministic completion of the
/// already-running animation segment.
pub struct SceneBundleRecorder {
    fps: u32,
    limits: BundleExportLimits,
    charged: usize,
    frames: u64,
    segments: Vec<RecordedSegment>,
    active: Option<ActiveSegment>,
    failure: Option<RecordingError>,
    render_only: bool,
    cameras: Option<Vec<CameraSample>>,
}

impl SceneBundleRecorder {
    /// Construct a full-state recorder at the Scene's effective frame rate.
    ///
    /// Retains the existing complete-arena snapshot contract. Portable picture
    /// exports can select [`Self::new_render_only`] instead; this constructor
    /// never drops authoring state or renumbers live snapshot identities.
    ///
    /// # Errors
    /// Refuses a zero frame rate before any scene work.
    pub fn new(fps: u32, limits: BundleExportLimits) -> Result<Self, RecordingError> {
        if fps == 0 {
            return Err(RecordingError::Unsupported("fps must be nonzero"));
        }
        Ok(Self {
            fps,
            limits,
            charged: 0,
            frames: 0,
            segments: Vec::new(),
            active: None,
            failure: None,
            render_only: false,
            cameras: None,
        })
    }

    /// Record only each captured frame's rooted render state.
    ///
    /// Unrooted animation copies and saved states do not consume every frame's
    /// byte budget. The shared FMNA codec, FMTL reader, rational clock and
    /// sticky failure/publication rules are unchanged. This mode must not be
    /// used to implement undo or reconstruct a live scene for further editing:
    /// handles are canonicalized and executable/restoration metadata is absent.
    /// It does not reclaim the source arena or avoid the initial FramePacket.
    ///
    /// # Errors
    /// As [`Self::new`]. Per-frame projection failures are reported by capture.
    pub fn new_render_only(fps: u32, limits: BundleExportLimits) -> Result<Self, RecordingError> {
        let mut recorder = Self::new(fps, limits)?;
        recorder.render_only = true;
        Ok(recorder)
    }

    /// Record render-only snapshots and their exact camera/light/background
    /// captures in FMTL/1 minor 1. Existing constructors still emit minor 0.
    /// Every frame must use `capture_with_camera` (including a terminal still);
    /// missing camera samples are sticky failures, never default-camera frames.
    ///
    /// # Errors
    /// Refuses zero FPS; per-frame values and budgets are checked at capture.
    pub fn new_render_only_with_camera(
        fps: u32,
        limits: BundleExportLimits,
    ) -> Result<Self, RecordingError> {
        let mut recorder = Self::new_render_only(fps, limits)?;
        recorder.cameras = Some(Vec::new());
        Ok(recorder)
    }

    fn encode_snapshot(&self, snapshot: &Snapshot) -> Result<Vec<u8>, RecordingError> {
        if self.render_only {
            snapshot
                .to_render_bytes()
                .map_err(RecordingError::Projection)
        } else {
            snapshot.to_bytes().map_err(RecordingError::from)
        }
    }

    /// Captures admitted so far. This is output order, not the Scene clock.
    #[must_use]
    pub fn frame_count(&self) -> u64 {
        self.frames
    }

    /// Consume a failed recorder to recover the original typed sink failure.
    /// Consuming it prevents error recovery from publishing a truncated bundle.
    #[must_use]
    pub fn into_error(self) -> Option<RecordingError> {
        self.failure
    }

    /// Capture the terminal state of a program that emitted no frames.
    ///
    /// This is explicit, matching native rendering's one-still policy. It does
    /// not advance the source Scene clock or invoke any updater.
    ///
    /// # Errors
    /// Refuses a nonempty/active/failed recording or exhausted capture budgets.
    pub fn capture_terminal_still(&mut self, stage: &Stage) -> Result<(), IntegrationError> {
        self.terminal_still(stage, None)
    }

    /// Record a static scene and its camera without advancing the scene clock.
    ///
    /// # Errors
    /// As `capture_terminal_still`, plus invalid camera or recording-mode errors.
    pub fn capture_terminal_still_with_camera(
        &mut self,
        stage: &Stage,
        camera: &Camera,
    ) -> Result<(), IntegrationError> {
        self.terminal_still(stage, Some(camera))
    }

    fn terminal_still(
        &mut self,
        stage: &Stage,
        camera: Option<&Camera>,
    ) -> Result<(), IntegrationError> {
        let result = if self.frames != 0 || self.active.is_some() {
            Err(RecordingError::Unsupported(
                "terminal still requires an idle empty recording",
            ))
        } else {
            self.admit_frame().and_then(|()| {
                let sample = self.camera_sample(camera)?;
                let bytes = self.encode_snapshot(&stage.snapshot());
                self.capture_snapshot(bytes, sample)
            })
        };
        self.retain_failure(result)
    }

    fn charge(&mut self, additional: usize, context: &'static str) -> Result<(), RecordingError> {
        let overflow = BundleError::CaptureLimitExceeded {
            context,
            needed: usize::MAX,
            limit: self.limits.max_capture_bytes,
        };
        let needed = self.charged.checked_add(additional).ok_or(overflow)?;
        if needed > self.limits.max_capture_bytes {
            return Err(BundleError::CaptureLimitExceeded {
                context,
                needed,
                limit: self.limits.max_capture_bytes,
            }
            .into());
        }
        self.charged = needed;
        Ok(())
    }

    fn reserve<T>(
        &mut self,
        table: &mut Vec<T>,
        context: &'static str,
    ) -> Result<(), RecordingError> {
        if table.len() == table.capacity() {
            // Geometric, fallible growth rather than reallocating on each frame.
            let additional = table.capacity().max(4);
            let overflow = BundleError::CaptureLimitExceeded {
                context,
                needed: usize::MAX,
                limit: self.limits.max_capture_bytes,
            };
            let bytes = additional
                .checked_mul(std::mem::size_of::<T>())
                .ok_or(overflow)?;
            self.charge(bytes, context)?;
            table
                .try_reserve_exact(additional)
                .map_err(|_| BundleError::AllocationFailed {
                    context,
                    requested: additional,
                })?;
        }
        Ok(())
    }

    fn admit_frame(&self) -> Result<(), RecordingError> {
        if self.failure.is_some() {
            return Err(RecordingError::Unsupported("recorder is already failed"));
        }
        let max_frames = self.limits.max_frames.min(u64::from(u32::MAX));
        if self.frames >= max_frames {
            return Err(BundleError::FrameLimitExceeded {
                frames: self.frames.saturating_add(1),
                max_frames,
            }
            .into());
        }
        Ok(())
    }

    fn append_segment(&mut self, segment: RecordedSegment) -> Result<(), RecordingError> {
        let mut segments = std::mem::take(&mut self.segments);
        let result = self.reserve(&mut segments, "recorded segment table");
        if result.is_ok() {
            segments.push(segment);
        }
        self.segments = segments;
        result
    }

    fn camera_sample(
        &self,
        camera: Option<&Camera>,
    ) -> Result<Option<CameraSample>, RecordingError> {
        if self.cameras.is_some() != camera.is_some() {
            return Err(RecordingError::Unsupported(
                "camera capture must match the recorder's declared mode",
            ));
        }
        camera
            .map(|camera| {
                if camera.fps() != self.fps {
                    return Err(RecordingError::Unsupported("camera frame rate changed"));
                }
                CameraSample::capture(camera).map_err(|error| {
                    RecordingError::Decode(BundleReadError::Camera(
                        fmn_render::camera::CameraSampleError::Camera(error),
                    ))
                })
            })
            .transpose()
    }

    fn capture_snapshot(
        &mut self,
        bytes: Result<Vec<u8>, RecordingError>,
        camera: Option<CameraSample>,
    ) -> Result<(), RecordingError> {
        self.admit_frame()?;
        let bytes = bytes?;
        if let Some(sample) = camera {
            // Charge the retained table AND serialization work, as for snapshots.
            self.charge(CameraSample::WIRE_BYTES, "recorded camera sample")?;
            let mut cameras = self
                .cameras
                .take()
                .ok_or(RecordingError::Unsupported("unexpected camera capture"))?;
            let result = self.reserve(&mut cameras, "recorded camera table");
            if result.is_ok() {
                cameras.push(sample);
            }
            self.cameras = Some(cameras);
            result?;
        }
        self.charge(bytes.len(), "recorded frame snapshot")?;
        let mut frames = self.active.as_mut().map_or_else(Vec::new, |active| {
            std::mem::take(&mut active.recorded.frames)
        });
        self.reserve(&mut frames, "recorded frame table")?;
        frames.push(bytes);
        if let Some(active) = &mut self.active {
            active.recorded.frames = frames;
        } else {
            self.append_segment(RecordedSegment {
                kind: SegmentKind::Wait,
                frames,
            })?;
        }
        self.frames += 1;
        Ok(())
    }

    fn observe(&mut self, event: LifecycleEvent) -> Result<(), RecordingError> {
        if event.time.fps() != self.fps {
            return Err(RecordingError::Unsupported("lifecycle frame rate changed"));
        }
        match event.phase {
            LifecyclePhase::DriveSegment => {
                if event.skipping || self.active.is_some() {
                    return Err(RecordingError::Unsupported("skipped or nested segment"));
                }
                self.active = Some(ActiveSegment {
                    recorded: RecordedSegment {
                        kind: event
                            .segment
                            .ok_or(RecordingError::Unsupported("segment kind missing"))?,
                        frames: Vec::new(),
                    },
                    base_frame: event.time.frames(),
                    play_index: event.play_index,
                });
            }
            LifecyclePhase::FinishSegment => {
                let active = self.active.take().ok_or(RecordingError::Unsupported(
                    "segment finished without beginning",
                ))?;
                if Some(active.recorded.kind) != event.segment
                    || active.play_index != event.play_index
                {
                    return Err(RecordingError::Unsupported("segment identity changed"));
                }
                let count = i64::try_from(active.recorded.frames.len())
                    .map_err(|_| RecordingError::Unsupported("frame count exceeds the clock"))?;
                if active.base_frame.checked_add(count) != Some(event.time.frames()) {
                    return Err(RecordingError::Unsupported(
                        "segment finish does not match captured frame count",
                    ));
                }
                self.append_segment(active.recorded)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn retain_failure(
        &mut self,
        result: Result<(), RecordingError>,
    ) -> Result<(), IntegrationError> {
        if self.failure.is_none() {
            self.failure = result.err();
        }
        match &self.failure {
            Some(error) => Err(IntegrationError::new("fmtl-recording", error.to_string())),
            None => Ok(()),
        }
    }

    /// Finish as a canonical artifact, checking the existing production reader.
    ///
    /// # Errors
    /// Returns the first capture failure, unfinished-segment errors, or canonical
    /// size/reader refusals. No file is created by this method.
    pub fn finish(self) -> Result<RecordedSceneBundle, RecordingError> {
        self.finish_with_max_bytes(Limits::DEFAULT.max_total)
    }

    /// Finish under an explicit canonical output cap, enforced before growth.
    ///
    /// Both the nested schedule and the complete bundle use bounded writers.
    /// Captured snapshots are released as they are encoded, before the finished
    /// bytes are decoded for validation. Capture and decoder budgets remain
    /// separate; this is not a bound on arbitrary scene-code allocations.
    ///
    /// # Errors
    /// As [`Self::finish`], plus [`RecordingError::OutputLimit`] when the next
    /// append would exceed the smaller of `max_bytes` and the reader's limit.
    pub fn finish_with_max_bytes(
        self,
        max_bytes: usize,
    ) -> Result<RecordedSceneBundle, RecordingError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        if self.active.is_some() {
            return Err(RecordingError::Unsupported("scene ended inside a segment"));
        }
        let count = u32::try_from(self.segments.len())
            .map_err(|_| RecordingError::Unsupported("segment count exceeds FMTL/1"))?;
        // Encode the existing FMNA/5 schedule and validate it with its sole
        // authoritative decoder. No new clock or playback law is introduced.
        let limits = Limits {
            max_total: max_bytes.min(Limits::DEFAULT.max_total),
            ..Limits::DEFAULT
        };
        let encoding_error = |error| match error {
            SerialError::SizeLimit { needed, limit } if limit == limits.max_total => {
                RecordingError::OutputLimit { needed, limit }
            }
            error => RecordingError::from(error),
        };
        let mut schedule = Writer::with_limits(TIMELINE_SCHEMA, limits);
        schedule.put_u32(self.fps);
        schedule.put_u32(count);
        let mut clock = RationalFrameClock::new(self.fps)
            .map_err(|_| RecordingError::Unsupported("fps must be nonzero"))?;
        for segment in &self.segments {
            let n = i64::try_from(segment.frames.len())
                .map_err(|_| RecordingError::Unsupported("frame count exceeds the clock"))?;
            // A rounded-up f64 quotient can create an extra frame (e.g. 3/30).
            // Choose the immediately lower value and verify with the exact clock.
            let duration = if n == 0 {
                0.0
            } else {
                (n as f64 / f64::from(self.fps)).next_down()
            };
            let sampled = clock
                .segment(duration)
                .map_err(fmn_anim::AnimError::Clock)
                .map_err(BundleError::Anim)?;
            if sampled.n_frames() != n {
                return Err(RecordingError::Unsupported(
                    "recorded duration does not fit the frame grid",
                ));
            }
            schedule.put_u8(match segment.kind {
                SegmentKind::Play => 0,
                SegmentKind::Wait => 1,
            });
            schedule.put_f64(duration);
            schedule.put_i64(clock.now().frames());
            schedule.put_i64(n);
            clock
                .advance_frames(n)
                .map_err(fmn_anim::AnimError::Clock)
                .map_err(BundleError::Anim)?;
        }
        schedule.put_u32(0); // Imperative Scene has no authored Timeline labels.
        let plan_bytes = schedule.finish().map_err(encoding_error)?;
        TimelinePlan::from_bytes(&plan_bytes).map_err(RecordingError::Plan)?;
        let schema = if self.cameras.is_some() {
            CAMERA_TIMELINE_BUNDLE_SCHEMA
        } else {
            TIMELINE_BUNDLE_SCHEMA
        };
        let mut writer = Writer::with_limits(schema, limits);
        writer.put_str(&bundle_engine_version());
        writer.put_u32(self.fps);
        writer.put_bytes(&plan_bytes);
        writer.put_u32(count);
        for segment in self.segments {
            writer.put_u8(1); // Observed snapshots, never an unproven pure law.
            writer.put_u32(
                u32::try_from(segment.frames.len()).map_err(|_| {
                    RecordingError::Unsupported("segment frame count exceeds FMTL/1")
                })?,
            );
            for frame in segment.frames {
                writer.put_bytes(&frame);
            }
        }
        if let Some(cameras) = self.cameras {
            if cameras.len() as u64 != self.frames {
                return Err(RecordingError::Unsupported("camera/frame count mismatch"));
            }
            writer.put_u32(
                u32::try_from(cameras.len())
                    .map_err(|_| RecordingError::Unsupported("camera count exceeds FMTL/1"))?,
            );
            for camera in cameras {
                camera.write_to(&mut writer);
            }
        }
        let bytes = writer.finish().map_err(encoding_error)?;
        let decoded = TimelineBundle::from_bytes(&bytes).map_err(RecordingError::Decode)?;
        Ok(RecordedSceneBundle {
            digest: sha256(&bytes),
            frame_count: decoded.frame_count(),
            segment_count: decoded.segment_count(),
            bytes,
        })
    }
}

impl SceneSink for SceneBundleRecorder {
    fn event(&mut self, event: LifecycleEvent) -> Result<(), IntegrationError> {
        let result = if self.failure.is_some() {
            Ok(())
        } else {
            self.observe(event)
        };
        self.retain_failure(result)
    }

    fn capture(
        &mut self,
        reason: CaptureReason,
        packet: FramePacket,
    ) -> Result<(), IntegrationError> {
        self.capture_packet(reason, packet, None)
    }
}

impl SceneBundleRecorder {
    /// Freeze a camera sampled at the same post-updater boundary as this packet.
    /// The caller owns camera synchronization; no callback executes here.
    ///
    /// # Errors
    /// Preserves sticky capture, clock, camera, schema and budget failures.
    pub fn capture_with_camera(
        &mut self,
        reason: CaptureReason,
        packet: FramePacket,
        camera: &Camera,
    ) -> Result<(), IntegrationError> {
        self.capture_packet(reason, packet, Some(camera))
    }

    fn capture_packet(
        &mut self,
        reason: CaptureReason,
        packet: FramePacket,
        camera: Option<&Camera>,
    ) -> Result<(), IntegrationError> {
        let result = (|| {
            self.admit_frame()?;
            if packet.time().fps() != self.fps {
                return Err(RecordingError::Unsupported("capture frame rate changed"));
            }
            match (reason, &self.active) {
                (CaptureReason::Segment, Some(active)) => {
                    let next = i64::try_from(active.recorded.frames.len())
                        .ok()
                        .and_then(|count| count.checked_add(1))
                        .ok_or(RecordingError::Unsupported("frame count exceeds the clock"))?;
                    if packet.segment_frame() != next
                        || active.base_frame.checked_add(next) != Some(packet.time().frames())
                    {
                        return Err(RecordingError::Unsupported(
                            "missing, duplicated or reordered segment capture",
                        ));
                    }
                }
                (CaptureReason::Show, None) if packet.segment_frame() == 0 => {}
                _ => {
                    return Err(RecordingError::Unsupported(
                        "capture requires an ordinary segment or explicit show",
                    ));
                }
            }
            let sample = self.camera_sample(camera)?;
            let bytes = self.encode_snapshot(packet.state());
            self.capture_snapshot(bytes, sample)
        })();
        self.retain_failure(result)
    }
}
