//! The video's color roles (storyboard, "Color roles"): one color per idea
//! for the whole video, on top of the corpus's shared roles. Only the roles
//! an implemented chapter uses are here; the rest of the storyboard's table
//! (an unfair pool) joins with the chapters that draw it.

use explainer_kit::Palette;
use fmn::prelude::Srgb;

/// A number someone saw, its box's number, its dot and its tick.
pub fn seen() -> Srgb {
    Palette::gold()
}

/// An unseen number in a gap below the largest, the gap's count and a
/// spacing arrow.
pub fn unseen() -> Srgb {
    Palette::blue()
}

/// The top gap: everything above the largest number, nobody knows how far.
pub fn top_gap() -> Srgb {
    Palette::hero()
}

/// An estimate the gap recipe gives.
pub fn estimate() -> Srgb {
    Palette::green()
}

/// The doubling recipe and its estimates (from 10).
pub fn doubling() -> Srgb {
    Palette::purple()
}

/// What is certain: the lower bound. Text and strokes only.
pub fn certain() -> Srgb {
    Palette::teal()
}

/// The line's dots and its thin continuation.
pub fn line() -> Srgb {
    Palette::faint()
}

/// The boxes, their lids, labels and the stakes scale.
pub fn outline() -> Srgb {
    Palette::dim()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_role_is_its_own_color() {
        let roles = [
            seen(),
            unseen(),
            top_gap(),
            estimate(),
            doubling(),
            certain(),
            line(),
            outline(),
            Palette::ink(),
        ];
        for (i, a) in roles.iter().enumerate() {
            for b in &roles[i + 1..] {
                assert_ne!(a, b);
            }
        }
    }
}
