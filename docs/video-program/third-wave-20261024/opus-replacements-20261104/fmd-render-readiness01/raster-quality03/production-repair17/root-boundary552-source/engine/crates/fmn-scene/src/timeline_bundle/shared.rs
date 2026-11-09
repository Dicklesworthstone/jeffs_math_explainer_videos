//! Thread-shareable FMTL input, preserving the ordinary reader and replay law.
//!
//! Snapshot callables and arena handles do not become Send. The validated
//! reader is consumed into canonical, immutable snapshot bytes. Workers bind
//! only the selected segment to their local arena and invoke the same
//! interpolation law as `TimelineBundle::stage_at`. One endpoint pair per
//! worker is cached, not a decoded copy of every frame of the whole bundle.

use std::sync::Arc;

use super::{
    BundleReadError, BundleSegmentKind, DEFAULT_MAX_BUNDLE_BYTES, SegmentData, TimelineBundle,
};
use fmn_anim::bundle::{bundle_sub_alpha, interpolate_between};
use fmn_anim::timeline::{Label, TimelinePlan};
use fmn_anim::{PathFunc, RateFunc};
use fmn_hash::SerialError;
use fmn_mobject::{Snapshot, Stage};

#[derive(Debug)]
enum Source {
    Pure {
        begin: Arc<[u8]>,
        end: Arc<[u8]>,
        path: PathFunc,
        // Already resolved and validated by the wire reader. Re-identifying
        // this callable by address can fail across codegen units (fm-34mh).
        rate: RateFunc,
    },
    Recorded(Arc<[u8]>),
}

#[derive(Debug)]
enum Segment {
    Pure(Arc<Source>),
    Recorded(Vec<Arc<Source>>),
}

/// A validated FMTL timeline whose frame inputs may cross worker boundaries.
///
/// Conversion never guesses purity: only the ordinary reader's proven kind-0
/// segments retain the pure reconstruction route. Kind-1 segments retain their
/// verbatim recorded snapshots, including the results of stateful callbacks.
/// This is replay of a compiled artifact, not speculative execution of arbitrary
/// live scene callbacks or a new RNG stream.
#[derive(Debug)]
pub struct SharedTimelineBundle {
    plan: TimelinePlan,
    segments: Vec<Segment>,
    frame_count: u32,
    engine_version: String,
    snapshot_bytes: usize,
    cameras: Option<Vec<fmn_render::camera::CameraSample>>,
}

fn freeze(snapshot: Snapshot, used: &mut usize) -> Result<Arc<[u8]>, BundleReadError> {
    let bytes = snapshot.to_bytes().map_err(BundleReadError::Malformed)?;
    let next = used
        .checked_add(bytes.len())
        .ok_or(BundleReadError::Malformed(SerialError::SizeLimit {
            limit: DEFAULT_MAX_BUNDLE_BYTES,
            needed: usize::MAX,
        }))?;
    if next > DEFAULT_MAX_BUNDLE_BYTES {
        return Err(BundleReadError::Malformed(SerialError::SizeLimit {
            limit: DEFAULT_MAX_BUNDLE_BYTES,
            needed: next,
        }));
    }
    *used = next;
    Ok(bytes.into())
}

impl TimelineBundle {
    /// Transfer this validated artifact into immutable worker inputs.
    ///
    /// The original decoded arenas are consumed and released as each segment
    /// is frozen. Canonical snapshot storage is cumulatively bounded by the
    /// same byte ceiling as the production bundle reader. No frame interpolation
    /// runs here, and pure segments store two endpoints, not one state per frame.
    ///
    /// # Errors
    /// Preserves canonical encoding/size/allocation failures. A legacy snapshot
    /// that grows when upgraded to the current schema can hit the byte ceiling
    /// even when its original input fitted; that refusal is explicit.
    pub fn into_shared(self) -> Result<SharedTimelineBundle, BundleReadError> {
        let mut segments = Vec::new();
        segments
            .try_reserve_exact(self.segments.len())
            .map_err(|_| BundleReadError::AllocationFailed {
                context: "shared segment table",
                requested: self.segments.len(),
            })?;
        let mut snapshot_bytes = 0;
        for segment in self.segments {
            let shared = match segment {
                SegmentData::Pure {
                    begin,
                    end,
                    path,
                    rate,
                } => Segment::Pure(Arc::new(Source::Pure {
                    begin: freeze(*begin, &mut snapshot_bytes)?,
                    end: freeze(*end, &mut snapshot_bytes)?,
                    path,
                    rate,
                })),
                SegmentData::Stateful { frames } => {
                    let mut shared = Vec::new();
                    shared.try_reserve_exact(frames.len()).map_err(|_| {
                        BundleReadError::AllocationFailed {
                            context: "shared frame table",
                            requested: frames.len(),
                        }
                    })?;
                    for frame in frames {
                        shared.push(Arc::new(Source::Recorded(freeze(
                            frame,
                            &mut snapshot_bytes,
                        )?)));
                    }
                    Segment::Recorded(shared)
                }
            };
            segments.push(shared);
        }
        Ok(SharedTimelineBundle {
            plan: self.plan,
            segments,
            frame_count: self.frame_count,
            engine_version: self.engine_version,
            snapshot_bytes,
            cameras: self.cameras,
        })
    }
}

impl SharedTimelineBundle {
    /// Validate through the one production reader, then freeze its inputs.
    ///
    /// # Errors
    /// Returns the reader's schema, engine, snapshot, or size refusal. No
    /// worker is started and no frame is rendered while loading.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, BundleReadError> {
        TimelineBundle::from_bytes(bytes)?.into_shared()
    }

    /// Total semantic frames on the validated clock.
    #[must_use]
    pub const fn frame_count(&self) -> u32 {
        self.frame_count
    }
    /// Frame rate of the compiled artifact, not a resampling request.
    #[must_use]
    pub fn fps(&self) -> u32 {
        self.plan.fps()
    }
    /// Authored play/wait segment count, including zero-frame segments.
    #[must_use]
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }
    /// The certified renderer and reconstruction-law identity of this input.
    #[must_use]
    pub fn engine_version(&self) -> &str {
        &self.engine_version
    }
    /// Exact bytes held by the immutable canonical snapshot payloads.
    #[must_use]
    pub const fn snapshot_bytes(&self) -> usize {
        self.snapshot_bytes
    }
    /// Original labels, in authored order.
    #[must_use]
    pub fn labels(&self) -> &[Label] {
        self.plan.labels()
    }
    /// Kind chosen by the export-time proof, never reclassified by a worker.
    #[must_use]
    pub fn segment_kind(&self, index: usize) -> Option<BundleSegmentKind> {
        self.segments.get(index).map(|segment| match segment {
            Segment::Pure(_) => BundleSegmentKind::Pure,
            Segment::Recorded(_) => BundleSegmentKind::Stateful,
        })
    }

    /// Select an immutable frame input without reconstructing its Stage.
    ///
    /// Only the selected endpoint pair or recorded frame is retained by the
    /// job; dropping this bundle does not leave every unselected segment alive.
    /// Jobs may execute in any order, and independently of the reader's lifetime.
    ///
    /// # Errors
    /// Refuses an index outside the authoritative 0-based frame range.
    pub fn frame_job(&self, index: u32) -> Result<TimelineFrameJob, BundleReadError> {
        let global = i64::from(index) + 1;
        let (segment, offset) =
            self.plan
                .locate(global)
                .ok_or(BundleReadError::FrameOutOfRange {
                    index,
                    total: self.frame_count,
                })?;
        let planned = &self.plan.segments()[segment];
        let source = match &self.segments[segment] {
            Segment::Pure(source) => Arc::clone(source),
            Segment::Recorded(frames) => Arc::clone(
                frames
                    .get(usize::try_from(offset - 1).unwrap_or(usize::MAX))
                    .ok_or(BundleReadError::PlanInconsistent(
                        "shared frame table is shorter than its plan",
                    ))?,
            ),
        };
        Ok(TimelineFrameJob {
            source,
            index,
            fps: self.fps(),
            offset,
            run_time: planned.run_time,
            camera: self.cameras.as_ref().map(|cameras| cameras[index as usize]),
        })
    }
}

/// Thread-shareable input for one compiled frame. Its reconstructed Stage is
/// deliberately local to the worker, and must be compiled/rendered there.
#[derive(Clone, Debug)]
pub struct TimelineFrameJob {
    source: Arc<Source>,
    index: u32,
    fps: u32,
    offset: i64,
    run_time: f64,
    camera: Option<fmn_render::camera::CameraSample>,
}

impl TimelineFrameJob {
    /// Original 0-based frame number (distinct from a caller's output sequence).
    #[must_use]
    pub const fn index(&self) -> u32 {
        self.index
    }
    /// Whether this frame must be rendered with its recorded camera.
    #[must_use]
    pub const fn has_camera_track(&self) -> bool {
        self.camera.is_some()
    }

    /// Reconstruct without a cache, retaining required camera semantics.
    ///
    /// # Errors
    /// Preserves snapshot and camera/viewport validation failures.
    pub fn materialize_with_camera(
        &self,
        resolution: (u32, u32),
    ) -> Result<(Stage, Option<fmn_render::Camera>), BundleReadError> {
        TimelineFrameCache::default().materialize_with_camera(self, resolution)
    }

    /// Kind chosen by the bundle exporter, including conservative demotions.
    #[must_use]
    pub fn kind(&self) -> BundleSegmentKind {
        match self.source.as_ref() {
            Source::Pure { .. } => BundleSegmentKind::Pure,
            Source::Recorded(_) => BundleSegmentKind::Stateful,
        }
    }
    /// Reconstruct without retaining a worker cache.
    ///
    /// # Errors
    /// Preserves the canonical snapshot decoder's failure. Inputs are already
    /// validated; a later decode failure is not permission to omit the frame.
    pub fn materialize(&self) -> Result<Stage, BundleReadError> {
        TimelineFrameCache::default().materialize(self)
    }
}

struct DecodedPure {
    key: Arc<Source>,
    begin: Snapshot,
    end: Snapshot,
    path: PathFunc,
    rate: RateFunc,
}

/// Worker-local endpoint cache. It contains ordinary non-Send snapshots and
/// must be created, used and dropped on that worker. At most one pure segment
/// is retained. Switching to recorded frames releases its decoded endpoints.
/// Cache identity affects work only, never the reconstruction law or pixels.
#[derive(Default)]
pub struct TimelineFrameCache {
    pure: Option<DecodedPure>,
    decoded_snapshots: u64,
}

impl TimelineFrameCache {
    /// Number of snapshots decoded by this cache; useful for proving endpoint
    /// reuse without substituting a timing assertion for correctness.
    #[must_use]
    pub const fn decoded_snapshots(&self) -> u64 {
        self.decoded_snapshots
    }

    /// Materialize a job using the same alpha, path and clock law as the
    /// ordinary FMTL reader. No live callbacks or RNG are invoked by replay.
    ///
    /// # Errors
    /// Preserves canonical decode errors without evicting a valid cached pair
    /// in favor of a partially decoded replacement.
    pub fn materialize(&mut self, job: &TimelineFrameJob) -> Result<Stage, BundleReadError> {
        if job.has_camera_track() {
            return Err(BundleReadError::CameraTrackRequired);
        }
        self.materialize_scene(job)
    }

    /// Reconstruct geometry and its recorded camera as one frame input.
    ///
    /// # Errors
    /// Refuses invalid viewports/cameras before decoding or replacing a cache.
    pub fn materialize_with_camera(
        &mut self,
        job: &TimelineFrameJob,
        resolution: (u32, u32),
    ) -> Result<(Stage, Option<fmn_render::Camera>), BundleReadError> {
        let camera = super::camera::restore(job.camera.as_ref(), resolution, job.fps, job.index)?;
        Ok((self.materialize_scene(job)?, camera))
    }

    fn materialize_scene(&mut self, job: &TimelineFrameJob) -> Result<Stage, BundleReadError> {
        let mut stage = match job.source.as_ref() {
            Source::Pure {
                begin,
                end,
                path,
                rate,
            } => {
                if self
                    .pure
                    .as_ref()
                    .is_none_or(|cached| !Arc::ptr_eq(&cached.key, &job.source))
                {
                    let binding = Stage::new();
                    let begin = Snapshot::from_bytes(begin, &binding)
                        .map_err(BundleReadError::Snapshot)?
                        .snapshot;
                    let end = Snapshot::from_bytes(end, &binding)
                        .map_err(BundleReadError::Snapshot)?
                        .snapshot;
                    self.pure = Some(DecodedPure {
                        key: Arc::clone(&job.source),
                        begin,
                        end,
                        path: *path,
                        rate: rate.clone(),
                    });
                    self.decoded_snapshots = self.decoded_snapshots.saturating_add(2);
                }
                let cached = self.pure.as_ref().ok_or(BundleReadError::PlanInconsistent(
                    "missing decoded endpoints",
                ))?;
                // Keep the same grouping as TimelineBundle::stage_at: binary64
                // duration boundaries and rate clamping are semantic, not hints.
                let alpha = (job.offset as f64 / f64::from(job.fps)) / job.run_time;
                interpolate_between(
                    &cached.begin,
                    &cached.end,
                    bundle_sub_alpha(alpha, &cached.rate),
                    cached.path,
                )
            }
            Source::Recorded(bytes) => {
                self.pure = None;
                let binding = Stage::new();
                let snapshot = Snapshot::from_bytes(bytes, &binding)
                    .map_err(BundleReadError::Snapshot)?
                    .snapshot;
                self.decoded_snapshots = self.decoded_snapshots.saturating_add(1);
                snapshot.materialize()
            }
        };
        stage.set_time_from_clock((i64::from(job.index) + 1) as f64 / f64::from(job.fps));
        Ok(stage)
    }
}
