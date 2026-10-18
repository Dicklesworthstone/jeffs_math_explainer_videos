//! Extra strings for the glyph probe (`pvalues glyphs`). The probe collects
//! every string the real scenes typeset, and each chapter keeps its fixed
//! words as constants at the top of its own file; what is listed here is
//! only what a scene builds at run time from a value in `facts` (the digits
//! and separators numbers are printed with, the fractions' TeX), the title
//! and the thumbnail's words (the probe renders chapters, not the
//! thumbnail).

/// The digits and separators numbers are printed with.
pub const DIGITS: &str = "0123456789 , . % + − = ( ) / ? ≈ · | : “ ” ' ×";

pub const TITLE: &str = "Why P-Values Confuse Even Doctors";

/// Plain text, in the kit's text face.
pub const TEXT: &[&str] = &[DIGITS, TITLE, super::thumbnail::OF_WHAT];
/// Mathematics (native TeX): the fractions of 01_10, 01_11, 02_5–02_6, 04_7
/// and 05_5–05_6 (some with a TeX thousands separator) and 02_9's "≠".
pub const TEX: &[&str] = &[
    r"\frac{90}{585}",
    r"\frac{495}{9{,}900}",
    r"\frac{450}{9{,}000}",
    r"\frac{900}{1{,}350}",
    r"\frac{21{,}700}{1{,}048{,}576}",
    r"\frac{60{,}460}{1{,}048{,}576}",
    r"\neq",
];
/// Code, in the typewriter face.
pub const MONO: &[&str] = &[];
