//! An owned SHA-256 primitive (FIPS 180-4) — the content-addressing hash for
//! the cache, snapshots, provenance manifests, and asset/font digests.
//!
//! Built in-house per the governed closure (D1): no external crypto crate is
//! allowed into an authoritative fmn crate. SHA-256 is a stable, published
//! standard, so an owned implementation is a fixed target locked by the FIPS
//! 180-4 test vectors (see the tests) rather than a moving dependency.
//!
//! The engine uses this for *content addressing*, not for secrecy: equality of
//! digests must mean equality of bytes on every platform, which SHA-256's
//! big-endian, byte-exact specification guarantees regardless of host
//! endianness.

use core::fmt;

/// The 64 round constants K[0..64]: the first 32 bits of the fractional parts
/// of the cube roots of the first 64 primes (FIPS 180-4 §4.2.2).
#[rustfmt::skip]
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// The initial hash values H[0..8]: the first 32 bits of the fractional parts
/// of the square roots of the first 8 primes (FIPS 180-4 §5.3.3).
const H0: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

/// A 256-bit SHA-256 digest. `Copy` and comparable; hex is the canonical
/// display form and the on-the-wire form in provenance manifests.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Digest([u8; 32]);

impl Digest {
    /// The raw 32 bytes, big-endian per the standard (H0..H7 concatenated).
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Consume into the raw 32 bytes.
    #[must_use]
    pub const fn into_bytes(self) -> [u8; 32] {
        self.0
    }

    /// Wrap raw bytes as a digest (for parsing a stored/received value).
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Lowercase hex, 64 characters, no prefix.
    #[must_use]
    pub fn to_hex(&self) -> String {
        let mut s = String::with_capacity(64);
        for &b in &self.0 {
            // Two lowercase hex nibbles, no allocation churn.
            const HEX: &[u8; 16] = b"0123456789abcdef";
            s.push(HEX[(b >> 4) as usize] as char);
            s.push(HEX[(b & 0x0f) as usize] as char);
        }
        s
    }

    /// Parse a 64-character lowercase-or-uppercase hex string into a digest.
    ///
    /// # Errors
    /// Returns [`HexError`] if the length is not 64 or a non-hex byte appears.
    pub fn from_hex(s: &str) -> Result<Self, HexError> {
        let bytes = s.as_bytes();
        if bytes.len() != 64 {
            return Err(HexError::BadLength { len: bytes.len() });
        }
        let mut out = [0u8; 32];
        for (i, out_byte) in out.iter_mut().enumerate() {
            let hi = hex_val(bytes[2 * i]).ok_or(HexError::BadChar { at: 2 * i })?;
            let lo = hex_val(bytes[2 * i + 1]).ok_or(HexError::BadChar { at: 2 * i + 1 })?;
            *out_byte = (hi << 4) | lo;
        }
        Ok(Self(out))
    }
}

const fn hex_val(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

impl fmt::Debug for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Digest({})", self.to_hex())
    }
}

impl fmt::LowerHex for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

/// Failure parsing a hex-encoded digest.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HexError {
    /// Not exactly 64 hex characters.
    BadLength { len: usize },
    /// A non-hex byte at the given index.
    BadChar { at: usize },
}

impl fmt::Display for HexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadLength { len } => write!(f, "hex digest must be 64 chars, got {len}"),
            Self::BadChar { at } => write!(f, "non-hex character at index {at}"),
        }
    }
}

impl std::error::Error for HexError {}

/// Incremental SHA-256 hasher. Feed bytes with [`update`](Self::update), then
/// [`finalize`](Self::finalize). The incremental form is what lets consumers
/// hash a large input closure (sources, config bytes, font files) without
/// materializing it in one buffer.
#[derive(Clone)]
pub struct Sha256 {
    state: [u32; 8],
    /// Partial block awaiting a full 64 bytes.
    block: [u8; 64],
    /// Bytes currently buffered in `block` (0..64).
    buffered: usize,
    /// Total message length in bytes (for the length padding).
    total_len: u64,
}

impl Default for Sha256 {
    fn default() -> Self {
        Self::new()
    }
}

impl Sha256 {
    /// A fresh hasher primed with the standard initial state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: H0,
            block: [0u8; 64],
            buffered: 0,
            total_len: 0,
        }
    }

    /// Absorb `data` into the hash.
    pub fn update(&mut self, mut data: &[u8]) {
        self.total_len = self.total_len.wrapping_add(data.len() as u64);

        // Top up a partial block first.
        if self.buffered > 0 {
            let want = 64 - self.buffered;
            let take = want.min(data.len());
            self.block[self.buffered..self.buffered + take].copy_from_slice(&data[..take]);
            self.buffered += take;
            data = &data[take..];
            if self.buffered == 64 {
                let block = self.block;
                self.compress(&block);
                self.buffered = 0;
            }
        }

        // Compress whole blocks straight from the input.
        while data.len() >= 64 {
            let mut block = [0u8; 64];
            block.copy_from_slice(&data[..64]);
            self.compress(&block);
            data = &data[64..];
        }

        // Stash the remainder.
        if !data.is_empty() {
            self.block[..data.len()].copy_from_slice(data);
            self.buffered = data.len();
        }
    }

    /// Finish and produce the digest. Consumes the hasher; clone first if the
    /// running state must be reused.
    #[must_use]
    pub fn finalize(mut self) -> Digest {
        // Message length in bits, captured before padding is appended.
        let bit_len = self.total_len.wrapping_mul(8);

        // Append 0x80, then zeros until 8 bytes short of a block boundary.
        self.update(&[0x80]);
        // `update` bumped total_len; pad relative to the current buffer fill.
        while self.buffered != 56 {
            self.update(&[0x00]);
        }
        // Append the 64-bit big-endian bit length; this completes a block.
        self.update(&bit_len.to_be_bytes());
        debug_assert_eq!(self.buffered, 0);

        let mut out = [0u8; 32];
        for (i, word) in self.state.iter().enumerate() {
            out[4 * i..4 * i + 4].copy_from_slice(&word.to_be_bytes());
        }
        Digest(out)
    }

    /// The SHA-256 compression function over one 512-bit block.
    fn compress(&mut self, block: &[u8; 64]) {
        let mut w = [0u32; 64];
        for (i, word) in w.iter_mut().enumerate().take(16) {
            *word = u32::from_be_bytes([
                block[4 * i],
                block[4 * i + 1],
                block[4 * i + 2],
                block[4 * i + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.state;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }

        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
        self.state[5] = self.state[5].wrapping_add(f);
        self.state[6] = self.state[6].wrapping_add(g);
        self.state[7] = self.state[7].wrapping_add(h);
    }
}

/// One-shot convenience: `sha256(bytes)` == `Sha256::new().update(bytes).finalize()`.
#[must_use]
pub fn sha256(data: &[u8]) -> Digest {
    let mut h = Sha256::new();
    h.update(data);
    h.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(data: &[u8]) -> String {
        sha256(data).to_hex()
    }

    #[test]
    fn fips_180_4_vectors() {
        // The canonical published SHA-256 test vectors.
        assert_eq!(
            hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
    }

    #[test]
    fn million_a_vector() {
        // FIPS 180-4: 1,000,000 repetitions of 'a', fed in awkward chunks to
        // exercise the incremental buffering across block boundaries.
        let mut h = Sha256::new();
        let chunk = [b'a'; 1000];
        for _ in 0..1000 {
            h.update(&chunk);
        }
        assert_eq!(
            h.finalize().to_hex(),
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
        );
    }

    #[test]
    fn incremental_matches_oneshot_at_all_split_points() {
        let msg: Vec<u8> = (0u16..200).map(|i| (i % 256) as u8).collect();
        let want = sha256(&msg);
        for split in 0..=msg.len() {
            let mut h = Sha256::new();
            h.update(&msg[..split]);
            h.update(&msg[split..]);
            assert_eq!(h.finalize(), want, "split at {split}");
        }
    }

    #[test]
    fn hex_round_trip() {
        let d = sha256(b"franken_manim");
        let parsed = Digest::from_hex(&d.to_hex()).unwrap();
        assert_eq!(d, parsed);
        // Uppercase is accepted too.
        assert_eq!(Digest::from_hex(&d.to_hex().to_uppercase()).unwrap(), d);
    }

    #[test]
    fn hex_rejects_bad_input() {
        assert_eq!(Digest::from_hex("abc"), Err(HexError::BadLength { len: 3 }));
        let mut s = sha256(b"x").to_hex();
        s.replace_range(5..6, "z");
        assert_eq!(Digest::from_hex(&s), Err(HexError::BadChar { at: 5 }));
    }
}
