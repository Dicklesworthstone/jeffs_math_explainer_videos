//! Owned JPEG decoder (§14.2): baseline + progressive, the common
//! subsampling matrix, restart markers, EXIF orientation.
//!
//! Decode only — exports are PNG/video, so there is no JPEG encoder.
//!
//! # Scope and policy
//!
//! - Baseline (SOF0) and extended sequential (SOF1) and progressive
//!   (SOF2) Huffman JPEGs; arithmetic coding, lossless, and
//!   hierarchical modes are named refusals.
//! - Component counts 1 (grayscale) and 3 (YCbCr, or RGB when an Adobe
//!   APP14 marker declares transform 0). **CMYK/YCCK (4-component) is
//!   rejected**, by policy: animation assets are RGB; converting
//!   CMYK without a profile silently misrenders, so the refusal names
//!   the format instead.
//! - Sampling factors 1–2 in each axis (4:4:4, 4:2:2, 4:2:0, 4:4:0);
//!   larger factors are a named refusal.
//! - EXIF orientation (APP1, tag 0x0112) is honored: the decoded RGBA
//!   is already in output orientation (D-23), dimensions swapped for
//!   the transposed orientations. A malformed EXIF segment is ignored
//!   (ancillary data never kills a decode).
//!
//! # Fidelity
//!
//! The IDCT is the classic 13-bit fixed-point "islow" AAN derivation,
//! chroma upsampling is triangle ("fancy") interpolation, and the
//! YCbCr matrix uses the standard 16-bit fixed-point constants — the
//! same arithmetic family as libjpeg, so outputs sit within ±1 of the
//! ecosystem's decoders while remaining fully deterministic here.
//!
//! # The untrusted-input posture (§16.5, R14)
//!
//! Pixel budgets are checked at SOF before any entropy decode or
//! plane allocation; segment lengths are bounds-checked; Huffman
//! tables are validated; every decode loop is bounded by the marker
//! segment or the MCU count, so truncated or hostile streams produce
//! typed refusals, never hangs or overallocation.

/// Typed refusals of the JPEG decoder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JpegError {
    /// Missing SOI, or the stream is not a JPEG.
    NotJpeg,
    /// The stream ended mid-segment.
    Truncated,
    /// A segment length contradicts its content.
    BadSegment(&'static str),
    /// The image exceeds the declared pixel budget.
    TooLarge {
        /// The configured pixel budget.
        max_pixels: u64,
    },
    /// SOF declares an unsupported mode (arithmetic, lossless,
    /// hierarchical).
    UnsupportedMode(&'static str),
    /// 4-component (CMYK/YCCK) input — rejected by documented policy.
    CmykUnsupported,
    /// Sampling factors outside the supported 1–2 range.
    UnsupportedSampling,
    /// A malformed or missing Huffman/quantization table reference.
    BadTable(&'static str),
    /// A malformed scan header.
    BadScan(&'static str),
    /// The entropy stream ran dry or decoded an impossible symbol.
    CorruptEntropy(&'static str),
    /// A restart marker was expected and not found.
    BadRestart,
}

impl std::fmt::Display for JpegError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotJpeg => write!(f, "not a JPEG (missing SOI)"),
            Self::Truncated => write!(f, "jpeg stream truncated"),
            Self::BadSegment(what) => write!(f, "malformed segment: {what}"),
            Self::TooLarge { max_pixels } => {
                write!(f, "jpeg exceeds the {max_pixels}-pixel budget")
            }
            Self::UnsupportedMode(what) => write!(f, "unsupported jpeg mode: {what}"),
            Self::CmykUnsupported => write!(
                f,
                "4-component (CMYK/YCCK) jpeg rejected by policy: convert the asset to RGB"
            ),
            Self::UnsupportedSampling => {
                write!(f, "sampling factors above 2 are not supported")
            }
            Self::BadTable(what) => write!(f, "bad table: {what}"),
            Self::BadScan(what) => write!(f, "bad scan header: {what}"),
            Self::CorruptEntropy(what) => write!(f, "corrupt entropy stream: {what}"),
            Self::BadRestart => write!(f, "missing or wrong restart marker"),
        }
    }
}

impl std::error::Error for JpegError {}

/// Decode resource budgets.
#[derive(Debug, Clone)]
pub struct JpegLimits {
    /// Maximum `width × height` in pixels.
    pub max_pixels: u64,
}

impl Default for JpegLimits {
    fn default() -> Self {
        Self {
            max_pixels: 1 << 28,
        }
    }
}

/// A decoded JPEG, normalized to RGBA8 (alpha 255), already in output
/// orientation (EXIF applied).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedJpeg {
    /// Width in pixels, after orientation.
    pub width: u32,
    /// Height in pixels, after orientation.
    pub height: u32,
    /// `width × height × 4` bytes, RGBA, row-major.
    pub rgba: Vec<u8>,
    /// Source component count (1 or 3).
    pub components: u8,
    /// Whether the source was progressive (SOF2).
    pub progressive: bool,
    /// The EXIF orientation that was applied (1 if none present).
    pub orientation: u8,
}

const ZIGZAG: [usize; 64] = [
    0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5, 12, 19, 26, 33, 40, 48, 41, 34, 27, 20,
    13, 6, 7, 14, 21, 28, 35, 42, 49, 56, 57, 50, 43, 36, 29, 22, 15, 23, 30, 37, 44, 51, 58, 59,
    52, 45, 38, 31, 39, 46, 53, 60, 61, 54, 47, 55, 62, 63,
];

/// A JPEG canonical Huffman table (MSB-first).
struct Huffman {
    mincode: [i32; 17],
    maxcode: [i32; 17],
    valptr: [usize; 17],
    symbols: Vec<u8>,
}

impl Huffman {
    fn new(counts: &[u8; 16], symbols: Vec<u8>) -> Result<Self, JpegError> {
        let total: usize = counts.iter().map(|&c| usize::from(c)).sum();
        if total != symbols.len() || total > 256 {
            return Err(JpegError::BadTable("huffman symbol count"));
        }
        let mut mincode = [0i32; 17];
        let mut maxcode = [-1i32; 17];
        let mut valptr = [0usize; 17];
        let mut code = 0i32;
        let mut k = 0usize;
        for len in 1..=16 {
            let count = i32::from(counts[len - 1]);
            mincode[len] = code;
            valptr[len] = k;
            if count > 0 {
                k += count as usize;
                code += count;
                maxcode[len] = code - 1;
            }
            if code > (1 << len) {
                return Err(JpegError::BadTable("huffman over-subscribed"));
            }
            code <<= 1;
        }
        Ok(Self {
            mincode,
            maxcode,
            valptr,
            symbols,
        })
    }
}

/// MSB-first entropy reader with 0xFF00 byte-stuffing; a real marker
/// ends the readable region.
struct ScanReader<'a> {
    data: &'a [u8],
    pos: usize,
    bits: u32,
    count: u32,
}

impl<'a> ScanReader<'a> {
    const fn new(data: &'a [u8], pos: usize) -> Self {
        Self {
            data,
            pos,
            bits: 0,
            count: 0,
        }
    }

    fn bit(&mut self) -> Result<u32, JpegError> {
        if self.count == 0 {
            let byte = *self
                .data
                .get(self.pos)
                .ok_or(JpegError::CorruptEntropy("out of data"))?;
            if byte == 0xff {
                match self.data.get(self.pos + 1) {
                    Some(0x00) => {
                        self.pos += 2;
                    }
                    _ => return Err(JpegError::CorruptEntropy("marker inside entropy data")),
                }
            } else {
                self.pos += 1;
            }
            self.bits = u32::from(byte);
            self.count = 8;
        }
        self.count -= 1;
        Ok((self.bits >> self.count) & 1)
    }

    fn take(&mut self, n: u32) -> Result<u32, JpegError> {
        let mut v = 0;
        for _ in 0..n {
            v = (v << 1) | self.bit()?;
        }
        Ok(v)
    }

    /// RECEIVE/EXTEND: a signed magnitude of `s` bits.
    fn receive_extend(&mut self, s: u32) -> Result<i32, JpegError> {
        let v = self.take(s)? as i32;
        if s > 0 && v < (1 << (s - 1)) {
            Ok(v - (1 << s) + 1)
        } else {
            Ok(v)
        }
    }

    fn decode(&mut self, table: &Huffman) -> Result<u8, JpegError> {
        let mut code = 0i32;
        for len in 1..=16usize {
            code = (code << 1) | self.bit()? as i32;
            if code <= table.maxcode[len] {
                let idx = table.valptr[len] + (code - table.mincode[len]) as usize;
                return table
                    .symbols
                    .get(idx)
                    .copied()
                    .ok_or(JpegError::CorruptEntropy("huffman index"));
            }
        }
        Err(JpegError::CorruptEntropy("invalid huffman code"))
    }

    /// Byte-align and consume the expected restart marker.
    fn restart(&mut self, n: u8) -> Result<(), JpegError> {
        self.bits = 0;
        self.count = 0;
        let m = self
            .data
            .get(self.pos..self.pos + 2)
            .ok_or(JpegError::BadRestart)?;
        if m[0] != 0xff || m[1] != 0xd0 + (n % 8) {
            return Err(JpegError::BadRestart);
        }
        self.pos += 2;
        Ok(())
    }

    /// Skip to the next real marker (after the scan's entropy data).
    fn skip_to_marker(&mut self) -> usize {
        let mut p = self.pos;
        while p + 1 < self.data.len() {
            if self.data[p] == 0xff && !matches!(self.data[p + 1], 0x00 | 0xd0..=0xd7) {
                return p;
            }
            p += 1;
        }
        self.data.len()
    }
}

#[derive(Clone)]
struct Component {
    id: u8,
    h: u32,
    v: u32,
    quant_id: u8,
    /// Block grid, MCU-padded.
    blocks_w: u32,
    blocks_h: u32,
    /// Raw coefficients, natural order, `blocks_w * blocks_h * 64`.
    coefs: Vec<i16>,
    /// Sample plane after IDCT, `blocks_w*8 × blocks_h*8`.
    plane: Vec<u8>,
}

struct Frame {
    width: u32,
    height: u32,
    progressive: bool,
    components: Vec<Component>,
    hmax: u32,
    vmax: u32,
    mcus_x: u32,
    mcus_y: u32,
}

/// Decode a JPEG to RGBA8 under the given budgets.
pub fn decode(data: &[u8], limits: &JpegLimits) -> Result<DecodedJpeg, JpegError> {
    if data.len() < 2 || data[0] != 0xff || data[1] != 0xd8 {
        return Err(JpegError::NotJpeg);
    }
    let mut pos = 2usize;
    let mut quant: [Option<[u16; 64]>; 4] = [None, None, None, None];
    let mut dc_tables: [Option<Huffman>; 4] = [None, None, None, None];
    let mut ac_tables: [Option<Huffman>; 4] = [None, None, None, None];
    let mut restart_interval = 0u32;
    let mut frame: Option<Frame> = None;
    let mut orientation = 1u8;
    let mut adobe_transform: Option<u8> = None;
    let mut eobrun = 0u32;

    loop {
        // Find the next marker (skip fill bytes).
        while pos < data.len() && data[pos] != 0xff {
            pos += 1;
        }
        while pos < data.len() && data[pos] == 0xff {
            pos += 1;
        }
        if pos >= data.len() {
            return Err(JpegError::Truncated);
        }
        let marker = data[pos];
        pos += 1;

        match marker {
            0xd9 => break,           // EOI
            0x01 | 0xd0..=0xd7 => {} // standalone
            0xc0..=0xc2 => {
                let seg = segment(data, &mut pos)?;
                frame = Some(parse_sof(seg, marker == 0xc2, limits)?);
            }
            0xc3 | 0xc5..=0xc7 | 0xc9..=0xcf => {
                return Err(JpegError::UnsupportedMode(
                    "arithmetic, lossless, or hierarchical SOF",
                ));
            }
            0xc4 => {
                let mut seg = segment(data, &mut pos)?;
                while !seg.is_empty() {
                    let tc_th = seg[0];
                    let (class, id) = (tc_th >> 4, tc_th & 0x0f);
                    if class > 1 || id > 3 || seg.len() < 17 {
                        return Err(JpegError::BadTable("DHT header"));
                    }
                    let mut counts = [0u8; 16];
                    counts.copy_from_slice(&seg[1..17]);
                    let total: usize = counts.iter().map(|&c| usize::from(c)).sum();
                    if seg.len() < 17 + total {
                        return Err(JpegError::BadTable("DHT truncated"));
                    }
                    let table = Huffman::new(&counts, seg[17..17 + total].to_vec())?;
                    if class == 0 {
                        dc_tables[usize::from(id)] = Some(table);
                    } else {
                        ac_tables[usize::from(id)] = Some(table);
                    }
                    seg = &seg[17 + total..];
                }
            }
            0xdb => {
                let mut seg = segment(data, &mut pos)?;
                while !seg.is_empty() {
                    let pq_tq = seg[0];
                    let (precision, id) = (pq_tq >> 4, pq_tq & 0x0f);
                    if precision > 1 || id > 3 {
                        return Err(JpegError::BadTable("DQT header"));
                    }
                    let n = if precision == 1 { 129 } else { 65 };
                    if seg.len() < n {
                        return Err(JpegError::BadTable("DQT truncated"));
                    }
                    let mut table = [0u16; 64];
                    for i in 0..64 {
                        let raw = if precision == 1 {
                            u16::from_be_bytes([seg[1 + i * 2], seg[2 + i * 2]])
                        } else {
                            u16::from(seg[1 + i])
                        };
                        table[ZIGZAG[i]] = raw;
                    }
                    quant[usize::from(id)] = Some(table);
                    seg = &seg[n..];
                }
            }
            0xdd => {
                let seg = segment(data, &mut pos)?;
                if seg.len() != 2 {
                    return Err(JpegError::BadSegment("DRI length"));
                }
                restart_interval = u32::from(u16::from_be_bytes([seg[0], seg[1]]));
            }
            0xda => {
                let f = frame.as_mut().ok_or(JpegError::BadScan("SOS before SOF"))?;
                let seg_start = pos;
                let seg = segment(data, &mut pos)?;
                let entropy_start = seg_start + 2 + seg.len();
                let next = decode_scan(
                    data,
                    entropy_start,
                    seg,
                    f,
                    &quant,
                    &dc_tables,
                    &ac_tables,
                    restart_interval,
                    &mut eobrun,
                )?;
                pos = next;
            }
            0xe1 => {
                let seg = segment(data, &mut pos)?;
                if let Some(o) = parse_exif_orientation(seg) {
                    orientation = o;
                }
            }
            0xee => {
                let seg = segment(data, &mut pos)?;
                if seg.len() >= 12 && &seg[..5] == b"Adobe" {
                    adobe_transform = Some(seg[11]);
                }
            }
            0xe0 | 0xe2..=0xed | 0xef | 0xfe => {
                segment(data, &mut pos)?;
            }
            0xdc => {
                segment(data, &mut pos)?; // DNL — tolerated, ignored
            }
            _ => return Err(JpegError::BadSegment("unknown marker")),
        }
    }

    let mut f = frame.ok_or(JpegError::Truncated)?;

    // IDCT every component to its sample plane.
    for comp in &mut f.components {
        let qt = quant[usize::from(comp.quant_id)]
            .as_ref()
            .ok_or(JpegError::BadTable("missing quantization table"))?;
        idct_component(comp, qt);
    }

    let rgb_direct = f.components.len() == 3 && adobe_transform == Some(0);
    let rgba = to_rgba(&f, rgb_direct);
    let (width, height, rgba) = apply_orientation(f.width, f.height, rgba, orientation);

    Ok(DecodedJpeg {
        width,
        height,
        rgba,
        components: f.components.len() as u8,
        progressive: f.progressive,
        orientation,
    })
}

/// Read one length-prefixed segment; advances `pos` past it.
fn segment<'a>(data: &'a [u8], pos: &mut usize) -> Result<&'a [u8], JpegError> {
    let header = data.get(*pos..*pos + 2).ok_or(JpegError::Truncated)?;
    let len = usize::from(u16::from_be_bytes([header[0], header[1]]));
    if len < 2 {
        return Err(JpegError::BadSegment("length below 2"));
    }
    let body = data.get(*pos + 2..*pos + len).ok_or(JpegError::Truncated)?;
    *pos += len;
    Ok(body)
}

fn parse_sof(seg: &[u8], progressive: bool, limits: &JpegLimits) -> Result<Frame, JpegError> {
    if seg.len() < 6 {
        return Err(JpegError::BadSegment("SOF length"));
    }
    if seg[0] != 8 {
        return Err(JpegError::UnsupportedMode("sample precision other than 8"));
    }
    let height = u32::from(u16::from_be_bytes([seg[1], seg[2]]));
    let width = u32::from(u16::from_be_bytes([seg[3], seg[4]]));
    let ncomp = usize::from(seg[5]);
    if width == 0 || height == 0 {
        return Err(JpegError::BadSegment("zero dimension"));
    }
    if u64::from(width) * u64::from(height) > limits.max_pixels {
        return Err(JpegError::TooLarge {
            max_pixels: limits.max_pixels,
        });
    }
    if ncomp == 4 {
        return Err(JpegError::CmykUnsupported);
    }
    if ncomp != 1 && ncomp != 3 {
        return Err(JpegError::BadSegment("component count"));
    }
    if seg.len() < 6 + 3 * ncomp {
        return Err(JpegError::BadSegment("SOF component list"));
    }
    let mut components = Vec::with_capacity(ncomp);
    let mut hmax = 1u32;
    let mut vmax = 1u32;
    for c in 0..ncomp {
        let at = 6 + c * 3;
        let h = u32::from(seg[at + 1] >> 4);
        let v = u32::from(seg[at + 1] & 0x0f);
        if h == 0 || v == 0 || h > 2 || v > 2 {
            return Err(JpegError::UnsupportedSampling);
        }
        hmax = hmax.max(h);
        vmax = vmax.max(v);
        let quant_id = seg[at + 2] & 0x0f;
        // The selector indexes the four-slot DQT array; an out-of-range
        // one is a malformed frame — a typed refusal, never an
        // out-of-bounds read at IDCT time.
        if quant_id > 3 {
            return Err(JpegError::BadTable("frame quantization selector"));
        }
        components.push(Component {
            id: seg[at],
            h,
            v,
            quant_id,
            blocks_w: 0,
            blocks_h: 0,
            coefs: Vec::new(),
            plane: Vec::new(),
        });
    }
    if ncomp == 1 {
        // A single component is never subsampled relative to itself.
        components[0].h = 1;
        components[0].v = 1;
        hmax = 1;
        vmax = 1;
    }
    let mcus_x = width.div_ceil(8 * hmax);
    let mcus_y = height.div_ceil(8 * vmax);
    for comp in &mut components {
        comp.blocks_w = mcus_x * comp.h;
        comp.blocks_h = mcus_y * comp.v;
        comp.coefs = vec![0i16; comp.blocks_w as usize * comp.blocks_h as usize * 64];
    }
    Ok(Frame {
        width,
        height,
        progressive,
        components,
        hmax,
        vmax,
        mcus_x,
        mcus_y,
    })
}

/// One scan's parameters per component.
struct ScanComp {
    comp_index: usize,
    dc_id: usize,
    ac_id: usize,
    pred: i32,
}

#[allow(clippy::too_many_arguments)]
fn decode_scan(
    data: &[u8],
    entropy_start: usize,
    seg: &[u8],
    frame: &mut Frame,
    quant: &[Option<[u16; 64]>; 4],
    dc_tables: &[Option<Huffman>; 4],
    ac_tables: &[Option<Huffman>; 4],
    restart_interval: u32,
    eobrun: &mut u32,
) -> Result<usize, JpegError> {
    let _ = quant;
    if seg.is_empty() {
        return Err(JpegError::BadScan("empty SOS"));
    }
    let ns = usize::from(seg[0]);
    if ns == 0 || ns > frame.components.len() || seg.len() < 1 + 2 * ns + 3 {
        return Err(JpegError::BadScan("SOS component list"));
    }
    let mut scomps = Vec::with_capacity(ns);
    for s in 0..ns {
        let id = seg[1 + s * 2];
        let tables = seg[2 + s * 2];
        let comp_index = frame
            .components
            .iter()
            .position(|c| c.id == id)
            .ok_or(JpegError::BadScan("unknown component id"))?;
        let (dc_id, ac_id) = (usize::from(tables >> 4), usize::from(tables & 0x0f));
        // The selectors index the four-slot DHT arrays; DHT validates its
        // own ids, so an out-of-range selector here is a malformed scan —
        // a typed refusal, never an out-of-bounds read downstream.
        if dc_id > 3 || ac_id > 3 {
            return Err(JpegError::BadTable("scan table selector"));
        }
        scomps.push(ScanComp {
            comp_index,
            dc_id,
            ac_id,
            pred: 0,
        });
    }
    let ss = u32::from(seg[1 + 2 * ns]);
    let se = u32::from(seg[2 + 2 * ns]);
    let ah = u32::from(seg[3 + 2 * ns] >> 4);
    let al = u32::from(seg[3 + 2 * ns] & 0x0f);
    if frame.progressive {
        if ss > 63 || se > 63 || ss > se || (ss == 0 && se != 0 && ns > 1) {
            return Err(JpegError::BadScan("spectral selection"));
        }
        if ss > 0 && ns != 1 {
            return Err(JpegError::BadScan("interleaved AC scan"));
        }
    } else if ss != 0 || se != 63 || ah != 0 || al != 0 {
        return Err(JpegError::BadScan("sequential scan parameters"));
    }
    // A fresh scan resets the EOB run.
    *eobrun = 0;

    let mut reader = ScanReader::new(data, entropy_start);
    let interleaved = ns > 1;

    // MCU geometry for this scan.
    let (mcus_x, mcus_y) = if interleaved {
        (frame.mcus_x, frame.mcus_y)
    } else {
        // Non-interleaved scans cover the component's own block grid,
        // unpadded (ceil of its scaled dimensions).
        let c = &frame.components[scomps[0].comp_index];
        let bw = (frame.width * c.h).div_ceil(8 * frame.hmax);
        let bh = (frame.height * c.v).div_ceil(8 * frame.vmax);
        (bw, bh)
    };

    let total_mcus = u64::from(mcus_x) * u64::from(mcus_y);
    let mut restart_count = 0u8;
    let mut since_restart = 0u32;

    for mcu in 0..total_mcus {
        if restart_interval > 0 && since_restart == restart_interval {
            reader.restart(restart_count)?;
            restart_count = restart_count.wrapping_add(1);
            since_restart = 0;
            for sc in &mut scomps {
                sc.pred = 0;
            }
            *eobrun = 0;
        }
        since_restart += 1;

        let mcu_x = (mcu % u64::from(mcus_x)) as u32;
        let mcu_y = (mcu / u64::from(mcus_x)) as u32;

        if interleaved {
            for sc_i in 0..scomps.len() {
                let comp_index = scomps[sc_i].comp_index;
                let (h, v, blocks_w) = {
                    let c = &frame.components[comp_index];
                    (c.h, c.v, c.blocks_w)
                };
                for by in 0..v {
                    for bx in 0..h {
                        let block_x = mcu_x * h + bx;
                        let block_y = mcu_y * v + by;
                        let at = (block_y * blocks_w + block_x) as usize * 64;
                        decode_block(
                            &mut reader,
                            frame,
                            &mut scomps,
                            sc_i,
                            at,
                            ss,
                            se,
                            ah,
                            al,
                            dc_tables,
                            ac_tables,
                            eobrun,
                        )?;
                    }
                }
            }
        } else {
            let comp_index = scomps[0].comp_index;
            let blocks_w = frame.components[comp_index].blocks_w;
            let at = (mcu_y * blocks_w + mcu_x) as usize * 64;
            decode_block(
                &mut reader,
                frame,
                &mut scomps,
                0,
                at,
                ss,
                se,
                ah,
                al,
                dc_tables,
                ac_tables,
                eobrun,
            )?;
        }
    }

    Ok(reader.skip_to_marker())
}

/// Decode (or refine) one block's band into the component's
/// coefficient store.
#[allow(clippy::too_many_arguments)]
fn decode_block(
    reader: &mut ScanReader<'_>,
    frame: &mut Frame,
    scomps: &mut [ScanComp],
    sc_i: usize,
    at: usize,
    ss: u32,
    se: u32,
    ah: u32,
    al: u32,
    dc_tables: &[Option<Huffman>; 4],
    ac_tables: &[Option<Huffman>; 4],
    eobrun: &mut u32,
) -> Result<(), JpegError> {
    let comp_index = scomps[sc_i].comp_index;
    let progressive = frame.progressive;
    let coefs = &mut frame.components[comp_index].coefs;
    let block = &mut coefs[at..at + 64];

    if !progressive {
        // Sequential: DC + all AC in one pass.
        let dc = dc_tables[scomps[sc_i].dc_id]
            .as_ref()
            .ok_or(JpegError::BadTable("missing DC table"))?;
        let ac = ac_tables[scomps[sc_i].ac_id]
            .as_ref()
            .ok_or(JpegError::BadTable("missing AC table"))?;
        let t = u32::from(reader.decode(dc)?);
        if t > 15 {
            return Err(JpegError::CorruptEntropy("DC magnitude"));
        }
        let diff = if t > 0 { reader.receive_extend(t)? } else { 0 };
        scomps[sc_i].pred = scomps[sc_i]
            .pred
            .checked_add(diff)
            .ok_or(JpegError::CorruptEntropy("DC overflow"))?;
        block[0] = clamp_coef(scomps[sc_i].pred);
        let mut k = 1usize;
        while k < 64 {
            let rs = reader.decode(ac)?;
            let r = usize::from(rs >> 4);
            let s = u32::from(rs & 0x0f);
            if s == 0 {
                if r == 15 {
                    k += 16;
                    continue;
                }
                break; // EOB
            }
            k += r;
            if k > 63 {
                return Err(JpegError::CorruptEntropy("AC run past block"));
            }
            block[ZIGZAG[k]] = clamp_coef(reader.receive_extend(s)?);
            k += 1;
        }
        return Ok(());
    }

    if ss == 0 {
        // Progressive DC.
        if ah == 0 {
            let dc = dc_tables[scomps[sc_i].dc_id]
                .as_ref()
                .ok_or(JpegError::BadTable("missing DC table"))?;
            let t = u32::from(reader.decode(dc)?);
            if t > 15 {
                return Err(JpegError::CorruptEntropy("DC magnitude"));
            }
            let diff = if t > 0 { reader.receive_extend(t)? } else { 0 };
            scomps[sc_i].pred = scomps[sc_i]
                .pred
                .checked_add(diff)
                .ok_or(JpegError::CorruptEntropy("DC overflow"))?;
            block[0] = clamp_coef(scomps[sc_i].pred << al);
        } else if reader.bit()? == 1 {
            block[0] |= 1 << al;
        }
        return Ok(());
    }

    // Progressive AC.
    let ac = ac_tables[scomps[sc_i].ac_id]
        .as_ref()
        .ok_or(JpegError::BadTable("missing AC table"))?;
    if ah == 0 {
        // First pass for this band.
        if *eobrun > 0 {
            *eobrun -= 1;
            return Ok(());
        }
        let mut k = ss as usize;
        while k <= se as usize {
            let rs = reader.decode(ac)?;
            let r = u32::from(rs >> 4);
            let s = u32::from(rs & 0x0f);
            if s == 0 {
                if r < 15 {
                    *eobrun = (1 << r) - 1;
                    if r > 0 {
                        *eobrun += reader.take(r)?;
                    }
                    break;
                }
                k += 16;
                continue;
            }
            k += r as usize;
            if k > se as usize {
                return Err(JpegError::CorruptEntropy("AC run past band"));
            }
            block[ZIGZAG[k]] = clamp_coef(reader.receive_extend(s)? << al);
            k += 1;
        }
        return Ok(());
    }

    // AC refinement (the libjpeg algorithm, bounds-checked).
    let p1 = 1i32 << al;
    let m1 = -1i32 << al;
    let mut k = ss as usize;
    if *eobrun == 0 {
        while k <= se as usize {
            let rs = reader.decode(ac)?;
            let mut r = u32::from(rs >> 4);
            let s = u32::from(rs & 0x0f);
            let mut newval = 0i32;
            if s != 0 {
                if s != 1 {
                    return Err(JpegError::CorruptEntropy("AC refine magnitude"));
                }
                newval = if reader.bit()? == 1 { p1 } else { m1 };
            } else if r != 15 {
                *eobrun = 1 << r;
                if r > 0 {
                    *eobrun += reader.take(r)?;
                }
                break;
            }
            while k <= se as usize {
                let idx = ZIGZAG[k];
                if block[idx] != 0 {
                    if reader.bit()? == 1 && (i32::from(block[idx]) & p1) == 0 {
                        let delta = if block[idx] >= 0 { p1 } else { m1 };
                        block[idx] = clamp_coef(i32::from(block[idx]) + delta);
                    }
                } else {
                    if r == 0 {
                        break;
                    }
                    r -= 1;
                }
                k += 1;
            }
            if newval != 0 {
                if k > se as usize {
                    return Err(JpegError::CorruptEntropy("AC refine position"));
                }
                block[ZIGZAG[k]] = clamp_coef(newval);
            }
            k += 1;
        }
    }
    if *eobrun > 0 {
        while k <= se as usize {
            let idx = ZIGZAG[k];
            if block[idx] != 0 && reader.bit()? == 1 && (i32::from(block[idx]) & p1) == 0 {
                let delta = if block[idx] >= 0 { p1 } else { m1 };
                block[idx] = clamp_coef(i32::from(block[idx]) + delta);
            }
            k += 1;
        }
        *eobrun -= 1;
    }
    Ok(())
}

fn clamp_coef(v: i32) -> i16 {
    v.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16
}

// ---- IDCT (13-bit fixed point, the classic "islow" derivation) -----

const FIX_0_298631336: i64 = 2446;
const FIX_0_390180644: i64 = 3196;
const FIX_0_541196100: i64 = 4433;
const FIX_0_765366865: i64 = 6270;
const FIX_0_899976223: i64 = 7373;
const FIX_1_175875602: i64 = 9633;
const FIX_1_501321110: i64 = 12299;
const FIX_1_847759065: i64 = 15137;
const FIX_1_961570560: i64 = 16069;
const FIX_2_053119869: i64 = 16819;
const FIX_2_562915447: i64 = 20995;
const FIX_3_072711026: i64 = 25172;
const CONST_BITS: i64 = 13;
const PASS1_BITS: i64 = 2;

fn descale(x: i64, n: i64) -> i64 {
    (x + (1 << (n - 1))) >> n
}

fn range_limit(x: i64) -> u8 {
    (x + 128).clamp(0, 255) as u8
}

/// Inverse DCT of one 8×8 block into `out` rows of `stride`.
fn idct_block(coefs: &[i16], qt: &[u16; 64], out: &mut [u8], stride: usize) {
    let mut ws = [0i64; 64];

    // Pass 1: columns.
    for c in 0..8 {
        let col = |r: usize| i64::from(coefs[r * 8 + c]) * i64::from(qt[r * 8 + c]);
        if (1..8).all(|r| coefs[r * 8 + c] == 0) {
            let dc = col(0) << PASS1_BITS;
            for r in 0..8 {
                ws[r * 8 + c] = dc;
            }
            continue;
        }
        let z2 = col(2);
        let z3 = col(6);
        let z1 = (z2 + z3) * FIX_0_541196100;
        let tmp2 = z1 - z3 * FIX_1_847759065;
        let tmp3 = z1 + z2 * FIX_0_765366865;
        let z2 = col(0);
        let z3 = col(4);
        let tmp0 = (z2 + z3) << CONST_BITS;
        let tmp1 = (z2 - z3) << CONST_BITS;
        let tmp10 = tmp0 + tmp3;
        let tmp13 = tmp0 - tmp3;
        let tmp11 = tmp1 + tmp2;
        let tmp12 = tmp1 - tmp2;
        let t0 = col(7);
        let t1 = col(5);
        let t2 = col(3);
        let t3 = col(1);
        let z1 = t0 + t3;
        let z2 = t1 + t2;
        let z3 = t0 + t2;
        let z4 = t1 + t3;
        let z5 = (z3 + z4) * FIX_1_175875602;
        let t0 = t0 * FIX_0_298631336;
        let t1 = t1 * FIX_2_053119869;
        let t2 = t2 * FIX_3_072711026;
        let t3 = t3 * FIX_1_501321110;
        let z1 = -z1 * FIX_0_899976223;
        let z2 = -z2 * FIX_2_562915447;
        let z3 = -z3 * FIX_1_961570560 + z5;
        let z4 = -z4 * FIX_0_390180644 + z5;
        let t0 = t0 + z1 + z3;
        let t1 = t1 + z2 + z4;
        let t2 = t2 + z2 + z3;
        let t3 = t3 + z1 + z4;
        ws[c] = descale(tmp10 + t3, CONST_BITS - PASS1_BITS);
        ws[56 + c] = descale(tmp10 - t3, CONST_BITS - PASS1_BITS);
        ws[8 + c] = descale(tmp11 + t2, CONST_BITS - PASS1_BITS);
        ws[48 + c] = descale(tmp11 - t2, CONST_BITS - PASS1_BITS);
        ws[16 + c] = descale(tmp12 + t1, CONST_BITS - PASS1_BITS);
        ws[40 + c] = descale(tmp12 - t1, CONST_BITS - PASS1_BITS);
        ws[24 + c] = descale(tmp13 + t0, CONST_BITS - PASS1_BITS);
        ws[32 + c] = descale(tmp13 - t0, CONST_BITS - PASS1_BITS);
    }

    // Pass 2: rows.
    for r in 0..8 {
        let row = &ws[r * 8..r * 8 + 8];
        let out_row = &mut out[r * stride..r * stride + 8];
        if row[1..].iter().all(|&x| x == 0) {
            let dc = range_limit(descale(row[0], PASS1_BITS + 3));
            out_row.fill(dc);
            continue;
        }
        let z2 = row[2];
        let z3 = row[6];
        let z1 = (z2 + z3) * FIX_0_541196100;
        let tmp2 = z1 - z3 * FIX_1_847759065;
        let tmp3 = z1 + z2 * FIX_0_765366865;
        let tmp0 = (row[0] + row[4]) << CONST_BITS;
        let tmp1 = (row[0] - row[4]) << CONST_BITS;
        let tmp10 = tmp0 + tmp3;
        let tmp13 = tmp0 - tmp3;
        let tmp11 = tmp1 + tmp2;
        let tmp12 = tmp1 - tmp2;
        let t0 = row[7];
        let t1 = row[5];
        let t2 = row[3];
        let t3 = row[1];
        let z1 = t0 + t3;
        let z2 = t1 + t2;
        let z3 = t0 + t2;
        let z4 = t1 + t3;
        let z5 = (z3 + z4) * FIX_1_175875602;
        let t0 = t0 * FIX_0_298631336;
        let t1 = t1 * FIX_2_053119869;
        let t2 = t2 * FIX_3_072711026;
        let t3 = t3 * FIX_1_501321110;
        let z1 = -z1 * FIX_0_899976223;
        let z2 = -z2 * FIX_2_562915447;
        let z3 = -z3 * FIX_1_961570560 + z5;
        let z4 = -z4 * FIX_0_390180644 + z5;
        let t0 = t0 + z1 + z3;
        let t1 = t1 + z2 + z4;
        let t2 = t2 + z2 + z3;
        let t3 = t3 + z1 + z4;
        let shift = CONST_BITS + PASS1_BITS + 3;
        out_row[0] = range_limit(descale(tmp10 + t3, shift));
        out_row[7] = range_limit(descale(tmp10 - t3, shift));
        out_row[1] = range_limit(descale(tmp11 + t2, shift));
        out_row[6] = range_limit(descale(tmp11 - t2, shift));
        out_row[2] = range_limit(descale(tmp12 + t1, shift));
        out_row[5] = range_limit(descale(tmp12 - t1, shift));
        out_row[3] = range_limit(descale(tmp13 + t0, shift));
        out_row[4] = range_limit(descale(tmp13 - t0, shift));
    }
}

fn idct_component(comp: &mut Component, qt: &[u16; 64]) {
    let stride = comp.blocks_w as usize * 8;
    comp.plane = vec![0u8; stride * comp.blocks_h as usize * 8];
    for by in 0..comp.blocks_h as usize {
        for bx in 0..comp.blocks_w as usize {
            let at = (by * comp.blocks_w as usize + bx) * 64;
            let out_at = by * 8 * stride + bx * 8;
            idct_block(
                &comp.coefs[at..at + 64],
                qt,
                &mut comp.plane[out_at..],
                stride,
            );
        }
    }
}

// ---- upsampling + color conversion ---------------------------------

/// Sample a component plane at full-image pixel (x, y) with triangle
/// ("fancy") interpolation for 2× subsampled axes.
///
/// The plane holds `blocks_w*8 × blocks_h*8` samples covering the
/// component's scaled image `cw × ch`.
struct Sampler<'a> {
    plane: &'a [u8],
    stride: usize,
    cw: usize,
    ch: usize,
    hshift: bool,
    vshift: bool,
}

impl Sampler<'_> {
    fn at(&self, sx: usize, sy: usize) -> i32 {
        let x = sx.min(self.cw - 1);
        let y = sy.min(self.ch - 1);
        i32::from(self.plane[y * self.stride + x])
    }

    /// Triangle-filtered sample for output pixel (x, y).
    fn sample(&self, x: usize, y: usize) -> i32 {
        match (self.hshift, self.vshift) {
            (false, false) => self.at(x, y),
            (true, false) => {
                let sx = x / 2;
                let even = x.is_multiple_of(2);
                let near = self.at(sx, y);
                let far = if even {
                    self.at(sx.saturating_sub(1), y)
                } else {
                    self.at(sx + 1, y)
                };
                (3 * near + far + if even { 1 } else { 2 }) >> 2
            }
            (false, true) => {
                let sy = y / 2;
                let even = y.is_multiple_of(2);
                let near = self.at(x, sy);
                let far = if even {
                    self.at(x, sy.saturating_sub(1))
                } else {
                    self.at(x, sy + 1)
                };
                (3 * near + far + if even { 1 } else { 2 }) >> 2
            }
            (true, true) => {
                let sx = x / 2;
                let sy = y / 2;
                let x_even = x.is_multiple_of(2);
                let fx = if x_even { sx.saturating_sub(1) } else { sx + 1 };
                let fy = if y.is_multiple_of(2) {
                    sy.saturating_sub(1)
                } else {
                    sy + 1
                };
                // 2D triangle: (9·near + 3·hfar + 3·vfar + diag) / 16,
                // with the libjpeg bias pattern (8 even / 7 odd cols).
                let colsum_near = 3 * self.at(sx, sy) + self.at(sx, fy);
                let colsum_far = 3 * self.at(fx, sy) + self.at(fx, fy);
                let bias = if x_even { 8 } else { 7 };
                (3 * colsum_near + colsum_far + bias) >> 4
            }
        }
    }
}

fn make_sampler<'a>(frame: &'a Frame, comp: &'a Component) -> Sampler<'a> {
    Sampler {
        plane: &comp.plane,
        stride: comp.blocks_w as usize * 8,
        cw: ((frame.width * comp.h).div_ceil(frame.hmax)) as usize,
        ch: ((frame.height * comp.v).div_ceil(frame.vmax)) as usize,
        hshift: comp.h < frame.hmax,
        vshift: comp.v < frame.vmax,
    }
}

const FIX_YCC_R_CR: i32 = 91881;
const FIX_YCC_B_CB: i32 = 116130;
const FIX_YCC_G_CB: i32 = 22554;
const FIX_YCC_G_CR: i32 = 46802;

fn to_rgba(frame: &Frame, rgb_direct: bool) -> Vec<u8> {
    let w = frame.width as usize;
    let h = frame.height as usize;
    let mut rgba = vec![255u8; w * h * 4];
    if frame.components.len() == 1 {
        let s = make_sampler(frame, &frame.components[0]);
        for y in 0..h {
            for x in 0..w {
                let g = s.sample(x, y).clamp(0, 255) as u8;
                let at = (y * w + x) * 4;
                rgba[at] = g;
                rgba[at + 1] = g;
                rgba[at + 2] = g;
            }
        }
        return rgba;
    }
    let sy = make_sampler(frame, &frame.components[0]);
    let scb = make_sampler(frame, &frame.components[1]);
    let scr = make_sampler(frame, &frame.components[2]);
    for y in 0..h {
        for x in 0..w {
            let at = (y * w + x) * 4;
            if rgb_direct {
                rgba[at] = sy.sample(x, y).clamp(0, 255) as u8;
                rgba[at + 1] = scb.sample(x, y).clamp(0, 255) as u8;
                rgba[at + 2] = scr.sample(x, y).clamp(0, 255) as u8;
                continue;
            }
            let yy = sy.sample(x, y);
            let cb = scb.sample(x, y) - 128;
            let cr = scr.sample(x, y) - 128;
            let r = yy + ((FIX_YCC_R_CR * cr + 32768) >> 16);
            let g = yy - ((FIX_YCC_G_CB * cb + FIX_YCC_G_CR * cr + 32768) >> 16);
            let b = yy + ((FIX_YCC_B_CB * cb + 32768) >> 16);
            rgba[at] = r.clamp(0, 255) as u8;
            rgba[at + 1] = g.clamp(0, 255) as u8;
            rgba[at + 2] = b.clamp(0, 255) as u8;
        }
    }
    rgba
}

// ---- EXIF orientation ----------------------------------------------

/// Extract EXIF orientation (1–8) from an APP1 payload, or `None`.
/// Malformed EXIF is ignored, never an error.
fn parse_exif_orientation(seg: &[u8]) -> Option<u8> {
    let tiff = seg.strip_prefix(b"Exif\0\0")?;
    if tiff.len() < 8 {
        return None;
    }
    let le = match &tiff[0..2] {
        b"II" => true,
        b"MM" => false,
        _ => return None,
    };
    let read16 = |at: usize| -> Option<u16> {
        let b = tiff.get(at..at + 2)?;
        Some(if le {
            u16::from_le_bytes([b[0], b[1]])
        } else {
            u16::from_be_bytes([b[0], b[1]])
        })
    };
    let read32 = |at: usize| -> Option<u32> {
        let b = tiff.get(at..at + 4)?;
        Some(if le {
            u32::from_le_bytes([b[0], b[1], b[2], b[3]])
        } else {
            u32::from_be_bytes([b[0], b[1], b[2], b[3]])
        })
    };
    if read16(2)? != 42 {
        return None;
    }
    let ifd = read32(4)? as usize;
    let count = usize::from(read16(ifd)?);
    for i in 0..count.min(256) {
        let entry = ifd + 2 + i * 12;
        if read16(entry)? == 0x0112 {
            let value = read16(entry + 8)?;
            if (1..=8).contains(&value) {
                return Some(value as u8);
            }
            return None;
        }
    }
    None
}

/// Apply an EXIF orientation to RGBA pixels; returns (w, h, pixels)
/// in output orientation.
fn apply_orientation(w: u32, h: u32, rgba: Vec<u8>, orientation: u8) -> (u32, u32, Vec<u8>) {
    if orientation <= 1 {
        return (w, h, rgba);
    }
    let (w, h) = (w as usize, h as usize);
    let swapped = orientation >= 5;
    let (ow, oh) = if swapped { (h, w) } else { (w, h) };
    let mut out = vec![0u8; rgba.len()];
    for oy in 0..oh {
        for ox in 0..ow {
            let (sx, sy) = match orientation {
                2 => (w - 1 - ox, oy),
                3 => (w - 1 - ox, h - 1 - oy),
                4 => (ox, h - 1 - oy),
                5 => (oy, ox),
                6 => (oy, h - 1 - ox),
                7 => (w - 1 - oy, h - 1 - ox),
                _ => (w - 1 - oy, ox), // 8
            };
            let src = (sy * w + sx) * 4;
            let dst = (oy * ow + ox) * 4;
            out[dst..dst + 4].copy_from_slice(&rgba[src..src + 4]);
        }
    }
    (ow as u32, oh as u32, out)
}
