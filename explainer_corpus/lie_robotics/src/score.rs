//! Video G's musical identity (plan §7.5): written for voice `michael`, to
//! be auditioned and tuned by the score bead once the narration exists (the
//! drafts of P5 use the scratch voice and no music).
//!
//! The idea in sound: a screw, a figure that climbs as it turns. The motif
//! rises through 1̂ 3̂ 5̂ and lands a step higher, on 6̂, where a plain arpeggio
//! would have come back round; the resolution winds down 5̂ 3̂ 1̂ when a
//! quiz's answer lands. F major, unhurried, a pad bed with an electric
//! piano melody and a vibraphone answer (machined and warm, like the arm).
//! Each countdown is a listen window: the music alone, one soft vibraphone
//! note a second climbing the motif and left open, and the resolution
//! winding down as the answer lands (`chapters/sound.rs`, built from this
//! identity's own motif and resolution). The title card's window is the
//! motif itself, left open.

use explainer_score::instr::Instrument;
use explainer_score::score::{BASS_UNDER_PAD_DB, Identity, Palette, Theme};
use explainer_score::theory::{Chord, IONIAN, Key, Motif};

pub fn identity() -> Identity {
    Identity {
        name: "lie_robotics".into(),
        key: Key::new(5, IONIAN),
        tonic_midi: 65,
        progression: vec![Chord::maj9(5), Chord::min9(2), Chord::maj9(10), Chord::dom7sus4(0)],
        motif: Motif::new(&[1, 3, 5, 6], &[0.5, 0.5, 0.5, 1.5]),
        resolution: Motif::new(&[5, 3, 1], &[0.5, 0.5, 2.0]),
        beat_s: 60.0 / 88.0,
        palette: Palette {
            bed: Instrument::Pad { brightness: 0.3 },
            melody: Instrument::FmEp,
            answer: Instrument::Vibraphone,
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
        // 01_3: "listen 3 s (the motif)"
        themes: vec![("01_3", Theme::Open)],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_identity_is_one_the_engine_can_play() {
        assert!(identity().validate().is_ok());
    }

    /// The title card's window plays the motif open; the quiz windows carry
    /// their own sound and name no theme.
    #[test]
    fn the_title_cards_window_plays_the_motif_open() {
        let id = identity();
        assert_eq!(id.theme_for("01_the_book:01_3"), Some(Theme::Open));
        assert_eq!(id.theme_for("01_the_book:01_1"), None);
        assert_eq!(id.theme_for("03_three_angles:03_2"), None);
    }
}
