//! The PCM master (plan §7.7, D-20): the programme's soundtrack rebuilt from
//! its sources, not from per-chapter AAC.
//!
//! franken_manim mixes each chapter natively, quantizes it to s16 without
//! dither and encodes AAC per chapter; joining those means two lossy
//! generations and a gap at every join. Instead, for each chapter:
//! 1. the exact cue log (`Narrator::cue_log`) is replayed through fmn's own
//!    decoder and mixer (`fmn-output`, the pinned revision), which reproduces
//!    its resampler, channel mapping, gains and summation exactly;
//! 2. the replay, quantized as fmn quantizes (s16, no dither), is compared
//!    with the `soundtrack.wav` fmn actually muxed (captured by the encoder
//!    wrapper): a mismatch fails the master;
//! 3. chapters concatenate sample-exactly (each is frames × 48000 / fps
//!    samples); one fixed gain takes the programme to −16 LUFS unless the
//!    −1.5 dBTP ceiling binds first;
//! 4. the chapters' video streams are joined without re-encoding (each
//!    remuxed video-only, then concatenated with their own timestamps), the
//!    master is muxed as AAC 256k once, and the BT.709 tags are written
//!    losslessly; the joined timestamps are read back and must sit exactly on
//!    the frame grid.

use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use explainer_score::{Stereo, digest, meter, units::db_to_gain, wav};
use fmn_codec::SampleFormat;
use fmn_output::{
    AudioDecodeLimits, AudioDecoder, DitherPolicy, MixReport, MixerConfig, SoundCue, SoundMixer, frames_to_samples,
};

/// One cue as the narrator logged it.
#[derive(Clone, Debug, PartialEq)]
pub struct LoggedCue {
    pub path: PathBuf,
    pub sha256: String,
    pub frame: i64,
    pub fps: u32,
    pub offset: f64,
    pub gain_db: Option<f64>,
}

fn bad(n: usize, m: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, format!("cue log line {n}: {m}"))
}

fn bits(n: usize, s: &str) -> io::Result<f64> {
    let h = s
        .strip_prefix("0x")
        .ok_or_else(|| bad(n, format!("not hex bits: {s}")))?;
    u64::from_str_radix(h, 16)
        .map(f64::from_bits)
        .map_err(|_| bad(n, format!("not hex bits: {s}")))
}

/// Parse a chapter's exact cue log. Offsets and gains come from their f64
/// bits, never from the readable columns.
pub fn parse_cue_log(text: &str) -> io::Result<Vec<LoggedCue>> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let n = i + 1;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() != 13 {
            return Err(bad(n, format!("{} fields, want 13", f.len())));
        }
        let int = |k: usize| {
            f[k].parse::<i64>()
                .map_err(|_| bad(n, format!("bad integer {:?}", f[k])))
        };
        out.push(LoggedCue {
            path: PathBuf::from(f[0]),
            sha256: f[1].to_owned(),
            frame: int(3)?,
            fps: u32::try_from(int(4)?).map_err(|_| bad(n, "bad fps"))?,
            offset: bits(n, f[6])?,
            gain_db: if f[8] == "-" { None } else { Some(bits(n, f[8])?) },
        });
    }
    Ok(out)
}

/// Replay a chapter's cues through fmn's own decoder and mixer, over the
/// chapter's `frames` at `fps` (the soundtrack spans at least the picture).
/// A cue file whose bytes changed since the render is refused.
pub fn replay(cues: &[LoggedCue], frames: u64, fps: u32) -> Result<MixReport, String> {
    let config = MixerConfig::default();
    let timeline = frames_to_samples(frames as i64, fps, config.sample_rate).map_err(|e| e.to_string())?;
    let mut mixer = SoundMixer::new(config)
        .map_err(|e| e.to_string())?
        .with_timeline_frames(timeline as u64);
    let mut decoder = AudioDecoder::new(AudioDecodeLimits::default(), None).map_err(|e| e.to_string())?;
    for c in cues {
        let bytes = std::fs::read(&c.path).map_err(|e| format!("{}: {e}", c.path.display()))?;
        let sha = digest::sha256_hex(&bytes);
        if sha != c.sha256 {
            return Err(format!(
                "{}: changed since the render (sha256 {sha}, logged {})",
                c.path.display(),
                c.sha256
            ));
        }
        let decoded = decoder
            .decode(&bytes)
            .map_err(|e| format!("{}: {e}", c.path.display()))?;
        mixer
            .add(SoundCue {
                audio: decoded.audio,
                frame: c.frame,
                fps: c.fps,
                time_offset: c.offset,
                gain: c.gain_db,
                gain_to_background: None,
            })
            .map_err(|e| format!("{}: {e}", c.path.display()))?;
    }
    let threads = std::thread::available_parallelism().map_or(1, |p| p.get());
    mixer.mix(threads).map_err(|e| e.to_string())
}

/// The null test: the replay quantized exactly as fmn quantized the
/// soundtrack it muxed (s16, no dither) against that captured file. `Ok(())`
/// only when they are identical, byte for byte.
pub fn null_test(replayed: &MixReport, captured: &[u8]) -> Result<(), String> {
    let ours = replayed
        .wav_bytes(SampleFormat::S16, DitherPolicy::None)
        .map_err(|e| e.to_string())?;
    if ours == captured {
        return Ok(());
    }
    let (a, b) = (wav::parse(&ours), wav::parse(captured));
    match (a, b) {
        (Ok(a), Ok(b)) if a.frames() == b.frames() => {
            let worst = a
                .samples
                .iter()
                .zip(&b.samples)
                .flat_map(|(x, y)| x.iter().zip(y).map(|(p, q)| (p - q).abs()))
                .fold(0.0_f64, f64::max);
            Err(format!(
                "replay differs from fmn's soundtrack by up to {:.1} dBFS",
                20.0 * worst.max(1e-12).log10()
            ))
        }
        (Ok(a), Ok(b)) => Err(format!(
            "replay has {} frames, fmn's soundtrack {}",
            a.frames(),
            b.frames()
        )),
        (a, b) => Err(format!("unreadable: replay {:?}, captured {:?}", a.err(), b.err())),
    }
}

/// The replay's audio as an f64 stereo buffer (the f32 mix, unquantized).
pub fn to_stereo(report: &MixReport) -> Stereo {
    let s = &report.audio.samples;
    match report.audio.channels {
        2 => Stereo {
            l: s.iter().step_by(2).map(|&x| f64::from(x)).collect(),
            r: s.iter().skip(1).step_by(2).map(|&x| f64::from(x)).collect(),
        },
        _ => Stereo::from_mono(&s.iter().map(|&x| f64::from(x)).collect::<Vec<_>>()),
    }
}

/// One fixed gain for the whole programme: −16 LUFS integrated unless the
/// −1.5 dBTP ceiling binds first. Returns (gain dB, LUFS, dBTP) after it.
pub fn apply_master_gain(programme: &mut Stereo) -> (f64, f64, f64) {
    let (i, tp) = (meter::integrated(programme), meter::true_peak_db(programme));
    let g = (-16.0 - i).min(-1.5 - tp);
    programme.scale(db_to_gain(g));
    (g, i + g, tp + g)
}

fn ffmpeg(args: &[&str]) -> io::Result<()> {
    let out = Command::new("ffmpeg")
        .args(["-nostdin", "-v", "error", "-y"])
        .args(args)
        .output()?;
    if out.status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "ffmpeg {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr)
        )))
    }
}

/// Join the chapters' video streams without re-encoding and mux the master
/// as AAC 256k, BT.709 tags written losslessly. `work` holds the
/// intermediates; `out` must not exist.
///
/// Each chapter is first remuxed video-only, so the concat demuxer offsets
/// every chapter by its picture's exact length (with the chapter's AAC still
/// in, the offset would be the longer audio's, and every join would open a
/// gap). Raw Annex-B concatenation is not an option: the raw stream carries
/// no presentation times, so B-frames come back with pts = dts and the
/// remux drops and misorders frames (seen on Hoeffding's master).
pub fn join(chapters: &[PathBuf], master_wav: &Path, work: &Path, out: &Path) -> io::Result<()> {
    if out.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("{} exists", out.display()),
        ));
    }
    std::fs::create_dir_all(work)?;
    let mut parts = Vec::new();
    for (k, c) in chapters.iter().enumerate() {
        let part = work.join(format!("{k:02}.video.mp4"));
        ffmpeg(&[
            "-i",
            &c.to_string_lossy(),
            "-map",
            "0:v:0",
            "-c:v",
            "copy",
            "-an",
            &part.to_string_lossy(),
        ])?;
        parts.push(part);
    }
    let list_path = work.join("chapters.txt");
    std::fs::write(&list_path, concat_list(&parts)?)?;
    ffmpeg(&[
        "-f",
        "concat",
        "-safe",
        "0",
        "-i",
        &list_path.to_string_lossy(),
        "-i",
        &master_wav.to_string_lossy(),
        "-map",
        "0:v:0",
        "-map",
        "1:a:0",
        "-c:v",
        "copy",
        "-bsf:v",
        "h264_metadata=colour_primaries=1:transfer_characteristics=1:matrix_coefficients=1",
        "-c:a",
        "aac",
        "-b:a",
        "256k",
        "-ar",
        "48000",
        "-movflags",
        "+faststart",
        &out.to_string_lossy(),
    ])
}

/// The concat demuxer's list: each part by its absolute path (the demuxer
/// resolves a relative entry against the list file's own directory, so a
/// relative `--out` named every part twice over), quoted with ', a ' inside
/// written '\''.
fn concat_list(parts: &[PathBuf]) -> io::Result<String> {
    let mut list = String::new();
    for p in parts {
        let abs = std::fs::canonicalize(p)?;
        list.push_str(&format!("file '{}'\n", abs.to_string_lossy().replace('\'', r"'\''")));
    }
    Ok(list)
}

/// The joined video's timestamps, read back: decode times strictly
/// increasing, `frames` packets, and the presentation times exactly the
/// frame grid (frame k at k / `fps`, within a millisecond), so no frame is
/// dropped, doubled or shown out of order.
pub fn check_timestamps(path: &Path, fps: u32, frames: u64) -> io::Result<()> {
    let o = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "packet=pts_time,dts_time",
            "-of",
            "csv=p=0",
        ])
        .arg(path)
        .output()?;
    if !o.status.success() {
        return Err(io::Error::other(String::from_utf8_lossy(&o.stderr).into_owned()));
    }
    let bad = |m: String| io::Error::new(io::ErrorKind::InvalidData, format!("{}: {m}", path.display()));
    let mut pts = Vec::new();
    let mut last_dts = f64::NEG_INFINITY;
    for line in String::from_utf8_lossy(&o.stdout).lines() {
        let n = pts.len();
        let mut f = line.split(',').map(|x| x.trim().parse::<f64>());
        let (Some(Ok(p)), Some(Ok(d))) = (f.next(), f.next()) else {
            return Err(bad(format!("packet {n} has no timestamps: {line:?}")));
        };
        if d <= last_dts {
            return Err(bad(format!("decode time {d} at packet {n} does not increase")));
        }
        last_dts = d;
        pts.push(p);
    }
    if pts.len() as u64 != frames {
        return Err(bad(format!("{} video packets, want {frames}", pts.len())));
    }
    pts.sort_by(f64::total_cmp);
    for (k, p) in pts.iter().enumerate() {
        let want = k as f64 / f64::from(fps);
        if (p - want).abs() > 1e-3 {
            return Err(bad(format!("frame {k} is shown at {p:.4} s, its slot is {want:.4} s")));
        }
    }
    Ok(())
}

/// The master's gates, read back from the file: the video frame count and
/// the audio's duration.
pub fn probe(path: &Path) -> io::Result<(u64, f64)> {
    let run = |args: &[&str]| -> io::Result<String> {
        let o = Command::new("ffprobe")
            .args(["-v", "error"])
            .args(args)
            .arg(path)
            .output()?;
        if o.status.success() {
            Ok(String::from_utf8_lossy(&o.stdout).trim().to_owned())
        } else {
            Err(io::Error::other(String::from_utf8_lossy(&o.stderr).into_owned()))
        }
    };
    let frames = run(&[
        "-select_streams",
        "v:0",
        "-count_packets",
        "-show_entries",
        "stream=nb_read_packets",
        "-of",
        "csv=p=0",
    ])?
    .parse()
    .map_err(|_| io::Error::other("unreadable frame count"))?;
    let secs = run(&[
        "-select_streams",
        "a:0",
        "-show_entries",
        "stream=duration",
        "-of",
        "csv=p=0",
    ])?
    .parse()
    .map_err(|_| io::Error::other("unreadable audio duration"))?;
    Ok((frames, secs))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_concat_list_names_every_part_absolutely_and_quotes_safely() {
        // cargo runs tests from the package root: this path is relative
        let list = concat_list(&[PathBuf::from("Cargo.toml")]).unwrap();
        let path = list.strip_prefix("file '").unwrap().strip_suffix("'\n").unwrap();
        assert!(Path::new(path).is_absolute(), "{list}");
        assert!(Path::new(path).ends_with("crates/explainer_kit/Cargo.toml"), "{list}");
        // a quote in a path is closed, escaped and reopened
        let odd = tmp("it's.mp4");
        std::fs::write(&odd, b"x").unwrap();
        let list = concat_list(std::slice::from_ref(&odd)).unwrap();
        assert!(list.contains(r"it'\''s.mp4'"), "{list}");
        assert!(
            concat_list(&[tmp("missing.mp4")]).is_err(),
            "a missing part is an error"
        );
    }

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("explainer_kit_master_{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        d.join(name)
    }

    /// A one-cue log in the narrator's exact format.
    fn log_for(path: &Path, frame: i64, offset: f64, gain: Option<f64>) -> String {
        let bytes = std::fs::read(path).unwrap();
        let (g, gb) = match gain {
            Some(g) => (format!("{g:?}"), format!("{:#018x}", g.to_bits())),
            None => ("-".into(), "-".into()),
        };
        format!(
            "# header\n{}\t{}\t{}\t{frame}\t60\t{offset:?}\t{:#018x}\t{g}\t{gb}\t48000\t2\t0\t0\n",
            path.display(),
            digest::sha256_hex(&bytes),
            bytes.len(),
            offset.to_bits()
        )
    }

    #[test]
    fn the_cue_log_round_trips_exactly() {
        let p = tmp("line.wav");
        wav::write(&p, &Stereo::silence(4_800), wav::Format::F32).unwrap();
        let offset = -1.0 / 3.0; // not exact in decimal: only the bits keep it
        let cues = parse_cue_log(&log_for(&p, 7, offset, Some(-9.4))).unwrap();
        assert_eq!(cues.len(), 1);
        assert_eq!((cues[0].frame, cues[0].fps), (7, 60));
        assert_eq!(cues[0].offset.to_bits(), offset.to_bits());
        assert_eq!(cues[0].gain_db, Some(-9.4));
        assert!(parse_cue_log("a\tb\tc\n").unwrap_err().to_string().contains("line 1"));
    }

    #[test]
    fn a_replay_nulls_against_its_own_quantization_and_nothing_else() {
        let p = tmp("tone.wav");
        let m: Vec<f64> = (0..24_000).map(|i| 0.3 * (i as f64 * 0.03).sin()).collect();
        wav::write(&p, &Stereo::from_mono(&m), wav::Format::F32).unwrap();
        let cues = parse_cue_log(&log_for(&p, 3, 0.0, Some(-3.0))).unwrap();
        let r = replay(&cues, 60, 60).unwrap();
        assert_eq!(
            r.audio.samples.len(),
            2 * 48_000,
            "the timeline spans the picture: 60 frames"
        );
        let fmn_s16 = r.wav_bytes(SampleFormat::S16, DitherPolicy::None).unwrap();
        assert!(null_test(&r, &fmn_s16).is_ok());
        let mut off = fmn_s16.clone();
        let k = off.len() - 2;
        off[k] ^= 0x40;
        assert!(null_test(&r, &off).unwrap_err().contains("differs"));
        // a cue whose file changed since the render is refused
        let m2: Vec<f64> = m.iter().map(|x| x * 0.5).collect();
        wav::write(&p, &Stereo::from_mono(&m2), wav::Format::F32).unwrap();
        assert!(replay(&cues, 60, 60).unwrap_err().contains("changed since the render"));
    }
}
