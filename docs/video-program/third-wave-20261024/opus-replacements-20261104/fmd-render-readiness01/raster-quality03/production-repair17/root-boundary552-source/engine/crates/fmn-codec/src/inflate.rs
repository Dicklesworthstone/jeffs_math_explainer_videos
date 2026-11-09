//! Owned DEFLATE decompression (RFC 1951) and the zlib wrapper
//! (RFC 1950) — §14.2, under the untrusted-input rules of §16.5/R14.
//!
//! This is a hostile-input parser: the output budget is declared BEFORE
//! decompression begins and enforced on every byte written, so a
//! decompression bomb is a typed refusal, never an allocation. Every
//! loop is bounded by the (finite) input or the budget — the decoder
//! cannot hang and cannot overallocate.

/// Typed refusals of the DEFLATE/zlib decoders.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InflateError {
    /// The stream ended mid-structure.
    UnexpectedEof,
    /// Reserved block type 3.
    InvalidBlockType,
    /// A stored block's LEN/NLEN complement check failed.
    InvalidStoredLength,
    /// A Huffman code table is over-subscribed or malformed.
    InvalidCodeLengths,
    /// A bit sequence decoded to no assigned symbol.
    InvalidCode,
    /// A length/distance symbol outside the defined alphabet.
    InvalidSymbol,
    /// A back-reference reaches before the start of output.
    DistanceTooFar,
    /// The declared output budget was exceeded — the bomb refusal.
    OutputLimit {
        /// The declared budget in bytes.
        limit: usize,
    },
    /// Trailing zlib checksum mismatch.
    AdlerMismatch,
    /// The zlib header is malformed or requests an unsupported feature
    /// (a preset dictionary — PNG streams never carry one).
    InvalidZlibHeader,
}

impl std::fmt::Display for InflateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedEof => write!(f, "deflate stream ended unexpectedly"),
            Self::InvalidBlockType => write!(f, "reserved deflate block type"),
            Self::InvalidStoredLength => write!(f, "stored block length complement mismatch"),
            Self::InvalidCodeLengths => write!(f, "malformed huffman code lengths"),
            Self::InvalidCode => write!(f, "bit sequence decodes to no symbol"),
            Self::InvalidSymbol => write!(f, "symbol outside the deflate alphabet"),
            Self::DistanceTooFar => write!(f, "back-reference before start of output"),
            Self::OutputLimit { limit } => {
                write!(
                    f,
                    "decompressed output would exceed the {limit}-byte budget"
                )
            }
            Self::AdlerMismatch => write!(f, "zlib adler-32 checksum mismatch"),
            Self::InvalidZlibHeader => write!(f, "invalid or unsupported zlib header"),
        }
    }
}

impl std::error::Error for InflateError {}

/// LSB-first bit reader over a byte slice.
struct BitReader<'a> {
    data: &'a [u8],
    /// Next unread byte.
    pos: usize,
    /// Bits not yet consumed, LSB-aligned.
    bits: u32,
    /// Count of valid bits in `bits`.
    count: u32,
}

impl<'a> BitReader<'a> {
    const fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            bits: 0,
            count: 0,
        }
    }

    /// Read `n` bits (n ≤ 16), LSB-first.
    fn take(&mut self, n: u32) -> Result<u32, InflateError> {
        while self.count < n {
            let byte = *self.data.get(self.pos).ok_or(InflateError::UnexpectedEof)?;
            self.bits |= u32::from(byte) << self.count;
            self.count += 8;
            self.pos += 1;
        }
        let value = self.bits & ((1u32 << n) - 1);
        self.bits >>= n;
        self.count -= n;
        Ok(value)
    }

    /// Read one bit.
    fn bit(&mut self) -> Result<u32, InflateError> {
        self.take(1)
    }

    /// Discard partial-byte bits and return the byte-aligned remainder
    /// cursor.
    fn align_to_byte(&mut self) {
        let whole = self.count / 8;
        self.pos -= whole as usize;
        self.bits = 0;
        self.count = 0;
    }

    /// Copy `len` bytes verbatim (caller must be byte-aligned).
    fn take_bytes(&mut self, len: usize) -> Result<&'a [u8], InflateError> {
        let end = self
            .pos
            .checked_add(len)
            .ok_or(InflateError::UnexpectedEof)?;
        if end > self.data.len() {
            return Err(InflateError::UnexpectedEof);
        }
        let slice = &self.data[self.pos..end];
        self.pos = end;
        Ok(slice)
    }
}

/// A canonical Huffman decoder: per-length symbol counts plus the
/// symbols sorted by (length, symbol) — RFC 1951 §3.2.2.
struct Huffman {
    counts: [u16; 16],
    symbols: Vec<u16>,
}

impl Huffman {
    /// Build from per-symbol code lengths (0 = unused). Refuses
    /// over-subscribed tables; incomplete tables build, and decoding a
    /// gap yields [`InflateError::InvalidCode`] (matching zlib's
    /// permissiveness for the legal single-code distance tables).
    fn new(lengths: &[u8]) -> Result<Self, InflateError> {
        let mut counts = [0u16; 16];
        for &len in lengths {
            if len > 15 {
                return Err(InflateError::InvalidCodeLengths);
            }
            counts[len as usize] += 1;
        }
        counts[0] = 0;
        // Over-subscription check.
        let mut left: i32 = 1;
        for &count in &counts[1..] {
            left = (left << 1) - i32::from(count);
            if left < 0 {
                return Err(InflateError::InvalidCodeLengths);
            }
        }
        // Symbols in canonical order: offsets per length, then place.
        let mut offsets = [0u16; 16];
        for len in 1..15 {
            offsets[len + 1] = offsets[len] + counts[len];
        }
        let total = usize::from(offsets[15] + counts[15]);
        let mut symbols = vec![0u16; total];
        for (symbol, &len) in lengths.iter().enumerate() {
            if len != 0 {
                symbols[usize::from(offsets[len as usize])] = symbol as u16;
                offsets[len as usize] += 1;
            }
        }
        Ok(Self { counts, symbols })
    }

    /// Decode one symbol, walking the canonical code bit by bit
    /// (bounded: at most 15 iterations).
    fn decode(&self, reader: &mut BitReader<'_>) -> Result<u16, InflateError> {
        let mut code: u32 = 0;
        let mut first: u32 = 0;
        let mut index: u32 = 0;
        for len in 1..=15 {
            code |= reader.bit()?;
            let count = u32::from(self.counts[len]);
            if code < first + count {
                return Ok(self.symbols[(index + code - first) as usize]);
            }
            index += count;
            first = (first + count) << 1;
            code <<= 1;
        }
        Err(InflateError::InvalidCode)
    }
}

/// Length symbol (257..=285) → (base, extra bits). RFC 1951 §3.2.5.
pub(crate) const LENGTH_BASE: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
pub(crate) const LENGTH_EXTRA: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
/// Distance symbol (0..=29) → (base, extra bits).
pub(crate) const DIST_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
pub(crate) const DIST_EXTRA: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];
/// The code-length-code transmission order. RFC 1951 §3.2.7.
pub(crate) const CLCODE_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

/// The fixed literal/length code lengths (RFC 1951 §3.2.6).
fn fixed_litlen_lengths() -> Vec<u8> {
    let mut lengths = vec![8u8; 288];
    lengths[144..256].fill(9);
    lengths[256..280].fill(7);
    lengths
}

/// Push one byte within budget.
fn push_limited(out: &mut Vec<u8>, byte: u8, limit: usize) -> Result<(), InflateError> {
    if out.len() >= limit {
        return Err(InflateError::OutputLimit { limit });
    }
    out.push(byte);
    Ok(())
}

/// Decompress a raw DEFLATE stream, refusing to produce more than
/// `max_output` bytes.
pub fn inflate(data: &[u8], max_output: usize) -> Result<Vec<u8>, InflateError> {
    let mut reader = BitReader::new(data);
    let mut out: Vec<u8> = Vec::new();

    loop {
        let bfinal = reader.bit()?;
        match reader.take(2)? {
            0 => {
                // Stored.
                reader.align_to_byte();
                let header = reader.take_bytes(4)?;
                let len16 = u16::from_le_bytes([header[0], header[1]]);
                let nlen = u16::from_le_bytes([header[2], header[3]]);
                if len16 != !nlen {
                    return Err(InflateError::InvalidStoredLength);
                }
                let len = usize::from(len16);
                let bytes = reader.take_bytes(len)?;
                if out.len() + len > max_output {
                    return Err(InflateError::OutputLimit { limit: max_output });
                }
                out.extend_from_slice(bytes);
            }
            btype @ (1 | 2) => {
                let (litlen, dist) = if btype == 1 {
                    (
                        Huffman::new(&fixed_litlen_lengths())?,
                        Huffman::new(&[5u8; 30])?,
                    )
                } else {
                    read_dynamic_tables(&mut reader)?
                };
                inflate_block(&mut reader, &litlen, &dist, &mut out, max_output)?;
            }
            _ => return Err(InflateError::InvalidBlockType),
        }
        if bfinal == 1 {
            return Ok(out);
        }
    }
}

/// Read the dynamic-block code tables (RFC 1951 §3.2.7).
fn read_dynamic_tables(reader: &mut BitReader<'_>) -> Result<(Huffman, Huffman), InflateError> {
    let hlit = reader.take(5)? as usize + 257;
    let hdist = reader.take(5)? as usize + 1;
    let hclen = reader.take(4)? as usize + 4;
    if hlit > 286 || hdist > 30 {
        return Err(InflateError::InvalidCodeLengths);
    }
    let mut cl_lengths = [0u8; 19];
    for &slot in CLCODE_ORDER.iter().take(hclen) {
        cl_lengths[slot] = reader.take(3)? as u8;
    }
    let cl_code = Huffman::new(&cl_lengths)?;

    let mut lengths = vec![0u8; hlit + hdist];
    let mut i = 0;
    while i < lengths.len() {
        match cl_code.decode(reader)? {
            sym @ 0..=15 => {
                lengths[i] = sym as u8;
                i += 1;
            }
            16 => {
                if i == 0 {
                    return Err(InflateError::InvalidCodeLengths);
                }
                let prev = lengths[i - 1];
                let repeat = reader.take(2)? as usize + 3;
                if i + repeat > lengths.len() {
                    return Err(InflateError::InvalidCodeLengths);
                }
                lengths[i..i + repeat].fill(prev);
                i += repeat;
            }
            17 => {
                let repeat = reader.take(3)? as usize + 3;
                if i + repeat > lengths.len() {
                    return Err(InflateError::InvalidCodeLengths);
                }
                i += repeat;
            }
            18 => {
                let repeat = reader.take(7)? as usize + 11;
                if i + repeat > lengths.len() {
                    return Err(InflateError::InvalidCodeLengths);
                }
                i += repeat;
            }
            _ => return Err(InflateError::InvalidSymbol),
        }
    }
    // The end-of-block code must exist.
    if lengths[256] == 0 {
        return Err(InflateError::InvalidCodeLengths);
    }
    let litlen = Huffman::new(&lengths[..hlit])?;
    let dist = Huffman::new(&lengths[hlit..])?;
    Ok((litlen, dist))
}

/// Decode one compressed block's symbol stream.
fn inflate_block(
    reader: &mut BitReader<'_>,
    litlen: &Huffman,
    dist: &Huffman,
    out: &mut Vec<u8>,
    max_output: usize,
) -> Result<(), InflateError> {
    loop {
        match litlen.decode(reader)? {
            sym @ 0..=255 => push_limited(out, sym as u8, max_output)?,
            256 => return Ok(()),
            sym @ 257..=285 => {
                let idx = usize::from(sym - 257);
                let length = usize::from(LENGTH_BASE[idx])
                    + reader.take(u32::from(LENGTH_EXTRA[idx]))? as usize;
                let dsym = dist.decode(reader)?;
                if dsym > 29 {
                    return Err(InflateError::InvalidSymbol);
                }
                let didx = usize::from(dsym);
                let distance = usize::from(DIST_BASE[didx])
                    + reader.take(u32::from(DIST_EXTRA[didx]))? as usize;
                if distance > out.len() {
                    return Err(InflateError::DistanceTooFar);
                }
                // Overlapping copy, byte by byte (RFC semantics).
                for _ in 0..length {
                    let byte = out[out.len() - distance];
                    push_limited(out, byte, max_output)?;
                }
            }
            _ => return Err(InflateError::InvalidSymbol),
        }
    }
}

/// Decompress a zlib stream (RFC 1950): header, DEFLATE body, Adler-32
/// trailer. Preset dictionaries are refused (PNG never uses them).
pub fn zlib_decompress(data: &[u8], max_output: usize) -> Result<Vec<u8>, InflateError> {
    if data.len() < 6 {
        return Err(InflateError::InvalidZlibHeader);
    }
    let cmf = data[0];
    let flg = data[1];
    let method = cmf & 0x0f;
    let cinfo = cmf >> 4;
    if method != 8 || cinfo > 7 || ((u16::from(cmf) << 8) | u16::from(flg)) % 31 != 0 {
        return Err(InflateError::InvalidZlibHeader);
    }
    if flg & 0x20 != 0 {
        // FDICT.
        return Err(InflateError::InvalidZlibHeader);
    }
    let body = &data[2..data.len() - 4];
    let out = inflate(body, max_output)?;
    let trailer = &data[data.len() - 4..];
    let expected = u32::from_be_bytes([trailer[0], trailer[1], trailer[2], trailer[3]]);
    if crate::checksum::adler32(&out) != expected {
        return Err(InflateError::AdlerMismatch);
    }
    Ok(out)
}
