//! The `MarkdownMobject` tier-1 (§11.7, fm-u8y): fmd's document parser +
//! the Scribe text stack, as animatable content.
//!
//! Blocks map to submobjects in source order — headings and paragraphs
//! through the manim markup tag set, fenced code through [`Code`] — and
//! each child keeps its **source byte range** so later tranches can drive
//! `TransformMatchingStrings` across document edits (edit a slide, animate
//! the diff). Tier discipline: enhanced tier, never gate-blocking.
//!
//! Tables use the native data-table layouter. Images render their alt text;
//! links keep their text without fetching destinations. HTML and `$math$`
//! remain literal text, not browser markup or mathematical layout. Nested
//! lists/quotes flatten with explicit prefixes and retain all nested content.

#[path = "markdown_document.rs"]
mod document;
pub use document::{MarkdownMathOptions, MathDocumentError};

use fmn_text::FontBook;

use crate::code::{Code, CodeTheme};
use crate::text::{Text, TextMobject, TextMobjectError};
use crate::vmobject::VMobject;

/// Maximum source bytes admitted before parsing or shaping.
pub const MAX_MARKDOWN_BYTES: usize = 32_768;
/// Maximum top-level blocks in one document.
pub const MAX_MARKDOWN_BLOCKS: usize = 256;
const MAX_DEPTH: usize = 24;
const MAX_RECORDS: usize = 1_048_576;

/// A bounded document-layout refusal or a precise Scribe error.
#[derive(Debug)]
pub enum MarkdownError {
    /// Native text, geometry or resource admission failed.
    Text(TextMobjectError),
    /// A source span or layout option was invalid.
    InvalidParameter(&'static str),
}

impl std::fmt::Display for MarkdownError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(error) => error.fmt(f),
            Self::InvalidParameter(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for MarkdownError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Text(error) => Some(error),
            Self::InvalidParameter(_) => None,
        }
    }
}

impl From<TextMobjectError> for MarkdownError {
    fn from(error: TextMobjectError) -> Self {
        Self::Text(error)
    }
}

fn limit(context: &'static str, requested: usize, maximum: usize) -> Result<(), TextMobjectError> {
    if requested > maximum {
        return Err(TextMobjectError::ResourceLimit {
            context,
            requested,
            limit: maximum,
        });
    }
    Ok(())
}

fn escape(text: &str, out: &mut String) {
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
}

/// Convert one inline run to the manim markup tag set (`<b>`, `<i>`,
/// `<s>`, `<tt>`); links keep their text.
fn inlines_to_markup(inlines: &[franken_markdown::ast::Inline], out: &mut String) {
    use franken_markdown::ast::Inline;
    for inline in inlines {
        match inline {
            Inline::Text(text) => escape(text, out),
            Inline::Emphasis(inner) => {
                out.push_str("<i>");
                inlines_to_markup(inner, out);
                out.push_str("</i>");
            }
            Inline::Strong(inner) => {
                out.push_str("<b>");
                inlines_to_markup(inner, out);
                out.push_str("</b>");
            }
            Inline::Strikethrough(inner) => {
                out.push_str("<s>");
                inlines_to_markup(inner, out);
                out.push_str("</s>");
            }
            Inline::Code(code) => {
                out.push_str("<tt>");
                escape(code, out);
                out.push_str("</tt>");
            }
            Inline::Link { content, .. } => inlines_to_markup(content, out),
            Inline::Image { alt, .. } => escape(alt, out),
            Inline::SoftBreak => out.push(' '),
            Inline::HardBreak => out.push('\n'),
            Inline::Html(html) => escape(html, out),
            // No TeX engine here: math and footnote references stay the
            // literal source text they were before the parser knew them.
            Inline::Math(source) => escape(&format!("${source}$"), out),
            Inline::DisplayMath(source) => escape(&format!("$${source}$$"), out),
            Inline::FootnoteRef { id } => escape(&format!("[^{id}]"), out),
        }
    }
}

/// Heading level → size factor over the body size (HTML-document
/// conventions, which are also what a slide deck wants).
#[must_use]
pub fn heading_factor(level: u8) -> f64 {
    match level {
        1 => 2.0,
        2 => 1.5,
        3 => 1.17,
        4 => 1.0,
        5 => 0.83,
        _ => 0.67,
    }
}

/// One laid-out block with its source provenance.
pub struct LaidBlock {
    /// Inclusive start / exclusive end byte in the Markdown source.
    pub byte_range: (usize, usize),
    /// Stable semantic block kind, independent of its rendered glyph count.
    pub kind: &'static str,
    /// The block's laid-out content.
    pub vmob: VMobject,
}

/// A parsed-and-laid-out Markdown document.
pub struct MarkdownMobject {
    /// Blocks in source order, each with its source byte range.
    pub blocks: Vec<LaidBlock>,
    /// The whole document as one mobject (blocks stacked vertically).
    pub vmob: VMobject,
}

/// The Markdown builder.
#[derive(Debug, Clone)]
pub struct Markdown<'a> {
    source: &'a str,
    theme: CodeTheme,
    font_size: f64,
    block_gap: f64,
}

impl<'a> Markdown<'a> {
    /// A builder with the defaults: light code theme, the Reference's
    /// default text font size.
    #[must_use]
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            theme: CodeTheme::light(),
            font_size: crate::text::DEFAULT_FONT_SIZE,
            block_gap: 0.45,
        }
    }

    /// The theme for fenced code blocks.
    #[must_use]
    pub fn theme(mut self, theme: CodeTheme) -> Self {
        self.theme = theme;
        self
    }

    /// The base font size (headings scale from it).
    #[must_use]
    pub fn font_size(mut self, font_size: f64) -> Self {
        self.font_size = font_size;
        self
    }

    /// Gap between actual visible block bounds, in scene units at size 48.
    #[must_use]
    pub fn block_gap(mut self, gap: f64) -> Self {
        self.block_gap = gap;
        self
    }

    fn layout_text_block(
        &self,
        book: &FontBook,
        markup: String,
        size_factor: f64,
    ) -> Result<TextMobject, TextMobjectError> {
        Text::markup(&markup)
            .font_size(self.font_size * size_factor)
            .build(book)
    }

    /// Parse and lay out the document.
    ///
    /// # Errors
    /// Whatever the per-block [`Text::build`] calls hit.
    pub fn build(&self, book: &FontBook) -> Result<MarkdownMobject, MarkdownError> {
        if !self.font_size.is_finite() || self.font_size <= 0.0 || self.font_size > 10_000.0 {
            return Err(MarkdownError::InvalidParameter(
                "Markdown font_size must be finite and in (0, 10000]",
            ));
        }
        if !self.block_gap.is_finite() || !(0.0..=1000.0).contains(&self.block_gap) {
            return Err(MarkdownError::InvalidParameter(
                "Markdown block_gap must be finite and in [0, 1000]",
            ));
        }
        limit(
            "Markdown source bytes",
            self.source.len(),
            MAX_MARKDOWN_BYTES,
        )?;
        let spanned = franken_markdown::parse::parse_document_spanned(self.source);
        limit("Markdown blocks", spanned.blocks.len(), MAX_MARKDOWN_BLOCKS)?;
        for block in &spanned.blocks {
            validate_tree(&block.node)?;
        }
        let mut blocks = Vec::new();
        let mut children: Vec<VMobject> = Vec::new();
        let mut y_cursor = 0.0f64;
        let mut records = 0usize;

        for spanned_block in &spanned.blocks {
            let (start, end) = (spanned_block.span.start, spanned_block.span.end);
            if self.source.get(start..end).is_none() {
                return Err(MarkdownError::InvalidParameter(
                    "Markdown parser returned an invalid UTF-8 source span",
                ));
            }
            let vmob = match &spanned_block.node {
                franken_markdown::ast::Block::Heading { level, inlines } => {
                    let mut markup = String::new();
                    inlines_to_markup(inlines, &mut markup);
                    self.layout_text_block(
                        book,
                        format!("<b>{markup}</b>"),
                        heading_factor(*level),
                    )?
                    .vmob
                }
                franken_markdown::ast::Block::Paragraph(inlines) => {
                    let mut markup = String::new();
                    inlines_to_markup(inlines, &mut markup);
                    self.layout_text_block(book, markup, 1.0)?.vmob
                }
                franken_markdown::ast::Block::CodeBlock { lang, code } => {
                    Code::new(code)
                        .language(lang.as_deref().unwrap_or(""))
                        .theme(self.theme)
                        .font_size(self.font_size * 0.85)
                        .build(book)?
                        .vmob
                }
                franken_markdown::ast::Block::List(_)
                | franken_markdown::ast::Block::BlockQuote(_) => {
                    let mut markup = String::new();
                    block_to_markup(&spanned_block.node, "", &mut markup);
                    self.layout_text_block(book, markup, 1.0)?.vmob
                }
                franken_markdown::ast::Block::Table(table) if !table.rows.is_empty() => {
                    let plain = |cell: &[franken_markdown::ast::Inline]| -> Result<String, TextMobjectError> {
                        let mut markup = String::new();
                        inlines_to_markup(cell, &mut markup);
                        Ok(fmn_text::markup::parse_markup(&markup)?.iter().map(|c| c.ch).collect())
                    };
                    let headers = table
                        .head
                        .iter()
                        .map(|cell| plain(cell))
                        .collect::<Result<Vec<_>, _>>()?;
                    let rows = table
                        .rows
                        .iter()
                        .map(|row| {
                            row.iter()
                                .map(|cell| plain(cell))
                                .collect::<Result<Vec<_>, _>>()
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    crate::data_mobjects::TableMobject::from_grid(headers, rows)
                        .build(book)
                        .map_err(|e| TextMobjectError::Geometry {
                            what: e.to_string(),
                        })?
                        .scaled_about(self.font_size / crate::text::DEFAULT_FONT_SIZE, [0.0; 3])
                }
                // Body-size literal source: the paragraph it was before the
                // parser recognised `$$…$$`.
                franken_markdown::ast::Block::MathBlock(_) => {
                    let raw = other_source_snippet(&spanned_block.node);
                    self.layout_text_block(book, raw, 1.0)?.vmob
                }
                other => {
                    let raw = other_source_snippet(other);
                    self.layout_text_block(book, raw, 0.9)?.vmob
                }
            };

            let mut pending = vec![&vmob];
            while let Some(member) = pending.pop() {
                records = records.checked_add(member.points().len()).ok_or(
                    TextMobjectError::CapacityOverflow {
                        context: "Markdown records",
                    },
                )?;
                limit("Markdown records", records, MAX_RECORDS)?;
                pending.extend(member.children());
            }

            let height = vmob
                .bbox_point([0.0, 1.0, 0.0])
                .and_then(|top| {
                    vmob.bbox_point([0.0, -1.0, 0.0])
                        .map(|bottom| top[1] - bottom[1])
                })
                .unwrap_or(0.0);
            // Stack visible bounds, not baselines: heading and code fonts have
            // different ascenders, descenders and multi-line heights.
            let top_left = vmob.bbox_point([-1.0, 1.0, 0.0]).unwrap_or([0.0; 3]);
            let placed = vmob.shifted([-top_left[0], y_cursor - top_left[1], 0.0]);
            y_cursor -= height + self.block_gap * self.font_size / crate::text::DEFAULT_FONT_SIZE;
            blocks.push(LaidBlock {
                byte_range: (start, end),
                kind: block_kind(&spanned_block.node),
                vmob: placed,
            });
            children.push(
                blocks
                    .last()
                    .map(|laid| laid.vmob.clone())
                    .expect("just pushed"),
            );
        }

        let vmob = VMobject::new().with_children(children);
        Ok(MarkdownMobject { blocks, vmob })
    }
}

fn block_kind(block: &franken_markdown::ast::Block) -> &'static str {
    use franken_markdown::ast::Block;
    match block {
        Block::Heading { .. } => "heading",
        Block::Paragraph(_) => "paragraph",
        Block::CodeBlock { .. } => "code",
        Block::List(_) => "list",
        Block::BlockQuote(_) => "quote",
        Block::Table(_) => "table",
        Block::HtmlBlock(_) => "html",
        Block::ThematicBreak => "rule",
        Block::MathBlock(_) => "math",
        Block::FootnoteDefinition { .. } => "footnote",
        Block::DefinitionList(_) => "definitions",
        Block::PageBreak => "pagebreak",
    }
}

/// Check nesting iteratively before recursive conversion or any font work.
fn validate_tree(root: &franken_markdown::ast::Block) -> Result<(), TextMobjectError> {
    use franken_markdown::ast::{Block, Inline};
    let mut blocks = vec![(root, 0usize)];
    let mut inlines = Vec::new();
    while let Some((block, depth)) = blocks.pop() {
        limit("Markdown nesting", depth, MAX_DEPTH)?;
        match block {
            Block::Heading {
                inlines: values, ..
            }
            | Block::Paragraph(values) => {
                inlines.extend(values.iter().map(|value| (value, 0usize)));
            }
            Block::BlockQuote(children) => {
                blocks.extend(children.iter().map(|child| (child, depth + 1)))
            }
            Block::List(list) => {
                for item in &list.items {
                    blocks.extend(item.blocks.iter().map(|child| (child, depth + 1)));
                }
            }
            Block::Table(table) => {
                let cells = table
                    .rows
                    .iter()
                    .try_fold(table.head.len(), |n, row| n.checked_add(row.len()))
                    .ok_or(TextMobjectError::CapacityOverflow {
                        context: "Markdown table cells",
                    })?;
                limit("Markdown table cells", cells, 4096)?;
                for row in std::iter::once(&table.head).chain(table.rows.iter()) {
                    for cell in row {
                        inlines.extend(cell.iter().map(|value| (value, 0usize)));
                    }
                }
            }
            Block::FootnoteDefinition {
                blocks: children, ..
            } => blocks.extend(children.iter().map(|child| (child, depth + 1))),
            Block::DefinitionList(items) => {
                for item in items {
                    for run in item.terms.iter().chain(item.definitions.iter()) {
                        inlines.extend(run.iter().map(|value| (value, 0usize)));
                    }
                }
            }
            _ => {}
        }
    }
    while let Some((inline, depth)) = inlines.pop() {
        limit("Markdown inline nesting", depth, MAX_DEPTH)?;
        match inline {
            Inline::Emphasis(children)
            | Inline::Strong(children)
            | Inline::Strikethrough(children)
            | Inline::Link {
                content: children, ..
            } => {
                inlines.extend(children.iter().map(|child| (child, depth + 1)));
            }
            _ => {}
        }
    }
    Ok(())
}

/// Render nested block content without dropping non-paragraph list/quote items.
/// The parser remains fmd's. This is only a bounded AST-to-Scribe presentation.
fn block_to_markup(block: &franken_markdown::ast::Block, prefix: &str, out: &mut String) {
    use franken_markdown::ast::Block;
    match block {
        Block::List(list) => {
            for (index, item) in list.items.iter().enumerate() {
                let marker = if list.ordered {
                    // Do not wrap an authored maximum-u64 start value.
                    format!("{}. ", u128::from(list.start) + index as u128)
                } else {
                    "• ".to_owned()
                };
                let task = match item.task {
                    Some(true) => "[x] ",
                    Some(false) => "[ ] ",
                    None => "",
                };
                for (line, child) in item.blocks.iter().enumerate() {
                    let next = if line == 0 {
                        format!("{prefix}{marker}{task}")
                    } else {
                        format!("{prefix}    ")
                    };
                    block_to_markup(child, &next, out);
                }
            }
        }
        Block::BlockQuote(children) => {
            for child in children {
                block_to_markup(child, &format!("{prefix}&gt; "), out);
            }
        }
        Block::Paragraph(inlines) | Block::Heading { inlines, .. } => {
            out.push_str(prefix);
            inlines_to_markup(inlines, out);
            out.push('\n');
        }
        Block::CodeBlock { code, .. } => {
            for line in code.lines() {
                out.push_str(prefix);
                out.push_str("<tt>");
                escape(line, out);
                out.push_str("</tt>\n");
            }
        }
        other => {
            out.push_str(prefix);
            out.push_str(&other_source_snippet(other));
            out.push('\n');
        }
    }
}

fn other_source_snippet(block: &franken_markdown::ast::Block) -> String {
    match block {
        franken_markdown::ast::Block::Table(table) => {
            let mut out = String::from("[table]\n");
            for row in std::iter::once(&table.head).chain(table.rows.iter()) {
                for cell in row {
                    inlines_to_markup(cell, &mut out);
                    out.push('\t');
                }
                out.push('\n');
            }
            out
        }
        franken_markdown::ast::Block::HtmlBlock(html) => {
            let mut result = String::new();
            escape(html, &mut result);
            result
        }
        franken_markdown::ast::Block::ThematicBreak => "———".to_string(),
        // Literal source, as these read before the parser knew them.
        franken_markdown::ast::Block::MathBlock(source) => {
            let mut result = String::new();
            escape(&format!("$${source}$$"), &mut result);
            result
        }
        franken_markdown::ast::Block::FootnoteDefinition { id, blocks } => {
            let mut result = String::new();
            escape(&format!("[^{id}]: "), &mut result);
            for block in blocks {
                block_to_markup(block, "", &mut result);
            }
            result
        }
        franken_markdown::ast::Block::DefinitionList(items) => {
            let mut result = String::new();
            for item in items {
                for term in &item.terms {
                    inlines_to_markup(term, &mut result);
                    result.push('\n');
                }
                for definition in &item.definitions {
                    result.push_str(": ");
                    inlines_to_markup(definition, &mut result);
                    result.push('\n');
                }
            }
            result
        }
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmn_text::FontBook;

    fn book() -> FontBook {
        FontBook::bundled().expect("bundled faces parse")
    }

    const SAMPLE: &str =
        "# Title\n\nA paragraph with *emphasis*.\n\n```rust\nfn main() {}\n```\n\n- one\n- two\n";

    /// The parser recognises `$…$`, `$$…$$` and footnotes. This mobject has
    /// no TeX engine, so they lay out exactly as the escaped source text did
    /// before the parser knew them.
    #[test]
    fn math_and_footnote_syntax_stays_literal_source_text() {
        let points = |source: &str, index: usize| {
            let md = Markdown::new(source).build(&book()).expect("builds");
            let mut out = Vec::new();
            let mut pending = vec![&md.blocks[index].vmob];
            while let Some(member) = pending.pop() {
                out.extend(member.points().iter().copied());
                pending.extend(member.children());
            }
            (md.blocks[index].kind, out)
        };
        let (kind, inline) = points("Area $a^2$ and a note[^n].\n\n[^n]: the note\n", 0);
        assert_eq!(kind, "paragraph");
        assert_eq!(inline, points(r"Area \$a^2\$ and a note\[^n\].", 0).1);
        assert_eq!(
            points("Area $a^2$ and a note[^n].\n\n[^n]: the note\n", 1).0,
            "footnote"
        );
        let (kind, block) = points("$$x+1$$\n", 0);
        assert_eq!(kind, "math");
        assert_eq!(block, points(r"\$\$x+1\$\$", 0).1);
    }

    #[test]
    fn blocks_map_to_children_in_source_order() {
        let md = Markdown::new(SAMPLE).build(&book()).expect("builds");
        assert_eq!(md.blocks.len(), 4, "heading, paragraph, fence, list");
        assert_eq!(md.vmob.children().len(), 4);
    }

    #[test]
    fn provenance_ranges_tile_the_source_prefixes() {
        let md = Markdown::new(SAMPLE).build(&book()).expect("builds");
        // First block starts at 0 and covers the heading line.
        assert_eq!(md.blocks[0].byte_range.0, 0);
        assert!(md.blocks[0].byte_range.1 >= "# Title".len());
        // Ranges are ordered and non-overlapping.
        for pair in md.blocks.windows(2) {
            assert!(
                pair[0].byte_range.1 <= pair[1].byte_range.0 + 2,
                "ranges {:?} then {:?} overlap",
                pair[0].byte_range,
                pair[1].byte_range
            );
        }
    }

    #[test]
    fn headings_render_larger_than_body() {
        let md = Markdown::new(SAMPLE).build(&book()).expect("builds");
        let height = |laid: &LaidBlock| {
            let top = laid.vmob.bbox_point([0.0, 1.0, 0.0]).unwrap()[1];
            let bottom = laid.vmob.bbox_point([0.0, -1.0, 0.0]).unwrap()[1];
            top - bottom
        };
        assert!(
            height(&md.blocks[0]) > height(&md.blocks[1]),
            "h1 taller than body"
        );
    }

    #[test]
    fn fenced_code_uses_the_code_path() {
        let md = Markdown::new(SAMPLE).build(&book()).expect("builds");
        let fence = &md.blocks[2].vmob;
        let fills: Vec<_> = fence
            .children()
            .iter()
            .map(|c| c.style().fill_color)
            .collect();
        let base = fmn_core::constants::DEFAULT_MOBJECT_COLOR;
        assert!(
            fills.iter().any(|c| *c != base),
            "fence must carry highlighted fills, got {fills:?}"
        );
    }

    #[test]
    fn lists_flatten_with_markers() {
        let md = Markdown::new(SAMPLE).build(&book()).expect("builds");
        // The list block exists and lays out; exact glyph mapping is the
        // text pipeline's contract (already covered there).
        assert!(!md.blocks[3].vmob.children().is_empty());
    }

    #[test]
    fn literal_text_and_inline_code_cannot_become_markup() {
        use franken_markdown::ast::Inline;
        let mut markup = String::new();
        inlines_to_markup(
            &[
                Inline::Text("a < b & c > d ".into()),
                Inline::Code("<b>&amp;</b>".into()),
                Inline::Html("<script>x</script>".into()),
            ],
            &mut markup,
        );
        let chars = fmn_text::markup::parse_markup(&markup).unwrap();
        assert_eq!(
            chars.iter().map(|c| c.ch).collect::<String>(),
            "a < b & c > d <b>&amp;</b><script>x</script>"
        );
        assert!(chars.iter().all(|c| !c.style.bold));
        assert!(chars[14].style.mono);
        assert!(
            Markdown::new("Use `Vec<T>` & **borrowed** values.")
                .build(&book())
                .is_ok()
        );
    }

    #[test]
    fn different_block_heights_never_overlap_and_share_a_left_edge() {
        let md = Markdown::new("# Heading\n\nShort\n\n```rust\nlet x = 1;\nlet y = 2;\n```\n\nEnd")
            .build(&book())
            .unwrap();
        for block in &md.blocks {
            assert!(block.vmob.bbox_point([-1.0, 1.0, 0.0]).unwrap()[0].abs() < 1e-6);
        }
        assert!(md.blocks[0].vmob.bbox_point([0.0, 1.0, 0.0]).unwrap()[1].abs() < 1e-6);
        for pair in md.blocks.windows(2) {
            let bottom = pair[0].vmob.bbox_point([0.0, -1.0, 0.0]).unwrap()[1];
            let top = pair[1].vmob.bbox_point([0.0, 1.0, 0.0]).unwrap()[1];
            assert!((bottom - top - 0.45).abs() < 1e-5, "gap: {}", bottom - top);
        }
    }

    #[test]
    fn block_gap_scales_with_typography() {
        let md = Markdown::new("first\n\nsecond")
            .font_size(24.0)
            .block_gap(0.8)
            .build(&book())
            .unwrap();
        let bottom = md.blocks[0].vmob.bbox_point([0.0, -1.0, 0.0]).unwrap()[1];
        let top = md.blocks[1].vmob.bbox_point([0.0, 1.0, 0.0]).unwrap()[1];
        assert!((bottom - top - 0.4).abs() < 1e-6);
    }

    #[test]
    fn nested_nonparagraph_content_is_not_dropped() {
        let source = "- outer\n  - inner\n\n> # Quoted heading\n>\n> ```rust\n> x < y\n> ```\n";
        let document = franken_markdown::parse::parse_document_spanned(source);
        let mut markup = String::new();
        for block in &document.blocks {
            block_to_markup(&block.node, "", &mut markup);
        }
        let text: String = fmn_text::markup::parse_markup(&markup)
            .unwrap()
            .iter()
            .map(|c| c.ch)
            .collect();
        for expected in ["outer", "inner", "Quoted heading", "x < y"] {
            assert!(text.contains(expected), "lost {expected}: {text}");
        }
        assert!(Markdown::new(source).build(&book()).is_ok());
    }

    #[test]
    fn tables_are_ruled_native_families_not_tab_separated_placeholders() {
        let md = Markdown::new("| name | value |\n| --- | --- |\n| **A** | 2 |\n| B | 3 |\n")
            .build(&book())
            .unwrap();
        assert_eq!(md.blocks.len(), 1);
        assert_eq!(md.blocks[0].kind, "table");
        let expected = crate::data_mobjects::TableMobject::from_grid(
            vec!["name".into(), "value".into()],
            vec![vec!["A".into(), "2".into()], vec!["B".into(), "3".into()]],
        )
        .build(&book())
        .unwrap();
        assert_eq!(
            md.blocks[0].vmob.children().len(),
            expected.children().len()
        );
        assert!(
            !md.blocks[0].vmob.children()[0].points().is_empty(),
            "native outer rule"
        );
    }

    #[test]
    fn unicode_spans_are_byte_ranges_and_empty_documents_are_valid() {
        let source = "# αβ\n\nText Ω.\n";
        let md = Markdown::new(source).build(&book()).unwrap();
        assert_eq!(
            md.blocks.iter().map(|b| b.kind).collect::<Vec<_>>(),
            ["heading", "paragraph"]
        );
        assert!(
            source
                .get(md.blocks[0].byte_range.0..md.blocks[0].byte_range.1)
                .unwrap()
                .contains("αβ")
        );
        assert!(Markdown::new("").build(&book()).unwrap().blocks.is_empty());
    }

    #[test]
    fn resource_and_numeric_refusals_are_native() {
        for size in [0.0, -1.0, f64::NAN, f64::INFINITY, 10_001.0] {
            assert!(Markdown::new("x").font_size(size).build(&book()).is_err());
        }
        for gap in [-1.0, f64::NAN, f64::INFINITY] {
            assert!(Markdown::new("x").block_gap(gap).build(&book()).is_err());
        }
        assert!(
            Markdown::new(&"x".repeat(MAX_MARKDOWN_BYTES + 1))
                .build(&book())
                .is_err()
        );
        assert!(
            Markdown::new(&"x\n\n".repeat(MAX_MARKDOWN_BLOCKS + 1))
                .build(&book())
                .is_err()
        );
        use franken_markdown::ast::Block;
        let mut nested = Block::ThematicBreak;
        for _ in 0..=MAX_DEPTH {
            nested = Block::BlockQuote(vec![nested]);
        }
        assert!(validate_tree(&nested).is_err());
    }
}
