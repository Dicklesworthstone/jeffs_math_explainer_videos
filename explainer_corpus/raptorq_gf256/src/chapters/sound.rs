//! Video B's sonifications (brief, "Sound"; plan §7.5): a byte is a chord.
//! Each set bit sounds one tone of E major, bit 0 lowest, voiced an octave
//! up (E5 … G♯6; creative B-1, B-2) so no two notes beat 15–60 Hz apart, on
//! the identity's warm-glass FM bell. 0x1D's set bits {0, 2, 3, 4} are then
//! E–G♯–B–C♯, the video's motif. A listen window's WAV is written at about
//! the narration's level, [`LEVEL_LUFS`]; the score re-levels it.

use std::f64::consts::{PI, TAU};
use std::path::PathBuf;

use explainer_score::instr::{Instrument, Note, render};
use explainer_score::sonify::{Grain, cloud};
use explainer_score::units::midi_to_hz;
use explainer_score::{FS, Stereo};

/// MIDI pitch of each bit, bit 0 (the ones place) first.
pub const BIT_PITCH: [f64; 8] = [76.0, 78.0, 80.0, 83.0, 85.0, 88.0, 90.0, 92.0];
/// The tone one slot above the top bit: where a doubled top bit would sound.
pub const OFF_THE_TOP: f64 = 95.0;
/// A sonification's integrated loudness as written (the narration's).
pub const LEVEL_LUFS: f64 = -23.0;
/// How long before a lap's window ends its dot lands home (11_4).
pub const LAP_TAIL_S: f64 = 1.1;
/// The fade every sonification ends with, seconds.
const TAIL_S: f64 = 0.25;
/// The gliding bell's attack. Its high partials stand about 35 dB over the
/// bed's at 10_5; the click gate allows a 20 dB rise per 5 ms frame, and a
/// raised cosine's first 5 ms carry -30 dB of its energy at 30 ms (-14 dB at
/// 12 ms, which still clicked: Codex B R11).
const ATTACK_S: f64 = 0.030;

fn frames(seconds: f64) -> usize {
    (seconds * FS).round() as usize
}

/// The chord of `byte` (one bell per set bit), held `hold` seconds, mixed
/// into `out` at `at` seconds. Velocities shrink with the chord's size, so
/// a full byte is not eight times as loud as one bit.
pub fn strike(out: &mut Stereo, byte: u8, at: f64, hold: f64, seed: u64) {
    let notes = byte.count_ones();
    if notes == 0 {
        return;
    }
    let velocity = 0.75 / f64::from(notes).sqrt();
    for (bit, &pitch) in BIT_PITCH.iter().enumerate() {
        if byte >> bit & 1 == 1 {
            let mut note = Note::new(pitch, velocity, hold, seed.wrapping_mul(31).wrapping_add(bit as u64));
            // the low bits a little left, the high ones a little right
            note.pan = (bit as f64 - 3.5) / 3.5 * 0.4;
            out.mix_in(&render(Instrument::WarmGlass, &note), frames(at), 1.0);
        }
    }
}

/// One bell whose pitch glides from `from` to `to` (MIDI) over `glide_s`
/// seconds from its start, fading away as it goes when `vanish`: the bit
/// that slides off the top (07_3, 10_5). The warm-glass recipe (an FM bell,
/// harmonic ratios) with a moving carrier.
pub fn glide(from: f64, to: f64, glide_s: f64, vanish: bool, seconds: f64) -> Stereo {
    let n = frames(seconds);
    let mut phase = 0.0;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f64 / FS;
        let u = (t / glide_s).clamp(0.0, 1.0);
        let s = u * u * (3.0 - 2.0 * u);
        let f = midi_to_hz(from + (to - from) * s);
        phase += TAU * f / FS;
        let fade = if vanish { (1.0 - u).powi(2) } else { 1.0 };
        // a raised-cosine attack (a 4 ms linear one clicked: Codex B R9)
        let a = (t / ATTACK_S).min(1.0);
        let env = (-t / 1.4).exp() * fade * 0.5 * (1.0 - (PI * a).cos());
        let idx = 0.9 * (-t / 0.5).exp() + 0.2;
        let y = (phase + idx * phase.sin()).sin() + 0.35 * (2.0 * phase + 0.6 * idx * (2.0 * phase).sin()).sin();
        out.push(0.3 * env * y);
    }
    Stereo::from_mono(&out)
}

/// Cut `s` to `seconds`, fade its last [`TAIL_S`] to an exact zero, and
/// scale it to [`LEVEL_LUFS`].
pub fn finish(mut s: Stereo, seconds: f64) -> Stereo {
    let n = frames(seconds);
    s.l.resize(n, 0.0);
    s.r.resize(n, 0.0);
    let tail = frames(TAIL_S).min(n);
    for k in 0..tail {
        let g = 0.5 - 0.5 * (std::f64::consts::PI * k as f64 / tail as f64).cos();
        s.l[n - 1 - k] *= g;
        s.r[n - 1 - k] *= g;
    }
    let lufs = explainer_score::meter::integrated(&s);
    if lufs.is_finite() {
        s.scale(10f64.powf((LEVEL_LUFS - lufs) / 20.0));
    }
    s
}

/// Chords in sequence: `(byte, start, hold)` each, `seconds` long in all.
pub fn chords(events: &[(u8, f64, f64)], seconds: f64) -> Stereo {
    let mut out = Stereo::silence(frames(seconds + 4.0));
    for (i, &(byte, at, hold)) in events.iter().enumerate() {
        strike(&mut out, byte, at, hold, i as u64 + 1);
    }
    finish(out, seconds)
}

/// The sound of doubling a byte whose top bit is set: the top bit's tone
/// slides up off the top and fades; with `fold`, the fold's chord blooms
/// where it vanished (10_5), otherwise silence is left (07_3).
pub fn overflow(fold: Option<u8>, seconds: f64) -> Stereo {
    let mut out = Stereo::silence(frames(seconds + 4.0));
    out.mix_in(&glide(BIT_PITCH[7], OFF_THE_TOP, 0.9, true, 1.4), 0, 1.0);
    if let Some(f) = fold {
        strike(&mut out, f, 0.95, seconds - 1.2, 7);
    }
    finish(out, seconds)
}

/// The byte clock's lap (11_4): one glassy grain per click, 255 of them,
/// climbing one octave from E5, then the home bell (E6) as the dot lands.
pub fn lap(clicks: usize, seconds: f64) -> Stereo {
    // the home bell lands before the score's fade of the window's last 0.8 s
    let run = seconds - LAP_TAIL_S;
    let grains: Vec<Grain> = (0..clicks)
        .map(|k| {
            let u = k as f64 / clicks as f64;
            Grain {
                start: u * run,
                pitch: BIT_PITCH[0] + 12.0 * u,
                pan: (u * 2.0 - 1.0) * 0.5,
                amp: 0.5,
            }
        })
        .collect();
    let mut out = cloud(&grains, seconds + 4.0);
    let home = Note::new(BIT_PITCH[0] + 12.0, 0.7, 0.4, 11);
    out.mix_in(&render(Instrument::WarmGlass, &home), frames(run), 1.0);
    finish(out, seconds)
}

/// The table's sort (12_3): a granular texture of the rows' chords (one
/// grain per set bit of each row's byte, in the order the rows move), thinning
/// as the rows and then the columns lock, settling into one held E chord
/// (E5 G♯5 B5 E6) at `settle` seconds, before the window's faded last 0.8 s.
pub fn settle(rows: &[u8], settle: f64, seconds: f64) -> Stereo {
    let mut grains = Vec::new();
    for (i, &byte) in rows.iter().enumerate() {
        let u = i as f64 / rows.len() as f64;
        // denser at the start, thinning toward the settle
        let start = settle * (1.0 - (1.0 - u).powf(0.6));
        for (bit, &pitch) in BIT_PITCH.iter().enumerate() {
            if byte >> bit & 1 == 1 {
                grains.push(Grain {
                    start,
                    pitch,
                    pan: (bit as f64 - 3.5) / 3.5 * 0.5,
                    amp: 0.18 * (1.0 - 0.6 * u),
                });
            }
        }
    }
    let mut out = cloud(&grains, seconds + 4.0);
    let e_chord = 0b0010_1101;
    strike(&mut out, e_chord, settle, seconds - settle, 5);
    finish(out, seconds)
}

/// Write a sonification where the narrator and the score read it; the
/// error is a scene error.
pub fn wav(label: &str, s: &Stereo) -> fmn::Result<PathBuf> {
    explainer_kit::sound_file(label, s).map_err(|e| {
        fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(
            format!("sonification {label}: {e}").into_boxed_str(),
        )))
    })
}

#[cfg(test)]
mod tests {
    use explainer_score::theory::rough;

    use super::*;

    #[test]
    fn the_byte_chord_never_beats_rough_on_the_videos_bytes() {
        // creative B-2: no two notes of a chord 15–60 Hz apart, on the bytes
        // the video plays (3, 7, 9, 29, 255) and on every other byte
        for byte in 0..=255u8 {
            let notes: Vec<i32> = (0..8)
                .filter(|b| byte >> b & 1 == 1)
                .map(|b| BIT_PITCH[b] as i32)
                .collect();
            for (i, &a) in notes.iter().enumerate() {
                for &b in &notes[i + 1..] {
                    assert!(!rough(a, b), "byte {byte}: {a} and {b} beat");
                }
            }
        }
    }

    #[test]
    fn the_folds_set_bits_play_the_motif() {
        // 0x1D = bits {0, 2, 3, 4}: E, G♯, B, C♯
        let motif: Vec<i32> = (0..8)
            .filter(|b| 0x1Du8 >> b & 1 == 1)
            .map(|b| BIT_PITCH[b] as i32 % 12)
            .collect();
        assert_eq!(motif, vec![4, 8, 11, 1]);
    }

    #[test]
    fn sonifications_have_their_length_level_and_a_clean_end() {
        for s in [
            chords(&[(3, 0.0, 0.8), (7, 1.0, 0.8), (9, 2.0, 0.9)], 3.0),
            overflow(None, 3.0),
            overflow(Some(0x1D), 3.0),
            lap(255, 3.0),
            settle(&(1..=255u8).collect::<Vec<_>>(), 2.0, 3.0),
        ] {
            assert_eq!(s.len(), frames(3.0));
            assert!(s.is_finite());
            assert!((explainer_score::meter::integrated(&s) - LEVEL_LUFS).abs() < 0.1);
            assert!(s.peak() < 1.0, "peak {}", s.peak());
            assert_eq!((s.l[s.len() - 1], s.r[s.len() - 1]), (0.0, 0.0));
        }
    }

    #[test]
    fn the_fallen_bit_leaves_silence_unless_it_is_folded_back() {
        // the last second: silent when the bit is lost, ringing when folded
        let tail = |s: &Stereo| s.l[frames(1.6)..frames(2.6)].iter().map(|x| x * x).sum::<f64>();
        let lost = overflow(None, 3.0);
        let folded = overflow(Some(0x1D), 3.0);
        assert!(
            tail(&folded) > 1e3 * tail(&lost),
            "{} vs {}",
            tail(&folded),
            tail(&lost)
        );
    }
}
