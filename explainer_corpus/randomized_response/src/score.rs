//! The video's musical identity (plan §7.5): written for voice `michael`, to
//! be auditioned and tuned once narration exists (drafts use the scratch
//! voice and no music; nothing here has been listened to).
//!
//! The idea in sound: a coin's toss and fall. The motif leaps up a fifth and
//! steps back down (5̂ 3̂ 1̂ 2̂), left open like an answer nobody can trust;
//! the resolution steps down to the tonic (3̂ 2̂ 1̂), as the class's count
//! comes back. G Dorian at 84 beats a minute, a soft pad with a marimba
//! melody (small struck things, like coins) and a felt-piano answer.

use explainer_score::instr::Instrument;
use explainer_score::score::{BASS_UNDER_PAD_DB, Identity, Palette};
use explainer_score::theory::{Chord, DORIAN, Key, Motif};

/// G, as a pitch class.
const G: i32 = 7;

pub fn identity() -> Identity {
    Identity {
        name: "randomized_response".into(),
        key: Key::new(G, DORIAN),
        // G4, above the narrator's pitch band
        tonic_midi: 67,
        progression: vec![Chord::min9(G), Chord::dom9(0), Chord::maj9(10), Chord::min9(2)],
        motif: Motif::new(&[5, 3, 1, 2], &[0.5, 0.5, 1.0, 1.5]),
        resolution: Motif::new(&[3, 2, 1], &[0.75, 0.75, 2.0]),
        beat_s: 60.0 / 84.0,
        palette: Palette {
            bed: Instrument::Pad { brightness: 0.25 },
            melody: Instrument::Marimba,
            answer: Instrument::FeltPiano,
            roll: Some(Instrument::Marimba),
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
        assert_eq!(id.name, "randomized_response");
        assert_eq!(id.progression[0].root, id.key.tonic);
    }

    /// The motif leaps and is left open; the resolution lands on 1̂.
    #[test]
    fn the_motif_is_left_open_and_the_resolution_lands_on_the_tonic() {
        let id = identity();
        assert_ne!(id.motif.degrees.last(), Some(&1));
        assert_eq!(id.resolution.degrees.last(), Some(&1));
    }
}
