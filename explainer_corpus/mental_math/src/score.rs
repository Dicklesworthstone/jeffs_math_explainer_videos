//! Video F's musical identity (plan §7.5): written for voice `michael`, to
//! be auditioned and tuned by the score bead once the narration exists (the
//! drafts of P5 use the scratch voice and no music).
//!
//! The idea in sound: a question and its answer. The motif climbs three
//! steps and stops on 5̂, unfinished, as a quiz card does; the resolution
//! walks back down to 1̂ when the answer lands. D major, a little brisker
//! than the corpus's other videos, a light pad bed with a marimba melody
//! and a plucked answer (small struck things, like counting on fingers).
//! Each countdown is a listen window: the music alone, one mallet note a
//! second.

use explainer_score::instr::Instrument;
use explainer_score::score::{BASS_UNDER_PAD_DB, Identity, Palette, Theme};
use explainer_score::theory::{Chord, IONIAN, Key, Motif};

pub fn identity() -> Identity {
    Identity {
        name: "mental_math".into(),
        key: Key::new(2, IONIAN),
        tonic_midi: 62,
        progression: vec![Chord::maj9(2), Chord::min9(11), Chord::maj9(7), Chord::dom7sus4(9)],
        motif: Motif::new(&[1, 2, 3, 5], &[0.5, 0.5, 0.5, 1.5]),
        resolution: Motif::new(&[3, 2, 1], &[0.5, 0.5, 2.0]),
        beat_s: 60.0 / 96.0,
        palette: Palette {
            bed: Instrument::Pad { brightness: 0.3 },
            melody: Instrument::Marimba,
            answer: Instrument::Pluck,
            roll: Some(Instrument::Marimba),
        },
        chord_s: (6.0, 12.0),
        bed_t60: 1.6,
        // the corpus's balance on michael's voice (Video C), 1.5 LU lower:
        // at -13 / -13.5 the 3 s windows at the edges of 12's short quiz
        // questions measured 11.5 LU over the music (QA p5, limit 12) once
        // 09's ticks stopped capping the whole stem; at -14 / -14.5, 11.96
        // once 06_5.0 gained 70 ms and the music after it re-flowed
        // (2026-10-13)
        bed_lu: -14.5,
        bass_db: BASS_UNDER_PAD_DB,
        melody_lu: -15.0,
        duck_db: 8.0,
        hold_duck: 0.5,
        steady_bed: true,
        settle_s: 0.8,
        sfx_lu: -16.0,
        listen_lu: -10.0,
        swing: 0.5,
        moods: Vec::new(),
        listen_chord: None,
        // the title's motif and the closing resolution as written: over
        // Dmaj9 and Bm9 rule 7 moved the tune's D (a semitone from C#) to E,
        // so the motif began on 2 and the video ended on it (2026-10-14)
        theme_chord: Some(Chord::six_nine(2)),
        // the storyboard's two motif windows: the title (01_3) plays the
        // motif's question and leaves it open, the receipts (14_6) answer it.
        // Without them the title's gap played whichever variant of the motif
        // its seed drew (2026-10-14)
        themes: vec![("01_3", Theme::Open), ("14_6", Theme::Resolved)],
    }
}

#[cfg(test)]
mod tests {
    use explainer_score::score::avoid_minor_ninths;

    use super::*;

    /// The storyboard's listen column says which windows are the motif: "(the
    /// motif's question only, unanswered)" left open, "(the motif,
    /// answered)". The identity's themes are exactly those windows, and the
    /// chapters give `kit.listen` labels that carry their ids.
    #[test]
    fn the_themes_are_the_storyboards_motif_windows_and_the_chapters_labels() {
        let mut want = Vec::new();
        for row in crate::STORYBOARD.lines().filter(|l| l.starts_with("| ")) {
            let cells: Vec<&str> = row.split('|').map(str::trim).collect();
            let (id, listen) = (cells[1], cells[cells.len() - 2]);
            let Some((_, what)) = listen.strip_prefix("listen").and_then(|l| l.split_once('(')) else {
                continue;
            };
            if what.starts_with("the motif's question") {
                want.push((id.to_string(), Theme::Open));
            } else if what.starts_with("the motif, answered") {
                want.push((id.to_string(), Theme::Resolved));
            }
        }
        let id = identity();
        let have: Vec<(String, Theme)> = id.themes.iter().map(|&(label, t)| (label.to_string(), t)).collect();
        assert_eq!(
            have, want,
            "the identity's themes against the storyboard's motif windows"
        );
        assert_eq!(want.len(), 2, "{want:?}");
        assert!(include_str!("chapters/eighty_five_squared.rs").contains("\"01_eighty_five_squared:01_3\""));
        assert!(include_str!("chapters/near_something_you_know.rs").contains("\"14_near_something_you_know:14_6\""));
        assert_eq!(id.theme_for("01_eighty_five_squared:01_3"), Some(Theme::Open));
        assert_eq!(id.theme_for("14_near_something_you_know:14_6"), Some(Theme::Resolved));
    }

    /// The theme sounds as written under the theme chord: rule 7 moves none
    /// of its notes there. Over the progression's tonic chord, Dmaj9, it
    /// moves some (the tune's D is a semitone from the C# of Dmaj9 and Bm9: the motif began on E and the closing resolution ended on it).
    #[test]
    fn the_tune_sounds_as_written_over_the_theme_chord() {
        let id = identity();
        let theme = id.theme_chord.clone().expect("a theme chord");
        for p in id.tune() {
            assert_eq!(avoid_minor_ninths(p, &theme, &id.key, 0), p, "{p} over {theme:?}");
        }
        let tonic = &id.progression[0];
        assert!(
            id.tune().iter().any(|&p| avoid_minor_ninths(p, tonic, &id.key, 0) != p),
            "over {tonic:?} the tune would sound as written: the theme chord is not needed"
        );
    }

    #[test]
    fn the_identity_is_one_the_engine_can_play() {
        let id = identity();
        id.validate().unwrap();
        assert_eq!(id.name, "mental_math");
        assert_eq!(id.progression[0].root, id.key.tonic);
        assert_eq!(id.progression[0].bass, id.key.tonic);
    }

    /// The motif climbs to 5̂ and stops unfinished; the resolution walks
    /// back down to 1̂.
    #[test]
    fn the_motif_climbs_to_the_fifth_and_the_resolution_walks_back_to_the_tonic() {
        let id = identity();
        assert_eq!(id.motif.degrees, vec![1, 2, 3, 5]);
        assert_eq!(id.resolution.degrees, vec![3, 2, 1]);
    }
}
