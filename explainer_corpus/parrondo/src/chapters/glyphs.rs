//! Extra strings for the glyph probe (`parrondo glyphs`). The probe
//! collects every string the real scenes typeset; each chapter keeps its
//! fixed words as constants at the top of its own file. What is listed here
//! is only what no chapter typesets on its own yet: the digits and signs
//! numbers are printed with, plus the title also typeset in chapter 05.
//! The video has no thumbnail yet.

/// The digits and signs counts, chances and cents are printed with.
pub const DIGITS: &str = "0123456789 , . : ? + = − % / · →";

pub const TITLE: &str = "Two Losing Games Can Win";
pub const SUBTITLE: &str = "Here is the catch";

/// Plain text, in the kit's text face.
pub const TEXT: &[&str] = &[DIGITS, TITLE, SUBTITLE];
/// Mathematics (native TeX) built outside a chapter: none.
pub const TEX: &[&str] = &[];
/// Code, in the typewriter face: none.
pub const MONO: &[&str] = &[];
