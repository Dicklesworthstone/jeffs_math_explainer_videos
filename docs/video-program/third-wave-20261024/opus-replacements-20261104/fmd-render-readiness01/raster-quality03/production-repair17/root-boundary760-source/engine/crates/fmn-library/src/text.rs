//! The Scribe bridge, text half (fm-p5d): a [`fmn_text::TextLayout`]
//! becomes a [`VMobject`] family — one child per glyph, decorations
//! trailing — with the submobject-index contract intact.
//!
//! The contract (§11.2, the Reference's `StringMobject` conventions):
//! child `i` is non-whitespace glyph `i` in reading order, so
//! `Text[a:b]` is [`TextLayout::submobject_slice`] and `isolate=` is
//! [`TextLayout::select`], and both keep working on the built family —
//! a blank glyph keeps its slot as an empty child rather than shifting
//! the ordinals. Underline/strikethrough [`Decoration`]s trail the
//! glyphs as filled rectangles, never disturbing the glyph ordinals.
//!
//! Style follows the Reference's text mobject: `stroke_width=0`,
//! `fill_opacity=1.0`, `fill_border_width=0.5`, color WHITE, with each
//! glyph's resolved fill (t2c/gradient/markup) overriding the base.
//!
//! Scale is calibrated exactly the Reference's way
//! (`text_mobject.py::get_text_mob_scale_factor`): typeset a reference
//! "0" in the book's default family and scale so its ink height is
//! `font_size / font_size_for_unit_height` manim units.

use std::sync::Arc;

use fmn_core::color::Srgb;
use fmn_core::constants::DEFAULT_MOBJECT_COLOR;
use fmn_core::types::Vec3;
use fmn_geom::QuadPath;
use fmn_mobject::Mobject;
use fmn_text::maps::CharOverrides;
use fmn_text::{
    Decoration, FontBook, LineBreaker, PlacedTextGlyph, StyleMaps, TextError, TextLayout,
    TextRequest, glyph_quadpath, layout_text,
};

pub use fmn_text::Align;
use fmn_text::markup::CharStyle;

use crate::spans::{SpanKindU8, SpanMapData, SpanMapEntry};
use crate::style::Style;
use crate::vmobject::VMobject;

/// The Reference's default `font_size=` for `Text`/`MarkupText`.
pub const DEFAULT_FONT_SIZE: f64 = 48.0;

/// The `default_config.yml` value of `text.font_size_for_unit_height`:
/// the font size at which a digit "0" stands one manim unit tall.
pub const DEFAULT_FONT_SIZE_FOR_UNIT_HEIGHT: f64 = 144.0;

/// The Reference's text-mobject style (fm-p5d): stroke off, fill on,
/// fill border 0.5, the mobject-default color (WHITE) on both lanes.
#[must_use]
pub fn text_style() -> Style {
    Style::default()
        .fill(DEFAULT_MOBJECT_COLOR, 1.0)
        .stroke(DEFAULT_MOBJECT_COLOR, 0.0, 1.0)
        .fill_border_width(0.5)
}

/// A text-bridge failure: the layouter's named errors pass through
/// untouched (markup diagnostics, font policy, unmapped characters,
/// outline decode); the bridge names its own geometry, calibration, and
/// bounded native-layout faults.
#[derive(Debug)]
pub enum TextMobjectError {
    /// The fmn-text pipeline's precise error, verbatim.
    Text(TextError),
    /// The calibration probe found no measurable "0" glyph — the book's
    /// default family maps no digit zero, so no honest scale exists.
    Calibration {
        /// The default family's name.
        family: String,
    },
    /// A positioned glyph could not be committed to a [`QuadPath`] —
    /// unreachable in practice (a subpath always starts before any
    /// segment is appended).
    Geometry {
        /// The geometry kernel's report.
        what: String,
    },
    /// A native text-backed surface refused proportional work above its
    /// declared limit.
    ResourceLimit {
        /// The surface or dimension being bounded.
        context: &'static str,
        /// Items the request would require.
        requested: usize,
        /// Maximum items admitted.
        limit: usize,
    },
    /// Native layout count arithmetic cannot be represented by the host.
    CapacityOverflow {
        /// The arithmetic operation being bounded.
        context: &'static str,
    },
    /// Reserving a validated bounded native-layout buffer failed.
    AllocationFailed {
        /// The buffer being reserved.
        context: &'static str,
        /// Validated capacity requested.
        requested: usize,
    },
}

impl core::fmt::Display for TextMobjectError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Text(e) => e.fmt(f),
            Self::Calibration { family } => write!(
                f,
                "the default family {family:?} maps no measurable \"0\" glyph; \
                 text scale calibration is impossible"
            ),
            Self::Geometry { what } => write!(f, "glyph path commit failed: {what}"),
            Self::ResourceLimit {
                context,
                requested,
                limit,
            } => write!(
                f,
                "{context} requires {requested} items, above the declared limit of {limit}"
            ),
            Self::CapacityOverflow { context } => {
                write!(f, "{context} exceeds the host's representable capacity")
            }
            Self::AllocationFailed { context, requested } => {
                write!(f, "{context} could not reserve {requested} validated items")
            }
        }
    }
}

impl std::error::Error for TextMobjectError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Text(e) => Some(e),
            _ => None,
        }
    }
}

impl From<TextError> for TextMobjectError {
    fn from(e: TextError) -> Self {
        Self::Text(e)
    }
}

/// A built text mobject: the [`VMobject`] family plus the layout that
/// produced it — the span map the `Text[a:b]` / `isolate=` / t2x
/// surfaces consume (§11.3's native provenance; the Reference's
/// render-twice-and-align hack is dead).
#[derive(Debug, Clone)]
pub struct TextMobject {
    /// The family: child `i` is glyph `i` of the layout (non-whitespace
    /// glyphs in reading order), decorations trailing.
    pub vmob: VMobject,
    /// The layout: submobject indices and source spans, intact.
    pub layout: TextLayout,
    /// The source string, verbatim — the byte-range anchor of the span
    /// map ([`TextMobject::span_map`]).
    pub source: Arc<str>,
}

impl TextMobject {
    /// The `Text[a:b]` surface: the glyphs of a submobject slice.
    #[must_use]
    pub fn submobject_slice(&self, start: usize, end: usize) -> &[PlacedTextGlyph] {
        self.layout.submobject_slice(start, end)
    }

    /// The `isolate=` surface: the child ordinals whose source spans are
    /// contained in `range` (containment semantics, exactly the math
    /// span map's).
    #[must_use]
    pub fn select(&self, range: (usize, usize)) -> Vec<usize> {
        self.layout.select(range)
    }

    /// `len(Text(...))`: the number of glyph children (decorations
    /// trail and are not submobjects of the text itself).
    #[must_use]
    pub fn len(&self) -> usize {
        self.layout.submobject_count()
    }

    /// True when the text produced no glyphs.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The native span map: one entry per glyph, in child order — entry
    /// `i` is glyph child `i`'s source byte range. Decorations trail the
    /// glyphs as children but carry no construct kind of their own, so
    /// they are not span entries; the glyph child ordinals are exactly
    /// the entries' ordinals. This is the data the composition root binds
    /// into the Studio inspector's `SpanRegistry` (§11.3's native
    /// provenance).
    #[must_use]
    pub fn span_map(&self) -> SpanMapData {
        let entries = self
            .layout
            .glyphs
            .iter()
            .map(|glyph| SpanMapEntry {
                start: glyph.span.0,
                end: glyph.span.1,
                kind: SpanKindU8::TextGlyph,
            })
            .collect();
        SpanMapData {
            source: Arc::clone(&self.source),
            entries,
        }
    }
}

/// Completing construction as the Reference's `SVGMobject` does
/// (`move_into_position` with `should_center`): a `Text`/`MarkupText` added
/// to a stage stands centred on the origin. [`TextMobject::vmob`] itself
/// stays in layout coordinates (the first baseline at `y = 0`) for builders
/// that compose laid-out pieces.
impl From<TextMobject> for Mobject {
    fn from(t: TextMobject) -> Self {
        t.vmob.moved_to([0.0; 3]).into()
    }
}

/// `Text` (Appendix A `mobject/svg/text_mobject`): the plain-text
/// builder over [`TextRequest`]. `MarkupText` (the same module's
/// sibling) is [`Text::markup`].
///
/// Builder setters are by-value, the §15.1 surface G0-1 ratified; the
/// t2x map slices are borrowed by the caller, exactly as
/// [`StyleMaps`] borrows them.
#[derive(Debug, Clone)]
pub struct Text<'a> {
    text: &'a str,
    markup: bool,
    ligatures: bool,
    width: Option<f64>,
    breaker: LineBreaker,
    align: Align,
    justify: bool,
    indent: f64,
    line_spacing: f64,
    maps: StyleMaps<'a>,
    overrides: CharOverrides<'a>,
    base_style: CharStyle,
    font_size: f64,
    font_size_for_unit_height: f64,
    style: Style,
}

impl<'a> Text<'a> {
    /// A plain `Text` with the Reference's defaults: ligatures off,
    /// greedy breaking, left alignment, no wrap measure, line spacing
    /// 1.0, font size 48 over 144-per-unit, the text style.
    #[must_use]
    pub fn new(text: &'a str) -> Self {
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
            maps: StyleMaps::default(),
            overrides: &[],
            base_style: CharStyle::base(),
            font_size: DEFAULT_FONT_SIZE,
            font_size_for_unit_height: DEFAULT_FONT_SIZE_FOR_UNIT_HEIGHT,
            style: text_style(),
        }
    }

    /// `MarkupText`: the same builder parsing the manim markup tag set.
    #[must_use]
    pub fn markup(text: &'a str) -> Self {
        Self {
            markup: true,
            ..Self::new(text)
        }
    }

    /// Inherited font family; markup and substring maps may override it.
    #[must_use]
    pub fn font(mut self, family: &str) -> Self {
        self.base_style.family = if family.is_empty() {
            None
        } else {
            Some(family.to_owned())
        };
        self
    }

    /// Inherited bold face selection, retaining inner markup overrides.
    #[must_use]
    pub fn bold(mut self, bold: bool) -> Self {
        self.base_style.bold = bold;
        self
    }

    /// Inherited italic face selection, retaining inner markup overrides.
    #[must_use]
    pub fn italic(mut self, italic: bool) -> Self {
        self.base_style.italic = italic;
        self
    }

    /// The `font_size=` surface.
    #[must_use]
    pub fn font_size(mut self, font_size: f64) -> Self {
        self.font_size = font_size;
        self
    }

    /// The config's `text.font_size_for_unit_height` — the font size at
    /// which "0" stands one manim unit tall.
    #[must_use]
    pub fn font_size_for_unit_height(mut self, fsuh: f64) -> Self {
        self.font_size_for_unit_height = fsuh;
        self
    }

    /// Bundled-face ligature sets (`disable_ligatures=False` in the
    /// Reference; off here by default, matching the familiar look).
    #[must_use]
    pub fn ligatures(mut self, on: bool) -> Self {
        self.ligatures = on;
        self
    }

    /// The wrap measure (`line_width=`), in ems.
    #[must_use]
    pub fn width(mut self, width: f64) -> Self {
        self.width = Some(width);
        self
    }

    /// Which line breaker (greedy is manim's; least-badness an explicit
    /// option).
    #[must_use]
    pub fn breaker(mut self, breaker: LineBreaker) -> Self {
        self.breaker = breaker;
        self
    }

    /// Horizontal alignment within the measure.
    #[must_use]
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// Justify interword spaces to the measure.
    #[must_use]
    pub fn justify(mut self, on: bool) -> Self {
        self.justify = on;
        self
    }

    /// First-line indent, ems.
    #[must_use]
    pub fn indent(mut self, indent: f64) -> Self {
        self.indent = indent;
        self
    }

    /// Line-spacing factor over the 1.2 em default baseline distance.
    #[must_use]
    pub fn line_spacing(mut self, factor: f64) -> Self {
        self.line_spacing = factor;
        self
    }

    /// The whole t2x map set at once.
    #[must_use]
    pub fn maps(mut self, maps: StyleMaps<'a>) -> Self {
        self.maps = maps;
        self
    }

    /// text→color (`t2c`).
    #[must_use]
    pub fn t2c(mut self, t2c: &'a [(&'a str, Srgb)]) -> Self {
        self.maps.t2c = t2c;
        self
    }

    /// Positional per-character overrides (fm-u8y) — the Code channel:
    /// index-aligned with the source's `chars()`, applied after every map,
    /// so a keyword inside an identifier can never bleed.
    #[must_use]
    pub fn char_overrides(mut self, overrides: CharOverrides<'a>) -> Self {
        self.overrides = overrides;
        self
    }

    /// text→font family (`t2f`); a family the book lacks is the named
    /// [`TextError::FontUnavailable`], never a silent substitution.
    #[must_use]
    pub fn t2f(mut self, t2f: &'a [(&'a str, &'a str)]) -> Self {
        self.maps.t2f = t2f;
        self
    }

    /// text→gradient stops (`t2g`), sampled per glyph at shaping.
    #[must_use]
    pub fn t2g(mut self, t2g: &'a [(&'a str, &'a [Srgb])]) -> Self {
        self.maps.t2g = t2g;
        self
    }

    /// text→slant (`t2s`).
    #[must_use]
    pub fn t2s(mut self, t2s: &'a [(&'a str, bool)]) -> Self {
        self.maps.t2s = t2s;
        self
    }

    /// text→weight (`t2w`).
    #[must_use]
    pub fn t2w(mut self, t2w: &'a [(&'a str, bool)]) -> Self {
        self.maps.t2w = t2w;
        self
    }

    /// Replace the base style (glyph fills still override per glyph).
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Lay out and build the family.
    ///
    /// # Errors
    ///
    /// [`TextMobjectError::Text`]: the layouter's named errors —
    /// markup diagnostics, font policy, unmapped characters — surface
    /// at construction time, never as a blank render.
    /// [`TextMobjectError::Calibration`]: the default family maps no
    /// measurable "0".
    pub fn build(&self, book: &FontBook) -> Result<TextMobject, TextMobjectError> {
        let req = TextRequest {
            text: self.text,
            markup: self.markup,
            ligatures: self.ligatures,
            width: self.width,
            breaker: self.breaker,
            align: self.align,
            justify: self.justify,
            indent: self.indent,
            line_spacing: self.line_spacing,
            maps: self.maps.clone(),
            overrides: self.overrides,
        };
        if let Some(family) = &self.base_style.family {
            book.family(family)?;
        }
        let layout = fmn_text::layout::layout_text_with_style(book, &req, &self.base_style)?;
        let scale = calibrate(book, self.font_size, self.font_size_for_unit_height)?;
        let mut children = Vec::with_capacity(layout.glyphs.len() + layout.decorations.len());
        for glyph in &layout.glyphs {
            children.push(self.glyph_child(book, glyph, scale)?);
        }
        for decoration in &layout.decorations {
            children.push(decoration_child(decoration, scale, self.style)?);
        }
        let vmob = VMobject::new()
            .with_style(self.style)
            .with_children(children);
        Ok(TextMobject {
            vmob,
            layout,
            source: Arc::from(self.text),
        })
    }

    /// One glyph child: the positioned outline scaled to scene units,
    /// styled with the base style and the glyph's own resolved fill.
    /// A blank glyph keeps its submobject slot as an empty child —
    /// ordinals never shift.
    fn glyph_child(
        &self,
        book: &FontBook,
        glyph: &PlacedTextGlyph,
        scale: f64,
    ) -> Result<VMobject, TextMobjectError> {
        let path = glyph_quadpath(book, glyph)?;
        let mut style = self.style;
        if let Some(fill) = glyph.fill {
            style.fill_color = fill;
        }
        if !path.has_points() {
            return Ok(VMobject::new().with_style(style));
        }
        let points: Vec<Vec3> = path
            .points()
            .iter()
            .map(|p| [p[0] * scale, p[1] * scale, p[2]])
            .collect();
        Ok(VMobject::from_points(points).with_style(style))
    }
}

/// `MarkupText` (Appendix A `mobject/svg/text_mobject`): the
/// markup-parsing sibling of [`Text`], with the same builder surface.
#[derive(Debug, Clone)]
pub struct MarkupText<'a> {
    inner: Text<'a>,
}

impl<'a> MarkupText<'a> {
    /// A markup text with the Reference's defaults.
    #[must_use]
    pub fn new(text: &'a str) -> Self {
        Self {
            inner: Text::markup(text),
        }
    }

    /// The `font_size=` surface.
    #[must_use]
    pub fn font_size(mut self, font_size: f64) -> Self {
        self.inner = self.inner.font_size(font_size);
        self
    }

    /// The config's `text.font_size_for_unit_height`.
    #[must_use]
    pub fn font_size_for_unit_height(mut self, fsuh: f64) -> Self {
        self.inner = self.inner.font_size_for_unit_height(fsuh);
        self
    }

    /// Bundled-face ligature sets (off by default).
    #[must_use]
    pub fn ligatures(mut self, on: bool) -> Self {
        self.inner = self.inner.ligatures(on);
        self
    }

    /// The wrap measure (`line_width=`), in ems.
    #[must_use]
    pub fn width(mut self, width: f64) -> Self {
        self.inner = self.inner.width(width);
        self
    }

    /// Which line breaker.
    #[must_use]
    pub fn breaker(mut self, breaker: LineBreaker) -> Self {
        self.inner = self.inner.breaker(breaker);
        self
    }

    /// Horizontal alignment within the measure.
    #[must_use]
    pub fn align(mut self, align: Align) -> Self {
        self.inner = self.inner.align(align);
        self
    }

    /// Justify interword spaces to the measure.
    #[must_use]
    pub fn justify(mut self, justify: bool) -> Self {
        self.inner = self.inner.justify(justify);
        self
    }

    /// First-line indent, ems.
    #[must_use]
    pub fn indent(mut self, indent: f64) -> Self {
        self.inner = self.inner.indent(indent);
        self
    }

    /// Line-spacing factor.
    #[must_use]
    pub fn line_spacing(mut self, factor: f64) -> Self {
        self.inner = self.inner.line_spacing(factor);
        self
    }

    /// The whole t2x map set at once.
    #[must_use]
    pub fn maps(mut self, maps: StyleMaps<'a>) -> Self {
        self.inner = self.inner.maps(maps);
        self
    }

    /// text→color (`t2c`).
    #[must_use]
    pub fn t2c(mut self, t2c: &'a [(&'a str, Srgb)]) -> Self {
        self.inner = self.inner.t2c(t2c);
        self
    }

    /// text→font family (`t2f`).
    #[must_use]
    pub fn t2f(mut self, t2f: &'a [(&'a str, &'a str)]) -> Self {
        self.inner = self.inner.t2f(t2f);
        self
    }

    /// text→gradient stops (`t2g`).
    #[must_use]
    pub fn t2g(mut self, t2g: &'a [(&'a str, &'a [Srgb])]) -> Self {
        self.inner = self.inner.t2g(t2g);
        self
    }

    /// text→slant (`t2s`).
    #[must_use]
    pub fn t2s(mut self, t2s: &'a [(&'a str, bool)]) -> Self {
        self.inner = self.inner.t2s(t2s);
        self
    }

    /// text→weight (`t2w`).
    #[must_use]
    pub fn t2w(mut self, t2w: &'a [(&'a str, bool)]) -> Self {
        self.inner = self.inner.t2w(t2w);
        self
    }

    /// Replace the base style.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.inner = self.inner.style(style);
        self
    }

    /// Lay out and build the family.
    ///
    /// # Errors
    ///
    /// As [`Text::build`].
    pub fn build(&self, book: &FontBook) -> Result<TextMobject, TextMobjectError> {
        self.inner.build(book)
    }
}

/// The ems→scene-units scale, calibrated the Reference's way: typeset a
/// reference "0" in the book's default family and scale so its ink
/// height is `font_size / font_size_for_unit_height` manim units. The
/// calibration is face-independent by design (the Reference calibrates
/// on the default face whatever the text's own font); per-glyph metric
/// differences are BN-05's territory.
pub(crate) fn calibrate(
    book: &FontBook,
    font_size: f64,
    fsuh: f64,
) -> Result<f64, TextMobjectError> {
    let probe = layout_text(book, &TextRequest::plain("0"))?;
    let calibration = || TextMobjectError::Calibration {
        family: book.default_family().name.clone(),
    };
    let glyph = probe.glyphs.first().ok_or_else(calibration)?;
    let path = glyph_quadpath(book, glyph)?;
    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    for p in path.points() {
        lo = lo.min(p[1]);
        hi = hi.max(p[1]);
    }
    let height = hi - lo;
    if !height.is_finite() || height <= 0.0 {
        return Err(calibration());
    }
    Ok(font_size / (fsuh * height))
}

/// A decoration child: the underline/strikethrough rectangle as a
/// filled, unstroked quad path, scaled to scene units.
fn decoration_child(d: &Decoration, scale: f64, base: Style) -> Result<VMobject, TextMobjectError> {
    let v = |x: f64, y: f64| -> Vec3 { [x * scale, y * scale, 0.0] };
    let mut path = QuadPath::new();
    path.start_new_path(v(d.x, d.y));
    let geometry = |e: fmn_geom::GeomError| TextMobjectError::Geometry {
        what: format!("{e:?}"),
    };
    path.add_line_to(v(d.x + d.width, d.y), true)
        .map_err(geometry)?;
    path.add_line_to(v(d.x + d.width, d.y + d.height), true)
        .map_err(geometry)?;
    path.add_line_to(v(d.x, d.y + d.height), true)
        .map_err(geometry)?;
    path.add_line_to(v(d.x, d.y), true).map_err(geometry)?;
    let mut style = base;
    style.stroke_width = 0.0;
    if let Some(fill) = d.fill {
        style.fill_color = fill;
    }
    Ok(VMobject::from_path(&path).with_style(style))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmn_core::constants::RED;
    use fmn_mobject::Stage;

    fn book() -> FontBook {
        FontBook::bundled().expect("bundled faces parse")
    }

    /// The deterministic golden format: one line per point of the
    /// family's whole point runs, fixed six decimals (the convention of
    /// fmd-math's `canonical_dump`).
    fn dump_family(vmob: &VMobject) -> String {
        let mut out = String::new();
        dump_one(vmob, &mut out);
        for child in vmob.children() {
            dump_one(child, &mut out);
        }
        out
    }

    fn dump_one(vmob: &VMobject, out: &mut String) {
        for p in vmob.points() {
            out.push_str(&format!("{:.6} {:.6} {:.6}\n", p[0], p[1], p[2]));
        }
    }

    #[test]
    fn text_children_are_the_glyphs_in_reading_order() {
        let m = Text::new("ab cd").build(&book()).expect("builds");
        // Spaces produce no submobject, exactly the Reference's
        // StringMobject convention: 4 glyphs, 4 children.
        assert_eq!(m.len(), 4);
        assert_eq!(m.vmob.children().len(), 4);
        let chars: Vec<char> = m.layout.glyphs.iter().map(|g| g.ch).collect();
        assert_eq!(chars, ['a', 'b', 'c', 'd']);
        // The text style, on every child: stroke 0, fill 1, border 0.5.
        for child in m.vmob.children() {
            let s = child.style();
            assert_eq!(s.stroke_width, 0.0);
            assert_eq!(s.fill_opacity, 1.0);
            assert_eq!(s.fill_border_width, 0.5);
            assert_eq!(s.fill_color, DEFAULT_MOBJECT_COLOR);
        }
    }

    #[test]
    fn text_a_b_slice_is_the_reference_convention() {
        let m = Text::new("ab cd").build(&book()).expect("builds");
        let slice = m.submobject_slice(1, 3);
        let chars: Vec<char> = slice.iter().map(|g| g.ch).collect();
        assert_eq!(chars, ['b', 'c']);
        // Clamped, never panicking.
        assert_eq!(m.submobject_slice(3, 100).len(), 1);
        assert_eq!(m.submobject_slice(9, 2).len(), 0);
    }

    #[test]
    fn isolate_selects_exactly_the_word() {
        let m = Text::new("hello world").build(&book()).expect("builds");
        let selected = m.select((6, 11));
        assert_eq!(selected, vec![5, 6, 7, 8, 9]);
        for (ord, ch) in selected.iter().zip("world".chars()) {
            assert_eq!(m.layout.glyphs[*ord].ch, ch);
            // The ordinal names the VMobject child.
            assert!(!m.vmob.children()[*ord].points().is_empty());
        }
        // Containment semantics: a range covering only part of a word
        // selects the glyphs fully inside it.
        assert_eq!(m.select((0, 2)), vec![0, 1]);
    }

    #[test]
    fn t2c_colors_exactly_the_mapped_occurrence() {
        let t2c = [("o", RED)];
        let m = Text::new("foo boo")
            .t2c(&t2c)
            .build(&book())
            .expect("builds");
        for (i, child) in m.vmob.children().iter().enumerate() {
            let ch = m.layout.glyphs[i].ch;
            let want = if ch == 'o' {
                RED
            } else {
                DEFAULT_MOBJECT_COLOR
            };
            assert_eq!(child.style().fill_color, want, "glyph {i} ({ch:?})");
        }
    }

    #[test]
    fn markup_color_and_weight_land_on_the_glyphs() {
        let m = MarkupText::new("<span foreground=\"#FF0000\">hi</span><b>!</b>")
            .build(&book())
            .expect("builds");
        assert_eq!(m.len(), 3);
        let red = Srgb::from_hex("#FF0000").expect("hex");
        assert_eq!(m.vmob.children()[0].style().fill_color, red);
        assert_eq!(m.vmob.children()[1].style().fill_color, red);
        assert_eq!(
            m.vmob.children()[2].style().fill_color,
            DEFAULT_MOBJECT_COLOR
        );
        // <b> resolved to the bold face at shaping time.
        assert!(m.layout.glyphs[2].face.key.bold);
        assert!(!m.layout.glyphs[0].face.key.bold);
    }

    #[test]
    fn underline_decorations_trail_the_glyphs() {
        let m = MarkupText::new("<u>ab</u>").build(&book()).expect("builds");
        assert_eq!(m.len(), 2);
        // Two glyph children plus one trailing decoration rectangle.
        assert_eq!(m.vmob.children().len(), 3);
        let deco = &m.vmob.children()[2];
        assert!(!deco.points().is_empty());
        let path = deco.path().expect("a valid path");
        assert!(path.is_closed(), "a decoration is a closed rectangle");
        let (_, ymax) = deco.extent().expect("has extent");
        // Underline sits just below the baseline (scaled): y < 0.
        assert!(ymax[1] < 0.0, "underline below baseline, got {ymax:?}");
        assert_eq!(deco.style().stroke_width, 0.0);
    }

    #[test]
    fn the_calibration_makes_a_digit_font_size_over_fsuh_tall() {
        let book = book();
        for font_size in [48.0, 96.0, 12.5] {
            let m = Text::new("0")
                .font_size(font_size)
                .build(&book)
                .expect("builds");
            let (min, max) = m.vmob.children()[0].extent().expect("has extent");
            let height = max[1] - min[1];
            let want = font_size / DEFAULT_FONT_SIZE_FOR_UNIT_HEIGHT;
            assert!(
                (height - want).abs() < 1e-9,
                "font_size {font_size}: \"0\" height {height} != {want}"
            );
        }
    }

    #[test]
    fn an_unmapped_char_is_the_named_error() {
        let err = Text::new("a 🦀 b").build(&book()).expect_err("fails");
        assert!(
            matches!(&err, TextMobjectError::Text(TextError::UnmappedChar { ch, .. }) if *ch == '🦀'),
            "expected UnmappedChar('🦀'), got {err:?}"
        );
    }

    #[test]
    fn bad_markup_is_a_positioned_diagnostic() {
        let err = MarkupText::new("<b>oops")
            .build(&book())
            .expect_err("fails");
        assert!(
            matches!(&err, TextMobjectError::Text(TextError::Markup { line, .. }) if *line == 1),
            "expected a line-1 markup diagnostic, got {err:?}"
        );
    }

    #[test]
    fn t2f_names_a_missing_family_precisely() {
        let t2f = [("hi", "No Such Family")];
        let err = Text::new("hi").t2f(&t2f).build(&book()).expect_err("fails");
        assert!(
            matches!(&err, TextMobjectError::Text(TextError::FontUnavailable { family, available })
                if family.as_str() == "No Such Family" && !available.is_empty()),
            "expected FontUnavailable naming the family, got {err:?}"
        );
    }

    #[test]
    fn the_family_enters_the_arena_as_a_family() {
        let mut stage = Stage::new();
        let m = Text::new("hi").build(&book()).expect("builds");
        let mob = stage.add(m.vmob);
        assert_eq!(stage.family(mob).len(), 3, "parent + two glyph children");
    }

    /// Golden maintenance: `cargo test -p fmn-library -- --ignored
    /// regenerate_text_goldens` rewrites the two golden files after a
    /// deliberate fmn-text / fmd-font pin move. The regenerated files
    /// are review material — never regenerate casually.
    #[test]
    #[ignore]
    fn regenerate_text_goldens() {
        let book = book();
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/goldens");
        std::fs::create_dir_all(dir).expect("goldens dir");
        let zero = Text::new("0").build(&book).expect("builds");
        std::fs::write(format!("{dir}/text_zero.txt"), dump_family(&zero.vmob))
            .expect("write text_zero");
        let bold = MarkupText::new("<b>0</b>").build(&book).expect("builds");
        std::fs::write(
            format!("{dir}/markup_bold_zero.txt"),
            dump_family(&bold.vmob),
        )
        .expect("write markup_bold_zero");
    }

    #[test]
    fn text_zero_golden() {
        let m = Text::new("0").build(&book()).expect("builds");
        assert_eq!(m.vmob.children().len(), 1);
        let dump = dump_family(&m.vmob);
        let expected = include_str!("../tests/goldens/text_zero.txt");
        assert_eq!(
            dump, expected,
            "golden drift (see tests/goldens/text_zero.txt)"
        );
    }

    #[test]
    fn markup_bold_zero_golden() {
        let m = MarkupText::new("<b>0</b>").build(&book()).expect("builds");
        assert_eq!(m.vmob.children().len(), 1);
        let dump = dump_family(&m.vmob);
        let expected = include_str!("../tests/goldens/markup_bold_zero.txt");
        assert_eq!(
            dump, expected,
            "golden drift (see tests/goldens/markup_bold_zero.txt)"
        );
    }
}
