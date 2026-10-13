//! Every fixed on-screen string of Video I, for the glyph probe (`proofs
//! glyphs`): the chapters build their text from these constants, and a
//! string that carries a number is built from its fixed parts here and a
//! value from `facts` (the probe covers the digits through [`DIGITS`]).

/// The digits and separators numbers are printed with.
pub const DIGITS: &str = "0123456789 , . + − × ÷ = ( ) ½ ⅓ ¼ ?";

pub const TITLE: &str = "Proofs You Can See";

/// Plain text (Computer Modern roman, the kit's default face).
pub const TEXT: &[&str] = &[DIGITS, TITLE];
/// Mathematics (Computer Modern, native TeX).
pub const TEX: &[&str] = &[];
/// Code, set in CM Typewriter.
pub const MONO: &[&str] = &[];
