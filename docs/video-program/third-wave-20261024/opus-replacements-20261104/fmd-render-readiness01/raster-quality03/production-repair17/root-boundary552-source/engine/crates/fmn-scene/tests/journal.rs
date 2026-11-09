//! fm-y7u acceptance: record/replay equivalence (bit-identical state),
//! invalidation at a changed callback, opaque-barrier conservatism,
//! purity-evidence round-trip with the W4 classifier, the pipeline-
//! barrier corpus assertion, and the repro-bundle end-to-end test.

use std::collections::BTreeMap;

use fmn_anim::purity::{classify_play, classify_wait};
use fmn_core::rng::Pcg64Dxsm;
use fmn_hash::serial::{Schema, Writer};
use fmn_hash::sha256::sha256;
use fmn_mobject::record::{RecordBuffer, RecordSchema};
use fmn_mobject::{Mob, Mobject, SceneState, Stage};
use fmn_scene::journal::{BUNDLE_SCHEMA, JOURNAL_SCHEMA};
use fmn_scene::{
    AssetRead, CommandKind, CommandRecord, EffectClass, Entry, ImpureEffectTag, InvalidationReason,
    Journal, MAX_RENDER_BACKEND_IDENTITY_BYTES, RenderBackendRecord, RenderBackendRole,
    ReplayAudit, ReproBundle, SubprocessRecord, plan_replay,
};

fn vmob(stage: &mut Stage, points: &[[f64; 3]]) -> Mob {
    let mob = stage.add(Mobject::new());
    let entry = stage.get_mut(mob).unwrap();
    entry.buffer = RecordBuffer::new(RecordSchema::vmobject(), points.len()).unwrap();
    #[allow(clippy::cast_possible_truncation)]
    let flat: Vec<f32> = points
        .iter()
        .flat_map(|p| p.iter().map(|v| *v as f32))
        .collect();
    entry.buffer.write_range("point", 0, &flat);
    mob
}

fn command(kind: CommandKind, label: &str) -> CommandRecord {
    CommandRecord {
        kind,
        identity: sha256(label.as_bytes()),
        label: label.to_string(),
    }
}

fn entry_for(kind: CommandKind, label: &str, state: &[u8]) -> Entry {
    Entry {
        command: command(kind, label),
        effect: EffectClass::Pure,
        reads: Vec::new(),
        subprocesses: Vec::new(),
        checkpoint: None,
        state_hash: sha256(state),
    }
}

fn pad_writer(writer: &mut Writer, bytes: usize) {
    for _ in 0..bytes {
        writer.put_u8(0);
    }
}

/// Drive a small scripted session against a real Stage, journaling
/// each command with its post-state, checkpointing at entries 0 and 2.
fn scripted_session(stage: &mut Stage, rng: &Pcg64Dxsm) -> (Journal, Vec<CommandRecord>, Vec<u8>) {
    let mut journal = Journal::new();
    let mut incoming = Vec::new();

    let a = vmob(stage, &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
    let b = vmob(stage, &[[0.0, 1.0, 0.0], [1.0, 1.0, 0.0], [2.0, 1.0, 0.0]]);
    stage.add_to_scene(a).unwrap();
    stage.add_to_scene(b).unwrap();
    let state0 = SceneState::capture(stage, 0, 30, 0, rng)
        .to_bytes()
        .unwrap();
    let mut e0 = entry_for(CommandKind::Add, "add a, b", &state0);
    e0.checkpoint = Some(state0.clone());
    incoming.push(e0.command.clone());
    journal.record(e0).expect("journal entry storage reserves");

    stage.shift(a, [3.0, -1.0, 0.0]);
    let state1 = SceneState::capture(stage, 0, 30, 1, rng)
        .to_bytes()
        .unwrap();
    let mut e1 = entry_for(CommandKind::Play, "play shift(a)", &state1);
    e1.effect = EffectClass::from_purity(&classify_play(stage, &[]));
    incoming.push(e1.command.clone());
    journal.record(e1).expect("journal entry storage reserves");

    stage.shift(b, [0.0, 2.5, 0.0]);
    let state2 = SceneState::capture(stage, 0, 30, 2, rng)
        .to_bytes()
        .unwrap();
    let mut e2 = entry_for(CommandKind::Play, "play shift(b)", &state2);
    e2.checkpoint = Some(state2.clone());
    incoming.push(e2.command.clone());
    journal.record(e2).expect("journal entry storage reserves");

    let state3 = SceneState::capture(stage, 0, 30, 3, rng)
        .to_bytes()
        .unwrap();
    let e3 = entry_for(CommandKind::Wait, "wait 1.0", &state3);
    incoming.push(e3.command.clone());
    journal.record(e3).expect("journal entry storage reserves");

    (journal, incoming, state3)
}

#[test]
fn journal_reads_legacy_journals_without_newer_streams() {
    let mut writer = Writer::new(Schema::new(*b"FMNA", 3, 1, 0));
    writer.put_u32(0);
    let minor_zero = writer.finish().expect("minor-zero fixture encodes");

    let mut writer = Writer::new(Schema::new(*b"FMNA", 3, 1, 1));
    writer.put_u32(0);
    writer.put_u32(0);
    let minor_one = writer.finish().expect("minor-one fixture encodes");

    let mut writer = Writer::new(Schema::new(*b"FMNA", 3, 1, 2));
    writer.put_u32(0).put_u32(0).put_u32(0);
    let minor_two = writer.finish().expect("minor-two fixture encodes");

    for legacy in [&minor_zero, &minor_one, &minor_two] {
        let decoded = Journal::from_bytes(legacy).expect("legacy journal remains readable");
        assert!(decoded.entries().is_empty());
        assert!(decoded.events().is_empty());
        assert!(decoded.render_backends().is_empty());
    }
}

#[test]
fn journal_collection_counts_are_preflighted_before_reserve() {
    const COUNT: u32 = 65_536;

    let mut entries = Writer::new(JOURNAL_SCHEMA);
    entries.put_u32(COUNT);

    let mut reads = Writer::new(JOURNAL_SCHEMA);
    reads.put_u32(1);
    reads.put_u8(1);
    reads.put_digest(&sha256(b"command"));
    reads.put_str("");
    reads.put_u8(0);
    reads.put_u32(COUNT);
    pad_writer(&mut reads, 37);

    let mut subprocesses = Writer::new(JOURNAL_SCHEMA);
    subprocesses.put_u32(1);
    subprocesses.put_u8(1);
    subprocesses.put_digest(&sha256(b"command"));
    subprocesses.put_str("");
    subprocesses.put_u8(0);
    subprocesses.put_u32(0);
    subprocesses.put_u32(COUNT);
    pad_writer(&mut subprocesses, 33);

    let mut events = Writer::new(JOURNAL_SCHEMA);
    events.put_u32(0);
    events.put_u32(COUNT);

    let mut backends = Writer::new(JOURNAL_SCHEMA);
    backends.put_u32(0);
    backends.put_u32(0);
    backends.put_u32(COUNT);

    let mut closure = Writer::new(BUNDLE_SCHEMA);
    closure.put_str("");
    closure.put_u64(0);
    closure.put_u32(30);
    closure.put_u32(1);
    closure.put_u32(COUNT);

    let cases = [
        (
            "entry",
            Journal::from_bytes(&entries.finish().unwrap()).unwrap_err(),
            u64::from(COUNT) * 83,
            0,
        ),
        (
            "asset read",
            Journal::from_bytes(&reads.finish().unwrap()).unwrap_err(),
            u64::from(COUNT) * 40,
            37,
        ),
        (
            "subprocess",
            Journal::from_bytes(&subprocesses.finish().unwrap()).unwrap_err(),
            u64::from(COUNT) * 48,
            33,
        ),
        (
            "input event",
            Journal::from_bytes(&events.finish().unwrap()).unwrap_err(),
            u64::from(COUNT) * 27,
            0,
        ),
        (
            "render backend",
            Journal::from_bytes(&backends.finish().unwrap()).unwrap_err(),
            u64::from(COUNT) * 41,
            0,
        ),
        (
            "repro closure",
            ReproBundle::from_bytes(&closure.finish().unwrap()).unwrap_err(),
            u64::from(COUNT) * 40,
            0,
        ),
    ];
    for (field, error, minimum_bytes, remaining_bytes) in cases {
        assert_eq!(
            error.to_string(),
            format!(
                "journal {field} count {COUNT} requires at least {minimum_bytes} encoded bytes, but only {remaining_bytes} remain"
            )
        );
    }

    let mut valid = Journal::new();
    let mut entry = entry_for(CommandKind::Sound, "bounded collections", b"state");
    entry.reads.push(AssetRead {
        path: "asset.bin".to_owned(),
        digest: sha256(b"asset"),
    });
    entry.subprocesses.push(SubprocessRecord {
        tool_sha256_hex: "ab".repeat(32),
        argv_digest: sha256(b"argv"),
        destination: "output.wav".to_owned(),
    });
    valid.record(entry).expect("journal entry storage reserves");
    let valid_bytes = valid.to_bytes().unwrap();
    assert_eq!(
        Journal::from_bytes(&valid_bytes)
            .unwrap()
            .to_bytes()
            .unwrap(),
        valid_bytes
    );
}

#[test]
fn render_backend_identities_round_trip_deduplicate_and_bind_role() {
    let identity = b"canonical fast-cpu engine journal".to_vec();
    let stream = RenderBackendRecord::new(RenderBackendRole::FrameStream, identity.clone())
        .expect("bounded identity is valid");
    let native = RenderBackendRecord::new(RenderBackendRole::NativePresentation, identity)
        .expect("the same identity may serve a distinct role");
    let mut journal = Journal::new();
    journal
        .record_render_backends(&[stream.clone(), stream.clone(), native.clone()])
        .expect("valid backend records append");

    assert_eq!(journal.render_backends(), &[stream.clone(), native.clone()]);
    let bytes = journal.to_bytes().expect("journal encodes");
    let decoded = Journal::from_bytes(&bytes).expect("journal decodes");
    assert_eq!(decoded.render_backends(), &[stream, native]);
    assert_eq!(decoded.to_bytes().unwrap(), bytes);
}

#[test]
fn render_backend_identity_refuses_tampering_and_oversize() {
    let identity = b"canonical backend";
    let wrong_digest = sha256(b"different backend");
    assert_eq!(
        RenderBackendRecord::from_parts(RenderBackendRole::FrameStream, identity, wrong_digest,)
            .unwrap_err()
            .to_string(),
        "malformed journal: render backend identity digest"
    );
    assert_eq!(
        RenderBackendRecord::new(RenderBackendRole::FrameStream, Vec::new())
            .unwrap_err()
            .to_string(),
        "malformed journal: empty render backend identity"
    );
    assert_eq!(
        RenderBackendRecord::new(
            RenderBackendRole::FrameStream,
            vec![0; MAX_RENDER_BACKEND_IDENTITY_BYTES + 1],
        )
        .unwrap_err()
        .to_string(),
        "malformed journal: render backend identity exceeds limit"
    );
}

#[test]
fn record_replay_equivalence_is_bit_identical() {
    let mut stage = Stage::new();
    let rng = Pcg64Dxsm::from_seed(7);
    let (journal, incoming, final_state) = scripted_session(&mut stage, &rng);

    // Unchanged session: the whole journal is reusable, resuming from
    // the latest checkpoint (entry 2).
    let plan = plan_replay(&journal, &incoming, &|_| true);
    assert_eq!(plan.reuse, 4);
    assert_eq!(plan.resume_checkpoint, Some(2));
    assert_eq!(plan.reason, None);

    // Restore that checkpoint into a FRESH stage (the supervisor's
    // restore path) and re-verify: the recorded state bytes round-trip
    // bit-identically, RNG included.
    let checkpoint = journal.entries()[2].checkpoint.as_ref().unwrap();
    let mut fresh = Stage::new();
    let decoded = SceneState::from_bytes(checkpoint, &fresh).unwrap();
    fresh.restore(&decoded.snapshot);
    let replayed = SceneState::capture(
        &fresh,
        decoded.frames_elapsed,
        decoded.fps,
        decoded.play_count,
        &decoded.rng(),
    )
    .to_bytes()
    .unwrap();
    assert_eq!(&replayed, checkpoint, "checkpoint restore diverged");

    // The audit confirms replayed states against recorded hashes.
    let mut audit = ReplayAudit::new();
    assert!(audit.step(&journal, 0, &journal.entries()[0].state_hash));
    assert!(audit.step(&journal, 1, &journal.entries()[1].state_hash));
    assert_eq!(audit.verified(), 2);
    assert_eq!(audit.diverged_at(), None);
    let _ = final_state;
}

#[test]
fn changed_callback_invalidates_from_that_point() {
    let mut stage = Stage::new();
    let rng = Pcg64Dxsm::from_seed(7);
    let (journal, mut incoming, _) = scripted_session(&mut stage, &rng);

    // The user edited the second play's callback: its version hash —
    // the identity digest — changes.
    incoming[2] = command(CommandKind::Play, "play shift(b) EDITED");
    let plan = plan_replay(&journal, &incoming, &|_| true);
    assert_eq!(plan.reuse, 2);
    assert_eq!(
        plan.reason,
        Some(InvalidationReason::CommandMismatch { index: 2 })
    );
    // Resume from the checkpoint at entry 0 (entry 2's checkpoint is
    // past the valid prefix).
    assert_eq!(plan.resume_checkpoint, Some(0));
}

#[test]
fn custom_commands_are_opaque_by_decree() {
    let mut journal = Journal::new();
    let mut entry = entry_for(CommandKind::Custom, "mystery callback", b"s");
    // The recorder claims purity; the journal refuses the claim (R16).
    entry.effect = EffectClass::Pure;
    journal
        .record(entry)
        .expect("journal entry storage reserves");
    assert_eq!(journal.entries()[0].effect, EffectClass::Opaque);
    assert!(journal.entries()[0].is_replay_barrier());

    // An opaque entry stops reuse even with a matching identity.
    let incoming = vec![command(CommandKind::Custom, "mystery callback")];
    let plan = plan_replay(&journal, &incoming, &|_| true);
    assert_eq!(plan.reuse, 0);
    assert_eq!(
        plan.reason,
        Some(InvalidationReason::ReplayBarrier { index: 0 })
    );
}

#[test]
fn typed_input_remains_stateful_replayable_and_bound_to_its_input_closure() {
    let mut journal = Journal::new();
    let mut input = entry_for(CommandKind::Input, "typed edit", b"edited state");
    input.effect = EffectClass::Stateful(vec![ImpureEffectTag::UnclassifiedAnimation]);
    input.reads.push(AssetRead {
        path: "native/source".into(),
        digest: sha256(b"source"),
    });
    let incoming = vec![input.command.clone()];
    journal.record(input).unwrap();
    let bytes = journal.to_bytes().unwrap();
    let decoded = Journal::from_bytes(&bytes).unwrap();
    assert_eq!(decoded.to_bytes().unwrap(), bytes);
    assert_eq!(decoded.entries()[0].command.kind, CommandKind::Input);
    assert!(matches!(
        decoded.entries()[0].effect,
        EffectClass::Stateful(_)
    ));
    assert!(!decoded.entries()[0].is_replay_barrier());
    assert_eq!(plan_replay(&decoded, &incoming, &|_| true).reuse, 1);
    assert_eq!(plan_replay(&decoded, &incoming, &|_| false).reuse, 0);
}

#[test]
fn input_discriminant_cannot_be_smuggled_into_an_older_journal_minor() {
    let mut writer = Writer::new(Schema::new(*b"FMNA", 3, 1, 2));
    writer
        .put_u32(1)
        .put_u8(7)
        .put_digest(&sha256(b"input"))
        .put_str("typed input")
        .put_u8(0)
        .put_u32(0)
        .put_u32(0)
        .put_bool(false)
        .put_digest(&sha256(b"state"))
        .put_u32(0)
        .put_u32(0);
    let error = Journal::from_bytes(&writer.finish().unwrap()).unwrap_err();
    assert!(
        error.to_string().contains("requires journal minor 1.3"),
        "{error}"
    );
}

#[test]
fn changed_assets_invalidate_by_name() {
    let mut journal = Journal::new();
    let mut e0 = entry_for(CommandKind::Add, "add image", b"s0");
    e0.reads.push(AssetRead {
        path: "assets/texture.png".into(),
        digest: sha256(b"original bytes"),
    });
    let mut e1 = entry_for(CommandKind::Play, "play fade", b"s1");
    e1.reads.push(AssetRead {
        path: "assets/other.png".into(),
        digest: sha256(b"other bytes"),
    });
    let incoming = vec![e0.command.clone(), e1.command.clone()];
    journal.record(e0).expect("journal entry storage reserves");
    journal.record(e1).expect("journal entry storage reserves");

    // Second entry's asset changed on disk.
    let plan = plan_replay(&journal, &incoming, &|read| read.path != "assets/other.png");
    assert_eq!(plan.reuse, 1);
    assert_eq!(
        plan.reason,
        Some(InvalidationReason::AssetChanged {
            index: 1,
            path: "assets/other.png".into()
        })
    );
}

#[test]
fn subprocess_entries_are_replay_barriers() {
    let mut journal = Journal::new();
    let mut entry = entry_for(CommandKind::Sound, "add_sound boom.mp3", b"s");
    entry.subprocesses.push(SubprocessRecord {
        tool_sha256_hex: "ab".repeat(32),
        argv_digest: sha256(b"argv"),
        destination: "/tmp/boom.wav".into(),
    });
    let incoming = vec![entry.command.clone()];
    journal
        .record(entry)
        .expect("journal entry storage reserves");
    let plan = plan_replay(&journal, &incoming, &|_| true);
    assert_eq!(plan.reuse, 0);
    assert_eq!(
        plan.reason,
        Some(InvalidationReason::ReplayBarrier { index: 0 })
    );
}

#[test]
fn ordinary_scene_code_never_hits_a_pipeline_barrier() {
    // §17.4's assertion, on the command corpus: the ordinary command
    // vocabulary — via the journal's own constructors — cannot produce
    // a PixelObserving effect.
    let mut stage = Stage::new();
    let mob = vmob(
        &mut stage,
        &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
    );
    stage.add_to_scene(mob).unwrap();

    let corpus = [
        (CommandKind::Add, EffectClass::Pure),
        (
            CommandKind::Play,
            EffectClass::from_purity(&classify_play(&stage, &[])),
        ),
        (
            CommandKind::Wait,
            EffectClass::from_purity(&classify_wait(&stage, false)),
        ),
        (
            CommandKind::Wait,
            EffectClass::from_purity(&classify_wait(&stage, true)),
        ),
        (CommandKind::Remove, EffectClass::Pure),
        (CommandKind::Sound, EffectClass::Opaque),
    ];
    for (kind, effect) in corpus {
        assert!(
            !effect.is_pipeline_barrier(),
            "{kind:?} produced a pipeline barrier"
        );
    }
}

#[test]
fn purity_evidence_round_trips_with_the_classifier() {
    let mut stage = Stage::new();
    let mob = vmob(
        &mut stage,
        &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
    );
    stage.add_to_scene(mob).unwrap();

    // Clean stage: pure. With an updater: stateful, and the recorded
    // tags survive the container round-trip.
    assert_eq!(
        EffectClass::from_purity(&classify_wait(&stage, false)),
        EffectClass::Pure
    );
    stage.add_updater(mob, |_, _| {}, false).unwrap();
    let classified = EffectClass::from_purity(&classify_wait(&stage, true));
    assert_eq!(
        classified,
        EffectClass::Stateful(vec![
            ImpureEffectTag::StopCondition,
            ImpureEffectTag::SceneUpdater
        ])
    );

    let mut journal = Journal::new();
    let mut entry = entry_for(CommandKind::Wait, "wait_until", b"s");
    entry.effect = classified.clone();
    journal
        .record(entry)
        .expect("journal entry storage reserves");
    let decoded = Journal::from_bytes(&journal.to_bytes().unwrap()).unwrap();
    assert_eq!(decoded.entries()[0].effect, classified);
}

#[test]
fn serialization_is_deterministic_and_refuses_corruption() {
    let mut stage = Stage::new();
    let rng = Pcg64Dxsm::from_seed(3);
    let (journal, _, _) = scripted_session(&mut stage, &rng);

    let bytes = journal.to_bytes().unwrap();
    assert_eq!(bytes, journal.to_bytes().unwrap());
    assert_eq!(
        journal.content_hash().unwrap(),
        sha256(&bytes),
        "content hash is the canonical bytes' hash"
    );

    // Round trip preserves everything.
    let decoded = Journal::from_bytes(&bytes).unwrap();
    assert_eq!(decoded.entries().len(), journal.entries().len());
    for (a, b) in decoded.entries().iter().zip(journal.entries()) {
        assert!(a.command.matches(&b.command));
        assert_eq!(a.effect, b.effect);
        assert_eq!(a.state_hash, b.state_hash);
        assert_eq!(a.checkpoint, b.checkpoint);
    }

    // A flipped byte is a refusal, not a garbled journal.
    let mut corrupt = bytes.clone();
    let mid = corrupt.len() / 2;
    corrupt[mid] ^= 0x40;
    assert!(Journal::from_bytes(&corrupt).is_err());
    // Truncation likewise.
    assert!(Journal::from_bytes(&bytes[..bytes.len() - 5]).is_err());
}

#[test]
fn repro_bundle_end_to_end() {
    let mut stage = Stage::new();
    let rng = Pcg64Dxsm::from_seed(11);
    let (journal, incoming, _) = scripted_session(&mut stage, &rng);

    // "Files" on the recording machine.
    let mut disk = BTreeMap::<String, Vec<u8>>::new();
    disk.insert("scene.py".into(), b"class Demo(Scene): ...".to_vec());
    disk.insert("fonts/cm.ttf".into(), b"OTTO fake font".to_vec());

    let bundle = ReproBundle {
        scene_label: "Demo".into(),
        seed: 42,
        fps: (30000, 1001),
        closure: disk
            .iter()
            .map(|(path, bytes)| AssetRead {
                path: path.clone(),
                digest: sha256(bytes),
            })
            .collect(),
        journal,
    };
    let bytes = bundle.to_bytes().unwrap();
    assert_eq!(bytes, bundle.to_bytes().unwrap(), "bundle bytes drift");

    // "Fresh machine": decode, verify the closure, replay the journal.
    let received = ReproBundle::from_bytes(&bytes).unwrap();
    assert_eq!(received.scene_label, "Demo");
    assert_eq!(received.seed, 42);
    assert_eq!(received.fps, (30000, 1001));
    let disk2 = disk.clone();
    received
        .verify(&|path| disk2.get(path).cloned())
        .expect("identical closure must verify");

    // Identical incoming commands: full reuse; the restored checkpoint
    // reproduces the recorded state bit-identically.
    let plan = plan_replay(&received.journal, &incoming, &|_| true);
    assert_eq!(plan.reuse, received.journal.entries().len());
    let at = plan.resume_checkpoint.unwrap();
    let checkpoint = received.journal.entries()[at].checkpoint.as_ref().unwrap();
    let mut fresh = Stage::new();
    let decoded = SceneState::from_bytes(checkpoint, &fresh).unwrap();
    fresh.restore(&decoded.snapshot);
    let replayed = SceneState::capture(
        &fresh,
        decoded.frames_elapsed,
        decoded.fps,
        decoded.play_count,
        &decoded.rng(),
    )
    .to_bytes()
    .unwrap();
    assert_eq!(&replayed, checkpoint, "fresh-machine replay diverged");

    // A divergent closure is named precisely.
    let mut tampered = disk;
    tampered.insert("fonts/cm.ttf".into(), b"DIFFERENT font".to_vec());
    let err = received
        .verify(&|path| tampered.get(path).cloned())
        .unwrap_err();
    assert_eq!(err.path, "fonts/cm.ttf");
    assert!(err.found.is_some());
    // And an absent file reads as nothing found.
    let err = received.verify(&|_| None).unwrap_err();
    assert!(err.found.is_none());
}

#[test]
fn audit_divergence_forces_reexecution_and_records_why() {
    let mut stage = Stage::new();
    let rng = Pcg64Dxsm::from_seed(5);
    let (journal, _, _) = scripted_session(&mut stage, &rng);

    let mut audit = ReplayAudit::new();
    assert!(audit.step(&journal, 0, &journal.entries()[0].state_hash));
    // A replayed entry produced the wrong state: replay must stop.
    let wrong = sha256(b"not the recorded state");
    assert!(!audit.step(&journal, 1, &wrong));
    assert_eq!(audit.verified(), 1);
    assert_eq!(audit.diverged_at(), Some(1));
    // Once diverged, nothing continues — even a matching hash.
    assert!(!audit.step(&journal, 2, &journal.entries()[2].state_hash));
}
