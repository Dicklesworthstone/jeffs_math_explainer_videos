//! Oscillators: band-limited (PolyBLEP) saw, pulse and triangle; sine; white
//! and pink noise; and Ornstein–Uhlenbeck pitch drift, the slow random wander
//! that makes a voice sound played rather than computed.
//!
//! References: Välimäki & Huovilainen, "Antialiasing oscillators in subtractive
//! synthesis", IEEE SPM 2007; docs/research/sound_design/MEMO.md §3.1.

use std::f64::consts::TAU;

use crate::FS;
use crate::rng::Rng;

/// Two-sample polynomial correction of a unit step at phase 0 (`t` = phase in
/// [0, 1), `dt` = frequency / sample rate).
#[inline]
pub fn poly_blep(t: f64, dt: f64) -> f64 {
    if t < dt {
        let x = t / dt;
        x + x - x * x - 1.0
    } else if t > 1.0 - dt {
        let x = (t - 1.0) / dt;
        x * x + x + x + 1.0
    } else {
        0.0
    }
}

/// Waveforms a [`BlepOsc`] can produce.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Wave {
    Saw,
    /// Pulse with the given width in (0, 1); 0.5 is a square.
    Pulse(f64),
    /// Triangle from a leaky-integrated band-limited square.
    Triangle,
    Sine,
}

/// A band-limited oscillator with phase continuity across frequency changes.
#[derive(Clone, Debug)]
pub struct BlepOsc {
    pub wave: Wave,
    phase: f64,
    tri: f64,
}

impl BlepOsc {
    /// `phase0` in [0, 1): start phase (randomize it per voice from a seed).
    pub fn new(wave: Wave, phase0: f64) -> Self {
        Self {
            wave,
            phase: phase0.rem_euclid(1.0),
            tri: 0.0,
        }
    }

    /// One sample at frequency `hz`, advancing the phase.
    #[inline]
    pub fn next(&mut self, hz: f64) -> f64 {
        let dt = (hz / FS).clamp(1e-9, 0.49);
        let t = self.phase;
        let y = match self.wave {
            Wave::Saw => 2.0 * t - 1.0 - poly_blep(t, dt),
            Wave::Pulse(w) => {
                let w = w.clamp(0.02, 0.98);
                let naive = if t < w { 1.0 } else { -1.0 };
                naive + poly_blep(t, dt) - poly_blep((t + 1.0 - w).rem_euclid(1.0), dt)
            }
            Wave::Triangle => {
                let sq =
                    (if t < 0.5 { 1.0 } else { -1.0 }) + poly_blep(t, dt) - poly_blep((t + 0.5).rem_euclid(1.0), dt);
                // Leaky integration of the square; scaled so the peak is ≈ 1.
                self.tri = dt * sq + (1.0 - dt) * self.tri;
                self.tri * 4.0
            }
            Wave::Sine => (TAU * t).sin(),
        };
        self.phase += dt;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        y
    }
}

/// White noise in [-1, 1).
#[derive(Clone, Debug)]
pub struct White(Rng);

impl White {
    pub fn new(seed: u64) -> Self {
        Self(Rng::new(seed))
    }
    #[inline]
    pub fn sample(&mut self) -> f64 {
        self.0.range(-1.0, 1.0)
    }
}

/// Pink (1/f) noise: Paul Kellet's refined filter over white noise.
#[derive(Clone, Debug)]
pub struct Pink {
    white: White,
    b: [f64; 7],
}

impl Pink {
    pub fn new(seed: u64) -> Self {
        Self {
            white: White::new(seed),
            b: [0.0; 7],
        }
    }
    #[inline]
    pub fn sample(&mut self) -> f64 {
        let w = self.white.sample();
        let b = &mut self.b;
        b[0] = 0.99886 * b[0] + w * 0.055_517_9;
        b[1] = 0.99332 * b[1] + w * 0.075_075_9;
        b[2] = 0.96900 * b[2] + w * 0.153_852_0;
        b[3] = 0.86650 * b[3] + w * 0.310_485_6;
        b[4] = 0.55000 * b[4] + w * 0.532_952_2;
        b[5] = -0.7616 * b[5] - w * 0.016_898_0;
        let out = b[0] + b[1] + b[2] + b[3] + b[4] + b[5] + b[6] + w * 0.5362;
        b[6] = w * 0.115_926;
        out * 0.11
    }
}

/// Ornstein–Uhlenbeck drift in cents: mean-reverting random wander with
/// stationary standard deviation `sigma_cents`, reversion rate `theta` (1/s).
/// Updated at control rate; call [`Drift::step`] once per `dt` seconds.
#[derive(Clone, Debug)]
pub struct Drift {
    x: f64,
    theta: f64,
    sigma: f64,
    rng: Rng,
}

impl Drift {
    pub fn new(seed: u64, sigma_cents: f64, theta: f64) -> Self {
        let mut rng = Rng::new(seed);
        // Start from the stationary distribution so there is no warm-up.
        let x = rng.normal() * sigma_cents;
        Self {
            x,
            theta,
            sigma: sigma_cents,
            rng,
        }
    }

    /// Advance by `dt` seconds; returns the current offset in cents.
    pub fn step(&mut self, dt: f64) -> f64 {
        // Exact discretization of the OU process.
        let a = (-self.theta * dt).exp();
        let s = self.sigma * (1.0 - a * a).sqrt();
        self.x = a * self.x + s * self.rng.normal();
        self.x
    }

    pub fn cents(&self) -> f64 {
        self.x
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Energy (dB relative to total) of the DFT bins below `fmax` that are not
    /// within ±2 bins of a harmonic of `f0`: aliasing and noise.
    fn inharmonic_db(x: &[f64], f0: f64, fmax: f64) -> f64 {
        let n = x.len();
        let mut total = 0.0;
        let mut bad = 0.0;
        // 4-term Blackman–Harris window (sidelobes ≈ −92 dB): a Hann window's
        // leakage from the strong low harmonics alone reads ≈ −36 dB here and
        // would hide the aliasing being measured.
        let w: Vec<f64> = (0..n)
            .map(|i| {
                let a = TAU * i as f64 / n as f64;
                0.35875 - 0.48829 * a.cos() + 0.14128 * (2.0 * a).cos() - 0.01168 * (3.0 * a).cos()
            })
            .collect();
        let kmax = (fmax * n as f64 / FS) as usize;
        for k in 1..kmax {
            let (mut re, mut im) = (0.0, 0.0);
            let step = TAU * k as f64 / n as f64;
            for (i, (&xi, &wi)) in x.iter().zip(&w).enumerate() {
                let a = step * i as f64;
                re += xi * wi * a.cos();
                im -= xi * wi * a.sin();
            }
            let p = re * re + im * im;
            total += p;
            let f = k as f64 * FS / n as f64;
            let h = (f / f0).round().max(1.0);
            // ±5 bins: the Blackman–Harris main lobe is 4 bins wide each side.
            if (f - h * f0).abs() > 5.0 * FS / n as f64 {
                bad += p;
            }
        }
        10.0 * (bad / total).log10()
    }

    #[test]
    fn polyblep_saw_is_far_cleaner_than_naive() {
        // The memo's measurement point: a 1.78 kHz saw.
        let f0 = 1_780.0;
        let n = 16_384;
        let mut o = BlepOsc::new(Wave::Saw, 0.1);
        let blep: Vec<f64> = (0..n).map(|_| o.next(f0)).collect();
        let mut ph: f64 = 0.1;
        let naive: Vec<f64> = (0..n)
            .map(|_| {
                let y = 2.0 * ph - 1.0;
                ph = (ph + f0 / FS) % 1.0;
                y
            })
            .collect();
        let (b, nv) = (inharmonic_db(&blep, f0, 5_000.0), inharmonic_db(&naive, f0, 5_000.0));
        assert!(b < -45.0, "polyblep inharmonic energy {b:.1} dB");
        assert!(b < nv - 20.0, "polyblep {b:.1} dB vs naive {nv:.1} dB");
    }

    #[test]
    fn waves_are_bounded_and_centered() {
        for wave in [
            Wave::Saw,
            Wave::Pulse(0.5),
            Wave::Pulse(0.3),
            Wave::Triangle,
            Wave::Sine,
        ] {
            let mut o = BlepOsc::new(wave, 0.37);
            let x: Vec<f64> = (0..48_000).map(|_| o.next(220.0)).collect();
            let peak = x.iter().fold(0.0_f64, |m, v| m.max(v.abs()));
            let mean = x[4_800..].iter().sum::<f64>() / (x.len() - 4_800) as f64;
            assert!(peak < 1.3 && peak > 0.5, "{wave:?} peak {peak}");
            let tol = if matches!(wave, Wave::Pulse(w) if w != 0.5) {
                0.5
            } else {
                0.05
            };
            assert!(mean.abs() < tol, "{wave:?} mean {mean}");
        }
    }

    #[test]
    fn drift_has_the_requested_spread_and_reverts() {
        let mut d = Drift::new(11, 3.0, 0.8);
        let xs: Vec<f64> = (0..200_000).map(|_| d.step(0.01)).collect();
        let m = xs.iter().sum::<f64>() / xs.len() as f64;
        let sd = (xs.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / xs.len() as f64).sqrt();
        assert!(m.abs() < 0.5, "mean {m}");
        assert!((sd - 3.0).abs() < 0.4, "sd {sd}");
    }

    #[test]
    fn pink_noise_falls_about_3db_per_octave() {
        let mut p = Pink::new(5);
        let x: Vec<f64> = (0..1 << 16).map(|_| p.sample()).collect();
        // Compare band energies via a crude octave filterbank: one-pole splits.
        fn band(x: &[f64], lo: f64, hi: f64) -> f64 {
            let (a_lo, a_hi) = ((-TAU * lo / FS).exp(), (-TAU * hi / FS).exp());
            let (mut l1, mut l2) = (0.0, 0.0);
            let mut e = 0.0;
            for &v in x {
                l1 = (1.0 - a_hi) * v + a_hi * l1; // low-pass at hi
                l2 = (1.0 - a_lo) * v + a_lo * l2; // low-pass at lo
                let b = l1 - l2;
                e += b * b;
            }
            e
        }
        // Pink noise has equal energy per octave, i.e. a power spectral DENSITY
        // falling 3 dB per octave: compare energy per hertz of bandwidth.
        let d1 = band(&x, 250.0, 500.0) / 250.0;
        let d2 = band(&x, 2_000.0, 4_000.0) / 2_000.0;
        let slope = 10.0 * (d2 / d1).log10() / 3.0; // dB per octave over 3 octaves
        assert!((-5.0..-1.0).contains(&slope), "slope {slope:.2} dB/oct");
    }
}
