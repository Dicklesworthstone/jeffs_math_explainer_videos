//! WAV files: stems are written as 32-bit float stereo (lossless, no dither);
//! s24 and s16 (with TPDF dither) are available for other consumers. The reader
//! takes the sample formats franken_manim's decoder takes (PCM u8/s16/s24/s32,
//! IEEE f32, including WAVE_FORMAT_EXTENSIBLE) but is deliberately stricter:
//! mono or stereo only, whole frames only, finite floats only, a complete
//! EXTENSIBLE header. It reads narration WAVs for ducking and QA; a file it
//! refuses is a file worth looking at.

use std::fs::File;
use std::io::{self, BufWriter, Read, Write};
use std::path::Path;

use crate::rng::Rng;
use crate::{SAMPLE_RATE, Stereo};

/// franken_manim refuses cue files above this size (`MAX_CUE_ASSET_BYTES`).
pub const MAX_CUE_BYTES: u64 = 64 * 1024 * 1024;

/// Output sample format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    F32,
    S24,
    /// 16-bit with TPDF dither (seeded, so output is reproducible).
    S16Tpdf {
        seed: u64,
    },
}

impl Format {
    fn bytes_per_sample(self) -> u32 {
        match self {
            Format::F32 => 4,
            Format::S24 => 3,
            Format::S16Tpdf { .. } => 2,
        }
    }
}

/// Size in bytes of a stereo WAV of `frames` frames in `fmt` (44-byte header,
/// or 80 with the EXTENSIBLE header used for f32).
pub fn encoded_size(frames: u64, fmt: Format) -> u64 {
    let header = if fmt == Format::F32 { 80 } else { 44 };
    header + frames * 2 * u64::from(fmt.bytes_per_sample())
}

/// Write a stereo WAV at 48 kHz. Refuses files franken_manim would refuse.
pub fn write(path: &Path, audio: &Stereo, fmt: Format) -> io::Result<()> {
    let frames = audio.len() as u64;
    if encoded_size(frames, fmt) > MAX_CUE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "{} frames as {fmt:?} is {} bytes, over franken_manim's {} byte cue limit; split the stem",
                frames,
                encoded_size(frames, fmt),
                MAX_CUE_BYTES
            ),
        ));
    }
    if !audio.is_finite() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "non-finite sample in stem"));
    }
    let bps = fmt.bytes_per_sample();
    let data_len = (frames * 2 * u64::from(bps)) as u32;
    let mut w = BufWriter::new(File::create(path)?);
    let block_align = (2 * bps) as u16;
    let byte_rate = SAMPLE_RATE * u32::from(block_align);
    w.write_all(b"RIFF")?;
    if fmt == Format::F32 {
        // WAVE_FORMAT_EXTENSIBLE with the IEEE-float subformat GUID.
        w.write_all(&(72 + data_len).to_le_bytes())?;
        w.write_all(b"WAVEfmt ")?;
        w.write_all(&40u32.to_le_bytes())?;
        w.write_all(&0xFFFEu16.to_le_bytes())?;
        w.write_all(&2u16.to_le_bytes())?;
        w.write_all(&SAMPLE_RATE.to_le_bytes())?;
        w.write_all(&byte_rate.to_le_bytes())?;
        w.write_all(&block_align.to_le_bytes())?;
        w.write_all(&32u16.to_le_bytes())?;
        w.write_all(&22u16.to_le_bytes())?; // cbSize
        w.write_all(&32u16.to_le_bytes())?; // valid bits
        w.write_all(&3u32.to_le_bytes())?; // channel mask FL|FR
        w.write_all(&[
            0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x80, 0x00, 0x00, 0xAA, 0x00, 0x38, 0x9B, 0x71,
        ])?; // KSDATAFORMAT_SUBTYPE_IEEE_FLOAT
        w.write_all(b"fact")?;
        w.write_all(&4u32.to_le_bytes())?;
        w.write_all(&(frames as u32).to_le_bytes())?;
    } else {
        w.write_all(&(36 + data_len).to_le_bytes())?;
        w.write_all(b"WAVEfmt ")?;
        w.write_all(&16u32.to_le_bytes())?;
        w.write_all(&1u16.to_le_bytes())?; // PCM
        w.write_all(&2u16.to_le_bytes())?;
        w.write_all(&SAMPLE_RATE.to_le_bytes())?;
        w.write_all(&byte_rate.to_le_bytes())?;
        w.write_all(&block_align.to_le_bytes())?;
        w.write_all(&((bps * 8) as u16).to_le_bytes())?;
    }
    w.write_all(b"data")?;
    w.write_all(&data_len.to_le_bytes())?;
    let mut rng = match fmt {
        Format::S16Tpdf { seed } => Some(Rng::new(seed)),
        _ => None,
    };
    for i in 0..audio.len() {
        for &x in &[audio.l[i], audio.r[i]] {
            match fmt {
                Format::F32 => w.write_all(&(x as f32).to_le_bytes())?,
                Format::S24 => {
                    let v = (x.clamp(-1.0, 1.0) * 8_388_607.0).round() as i32;
                    w.write_all(&v.to_le_bytes()[..3])?;
                }
                Format::S16Tpdf { .. } => {
                    let r = rng.as_mut().expect("seeded");
                    // TPDF: sum of two uniform variables, ±1 LSB peak.
                    let d = r.uniform() - r.uniform();
                    let v = (x.clamp(-1.0, 1.0) * 32_767.0 + d).round().clamp(-32_768.0, 32_767.0) as i16;
                    w.write_all(&v.to_le_bytes())?;
                }
            }
        }
    }
    w.flush()
}

/// A decoded WAV: interleaved channels as f64 in [-1, 1], at its own rate.
#[derive(Clone, Debug)]
pub struct Decoded {
    pub rate: u32,
    pub channels: u16,
    /// One vector per channel.
    pub samples: Vec<Vec<f64>>,
}

impl Decoded {
    pub fn frames(&self) -> usize {
        self.samples.first().map_or(0, Vec::len)
    }
    /// Mono mix (channel average), as a voice bus needs.
    pub fn mono(&self) -> Vec<f64> {
        let n = self.frames();
        let c = self.samples.len().max(1) as f64;
        (0..n)
            .map(|i| self.samples.iter().map(|ch| ch[i]).sum::<f64>() / c)
            .collect()
    }
}

fn bad(msg: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg.into())
}

/// Read a WAV file (PCM 8/16/24/32-bit or IEEE f32; plain or EXTENSIBLE).
pub fn read(path: &Path) -> io::Result<Decoded> {
    let mut bytes = Vec::new();
    File::open(path)?.read_to_end(&mut bytes)?;
    parse(&bytes)
}

/// Parse WAV bytes (see [`read`]).
pub fn parse(bytes: &[u8]) -> io::Result<Decoded> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(bad("not a RIFF/WAVE file"));
    }
    let mut pos = 12;
    let mut fmt: Option<(u16, u16, u32, u16)> = None; // (tag, channels, rate, bits)
    let mut data: Option<&[u8]> = None;
    while pos + 8 <= bytes.len() {
        let id = &bytes[pos..pos + 4];
        let size = u32::from_le_bytes(bytes[pos + 4..pos + 8].try_into().unwrap()) as usize;
        let body_start = pos + 8;
        let body_end = body_start.checked_add(size).ok_or_else(|| bad("chunk size overflow"))?;
        if body_end > bytes.len() {
            return Err(bad(format!(
                "truncated chunk {:?}: declares {size} bytes, {} available",
                String::from_utf8_lossy(id),
                bytes.len() - body_start
            )));
        }
        let body = &bytes[body_start..body_end];
        match id {
            b"fmt " => {
                if body.len() < 16 {
                    return Err(bad("short fmt chunk"));
                }
                let mut tag = u16::from_le_bytes([body[0], body[1]]);
                let channels = u16::from_le_bytes([body[2], body[3]]);
                let rate = u32::from_le_bytes(body[4..8].try_into().unwrap());
                let bits = u16::from_le_bytes([body[14], body[15]]);
                if tag == 0xFFFE {
                    if body.len() < 40 {
                        return Err(bad("short extensible fmt chunk"));
                    }
                    tag = u16::from_le_bytes([body[24], body[25]]);
                }
                fmt = Some((tag, channels, rate, bits));
            }
            b"data" => data = Some(body),
            _ => {}
        }
        pos = body_end + (size & 1);
    }
    let (tag, channels, rate, bits) = fmt.ok_or_else(|| bad("no fmt chunk"))?;
    let data = data.ok_or_else(|| bad("no data chunk"))?;
    if !(1..=2).contains(&channels) {
        return Err(bad(format!(
            "{channels} channels; franken_manim accepts mono or stereo"
        )));
    }
    let bps = usize::from(bits / 8);
    let frame = bps * usize::from(channels);
    if frame == 0 || data.len() % frame != 0 {
        return Err(bad("data length is not a whole number of frames"));
    }
    let n = data.len() / frame;
    let mut samples = vec![Vec::with_capacity(n); usize::from(channels)];
    for f in 0..n {
        for (c, ch) in samples.iter_mut().enumerate() {
            let o = f * frame + c * bps;
            let s = &data[o..o + bps];
            let x = match (tag, bits) {
                (1, 8) => (f64::from(s[0]) - 128.0) / 128.0,
                (1, 16) => f64::from(i16::from_le_bytes([s[0], s[1]])) / 32_768.0,
                (1, 24) => f64::from(i32::from_le_bytes([0, s[0], s[1], s[2]]) >> 8) / 8_388_608.0,
                (1, 32) => f64::from(i32::from_le_bytes([s[0], s[1], s[2], s[3]])) / 2_147_483_648.0,
                (3, 32) => f64::from(f32::from_le_bytes([s[0], s[1], s[2], s[3]])),
                _ => return Err(bad(format!("unsupported WAV format tag {tag}, {bits} bits"))),
            };
            if !x.is_finite() {
                return Err(bad(format!("non-finite sample at frame {f}")));
            }
            ch.push(x);
        }
    }
    Ok(Decoded {
        rate,
        channels,
        samples,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("explainer_score_wav_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join(name)
    }

    fn ramp(n: usize) -> Stereo {
        Stereo {
            l: (0..n).map(|i| (i as f64 / n as f64) * 1.6 - 0.8).collect(),
            r: (0..n).map(|i| ((i as f64) * 0.01).sin() * 0.5).collect(),
        }
    }

    #[test]
    fn f32_round_trip_is_exact_to_f32() {
        let a = ramp(4_801);
        let p = tmp("f32.wav");
        write(&p, &a, Format::F32).unwrap();
        let d = read(&p).unwrap();
        assert_eq!((d.rate, d.channels, d.frames()), (48_000, 2, 4_801));
        for i in 0..a.len() {
            assert_eq!(d.samples[0][i], f64::from(a.l[i] as f32));
            assert_eq!(d.samples[1][i], f64::from(a.r[i] as f32));
        }
        assert_eq!(std::fs::metadata(&p).unwrap().len(), encoded_size(4_801, Format::F32));
    }

    #[test]
    fn s24_and_s16_round_trip_within_a_few_lsb() {
        let a = ramp(2_000);
        for (fmt, lsb) in [
            (Format::S24, 1.0 / 8_388_607.0),
            (Format::S16Tpdf { seed: 1 }, 1.0 / 32_767.0),
        ] {
            let p = tmp(&format!("{fmt:?}.wav").replace(['{', '}', ' ', ':'], "_"));
            write(&p, &a, fmt).unwrap();
            let d = read(&p).unwrap();
            // Write scales by 2^(b-1) - 1, read divides by 2^(b-1) (both as
            // franken_manim does), plus rounding and, for s16, ±1 LSB of TPDF.
            let tol = if matches!(fmt, Format::S16Tpdf { .. }) {
                3.0 * lsb
            } else {
                2.0 * lsb
            };
            for i in 0..a.len() {
                assert!((d.samples[0][i] - a.l[i]).abs() <= tol, "{fmt:?} {i}");
            }
        }
    }

    #[test]
    fn tpdf_dither_is_reproducible_from_its_seed() {
        let a = ramp(1_000);
        let (p1, p2) = (tmp("d1.wav"), tmp("d2.wav"));
        write(&p1, &a, Format::S16Tpdf { seed: 9 }).unwrap();
        write(&p2, &a, Format::S16Tpdf { seed: 9 }).unwrap();
        assert_eq!(std::fs::read(&p1).unwrap(), std::fs::read(&p2).unwrap());
    }

    #[test]
    fn oversize_stems_and_bad_samples_are_refused() {
        // 174.8 s of f32 stereo fits in 64 MiB; 175 s does not.
        assert!(encoded_size(48_000 * 174 + 36_000, Format::F32) <= MAX_CUE_BYTES);
        assert!(encoded_size(48_000 * 175, Format::F32) > MAX_CUE_BYTES);
        let big = Stereo::silence(48_000 * 175);
        assert!(write(&tmp("big.wav"), &big, Format::F32).is_err());
        let mut nan = Stereo::silence(10);
        nan.l[3] = f64::NAN;
        assert!(write(&tmp("nan.wav"), &nan, Format::F32).is_err());
    }

    #[test]
    fn truncated_and_foreign_files_are_refused() {
        let a = ramp(100);
        let p = tmp("trunc.wav");
        write(&p, &a, Format::S24).unwrap();
        let bytes = std::fs::read(&p).unwrap();
        assert!(parse(&bytes[..bytes.len() - 7]).is_err(), "truncated data chunk");
        assert!(parse(b"RIFF\0\0\0\0WAVX").is_err());
    }

    #[test]
    fn reads_mono_s16_like_ftts_output() {
        // 24 kHz mono s16, the narration format.
        let mut b = Vec::new();
        let n: u32 = 3;
        b.extend_from_slice(b"RIFF");
        b.extend_from_slice(&(36 + n * 2).to_le_bytes());
        b.extend_from_slice(b"WAVEfmt ");
        b.extend_from_slice(&16u32.to_le_bytes());
        b.extend_from_slice(&1u16.to_le_bytes());
        b.extend_from_slice(&1u16.to_le_bytes());
        b.extend_from_slice(&24_000u32.to_le_bytes());
        b.extend_from_slice(&48_000u32.to_le_bytes());
        b.extend_from_slice(&2u16.to_le_bytes());
        b.extend_from_slice(&16u16.to_le_bytes());
        b.extend_from_slice(b"data");
        b.extend_from_slice(&(n * 2).to_le_bytes());
        for v in [0i16, 16_384, -32_768] {
            b.extend_from_slice(&v.to_le_bytes());
        }
        let d = parse(&b).unwrap();
        assert_eq!((d.rate, d.channels), (24_000, 1));
        assert_eq!(d.samples[0], vec![0.0, 0.5, -1.0]);
    }
}
