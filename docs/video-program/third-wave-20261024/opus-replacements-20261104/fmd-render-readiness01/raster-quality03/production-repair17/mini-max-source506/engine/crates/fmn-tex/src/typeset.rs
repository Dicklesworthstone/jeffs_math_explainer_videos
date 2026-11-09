//! The typeset result: fmd-math's layout plus the submobject structure and
//! the span-map consumption surface (§11.4–11.5), serializable for the
//! content-addressed cache.
//!
//! # Submobject structure (the SingleStringTex conventions, span-first)
//!
//! A [`Typeset`] enumerates its primitives as ordered submobjects, glyphs,
//! rules and drawn paths, each carrying its source span, in TeX's emission
//! order (fmd-math's `Layout::order`). That is the order the Reference's
//! SVG members follow, so scene code that indexes or slices a formula
//! (`eq[0][4:7]`) reaches the same primitives (fm-aia1). The *span* is
//! the matching surface: `isolate=`, `tex_to_color_map`, substring slicing
//! and `TransformMatchingTex` all match by source identity through
//! [`Typeset::occurrences`] (§11.3's consumption pattern; the Reference's
//! render-twice-and-align hack is dead).
//!
//! # Serialization
//!
//! [`Typeset::to_bytes`]/[`Typeset::from_bytes`] are the cache payload
//! codec: versioned magic, fixed little-endian, length-prefixed, floats as
//! IEEE-754 bits — a cache hit reproduces the layout **bit-for-bit**
//! (tested), which is what lets the cache key participate in the certified
//! input closure: a hit is definitionally equivalent to a recompute.
//! Encoding and decoding are total and allocation-fallible: corrupt,
//! non-canonical, over-limit, or temporarily unallocatable documents return
//! a typed [`TypesetError`] (the cache treats every one as a miss), never a
//! partial payload or a panic.

use fmd_math::{Layout, PathContour, PathSeg, Placed, PlacedGlyph, PlacedPath, PlacedRule, Span};
use std::collections::TryReserveError;
use std::fmt;
use std::str::Utf8Error;

/// The serialization format tag; bump on any layout change to the byte
/// format (the cache namespace version rides this). Version 2 adds the
/// emission-order table (fm-aia1).
pub const TYPESET_FORMAT_VERSION: u32 = 2;

/// Maximum encoded FMNTEX document size.
///
/// This matches fmn-hash's canonical field ceiling. A larger in-memory
/// layout remains usable, but deliberately goes uncached.
pub const TYPESET_DOCUMENT_LIMIT_BYTES: usize = 64 * 1024 * 1024;

// The last byte is TYPESET_FORMAT_VERSION.
const MAGIC: &[u8; 8] = b"FMNTEX\x00\x02";
const U32_BYTES: usize = 4;
const F64_BYTES: usize = 8;
const GLYPH_BYTES: usize = 44;
const RULE_BYTES: usize = 40;
const PATH_MIN_BYTES: usize = 12;
const CONTOUR_MIN_BYTES: usize = 20;
const LINE_SEGMENT_BYTES: usize = 17;
const QUAD_SEGMENT_BYTES: usize = 33;
/// One emission-order entry: a primitive-kind tag and a u32 index.
const ORDER_ENTRY_BYTES: usize = 5;

/// Failure to construct, encode, or decode a [`Typeset`] document.
#[derive(Debug)]
pub enum TypesetError {
    /// The encoded document exceeds the format's fixed resource ceiling.
    DocumentTooLarge {
        /// Required or supplied document bytes.
        bytes: usize,
        /// Maximum admitted document bytes.
        limit: usize,
    },
    /// A public count or source offset cannot be represented by the wire's
    /// fixed-width `u32` field.
    IntegerOutOfRange {
        /// Field that cannot be represented.
        field: &'static str,
        /// In-memory value that was refused.
        value: usize,
    },
    /// Aggregate encoded or in-memory size arithmetic overflowed.
    SizeOverflow {
        /// Aggregate whose size could not be represented.
        context: &'static str,
    },
    /// Required owned storage could not be reserved.
    AllocationFailed {
        /// Storage being reserved.
        context: &'static str,
        /// Elements or bytes requested, as named by `context`.
        requested: usize,
        /// Allocator refusal.
        error: TryReserveError,
    },
    /// The format magic/version tag is not FMNTEX v2.
    InvalidMagic,
    /// A fixed or declared field ran past the supplied document.
    UnexpectedEnd {
        /// Bytes requested by the field.
        requested: usize,
        /// Bytes still available.
        remaining: usize,
    },
    /// The source field is not UTF-8.
    InvalidUtf8 {
        /// UTF-8 validation failure.
        error: Utf8Error,
    },
    /// A collection count cannot fit in the bytes still present even at its
    /// smallest legal item encoding.
    ImpossibleCount {
        /// Collection carrying the count.
        field: &'static str,
        /// Declared item count.
        count: usize,
        /// Smallest bytes per item.
        minimum_item_bytes: usize,
        /// Bytes remaining after the count.
        remaining_bytes: usize,
    },
    /// A path segment carries no defined FMNTEX tag.
    InvalidSegmentTag {
        /// Refused tag byte.
        tag: u8,
    },
    /// Bytes remain after the single canonical document.
    TrailingBytes {
        /// Unconsumed byte count.
        bytes: usize,
    },
    /// Public or decoded data has no canonical FMNTEX representation.
    NonCanonical {
        /// Field or table that failed validation.
        field: &'static str,
        /// Stable refusal reason.
        reason: &'static str,
    },
    /// A redundant structural count disagrees with the canonical layout.
    CountMismatch {
        /// Count being checked.
        field: &'static str,
        /// Count implied by the layout.
        expected: usize,
        /// Count carried by the document or public table.
        actual: usize,
    },
}

impl fmt::Display for TypesetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DocumentTooLarge { bytes, limit } => {
                write!(f, "FMNTEX document has {bytes} bytes; limit is {limit}")
            }
            Self::IntegerOutOfRange { field, value } => {
                write!(f, "FMNTEX {field} value {value} does not fit u32")
            }
            Self::SizeOverflow { context } => {
                write!(f, "FMNTEX {context} size overflowed usize")
            }
            Self::AllocationFailed {
                context,
                requested,
                error,
            } => write!(
                f,
                "FMNTEX could not reserve {requested} units for {context}: {error}"
            ),
            Self::InvalidMagic => f.write_str("not an FMNTEX v2 document"),
            Self::UnexpectedEnd {
                requested,
                remaining,
            } => write!(
                f,
                "FMNTEX field needs {requested} bytes but only {remaining} remain"
            ),
            Self::InvalidUtf8 { error } => write!(f, "FMNTEX source is not UTF-8: {error}"),
            Self::ImpossibleCount {
                field,
                count,
                minimum_item_bytes,
                remaining_bytes,
            } => write!(
                f,
                "FMNTEX {field} count {count} needs at least {minimum_item_bytes} bytes per item, but only {remaining_bytes} bytes remain"
            ),
            Self::InvalidSegmentTag { tag } => {
                write!(f, "FMNTEX path segment tag {tag} is undefined")
            }
            Self::TrailingBytes { bytes } => {
                write!(f, "FMNTEX document has {bytes} trailing bytes")
            }
            Self::NonCanonical { field, reason } => {
                write!(f, "FMNTEX {field} is non-canonical: {reason}")
            }
            Self::CountMismatch {
                field,
                expected,
                actual,
            } => write!(
                f,
                "FMNTEX {field} count is {actual}; canonical layout requires {expected}"
            ),
        }
    }
}

impl std::error::Error for TypesetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::AllocationFailed { error, .. } => Some(error),
            Self::InvalidUtf8 { error } => Some(error),
            _ => None,
        }
    }
}

/// Which primitive a submobject is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prim {
    /// `Layout::glyphs[i]`.
    Glyph(usize),
    /// `Layout::rules[i]`.
    Rule(usize),
    /// `Layout::paths[i]`.
    Path(usize),
}

/// Built-in commands whose own ink (fraction bar, radical, accent, over or
/// under line or brace) fmd-math spans over the whole construct. The
/// Reference selects that ink by the keyword alone: its
/// `select_unisolated_substring` counts `\frac`, `\over`, `\sqrt` or `\hat`
/// as one symbol. So [`Typeset::new`] re-spans the command's own ink to the
/// keyword token, and argument ink keeps its own spans. Macro calls are
/// untouched: their ink keeps the call-site span.
pub const KEYWORD_INK_COMMANDS: &[&str] = &[
    "frac",
    "dfrac",
    "tfrac",
    "cfrac",
    "sqrt",
    "over",
    "overline",
    "underline",
    "hat",
    "widehat",
    "check",
    "tilde",
    "widetilde",
    "acute",
    "grave",
    "dot",
    "ddot",
    "breve",
    "bar",
    "vec",
    "overbrace",
    "underbrace",
    "overrightarrow",
    "overleftarrow",
];

/// The keyword token that owns a primitive drawn by one of
/// [`KEYWORD_INK_COMMANDS`], if its span is that command's whole construct:
/// a top-level infix `\over` (whose group span can begin with another
/// command, as in `{\sqrt{x} \over 2}`), else a prefix command at the span's
/// start.
fn keyword_ink_span(source: &str, span: Span) -> Option<Span> {
    let text = source.get(span.start..span.end)?;
    let token_at = |offset: usize| -> Option<&str> {
        let name = text.get(offset + 1..)?;
        let len = name.bytes().take_while(u8::is_ascii_alphabetic).count();
        (text.as_bytes().get(offset) == Some(&b'\\') && len > 0).then(|| &name[..len])
    };
    if let Some(infix) = infix_over(text, &token_at) {
        let start = span.start + infix;
        return Some(Span::new(start, start + "\\over".len()));
    }
    let name = token_at(0)?;
    (name != "over" && KEYWORD_INK_COMMANDS.contains(&name) && text.len() > 1 + name.len())
        .then(|| Span::new(span.start, span.start + 1 + name.len()))
}

/// The byte offset of a `\over` at brace depth 0 in `text`.
fn infix_over<'a>(text: &'a str, token_at: &dyn Fn(usize) -> Option<&'a str>) -> Option<usize> {
    let mut depth = 0_i32;
    let mut index = 0;
    let bytes = text.as_bytes();
    while index < bytes.len() {
        match bytes[index] {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            b'\\' => {
                if let Some(name) = token_at(index) {
                    if depth == 0 && name == "over" {
                        return Some(index);
                    }
                    index += name.len();
                } else {
                    // A control symbol (\{, \\, \,): skip its character.
                    index += 1;
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

/// One submobject: a primitive plus its source span.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sub {
    /// The primitive.
    pub prim: Prim,
    /// Its source byte span.
    pub span: Span,
}

/// A typeset string: the layout, the submobject table, and the source.
#[derive(Clone, Debug, PartialEq)]
pub struct Typeset {
    /// The source string, verbatim.
    pub source: String,
    /// fmd-math's placed output (ems, y-up, baseline 0).
    pub layout: Layout,
    /// The ordered submobjects.
    pub subs: Vec<Sub>,
}

impl Typeset {
    /// Build the submobject table over a layout.
    ///
    /// # Errors
    ///
    /// [`TypesetError::SizeOverflow`] if the primitive counts cannot be
    /// aggregated, or [`TypesetError::AllocationFailed`] if the canonical
    /// submobject table cannot be reserved.
    pub fn new(source: String, mut layout: Layout) -> Result<Self, TypesetError> {
        let spans = layout
            .glyphs
            .iter_mut()
            .map(|glyph| &mut glyph.span)
            .chain(layout.rules.iter_mut().map(|rule| &mut rule.span))
            .chain(layout.paths.iter_mut().map(|path| &mut path.span));
        for span in spans {
            if let Some(keyword) = keyword_ink_span(&source, *span) {
                *span = keyword;
            }
        }
        let subs = submobject_table(&layout)?;
        Ok(Self {
            source,
            layout,
            subs,
        })
    }

    pub(crate) fn from_borrowed(source: &str, layout: Layout) -> Result<Self, TypesetError> {
        let mut owned = String::new();
        owned
            .try_reserve_exact(source.len())
            .map_err(|error| TypesetError::AllocationFailed {
                context: "source string",
                requested: source.len(),
                error,
            })?;
        owned.push_str(source);
        Self::new(owned, layout)
    }

    /// The submobject ordinals selected by each occurrence of `needle` in
    /// the source — the `isolate=` / `tex_to_color_map` /
    /// `TransformMatchingTex` surface, by source identity (§11.3).
    #[must_use]
    pub fn occurrences(&self, needle: &str) -> Vec<Vec<usize>> {
        fmd_math::find_occurrences(&self.source, needle)
            .into_iter()
            .map(|span| {
                let sel = self.layout.select(span);
                let mut ords = Vec::new();
                for (ord, sub) in self.subs.iter().enumerate() {
                    let hit = match sub.prim {
                        Prim::Glyph(i) => sel.glyphs.contains(&i),
                        Prim::Rule(i) => sel.rules.contains(&i),
                        Prim::Path(i) => sel.paths.contains(&i),
                    };
                    if hit {
                        ords.push(ord);
                    }
                }
                ords
            })
            .collect()
    }

    // ── The cache payload codec ─────────────────────────────────────────

    /// Serialize for the cache: versioned, little-endian, bit-exact floats.
    ///
    /// # Errors
    ///
    /// Returns [`TypesetError`] without emitting a partial document when
    /// public data is non-canonical, a fixed-width field or aggregate size
    /// overflows, the document exceeds [`TYPESET_DOCUMENT_LIMIT_BYTES`], or
    /// its exact storage reservation is refused.
    pub fn to_bytes(&self) -> Result<Vec<u8>, TypesetError> {
        self.validate_for_codec()?;
        let encoded_len = self.encoded_len()?;
        if encoded_len > TYPESET_DOCUMENT_LIMIT_BYTES {
            return Err(TypesetError::DocumentTooLarge {
                bytes: encoded_len,
                limit: TYPESET_DOCUMENT_LIMIT_BYTES,
            });
        }

        let mut bytes = Vec::new();
        reserve_exact(&mut bytes, encoded_len, "encoded document bytes")?;
        let mut w = Wr(bytes);
        w.0.extend_from_slice(MAGIC);
        w.bytes(self.source.as_bytes(), "source length")?;
        w.f64(self.layout.width);
        w.f64(self.layout.height);
        w.f64(self.layout.depth);
        w.count(self.layout.glyphs.len(), "glyph count")?;
        for g in &self.layout.glyphs {
            w.count(g.face.0, "glyph face id")?;
            w.raw_u32(u32::from(g.gid));
            w.raw_u32(u32::from(g.ch));
            w.f64(g.x);
            w.f64(g.y);
            w.f64(g.size);
            w.span(g.span, "glyph span start", "glyph span end")?;
        }
        w.count(self.layout.rules.len(), "rule count")?;
        for r in &self.layout.rules {
            w.f64(r.x);
            w.f64(r.y);
            w.f64(r.width);
            w.f64(r.height);
            w.span(r.span, "rule span start", "rule span end")?;
        }
        w.count(self.layout.paths.len(), "path count")?;
        for p in &self.layout.paths {
            w.span(p.span, "path span start", "path span end")?;
            w.count(p.contours.len(), "path contour count")?;
            for c in &p.contours {
                w.f64(c.start.0);
                w.f64(c.start.1);
                w.count(c.segments.len(), "path segment count")?;
                for s in &c.segments {
                    match s {
                        PathSeg::Line { to } => {
                            w.0.push(1);
                            w.f64(to.0);
                            w.f64(to.1);
                        }
                        PathSeg::Quad { ctrl, to } => {
                            w.0.push(2);
                            w.f64(ctrl.0);
                            w.f64(ctrl.1);
                            w.f64(to.0);
                            w.f64(to.1);
                        }
                    }
                }
            }
        }
        w.count(self.layout.order.len(), "emission order count")?;
        for placed in &self.layout.order {
            let (tag, index) = match *placed {
                Placed::Glyph(i) => (0, i),
                Placed::Rule(i) => (1, i),
                Placed::Path(i) => (2, i),
            };
            w.0.push(tag);
            w.count(index, "emission order index")?;
        }
        // The submobject table is derivable from the layout; store only a
        // count for a structural cross-check on decode.
        w.count(self.subs.len(), "submobject count")?;
        if w.0.len() != encoded_len {
            return Err(TypesetError::NonCanonical {
                field: "encoded document",
                reason: "preflighted size disagrees with emitted size",
            });
        }
        Ok(w.0)
    }

    /// Decode one cache payload.
    ///
    /// # Errors
    ///
    /// Returns a typed [`TypesetError`] on every format, canonicality,
    /// resource-limit, or allocation fault. Cache callers treat all errors as
    /// misses and re-typeset.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, TypesetError> {
        if bytes.len() > TYPESET_DOCUMENT_LIMIT_BYTES {
            return Err(TypesetError::DocumentTooLarge {
                bytes: bytes.len(),
                limit: TYPESET_DOCUMENT_LIMIT_BYTES,
            });
        }
        let mut r = Rd { b: bytes, at: 0 };
        if r.take(MAGIC.len())? != MAGIC.as_slice() {
            return Err(TypesetError::InvalidMagic);
        }
        let source =
            std::str::from_utf8(r.bytes()?).map_err(|error| TypesetError::InvalidUtf8 { error })?;
        let width = r.f64()?;
        let height = r.f64()?;
        let depth = r.f64()?;
        let mut layout = Layout {
            width,
            height,
            depth,
            ..Layout::default()
        };
        let glyph_count = r.count()?;
        r.ensure_count("glyphs", glyph_count, GLYPH_BYTES)?;
        reserve_exact(&mut layout.glyphs, glyph_count, "decoded glyph table")?;
        for _ in 0..glyph_count {
            layout.glyphs.push(PlacedGlyph {
                face: fmd_math::FaceId(r.count()?),
                gid: u16::try_from(r.raw_u32()?).map_err(|_| TypesetError::NonCanonical {
                    field: "glyph id",
                    reason: "value does not fit u16",
                })?,
                ch: char::from_u32(r.raw_u32()?).ok_or(TypesetError::NonCanonical {
                    field: "glyph character",
                    reason: "value is not a Unicode scalar",
                })?,
                x: r.f64()?,
                y: r.f64()?,
                size: r.f64()?,
                span: r.span()?,
            });
        }
        let rule_count = r.count()?;
        r.ensure_count("rules", rule_count, RULE_BYTES)?;
        reserve_exact(&mut layout.rules, rule_count, "decoded rule table")?;
        for _ in 0..rule_count {
            layout.rules.push(PlacedRule {
                x: r.f64()?,
                y: r.f64()?,
                width: r.f64()?,
                height: r.f64()?,
                span: r.span()?,
            });
        }
        let path_count = r.count()?;
        r.ensure_count("paths", path_count, PATH_MIN_BYTES)?;
        reserve_exact(&mut layout.paths, path_count, "decoded path table")?;
        for _ in 0..path_count {
            let span = r.span()?;
            let mut contours = Vec::new();
            let contour_count = r.count()?;
            r.ensure_count("path contours", contour_count, CONTOUR_MIN_BYTES)?;
            reserve_exact(&mut contours, contour_count, "decoded path contours")?;
            for _ in 0..contour_count {
                let start = (r.f64()?, r.f64()?);
                let mut segments = Vec::new();
                let segment_count = r.count()?;
                r.ensure_count("path segments", segment_count, LINE_SEGMENT_BYTES)?;
                reserve_exact(&mut segments, segment_count, "decoded path segments")?;
                for _ in 0..segment_count {
                    let tag = r.take(1)?[0];
                    segments.push(match tag {
                        1 => PathSeg::Line {
                            to: (r.f64()?, r.f64()?),
                        },
                        2 => PathSeg::Quad {
                            ctrl: (r.f64()?, r.f64()?),
                            to: (r.f64()?, r.f64()?),
                        },
                        _ => return Err(TypesetError::InvalidSegmentTag { tag }),
                    });
                }
                contours.push(PathContour { start, segments });
            }
            layout.paths.push(PlacedPath { contours, span });
        }
        let order_count = r.count()?;
        r.ensure_count("emission order", order_count, ORDER_ENTRY_BYTES)?;
        reserve_exact(&mut layout.order, order_count, "decoded emission order")?;
        for _ in 0..order_count {
            let tag = r.take(1)?[0];
            let index = r.count()?;
            layout.order.push(match tag {
                0 => Placed::Glyph(index),
                1 => Placed::Rule(index),
                2 => Placed::Path(index),
                _ => {
                    return Err(TypesetError::NonCanonical {
                        field: "emission order",
                        reason: "unknown primitive tag",
                    });
                }
            });
        }
        let encoded_subs = r.count()?;
        if r.remaining() != 0 {
            return Err(TypesetError::TrailingBytes {
                bytes: r.remaining(),
            });
        }
        let expected_subs = primitive_count(&layout)?;
        if encoded_subs != expected_subs {
            return Err(TypesetError::CountMismatch {
                field: "submobject table",
                expected: expected_subs,
                actual: encoded_subs,
            });
        }
        validate_layout(source, &layout)?;
        Self::from_borrowed(source, layout)
    }

    fn validate_for_codec(&self) -> Result<(), TypesetError> {
        validate_layout(&self.source, &self.layout)?;
        checked_u32("source length", self.source.len())?;
        checked_u32("glyph count", self.layout.glyphs.len())?;
        checked_u32("rule count", self.layout.rules.len())?;
        checked_u32("path count", self.layout.paths.len())?;
        checked_u32("emission order count", self.layout.order.len())?;
        checked_u32("submobject count", self.subs.len())?;

        let expected_subs = primitive_count(&self.layout)?;
        if self.subs.len() != expected_subs {
            return Err(TypesetError::CountMismatch {
                field: "submobject table",
                expected: expected_subs,
                actual: self.subs.len(),
            });
        }
        if self.subs != submobject_table(&self.layout)? {
            return Err(TypesetError::NonCanonical {
                field: "submobject table",
                reason: "differs from the table the emission order derives",
            });
        }
        Ok(())
    }

    fn encoded_len(&self) -> Result<usize, TypesetError> {
        let mut bytes = MAGIC.len();
        add_size(&mut bytes, U32_BYTES, "encoded document")?;
        add_size(&mut bytes, self.source.len(), "encoded document")?;
        add_size(&mut bytes, 3 * F64_BYTES + U32_BYTES, "encoded document")?;
        add_product(
            &mut bytes,
            self.layout.glyphs.len(),
            GLYPH_BYTES,
            "glyph table",
        )?;
        add_size(&mut bytes, U32_BYTES, "encoded document")?;
        add_product(
            &mut bytes,
            self.layout.rules.len(),
            RULE_BYTES,
            "rule table",
        )?;
        add_size(&mut bytes, U32_BYTES, "encoded document")?;
        for path in &self.layout.paths {
            add_size(&mut bytes, PATH_MIN_BYTES, "path table")?;
            for contour in &path.contours {
                add_size(&mut bytes, CONTOUR_MIN_BYTES, "path contour table")?;
                for segment in &contour.segments {
                    let segment_bytes = match segment {
                        PathSeg::Line { .. } => LINE_SEGMENT_BYTES,
                        PathSeg::Quad { .. } => QUAD_SEGMENT_BYTES,
                    };
                    add_size(&mut bytes, segment_bytes, "path segment table")?;
                }
            }
        }
        add_size(&mut bytes, U32_BYTES, "encoded document")?;
        add_product(
            &mut bytes,
            self.layout.order.len(),
            ORDER_ENTRY_BYTES,
            "emission order table",
        )?;
        add_size(&mut bytes, U32_BYTES, "encoded document")?;
        Ok(bytes)
    }
}

/// Little-endian writer.
struct Wr(Vec<u8>);

impl Wr {
    fn raw_u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }

    fn count(&mut self, v: usize, field: &'static str) -> Result<(), TypesetError> {
        self.raw_u32(checked_u32(field, v)?);
        Ok(())
    }

    fn f64(&mut self, v: f64) {
        self.0.extend_from_slice(&v.to_bits().to_le_bytes());
    }

    fn span(
        &mut self,
        s: Span,
        start_field: &'static str,
        end_field: &'static str,
    ) -> Result<(), TypesetError> {
        self.count(s.start, start_field)?;
        self.count(s.end, end_field)?;
        Ok(())
    }

    fn bytes(&mut self, b: &[u8], field: &'static str) -> Result<(), TypesetError> {
        self.count(b.len(), field)?;
        self.0.extend_from_slice(b);
        Ok(())
    }
}

/// Bounds-checked little-endian reader; every method is total.
struct Rd<'a> {
    b: &'a [u8],
    at: usize,
}

impl<'a> Rd<'a> {
    fn remaining(&self) -> usize {
        self.b.len() - self.at
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], TypesetError> {
        let remaining = self.remaining();
        if n > remaining {
            return Err(TypesetError::UnexpectedEnd {
                requested: n,
                remaining,
            });
        }
        let end = self.at + n;
        let s = &self.b[self.at..end];
        self.at = end;
        Ok(s)
    }

    fn raw_u32(&mut self) -> Result<u32, TypesetError> {
        let s = self.take(4)?;
        Ok(u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
    }

    fn count(&mut self) -> Result<usize, TypesetError> {
        usize::try_from(self.raw_u32()?).map_err(|_| TypesetError::SizeOverflow {
            context: "decoded u32 count",
        })
    }

    fn f64(&mut self) -> Result<f64, TypesetError> {
        let s = self.take(8)?;
        Ok(f64::from_bits(u64::from_le_bytes([
            s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7],
        ])))
    }

    fn span(&mut self) -> Result<Span, TypesetError> {
        let start = self.count()?;
        let end = self.count()?;
        Ok(Span::new(start, end))
    }

    fn bytes(&mut self) -> Result<&'a [u8], TypesetError> {
        let n = self.count()?;
        self.take(n)
    }

    fn ensure_count(
        &self,
        field: &'static str,
        count: usize,
        minimum_item_bytes: usize,
    ) -> Result<(), TypesetError> {
        let Some(minimum_bytes) = count.checked_mul(minimum_item_bytes) else {
            return Err(TypesetError::SizeOverflow { context: field });
        };
        let remaining_bytes = self.remaining();
        if minimum_bytes > remaining_bytes {
            return Err(TypesetError::ImpossibleCount {
                field,
                count,
                minimum_item_bytes,
                remaining_bytes,
            });
        }
        Ok(())
    }
}

fn primitive_count(layout: &Layout) -> Result<usize, TypesetError> {
    layout
        .glyphs
        .len()
        .checked_add(layout.rules.len())
        .and_then(|count| count.checked_add(layout.paths.len()))
        .ok_or(TypesetError::SizeOverflow {
            context: "submobject table",
        })
}

/// The submobject table: one entry per primitive, in the layout's emission
/// order, which must name every primitive exactly once.
fn submobject_table(layout: &Layout) -> Result<Vec<Sub>, TypesetError> {
    let count = primitive_count(layout)?;
    if layout.order.len() != count {
        return Err(TypesetError::CountMismatch {
            field: "emission order",
            expected: count,
            actual: layout.order.len(),
        });
    }
    let mut seen = Vec::new();
    reserve_exact(&mut seen, count, "emission order check")?;
    seen.resize(count, false);
    let mut subs = Vec::new();
    reserve_exact(&mut subs, count, "submobject table")?;
    let (glyphs, rules) = (layout.glyphs.len(), layout.rules.len());
    for placed in &layout.order {
        let (slot, sub) = match *placed {
            Placed::Glyph(i) => (
                i,
                layout.glyphs.get(i).map(|g| Sub {
                    prim: Prim::Glyph(i),
                    span: g.span,
                }),
            ),
            Placed::Rule(i) => (
                glyphs + i,
                layout.rules.get(i).map(|r| Sub {
                    prim: Prim::Rule(i),
                    span: r.span,
                }),
            ),
            Placed::Path(i) => (
                glyphs + rules + i,
                layout.paths.get(i).map(|p| Sub {
                    prim: Prim::Path(i),
                    span: p.span,
                }),
            ),
        };
        let Some(sub) = sub else {
            return Err(TypesetError::NonCanonical {
                field: "emission order",
                reason: "names a primitive the layout does not have",
            });
        };
        if seen[slot] {
            return Err(TypesetError::NonCanonical {
                field: "emission order",
                reason: "names a primitive twice",
            });
        }
        seen[slot] = true;
        subs.push(sub);
    }
    Ok(subs)
}

fn reserve_exact<T>(
    values: &mut Vec<T>,
    additional: usize,
    context: &'static str,
) -> Result<(), TypesetError> {
    values
        .try_reserve_exact(additional)
        .map_err(|error| TypesetError::AllocationFailed {
            context,
            requested: additional,
            error,
        })
}

fn checked_u32(field: &'static str, value: usize) -> Result<u32, TypesetError> {
    u32::try_from(value).map_err(|_| TypesetError::IntegerOutOfRange { field, value })
}

fn add_size(
    total: &mut usize,
    additional: usize,
    context: &'static str,
) -> Result<(), TypesetError> {
    *total = total
        .checked_add(additional)
        .ok_or(TypesetError::SizeOverflow { context })?;
    Ok(())
}

fn add_product(
    total: &mut usize,
    count: usize,
    item_bytes: usize,
    context: &'static str,
) -> Result<(), TypesetError> {
    let additional = count
        .checked_mul(item_bytes)
        .ok_or(TypesetError::SizeOverflow { context })?;
    add_size(total, additional, context)
}

fn validate_layout(source: &str, layout: &Layout) -> Result<(), TypesetError> {
    validate_f64("layout width", layout.width)?;
    validate_f64("layout height", layout.height)?;
    validate_f64("layout depth", layout.depth)?;
    checked_u32("glyph count", layout.glyphs.len())?;
    checked_u32("rule count", layout.rules.len())?;
    checked_u32("path count", layout.paths.len())?;

    for glyph in &layout.glyphs {
        checked_u32("glyph face id", glyph.face.0)?;
        validate_f64("glyph x", glyph.x)?;
        validate_f64("glyph y", glyph.y)?;
        validate_f64("glyph size", glyph.size)?;
        validate_span(
            source,
            glyph.span,
            "glyph span",
            "glyph span start",
            "glyph span end",
        )?;
    }
    for rule in &layout.rules {
        validate_f64("rule x", rule.x)?;
        validate_f64("rule y", rule.y)?;
        validate_f64("rule width", rule.width)?;
        validate_f64("rule height", rule.height)?;
        validate_span(
            source,
            rule.span,
            "rule span",
            "rule span start",
            "rule span end",
        )?;
    }
    for path in &layout.paths {
        checked_u32("path contour count", path.contours.len())?;
        validate_span(
            source,
            path.span,
            "path span",
            "path span start",
            "path span end",
        )?;
        for contour in &path.contours {
            checked_u32("path segment count", contour.segments.len())?;
            validate_f64("path contour start x", contour.start.0)?;
            validate_f64("path contour start y", contour.start.1)?;
            for segment in &contour.segments {
                match segment {
                    PathSeg::Line { to } => {
                        validate_f64("path line end x", to.0)?;
                        validate_f64("path line end y", to.1)?;
                    }
                    PathSeg::Quad { ctrl, to } => {
                        validate_f64("path quadratic control x", ctrl.0)?;
                        validate_f64("path quadratic control y", ctrl.1)?;
                        validate_f64("path quadratic end x", to.0)?;
                        validate_f64("path quadratic end y", to.1)?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn validate_f64(field: &'static str, value: f64) -> Result<(), TypesetError> {
    if !value.is_finite() {
        return Err(TypesetError::NonCanonical {
            field,
            reason: "value is not finite",
        });
    }
    if value == 0.0 && value.is_sign_negative() {
        return Err(TypesetError::NonCanonical {
            field,
            reason: "negative zero has no canonical encoding",
        });
    }
    Ok(())
}

fn validate_span(
    source: &str,
    span: Span,
    field: &'static str,
    start_field: &'static str,
    end_field: &'static str,
) -> Result<(), TypesetError> {
    checked_u32(start_field, span.start)?;
    checked_u32(end_field, span.end)?;
    if span.start > span.end {
        return Err(TypesetError::NonCanonical {
            field,
            reason: "start is after end",
        });
    }
    if span.end > source.len() {
        return Err(TypesetError::NonCanonical {
            field,
            reason: "span lies outside the source",
        });
    }
    if !source.is_char_boundary(span.start) || !source.is_char_boundary(span.end) {
        return Err(TypesetError::NonCanonical {
            field,
            reason: "span splits a UTF-8 scalar",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_counts_are_rejected_instead_of_saturated() {
        assert!(matches!(
            checked_u32("test count", usize::MAX),
            Err(TypesetError::IntegerOutOfRange {
                field: "test count",
                value: usize::MAX,
            })
        ));
    }

    #[test]
    fn reservation_refusal_is_typed() {
        let mut bytes = Vec::<u8>::new();
        assert!(matches!(
            reserve_exact(&mut bytes, usize::MAX, "test bytes"),
            Err(TypesetError::AllocationFailed {
                context: "test bytes",
                requested: usize::MAX,
                ..
            })
        ));
    }
}
