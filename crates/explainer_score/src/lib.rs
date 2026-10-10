//! `explainer_score`: the sound engine for the explainer corpus.
//!
//! The music must never compete with the narration (plan §7): it sits 18 LU
//! under the voice while someone is speaking, opens up only in authored gaps,
//! keeps out of the narrator's pitch range, and is ducked offline with perfect
//! look-ahead because the narration timeline is known before a note is
//! synthesized. Everything here is std-only Rust, f64 internally, at 48 kHz,
//! and deterministic from seeds.

pub mod filter;
pub mod fx;
pub mod instr;
pub mod meter;
pub mod osc;
pub mod resample;
pub mod rng;
pub mod theory;
pub mod units;
pub mod wav;

/// The engine's sample rate, matching franken_manim's mixer.
pub const SAMPLE_RATE: u32 = 48_000;
/// [`SAMPLE_RATE`] as `f64`, for DSP arithmetic.
pub const FS: f64 = SAMPLE_RATE as f64;

/// A stereo buffer of f64 samples at [`SAMPLE_RATE`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stereo {
    pub l: Vec<f64>,
    pub r: Vec<f64>,
}

impl Stereo {
    /// `n` frames of silence.
    pub fn silence(n: usize) -> Self {
        Self {
            l: vec![0.0; n],
            r: vec![0.0; n],
        }
    }

    /// Number of frames (samples per channel).
    pub fn len(&self) -> usize {
        debug_assert_eq!(self.l.len(), self.r.len());
        self.l.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The same signal in both channels.
    pub fn from_mono(m: &[f64]) -> Self {
        Self {
            l: m.to_vec(),
            r: m.to_vec(),
        }
    }

    /// Mix `other` into `self` starting at frame `at`, scaled by `gain`;
    /// samples past the end of `self` are dropped (stems have a fixed length).
    pub fn mix_in(&mut self, other: &Stereo, at: usize, gain: f64) {
        let n = self.len();
        for i in 0..other.len() {
            let j = at + i;
            if j >= n {
                break;
            }
            self.l[j] += gain * other.l[i];
            self.r[j] += gain * other.r[i];
        }
    }

    /// Largest absolute sample value over both channels.
    pub fn peak(&self) -> f64 {
        self.l.iter().chain(&self.r).fold(0.0_f64, |m, &x| m.max(x.abs()))
    }

    /// True if every sample is finite.
    pub fn is_finite(&self) -> bool {
        self.l.iter().chain(&self.r).all(|x| x.is_finite())
    }

    /// Multiply every sample by `g`.
    pub fn scale(&mut self, g: f64) {
        for x in self.l.iter_mut().chain(self.r.iter_mut()) {
            *x *= g;
        }
    }
}
