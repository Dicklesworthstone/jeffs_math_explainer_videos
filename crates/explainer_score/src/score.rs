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
//!    preferring chapter starts; voices are led by the solver (bass E1–E♭2,
//!    upper voices from C4, out of the narrator's band).
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

use crate::duck::{self, DuckShape};
use crate::fx::{Chorus, Fdn, Plate, Tape, Warmth, from_mid_side, mid_side, rcos};
use crate::instr::{self, Instrument, Note};
use crate::meter;
use crate::rng::{Rng, seed_for};
use crate::theory::{Chord, Key, Motif, UPPER_LOW, Voicing, voice_lead};
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
}

/// A video's fixed musical identity; variation lives inside it (plan §7.5).
#[derive(Clone, Debug)]
pub struct Identity {
    /// Seeds every random choice (with chapter, track and event index).
    pub name: String,
    pub key: Key,
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
    /// Melody level relative to the voice while it plays (LU).
    pub melody_lu: f64,
    /// Duck depth under speech (dB).
    pub duck_db: f64,
    /// Sound-effect level relative to the voice (LU), for a reference whoosh.
    pub sfx_lu: f64,
}

impl Identity {
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
        if let Some(c) = self.progression.iter().find(|c| c.pitch_classes().len() < 3) {
            return bad(format!("{c:?} has fewer than three pitch classes"));
        }
        if self.motif.degrees.len() < 2 || self.resolution.degrees.len() < 2 {
            return bad("the motif and the resolution need at least two notes each".into());
        }
        if !(self.beat_s > 0.0 && self.chord_s.0 > 0.0 && self.chord_s.0 <= self.chord_s.1) {
            return bad(format!("beat {} s, chord lengths {:?}", self.beat_s, self.chord_s));
        }
        if ![self.bed_t60, self.bed_lu, self.melody_lu, self.duck_db, self.sfx_lu]
            .iter()
            .all(|v| v.is_finite())
        {
            return bad("non-finite level or reverb time".into());
        }
        Ok(())
    }

    /// Hoeffding's D, v6 (plan §7.5): D major, Dmaj9 → Bm9 → Gmaj9(♯11) →
    /// Asus4, felt piano over a Juno pad with a soft vibraphone answer, 72 BPM
    /// and free, the rising question 1̂–5̂–6̂–3̂ resolving 2̂–1̂ at the end.
    pub fn hoeffding() -> Self {
        Self {
            name: "hoeffding_d".into(),
            key: Key::new(2, crate::theory::IONIAN),
            tonic_midi: 62,
            progression: vec![Chord::maj9(2), Chord::min9(11), Chord::maj9_sharp11(7), Chord::sus4(9)],
            motif: Motif::new(&[1, 5, 6, 3], &[1.0, 1.0, 1.0, 2.0]),
            resolution: Motif::new(&[2, 1], &[1.0, 3.0]),
            beat_s: 60.0 / 72.0,
            palette: Palette {
                bed: Instrument::Pad { brightness: 0.3 },
                melody: Instrument::FeltPiano,
                answer: Instrument::Vibraphone,
            },
            chord_s: (6.0, 12.0),
            bed_t60: 2.5,
            // V - 12 before the 8 dB duck: V - 20 nominal, which measures as
            // a 17-18.5 LU median gap over speech once duck ramps, reverb and
            // bass are counted (plan 7.2's V - 18; at V - 11 the Hoeffding
            // median sat at 16.7-17.5 LU, and short speech regions fell
            // under the 16 LU gate)
            bed_lu: -12.0,
            melody_lu: -12.0,
            duck_db: 8.0,
            sfx_lu: -16.0,
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
}

impl CueKind {
    pub fn from_label(label: &str) -> Self {
        match label.split_once(':').map(|(k, _)| k) {
            Some("swell") => Self::Swell,
            Some("resolve") => Self::Resolve,
            Some("tick") => Self::Tick,
            Some("pizz") => Self::Pizz,
            Some("whoosh") => Self::Whoosh,
            _ => Self::Reveal,
        }
    }
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
    /// The voice bus it was scored against.
    pub voice: Stereo,
    pub events: Vec<Event>,
    pub chords: Vec<ChordSpan>,
    /// Melodic phrases (start, end of the last onset) in seconds.
    pub phrases: Vec<(f64, f64)>,
    /// Reveal cues placed for authored marks: (seconds, label).
    pub cues: Vec<(f64, String)>,
    /// Marks with no room for a cue (deep inside a long sentence).
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
            .or_else(|| candidates.iter().copied().find(|&c| c > hi && c <= total - min));
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

/// The motif notes that fit a gap: (onset s, MIDI pitch, duration s). Onsets
/// run from `start`; the longest prefix whose last onset is ≤ `limit` is
/// kept, if it has at least two notes.
pub fn fit_phrase(
    motif: &Motif,
    key: &Key,
    tonic_midi: i32,
    beat_s: f64,
    start: f64,
    limit: f64,
) -> Vec<(f64, i32, f64)> {
    let pitches = motif.pitches(key, tonic_midi);
    let mut out = Vec::new();
    let mut t = start;
    for (p, d) in pitches.into_iter().zip(&motif.durations) {
        if t > limit {
            break;
        }
        // keep the melody in its register: C4..C6
        let mut p = p;
        while p < UPPER_LOW {
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
    let fit = |beat: f64| fit_phrase(motif, &identity.key, identity.tonic_midi, beat, start, limit);
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
/// melody's register (C4 up: never into the narrator's band).
pub fn avoid_minor_ninths(pitch: i32, chord: &Chord, key: &Key) -> i32 {
    let tones = chord.pitch_classes();
    let clashes = |p: i32| {
        tones
            .iter()
            .any(|&c| (p - c).rem_euclid(12) == 1 || (c - p).rem_euclid(12) == 1)
    };
    if !clashes(pitch) {
        return pitch;
    }
    let scale = key.pitch_classes();
    for d in [1, -1, 2, -2, 3, -3] {
        let p = pitch + d;
        if (UPPER_LOW..=88).contains(&p) && scale.contains(&p.rem_euclid(12)) && !clashes(p) {
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
/// side) scaled by `gain` (per sample: the thin-outs before reveals), an FDN
/// room fed at `send` (per sample: more room at higher intensity), and the
/// bass through gentle warmth.
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
    let warm: Vec<f64> = bass.l.iter().map(|&x| w.process(x)).collect();
    add_scaled(
        &mut bed,
        &Stereo {
            r: warm.clone(),
            l: warm,
        },
        1.0,
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
    let mut g = vec![1.0_f64; n];
    let depth = 1.0 - db_to_gain(-4.0);
    for &t in reveals {
        let (a, b) = (((t - 1.0).max(0.0) * FS) as usize, (((t + 1.5) * FS) as usize).min(n));
        for (i, v) in g.iter_mut().enumerate().take(b).skip(a) {
            let s = i as f64 / FS;
            let dip = if s < t {
                rcos((s - (t - 1.0)) / 1.0)
            } else {
                1.0 - rcos((s - t) / 1.5)
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
        .filter(|e| matches!(e.track, Track::Melody | Track::Answer | Track::Cue))
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
        voicing: voice_lead(&chord, 4, None),
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
        for l in &s.timeline.lines {
            let wav = std::fs::read(&l.wav)?;
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

    // 3. harmony, planned inside each chapter: a chapter enters the
    //    progression where its index puts it, so consecutive chapters keep
    //    moving; the video's last chord is the tonic when the plan resolves
    let resolve = sections.last().is_some_and(|s| s.plan.resolve);
    let len = identity.progression.len();
    let mut chords: Vec<ChordSpan> = Vec::new();
    let mut events = Vec::new();
    for (c, (name, a, b)) in chapters.iter().enumerate() {
        let (t0, t1) = (*a as f64 / FS, *b as f64 / FS);
        let local: Vec<(f64, f64)> = fine
            .iter()
            .filter(|r| r.1 > t0 && r.0 < t1)
            .map(|&(s, e)| (s.max(t0) - t0, e.min(t1) - t0))
            .collect();
        let cands = change_candidates(&local, t1 - t0);
        let mut rng = Rng::new(seed_for(&identity.name, name, "harmony", 0));
        let changes = plan_changes(&cands, t1 - t0, identity.chord_s, &mut rng);
        // the resolving chapter enters the progression so that its last chord
        // is the tonic: the cycle is kept whole and nothing is merged
        let offset = if c + 1 == chapters.len() && resolve {
            (1 + len * changes.len() - changes.len()) % len
        } else {
            c % len
        };
        let mut prev: Option<Voicing> = None;
        let mut mine: Vec<ChordSpan> = Vec::new();
        for (k, &s) in changes.iter().enumerate() {
            let (start, end) = (t0 + s, changes.get(k + 1).map_or(t1, |&x| t0 + x));
            let chord = identity.progression[(offset + k) % len].clone();
            // a progression that repeats a chord just holds it
            if let Some(p) = mine.last_mut().filter(|p| p.chord == chord) {
                p.end = end;
                continue;
            }
            let voicing = voice_lead(&chord, 4, prev.as_ref());
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

    // 4. the leitmotif in gaps, numbered within each chapter
    let mut phrases = Vec::new();
    let gaps = duck::gaps(&speech, total, MIN_PHRASE_GAP);
    let mut gap_index = vec![0usize; chapters.len()];
    for (g, &(a, b)) in gaps.iter().enumerate() {
        let ch = chapter_at(a);
        let gi = gap_index[ch];
        gap_index[ch] += 1;
        let name = chapters[ch].0.as_str();
        let mut prng = Rng::new(seed_for(&identity.name, name, "phrase", gi as u64));
        let final_gap = g + 1 == gaps.len() && resolve;
        let motif = if final_gap {
            let mut m = identity.motif.clone();
            m.degrees.extend(&identity.resolution.degrees);
            m.durations.extend(&identity.resolution.durations);
            m
        } else {
            match prng.uniform() {
                u if u < 0.45 => identity.motif.clone(),
                u if u < 0.65 => identity.motif.transposed(2),
                u if u < 0.85 => identity.motif.inverted(),
                _ => identity.motif.augmented(1.5),
            }
        };
        // start once the voice has cleared; the last onset leaves room for
        // the duck to come down before the next line (pre-roll + attack)
        let start = if a <= 0.0 { 0.9 } else { a + 0.35 };
        let limit = if b >= total { total - 2.5 } else { b - 0.6 };
        let mut notes = elastic_phrase(&motif, identity, start, limit, final_gap);
        if final_gap && notes.len() < motif.degrees.len() {
            // no room for the whole closing phrase: the resolution alone, so
            // the video still ends on the tonic it promised
            let res = elastic_phrase(&identity.resolution, identity, start, limit, true);
            if res.len() == identity.resolution.degrees.len() {
                notes = res;
            }
        }
        let Some(&(last_t, _, last_d)) = notes.last() else {
            continue;
        };
        for (i, &(t, p, d)) in notes.iter().enumerate() {
            let p = avoid_minor_ninths(p, &chord_at(t).chord, &identity.key);
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
        // a soft answer: two chord tones an octave up, if there is room
        let t = last_t + last_d + 0.25;
        if t + 0.4 <= limit {
            for (i, &m) in top_two(&chord_at(t).voicing).iter().enumerate() {
                let p = if m < 72 { m + 12 } else { m };
                events.push(Event {
                    start: t + 0.03 * i as f64,
                    hold: 1.2,
                    track: Track::Answer,
                    inst: identity.palette.answer,
                    pitch: f64::from(p.min(88)),
                    velocity: 0.32,
                    seed: seed_for(&identity.name, name, "answer", (gi * 4 + i) as u64),
                    pan: if i == 0 { -0.4 } else { 0.4 },
                });
            }
        }
    }

    // 5. authored marks, by kind: a reveal, pizzicato or resolution goes on
    //    the mark, or where the sentence it falls in ends (if that is within
    //    2 s), else it is skipped; a swell rises into the mark, a whoosh is
    //    centred on it, a tick lands on it even under speech (a counter must
    //    stay in sync with what it counts; the duck and pocket keep it soft)
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
            let seed = |track: &str, j: usize| seed_for(&identity.name, name, track, (i * 8 + j) as u64);
            let when = match kind {
                CueKind::Swell | CueKind::Whoosh | CueKind::Tick => Some(t),
                _ => match fine.iter().find(|r| t >= r.0 && t < r.1) {
                    None => Some(t),
                    Some(r) if r.1 - t <= 2.0 => Some(r.1 + 0.05),
                    Some(_) => None,
                },
            };
            // no room (deep in a sentence, at the very end), or too dense (rule 5)
            let Some(when) = when.filter(|&w| w < total - 0.5 && cue_allowed(kind, w, &placed)) else {
                skipped_marks.push(m.label.clone());
                continue;
            };
            placed.push((when, kind));
            let v = &chord_at(when).voicing;
            match kind {
                CueKind::Reveal => {
                    for (j, &mp) in top_two(v).iter().enumerate() {
                        let p = if mp < 72 { mp + 12 } else { mp };
                        events.push(Event {
                            start: when + 0.04 * j as f64,
                            hold: 1.5,
                            track: Track::Cue,
                            inst: identity.palette.answer,
                            pitch: f64::from(p.min(88)),
                            velocity: 0.4,
                            seed: seed("cue", j),
                            pan: if j == 0 { -0.45 } else { 0.45 },
                        });
                    }
                }
                CueKind::Pizz => {
                    let p = v.upper[v.upper.len() / 2];
                    events.push(Event {
                        start: when,
                        hold: 0.4,
                        track: Track::Cue,
                        inst: Instrument::Pluck,
                        pitch: f64::from(if p < 67 { p + 12 } else { p }),
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
                    );
                    for (j, (nt, np, nd)) in notes.into_iter().enumerate() {
                        events.push(Event {
                            start: nt,
                            hold: nd * 1.4,
                            track: Track::Cue,
                            inst: identity.palette.melody,
                            pitch: f64::from(avoid_minor_ninths(np, &chord_at(nt).chord, &identity.key)),
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
            }
            cues.push((when, m.label.clone()));
        }
    }

    // rule 7 for as long as a note sounds: a melody, answer or cue note held
    // into a chord it would clash with is released at that chord's change
    for e in &mut events {
        if matches!(e.track, Track::Melody | Track::Answer | Track::Cue) {
            e.hold = hold_until_clash(e.pitch.round() as i32, e.start, e.hold, &chords).max(0.05);
        }
    }

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
        .filter(|c| matches!(CueKind::from_label(&c.1), CueKind::Reveal | CueKind::Resolve))
        .map(|c| c.0)
        .collect();
    let mut sum = bed_bus(identity, &events, n, &send, &thin_outs(&reveals, n));
    drop(send);
    sum.scale(db_to_gain(v_ref + identity.bed_lu - ref_bed));
    let melody = melody_bus(&events, n);
    add_scaled(&mut sum, &melody, db_to_gain(v_ref + identity.melody_lu - ref_mel));
    drop(melody);
    if !sfx.is_empty() {
        let effects = sfx_bus(&sfx, n);
        add_scaled(&mut sum, &effects, db_to_gain(v_ref + identity.sfx_lu - ref_sfx));
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

    // 7. the duck, with breaths and listen windows released
    let shape = DuckShape::default();
    let mut u = duck::duck_curve(&speech, n, shape);
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
    let mut music = duck::apply(&sum, &u, identity.duck_db);
    drop(sum);

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
    })
}

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
    }

    #[test]
    fn phrases_fit_their_gap_and_stay_in_register() {
        let key = Key::new(2, IONIAN);
        let m = Motif::new(&[1, 5, 6, 3], &[1.0, 1.0, 1.0, 2.0]);
        let all = fit_phrase(&m, &key, 62, 0.8, 10.0, 20.0);
        assert_eq!(all.iter().map(|n| n.1).collect::<Vec<_>>(), vec![62, 69, 71, 66]);
        assert!((all[3].0 - 12.4).abs() < 1e-9);
        // only three onsets fit before 11.7 s
        assert_eq!(fit_phrase(&m, &key, 62, 0.8, 10.0, 11.7).len(), 3);
        assert!(
            fit_phrase(&m, &key, 62, 0.8, 10.0, 10.5).is_empty(),
            "one note is not a phrase"
        );
        // a low motif is lifted to C4 or above
        let low = fit_phrase(&m.transposed(-7), &key, 62, 0.8, 0.0, 9.0);
        assert!(low.iter().all(|n| n.1 >= UPPER_LOW));
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
        let p = avoid_minor_ninths(67, &d, &key);
        assert_ne!(p, 67);
        assert!(key.pitch_classes().contains(&p.rem_euclid(12)));
        assert!(
            d.pitch_classes()
                .iter()
                .all(|&c| (p - c).rem_euclid(12) != 1 && (c - p).rem_euclid(12) != 1)
        );
        // a chord tone stays
        assert_eq!(avoid_minor_ninths(66, &d, &key), 66);
        // never moved down into the narrator's band: C♯4 clashes with D, and
        // B3 (246.9 Hz) is not an option
        let p = avoid_minor_ninths(61, &d, &key);
        assert!(p >= UPPER_LOW, "{p}");
    }

    #[test]
    fn a_held_note_is_released_where_it_would_clash() {
        let key = Key::new(2, IONIAN);
        let mk = |start: f64, end: f64, chord: Chord| ChordSpan {
            start,
            end,
            voicing: voice_lead(&chord, 4, None),
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
        // 12–16 s gap, one 1 s before a sentence ends, one deep inside a long one
        sections[0].timeline.marks = vec![mark("gap", 13.0), mark("late", 23.0), mark("deep", 18.0)];
        let s = render(&Identity::hoeffding(), &sections, None).unwrap();
        let labels: Vec<&str> = s.cues.iter().map(|c| c.1.as_str()).collect();
        assert_eq!(labels, vec!["gap", "late"]);
        assert_eq!(s.skipped_marks, vec!["deep".to_string()]);
        assert!((s.cues[0].0 - 13.0).abs() < 1e-9, "on the mark");
        assert!(
            s.cues[1].0 > 23.9 && s.cues[1].0 < 24.6,
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
            vec!["too_close".to_string()],
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
            voicing: voice_lead(&chord, 4, None),
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
                let f = hz(e.pitch);
                assert!(!(95.0..250.0).contains(&f), "{e:?}");
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
}
