//! Extra strings for the glyph probe (`survey glyphs`). The probe collects
//! every string the real scenes typeset, and each chapter keeps its fixed
//! words as constants at the top of its own file; what is listed here is
//! only what a scene builds at run time from a value in `facts` (the digits
//! and signs counts are printed with) and the title.

/// The digits and signs counts, totals and the code card are printed with.
pub const DIGITS: &str = "0123456789 + − - * / = , . : ? ×";

pub const TITLE: &str = "How to Learn a Secret About a Crowd Without Learning Each Person's Answer";

/// Plain text, in the kit's text face.
pub const TEXT: &[&str] = &[DIGITS, TITLE];
/// Mathematics (native TeX) built outside a chapter: none (the probe
/// captures the chapters' identities from the scenes).
pub const TEX: &[&str] = &[];
/// Code, in the typewriter face, built outside a chapter: none.
pub const MONO: &[&str] = &[];
