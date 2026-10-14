//! Every fixed on-screen string of Video G, for the glyph probe (`lie
//! glyphs`): the chapters build their text from these constants, and a
//! string that carries a number is built from its fixed parts here and a
//! value from `facts` (the probe covers the digits through [`DIGITS`]).

/// The digits and separators numbers are printed with.
pub const DIGITS: &str = "0123456789 , . ° % + − × ÷ = ≠ ( ) ′ ‖ – — ’ “ ” … ² ³ : ?";

pub const TITLE: &str = "Why Robot Arms Think in Lie Groups";

/// Plain text (IBM Plex Sans).
pub const TEXT: &[&str] = &[DIGITS, TITLE];
/// Mathematics (Computer Modern, native TeX).
pub const TEX: &[&str] = &[];
/// Code, set in CM Typewriter.
pub const MONO: &[&str] = &[];
