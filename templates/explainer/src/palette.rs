//! Video role colors (plan §3.5: each idea gets one named color role).

use explainer_kit::Palette;
use fmn::prelude::Srgb;

/// Primary hero quantity (the quadruple count).
pub fn hero() -> Srgb {
    Palette::hero()
}

/// The input parameter $N$ in symbolic and numeric formulas.
pub fn question() -> Srgb {
    Palette::blue()
}

/// Primary foreground ink.
pub fn ink() -> Srgb {
    Palette::ink()
}

/// Dimmed secondary text (credits and annotations).
pub fn dim() -> Srgb {
    Palette::dim()
}
