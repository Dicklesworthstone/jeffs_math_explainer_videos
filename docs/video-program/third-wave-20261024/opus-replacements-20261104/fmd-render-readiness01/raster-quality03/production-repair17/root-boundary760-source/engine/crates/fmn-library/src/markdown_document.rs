//! Animatable Markdown documents over the one fmd parser, Scribe and Tex engine.
//! Source ranges belong to top-level blocks; words/formulas remain native child
//! families. No HTML rendering, external assets or host font discovery occurs.
#[path = "markdown_flow.rs"]
mod flow;
#[path = "markdown_math.rs"]
mod math;

use crate::code::{Code, CodeTheme};
use crate::data_mobjects::TableMobject;
use crate::line::Line;
use crate::tex::{Tex, TexMobjectError, TexText};
use crate::text::{DEFAULT_FONT_SIZE_FOR_UNIT_HEIGHT, Text, TextMobjectError};
use crate::vmobject::VMobject;
use flow::InlineBox;
use fmn_core::color::Srgb;
use fmn_core::constants::{ORIGIN, WHITE};
use fmn_tex::TexEngine;
use fmn_text::FontBook;
use franken_markdown::ast::{Block, Inline};

/// Bounded document admission, shared by native and portal entry points.
const MARKDOWN_MAX_BYTES: usize = super::MAX_MARKDOWN_BYTES;
const MAX_ATOMS: usize = 8192;
const MAX_GLYPHS: usize = 32768;
const MAX_POINTS: usize = 1_048_576;

/// A native document error; typesetting errors retain their original detail.
#[derive(Debug)]
pub enum MathDocumentError {
    /// Invalid dimension or configuration.
    Invalid(&'static str),
    /// The declared work budget was exceeded before another layout operation.
    Limit(&'static str),
    /// Unsupported asset or missing math capability.
    Unsupported(&'static str),
    /// Scribe text failure.
    Text(TextMobjectError),
    /// Native formula failure.
    Math(TexMobjectError),
    /// A structural geometry failure.
    Geometry(String),
}
impl std::fmt::Display for MathDocumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(s) | Self::Limit(s) | Self::Unsupported(s) => f.write_str(s),
            Self::Text(e) => e.fmt(f),
            Self::Math(e) => e.fmt(f),
            Self::Geometry(s) => f.write_str(s),
        }
    }
}
impl std::error::Error for MathDocumentError {}
impl From<TextMobjectError> for MathDocumentError {
    fn from(e: TextMobjectError) -> Self {
        Self::Text(e)
    }
}
impl From<TexMobjectError> for MathDocumentError {
    fn from(e: TexMobjectError) -> Self {
        Self::Math(e)
    }
}

use super::{LaidBlock, MarkdownMobject, heading_factor};

/// Immutable, bounded native document builder.
#[derive(Debug, Clone)]
struct Composer<'a> {
    source: &'a str,
    theme: CodeTheme,
    font_size: f64,
    width: Option<f64>,
    block_gap: f64,
    color: Srgb,
}

#[derive(Default)]
struct Budget {
    atoms: usize,
    glyphs: usize,
    points: usize,
}
impl Budget {
    fn input(&mut self, source: &str) -> Result<(), MathDocumentError> {
        self.atoms += 1;
        self.glyphs = self.glyphs.saturating_add(source.chars().count());
        if self.atoms > MAX_ATOMS || self.glyphs > MAX_GLYPHS {
            return Err(MathDocumentError::Limit(
                "Markdown exceeds 8192 layout atoms or 32768 shaped characters",
            ));
        }
        Ok(())
    }
    fn output(&mut self, geometry: &VMobject) -> Result<(), MathDocumentError> {
        fn count(v: &VMobject) -> usize {
            v.children()
                .iter()
                .fold(v.points().len(), |n, c| n.saturating_add(count(c)))
        }
        self.points = self.points.saturating_add(count(geometry));
        if self.points > MAX_POINTS {
            return Err(MathDocumentError::Limit(
                "Markdown exceeds 1048576 native points",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Default)]
struct Face {
    bold: bool,
    italic: bool,
    mono: bool,
    strike: bool,
}

fn escaped(source: &str) -> String {
    let mut out = String::new();
    for ch in source.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
    out
}

fn top_left(v: VMobject, x: f64, y: f64) -> VMobject {
    match v.extent() {
        Some((lo, hi)) => v.shifted([x - lo[0], y - hi[1], 0.0]),
        None => v,
    }
}
fn stack(items: Vec<VMobject>, gap: f64) -> VMobject {
    let mut y = 0.0;
    let mut placed = Vec::new();
    for item in items {
        let h = item.length_over_dim(1);
        placed.push(top_left(item, 0.0, y));
        y -= h + gap;
    }
    if placed.len() == 1 {
        return placed.pop().expect("one item");
    }
    VMobject::new().with_children(placed)
}
fn kind(block: &Block) -> &'static str {
    match block {
        Block::Heading { .. } => "heading",
        Block::Paragraph(_) => "paragraph",
        Block::CodeBlock { lang, .. }
            if lang
                .as_deref()
                .is_some_and(|l| matches!(l, "math" | "tex" | "textext")) =>
        {
            "math"
        }
        Block::CodeBlock { .. } => "code",
        Block::List(_) => "list",
        Block::BlockQuote(_) => "quote",
        Block::Table(_) => "table",
        Block::ThematicBreak => "rule",
        Block::HtmlBlock(_) => "literal",
        Block::MathBlock(_) => "math",
        Block::FootnoteDefinition { .. } => "footnote",
        Block::DefinitionList(_) => "definitions",
        Block::PageBreak => "pagebreak",
    }
}

impl<'a> Composer<'a> {
    fn build_inner(
        &self,
        book: &FontBook,
        engine: Option<&TexEngine>,
    ) -> Result<MarkdownMobject, MathDocumentError> {
        if self.source.len() > MARKDOWN_MAX_BYTES {
            return Err(MathDocumentError::Limit(
                "Markdown source exceeds 32768 UTF-8 bytes",
            ));
        }
        if !self.font_size.is_finite()
            || self.font_size <= 0.0
            || self.font_size > 10000.0
            || self
                .width
                .is_some_and(|w| !w.is_finite() || w <= 0.0 || w > 1e6)
            || !self.block_gap.is_finite()
            || !(0.0..=1000.0).contains(&self.block_gap)
            || [self.color.r, self.color.g, self.color.b]
                .iter()
                .any(|c| !c.is_finite() || !(0.0..=1.0).contains(c))
        {
            return Err(MathDocumentError::Invalid(
                "Markdown requires a finite positive font size/width, nonnegative gap and RGB in [0,1]",
            ));
        }
        let parsed = franken_markdown::parse::parse_document_spanned(self.source);
        if parsed.blocks.len() > super::MAX_MARKDOWN_BLOCKS {
            return Err(MathDocumentError::Limit(
                "Markdown exceeds 256 top-level blocks",
            ));
        }
        // Protect each original block, but reparse the normalized document as
        // a whole. Reference-link definitions outside a paragraph must remain
        // in the parser context when that paragraph contains mathematics.
        let mut normalized = String::new();
        let mut prepared = Vec::new();
        let mut cursor = 0;
        for original in &parsed.blocks {
            super::validate_tree(&original.node)?;
            let raw = original
                .span
                .slice(self.source)
                .ok_or(MathDocumentError::Invalid(
                    "Markdown parser returned an invalid source span",
                ))?;
            if original.span.start < cursor {
                return Err(MathDocumentError::Invalid(
                    "Markdown parser returned overlapping block ranges",
                ));
            }
            normalized.push_str(&self.source[cursor..original.span.start]);
            let start = normalized.len();
            let protected = if matches!(
                &original.node,
                Block::CodeBlock { .. } | Block::HtmlBlock(_)
            ) {
                None
            } else {
                Some(math::protect(raw)?)
            };
            normalized.push_str(protected.as_ref().map_or(raw, |part| part.source.as_str()));
            prepared.push((start, normalized.len(), protected));
            cursor = original.span.end;
        }
        normalized.push_str(&self.source[cursor..]);
        let resolved = franken_markdown::parse::parse_document_spanned(&normalized);
        let mut budget = Budget::default();
        let mut blocks = Vec::new();
        let mut y = 0.0;
        for (original, (start, end, protected)) in parsed.blocks.iter().zip(prepared) {
            let empty = math::Protected {
                source: String::new(),
                prefix: String::new(),
                islands: Vec::new(),
            };
            let content = if let Some(protected) = protected.filter(|part| !part.islands.is_empty())
            {
                let mut parts = Vec::new();
                for part in resolved
                    .blocks
                    .iter()
                    .filter(|part| start <= part.span.start && part.span.start < end)
                {
                    if part.span.end > end {
                        return Err(MathDocumentError::Invalid(
                            "Markdown math normalization crossed a source block boundary",
                        ));
                    }
                    super::validate_tree(&part.node)?;
                    parts.push(self.block(
                        book,
                        engine,
                        &part.node,
                        &protected,
                        &mut budget,
                        0,
                        self.width,
                    )?);
                }
                if parts.is_empty() {
                    return Err(MathDocumentError::Invalid(
                        "Markdown math normalization lost a source block",
                    ));
                }
                stack(parts, self.block_gap)
            } else {
                self.block(
                    book,
                    engine,
                    &original.node,
                    &empty,
                    &mut budget,
                    0,
                    self.width,
                )?
            };
            let height = content.length_over_dim(1);
            let placed = top_left(content, 0.0, y);
            y -= height + self.block_gap;
            blocks.push(LaidBlock {
                byte_range: (original.span.start, original.span.end),
                kind: kind(&original.node),
                vmob: placed,
            });
        }
        let vmob = VMobject::new().with_children(blocks.iter().map(|b| b.vmob.clone()));
        Ok(MarkdownMobject { blocks, vmob })
    }

    #[allow(clippy::too_many_arguments)]
    fn block(
        &self,
        book: &FontBook,
        engine: Option<&TexEngine>,
        node: &Block,
        protected: &math::Protected,
        budget: &mut Budget,
        depth: usize,
        width: Option<f64>,
    ) -> Result<VMobject, MathDocumentError> {
        if depth > 24 {
            return Err(MathDocumentError::Limit(
                "Markdown nesting exceeds 24 levels",
            ));
        }
        budget.input("")?;
        match node {
            Block::Paragraph(inlines) => self.prose(
                book,
                engine,
                inlines,
                protected,
                budget,
                Face::default(),
                self.font_size,
                width,
            ),
            Block::Heading { level, inlines } => self.prose(
                book,
                engine,
                inlines,
                protected,
                budget,
                Face {
                    bold: true,
                    ..Face::default()
                },
                self.font_size * heading_factor(*level),
                width,
            ),
            Block::CodeBlock { lang, code } => {
                budget.input(code)?;
                let geometry = match lang.as_deref() {
                    Some("math" | "tex" | "textext") => {
                        let engine = engine.ok_or(MathDocumentError::Unsupported(
                            "Markdown mathematics requires build_with_math",
                        ))?;
                        if lang.as_deref() == Some("textext") {
                            TexText::new(code)
                                .font_size(self.font_size)
                                .build(engine)?
                                .vmob
                                .map_style_deep(|s| s.color(self.color))
                        } else {
                            Tex::new(code)
                                .display()
                                .font_size(self.font_size)
                                .build(engine)?
                                .vmob
                                .map_style_deep(|s| s.color(self.color))
                        }
                    }
                    _ => {
                        Code::new(code)
                            .language(lang.as_deref().unwrap_or(""))
                            .theme(self.theme)
                            .font_size(self.font_size * 0.85)
                            .build(book)?
                            .vmob
                    }
                };
                budget.output(&geometry)?;
                Ok(geometry)
            }
            Block::BlockQuote(inner) => {
                let indent = self.font_size / 96.0;
                let mut items = Vec::new();
                for child in inner {
                    items.push(self.block(
                        book,
                        engine,
                        child,
                        protected,
                        budget,
                        depth + 1,
                        width.map(|w| (w - indent).max(indent)),
                    )?);
                }
                let content = stack(items, self.block_gap * 0.5);
                let height = content.length_over_dim(1);
                let rule = Line::new([0.0, 0.0, 0.0], [0.0, -height, 0.0])
                    .build()
                    .map_err(|e| MathDocumentError::Geometry(e.to_string()))?
                    .with_color(self.color);
                budget.output(&rule)?;
                Ok(VMobject::new().with_children([rule, content.shifted([indent, 0.0, 0.0])]))
            }
            Block::List(list) => {
                let indent = self.font_size / 48.0;
                let mut items = Vec::new();
                for (index, item) in list.items.iter().enumerate() {
                    let marker = if list.ordered {
                        format!(
                            "{}.",
                            list.start.checked_add(index as u64).ok_or(
                                MathDocumentError::Limit("Markdown ordered list index overflow")
                            )?
                        )
                    } else {
                        "•".to_owned()
                    };
                    let marker = match item.task {
                        Some(true) => format!("{marker} [x]"),
                        Some(false) => format!("{marker} [ ]"),
                        None => marker,
                    };
                    let marker =
                        self.text_box(book, &marker, Face::default(), self.font_size, budget)?;
                    let margin = indent.max(marker.advance + self.font_size / 144.0);
                    let mut content = Vec::new();
                    for child in &item.blocks {
                        content.push(self.block(
                            book,
                            engine,
                            child,
                            protected,
                            budget,
                            depth + 1,
                            width.map(|w| (w - margin).max(indent)),
                        )?);
                    }
                    let content = stack(content, self.block_gap * 0.5);
                    items.push(VMobject::new().with_children([
                        top_left(marker.geometry, 0.0, 0.0),
                        content.shifted([margin, 0.0, 0.0]),
                    ]));
                }
                Ok(stack(items, self.block_gap * 0.5))
            }
            Block::Table(table) => {
                if table.head.is_empty()
                    || table.head.len() > 64
                    || (table.rows.len() + 1).saturating_mul(table.head.len()) > 4096
                {
                    return Err(MathDocumentError::Limit(
                        "Markdown table exceeds 64 columns or 4096 cells",
                    ));
                }
                let plain = |inlines: &[Inline]| -> Result<String, MathDocumentError> {
                    plain_inlines(inlines, protected, 0)
                };
                let headers = table
                    .head
                    .iter()
                    .map(|c| plain(c))
                    .collect::<Result<Vec<_>, _>>()?;
                let rows = table
                    .rows
                    .iter()
                    .map(|r| r.iter().map(|c| plain(c)).collect::<Result<Vec<_>, _>>())
                    .collect::<Result<Vec<_>, _>>()?;
                for cell in headers.iter().chain(rows.iter().flatten()) {
                    budget.input(cell)?;
                }
                let geometry = TableMobject::from_grid(headers, rows)
                    .build(book)
                    .map_err(|e| MathDocumentError::Geometry(e.to_string()))?
                    .scaled_about(self.font_size / 48.0, ORIGIN)
                    .map_style_deep(|s| s.color(self.color));
                budget.output(&geometry)?;
                Ok(geometry)
            }
            Block::ThematicBreak => {
                let rule = Line::new(ORIGIN, [width.unwrap_or(4.0), 0.0, 0.0])
                    .build()
                    .map_err(|e| MathDocumentError::Geometry(e.to_string()))?
                    .with_color(self.color);
                budget.output(&rule)?;
                Ok(rule)
            }
            // A `$$…$$` block the parser recognises renders as a math fence.
            Block::MathBlock(source) => self.block(
                book,
                engine,
                &Block::CodeBlock {
                    lang: Some("math".to_owned()),
                    code: source.clone(),
                },
                protected,
                budget,
                depth + 1,
                width,
            ),
            // A footnote definition is a quoted note under its `[^id]:` label.
            Block::FootnoteDefinition { id, blocks } => {
                let mut children = vec![Block::Paragraph(vec![Inline::Text(format!("[^{id}]:"))])];
                children.extend(blocks.iter().cloned());
                self.block(
                    book,
                    engine,
                    &Block::BlockQuote(children),
                    protected,
                    budget,
                    depth + 1,
                    width,
                )
            }
            // Each term in bold, its definitions as a bulleted list beneath.
            Block::DefinitionList(items) => {
                let mut parts = Vec::new();
                for item in items {
                    for term in &item.terms {
                        parts.push(self.block(
                            book,
                            engine,
                            &Block::Paragraph(vec![Inline::Strong(term.clone())]),
                            protected,
                            budget,
                            depth + 1,
                            width,
                        )?);
                    }
                    let definitions = franken_markdown::ast::List {
                        ordered: false,
                        start: 1,
                        tight: true,
                        items: item
                            .definitions
                            .iter()
                            .map(|definition| franken_markdown::ast::ListItem {
                                task: None,
                                blocks: vec![Block::Paragraph(definition.clone())],
                            })
                            .collect(),
                    };
                    parts.push(self.block(
                        book,
                        engine,
                        &Block::List(definitions),
                        protected,
                        budget,
                        depth + 1,
                        width,
                    )?);
                }
                Ok(stack(parts, self.block_gap * 0.5))
            }
            // A canvas has no pages.
            Block::PageBreak => Ok(VMobject::new()),
            Block::HtmlBlock(source) => Ok(self
                .text_box(book, source, Face::default(), self.font_size, budget)?
                .geometry),
        }
    }

    fn text_box(
        &self,
        book: &FontBook,
        source: &str,
        face: Face,
        size: f64,
        budget: &mut Budget,
    ) -> Result<InlineBox, MathDocumentError> {
        budget.input(source)?;
        let markup = if face.strike {
            format!("<s>{}</s>", escaped(source))
        } else {
            escaped(source)
        };
        let mut builder = Text::markup(&markup)
            .font_size(size)
            .bold(face.bold)
            .italic(face.italic);
        if face.mono {
            builder = builder.font(fmn_text::font::MONO_FAMILY);
        }
        let built = builder.build(book)?;
        let scale = crate::text::calibrate(book, size, DEFAULT_FONT_SIZE_FOR_UNIT_HEIGHT)?;
        let geometry = built.vmob.map_style_deep(|s| s.color(self.color));
        budget.output(&geometry)?;
        Ok(InlineBox {
            geometry,
            advance: built
                .layout
                .lines
                .iter()
                .map(|line| line.width)
                .fold(0.0, f64::max)
                * scale,
            height: built.layout.height * scale,
            depth: built.layout.depth * scale,
            space: source.chars().all(char::is_whitespace),
            hard_break: false,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn prose(
        &self,
        book: &FontBook,
        engine: Option<&TexEngine>,
        inlines: &[Inline],
        protected: &math::Protected,
        budget: &mut Budget,
        face: Face,
        size: f64,
        width: Option<f64>,
    ) -> Result<VMobject, MathDocumentError> {
        let mut boxes = Vec::new();
        self.inline_boxes(
            book, engine, inlines, protected, budget, face, size, &mut boxes, 0,
        )?;
        Ok(flow::flow(boxes, width, size / 96.0))
    }

    #[allow(clippy::too_many_arguments)]
    fn inline_boxes(
        &self,
        book: &FontBook,
        engine: Option<&TexEngine>,
        inlines: &[Inline],
        protected: &math::Protected,
        budget: &mut Budget,
        face: Face,
        size: f64,
        boxes: &mut Vec<InlineBox>,
        depth: usize,
    ) -> Result<(), MathDocumentError> {
        if depth > 24 {
            return Err(MathDocumentError::Limit(
                "Markdown inline nesting exceeds 24 levels",
            ));
        }
        for inline in inlines {
            match inline {
                Inline::Text(text) => {
                    let mut rest = text.as_str();
                    while !rest.is_empty() {
                        if let Some(tail) = rest
                            .strip_prefix(&protected.prefix)
                            .filter(|_| !protected.islands.is_empty())
                        {
                            let n = tail.find('Q').ok_or(MathDocumentError::Invalid(
                                "invalid protected math marker",
                            ))?;
                            let index: usize = tail[..n].parse().map_err(|_| {
                                MathDocumentError::Invalid("invalid protected math index")
                            })?;
                            let island =
                                protected
                                    .islands
                                    .get(index)
                                    .ok_or(MathDocumentError::Invalid(
                                        "missing protected math island",
                                    ))?;
                            self.math_box(
                                engine,
                                &island.source,
                                island.display,
                                size,
                                budget,
                                boxes,
                            )?;
                            rest = &tail[n + 1..];
                        } else {
                            let end = if protected.islands.is_empty() {
                                rest.len()
                            } else {
                                rest.find(&protected.prefix).unwrap_or(rest.len())
                            };
                            self.words(book, &rest[..end], face, size, budget, boxes)?;
                            rest = &rest[end..];
                        }
                    }
                }
                Inline::Code(source) => boxes.push(self.text_box(
                    book,
                    source,
                    Face { mono: true, ..face },
                    size,
                    budget,
                )?),
                Inline::Html(source) => self.words(book, source, face, size, budget, boxes)?,
                Inline::Emphasis(inner) => self.inline_boxes(
                    book,
                    engine,
                    inner,
                    protected,
                    budget,
                    Face {
                        italic: true,
                        ..face
                    },
                    size,
                    boxes,
                    depth + 1,
                )?,
                Inline::Strong(inner) => self.inline_boxes(
                    book,
                    engine,
                    inner,
                    protected,
                    budget,
                    Face { bold: true, ..face },
                    size,
                    boxes,
                    depth + 1,
                )?,
                Inline::Strikethrough(inner) => self.inline_boxes(
                    book,
                    engine,
                    inner,
                    protected,
                    budget,
                    Face {
                        strike: true,
                        ..face
                    },
                    size,
                    boxes,
                    depth + 1,
                )?,
                Inline::Link { content, .. } => self.inline_boxes(
                    book,
                    engine,
                    content,
                    protected,
                    budget,
                    face,
                    size,
                    boxes,
                    depth + 1,
                )?,
                Inline::Image { .. } => {
                    return Err(MathDocumentError::Unsupported(
                        "Markdown images require an explicit scene asset; no image is fetched or silently substituted",
                    ));
                }
                Inline::SoftBreak => boxes.push(self.text_box(book, " ", face, size, budget)?),
                Inline::HardBreak => boxes.push(InlineBox::line_break()),
                // The parser's own math nodes, for any `$…$` the island
                // protection left in place: typeset exactly as an island.
                Inline::Math(source) => {
                    self.math_box(engine, source, false, size, budget, boxes)?
                }
                Inline::DisplayMath(source) => {
                    self.math_box(engine, source, true, size, budget, boxes)?
                }
                Inline::FootnoteRef { id } => {
                    self.words(book, &format!("[^{id}]"), face, size, budget, boxes)?
                }
            }
        }
        Ok(())
    }

    /// One mathematics island as an inline box: text style inline, or
    /// display style on its own line.
    fn math_box(
        &self,
        engine: Option<&TexEngine>,
        source: &str,
        display: bool,
        size: f64,
        budget: &mut Budget,
        boxes: &mut Vec<InlineBox>,
    ) -> Result<(), MathDocumentError> {
        let engine = engine.ok_or(MathDocumentError::Unsupported(
            "Markdown mathematics requires build_with_math",
        ))?;
        budget.input(source)?;
        // `$…$` is inline (text-style) mathematics; `$$…$$` is display.
        let style = if display {
            fmn_tex::Style::Display
        } else {
            fmn_tex::Style::Text
        };
        let built = Tex::new(source)
            .font_size(size)
            .math_style(style)
            .build(engine)?;
        let scale = crate::tex::calibrate(engine, size, DEFAULT_FONT_SIZE_FOR_UNIT_HEIGHT)?;
        let geometry = built.vmob.map_style_deep(|s| s.color(self.color));
        budget.output(&geometry)?;
        if display {
            boxes.push(InlineBox::line_break());
        }
        boxes.push(InlineBox {
            geometry,
            advance: built.typeset.layout.width * scale,
            height: built.typeset.layout.height * scale,
            depth: built.typeset.layout.depth * scale,
            space: false,
            hard_break: false,
        });
        if display {
            boxes.push(InlineBox::line_break());
        }
        Ok(())
    }

    fn words(
        &self,
        book: &FontBook,
        source: &str,
        face: Face,
        size: f64,
        budget: &mut Budget,
        boxes: &mut Vec<InlineBox>,
    ) -> Result<(), MathDocumentError> {
        let mut start = 0;
        let mut space = None;
        for (index, ch) in source.char_indices() {
            if space.is_some_and(|s| s != ch.is_whitespace()) {
                boxes.push(self.text_box(book, &source[start..index], face, size, budget)?);
                start = index;
            }
            space = Some(ch.is_whitespace());
        }
        if start < source.len() {
            boxes.push(self.text_box(book, &source[start..], face, size, budget)?);
        }
        Ok(())
    }
}

fn plain_inlines(
    inlines: &[Inline],
    protected: &math::Protected,
    depth: usize,
) -> Result<String, MathDocumentError> {
    if depth > 24 {
        return Err(MathDocumentError::Limit(
            "Markdown table inline nesting exceeds 24 levels",
        ));
    }
    let mut out = String::new();
    for inline in inlines {
        match inline {
            Inline::Text(s) | Inline::Code(s) | Inline::Html(s) => {
                if !protected.islands.is_empty() && s.contains(&protected.prefix) {
                    return Err(MathDocumentError::Unsupported(
                        "Markdown math in table cells is not yet supported; use a native TexMatrix",
                    ));
                }
                out.push_str(s);
            }
            Inline::Emphasis(inner)
            | Inline::Strong(inner)
            | Inline::Strikethrough(inner)
            | Inline::Link { content: inner, .. } => {
                out.push_str(&plain_inlines(inner, protected, depth + 1)?)
            }
            Inline::SoftBreak => out.push(' '),
            Inline::HardBreak => out.push('\n'),
            Inline::Image { .. } => {
                return Err(MathDocumentError::Unsupported(
                    "Markdown table images require explicit scene assets",
                ));
            }
            Inline::Math(_) | Inline::DisplayMath(_) => {
                return Err(MathDocumentError::Unsupported(
                    "Markdown math in table cells is not yet supported; use a native TexMatrix",
                ));
            }
            Inline::FootnoteRef { id } => out.push_str(&format!("[^{id}]")),
        }
    }
    Ok(out)
}

/// Extra layout controls for math-enabled Markdown; dimensions are scene units.
#[derive(Debug, Clone, Copy)]
pub struct MarkdownMathOptions {
    /// Wrap prose at whitespace; no formula or word is split.
    pub line_width: Option<f64>,
    /// Body/rule color, leaving fenced-code token colors intact.
    pub color: Srgb,
}
impl Default for MarkdownMathOptions {
    fn default() -> Self {
        Self {
            line_width: None,
            color: WHITE,
        }
    }
}
impl super::Markdown<'_> {
    /// Typeset dollar and math-fenced formulas alongside native prose and code.
    /// The existing text-only `build` remains literal at dollar delimiters.
    ///
    /// # Errors
    /// Precise native typesetting, resource, input and unsupported-image errors.
    pub fn build_with_math(
        &self,
        book: &FontBook,
        engine: &TexEngine,
    ) -> Result<MarkdownMobject, MathDocumentError> {
        self.build_with_math_options(book, engine, MarkdownMathOptions::default())
    }
    /// Math-enabled document construction with a paragraph measure/body color.
    ///
    /// # Errors
    /// As [`Self::build_with_math`]. No partially constructed family is returned.
    pub fn build_with_math_options(
        &self,
        book: &FontBook,
        engine: &TexEngine,
        options: MarkdownMathOptions,
    ) -> Result<MarkdownMobject, MathDocumentError> {
        let builder = Composer {
            source: self.source,
            theme: self.theme,
            font_size: self.font_size,
            width: options.line_width,
            block_gap: self.block_gap * self.font_size / 48.0,
            color: options.color,
        };
        builder.build_inner(book, Some(engine))
    }
}
