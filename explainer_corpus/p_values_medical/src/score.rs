//! The video's musical identity (plan §7.5): written for voice `michael`, to
//! be auditioned and tuned by a score task once the narration exists (the
//! drafts use the scratch voice and no music).
//!
//! The idea in sound: the same notes mean something different over a
//! different bass, as the same count means something different over a
//! different group. The motif (5̂ 6̂ 5̂ 2̂) returns unchanged while the chords
//! under it move; it ends on 2̂, a question left open. F major, unhurried
//! and warm, curious rather than alarmed (research brief: no mocking or
//! alarmist music): a soft pad bed, felt-piano melody, vibraphone answer.

use explainer_score::instr::Instrument;
use explainer_score::score::{BASS_UNDER_PAD_DB, Identity, Palette, Theme};
use explainer_score::theory::{Chord, IONIAN, Key, Motif};

pub fn identity() -> Identity {
    Identity {
        name: "p_values_medical".into(),
        key: Key::new(5, IONIAN),
        tonic_midi: 65,
        progression: vec![Chord::maj9(5), Chord::min9(2), Chord::maj9(10), Chord::dom7sus4(0)],
        motif: Motif::new(&[5, 6, 5, 2], &[0.5, 0.5, 1.0, 1.5]),
        resolution: Motif::new(&[2, 1], &[1.0, 2.0]),
        beat_s: 60.0 / 76.0,
        palette: Palette {
            bed: Instrument::Pad { brightness: 0.28 },
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
        // one chapter so far, on the identity's own progression; the acts of
        // storyboard §2 get their moods as their chapters land
        moods: vec![],
        listen_chord: None,
        theme_chord: None,
        // the title card's soundless listen window carries the motif, left open
        themes: vec![("01_3", Theme::Open)],
    }
}

#[cfg(test)]
mod tests {
    use explainer_score::score::clashes;

    use super::*;
    use crate::chapters::five_percent::TITLE_LABEL;

    #[test]
    fn the_identity_is_one_the_engine_can_play() {
        let id = identity();
        id.validate().unwrap();
        assert_eq!(id.name, "p_values_medical");
        assert_eq!(id.progression[0].root, id.key.tonic);
        assert_eq!(id.progression[0].bass, id.key.tonic);
    }

    /// The storyboard's listen column names the motif windows; the identity's
    /// themes are exactly those, and the chapter's listen label is the one
    /// the score answers.
    #[test]
    fn the_themes_are_the_storyboards_motif_windows_and_the_chapters_labels() {
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
            for chord in id.progression_for("01_five_percent_of_what") {
                assert!(!clashes(pitch, &chord), "motif note {pitch} against {chord:?}");
            }
        }
    }
}
