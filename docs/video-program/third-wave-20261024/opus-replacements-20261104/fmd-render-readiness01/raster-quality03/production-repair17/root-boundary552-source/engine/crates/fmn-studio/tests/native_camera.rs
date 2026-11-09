//! Camera-native Studio acceptance against the real retained mixed renderer.

use std::cell::Cell;
use std::rc::Rc;

use fmn_core::color::LinearRgba;
use fmn_frame::convert::rgba16f_to_rgba8;
use fmn_frame::{FrameBuffer, FrameLayout, PixelFormat};
use fmn_mobject::{
    ImageColorSpace, ImageResource, ImageSampler, Mobject, RecordBuffer, RecordSchema,
    RenderPrimitive,
};
use fmn_render::{
    Camera, CameraConfig, CameraFrame, EngineIdentity, FrameConfig, RetainedFrameRenderer,
    RetainedFrameRendererConfig, ScreenMap, Tiling, Viewport,
};
use fmn_scene::{EventPayload, Journal, Key, Modifiers, RuntimeConfig, Scene};
use fmn_studio::native::{
    NativeReplayPolicy, NativeSceneProgram, NativeSceneWorker, NativeSegment, NativeWorkerConfig,
};
use fmn_studio::protocol::studio_seek_command;
use fmn_studio::{
    Checkpoint, DebugLayerSet, FrameHub, FramePayload, FrameStream, JournalReplay, ProtocolLimits,
    ServiceError, SupervisorRequest, WorkerErrorCode, WorkerResponse, WorkerService,
    protocol_digest,
};

const NAME: &str = "CameraScene";

#[derive(Clone, Copy, Debug)]
enum Kind {
    Vector,
    Surface,
    Mesh,
    Dots,
    Image,
    Mixed,
}

fn object(kind: Kind) -> Mobject {
    match kind {
        Kind::Vector => {
            let mut buffer = RecordBuffer::new(RecordSchema::vmobject(), 3).unwrap();
            buffer.write_range(
                "point",
                0,
                &[-1.0, -0.8, 0.0, 0.0, 1.5, 0.0, 1.0, -0.8, 0.0],
            );
            buffer.write_range("fill_rgba", 0, &[0.1, 0.3, 1.0, 0.8].repeat(3));
            Mobject::from_buffer(buffer)
        }
        Kind::Surface | Kind::Mesh => {
            let schema = RecordSchema::new(
                &[("point", 3), ("d_normal_point", 3), ("rgba", 4)],
                &["point"],
                &["point", "d_normal_point"],
            )
            .unwrap();
            let points = if matches!(kind, Kind::Surface) {
                vec![
                    [-1.0, -1.0, 0.2],
                    [-1.0, 1.0, 0.2],
                    [1.0, -1.0, 0.2],
                    [1.0, 1.0, 0.2],
                ]
            } else {
                vec![[-1.0, -1.0, 0.2], [1.0, -1.0, 0.2], [0.0, 1.0, 0.2]]
            };
            let mut buffer = RecordBuffer::new(schema, points.len()).unwrap();
            for (index, point) in points.into_iter().enumerate() {
                buffer.write(index, "point", &point);
                buffer.write(
                    index,
                    "d_normal_point",
                    &[point[0], point[1], point[2] + 1.0],
                );
                buffer.write(index, "rgba", &[1.0, 0.2, 0.1, 0.9]);
            }
            let primitive = if matches!(kind, Kind::Surface) {
                RenderPrimitive::SurfaceGrid { resolution: (2, 2) }
            } else {
                RenderPrimitive::TriangleMesh
            };
            Mobject::from_buffer(buffer).with_render_primitive(primitive)
        }
        Kind::Dots => {
            let schema = RecordSchema::new(
                &[("point", 3), ("rgba", 4), ("radius", 1), ("glow_factor", 1)],
                &["point"],
                &["point"],
            )
            .unwrap();
            let mut buffer = RecordBuffer::new(schema, 1).unwrap();
            buffer.write(0, "point", &[0.0, 0.0, 0.4]);
            buffer.write(0, "rgba", &[0.1, 1.0, 0.3, 1.0]);
            buffer.write(0, "radius", &[0.7]);
            buffer.write(0, "glow_factor", &[0.0]);
            Mobject::from_buffer(buffer).with_render_primitive(RenderPrimitive::DotCloud)
        }
        Kind::Image => {
            let schema = RecordSchema::new(
                &[("point", 3), ("im_coords", 2), ("opacity", 1)],
                &["point"],
                &["point"],
            )
            .unwrap();
            let mut buffer = RecordBuffer::new(schema, 6).unwrap();
            let vertices = [
                ([-0.8, 0.8, 0.1], [0.0, 0.0]),
                ([-0.8, -0.8, 0.1], [0.0, 1.0]),
                ([0.8, 0.8, 0.1], [1.0, 0.0]),
                ([0.8, 0.8, 0.1], [1.0, 0.0]),
                ([-0.8, -0.8, 0.1], [0.0, 1.0]),
                ([0.8, -0.8, 0.1], [1.0, 1.0]),
            ];
            for (index, (point, uv)) in vertices.into_iter().enumerate() {
                buffer.write(index, "point", &point);
                buffer.write(index, "im_coords", &uv);
                buffer.write(index, "opacity", &[0.85]);
            }
            let image = ImageResource::rgba8(
                2,
                2,
                vec![
                    255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
                ],
                ImageColorSpace::Srgb,
                ImageSampler::default(),
            )
            .unwrap();
            Mobject::from_buffer(buffer).with_image_resource(image)
        }
        Kind::Mixed => Mobject::group(vec![
            object(Kind::Surface),
            object(Kind::Vector),
            object(Kind::Image),
            object(Kind::Dots),
        ]),
    }
}

fn program(kind: Kind) -> Result<NativeSceneProgram, ServiceError> {
    let mut scene = Scene::new(RuntimeConfig::default(), 17).unwrap();
    let root = scene.stage_mut().add(object(kind));
    scene.stage_mut().add_to_scene(root).unwrap();
    scene
        .stage_mut()
        .add_updater(
            root,
            |stage, target| {
                stage.shift_many(&[target], [0.1, 0.0, 0.0]);
            },
            false,
        )
        .unwrap();
    NativeSceneProgram::new(
        scene,
        vec![NativeSegment::Wait {
            duration: Some(1.0),
        }],
        30,
    )
}

fn config(threads: usize) -> NativeWorkerConfig {
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
    let mut config = NativeWorkerConfig::new(
        NAME,
        protocol_digest(b"camera-test-build"),
        protocol_digest(b"camera-test-source"),
        31,
        30,
        renderer,
    );
    let mut frame = CameraFrame::default();
    frame.set_shape([4.0, 4.0]).unwrap();
    config.camera = Some(CameraConfig {
        resolution: (64, 64),
        fps: 30,
        background,
        frame,
        ..CameraConfig::default()
    });
    config.replay = NativeReplayPolicy::ColdVerified;
    config
}

fn seek(worker: &mut NativeSceneWorker, frame: i64) -> FrameStream {
    let response = worker
        .handle(SupervisorRequest::Scrub {
            scene: NAME.into(),
            frame,
        })
        .unwrap();
    let WorkerResponse::Frame(frame) = response else {
        panic!("expected native camera frame")
    };
    frame.validate(ProtocolLimits::default()).unwrap();
    // This also decodes the actual PNG and checks the publication pixel budget.
    FrameHub::new(1, 1_000_000)
        .unwrap()
        .publish(&frame, ProtocolLimits::default())
        .unwrap();
    frame
}

fn png(frame: &FrameStream) -> &[u8] {
    let FramePayload::Pipe { bytes, .. } = &frame.payload else {
        panic!("inline PNG")
    };
    bytes
}

fn direct(kind: Kind, frame: u64, config: &NativeWorkerConfig) -> Vec<u8> {
    let mut scene = program(kind).unwrap();
    scene.advance_to(frame).unwrap();
    let camera = Camera::new(config.camera.clone().unwrap()).unwrap();
    let mut renderer = RetainedFrameRenderer::new(config.renderer).unwrap();
    renderer
        .render_with_camera(scene.preview().stage(), &camera)
        .unwrap();
    let mut rgba = FrameBuffer::new(FrameLayout::tight(PixelFormat::Rgba8, 64, 64).unwrap());
    rgba16f_to_rgba8(renderer.frame(), &mut rgba).unwrap();
    fmn_codec::encode_rgba8(64, 64, rgba.as_bytes(), fmn_codec::CompressionLevel::Fast)
}

#[test]
fn every_primitive_and_mixed_painter_sequence_matches_direct_lumen_pixels() {
    for kind in [
        Kind::Vector,
        Kind::Surface,
        Kind::Mesh,
        Kind::Dots,
        Kind::Image,
        Kind::Mixed,
    ] {
        let policy = config(1);
        let expected = direct(kind, 4, &policy);
        let mut worker = NativeSceneWorker::new(policy, move || program(kind)).unwrap();
        let frame = seek(&mut worker, 4);
        assert_eq!(png(&frame), expected, "{kind:?}");
        assert_eq!((frame.width, frame.height, frame.frame_index), (64, 64, 4));
        assert_eq!(frame.render_backends.len(), 1);
    }
}

#[test]
fn nonvector_content_still_refuses_without_explicit_camera_selection() {
    let mut policy = config(1);
    policy.camera = None;
    let mut worker = NativeSceneWorker::new(policy, || program(Kind::Surface)).unwrap();
    let error = worker
        .handle(SupervisorRequest::Scrub {
            scene: NAME.into(),
            frame: 0,
        })
        .unwrap_err();
    assert!(error.message.contains("render_with_camera"));
}

#[test]
fn camera_scene_animates_rewinds_and_reconstructs_callback_state() {
    let mut worker = NativeSceneWorker::new(config(1), || program(Kind::Mixed)).unwrap();
    let initial = seek(&mut worker, 0);
    let moved = seek(&mut worker, 4);
    assert_ne!(png(&initial), png(&moved));
    assert_eq!(seek(&mut worker, 0), initial);
    assert_eq!(seek(&mut worker, 4), moved);
}

#[test]
fn camera_bits_and_backend_identity_are_independent_of_render_team_width() {
    let mut reference = NativeSceneWorker::new(config(1), || program(Kind::Mixed)).unwrap();
    let expected = seek(&mut reference, 4);
    for threads in [4, 16] {
        let mut worker = NativeSceneWorker::new(config(threads), || program(Kind::Mixed)).unwrap();
        assert_eq!(seek(&mut worker, 4), expected);
    }
}

#[test]
fn camera_constructor_refuses_inconsistent_policy_before_factory_execution() {
    let calls = Rc::new(Cell::new(0));
    for variant in 0..5 {
        let mut policy = config(1);
        match variant {
            0 => policy.camera.as_mut().unwrap().resolution = (128, 64),
            1 => policy.camera.as_mut().unwrap().fps = 24,
            2 => policy.camera.as_mut().unwrap().background.r = 0.5,
            3 => policy.renderer.engine = EngineIdentity::fast(),
            _ => policy.camera.as_mut().unwrap().light_source_position[0] = f64::NAN,
        }
        let observed = Rc::clone(&calls);
        assert!(
            NativeSceneWorker::new(policy, move || {
                observed.set(observed.get() + 1);
                program(Kind::Mixed)
            })
            .is_err()
        );
    }
    assert_eq!(calls.get(), 0);
}

#[test]
fn camera_inspection_does_not_advertise_affine_input_or_publish_false_overlays() {
    let mut worker = NativeSceneWorker::new(config(1), || program(Kind::Mixed)).unwrap();
    let before = seek(&mut worker, 3);
    let WorkerResponse::StudioData { bytes, .. } = worker
        .handle(SupervisorRequest::Inspect { scene: NAME.into() })
        .unwrap()
    else {
        panic!("inspection")
    };
    let json = String::from_utf8(bytes).unwrap();
    assert!(json.contains("\"input_events\":false"));
    assert!(json.contains("\"frame_index\":3"));
    for request in [
        SupervisorRequest::Event {
            scene: NAME.into(),
            event: EventPayload::KeyPress {
                key: Key::ArrowUp,
                modifiers: Modifiers::NONE,
            },
        },
        SupervisorRequest::Overlay {
            scene: NAME.into(),
            layers: DebugLayerSet::ALL,
        },
    ] {
        assert_eq!(
            worker.handle(request).unwrap_err().code,
            WorkerErrorCode::InvalidRequest
        );
    }
    assert_eq!(seek(&mut worker, 3), before);
}

fn committed(worker: &mut NativeSceneWorker) -> Journal {
    let WorkerResponse::JournalSegment { journal, .. } = worker
        .handle(SupervisorRequest::Play {
            scene: NAME.into(),
            command: studio_seek_command(NAME, 3).unwrap(),
        })
        .unwrap()
    else {
        panic!("journal")
    };
    Journal::from_bytes(&journal).unwrap()
}

#[test]
fn changed_camera_policy_invalidates_replay_before_running_any_factory() {
    let mut original = NativeSceneWorker::new(config(1), || program(Kind::Mixed)).unwrap();
    let journal = committed(&mut original);
    assert!(
        journal.entries()[0]
            .reads
            .iter()
            .any(|read| read.path == "native/camera-capture-policy")
    );
    for variant in 0..5 {
        let mut policy = config(1);
        let camera = policy.camera.as_mut().unwrap();
        match variant {
            0 => {
                camera.frame.set_center([0.5, 0.0, 0.0]).unwrap();
            }
            1 => {
                camera.frame.set_field_of_view(0.7).unwrap();
            }
            2 => {
                camera
                    .frame
                    .set_euler_angles(Some(0.3), Some(0.4), None)
                    .unwrap();
            }
            3 => camera.samples = 4,
            _ => camera.light_source_position = [10.0, 0.0, 10.0],
        }
        let calls = Rc::new(Cell::new(0));
        let observed = Rc::clone(&calls);
        let mut worker = NativeSceneWorker::new(policy, move || {
            observed.set(observed.get() + 1);
            program(Kind::Mixed)
        })
        .unwrap();
        let error = worker
            .handle(SupervisorRequest::ReplayJournal(JournalReplay {
                scene: NAME.into(),
                from_entry: 0,
                through_entry: 1,
                journal: journal.to_bytes().unwrap(),
            }))
            .unwrap_err();
        assert_eq!(error.code, WorkerErrorCode::ReplayFailed);
        assert_eq!(calls.get(), 1, "replay must not invoke the factory");
        assert_eq!(worker.program().unwrap().frame_index(), 0);
    }
}

#[test]
fn restored_and_replayed_camera_workers_keep_the_selected_renderer() {
    let mut original = NativeSceneWorker::new(config(1), || program(Kind::Mixed)).unwrap();
    let journal = committed(&mut original);
    let entry = &journal.entries()[0];
    let expected = seek(&mut original, 6);
    let mut restored = NativeSceneWorker::new(config(1), || program(Kind::Mixed)).unwrap();
    restored
        .handle(SupervisorRequest::RestoreCheckpoint(Checkpoint {
            scene: NAME.into(),
            after_entry: 0,
            state_hash: entry.state_hash,
            state: entry.checkpoint.clone().unwrap(),
        }))
        .unwrap();
    assert_eq!(seek(&mut restored, 6), expected);
    let mut replayed = NativeSceneWorker::new(config(1), || program(Kind::Mixed)).unwrap();
    replayed
        .handle(SupervisorRequest::ReplayJournal(JournalReplay {
            scene: NAME.into(),
            from_entry: 0,
            through_entry: 1,
            journal: journal.to_bytes().unwrap(),
        }))
        .unwrap();
    assert_eq!(seek(&mut replayed, 6), expected);
}
