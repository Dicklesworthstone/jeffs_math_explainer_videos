//! Video H's musical identity (plan §7.5): written for voice `michael`, to
//! be auditioned and tuned by the score bead once the narration exists (the
//! drafts of P5 use the scratch voice and no music).
//!
//! The idea in sound: a search that widens. The motif steps out from 1̂ and
//! comes back to it, then reaches one note further (4̂); the resolution
//! settles on 1̂ from above (3̂–2̂–1̂) once, after the silence that follows
//! the dead (14_6), and not before. D Dorian, unhurried and sober, a low pad
//! bed with a vibraphone melody and a plucked answer. Each countdown is a
//! listen window: the music alone, one mallet note a second.
//!
//! **The bed keeps out of the melody's way.** The score's hard rule 7 moves
//! any melody note that is a semitone from a tone of the chord it sounds
//! over. The first draft's chords (Dm9, Fmaj9, Am9, G7sus4) each held E or F
//! or both, and the motif's 2̂ and the resolution's 3̂ and 2̂ are exactly E and
//! F: every one of them was moved, and on the real timelines the title
//! sounded D–D–D–G and the close G–D–D (measured on the narration of
//! 2026-10-13). So no chord of the bed holds an F, the motif sounds as
//! written over every chord, and the tonic, the chord the close sits on, holds
//! neither E nor F, so that the resolution does too. The other chords are
//! sixth-ninths and a seventh-sus with no third: fourths and fifths, so the
//! mode is carried by the melody and no pair of tones in the bed is a semitone
//! (`the_motif_and_its_resolution_sound_as_written_where_they_are_played`).

use explainer_score::instr::Instrument;
use explainer_score::score::{BASS_UNDER_PAD_DB, Identity, Palette, Theme};
use explainer_score::theory::{Chord, DORIAN, Key, Motif};

/// The chord the bed holds under every countdown: G, A and D, so that each of
/// the countdown's seven notes (D Dorian from E5 up, `chapters::sound`) is at
/// least a whole tone from every tone of it. It is no chord of the
/// progression: a listen chord equal to the chord before or after its window
/// merges with it (the engine holds a repeated chord), and 03_5's countdown
/// then sat on one chord for 22.8 s against QA's limit of 16.
pub fn listen_chord() -> Chord {
    Chord::new(7, &[0, 2, 7])
}

pub fn identity() -> Identity {
    Identity {
        name: "bayesian_search".into(),
        key: Key::new(2, DORIAN),
        tonic_midi: 62,
        progression: vec![
            // i: D G A C, the tonic the video comes home to (no E and no F:
            // the resolution's 3̂–2̂–1̂ is F–E–D over it)
            Chord::new(2, &[0, 5, 7, 10]),
            // IV, Dorian's major fourth, as a sixth-ninth: G B D E A
            Chord::six_nine(7),
            // bVII: C E G A D
            Chord::six_nine(0),
            // v: A D E G
            Chord::dom7sus4(9),
        ],
        motif: Motif::new(&[1, 2, 1, 4], &[0.5, 0.5, 0.5, 1.5]),
        resolution: Motif::new(&[3, 2, 1], &[0.5, 0.5, 2.0]),
        beat_s: 60.0 / 76.0,
        palette: Palette {
            bed: Instrument::Pad { brightness: 0.25 },
            melody: Instrument::Vibraphone,
            answer: Instrument::Pluck,
            roll: Some(Instrument::Vibraphone),
        },
        // 5 to 10 s, not the corpus's 6 to 12: with these chords the chord
        // that spans the authored silence of 02 (54.2 to 56.7 s) sat on one
        // chord until the bed came back, the music's fade-in was then the
        // only slow change in a steady stretch and QA's exposed beating
        // measured 5.6 dB against 6 (2.3 dB before the retune); at 5 to 10 the
        // worst chapter is 2.3 dB and the longest chord 11.2 s (limit 16)
        chord_s: (5.0, 10.0),
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
        // a countdown sits on one chord for its whole window
        listen_chord: Some(listen_chord()),
        // the title leaves the motif open (01_3); the receipts close on it
        // resolved (14_7), and that is the only resolution there is
        themes: vec![("01_3", Theme::Open), ("14_7", Theme::Resolved)],
    }
}

#[cfg(test)]
mod tests {
    use explainer_score::score::{Theme, avoid_minor_ninths};

    use super::*;
    use crate::chapters::sound::countdown_pitches;

    /// Every pitch class the motif and the resolution use, in the key.
    fn melody_pitches(id: &Identity) -> Vec<i32> {
        id.motif
            .degrees
            .iter()
            .chain(&id.resolution.degrees)
            .map(|&d| id.key.degree(d, id.tonic_midi))
            .collect()
    }

    /// The chords the bed plays: the progression, and the listen chord.
    fn chords(id: &Identity) -> Vec<Chord> {
        let mut all = id.progression.clone();
        all.extend(id.listen_chord.clone());
        all
    }

    #[test]
    fn the_identity_is_one_the_engine_can_play() {
        let id = identity();
        id.validate().unwrap();
        assert_eq!(id.name, "bayesian_search");
        assert_eq!(id.beat_s, 60.0 / 76.0);
        // the progression's first chord is the tonic the close comes home to
        assert_eq!(id.progression[0].root, id.key.tonic);
        assert_eq!(id.progression[0].bass, id.key.tonic);
        // the listen chord is no chord of the progression: it would merge
        // with a neighbour and make one chord of the countdown and its gap
        assert!(!id.progression.contains(&listen_chord()));
    }

    /// The first draft's progression moved every E and F of the melody
    /// (rule 7). The motif (D E D G) must sound as written over every chord
    /// of the bed, and the resolution (F E D) over the chord the close sits
    /// on (the tonic, the progression's first) and over the listen chord.
    /// Checked with the engine's own rule, and by hand.
    #[test]
    fn the_motif_and_its_resolution_sound_as_written_where_they_are_played() {
        let id = identity();
        assert_eq!(
            melody_pitches(&id),
            vec![62, 64, 62, 67, 65, 64, 62],
            "D E D G, then F E D"
        );
        let motif: Vec<i32> = id
            .motif
            .degrees
            .iter()
            .map(|&d| id.key.degree(d, id.tonic_midi))
            .collect();
        let resolution: Vec<i32> = id
            .resolution
            .degrees
            .iter()
            .map(|&d| id.key.degree(d, id.tonic_midi))
            .collect();
        let sounds_as_written = |pitches: &[i32], chord: &Chord| {
            for &p in pitches {
                assert_eq!(
                    avoid_minor_ninths(p, chord, &id.key, 48),
                    p,
                    "{p} would be moved over {chord:?}"
                );
                for t in chord.pitch_classes() {
                    let d = (p - t).rem_euclid(12);
                    assert!(d != 1 && d != 11, "{p} is a semitone from {t} in {chord:?}");
                }
            }
        };
        for chord in chords(&id) {
            sounds_as_written(&motif, &chord);
        }
        sounds_as_written(&resolution, &id.progression[0]);
        sounds_as_written(&resolution, &listen_chord());
        // and no chord holds an F, the only pitch of the melody that the
        // resolution's other notes and the motif's variants would clash with
        // an E over
        for chord in chords(&id) {
            assert!(!chord.pitch_classes().contains(&5), "{chord:?} holds an F");
        }
    }

    /// No chord of the bed has a semitone inside it, which would put a
    /// rough pair under the melody (and the countdown's notes over it).
    #[test]
    fn no_chord_has_a_semitone_inside_it() {
        for chord in chords(&identity()) {
            let pcs = chord.pitch_classes();
            for (i, &a) in pcs.iter().enumerate() {
                for &b in &pcs[i + 1..] {
                    let d = (a - b).rem_euclid(12);
                    assert!(d != 1 && d != 11, "{chord:?} holds {a} and {b}");
                }
            }
            assert!(pcs.len() >= 3, "{chord:?}");
        }
    }

    /// The countdown's notes (`chapters::sound`) are all at least a whole
    /// tone from every tone of the chord that is held under them, so they
    /// are consonant over it: the engine's own listen scale holds each.
    #[test]
    fn every_countdown_note_is_consonant_over_the_listen_chord() {
        let id = identity();
        let scale = id.listen_scale();
        let chord = id.listen_chord.clone().unwrap();
        assert_eq!(chord, listen_chord());
        for seconds in 4..=6u32 {
            for p in countdown_pitches(seconds) {
                let pc = (p as i32).rem_euclid(12);
                assert!(scale.contains(&pc), "{p} ({pc}) is not in the listen scale {scale:?}");
                for t in chord.pitch_classes() {
                    let d = (pc - t).rem_euclid(12);
                    assert!(d != 1 && d != 11, "{p} is a semitone from {t}");
                }
            }
        }
    }

    /// The title's window is the motif left open; the receipts' window is
    /// the video's one resolution: both are the labels the chapters give.
    #[test]
    fn the_title_is_left_open_and_the_receipts_resolve() {
        let id = identity();
        assert_eq!(id.theme_for("01_the_question:01_3"), Some(Theme::Open));
        assert_eq!(id.theme_for("14_four_maps:14_7"), Some(Theme::Resolved));
        // a countdown's window and the other listen windows play no theme
        for q in crate::model::QUIZZES {
            let label = format!("03_x:{}", q.line);
            assert_eq!(id.theme_for(&label), None, "{label}");
        }
        assert_eq!(id.theme_for("14_four_maps:14_70"), None);
        // the motif's last note is not the tonic: left open it asks;
        // the resolution's last degree is 1̂
        assert_ne!(id.motif.degrees.last(), Some(&1));
        assert_eq!(id.resolution.degrees.last(), Some(&1));
        // and both windows exist in the storyboard, with their seconds
        let board = explainer_kit::storyboard::parse(crate::STORYBOARD).unwrap();
        let listen = |id: &str| {
            board
                .iter()
                .flat_map(|c| &c.beats)
                .find(|b| b.id == id)
                .and_then(|b| b.listen_s())
        };
        assert_eq!(listen("01_3"), Some(3.0));
        assert_eq!(listen("14_7"), Some(4.0));
    }
}
