//! Every fixed on-screen string of Video F, for the glyph probe (`mental
//! glyphs`): the chapters build their text from these constants, and a
//! string that carries a number is built from its fixed parts here and a
//! value from `facts` (the probe covers the digits through [`DIGITS`]).

/// The digits and separators numbers are printed with.
pub const DIGITS: &str = "0123456789 , . % $ + − × ÷ = ( )";

pub const TITLE: &str = "Mental Math Tricks That Actually Work (and Why)";

/// Plain text, in the kit's text face (a serif).
pub const TEXT: &[&str] = &[DIGITS, TITLE];
/// Mathematics (Computer Modern, native TeX).
pub const TEX: &[&str] = &[];
/// Code, set in CM Typewriter.
pub const MONO: &[&str] = &[];
