//! Camera-bearing artifact -> real isolated-worker protocol -> retained PNGs.
//! Expected pictures are captured from the actual Scene, before serialization.

use std::cell::Cell;
use std::io::Cursor;
use std::rc::Rc;

use fmn_anim::FramePacket;
use fmn_core::color::LinearRgba;
use fmn_frame::convert::rgba16f_to_rgba8;
use fmn_frame::{FrameBuffer, FrameLayout, PixelFormat};
use fmn_hash::sha256;
use fmn_mobject::{Mobject, RecordBuffer, RecordSchema, RenderPrimitive, Stage};
use fmn_render::{
    Camera, CameraConfig, EngineIdentity, FrameConfig, RetainedFrameRenderer,
    RetainedFrameRendererConfig, ScreenMap, Tiling, Viewport,
};
use fmn_scene::recording::SceneBundleRecorder;
use fmn_scene::{
    AssetRead, BundleExportLimits, CaptureReason, EffectClass, Entry, IntegrationError, Journal,
    LifecycleEvent, RuntimeConfig, Scene, SceneSink, TimelineBundle,
};
use fmn_studio::camera_bundle::{CameraBundleView, CameraBundleWorker};
use fmn_studio::protocol::studio_seek_command;
use fmn_studio::{
    CURRENT_VERSION, DebugLayerSet, FramePayload, FrameStream, JournalReplay, ProtocolLimits,
    RequestEnvelope, SupervisorRequest, WorkerErrorCode, WorkerResponse, WorkerServeOutcome,
    WorkerService, read_response, serve_worker, write_request,
};

const NAME: &str = "RecordedOrbit";
const SIZE: (u32, u32) = (64, 40);

fn policy(threads: usize) -> RetainedFrameRendererConfig {
    RetainedFrameRendererConfig {
        frame: FrameConfig::new(
            Viewport {
                width: SIZE.0,
                height: SIZE.1,
            },
            ScreenMap {
                scale: 8.0,
                origin: [32.0, 20.0],
                y_up: true,
            },
            LinearRgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        ),
        tiling: Tiling {
            macro_tile: 64,
            fine_tile: 8,
        },
        engine: EngineIdentity::certified(),
        threads,
    }
}

fn geometry() -> Mobject {
    let mut vector = RecordBuffer::new(RecordSchema::vmobject(), 5).unwrap();
    vector.write_range(
        "point",
        0,
        &[
            -1.0, -0.5, 0.2, 0.0, 1.5, 0.2, 1.0, -0.5, 0.2, 0.0, -0.5, 0.2, -1.0, -0.5, 0.2,
        ],
    );
    vector.write_range("fill_rgba", 0, &[0.1, 0.4, 1.0, 0.8].repeat(5));
    vector.write_range("stroke_width", 0, &[0.0; 5]);
    let schema = RecordSchema::new(
        &[("point", 3), ("d_normal_point", 3), ("rgba", 4)],
        &["point"],
        &["point", "d_normal_point"],
    )
    .unwrap();
    let mut surface = RecordBuffer::new(schema, 4).unwrap();
    for (index, point) in [
        [-0.8, -0.8, -0.2],
        [-0.8, 0.8, -0.2],
        [0.8, -0.8, -0.2],
        [0.8, 0.8, -0.2],
    ]
    .iter()
    .enumerate()
    {
        surface.write(index, "point", point);
        surface.write(
            index,
            "d_normal_point",
            &[point[0], point[1], point[2] + 1.0],
        );
        surface.write(index, "rgba", &[1.0, 0.1, 0.3, 0.9]);
    }
    Mobject::group(vec![
        Mobject::from_buffer(vector),
        Mobject::from_buffer(surface)
            .with_render_primitive(RenderPrimitive::SurfaceGrid { resolution: (2, 2) }),
    ])
}

fn png(stage: &Stage, camera: &Camera) -> Vec<u8> {
    let mut renderer = RetainedFrameRenderer::new(policy(1)).unwrap();
    renderer.render_with_camera(stage, camera).unwrap();
    let mut rgba =
        FrameBuffer::new(FrameLayout::tight(PixelFormat::Rgba8, SIZE.0, SIZE.1).unwrap());
    rgba16f_to_rgba8(renderer.frame(), &mut rgba).unwrap();
    fmn_codec::encode_rgba8(
        SIZE.0,
        SIZE.1,
        rgba.as_bytes(),
        fmn_codec::CompressionLevel::Fast,
    )
}

struct Capture {
    recorder: SceneBundleRecorder,
    expected: Vec<Vec<u8>>,
    different_camera: bool,
}
impl SceneSink for Capture {
    fn event(&mut self, event: LifecycleEvent) -> Result<(), IntegrationError> {
        self.recorder.event(event)
    }
    fn capture(
        &mut self,
        reason: CaptureReason,
        packet: FramePacket,
    ) -> Result<(), IntegrationError> {
        let n = self.expected.len() as f64;
        let mut config = CameraConfig {
            resolution: SIZE,
            fps: 8,
            samples: 4,
            background: LinearRgba {
                r: n / 12.0,
                g: 0.03,
                b: 0.06,
                a: 1.0,
            },
            light_source_position: [-4.0 + n, 5.0, 6.0],
            ..CameraConfig::default()
        };
        config.frame.set_width(5.0).unwrap();
        config
            .frame
            .set_orientation([0.1 + n / 8.0, 0.2, 0.0, 1.0])
            .unwrap();
        if self.different_camera {
            config.frame.set_center([0.6, 0.0, 0.0]).unwrap();
        }
        let camera = Camera::new(config).unwrap();
        let expected = png(&packet.materialize_stage(), &camera);
        assert_ne!(
            expected,
            png(&Stage::new(), &camera),
            "empty geometry is not a valid control"
        );
        self.expected.push(expected);
        self.recorder.capture_with_camera(reason, packet, &camera)
    }
}
fn artifact(different_camera: bool) -> (Vec<u8>, Vec<Vec<u8>>, Rc<Cell<u64>>) {
    let mut scene = Scene::new(
        RuntimeConfig {
            fps: 8,
            ..RuntimeConfig::default()
        },
        19,
    )
    .unwrap();
    let root = scene.add_mobject(geometry()).unwrap();
    let calls = Rc::new(Cell::new(0));
    let called = Rc::clone(&calls);
    scene
        .stage_mut()
        .add_dt_updater(
            root,
            move |stage, mob, dt| {
                called.set(called.get() + 1);
                stage.shift(mob, [dt, 0.0, 0.0]);
            },
            false,
        )
        .unwrap();
    let mut capture = Capture {
        recorder: SceneBundleRecorder::new_render_only_with_camera(
            8,
            BundleExportLimits::default(),
        )
        .unwrap(),
        expected: Vec::new(),
        different_camera,
    };
    scene.wait(Some(0.5), &mut capture).unwrap();
    (
        capture.recorder.finish().unwrap().bytes,
        capture.expected,
        calls,
    )
}
fn worker(bytes: &[u8], threads: usize) -> CameraBundleWorker {
    CameraBundleWorker::new(
        NAME.into(),
        sha256(b"worker build"),
        AssetRead {
            path: "/recorded/orbit.fmtl".into(),
            digest: sha256(bytes),
        },
        TimelineBundle::from_bytes(bytes).unwrap(),
        CameraBundleView {
            renderer: policy(threads),
            seed: 19,
            fixed_camera: None,
        },
    )
    .unwrap()
}
fn scrub(worker: &mut CameraBundleWorker, index: i64) -> FrameStream {
    let WorkerResponse::Frame(frame) = worker
        .handle(SupervisorRequest::Scrub {
            scene: NAME.into(),
            frame: index,
        })
        .unwrap()
    else {
        panic!("frame")
    };
    frame
}
fn bytes(frame: &FrameStream) -> &[u8] {
    let FramePayload::Pipe { bytes, digest } = &frame.payload else {
        panic!("inline PNG")
    };
    assert_eq!(sha256(bytes), *digest);
    bytes
}
fn inspect(worker: &mut CameraBundleWorker) -> String {
    let WorkerResponse::StudioData { bytes, digest, .. } = worker
        .handle(SupervisorRequest::Inspect { scene: NAME.into() })
        .unwrap()
    else {
        panic!("inspection")
    };
    assert_eq!(sha256(&bytes), digest);
    String::from_utf8(bytes).unwrap()
}
fn commit(worker: &mut CameraBundleWorker, frame: i64, position: u64) -> Entry {
    let WorkerResponse::JournalSegment {
        journal,
        start_entry,
        ..
    } = worker
        .handle(SupervisorRequest::Play {
            scene: NAME.into(),
            command: studio_seek_command(NAME, frame).unwrap(),
        })
        .unwrap()
    else {
        panic!("journal")
    };
    assert_eq!(start_entry, position);
    let journal = Journal::from_bytes(&journal).unwrap();
    assert_eq!(journal.entries().len(), 1);
    let entry = journal.entries()[0].clone();
    assert_eq!(entry.effect, EffectClass::Pure);
    assert!(entry.checkpoint.is_none());
    assert_eq!(entry.reads.len(), 2);
    assert!(entry.subprocesses.is_empty());
    entry
}
fn replay(journal: &Journal, from: u64, through: u64) -> SupervisorRequest {
    SupervisorRequest::ReplayJournal(JournalReplay {
        scene: NAME.into(),
        from_entry: from,
        through_entry: through,
        journal: journal.to_bytes().unwrap(),
    })
}

fn fixed_camera() -> Camera {
    let mut config = CameraConfig {
        resolution: SIZE,
        fps: 8,
        light_source_position: [4.0, 6.0, 3.0],
        ..CameraConfig::default()
    };
    config.frame.set_width(5.0).unwrap();
    config.frame.set_center([0.3, -0.2, 0.0]).unwrap();
    config.frame.set_orientation([0.2, 0.3, 0.0, 1.0]).unwrap();
    Camera::new(config).unwrap()
}

fn legacy_artifact(recorded: bool) -> Vec<u8> {
    let mut stage = Stage::new();
    let root = stage.add(geometry());
    stage.add_to_scene(root).unwrap();
    let mut timeline = fmn_anim::Timeline::new(8).unwrap();
    if recorded {
        stage
            .add_dt_updater(
                root,
                |stage, mob, dt| {
                    stage.shift(mob, [dt, 0.0, 0.0]);
                },
                false,
            )
            .unwrap();
        timeline.wait(0.5).unwrap();
    } else {
        let animation = root
            .animate()
            .set_anim_args(fmn_mobject::AnimateArgs {
                run_time: Some(0.5),
                rate_func: Some(fmn_core::rate::linear),
                ..Default::default()
            })
            .unwrap()
            .shift([0.5, 0.0, 0.0])
            .unwrap();
        timeline
            .play(vec![
                fmn_anim::prepare_animation(animation, &mut stage).unwrap(),
            ])
            .unwrap();
    }
    fmn_scene::export_timeline_bundle(timeline, &mut stage, &fmn_core::rng::RngRoot::from_seed(19))
        .unwrap()
}

fn fixed_worker(source: &[u8], camera: Camera, threads: usize) -> CameraBundleWorker {
    CameraBundleWorker::new(
        NAME.into(),
        sha256(b"worker build"),
        AssetRead {
            path: "/recorded/legacy.fmtl".into(),
            digest: sha256(source),
        },
        TimelineBundle::from_bytes(source).unwrap(),
        CameraBundleView {
            renderer: policy(threads),
            seed: 19,
            fixed_camera: Some(camera),
        },
    )
    .unwrap()
}

#[test]
fn legacy_surface_bundles_use_the_explicit_view_for_pure_and_recorded_random_access() {
    for recorded in [false, true] {
        let source = legacy_artifact(recorded);
        let bundle = TimelineBundle::from_bytes(&source).unwrap();
        assert!(!bundle.has_camera_track());
        assert!(bundle.requires_camera());
        assert_eq!(
            bundle.segment_kind(0),
            Some(if recorded {
                fmn_scene::timeline_bundle::BundleSegmentKind::Stateful
            } else {
                fmn_scene::timeline_bundle::BundleSegmentKind::Pure
            })
        );
        let camera = fixed_camera();
        let expected: Vec<_> = (0..bundle.frame_count())
            .map(|index| png(&bundle.stage_at(index).unwrap(), &camera))
            .collect();
        assert!(expected.windows(2).all(|frames| frames[0] != frames[1]));
        assert_ne!(expected[0], png(&Stage::new(), &camera));
        for threads in [1, 4, 16] {
            let mut worker = fixed_worker(&source, camera.clone(), threads);
            for frame in [3, 0, 2, 1, 3] {
                assert_eq!(bytes(&scrub(&mut worker, frame)), expected[frame as usize]);
            }
            assert!(inspect(&mut worker).contains("\"frame_count\":4"));
            assert!(worker.journal_tail().is_empty());
        }
    }
}

#[test]
fn fixed_camera_is_required_validated_and_bound_to_cold_recovery() {
    let source = legacy_artifact(false);
    let bundle = TimelineBundle::from_bytes(&source).unwrap();
    let read = AssetRead {
        path: "/recorded/legacy.fmtl".into(),
        digest: sha256(&source),
    };
    let view = CameraBundleView {
        renderer: policy(1),
        seed: 19,
        fixed_camera: Some(fixed_camera()),
    };
    let identity =
        CameraBundleWorker::input_reads(sha256(b"build"), &read, &bundle, &view).unwrap();
    let mut absent = view.clone();
    absent.fixed_camera = None;
    assert!(CameraBundleWorker::input_reads(sha256(b"build"), &read, &bundle, &absent).is_err());
    let (recorded, _, _) = artifact(false);
    let tracked = TimelineBundle::from_bytes(&recorded).unwrap();
    assert!(CameraBundleWorker::input_reads(sha256(b"build"), &read, &tracked, &view).is_err());
    let mut wrong_viewport = view.clone();
    wrong_viewport.renderer.frame.viewport.width += 1;
    assert!(
        CameraBundleWorker::input_reads(sha256(b"build"), &read, &bundle, &wrong_viewport).is_err()
    );
    let mut wrong_fps = view.clone();
    wrong_fps.fixed_camera = Some(
        Camera::new(CameraConfig {
            resolution: SIZE,
            fps: 9,
            ..CameraConfig::default()
        })
        .unwrap(),
    );
    assert!(CameraBundleWorker::input_reads(sha256(b"build"), &read, &bundle, &wrong_fps).is_err());
    let mut original = fixed_worker(&source, fixed_camera(), 1);
    let entry = commit(&mut original, 2, 0);
    let mut journal = Journal::new();
    journal.record(entry).unwrap();
    let mut matching = fixed_worker(&source, fixed_camera(), 4);
    matching.handle(replay(&journal, 0, 1)).unwrap();
    assert_eq!(
        bytes(&scrub(&mut matching, 2)),
        bytes(&scrub(&mut original, 2))
    );
    let mut changed = fixed_camera();
    changed.frame_mut().set_center([1.0, 0.0, 0.0]).unwrap();
    let changed_view = CameraBundleView {
        fixed_camera: Some(changed.clone()),
        ..view
    };
    assert_ne!(
        identity,
        CameraBundleWorker::input_reads(sha256(b"build"), &read, &bundle, &changed_view).unwrap()
    );
    let mut foreign = fixed_worker(&source, changed, 1);
    assert_eq!(
        foreign.handle(replay(&journal, 0, 1)).unwrap_err().code,
        WorkerErrorCode::ReplayFailed
    );
    assert!(foreign.last_state_hash().is_none());
    assert!(foreign.journal_tail().is_empty());
}

#[test]
fn mixed_camera_pictures_replay_out_of_order_at_every_worker_width() {
    let (source, expected, calls) = artifact(false);
    assert_eq!(expected.len(), 4);
    assert!(expected.windows(2).all(|frames| frames[0] != frames[1]));
    let before = calls.get();
    for threads in [1, 4, 16] {
        let mut worker = worker(&source, threads);
        let mut backend = None;
        for frame in [3, 0, 2, 1, 3, 0] {
            let picture = scrub(&mut worker, frame);
            assert_eq!(bytes(&picture), expected[frame as usize]);
            assert_eq!(picture.render_backends.len(), 1);
            if let Some(previous) = &backend {
                assert_eq!(previous, &picture.render_backends[0]);
            } else {
                backend = Some(picture.render_backends[0].clone());
            }
            assert!(
                picture.render_backends[0]
                    .identity()
                    .windows(b"lumen-retained-camera-cpu-v1".len())
                    .any(|s| s == b"lumen-retained-camera-cpu-v1")
            );
        }
        assert!(
            worker.journal_tail().is_empty(),
            "scrub is not committed playback"
        );
        assert!(worker.last_state_hash().is_none());
        let info = inspect(&mut worker);
        assert!(info.contains("\"frame_count\":4"));
        assert!(info.contains("\"fps\":8"));
        assert!(info.contains("\"input_events\":false"));
        assert!(info.contains("\"scene_time\":0.125"));
    }
    assert_eq!(
        before,
        calls.get(),
        "bundle replay executed an authored updater"
    );
}

#[test]
fn cold_recovery_uses_input_verified_canonical_seek_entries() {
    let (source, expected, _) = artifact(false);
    let mut original = worker(&source, 1);
    let first = commit(&mut original, 3, 0);
    let second = commit(&mut original, 1, 1);
    let mut journal = Journal::new();
    journal.record(first.clone()).unwrap();
    journal.record(second.clone()).unwrap();
    let mut recovered = worker(&source, 4);
    assert_eq!(
        recovered.handle(replay(&journal, 0, 1)).unwrap(),
        WorkerResponse::ReplayComplete {
            from_entry: 0,
            state_hashes: vec![first.state_hash]
        }
    );
    assert!(inspect(&mut recovered).contains("\"frame_index\":3"));
    assert_eq!(
        recovered.handle(replay(&journal, 1, 2)).unwrap(),
        WorkerResponse::ReplayComplete {
            from_entry: 1,
            state_hashes: vec![second.state_hash]
        }
    );
    assert_eq!(bytes(&scrub(&mut recovered, 1)), expected[1]);
    assert_eq!(recovered.last_state_hash(), Some(second.state_hash));
    assert_eq!(commit(&mut recovered, 0, 2).reads, first.reads);
}

#[test]
fn changed_camera_inputs_divergence_and_bad_ranges_never_publish_a_prefix() {
    let (source, _, _) = artifact(false);
    let (different, _, _) = artifact(true);
    let mut original = worker(&source, 1);
    let first = commit(&mut original, 3, 0);
    let second = commit(&mut original, 2, 1);
    let mut valid = Journal::new();
    valid.record(first.clone()).unwrap();
    valid.record(second.clone()).unwrap();
    let mut foreign = worker(&different, 1);
    assert_eq!(
        foreign.handle(replay(&valid, 0, 2)).unwrap_err().code,
        WorkerErrorCode::ReplayFailed
    );
    let mut recovered = worker(&source, 1);
    let before = scrub(&mut recovered, 1);
    for (from, through) in [(0, 3), (1, 2), (2, 1), (0, u64::MAX)] {
        assert!(recovered.handle(replay(&valid, from, through)).is_err());
    }
    let mut tampered = Journal::new();
    tampered.record(first).unwrap();
    let mut wrong = second;
    wrong.state_hash = sha256(b"wrong state");
    tampered.record(wrong).unwrap();
    assert_eq!(
        recovered.handle(replay(&tampered, 0, 2)).unwrap_err().code,
        WorkerErrorCode::ReplayFailed
    );
    assert!(inspect(&mut recovered).contains("\"frame_index\":1"));
    assert!(recovered.last_state_hash().is_none());
    assert!(recovered.journal_tail().is_empty());
    assert_eq!(bytes(&scrub(&mut recovered, 1)), bytes(&before));
    recovered.handle(replay(&valid, 0, 2)).unwrap();
}

#[test]
fn viewport_seed_and_engine_are_truthful_inputs_and_failures_leave_position_intact() {
    let (source, _, _) = artifact(false);
    let bundle = TimelineBundle::from_bytes(&source).unwrap();
    let read = AssetRead {
        path: "/recorded/orbit.fmtl".into(),
        digest: sha256(&source),
    };
    let view = CameraBundleView {
        renderer: policy(1),
        seed: 19,
        fixed_camera: None,
    };
    let identity =
        CameraBundleWorker::input_reads(sha256(b"build"), &read, &bundle, &view).unwrap();
    let mut other = view.clone();
    other.renderer.frame.viewport.width = 48;
    assert_ne!(
        identity,
        CameraBundleWorker::input_reads(sha256(b"build"), &read, &bundle, &other).unwrap()
    );
    other = view.clone();
    other.seed = 20;
    assert_ne!(
        identity,
        CameraBundleWorker::input_reads(sha256(b"build"), &read, &bundle, &other).unwrap()
    );
    other = view;
    other.renderer.engine = EngineIdentity::fast();
    assert!(CameraBundleWorker::input_reads(sha256(b"build"), &read, &bundle, &other).is_err());
    let mut worker = worker(&source, 1);
    scrub(&mut worker, 2);
    worker.begin_session(sha256(b"host"), 1).unwrap();
    assert!(
        worker
            .handle(SupervisorRequest::Scrub {
                scene: NAME.into(),
                frame: 0
            })
            .is_err()
    );
    assert!(inspect(&mut worker).contains("\"frame_index\":2"));
    assert!(worker.begin_session(sha256(b"host"), 0).is_err());
    for frame in [-1, 4, i64::MAX] {
        assert!(
            worker
                .handle(SupervisorRequest::Scrub {
                    scene: NAME.into(),
                    frame
                })
                .is_err()
        );
    }
    assert!(
        worker
            .handle(SupervisorRequest::Overlay {
                scene: NAME.into(),
                layers: DebugLayerSet::ALL
            })
            .is_err()
    );
    assert!(
        worker
            .handle(SupervisorRequest::Inspect {
                scene: "foreign".into()
            })
            .is_err()
    );
    worker
        .begin_session(sha256(b"host"), ProtocolLimits::default().max_frame_bytes)
        .unwrap();
    scrub(&mut worker, 0);
}

#[test]
fn camera_worker_serves_the_real_framed_protocol_and_graceful_shutdown() {
    let (source, expected, _) = artifact(false);
    let limits = ProtocolLimits::default();
    let mut requests = Vec::new();
    for (index, request) in [
        SupervisorRequest::Hello {
            version: CURRENT_VERSION,
            max_frame_bytes: limits.max_frame_bytes as u64,
            supervisor_build: sha256(b"host"),
        },
        SupervisorRequest::Scrub {
            scene: NAME.into(),
            frame: 2,
        },
        SupervisorRequest::Play {
            scene: NAME.into(),
            command: studio_seek_command(NAME, 2).unwrap(),
        },
        SupervisorRequest::Inspect { scene: NAME.into() },
        SupervisorRequest::Shutdown,
    ]
    .into_iter()
    .enumerate()
    {
        write_request(
            &mut requests,
            &RequestEnvelope {
                request_id: index as u64 + 1,
                request,
            },
            limits,
        )
        .unwrap();
    }
    let mut output = Vec::new();
    assert_eq!(
        serve_worker(
            &mut worker(&source, 1),
            &mut Cursor::new(requests),
            &mut output,
            limits
        )
        .unwrap(),
        WorkerServeOutcome::Shutdown
    );
    let mut output = Cursor::new(output);
    read_response(&mut output, limits).unwrap();
    let response = read_response(&mut output, limits).unwrap();
    assert_eq!(response.request_id, 2);
    let WorkerResponse::Frame(frame) = response.response else {
        panic!("frame")
    };
    assert_eq!(bytes(&frame), expected[2]);
    assert!(matches!(
        read_response(&mut output, limits).unwrap().response,
        WorkerResponse::JournalSegment { .. }
    ));
    assert!(matches!(
        read_response(&mut output, limits).unwrap().response,
        WorkerResponse::StudioData { .. }
    ));
    read_response(&mut output, limits).unwrap();
}
