//! Video H's musical identity (plan §7.5): written for voice `michael`, to
//! be auditioned and tuned by the score bead once the narration exists (the
//! drafts of P5 use the scratch voice and no music).
//!
//! The idea in sound: a search that widens. The motif steps out from 1̂ and
//! comes back to it, then reaches one note further each time it returns,
//! as a search leaves its starting cell and comes back to the map; the
//! resolution settles on 1̂ from above when something is found. D Dorian,
//! unhurried and sober, a low pad bed with a vibraphone melody and a
//! plucked answer. Each countdown is a listen window: the music alone, one
//! mallet note a second.

use explainer_score::instr::Instrument;
use explainer_score::score::{BASS_UNDER_PAD_DB, Identity, Palette};
use explainer_score::theory::{Chord, DORIAN, Key, Motif};

pub fn identity() -> Identity {
    Identity {
        name: "bayesian_search".into(),
        key: Key::new(2, DORIAN),
        tonic_midi: 62,
        progression: vec![Chord::min9(2), Chord::maj9(5), Chord::min9(9), Chord::dom7sus4(7)],
        motif: Motif::new(&[1, 2, 1, 4], &[0.5, 0.5, 0.5, 1.5]),
        resolution: Motif::new(&[3, 2, 1], &[0.5, 0.5, 2.0]),
        beat_s: 60.0 / 76.0,
        palette: Palette {
            bed: Instrument::Pad { brightness: 0.25 },
            melody: Instrument::Vibraphone,
            answer: Instrument::Pluck,
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
