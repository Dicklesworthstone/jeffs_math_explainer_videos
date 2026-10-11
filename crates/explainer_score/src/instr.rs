//! Instruments. Each renders one note to a buffer; buses, effects, ducking and
//! mixing happen later (`score`). Recipes follow
//! docs/research/sound_design/MEMO.md §3.3: warm, analog-style voices with no
//! bare sine tones above 100 Hz and no free-free bar ratios (2.76, 5.4), whose
//! piercing partials land in the speech-intelligibility band.

use std::f64::consts::{PI, TAU};

use crate::filter::{AdaaTanh, DcBlock, Ladder, OnePole, Svf};
use crate::fx::{Adsr, DelayLine, rcos};
use crate::osc::{BlepOsc, Drift, Pink, Wave, White};
use crate::rng::Rng;
use crate::units::{cents_ratio, midi_to_hz};
use crate::{FS, Stereo};

/// One note: MIDI pitch (fractional allowed), velocity in [0, 1], duration
/// held in seconds, a per-note seed (round-robin variation) and a pan.
#[derive(Clone, Copy, Debug)]
pub struct Note {
    pub pitch: f64,
    pub velocity: f64,
    pub hold: f64,
    pub seed: u64,
    pub pan: f64,
}

impl Note {
    pub fn new(pitch: f64, velocity: f64, hold: f64, seed: u64) -> Self {
        Self {
            pitch,
            velocity,
            hold,
            seed,
            pan: 0.0,
        }
    }
    pub fn hz(&self) -> f64 {
        midi_to_hz(self.pitch)
    }
}

/// The instruments the score can call.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Instrument {
    /// Juno-style pad: two drifting saws and a PWM pulse into a
    /// ladder filter (chorus is applied on the bus).
    Pad {
        brightness: f64,
    },
    /// Solina-style strings: saw + octave saw, high- and low-passed (ensemble on the bus).
    Strings {
        brightness: f64,
    },
    /// Two-operator FM electric piano with a ratio-14 "tine" and tremolo.
    FmEp,
    /// Additive felt piano with stiff-string partials and a soft thump.
    FeltPiano,
    /// Modal mallets.
    Marimba,
    Vibraphone,
    Kalimba,
    /// A warm FM bell with harmonic ratios (never the 2.76/5.4 bar ratios).
    WarmGlass,
    /// Karplus–Strong plucked string (pizzicato).
    Pluck,
    /// Sine + triangle through gentle saturation and a low-pass at 2× f0.
    Bass,
}

/// Render `note` with `inst`; returns a stereo buffer (instrument-internal
/// stereo such as the EP's autopan; otherwise the note's pan). Every voice
/// leaves through [`shape_edges`]: the onset floor of hard rule 4 and a short
/// closing fade, so no recipe can click or strike too fast.
pub fn render(inst: Instrument, note: &Note) -> Stereo {
    let voice = |mono: Vec<f64>| crate::fx::pan(&mono, note.pan);
    let mut out = match inst {
        Instrument::Pad { brightness } => voice(pad(note, brightness)),
        Instrument::Strings { brightness } => voice(strings(note, brightness)),
        Instrument::FmEp => fm_ep(note),
        Instrument::FeltPiano => voice(felt_piano(note)),
        // the upper modes must ring long enough to be heard: a two-partial bar
        // is the sterile "test tone" hard rule 3 exists to prevent
        Instrument::Marimba => voice(modal(
            note,
            &[(1.0, 1.0, 0.55), (3.92, 0.22, 0.22), (9.2, 0.08, 0.09)],
            0.0018,
            None,
        )),
        Instrument::Vibraphone => voice(modal(
            note,
            &[(1.0, 1.0, 2.2), (4.0, 0.18, 0.55), (10.0, 0.05, 0.15)],
            0.0012,
            Some(4.0),
        )),
        // a tine's fundamental, a second harmonic from the bridge, and the
        // inharmonic first overtone (~5.3–6.8×) that gives it its plink
        Instrument::Kalimba => voice(modal(
            note,
            &[(1.0, 1.0, 0.9), (2.0, 0.07, 0.7), (5.9, 0.18, 0.12)],
            0.0009,
            None,
        )),
        Instrument::WarmGlass => voice(warm_glass(note)),
        Instrument::Pluck => voice(pluck(note)),
        Instrument::Bass => voice(bass(note)),
    };
    let onset = onset_floor(inst);
    shape_edges(&mut out.l, onset);
    shape_edges(&mut out.r, onset);
    out
}

/// Hard rule 4 (plan §7.3): the shortest onset (0 → full, raised cosine) an
/// instrument may have: 8 ms for plucked voices, 3 ms for everything else.
/// The recipes' own contact widths still set brightness; this sets the rise.
pub fn onset_floor(inst: Instrument) -> f64 {
    match inst {
        Instrument::Kalimba | Instrument::Pluck => 0.008,
        _ => 0.003,
    }
}

/// The closing fade every voice ends with (seconds).
pub const TAIL_FADE: f64 = 0.02;

/// Raised-cosine onset over `onset` seconds and a [`TAIL_FADE`] fade to an
/// exact zero at the last sample: a note buffer never starts or stops with a step.
pub fn shape_edges(x: &mut [f64], onset: f64) {
    let n_on = ((onset * FS) as usize).min(x.len());
    for (i, v) in x.iter_mut().enumerate().take(n_on) {
        *v *= rcos(i as f64 / n_on as f64);
    }
    let n_tail = ((TAIL_FADE * FS) as usize).min(x.len());
    let len = x.len();
    for (j, v) in x[len - n_tail..].iter_mut().enumerate() {
        *v *= rcos((n_tail - 1 - j) as f64 / n_tail as f64);
    }
}

/// A brightness in [0, 1] (non-finite input falls back to 0.5).
fn unit(b: f64) -> f64 {
    if b.is_finite() { b.clamp(0.0, 1.0) } else { 0.5 }
}

fn amp_from_velocity(v: f64) -> f64 {
    // A gentle curve: soft notes stay audible, loud ones don't jump out.
    0.15 + 0.85 * v.clamp(0.0, 1.0).powf(1.6)
}

fn pad(note: &Note, brightness: f64) -> Vec<f64> {
    let brightness = unit(brightness);
    let f0 = note.hz();
    // release τ 1 s: a chord change crossfades over ~3 s instead of smearing
    // the old chord across the next one for ~12 s
    let env = Adsr {
        attack: 1.1,
        decay: 2.0,
        sustain: 0.85,
        release: 1.0,
    }
    .render(note.hold);
    let fenv = Adsr {
        attack: 0.8,
        decay: 3.0,
        sustain: 0.6,
        release: 1.0,
    }
    .render(note.hold);
    let mut rng = Rng::new(note.seed);
    let detune = rng.range(5.0, 9.0);
    let mut o1 = BlepOsc::new(Wave::Saw, rng.uniform());
    let mut o2 = BlepOsc::new(Wave::Saw, rng.uniform());
    let mut pw = BlepOsc::new(Wave::Pulse(0.5), rng.uniform());
    let (mut d1, mut d2) = (
        Drift::new(rng.next_u64(), 3.0, 0.7),
        Drift::new(rng.next_u64(), 3.0, 0.7),
    );
    let mut lad = Ladder::new(3.0 * f0, 0.8, 1.2);
    let lfo_rate = rng.range(0.3, 0.6);
    let lfo_phase = rng.uniform();
    let amp = amp_from_velocity(note.velocity) * 0.22;
    let mut out = Vec::with_capacity(env.len());
    let (mut c1, mut c2, mut pw_dc) = (0.0, 0.0, 0.0);
    for (i, (&e, &fe)) in env.iter().zip(&fenv).enumerate() {
        if i % 32 == 0 {
            c1 = d1.step(32.0 / FS);
            c2 = d2.step(32.0 / FS);
            let fc = (2.5 + 1.5 * brightness) * f0 + 600.0 * fe * brightness;
            lad.set(fc.clamp(250.0, 2_000.0 + 4_000.0 * brightness), 0.8);
            let w = 0.5 + 0.15 * (TAU * (lfo_rate * i as f64 / FS + lfo_phase)).sin();
            pw.wave = Wave::Pulse(w);
            // a ±1 pulse of width w has mean 2w − 1: remove it, or the PWM
            // LFO becomes a slow DC wobble
            pw_dc = 2.0 * w - 1.0;
        }
        let a = o1.next(f0 * cents_ratio(detune * 0.5 + c1));
        let b = o2.next(f0 * cents_ratio(-detune * 0.5 + c2));
        let p = pw.next(f0 * cents_ratio(c1 * 0.5)) - pw_dc;
        // no sub-oscillator: an octave below a C4 voice is 131 Hz, inside the
        // narrator's pitch band (hard rule 1); the bass instrument owns the low end
        let x = 0.48 * (a + b) + 0.3 * p;
        out.push(amp * e * lad.process(x));
    }
    out
}

fn strings(note: &Note, brightness: f64) -> Vec<f64> {
    let brightness = unit(brightness);
    let f0 = note.hz();
    let env = Adsr {
        attack: 0.6,
        decay: 1.0,
        sustain: 0.9,
        release: 1.8,
    }
    .render(note.hold);
    let mut rng = Rng::new(note.seed);
    let mut o1 = BlepOsc::new(Wave::Saw, rng.uniform());
    let mut o2 = BlepOsc::new(Wave::Saw, rng.uniform());
    let mut drift = Drift::new(rng.next_u64(), 2.5, 0.6);
    let mut hp = Svf::new(200.0, 0.707);
    let mut lp = Svf::new(1_500.0 + 1_500.0 * brightness, 0.707);
    let amp = amp_from_velocity(note.velocity) * 0.2;
    let mut c = 0.0;
    env.iter()
        .enumerate()
        .map(|(i, &e)| {
            if i % 32 == 0 {
                c = drift.step(32.0 / FS);
            }
            let x = o1.next(f0 * cents_ratio(c)) + 0.6 * o2.next(2.0 * f0 * cents_ratio(-c));
            amp * e * lp.step(hp.step(x).high).low
        })
        .collect()
}

fn fm_ep(note: &Note) -> Stereo {
    let f0 = note.hz();
    let v = note.velocity.clamp(0.0, 1.0);
    let index_v = 0.5 + 0.7 * v; // ≤ 1.2 keeps it soft under speech
    let decay = 1.4 + 1.2 * (1.0 - (note.pitch - 48.0).max(0.0) / 48.0);
    let n = ((note.hold + decay * 2.5) * FS) as usize;
    let mut rng = Rng::new(note.seed);
    let (ph0, trem_ph) = (rng.uniform() * TAU, rng.uniform());
    let mut sat = AdaaTanh::default();
    let mut dc = DcBlock::default();
    let pan = note.pan.clamp(-1.0, 1.0);
    let amp = amp_from_velocity(v) * 0.32;
    let mut s = Stereo::silence(n);
    for i in 0..n {
        let t = i as f64 / FS;
        let rel = if t > note.hold {
            (-(t - note.hold) / 0.25).exp()
        } else {
            1.0
        };
        let a = (t / 0.004).min(1.0) * (-t / decay).exp() * rel;
        let idx = index_v * (0.15 + 0.85 * (-t / 0.25).exp());
        let m = (TAU * f0 * t + ph0).sin();
        let body = (TAU * f0 * t + idx * m).sin();
        let tine = 0.12 * (-t / 0.02).exp() * (TAU * f0 * t + 1.5 * (TAU * 14.0 * f0 * t).sin()).sin();
        // a touch of asymmetric "bark" from the pickup; the asymmetry leaves a
        // DC offset that the blocker removes
        let y = dc.process(sat.process(1.2 * (body + tine) + 0.08));
        let x = amp * a * y;
        // constant-power position (the same law as `fx::pan`): the note's pan
        // plus the tremolo's autopan
        let p = (pan + 0.25 * (TAU * (4.5 * t + trem_ph)).sin()).clamp(-1.0, 1.0);
        let angle = (p + 1.0) * 0.25 * PI;
        s.l[i] = x * angle.cos();
        s.r[i] = x * angle.sin();
    }
    s
}

fn felt_piano(note: &Note) -> Vec<f64> {
    let f0 = note.hz();
    let v = note.velocity.clamp(0.0, 1.0);
    let b = 2.0e-4;
    let felt = 300.0 + 600.0 * v;
    let tau1 = 2.8 * (1.0 + (60.0 - note.pitch).max(0.0) / 24.0);
    let n = ((note.hold.max(0.3) + tau1 * 1.2) * FS) as usize;
    let mut rng = Rng::new(note.seed);
    let mut partials = Vec::new();
    let mut k = 1.0;
    loop {
        let f = k * f0 * (1.0 + b * k * k).sqrt();
        if f > 7_000.0 || k > 24.0 {
            break;
        }
        let amp = k.powf(-1.2) / (1.0 + (k * f0 / felt).powi(2));
        let tau = tau1 / (1.0 + 0.25 * (k - 1.0));
        partials.push((f, amp, tau, rng.uniform() * TAU, rng.uniform() * TAU));
        k += 1.0;
    }
    let norm: f64 = partials.iter().map(|p| p.1).sum::<f64>().max(1e-9);
    let detune = cents_ratio(0.7);
    let mut white = White::new(rng.next_u64());
    let mut thump_lp = OnePole::new(400.0);
    let attack_s = 0.008 + 0.007 * (1.0 - v);
    let amp = amp_from_velocity(v) * 0.5 / norm;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f64 / FS;
        let att = rcos(t / attack_s);
        let rel = if t > note.hold {
            (-(t - note.hold) / 0.35).exp()
        } else {
            1.0
        };
        let mut y = 0.0;
        for &(f, a, tau, p1, p2) in &partials {
            let env = (-t / tau).exp();
            // two strings per note, ±0.7 cent: the double decay of a real piano
            y += a * env * 0.5 * ((TAU * f * detune * t + p1).sin() + (TAU * f / detune * t + p2).sin());
        }
        let thump = if t < 0.02 {
            thump_lp.lp(white.sample()) * 0.025 * (1.0 - t / 0.02)
        } else {
            0.0
        };
        out.push(amp * att * rel * y + thump * amp * norm);
    }
    out
}

/// Modal synthesis: each mode `(ratio, amplitude, decay seconds)` is a damped
/// sinusoid excited by a raised-cosine contact of `contact` seconds; optional
/// vibraphone motor tremolo (Hz) acting mainly on the fundamental.
fn modal(note: &Note, modes: &[(f64, f64, f64)], contact: f64, tremolo: Option<f64>) -> Vec<f64> {
    let f0 = note.hz();
    let v = note.velocity.clamp(0.0, 1.0);
    let longest = modes.iter().map(|m| m.2).fold(0.0, f64::max);
    let n = ((note.hold.max(0.05) + longest * 5.0) * FS) as usize;
    let mut rng = Rng::new(note.seed);
    // ±15% hardness and ±10% decay variation per strike (round-robin feel)
    let contact = contact * (1.0 + rng.jitter(0.15));
    let decay_scale = 1.0 + rng.jitter(0.10);
    let trem_phase = rng.uniform();
    let amp = amp_from_velocity(v) * 0.6 / modes.iter().map(|m| m.1).sum::<f64>();
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f64 / FS;
        let att = rcos(t / contact.max(1e-4));
        let damp = if t > note.hold + 0.05 {
            (-(t - note.hold - 0.05) / 0.4).exp()
        } else {
            1.0
        };
        let mut y = 0.0;
        for (k, &(ratio, a, tau)) in modes.iter().enumerate() {
            let f = f0 * ratio;
            if f > 18_000.0 {
                continue;
            }
            // harder (shorter) contact excites upper modes more
            let bright = if k == 0 { 1.0 } else { (0.0015 / contact).min(1.5) };
            let mut m = a * bright * (-t / (tau * decay_scale)).exp() * (TAU * f * t).sin();
            if let (Some(rate), 0) = (tremolo, k) {
                m *= 1.0 - 0.5 * (0.5 + 0.5 * (TAU * (rate * t + trem_phase)).sin());
            } else if let Some(rate) = tremolo {
                m *= 1.0 - 0.15 * (0.5 + 0.5 * (TAU * (rate * t + trem_phase)).sin());
            }
            y += m;
        }
        out.push(amp * att * damp * y);
    }
    out
}

fn warm_glass(note: &Note) -> Vec<f64> {
    // FM bell with harmonic ratios (1:1 and 1:2), low index, softened top. It
    // rings freely (τ ≈ 1.4 s) for `hold`, then damps (τ 0.6 s); the buffer
    // runs six damping constants past `hold`, so it ends > 50 dB down.
    let f0 = note.hz();
    let v = note.velocity.clamp(0.0, 1.0);
    let mut rng = Rng::new(note.seed);
    // round-robin variation (rule 6): ±10% decay, ±15% index
    let tau = 1.4 * (1.0 + rng.jitter(0.10));
    let index = 0.9 * (1.0 + rng.jitter(0.15));
    let hold = note.hold.max(0.1);
    let n = ((hold + 6.0 * 0.6) * FS) as usize;
    let mut lp = Svf::new(2_200.0 + 1_500.0 * v, 0.6);
    let amp = amp_from_velocity(v) * 0.35;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f64 / FS;
        let rel = if t > hold { (-(t - hold) / 0.6).exp() } else { 1.0 };
        let env = (-t / tau).exp() * rel;
        let idx = index * (-t / 0.5).exp() + 0.2;
        let m1 = (TAU * f0 * t).sin();
        let m2 = (TAU * 2.0 * f0 * t).sin();
        let y = (TAU * f0 * t + idx * m1).sin() + 0.35 * (TAU * 2.0 * f0 * t + 0.6 * idx * m2).sin();
        out.push(amp * env * lp.step(y).low);
    }
    out
}

/// Karplus–Strong with first-order allpass fractional-delay tuning, a
/// two-point averaging loop filter and a pick-position comb.
fn pluck(note: &Note) -> Vec<f64> {
    let f0 = note.hz();
    let v = note.velocity.clamp(0.0, 1.0);
    // loop delay = N + 0.5 (averaging filter) + allpass delay Δ
    let total = FS / f0;
    let n_int = (total - 0.5).floor() as usize;
    let mut delta = total - 0.5 - n_int as f64;
    if delta < 0.1 {
        // keep the allpass coefficient well-behaved
        delta += 1.0;
    }
    let n_int = (total - 0.5 - delta).round() as usize;
    let c = (1.0 - delta) / (1.0 + delta);
    let rho = 0.996;
    let len = ((note.hold.max(0.2) + 2.5) * FS) as usize;
    let mut line = DelayLine::new(n_int + 4);
    let mut rng = Rng::new(note.seed);
    let mut white = White::new(rng.next_u64());
    // excitation: 1.5 ms raised-cosine pluck plus a little noise, then a
    // pick-position comb (β ≈ 0.15) to shape the harmonic series
    let exc_len = (0.0015 * FS) as usize;
    let beta = (0.15 * n_int as f64).round().max(1.0) as usize;
    // the finger's softness: the noisy pulse through two one-pole low-passes
    // (2.5 kHz soft to 5.5 kHz hard), so a pluck's first period carries no
    // hiss above 8 kHz (QA's click detector heard the raw noise on every
    // pluck in a listen window)
    let (mut lp1, mut lp2) = (OnePole::new(2_500.0 + 3_000.0 * v), OnePole::new(2_500.0 + 3_000.0 * v));
    let mut exc: Vec<f64> = (0..n_int.max(exc_len))
        .map(|i| {
            let w = if i < exc_len {
                (PI * i as f64 / exc_len as f64).sin()
            } else {
                0.0
            };
            lp2.lp(lp1.lp(w * (0.8 + 0.4 * white.sample())))
        })
        .collect();
    for i in (beta..exc.len()).rev() {
        exc[i] -= exc[i - beta];
    }
    let (mut ap_x1, mut ap_y1, mut prev) = (0.0, 0.0, 0.0);
    // the comb-differenced excitation can reach ±2.4, so keep the gain modest
    let amp = amp_from_velocity(v) * 0.3;
    let mut out = Vec::with_capacity(len);
    for i in 0..len {
        let input = if i < exc.len() { exc[i] } else { 0.0 };
        let delayed = line.read(n_int as f64);
        // loop filter: ρ·(x[n] + x[n−1])/2
        let avg = rho * 0.5 * (delayed + prev);
        prev = delayed;
        // first-order allpass: y = c·x + x1 − c·y1
        let ap = c * avg + ap_x1 - c * ap_y1;
        ap_x1 = avg;
        ap_y1 = ap;
        let y = input + ap;
        line.push(y);
        let damp = if i as f64 / FS > note.hold {
            (-(i as f64 / FS - note.hold) / 0.15).exp()
        } else {
            1.0
        };
        out.push(amp * damp * delayed);
    }
    out
}

fn bass(note: &Note) -> Vec<f64> {
    let f0 = note.hz();
    let env = Adsr {
        attack: 0.015,
        decay: 0.6,
        sustain: 0.8,
        release: 0.4,
    }
    .render(note.hold);
    let mut rng = Rng::new(note.seed);
    let mut s = BlepOsc::new(Wave::Sine, rng.uniform());
    let mut t = BlepOsc::new(Wave::Triangle, rng.uniform());
    let mut sat = AdaaTanh::default();
    // the low-pass tracks the note (2× f0) so the saturation's and the
    // triangle's harmonics stay out of the narrator's 95–250 Hz band: the
    // intent of hard rule 1, not just its letter (fundamentals)
    let mut lp = Svf::new((2.0 * f0).clamp(60.0, 350.0), 0.707);
    let amp = amp_from_velocity(note.velocity) * 0.5;
    env.iter()
        .map(|&e| {
            let x = s.next(f0) + 0.25 * t.next(f0);
            amp * e * lp.step(sat.process(0.8 * x) / 0.8).low
        })
        .collect()
}

// ----------------------------------------------------------------- sound effects

/// Noise whoosh: pink noise through a band-pass whose centre sweeps
/// logarithmically `f_start → f_peak → f_end`; independent seeds per channel.
pub fn whoosh(seconds: f64, f_start: f64, f_peak: f64, f_end: f64, seed: u64) -> Stereo {
    let n = (seconds * FS) as usize;
    let mut out = Stereo::silence(n);
    for (ch, buf) in [&mut out.l, &mut out.r].into_iter().enumerate() {
        let mut p = Pink::new(seed ^ (ch as u64 + 1));
        let mut bp = Svf::new(f_start, 1.2);
        for (i, y) in buf.iter_mut().enumerate() {
            let u = i as f64 / n.max(1) as f64;
            let fc = if u < 0.5 {
                f_start * (f_peak / f_start).powf(u * 2.0)
            } else {
                f_peak * (f_end / f_peak).powf((u - 0.5) * 2.0)
            };
            if i % 32 == 0 {
                bp.set(fc, 1.2);
            }
            let env = (PI * u).sin().powi(2);
            *y = 0.5 * env * bp.step(p.sample()).band;
        }
    }
    out
}

/// A rising band-passed noise swell that ends `seconds` after it starts.
pub fn riser(seconds: f64, seed: u64) -> Stereo {
    let mut s = whoosh(seconds, 300.0, 1_400.0, 2_400.0, seed);
    let n = s.len();
    for i in 0..n {
        let u = i as f64 / n as f64;
        let g = u * u / (0.15 + (PI * u).sin().powi(2)).max(0.15);
        s.l[i] *= g;
        s.r[i] *= g;
    }
    s
}

/// A bubble: a damped sinusoid at the Minnaert frequency f ≈ 3.26 / r (r in
/// metres) gliding slightly upward.
pub fn bubble(radius_m: f64, seed: u64) -> Vec<f64> {
    let f = 3.26 / radius_m.clamp(0.002, 0.02);
    let mut rng = Rng::new(seed);
    let glide = 1.0 + rng.range(0.1, 0.25);
    let n = (0.12 * FS) as usize;
    let mut ph = 0.0;
    let mut out: Vec<f64> = (0..n)
        .map(|i| {
            let t = i as f64 / FS;
            let fi = f * (1.0 + (glide - 1.0) * (t / 0.12));
            ph += TAU * fi / FS;
            0.4 * (-t / 0.025).exp() * ph.sin()
        })
        .collect();
    shape_edges(&mut out, 0.004);
    out
}

/// A soft wooden "tok" for counters: two modes, 20–40 ms, low-passed < 3 kHz.
pub fn tick(pitch_hz: f64, seed: u64) -> Vec<f64> {
    let mut rng = Rng::new(seed);
    let f = pitch_hz.clamp(500.0, 1_000.0) * (1.0 + rng.jitter(0.02));
    let n = (0.04 * FS) as usize;
    let mut lp = Svf::new(2_800.0, 0.6);
    let mut out: Vec<f64> = (0..n)
        .map(|i| {
            let t = i as f64 / FS;
            let y = (-t / 0.008).exp() * (TAU * f * t).sin() + 0.4 * (-t / 0.004).exp() * (TAU * 2.3 * f * t).sin();
            0.35 * lp.step(y).low
        })
        .collect();
    // a deliberate tick may start in 1 ms: it is low-passed under 3 kHz
    shape_edges(&mut out, 0.001);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [Instrument; 10] = [
        Instrument::Pad { brightness: 0.5 },
        Instrument::Strings { brightness: 0.5 },
        Instrument::FmEp,
        Instrument::FeltPiano,
        Instrument::Marimba,
        Instrument::Vibraphone,
        Instrument::Kalimba,
        Instrument::WarmGlass,
        Instrument::Pluck,
        Instrument::Bass,
    ];

    /// Fundamental by normalized autocorrelation with parabolic refinement.
    fn pitch_of(x: &[f64], lo: f64, hi: f64) -> f64 {
        let (min_lag, max_lag) = ((FS / hi) as usize, (FS / lo) as usize);
        let seg = &x[..x.len().min(24_000)];
        let mut best = (0, f64::MIN);
        let r: Vec<f64> = (0..=max_lag + 1)
            .map(|lag| seg.iter().zip(&seg[lag..]).map(|(a, b)| a * b).sum::<f64>())
            .collect();
        for lag in min_lag..=max_lag {
            if r[lag] > best.1 && r[lag] > r[lag - 1] && r[lag] >= r[lag + 1] {
                best = (lag, r[lag]);
            }
        }
        let l = best.0;
        let (a, b, c) = (r[l - 1], r[l], r[l + 1]);
        let shift = 0.5 * (a - c) / (a - 2.0 * b + c);
        FS / (l as f64 + shift)
    }

    #[test]
    fn every_instrument_is_finite_and_within_its_peak_budget() {
        for inst in ALL {
            for pitch in [40.0, 57.0, 69.0, 81.0] {
                let s = render(inst, &Note::new(pitch, 1.0, 1.5, 3));
                assert!(s.is_finite(), "{inst:?} {pitch}");
                let pk = s.peak();
                assert!(pk <= 0.891, "{inst:?} at {pitch}: peak {pk:.3} > −1 dBFS");
                assert!(pk > 1e-3, "{inst:?} at {pitch}: silent");
            }
        }
    }

    #[test]
    fn karplus_strong_is_in_tune_within_two_cents() {
        for pitch in [45.0, 52.0, 57.0, 64.0, 69.0] {
            let x = render(Instrument::Pluck, &Note::new(pitch, 0.8, 1.0, 1)).l;
            let f = pitch_of(&x[2_000..], midi_to_hz(pitch) * 0.8, midi_to_hz(pitch) * 1.25);
            let cents = 1_200.0 * (f / midi_to_hz(pitch)).log2();
            assert!(cents.abs() < 2.0, "pitch {pitch}: {cents:.2} cents off");
        }
    }

    #[test]
    fn instruments_track_their_pitch() {
        for inst in [
            Instrument::Pad { brightness: 0.4 },
            Instrument::FeltPiano,
            Instrument::FmEp,
            Instrument::Marimba,
        ] {
            let x = render(inst, &Note::new(57.0, 0.7, 1.0, 2)).l;
            let f = pitch_of(&x[12_000..], 150.0, 300.0);
            let cents = 1_200.0 * (f / midi_to_hz(57.0)).log2();
            assert!(
                cents.abs() < 15.0,
                "{inst:?}: {cents:.1} cents (drift and stiffness allowed, not wrong notes)"
            );
        }
    }

    /// Magnitude (dB re max) at `f` from a single-frequency DFT over `x`.
    fn level_at(x: &[f64], f: f64) -> f64 {
        let (mut re, mut im) = (0.0, 0.0);
        for (i, v) in x.iter().enumerate() {
            let w = 0.5 - 0.5 * (TAU * i as f64 / x.len() as f64).cos();
            let a = TAU * f * i as f64 / FS;
            re += v * w * a.cos();
            im -= v * w * a.sin();
        }
        (re * re + im * im).sqrt()
    }

    #[test]
    fn modal_ratios_are_where_the_recipes_put_them() {
        // marimba 1 : 3.92 : 9.2 at A2 (110 Hz): each mode is a local peak.
        let x = render(Instrument::Marimba, &Note::new(45.0, 1.0, 0.5, 4)).l;
        let seg = &x[0..16_384];
        for ratio in [1.0, 3.92] {
            let f = 110.0 * ratio;
            let at = level_at(seg, f);
            let off = level_at(seg, f * 1.04).max(level_at(seg, f * 0.96));
            assert!(at > 3.0 * off, "mode {ratio}: {at:.3} vs neighbours {off:.3}");
        }
    }

    #[test]
    fn modal_frequencies_are_within_half_a_percent_of_the_recipe() {
        // Scan ±3% around each mode in 0.02% steps for the windowed-DFT peak.
        // Each mode is a decaying sinusoid at exactly f0·ratio, so the peak sits
        // there unless another mode or the window bias pulls it away.
        for (inst, ratios) in [
            (Instrument::Marimba, vec![1.0, 3.92, 9.2]),
            (Instrument::Vibraphone, vec![1.0, 4.0, 10.0]),
            (Instrument::Kalimba, vec![1.0, 2.0, 5.9]),
        ] {
            let x = render(inst, &Note::new(57.0, 1.0, 0.5, 4)).l;
            let seg = &x[480..480 + 16_384];
            let f0 = midi_to_hz(57.0);
            for r in ratios {
                let target = f0 * r;
                let best = (-150..=150)
                    .map(|k| target * (1.0 + f64::from(k) * 2e-4))
                    .max_by(|a, b| level_at(seg, *a).total_cmp(&level_at(seg, *b)))
                    .unwrap();
                let err = (best / target - 1.0) * 100.0;
                assert!(err.abs() < 0.5, "{inst:?} mode {r}: peak {best:.1} Hz, {err:+.3}%");
            }
        }
    }

    #[test]
    fn no_bare_sines_above_100_hz() {
        // Hard rule 3 (plan §7.3): at least three partials above −40 dB.
        for inst in ALL.iter().filter(|i| !matches!(i, Instrument::Bass)) {
            let x = render(*inst, &Note::new(57.0, 0.8, 1.0, 5)).l;
            let seg = &x[2_400..2_400 + 16_384];
            let f0 = midi_to_hz(57.0);
            let fund = (1..=16).map(|k| level_at(seg, f0 * k as f64)).fold(0.0, f64::max);
            let strong = (1..=16)
                .map(|k| level_at(seg, f0 * k as f64))
                .chain([3.92, 9.2, 4.0, 10.0, 5.9].iter().map(|r| level_at(seg, f0 * r)))
                .filter(|&l| l > fund * 0.01)
                .count();
            assert!(strong >= 3, "{inst:?}: only {strong} partials above −40 dB");
        }
    }

    #[test]
    fn mallet_tails_keep_more_than_one_partial() {
        // Hard rule 3 in the tail, 0.5 s after the strike: a struck voice must
        // not decay into a bare sine. (The colour bus adds chorus and plate on
        // top; this checks the dry voice keeps at least two partials.)
        for (inst, ratios) in [
            (Instrument::Marimba, [1.0, 3.92]),
            (Instrument::Vibraphone, [1.0, 4.0]),
            (Instrument::Kalimba, [1.0, 2.0]),
        ] {
            let x = render(inst, &Note::new(57.0, 0.8, 1.0, 5)).l;
            let seg = &x[24_000..24_000 + 16_384];
            let f0 = midi_to_hz(57.0);
            let lv: Vec<f64> = ratios.iter().map(|r| level_at(seg, f0 * r)).collect();
            assert!(
                lv[1] > 0.01 * lv[0],
                "{inst:?}: second partial {:.1} dB",
                20.0 * (lv[1] / lv[0]).log10()
            );
        }
    }

    #[test]
    fn onsets_respect_hard_rule_4() {
        // Within the first half of its onset floor a note stays under ~half of
        // its peak (a raised cosine is exactly 0.5 there). Kalimba and pluck
        // must not reach it before 4 ms, mallets not before 1.5 ms.
        for inst in ALL {
            for pitch in [45.0, 57.0, 69.0, 81.0] {
                let s = render(inst, &Note::new(pitch, 1.0, 0.8, 7));
                let k = (onset_floor(inst) * 0.5 * FS) as usize;
                let early = s.l[..k].iter().chain(&s.r[..k]).fold(0.0_f64, |m, v| m.max(v.abs()));
                assert!(
                    early <= 0.55 * s.peak(),
                    "{inst:?} at {pitch}: {early:.3} within {k} samples (peak {:.3})",
                    s.peak()
                );
            }
        }
        assert!(onset_floor(Instrument::Pluck) >= 0.008 && onset_floor(Instrument::Kalimba) >= 0.008);
        assert!(ALL.iter().all(|&i| onset_floor(i) >= 0.003));
    }

    #[test]
    fn every_note_ends_in_silence_without_a_cut() {
        // The closing fade reaches an exact zero, and it is not hiding a hard
        // cut: just before the fade the note is already 40 dB under its peak.
        let fade = (TAIL_FADE * FS) as usize;
        for inst in ALL {
            for hold in [0.1, 1.5] {
                let s = render(inst, &Note::new(60.0, 0.9, hold, 9));
                let n = s.len();
                assert_eq!((s.l[n - 1], s.r[n - 1]), (0.0, 0.0), "{inst:?}");
                let before = s.l[n - fade - 2_400..n - fade]
                    .iter()
                    .chain(&s.r[n - fade - 2_400..n - fade])
                    .fold(0.0_f64, |m, v| m.max(v.abs()));
                assert!(
                    before < 0.01 * s.peak(),
                    "{inst:?} hold {hold}: {:.1} dB re peak before the fade",
                    20.0 * (before / s.peak()).log10()
                );
            }
        }
        for m in [bubble(0.005, 3), tick(700.0, 4)] {
            assert_eq!(m[0], 0.0);
            assert_eq!(*m.last().unwrap(), 0.0);
        }
    }

    #[test]
    fn bass_keeps_out_of_the_narrators_band() {
        // Hard rule 1's intent: E1–E♭2 bass notes put ≥ 20 dB less energy in
        // 95–250 Hz than in total (harmonics included, not only fundamentals).
        for pitch in [28.0, 33.0, 39.0] {
            let x = render(Instrument::Bass, &Note::new(pitch, 1.0, 2.0, 1)).l;
            let (df, psd) = crate::fft::welch(&x[4_800..96_000], FS, 8_192);
            let band = |lo: f64, hi: f64| -> f64 {
                psd.iter()
                    .enumerate()
                    .filter(|(k, _)| (lo..hi).contains(&(*k as f64 * df)))
                    .map(|(_, p)| p)
                    .sum()
            };
            let ratio = 10.0 * (band(95.0, 250.0) / band(0.0, 24_000.0)).log10();
            assert!(ratio < -20.0, "pitch {pitch}: {ratio:.1} dB in 95–250 Hz");
        }
    }

    #[test]
    fn fm_ep_pan_moves_the_image_at_constant_power() {
        let energy = |pan: f64| {
            let mut n = Note::new(64.0, 0.7, 1.0, 3);
            n.pan = pan;
            let s = render(Instrument::FmEp, &n);
            let e = |v: &[f64]| v.iter().map(|x| x * x).sum::<f64>();
            (e(&s.l), e(&s.r))
        };
        let (l0, r0) = energy(0.0);
        let (lp, rp) = energy(1.0);
        let (lm, rm) = energy(-1.0);
        assert!((10.0 * (l0 / r0).log10()).abs() < 1.0, "centre is balanced");
        assert!(10.0 * (rp / lp).log10() > 6.0, "pan +1 leans right");
        assert!(10.0 * (lm / rm).log10() > 6.0, "pan −1 leans left");
        for (l, r) in [(lp, rp), (lm, rm)] {
            assert!((10.0 * ((l + r) / (l0 + r0)).log10()).abs() < 0.5, "constant power");
        }
    }

    #[test]
    fn same_seed_same_note_different_seed_different_note() {
        for inst in [Instrument::Vibraphone, Instrument::WarmGlass, Instrument::FmEp] {
            let a = render(inst, &Note::new(64.0, 0.7, 0.5, 10));
            let b = render(inst, &Note::new(64.0, 0.7, 0.5, 10));
            let c = render(inst, &Note::new(64.0, 0.7, 0.5, 11));
            assert_eq!(a, b, "{inst:?}");
            assert_ne!(a, c, "{inst:?} (rule 6: no identical repeats)");
        }
    }

    #[test]
    fn bad_brightness_does_not_panic() {
        for b in [-3.0, f64::NAN, 7.0] {
            assert!(render(Instrument::Pad { brightness: b }, &Note::new(60.0, 0.5, 0.3, 1)).is_finite());
            assert!(render(Instrument::Strings { brightness: b }, &Note::new(60.0, 0.5, 0.3, 1)).is_finite());
        }
    }

    #[test]
    fn effects_are_finite_and_soft() {
        for s in [whoosh(1.2, 250.0, 1_600.0, 400.0, 1), riser(2.0, 2)] {
            assert!(s.is_finite() && s.peak() < 0.9 && s.peak() > 1e-3);
        }
        for m in [bubble(0.005, 3), tick(700.0, 4)] {
            assert!(m.iter().all(|v| v.is_finite()) && m.iter().fold(0.0_f64, |a, b| a.max(b.abs())) < 0.9);
        }
    }
}
