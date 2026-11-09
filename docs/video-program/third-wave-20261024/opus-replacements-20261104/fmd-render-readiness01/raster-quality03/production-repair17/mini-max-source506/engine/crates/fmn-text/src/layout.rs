//! Text layout (§11.2): line breaking with manim's width semantics —
//! greedy by default, least-badness (the Knuth–Plass criterion over the
//! same break points) as an explicit option, never a silent default —
//! then alignment/justification/indent/line-spacing, producing positioned
//! glyphs with submobject indices matching the Reference's
//! `StringMobject` conventions (non-whitespace glyphs, in order).
//!
//! Units are ems of the base size; y-up; the first line's baseline is 0.

use crate::error::TextError;
use crate::font::{FontBook, glyph_metrics};
use crate::shape::{FaceSel, ShapedItem};
use fmn_core::color::Srgb;

/// Line-breaking algorithm.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LineBreaker {
    /// Fill each line as far as it fits — manim's semantics, the default.
    #[default]
    Greedy,
    /// Minimize the sum of squared per-line slack over the same break
    /// points (the Knuth–Plass criterion on word boundaries). An explicit
    /// option, never the silent default.
    LeastBadness,
}

/// Horizontal alignment within the measure.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Align {
    /// Left (the Reference's default).
    #[default]
    Left,
    /// Centered.
    Center,
    /// Right.
    Right,
}

/// A text-layout request.
#[derive(Clone, Debug)]
pub struct TextRequest<'a> {
    /// The source text.
    pub text: &'a str,
    /// Parse the markup tag set (`MarkupText`) instead of plain `Text`.
    pub markup: bool,
    /// Ligatures from the bundled faces (off by default — the familiar
    /// manim look).
    pub ligatures: bool,
    /// Wrap to this measure, in ems (None: only `\n` breaks lines).
    pub width: Option<f64>,
    /// Which breaker (only meaningful with a width).
    pub breaker: LineBreaker,
    /// Alignment.
    pub align: Align,
    /// Justify (stretch interword spaces to the measure; last line and
    /// single-word lines stay natural).
    pub justify: bool,
    /// First-line indent, ems.
    pub indent: f64,
    /// Line-spacing factor over the 1.2 em default baseline distance.
    pub line_spacing: f64,
    /// The `t2c`-family maps.
    pub maps: crate::maps::StyleMaps<'a>,
    /// Positional per-character overrides (fm-u8y): index-aligned with the
    /// source's `chars()`, applied after the maps. The wire the Code
    /// mobject's token spans ride — positional, so substrings inside
    /// identifiers can never bleed.
    pub overrides: crate::maps::CharOverrides<'a>,
}

impl<'a> TextRequest<'a> {
    /// A plain-text request with the Reference's defaults.
    #[must_use]
    pub fn plain(text: &'a str) -> Self {
        Self {
            text,
            markup: false,
            ligatures: false,
            width: None,
            breaker: LineBreaker::Greedy,
            align: Align::Left,
            justify: false,
            indent: 0.0,
            line_spacing: 1.0,
            maps: crate::maps::StyleMaps::default(),
            overrides: &[],
        }
    }

    /// A markup request with the Reference's defaults.
    #[must_use]
    pub fn markup(text: &'a str) -> Self {
        Self {
            markup: true,
            ..Self::plain(text)
        }
    }
}

/// The default baseline-to-baseline distance, ems (× `line_spacing`).
pub const BASELINE_SKIP: f64 = 1.2;

/// A positioned glyph.
#[derive(Clone, Debug, PartialEq)]
pub struct PlacedTextGlyph {
    /// The face it renders from.
    pub face: FaceSel,
    /// Glyph id.
    pub gid: u16,
    /// The (first) character it renders.
    pub ch: char,
    /// Left edge of the glyph origin, ems.
    pub x: f64,
    /// Baseline position, ems, y-up (line baselines descend from 0).
    pub y: f64,
    /// Size factor.
    pub size: f64,
    /// Source byte span.
    pub span: (usize, usize),
    /// First covered character index.
    pub char_index: usize,
    /// Covered character count (ligatures > 1).
    pub cluster_len: usize,
    /// Submobject ordinal — the `Text[i]` surface (non-whitespace glyphs
    /// in order, exactly the Reference's convention).
    pub submobject_index: usize,
    /// Which line the glyph sits on.
    pub line: usize,
    /// Resolved fill, if any.
    pub fill: Option<Srgb>,
}

/// An underline/strikethrough decoration rectangle.
#[derive(Clone, Debug, PartialEq)]
pub struct Decoration {
    /// Left edge, ems.
    pub x: f64,
    /// Bottom edge, ems, y-up.
    pub y: f64,
    /// Width, ems.
    pub width: f64,
    /// Thickness, ems.
    pub height: f64,
    /// Source byte span of the decorated range.
    pub span: (usize, usize),
    /// Fill, if the decorated text carried one.
    pub fill: Option<Srgb>,
}

/// One laid-out line.
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    /// Baseline y, ems (0 for the first line, descending).
    pub baseline: f64,
    /// Natural width after alignment/justification, ems.
    pub width: f64,
    /// Range into [`TextLayout::glyphs`].
    pub glyphs: (usize, usize),
}

/// The laid-out text.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextLayout {
    /// Every glyph, positioned, in reading order.
    pub glyphs: Vec<PlacedTextGlyph>,
    /// Underline/strikethrough rectangles.
    pub decorations: Vec<Decoration>,
    /// The lines.
    pub lines: Vec<Line>,
    /// Overall width, ems.
    pub width: f64,
    /// Extent above the first baseline, ems.
    pub height: f64,
    /// Extent below the first baseline, ems (positive).
    pub depth: f64,
}

impl TextLayout {
    /// Number of submobjects (the `len(Text(...))` surface).
    #[must_use]
    pub fn submobject_count(&self) -> usize {
        self.glyphs.len()
    }

    /// The glyphs of a `Text[a:b]` submobject slice.
    #[must_use]
    pub fn submobject_slice(&self, start: usize, end: usize) -> &[PlacedTextGlyph] {
        let end = end.min(self.glyphs.len());
        let start = start.min(end);
        &self.glyphs[start..end]
    }

    /// The glyphs whose source spans are contained in a byte range — the
    /// `isolate=` surface, same containment semantics as the math span
    /// map.
    #[must_use]
    pub fn select(&self, range: (usize, usize)) -> Vec<usize> {
        self.glyphs
            .iter()
            .enumerate()
            .filter(|(_, g)| g.span.0 >= range.0 && g.span.1 <= range.1)
            .map(|(i, _)| i)
            .collect()
    }
}

/// Lay text out.
///
/// # Errors
///
/// Markup diagnostics, font-policy errors, and unmapped characters — all
/// named ([`TextError`]).
pub fn layout_text(book: &FontBook, req: &TextRequest<'_>) -> Result<TextLayout, TextError> {
    layout_text_with_style(book, req, &crate::markup::CharStyle::base())
}

/// Lay out text with a base style inherited by markup and overridden by maps.
/// Source bytes, glyph ordinals and the existing shaping/layout kernel are unchanged.
///
/// # Errors
/// Propagates the same font, markup and shaping errors as [`layout_text`].
pub fn layout_text_with_style(
    book: &FontBook,
    req: &TextRequest<'_>,
    base: &crate::markup::CharStyle,
) -> Result<TextLayout, TextError> {
    let mut chars = if req.markup {
        crate::markup::parse_markup_with_style(req.text, base)?
    } else {
        crate::markup::plain_chars_with_style(req.text, base)
    };
    crate::maps::apply_maps(&mut chars, req.text, &req.maps);
    crate::maps::apply_overrides(&mut chars, req.overrides);
    let items = crate::shape::shape(book, &chars, req.ligatures)?;
    let lines = break_lines(&items, req);
    place(book, &items, &lines, req)
}

/// A line as item-index ranges plus which break ended it.
struct LineSpec {
    items: (usize, usize),
    natural_width: f64,
    space_count: usize,
    /// Justification never stretches a line ended by `\n` or end of text.
    hard_break: bool,
}

fn item_width(item: &ShapedItem) -> f64 {
    match item {
        ShapedItem::Glyph(g) => g.kern + g.advance,
        ShapedItem::Space { width, .. } => *width,
        ShapedItem::Newline { .. } => 0.0,
    }
}

/// Break into lines: paragraphs split at `\n` (always hard); within a
/// paragraph, break at spaces per the requested breaker when a measure is
/// set.
fn break_lines(items: &[ShapedItem], req: &TextRequest<'_>) -> Vec<LineSpec> {
    let mut out = Vec::new();
    let mut para_start = 0;
    for (ix, item) in items.iter().enumerate() {
        if matches!(item, ShapedItem::Newline { .. }) {
            break_paragraph(items, para_start, ix, req, &mut out);
            para_start = ix + 1;
        }
    }
    break_paragraph(items, para_start, items.len(), req, &mut out);
    out
}

fn break_paragraph(
    items: &[ShapedItem],
    start: usize,
    end: usize,
    req: &TextRequest<'_>,
    out: &mut Vec<LineSpec>,
) {
    let measure = req.width;
    let Some(measure) = measure else {
        out.push(line_spec(items, start, end, true, 0.0));
        return;
    };
    // Word extents: (first item, one-past-last item, width).
    let words = split_words(items, start, end);
    if words.is_empty() {
        out.push(line_spec(items, start, end, true, 0.0));
        return;
    }
    let breaks = match req.breaker {
        LineBreaker::Greedy => greedy_breaks(items, &words, measure, req.indent),
        LineBreaker::LeastBadness => least_badness_breaks(items, &words, measure, req.indent),
    };
    let mut word_ix = 0;
    for (bi, &break_after) in breaks.iter().enumerate() {
        let first_word = words[word_ix];
        let last_word = words[break_after];
        let is_last = bi == breaks.len() - 1;
        out.push(line_spec(
            items,
            first_word.0,
            last_word.1,
            is_last,
            if bi == 0 { req.indent } else { 0.0 },
        ));
        word_ix = break_after + 1;
    }
}

fn line_spec(
    items: &[ShapedItem],
    start: usize,
    end: usize,
    hard_break: bool,
    indent: f64,
) -> LineSpec {
    let mut width = indent;
    let mut space_count = 0;
    for item in &items[start..end] {
        width += item_width(item);
        if matches!(item, ShapedItem::Space { .. }) {
            space_count += 1;
        }
    }
    LineSpec {
        items: (start, end),
        natural_width: width,
        space_count,
        hard_break,
    }
}

/// Words within a paragraph: `(first item, one-past-last, width)`,
/// excluding surrounding spaces.
fn split_words(items: &[ShapedItem], start: usize, end: usize) -> Vec<(usize, usize, f64)> {
    let mut words = Vec::new();
    let mut ix = start;
    while ix < end {
        if matches!(items[ix], ShapedItem::Space { .. }) {
            ix += 1;
            continue;
        }
        let word_start = ix;
        let mut width = 0.0;
        while ix < end && !matches!(items[ix], ShapedItem::Space { .. }) {
            width += item_width(&items[ix]);
            ix += 1;
        }
        words.push((word_start, ix, width));
    }
    words
}

fn inter_word_space(items: &[ShapedItem], words: &[(usize, usize, f64)], a: usize) -> f64 {
    // The spaces between word a and word a+1.
    let gap = (words[a].1, words[a + 1].0);
    items[gap.0..gap.1].iter().map(item_width).sum()
}

/// Greedy: last word index of each line.
fn greedy_breaks(
    items: &[ShapedItem],
    words: &[(usize, usize, f64)],
    measure: f64,
    indent: f64,
) -> Vec<usize> {
    let mut breaks = Vec::new();
    let mut line_width = indent + words[0].2;
    for w in 1..words.len() {
        let candidate = line_width + inter_word_space(items, words, w - 1) + words[w].2;
        if candidate > measure && line_width > 0.0 {
            breaks.push(w - 1);
            line_width = words[w].2;
        } else {
            line_width = candidate;
        }
    }
    breaks.push(words.len() - 1);
    breaks
}

/// Least-badness over the same break points: minimize Σ slack² for every
/// line but the last (the Knuth–Plass criterion restricted to word
/// boundaries, computed by DP).
fn least_badness_breaks(
    items: &[ShapedItem],
    words: &[(usize, usize, f64)],
    measure: f64,
    indent: f64,
) -> Vec<usize> {
    let n = words.len();
    // width(i..=j): indent (first line only, resolved by caller position)
    // is approximated into line 0 via the DP start.
    let mut best: Vec<(f64, usize)> = vec![(f64::INFINITY, 0); n + 1];
    best[0] = (0.0, 0);
    for j in 1..=n {
        // Try a line of words i..j (1-based end).
        let mut width = 0.0;
        for i in (1..=j).rev() {
            width += words[i - 1].2;
            if i < j {
                width += inter_word_space(items, words, i - 1);
            }
            let line_width = if i == 1 { width + indent } else { width };
            if line_width > measure && i < j {
                break;
            }
            let slack = (measure - line_width).max(0.0);
            let badness = if j == n { 0.0 } else { slack * slack };
            let over_penalty = if line_width > measure {
                (line_width - measure) * 1e6
            } else {
                0.0
            };
            let total = best[i - 1].0 + badness + over_penalty;
            if total < best[j].0 {
                best[j] = (total, i - 1);
            }
        }
    }
    // Recover break positions (last word of each line, 0-based).
    let mut breaks = Vec::new();
    let mut j = n;
    while j > 0 {
        breaks.push(j - 1);
        j = best[j].1;
    }
    breaks.reverse();
    breaks
}

/// A pending underline/strike run: (strike?, start x, span, fill).
type DecoRun = (bool, f64, (usize, usize), Option<Srgb>);

/// Decoration metrics, ems.
const UNDERLINE_Y: f64 = -0.12;
const STRIKE_Y: f64 = 0.16;
const DECO_THICKNESS: f64 = 0.045;

#[allow(clippy::too_many_lines)]
fn place(
    book: &FontBook,
    items: &[ShapedItem],
    lines: &[LineSpec],
    req: &TextRequest<'_>,
) -> Result<TextLayout, TextError> {
    let measure = req.width;
    let max_natural = lines
        .iter()
        .map(|l| l.natural_width)
        .fold(0.0_f64, f64::max);
    let frame = measure.unwrap_or(max_natural);
    let mut layout = TextLayout::default();
    let mut baseline = 0.0_f64;
    let skip = BASELINE_SKIP * req.line_spacing;
    for (line_ix, spec) in lines.iter().enumerate() {
        if line_ix > 0 {
            baseline -= skip;
        }
        let extra = (frame - spec.natural_width).max(0.0);
        let (mut x, space_stretch) = if req.justify && !spec.hard_break && spec.space_count > 0 {
            (0.0, extra / spec.space_count as f64)
        } else {
            let x0 = match req.align {
                Align::Left => 0.0,
                Align::Center => extra / 2.0,
                Align::Right => extra,
            };
            (x0, 0.0)
        };
        if line_ix == 0 {
            x += req.indent;
        }
        let glyph_start = layout.glyphs.len();
        let mut deco_run: Option<DecoRun> = None;
        for item in &items[spec.items.0..spec.items.1] {
            match item {
                ShapedItem::Newline { .. } => {}
                ShapedItem::Space { width, .. } => {
                    flush_deco(&mut layout, &mut deco_run, x, baseline);
                    x += width + space_stretch;
                }
                ShapedItem::Glyph(g) => {
                    x += g.kern;
                    let y = baseline + g.baseline_shift;
                    layout.glyphs.push(PlacedTextGlyph {
                        face: g.face.clone(),
                        gid: g.gid,
                        ch: g.ch,
                        x,
                        y,
                        size: g.size,
                        span: g.span,
                        char_index: g.char_index,
                        cluster_len: g.cluster_len,
                        submobject_index: layout.glyphs.len(),
                        line: line_ix,
                        fill: g.fill,
                    });
                    let face = book.resolve_face(&g.face.family, g.face.key)?;
                    let m = glyph_metrics(face, g.gid);
                    layout.height = layout.height.max(y + m.height * g.size);
                    layout.depth = layout.depth.max(-(y - m.depth * g.size));
                    if g.underline || g.strike {
                        match &mut deco_run {
                            Some((strike, _, span, _)) if *strike == g.strike => {
                                span.1 = g.span.1;
                            }
                            _ => {
                                flush_deco(&mut layout, &mut deco_run, x, baseline);
                                deco_run = Some((g.strike, x, g.span, g.fill));
                            }
                        }
                    } else {
                        flush_deco(&mut layout, &mut deco_run, x, baseline);
                    }
                    x += g.advance;
                    layout.width = layout.width.max(x);
                }
            }
        }
        flush_deco(&mut layout, &mut deco_run, x, baseline);
        layout.lines.push(Line {
            baseline,
            width: x,
            glyphs: (glyph_start, layout.glyphs.len()),
        });
        layout.depth = layout.depth.max(-baseline);
    }
    Ok(layout)
}

fn flush_deco(layout: &mut TextLayout, run: &mut Option<DecoRun>, x_end: f64, baseline: f64) {
    if let Some((strike, x0, span, fill)) = run.take() {
        let y = baseline + if strike { STRIKE_Y } else { UNDERLINE_Y };
        layout.decorations.push(Decoration {
            x: x0,
            y,
            width: (x_end - x0).max(0.0),
            height: DECO_THICKNESS,
            span,
            fill,
        });
    }
}
