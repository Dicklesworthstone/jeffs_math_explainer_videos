//! The video's musical identity (plan §7.5): written for voice `michael`, to
//! be auditioned and tuned once narration exists (drafts use the scratch
//! voice and no music; nothing here has been listened to).
//!
//! The idea in sound: the motif sounds the two ends and then the note
//! between them, 1̂ 5̂ 3̂, as a middle junction settles between its
//! neighbours; the resolution walks home 3̂ 2̂ 1̂. E-flat Lydian, 74 beats a
//! minute, a pad bed with a felt-piano melody and a marimba answer.

use explainer_score::instr::Instrument;
use explainer_score::score::{BASS_UNDER_PAD_DB, Identity, Palette};
use explainer_score::theory::{Chord, Key, LYDIAN, Motif};

/// E-flat, as a pitch class.
const E_FLAT: i32 = 3;

pub fn identity() -> Identity {
    Identity {
        name: "resistor_random_walk".into(),
        key: Key::new(E_FLAT, LYDIAN),
        tonic_midi: 63,
        // I, V, vi, II of E-flat Lydian: E♭maj9, B♭maj7, Cm9, Fsus4
        progression: vec![Chord::maj9(E_FLAT), Chord::maj7(10), Chord::min9(0), Chord::sus4(5)],
        motif: Motif::new(&[1, 5, 3], &[0.75, 0.75, 1.5]),
        resolution: Motif::new(&[3, 2, 1], &[0.75, 0.75, 2.0]),
        beat_s: 60.0 / 74.0,
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
        assert_eq!(id.name, "resistor_random_walk");
        assert_eq!(id.progression[0].root, id.key.tonic);
    }

    /// The motif names both ends before the note between them, and the
    /// resolution lands on 1̂.
    #[test]
    fn the_motif_settles_between_its_ends_and_the_resolution_lands_on_the_tonic() {
        let id = identity();
        assert_eq!(id.motif.degrees, vec![1, 5, 3]);
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
                assert!(scale.contains(&pc), "{chord:?} holds {pc}, outside E-flat Lydian");
            }
        }
    }
}
