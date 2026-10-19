//! The video's musical identity (plan §7.5): written for voice `michael`, to
//! be auditioned and tuned by a score task once accepted narration exists
//! (drafts use the scratch voice and no music; nothing here has been
//! listened to).
//!
//! The idea in sound: the motif climbs 2̂ 3̂ 5̂ 6̂ toward the octave and stops
//! one step short, left open, as the curves get closer and the guarantee is
//! still missing; the resolution 6̂ 8̂ arrives. G major, unhurried and
//! clear: a soft pad bed, a felt-piano melody, a vibraphone answer. The
//! progression (I maj9, vi min9, V sus4, ii sus4) holds no tone a semitone
//! from any note of the motif.

use explainer_score::instr::Instrument;
use explainer_score::score::{BASS_UNDER_PAD_DB, Identity, Palette, Theme};
use explainer_score::theory::{Chord, IONIAN, Key, Motif};

pub fn identity() -> Identity {
    Identity {
        name: "epsilon_delta".into(),
        key: Key::new(7, IONIAN),
        tonic_midi: 67,
        progression: vec![Chord::maj9(7), Chord::min9(4), Chord::sus4(2), Chord::sus4(9)],
        motif: Motif::new(&[2, 3, 5, 6], &[1.0, 0.5, 0.5, 1.5]),
        resolution: Motif::new(&[6, 8], &[1.0, 2.5]),
        beat_s: 60.0 / 74.0,
        palette: Palette {
            bed: Instrument::Pad { brightness: 0.26 },
            melody: Instrument::FeltPiano,
            answer: Instrument::Vibraphone,
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
        steady_bed: true,
        settle_s: 0.8,
        sfx_lu: -16.0,
        listen_lu: -10.0,
        swing: 0.5,
        // one chapter so far, on the identity's own progression
        moods: Vec::new(),
        listen_chord: None,
        theme_chord: None,
        // the title card's soundless listen window carries the motif, left open
        themes: vec![("01_8", Theme::Open)],
    }
}

#[cfg(test)]
mod tests {
    use explainer_score::score::clashes;

    use super::*;
    use crate::chapters::limit_that_jumps::TITLE_LABEL;

    #[test]
    fn the_identity_is_one_the_engine_can_play() {
        let id = identity();
        id.validate().unwrap();
        assert_eq!(id.name, "epsilon_delta");
        assert_eq!(id.progression[0].root, id.key.tonic);
        assert_eq!(id.progression[0].bass, id.key.tonic);
    }

    /// The storyboard's listen column names the motif windows; the
    /// identity's themes are exactly those, and the chapter's listen label
    /// is the one the score answers.
    #[test]
    fn the_themes_are_the_storyboards_motif_windows() {
        let mut want = Vec::new();
        for row in crate::STORYBOARD.lines().filter(|l| l.starts_with("| 0")) {
            let cells: Vec<&str> = row.split('|').map(str::trim).collect();
            let (id, listen) = (cells[1], cells[cells.len() - 2]);
            if listen.contains("(the motif: left open)") {
                want.push((id.to_string(), Theme::Open));
            }
        }
        let id = identity();
        let have: Vec<(String, Theme)> = id.themes.iter().map(|&(l, t)| (l.to_string(), t)).collect();
        assert_eq!(have, want);
        assert_eq!(id.theme_for(TITLE_LABEL), Some(Theme::Open));
    }

    /// No note of the motif is a semitone from a tone of a chord it sounds
    /// over (the chapter plays the identity's own progression).
    #[test]
    fn no_motif_note_is_a_minor_ninth_from_a_chord_it_sounds_over() {
        let id = identity();
        for &d in &id.motif.degrees {
            let pitch = id.key.degree(d, id.tonic_midi);
            for chord in id.progression_for("01_a_limit_that_jumps") {
                assert!(!clashes(pitch, &chord), "motif note {pitch} against {chord:?}");
            }
        }
    }
}
