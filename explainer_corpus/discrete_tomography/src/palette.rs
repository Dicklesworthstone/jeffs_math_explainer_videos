//! The video's color roles (storyboard, "Color roles"): one color per idea
//! for the whole video, on top of the corpus's shared roles.

use explainer_kit::Palette;
use fmn::prelude::Srgb;

/// A filled square, in every picture and every thumbnail.
pub fn filled() -> Srgb {
    Palette::blue()
}

/// A switch: its rectangle, its corners' rings and its word. Strokes and
/// words only.
pub fn switch() -> Srgb {
    Palette::hero()
}

/// A route along rows and columns: a beam counting squares (chapter 01), a
/// unit of water's path (chapter 02). Strokes only.
pub fn beam() -> Srgb {
    Palette::teal()
}

/// Water taken back out of a square, and the way it goes back (chapter 02).
pub fn taken_back() -> Srgb {
    Palette::purple()
}

/// A count that matches the face's.
pub fn matches() -> Srgb {
    Palette::green()
}

/// What no picture can do: a count, a cross, the verdict.
pub fn impossible() -> Srgb {
    Palette::red()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_role_is_its_own_color() {
        let roles = [
            filled(),
            switch(),
            beam(),
            taken_back(),
            matches(),
            impossible(),
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
