//! Provisional musical identity; score QA and listening remain required.
use explainer_score::instr::Instrument;
use explainer_score::score::{BASS_UNDER_PAD_DB, Identity, Palette};
use explainer_score::theory::{Chord, DORIAN, Key, Motif};

pub fn identity() -> Identity {
    Identity {
        name: "algorithm_shapes".into(),
        key: Key::new(2, DORIAN),
        tonic_midi: 50,
        progression: vec![Chord::min9(2), Chord::maj9(0), Chord::min9(4), Chord::dom7sus4(7)],
        motif: Motif::new(&[1, 2, 5, 3], &[0.5, 0.5, 1.0, 1.0]),
        resolution: Motif::new(&[5, 3, 2, 1], &[0.5, 0.5, 0.75, 1.5]),
        beat_s: 60.0 / 76.0,
        palette: Palette {
            bed: Instrument::Pad { brightness: 0.22 },
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
