//! The video's color roles (storyboard, "Color roles"): one color per idea
//! for the whole video, on top of the corpus's shared roles. Only the roles
//! a written chapter draws are here; the short gap (teal) and the spread
//! (`#C792EA`) are in the storyboard's table and join with their chapters.

use explainer_kit::Palette;
use fmn::prelude::Srgb;

/// The blue bus: its markers and its row of times.
pub fn blue_bus() -> Srgb {
    Palette::blue()
}

/// The red bus: its markers and its row of times.
pub fn red_bus() -> Srgb {
    Palette::red()
}

/// A long gap (in chapter 03 also the gap that's growing): its bracket, its
/// triangle and the passengers in it.
pub fn long_gap() -> Srgb {
    Palette::gold()
}

/// The true average wait, a verdict.
pub fn verdict() -> Srgb {
    Palette::green()
}

/// The ring around you, the think ring and "?".
pub fn you() -> Srgb {
    Palette::hero()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// No two roles share a color, and none is the short gap's teal, which
    /// the storyboard keeps for its chapters.
    #[test]
    fn every_role_is_its_own_color() {
        let roles = [
            blue_bus(),
            red_bus(),
            Palette::teal(),
            long_gap(),
            verdict(),
            you(),
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
