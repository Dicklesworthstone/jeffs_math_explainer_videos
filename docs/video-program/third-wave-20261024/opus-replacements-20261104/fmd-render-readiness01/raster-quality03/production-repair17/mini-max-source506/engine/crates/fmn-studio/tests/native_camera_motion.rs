//! Moving-camera acceptance: actual native animations, frozen captures and PNGs.

use std::cell::Cell;
use std::rc::Rc;

use fmn_anim::{Animation, Transform};
use fmn_core::color::LinearRgba;
use fmn_frame::convert::rgba16f_to_rgba8;
use fmn_frame::{FrameBuffer, FrameLayout, PixelFormat};
use fmn_mobject::{Mob, Mobject, RecordBuffer, RecordSchema, Uniforms};
use fmn_render::{
    Camera, CameraConfig, CameraFrame, EngineIdentity, FrameConfig, RetainedFrameRenderer,
    RetainedFrameRendererConfig, ScreenMap, Tiling, Viewport,
};
use fmn_scene::studio_bridge::FramePacket;
use fmn_scene::{
    CameraRig, CaptureReason, IntegrationError, Journal, PlayOverrides, RuntimeConfig, Scene,
    SceneSink,
};
use fmn_studio::native::{
    NativeReplayPolicy, NativeSceneProgram, NativeSceneWorker, NativeSegment, NativeWorkerConfig,
};
use fmn_studio::protocol::studio_seek_command;
use fmn_studio::{
    Checkpoint, FramePayload, FrameStream, JournalReplay, ServiceError, SupervisorRequest,
    WorkerErrorCode, WorkerResponse, WorkerService, protocol_digest,
};

const NAME: &str = "MovingCamera";

fn policy(fps: u32, threads: usize) -> NativeWorkerConfig {
    let background = LinearRgba {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    let renderer = RetainedFrameRendererConfig {
        frame: FrameConfig::new(
            Viewport {
                width: 64,
                height: 64,
            },
            ScreenMap {
                scale: 16.0,
                origin: [32.0, 32.0],
                y_up: true,
            },
            background,
        ),
        tiling: Tiling {
            macro_tile: 32,
            fine_tile: 8,
        },
        engine: EngineIdentity::certified(),
        threads,
    };
    let mut frame = CameraFrame::default();
    frame.set_shape([4.0, 4.0]).unwrap();
    let mut config = NativeWorkerConfig::new(
        NAME,
        protocol_digest(b"rig-build"),
        protocol_digest(b"rig-program-v1"),
        u64::from(fps) + 1,
        fps,
        renderer,
    );
    config.camera = Some(CameraConfig {
        resolution: (64, 64),
        fps,
        background,
        frame,
        ..CameraConfig::default()
    });
    config.replay = NativeReplayPolicy::ColdVerified;
    config.checkpoint_frames = 1;
    config
}

fn source(fps: u32, fixed: bool) -> (Scene, CameraRig, Mob, Vec<Box<dyn Animation>>) {
    let base = policy(fps, 1).camera.unwrap();
    let mut scene = Scene::new(
        RuntimeConfig {
            fps,
            ..RuntimeConfig::default()
        },
        37,
    )
    .unwrap();
    let mut buffer = RecordBuffer::new(RecordSchema::vmobject(), 3).unwrap();
    buffer.write_range(
        "point",
        0,
        &[-1.0, -0.8, 0.0, 0.0, 1.5, 0.0, 1.0, -0.8, 0.0],
    );
    buffer.write_range("fill_rgba", 0, &[0.1, 0.7, 1.0, 1.0].repeat(3));
    let body = scene
        .add_mobject(Mobject::from_buffer(buffer).with_uniforms(Uniforms {
            is_fixed_in_frame: if fixed { 1.0 } else { 0.0 },
            ..Uniforms::default()
        }))
        .unwrap();
    let rig = CameraRig::new(&mut scene, &base).unwrap();
    // A mutable closure continues after the camera Transform resumes its root.
    let mut tick = 0_u32;
    scene
        .stage_mut()
        .add_dt_updater(
            rig.center()[0],
            move |stage, handle, dt| {
                tick += 1;
                let x = stage.tracker_value(handle).unwrap();
                stage
                    .set_tracker_value(handle, x + f64::from(tick) * dt * 0.1)
                    .unwrap();
            },
            false,
        )
        .unwrap();
    let target = scene.stage_mut().copy_family(rig.root()).unwrap();
    let handles = scene.stage().get(target).unwrap().submobjects().to_vec();
    let mut pose = base.frame.clone();
    pose.set_euler_angles(Some(0.3), Some(0.4), Some(0.2))
        .unwrap();
    let q = pose.orientation();
    let values = [
        0.8, 0.2, 0.1, 2.0, q[0], q[1], q[2], q[3], 0.9, 5.0, 2.0, 8.0,
    ];
    for (handle, value) in handles.into_iter().zip(values) {
        scene.stage_mut().set_tracker_value(handle, value).unwrap();
    }
    (
        scene,
        rig,
        body,
        vec![Box::new(Transform::new(rig.root(), target))],
    )
}

fn program(fps: u32, fixed: bool) -> Result<NativeSceneProgram, ServiceError> {
    let (scene, rig, _, animations) = source(fps, fixed);
    NativeSceneProgram::new(
        scene,
        vec![
            NativeSegment::Play {
                animations,
                overrides: PlayOverrides {
                    run_time: Some(0.5),
                    ..PlayOverrides::default()
                },
            },
            NativeSegment::Wait {
                duration: Some(0.5),
            },
        ],
        u64::from(fps),
    )?
    .with_camera_rig(rig)
}

fn worker(fps: u32, threads: usize, fixed: bool) -> NativeSceneWorker {
    NativeSceneWorker::new(policy(fps, threads), move || program(fps, fixed)).unwrap()
}

fn frame(worker: &mut NativeSceneWorker, index: i64) -> FrameStream {
    let response = worker
        .handle(SupervisorRequest::Scrub {
            scene: NAME.into(),
            frame: index,
        })
        .unwrap();
    let WorkerResponse::Frame(frame) = response else {
        panic!("camera frame")
    };
    frame
}

fn png(frame: &FrameStream) -> &[u8] {
    let FramePayload::Pipe { bytes, .. } = &frame.payload else {
        panic!("PNG pipe")
    };
    bytes
}

fn direct(stage: &fmn_mobject::Stage, rig: CameraRig, policy: &NativeWorkerConfig) -> Vec<u8> {
    let camera = Camera::new(rig.sample(stage, policy.camera.as_ref().unwrap()).unwrap()).unwrap();
    let mut renderer = RetainedFrameRenderer::new(policy.renderer).unwrap();
    renderer.render_with_camera(stage, &camera).unwrap();
    let mut rgba = FrameBuffer::new(FrameLayout::tight(PixelFormat::Rgba8, 64, 64).unwrap());
    rgba16f_to_rgba8(renderer.frame(), &mut rgba).unwrap();
    fmn_codec::encode_rgba8(64, 64, rgba.as_bytes(), fmn_codec::CompressionLevel::Fast)
}

#[derive(Default)]
struct Captures(Vec<FramePacket>);
impl SceneSink for Captures {
    fn capture(&mut self, _: CaptureReason, packet: FramePacket) -> Result<(), IntegrationError> {
        self.0.push(packet);
        Ok(())
    }
}

#[test]
fn every_camera_animation_capture_matches_ordinary_scene_playback_at_two_rates() {
    for fps in [8, 30] {
        let (mut scene, rig, body, animations) = source(fps, false);
        let body_bounds = scene.stage().get_bounding_box(body);
        let mut captures = Captures::default();
        scene
            .play(
                animations,
                PlayOverrides {
                    run_time: Some(0.5),
                    ..PlayOverrides::default()
                },
                &mut captures,
            )
            .unwrap();
        scene.wait(Some(0.5), &mut captures).unwrap();
        let policy = policy(fps, 1);
        let mut worker = worker(fps, 1, false);
        let initial = frame(&mut worker, 0);
        for packet in captures.0 {
            let stage = packet.materialize_stage();
            assert_eq!(
                stage.get_bounding_box(body),
                body_bounds,
                "only the camera is animated"
            );
            let expected = direct(&stage, rig, &policy);
            let actual = frame(&mut worker, packet.frame_index());
            assert_eq!(
                png(&actual),
                expected,
                "frame {} at {fps}fps",
                packet.frame_index()
            );
            assert_eq!(
                actual.render_backends, initial.render_backends,
                "pose is state, not an unbounded backend list"
            );
        }
        assert_ne!(png(&initial), png(&frame(&mut worker, i64::from(fps))));
    }
}

#[test]
fn fixed_frame_geometry_stays_fixed_during_camera_translation_zoom_and_rotation() {
    let mut worker = worker(8, 1, true);
    let initial = frame(&mut worker, 0);
    for index in 1..=8 {
        assert_eq!(png(&frame(&mut worker, index)), png(&initial));
    }
}

#[test]
fn camera_sampling_does_not_modify_native_state_or_repeat_updater_ticks() {
    let mut worker = worker(8, 1, false);
    let before = frame(&mut worker, 6);
    let stage = worker
        .program()
        .unwrap()
        .preview()
        .stage()
        .snapshot()
        .to_bytes()
        .unwrap();
    for _ in 0..3 {
        assert_eq!(frame(&mut worker, 6), before);
        assert_eq!(
            worker
                .program()
                .unwrap()
                .preview()
                .stage()
                .snapshot()
                .to_bytes()
                .unwrap(),
            stage
        );
    }
    assert_eq!(
        frame(&mut worker, 2),
        frame(&mut self::worker(8, 1, false), 2)
    );
}

#[test]
fn moving_camera_frames_and_identities_match_across_render_teams() {
    let mut baseline = worker(8, 1, false);
    let expected: Vec<_> = (0..=8).map(|index| frame(&mut baseline, index)).collect();
    for threads in [4, 16] {
        let mut parallel = worker(8, threads, false);
        for (index, expected) in expected.iter().enumerate() {
            assert_eq!(
                &frame(&mut parallel, i64::try_from(index).unwrap()),
                expected
            );
        }
    }
}

fn commit(worker: &mut NativeSceneWorker, index: i64) -> Journal {
    let WorkerResponse::JournalSegment { journal, .. } = worker
        .handle(SupervisorRequest::Play {
            scene: NAME.into(),
            command: studio_seek_command(NAME, index).unwrap(),
        })
        .unwrap()
    else {
        panic!("journal")
    };
    Journal::from_bytes(&journal).unwrap()
}

#[test]
fn checkpoint_recovery_and_replay_continue_the_reconstructed_camera_closure() {
    let mut original = worker(8, 1, false);
    let journal = commit(&mut original, 6);
    let entry = &journal.entries()[0];
    assert!(
        entry
            .reads
            .iter()
            .any(|read| read.path == "native/camera-rig-binding")
    );
    let expected = frame(&mut original, 8);
    let mut restored = worker(8, 1, false);
    restored
        .handle(SupervisorRequest::RestoreCheckpoint(Checkpoint {
            scene: NAME.into(),
            after_entry: 0,
            state_hash: entry.state_hash,
            state: entry.checkpoint.clone().unwrap(),
        }))
        .unwrap();
    assert_eq!(frame(&mut restored, 8), expected);
    let mut replayed = worker(8, 1, false);
    replayed
        .handle(SupervisorRequest::ReplayJournal(JournalReplay {
            scene: NAME.into(),
            from_entry: 0,
            through_entry: 1,
            journal: journal.to_bytes().unwrap(),
        }))
        .unwrap();
    assert_eq!(frame(&mut replayed, 8), expected);
}

#[test]
fn inspection_reads_animated_zoom_even_before_a_new_renderer_has_drawn() {
    let mut worker = worker(8, 1, false);
    commit(&mut worker, 4);
    let WorkerResponse::StudioData { bytes, .. } = worker
        .handle(SupervisorRequest::Inspect { scene: NAME.into() })
        .unwrap()
    else {
        panic!("inspection")
    };
    let text = String::from_utf8(bytes).unwrap();
    assert!(text.contains("\"scale\":32"), "{text}");
    assert!(text.contains("\"input_events\":false"));
}

#[test]
fn a_rig_requires_camera_policy_and_cannot_be_bound_late_or_to_another_scene() {
    let mut config = policy(8, 1);
    config.camera = None;
    assert!(NativeSceneWorker::new(config, || program(8, false)).is_err());
    let (scene, rig, _, animations) = source(8, false);
    let mut program = NativeSceneProgram::new(
        scene,
        vec![NativeSegment::Play {
            animations,
            overrides: PlayOverrides::default(),
        }],
        8,
    )
    .unwrap();
    program.advance_to(1).unwrap();
    assert!(program.with_camera_rig(rig).is_err());
    assert!(
        NativeSceneProgram::new(Scene::default(), Vec::new(), 1)
            .unwrap()
            .with_camera_rig(rig)
            .is_err()
    );
}

#[test]
fn invalid_camera_updates_cannot_publish_frames_or_successful_checkpoints() {
    let factory = || {
        let mut scene = Scene::new(
            RuntimeConfig {
                fps: 8,
                ..RuntimeConfig::default()
            },
            17,
        )
        .unwrap();
        let rig = CameraRig::new(&mut scene, &policy(8, 1).camera.unwrap()).unwrap();
        scene
            .stage_mut()
            .add_updater(
                rig.width(),
                |stage, width| {
                    stage.set_tracker_value(width, 0.0).unwrap();
                },
                false,
            )
            .unwrap();
        NativeSceneProgram::new(
            scene,
            vec![NativeSegment::Wait {
                duration: Some(1.0),
            }],
            8,
        )?
        .with_camera_rig(rig)
    };
    let mut worker = NativeSceneWorker::new(policy(8, 1), factory).unwrap();
    let good = frame(&mut worker, 0);
    assert!(
        worker
            .handle(SupervisorRequest::Scrub {
                scene: NAME.into(),
                frame: 1
            })
            .is_err()
    );
    assert!(worker.program().is_none());
    assert!(
        worker
            .handle(SupervisorRequest::Play {
                scene: NAME.into(),
                command: studio_seek_command(NAME, 1).unwrap()
            })
            .is_err()
    );
    assert!(worker.journal_tail().is_empty());
    assert_eq!(frame(&mut worker, 0), good);
}

#[test]
fn inconsistent_factory_rig_selection_is_rejected_before_execution() {
    let count = Rc::new(Cell::new(0));
    let observed = Rc::clone(&count);
    let mut worker = NativeSceneWorker::new(policy(8, 1), move || {
        let iteration = observed.get();
        observed.set(iteration + 1);
        let (mut scene, rig, _, _) = source(8, false);
        let other = CameraRig::new(&mut scene, &policy(8, 1).camera.unwrap()).unwrap();
        NativeSceneProgram::new(
            scene,
            vec![NativeSegment::Wait {
                duration: Some(1.0),
            }],
            8,
        )?
        .with_camera_rig(if iteration == 0 { rig } else { other })
    })
    .unwrap();
    let error = worker
        .handle(SupervisorRequest::Play {
            scene: NAME.into(),
            command: studio_seek_command(NAME, 1).unwrap(),
        })
        .unwrap_err();
    assert_eq!(error.code, WorkerErrorCode::InvalidRequest);
    assert!(error.message.contains("camera rig binding"));
    assert_eq!(worker.program().unwrap().frame_index(), 0);
}
