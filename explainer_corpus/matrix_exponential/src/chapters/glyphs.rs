//! Every fixed on-screen string of Video J, for the glyph probe (`expm
//! glyphs`): the chapters build their text from these constants, and a
//! string that carries a number is built from its fixed parts here and a
//! value from `facts` (the probe covers the digits through [`DIGITS`]).

/// The digits and separators numbers are printed with.
pub const DIGITS: &str = "0123456789 , . + − × ÷ = ( ) ! ≈ ≠ ≤";

pub const TITLE: &str = "What Does It Mean to Raise e to a Matrix?";

/// Plain text (IBM Plex Sans).
pub const TEXT: &[&str] = &[DIGITS, TITLE];
/// Mathematics (Computer Modern, native TeX).
pub const TEX: &[&str] = &[];
/// Code, set in CM Typewriter.
pub const MONO: &[&str] = &[];
