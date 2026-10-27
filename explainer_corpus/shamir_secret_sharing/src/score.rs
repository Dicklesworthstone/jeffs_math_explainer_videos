//! The video's musical identity (plan §7.5): written before any narration
//! exists, to be auditioned and tuned once it does (drafts use the scratch
//! voice and no music; nothing here has been listened to).
//!
//! The idea in sound: a dial clicking round, and a lock that gives on the
//! third key. The motif climbs three steps (1̂ 3̂ 5̂) and stops one step past,
//! left open like two shares that say nothing; the resolution comes down to
//! the tonic (5̂ 2̂ 1̂) as three shares open the door. C Aeolian at 80 beats a
//! minute, a soft pad with a kalimba melody (small plucked tines, like a
//! dial's clicks) and a felt-piano answer.

use explainer_score::instr::Instrument;
use explainer_score::score::{BASS_UNDER_PAD_DB, Identity, Palette};
use explainer_score::theory::{AEOLIAN, Chord, Key, Motif};

/// C, A flat, E flat and B flat, as pitch classes.
const C: i32 = 0;
const A_FLAT: i32 = 8;
const E_FLAT: i32 = 3;
const B_FLAT: i32 = 10;

pub fn identity() -> Identity {
    Identity {
        name: "shamir_secret_sharing".into(),
        key: Key::new(C, AEOLIAN),
        // C5, above the narrator's pitch band
        tonic_midi: 72,
        progression: vec![
            Chord::min9(C),
            Chord::maj9(A_FLAT),
            Chord::maj9(E_FLAT),
            Chord::dom9(B_FLAT),
        ],
        motif: Motif::new(&[1, 3, 5, 6], &[0.5, 0.5, 1.0, 1.5]),
        resolution: Motif::new(&[5, 2, 1], &[0.75, 0.75, 2.0]),
        beat_s: 60.0 / 80.0,
        palette: Palette {
            bed: Instrument::Pad { brightness: 0.25 },
            melody: Instrument::Kalimba,
            answer: Instrument::FeltPiano,
            roll: Some(Instrument::Kalimba),
        },
        chord_s: (6.0, 12.0),
        bed_t60: 1.8,
        // the corpus's balance (randomized_response's identity), untuned here
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
        assert_eq!(id.name, "shamir_secret_sharing");
        assert_eq!(id.progression[0].root, id.key.tonic);
    }

    /// The motif is left open; the resolution lands on 1̂.
    #[test]
    fn the_motif_is_left_open_and_the_resolution_lands_on_the_tonic() {
        let id = identity();
        assert_ne!(id.motif.degrees.last(), Some(&1));
        assert_eq!(id.resolution.degrees.last(), Some(&1));
    }

    /// Every chord of the bed stays inside C Aeolian (no chromatic tone
    /// under the narration).
    #[test]
    fn the_progression_stays_in_the_key() {
        let id = identity();
        let scale = id.key.pitch_classes();
        for chord in &id.progression {
            for pc in chord.pitch_classes() {
                assert!(scale.contains(&pc), "{chord:?} has {pc}");
            }
        }
    }
}
