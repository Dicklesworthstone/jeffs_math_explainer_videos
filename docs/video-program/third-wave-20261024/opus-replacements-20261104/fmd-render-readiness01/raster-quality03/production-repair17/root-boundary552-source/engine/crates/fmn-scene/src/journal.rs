//! The replay journal + effect model (§13.4, §18, R16, fm-y7u):
//! record everything, replay cheaply, invalidate conservatively.
//!
//! The journal is the one record with three consumers:
//!
//! 1. **The supervisor's edit-replay** (W9SUPER, fm-39s): restore the
//!    nearest checkpoint at or before the last valid entry and replay
//!    from there. Entries whose recorded identities and asset hashes
//!    still match are trusted; a changed callback hash invalidates
//!    from exactly that point ([`plan_replay`]).
//! 2. **The purity classifier's evidence** (§9.5, R20): the effect
//!    model *is* the classifier's vocabulary — [`EffectClass`] embeds
//!    the recorded [`fmn_anim::purity`] classification per segment, so
//!    frame-parallel eligibility is journaled, auditable state.
//! 3. **The pipeline's synchronization vocabulary** (§17.4): an entry
//!    classed [`EffectClass::PixelObserving`] is a pipeline barrier
//!    that drains in-flight frames. Ordinary manim scene code never
//!    produces one — asserted on the command corpus in tests.
//!
//! **The soundness doctrine (R16).** Replay-cache unsoundness is the
//! failure mode, and the design answer is conservative invalidation
//! everywhere: a [`CommandKind::Custom`] command is coerced to
//! [`EffectClass::Opaque`] no matter what its recorder claimed; any
//! entry that touched a subprocess is a replay barrier; a divergent
//! state hash mid-replay ([`ReplayAudit`]) falls back to full
//! re-execution and records why. When in doubt, a barrier.
//!
//! Also here: the one-command **repro bundle** (§18) — journal + input
//! closure, content-addressed, so every bug report is a deterministic
//! replay — and the journal's content hash for provenance sidecars. Since
//! schema minor 1.1, the same container carries the exact typed input stream
//! (sequence + rational-clock timestamp + payload). Minor 1.2 adds the
//! renderer/backend identities actually used by the session, separate from
//! replay-command identity. Minor 1.3 distinguishes typed scene input from
//! opaque custom callbacks. Older minors remain readable with absent newer
//! streams represented as empty; they cannot claim the new input vocabulary.

use fmn_anim::RationalTime;
use fmn_anim::purity::{ImpureEffect, Purity};
use fmn_hash::serial::{Error as SerialError, Limits, Reader, Schema, UnknownPolicy, Writer};
use fmn_hash::sha256::{Digest, sha256};
use std::collections::TryReserveError;

use crate::events::{EventError, EventPayload, EventType, InputEvent, Key, Modifiers, MouseButton};

/// The journal's versioned container schema (FMNA/3).
pub const JOURNAL_SCHEMA: Schema = Schema::new(*b"FMNA", 3, 1, 3);
/// The repro bundle's versioned container schema (FMNA/4).
pub const BUNDLE_SCHEMA: Schema = Schema::new(*b"FMNA", 4, 1, 0);

const LENGTH_PREFIX_BYTES: u64 = 8;
const DIGEST_BYTES: u64 = 32;
const MIN_COMMAND_BYTES: u64 = 1 + DIGEST_BYTES + LENGTH_PREFIX_BYTES;
const MIN_ENTRY_BYTES: u64 = MIN_COMMAND_BYTES + 1 + 4 + 4 + 1 + DIGEST_BYTES;
const MIN_ASSET_READ_BYTES: u64 = LENGTH_PREFIX_BYTES + DIGEST_BYTES;
const MIN_SUBPROCESS_BYTES: u64 = LENGTH_PREFIX_BYTES + DIGEST_BYTES + LENGTH_PREFIX_BYTES;
// A key event is the smallest input event: sequence, time, fps, event type,
// key tag, key value, and modifier bits.
const MIN_INPUT_EVENT_BYTES: u64 = 8 + 8 + 4 + 1 + 1 + 4 + 1;
const MIN_RENDER_BACKEND_BYTES: u64 = 1 + DIGEST_BYTES + LENGTH_PREFIX_BYTES;

/// Maximum canonical identity document retained for one renderer/backend.
///
/// Engine journals are small structured documents. A fixed ceiling prevents a
/// worker from turning provenance into an unbounded session-journal payload.
pub const MAX_RENDER_BACKEND_IDENTITY_BYTES: usize = 64 * 1024;

/// A journal failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JournalError {
    /// The canonical container refused the bytes.
    Serial(SerialError),
    /// A recorded input event violated its typed contract.
    Event(EventError),
    /// The payload decoded but violates a journal invariant.
    Malformed(&'static str),
    /// Journal-owned collection storage could not be reserved.
    StorageUnavailable {
        /// Collection whose capacity growth was refused.
        collection: &'static str,
        /// Additional elements requested by the append operation.
        additional: usize,
        /// Allocator or capacity-overflow refusal.
        source: TryReserveError,
    },
    /// A declared collection cannot fit in the bytes that remain.
    CollectionPayloadTooShort {
        /// Collection field.
        field: &'static str,
        /// Declared item count.
        count: usize,
        /// Minimum bytes required for the declared items.
        minimum_bytes: u64,
        /// Bytes left after the count field.
        remaining_bytes: usize,
    },
}

impl std::fmt::Display for JournalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serial(e) => write!(f, "journal container: {e}"),
            Self::Event(e) => write!(f, "journal event: {e}"),
            Self::Malformed(what) => write!(f, "malformed journal: {what}"),
            Self::StorageUnavailable {
                collection,
                additional,
                source,
            } => write!(
                f,
                "journal could not reserve {additional} additional {collection}: {source}"
            ),
            Self::CollectionPayloadTooShort {
                field,
                count,
                minimum_bytes,
                remaining_bytes,
            } => write!(
                f,
                "journal {field} count {count} requires at least {minimum_bytes} encoded bytes, but only {remaining_bytes} remain"
            ),
        }
    }
}

impl std::error::Error for JournalError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Serial(source) => Some(source),
            Self::Event(source) => Some(source),
            Self::StorageUnavailable { source, .. } => Some(source),
            Self::Malformed(_) | Self::CollectionPayloadTooShort { .. } => None,
        }
    }
}

impl From<SerialError> for JournalError {
    fn from(e: SerialError) -> Self {
        Self::Serial(e)
    }
}

impl From<EventError> for JournalError {
    fn from(e: EventError) -> Self {
        Self::Event(e)
    }
}

fn try_vec_with_capacity<T>(
    additional: usize,
    collection: &'static str,
) -> Result<Vec<T>, JournalError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(additional)
        .map_err(|source| JournalError::StorageUnavailable {
            collection,
            additional,
            source,
        })?;
    Ok(values)
}

fn try_string_with_capacity(
    additional: usize,
    collection: &'static str,
) -> Result<String, JournalError> {
    let mut value = String::new();
    value
        .try_reserve_exact(additional)
        .map_err(|source| JournalError::StorageUnavailable {
            collection,
            additional,
            source,
        })?;
    Ok(value)
}

fn try_clone_string(value: &str, collection: &'static str) -> Result<String, JournalError> {
    let mut cloned = try_string_with_capacity(value.len(), collection)?;
    cloned.push_str(value);
    Ok(cloned)
}

fn try_clone_bytes(value: &[u8], collection: &'static str) -> Result<Vec<u8>, JournalError> {
    let mut cloned = try_vec_with_capacity(value.len(), collection)?;
    cloned.extend_from_slice(value);
    Ok(cloned)
}

/// Serializable mirror of the classifier's impurity vocabulary
/// ([`ImpureEffect`]) — the journal's on-disk form of R20 evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImpureEffectTag {
    /// A dt-updater somewhere in the frame's families.
    DtUpdater,
    /// A non-dt (scene) updater.
    SceneUpdater,
    /// An animation without a declared pure signature.
    UnclassifiedAnimation,
    /// A `wait_until` stop condition.
    StopCondition,
}

impl From<ImpureEffect> for ImpureEffectTag {
    fn from(e: ImpureEffect) -> Self {
        match e {
            ImpureEffect::DtUpdater => Self::DtUpdater,
            ImpureEffect::SceneUpdater => Self::SceneUpdater,
            ImpureEffect::UnclassifiedAnimation => Self::UnclassifiedAnimation,
            ImpureEffect::StopCondition => Self::StopCondition,
        }
    }
}

impl ImpureEffectTag {
    const fn code(self) -> u8 {
        match self {
            Self::DtUpdater => 0,
            Self::SceneUpdater => 1,
            Self::UnclassifiedAnimation => 2,
            Self::StopCondition => 3,
        }
    }

    fn from_code(code: u8) -> Result<Self, JournalError> {
        Ok(match code {
            0 => Self::DtUpdater,
            1 => Self::SceneUpdater,
            2 => Self::UnclassifiedAnimation,
            3 => Self::StopCondition,
            _ => return Err(JournalError::Malformed("impure effect tag")),
        })
    }
}

/// The journal-level effect model (§13.4) — what one command did to
/// the world, as far as replay and the pipeline are concerned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectClass {
    /// Pure per §9.5: frame-parallel eligible, freely replayable.
    Pure,
    /// Stateful with the recorded reasons: serial front-end, still
    /// replayable (its outcome is a function of journaled state).
    Stateful(Vec<ImpureEffectTag>),
    /// The operation observes rendered pixels: a §17.4 pipeline
    /// barrier (drains in-flight frames) and a replay barrier.
    PixelObserving,
    /// Unrecognized: the conservative default. A replay barrier.
    Opaque,
}

impl EffectClass {
    /// Lift a segment classification into the journal vocabulary.
    #[must_use]
    pub fn from_purity(purity: &Purity) -> Self {
        match purity {
            Purity::Pure => Self::Pure,
            Purity::Stateful(effects) => {
                Self::Stateful(effects.iter().map(|&e| e.into()).collect())
            }
        }
    }

    /// Whether the §17.4 pipeline must drain in-flight frames before
    /// this effect runs.
    #[must_use]
    pub const fn is_pipeline_barrier(&self) -> bool {
        matches!(self, Self::PixelObserving)
    }

    /// Whether replay must stop *before* this entry and re-execute it
    /// (conservative: anything that cannot be proven equivalent from
    /// the record).
    #[must_use]
    pub const fn is_replay_barrier(&self) -> bool {
        matches!(self, Self::PixelObserving | Self::Opaque)
    }
}

/// The operation classes the journal recognizes. Anything else is
/// [`CommandKind::Custom`] — and Custom is opaque by decree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandKind {
    /// A `play()` segment.
    Play,
    /// A `wait()`/`wait_until()` segment.
    Wait,
    /// Scene membership addition.
    Add,
    /// Scene membership removal.
    Remove,
    /// A camera state change.
    CameraChange,
    /// An `add_sound` operation.
    Sound,
    /// Anything the vocabulary does not recognize.
    Custom,
    /// Typed, recorded scene input. Its effect and input closure must still be
    /// classified and verified; this is not a purity claim or arbitrary code.
    Input,
}

impl CommandKind {
    const fn code(self) -> u8 {
        match self {
            Self::Play => 0,
            Self::Wait => 1,
            Self::Add => 2,
            Self::Remove => 3,
            Self::CameraChange => 4,
            Self::Sound => 5,
            Self::Custom => 6,
            Self::Input => 7,
        }
    }

    fn from_code(code: u8) -> Result<Self, JournalError> {
        Ok(match code {
            0 => Self::Play,
            1 => Self::Wait,
            2 => Self::Add,
            3 => Self::Remove,
            4 => Self::CameraChange,
            5 => Self::Sound,
            6 => Self::Custom,
            7 => Self::Input,
            _ => return Err(JournalError::Malformed("command kind")),
        })
    }
}

/// One command's identity: its kind, a canonical digest of everything
/// that determines its behavior (parameters, callback version hashes),
/// and a human label for the inspector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandRecord {
    /// The operation class.
    pub kind: CommandKind,
    /// The identity digest — two commands with equal digests are
    /// behaviorally identical by construction of the digest.
    pub identity: Digest,
    /// Human-facing label (`"play FadeIn(circle)"`).
    pub label: String,
}

impl CommandRecord {
    /// Whether `other` re-executes to the same behavior.
    #[must_use]
    pub fn matches(&self, other: &Self) -> bool {
        // ubs:ignore - command identities are public content hashes, not secrets.
        self.kind == other.kind && self.identity == other.identity
    }
}

/// A content-addressed file/font/asset read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetRead {
    /// The path as the scene addressed it.
    pub path: String,
    /// SHA-256 of the bytes that were read.
    pub digest: Digest,
}

/// A journaled subprocess (ffmpeg) invocation — provenance, and a
/// replay barrier (side effects on disk cannot be proven equivalent
/// from the record).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubprocessRecord {
    /// SHA-256 of the tool's executable bytes, hex.
    pub tool_sha256_hex: String,
    /// Digest of the canonical argv.
    pub argv_digest: Digest,
    /// The published destination.
    pub destination: String,
}

/// How a renderer/backend participated in a presented Studio frame.
///
/// Native presentation and CPU-visible streaming are intentionally distinct:
/// a Metal Studio frame may use both, because the native surface has no pixel
/// readback while browser and terminal clients still require PNG bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderBackendRole {
    /// Produced the CPU-visible frame stream delivered to browser/TUI clients.
    FrameStream,
    /// Presented a native surface without CPU pixel readback.
    NativePresentation,
}

impl RenderBackendRole {
    /// Stable wire code used by the journal and Studio IPC.
    #[must_use]
    pub const fn wire_code(self) -> u8 {
        match self {
            Self::FrameStream => 0,
            Self::NativePresentation => 1,
        }
    }

    /// Recover a stable wire code.
    pub fn from_wire_code(code: u8) -> Result<Self, JournalError> {
        match code {
            0 => Ok(Self::FrameStream),
            1 => Ok(Self::NativePresentation),
            _ => Err(JournalError::Malformed("render backend role")),
        }
    }
}

/// One canonical renderer/backend identity actually used by the session.
///
/// The digest is redundant by design: it is the stable lookup key while the
/// canonical bytes remain the inspectable provenance document. Construction
/// and decoding always verify that the two agree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderBackendRecord {
    role: RenderBackendRole,
    identity: Vec<u8>,
    digest: Digest,
}

impl RenderBackendRecord {
    /// Construct from an owned canonical identity document.
    pub fn new(role: RenderBackendRole, identity: Vec<u8>) -> Result<Self, JournalError> {
        Self::validate_identity(&identity)?;
        let digest = sha256(&identity);
        Ok(Self {
            role,
            identity,
            digest,
        })
    }

    /// Decode a record while authenticating its claimed digest before owning
    /// the identity bytes.
    pub fn from_parts(
        role: RenderBackendRole,
        identity: &[u8],
        digest: Digest,
    ) -> Result<Self, JournalError> {
        Self::validate_identity(identity)?;
        // ubs:ignore - renderer identities are public content hashes, not secrets.
        if sha256(identity) != digest {
            return Err(JournalError::Malformed("render backend identity digest"));
        }
        Ok(Self {
            role,
            identity: try_clone_bytes(identity, "render backend identity bytes")?,
            digest,
        })
    }

    fn validate_identity(identity: &[u8]) -> Result<(), JournalError> {
        if identity.is_empty() {
            return Err(JournalError::Malformed("empty render backend identity"));
        }
        if identity.len() > MAX_RENDER_BACKEND_IDENTITY_BYTES {
            return Err(JournalError::Malformed(
                "render backend identity exceeds limit",
            ));
        }
        Ok(())
    }

    /// Participation role for this identity.
    #[must_use]
    pub const fn role(&self) -> RenderBackendRole {
        self.role
    }

    /// Canonical backend identity bytes.
    #[must_use]
    pub fn identity(&self) -> &[u8] {
        &self.identity
    }

    /// SHA-256 of [`Self::identity`].
    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }

    /// Recheck the size and content-integrity contract without allocating.
    pub fn validate(&self) -> Result<(), JournalError> {
        Self::validate_identity(&self.identity)?;
        // ubs:ignore - renderer identities are public content hashes, not secrets.
        if sha256(&self.identity) != self.digest {
            return Err(JournalError::Malformed("render backend identity digest"));
        }
        Ok(())
    }

    /// Fallibly copy the owned identity document.
    pub fn try_clone(&self) -> Result<Self, JournalError> {
        Self::from_parts(self.role, &self.identity, self.digest)
    }
}

/// One journal entry: a command, its effect, everything it read, and
/// the state it produced.
#[derive(Debug, Clone)]
pub struct Entry {
    /// The command's identity.
    pub command: CommandRecord,
    /// The recorded effect class.
    pub effect: EffectClass,
    /// Content-addressed reads this command performed.
    pub reads: Vec<AssetRead>,
    /// Subprocess invocations this command performed.
    pub subprocesses: Vec<SubprocessRecord>,
    /// A full [`fmn_mobject::SceneState`] checkpoint (its canonical
    /// bytes) taken after this command, when the checkpoint policy
    /// took one. Carries the RNG state at the barrier by construction.
    pub checkpoint: Option<Vec<u8>>,
    /// SHA-256 of the post-command `SceneState` bytes — the divergence
    /// detector [`ReplayAudit`] compares against.
    pub state_hash: Digest,
}

impl Entry {
    /// Whether replay may not skip past this entry.
    #[must_use]
    pub fn is_replay_barrier(&self) -> bool {
        self.effect.is_replay_barrier() || !self.subprocesses.is_empty()
    }

    /// Fallibly copy every owned part of this entry.
    ///
    /// # Errors
    /// [`JournalError::StorageUnavailable`] names the exact field whose
    /// storage could not be reserved. The source entry is never mutated.
    pub fn try_clone(&self) -> Result<Self, JournalError> {
        let command = CommandRecord {
            kind: self.command.kind,
            identity: self.command.identity,
            label: try_clone_string(&self.command.label, "entry command label bytes")?,
        };
        let effect = match &self.effect {
            EffectClass::Pure => EffectClass::Pure,
            EffectClass::Stateful(source) => {
                let mut tags = try_vec_with_capacity(source.len(), "entry stateful effect tags")?;
                tags.extend_from_slice(source);
                EffectClass::Stateful(tags)
            }
            EffectClass::PixelObserving => EffectClass::PixelObserving,
            EffectClass::Opaque => EffectClass::Opaque,
        };
        let mut reads = try_vec_with_capacity(self.reads.len(), "entry asset reads")?;
        for read in &self.reads {
            reads.push(AssetRead {
                path: try_clone_string(&read.path, "entry asset path bytes")?,
                digest: read.digest,
            });
        }
        let mut subprocesses =
            try_vec_with_capacity(self.subprocesses.len(), "entry subprocess records")?;
        for subprocess in &self.subprocesses {
            subprocesses.push(SubprocessRecord {
                tool_sha256_hex: try_clone_string(
                    &subprocess.tool_sha256_hex,
                    "entry subprocess tool identity bytes",
                )?,
                argv_digest: subprocess.argv_digest,
                destination: try_clone_string(
                    &subprocess.destination,
                    "entry subprocess destination bytes",
                )?,
            });
        }
        let checkpoint = self
            .checkpoint
            .as_deref()
            .map(|bytes| try_clone_bytes(bytes, "entry checkpoint bytes"))
            .transpose()?;
        Ok(Self {
            command,
            effect,
            reads,
            subprocesses,
            checkpoint,
            state_hash: self.state_hash,
        })
    }
}

/// The append-only journal.
#[derive(Debug, Clone, Default)]
pub struct Journal {
    entries: Vec<Entry>,
    events: Vec<InputEvent>,
    render_backends: Vec<RenderBackendRecord>,
}

impl Journal {
    /// An empty journal.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append an entry, applying the conservative coercions: a
    /// [`CommandKind::Custom`] command's effect becomes
    /// [`EffectClass::Opaque`] regardless of what the recorder claimed
    /// (R16 — unrecognized operations do not get to describe
    /// themselves as replayable).
    ///
    /// # Errors
    /// [`JournalError::StorageUnavailable`] if entry storage cannot grow.
    pub fn record(&mut self, entry: Entry) -> Result<(), JournalError> {
        self.reserve_entries(1)?;
        self.entries.push(Self::canonical_entry(entry));
        Ok(())
    }

    fn canonical_entry(mut entry: Entry) -> Entry {
        if entry.command.kind == CommandKind::Custom {
            entry.effect = EffectClass::Opaque;
        }
        entry
    }

    fn reserve_entries(&mut self, additional: usize) -> Result<(), JournalError> {
        self.entries
            .try_reserve(additional)
            .map_err(|source| JournalError::StorageUnavailable {
                collection: "entries",
                additional,
                source,
            })
    }

    fn reserve_events(&mut self, additional: usize) -> Result<(), JournalError> {
        self.events
            .try_reserve(additional)
            .map_err(|source| JournalError::StorageUnavailable {
                collection: "events",
                additional,
                source,
            })
    }

    fn reserve_render_backends(&mut self, additional: usize) -> Result<(), JournalError> {
        self.render_backends
            .try_reserve(additional)
            .map_err(|source| JournalError::StorageUnavailable {
                collection: "render backends",
                additional,
                source,
            })
    }

    fn event_batch_scratch(additional: usize) -> Result<Vec<InputEvent>, JournalError> {
        try_vec_with_capacity(additional, "event batch scratch")
    }

    /// The recorded entries, in order.
    #[must_use]
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Append one dispatched input event.
    ///
    /// Events must remain in exact dispatch order: timestamps never move
    /// backward and sequence ids always increase.
    pub fn record_event(&mut self, event: InputEvent) -> Result<(), JournalError> {
        let event = InputEvent::new(event.sequence, event.timestamp, event.payload)?;
        if self.events.last().is_some_and(|previous| {
            event.timestamp < previous.timestamp || event.sequence <= previous.sequence
        }) {
            return Err(EventError::ReplayOutOfOrder.into());
        }
        self.reserve_events(1)?;
        self.events.push(event);
        Ok(())
    }

    /// Append a dispatched event stream without partial mutation on refusal.
    pub fn record_events(&mut self, events: &[InputEvent]) -> Result<(), JournalError> {
        let mut canonical = Self::event_batch_scratch(events.len())?;
        let mut previous = self
            .events
            .last()
            .map(|event| (event.timestamp, event.sequence));
        for event in events {
            let event = InputEvent::new(event.sequence, event.timestamp, event.payload.clone())?;
            if previous.is_some_and(|(timestamp, sequence)| {
                event.timestamp < timestamp || event.sequence <= sequence
            }) {
                return Err(EventError::ReplayOutOfOrder.into());
            }
            previous = Some((event.timestamp, event.sequence));
            canonical.push(event);
        }
        self.reserve_events(canonical.len())?;
        self.events.extend(canonical);
        Ok(())
    }

    /// Recorded input events in dispatch order.
    #[must_use]
    pub fn events(&self) -> &[InputEvent] {
        &self.events
    }

    /// Renderer/backend identities actually used during this session.
    ///
    /// Records are unique by `(role, digest)` and retain first-observation
    /// order, making the serialized journal deterministic without one entry per
    /// rendered frame.
    #[must_use]
    pub fn render_backends(&self) -> &[RenderBackendRecord] {
        &self.render_backends
    }

    /// Record one renderer/backend identity, deduplicating an existing match.
    pub fn record_render_backend(
        &mut self,
        record: &RenderBackendRecord,
    ) -> Result<(), JournalError> {
        self.record_render_backends(std::slice::from_ref(record))
    }

    /// Atomically record a batch of renderer/backend identities.
    ///
    /// A repeated `(role, digest)` with different canonical bytes is rejected as
    /// a collision. The journal is not mutated until every incoming record has
    /// validated and all required storage has been reserved.
    pub fn record_render_backends(
        &mut self,
        records: &[RenderBackendRecord],
    ) -> Result<(), JournalError> {
        let mut additions = try_vec_with_capacity(records.len(), "render backend batch scratch")?;
        for record in records {
            record.validate()?;
            let existing = self
                .render_backends
                .iter()
                .chain(additions.iter())
                .find(|candidate| {
                    candidate.role() == record.role() && candidate.digest() == record.digest()
                });
            if let Some(existing) = existing {
                if existing.identity() != record.identity() {
                    return Err(JournalError::Malformed(
                        "render backend identity digest collision",
                    ));
                }
                continue;
            }
            additions.push(record.try_clone()?);
        }
        self.reserve_render_backends(additions.len())?;
        self.render_backends.extend(additions);
        Ok(())
    }

    /// Fallibly copy the complete durable session journal.
    pub fn try_clone(&self) -> Result<Self, JournalError> {
        let mut cloned = Self::new();
        cloned.reserve_entries(self.entries.len())?;
        for entry in &self.entries {
            cloned.entries.push(entry.try_clone()?);
        }
        cloned.record_events(&self.events)?;
        cloned.record_render_backends(&self.render_backends)?;
        Ok(cloned)
    }

    /// Serialize into the versioned canonical container.
    ///
    /// # Errors
    /// [`SerialError`] on size-limit overflow.
    pub fn to_bytes(&self) -> Result<Vec<u8>, SerialError> {
        let entry_count = wire_count(self.entries.len())?;
        let mut w = Writer::new(JOURNAL_SCHEMA);
        w.put_u32(entry_count);
        for entry in &self.entries {
            put_command(&mut w, &entry.command);
            put_effect(&mut w, &entry.effect)?;
            w.put_u32(wire_count(entry.reads.len())?);
            for read in &entry.reads {
                w.put_str(&read.path);
                w.put_digest(&read.digest);
            }
            w.put_u32(wire_count(entry.subprocesses.len())?);
            for sub in &entry.subprocesses {
                w.put_str(&sub.tool_sha256_hex);
                w.put_digest(&sub.argv_digest);
                w.put_str(&sub.destination);
            }
            match &entry.checkpoint {
                Some(bytes) => {
                    w.put_bool(true);
                    w.put_bytes(bytes);
                }
                None => {
                    w.put_bool(false);
                }
            }
            w.put_digest(&entry.state_hash);
        }
        w.put_u32(wire_count(self.events.len())?);
        for event in &self.events {
            put_input_event(&mut w, event);
        }
        w.put_u32(wire_count(self.render_backends.len())?);
        for backend in &self.render_backends {
            w.put_u8(backend.role().wire_code());
            w.put_digest(&backend.digest());
            w.put_bytes(backend.identity());
        }
        w.finish()
    }

    /// Decode a journal.
    ///
    /// # Errors
    /// [`JournalError`] on container or payload violations.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, JournalError> {
        let mut r = Reader::open(
            bytes,
            JOURNAL_SCHEMA,
            Limits::DEFAULT,
            UnknownPolicy::Strict,
        )?;
        let count = r.get_u32()? as usize;
        require_collection_payload(&r, "entry", count, MIN_ENTRY_BYTES)?;
        let mut entries = Vec::with_capacity(count.min(65_536));
        for _ in 0..count {
            let command = get_command(&mut r)?;
            if command.kind == CommandKind::Input && r.version().1 < 3 {
                return Err(JournalError::Malformed(
                    "input command requires journal minor 1.3",
                ));
            }
            let effect = get_effect(&mut r)?;
            if command.kind == CommandKind::Custom && effect != EffectClass::Opaque {
                return Err(JournalError::Malformed("custom command effect"));
            }
            let read_count = r.get_u32()? as usize;
            require_collection_payload(&r, "asset read", read_count, MIN_ASSET_READ_BYTES)?;
            let mut reads = Vec::with_capacity(read_count.min(4096));
            for _ in 0..read_count {
                reads.push(AssetRead {
                    path: r.get_str()?.to_string(),
                    digest: r.get_digest()?,
                });
            }
            let sub_count = r.get_u32()? as usize;
            require_collection_payload(&r, "subprocess", sub_count, MIN_SUBPROCESS_BYTES)?;
            let mut subprocesses = Vec::with_capacity(sub_count.min(4096));
            for _ in 0..sub_count {
                subprocesses.push(SubprocessRecord {
                    tool_sha256_hex: r.get_str()?.to_string(),
                    argv_digest: r.get_digest()?,
                    destination: r.get_str()?.to_string(),
                });
            }
            let checkpoint = if r.get_bool()? {
                Some(r.get_bytes()?.to_vec())
            } else {
                None
            };
            let state_hash = r.get_digest()?;
            entries.push(Entry {
                command,
                effect,
                reads,
                subprocesses,
                checkpoint,
                state_hash,
            });
        }
        let mut journal = Self {
            entries,
            events: Vec::new(),
            render_backends: Vec::new(),
        };
        if r.version().1 >= 1 {
            let event_count = r.get_u32()? as usize;
            require_collection_payload(&r, "input event", event_count, MIN_INPUT_EVENT_BYTES)?;
            for _ in 0..event_count {
                journal.record_event(get_input_event(&mut r)?)?;
            }
        }
        if r.version().1 >= 2 {
            let backend_count = r.get_u32()? as usize;
            require_collection_payload(
                &r,
                "render backend",
                backend_count,
                MIN_RENDER_BACKEND_BYTES,
            )?;
            for _ in 0..backend_count {
                let role = RenderBackendRole::from_wire_code(r.get_u8()?)?;
                let digest = r.get_digest()?;
                let identity = r.get_bytes()?;
                let backend = RenderBackendRecord::from_parts(role, identity, digest)?;
                journal.record_render_backend(&backend)?;
            }
        }
        r.finish()?;
        Ok(journal)
    }

    /// The journal's content address — what provenance sidecars carry.
    ///
    /// # Errors
    /// As [`Journal::to_bytes`].
    pub fn content_hash(&self) -> Result<Digest, SerialError> {
        Ok(sha256(&self.to_bytes()?))
    }
}

fn put_command(w: &mut Writer, command: &CommandRecord) {
    w.put_u8(command.kind.code());
    w.put_digest(&command.identity);
    w.put_str(&command.label);
}

fn get_command(r: &mut Reader<'_>) -> Result<CommandRecord, JournalError> {
    let kind = CommandKind::from_code(r.get_u8()?)?;
    let identity = r.get_digest()?;
    let label = r.get_str()?.to_string();
    Ok(CommandRecord {
        kind,
        identity,
        label,
    })
}

fn put_effect(w: &mut Writer, effect: &EffectClass) -> Result<(), SerialError> {
    match effect {
        EffectClass::Pure => {
            w.put_u8(0);
        }
        EffectClass::Stateful(tags) => {
            let count = wire_count(tags.len())?;
            w.put_u8(1);
            w.put_u32(count);
            for tag in tags {
                w.put_u8(tag.code());
            }
        }
        EffectClass::PixelObserving => {
            w.put_u8(2);
        }
        EffectClass::Opaque => {
            w.put_u8(3);
        }
    }
    Ok(())
}

fn get_effect(r: &mut Reader<'_>) -> Result<EffectClass, JournalError> {
    Ok(match r.get_u8()? {
        0 => EffectClass::Pure,
        1 => {
            let count = r.get_u32()? as usize;
            if count > 16 {
                return Err(JournalError::Malformed("impure effect count"));
            }
            require_collection_payload(r, "impure effect tag", count, 1)?;
            let mut tags = Vec::with_capacity(count);
            for _ in 0..count {
                tags.push(ImpureEffectTag::from_code(r.get_u8()?)?);
            }
            EffectClass::Stateful(tags)
        }
        2 => EffectClass::PixelObserving,
        3 => EffectClass::Opaque,
        _ => return Err(JournalError::Malformed("effect class")),
    })
}

fn require_collection_payload(
    reader: &Reader<'_>,
    field: &'static str,
    count: usize,
    minimum_item_bytes: u64,
) -> Result<(), JournalError> {
    let count_u64 = u64::try_from(count)
        .map_err(|_| JournalError::Malformed("collection count overflows u64"))?;
    let minimum_bytes =
        count_u64
            .checked_mul(minimum_item_bytes)
            .ok_or(JournalError::Malformed(
                "collection minimum byte count overflow",
            ))?;
    let remaining_bytes = reader.remaining();
    let remaining_u64 = u64::try_from(remaining_bytes).unwrap_or(u64::MAX);
    if minimum_bytes > remaining_u64 {
        Err(JournalError::CollectionPayloadTooShort {
            field,
            count,
            minimum_bytes,
            remaining_bytes,
        })
    } else {
        Ok(())
    }
}

fn wire_count(needed: usize) -> Result<u32, SerialError> {
    u32::try_from(needed).map_err(|_| SerialError::SizeLimit {
        limit: usize::try_from(u32::MAX).unwrap_or(usize::MAX),
        needed,
    })
}

fn put_input_event(w: &mut Writer, event: &InputEvent) {
    w.put_u64(event.sequence);
    w.put_i64(event.timestamp.frames());
    w.put_u32(event.timestamp.fps());
    w.put_u8(event.event_type().code());
    match &event.payload {
        EventPayload::MouseMotion {
            point,
            delta,
            modifiers,
        } => {
            put_vec3(w, *point);
            put_vec3(w, *delta);
            w.put_u8(modifiers.bits());
        }
        EventPayload::MousePress {
            point,
            button,
            modifiers,
        }
        | EventPayload::MouseRelease {
            point,
            button,
            modifiers,
        } => {
            put_vec3(w, *point);
            put_mouse_button(w, *button);
            w.put_u8(modifiers.bits());
        }
        EventPayload::MouseDrag {
            point,
            delta,
            button,
            modifiers,
        } => {
            put_vec3(w, *point);
            put_vec3(w, *delta);
            put_mouse_button(w, *button);
            w.put_u8(modifiers.bits());
        }
        EventPayload::MouseScroll {
            point,
            offset,
            modifiers,
        } => {
            put_vec3(w, *point);
            w.put_f64(offset[0]);
            w.put_f64(offset[1]);
            w.put_u8(modifiers.bits());
        }
        EventPayload::KeyPress { key, modifiers } | EventPayload::KeyRelease { key, modifiers } => {
            let (tag, value) = key.code();
            w.put_u8(tag);
            w.put_u32(value);
            w.put_u8(modifiers.bits());
        }
    }
}

fn get_input_event(r: &mut Reader<'_>) -> Result<InputEvent, JournalError> {
    let sequence = r.get_u64()?;
    let frames = r.get_i64()?;
    let fps = r.get_u32()?;
    let event_type = EventType::from_code(r.get_u8()?)?;
    let payload = match event_type {
        EventType::MouseMotion => EventPayload::MouseMotion {
            point: get_vec3(r)?,
            delta: get_vec3(r)?,
            modifiers: Modifiers::from_bits(r.get_u8()?)?,
        },
        EventType::MousePress => EventPayload::MousePress {
            point: get_vec3(r)?,
            button: get_mouse_button(r)?,
            modifiers: Modifiers::from_bits(r.get_u8()?)?,
        },
        EventType::MouseRelease => EventPayload::MouseRelease {
            point: get_vec3(r)?,
            button: get_mouse_button(r)?,
            modifiers: Modifiers::from_bits(r.get_u8()?)?,
        },
        EventType::MouseDrag => EventPayload::MouseDrag {
            point: get_vec3(r)?,
            delta: get_vec3(r)?,
            button: get_mouse_button(r)?,
            modifiers: Modifiers::from_bits(r.get_u8()?)?,
        },
        EventType::MouseScroll => EventPayload::MouseScroll {
            point: get_vec3(r)?,
            offset: [r.get_f64()?, r.get_f64()?],
            modifiers: Modifiers::from_bits(r.get_u8()?)?,
        },
        EventType::KeyPress | EventType::KeyRelease => {
            let key = Key::from_code(r.get_u8()?, r.get_u32()?)?;
            let modifiers = Modifiers::from_bits(r.get_u8()?)?;
            if event_type == EventType::KeyPress {
                EventPayload::KeyPress { key, modifiers }
            } else {
                EventPayload::KeyRelease { key, modifiers }
            }
        }
    };
    InputEvent::new(sequence, RationalTime::zero(fps) + frames, payload).map_err(Into::into)
}

fn put_vec3(w: &mut Writer, point: [f64; 3]) {
    for value in point {
        w.put_f64(value);
    }
}

fn get_vec3(r: &mut Reader<'_>) -> Result<[f64; 3], JournalError> {
    Ok([r.get_f64()?, r.get_f64()?, r.get_f64()?])
}

fn put_mouse_button(w: &mut Writer, button: MouseButton) {
    let (tag, value) = button.code();
    w.put_u8(tag);
    w.put_u16(value);
}

fn get_mouse_button(r: &mut Reader<'_>) -> Result<MouseButton, JournalError> {
    MouseButton::from_code(r.get_u8()?, r.get_u16()?).map_err(Into::into)
}

// ---- replay planning ------------------------------------------------

/// Why reuse stopped short of the full recorded journal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidationReason {
    /// The incoming command at `index` differs from the recorded one
    /// (a changed callback hash lands here).
    CommandMismatch {
        /// The first divergent entry.
        index: usize,
    },
    /// A recorded asset read no longer verifies.
    AssetChanged {
        /// The entry whose read failed.
        index: usize,
        /// The changed asset's path.
        path: String,
    },
    /// The entry is a replay barrier (opaque, pixel-observing, or
    /// subprocess-touching): it must re-execute.
    ReplayBarrier {
        /// The barrier entry.
        index: usize,
    },
    /// The incoming stream ended before the recorded journal did.
    StreamExhausted {
        /// The first recorded entry with no incoming counterpart.
        index: usize,
    },
}

/// The replay plan: how much of the recorded journal to trust, and
/// where execution resumes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayPlan {
    /// Entries `[0, reuse)` are verified reusable: their identities
    /// match, their asset reads still verify, and none is a barrier.
    pub reuse: usize,
    /// The latest entry index `< reuse` carrying a checkpoint —
    /// restore it, then re-execute commands after it (each verified
    /// equivalent by the record). `None`: cold start from the top.
    pub resume_checkpoint: Option<usize>,
    /// Why reuse stopped, when it stopped short of the whole journal.
    pub reason: Option<InvalidationReason>,
}

/// Plan a replay of `incoming` against the recorded journal.
///
/// The conservative walk: reuse grows only while the incoming command
/// matches the recorded identity, every recorded read still verifies
/// (`asset_ok`), and the entry is not a replay barrier. The first
/// failure stops the walk and is recorded as the reason — when in
/// doubt, a barrier (R16).
#[must_use]
pub fn plan_replay(
    journal: &Journal,
    incoming: &[CommandRecord],
    asset_ok: &dyn Fn(&AssetRead) -> bool,
) -> ReplayPlan {
    let mut reuse = 0usize;
    let mut reason = None;
    for (index, entry) in journal.entries().iter().enumerate() {
        let Some(command) = incoming.get(index) else {
            reason = Some(InvalidationReason::StreamExhausted { index });
            break;
        };
        if !entry.command.matches(command) {
            reason = Some(InvalidationReason::CommandMismatch { index });
            break;
        }
        if entry.is_replay_barrier() {
            reason = Some(InvalidationReason::ReplayBarrier { index });
            break;
        }
        if let Some(read) = entry.reads.iter().find(|read| !asset_ok(read)) {
            reason = Some(InvalidationReason::AssetChanged {
                index,
                path: read.path.clone(),
            });
            break;
        }
        reuse = index + 1;
    }
    let resume_checkpoint = journal.entries()[..reuse]
        .iter()
        .rposition(|entry| entry.checkpoint.is_some());
    ReplayPlan {
        reuse,
        resume_checkpoint,
        reason,
    }
}

/// The mid-replay divergence detector (R16's fallback clause): as the
/// supervisor re-executes verified entries, it feeds each produced
/// state hash here; the first mismatch flips the audit to diverged,
/// recording where — the caller then falls back to full re-execution
/// from its last good checkpoint, correctly and silently.
#[derive(Debug, Clone, Default)]
pub struct ReplayAudit {
    verified: usize,
    diverged_at: Option<usize>,
}

impl ReplayAudit {
    /// A fresh audit.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check one replayed entry's produced state against the record.
    /// Returns whether replay may continue.
    pub fn step(&mut self, journal: &Journal, index: usize, produced: &Digest) -> bool {
        if self.diverged_at.is_some() {
            return false;
        }
        match journal.entries().get(index) {
            // ubs:ignore - state hashes are public determinism evidence, not secrets.
            Some(entry) if entry.state_hash == *produced => {
                self.verified = index + 1;
                true
            }
            _ => {
                self.diverged_at = Some(index);
                false
            }
        }
    }

    /// Entries verified equivalent so far.
    #[must_use]
    pub const fn verified(&self) -> usize {
        self.verified
    }

    /// Where replay diverged, if it did — the recorded "why".
    #[must_use]
    pub const fn diverged_at(&self) -> Option<usize> {
        self.diverged_at
    }
}

// ---- the repro bundle (§18) ----------------------------------------

/// A bundle failed verification: the named divergent asset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleDivergence {
    /// The divergent path.
    pub path: String,
    /// The digest the bundle recorded.
    pub expected: Digest,
    /// The digest found on this machine (`None`: unreadable/absent).
    pub found: Option<Digest>,
}

impl std::fmt::Display for BundleDivergence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "repro bundle diverges at {:?}: expected {}, found {}",
            self.path,
            self.expected.to_hex(),
            self.found
                .as_ref()
                .map_or_else(|| "nothing readable".to_string(), Digest::to_hex),
        )
    }
}

/// The one-command repro bundle: the journal plus the content-hashed
/// input closure. A bug report carrying one is a deterministic replay.
#[derive(Debug, Clone)]
pub struct ReproBundle {
    /// The scene's human name.
    pub scene_label: String,
    /// The session seed.
    pub seed: u64,
    /// The exact rational frame rate.
    pub fps: (u32, u32),
    /// The input closure: every file the session read, content-hashed
    /// (scene sources, fonts, assets, config).
    pub closure: Vec<AssetRead>,
    /// The session journal.
    pub journal: Journal,
}

impl ReproBundle {
    /// Serialize into the versioned canonical container.
    ///
    /// # Errors
    /// [`SerialError`] on size-limit overflow.
    pub fn to_bytes(&self) -> Result<Vec<u8>, SerialError> {
        let closure_count = wire_count(self.closure.len())?;
        let mut w = Writer::new(BUNDLE_SCHEMA);
        w.put_str(&self.scene_label);
        w.put_u64(self.seed);
        w.put_u32(self.fps.0);
        w.put_u32(self.fps.1);
        w.put_u32(closure_count);
        for read in &self.closure {
            w.put_str(&read.path);
            w.put_digest(&read.digest);
        }
        w.put_bytes(&self.journal.to_bytes()?);
        w.finish()
    }

    /// Decode a bundle.
    ///
    /// # Errors
    /// [`JournalError`] on container or payload violations.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, JournalError> {
        let mut r = Reader::open(bytes, BUNDLE_SCHEMA, Limits::DEFAULT, UnknownPolicy::Strict)?;
        let scene_label = r.get_str()?.to_string();
        let seed = r.get_u64()?;
        let fps = (r.get_u32()?, r.get_u32()?);
        let count = r.get_u32()? as usize;
        require_collection_payload(&r, "repro closure", count, MIN_ASSET_READ_BYTES)?;
        let mut closure = Vec::with_capacity(count.min(65_536));
        for _ in 0..count {
            closure.push(AssetRead {
                path: r.get_str()?.to_string(),
                digest: r.get_digest()?,
            });
        }
        let journal = Journal::from_bytes(r.get_bytes()?)?;
        r.finish()?;
        Ok(Self {
            scene_label,
            seed,
            fps,
            closure,
            journal,
        })
    }

    /// The bundle's content address.
    ///
    /// # Errors
    /// As [`ReproBundle::to_bytes`].
    pub fn content_hash(&self) -> Result<Digest, SerialError> {
        Ok(sha256(&self.to_bytes()?))
    }

    /// Verify the closure on this machine: every recorded asset must
    /// read back to its recorded digest.
    ///
    /// # Errors
    /// The first [`BundleDivergence`], named.
    pub fn verify(&self, read: &dyn Fn(&str) -> Option<Vec<u8>>) -> Result<(), BundleDivergence> {
        for asset in &self.closure {
            let found = read(&asset.path).map(|bytes| sha256(&bytes));
            // ubs:ignore - asset digests are public content addresses, not secrets.
            if found.as_ref() != Some(&asset.digest) {
                return Err(BundleDivergence {
                    path: asset.path.clone(),
                    expected: asset.digest,
                    found,
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fmn_hash::sha256::sha256;

    use super::{
        AssetRead, CommandKind, CommandRecord, EffectClass, Entry, EventError, EventPayload,
        ImpureEffectTag, InputEvent, Journal, JournalError, Key, Modifiers, RationalTime,
        SerialError, SubprocessRecord, try_string_with_capacity, try_vec_with_capacity, wire_count,
    };

    fn key_event(sequence: u64, frame: i64) -> InputEvent {
        InputEvent::new(
            sequence,
            RationalTime::zero(30) + frame,
            EventPayload::KeyPress {
                key: Key::Character('A'),
                modifiers: Modifiers::NONE,
            },
        )
        .expect("valid journal event fixture")
    }

    #[test]
    fn wire_count_accepts_u32_max_and_refuses_one_over() {
        let max = usize::try_from(u32::MAX).unwrap_or(usize::MAX);
        assert_eq!(wire_count(max).unwrap(), u32::MAX);
        if let Some(one_over) = max.checked_add(1) {
            assert!(matches!(
                wire_count(one_over),
                Err(SerialError::SizeLimit { limit, needed })
                    if limit == max && needed == one_over
            ));
        }
    }

    #[test]
    fn decoded_custom_command_cannot_claim_replayable_effect() {
        let forged = Journal {
            entries: vec![Entry {
                command: CommandRecord {
                    kind: CommandKind::Custom,
                    identity: sha256(b"custom"),
                    label: "custom".to_string(),
                },
                effect: EffectClass::Pure,
                reads: Vec::new(),
                subprocesses: Vec::new(),
                checkpoint: None,
                state_hash: sha256(b"state"),
            }],
            events: Vec::new(),
            render_backends: Vec::new(),
        }
        .to_bytes()
        .unwrap();

        assert!(matches!(
            Journal::from_bytes(&forged),
            Err(JournalError::Malformed("custom command effect"))
        ));
    }

    #[test]
    fn fallible_record_preserves_custom_effect_coercion() {
        let mut journal = Journal::new();
        journal
            .record(Entry {
                command: CommandRecord {
                    kind: CommandKind::Custom,
                    identity: sha256(b"fallible custom"),
                    label: "fallible custom".to_string(),
                },
                effect: EffectClass::Pure,
                reads: Vec::new(),
                subprocesses: Vec::new(),
                checkpoint: None,
                state_hash: sha256(b"fallible state"),
            })
            .expect("one journal entry reserves");

        assert!(matches!(
            journal.entries().first().map(|entry| &entry.effect),
            Some(EffectClass::Opaque)
        ));
    }

    #[test]
    fn append_storage_refusals_are_typed_and_leave_contents_unchanged() {
        use std::error::Error as _;

        let mut journal = Journal::new();
        let entry_error = journal
            .reserve_entries(usize::MAX)
            .expect_err("impossible entry capacity must refuse");
        assert!(matches!(
            &entry_error,
            JournalError::StorageUnavailable {
                collection: "entries",
                additional: usize::MAX,
                ..
            }
        ));
        assert!(entry_error.source().is_some());
        assert!(journal.entries().is_empty());

        let event_error = journal
            .reserve_events(usize::MAX)
            .expect_err("impossible event capacity must refuse");
        assert!(matches!(
            &event_error,
            JournalError::StorageUnavailable {
                collection: "events",
                additional: usize::MAX,
                ..
            }
        ));
        assert!(event_error.source().is_some());
        assert!(journal.events().is_empty());

        let scratch_error = Journal::event_batch_scratch(usize::MAX)
            .expect_err("impossible batch scratch capacity must refuse");
        assert!(matches!(
            &scratch_error,
            JournalError::StorageUnavailable {
                collection: "event batch scratch",
                additional: usize::MAX,
                ..
            }
        ));
        assert!(scratch_error.source().is_some());
        assert!(journal.events().is_empty());

        let string_error = try_string_with_capacity(usize::MAX, "entry command label bytes")
            .expect_err("impossible entry string capacity must refuse");
        assert!(matches!(
            &string_error,
            JournalError::StorageUnavailable {
                collection: "entry command label bytes",
                additional: usize::MAX,
                ..
            }
        ));
        assert!(string_error.source().is_some());

        let row_error = try_vec_with_capacity::<AssetRead>(usize::MAX, "entry asset reads")
            .expect_err("impossible entry row capacity must refuse");
        assert!(matches!(
            &row_error,
            JournalError::StorageUnavailable {
                collection: "entry asset reads",
                additional: usize::MAX,
                ..
            }
        ));
        assert!(row_error.source().is_some());
    }

    #[test]
    fn fallible_entry_clone_is_exact_independent_and_keeps_record_coercion() {
        let original = Entry {
            command: CommandRecord {
                kind: CommandKind::Custom,
                identity: sha256(b"copy command"),
                label: "custom callback".to_owned(),
            },
            effect: EffectClass::Stateful(vec![ImpureEffectTag::SceneUpdater]),
            reads: vec![AssetRead {
                path: "assets/input.svg".to_owned(),
                digest: sha256(b"asset"),
            }],
            subprocesses: vec![SubprocessRecord {
                tool_sha256_hex: "ffmpeg-digest".to_owned(),
                argv_digest: sha256(b"argv"),
                destination: "output.mp4".to_owned(),
            }],
            checkpoint: Some(b"checkpoint".to_vec()),
            state_hash: sha256(b"state"),
        };
        let mut cloned = original.try_clone().expect("entry fields reserve");

        assert_eq!(cloned.command, original.command);
        assert_eq!(cloned.effect, original.effect);
        assert_eq!(cloned.reads, original.reads);
        assert_eq!(cloned.subprocesses, original.subprocesses);
        assert_eq!(cloned.checkpoint, original.checkpoint);
        assert_eq!(cloned.state_hash, original.state_hash);

        cloned.command.label.push_str(" changed");
        cloned.reads[0].path.push_str(".changed");
        cloned
            .checkpoint
            .as_mut()
            .expect("cloned checkpoint")
            .push(0);
        assert_eq!(original.command.label, "custom callback");
        assert_eq!(original.reads[0].path, "assets/input.svg");
        assert_eq!(
            original.checkpoint.as_deref(),
            Some(b"checkpoint".as_slice())
        );

        let mut journal = Journal::new();
        journal.record(cloned).expect("entry collection reserves");
        assert_eq!(journal.entries()[0].effect, EffectClass::Opaque);
    }

    #[test]
    fn event_batch_refusal_is_atomic_and_success_remains_canonical() {
        let mut journal = Journal::new();
        journal
            .record_event(key_event(1, 0))
            .expect("initial event records");
        let before = journal.events().to_vec();
        let invalid = [key_event(2, 1), key_event(2, 2)];

        assert!(matches!(
            journal.record_events(&invalid),
            Err(JournalError::Event(EventError::ReplayOutOfOrder))
        ));
        assert_eq!(journal.events(), before);

        let canonicalized = InputEvent {
            sequence: 2,
            timestamp: RationalTime::zero(30) + 1,
            payload: EventPayload::KeyPress {
                key: Key::Character('A'),
                modifiers: Modifiers::NONE,
            },
        };
        journal
            .record_events(&[canonicalized])
            .expect("valid batch records");
        assert!(matches!(
            journal.events()[1].payload,
            EventPayload::KeyPress {
                key: Key::Character('a'),
                ..
            }
        ));
    }
}
