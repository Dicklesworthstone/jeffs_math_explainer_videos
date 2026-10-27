//! Extra strings for the glyph probe (`vault glyphs`). The probe collects
//! every string the real scenes typeset, and each chapter keeps its fixed
//! words as constants at the top of its own file; what is listed here is
//! only what a scene builds at run time from a value in `facts` (the digits
//! and signs marks, counts and cards are printed with) and the title.

/// The digits and signs marks, counts and cards are printed with.
pub const DIGITS: &str = "0123456789 + − - * / = , . : ? × →";

pub const TITLE: &str = "Three Keys Open the Secret. Two Reveal Nothing.";

/// Plain text, in the kit's text face.
pub const TEXT: &[&str] = &[DIGITS, TITLE];
/// Mathematics (native TeX) built outside a chapter: none.
pub const TEX: &[&str] = &[];
/// Code, in the typewriter face, built outside a chapter: none.
pub const MONO: &[&str] = &[];
