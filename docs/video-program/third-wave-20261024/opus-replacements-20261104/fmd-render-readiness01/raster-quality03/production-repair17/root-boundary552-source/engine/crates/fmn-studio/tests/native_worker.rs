//! Real native callbacks, CPU pixels, canonical protocol and recovery together.

use std::cell::Cell;
use std::io::Cursor;
use std::rc::Rc;

use fmn_core::color::LinearRgba;
use fmn_mobject::{Mobject, RecordBuffer, RecordSchema};
use fmn_render::{
    EngineIdentity, FrameConfig, RetainedFrameRendererConfig, ScreenMap, Tiling, Viewport,
};
use fmn_scene::{
    EffectClass, Entry, EventPayload, Journal, Key, Modifiers, MouseButton, RuntimeConfig, Scene,
};
use fmn_studio::native::{
    NativeReplayPolicy, NativeSceneProgram, NativeSceneWorker, NativeSegment, NativeWorkerConfig,
};
use fmn_studio::protocol::studio_seek_command;
use fmn_studio::{
    CURRENT_VERSION, Checkpoint, DebugLayerSet, FramePayload, FrameStream, JournalReplay,
    ProtocolLimits, RequestEnvelope, ServiceError, SupervisorRequest, WorkerErrorCode,
    WorkerResponse, WorkerServeOutcome, WorkerService, protocol_digest, read_response,
    serve_worker, write_request,
};

const NAME: &str = "LiveSquare";

fn program() -> Result<NativeSceneProgram, ServiceError> {
    let mut scene = Scene::new(RuntimeConfig::default(), 71).unwrap();
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
        buffer.write(index, "fill_rgba", &[1.0, 0.25, 0.0, 1.0]);
        buffer.write(index, "stroke_width", &[0.0]);
    }
    let root = scene.stage_mut().add(Mobject::from_buffer(buffer));
    scene.stage_mut().add_to_scene(root).unwrap();
    // Fresh, *mutable* closure state per factory invocation. Snapshot cloning
    // would incorrectly share this counter across replay attempts.
    let mut step = 0_u32;
    scene
        .stage_mut()
        .add_updater(
            root,
            move |stage, target| {
                step += 1;
                stage.shift_many(&[target], [f64::from(step) * 0.03, 0.0, 0.0]);
            },
            false,
        )
        .unwrap();
    NativeSceneProgram::new(
        scene,
        vec![NativeSegment::Wait {
            duration: Some(0.4),
        }],
        12,
    )
}

fn config() -> NativeWorkerConfig {
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
        threads: 1,
    };
    let mut config = NativeWorkerConfig::new(
        NAME,
        protocol_digest(b"test-build"),
        protocol_digest(b"independent-square-code-and-inputs-v1"),
        13,
        30,
        renderer,
    );
    config.checkpoint_frames = 2;
    config.replay = NativeReplayPolicy::ColdVerified;
    config
}

fn worker() -> NativeSceneWorker {
    NativeSceneWorker::new(config(), program).unwrap()
}

fn seek(worker: &mut NativeSceneWorker, frame: i64) -> FrameStream {
    let response = worker
        .handle(SupervisorRequest::Scrub {
            scene: NAME.into(),
            frame,
        })
        .unwrap();
    let WorkerResponse::Frame(frame) = response else {
        panic!("expected pixels")
    };
    frame.validate(ProtocolLimits::default()).unwrap();
    frame
}

fn png(frame: &FrameStream) -> &[u8] {
    let FramePayload::Pipe { bytes, .. } = &frame.payload else {
        panic!("expected inline PNG")
    };
    bytes
}

fn commit(worker: &mut NativeSceneWorker, frame: i64) -> Entry {
    let response = worker
        .handle(SupervisorRequest::Play {
            scene: NAME.into(),
            command: studio_seek_command(NAME, frame).unwrap(),
        })
        .unwrap();
    let WorkerResponse::JournalSegment { journal, .. } = response else {
        panic!("expected journal")
    };
    Journal::from_bytes(&journal).unwrap().entries()[0].clone()
}

fn checkpoint(entry: &Entry, after_entry: u64) -> Checkpoint {
    Checkpoint {
        scene: NAME.into(),
        after_entry,
        state_hash: entry.state_hash,
        state: entry.checkpoint.clone().expect("checkpoint attached"),
    }
}

fn event(worker: &mut NativeSceneWorker, event: EventPayload) -> FrameStream {
    let WorkerResponse::Frame(frame) = worker
        .handle(SupervisorRequest::Event {
            scene: NAME.into(),
            event,
        })
        .unwrap()
    else {
        panic!("input must return real rendered pixels")
    };
    frame
}

#[test]
fn clean_forward_preview_continues_callbacks_and_backward_scrub_recreates_them() {
    let constructions = Rc::new(Cell::new(0));
    let observed = Rc::clone(&constructions);
    let mut worker = NativeSceneWorker::new(config(), move || {
        observed.set(observed.get() + 1);
        program()
    })
    .unwrap();
    let initial = seek(&mut worker, 0);
    let moved = seek(&mut worker, 3);
    assert_eq!(
        constructions.get(),
        1,
        "forward preview must keep the actual callback owner"
    );
    assert_ne!(png(&initial), png(&moved), "real CPU pixels must move");
    assert_eq!(moved.frame_index, 3);
    let repeated = seek(&mut worker, 0);
    assert_eq!(constructions.get(), 2);
    assert_eq!(png(&initial), png(&repeated));
    assert_eq!(png(&moved), png(&seek(&mut worker, 3)));
    assert!(!moved.render_backends.is_empty());
}

#[test]
fn live_input_undo_and_inspection_share_one_native_arena_and_scrub_resets_edits() {
    let mut worker = worker();
    let before = seek(&mut worker, 3);
    let stage = worker.program().unwrap().preview().stage();
    let center = stage.get_bounding_box(stage.roots()[0]).mid;
    event(
        &mut worker,
        EventPayload::MousePress {
            point: center,
            button: MouseButton::Left,
            modifiers: Modifiers::PRIMARY,
        },
    );
    let moved = event(
        &mut worker,
        EventPayload::KeyPress {
            key: Key::ArrowUp,
            modifiers: Modifiers::SHIFT,
        },
    );
    assert_ne!(png(&before), png(&moved));
    assert_eq!(moved.frame_index, before.frame_index);
    let inspection = worker
        .handle(SupervisorRequest::Inspect { scene: NAME.into() })
        .unwrap();
    let WorkerResponse::StudioData { bytes, digest, .. } = inspection else {
        panic!("inspection")
    };
    assert_eq!(protocol_digest(&bytes), digest);
    let json = String::from_utf8(bytes).unwrap();
    assert!(json.contains("\"input_events\":true"));
    assert!(json.contains("\"frame_index\":3"));
    let undone = event(
        &mut worker,
        EventPayload::KeyPress {
            key: Key::Character('z'),
            modifiers: Modifiers::PRIMARY,
        },
    );
    assert_eq!(png(&undone), png(&before));
    event(
        &mut worker,
        EventPayload::KeyPress {
            key: Key::ArrowUp,
            modifiers: Modifiers::SHIFT,
        },
    );
    assert_eq!(png(&seek(&mut worker, 3)), png(&before));
    assert!(worker.program().unwrap().preview().selection().is_empty());
    assert!(
        worker.journal_tail().is_empty(),
        "transient edits are not canonical commands"
    );
    assert_eq!(worker.last_state_hash(), None);
}

#[test]
fn checkpoint_cold_execution_restores_real_callback_state_for_continuation() {
    let mut original = worker();
    let entry = commit(&mut original, 3);
    let expected = seek(&mut original, 6);
    let builds = Rc::new(Cell::new(0));
    let observed = Rc::clone(&builds);
    let mut recovered = NativeSceneWorker::new(config(), move || {
        observed.set(observed.get() + 1);
        program()
    })
    .unwrap();
    let ack = recovered
        .handle(SupervisorRequest::RestoreCheckpoint(checkpoint(&entry, 0)))
        .unwrap();
    assert_eq!(
        ack,
        WorkerResponse::Ack {
            state_hash: Some(entry.state_hash),
            journal_len: 1
        }
    );
    let count = builds.get();
    assert_eq!(png(&seek(&mut recovered, 6)), png(&expected));
    assert_eq!(
        builds.get(),
        count,
        "resuming the restored cursor must not rebuild callbacks again"
    );
}

#[test]
fn replay_suffix_executes_and_checks_every_actual_state_hash() {
    let mut original = worker();
    let first = commit(&mut original, 3);
    let second = commit(&mut original, 6);
    let mut journal = Journal::new();
    journal.record(first.clone()).unwrap();
    journal.record(second.clone()).unwrap();
    let mut recovered = worker();
    recovered
        .handle(SupervisorRequest::RestoreCheckpoint(checkpoint(&first, 0)))
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
            state_hashes: vec![second.state_hash]
        }
    );
    assert_eq!(recovered.last_state_hash(), Some(second.state_hash));
    assert_eq!(recovered.program().unwrap().frame_index(), 6);
}

#[test]
fn divergent_replay_installs_no_prefix_and_leaves_the_existing_owner_intact() {
    let mut original = worker();
    let first = commit(&mut original, 3);
    let mut second = commit(&mut original, 6);
    second.state_hash = protocol_digest(b"wrong state");
    second.checkpoint = None;
    let mut journal = Journal::new();
    journal.record(first).unwrap();
    journal.record(second).unwrap();
    let mut recovered = worker();
    let before = seek(&mut recovered, 2);
    let error = recovered
        .handle(SupervisorRequest::ReplayJournal(JournalReplay {
            scene: NAME.into(),
            from_entry: 0,
            through_entry: 2,
            journal: journal.to_bytes().unwrap(),
        }))
        .unwrap_err();
    assert_eq!(error.code, WorkerErrorCode::ReplayFailed);
    assert_eq!(recovered.program().unwrap().frame_index(), 2);
    assert!(recovered.journal_tail().is_empty());
    assert_eq!(png(&seek(&mut recovered, 2)), png(&before));
}

#[test]
fn changed_source_closure_and_aggregate_work_budget_refuse_before_factory_execution() {
    let mut original = worker();
    let first = commit(&mut original, 3);
    let second = commit(&mut original, 3);
    let mut journal = Journal::new();
    journal.record(first).unwrap();
    journal.record(second).unwrap();
    for changed_source in [false, true] {
        let builds = Rc::new(Cell::new(0));
        let observed = Rc::clone(&builds);
        let mut policy = config();
        if changed_source {
            policy.source_digest = protocol_digest(b"different source");
        } else {
            policy.max_replay_frames = 5;
        }
        let mut recovered = NativeSceneWorker::new(policy, move || {
            observed.set(observed.get() + 1);
            program()
        })
        .unwrap();
        let count = builds.get();
        let result = recovered.handle(SupervisorRequest::ReplayJournal(JournalReplay {
            scene: NAME.into(),
            from_entry: 0,
            through_entry: 2,
            journal: journal.to_bytes().unwrap(),
        }));
        assert!(result.is_err());
        assert_eq!(builds.get(), count);
        assert_eq!(recovered.program().unwrap().frame_index(), 0);
    }
}

#[test]
fn unclassified_callbacks_are_opaque_and_cannot_be_restored_or_replayed() {
    let mut policy = config();
    policy.replay = NativeReplayPolicy::Disabled;
    let mut worker = NativeSceneWorker::new(policy, program).unwrap();
    let entry = commit(&mut worker, 3);
    assert_eq!(entry.effect, EffectClass::Opaque);
    assert_eq!(
        worker
            .handle(SupervisorRequest::RestoreCheckpoint(checkpoint(&entry, 0)))
            .unwrap_err()
            .code,
        WorkerErrorCode::CheckpointRejected
    );
    let mut journal = Journal::new();
    journal.record(entry).unwrap();
    assert_eq!(
        worker
            .handle(SupervisorRequest::ReplayJournal(JournalReplay {
                scene: NAME.into(),
                from_entry: 1,
                through_entry: 1,
                journal: journal.to_bytes().unwrap(),
            }))
            .unwrap_err()
            .code,
        WorkerErrorCode::ReplayFailed
    );
}

#[test]
fn bad_requests_and_forged_checkpoints_do_not_move_the_native_owner() {
    let mut worker = worker();
    let entry = commit(&mut worker, 3);
    let mut forged = checkpoint(&entry, 0);
    forged.state_hash = protocol_digest(b"forged");
    assert!(
        worker
            .handle(SupervisorRequest::RestoreCheckpoint(forged))
            .is_err()
    );
    assert!(
        worker
            .handle(SupervisorRequest::Scrub {
                scene: NAME.into(),
                frame: -1
            })
            .is_err()
    );
    assert!(
        worker
            .handle(SupervisorRequest::Scrub {
                scene: NAME.into(),
                frame: 13
            })
            .is_err()
    );
    assert_eq!(
        worker
            .handle(SupervisorRequest::Inspect {
                scene: "Other".into()
            })
            .unwrap_err()
            .code,
        WorkerErrorCode::SceneNotFound
    );
    assert_eq!(worker.program().unwrap().frame_index(), 3);
    assert_eq!(worker.last_state_hash(), Some(entry.state_hash));
}

#[test]
fn inspector_and_overlay_respect_wire_limits_without_claiming_truncated_success() {
    let mut policy = config();
    policy.limits.max_studio_data_bytes = 16;
    let mut worker = NativeSceneWorker::new(policy, program).unwrap();
    assert!(
        worker
            .handle(SupervisorRequest::Inspect { scene: NAME.into() })
            .is_err()
    );
    assert!(
        worker
            .handle(SupervisorRequest::Overlay {
                scene: NAME.into(),
                layers: DebugLayerSet::ALL
            })
            .is_err()
    );
    assert_eq!(worker.program().unwrap().frame_index(), 0);
}

#[test]
fn handshake_and_requests_round_trip_through_the_actual_worker_pipe() {
    let limits = ProtocolLimits::default();
    let requests = [
        SupervisorRequest::Hello {
            version: CURRENT_VERSION,
            supervisor_build: protocol_digest(b"supervisor"),
            max_frame_bytes: 1_000_000,
        },
        SupervisorRequest::EnumerateScenes,
        SupervisorRequest::Scrub {
            scene: NAME.into(),
            frame: 3,
        },
        SupervisorRequest::Event {
            scene: NAME.into(),
            event: EventPayload::KeyPress {
                key: Key::Character('a'),
                modifiers: Modifiers::PRIMARY,
            },
        },
        SupervisorRequest::Inspect { scene: NAME.into() },
        SupervisorRequest::Shutdown,
    ];
    let mut input = Vec::new();
    for (index, request) in requests.into_iter().enumerate() {
        write_request(
            &mut input,
            &RequestEnvelope {
                request_id: u64::try_from(index).unwrap() + 1,
                request,
            },
            limits,
        )
        .unwrap();
    }
    let mut output = Vec::new();
    let mut worker = worker();
    let outcome = serve_worker(&mut worker, &mut Cursor::new(input), &mut output, limits).unwrap();
    assert!(matches!(outcome, WorkerServeOutcome::Shutdown));
    let mut output = Cursor::new(output);
    for index in 1..=6 {
        let envelope = read_response(&mut output, limits).unwrap();
        assert_eq!(envelope.request_id, index);
        match index {
            1 => assert!(matches!(envelope.response, WorkerResponse::Hello { .. })),
            2 => assert_eq!(envelope.response, WorkerResponse::Scenes(vec![NAME.into()])),
            3 | 4 => assert!(matches!(envelope.response, WorkerResponse::Frame(_))),
            5 => assert!(matches!(
                envelope.response,
                WorkerResponse::StudioData { .. }
            )),
            6 => assert_eq!(envelope.response, WorkerResponse::Bye),
            _ => unreachable!(),
        }
    }
}
