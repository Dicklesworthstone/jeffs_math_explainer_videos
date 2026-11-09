//! The Scribe bridge, math half (fm-p5d): a [`fmn_tex::Typeset`]
//! becomes a [`VMobject`] family — **one child per `Sub`** — with the
//! span map intact.
//!
//! The contract (§11.3–11.5): child `i` is `typeset.subs[i]`, so the
//! ordinals [`Typeset::occurrences`] returns are the family's child
//! indices — `isolate=`, `tex_to_color_map`, and
//! `TransformMatchingTex` consume them by source identity, and the
//! Reference's render-twice-and-align hack stays dead. Resolving a
//! whole layout through `fmd_math::paths::resolve_paths` would flatten
//! the per-`Sub` grouping, so each primitive resolves on its own via
//! [`TexEngine::resolve_prim`]: glyphs through the engine's pinned
//! size/upm transform, rules as rectangles, drawn paths (extensible
//! delimiters, radicals, stretchy bands) positioned already.
//!
//! Style follows the Reference's tex mobject — the same defaults as
//! text ([`text_style`]): `stroke_width=0`, `fill_opacity=1.0`,
//! `fill_border_width=0.5`, color WHITE. Scale is calibrated the
//! Reference's way (`tex_mobject.py::get_tex_mob_scale_factor`):
//! typeset a reference "0" and scale so its height is
//! `font_size / font_size_for_unit_height` manim units.

use std::sync::Arc;

use fmn_core::color::Srgb;
use fmn_core::types::Vec3;
use fmn_geom::QuadPath;
use fmn_mobject::Mobject;
pub use fmn_tex::LineAlign;
use fmn_tex::{Mode, PathContour, PathSeg, Prim, Style as MathStyle, TexEngine, TexError, Typeset};

use crate::spans::{SpanKindU8, SpanMapData, SpanMapEntry};
use crate::style::Style;
use crate::text::{DEFAULT_FONT_SIZE, DEFAULT_FONT_SIZE_FOR_UNIT_HEIGHT, text_style};
use crate::vmobject::VMobject;

/// A tex-bridge failure: fmd-math's precise, tier-tagged construct
/// errors pass through untouched (never a blank render); the bridge
/// itself only names its own two faults.
#[derive(Debug)]
pub enum TexMobjectError {
    /// The fmn-tex pipeline's precise error, verbatim.
    Tex(TexError),
    /// The calibration probe found no measurable "0" — the math face
    /// roster maps no digit zero (build corruption).
    Calibration,
    /// A contour could not be committed to a [`QuadPath`] — unreachable
    /// in practice (a subpath always starts before any segment is
    /// appended).
    Geometry {
        /// The geometry kernel's report.
        what: String,
    },
}

impl core::fmt::Display for TexMobjectError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Tex(e) => e.fmt(f),
            Self::Calibration => write!(
                f,
                "the math face roster maps no measurable \"0\" glyph; \
                 tex scale calibration is impossible"
            ),
            Self::Geometry { what } => write!(f, "math contour commit failed: {what}"),
        }
    }
}

impl std::error::Error for TexMobjectError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Tex(e) => Some(e),
            _ => None,
        }
    }
}

impl From<TexError> for TexMobjectError {
    fn from(e: TexError) -> Self {
        Self::Tex(e)
    }
}

/// A built tex mobject: the [`VMobject`] family plus the typeset that
/// produced it — the span map `isolate=` / `tex_to_color_map` /
/// `TransformMatchingTex` consume (§11.3).
#[derive(Debug, Clone)]
pub struct TexMobject {
    /// The family: child `i` is `typeset.subs[i]` — one child per glyph,
    /// per rule, per drawn path, in emission order.
    pub vmob: VMobject,
    /// The typeset: the source, the layout, and the submobject table,
    /// intact.
    pub typeset: Typeset,
}

impl TexMobject {
    /// The `isolate=` surface: the child ordinals selected by each
    /// occurrence of `needle` in the source, by source identity.
    #[must_use]
    pub fn occurrences(&self, needle: &str) -> Vec<Vec<usize>> {
        self.typeset.occurrences(needle)
    }

    /// The number of submobjects (`len(Tex(...))`).
    #[must_use]
    pub fn len(&self) -> usize {
        self.typeset.subs.len()
    }

    /// True when the typeset produced no primitives.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The native span map: one entry per `Sub`, in child order — entry
    /// `i` is child ordinal `i`'s source byte range and construct kind.
    /// This is the data the composition root binds into the Studio
    /// inspector's `SpanRegistry` (§11.4).
    #[must_use]
    pub fn span_map(&self) -> SpanMapData {
        let entries = self
            .typeset
            .subs
            .iter()
            .map(|sub| SpanMapEntry {
                start: sub.span.start,
                end: sub.span.end,
                kind: match sub.prim {
                    Prim::Glyph(_) => SpanKindU8::MathGlyph,
                    Prim::Rule(_) => SpanKindU8::MathRule,
                    Prim::Path(_) => SpanKindU8::MathPath,
                },
            })
            .collect();
        SpanMapData {
            source: Arc::from(self.typeset.source.as_str()),
            entries,
        }
    }
}

/// Completing construction as the Reference's `SVGMobject` does
/// (`move_into_position` with `should_center`): a `Tex`/`TexText` added to a
/// stage stands centred on the origin. [`TexMobject::vmob`] itself stays in
/// layout coordinates for builders that compose typeset pieces.
impl From<TexMobject> for Mobject {
    fn from(t: TexMobject) -> Self {
        t.vmob.moved_to([0.0; 3]).into()
    }
}

/// `Tex` (Appendix A `mobject/svg/tex_mobject`): the math builder over
/// [`TexEngine`]. `TexText` (the same module's sibling) is the
/// text-mainland mode.
#[derive(Debug, Clone)]
pub struct Tex<'a> {
    source: &'a str,
    preamble: &'a str,
    mode: Mode,
    font_size: f64,
    font_size_for_unit_height: f64,
    style: Style,
    t2c: &'a [(&'a str, Srgb)],
    align: LineAlign,
}

impl<'a> Tex<'a> {
    /// A `Tex` with the Reference's defaults: display-style mathematics
    /// (the Reference wraps every `Tex` in `\begin{align*}`, which is display
    /// math), font size 48 over 144-per-unit, the tex style, no color map.
    /// Inline (text-style) mathematics is `.math_style(MathStyle::Text)`.
    #[must_use]
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            preamble: "",
            mode: Mode::Math(MathStyle::Display),
            font_size: DEFAULT_FONT_SIZE,
            font_size_for_unit_height: DEFAULT_FONT_SIZE_FOR_UNIT_HEIGHT,
            style: text_style(),
            t2c: &[],
            align: LineAlign::Left,
        }
    }

    /// Display-style mathematics (`\displaystyle`).
    #[must_use]
    pub fn display(mut self) -> Self {
        self.mode = Mode::Math(MathStyle::Display);
        self
    }

    /// The outer math style explicitly.
    #[must_use]
    pub fn math_style(mut self, style: MathStyle) -> Self {
        self.mode = Mode::Math(style);
        self
    }

    /// The `font_size=` surface.
    #[must_use]
    pub fn font_size(mut self, font_size: f64) -> Self {
        self.font_size = font_size;
        self
    }

    /// Native macro declarations applied only to this formula. Spans continue
    /// to address the original source, never the prepended definitions.
    #[must_use]
    pub fn preamble(mut self, preamble: &'a str) -> Self {
        self.preamble = preamble;
        self
    }

    /// The config's `tex.font_size_for_unit_height` — the font size at
    /// which "0" stands one manim unit tall.
    #[must_use]
    pub fn font_size_for_unit_height(mut self, fsuh: f64) -> Self {
        self.font_size_for_unit_height = fsuh;
        self
    }

    /// Replace the base style.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// `tex_to_color_map` (`t2c`), applied by source identity through
    /// the span map; later entries win within the map.
    #[must_use]
    pub fn t2c(mut self, t2c: &'a [(&'a str, Srgb)]) -> Self {
        self.t2c = t2c;
        self
    }

    /// Typeset and build the family.
    ///
    /// # Errors
    ///
    /// [`TexMobjectError::Tex`]: an unsupported construct is fmd-math's
    /// precise, named, tier-tagged error at construction time — never
    /// silence, never garbage. [`TexMobjectError::Calibration`]: the
    /// math face roster maps no measurable "0".
    pub fn build(&self, engine: &TexEngine) -> Result<TexMobject, TexMobjectError> {
        let typeset = engine.typeset_aligned(self.mode, self.source, self.preamble, self.align)?;
        let scale = calibrate(engine, self.font_size, self.font_size_for_unit_height)?;
        // tex_to_color_map, resolved to child ordinals before
        // construction: later entries win (the Reference's dict-update).
        let mut fills: Vec<Option<Srgb>> = vec![None; typeset.subs.len()];
        for (needle, color) in self.t2c {
            for occurrence in typeset.occurrences(needle) {
                for ord in occurrence {
                    fills[ord] = Some(*color);
                }
            }
        }
        let mut children = Vec::with_capacity(typeset.subs.len());
        for (ord, sub) in typeset.subs.iter().enumerate() {
            let mut style = self.style;
            if let Some(fill) = fills[ord] {
                style.fill_color = fill;
            }
            children.push(prim_child(engine, &typeset, sub, style, scale)?);
        }
        let vmob = VMobject::new()
            .with_style(self.style)
            .with_children(children);
        Ok(TexMobject { vmob, typeset })
    }
}

/// `TexText` (Appendix A `mobject/svg/tex_mobject`): the text-mainland
/// sibling of [`Tex`] — prose with `$…$` math islands.
#[derive(Debug, Clone)]
pub struct TexText<'a> {
    inner: Tex<'a>,
}

impl<'a> TexText<'a> {
    /// A `TexText` with the Reference's defaults, including its
    /// `alignment="\centering"`: `\\`-split lines center.
    #[must_use]
    pub fn new(source: &'a str) -> Self {
        Self {
            inner: Tex {
                mode: Mode::Text,
                align: LineAlign::Center,
                ..Tex::new(source)
            },
        }
    }

    /// The `font_size=` surface.
    #[must_use]
    pub fn font_size(mut self, font_size: f64) -> Self {
        self.inner = self.inner.font_size(font_size);
        self
    }

    /// Native macro declarations for this text-mainland request.
    #[must_use]
    pub fn preamble(mut self, preamble: &'a str) -> Self {
        self.inner = self.inner.preamble(preamble);
        self
    }

    /// The `alignment=` surface: how the `\\`-split lines align. The
    /// default is the Reference's `\centering` ([`LineAlign::Center`]).
    #[must_use]
    pub fn line_align(mut self, align: LineAlign) -> Self {
        self.inner.align = align;
        self
    }

    /// The config's `tex.font_size_for_unit_height`.
    #[must_use]
    pub fn font_size_for_unit_height(mut self, fsuh: f64) -> Self {
        self.inner = self.inner.font_size_for_unit_height(fsuh);
        self
    }

    /// Replace the base style.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.inner = self.inner.style(style);
        self
    }

    /// `tex_to_color_map` (`t2c`).
    #[must_use]
    pub fn t2c(mut self, t2c: &'a [(&'a str, Srgb)]) -> Self {
        self.inner = self.inner.t2c(t2c);
        self
    }

    /// Typeset and build the family.
    ///
    /// # Errors
    ///
    /// As [`Tex::build`].
    pub fn build(&self, engine: &TexEngine) -> Result<TexMobject, TexMobjectError> {
        self.inner.build(engine)
    }
}

/// `OldTex` / `OldTexText` (old_tex_mobject.py): the legacy multi-argument
/// Tex whose submobjects are **one group per argument** — `old_tex[1]` is
/// the whole second argument, not its first glyph (contrast `Tex`'s flat
/// glyph family).
///
/// The Reference typesets the joined string, then re-typesets every argument
/// on its own (`SingleStringTex`) and slices the full glyph run by those
/// counts — which silently misassigns glyphs whenever the parts lay out
/// differently in context. Here the joined string is typeset once and every
/// primitive joins the argument whose byte range holds its source span
/// start, the native provenance (§11.3). Arguments are first split around
/// every `isolate` / `tex_to_color_map` substring, as the Reference's
/// `break_up_tex_strings` does; arguments that are blank or draw nothing
/// form no group. A single argument is one group of every glyph.
///
/// `math_mode` true is `OldTex` (display mathematics, the Reference's
/// `align*`); false is `OldTexText` (text mainland with `$…$` islands).
#[derive(Debug, Clone)]
pub struct OldTex<'a> {
    strings: Vec<&'a str>,
    arg_separator: &'a str,
    isolate: Vec<&'a str>,
    t2c: &'a [(&'a str, Srgb)],
    math_mode: bool,
    font_size: f64,
    preamble: &'a str,
    style: Style,
}

/// A built [`OldTex`]: the grouped family plus each group's argument.
#[derive(Debug, Clone)]
pub struct OldTexMobject {
    /// One child per drawn argument, each a group of that argument's
    /// primitives in emission order. Layout coordinates; converting into a
    /// [`Mobject`] centres it, as the Reference's constructor does.
    pub vmob: VMobject,
    /// Group `i`'s argument, stripped (`old_tex[i].get_tex()`).
    pub tex_strings: Vec<String>,
    /// The typeset of the joined string.
    pub typeset: Typeset,
}

impl OldTexMobject {
    /// `get_parts_by_tex(tex, substring=True)`: the group indices whose
    /// argument contains `tex` (or equals it when `substring` is false).
    #[must_use]
    pub fn parts_by_tex(&self, tex: &str, substring: bool) -> Vec<usize> {
        self.tex_strings
            .iter()
            .enumerate()
            .filter(|(_, part)| {
                if substring {
                    part.contains(tex)
                } else {
                    part.as_str() == tex
                }
            })
            .map(|(index, _)| index)
            .collect()
    }

    /// `get_part_by_tex`: the first match of [`Self::parts_by_tex`].
    #[must_use]
    pub fn part_by_tex(&self, tex: &str) -> Option<usize> {
        self.parts_by_tex(tex, true).first().copied()
    }
}

impl From<OldTexMobject> for Mobject {
    fn from(t: OldTexMobject) -> Self {
        t.vmob.moved_to([0.0; 3]).into()
    }
}

impl<'a> OldTex<'a> {
    /// `OldTex(*tex_strings)` with the Reference's defaults: no separator,
    /// display mathematics, font size 48.
    #[must_use]
    pub fn new(strings: &[&'a str]) -> Self {
        Self {
            strings: strings.to_vec(),
            arg_separator: "",
            isolate: Vec::new(),
            t2c: &[],
            math_mode: true,
            font_size: DEFAULT_FONT_SIZE,
            preamble: "",
            style: text_style(),
        }
    }

    /// `OldTexText(*tex_strings)`: the same grouping over text mainland.
    #[must_use]
    pub fn text(strings: &[&'a str]) -> Self {
        Self {
            math_mode: false,
            ..Self::new(strings)
        }
    }

    /// The `arg_separator=` surface, joined between arguments.
    #[must_use]
    pub fn arg_separator(mut self, separator: &'a str) -> Self {
        self.arg_separator = separator;
        self
    }

    /// The `isolate=` surface: substrings split out as their own groups.
    #[must_use]
    pub fn isolate(mut self, isolate: &[&'a str]) -> Self {
        self.isolate = isolate.to_vec();
        self
    }

    /// `tex_to_color_map`: its keys are isolated, and every group whose
    /// argument contains a key takes that colour (later entries win).
    #[must_use]
    pub fn t2c(mut self, t2c: &'a [(&'a str, Srgb)]) -> Self {
        self.t2c = t2c;
        self
    }

    /// The `font_size=` surface.
    #[must_use]
    pub fn font_size(mut self, font_size: f64) -> Self {
        self.font_size = font_size;
        self
    }

    /// Native macro declarations for the joined request.
    #[must_use]
    pub fn preamble(mut self, preamble: &'a str) -> Self {
        self.preamble = preamble;
        self
    }

    /// Replace the base style.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// `break_up_tex_strings`: split each argument around every isolated
    /// substring (kept as its own piece), dropping empty pieces. The
    /// Reference's regex alternation takes the leftmost match, and the
    /// earliest-listed substring among those starting there.
    fn pieces(&self) -> Vec<&'a str> {
        let needles: Vec<&str> = self
            .isolate
            .iter()
            .copied()
            .chain(self.t2c.iter().map(|(needle, _)| *needle))
            .filter(|needle| !needle.is_empty())
            .collect();
        let mut pieces = Vec::new();
        for &string in &self.strings {
            let mut rest = string;
            loop {
                let next = needles
                    .iter()
                    .filter_map(|needle| rest.find(needle).map(|at| (at, *needle)))
                    .min_by_key(|(at, _)| *at);
                let Some((at, needle)) = next else {
                    pieces.push(rest);
                    break;
                };
                pieces.push(&rest[..at]);
                pieces.push(&rest[at..at + needle.len()]);
                rest = &rest[at + needle.len()..];
            }
        }
        pieces.retain(|piece| !piece.is_empty());
        pieces
    }

    /// Typeset the joined string and group its primitives by argument.
    ///
    /// # Errors
    ///
    /// As [`Tex::build`]: the joined string must typeset as a whole.
    pub fn build(&self, engine: &TexEngine) -> Result<OldTexMobject, TexMobjectError> {
        let pieces = self.pieces();
        let joined = pieces.join(self.arg_separator);
        let tex = Tex {
            source: &joined,
            preamble: self.preamble,
            mode: if self.math_mode {
                Mode::Math(MathStyle::Display)
            } else {
                Mode::Text
            },
            font_size: self.font_size,
            font_size_for_unit_height: DEFAULT_FONT_SIZE_FOR_UNIT_HEIGHT,
            style: self.style,
            t2c: &[],
            align: if self.math_mode {
                LineAlign::Left
            } else {
                LineAlign::Center
            },
        };
        let built = tex.build(engine)?;
        // Each piece's byte range in the joined source.
        let mut ranges = Vec::with_capacity(pieces.len());
        let mut offset = 0;
        for piece in &pieces {
            ranges.push(offset..offset + piece.len());
            offset += piece.len() + self.arg_separator.len();
        }
        let mut members: Vec<Vec<VMobject>> = vec![Vec::new(); pieces.len()];
        for (sub, child) in built.typeset.subs.iter().zip(built.vmob.children()) {
            // The piece holding the span start; a span starting inside a
            // separator belongs to the piece before it.
            let start = sub.span.start;
            let index = ranges
                .iter()
                .rposition(|range| range.start <= start)
                .unwrap_or(0);
            if let Some(group) = members.get_mut(index) {
                group.push(child.clone());
            }
        }
        let mut groups = Vec::new();
        let mut tex_strings = Vec::new();
        for (piece, children) in pieces.iter().zip(members) {
            let stripped = piece.trim();
            if stripped.is_empty() || children.is_empty() {
                continue;
            }
            let mut group = VMobject::new()
                .with_style(self.style)
                .with_children(children);
            for (needle, color) in self.t2c {
                if stripped.contains(needle) {
                    let color = *color;
                    group = group.map_style_deep(move |style| style.color(color));
                }
            }
            groups.push(group);
            tex_strings.push(stripped.to_owned());
        }
        Ok(OldTexMobject {
            vmob: VMobject::new().with_style(self.style).with_children(groups),
            tex_strings,
            typeset: built.typeset,
        })
    }
}

/// The ems→scene-units scale, calibrated the Reference's way: typeset a
/// reference "0" (text-style math, the Reference's calibration surface)
/// and scale so its height is `font_size / font_size_for_unit_height`.
pub(crate) fn calibrate(
    engine: &TexEngine,
    font_size: f64,
    fsuh: f64,
) -> Result<f64, TexMobjectError> {
    let probe = engine.typeset(Mode::Math(MathStyle::Text), "0")?;
    let height = probe.layout.height + probe.layout.depth;
    if !height.is_finite() || height <= 0.0 {
        return Err(TexMobjectError::Calibration);
    }
    Ok(font_size / (fsuh * height))
}

/// One `Sub` child: its resolved contours as a positioned [`QuadPath`],
/// scaled to scene units. A primitive with no contours keeps its slot
/// as an empty child — ordinals never shift.
fn prim_child(
    engine: &TexEngine,
    typeset: &Typeset,
    sub: &fmn_tex::Sub,
    style: Style,
    scale: f64,
) -> Result<VMobject, TexMobjectError> {
    let contours = engine.resolve_prim(typeset, sub.prim)?;
    if contours.is_empty() {
        return Ok(VMobject::new().with_style(style));
    }
    let mut path = QuadPath::new();
    for contour in &contours {
        append_contour(&mut path, contour, scale)?;
    }
    Ok(VMobject::from_path(&path).with_style(style))
}

/// Append one resolved contour as a subpath — the break convention
/// (`start_new_path`'s handle-on-anchor marker) is the geometry
/// kernel's own, so counters and multi-contour constructions compile
/// correctly downstream (fm-ig3).
fn append_contour(
    path: &mut QuadPath,
    contour: &PathContour,
    scale: f64,
) -> Result<(), TexMobjectError> {
    let v = |x: f64, y: f64| -> Vec3 { [x * scale, y * scale, 0.0] };
    let geometry = |e: fmn_geom::GeomError| TexMobjectError::Geometry {
        what: format!("{e:?}"),
    };
    path.start_new_path(v(contour.start.0, contour.start.1));
    for seg in &contour.segments {
        match seg {
            PathSeg::Line { to } => {
                path.add_line_to(v(to.0, to.1), true).map_err(geometry)?;
            }
            PathSeg::Quad { ctrl, to } => {
                path.add_quadratic_bezier_curve_to(v(ctrl.0, ctrl.1), v(to.0, to.1), true)
                    .map_err(geometry)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmn_core::constants::{BLUE, RED};
    use fmn_mobject::Stage;
    use fmn_tex::Prim;

    fn engine() -> TexEngine {
        TexEngine::new("fmd-math/pack/default", None).expect("engine")
    }

    #[test]
    fn old_tex_groups_primitives_by_argument() {
        let engine = engine();
        let old = OldTex::new(&["a^2", "+", "b^2"])
            .build(&engine)
            .expect("typesets");
        assert_eq!(old.tex_strings, ["a^2", "+", "b^2"]);
        let sizes: Vec<usize> = old
            .vmob
            .children()
            .iter()
            .map(|g| g.children().len())
            .collect();
        assert_eq!(sizes, [2, 1, 2]);
        // Every primitive of the joined typeset lands in exactly one group,
        // and the groups read left to right.
        assert_eq!(sizes.iter().sum::<usize>(), old.typeset.subs.len());
        let xs: Vec<f64> = old
            .vmob
            .children()
            .iter()
            .map(|g| g.center_point()[0])
            .collect();
        assert!(xs[0] < xs[1] && xs[1] < xs[2], "{xs:?}");
        assert_eq!(old.parts_by_tex("2", true), [0, 2]);
        assert_eq!(old.part_by_tex("+"), Some(1));
        assert!(old.parts_by_tex("a", false).is_empty());

        // One argument is one group of everything.
        let single = OldTex::new(&[r"\frac{1}{2}"])
            .build(&engine)
            .expect("typesets");
        assert_eq!(single.vmob.children().len(), 1);
        assert_eq!(
            single.vmob.children()[0].children().len(),
            single.typeset.subs.len()
        );
    }

    #[test]
    fn old_tex_isolates_and_colours_substrings() {
        let engine = engine();
        let t2c = [("y", RED)];
        let old = OldTex::new(&["x + y = z"])
            .isolate(&["="])
            .t2c(&t2c)
            .build(&engine)
            .expect("typesets");
        assert_eq!(old.tex_strings, ["x +", "y", "=", "z"]);
        let y = &old.vmob.children()[1];
        assert_eq!(y.style().fill_color, RED);
        assert!(
            y.children()
                .iter()
                .all(|glyph| glyph.style().fill_color == RED)
        );
        assert_ne!(old.vmob.children()[0].style().fill_color, RED);
        // Text mainland, with a separator between arguments.
        let text = OldTex::text(&["Hello", "world"])
            .arg_separator(" ")
            .build(&engine)
            .expect("typesets");
        assert_eq!(text.tex_strings, ["Hello", "world"]);
        assert_eq!(text.vmob.children()[0].children().len(), 5);
        assert_eq!(text.vmob.children()[1].children().len(), 5);
        // Added to a stage it stands centred, as the Reference's does.
        let mut stage = Stage::new();
        let mob = stage.add(text);
        let [x, y, _] = stage.get_center(mob);
        assert!(x.abs() < 1e-6 && y.abs() < 1e-6);
    }

    /// The Reference centres every Tex/TexText/Text at construction; a built
    /// mobject added to a stage stands there too, while the layout-space
    /// `vmob` keeps the baseline origin for composite builders.
    #[test]
    fn added_typeset_mobjects_are_centred_like_the_reference() {
        let engine = engine();
        let book = crate::FontBook::bundled().expect("bundled faces");
        let mut stage = Stage::new();
        let tex = Tex::new(r"\sum_{n=1}^{\infty} \frac{1}{n^2}")
            .build(&engine)
            .expect("typesets");
        let layout_center = tex.vmob.center_point();
        assert!(
            layout_center[0].abs() > 0.1,
            "layout space is baseline-anchored"
        );
        let text = crate::text::Text::new("Hello world")
            .build(&book)
            .expect("lays out");
        let textext = TexText::new(r"area $\pi r^2$")
            .build(&engine)
            .expect("typesets");
        for mob in [stage.add(tex), stage.add(text), stage.add(textext)] {
            let [x, y, _] = stage.get_center(mob);
            assert!(x.abs() < 1e-6 && y.abs() < 1e-6, "centre ({x}, {y})");
        }
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
    fn one_child_per_sub_with_the_span_map_intact() {
        let m = Tex::new(r"\frac{a}{b}").build(&engine()).expect("builds");
        assert_eq!(m.vmob.children().len(), m.typeset.subs.len());
        // Every sub is exactly one child, in order; 'a' and 'b' select
        // single children by source identity.
        // 'a' also appears inside "\frac" itself; the containment
        // semantics select nothing for that occurrence (the command's
        // primitives carry the command's whole span), so keep the
        // non-empty selections.
        for (needle, ch) in [("a", 'a'), ("b", 'b')] {
            let occ = m.occurrences(needle);
            let selected: Vec<&Vec<usize>> = occ.iter().filter(|o| !o.is_empty()).collect();
            assert_eq!(selected.len(), 1, "one selecting occurrence of {needle:?}");
            assert_eq!(selected[0].len(), 1, "one glyph for {needle:?}");
            let ord = selected[0][0];
            assert!(
                matches!(m.typeset.subs[ord].prim, fmn_tex::Prim::Glyph(g)
                    if m.typeset.layout.glyphs[g].ch == ch),
                "{needle:?} should be a glyph for {ch:?}, got {:?}",
                m.typeset.subs[ord].prim
            );
            assert!(!m.vmob.children()[ord].points().is_empty());
        }
        // "\frac" itself is not matched by a containment query for "a".
        assert_eq!(m.occurrences("frac").len(), 1);
    }

    #[test]
    fn a_fraction_rule_is_a_rectangle_child() {
        let m = Tex::new(r"\frac{a}{b}").build(&engine()).expect("builds");
        let (rule_ord, r) = m
            .typeset
            .subs
            .iter()
            .enumerate()
            .find_map(|(i, s)| match s.prim {
                Prim::Rule(r) => Some((i, r)),
                _ => None,
            })
            .expect("a fraction has a rule");
        let rule = &m.typeset.layout.rules[r];
        let child = &m.vmob.children()[rule_ord];
        let (min, max) = child.extent().expect("has extent");
        // The child is the rule's rectangle at the calibrated scale:
        // recompute the scale from the "0" probe, exactly as build does.
        let probe = engine()
            .typeset(Mode::Math(MathStyle::Text), "0")
            .expect("probe");
        let scale = DEFAULT_FONT_SIZE
            / (DEFAULT_FONT_SIZE_FOR_UNIT_HEIGHT * (probe.layout.height + probe.layout.depth));
        assert!((min[0] - rule.x * scale).abs() < 1e-9);
        assert!((max[0] - (rule.x + rule.width) * scale).abs() < 1e-9);
        assert!((min[1] - rule.y * scale).abs() < 1e-9);
        assert!((max[1] - (rule.y + rule.height) * scale).abs() < 1e-9);
        let path = child.path().expect("a valid path");
        assert!(path.is_closed(), "a rule rectangle is closed");
    }

    #[test]
    fn a_stretchy_overbrace_is_a_drawn_path_child() {
        // Past the glyph-scaling ceiling, the delimiter engine draws the
        // construction parametrically (ADR-0005): one PlacedPath sub
        // (scribe2's fixture: overbrace = 3 glyphs / 0 rules / 1 path).
        let m = Tex::new(r"\overbrace{abc}")
            .build(&engine())
            .expect("builds");
        let path_ords: Vec<usize> = m
            .typeset
            .subs
            .iter()
            .enumerate()
            .filter(|(_, s)| matches!(s.prim, Prim::Path(_)))
            .map(|(i, _)| i)
            .collect();
        assert_eq!(path_ords.len(), 1, "one drawn radical path");
        let child = &m.vmob.children()[path_ords[0]];
        assert!(!child.points().is_empty(), "the radical has geometry");
        assert_eq!(
            child.points().len() % 2,
            1,
            "shared-anchor runs have odd length"
        );
    }

    #[test]
    fn t2c_colors_by_source_identity() {
        let t2c = [("x", RED), ("y", BLUE)];
        let m = Tex::new("x+y").t2c(&t2c).build(&engine()).expect("builds");
        let x = &m.occurrences("x")[0];
        let y = &m.occurrences("y")[0];
        let plus = &m.occurrences("+")[0];
        for &ord in x {
            assert_eq!(m.vmob.children()[ord].style().fill_color, RED);
        }
        for &ord in y {
            assert_eq!(m.vmob.children()[ord].style().fill_color, BLUE);
        }
        for &ord in plus {
            assert_eq!(
                m.vmob.children()[ord].style().fill_color,
                fmn_core::constants::WHITE
            );
        }
    }

    #[test]
    fn the_tex_style_is_the_text_style() {
        let m = Tex::new("x").build(&engine()).expect("builds");
        for child in m.vmob.children() {
            let s = child.style();
            assert_eq!(s.stroke_width, 0.0);
            assert_eq!(s.fill_opacity, 1.0);
            assert_eq!(s.fill_border_width, 0.5);
        }
    }

    #[test]
    fn the_calibration_makes_a_digit_font_size_over_fsuh_tall() {
        let engine = engine();
        for font_size in [48.0, 96.0] {
            let m = Tex::new("0")
                .font_size(font_size)
                .build(&engine)
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
    fn display_style_changes_the_layout() {
        let engine = engine();
        let text = Tex::new(r"\frac{a}{b}")
            .math_style(MathStyle::Text)
            .build(&engine)
            .expect("builds");
        let display = Tex::new(r"\frac{a}{b}")
            .display()
            .build(&engine)
            .expect("builds");
        let extent = |m: &TexMobject| {
            let (min, max) = m.vmob.extent().expect("has extent");
            max[1] - min[1]
        };
        assert!(
            extent(&display) > extent(&text),
            "display fractions are taller: {} vs {}",
            extent(&display),
            extent(&text)
        );
    }

    /// A display fraction sets its numerator in text style, at full size;
    /// a text-style fraction drops it to script size (fm-tex-display-style-nclg).
    #[test]
    fn display_fraction_numerators_are_text_size() {
        let engine = engine();
        let height = |m: &VMobject| {
            let (min, max) = m.extent().expect("has extent");
            max[1] - min[1]
        };
        let digit = height(&Tex::new("1").build(&engine).expect("builds").vmob);
        let numerator = |style: MathStyle| {
            let tex = Tex::new(r"\frac{1}{2}")
                .math_style(style)
                .build(&engine)
                .expect("builds");
            let ordinal = tex.occurrences("1")[0][0];
            height(&tex.vmob.children()[ordinal]) / digit
        };
        let display = numerator(MathStyle::Display);
        assert!(display >= 0.9, "display numerator is {display} of a digit");
        let text = numerator(MathStyle::Text);
        assert!(text < 0.8, "text-style numerator is {text} of a digit");
    }

    /// A display `\int` takes cmex10's display-size glyph, twice the text
    /// one, as the Reference measures (1.082 vs 0.541 tall); `\sum` keeps
    /// its 1.4 (fm-tex-display-style-nclg, franken_markdown 68fe29b).
    #[test]
    fn display_integrals_use_the_display_size_glyph() {
        let engine = engine();
        let height = |source: &str, style: MathStyle| {
            let tex = Tex::new(source)
                .math_style(style)
                .build(&engine)
                .expect("builds");
            let (min, max) = tex.vmob.extent().expect("has extent");
            max[1] - min[1]
        };
        let integral = height(r"\int", MathStyle::Display) / height(r"\int", MathStyle::Text);
        assert!(
            (integral - 2.0).abs() < 1e-6,
            "display/text integral {integral}"
        );
        let sum = height(r"\sum", MathStyle::Display) / height(r"\sum", MathStyle::Text);
        assert!((sum - 1.4).abs() < 1e-6, "display/text sum {sum}");
    }

    /// `TexText`'s `$…$` islands are inline (text-style) mathematics, as in
    /// LaTeX prose: the display default of `Tex` must not leak into them.
    #[test]
    fn textext_math_islands_stay_text_style() {
        let engine = engine();
        let height = |m: &TexMobject| {
            let (min, max) = m.vmob.extent().expect("has extent");
            max[1] - min[1]
        };
        let island = TexText::new(r"$\frac{1}{2}$")
            .build(&engine)
            .expect("builds");
        let inline = Tex::new(r"\frac{1}{2}")
            .math_style(MathStyle::Text)
            .build(&engine)
            .expect("builds");
        let display = Tex::new(r"\frac{1}{2}").build(&engine).expect("builds");
        assert!(
            (height(&island) - height(&inline)).abs() < 1e-9,
            "island {} vs inline {}",
            height(&island),
            height(&inline)
        );
        assert!(height(&display) > 1.3 * height(&island));
    }

    /// The Reference typesets every `Tex` inside `align*`, i.e. display
    /// math (fm-tex-display-style-nclg): the default must be display style,
    /// so big-operator limits stack and fractions take display size.
    #[test]
    fn the_default_is_the_references_display_style() {
        let engine = engine();
        let height = |m: &TexMobject| {
            let (min, max) = m.vmob.extent().expect("has extent");
            max[1] - min[1]
        };
        let source = r"\sum_{n=1}^{N} \frac{1}{n^2}";
        let default = Tex::new(source).build(&engine).expect("builds");
        let display = Tex::new(source).display().build(&engine).expect("builds");
        let text = Tex::new(source)
            .math_style(MathStyle::Text)
            .build(&engine)
            .expect("builds");
        assert_eq!(dump_family(&default.vmob), dump_family(&display.vmob));
        // Reference (6199a00d, TeX Live 2025): 1.413 tall in display style;
        // the text-style layout is under half that.
        assert!(
            height(&default) > 1.8 * height(&text),
            "display {} vs text {}",
            height(&default),
            height(&text)
        );
        // Limits stack: the lower limit is centred on the summation sign
        // (TeX centres limits on the operator) and lies entirely below it.
        let sigma = default
            .typeset
            .subs
            .iter()
            .position(|sub| {
                matches!(sub.prim, Prim::Glyph(g) if default.typeset.layout.glyphs[g].ch == '∑')
            })
            .expect("a summation glyph");
        let (sigma_min, sigma_max) = default.vmob.children()[sigma]
            .extent()
            .expect("sigma extent");
        let lower = default.occurrences("n=1");
        let lower: Vec<usize> = lower.into_iter().flatten().collect();
        assert!(!lower.is_empty(), "the lower limit selects glyphs");
        let (mut left, mut right) = (f64::INFINITY, f64::NEG_INFINITY);
        for ord in lower {
            let (min, max) = default.vmob.children()[ord].extent().expect("limit extent");
            assert!(
                max[1] < sigma_min[1],
                "lower-limit glyph not below the sign"
            );
            left = left.min(min[0]);
            right = right.max(max[0]);
        }
        let limit_centre = 0.5 * (left + right);
        let sign_centre = 0.5 * (sigma_min[0] + sigma_max[0]);
        assert!(
            (limit_centre - sign_centre).abs() < 0.03,
            "lower limit centred at {limit_centre}, sign at {sign_centre}"
        );
    }

    #[test]
    fn an_unsupported_construct_is_the_named_tier_tagged_error() {
        // The pending tier-2 example advances as constructs graduate (fm-j5t).
        let err = Tex::new(r"\dx").build(&engine()).expect_err("fails");
        assert!(
            matches!(&err, TexMobjectError::Tex(TexError::Math(_))),
            "expected a math error, got {err:?}"
        );
        // TexMobjectError's Display delegates to the MathError verbatim.
        let what = err.to_string();
        assert!(what.contains("\\dx"), "names the construct: {what}");
        assert!(what.contains("tier"), "carries the tier tag: {what}");
    }

    #[test]
    fn an_unmapped_char_is_the_named_error() {
        let err = Tex::new("x 🦀 y").build(&engine()).expect_err("fails");
        assert!(
            matches!(&err, TexMobjectError::Tex(TexError::Math(_))),
            "expected a math error, got {err:?}"
        );
        let what = err.to_string();
        assert!(what.contains('🦀'), "names the char: {what}");
    }

    #[test]
    fn textext_mixes_prose_and_math_islands() {
        let m = TexText::new("a $b$ c").build(&engine()).expect("builds");
        assert_eq!(m.vmob.children().len(), m.typeset.subs.len());
        // The island glyph is found by source identity.
        let occ = m.occurrences("b");
        assert_eq!(occ.len(), 1);
        assert!(!m.vmob.children()[occ[0][0]].points().is_empty());
    }

    #[test]
    fn resolve_prim_names_a_bad_index() {
        let engine = engine();
        let m = Tex::new("x").build(&engine).expect("builds");
        let err = engine
            .resolve_prim(&m.typeset, Prim::Glyph(999))
            .expect_err("out of range");
        assert!(
            matches!(&err, TexError::BadPrim { .. }),
            "expected BadPrim, got {err:?}"
        );
        let what = err.to_string();
        assert!(what.contains("999"), "names the index: {what}");
    }

    #[test]
    fn the_family_enters_the_arena_as_a_family() {
        let mut stage = Stage::new();
        let m = Tex::new("xy").build(&engine()).expect("builds");
        let n = m.vmob.children().len();
        let mob = stage.add(m.vmob);
        assert_eq!(stage.family(mob).len(), n + 1);
    }

    /// Golden maintenance: `cargo test -p fmn-library -- --ignored
    /// regenerate_tex_goldens` rewrites the golden files after a
    /// deliberate fmd-math / fmd-font pin move. The regenerated files
    /// are review material — never regenerate casually.
    #[test]
    #[ignore]
    fn regenerate_tex_goldens() {
        let engine = engine();
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/goldens");
        std::fs::create_dir_all(dir).expect("goldens dir");
        let zero = Tex::new("0").build(&engine).expect("builds");
        std::fs::write(format!("{dir}/tex_zero.txt"), dump_family(&zero.vmob))
            .expect("write tex_zero");
        let island = TexText::new("a $b$").build(&engine).expect("builds");
        std::fs::write(
            format!("{dir}/textext_island.txt"),
            dump_family(&island.vmob),
        )
        .expect("write textext_island");
    }

    #[test]
    fn tex_zero_golden() {
        let m = Tex::new("0").build(&engine()).expect("builds");
        assert_eq!(m.vmob.children().len(), 1);
        let dump = dump_family(&m.vmob);
        let expected = include_str!("../tests/goldens/tex_zero.txt");
        assert_eq!(
            dump, expected,
            "golden drift (see tests/goldens/tex_zero.txt)"
        );
    }

    /// The Reference's TexText default is `alignment="\centering"`: each
    /// `\\`-split line centers on the widest. `line_align(Left)` keeps
    /// fmd-math's flush-left block.
    #[test]
    fn textext_centers_its_lines_by_default() {
        let line_centers = |m: &TexMobject| {
            let mut lines: Vec<(f64, f64, f64)> = Vec::new();
            for glyph in m.vmob.children() {
                let xs = glyph.points().iter().map(|p| p[0]);
                let (lo, hi) = xs.fold((f64::MAX, f64::MIN), |(l, h), x| (l.min(x), h.max(x)));
                let y =
                    glyph.points().iter().map(|p| p[1]).sum::<f64>() / glyph.points().len() as f64;
                match lines.iter_mut().find(|line| (line.2 - y).abs() < 0.2) {
                    Some(line) => {
                        line.0 = line.0.min(lo);
                        line.1 = line.1.max(hi);
                    }
                    None => lines.push((lo, hi, y)),
                }
            }
            lines
                .iter()
                .map(|(lo, hi, _)| (lo + hi) / 2.0)
                .collect::<Vec<_>>()
        };
        // One glyph on both lines, so side bearings cancel: the center
        // environment centers boxes, not ink.
        // Spaces around `\\` do not move a line: LaTeX's `\\` unskips the
        // space before it and swallows the space after it (fmd-math
        // e911be2a, UPSTREAM_LEDGER row 15).
        for source in [r"aaaa\\aa", r"aaaa \\aa", r"aaaa\\ aa", r"aaaa \\ aa"] {
            let centered = TexText::new(source).build(&engine()).expect("builds");
            let centers = line_centers(&centered);
            assert_eq!(centers.len(), 2, "{source}");
            assert!(
                (centers[0] - centers[1]).abs() < 1e-3,
                "{source}: {centers:?}"
            );
        }
        let flush = TexText::new(r"aaaa\\aa")
            .line_align(LineAlign::Left)
            .build(&engine())
            .expect("builds");
        let centers = line_centers(&flush);
        assert!((centers[0] - centers[1]).abs() > 0.1, "{centers:?}");
    }

    #[test]
    fn textext_island_golden() {
        let m = TexText::new("a $b$").build(&engine()).expect("builds");
        let dump = dump_family(&m.vmob);
        let expected = include_str!("../tests/goldens/textext_island.txt");
        assert_eq!(
            dump, expected,
            "golden drift (see tests/goldens/textext_island.txt)"
        );
    }
}
