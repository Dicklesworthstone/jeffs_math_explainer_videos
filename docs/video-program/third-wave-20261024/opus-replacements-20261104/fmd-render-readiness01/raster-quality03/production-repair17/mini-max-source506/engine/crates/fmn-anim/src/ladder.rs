//! Declared updater operations for the Python acceleration ladder (§15.2
//! Rev 4, fm-zoi).
//!
//! Rungs 2 (array updater) and 3 (native updater) of the binding-tax
//! ladder execute one of these declared operations instead of a Python
//! callable. The declared class is deliberately narrow — a single
//! elementwise, per-record column transform over one RecordBuffer field —
//! so that a Python updater (rung 0), one vectorized RecordBuffer
//! operation (rung 2), and an engine-executable native updater (rung 3)
//! can be *bit-equal* on every frame.
//!
//! # Bit-equality contract
//!
//! Bit-equality is achievable only because the arithmetic is pinned:
//!
//! - every lane is widened from `f32` to `f64` (exact), all arithmetic is
//!   IEEE-754 binary64, and the result is stored back with a single
//!   round-to-nearest-even `f64 -> f32` conversion — exactly what the
//!   bridge's `set_field` does with Python floats;
//! - only the four correctly rounded operations (`+`, `-`, `*`, `/`) plus
//!   the exact `floor`/`abs` appear in any operation — no transcendental
//!   functions, whose libm results may differ between Python, NumPy, and
//!   Rust on some hosts;
//! - the oscillators are therefore a triangle wave (scale pulse) and a
//!   sawtooth (color ramp), both pure arithmetic.
//!
//! A rung-0 reference updater written against this contract (compute in
//! Python `float`, store through `set_field`) produces bit-identical
//! RecordBuffer state on every frame.
//!
//! # Time model
//!
//! Each [`DeclaredUpdater`] owns a private time `t` starting at `0.0`.
//! Every dispatch computes the transform at the current `t` and *then*
//! advances `t += dt`. Frame 0 of both oscillators is therefore an
//! identity/initial state (triangle starts at zero, sawtooth at the `from`
//! endpoint), and no registration-time `dt = 0` pass can double-apply a
//! nonzero step.

use fmn_mobject::RecordBuffer;
use std::fmt;

/// One declared elementwise column transform.
///
/// Parameter vectors must match the target field's lane width; that is
/// validated at construction against the declared width and again at
/// application against the live schema.
#[derive(Clone, Debug, PartialEq)]
pub enum DeclaredOp {
    /// Position-shifter class: `lane += velocity[lane] * dt`.
    Shift {
        /// Target field (conventionally `point`).
        field: String,
        /// Per-lane velocity, one entry per field lane.
        velocity: Vec<f64>,
    },
    /// Scale-pulser class: `lane = center + (lane - center) * s(t)` where
    /// `s(t) = 1 + amplitude * triangle(t / period)` and
    /// `triangle(u) = 1 - |2 * frac(u) - 1|`.
    ScalePulse {
        /// Target field (conventionally `point`).
        field: String,
        /// Per-lane pulse center, one entry per field lane.
        center: Vec<f64>,
        /// Peak scale excursion (0.25 pulses `s` over `0.75..=1.25`).
        amplitude: f64,
        /// Oscillation period in seconds; strictly positive.
        period: f64,
    },
    /// Color-ramp class: `lane = from + (to - from) * alpha(t)` where
    /// `alpha(t) = frac(t / period)`.
    ColorRamp {
        /// Target field (conventionally `rgba`).
        field: String,
        /// Per-lane ramp start, one entry per field lane.
        from: Vec<f64>,
        /// Per-lane ramp end, one entry per field lane.
        to: Vec<f64>,
        /// Ramp period in seconds; strictly positive.
        period: f64,
    },
}

impl DeclaredOp {
    /// The field this operation transforms.
    #[must_use]
    pub fn field(&self) -> &str {
        match self {
            Self::Shift { field, .. }
            | Self::ScalePulse { field, .. }
            | Self::ColorRamp { field, .. } => field,
        }
    }

    /// The declared parameter width; must equal the field's lane width.
    #[must_use]
    pub fn width(&self) -> usize {
        match self {
            Self::Shift { velocity, .. } => velocity.len(),
            Self::ScalePulse { center, .. } => center.len(),
            Self::ColorRamp { from, .. } => from.len(),
        }
    }

    /// The stable class tag used by the ladder corpus and the PG-8 rig.
    #[must_use]
    pub fn class_tag(&self) -> &'static str {
        match self {
            Self::Shift { .. } => "shift",
            Self::ScalePulse { .. } => "scale-pulse",
            Self::ColorRamp { .. } => "color-ramp",
        }
    }

    /// Validate parameter finiteness and shape. Field-width agreement is
    /// checked separately against a concrete schema.
    ///
    /// # Errors
    /// [`LadderError`] naming the first malformed parameter.
    pub fn validate(&self) -> Result<(), LadderError> {
        if self.field().is_empty() {
            return Err(LadderError::EmptyField);
        }
        if self.width() == 0 {
            return Err(LadderError::EmptyParameters);
        }
        let finite = |values: &[f64]| values.iter().all(|v| v.is_finite());
        match self {
            Self::Shift { velocity, .. } if !finite(velocity) => {
                Err(LadderError::NonFiniteParameter)
            }
            Self::ScalePulse {
                center,
                amplitude,
                period,
                ..
            } => {
                if !finite(center) || !amplitude.is_finite() {
                    return Err(LadderError::NonFiniteParameter);
                }
                if !period.is_finite() || *period <= 0.0 {
                    return Err(LadderError::NonPositivePeriod);
                }
                Ok(())
            }
            Self::ColorRamp {
                from, to, period, ..
            } => {
                if from.len() != to.len() {
                    return Err(LadderError::ParameterWidthMismatch);
                }
                if !finite(from) || !finite(to) {
                    return Err(LadderError::NonFiniteParameter);
                }
                if !period.is_finite() || *period <= 0.0 {
                    return Err(LadderError::NonPositivePeriod);
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

/// Malformed declared operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LadderError {
    /// The target field name is empty.
    EmptyField,
    /// A parameter vector is empty.
    EmptyParameters,
    /// A parameter is NaN or infinite.
    NonFiniteParameter,
    /// An oscillator period is non-finite or not strictly positive.
    NonPositivePeriod,
    /// `from`/`to` widths disagree.
    ParameterWidthMismatch,
}

impl fmt::Display for LadderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField => f.write_str("declared operation field name is empty"),
            Self::EmptyParameters => f.write_str("declared operation parameter vector is empty"),
            Self::NonFiniteParameter => {
                f.write_str("declared operation parameter is NaN or infinite")
            }
            Self::NonPositivePeriod => {
                f.write_str("declared operation period must be finite and positive")
            }
            Self::ParameterWidthMismatch => {
                f.write_str("color-ramp `from` and `to` widths disagree")
            }
        }
    }
}

impl std::error::Error for LadderError {}

/// Triangle wave over the unit interval: `1 - |2 * frac(u) - 1|`, in
/// `[0, 1]`. Pure arithmetic; bit-portable across Python, NumPy, and Rust.
#[must_use]
pub fn triangle(u: f64) -> f64 {
    let frac = u - u.floor();
    1.0 - (2.0 * frac - 1.0).abs()
}

/// Sawtooth over the unit interval: `frac(u)`, in `[0, 1)`.
#[must_use]
pub fn sawtooth(u: f64) -> f64 {
    u - u.floor()
}

/// A declared operation plus its private time base.
///
/// The time base is what makes rung 2 and rung 3 interchangeable with a
/// stateful rung-0 Python closure: all rungs advance `t` by `dt` after
/// computing at the current `t`.
#[derive(Clone, Debug)]
pub struct DeclaredUpdater {
    op: DeclaredOp,
    t: f64,
}

impl DeclaredUpdater {
    /// Validate and wrap a declared operation with `t = 0`.
    ///
    /// # Errors
    /// [`LadderError`] for malformed parameters.
    pub fn new(op: DeclaredOp) -> Result<Self, LadderError> {
        op.validate()?;
        Ok(Self { op, t: 0.0 })
    }

    /// The declared operation.
    #[must_use]
    pub fn op(&self) -> &DeclaredOp {
        &self.op
    }

    /// The private time base; advances by `dt` after every applied frame.
    #[must_use]
    pub fn time(&self) -> f64 {
        self.t
    }

    /// Compute the transform at the current time and write it back as one
    /// vectorized `write_range` over the whole field column, then advance
    /// the time base by `dt`.
    ///
    /// Returns `false` — without mutating the buffer or the time base —
    /// when the field is absent from the schema or its lane width disagrees
    /// with the declared parameters. Callers are expected to validate the
    /// schema once at registration, so a `false` here indicates schema
    /// drift (for example a rebuilt mobject) and is a documented no-op.
    pub fn apply(&mut self, buffer: &mut RecordBuffer, dt: f64) -> bool {
        let field = self.op.field();
        let width = self.op.width();
        if buffer.schema().field_width(field) != Some(width) {
            return false;
        }
        let Some(column) = buffer.read_column(field) else {
            return false;
        };
        let next = self.compute_column(&column, dt);
        if next.len() != column.len() {
            return false;
        }
        if !next.is_empty() && !buffer.write_range(field, 0, &next) {
            return false;
        }
        self.t += dt;
        true
    }

    /// The whole transformed column at the current time, computed lane by
    /// lane in binary64 and narrowed once. Exposed for the bit-equality
    /// corpus: the Python reference performs the same per-lane expression.
    #[must_use]
    fn compute_column(&self, column: &[f32], dt: f64) -> Vec<f32> {
        let width = self.op.width();
        let mut next = Vec::with_capacity(column.len());
        for chunk in column.chunks_exact(width) {
            match &self.op {
                DeclaredOp::Shift { velocity, .. } => {
                    for (lane, &x) in chunk.iter().enumerate() {
                        next.push((f64::from(x) + velocity[lane] * dt) as f32);
                    }
                }
                DeclaredOp::ScalePulse {
                    center,
                    amplitude,
                    period,
                    ..
                } => {
                    let scale = 1.0 + amplitude * triangle(self.t / period);
                    for (lane, &x) in chunk.iter().enumerate() {
                        let c = center[lane];
                        next.push((c + (f64::from(x) - c) * scale) as f32);
                    }
                }
                DeclaredOp::ColorRamp {
                    from, to, period, ..
                } => {
                    let alpha = sawtooth(self.t / period);
                    for (lane, &x) in chunk.iter().enumerate() {
                        // The current lane participates so the transform is
                        // defined even off the ramp endpoints: at integer
                        // periods alpha wraps to zero and the lane snaps
                        // back to `from`, exactly like the Python sawtooth.
                        let _ = x;
                        next.push((from[lane] + (to[lane] - from[lane]) * alpha) as f32);
                    }
                }
            }
        }
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmn_mobject::RecordSchema;

    fn buffer() -> RecordBuffer {
        let schema =
            RecordSchema::new(&[("point", 3), ("rgba", 4)], &["point"], &["point"]).unwrap();
        let mut buffer = RecordBuffer::new(schema, 2).unwrap();
        assert!(buffer.write(0, "point", &[1.0, 2.0, 3.0]));
        assert!(buffer.write(1, "point", &[-1.0, 0.5, 4.0]));
        assert!(buffer.write(0, "rgba", &[0.0, 0.0, 0.0, 1.0]));
        assert!(buffer.write(1, "rgba", &[1.0, 1.0, 1.0, 1.0]));
        buffer
    }

    #[test]
    fn shift_matches_hand_computed_binary64() {
        let mut buffer = buffer();
        let mut updater = DeclaredUpdater::new(DeclaredOp::Shift {
            field: "point".to_owned(),
            velocity: vec![0.1, -0.2, 0.3],
        })
        .expect("valid shift");
        assert!(updater.apply(&mut buffer, 0.25));
        // Hand-computed: f64 lane + velocity * dt, narrowed once.
        let expected = [
            (f64::from(1.0f32) + 0.1 * 0.25) as f32,
            (f64::from(2.0f32) + -0.2 * 0.25) as f32,
            (f64::from(3.0f32) + 0.3 * 0.25) as f32,
        ];
        assert_eq!(buffer.read(0, "point").expect("point"), expected);
        assert_eq!(updater.time(), 0.25);
    }

    #[test]
    fn scale_pulse_starts_at_identity_and_pulses() {
        let mut buffer = buffer();
        let mut updater = DeclaredUpdater::new(DeclaredOp::ScalePulse {
            field: "point".to_owned(),
            center: vec![0.0, 0.0, 0.0],
            amplitude: 0.5,
            period: 2.0,
        })
        .expect("valid pulse");
        // t = 0: triangle(0) = 0, scale = 1 — identity, bit-for-bit.
        let before = buffer.read_column("point").expect("point");
        assert!(updater.apply(&mut buffer, 0.5));
        assert_eq!(buffer.read_column("point").expect("point"), before);
        // t = 0.5: u = 0.25, triangle = 0.5, scale = 1.25.
        assert!(updater.apply(&mut buffer, 0.0));
        let column = buffer.read_column("point").expect("point");
        assert_eq!(column[0], (f64::from(before[0]) * 1.25) as f32);
        assert_eq!(column[3], (f64::from(before[3]) * 1.25) as f32);
    }

    #[test]
    fn color_ramp_interpolates_and_wraps() {
        let mut buffer = buffer();
        let mut updater = DeclaredUpdater::new(DeclaredOp::ColorRamp {
            field: "rgba".to_owned(),
            from: vec![0.0, 0.0, 0.0, 1.0],
            to: vec![1.0, 0.5, 0.25, 1.0],
            period: 1.0,
        })
        .expect("valid ramp");
        assert!(updater.apply(&mut buffer, 0.5));
        // t = 0: alpha = 0 → exactly `from`.
        assert_eq!(
            buffer.read(0, "rgba").expect("rgba"),
            vec![0.0, 0.0, 0.0, 1.0]
        );
        assert!(updater.apply(&mut buffer, 0.0));
        // t = 0.5: alpha = 0.5 → midpoint.
        let expected: Vec<f32> = [0.0, 0.0, 0.0, 1.0]
            .iter()
            .zip([1.0, 0.5, 0.25, 1.0].iter())
            .map(|(a, b)| (a + (b - a) * 0.5) as f32)
            .collect();
        assert_eq!(buffer.read(1, "rgba").expect("rgba"), expected);
        // t = 1.0: alpha wraps to 0 → exactly `from` again.
        assert!(updater.apply(&mut buffer, 0.5));
        assert!(updater.apply(&mut buffer, 0.0));
        assert_eq!(
            buffer.read(0, "rgba").expect("rgba"),
            vec![0.0, 0.0, 0.0, 1.0]
        );
    }

    #[test]
    fn malformed_operations_are_rejected() {
        assert_eq!(
            DeclaredUpdater::new(DeclaredOp::Shift {
                field: String::new(),
                velocity: vec![1.0],
            })
            .expect_err("empty field"),
            LadderError::EmptyField
        );
        assert_eq!(
            DeclaredUpdater::new(DeclaredOp::Shift {
                field: "point".to_owned(),
                velocity: Vec::new(),
            })
            .expect_err("empty velocity"),
            LadderError::EmptyParameters
        );
        assert_eq!(
            DeclaredUpdater::new(DeclaredOp::ScalePulse {
                field: "point".to_owned(),
                center: vec![0.0],
                amplitude: f64::NAN,
                period: 1.0,
            })
            .expect_err("NaN amplitude"),
            LadderError::NonFiniteParameter
        );
        assert_eq!(
            DeclaredUpdater::new(DeclaredOp::ColorRamp {
                field: "rgba".to_owned(),
                from: vec![0.0],
                to: vec![1.0],
                period: 0.0,
            })
            .expect_err("zero period"),
            LadderError::NonPositivePeriod
        );
        assert_eq!(
            DeclaredUpdater::new(DeclaredOp::ColorRamp {
                field: "rgba".to_owned(),
                from: vec![0.0, 0.0],
                to: vec![1.0],
                period: 1.0,
            })
            .expect_err("width mismatch"),
            LadderError::ParameterWidthMismatch
        );
    }

    #[test]
    fn schema_drift_is_a_documented_no_op() {
        let mut buffer = buffer();
        let mut updater = DeclaredUpdater::new(DeclaredOp::Shift {
            field: "missing".to_owned(),
            velocity: vec![1.0, 2.0, 3.0],
        })
        .expect("valid shift");
        assert!(!updater.apply(&mut buffer, 1.0));
        assert_eq!(
            updater.time(),
            0.0,
            "a rejected frame does not advance time"
        );
        let mut wrong_width = DeclaredUpdater::new(DeclaredOp::Shift {
            field: "point".to_owned(),
            velocity: vec![1.0],
        })
        .expect("valid shift");
        assert!(!wrong_width.apply(&mut buffer, 1.0));
    }

    #[test]
    fn oscillators_are_pure_arithmetic() {
        assert_eq!(triangle(0.0), 0.0);
        assert_eq!(triangle(0.25), 0.5);
        assert_eq!(triangle(0.5), 1.0);
        assert_eq!(triangle(0.75), 0.5);
        assert_eq!(triangle(1.0), 0.0);
        assert_eq!(triangle(-0.25), 0.5);
        assert_eq!(sawtooth(0.0), 0.0);
        assert_eq!(sawtooth(0.75), 0.75);
        assert_eq!(sawtooth(1.0), 0.0);
        assert_eq!(sawtooth(2.25), 0.25);
    }
}
