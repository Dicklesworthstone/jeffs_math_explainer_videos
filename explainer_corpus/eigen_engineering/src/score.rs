//! Video K's musical identity (plan §7.5): written for voice `michael`, to
//! be auditioned and tuned by the score bead once the narration exists (the
//! drafts of P5 use the scratch voice and no music).
//!
//! The idea in sound: a mode is a tone a thing owns. The motif climbs the
//! overtones of one note (1̂, 5̂, 1̂ above, 3̂ above), the tones a single
//! string or bar already holds, and the resolution walks back to 1̂. E
//! major, unhurried, a pad bed with a vibraphone melody (struck metal bars
//! are modes you can hear) and a plucked answer. Each quiz countdown is a
//! listen window: the music alone, one soft vibraphone note a second
//! climbing the motif's overtones and left open, and the resolution walking
//! back as the answer lands (`chapters/sound.rs`, which also writes the
//! truss's modes at 07_3, the bathroom's hum at 09_5 and the hall's at
//! 10_6). The title card's window is the motif itself, left open; the
//! closing card's is the resolution.

use explainer_score::instr::Instrument;
use explainer_score::score::{BASS_UNDER_PAD_DB, Identity, Palette, Theme};
use explainer_score::theory::{Chord, IONIAN, Key, Motif};

pub fn identity() -> Identity {
    Identity {
        name: "eigen_engineering".into(),
        key: Key::new(4, IONIAN),
        tonic_midi: 64,
        progression: vec![Chord::maj9(4), Chord::min9(1), Chord::maj9(9), Chord::dom7sus4(11)],
        motif: Motif::new(&[1, 5, 8, 10], &[0.5, 0.5, 0.5, 1.5]),
        resolution: Motif::new(&[5, 3, 1], &[0.5, 0.5, 2.0]),
        beat_s: 60.0 / 84.0,
        palette: Palette {
            bed: Instrument::Pad { brightness: 0.3 },
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
        steady_bed: true,
        settle_s: 0.8,
        sfx_lu: -16.0,
        listen_lu: -10.0,
        swing: 0.5,
        moods: Vec::new(),
        listen_chord: None,
        // 01_3 "listen 2.5 s (the motif: the overtones of one note)" and
        // 14_8 "listen 2.5 s (the resolution)"
        themes: vec![("01_3", Theme::Open), ("14_8", Theme::Resolved)],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_identity_is_one_the_engine_can_play() {
        assert!(identity().validate().is_ok());
    }

    /// The title card's window plays the motif open and the closing card's
    /// the resolution; the quiz and sonification windows carry their own
    /// sound and name no theme.
    #[test]
    fn the_first_and_last_windows_are_the_motif_and_its_resolution() {
        let id = identity();
        assert_eq!(id.theme_for("01_the_bridge_that_swayed:01_3"), Some(Theme::Open));
        assert_eq!(id.theme_for("14_one_idea:14_8"), Some(Theme::Resolved));
        for label in [
            "02_two_carts:02_6",
            "07_every_mode_a_note:07_3",
            "09_the_air_in_a_room:09_5",
        ] {
            assert_eq!(id.theme_for(label), None, "{label}");
        }
    }
}
