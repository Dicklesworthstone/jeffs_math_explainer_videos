#!/usr/bin/env python3
"""Integrate the shared 3D path law, validating every edited source anchor."""
from pathlib import Path
import hashlib
pending = {}
def replace(name, old, new):
    text = pending.get(name, Path(name).read_text())
    if new in text:
        return
    if text.count(old) != 1:
        raise SystemExit(f"changed source anchor in {name}")
    pending[name] = text.replace(old, new, 1)

name = 'crates/fmn-anim/src/transform.rs'
text = Path(name).read_text()
new = '''#[path = "motion_path.rs"]
mod motion_path;
pub use motion_path::{PathFunc, STRAIGHT_PATH_THRESHOLD};
use motion_path::interpolate_placement;

// ------------------------------------------------------------- lerp core

'''
if 'mod motion_path;' not in text:
    a = text.index('/// `STRAIGHT_PATH_THRESHOLD`')
    b = text.index("/// The Reference's `Mobject.interpolate`", a)
    if hashlib.sha256(text[a:b].encode()).hexdigest() != '31d2d261291d3058206360b4a73311e58816e4858d753511f2a8de66b945cdcb':
        raise SystemExit('path implementation changed; refusing overwrite')
    pending[name] = text[:a] + new + text[b:]
replace(name, '    fn setup(&mut self, stage: &mut Stage) -> Result<(), AnimError> {\n        let mobject = self.state.mobject();', '    fn setup(&mut self, stage: &mut Stage) -> Result<(), AnimError> {\n        self.path.validate()?;\n        let mobject = self.state.mobject();')
replace(name, '''//! - [`PathFunc`] carries the exact path formulas: `straight_path` is the
//!   plain lerp; `path_along_arc` rotates the start radius about the
//!   computed arc center (`center + cos(αθ)·r + sin(αθ)·(axis×r)`), and a
//!   scalar `|arc_angle| <` [`STRAIGHT_PATH_THRESHOLD`] collapses to
//!   straight, exactly as the Reference's early return.
''', '''//! - [`PathFunc`] carries the shared path law: straight lerp or rotation
//!   about the computed arc center with linear axial motion (a helix in
//!   3D; BN-12). A scalar `|arc_angle| <` [`STRAIGHT_PATH_THRESHOLD`]
//!   collapses to straight, as in the Reference's early return.
''')
replace('crates/fmn-anim/src/animation.rs', '    InvalidFramePhase(&\'static str),\n', '    InvalidFramePhase(&\'static str),\n    /// A non-finite point-path parameter, rejected before Stage mutation.\n    InvalidPath(&\'static str),\n')
replace('crates/fmn-anim/src/animation.rs', '            Self::InvalidFramePhase(message) => {\n', '            Self::InvalidPath(message) => write!(f, "invalid animation path: {message}"),\n            Self::InvalidFramePhase(message) => {\n')
replace('docs/behavior_notes/BN-12-animation-contract.md', '''## Staging boundaries (not divergences)

Two precise, named errors mark where the Transform family (fm-cye) takes
over from the fm-67a carrier; both disappear as capabilities when it lands:

- `AnimError::PathArcUnsupported` — a recorded `path_arc` on a built
  `.animate` chain (arcs are Transform's `path_func` mechanism). Never a
  silent straight line.
- `AnimError::UnalignedFamilies` — a source/target pair that structurally
  diverged between build and play (alignment of heterogeneous pairs is
  `align_data`). Never a partial lerp.
''', '''## 4. Three-dimensional arc paths preserve axial motion

The native `PathFunc::Arc` uses full Rodrigues rotation perpendicular to its
axis and linearly interpolates displacement along that axis. Different axial
coordinates therefore follow a helix with exact start/end values rather than
the old two-term rotation, which could miss its requested endpoint. Finite
nonzero axes are normalized without overflow or underflow; a zero axis still
means OUT. The existing small-angle straight-path threshold is unchanged.

Non-finite arc angles and axes are refused as `AnimError::InvalidPath` before
Transform alignment or snapshot allocation. The affine lift interpolates its
linear columns separately from translation, so a large world origin cannot
round away local shape through translated-basis subtraction.

**Migration:** spatial arc motion with displacement parallel to the axis now
has a linear axial component. Planar arcs keep their circular geometry. Code
must not rely on malformed non-finite arc parameters reaching interpolation.

## Complete native method transforms

Native `.animate` recordings use the same Transform implementation as explicit
transforms: family/record alignment, `path_arc` and `path_arc_axis`, matching
field locks, tracker/uniform interpolation, and host-view materialization.
The former staging refusals `PathArcUnsupported` and `UnalignedFamilies` remain
as enum variants for source compatibility, but the method carrier no longer
emits them. A diverged target is aligned privately rather than mutated.
''')
new_files = {
'crates/fmn-anim/src/motion_path.rs': r'''//! The shared point-path law and its translation-independent affine lift.
use fmn_core::types::Vec3;
use fmn_mobject::Placement;

use crate::animation::AnimError;

/// Arc angles below this threshold use the straight path (the Reference rule).
pub const STRAIGHT_PATH_THRESHOLD: f64 = 0.01;

fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn dot(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn unit_axis(axis: Vec3) -> Vec3 {
    let scale = axis.iter().fold(0.0_f64, |largest, value| largest.max(value.abs()));
    if scale == 0.0 {
        return [0.0, 0.0, 1.0];
    }
    // Scale before squaring: every finite nonzero axis has the same direction
    // at 1e-300 and 1e300, without underflow to the OUT fallback or overflow.
    let scaled = axis.map(|value| value / scale);
    let norm = dot(scaled, scaled).sqrt();
    scaled.map(|value| value / norm)
}

fn nan_to_num(value: f64) -> f64 {
    if value.is_nan() {
        0.0
    } else if value == f64::INFINITY {
        f64::MAX
    } else if value == f64::NEG_INFINITY {
        f64::MIN
    } else {
        value
    }
}

/// A point path as composable, journal-able data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathFunc {
    /// `(1-alpha) * start + alpha * end`.
    Straight,
    /// A circular path perpendicular to `axis`, with linear axial motion.
    /// Unequal axial coordinates therefore trace a helix, not a broken arc.
    Arc {
        /// Signed arc angle in radians.
        angle: f64,
        /// Finite rotation axis, normalized at eval; zero means OUT.
        axis: Vec3,
    },
}

impl PathFunc {
    /// Preserve the Reference's small-angle straight-path threshold.
    #[must_use]
    pub fn from_path_arc(angle: f64, axis: Vec3) -> Self {
        if angle.abs() < STRAIGHT_PATH_THRESHOLD {
            Self::Straight
        } else {
            Self::Arc { angle, axis }
        }
    }

    /// Reject non-finite arc parameters before an animation mutates its Stage.
    ///
    /// # Errors
    /// [`AnimError::InvalidPath`] for a non-finite angle or axis component.
    pub fn validate(&self) -> Result<(), AnimError> {
        if let Self::Arc { angle, axis } = self {
            if !angle.is_finite() {
                return Err(AnimError::InvalidPath("arc angle must be finite"));
            }
            if axis.iter().any(|value| !value.is_finite()) {
                return Err(AnimError::InvalidPath("arc axis must be finite"));
            }
        }
        Ok(())
    }

    /// Evaluate one path. Arc endpoints are exact; its axial component lerps
    /// independently of its perpendicular rotation (BN-12).
    #[must_use]
    pub fn eval(&self, start: Vec3, end: Vec3, alpha: f64) -> Vec3 {
        match *self {
            Self::Straight => std::array::from_fn(|i| (1.0 - alpha) * start[i] + alpha * end[i]),
            Self::Arc { angle, axis } => {
                if angle.abs() < STRAIGHT_PATH_THRESHOLD {
                    return Self::Straight.eval(start, end, alpha);
                }
                if alpha == 0.0 || start == end {
                    return start;
                }
                if alpha == 1.0 {
                    return end;
                }
                let unit = unit_axis(axis);
                let half: Vec3 = std::array::from_fn(|i| (end[i] - start[i]) / 2.0);
                let tan_half = fmn_dmath::tan(angle / 2.0);
                let c = cross(unit, half);
                let center: Vec3 = std::array::from_fn(|i| start[i] + half[i] + nan_to_num(c[i] / tan_half));
                let radius: Vec3 = std::array::from_fn(|i| start[i] - center[i]);
                let perpendicular = cross(unit, radius);
                let sin_a = fmn_dmath::sin(alpha * angle);
                let cos_a = fmn_dmath::cos(alpha * angle);
                // Complete Rodrigues rotation, then translate along the axis.
                // The old two-term rotation dropped this projection, so a 3D
                // arc could fail to reach its requested endpoint.
                let axial = (1.0 - cos_a) * dot(unit, radius) + 2.0 * alpha * dot(unit, half);
                std::array::from_fn(|i| {
                    center[i] + cos_a * radius[i] + sin_a * perpendicular[i] + unit[i] * axial
                })
            }
        }
    }
}

/// Lift the point-path law to an affine map without subtracting two large
/// translated points. For fixed path/alpha the law is linear in its endpoint
/// vectors, so each matrix column can be evaluated independently of origin.
pub(super) fn interpolate_placement(from: Placement, to: Placement, alpha: f64, path: PathFunc) -> Placement {
    let origin = path.eval(from.translation(), to.translation(), alpha);
    let a = from.linear();
    let b = to.linear();
    let mut linear = [[0.0; 3]; 3];
    for column in 0..3 {
        let value = path.eval(
            [a[0][column], a[1][column], a[2][column]],
            [b[0][column], b[1][column], b[2][column]],
            alpha,
        );
        for row in 0..3 {
            linear[row][column] = value[row];
        }
    }
    Placement::new(linear, origin)
}
''',
'crates/fmn-anim/tests/arc_motion.rs': r'''//! Independent mathematical oracles for 3D arc paths and large translations.
use fmn_anim::{AnimConfig, AnimError, Animation, PathFunc, RateFunc, Transform};
use fmn_mobject::{Mobject, Placement, Stage};

fn close(actual: [f64; 3], expected: [f64; 3]) {
    for axis in 0..3 {
        assert!((actual[axis] - expected[axis]).abs() < 2e-12, "{actual:?} != {expected:?}");
    }
}

#[test]
fn quarter_arc_with_axial_motion_is_an_endpoint_exact_helix() {
    let path = PathFunc::from_path_arc(std::f64::consts::FRAC_PI_2, [0.0, 0.0, 1.0]);
    let a = [1.0, 0.0, 2.0];
    let b = [0.0, 1.0, 6.0];
    assert_eq!(path.eval(a, b, 0.0), a);
    assert_eq!(path.eval(a, b, 1.0), b);
    for i in 0..=16 {
        let alpha = f64::from(i) / 16.0;
        let angle = alpha * std::f64::consts::FRAC_PI_2;
        close(path.eval(a, b, alpha), [fmn_dmath::cos(angle), fmn_dmath::sin(angle), 2.0 + 4.0 * alpha]);
    }
}

#[test]
fn purely_axial_displacement_is_linear_for_every_arc_angle() {
    for angle in [-3.0, -1.5, 0.1, 0.7, 2.0, 3.0] {
        let path = PathFunc::from_path_arc(angle, [0.0, 1.0, 0.0]);
        for i in 0..=16 {
            let alpha = f64::from(i) / 16.0;
            close(path.eval([3.0, -5.0, 2.0], [3.0, 7.0, 2.0], alpha), [3.0, -5.0 + 12.0 * alpha, 2.0]);
        }
    }
}

#[test]
fn axis_scale_and_reverse_motion_preserve_the_same_spatial_path() {
    let a = [1.0, -2.0, 3.0];
    let b = [-4.0, 7.0, 1.0];
    let axis = [1.0, 2.0, -3.0];
    let forward = PathFunc::from_path_arc(1.2, axis);
    let reverse = PathFunc::from_path_arc(-1.2, axis);
    for scale in [1e-300, 1.0, 1e300] {
        let scaled = PathFunc::from_path_arc(1.2, axis.map(|value| value * scale));
        for i in 0..=16 {
            let alpha = f64::from(i) / 16.0;
            let expected = forward.eval(a, b, alpha);
            close(scaled.eval(a, b, alpha), expected);
            close(reverse.eval(b, a, 1.0 - alpha), expected);
            let along_axis = expected[0] + 2.0 * expected[1] - 3.0 * expected[2];
            let expected_projection = (1.0 - alpha) * (a[0] + 2.0 * a[1] - 3.0 * a[2])
                + alpha * (b[0] + 2.0 * b[1] - 3.0 * b[2]);
            assert!((along_axis - expected_projection).abs() < 2e-12);
        }
    }
}

#[test]
fn stationary_points_and_zero_axis_keep_their_contracts() {
    let point = [0.1, -10.5, 3.0];
    let zero = PathFunc::from_path_arc(1.3, [0.0; 3]);
    let out = PathFunc::from_path_arc(1.3, [0.0, 0.0, 1.0]);
    for i in 0..=16 {
        let alpha = f64::from(i) / 16.0;
        assert_eq!(zero.eval(point, point, alpha), point);
        assert_eq!(zero.eval(point, [2.0; 3], alpha), out.eval(point, [2.0; 3], alpha));
    }
}

#[test]
fn large_world_origins_do_not_destroy_local_shape_during_transforms() {
    let huge = 18_014_398_509_481_984.0; // 2^54: adding one is rounded away.
    for path in [PathFunc::Straight, PathFunc::from_path_arc(1.2, [0.0, 0.0, 1.0])] {
        let mut stage = Stage::new();
        let source = stage.add(Mobject::from_points(&[[-1.0; 3], [1.0; 3]]));
        let target = stage.copy_family(source).unwrap();
        let a = Placement::from_translation([huge, -huge, huge]);
        let b = Placement::from_translation([huge + 64.0, -huge + 128.0, huge + 32.0]);
        stage.set_placement(source, a).unwrap();
        stage.set_placement(target, b).unwrap();
        let revision = stage.get(source).unwrap().buffer.field_revision("point");
        let points = stage.get_object_points(source).unwrap();
        let mut animation = Transform::new(source, target).with_path_func(path).with_config(AnimConfig {
            rate_func: RateFunc::linear(), ..AnimConfig::default()
        });
        animation.begin(&mut stage).unwrap();
        for alpha in [0.0, 0.25, 0.5, 0.75, 1.0] {
            animation.interpolate(&mut stage, alpha);
            let placement = stage.placement(source).unwrap();
            assert_eq!(placement.linear(), Placement::IDENTITY.linear());
            assert_eq!(placement.translation(), path.eval(a.translation(), b.translation(), alpha));
            assert_eq!(stage.get_object_points(source).unwrap(), points);
            assert_eq!(stage.get(source).unwrap().buffer.field_revision("point"), revision);
        }
        animation.finish(&mut stage);
        assert_eq!(stage.placement(source), Some(b));
    }
}

#[test]
fn invalid_arc_parameters_fail_before_family_alignment_or_copying() {
    for (angle, axis) in [(f64::NAN, [0.0; 3]), (f64::INFINITY, [0.0; 3]), (1.0, [f64::INFINITY, 0.0, 0.0]), (1.0, [0.0, f64::NAN, 0.0])] {
        let mut stage = Stage::new();
        let source = stage.add(Mobject::from_points(&[[0.0; 3]]));
        let target = stage.add(Mobject::from_points(&[[1.0; 3], [2.0; 3]]));
        let before = stage.snapshot().to_bytes().unwrap();
        let mut animation = Transform::new(source, target).with_path_arc(angle, axis);
        assert!(matches!(animation.begin(&mut stage), Err(AnimError::InvalidPath(_))));
        assert_eq!(stage.snapshot().to_bytes().unwrap(), before);
    }
}
''',
}
for name, text in new_files.items():
    if Path(name).exists() and Path(name).read_text() != text:
        raise SystemExit('new source already exists with different contents: ' + name)
    pending[name] = text
for name, text in pending.items():
    Path(name).write_text(text)
