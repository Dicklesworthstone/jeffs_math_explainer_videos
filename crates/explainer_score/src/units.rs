//! Units and exact timeline arithmetic.

use crate::SAMPLE_RATE;

/// Decibels to linear amplitude.
pub fn db_to_gain(db: f64) -> f64 {
    10f64.powf(db / 20.0)
}

/// Linear amplitude to decibels (−∞ for zero).
pub fn gain_to_db(g: f64) -> f64 {
    20.0 * g.abs().log10()
}

/// MIDI note number to frequency (A4 = 69 = 440 Hz).
pub fn midi_to_hz(m: f64) -> f64 {
    440.0 * 2f64.powf((m - 69.0) / 12.0)
}

/// Frequency to (fractional) MIDI note number.
pub fn hz_to_midi(hz: f64) -> f64 {
    69.0 + 12.0 * (hz / 440.0).log2()
}

/// Frequency ratio of a detune in cents.
pub fn cents_ratio(cents: f64) -> f64 {
    2f64.powf(cents / 1200.0)
}

/// Round half away from zero (franken_manim's placement rounding).
pub fn round_half_away(x: f64) -> i64 {
    if x >= 0.0 {
        (x + 0.5).floor() as i64
    } else {
        -((-x + 0.5).floor() as i64)
    }
}

/// The first sample of video frame `frame` at `fps`, exactly as franken_manim
/// places audio: `round(frame * 48000 / fps)` in exact integer arithmetic.
pub fn frame_to_sample(frame: u64, fps: u32) -> u64 {
    let num = u128::from(frame) * u128::from(SAMPLE_RATE);
    let den = u128::from(fps);
    // round half away from zero for non-negative values = floor((2n + d) / 2d)
    ((2 * num + den) / (2 * den)) as u64
}

/// Seconds to the nearest sample index (round half away from zero).
pub fn seconds_to_sample(seconds: f64) -> i64 {
    round_half_away(seconds * f64::from(SAMPLE_RATE))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_map_to_exact_samples() {
        assert_eq!(frame_to_sample(1, 60), 800);
        assert_eq!(frame_to_sample(60, 60), 48_000);
        assert_eq!(frame_to_sample(5_100, 60), 4_080_000);
        assert_eq!(frame_to_sample(1, 30), 1_600);
        // 1 frame at a rate that does not divide 48 kHz rounds half away.
        assert_eq!(frame_to_sample(1, 7), 6_857); // 6857.14…
    }

    #[test]
    fn three_decimal_seconds_would_be_wrong() {
        // The reason cue logs must store integer frames, not "{:.3}" seconds:
        // one 60 fps frame printed as 0.017 s lands 16 samples late.
        let printed: f64 = format!("{:.3}", 1.0 / 60.0).parse().unwrap();
        assert_eq!(seconds_to_sample(printed), 816);
        assert_eq!(frame_to_sample(1, 60), 800);
    }

    #[test]
    fn decibels_and_pitch_round_trip() {
        for db in [-60.0, -18.0, 0.0, 6.0] {
            assert!((gain_to_db(db_to_gain(db)) - db).abs() < 1e-12);
        }
        assert!((midi_to_hz(69.0) - 440.0).abs() < 1e-12);
        assert!((hz_to_midi(midi_to_hz(47.0)) - 47.0).abs() < 1e-12);
        assert!((cents_ratio(1200.0) - 2.0).abs() < 1e-12);
        assert_eq!(round_half_away(2.5), 3);
        assert_eq!(round_half_away(-2.5), -3);
    }
}
