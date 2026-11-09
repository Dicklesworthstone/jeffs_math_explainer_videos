//! Owned GIF89a encoder (§14.2) — the native `--format gif` path; no
//! ffmpeg anywhere near it.
//!
//! Deterministic by construction: the color histogram is ordered
//! (BTreeMap), median-cut box selection and splits use total orders
//! with fixed tie-breaks, the palette is sorted, Floyd–Steinberg
//! scans a fixed order with integer error arithmetic, and the LZW
//! dictionary is an ordered map — the same frames produce the same
//! bytes, always (the golden test locks a digest).
//!
//! Frame timing comes from the exact rational clock: per-frame delays
//! are successive differences of `round(100·k·den/num)`, so
//! centisecond rounding never accumulates drift. Alpha below 128 maps
//! to a reserved transparent index (GIF's 1-bit transparency).

use std::collections::BTreeMap;

/// Typed refusals of the GIF encoder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GifError {
    /// Zero dimensions, zero frames, or a zero frame rate.
    EmptyInput(&'static str),
    /// A frame buffer's length does not match the geometry.
    BadFrameSize {
        /// The frame that mismatched.
        frame: usize,
        /// Expected byte length.
        expected: usize,
        /// Actual byte length.
        got: usize,
    },
    /// GIF dimensions are 16-bit.
    TooLarge,
}

impl std::fmt::Display for GifError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyInput(what) => write!(f, "gif encode: empty {what}"),
            Self::BadFrameSize {
                frame,
                expected,
                got,
            } => write!(
                f,
                "gif frame {frame} is {got} bytes, geometry requires {expected}"
            ),
            Self::TooLarge => write!(f, "gif dimensions exceed 65535"),
        }
    }
}

impl std::error::Error for GifError {}

const TRANSPARENT_THRESHOLD: u8 = 128;
const MAX_COLORS: usize = 256;

/// A duration-independent GIF89a encoder.
///
/// Each frame carries a deterministic local palette. That keeps resident
/// memory bounded by one RGBA frame plus its encoded chunk and avoids a
/// whole-render palette pass. Palette identity has no visual semantics in
/// GIF; local tables also give every frame the full color budget.
#[derive(Debug)]
pub struct GifStreamEncoder {
    width: u32,
    height: u32,
    fps: (u32, u32),
    loop_forever: bool,
    next_frame: u64,
}

impl GifStreamEncoder {
    /// Validate a streaming GIF configuration.
    ///
    /// # Errors
    /// [`GifError`] for zero/oversized geometry or a zero frame rate.
    pub fn new(
        width: u32,
        height: u32,
        fps: (u32, u32),
        loop_forever: bool,
    ) -> Result<Self, GifError> {
        if width == 0 || height == 0 {
            return Err(GifError::EmptyInput("dimensions"));
        }
        if width > 0xffff || height > 0xffff {
            return Err(GifError::TooLarge);
        }
        if fps.0 == 0 || fps.1 == 0 {
            return Err(GifError::EmptyInput("frame rate"));
        }
        Ok(Self {
            width,
            height,
            fps,
            loop_forever,
            next_frame: 0,
        })
    }

    /// GIF89a header and optional forever-loop extension.
    #[must_use]
    pub fn header(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(b"GIF89a");
        out.extend_from_slice(&(self.width as u16).to_le_bytes());
        out.extend_from_slice(&(self.height as u16).to_le_bytes());
        // No global color table. Every frame declares its local table.
        out.extend_from_slice(&[0x70, 0, 0]);
        if self.loop_forever {
            out.extend_from_slice(&[0x21, 0xff, 0x0b]);
            out.extend_from_slice(b"NETSCAPE2.0");
            out.extend_from_slice(&[0x03, 0x01, 0x00, 0x00, 0x00]);
        }
        out
    }

    /// Encode the next full-canvas RGBA frame with a local palette.
    ///
    /// # Errors
    /// [`GifError::BadFrameSize`] when `rgba` does not match the configured
    /// geometry.
    #[allow(clippy::cast_possible_truncation)]
    pub fn encode_frame(&mut self, rgba: &[u8]) -> Result<Vec<u8>, GifError> {
        let (width, height) = (self.width as usize, self.height as usize);
        let expected = width
            .checked_mul(height)
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or(GifError::TooLarge)?;
        if rgba.len() != expected {
            return Err(GifError::BadFrameSize {
                frame: usize::try_from(self.next_frame).unwrap_or(usize::MAX),
                expected,
                got: rgba.len(),
            });
        }

        let mut histogram = BTreeMap::new();
        let mut any_transparent = false;
        for pixel in rgba.as_chunks::<4>().0 {
            if pixel[3] < TRANSPARENT_THRESHOLD {
                any_transparent = true;
            } else {
                *histogram
                    .entry(pack(pixel[0], pixel[1], pixel[2]))
                    .or_insert(0) += 1;
            }
        }
        let color_budget = if any_transparent {
            MAX_COLORS - 1
        } else {
            MAX_COLORS
        };
        let palette = median_cut(&histogram, color_budget);
        let transparent_index = any_transparent.then_some(palette.len());
        let total_entries = palette.len() + usize::from(any_transparent);
        let mut table_bits = 1u8;
        while (1usize << table_bits) < total_entries.max(2) {
            table_bits += 1;
        }
        let table_len = 1usize << table_bits;
        let min_code_size = table_bits.max(2);

        let delay_at = |frame: u64| -> u128 {
            let numerator =
                100u128 * u128::from(frame) * u128::from(self.fps.1) + u128::from(self.fps.0) / 2;
            numerator / u128::from(self.fps.0)
        };
        let delay = (delay_at(self.next_frame + 1) - delay_at(self.next_frame))
            .min(u128::from(u16::MAX)) as u16;

        let mut out = Vec::new();
        out.extend_from_slice(&[0x21, 0xf9, 0x04]);
        out.push(if transparent_index.is_some() {
            0x09
        } else {
            0x04
        });
        out.extend_from_slice(&delay.to_le_bytes());
        out.push(transparent_index.unwrap_or(0) as u8);
        out.push(0);
        out.push(0x2c);
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&(self.width as u16).to_le_bytes());
        out.extend_from_slice(&(self.height as u16).to_le_bytes());
        out.push(0x80 | (table_bits - 1));
        for &[r, g, b] in &palette {
            out.extend_from_slice(&[r, g, b]);
        }
        for _ in palette.len()..table_len {
            out.extend_from_slice(&[0, 0, 0]);
        }
        let indices = dither(rgba, width, height, &palette, transparent_index);
        out.push(min_code_size);
        let compressed = lzw_compress(&indices, min_code_size);
        for block in compressed.chunks(255) {
            out.push(block.len() as u8);
            out.extend_from_slice(block);
        }
        out.push(0);
        self.next_frame = self.next_frame.saturating_add(1);
        Ok(out)
    }

    /// GIF trailer byte.
    #[must_use]
    pub const fn trailer() -> [u8; 1] {
        [0x3b]
    }

    /// Frames encoded so far.
    #[must_use]
    pub const fn frame_count(&self) -> u64 {
        self.next_frame
    }
}

fn pack(r: u8, g: u8, b: u8) -> u32 {
    (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b)
}

fn unpack(c: u32) -> [u8; 3] {
    [(c >> 16) as u8, (c >> 8) as u8, c as u8]
}

/// Median-cut quantization over an ordered histogram. Returns ≤
/// `target` colors, sorted by packed RGB for palette determinism.
fn median_cut(histogram: &BTreeMap<u32, u64>, target: usize) -> Vec<[u8; 3]> {
    #[derive(Clone)]
    struct Box {
        colors: Vec<(u32, u64)>,
    }
    impl Box {
        /// (channel, range) of the widest channel — the split axis.
        fn widest(&self) -> (usize, u8) {
            let mut lo = [255u8; 3];
            let mut hi = [0u8; 3];
            for &(c, _) in &self.colors {
                let rgb = unpack(c);
                for ch in 0..3 {
                    lo[ch] = lo[ch].min(rgb[ch]);
                    hi[ch] = hi[ch].max(rgb[ch]);
                }
            }
            let ranges = [hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]];
            let channel = (0..3).max_by_key(|&ch| ranges[ch]).unwrap_or(0);
            (channel, ranges[channel])
        }
    }

    if histogram.is_empty() {
        return vec![[0, 0, 0]];
    }
    let mut boxes = vec![Box {
        colors: histogram.iter().map(|(&c, &n)| (c, n)).collect(),
    }];
    while boxes.len() < target {
        // Split the box with the widest channel range (ties: lowest
        // index — a total order).
        let Some((index, (channel, range))) = boxes
            .iter()
            .enumerate()
            .map(|(i, b)| (i, b.widest()))
            .filter(|&(_, (_, range))| range > 0)
            .max_by(|a, b| a.1.1.cmp(&b.1.1).then(b.0.cmp(&a.0)))
        else {
            break; // nothing splittable
        };
        let _ = range;
        let mut splitting = boxes.swap_remove(index);
        splitting
            .colors
            .sort_by_key(|&(c, _)| (unpack(c)[channel], c));
        // Weighted median split point.
        let total: u64 = splitting.colors.iter().map(|&(_, n)| n).sum();
        let mut acc = 0u64;
        let mut cut = splitting.colors.len() - 1;
        for (i, &(_, n)) in splitting.colors.iter().enumerate() {
            acc += n;
            if acc * 2 >= total {
                cut = i + 1;
                break;
            }
        }
        let cut = cut.clamp(1, splitting.colors.len() - 1);
        let right = splitting.colors.split_off(cut);
        boxes.push(splitting);
        boxes.push(Box { colors: right });
    }
    let mut palette: Vec<[u8; 3]> = boxes
        .iter()
        .map(|b| {
            // Weighted mean, round half up.
            let mut sums = [0u64; 3];
            let mut count = 0u64;
            for &(c, n) in &b.colors {
                let rgb = unpack(c);
                for ch in 0..3 {
                    sums[ch] += u64::from(rgb[ch]) * n;
                }
                count += n;
            }
            let mut mean = [0u8; 3];
            for ch in 0..3 {
                mean[ch] = ((sums[ch] * 2 + count) / (count * 2)) as u8;
            }
            mean
        })
        .collect();
    palette.sort_unstable_by_key(|&[r, g, b]| pack(r, g, b));
    palette.dedup();
    palette
}

/// Nearest palette index by squared distance; ties take the lowest
/// index (the palette is sorted, so this is a total order).
fn nearest(palette: &[[u8; 3]], r: i32, g: i32, b: i32) -> usize {
    let mut best = 0usize;
    let mut best_d = i64::MAX;
    for (i, &[pr, pg, pb]) in palette.iter().enumerate() {
        let dr = i64::from(r - i32::from(pr));
        let dg = i64::from(g - i32::from(pg));
        let db = i64::from(b - i32::from(pb));
        let d = dr * dr + dg * dg + db * db;
        if d < best_d {
            best_d = d;
            best = i;
        }
    }
    best
}

/// Floyd–Steinberg dither one frame to palette indices. Fixed
/// left→right, top→bottom order; integer error arithmetic (>> 4 with
/// the standard 7/3/5/1 weights); transparent pixels take
/// `transparent_index` and diffuse no error.
fn dither(
    rgba: &[u8],
    width: usize,
    height: usize,
    palette: &[[u8; 3]],
    transparent_index: Option<usize>,
) -> Vec<u8> {
    let mut indices = vec![0u8; width * height];
    // Error rows: (r, g, b) per pixel, current and next.
    let mut err_now = vec![[0i32; 3]; width + 2];
    let mut err_next = vec![[0i32; 3]; width + 2];
    for y in 0..height {
        for x in 0..width {
            let at = (y * width + x) * 4;
            if let Some(t) = transparent_index
                && rgba[at + 3] < TRANSPARENT_THRESHOLD
            {
                indices[y * width + x] = t as u8;
                continue;
            }
            let e = err_now[x + 1];
            let r = (i32::from(rgba[at]) + e[0]).clamp(0, 255);
            let g = (i32::from(rgba[at + 1]) + e[1]).clamp(0, 255);
            let b = (i32::from(rgba[at + 2]) + e[2]).clamp(0, 255);
            let pick = nearest(palette, r, g, b);
            indices[y * width + x] = pick as u8;
            let [pr, pg, pb] = palette[pick];
            let err = [r - i32::from(pr), g - i32::from(pg), b - i32::from(pb)];
            for ch in 0..3 {
                err_now[x + 2][ch] += err[ch] * 7 / 16;
                err_next[x][ch] += err[ch] * 3 / 16;
                err_next[x + 1][ch] += err[ch] * 5 / 16;
                err_next[x + 2][ch] += err[ch] / 16;
            }
        }
        std::mem::swap(&mut err_now, &mut err_next);
        err_next.iter_mut().for_each(|e| *e = [0; 3]);
    }
    indices
}

/// GIF-variant LZW compression of an index stream.
fn lzw_compress(indices: &[u8], min_code_size: u8) -> Vec<u8> {
    struct BitPacker {
        out: Vec<u8>,
        bits: u32,
        count: u32,
    }
    impl BitPacker {
        fn put(&mut self, code: u16, width: u32) {
            self.bits |= u32::from(code) << self.count;
            self.count += width;
            while self.count >= 8 {
                self.out.push(self.bits as u8);
                self.bits >>= 8;
                self.count -= 8;
            }
        }
        fn flush(&mut self) {
            if self.count > 0 {
                self.out.push(self.bits as u8);
                self.bits = 0;
                self.count = 0;
            }
        }
    }

    let clear: u16 = 1 << min_code_size;
    let eoi: u16 = clear + 1;
    let mut packer = BitPacker {
        out: Vec::new(),
        bits: 0,
        count: 0,
    };
    let mut dict: BTreeMap<(u16, u8), u16> = BTreeMap::new();
    let mut next: u16 = eoi + 1;
    let mut width: u32 = u32::from(min_code_size) + 1;
    packer.put(clear, width);

    let mut iter = indices.iter();
    let Some(&first) = iter.next() else {
        packer.put(eoi, width);
        packer.flush();
        return packer.out;
    };
    let mut w: u16 = u16::from(first);
    for &k in iter {
        if let Some(&code) = dict.get(&(w, k)) {
            w = code;
            continue;
        }
        packer.put(w, width);
        if next == 4096 {
            packer.put(clear, width);
            dict.clear();
            next = eoi + 1;
            width = u32::from(min_code_size) + 1;
        } else {
            dict.insert((w, k), next);
            if next == (1 << width) && width < 12 {
                width += 1;
            }
            next += 1;
        }
        w = u16::from(k);
    }
    packer.put(w, width);
    packer.put(eoi, width);
    packer.flush();
    packer.out
}

/// Encode RGBA frames as an animated GIF89a.
///
/// `fps` is the exact rational frame rate; per-frame centisecond
/// delays are drift-free successive differences. `loop_forever` emits
/// the NETSCAPE looping extension.
///
/// # Errors
/// Every refusal in [`GifError`].
#[allow(clippy::cast_possible_truncation)]
pub fn encode_gif(
    width: u32,
    height: u32,
    frames: &[&[u8]],
    fps: (u32, u32),
    loop_forever: bool,
) -> Result<Vec<u8>, GifError> {
    if width == 0 || height == 0 {
        return Err(GifError::EmptyInput("dimensions"));
    }
    if width > 0xffff || height > 0xffff {
        return Err(GifError::TooLarge);
    }
    if frames.is_empty() {
        return Err(GifError::EmptyInput("frame list"));
    }
    if fps.0 == 0 || fps.1 == 0 {
        return Err(GifError::EmptyInput("frame rate"));
    }
    let (w, h) = (width as usize, height as usize);
    let frame_bytes = w * h * 4;
    for (i, frame) in frames.iter().enumerate() {
        if frame.len() != frame_bytes {
            return Err(GifError::BadFrameSize {
                frame: i,
                expected: frame_bytes,
                got: frame.len(),
            });
        }
    }

    // Ordered histogram over every opaque pixel of every frame.
    let mut histogram: BTreeMap<u32, u64> = BTreeMap::new();
    let mut any_transparent = false;
    for frame in frames {
        for px in frame.as_chunks::<4>().0 {
            if px[3] < TRANSPARENT_THRESHOLD {
                any_transparent = true;
            } else {
                *histogram.entry(pack(px[0], px[1], px[2])).or_insert(0) += 1;
            }
        }
    }
    let color_budget = if any_transparent {
        MAX_COLORS - 1
    } else {
        MAX_COLORS
    };
    let palette = median_cut(&histogram, color_budget);
    let transparent_index = any_transparent.then_some(palette.len());
    let total_entries = palette.len() + usize::from(any_transparent);

    // GCT size: power of two ≥ max(2, total).
    let mut gct_bits = 1u8;
    while (1usize << gct_bits) < total_entries.max(2) {
        gct_bits += 1;
    }
    let gct_len = 1usize << gct_bits;
    let min_code_size = gct_bits.max(2);

    let mut out = Vec::new();
    out.extend_from_slice(b"GIF89a");
    out.extend_from_slice(&(width as u16).to_le_bytes());
    out.extend_from_slice(&(height as u16).to_le_bytes());
    // GCT present, color resolution 7, size bits.
    out.push(0x80 | 0x70 | (gct_bits - 1));
    out.push(0); // background color index
    out.push(0); // square pixels
    for &[r, g, b] in &palette {
        out.extend_from_slice(&[r, g, b]);
    }
    // Pad the table to its declared power-of-two size (the transparent
    // slot and any remainder are black).
    for _ in palette.len()..gct_len {
        out.extend_from_slice(&[0, 0, 0]);
    }

    if loop_forever {
        out.extend_from_slice(&[0x21, 0xff, 0x0b]);
        out.extend_from_slice(b"NETSCAPE2.0");
        out.extend_from_slice(&[0x03, 0x01, 0x00, 0x00, 0x00]);
    }

    // Drift-free centisecond schedule.
    let delay_at =
        |k: u64| -> u64 { (100 * k * u64::from(fps.1) + u64::from(fps.0) / 2) / u64::from(fps.0) };

    for (i, frame) in frames.iter().enumerate() {
        let delay = (delay_at(i as u64 + 1) - delay_at(i as u64)).min(u64::from(u16::MAX)) as u16;
        // Graphic Control Extension.
        out.extend_from_slice(&[0x21, 0xf9, 0x04]);
        let flags = if transparent_index.is_some() {
            0x09 // disposal: restore to background; transparency on
        } else {
            0x04 // disposal: do not dispose
        };
        out.push(flags);
        out.extend_from_slice(&delay.to_le_bytes());
        out.push(transparent_index.unwrap_or(0) as u8);
        out.push(0);
        // Image descriptor: full frame, no local palette.
        out.push(0x2c);
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&(width as u16).to_le_bytes());
        out.extend_from_slice(&(height as u16).to_le_bytes());
        out.push(0);
        // Pixel data.
        let indices = dither(frame, w, h, &palette, transparent_index);
        out.push(min_code_size);
        let compressed = lzw_compress(&indices, min_code_size);
        for block in compressed.chunks(255) {
            out.push(block.len() as u8);
            out.extend_from_slice(block);
        }
        out.push(0);
    }
    out.push(0x3b);
    Ok(out)
}
