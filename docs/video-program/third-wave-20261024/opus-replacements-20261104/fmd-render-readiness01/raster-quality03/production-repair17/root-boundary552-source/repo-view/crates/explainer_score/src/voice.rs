//! The narrator's pitch, measured from a cut's own narration.
//!
//! Plan §7.3 hard rule 1 keeps sustained music fundamentals out of the
//! narrator's pitch band. Which band that is belongs to the voice that reads
//! the cut, not to the video: a video can be read by another voice (its
//! narrator can change, and `VOICE=<voice> ./x` makes variants), so a score
//! measures the narration it is mixed against, every time, and places its
//! [`Register`] from that. Nothing is looked up by a voice's name.
//!
//! The measurement is YIN (de Cheveigné and Kawahara, 2002) as
//! `docs/research/sound_design/scripts/yin2.py` runs it: each line decimated
//! to 8 kHz, 30 ms frames every 10 ms, frames quieter than −36.5 dBFS RMS
//! skipped, the first lag from 400 Hz down whose cumulative-mean-normalized
//! difference is under 0.12, followed to its local minimum, down to 60 Hz.
//! The 5th and 95th percentiles of the voiced frames' f0 give the band
//! ([`Register::from_f0`]).

use std::io;
use std::path::{Path, PathBuf};

use crate::resample::resample;
use crate::theory::Register;

const RATE: u32 = 8_000;
/// The frame (30 ms) and hop (10 ms) at 8 kHz.
const FRAME: usize = 240;
const HOP: usize = 80;
/// Lags searched (samples at 8 kHz): 400 Hz down to just above 60 Hz.
const LAG_MIN: usize = 20;
const LAG_END: usize = 133;
/// A frame quieter than this (RMS) is not speech (−36.5 dBFS).
const RMS_FLOOR: f64 = 0.015;
/// YIN's absolute threshold on the normalized difference.
const DIP: f64 = 0.12;

/// The least voiced speech a profile is taken from: 300 frames (3 s). A real
/// narration has thousands; under this the percentiles are not a voice.
pub const MIN_VOICED_FRAMES: usize = 300;

/// The f0 (Hz) of every voiced frame of one line.
pub fn f0_frames(samples: &[f64], rate: u32) -> Vec<f64> {
    let x = resample(samples, rate, RATE);
    let mut f0 = Vec::new();
    let mut cm = [1.0_f64; LAG_END];
    let mut i = 0;
    while i + FRAME + LAG_END <= x.len() {
        let a = &x[i..i + FRAME];
        let rms = (a.iter().map(|v| v * v).sum::<f64>() / FRAME as f64).sqrt();
        if rms > RMS_FLOOR {
            let mut cum = 0.0;
            for (lag, c) in cm.iter_mut().enumerate().skip(1) {
                let b = &x[i + lag..i + lag + FRAME];
                let d: f64 = a.iter().zip(b).map(|(p, q)| (p - q) * (p - q)).sum();
                cum += d;
                *c = d * lag as f64 / cum.max(1e-12);
            }
            if let Some(mut lag) = (LAG_MIN..LAG_END).find(|&l| cm[l] < DIP) {
                while lag + 1 < LAG_END && cm[lag + 1] < cm[lag] {
                    lag += 1;
                }
                f0.push(f64::from(RATE) / lag as f64);
            }
        }
        i += HOP;
    }
    f0
}

/// The `q`-th percentile (0–100) of sorted values, linearly interpolated.
fn percentile(sorted: &[f64], q: f64) -> f64 {
    let pos = q / 100.0 * (sorted.len() - 1) as f64;
    let (lo, frac) = (pos.floor() as usize, pos.fract());
    let hi = (lo + 1).min(sorted.len() - 1);
    sorted[lo] + (sorted[hi] - sorted[lo]) * frac
}

/// One cut's narrator, as measured from its narration lines.
#[derive(Clone, Debug, PartialEq)]
pub struct VoiceProfile {
    /// The voiced frames' f0 (Hz): 5th percentile, median, 95th percentile.
    pub p5: f64,
    pub median: f64,
    pub p95: f64,
    pub voiced_frames: usize,
    pub lines: usize,
    /// The directory the lines were read from (one cut, one voice).
    pub dir: PathBuf,
    /// SHA-256 over the lines' own SHA-256 digests, in the order given: the
    /// narration this profile was measured from, and no other.
    pub source: String,
}

impl VoiceProfile {
    /// Measure the narration `lines` (WAV files, any rate). Refuses lines
    /// from more than one directory (a cut is read by one voice; a primary
    /// cut and a variant never share a score) and narration with too little
    /// voiced speech to tell where the voice sits.
    pub fn measure(lines: &[&Path]) -> io::Result<Self> {
        let bad = |m: String| Err(io::Error::new(io::ErrorKind::InvalidData, m));
        let dirs: std::collections::BTreeSet<&Path> = lines.iter().filter_map(|p| p.parent()).collect();
        if dirs.len() > 1 {
            return bad(format!(
                "the narration's lines come from {} directories ({}): one cut is read by one voice",
                dirs.len(),
                dirs.iter()
                    .map(|d| d.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        let mut f0 = Vec::new();
        let mut digests = Vec::with_capacity(lines.len() * 32);
        for path in lines {
            let bytes = std::fs::read(path)?;
            digests.extend(crate::digest::sha256(&bytes));
            let d =
                crate::wav::parse(&bytes).map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", path.display())))?;
            f0.extend(f0_frames(&d.mono(), d.rate));
        }
        if f0.len() < MIN_VOICED_FRAMES {
            return bad(format!(
                "the narration has {} voiced frames in {} lines (at least {MIN_VOICED_FRAMES} needed): \
                 the music's register is placed from the narrator's measured pitch, and there is none to measure",
                f0.len(),
                lines.len()
            ));
        }
        f0.sort_by(f64::total_cmp);
        Ok(Self {
            p5: percentile(&f0, 5.0),
            median: percentile(&f0, 50.0),
            p95: percentile(&f0, 95.0),
            voiced_frames: f0.len(),
            lines: lines.len(),
            dir: dirs.into_iter().next().unwrap_or(Path::new("")).to_path_buf(),
            source: crate::digest::sha256_hex(&digests),
        })
    }

    /// Where the music may sit around this voice.
    pub fn register(&self) -> Register {
        Register::from_f0(self.p5, self.p95)
    }

    /// The record `./x <slug> score` leaves beside the stems.
    pub fn to_text(&self) -> String {
        let r = self.register();
        format!(
            "# voice profile v1: the narrator's pitch, measured from this cut's own narration\n\
             # (YIN at 8 kHz, 30 ms frames every 10 ms); the score's register follows it\n\
             narration_dir\t{}\nnarration_sha256\t{}\nlines\t{}\nvoiced_frames\t{}\n\
             f0_p5_hz\t{:.2}\nf0_median_hz\t{:.2}\nf0_p95_hz\t{:.2}\n\
             no_fly_hz\t{:.2}\t{:.2}\nbass_midi\t{}\t{}\nupper_midi\t{}\t{}\n",
            self.dir.display(),
            self.source,
            self.lines,
            self.voiced_frames,
            self.p5,
            self.median,
            self.p95,
            r.no_fly.0,
            r.no_fly.1,
            r.bass.0,
            r.bass.1,
            r.upper.0,
            r.upper.1
        )
    }
}

/// A voice-like line for tests and auditions without real takes: a
/// band-limited pulse train (harmonics falling 6 dB per octave under
/// 3.4 kHz) whose f0 glides sinusoidally between `f0.0` and `f0.1` Hz every
/// 2.3 s, under a 4 Hz syllable envelope, with a little noise. 48 kHz mono.
pub fn voiced_line(seconds: f64, f0: (f64, f64), seed: u64) -> Vec<f64> {
    let mut rng = crate::rng::Rng::new(seed);
    let (mid, half) = ((f0.0 + f0.1) / 2.0, (f0.1 - f0.0) / 2.0);
    let start = rng.uniform() * std::f64::consts::TAU;
    let mut phase = 0.0_f64;
    (0..(seconds * crate::FS) as usize)
        .map(|k| {
            let t = k as f64 / crate::FS;
            let f = mid + half * (std::f64::consts::TAU * t / 2.3 + start).sin();
            phase += f / crate::FS;
            let harmonics = (3_400.0 / f) as usize;
            let pulse: f64 = (1..=harmonics)
                .map(|h| (std::f64::consts::TAU * phase * h as f64).sin() / h as f64)
                .sum();
            let syllables = 0.6 + 0.4 * (std::f64::consts::TAU * 4.0 * t).sin();
            0.12 * syllables * (pulse + 0.05 * rng.normal())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Stereo, wav};

    fn write(dir: &Path, name: &str, mono: &[f64]) -> PathBuf {
        std::fs::create_dir_all(dir).unwrap();
        let p = dir.join(name);
        wav::write(&p, &Stereo::from_mono(mono), wav::Format::F32).unwrap();
        p
    }

    fn scratch(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("explainer_score_voice_{name}_{}", std::process::id()))
    }

    #[test]
    fn a_steady_tone_is_found_to_its_lag() {
        // 125 Hz is 64 samples at 8 kHz exactly; the harmonics make it a voice
        let x: Vec<f64> = (0..48_000)
            .map(|k| {
                let t = k as f64 / 48_000.0;
                (1..=20)
                    .map(|h| 0.1 * (std::f64::consts::TAU * 125.0 * f64::from(h) * t).sin() / f64::from(h))
                    .sum()
            })
            .collect();
        let f0 = f0_frames(&x, 48_000);
        assert!(f0.len() > 90, "{} frames in one second", f0.len());
        assert!(f0.iter().all(|&f| (f - 125.0).abs() < 1e-9), "{f0:?}");
    }

    #[test]
    fn a_glide_gives_its_own_percentiles() {
        // f0 = mid + half·sin: the q-th percentile is mid + half·sin(π(q − ½))
        for (lo, hi) in [(80.0, 160.0), (110.0, 210.0)] {
            let dir = scratch(&format!("glide{lo}"));
            let lines: Vec<PathBuf> = (0..4)
                .map(|i| write(&dir, &format!("l{i}.wav"), &voiced_line(6.0, (lo, hi), i)))
                .collect();
            let paths: Vec<&Path> = lines.iter().map(PathBuf::as_path).collect();
            let p = VoiceProfile::measure(&paths).unwrap();
            let (mid, half) = ((lo + hi) / 2.0, (hi - lo) / 2.0);
            let want = |q: f64| mid + half * (std::f64::consts::PI * (q - 0.5)).sin();
            // within a quarter tone (3 %): lags are whole samples at 8 kHz
            for (got, q) in [(p.p5, 0.05), (p.median, 0.5), (p.p95, 0.95)] {
                assert!(
                    (got / want(q) - 1.0).abs() < 0.03,
                    "{lo}-{hi} q{q}: {got} vs {}",
                    want(q)
                );
            }
            assert!(p.voiced_frames > 2_000 && p.lines == 4 && p.dir == dir, "{p:?}");
            p.register().check().unwrap();
        }
    }

    #[test]
    fn the_register_follows_the_voice() {
        let dir = scratch("two");
        let low = write(&dir, "low.wav", &voiced_line(12.0, (78.0, 160.0), 1));
        let high = write(&dir, "high.wav", &voiced_line(12.0, (108.0, 204.0), 2));
        let (l, h) = (
            VoiceProfile::measure(&[&low]).unwrap(),
            VoiceProfile::measure(&[&high]).unwrap(),
        );
        assert_ne!(l.source, h.source);
        let (rl, rh) = (l.register(), h.register());
        // the low voice pushes the bass down and lets the upper voices start lower
        assert!(rl.bass.1 < rh.bass.1 && rl.upper.0 < rh.upper.0, "{rl:?} {rh:?}");
        // E2 (82.4 Hz) is a bass note beside the high voice and inside the low one's band
        assert!(rl.in_band(82.4) && !rh.in_band(82.4));
        // the same bytes measure the same, and the record names what was measured
        assert_eq!(l, VoiceProfile::measure(&[&low]).unwrap());
        let text = l.to_text();
        assert!(text.contains(&format!("narration_sha256\t{}", l.source)) && text.contains("f0_p5_hz\t"));
    }

    #[test]
    fn narration_without_a_voice_is_refused() {
        let dir = scratch("none");
        let mut rng = crate::rng::Rng::new(3);
        let noise: Vec<f64> = (0..480_000).map(|_| 0.1 * rng.normal()).collect();
        let n = write(&dir, "noise.wav", &noise);
        let quiet = write(&dir, "quiet.wav", &vec![0.0; 480_000]);
        let brief = write(&dir, "brief.wav", &voiced_line(1.5, (90.0, 150.0), 4));
        for p in [&n, &quiet, &brief] {
            let e = VoiceProfile::measure(&[p]).unwrap_err();
            assert_eq!(e.kind(), io::ErrorKind::InvalidData);
            assert!(e.to_string().contains("voiced frames"), "{e}");
        }
        assert!(VoiceProfile::measure(&[]).is_err(), "no lines at all");
        // lines from two directories are two cuts
        let other = write(&dir.join("variants"), "v.wav", &voiced_line(8.0, (90.0, 150.0), 5));
        let here = write(&dir, "here.wav", &voiced_line(8.0, (90.0, 150.0), 6));
        let e = VoiceProfile::measure(&[&here, &other]).unwrap_err();
        assert!(e.to_string().contains("2 directories"), "{e}");
        VoiceProfile::measure(&[&here]).unwrap();
    }
}
