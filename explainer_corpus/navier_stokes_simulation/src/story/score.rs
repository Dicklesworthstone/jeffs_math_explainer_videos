//! Restrained score for the measured wind-tunnel sequence.
use explainer_score::instr::Instrument;
use explainer_score::score::{BASS_UNDER_PAD_DB, Identity, Palette};
use explainer_score::theory::{AEOLIAN, Chord, Key, Motif};

pub fn identity() -> Identity {
    Identity {
        name: "navier_stokes_simulation".into(),
        key: Key::new(5, AEOLIAN),
        tonic_midi: 53,
        progression: vec![
            Chord::min9(5),
            Chord::maj9(1),
            Chord::min9(10),
            Chord::dom7sus4(0),
        ],
        motif: Motif::new(&[1, 5, 4, 3], &[0.5, 0.75, 0.75, 1.5]),
        resolution: Motif::new(&[5, 4, 2, 1], &[0.5, 0.75, 0.75, 2.0]),
        beat_s: 60.0 / 72.0,
        palette: Palette {
            bed: Instrument::Pad { brightness: 0.2 },
            melody: Instrument::Marimba,
            answer: Instrument::Vibraphone,
            roll: Some(Instrument::FeltPiano),
        },
        chord_s: (7.0, 13.0),
        bed_t60: 1.8,
        bed_lu: -13.0,
        bass_db: BASS_UNDER_PAD_DB,
        melody_lu: -14.0,
        duck_db: 9.0,
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
