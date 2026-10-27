//! The video's musical identity (plan §7.5): written for voice `michael`, to
//! be auditioned and tuned once narration exists (drafts use the scratch
//! voice and no music; nothing here has been listened to).
//!
//! The idea in sound: the motif climbs 1̂ 2̂ 3̂ and leaps to 5̂, a count that
//! keeps going past what was seen; the resolution walks home 5̂ 3̂ 2̂ 1̂. D
//! major, 72 beats a minute, a pad bed with a felt-piano melody and a marimba
//! answer.

use explainer_score::instr::Instrument;
use explainer_score::score::{BASS_UNDER_PAD_DB, Identity, Palette};
use explainer_score::theory::{Chord, IONIAN, Key, Motif};

/// D, as a pitch class.
const D: i32 = 2;

pub fn identity() -> Identity {
    Identity {
        name: "german_tank".into(),
        key: Key::new(D, IONIAN),
        tonic_midi: 62,
        // I, IV, vi, V of D major: Dmaj9, Gmaj7, Bm9, Asus4
        progression: vec![Chord::maj9(D), Chord::maj7(7), Chord::min9(11), Chord::sus4(9)],
        motif: Motif::new(&[1, 2, 3, 5], &[0.5, 0.5, 0.5, 1.5]),
        resolution: Motif::new(&[5, 3, 2, 1], &[0.5, 0.5, 0.75, 2.0]),
        beat_s: 60.0 / 72.0,
        palette: Palette {
            bed: Instrument::Pad { brightness: 0.25 },
            melody: Instrument::FeltPiano,
            answer: Instrument::Marimba,
            roll: None,
        },
        chord_s: (6.0, 12.0),
        bed_t60: 1.8,
        // the corpus's balance on michael's voice (Video E's identity)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_identity_is_one_the_engine_can_play() {
        let id = identity();
        id.validate().unwrap();
        assert_eq!(id.name, "german_tank");
        assert_eq!(id.progression[0].root, id.key.tonic);
    }

    /// The motif climbs before it leaps, and the resolution lands on 1̂.
    #[test]
    fn the_motif_climbs_then_leaps_and_the_resolution_lands_on_the_tonic() {
        let id = identity();
        assert_eq!(id.motif.degrees, vec![1, 2, 3, 5]);
        assert_eq!(id.resolution.degrees.last(), Some(&1));
    }

    /// Every chord's tones are in the key, so nothing in the bed is foreign
    /// to the tune.
    #[test]
    fn every_chord_keeps_to_the_key() {
        let id = identity();
        let scale = id.key.pitch_classes();
        for chord in &id.progression {
            for pc in chord.pitch_classes() {
                assert!(scale.contains(&pc), "{chord:?} holds {pc}, outside D major");
            }
        }
    }
}
