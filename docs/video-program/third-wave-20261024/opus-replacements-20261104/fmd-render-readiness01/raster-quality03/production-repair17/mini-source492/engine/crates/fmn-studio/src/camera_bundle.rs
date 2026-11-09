//! On-demand Studio playback of immutable FMTL artifacts through a camera.
//!
//! The bundle reader owns reconstruction and Lumen owns rendering. A committed
//! command selects a recorded frame; it never re-executes authored callbacks or
//! reclassifies their effects. The ordinary journal binds the entire artifact
//! and viewport/backend input. Cold recovery replays those cheap seek commands,
//! not SceneState checkpoints containing an unrecorded camera or Python state.

use fmn_core::rng::RngRoot;
use fmn_frame::convert::rgba16f_to_rgba8;
use fmn_frame::{FrameBuffer, FrameLayout, PixelFormat};
use fmn_hash::{Digest, Schema, Writer, sha256};
use fmn_render::{Camera, RetainedFrameRenderer, RetainedFrameRendererConfig, ScreenMap};
use fmn_scene::studio_bridge::{SceneState, Stage};
use fmn_scene::timeline_bundle::{SharedTimelineBundle, TimelineFrameCache};
use fmn_scene::{
    AssetRead, CommandRecord, EffectClass, Entry, Journal, RenderBackendRecord, RenderBackendRole,
    TimelineBundle,
};

use crate::native::camera_capture_backend;
use crate::protocol::{studio_seek_command, studio_seek_frame};
use crate::{
    FrameEncoding, FramePayload, FrameStream, InspectorLimits, InspectorSnapshot, InspectorView,
    JournalReplay, ProtocolLimits, ServiceError, SpanRegistry, StudioDataKind, SupervisorRequest,
    WorkerErrorCode, WorkerResponse, WorkerService,
};

fn failed(error: impl std::fmt::Display) -> ServiceError {
    ServiceError::new(WorkerErrorCode::ExecutionFailed, error.to_string())
}
fn invalid(message: &str) -> ServiceError {
    ServiceError::new(WorkerErrorCode::InvalidRequest, message)
}
fn replay_failed(message: &str) -> ServiceError {
    ServiceError::new(WorkerErrorCode::ReplayFailed, message)
}

/// The explicitly selected view of a compiled Studio timeline.
///
/// Camera-bearing artifacts own every camera sample and require `fixed_camera`
/// to be `None`. Older artifacts containing surfaces, images or point clouds
/// require an explicit fixed camera instead. Its complete pose, lighting and
/// background enter the replay input identity; no default camera is inferred
/// from arbitrary tracker records or substituted for a captured camera.
#[derive(Clone, Debug)]
pub struct CameraBundleView {
    /// Actual camera renderer, including its viewport and certified CPU kernel.
    pub renderer: RetainedFrameRendererConfig,
    /// Deterministic scene-state seed used by the seek journal.
    pub seed: u64,
    /// Caller-selected camera for an artifact without a camera track.
    pub fixed_camera: Option<Camera>,
}

/// A code-free camera timeline hosted by the existing isolated worker protocol.
///
/// Only the immutable bundle, one decoded endpoint cache, and one retained
/// renderer are owned. No PNG history or materialized per-frame Stage is kept.
/// Transient preview never changes the committed journal; failed rendering or
/// replay leaves the last usable position and journal cursor intact.
pub struct CameraBundleWorker {
    build_id: Digest,
    scene: String,
    bundle: SharedTimelineBundle,
    cache: TimelineFrameCache,
    renderer: RetainedFrameRenderer,
    rgba: FrameBuffer,
    reads: Vec<AssetRead>,
    backend: RenderBackendRecord,
    fixed_camera: Option<Camera>,
    seed: u64,
    position: u32,
    journal_position: u64,
    journal_tail: Vec<u8>,
    last_state_hash: Option<Digest>,
    limits: ProtocolLimits,
}

impl CameraBundleWorker {
    /// Compute the input records independently in the supervisor and worker.
    /// The caller supplies the digest/path of the bytes used by the production
    /// TimelineBundle reader; it must not read a different generation here.
    ///
    /// The artifact binds geometry and every recorded camera sample, including
    /// views whose geometry happens to be identical. The additional native
    /// record binds an explicit fixed camera when supplied, the viewport, actual
    /// CPU kernel, build and deterministic state seed.
    pub fn input_reads(
        build_id: Digest,
        source: &AssetRead,
        bundle: &TimelineBundle,
        view: &CameraBundleView,
    ) -> Result<Vec<AssetRead>, ServiceError> {
        if bundle.frame_count() == 0 {
            return Err(invalid(
                "Studio camera playback requires a nonempty FMTL bundle",
            ));
        }
        if source.path.is_empty()
            || source.path.len() > 4096
            || source.path.contains('\0')
            || source.path == "native/camera-bundle-view-v1"
        {
            return Err(invalid("invalid compiled camera source identity"));
        }
        if view.renderer.frame.aa != fmn_core::AaPolicy::Adaptive {
            return Err(invalid(
                "camera-bundle playback requires adaptive camera coverage",
            ));
        }
        let viewport = view.renderer.frame.viewport;
        if u64::from(viewport.width) * u64::from(viewport.height) > 16_777_216 {
            return Err(invalid("Studio camera viewport exceeds 16M pixels"));
        }
        let backend = Self::capture_backend(source, bundle, view)?;
        let mut identity = Writer::new(Schema::new(*b"FMCB", 1, 1, 0));
        identity
            .put_bytes(build_id.as_bytes())
            .put_u64(view.seed)
            .put_bytes(backend.identity());
        Ok(vec![
            source.clone(),
            AssetRead {
                path: "native/camera-bundle-view-v1".to_owned(),
                digest: sha256(&identity.finish().map_err(failed)?),
            },
        ])
    }

    fn capture_backend(
        source: &AssetRead,
        bundle: &TimelineBundle,
        view: &CameraBundleView,
    ) -> Result<RenderBackendRecord, ServiceError> {
        let viewport = view.renderer.frame.viewport;
        let camera = if bundle.has_camera_track() {
            if view.fixed_camera.is_some() {
                return Err(invalid(
                    "a fixed camera cannot override a recorded FMTL camera track",
                ));
            }
            bundle
                .camera_at(0, (viewport.width, viewport.height))
                .map_err(failed)?
                .ok_or_else(|| invalid("missing recorded camera"))?
        } else {
            let camera = view.fixed_camera.clone().ok_or_else(|| {
                invalid("FMTL without a camera track requires an explicit fixed camera")
            })?;
            if camera.fps() != bundle.fps() {
                return Err(invalid(
                    "fixed camera frame rate must match the compiled timeline",
                ));
            }
            camera
        };
        let policy = camera_capture_backend(&camera, view.renderer)?;
        // One immutable view identity, not a growing backend catalog per pose.
        // The source digest binds the geometry and any recorded camera track;
        // the nested descriptor also binds the explicit fixed camera, when
        // supplied, and names the actual kernel and output viewport.
        let mut identity = Writer::new(Schema::new(*b"FMCB", 2, 1, 0));
        identity
            .put_str(if view.fixed_camera.is_some() {
                "lumen-fixed-camera-bundle-v1"
            } else {
                "lumen-recorded-camera-bundle-v1"
            })
            .put_bytes(source.digest.as_bytes())
            .put_bytes(policy.identity());
        RenderBackendRecord::new(
            RenderBackendRole::FrameStream,
            identity.finish().map_err(failed)?,
        )
        .map_err(failed)
    }

    /// Create a worker from an already bounded, validated artifact generation.
    /// Camera rendering must be admitted as the real certified-CPU kernel by
    /// the composition root, even when the source's determinism mode is standard.
    /// No accelerator/affine fallback or recorded-camera override is substituted.
    pub fn new(
        scene: String,
        build_id: Digest,
        source: AssetRead,
        bundle: TimelineBundle,
        view: CameraBundleView,
    ) -> Result<Self, ServiceError> {
        studio_seek_command(&scene, 0).map_err(failed)?;
        let reads = Self::input_reads(build_id, &source, &bundle, &view)?;
        let backend = Self::capture_backend(&source, &bundle, &view)?;
        let layout = FrameLayout::tight(
            PixelFormat::Rgba8,
            view.renderer.frame.viewport.width,
            view.renderer.frame.viewport.height,
        )
        .map_err(failed)?;
        Ok(Self {
            build_id,
            scene,
            reads,
            backend,
            seed: view.seed,
            fixed_camera: view.fixed_camera,
            bundle: bundle.into_shared().map_err(failed)?,
            cache: TimelineFrameCache::default(),
            renderer: RetainedFrameRenderer::new(view.renderer).map_err(failed)?,
            rgba: FrameBuffer::new(layout),
            position: 0,
            journal_position: 0,
            journal_tail: Vec::new(),
            last_state_hash: None,
            limits: ProtocolLimits::default(),
        })
    }

    fn require_scene(&self, scene: &str) -> Result<(), ServiceError> {
        if scene != self.scene {
            return Err(ServiceError::new(
                WorkerErrorCode::SceneNotFound,
                "scene is not registered in this camera-bundle worker",
            ));
        }
        Ok(())
    }

    fn frame_index(&self, frame: i64) -> Result<u32, ServiceError> {
        u32::try_from(frame)
            .ok()
            .filter(|index| *index < self.bundle.frame_count())
            .ok_or_else(|| invalid("camera-bundle frame is outside the recorded timeline"))
    }

    fn materialize(&mut self, frame: u32) -> Result<(Stage, Camera), ServiceError> {
        let viewport = self.renderer.config().frame.viewport;
        let job = self.bundle.frame_job(frame).map_err(failed)?;
        let (stage, camera) = self
            .cache
            .materialize_with_camera(&job, (viewport.width, viewport.height))
            .map_err(failed)?;
        Ok((
            stage,
            camera
                .or_else(|| self.fixed_camera.clone())
                .ok_or_else(|| invalid("compiled camera view disappeared"))?,
        ))
    }

    fn render_frame(&mut self, frame: u32) -> Result<WorkerResponse, ServiceError> {
        let (stage, camera) = self.materialize(frame)?;
        self.renderer
            .render_with_camera(&stage, &camera)
            .map_err(failed)?;
        rgba16f_to_rgba8(self.renderer.frame(), &mut self.rgba).map_err(failed)?;
        let viewport = self.renderer.config().frame.viewport;
        let bytes = fmn_codec::encode_rgba8(
            viewport.width,
            viewport.height,
            self.rgba.as_bytes(),
            fmn_codec::CompressionLevel::Fast,
        );
        let stream = FrameStream {
            scene: self.scene.clone(),
            frame_index: u64::from(frame),
            width: viewport.width,
            height: viewport.height,
            stride: 0,
            encoding: FrameEncoding::Png,
            payload: FramePayload::Pipe {
                digest: sha256(&bytes),
                bytes,
            },
            render_backends: vec![self.backend.try_clone().map_err(failed)?],
        };
        stream.validate(self.limits).map_err(failed)?;
        Ok(WorkerResponse::Frame(stream))
    }

    fn inspect(&mut self) -> Result<WorkerResponse, ServiceError> {
        let (stage, camera) = self.materialize(self.position)?;
        let limits = InspectorLimits {
            max_json_bytes: self.limits.max_studio_data_bytes,
            ..InspectorLimits::default()
        };
        let mut snapshot =
            InspectorSnapshot::capture(&stage, &SpanRegistry::new(), limits).map_err(failed)?;
        snapshot.view = Some(
            InspectorView::new(
                u64::from(self.position),
                u64::from(self.bundle.frame_count()),
                self.bundle.fps(),
                self.renderer.config().frame.viewport,
                ScreenMap {
                    scale: 1.0 / camera.pixel_size(),
                    origin: [
                        f64::from(camera.pixel_width()) * 0.5,
                        f64::from(camera.pixel_height()) * 0.5,
                    ],
                    y_up: true,
                },
                false,
            )
            .map_err(failed)?,
        );
        let bytes = snapshot.to_json(limits).map_err(failed)?;
        Ok(WorkerResponse::StudioData {
            scene: self.scene.clone(),
            kind: StudioDataKind::Inspection,
            digest: sha256(&bytes),
            bytes,
        })
    }

    fn state_hash(&mut self, frame: u32, commands: u64) -> Result<Digest, ServiceError> {
        let (stage, _) = self.materialize(frame)?;
        let clock_frame = i64::from(frame) + 1;
        let rng = RngRoot::from_seed(self.seed)
            .substream("scene")
            .fork_frame(clock_frame.cast_unsigned());
        let bytes = SceneState::capture(&stage, clock_frame, self.bundle.fps(), commands, &rng)
            .to_bytes()
            .map_err(failed)?;
        Ok(sha256(&bytes))
    }

    fn entry_bytes(&self, entry: Entry) -> Result<Vec<u8>, ServiceError> {
        let mut journal = Journal::new();
        journal.record(entry).map_err(failed)?;
        let bytes = journal.to_bytes().map_err(failed)?;
        if bytes.len() > self.limits.max_journal_bytes {
            return Err(invalid("camera seek journal exceeds protocol budget"));
        }
        Ok(bytes)
    }

    fn record_seek(&mut self, command: CommandRecord) -> Result<WorkerResponse, ServiceError> {
        let frame = self.frame_index(studio_seek_frame(&self.scene, &command).map_err(failed)?)?;
        let next = self
            .journal_position
            .checked_add(1)
            .ok_or_else(|| invalid("Studio journal cursor exhausted"))?;
        let state_hash = self.state_hash(frame, next)?;
        let journal = self.entry_bytes(Entry {
            command,
            effect: EffectClass::Pure,
            reads: self.reads.clone(),
            subprocesses: Vec::new(),
            checkpoint: None,
            state_hash,
        })?;
        let response = WorkerResponse::JournalSegment {
            scene: self.scene.clone(),
            start_entry: self.journal_position,
            journal: journal.clone(),
        };
        self.position = frame;
        self.journal_position = next;
        self.last_state_hash = Some(state_hash);
        self.journal_tail = journal;
        Ok(response)
    }

    fn replay(&mut self, replay: JournalReplay) -> Result<WorkerResponse, ServiceError> {
        self.require_scene(&replay.scene)?;
        if replay.from_entry != self.journal_position
            || replay.through_entry < replay.from_entry
            || replay.journal.len() > self.limits.max_journal_bytes
            || replay.through_entry - replay.from_entry > self.limits.max_replay_hashes as u64
        {
            return Err(replay_failed(
                "invalid camera-bundle replay cursor or budget",
            ));
        }
        let journal = Journal::from_bytes(&replay.journal).map_err(failed)?;
        let from = usize::try_from(replay.from_entry).map_err(failed)?;
        let through = usize::try_from(replay.through_entry).map_err(failed)?;
        let entries = journal
            .entries()
            .get(from..through)
            .ok_or_else(|| replay_failed("replay range exceeds camera seek journal"))?;
        let mut state_hashes = Vec::new();
        state_hashes
            .try_reserve_exact(entries.len())
            .map_err(failed)?;
        let mut position = self.position;
        for (offset, entry) in entries.iter().enumerate() {
            if entry.reads != self.reads
                || entry.effect != EffectClass::Pure
                || !entry.subprocesses.is_empty()
                || entry.checkpoint.is_some()
            {
                return Err(replay_failed(
                    "camera seek journal has different inputs or unsupported effects",
                ));
            }
            position =
                self.frame_index(studio_seek_frame(&self.scene, &entry.command).map_err(failed)?)?;
            let hash = self.state_hash(position, replay.from_entry + offset as u64 + 1)?;
            if hash != entry.state_hash {
                return Err(replay_failed("camera seek journal state diverged"));
            }
            state_hashes.push(hash);
        }
        // Prepare the bounded crash tail before publishing any replay progress.
        let tail = entries
            .last()
            .map(|entry| self.entry_bytes(entry.try_clone().map_err(failed)?))
            .transpose()?;
        self.position = position;
        self.journal_position = replay.through_entry;
        if let Some(hash) = state_hashes.last() {
            self.last_state_hash = Some(*hash);
        }
        if let Some(tail) = tail {
            self.journal_tail = tail;
        }
        Ok(WorkerResponse::ReplayComplete {
            from_entry: replay.from_entry,
            state_hashes,
        })
    }
}

impl WorkerService for CameraBundleWorker {
    fn build_id(&self) -> Digest {
        self.build_id
    }

    fn begin_session(
        &mut self,
        _supervisor: Digest,
        max_frame_bytes: usize,
    ) -> Result<(), ServiceError> {
        if max_frame_bytes == 0 || max_frame_bytes > ProtocolLimits::default().max_frame_bytes {
            return Err(invalid("invalid negotiated camera frame budget"));
        }
        self.limits.max_frame_bytes = max_frame_bytes;
        Ok(())
    }

    fn handle(&mut self, request: SupervisorRequest) -> Result<WorkerResponse, ServiceError> {
        match request {
            SupervisorRequest::EnumerateScenes => {
                Ok(WorkerResponse::Scenes(vec![self.scene.clone()]))
            }
            SupervisorRequest::Seek { scene, frame }
            | SupervisorRequest::Scrub { scene, frame } => {
                self.require_scene(&scene)?;
                let frame = self.frame_index(frame)?;
                let response = self.render_frame(frame)?;
                self.position = frame;
                Ok(response)
            }
            SupervisorRequest::Play { scene, command } => {
                self.require_scene(&scene)?;
                self.record_seek(command)
            }
            SupervisorRequest::Inspect { scene } => {
                self.require_scene(&scene)?;
                self.inspect()
            }
            SupervisorRequest::ReplayJournal(replay) => self.replay(replay),
            SupervisorRequest::RestoreCheckpoint(checkpoint) => {
                self.require_scene(&checkpoint.scene)?;
                Err(ServiceError::new(
                    WorkerErrorCode::CheckpointRejected,
                    "camera bundles recover from their input-verified seek journal, not live-scene checkpoints",
                ))
            }
            SupervisorRequest::Event { scene, .. } | SupervisorRequest::Overlay { scene, .. } => {
                self.require_scene(&scene)?;
                Err(invalid(
                    "compiled camera playback has no live input or projected overlay adapter",
                ))
            }
            SupervisorRequest::Hello { .. } | SupervisorRequest::Shutdown => {
                Err(invalid("the worker protocol owns handshake and shutdown"))
            }
        }
    }
    fn active_scene(&self) -> Option<&str> {
        Some(&self.scene)
    }
    fn journal_tail(&self) -> &[u8] {
        &self.journal_tail
    }
    fn last_state_hash(&self) -> Option<Digest> {
        self.last_state_hash
    }
}
