//! Filters: topology-preserving one-pole, Simper's state-variable filter, a
//! zero-delay-feedback 4-pole ladder with antiderivative-anti-aliased
//! saturation in the loop, RBJ biquads, zero-phase forward–backward filtering,
//! and a DC blocker.
//!
//! References: Zavalishin, *The Art of VA Filter Design* (TPT, ZDF ladder);
//! Simper, "Linear trapezoidal integrated SVF"; Bristow-Johnson, "Audio EQ
//! Cookbook"; Parker, Zavalishin & Le Bivic, "Reducing the aliasing of nonlinear
//! waveshaping using continuous-time convolution" (DAFx-16).

use std::f64::consts::PI;

use crate::FS;

/// Pre-warped integrator gain for cutoff `fc` (Hz).
#[inline]
fn prewarp(fc: f64) -> f64 {
    (PI * fc.clamp(1.0, 0.49 * FS) / FS).tan()
}

/// Topology-preserving one-pole (low-pass output; `x - lp` is the high-pass).
#[derive(Clone, Debug, Default)]
pub struct OnePole {
    s: f64,
    g_big: f64,
}

impl OnePole {
    pub fn new(fc: f64) -> Self {
        let mut f = Self::default();
        f.set(fc);
        f
    }
    pub fn set(&mut self, fc: f64) {
        let g = prewarp(fc);
        self.g_big = g / (1.0 + g);
    }
    /// Low-pass output.
    #[inline]
    pub fn lp(&mut self, x: f64) -> f64 {
        let v = (x - self.s) * self.g_big;
        let y = v + self.s;
        self.s = y + v;
        y
    }
    /// High-pass output.
    #[inline]
    pub fn hp(&mut self, x: f64) -> f64 {
        x - self.lp(x)
    }
}

/// Outputs of one [`Svf`] step.
#[derive(Clone, Copy, Debug)]
pub struct SvfOut {
    pub low: f64,
    pub band: f64,
    pub high: f64,
}

/// Simper's trapezoidal state-variable filter (stable under modulation).
#[derive(Clone, Debug, Default)]
pub struct Svf {
    ic1: f64,
    ic2: f64,
    a1: f64,
    a2: f64,
    a3: f64,
    k: f64,
}

impl Svf {
    /// `q` ≥ 0.5; 0.707 is Butterworth.
    pub fn new(fc: f64, q: f64) -> Self {
        let mut f = Self::default();
        f.set(fc, q);
        f
    }
    pub fn set(&mut self, fc: f64, q: f64) {
        let g = prewarp(fc);
        self.k = 1.0 / q.max(0.5);
        self.a1 = 1.0 / (1.0 + g * (g + self.k));
        self.a2 = g * self.a1;
        self.a3 = g * self.a2;
    }
    #[inline]
    pub fn step(&mut self, x: f64) -> SvfOut {
        let v3 = x - self.ic2;
        let v1 = self.a1 * self.ic1 + self.a2 * v3;
        let v2 = self.ic2 + self.a2 * self.ic1 + self.a3 * v3;
        self.ic1 = 2.0 * v1 - self.ic1;
        self.ic2 = 2.0 * v2 - self.ic2;
        SvfOut {
            low: v2,
            band: v1,
            high: x - self.k * v1 - v2,
        }
    }
}

/// First-order antiderivative anti-aliasing for tanh (ADAA).
#[derive(Clone, Debug, Default)]
pub struct AdaaTanh {
    x1: f64,
}

/// ln(cosh x), written so it never overflows.
#[inline]
fn log_cosh(x: f64) -> f64 {
    let a = x.abs();
    a + (-2.0 * a).exp().ln_1p() - std::f64::consts::LN_2
}

impl AdaaTanh {
    #[inline]
    pub fn process(&mut self, x: f64) -> f64 {
        let d = x - self.x1;
        let y = if d.abs() < 1e-6 {
            (0.5 * (x + self.x1)).tanh()
        } else {
            (log_cosh(x) - log_cosh(self.x1)) / d
        };
        self.x1 = x;
        y
    }
}

/// Zero-delay-feedback 4-pole low-pass ladder (Moog-style) with saturation in
/// the feedback path. `resonance` k in [0, 4); stays stable for any cutoff up to
/// 0.45·fs. Output carries partial passband compensation (1 + k/2).
#[derive(Clone, Debug)]
pub struct Ladder {
    s: [f64; 4],
    g: f64,
    g_big: f64,
    k: f64,
    drive: f64,
    sat: AdaaTanh,
}

impl Ladder {
    pub fn new(fc: f64, resonance: f64, drive: f64) -> Self {
        let mut l = Self {
            s: [0.0; 4],
            g: 0.0,
            g_big: 0.0,
            k: 0.0,
            drive: drive.max(0.05),
            sat: AdaaTanh::default(),
        };
        l.set(fc, resonance);
        l
    }
    pub fn set(&mut self, fc: f64, resonance: f64) {
        self.g = prewarp(fc);
        self.g_big = self.g / (1.0 + self.g);
        self.k = resonance.clamp(0.0, 3.95);
    }
    #[inline]
    pub fn process(&mut self, x: f64) -> f64 {
        let gb = self.g_big;
        let inv = 1.0 / (1.0 + self.g);
        let s = &self.s;
        let big_s = (gb * gb * gb * s[0] + gb * gb * s[1] + gb * s[2] + s[3]) * inv;
        let g4 = gb * gb * gb * gb;
        let y4_lin = (g4 * x + big_s) / (1.0 + self.k * g4);
        let u = self.sat.process(self.drive * (x - self.k * y4_lin)) / self.drive;
        let mut v = u;
        for st in &mut self.s {
            let w = (v - *st) * gb;
            let y = w + *st;
            *st = y + w;
            v = y;
        }
        v * (1.0 + 0.5 * self.k)
    }
}

/// RBJ biquad (Direct Form I in f64).
#[derive(Clone, Debug, Default)]
pub struct Biquad {
    b0: f64,
    b1: f64,
    b2: f64,
    a1: f64,
    a2: f64,
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,
}

/// Biquad designs from the Audio EQ Cookbook.
#[derive(Clone, Copy, Debug)]
pub enum Kind {
    LowPass { q: f64 },
    HighPass { q: f64 },
    BandPass { q: f64 },
    Peaking { q: f64, db: f64 },
    LowShelf { db: f64 },
    HighShelf { db: f64 },
}

impl Biquad {
    pub fn new(kind: Kind, f0: f64) -> Self {
        let w0 = 2.0 * PI * f0.clamp(1.0, 0.49 * FS) / FS;
        let (cw, sw) = (w0.cos(), w0.sin());
        let (b0, b1, b2, a0, a1, a2) = match kind {
            Kind::LowPass { q } => {
                let al = sw / (2.0 * q);
                (
                    (1.0 - cw) / 2.0,
                    1.0 - cw,
                    (1.0 - cw) / 2.0,
                    1.0 + al,
                    -2.0 * cw,
                    1.0 - al,
                )
            }
            Kind::HighPass { q } => {
                let al = sw / (2.0 * q);
                (
                    (1.0 + cw) / 2.0,
                    -(1.0 + cw),
                    (1.0 + cw) / 2.0,
                    1.0 + al,
                    -2.0 * cw,
                    1.0 - al,
                )
            }
            Kind::BandPass { q } => {
                let al = sw / (2.0 * q);
                (al, 0.0, -al, 1.0 + al, -2.0 * cw, 1.0 - al)
            }
            Kind::Peaking { q, db } => {
                let a = 10f64.powf(db / 40.0);
                let al = sw / (2.0 * q);
                (
                    1.0 + al * a,
                    -2.0 * cw,
                    1.0 - al * a,
                    1.0 + al / a,
                    -2.0 * cw,
                    1.0 - al / a,
                )
            }
            Kind::LowShelf { db } | Kind::HighShelf { db } => {
                let a = 10f64.powf(db / 40.0);
                let al = sw / 2.0 * 2f64.sqrt(); // shelf slope S = 1
                let tsa = 2.0 * a.sqrt() * al;
                if matches!(kind, Kind::LowShelf { .. }) {
                    (
                        a * ((a + 1.0) - (a - 1.0) * cw + tsa),
                        2.0 * a * ((a - 1.0) - (a + 1.0) * cw),
                        a * ((a + 1.0) - (a - 1.0) * cw - tsa),
                        (a + 1.0) + (a - 1.0) * cw + tsa,
                        -2.0 * ((a - 1.0) + (a + 1.0) * cw),
                        (a + 1.0) + (a - 1.0) * cw - tsa,
                    )
                } else {
                    (
                        a * ((a + 1.0) + (a - 1.0) * cw + tsa),
                        -2.0 * a * ((a - 1.0) + (a + 1.0) * cw),
                        a * ((a + 1.0) + (a - 1.0) * cw - tsa),
                        (a + 1.0) - (a - 1.0) * cw + tsa,
                        2.0 * ((a - 1.0) - (a + 1.0) * cw),
                        (a + 1.0) - (a - 1.0) * cw - tsa,
                    )
                }
            }
        };
        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
            ..Self::default()
        }
    }

    /// A biquad from normalized coefficients: `b` = [b0, b1, b2] and
    /// `a` = [a1, a2] with a0 = 1 (e.g. published K-weighting filters).
    pub fn from_coefficients(b: [f64; 3], a: [f64; 2]) -> Self {
        Self {
            b0: b[0],
            b1: b[1],
            b2: b[2],
            a1: a[0],
            a2: a[1],
            ..Self::default()
        }
    }

    #[inline]
    pub fn process(&mut self, x: f64) -> f64 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2 - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }

    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }

    /// Magnitude response in dB at `f` Hz (analytic, for tests and design).
    pub fn magnitude_db(&self, f: f64) -> f64 {
        let w = 2.0 * PI * f / FS;
        let (c1, s1, c2, s2) = (w.cos(), w.sin(), (2.0 * w).cos(), (2.0 * w).sin());
        let (nr, ni) = (self.b0 + self.b1 * c1 + self.b2 * c2, -(self.b1 * s1 + self.b2 * s2));
        let (dr, di) = (1.0 + self.a1 * c1 + self.a2 * c2, -(self.a1 * s1 + self.a2 * s2));
        10.0 * ((nr * nr + ni * ni) / (dr * dr + di * di)).log10()
    }
}

/// Zero-phase filtering: run `make()` forward, then a fresh copy backward, over
/// a reflection-padded copy of `x`. The magnitude response is the filter's
/// squared (so design each pass for half the dB); the phase is exactly zero,
/// which is what lets a dry/filtered crossfade act as a click-free time-varying
/// EQ (the speech pocket, plan §7.2).
pub fn filtfilt(x: &[f64], make: impl Fn() -> Biquad) -> Vec<f64> {
    if x.is_empty() {
        return Vec::new();
    }
    let pad = 4_800.min(x.len() - 1);
    let mut buf = Vec::with_capacity(x.len() + 2 * pad);
    // odd reflection about the end points keeps the edges continuous
    for i in (1..=pad).rev() {
        buf.push(2.0 * x[0] - x[i]);
    }
    buf.extend_from_slice(x);
    let last = x[x.len() - 1];
    for i in 1..=pad {
        buf.push(2.0 * last - x[x.len() - 1 - i]);
    }
    let mut f = make();
    for v in buf.iter_mut() {
        *v = f.process(*v);
    }
    let mut b = make();
    for v in buf.iter_mut().rev() {
        *v = b.process(*v);
    }
    buf[pad..pad + x.len()].to_vec()
}

/// DC blocker: y = x − x[n−1] + R·y[n−1].
#[derive(Clone, Debug)]
pub struct DcBlock {
    x1: f64,
    y1: f64,
    r: f64,
}

impl Default for DcBlock {
    fn default() -> Self {
        Self {
            x1: 0.0,
            y1: 0.0,
            r: 0.9995,
        }
    }
}

impl DcBlock {
    #[inline]
    pub fn process(&mut self, x: f64) -> f64 {
        let y = x - self.x1 + self.r * self.y1;
        self.x1 = x;
        self.y1 = y;
        y
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::TAU;

    /// Steady-state gain (dB) of `f` for a sine at `hz`.
    fn sine_gain_db(mut f: impl FnMut(f64) -> f64, hz: f64) -> f64 {
        let n = 48_000;
        let mut e_in = 0.0;
        let mut e_out = 0.0;
        for i in 0..n {
            let x = (TAU * hz * i as f64 / FS).sin();
            let y = f(x);
            if i > n / 2 {
                e_in += x * x;
                e_out += y * y;
            }
        }
        10.0 * (e_out / e_in).log10()
    }

    #[test]
    fn one_pole_is_3db_down_at_cutoff() {
        for fc in [100.0, 1_000.0, 8_000.0] {
            let mut f = OnePole::new(fc);
            let g = sine_gain_db(|x| f.lp(x), fc);
            assert!((g + 3.01).abs() < 0.05, "fc {fc}: {g:.3} dB");
        }
    }

    #[test]
    fn svf_low_pass_at_butterworth_q_is_3db_down_at_cutoff() {
        for fc in [200.0, 2_000.0] {
            let mut f = Svf::new(fc, std::f64::consts::FRAC_1_SQRT_2);
            let g = sine_gain_db(|x| f.step(x).low, fc);
            assert!((g + 3.01).abs() < 0.1, "fc {fc}: {g:.3} dB");
            let mut f2 = Svf::new(fc, std::f64::consts::FRAC_1_SQRT_2);
            let pass = sine_gain_db(|x| f2.step(x).low, fc / 10.0);
            assert!(pass.abs() < 0.1, "passband {pass}");
        }
    }

    #[test]
    fn ladder_cutoff_within_five_percent_and_stable_everywhere() {
        // With no resonance, a 4-pole ladder is -12 dB at fc (four one-poles,
        // each -3 dB); find where the response crosses -12 dB.
        for fc in [300.0, 1_200.0, 4_000.0] {
            let gain = |hz: f64| {
                let mut l = Ladder::new(fc, 0.0, 0.1);
                sine_gain_db(|x| l.process(0.05 * x) / 0.05, hz)
            };
            let (mut lo, mut hi) = (fc * 0.5, fc * 2.0);
            for _ in 0..30 {
                let mid = (lo * hi).sqrt();
                if gain(mid) > -12.04 { lo = mid } else { hi = mid }
            }
            let measured = (lo * hi).sqrt();
            assert!((measured / fc - 1.0).abs() < 0.05, "fc {fc}: measured {measured:.1}");
        }
        // Stability: white noise through high resonance at extreme cutoffs.
        let mut w = crate::osc::White::new(3);
        for fc in [40.0, 1_000.0, 0.45 * FS] {
            for k in [0.0, 2.0, 3.9] {
                let mut l = Ladder::new(fc, k, 2.0);
                for _ in 0..48_000 {
                    let y = l.process(w.sample());
                    assert!(y.is_finite() && y.abs() < 50.0, "fc {fc} k {k}: {y}");
                }
            }
        }
    }

    #[test]
    fn adaa_tanh_matches_tanh_on_slow_signals_and_aliases_less_on_fast_ones() {
        // ADAA evaluates tanh at the midpoint of consecutive inputs; prime its
        // state with the sample before the ramp so the first output is not a
        // start-up transient from the zero initial state.
        let mut s = AdaaTanh::default();
        s.process(-2.0 - 0.004);
        for i in 0..1_000 {
            let x = (i as f64 / 1_000.0) * 4.0 - 2.0;
            let y = s.process(x);
            assert!((y - (x - 0.002).tanh()).abs() < 1e-3, "{x}: {y}");
        }
        // A 3 kHz sine driven hard: compare energy above 10 kHz, which only
        // aliased and high harmonics occupy, against naive tanh.
        let (mut a, mut ea, mut en) = (AdaaTanh::default(), 0.0, 0.0);
        let (mut ha, mut hn) = (Svf::new(12_000.0, 0.707), Svf::new(12_000.0, 0.707));
        for i in 0..48_000 {
            let x = 4.0 * (TAU * 3_000.0 * i as f64 / FS).sin();
            let ya = ha.step(a.process(x)).high;
            let yn = hn.step(x.tanh()).high;
            if i > 1_000 {
                ea += ya * ya;
                en += yn * yn;
            }
        }
        assert!(ea < en, "adaa {ea} naive {en}");
    }

    #[test]
    fn biquad_designs_hit_their_specs() {
        let pk = Biquad::new(Kind::Peaking { q: 0.7, db: -5.0 }, 2_000.0);
        assert!((pk.magnitude_db(2_000.0) + 5.0).abs() < 0.01);
        assert!(pk.magnitude_db(60.0).abs() < 0.3);
        let ls = Biquad::new(Kind::LowShelf { db: 1.5 }, 70.0);
        assert!((ls.magnitude_db(15.0) - 1.5).abs() < 0.1);
        assert!(ls.magnitude_db(5_000.0).abs() < 0.05);
        let hs = Biquad::new(Kind::HighShelf { db: -2.0 }, 10_000.0);
        assert!((hs.magnitude_db(20_000.0) + 2.0).abs() < 0.3);
        let lp = Biquad::new(
            Kind::LowPass {
                q: std::f64::consts::FRAC_1_SQRT_2,
            },
            1_000.0,
        );
        assert!((lp.magnitude_db(1_000.0) + 3.01).abs() < 0.05);
        // the analytic response agrees with the filter actually run
        let mut run = Biquad::new(Kind::Peaking { q: 0.7, db: -5.0 }, 2_000.0);
        let g = sine_gain_db(|x| run.process(x), 2_000.0);
        assert!((g + 5.0).abs() < 0.05, "{g}");
    }

    #[test]
    fn filtfilt_is_zero_phase_with_squared_magnitude() {
        // A 2 kHz sine through a -2.5 dB peaking filter twice: -5 dB, no shift.
        let n = 24_000;
        let x: Vec<f64> = (0..n).map(|i| (TAU * 2_000.0 * i as f64 / FS).sin()).collect();
        let y = filtfilt(&x, || Biquad::new(Kind::Peaking { q: 0.7, db: -2.5 }, 2_000.0));
        let g = 10f64.powf(-5.0 / 20.0);
        for i in 6_000..18_000 {
            assert!((y[i] - g * x[i]).abs() < 2e-3, "{i}: {} vs {}", y[i], g * x[i]);
        }
    }

    #[test]
    fn dc_blocker_removes_offset() {
        let mut d = DcBlock::default();
        let mut last = 0.0;
        for i in 0..96_000 {
            last = d.process(0.3 + 0.1 * (TAU * 440.0 * i as f64 / FS).sin());
        }
        assert!(last.abs() < 0.15);
        let mut d2 = DcBlock::default();
        let tail: f64 = (0..96_000)
            .map(|_| d2.process(0.3))
            .skip(90_000)
            .map(f64::abs)
            .fold(0.0, f64::max);
        assert!(tail < 1e-3, "{tail}");
    }
}
