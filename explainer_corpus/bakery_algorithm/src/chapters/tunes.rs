//! Video C's teaching sounds (plan §7.5's signature sonification): whoever
//! is in the room plays the bakery's "served" motif (5̂–4̂–2̂–1̂, the
//! identity's `resolution`) in her own timbre, so one tune at a time is
//! mutual exclusion you can hear, and two at once is the bug. Each sound is
//! written as a WAV for `kit.sonify`; the score plays it in its listen window
//! at the identity's listen level (music lane, 2026-10-11: pitches on B♭ C D
//! E♭ F G, marimba no higher than E5, written near −23 LUFS).

use std::path::PathBuf;

use explainer_score::instr::{Instrument, Note, render};
use explainer_score::score::Identity;
use explainer_score::sonify::motif;
use explainer_score::{FS as SAMPLE_RATE, Stereo, meter};

/// Customer `k`'s timbre (1-based): Blue marimba, Green pluck, Purple FM
/// electric piano, Teal felt piano, Pink kalimba.
pub fn timbre(k: usize) -> Instrument {
    match k {
        1 => Instrument::Marimba,
        2 => Instrument::Pluck,
        3 => Instrument::FmEp,
        4 => Instrument::FeltPiano,
        _ => Instrument::Kalimba,
    }
}

fn identity() -> Identity {
    Identity::bakery_algorithm()
}

/// One customer's "served", and its notes' onsets (seconds).
pub fn served(k: usize) -> (Stereo, Vec<f64>) {
    let id = identity();
    motif(&id, &id.resolution, timbre(k), 0, 0.8, 0x5E7E_0000 + k as u64)
}

/// The scale degree of each note of "served" (for drawing note-heads).
pub fn served_degrees() -> Vec<i32> {
    identity().resolution.degrees
}

/// Customers served one after another, in `order`, each starting just after
/// the last one's final note ends: the sound and, per turn, its notes'
/// onsets from the start.
pub fn turns(order: &[usize]) -> (Stereo, Vec<Vec<f64>>) {
    let beat = identity().beat_s;
    let mut parts = Vec::new();
    let mut at = 0.0;
    let mut onsets = Vec::new();
    for &k in order {
        let (s, o) = served(k);
        let length = o.last().copied().unwrap_or(0.0) + 1.5 * beat;
        onsets.push(o.iter().map(|t| t + at).collect());
        parts.push((s, at));
        // the next customer starts a sixteenth after this one's last note
        // ends, so only one tune ever sounds at a time
        at += length + 0.25 * beat;
    }
    let total = parts
        .iter()
        .map(|(s, t)| t + s.len() as f64 / SAMPLE_RATE)
        .fold(0.0, f64::max);
    let mut out = Stereo::silence((total * SAMPLE_RATE).ceil() as usize);
    for (s, t) in &parts {
        out.mix_in(s, (t * SAMPLE_RATE).round() as usize, 1.0);
    }
    (out, onsets)
}

/// Two customers in the room at once: both tunes together, the second an
/// eighth late (the clash is rhythm and timbre, never out of key), cut to
/// `seconds` with a short fade.
pub fn clash(a: usize, b: usize, seconds: f64) -> Stereo {
    let beat = identity().beat_s;
    let (sa, _) = served(a);
    let (sb, _) = served(b);
    let offset = (0.5 * beat * SAMPLE_RATE).round() as usize;
    let mut out = Stereo::silence(sa.len().max(sb.len() + offset));
    out.mix_in(&sa, 0, 1.0);
    out.mix_in(&sb, offset, 1.0);
    let keep = ((seconds * SAMPLE_RATE).round() as usize).min(out.len());
    out.l.truncate(keep);
    out.r.truncate(keep);
    let fade = ((0.25 * SAMPLE_RATE) as usize).min(keep);
    for i in 0..fade {
        let g = i as f64 / fade as f64;
        out.l[keep - 1 - i] *= g;
        out.r[keep - 1 - i] *= g;
    }
    out
}

/// "After you. No, after you." (04_3): two customers trading two soft notes
/// a beat apart, never landing on the tonic, `notes` in all.
pub fn after_you(a: usize, b: usize, notes: usize) -> Stereo {
    let id = identity();
    let beat = id.beat_s;
    // D and C over B♭: a polite step that never resolves
    let pitches = [f64::from(id.tonic_midi + 4), f64::from(id.tonic_midi + 2)];
    let length = (notes as f64 * beat + 1.0) * SAMPLE_RATE;
    let mut out = Stereo::silence(length.ceil() as usize);
    for n in 0..notes {
        let (who, pitch) = if n % 2 == 0 { (a, pitches[0]) } else { (b, pitches[1]) };
        let note = Note::new(pitch, 0.55, 0.8 * beat, 0xAF7E_0000 + n as u64);
        out.mix_in(
            &render(timbre(who), &note),
            (n as f64 * beat * SAMPLE_RATE).round() as usize,
            1.0,
        );
    }
    out
}

/// The level the score expects a sonification at (LUFS).
pub const LEVEL_LUFS: f64 = -23.0;

/// Level `sound` to [`LEVEL_LUFS`] by one fixed gain and write it where the
/// narrator and the score can read it; returns the path and its length (s).
pub fn write(label: &str, mut sound: Stereo) -> std::io::Result<(PathBuf, f64)> {
    let lufs = meter::integrated(&sound);
    if lufs.is_finite() {
        sound.scale(10f64.powf((LEVEL_LUFS - lufs) / 20.0));
    }
    let seconds = sound.len() as f64 / SAMPLE_RATE;
    Ok((explainer_kit::sound_file(label, &sound)?, seconds))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// MIDI pitch classes the listen chord leaves room for: B♭ C D E♭ F G.
    const IN_KEY: [i32; 6] = [10, 0, 2, 3, 5, 7];

    #[test]
    fn every_teaching_sound_is_in_key_and_leveled() {
        let id = identity();
        for p in id.resolution.pitches(&id.key, id.tonic_midi) {
            assert!(
                IN_KEY.contains(&p.rem_euclid(12)),
                "served note {p} is off the listen scale"
            );
            // marimba no higher than E5 (MIDI 76)
            assert!(p <= 76, "served note {p} above E5");
        }
        for p in [id.tonic_midi + 4, id.tonic_midi + 2] {
            assert!(IN_KEY.contains(&p.rem_euclid(12)) && p % 12 != id.tonic_midi % 12);
        }
        let (mut s, onsets) = turns(&[5, 2, 1]);
        assert_eq!(onsets.len(), 3);
        // one tune at a time: each turn's notes start after the last turn's
        for w in onsets.windows(2) {
            assert!(w[1][0] > *w[0].last().expect("notes"));
        }
        let lufs = meter::integrated(&s);
        s.scale(10f64.powf((LEVEL_LUFS - lufs) / 20.0));
        assert!((meter::integrated(&s) - LEVEL_LUFS).abs() < 0.1);
        assert!(s.is_finite() && s.peak() < 1.0);
        let c = clash(1, 2, 2.0);
        assert!(c.is_finite() && (c.len() as f64 / SAMPLE_RATE - 2.0).abs() < 1e-3);
        assert!(after_you(3, 4, 5).is_finite());
    }
}
