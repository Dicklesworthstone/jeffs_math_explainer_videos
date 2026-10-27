//! Extra strings for the glyph probe (`inspection_paradox glyphs`). The
//! probe collects every string the real scenes typeset; each chapter keeps
//! its fixed words as constants at the top of its own file. What is listed
//! here is only what no chapter typesets on its own: the digits and signs
//! numbers are printed with, and the whole title (chapter 01's title card
//! sets it in two halves; the video has no thumbnail yet).

/// The digits and signs times, counts and shares are printed with.
pub const DIGITS: &str = "0123456789 , . : ? + = − / × ÷ ½ ⅓ ⅙ ⅚";

pub const TITLE: &str = "Why Your Bus Wait Is Longer Than You Expect";

/// Plain text, in the kit's text face.
pub const TEXT: &[&str] = &[DIGITS, TITLE];
/// Mathematics (native TeX) built outside a chapter: none.
pub const TEX: &[&str] = &[];
/// Code, in the typewriter face: none.
pub const MONO: &[&str] = &[];
