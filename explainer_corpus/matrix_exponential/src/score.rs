//! Video J's musical identity (plan §7.5): written for voice `michael`, to
//! be auditioned and tuned by the score bead once the narration exists (the
//! drafts of P5 use the scratch voice and no music).
//!
//! The idea in sound: one small step, repeated, becomes a whole motion. The
//! motif is a single rising step taken three times, each landing a little
//! higher, the way compounding turns a nudge into growth; the resolution
//! comes round to where it began, as a rotation does. A Dorian (a minor
//! colour that keeps a bright sixth: neither the growth nor the decay side
//! of the exponential), unhurried, a warm pad with an electric piano melody
//! and a vibraphone answer. Each quiz countdown is a listen window: the
//! music alone, one soft note a second.

use explainer_score::instr::Instrument;
use explainer_score::score::{BASS_UNDER_PAD_DB, Identity, Palette};
use explainer_score::theory::{Chord, DORIAN, Key, Motif};

pub fn identity() -> Identity {
    Identity {
        name: "matrix_exponential".into(),
        key: Key::new(9, DORIAN),
        tonic_midi: 57,
        progression: vec![Chord::min9(9), Chord::maj9(0), Chord::min9(4), Chord::dom7sus4(2)],
        motif: Motif::new(&[1, 2, 2, 3, 3, 5], &[0.5, 1.0, 0.5, 1.0, 0.5, 1.5]),
        resolution: Motif::new(&[4, 3, 2, 1], &[0.5, 0.5, 0.5, 2.0]),
        beat_s: 60.0 / 84.0,
        palette: Palette {
            bed: Instrument::Pad { brightness: 0.3 },
            melody: Instrument::FmEp,
            answer: Instrument::Vibraphone,
            roll: Some(Instrument::Vibraphone),
        },
        chord_s: (6.0, 12.0),
        bed_t60: 1.8,
        // the corpus's balance on michael's voice (Video C): V − 21 under
        // speech, the gap music near V − 10
        bed_lu: -13.0,
        bass_db: BASS_UNDER_PAD_DB,
        melody_lu: -13.5,
        duck_db: 8.0,
        hold_duck: 0.5,
        settle_s: 0.8,
        sfx_lu: -16.0,
        listen_lu: -10.0,
        swing: 0.5,
        moods: Vec::new(),
        listen_chord: None,
        themes: Vec::new(),
    }
}
