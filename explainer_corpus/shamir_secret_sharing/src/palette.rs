//! The video's color roles (docs/storyboard-draft.md, "Color roles"): one
//! color per idea for the whole video, on top of the corpus's shared roles.
//! Only the roles an implemented chapter draws are here.

use explainer_kit::Palette;
use fmn::prelude::Srgb;

/// The secret: its card, the needle, the strip's lit ticks, the count.
pub fn secret() -> Srgb {
    Palette::hero()
}

/// A trustee, and the share it holds.
pub fn share() -> Srgb {
    Palette::blue()
}

/// A trio acting together.
pub fn trio() -> Srgb {
    Palette::teal()
}

/// Trustees 4 and 5, the pair who meet.
pub fn pair() -> Srgb {
    Palette::purple()
}

/// A mark no longer possible, a shut door, an absent trustee, an unlit lamp.
pub fn ghost() -> Srgb {
    Palette::faint()
}

/// A leak or a failure: the alarm, a lost or bribed keeper, a slipped copy.
pub fn leak() -> Srgb {
    Palette::red()
}

/// The door opening.
pub fn open() -> Srgb {
    Palette::green()
}

/// The dealer's fair spins: the spare dial, its title and the mark it stops
/// on (chapter 03 on).
pub fn spin() -> Srgb {
    Palette::yellow()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_role_is_its_own_color() {
        let roles = [secret(), share(), trio(), pair(), ghost(), leak(), open(), spin()];
        for (i, a) in roles.iter().enumerate() {
            for b in &roles[i + 1..] {
                assert_ne!(a, b);
            }
        }
    }
}
