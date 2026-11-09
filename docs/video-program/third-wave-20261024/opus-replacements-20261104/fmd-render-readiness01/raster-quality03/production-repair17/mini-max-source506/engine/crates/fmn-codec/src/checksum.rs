//! CRC-32 and Adler-32, owned (§14.2; the governed closure admits no
//! compression/checksum crates).
//!
//! CRC-32 is the PNG chunk checksum (ISO 3309 / ITU-T V.42, reflected,
//! polynomial 0xEDB88320). Adler-32 is the zlib (RFC 1950) stream
//! checksum. Both are deterministic byte-serial definitions; SIMD
//! variants, if ever needed, must match these bit-for-bit (§17.3).

/// The reflected CRC-32 table, built at compile time.
const CRC_TABLE: [u32; 256] = {
    let mut table = [0u32; 256];
    let mut n = 0;
    while n < 256 {
        let mut c = n as u32;
        let mut k = 0;
        while k < 8 {
            c = if c & 1 != 0 {
                0xedb8_8320 ^ (c >> 1)
            } else {
                c >> 1
            };
            k += 1;
        }
        table[n] = c;
        n += 1;
    }
    table
};

/// A running CRC-32 (IEEE, reflected). Feed bytes, then [`Crc32::value`].
#[derive(Debug, Clone)]
pub struct Crc32 {
    state: u32,
}

impl Default for Crc32 {
    fn default() -> Self {
        Self::new()
    }
}

impl Crc32 {
    /// A fresh checksum.
    #[must_use]
    pub const fn new() -> Self {
        Self { state: 0xffff_ffff }
    }

    /// Absorb `data`.
    pub fn update(&mut self, data: &[u8]) {
        let mut c = self.state;
        for &byte in data {
            c = CRC_TABLE[((c ^ u32::from(byte)) & 0xff) as usize] ^ (c >> 8);
        }
        self.state = c;
    }

    /// The finalized checksum value.
    #[must_use]
    pub const fn value(&self) -> u32 {
        self.state ^ 0xffff_ffff
    }
}

/// One-shot CRC-32 of `data`.
#[must_use]
pub fn crc32(data: &[u8]) -> u32 {
    let mut c = Crc32::new();
    c.update(data);
    c.value()
}

const ADLER_MOD: u32 = 65521;
/// Largest n with 255n(n+1)/2 + (n+1)(MOD−1) < 2³² — the standard
/// batching bound that defers the modulo.
const ADLER_NMAX: usize = 5552;

/// One-shot Adler-32 of `data` (RFC 1950).
#[must_use]
pub fn adler32(data: &[u8]) -> u32 {
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for chunk in data.chunks(ADLER_NMAX) {
        for &byte in chunk {
            a += u32::from(byte);
            b += a;
        }
        a %= ADLER_MOD;
        b %= ADLER_MOD;
    }
    (b << 16) | a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc32_vectors() {
        // The canonical check value.
        assert_eq!(crc32(b"123456789"), 0xcbf4_3926);
        assert_eq!(crc32(b""), 0);
        // PNG's IEND chunk: crc over type + (empty) data.
        assert_eq!(crc32(b"IEND"), 0xae42_6082);
    }

    #[test]
    fn crc32_is_incremental() {
        let mut c = Crc32::new();
        c.update(b"1234");
        c.update(b"56789");
        assert_eq!(c.value(), crc32(b"123456789"));
    }

    #[test]
    fn adler32_vectors() {
        assert_eq!(adler32(b""), 1);
        // The canonical "Wikipedia" vector.
        assert_eq!(adler32(b"Wikipedia"), 0x11e6_0398);
        // Batching boundary exercise.
        let big = vec![0xabu8; ADLER_NMAX * 3 + 17];
        let mut a: u64 = 1;
        let mut b: u64 = 0;
        for _ in 0..big.len() {
            a = (a + 0xab) % u64::from(ADLER_MOD);
            b = (b + a) % u64::from(ADLER_MOD);
        }
        assert_eq!(adler32(&big), ((b as u32) << 16) | a as u32);
    }
}
