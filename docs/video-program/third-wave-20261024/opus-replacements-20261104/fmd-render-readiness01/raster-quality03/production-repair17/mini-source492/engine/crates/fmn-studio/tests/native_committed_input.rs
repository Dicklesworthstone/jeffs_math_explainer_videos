//! Durable native input is replayed by the actual editor, not a patch simulator.

use std::cell::Cell;
use std::rc::Rc;

use fmn_core::color::LinearRgba;
use fmn_mobject::{Mobject, RecordBuffer, RecordSchema};
use fmn_render::{
    EngineIdentity, FrameConfig, RetainedFrameRendererConfig, ScreenMap, Tiling, Viewport,
};
use fmn_scene::{Entry, EventPayload, Journal, Key, Modifiers, MouseButton, RuntimeConfig, Scene};
use fmn_studio::native::{
    NativeReplayPolicy, NativeSceneProgram, NativeSceneWorker, NativeSegment, NativeWorkerConfig,
};
use fmn_studio::protocol::{StudioInput, studio_input_command, studio_seek_command};
use fmn_studio::{
    Checkpoint, FrameStream, JournalReplay, ProtocolLimits, ServiceError, SupervisorRequest,
    WorkerErrorCode, WorkerResponse, WorkerService, protocol_digest,
};

const NAME: &str = "PersistentSquare";

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
    scene.add(&[root]).unwrap();
    let mut ticks = 0_u32;
    scene
        .stage_mut()
        .add_updater(
            root,
            move |stage, target| {
                ticks += 1;
                stage.shift_many(&[target], [f64::from(ticks) * 0.01, 0.0, 0.0]);
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
        protocol_digest(b"persistent-build"),
        protocol_digest(b"persistent-source"),
        31,
        30,
        renderer,
    );
    config.replay = NativeReplayPolicy::ColdVerified;
    config.checkpoint_frames = 1;
    config
}

fn worker() -> NativeSceneWorker {
    NativeSceneWorker::new(config(), program).unwrap()
}

#[test]
fn actual_editor_journal_reuses_typed_input_without_bypassing_effect_or_asset_checks() {
    let mut worker = worker();
    let mut journal = Journal::new();
    journal.record(commit_seek(&mut worker, 0)).unwrap();
    journal.record(select(&mut worker)).unwrap();
    journal.record(input(&mut worker, nudge())).unwrap();
    journal.record(commit_seek(&mut worker, 3)).unwrap();
    let journal = Journal::from_bytes(&journal.to_bytes().unwrap()).unwrap();
    let commands = journal
        .entries()
        .iter()
        .map(|entry| entry.command.clone())
        .collect::<Vec<_>>();
    let plan = fmn_scene::plan_replay(&journal, &commands, &|_| true);
    assert_eq!(plan.reuse, 4);
    assert!(plan.reason.is_none());
    assert!(
        journal
            .entries()
            .iter()
            .all(|entry| matches!(entry.effect, fmn_scene::EffectClass::Stateful(_)))
    );
    assert_eq!(
        journal.entries()[1].command.kind,
        fmn_scene::CommandKind::Input
    );
    assert_eq!(
        fmn_scene::plan_replay(&journal, &commands, &|_| false).reuse,
        0
    );
}

#[test]
fn custom_command_with_a_valid_input_label_cannot_execute_or_be_promoted() {
    let mut worker = worker();
    let before = seek(&mut worker, 0);
    let mut command = studio_input_command(
        NAME,
        &StudioInput {
            frame: 0,
            revision: 0,
            target: None,
            event: nudge(),
        },
    )
    .unwrap();
    command.kind = fmn_scene::CommandKind::Custom;
    assert!(
        worker
            .handle(SupervisorRequest::Play {
                scene: NAME.into(),
                command
            })
            .is_err()
    );
    assert_eq!(worker.journal_position(), 0);
    assert_eq!(worker.committed_input_count(), 0);
    assert_eq!(seek(&mut worker, 0), before);
}
fn seek(worker: &mut NativeSceneWorker, frame: i64) -> FrameStream {
    let WorkerResponse::Frame(frame) = worker
        .handle(SupervisorRequest::Scrub {
            scene: NAME.into(),
            frame,
        })
        .unwrap()
    else {
        panic!("native frame")
    };
    frame.validate(ProtocolLimits::default()).unwrap();
    frame
}
fn entry(response: WorkerResponse) -> Entry {
    let WorkerResponse::JournalSegment { journal, .. } = response else {
        panic!("journal segment")
    };
    Journal::from_bytes(&journal).unwrap().entries()[0].clone()
}
fn input(worker: &mut NativeSceneWorker, event: EventPayload) -> Entry {
    let frame = worker.program().unwrap().frame_index() as i64;
    let revision = worker.journal_position();
    entry(
        worker
            .commit_input(StudioInput {
                frame,
                revision,
                target: None,
                event,
            })
            .unwrap(),
    )
}
fn commit_seek(worker: &mut NativeSceneWorker, frame: i64) -> Entry {
    entry(
        worker
            .handle(SupervisorRequest::Play {
                scene: NAME.into(),
                command: studio_seek_command(NAME, frame).unwrap(),
            })
            .unwrap(),
    )
}
fn select(worker: &mut NativeSceneWorker) -> Entry {
    let stage = worker.program().unwrap().preview().stage();
    let point = stage.get_bounding_box(stage.roots()[0]).mid;
    input(
        worker,
        EventPayload::MousePress {
            point,
            button: MouseButton::Left,
            modifiers: Modifiers::PRIMARY,
        },
    )
}
fn key(key: Key, modifiers: Modifiers) -> EventPayload {
    EventPayload::KeyPress { key, modifiers }
}
fn nudge() -> EventPayload {
    key(Key::ArrowUp, Modifiers::SHIFT)
}
fn checkpoint(entry: &Entry, after_entry: u64) -> Checkpoint {
    Checkpoint {
        scene: NAME.into(),
        after_entry,
        state_hash: entry.state_hash,
        state: entry.checkpoint.clone().unwrap(),
    }
}
fn restore(worker: &mut NativeSceneWorker, entry: &Entry, after_entry: u64) {
    worker
        .handle(SupervisorRequest::RestoreCheckpoint(checkpoint(
            entry,
            after_entry,
        )))
        .unwrap();
}
fn y(worker: &NativeSceneWorker) -> f64 {
    let stage = worker.program().unwrap().preview().stage();
    stage.get_bounding_box(stage.roots()[0]).mid[1]
}
fn journal(entries: &[Entry]) -> Vec<u8> {
    let mut journal = Journal::new();
    for entry in entries {
        journal.record(entry.clone()).unwrap();
    }
    journal.to_bytes().unwrap()
}

#[test]
fn committed_selection_and_nudge_survive_scrub_without_repeating_current_frame_input() {
    let mut worker = worker();
    let original = seek(&mut worker, 3);
    select(&mut worker);
    input(&mut worker, nudge());
    let edited = seek(&mut worker, 3);
    assert_ne!(original.payload, edited.payload);
    assert_eq!(seek(&mut worker, 3), edited);
    assert!((y(&worker) - 0.5).abs() < 1.0e-6);
    let forward = seek(&mut worker, 6);
    seek(&mut worker, 2);
    assert!(y(&worker).abs() < 1.0e-6);
    assert_eq!(seek(&mut worker, 3), edited);
    assert_eq!(seek(&mut worker, 6), forward);
    assert_eq!(worker.committed_input_count(), 2);
}

#[test]
fn checkpoint_reconstructs_selection_history_and_future_mutable_updater_state() {
    let mut original = worker();
    seek(&mut original, 3);
    select(&mut original);
    let edit = input(&mut original, nudge());
    assert!(edit.checkpoint.as_ref().unwrap().starts_with(b"FMEI"));
    let expected = seek(&mut original, 6);
    let mut recovered = worker();
    restore(&mut recovered, &edit, 1);
    assert_eq!(recovered.journal_position(), 2);
    assert_eq!(recovered.committed_input_count(), 2);
    assert_eq!(recovered.program().unwrap().preview().selection().len(), 1);
    assert_eq!(seek(&mut recovered, 6), expected);
    // Undo is itself native input, with the reconstructed snapshot history.
    seek(&mut recovered, 3);
    input(&mut recovered, key(Key::Character('z'), Modifiers::PRIMARY));
    assert!(y(&recovered).abs() < 1.0e-6);
    input(
        &mut recovered,
        key(Key::Character('z'), Modifiers::PRIMARY | Modifiers::SHIFT),
    );
    assert!((y(&recovered) - 0.5).abs() < 1.0e-6);
}

#[test]
fn clipboard_templates_and_paste_handles_are_recreated_before_later_input() {
    let mut original = worker();
    select(&mut original);
    let copy = input(&mut original, key(Key::Character('c'), Modifiers::PRIMARY));
    let mut recovered = worker();
    restore(&mut recovered, &copy, 1);
    input(&mut recovered, key(Key::Character('v'), Modifiers::PRIMARY));
    assert_eq!(
        recovered.program().unwrap().preview().stage().roots().len(),
        2
    );
    let pasted = input(&mut recovered, nudge());
    let expected = seek(&mut recovered, 0);
    let mut again = worker();
    restore(&mut again, &pasted, 3);
    assert_eq!(seek(&mut again, 0), expected);
    assert_eq!(again.program().unwrap().preview().selection().len(), 1);
}

#[test]
fn backward_checkpoint_keeps_future_edits_without_applying_them_early() {
    let mut original = worker();
    seek(&mut original, 5);
    select(&mut original);
    input(&mut original, nudge());
    let expected = seek(&mut original, 7);
    let past = commit_seek(&mut original, 1);
    assert!(y(&original).abs() < 1.0e-6);
    let mut recovered = worker();
    restore(&mut recovered, &past, 2);
    assert!(y(&recovered).abs() < 1.0e-6);
    assert_eq!(recovered.committed_input_count(), 2);
    assert_eq!(seek(&mut recovered, 7), expected);
}

#[test]
fn editing_the_past_discards_only_the_later_input_branch() {
    let mut worker = worker();
    select(&mut worker); // frame zero, remains when future inputs are replaced
    seek(&mut worker, 5);
    input(&mut worker, nudge());
    seek(&mut worker, 2);
    input(&mut worker, key(Key::ArrowDown, Modifiers::SHIFT));
    assert_eq!(worker.committed_input_count(), 2);
    seek(&mut worker, 7);
    assert!((y(&worker) + 0.5).abs() < 1.0e-6);
}

#[test]
fn mixed_seek_and_input_journal_replays_the_complete_track_and_exact_hashes() {
    let mut original = worker();
    let entries = vec![
        commit_seek(&mut original, 3),
        select(&mut original),
        input(&mut original, nudge()),
        commit_seek(&mut original, 7),
    ];
    let expected = seek(&mut original, 7);
    let mut recovered = worker();
    let response = recovered
        .handle(SupervisorRequest::ReplayJournal(JournalReplay {
            scene: NAME.into(),
            from_entry: 0,
            through_entry: 4,
            journal: journal(&entries),
        }))
        .unwrap();
    assert_eq!(
        response,
        WorkerResponse::ReplayComplete {
            from_entry: 0,
            state_hashes: entries.iter().map(|entry| entry.state_hash).collect(),
        }
    );
    assert_eq!(seek(&mut recovered, 7), expected);
    assert_eq!(recovered.journal_position(), 4);
    assert_eq!(recovered.committed_input_count(), 2);
}

#[test]
fn replay_suffix_after_an_edited_checkpoint_preserves_native_undo_redo() {
    let mut original = worker();
    let entries = vec![
        select(&mut original),
        input(&mut original, nudge()),
        input(&mut original, key(Key::Character('z'), Modifiers::PRIMARY)),
        input(
            &mut original,
            key(Key::Character('z'), Modifiers::PRIMARY | Modifiers::SHIFT),
        ),
    ];
    let mut recovered = worker();
    restore(&mut recovered, &entries[1], 1);
    recovered
        .handle(SupervisorRequest::ReplayJournal(JournalReplay {
            scene: NAME.into(),
            from_entry: 2,
            through_entry: 4,
            journal: journal(&entries),
        }))
        .unwrap();
    assert_eq!(seek(&mut recovered, 0), seek(&mut original, 0));
}

#[test]
fn stale_revisions_bad_targets_and_out_of_range_frames_refuse_before_factory_execution() {
    let calls = Rc::new(Cell::new(0));
    let observed = Rc::clone(&calls);
    let mut worker = NativeSceneWorker::new(config(), move || {
        observed.set(observed.get() + 1);
        program()
    })
    .unwrap();
    select(&mut worker);
    let count = calls.get();
    for input in [
        StudioInput {
            frame: 0,
            revision: 0,
            target: None,
            event: nudge(),
        },
        StudioInput {
            frame: 0,
            revision: 1,
            target: Some(0),
            event: nudge(),
        },
        StudioInput {
            frame: 31,
            revision: 1,
            target: None,
            event: nudge(),
        },
    ] {
        assert!(worker.commit_input(input).is_err());
    }
    assert_eq!(calls.get(), count);
    assert_eq!(worker.journal_position(), 1);
    assert_eq!(worker.committed_input_count(), 1);
}

#[test]
fn edited_checkpoint_binds_source_closure_and_exact_journal_position() {
    let mut original = worker();
    let entry = select(&mut original);
    for wrong_position in [false, true] {
        let calls = Rc::new(Cell::new(0));
        let observed = Rc::clone(&calls);
        let mut policy = config();
        if !wrong_position {
            policy.source_digest = protocol_digest(b"changed source");
        }
        let mut recovered = NativeSceneWorker::new(policy, move || {
            observed.set(observed.get() + 1);
            program()
        })
        .unwrap();
        let count = calls.get();
        let result = recovered.handle(SupervisorRequest::RestoreCheckpoint(checkpoint(
            &entry,
            u64::from(wrong_position),
        )));
        assert!(result.is_err());
        assert_eq!(calls.get(), count);
        assert_eq!(recovered.journal_position(), 0);
    }
}

#[test]
fn input_count_limit_refuses_before_execution_and_allows_replacing_a_future_branch() {
    let mut policy = config();
    policy.max_recorded_inputs = 1;
    let mut worker = NativeSceneWorker::new(policy, program).unwrap();
    seek(&mut worker, 5);
    select(&mut worker);
    assert!(
        worker
            .commit_input(StudioInput {
                frame: 5,
                revision: 1,
                target: None,
                event: nudge()
            })
            .is_err()
    );
    assert_eq!(worker.journal_position(), 1);
    seek(&mut worker, 0);
    select(&mut worker);
    assert_eq!(worker.committed_input_count(), 1);
    assert_eq!(worker.journal_position(), 2);
}

#[test]
fn zero_frame_input_replay_has_an_aggregate_work_budget_before_any_factory_runs() {
    let mut original = worker();
    let entries = vec![select(&mut original), input(&mut original, nudge())];
    let calls = Rc::new(Cell::new(0));
    let observed = Rc::clone(&calls);
    let mut policy = config();
    policy.max_replay_inputs = 2; // two entries require 1 + 2 dispatches
    let mut recovered = NativeSceneWorker::new(policy, move || {
        observed.set(observed.get() + 1);
        program()
    })
    .unwrap();
    let count = calls.get();
    assert!(
        recovered
            .handle(SupervisorRequest::ReplayJournal(JournalReplay {
                scene: NAME.into(),
                from_entry: 0,
                through_entry: 2,
                journal: journal(&entries),
            }))
            .is_err()
    );
    assert_eq!(calls.get(), count);
    assert_eq!(recovered.committed_input_count(), 0);
}

#[test]
fn divergent_late_replay_installs_neither_the_prefix_nor_its_input_track() {
    let mut original = worker();
    let mut entries = vec![select(&mut original), input(&mut original, nudge())];
    entries[1].state_hash = protocol_digest(b"wrong");
    entries[1].checkpoint = None;
    let mut recovered = worker();
    let before = seek(&mut recovered, 1);
    assert!(
        recovered
            .handle(SupervisorRequest::ReplayJournal(JournalReplay {
                scene: NAME.into(),
                from_entry: 0,
                through_entry: 2,
                journal: journal(&entries),
            }))
            .is_err()
    );
    assert_eq!(recovered.committed_input_count(), 0);
    assert_eq!(recovered.journal_position(), 0);
    assert_eq!(seek(&mut recovered, 1), before);
}

#[test]
fn unknown_callbacks_cannot_be_mislabeled_as_durable_edits() {
    let mut policy = config();
    policy.replay = NativeReplayPolicy::Disabled;
    let mut worker = NativeSceneWorker::new(policy, program).unwrap();
    assert_eq!(
        worker
            .commit_input(StudioInput {
                frame: 0,
                revision: 0,
                target: None,
                event: nudge()
            })
            .unwrap_err()
            .code,
        WorkerErrorCode::InvalidRequest
    );
    assert_eq!(worker.committed_input_count(), 0);
}

#[test]
fn transient_input_is_discarded_but_the_committed_track_survives() {
    let mut worker = worker();
    select(&mut worker);
    input(&mut worker, nudge());
    let canonical = seek(&mut worker, 0);
    worker
        .handle(SupervisorRequest::Event {
            scene: NAME.into(),
            event: nudge(),
        })
        .unwrap();
    assert!((y(&worker) - 1.0).abs() < 1.0e-6);
    assert_eq!(seek(&mut worker, 0), canonical);
    assert_eq!(worker.journal_position(), 2);
}

#[test]
fn inspector_exposes_the_revision_and_input_capability_with_a_bounded_additive_field() {
    let mut worker = worker();
    select(&mut worker);
    let WorkerResponse::StudioData { bytes, .. } = worker
        .handle(SupervisorRequest::Inspect { scene: NAME.into() })
        .unwrap()
    else {
        panic!("inspection")
    };
    let text = String::from_utf8(bytes).unwrap();
    assert!(
        text.contains("\"native_edit\":{\"revision\":1,\"committed_inputs\":1,\"enabled\":true")
    );
}

#[test]
fn committed_input_command_tampering_does_not_advance_the_journal() {
    let mut worker = worker();
    let mut command = studio_input_command(
        NAME,
        &StudioInput {
            frame: 0,
            revision: 0,
            target: None,
            event: nudge(),
        },
    )
    .unwrap();
    command.identity = protocol_digest(b"wrong command");
    assert!(
        worker
            .handle(SupervisorRequest::Play {
                scene: NAME.into(),
                command
            })
            .is_err()
    );
    assert_eq!(worker.journal_position(), 0);
    assert_eq!(worker.committed_input_count(), 0);
}

fn staged_worker() -> NativeSceneWorker {
    let mut policy = config();
    policy.stage_input_events = true;
    NativeSceneWorker::new(policy, program).unwrap()
}
fn stage_event(worker: &mut NativeSceneWorker, event: EventPayload) -> FrameStream {
    let WorkerResponse::Frame(frame) = worker
        .handle(SupervisorRequest::Event {
            scene: NAME.into(),
            event,
        })
        .unwrap()
    else {
        panic!("staged input frame")
    };
    frame
}
fn stage_select(worker: &mut NativeSceneWorker) {
    let stage = worker.program().unwrap().preview().stage();
    let point = stage.get_bounding_box(stage.roots()[0]).mid;
    stage_event(
        worker,
        EventPayload::MousePress {
            point,
            button: MouseButton::Left,
            modifiers: Modifiers::PRIMARY,
        },
    );
}
fn save(worker: &mut NativeSceneWorker) -> Vec<Entry> {
    let frame = worker.program().unwrap().frame_index() as i64;
    let WorkerResponse::JournalSegment { journal, .. } = worker
        .handle(SupervisorRequest::Play {
            scene: NAME.into(),
            command: studio_seek_command(NAME, frame).unwrap(),
        })
        .unwrap()
    else {
        panic!("saved batch")
    };
    Journal::from_bytes(&journal).unwrap().entries().to_vec()
}

#[test]
fn preview_save_publishes_one_complete_batch_and_only_its_final_checkpoint() {
    let mut worker = staged_worker();
    seek(&mut worker, 3);
    stage_select(&mut worker);
    let edited = stage_event(&mut worker, nudge());
    assert_eq!(worker.journal_position(), 0);
    assert_eq!(worker.pending_input_count(), 2);
    assert_eq!(worker.committed_input_count(), 0);
    assert!(worker.journal_tail().is_empty());
    let entries = save(&mut worker);
    assert_eq!(entries.len(), 3);
    assert!(entries[..2].iter().all(|entry| entry.checkpoint.is_none()));
    assert!(
        entries[..2]
            .iter()
            .all(|entry| entry.command.kind == fmn_scene::CommandKind::Input
                && matches!(entry.effect, fmn_scene::EffectClass::Stateful(_))
                && !entry.is_replay_barrier())
    );
    assert!(entries[2].checkpoint.is_some());
    assert_eq!(worker.journal_position(), 3);
    assert_eq!(worker.pending_input_count(), 0);
    assert_eq!(worker.committed_input_count(), 2);
    assert_eq!(seek(&mut worker, 3), edited);
    assert_eq!(worker.last_state_hash(), Some(entries[2].state_hash));
}

#[test]
fn saved_batch_replays_each_intermediate_hash_and_restores_native_history() {
    let mut original = staged_worker();
    seek(&mut original, 3);
    stage_select(&mut original);
    stage_event(&mut original, nudge());
    let entries = save(&mut original);
    let expected = seek(&mut original, 6);
    let mut replayed = staged_worker();
    let response = replayed
        .handle(SupervisorRequest::ReplayJournal(JournalReplay {
            scene: NAME.into(),
            from_entry: 0,
            through_entry: 3,
            journal: journal(&entries),
        }))
        .unwrap();
    assert_eq!(
        response,
        WorkerResponse::ReplayComplete {
            from_entry: 0,
            state_hashes: entries.iter().map(|entry| entry.state_hash).collect(),
        }
    );
    assert_eq!(seek(&mut replayed, 6), expected);
    let mut restored = staged_worker();
    restore(&mut restored, &entries[2], 2);
    assert_eq!(seek(&mut restored, 6), expected);
    seek(&mut restored, 3);
    stage_event(&mut restored, key(Key::Character('z'), Modifiers::PRIMARY));
    save(&mut restored);
    assert!(y(&restored).abs() < 1.0e-6);
    seek(&mut restored, 7);
    assert!(y(&restored).abs() < 1.0e-6);
}

#[test]
fn noncommitting_scrub_discards_preview_input_without_erasing_saved_input() {
    let mut worker = staged_worker();
    stage_select(&mut worker);
    stage_event(&mut worker, nudge());
    save(&mut worker);
    let canonical = seek(&mut worker, 0);
    stage_event(&mut worker, nudge());
    assert_eq!(worker.pending_input_count(), 1);
    assert_eq!(seek(&mut worker, 0), canonical);
    assert_eq!(worker.pending_input_count(), 0);
    assert_eq!(worker.committed_input_count(), 2);
    assert_eq!(worker.journal_position(), 3);
}

#[test]
fn navigating_to_a_different_committed_frame_does_not_silently_save_the_preview() {
    let mut worker = staged_worker();
    stage_select(&mut worker);
    stage_event(&mut worker, nudge());
    commit_seek(&mut worker, 4);
    assert_eq!(worker.pending_input_count(), 0);
    assert_eq!(worker.committed_input_count(), 0);
    assert_eq!(worker.journal_position(), 1);
    assert!(y(&worker).abs() < 1.0e-6);
}

#[test]
fn failed_save_keeps_pending_edits_and_committed_recovery_authority() {
    let mut policy = config();
    policy.stage_input_events = true;
    policy.limits.max_checkpoint_bytes = 8;
    let mut worker = NativeSceneWorker::new(policy, program).unwrap();
    stage_select(&mut worker);
    stage_event(&mut worker, nudge());
    let root_count = worker.program().unwrap().preview().stage().roots().len();
    let result = worker.handle(SupervisorRequest::Play {
        scene: NAME.into(),
        command: studio_seek_command(NAME, 0).unwrap(),
    });
    assert!(result.is_err());
    assert_eq!(worker.pending_input_count(), 2);
    assert_eq!(worker.committed_input_count(), 0);
    assert_eq!(worker.journal_position(), 0);
    assert_eq!(worker.last_state_hash(), None);
    assert_eq!(
        worker.program().unwrap().preview().stage().roots().len(),
        root_count
    );
    assert!((y(&worker) - 0.5).abs() < 1.0e-6);
    seek(&mut worker, 0);
    assert_eq!(worker.pending_input_count(), 0);
}

#[test]
fn preview_staging_is_explicit_and_unknown_callbacks_are_never_promoted_to_durable() {
    for replay in [
        NativeReplayPolicy::Disabled,
        NativeReplayPolicy::ColdVerified,
    ] {
        let mut policy = config();
        policy.replay = replay;
        policy.stage_input_events = replay == NativeReplayPolicy::Disabled;
        let mut worker = NativeSceneWorker::new(policy, program).unwrap();
        stage_select(&mut worker);
        stage_event(&mut worker, nudge());
        assert_eq!(worker.pending_input_count(), 0);
        assert_eq!(save(&mut worker).len(), 1);
        assert!(y(&worker).abs() < 1.0e-6);
        assert_eq!(worker.committed_input_count(), 0);
    }
}

#[test]
fn a_full_pending_queue_refuses_before_dispatch_and_can_be_discarded() {
    let mut policy = config();
    policy.stage_input_events = true;
    policy.max_recorded_inputs = 1;
    let mut worker = NativeSceneWorker::new(policy, program).unwrap();
    stage_select(&mut worker);
    let before = y(&worker);
    assert!(
        worker
            .handle(SupervisorRequest::Event {
                scene: NAME.into(),
                event: nudge()
            })
            .is_err()
    );
    assert_eq!(worker.pending_input_count(), 1);
    assert_eq!(y(&worker), before);
    assert!(worker.program().is_some());
    seek(&mut worker, 0);
    assert_eq!(worker.pending_input_count(), 0);
}

#[test]
fn saving_a_grab_keeps_one_native_undo_transition_across_restart() {
    let mut worker = staged_worker();
    stage_select(&mut worker);
    stage_event(&mut worker, key(Key::Character('g'), Modifiers::NONE));
    for x in [0.5, 1.0, 1.5] {
        stage_event(
            &mut worker,
            EventPayload::MouseMotion {
                point: [x, 0.0, 0.0],
                delta: [0.5, 0.0, 0.0],
                modifiers: Modifiers::NONE,
            },
        );
    }
    stage_event(
        &mut worker,
        EventPayload::KeyRelease {
            key: Key::Character('g'),
            modifiers: Modifiers::NONE,
        },
    );
    let entries = save(&mut worker);
    let mut recovered = staged_worker();
    restore(
        &mut recovered,
        entries.last().unwrap(),
        entries.len() as u64 - 1,
    );
    stage_event(&mut recovered, key(Key::Character('z'), Modifiers::PRIMARY));
    save(&mut recovered);
    let stage = recovered.program().unwrap().preview().stage();
    assert!(stage.get_bounding_box(stage.roots()[0]).mid[0].abs() < 1.0e-6);
}
