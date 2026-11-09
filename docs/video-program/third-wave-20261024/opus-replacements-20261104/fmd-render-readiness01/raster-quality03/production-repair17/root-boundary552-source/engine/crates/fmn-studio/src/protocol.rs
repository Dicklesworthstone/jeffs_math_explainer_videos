//! Canonical supervisor ↔ scene-worker IPC (§13.3, D-14).
//!
//! Control messages are fmn-hash documents carried in a bounded
//! little-endian length frame.  The canonical document supplies versioning,
//! integrity, and deterministic encoding; the outer length lets a pipe reader
//! reject an oversized message before allocating it.  Frame bytes may travel
//! inline through the same pipe or out-of-band through an opaque shared-memory
//! token.  No filesystem path crosses this boundary.

use std::collections::TryReserveError;
use std::fmt;
use std::io::{Read, Write};

use fmn_hash::{Digest, Limits, Reader, Schema, SerialError, UnknownPolicy, Writer, sha256};
use fmn_scene::{
    CommandKind, CommandRecord, EventPayload, Journal, JournalError, Key, Modifiers, MouseButton,
    RenderBackendRecord, RenderBackendRole,
};

/// Canonical request envelope schema.
pub const REQUEST_SCHEMA: Schema = Schema::new(*b"FMNI", 1, 1, 1);
/// Canonical response envelope schema.
pub const RESPONSE_SCHEMA: Schema = Schema::new(*b"FMNI", 2, 1, 2);
const STUDIO_SEEK_COMMAND_SCHEMA: Schema = Schema::new(*b"FMNI", 3, 1, 0);

/// The live protocol version advertised during the mandatory handshake.
pub const CURRENT_VERSION: ProtocolVersion = ProtocolVersion { major: 1, minor: 3 };

/// Maximum distinct renderer/backend identities attached to one frame.
///
/// Current production frames carry one CPU stream identity, or a native
/// presentation identity plus the companion CPU stream identity.
pub const MAX_FRAME_RENDER_BACKENDS: usize = 4;

const STUDIO_SEEK_LABEL_PREFIX: &str = "studio seek frame ";

/// Build the canonical replay command for one committed Studio seek.
///
/// The frame remains human-readable in the label while the command identity
/// binds both the scene and frame through a domain-separated canonical
/// document. The worker validates the complete record before executing it, so
/// a label cannot be edited independently of the identity.
pub fn studio_seek_command(scene: &str, frame: i64) -> Result<CommandRecord, ProtocolError> {
    require_scene(scene)?;
    if frame < 0 {
        return Err(ProtocolError::Malformed("negative Studio seek frame"));
    }
    let mut identity = Writer::new(STUDIO_SEEK_COMMAND_SCHEMA);
    identity.put_str(scene).put_i64(frame);
    let identity = sha256(&identity.finish()?);
    let mut label = string_with_capacity(64, "Studio seek command label")?;
    label.push_str(STUDIO_SEEK_LABEL_PREFIX);
    fmt::write(&mut label, format_args!("{frame}"))
        .map_err(|_| ProtocolError::Malformed("Studio seek command label"))?;
    Ok(CommandRecord {
        kind: CommandKind::Wait,
        identity,
        label,
    })
}

/// Recover and authenticate the frame carried by a Studio seek command.
pub fn studio_seek_frame(scene: &str, command: &CommandRecord) -> Result<i64, ProtocolError> {
    let frame = command
        .label
        .strip_prefix(STUDIO_SEEK_LABEL_PREFIX)
        .ok_or(ProtocolError::Malformed("Studio seek command label"))?
        .parse::<i64>()
        .map_err(|_| ProtocolError::Malformed("Studio seek command frame"))?;
    let expected = studio_seek_command(scene, frame)?;
    if &expected != command {
        return Err(ProtocolError::Malformed("Studio seek command identity"));
    }
    Ok(frame)
}

const STUDIO_INPUT_COMMAND_SCHEMA: Schema = Schema::new(*b"FMNI", 4, 1, 0);
const STUDIO_INPUT_LABEL_PREFIX: &str = "studio input ";
const STUDIO_INPUT_DOCUMENT_LIMIT: usize = 1024;
pub const MAX_STUDIO_INPUT_LABEL_BYTES: usize = 2048;

#[derive(Clone, Debug, PartialEq)]
pub struct StudioInput {
    pub frame: i64,
    pub revision: u64,
    pub target: Option<u64>,
    pub event: EventPayload,
}

pub fn studio_input_command(
    scene: &str,
    input: &StudioInput,
) -> Result<CommandRecord, ProtocolError> {
    require_scene(scene)?;
    if input.frame < 0 {
        return Err(ProtocolError::Malformed("negative Studio input frame"));
    }
    input
        .event
        .validate()
        .map_err(|_| ProtocolError::Malformed("invalid Studio input event"))?;
    // Bound every writer append before allocation, including the hex label's
    // tighter effective document ceiling. A huge scene name cannot allocate an
    // unbounded intermediate document only to be rejected afterward.
    let document_limit = STUDIO_INPUT_DOCUMENT_LIMIT
        .min((MAX_STUDIO_INPUT_LABEL_BYTES - STUDIO_INPUT_LABEL_PREFIX.len()) / 2);
    limit_payload("Studio input scene name", scene.len(), document_limit)?;
    let mut document = Writer::with_limits(
        STUDIO_INPUT_COMMAND_SCHEMA,
        Limits {
            max_total: document_limit,
            max_field: document_limit,
        },
    );
    document
        .put_str(scene)
        .put_i64(input.frame)
        .put_u64(input.revision);
    match input.target {
        None => {
            document.put_u8(0);
        }
        Some(target) => {
            document.put_u8(1);
            document.put_u64(target);
        }
    }
    put_event(&mut document, &input.event);
    let document = document.finish()?;
    limit_payload(
        "Studio input command document",
        document.len(),
        STUDIO_INPUT_DOCUMENT_LIMIT,
    )?;
    let identity = sha256(&document);
    let label_bytes = STUDIO_INPUT_LABEL_PREFIX.len() + document.len() * 2;
    let mut label = string_with_capacity(label_bytes, "Studio input command label")?;
    label.push_str(STUDIO_INPUT_LABEL_PREFIX);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in document {
        label.push(char::from(HEX[usize::from(byte >> 4)]));
        label.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    limit_payload(
        "Studio input command label",
        label.len(),
        MAX_STUDIO_INPUT_LABEL_BYTES,
    )?;
    Ok(CommandRecord {
        kind: CommandKind::Input,
        identity,
        label,
    })
}

pub fn studio_input_payload(
    scene: &str,
    command: &CommandRecord,
) -> Result<StudioInput, ProtocolError> {
    if command.kind != CommandKind::Input {
        return Err(ProtocolError::Malformed("Studio input command kind"));
    }
    let hex = command
        .label
        .strip_prefix(STUDIO_INPUT_LABEL_PREFIX)
        .ok_or(ProtocolError::Malformed("Studio input command label"))?;
    limit_payload(
        "Studio input command label",
        command.label.len(),
        MAX_STUDIO_INPUT_LABEL_BYTES,
    )?;
    if hex.len() % 2 != 0
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ProtocolError::Malformed("Studio input command label"));
    }
    let mut document = vec_with_capacity(hex.len() / 2, "Studio input command document")?;
    let bytes = hex.as_bytes();
    for pair in bytes.as_chunks::<2>().0 {
        document.push(hex_digit(pair[0])? << 4 | hex_digit(pair[1])?);
    }
    let limits = Limits {
        max_total: STUDIO_INPUT_DOCUMENT_LIMIT,
        max_field: STUDIO_INPUT_DOCUMENT_LIMIT,
    };
    let mut reader = Reader::open(
        &document,
        STUDIO_INPUT_COMMAND_SCHEMA,
        limits,
        UnknownPolicy::Strict,
    )?;
    let decoded_scene = reader.get_str()?;
    let frame = reader.get_i64()?;
    let revision = reader.get_u64()?;
    let target = match reader.get_u8()? {
        0 => None,
        1 => Some(reader.get_u64()?),
        _ => return Err(ProtocolError::Malformed("Studio input target tag")),
    };
    let event = get_event(&mut reader)?;
    reader.finish()?;
    let decoded = StudioInput {
        frame,
        revision,
        target,
        event,
    };
    let expected = studio_input_command(decoded_scene, &decoded)?;
    if &expected != command || decoded_scene != scene {
        return Err(ProtocolError::Malformed("Studio input command identity"));
    }
    Ok(decoded)
}

fn hex_digit(byte: u8) -> Result<u8, ProtocolError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(ProtocolError::Malformed("Studio input command label")),
    }
}

/// A live-protocol version.
///
/// fmn-hash already rejects incompatible document schemas.  The explicit
/// version in `Hello` additionally prevents two processes that happen to
/// understand the envelope from silently disagreeing about session semantics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProtocolVersion {
    /// Breaking version.
    pub major: u16,
    /// Additive version.
    pub minor: u16,
}

impl ProtocolVersion {
    /// Require the exact live protocol used by this build.
    ///
    /// Unlike a durable file reader, a supervisor can always restart a worker
    /// from the same build.  Exact matching is therefore safer than carrying
    /// compatibility branches in a security boundary.
    pub fn require_current(self) -> Result<(), ProtocolError> {
        if self == CURRENT_VERSION {
            Ok(())
        } else {
            Err(ProtocolError::VersionSkew {
                local: CURRENT_VERSION,
                peer: self,
            })
        }
    }
}

/// Resource budgets for one IPC peer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProtocolLimits {
    /// Maximum canonical document, including its fmn-hash envelope.
    pub max_message_bytes: usize,
    /// Maximum individual bytes/string field.
    pub max_field_bytes: usize,
    /// Maximum inline or shared frame payload.
    pub max_frame_bytes: usize,
    /// Maximum serialized checkpoint.
    pub max_checkpoint_bytes: usize,
    /// Maximum serialized journal or journal segment.
    pub max_journal_bytes: usize,
    /// Maximum crash-report diagnostic string.
    pub max_crash_message_bytes: usize,
    /// Maximum typed worker-refusal diagnostic string.
    pub max_error_message_bytes: usize,
    /// Maximum crash-report journal tail.
    pub max_crash_tail_bytes: usize,
    /// Maximum scene names in one enumeration response.
    pub max_scenes: usize,
    /// Maximum state hashes in one replay response.
    pub max_replay_hashes: usize,
    /// Maximum inspector or debug-overlay document returned by a worker.
    pub max_studio_data_bytes: usize,
}

impl ProtocolLimits {
    fn serial(self) -> Limits {
        Limits {
            max_total: self.max_message_bytes,
            max_field: self.max_field_bytes,
        }
    }
}

impl Default for ProtocolLimits {
    fn default() -> Self {
        Self {
            max_message_bytes: 128 * 1024 * 1024,
            max_field_bytes: 64 * 1024 * 1024,
            max_frame_bytes: 32 * 1024 * 1024,
            max_checkpoint_bytes: 64 * 1024 * 1024,
            max_journal_bytes: 64 * 1024 * 1024,
            max_crash_message_bytes: 64 * 1024,
            max_error_message_bytes: 64 * 1024,
            max_crash_tail_bytes: 1024 * 1024,
            max_scenes: 4096,
            max_replay_hashes: 1_000_000,
            max_studio_data_bytes: 8 * 1024 * 1024,
        }
    }
}

/// A protocol document was invalid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProtocolError {
    /// Canonical framing/version/integrity failure.
    Serial(SerialError),
    /// The live peers do not run the exact same protocol.
    VersionSkew {
        /// This build.
        local: ProtocolVersion,
        /// The peer.
        peer: ProtocolVersion,
    },
    /// A bounded collection exceeded its protocol-specific ceiling.
    CountLimit {
        /// Which collection.
        field: &'static str,
        /// Ceiling.
        limit: usize,
        /// Supplied size.
        needed: usize,
    },
    /// A byte payload exceeded its protocol-specific ceiling.
    PayloadLimit {
        /// Which payload.
        field: &'static str,
        /// Ceiling.
        limit: usize,
        /// Supplied size.
        needed: usize,
    },
    /// A collection count cannot fit in the bytes left in the document.
    CollectionPayloadTooShort {
        /// Which collection.
        field: &'static str,
        /// Declared item count.
        count: usize,
        /// Minimum encoded bytes required for that count.
        minimum_bytes: u64,
        /// Bytes actually left after the count.
        remaining_bytes: usize,
    },
    /// Storage for an admitted decoded field or collection could not be reserved.
    StorageUnavailable {
        /// Which decoded field or collection needed ownership.
        field: &'static str,
        /// Additional elements or bytes requested from the allocator.
        additional: usize,
        /// Allocation refusal.
        source: TryReserveError,
    },
    /// A decoded payload violated a semantic invariant.
    Malformed(&'static str),
    /// A journal field was not a valid canonical replay journal.
    InvalidJournal(JournalError),
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serial(error) => write!(f, "IPC container: {error}"),
            Self::VersionSkew { local, peer } => write!(
                f,
                "worker protocol version skew: local {}.{}, peer {}.{}",
                local.major, local.minor, peer.major, peer.minor
            ),
            Self::CountLimit {
                field,
                limit,
                needed,
            } => write!(
                f,
                "IPC {field} count {needed} exceeds the configured limit {limit}"
            ),
            Self::PayloadLimit {
                field,
                limit,
                needed,
            } => write!(
                f,
                "IPC {field} payload {needed} bytes exceeds the configured limit {limit}"
            ),
            Self::CollectionPayloadTooShort {
                field,
                count,
                minimum_bytes,
                remaining_bytes,
            } => write!(
                f,
                "IPC {field} count {count} requires at least {minimum_bytes} encoded bytes, but only {remaining_bytes} remain"
            ),
            Self::StorageUnavailable {
                field,
                additional,
                source,
            } => write!(
                f,
                "IPC {field} storage could not reserve {additional} additional elements or bytes: {source}"
            ),
            Self::Malformed(message) => write!(f, "malformed IPC payload: {message}"),
            Self::InvalidJournal(message) => write!(f, "invalid replay journal: {message}"),
        }
    }
}

impl std::error::Error for ProtocolError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Serial(error) => Some(error),
            Self::StorageUnavailable { source, .. } => Some(source),
            Self::InvalidJournal(error) => Some(error),
            _ => None,
        }
    }
}

impl From<SerialError> for ProtocolError {
    fn from(error: SerialError) -> Self {
        Self::Serial(error)
    }
}

impl From<JournalError> for ProtocolError {
    fn from(error: JournalError) -> Self {
        Self::InvalidJournal(error)
    }
}

/// Outer pipe framing failure.
#[derive(Debug)]
pub enum FramingError {
    /// The peer closed cleanly before another frame began.
    Closed,
    /// The pipe failed.
    Io(std::io::Error),
    /// The length prefix exceeded the negotiated message budget.
    FrameTooLarge {
        /// Ceiling.
        limit: usize,
        /// Declared frame length.
        needed: u64,
    },
    /// Storage for an admitted frame could not be reserved.
    FrameStorageAllocationFailed {
        /// Declared frame bytes requested from the allocator.
        bytes: usize,
        /// Allocation refusal.
        error: TryReserveError,
    },
    /// A `Read` implementation returned more bytes than its supplied buffer.
    InvalidReadCount {
        /// Bytes available in the supplied buffer.
        requested: usize,
        /// Byte count reported by the reader.
        returned: usize,
    },
    /// The canonical payload was invalid.
    Protocol(ProtocolError),
}

impl fmt::Display for FramingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Closed => f.write_str("worker IPC pipe closed"),
            Self::Io(error) => write!(f, "worker IPC pipe failed: {error}"),
            Self::FrameTooLarge { limit, needed } => write!(
                f,
                "worker IPC frame declares {needed} bytes, over the {limit}-byte limit"
            ),
            Self::FrameStorageAllocationFailed { bytes, error } => write!(
                f,
                "worker IPC frame could not reserve {bytes} bytes of storage: {error}"
            ),
            Self::InvalidReadCount {
                requested,
                returned,
            } => write!(
                f,
                "IPC reader violated Read by returning {returned} bytes for a {requested}-byte buffer"
            ),
            Self::Protocol(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for FramingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::FrameStorageAllocationFailed { error, .. } => Some(error),
            Self::Protocol(error) => Some(error),
            Self::Closed | Self::FrameTooLarge { .. } | Self::InvalidReadCount { .. } => None,
        }
    }
}

impl From<std::io::Error> for FramingError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<ProtocolError> for FramingError {
    fn from(error: ProtocolError) -> Self {
        Self::Protocol(error)
    }
}

/// Which frame-transfer mechanisms a worker can serve.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransportCapabilities {
    /// Canonical bytes inline on the control pipe.
    pub pipe: bool,
    /// An out-of-band region identified by an opaque token.
    pub shared_memory: bool,
}

impl TransportCapabilities {
    const PIPE_BIT: u8 = 1;
    const SHARED_MEMORY_BIT: u8 = 2;

    fn bits(self) -> u8 {
        (u8::from(self.pipe) * Self::PIPE_BIT)
            | (u8::from(self.shared_memory) * Self::SHARED_MEMORY_BIT)
    }

    fn from_bits(bits: u8) -> Result<Self, ProtocolError> {
        if bits & !(Self::PIPE_BIT | Self::SHARED_MEMORY_BIT) != 0 {
            return Err(ProtocolError::Malformed("frame transport capability bits"));
        }
        Ok(Self {
            pipe: bits & Self::PIPE_BIT != 0,
            shared_memory: bits & Self::SHARED_MEMORY_BIT != 0,
        })
    }
}

/// Encoding of a streamed preview frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameEncoding {
    /// Canonical PNG bytes.
    Png,
    /// Tight or explicitly-strided top-row-first RGBA8.
    Rgba8,
}

impl FrameEncoding {
    const fn code(self) -> u8 {
        match self {
            Self::Png => 0,
            Self::Rgba8 => 1,
        }
    }

    fn from_code(code: u8) -> Result<Self, ProtocolError> {
        match code {
            0 => Ok(Self::Png),
            1 => Ok(Self::Rgba8),
            _ => Err(ProtocolError::Malformed("frame encoding")),
        }
    }
}

/// Where a frame's bytes live.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FramePayload {
    /// Bytes carried inline on the bounded pipe.
    Pipe {
        /// Encoded payload.
        bytes: Vec<u8>,
        /// Content digest, checked on receipt.
        digest: Digest,
    },
    /// Bytes held in a host-mapped shared region.
    SharedMemory {
        /// Opaque, unforgeable region token.  It is never interpreted as a
        /// filesystem path.
        token: Digest,
        /// Mapped payload length.
        len: u64,
        /// Digest the mapper verifies after reading the region.
        digest: Digest,
    },
}

impl FramePayload {
    fn len(&self) -> Result<usize, ProtocolError> {
        match self {
            Self::Pipe { bytes, .. } => Ok(bytes.len()),
            Self::SharedMemory { len, .. } => usize::try_from(*len)
                .map_err(|_| ProtocolError::Malformed("shared frame length overflows usize")),
        }
    }
}

/// One frame-stream event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameStream {
    /// Scene whose renderer produced the frame.
    pub scene: String,
    /// Global, zero-based preview frame index.
    pub frame_index: u64,
    /// Pixel width.
    pub width: u32,
    /// Pixel height.
    pub height: u32,
    /// Bytes per row for raw RGBA8; zero for PNG.
    pub stride: u32,
    /// Payload encoding.
    pub encoding: FrameEncoding,
    /// Pipe or shared-memory transport.
    pub payload: FramePayload,
    /// Renderer/backend identities that actually participated in this frame.
    pub render_backends: Vec<RenderBackendRecord>,
}

impl FrameStream {
    /// Recheck dimensions, budgets, and inline content integrity.
    pub fn validate(&self, limits: ProtocolLimits) -> Result<(), ProtocolError> {
        require_scene(&self.scene)?;
        if self.width == 0 || self.height == 0 {
            return Err(ProtocolError::Malformed("zero-sized frame"));
        }
        let payload_len = self.payload.len()?;
        limit_payload("frame", payload_len, limits.max_frame_bytes)?;
        limit_count(
            "frame render backend",
            self.render_backends.len(),
            MAX_FRAME_RENDER_BACKENDS,
        )?;
        if self.render_backends.is_empty() {
            return Err(ProtocolError::Malformed(
                "frame has no render backend identity",
            ));
        }
        for backend in &self.render_backends {
            backend.validate()?;
        }
        match &self.payload {
            FramePayload::Pipe { bytes, digest } => {
                // ubs:ignore - public content-integrity digest, not an authentication secret.
                if sha256(bytes) != *digest {
                    return Err(ProtocolError::Malformed("inline frame digest"));
                }
            }
            FramePayload::SharedMemory { .. } => {}
        }
        match self.encoding {
            FrameEncoding::Png => {
                if self.stride != 0 {
                    return Err(ProtocolError::Malformed("PNG frame has a stride"));
                }
                if let FramePayload::Pipe { bytes, .. } = &self.payload
                    && !bytes.starts_with(b"\x89PNG\r\n\x1a\n")
                {
                    return Err(ProtocolError::Malformed("inline PNG signature"));
                }
            }
            FrameEncoding::Rgba8 => {
                let row = self
                    .width
                    .checked_mul(4)
                    .ok_or(ProtocolError::Malformed("RGBA row overflow"))?;
                if self.stride < row {
                    return Err(ProtocolError::Malformed("RGBA stride too small"));
                }
                let expected = u64::from(self.stride)
                    .checked_mul(u64::from(self.height))
                    .ok_or(ProtocolError::Malformed("RGBA payload length overflow"))?;
                if u64::try_from(payload_len).ok() != Some(expected) {
                    return Err(ProtocolError::Malformed("RGBA payload length"));
                }
            }
        }
        Ok(())
    }
}

/// A durable worker checkpoint pushed to, or restored by, the supervisor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Checkpoint {
    /// Scene identity.
    pub scene: String,
    /// Journal entry after which this state was captured.
    pub after_entry: u64,
    /// Digest of `state`.
    pub state_hash: Digest,
    /// Canonical `SceneState` bytes.
    pub state: Vec<u8>,
}

impl Checkpoint {
    fn validate(&self, limits: ProtocolLimits) -> Result<(), ProtocolError> {
        require_scene(&self.scene)?;
        limit_payload("checkpoint", self.state.len(), limits.max_checkpoint_bytes)?;
        if sha256(&self.state) != self.state_hash {
            return Err(ProtocolError::Malformed("checkpoint state hash"));
        }
        Ok(())
    }
}

/// A range of a canonical journal to re-execute.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JournalReplay {
    /// Scene identity.
    pub scene: String,
    /// First entry to execute, inclusive.
    pub from_entry: u64,
    /// End entry, exclusive.
    pub through_entry: u64,
    /// Canonical complete journal.  Ranges refer to this record.
    pub journal: Vec<u8>,
}

impl JournalReplay {
    fn validate(&self, limits: ProtocolLimits) -> Result<(), ProtocolError> {
        require_scene(&self.scene)?;
        if self.from_entry > self.through_entry {
            return Err(ProtocolError::Malformed("replay range is reversed"));
        }
        limit_payload("journal", self.journal.len(), limits.max_journal_bytes)?;
        let journal = Journal::from_bytes(&self.journal)?;
        let end = usize::try_from(self.through_entry)
            .map_err(|_| ProtocolError::Malformed("replay range overflows usize"))?;
        if end > journal.entries().len() {
            return Err(ProtocolError::Malformed("replay range exceeds journal"));
        }
        Ok(())
    }
}

/// A structured worker crash report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrashReport {
    /// Active scene, when known.
    pub scene: Option<String>,
    /// Panic/termination diagnostic.
    pub message: String,
    /// Canonical journal tail or the best bounded diagnostic tail available.
    pub journal_tail: Vec<u8>,
    /// Last known state hash.
    pub state_hash: Option<Digest>,
}

impl CrashReport {
    fn validate(&self, limits: ProtocolLimits) -> Result<(), ProtocolError> {
        if let Some(scene) = &self.scene {
            require_scene(scene)?;
        }
        if self.message.is_empty() {
            return Err(ProtocolError::Malformed("empty crash message"));
        }
        limit_payload(
            "crash message",
            self.message.len(),
            limits.max_crash_message_bytes,
        )?;
        limit_payload(
            "crash journal tail",
            self.journal_tail.len(),
            limits.max_crash_tail_bytes,
        )
    }
}

/// Requested render-debug layers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DebugLayerSet(u8);

impl DebugLayerSet {
    /// No overlays.
    pub const NONE: Self = Self(0);
    /// Raster tile boundaries.
    pub const TILES: Self = Self(1 << 0);
    /// Mobject control points and cages.
    pub const CONTROL_POINTS: Self = Self(1 << 1);
    /// Mobject bounding boxes.
    pub const BOUNDING_BOXES: Self = Self(1 << 2);
    /// Path winding direction.
    pub const WINDING: Self = Self(1 << 3);
    /// Depth and z-order diagnostics.
    pub const DEPTH: Self = Self(1 << 4);
    /// Every defined layer.
    pub const ALL: Self = Self(
        Self::TILES.0
            | Self::CONTROL_POINTS.0
            | Self::BOUNDING_BOXES.0
            | Self::WINDING.0
            | Self::DEPTH.0,
    );

    /// Construct after rejecting unknown bits.
    pub fn from_bits(bits: u8) -> Result<Self, ProtocolError> {
        if bits & !Self::ALL.0 != 0 {
            Err(ProtocolError::Malformed("debug layer bits"))
        } else {
            Ok(Self(bits))
        }
    }

    /// Stable wire representation.
    #[must_use]
    pub const fn bits(self) -> u8 {
        self.0
    }

    /// Whether every bit in `other` is enabled.
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl std::ops::BitOr for DebugLayerSet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

/// One supervisor request.
#[derive(Clone, Debug, PartialEq)]
pub enum SupervisorRequest {
    /// Mandatory first request.
    Hello {
        /// Explicit live-protocol version.
        version: ProtocolVersion,
        /// Supervisor build/content identity.
        supervisor_build: Digest,
        /// Largest frame the supervisor will accept.
        max_frame_bytes: u64,
    },
    /// Enumerate scene names in source order.
    EnumerateScenes,
    /// Advance one declared scene command.
    Play {
        /// Scene identity.
        scene: String,
        /// Expected command identity from the compiled scene.
        command: CommandRecord,
    },
    /// Seek deterministically to a global frame.
    Seek {
        /// Scene identity.
        scene: String,
        /// Global frame.
        frame: i64,
    },
    /// Interactive scrub request (kept distinct from a committed seek).
    Scrub {
        /// Scene identity.
        scene: String,
        /// Global frame.
        frame: i64,
    },
    /// Restore a supervisor-owned durable checkpoint.
    RestoreCheckpoint(Checkpoint),
    /// Re-execute a verified journal suffix.
    ReplayJournal(JournalReplay),
    /// Submit one validated host event to the isolated scene worker.
    Event {
        /// Scene identity.
        scene: String,
        /// Platform-neutral event payload.
        event: EventPayload,
    },
    /// Capture a bounded mobject-inspector document.
    Inspect {
        /// Scene identity.
        scene: String,
    },
    /// Capture bounded debug-overlay geometry.
    Overlay {
        /// Scene identity.
        scene: String,
        /// Requested diagnostic layers.
        layers: DebugLayerSet,
    },
    /// Graceful worker shutdown.
    Shutdown,
}

impl SupervisorRequest {
    fn validate(&self, limits: ProtocolLimits) -> Result<(), ProtocolError> {
        match self {
            Self::Hello {
                max_frame_bytes, ..
            } => {
                if *max_frame_bytes == 0 {
                    return Err(ProtocolError::Malformed("zero frame budget"));
                }
            }
            Self::EnumerateScenes | Self::Shutdown => {}
            Self::Play { scene, command } => {
                require_scene(scene)?;
                if command.label.is_empty() {
                    return Err(ProtocolError::Malformed("empty command label"));
                }
            }
            Self::Seek { scene, frame } | Self::Scrub { scene, frame } => {
                require_scene(scene)?;
                if *frame < 0 {
                    return Err(ProtocolError::Malformed("negative frame index"));
                }
            }
            Self::RestoreCheckpoint(checkpoint) => checkpoint.validate(limits)?,
            Self::ReplayJournal(replay) => replay.validate(limits)?,
            Self::Event { scene, event } => {
                require_scene(scene)?;
                event
                    .validate()
                    .map_err(|_| ProtocolError::Malformed("invalid input event"))?;
            }
            Self::Inspect { scene } => require_scene(scene)?,
            Self::Overlay { scene, layers } => {
                require_scene(scene)?;
                DebugLayerSet::from_bits(layers.bits())?;
            }
        }
        Ok(())
    }
}

/// Stable worker-side error classes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkerErrorCode {
    /// Request was illegal in the current session state.
    InvalidRequest,
    /// Named scene was not present.
    SceneNotFound,
    /// Checkpoint could not be restored.
    CheckpointRejected,
    /// Journal replay failed.
    ReplayFailed,
    /// Peer version mismatch.
    VersionSkew,
    /// Scene/renderer integration failed.
    ExecutionFailed,
}

impl WorkerErrorCode {
    const fn code(self) -> u16 {
        match self {
            Self::InvalidRequest => 1,
            Self::SceneNotFound => 2,
            Self::CheckpointRejected => 3,
            Self::ReplayFailed => 4,
            Self::VersionSkew => 5,
            Self::ExecutionFailed => 6,
        }
    }

    fn from_code(code: u16) -> Result<Self, ProtocolError> {
        match code {
            1 => Ok(Self::InvalidRequest),
            2 => Ok(Self::SceneNotFound),
            3 => Ok(Self::CheckpointRejected),
            4 => Ok(Self::ReplayFailed),
            5 => Ok(Self::VersionSkew),
            6 => Ok(Self::ExecutionFailed),
            _ => Err(ProtocolError::Malformed("worker error code")),
        }
    }
}

/// Worker-produced Studio document class.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StudioDataKind {
    /// Mobject family, record, uniform, and source-span inspection.
    Inspection,
    /// Render-debug overlay geometry.
    Overlay,
}

impl StudioDataKind {
    const fn code(self) -> u8 {
        match self {
            Self::Inspection => 0,
            Self::Overlay => 1,
        }
    }

    fn from_code(code: u8) -> Result<Self, ProtocolError> {
        match code {
            0 => Ok(Self::Inspection),
            1 => Ok(Self::Overlay),
            _ => Err(ProtocolError::Malformed("Studio data kind")),
        }
    }
}

/// One worker response/event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkerResponse {
    /// Mandatory handshake response.
    Hello {
        /// Exact live-protocol version.
        version: ProtocolVersion,
        /// Worker executable/content identity.
        worker_build: Digest,
        /// Available frame transports.
        transports: TransportCapabilities,
    },
    /// Ordered scene names.
    Scenes(Vec<String>),
    /// Command completed.
    Ack {
        /// State after the command, when state-producing.
        state_hash: Option<Digest>,
        /// Journal length after the command.
        journal_len: u64,
    },
    /// Preview/export frame.
    Frame(FrameStream),
    /// Newly captured checkpoint for supervisor storage.
    Checkpoint(Checkpoint),
    /// Newly recorded journal segment.
    JournalSegment {
        /// Scene identity.
        scene: String,
        /// Global entry index represented by segment entry zero.
        start_entry: u64,
        /// Canonical standalone `Journal`.
        journal: Vec<u8>,
    },
    /// State hashes produced by one replay request, in entry order.
    ReplayComplete {
        /// First replayed entry.
        from_entry: u64,
        /// One hash per replayed entry.
        state_hashes: Vec<Digest>,
    },
    /// Scene code panicked or the worker is terminating abnormally.
    Crash(CrashReport),
    /// Typed worker refusal.
    Error {
        /// Stable class.
        code: WorkerErrorCode,
        /// Human diagnostic.
        message: String,
    },
    /// Bounded UTF-8 JSON produced inside the isolated scene worker.
    StudioData {
        /// Scene identity.
        scene: String,
        /// Document class.
        kind: StudioDataKind,
        /// UTF-8 JSON bytes.
        bytes: Vec<u8>,
        /// Digest of `bytes`, checked on receipt.
        digest: Digest,
    },
    /// Graceful shutdown acknowledged.
    Bye,
}

impl WorkerResponse {
    pub(crate) fn validate(&self, limits: ProtocolLimits) -> Result<(), ProtocolError> {
        match self {
            Self::Hello { .. } | Self::Ack { .. } | Self::Bye => {}
            Self::Scenes(scenes) => {
                limit_count("scene", scenes.len(), limits.max_scenes)?;
                for scene in scenes {
                    require_scene(scene)?;
                }
            }
            Self::Frame(frame) => frame.validate(limits)?,
            Self::Checkpoint(checkpoint) => checkpoint.validate(limits)?,
            Self::JournalSegment { scene, journal, .. } => {
                require_scene(scene)?;
                limit_payload("journal", journal.len(), limits.max_journal_bytes)?;
                Journal::from_bytes(journal)?;
            }
            Self::ReplayComplete { state_hashes, .. } => {
                limit_count(
                    "replay state hash",
                    state_hashes.len(),
                    limits.max_replay_hashes,
                )?;
            }
            Self::Crash(report) => report.validate(limits)?,
            Self::Error { message, .. } => {
                if message.is_empty() {
                    return Err(ProtocolError::Malformed("empty worker error"));
                }
                limit_payload(
                    "worker error",
                    message.len(),
                    limits.max_error_message_bytes,
                )?;
            }
            Self::StudioData {
                scene,
                bytes,
                digest,
                ..
            } => {
                require_scene(scene)?;
                limit_payload("Studio data", bytes.len(), limits.max_studio_data_bytes)?;
                // ubs:ignore - public content-integrity digest, not an authentication secret.
                if sha256(bytes) != *digest {
                    return Err(ProtocolError::Malformed("Studio data digest"));
                }
                std::str::from_utf8(bytes)
                    .map_err(|_| ProtocolError::Malformed("Studio data is not UTF-8"))?;
                if !valid_json_container(bytes) {
                    return Err(ProtocolError::Malformed("Studio data is not valid JSON"));
                }
            }
        }
        Ok(())
    }
}

/// A request plus its monotonically increasing correlation id.
#[derive(Clone, Debug, PartialEq)]
pub struct RequestEnvelope {
    /// Correlation id.
    pub request_id: u64,
    /// Request.
    pub request: SupervisorRequest,
}

impl RequestEnvelope {
    /// Encode canonically.
    pub fn to_bytes(&self, limits: ProtocolLimits) -> Result<Vec<u8>, ProtocolError> {
        self.request.validate(limits)?;
        let mut writer = Writer::with_limits(REQUEST_SCHEMA, limits.serial());
        writer.put_u64(self.request_id);
        put_request(&mut writer, &self.request);
        Ok(writer.finish()?)
    }

    /// Decode and validate canonically.
    pub fn from_bytes(bytes: &[u8], limits: ProtocolLimits) -> Result<Self, ProtocolError> {
        let mut reader = Reader::open(
            bytes,
            REQUEST_SCHEMA,
            limits.serial(),
            UnknownPolicy::Strict,
        )?;
        let request_id = reader.get_u64()?;
        let request = get_request(&mut reader, limits)?;
        reader.finish()?;
        request.validate(limits)?;
        Ok(Self {
            request_id,
            request,
        })
    }
}

/// A response correlated to exactly one request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResponseEnvelope {
    /// Correlation id copied from the request.
    pub request_id: u64,
    /// Response/event.
    pub response: WorkerResponse,
}

impl ResponseEnvelope {
    /// Encode canonically.
    pub fn to_bytes(&self, limits: ProtocolLimits) -> Result<Vec<u8>, ProtocolError> {
        self.response.validate(limits)?;
        let mut writer = Writer::with_limits(RESPONSE_SCHEMA, limits.serial());
        writer.put_u64(self.request_id);
        put_response(&mut writer, &self.response)?;
        Ok(writer.finish()?)
    }

    /// Decode and validate canonically.
    pub fn from_bytes(bytes: &[u8], limits: ProtocolLimits) -> Result<Self, ProtocolError> {
        let mut reader = Reader::open(
            bytes,
            RESPONSE_SCHEMA,
            limits.serial(),
            UnknownPolicy::Strict,
        )?;
        let request_id = reader.get_u64()?;
        let response = get_response(&mut reader, limits)?;
        reader.finish()?;
        response.validate(limits)?;
        Ok(Self {
            request_id,
            response,
        })
    }
}

/// Write one length-framed request.
pub fn write_request(
    writer: &mut impl Write,
    request: &RequestEnvelope,
    limits: ProtocolLimits,
) -> Result<(), FramingError> {
    write_document(writer, &request.to_bytes(limits)?, limits)
}

/// Read one length-framed request.
pub fn read_request(
    reader: &mut impl Read,
    limits: ProtocolLimits,
) -> Result<RequestEnvelope, FramingError> {
    let bytes = read_document(reader, limits)?;
    Ok(RequestEnvelope::from_bytes(&bytes, limits)?)
}

/// Write one length-framed response.
pub fn write_response(
    writer: &mut impl Write,
    response: &ResponseEnvelope,
    limits: ProtocolLimits,
) -> Result<(), FramingError> {
    write_document(writer, &response.to_bytes(limits)?, limits)
}

/// Read one length-framed response.
pub fn read_response(
    reader: &mut impl Read,
    limits: ProtocolLimits,
) -> Result<ResponseEnvelope, FramingError> {
    let bytes = read_document(reader, limits)?;
    Ok(ResponseEnvelope::from_bytes(&bytes, limits)?)
}

pub(crate) fn write_document(
    writer: &mut impl Write,
    bytes: &[u8],
    limits: ProtocolLimits,
) -> Result<(), FramingError> {
    if bytes.len() > limits.max_message_bytes {
        let needed = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        return Err(FramingError::FrameTooLarge {
            limit: limits.max_message_bytes,
            needed,
        });
    }
    let len = u64::try_from(bytes.len()).map_err(|_| FramingError::FrameTooLarge {
        limit: limits.max_message_bytes,
        needed: u64::MAX,
    })?;
    writer.write_all(&len.to_le_bytes())?;
    writer.write_all(bytes)?;
    writer.flush()?;
    Ok(())
}

fn read_document(reader: &mut impl Read, limits: ProtocolLimits) -> Result<Vec<u8>, FramingError> {
    let mut prefix = [0u8; 8];
    loop {
        match read_checked(reader, &mut prefix[..1]) {
            Ok(0) => return Err(FramingError::Closed),
            Ok(_) => break,
            Err(FramingError::Io(error)) if error.kind() == std::io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error),
        }
    }
    read_exact_checked(reader, &mut prefix[1..])?;
    let declared = u64::from_le_bytes(prefix);
    let message_limit = u64::try_from(limits.max_message_bytes).unwrap_or(u64::MAX);
    if declared > message_limit {
        return Err(FramingError::FrameTooLarge {
            limit: limits.max_message_bytes,
            needed: declared,
        });
    }
    let len = usize::try_from(declared).map_err(|_| FramingError::FrameTooLarge {
        limit: limits.max_message_bytes,
        needed: declared,
    })?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(len)
        .map_err(|error| FramingError::FrameStorageAllocationFailed { bytes: len, error })?;
    bytes.resize(len, 0);
    read_exact_checked(reader, &mut bytes)?;
    Ok(bytes)
}

fn read_checked(reader: &mut impl Read, buffer: &mut [u8]) -> Result<usize, FramingError> {
    let requested = buffer.len();
    let returned = reader.read(buffer)?;
    if returned > requested {
        return Err(FramingError::InvalidReadCount {
            requested,
            returned,
        });
    }
    Ok(returned)
}

fn read_exact_checked(reader: &mut impl Read, mut buffer: &mut [u8]) -> Result<(), FramingError> {
    while !buffer.is_empty() {
        match read_checked(reader, buffer) {
            Ok(0) => {
                return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof).into());
            }
            Ok(count) => buffer = &mut buffer[count..],
            Err(FramingError::Io(error)) if error.kind() == std::io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

fn put_request(writer: &mut Writer, request: &SupervisorRequest) {
    match request {
        SupervisorRequest::Hello {
            version,
            supervisor_build,
            max_frame_bytes,
        } => {
            writer.put_u8(0);
            put_version(writer, *version);
            writer.put_digest(supervisor_build);
            writer.put_u64(*max_frame_bytes);
        }
        SupervisorRequest::EnumerateScenes => {
            writer.put_u8(1);
        }
        SupervisorRequest::Play { scene, command } => {
            writer.put_u8(2);
            writer.put_str(scene);
            put_command(writer, command);
        }
        SupervisorRequest::Seek { scene, frame } => {
            writer.put_u8(3);
            writer.put_str(scene);
            writer.put_i64(*frame);
        }
        SupervisorRequest::Scrub { scene, frame } => {
            writer.put_u8(4);
            writer.put_str(scene);
            writer.put_i64(*frame);
        }
        SupervisorRequest::RestoreCheckpoint(checkpoint) => {
            writer.put_u8(5);
            put_checkpoint(writer, checkpoint);
        }
        SupervisorRequest::ReplayJournal(replay) => {
            writer.put_u8(6);
            put_replay(writer, replay);
        }
        SupervisorRequest::Shutdown => {
            writer.put_u8(7);
        }
        SupervisorRequest::Event { scene, event } => {
            writer.put_u8(8);
            writer.put_str(scene);
            put_event(writer, event);
        }
        SupervisorRequest::Inspect { scene } => {
            writer.put_u8(9);
            writer.put_str(scene);
        }
        SupervisorRequest::Overlay { scene, layers } => {
            writer.put_u8(10);
            writer.put_str(scene);
            writer.put_u8(layers.bits());
        }
    }
}

fn get_request(
    reader: &mut Reader<'_>,
    limits: ProtocolLimits,
) -> Result<SupervisorRequest, ProtocolError> {
    match reader.get_u8()? {
        0 => Ok(SupervisorRequest::Hello {
            version: get_version(reader)?,
            supervisor_build: reader.get_digest()?,
            max_frame_bytes: reader.get_u64()?,
        }),
        1 => Ok(SupervisorRequest::EnumerateScenes),
        2 => Ok(SupervisorRequest::Play {
            scene: owned_string(reader.get_str()?, "scene")?,
            command: get_command(reader)?,
        }),
        3 => Ok(SupervisorRequest::Seek {
            scene: owned_string(reader.get_str()?, "scene")?,
            frame: reader.get_i64()?,
        }),
        4 => Ok(SupervisorRequest::Scrub {
            scene: owned_string(reader.get_str()?, "scene")?,
            frame: reader.get_i64()?,
        }),
        5 => Ok(SupervisorRequest::RestoreCheckpoint(get_checkpoint(
            reader, limits,
        )?)),
        6 => Ok(SupervisorRequest::ReplayJournal(get_replay(
            reader, limits,
        )?)),
        7 => Ok(SupervisorRequest::Shutdown),
        8 => Ok(SupervisorRequest::Event {
            scene: owned_string(reader.get_str()?, "scene")?,
            event: get_event(reader)?,
        }),
        9 => Ok(SupervisorRequest::Inspect {
            scene: owned_string(reader.get_str()?, "scene")?,
        }),
        10 => Ok(SupervisorRequest::Overlay {
            scene: owned_string(reader.get_str()?, "scene")?,
            layers: DebugLayerSet::from_bits(reader.get_u8()?)?,
        }),
        _ => Err(ProtocolError::Malformed("supervisor request tag")),
    }
}

fn put_response(writer: &mut Writer, response: &WorkerResponse) -> Result<(), ProtocolError> {
    match response {
        WorkerResponse::Hello {
            version,
            worker_build,
            transports,
        } => {
            writer.put_u8(0);
            put_version(writer, *version);
            writer.put_digest(worker_build);
            writer.put_u8(transports.bits());
        }
        WorkerResponse::Scenes(scenes) => {
            writer.put_u8(1);
            writer.put_u32(wire_count("scene", scenes.len())?);
            for scene in scenes {
                writer.put_str(scene);
            }
        }
        WorkerResponse::Ack {
            state_hash,
            journal_len,
        } => {
            writer.put_u8(2);
            put_optional_digest(writer, state_hash);
            writer.put_u64(*journal_len);
        }
        WorkerResponse::Frame(frame) => {
            writer.put_u8(3);
            put_frame(writer, frame)?;
        }
        WorkerResponse::Checkpoint(checkpoint) => {
            writer.put_u8(4);
            put_checkpoint(writer, checkpoint);
        }
        WorkerResponse::JournalSegment {
            scene,
            start_entry,
            journal,
        } => {
            writer.put_u8(5);
            writer.put_str(scene);
            writer.put_u64(*start_entry);
            writer.put_bytes(journal);
        }
        WorkerResponse::ReplayComplete {
            from_entry,
            state_hashes,
        } => {
            writer.put_u8(6);
            writer.put_u64(*from_entry);
            writer.put_u32(wire_count("replay state hash", state_hashes.len())?);
            for hash in state_hashes {
                writer.put_digest(hash);
            }
        }
        WorkerResponse::Crash(report) => {
            writer.put_u8(7);
            put_crash(writer, report);
        }
        WorkerResponse::Error { code, message } => {
            writer.put_u8(8);
            writer.put_u16(code.code());
            writer.put_str(message);
        }
        WorkerResponse::Bye => {
            writer.put_u8(9);
        }
        WorkerResponse::StudioData {
            scene,
            kind,
            bytes,
            digest,
        } => {
            writer.put_u8(10);
            writer.put_str(scene);
            writer.put_u8(kind.code());
            writer.put_digest(digest);
            writer.put_bytes(bytes);
        }
    }
    Ok(())
}

fn get_response(
    reader: &mut Reader<'_>,
    limits: ProtocolLimits,
) -> Result<WorkerResponse, ProtocolError> {
    match reader.get_u8()? {
        0 => Ok(WorkerResponse::Hello {
            version: get_version(reader)?,
            worker_build: reader.get_digest()?,
            transports: TransportCapabilities::from_bits(reader.get_u8()?)?,
        }),
        1 => {
            let count = count(reader.get_u32()?, "scene", limits.max_scenes)?;
            require_collection_payload(reader, "scene", count, 8)?;
            let mut scenes = vec_with_capacity(count, "scene")?;
            for _ in 0..count {
                scenes.push(owned_string(reader.get_str()?, "scene")?);
            }
            Ok(WorkerResponse::Scenes(scenes))
        }
        2 => Ok(WorkerResponse::Ack {
            state_hash: get_optional_digest(reader)?,
            journal_len: reader.get_u64()?,
        }),
        3 => Ok(WorkerResponse::Frame(get_frame(reader, limits)?)),
        4 => Ok(WorkerResponse::Checkpoint(get_checkpoint(reader, limits)?)),
        5 => Ok(WorkerResponse::JournalSegment {
            scene: owned_string(reader.get_str()?, "scene")?,
            start_entry: reader.get_u64()?,
            journal: bounded_bytes(reader, "journal", limits.max_journal_bytes)?,
        }),
        6 => {
            let from_entry = reader.get_u64()?;
            let count = count(
                reader.get_u32()?,
                "replay state hash",
                limits.max_replay_hashes,
            )?;
            require_collection_payload(reader, "replay state hash", count, 32)?;
            let mut state_hashes = vec_with_capacity(count, "replay state hash")?;
            for _ in 0..count {
                state_hashes.push(reader.get_digest()?);
            }
            Ok(WorkerResponse::ReplayComplete {
                from_entry,
                state_hashes,
            })
        }
        7 => Ok(WorkerResponse::Crash(get_crash(reader, limits)?)),
        8 => Ok(WorkerResponse::Error {
            code: WorkerErrorCode::from_code(reader.get_u16()?)?,
            message: bounded_string(reader, "worker error", limits.max_error_message_bytes)?,
        }),
        9 => Ok(WorkerResponse::Bye),
        10 => Ok(WorkerResponse::StudioData {
            scene: owned_string(reader.get_str()?, "scene")?,
            kind: StudioDataKind::from_code(reader.get_u8()?)?,
            digest: reader.get_digest()?,
            bytes: bounded_bytes(reader, "Studio data", limits.max_studio_data_bytes)?,
        }),
        _ => Err(ProtocolError::Malformed("worker response tag")),
    }
}

fn put_version(writer: &mut Writer, version: ProtocolVersion) {
    writer.put_u16(version.major);
    writer.put_u16(version.minor);
}

fn get_version(reader: &mut Reader<'_>) -> Result<ProtocolVersion, ProtocolError> {
    Ok(ProtocolVersion {
        major: reader.get_u16()?,
        minor: reader.get_u16()?,
    })
}

fn put_command(writer: &mut Writer, command: &CommandRecord) {
    writer.put_u8(command_kind_code(command.kind));
    writer.put_digest(&command.identity);
    writer.put_str(&command.label);
}

fn get_command(reader: &mut Reader<'_>) -> Result<CommandRecord, ProtocolError> {
    Ok(CommandRecord {
        kind: command_kind_from_code(reader.get_u8()?)?,
        identity: reader.get_digest()?,
        label: owned_string(reader.get_str()?, "command label")?,
    })
}

const fn command_kind_code(kind: CommandKind) -> u8 {
    match kind {
        CommandKind::Play => 0,
        CommandKind::Wait => 1,
        CommandKind::Add => 2,
        CommandKind::Remove => 3,
        CommandKind::CameraChange => 4,
        CommandKind::Sound => 5,
        CommandKind::Custom => 6,
        CommandKind::Input => 7,
    }
}

fn command_kind_from_code(code: u8) -> Result<CommandKind, ProtocolError> {
    match code {
        0 => Ok(CommandKind::Play),
        1 => Ok(CommandKind::Wait),
        2 => Ok(CommandKind::Add),
        3 => Ok(CommandKind::Remove),
        4 => Ok(CommandKind::CameraChange),
        5 => Ok(CommandKind::Sound),
        6 => Ok(CommandKind::Custom),
        7 => Ok(CommandKind::Input),
        _ => Err(ProtocolError::Malformed("command kind")),
    }
}

fn put_event(writer: &mut Writer, event: &EventPayload) {
    match event {
        EventPayload::MouseMotion {
            point,
            delta,
            modifiers,
        } => {
            writer.put_u8(0);
            put_vec3(writer, *point);
            put_vec3(writer, *delta);
            writer.put_u8(modifiers.bits());
        }
        EventPayload::MousePress {
            point,
            button,
            modifiers,
        } => {
            writer.put_u8(1);
            put_vec3(writer, *point);
            put_mouse_button(writer, *button);
            writer.put_u8(modifiers.bits());
        }
        EventPayload::MouseRelease {
            point,
            button,
            modifiers,
        } => {
            writer.put_u8(2);
            put_vec3(writer, *point);
            put_mouse_button(writer, *button);
            writer.put_u8(modifiers.bits());
        }
        EventPayload::MouseDrag {
            point,
            delta,
            button,
            modifiers,
        } => {
            writer.put_u8(3);
            put_vec3(writer, *point);
            put_vec3(writer, *delta);
            put_mouse_button(writer, *button);
            writer.put_u8(modifiers.bits());
        }
        EventPayload::MouseScroll {
            point,
            offset,
            modifiers,
        } => {
            writer.put_u8(4);
            put_vec3(writer, *point);
            writer.put_f64(offset[0]);
            writer.put_f64(offset[1]);
            writer.put_u8(modifiers.bits());
        }
        EventPayload::KeyPress { key, modifiers } => {
            writer.put_u8(5);
            put_key(writer, *key);
            writer.put_u8(modifiers.bits());
        }
        EventPayload::KeyRelease { key, modifiers } => {
            writer.put_u8(6);
            put_key(writer, *key);
            writer.put_u8(modifiers.bits());
        }
    }
}

fn get_event(reader: &mut Reader<'_>) -> Result<EventPayload, ProtocolError> {
    let event = match reader.get_u8()? {
        0 => EventPayload::MouseMotion {
            point: get_vec3(reader)?,
            delta: get_vec3(reader)?,
            modifiers: get_modifiers(reader)?,
        },
        1 => EventPayload::MousePress {
            point: get_vec3(reader)?,
            button: get_mouse_button(reader)?,
            modifiers: get_modifiers(reader)?,
        },
        2 => EventPayload::MouseRelease {
            point: get_vec3(reader)?,
            button: get_mouse_button(reader)?,
            modifiers: get_modifiers(reader)?,
        },
        3 => EventPayload::MouseDrag {
            point: get_vec3(reader)?,
            delta: get_vec3(reader)?,
            button: get_mouse_button(reader)?,
            modifiers: get_modifiers(reader)?,
        },
        4 => EventPayload::MouseScroll {
            point: get_vec3(reader)?,
            offset: [reader.get_f64()?, reader.get_f64()?],
            modifiers: get_modifiers(reader)?,
        },
        5 => EventPayload::KeyPress {
            key: get_key(reader)?,
            modifiers: get_modifiers(reader)?,
        },
        6 => EventPayload::KeyRelease {
            key: get_key(reader)?,
            modifiers: get_modifiers(reader)?,
        },
        _ => return Err(ProtocolError::Malformed("input event tag")),
    };
    event
        .validate()
        .map_err(|_| ProtocolError::Malformed("invalid input event"))?;
    Ok(event)
}

fn put_vec3(writer: &mut Writer, vector: [f64; 3]) {
    for value in vector {
        writer.put_f64(value);
    }
}

fn get_vec3(reader: &mut Reader<'_>) -> Result<[f64; 3], ProtocolError> {
    Ok([reader.get_f64()?, reader.get_f64()?, reader.get_f64()?])
}

fn put_mouse_button(writer: &mut Writer, button: MouseButton) {
    match button {
        MouseButton::Left => writer.put_u8(0).put_u16(0),
        MouseButton::Middle => writer.put_u8(1).put_u16(0),
        MouseButton::Right => writer.put_u8(2).put_u16(0),
        MouseButton::Other(value) => writer.put_u8(3).put_u16(value),
    };
}

fn get_mouse_button(reader: &mut Reader<'_>) -> Result<MouseButton, ProtocolError> {
    let tag = reader.get_u8()?;
    let value = reader.get_u16()?;
    match (tag, value) {
        (0, 0) => Ok(MouseButton::Left),
        (1, 0) => Ok(MouseButton::Middle),
        (2, 0) => Ok(MouseButton::Right),
        (3, value) => Ok(MouseButton::Other(value)),
        _ => Err(ProtocolError::Malformed("mouse button")),
    }
}

fn put_key(writer: &mut Writer, key: Key) {
    match key {
        Key::Character(character) => writer.put_u8(0).put_u32(u32::from(character)),
        Key::Backspace => writer.put_u8(1).put_u32(0),
        Key::ArrowLeft => writer.put_u8(2).put_u32(0),
        Key::ArrowUp => writer.put_u8(3).put_u32(0),
        Key::ArrowRight => writer.put_u8(4).put_u32(0),
        Key::ArrowDown => writer.put_u8(5).put_u32(0),
        Key::Escape => writer.put_u8(6).put_u32(0),
        Key::Enter => writer.put_u8(7).put_u32(0),
        Key::Tab => writer.put_u8(8).put_u32(0),
        Key::Other(value) => writer.put_u8(9).put_u32(value),
    };
}

fn get_key(reader: &mut Reader<'_>) -> Result<Key, ProtocolError> {
    let tag = reader.get_u8()?;
    let value = reader.get_u32()?;
    match (tag, value) {
        (0, value) => char::from_u32(value)
            .map(Key::Character)
            .ok_or(ProtocolError::Malformed("key character")),
        (1, 0) => Ok(Key::Backspace),
        (2, 0) => Ok(Key::ArrowLeft),
        (3, 0) => Ok(Key::ArrowUp),
        (4, 0) => Ok(Key::ArrowRight),
        (5, 0) => Ok(Key::ArrowDown),
        (6, 0) => Ok(Key::Escape),
        (7, 0) => Ok(Key::Enter),
        (8, 0) => Ok(Key::Tab),
        (9, value) => Ok(Key::Other(value)),
        _ => Err(ProtocolError::Malformed("key")),
    }
}

fn get_modifiers(reader: &mut Reader<'_>) -> Result<Modifiers, ProtocolError> {
    Modifiers::from_bits(reader.get_u8()?).map_err(|_| ProtocolError::Malformed("modifier bits"))
}

const MAX_STUDIO_JSON_DEPTH: usize = 128;

fn valid_json_container(bytes: &[u8]) -> bool {
    let mut parser = JsonParser { bytes, index: 0 };
    parser.skip_whitespace();
    if !matches!(parser.peek(), Some(b'{' | b'[')) || !parser.value(0) {
        return false;
    }
    parser.skip_whitespace();
    parser.index == bytes.len()
}

struct JsonParser<'a> {
    bytes: &'a [u8],
    index: usize,
}

impl JsonParser<'_> {
    fn value(&mut self, depth: usize) -> bool {
        if depth > MAX_STUDIO_JSON_DEPTH {
            return false;
        }
        self.skip_whitespace();
        match self.peek() {
            Some(b'{') => self.object(depth),
            Some(b'[') => self.array(depth),
            Some(b'"') => self.string(),
            Some(b'-' | b'0'..=b'9') => self.number(),
            Some(b't') => self.literal(b"true"),
            Some(b'f') => self.literal(b"false"),
            Some(b'n') => self.literal(b"null"),
            _ => false,
        }
    }

    fn object(&mut self, depth: usize) -> bool {
        self.index += 1;
        self.skip_whitespace();
        if self.consume(b'}') {
            return true;
        }
        loop {
            if !self.string() {
                return false;
            }
            self.skip_whitespace();
            if !self.consume(b':') || !self.value(depth + 1) {
                return false;
            }
            self.skip_whitespace();
            if self.consume(b'}') {
                return true;
            }
            if !self.consume(b',') {
                return false;
            }
            self.skip_whitespace();
        }
    }

    fn array(&mut self, depth: usize) -> bool {
        self.index += 1;
        self.skip_whitespace();
        if self.consume(b']') {
            return true;
        }
        loop {
            if !self.value(depth + 1) {
                return false;
            }
            self.skip_whitespace();
            if self.consume(b']') {
                return true;
            }
            if !self.consume(b',') {
                return false;
            }
        }
    }

    fn string(&mut self) -> bool {
        if !self.consume(b'"') {
            return false;
        }
        while let Some(byte) = self.peek() {
            self.index += 1;
            match byte {
                b'"' => return true,
                0x00..=0x1f => return false,
                b'\\' => {
                    let Some(escape) = self.peek() else {
                        return false;
                    };
                    self.index += 1;
                    match escape {
                        b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => {}
                        b'u' => {
                            for _ in 0..4 {
                                let Some(digit) = self.peek() else {
                                    return false;
                                };
                                if !digit.is_ascii_hexdigit() {
                                    return false;
                                }
                                self.index += 1;
                            }
                        }
                        _ => return false,
                    }
                }
                _ => {}
            }
        }
        false
    }

    fn number(&mut self) -> bool {
        self.consume(b'-');
        match self.peek() {
            Some(b'0') => {
                self.index += 1;
                if matches!(self.peek(), Some(b'0'..=b'9')) {
                    return false;
                }
            }
            Some(b'1'..=b'9') => {
                self.index += 1;
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.index += 1;
                }
            }
            _ => return false,
        }
        if self.consume(b'.') {
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return false;
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.index += 1;
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.index += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.index += 1;
            }
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return false;
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.index += 1;
            }
        }
        true
    }

    fn literal(&mut self, literal: &[u8]) -> bool {
        let Some(candidate) = self
            .bytes
            .get(self.index..self.index.saturating_add(literal.len()))
        else {
            return false;
        };
        if candidate != literal {
            return false;
        }
        self.index += literal.len();
        true
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.index += 1;
        }
    }

    fn consume(&mut self, byte: u8) -> bool {
        if self.peek() == Some(byte) {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.index).copied()
    }
}

fn put_checkpoint(writer: &mut Writer, checkpoint: &Checkpoint) {
    writer.put_str(&checkpoint.scene);
    writer.put_u64(checkpoint.after_entry);
    writer.put_digest(&checkpoint.state_hash);
    writer.put_bytes(&checkpoint.state);
}

fn get_checkpoint(
    reader: &mut Reader<'_>,
    limits: ProtocolLimits,
) -> Result<Checkpoint, ProtocolError> {
    Ok(Checkpoint {
        scene: owned_string(reader.get_str()?, "scene")?,
        after_entry: reader.get_u64()?,
        state_hash: reader.get_digest()?,
        state: bounded_bytes(reader, "checkpoint", limits.max_checkpoint_bytes)?,
    })
}

fn put_replay(writer: &mut Writer, replay: &JournalReplay) {
    writer.put_str(&replay.scene);
    writer.put_u64(replay.from_entry);
    writer.put_u64(replay.through_entry);
    writer.put_bytes(&replay.journal);
}

fn get_replay(
    reader: &mut Reader<'_>,
    limits: ProtocolLimits,
) -> Result<JournalReplay, ProtocolError> {
    Ok(JournalReplay {
        scene: owned_string(reader.get_str()?, "scene")?,
        from_entry: reader.get_u64()?,
        through_entry: reader.get_u64()?,
        journal: bounded_bytes(reader, "journal", limits.max_journal_bytes)?,
    })
}

fn put_frame(writer: &mut Writer, frame: &FrameStream) -> Result<(), ProtocolError> {
    writer.put_str(&frame.scene);
    writer.put_u64(frame.frame_index);
    writer.put_u32(frame.width);
    writer.put_u32(frame.height);
    writer.put_u32(frame.stride);
    writer.put_u8(frame.encoding.code());
    match &frame.payload {
        FramePayload::Pipe { bytes, digest } => {
            writer.put_u8(0);
            writer.put_digest(digest);
            writer.put_bytes(bytes);
        }
        FramePayload::SharedMemory { token, len, digest } => {
            writer.put_u8(1);
            writer.put_digest(token);
            writer.put_u64(*len);
            writer.put_digest(digest);
        }
    }
    writer.put_u32(wire_count(
        "frame render backend",
        frame.render_backends.len(),
    )?);
    for backend in &frame.render_backends {
        writer.put_u8(backend.role().wire_code());
        writer.put_digest(&backend.digest());
        writer.put_bytes(backend.identity());
    }
    Ok(())
}

fn get_frame(
    reader: &mut Reader<'_>,
    limits: ProtocolLimits,
) -> Result<FrameStream, ProtocolError> {
    let scene = owned_string(reader.get_str()?, "scene")?;
    let frame_index = reader.get_u64()?;
    let width = reader.get_u32()?;
    let height = reader.get_u32()?;
    let stride = reader.get_u32()?;
    let encoding = FrameEncoding::from_code(reader.get_u8()?)?;
    let payload = match reader.get_u8()? {
        0 => {
            let digest = reader.get_digest()?;
            let bytes = bounded_bytes(reader, "frame", limits.max_frame_bytes)?;
            FramePayload::Pipe { bytes, digest }
        }
        1 => FramePayload::SharedMemory {
            token: reader.get_digest()?,
            len: reader.get_u64()?,
            digest: reader.get_digest()?,
        },
        _ => return Err(ProtocolError::Malformed("frame transport")),
    };
    let mut render_backends = Vec::new();
    if reader.version().1 >= 1 {
        let count = count(
            reader.get_u32()?,
            "frame render backend",
            MAX_FRAME_RENDER_BACKENDS,
        )?;
        require_collection_payload(reader, "frame render backend", count, 41)?;
        render_backends = vec_with_capacity(count, "frame render backend")?;
        for _ in 0..count {
            let role = RenderBackendRole::from_wire_code(reader.get_u8()?)?;
            let digest = reader.get_digest()?;
            let identity = reader.get_bytes()?;
            render_backends.push(RenderBackendRecord::from_parts(role, identity, digest)?);
        }
    }
    Ok(FrameStream {
        scene,
        frame_index,
        width,
        height,
        stride,
        encoding,
        payload,
        render_backends,
    })
}

fn put_crash(writer: &mut Writer, report: &CrashReport) {
    match &report.scene {
        Some(scene) => {
            writer.put_bool(true);
            writer.put_str(scene);
        }
        None => {
            writer.put_bool(false);
        }
    }
    writer.put_str(&report.message);
    writer.put_bytes(&report.journal_tail);
    put_optional_digest(writer, &report.state_hash);
}

fn get_crash(
    reader: &mut Reader<'_>,
    limits: ProtocolLimits,
) -> Result<CrashReport, ProtocolError> {
    let scene = if reader.get_bool()? {
        Some(owned_string(reader.get_str()?, "scene")?)
    } else {
        None
    };
    Ok(CrashReport {
        scene,
        message: bounded_string(reader, "crash message", limits.max_crash_message_bytes)?,
        journal_tail: bounded_bytes(reader, "crash journal tail", limits.max_crash_tail_bytes)?,
        state_hash: get_optional_digest(reader)?,
    })
}

fn put_optional_digest(writer: &mut Writer, digest: &Option<Digest>) {
    match digest {
        Some(digest) => {
            writer.put_bool(true);
            writer.put_digest(digest);
        }
        None => {
            writer.put_bool(false);
        }
    }
}

fn get_optional_digest(reader: &mut Reader<'_>) -> Result<Option<Digest>, ProtocolError> {
    if reader.get_bool()? {
        Ok(Some(reader.get_digest()?))
    } else {
        Ok(None)
    }
}

fn bounded_bytes(
    reader: &mut Reader<'_>,
    field: &'static str,
    limit: usize,
) -> Result<Vec<u8>, ProtocolError> {
    let bytes = reader.get_bytes()?;
    limit_payload(field, bytes.len(), limit)?;
    owned_bytes(bytes, field)
}

fn bounded_string(
    reader: &mut Reader<'_>,
    field: &'static str,
    limit: usize,
) -> Result<String, ProtocolError> {
    let value = reader.get_str()?;
    limit_payload(field, value.len(), limit)?;
    owned_string(value, field)
}

fn vec_with_capacity<T>(additional: usize, field: &'static str) -> Result<Vec<T>, ProtocolError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(additional)
        .map_err(|source| ProtocolError::StorageUnavailable {
            field,
            additional,
            source,
        })?;
    Ok(values)
}

fn string_with_capacity(additional: usize, field: &'static str) -> Result<String, ProtocolError> {
    let mut value = String::new();
    value
        .try_reserve_exact(additional)
        .map_err(|source| ProtocolError::StorageUnavailable {
            field,
            additional,
            source,
        })?;
    Ok(value)
}

fn owned_bytes(bytes: &[u8], field: &'static str) -> Result<Vec<u8>, ProtocolError> {
    let mut owned = vec_with_capacity(bytes.len(), field)?;
    owned.extend_from_slice(bytes);
    Ok(owned)
}

fn owned_string(value: &str, field: &'static str) -> Result<String, ProtocolError> {
    let mut owned = string_with_capacity(value.len(), field)?;
    owned.push_str(value);
    Ok(owned)
}

fn count(raw: u32, field: &'static str, limit: usize) -> Result<usize, ProtocolError> {
    let needed =
        usize::try_from(raw).map_err(|_| ProtocolError::Malformed("wire count overflows usize"))?;
    limit_count(field, needed, limit)?;
    Ok(needed)
}

fn require_collection_payload(
    reader: &Reader<'_>,
    field: &'static str,
    count: usize,
    minimum_item_bytes: u64,
) -> Result<(), ProtocolError> {
    let count_u64 = u64::try_from(count)
        .map_err(|_| ProtocolError::Malformed("collection count overflows u64"))?;
    let minimum_bytes =
        count_u64
            .checked_mul(minimum_item_bytes)
            .ok_or(ProtocolError::Malformed(
                "collection minimum byte count overflow",
            ))?;
    let remaining_bytes = reader.remaining();
    let remaining_u64 = u64::try_from(remaining_bytes).unwrap_or(u64::MAX);
    if minimum_bytes > remaining_u64 {
        Err(ProtocolError::CollectionPayloadTooShort {
            field,
            count,
            minimum_bytes,
            remaining_bytes,
        })
    } else {
        Ok(())
    }
}

fn wire_count(field: &'static str, needed: usize) -> Result<u32, ProtocolError> {
    u32::try_from(needed).map_err(|_| ProtocolError::CountLimit {
        field,
        limit: usize::try_from(u32::MAX).unwrap_or(usize::MAX),
        needed,
    })
}

fn limit_count(field: &'static str, needed: usize, limit: usize) -> Result<(), ProtocolError> {
    if needed > limit {
        Err(ProtocolError::CountLimit {
            field,
            limit,
            needed,
        })
    } else {
        Ok(())
    }
}

fn limit_payload(field: &'static str, needed: usize, limit: usize) -> Result<(), ProtocolError> {
    if needed > limit {
        Err(ProtocolError::PayloadLimit {
            field,
            limit,
            needed,
        })
    } else {
        Ok(())
    }
}

fn require_scene(scene: &str) -> Result<(), ProtocolError> {
    if scene.is_empty() {
        Err(ProtocolError::Malformed("empty scene name"))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod studio_command_tests {
    use super::*;

    #[test]
    fn committed_seek_command_round_trips_and_binds_scene_frame_and_label() {
        let command = studio_seek_command("Demo", 17).expect("valid seek command");
        assert_eq!(studio_seek_frame("Demo", &command), Ok(17));

        let mut wrong_label = command.clone();
        wrong_label.label = "studio seek frame 18".to_owned();
        assert!(studio_seek_frame("Demo", &wrong_label).is_err());
        assert!(studio_seek_frame("Other", &command).is_err());
        assert!(studio_seek_command("Demo", -1).is_err());
    }

    #[test]
    fn studio_input_command_round_trips_and_binds_all_fields() {
        let input = StudioInput {
            frame: 2,
            revision: 7,
            target: Some(3),
            event: EventPayload::MousePress {
                point: [-1.5, 0.0, 0.0],
                button: MouseButton::Left,
                modifiers: Modifiers::PRIMARY,
            },
        };
        let command = studio_input_command("interactive.v1", &input).expect("valid input");
        assert_eq!(command.kind, CommandKind::Input);
        assert!(command.label.starts_with("studio input "));
        assert!(command.label.len() <= MAX_STUDIO_INPUT_LABEL_BYTES);
        assert_eq!(
            studio_input_payload("interactive.v1", &command).expect("decode"),
            input
        );

        let keyed = StudioInput {
            frame: 0,
            revision: 0,
            target: None,
            event: EventPayload::KeyRelease {
                key: Key::ArrowRight,
                modifiers: Modifiers::NONE,
            },
        };
        let command = studio_input_command("interactive.v1", &keyed).expect("valid input");
        assert_eq!(
            studio_input_payload("interactive.v1", &command).expect("decode"),
            keyed
        );
    }

    #[test]
    fn studio_input_command_rejects_tampering_and_malformed_labels() {
        let input = StudioInput {
            frame: 1,
            revision: 0,
            target: None,
            event: EventPayload::MouseMotion {
                point: [0.0, 0.0, 0.0],
                delta: [0.0, 0.0, 0.0],
                modifiers: Modifiers::NONE,
            },
        };
        let command = studio_input_command("interactive.v1", &input).expect("valid input");

        assert!(studio_input_payload("other.v1", &command).is_err());

        let mut wrong_kind = command.clone();
        wrong_kind.kind = CommandKind::Wait;
        assert!(studio_input_payload("interactive.v1", &wrong_kind).is_err());
        wrong_kind.kind = CommandKind::Custom;
        assert!(studio_input_payload("interactive.v1", &wrong_kind).is_err());

        let mut wrong_identity = command.clone();
        wrong_identity.identity = sha256(b"tampered");
        assert!(studio_input_payload("interactive.v1", &wrong_identity).is_err());

        let mut wrong_label = command.clone();
        wrong_label.label = "studio input 00".to_owned();
        assert!(studio_input_payload("interactive.v1", &wrong_label).is_err());

        let mut odd_hex = command.clone();
        odd_hex.label.push('0');
        assert!(studio_input_payload("interactive.v1", &odd_hex).is_err());

        let mut upper_hex = command.clone();
        upper_hex.label = upper_hex.label.to_ascii_uppercase();
        assert!(studio_input_payload("interactive.v1", &upper_hex).is_err());

        let mut over_budget = command.clone();
        over_budget.label = format!("studio input {}", "00".repeat(1025));
        assert!(studio_input_payload("interactive.v1", &over_budget).is_err());

        let mut truncated = command.clone();
        truncated.label.truncate(truncated.label.len() - 2);
        assert!(studio_input_payload("interactive.v1", &truncated).is_err());

        assert!(
            studio_input_command(
                "interactive.v1",
                &StudioInput {
                    frame: -1,
                    ..input.clone()
                },
            )
            .is_err()
        );
        assert!(
            studio_input_command(
                "interactive.v1",
                &StudioInput {
                    event: EventPayload::MouseMotion {
                        point: [f64::NAN, 0.0, 0.0],
                        delta: [0.0, 0.0, 0.0],
                        modifiers: Modifiers::NONE,
                    },
                    ..input
                },
            )
            .is_err()
        );
        assert!(studio_input_command("", &input).is_err());
    }
}

#[cfg(test)]
mod decoded_storage_tests {
    #![allow(clippy::expect_used)]

    use std::error::Error as _;

    use super::{ProtocolError, string_with_capacity, vec_with_capacity};

    fn assert_storage_unavailable(
        error: ProtocolError,
        expected_field: &'static str,
        expected_additional: usize,
    ) {
        assert!(error.source().is_some());
        let cloned = error.clone();
        assert_eq!(cloned, error);
        assert!(matches!(error, ProtocolError::StorageUnavailable { .. }));
        if let ProtocolError::StorageUnavailable {
            field, additional, ..
        } = error
        {
            assert_eq!(field, expected_field);
            assert_eq!(additional, expected_additional);
        }
    }

    #[test]
    fn impossible_decoded_vector_capacity_is_typed() {
        let error = vec_with_capacity::<u8>(usize::MAX, "decoded byte field")
            .expect_err("an impossible vector capacity must be refused");
        assert_storage_unavailable(error, "decoded byte field", usize::MAX);
    }

    #[test]
    fn impossible_decoded_string_capacity_is_typed() {
        let error = string_with_capacity(usize::MAX, "decoded string field")
            .expect_err("an impossible string capacity must be refused");
        assert_storage_unavailable(error, "decoded string field", usize::MAX);
    }
}
