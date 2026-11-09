//! Envelopes and effects: exponential ADSR, raised-cosine ramps, breakpoint
//! automation, a fractional delay line, asymmetric "warmth" saturation, Juno-
//! and Solina-style chorus, tape (wow, flutter, head bump, high-frequency
//! shelf), an 8-line feedback-delay-network reverb with early reflections, a
//! Dattorro plate, a ping-pong delay, mid/side and constant-power panning.
//!
//! Parameters follow docs/research/sound_design/MEMO.md §3.3–3.4.

use std::f64::consts::{FRAC_PI_2, TAU};

use crate::filter::{AdaaTanh, Biquad, DcBlock, Kind, OnePole};
use crate::rng::Rng;
use crate::{FS, Stereo};

// ----------------------------------------------------------------- envelopes

/// Exponential ADSR (one-pole segments, the shape real circuits make).
/// Times in seconds; `sustain` in [0, 1].
#[derive(Clone, Copy, Debug)]
pub struct Adsr {
    pub attack: f64,
    pub decay: f64,
    pub sustain: f64,
    pub release: f64,
}

impl Adsr {
    /// Render the envelope for a note held `hold` seconds, followed by the
    /// release; length = (hold + release·5) seconds (until ≈ −43 dB).
    pub fn render(&self, hold: f64) -> Vec<f64> {
        let n_hold = (hold.max(0.0) * FS) as usize;
        let n_rel = (self.release.max(1e-3) * 5.0 * FS) as usize;
        let mut out = Vec::with_capacity(n_hold + n_rel);
        let (ka, kd, kr) = (
            1.0 - (-1.0 / (self.attack.max(1e-4) * FS)).exp(),
            1.0 - (-1.0 / (self.decay.max(1e-4) * FS)).exp(),
            1.0 - (-1.0 / (self.release.max(1e-4) * FS)).exp(),
        );
        let mut y = 0.0;
        let mut attacking = true;
        for _ in 0..n_hold {
            if attacking {
                // aim above 1 so the attack reaches 1 in finite time
                y += (1.25 - y) * ka;
                if y >= 1.0 {
                    y = 1.0;
                    attacking = false;
                }
            } else {
                y += (self.sustain - y) * kd;
            }
            out.push(y);
        }
        for _ in 0..n_rel {
            y += (0.0 - y) * kr;
            out.push(y);
        }
        out
    }
}

/// Raised-cosine ramp from 0 to 1 over [0, 1].
#[inline]
pub fn rcos(u: f64) -> f64 {
    0.5 - 0.5 * (std::f64::consts::PI * u.clamp(0.0, 1.0)).cos()
}

/// Breakpoint automation: `(seconds, value)` points joined by raised-cosine
/// segments (smooth at every breakpoint), constant before the first point and
/// after the last. Drives slow musical moves: intensity, cutoff, sends.
#[derive(Clone, Debug, PartialEq)]
pub struct Automation {
    points: Vec<(f64, f64)>,
}

impl Automation {
    /// Points are sorted by time; an empty list is a constant 0.
    pub fn new(points: &[(f64, f64)]) -> Self {
        let mut points = points.to_vec();
        points.sort_by(|a, b| a.0.total_cmp(&b.0));
        Self { points }
    }

    /// The value at `t` seconds.
    pub fn at(&self, t: f64) -> f64 {
        let Some(&(t0, v0)) = self.points.first() else {
            return 0.0;
        };
        if t <= t0 {
            return v0;
        }
        for w in self.points.windows(2) {
            let ((ta, va), (tb, vb)) = (w[0], w[1]);
            if t <= tb {
                let u = if tb > ta { (t - ta) / (tb - ta) } else { 1.0 };
                return va + (vb - va) * rcos(u);
            }
        }
        self.points[self.points.len() - 1].1
    }

    /// One value per sample for `n` samples at 48 kHz.
    pub fn render(&self, n: usize) -> Vec<f64> {
        (0..n).map(|i| self.at(i as f64 / FS)).collect()
    }
}

// ----------------------------------------------------------------- saturation

/// Asymmetric soft saturation, the even-harmonic "warmth" of a biased tube or
/// tape stage: first-order ADAA tanh of `drive·x + bias`, minus the bias's
/// static offset, scaled to unity small-signal gain, then a DC blocker (the
/// asymmetry rectifies a little of every signal into DC).
#[derive(Clone, Debug)]
pub struct Warmth {
    sat: AdaaTanh,
    dc: DcBlock,
    drive: f64,
    bias: f64,
    norm: f64,
}

impl Warmth {
    pub fn new(drive: f64, bias: f64) -> Self {
        let drive = drive.max(1e-3);
        let slope = 1.0 - bias.tanh().powi(2);
        Self {
            sat: AdaaTanh::default(),
            dc: DcBlock::default(),
            drive,
            bias,
            norm: 1.0 / (drive * slope),
        }
    }

    #[inline]
    pub fn process(&mut self, x: f64) -> f64 {
        let y = self.sat.process(self.drive * x + self.bias) - self.bias.tanh();
        self.dc.process(y * self.norm)
    }
}

// ----------------------------------------------------------------- delay line

/// A delay line with cubic (Hermite) fractional reads.
#[derive(Clone, Debug)]
pub struct DelayLine {
    buf: Vec<f64>,
    mask: usize,
    w: usize,
}

impl DelayLine {
    /// Capacity at least `max_samples` (rounded up to a power of two).
    pub fn new(max_samples: usize) -> Self {
        let cap = (max_samples + 4).next_power_of_two();
        Self {
            buf: vec![0.0; cap],
            mask: cap - 1,
            w: 0,
        }
    }
    #[inline]
    pub fn push(&mut self, x: f64) {
        self.buf[self.w] = x;
        self.w = (self.w + 1) & self.mask;
    }
    /// The longest delay [`read`](Self::read) can return.
    pub fn max_delay(&self) -> f64 {
        (self.mask - 2) as f64
    }

    /// Read the value written `d` pushes ago (`read(1)` is the latest write;
    /// fractional `d` interpolates). Read before pushing the current input for
    /// a delay of exactly `d` samples. `d` must lie in [1, `max_delay`]; it is
    /// clamped there (and checked in debug builds), never wrapped.
    #[inline]
    pub fn read(&self, d: f64) -> f64 {
        debug_assert!(d <= self.max_delay(), "delay {d} exceeds capacity {}", self.max_delay());
        let d = d.clamp(1.0, self.max_delay());
        let i = d.floor() as usize;
        let f = d - i as f64;
        let at = |k: usize| self.buf[(self.w + self.mask + 1 - k) & self.mask];
        let (xm1, x0, x1, x2) = (at(i - 1 + (i == 1) as usize), at(i), at(i + 1), at(i + 2));
        // Hermite between x0 (delay i) and x1 (delay i+1).
        let c0 = x0;
        let c1 = 0.5 * (x1 - xm1);
        let c2 = xm1 - 2.5 * x0 + 2.0 * x1 - 0.5 * x2;
        let c3 = 0.5 * (x2 - xm1) + 1.5 * (x0 - x1);
        ((c3 * f + c2) * f + c1) * f + c0
    }
}

// ----------------------------------------------------------------- chorus

/// Juno-60-style stereo chorus: two bucket-brigade-like taps modulated by
/// opposite-phase triangle LFOs, wet path low-passed at 8 kHz.
#[derive(Clone, Debug)]
pub struct Chorus {
    line: DelayLine,
    phase: f64,
    rate: f64,
    center_ms: f64,
    depth_ms: f64,
    mix: f64,
    lp_l: OnePole,
    lp_r: OnePole,
}

impl Chorus {
    /// Mode I ≈ 0.5 Hz; mode II ≈ 0.83 Hz.
    pub fn juno(rate_hz: f64, mix: f64) -> Self {
        Self {
            line: DelayLine::new((0.02 * FS) as usize),
            phase: 0.0,
            rate: rate_hz,
            center_ms: 3.5,
            depth_ms: 1.8,
            mix,
            lp_l: OnePole::new(8_000.0),
            lp_r: OnePole::new(8_000.0),
        }
    }

    pub fn process(&mut self, x: f64) -> (f64, f64) {
        self.line.push(x);
        let tri = 4.0 * (self.phase - 0.5).abs() - 1.0; // −1..1
        self.phase = (self.phase + self.rate / FS) % 1.0;
        let dl = (self.center_ms + self.depth_ms * tri) * 1e-3 * FS;
        let dr = (self.center_ms - self.depth_ms * tri) * 1e-3 * FS;
        let wl = self.lp_l.lp(self.line.read(dl));
        let wr = self.lp_r.lp(self.line.read(dr));
        let dry = 1.0 - self.mix;
        (dry * x + self.mix * wl, dry * x + self.mix * wr)
    }

    pub fn run(&mut self, mono: &[f64]) -> Stereo {
        let mut s = Stereo::silence(mono.len());
        for (i, &x) in mono.iter().enumerate() {
            let (l, r) = self.process(x);
            s.l[i] = l;
            s.r[i] = r;
        }
        s
    }
}

/// Solina-style string ensemble: three ~5 ms lines modulated at 0/120/240°
/// by a slow (0.6 Hz, ±1.5 ms) plus a fast (6 Hz, ±0.15 ms) LFO.
#[derive(Clone, Debug)]
pub struct Ensemble {
    line: DelayLine,
    slow: f64,
    fast: f64,
    mix: f64,
}

impl Ensemble {
    pub fn new(mix: f64) -> Self {
        Self {
            line: DelayLine::new((0.02 * FS) as usize),
            slow: 0.0,
            fast: 0.0,
            mix,
        }
    }

    pub fn run(&mut self, mono: &[f64]) -> Stereo {
        let mut s = Stereo::silence(mono.len());
        for (i, &x) in mono.iter().enumerate() {
            self.line.push(x);
            self.slow = (self.slow + 0.6 / FS) % 1.0;
            self.fast = (self.fast + 6.0 / FS) % 1.0;
            let tap = |k: f64| {
                let ph = k / 3.0;
                let d = 5.0 + 1.5 * (TAU * (self.slow + ph)).sin() + 0.15 * (TAU * (self.fast + ph)).sin();
                self.line.read(d * 1e-3 * FS)
            };
            let (a, b, c) = (tap(0.0), tap(1.0), tap(2.0));
            let dry = 1.0 - self.mix;
            s.l[i] = dry * x + self.mix * (0.6 * a + 0.4 * b);
            s.r[i] = dry * x + self.mix * (0.6 * c + 0.4 * b);
        }
        s
    }
}

// ----------------------------------------------------------------- tape

/// Tape colour: wow (0.08% at 0.8 Hz) and flutter (0.03% at 8 Hz) as a
/// modulated delay, a +1.5 dB head bump at 70 Hz and a −2 dB shelf at 10 kHz.
/// Delay amplitude for pitch deviation δ at rate f is δ / (2π f).
#[derive(Clone, Debug)]
pub struct Tape {
    line: DelayLine,
    t: f64,
    wow_phase: f64,
    bump: Biquad,
    shelf: Biquad,
}

impl Tape {
    pub fn new(seed: u64) -> Self {
        let mut r = Rng::new(seed);
        Self {
            line: DelayLine::new((0.01 * FS) as usize),
            t: 0.0,
            wow_phase: r.uniform(),
            bump: Biquad::new(Kind::Peaking { q: 0.9, db: 1.5 }, 70.0),
            shelf: Biquad::new(Kind::HighShelf { db: -2.0 }, 10_000.0),
        }
    }

    #[inline]
    pub fn process(&mut self, x: f64) -> f64 {
        self.line.push(x);
        let wow = 0.0008 / (TAU * 0.8) * (TAU * (0.8 * self.t + self.wow_phase)).sin();
        let flutter = 0.0003 / (TAU * 8.0) * (TAU * 8.0 * self.t).sin();
        self.t += 1.0 / FS;
        let d = (0.002 + wow + flutter) * FS;
        self.shelf.process(self.bump.process(self.line.read(d)))
    }
}

// ----------------------------------------------------------------- reverb

/// 8-line feedback delay network: Householder feedback, per-line decay gain for
/// the target T60, one-pole damping in each loop, slow length modulation,
/// input diffusion and a pre-delay. Stereo in, stereo out (wet only).
#[derive(Clone, Debug)]
pub struct Fdn {
    lines: Vec<DelayLine>,
    lens: [f64; 8],
    gains: [f64; 8],
    damp: Vec<OnePole>,
    pre: DelayLine,
    pre_d: f64,
    diff: Vec<(DelayLine, f64)>,
    t: f64,
    dc: [DcBlock; 2],
}

/// Early reflections: (delay after the pre-delay in ms, gain); alternate taps
/// go left and right, so the first echoes already give the room its width.
const EARLY: [(f64, f64); 6] = [
    (7.3, 0.5),
    (11.9, -0.42),
    (17.1, 0.36),
    (23.3, -0.3),
    (29.7, 0.25),
    (37.1, -0.2),
];

const FDN_LENS: [f64; 8] = [1_427.0, 1_637.0, 1_871.0, 2_053.0, 2_311.0, 2_539.0, 2_833.0, 3_373.0];

impl Fdn {
    /// `t60` seconds; `damping_hz` loop low-pass; `predelay_ms` (20–40 ms is
    /// typical; the six early reflections follow it within 40 ms).
    pub fn new(t60: f64, damping_hz: f64, predelay_ms: f64) -> Self {
        let lens = FDN_LENS;
        let gains = lens.map(|m| 10f64.powf(-3.0 * m / (t60.max(0.05) * FS)));
        Self {
            lines: lens.iter().map(|&m| DelayLine::new(m as usize + 16)).collect(),
            lens,
            gains,
            damp: (0..8).map(|_| OnePole::new(damping_hz)).collect(),
            pre: DelayLine::new(((predelay_ms.max(0.0) + 40.0) * 1e-3 * FS) as usize + 4),
            pre_d: (predelay_ms * 1e-3 * FS).max(1.0),
            diff: [142.0, 107.0, 379.0, 277.0]
                .iter()
                .map(|&d| (DelayLine::new(d as usize + 4), d))
                .collect(),
            t: 0.0,
            dc: [DcBlock::default(), DcBlock::default()],
        }
    }

    /// Process one stereo frame; returns the wet signal.
    pub fn process(&mut self, l: f64, r: f64) -> (f64, f64) {
        self.pre.push(0.5 * (l + r));
        let mut x = self.pre.read(self.pre_d);
        let (mut el, mut er) = (0.0, 0.0);
        for (k, &(ms, g)) in EARLY.iter().enumerate() {
            let tap = g * self.pre.read(self.pre_d + ms * 1e-3 * FS);
            if k % 2 == 0 { el += tap } else { er += tap }
        }
        // Schroeder allpass diffusion, g = 0.6
        for (line, d) in &mut self.diff {
            let delayed = line.read(*d);
            let v = x + 0.6 * delayed;
            line.push(v);
            x = delayed - 0.6 * v;
        }
        let mut outs = [0.0; 8];
        for (k, line) in self.lines.iter().enumerate() {
            let m = self.lens[k] + 3.0 * (TAU * (0.07 + 0.013 * k as f64) * self.t).sin();
            outs[k] = line.read(m);
        }
        self.t += 1.0 / FS;
        let sum: f64 = outs.iter().sum::<f64>() * (2.0 / 8.0);
        for k in 0..8 {
            // Householder reflection: y = x − (2/N)·Σx
            let fb = outs[k] - sum;
            let damped = self.damp[k].lp(fb * self.gains[k]);
            let inject = if k % 2 == 0 { x } else { -x };
            self.lines[k].push(damped + 0.35 * inject);
        }
        let wl = outs[0] + outs[2] + outs[4] + outs[6];
        let wr = outs[1] + outs[3] + outs[5] + outs[7];
        (
            self.dc[0].process(0.25 * wl + 0.5 * el),
            self.dc[1].process(0.25 * wr + 0.5 * er),
        )
    }

    /// Wet response of a whole stereo buffer, with a tail of `tail_s` seconds.
    pub fn run(&mut self, input: &Stereo, tail_s: f64) -> Stereo {
        let n = input.len() + (tail_s * FS) as usize;
        let mut out = Stereo::silence(n);
        for i in 0..n {
            let (l, r) = if i < input.len() {
                (input.l[i], input.r[i])
            } else {
                (0.0, 0.0)
            };
            let (a, b) = self.process(l, r);
            out.l[i] = a;
            out.r[i] = b;
        }
        out
    }
}

/// Dattorro's plate (J. Dattorro, "Effect Design, Part 1", JAES 1997): input
/// diffusion, then a figure-of-eight tank of modulated allpasses, delays and
/// damping. Delay lengths and diffusion gains are Dattorro's published values
/// (at 29,761 Hz) scaled to 48 kHz; the output uses four of his seven taps per
/// side plus one rewired second-delay tap (decorrelation is what matters
/// here, and the test measures it). Bright and dense: the classic choice for
/// bells and glassy mallets.
#[derive(Clone, Debug)]
pub struct Plate {
    pre: DelayLine,
    bw: OnePole,
    in_ap: Vec<(DelayLine, f64, f64)>,
    tank: [TankSide; 2],
    decay: f64,
    t: f64,
}

#[derive(Clone, Debug)]
struct TankSide {
    mod_ap: (DelayLine, f64),
    d1: (DelayLine, f64),
    damp: OnePole,
    ap2: (DelayLine, f64),
    d2: (DelayLine, f64),
    phase: f64,
}

fn scaled(n: f64) -> f64 {
    n * FS / 29_761.0
}

fn allpass_step(line: &mut DelayLine, d: f64, g: f64, x: f64) -> f64 {
    let delayed = line.read(d);
    let v = x + g * delayed;
    line.push(v);
    delayed - g * v
}

impl Plate {
    /// `decay` in (0, 1): tank feedback (0.5 ≈ 1.5 s, 0.7 ≈ 3 s);
    /// `damping_hz`: tank low-pass.
    pub fn new(decay: f64, damping_hz: f64) -> Self {
        let line = |n: f64| DelayLine::new(scaled(n) as usize + 40);
        let side = |a: f64, b: f64, c: f64, d: f64, ph: f64| TankSide {
            mod_ap: (line(a + 16.0), scaled(a)),
            d1: (line(b), scaled(b)),
            damp: OnePole::new(damping_hz),
            ap2: (line(c), scaled(c)),
            d2: (line(d), scaled(d)),
            phase: ph,
        };
        Self {
            pre: DelayLine::new(scaled(10.0) as usize + 4),
            bw: OnePole::new(9_000.0),
            in_ap: [(142.0, 0.75), (107.0, 0.75), (379.0, 0.625), (277.0, 0.625)]
                .iter()
                .map(|&(n, g)| (line(n), scaled(n), g))
                .collect(),
            tank: [
                side(672.0, 4_453.0, 1_800.0, 3_720.0, 0.0),
                side(908.0, 4_217.0, 2_656.0, 3_163.0, 0.25),
            ],
            decay: decay.clamp(0.0, 0.95),
            t: 0.0,
        }
    }

    /// Process one stereo frame; returns the wet signal.
    pub fn process(&mut self, l: f64, r: f64) -> (f64, f64) {
        self.pre.push(0.5 * (l + r));
        let mut x = self.bw.lp(self.pre.read(scaled(10.0)));
        for (line, d, g) in &mut self.in_ap {
            x = allpass_step(line, *d, *g, x);
        }
        // figure of eight: each side is fed by the input plus the other side's output
        let fb = [
            self.tank[1].d2.0.read(self.tank[1].d2.1),
            self.tank[0].d2.0.read(self.tank[0].d2.1),
        ];
        let excursion = scaled(8.0);
        for (k, side) in self.tank.iter_mut().enumerate() {
            let m = side.mod_ap.1 + excursion * (TAU * (1.0 * self.t + side.phase)).sin();
            let a = allpass_step(&mut side.mod_ap.0, m, -0.7, x + self.decay * fb[k]);
            let delayed1 = side.d1.0.read(side.d1.1);
            side.d1.0.push(a);
            let damped = side.damp.lp(delayed1) * self.decay;
            let b = allpass_step(&mut side.ap2.0, side.ap2.1, 0.5, damped);
            side.d2.0.push(b);
        }
        self.t += 1.0 / FS;
        // output taps: Dattorro's d1/ap2 taps, and one rewired d2 tap per side
        let tap = |side: &TankSide, a: f64, b: f64, c: f64| {
            side.d1.0.read(scaled(a)) + side.d1.0.read(scaled(b)) - side.ap2.0.read(scaled(c))
        };
        let yl = 0.6 * (tap(&self.tank[1], 266.0, 2_974.0, 1_913.0) - self.tank[0].d2.0.read(scaled(1_996.0)));
        let yr = 0.6 * (tap(&self.tank[0], 353.0, 3_627.0, 1_228.0) - self.tank[1].d2.0.read(scaled(1_066.0)));
        (yl, yr)
    }

    /// Wet response of a whole stereo buffer, with a tail of `tail_s` seconds.
    pub fn run(&mut self, input: &Stereo, tail_s: f64) -> Stereo {
        let n = input.len() + (tail_s * FS) as usize;
        let mut out = Stereo::silence(n);
        for i in 0..n {
            let (l, r) = if i < input.len() {
                (input.l[i], input.r[i])
            } else {
                (0.0, 0.0)
            };
            let (a, b) = self.process(l, r);
            out.l[i] = a;
            out.r[i] = b;
        }
        out
    }
}

/// Ping-pong delay (gaps only): `time_s`, `feedback`, low-pass in the loop.
pub fn ping_pong(input: &Stereo, time_s: f64, feedback: f64, lp_hz: f64, mix: f64) -> Stereo {
    let d = (time_s * FS).max(1.0);
    let mut ll = DelayLine::new(d as usize + 8);
    let mut lr = DelayLine::new(d as usize + 8);
    let (mut fl, mut fr) = (OnePole::new(lp_hz), OnePole::new(lp_hz));
    let mut out = input.clone();
    for i in 0..input.len() {
        let el = ll.read(d);
        let er = lr.read(d);
        ll.push(fl.lp(0.5 * (input.l[i] + input.r[i]) + feedback * er));
        lr.push(fr.lp(feedback * el));
        out.l[i] += mix * el;
        out.r[i] += mix * er;
    }
    out
}

/// Split into mid (L+R)/2 and side (L−R)/2.
pub fn mid_side(x: &Stereo) -> (Vec<f64>, Vec<f64>) {
    let mid = x.l.iter().zip(&x.r).map(|(a, b)| 0.5 * (a + b)).collect();
    let side = x.l.iter().zip(&x.r).map(|(a, b)| 0.5 * (a - b)).collect();
    (mid, side)
}

/// Join mid and side back into left/right (exact inverse of [`mid_side`]).
pub fn from_mid_side(mid: &[f64], side: &[f64]) -> Stereo {
    Stereo {
        l: mid.iter().zip(side).map(|(m, s)| m + s).collect(),
        r: mid.iter().zip(side).map(|(m, s)| m - s).collect(),
    }
}

/// Scale the stereo width: 0 = mono, 1 = unchanged, > 1 = wider.
pub fn width(x: &Stereo, w: f64) -> Stereo {
    let (m, s) = mid_side(x);
    let s: Vec<f64> = s.iter().map(|v| v * w).collect();
    from_mid_side(&m, &s)
}

/// Constant-power pan of a mono signal; `pan` in [−1 (left), +1 (right)].
pub fn pan(mono: &[f64], pan: f64) -> Stereo {
    let a = (pan.clamp(-1.0, 1.0) + 1.0) * 0.5 * FRAC_PI_2;
    let (gl, gr) = (a.cos(), a.sin());
    Stereo {
        l: mono.iter().map(|x| x * gl).collect(),
        r: mono.iter().map(|x| x * gr).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rms(x: &[f64]) -> f64 {
        (x.iter().map(|v| v * v).sum::<f64>() / x.len().max(1) as f64).sqrt()
    }

    #[test]
    fn adsr_reaches_peak_sustains_and_releases() {
        let e = Adsr {
            attack: 0.01,
            decay: 0.2,
            sustain: 0.5,
            release: 0.3,
        }
        .render(1.0);
        let peak = e.iter().cloned().fold(0.0, f64::max);
        assert!((peak - 1.0).abs() < 1e-9);
        let at_hold_end = e[(0.99 * FS) as usize];
        assert!((at_hold_end - 0.5).abs() < 0.02, "{at_hold_end}");
        assert!(*e.last().unwrap() < 0.01);
        assert!(e.iter().all(|v| v.is_finite() && *v >= 0.0));
    }

    #[test]
    fn delay_line_reads_integer_and_fractional_delays() {
        let mut d = DelayLine::new(64);
        for i in 0..64 {
            d.push(i as f64);
        }
        assert!((d.read(1.0) - 63.0).abs() < 1e-12);
        assert!((d.read(10.0) - 54.0).abs() < 1e-12);
        assert!((d.read(10.5) - 53.5).abs() < 1e-9); // linear data: exact
    }

    /// T60 by Schroeder backward integration of an impulse response.
    fn measured_t60(ir: &[f64]) -> f64 {
        let mut e: Vec<f64> = ir.iter().map(|v| v * v).collect();
        for i in (0..e.len() - 1).rev() {
            e[i] += e[i + 1];
        }
        let e0 = e[0];
        let db = |i: usize| 10.0 * (e[i] / e0).log10();
        // fit the −5..−25 dB range (T20) and extrapolate to 60 dB
        let i5 = (0..e.len()).find(|&i| db(i) <= -5.0).unwrap();
        let i25 = (0..e.len()).find(|&i| db(i) <= -25.0).unwrap();
        3.0 * (i25 - i5) as f64 / FS
    }

    #[test]
    fn fdn_decay_time_is_within_ten_percent_of_target() {
        // Reverb time is specified at mid frequencies (the 500 Hz–1 kHz octave);
        // the loop damping and the modulated fractional reads shorten the high
        // end on purpose, so a broadband measurement would read ~10% short.
        for t60 in [0.9, 1.6, 2.5] {
            let mut f = Fdn::new(t60, 20_000.0, 0.0);
            let mut imp = Stereo::silence(1);
            imp.l[0] = 1.0;
            imp.r[0] = 1.0;
            let out = f.run(&imp, t60 * 1.5);
            let mut bp = Biquad::new(Kind::BandPass { q: 1.41 }, 707.0);
            let m: Vec<f64> = out.l.iter().zip(&out.r).map(|(a, b)| bp.process(a + b)).collect();
            let t = measured_t60(&m);
            assert!((t / t60 - 1.0).abs() < 0.10, "target {t60}: measured {t:.3}");
            assert!(out.is_finite());
        }
    }

    #[test]
    fn chorus_tape_and_ensemble_preserve_level() {
        // Broadband (pink) input: a dry/wet chorus is a comb filter, so the level
        // of any single sine depends on where it falls between the teeth; the
        // property that matters is the level of real, broadband material.
        let mut p = crate::osc::Pink::new(9);
        let x: Vec<f64> = (0..96_000).map(|_| 0.5 * p.sample()).collect();
        let r0 = rms(&x[4_800..]);
        let c = Chorus::juno(0.5, 0.5).run(&x);
        let e = Ensemble::new(0.5).run(&x);
        let mut tape = Tape::new(1);
        let t: Vec<f64> = x.iter().map(|&v| tape.process(v)).collect();
        // A linear 50/50 dry/wet mix lands between −3.01 dB (wet fully
        // decorrelated from dry) and 0 dB (fully correlated): never louder, never
        // quieter than an uncorrelated sum.
        for (name, y) in [
            ("chorus L", &c.l),
            ("chorus R", &c.r),
            ("ensemble L", &e.l),
            ("ensemble R", &e.r),
        ] {
            let g = 20.0 * (rms(&y[4_800..]) / r0).log10();
            assert!((-3.1..=0.1).contains(&g), "{name}: {g:.2} dB");
        }
        // Tape is one path with gentle EQ: level within ±1 dB.
        let g = 20.0 * (rms(&t[4_800..]) / r0).log10();
        assert!(g.abs() < 1.0, "tape: {g:.2} dB");
    }

    #[test]
    fn automation_is_smooth_and_hits_its_breakpoints() {
        let a = Automation::new(&[(2.0, 1.0), (0.0, 0.0), (4.0, 0.25)]);
        assert_eq!(a.at(-1.0), 0.0);
        assert_eq!(a.at(0.0), 0.0);
        assert_eq!(a.at(2.0), 1.0);
        assert_eq!(a.at(4.0), 0.25);
        assert_eq!(a.at(9.0), 0.25);
        assert!((a.at(1.0) - 0.5).abs() < 1e-12, "raised-cosine midpoint");
        let v = a.render((5.0 * FS) as usize);
        let max_step = v.windows(2).map(|w| (w[1] - w[0]).abs()).fold(0.0, f64::max);
        assert!(max_step < 1e-4, "{max_step}");
        assert!(
            v[..(2.0 * FS) as usize].windows(2).all(|w| w[1] >= w[0]),
            "monotone rise"
        );
        assert_eq!(Automation::new(&[]).at(3.0), 0.0);
    }

    #[test]
    fn warmth_adds_even_harmonics_at_unity_gain_without_dc() {
        let mut w = Warmth::new(2.0, 0.3);
        let small: Vec<f64> = (0..48_000)
            .map(|i| 0.01 * (TAU * 500.0 * i as f64 / FS).sin())
            .collect();
        let y: Vec<f64> = small.iter().map(|&v| w.process(v)).collect();
        let g = 20.0 * (rms(&y[24_000..]) / rms(&small[24_000..])).log10();
        assert!(g.abs() < 0.2, "small-signal gain {g:.2} dB");
        let mut w = Warmth::new(2.0, 0.3);
        let big: Vec<f64> = (0..48_000).map(|i| 0.5 * (TAU * 500.0 * i as f64 / FS).sin()).collect();
        let y: Vec<f64> = big.iter().map(|&v| w.process(v)).collect();
        let seg = &y[24_000..];
        let at = |f: f64| {
            let (mut re, mut im) = (0.0, 0.0);
            for (i, v) in seg.iter().enumerate() {
                let a = TAU * f * i as f64 / FS;
                re += v * a.cos();
                im += v * a.sin();
            }
            (re * re + im * im).sqrt()
        };
        let h2 = 20.0 * (at(1_000.0) / at(500.0)).log10();
        assert!(h2 > -40.0 && h2 < -10.0, "second harmonic {h2:.1} dB");
        let mean = seg.iter().sum::<f64>() / seg.len() as f64;
        assert!(mean.abs() < 1e-3, "DC {mean}");
    }

    #[test]
    fn mid_side_round_trips_and_width_scales_the_side() {
        let x = Stereo {
            l: vec![1.0, 0.5, -0.25],
            r: vec![0.0, 0.5, 0.75],
        };
        let (m, s) = mid_side(&x);
        assert_eq!(from_mid_side(&m, &s), x);
        let mono = width(&x, 0.0);
        assert_eq!(mono.l, mono.r);
        assert_eq!(width(&x, 1.0), x);
    }

    #[test]
    fn fdn_early_reflections_arrive_first_and_alternate_sides() {
        let mut f = Fdn::new(1.6, 6_000.0, 20.0);
        let mut imp = Stereo::silence(1);
        imp.l[0] = 1.0;
        imp.r[0] = 1.0;
        let out = f.run(&imp, 0.2);
        let first = |v: &[f64]| v.iter().position(|x| x.abs() > 1e-3).unwrap() as f64 / FS * 1e3;
        // the first echo: 20 ms pre-delay + 7.3 ms, on the left; the right's
        // first is the 11.9 ms tap; the FDN's own tail arrives later
        assert!((first(&out.l) - 27.3).abs() < 0.2, "left {:.2} ms", first(&out.l));
        assert!((first(&out.r) - 31.9).abs() < 0.2, "right {:.2} ms", first(&out.r));
    }

    #[test]
    fn plate_is_dense_decorrelated_and_decays() {
        let mut p = Plate::new(0.5, 6_000.0);
        let mut imp = Stereo::silence(1);
        imp.l[0] = 1.0;
        imp.r[0] = 1.0;
        let out = p.run(&imp, 4.0);
        assert!(out.is_finite());
        let e = |v: &[f64], a: f64, b: f64| {
            v[(a * FS) as usize..(b * FS) as usize]
                .iter()
                .map(|x| x * x)
                .sum::<f64>()
        };
        let early = e(&out.l, 0.05, 0.25) + e(&out.r, 0.05, 0.25);
        let late = e(&out.l, 3.5, 4.0) + e(&out.r, 3.5, 4.0);
        assert!(
            early > 0.0 && 10.0 * (late / early).log10() < -40.0,
            "decay {:.1} dB",
            10.0 * (late / early).log10()
        );
        // left and right are decorrelated (a plate's width)
        let (a, b) = (&out.l[2_400..96_000], &out.r[2_400..96_000]);
        let dot: f64 = a.iter().zip(b).map(|(x, y)| x * y).sum();
        let corr = dot / (a.iter().map(|x| x * x).sum::<f64>() * b.iter().map(|y| y * y).sum::<f64>()).sqrt();
        assert!(corr.abs() < 0.5, "L/R correlation {corr:.2}");
    }

    /// T60 from Schroeder backward integration (the −5 to −35 dB slope, ×2).
    fn t60_of(x: &[f64]) -> f64 {
        let mut edc: Vec<f64> = x
            .iter()
            .rev()
            .scan(0.0, |acc, v| {
                *acc += v * v;
                Some(*acc)
            })
            .collect();
        edc.reverse();
        let db = |i: usize| 10.0 * (edc[i] / edc[0]).log10();
        let t5 = (0..edc.len()).find(|&i| db(i) < -5.0).unwrap();
        let t35 = (0..edc.len()).find(|&i| db(i) < -35.0).unwrap();
        2.0 * (t35 - t5) as f64 / FS
    }

    #[test]
    fn plate_decay_parameter_sets_the_tail_length() {
        let t60 = |decay: f64| {
            let mut p = Plate::new(decay, 6_000.0);
            let mut imp = Stereo::silence(1);
            imp.l[0] = 1.0;
            imp.r[0] = 1.0;
            let out = p.run(&imp, 12.0);
            t60_of(&out.l)
        };
        let (a, b) = (t60(0.5), t60(0.7));
        // the doc's figures: 0.5 ≈ 1.5 s, 0.7 ≈ 3 s
        assert!((1.1..=1.9).contains(&a), "decay 0.5: T60 {a:.2} s");
        assert!((2.4..=3.6).contains(&b), "decay 0.7: T60 {b:.2} s");
    }

    #[test]
    fn ping_pong_echoes_alternate_left_then_right() {
        let mut x = Stereo::silence((1.2 * FS) as usize);
        x.l[0] = 1.0;
        x.r[0] = 1.0;
        let y = ping_pong(&x, 0.25, 0.5, 20_000.0, 1.0);
        let near = |v: &[f64], t: f64| {
            v[(t * FS) as usize - 50..(t * FS) as usize + 50]
                .iter()
                .fold(0.0_f64, |m, s| m.max(s.abs()))
        };
        // first echo on the left at 0.25 s, then on the right at 0.5 s, then left at 0.75 s
        assert!(near(&y.l, 0.25) > 0.3 && near(&y.r, 0.25) < 0.01);
        assert!(near(&y.r, 0.5) > 0.1 && near(&y.l, 0.5) < 0.01);
        assert!(near(&y.l, 0.75) > 0.05 && near(&y.r, 0.75) < 0.01);
        assert!(near(&y.l, 0.75) < near(&y.l, 0.25), "echoes decay");
    }

    #[test]
    fn constant_power_pan() {
        let s = pan(&[1.0], 0.0);
        assert!((s.l[0] * s.l[0] + s.r[0] * s.r[0] - 1.0).abs() < 1e-12);
        let hard_left = pan(&[1.0], -1.0);
        assert!((hard_left.l[0] - 1.0).abs() < 1e-12 && hard_left.r[0].abs() < 1e-12);
    }
}
