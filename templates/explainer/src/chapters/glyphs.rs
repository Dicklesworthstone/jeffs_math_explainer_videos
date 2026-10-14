//! Supplemental strings for the glyph probe (`__SLUG__ glyphs`). The probe
//! collects every string the real scenes typeset and renders them onto pages.

/// The digits and separators numbers are printed with.
pub const DIGITS: &str = "0123456789 , . % + − – × ÷ = ( ) / ? ’";

pub const TITLE: &str = "__TITLE__";

/// Plain text, in the kit's text face.
pub const TEXT: &[&str] = &[
    DIGITS,
    TITLE,
    "How many ways to choose 4 of 5,000 points?",
    "Every number here was computed, never typed.",
    "franken_manim  ·  FrankenTTS",
];

/// Mathematics (native TeX).
pub const TEX: &[&str] = &[
    r"C(N, 4) = \frac{N(N-1)(N-2)(N-3)}{24}",
    r"C(5000, 4) = \frac{5000\cdot 4999\cdot 4998\cdot 4997}{24}",
];

/// Code, in the typewriter face.
pub const MONO: &[&str] = &[];
