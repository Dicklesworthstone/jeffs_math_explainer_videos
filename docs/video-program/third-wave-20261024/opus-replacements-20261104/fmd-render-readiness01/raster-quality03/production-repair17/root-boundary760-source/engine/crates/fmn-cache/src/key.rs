//! Cache keys: canonical key material hashed to a content address.
//!
//! A [`CacheKey`] is nothing but a SHA-256 digest — the *address* of an entry.
//! [`KeyBuilder`] derives that digest from structured key material through
//! fmn-hash's canonical serialization, so equal meaning always produces equal
//! addresses (fixed field order, little-endian integers, canonicalized
//! floats) and a key can never be built from unordered or platform-dependent
//! bytes by accident. The digest — never the material — is what touches the
//! filesystem, which is half of the store's traversal protection.

use fmn_hash::{Digest, Schema, SerialError, Writer, sha256};

/// The serial schema for key material. Bumping `major` here re-addresses every
/// keyed entry (a deliberate, global invalidation); consumers version their
/// *own* key shapes with the domain string and explicit fields instead.
const KEY_SCHEMA: Schema = Schema::new(*b"FMNC", 3, 1, 0);

/// The address of a cache entry: a SHA-256 digest of canonical key material
/// ([`KeyBuilder`]) or of raw content ([`CacheKey::of_content`]).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct CacheKey {
    digest: Digest,
}

impl CacheKey {
    /// The content address for `bytes` themselves — blob addressing, used for
    /// fetched assets where the address must equal the content hash.
    #[must_use]
    pub fn of_content(bytes: &[u8]) -> Self {
        Self {
            digest: sha256(bytes),
        }
    }

    /// Wrap an already-computed digest as a key (e.g. an asset digest recorded
    /// in a manifest).
    #[must_use]
    pub const fn from_digest(digest: Digest) -> Self {
        Self { digest }
    }

    /// The underlying digest.
    #[must_use]
    pub const fn digest(&self) -> &Digest {
        &self.digest
    }
}

/// Builds a [`CacheKey`] from structured key material in a fixed field order.
///
/// The push-call sequence *is* the key schema: consumers must push the
/// complete semantic inputs of the cached computation (that completeness is
/// what makes a cache hit definitionally equivalent to a recompute, the
/// determinism contract's requirement) in a fixed order, starting from a
/// domain string that separates unrelated key shapes sharing a namespace.
///
/// ```
/// use fmn_cache::KeyBuilder;
/// use fmn_hash::sha256;
///
/// let font_digest = sha256(b"...font bytes...");
/// let key = KeyBuilder::new("typeset/tex")
///     .push_str(r"\int_0^\infty e^{-x^2}\,dx")
///     .push_str("default") // preamble pack
///     .push_digest(&font_digest)
///     .push_u32(2) // engine layout version
///     .finish()
///     .expect("key material within limits");
/// let again = KeyBuilder::new("typeset/tex")
///     .push_str(r"\int_0^\infty e^{-x^2}\,dx")
///     .push_str("default")
///     .push_digest(&font_digest)
///     .push_u32(2)
///     .finish()
///     .expect("key material within limits");
/// assert_eq!(key, again);
/// ```
pub struct KeyBuilder {
    writer: Writer,
}

impl KeyBuilder {
    /// Start key material under `domain` — a short, fixed label for the key
    /// shape (e.g. `"typeset/tex"`), pushed as the first field.
    #[must_use]
    pub fn new(domain: &str) -> Self {
        let mut writer = Writer::new(KEY_SCHEMA);
        writer.put_str(domain);
        Self { writer }
    }

    /// Append a string field.
    #[must_use]
    pub fn push_str(mut self, v: &str) -> Self {
        self.writer.put_str(v);
        self
    }

    /// Append a raw byte field.
    #[must_use]
    pub fn push_bytes(mut self, v: &[u8]) -> Self {
        self.writer.put_bytes(v);
        self
    }

    /// Append a `bool` field.
    #[must_use]
    pub fn push_bool(mut self, v: bool) -> Self {
        self.writer.put_bool(v);
        self
    }

    /// Append a `u32` field.
    #[must_use]
    pub fn push_u32(mut self, v: u32) -> Self {
        self.writer.put_u32(v);
        self
    }

    /// Append a `u64` field.
    #[must_use]
    pub fn push_u64(mut self, v: u64) -> Self {
        self.writer.put_u64(v);
        self
    }

    /// Append an `i64` field.
    #[must_use]
    pub fn push_i64(mut self, v: i64) -> Self {
        self.writer.put_i64(v);
        self
    }

    /// Append an `f64` field, canonicalized (`-0.0 → +0.0`, one NaN) by the
    /// serializer so equal values always produce equal addresses.
    #[must_use]
    pub fn push_f64(mut self, v: f64) -> Self {
        self.writer.put_f64(v);
        self
    }

    /// Append a digest field (a font hash, a pack hash, an asset address).
    #[must_use]
    pub fn push_digest(mut self, v: &Digest) -> Self {
        self.writer.put_digest(v);
        self
    }

    /// Seal the material and hash it into the address.
    ///
    /// # Errors
    /// [`SerialError`] if a field exceeded the canonical format's limits.
    pub fn finish(self) -> Result<CacheKey, SerialError> {
        let doc = self.writer.finish()?;
        Ok(CacheKey {
            digest: sha256(&doc),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_material_equal_address() {
        let a = KeyBuilder::new("d")
            .push_str("x")
            .push_u32(7)
            .finish()
            .unwrap();
        let b = KeyBuilder::new("d")
            .push_str("x")
            .push_u32(7)
            .finish()
            .unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn any_field_difference_changes_the_address() {
        let base = KeyBuilder::new("d")
            .push_str("x")
            .push_u32(7)
            .finish()
            .unwrap();
        let other_domain = KeyBuilder::new("e")
            .push_str("x")
            .push_u32(7)
            .finish()
            .unwrap();
        let other_str = KeyBuilder::new("d")
            .push_str("y")
            .push_u32(7)
            .finish()
            .unwrap();
        let other_int = KeyBuilder::new("d")
            .push_str("x")
            .push_u32(8)
            .finish()
            .unwrap();
        assert_ne!(base, other_domain);
        assert_ne!(base, other_str);
        assert_ne!(base, other_int);
    }

    #[test]
    fn field_boundaries_are_framed_not_concatenated() {
        // "ab" + "c" must not collide with "a" + "bc": length prefixes frame
        // every variable-width field.
        let a = KeyBuilder::new("d")
            .push_str("ab")
            .push_str("c")
            .finish()
            .unwrap();
        let b = KeyBuilder::new("d")
            .push_str("a")
            .push_str("bc")
            .finish()
            .unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn float_canonicalization_reaches_the_address() {
        let neg = KeyBuilder::new("d").push_f64(-0.0).finish().unwrap();
        let pos = KeyBuilder::new("d").push_f64(0.0).finish().unwrap();
        assert_eq!(neg, pos);
    }

    #[test]
    fn content_addressing_matches_sha256() {
        let key = CacheKey::of_content(b"asset bytes");
        assert_eq!(*key.digest(), sha256(b"asset bytes"));
        assert_eq!(CacheKey::from_digest(*key.digest()), key);
    }

    #[test]
    fn address_is_stable_across_builds() {
        // A self-golden: this hex is the address of this exact material under
        // key-schema 1.0. If it moves, every persisted keyed entry silently
        // cold-misses — that must be a deliberate, reviewed change.
        let key = KeyBuilder::new("golden")
            .push_str("material")
            .push_u32(1)
            .finish()
            .unwrap();
        let hex = key.digest().to_hex();
        assert_eq!(
            hex, "f0ef2225f0235912603a561e68436dc7605e278bd3c4fbf65cc72196aacd0581",
            "key-schema hash moved; bump KEY_SCHEMA major deliberately if intended"
        );
    }
}
