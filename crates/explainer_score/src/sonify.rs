//! Sonifications for listen windows (plan §7.2 and §7.5's signature
//! sonifications): data the viewer hears. A grain cloud turns many values
//! into one texture (Hoeffding's 2,000 shuffles, a Monte Carlo run); the
//! scene writes it as a WAV, its listen window carries it in the timeline,
//! and the score plays it there at the identity's listen level.

use std::collections::HashMap;
use std::f64::consts::TAU;

use crate::fx::pan;
use crate::units::midi_to_hz;
use crate::{FS, Stereo};

/// One grain: its start (seconds), MIDI pitch, pan (−1..1) and amplitude.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grain {
    pub start: f64,
    pub pitch: f64,
    pub pan: f64,
    pub amp: f64,
}

/// A grain's length: long enough to have a pitch, short enough that
/// hundreds a second stay a texture.
pub const GRAIN_S: f64 = 0.07;

/// A glassy grain: a Hann-windowed sine with two soft, slightly stretched
/// partials (as a small bell's are), `seconds` long, peak about 1. The
/// window's slow rise keeps every onset far above the engine's attack floor.
pub fn grain(pitch: f64, seconds: f64) -> Vec<f64> {
    let f = midi_to_hz(pitch);
    let n = (seconds * FS).round().max(2.0) as usize;
    (0..n)
        .map(|i| {
            let t = i as f64 / FS;
            let w = 0.5 - 0.5 * (TAU * i as f64 / (n - 1) as f64).cos();
            let y = (TAU * f * t).sin() + 0.18 * (TAU * 2.01 * f * t).sin() + 0.06 * (TAU * 3.03 * f * t).sin();
            w * y / 1.24
        })
        .collect()
}

/// The grains mixed into a buffer `seconds` long; a grain running past the
/// end is cut.
pub fn cloud(grains: &[Grain], seconds: f64) -> Stereo {
    let mut out = Stereo::silence((seconds * FS).round() as usize);
    let mut shapes: HashMap<u64, Vec<f64>> = HashMap::new();
    for g in grains {
        let mono = shapes
            .entry(g.pitch.to_bits())
            .or_insert_with(|| grain(g.pitch, GRAIN_S));
        let at = (g.start.max(0.0) * FS).round() as usize;
        out.mix_in(&pan(mono, g.pan), at, g.amp);
    }
    out
}

/// The pitch nearest `pitch` whose pitch class is one of `classes` (0 = C);
/// a tie goes down. Data mapped to pitch stays in the video's key this way.
pub fn snap(pitch: f64, classes: &[u8]) -> f64 {
    assert!(
        classes.iter().all(|&c| c < 12) && !classes.is_empty(),
        "pitch classes 0..12"
    );
    let base = pitch.round() as i64;
    (0..=6_i64)
        .flat_map(|d| [base - d, base + d])
        .filter(|m| classes.contains(&(m.rem_euclid(12) as u8)))
        .min_by(|a, b| {
            (*a as f64 - pitch)
                .abs()
                .total_cmp(&(*b as f64 - pitch).abs())
                .then(a.cmp(b))
        })
        .map_or(pitch, |m| m as f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meter;

    #[test]
    fn a_grain_has_its_pitch_and_no_click() {
        let g = grain(81.0, GRAIN_S); // A5, 880 Hz
        assert_eq!(g.len(), (GRAIN_S * FS).round() as usize);
        assert!(
            g[0].abs() < 1e-9 && g[g.len() - 1].abs() < 1e-9,
            "starts and ends at rest"
        );
        assert!(g.iter().all(|x| x.abs() <= 1.0));
        // zero crossings in the middle third: about 2 × 880 per second
        let mid = &g[g.len() / 3..2 * g.len() / 3];
        let crossings = mid.windows(2).filter(|w| w[0] < 0.0 && w[1] >= 0.0).count() as f64;
        let hz = crossings / (mid.len() as f64 / FS);
        assert!((hz - 880.0).abs() < 60.0, "{hz} Hz");
        // hard rule 4: no onset faster than a 3 ms raised cosine; the grain
        // stays under that ramp sample by sample (its window rises over 35 ms)
        let peak = g.iter().fold(0.0_f64, |m, x| m.max(x.abs()));
        for (i, x) in g[..(0.003 * FS) as usize].iter().enumerate() {
            let ramp = crate::fx::rcos(i as f64 / (0.003 * FS));
            assert!(x.abs() <= peak * ramp + 1e-12, "sample {i}: {x} over the 3 ms ramp");
        }
    }

    #[test]
    fn a_cloud_places_grains_and_cuts_at_the_end() {
        let g = |start: f64| Grain {
            start,
            pitch: 76.0,
            pan: 0.0,
            amp: 0.5,
        };
        let c = cloud(&[g(0.1), g(0.5), g(0.98)], 1.0);
        assert_eq!(c.len(), 48_000);
        assert!(
            c.l[..(0.1 * FS) as usize].iter().all(|&x| x == 0.0),
            "silent before the first grain"
        );
        let mid = (0.535 * FS) as usize;
        assert!(c.l[mid].abs() > 0.0, "the second grain sounds at 0.5 s");
        let quiet = &c.l[(0.2 * FS) as usize..(0.49 * FS) as usize];
        assert!(quiet.iter().all(|&x| x == 0.0), "nothing between grains");
        // the last grain is cut at the buffer's end, not appended
        assert!(meter::integrated(&c).is_finite());
        // pans: hard left is silent on the right
        let left = cloud(&[Grain { pan: -1.0, ..g(0.1) }], 0.5);
        assert!(left.r.iter().all(|&x| x.abs() < 1e-12));
        assert!(left.l.iter().any(|&x| x.abs() > 0.1));
    }

    #[test]
    fn snapping_keeps_data_in_key() {
        let d_pentatonic = [2, 4, 6, 9, 11]; // D E F♯ A B
        assert_eq!(snap(74.0, &d_pentatonic), 74.0); // D5 stays
        assert_eq!(snap(75.2, &d_pentatonic), 76.0); // E♭ + → E
        assert_eq!(snap(79.0, &d_pentatonic), 78.0); // G → F♯
        assert_eq!(snap(80.0, &[9, 7]), 79.0); // G♯ between G and A: a tie goes down
        assert_eq!(snap(79.6, &d_pentatonic), 81.0); // nearer A
        assert_eq!(snap(60.0, &[2]), 62.0); // C → D
        for k in 0..240 {
            let p = 50.0 + k as f64 * 0.17;
            let s = snap(p, &d_pentatonic);
            assert!(d_pentatonic.contains(&((s as i64).rem_euclid(12) as u8)));
            assert!((s - p).abs() <= 2.0, "{p} -> {s}");
        }
    }
}
