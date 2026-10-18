//! Extra strings for the glyph probe (`golay glyphs`). The probe collects
//! every string the real scenes typeset, and each chapter keeps its fixed
//! words as constants at the top of its own file; what is listed here is
//! only what a scene builds at run time from a value in `facts` (the digits
//! and signs scores are printed with) and the title.

/// The digits and signs scores and shifts are printed with.
pub const DIGITS: &str = "0123456789 + − , . ? [ ] * =";

pub const TITLE: &str = "Two Bad Echoes Can Make One Perfect Measurement";

/// Plain text, in the kit's text face.
pub const TEXT: &[&str] = &[DIGITS, TITLE];
/// Mathematics (native TeX): chapter 01 typesets none.
pub const TEX: &[&str] = &[];
/// Code, in the typewriter face: chapter 01 shows none.
pub const MONO: &[&str] = &[];
