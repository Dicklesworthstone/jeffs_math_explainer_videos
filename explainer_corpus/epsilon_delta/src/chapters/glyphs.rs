//! Extra strings for the glyph probe (`epsdelta glyphs`). The probe collects
//! every string the real scenes typeset, and each chapter keeps its fixed
//! words as constants at the top of its own file; what is listed here is
//! the title and the digits and signs numbers are printed with.

/// The digits and signs numbers are printed with.
pub const DIGITS: &str = "0123456789 , . = ≈ × ( ) [ ] / : < ≤ − ^ ?";

pub const TITLE: &str = "Epsilon and Delta";
pub const SUBTITLE: &str = "Why Getting Closer Wasn't Enough";

/// Plain text, in the kit's text face.
pub const TEXT: &[&str] = &[DIGITS, TITLE, SUBTITLE];
/// Mathematics (native TeX) the scenes build from facts at run time.
pub const TEX: &[&str] = &[r"0.9^{n}", r"y = x^{50}", r"1^{n} = 1", r"x = 2^{-1/n}"];
/// Code, in the typewriter face.
pub const MONO: &[&str] = &[];
