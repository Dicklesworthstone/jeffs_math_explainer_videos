//! The RationalFrameClock (§9.2, D-07, BN-02): manim's exact nominal
//! sample points, drift-free.
//!
//! The Reference advances a float accumulator over
//! `np.arange(0, run_time, 1/fps) + 1/fps`; at 30 fps an hour of frames
//! drifts visibly and two runs disagree under different chunkings. Here
//! time never accumulates — it *derives*: the clock is `(frame_index,
//! fps)`, a [`RationalTime`] whose value is exactly `frames / fps`.
//!
//! The emission semantics kept from the Reference, exactly:
//! - samples are `t_k = k / fps` for `k = 1..=n` — **no alpha-zero frame**
//!   in the emission sequence (`begin()` interpolates at zero separately);
//! - **upward duration rounding**: `n = ceil(run_time · fps)` computed on
//!   the exact rational value of the `f64` duration, so a run_time that is
//!   not a whole number of frames still covers its end;
//! - the final sample may exceed `run_time`; **alpha clamps to [0, 1]**
//!   (the Reference's `interpolate` clip);
//! - skipped playback visits the same grid samples while suppressing capture,
//!   so state evolution is identical rather than merely time-equivalent.
//!
//! **PERMANENT REFUSAL (D-18, §10.5):** adaptive or variable frame
//! sampling is refused forever. The type system encodes it: a
//! [`RationalTime`] is only constructible as a whole number of frames
//! over the clock's fps — there is no API that emits an off-grid sample.
//!
//! Behavior Note **BN-02** documents the deliberate divergence: frame
//! counts can differ by one from the Python engine exactly where binary
//! `f64` representation makes the requested duration land off its decimal
//! intent (e.g. `0.1` at 30 fps is 4 frames here — the f64 is strictly
//! greater than 1/10, and three frames genuinely do not cover it — where
//! float `arange` happens to produce 3).

/// Exact rational time: `frames / fps` seconds. The only time values the
/// engine can express are on the frame grid — off-grid time is
/// unrepresentable by construction.
#[derive(Debug, Clone, Copy)]
pub struct RationalTime {
    frames: i64,
    fps: u32,
}

impl RationalTime {
    /// The zero instant on a given grid.
    #[must_use]
    pub fn zero(fps: u32) -> Self {
        Self { frames: 0, fps }
    }

    #[must_use]
    pub fn frames(&self) -> i64 {
        self.frames
    }

    #[must_use]
    pub fn fps(&self) -> u32 {
        self.fps
    }

    /// The value in seconds, converted with a single division — the one
    /// rounding at the consumer boundary (§6.1).
    #[must_use]
    pub fn to_f64(&self) -> f64 {
        self.frames as f64 / f64::from(self.fps)
    }

    /// Exact comparison against an `f64` number of seconds (no rounding:
    /// cross-multiplied in wide integers).
    #[must_use]
    pub fn cmp_seconds(&self, seconds: f64) -> Option<std::cmp::Ordering> {
        if !seconds.is_finite() {
            return None;
        }
        // self.frames / fps  vs  m·2^e  ⇔  frames·2^-e vs m·fps (e < 0).
        let (m, e, sign) = decompose(seconds);
        let lhs_sign = self.frames.signum();
        let rhs_sign = if m == 0 { 0 } else { sign };
        if lhs_sign != rhs_sign {
            return Some(lhs_sign.cmp(&(rhs_sign)));
        }
        if lhs_sign == 0 && rhs_sign == 0 {
            return Some(std::cmp::Ordering::Equal);
        }
        // Compare |frames|·2^max(-e,0) vs m·fps·2^max(e,0), then apply sign.
        let lhs = i128::from(self.frames.unsigned_abs());
        let rhs = i128::from(m) * i128::from(self.fps);
        let ord = shifted_cmp(lhs, rhs, e);
        Some(if lhs_sign < 0 { ord.reverse() } else { ord })
    }
}

impl std::ops::Add<i64> for RationalTime {
    type Output = Self;

    /// Advance by whole frames — the only arithmetic the grid admits.
    fn add(self, frames: i64) -> Self {
        Self {
            frames: self.frames + frames,
            fps: self.fps,
        }
    }
}

impl PartialEq for RationalTime {
    fn eq(&self, other: &Self) -> bool {
        i128::from(self.frames) * i128::from(other.fps)
            == i128::from(other.frames) * i128::from(self.fps)
    }
}

impl Eq for RationalTime {}

impl PartialOrd for RationalTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RationalTime {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (i128::from(self.frames) * i128::from(other.fps))
            .cmp(&(i128::from(other.frames) * i128::from(self.fps)))
    }
}

/// (mantissa, exponent, sign) with value = sign · m · 2^e for finite x.
fn decompose(x: f64) -> (u64, i32, i64) {
    let bits = x.to_bits();
    let sign = if bits >> 63 == 1 { -1 } else { 1 };
    let biased = ((bits >> 52) & 0x7ff) as i32;
    let frac = bits & 0x000f_ffff_ffff_ffff;
    if biased == 0 {
        (frac, -1074, sign)
    } else {
        (frac | (1 << 52), biased - 1075, sign)
    }
}

/// Compare `a` vs `b · 2^k` for positive `a`, `b` and any `k ≥ 0`,
/// overflow-free: bit lengths decide unless they tie, and a tie means the
/// shift fits in i128.
fn cmp_with_shift(a: i128, b: i128, k: u32) -> std::cmp::Ordering {
    let len_a = 128 - a.leading_zeros();
    let len_b = (128 - b.leading_zeros()).saturating_add(k);
    if len_a != len_b {
        return len_a.cmp(&len_b);
    }
    // Equal bit lengths: len_b ≤ 127, so b << k cannot overflow.
    a.cmp(&(b << k))
}

/// Compare `lhs` vs `rhs · 2^e` for positive operands and any f64 exponent.
fn shifted_cmp(lhs: i128, rhs: i128, e: i32) -> std::cmp::Ordering {
    if e >= 0 {
        cmp_with_shift(lhs, rhs, e as u32)
    } else {
        cmp_with_shift(rhs, lhs, (-e) as u32).reverse()
    }
}

/// Errors from the clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockError {
    /// fps must be nonzero.
    ZeroFps,
    /// run_time must be finite (NaN/±inf refused, never silently coerced).
    NonFiniteRunTime,
    /// The requested duration exceeds what an i64 frame index can count.
    RunTimeTooLong,
    /// A clock can only advance, never move to a negative earlier frame.
    NegativeFrameAdvance,
    /// The current frame plus an otherwise valid advance exceeds `i64`.
    FrameCounterOverflow,
}

impl std::fmt::Display for ClockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZeroFps => write!(f, "fps must be nonzero"),
            Self::NonFiniteRunTime => write!(f, "run_time must be finite"),
            Self::RunTimeTooLong => write!(f, "run_time exceeds the frame counter's range"),
            Self::NegativeFrameAdvance => write!(f, "frame advance must be non-negative"),
            Self::FrameCounterOverflow => write!(f, "frame advance exceeds the counter's range"),
        }
    }
}

impl std::error::Error for ClockError {}

/// `ceil(run_time · fps)` on the exact rational value of the f64 —
/// the segment frame count (0 for nonpositive durations).
fn frames_covering(run_time: f64, fps: u32) -> Result<i64, ClockError> {
    if !run_time.is_finite() {
        return Err(ClockError::NonFiniteRunTime);
    }
    if run_time <= 0.0 {
        return Ok(0);
    }
    let (m, e, _) = decompose(run_time);
    let product = i128::from(m) * i128::from(fps);
    if product == 0 {
        return Ok(0);
    }
    let n = if e >= 0 {
        if e >= 40 || (product << e) > i128::from(i64::MAX) {
            return Err(ClockError::RunTimeTooLong);
        }
        product << e
    } else {
        let s = -e;
        if s >= 127 {
            1 // a positive duration below every grid step still takes one frame
        } else {
            ((product - 1) >> s) + 1
        }
    };
    i64::try_from(n).map_err(|_| ClockError::RunTimeTooLong)
}

/// One emitted sample of a segment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameSample {
    /// 1-based frame number within the segment (there is no zero-alpha
    /// emission frame).
    pub frame: i64,
    /// Exact segment-local time `frame / fps`.
    pub time: RationalTime,
    /// `time / run_time`, clamped to `[0, 1]` (the Reference's clip).
    pub alpha: f64,
}

/// The sampling plan for one `play()`/`wait()` segment.
#[derive(Debug, Clone, Copy)]
pub struct FrameSegment {
    fps: u32,
    n_frames: i64,
    run_time: f64,
}

impl FrameSegment {
    /// Number of frames the segment emits.
    #[must_use]
    pub fn n_frames(&self) -> i64 {
        self.n_frames
    }

    /// The requested duration in seconds (as received; the grid end may
    /// exceed it — upward rounding).
    #[must_use]
    pub fn run_time(&self) -> f64 {
        self.run_time
    }

    /// The exact grid time of the segment's end (`n_frames / fps`), which
    /// is `>= run_time` for positive durations.
    #[must_use]
    pub fn end_time(&self) -> RationalTime {
        RationalTime {
            frames: self.n_frames,
            fps: self.fps,
        }
    }

    /// Return one 1-based sample without walking the preceding frame grid.
    ///
    /// Resumable front ends use this to prepare exactly one frame, release
    /// their arena borrow for a host-language updater phase, then resume the
    /// same sample for capture. Keeping the lookup O(1) avoids turning that
    /// release window into quadratic segment work.
    #[must_use]
    pub fn sample(&self, frame: i64) -> Option<FrameSample> {
        if !(1..=self.n_frames).contains(&frame) {
            return None;
        }
        let time = RationalTime {
            frames: frame,
            fps: self.fps,
        };
        let raw = time.to_f64() / self.run_time;
        Some(FrameSample {
            frame,
            time,
            alpha: raw.clamp(0.0, 1.0),
        })
    }

    /// The emitted samples, in frame order. Alphas are nondecreasing, the
    /// first is strictly positive, the last is exactly 1 whenever the grid
    /// end reaches or exceeds `run_time`.
    pub fn samples(&self) -> impl Iterator<Item = FrameSample> + '_ {
        (1..=self.n_frames).map(|frame| {
            self.sample(frame)
                .expect("the iterator only produces in-range frame indices")
        })
    }
}

/// The clock: an i64 frame counter over fps. Time derives; it never
/// accumulates.
#[derive(Debug, Clone, Copy)]
pub struct RationalFrameClock {
    fps: u32,
    frames_elapsed: i64,
}

impl RationalFrameClock {
    pub fn new(fps: u32) -> Result<Self, ClockError> {
        if fps == 0 {
            return Err(ClockError::ZeroFps);
        }
        Ok(Self {
            fps,
            frames_elapsed: 0,
        })
    }

    #[must_use]
    pub fn fps(&self) -> u32 {
        self.fps
    }

    /// The current scene time — exact, derived, drift-free.
    #[must_use]
    pub fn now(&self) -> RationalTime {
        RationalTime {
            frames: self.frames_elapsed,
            fps: self.fps,
        }
    }

    /// The frame step, exactly `1 / fps`.
    #[must_use]
    pub fn dt(&self) -> RationalTime {
        RationalTime {
            frames: 1,
            fps: self.fps,
        }
    }

    /// Advance by emitted frames (one per capture; a consumer that stops a
    /// wait early — `wait_until` — simply advances by fewer).
    ///
    /// # Errors
    /// [`ClockError::NegativeFrameAdvance`] for a negative delta, or
    /// [`ClockError::FrameCounterOverflow`] if the resulting frame cannot be
    /// represented.
    pub fn advance_frames(&mut self, n: i64) -> Result<(), ClockError> {
        if n < 0 {
            return Err(ClockError::NegativeFrameAdvance);
        }
        self.frames_elapsed = self
            .frames_elapsed
            .checked_add(n)
            .ok_or(ClockError::FrameCounterOverflow)?;
        Ok(())
    }

    /// Plan a segment of `run_time` seconds on this clock's grid.
    pub fn segment(&self, run_time: f64) -> Result<FrameSegment, ClockError> {
        let n_frames = frames_covering(run_time, self.fps)?;
        self.frames_elapsed
            .checked_add(n_frames)
            .ok_or(ClockError::FrameCounterOverflow)?;
        Ok(FrameSegment {
            fps: self.fps,
            n_frames,
            run_time,
        })
    }
}
