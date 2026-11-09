//! Color roles, not hues: each recurring idea gets ONE role for the whole video
//! (the storyboard's color table). 3Blue1Brown values.

use fmn::core::color::color_gradient;
use fmn::prelude::*;

/// The corpus background.
pub const BACKGROUND: &str = "#0E1116";

/// Parse a palette literal.
///
/// # Panics
/// On an invalid hex literal; palette literals are compile-time constants, so a
/// panic here is a programming error caught by the tests.
pub fn hex(s: &str) -> Srgb {
    Srgb::from_hex(s).expect("palette literals are valid hex")
}

/// Shared roles (plan §6.1). Each video registers its own idea → role table on
/// top of these in its `palette.rs`.
pub struct Palette;
impl Palette {
    pub fn ink() -> Srgb {
        hex("#ECEFF4")
    }
    pub fn dim() -> Srgb {
        hex("#8A93A3")
    }
    pub fn faint() -> Srgb {
        hex("#3B4352")
    }
    /// The video's main object.
    pub fn hero() -> Srgb {
        hex("#FFD866")
    }
    /// 3b1b BLUE_C.
    pub fn blue() -> Srgb {
        hex("#58C4DD")
    }
    /// 3b1b TEAL_C.
    pub fn teal() -> Srgb {
        hex("#5CD0B3")
    }
    /// 3b1b GREEN_C; also the "good" role.
    pub fn green() -> Srgb {
        hex("#83C167")
    }
    /// 3b1b GOLD_C.
    pub fn gold() -> Srgb {
        hex("#F0AC5F")
    }
    /// 3b1b RED_C; also the "warn" role.
    pub fn red() -> Srgb {
        hex("#FC6255")
    }
    /// 3b1b PURPLE_C.
    pub fn purple() -> Srgb {
        hex("#9A72AC")
    }
    /// YELLOW: highlights only.
    pub fn yellow() -> Srgb {
        hex("#FFFF00")
    }
}

/// A smooth blue → teal → green → gold ramp, one color per point.
pub fn point_colors(n: usize) -> Vec<Srgb> {
    color_gradient(
        &[Palette::blue(), Palette::teal(), Palette::green(), Palette::gold()],
        n,
    )
}

/// Solid fill in `color` (text and filled shapes).
pub fn ink_style(color: Srgb) -> Style {
    Style::default()
        .fill(color, 1.0)
        .stroke(color, 0.0, 1.0)
        .fill_border_width(0.5)
}

/// Outline only, `width` in Reference stroke units.
pub fn stroke_style(color: Srgb, width: f64) -> Style {
    Style::default().fill(color, 0.0).stroke(color, width, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_role_parses_and_roles_are_distinct() {
        let roles = [
            Palette::ink(),
            Palette::dim(),
            Palette::faint(),
            Palette::hero(),
            Palette::blue(),
            Palette::teal(),
            Palette::green(),
            Palette::gold(),
            Palette::red(),
            Palette::purple(),
            Palette::yellow(),
        ];
        for (i, a) in roles.iter().enumerate() {
            for b in &roles[i + 1..] {
                assert_ne!(a, b);
            }
        }
        assert_eq!(point_colors(150).len(), 150);
        let _ = hex(BACKGROUND);
    }
}
