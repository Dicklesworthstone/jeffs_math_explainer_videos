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
//! motif itself, left open; the closing window, over the receipts (15_5), is
//! the motif and its resolution, the video's one answer.
//!
//! **Nothing in the bed rubs against the tune.** Every chord is drawn from
//! the F major pentatonic alone (F G A C D): a sixth-ninth on the tonic, a
//! minor seventh, and two sus chords. The motif (F A C D) and its resolution
//! (C A F) are at least a whole tone from every tone of every chord, so the
//! score's rule 7 never moves a note of the tune. The first draft's chords
//! (Fmaj9, Dm9, B♭maj9, C7sus4) held an E and a B♭, a semitone from the tune's
//! F and A: the engine moved both F's of the closing phrase up to G, and the
//! video's resolution ended on 2̂ over the tonic.

use explainer_score::instr::Instrument;
use explainer_score::score::{BASS_UNDER_PAD_DB, Identity, Palette, Theme};
use explainer_score::theory::{Chord, IONIAN, Key, Motif};

pub fn identity() -> Identity {
    Identity {
        name: "lie_robotics".into(),
        key: Key::new(5, IONIAN),
        tonic_midi: 65,
        progression: vec![
            // I: the whole picture, F A C D G
            Chord::six_nine(5),
            // vi: D F A C
            Chord::new(2, &[0, 3, 7, 10]),
            // ii, a plain sus: G C D
            Chord::sus4(7),
            // V, a plain sus: C F G
            Chord::sus4(0),
        ],
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
        steady_bed: true,
        settle_s: 0.8,
        sfx_lu: -16.0,
        listen_lu: -10.0,
        swing: 0.5,
        moods: Vec::new(),
        listen_chord: None,
        // 01_3: "listen 3 s (the motif)"; 15_5: "listen 4.7 s (the resolution)"
        themes: vec![("01_3", Theme::Open), ("15_5", Theme::Resolved)],
    }
}

#[cfg(test)]
mod tests {
    use explainer_score::score::{ChapterPlan, Event, Score, Section, Track, clashes, render};
    use explainer_score::timeline::{ChapterTimeline, LinePlacement, Window};
    use explainer_score::{Stereo, wav};

    use super::*;
    use crate::chapters::everywhere::{RESOLVE_LABEL, RESOLVE_S};

    #[test]
    fn the_identity_is_one_the_engine_can_play() {
        let id = identity();
        id.validate().unwrap();
        // the progression's first chord is the tonic the video comes home to
        assert_eq!(id.progression[0].root, id.key.tonic);
        assert_eq!(id.progression[0].bass, id.key.tonic);
    }

    /// The first draft's chords (before the tune was kept clear of them).
    fn first_draft_chords() -> Vec<Chord> {
        vec![Chord::maj9(5), Chord::min9(2), Chord::maj9(10), Chord::dom7sus4(0)]
    }

    /// The tune's pitches (the motif, then the resolution) with each pitch
    /// the engine's rule 7 would move over some chord of `chords`, as
    /// "pitch over chord -> where it goes".
    fn tune_moved_by(id: &Identity, chords: &[Chord]) -> Vec<String> {
        let tune = |m: &explainer_score::theory::Motif| -> Vec<i32> {
            m.degrees.iter().map(|&d| id.key.degree(d, id.tonic_midi)).collect()
        };
        tune(&id.motif)
            .into_iter()
            .chain(tune(&id.resolution))
            .flat_map(|p| {
                chords.iter().filter_map(move |c| {
                    let to = explainer_score::score::avoid_minor_ninths(p, c, &id.key, 48);
                    (to != p).then(|| format!("{p} over {c:?} -> {to}"))
                })
            })
            .collect()
    }

    /// The motif (F A C D) and the resolution (C A F) sound as written over
    /// every chord of the bed, by the engine's own rule, and no chord has a
    /// semitone inside it.
    #[test]
    fn the_tune_sounds_as_written_over_every_chord() {
        let id = identity();
        let pitches = |m: &explainer_score::theory::Motif| -> Vec<i32> {
            m.degrees.iter().map(|&d| id.key.degree(d, id.tonic_midi)).collect()
        };
        assert_eq!(pitches(&id.motif), vec![65, 69, 72, 74], "F A C D");
        assert_eq!(pitches(&id.resolution), vec![72, 69, 65], "C A F");
        assert_eq!(tune_moved_by(&id, &id.progression), Vec::<String>::new());
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
        }
    }

    /// The planted negative of the above: the first draft's chords hold an E
    /// and a B♭, and the engine moved the tonic up to G over the tonic chord
    /// and the A down to G over the dominant's B♭.
    #[test]
    fn the_first_drafts_chords_moved_the_tune() {
        let moved = tune_moved_by(&identity(), &first_draft_chords());
        assert!(
            moved.iter().any(|m| m.starts_with("65 over") && m.ends_with("-> 67")),
            "{moved:?}"
        );
        assert!(moved.len() >= 4, "{moved:?}");
    }

    /// The title card's window plays the motif open and the closing card's
    /// the motif with its resolution; the quiz windows carry their own sound
    /// and name no theme, nor does any other beat.
    #[test]
    fn the_title_cards_window_plays_the_motif_open_and_the_closing_cards_resolves() {
        let id = identity();
        assert_eq!(id.theme_for("01_the_book:01_3"), Some(Theme::Open));
        assert_eq!(id.theme_for(RESOLVE_LABEL), Some(Theme::Resolved));
        assert_eq!(id.theme_for("01_the_book:01_1"), None);
        assert_eq!(id.theme_for("03_three_angles:03_2"), None);
        // the quiz of chapter 15 sits two beats before the resolution and plays no theme
        for q in crate::model::QUIZZES {
            let chapter = crate::chapters::registry()
                .into_iter()
                .find(|c| c.key.starts_with(&q.line[..3]))
                .expect("a chapter of the quiz")
                .key;
            assert_eq!(id.theme_for(&format!("{chapter}:{}", q.line)), None, "{}", q.line);
        }
        assert_eq!(id.themes.len(), 2, "the title's and the ending's");
    }

    const FPS: u32 = 60;
    /// Where the last line of chapter 15 stops in the synthetic chapter below
    /// (s), and when the card has come: its 0.6 s leave and 0.8 s fade-in,
    /// to the frame.
    const LAST_LINE_END: f64 = 31.6;
    const WINDOW_OPENS: f64 = LAST_LINE_END + 1.44;

    /// A voiced line of `seconds`: a buzz of six harmonics whose pitch swings
    /// between 100 and 190 Hz (a narrator's band, which the score measures
    /// before it places its register) under a 4 Hz syllable envelope.
    fn voiced_line(seconds: f64, swing: f64) -> Vec<f64> {
        let mut phase = 0.0_f64;
        (0..(seconds * 48_000.0) as usize)
            .map(|k| {
                let t = k as f64 / 48_000.0;
                let f0 = 145.0 + 45.0 * (std::f64::consts::TAU * swing * t).sin();
                phase += std::f64::consts::TAU * f0 / 48_000.0;
                let buzz: f64 = (1..=6).map(|h| (phase * f64::from(h)).sin() / f64::from(h)).sum();
                0.07 * (0.6 + 0.4 * (std::f64::consts::TAU * 4.0 * t).sin()) * buzz
            })
            .collect()
    }

    /// The end of chapter 15 as the score meets it: four narration lines
    /// (voiced, the last 24.0 to `LAST_LINE_END`),
    /// then the receipts card's window of `seconds`, labelled for a theme or,
    /// without `label`, the 1.5 s unlabelled breath the chapter had before.
    fn closing_score(tag: &str, id: &Identity, resolve: bool, seconds: f64, label: Option<&str>) -> Score {
        let dir = std::env::temp_dir().join(format!("lie_robotics_closing_{}_{tag}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut lines = Vec::new();
        for (i, (a, b)) in [(0.6, 7.0), (8.0, 15.0), (16.0, 23.0), (24.0, LAST_LINE_END)]
            .into_iter()
            .enumerate()
        {
            let samples = voiced_line(b - a, 0.3 + 0.1 * i as f64);
            let path = dir.join(format!("15_{}.wav", i + 1));
            wav::write(&path, &Stereo::from_mono(&samples), wav::Format::F32).unwrap();
            lines.push(LinePlacement {
                id: format!("15_{}", i + 1),
                start_frame: (a * f64::from(FPS)).round() as u64,
                gain_db: 0.0,
                wav: path,
            });
        }
        let window = Window {
            frame: (WINDOW_OPENS * f64::from(FPS)).round() as u64,
            seconds,
            label: label.unwrap_or_default().to_owned(),
            sound: None,
        };
        let (breaths, listens) = if label.is_some() {
            (vec![], vec![window])
        } else {
            (vec![window], vec![])
        };
        let timeline = ChapterTimeline {
            chapter: "15_everywhere".into(),
            fps: FPS,
            frames: ((WINDOW_OPENS + seconds) * f64::from(FPS)).round() as u64,
            lines,
            marks: vec![],
            breaths,
            listens,
        };
        let plan = ChapterPlan {
            intensity: 0.65,
            resolve,
        };
        render(id, &[Section { timeline, plan }], None).unwrap()
    }

    /// The melody notes the score plays after the last line.
    fn closing_phrase(s: &Score) -> Vec<&Event> {
        s.events
            .iter()
            .filter(|e| e.track == Track::Melody && e.start > LAST_LINE_END)
            .collect()
    }

    /// The melodic notes (the melody, its answer and the cues: not the bed,
    /// bass or roll, which are the chord's own voicing) that sound a
    /// semitone from a tone of a chord they sound over: the engine's rule 7.
    fn minor_ninths(s: &Score) -> Vec<String> {
        s.events
            .iter()
            .filter(|e| matches!(e.track, Track::Melody | Track::Answer | Track::Cue))
            .flat_map(|e| {
                s.chords
                    .iter()
                    .filter(move |c| {
                        c.start < e.start + e.hold && c.end > e.start && clashes(e.pitch.round() as i32, &c.chord)
                    })
                    .map(move |c| format!("{:?} {} at {:.3} over {:?}", e.track, e.pitch, e.start, c.chord))
            })
            .collect()
    }

    /// The theme's notes as pitch classes of the key: the motif, then the resolution.
    fn theme_pitch_classes(id: &Identity) -> Vec<i32> {
        let scale = id.key.pitch_classes();
        id.motif
            .degrees
            .iter()
            .chain(&id.resolution.degrees)
            .map(|&d| scale[(d - 1) as usize % 7])
            .collect()
    }

    /// The video's end, as the chapter leaves it: the receipts card's
    /// window, labelled `15_5`, makes the score write the whole theme (the
    /// motif's four notes and the resolution's three) into the gap after the
    /// last line, none of them a minor ninth over the chord it sounds over.
    /// Before, the chapter ended in a 1.5 s breath, too short for any note:
    /// the video had no resolution.
    #[test]
    fn the_closing_window_plays_the_whole_theme_and_no_minor_ninth() {
        let id = identity();
        // the theme as written clashes with no chord of the progression (the
        // first draft's did: F over an E, A over a B♭)
        let raw = theme_pitch_classes(&id);
        assert_eq!(raw, [5, 9, 0, 2, 0, 9, 5]);
        assert!(!raw.iter().any(|&pc| id.progression.iter().any(|c| clashes(pc, c))));
        assert!(
            raw.iter()
                .any(|&pc| first_draft_chords().iter().any(|c| clashes(pc, c)))
        );
        let s = closing_score("window", &id, true, RESOLVE_S, Some(RESOLVE_LABEL));
        let phrase = closing_phrase(&s);
        assert_eq!(
            phrase.len(),
            7,
            "the whole theme: {:?}",
            phrase.iter().map(|e| (e.start, e.pitch)).collect::<Vec<_>>()
        );
        assert!(phrase.windows(2).all(|w| w[0].start < w[1].start));
        assert!(
            phrase[0].start >= LAST_LINE_END,
            "the theme starts after the voice has cleared"
        );
        let total = WINDOW_OPENS + RESOLVE_S;
        assert!(
            phrase[6].start <= total - 2.5 + 1e-9,
            "the last onset {} leaves the duck its time before {total}",
            phrase[6].start
        );
        // the notes sound as written (the engine moved none), the last one home on F
        // and no melodic note of the video is a minor ninth over the chord it sounds over
        let played: Vec<i32> = phrase.iter().map(|e| e.pitch.round() as i32 % 12).collect();
        assert_eq!(
            played,
            raw,
            "the closing phrase: {:?}",
            phrase.iter().map(|e| e.pitch).collect::<Vec<_>>()
        );
        assert_eq!(played[6], id.key.tonic, "the resolution ends on 1̂");
        assert_eq!(minor_ninths(&s), Vec::<String>::new());
    }

    /// The planted negatives of the above: the old tail (a 1.5 s breath) has
    /// no theme at all, and without the identity's `Resolved` theme a window
    /// that is not the video's last gap plays a four-note variant of the motif
    /// and not the answer.
    #[test]
    fn without_the_window_or_the_theme_the_video_does_not_resolve() {
        let id = identity();
        let breath = closing_score("breath", &id, true, 1.5, None);
        assert_eq!(
            closing_phrase(&breath).len(),
            0,
            "a 1.5 s breath has no room for any note"
        );
        // the video's last gap resolves on its own when the chapter asks (`resolve`); a window that
        // the video has not asked to resolve needs the theme to
        let mut unthemed = identity();
        unthemed.themes.retain(|t| t.1 != Theme::Resolved);
        let without = closing_score("unthemed", &unthemed, false, RESOLVE_S, Some(RESOLVE_LABEL));
        assert_ne!(closing_phrase(&without).len(), 7, "no theme, no answer");
        let with = closing_score("themed", &id, false, RESOLVE_S, Some(RESOLVE_LABEL));
        assert_eq!(
            closing_phrase(&with).len(),
            7,
            "the theme alone makes the window the answer"
        );
    }
}
