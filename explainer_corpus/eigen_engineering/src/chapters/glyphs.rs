//! Every fixed on-screen string of Video K, for the glyph probe (`eigen
//! glyphs`): the chapters build their text from these constants, and a
//! string that carries a number is built from its fixed parts here and a
//! value from `facts` (the probe covers the digits through [`DIGITS`]).

/// The digits and separators numbers are printed with. The minus of a number
/// in plain text is the en dash the engine's live numbers print (the text
/// face has no U+2212 of its own); TeX sets its own.
pub const DIGITS: &str = "0123456789 , . % ° + – × ÷ = ( ) √";

pub const TITLE: &str = "The One Idea Behind Bridges, Concert Halls and Economies";

/// Plain text (IBM Plex Sans).
pub const TEXT: &[&str] = &[DIGITS, TITLE];
/// Mathematics (Computer Modern, native TeX).
pub const TEX: &[&str] = &[];
/// Code, set in CM Typewriter.
pub const MONO: &[&str] = &[];
