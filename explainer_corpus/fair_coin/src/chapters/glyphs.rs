//! Extra strings for the glyph probe (`coin glyphs`). The probe collects
//! every string the real scenes typeset, and each chapter keeps its fixed
//! words as constants at the top of its own file; what is listed here is
//! only what a scene builds at run time from a value in `facts` (the digits
//! and separators numbers are printed with) and the title.

/// The digits and separators numbers are printed with.
pub const DIGITS: &str = "0123456789 , . % + − – × ÷ = ( ) / ? ’";

pub const TITLE: &str = "How to Make a Fair Coin Out of an Unfair One";

/// Plain text, in the kit's text face.
pub const TEXT: &[&str] = &[DIGITS, TITLE];
/// Mathematics (native TeX).
pub const TEX: &[&str] = &["p", "1-p", "(1-p)", r"\times"];
/// Code, in the typewriter face.
pub const MONO: &[&str] = &[];
