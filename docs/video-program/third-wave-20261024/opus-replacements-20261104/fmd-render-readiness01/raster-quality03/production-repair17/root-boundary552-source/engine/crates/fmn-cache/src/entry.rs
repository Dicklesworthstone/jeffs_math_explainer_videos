//! The on-disk entry envelope: fmn-hash's serial container around a payload,
//! carrying the address it was stored under.
//!
//! The container's trailing SHA-256 is the per-entry checksum verified on
//! every read; the recorded address defends against a valid envelope sitting
//! at the wrong path (a mis-placed or maliciously copied file); blob entries
//! additionally self-certify (payload digest = address). Every decode failure
//! collapses to one classification — corrupt — and the store's response is
//! always the same: evict, miss, recompute. Never trusted, never fatal.

use fmn_hash::{Digest, Limits, Reader, Schema, SerialError, UnknownPolicy, Writer, sha256};

/// The serial schema for cache entries. `major` bumps re-address nothing but
/// invalidate every entry on read (decode failure → evict + miss), which is
/// exactly the versioned-invalidation the store wants.
const ENTRY_SCHEMA: Schema = Schema::new(*b"FMNC", 1, 1, 0);

/// How an entry is addressed; recorded in the envelope and checked on read so
/// a keyed entry can never satisfy a blob lookup or vice versa.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EntryKind {
    /// Addressed by the digest of canonical key material.
    Keyed = 1,
    /// Addressed by the digest of the payload itself (self-certifying).
    Blob = 2,
}

/// Why a decode was rejected. Diagnostic only — every variant is handled
/// identically by the store (evict + miss), so the payloads are read by the
/// tests and by `Debug`, not by production code paths.
#[derive(Debug)]
pub enum Corrupt {
    /// Framing, checksum, or size-limit failure in the envelope.
    Envelope(#[cfg_attr(not(test), allow(dead_code))] SerialError),
    /// The envelope decodes but its kind byte is not a known [`EntryKind`].
    UnknownKind(#[cfg_attr(not(test), allow(dead_code))] u8),
    /// The recorded kind does not match the accessor.
    KindMismatch,
    /// The recorded address does not match the path the entry was read from.
    AddressMismatch,
    /// A blob's payload does not hash to its address.
    ContentMismatch,
}

impl From<SerialError> for Corrupt {
    fn from(err: SerialError) -> Self {
        Self::Envelope(err)
    }
}

/// Encode `payload` as an entry stored at `address`.
pub fn encode(
    kind: EntryKind,
    address: &Digest,
    payload: &[u8],
    limits: Limits,
) -> Result<Vec<u8>, SerialError> {
    let mut w = Writer::with_limits(ENTRY_SCHEMA, limits);
    w.put_u8(kind as u8);
    w.put_digest(address);
    w.put_bytes(payload);
    w.finish()
}

/// Decode an entry read from the path for `address`, verifying the envelope
/// checksum, the recorded kind, the recorded address, and — for blobs — the
/// payload's self-certification. Returns the payload.
pub fn decode(
    bytes: &[u8],
    kind: EntryKind,
    address: &Digest,
    limits: Limits,
) -> Result<Vec<u8>, Corrupt> {
    let mut r = Reader::open(bytes, ENTRY_SCHEMA, limits, UnknownPolicy::Strict)?;
    let stored_kind = r.get_u8()?;
    let stored_address = r.get_digest()?;
    let payload = r.get_bytes()?.to_vec();
    r.finish()?;

    let stored_kind = match stored_kind {
        1 => EntryKind::Keyed,
        2 => EntryKind::Blob,
        other => return Err(Corrupt::UnknownKind(other)),
    };
    if stored_kind != kind {
        return Err(Corrupt::KindMismatch);
    }
    if stored_address != *address {
        return Err(Corrupt::AddressMismatch);
    }
    if kind == EntryKind::Blob && sha256(&payload) != *address {
        return Err(Corrupt::ContentMismatch);
    }
    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIMITS: Limits = Limits::DEFAULT;

    #[test]
    fn keyed_round_trip() {
        let addr = sha256(b"key material");
        let doc = encode(EntryKind::Keyed, &addr, b"value", LIMITS).unwrap();
        let payload = decode(&doc, EntryKind::Keyed, &addr, LIMITS).unwrap();
        assert_eq!(payload, b"value");
    }

    #[test]
    fn blob_round_trip_self_certifies() {
        let addr = sha256(b"content");
        let doc = encode(EntryKind::Blob, &addr, b"content", LIMITS).unwrap();
        assert_eq!(
            decode(&doc, EntryKind::Blob, &addr, LIMITS).unwrap(),
            b"content"
        );
        // A blob whose payload does not hash to its address is corrupt even
        // when the envelope checksum is intact.
        let forged = encode(EntryKind::Blob, &addr, b"other bytes", LIMITS).unwrap();
        assert!(matches!(
            decode(&forged, EntryKind::Blob, &addr, LIMITS),
            Err(Corrupt::ContentMismatch)
        ));
    }

    #[test]
    fn every_flipped_byte_is_detected() {
        let addr = sha256(b"k");
        let doc = encode(EntryKind::Keyed, &addr, b"payload bytes", LIMITS).unwrap();
        for i in 0..doc.len() {
            let mut bad = doc.clone();
            bad[i] ^= 0x01;
            assert!(
                decode(&bad, EntryKind::Keyed, &addr, LIMITS).is_err(),
                "flip at byte {i} went undetected"
            );
        }
    }

    #[test]
    fn truncation_is_detected_never_panics() {
        let addr = sha256(b"k");
        let doc = encode(EntryKind::Keyed, &addr, b"payload", LIMITS).unwrap();
        for cut in 0..doc.len() {
            match decode(&doc[..cut], EntryKind::Keyed, &addr, LIMITS) {
                Ok(_) => panic!("prefix of len {cut} unexpectedly decoded"),
                // Every truncation is a precise envelope error, carried for
                // diagnostics.
                Err(Corrupt::Envelope(err)) => {
                    let _ = format!("{err}");
                }
                Err(other) => panic!("prefix of len {cut}: unexpected class {other:?}"),
            }
        }
    }

    #[test]
    fn unknown_kind_byte_is_corrupt() {
        // A well-formed envelope whose kind byte is from the future: decode
        // reports the byte, then the store evicts like any other corruption.
        let addr = sha256(b"k");
        let mut w = Writer::with_limits(ENTRY_SCHEMA, LIMITS);
        w.put_u8(7);
        w.put_digest(&addr);
        w.put_bytes(b"v");
        let doc = w.finish().unwrap();
        match decode(&doc, EntryKind::Keyed, &addr, LIMITS) {
            Err(Corrupt::UnknownKind(found)) => assert_eq!(found, 7),
            other => panic!("expected UnknownKind, got {other:?}"),
        }
    }

    #[test]
    fn kind_and_address_are_bound() {
        let addr = sha256(b"k");
        let doc = encode(EntryKind::Keyed, &addr, b"v", LIMITS).unwrap();
        assert!(matches!(
            decode(&doc, EntryKind::Blob, &addr, LIMITS),
            Err(Corrupt::KindMismatch)
        ));
        let elsewhere = sha256(b"different key");
        assert!(matches!(
            decode(&doc, EntryKind::Keyed, &elsewhere, LIMITS),
            Err(Corrupt::AddressMismatch)
        ));
    }
}
