//! The video's musical identity (plan §7.5): written for voice `michael`, to
//! be auditioned and tuned once narration exists (drafts use the scratch
//! voice and no music; nothing here has been listened to).
//!
//! The idea in sound: the motif climbs 1̂ 2̂ 3̂ and steps back, and the
//! resolution comes down 3̂ 2̂ 1̂, as the added rows come down to the echo
//! alone. D Dorian, a little dark (echoes, things unseen), a soft pad with a
//! vibraphone melody and a marimba answer (struck things, like pulses).

use explainer_score::instr::Instrument;
use explainer_score::score::{BASS_UNDER_PAD_DB, Identity, Palette};
use explainer_score::theory::{Chord, DORIAN, Key, Motif};

pub fn identity() -> Identity {
    Identity {
        name: "golay_echoes".into(),
        key: Key::new(2, DORIAN),
        tonic_midi: 62,
        progression: vec![Chord::min9(2), Chord::maj9(5), Chord::min9(9), Chord::dom7sus4(7)],
        motif: Motif::new(&[1, 2, 3, 2], &[0.5, 0.5, 1.0, 1.5]),
        resolution: Motif::new(&[3, 2, 1], &[0.75, 0.75, 2.0]),
        beat_s: 60.0 / 78.0,
        palette: Palette {
            bed: Instrument::Pad { brightness: 0.25 },
            melody: Instrument::Vibraphone,
            answer: Instrument::Marimba,
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
        assert_eq!(id.name, "golay_echoes");
        assert_eq!(id.progression[0].root, id.key.tonic);
    }

    /// The answer mirrors the motif's steps, and the resolution lands on 1̂.
    #[test]
    fn the_motif_climbs_and_the_resolution_lands_on_the_tonic() {
        let id = identity();
        assert_eq!(id.motif.degrees, vec![1, 2, 3, 2]);
        assert_eq!(id.resolution.degrees.last(), Some(&1));
    }
}
