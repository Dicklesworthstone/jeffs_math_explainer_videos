//! Video E's musical identity (plan §7.5): written for voice `michael`, to
//! be auditioned and tuned by the score bead once the narration exists (the
//! drafts of P5 use the scratch voice and no music).
//!
//! The idea in sound: two notes a step apart that swap places. The motif
//! rises 5̂ to 6̂ and at once answers 6̂ to 5̂, the same two notes in the other
//! order, as heads-tails answers tails-heads; the video resolves when the
//! pair settles on 1̂. G major, unhurried, a pad bed with a felt-piano
//! melody and a kalimba answer (a small struck thing, like a coin on a
//! table).

use explainer_score::instr::Instrument;
use explainer_score::score::{BASS_UNDER_PAD_DB, Identity, Mood, Palette, Theme};
use explainer_score::theory::{Chord, IONIAN, Key, Motif};

pub fn identity() -> Identity {
    Identity {
        name: "fair_coin".into(),
        key: Key::new(7, IONIAN),
        tonic_midi: 67,
        progression: vec![Chord::maj9(7), Chord::min9(4), Chord::maj9(0), Chord::dom7sus4(2)],
        motif: Motif::new(&[5, 6, 6, 5], &[0.5, 1.0, 0.5, 1.5]),
        resolution: Motif::new(&[2, 1], &[1.0, 2.0]),
        beat_s: 60.0 / 84.0,
        palette: Palette {
            bed: Instrument::Pad { brightness: 0.3 },
            melody: Instrument::FeltPiano,
            answer: Instrument::Kalimba,
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
        // one harmonic world per act (storyboard §2), all in G major's own
        // tones so the motif and the sonifications stay consonant: act I (01
        // to 03) plays the identity's progression; the price and the breaks
        // (04, 05) lean minor and end unresolved; what it grows into (06 to
        // 09) is brighter; the coda (10, 11) settles on chords with no ♯4
        // (F♯) against the 1̂ the resolution lands on. A first draft, to be
        // auditioned with the score bead like the rest of the identity.
        moods: vec![
            Mood {
                name: "limits",
                chapters: vec!["04", "05"],
                progression: vec![Chord::min9(4), Chord::min9(9), Chord::maj9(0), Chord::dom7sus4(2)],
                pedal: None,
                swing: None,
            },
            Mood {
                name: "grows",
                chapters: vec!["06", "07", "08", "09"],
                progression: vec![Chord::maj9(0), Chord::maj9(7), Chord::min9(9), Chord::dom7sus4(2)],
                pedal: None,
                swing: None,
            },
            Mood {
                name: "settled",
                chapters: vec!["10", "11"],
                progression: vec![
                    Chord::new(7, &[0, 4, 7, 14]),
                    Chord::maj9(0),
                    Chord::min9(9),
                    Chord::dom7sus4(2),
                ],
                pedal: None,
                swing: None,
            },
        ],
        listen_chord: None,
        // the two soundless listen windows the score fills itself: the title
        // card's (01_2, the motif left open) and the last (11_3, resolved);
        // the storyboard gives the other four (02_8, 03_2, 05_2, 06_6) sounds
        // of their own, so they name no theme
        themes: vec![("01_2", Theme::Open), ("11_3", Theme::Resolved)],
    }
}

#[cfg(test)]
mod tests {
    use explainer_score::score::clashes;

    use super::*;

    #[test]
    fn the_identity_is_one_the_engine_can_play() {
        let id = identity();
        id.validate().unwrap();
        assert_eq!(id.name, "fair_coin");
        assert_eq!(id.progression[0].root, id.key.tonic);
        assert_eq!(id.progression[0].bass, id.key.tonic);
    }

    /// The motif's two pairs of notes swap places (5̂–6̂ then 6̂–5̂), and the
    /// resolution settles on 1̂.
    #[test]
    fn the_motif_swaps_two_notes_and_the_resolution_lands_on_the_tonic() {
        let id = identity();
        assert_eq!(id.motif.degrees, vec![5, 6, 6, 5]);
        assert_eq!(id.resolution.degrees, vec![2, 1]);
    }

    /// The storyboard's listen column says which windows are the motif:
    /// "(the motif: …)" left open, "(the motif, resolved)". The identity's
    /// themes are exactly those windows (the ids are lines of the script and
    /// the labels are the ones the chapters give `kit.listen`), no other.
    #[test]
    fn the_themes_are_the_storyboards_motif_windows_and_the_chapters_labels() {
        let mut want = Vec::new();
        for row in crate::STORYBOARD.lines().filter(|l| l.starts_with("| ")) {
            let cells: Vec<&str> = row.split('|').map(str::trim).collect();
            let (id, listen) = (cells[1], cells[cells.len() - 2]);
            let Some((_, what)) = listen.strip_prefix("listen").and_then(|l| l.split_once('(')) else {
                continue;
            };
            if what.starts_with("the motif:") {
                want.push((id.to_string(), Theme::Open));
            } else if what.starts_with("the motif, resolved") {
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
        // the labels the chapters give `kit.listen` carry those ids
        assert!(include_str!("chapters/call_it.rs").contains("\"01_call_it:01_2\""));
        assert!(include_str!("chapters/the_fair_coin_inside.rs").contains("\"11_the_fair_coin_inside:11_3\""));
        assert_eq!(id.theme_for("01_call_it:01_2"), Some(Theme::Open));
        assert_eq!(id.theme_for("11_the_fair_coin_inside:11_3"), Some(Theme::Resolved));
        // the windows with sounds of their own name none
        for label in [
            "02_toss_twice:02_8",
            "03_every_coin:03_2",
            "05_where_it_breaks:05_2",
            "06_same_counts:06_6",
        ] {
            assert_eq!(id.theme_for(label), None, "{label}");
        }
    }

    /// One mood per act after the first (storyboard §2): the chapters 04 to
    /// 11 each in exactly one, 01 to 03 on the identity's own progression.
    #[test]
    fn the_moods_follow_the_acts_and_cover_the_chapters_after_the_first_act() {
        let id = identity();
        id.validate().unwrap();
        assert!(!id.moods.is_empty());
        let keys: Vec<&str> = crate::chapters::registry().iter().map(|c| c.key).collect();
        let number = |key: &str| key.split('_').next().unwrap().to_string();
        for key in keys.iter().filter(|k| ["01", "02", "03"].contains(&number(k).as_str())) {
            assert_eq!(
                id.progression_for(key),
                id.progression,
                "{key} plays the identity's own"
            );
        }
        for key in keys
            .iter()
            .filter(|k| !["01", "02", "03"].contains(&number(k).as_str()))
        {
            assert_ne!(id.progression_for(key), id.progression, "{key} has a mood of its own");
        }
        // the moods name only real chapters, each once
        let mut named: Vec<String> = id
            .moods
            .iter()
            .flat_map(|m| m.chapters.iter().map(|c| c.to_string()))
            .collect();
        named.sort();
        let mut after_act_one: Vec<String> = keys
            .iter()
            .map(|k| number(k))
            .filter(|n| !["01", "02", "03"].contains(&n.as_str()))
            .collect();
        after_act_one.sort();
        assert_eq!(named, after_act_one);
        // the price and the breaks lean minor, the coda settles
        assert_eq!(id.progression_for("05_where_it_breaks")[0], Chord::min9(4));
        assert_eq!(
            id.progression_for("04_the_price"),
            id.progression_for("05_where_it_breaks")
        );
        assert_eq!(id.progression_for("09_six_orders")[0], Chord::maj9(0));
        assert_eq!(
            id.progression_for("11_the_fair_coin_inside")[0],
            Chord::new(7, &[0, 4, 7, 14])
        );
    }

    /// No note of a theme is a minor ninth (a semitone) from a tone of any
    /// chord of the chapter it plays in: the open motif over chapter 01's
    /// chords, the resolution over chapter 11's. The identity's own Gmaj9
    /// puts its F♯ a semitone under the resolution's 1̂, which is why the
    /// coda has its own chords.
    #[test]
    fn no_theme_note_is_a_minor_ninth_from_a_chord_it_sounds_over() {
        let id = identity();
        let notes =
            |degrees: &[i32]| -> Vec<i32> { degrees.iter().map(|&d| id.key.degree(d, id.tonic_midi)).collect() };
        for pitch in notes(&id.motif.degrees) {
            for chord in id.progression_for("01_call_it") {
                assert!(!clashes(pitch, &chord), "motif note {pitch} against {chord:?}");
            }
        }
        for pitch in notes(&id.resolution.degrees) {
            for chord in id.progression_for("11_the_fair_coin_inside") {
                assert!(!clashes(pitch, &chord), "resolution note {pitch} against {chord:?}");
            }
        }
        // the clash the coda's chords avoid: 1̂ against the F♯ of Gmaj9
        assert!(clashes(id.tonic_midi, &Chord::maj9(7)));
    }
}
