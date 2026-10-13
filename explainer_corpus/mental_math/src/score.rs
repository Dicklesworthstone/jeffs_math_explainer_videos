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
use explainer_score::score::{BASS_UNDER_PAD_DB, Identity, Palette};
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
        settle_s: 0.8,
        sfx_lu: -16.0,
        listen_lu: -10.0,
        swing: 0.5,
        moods: Vec::new(),
        listen_chord: None,
        themes: Vec::new(),
    }
}
