//! Extra strings for the glyph probe (`pvalues glyphs`). The probe collects
//! every string the real scenes typeset, and each chapter keeps its fixed
//! words as constants at the top of its own file; what is listed here is
//! only what a scene builds at run time from a value in `facts` (the digits
//! and separators numbers are printed with, the fractions' TeX) and the
//! title.

/// The digits and separators numbers are printed with.
pub const DIGITS: &str = "0123456789 , . % + − = ( ) / ? ≈ · | : “ ” '";

pub const TITLE: &str = "Why P-Values Confuse Even Doctors";

/// Plain text, in the kit's text face.
pub const TEXT: &[&str] = &[DIGITS, TITLE];
/// Mathematics (native TeX): the fractions of 01_10 and 01_11, the second
/// with a TeX thousands separator.
pub const TEX: &[&str] = &[r"\frac{90}{585}", r"\frac{495}{9{,}900}"];
/// Code, in the typewriter face.
pub const MONO: &[&str] = &[];
