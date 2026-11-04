//! Provisional score identity. It has not been rendered or auditioned.
use explainer_score::instr::Instrument;
use explainer_score::score::{BASS_UNDER_PAD_DB, Identity, Palette};
use explainer_score::theory::{AEOLIAN, Chord, Key, Motif};

pub fn identity() -> Identity {
    Identity {
        name: "exponential_magic".into(),
        key: Key::new(9, AEOLIAN),
        tonic_midi: 57,
        progression: vec![Chord::min9(9), Chord::maj9(5), Chord::maj9(0), Chord::dom7sus4(4)],
        motif: Motif::new(&[1, 3, 2, 1], &[0.5, 0.5, 1.0, 1.5]),
        resolution: Motif::new(&[3, 2, 1], &[0.75, 0.75, 2.0]),
        beat_s: 60.0 / 74.0,
        palette: Palette {
            bed: Instrument::Pad { brightness: 0.25 },
            melody: Instrument::Marimba,
            answer: Instrument::Vibraphone,
            roll: Some(Instrument::FeltPiano),
        },
        chord_s: (6.0, 12.0),
        bed_t60: 1.8,
        bed_lu: -13.0,
        bass_db: BASS_UNDER_PAD_DB,
        melody_lu: -13.5,
        duck_db: 8.0,
        hold_duck: 0.5,
        steady_bed: true,
        settle_s: 0.8,
        sfx_lu: -16.0,
        listen_lu: -10.0,
        swing: 0.5,
        moods: Vec::new(),
        listen_chord: None,
        themes: Vec::new(),
        theme_chord: None,
    }
}
