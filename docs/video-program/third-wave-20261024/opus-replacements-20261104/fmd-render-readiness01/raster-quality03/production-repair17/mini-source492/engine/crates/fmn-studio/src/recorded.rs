//! Bounded, read-only Studio timelines captured by an imperative front door.
//!
//! This is the inspection/playback half of Studio, not a callback serializer.
//! The authoring worker executes once and supplies real rendered PNGs and
//! native inspector snapshots at the normal capture boundary. Scrubbing never
//! executes authored code. A source reload replaces the process and captures
//! again; every committed position is an opaque journal barrier, so neither a
//! checkpoint nor an old Python closure can be mistaken for replay authority.

use std::sync::Arc;
use std::time::Duration;

mod advance;
mod live;

use fmn_codec::{PngLimits, decode_png};
use fmn_hash::{Digest, Schema, Writer, sha256};
use fmn_scene::{AssetRead, EffectClass, Entry, Journal};

use crate::protocol::studio_seek_frame;
use crate::{
    CapabilityToken, FrameEncoding, FrameHub, FramePayload, FrameStream, HostError,
    InspectorLimits, InspectorSnapshot, ProtocolLimits, RebuildDriver, ServiceError,
    StdWorkerLauncher, StudioDataKind, StudioHost, StudioHostConfig, StudioWorkerSession,
    Supervisor, SupervisorConfig, SupervisorReply, SupervisorRequest, WorkerErrorCode,
    WorkerResponse, WorkerService,
};

fn failed(error: impl std::fmt::Display) -> ServiceError {
    ServiceError::new(WorkerErrorCode::ExecutionFailed, error.to_string())
}
fn invalid(message: &str) -> ServiceError {
    ServiceError::new(WorkerErrorCode::InvalidRequest, message)
}

struct RecordedFrame {
    stream: FrameStream,
    inspection: InspectorSnapshot,
    state_hash: Digest,
    encoded_bytes: usize,
}

/// One bounded capture generation. The byte ceiling counts encoded PNG and
/// inspector documents; inspector traversal/value ceilings independently bound
/// their structured in-memory representation. No live Stage/proxy is retained.
pub struct RecordedTimeline {
    scene: String,
    build_id: Digest,
    source_digest: Digest,
    frames: Vec<RecordedFrame>,
    position: usize,
    encoded_bytes: usize,
    max_frames: usize,
    max_bytes: usize,
    commands: u64,
    input_revision: Option<u64>,
    advance_enabled: bool,
    limits: ProtocolLimits,
}

impl RecordedTimeline {
    /// Construct an empty, resource-bounded timeline. The hard limits prevent
    /// a caller from disabling accounting by passing an effectively infinite cap.
    pub fn new(
        scene: String,
        build_id: Digest,
        source_digest: Digest,
        max_frames: usize,
        max_bytes: usize,
    ) -> Result<Self, ServiceError> {
        let limits = ProtocolLimits::default();
        if scene.is_empty() || scene.len() > 4096 || scene.contains('\0') {
            return Err(invalid("invalid recorded scene name"));
        }
        if !(1..=100_000).contains(&max_frames) || !(1..=1024 * 1024 * 1024).contains(&max_bytes) {
            return Err(invalid(
                "recorded timeline requires 1..100000 frames and at most 1 GiB",
            ));
        }
        Ok(Self {
            scene,
            build_id,
            source_digest,
            frames: Vec::new(),
            position: 0,
            encoded_bytes: 0,
            max_frames,
            max_bytes,
            commands: 0,
            input_revision: None,
            advance_enabled: false,
            limits,
        })
    }

    /// Admit one real capture atomically. A refused capture leaves the previous
    /// timeline untouched. Ordinals are presentation indices, not scene times;
    /// the exact captured time remains in the native inspector snapshot.
    pub fn push(
        &mut self,
        stream: FrameStream,
        inspection: InspectorSnapshot,
    ) -> Result<(), ServiceError> {
        if self.frames.len() == self.max_frames {
            return Err(invalid("recorded timeline frame budget exceeded"));
        }
        if self.input_revision.is_some() {
            return Err(invalid(
                "live capture must replace its final frame, not extend history",
            ));
        }
        let frame = self.admit_frame(
            stream,
            inspection,
            self.frames.len() as u64,
            self.encoded_bytes,
        )?;
        self.frames.try_reserve(1).map_err(failed)?;
        self.encoded_bytes += frame.encoded_bytes;
        self.frames.push(frame);
        Ok(())
    }

    fn admit_frame(
        &self,
        mut stream: FrameStream,
        mut inspection: InspectorSnapshot,
        index: u64,
        occupied_bytes: usize,
    ) -> Result<RecordedFrame, ServiceError> {
        if stream.scene != self.scene || stream.encoding != FrameEncoding::Png {
            return Err(invalid(
                "recorded timeline requires same-scene PNG captures",
            ));
        }
        stream.validate(self.limits).map_err(failed)?;
        let FramePayload::Pipe { bytes, .. } = &stream.payload else {
            return Err(invalid(
                "recorded timelines cannot retain shared-memory tokens",
            ));
        };
        if bytes.len() > self.max_bytes.saturating_sub(occupied_bytes) {
            return Err(invalid("recorded timeline byte budget exceeded"));
        }
        let png = decode_png(
            bytes,
            &PngLimits {
                max_pixels: 16_777_216,
                ..PngLimits::default()
            },
        )
        .map_err(failed)?;
        if png.width != stream.width || png.height != stream.height {
            return Err(invalid("recorded PNG dimensions do not match the frame"));
        }
        let view = inspection
            .view
            .as_mut()
            .ok_or_else(|| invalid("capture has no native inspector view"))?;
        if view.width != stream.width || view.height != stream.height || view.input_events {
            return Err(invalid(
                "recorded inspector geometry or input policy disagrees with capture",
            ));
        }
        if let Some(first) = self.frames.first() {
            let previous = first
                .inspection
                .view
                .as_ref()
                .expect("admitted capture has a view");
            if (view.width, view.height, view.fps)
                != (previous.width, previous.height, previous.fps)
            {
                return Err(invalid("recorded timeline cannot change resolution or FPS"));
            }
        }
        view.frame_index = index;
        view.frame_count = index + 1;
        view.input_revision = None;
        stream.frame_index = index;
        let document = inspection
            .to_json(InspectorLimits::default())
            .map_err(failed)?;
        // Include bounded protocol metadata and space for the final count's
        // digits; the count becomes known only after authoring has completed.
        let cost = bytes
            .len()
            .checked_add(document.len())
            .and_then(|n| n.checked_add(4096))
            .filter(|n| *n <= self.max_bytes.saturating_sub(occupied_bytes))
            .ok_or_else(|| invalid("recorded timeline byte budget exceeded"))?;
        let mut identity = Writer::new(Schema::new(*b"FMRP", 1, 1, 0));
        identity
            .put_bytes(self.source_digest.as_bytes())
            .put_u64(index)
            .put_bytes(sha256(bytes).as_bytes())
            .put_bytes(sha256(&document).as_bytes());
        let state_hash = sha256(&identity.finish().map_err(failed)?);
        Ok(RecordedFrame {
            stream,
            inspection,
            state_hash,
            encoded_bytes: cost,
        })
    }

    #[must_use]
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }
    #[must_use]
    pub const fn encoded_bytes(&self) -> usize {
        self.encoded_bytes
    }

    fn select(&mut self, scene: &str, frame: i64) -> Result<(), ServiceError> {
        self.check_scene(scene)?;
        let index = usize::try_from(frame).map_err(|_| invalid("negative recorded frame"))?;
        if index >= self.frames.len() {
            return Err(invalid("frame is outside the captured timeline"));
        }
        self.position = index;
        Ok(())
    }

    fn check_scene(&self, scene: &str) -> Result<(), ServiceError> {
        if scene != self.scene {
            return Err(ServiceError::new(
                WorkerErrorCode::SceneNotFound,
                "unknown captured scene",
            ));
        }
        if self.frames.is_empty() {
            return Err(invalid("recorded timeline is empty"));
        }
        Ok(())
    }

    fn frame(&self) -> Result<WorkerResponse, ServiceError> {
        let source = &self.frames[self.position].stream;
        let FramePayload::Pipe { bytes, digest } = &source.payload else {
            unreachable!()
        };
        let mut copy = Vec::new();
        copy.try_reserve_exact(bytes.len()).map_err(failed)?;
        copy.extend_from_slice(bytes);
        let render_backends = source
            .render_backends
            .iter()
            .map(|b| b.try_clone().map_err(failed))
            .collect::<Result<_, _>>()?;
        Ok(WorkerResponse::Frame(FrameStream {
            scene: self.scene.clone(),
            frame_index: self.position as u64,
            width: source.width,
            height: source.height,
            stride: 0,
            encoding: FrameEncoding::Png,
            payload: FramePayload::Pipe {
                bytes: copy,
                digest: *digest,
            },
            render_backends,
        }))
    }
}

impl WorkerService for RecordedTimeline {
    fn build_id(&self) -> Digest {
        self.build_id
    }
    fn active_scene(&self) -> Option<&str> {
        Some(&self.scene)
    }
    fn last_state_hash(&self) -> Option<Digest> {
        self.frames.get(self.position).map(|f| f.state_hash)
    }

    fn begin_session(&mut self, _: Digest, max_frame_bytes: usize) -> Result<(), ServiceError> {
        self.check_scene(&self.scene)?;
        self.limits.max_frame_bytes = self.limits.max_frame_bytes.min(max_frame_bytes);
        for frame in &self.frames {
            frame.stream.validate(self.limits).map_err(failed)?;
        }
        Ok(())
    }

    fn handle(&mut self, request: SupervisorRequest) -> Result<WorkerResponse, ServiceError> {
        match request {
            SupervisorRequest::EnumerateScenes => {
                Ok(WorkerResponse::Scenes(vec![self.scene.clone()]))
            }
            SupervisorRequest::Scrub { scene, frame }
            | SupervisorRequest::Seek { scene, frame } => {
                self.select(&scene, frame)?;
                self.frame()
            }
            SupervisorRequest::Inspect { scene } => {
                self.check_scene(&scene)?;
                let count = self.frames.len() as u64;
                let revision = self
                    .input_revision
                    .filter(|_| self.position + 1 == self.frames.len());
                let snapshot = &mut self.frames[self.position].inspection;
                let view = snapshot.view.as_mut().expect("admitted capture has a view");
                view.frame_count = count;
                view.input_events = revision.is_some();
                view.input_revision = revision;
                view.live_advance = self.advance_enabled && revision.is_some();
                let bytes = snapshot
                    .to_json(InspectorLimits {
                        max_json_bytes: self.limits.max_studio_data_bytes,
                        ..InspectorLimits::default()
                    })
                    .map_err(failed)?;
                let digest = sha256(&bytes);
                Ok(WorkerResponse::StudioData {
                    scene,
                    kind: StudioDataKind::Inspection,
                    bytes,
                    digest,
                })
            }
            SupervisorRequest::Play { scene, command } => {
                if self.commands >= 4096 {
                    return Err(invalid("recorded position journal budget exceeded"));
                }
                let frame = studio_seek_frame(&scene, &command).map_err(failed)?;
                let previous = self.position;
                self.select(&scene, frame)?;
                let result = self.record_opaque(command);
                if result.is_err() {
                    self.position = previous;
                }
                result
            }
            SupervisorRequest::RestoreCheckpoint(_) => Err(ServiceError::new(
                WorkerErrorCode::CheckpointRejected,
                "captured previews do not restore authored callbacks; reload the source",
            )),
            SupervisorRequest::ReplayJournal(_) => Err(ServiceError::new(
                WorkerErrorCode::ReplayFailed,
                "captured Python execution is opaque; source reload must execute a fresh worker",
            )),
            SupervisorRequest::Event { .. } => Err(invalid(
                "captured preview is read-only; live Python event editing is not connected",
            )),
            SupervisorRequest::Overlay { .. } => {
                Err(invalid("captured camera overlays are not available"))
            }
            SupervisorRequest::Hello { .. } | SupervisorRequest::Shutdown => {
                Err(invalid("request belongs to the worker protocol driver"))
            }
        }
    }
}

/// Start the existing authenticated Studio host with an explicitly supplied
/// authoring-worker builder. Automatic crash re-execution is disabled: arbitrary
/// source effects must never repeat merely because a preview request failed.
pub fn start_host(
    scene: &str,
    mut builder: Box<dyn RebuildDriver + Send>,
    token: CapabilityToken,
    config: StudioHostConfig,
    request_timeout: Duration,
) -> Result<(StudioHost, Arc<StudioWorkerSession>, FrameHub), HostError> {
    use fmn_cache::{NamespacePolicy, Store, StoreConfig};
    use fmn_platform::clock::{Clock, StdClock};
    use fmn_platform::fs::VirtualFs;
    let clock: Arc<dyn Clock> = Arc::new(StdClock::new());
    let store = Store::open(
        Arc::new(VirtualFs::new()),
        Arc::clone(&clock),
        std::env::temp_dir().join("fmn-recorded-studio-cache"),
        StoreConfig::default(),
    )
    .map_err(|e| HostError::Io(std::io::Error::other(e.to_string())))?;
    let cache = store
        .namespace(
            "captured-preview",
            1,
            NamespacePolicy {
                ceiling_bytes: Some(8 * 1024 * 1024),
            },
        )
        .map_err(|e| HostError::Io(std::io::Error::other(e.to_string())))?;
    let mut supervisor = Supervisor::new(
        Box::new(StdWorkerLauncher::default()),
        Arc::clone(&clock),
        cache,
        SupervisorConfig {
            request_timeout,
            auto_restart: false,
            ..SupervisorConfig::default()
        },
    );
    supervisor.install_session(scene, Journal::new())?;
    supervisor.build_and_start(&mut *builder)?;
    let reply = supervisor.request(
        SupervisorRequest::Scrub {
            scene: scene.into(),
            frame: 0,
        },
        &|_| false,
    )?;
    let SupervisorReply::Worker(WorkerResponse::Frame(first)) = reply else {
        return Err(HostError::UnexpectedWorkerResponse);
    };
    let frames = FrameHub::new(config.max_frame_history, config.max_png_bytes)?;
    frames.publish(&first, supervisor.protocol_limits())?;
    let session = Arc::new(
        StudioWorkerSession::new(scene, supervisor, builder, Arc::new(|_| false))?
            .with_fresh_capture_reload()
            .require_guarded_input(),
    );
    let host = StudioHost::bind(Arc::clone(&session), frames.clone(), token, clock, config)?;
    Ok((host, session, frames))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{InspectorView, SpanRegistry};
    use fmn_render::{EngineIdentity, FrameConfig, ScreenMap, Tiling, Viewport};
    use fmn_scene::{RenderBackendRecord, RenderBackendRole};

    pub(super) fn capture(scene: &str, value: u8) -> (FrameStream, InspectorSnapshot) {
        let stage = fmn_scene::studio_bridge::Stage::new();
        let viewport = Viewport {
            width: 2,
            height: 2,
        };
        let map = ScreenMap {
            scale: 1.0,
            origin: [1.0, 1.0],
            y_up: true,
        };
        let config = FrameConfig::new(
            viewport,
            map,
            fmn_core::color::LinearRgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        );
        let backend = RenderBackendRecord::new(
            RenderBackendRole::FrameStream,
            fmn_render::engine::journal(
                EngineIdentity::certified(),
                &config,
                Tiling {
                    macro_tile: 64,
                    fine_tile: 8,
                },
            ),
        )
        .unwrap();
        let bytes = fmn_codec::encode_rgba8(2, 2, &[value; 16], fmn_codec::CompressionLevel::Fast);
        let digest = sha256(&bytes);
        let stream = FrameStream {
            scene: scene.into(),
            frame_index: 100,
            width: 2,
            height: 2,
            stride: 0,
            encoding: FrameEncoding::Png,
            payload: FramePayload::Pipe { bytes, digest },
            render_backends: vec![backend],
        };
        let mut inspect =
            InspectorSnapshot::capture(&stage, &SpanRegistry::new(), InspectorLimits::default())
                .unwrap();
        inspect.view = Some(InspectorView::new(0, 1, 30, viewport, map, false).unwrap());
        inspect.scene_time = f64::from(value) / 30.0;
        (stream, inspect)
    }

    #[test]
    fn recorded_timeline_scrubs_real_pngs_and_matching_native_inspection() {
        let mut worker = RecordedTimeline::new(
            "scene".into(),
            sha256(b"build"),
            sha256(b"source"),
            3,
            1 << 20,
        )
        .unwrap();
        for value in [0, 120, 255] {
            let (f, i) = capture("scene", value);
            worker.push(f, i).unwrap();
        }
        worker.begin_session(sha256(b"host"), 1024).unwrap();
        for index in [2, 0, 1, 2] {
            let WorkerResponse::Frame(frame) = worker
                .handle(SupervisorRequest::Scrub {
                    scene: "scene".into(),
                    frame: index,
                })
                .unwrap()
            else {
                panic!()
            };
            assert_eq!(frame.frame_index, index as u64);
            let FramePayload::Pipe { bytes, .. } = frame.payload else {
                panic!()
            };
            assert_eq!(
                decode_png(&bytes, &PngLimits::default()).unwrap().rgba[0],
                [0, 120, 255][index as usize]
            );
            let WorkerResponse::StudioData { bytes, .. } = worker
                .handle(SupervisorRequest::Inspect {
                    scene: "scene".into(),
                })
                .unwrap()
            else {
                panic!()
            };
            let text = String::from_utf8(bytes).unwrap();
            assert!(text.contains("\"frame_count\":3"));
            assert!(text.contains(&format!("\"frame_index\":{index}")));
            assert!(text.contains("\"input_events\":false"));
        }
        let command = crate::protocol::studio_seek_command("scene", 1).unwrap();
        let WorkerResponse::JournalSegment { journal, .. } = worker
            .handle(SupervisorRequest::Play {
                scene: "scene".into(),
                command,
            })
            .unwrap()
        else {
            panic!()
        };
        let journal = Journal::from_bytes(&journal).unwrap();
        assert!(journal.entries()[0].is_replay_barrier());
        assert!(journal.entries()[0].checkpoint.is_none());
    }

    #[test]
    fn recorded_timeline_refusals_are_atomic_and_bounded() {
        let mut worker =
            RecordedTimeline::new("scene".into(), sha256(b"b"), sha256(b"s"), 1, 1 << 20).unwrap();
        let (frame, inspection) = capture("other", 0);
        assert!(worker.push(frame, inspection).is_err());
        assert_eq!(worker.frame_count(), 0);
        let (frame, inspection) = capture("scene", 0);
        worker.push(frame, inspection).unwrap();
        let used = worker.encoded_bytes();
        let (frame, inspection) = capture("scene", 1);
        assert!(worker.push(frame, inspection).is_err());
        assert_eq!(worker.encoded_bytes(), used);
        assert!(
            worker
                .handle(SupervisorRequest::Scrub {
                    scene: "scene".into(),
                    frame: 1
                })
                .is_err()
        );
        assert!(
            worker
                .handle(SupervisorRequest::Scrub {
                    scene: "other".into(),
                    frame: 0
                })
                .is_err()
        );
        assert!(worker.begin_session(sha256(b"h"), 1).is_err());
        let mut tiny =
            RecordedTimeline::new("scene".into(), sha256(b"b"), sha256(b"s"), 1, 1).unwrap();
        let (frame, inspection) = capture("scene", 0);
        assert!(tiny.push(frame, inspection).is_err());
        assert_eq!(tiny.frame_count(), 0);
        assert!(RecordedTimeline::new("scene".into(), sha256(b"b"), sha256(b"s"), 0, 1).is_err());
    }
}
