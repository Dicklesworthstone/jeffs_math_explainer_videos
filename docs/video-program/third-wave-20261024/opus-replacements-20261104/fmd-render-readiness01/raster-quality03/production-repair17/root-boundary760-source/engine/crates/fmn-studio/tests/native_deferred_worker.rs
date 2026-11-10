//! Deferred construction through real pixels, native callback replay and IPC.

use std::cell::Cell;
use std::io::Cursor;
use std::rc::Rc;

use fmn_core::color::LinearRgba;
use fmn_mobject::{Mob, Mobject, RecordBuffer, RecordSchema};
use fmn_render::{
    EngineIdentity, FrameConfig, RetainedFrameRendererConfig, ScreenMap, Tiling, Viewport,
};
use fmn_scene::{Entry, Journal, RuntimeConfig, Scene, SceneError};
use fmn_studio::native::{
    NativeReplayPolicy, NativeSceneProgram, NativeSceneWorker, NativeSegment, NativeWorkerConfig,
};
use fmn_studio::protocol::studio_seek_command;
use fmn_studio::{
    CURRENT_VERSION, Checkpoint, FrameHub, FramePayload, FrameStream, JournalReplay,
    ProtocolLimits, RequestEnvelope, ServiceError, SupervisorRequest, WorkerResponse,
    WorkerService, protocol_digest, read_response, serve_worker, write_request,
};

const NAME: &str = "DeferredScene";

fn initial() -> (Scene, Mob) {
    let mut scene = Scene::new(
        RuntimeConfig {
            fps: 8,
            ..RuntimeConfig::default()
        },
        71,
    )
    .unwrap();
    let points = [
        [-0.5_f32, -0.5, 0.0],
        [0.0, -0.5, 0.0],
        [0.5, -0.5, 0.0],
        [0.5, 0.0, 0.0],
        [0.5, 0.5, 0.0],
        [0.0, 0.5, 0.0],
        [-0.5, 0.5, 0.0],
        [-0.5, 0.0, 0.0],
        [-0.5, -0.5, 0.0],
    ];
    let mut buffer = RecordBuffer::new(RecordSchema::vmobject(), points.len()).unwrap();
    for (index, point) in points.iter().enumerate() {
        buffer.write(index, "point", point);
        buffer.write(index, "fill_rgba", &[0.1, 0.3, 1.0, 1.0]);
        buffer.write(index, "stroke_width", &[0.0]);
    }
    let root = scene.add_mobject(Mobject::from_buffer(buffer)).unwrap();
    (scene, root)
}

fn program(builds: Rc<Cell<usize>>) -> Result<NativeSceneProgram, ServiceError> {
    let (scene, original) = initial();
    NativeSceneProgram::new(
        scene,
        vec![
            NativeSegment::Wait {
                duration: Some(0.5),
            },
            NativeSegment::defer(move |context| {
                assert_eq!(context.scene().time().frames(), 4);
                assert_eq!(context.scene().play_count(), 1);
                builds.set(builds.get() + 1);
                let stage = context.stage_mut();
                let buffer = stage.get(original).unwrap().buffer.deep_clone();
                let late = stage.add(Mobject::from_buffer(buffer));
                let entry = stage.get_mut(late).unwrap();
                let count = entry.buffer.len();
                entry
                    .buffer
                    .write_range("fill_rgba", 0, &[1.0, 0.2, 0.1, 1.0].repeat(count));
                stage.remove_from_scene(original);
                stage.add_to_scene(late)?;
                // This state exists only after the deferred constructor runs.
                // Recovery must rebuild it, not copy callback identity from bytes.
                let mut tick = 0_u32;
                stage.add_updater(
                    late,
                    move |stage, target| {
                        tick += 1;
                        stage.shift_many(&[target], [f64::from(tick) * 0.01, 0.0, 0.0]);
                    },
                    false,
                )?;
                Ok(vec![NativeSegment::Wait {
                    duration: Some(1.5),
                }])
            }),
        ],
        16,
    )
}

fn config(threads: usize) -> NativeWorkerConfig {
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
    };
    let mut config = NativeWorkerConfig::new(
        NAME,
        protocol_digest(b"deferred-build"),
        protocol_digest(b"deferred-source-and-inputs"),
        17,
        8,
        renderer,
    );
    config.checkpoint_frames = 1;
    config.replay = NativeReplayPolicy::ColdVerified;
    config
}

fn worker() -> NativeSceneWorker {
    NativeSceneWorker::new(config(1), || program(Rc::new(Cell::new(0)))).unwrap()
}

fn seek(worker: &mut NativeSceneWorker, frame: i64) -> FrameStream {
    let WorkerResponse::Frame(frame) = worker
        .handle(SupervisorRequest::Scrub {
            scene: NAME.into(),
            frame,
        })
        .unwrap()
    else {
        panic!("real PNG frame")
    };
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

fn commit(worker: &mut NativeSceneWorker, frame: i64) -> Entry {
    let WorkerResponse::JournalSegment { journal, .. } = worker
        .handle(SupervisorRequest::Play {
            scene: NAME.into(),
            command: studio_seek_command(NAME, frame).unwrap(),
        })
        .unwrap()
    else {
        panic!("journal")
    };
    Journal::from_bytes(&journal).unwrap().entries()[0].clone()
}

fn checkpoint(entry: &Entry, after_entry: u64) -> Checkpoint {
    Checkpoint {
        scene: NAME.into(),
        after_entry,
        state_hash: entry.state_hash,
        state: entry.checkpoint.clone().unwrap(),
    }
}

#[test]
fn later_construction_changes_real_pixels_once_and_is_absent_before_its_boundary() {
    let builds = Rc::new(Cell::new(0));
    let observed = Rc::clone(&builds);
    let mut worker =
        NativeSceneWorker::new(config(1), move || program(Rc::clone(&observed))).unwrap();
    let initial = seek(&mut worker, 0);
    let before = seek(&mut worker, 4);
    assert_eq!(png(&initial), png(&before));
    assert_eq!(builds.get(), 0);
    let original = worker.program().unwrap().preview().stage().roots()[0];
    let after = seek(&mut worker, 5);
    assert_ne!(png(&after), png(&before));
    assert_eq!(builds.get(), 1);
    assert_ne!(
        worker.program().unwrap().preview().stage().roots()[0],
        original
    );
    assert_eq!(seek(&mut worker, 5), after);
    worker
        .handle(SupervisorRequest::Inspect { scene: NAME.into() })
        .unwrap();
    assert_eq!(
        builds.get(),
        1,
        "rendering and inspection must not invoke constructors"
    );
    let later = seek(&mut worker, 8);
    assert_ne!(png(&later), png(&after));
    assert_eq!(builds.get(), 1);
}

#[test]
fn rewind_recreates_late_objects_and_mutable_callback_captures() {
    let mut worker = worker();
    let expected = seek(&mut worker, 10);
    seek(&mut worker, 0);
    assert_eq!(seek(&mut worker, 10), expected);
}

#[test]
fn restoring_before_and_after_construction_preserves_future_callback_motion() {
    for frame in [3, 6] {
        let mut original = worker();
        let entry = commit(&mut original, frame);
        let expected = seek(&mut original, 10);
        let mut recovered = worker();
        recovered
            .handle(SupervisorRequest::RestoreCheckpoint(checkpoint(&entry, 0)))
            .unwrap();
        assert_eq!(seek(&mut recovered, 10), expected, "checkpoint at {frame}");
    }
}

#[test]
fn journal_replay_crosses_the_dynamic_construction_boundary_with_exact_hashes() {
    let mut original = worker();
    let before = commit(&mut original, 3);
    let after = commit(&mut original, 8);
    let mut journal = Journal::new();
    journal.record(before.clone()).unwrap();
    journal.record(after.clone()).unwrap();
    let expected = seek(&mut original, 10);
    let mut recovered = worker();
    recovered
        .handle(SupervisorRequest::RestoreCheckpoint(checkpoint(&before, 0)))
        .unwrap();
    let response = recovered
        .handle(SupervisorRequest::ReplayJournal(JournalReplay {
            scene: NAME.into(),
            from_entry: 1,
            through_entry: 2,
            journal: journal.to_bytes().unwrap(),
        }))
        .unwrap();
    assert_eq!(
        response,
        WorkerResponse::ReplayComplete {
            from_entry: 1,
            state_hashes: vec![after.state_hash]
        }
    );
    assert_eq!(seek(&mut recovered, 10), expected);
}

#[test]
fn late_geometry_and_mutable_updaters_are_identical_across_render_teams() {
    let expected = seek(&mut worker(), 10);
    for threads in [4, 16] {
        let mut worker =
            NativeSceneWorker::new(config(threads), || program(Rc::new(Cell::new(0)))).unwrap();
        assert_eq!(seek(&mut worker, 10), expected);
    }
}

#[test]
fn failed_deferred_build_cannot_publish_a_successful_frame_or_checkpoint() {
    let mut worker = NativeSceneWorker::new(config(1), || {
        NativeSceneProgram::new(
            initial().0,
            vec![
                NativeSegment::Wait {
                    duration: Some(0.5),
                },
                NativeSegment::edit(|_| Err(SceneError::InvalidState("late construction refused"))),
            ],
            16,
        )
    })
    .unwrap();
    let before = seek(&mut worker, 3);
    let failure = worker
        .handle(SupervisorRequest::Scrub {
            scene: NAME.into(),
            frame: 5,
        })
        .unwrap_err();
    assert!(failure.message.contains("late construction refused"));
    assert!(worker.program().is_none());
    assert!(
        worker
            .handle(SupervisorRequest::Play {
                scene: NAME.into(),
                command: studio_seek_command(NAME, 5).unwrap(),
            })
            .is_err()
    );
    assert!(worker.journal_tail().is_empty());
    assert_eq!(worker.last_state_hash(), None);
    assert_eq!(
        seek(&mut worker, 3),
        before,
        "a fresh healthy prefix remains available"
    );
}

#[test]
fn a_deferred_panic_crosses_the_existing_worker_crash_boundary() {
    let mut worker = NativeSceneWorker::new(config(1), || {
        NativeSceneProgram::new(
            initial().0,
            vec![
                NativeSegment::Wait {
                    duration: Some(0.5),
                },
                NativeSegment::edit(|_| panic!("deferred native construction panic")),
            ],
            16,
        )
    })
    .unwrap();
    let limits = ProtocolLimits::default();
    let mut input = Vec::new();
    for (request_id, request) in [
        (
            1,
            SupervisorRequest::Hello {
                version: CURRENT_VERSION,
                supervisor_build: protocol_digest(b"parent"),
                max_frame_bytes: 1_000_000,
            },
        ),
        (
            2,
            SupervisorRequest::Scrub {
                scene: NAME.into(),
                frame: 5,
            },
        ),
    ] {
        write_request(
            &mut input,
            &RequestEnvelope {
                request_id,
                request,
            },
            limits,
        )
        .unwrap();
    }
    let mut output = Vec::new();
    let outcome = serve_worker(&mut worker, &mut Cursor::new(input), &mut output, limits).unwrap();
    assert!(matches!(
        outcome,
        fmn_studio::WorkerServeOutcome::Crashed(_)
    ));
    let mut output = Cursor::new(output);
    assert!(matches!(
        read_response(&mut output, limits).unwrap().response,
        WorkerResponse::Hello { .. }
    ));
    let crash = read_response(&mut output, limits).unwrap();
    assert_eq!(crash.request_id, 2);
    assert!(matches!(crash.response, WorkerResponse::Crash(_)));
}
