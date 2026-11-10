//! Concrete native WorkerService, with cold, hash-verified recovery.

mod pending;
mod recovery;

use fmn_render::{CameraConfig, RetainedFrameRendererConfig};
use fmn_scene::{
    AssetRead, CommandKind, CommandRecord, EffectClass, Entry, EventPayload, ImpureEffectTag,
    Journal,
};

use crate::protocol::{StudioInput, studio_input_command, studio_input_payload, studio_seek_frame};
use crate::{
    ProtocolDigest, ProtocolLimits, ResponseEnvelope, ServiceError, StudioDataKind,
    SupervisorRequest, WorkerErrorCode, WorkerResponse, WorkerService, protocol_digest,
};

use super::NativeSceneProgram;
use super::edits::{EditTrack, MAX_RECORDED_INPUTS, encode_state};
use super::program::{execution_error, invalid};
use super::raster::NativeRaster;

/// Whether a factory may be run again during supervisor recovery.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum NativeReplayPolicy {
    /// Arbitrary callbacks are opaque barriers; durable input and recovery refuse.
    #[default]
    Disabled,
    /// Every invocation creates independent captures and depends only on the
    /// content-hashed closure. No unjournaled I/O or shared mutable state may
    /// affect execution. Recovery verifies real execution, never decoded closures.
    ColdVerified,
}

/// Policy and content identities for a single registered native scene.
#[derive(Debug, Clone)]
pub struct NativeWorkerConfig {
    /// Stable scene name used by canonical Studio command identities.
    pub scene: String,
    /// Identity of the actual worker executable/build.
    pub build_id: ProtocolDigest,
    /// Digest covering scene code, assets and factory inputs (including seed).
    pub source_digest: ProtocolDigest,
    /// Number of addressable states, including constructed frame zero.
    pub frame_count: u64,
    /// Exact factory Scene frame rate.
    pub fps: u32,
    /// Existing retained CPU renderer policy.
    pub renderer: RetainedFrameRendererConfig,
    /// Optional base camera. A native rig supplies its animated pose/light.
    /// Camera editing still requires a camera-aware input projection.
    pub camera: Option<CameraConfig>,
    /// Canonical IPC/document limits; handshake can only reduce them.
    pub limits: ProtocolLimits,
    /// Frame-distance checkpoint cadence for committed seeks.
    pub checkpoint_frames: u64,
    /// Explicit factory replay contract; disabled by default.
    pub replay: NativeReplayPolicy,
    /// Maximum aggregate frame steps in one replay, or one cold seek.
    pub max_replay_frames: u64,
    /// Maximum viewport pixels, checked before allocating render buffers.
    pub max_pixels: u64,
    /// Maximum retained committed input events. Zero disables committed input;
    /// the hard ceiling is 65,536. Transient Event requests remain independent.
    pub max_recorded_inputs: usize,
    /// Maximum aggregate input dispatches in one replay/seek/commit request.
    /// This separately bounds zero-frame callbacks that a frame limit cannot.
    pub max_replay_inputs: u64,
    /// Stage successful preview Event requests for explicit Save. A committed
    /// seek to that same frame publishes the pending inputs as one journal
    /// batch. A noncommitting scrub discards them. Disabled by default; requires
    /// ColdVerified and affine input support. No new protocol variant is used.
    pub stage_input_events: bool,
}

impl NativeWorkerConfig {
    /// Configure a bounded CPU worker without assuming arbitrary callbacks replay.
    #[must_use]
    pub fn new(
        scene: impl Into<String>,
        build_id: ProtocolDigest,
        source_digest: ProtocolDigest,
        frame_count: u64,
        fps: u32,
        renderer: RetainedFrameRendererConfig,
    ) -> Self {
        Self {
            scene: scene.into(),
            build_id,
            source_digest,
            frame_count,
            fps,
            renderer,
            camera: None,
            limits: ProtocolLimits::default(),
            checkpoint_frames: 30,
            replay: NativeReplayPolicy::Disabled,
            max_replay_frames: 1_000_000,
            max_pixels: 16_777_216,
            max_recorded_inputs: 4096,
            max_replay_inputs: 65_536,
            stage_input_events: false,
        }
    }

    /// Base content identities shared by worker execution and host validation.
    /// This does not execute the scene factory. Camera capture/rig identities,
    /// when applicable, are additional reads bound by the worker at construction.
    #[must_use]
    pub fn base_input_reads(&self) -> Vec<AssetRead> {
        let mut contract = b"fmn-native-studio-v1\0".to_vec();
        contract.extend_from_slice(&self.fps.to_le_bytes());
        contract.extend_from_slice(&self.frame_count.to_le_bytes());
        contract.extend_from_slice(self.scene.as_bytes());
        vec![
            AssetRead {
                path: "native/source-closure".into(),
                digest: self.source_digest,
            },
            AssetRead {
                path: "native/worker-build".into(),
                digest: self.build_id,
            },
            AssetRead {
                path: "native/capture-contract".into(),
                digest: protocol_digest(&contract),
            },
            AssetRead {
                path: "native/input-track-semantics".into(),
                digest: protocol_digest(b"frame-stamped-native-input-track-v2"),
            },
        ]
    }
}

/// Native protocol service over one actual scene, editor and renderer.
///
/// Canonical Play seeks replay the committed input track. Canonical Play input
/// commands add to that track; `Event` remains a transient preview operation.
/// Clean forward scrubs execute only the remaining captures and input events.
/// Backward/dirty scrubs reconstruct from a fresh independent factory.
///
/// Committed input is opt-in through `ColdVerified`, frame-stamped and guarded
/// by the journal revision. Input in the past abandons the later input branch.
/// Every event uses the ordinary native editor/application dispatcher. Neither
/// closures nor editor history are deserialized: checkpoints include their
/// canonical input transcript and actual SceneState, then verify reconstruction.
/// Native callbacks still run only in the disposable worker, not the UI host.
pub struct NativeSceneWorker {
    config: NativeWorkerConfig,
    factory: Box<dyn Fn() -> Result<NativeSceneProgram, ServiceError>>,
    program: Option<NativeSceneProgram>,
    preview_clean: bool,
    raster: NativeRaster,
    camera_binding: Option<u64>,
    reads: Vec<AssetRead>,
    position: u64,
    last_hash: Option<ProtocolDigest>,
    last_checkpoint: Option<u64>,
    tail: Vec<u8>,
    edits: EditTrack,
    pending_inputs: Vec<CommandRecord>,
}

impl NativeSceneWorker {
    /// Construct a scene service without prerendering its movie.
    pub fn new(
        config: NativeWorkerConfig,
        factory: impl Fn() -> Result<NativeSceneProgram, ServiceError> + 'static,
    ) -> Result<Self, ServiceError> {
        crate::protocol::studio_seek_command(&config.scene, 0).map_err(execution_error)?;
        let viewport = config.renderer.frame.viewport;
        let pixels = u64::from(viewport.width) * u64::from(viewport.height);
        if config.frame_count == 0
            || config.frame_count > i64::MAX.cast_unsigned()
            || config.fps == 0
            || config.checkpoint_frames == 0
            || pixels == 0
            || pixels > config.max_pixels
            || config.max_recorded_inputs > MAX_RECORDED_INPUTS
        {
            return Err(invalid(
                "invalid native worker frame, clock, checkpoint, input or pixel budget",
            ));
        }
        if config
            .camera
            .as_ref()
            .is_some_and(|camera| camera.fps != config.fps)
        {
            return Err(invalid("native camera FPS must match the scene clock"));
        }
        let mut reads = config.base_input_reads();
        crate::InspectorView::new(
            0,
            config.frame_count,
            config.fps,
            viewport,
            config.renderer.frame.map,
            config.camera.is_none(),
        )
        .map_err(execution_error)?;
        let mut raster = NativeRaster::new(config.renderer, config.camera.clone())?;
        if config.camera.is_some() {
            reads.push(AssetRead {
                path: "native/camera-capture-policy".into(),
                digest: raster.backend()?.digest(),
            });
        }
        let program = factory()?;
        Self::validate_program(&config, &program)?;
        let camera_binding = program.camera_binding_index()?;
        raster.bind_camera(camera_binding)?;
        if camera_binding.is_some() {
            reads.push(AssetRead {
                path: "native/camera-rig-binding".into(),
                digest: raster.backend()?.digest(),
            });
        }
        Ok(Self {
            config,
            factory: Box::new(factory),
            program: Some(program),
            preview_clean: true,
            raster,
            camera_binding,
            reads,
            position: 0,
            last_hash: None,
            last_checkpoint: None,
            tail: Vec::new(),
            edits: EditTrack::default(),
            pending_inputs: Vec::new(),
        })
    }

    /// Current paused owner, absent after failed transient input/forward execution.
    #[must_use]
    pub fn program(&self) -> Option<&NativeSceneProgram> {
        self.program.as_ref()
    }

    /// Optimistic revision required by the next canonical committed input.
    #[must_use]
    pub const fn journal_position(&self) -> u64 {
        self.position
    }

    /// Number of events retained on the committed timeline branch.
    #[must_use]
    pub fn committed_input_count(&self) -> usize {
        self.edits.len()
    }

    /// Commit native input through exactly the same path as a canonical Play
    /// input command. It returns a JournalSegment for supervisor persistence,
    /// not an unjournaled Frame. Transient preview edits are not imported.
    pub fn commit_input(&mut self, input: StudioInput) -> Result<WorkerResponse, ServiceError> {
        let command = studio_input_command(&self.config.scene, &input)
            .map_err(|error| invalid(error.to_string()))?;
        self.record_command(command)
    }

    /// Successful preview inputs awaiting a same-frame committed seek (Save).
    #[must_use]
    pub fn pending_input_count(&self) -> usize {
        self.pending_inputs.len()
    }

    fn new_raster(&self) -> Result<NativeRaster, ServiceError> {
        let mut raster = NativeRaster::new(self.config.renderer, self.config.camera.clone())?;
        raster.bind_camera(self.camera_binding)?;
        Ok(raster)
    }

    fn validate_program(
        config: &NativeWorkerConfig,
        program: &NativeSceneProgram,
    ) -> Result<(), ServiceError> {
        if program.frame_index() != 0
            || program.frame_limit() != config.frame_count - 1
            || program.preview().scene().fps() != config.fps
        {
            return Err(invalid(
                "native factory disagrees with its initial-frame, length or fps contract",
            ));
        }
        if program.camera_rig().is_some() && config.camera.is_none() {
            return Err(invalid(
                "camera rig requires an explicit worker CameraConfig",
            ));
        }
        Ok(())
    }

    fn target(&self, frame: i64) -> Result<u64, ServiceError> {
        let frame = u64::try_from(frame).map_err(|_| invalid("negative native frame"))?;
        if frame >= self.config.frame_count || frame > self.config.max_replay_frames {
            return Err(invalid(
                "native target exceeds the scene or execution budget",
            ));
        }
        Ok(frame)
    }

    fn require_input_budget(&self, edits: &EditTrack, frame: u64) -> Result<(), ServiceError> {
        if edits.visible_count(frame) > self.config.max_replay_inputs {
            return Err(invalid(
                "native execution exceeds its input dispatch budget",
            ));
        }
        Ok(())
    }

    fn fresh_at(&self, frame: u64, edits: &EditTrack) -> Result<NativeSceneProgram, ServiceError> {
        self.require_input_budget(edits, frame)?;
        let mut program = (self.factory)()?;
        Self::validate_program(&self.config, &program)?;
        if program.camera_binding_index()? != self.camera_binding {
            return Err(invalid("native factory changed its camera rig binding"));
        }
        edits.advance(&mut program, frame, None)?;
        Ok(program)
    }

    fn state_bytes(
        &self,
        program: &mut NativeSceneProgram,
        edits: &EditTrack,
        position: u64,
    ) -> Result<Vec<u8>, ServiceError> {
        encode_state(
            &self.config.scene,
            position,
            &self.reads,
            edits,
            program.state_bytes()?,
            self.config.limits.max_checkpoint_bytes,
        )
    }

    fn checked(&self, response: WorkerResponse) -> Result<WorkerResponse, ServiceError> {
        let envelope = ResponseEnvelope {
            request_id: 1,
            response,
        };
        let _ = envelope
            .to_bytes(self.config.limits)
            .map_err(execution_error)?;
        Ok(envelope.response)
    }

    fn require_scene(&self, scene: &str) -> Result<(), ServiceError> {
        if scene != self.config.scene {
            return Err(ServiceError::new(
                WorkerErrorCode::SceneNotFound,
                "native scene is not registered",
            ));
        }
        Ok(())
    }

    fn effect(&self) -> EffectClass {
        match self.config.replay {
            NativeReplayPolicy::Disabled => EffectClass::Opaque,
            NativeReplayPolicy::ColdVerified => {
                EffectClass::Stateful(vec![ImpureEffectTag::UnclassifiedAnimation])
            }
        }
    }

    fn require_replay(&self, code: WorkerErrorCode) -> Result<(), ServiceError> {
        if self.config.replay == NativeReplayPolicy::Disabled {
            return Err(ServiceError::new(
                code,
                "native callback replay requires an explicit ColdVerified factory contract",
            ));
        }
        Ok(())
    }

    fn input_enabled(&self) -> bool {
        self.config.replay == NativeReplayPolicy::ColdVerified
            && self.config.camera.is_none()
            && self.config.max_recorded_inputs != 0
    }

    // Operates on a candidate track only; all revision/policy/size checks precede
    // factory execution and any mutation of the installed owner or journal.
    fn prepare_command(
        &self,
        command: &CommandRecord,
        position: u64,
        edits: &mut EditTrack,
    ) -> Result<u64, ServiceError> {
        let frame = if command.kind == CommandKind::Input {
            if !self.input_enabled() {
                return Err(invalid(
                    "native committed input is not enabled for this worker",
                ));
            }
            let input = studio_input_payload(&self.config.scene, command)
                .map_err(|error| invalid(error.to_string()))?;
            self.target(input.frame)?;
            edits.append(
                &self.config.scene,
                command,
                position,
                self.config.max_recorded_inputs,
            )?
        } else {
            self.target(
                studio_seek_frame(&self.config.scene, command)
                    .map_err(|error| invalid(error.to_string()))?,
            )?
        };
        self.require_input_budget(edits, frame)?;
        Ok(frame)
    }

    fn seek(&mut self, frame: i64) -> Result<WorkerResponse, ServiceError> {
        let frame = self.target(frame)?;
        self.require_input_budget(&self.edits, frame)?;
        if self.preview_clean
            && self
                .program
                .as_ref()
                .is_some_and(|program| program.frame_index() <= frame)
        {
            let mut program = self
                .program
                .take()
                .ok_or_else(|| invalid("native cursor disappeared"))?;
            let after = program.frame_index();
            self.edits.advance(&mut program, frame, Some(after))?;
            let response = self.raster.frame(&self.config.scene, &program)?;
            let response = self.checked(response)?;
            self.program = Some(program);
            self.pending_inputs.clear();
            return Ok(response);
        }
        let program = self.fresh_at(frame, &self.edits)?;
        let mut raster = self.new_raster()?;
        let response = raster.frame(&self.config.scene, &program)?;
        let response = self.checked(response)?;
        self.program = Some(program);
        self.pending_inputs.clear();
        self.preview_clean = true;
        self.raster = raster;
        Ok(response)
    }

    fn event(&mut self, event: EventPayload) -> Result<WorkerResponse, ServiceError> {
        event
            .validate()
            .map_err(|error| invalid(error.to_string()))?;
        if self.config.camera.is_some() {
            return Err(invalid(
                "native camera editing requires camera-aware input projection",
            ));
        }
        let command = self.staged_command(&event)?;
        let mut program = self
            .program
            .take()
            .ok_or_else(|| invalid("seek before sending input to a failed native preview"))?;
        self.preview_clean = false;
        let result = (|| {
            program.dispatch(event)?;
            let response = self.raster.frame(&self.config.scene, &program)?;
            self.checked(response)
        })();
        match result {
            Ok(response) => {
                self.program = Some(program);
                if let Some(command) = command {
                    self.pending_inputs.push(command);
                }
                Ok(response)
            }
            Err(error) => {
                self.pending_inputs.clear();
                Err(error)
            }
        }
    }

    fn record_command(&mut self, command: CommandRecord) -> Result<WorkerResponse, ServiceError> {
        if self.saves_pending(&command)? {
            return self.save_pending(command);
        }
        let next = self
            .position
            .checked_add(1)
            .ok_or_else(|| invalid("native journal position exhausted"))?;
        let mut edits = self.edits.try_clone()?;
        let frame = self.prepare_command(&command, self.position, &mut edits)?;
        let mut program = self.fresh_at(frame, &edits)?;
        let state = self.state_bytes(&mut program, &edits, next)?;
        let mut raster = self.new_raster()?;
        if command.kind == CommandKind::Input {
            // Do not record an input which cannot produce a valid native frame.
            let frame_response = raster.frame(&self.config.scene, &program)?;
            self.checked(frame_response)?;
        }
        let state_hash = protocol_digest(&state);
        let checkpoint = command.kind == CommandKind::Input
            || self
                .last_checkpoint
                .is_none_or(|last| last.abs_diff(frame) >= self.config.checkpoint_frames);
        let entry = Entry {
            command,
            effect: self.effect(),
            reads: self.reads.clone(),
            subprocesses: Vec::new(),
            checkpoint: checkpoint.then_some(state),
            state_hash,
        };
        let mut journal = Journal::new();
        journal.record(entry).map_err(execution_error)?;
        let journal = journal.to_bytes().map_err(execution_error)?;
        let response = self.checked(WorkerResponse::JournalSegment {
            scene: self.config.scene.clone(),
            start_entry: self.position,
            journal: journal.clone(),
        })?;
        self.program = Some(program);
        self.edits = edits;
        self.preview_clean = true;
        self.pending_inputs.clear();
        self.raster = raster;
        self.position = next;
        self.last_hash = Some(state_hash);
        self.tail = journal;
        if checkpoint {
            self.last_checkpoint = Some(frame);
        }
        Ok(response)
    }

    fn inspection(&self) -> Result<Vec<u8>, ServiceError> {
        let program = self
            .program
            .as_ref()
            .ok_or_else(|| invalid("native preview requires a successful seek"))?;
        let max = self.config.limits.max_studio_data_bytes;
        let suffix = format!(
            ",\"native_edit\":{{\"revision\":{},\"committed_inputs\":{},\"enabled\":{},\"preview_commit\":{},\"pending_inputs\":{}}}}}",
            self.position,
            self.edits.len(),
            self.input_enabled(),
            self.input_enabled() && self.config.stage_input_events,
            self.pending_inputs.len()
        );
        // The existing inspector remains the authority for the whole object tree.
        // Only additive, bounded native command metadata is appended here.
        let mut bytes = self.raster.inspect(
            program,
            self.config.frame_count,
            max,
            self.input_enabled().then_some(self.position),
        )?;
        if bytes.last() != Some(&b'}') {
            return Err(execution_error("native inspector omitted its root object"));
        }
        if bytes
            .len()
            .checked_add(suffix.len())
            .is_none_or(|length| length - 1 > max)
        {
            return Err(execution_error("native inspection exceeds its JSON budget"));
        }
        bytes.try_reserve(suffix.len()).map_err(execution_error)?;
        bytes.pop();
        bytes.extend_from_slice(suffix.as_bytes());
        Ok(bytes)
    }
}

fn refuse_owned(code: WorkerErrorCode, error: impl std::fmt::Display) -> ServiceError {
    ServiceError::new(code, error.to_string())
}

impl WorkerService for NativeSceneWorker {
    fn build_id(&self) -> ProtocolDigest {
        self.config.build_id
    }
    fn begin_session(
        &mut self,
        _supervisor_build: ProtocolDigest,
        max_frame_bytes: usize,
    ) -> Result<(), ServiceError> {
        if max_frame_bytes == 0 {
            return Err(invalid("zero native frame budget"));
        }
        self.config.limits.max_frame_bytes =
            self.config.limits.max_frame_bytes.min(max_frame_bytes);
        Ok(())
    }
    fn handle(&mut self, request: SupervisorRequest) -> Result<WorkerResponse, ServiceError> {
        match request {
            SupervisorRequest::EnumerateScenes => {
                self.checked(WorkerResponse::Scenes(vec![self.config.scene.clone()]))
            }
            SupervisorRequest::Play { scene, command } => {
                self.require_scene(&scene)?;
                self.record_command(command)
            }
            SupervisorRequest::Seek { scene, frame }
            | SupervisorRequest::Scrub { scene, frame } => {
                self.require_scene(&scene)?;
                self.seek(frame)
            }
            SupervisorRequest::Event { scene, event } => {
                self.require_scene(&scene)?;
                self.event(event)
            }
            SupervisorRequest::Inspect { scene } => {
                self.require_scene(&scene)?;
                let bytes = self.inspection()?;
                self.checked(WorkerResponse::StudioData {
                    scene,
                    kind: StudioDataKind::Inspection,
                    digest: protocol_digest(&bytes),
                    bytes,
                })
            }
            SupervisorRequest::Overlay { scene, layers } => {
                self.require_scene(&scene)?;
                let program = self
                    .program
                    .as_ref()
                    .ok_or_else(|| invalid("native preview requires a successful seek"))?;
                let bytes = self.raster.overlay(
                    program,
                    layers,
                    self.config.limits.max_studio_data_bytes,
                )?;
                self.checked(WorkerResponse::StudioData {
                    scene,
                    kind: StudioDataKind::Overlay,
                    digest: protocol_digest(&bytes),
                    bytes,
                })
            }
            SupervisorRequest::RestoreCheckpoint(checkpoint) => self.restore(checkpoint),
            SupervisorRequest::ReplayJournal(replay) => self.replay(replay),
            SupervisorRequest::Hello { .. } | SupervisorRequest::Shutdown => {
                Err(invalid("the protocol driver owns handshake and shutdown"))
            }
        }
    }
    fn active_scene(&self) -> Option<&str> {
        Some(&self.config.scene)
    }
    fn journal_tail(&self) -> &[u8] {
        &self.tail
    }
    fn last_state_hash(&self) -> Option<ProtocolDigest> {
        self.last_hash
    }
}
