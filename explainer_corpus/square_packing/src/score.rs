//! The five-tile video's musical identity (plan §7.5): written for voice
//! `michael`, to be auditioned and tuned once the narration exists (the code-
//! first drafts use the scratch voice and no music).
//!
//! The idea in sound: a turn. The motif is the ornament of that name, the
//! note above, the note, the note below and the note again (3̂ 2̂ 1̂ 2̂): it
//! circles its note and lands on it without settling, as the middle tile
//! turns in place. Left open it asks the chapter's question in the think
//! window (01_4); the video's one resolution, 5̂ 3̂ 2̂ 1̂, waits for its end.
//! B-flat major at 80 beats a minute, a pad bed with a kalimba melody
//! (plucked tines, small pieces clicking into place) and a glassy answer.
//!
//! **Distinct from the corpus.** No other identity is in B-flat, none moves
//! at 80, none leads with the kalimba (Video E answers with it).
//!
//! **Nothing in the bed rubs against the melody.** Every chord is drawn from
//! the major pentatonic (B♭ C D F G): a sixth-ninth on the tonic, a minor
//! seventh on G, a seventh-sus on C and a plain sus on F. No two tones of a
//! chord are a semitone apart, and the motif and resolution keep to the same
//! five notes, so the score's hard rule 7 never moves a note of the tune.

use explainer_score::instr::Instrument;
use explainer_score::score::{BASS_UNDER_PAD_DB, Identity, Palette, Theme};
use explainer_score::theory::{Chord, IONIAN, Key, Motif};

/// B-flat, as a pitch class.
const B_FLAT: i32 = 10;

pub fn identity() -> Identity {
    Identity {
        name: "square_packing".into(),
        key: Key::new(B_FLAT, IONIAN),
        tonic_midi: 58,
        progression: vec![
            // I: B♭ D F G C
            Chord::six_nine(B_FLAT),
            // vi: G B♭ D F
            Chord::new(7, &[0, 3, 7, 10]),
            // ii: C F G B♭, a seventh-sus with no third
            Chord::dom7sus4(0),
            // V: F B♭ C, a plain sus
            Chord::sus4(5),
        ],
        motif: Motif::new(&[3, 2, 1, 2], &[0.5, 0.5, 0.5, 1.5]),
        resolution: Motif::new(&[5, 3, 2, 1], &[0.5, 0.5, 0.5, 2.0]),
        beat_s: 60.0 / 80.0,
        palette: Palette {
            bed: Instrument::Pad { brightness: 0.3 },
            melody: Instrument::Kalimba,
            answer: Instrument::WarmGlass,
            roll: Some(Instrument::Kalimba),
        },
        chord_s: (6.0, 12.0),
        bed_t60: 1.8,
        // the corpus's balance on michael's voice: V − 21 under speech, the
        // gap music near V − 10
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
        theme_chord: None,
        // each think window asks with the motif left open (01_4, 02_5)
        themes: vec![("01_4", Theme::Open), ("02_5", Theme::Open)],
    }
}

#[cfg(test)]
mod tests {
    use explainer_score::score::avoid_minor_ninths;

    use super::*;

    fn pitches(id: &Identity, motif: &Motif) -> Vec<i32> {
        motif.degrees.iter().map(|&d| id.key.degree(d, id.tonic_midi)).collect()
    }

    #[test]
    fn the_identity_is_one_the_engine_can_play() {
        let id = identity();
        id.validate().unwrap();
        assert_eq!(id.progression[0].root, id.key.tonic);
        assert_eq!(id.progression[0].bass, id.key.tonic);
    }

    /// The motif is a turn about 2̂ (D C B♭ C) and the resolution ends on the
    /// tonic; both sound as written over every chord, and no chord has a
    /// semitone inside it.
    #[test]
    fn the_tune_sounds_as_written_over_every_chord() {
        let id = identity();
        assert_eq!(pitches(&id, &id.motif), vec![62, 60, 58, 60], "D C B♭ C");
        assert_eq!(pitches(&id, &id.resolution), vec![65, 62, 60, 58], "F D C B♭");
        for chord in &id.progression {
            let pcs = chord.pitch_classes();
            assert!(pcs.len() >= 3, "{chord:?}");
            for (i, &a) in pcs.iter().enumerate() {
                for &b in &pcs[i + 1..] {
                    assert!(
                        ![1, 11].contains(&(a - b).rem_euclid(12)),
                        "{chord:?} holds {a} and {b}"
                    );
                }
            }
            for &p in pitches(&id, &id.motif).iter().chain(&pitches(&id, &id.resolution)) {
                assert_eq!(
                    avoid_minor_ninths(p, chord, &id.key, 48),
                    p,
                    "{p} would be moved over {chord:?}"
                );
            }
        }
    }

    /// Each think window is the motif left open, and they are the
    /// storyboard's only listen windows.
    #[test]
    fn the_think_windows_ask_with_the_motif() {
        let id = identity();
        assert_eq!(id.theme_for("01_the_turned_tile:01_4"), Some(Theme::Open));
        assert_eq!(id.theme_for("02_four_pins:02_5"), Some(Theme::Open));
        assert_ne!(id.motif.degrees.last(), Some(&1));
        assert_eq!(id.resolution.degrees.last(), Some(&1));
        let board = explainer_kit::storyboard::parse(crate::STORYBOARD).unwrap();
        let windows: Vec<&str> = board
            .iter()
            .flat_map(|c| &c.beats)
            .filter(|b| b.listen_s().is_some())
            .map(|b| b.id.as_str())
            .collect();
        assert_eq!(windows, vec!["01_4", "02_5"]);
    }

    /// Key, tempo and lead differ from every other identity in the corpus as
    /// they stand today (tonic pitch class, beats a minute, melody).
    #[test]
    fn it_is_no_other_video_s() {
        let id = identity();
        let others = [
            (0, 72.0, Instrument::Marimba),
            (2, 76.0, Instrument::Vibraphone),
            (4, 84.0, Instrument::Vibraphone),
            (7, 84.0, Instrument::FeltPiano),
            (5, 88.0, Instrument::FmEp),
            (9, 84.0, Instrument::FmEp),
            (2, 96.0, Instrument::Marimba),
        ];
        let bpm = 60.0 / id.beat_s;
        for (tonic, other_bpm, lead) in others {
            assert_ne!(id.key.tonic, tonic);
            assert!((bpm - other_bpm).abs() > 1.0, "{bpm}");
            assert!(id.palette.melody != lead, "a lead of the corpus");
        }
    }
}
