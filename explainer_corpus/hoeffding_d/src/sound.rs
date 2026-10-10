//! A tiny procedural score: one soft chord pad per chapter and a bell
//! chime for key reveals, written as 48 kHz s16 stereo WAV (std only) and
//! handed to franken_manim's native, sample-exact mixer via `add_sound`.

use std::f64::consts::TAU;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use fmn::prelude::*;

const RATE: u32 = 48_000;

fn write_wav(path: &Path, left: &[f64], right: &[f64]) -> std::io::Result<()> {
    let n = left.len() as u32;
    let data_len = n * 4;
    let mut w = BufWriter::new(File::create(path)?);
    w.write_all(b"RIFF")?;
    w.write_all(&(36 + data_len).to_le_bytes())?;
    w.write_all(b"WAVEfmt ")?;
    w.write_all(&16u32.to_le_bytes())?;
    w.write_all(&1u16.to_le_bytes())?; // PCM
    w.write_all(&2u16.to_le_bytes())?; // stereo
    w.write_all(&RATE.to_le_bytes())?;
    w.write_all(&(RATE * 4).to_le_bytes())?;
    w.write_all(&4u16.to_le_bytes())?;
    w.write_all(&16u16.to_le_bytes())?;
    w.write_all(b"data")?;
    w.write_all(&data_len.to_le_bytes())?;
    for (l, r) in left.iter().zip(right) {
        for s in [l, r] {
            let v = (s.clamp(-1.0, 1.0) * 32767.0).round() as i16;
            w.write_all(&v.to_le_bytes())?;
        }
    }
    w.flush()
}

/// A warm, slowly breathing chord: detuned sine pairs panned apart.
fn pad(seconds: f64, chord: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let n = (seconds * f64::from(RATE)).round() as usize;
    let (mut l, mut r) = (vec![0.0; n], vec![0.0; n]);
    let fade = 2.5_f64.min(seconds / 3.0);
    let level = 0.15 / chord.len() as f64;
    for (k, &f) in chord.iter().enumerate() {
        let pan = if k % 2 == 0 { 0.35 } else { 0.65 };
        let phase = k as f64 * 1.3;
        for i in 0..n {
            let t = i as f64 / f64::from(RATE);
            let env = (t / fade).min(1.0).min((seconds - t) / fade).max(0.0);
            let env = env * env * (3.0 - 2.0 * env); // smoothstep
            let breathe = 0.8 + 0.2 * (TAU * 0.07 * t + phase).sin();
            let s = (TAU * f * t).sin()
                + 0.6 * (TAU * f * 1.003 * t + phase).sin()
                + 0.12 * (TAU * 2.0 * f * t).sin();
            let v = level * env * breathe * s;
            l[i] += v * (1.0 - pan);
            r[i] += v * pan;
        }
    }
    (l, r)
}

/// A soft bell: inharmonic partials with exponential decays.
fn chime(freq: f64) -> (Vec<f64>, Vec<f64>) {
    let seconds = 2.8;
    let n = (seconds * f64::from(RATE)) as usize;
    let partials = [
        (1.0, 1.0, 1.6),
        (2.76, 0.45, 0.9),
        (5.4, 0.2, 0.5),
        (0.5, 0.25, 2.2),
    ];
    let mut out = vec![0.0; n];
    for (i, o) in out.iter_mut().enumerate() {
        let t = i as f64 / f64::from(RATE);
        let attack = (t / 0.006).min(1.0);
        *o = 0.16
            * attack
            * partials
                .iter()
                .map(|&(m, a, d)| a * (-t / d).exp() * (TAU * freq * m * t).sin())
                .sum::<f64>();
    }
    (out.clone(), out)
}

/// The chapter's score, keyed by chapter.
#[derive(Clone)]
pub struct Score {
    pub dir: PathBuf,
    /// Duck the bed and the chimes under a voice-over.
    pub under_voice: bool,
}

pub const CHORDS: [&[f64]; 8] = [
    &[146.83, 220.00, 293.66, 369.99],         // D
    &[123.47, 185.00, 293.66, 440.00],         // Bm7
    &[98.00, 146.83, 246.94, 369.99],          // Gmaj7
    &[146.83, 220.00, 329.63, 369.99],         // Dadd9
    &[164.81, 246.94, 293.66, 392.00],         // Em7
    &[98.00, 146.83, 220.00, 246.94, 369.99],  // Gmaj9
    &[123.47, 185.00, 220.00, 293.66],         // Bm7
    &[146.83, 220.00, 293.66, 369.99, 440.00], // D
];

impl Score {
    /// Lay a pad under everything played so far (call at the end of
    /// `construct`): the WAV is exactly the scene's length and starts at 0.
    pub fn pad_under(&self, stage: &mut Stage<'_>, chapter: usize) -> fmn::Result<()> {
        let seconds = stage.scene().time().to_f64();
        if seconds <= 0.5 {
            return Ok(());
        }
        let path = self.dir.join(format!("pad_{chapter:02}.wav"));
        let (l, r) = pad(seconds, CHORDS[chapter % CHORDS.len()]);
        write_wav(&path, &l, &r)
            .map_err(|e| fmn::Error::Scene(SceneError::InvalidConfig(leak(format!("pad: {e}")))))?;
        let gain = if self.under_voice { -9.0 } else { 0.0 };
        stage
            .scene_mut()
            .add_sound(path, -seconds, Some(gain), None)?;
        Ok(())
    }

    /// A bell at the current moment.
    pub fn chime(&self, stage: &mut Stage<'_>, freq: f64) -> fmn::Result<()> {
        let path = self.dir.join(format!("chime_{}.wav", freq.round() as u32));
        if !path.exists() {
            let (l, r) = chime(freq);
            write_wav(&path, &l, &r).map_err(|e| {
                fmn::Error::Scene(SceneError::InvalidConfig(leak(format!("chime: {e}"))))
            })?;
        }
        let gain = if self.under_voice { -10.0 } else { -3.0 };
        stage.scene_mut().add_sound(path, 0.0, Some(gain), None)?;
        Ok(())
    }
}

fn leak(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}
