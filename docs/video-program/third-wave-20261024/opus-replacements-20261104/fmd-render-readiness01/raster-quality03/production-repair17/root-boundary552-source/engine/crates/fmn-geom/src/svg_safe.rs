//! Public SVG admission boundary.
//!
//! The parser implementation remains in `svg.rs`; this facade owns the
//! byte-level preflight that must complete before the XML tokenizer can
//! inspect fixed ASCII prefixes. Keeping that boundary separate makes the
//! untrusted-input contract explicit: UTF-8 is validated once, every
//! fixed-width probe is checked at a character boundary, and only then does
//! the document parser run.

use crate::svg_parser;
pub use crate::svg_parser::{
    DEFAULT_SVG_TOLERANCE, LineCap, LineJoin, Paint, SvgError, SvgLimits, SvgShape, SvgStyle,
    emit_path_data, emit_svg,
};

const UNSUPPORTED_DECLARATION: &str = concat!(
    "unsupported markup declaration (only elements, comments, CDATA, ",
    "and processing instructions are accepted)"
);

/// A parsed + resolved SVG document admitted through the UTF-8-safe public
/// boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct SvgDocument {
    /// Viewport width in px.
    pub width: f64,
    /// Viewport height in px.
    pub height: f64,
    /// The root `viewBox` `(min_x, min_y, width, height)`, if declared.
    pub view_box: Option<[f64; 4]>,
    /// The flattened shapes, in document order.
    pub shapes: Vec<SvgShape>,
}

impl From<svg_parser::SvgDocument> for SvgDocument {
    fn from(document: svg_parser::SvgDocument) -> Self {
        Self {
            width: document.width,
            height: document.height,
            view_box: document.view_box,
            shapes: document.shapes,
        }
    }
}

impl SvgDocument {
    /// Parse SVG bytes under the default budgets.
    pub fn parse(bytes: &[u8]) -> Result<Self, SvgError> {
        validate_admission(bytes, &SvgLimits::default())?;
        svg_parser::SvgDocument::parse(bytes).map(Into::into)
    }

    /// Parse SVG bytes under explicit budgets.
    pub fn parse_with_limits(bytes: &[u8], limits: &SvgLimits) -> Result<Self, SvgError> {
        validate_admission(bytes, limits)?;
        svg_parser::SvgDocument::parse_with_limits(bytes, limits).map(Into::into)
    }
}

/// Emit one resolved document back to SVG bytes without cloning its retained
/// geometry. Resolved shapes are already in post-`viewBox` user space, so the
/// original viewBox is deliberately not applied a second time.
#[must_use]
pub fn emit_svg_document(document: &SvgDocument) -> String {
    emit_svg(&document.shapes, document.width, document.height, None)
}

fn validate_admission(bytes: &[u8], limits: &SvgLimits) -> Result<(), SvgError> {
    if bytes.len() > limits.max_bytes {
        return Err(SvgError::TooLarge {
            bytes: bytes.len(),
            limit: limits.max_bytes,
        });
    }
    let text = std::str::from_utf8(bytes).map_err(|error| SvgError::NotUtf8 {
        offset: error.valid_up_to(),
    })?;
    validate_fixed_markup_probes(text)
}

fn validate_fixed_markup_probes(text: &str) -> Result<(), SvgError> {
    let bytes = text.as_bytes();
    let mut cursor = 0;
    while cursor < bytes.len() {
        let Some(offset) = bytes[cursor..].iter().position(|byte| *byte == b'<') else {
            break;
        };
        let start = cursor + offset;
        let rest = &text[start..];
        if rest.starts_with("<!--") {
            let Some(end) = rest.find("-->") else {
                break;
            };
            cursor = start + end + 3;
            continue;
        }
        if rest.starts_with("<![CDATA[") {
            let Some(end) = rest.find("]]>") else {
                break;
            };
            cursor = start + end + 3;
            continue;
        }
        if rest.starts_with("<?") {
            let Some(end) = rest.find("?>") else {
                break;
            };
            cursor = start + end + 2;
            continue;
        }
        if bytes.get(start + 1) == Some(&b'!')
            && bytes.len() - start >= b"<!doctype".len()
            && !text.is_char_boundary(start + b"<!doctype".len())
        {
            let line = 1 + bytes[..start].iter().filter(|byte| **byte == b'\n').count();
            return Err(SvgError::Malformed {
                line,
                message: UNSUPPORTED_DECLARATION.to_owned(),
            });
        }

        // Skip a complete ordinary tag so a '<' inside a quoted attribute
        // value is never mistaken for a tokenizer-level markup probe.
        let mut quote = None;
        let mut index = start + 1;
        while let Some(&byte) = bytes.get(index) {
            match (quote, byte) {
                (Some(open), close) if close == open => quote = None,
                (None, b'\'' | b'"') => quote = Some(byte),
                (None, b'>') => {
                    index += 1;
                    break;
                }
                _ => {}
            }
            index += 1;
        }
        cursor = index.max(start + 1);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoted_markup_like_text_does_not_trigger_the_boundary_guard() {
        let document = SvgDocument::parse(
            "<svg><rect id=\"<!12345¤\" width=\"1\" height=\"1\"/></svg>".as_bytes(),
        )
        .expect("quoted attribute text is not a tokenizer-level declaration");
        assert_eq!(document.shapes.len(), 1);
    }
}
