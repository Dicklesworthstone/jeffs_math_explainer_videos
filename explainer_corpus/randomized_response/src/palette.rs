//! The video's color roles (storyboard, "Color roles"): one color per idea
//! for the whole video, on top of the corpus's shared roles.

use explainer_kit::{Palette, hex};
use fmn::prelude::Srgb;

/// Copied: the truth only we can see. A ring around a student, and the
/// word; strokes and words only.
pub fn copied() -> Srgb {
    Palette::hero()
}

/// A yes answer: its column and the students in it.
pub fn yes() -> Srgb {
    Palette::blue()
}

/// A no answer: its column and the students in it.
pub fn no() -> Srgb {
    hex("#C792EA")
}

/// Heads on the first coin, the truth: its lane and label.
pub fn truth() -> Srgb {
    Palette::green()
}

/// The second coin: its lane, the coin, the outcomes it decides. A voice
/// that carries no information.
pub fn forced() -> Srgb {
    Palette::dim()
}

/// A student before answering, and the question.
pub fn student() -> Srgb {
    Palette::ink()
}

/// A ghost copy of a doubled count, and the places answers land in.
pub fn ghost() -> Srgb {
    Palette::faint()
}

/// The color of an answer.
pub fn answer(said_yes: bool) -> Srgb {
    if said_yes { yes() } else { no() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_role_is_its_own_color() {
        let roles = [copied(), yes(), no(), truth(), forced(), student(), ghost()];
        for (i, a) in roles.iter().enumerate() {
            for b in &roles[i + 1..] {
                assert_ne!(a, b);
            }
        }
        assert_eq!(answer(true), yes());
        assert_eq!(answer(false), no());
    }
}
