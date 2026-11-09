//! Canonical hashing and versioned serialization — the content-addressing and
//! durable-format substrate (§6.7, D-17).
//!
//! Two owned primitives, shared by everything that must turn *meaning* into
//! *bytes* deterministically:
//!
//! - [`sha256()`] / [`Sha256`] / [`Digest`] — an in-house SHA-256 (FIPS 180-4),
//!   the content-address hash for the cache, arena snapshots, provenance
//!   manifests, and asset/font digests. Owned rather than pulled from a crate
//!   because the governed closure (D1) admits no external crypto dependency and
//!   because content addressing needs a *fixed*, byte-exact target — which a
//!   published standard is and a moving dependency is not.
//!
//! - [`serial`] — a versioned canonical binary format (magic, fixed
//!   little-endian, defined field order, integrity checksum, size limits, and a
//!   semver-style major/minor migration policy) that backs cache keys, arena
//!   snapshots, `SceneState`, the replay journal's content hashes, and the
//!   provenance sidecar. Float fields are canonicalized at the boundary
//!   (`-0.0 → +0.0`, NaN → the one canonical NaN) in this dependency-free
//!   substrate, so equal values always hash equally on every platform.
//!
//! Certified determinism (§16.7) hashes the complete input closure and requires
//! that equal hashes mean equal meaning; any nondeterminism in serialization —
//! map-iteration order, locale-sensitive float text, host endianness — would
//! silently break certification. That is why this layer is *specified* here
//! rather than derived from a general-purpose serializer.
#![forbid(unsafe_code)]

pub mod serial;
pub mod sha256;

pub use serial::{Error as SerialError, Limits, Reader, Schema, UnknownPolicy, Writer, fuzz_probe};
pub use sha256::{Digest, HexError, Sha256, sha256};
