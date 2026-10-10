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
//! 5. Levels come from the voice's integrated loudness V: the bed at
//!    V + `bed_lu`, the melody at V + `melody_lu` while it plays; the sum goes
//!    through tape, then the duck (`duck_db`) with the speech pocket; a fixed
//!    gain caps the stem's true peak at −12 dBTP (never a limiter).

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
}

impl Identity {
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
            bed_lu: -11.0,
            melody_lu: -12.0,
            duck_db: 8.0,
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

/// Times where a chord may change: just after each pause of ≥ 0.2 s begins,
/// every 0.5 s through longer silences, and at chapter starts; never within
/// 150 ms after a speech onset. `fine` are speech regions (sorted).
pub fn change_candidates(fine: &[(f64, f64)], total: f64, chapter_starts: &[f64]) -> Vec<f64> {
    let mut c = Vec::new();
    let silence = |a: f64, b: f64, c: &mut Vec<f64>| {
        // a silent span [a, b): its start, then every 0.5 s
        let mut t = a;
        while t < b {
            c.push(t);
            t += 0.5;
        }
    };
    let mut prev_end = 0.0;
    for &(s, e) in fine {
        if s - prev_end >= MIN_PAUSE || prev_end == 0.0 {
            let a = if prev_end == 0.0 { 0.0 } else { prev_end + 0.05 };
            silence(a, s, &mut c);
        }
        prev_end = e;
    }
    silence(if fine.is_empty() { 0.0 } else { prev_end + 0.05 }, total, &mut c);
    c.extend(chapter_starts.iter().copied().filter(|&t| t < total));
    c.retain(|&t| !fine.iter().any(|&(s, e)| t >= s && (t < e || t < s + ONSET_GUARD)));
    c.sort_by(f64::total_cmp);
    c.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
    c
}

/// Choose chord change times (the first is 0): each chord lasts a seeded
/// target in [min, max]; a chapter start inside [min/2, max] wins; otherwise
/// the candidate nearest the target within [min, max]; with none there, the
/// first candidate after max (the chord holds, a pedal). A last chord shorter
/// than `min` is not started.
pub fn plan_changes(
    candidates: &[f64],
    chapter_starts: &[f64],
    total: f64,
    (min, max): (f64, f64),
    rng: &mut Rng,
) -> Vec<f64> {
    let mut out = vec![0.0];
    let mut t = 0.0;
    loop {
        let target = t + rng.range(min, max);
        let chapter = chapter_starts
            .iter()
            .copied()
            .find(|&c| c >= t + 0.5 * min && c <= t + max && candidates.iter().any(|&k| (k - c).abs() < 1e-6));
        let inside = candidates
            .iter()
            .copied()
            .filter(|&c| c >= t + min && c <= t + max)
            .min_by(|a, b| (a - target).abs().total_cmp(&(b - target).abs()));
        let next = chapter
            .or(inside)
            .or_else(|| candidates.iter().copied().find(|&c| c > t + max));
        match next {
            Some(c) if total - c >= min => {
                out.push(c);
                t = c;
            }
            _ => break,
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

/// Move `pitch` off a minor ninth (a semitone from any chord tone, hard rule
/// 7) to the nearest key tone that clashes with nothing.
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
        if scale.contains(&p.rem_euclid(12)) && !clashes(p) {
            return p;
        }
    }
    pitch
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

/// Score a whole video: `sections` in order, the voice placed at each line's
/// own gain or at `voice_gain_db` when given.
pub fn render(identity: &Identity, sections: &[Section], voice_gain_db: Option<f64>) -> io::Result<Score> {
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

    // 3. harmony
    let mut rng = Rng::new(seed_for(&identity.name, "", "harmony", 0));
    let cands = change_candidates(&fine, total, &chapter_starts);
    let changes = plan_changes(&cands, &chapter_starts, total, identity.chord_s, &mut rng);
    let resolve = sections.last().is_some_and(|s| s.plan.resolve);
    let mut chords: Vec<ChordSpan> = Vec::new();
    let mut prev: Option<Voicing> = None;
    for (k, &start) in changes.iter().enumerate() {
        let end = changes.get(k + 1).copied().unwrap_or(total);
        let last = k + 1 == changes.len();
        let chord = if last && resolve {
            identity.progression[0].clone()
        } else {
            identity.progression[k % identity.progression.len()].clone()
        };
        // a resolution onto the chord already sounding just extends it
        if let Some(p) = chords.last_mut().filter(|p| p.chord == chord) {
            p.end = end;
            continue;
        }
        let voicing = voice_lead(&chord, 4, prev.as_ref());
        prev = Some(voicing.clone());
        chords.push(ChordSpan {
            start,
            end,
            chord,
            voicing,
        });
    }

    // 4. notes
    let mut events = Vec::new();
    let chapter_name = |k: usize| chapters[k].0.as_str();
    for (k, span) in chords.iter().enumerate() {
        let ch = chapter_at(span.start);
        let intensity = sections[ch].plan.intensity.clamp(0.0, 1.0);
        let bed = match identity.palette.bed {
            Instrument::Pad { .. } => Instrument::Pad {
                brightness: 0.15 + 0.35 * intensity,
            },
            Instrument::Strings { .. } => Instrument::Strings {
                brightness: 0.15 + 0.35 * intensity,
            },
            other => other,
        };
        // the next chord swells in from 0.8 s before its change; this one
        // starts to release 0.3 s before the next change
        let note_start = (span.start - 0.8).max(0.0);
        let hold = (span.end - 0.3 - note_start).max(0.5);
        for (j, &m) in span.voicing.upper.iter().enumerate() {
            events.push(Event {
                start: note_start,
                hold,
                track: Track::Bed,
                inst: bed,
                pitch: f64::from(m),
                velocity: 0.35 + 0.3 * intensity,
                seed: seed_for(&identity.name, chapter_name(ch), "bed", (k * 8 + j) as u64),
                pan: [-0.35, -0.12, 0.12, 0.35][j % 4],
            });
        }
        events.push(Event {
            start: span.start,
            hold: (span.end - span.start - 0.15).max(0.3),
            track: Track::Bass,
            inst: Instrument::Bass,
            pitch: f64::from(span.voicing.bass),
            velocity: 0.5 + 0.2 * intensity,
            seed: seed_for(&identity.name, chapter_name(ch), "bass", k as u64),
            pan: 0.0,
        });
    }
    let chord_at = |t: f64| {
        chords
            .iter()
            .find(|c| t >= c.start && t < c.end)
            .or(chords.last())
            .expect("at least one chord")
    };
    let mut phrases = Vec::new();
    let gaps = duck::gaps(&speech, total, MIN_PHRASE_GAP);
    for (g, &(a, b)) in gaps.iter().enumerate() {
        let ch = chapter_at(a);
        let mut prng = Rng::new(seed_for(&identity.name, chapter_name(ch), "phrase", g as u64));
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
        let notes = fit_phrase(
            &motif,
            &identity.key,
            identity.tonic_midi,
            identity.beat_s,
            start,
            limit,
        );
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
                seed: seed_for(&identity.name, chapter_name(ch), "melody", (g * 16 + i) as u64),
                pan: 0.12 * prng.jitter(1.0),
            });
        }
        phrases.push((notes[0].0, last_t));
        // a soft answer: two chord tones an octave up, if there is room
        let t = last_t + last_d + 0.25;
        if t + 0.4 <= limit {
            let v = &chord_at(t).voicing;
            for (i, &m) in v.upper[2..].iter().enumerate() {
                let p = if m < 72 { m + 12 } else { m };
                events.push(Event {
                    start: t + 0.03 * i as f64,
                    hold: 1.2,
                    track: Track::Answer,
                    inst: identity.palette.answer,
                    pitch: f64::from(p.min(88)),
                    velocity: 0.32,
                    seed: seed_for(&identity.name, chapter_name(ch), "answer", (g * 4 + i) as u64),
                    pan: if i == 0 { -0.4 } else { 0.4 },
                });
            }
        }
    }

    // 5. buses
    let of = |track: Track| events.iter().filter(|e| e.track == track).collect::<Vec<_>>();
    let mut bed = {
        let dry = render_track(&of(Track::Bed), n);
        let (mid, side) = mid_side(&dry);
        let mut chorused = Chorus::juno(0.5, 0.5).run(&mid);
        add_scaled(&mut chorused, &from_mid_side(&vec![0.0; n], &side), 1.0);
        let wet = Fdn::new(identity.bed_t60, 5_000.0, 25.0).run(&chorused, 0.0);
        add_scaled(&mut chorused, &wet, 0.3);
        chorused
    };
    let bass = {
        let dry = render_track(&of(Track::Bass), n);
        let mut w = Warmth::new(1.3, 0.12);
        let l: Vec<f64> = dry.l.iter().map(|&x| w.process(x)).collect();
        Stereo { r: l.clone(), l }
    };
    add_scaled(&mut bed, &bass, 1.0);
    drop(bass);
    let melody = {
        let mut dry = render_track(&of(Track::Melody), n);
        add_scaled(&mut dry, &render_track(&of(Track::Answer), n), 1.0);
        let wet = Plate::new(0.55, 5_500.0).run(&dry, 0.0);
        add_scaled(&mut dry, &wet, 0.3);
        dry
    };

    // 6. levels
    let voice_lufs = meter::integrated(&voice);
    let bed_gain = db_to_gain(voice_lufs + identity.bed_lu - meter::integrated(&bed));
    let mel_lufs = meter::integrated(&melody);
    let mel_gain = if mel_lufs.is_finite() {
        db_to_gain(voice_lufs + identity.melody_lu - mel_lufs)
    } else {
        0.0
    };
    let mut sum = Stereo::silence(n);
    add_scaled(&mut sum, &bed, bed_gain);
    add_scaled(&mut sum, &melody, mel_gain);
    drop((bed, melody));
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
    if true_peak_db > -12.0 {
        cap_db = -12.1 - true_peak_db;
        music.scale(db_to_gain(cap_db));
        true_peak_db += cap_db;
    }

    Ok(Score {
        music,
        voice,
        events,
        chords,
        phrases,
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
        let c = change_candidates(&fine, 12.0, &[]);
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
        // a chapter start inside speech is not a candidate
        let c = change_candidates(&fine, 12.0, &[5.0]);
        assert!(!c.contains(&5.0));
    }

    #[test]
    fn changes_land_on_candidates_within_the_length_range() {
        let cands: Vec<f64> = (0..400).map(|k| k as f64 * 0.37).collect();
        let total = 140.0;
        let mut rng = Rng::new(7);
        let ch = plan_changes(&cands, &[], total, (6.0, 12.0), &mut rng);
        assert_eq!(ch[0], 0.0);
        for w in ch.windows(2) {
            let d = w[1] - w[0];
            assert!((6.0..=12.0).contains(&d), "{d}");
            assert!(cands.iter().any(|&c| (c - w[1]).abs() < 1e-9));
        }
        assert!(total - ch.last().unwrap() >= 6.0);
        // a chapter start inside the window wins
        let ch = plan_changes(&cands, &[cands[10]], total, (6.0, 12.0), &mut Rng::new(7));
        assert!((ch[1] - cands[10]).abs() < 1e-9, "{:?}", &ch[..3]);
        // no candidate for a long stretch: the chord holds (a pedal)
        let sparse = [0.0, 3.0, 30.0, 60.0];
        let ch = plan_changes(&sparse, &[], 80.0, (6.0, 12.0), &mut Rng::new(1));
        assert_eq!(ch, vec![0.0, 30.0, 60.0]);
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
        assert!(s.music.is_finite());
        assert!(s.true_peak_db <= -12.0 + 1e-9, "{}", s.true_peak_db);
        // harmony: chords last 6–12 s except the last, and the video ends on the tonic
        assert!(s.chords.len() >= 5, "{}", s.chords.len());
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
