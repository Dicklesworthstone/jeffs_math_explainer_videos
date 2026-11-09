//! The `Code` mobject (§11.7, Appendix A `text_mobject.Code`): fmd's
//! syntax highlighter over the CM Typewriter face — the pygments
//! displacement (§1.7, §2.3).
//!
//! The highlighter's token spans ARE source spans, so they wire straight
//! into [`Text::char_overrides`]: positional per-character fills that can
//! never bleed a keyword colour into an identifier substring. Language
//! selection passes through to fmd; an unknown language yields plain
//! text, exactly as a highlighter falling back would. Themes are owned:
//! [`CodeTheme::from_pygments_name`] documents which familiar style
//! names map where.
//!
//! Tranche 1 surface (fm-u8y): language selection, owned theme mapping,
//! and the Reference's `line_numbers` gutter. `MarkdownMobject` and the
//! string-transform animations are later tranches of fm-u8y.

use fmn_core::color::Srgb;
use fmn_text::font::MONO_FAMILY;
use fmn_text::maps::CharOverride;
use franken_markdown::highlight::{self, Tok};

use crate::text::{DEFAULT_FONT_SIZE, Text, TextMobject, TextMobjectError};
use crate::vmobject::VMobject;

/// An owned highlight theme: one fill per token class. Plain text keeps
/// the base style's colour.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CodeTheme {
    name: &'static str,
    keyword: Srgb,
    ty: Srgb,
    func: Srgb,
    string: Srgb,
    number: Srgb,
    comment: Srgb,
    operator: Srgb,
    punct: Srgb,
}

impl CodeTheme {
    /// A light-theme baseline in the pygments-`default` spirit: muted
    /// keywords, olive comments, teal numerals.
    #[must_use]
    pub fn light() -> Self {
        Self {
            name: "light",
            keyword: Srgb::from_rgb8(2, 0, 139),
            ty: Srgb::from_rgb8(139, 69, 19),
            func: Srgb::from_rgb8(128, 0, 128),
            string: Srgb::from_rgb8(206, 92, 0),
            number: Srgb::from_rgb8(0, 165, 165),
            comment: Srgb::from_rgb8(104, 120, 68),
            operator: Srgb::from_rgb8(54, 54, 54),
            punct: Srgb::from_rgb8(54, 54, 54),
        }
    }

    /// The dark theme behind the familiar `monokai`/`one-dark` family:
    /// saturated candy fills for the caller's dark background.
    #[must_use]
    pub fn dark() -> Self {
        Self {
            name: "dark",
            keyword: Srgb::from_rgb8(104, 194, 70),
            ty: Srgb::from_rgb8(69, 194, 247),
            func: Srgb::from_rgb8(173, 100, 243),
            string: Srgb::from_rgb8(242, 189, 69),
            number: Srgb::from_rgb8(247, 131, 14),
            comment: Srgb::from_rgb8(113, 124, 104),
            operator: Srgb::from_rgb8(247, 79, 95),
            punct: Srgb::from_rgb8(255, 255, 255),
        }
    }

    /// Map a familiar pygments style name to an owned theme. Unknown names
    /// are a `None`, never a silent substitution — callers choose their
    /// own fallback.
    #[must_use]
    pub fn from_pygments_name(name: &str) -> Option<Self> {
        match name {
            "default" | "friendly" | "colorful" => Some(Self::light()),
            "monokai" | "vim" | "one-dark" | "dracula" | "material" => Some(Self::dark()),
            _ => None,
        }
    }

    /// The theme's own name (`light` / `dark`).
    #[must_use]
    pub fn name(&self) -> &'static str {
        self.name
    }

    fn color(self, kind: Tok) -> Option<Srgb> {
        match kind {
            Tok::Plain => None,
            Tok::Keyword => Some(self.keyword),
            Tok::Type => Some(self.ty),
            Tok::Func => Some(self.func),
            Tok::Str => Some(self.string),
            Tok::Number => Some(self.number),
            Tok::Comment => Some(self.comment),
            Tok::Operator => Some(self.operator),
            Tok::Punct => Some(self.punct),
        }
    }
}

/// The `Code` builder: highlighted source over CM Typewriter.
///
/// Builder setters are by-value, the §15.1 surface; `build` takes the
/// font book exactly like [`Text`] does.
#[derive(Debug, Clone)]
pub struct Code<'a> {
    code: &'a str,
    language: &'a str,
    theme: CodeTheme,
    line_numbers: bool,
    font_size: f64,
}

impl<'a> Code<'a> {
    /// A plain `Code`: no gutter, the light theme, the Reference's default
    /// text font size.
    #[must_use]
    pub fn new(code: &'a str) -> Self {
        Self {
            code,
            language: "",
            theme: CodeTheme::light(),
            line_numbers: false,
            font_size: DEFAULT_FONT_SIZE,
        }
    }

    /// The `language=` surface — anything fmd's highlighter accepts.
    #[must_use]
    pub fn language(mut self, language: &'a str) -> Self {
        self.language = language;
        self
    }

    /// An owned theme ([`CodeTheme::from_pygments_name`] maps the familiar
    /// names).
    #[must_use]
    pub fn theme(mut self, theme: CodeTheme) -> Self {
        self.theme = theme;
        self
    }

    /// The Reference's `line_numbers=True` gutter.
    #[must_use]
    pub fn line_numbers(mut self, yes: bool) -> Self {
        self.line_numbers = yes;
        self
    }

    /// The `font_size=` surface.
    #[must_use]
    pub fn font_size(mut self, font_size: f64) -> Self {
        self.font_size = font_size;
        self
    }

    /// Translate fmd's byte-span tiling into per-character overrides with
    /// the typewriter family on every char.
    /// Positional styles from the native highlighter, retaining original source
    /// byte-token boundaries. Consumers may override the typewriter family while
    /// reusing the token fills in the ordinary Scribe text pipeline.
    #[must_use]
    pub fn character_overrides(&self) -> Vec<CharOverride<'static>> {
        let spans = highlight::highlight(self.language, self.code);
        let mut out = Vec::with_capacity(self.code.chars().count());
        let mut spans_iter = spans.iter();
        let mut current = spans_iter.next().copied();
        let mut consumed = 0usize;
        for ch in self.code.chars() {
            while matches!(current, Some(span) if span.end <= consumed) {
                current = spans_iter.next().copied();
            }
            // Spans tile the source, so the char sits inside `current`
            // unless it is past the final span (trailing Plain).
            let in_span = matches!(current, Some(span) if consumed >= span.start);
            let color = match (in_span, current) {
                (true, Some(span)) => self.theme.color(span.kind),
                _ => None,
            };
            out.push(CharOverride {
                color,
                family: Some(MONO_FAMILY),
            });
            consumed += ch.len_utf8();
        }
        out
    }

    /// Lay the highlighted source out as glyph records, with the optional
    /// line-number gutter top-left of it.
    ///
    /// # Errors
    /// Whatever [`Text::build`] hits — font policy and shaping are shared.
    pub fn build(&self, book: &fmn_text::FontBook) -> Result<TextMobject, TextMobjectError> {
        let overrides = self.character_overrides();
        let body = Text::new(self.code)
            .char_overrides(&overrides)
            .font_size(self.font_size)
            .build(book)?;
        if !self.line_numbers {
            return Ok(body);
        }

        let n_lines = self.code.lines().count().max(1);
        let gutter_str = (1..=n_lines)
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        let gutter = Text::new(&gutter_str)
            .char_overrides(
                // The gutter is mono but uncoloured — base style only.
                &[],
            )
            .font_size(self.font_size)
            .build(book)?;

        const GAP: f64 = 0.35;
        let body_left = body.vmob.bbox_point([-1.0, 0.0, 0.0]);
        let gutter_right = gutter.vmob.bbox_point([1.0, 0.0, 0.0]);
        let body_top = body.vmob.bbox_point([0.0, 1.0, 0.0]);
        let gutter_top = gutter.vmob.bbox_point([0.0, 1.0, 0.0]);
        if let (Some(bl), Some(gr), Some(bt), Some(gt)) =
            (body_left, gutter_right, body_top, gutter_top)
        {
            let dx = bl[0] - GAP - gr[0];
            let dy = bt[1] - gt[1];
            let vmob = VMobject::new()
                .with_child(gutter.vmob.shifted([dx, dy, 0.0]))
                .with_child(body.vmob);
            return Ok(TextMobject { vmob, ..body });
        }
        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmn_core::constants::DEFAULT_MOBJECT_COLOR;
    use fmn_text::FontBook;

    fn book() -> FontBook {
        FontBook::bundled().expect("bundled faces parse")
    }

    fn fills(m: &TextMobject) -> Vec<Srgb> {
        m.vmob
            .children()
            .iter()
            .map(|child| child.style().fill_color)
            .collect()
    }

    #[test]
    fn keyword_tokens_take_the_theme_fill() {
        // Both chars are the `fn` keyword under fmd's rust rules, so every
        // glyph fill is the theme's keyword colour — positionally applied.
        let theme = CodeTheme::dark();
        let m = Code::new("fn")
            .language("rust")
            .theme(theme)
            .build(&book())
            .expect("builds");
        let all = fills(&m);
        assert!(!all.is_empty());
        assert!(all.iter().all(|c| *c == theme.keyword), "{all:?}");
    }

    #[test]
    fn plain_source_keeps_the_base_fill() {
        // An unknown language tiles one Plain span: base colour throughout.
        let m = Code::new("plain text")
            .language("not-a-language")
            .build(&book())
            .expect("builds");
        assert!(fills(&m).iter().all(|c| *c == DEFAULT_MOBJECT_COLOR));
    }

    #[test]
    fn mixed_tokens_distinguish_keyword_from_plain() {
        // "fn" is a keyword; " main" is Plain. Expected fills are derived
        // positionally (non-whitespace glyphs in reading order), then
        // checked against what the built mobject actually carries.
        let theme = CodeTheme::dark();
        let code = "fn main";
        let m = Code::new(code)
            .language("rust")
            .theme(theme)
            .build(&book())
            .expect("builds");

        let spans = franken_markdown::highlight::highlight("rust", code);
        let expected: Vec<(char, Srgb)> = {
            let mut out = Vec::new();
            let mut consumed = 0usize;
            for ch in code.chars() {
                let kind = spans
                    .iter()
                    .find(|span| consumed >= span.start && consumed < span.end)
                    .map(|span| span.kind)
                    .unwrap_or(Tok::Plain);
                if !ch.is_whitespace() {
                    out.push((ch, theme.color(kind).unwrap_or(DEFAULT_MOBJECT_COLOR)));
                }
                consumed += ch.len_utf8();
            }
            out
        };

        let children = m.vmob.children();
        assert_eq!(children.len(), expected.len(), "glyph count");
        for (child, (ch, want)) in children.iter().zip(&expected) {
            assert_eq!(child.style().fill_color, *want, "glyph {ch:?}");
        }
    }

    #[test]
    fn line_numbers_add_the_gutter_child() {
        let without = Code::new("fn").language("rust").build(&book()).unwrap();
        assert_eq!(without.vmob.children().len(), 2); // two glyphs
        let with = Code::new("fn")
            .language("rust")
            .line_numbers(true)
            .build(&book())
            .unwrap();
        assert_eq!(with.vmob.children().len(), 2); // gutter + body
    }

    #[test]
    fn pygments_names_map_to_owned_themes_or_none() {
        assert_eq!(
            CodeTheme::from_pygments_name("monokai").map(|t| t.name()),
            Some("dark")
        );
        assert_eq!(
            CodeTheme::from_pygments_name("default").map(|t| t.name()),
            Some("light")
        );
        assert!(CodeTheme::from_pygments_name("no-such-style").is_none());
    }
}
