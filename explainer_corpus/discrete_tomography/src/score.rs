//! The video's musical identity (plan §7.5): written for voice `michael`, to
//! be auditioned and tuned by a score task once accepted narration exists
//! (drafts use the scratch voice and no music; nothing here has been
//! listened to).
//!
//! The idea in sound: the motif steps up and back down to where it began,
//! 1̂ 3̂ 2̂ 1̂, a different path to the same place, as different pictures
//! cast the same shadows. A minor, quiet and inquisitive: a soft pad bed,
//! marimba melody (squares placed one by one), vibraphone answer.

use explainer_score::instr::Instrument;
use explainer_score::score::{BASS_UNDER_PAD_DB, Identity, Palette};
use explainer_score::theory::{AEOLIAN, Chord, Key, Motif};

pub fn identity() -> Identity {
    Identity {
        name: "discrete_tomography".into(),
        key: Key::new(9, AEOLIAN),
        tonic_midi: 57,
        progression: vec![Chord::min9(9), Chord::maj9(5), Chord::maj9(0), Chord::dom7sus4(4)],
        motif: Motif::new(&[1, 3, 2, 1], &[0.5, 0.5, 1.0, 1.5]),
        resolution: Motif::new(&[3, 2, 1], &[0.75, 0.75, 2.0]),
        beat_s: 60.0 / 74.0,
        palette: Palette {
            bed: Instrument::Pad { brightness: 0.25 },
            melody: Instrument::Marimba,
            answer: Instrument::Vibraphone,
            roll: Some(Instrument::FeltPiano),
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
        assert_eq!(id.name, "discrete_tomography");
        assert_eq!(id.progression[0].root, id.key.tonic);
    }

    /// The motif returns to where it began, and the resolution lands on 1̂.
    #[test]
    fn the_motif_returns_and_the_resolution_lands_on_the_tonic() {
        let id = identity();
        assert_eq!(id.motif.degrees.first(), id.motif.degrees.last());
        assert_eq!(id.resolution.degrees.last(), Some(&1));
    }
}
