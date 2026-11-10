//! fm-39s canonical IPC acceptance: every message class round-trips, the
//! request/response bytes are self-goldened, incompatible versions and
//! malformed ranges fail closed, and the outer pipe rejects oversized lengths
//! before allocating.

use std::error::Error as _;
use std::io::{Cursor, Read};

use fmn_hash::{Schema, Writer, sha256};
use fmn_scene::{
    CommandKind, CommandRecord, EffectClass, Entry, EventPayload, Journal, JournalError, Key,
    Modifiers, RenderBackendRecord, RenderBackendRole,
};
use fmn_studio::{
    CURRENT_VERSION, Checkpoint, CrashReport, DebugLayerSet, FrameEncoding, FramePayload,
    FrameStream, FramingError, JournalReplay, MAX_FRAME_RENDER_BACKENDS, ProtocolError,
    ProtocolLimits, ProtocolVersion, REQUEST_SCHEMA, RequestEnvelope, ResponseEnvelope,
    StudioDataKind, SupervisorRequest, TransportCapabilities, WorkerErrorCode, WorkerResponse,
    read_request, read_response, write_request, write_response,
};

fn test_render_backend() -> RenderBackendRecord {
    RenderBackendRecord::new(
        RenderBackendRole::FrameStream,
        b"protocol-test-render-backend".to_vec(),
    )
    .unwrap()
}

fn command(label: &str) -> CommandRecord {
    CommandRecord {
        kind: CommandKind::Play,
        identity: sha256(label.as_bytes()),
        label: label.to_owned(),
    }
}

fn journal() -> Journal {
    let state = b"canonical checkpoint".to_vec();
    let mut journal = Journal::new();
    journal
        .record(Entry {
            command: command("play FadeIn(dot)"),
            effect: EffectClass::Pure,
            reads: Vec::new(),
            subprocesses: Vec::new(),
            checkpoint: Some(state.clone()),
            state_hash: sha256(&state),
        })
        .expect("journal entry storage reserves");
    journal
}

fn checkpoint() -> Checkpoint {
    let state = b"canonical checkpoint".to_vec();
    Checkpoint {
        scene: "Demo".to_owned(),
        after_entry: 0,
        state_hash: sha256(&state),
        state,
    }
}

fn request(request_id: u64, request: SupervisorRequest) -> RequestEnvelope {
    RequestEnvelope {
        request_id,
        request,
    }
}

fn response(request_id: u64, response: WorkerResponse) -> ResponseEnvelope {
    ResponseEnvelope {
        request_id,
        response,
    }
}

#[test]
fn every_request_variant_round_trips_canonically() {
    let limits = ProtocolLimits::default();
    let replay = JournalReplay {
        scene: "Demo".to_owned(),
        from_entry: 0,
        through_entry: 1,
        journal: journal().to_bytes().unwrap(),
    };
    let variants = vec![
        SupervisorRequest::Hello {
            version: CURRENT_VERSION,
            supervisor_build: sha256(b"supervisor"),
            max_frame_bytes: 1024,
        },
        SupervisorRequest::EnumerateScenes,
        SupervisorRequest::Play {
            scene: "Demo".to_owned(),
            command: command("play FadeIn(dot)"),
        },
        SupervisorRequest::Seek {
            scene: "Demo".to_owned(),
            frame: 42,
        },
        SupervisorRequest::Scrub {
            scene: "Demo".to_owned(),
            frame: 17,
        },
        SupervisorRequest::RestoreCheckpoint(checkpoint()),
        SupervisorRequest::ReplayJournal(replay),
        SupervisorRequest::Event {
            scene: "Demo".to_owned(),
            event: EventPayload::KeyPress {
                key: Key::Character('k'),
                modifiers: Modifiers::CONTROL | Modifiers::SHIFT,
            },
        },
        SupervisorRequest::Inspect {
            scene: "Demo".to_owned(),
        },
        SupervisorRequest::Overlay {
            scene: "Demo".to_owned(),
            layers: DebugLayerSet::TILES | DebugLayerSet::DEPTH,
        },
        SupervisorRequest::Shutdown,
    ];
    for (index, variant) in variants.into_iter().enumerate() {
        let envelope = request(index as u64 + 1, variant);
        let bytes = envelope.to_bytes(limits).unwrap();
        let decoded = RequestEnvelope::from_bytes(&bytes, limits).unwrap();
        assert_eq!(decoded, envelope);
        assert_eq!(decoded.to_bytes(limits).unwrap(), bytes);
    }
}

#[test]
fn every_response_variant_round_trips_canonically() {
    let limits = ProtocolLimits::default();
    let rgba = vec![0, 1, 2, 3, 4, 5, 6, 7];
    let png = b"\x89PNG\r\n\x1a\nfixture".to_vec();
    let journal = journal().to_bytes().unwrap();
    let inspection = br#"{"scene":"Demo","nodes":[]}"#.to_vec();
    let variants = vec![
        WorkerResponse::Hello {
            version: CURRENT_VERSION,
            worker_build: sha256(b"worker"),
            transports: TransportCapabilities {
                pipe: true,
                shared_memory: true,
            },
        },
        WorkerResponse::Scenes(vec!["Demo".to_owned(), "Other".to_owned()]),
        WorkerResponse::Ack {
            state_hash: Some(sha256(b"state")),
            journal_len: 4,
        },
        WorkerResponse::Frame(FrameStream {
            scene: "Demo".to_owned(),
            frame_index: 7,
            width: 2,
            height: 1,
            stride: 8,
            encoding: FrameEncoding::Rgba8,
            payload: FramePayload::Pipe {
                digest: sha256(&rgba),
                bytes: rgba,
            },
            render_backends: vec![
                RenderBackendRecord::new(
                    RenderBackendRole::FrameStream,
                    b"canonical fast-cpu backend".to_vec(),
                )
                .unwrap(),
            ],
        }),
        WorkerResponse::Frame(FrameStream {
            scene: "Demo".to_owned(),
            frame_index: 8,
            width: 1920,
            height: 1080,
            stride: 0,
            encoding: FrameEncoding::Png,
            payload: FramePayload::Pipe {
                digest: sha256(&png),
                bytes: png,
            },
            render_backends: vec![test_render_backend()],
        }),
        WorkerResponse::Frame(FrameStream {
            scene: "Demo".to_owned(),
            frame_index: 9,
            width: 2,
            height: 1,
            stride: 8,
            encoding: FrameEncoding::Rgba8,
            payload: FramePayload::SharedMemory {
                token: sha256(b"opaque region token"),
                len: 8,
                digest: sha256(b"region contents"),
            },
            render_backends: vec![test_render_backend()],
        }),
        WorkerResponse::Checkpoint(checkpoint()),
        WorkerResponse::JournalSegment {
            scene: "Demo".to_owned(),
            start_entry: 3,
            journal,
        },
        WorkerResponse::ReplayComplete {
            from_entry: 2,
            state_hashes: vec![sha256(b"s2"), sha256(b"s3")],
        },
        WorkerResponse::Crash(CrashReport {
            scene: Some("Demo".to_owned()),
            message: "scene panic".to_owned(),
            journal_tail: b"tail".to_vec(),
            state_hash: Some(sha256(b"last state")),
        }),
        WorkerResponse::Error {
            code: WorkerErrorCode::ReplayFailed,
            message: "replay refused".to_owned(),
        },
        WorkerResponse::StudioData {
            scene: "Demo".to_owned(),
            kind: StudioDataKind::Inspection,
            digest: sha256(&inspection),
            bytes: inspection,
        },
        WorkerResponse::Bye,
    ];
    for (index, variant) in variants.into_iter().enumerate() {
        let envelope = response(index as u64 + 1, variant);
        let bytes = envelope.to_bytes(limits).unwrap();
        let decoded = ResponseEnvelope::from_bytes(&bytes, limits).unwrap();
        assert_eq!(decoded, envelope);
        assert_eq!(decoded.to_bytes(limits).unwrap(), bytes);
    }
}

#[test]
fn request_and_response_bytes_are_self_goldened() {
    let limits = ProtocolLimits::default();
    let request = request(
        0x0102_0304_0506_0708,
        SupervisorRequest::Play {
            scene: "GoldenScene".to_owned(),
            command: command("play golden"),
        },
    );
    let response = response(
        0x0102_0304_0506_0708,
        WorkerResponse::Crash(CrashReport {
            scene: Some("GoldenScene".to_owned()),
            message: "golden crash".to_owned(),
            journal_tail: b"tail".to_vec(),
            state_hash: Some(sha256(b"state")),
        }),
    );
    let actual = format!(
        "request={}\nresponse={}\n",
        hex(&request.to_bytes(limits).unwrap()),
        hex(&response.to_bytes(limits).unwrap())
    );
    assert_eq!(
        actual,
        include_str!("../fixtures/protocol_v1.hex"),
        "protocol bytes moved; review the schema and deliberately re-bless the fixture"
    );
}

#[test]
fn live_and_container_version_skew_are_rejected() {
    let peer = ProtocolVersion { major: 2, minor: 0 };
    assert!(matches!(
        peer.require_current(),
        Err(ProtocolError::VersionSkew {
            local: CURRENT_VERSION,
            peer: found,
        }) if found == peer
    ));

    let limits = ProtocolLimits::default();
    let mut writer = Writer::new(Schema::new(
        REQUEST_SCHEMA.magic,
        REQUEST_SCHEMA.id,
        REQUEST_SCHEMA.major + 1,
        0,
    ));
    writer.put_u64(1);
    writer.put_u8(1);
    let foreign = writer.finish().unwrap();
    assert!(matches!(
        RequestEnvelope::from_bytes(&foreign, limits),
        Err(ProtocolError::Serial(
            fmn_hash::SerialError::MajorMismatch { reader: 1, doc: 2 }
        ))
    ));
}

#[test]
fn malformed_semantics_fail_before_crossing_the_pipe() {
    let limits = ProtocolLimits::default();
    let mut bad_checkpoint = checkpoint();
    bad_checkpoint.state.push(0);
    assert!(matches!(
        request(1, SupervisorRequest::RestoreCheckpoint(bad_checkpoint)).to_bytes(limits),
        Err(ProtocolError::Malformed("checkpoint state hash"))
    ));

    let replay = JournalReplay {
        scene: "Demo".to_owned(),
        from_entry: 0,
        through_entry: 2,
        journal: journal().to_bytes().unwrap(),
    };
    assert!(matches!(
        request(2, SupervisorRequest::ReplayJournal(replay)).to_bytes(limits),
        Err(ProtocolError::Malformed("replay range exceeds journal"))
    ));

    assert!(matches!(
        request(
            3,
            SupervisorRequest::Event {
                scene: "Demo".to_owned(),
                event: EventPayload::MouseMotion {
                    point: [f64::INFINITY, 0.0, 0.0],
                    delta: [0.0, 0.0, 0.0],
                    modifiers: Modifiers::NONE,
                },
            }
        )
        .to_bytes(limits),
        Err(ProtocolError::Malformed("invalid input event"))
    ));

    let bytes = br#"{"nodes":[]}"#.to_vec();
    assert!(matches!(
        response(
            4,
            WorkerResponse::StudioData {
                scene: "Demo".to_owned(),
                kind: StudioDataKind::Inspection,
                digest: sha256(b"different"),
                bytes,
            }
        )
        .to_bytes(limits),
        Err(ProtocolError::Malformed("Studio data digest"))
    ));

    let malformed_json = b"{]".to_vec();
    assert!(matches!(
        response(
            5,
            WorkerResponse::StudioData {
                scene: "Demo".to_owned(),
                kind: StudioDataKind::Inspection,
                digest: sha256(&malformed_json),
                bytes: malformed_json,
            }
        )
        .to_bytes(limits),
        Err(ProtocolError::Malformed("Studio data is not valid JSON"))
    ));

    let mut writer = Writer::new(REQUEST_SCHEMA);
    writer.put_u64(6);
    writer.put_u8(99);
    assert!(matches!(
        RequestEnvelope::from_bytes(&writer.finish().unwrap(), limits),
        Err(ProtocolError::Malformed("supervisor request tag"))
    ));
}

#[test]
fn frame_backend_provenance_is_required_bounded_and_authenticated() {
    let limits = ProtocolLimits::default();
    let rgba = vec![0, 0, 0, 255];
    let frame = FrameStream {
        scene: "Demo".to_owned(),
        frame_index: 0,
        width: 1,
        height: 1,
        stride: 4,
        encoding: FrameEncoding::Rgba8,
        payload: FramePayload::Pipe {
            digest: sha256(&rgba),
            bytes: rgba,
        },
        render_backends: Vec::new(),
    };
    assert_eq!(
        response(1, WorkerResponse::Frame(frame)).to_bytes(limits),
        Err(ProtocolError::Malformed(
            "frame has no render backend identity"
        ))
    );

    let mut writer = Writer::new(fmn_studio::RESPONSE_SCHEMA);
    writer.put_u64(2);
    writer.put_u8(3);
    writer.put_str("Demo");
    writer.put_u64(0);
    writer.put_u32(1);
    writer.put_u32(1);
    writer.put_u32(4);
    writer.put_u8(1);
    writer.put_u8(0);
    writer.put_digest(&sha256(&[0, 0, 0, 255]));
    writer.put_bytes(&[0, 0, 0, 255]);
    writer.put_u32(1);
    writer.put_u8(RenderBackendRole::FrameStream.wire_code());
    writer.put_digest(&sha256(b"different backend"));
    writer.put_bytes(b"actual backend");
    assert!(matches!(
        ResponseEnvelope::from_bytes(&writer.finish().unwrap(), limits),
        Err(ProtocolError::InvalidJournal(JournalError::Malformed(
            "render backend identity digest"
        )))
    ));

    let mut writer = Writer::new(fmn_studio::RESPONSE_SCHEMA);
    writer.put_u64(3);
    writer.put_u8(3);
    writer.put_str("Demo");
    writer.put_u64(0);
    writer.put_u32(1);
    writer.put_u32(1);
    writer.put_u32(4);
    writer.put_u8(1);
    writer.put_u8(0);
    writer.put_digest(&sha256(&[0, 0, 0, 255]));
    writer.put_bytes(&[0, 0, 0, 255]);
    writer.put_u32(u32::try_from(MAX_FRAME_RENDER_BACKENDS + 1).unwrap());
    assert_eq!(
        ResponseEnvelope::from_bytes(&writer.finish().unwrap(), limits),
        Err(ProtocolError::CountLimit {
            field: "frame render backend",
            limit: MAX_FRAME_RENDER_BACKENDS,
            needed: MAX_FRAME_RENDER_BACKENDS + 1,
        })
    );
}

#[test]
fn invalid_journals_preserve_the_typed_source_across_protocol_validation() {
    let limits = ProtocolLimits::default();
    let invalid_journal = vec![0];
    let request_error = request(
        7,
        SupervisorRequest::ReplayJournal(JournalReplay {
            scene: "Demo".to_owned(),
            from_entry: 0,
            through_entry: 0,
            journal: invalid_journal.clone(),
        }),
    )
    .to_bytes(limits)
    .expect_err("a truncated replay journal must be refused");
    let response_error = response(
        8,
        WorkerResponse::JournalSegment {
            scene: "Demo".to_owned(),
            start_entry: 0,
            journal: invalid_journal,
        },
    )
    .to_bytes(limits)
    .expect_err("a truncated journal segment must be refused");

    for error in [request_error, response_error] {
        assert!(matches!(
            &error,
            ProtocolError::InvalidJournal(JournalError::Serial(fmn_hash::SerialError::Truncated {
                need: 56,
                got: 1
            }))
        ));
        assert_eq!(error.clone(), error);
        assert_eq!(
            error.to_string(),
            "invalid replay journal: journal container: document truncated: need at least 56 bytes, got 1"
        );
        assert_eq!(
            error.source().map(ToString::to_string),
            Some("journal container: document truncated: need at least 56 bytes, got 1".to_owned())
        );
    }
}

#[test]
fn crash_messages_use_their_specific_limit_before_ownership() {
    let limits = ProtocolLimits {
        max_crash_message_bytes: 4,
        ..ProtocolLimits::default()
    };
    let report = CrashReport {
        scene: None,
        message: "12345".to_owned(),
        journal_tail: Vec::new(),
        state_hash: None,
    };
    assert_eq!(
        response(7, WorkerResponse::Crash(report)).to_bytes(limits),
        Err(ProtocolError::PayloadLimit {
            field: "crash message",
            limit: 4,
            needed: 5,
        })
    );

    let mut writer = Writer::new(fmn_studio::RESPONSE_SCHEMA);
    writer.put_u64(8);
    writer.put_u8(7);
    writer.put_bool(false);
    writer.put_str("12345");
    writer.put_bytes(&[]);
    writer.put_bool(false);
    assert_eq!(
        ResponseEnvelope::from_bytes(&writer.finish().unwrap(), limits),
        Err(ProtocolError::PayloadLimit {
            field: "crash message",
            limit: 4,
            needed: 5,
        })
    );
}

#[test]
fn worker_error_messages_use_their_specific_limit_before_ownership() {
    let limits = ProtocolLimits {
        max_error_message_bytes: 4,
        ..ProtocolLimits::default()
    };
    assert_eq!(
        response(
            9,
            WorkerResponse::Error {
                code: WorkerErrorCode::ReplayFailed,
                message: "12345".to_owned(),
            },
        )
        .to_bytes(limits),
        Err(ProtocolError::PayloadLimit {
            field: "worker error",
            limit: 4,
            needed: 5,
        })
    );

    let mut writer = Writer::new(fmn_studio::RESPONSE_SCHEMA);
    writer.put_u64(10);
    writer.put_u8(8);
    writer.put_u16(4);
    writer.put_str("12345");
    assert_eq!(
        ResponseEnvelope::from_bytes(&writer.finish().unwrap(), limits),
        Err(ProtocolError::PayloadLimit {
            field: "worker error",
            limit: 4,
            needed: 5,
        })
    );
}

#[test]
fn length_framing_round_trips_and_rejects_oversize_before_allocation() {
    let limits = ProtocolLimits::default();
    let request = request(1, SupervisorRequest::EnumerateScenes);
    let mut pipe = Vec::new();
    write_request(&mut pipe, &request, limits).unwrap();
    assert_eq!(
        read_request(&mut Cursor::new(pipe), limits).unwrap(),
        request
    );

    let response = response(1, WorkerResponse::Scenes(vec!["Demo".to_owned()]));
    let mut pipe = Vec::new();
    write_response(&mut pipe, &response, limits).unwrap();
    assert_eq!(
        read_response(&mut Cursor::new(pipe), limits).unwrap(),
        response
    );

    let tight = ProtocolLimits {
        max_message_bytes: 64,
        ..limits
    };
    let prefix = 65u64.to_le_bytes();
    assert!(matches!(
        read_request(&mut Cursor::new(prefix), tight),
        Err(FramingError::FrameTooLarge {
            limit: 64,
            needed: 65,
        })
    ));
}

#[test]
fn admitted_frame_storage_refusal_is_typed_before_payload_read() {
    let needed = usize::MAX;
    let limits = ProtocolLimits {
        max_message_bytes: needed,
        ..ProtocolLimits::default()
    };
    let prefix = u64::try_from(needed)
        .expect("supported targets represent usize in the wire width")
        .to_le_bytes();

    assert!(matches!(
        read_request(&mut Cursor::new(prefix), limits),
        Err(FramingError::FrameStorageAllocationFailed { bytes, .. }) if bytes == needed
    ));
}

struct OverreportingReader {
    inner: Cursor<Vec<u8>>,
    overreport_on_call: usize,
    returned: usize,
    calls: usize,
}

impl Read for OverreportingReader {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        self.calls += 1;
        if self.calls == self.overreport_on_call {
            return Ok(self.returned);
        }
        self.inner.read(buffer)
    }
}

struct FragmentingReader {
    inner: Cursor<Vec<u8>>,
    interrupt_next: bool,
}

impl Read for FragmentingReader {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        if self.interrupt_next {
            self.interrupt_next = false;
            return Err(std::io::ErrorKind::Interrupted.into());
        }
        self.interrupt_next = true;
        let take = buffer.len().min(1);
        self.inner.read(&mut buffer[..take])
    }
}

#[test]
fn invalid_read_counts_are_contained_at_every_frame_stage() {
    let limits = ProtocolLimits::default();
    let request = request(1, SupervisorRequest::EnumerateScenes);
    let mut frame = Vec::new();
    write_request(&mut frame, &request, limits).unwrap();
    let payload_len = frame.len() - 8;

    for (overreport_on_call, requested, returned) in [
        (1, 1, 2),
        (2, 7, 8),
        (3, payload_len, payload_len.saturating_add(1)),
    ] {
        let mut reader = OverreportingReader {
            inner: Cursor::new(frame.clone()),
            overreport_on_call,
            returned,
            calls: 0,
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            read_request(&mut reader, limits)
        }));
        let error = result
            .expect("an invalid Read count must not panic")
            .expect_err("an invalid Read count must be refused");
        assert!(matches!(
            error,
            FramingError::InvalidReadCount {
                requested: actual_requested,
                returned: actual_returned,
            } if actual_requested == requested && actual_returned == returned
        ));
        assert_eq!(reader.calls, overreport_on_call);
    }

    let error = FramingError::InvalidReadCount {
        requested: 7,
        returned: 8,
    };
    assert_eq!(
        error.to_string(),
        "IPC reader violated Read by returning 8 bytes for a 7-byte buffer"
    );
    assert!(error.source().is_none());
}

#[test]
fn checked_frame_reads_preserve_partial_interrupt_and_eof_semantics() {
    let limits = ProtocolLimits::default();
    let request = request(1, SupervisorRequest::EnumerateScenes);
    let mut frame = Vec::new();
    write_request(&mut frame, &request, limits).unwrap();
    let mut fragmented = FragmentingReader {
        inner: Cursor::new(frame),
        interrupt_next: true,
    };
    assert_eq!(read_request(&mut fragmented, limits).unwrap(), request);

    assert!(matches!(
        read_request(&mut Cursor::new(Vec::<u8>::new()), limits),
        Err(FramingError::Closed)
    ));

    for truncated in [vec![1], 1u64.to_le_bytes().to_vec()] {
        assert!(matches!(
            read_request(&mut Cursor::new(truncated), limits),
            Err(FramingError::Io(error))
                if error.kind() == std::io::ErrorKind::UnexpectedEof
        ));
    }
}

#[test]
fn response_collection_counts_are_preflighted_before_reserve() {
    let limits = ProtocolLimits::default();

    let mut scenes = Writer::new(fmn_studio::RESPONSE_SCHEMA);
    scenes.put_u64(41);
    scenes.put_u8(1);
    scenes.put_u32(u32::try_from(limits.max_scenes).unwrap());
    assert_eq!(
        ResponseEnvelope::from_bytes(&scenes.finish().unwrap(), limits),
        Err(ProtocolError::CollectionPayloadTooShort {
            field: "scene",
            count: limits.max_scenes,
            minimum_bytes: u64::try_from(limits.max_scenes).unwrap() * 8,
            remaining_bytes: 0,
        })
    );

    let mut replay = Writer::new(fmn_studio::RESPONSE_SCHEMA);
    replay.put_u64(42);
    replay.put_u8(6);
    replay.put_u64(0);
    replay.put_u32(u32::try_from(limits.max_replay_hashes).unwrap());
    assert_eq!(
        ResponseEnvelope::from_bytes(&replay.finish().unwrap(), limits),
        Err(ProtocolError::CollectionPayloadTooShort {
            field: "replay state hash",
            count: limits.max_replay_hashes,
            minimum_bytes: u64::try_from(limits.max_replay_hashes).unwrap() * 32,
            remaining_bytes: 0,
        })
    );
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(char::from(DIGITS[usize::from(byte >> 4)]));
        out.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    out
}
