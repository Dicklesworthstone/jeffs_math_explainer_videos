//! The video's color roles (storyboard, "Color roles"): one color per idea
//! for the whole video, on top of the corpus's shared roles.

use explainer_kit::{Palette, hex};
use fmn::prelude::Srgb;

/// Matrix A, and what A multiplies into: A·(B·r).
pub fn matrix_a() -> Srgb {
    Palette::blue()
}

/// Matrix B, and its heads columns added: B·r.
pub fn matrix_b() -> Srgb {
    hex("#C792EA")
}

/// The claimed answer C, and its heads columns added: C·r.
pub fn claim() -> Srgb {
    Palette::gold()
}

/// A coin that landed heads, and the columns it keeps. Tails is `faint`.
pub fn heads() -> Srgb {
    Palette::teal()
}

/// A toss that catches the mistake (the two columns disagree).
pub fn caught() -> Srgb {
    Palette::green()
}

/// The mistake: the wrong entry, its column, and a toss that misses it.
pub fn mistake() -> Srgb {
    Palette::red()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_role_is_its_own_color() {
        let roles = [
            matrix_a(),
            matrix_b(),
            claim(),
            heads(),
            caught(),
            mistake(),
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
