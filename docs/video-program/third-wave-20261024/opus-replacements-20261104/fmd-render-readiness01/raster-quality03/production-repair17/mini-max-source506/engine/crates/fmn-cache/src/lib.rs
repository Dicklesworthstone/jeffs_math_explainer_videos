//! The content-addressed store behind typeset, path, and render caches (§14.4).
//!
//! One persistent store, one discipline, three consumers: **typeset results**
//! (fmd-math + text layout, keyed on the string, the preamble-pack hash, the
//! font hashes, and the engine version — the dominant win, PG-7's <100 µs
//! cached path), **fetched assets** (through the `AssetFetcher` capability,
//! addressed by content hash), and **the replay journal's storage** (§13.4 —
//! segments and checkpoint snapshots under a manually-managed namespace).
//!
//! # The discipline
//!
//! - **Content-addressed on fmn-hash.** An entry's address is a SHA-256
//!   [`Digest`](fmn_hash::Digest): either the digest of canonical key material
//!   (built with [`KeyBuilder`], serialized by fmn-hash's canonical format) or
//!   the digest of the content itself ([`Namespace::put_blob`]). Filesystem
//!   paths derive **only** from validated namespace names and digest hex —
//!   arbitrary key bytes never touch a path, which is the traversal
//!   protection: there is no key that can name a path outside the store root.
//! - **Atomic writes.** Entries land once through the capability's
//!   create-if-absent publication; indexes land via `write_atomic`
//!   (write-temp + rename). A reader sees complete bytes or absence — never a
//!   torn intermediate, even under `kill -9`. Re-publishing the same keyed
//!   payload is idempotent; a different payload at an existing key is a typed
//!   [`CacheError::KeyConflict`] and cannot replace the incumbent.
//!   Before any cache read, write, listing, or lifecycle removal, every
//!   component from the owned root down is classified without following its
//!   leaf; links, Windows reparse points, devices, and wrong-kind nodes fail
//!   closed. Missing write directories are created one exact leaf at a time.
//! - **Checksums verified on read.** Entries ride fmn-hash's serial envelope,
//!   whose trailing SHA-256 covers the whole document; the envelope also
//!   records the address it was stored under, and blob entries additionally
//!   self-certify (payload digest = address). Any mismatch — flipped bytes,
//!   truncation, a valid envelope at the wrong address — classifies the entry
//!   as corrupt: it is **evicted and reported as a miss, never trusted, never
//!   fatal**.
//! - **Versioned namespaces.** A namespace is `(name, schema_version)`; its
//!   directory is `ns/<name>/v<version>`. Bumping the version is a clean
//!   invalidation — a cold directory — without touching unrelated namespaces.
//!   Versions coexist until the ownership-authorized whole-store clear: an
//!   opener never guesses that a sibling version is abandoned while another
//!   process may still be using it.
//! - **Cross-process safety.** Entry writes are atomic, immutable, and
//!   digest-addressed: keyed entries derive their address from canonical key
//!   material, while blobs derive it from their content. The first complete
//!   publication for an address wins; later writers verify that winner instead
//!   of replacing it. Ordinary put/get therefore needs no lock. Maintenance
//!   (eviction) takes an advisory lock file with wall-clock staleness breaking;
//!   the LRU index is an advisory hint that eviction reconciles against the
//!   disk truth, so a lost index is a rebuild, never corruption.
//! - **Defined eviction.** LRU-class by logical access sequence with
//!   **pinning** for in-use entries ([`Namespace::pin`]) and a config-visible
//!   size ceiling per namespace ([`NamespacePolicy`]). A `None` ceiling is the
//!   manual policy (the journal namespace: explicit lifecycle, no automatic
//!   eviction).
//!
//! # Determinism
//!
//! A cache is an **optimization, never an oracle**: every key includes the
//! complete semantic inputs (that is the consumer's contract, enforced by
//! construction in [`KeyBuilder`]'s canonical serialization), so a hit is
//! definitionally equivalent to a recompute and certified renders are
//! bit-identical with a cold or warm cache. Every cache failure degrades to a
//! recompute ([`Namespace::get_or_compute`] swallows storage trouble); nothing
//! in this crate can fail a render. `--clear-cache`
//! ([`CacheClearAuthorization`]) retains the path-bound owned root and
//! atomically quarantines only its managed namespace tree, then rotates the
//! owner generation. Concurrent readers see misses; handles from the retired
//! generation cannot mutate, maintain, or flush bookkeeping into the fresh
//! root.
//!
//! Root authorization and ordinary traversal protect against dangerous
//! configuration, static symlinks/reparse points, copied markers, and races
//! among cooperating FrankenManim processes. Each Store carries a path-bound,
//! generation-bearing root context, and every mutation or maintenance pass
//! revalidates its bounded owner manifest immediately before path traversal.
//! Portable safe `std` does not provide the same handle-relative
//! read/write/rename/remove primitives on every supported platform, so this
//! does **not** claim immunity from a hostile same-user process replacing an
//! owned-root ancestor or checked component between validation and the next
//! path-based operation. Recursive deletion is required not to follow
//! link-like children, and clear keeps its deletion boundary to `ns`.
//!
//! LRU bookkeeping uses a logical sequence counter — never wall time — so
//! eviction order is reproducible in the deterministic lab; the only clock
//! use is advisory-lock staleness, which is maintenance, not semantics.
#![forbid(unsafe_code)]

mod entry;
mod key;
mod store;

pub use entry::{Corrupt, EntryKind, decode as decode_entry_envelope};
pub use key::{CacheKey, KeyBuilder};
pub use store::{
    CacheClearAuthorization, CacheClearOutcome, CacheRootError, DEFAULT_CACHE_LEAF, EvictOutcome,
    EvictReport, Namespace, NamespacePolicy, Pin, Store, StoreConfig, object_relative_path,
    resolve_host_cache_root, validate_namespace_name,
};

use fmn_platform::fs::FsError;
use std::fmt;

/// Stable machine-readable reason for [`CacheError::RootRefused`].
///
/// The accompanying human-readable `reason` may gain detail; callers that
/// need policy decisions or robot output match this code instead.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RootRefusalCode {
    /// The configured path is not an admissible absolute cache leaf.
    InvalidPath,
    /// The path names a filesystem root, home, current directory, or an
    /// ancestor of one of those protected locations.
    ProtectedPath,
    /// A host-only lifecycle operation was attempted through another
    /// filesystem capability.
    CapabilityRequired,
    /// A root or marker has the wrong filesystem node kind.
    WrongNodeKind,
    /// An existing root has no ownership manifest.
    OwnershipMissing,
    /// The ownership manifest belongs to another canonical path.
    OwnershipMismatch,
    /// An ownership or format marker is malformed.
    MarkerInvalid,
    /// A lifecycle marker exceeds its fixed read budget.
    MarkerTooLarge,
    /// The canonical root identity changed across a lifecycle check.
    IdentityChanged,
    /// A live handle's owner generation is no longer current.
    GenerationChanged,
    /// A bounded lifecycle-name reservation could not be obtained.
    LifecycleCollision,
}

impl RootRefusalCode {
    /// Stable lowercase spelling for robot diagnostics.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidPath => "invalid_path",
            Self::ProtectedPath => "protected_path",
            Self::CapabilityRequired => "capability_required",
            Self::WrongNodeKind => "wrong_node_kind",
            Self::OwnershipMissing => "ownership_missing",
            Self::OwnershipMismatch => "ownership_mismatch",
            Self::MarkerInvalid => "marker_invalid",
            Self::MarkerTooLarge => "marker_too_large",
            Self::IdentityChanged => "identity_changed",
            Self::GenerationChanged => "generation_changed",
            Self::LifecycleCollision => "lifecycle_collision",
        }
    }
}

/// Stable diagnostic details for [`CacheError::KeyConflict`].
#[derive(Debug)]
pub struct KeyConflict {
    /// Namespace containing the keyed object.
    pub namespace: String,
    /// Namespace schema version.
    pub version: u32,
    /// Canonical key digest naming the immutable object.
    pub key: fmn_hash::Digest,
    /// Hash of the payload already published at `key`.
    pub incumbent_payload: fmn_hash::Digest,
    /// Hash of the payload the losing producer offered.
    pub offered_payload: fmn_hash::Digest,
}

/// A cache failure. Per the never-fatal doctrine, consumers treat every one of
/// these as "skip the cache" — [`Namespace::get_or_compute`] does so
/// structurally — but each is precise for diagnostics.
#[derive(Debug)]
pub enum CacheError {
    /// A namespace name failed validation (the traversal-protection boundary).
    InvalidNamespace {
        /// The offending name.
        name: String,
        /// Why it was rejected.
        reason: &'static str,
    },
    /// The store root carries a format stamp this build does not support; the
    /// remedy is `--clear-cache`.
    FormatUnsupported {
        /// The stamp found on disk.
        found: String,
    },
    /// A store root could not be claimed or cleared without risking foreign
    /// data.
    RootRefused {
        /// The configured or resolved root.
        root: std::path::PathBuf,
        /// Stable refusal class for programmatic consumers.
        code: RootRefusalCode,
        /// The fail-closed reason.
        reason: String,
    },
    /// The effective cache configuration could not be resolved to one
    /// absolute host root.
    RootResolution(CacheRootError),
    /// An entry payload exceeds the configured per-entry ceiling; the caller
    /// skips caching this value.
    EntryTooLarge {
        /// The configured cap ([`StoreConfig::max_entry_bytes`]).
        limit: usize,
        /// The payload size that was offered.
        needed: usize,
    },
    /// Two producers offered different payloads for one immutable keyed
    /// address. The incumbent remains unchanged.
    KeyConflict(Box<KeyConflict>),
    /// The filesystem capability failed.
    Storage(FsError),
    /// Canonical serialization failed (an over-limit field in key material or
    /// an entry envelope).
    Encode(fmn_hash::SerialError),
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNamespace { name, reason } => {
                write!(f, "invalid cache namespace {name:?}: {reason}")
            }
            Self::FormatUnsupported { found } => write!(
                f,
                "unsupported cache store format {found:?}; clear the cache to migrate"
            ),
            Self::RootRefused { root, code, reason } => {
                write!(
                    f,
                    "refusing cache root {} [{}]: {reason}",
                    root.display(),
                    code.as_str()
                )
            }
            Self::RootResolution(err) => write!(f, "cache root resolution failed: {err}"),
            Self::EntryTooLarge { limit, needed } => {
                write!(
                    f,
                    "cache entry too large: {needed} bytes over the {limit}-byte ceiling"
                )
            }
            Self::KeyConflict(conflict) => write!(
                f,
                "conflicting cache producers for namespace {:?} v{}, key {}: \
                 immutable payload {} is already published, offered {}",
                conflict.namespace,
                conflict.version,
                conflict.key.to_hex(),
                conflict.incumbent_payload.to_hex(),
                conflict.offered_payload.to_hex()
            ),
            Self::Storage(err) => write!(f, "cache storage failure: {err}"),
            Self::Encode(err) => write!(f, "cache serialization failure: {err}"),
        }
    }
}

impl std::error::Error for CacheError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Storage(err) => Some(err),
            Self::Encode(err) => Some(err),
            Self::RootResolution(err) => Some(err),
            _ => None,
        }
    }
}

impl From<FsError> for CacheError {
    fn from(err: FsError) -> Self {
        Self::Storage(err)
    }
}

impl From<fmn_hash::SerialError> for CacheError {
    fn from(err: fmn_hash::SerialError) -> Self {
        Self::Encode(err)
    }
}

impl From<CacheRootError> for CacheError {
    fn from(err: CacheRootError) -> Self {
        Self::RootResolution(err)
    }
}

impl Store {
    /// Open the native host store using the ordinary owned-root protocol.
    ///
    /// This explicit convenience boundary supplies `StdFs` and `StdClock`;
    /// it does not weaken root validation or introduce another cache format.
    /// An empty setting uses the platform's dedicated FrankenManim leaf.
    /// Sandboxed hosts and deterministic tests continue to inject their own
    /// capabilities through `open` or `open_host` instead.
    ///
    /// # Errors
    /// The same root-resolution, ownership, format, and storage errors as
    /// [`Self::open_host`]. No fallback directory is selected on failure.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open_native(configured: &str, config: StoreConfig) -> Result<Self, CacheError> {
        Self::open_host(
            std::sync::Arc::new(fmn_platform::fs::StdFs),
            std::sync::Arc::new(fmn_platform::clock::StdClock::new()),
            configured,
            config,
        )
    }
}
