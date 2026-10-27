//! Extra strings for the glyph probe (`tank glyphs`). The probe collects
//! every string the real scenes typeset, and each chapter keeps its fixed
//! words as constants at the top of its own file; what is listed here is
//! only what a scene builds at run time from a value in `facts` (the digits
//! and signs values are printed with) and the title. There is no thumbnail
//! yet.

/// The digits and signs values, counts and the countdown are printed with.
pub const DIGITS: &str = "0123456789 / , . ? ( ) % -";

pub const TITLE: &str = "How Much Can Five Serial Numbers Reveal?";

/// Plain text, in the kit's text face.
pub const TEXT: &[&str] = &[DIGITS, TITLE];
/// Mathematics (native TeX) built outside a chapter: none (the probe
/// captures the chapters' sums and equations from the scenes).
pub const TEX: &[&str] = &[];
/// Code, in the typewriter face, built outside a chapter: none.
pub const MONO: &[&str] = &[];
