//! Score QA (plan §7.6): per chapter, from the voice bus and the music stem,
//! the measurements that say whether the music stays out of the narrator's
//! way; and, for a score this engine wrote, checks of the notes it played
//! against the hard rules of §7.3. A failing metric fails the render; the
//! JSON report keeps every number either way.
//!
//! Audio metrics need only two buffers, so the same code judges a score made
//! elsewhere (the Hoeffding v5 pad and chimes) for a fair A/B. Metrics with
//! nothing to measure (no speech in a chapter, no notes for an outside score)
//! are reported as informational, never as passes.

use std::fmt::Write as _;

use crate::duck;
use crate::fft::{third_octave_levels, welch};
use crate::filter::{Biquad, Kind};
use crate::meter;
use crate::score::{ChordSpan, Event, Score, Track};
use crate::units::midi_to_hz;
use crate::{FS, Stereo};

/// ANSI S3.5-1997 one-third-octave band importance, average speech, for the
/// bands of [`crate::fft::THIRD_OCTAVES`] (160 Hz–8 kHz).
pub const BAND_IMPORTANCE: [f64; 18] = [
    0.0083, 0.0095, 0.0150, 0.0289, 0.0440, 0.0578, 0.0653, 0.0711, 0.0818, 0.0844, 0.0882, 0.0898, 0.0868, 0.0844,
    0.0771, 0.0527, 0.0364, 0.0185,
];

/// One measurement. `pass` is `None` for informational rows.
#[derive(Clone, Debug, PartialEq)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub limit: String,
    pub pass: Option<bool>,
    pub detail: String,
}

/// A chapter's QA.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Report {
    pub chapter: String,
    pub metrics: Vec<Metric>,
}

impl Report {
    pub fn passed(&self) -> bool {
        self.metrics.iter().all(|m| m.pass != Some(false))
    }

    pub fn failures(&self) -> Vec<&Metric> {
        self.metrics.iter().filter(|m| m.pass == Some(false)).collect()
    }

    pub fn get(&self, name: &str) -> Option<&Metric> {
        self.metrics.iter().find(|m| m.name == name)
    }

    fn push(&mut self, name: &str, value: f64, limit: &str, pass: Option<bool>, detail: impl Into<String>) {
        self.metrics.push(Metric {
            name: name.into(),
            value,
            limit: limit.into(),
            pass,
            detail: detail.into(),
        });
    }

    /// The report as JSON (std only; non-finite numbers become `null`).
    pub fn to_json(&self) -> String {
        let mut s = String::new();
        let _ = write!(
            s,
            "{{\"chapter\":{},\"passed\":{},\"metrics\":[",
            json_str(&self.chapter),
            self.passed()
        );
        for (i, m) in self.metrics.iter().enumerate() {
            let _ = write!(
                s,
                "{}{{\"name\":{},\"value\":{},\"limit\":{},\"pass\":{},\"detail\":{}}}",
                if i == 0 { "" } else { "," },
                json_str(&m.name),
                json_num(m.value),
                json_str(&m.limit),
                m.pass.map_or("null".to_string(), |p| p.to_string()),
                json_str(&m.detail)
            );
        }
        s.push_str("]}");
        s
    }
}

fn json_num(v: f64) -> String {
    if v.is_finite() {
        format!("{}", (v * 1e4).round() / 1e4)
    } else {
        "null".into()
    }
}

fn json_str(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// What a score played inside one chapter, in chapter-relative seconds.
#[derive(Clone, Debug, Default)]
pub struct ScoreFacts {
    pub events: Vec<Event>,
    pub chords: Vec<ChordSpan>,
    pub phrases: Vec<(f64, f64)>,
}

impl ScoreFacts {
    /// Chapter `k` of a whole-video score: notes that start in it, chords that
    /// overlap it (whole, so a chord's real length is judged), phrases that
    /// start in it.
    pub fn chapter(score: &Score, k: usize) -> Self {
        let (_, a, b) = score.chapters[k];
        let (t0, t1) = (a as f64 / FS, b as f64 / FS);
        Self {
            events: score
                .events
                .iter()
                .filter(|e| e.start >= t0 && e.start < t1)
                .map(|e| Event {
                    start: e.start - t0,
                    ..e.clone()
                })
                .collect(),
            chords: score
                .chords
                .iter()
                .filter(|c| c.end > t0 && c.start < t1)
                .map(|c| ChordSpan {
                    start: c.start - t0,
                    end: c.end - t0,
                    ..c.clone()
                })
                .collect(),
            phrases: score
                .phrases
                .iter()
                .filter(|p| p.0 >= t0 && p.0 < t1)
                .map(|p| (p.0 - t0, p.1 - t0))
                .collect(),
        }
    }
}

// ----------------------------------------------------------------- helpers

fn mono(x: &Stereo) -> Vec<f64> {
    x.l.iter().zip(&x.r).map(|(a, b)| (a + b) / 2f64.sqrt()).collect()
}

fn inside(t: f64, regions: &[(f64, f64)]) -> bool {
    regions.iter().any(|&(a, b)| t >= a && t < b)
}

fn percentile(v: &[f64], p: f64) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    let mut s = v.to_vec();
    s.sort_by(f64::total_cmp);
    let i = ((s.len() - 1) as f64 * p / 100.0).round() as usize;
    s[i]
}

/// Speech regions of a voice bus, as the duck sees them.
pub fn speech_regions(voice: &Stereo) -> Vec<(f64, f64)> {
    duck::regions(&duck::vad(voice), 1.2)
}

/// Voice minus music short-term (3 s) loudness, one value per 0.1 s window
/// whose centre lies in speech. Silent music counts as −120 LUFS.
pub fn voice_minus_music(voice: &Stereo, music: &Stereo, speech: &[(f64, f64)]) -> Vec<f64> {
    let v = meter::windowed(voice, 3.0, 0.1);
    let m = meter::windowed(music, 3.0, 0.1);
    v.iter()
        .zip(&m)
        .enumerate()
        .filter(|(k, _)| inside(*k as f64 * 0.1 + 1.5, speech))
        .map(|(_, (&a, &b))| a - b.max(-120.0))
        .collect()
}

/// The music's loudest momentary (400 ms) loudness during speech minus the
/// speech's loudest (LU).
pub fn loudest_400ms(voice: &Stereo, music: &Stereo, speech: &[(f64, f64)]) -> f64 {
    let v = meter::windowed(voice, 0.4, 0.1);
    let m = meter::windowed(music, 0.4, 0.1);
    let (mut vmax, mut mmax) = (f64::NEG_INFINITY, f64::NEG_INFINITY);
    for (k, (&a, &b)) in v.iter().zip(&m).enumerate() {
        if inside(k as f64 * 0.1 + 0.2, speech) {
            vmax = vmax.max(a);
            mmax = mmax.max(b);
        }
    }
    mmax.max(-120.0) - vmax
}

/// SII-lite loss per 400 ms block of speech (docs/research/sound_design/
/// scripts/sii.py): the speech spectrum S is the voice's long-term third-octave
/// levels over its active samples; each block's music levels N are the masker;
/// band audibility clip((S − N + 15) / 30); loss = Σ w·(1 − A).
pub fn sii_losses(voice: &Stereo, music: &Stereo, speech: &[(f64, f64)]) -> Vec<f64> {
    let active = duck::vad(voice);
    let vm = mono(voice);
    let speech_samples: Vec<f64> = active
        .iter()
        .enumerate()
        .filter(|(_, a)| **a)
        .flat_map(|(k, _)| vm[k * duck::VAD_FRAME..(k + 1) * duck::VAD_FRAME].iter().copied())
        .collect();
    if speech_samples.len() < 8_192 {
        return Vec::new();
    }
    let s = third_octave_levels(&speech_samples, FS);
    let mm = mono(music);
    let block = (0.4 * FS) as usize;
    (0..mm.len() / block)
        .filter(|&k| inside((k as f64 + 0.5) * 0.4, speech))
        .map(|k| {
            let n = third_octave_levels(&mm[k * block..(k + 1) * block], FS);
            s.iter()
                .zip(&n)
                .zip(&BAND_IMPORTANCE)
                .map(|((sv, nv), w)| w * (1.0 - ((sv - nv + 15.0) / 30.0).clamp(0.0, 1.0)))
                .sum()
        })
        .collect()
}

/// Music-to-voice energy ratio (dB) in [lo, hi) Hz over the speech regions.
pub fn band_ratio_db(voice: &Stereo, music: &Stereo, speech: &[(f64, f64)], lo: f64, hi: f64) -> f64 {
    let pick = |x: &Stereo| -> Vec<f64> {
        let m = mono(x);
        speech
            .iter()
            .flat_map(|&(a, b)| {
                let (i, j) = ((a * FS) as usize, ((b * FS) as usize).min(m.len()));
                m[i.min(j)..j].to_vec()
            })
            .collect()
    };
    let (v, m) = (pick(voice), pick(music));
    if v.len() < 4_096 {
        return f64::NAN;
    }
    let band = |x: &[f64]| {
        let (df, p) = welch(x, FS, 4_096);
        p.iter()
            .enumerate()
            .filter(|(k, _)| (lo..hi).contains(&(*k as f64 * df)))
            .map(|(_, v)| v)
            .sum::<f64>()
    };
    10.0 * (band(&m).max(1e-30) / band(&v).max(1e-30)).log10()
}

/// Clicks: 5 ms frames of the music above 8 kHz that jump ≥ 20 dB over the
/// previous 50 ms and exceed −70 dBFS in that band.
pub fn click_flags(music: &Stereo) -> usize {
    let mut hp = Biquad::new(Kind::HighPass { q: 0.707 }, 8_000.0);
    let x: Vec<f64> = mono(music).into_iter().map(|v| hp.process(v)).collect();
    let frame = 240;
    let db: Vec<f64> = x
        .chunks(frame)
        .map(|c| 10.0 * (c.iter().map(|v| v * v).sum::<f64>() / c.len() as f64 + 1e-30).log10())
        .collect();
    (10..db.len())
        .filter(|&k| {
            let prev = db[k - 10..k].iter().copied().fold(f64::NEG_INFINITY, f64::max);
            db[k] > -70.0 && db[k] >= prev + 20.0
        })
        .count()
}

/// The lowest L/R correlation over 3 s windows with any real signal.
pub fn min_mono_correlation(music: &Stereo) -> f64 {
    let w = (3.0 * FS) as usize;
    let mut worst = f64::INFINITY;
    for k in 0..music.len() / w {
        let (l, r) = (&music.l[k * w..(k + 1) * w], &music.r[k * w..(k + 1) * w]);
        let (ll, rr): (f64, f64) = (l.iter().map(|v| v * v).sum(), r.iter().map(|v| v * v).sum());
        if (ll + rr) / (2 * w) as f64 > 1e-6 {
            let lr: f64 = l.iter().zip(r).map(|(a, b)| a * b).sum();
            worst = worst.min(lr / (ll * rr).sqrt().max(1e-30));
        }
    }
    worst
}

/// The most times any 8 s segment recurs (correlation > 0.98) elsewhere,
/// comparing the stem's consecutive 8 s chunks (a loop whose period is a
/// multiple of 8 s is caught exactly; other periods only partly).
pub fn max_repeats(music: &Stereo) -> usize {
    let w = (8.0 * FS) as usize;
    let m = mono(music);
    let segs: Vec<&[f64]> = m
        .chunks_exact(w)
        .filter(|s| s.iter().map(|v| v * v).sum::<f64>() / w as f64 > 1e-6)
        .collect();
    let norms: Vec<f64> = segs
        .iter()
        .map(|s| s.iter().map(|v| v * v).sum::<f64>().sqrt())
        .collect();
    (0..segs.len())
        .map(|i| {
            (0..segs.len())
                .filter(|&j| j != i)
                .filter(|&j| {
                    let dot: f64 = segs[i].iter().zip(segs[j]).map(|(a, b)| a * b).sum();
                    dot / (norms[i] * norms[j]) > 0.98
                })
                .count()
        })
        .max()
        .unwrap_or(0)
}

/// Exposed beating (plan §7.6): the deepest slow amplitude modulation of the
/// music over steady stretches, in dB. Each 2 s window (hop 0.5 s) counts
/// only when the level cannot be moving for another reason: no score onset
/// in the 3 s before it or inside it (a chord change crossfades for ~3 s), no
/// duck ramp (wholly inside speech after its first 0.4 s, or wholly in a gap
/// from 1.5 s after the voice stops to 0.4 s before it returns), clear of the
/// stem's first and last 3 s, and above −60 dBFS. Depth is the p95/p5 ratio
/// of the 25 ms RMS envelope divided by its 1 s moving mean. `None` when no
/// window qualifies.
pub fn exposed_beating_db(music: &Stereo, onsets: &[f64], speech: &[(f64, f64)]) -> Option<f64> {
    let m = mono(music);
    let frame = (0.025 * FS) as usize;
    let env: Vec<f64> = m
        .chunks(frame)
        .map(|c| (c.iter().map(|v| v * v).sum::<f64>() / c.len() as f64).sqrt())
        .collect();
    let fps = 1.0 / 0.025;
    let half = (0.5 * fps) as usize;
    let total = m.len() as f64 / FS;
    let steady = |a: f64, b: f64| {
        let in_speech = speech.iter().any(|&(s, e)| a >= s + 0.4 && b <= e);
        let in_gap = !speech.iter().any(|&(s, e)| s < b + 0.4 && e + 1.5 > a);
        (in_speech || in_gap) && a >= 3.0 && b <= total - 3.0 && !onsets.iter().any(|&t| t > a - 3.0 && t < b)
    };
    let mut worst: Option<f64> = None;
    let mut a = 0.0;
    while a + 2.0 <= total {
        let b = a + 2.0;
        if steady(a, b) {
            let (i0, i1) = ((a * fps) as usize, (b * fps) as usize);
            let ratio: Vec<f64> = (i0..i1.min(env.len()))
                .filter_map(|i| {
                    let (lo, hi) = (i.saturating_sub(half), (i + half).min(env.len()));
                    let mean = env[lo..hi].iter().sum::<f64>() / (hi - lo) as f64;
                    (mean > 1e-3).then(|| env[i] / mean)
                })
                .collect();
            if ratio.len() > 20 {
                let depth = 20.0 * (percentile(&ratio, 95.0) / percentile(&ratio, 5.0).max(1e-9)).log10();
                worst = Some(worst.map_or(depth, |w: f64| w.max(depth)));
            }
        }
        a += 0.5;
    }
    worst
}

/// Unique onset times of the score's notes (a chord's bed notes start together).
fn onsets(events: &[Event]) -> Vec<f64> {
    let mut t: Vec<f64> = events.iter().map(|e| (e.start * 100.0).round() / 100.0).collect();
    t.sort_by(f64::total_cmp);
    t.dedup();
    t
}

fn max_per_second(times: &[f64]) -> usize {
    times
        .iter()
        .map(|&t| times.iter().filter(|&&u| u >= t && u < t + 1.0).count())
        .max()
        .unwrap_or(0)
}

// ----------------------------------------------------------------- evaluate

/// QA one chapter: `voice` and `music` are its buses (same length, which must
/// equal `expected_len`); `facts` are the score's notes when it is ours.
pub fn evaluate(
    chapter: &str,
    voice: &Stereo,
    music: &Stereo,
    expected_len: usize,
    facts: Option<&ScoreFacts>,
) -> Report {
    let mut r = Report {
        chapter: chapter.into(),
        ..Default::default()
    };
    let speech = speech_regions(voice);
    let has_speech = !speech.is_empty();
    // a speech metric with nothing to measure (no speech, a chapter shorter
    // than its window) is informational, never a pass and never a failure
    let judge = |value: f64, ok: bool| {
        if has_speech && value.is_finite() {
            Some(ok)
        } else {
            None
        }
    };

    // sanity first: everything below assumes finite, aligned buffers
    let finite = music.is_finite();
    r.push("finite", if finite { 1.0 } else { 0.0 }, "no NaN/inf", Some(finite), "");
    let len_ok = music.len() == expected_len && voice.len() == expected_len;
    r.push(
        "length",
        music.len() as f64,
        &format!("= {expected_len} samples"),
        Some(len_ok),
        format!("voice {} samples", voice.len()),
    );
    if !finite || !len_ok {
        return r;
    }
    let dc = [&music.l, &music.r]
        .iter()
        .map(|c| (c.iter().sum::<f64>() / c.len().max(1) as f64).abs())
        .fold(0.0, f64::max);
    let dc_db = 20.0 * dc.max(1e-12).log10();
    r.push("dc_dbfs", dc_db, "<= -60", Some(dc_db <= -60.0), "");

    // levels and masking over speech
    let vm = voice_minus_music(voice, music, &speech);
    let (med, p5) = (percentile(&vm, 50.0), percentile(&vm, 5.0));
    r.push(
        "voice_minus_music_median_lu",
        med,
        ">= 16",
        judge(med, med >= 16.0),
        "short-term (3 s) windows centred in speech",
    );
    r.push("voice_minus_music_p5_lu", p5, ">= 12", judge(p5, p5 >= 12.0), "");
    let l400 = loudest_400ms(voice, music, &speech);
    r.push(
        "music_loudest_400ms_re_speech_lu",
        l400,
        "<= -15",
        judge(l400, l400 <= -15.0),
        "music's loudest momentary during speech minus the speech's",
    );
    let sii = sii_losses(voice, music, &speech);
    let (sii_med, sii_max) = (percentile(&sii, 50.0), sii.iter().copied().fold(f64::NAN, f64::max));
    r.push(
        "sii_loss_median",
        sii_med,
        "<= 0.01",
        judge(sii_med, sii_med <= 0.01),
        format!("{} blocks of 400 ms", sii.len()),
    );
    r.push("sii_loss_max", sii_max, "<= 0.03", judge(sii_max, sii_max <= 0.03), "");
    let br = band_ratio_db(voice, music, &speech, 1_000.0, 4_000.0);
    r.push(
        "music_to_voice_1_4khz_db",
        br,
        "<= -20",
        judge(br, br <= -20.0),
        "energy ratio over speech",
    );

    // the stem itself
    let tp = meter::true_peak_db(music);
    r.push("true_peak_dbtp", tp, "<= -12", Some(tp <= -12.0), "");
    let clicks = click_flags(music);
    r.push(
        "clicks",
        clicks as f64,
        "= 0",
        Some(clicks == 0),
        "5 ms frames > 8 kHz jumping >= 20 dB",
    );
    let corr = min_mono_correlation(music);
    r.push(
        "min_lr_correlation",
        corr,
        ">= 0",
        if corr.is_finite() { Some(corr >= 0.0) } else { None },
        "per 3 s window",
    );
    let rep = max_repeats(music);
    r.push(
        "max_8s_repeats",
        rep as f64,
        "<= 3",
        Some(rep <= 3),
        "other 8 s segments correlating > 0.98 with one",
    );

    // the score's own notes
    let Some(f) = facts else {
        r.push(
            "score_rules",
            f64::NAN,
            "n/a",
            None,
            "no score facts: an outside score is judged on its audio only",
        );
        return r;
    };
    // every chord sounding here, at its full length (one held through a
    // whole chapter is exactly what this gate exists to catch)
    let longest = f.chords.iter().map(|c| c.end - c.start).fold(0.0, f64::max);
    r.push(
        "longest_chord_s",
        longest,
        "<= 16",
        if f.chords.is_empty() {
            None
        } else {
            Some(longest <= 16.0)
        },
        format!("{} chords sound in this chapter", f.chords.len()),
    );
    let all = onsets(&f.events);
    let (in_speech, in_gaps): (Vec<f64>, Vec<f64>) = all.iter().partition(|&&t| inside(t, &speech));
    let (bs, bg) = (max_per_second(&in_speech), max_per_second(&in_gaps));
    r.push("onsets_per_s_speech", bs as f64, "<= 2", Some(bs <= 2), "");
    r.push("onsets_per_s_gaps", bg as f64, "<= 6", Some(bg <= 6), "");
    let rule1 = f
        .events
        .iter()
        .filter(|e| e.hold >= 1.0 && (95.0..250.0).contains(&midi_to_hz(e.pitch)))
        .filter(|e| speech.iter().any(|&(a, b)| e.start < b && e.start + e.hold > a))
        .count();
    r.push(
        "rule1_fundamentals_95_250hz_in_speech",
        rule1 as f64,
        "= 0",
        Some(rule1 == 0),
        "sustained notes in the narrator's band",
    );
    let rule8 = f.phrases.iter().filter(|p| inside(p.0, &speech)).count();
    r.push(
        "rule8_phrases_start_in_speech",
        rule8 as f64,
        "= 0",
        Some(rule8 == 0),
        "",
    );
    // a note is judged against every chord it sounds over, not only the one
    // it starts on
    let rule7 = f
        .events
        .iter()
        .filter(|e| matches!(e.track, Track::Melody | Track::Answer | Track::Cue))
        .filter(|e| {
            f.chords
                .iter()
                .filter(|c| c.start < e.start + e.hold && c.end > e.start)
                .any(|c| crate::score::clashes(e.pitch.round() as i32, &c.chord))
        })
        .count();
    r.push(
        "rule7_minor_ninths",
        rule7 as f64,
        "= 0",
        Some(rule7 == 0),
        "melody notes a semitone from a tone of a chord they sound over",
    );
    let mut rule6 = 0;
    for (i, a) in f.events.iter().enumerate() {
        for b in &f.events[i + 1..] {
            if (b.start - a.start).abs() < 10.0
                && a.inst == b.inst
                && a.pitch == b.pitch
                && a.velocity == b.velocity
                && a.seed == b.seed
                && a.hold == b.hold
            {
                rule6 += 1;
            }
        }
    }
    r.push("rule6_identical_notes_10s", rule6 as f64, "= 0", Some(rule6 == 0), "");
    match exposed_beating_db(music, &onsets(&f.events), &speech) {
        Some(d) => r.push(
            "exposed_beating_db",
            d,
            "<= 6",
            Some(d <= 6.0),
            "deepest slow amplitude modulation over steady stretches",
        ),
        None => r.push(
            "exposed_beating_db",
            f64::NAN,
            "<= 6",
            None,
            "no steady stretch to measure",
        ),
    }
    let rough = f
        .chords
        .iter()
        .map(|c| {
            let mut hz: Vec<f64> = c.voicing.upper.iter().map(|&m| midi_to_hz(f64::from(m))).collect();
            hz.sort_by(f64::total_cmp);
            hz.windows(2).filter(|w| (15.0..=60.0).contains(&(w[1] - w[0]))).count()
        })
        .sum::<usize>();
    r.push(
        "rough_voice_pairs",
        rough as f64,
        "= 0",
        Some(rough == 0),
        "adjacent bed voices 15-60 Hz apart",
    );
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::Rng;
    use std::f64::consts::TAU;

    /// Speech-like noise bursts at `spans` (seconds) over `total` seconds.
    fn speech(spans: &[(f64, f64)], total: f64, level: f64) -> Stereo {
        let mut rng = Rng::new(3);
        let n = (total * FS) as usize;
        let m: Vec<f64> = (0..n)
            .map(|i| {
                let t = i as f64 / FS;
                if spans.iter().any(|&(a, b)| t >= a && t < b) {
                    level * (0.6 + 0.4 * (TAU * 4.0 * t).sin()) * rng.normal()
                } else {
                    0.0
                }
            })
            .collect();
        Stereo::from_mono(&m)
    }

    fn tone(hz: f64, amp: f64, total: f64) -> Stereo {
        let n = (total * FS) as usize;
        let m: Vec<f64> = (0..n).map(|i| amp * (TAU * hz * i as f64 / FS).sin()).collect();
        Stereo::from_mono(&m)
    }

    #[test]
    fn json_is_well_formed_and_escaped() {
        let mut r = Report {
            chapter: "a\"b".into(),
            ..Default::default()
        };
        r.push("x", f64::NAN, "<= 1", None, "line\nbreak");
        r.push("y", 0.123456789, "= 0", Some(false), "");
        let j = r.to_json();
        assert!(j.contains("\"chapter\":\"a\\\"b\""), "{j}");
        assert!(j.contains("\"value\":null") && j.contains("\"value\":0.1235"), "{j}");
        assert!(j.contains("\"passed\":false") && j.contains("line\\nbreak"), "{j}");
    }

    #[test]
    fn quiet_low_music_passes_and_a_loud_chime_under_speech_fails() {
        let total = 20.0;
        let v = speech(&[(1.0, 9.0), (11.0, 19.0)], total, 0.1);
        // a quiet low pad-like tone, 30 dB under the voice
        let quiet = tone(330.0, 0.1 * 10f64.powf(-30.0 / 20.0), total);
        let r = evaluate("ok", &v, &quiet, v.len(), None);
        for m in [
            "voice_minus_music_median_lu",
            "music_loudest_400ms_re_speech_lu",
            "sii_loss_max",
            "music_to_voice_1_4khz_db",
        ] {
            assert_eq!(r.get(m).unwrap().pass, Some(true), "{m}: {:?}", r.get(m));
        }
        // a planted negative: a loud 2.4 kHz "chime" under speech fails
        // intelligibility, level and the speech-band ratio
        let chime = tone(2_400.0, 0.1 * 10f64.powf(-8.0 / 20.0), total);
        let r = evaluate("chime", &v, &chime, v.len(), None);
        for m in [
            "sii_loss_max",
            "music_loudest_400ms_re_speech_lu",
            "music_to_voice_1_4khz_db",
        ] {
            assert_eq!(r.get(m).unwrap().pass, Some(false), "{m}: {:?}", r.get(m));
        }
        assert!(!r.passed());
    }

    #[test]
    fn the_v5_chime_under_speech_fails_and_a_static_90_s_chord_fails_monotony() {
        // the v5 chime: free-free bar partials 1 : 2.76 : 5.4 at 880 Hz (2.4
        // and 4.75 kHz, in the intelligibility band), struck under speech at
        // the level v5 used against its −9.4 dB narration
        let total = 30.0;
        let v = speech(&[(1.0, 14.0), (15.0, 29.0)], total, 0.1 * 10f64.powf(-9.4 / 20.0));
        let n = v.len();
        let mut chime = vec![0.0; n];
        for strike in [3.0, 9.0, 18.0, 24.0] {
            let at = (strike * FS) as usize;
            for i in 0..(2.0 * FS) as usize {
                let t = i as f64 / FS;
                let y: f64 = [(1.0, 1.0, 1.2), (2.76, 0.5, 0.5), (5.4, 0.3, 0.25)]
                    .iter()
                    .map(|&(r, a, tau)| a * (-t / tau).exp() * (TAU * 880.0 * r * t).sin())
                    .sum();
                chime[at + i] += 0.3 * 10f64.powf(-10.0 / 20.0) * y;
            }
        }
        let r = evaluate("v5 chime", &v, &Stereo::from_mono(&chime), n, None);
        for m in ["sii_loss_max", "music_loudest_400ms_re_speech_lu"] {
            assert_eq!(r.get(m).unwrap().pass, Some(false), "{m}: {:?}", r.get(m));
        }
        // a 90 s static chord: one ChordSpan for the whole chapter
        let v = speech(&[(1.0, 40.0), (45.0, 85.0)], 90.0, 0.1);
        let chord = crate::theory::Chord::maj9(2);
        let facts = ScoreFacts {
            chords: vec![ChordSpan {
                start: 0.0,
                end: 90.0,
                voicing: crate::theory::voice_lead(&chord, 4, None),
                chord,
            }],
            ..Default::default()
        };
        let r = evaluate("static", &v, &Stereo::silence(v.len()), v.len(), Some(&facts));
        let m = r.get("longest_chord_s").unwrap();
        assert_eq!(m.pass, Some(false), "{m:?}");
    }

    #[test]
    fn nothing_to_measure_is_informational_not_a_failure() {
        // 2 s with speech: no 3 s short-term window fits, so the level metrics
        // have nothing to measure
        let v = speech(&[(0.2, 1.8)], 2.0, 0.1);
        let r = evaluate("short", &v, &Stereo::silence(v.len()), v.len(), None);
        let m = r.get("voice_minus_music_median_lu").unwrap();
        assert_eq!(m.pass, None, "{m:?}");
        assert!(r.passed(), "{:?}", r.failures());
    }

    #[test]
    fn a_note_held_into_a_clashing_chord_breaks_rule_7() {
        let v = speech(&[(1.0, 4.0)], 20.0, 0.1);
        let mk = |start: f64, end: f64, chord: crate::theory::Chord| ChordSpan {
            start,
            end,
            voicing: crate::theory::voice_lead(&chord, 4, None),
            chord,
        };
        let facts = |hold: f64| ScoreFacts {
            events: vec![Event {
                start: 10.0,
                hold,
                track: Track::Melody,
                inst: crate::instr::Instrument::FeltPiano,
                pitch: 66.0, // F♯4: fine over Bm9, a minor ninth under Gmaj9♯11's G
                velocity: 0.5,
                seed: 1,
                pan: 0.0,
            }],
            chords: vec![
                mk(0.0, 11.0, crate::theory::Chord::min9(11)),
                mk(11.0, 20.0, crate::theory::Chord::maj9_sharp11(7)),
            ],
            phrases: vec![],
        };
        let rule7 = |hold: f64| {
            evaluate("r7", &v, &Stereo::silence(v.len()), v.len(), Some(&facts(hold)))
                .get("rule7_minor_ninths")
                .unwrap()
                .pass
        };
        assert_eq!(rule7(0.9), Some(true), "released before the change");
        assert_eq!(rule7(2.0), Some(false), "held over the change");
    }

    #[test]
    fn detuned_sines_beat_and_a_steady_tone_does_not() {
        // v5's pad: sines detuned by 1.003 beat at 0.66 Hz; amplitudes 1 and 0.6
        // swing the envelope 1.6 / 0.4, a 12 dB tremolo
        let n = (30.0 * FS) as usize;
        let beat: Vec<f64> = (0..n)
            .map(|i| {
                let t = i as f64 / FS;
                0.1 * ((TAU * 220.0 * t).sin() + 0.6 * (TAU * 220.0 * 1.003 * t).sin())
            })
            .collect();
        let d = exposed_beating_db(&Stereo::from_mono(&beat), &[], &[]).unwrap();
        assert!(d > 6.0, "{d:.1} dB");
        // eight steady harmonics: no slow modulation at all
        let steady: Vec<f64> = (0..n)
            .map(|i| {
                let t = i as f64 / FS;
                (1..=8)
                    .map(|k| 0.02 / k as f64 * (TAU * 220.0 * k as f64 * t + k as f64).sin())
                    .sum()
            })
            .collect();
        let d = exposed_beating_db(&Stereo::from_mono(&steady), &[], &[]).unwrap();
        assert!(d < 1.0, "{d:.2} dB");
        // an onset 1 s before every window leaves nothing steady to measure
        let onsets: Vec<f64> = (0..30).map(|k| k as f64).collect();
        assert_eq!(exposed_beating_db(&Stereo::from_mono(&beat), &onsets, &[]), None);
    }

    #[test]
    fn length_and_finiteness_are_checked_first() {
        let v = speech(&[(1.0, 3.0)], 4.0, 0.1);
        let short = Stereo::silence(v.len() - 1);
        assert!(!evaluate("x", &v, &short, v.len(), None).passed());
        let mut nan = Stereo::silence(v.len());
        nan.l[5] = f64::NAN;
        let r = evaluate("x", &v, &nan, v.len(), None);
        assert_eq!(r.get("finite").unwrap().pass, Some(false));
    }

    #[test]
    fn a_click_is_detected_and_a_soft_tone_is_not() {
        let mut x = tone(440.0, 0.05, 2.0);
        assert_eq!(click_flags(&x), 0);
        x.l[48_000] += 0.5;
        x.r[48_000] += 0.5;
        assert!(click_flags(&x) >= 1);
    }

    #[test]
    fn out_of_phase_music_fails_mono_compatibility() {
        let mut x = tone(440.0, 0.1, 6.0);
        assert!(min_mono_correlation(&x) > 0.99);
        for v in &mut x.r {
            *v = -*v;
        }
        assert!(min_mono_correlation(&x) < -0.99);
    }

    #[test]
    fn a_loop_repeated_five_times_is_monotonous() {
        let mut rng = Rng::new(9);
        let w = (8.0 * FS) as usize;
        let one: Vec<f64> = (0..w).map(|_| 0.1 * rng.normal()).collect();
        let looped: Vec<f64> = one.iter().cycle().take(5 * w).copied().collect();
        assert_eq!(max_repeats(&Stereo::from_mono(&looped)), 4);
        let fresh: Vec<f64> = (0..5 * w).map(|_| 0.1 * rng.normal()).collect();
        assert_eq!(max_repeats(&Stereo::from_mono(&fresh)), 0);
    }

    #[test]
    fn score_rules_catch_planted_violations() {
        let total = 20.0;
        let v = speech(&[(1.0, 9.0), (11.0, 19.0)], total, 0.1);
        let music = Stereo::silence(v.len());
        let chord = crate::theory::Chord::maj9(2);
        let voicing = crate::theory::voice_lead(&chord, 4, None);
        let ev = |start: f64, hold: f64, track: Track, pitch: f64, seed: u64| Event {
            start,
            hold,
            track,
            inst: crate::instr::Instrument::FeltPiano,
            pitch,
            velocity: 0.5,
            seed,
            pan: 0.0,
        };
        let clean = ScoreFacts {
            events: vec![
                ev(9.5, 1.0, Track::Melody, 66.0, 1),
                ev(10.2, 1.0, Track::Melody, 69.0, 2),
            ],
            chords: vec![ChordSpan {
                start: 0.0,
                end: 12.0,
                chord: chord.clone(),
                voicing: voicing.clone(),
            }],
            phrases: vec![(9.5, 10.2)],
        };
        let r = evaluate("clean", &v, &music, v.len(), Some(&clean));
        for m in [
            "rule1_fundamentals_95_250hz_in_speech",
            "rule8_phrases_start_in_speech",
            "rule7_minor_ninths",
            "rule6_identical_notes_10s",
            "longest_chord_s",
        ] {
            assert_eq!(r.get(m).unwrap().pass, Some(true), "{m}: {:?}", r.get(m));
        }
        let bad = ScoreFacts {
            events: vec![
                // a 150 Hz sustained note under speech (rule 1)
                ev(2.0, 4.0, Track::Bed, 50.0, 3),
                // G over Dmaj9's F♯: a minor ninth (rule 7), starting in speech
                ev(3.0, 1.0, Track::Melody, 67.0, 4),
                // the same note twice within 10 s (rule 6)
                ev(9.5, 1.0, Track::Answer, 74.0, 5),
                ev(10.4, 1.0, Track::Answer, 74.0, 5),
            ],
            chords: vec![ChordSpan {
                start: 0.0,
                end: 19.0,
                chord,
                voicing,
            }],
            phrases: vec![(3.0, 3.0)],
        };
        let r = evaluate("bad", &v, &music, v.len(), Some(&bad));
        for m in [
            "rule1_fundamentals_95_250hz_in_speech",
            "rule8_phrases_start_in_speech",
            "rule7_minor_ninths",
            "rule6_identical_notes_10s",
            "longest_chord_s",
        ] {
            assert_eq!(r.get(m).unwrap().pass, Some(false), "{m}: {:?}", r.get(m));
        }
    }
}
