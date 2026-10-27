//! The video's color roles (storyboard, "Color roles and stable objects"):
//! one color per idea for the whole video, on top of the corpus's shared
//! roles.

use explainer_kit::{Palette, hex};
use fmn::prelude::Srgb;

/// Game A: its card, its coin, its arrows, its runs and rounds.
pub fn game_a() -> Srgb {
    Palette::blue()
}

/// Game B: its card, its arrows, its runs and rounds.
pub fn game_b() -> Srgb {
    hex("#C792EA")
}

/// The mix: the fair deciding coin, its arrows, runs, bars and verdicts.
pub fn mix() -> Srgb {
    Palette::gold()
}

/// The bad coin and corner 0, the count with nothing left over.
pub fn bad() -> Srgb {
    Palette::red()
}

/// The good coin, wins and forward steps.
pub fn good() -> Srgb {
    Palette::green()
}

/// The player's token and dollar chips, then every copy in the crowd.
pub fn player() -> Srgb {
    Palette::teal()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_role_is_its_own_color() {
        let roles = [
            game_a(),
            game_b(),
            mix(),
            bad(),
            good(),
            player(),
            Palette::hero(),
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
