//! The arrangement engine (plan §7.4–7.5): a musical identity and the
//! narration timelines in, one ducked, mixed music stem out.
//!
//! In order:
//! 1. The whole video's voice bus is built from the chapter timelines, so the
//!    music is planned across chapter cuts: tails, phrases and duck releases
//!    that cross a cut stay continuous, and each chapter's stem is a
//!    frame-exact slice of one render (D-22, two-pass scoring).
//! 2. Speech is found on that bus: coarse regions (gaps under 1.2 s merged)
//!    drive the duck; fine regions (gaps under 0.2 s merged) give the pauses
//!    where a chord may change.
//! 3. Harmony: each chord lasts a seeded `chord_s` length; it changes only in
//!    a pause of ≥ 0.2 s or a gap, never within 150 ms after a speech onset,
//!    preferring chapter starts; voices are led by the solver inside the
//!    identity's [`Register`] (a bass octave under the narrator's measured
//!    pitch band, upper voices over it).
//! 4. Notes: the bed holds each chord (Juno chorus, FDN), the bass roots it,
//!    and the leitmotif plays only in gaps of ≥ 1.6 s (no melody under
//!    speech, hard rule 8), its notes moved off any minor ninth against the
//!    chord (rule 7), answered by a soft dyad when there is room.
//! 5. Authored marks get a soft reveal dyad, on the mark or where its
//!    sentence ends.
//! 6. Levels come from the voice's integrated loudness V against fixed
//!    reference renders: a tonic bed at intensity 0.5 sits at V + `bed_lu`,
//!    the motif at V + `melody_lu`; the sum goes through tape, then the duck
//!    (`duck_db`) with the speech pocket; a fixed gain caps the stem's true
//!    peak at −12 dBTP (never a limiter).

use std::io;
use std::path::Path;

use crate::duck::{self, DuckShape};
use crate::filter::{Biquad, Kind};
use crate::fx::{Chorus, Fdn, Plate, Tape, Warmth, from_mid_side, mid_side, rcos};
use crate::instr::{self, Instrument, Note};
use crate::meter;
use crate::rng::{Rng, seed_for};
use crate::theory::{Chord, Key, Motif, Register, Voicing, voice_lead};
use crate::timeline::ChapterTimeline;
use crate::units::{db_to_gain, frame_to_sample, midi_to_hz};
use crate::{FS, Stereo};

/// Which instrument plays which role.
#[derive(Clone, Debug)]
pub struct Palette {
    /// Sustained chords, under speech too.
    pub bed: Instrument,
    /// The leitmotif, in gaps only.
    pub melody: Instrument,
    /// A soft dyad answering a phrase.
    pub answer: Instrument,
    /// Spread two-hand voicings of the chord, rolled slowly, in the longer
    /// gaps (breaths, titles, chapter ends), the pedal lifted before the
    /// voice returns. None: no roll.
    pub roll: Option<Instrument>,
}

/// A chapter's harmonic world (plan §7.5): the chords it cycles through,
/// each held a seeded `chord_s` length, and an optional pedal the bass holds
/// under every one of them (dense derivations sit on a pedal).
#[derive(Clone, Debug, PartialEq)]
pub struct Mood {
    pub name: &'static str,
    /// The chapters that play it, by the number their key starts with ("07").
    pub chapters: Vec<&'static str>,
    pub progression: Vec<Chord>,
    /// A pitch class the bass holds under every chord.
    pub pedal: Option<i32>,
}

/// What the score plays in a listen window it fills itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    /// The leitmotif as it is, left open (a title card).
    Open,
    /// The leitmotif and its resolution: the video's question answered.
    Resolved,
}

/// The narrators' voiced f0, 5th and 95th percentiles (Hz), measured with
/// `docs/research/sound_design/scripts/yin2.py` (YIN on 8 kHz audio) over
/// every take of each video's assembled narration (2026-10-11): `matt` (A)
/// 89 lines, 18,284 voiced frames, median 101 Hz; `leo` (B) 104 lines,
/// 36,442 frames, median 140 Hz; `michael` (C) 84 lines, 12,483 frames,
/// median 107 Hz. (Twelve probe sentences each had read 81–146, 107–190
/// and 83–178 Hz: the real takes reach lower for matt and michael and
/// higher for leo.)
pub const MATT_F0: (f64, f64) = (76.9, 150.9);
pub const LEO_F0: (f64, f64) = (106.7, 205.1);
pub const MICHAEL_F0: (f64, f64) = (76.2, 163.3);

/// The new identities' bass against their pad (dB): about 4 LU under it.
pub const BASS_UNDER_PAD_DB: f64 = -9.0;

/// A video's fixed musical identity; variation lives inside it (plan §7.5).
#[derive(Clone, Debug)]
pub struct Identity {
    /// Seeds every random choice (with chapter, track and event index).
    pub name: String,
    pub key: Key,
    /// Where the music may sit around this video's narrator, from the
    /// narrator's measured pitch (plan §7.3 hard rule 1).
    pub register: Register,
    /// MIDI pitch of the motif's tonic (degree 1).
    pub tonic_midi: i32,
    /// Chords in order, cycled.
    pub progression: Vec<Chord>,
    /// The leitmotif (durations in beats).
    pub motif: Motif,
    /// Degrees appended to the motif where the video resolves.
    pub resolution: Motif,
    pub beat_s: f64,
    pub palette: Palette,
    /// Chord length range in seconds (min, max).
    pub chord_s: (f64, f64),
    /// Bed reverb T60 (s).
    pub bed_t60: f64,
    /// Bed level relative to the voice's integrated loudness, before the duck (LU).
    pub bed_lu: f64,
    /// The bass against the pad inside the bed (dB): at 0 the bass measured
    /// 4–6 LU over the pad in every identity, so the bed's level was mostly a
    /// 35–65 Hz hum under the voice (and, on small speakers, which lose it, a
    /// chord quieter than meant); the pad should carry the bed.
    pub bass_db: f64,
    /// Melody level relative to the voice while it plays (LU).
    pub melody_lu: f64,
    /// Duck depth under speech (dB).
    pub duck_db: f64,
    /// How long the music recedes at the end of an authored window before
    /// the voice returns (s; 0: the duck's own attack, as Hoeffding was
    /// scored): the duck's slower return, and a sonification's closing fade.
    pub settle_s: f64,
    /// How far the duck stays down (0..1 of `duck_db`) through a hold the
    /// author did not write for music (a gap shorter than [`HOLD_MERGE`]);
    /// authored breaths and listen windows release it fully.
    pub hold_duck: f64,
    /// Sound-effect level relative to the voice (LU), for a reference whoosh.
    pub sfx_lu: f64,
    /// A listen window's sonification, relative to the voice (LU): V − 10 or
    /// louder (plan §7.2), so a sound that teaches is heard.
    pub listen_lu: f64,
    /// Chapters' harmonic worlds; a chapter no mood names plays `progression`.
    pub moods: Vec<Mood>,
    /// The chord the bed holds under every listen window that carries a
    /// sonification (None: the progression goes on). It has no two tones a
    /// semitone apart, and a sonification whose pitches keep to
    /// [`Identity::listen_scale`] is consonant over it.
    pub listen_chord: Option<Chord>,
    /// Soundless listen windows the score fills itself, by the label the
    /// scene gave `kit.listen` (its storyboard id): the motif, open or resolved.
    pub themes: Vec<(&'static str, Theme)>,
}

impl Identity {
    /// The chords chapter `chapter` cycles through: its mood's (over the
    /// mood's pedal), or the identity's own progression.
    pub fn progression_for(&self, chapter: &str) -> Vec<Chord> {
        let number = chapter.split('_').next().unwrap_or(chapter);
        match self.moods.iter().find(|m| m.chapters.contains(&number)) {
            Some(m) => m
                .progression
                .iter()
                .map(|c| match m.pedal {
                    Some(p) => c.clone().on(p),
                    None => c.clone(),
                })
                .collect(),
            None => self.progression.clone(),
        }
    }

    /// The pitch classes a sonification may use: every one at least a whole
    /// tone from each tone of the listen chord (or the key's, without one).
    pub fn listen_scale(&self) -> Vec<i32> {
        let chord = self
            .listen_chord
            .as_ref()
            .map_or_else(|| self.key.pitch_classes(), Chord::pitch_classes);
        (0..12)
            .filter(|&pc| chord.iter().all(|&c| !matches!((pc - c).rem_euclid(12), 1 | 11)))
            .collect()
    }

    /// The theme a soundless listen window labelled `label` asks for: its
    /// storyboard id alone (`13_4`) or with the chapter's key or a tag
    /// around it (`13_proudest:13_4`, as the animatic writes).
    pub fn theme_for(&self, label: &str) -> Option<Theme> {
        self.themes
            .iter()
            .find(|t| label.split(':').any(|part| part == t.0))
            .map(|t| t.1)
    }

    /// Refuse an identity the engine cannot play: an empty progression, a
    /// chord too thin for a four-voice bed (fewer than three pitch classes),
    /// a motif or resolution of fewer than two notes, a non-positive beat, an
    /// empty chord-length range, or a non-finite level.
    pub fn validate(&self) -> io::Result<()> {
        let bad = |m: String| {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("identity {}: {m}", self.name),
            ))
        };
        if self.progression.is_empty() {
            return bad("empty progression".into());
        }
        if let Err(e) = self.register.check() {
            return bad(e);
        }
        let moods = self.moods.iter().flat_map(|m| &m.progression);
        if let Some(c) = self
            .progression
            .iter()
            .chain(moods)
            .find(|c| c.pitch_classes().len() < 3)
        {
            return bad(format!("{c:?} has fewer than three pitch classes"));
        }
        if let Some(m) = self
            .moods
            .iter()
            .find(|m| m.progression.is_empty() || m.chapters.is_empty())
        {
            return bad(format!("mood {} has no chords or no chapters", m.name));
        }
        let mut numbers: Vec<&str> = self.moods.iter().flat_map(|m| m.chapters.iter().copied()).collect();
        numbers.sort_unstable();
        if let Some(w) = numbers.windows(2).find(|w| w[0] == w[1]) {
            return bad(format!("chapter {} is in two moods", w[0]));
        }
        if let Some(c) = &self.listen_chord {
            let scale = self.listen_scale();
            if c.pitch_classes().len() < 3 || !c.pitch_classes().iter().all(|pc| scale.contains(pc)) {
                return bad(format!("listen chord {c:?} is thin or has a semitone inside it"));
            }
        }
        let mut labels: Vec<&str> = self.themes.iter().map(|t| t.0).collect();
        labels.sort_unstable();
        if labels.windows(2).any(|w| w[0] == w[1]) {
            return bad("a listen window with two themes".into());
        }
        if self.motif.degrees.len() < 2 || self.resolution.degrees.len() < 2 {
            return bad("the motif and the resolution need at least two notes each".into());
        }
        if !(self.beat_s > 0.0 && self.chord_s.0 > 0.0 && self.chord_s.0 <= self.chord_s.1) {
            return bad(format!("beat {} s, chord lengths {:?}", self.beat_s, self.chord_s));
        }
        if ![
            self.bed_t60,
            self.bed_lu,
            self.bass_db,
            self.melody_lu,
            self.duck_db,
            self.sfx_lu,
            self.listen_lu,
        ]
        .iter()
        .all(|v| v.is_finite())
        {
            return bad("non-finite level or reverb time".into());
        }
        if !(0.0..=2.0).contains(&self.settle_s) {
            return bad(format!("settle {} s outside 0..2", self.settle_s));
        }
        if !(0.0..=1.0).contains(&self.hold_duck) {
            return bad(format!("hold duck {} outside 0..1", self.hold_duck));
        }
        Ok(())
    }

    /// Hoeffding's D, v6 (plan §7.5): D major, Dmaj9 → Bm9 → Gmaj9(♯11) →
    /// Asus4, felt piano over a Juno pad with a soft vibraphone answer, 72 BPM
    /// and free, the rising question 1̂–5̂–6̂–3̂ resolving 2̂–1̂ at the end.
    /// (Its score was checked as it is: no moods, rolls, listen chord or
    /// themes.)
    pub fn hoeffding() -> Self {
        Self {
            name: "hoeffding_d".into(),
            key: Key::new(2, crate::theory::IONIAN),
            register: Register::ROBERT,
            tonic_midi: 62,
            progression: vec![Chord::maj9(2), Chord::min9(11), Chord::maj9_sharp11(7), Chord::sus4(9)],
            motif: Motif::new(&[1, 5, 6, 3], &[1.0, 1.0, 1.0, 2.0]),
            resolution: Motif::new(&[2, 1], &[1.0, 3.0]),
            beat_s: 60.0 / 72.0,
            palette: Palette {
                bed: Instrument::Pad { brightness: 0.3 },
                melody: Instrument::FeltPiano,
                answer: Instrument::Vibraphone,
                roll: None,
            },
            chord_s: (6.0, 12.0),
            bed_t60: 2.5,
            // V - 12 before the 8 dB duck: V - 20 nominal, which measures as
            // a 17-18.5 LU median gap over speech once duck ramps, reverb and
            // bass are counted (plan 7.2's V - 18; at V - 11 the Hoeffding
            // median sat at 16.7-17.5 LU, and short speech regions fell
            // under the 16 LU gate)
            bed_lu: -12.0,
            bass_db: 0.0,
            melody_lu: -12.0,
            duck_db: 8.0,
            hold_duck: 0.0,
            settle_s: 0.0,
            sfx_lu: -16.0,
            // the 2,000 shuffles as a texture: clearly heard, still under the
            // voice that explains it a moment later
            listen_lu: -8.0,
            moods: Vec::new(),
            listen_chord: None,
            themes: Vec::new(),
        }
    }

    /// Video A, capture–recapture (plan §7.5; R1 X-1: each video its own
    /// harmonic language). 6/8 at dotted quarter = 56 in F, in three moods:
    /// - **the lake** (01 the hook, 08 the table balances, 15 the coda): the
    ///   owner's planing major sevenths, moved from A♭ to F and given their
    ///   ninths: Fmaj9, Gmaj9, Dmaj9/F♯, Fmaj9, Cmaj9/E, E♭maj9, Fmaj9, Em11,
    ///   Gmaj9, F♯m11, home to Fmaj9 (held at the very end). The bass circles
    ///   F by step, F G F♯ F E E♭ F E G F♯ F: water lapping at a shore;
    /// - **home** (the method: 02–07, 09, 10, 12): F Lydian, still and modal,
    ///   Fmaj9 → Gadd9 → Dm9 → Cmaj9 (the raised fourth B♮ in the II and V);
    /// - **doubt** (how sure, every failure, the dark cell: 11, 13, 14): D
    ///   Dorian, Dm9 → G13 → Cmaj9 → Am11.
    ///
    /// A Juno "water" pad under speech; in the gaps, slow felt-piano rolls of
    /// the chord and the felt-piano motif 5̂–1̂–2̂–3̂ (C–F–G–A), open at the
    /// title (01_4) and resolved 2̂–1̂ when the table balances (08_3, plan
    /// §7.5 revision 2); vibraphone answers (the marked fish's instrument).
    /// Sonifications keep to F major pentatonic (F G A C D) over the listen
    /// chord, F6/9.
    pub fn capture_recapture() -> Self {
        Self {
            name: "capture_recapture".into(),
            key: Key::new(5, crate::theory::LYDIAN),
            register: Register::from_f0(MATT_F0.0, MATT_F0.1),
            tonic_midi: 65,
            progression: vec![
                Chord::maj9(5),
                Chord::new(7, &[0, 4, 7, 14]),
                Chord::min9(2),
                Chord::maj9(0),
            ],
            motif: Motif::new(&[-2, 1, 2, 3], &[0.5, 1.0, 0.5, 1.5]),
            resolution: Motif::new(&[2, 1], &[1.0, 2.0]),
            beat_s: 60.0 / 56.0,
            palette: Palette {
                bed: Instrument::Pad { brightness: 0.25 },
                melody: Instrument::FeltPiano,
                answer: Instrument::Vibraphone,
                roll: Some(Instrument::FeltPiano),
            },
            chord_s: (8.0, 14.0),
            bed_t60: 2.8,
            // a dB under Hoeffding's balance (see `hoeffding`): V − 21 under
            // speech, and the gap music (motif and rolls over the released
            // bed) near plan §7.2's V − 10, measured on matt's voice
            bed_lu: -13.0,
            bass_db: BASS_UNDER_PAD_DB,
            melody_lu: -13.5,
            duck_db: 8.0,
            hold_duck: 0.5,
            settle_s: 0.8,
            sfx_lu: -16.0,
            // plan §7.2's floor for a sound that teaches, over a bed thinned
            // 4 dB under it: at V − 8 its window's edges measured under the
            // 12 LU p5 gate on matt's and michael's voices
            listen_lu: -10.0,
            moods: vec![
                Mood {
                    name: "lake",
                    chapters: vec!["01", "08", "15"],
                    progression: vec![
                        Chord::maj9(5),
                        Chord::maj9(7),
                        Chord::maj9(2).over(6),
                        Chord::maj9(5),
                        Chord::maj9(0).over(4),
                        Chord::maj9(3),
                        Chord::maj9(5),
                        Chord::min11(4),
                        Chord::maj9(7),
                        Chord::min11(6),
                    ],
                    pedal: None,
                },
                Mood {
                    name: "doubt",
                    chapters: vec!["11", "13", "14"],
                    progression: vec![Chord::min9(2), Chord::dom13(7), Chord::maj9(0), Chord::min11(9)],
                    pedal: None,
                },
            ],
            listen_chord: Some(Chord::six_nine(5)),
            themes: vec![("01_4", Theme::Open), ("08_3", Theme::Resolved)],
        }
    }

    /// Video B, RaptorQ's byte field (plan §7.5; R1 X-1). E, 90 BPM (R1 B-15:
    /// sixteenths at 96 were 6.4 notes a second, over the 6 a second gaps
    /// allow), one mood per act:
    /// - **home** (packets as equations, 01–06, and the payoff, 17): E Lydian
    ///   with the raised fourth (A♯) on the tonic, Emaj9(♯11) → G♯m7 → C♯m9
    ///   → Amaj9 (I–iii–vi–IV);
    /// - **the field** (building the number system, 07–12): the same colours
    ///   on an E pedal, Emaj9 → F♯add9/E (the Lydian II over I) → C♯m9/E →
    ///   Amaj9(♯11)/E, still enough for the algebra;
    /// - **spending it** (13–16): off the pedal and darker, C♯m9 →
    ///   Amaj9(♯11) → F♯m11 → B6sus4.
    ///
    /// Solina strings under speech; in the gaps, Karplus–Strong rolls of the
    /// chord and the warm-glass FM bell motif E–G♯–B–C♯ (0x1D's set bits
    /// {0, 2, 3, 4} as degrees 1, 3, 5, 6), open at the title (01_3),
    /// resolved at the close (17_7); plucked answers. Sonifications keep to E
    /// major pentatonic (E F♯ G♯ B C♯: a byte's eight bit-notes, plan §7.5)
    /// over the listen chord, E6/9.
    pub fn raptorq_gf256() -> Self {
        Self {
            name: "raptorq_gf256".into(),
            key: Key::new(4, crate::theory::LYDIAN),
            register: Register::from_f0(LEO_F0.0, LEO_F0.1),
            tonic_midi: 64,
            progression: vec![
                Chord::maj9_sharp11(4),
                Chord::new(8, &[0, 3, 7, 10]),
                Chord::min9(1),
                Chord::maj9(9),
            ],
            motif: Motif::new(&[1, 3, 5, 6], &[0.5, 0.5, 0.5, 1.5]),
            resolution: Motif::new(&[5, 8], &[1.0, 2.0]),
            beat_s: 60.0 / 90.0,
            palette: Palette {
                bed: Instrument::Strings { brightness: 0.3 },
                melody: Instrument::WarmGlass,
                answer: Instrument::Pluck,
                roll: Some(Instrument::Pluck),
            },
            chord_s: (6.0, 12.0),
            bed_t60: 2.2,
            bed_lu: -12.0,
            bass_db: BASS_UNDER_PAD_DB,
            melody_lu: -12.0,
            duck_db: 8.0,
            hold_duck: 0.5,
            settle_s: 0.8,
            sfx_lu: -16.0,
            listen_lu: -8.0,
            moods: vec![
                Mood {
                    name: "field",
                    chapters: vec!["07", "08", "09", "10", "11", "12"],
                    progression: vec![
                        Chord::maj9(4),
                        Chord::new(6, &[0, 4, 7, 14]),
                        Chord::min9(1),
                        Chord::maj9_sharp11(9),
                    ],
                    pedal: Some(4),
                },
                Mood {
                    name: "spending",
                    chapters: vec!["13", "14", "15", "16"],
                    progression: vec![
                        Chord::min9(1),
                        Chord::maj9_sharp11(9),
                        Chord::min11(6),
                        Chord::six_sus4(11),
                    ],
                    pedal: None,
                },
            ],
            listen_chord: Some(Chord::six_nine(4)),
            themes: vec![("01_3", Theme::Open), ("17_7", Theme::Resolved)],
        }
    }

    /// Video C, the bakery algorithm (plan §7.5; R1 X-1). B♭ major at 104
    /// BPM in a small room (T60 0.9 s), one mood per act:
    /// - **the bakery** (the problem and the building, 01–03, 06, 07, and
    ///   the close, 13): a jazz turnaround, B♭maj9 → G7♯5 → Cm9 → F13;
    /// - **the race** (same number, the unwritten ticket: 04, 05): augmented
    ///   and whole-tone tension, Gm9 → E♭maj7♯5 → C9♯11 → D7♯5 (plan §7.5);
    /// - **the proof** (why never two, garbage, the old zero: 08–10): a B♭
    ///   pedal under the lemmas, B♭maj9 → E♭maj9/B♭ → Gm9/B♭ → Cm11/B♭;
    /// - **the legacy** (costs and CPUs: 11, 12): Gm9 → E♭maj9 → B♭maj9/D →
    ///   F7sus4.
    ///
    /// A pad under speech; in the gaps, felt-piano rolls of the chord and the
    /// marimba motif "take a number" 1̂–2̂–3̂, open at the title (01_3) and
    /// resolved with "served" 5̂–4̂–2̂–1̂ at the close (13_4); plucked answers.
    /// Sonifications keep to B♭ C D E♭ F G (the motif's own notes and G)
    /// over the listen chord, B♭6sus2 (B♭ C F G), which leaves room for both
    /// the motif's D and its E♭.
    pub fn bakery_algorithm() -> Self {
        Self {
            name: "bakery_algorithm".into(),
            key: Key::new(10, crate::theory::IONIAN),
            register: Register::from_f0(MICHAEL_F0.0, MICHAEL_F0.1),
            tonic_midi: 58,
            progression: vec![
                Chord::maj9(10),
                Chord::dom7_sharp5(7),
                Chord::min9(0),
                Chord::new(5, &[0, 4, 10, 14, 21]),
            ],
            motif: Motif::new(&[1, 2, 3], &[0.5, 0.5, 1.0]),
            resolution: Motif::new(&[5, 4, 2, 1], &[0.5, 0.5, 0.5, 1.5]),
            beat_s: 60.0 / 104.0,
            palette: Palette {
                bed: Instrument::Pad { brightness: 0.35 },
                melody: Instrument::Marimba,
                answer: Instrument::Pluck,
                roll: Some(Instrument::FeltPiano),
            },
            chord_s: (6.0, 10.0),
            bed_t60: 0.9,
            // as A: V − 21 under speech, the gap music near V − 10 (michael)
            bed_lu: -13.0,
            bass_db: BASS_UNDER_PAD_DB,
            melody_lu: -13.5,
            duck_db: 8.0,
            hold_duck: 0.5,
            settle_s: 0.8,
            sfx_lu: -16.0,
            // plan §7.2's floor for a sound that teaches, over a bed thinned
            // 4 dB under it: at V − 8 its window's edges measured under the
            // 12 LU p5 gate on matt's and michael's voices
            listen_lu: -10.0,
            moods: vec![
                Mood {
                    name: "race",
                    chapters: vec!["04", "05"],
                    progression: vec![
                        Chord::min9(7),
                        Chord::maj7_sharp5(3),
                        Chord::dom9_sharp11(0),
                        Chord::dom7_sharp5(2),
                    ],
                    pedal: None,
                },
                Mood {
                    name: "proof",
                    chapters: vec!["08", "09", "10"],
                    progression: vec![Chord::maj9(10), Chord::maj9(3), Chord::min9(7), Chord::min11(0)],
                    pedal: Some(10),
                },
                Mood {
                    name: "legacy",
                    chapters: vec!["11", "12"],
                    progression: vec![
                        Chord::min9(7),
                        Chord::maj9(3),
                        Chord::maj9(10).over(2),
                        Chord::dom7sus4(5),
                    ],
                    pedal: None,
                },
            ],
            listen_chord: Some(Chord::new(10, &[0, 2, 7, 9])),
            themes: vec![("01_3", Theme::Open), ("13_4", Theme::Resolved)],
        }
    }
}

/// What one chapter asks of the score.
#[derive(Clone, Debug)]
pub struct ChapterPlan {
    /// 0..1: bed velocity and brightness.
    pub intensity: f64,
    /// End this chapter (the video) on the tonic with the motif resolved.
    pub resolve: bool,
}

/// One chapter: its timeline and its plan.
#[derive(Clone, Debug)]
pub struct Section {
    pub timeline: ChapterTimeline,
    pub plan: ChapterPlan,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Track {
    Bed,
    Bass,
    Melody,
    Answer,
    /// A reveal answering an authored mark.
    Cue,
    /// The spread voicing rolled in a gap (`Palette::roll`).
    Roll,
}

/// What an authored mark asks for, read from its label: `kind:name`, or a
/// bare name for a reveal (`kit.mark(stage, "swell:tableau")`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CueKind {
    /// A soft dyad of chord tones, in the gap (the default).
    Reveal,
    /// A noise swell rising into the mark (a transition).
    Swell,
    /// The motif's resolution on the melody instrument (an "aha").
    Resolve,
    /// A soft wooden tick on the mark itself, even under speech (a counter).
    Tick,
    /// One plucked chord tone (pizzicato counts).
    Pizz,
    /// A noise whoosh centred on the mark (a scene change).
    Whoosh,
    /// The bed thins 4 dB into the mark and recovers after it, with no cue
    /// of its own (a line that needs space, not an answer).
    Thin,
    /// Silence from the next sentence on (plan §7.2's key line, or a beat
    /// that must sound bare): see [`SilenceSpan`].
    Silence,
    /// The music may return (at the next pause) after a `silence:` mark of
    /// the same name.
    SilenceEnd,
}

impl CueKind {
    pub fn from_label(label: &str) -> Self {
        match label.split_once(':').map(|(k, _)| k) {
            Some("swell") => Self::Swell,
            Some("resolve") => Self::Resolve,
            Some("tick") => Self::Tick,
            Some("pizz") => Self::Pizz,
            Some("whoosh") => Self::Whoosh,
            Some("thin") => Self::Thin,
            Some("silence") => Self::Silence,
            Some("silence_end") => Self::SilenceEnd,
            _ => Self::Reveal,
        }
    }
}

/// An authored silence (`kit.mark(stage, "silence:<name>")`, optionally
/// closed by `"silence_end:<name>"`), in seconds from the video's start. The
/// music fades out in the pause before the next sentence (`fade_start` to
/// `fade_end`: up to [`SILENCE_FADE`] long, ending up to [`KEY_LEAD`] before
/// that sentence, never starting before the previous one has ended), is
/// exactly zero from `fade_end` to `back`, and fades in again over
/// [`RETURN_FADE`] from `back`: the first pause at or after the end mark,
/// or, without one, where the voice next pauses for good (a gap of 1.2 s or
/// more).
#[derive(Clone, Debug, PartialEq)]
pub struct SilenceSpan {
    pub name: String,
    pub fade_start: f64,
    pub fade_end: f64,
    pub back: f64,
    pub back_end: f64,
}

/// The music is gone this long before the sentence an authored silence is
/// for, when the pause before it allows (s; plan §7.2's key line).
pub const KEY_LEAD: f64 = 0.5;
/// The longest fade into an authored silence (s; A's storyboard, 15_1).
pub const SILENCE_FADE: f64 = 1.2;
/// The shortest: a key line between two sentences drops out this fast (s).
const MIN_SILENCE_FADE: f64 = 0.1;
/// The music's return after an authored silence (s).
pub const RETURN_FADE: f64 = 1.5;

/// The authored silences of a video: each `silence:` mark's span (see
/// [`SilenceSpan`]), and a note for each that could not have the fade it
/// asked for (a pause too short for 1.2 s).
fn authored_silences(
    sections: &[Section],
    starts: &[f64],
    fine: &[(f64, f64)],
    coarse: &[(f64, f64)],
    total: f64,
) -> (Vec<SilenceSpan>, Vec<String>) {
    let mut spans = Vec::new();
    let mut notes = Vec::new();
    for (s, &t0) in sections.iter().zip(starts) {
        let fps = f64::from(s.timeline.fps);
        let at = |m: &crate::timeline::Mark| t0 + m.frame as f64 / fps;
        for m in &s.timeline.marks {
            if CueKind::from_label(&m.label) != CueKind::Silence {
                continue;
            }
            let name = m.label.split_once(':').map_or("", |x| x.1).to_string();
            let t = at(m);
            let onset = fine
                .iter()
                .map(|r| r.0)
                .find(|&o| o >= t - ONSET_GUARD)
                .unwrap_or(total);
            let prev_end = fine.iter().map(|r| r.1).filter(|&e| e <= onset).fold(0.0, f64::max);
            let lead = (onset - prev_end - 0.1).clamp(0.05, KEY_LEAD);
            let fade_end = onset - lead;
            let fade = (fade_end - prev_end - 0.05).clamp(MIN_SILENCE_FADE, SILENCE_FADE);
            if fade < SILENCE_FADE {
                notes.push(format!(
                    "silence:{name}: a {:.2} s pause before its sentence, so a {fade:.2} s fade",
                    onset - prev_end
                ));
            }
            // the return: at the end mark, or the end of the sentence it falls
            // in; without one, where the voice next pauses for good
            let end_mark = s
                .timeline
                .marks
                .iter()
                .filter(|e| CueKind::from_label(&e.label) == CueKind::SilenceEnd)
                .filter(|e| e.label.split_once(':').map_or("", |x| x.1) == name)
                .map(at)
                .find(|&e| e > t);
            let back = match end_mark {
                Some(e) => fine.iter().find(|r| e >= r.0 && e < r.1).map_or(e, |r| r.1 + 0.05),
                None => coarse
                    .iter()
                    .find(|r| onset >= r.0 && onset < r.1)
                    .map_or(onset, |r| r.1 + 0.05),
            }
            .min(total);
            spans.push(SilenceSpan {
                name,
                fade_start: fade_end - fade,
                fade_end,
                back: back.max(fade_end),
                back_end: (back.max(fade_end) + RETURN_FADE).min(total),
            });
        }
    }
    (spans, notes)
}

/// The gain the authored silences leave the music (1 outside them, raised
/// cosine fades, exactly 0 inside).
fn silence_gain(spans: &[SilenceSpan], n: usize) -> Vec<f64> {
    let mut g = vec![1.0_f64; n];
    for sp in spans {
        let (a, b) = (
            (sp.fade_start.max(0.0) * FS) as usize,
            ((sp.back_end * FS).ceil() as usize).min(n),
        );
        for (i, v) in g.iter_mut().enumerate().take(b).skip(a) {
            let t = i as f64 / FS;
            let x = if t < sp.fade_end {
                1.0 - rcos((t - sp.fade_start) / (sp.fade_end - sp.fade_start))
            } else if t < sp.back {
                0.0
            } else {
                rcos((t - sp.back) / (sp.back_end - sp.back).max(1e-9))
            };
            *v = v.min(x);
        }
    }
    g
}

/// A sound effect (noise or tick) placed by the score.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SfxKind {
    Swell { seconds: f64 },
    Whoosh { seconds: f64 },
    Tick { hz: f64 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sfx {
    pub start: f64,
    pub kind: SfxKind,
    pub seed: u64,
}

/// One note the score played (times in seconds from the video start).
#[derive(Clone, Debug)]
pub struct Event {
    pub start: f64,
    pub hold: f64,
    pub track: Track,
    pub inst: Instrument,
    pub pitch: f64,
    pub velocity: f64,
    pub seed: u64,
    pub pan: f64,
}

/// A chord as it sounds: from its change time to the next.
#[derive(Clone, Debug)]
pub struct ChordSpan {
    pub start: f64,
    pub end: f64,
    pub chord: Chord,
    pub voicing: Voicing,
}

/// A rendered score with everything QA needs to judge it.
#[derive(Clone, Debug)]
pub struct Score {
    /// The music stem for the whole video, ducked and at its final level.
    pub music: Stereo,
    /// The register it was scored in (QA judges hard rule 1 against its band).
    pub register: Register,
    /// The voice bus it was scored against.
    pub voice: Stereo,
    pub events: Vec<Event>,
    pub chords: Vec<ChordSpan>,
    /// Melodic phrases (start, end of the last onset) in seconds.
    pub phrases: Vec<(f64, f64)>,
    /// Reveal cues placed for authored marks: (seconds, label).
    pub cues: Vec<(f64, String)>,
    /// Marks the score could not answer, as `chapter/label`: no room (deep
    /// inside a long sentence, at the very end), too close to another cue
    /// (rule 5), or no pitch clear of a minor ninth.
    pub skipped_marks: Vec<String>,
    /// Sound effects placed for marks.
    pub sfx: Vec<Sfx>,
    /// Coarse speech regions (seconds) the duck follows.
    pub speech: Vec<(f64, f64)>,
    /// Chapter name and sample span [start, end) within the video.
    pub chapters: Vec<(String, usize, usize)>,
    /// The voice's integrated loudness (LUFS).
    pub voice_lufs: f64,
    /// The music stem's true peak (dBTP) after the cap.
    pub true_peak_db: f64,
    /// Gain the true-peak cap applied (dB; 0 when the cap did not bind).
    pub cap_db: f64,
    /// The authored silences (seconds), where the stem is exactly zero.
    pub silences: Vec<SilenceSpan>,
    /// What the score could not do as authored (a silence's fade shortened).
    pub notes: Vec<String>,
}

impl Score {
    /// Chapter `k`'s stem: exactly its scene's length.
    pub fn chapter_music(&self, k: usize) -> Stereo {
        let (_, a, b) = self.chapters[k];
        Stereo {
            l: self.music.l[a..b].to_vec(),
            r: self.music.r[a..b].to_vec(),
        }
    }

    /// Chapter `k`'s voice bus.
    pub fn chapter_voice(&self, k: usize) -> Stereo {
        let (_, a, b) = self.chapters[k];
        Stereo {
            l: self.voice.l[a..b].to_vec(),
            r: self.voice.r[a..b].to_vec(),
        }
    }
}

// ----------------------------------------------------------------- planning

/// No chord change within this long after a speech onset (plan §7.5).
const ONSET_GUARD: f64 = 0.15;
/// The shortest pause in speech a chord may change in.
const MIN_PAUSE: f64 = 0.2;
/// The shortest gap a melodic phrase may play in.
pub const MIN_PHRASE_GAP: f64 = 1.6;
/// A gap in speech shorter than this, not authored as a breath or a listen
/// window, is a hold: the duck stays partly down through it (s).
pub const HOLD_MERGE: f64 = 4.0;

/// Times where a chord may change: the start of the span and of each pause
/// of ≥ 0.2 s (just after the voice stops), then every 0.5 s through the
/// silence; never inside speech or within 150 ms after a speech onset.
/// `fine` are the speech regions (sorted, merged across gaps under 0.2 s).
pub fn change_candidates(fine: &[(f64, f64)], total: f64) -> Vec<f64> {
    let mut c = Vec::new();
    let silence = |a: f64, b: f64, c: &mut Vec<f64>| {
        let mut t = a;
        while t < b {
            c.push(t);
            t += 0.5;
        }
    };
    let mut prev_end: Option<f64> = None;
    for &(s, e) in fine {
        match prev_end {
            None => silence(0.0, s, &mut c),
            Some(p) if s - p >= MIN_PAUSE => silence(p + 0.05, s, &mut c),
            Some(_) => {}
        }
        prev_end = Some(e);
    }
    silence(prev_end.map_or(0.0, |p| p + 0.05), total, &mut c);
    c.retain(|&t| t < total && !fine.iter().any(|&(s, e)| t >= s && (t < e || t < s + ONSET_GUARD)));
    c.sort_by(f64::total_cmp);
    c.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
    c
}

/// Plan §7.5's limits on any chord (s): an identity's preferred range sits
/// inside them; QA fails a chord longer than the longest.
pub const SHORTEST_CHORD: f64 = 4.0;
pub const LONGEST_CHORD: f64 = 16.0;

/// Choose chord change times (the first is 0) so every chord lasts between
/// `min` and `max` whenever the candidates allow: while more than `max`
/// remains, the next change goes to the candidate nearest a seeded target in
/// [t + min, t + max] that still leaves ≥ `min` for what follows; with no
/// candidate there (speech allowed no pause), to the first later one that
/// does (the chord holds, a pedal, longer than `max`).
pub fn plan_changes(candidates: &[f64], total: f64, (min, max): (f64, f64), rng: &mut Rng) -> Vec<f64> {
    let mut out = vec![0.0];
    let mut t = 0.0;
    while total - t > max {
        let (lo, hi) = (t + min, (t + max).min(total - min));
        let target = (t + rng.range(min, max)).clamp(lo, hi.max(lo));
        let next = candidates
            .iter()
            .copied()
            .filter(|&c| c >= lo && c <= hi)
            .min_by(|a, b| (a - target).abs().total_cmp(&(b - target).abs()))
            .or_else(|| candidates.iter().copied().find(|&c| c > hi && c <= total - min))
            .or_else(|| {
                // the end is near and no pause fits the preferred lengths: the
                // most central pause that keeps both chords at least
                // SHORTEST_CHORD long (plan §7.5's chords of 4–16 s), rather
                // than one chord past 16 s
                let mid = (t + total) / 2.0;
                candidates
                    .iter()
                    .copied()
                    .filter(|&c| c >= t + SHORTEST_CHORD && c <= total - SHORTEST_CHORD)
                    .filter(|_| total - t > LONGEST_CHORD)
                    .min_by(|a, b| (a - mid).abs().total_cmp(&(b - mid).abs()))
            });
        match next {
            Some(c) => {
                out.push(c);
                t = c;
            }
            None => break,
        }
    }
    out
}

/// A listen stretch shorter than this between two listen windows is not
/// given its own chord: the listen chord holds across it (s).
const MIN_FREE_STRETCH: f64 = 2.0;

/// Chord changes for a chapter `total` seconds long whose listen spans (each
/// `[start, until)`, chapter-relative) hold the listen chord: each span is
/// one change flagged `true`, and every stretch between spans is planned by
/// [`plan_changes`] on its own candidates (flagged `false`). A span starts
/// where its window opens (the voice has stopped) and runs to the first
/// change candidate at or after the window's end, so the progression comes
/// back in a pause, never on a speech onset.
pub fn plan_with_listens(
    candidates: &[f64],
    total: f64,
    range: (f64, f64),
    listens: &[(f64, f64)],
    rng: &mut Rng,
) -> Vec<(f64, bool)> {
    let mut spans: Vec<(f64, f64)> = listens
        .iter()
        .filter(|s| s.0 < total)
        .map(|&(s, u)| (s.max(0.0), u.min(total)))
        .collect();
    spans.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut out = Vec::new();
    let free = |from: f64, to: f64, out: &mut Vec<(f64, bool)>, rng: &mut Rng| {
        let local: Vec<f64> = candidates
            .iter()
            .filter(|&&x| x >= from && x < to)
            .map(|x| x - from)
            .collect();
        out.extend(
            plan_changes(&local, to - from, range, rng)
                .into_iter()
                .map(|x| (from + x, false)),
        );
    };
    let mut from = 0.0;
    for (s, u) in spans {
        if s - from >= MIN_FREE_STRETCH || (out.is_empty() && s > 0.0) {
            free(from, s, &mut out, rng);
        }
        if out.last().is_none_or(|l| !l.1) {
            out.push((s, true));
        }
        from = from.max(u);
    }
    if total - from > 1e-9 {
        free(from, total, &mut out, rng);
    }
    out
}

/// The roll's onsets are this far apart (s): five notes in 0.8 s, a slow
/// harp-like unfolding inside the six onsets a second gaps allow (§7.6).
const ROLL_STEP: f64 = 0.2;
/// A gap needs this long for a roll (s).
const ROLL_GAP: f64 = 2.4;
/// The pedal is lifted this long before the voice returns (s): the felt
/// piano's low notes are inside the narrator's band and must have died away.
const PEDAL_CLEAR: f64 = 0.7;

/// A spread two-hand voicing for the roll, low to high: the left hand's bass
/// (the chord's bass note, C2–B2) and the next chord tone a fifth or more
/// above it, then the right hand's three highest bed voices, lifted to E4 or
/// above. In a gap only: the left hand sits in the narrator's band.
pub fn roll_voicing(span: &ChordSpan) -> Vec<i32> {
    let pcs = span.chord.pitch_classes();
    let bass = 36 + (span.chord.bass - 36).rem_euclid(12);
    let second = (bass + 7..bass + 19)
        .find(|&m| pcs.contains(&m.rem_euclid(12)) && m.rem_euclid(12) != span.chord.bass && !minor_ninth(bass, m))
        .unwrap_or(bass + 12);
    // the right hand from E4, over the left: each tone lifted by octaves,
    // then laid out rising, a whole tone or more apart, up to E6. A tone a
    // minor ninth over a lower one (E under F, at any octave: lifting it
    // never helps, the pitch classes are the problem) is left out, so F4
    // under E5, a major seventh, stays and E4 under F5 does not happen
    let floor = 64.max(second + 3);
    let k = span.voicing.upper.len().saturating_sub(3);
    let mut right: Vec<i32> = span.voicing.upper[k..]
        .iter()
        .map(|&m| m + 12 * ((floor - m + 11).max(0) / 12))
        .collect();
    right.sort_unstable();
    right.dedup();
    let mut out = vec![bass, second];
    let place = |out: &[i32], mut p: i32| {
        while p <= out[out.len() - 1] + 1 && p <= ROLL_TOP {
            p += 12;
        }
        (p <= ROLL_TOP && !out.iter().any(|&q| minor_ninth(q, p))).then_some(p)
    };
    for p in right {
        if let Some(p) = place(&out, p) {
            out.push(p);
        }
    }
    // a voice left out is replaced by another chord tone that sits clean
    // over everything below it, the lowest first, while there is room
    let mut spare: Vec<i32> = pcs
        .iter()
        .filter(|&&pc| pc != span.chord.bass && !out[2..].iter().any(|m| m.rem_euclid(12) == pc))
        .map(|&pc| floor + (pc - floor).rem_euclid(12))
        .collect();
    spare.sort_unstable();
    for p in spare {
        if out.len() >= 5 {
            break;
        }
        if let Some(p) = place(&out, p) {
            out.push(p);
        }
    }
    out
}

/// The roll's highest note (E6).
const ROLL_TOP: i32 = 88;
/// A gap's motif waits this long after an authored cue in the same gap (s).
const AFTER_CUE: f64 = 1.2;

/// True when `hi` sits a minor ninth (or a compound of it) above `lo`.
pub fn minor_ninth(lo: i32, hi: i32) -> bool {
    hi - lo > 1 && (hi - lo).rem_euclid(12) == 1
}

/// The motif notes that fit a gap: (onset s, MIDI pitch, duration s). Onsets
/// run from `start`; the longest prefix whose last onset is ≤ `limit` is
/// kept, if it has at least two notes. Notes fold by octaves into `low`..=C6
/// (`low`: the register's lowest upper voice, above the narrator's band).
pub fn fit_phrase(
    motif: &Motif,
    key: &Key,
    tonic_midi: i32,
    beat_s: f64,
    start: f64,
    limit: f64,
    low: i32,
) -> Vec<(f64, i32, f64)> {
    let pitches = motif.pitches(key, tonic_midi);
    let mut out = Vec::new();
    let mut t = start;
    for (p, d) in pitches.into_iter().zip(&motif.durations) {
        if t > limit {
            break;
        }
        // keep the melody in its register, over the narrator's band
        let mut p = p;
        while p < low {
            p += 12;
        }
        while p > 84 {
            p -= 12;
        }
        out.push((t, p, d * beat_s));
        t += d * beat_s;
    }
    if out.len() < 2 { Vec::new() } else { out }
}

/// Elastic phrasing (±8%): the video's closing resolution relaxes into the
/// end (beat × 1.08) when that still fits; a phrase that does not fit whole
/// at tempo may tighten (beat × 0.92) to fit whole; otherwise the longest
/// prefix that fits at tempo.
pub fn elastic_phrase(
    motif: &Motif,
    identity: &Identity,
    start: f64,
    limit: f64,
    closing: bool,
) -> Vec<(f64, i32, f64)> {
    let fit = |beat: f64| {
        fit_phrase(
            motif,
            &identity.key,
            identity.tonic_midi,
            beat,
            start,
            limit,
            identity.register.upper.0,
        )
    };
    let whole = motif.degrees.len();
    if closing {
        let relaxed = fit(identity.beat_s * 1.08);
        if relaxed.len() == whole {
            return relaxed;
        }
    }
    let nominal = fit(identity.beat_s);
    if nominal.len() < whole {
        let tight = fit(identity.beat_s * 0.92);
        if tight.len() == whole {
            return tight;
        }
    }
    nominal
}

/// Move `pitch` off a minor ninth (a semitone from any chord tone, hard rule
/// 7) to the nearest key tone that clashes with nothing and stays in the
/// melody's register (`low` up: never into the narrator's band).
pub fn avoid_minor_ninths(pitch: i32, chord: &Chord, key: &Key, low: i32) -> i32 {
    let tones = chord.pitch_classes();
    let clashes = |p: i32| {
        tones
            .iter()
            .any(|&c| (p - c).rem_euclid(12) == 1 || (c - p).rem_euclid(12) == 1)
    };
    if !clashes(pitch) {
        return pitch;
    }
    // the key's tones, or the chord's own (a mood may leave the key)
    let scale = key.pitch_classes();
    for d in [1, -1, 2, -2, 3, -3, 4, -4] {
        let p = pitch + d;
        let pc = p.rem_euclid(12);
        if (low..=88).contains(&p) && (scale.contains(&pc) || tones.contains(&pc)) && !clashes(p) {
            return p;
        }
    }
    pitch
}

/// The voicing's top two upper voices (fewer if it has fewer).
fn top_two(v: &Voicing) -> Vec<i32> {
    let k = v.upper.len().saturating_sub(2);
    v.upper[k..].to_vec()
}

/// The shortest melody, answer or cue note worth playing once rule 7 has cut
/// it at a clashing change (s).
const MIN_NOTE_S: f64 = 0.12;
/// A note cue (reveal, pizzicato, resolution) needs this much clear time
/// before the next speech onset, or it is skipped: struck in the pause
/// between two sentences it rings into the second (found on Hoeffding's
/// hook; plan §7.2: the music answers in the gaps the author writes).
const CUE_MIN_GAP: f64 = 1.0;
/// A cue note is released this long before the next speech onset (s).
const CUE_CLEAR: f64 = 0.3;
/// A gap's melody and answer notes are released this long before the voice
/// returns (s): the music breathes out, then the narrator speaks.
const GAP_CLEAR: f64 = 0.5;

/// A reveal's dyad: the voicing's top two voices, up into the cue register
/// (folding by octaves above E6), each moved off any minor ninth, since in a
/// ninth chord one chord tone can clash with another (Bm9's D against its
/// C♯); a tone with no safe neighbour is left out, and a repeat is dropped.
fn reveal_dyad(v: &Voicing, chord: &Chord, key: &Key, low: i32) -> Vec<i32> {
    let mut dyad: Vec<i32> = Vec::new();
    for &mp in &top_two(v) {
        let mut p = if mp < 72 { mp + 12 } else { mp };
        while p > 88 {
            p -= 12;
        }
        let p = avoid_minor_ninths(p, chord, key, low);
        if !clashes(p, chord) && !dyad.contains(&p) {
            dyad.push(p);
        }
    }
    dyad
}

/// True when `pitch` is a semitone from any tone of `chord` (a minor ninth).
pub fn clashes(pitch: i32, chord: &Chord) -> bool {
    chord
        .pitch_classes()
        .iter()
        .any(|&c| (pitch - c).rem_euclid(12) == 1 || (c - pitch).rem_euclid(12) == 1)
}

/// The hold of a note at `pitch` from `start`, cut where a later chord it
/// would clash with begins (rule 7 holds for as long as the note sounds).
pub fn hold_until_clash(pitch: i32, start: f64, hold: f64, chords: &[ChordSpan]) -> f64 {
    chords
        .iter()
        .filter(|c| c.start > start && c.start < start + hold && clashes(pitch, &c.chord))
        .map(|c| c.start - start)
        .fold(hold, f64::min)
}

// ----------------------------------------------------------------- rendering

fn render_track(events: &[&Event], n: usize) -> Stereo {
    let threads = std::thread::available_parallelism().map_or(4, |p| p.get()).clamp(1, 8);
    let mut bus = Stereo::silence(n);
    for batch in events.chunks(threads) {
        let rendered: Vec<(usize, Stereo)> = std::thread::scope(|s| {
            let handles: Vec<_> = batch
                .iter()
                .map(|e| {
                    s.spawn(move || {
                        let mut note = Note::new(e.pitch, e.velocity, e.hold, e.seed);
                        note.pan = e.pan;
                        ((e.start * FS).round() as usize, instr::render(e.inst, &note))
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|h| h.join().expect("note render panicked"))
                .collect()
        });
        for (at, s) in rendered {
            bus.mix_in(&s, at, 1.0);
        }
    }
    bus
}

fn add_scaled(acc: &mut Stereo, x: &Stereo, g: f64) {
    for (a, b) in acc.l.iter_mut().zip(&x.l) {
        *a += g * b;
    }
    for (a, b) in acc.r.iter_mut().zip(&x.r) {
        *a += g * b;
    }
}

/// The bed instrument at a chapter's intensity.
fn bed_instrument(identity: &Identity, intensity: f64) -> Instrument {
    match identity.palette.bed {
        Instrument::Pad { .. } => Instrument::Pad {
            brightness: 0.15 + 0.35 * intensity,
        },
        Instrument::Strings { .. } => Instrument::Strings {
            brightness: 0.15 + 0.35 * intensity,
        },
        other => other,
    }
}

/// The bed and bass notes of one chord. Seeds come from the chapter and the
/// chord's index within it, so no other chapter's edit can change them.
fn chord_events(identity: &Identity, span: &ChordSpan, intensity: f64, chapter: &str, k: usize) -> Vec<Event> {
    let intensity = intensity.clamp(0.0, 1.0);
    let mut out = Vec::new();
    // the chord swells in from its change (in a pause), and releases from the
    // next change: the crossfade begins where the rules allow a change, never
    // inside the sentence before it, and never across a chapter's cut
    let note_start = span.start;
    let hold = (span.end - span.start).max(0.5);
    for (j, &m) in span.voicing.upper.iter().enumerate() {
        out.push(Event {
            start: note_start,
            hold,
            track: Track::Bed,
            inst: bed_instrument(identity, intensity),
            pitch: f64::from(m),
            velocity: 0.35 + 0.3 * intensity,
            seed: seed_for(&identity.name, chapter, "bed", (k * 8 + j) as u64),
            pan: [-0.35, -0.12, 0.12, 0.35][j % 4],
        });
    }
    // intensity adds density and spread: from 0.6 up, the top voice is
    // doubled an octave higher, softly
    if intensity >= 0.6
        && let Some(&top) = span.voicing.upper.last()
        && top + 12 <= 88
    {
        out.push(Event {
            start: note_start,
            hold,
            track: Track::Bed,
            inst: bed_instrument(identity, intensity),
            pitch: f64::from(top + 12),
            velocity: 0.6 * (0.35 + 0.3 * intensity),
            seed: seed_for(&identity.name, chapter, "bed", (k * 8 + 4) as u64),
            pan: 0.0,
        });
    }
    out.push(Event {
        start: span.start,
        hold: (span.end - span.start - 0.15).max(0.3),
        track: Track::Bass,
        inst: Instrument::Bass,
        pitch: f64::from(span.voicing.bass),
        velocity: 0.5 + 0.2 * intensity,
        seed: seed_for(&identity.name, chapter, "bass", k as u64),
        pan: 0.0,
    });
    out
}

/// The bed bus: pads (Juno chorus on the mid, the voices' spread kept on the
/// side) and the bass (through gentle warmth, at the identity's `bass_db`),
/// both scaled by `gain` (per sample: the thin-outs before reveals and under
/// sonifications), with an FDN room fed from the pads at `send` (per sample:
/// more room at higher intensity).
fn bed_bus(identity: &Identity, events: &[Event], n: usize, send: &[f64], gain: &[f64]) -> Stereo {
    let of = |t: Track| events.iter().filter(|e| e.track == t).collect::<Vec<_>>();
    let dry = render_track(&of(Track::Bed), n);
    let (mid, side) = mid_side(&dry);
    drop(dry);
    let mut bed = Chorus::juno(0.5, 0.5).run(&mid);
    add_scaled(&mut bed, &from_mid_side(&vec![0.0; n], &side), 1.0);
    for (i, g) in gain.iter().enumerate().take(n) {
        bed.l[i] *= g;
        bed.r[i] *= g;
    }
    let fed = Stereo {
        l: bed.l.iter().zip(send).map(|(x, s)| x * s).collect(),
        r: bed.r.iter().zip(send).map(|(x, s)| x * s).collect(),
    };
    let wet = Fdn::new(identity.bed_t60, 5_000.0, 25.0).run(&fed, 0.0);
    add_scaled(&mut bed, &wet, 1.0);
    let bass = render_track(&of(Track::Bass), n);
    let mut w = Warmth::new(1.3, 0.12);
    // the bass thins with the pad (a reveal's space is the whole bed's)
    let warm: Vec<f64> = bass.l.iter().zip(gain).map(|(&x, g)| g * w.process(x)).collect();
    add_scaled(
        &mut bed,
        &Stereo {
            r: warm.clone(),
            l: warm,
        },
        db_to_gain(identity.bass_db),
    );
    bed
}

/// The bed's reverb send at an intensity: 0.22 (dry, close) to 0.38.
fn reverb_send(intensity: f64) -> f64 {
    0.22 + 0.16 * intensity.clamp(0.0, 1.0)
}

/// Cue density (hard rule 5): onsets of foreground cues at least this far
/// apart, at most [`MAX_CUES_PER_30S`] in any 30 s, ticks no faster than 6/s.
pub const MIN_CUE_GAP: f64 = 1.5;
pub const MAX_CUES_PER_30S: usize = 6;
const MIN_TICK_GAP: f64 = 1.0 / 6.0;

/// Whether a cue of `kind` may sound at `t`, given the cues placed so far.
fn cue_allowed(kind: CueKind, t: f64, placed: &[(f64, CueKind)]) -> bool {
    if kind == CueKind::Tick {
        return placed
            .iter()
            .filter(|p| p.1 == CueKind::Tick)
            .all(|p| (p.0 - t).abs() >= MIN_TICK_GAP);
    }
    let fg: Vec<f64> = placed.iter().filter(|p| p.1 != CueKind::Tick).map(|p| p.0).collect();
    fg.iter().all(|&u| (u - t).abs() >= MIN_CUE_GAP)
        && fg.iter().filter(|&&u| (u - t).abs() < 30.0).count() < MAX_CUES_PER_30S
}

/// The bed's gain: 1, dipping 4 dB over the second before each reveal or
/// resolution and recovering over the 1.5 s after it, so the reveal lands in
/// a moment of space.
fn thin_outs(reveals: &[f64], n: usize) -> Vec<f64> {
    thin(reveals.iter().map(|&t| (t, t)), n, (1.0, 1.5))
}

/// The bed's gain under `spans` (start, end): 4 dB down from `ramps.0`
/// seconds before each start to its end, back up over `ramps.1` seconds,
/// raised-cosine all the way (a span of zero length is a reveal's dip).
fn thin(spans: impl Iterator<Item = (f64, f64)>, n: usize, (down, up): (f64, f64)) -> Vec<f64> {
    let mut g = vec![1.0_f64; n];
    let depth = 1.0 - db_to_gain(-4.0);
    for (t0, t1) in spans {
        let (a, b) = (((t0 - down).max(0.0) * FS) as usize, (((t1 + up) * FS) as usize).min(n));
        for (i, v) in g.iter_mut().enumerate().take(b).skip(a) {
            let s = i as f64 / FS;
            let dip = if s < t0 {
                rcos((s - (t0 - down)) / down)
            } else if s <= t1 {
                1.0
            } else {
                1.0 - rcos((s - t1) / up)
            };
            *v = (*v).min(1.0 - depth * dip);
        }
    }
    g
}

/// The melody bus: melody, answers and cues, with a plate.
fn melody_bus(events: &[Event], n: usize) -> Stereo {
    let notes: Vec<&Event> = events
        .iter()
        .filter(|e| matches!(e.track, Track::Melody | Track::Answer | Track::Cue | Track::Roll))
        .collect();
    let mut dry = render_track(&notes, n);
    let wet = Plate::new(0.55, 5_500.0).run(&dry, 0.0);
    add_scaled(&mut dry, &wet, 0.3);
    dry
}

/// The sound-effect bus: swells, whooshes and ticks.
fn sfx_bus(sfx: &[Sfx], n: usize) -> Stereo {
    let mut bus = Stereo::silence(n);
    for f in sfx {
        let buf = match f.kind {
            SfxKind::Swell { seconds } => instr::riser(seconds, f.seed),
            SfxKind::Whoosh { seconds } => instr::whoosh(seconds, 250.0, 1_600.0, 400.0, f.seed),
            SfxKind::Tick { hz } => crate::fx::pan(&instr::tick(hz, f.seed), 0.3),
        };
        bus.mix_in(&buf, (f.start.max(0.0) * FS).round() as usize, 1.0);
    }
    bus
}

/// The melody's note events for a phrase starting at `start` (all notes at
/// the reference velocity): used to calibrate the melody bus.
fn reference_phrase(identity: &Identity, start: f64) -> Vec<Event> {
    fit_phrase(
        &identity.motif,
        &identity.key,
        identity.tonic_midi,
        identity.beat_s,
        start,
        f64::INFINITY,
        identity.register.upper.0,
    )
    .into_iter()
    .enumerate()
    .map(|(i, (t, p, d))| Event {
        start: t,
        hold: d * 1.4,
        track: Track::Melody,
        inst: identity.palette.melody,
        pitch: f64::from(p),
        velocity: 0.56,
        seed: seed_for(&identity.name, "", "reference", i as u64),
        pan: 0.0,
    })
    .collect()
}

/// The loudness (LUFS) of a reference bed (the tonic chord at intensity 0.5,
/// measured over its sustained middle) and of a reference phrase (the motif)
/// through their buses at unity gain. Levels are set against these fixed
/// points, never against the video's own music, so one chapter's plan cannot
/// move another chapter's level.
pub fn reference_levels(identity: &Identity) -> (f64, f64) {
    let n = (10.0 * FS) as usize;
    let chord = identity.progression[0].clone();
    let span = ChordSpan {
        start: 0.0,
        end: 9.0,
        voicing: voice_lead(&chord, 4, None, &identity.register),
        chord,
    };
    let bed = bed_bus(
        identity,
        &chord_events(identity, &span, 0.5, "", 0),
        n,
        &vec![reverb_send(0.5); n],
        &vec![1.0; n],
    );
    let (a, b) = ((3.0 * FS) as usize, (8.0 * FS) as usize);
    let bed_lufs = meter::integrated(&Stereo {
        l: bed.l[a..b].to_vec(),
        r: bed.r[a..b].to_vec(),
    });
    let phrase = reference_phrase(identity, 0.5);
    let len = phrase.last().map_or(1.0, |e| e.start + e.hold) + 3.0;
    let mel_lufs = meter::integrated(&melody_bus(&phrase, (len * FS) as usize));
    (bed_lufs, mel_lufs)
}

/// The loudness (LUFS) of a reference sound effect (a 0.8 s whoosh) at unity
/// gain: the fixed point the sfx bus is set against.
pub fn reference_sfx_level(identity: &Identity) -> f64 {
    let f = Sfx {
        start: 0.1,
        kind: SfxKind::Whoosh { seconds: 0.8 },
        seed: seed_for(&identity.name, "", "reference-sfx", 0),
    };
    meter::integrated(&sfx_bus(&[f], (1.5 * FS) as usize))
}

/// The engine's own source: any change to how notes are chosen or sounds are
/// made changes every fingerprint, so a stale stem is never reused.
const ENGINE_SOURCE: [&str; 14] = [
    include_str!("score.rs"),
    include_str!("instr.rs"),
    include_str!("fx.rs"),
    include_str!("filter.rs"),
    include_str!("osc.rs"),
    include_str!("theory.rs"),
    include_str!("duck.rs"),
    include_str!("timeline.rs"),
    include_str!("resample.rs"),
    include_str!("rng.rs"),
    include_str!("units.rs"),
    include_str!("meter.rs"),
    include_str!("wav.rs"),
    include_str!("lib.rs"),
];

/// A digest of everything a render depends on: the engine's source and
/// version, the identity, each chapter's timeline and plan, the bytes of
/// every narration WAV, and the voice gain. A cached stem is reused only
/// under the same fingerprint.
pub fn fingerprint(identity: &Identity, sections: &[Section], voice_gain_db: Option<f64>) -> io::Result<u64> {
    let mut bytes = Vec::new();
    bytes.extend(env!("CARGO_PKG_VERSION").as_bytes());
    for src in ENGINE_SOURCE {
        bytes.extend(src.as_bytes());
    }
    bytes.extend(format!("{identity:?}|{voice_gain_db:?}").as_bytes());
    for s in sections {
        bytes.extend(s.timeline.to_text().as_bytes());
        bytes.extend(format!("{:?}", s.plan).as_bytes());
        let sounds = s.timeline.listens.iter().filter_map(|w| w.sound.as_ref());
        for path in s.timeline.lines.iter().map(|l| &l.wav).chain(sounds) {
            let wav = std::fs::read(path)?;
            bytes.extend((wav.len() as u64).to_le_bytes());
            bytes.extend(wav);
        }
    }
    Ok(crate::rng::fnv1a64(&bytes))
}

/// Score a whole video: `sections` in order, the voice placed at each line's
/// own gain or at `voice_gain_db` when given.
///
/// Everything a chapter contributes depends only on that chapter (its plan,
/// its timeline, the identity) and on the voice's integrated loudness: its
/// harmony is planned inside it, its seeds are its own, and the levels are
/// set against [`reference_levels`]. Only tails that ring across a cut (a
/// few seconds of release and reverb) and the true-peak cap, if it ever
/// binds, reach a neighbour.
pub fn render(identity: &Identity, sections: &[Section], voice_gain_db: Option<f64>) -> io::Result<Score> {
    identity.validate()?;
    if sections.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "no chapters to score"));
    }
    // 1. the voice bus of the whole video, chapter after chapter
    let mut chapters = Vec::new();
    let mut voice = Stereo::default();
    for s in sections {
        let v = s.timeline.voice_bus(voice_gain_db)?;
        let a = voice.len();
        voice.l.extend(&v.l);
        voice.r.extend(&v.r);
        chapters.push((s.timeline.chapter.clone(), a, voice.len()));
    }
    let n = voice.len();
    let total = n as f64 / FS;
    let chapter_starts: Vec<f64> = chapters.iter().map(|c| c.1 as f64 / FS).collect();
    let chapter_at = |t: f64| chapter_starts.iter().rposition(|&s| s <= t).unwrap_or(0);

    // 2. speech
    let active = duck::vad(&voice);
    let speech = duck::regions(&active, 1.2);
    let fine = duck::regions(&active, MIN_PAUSE);
    // authored silences: no note starts inside one, and the stem is zero there
    let (silences, notes) = authored_silences(sections, &chapter_starts, &fine, &speech, total);
    let silent = |t: f64| silences.iter().any(|s| t >= s.fade_start && t < s.back);

    // 3. harmony, planned inside each chapter from its own progression (its
    //    mood's or the identity's): the identity's is entered where the
    //    chapter's index puts it, so consecutive chapters keep moving; a
    //    mood's starts on its first chord. Under a listen window that carries
    //    a sonification the bed holds the listen chord. The video's last
    //    chord is the progression's first when the plan resolves
    let resolve = sections.last().is_some_and(|s| s.plan.resolve);
    let mut chords: Vec<ChordSpan> = Vec::new();
    let mut events = Vec::new();
    for (c, (name, a, b)) in chapters.iter().enumerate() {
        let (t0, t1) = (*a as f64 / FS, *b as f64 / FS);
        let progression = identity.progression_for(name);
        let len = progression.len();
        let in_mood = progression != identity.progression;
        let local: Vec<(f64, f64)> = fine
            .iter()
            .filter(|r| r.1 > t0 && r.0 < t1)
            .map(|&(s, e)| (s.max(t0) - t0, e.min(t1) - t0))
            .collect();
        let cands = change_candidates(&local, t1 - t0);
        let fps = f64::from(sections[c].timeline.fps);
        let listens: Vec<(f64, f64)> = match identity.listen_chord {
            Some(_) => sections[c]
                .timeline
                .listens
                .iter()
                .filter(|w| w.sound.is_some())
                .map(|w| {
                    let s = w.frame as f64 / fps;
                    let until = cands.iter().copied().find(|&x| x >= s + w.seconds).unwrap_or(t1 - t0);
                    (s, until)
                })
                .collect(),
            None => Vec::new(),
        };
        let mut rng = Rng::new(seed_for(&identity.name, name, "harmony", 0));
        let changes = plan_with_listens(&cands, t1 - t0, identity.chord_s, &listens, &mut rng);
        let steps = changes.iter().filter(|x| !x.1).count();
        // the resolving chapter enters the progression so that its last chord
        // is the tonic: the cycle is kept whole and nothing is merged
        let offset = if c + 1 == chapters.len() && resolve {
            (1 + len * steps - steps) % len
        } else if in_mood {
            0
        } else {
            c % len
        };
        let mut prev: Option<Voicing> = None;
        let mut mine: Vec<ChordSpan> = Vec::new();
        let mut k = 0;
        for (j, &(s, listen)) in changes.iter().enumerate() {
            let (start, end) = (t0 + s, changes.get(j + 1).map_or(t1, |x| t0 + x.0));
            let chord = match (&identity.listen_chord, listen) {
                (Some(l), true) => l.clone(),
                _ => {
                    k += 1;
                    progression[(offset + k - 1) % len].clone()
                }
            };
            // a progression that repeats a chord just holds it
            if let Some(p) = mine.last_mut().filter(|p| p.chord == chord) {
                p.end = end;
                continue;
            }
            let voicing = voice_lead(&chord, 4, prev.as_ref(), &identity.register);
            prev = Some(voicing.clone());
            mine.push(ChordSpan {
                start,
                end,
                chord,
                voicing,
            });
        }
        for (k, span) in mine.iter().enumerate() {
            events.extend(chord_events(identity, span, sections[c].plan.intensity, name, k));
        }
        chords.extend(mine);
    }
    let chord_at = |t: f64| {
        chords
            .iter()
            .find(|c| t >= c.start && t < c.end)
            .or(chords.last())
            .expect("at least one chord")
    };

    // 4. the gaps, numbered within each chapter. A gap that holds a
    //    sonification is the sonification's (over the listen chord); a
    //    soundless listen window the identity names gets its theme; the
    //    video's last gap resolves when the plan does; any other gap gets the
    //    leitmotif, varied, and a long one first a slow roll of the chord
    let mut phrases = Vec::new();
    let gaps = duck::gaps(&speech, total, MIN_PHRASE_GAP);
    let mut gap_index = vec![0usize; chapters.len()];
    // where the authored note cues (reveal, pizzicato, resolution) will land:
    // on the mark, or at the end of the sentence it falls in (as step 5
    // places them). A gap one lands in is that cue's: no roll there, and the
    // motif only after it (one gesture at a time; a roll and a reveal's dyad
    // together were seven onsets in a second on C's real narration)
    let authored: Vec<f64> = sections
        .iter()
        .zip(&chapter_starts)
        .flat_map(|(s, &t0)| {
            let fps = f64::from(s.timeline.fps);
            s.timeline
                .marks
                .iter()
                .filter(|m| {
                    matches!(
                        CueKind::from_label(&m.label),
                        CueKind::Reveal | CueKind::Pizz | CueKind::Resolve
                    )
                })
                .map(move |m| t0 + m.frame as f64 / fps)
        })
        .map(|t| match fine.iter().find(|r| t >= r.0 - ONSET_GUARD && t < r.1) {
            Some(r) => r.1 + 0.05,
            None => t,
        })
        .collect();
    let theme_motif = || {
        let mut m = identity.motif.clone();
        m.degrees.extend(&identity.resolution.degrees);
        m.durations.extend(&identity.resolution.durations);
        m
    };
    for (g, &(a, b)) in gaps.iter().enumerate() {
        let ch = chapter_at(a);
        let gi = gap_index[ch];
        gap_index[ch] += 1;
        if silent(a + 0.35) {
            continue;
        }
        let name = chapters[ch].0.as_str();
        let fps = f64::from(sections[ch].timeline.fps);
        let windows: Vec<&crate::timeline::Window> = sections[ch]
            .timeline
            .listens
            .iter()
            .filter(|w| {
                let s = chapter_starts[ch] + w.frame as f64 / fps;
                s >= a - 0.1 && s < b
            })
            .collect();
        if windows.iter().any(|w| w.sound.is_some()) {
            continue;
        }
        let theme = windows.iter().find_map(|w| identity.theme_for(&w.label));
        let mut prng = Rng::new(seed_for(&identity.name, name, "phrase", gi as u64));
        let final_gap = g + 1 == gaps.len() && resolve;
        let resolving = final_gap || theme == Some(Theme::Resolved);
        let motif = match theme {
            Some(Theme::Open) => identity.motif.clone(),
            _ if resolving => theme_motif(),
            _ => match prng.uniform() {
                u if u < 0.45 => identity.motif.clone(),
                u if u < 0.65 => identity.motif.transposed(2),
                u if u < 0.85 => identity.motif.inverted(),
                _ => identity.motif.augmented(1.5),
            },
        };
        // start once the voice has cleared; the last onset leaves room for
        // the duck to come down before the next line (pre-roll + attack)
        let opening = if a <= 0.0 { 0.9 } else { a + 0.35 };
        let limit = if b >= total { total - 2.5 } else { b - 0.6 };
        // the pedal comes up before the voice returns
        let lift = if b >= total { total - 0.5 } else { b - PEDAL_CLEAR };
        let mut start = opening;
        let answered = authored.iter().copied().find(|&c| c >= a - 0.2 && c < b);
        if let Some(c) = answered {
            start = start.max(c + AFTER_CUE);
        }
        if let Some(inst) = identity
            .palette
            .roll
            .filter(|_| b - a >= ROLL_GAP && answered.is_none())
        {
            // a theme is struck over one soft block chord; otherwise the
            // chord unfolds from the bass before the motif enters. Either
            // enters with the harmony: on the gap's chord change, if one
            // comes within its first 0.9 s (a theme) or 1.5 s (a roll).
            // While the pedal is down, each later change of chord is struck
            // again, softly, as a pianist changes pedal
            let block = theme.is_some() || resolving;
            let wait = if block { 0.9 } else { 1.5 };
            let at = chords
                .iter()
                .map(|c| c.start)
                .find(|&s| s >= opening && s < opening + wait)
                .unwrap_or(opening);
            let mut strikes = vec![at];
            strikes.extend(
                chords
                    .iter()
                    .map(|c| c.start)
                    .filter(|&s| s > at + 0.5 && s < lift - 0.8),
            );
            for (j, &strike) in strikes.iter().enumerate() {
                let span = chord_at(strike + 1e-6);
                let release = span.end.min(lift);
                if release - strike < 0.8 {
                    continue;
                }
                let rolled = !block && j == 0;
                for (i, p) in roll_voicing(span).into_iter().enumerate() {
                    let t = if rolled { strike + ROLL_STEP * i as f64 } else { strike };
                    let v = if rolled {
                        [0.4, 0.34, 0.38, 0.4, 0.44][i.min(4)]
                    } else {
                        0.3
                    };
                    events.push(Event {
                        start: t,
                        hold: release - t,
                        track: Track::Roll,
                        inst,
                        pitch: f64::from(p),
                        velocity: v + 0.03 * prng.jitter(1.0),
                        seed: seed_for(&identity.name, name, "roll", (gi * 32 + j * 8 + i) as u64),
                        pan: if i < 2 { -0.25 } else { 0.1 + 0.08 * i as f64 },
                    });
                }
                if j == 0 {
                    start = if rolled { at + 5.0 * ROLL_STEP + 0.3 } else { at };
                }
            }
        }
        let mut notes = elastic_phrase(&motif, identity, start, limit, resolving);
        if resolving && notes.len() < motif.degrees.len() {
            // no room for the whole closing phrase: the resolution alone, so
            // the question is still answered on the tonic it promised
            let res = elastic_phrase(&identity.resolution, identity, start, limit, true);
            if res.len() == identity.resolution.degrees.len() {
                notes = res;
            }
        }
        let Some(&(last_t, _, last_d)) = notes.last() else {
            continue;
        };
        for (i, &(t, p, d)) in notes.iter().enumerate() {
            let p = avoid_minor_ninths(p, &chord_at(t).chord, &identity.key, identity.register.upper.0);
            events.push(Event {
                start: t,
                hold: d * 1.4,
                track: Track::Melody,
                inst: identity.palette.melody,
                pitch: f64::from(p),
                velocity: 0.5 + 0.12 * prng.uniform() + if i == 0 { 0.06 } else { 0.0 },
                seed: seed_for(&identity.name, name, "melody", (gi * 16 + i) as u64),
                pan: 0.12 * prng.jitter(1.0),
            });
        }
        phrases.push((notes[0].0, last_t));
        // a soft answer: two chord tones an octave up (each kept off a
        // minor ninth, as a reveal's are), if there is room
        let t = last_t + last_d + 0.25;
        if t + 0.4 <= limit {
            let span = chord_at(t);
            let dyad = reveal_dyad(&span.voicing, &span.chord, &identity.key, identity.register.upper.0);
            for (i, &p) in dyad.iter().enumerate() {
                events.push(Event {
                    start: t + 0.03 * i as f64,
                    hold: 1.2,
                    track: Track::Answer,
                    inst: identity.palette.answer,
                    pitch: f64::from(p),
                    velocity: 0.32,
                    seed: seed_for(&identity.name, name, "answer", (gi * 4 + i) as u64),
                    pan: if i == 0 { -0.4 } else { 0.4 },
                });
            }
        }
    }

    // 5. authored marks, by kind: a reveal, pizzicato or resolution goes on
    //    the mark, or where the sentence it falls in ends (if that is within
    //    2 s), else it is skipped (a mark on a sentence's onset, within the
    //    onset guard, is in that sentence: a note struck there masks the
    //    first syllable); a swell rises into the mark, a whoosh is
    //    centred on it, a tick lands on it even under speech (a counter must
    //    stay in sync with what it counts; the duck and pocket keep it soft)
    let next_onset = |t: f64| fine.iter().map(|r| r.0).find(|&s| s > t).unwrap_or(f64::INFINITY);
    let mut cues = Vec::new();
    let mut placed: Vec<(f64, CueKind)> = Vec::new();
    let mut skipped_marks = Vec::new();
    let mut sfx = Vec::new();
    for (c, s) in sections.iter().enumerate() {
        let t0 = chapter_starts[c];
        let name = chapters[c].0.as_str();
        for (i, m) in s.timeline.marks.iter().enumerate() {
            let t = t0 + m.frame as f64 / f64::from(s.timeline.fps);
            let kind = CueKind::from_label(&m.label);
            match kind {
                // the silences are planned above; a thin is the bed's alone
                CueKind::Silence | CueKind::SilenceEnd => continue,
                CueKind::Thin if !silent(t) => {
                    cues.push((t, m.label.clone()));
                    continue;
                }
                _ if silent(t) => {
                    skipped_marks.push(format!("{name}/{}", m.label));
                    continue;
                }
                _ => {}
            }
            let seed = |track: &str, j: usize| seed_for(&identity.name, name, track, (i * 8 + j) as u64);
            let when = match kind {
                CueKind::Swell | CueKind::Whoosh | CueKind::Tick => Some(t),
                _ => match fine.iter().find(|r| t >= r.0 - ONSET_GUARD && t < r.1) {
                    None => Some(t),
                    Some(r) if r.1 - t <= 2.0 => Some(r.1 + 0.05),
                    Some(_) => None,
                },
            };
            // a note cue needs a clear second before the next sentence
            let when = when.filter(|&w| {
                !matches!(kind, CueKind::Reveal | CueKind::Pizz | CueKind::Resolve) || next_onset(w) - w >= CUE_MIN_GAP
            });
            // no room (deep in a sentence, at the very end), or too dense (rule 5)
            let Some(when) = when.filter(|&w| w < total - 0.5 && cue_allowed(kind, w, &placed)) else {
                skipped_marks.push(format!("{name}/{}", m.label));
                continue;
            };
            placed.push((when, kind));
            let (v, chord) = (&chord_at(when).voicing, &chord_at(when).chord);
            match kind {
                CueKind::Reveal => {
                    let dyad = reveal_dyad(v, chord, &identity.key, identity.register.upper.0);
                    if dyad.is_empty() {
                        skipped_marks.push(format!("{name}/{}", m.label));
                        continue;
                    }
                    for (j, p) in dyad.into_iter().enumerate() {
                        events.push(Event {
                            start: when + 0.04 * j as f64,
                            hold: 1.5,
                            track: Track::Cue,
                            inst: identity.palette.answer,
                            pitch: f64::from(p),
                            velocity: 0.4,
                            seed: seed("cue", j),
                            pan: if j == 0 { -0.45 } else { 0.45 },
                        });
                    }
                }
                CueKind::Pizz => {
                    let p = v.upper[v.upper.len() / 2];
                    let p = avoid_minor_ninths(
                        if p < 67 { p + 12 } else { p },
                        chord,
                        &identity.key,
                        identity.register.upper.0,
                    );
                    if clashes(p, chord) {
                        skipped_marks.push(format!("{name}/{}", m.label));
                        continue;
                    }
                    events.push(Event {
                        start: when,
                        hold: 0.4,
                        track: Track::Cue,
                        inst: Instrument::Pluck,
                        pitch: f64::from(p),
                        velocity: 0.45,
                        seed: seed("pizz", 0),
                        pan: 0.35,
                    });
                }
                CueKind::Resolve => {
                    let notes = fit_phrase(
                        &identity.resolution,
                        &identity.key,
                        identity.tonic_midi,
                        identity.beat_s,
                        when,
                        f64::INFINITY,
                        identity.register.upper.0,
                    );
                    for (j, (nt, np, nd)) in notes.into_iter().enumerate() {
                        events.push(Event {
                            start: nt,
                            hold: nd * 1.4,
                            track: Track::Cue,
                            inst: identity.palette.melody,
                            pitch: f64::from(avoid_minor_ninths(
                                np,
                                &chord_at(nt).chord,
                                &identity.key,
                                identity.register.upper.0,
                            )),
                            velocity: 0.55,
                            seed: seed("resolve", j),
                            pan: 0.1,
                        });
                    }
                }
                CueKind::Swell => sfx.push(Sfx {
                    start: (when - 2.0).max(0.0),
                    kind: SfxKind::Swell { seconds: when.min(2.0) },
                    seed: seed("swell", 0),
                }),
                CueKind::Whoosh => sfx.push(Sfx {
                    start: (when - 0.4).max(0.0),
                    kind: SfxKind::Whoosh { seconds: 0.8 },
                    seed: seed("whoosh", 0),
                }),
                CueKind::Tick => sfx.push(Sfx {
                    start: when,
                    kind: SfxKind::Tick { hz: 700.0 },
                    seed: seed("tick", 0),
                }),
                // handled before the cue is placed
                CueKind::Thin | CueKind::Silence | CueKind::SilenceEnd => {}
            }
            cues.push((when, m.label.clone()));
        }
    }
    // nothing of the score's own starts inside an authored silence (a phrase
    // that began before it was faded and then zeroed)
    events.retain(|e| matches!(e.track, Track::Bed | Track::Bass) || !silent(e.start));
    sfx.retain(|f| !silent(f.start));
    phrases.retain(|p: &(f64, f64)| !silent(p.0));

    // rule 7 for as long as a note sounds: a melody, answer or cue note held
    // into a chord it would clash with is released at that chord's change;
    // one that would clash almost at once is not played (flooring its hold
    // would sound it over the clash, and a blip is no note)
    // (a cue note is also released before the next sentence begins)
    events.retain_mut(|e| {
        if !matches!(e.track, Track::Melody | Track::Answer | Track::Cue | Track::Roll) {
            return true;
        }
        e.hold = hold_until_clash(e.pitch.round() as i32, e.start, e.hold, &chords);
        let clear = match e.track {
            Track::Cue => CUE_CLEAR,
            Track::Melody | Track::Answer => GAP_CLEAR,
            _ => 0.0,
        };
        e.hold = e.hold.min(next_onset(e.start) - e.start - clear);
        e.hold >= MIN_NOTE_S
    });

    // 6. buses and levels, against the fixed reference levels
    let voice_lufs = meter::integrated(&voice);
    // a video with no narration still gets music, set against the level a
    // narrated sentence would have (−23 LUFS, the narration doctrine's)
    let v_ref = if voice_lufs.is_finite() { voice_lufs } else { -23.0 };
    let (ref_bed, ref_mel) = reference_levels(identity);
    let ref_sfx = reference_sfx_level(identity);
    if !(ref_bed.is_finite() && ref_mel.is_finite() && ref_sfx.is_finite()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "identity {}: a silent reference ({ref_bed}, {ref_mel}, {ref_sfx} LUFS)",
                identity.name
            ),
        ));
    }
    // per-sample reverb send from each chapter's intensity, moving over the
    // first 2 s of a chapter (like a tail, a change reaches forward over a
    // cut, never back), and the thin-outs before reveals
    let mut send_points = Vec::new();
    for (c, (_, a, b)) in chapters.iter().enumerate() {
        let s = reverb_send(sections[c].plan.intensity);
        let (t0, t1) = (*a as f64 / FS, *b as f64 / FS);
        if c == 0 {
            send_points.push((t0, s));
        } else {
            let prev = reverb_send(sections[c - 1].plan.intensity);
            send_points.push((t0, prev));
            send_points.push(((t0 + 2.0).min(t1), s));
        }
    }
    let send = crate::fx::Automation::new(&send_points).render(n);
    let reveals: Vec<f64> = cues
        .iter()
        .filter(|c| {
            matches!(
                CueKind::from_label(&c.1),
                CueKind::Reveal | CueKind::Resolve | CueKind::Thin
            )
        })
        .map(|c| c.0)
        .collect();
    // the bed also makes room under every sonification (an identity with a
    // listen chord: the new engine's), from just before its window to its end
    let mut room = thin_outs(&reveals, n);
    if identity.listen_chord.is_some() {
        let spans = sections.iter().zip(&chapter_starts).flat_map(|(s, &t0)| {
            let fps = f64::from(s.timeline.fps);
            s.timeline
                .listens
                .iter()
                .filter(|w| w.sound.is_some())
                .map(move |w| (t0 + w.frame as f64 / fps, t0 + w.frame as f64 / fps + w.seconds))
        });
        for (r, x) in room.iter_mut().zip(thin(spans, n, (0.3, 1.0))) {
            *r = r.min(x);
        }
    }
    let mut sum = bed_bus(identity, &events, n, &send, &room);
    drop(send);
    sum.scale(db_to_gain(v_ref + identity.bed_lu - ref_bed));
    let melody = melody_bus(&events, n);
    add_scaled(&mut sum, &melody, db_to_gain(v_ref + identity.melody_lu - ref_mel));
    drop(melody);
    if !sfx.is_empty() {
        let effects = sfx_bus(&sfx, n);
        add_scaled(&mut sum, &effects, db_to_gain(v_ref + identity.sfx_lu - ref_sfx));
    }
    // listen windows' sonifications at V + listen_lu, each cut at its
    // window's end with a short fade so it never runs into the next line
    for (c, s) in sections.iter().enumerate() {
        let (a, b) = (chapters[c].1, chapters[c].2);
        for w in &s.timeline.listens {
            let Some(path) = &w.sound else { continue };
            let start = a + frame_to_sample(w.frame, s.timeline.fps) as usize;
            let end = (start + (w.seconds * FS).round() as usize).min(b);
            let sound = listen_sound(path, end.saturating_sub(start), identity.settle_s.max(0.25))?;
            let lufs = meter::integrated(&sound);
            if !lufs.is_finite() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("{}: a silent sonification", path.display()),
                ));
            }
            sum.mix_in(&sound, start, db_to_gain(v_ref + identity.listen_lu - lufs));
        }
    }
    // tape glue: one transport, so both channels share the wow and flutter
    let tape_seed = seed_for(&identity.name, "", "tape", 0);
    let (mut tl, mut tr) = (Tape::new(tape_seed), Tape::new(tape_seed));
    for v in &mut sum.l {
        *v = tl.process(*v);
    }
    for v in &mut sum.r {
        *v = tr.process(*v);
    }

    // 7. the duck, with breaths and listen windows released. In a hold the
    //    author did not write for music (a gap under HOLD_MERGE s), the duck
    //    lets the music up only part way (plan §7.2: the music answers fully
    //    in the authored gaps); a 3 s window straddling such a hold's edge
    //    otherwise hears the whole gap's music against half a window of voice
    let shape = DuckShape::default();
    let mut u = duck::duck_curve(&speech, n, shape);
    if identity.hold_duck > 0.0 {
        let held = duck::duck_curve(&duck::regions(&active, HOLD_MERGE), n, shape);
        for (x, h) in u.iter_mut().zip(&held) {
            *x = x.max(identity.hold_duck * h);
        }
    }
    let windows: Vec<(f64, f64)> = sections
        .iter()
        .zip(&chapter_starts)
        .flat_map(|(s, &t0)| {
            let fps = f64::from(s.timeline.fps);
            s.timeline
                .breaths
                .iter()
                .chain(&s.timeline.listens)
                .map(move |w| (t0 + w.frame as f64 / fps, w.seconds))
        })
        .collect();
    duck::release_windows(&mut u, &windows, shape);
    if identity.settle_s > 0.0 {
        // an authored window the voice resumes from (within 0.5 s of its end)
        let next = |t: f64| speech.iter().map(|r| r.0).find(|&s| s >= t - 0.05);
        let resumes: Vec<f64> = windows
            .iter()
            .filter_map(|&(s, len)| next(s + len).filter(|&o| o - (s + len) < 0.5))
            .collect();
        duck::recede(&mut u, &resumes, identity.settle_s);
    }
    let mut music = duck::apply(&sum, &u, identity.duck_db);
    drop(sum);
    // the authored silences, applied to everything (bed, notes, tails,
    // sonifications): exactly zero inside
    if !silences.is_empty() {
        for (i, g) in silence_gain(&silences, n).into_iter().enumerate() {
            music.l[i] *= g;
            music.r[i] *= g;
        }
    }

    // 8. the video's edges: a 1 s fade-in, a 3 s fade-out
    let fade_in = (1.0 * FS) as usize;
    let fade_out = (3.0 * FS) as usize;
    for i in 0..n {
        let g = rcos(i as f64 / fade_in as f64) * rcos((n - 1 - i) as f64 / fade_out as f64);
        music.l[i] *= g;
        music.r[i] *= g;
    }

    // 9. the true-peak cap: a fixed gain, never a limiter
    let mut true_peak_db = meter::true_peak_db(&music);
    let mut cap_db = 0.0;
    // a 0.5 dB margin: a chapter's slice re-measured alone can read a hair
    // higher than the whole (the interpolator sees a cut edge)
    if true_peak_db > -12.5 {
        cap_db = -12.5 - true_peak_db;
        music.scale(db_to_gain(cap_db));
        true_peak_db += cap_db;
    }

    Ok(Score {
        music,
        register: identity.register,
        voice,
        events,
        chords,
        phrases,
        cues,
        skipped_marks,
        sfx,
        speech,
        chapters,
        voice_lufs,
        true_peak_db,
        cap_db,
        silences,
        notes,
    })
}

/// A listen window's sonification: decoded, at 48 kHz, through the score's
/// own top end (a 4th-order 5.5 kHz low-pass: a mallet or pluck struck in a
/// quiet window otherwise lands its upper partials above 8 kHz as a click,
/// §7.6), cut to `max` samples (the window) and faded over the window's
/// last `settle` seconds, so it recedes before the voice returns.
fn listen_sound(path: &Path, max: usize, settle: f64) -> io::Result<Stereo> {
    let d = crate::wav::read(path).map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", path.display())))?;
    let to48 = |x: &[f64]| crate::resample::resample(x, d.rate, crate::SAMPLE_RATE);
    let mut s = match d.samples.as_slice() {
        [l, r] => Stereo { l: to48(l), r: to48(r) },
        [m] => Stereo::from_mono(&to48(m)),
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{}: {} channels (mono or stereo)", path.display(), d.channels),
            ));
        }
    };
    for ch in [&mut s.l, &mut s.r] {
        let lp = || Biquad::new(Kind::LowPass { q: 0.707 }, LISTEN_TOP_HZ);
        let (mut a, mut b) = (lp(), lp());
        for v in ch.iter_mut() {
            *v = b.process(a.process(*v));
        }
    }
    // cut at the window's end, faded over its last `settle` seconds (to an
    // exact zero at the end) wherever the sound still sounds there
    s.l.truncate(max);
    s.r.truncate(max);
    let fade = ((settle * FS) as usize).clamp(1, max.max(1));
    for i in max.saturating_sub(fade)..s.len() {
        let g = rcos((max - 1 - i) as f64 / fade as f64);
        s.l[i] *= g;
        s.r[i] *= g;
    }
    Ok(s)
}

/// The top of a sonification as the score plays it (Hz).
const LISTEN_TOP_HZ: f64 = 5_500.0;

/// Sample index of a chapter-relative frame (for callers placing stems).
pub fn frame_sample(frame: u64, fps: u32) -> usize {
    frame_to_sample(frame, fps) as usize
}

/// Frequency of a MIDI pitch (convenience for QA).
pub fn hz(pitch: f64) -> f64 {
    midi_to_hz(pitch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theory::IONIAN;

    #[test]
    fn candidates_respect_pauses_and_onsets() {
        // speech 1–4 s, a 0.1 s pause (too short), 4.1–6 s, a 0.5 s pause, 6.5–9 s
        let fine = [(1.0, 4.0), (4.1, 6.0), (6.5, 9.0)];
        let c = change_candidates(&fine, 12.0);
        assert!(c.contains(&0.0) && c.contains(&0.5), "{c:?}");
        assert!(
            !c.iter().any(|&t| (4.0..4.1).contains(&t)),
            "the 0.1 s pause is too short"
        );
        assert!(
            c.iter().any(|&t| (t - 6.05).abs() < 1e-9),
            "just after the 0.5 s pause opens"
        );
        for &t in &c {
            for &(s, e) in &fine {
                assert!(!(t >= s && t < e), "{t} inside speech");
                assert!(!(t >= s && t < s + ONSET_GUARD), "{t} within 150 ms of an onset");
            }
        }
        assert!(c.iter().any(|&t| t > 9.0 && t < 12.0), "the closing silence");
        assert!(c.iter().all(|&t| t < 12.0));
        // a span that starts in speech (a sentence carried over a cut) has no
        // candidate until that sentence ends
        let c = change_candidates(&[(0.0, 3.0)], 6.0);
        assert!(c.iter().all(|&t| t >= 3.0), "{c:?}");
        // no speech at all: the whole span, every 0.5 s
        assert_eq!(change_candidates(&[], 2.0), vec![0.0, 0.5, 1.0, 1.5]);
    }

    #[test]
    fn changes_land_on_candidates_within_the_length_range() {
        let cands: Vec<f64> = (0..400).map(|k| k as f64 * 0.37).collect();
        let total = 140.0;
        let mut rng = Rng::new(7);
        let ch = plan_changes(&cands, total, (6.0, 12.0), &mut rng);
        assert_eq!(ch[0], 0.0);
        let lengths: Vec<f64> = ch
            .windows(2)
            .map(|w| w[1] - w[0])
            .chain([total - ch.last().unwrap()])
            .collect();
        for &d in &lengths {
            assert!((6.0..=12.0).contains(&d), "{d} in {lengths:?}");
        }
        for &c in &ch[1..] {
            assert!(cands.iter().any(|&k| (k - c).abs() < 1e-9));
        }
        // short spans in silence, every seed: the last chord never runs long
        // (a target near the end must not strand the rest as one chord)
        let dense: Vec<f64> = (0..60).map(|k| k as f64 * 0.5).collect();
        for total in [13.0, 17.0, 23.5, 29.0] {
            for seed in 0..64 {
                let ch = plan_changes(&dense, total, (6.0, 12.0), &mut Rng::new(seed));
                let last = total - ch.last().unwrap();
                assert!((6.0..=12.0).contains(&last), "total {total} seed {seed}: {ch:?}");
                for w in ch.windows(2) {
                    assert!(
                        (6.0..=12.0).contains(&(w[1] - w[0])),
                        "total {total} seed {seed}: {ch:?}"
                    );
                }
            }
        }
        // no candidate for a long stretch: the chord holds (a pedal)
        let sparse = [0.0, 3.0, 30.0, 60.0];
        let ch = plan_changes(&sparse, 80.0, (6.0, 12.0), &mut Rng::new(1));
        assert_eq!(ch, vec![0.0, 30.0, 60.0]);
        // a span shorter than the longest chord is one chord
        assert_eq!(plan_changes(&dense, 11.0, (6.0, 12.0), &mut Rng::new(3)), vec![0.0]);
        // 16.25 s left and no pause where the preferred 8–14 s lengths want
        // one (found on an audition's chapter end): the most central pause
        // that leaves both chords 4 s or more, never one chord past 16 s
        for seed in 0..16 {
            let ch = plan_changes(&[0.0, 2.0, 6.5, 10.0, 14.0], 16.25, (8.0, 14.0), &mut Rng::new(seed));
            assert_eq!(ch, vec![0.0, 6.5], "seed {seed}");
        }
        // 15.5 s with no pause in range: one chord, under the 16 s limit
        assert_eq!(
            plan_changes(&[0.0, 6.5], 15.5, (8.0, 14.0), &mut Rng::new(1)),
            vec![0.0]
        );
    }

    #[test]
    fn phrases_fit_their_gap_and_stay_in_register() {
        let key = Key::new(2, IONIAN);
        let m = Motif::new(&[1, 5, 6, 3], &[1.0, 1.0, 1.0, 2.0]);
        let all = fit_phrase(&m, &key, 62, 0.8, 10.0, 20.0, 60);
        assert_eq!(all.iter().map(|n| n.1).collect::<Vec<_>>(), vec![62, 69, 71, 66]);
        assert!((all[3].0 - 12.4).abs() < 1e-9);
        // only three onsets fit before 11.7 s
        assert_eq!(fit_phrase(&m, &key, 62, 0.8, 10.0, 11.7, 60).len(), 3);
        assert!(
            fit_phrase(&m, &key, 62, 0.8, 10.0, 10.5, 60).is_empty(),
            "one note is not a phrase"
        );
        // a low motif is lifted to the register's floor or above: C4 for
        // robert, F3 for matt's lower voice (so A3 stays where it is)
        let low = fit_phrase(&m.transposed(-7), &key, 62, 0.8, 0.0, 9.0, 60);
        assert!(low.iter().all(|n| n.1 >= 60), "{low:?}");
        let matt = fit_phrase(&m.transposed(-7), &key, 62, 0.8, 0.0, 9.0, 53);
        assert!(matt.iter().all(|n| n.1 >= 53), "{matt:?}");
        assert!(matt.iter().any(|n| n.1 < 60), "{matt:?}");
    }

    #[test]
    fn phrasing_is_elastic_within_eight_percent() {
        let id = Identity::hoeffding();
        let b = id.beat_s;
        // onsets at 0, 1, 2, 3 beats: the last at 3 b; 2.85 b fits only tightened
        let tight = elastic_phrase(&id.motif, &id, 10.0, 10.0 + 2.85 * b, false);
        assert_eq!(tight.len(), 4);
        assert!((tight[3].0 - (10.0 + 3.0 * 0.92 * b)).abs() < 1e-9);
        // 2.7 b fits neither way: the prefix at tempo
        let short = elastic_phrase(&id.motif, &id, 10.0, 10.0 + 2.7 * b, false);
        assert_eq!(short.len(), 3);
        assert!((short[2].0 - (10.0 + 2.0 * b)).abs() < 1e-9);
        // plenty of room: at tempo, unless it is the closing phrase (relaxed)
        let free = elastic_phrase(&id.motif, &id, 10.0, 30.0, false);
        assert!((free[3].0 - (10.0 + 3.0 * b)).abs() < 1e-9);
        let closing = elastic_phrase(&id.motif, &id, 10.0, 30.0, true);
        assert!((closing[3].0 - (10.0 + 3.0 * 1.08 * b)).abs() < 1e-9);
    }

    #[test]
    fn cue_kinds_come_from_mark_labels() {
        assert_eq!(CueKind::from_label("reveal_mle"), CueKind::Reveal);
        assert_eq!(CueKind::from_label("swell:table"), CueKind::Swell);
        assert_eq!(CueKind::from_label("resolve:estimate"), CueKind::Resolve);
        assert_eq!(CueKind::from_label("tick:3"), CueKind::Tick);
        assert_eq!(CueKind::from_label("pizz:1"), CueKind::Pizz);
        assert_eq!(CueKind::from_label("whoosh:cut"), CueKind::Whoosh);
        assert_eq!(CueKind::from_label("other:x"), CueKind::Reveal);
    }

    #[test]
    fn minor_ninths_are_moved_to_safe_key_tones() {
        let key = Key::new(2, IONIAN);
        let d = Chord::maj9(2); // D F♯ A C♯ E
        // G (67) is a semitone above F♯: moved to a safe D-major tone
        let p = avoid_minor_ninths(67, &d, &key, 60);
        assert_ne!(p, 67);
        assert!(key.pitch_classes().contains(&p.rem_euclid(12)));
        assert!(
            d.pitch_classes()
                .iter()
                .all(|&c| (p - c).rem_euclid(12) != 1 && (c - p).rem_euclid(12) != 1)
        );
        // a chord tone stays
        assert_eq!(avoid_minor_ninths(66, &d, &key, 60), 66);
        // never moved down into the narrator's band: C♯4 clashes with D, and
        // B3 (246.9 Hz) is not an option under robert's register
        let p = avoid_minor_ninths(61, &d, &key, 60);
        assert!(p >= 60, "{p}");
    }

    #[test]
    fn a_held_note_is_released_where_it_would_clash() {
        let key = Key::new(2, IONIAN);
        let mk = |start: f64, end: f64, chord: Chord| ChordSpan {
            start,
            end,
            voicing: voice_lead(&chord, 4, None, &Register::ROBERT),
            chord,
        };
        // F♯4 over Bm9 is fine; from 11 s, Gmaj9♯11 has G a semitone above it
        let chords = vec![mk(0.0, 11.0, Chord::min9(11)), mk(11.0, 20.0, Chord::maj9_sharp11(7))];
        assert!(!clashes(66, &chords[0].chord));
        assert!(clashes(66, &chords[1].chord));
        assert!((hold_until_clash(66, 10.0, 2.5, &chords) - 1.0).abs() < 1e-9);
        // a note that is safe over both keeps its hold
        assert_eq!(hold_until_clash(71, 10.0, 2.5, &chords), 2.5);
        let _ = key;
    }

    #[test]
    fn every_corpus_identity_plays_and_none_repeats_another_s_harmony() {
        let ids = [
            Identity::hoeffding(),
            Identity::capture_recapture(),
            Identity::raptorq_gf256(),
            Identity::bakery_algorithm(),
        ];
        for id in &ids {
            id.validate().unwrap();
        }
        // R1 X-1: the corpus played one I–vi–IV–V loop in four keys; the root
        // motion (each root relative to the first) must differ between videos
        let motion = |id: &Identity| -> Vec<i32> {
            let r0 = id.progression[0].root;
            id.progression.iter().map(|c| (c.root - r0).rem_euclid(12)).collect()
        };
        for (i, a) in ids.iter().enumerate() {
            for b in &ids[i + 1..] {
                assert_ne!(motion(a), motion(b), "{} and {} share a progression", a.name, b.name);
            }
        }
        // B's motif is 0x1D's set bits as E–G♯–B–C♯; A's opens on 5̂ below the tonic
        let b = Identity::raptorq_gf256();
        let pcs: Vec<i32> = b
            .motif
            .degrees
            .iter()
            .map(|&d| b.key.degree(d, b.tonic_midi).rem_euclid(12))
            .collect();
        assert_eq!(pcs, [4, 8, 11, 1]);
        let a = Identity::capture_recapture();
        assert_eq!(a.key.degree(a.motif.degrees[0], a.tonic_midi), a.tonic_midi - 5);
    }

    #[test]
    fn an_unplayable_identity_is_refused() {
        let mut id = Identity::hoeffding();
        assert!(id.validate().is_ok());
        id.progression.clear();
        assert!(id.validate().is_err());
        let mut id = Identity::hoeffding();
        id.progression.push(Chord::new(2, &[0, 7]));
        assert!(id.validate().is_err(), "a bare fifth is too thin for the bed");
        let mut id = Identity::hoeffding();
        id.motif = Motif::new(&[1], &[1.0]);
        assert!(id.validate().is_err());
        let mut id = Identity::hoeffding();
        id.chord_s = (12.0, 6.0);
        assert!(id.validate().is_err());
    }

    /// A synthetic narration: noise bursts as "sentences" in a 40 s chapter
    /// and a 30 s one, written as real WAVs and timelines.
    fn synthetic_sections(dir: &std::path::Path) -> Vec<Section> {
        std::fs::create_dir_all(dir).unwrap();
        let mut sections = Vec::new();
        for (c, spans) in [
            ("01_a", vec![(0.6_f64, 6.0), (6.6, 12.0), (16.0, 24.0), (24.6, 33.0)]),
            ("02_b", vec![(0.6_f64, 8.0), (8.6, 15.0), (19.0, 26.0)]),
        ] {
            let secs = if c == "01_a" { 40.0 } else { 30.0 };
            let mut lines = Vec::new();
            for (i, &(a, b)) in spans.iter().enumerate() {
                let mut rng = Rng::new(i as u64 + 1);
                let len = ((b - a) * FS) as usize;
                // speech-like: noise with a 4 Hz syllable envelope (48 kHz,
                // the rate `wav::write` declares)
                let m: Vec<f64> = (0..len)
                    .map(|k| {
                        let t = k as f64 / FS;
                        0.1 * (0.6 + 0.4 * (std::f64::consts::TAU * 4.0 * t).sin()) * rng.normal()
                    })
                    .collect();
                let p = dir.join(format!("{c}_{i}.wav"));
                crate::wav::write(&p, &Stereo::from_mono(&m), crate::wav::Format::F32).unwrap();
                lines.push(crate::timeline::LinePlacement {
                    id: format!("{c}{i}"),
                    start_frame: (a * 60.0).round() as u64,
                    gain_db: 0.0,
                    wav: p,
                });
            }
            sections.push(Section {
                timeline: ChapterTimeline {
                    chapter: c.into(),
                    fps: 60,
                    frames: (secs * 60.0) as u64,
                    lines,
                    ..Default::default()
                },
                plan: ChapterPlan {
                    intensity: 0.5,
                    resolve: c == "02_b",
                },
            });
        }
        sections
    }

    /// Third-octave levels (dB) of each whole second of `x` in [a, b) s.
    fn band_levels_per_second(x: &Stereo, a: f64, b: f64) -> Vec<[f64; 18]> {
        (a as usize..b as usize)
            .map(|s| {
                let m: Vec<f64> = (s * 48_000..(s + 1) * 48_000).map(|i| x.l[i] + x.r[i]).collect();
                crate::fft::third_octave_levels(&m, FS)
            })
            .collect()
    }

    #[test]
    fn editing_one_chapter_leaves_the_others_alone() {
        let dir = std::env::temp_dir().join(format!("explainer_score_indep_{}", std::process::id()));
        let mut sections = synthetic_sections(&dir);
        let id = Identity::hoeffding();
        let before = render(&id, &sections, None).unwrap();
        // the other direction first: nothing of chapter 2 starts before its
        // cut, so editing it leaves chapter 1 (0–40 s) unchanged but for the
        // zero-phase pocket's few-millisecond pre-ring of the cut, at the level
        // of a swell's first samples
        let mut later = sections.clone();
        later[1].plan.intensity = 0.95;
        let edited = render(&id, &later, None).unwrap();
        let (x, y) = (before.chapter_music(0), edited.chapter_music(0));
        let diff =
            x.l.iter()
                .zip(&y.l)
                .chain(x.r.iter().zip(&y.r))
                .map(|(a, b)| (a - b).abs())
                .fold(0.0, f64::max);
        assert!(
            diff <= 1e-6 * x.peak(),
            "chapter 1 moved by {diff:e} (peak {:e})",
            x.peak()
        );
        sections[0].plan.intensity = 0.95;
        let after = render(&id, &sections, None).unwrap();
        assert_eq!(before.cap_db, 0.0);
        assert_eq!(after.cap_db, 0.0);
        // chapter 0 did change...
        assert_ne!(before.chapter_music(0), after.chapter_music(0));
        // ...chapter 1 (40–70 s) did not, past the few seconds its neighbour's
        // tails ring into it: within ±0.5 dB per third octave per second
        let (fa, fb) = (
            band_levels_per_second(&before.music, 45.0, 70.0),
            band_levels_per_second(&after.music, 45.0, 70.0),
        );
        for (sec, (x, y)) in fa.iter().zip(&fb).enumerate() {
            for (band, (p, q)) in x.iter().zip(y).enumerate() {
                if *p > -100.0 {
                    assert!(
                        (p - q).abs() <= 0.5,
                        "second {} band {band}: {p:.2} vs {q:.2} dB",
                        45 + sec
                    );
                }
            }
        }
        // and its notes are the very same notes
        let notes = |s: &Score| {
            s.events
                .iter()
                .filter(|e| e.start >= 40.0)
                .map(|e| (e.seed, e.pitch.to_bits(), e.start.to_bits()))
                .collect::<Vec<_>>()
        };
        assert_eq!(notes(&before), notes(&after));
    }

    #[test]
    fn marks_become_reveal_cues_outside_speech() {
        let dir = std::env::temp_dir().join(format!("explainer_score_marks_{}", std::process::id()));
        let mut sections = synthetic_sections(&dir);
        let mark = |label: &str, secs: f64| crate::timeline::Mark {
            label: label.into(),
            frame: (secs * 60.0) as u64,
        };
        // chapter 1 (speech 0.6–6, 6.6–12, 16–24, 24.6–33): a mark in the
        // 12–16 s gap; one 1 s before a sentence that a long gap follows
        // ends; one deep inside a long sentence; one 0.1 s before a
        // sentence's onset, which belongs to that sentence (struck there, it
        // masked Hoeffding's h5 onset); and one before a sentence whose end
        // leaves only a 0.6 s pause (a cue there rings into the next one)
        sections[0].timeline.marks = vec![
            mark("gap", 14.0),
            mark("late", 11.0),
            mark("deep", 18.0),
            mark("onset", 15.9),
            mark("short_pause", 23.0),
        ];
        let s = render(&Identity::hoeffding(), &sections, None).unwrap();
        let labels: Vec<&str> = s.cues.iter().map(|c| c.1.as_str()).collect();
        assert_eq!(labels, vec!["gap", "late"]);
        assert_eq!(
            s.skipped_marks,
            ["01_a/deep", "01_a/onset", "01_a/short_pause"]
                .map(String::from)
                .to_vec()
        );
        assert!((s.cues[0].0 - 14.0).abs() < 1e-9, "on the mark");
        assert!(
            s.cues[1].0 > 11.9 && s.cues[1].0 < 12.6,
            "at the sentence's end: {}",
            s.cues[1].0
        );
        let cue_notes = s.events.iter().filter(|e| e.track == Track::Cue).count();
        assert_eq!(cue_notes, 4, "a dyad per cue");
    }

    #[test]
    fn each_cue_kind_lands_where_it_should() {
        let dir = std::env::temp_dir().join(format!("explainer_score_kinds_{}", std::process::id()));
        let mut sections = synthetic_sections(&dir);
        let mark = |label: &str, secs: f64| crate::timeline::Mark {
            label: label.into(),
            frame: (secs * 60.0) as u64,
        };
        // chapter 1 has gaps at 12–16 s and 33–40 s; 3.0 s is inside a
        // sentence; foreground cues are kept 1.5 s apart (rule 5)
        sections[0].timeline.marks = vec![
            mark("pizz:d", 12.5),
            mark("whoosh:b", 14.0),
            mark("swell:a", 15.5),
            mark("tick:c", 3.0),
            mark("resolve:e", 34.0),
            mark("too_close", 14.6),
        ];
        let s = render(&Identity::hoeffding(), &sections, None).unwrap();
        assert_eq!(
            s.skipped_marks,
            vec!["01_a/too_close".to_string()],
            "rule 5: 0.6 s after the whoosh"
        );
        let swell = s.sfx.iter().find(|f| matches!(f.kind, SfxKind::Swell { .. })).unwrap();
        let SfxKind::Swell { seconds } = swell.kind else {
            unreachable!()
        };
        assert!(
            (swell.start + seconds - 15.5).abs() < 1e-9,
            "the swell ends on its mark"
        );
        let whoosh = s.sfx.iter().find(|f| matches!(f.kind, SfxKind::Whoosh { .. })).unwrap();
        assert!((whoosh.start - 13.6).abs() < 1e-9, "centred on 14 s");
        let tick = s.sfx.iter().find(|f| matches!(f.kind, SfxKind::Tick { .. })).unwrap();
        assert!(
            (tick.start - 3.0).abs() < 1e-9,
            "a tick stays on its mark, even in speech"
        );
        let pizz: Vec<&Event> = s.events.iter().filter(|e| e.inst == Instrument::Pluck).collect();
        assert_eq!(pizz.len(), 1);
        assert!((pizz[0].start - 12.5).abs() < 1e-9);
        let resolve: Vec<&Event> = s
            .events
            .iter()
            .filter(|e| e.track == Track::Cue && e.inst == Instrument::FeltPiano)
            .collect();
        assert_eq!(resolve.len(), 2, "the resolution 2̂–1̂");
        assert!((resolve[0].start - 34.0).abs() < 1e-9);
        assert!(s.music.is_finite());
    }

    #[test]
    fn cue_density_follows_rule_5() {
        let mut placed = Vec::new();
        for (t, k) in [(10.0, CueKind::Reveal), (11.6, CueKind::Pizz), (13.2, CueKind::Whoosh)] {
            assert!(cue_allowed(k, t, &placed));
            placed.push((t, k));
        }
        assert!(!cue_allowed(CueKind::Reveal, 12.0, &placed), "0.4 s after a cue");
        // at most six foreground cues in any 30 s
        for t in [14.8, 16.4, 18.0] {
            assert!(cue_allowed(CueKind::Reveal, t, &placed));
            placed.push((t, CueKind::Reveal));
        }
        assert!(!cue_allowed(CueKind::Reveal, 19.6, &placed), "a seventh within 30 s");
        assert!(cue_allowed(CueKind::Reveal, 41.0, &placed), "30 s on, room again");
        // ticks are counters: they ignore the foreground gap, but a stream
        // faster than 6 per second is thinned
        assert!(cue_allowed(CueKind::Tick, 10.1, &placed));
        placed.push((10.1, CueKind::Tick));
        assert!(!cue_allowed(CueKind::Tick, 10.2, &placed));
        assert!(cue_allowed(CueKind::Tick, 10.3, &placed));
    }

    #[test]
    fn the_bed_thins_before_a_reveal() {
        let n = (10.0 * FS) as usize;
        let g = thin_outs(&[5.0], n);
        let at = |t: f64| g[(t * FS) as usize];
        assert_eq!(at(3.9), 1.0);
        assert!(
            (crate::units::gain_to_db(at(5.0)) + 4.0).abs() < 1e-6,
            "4 dB down at the reveal"
        );
        assert!(at(4.5) < 1.0 && at(4.5) > at(5.0), "a smooth dip");
        assert!((at(6.6) - 1.0).abs() < 1e-12, "recovered 1.5 s later");
        let steps = g.windows(2).map(|w| (w[1] - w[0]).abs()).fold(0.0, f64::max);
        assert!(steps < 1e-4, "{steps}");
    }

    #[test]
    fn intensity_adds_a_doubled_top_voice() {
        let id = Identity::hoeffding();
        let chord = id.progression[0].clone();
        let span = ChordSpan {
            start: 0.0,
            end: 8.0,
            voicing: voice_lead(&chord, 4, None, &id.register),
            chord,
        };
        let bed = |i: f64| {
            chord_events(&id, &span, i, "x", 0)
                .iter()
                .filter(|e| e.track == Track::Bed)
                .count()
        };
        assert_eq!(bed(0.5), 4);
        assert_eq!(bed(0.7), 5);
        assert!(reverb_send(0.9) > reverb_send(0.3));
    }

    #[test]
    fn a_reveal_never_sounds_a_minor_ninth_even_on_a_ninth_chords_own_tones() {
        // Bm9 (B D F♯ A C♯) voiced C♯4 F♯4 A4 D5: the dyad took the top two,
        // A5 and D5, and D5 sits a semitone over the chord's own C♯ (found on
        // Hoeffding's v6: four chapters failed rule 7)
        let key = Key::new(2, IONIAN);
        let bm9 = Chord::min9(11);
        let v = Voicing {
            bass: 35,
            upper: vec![61, 66, 69, 74],
        };
        assert!(clashes(74, &bm9), "the old dyad's D5");
        assert_eq!(reveal_dyad(&v, &bm9, &key, 60), vec![81, 76], "A5 and E5");
        // every chord of the synthetic progression, every inversion's top two
        for chord in Identity::hoeffding().progression {
            for top in 60..84 {
                let v = Voicing {
                    bass: 40,
                    upper: vec![top - 7, top],
                };
                for p in reveal_dyad(&v, &chord, &key, 60) {
                    assert!(!clashes(p, &chord) && p <= 88, "{p} over {chord:?}");
                }
            }
        }
    }

    #[test]
    fn reveals_at_every_chord_change_keep_rule_7() {
        let dir = std::env::temp_dir().join(format!("explainer_score_reveal_{}", std::process::id()));
        let mut sections = synthetic_sections(&dir);
        let id = Identity::hoeffding();
        let plain = render(&id, &sections, None).unwrap();
        // a reveal 0.1 s before every chord change in 01_a: the ones in a
        // real gap sound, the rest are skipped, and none sounds a minor ninth
        sections[0].timeline.marks = plain
            .chords
            .iter()
            .filter(|c| c.start > 1.0 && c.start < 39.0)
            .enumerate()
            .map(|(k, c)| crate::timeline::Mark {
                label: format!("reveal:{k}"),
                frame: ((c.start - 0.1) * 60.0).floor() as u64,
            })
            .collect();
        let s = render(&id, &sections, None).unwrap();
        let starts = |x: &Score| x.chords.iter().map(|c| c.start).collect::<Vec<_>>();
        assert_eq!(starts(&s), starts(&plain), "marks do not move the harmony");
        let cue: Vec<&Event> = s.events.iter().filter(|e| e.track == Track::Cue).collect();
        assert!(!cue.is_empty(), "some reveals sound: {:?}", s.skipped_marks);
        assert!(cue.iter().all(|e| e.hold >= MIN_NOTE_S && e.pitch <= 88.0));
        let r = crate::qa::evaluate(
            "01_a",
            &s.chapter_voice(0),
            &s.chapter_music(0),
            40 * 48_000,
            Some(&crate::qa::ScoreFacts::chapter(&s, 0)),
        );
        let offenders = |x: &Score| {
            x.events
                .iter()
                .filter(|e| matches!(e.track, Track::Melody | Track::Answer | Track::Cue | Track::Roll))
                .filter_map(|e| {
                    let over: Vec<String> = x
                        .chords
                        .iter()
                        .filter(|c| {
                            c.start < e.start + e.hold && c.end > e.start && clashes(e.pitch.round() as i32, &c.chord)
                        })
                        .map(|c| format!("{:?}@{:.3}-{:.3}", c.chord, c.start, c.end))
                        .collect();
                    (!over.is_empty()).then(|| {
                        format!(
                            "{:?} {} at {:.3}+{:.3} over {over:?}",
                            e.track, e.pitch, e.start, e.hold
                        )
                    })
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(
            r.get("rule7_minor_ninths").unwrap().pass,
            Some(true),
            "with the mark: {:?}; without: {:?}",
            offenders(&s),
            offenders(&plain)
        );
    }

    #[test]
    fn a_listen_window_plays_its_sonification_at_the_listen_level_and_stops_at_its_end() {
        let dir = std::env::temp_dir().join(format!("explainer_score_listen_{}", std::process::id()));
        let mut sections = synthetic_sections(&dir);
        // 01_a is silent from 12 s to 16 s: a 3 s window from 12.4 s, and a
        // 5 s cloud of grains, longer than the window, so it must be cut
        let grains: Vec<crate::sonify::Grain> = (0..400)
            .map(|k| crate::sonify::Grain {
                start: k as f64 * 0.0125,
                pitch: crate::sonify::snap(74.0 + (k % 9) as f64, &[2, 4, 6, 9, 11]),
                pan: if k % 2 == 0 { -0.3 } else { 0.3 },
                amp: 0.3,
            })
            .collect();
        let son = dir.join("cloud.wav");
        let cloud = crate::sonify::cloud(&grains, 5.0);
        crate::wav::write(&son, &cloud, crate::wav::Format::F32).unwrap();
        // the same cloud with its polarity flipped: a render with it differs
        // from one with the cloud by exactly twice the cloud's part (the
        // gap's other music is the same in both: a sonified gap gets no
        // phrase of its own)
        let flipped = dir.join("cloud_flipped.wav");
        let mut neg = cloud.clone();
        neg.scale(-1.0);
        crate::wav::write(&flipped, &neg, crate::wav::Format::F32).unwrap();
        let window = |sound: Option<std::path::PathBuf>| crate::timeline::Window {
            frame: (12.4 * 60.0) as u64,
            seconds: 3.0,
            label: "cloud".into(),
            sound,
        };
        let id = Identity::hoeffding();
        sections[0].timeline.listens = vec![window(None)];
        let quiet = render(&id, &sections, None).unwrap();
        let f_quiet = fingerprint(&id, &sections, None).unwrap();
        sections[0].timeline.listens = vec![window(Some(flipped))];
        let opposite = render(&id, &sections, None).unwrap();
        sections[0].timeline.listens = vec![window(Some(son.clone()))];
        let heard = render(&id, &sections, None).unwrap();
        assert_eq!(
            (opposite.cap_db, heard.cap_db),
            (0.0, 0.0),
            "no true-peak cap to confound the difference"
        );
        let diff = Stereo {
            l: heard
                .music
                .l
                .iter()
                .zip(&opposite.music.l)
                .map(|(a, b)| (a - b) / 2.0)
                .collect(),
            r: heard
                .music
                .r
                .iter()
                .zip(&opposite.music.r)
                .map(|(a, b)| (a - b) / 2.0)
                .collect(),
        };
        // the soundless window's gap plays the motif; the sonified one does not
        let in_gap = |x: &Score| {
            x.events
                .iter()
                .filter(|e| e.track == Track::Melody && e.start > 12.0 && e.start < 16.0)
                .count()
        };
        assert!(in_gap(&quiet) > 0 && in_gap(&heard) == 0);
        let (start, end) = ((12.4 * FS) as usize, (15.4 * FS) as usize);
        let peak = diff.peak();
        assert!(peak > 0.0, "the sonification is in the stem");
        // nothing before the window (the pocket's zero-phase filter may
        // pre-ring at the 1e-6 level), nothing after it but the tape's 2 ms
        let before = diff.l[..start]
            .iter()
            .chain(&diff.r[..start])
            .fold(0.0_f64, |m, x| m.max(x.abs()));
        assert!(before <= 1e-6 * peak, "{before} before the window");
        let tail = end + (0.01 * FS) as usize;
        let after = diff.l[tail..]
            .iter()
            .chain(&diff.r[tail..])
            .fold(0.0_f64, |m, x| m.max(x.abs()));
        assert!(after <= 1e-6 * peak, "{after} after the window");
        // at the listen level, against the voice, inside the window
        let inside = Stereo {
            l: diff.l[start..end].to_vec(),
            r: diff.r[start..end].to_vec(),
        };
        let lufs = meter::integrated(&inside);
        let want = heard.voice_lufs + id.listen_lu;
        assert!((lufs - want).abs() < 1.5, "{lufs:.2} LUFS, want {want:.2}");
        // the sonification's bytes are part of the fingerprint
        let f_heard = fingerprint(&id, &sections, None).unwrap();
        assert_ne!(f_quiet, f_heard);
        let mut bytes = std::fs::read(&son).unwrap();
        let k = bytes.len() - 1;
        bytes[k] ^= 1;
        std::fs::write(&son, &bytes).unwrap();
        assert_ne!(
            f_heard,
            fingerprint(&id, &sections, None).unwrap(),
            "sonification bytes"
        );
        // a sonification with no sound in it is refused, not silently skipped
        crate::wav::write(&son, &Stereo::silence(48_000), crate::wav::Format::F32).unwrap();
        assert!(
            render(&id, &sections, None)
                .unwrap_err()
                .to_string()
                .contains("silent sonification")
        );
    }

    #[test]
    fn the_fingerprint_changes_with_any_input_and_only_then() {
        let dir = std::env::temp_dir().join(format!("explainer_score_fp_{}", std::process::id()));
        let mut sections = synthetic_sections(&dir);
        let id = Identity::hoeffding();
        let f0 = fingerprint(&id, &sections, None).unwrap();
        assert_eq!(f0, fingerprint(&id, &sections, None).unwrap());
        assert_ne!(f0, fingerprint(&id, &sections, Some(0.0)).unwrap(), "voice gain");
        let mut other = id.clone();
        other.bed_lu -= 1.0;
        assert_ne!(f0, fingerprint(&other, &sections, None).unwrap(), "identity");
        sections[1].plan.intensity += 0.1;
        let f1 = fingerprint(&id, &sections, None).unwrap();
        assert_ne!(f0, f1, "plan");
        // one byte of one narration WAV
        let wav = sections[0].timeline.lines[0].wav.clone();
        let mut bytes = std::fs::read(&wav).unwrap();
        let k = bytes.len() - 1;
        bytes[k] ^= 1;
        std::fs::write(&wav, &bytes).unwrap();
        assert_ne!(f1, fingerprint(&id, &sections, None).unwrap(), "narration bytes");
    }

    #[test]
    fn a_rendered_score_follows_the_rules_it_can_check_itself() {
        let dir = std::env::temp_dir().join(format!("explainer_score_render_{}", std::process::id()));
        let sections = synthetic_sections(&dir);
        let id = Identity::hoeffding();
        let s = render(&id, &sections, None).unwrap();
        // exact lengths: the stem is the scenes, frame for frame
        assert_eq!(s.music.len(), (40 + 30) * 48_000);
        assert_eq!(s.chapter_music(0).len(), 40 * 48_000);
        assert_eq!(s.chapter_music(1).len(), 30 * 48_000);
        let (a, b) = (s.chapter_music(0), s.chapter_music(1));
        assert_eq!(
            [a.l, b.l].concat(),
            s.music.l,
            "slices concatenate to the render exactly"
        );
        assert!(s.music.is_finite());
        assert!(s.true_peak_db <= -12.0 + 1e-9, "{}", s.true_peak_db);
        // harmony: chords last 6–12 s where pauses allow, never past the 16 s
        // QA limit here, and the video ends on the tonic
        assert!(s.chords.len() >= 5, "{}", s.chords.len());
        for c in &s.chords {
            let d = c.end - c.start;
            assert!(
                (6.0 - 1e-9..=16.0).contains(&d),
                "{:.2}–{:.2} s: {d:.2} s",
                c.start,
                c.end
            );
        }
        assert_eq!(s.chords.last().unwrap().chord, id.progression[0]);
        // no melodic onset inside speech (rule 8), no bed note in 95–250 Hz (rule 1)
        for e in &s.events {
            if matches!(e.track, Track::Melody | Track::Answer) {
                assert!(
                    !s.speech.iter().any(|&(a, b)| e.start >= a && e.start < b),
                    "{e:?} starts in speech"
                );
            }
            if e.track == Track::Bed {
                assert!(!s.register.in_band(hz(e.pitch)), "{e:?}");
            }
        }
        assert!(!s.phrases.is_empty(), "the 4 s gaps get phrases");
        // the duck: music under speech is well below music in a gap
        let st = |x: &Stereo, a: f64, b: f64| {
            let seg = Stereo {
                l: x.l[(a * FS) as usize..(b * FS) as usize].to_vec(),
                r: x.r[(a * FS) as usize..(b * FS) as usize].to_vec(),
            };
            meter::integrated(&seg)
        };
        let under = st(&s.music, 2.0, 5.0);
        let voice = st(&s.voice, 2.0, 5.0);
        assert!(voice - under >= 16.0, "voice {voice:.1}, music under it {under:.1}");
        // deterministic: the same inputs give the same stem
        let again = render(&id, &sections, None).unwrap();
        assert_eq!(again.music, s.music);
    }
    #[test]
    fn listen_spans_hold_the_listen_chord_and_the_progression_returns_in_a_pause() {
        // candidates every 0.5 s except 20–30 s (one long sentence)
        let cands: Vec<f64> = (0..120)
            .map(|k| k as f64 * 0.5)
            .filter(|t| !(20.0..30.0).contains(t))
            .collect();
        let mut rng = Rng::new(5);
        // a window from 12 s to 15 s; the voice comes back at once, so the
        // listen chord holds to 15 s (the next candidate); one at 50–53 s
        let ch = plan_with_listens(&cands, 60.0, (6.0, 10.0), &[(12.0, 15.0), (50.0, 53.0)], &mut rng);
        let listen: Vec<f64> = ch.iter().filter(|c| c.1).map(|c| c.0).collect();
        assert_eq!(listen, vec![12.0, 50.0], "{ch:?}");
        for w in ch.windows(2) {
            assert!(w[1].0 > w[0].0, "in order: {ch:?}");
            if w[0].1 {
                assert!(w[1].0 >= w[0].0 + 3.0, "nothing inside a window: {ch:?}");
            }
        }
        // every free change is a candidate, and none is inside 12–15 or 50–53
        for &(t, l) in &ch {
            if !l {
                assert!(cands.iter().any(|&c| (c - t).abs() < 1e-9), "{t}");
                assert!(!(12.0..15.0).contains(&t) && !(50.0..53.0).contains(&t), "{t}");
            }
        }
        assert_eq!(ch[0], (0.0, false));
        // two windows 1 s apart: the listen chord holds across the stretch
        let ch = plan_with_listens(&cands, 40.0, (6.0, 10.0), &[(10.0, 13.0), (14.0, 17.0)], &mut rng);
        assert_eq!(ch.iter().filter(|c| c.1).count(), 1, "{ch:?}");
        assert!(!ch.iter().any(|c| c.0 > 10.0 && c.0 < 17.0), "{ch:?}");
        // a window at the very start of a chapter
        let ch = plan_with_listens(&cands, 40.0, (6.0, 10.0), &[(0.0, 3.0)], &mut rng);
        assert_eq!(ch[0], (0.0, true), "{ch:?}");
        assert!(ch[1].0 >= 3.0 && !ch[1].1, "{ch:?}");
        // no windows: exactly plan_changes
        let a = plan_with_listens(&cands, 40.0, (6.0, 10.0), &[], &mut Rng::new(9));
        let b = plan_changes(&cands, 40.0, (6.0, 10.0), &mut Rng::new(9));
        assert_eq!(a.iter().map(|c| c.0).collect::<Vec<_>>(), b);
    }

    #[test]
    fn every_chord_of_every_identity_voices_without_a_minor_ninth() {
        for id in [
            Identity::hoeffding(),
            Identity::capture_recapture(),
            Identity::raptorq_gf256(),
            Identity::bakery_algorithm(),
        ] {
            let mut progressions = vec![id.progression.clone()];
            progressions.extend(
                id.moods
                    .iter()
                    .map(|m| id.progression_for(&format!("{}_x", m.chapters[0]))),
            );
            progressions.extend(id.listen_chord.clone().map(|c| vec![c]));
            for prog in progressions {
                let mut prev: Option<Voicing> = None;
                for _ in 0..3 {
                    for c in &prog {
                        let v = voice_lead(c, 4, prev.as_ref(), &id.register);
                        let all: Vec<i32> = std::iter::once(v.bass).chain(v.upper.iter().copied()).collect();
                        for (i, &lo) in all.iter().enumerate() {
                            for &hi in &all[i + 1..] {
                                assert!(!minor_ninth(lo, hi), "{}: {c:?} voiced {v:?}", id.name);
                            }
                        }
                        let roll = roll_voicing(&ChordSpan {
                            start: 0.0,
                            end: 8.0,
                            chord: c.clone(),
                            voicing: v.clone(),
                        });
                        assert!(roll.len() >= 4, "{}: {c:?} rolls {roll:?}", id.name);
                        for (i, &lo) in roll.iter().enumerate() {
                            for &hi in &roll[i + 1..] {
                                assert!(!minor_ninth(lo, hi), "{}: {c:?} rolls {roll:?}", id.name);
                            }
                        }
                        prev = Some(v);
                    }
                }
            }
        }
    }

    #[test]
    fn an_authored_silence_is_exact_zero_faded_in_the_gap_before_it_and_the_music_returns() {
        let dir = std::env::temp_dir().join(format!("explainer_score_silence_{}", std::process::id()));
        let mut sections = synthetic_sections(&dir);
        let mark = |label: &str, secs: f64| crate::timeline::Mark {
            label: label.into(),
            frame: (secs * 60.0) as u64,
        };
        // 01_a speaks 0.6–6, 6.6–12, 16–24, 24.6–33: silence for the sentence
        // at 16 s (after a 4 s gap; A's 15_1), the music back after 24.3 s;
        // and a key line at 6.6 s, after a 0.6 s pause, with no end mark
        sections[0].timeline.marks = vec![
            mark("silence:numbers", 15.9),
            mark("silence_end:numbers", 24.3),
            mark("silence:key", 6.55),
        ];
        // a reveal asked for inside the silence is not played
        sections[0].timeline.marks.push(mark("reveal_inside", 20.0));
        for id in [Identity::capture_recapture(), Identity::hoeffding()] {
            let s = render(&id, &sections, None).unwrap();
            let sp = |name: &str| s.silences.iter().find(|x| x.name == name).unwrap().clone();
            let (numbers, key) = (sp("numbers"), sp("key"));
            // 15_1: a 1.2 s fade ending 0.5 s before the sentence, after 12 s
            assert!((numbers.fade_end - 15.5).abs() < 0.05, "{numbers:?}");
            assert!((numbers.fade_end - numbers.fade_start - SILENCE_FADE).abs() < 1e-9);
            assert!(numbers.fade_start > 12.0, "never mid-sentence: {numbers:?}");
            assert!(numbers.back >= 24.0 && numbers.back < 24.6, "{numbers:?}");
            // the key line drops out in its 0.6 s pause, from the previous
            // sentence's end, and stays out to the end of the region (12 s)
            assert!(key.fade_start >= 5.99 && key.fade_end <= 6.6 - 0.09, "{key:?}");
            assert!(key.back >= 12.0 && key.back < 12.2, "{key:?}");
            assert!(s.notes.iter().any(|n| n.contains("silence:key")), "{:?}", s.notes);
            // exactly zero inside both, sound on either side
            let zero = |a: f64, b: f64| {
                let (i, j) = (((a * FS).ceil()) as usize, (b * FS) as usize);
                s.music.l[i..j].iter().chain(&s.music.r[i..j]).all(|&x| x == 0.0)
            };
            assert!(
                zero(numbers.fade_end, numbers.back) && zero(key.fade_end, key.back),
                "{}",
                id.name
            );
            let loud = |a: f64, b: f64| {
                let seg = Stereo {
                    l: s.music.l[(a * FS) as usize..(b * FS) as usize].to_vec(),
                    r: s.music.r[(a * FS) as usize..(b * FS) as usize].to_vec(),
                };
                meter::integrated(&seg).is_finite()
            };
            assert!(loud(2.0, 5.5) && loud(13.0, 14.0) && loud(26.5, 32.0), "{}", id.name);
            // no note of the score's own starts inside, and the reveal is skipped
            assert!(
                s.events
                    .iter()
                    .filter(|e| !matches!(e.track, Track::Bed | Track::Bass))
                    .all(|e| !(e.start >= numbers.fade_start && e.start < numbers.back)),
                "{}",
                id.name
            );
            assert!(
                s.skipped_marks.contains(&"01_a/reveal_inside".to_string()),
                "{:?}",
                s.skipped_marks
            );
            // QA: the chapter's silences are exactly zero, and a planted sample fails
            let facts = crate::qa::ScoreFacts::chapter(&s, 0);
            assert_eq!(facts.silences.len(), 2);
            let mut music = s.chapter_music(0);
            let r = crate::qa::evaluate("01_a", &s.chapter_voice(0), &music, music.len(), Some(&facts));
            assert_eq!(r.get("authored_silence_peak").unwrap().pass, Some(true));
            music.l[(20.0 * FS) as usize] = 1e-6;
            let r = crate::qa::evaluate("01_a", &s.chapter_voice(0), &music, music.len(), Some(&facts));
            assert_eq!(r.get("authored_silence_peak").unwrap().pass, Some(false));
        }
    }

    #[test]
    fn the_pad_carries_the_bed_and_the_bass_supports_it() {
        // at unity the bass measured 4–6 LU over the pad in every identity
        // (K-weighted); the new identities keep it at least 3 LU under
        for id in [
            Identity::capture_recapture(),
            Identity::raptorq_gf256(),
            Identity::bakery_algorithm(),
        ] {
            let n = (10.0 * FS) as usize;
            let chord = id.progression[0].clone();
            let span = ChordSpan {
                start: 0.0,
                end: 9.0,
                voicing: voice_lead(&chord, 4, None, &id.register),
                chord,
            };
            let ev = chord_events(&id, &span, 0.5, "", 0);
            let level = |track: Track| {
                let only: Vec<Event> = ev.iter().filter(|e| e.track == track).cloned().collect();
                let b = bed_bus(&id, &only, n, &vec![reverb_send(0.5); n], &vec![1.0; n]);
                let (a, c) = ((3.0 * FS) as usize, (8.0 * FS) as usize);
                meter::integrated(&Stereo {
                    l: b.l[a..c].to_vec(),
                    r: b.r[a..c].to_vec(),
                })
            };
            let (pad, bass) = (level(Track::Bed), level(Track::Bass));
            assert!(pad - bass >= 3.0, "{}: pad {pad:.1}, bass {bass:.1} LUFS", id.name);
            assert!(
                pad - bass <= 6.0,
                "{}: the bass still supports it ({pad:.1}, {bass:.1})",
                id.name
            );
        }
    }

    #[test]
    fn an_authored_reveal_owns_its_gap_no_roll_and_the_motif_after_it() {
        // found on C's real narration: a lane's reveal dyad inside the
        // gap's roll made seven onsets in a second (QA allows six)
        let dir = std::env::temp_dir().join(format!("explainer_score_owned_{}", std::process::id()));
        let mut sections = synthetic_sections(&dir);
        sections[0].timeline.marks = vec![crate::timeline::Mark {
            label: "reveal_answer".into(),
            frame: (33.4 * 60.0) as u64,
        }];
        let id = Identity::bakery_algorithm();
        let s = render(&id, &sections, None).unwrap();
        let in_gap = |t: Track| {
            s.events
                .iter()
                .filter(|e| e.track == t && e.start > 33.0 && e.start < 40.0)
                .count()
        };
        assert_eq!(in_gap(Track::Cue), 2, "the reveal's dyad");
        assert_eq!(in_gap(Track::Roll), 0, "no roll in the reveal's gap");
        let first_melody = s
            .events
            .iter()
            .filter(|e| e.track == Track::Melody && e.start > 33.0 && e.start < 40.0)
            .map(|e| e.start)
            .fold(f64::INFINITY, f64::min);
        assert!(first_melody >= 33.4 + AFTER_CUE - 1e-9, "{first_melody}");
        let r = crate::qa::evaluate(
            "01_a",
            &s.chapter_voice(0),
            &s.chapter_music(0),
            s.chapter_music(0).len(),
            Some(&crate::qa::ScoreFacts::chapter(&s, 0)),
        );
        assert_eq!(r.get("onsets_per_s_gaps").unwrap().pass, Some(true));
        // the other chapter's closing gap still rolls
        assert!(s.events.iter().any(|e| e.track == Track::Roll && e.start > 40.0));
    }

    #[test]
    fn a_thin_mark_dips_the_bed_and_places_no_cue_of_its_own() {
        let dir = std::env::temp_dir().join(format!("explainer_score_thin_{}", std::process::id()));
        let mut sections = synthetic_sections(&dir);
        let plain = render(&Identity::capture_recapture(), &sections, None).unwrap();
        sections[0].timeline.marks = vec![crate::timeline::Mark {
            label: "thin:08_4".into(),
            frame: (20.0 * 60.0) as u64,
        }];
        let thin = render(&Identity::capture_recapture(), &sections, None).unwrap();
        assert_eq!(plain.events.len(), thin.events.len(), "no notes of its own");
        let level = |x: &Score, a: f64, b: f64| {
            meter::integrated(&Stereo {
                l: x.music.l[(a * FS) as usize..(b * FS) as usize].to_vec(),
                r: x.music.r[(a * FS) as usize..(b * FS) as usize].to_vec(),
            })
        };
        // mid-sentence, where only the (ducked) bed sounds
        let dip = level(&plain, 19.6, 20.6) - level(&thin, 19.6, 20.6);
        assert!(dip > 1.5, "the bed thins around the mark: {dip:.2} dB");
        assert!((level(&plain, 2.0, 10.0) - level(&thin, 2.0, 10.0)).abs() < 0.01);
    }

    #[test]
    fn a_roll_over_a_bass_a_semitone_under_a_chord_tone_drops_that_tone_and_ends() {
        // found by the test profile's overflow checks: F over an E pedal is a
        // minor ninth at every octave, and the search lifted it forever (in
        // release it wrapped round i32 and left with a wrong pitch class)
        let cases = [
            // Fmaj7 over an E pedal: E2 under F, A, C, E
            (Chord::maj7(5).on(4), vec![65, 69, 72, 76]),
            // the right hand's own E then F (a semitone cluster's tones)
            (Chord::new(4, &[0, 1, 7, 11]), vec![64, 65, 71, 75]),
            // every upper tone in conflict with the bass's pitch class
            (Chord::new(4, &[0, 1, 13]).on(4), vec![65, 77, 89]),
        ];
        for (chord, upper) in cases {
            let span = ChordSpan {
                start: 0.0,
                end: 8.0,
                voicing: Voicing { bass: 40, upper },
                chord: chord.clone(),
            };
            let v = roll_voicing(&span);
            assert!(v.len() >= 2, "{chord:?}: {v:?}");
            assert!(v.windows(2).all(|w| w[1] > w[0]), "rising: {v:?}");
            assert!(v.iter().all(|&m| (36..=ROLL_TOP).contains(&m)), "{v:?}");
            for (i, &lo) in v.iter().enumerate() {
                for &hi in &v[i + 1..] {
                    assert!(!minor_ninth(lo, hi), "{chord:?}: {lo} under {hi} in {v:?}");
                }
            }
            let pcs = chord.pitch_classes();
            assert!(
                v[1..].iter().all(|m| pcs.contains(&m.rem_euclid(12))),
                "{chord:?}: {v:?}"
            );
        }
    }

    #[test]
    fn a_roll_spreads_the_chord_up_from_its_bass() {
        let reg = Register::from_f0(MATT_F0.0, MATT_F0.1);
        let span = |chord: Chord| ChordSpan {
            start: 0.0,
            end: 8.0,
            voicing: voice_lead(&chord, 4, None, &reg),
            chord,
        };
        for chord in [
            Chord::maj9(5),
            Chord::maj9(2).over(6),
            Chord::new(6, &[0, 4, 7, 14]).on(4),
            Chord::min11(4),
        ] {
            let s = span(chord.clone());
            let v = roll_voicing(&s);
            assert_eq!(v.len(), 5, "{chord:?}: {v:?}");
            assert!(v.windows(2).all(|w| w[1] > w[0]), "rising: {v:?}");
            assert_eq!(v[0].rem_euclid(12), chord.bass, "{v:?}");
            assert!((36..48).contains(&v[0]), "the left hand's bass in C2–B2: {v:?}");
            assert!(v[1] - v[0] >= 7, "a fifth or more above it: {v:?}");
            assert!(v[2..].iter().all(|m| (64..=88).contains(m)), "the right hand: {v:?}");
            let pcs = chord.pitch_classes();
            assert!(v[1..].iter().all(|m| pcs.contains(&m.rem_euclid(12))), "{v:?}");
        }
    }

    #[test]
    fn moods_follow_the_acts_and_sonification_scales_are_pentatonic() {
        let a = Identity::capture_recapture();
        assert_eq!(a.progression_for("01_hook")[0], Chord::maj9(5));
        assert_eq!(a.progression_for("08_the_table_balances").len(), 10);
        assert_eq!(a.progression_for("11_how_sure")[0], Chord::min9(2));
        assert_eq!(a.progression_for("02_the_marks_measure_the_net"), a.progression);
        // the lake's bass circles F by step: F G F♯ F E E♭ F E G F♯
        let bass: Vec<i32> = a
            .progression_for("15_counting_the_dead")
            .iter()
            .map(|c| c.bass)
            .collect();
        assert_eq!(bass, vec![5, 7, 6, 5, 4, 3, 5, 4, 7, 6]);
        // B's field sits on an E pedal
        let b = Identity::raptorq_gf256();
        assert!(b.progression_for("10_the_fold").iter().all(|c| c.bass == 4));
        assert!(b.progression_for("13_the_byte_lottery").iter().any(|c| c.bass != 4));
        // the sonification scales: F and E major pentatonic, and for C the
        // motif's own notes B♭ C D E♭ F with G
        assert_eq!(a.listen_scale(), vec![0, 2, 5, 7, 9]);
        assert_eq!(b.listen_scale(), vec![1, 4, 6, 8, 11]);
        let c = Identity::bakery_algorithm();
        assert_eq!(c.listen_scale(), vec![0, 2, 3, 5, 7, 10]);
        let motif: Vec<i32> = c
            .motif
            .degrees
            .iter()
            .chain(&c.resolution.degrees)
            .map(|&d| c.key.degree(d, c.tonic_midi).rem_euclid(12))
            .collect();
        assert!(motif.iter().all(|pc| c.listen_scale().contains(pc)), "{motif:?}");
        // a byte's eight bit-notes (plan §7.5 B: E4 F♯4 G♯4 B4 C♯5 E5 F♯5 G♯5)
        assert!(
            [64, 66, 68, 71, 73, 76, 78, 80]
                .iter()
                .all(|m: &i32| b.listen_scale().contains(&m.rem_euclid(12)))
        );
        // a window's label names its storyboard id alone or with its chapter
        assert_eq!(c.theme_for("13_4"), Some(Theme::Resolved));
        assert_eq!(c.theme_for("13_proudest:13_4"), Some(Theme::Resolved));
        assert_eq!(c.theme_for("01_one_plus_one_is_one:01_3"), Some(Theme::Open));
        assert_eq!(c.theme_for("13_proudest:13_40"), None);
        assert_eq!(c.theme_for("03_take_a_number:03_6"), None);
        // every theme names a window, every mood real chords
        for id in [a, b, c] {
            id.validate().unwrap();
            assert_eq!(id.themes.len(), 2, "{}", id.name);
        }
        // two moods claiming one chapter are refused
        let mut bad = Identity::capture_recapture();
        bad.moods[1].chapters.push("01");
        assert!(bad.validate().is_err());
        // a listen chord with a semitone inside is refused
        let mut bad = Identity::capture_recapture();
        bad.listen_chord = Some(Chord::maj7(5));
        assert!(bad.validate().is_err());
    }

    #[test]
    fn the_new_identities_roll_in_long_gaps_theme_their_windows_and_hold_still_under_sonifications() {
        let dir = std::env::temp_dir().join(format!("explainer_score_ids_{}", std::process::id()));
        let base = synthetic_sections(&dir);
        let son = dir.join("tones.wav");
        let tones: Vec<crate::sonify::Grain> = (0..40)
            .map(|k| crate::sonify::Grain {
                start: k as f64 * 0.07,
                pitch: 76.0,
                pan: 0.0,
                amp: 0.3,
            })
            .collect();
        crate::wav::write(&son, &crate::sonify::cloud(&tones, 3.0), crate::wav::Format::F32).unwrap();
        for (id, title) in [
            (Identity::capture_recapture(), "01_4"),
            (Identity::raptorq_gf256(), "01_3"),
            (Identity::bakery_algorithm(), "01_3"),
        ] {
            let mut sections = base.clone();
            // a soundless title window in 01_a's 12–16 s gap; a sonification
            // in 02_b's 15–19 s gap (55–59 s in the video)
            sections[0].timeline.listens = vec![crate::timeline::Window {
                frame: (12.3 * 60.0) as u64,
                seconds: 3.0,
                label: title.into(),
                sound: None,
            }];
            sections[1].timeline.listens = vec![crate::timeline::Window {
                frame: (15.2 * 60.0) as u64,
                seconds: 3.0,
                label: "02_x".into(),
                sound: Some(son.clone()),
            }];
            let s = render(&id, &sections, None).unwrap();
            let name = &id.name;
            // the title: the motif as it is, over one soft block chord
            let title_notes: Vec<&Event> = s
                .events
                .iter()
                .filter(|e| e.track == Track::Melody && e.start > 12.0 && e.start < 16.0)
                .collect();
            assert_eq!(title_notes.len(), id.motif.degrees.len(), "{name}");
            let block: Vec<&Event> = s
                .events
                .iter()
                .filter(|e| e.track == Track::Roll && e.start > 12.0 && e.start < 16.0)
                .collect();
            assert!(block.len() >= 4, "{name}: {block:?}");
            assert!(block.iter().all(|e| (e.start - block[0].start).abs() < 1e-9));
            // the sonified gap: the listen chord, nothing melodic of its own
            let listen = s.chords.iter().find(|c| c.start <= 55.3 && c.end > 58.0).unwrap();
            assert_eq!(Some(&listen.chord), id.listen_chord.as_ref(), "{name}");
            assert!(
                !s.events
                    .iter()
                    .any(|e| e.track != Track::Bed && e.track != Track::Bass && e.start > 55.0 && e.start < 59.0),
                "{name}"
            );
            // 01_a's closing 7 s gap: a slow roll, then the motif
            let roll: Vec<&Event> = s
                .events
                .iter()
                .filter(|e| e.track == Track::Roll && e.start > 33.0 && e.start < 40.0)
                .collect();
            assert_eq!(roll.len(), 5, "{name}: {roll:?}");
            for w in roll.windows(2) {
                assert!((w[1].start - w[0].start - ROLL_STEP).abs() < 1e-9, "{name}");
            }
            // the pedal is up before the voice returns, and so is every note
            // that sits in the narrator's band
            for e in s.events.iter().filter(|e| e.track == Track::Roll) {
                let next = s
                    .speech
                    .iter()
                    .map(|r| r.0)
                    .find(|&a| a > e.start)
                    .unwrap_or(f64::INFINITY);
                assert!(e.start + e.hold <= next - PEDAL_CLEAR + 1e-9, "{name}: {e:?}");
            }
            // QA's note rules, chapter by chapter
            for k in 0..2 {
                let r = crate::qa::evaluate(
                    &s.chapters[k].0,
                    &s.chapter_voice(k),
                    &s.chapter_music(k),
                    s.chapter_music(k).len(),
                    Some(&crate::qa::ScoreFacts::chapter(&s, k)),
                );
                for m in [
                    "rule1_fundamentals_in_voice_band_in_speech",
                    "rule7_minor_ninths",
                    "rule8_phrases_start_in_speech",
                    "onsets_per_s_speech",
                    "onsets_per_s_gaps",
                    "rough_voice_pairs",
                ] {
                    assert_eq!(r.get(m).unwrap().pass, Some(true), "{name} {k} {m}: {:?}", r.get(m));
                }
            }
            // the video ends on the theme resolved, on its first chord
            assert_eq!(
                &s.chords.last().unwrap().chord,
                &id.progression_for(&s.chapters[1].0)[0]
            );
        }
    }
}
