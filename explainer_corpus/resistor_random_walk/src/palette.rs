//! The video's color roles (storyboard, "Color roles"): one color per idea
//! for the whole video, on top of the corpus's shared roles. A voltage and a
//! chance are the same number here, so they share one scale: the bottom's
//! zero is blue, the top's one is gold, and every value in between is mixed
//! in proportion.

use explainer_kit::Palette;
use fmn::core::color::interpolate_color;
use fmn::prelude::Srgb;

use crate::model::Q;

/// The top: one volt, and a chance of one.
pub fn top() -> Srgb {
    Palette::gold()
}

/// The bottom: zero volts, and a chance of zero.
pub fn bottom() -> Srgb {
    Palette::blue()
}

/// A value between zero and one: a junction once its value is known, and a
/// value travelling along a wire.
pub fn value(q: Q) -> Srgb {
    interpolate_color(bottom(), top(), q.f64().clamp(0.0, 1.0))
}

/// The walker.
pub fn walker() -> Srgb {
    Palette::green()
}

/// A current (chapter 02): a wire's drop and the arrow of its flow. Text
/// and strokes only.
pub fn current() -> Srgb {
    Palette::teal()
}

/// Wires and the battery.
pub fn wire() -> Srgb {
    Palette::dim()
}

/// A middle junction whose value is not yet shown.
pub fn unknown() -> Srgb {
    Palette::faint()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_role_is_its_own_color() {
        let roles = [
            top(),
            bottom(),
            walker(),
            current(),
            wire(),
            unknown(),
            Palette::hero(),
            Palette::ink(),
        ];
        for (i, a) in roles.iter().enumerate() {
            for b in &roles[i + 1..] {
                assert_ne!(a, b);
            }
        }
    }

    /// The scale's ends are the top's and the bottom's colors, and the
    /// opening's two middle values sit strictly between them.
    #[test]
    fn the_value_scale_runs_from_the_bottom_to_the_top() {
        let close = |a: Srgb, b: Srgb| (a.r - b.r).abs() + (a.g - b.g).abs() + (a.b - b.b).abs() < 1e-9;
        assert!(close(value(Q::ZERO), bottom()));
        assert!(close(value(Q::ONE), top()));
        for q in [Q::new(2, 5), Q::new(3, 5)] {
            assert!(!close(value(q), bottom()));
            assert!(!close(value(q), top()));
        }
        assert!(!close(value(Q::new(2, 5)), value(Q::new(3, 5))));
    }
}
