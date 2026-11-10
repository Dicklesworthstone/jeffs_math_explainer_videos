//! fm-39s supervisor acceptance: warm-cache checkpoint fidelity,
//! trailing-edit PG-4 recovery, audited replay fallback, structured panic
//! conversion, and a real subprocess crash that leaves the parent supervisor
//! alive and automatically restores a replacement worker.

use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::Child;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use fmn_anim::RationalTime;
use fmn_cache::{Namespace, NamespacePolicy, Store, StoreConfig};
use fmn_hash::{Digest, sha256};
use fmn_platform::clock::{Clock, FakeClock};
use fmn_platform::fs::{FileSystem, VirtualFs};
use fmn_scene::{
    AssetRead, CommandKind, CommandRecord, EffectClass, Entry, EventError, EventPayload,
    InputEvent, Journal, JournalError, Key, Modifiers, RenderBackendRecord, RenderBackendRole,
    Scene,
};
use fmn_studio::{
    BuildError, ChannelError, ChannelFailureKind, Checkpoint, CheckpointSource, CrashReport,
    FrameEncoding, FramePayload, FrameStream, FramingError, JournalReplay, LaunchError,
    ProtocolError, ProtocolLimits, ProtocolVersion, RebuildDriver, RequestEnvelope,
    ResponseEnvelope, ServiceError, StudioDataKind, Supervisor, SupervisorConfig, SupervisorError,
    SupervisorReply, SupervisorRequest, TransportCapabilities, WorkerArtifact, WorkerChannel,
    WorkerErrorCode, WorkerLauncher, WorkerResponse, WorkerServeError, WorkerServeOutcome,
    WorkerService, read_response, serve_worker, write_request,
};

fn lock_poisoned<T>(error: PoisonError<T>) -> T {
    error.into_inner()
}

fn command(index: usize) -> CommandRecord {
    let label = format!("play command {index}");
    CommandRecord {
        kind: CommandKind::Play,
        identity: sha256(label.as_bytes()),
        label,
    }
}

fn scene_state_bytes() -> Vec<u8> {
    let mut scene = Scene::default();
    scene.state_bytes().expect("default SceneState encodes")
}

fn make_journal(count: usize, checkpoint_index: Option<usize>) -> Journal {
    let checkpoint = scene_state_bytes();
    let mut journal = Journal::new();
    for index in 0..count {
        let state_hash = if checkpoint_index == Some(index) {
            sha256(&checkpoint)
        } else {
            sha256(format!("state {index}").as_bytes())
        };
        journal
            .record(Entry {
                command: command(index),
                effect: EffectClass::Pure,
                reads: Vec::new(),
                subprocesses: Vec::new(),
                checkpoint: (checkpoint_index == Some(index)).then(|| checkpoint.clone()),
                state_hash,
            })
            .expect("journal entry storage reserves");
    }
    journal
}

fn commands(journal: &Journal) -> Vec<CommandRecord> {
    journal
        .entries()
        .iter()
        .map(|entry| entry.command.clone())
        .collect()
}

// Store::open validates absoluteness with host path semantics even over a
// VirtualFs; `/studio-cache` has no drive prefix on Windows.
const STUDIO_CACHE_ROOT: &str = if cfg!(windows) {
    r"C:\studio-cache"
} else {
    "/studio-cache"
};

fn cache(clock: Arc<FakeClock>) -> Namespace {
    let fs: Arc<dyn FileSystem> = Arc::new(VirtualFs::new());
    let clock_cap: Arc<dyn Clock> = clock;
    Store::open(fs, clock_cap, STUDIO_CACHE_ROOT, StoreConfig::default())
        .expect("store opens")
        .namespace(
            "studio-replay",
            1,
            NamespacePolicy {
                ceiling_bytes: None,
            },
        )
        .expect("namespace opens")
}

#[derive(Default)]
struct FakeState {
    launches: usize,
    terminated: usize,
    crash_first_play: bool,
    crash_every_play: bool,
    channel_error_first_play: bool,
    diverge_next_replay: bool,
    skew_handshake: bool,
    wrong_build_handshake: bool,
    frame_response: Option<FrameStream>,
    journal_segment: Option<(u64, Vec<u8>)>,
    checkpoint_response: Option<Checkpoint>,
    inspection_scene: Option<String>,
    inspection_bytes: Option<Vec<u8>>,
    inspection_digest: Option<Digest>,
    restored: Vec<Vec<u8>>,
    replayed: Vec<(u64, u64)>,
    replayed_events: Vec<InputEvent>,
}

struct FakeLauncher {
    state: Arc<Mutex<FakeState>>,
    clock: Arc<FakeClock>,
    exchange_cost: Duration,
}

impl WorkerLauncher for FakeLauncher {
    fn launch(
        &mut self,
        artifact: &WorkerArtifact,
        _limits: ProtocolLimits,
    ) -> Result<Box<dyn WorkerChannel>, LaunchError> {
        let mut state = self.state.lock().unwrap_or_else(lock_poisoned);
        state.launches += 1;
        let launch = state.launches;
        let crash_every_play = state.crash_every_play;
        let crash_on_play = crash_every_play || (state.crash_first_play && launch == 1);
        let channel_error_on_play = state.channel_error_first_play && launch == 1;
        drop(state);
        Ok(Box::new(FakeChannel {
            state: Arc::clone(&self.state),
            clock: Arc::clone(&self.clock),
            exchange_cost: self.exchange_cost,
            build_id: artifact.build_id,
            crash_on_play,
            crash_message: if crash_every_play {
                format!("scripted scene panic {launch}")
            } else {
                "scripted scene panic".to_owned()
            },
            channel_error_on_play,
            terminated: false,
        }))
    }
}

struct FakeChannel {
    state: Arc<Mutex<FakeState>>,
    clock: Arc<FakeClock>,
    exchange_cost: Duration,
    build_id: Digest,
    crash_on_play: bool,
    crash_message: String,
    channel_error_on_play: bool,
    terminated: bool,
}

impl WorkerChannel for FakeChannel {
    fn exchange(
        &mut self,
        request: &RequestEnvelope,
        _timeout: Duration,
    ) -> Result<ResponseEnvelope, ChannelError> {
        self.clock.advance(self.exchange_cost);
        if self.channel_error_on_play && matches!(&request.request, SupervisorRequest::Play { .. })
        {
            self.channel_error_on_play = false;
            return Err(ChannelError {
                kind: ChannelFailureKind::Closed,
                detail: "short diagnostic".to_owned(),
                stderr_tail: vec![0xff; 4096],
            });
        }
        let response = match &request.request {
            SupervisorRequest::Hello { .. } => {
                let skew = self
                    .state
                    .lock()
                    .unwrap_or_else(lock_poisoned)
                    .skew_handshake;
                let worker_build = if self
                    .state
                    .lock()
                    .unwrap_or_else(lock_poisoned)
                    .wrong_build_handshake
                {
                    sha256(b"wrong worker build")
                } else {
                    self.build_id
                };
                WorkerResponse::Hello {
                    version: if skew {
                        ProtocolVersion { major: 2, minor: 0 }
                    } else {
                        fmn_studio::CURRENT_VERSION
                    },
                    worker_build,
                    transports: TransportCapabilities {
                        pipe: true,
                        shared_memory: false,
                    },
                }
            }
            SupervisorRequest::EnumerateScenes => {
                let mut state = self.state.lock().unwrap_or_else(lock_poisoned);
                if let Some(frame) = state.frame_response.take() {
                    WorkerResponse::Frame(frame)
                } else if let Some((start_entry, journal)) = state.journal_segment.take() {
                    WorkerResponse::JournalSegment {
                        scene: "Demo".to_owned(),
                        start_entry,
                        journal,
                    }
                } else if let Some(checkpoint) = state.checkpoint_response.take() {
                    WorkerResponse::Checkpoint(checkpoint)
                } else {
                    WorkerResponse::Scenes(vec!["Demo".to_owned()])
                }
            }
            SupervisorRequest::Play { .. } if self.crash_on_play => {
                self.crash_on_play = false;
                WorkerResponse::Crash(CrashReport {
                    scene: Some("Demo".to_owned()),
                    message: self.crash_message.clone(),
                    journal_tail: b"scripted journal tail".to_vec(),
                    state_hash: None,
                })
            }
            SupervisorRequest::Inspect { .. } => {
                let state = self.state.lock().unwrap_or_else(lock_poisoned);
                let scene = state.inspection_scene.clone();
                if let Some(scene) = scene {
                    let bytes = state
                        .inspection_bytes
                        .clone()
                        .unwrap_or_else(|| br#"{"nodes":[]}"#.to_vec());
                    let digest = state.inspection_digest.unwrap_or_else(|| sha256(&bytes));
                    WorkerResponse::StudioData {
                        scene,
                        kind: StudioDataKind::Inspection,
                        digest,
                        bytes,
                    }
                } else {
                    WorkerResponse::Ack {
                        state_hash: None,
                        journal_len: 0,
                    }
                }
            }
            SupervisorRequest::Play { .. }
            | SupervisorRequest::Seek { .. }
            | SupervisorRequest::Scrub { .. }
            | SupervisorRequest::Event { .. }
            | SupervisorRequest::Overlay { .. } => WorkerResponse::Ack {
                state_hash: None,
                journal_len: 0,
            },
            SupervisorRequest::RestoreCheckpoint(checkpoint) => {
                self.state
                    .lock()
                    .unwrap_or_else(lock_poisoned)
                    .restored
                    .push(checkpoint.state.clone());
                WorkerResponse::Ack {
                    state_hash: Some(checkpoint.state_hash),
                    journal_len: checkpoint.after_entry + 1,
                }
            }
            SupervisorRequest::ReplayJournal(replay) => {
                let journal = Journal::from_bytes(&replay.journal).expect("valid replay journal");
                let start = usize::try_from(replay.from_entry).expect("test range fits");
                let end = usize::try_from(replay.through_entry).expect("test range fits");
                let mut hashes: Vec<Digest> = journal.entries()[start..end]
                    .iter()
                    .map(|entry| entry.state_hash)
                    .collect();
                let mut state = self.state.lock().unwrap_or_else(lock_poisoned);
                state
                    .replayed
                    .push((replay.from_entry, replay.through_entry));
                state.replayed_events = journal.events().to_vec();
                if state.diverge_next_replay && !hashes.is_empty() {
                    state.diverge_next_replay = false;
                    hashes[0] = sha256(b"deliberate divergence");
                }
                WorkerResponse::ReplayComplete {
                    from_entry: replay.from_entry,
                    state_hashes: hashes,
                }
            }
            SupervisorRequest::Shutdown => WorkerResponse::Bye,
        };
        Ok(ResponseEnvelope {
            request_id: request.request_id,
            response,
        })
    }

    fn terminate(&mut self) {
        if !self.terminated {
            self.terminated = true;
            self.state.lock().unwrap_or_else(lock_poisoned).terminated += 1;
        }
    }
}

struct ScriptedBuilder {
    clock: Arc<FakeClock>,
    build_cost: Duration,
    build: usize,
    executable: PathBuf,
}

impl ScriptedBuilder {
    fn fake(clock: Arc<FakeClock>, build_cost: Duration) -> Self {
        Self {
            clock,
            build_cost,
            build: 0,
            // The launcher is scripted, so the path is never opened — but the
            // supervisor validates absoluteness with host path semantics, and
            // `/worker/fmn` has no drive prefix on Windows.
            executable: PathBuf::from(if cfg!(windows) {
                r"C:\worker\fmn.exe"
            } else {
                "/worker/fmn"
            }),
        }
    }
}

impl RebuildDriver for ScriptedBuilder {
    fn rebuild(&mut self) -> Result<WorkerArtifact, BuildError> {
        self.clock.advance(self.build_cost);
        self.build += 1;
        Ok(WorkerArtifact {
            executable: self.executable.clone(),
            argv: vec!["--studio-worker".to_owned()],
            env: Vec::new(),
            cwd: Some(PathBuf::from(if cfg!(windows) {
                r"C:\workspace"
            } else {
                "/workspace"
            })),
            build_id: sha256(format!("worker build {}", self.build).as_bytes()),
        })
    }
}

fn fake_supervisor(
    state: Arc<Mutex<FakeState>>,
    clock: Arc<FakeClock>,
    exchange_cost: Duration,
) -> Supervisor {
    let config = SupervisorConfig {
        request_timeout: Duration::from_secs(1),
        edit_to_frame_budget: Duration::from_secs(1),
        ..SupervisorConfig::default()
    };
    Supervisor::new(
        Box::new(FakeLauncher {
            state,
            clock: Arc::clone(&clock),
            exchange_cost,
        }),
        clock.clone(),
        cache(clock),
        config,
    )
}

#[test]
fn worker_crash_auto_restarts_restores_warm_checkpoint_and_parent_survives() {
    let state = Arc::new(Mutex::new(FakeState {
        crash_first_play: true,
        ..FakeState::default()
    }));
    let clock = Arc::new(FakeClock::new());
    let mut supervisor = fake_supervisor(
        Arc::clone(&state),
        Arc::clone(&clock),
        Duration::from_millis(10),
    );
    let journal = make_journal(2, Some(0));
    let checkpoint = journal.entries()[0]
        .checkpoint
        .as_ref()
        .expect("checkpoint")
        .clone();
    supervisor
        .install_session("Demo", journal.clone())
        .expect("session");
    let mut builder = ScriptedBuilder::fake(Arc::clone(&clock), Duration::from_millis(20));
    supervisor.build_and_start(&mut builder).expect("start");

    let reply = supervisor
        .request(
            SupervisorRequest::Play {
                scene: "Demo".to_owned(),
                command: command(1),
            },
            &|_: &AssetRead| true,
        )
        .expect("supervisor recovers");
    let SupervisorReply::Recovered { crash, recovery } = reply else {
        // ubs:ignore - deliberate assertion failure in a recovery test.
        std::panic::panic_any("expected automatic recovery");
    };
    assert_eq!(crash.message, "scripted scene panic");
    assert_eq!(recovery.restored_checkpoint, Some(0));
    assert_eq!(recovery.checkpoint_source, Some(CheckpointSource::Cache));
    assert_eq!(recovery.replayed_entries, 1);
    assert!(!recovery.cold_fallback);
    assert_eq!(supervisor.generation(), 2);
    assert_eq!(supervisor.crashes(), &[crash]);

    let state = state.lock().unwrap_or_else(lock_poisoned);
    assert_eq!(state.launches, 2);
    assert_eq!(state.restored.last(), Some(&checkpoint));
    drop(state);

    assert_eq!(
        supervisor
            .request(SupervisorRequest::EnumerateScenes, &|_| true)
            .expect("replacement answers"),
        SupervisorReply::Worker(WorkerResponse::Scenes(vec!["Demo".to_owned()]))
    );
}

#[test]
fn multiple_installed_checkpoints_and_worker_refresh_recover_from_latest_row() {
    let checkpoint = scene_state_bytes();
    let checkpoint_hash = sha256(&checkpoint);
    let mut journal = Journal::new();
    for index in 0..3 {
        journal
            .record(Entry {
                command: command(index),
                effect: EffectClass::Pure,
                reads: Vec::new(),
                subprocesses: Vec::new(),
                checkpoint: (index < 2).then(|| checkpoint.clone()),
                state_hash: if index < 2 {
                    checkpoint_hash
                } else {
                    sha256(b"terminal state")
                },
            })
            .expect("journal entry storage reserves");
    }
    let incoming = commands(&journal);
    let state = Arc::new(Mutex::new(FakeState::default()));
    let clock = Arc::new(FakeClock::new());
    let mut supervisor = fake_supervisor(Arc::clone(&state), Arc::clone(&clock), Duration::ZERO);
    supervisor
        .install_session("Demo", journal)
        .expect("multiple checkpoints install");
    let mut builder = ScriptedBuilder::fake(clock, Duration::ZERO);
    supervisor.build_and_start(&mut builder).expect("start");

    state
        .lock()
        .unwrap_or_else(lock_poisoned)
        .checkpoint_response = Some(Checkpoint {
        scene: "Demo".to_owned(),
        after_entry: 1,
        state_hash: checkpoint_hash,
        state: checkpoint.clone(),
    });
    let reply = supervisor
        .request(SupervisorRequest::EnumerateScenes, &|_| true)
        .expect("matching worker checkpoint is retained");
    assert!(matches!(
        reply,
        SupervisorReply::Worker(WorkerResponse::Checkpoint(Checkpoint {
            after_entry: 1,
            ..
        }))
    ));

    let report = supervisor
        .rebuild_and_restart(&mut builder, &incoming, &|_| true)
        .expect("latest retained checkpoint recovers");
    assert_eq!(report.restored_checkpoint, Some(1));
    assert_eq!(report.checkpoint_source, Some(CheckpointSource::Cache));
    assert_eq!(
        state.lock().unwrap_or_else(lock_poisoned).restored.last(),
        Some(&checkpoint)
    );
}

fn repeating_crash_supervisor(max_crash_reports: usize) -> Supervisor {
    let state = Arc::new(Mutex::new(FakeState {
        crash_every_play: true,
        ..FakeState::default()
    }));
    let clock = Arc::new(FakeClock::new());
    let mut supervisor = Supervisor::new(
        Box::new(FakeLauncher {
            state,
            clock: Arc::clone(&clock),
            exchange_cost: Duration::from_millis(1),
        }),
        clock.clone(),
        cache(clock.clone()),
        SupervisorConfig {
            max_crash_reports,
            ..SupervisorConfig::default()
        },
    );
    supervisor
        .install_session("Demo", Journal::new())
        .expect("session");
    let mut builder = ScriptedBuilder::fake(clock, Duration::ZERO);
    supervisor.build_and_start(&mut builder).expect("start");
    supervisor
}

#[test]
fn repeated_crash_history_evicts_oldest_and_zero_disables_retention() {
    let mut bounded = repeating_crash_supervisor(2);
    for launch in 1..=3 {
        let reply = bounded
            .request(
                SupervisorRequest::Play {
                    scene: "Demo".to_owned(),
                    command: command(launch),
                },
                &|_| true,
            )
            .expect("supervisor recovers");
        let SupervisorReply::Recovered { crash, .. } = reply else {
            std::panic::panic_any("expected automatic recovery");
        };
        assert_eq!(crash.message, format!("scripted scene panic {launch}"));
    }
    assert_eq!(
        bounded
            .crashes()
            .iter()
            .map(|report| report.message.as_str())
            .collect::<Vec<_>>(),
        ["scripted scene panic 2", "scripted scene panic 3"]
    );

    let mut disabled = repeating_crash_supervisor(0);
    let reply = disabled
        .request(
            SupervisorRequest::Play {
                scene: "Demo".to_owned(),
                command: command(1),
            },
            &|_| true,
        )
        .expect("supervisor recovers without retaining history");
    assert!(matches!(reply, SupervisorReply::Recovered { .. }));
    assert!(disabled.crashes().is_empty());
}

#[test]
fn locally_synthesized_crash_message_honors_the_effective_wire_budget() {
    let empty_journal_bytes = Journal::new().to_bytes().unwrap().len();
    assert_eq!(empty_journal_bytes, 68);
    let limits = ProtocolLimits {
        // Keep this at the smallest field budget that admits the current empty
        // journal, so max_field remains the active crash-message ceiling.
        max_field_bytes: empty_journal_bytes,
        max_crash_message_bytes: 128,
        ..ProtocolLimits::default()
    };
    let (crash, _) = recover_from_channel_error(Journal::new(), limits)
        .expect("a representable local crash report recovers");
    assert!(crash.message.len() <= limits.max_field_bytes);
    assert!(
        crash
            .message
            .starts_with("worker channel Closed: short diagnostic")
    );
    assert_eq!(
        crash.message,
        "worker channel Closed: short diagnostic; stderr tail: \u{fffd}\u{fffd}\u{fffd}\u{fffd}"
    );
    ResponseEnvelope {
        request_id: 1,
        response: WorkerResponse::Crash(crash),
    }
    .to_bytes(limits)
    .expect("the locally synthesized crash report re-encodes under the same limits");
}

#[test]
fn locally_synthesized_crash_refuses_an_impossible_zero_message_budget() {
    let limits = ProtocolLimits {
        max_crash_message_bytes: 0,
        ..ProtocolLimits::default()
    };
    let error = recover_from_channel_error(Journal::new(), limits)
        .expect_err("a nonempty crash diagnostic cannot fit a zero-byte wire budget");
    assert!(matches!(
        error,
        SupervisorError::Protocol(fmn_studio::ProtocolError::PayloadLimit {
            field: "crash message",
            limit: 0,
            needed: 1,
        })
    ));
}

#[test]
fn locally_synthesized_crash_tail_releases_the_full_journal_allocation() {
    for tail_limit in [16, 0] {
        let journal = make_journal(8, None);
        let journal_bytes = journal.to_bytes().expect("journal encodes");
        assert!(journal_bytes.len() > tail_limit);
        let expected_tail = journal_bytes[journal_bytes.len() - tail_limit..].to_vec();
        let limits = ProtocolLimits {
            max_crash_tail_bytes: tail_limit,
            ..ProtocolLimits::default()
        };

        let (crash, retained_tail_capacity) = recover_from_channel_error(journal, limits)
            .expect("a representable local crash report recovers");
        assert_eq!(crash.journal_tail, expected_tail);
        assert!(
            crash.journal_tail.capacity() <= tail_limit,
            "returned tail retained capacity {} above limit {tail_limit}",
            crash.journal_tail.capacity()
        );
        assert!(
            retained_tail_capacity <= tail_limit,
            "history tail retained capacity {retained_tail_capacity} above limit {tail_limit}"
        );
    }
}

fn recover_from_channel_error(
    journal: Journal,
    limits: ProtocolLimits,
) -> Result<(CrashReport, usize), SupervisorError> {
    let state = Arc::new(Mutex::new(FakeState {
        channel_error_first_play: true,
        ..FakeState::default()
    }));
    let clock = Arc::new(FakeClock::new());
    let mut supervisor = Supervisor::new(
        Box::new(FakeLauncher {
            state,
            clock: Arc::clone(&clock),
            exchange_cost: Duration::from_millis(1),
        }),
        clock.clone(),
        cache(clock.clone()),
        SupervisorConfig {
            protocol_limits: limits,
            ..SupervisorConfig::default()
        },
    );
    supervisor
        .install_session("Demo", journal)
        .expect("session");
    let mut builder = ScriptedBuilder::fake(clock, Duration::ZERO);
    supervisor.build_and_start(&mut builder).expect("start");
    let reply = supervisor.request(
        SupervisorRequest::Play {
            scene: "Demo".to_owned(),
            command: command(1),
        },
        &|_| true,
    )?;
    let SupervisorReply::Recovered { crash, .. } = reply else {
        std::panic::panic_any("expected automatic recovery");
    };
    assert_eq!(supervisor.crashes().first(), Some(&crash));
    let retained_tail_capacity = supervisor.crashes()[0].journal_tail.capacity();
    Ok((crash, retained_tail_capacity))
}

#[test]
fn scene_bearing_responses_are_correlated_to_the_request_scene() {
    let state = Arc::new(Mutex::new(FakeState {
        inspection_scene: Some("Other".to_owned()),
        ..FakeState::default()
    }));
    let clock = Arc::new(FakeClock::new());
    let mut supervisor = fake_supervisor(Arc::clone(&state), Arc::clone(&clock), Duration::ZERO);
    supervisor
        .install_session("Demo", Journal::new())
        .expect("session");
    let mut builder = ScriptedBuilder::fake(clock, Duration::ZERO);
    supervisor.build_and_start(&mut builder).expect("start");

    let error = supervisor
        .request(
            SupervisorRequest::Inspect {
                scene: "Demo".to_owned(),
            },
            &|_| true,
        )
        .expect_err("a response for another scene must not cross the supervisor boundary");
    let SupervisorError::Channel(error) = error else {
        std::panic::panic_any(format!(
            "expected a channel correlation error, found {error}"
        ));
    };
    assert_eq!(error.kind, ChannelFailureKind::Correlation);
    assert_eq!(
        error.detail,
        "scene-bearing response did not match request scene"
    );

    state.lock().unwrap_or_else(lock_poisoned).inspection_scene = Some("Demo".to_owned());
    let reply = supervisor
        .request(
            SupervisorRequest::Inspect {
                scene: "Demo".to_owned(),
            },
            &|_| true,
        )
        .expect("the matching scene response remains valid");
    let SupervisorReply::Worker(WorkerResponse::StudioData { scene, .. }) = reply else {
        std::panic::panic_any("expected matching Studio data response");
    };
    // ubs:ignore - scene names are public routing identifiers, not secrets.
    assert_eq!(scene, "Demo");
}

#[test]
fn injected_worker_responses_are_revalidated_before_exposure() {
    let state = Arc::new(Mutex::new(FakeState {
        inspection_scene: Some("Demo".to_owned()),
        inspection_digest: Some(sha256(b"different Studio data")),
        ..FakeState::default()
    }));
    let clock = Arc::new(FakeClock::new());
    let mut supervisor = fake_supervisor(Arc::clone(&state), Arc::clone(&clock), Duration::ZERO);
    let mut builder = ScriptedBuilder::fake(clock, Duration::ZERO);
    supervisor.build_and_start(&mut builder).expect("start");

    let error = supervisor
        .request(
            SupervisorRequest::Inspect {
                scene: "Demo".to_owned(),
            },
            &|_| true,
        )
        .expect_err("an injected response with the wrong digest must be refused");
    let SupervisorError::Channel(error) = error else {
        std::panic::panic_any(format!("expected a channel protocol error, found {error}"));
    };
    assert_eq!(error.kind, ChannelFailureKind::Protocol);
    assert!(error.detail.contains("Studio data digest"));
    assert_eq!(state.lock().unwrap_or_else(lock_poisoned).terminated, 1);
    assert_eq!(supervisor.transports(), None);

    let checkpoint = b"12345".to_vec();
    let checkpoint_state = Arc::new(Mutex::new(FakeState {
        checkpoint_response: Some(Checkpoint {
            scene: "Demo".to_owned(),
            after_entry: 0,
            state_hash: sha256(&checkpoint),
            state: checkpoint,
        }),
        ..FakeState::default()
    }));
    let checkpoint_clock = Arc::new(FakeClock::new());
    let limits = ProtocolLimits {
        max_checkpoint_bytes: 4,
        ..ProtocolLimits::default()
    };
    let mut checkpoint_supervisor = Supervisor::new(
        Box::new(FakeLauncher {
            state: Arc::clone(&checkpoint_state),
            clock: Arc::clone(&checkpoint_clock),
            exchange_cost: Duration::ZERO,
        }),
        checkpoint_clock.clone(),
        cache(checkpoint_clock.clone()),
        SupervisorConfig {
            protocol_limits: limits,
            ..SupervisorConfig::default()
        },
    );
    let mut checkpoint_builder = ScriptedBuilder::fake(checkpoint_clock, Duration::ZERO);
    checkpoint_supervisor
        .build_and_start(&mut checkpoint_builder)
        .expect("start");
    let error = checkpoint_supervisor
        .request(SupervisorRequest::EnumerateScenes, &|_| true)
        .expect_err("an injected checkpoint above its budget must be refused");
    let SupervisorError::Channel(error) = error else {
        std::panic::panic_any(format!("expected a channel protocol error, found {error}"));
    };
    assert_eq!(error.kind, ChannelFailureKind::Protocol);
    assert!(error.detail.contains("checkpoint payload 5 bytes"));
    assert_eq!(
        checkpoint_state
            .lock()
            .unwrap_or_else(lock_poisoned)
            .terminated,
        1
    );
    assert_eq!(checkpoint_supervisor.transports(), None);
}

#[test]
fn trailing_edit_restores_entry_28_and_meets_pg4_scripted_budget() {
    let state = Arc::new(Mutex::new(FakeState::default()));
    let clock = Arc::new(FakeClock::new());
    let mut supervisor = fake_supervisor(
        Arc::clone(&state),
        Arc::clone(&clock),
        Duration::from_millis(50),
    );
    let journal = make_journal(30, Some(28));
    supervisor
        .install_session("Demo", journal.clone())
        .expect("session");
    let mut builder = ScriptedBuilder::fake(Arc::clone(&clock), Duration::from_millis(400));
    supervisor
        .build_and_start(&mut builder)
        .expect("initial start");

    let mut edited = commands(&journal);
    edited[29] = CommandRecord {
        kind: CommandKind::Play,
        identity: sha256(b"edited trailing command"),
        label: "play edited trailing command".to_owned(),
    };
    let report = supervisor
        .rebuild_and_restart(&mut builder, &edited, &|_| true)
        .expect("autoreload");
    assert_eq!(report.plan.reuse, 29);
    assert_eq!(report.restored_checkpoint, Some(28));
    assert_eq!(report.checkpoint_source, Some(CheckpointSource::Cache));
    assert_eq!(report.replayed_entries, 0);
    assert_eq!(report.elapsed, Duration::from_millis(550));
    assert!(report.within_edit_to_frame_budget);
    assert_eq!(report.generation, 2);
}

#[test]
fn replay_hash_divergence_discards_reuse_and_cold_reexecutes() {
    let state = Arc::new(Mutex::new(FakeState {
        diverge_next_replay: true,
        ..FakeState::default()
    }));
    let clock = Arc::new(FakeClock::new());
    let mut supervisor = fake_supervisor(
        Arc::clone(&state),
        Arc::clone(&clock),
        Duration::from_millis(1),
    );
    let journal = make_journal(3, Some(0));
    let incoming = commands(&journal);
    supervisor
        .install_session("Demo", journal)
        .expect("session");
    let mut builder = ScriptedBuilder::fake(Arc::clone(&clock), Duration::from_millis(1));
    supervisor.build_and_start(&mut builder).expect("start");
    let report = supervisor
        .rebuild_and_restart(&mut builder, &incoming, &|_| true)
        .expect("fallback succeeds");
    assert_eq!(report.diverged_at, Some(1));
    assert!(report.cold_fallback);
    assert_eq!(report.restored_checkpoint, None);
    assert_eq!(report.replayed_entries, 3);
    assert_eq!(report.generation, 3);
    assert_eq!(
        state.lock().unwrap_or_else(lock_poisoned).replayed,
        vec![(1, 3), (0, 3)]
    );
}

#[test]
fn handshake_version_skew_fails_closed_before_scene_commands() {
    let state = Arc::new(Mutex::new(FakeState {
        skew_handshake: true,
        ..FakeState::default()
    }));
    let clock = Arc::new(FakeClock::new());
    let mut supervisor = fake_supervisor(Arc::clone(&state), Arc::clone(&clock), Duration::ZERO);
    let mut builder = ScriptedBuilder::fake(clock, Duration::ZERO);
    assert!(matches!(
        supervisor.build_and_start(&mut builder),
        Err(fmn_studio::SupervisorError::Protocol(
            fmn_studio::ProtocolError::VersionSkew { .. }
        ))
    ));
    assert_eq!(supervisor.generation(), 0);
    assert_eq!(state.lock().unwrap_or_else(lock_poisoned).launches, 1);
}

#[test]
fn bad_replacement_keeps_the_previous_healthy_worker_live() {
    let state = Arc::new(Mutex::new(FakeState::default()));
    let clock = Arc::new(FakeClock::new());
    let mut supervisor = fake_supervisor(Arc::clone(&state), Arc::clone(&clock), Duration::ZERO);
    let journal = make_journal(1, Some(0));
    let incoming = commands(&journal);
    supervisor
        .install_session("Demo", journal)
        .expect("session");
    let mut builder = ScriptedBuilder::fake(clock, Duration::ZERO);
    supervisor
        .build_and_start(&mut builder)
        .expect("initial worker");

    state
        .lock()
        .unwrap_or_else(lock_poisoned)
        .wrong_build_handshake = true;
    assert!(matches!(
        supervisor.rebuild_and_restart(&mut builder, &incoming, &|_| true),
        Err(fmn_studio::SupervisorError::BuildIdentityMismatch { .. })
    ));
    assert_eq!(supervisor.generation(), 1);
    assert_eq!(
        supervisor
            .request(SupervisorRequest::EnumerateScenes, &|_| true)
            .expect("old worker remains live"),
        SupervisorReply::Worker(WorkerResponse::Scenes(vec!["Demo".to_owned()]))
    );
    let state = state.lock().unwrap_or_else(lock_poisoned);
    assert_eq!(state.launches, 2);
    assert_eq!(state.terminated, 1, "only the bad candidate was retired");
}

#[test]
fn invalid_session_install_is_atomic_and_preserves_prior_recovery_state() {
    let state = Arc::new(Mutex::new(FakeState::default()));
    let clock = Arc::new(FakeClock::new());
    let mut supervisor = fake_supervisor(Arc::clone(&state), Arc::clone(&clock), Duration::ZERO);
    let valid = make_journal(2, Some(0));
    let incoming = commands(&valid);
    supervisor
        .install_session("Demo", valid)
        .expect("valid session");
    let old_journal_digest = supervisor.journal_cache_digest();

    let mut malformed = Journal::new();
    malformed
        .record(Entry {
            command: command(99),
            effect: EffectClass::Pure,
            reads: Vec::new(),
            subprocesses: Vec::new(),
            checkpoint: Some(b"malformed checkpoint".to_vec()),
            state_hash: sha256(b"different bytes"),
        })
        .expect("journal entry storage reserves");
    assert!(matches!(
        supervisor.install_session("Broken", malformed),
        Err(fmn_studio::SupervisorError::InvalidSession(
            "checkpoint bytes do not match their state hash"
        ))
    ));
    assert_eq!(supervisor.journal_cache_digest(), old_journal_digest);

    let mut builder = ScriptedBuilder::fake(clock, Duration::ZERO);
    supervisor.build_and_start(&mut builder).expect("start");
    let report = supervisor
        .rebuild_and_restart(&mut builder, &incoming, &|_| true)
        .expect("old session still recovers");
    assert_eq!(report.plan.reuse, 2);
    assert_eq!(report.restored_checkpoint, Some(0));
}

#[test]
fn session_install_refuses_unrecoverable_protocol_payloads_atomically() {
    let oversized_checkpoint = b"12345".to_vec();
    let mut checkpoint_journal = Journal::new();
    checkpoint_journal
        .record(Entry {
            command: command(1),
            effect: EffectClass::Pure,
            reads: Vec::new(),
            subprocesses: Vec::new(),
            checkpoint: Some(oversized_checkpoint.clone()),
            state_hash: sha256(&oversized_checkpoint),
        })
        .expect("journal entry storage reserves");
    let checkpoint_journal_len = checkpoint_journal
        .to_bytes()
        .expect("checkpoint journal encodes")
        .len();
    let max_field_bytes = checkpoint_journal_len
        .checked_add(64)
        .expect("fixture field budget");
    let limits = ProtocolLimits {
        max_field_bytes,
        max_journal_bytes: checkpoint_journal_len,
        max_checkpoint_bytes: oversized_checkpoint.len() - 1,
        ..ProtocolLimits::default()
    };

    let state = Arc::new(Mutex::new(FakeState::default()));
    let clock = Arc::new(FakeClock::new());
    let mut supervisor = Supervisor::new(
        Box::new(FakeLauncher {
            state: Arc::clone(&state),
            clock: Arc::clone(&clock),
            exchange_cost: Duration::ZERO,
        }),
        clock.clone(),
        cache(clock.clone()),
        SupervisorConfig {
            protocol_limits: limits,
            ..SupervisorConfig::default()
        },
    );
    supervisor
        .install_session("Demo", Journal::new())
        .expect("boundary-compatible prior session");
    let prior_journal_digest = supervisor.journal_cache_digest();

    let oversized_scene = "S".repeat(max_field_bytes + 1);
    assert!(matches!(
        supervisor.install_session(oversized_scene, Journal::new()),
        Err(fmn_studio::SupervisorError::InvalidSession(
            "scene name exceeds protocol field budget"
        ))
    ));
    assert_eq!(supervisor.journal_cache_digest(), prior_journal_digest);

    let mut oversized_journal = checkpoint_journal.clone();
    oversized_journal
        .record(Entry {
            command: command(2),
            effect: EffectClass::Pure,
            reads: Vec::new(),
            subprocesses: Vec::new(),
            checkpoint: None,
            state_hash: sha256(b"second state"),
        })
        .expect("journal entry storage reserves");
    assert!(
        oversized_journal
            .to_bytes()
            .expect("oversized journal encodes")
            .len()
            > limits.max_journal_bytes
    );
    assert!(matches!(
        supervisor.install_session("Next", oversized_journal),
        Err(fmn_studio::SupervisorError::InvalidSession(
            "journal exceeds protocol payload budget"
        ))
    ));
    assert_eq!(supervisor.journal_cache_digest(), prior_journal_digest);

    assert_eq!(
        checkpoint_journal
            .to_bytes()
            .expect("boundary journal encodes")
            .len(),
        limits.max_journal_bytes
    );
    assert!(matches!(
        supervisor.install_session("Next", checkpoint_journal.clone()),
        Err(fmn_studio::SupervisorError::InvalidSession(
            "checkpoint exceeds protocol payload budget"
        ))
    ));
    assert_eq!(supervisor.journal_cache_digest(), prior_journal_digest);

    state
        .lock()
        .unwrap_or_else(lock_poisoned)
        .channel_error_first_play = true;
    let mut builder = ScriptedBuilder::fake(clock, Duration::ZERO);
    supervisor.build_and_start(&mut builder).expect("start");
    let reply = supervisor
        .request(
            SupervisorRequest::Play {
                scene: "Demo".to_owned(),
                command: command(0),
            },
            &|_| true,
        )
        .expect("prior session still recovers");
    let SupervisorReply::Recovered { crash, recovery } = reply else {
        std::panic::panic_any("expected recovery from the preserved session");
    };
    assert_eq!(crash.scene.as_deref(), Some("Demo"));
    assert_eq!(recovery.plan.reuse, 0);

    let boundary_state = Arc::new(Mutex::new(FakeState::default()));
    let boundary_clock = Arc::new(FakeClock::new());
    let boundary_limits = ProtocolLimits {
        max_checkpoint_bytes: oversized_checkpoint.len(),
        ..limits
    };
    let mut boundary_supervisor = Supervisor::new(
        Box::new(FakeLauncher {
            state: boundary_state,
            clock: Arc::clone(&boundary_clock),
            exchange_cost: Duration::ZERO,
        }),
        boundary_clock.clone(),
        cache(boundary_clock),
        SupervisorConfig {
            protocol_limits: boundary_limits,
            ..SupervisorConfig::default()
        },
    );
    boundary_supervisor
        .install_session("S".repeat(max_field_bytes), checkpoint_journal)
        .expect("exact scene, journal, and checkpoint budgets are admitted");
}

#[test]
fn worker_checkpoint_must_match_the_installed_journal_authority() {
    let journal = make_journal(2, Some(0));
    let incoming = commands(&journal);
    let original_checkpoint = journal.entries()[0]
        .checkpoint
        .clone()
        .expect("journal checkpoint");
    let divergent_state = b"internally valid but not the journal state".to_vec();
    let state = Arc::new(Mutex::new(FakeState {
        checkpoint_response: Some(Checkpoint {
            scene: "Demo".to_owned(),
            after_entry: 0,
            state_hash: sha256(&divergent_state),
            state: divergent_state,
        }),
        ..FakeState::default()
    }));
    let clock = Arc::new(FakeClock::new());
    let mut supervisor = fake_supervisor(Arc::clone(&state), Arc::clone(&clock), Duration::ZERO);
    supervisor
        .install_session("Demo", journal)
        .expect("session");
    let mut builder = ScriptedBuilder::fake(clock, Duration::ZERO);
    supervisor.build_and_start(&mut builder).expect("start");

    assert!(matches!(
        supervisor.request(SupervisorRequest::EnumerateScenes, &|_| true),
        Err(fmn_studio::SupervisorError::InvalidSession(
            "worker checkpoint does not match the journal entry state hash"
        ))
    ));

    let report = supervisor
        .rebuild_and_restart(&mut builder, &incoming, &|_| true)
        .expect("known-good checkpoint remains authoritative");
    assert_eq!(report.restored_checkpoint, Some(0));
    assert_eq!(
        state.lock().unwrap_or_else(lock_poisoned).restored.last(),
        Some(&original_checkpoint)
    );
}

#[cfg(unix)]
#[test]
fn stdio_worker_deadline_cancels_a_blocked_checkpoint_write() {
    use std::path::Path;

    use fmn_studio::StdWorkerLauncher;

    let sleep = Path::new("/bin/sleep");
    if !sleep.is_file() {
        return;
    }
    let mut launcher = StdWorkerLauncher::default();
    let limits = ProtocolLimits {
        max_checkpoint_bytes: 2 * 1024 * 1024,
        max_message_bytes: 3 * 1024 * 1024,
        ..ProtocolLimits::default()
    };
    let artifact = WorkerArtifact {
        executable: sleep.to_path_buf(),
        argv: vec!["10".to_owned()],
        env: Vec::new(),
        cwd: None,
        build_id: sha256(b"unresponsive test worker"),
    };
    let mut channel = launcher
        .launch(&artifact, limits)
        .expect("sleep fixture launches");
    let state = vec![0x5a; 1024 * 1024];
    let request = RequestEnvelope {
        request_id: 1,
        request: SupervisorRequest::RestoreCheckpoint(Checkpoint {
            scene: "Demo".to_owned(),
            after_entry: 0,
            state_hash: sha256(&state),
            state,
        }),
    };

    let started = Instant::now();
    let error = channel
        .exchange(&request, Duration::from_millis(100))
        .expect_err("unresponsive worker must hit its deadline");
    assert_eq!(error.kind, ChannelFailureKind::Timeout);
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "blocked pipe write outlived cancellation budget"
    );
}

#[test]
fn journal_segment_replaces_only_its_declared_tail() {
    let replacement_command = command(99);
    let mut replacement = Journal::new();
    let segment_backend = RenderBackendRecord::new(
        RenderBackendRole::NativePresentation,
        b"native presentation backend".to_vec(),
    )
    .unwrap();
    replacement
        .record(Entry {
            command: replacement_command.clone(),
            effect: EffectClass::Pure,
            reads: Vec::new(),
            subprocesses: Vec::new(),
            checkpoint: None,
            state_hash: sha256(b"replacement tail state"),
        })
        .expect("journal entry storage reserves");
    let segment_event = InputEvent::new(
        2,
        RationalTime::zero(30) + 1,
        EventPayload::KeyRelease {
            key: Key::Character('s'),
            modifiers: Modifiers::NONE,
        },
    )
    .expect("segment event");
    replacement
        .record_event(segment_event.clone())
        .expect("segment event records");
    replacement
        .record_render_backend(&segment_backend)
        .expect("segment backend records");
    let state = Arc::new(Mutex::new(FakeState {
        journal_segment: Some((1, replacement.to_bytes().expect("segment encodes"))),
        ..FakeState::default()
    }));
    let clock = Arc::new(FakeClock::new());
    let mut supervisor = fake_supervisor(Arc::clone(&state), Arc::clone(&clock), Duration::ZERO);
    let mut original = make_journal(2, Some(0));
    let original_backend = RenderBackendRecord::new(
        RenderBackendRole::FrameStream,
        b"fast-cpu stream backend".to_vec(),
    )
    .unwrap();
    let original_event = InputEvent::new(
        1,
        RationalTime::zero(30),
        EventPayload::KeyPress {
            key: Key::Character('s'),
            modifiers: Modifiers::NONE,
        },
    )
    .expect("original event");
    original
        .record_event(original_event.clone())
        .expect("original event records");
    original
        .record_render_backend(&original_backend)
        .expect("original backend records");
    let first = original.entries()[0].command.clone();
    supervisor
        .install_session("Demo", original)
        .expect("session");
    let mut builder = ScriptedBuilder::fake(clock, Duration::ZERO);
    supervisor.build_and_start(&mut builder).expect("start");
    assert!(matches!(
        supervisor
            .request(SupervisorRequest::EnumerateScenes, &|_| true)
            .expect("segment accepted"),
        SupervisorReply::Worker(WorkerResponse::JournalSegment { start_entry: 1, .. })
    ));
    assert_eq!(
        supervisor.render_backends(),
        &[original_backend, segment_backend],
        "tail replacement must preserve both observed backend identities"
    );

    let report = supervisor
        .rebuild_and_restart(&mut builder, &[first, replacement_command], &|_| true)
        .expect("merged journal replays");
    assert_eq!(report.plan.reuse, 2);
    assert_eq!(report.restored_checkpoint, Some(0));
    assert_eq!(report.replayed_entries, 1);
    assert_eq!(
        state.lock().unwrap_or_else(lock_poisoned).replayed_events,
        vec![original_event, segment_event],
        "the retained and segment-local input streams must both reach replay"
    );
}

#[test]
fn accepted_frames_update_the_durable_session_backend_identity_once() {
    let backend = RenderBackendRecord::new(
        RenderBackendRole::FrameStream,
        b"canonical fast-cpu engine journal".to_vec(),
    )
    .unwrap();
    let pixels = vec![0, 0, 0, 255];
    let frame = FrameStream {
        scene: "Demo".to_owned(),
        frame_index: 0,
        width: 1,
        height: 1,
        stride: 4,
        encoding: FrameEncoding::Rgba8,
        payload: FramePayload::Pipe {
            digest: sha256(&pixels),
            bytes: pixels,
        },
        render_backends: vec![backend.clone()],
    };
    let state = Arc::new(Mutex::new(FakeState {
        frame_response: Some(frame.clone()),
        ..FakeState::default()
    }));
    let clock = Arc::new(FakeClock::new());
    let mut supervisor = fake_supervisor(Arc::clone(&state), Arc::clone(&clock), Duration::ZERO);
    supervisor
        .install_session("Demo", Journal::new())
        .expect("session");
    let empty_digest = supervisor.journal_cache_digest();
    let mut builder = ScriptedBuilder::fake(clock, Duration::ZERO);
    supervisor.build_and_start(&mut builder).expect("start");

    assert!(matches!(
        supervisor
            .request(SupervisorRequest::EnumerateScenes, &|_| true)
            .expect("frame accepted"),
        SupervisorReply::Worker(WorkerResponse::Frame(_))
    ));
    assert_eq!(supervisor.render_backends(), std::slice::from_ref(&backend));
    let recorded_digest = supervisor.journal_cache_digest();
    assert_ne!(recorded_digest, empty_digest);

    state.lock().unwrap_or_else(lock_poisoned).frame_response = Some(frame);
    supervisor
        .request(SupervisorRequest::EnumerateScenes, &|_| true)
        .expect("duplicate backend frame accepted");
    assert_eq!(supervisor.render_backends(), std::slice::from_ref(&backend));
    assert_eq!(
        supervisor.journal_cache_digest(),
        recorded_digest,
        "a repeated identity must not rewrite or grow the durable journal"
    );
}

#[test]
fn journal_segment_rejects_out_of_order_events_without_replacing_the_session() {
    let original_event = InputEvent::new(
        2,
        RationalTime::zero(30) + 1,
        EventPayload::KeyPress {
            key: Key::Character('s'),
            modifiers: Modifiers::NONE,
        },
    )
    .expect("original event");
    let mut original = make_journal(2, Some(0));
    original
        .record_event(original_event.clone())
        .expect("original event records");

    let mut replacement = make_journal(1, None);
    replacement
        .record_event(
            InputEvent::new(
                1,
                RationalTime::zero(30) + 2,
                EventPayload::KeyRelease {
                    key: Key::Character('s'),
                    modifiers: Modifiers::NONE,
                },
            )
            .expect("out-of-order segment event"),
        )
        .expect("segment-local stream is valid on its own");

    let state = Arc::new(Mutex::new(FakeState {
        journal_segment: Some((1, replacement.to_bytes().expect("segment encodes"))),
        ..FakeState::default()
    }));
    let clock = Arc::new(FakeClock::new());
    let mut supervisor = fake_supervisor(Arc::clone(&state), Arc::clone(&clock), Duration::ZERO);
    let incoming = commands(&original);
    supervisor
        .install_session("Demo", original)
        .expect("session");
    let prior_digest = supervisor.journal_cache_digest();
    let mut builder = ScriptedBuilder::fake(clock, Duration::ZERO);
    supervisor.build_and_start(&mut builder).expect("start");

    let error = supervisor
        .request(SupervisorRequest::EnumerateScenes, &|_| true)
        .expect_err("an out-of-order segment event must be refused");
    assert!(matches!(
        &error,
        SupervisorError::InvalidJournal(JournalError::Event(EventError::ReplayOutOfOrder))
    ));
    assert!(std::error::Error::source(&error).is_some());
    assert_eq!(supervisor.journal_cache_digest(), prior_digest);

    let report = supervisor
        .rebuild_and_restart(&mut builder, &incoming, &|_| true)
        .expect("the prior session remains recoverable");
    assert_eq!(report.plan.reuse, 2);
    assert_eq!(
        state.lock().unwrap_or_else(lock_poisoned).replayed_events,
        vec![original_event],
        "a refused segment must leave the installed event stream unchanged"
    );
}

struct PanicService {
    build_id: Digest,
    negotiated_frame_budget: Option<usize>,
    panic_message: Option<String>,
    active_scene: Option<String>,
    journal_tail: Vec<u8>,
}

struct RefusalService {
    build_id: Digest,
    session_error: Option<ServiceError>,
    request_error: Option<ServiceError>,
}

impl WorkerService for RefusalService {
    fn build_id(&self) -> Digest {
        self.build_id
    }

    fn begin_session(
        &mut self,
        _supervisor_build: Digest,
        _max_frame_bytes: usize,
    ) -> Result<(), ServiceError> {
        match self.session_error.take() {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    fn handle(&mut self, _request: SupervisorRequest) -> Result<WorkerResponse, ServiceError> {
        match self.request_error.take() {
            Some(error) => Err(error),
            None => Err(ServiceError::new(
                WorkerErrorCode::InvalidRequest,
                "fixture refusal was not configured",
            )),
        }
    }
}

impl WorkerService for PanicService {
    fn build_id(&self) -> Digest {
        self.build_id
    }

    fn begin_session(
        &mut self,
        _supervisor_build: Digest,
        max_frame_bytes: usize,
    ) -> Result<(), ServiceError> {
        self.negotiated_frame_budget = Some(max_frame_bytes);
        Ok(())
    }

    fn handle(&mut self, _request: SupervisorRequest) -> Result<WorkerResponse, ServiceError> {
        if let Some(message) = self.panic_message.take() {
            // ubs:ignore - deliberate fixture panic exercises bounded crash reporting.
            std::panic::panic_any(message);
        }
        // ubs:ignore - deliberate fixture panic exercises worker isolation.
        std::panic::panic_any("fixture scene panic")
    }

    fn active_scene(&self) -> Option<&str> {
        self.active_scene.as_deref()
    }

    fn journal_tail(&self) -> &[u8] {
        &self.journal_tail
    }

    fn last_state_hash(&self) -> Option<Digest> {
        Some(sha256(b"last state"))
    }
}

#[test]
fn worker_loop_converts_scene_panic_to_structured_correlated_report() {
    let limits = ProtocolLimits::default();
    let build_id = sha256(b"panic worker");
    let mut input = Vec::new();
    write_request(
        &mut input,
        &RequestEnvelope {
            request_id: 1,
            request: SupervisorRequest::Hello {
                version: fmn_studio::CURRENT_VERSION,
                supervisor_build: sha256(b"supervisor"),
                max_frame_bytes: 1024,
            },
        },
        limits,
    )
    .expect("hello");
    write_request(
        &mut input,
        &RequestEnvelope {
            request_id: 2,
            request: SupervisorRequest::EnumerateScenes,
        },
        limits,
    )
    .expect("command");
    let mut output = Vec::new();
    let mut service = PanicService {
        build_id,
        negotiated_frame_budget: None,
        panic_message: None,
        active_scene: Some("Demo".to_owned()),
        journal_tail: b"canonical journal tail".to_vec(),
    };
    let source_scene_allocation = service
        .active_scene
        .as_deref()
        .expect("fixture scene")
        .as_ptr();
    let source_tail_allocation = service.journal_tail.as_ptr();
    let outcome = serve_worker(
        &mut service,
        &mut std::io::Cursor::new(input),
        &mut output,
        limits,
    )
    .expect("worker loop reports panic");
    let WorkerServeOutcome::Crashed(report) = outcome else {
        // ubs:ignore - deliberate assertion failure in a crash-report test.
        std::panic::panic_any("expected crash outcome");
    };
    assert_eq!(report.scene.as_deref(), Some("Demo"));
    assert_eq!(report.message, "fixture scene panic");
    assert_eq!(report.journal_tail, b"canonical journal tail");
    assert_ne!(report.message.as_ptr(), "fixture scene panic".as_ptr());
    assert_ne!(
        report.scene.as_deref().expect("reported scene").as_ptr(),
        source_scene_allocation
    );
    assert_ne!(report.journal_tail.as_ptr(), source_tail_allocation);
    assert_eq!(service.negotiated_frame_budget, Some(1024));

    let mut output = std::io::Cursor::new(output);
    assert!(matches!(
        read_response(&mut output, limits)
            .expect("hello response")
            .response,
        WorkerResponse::Hello { worker_build, .. } if worker_build == build_id // ubs:ignore - public build identity.
    ));
    let crash = read_response(&mut output, limits).expect("crash response");
    assert_eq!(crash.request_id, 2);
    assert_eq!(crash.response, WorkerResponse::Crash(report));
}

#[test]
fn worker_bounds_owned_panic_message_before_crash_envelope_validation() {
    let limits = ProtocolLimits {
        max_crash_message_bytes: 7,
        ..ProtocolLimits::default()
    };
    let build_id = sha256(b"bounded panic worker");
    let mut input = Vec::new();
    write_request(
        &mut input,
        &RequestEnvelope {
            request_id: 1,
            request: SupervisorRequest::Hello {
                version: fmn_studio::CURRENT_VERSION,
                supervisor_build: sha256(b"supervisor"),
                max_frame_bytes: 1024,
            },
        },
        limits,
    )
    .expect("hello");
    write_request(
        &mut input,
        &RequestEnvelope {
            request_id: 2,
            request: SupervisorRequest::EnumerateScenes,
        },
        limits,
    )
    .expect("command");

    let panic_message = "éééé".to_owned();
    let panic_message_allocation = panic_message.as_ptr();
    let mut output = Vec::new();
    let mut service = PanicService {
        build_id,
        negotiated_frame_budget: None,
        panic_message: Some(panic_message),
        active_scene: Some("Demo".to_owned()),
        journal_tail: b"canonical journal tail".to_vec(),
    };
    let outcome = serve_worker(
        &mut service,
        &mut std::io::Cursor::new(input),
        &mut output,
        limits,
    )
    .expect("worker emits a bounded crash report");
    let WorkerServeOutcome::Crashed(report) = outcome else {
        std::panic::panic_any("expected crash outcome");
    };
    assert_eq!(report.message, "ééé");
    assert_eq!(report.message.len(), 6);
    assert_eq!(report.message.as_ptr(), panic_message_allocation);

    let mut output = std::io::Cursor::new(output);
    let hello = read_response(&mut output, limits).expect("hello response");
    assert!(matches!(hello.response, WorkerResponse::Hello { .. }));
    let crash = read_response(&mut output, limits).expect("crash response");
    assert_eq!(crash.request_id, 2);
    assert_eq!(crash.response, WorkerResponse::Crash(report));
}

#[test]
fn worker_refuses_an_impossible_zero_crash_message_budget_precisely() {
    let limits = ProtocolLimits {
        max_crash_message_bytes: 0,
        ..ProtocolLimits::default()
    };
    let build_id = sha256(b"zero-budget panic worker");
    let mut input = Vec::new();
    write_request(
        &mut input,
        &RequestEnvelope {
            request_id: 1,
            request: SupervisorRequest::Hello {
                version: fmn_studio::CURRENT_VERSION,
                supervisor_build: sha256(b"supervisor"),
                max_frame_bytes: 1024,
            },
        },
        limits,
    )
    .expect("hello");
    write_request(
        &mut input,
        &RequestEnvelope {
            request_id: 2,
            request: SupervisorRequest::EnumerateScenes,
        },
        limits,
    )
    .expect("command");

    let mut output = Vec::new();
    let mut service = PanicService {
        build_id,
        negotiated_frame_budget: None,
        panic_message: None,
        active_scene: Some("Demo".to_owned()),
        journal_tail: b"canonical journal tail".to_vec(),
    };
    let error = serve_worker(
        &mut service,
        &mut std::io::Cursor::new(input),
        &mut output,
        limits,
    )
    .expect_err("a nonempty panic diagnostic cannot fit a zero-byte wire budget");
    assert_eq!(
        error.to_string(),
        "IPC crash message payload 1 bytes exceeds the configured limit 0"
    );
    assert!(std::error::Error::source(&error).is_some());
    assert!(matches!(
        error,
        WorkerServeError::CrashReport(ProtocolError::PayloadLimit {
            field: "crash message",
            limit: 0,
            needed: 1,
        })
    ));
}

#[test]
fn worker_refuses_an_impossible_zero_error_message_budget_precisely() {
    let limits = ProtocolLimits {
        max_error_message_bytes: 0,
        ..ProtocolLimits::default()
    };
    let mut input = Vec::new();
    write_request(
        &mut input,
        &RequestEnvelope {
            request_id: 1,
            request: SupervisorRequest::EnumerateScenes,
        },
        limits,
    )
    .expect("non-Hello request");

    let mut output = Vec::new();
    let mut service = PanicService {
        build_id: sha256(b"zero-budget refusal worker"),
        negotiated_frame_budget: None,
        panic_message: None,
        active_scene: None,
        journal_tail: Vec::new(),
    };
    let error = serve_worker(
        &mut service,
        &mut std::io::Cursor::new(input),
        &mut output,
        limits,
    )
    .expect_err("a nonempty worker refusal cannot fit a zero-byte wire budget");
    assert_eq!(
        error.to_string(),
        "IPC worker error payload 1 bytes exceeds the configured limit 0"
    );
    assert!(std::error::Error::source(&error).is_some());
    assert!(matches!(
        error,
        WorkerServeError::ErrorResponse(ProtocolError::PayloadLimit {
            field: "worker error",
            limit: 0,
            needed: 1,
        })
    ));
    assert!(output.is_empty(), "no malformed refusal may be emitted");
}

#[test]
fn worker_bounds_crash_context_to_the_generic_field_budget() {
    let limits = ProtocolLimits {
        max_field_bytes: 5,
        max_crash_message_bytes: 32,
        max_crash_tail_bytes: 32,
        ..ProtocolLimits::default()
    };
    let build_id = sha256(b"field-bounded crash worker");
    let mut input = Vec::new();
    write_request(
        &mut input,
        &RequestEnvelope {
            request_id: 1,
            request: SupervisorRequest::Hello {
                version: fmn_studio::CURRENT_VERSION,
                supervisor_build: sha256(b"supervisor"),
                max_frame_bytes: 1024,
            },
        },
        limits,
    )
    .expect("hello");
    write_request(
        &mut input,
        &RequestEnvelope {
            request_id: 2,
            request: SupervisorRequest::EnumerateScenes,
        },
        limits,
    )
    .expect("command");

    let mut output = Vec::new();
    let mut service = PanicService {
        build_id,
        negotiated_frame_budget: None,
        panic_message: Some("ééé".to_owned()),
        active_scene: Some("Demo".to_owned()),
        journal_tail: b"abcdef".to_vec(),
    };
    let outcome = serve_worker(
        &mut service,
        &mut std::io::Cursor::new(input),
        &mut output,
        limits,
    )
    .expect("worker emits field-bounded crash context");
    let WorkerServeOutcome::Crashed(report) = outcome else {
        std::panic::panic_any("expected crash outcome");
    };
    assert_eq!(report.message, "éé");
    assert_eq!(report.journal_tail, b"bcdef");

    let mut output = std::io::Cursor::new(output);
    assert!(matches!(
        read_response(&mut output, limits)
            .expect("hello response")
            .response,
        WorkerResponse::Hello { .. }
    ));
    let crash = read_response(&mut output, limits).expect("crash response");
    assert_eq!(crash.request_id, 2);
    assert_eq!(crash.response, WorkerResponse::Crash(report));
}

#[test]
fn worker_omits_invalid_scene_and_copies_the_bounded_journal_suffix() {
    let limits = ProtocolLimits {
        max_crash_tail_bytes: 3,
        ..ProtocolLimits::default()
    };
    let build_id = sha256(b"bounded crash context worker");
    let mut input = Vec::new();
    write_request(
        &mut input,
        &RequestEnvelope {
            request_id: 1,
            request: SupervisorRequest::Hello {
                version: fmn_studio::CURRENT_VERSION,
                supervisor_build: sha256(b"supervisor"),
                max_frame_bytes: 1024,
            },
        },
        limits,
    )
    .expect("hello");
    write_request(
        &mut input,
        &RequestEnvelope {
            request_id: 2,
            request: SupervisorRequest::EnumerateScenes,
        },
        limits,
    )
    .expect("command");

    let mut output = Vec::new();
    let mut service = PanicService {
        build_id,
        negotiated_frame_budget: None,
        panic_message: None,
        active_scene: Some(String::new()),
        journal_tail: b"abcdef".to_vec(),
    };
    let outcome = serve_worker(
        &mut service,
        &mut std::io::Cursor::new(input),
        &mut output,
        limits,
    )
    .expect("worker emits a valid crash report despite invalid optional context");
    let WorkerServeOutcome::Crashed(report) = outcome else {
        std::panic::panic_any("expected crash outcome");
    };
    assert_eq!(report.scene, None);
    assert_eq!(report.journal_tail, b"def");

    let mut output = std::io::Cursor::new(output);
    let hello = read_response(&mut output, limits).expect("hello response");
    assert!(matches!(hello.response, WorkerResponse::Hello { .. }));
    let crash = read_response(&mut output, limits).expect("crash response");
    assert_eq!(crash.request_id, 2);
    assert_eq!(crash.response, WorkerResponse::Crash(report));
}

#[test]
fn worker_bounds_session_and_request_refusals_before_framing() {
    let limits = ProtocolLimits {
        max_field_bytes: 5,
        ..ProtocolLimits::default()
    };
    let supervisor_build = sha256(b"supervisor");

    let mut session_input = Vec::new();
    write_request(
        &mut session_input,
        &RequestEnvelope {
            request_id: 1,
            request: SupervisorRequest::Hello {
                version: fmn_studio::CURRENT_VERSION,
                supervisor_build,
                max_frame_bytes: 1024,
            },
        },
        limits,
    )
    .expect("session hello");
    let mut session_output = Vec::new();
    let mut session_service = RefusalService {
        build_id: sha256(b"session refusal worker"),
        session_error: Some(ServiceError::new(WorkerErrorCode::InvalidRequest, "abcdef")),
        request_error: None,
    };
    assert_eq!(
        serve_worker(
            &mut session_service,
            &mut std::io::Cursor::new(session_input),
            &mut session_output,
            limits,
        )
        .expect("session refusal remains structured"),
        WorkerServeOutcome::HandshakeRejected
    );
    assert_eq!(
        read_response(&mut std::io::Cursor::new(session_output), limits)
            .expect("session refusal")
            .response,
        WorkerResponse::Error {
            code: WorkerErrorCode::InvalidRequest,
            message: "abcde".to_owned(),
        }
    );

    let mut request_input = Vec::new();
    write_request(
        &mut request_input,
        &RequestEnvelope {
            request_id: 1,
            request: SupervisorRequest::Hello {
                version: fmn_studio::CURRENT_VERSION,
                supervisor_build,
                max_frame_bytes: 1024,
            },
        },
        limits,
    )
    .expect("request hello");
    write_request(
        &mut request_input,
        &RequestEnvelope {
            request_id: 2,
            request: SupervisorRequest::EnumerateScenes,
        },
        limits,
    )
    .expect("request command");
    let mut request_output = Vec::new();
    let mut request_service = RefusalService {
        build_id: sha256(b"request refusal worker"),
        session_error: None,
        request_error: Some(ServiceError::new(WorkerErrorCode::ReplayFailed, "ééé")),
    };
    assert_eq!(
        serve_worker(
            &mut request_service,
            &mut std::io::Cursor::new(request_input),
            &mut request_output,
            limits,
        )
        .expect("request refusal remains structured"),
        WorkerServeOutcome::PeerClosed
    );
    let mut request_output = std::io::Cursor::new(request_output);
    assert!(matches!(
        read_response(&mut request_output, limits)
            .expect("worker hello")
            .response,
        WorkerResponse::Hello { .. }
    ));
    assert_eq!(
        read_response(&mut request_output, limits)
            .expect("request refusal")
            .response,
        WorkerResponse::Error {
            code: WorkerErrorCode::ReplayFailed,
            message: "éé".to_owned(),
        }
    );
}

#[test]
fn worker_loop_rejects_reserved_and_nonmonotonic_request_ids() {
    let limits = ProtocolLimits {
        max_error_message_bytes: 5,
        ..ProtocolLimits::default()
    };
    let build_id = sha256(b"request-id worker");
    let hello = |request_id| RequestEnvelope {
        request_id,
        request: SupervisorRequest::Hello {
            version: fmn_studio::CURRENT_VERSION,
            supervisor_build: sha256(b"supervisor"),
            max_frame_bytes: 1024,
        },
    };

    let mut zero_input = Vec::new();
    write_request(&mut zero_input, &hello(0), limits).expect("zero-id hello encodes");
    let mut zero_output = Vec::new();
    let mut service = PanicService {
        build_id,
        negotiated_frame_budget: None,
        panic_message: None,
        active_scene: Some("Demo".to_owned()),
        journal_tail: b"canonical journal tail".to_vec(),
    };
    assert_eq!(
        serve_worker(
            &mut service,
            &mut std::io::Cursor::new(zero_input),
            &mut zero_output,
            limits,
        )
        .expect("zero id is a typed refusal"),
        WorkerServeOutcome::HandshakeRejected
    );
    assert_eq!(
        read_response(&mut std::io::Cursor::new(zero_output), limits)
            .expect("zero-id refusal")
            .response,
        WorkerResponse::Error {
            code: WorkerErrorCode::InvalidRequest,
            message: "reque".to_owned(),
        }
    );

    let mut repeated_input = Vec::new();
    write_request(&mut repeated_input, &hello(1), limits).expect("hello");
    write_request(
        &mut repeated_input,
        &RequestEnvelope {
            request_id: 1,
            request: SupervisorRequest::EnumerateScenes,
        },
        limits,
    )
    .expect("repeated request id encodes");
    let mut repeated_output = Vec::new();
    assert_eq!(
        serve_worker(
            &mut service,
            &mut std::io::Cursor::new(repeated_input),
            &mut repeated_output,
            limits,
        )
        .expect("nonmonotonic id is a typed refusal"),
        WorkerServeOutcome::HandshakeRejected
    );
    let mut repeated_output = std::io::Cursor::new(repeated_output);
    assert!(matches!(
        read_response(&mut repeated_output, limits)
            .expect("hello response")
            .response,
        WorkerResponse::Hello { .. }
    ));
    assert_eq!(
        read_response(&mut repeated_output, limits)
            .expect("nonmonotonic refusal")
            .response,
        WorkerResponse::Error {
            code: WorkerErrorCode::InvalidRequest,
            message: "reque".to_owned(),
        }
    );
}

// ---------------------------------------------------------------------------
// Real process-isolation acceptance.  The test harness's stdout is irrelevant:
// control messages use a private loopback socket solely because stable Rust
// does not expose arbitrary inherited file descriptors portably.  Production
// `StdWorkerLauncher` uses the bounded stdin/stdout pipe implementation.
// ---------------------------------------------------------------------------

struct TcpLauncher {
    launches: Arc<AtomicUsize>,
    crash_first: bool,
}

impl WorkerLauncher for TcpLauncher {
    fn launch(
        &mut self,
        artifact: &WorkerArtifact,
        _limits: ProtocolLimits,
    ) -> Result<Box<dyn WorkerChannel>, LaunchError> {
        let launch = self.launches.fetch_add(1, Ordering::SeqCst) + 1;
        let listener = TcpListener::bind("127.0.0.1:0").map_err(|error| LaunchError::Spawn {
            program: artifact.executable.clone(),
            error,
        })?;
        listener
            .set_nonblocking(true)
            .map_err(|error| LaunchError::Spawn {
                program: artifact.executable.clone(),
                error,
            })?;
        let address = listener.local_addr().map_err(|error| LaunchError::Spawn {
            program: artifact.executable.clone(),
            error,
        })?;
        let mut command = std::process::Command::new(&artifact.executable);
        command
            .arg("--exact")
            .arg("subprocess_worker_entry")
            .arg("--nocapture")
            .env("FMN_STUDIO_CHILD", "1")
            .env("FMN_STUDIO_ADDR", address.to_string())
            .env("FMN_STUDIO_BUILD_ID", artifact.build_id.to_hex())
            .env(
                "FMN_STUDIO_CRASH",
                if self.crash_first && launch == 1 {
                    "1"
                } else {
                    "0"
                },
            )
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        let mut child = command.spawn().map_err(|error| LaunchError::Spawn {
            program: artifact.executable.clone(),
            error,
        })?;
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    // Darwin carries the listener's nonblocking mode onto the
                    // accepted socket. The channel below is intentionally
                    // blocking and enforces its own read/write deadlines.
                    if let Err(error) = stream.set_nonblocking(false) {
                        let _ = child.kill();
                        let _ = child.wait();
                        return Err(LaunchError::Spawn {
                            program: artifact.executable.clone(),
                            error,
                        });
                    }
                    return Ok(Box::new(TcpChannel {
                        stream,
                        child: Some(child),
                    }));
                }
                // ubs:ignore - I/O error kinds are public enum values, not secrets.
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if let Ok(Some(_)) = child.try_wait() {
                        return Err(LaunchError::InvalidArtifact(
                            "test worker exited before connecting",
                        ));
                    }
                    if Instant::now() >= deadline {
                        let _ = child.kill();
                        let _ = child.wait();
                        return Err(LaunchError::InvalidArtifact(
                            "test worker did not connect before deadline",
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(LaunchError::Spawn {
                        program: artifact.executable.clone(),
                        error,
                    });
                }
            }
        }
    }
}

struct TcpChannel {
    stream: TcpStream,
    child: Option<Child>,
}

impl TcpChannel {
    fn map(error: FramingError) -> ChannelError {
        let kind = match &error {
            FramingError::Closed => ChannelFailureKind::Closed,
            FramingError::Io(_)
            | FramingError::FrameStorageAllocationFailed { .. }
            | FramingError::InvalidReadCount { .. } => ChannelFailureKind::Io,
            FramingError::FrameTooLarge { .. } | FramingError::Protocol(_) => {
                ChannelFailureKind::Protocol
            }
        };
        ChannelError {
            kind,
            detail: error.to_string(),
            stderr_tail: Vec::new(),
        }
    }
}

impl WorkerChannel for TcpChannel {
    fn exchange(
        &mut self,
        request: &RequestEnvelope,
        timeout: Duration,
    ) -> Result<ResponseEnvelope, ChannelError> {
        self.stream
            .set_read_timeout(Some(timeout))
            .map_err(|error| ChannelError {
                kind: ChannelFailureKind::Io,
                detail: error.to_string(),
                stderr_tail: Vec::new(),
            })?;
        self.stream
            .set_write_timeout(Some(timeout))
            .map_err(|error| ChannelError {
                kind: ChannelFailureKind::Io,
                detail: error.to_string(),
                stderr_tail: Vec::new(),
            })?;
        write_request(&mut self.stream, request, ProtocolLimits::default()).map_err(Self::map)?;
        read_response(&mut self.stream, ProtocolLimits::default()).map_err(Self::map)
    }

    fn terminate(&mut self) {
        let _ = self.stream.shutdown(Shutdown::Both);
        if let Some(mut child) = self.child.take() {
            match child.try_wait() {
                Ok(Some(_)) => {}
                Ok(None) | Err(_) => {
                    let _ = child.kill();
                    let _ = child.wait();
                }
            }
        }
    }
}

impl Drop for TcpChannel {
    fn drop(&mut self) {
        self.terminate();
    }
}

struct ChildService {
    build_id: Digest,
    crash: bool,
}

impl WorkerService for ChildService {
    fn build_id(&self) -> Digest {
        self.build_id
    }

    fn handle(&mut self, request: SupervisorRequest) -> Result<WorkerResponse, ServiceError> {
        match request {
            SupervisorRequest::EnumerateScenes => {
                Ok(WorkerResponse::Scenes(vec!["Demo".to_owned()]))
            }
            SupervisorRequest::Play { .. } if self.crash => {
                self.crash = false;
                // ubs:ignore - deliberate subprocess crash fixture.
                std::panic::panic_any("subprocess scene panic")
            }
            SupervisorRequest::Play { .. }
            | SupervisorRequest::Seek { .. }
            | SupervisorRequest::Scrub { .. }
            | SupervisorRequest::Event { .. }
            | SupervisorRequest::Inspect { .. }
            | SupervisorRequest::Overlay { .. } => Ok(WorkerResponse::Ack {
                state_hash: None,
                journal_len: 0,
            }),
            SupervisorRequest::RestoreCheckpoint(checkpoint) => Ok(WorkerResponse::Ack {
                state_hash: Some(checkpoint.state_hash),
                journal_len: checkpoint.after_entry + 1,
            }),
            SupervisorRequest::ReplayJournal(JournalReplay {
                from_entry,
                through_entry,
                journal,
                ..
            }) => {
                let journal = Journal::from_bytes(&journal).map_err(|error| {
                    ServiceError::new(WorkerErrorCode::ReplayFailed, error.to_string())
                })?;
                let start = usize::try_from(from_entry).map_err(|_| {
                    ServiceError::new(WorkerErrorCode::ReplayFailed, "range overflow")
                })?;
                let end = usize::try_from(through_entry).map_err(|_| {
                    ServiceError::new(WorkerErrorCode::ReplayFailed, "range overflow")
                })?;
                Ok(WorkerResponse::ReplayComplete {
                    from_entry,
                    state_hashes: journal.entries()[start..end]
                        .iter()
                        .map(|entry| entry.state_hash)
                        .collect(),
                })
            }
            SupervisorRequest::Hello { .. } | SupervisorRequest::Shutdown => {
                Err(ServiceError::new(
                    WorkerErrorCode::InvalidRequest,
                    "protocol driver request reached service",
                ))
            }
        }
    }

    fn active_scene(&self) -> Option<&str> {
        Some("Demo")
    }

    fn journal_tail(&self) -> &[u8] {
        b"subprocess journal tail"
    }
}

#[test]
fn subprocess_worker_entry() {
    if std::env::var("FMN_STUDIO_CHILD").as_deref() != Ok("1") {
        return;
    }
    let address = std::env::var("FMN_STUDIO_ADDR").expect("child address");
    let build_id = Digest::from_hex(&std::env::var("FMN_STUDIO_BUILD_ID").expect("child build id"))
        .expect("valid build id");
    let crash = std::env::var("FMN_STUDIO_CRASH").as_deref() == Ok("1");
    let stream = TcpStream::connect(address).expect("connect supervisor");
    let mut reader = stream.try_clone().expect("clone stream");
    let mut writer = stream;
    let outcome = serve_worker(
        &mut ChildService { build_id, crash },
        &mut reader,
        &mut writer,
        ProtocolLimits::default(),
    )
    .expect("serve worker");
    if matches!(outcome, WorkerServeOutcome::Crashed(_)) {
        // ubs:ignore - child exits after its deliberate structured crash.
        std::panic::panic_any("worker process exits after its structured crash report");
    }
}

#[test]
fn real_worker_process_panic_is_isolated_and_auto_restarted() {
    if std::env::var("FMN_STUDIO_CHILD").as_deref() == Ok("1") {
        return;
    }
    let clock = Arc::new(FakeClock::new());
    let launches = Arc::new(AtomicUsize::new(0));
    let mut supervisor = Supervisor::new(
        Box::new(TcpLauncher {
            launches: Arc::clone(&launches),
            crash_first: true,
        }),
        clock.clone(),
        cache(clock.clone()),
        SupervisorConfig {
            request_timeout: Duration::from_secs(3),
            ..SupervisorConfig::default()
        },
    );
    let journal = make_journal(2, Some(0));
    supervisor
        .install_session("Demo", journal)
        .expect("session");
    let mut builder = ScriptedBuilder {
        clock,
        build_cost: Duration::ZERO,
        build: 0,
        executable: std::env::current_exe().expect("current test executable"),
    };
    supervisor
        .build_and_start(&mut builder)
        .expect("start child");
    let reply = supervisor
        .request(
            SupervisorRequest::Play {
                scene: "Demo".to_owned(),
                command: command(1),
            },
            &|_| true,
        )
        .expect("parent survives and recovers");
    let SupervisorReply::Recovered { crash, recovery } = reply else {
        // ubs:ignore - deliberate assertion failure in a process-recovery test.
        std::panic::panic_any("expected process crash recovery");
    };
    assert_eq!(crash.message, "subprocess scene panic");
    assert_eq!(crash.journal_tail, b"subprocess journal tail");
    assert_eq!(recovery.restored_checkpoint, Some(0));
    assert_eq!(supervisor.generation(), 2);
    assert_eq!(launches.load(Ordering::SeqCst), 2);
    assert_eq!(
        supervisor
            .request(SupervisorRequest::EnumerateScenes, &|_| true)
            .expect("replacement child answers"),
        SupervisorReply::Worker(WorkerResponse::Scenes(vec!["Demo".to_owned()]))
    );
}

#[test]
fn saved_session_restores_into_a_new_supervisor_without_the_old_cache() {
    use fmn_studio::supervisor::session::SavedSession;
    let clock = Arc::new(FakeClock::new());
    let state = Arc::new(Mutex::new(FakeState::default()));
    let mut original = fake_supervisor(state.clone(), clock.clone(), Duration::ZERO);
    original
        .install_session("Demo", make_journal(4, Some(1)))
        .unwrap();
    original
        .build_and_start(&mut ScriptedBuilder::fake(clock.clone(), Duration::ZERO))
        .unwrap();
    let context = sha256(b"resolved configuration");
    let bytes = original.save_session(context, 3, &|_| true).unwrap();
    original.shutdown_worker();
    drop(original);
    // fake_supervisor opens an independent VirtualFs cache each time.
    let mut resumed = fake_supervisor(state.clone(), clock.clone(), Duration::ZERO);
    let saved = SavedSession::from_bytes(&bytes, ProtocolLimits::default()).unwrap();
    let result = resumed
        .resume_saved_session(
            &mut ScriptedBuilder::fake(clock, Duration::ZERO),
            saved,
            "Demo",
            context,
            &|_| true,
        )
        .unwrap();
    assert_eq!(result.plan.reuse, 4);
    assert_eq!(result.restored_checkpoint, Some(1));
    assert_eq!(result.replayed_entries, 2);
    assert!(!result.cold_fallback);
    assert_eq!(resumed.save_session(context, 3, &|_| true).unwrap(), bytes);
    assert_eq!(state.lock().unwrap().launches, 2);
}

#[test]
fn saved_session_does_not_replace_a_running_supervisor() {
    use fmn_studio::supervisor::session::SavedSession;
    let clock = Arc::new(FakeClock::new());
    let state = Arc::new(Mutex::new(FakeState::default()));
    let mut supervisor = fake_supervisor(state.clone(), clock.clone(), Duration::ZERO);
    supervisor
        .install_session("Demo", make_journal(3, Some(0)))
        .unwrap();
    supervisor
        .build_and_start(&mut ScriptedBuilder::fake(clock.clone(), Duration::ZERO))
        .unwrap();
    let context = sha256(b"context");
    let bytes = supervisor.save_session(context, 2, &|_| true).unwrap();
    let saved = SavedSession::from_bytes(&bytes, ProtocolLimits::default()).unwrap();
    let error = supervisor
        .resume_saved_session(
            &mut ScriptedBuilder::fake(clock, Duration::ZERO),
            saved,
            "Demo",
            context,
            &|_| true,
        )
        .unwrap_err();
    assert!(error.to_string().contains("fresh supervisor"));
    assert_eq!(
        supervisor.save_session(context, 2, &|_| true).unwrap(),
        bytes
    );
    assert_eq!(state.lock().unwrap().launches, 1);
    assert_eq!(state.lock().unwrap().terminated, 0);
}

#[test]
fn saved_session_changed_context_or_build_refuses_before_worker_launch() {
    use fmn_studio::supervisor::session::SavedSession;
    let clock = Arc::new(FakeClock::new());
    let state = Arc::new(Mutex::new(FakeState::default()));
    let mut original = fake_supervisor(state.clone(), clock.clone(), Duration::ZERO);
    original
        .install_session("Demo", make_journal(3, Some(0)))
        .unwrap();
    original
        .build_and_start(&mut ScriptedBuilder::fake(clock.clone(), Duration::ZERO))
        .unwrap();
    let context = sha256(b"config");
    let bytes = original.save_session(context, 2, &|_| true).unwrap();
    original.shutdown_worker();
    for (name, supplied_context, build, diagnostic) in [
        ("Other", context, 0, "configuration"),
        ("Demo", sha256(b"different"), 0, "configuration"),
        ("Demo", context, 1, "worker build"),
    ] {
        let mut resumed = fake_supervisor(state.clone(), clock.clone(), Duration::ZERO);
        let saved = SavedSession::from_bytes(&bytes, ProtocolLimits::default()).unwrap();
        let mut builder = ScriptedBuilder::fake(clock.clone(), Duration::ZERO);
        builder.build = build;
        let error = resumed
            .resume_saved_session(&mut builder, saved, name, supplied_context, &|_| true)
            .unwrap_err();
        assert!(error.to_string().contains(diagnostic), "{error}");
        assert_eq!(state.lock().unwrap().launches, 1);
        assert_eq!(resumed.generation(), 0);
    }
}

#[test]
fn saved_session_replay_divergence_is_failure_not_successful_cold_fallback() {
    use fmn_studio::supervisor::session::SavedSession;
    let clock = Arc::new(FakeClock::new());
    let state = Arc::new(Mutex::new(FakeState::default()));
    let mut original = fake_supervisor(state.clone(), clock.clone(), Duration::ZERO);
    original
        .install_session("Demo", make_journal(3, Some(0)))
        .unwrap();
    original
        .build_and_start(&mut ScriptedBuilder::fake(clock.clone(), Duration::ZERO))
        .unwrap();
    let context = sha256(b"config");
    let bytes = original.save_session(context, 2, &|_| true).unwrap();
    original.shutdown_worker();
    state.lock().unwrap().diverge_next_replay = true;
    let mut resumed = fake_supervisor(state.clone(), clock.clone(), Duration::ZERO);
    let saved = SavedSession::from_bytes(&bytes, ProtocolLimits::default()).unwrap();
    let error = resumed
        .resume_saved_session(
            &mut ScriptedBuilder::fake(clock, Duration::ZERO),
            saved,
            "Demo",
            context,
            &|_| true,
        )
        .unwrap_err();
    assert!(
        error.to_string().contains("refusing partial recovery"),
        "{error}"
    );
    let state = state.lock().unwrap();
    assert_eq!(state.launches, state.terminated);
}
