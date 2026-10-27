//! The video's color roles (storyboard, "Color roles"): one color per idea
//! for the whole video, on top of the corpus's shared roles.

use explainer_kit::Palette;
use fmn::prelude::Srgb;

/// The coast: always this blue, however it is cut or zoomed.
pub fn coast() -> Srgb {
    Palette::blue()
}

/// The true length, and anything that traces it (01_2's dot and trail).
pub fn length() -> Srgb {
    Palette::green()
}

/// The dividers, their landings and their chords: a measurement that
/// comes out short.
pub fn dividers() -> Srgb {
    Palette::red()
}

/// A point where a straight edge meets the coast (01_3), crossings later.
pub fn touch() -> Srgb {
    Palette::gold()
}

/// The card, a panel a little lighter than the background. The ruler and
/// labels are `ink` and `dim`.
pub fn card() -> Srgb {
    Palette::faint()
}

/// The coin of known length, its rim and the rim laid out.
pub fn coin() -> Srgb {
    Palette::teal()
}

/// A straight stick (07) and its shadow.
pub fn stick() -> Srgb {
    Palette::purple()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_role_is_its_own_color() {
        let roles = [
            coast(),
            coin(),
            stick(),
            length(),
            dividers(),
            touch(),
            card(),
            Palette::hero(),
            Palette::ink(),
            Palette::dim(),
        ];
        for (i, a) in roles.iter().enumerate() {
            for b in &roles[i + 1..] {
                assert_ne!(a, b);
            }
        }
    }
}
