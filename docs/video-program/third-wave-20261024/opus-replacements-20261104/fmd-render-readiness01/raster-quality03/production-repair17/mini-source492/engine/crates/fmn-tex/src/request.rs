//! Complete cache-warming inputs; no geometry or second layout policy.

use crate::{LineAlign, Mode, Style};

/// The semantic inputs of one Tex/TexText construction, excluding visual
/// scale and paint which are applied after layout. Extra declarations remain
/// request-local and source spans always address the original formula.
#[derive(Clone, Copy, Debug)]
pub struct TypesetRequest<'a> {
    /// Mathematics at a particular style, or text with math islands.
    pub mode: Mode,
    /// Original source, before preamble/alignment wrapping.
    pub source: &'a str,
    /// Native macro declarations, not an external LaTeX preamble.
    pub preamble: &'a str,
    /// Line alignment for text mode; mathematics requires Left.
    pub align: LineAlign,
}

impl<'a> TypesetRequest<'a> {
    /// Default Tex semantics: display-style mathematics.
    #[must_use]
    pub const fn math(source: &'a str) -> Self {
        Self {
            mode: Mode::Math(Style::Display),
            source,
            preamble: "",
            align: LineAlign::Left,
        }
    }

    /// Text-style mathematics, including the library's scale calibration.
    #[must_use]
    pub const fn inline_math(source: &'a str) -> Self {
        Self {
            mode: Mode::Math(Style::Text),
            ..Self::math(source)
        }
    }

    /// Default TexText semantics: text-mode layout with centered lines.
    #[must_use]
    pub const fn text(source: &'a str) -> Self {
        Self {
            mode: Mode::Text,
            source,
            preamble: "",
            align: LineAlign::Center,
        }
    }
}
