//! Video I's musical identity (plan §7.5): written for voice `michael`, to
//! be auditioned and tuned by the score bead once the narration exists (the
//! drafts of P5 use the scratch voice and no music).
//!
//! The idea in sound: pieces, then the whole. The motif is three notes that
//! arrive one at a time, 1̂ 3̂ 5̂, and are never held together until the video
//! comes home to them (15_6); the resolution steps down onto 1̂ when a
//! picture closes. C major, the plainest key, unhurried at 72 beats a
//! minute, a pad bed with a marimba melody (struck wooden bars, the pieces
//! of a picture landing) and a glassy answer.
//!
//! **Distinct from Videos G and K** (R1 C10: they share F major, 88 beats
//! and four chords, which this video had drafted too): another key, a slower
//! beat, other chords and another lead (G's melody is the electric piano,
//! K's the vibraphone).
//!
//! **Nothing in the bed rubs against the melody.** Every chord is drawn from
//! the major pentatonic alone (C D E G A): a sixth-ninth on the tonic, a
//! minor seventh, a seventh-sus and a plain sus. No two tones of any chord
//! are a semitone apart, and the motif (C E G) and its resolution (E D C)
//! are at least a whole tone from every tone of every chord, so the score's
//! hard rule 7 never moves a note of the tune, and the pitches of the
//! countdowns and the landings (`chapters::sound`) are consonant over
//! whichever chord the bed is on.
//!
//! Each countdown is a listen window: one mallet note a second, stepping
//! down, over a chord of its own (`chapters::sound`). The title leaves the
//! motif open (01_4); the video's one resolution is at 15_6.

use explainer_score::instr::Instrument;
use explainer_score::score::{BASS_UNDER_PAD_DB, Identity, Palette, Theme};
use explainer_score::theory::{Chord, IONIAN, Key, Motif};

pub fn identity() -> Identity {
    Identity {
        name: "visual_proofs".into(),
        key: Key::new(0, IONIAN),
        tonic_midi: 60,
        progression: vec![
            // I: the whole picture, C E G A D
            Chord::six_nine(0),
            // vi: A C E G
            Chord::new(9, &[0, 3, 7, 10]),
            // ii: D G A C, a seventh-sus with no third
            Chord::dom7sus4(2),
            // V: G C D, a plain sus
            Chord::sus4(7),
        ],
        motif: Motif::new(&[1, 3, 5], &[0.5, 0.5, 2.0]),
        resolution: Motif::new(&[3, 2, 1], &[0.5, 0.5, 2.0]),
        beat_s: 60.0 / 72.0,
        palette: Palette {
            bed: Instrument::Pad { brightness: 0.3 },
            melody: Instrument::Marimba,
            answer: Instrument::WarmGlass,
            roll: Some(Instrument::Marimba),
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
        theme_chord: None,
        // the title leaves the motif open (01_4: three notes arriving one at
        // a time); the video's one resolution is the last beat's (15_6)
        themes: vec![("01_4", Theme::Open), ("15_6", Theme::Resolved)],
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
        assert_eq!(id.name, "visual_proofs");
        // the progression's first chord is the tonic the video comes home to
        assert_eq!(id.progression[0].root, id.key.tonic);
        assert_eq!(id.progression[0].bass, id.key.tonic);
    }

    /// The storyboard (Sound plan, "Identity") asks for a key and a lead
    /// distinct from Videos G and K, which share F major, 88 beats a minute
    /// and four chords (the electric piano leads G, the vibraphone K; K is E
    /// major at 84). The key, tempo and leads are theirs as they stand today.
    /// The chords checked are G's FIRST draft (F, Dm, Bb, C7sus4 as maj9,
    /// min9, maj9, dom7sus4): G has since moved to F6/9, Dm7, Gsus4 and Csus4
    /// so its tune is not displaced (score.rs there). Those are not the list
    /// here on purpose: this video's V chord is also a G sus4, which in C is
    /// the dominant and in F the ii, so a chord-for-chord match with G's
    /// present chords would fail a pair of progressions that sound different.
    #[test]
    fn it_is_not_g_s_or_k_s() {
        let id = identity();
        let (g_tonic, g_bpm, g_lead) = (5, 88.0, Instrument::FmEp);
        let (k_tonic, k_bpm, k_lead) = (4, 84.0, Instrument::Vibraphone);
        assert!(id.key.tonic != g_tonic && id.key.tonic != k_tonic);
        let bpm = 60.0 / id.beat_s;
        assert!((bpm - g_bpm).abs() > 1.0 && (bpm - k_bpm).abs() > 1.0, "{bpm}");
        assert!(id.palette.melody != g_lead && id.palette.melody != k_lead);
        let g_first_draft = [Chord::maj9(5), Chord::min9(2), Chord::maj9(10), Chord::dom7sus4(0)];
        for c in &id.progression {
            assert!(!g_first_draft.contains(c), "{c:?} is one of G's first-draft chords");
        }
    }

    /// The motif (C E G) and the resolution (E D C) sound as written over
    /// every chord of the bed, by the engine's own rule and by hand, and no
    /// chord has a semitone inside it.
    #[test]
    fn the_tune_sounds_as_written_over_every_chord() {
        let id = identity();
        assert_eq!(pitches(&id, &id.motif), vec![60, 64, 67], "C E G");
        assert_eq!(pitches(&id, &id.resolution), vec![64, 62, 60], "E D C");
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

    /// The title is the motif left open and the last beat the resolution:
    /// both are the labels the chapters give their windows.
    #[test]
    fn the_title_is_left_open_and_the_last_beat_resolves() {
        let id = identity();
        assert_eq!(id.theme_for("01_cold_open:01_4"), Some(Theme::Open));
        assert_eq!(id.theme_for("15_what_makes_a_proof:15_6"), Some(Theme::Resolved));
        // no countdown names a theme
        for q in crate::model::QUIZZES {
            let label = format!("00_x:{}", q.line);
            assert_eq!(id.theme_for(&label), None, "{label}");
        }
        // the motif's last note is not the tonic: left open it asks;
        // the resolution's last degree is 1̂
        assert_ne!(id.motif.degrees.last(), Some(&1));
        assert_eq!(id.resolution.degrees.last(), Some(&1));
        // and both windows are beats of the storyboard
        let board = explainer_kit::storyboard::parse(crate::STORYBOARD).unwrap();
        let beats: Vec<&str> = board.iter().flat_map(|c| &c.beats).map(|b| b.id.as_str()).collect();
        assert!(beats.contains(&"01_4") && beats.contains(&"15_6"));
    }
}
