//! The video's color roles (storyboard, "Color roles"): one color per idea
//! for the whole video, on top of the corpus's shared roles.

use explainer_kit::{Palette, hex};
use fmn::prelude::Srgb;

/// Pattern A's pulses.
pub fn pattern_a() -> Srgb {
    Palette::blue()
}

/// Pattern B's pulses.
pub fn pattern_b() -> Srgb {
    hex("#C792EA")
}

/// An agreement (a product of +1), and a block above the line.
pub fn agree() -> Srgb {
    Palette::green()
}

/// A disagreement (a product of −1), and a block below the line.
pub fn disagree() -> Srgb {
    Palette::red()
}

/// The echo: where the thing really is. Strokes and words only.
pub fn echo() -> Srgb {
    Palette::hero()
}

/// A pair that reaches across the middle of a doubled pattern (chapter
/// 03): its arc, and the ring on its block. Strokes only.
pub fn across() -> Srgb {
    Palette::teal()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_role_is_its_own_color() {
        let roles = [
            pattern_a(),
            pattern_b(),
            agree(),
            disagree(),
            echo(),
            across(),
            Palette::ink(),
            Palette::dim(),
            Palette::faint(),
        ];
        for (i, a) in roles.iter().enumerate() {
            for b in &roles[i + 1..] {
                assert_ne!(a, b);
            }
        }
    }
}
