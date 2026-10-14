//! Musical identity for `__SLUG__` (plan §7.5).

use explainer_score::instr::Instrument;
use explainer_score::score::{BASS_UNDER_PAD_DB, Identity, Palette};
use explainer_score::theory::{Chord, IONIAN, Key, Motif};

pub fn identity() -> Identity {
    Identity {
        name: "__SLUG__".into(),
        key: Key::new(0, IONIAN),
        tonic_midi: 60,
        progression: vec![Chord::maj9(0), Chord::min9(9), Chord::maj9(5), Chord::dom7sus4(7)],
        motif: Motif::new(&[1, 3, 5, 6], &[0.75, 0.75, 1.0, 1.5]),
        resolution: Motif::new(&[5, 3, 1], &[0.75, 0.75, 2.0]),
        beat_s: 60.0 / 80.0,
        palette: Palette {
            bed: Instrument::Pad { brightness: 0.3 },
            melody: Instrument::FeltPiano,
            answer: Instrument::Kalimba,
            roll: Some(Instrument::FeltPiano),
        },
        chord_s: (6.0, 12.0),
        bed_t60: 1.8,
        bed_lu: -13.0,
        bass_db: BASS_UNDER_PAD_DB,
        melody_lu: -13.5,
        duck_db: 8.0,
        hold_duck: 0.5,
        steady_bed: false,
        settle_s: 0.8,
        sfx_lu: -16.0,
        listen_lu: -10.0,
        swing: 0.5,
        moods: Vec::new(),
        listen_chord: None,
        themes: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn musical_identity_is_valid() {
        let id = identity();
        id.validate().expect("valid score identity");
        assert_eq!(id.name, "__SLUG__");
    }
}
