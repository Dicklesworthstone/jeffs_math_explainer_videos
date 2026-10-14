//! Video E's musical identity (plan §7.5): written for voice `michael`, to
//! be auditioned and tuned by the score bead once the narration exists (the
//! drafts of P5 use the scratch voice and no music).
//!
//! The idea in sound: two notes a step apart that swap places. The motif
//! rises 5̂ to 6̂ and at once answers 6̂ to 5̂, the same two notes in the other
//! order, as heads-tails answers tails-heads; the video resolves when the
//! pair settles on 1̂. G major, unhurried, a pad bed with a felt-piano
//! melody and a kalimba answer (a small struck thing, like a coin on a
//! table).

use explainer_score::instr::Instrument;
use explainer_score::score::{BASS_UNDER_PAD_DB, Identity, Palette};
use explainer_score::theory::{Chord, IONIAN, Key, Motif};

pub fn identity() -> Identity {
    Identity {
        name: "fair_coin".into(),
        key: Key::new(7, IONIAN),
        tonic_midi: 67,
        progression: vec![Chord::maj9(7), Chord::min9(4), Chord::maj9(0), Chord::dom7sus4(2)],
        motif: Motif::new(&[5, 6, 6, 5], &[0.5, 1.0, 0.5, 1.5]),
        resolution: Motif::new(&[2, 1], &[1.0, 2.0]),
        beat_s: 60.0 / 84.0,
        palette: Palette {
            bed: Instrument::Pad { brightness: 0.3 },
            melody: Instrument::FeltPiano,
            answer: Instrument::Kalimba,
            roll: Some(Instrument::FeltPiano),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_identity_is_one_the_engine_can_play() {
        let id = identity();
        id.validate().unwrap();
        assert_eq!(id.name, "fair_coin");
        assert_eq!(id.progression[0].root, id.key.tonic);
        assert_eq!(id.progression[0].bass, id.key.tonic);
    }

    /// The motif's two pairs of notes swap places (5̂–6̂ then 6̂–5̂), and the
    /// resolution settles on 1̂.
    #[test]
    fn the_motif_swaps_two_notes_and_the_resolution_lands_on_the_tonic() {
        let id = identity();
        assert_eq!(id.motif.degrees, vec![5, 6, 6, 5]);
        assert_eq!(id.resolution.degrees, vec![2, 1]);
    }
}
