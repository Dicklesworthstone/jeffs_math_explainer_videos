//! Owned DEFLATE compression (RFC 1951) and the zlib wrapper
//! (RFC 1950) — §14.2, under the governed closure (no flate2, no zlib).
//!
//! # The determinism contract
//!
//! Compressed bytes are a pure function of (input, parameters): the
//! matcher's hash, chain order, tie-breaking, lazy rule, Huffman
//! construction (package-merge with total ordering on ties), and
//! block-type choice are all fixed. The same input and level produce
//! the same bytes on every platform, forever — the W8CODEC2
//! deterministic-parallel-PNG gate rides on this.
//!
//! # Fixed block boundaries (the parallel-composition interlock)
//!
//! [`deflate_segment`] compresses one segment to a **byte-aligned**
//! member: its data blocks (all `BFINAL = 0` unless `last`), then — for
//! non-final segments — a sync flush (an empty stored block) that pads
//! to a byte boundary. A full stream is therefore the plain
//! concatenation of its segments' bytes, each of which can be produced
//! independently given the previous 32 KiB of *plaintext* as `dict`.
//! Serial and parallel encodes compose the identical byte string by
//! construction — the parallel path is a composition, not a fork.

use crate::checksum::adler32;
use crate::inflate::{CLCODE_ORDER, DIST_BASE, DIST_EXTRA, LENGTH_BASE, LENGTH_EXTRA};

/// Deterministic effort levels. Each maps to fixed matcher parameters —
/// there are no tunables outside this enum, because every knob is a
/// reproducibility surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompressionLevel {
    /// Shallow chains, greedy matching — the streaming/preview case.
    Fast,
    /// Moderate chains with lazy matching — stills and sequences.
    Default,
    /// Deep chains — archival/certified artifacts where bytes are kept.
    Best,
}

struct MatchParams {
    max_chain: usize,
    lazy: bool,
    nice_len: usize,
}

impl CompressionLevel {
    const fn params(self) -> MatchParams {
        match self {
            Self::Fast => MatchParams {
                max_chain: 16,
                lazy: false,
                nice_len: 32,
            },
            Self::Default => MatchParams {
                max_chain: 128,
                lazy: true,
                nice_len: 130,
            },
            Self::Best => MatchParams {
                max_chain: 4096,
                lazy: true,
                nice_len: 258,
            },
        }
    }
}

const MIN_MATCH: usize = 3;
const MAX_MATCH: usize = 258;
const WINDOW: usize = 32768;
const HASH_BITS: u32 = 15;
const HASH_SIZE: usize = 1 << HASH_BITS;
/// Tokens per emitted block — bounds per-block memory and keeps the
/// Huffman statistics adaptive over long streams.
const MAX_BLOCK_TOKENS: usize = 1 << 16;

/// LSB-first bit writer (RFC 1951 packing).
struct BitWriter {
    out: Vec<u8>,
    bits: u64,
    count: u32,
}

impl BitWriter {
    const fn new() -> Self {
        Self {
            out: Vec::new(),
            bits: 0,
            count: 0,
        }
    }

    fn put(&mut self, value: u32, n: u32) {
        self.bits |= u64::from(value) << self.count;
        self.count += n;
        while self.count >= 8 {
            self.out.push(self.bits as u8);
            self.bits >>= 8;
            self.count -= 8;
        }
    }

    /// Pad the current byte with zero bits.
    fn align(&mut self) {
        if self.count > 0 {
            self.out.push(self.bits as u8);
            self.bits = 0;
            self.count = 0;
        }
    }
}

/// Reverse the low `len` bits of `code` (Huffman codes are emitted
/// most-significant-bit first through the LSB-first writer).
fn reverse_bits(code: u16, len: u8) -> u16 {
    let mut r = 0u16;
    for i in 0..len {
        r |= ((code >> i) & 1) << (len - 1 - i);
    }
    r
}

/// Length-limited Huffman code lengths via boundary package-merge —
/// optimal, and deterministic through a total order on ties
/// (frequency, then leaf-before-package, then symbol).
fn huffman_lengths(freqs: &[u32], max_len: usize, out_len: usize) -> Vec<u8> {
    let mut lengths = vec![0u8; out_len];
    let mut leaves: Vec<(u64, u16)> = freqs
        .iter()
        .enumerate()
        .filter(|&(_, &f)| f > 0)
        .map(|(sym, &f)| (u64::from(f), sym as u16))
        .collect();
    match leaves.len() {
        0 => return lengths,
        1 => {
            lengths[usize::from(leaves[0].1)] = 1;
            return lengths;
        }
        _ => {}
    }
    leaves.sort_unstable();

    // A package is (weight, constituent leaf symbols). Coin-collector:
    // start at the deepest level and merge upward max_len − 1 times.
    type Package = (u64, Vec<u16>);
    let singletons: Vec<Package> = leaves.iter().map(|&(f, s)| (f, vec![s])).collect();
    let mut current = singletons.clone();
    for _ in 1..max_len {
        let paired: Vec<Package> = current
            .chunks(2)
            .filter(|c| c.len() == 2)
            .map(|c| {
                let mut syms = c[0].1.clone();
                syms.extend_from_slice(&c[1].1);
                (c[0].0 + c[1].0, syms)
            })
            .collect();
        // Stable merge: on equal weight, singletons (leaves) first.
        let mut merged = Vec::with_capacity(singletons.len() + paired.len());
        let (mut i, mut j) = (0, 0);
        while i < singletons.len() || j < paired.len() {
            let take_leaf = match (singletons.get(i), paired.get(j)) {
                (Some(a), Some(b)) => a.0 <= b.0,
                (Some(_), None) => true,
                _ => false,
            };
            if take_leaf {
                merged.push(singletons[i].clone());
                i += 1;
            } else {
                merged.push(paired[j].clone());
                j += 1;
            }
        }
        current = merged;
    }
    // The first 2n − 2 packages define the code: each appearance of a
    // leaf adds one to its depth.
    for package in current.iter().take(2 * leaves.len() - 2) {
        for &sym in &package.1 {
            lengths[usize::from(sym)] += 1;
        }
    }
    lengths
}

/// Canonical codes from lengths, pre-reversed for the LSB-first writer.
fn canonical_codes(lengths: &[u8]) -> Vec<u16> {
    let mut count = [0u16; 16];
    for &len in lengths {
        count[usize::from(len)] += 1;
    }
    count[0] = 0;
    let mut next = [0u16; 16];
    let mut code = 0u16;
    for len in 1..16 {
        code = (code + count[len - 1]) << 1;
        next[len] = code;
    }
    let mut codes = vec![0u16; lengths.len()];
    for (sym, &len) in lengths.iter().enumerate() {
        if len != 0 {
            codes[sym] = reverse_bits(next[usize::from(len)], len);
            next[usize::from(len)] += 1;
        }
    }
    codes
}

/// The fixed-Huffman litlen lengths (RFC 1951 §3.2.6).
fn fixed_litlen_lengths() -> Vec<u8> {
    let mut lengths = vec![8u8; 288];
    lengths[144..256].fill(9);
    lengths[256..280].fill(7);
    lengths
}

/// Map a match length (3..=258) to (symbol, extra-bit count, extra value).
fn length_symbol(len: usize) -> (u16, u32, u32) {
    debug_assert!((MIN_MATCH..=MAX_MATCH).contains(&len));
    let mut idx = LENGTH_BASE.len() - 1;
    for (i, &base) in LENGTH_BASE.iter().enumerate() {
        if usize::from(base) > len {
            idx = i - 1;
            break;
        }
    }
    (
        257 + idx as u16,
        u32::from(LENGTH_EXTRA[idx]),
        (len - usize::from(LENGTH_BASE[idx])) as u32,
    )
}

/// Map a distance (1..=32768) to (symbol, extra-bit count, extra value).
fn dist_symbol(dist: usize) -> (u16, u32, u32) {
    debug_assert!((1..=WINDOW).contains(&dist));
    let mut idx = DIST_BASE.len() - 1;
    for (i, &base) in DIST_BASE.iter().enumerate() {
        if usize::from(base) > dist {
            idx = i - 1;
            break;
        }
    }
    (
        idx as u16,
        u32::from(DIST_EXTRA[idx]),
        (dist - usize::from(DIST_BASE[idx])) as u32,
    )
}

#[derive(Clone, Copy)]
enum Token {
    Literal(u8),
    Match { len: u16, dist: u16 },
}

/// The deterministic LZ77 matcher: fixed hash, LIFO chains (nearest
/// candidate first), first-longest wins, bounded walk.
struct Matcher<'a> {
    data: &'a [u8],
    head: Vec<u32>,
    prev: Vec<u32>,
    /// Positions `< inserted` are in the chains.
    inserted: usize,
    params: MatchParams,
}

const NIL: u32 = u32::MAX;

impl<'a> Matcher<'a> {
    fn new(data: &'a [u8], params: MatchParams) -> Self {
        Self {
            data,
            head: vec![NIL; HASH_SIZE],
            prev: vec![NIL; data.len()],
            inserted: 0,
            params,
        }
    }

    fn hash(&self, pos: usize) -> usize {
        let h = (u32::from(self.data[pos]) << 10)
            ^ (u32::from(self.data[pos + 1]) << 5)
            ^ u32::from(self.data[pos + 2]);
        (h.wrapping_mul(2654) & (HASH_SIZE as u32 - 1)) as usize
    }

    /// Insert every position in `[inserted, upto)` into the chains.
    fn insert_to(&mut self, upto: usize) {
        let last_hashable = self.data.len().saturating_sub(MIN_MATCH - 1);
        let end = upto.min(last_hashable);
        while self.inserted < end {
            let h = self.hash(self.inserted);
            self.prev[self.inserted] = self.head[h];
            self.head[h] = self.inserted as u32;
            self.inserted += 1;
        }
        self.inserted = self.inserted.max(upto.min(self.data.len()));
    }

    /// Longest match at `pos` against earlier positions in the window.
    fn best_match(&mut self, pos: usize) -> Option<(usize, usize)> {
        if pos + MIN_MATCH > self.data.len() {
            return None;
        }
        self.insert_to(pos);
        let max_len = (self.data.len() - pos).min(MAX_MATCH);
        let mut best_len = MIN_MATCH - 1;
        let mut best_dist = 0usize;
        let mut candidate = self.head[self.hash(pos)];
        let mut chain = self.params.max_chain;
        while candidate != NIL && chain > 0 {
            let cand = candidate as usize;
            if pos - cand > WINDOW {
                break;
            }
            let len = common_prefix(self.data, cand, pos, max_len);
            if len > best_len {
                best_len = len;
                best_dist = pos - cand;
                if len >= self.params.nice_len {
                    break;
                }
            }
            candidate = self.prev[cand];
            chain -= 1;
        }
        (best_len >= MIN_MATCH).then_some((best_len, best_dist))
    }
}

/// The length of the common prefix of `data[a..]` and `data[b..]`, capped at
/// `max_len` (`a < b`, and `b + max_len <= data.len()`).
///
/// Eight bytes per step: the first differing byte of a word is its lowest
/// set byte of the XOR, so the result is exactly the bytewise count — the
/// matcher's choices, and the encoded stream, do not change.
fn common_prefix(data: &[u8], a: usize, b: usize, max_len: usize) -> usize {
    let mut len = 0;
    while len + 8 <= max_len {
        let x = u64::from_le_bytes(data[a + len..a + len + 8].try_into().expect("8 bytes"));
        let y = u64::from_le_bytes(data[b + len..b + len + 8].try_into().expect("8 bytes"));
        let diff = x ^ y;
        if diff != 0 {
            return len + (diff.trailing_zeros() / 8) as usize;
        }
        len += 8;
    }
    while len < max_len && data[a + len] == data[b + len] {
        len += 1;
    }
    len
}

/// Emit one block, choosing stored / fixed / dynamic by exact bit cost
/// (ties resolve stored ≺ fixed ≺ dynamic — the simpler encoding wins).
fn emit_block(writer: &mut BitWriter, tokens: &[Token], raw: &[u8], bfinal: bool) {
    // Symbol statistics.
    let mut lit_freq = [0u32; 286];
    let mut dist_freq = [0u32; 30];
    let mut extra_bits: u64 = 0;
    for token in tokens {
        match *token {
            Token::Literal(b) => lit_freq[usize::from(b)] += 1,
            Token::Match { len, dist } => {
                let (ls, le, _) = length_symbol(usize::from(len));
                let (ds, de, _) = dist_symbol(usize::from(dist));
                lit_freq[usize::from(ls)] += 1;
                dist_freq[usize::from(ds)] += 1;
                extra_bits += u64::from(le) + u64::from(de);
            }
        }
    }
    lit_freq[256] += 1; // end-of-block

    // Dynamic tables.
    let lit_lengths = huffman_lengths(&lit_freq, 15, 286);
    let mut dist_lengths = huffman_lengths(&dist_freq, 15, 30);
    if dist_lengths.iter().all(|&l| l == 0) {
        // RFC permits signalling "no distance codes", but one code of
        // length 1 is simpler and universally accepted.
        dist_lengths[0] = 1;
    }
    let hlit = lit_lengths
        .iter()
        .rposition(|&l| l != 0)
        .unwrap_or(0)
        .max(256)
        + 1;
    let hdist = dist_lengths.iter().rposition(|&l| l != 0).unwrap_or(0) + 1;

    // Code-length (CL) RLE over the concatenated length sequence.
    let mut sequence: Vec<u8> = Vec::with_capacity(hlit + hdist);
    sequence.extend_from_slice(&lit_lengths[..hlit]);
    sequence.extend_from_slice(&dist_lengths[..hdist]);
    let cl_tokens = rle_code_lengths(&sequence);
    let mut cl_freq = [0u32; 19];
    for &(sym, _, _) in &cl_tokens {
        cl_freq[usize::from(sym)] += 1;
    }
    let cl_lengths = huffman_lengths(&cl_freq, 7, 19);
    let hclen = CLCODE_ORDER
        .iter()
        .rposition(|&slot| cl_lengths[slot] != 0)
        .unwrap_or(0)
        .max(3)
        + 1;

    // Exact costs.
    let sym_cost = |freqs: &[u32], lengths: &[u8]| -> u64 {
        freqs
            .iter()
            .zip(lengths)
            .map(|(&f, &l)| u64::from(f) * u64::from(l))
            .sum()
    };
    let cl_extra: u64 = cl_tokens.iter().map(|&(_, bits, _)| u64::from(bits)).sum();
    let dynamic_cost = 14
        + 3 * hclen as u64
        + sym_cost(&cl_freq, &cl_lengths)
        + cl_extra
        + sym_cost(&lit_freq, &lit_lengths)
        + sym_cost(&dist_freq, &dist_lengths)
        + extra_bits;

    let fixed_lit = fixed_litlen_lengths();
    let fixed_cost = sym_cost(&lit_freq, &fixed_lit)
        + dist_freq.iter().map(|&f| u64::from(f) * 5).sum::<u64>()
        + extra_bits;

    // Stored: 3 header bits, align (worst case counted at current bit
    // position), then 4 + 65535-chunked payload bytes.
    let chunks = raw.len().div_ceil(65535).max(1) as u64;
    let stored_cost = 8 * (raw.len() as u64 + 4 * chunks) + 3 * chunks;

    if stored_cost < fixed_cost.min(dynamic_cost) {
        emit_stored(writer, raw, bfinal);
        return;
    }

    let bfinal_bit = u32::from(bfinal);
    if fixed_cost <= dynamic_cost {
        writer.put(bfinal_bit, 1);
        writer.put(1, 2);
        let lit_codes = canonical_codes(&fixed_lit);
        let dist_codes = canonical_codes(&[5u8; 30]);
        emit_tokens(
            writer,
            tokens,
            &lit_codes,
            &fixed_lit,
            &dist_codes,
            &[5u8; 30],
        );
    } else {
        writer.put(bfinal_bit, 1);
        writer.put(2, 2);
        writer.put((hlit - 257) as u32, 5);
        writer.put((hdist - 1) as u32, 5);
        writer.put((hclen - 4) as u32, 4);
        for &slot in CLCODE_ORDER.iter().take(hclen) {
            writer.put(u32::from(cl_lengths[slot]), 3);
        }
        let cl_codes = canonical_codes(&cl_lengths);
        for &(sym, bits, value) in &cl_tokens {
            writer.put(
                u32::from(cl_codes[usize::from(sym)]),
                u32::from(cl_lengths[usize::from(sym)]),
            );
            if bits > 0 {
                writer.put(value, bits);
            }
        }
        let lit_codes = canonical_codes(&lit_lengths);
        let dist_codes = canonical_codes(&dist_lengths);
        emit_tokens(
            writer,
            tokens,
            &lit_codes,
            &lit_lengths,
            &dist_codes,
            &dist_lengths,
        );
    }
}

/// Stored block(s) for `raw`, chunked at the 65535-byte format limit.
fn emit_stored(writer: &mut BitWriter, raw: &[u8], bfinal: bool) {
    let chunks: Vec<&[u8]> = if raw.is_empty() {
        vec![&[][..]]
    } else {
        raw.chunks(65535).collect()
    };
    for (i, chunk) in chunks.iter().enumerate() {
        let last = bfinal && i + 1 == chunks.len();
        writer.put(u32::from(last), 1);
        writer.put(0, 2);
        writer.align();
        let len = chunk.len() as u16;
        writer.put(u32::from(len & 0xff), 8);
        writer.put(u32::from(len >> 8), 8);
        writer.put(u32::from(!len & 0xff), 8);
        writer.put(u32::from(!len >> 8), 8);
        for &byte in *chunk {
            writer.put(u32::from(byte), 8);
        }
    }
}

fn emit_tokens(
    writer: &mut BitWriter,
    tokens: &[Token],
    lit_codes: &[u16],
    lit_lengths: &[u8],
    dist_codes: &[u16],
    dist_lengths: &[u8],
) {
    let put_code = |w: &mut BitWriter, codes: &[u16], lengths: &[u8], sym: usize| {
        w.put(u32::from(codes[sym]), u32::from(lengths[sym]));
    };
    for token in tokens {
        match *token {
            Token::Literal(b) => put_code(writer, lit_codes, lit_lengths, usize::from(b)),
            Token::Match { len, dist } => {
                let (ls, le, lv) = length_symbol(usize::from(len));
                put_code(writer, lit_codes, lit_lengths, usize::from(ls));
                if le > 0 {
                    writer.put(lv, le);
                }
                let (ds, de, dv) = dist_symbol(usize::from(dist));
                put_code(writer, dist_codes, dist_lengths, usize::from(ds));
                if de > 0 {
                    writer.put(dv, de);
                }
            }
        }
    }
    put_code(writer, lit_codes, lit_lengths, 256);
}

/// RLE-encode a code-length sequence into (CL symbol, extra-bit count,
/// extra value) triples (RFC 1951 §3.2.7).
fn rle_code_lengths(sequence: &[u8]) -> Vec<(u16, u32, u32)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < sequence.len() {
        let value = sequence[i];
        let mut run = 1;
        while i + run < sequence.len() && sequence[i + run] == value {
            run += 1;
        }
        if value == 0 {
            let mut remaining = run;
            while remaining >= 11 {
                let take = remaining.min(138);
                out.push((18, 7, (take - 11) as u32));
                remaining -= take;
            }
            if remaining >= 3 {
                out.push((17, 3, (remaining - 3) as u32));
                remaining = 0;
            }
            for _ in 0..remaining {
                out.push((0, 0, 0));
            }
        } else {
            out.push((u16::from(value), 0, 0));
            let mut remaining = run - 1;
            while remaining >= 3 {
                let take = remaining.min(6);
                out.push((16, 2, (take - 3) as u32));
                remaining -= take;
            }
            for _ in 0..remaining {
                out.push((u16::from(value), 0, 0));
            }
        }
        i += run;
    }
    out
}

/// Compress one segment to a byte-aligned DEFLATE member (see the
/// module docs for the composition contract).
///
/// `dict` is the plaintext immediately preceding `data` in the full
/// stream (at most the trailing 32 KiB is used); pass `&[]` for the
/// first segment. `last` marks the stream's final segment (`BFINAL`);
/// non-final segments end with a sync flush.
#[must_use]
pub fn deflate_segment(dict: &[u8], data: &[u8], level: CompressionLevel, last: bool) -> Vec<u8> {
    let dict_tail = &dict[dict.len().saturating_sub(WINDOW)..];
    let mut combined = Vec::with_capacity(dict_tail.len() + data.len());
    combined.extend_from_slice(dict_tail);
    combined.extend_from_slice(data);
    let start = dict_tail.len();

    let params = level.params();
    let lazy = params.lazy;
    let nice = params.nice_len;
    let mut matcher = Matcher::new(&combined, params);
    let mut writer = BitWriter::new();

    let mut tokens: Vec<Token> = Vec::new();
    let mut block_start = start;
    let mut pos = start;
    let mut final_emitted = false;
    while pos < combined.len() {
        let found = matcher.best_match(pos);
        let action = match found {
            Some((len, dist)) if lazy && len < nice && pos + 1 < combined.len() => {
                // Lazy: prefer a strictly longer match one byte later.
                match matcher.best_match(pos + 1) {
                    Some((next_len, _)) if next_len > len => None,
                    _ => Some((len, dist)),
                }
            }
            other => other,
        };
        match action {
            Some((len, dist)) => {
                tokens.push(Token::Match {
                    len: len as u16,
                    dist: dist as u16,
                });
                pos += len;
            }
            None => {
                tokens.push(Token::Literal(combined[pos]));
                pos += 1;
            }
        }
        if tokens.len() >= MAX_BLOCK_TOKENS {
            let is_final = last && pos == combined.len();
            emit_block(&mut writer, &tokens, &combined[block_start..pos], is_final);
            final_emitted |= is_final;
            tokens.clear();
            block_start = pos;
        }
    }
    if !tokens.is_empty() || block_start == start {
        // Data blocks carry BFINAL only on the final segment (an empty
        // final segment emits an empty block for the BFINAL bit).
        emit_block(&mut writer, &tokens, &combined[block_start..pos], last);
        final_emitted |= last;
    }
    if last && !final_emitted {
        emit_block(&mut writer, &[], &[], true);
    }
    if last {
        writer.align();
    } else {
        // Sync flush: empty stored block, leaves the stream byte-aligned.
        writer.put(0, 1);
        writer.put(0, 2);
        writer.align();
        writer.out.extend_from_slice(&[0x00, 0x00, 0xff, 0xff]);
    }
    writer.out
}

/// Compress `data` as one complete DEFLATE stream.
#[must_use]
pub fn deflate(data: &[u8], level: CompressionLevel) -> Vec<u8> {
    deflate_segment(&[], data, level, true)
}

/// The two-byte zlib header (RFC 1950) for `level`.
#[must_use]
pub fn zlib_header(level: CompressionLevel) -> [u8; 2] {
    let flevel: u8 = match level {
        CompressionLevel::Fast => 0,
        CompressionLevel::Default => 2,
        CompressionLevel::Best => 3,
    };
    let cmf: u8 = 0x78; // deflate, 32 KiB window
    let mut flg = flevel << 6;
    let check = ((u16::from(cmf) << 8) | u16::from(flg)) % 31;
    if check != 0 {
        flg += (31 - check) as u8;
    }
    [cmf, flg]
}

/// Compress `data` as a zlib stream (RFC 1950).
#[must_use]
pub fn zlib_compress(data: &[u8], level: CompressionLevel) -> Vec<u8> {
    let mut out = zlib_header(level).to_vec();
    out.extend_from_slice(&deflate(data, level));
    out.extend_from_slice(&adler32(data).to_be_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::common_prefix;

    #[test]
    fn word_prefix_matches_the_bytewise_count() {
        // Runs, near-runs and noise: every (a, b, cap) agrees with the
        // byte-at-a-time definition the matcher used before.
        let mut data = Vec::new();
        let mut x = 0x2545_f491_4f6c_dd1du64;
        for i in 0..2048u32 {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            data.push(match i % 512 {
                0..=255 => 7,
                256..=383 => (i % 3) as u8,
                _ => x as u8,
            });
        }
        for a in (0..600).step_by(7) {
            for b in (a + 1..a + 700).step_by(13) {
                for cap in [0, 1, 7, 8, 9, 15, 16, 63, 258] {
                    if b + cap > data.len() {
                        continue;
                    }
                    let mut want = 0;
                    while want < cap && data[a + want] == data[b + want] {
                        want += 1;
                    }
                    assert_eq!(
                        common_prefix(&data, a, b, cap),
                        want,
                        "a={a} b={b} cap={cap}"
                    );
                }
            }
        }
    }
}
