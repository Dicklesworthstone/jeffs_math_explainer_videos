//! The video's musical identity (plan §7.5): written for voice `michael`, to
//! be auditioned and tuned once narration exists (drafts use the scratch
//! voice and no music; nothing here has been listened to).
//!
//! The idea in sound: a toss. The motif jumps up and back down and up
//! again (5̂ 3̂ 5̂ 2̂), a coin turning over, and stops short of home; the one
//! resolution, 3̂ 2̂ 1̂, waits for the end. E-flat major (Ionian) at 74 beats a
//! minute: a pad bed (brightness 0.25), a marimba melody, a plucked string
//! answer (small struck things), and no roll.
//!
//! **Nothing in the bed rubs against the melody.** Every chord is drawn from
//! the major pentatonic (E♭ F G B♭ C): a sixth-ninth on the tonic, a minor
//! seventh on C, a seventh-sus on F and a plain sus on B♭. No two tones of
//! a chord are a semitone apart, and the motif and resolution keep to the
//! same five notes.

use explainer_score::instr::Instrument;
use explainer_score::score::{BASS_UNDER_PAD_DB, Identity, Palette};
use explainer_score::theory::{Chord, IONIAN, Key, Motif};

/// E-flat, as a pitch class.
const E_FLAT: i32 = 3;

pub fn identity() -> Identity {
    Identity {
        name: "freivalds".into(),
        key: Key::new(E_FLAT, IONIAN),
        tonic_midi: 63,
        progression: vec![
            // I: E♭ G B♭ C F
            Chord::six_nine(E_FLAT),
            // vi: C E♭ G B♭
            Chord::new(0, &[0, 3, 7, 10]),
            // ii: F B♭ C E♭, a seventh-sus with no third
            Chord::dom7sus4(5),
            // V: B♭ E♭ F, a plain sus
            Chord::sus4(10),
        ],
        motif: Motif::new(&[5, 3, 5, 2], &[0.5, 0.5, 0.5, 1.5]),
        resolution: Motif::new(&[3, 2, 1], &[0.75, 0.75, 2.0]),
        beat_s: 60.0 / 74.0,
        palette: Palette {
            bed: Instrument::Pad { brightness: 0.25 },
            melody: Instrument::Marimba,
            answer: Instrument::Pluck,
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

    fn pitches(id: &Identity, motif: &Motif) -> Vec<i32> {
        motif.degrees.iter().map(|&d| id.key.degree(d, id.tonic_midi)).collect()
    }

    #[test]
    fn the_identity_is_one_the_engine_can_play() {
        let id = identity();
        id.validate().unwrap();
        assert_eq!(id.name, "freivalds");
        assert_eq!(id.progression[0].root, id.key.tonic);
    }

    /// The motif turns over (B♭ G B♭ F) and the resolution lands on the
    /// tonic; every note of the tune and every chord tone is pentatonic,
    /// and no chord holds a semitone.
    #[test]
    fn the_tune_and_the_chords_keep_to_the_pentatonic() {
        let id = identity();
        assert_eq!(pitches(&id, &id.motif), vec![70, 67, 70, 65], "B♭ G B♭ F");
        assert_eq!(pitches(&id, &id.resolution), vec![67, 65, 63], "G F E♭");
        let pentatonic = [3, 5, 7, 10, 0];
        for p in pitches(&id, &id.motif).iter().chain(&pitches(&id, &id.resolution)) {
            assert!(pentatonic.contains(&p.rem_euclid(12)), "{p}");
        }
        for chord in &id.progression {
            let pcs = chord.pitch_classes();
            assert!(pcs.len() >= 3, "{chord:?}");
            for (i, &a) in pcs.iter().enumerate() {
                assert!(pentatonic.contains(&a.rem_euclid(12)), "{chord:?} holds {a}");
                for &b in &pcs[i + 1..] {
                    assert!(
                        ![1, 11].contains(&(a - b).rem_euclid(12)),
                        "{chord:?} holds {a} and {b}"
                    );
                }
            }
        }
    }
}
