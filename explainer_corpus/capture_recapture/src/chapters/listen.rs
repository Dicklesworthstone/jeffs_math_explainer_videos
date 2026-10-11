//! What the viewer hears in Video A's listen windows (plan §7.2; the music
//! lane's contract, mx-e-a-8y5.13): each sound is built from the very data
//! the picture shows at the same moments, written as a WAV for
//! `Kit::sonify`, which the score plays at the identity's listen level.
//! Pitches stay on F major pentatonic (the bed holds F6/9 under a window),
//! mallets no higher than D5, nothing that matters in a window's last 0.8 s
//! (the score fades it), and every file near the narration's −23 LUFS.

use std::io;
use std::path::PathBuf;

use explainer_score::instr::{Instrument, Note, render};
use explainer_score::sonify::{Grain, cloud, snap};
use explainer_score::{FS, Stereo};

/// F major pentatonic (pitch classes, C = 0): F G A C D.
pub const PENTATONIC_F: [u8; 5] = [0, 2, 5, 7, 9];
/// The highest mallet note (D5): higher bars ring past 7 kHz, which the
/// score's click gate catches in a quiet window.
pub const MALLET_TOP: f64 = 74.0;
/// The score fades the last this-many seconds of every window.
pub const SCORE_FADE_S: f64 = 0.8;
/// Where a window's sound must have stopped starting notes.
pub fn last_onset(window: f64) -> f64 {
    window - SCORE_FADE_S - 0.2
}

/// One fish heard: when it lands (seconds into the window) and whether it
/// is marked.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plink {
    pub at: f64,
    pub marked: bool,
}

/// The pitch of the `i`th plink: unmarked fish walk up and down F G A C
/// (kalimba, a gentle figure), every marked one rings D5 on the vibraphone.
pub fn plink_pitch(i: usize, marked: bool) -> f64 {
    if marked {
        MALLET_TOP
    } else {
        const WALK: [f64; 6] = [65.0, 67.0, 69.0, 72.0, 69.0, 67.0];
        snap(WALK[i % WALK.len()], &PENTATONIC_F)
    }
}

/// A catch, heard in catch order (03_5, 05_1): a kalimba note for each
/// unmarked fish, a vibraphone bell for each marked one, at the moments
/// the picture shows them.
pub fn catch_sound(label: &str, plinks: &[Plink], window: f64) -> io::Result<PathBuf> {
    let mut out = Stereo::silence((window * FS).round() as usize);
    for (i, p) in plinks.iter().enumerate() {
        let (inst, velocity, gain) = if p.marked {
            (Instrument::Vibraphone, 0.8, 0.7)
        } else {
            (Instrument::Kalimba, 0.55, 0.45)
        };
        let mut note = Note::new(plink_pitch(i, p.marked), velocity, 0.12, 300 + i as u64);
        note.pan = 0.5 * ((i * 7919 % 200) as f64 / 100.0 - 1.0);
        out.mix_in(&render(inst, &note), (p.at * FS).round() as usize, gain);
    }
    to_listen_level(&mut out);
    explainer_kit::sound_file(label, &out)
}

/// The pour (04_5): one soft grain per netful as it lands, at its pile's
/// note (pile k on the k-th note of the scale up from F4), so the piles'
/// sizes are heard as the rain's weight on each note.
pub fn pour_sound(label: &str, landings: &[(f64, usize)], window: f64) -> io::Result<PathBuf> {
    let notes = pile_notes(5);
    let grains: Vec<Grain> = landings
        .iter()
        .enumerate()
        .map(|(i, &(at, pile))| Grain {
            start: at,
            pitch: notes[pile.min(notes.len() - 1)],
            pan: 0.7 * ((i * 104_729 % 200) as f64 / 100.0 - 1.0),
            amp: 0.04,
        })
        .collect();
    let mut out = cloud(&grains, window);
    to_listen_level(&mut out);
    explainer_kit::sound_file(label, &out)
}

/// The piles' notes: F4 and up the pentatonic scale.
pub fn pile_notes(piles: usize) -> Vec<f64> {
    let mut out = Vec::with_capacity(piles);
    let mut p = 65.0;
    while out.len() < piles {
        let q = snap(p, &PENTATONIC_F);
        if out.last() != Some(&q) {
            out.push(q);
        }
        p += 1.0;
    }
    out
}

/// The sweep (05_5): one soft pad tone per lake size as the sweep passes
/// it, louder and brighter the higher that size's score (`score / peak`),
/// so the tone swells to the peak and fades after it.
pub fn sweep_sound(label: &str, passes: &[(f64, f64)], window: f64) -> io::Result<PathBuf> {
    let mut out = Stereo::silence((window * FS).round() as usize);
    for (i, &(at, height)) in passes.iter().enumerate() {
        let h = height.clamp(0.0, 1.0);
        let note = Note::new(65.0, 0.15 + 0.6 * h, 0.3, 500 + i as u64);
        out.mix_in(
            &render(
                Instrument::Pad {
                    brightness: 0.1 + 0.8 * h,
                },
                &note,
            ),
            (at * FS).round() as usize,
            0.2 + 0.8 * h,
        );
    }
    to_listen_level(&mut out);
    explainer_kit::sound_file(label, &out)
}

/// Bring a sonification to about the narration's −23 LUFS (the score
/// re-levels it; this only puts it in range).
fn to_listen_level(out: &mut Stereo) {
    let lufs = explainer_score::meter::integrated(out);
    if lufs.is_finite() {
        out.scale(10f64.powf((-23.0 - lufs) / 20.0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pitch_is_on_the_scale_and_under_the_mallet_ceiling() {
        for i in 0..12 {
            for marked in [false, true] {
                let p = plink_pitch(i, marked);
                assert!(p <= MALLET_TOP);
                assert!(PENTATONIC_F.contains(&((p as i64).rem_euclid(12) as u8)), "{p}");
            }
        }
        let piles = pile_notes(5);
        assert_eq!(piles, [65.0, 67.0, 69.0, 72.0, 74.0]);
        assert!(piles.iter().all(|&p| p <= MALLET_TOP));
    }

    #[test]
    fn a_catch_sounds_once_per_fish_and_marks_its_bells() {
        let plinks: Vec<Plink> = (0..10)
            .map(|i| Plink {
                at: 0.3 * i as f64,
                marked: [2, 5, 8].contains(&i),
            })
            .collect();
        let path = catch_sound("test_catch", &plinks, 4.0).expect("writes");
        assert!(path.exists());
        let bells = (0..plinks.len())
            .filter(|&i| plink_pitch(i, plinks[i].marked) == MALLET_TOP)
            .count();
        assert_eq!(bells, 3);
    }
}
