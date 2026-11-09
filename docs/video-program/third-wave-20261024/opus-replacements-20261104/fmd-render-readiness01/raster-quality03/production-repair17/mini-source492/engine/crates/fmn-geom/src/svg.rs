//! The SVG document processor (§7.6, fm-6nm): user SVGs, hardened, with an
//! explicit accept/reject matrix.
//!
//! This is the substrate `SVGMobject` builds on — a real document processor
//! that displaces the Reference's `svgelements` dependency. Scope is USER
//! SVGs: with dvisvgm gone there are no TeX-output quirks to replicate, so
//! this is a clean, hardened importer for untrusted input (§16.5, R14).
//!
//! The output is the parsed + resolved *document*: a viewport, the optional
//! `viewBox`, and a flat list of [`SvgShape`]s — each a [`QuadPath`] in
//! viewport user-space coordinates (the full transform cascade and the
//! `viewBox` mapping already applied, z = 0) with a resolved [`SvgStyle`]
//! record attached. A consumer in fmn-library later builds VMobjects from
//! these records; nothing here knows about mobjects.
//!
//! # The accept/reject matrix
//!
//! Every SVG feature is either implemented, refused with a *named* error, or
//! explicitly ignored. Nothing is silently dropped.
//!
//! | Feature                                            | Decision |
//! |----------------------------------------------------|----------|
//! | `svg` (root and nested viewports)                  | accepted; nested `overflow` clipping is NOT applied |
//! | `g`, `a` (fragment hrefs only), `symbol`, `defs`   | accepted |
//! | `use` (fragment hrefs only)                        | accepted, under `max_use_expansions` with cycle detection ([`SvgError::Cycle`]) |
//! | `path` — M/L/H/V/C/S/Q/T/A/Z, relative + absolute  | accepted; arcs go endpoint→center (SVG F.6.5), then cubic→quad through the one error-bounded converter ([`crate::cubic`]) |
//! | `rect` (incl. `rx`/`ry`), `circle`, `ellipse`, `line`, `polyline`, `polygon` | accepted |
//! | `viewBox`, `preserveAspectRatio` (all aligns, meet/slice, `none`) | accepted |
//! | Units `px pt pc mm cm in %` (+ unitless)           | accepted (`em`/`ex` are outside the subset → [`SvgError::Malformed`]) |
//! | `transform`: matrix/translate/scale/rotate/skewX/skewY | accepted, composed in document order |
//! | Presentation attributes + the `style` attribute    | accepted for the cascaded subset below |
//! | `fill`, `stroke`: `none`, hex, `rgb[a()]`, named colors, `currentColor`, `transparent` (→ alpha 0) | accepted |
//! | `fill-opacity`, `stroke-opacity`, `opacity` (multiplied down groups — a documented flattening of group-opacity compositing) | accepted |
//! | `fill-rule` (nonzero/evenodd), stroke cap/join/miterlimit, dasharray/dashoffset | accepted |
//! | `display:none`, `visibility`                       | accepted (subtree elision) |
//! | `color`                                            | accepted (feeds `currentColor`) |
//! | `clip-path` attributes and `clipPath` elements     | **rejected** — [`SvgError::UnsupportedFeature`] `"clip-path"` |
//! | `mask` attributes and elements                     | **rejected** — `"mask"` |
//! | `linearGradient`/`radialGradient`/`meshgradient`, and `fill`/`stroke` `url(#…)` resolving to one | **rejected** — `"gradient"` |
//! | `pattern`/`hatch`, and `url(#…)` resolving to one  | **rejected** — `"pattern"` |
//! | `filter` attributes and filter primitives          | **rejected** — `"filter"` |
//! | `marker*` attributes and `marker` elements         | **rejected** — `"marker"` |
//! | `text`/`tspan`/`textPath`, `image`, `foreignObject`, `script`, `style` (CSS), `animate*`/`set`/`mpath`, `switch`, `view`, fonts, audio/video, and any other element | **rejected** — [`SvgError::UnsupportedFeature`] naming the element |
//! | `title`, `desc`, `metadata`                        | ignored (non-rendering metadata) |
//! | comments, CDATA, processing instructions, `class`/`id`/`xmlns*` attributes | ignored |
//! | DOCTYPE (any, case-insensitive)                    | **rejected** — [`SvgError::DoctypeRefused`]; entity bombs are refused by construction |
//! | entities: `amp lt gt quot apos` + numeric `&#N;`/`&#xN;` | accepted; any other name → [`SvgError::UnknownEntity`] |
//! | any non-fragment `href` (http/https/file/data/…)   | **rejected** — [`SvgError::ExternalRef`]; there are no remote references of any kind |
//!
//! Out of the subset by design (documented, not bugs): CSS stylesheets and
//! selectors (`class` attributes are inert), unknown *properties* inside the
//! `style` attribute (they configure features outside this matrix), text
//! layout, raster images, scripting, animation, and all paint servers.
//! Text content between elements is ignored. Element-local tolerances: the
//! cubic→quadratic conversion runs at [`DEFAULT_SVG_TOLERANCE`] user units
//! *before* the transform cascade is applied, and `stroke-width`/dash
//! lengths are rescaled by `sqrt|det|` of the cascaded affine — both
//! documented flattening choices, per the matrix.
//!
//! # Security posture (§16.5, R14)
//!
//! This parses untrusted input. Therefore: budgets are declared before any
//! allocation ([`SvgLimits`]); DOCTYPE is refused by name, so there are no
//! custom entities and no entity-expansion attacks; every coordinate is
//! validated finite ([`SvgError::NonFinite`]); nesting depth, element count,
//! path-command count, and `use` expansion are all bounded; and the only
//! references are same-document `#fragment`s. Every failure is a typed
//! [`SvgError`] — this module never panics on any byte stream.

use std::collections::HashMap;

use fmn_core::color::Srgb;
use fmn_core::types::Vec3;

use crate::{FillRule, GeomError, QuadPath, cubic, scalar};

/// The cubic→quadratic conversion tolerance for SVG ingress, in path-local
/// user units: the G0-2 pixel tolerance applied to SVG's px-scaled user
/// space. See [`crate::cubic::DEFAULT_TOLERANCE_PX`].
pub const DEFAULT_SVG_TOLERANCE: f64 = cubic::DEFAULT_TOLERANCE_PX;

/// The κ for a quarter circle/ellipse cubic: `4/3 · tan(π/8)`.
const QUARTER_ARC_KAPPA: f64 = 0.552_284_749_830_793_6;

// ------------------------------------------------------------------ errors

/// Why byte input could not become a [`crate::SvgDocument`]. Every variant is a
/// named refusal; the parser never panics on any byte stream.
#[derive(Debug, Clone, PartialEq)]
pub enum SvgError {
    /// The input is not UTF-8.
    NotUtf8 {
        /// Byte offset of the first invalid UTF-8 sequence.
        offset: usize,
    },
    /// The input exceeds the byte budget (checked before any allocation).
    TooLarge {
        /// The input size in bytes.
        bytes: usize,
        /// The configured budget.
        limit: usize,
    },
    /// The XML or an attribute value is not well-formed. `line` is the
    /// 1-based line of the offending construct (attribute errors report
    /// their element's line).
    Malformed {
        /// 1-based line number.
        line: usize,
        /// What is wrong.
        message: String,
    },
    /// A DOCTYPE declaration was found. DOCTYPE is refused by name, so no
    /// DOCTYPE-defined entities exist at all.
    DoctypeRefused {
        /// 1-based line number.
        line: usize,
    },
    /// An entity reference that is not one of the five predefined XML
    /// entities or a numeric character reference.
    UnknownEntity {
        /// 1-based line number.
        line: usize,
        /// The entity name as written (without `&`/`;`).
        name: String,
    },
    /// A non-fragment reference (`http:`, `https:`, `file:`, `data:`, …).
    /// There are no remote references of any kind.
    ExternalRef {
        /// 1-based line number.
        line: usize,
        /// The reference as written.
        reference: String,
    },
    /// The element nesting depth exceeded the budget.
    TooDeep {
        /// The depth that was attempted (root is depth 1).
        depth: usize,
        /// The configured budget.
        limit: usize,
    },
    /// The element count exceeded the budget.
    TooManyElements {
        /// The configured budget.
        limit: usize,
    },
    /// The path-command count (across the whole document) exceeded the
    /// budget.
    TooManyCommands {
        /// The configured budget.
        limit: usize,
    },
    /// The `use`-expansion count exceeded the budget.
    TooManyUseExpansions {
        /// The configured budget.
        limit: usize,
    },
    /// A coordinate or scalar parsed to a non-finite value (`nan`, `inf`,
    /// overflowing exponent) or a transform produced one.
    NonFinite {
        /// 1-based line number.
        line: usize,
        /// What was being parsed (`"width"`, `"path data"`, …).
        context: &'static str,
    },
    /// A feature outside the accept matrix was used. See the module docs.
    UnsupportedFeature {
        /// 1-based line number.
        line: usize,
        /// The feature: `"clip-path"`, `"mask"`, `"gradient"`, `"pattern"`,
        /// `"filter"`, `"marker"`, or an element name.
        feature: String,
    },
    /// A `use` reference cycle (`#a` → … → `#a`).
    Cycle {
        /// The id that closed the cycle.
        id: String,
    },
    /// A `use` or paint-server reference to an id that does not exist.
    MissingReference {
        /// 1-based line number.
        line: usize,
        /// The missing id.
        id: String,
    },
    /// The geometry kernel refused (a converter budget, the shared-anchor
    /// invariant).
    Geometry {
        /// The kernel error.
        source: GeomError,
    },
}

fn malformed(line: usize, message: impl Into<String>) -> SvgError {
    SvgError::Malformed {
        line,
        message: message.into(),
    }
}

fn geom(source: GeomError) -> SvgError {
    SvgError::Geometry { source }
}

impl std::fmt::Display for SvgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotUtf8 { offset } => {
                write!(
                    f,
                    "input is not UTF-8 (first invalid byte at offset {offset})"
                )
            }
            Self::TooLarge { bytes, limit } => {
                write!(f, "input is {bytes} bytes, over the {limit}-byte budget")
            }
            Self::Malformed { line, message } => write!(f, "line {line}: {message}"),
            Self::DoctypeRefused { line } => write!(
                f,
                "line {line}: DOCTYPE is refused (no DOCTYPE-defined entities exist)"
            ),
            Self::UnknownEntity { line, name } => write!(
                f,
                "line {line}: entity `&{name};` is not a predefined or numeric entity"
            ),
            Self::ExternalRef { line, reference } => write!(
                f,
                "line {line}: external reference {reference:?} refused \
                 (only same-document #fragment references are accepted)"
            ),
            Self::TooDeep { depth, limit } => {
                write!(
                    f,
                    "element nesting depth {depth} exceeds the budget of {limit}"
                )
            }
            Self::TooManyElements { limit } => {
                write!(f, "element budget of {limit} exceeded")
            }
            Self::TooManyCommands { limit } => {
                write!(f, "path-command budget of {limit} exceeded")
            }
            Self::TooManyUseExpansions { limit } => {
                write!(f, "use-expansion budget of {limit} exceeded")
            }
            Self::NonFinite { line, context } => {
                write!(f, "line {line}: {context} is not a finite number")
            }
            Self::UnsupportedFeature { line, feature } => {
                write!(f, "line {line}: unsupported SVG feature: {feature}")
            }
            Self::Cycle { id } => write!(f, "use reference cycle through `#{id}`"),
            Self::MissingReference { line, id } => {
                write!(f, "line {line}: reference to unknown id `#{id}`")
            }
            Self::Geometry { source } => write!(f, "geometry kernel refused: {source}"),
        }
    }
}

impl std::error::Error for SvgError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Geometry { source } => Some(source),
            _ => None,
        }
    }
}

// ------------------------------------------------------------------ limits

/// Resource budgets for the SVG processor, declared before any allocation
/// (§16.5, R14).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SvgLimits {
    /// Maximum input size. Default 1 MiB — generous for any real icon or
    /// figure, far too small for a decompression-style bomb.
    pub max_bytes: usize,
    /// Maximum element nesting depth (root is depth 1). Default 32.
    pub max_depth: usize,
    /// Maximum path-data commands across the whole document (implicit
    /// repetitions count). Default `1 << 20`.
    pub max_path_commands: usize,
    /// Maximum `use`-element expansions across the whole document.
    /// Default 1024.
    pub max_use_expansions: usize,
    /// Maximum element count. Default `1 << 16`.
    pub max_elements: usize,
}

impl Default for SvgLimits {
    fn default() -> Self {
        Self {
            max_bytes: 1 << 20,
            max_depth: 32,
            max_path_commands: 1 << 20,
            max_use_expansions: 1024,
            max_elements: 1 << 16,
        }
    }
}

// ------------------------------------------------------------------ output

/// A paint value. Gradients and patterns are rejected by name (see the
/// module's accept/reject matrix), so the only accepted paint is a flat
/// color; `None` on the style field is SVG `none`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Paint {
    /// A flat sRGB color.
    Color(Srgb),
}

/// Stroke line caps (SVG `stroke-linecap`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineCap {
    /// Butt cap (the default).
    Butt,
    /// Round cap.
    Round,
    /// Square cap.
    Square,
}

impl std::fmt::Display for LineCap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Butt => write!(f, "butt"),
            Self::Round => write!(f, "round"),
            Self::Square => write!(f, "square"),
        }
    }
}

/// Stroke line joins (SVG `stroke-linejoin`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineJoin {
    /// Miter join (the default).
    Miter,
    /// Round join.
    Round,
    /// Bevel join.
    Bevel,
}

impl std::fmt::Display for LineJoin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Miter => write!(f, "miter"),
            Self::Round => write!(f, "round"),
            Self::Bevel => write!(f, "bevel"),
        }
    }
}

/// A fully resolved style record for one shape, after the
/// presentation-attribute + `style`-attribute cascade and group-opacity
/// folding. Opacities are clamped to [0, 1] per the SVG spec.
#[derive(Debug, Clone, PartialEq)]
pub struct SvgStyle {
    /// The fill paint (`None` = SVG `none`). Default: black.
    pub fill: Option<Paint>,
    /// `fill-opacity` (the `transparent` keyword folds its zero alpha here).
    pub fill_opacity: f64,
    /// `fill-rule`.
    pub fill_rule: FillRule,
    /// The stroke paint (`None` = SVG `none`, the default).
    pub stroke: Option<Paint>,
    /// `stroke-width` in viewport units, rescaled by `sqrt|det|` of the
    /// cascaded affine (see the module docs).
    pub stroke_width: f64,
    /// `stroke-opacity`.
    pub stroke_opacity: f64,
    /// `stroke-linecap`.
    pub line_cap: LineCap,
    /// `stroke-linejoin`.
    pub line_join: LineJoin,
    /// `stroke-miterlimit`.
    pub miter_limit: f64,
    /// `stroke-dasharray` (empty = solid), rescaled like `stroke_width`.
    pub stroke_dasharray: Vec<f64>,
    /// `stroke-dashoffset`, rescaled like `stroke_width`.
    pub stroke_dashoffset: f64,
    /// Element `opacity` multiplied down through ancestor groups (the
    /// documented flattening of group compositing).
    pub opacity: f64,
}

/// One shape of the resolved document: the geometry in viewport user-space
/// coordinates (transform cascade and viewBox mapping applied, z = 0) plus
/// its resolved style.
#[derive(Debug, Clone, PartialEq)]
pub struct SvgShape {
    /// The outline. Subpath breaks use the shared-anchor null-curve
    /// encoding; fill semantics come from [`SvgStyle::fill_rule`].
    pub path: QuadPath,
    /// The resolved style record.
    pub style: SvgStyle,
}

/// A parsed + resolved SVG document.
#[derive(Debug, Clone, PartialEq)]
pub struct SvgDocument {
    /// Viewport width in px (root `width` resolved, or the viewBox width,
    /// or the 300×150 replaced-element default).
    pub width: f64,
    /// Viewport height in px.
    pub height: f64,
    /// The root `viewBox` `(min_x, min_y, width, height)`, if declared.
    pub view_box: Option<[f64; 4]>,
    /// The flattened shapes, in document order.
    pub shapes: Vec<SvgShape>,
}

impl SvgDocument {
    /// Parse SVG bytes under the default budgets.
    pub fn parse(bytes: &[u8]) -> Result<Self, SvgError> {
        Self::parse_with_limits(bytes, &SvgLimits::default())
    }

    /// Parse SVG bytes under explicit budgets (the untrusted-input path).
    pub fn parse_with_limits(bytes: &[u8], limits: &SvgLimits) -> Result<Self, SvgError> {
        if bytes.len() > limits.max_bytes {
            return Err(SvgError::TooLarge {
                bytes: bytes.len(),
                limit: limits.max_bytes,
            });
        }
        let text = std::str::from_utf8(bytes).map_err(|e| SvgError::NotUtf8 {
            offset: e.valid_up_to(),
        })?;
        let root = build_tree(text, limits)?;
        Interpreter::new(limits, &root).document(&root)
    }
}

// ------------------------------------------------------------------ affine

/// A 2D affine `x' = a·x + c·y + e`, `y' = b·x + d·y + f`.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Affine {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    e: f64,
    f: f64,
}

impl Affine {
    const IDENTITY: Self = Self {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };

    fn translate(tx: f64, ty: f64) -> Self {
        Self {
            e: tx,
            f: ty,
            ..Self::IDENTITY
        }
    }

    fn scale(sx: f64, sy: f64) -> Self {
        Self {
            a: sx,
            d: sy,
            ..Self::IDENTITY
        }
    }

    fn rotate(degrees: f64) -> Self {
        let radians = degrees.to_radians();
        let (sin, cos) = (scalar::sin(radians), scalar::cos(radians));
        Self {
            a: cos,
            b: sin,
            c: -sin,
            d: cos,
            e: 0.0,
            f: 0.0,
        }
    }

    fn skew_x(degrees: f64) -> Self {
        Self {
            c: scalar::tan(degrees.to_radians()),
            ..Self::IDENTITY
        }
    }

    fn skew_y(degrees: f64) -> Self {
        Self {
            b: scalar::tan(degrees.to_radians()),
            ..Self::IDENTITY
        }
    }

    /// `self ∘ next`: apply `next` first, then `self`.
    fn then(self, next: Self) -> Self {
        Self {
            a: self.a * next.a + self.c * next.b,
            b: self.b * next.a + self.d * next.b,
            c: self.a * next.c + self.c * next.d,
            d: self.b * next.c + self.d * next.d,
            e: self.a * next.e + self.c * next.f + self.e,
            f: self.b * next.e + self.d * next.f + self.f,
        }
    }

    fn apply(self, x: f64, y: f64) -> (f64, f64) {
        (
            self.a * x + self.c * y + self.e,
            self.b * x + self.d * y + self.f,
        )
    }

    fn determinant(self) -> f64 {
        self.a * self.d - self.b * self.c
    }

    fn is_finite(self) -> bool {
        [self.a, self.b, self.c, self.d, self.e, self.f]
            .iter()
            .all(|v| v.is_finite())
    }
}

// -------------------------------------------------------------- tokenizer

/// XML name characters: the ASCII subset of the XML Name production. This
/// covers the element/attribute/id names SVGs in the wild use; anything
/// else is malformed (a documented subset choice).
fn is_name_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_' || b == b':'
}

fn is_name_char(b: u8) -> bool {
    is_name_start(b) || b.is_ascii_digit() || b == b'-' || b == b'.'
}

/// The local part of a possibly prefixed name (`xlink:href` → `href`).
fn local_name(name: &str) -> &str {
    match name.rfind(':') {
        Some(i) => &name[i + 1..],
        None => name,
    }
}

/// A markup token. Text between elements is skipped by the tokenizer
/// (documented: it renders nothing in the accepted subset).
#[derive(Debug)]
enum Lexeme {
    Start {
        name: String,
        attrs: Vec<(String, String)>,
        self_closing: bool,
        line: usize,
    },
    End {
        name: String,
        line: usize,
    },
}

/// A byte-level XML tokenizer over the UTF-8-checked input. Line numbers
/// advance incrementally (counting `\n` in consumed spans), so tokenization
/// is O(n) even for adversarial inputs.
struct Lexer<'a> {
    text: &'a str,
    pos: usize,
    line: usize,
}

impl<'a> Lexer<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            text,
            pos: 0,
            line: 1,
        }
    }

    /// Advance to `to`, counting newlines in the consumed span.
    fn advance(&mut self, to: usize) {
        debug_assert!(to >= self.pos);
        self.line += self.text.as_bytes()[self.pos..to]
            .iter()
            .filter(|&&b| b == b'\n')
            .count();
        self.pos = to;
    }

    /// Decode the predefined + numeric entities in an attribute value.
    /// Output size is bounded by the input: every entity decodes to at most
    /// one scalar value (≤ 4 bytes) from ≥ 3 source bytes.
    fn decode_entities(raw: &str, line: usize) -> Result<String, SvgError> {
        if !raw.contains('&') {
            return Ok(raw.to_owned());
        }
        let mut out = String::with_capacity(raw.len());
        let mut rest = raw;
        while let Some(amp) = rest.find('&') {
            out.push_str(&rest[..amp]);
            let after = &rest[amp + 1..];
            let semi = after.find(';').ok_or_else(|| {
                malformed(line, "unterminated entity reference in attribute value")
            })?;
            let body = &after[..semi];
            if body.is_empty()
                || body.len() > 32
                || !body.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'#')
            {
                return Err(malformed(
                    line,
                    "invalid entity reference in attribute value",
                ));
            }
            match body {
                "amp" => out.push('&'),
                "lt" => out.push('<'),
                "gt" => out.push('>'),
                "quot" => out.push('"'),
                "apos" => out.push('\''),
                _ if body.starts_with("#x") || body.starts_with("#X") => {
                    let digits = &body[2..];
                    let value = u32::from_str_radix(digits, 16)
                        .map_err(|_| malformed(line, "malformed numeric character reference"))?;
                    push_char(&mut out, value, line)?;
                }
                _ if body.starts_with('#') => {
                    let digits = &body[1..];
                    let value = digits
                        .parse::<u32>()
                        .map_err(|_| malformed(line, "malformed numeric character reference"))?;
                    push_char(&mut out, value, line)?;
                }
                _ => {
                    return Err(SvgError::UnknownEntity {
                        line,
                        name: body.to_owned(),
                    });
                }
            }
            rest = &after[semi + 1..];
        }
        out.push_str(rest);
        Ok(out)
    }

    fn next_lexeme(&mut self) -> Result<Option<Lexeme>, SvgError> {
        loop {
            let lt = match self.text[self.pos..].find('<') {
                Some(off) => self.pos + off,
                None => {
                    self.advance(self.text.len());
                    return Ok(None);
                }
            };
            self.advance(lt);
            let start_line = self.line;
            let rest = &self.text[self.pos..];
            let rbytes = rest.as_bytes();
            if rest.starts_with("<!--") {
                return match rest.find("-->") {
                    Some(end) => {
                        self.advance(self.pos + end + 3);
                        continue;
                    }
                    None => Err(malformed(start_line, "unterminated comment")),
                };
            }
            if rest.starts_with("<![CDATA[") {
                return match rest.find("]]>") {
                    Some(end) => {
                        self.advance(self.pos + end + 3);
                        continue;
                    }
                    None => Err(malformed(start_line, "unterminated CDATA section")),
                };
            }
            if rbytes.get(1) == Some(&b'!') {
                if rest.len() >= 9 && rest[..9].eq_ignore_ascii_case("<!doctype") {
                    return Err(SvgError::DoctypeRefused { line: start_line });
                }
                return Err(malformed(
                    start_line,
                    "unsupported markup declaration (only elements, comments, \
                     CDATA, and processing instructions are accepted)",
                ));
            }
            if rest.starts_with("<?") {
                return match rest.find("?>") {
                    Some(end) => {
                        self.advance(self.pos + end + 2);
                        continue;
                    }
                    None => Err(malformed(start_line, "unterminated processing instruction")),
                };
            }
            if rest.starts_with("</") {
                let mut i = 2;
                let name_start = i;
                while i < rbytes.len() && is_name_char(rbytes[i]) {
                    i += 1;
                }
                if i == name_start {
                    return Err(malformed(start_line, "end tag with no name"));
                }
                let name = rest[name_start..i].to_owned();
                while i < rbytes.len() && rbytes[i].is_ascii_whitespace() {
                    i += 1;
                }
                if rbytes.get(i) != Some(&b'>') {
                    return Err(malformed(start_line, "malformed end tag"));
                }
                self.advance(self.pos + i + 1);
                return Ok(Some(Lexeme::End {
                    name,
                    line: start_line,
                }));
            }
            // Start tag.
            if rbytes.get(1).is_none_or(|b| !is_name_start(*b)) {
                return Err(malformed(start_line, "malformed start tag"));
            }
            let mut i = 1;
            let name_start = i;
            while i < rbytes.len() && is_name_char(rbytes[i]) {
                i += 1;
            }
            let name = rest[name_start..i].to_owned();
            let mut attrs: Vec<(String, String)> = Vec::new();
            let self_closing = loop {
                while i < rbytes.len() && rbytes[i].is_ascii_whitespace() {
                    i += 1;
                }
                match rbytes.get(i) {
                    None => return Err(malformed(start_line, "unterminated start tag")),
                    Some(b'>') => {
                        i += 1;
                        break false;
                    }
                    Some(b'/') => {
                        if rbytes.get(i + 1) != Some(&b'>') {
                            return Err(malformed(start_line, "malformed self-closing tag"));
                        }
                        i += 2;
                        break true;
                    }
                    Some(&b) if is_name_start(b) => {
                        let attr_start = i;
                        while i < rbytes.len() && is_name_char(rbytes[i]) {
                            i += 1;
                        }
                        let attr_name = &rest[attr_start..i];
                        while i < rbytes.len() && rbytes[i].is_ascii_whitespace() {
                            i += 1;
                        }
                        if rbytes.get(i) != Some(&b'=') {
                            return Err(malformed(
                                start_line,
                                format!("attribute `{attr_name}` has no `=`"),
                            ));
                        }
                        i += 1;
                        while i < rbytes.len() && rbytes[i].is_ascii_whitespace() {
                            i += 1;
                        }
                        let quote = match rbytes.get(i) {
                            Some(&q @ (b'"' | b'\'')) => q,
                            _ => {
                                return Err(malformed(
                                    start_line,
                                    format!("attribute `{attr_name}` value must be quoted"),
                                ));
                            }
                        };
                        i += 1;
                        let value_start = i;
                        while i < rbytes.len() && rbytes[i] != quote {
                            i += 1;
                        }
                        if i >= rbytes.len() {
                            return Err(malformed(start_line, "unterminated attribute value"));
                        }
                        if attrs.iter().any(|(n, _)| n == attr_name) {
                            return Err(malformed(
                                start_line,
                                format!("duplicate attribute `{attr_name}`"),
                            ));
                        }
                        let value = Self::decode_entities(&rest[value_start..i], start_line)?;
                        attrs.push((attr_name.to_owned(), value));
                        i += 1;
                    }
                    Some(_) => {
                        return Err(malformed(start_line, "malformed attribute in start tag"));
                    }
                }
            };
            self.advance(self.pos + i);
            let attrs = attrs
                .into_iter()
                .filter(|(n, _)| n != "xmlns" && !n.starts_with("xmlns:"))
                .collect();
            return Ok(Some(Lexeme::Start {
                name,
                attrs,
                self_closing,
                line: start_line,
            }));
        }
    }
}

/// Push a numeric character reference, rejecting non-scalars and the C0
/// controls XML forbids (tab/LF/CR excepted).
fn push_char(out: &mut String, value: u32, line: usize) -> Result<(), SvgError> {
    let ch = char::from_u32(value)
        .ok_or_else(|| malformed(line, "numeric character reference is not a Unicode scalar"))?;
    if (ch as u32) < 0x20 && ch != '\t' && ch != '\n' && ch != '\r' {
        return Err(malformed(line, "control character in character reference"));
    }
    out.push(ch);
    Ok(())
}

// ---------------------------------------------------------------- the tree

/// A parsed XML element. Names keep any namespace prefix; every consumer
/// matches on [`local_name`].
#[derive(Debug)]
struct Element {
    name: String,
    attrs: Vec<(String, String)>,
    children: Vec<Element>,
    line: usize,
}

/// The attribute with this local name, if present (first wins).
fn attr<'a>(el: &'a Element, local: &str) -> Option<&'a str> {
    el.attrs
        .iter()
        .find(|(n, _)| local_name(n) == local)
        .map(|(_, v)| v.as_str())
}

/// Build the element tree under the depth and element budgets.
fn build_tree(text: &str, limits: &SvgLimits) -> Result<Element, SvgError> {
    let mut lexer = Lexer::new(text);
    let mut stack: Vec<Element> = Vec::new();
    let mut root: Option<Element> = None;
    let mut elements = 0usize;
    while let Some(lexeme) = lexer.next_lexeme()? {
        match lexeme {
            Lexeme::Start {
                name,
                attrs,
                self_closing,
                line,
            } => {
                elements += 1;
                if elements > limits.max_elements {
                    return Err(SvgError::TooManyElements {
                        limit: limits.max_elements,
                    });
                }
                let el = Element {
                    name,
                    attrs,
                    children: Vec::new(),
                    line,
                };
                if self_closing {
                    match stack.last_mut() {
                        Some(parent) => parent.children.push(el),
                        None => {
                            if root.replace(el).is_some() {
                                return Err(malformed(line, "multiple root elements"));
                            }
                        }
                    }
                } else {
                    let depth = stack.len() + 1;
                    if depth > limits.max_depth {
                        return Err(SvgError::TooDeep {
                            depth,
                            limit: limits.max_depth,
                        });
                    }
                    stack.push(el);
                }
            }
            Lexeme::End { name, line } => {
                let el = stack.pop().ok_or_else(|| {
                    malformed(line, format!("end tag `</{name}>` has no open element"))
                })?;
                if el.name != name {
                    return Err(malformed(
                        line,
                        format!("end tag `</{name}>` does not match `<{}>`", el.name),
                    ));
                }
                match stack.last_mut() {
                    Some(parent) => parent.children.push(el),
                    None => {
                        if root.replace(el).is_some() {
                            return Err(malformed(line, "multiple root elements"));
                        }
                    }
                }
            }
        }
    }
    if let Some(open) = stack.last() {
        return Err(malformed(
            open.line,
            format!("unclosed element `<{}>`", open.name),
        ));
    }
    root.ok_or_else(|| malformed(1, "empty document: no root element"))
}

// ------------------------------------------------------------ value parses

/// A resolved length: user units (unit-less and absolute units already
/// converted to px at 96 dpi) or a fraction of a context reference.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Length {
    User(f64),
    Percent(f64),
}

/// The viewport percentages resolve against (per SVG §7.10).
#[derive(Debug, Clone, Copy)]
struct Viewport {
    width: f64,
    height: f64,
}

impl Viewport {
    fn x(self, l: Length) -> f64 {
        match l {
            Length::User(n) => n,
            Length::Percent(p) => p * self.width,
        }
    }

    fn y(self, l: Length) -> f64 {
        match l {
            Length::User(n) => n,
            Length::Percent(p) => p * self.height,
        }
    }

    fn other(self, l: Length) -> f64 {
        match l {
            Length::User(n) => n,
            Length::Percent(p) => p * self.diagonal(),
        }
    }

    /// The normalized diagonal: `sqrt(w² + h²) / sqrt(2)`.
    fn diagonal(self) -> f64 {
        ((self.width * self.width + self.height * self.height) / 2.0).sqrt()
    }
}

/// Scan the longest valid SVG-number prefix; returns its end offset.
fn number_prefix_end(text: &str) -> usize {
    let b = text.as_bytes();
    let mut i = 0;
    if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
        i += 1;
    }
    let mut digits = 0;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
        digits += 1;
    }
    if i < b.len() && b[i] == b'.' {
        i += 1;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
            digits += 1;
        }
    }
    if digits == 0 {
        return 0;
    }
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        let mut j = i + 1;
        if j < b.len() && (b[j] == b'+' || b[j] == b'-') {
            j += 1;
        }
        let exp_start = j;
        while j < b.len() && b[j].is_ascii_digit() {
            j += 1;
        }
        if j > exp_start {
            i = j;
        }
    }
    i
}

/// Parse a finite f64, refusing `nan`/`inf`/overflow by name.
fn parse_number(text: &str, line: usize, context: &'static str) -> Result<f64, SvgError> {
    let t = text.trim();
    let n: f64 = t
        .parse()
        .map_err(|_| malformed(line, format!("{context} {t:?} is not a number")))?;
    if !n.is_finite() {
        return Err(SvgError::NonFinite { line, context });
    }
    Ok(n)
}

/// Parse a length: number plus optional unit (`px pt pc mm cm in %`).
fn parse_length(text: &str, line: usize, context: &'static str) -> Result<Length, SvgError> {
    let t = text.trim();
    let end = number_prefix_end(t);
    if end == 0 {
        // `nan`/`inf` spell no numeric prefix but parse as f64: refuse
        // them by name rather than as a generic malformation.
        if t.parse::<f64>().is_ok() {
            return Err(SvgError::NonFinite { line, context });
        }
        return Err(malformed(line, format!("{context} {t:?} is not a length")));
    }
    let n = parse_number(&t[..end], line, context)?;
    match t[end..].trim() {
        "" | "px" => Ok(Length::User(n)),
        "pt" => Ok(Length::User(n * (96.0 / 72.0))),
        "pc" => Ok(Length::User(n * 16.0)),
        "mm" => Ok(Length::User(n * (96.0 / 25.4))),
        "cm" => Ok(Length::User(n * (96.0 / 2.54))),
        "in" => Ok(Length::User(n * 96.0)),
        "%" => Ok(Length::Percent(n / 100.0)),
        unit @ ("em" | "ex") => Err(malformed(
            line,
            format!("{context}: font-relative unit `{unit}` is outside the supported subset"),
        )),
        unit => Err(malformed(line, format!("{context}: unknown unit `{unit}`"))),
    }
}

/// Parse a clamped [0, 1] opacity (a bare number or a percentage).
fn parse_opacity(text: &str, line: usize, context: &'static str) -> Result<f64, SvgError> {
    let t = text.trim();
    let v = if let Some(pct) = t.strip_suffix('%') {
        parse_number(pct, line, context)? / 100.0
    } else {
        parse_number(t, line, context)?
    };
    Ok(v.clamp(0.0, 1.0))
}

/// Parse a whitespace/comma-separated list of finite numbers.
fn parse_number_list(text: &str, line: usize, context: &'static str) -> Result<Vec<f64>, SvgError> {
    let mut out = Vec::new();
    for piece in text.split(|c: char| c.is_ascii_whitespace() || c == ',') {
        if piece.is_empty() {
            continue;
        }
        out.push(parse_number(piece, line, context)?);
    }
    Ok(out)
}

// ------------------------------------------------------------------ colors

/// Parse the body of `rgb(...)`/`rgba(...)`: comma- or space-separated,
/// components as 0–255 integers or percentages, optional alpha.
fn parse_rgb_function(inner: &str, line: usize) -> Result<(Srgb, f64), SvgError> {
    let parts: Vec<&str> = if inner.contains(',') {
        inner.split(',').map(str::trim).collect()
    } else {
        inner.split_ascii_whitespace().collect()
    };
    if parts.len() != 3 && parts.len() != 4 {
        return Err(malformed(
            line,
            format!("rgb() takes 3 or 4 components, got {}", parts.len()),
        ));
    }
    let component = |piece: &str| -> Result<f64, SvgError> {
        if let Some(pct) = piece.strip_suffix('%') {
            Ok(parse_number(pct, line, "rgb() component")? / 100.0)
        } else {
            Ok(parse_number(piece, line, "rgb() component")? / 255.0)
        }
    };
    let r = component(parts[0])?.clamp(0.0, 1.0);
    let g = component(parts[1])?.clamp(0.0, 1.0);
    let b = component(parts[2])?.clamp(0.0, 1.0);
    let a = if parts.len() == 4 {
        parse_opacity(parts[3], line, "rgb() alpha")?
    } else {
        1.0
    };
    Ok((Srgb { r, g, b }, a))
}

/// The CSS/SVG named colors (the full keyword table; sRGB hex values).
fn named_color(name: &str) -> Option<Srgb> {
    let hex: u32 = match name {
        "aliceblue" => 0xf0f8ff,
        "antiquewhite" => 0xfaebd7,
        "aqua" => 0x00ffff,
        "aquamarine" => 0x7fffd4,
        "azure" => 0xf0ffff,
        "beige" => 0xf5f5dc,
        "bisque" => 0xffe4c4,
        "black" => 0x000000,
        "blanchedalmond" => 0xffebcd,
        "blue" => 0x0000ff,
        "blueviolet" => 0x8a2be2,
        "brown" => 0xa52a2a,
        "burlywood" => 0xdeb887,
        "cadetblue" => 0x5f9ea0,
        "chartreuse" => 0x7fff00,
        "chocolate" => 0xd2691e,
        "coral" => 0xff7f50,
        "cornflowerblue" => 0x6495ed,
        "cornsilk" => 0xfff8dc,
        "crimson" => 0xdc143c,
        "cyan" => 0x00ffff,
        "darkblue" => 0x00008b,
        "darkcyan" => 0x008b8b,
        "darkgoldenrod" => 0xb8860b,
        "darkgray" | "darkgrey" => 0xa9a9a9,
        "darkgreen" => 0x006400,
        "darkkhaki" => 0xbdb76b,
        "darkmagenta" => 0x8b008b,
        "darkolivegreen" => 0x556b2f,
        "darkorange" => 0xff8c00,
        "darkorchid" => 0x9932cc,
        "darkred" => 0x8b0000,
        "darksalmon" => 0xe9967a,
        "darkseagreen" => 0x8fbc8f,
        "darkslateblue" => 0x483d8b,
        "darkslategray" | "darkslategrey" => 0x2f4f4f,
        "darkturquoise" => 0x00ced1,
        "darkviolet" => 0x9400d3,
        "deeppink" => 0xff1493,
        "deepskyblue" => 0x00bfff,
        "dimgray" | "dimgrey" => 0x696969,
        "dodgerblue" => 0x1e90ff,
        "firebrick" => 0xb22222,
        "floralwhite" => 0xfffaf0,
        "forestgreen" => 0x228b22,
        "fuchsia" => 0xff00ff,
        "gainsboro" => 0xdcdcdc,
        "ghostwhite" => 0xf8f8ff,
        "gold" => 0xffd700,
        "goldenrod" => 0xdaa520,
        "gray" | "grey" => 0x808080,
        "green" => 0x008000,
        "greenyellow" => 0xadff2f,
        "honeydew" => 0xf0fff0,
        "hotpink" => 0xff69b4,
        "indianred" => 0xcd5c5c,
        "indigo" => 0x4b0082,
        "ivory" => 0xfffff0,
        "khaki" => 0xf0e68c,
        "lavender" => 0xe6e6fa,
        "lavenderblush" => 0xfff0f5,
        "lawngreen" => 0x7cfc00,
        "lemonchiffon" => 0xfffacd,
        "lightblue" => 0xadd8e6,
        "lightcoral" => 0xf08080,
        "lightcyan" => 0xe0ffff,
        "lightgoldenrodyellow" => 0xfafad2,
        "lightgray" | "lightgrey" => 0xd3d3d3,
        "lightgreen" => 0x90ee90,
        "lightpink" => 0xffb6c1,
        "lightsalmon" => 0xffa07a,
        "lightseagreen" => 0x20b2aa,
        "lightskyblue" => 0x87cefa,
        "lightslategray" | "lightslategrey" => 0x778899,
        "lightsteelblue" => 0xb0c4de,
        "lightyellow" => 0xffffe0,
        "lime" => 0x00ff00,
        "limegreen" => 0x32cd32,
        "linen" => 0xfaf0e6,
        "magenta" => 0xff00ff,
        "maroon" => 0x800000,
        "mediumaquamarine" => 0x66cdaa,
        "mediumblue" => 0x0000cd,
        "mediumorchid" => 0xba55d3,
        "mediumpurple" => 0x9370db,
        "mediumseagreen" => 0x3cb371,
        "mediumslateblue" => 0x7b68ee,
        "mediumspringgreen" => 0x00fa9a,
        "mediumturquoise" => 0x48d1cc,
        "mediumvioletred" => 0xc71585,
        "midnightblue" => 0x191970,
        "mintcream" => 0xf5fffa,
        "mistyrose" => 0xffe4e1,
        "moccasin" => 0xffe4b5,
        "navajowhite" => 0xffdead,
        "navy" => 0x000080,
        "oldlace" => 0xfdf5e6,
        "olive" => 0x808000,
        "olivedrab" => 0x6b8e23,
        "orange" => 0xffa500,
        "orangered" => 0xff4500,
        "orchid" => 0xda70d6,
        "palegoldenrod" => 0xeee8aa,
        "palegreen" => 0x98fb98,
        "paleturquoise" => 0xafeeee,
        "palevioletred" => 0xdb7093,
        "papayawhip" => 0xffefd5,
        "peachpuff" => 0xffdab9,
        "peru" => 0xcd853f,
        "pink" => 0xffc0cb,
        "plum" => 0xdda0dd,
        "powderblue" => 0xb0e0e6,
        "purple" => 0x800080,
        "rebeccapurple" => 0x663399,
        "red" => 0xff0000,
        "rosybrown" => 0xbc8f8f,
        "royalblue" => 0x4169e1,
        "saddlebrown" => 0x8b4513,
        "salmon" => 0xfa8072,
        "sandybrown" => 0xf4a460,
        "seagreen" => 0x2e8b57,
        "seashell" => 0xfff5ee,
        "sienna" => 0xa0522d,
        "silver" => 0xc0c0c0,
        "skyblue" => 0x87ceeb,
        "slateblue" => 0x6a5acd,
        "slategray" | "slategrey" => 0x708090,
        "snow" => 0xfffafa,
        "springgreen" => 0x00ff7f,
        "steelblue" => 0x4682b4,
        "tan" => 0xd2b48c,
        "teal" => 0x008080,
        "thistle" => 0xd8bfd8,
        "tomato" => 0xff6347,
        "turquoise" => 0x40e0d0,
        "violet" => 0xee82ee,
        "wheat" => 0xf5deb3,
        "white" => 0xffffff,
        "whitesmoke" => 0xf5f5f5,
        "yellow" => 0xffff00,
        "yellowgreen" => 0x9acd32,
        _ => return None,
    };
    Some(Srgb::from_rgb8(
        (hex >> 16) as u8,
        (hex >> 8) as u8,
        hex as u8,
    ))
}

/// Parse a color value (not a paint: no `none`/`url()`): hex, `rgb[a()]`,
/// named colors, `currentColor`. Returns the color and an alpha multiplier.
fn parse_color(text: &str, line: usize) -> Result<Option<(Srgb, f64)>, SvgError> {
    let t = text.trim();
    if t.eq_ignore_ascii_case("inherit") {
        return Ok(None);
    }
    if let Some(hex) = t.strip_prefix('#') {
        let color = Srgb::from_hex(t)
            .map_err(|_| malformed(line, format!("malformed hex color {hex:?}")))?;
        return Ok(Some((color, 1.0)));
    }
    let lower = t.to_ascii_lowercase();
    let rest = lower
        .strip_prefix("rgb")
        .map(str::trim_start)
        .map(|r| r.strip_prefix('a').map_or(r, str::trim_start));
    if let Some(body) = rest
        .and_then(|r| r.strip_prefix('('))
        .and_then(|r| r.strip_suffix(')'))
    {
        let (color, alpha) = parse_rgb_function(body, line)?;
        return Ok(Some((color, alpha)));
    }
    if let Some(color) = named_color(&lower) {
        return Ok(Some((color, 1.0)));
    }
    Err(malformed(line, format!("unknown color {t:?}")))
}

// ------------------------------------------------------------------- style

/// A parsed `fill`/`stroke` value, before id resolution.
#[derive(Debug, Clone, PartialEq)]
enum PaintInput {
    /// SVG `none`.
    None,
    /// A flat color with an alpha multiplier (from `rgba()`/`transparent`).
    Color(Srgb, f64),
    /// `url(#id)` — resolved against the id map at emission time so a
    /// gradient or pattern is rejected by name, never silently dropped.
    Url(String),
    /// `currentColor` — resolved from the cascaded `color` at emission.
    CurrentColor,
}

/// The cascading style state. Fields not set on an element inherit from its
/// parent (SVG/CSS inheritance for the accepted subset).
#[derive(Debug, Clone)]
struct Style {
    fill: PaintInput,
    fill_opacity: f64,
    fill_rule: FillRule,
    stroke: PaintInput,
    stroke_width: f64,
    stroke_opacity: f64,
    line_cap: LineCap,
    line_join: LineJoin,
    miter_limit: f64,
    dash: Vec<f64>,
    dash_offset: f64,
    opacity: f64,
    color: Srgb,
    display_none: bool,
    visibility_hidden: bool,
}

const BLACK: Srgb = Srgb {
    r: 0.0,
    g: 0.0,
    b: 0.0,
};

impl Default for Style {
    /// The SVG initial values.
    fn default() -> Self {
        Self {
            fill: PaintInput::Color(BLACK, 1.0),
            fill_opacity: 1.0,
            fill_rule: FillRule::NonZero,
            stroke: PaintInput::None,
            stroke_width: 1.0,
            stroke_opacity: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            miter_limit: 4.0,
            dash: Vec::new(),
            dash_offset: 0.0,
            opacity: 1.0,
            color: BLACK,
            display_none: false,
            visibility_hidden: false,
        }
    }
}

/// The cascaded presentation properties (accepted subset, module docs).
const PRESENTATION_PROPERTIES: &[&str] = &[
    "fill",
    "fill-opacity",
    "fill-rule",
    "stroke",
    "stroke-width",
    "stroke-opacity",
    "stroke-linecap",
    "stroke-linejoin",
    "stroke-miterlimit",
    "stroke-dasharray",
    "stroke-dashoffset",
    "opacity",
    "color",
    "display",
    "visibility",
];

impl Style {
    /// Apply this element's presentation attributes, then its `style`
    /// attribute (which wins, per the CSS cascade).
    fn cascade(&mut self, el: &Element, vp: Viewport) -> Result<(), SvgError> {
        for (raw_name, value) in &el.attrs {
            let name = local_name(raw_name);
            if PRESENTATION_PROPERTIES.contains(&name) {
                self.apply(name, value, vp, el.line)?;
            }
        }
        if let Some(style_attr) = attr(el, "style") {
            for declaration in style_attr.split(';') {
                let declaration = declaration.trim();
                if declaration.is_empty() {
                    continue;
                }
                let (name, value) = declaration.split_once(':').ok_or_else(|| {
                    malformed(
                        el.line,
                        format!("style declaration {declaration:?} has no `:`"),
                    )
                })?;
                let name = name.trim();
                // Unknown properties configure features outside the accept
                // matrix (fonts, text layout, …) and are ignored by design.
                if PRESENTATION_PROPERTIES.contains(&name) {
                    self.apply(name, value, vp, el.line)?;
                }
            }
        }
        Ok(())
    }

    /// Apply one property/value pair.
    fn apply(
        &mut self,
        name: &str,
        value: &str,
        vp: Viewport,
        line: usize,
    ) -> Result<(), SvgError> {
        // `!important` is inert here (this is the only cascade origin).
        let value = value
            .trim()
            .strip_suffix("!important")
            .map_or(value.trim(), str::trim_end);
        match name {
            "fill" => {
                if let Some((paint, alpha)) = parse_paint(value, line)? {
                    self.fill = paint;
                    if alpha < 1.0 {
                        self.fill_opacity = alpha;
                    }
                }
            }
            "stroke" => {
                if let Some((paint, alpha)) = parse_paint(value, line)? {
                    self.stroke = paint;
                    if alpha < 1.0 {
                        self.stroke_opacity = alpha;
                    }
                }
            }
            "fill-opacity" => self.fill_opacity = parse_opacity(value, line, "fill-opacity")?,
            "stroke-opacity" => {
                self.stroke_opacity = parse_opacity(value, line, "stroke-opacity")?;
            }
            "opacity" => self.opacity *= parse_opacity(value, line, "opacity")?,
            "fill-rule" => {
                self.fill_rule = match value.trim() {
                    "nonzero" => FillRule::NonZero,
                    "evenodd" => FillRule::EvenOdd,
                    "inherit" => self.fill_rule,
                    other => {
                        return Err(malformed(line, format!("unknown fill-rule {other:?}")));
                    }
                };
            }
            "stroke-width" => {
                let w = vp.other(parse_length(value, line, "stroke-width")?);
                if w < 0.0 {
                    return Err(malformed(line, "stroke-width must be non-negative"));
                }
                self.stroke_width = w;
            }
            "stroke-linecap" => {
                self.line_cap = match value.trim() {
                    "butt" => LineCap::Butt,
                    "round" => LineCap::Round,
                    "square" => LineCap::Square,
                    "inherit" => self.line_cap,
                    other => {
                        return Err(malformed(line, format!("unknown stroke-linecap {other:?}")));
                    }
                };
            }
            "stroke-linejoin" => {
                self.line_join = match value.trim() {
                    "miter" | "miter-clip" => LineJoin::Miter,
                    "round" => LineJoin::Round,
                    "bevel" => LineJoin::Bevel,
                    "inherit" => self.line_join,
                    other => {
                        return Err(malformed(
                            line,
                            format!("unknown stroke-linejoin {other:?}"),
                        ));
                    }
                };
            }
            "stroke-miterlimit" => {
                let m = parse_number(value, line, "stroke-miterlimit")?;
                if m < 1.0 {
                    return Err(malformed(line, "stroke-miterlimit must be at least 1"));
                }
                self.miter_limit = m;
            }
            "stroke-dasharray" => {
                let v = value.trim();
                if v == "none" || v == "inherit" {
                    if v == "none" {
                        self.dash.clear();
                    }
                } else {
                    let mut dash = Vec::new();
                    for piece in v.split(|c: char| c.is_ascii_whitespace() || c == ',') {
                        if piece.is_empty() {
                            continue;
                        }
                        let d = vp.other(parse_length(piece, line, "stroke-dasharray")?);
                        if d < 0.0 {
                            return Err(malformed(
                                line,
                                "stroke-dasharray values must be non-negative",
                            ));
                        }
                        dash.push(d);
                    }
                    // Per spec, an all-zero dasharray renders solid.
                    if dash.iter().any(|&d| d > 0.0) {
                        self.dash = dash;
                    } else {
                        self.dash.clear();
                    }
                }
            }
            "stroke-dashoffset" => {
                self.dash_offset = vp.other(parse_length(value, line, "stroke-dashoffset")?);
            }
            "color" => {
                if let Some((color, _)) = parse_color(value, line)? {
                    self.color = color;
                }
            }
            "display" => {
                // `display:none` elides the subtree; it is never un-set by
                // a descendant (CSS: the subtree generates no boxes).
                if value.trim() == "none" {
                    self.display_none = true;
                }
            }
            "visibility" => match value.trim() {
                "hidden" | "collapse" => self.visibility_hidden = true,
                "visible" => self.visibility_hidden = false,
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }
}

/// Parse a `fill`/`stroke` value: `Ok(None)` = `inherit` (keep parent);
/// otherwise the paint plus an alpha multiplier.
fn parse_paint(text: &str, line: usize) -> Result<Option<(PaintInput, f64)>, SvgError> {
    let t = text.trim();
    if t.eq_ignore_ascii_case("inherit") {
        return Ok(None);
    }
    if t == "none" {
        return Ok(Some((PaintInput::None, 1.0)));
    }
    if t.eq_ignore_ascii_case("currentcolor") {
        return Ok(Some((PaintInput::CurrentColor, 1.0)));
    }
    if t.eq_ignore_ascii_case("transparent") {
        return Ok(Some((PaintInput::Color(BLACK, 0.0), 0.0)));
    }
    if let Some(rest) = t.strip_prefix("url(") {
        let inner = rest
            .strip_suffix(')')
            .ok_or_else(|| malformed(line, "malformed url() paint"))?;
        let inner = inner.trim().trim_matches('"').trim_matches('\'').trim();
        let id = inner
            .strip_prefix('#')
            .ok_or_else(|| SvgError::ExternalRef {
                line,
                reference: inner.to_owned(),
            })?;
        if id.is_empty() {
            return Err(malformed(line, "url() paint with an empty fragment"));
        }
        return Ok(Some((PaintInput::Url(id.to_owned()), 1.0)));
    }
    let (color, alpha) = parse_color(t, line)?
        .ok_or_else(|| malformed(line, "unreachable: inherit handled above"))?;
    Ok(Some((PaintInput::Color(color, alpha), alpha)))
}

// ------------------------------------------------------------- path data

/// A QuadPath under construction with the document's command budget.
struct PathSink<'a> {
    path: QuadPath,
    line: usize,
    commands: &'a mut usize,
    limit: usize,
}

impl PathSink<'_> {
    fn count(&mut self) -> Result<(), SvgError> {
        *self.commands += 1;
        if *self.commands > self.limit {
            return Err(SvgError::TooManyCommands { limit: self.limit });
        }
        Ok(())
    }

    fn move_to(&mut self, x: f64, y: f64) {
        self.path.start_new_path([x, y, 0.0]);
    }

    fn line_to(&mut self, x: f64, y: f64) -> Result<(), SvgError> {
        self.path.add_line_to([x, y, 0.0], false).map_err(geom)?;
        Ok(())
    }

    fn quad_to(&mut self, hx: f64, hy: f64, x: f64, y: f64) -> Result<(), SvgError> {
        self.path
            .add_quadratic_bezier_curve_to([hx, hy, 0.0], [x, y, 0.0], false)
            .map_err(geom)?;
        Ok(())
    }

    fn cubic_to(&mut self, h1: (f64, f64), h2: (f64, f64), to: (f64, f64)) -> Result<(), SvgError> {
        self.path
            .add_cubic_bezier_curve_to_with_tolerance(
                [h1.0, h1.1, 0.0],
                [h2.0, h2.1, 0.0],
                [to.0, to.1, 0.0],
                DEFAULT_SVG_TOLERANCE,
            )
            .map_err(geom)?;
        Ok(())
    }

    fn close(&mut self) -> Result<(), SvgError> {
        if !self.path.has_points() {
            return Err(malformed(self.line, "close command with no open subpath"));
        }
        self.path.close_path(false).map_err(geom)?;
        Ok(())
    }
}

/// A cursor over path-data bytes with the SVG number grammar.
struct PathCursor<'a> {
    bytes: &'a [u8],
    pos: usize,
    line: usize,
}

impl PathCursor<'_> {
    fn at_end(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    /// Separators: whitespace and commas, in any combination (a documented
    /// leniency over the strict one-comma grammar).
    fn skip_separators(&mut self) {
        while let Some(&b) = self.bytes.get(self.pos) {
            if b.is_ascii_whitespace() || b == b',' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    /// Parse one SVG number. `Ok(None)` when the next token is not a
    /// number (a command letter or the end).
    fn number(&mut self) -> Result<Option<f64>, SvgError> {
        self.skip_separators();
        let start = self.pos;
        let Some(&b) = self.bytes.get(self.pos) else {
            return Ok(None);
        };
        if !(b == b'+' || b == b'-' || b == b'.' || b.is_ascii_digit()) {
            return Ok(None);
        }
        let end = self.pos + number_prefix_end(self.text_tail());
        if end == self.pos {
            return Ok(None);
        }
        self.pos = end;
        let piece = std::str::from_utf8(&self.bytes[start..end])
            .map_err(|_| malformed(self.line, "invalid UTF-8 in path data"))?;
        let n: f64 = piece.parse().map_err(|_| {
            malformed(
                self.line,
                format!("path coordinate {piece:?} is not a number"),
            )
        })?;
        if !n.is_finite() {
            return Err(SvgError::NonFinite {
                line: self.line,
                context: "path data",
            });
        }
        Ok(Some(n))
    }

    fn text_tail(&self) -> &str {
        // Path data is an attribute value: already UTF-8.
        std::str::from_utf8(&self.bytes[self.pos..]).unwrap_or("")
    }

    fn need_number(&mut self) -> Result<f64, SvgError> {
        self.number()?
            .ok_or_else(|| malformed(self.line, "expected a coordinate in path data"))
    }

    fn pair(&mut self) -> Result<(f64, f64), SvgError> {
        let x = self.need_number()?;
        let y = self.need_number()?;
        Ok((x, y))
    }

    fn flag(&mut self) -> Result<bool, SvgError> {
        self.skip_separators();
        match self.peek() {
            Some(b'0') => {
                self.pos += 1;
                Ok(false)
            }
            Some(b'1') => {
                self.pos += 1;
                Ok(true)
            }
            Some(b) if b.is_ascii_digit() => Err(malformed(self.line, "arc flags must be 0 or 1")),
            _ => Err(malformed(self.line, "expected an arc flag in path data")),
        }
    }
}

/// The angle from vector `u` to vector `v`, signed, in (-π, π].
fn angle_between(u: (f64, f64), v: (f64, f64)) -> f64 {
    let cross = u.0 * v.1 - u.1 * v.0;
    let dot = u.0 * v.0 + u.1 * v.1;
    scalar::atan2(cross, dot)
}

/// An SVG elliptical arc, endpoint → center parameterization (SVG F.6.5),
/// then ≤ 90° cubic segments through the one error-bounded converter.
#[allow(clippy::too_many_arguments)]
fn arc_to(
    sink: &mut PathSink,
    from: (f64, f64),
    mut rx: f64,
    mut ry: f64,
    rotation_deg: f64,
    large_arc: bool,
    sweep: bool,
    to: (f64, f64),
) -> Result<(), SvgError> {
    // F.6.4: zero radii or a coincident endpoint degrade to a line/nothing.
    if from.0 == to.0 && from.1 == to.1 {
        return Ok(());
    }
    if rx == 0.0 || ry == 0.0 {
        return sink.line_to(to.0, to.1);
    }
    rx = rx.abs();
    ry = ry.abs();
    let rotation = rotation_deg.to_radians();
    let (sin_phi, cos_phi) = (scalar::sin(rotation), scalar::cos(rotation));
    let dx = (from.0 - to.0) / 2.0;
    let dy = (from.1 - to.1) / 2.0;
    // F.6.5.1: the primed (ellipse-aligned) coordinates of the start point.
    let x1p = cos_phi * dx + sin_phi * dy;
    let y1p = -sin_phi * dx + cos_phi * dy;
    // F.6.5.2: out-of-range radii are scaled up uniformly.
    let lambda = (x1p * x1p) / (rx * rx) + (y1p * y1p) / (ry * ry);
    if lambda > 1.0 {
        let scale = lambda.sqrt();
        rx *= scale;
        ry *= scale;
    }
    // F.6.5.3: the center in primed coordinates.
    let num = rx * rx * ry * ry - rx * rx * y1p * y1p - ry * ry * x1p * x1p;
    let den = rx * rx * y1p * y1p + ry * ry * x1p * x1p;
    if den == 0.0 {
        return sink.line_to(to.0, to.1);
    }
    let sign = if large_arc == sweep { -1.0 } else { 1.0 };
    let coef = sign * (num.max(0.0) / den).sqrt();
    let cxp = coef * (rx * y1p) / ry;
    let cyp = -coef * (ry * x1p) / rx;
    // F.6.5.4: the center in user coordinates.
    let cx = cos_phi * cxp - sin_phi * cyp + (from.0 + to.0) / 2.0;
    let cy = sin_phi * cxp + cos_phi * cyp + (from.1 + to.1) / 2.0;
    // F.6.5.5/6: the start angle and sweep.
    let u = ((x1p - cxp) / rx, (y1p - cyp) / ry);
    let v = ((-x1p - cxp) / rx, (-y1p - cyp) / ry);
    let theta1 = angle_between((1.0, 0.0), u);
    let mut delta = angle_between(u, v);
    if !sweep && delta > 0.0 {
        delta -= 2.0 * std::f64::consts::PI;
    } else if sweep && delta < 0.0 {
        delta += 2.0 * std::f64::consts::PI;
    }
    if ![cx, cy, theta1, delta].iter().all(|v| v.is_finite()) {
        return Err(SvgError::NonFinite {
            line: sink.line,
            context: "arc parameterization",
        });
    }
    // Split into ≤ 90° segments, one κ-cubic each, through the converter.
    let segments = (delta.abs() / std::f64::consts::FRAC_PI_2).ceil().max(1.0) as usize;
    let map = |u: f64, v: f64| -> (f64, f64) {
        (
            cx + rx * cos_phi * u - ry * sin_phi * v,
            cy + rx * sin_phi * u + ry * cos_phi * v,
        )
    };
    for i in 0..segments {
        let t0 = theta1 + delta * (i as f64 / segments as f64);
        let t1 = theta1 + delta * ((i + 1) as f64 / segments as f64);
        let k = 4.0 / 3.0 * scalar::tan((t1 - t0) / 4.0);
        let (s0, c0) = (scalar::sin(t0), scalar::cos(t0));
        let (s1, c1) = (scalar::sin(t1), scalar::cos(t1));
        let p0 = map(c0, s0);
        let p1 = map(c0 - k * s0, s0 + k * c0);
        let p2 = map(c1 + k * s1, s1 - k * c1);
        let p3 = map(c1, s1);
        let _ = p0; // the segment starts at the current point by construction
        sink.cubic_to(p1, p2, p3)?;
    }
    Ok(())
}

/// Parse path data (`d`) into a QuadPath in path-local coordinates.
fn parse_path_data(
    d: &str,
    line: usize,
    commands: &mut usize,
    limit: usize,
) -> Result<QuadPath, SvgError> {
    let mut sink = PathSink {
        path: QuadPath::new(),
        line,
        commands,
        limit,
    };
    let mut cursor = PathCursor {
        bytes: d.as_bytes(),
        pos: 0,
        line,
    };
    let mut cmd = 0u8;
    let mut prev = 0u8;
    let mut cur = (0.0, 0.0);
    let mut sub_start = (0.0, 0.0);
    let mut last_cubic_ctrl: Option<(f64, f64)> = None;
    let mut last_quad_ctrl: Option<(f64, f64)> = None;
    loop {
        cursor.skip_separators();
        if cursor.at_end() {
            break;
        }
        if let Some(b) = cursor.peek() {
            if b.is_ascii_alphabetic() {
                cmd = b;
                cursor.pos += 1;
            } else if cmd == 0 {
                return Err(malformed(line, "path data must begin with a command"));
            }
        }
        sink.count()?;
        let relative = cmd.is_ascii_lowercase();
        let abs = |v: (f64, f64)| -> (f64, f64) {
            if relative {
                (cur.0 + v.0, cur.1 + v.1)
            } else {
                v
            }
        };
        match cmd.to_ascii_uppercase() {
            b'M' => {
                let to = abs(cursor.pair()?);
                sink.move_to(to.0, to.1);
                cur = to;
                sub_start = to;
                last_cubic_ctrl = None;
                last_quad_ctrl = None;
            }
            b'L' => {
                let to = abs(cursor.pair()?);
                sink.line_to(to.0, to.1)?;
                cur = to;
                last_cubic_ctrl = None;
                last_quad_ctrl = None;
            }
            b'H' => {
                let x = cursor.need_number()?;
                let to = (if relative { cur.0 + x } else { x }, cur.1);
                sink.line_to(to.0, to.1)?;
                cur = to;
                last_cubic_ctrl = None;
                last_quad_ctrl = None;
            }
            b'V' => {
                let y = cursor.need_number()?;
                let to = (cur.0, if relative { cur.1 + y } else { y });
                sink.line_to(to.0, to.1)?;
                cur = to;
                last_cubic_ctrl = None;
                last_quad_ctrl = None;
            }
            b'C' => {
                let h1 = abs(cursor.pair()?);
                let h2 = abs(cursor.pair()?);
                let to = abs(cursor.pair()?);
                sink.cubic_to(h1, h2, to)?;
                last_cubic_ctrl = Some(h2);
                last_quad_ctrl = None;
                cur = to;
            }
            b'S' => {
                let h1 = if matches!(prev, b'C' | b'S') {
                    last_cubic_ctrl.map_or(cur, |h| (2.0 * cur.0 - h.0, 2.0 * cur.1 - h.1))
                } else {
                    cur
                };
                let h2 = abs(cursor.pair()?);
                let to = abs(cursor.pair()?);
                sink.cubic_to(h1, h2, to)?;
                last_cubic_ctrl = Some(h2);
                last_quad_ctrl = None;
                cur = to;
            }
            b'Q' => {
                let h = abs(cursor.pair()?);
                let to = abs(cursor.pair()?);
                sink.quad_to(h.0, h.1, to.0, to.1)?;
                last_quad_ctrl = Some(h);
                last_cubic_ctrl = None;
                cur = to;
            }
            b'T' => {
                let h = if matches!(prev, b'Q' | b'T') {
                    last_quad_ctrl.map_or(cur, |h| (2.0 * cur.0 - h.0, 2.0 * cur.1 - h.1))
                } else {
                    cur
                };
                let to = abs(cursor.pair()?);
                sink.quad_to(h.0, h.1, to.0, to.1)?;
                last_quad_ctrl = Some(h);
                last_cubic_ctrl = None;
                cur = to;
            }
            b'A' => {
                let rx = cursor.need_number()?;
                let ry = cursor.need_number()?;
                let rotation = cursor.need_number()?;
                let large_arc = cursor.flag()?;
                let sweep = cursor.flag()?;
                let to = abs(cursor.pair()?);
                arc_to(&mut sink, cur, rx, ry, rotation, large_arc, sweep, to)?;
                cur = to;
                last_cubic_ctrl = None;
                last_quad_ctrl = None;
            }
            b'Z' => {
                sink.close()?;
                cur = sub_start;
                last_cubic_ctrl = None;
                last_quad_ctrl = None;
            }
            _ => {
                return Err(malformed(
                    line,
                    format!("unknown path command `{}`", cmd as char),
                ));
            }
        }
        prev = cmd.to_ascii_uppercase();
        // After a move's first pair, subsequent implicit pairs are lines.
        if cmd == b'M' {
            cmd = b'L';
        } else if cmd == b'm' {
            cmd = b'l';
        }
    }
    Ok(sink.path)
}

// ------------------------------------------------------------ basic shapes

/// One κ-cubic quarter arc of an axis-aligned ellipse, appended to `sink`.
fn quarter_arc(
    path: &mut QuadPath,
    center: (f64, f64),
    rx: f64,
    ry: f64,
    from: (f64, f64),
    to: (f64, f64),
) -> Result<(), SvgError> {
    // Unit direction vectors of the start/end radii.
    let u = ((from.0 - center.0) / rx, (from.1 - center.1) / ry);
    let v = ((to.0 - center.0) / rx, (to.1 - center.1) / ry);
    let k = QUARTER_ARC_KAPPA;
    // Handles: k·(tangent at each endpoint); tangent = radius rotated ±90°.
    let cross = u.0 * v.1 - u.1 * v.0;
    let sign = if cross >= 0.0 { 1.0 } else { -1.0 };
    let h1 = (from.0 - sign * k * u.1 * rx, from.1 + sign * k * u.0 * ry);
    let h2 = (to.0 + sign * k * v.1 * rx, to.1 - sign * k * v.0 * ry);
    path.add_cubic_bezier_curve_to_with_tolerance(
        [h1.0, h1.1, 0.0],
        [h2.0, h2.1, 0.0],
        [to.0, to.1, 0.0],
        DEFAULT_SVG_TOLERANCE,
    )
    .map_err(geom)?;
    Ok(())
}

/// A full ellipse as four quarter arcs, starting at (cx + rx, cy).
fn ellipse_path(cx: f64, cy: f64, rx: f64, ry: f64) -> Result<QuadPath, SvgError> {
    let mut path = QuadPath::new();
    path.start_new_path([cx + rx, cy, 0.0]);
    let points = [
        (cx + rx, cy),
        (cx, cy + ry),
        (cx - rx, cy),
        (cx, cy - ry),
        (cx + rx, cy),
    ];
    for w in points.windows(2) {
        quarter_arc(&mut path, (cx, cy), rx, ry, w[0], w[1])?;
    }
    path.close_path(false).map_err(geom)?;
    Ok(path)
}

/// A rectangle, rounded when `rx`/`ry` are positive.
fn rect_path(x: f64, y: f64, w: f64, h: f64, rx: f64, ry: f64) -> Result<QuadPath, SvgError> {
    let mut path = QuadPath::new();
    if rx <= 0.0 || ry <= 0.0 {
        path.start_new_path([x, y, 0.0]);
        path.add_line_to([x + w, y, 0.0], false).map_err(geom)?;
        path.add_line_to([x + w, y + h, 0.0], false).map_err(geom)?;
        path.add_line_to([x, y + h, 0.0], false).map_err(geom)?;
        path.close_path(false).map_err(geom)?;
        return Ok(path);
    }
    let rx = rx.min(w / 2.0);
    let ry = ry.min(h / 2.0);
    path.start_new_path([x + rx, y, 0.0]);
    path.add_line_to([x + w - rx, y, 0.0], false)
        .map_err(geom)?;
    quarter_arc(
        &mut path,
        (x + w - rx, y + ry),
        rx,
        ry,
        (x + w - rx, y),
        (x + w, y + ry),
    )?;
    path.add_line_to([x + w, y + h - ry, 0.0], false)
        .map_err(geom)?;
    quarter_arc(
        &mut path,
        (x + w - rx, y + h - ry),
        rx,
        ry,
        (x + w, y + h - ry),
        (x + w - rx, y + h),
    )?;
    path.add_line_to([x + rx, y + h, 0.0], false)
        .map_err(geom)?;
    quarter_arc(
        &mut path,
        (x + rx, y + h - ry),
        rx,
        ry,
        (x + rx, y + h),
        (x, y + h - ry),
    )?;
    path.add_line_to([x, y + ry, 0.0], false).map_err(geom)?;
    quarter_arc(
        &mut path,
        (x + rx, y + ry),
        rx,
        ry,
        (x, y + ry),
        (x + rx, y),
    )?;
    path.close_path(false).map_err(geom)?;
    Ok(path)
}

// -------------------------------------------------------------- transforms

/// Parse a `transform` list, composed in document order.
fn parse_transform(text: &str, line: usize) -> Result<Affine, SvgError> {
    let bytes = text.as_bytes();
    let mut pos = 0;
    let mut result = Affine::IDENTITY;
    loop {
        while pos < bytes.len() && (bytes[pos].is_ascii_whitespace() || bytes[pos] == b',') {
            pos += 1;
        }
        if pos >= bytes.len() {
            break;
        }
        let name_start = pos;
        while pos < bytes.len() && bytes[pos].is_ascii_alphabetic() {
            pos += 1;
        }
        if pos == name_start {
            return Err(malformed(line, "expected a transform function name"));
        }
        let name = &text[name_start..pos];
        while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if bytes.get(pos) != Some(&b'(') {
            return Err(malformed(line, format!("transform `{name}` missing `(`")));
        }
        pos += 1;
        let close = text[pos..]
            .find(')')
            .ok_or_else(|| malformed(line, format!("transform `{name}` missing `)`")))?;
        let args = parse_number_list(&text[pos..pos + close], line, "transform")?;
        pos += close + 1;
        let step = match name {
            "matrix" => {
                if args.len() != 6 {
                    return Err(malformed(line, "matrix() takes 6 arguments"));
                }
                Affine {
                    a: args[0],
                    b: args[1],
                    c: args[2],
                    d: args[3],
                    e: args[4],
                    f: args[5],
                }
            }
            "translate" => {
                if args.len() > 2 || args.is_empty() {
                    return Err(malformed(line, "translate() takes 1 or 2 arguments"));
                }
                Affine::translate(args[0], args.get(1).copied().unwrap_or(0.0))
            }
            "scale" => {
                if args.len() > 2 || args.is_empty() {
                    return Err(malformed(line, "scale() takes 1 or 2 arguments"));
                }
                Affine::scale(args[0], args.get(1).copied().unwrap_or(args[0]))
            }
            "rotate" => match args.len() {
                1 => Affine::rotate(args[0]),
                3 => Affine::translate(args[1], args[2])
                    .then(Affine::rotate(args[0]))
                    .then(Affine::translate(-args[1], -args[2])),
                _ => return Err(malformed(line, "rotate() takes 1 or 3 arguments")),
            },
            "skewX" => {
                if args.len() != 1 {
                    return Err(malformed(line, "skewX() takes 1 argument"));
                }
                Affine::skew_x(args[0])
            }
            "skewY" => {
                if args.len() != 1 {
                    return Err(malformed(line, "skewY() takes 1 argument"));
                }
                Affine::skew_y(args[0])
            }
            other => {
                return Err(malformed(
                    line,
                    format!("unknown transform function `{other}`"),
                ));
            }
        };
        result = result.then(step);
        if !result.is_finite() {
            return Err(SvgError::NonFinite {
                line,
                context: "transform",
            });
        }
    }
    Ok(result)
}

// ------------------------------------------------------------ viewBox math

/// `preserveAspectRatio` alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParAlign {
    None,
    Min,
    Mid,
    Max,
}

/// Parse `preserveAspectRatio` into ((x-align, y-align), slice).
fn parse_par(
    text: Option<&str>,
    line: usize,
) -> Result<((ParAlign, ParAlign, bool), bool), SvgError> {
    let Some(text) = text else {
        return Ok(((ParAlign::Mid, ParAlign::Mid, true), false));
    };
    let mut tokens = text.split_ascii_whitespace();
    let mut first = tokens.next();
    if first == Some("defer") {
        first = tokens.next();
    }
    let align = match first {
        None => "xMidYMid",
        Some(a) => a,
    };
    let parsed = match align {
        "none" => (ParAlign::None, ParAlign::None, false),
        "xMinYMin" => (ParAlign::Min, ParAlign::Min, true),
        "xMidYMin" => (ParAlign::Mid, ParAlign::Min, true),
        "xMaxYMin" => (ParAlign::Max, ParAlign::Min, true),
        "xMinYMid" => (ParAlign::Min, ParAlign::Mid, true),
        "xMidYMid" => (ParAlign::Mid, ParAlign::Mid, true),
        "xMaxYMid" => (ParAlign::Max, ParAlign::Mid, true),
        "xMinYMax" => (ParAlign::Min, ParAlign::Max, true),
        "xMidYMax" => (ParAlign::Mid, ParAlign::Max, true),
        "xMaxYMax" => (ParAlign::Max, ParAlign::Max, true),
        other => {
            return Err(malformed(
                line,
                format!("unknown preserveAspectRatio align {other:?}"),
            ));
        }
    };
    let slice = match tokens.next() {
        None => false,
        Some("meet") => false,
        Some("slice") => true,
        Some(other) => {
            return Err(malformed(
                line,
                format!("unknown preserveAspectRatio value {other:?}"),
            ));
        }
    };
    Ok((parsed, slice))
}

/// The viewBox → viewport mapping under `preserveAspectRatio`.
fn viewbox_affine(
    view_box: Option<[f64; 4]>,
    vp: Viewport,
    par: Option<&str>,
    line: usize,
) -> Result<Affine, SvgError> {
    let Some(vb) = view_box else {
        return Ok(Affine::IDENTITY);
    };
    let ((ax, ay, uniform), slice) = parse_par(par, line)?;
    let sx = vp.width / vb[2];
    let sy = vp.height / vb[3];
    if !uniform {
        return Ok(Affine {
            a: sx,
            b: 0.0,
            c: 0.0,
            d: sy,
            e: -vb[0] * sx,
            f: -vb[1] * sy,
        });
    }
    let s = if slice { sx.max(sy) } else { sx.min(sy) };
    let extra_x = vp.width - vb[2] * s;
    let extra_y = vp.height - vb[3] * s;
    let tx = match ax {
        ParAlign::Min => 0.0,
        ParAlign::Mid => extra_x / 2.0,
        ParAlign::Max | ParAlign::None => extra_x,
    };
    let ty = match ay {
        ParAlign::Min => 0.0,
        ParAlign::Mid => extra_y / 2.0,
        ParAlign::Max | ParAlign::None => extra_y,
    };
    Ok(Affine {
        a: s,
        b: 0.0,
        c: 0.0,
        d: s,
        e: tx - vb[0] * s,
        f: ty - vb[1] * s,
    })
}

/// Parse a `viewBox` value: four numbers, width and height positive.
fn parse_viewbox(text: Option<&str>, line: usize) -> Result<Option<[f64; 4]>, SvgError> {
    let Some(text) = text else {
        return Ok(None);
    };
    let numbers = parse_number_list(text, line, "viewBox")?;
    if numbers.len() != 4 {
        return Err(malformed(line, "viewBox takes exactly 4 numbers"));
    }
    if numbers[2] <= 0.0 || numbers[3] <= 0.0 {
        return Err(malformed(line, "viewBox width and height must be positive"));
    }
    Ok(Some([numbers[0], numbers[1], numbers[2], numbers[3]]))
}

// ------------------------------------------------------------- interpreter

/// The document interpreter: resolves the tree into [`SvgShape`]s under the
/// declared budgets.
struct Interpreter<'a, 'tree> {
    limits: &'a SvgLimits,
    ids: HashMap<String, &'tree Element>,
    commands: usize,
    expansions: usize,
    use_stack: Vec<String>,
    shapes: Vec<SvgShape>,
}

impl<'a, 'tree> Interpreter<'a, 'tree> {
    fn new(limits: &'a SvgLimits, root: &'tree Element) -> Self {
        let mut ids = HashMap::new();
        collect_ids(root, &mut ids);
        Self {
            limits,
            ids,
            commands: 0,
            expansions: 0,
            use_stack: Vec::new(),
            shapes: Vec::new(),
        }
    }

    fn document(mut self, root: &'tree Element) -> Result<SvgDocument, SvgError> {
        let line = root.line;
        if local_name(&root.name) != "svg" {
            return Err(malformed(
                line,
                format!("root element is `<{}>`, not `<svg>`", root.name),
            ));
        }
        self.check_refusals(root)?;
        let view_box = parse_viewbox(attr(root, "viewBox"), line)?;
        let width = root_dimension(root, "width", view_box.map(|vb| vb[2]), line)?;
        let height = root_dimension(root, "height", view_box.map(|vb| vb[3]), line)?;
        if width <= 0.0 || height <= 0.0 {
            return Err(malformed(line, "svg viewport must have positive size"));
        }
        let vp = Viewport { width, height };
        let affine = viewbox_affine(view_box, vp, attr(root, "preserveAspectRatio"), line)?;
        let mut style = Style::default();
        style.cascade(root, vp)?;
        self.render_children(root, affine, &style, vp)?;
        Ok(SvgDocument {
            width,
            height,
            view_box,
            shapes: self.shapes,
        })
    }

    /// Render each child element in order.
    fn render_children(
        &mut self,
        el: &'tree Element,
        affine: Affine,
        style: &Style,
        vp: Viewport,
    ) -> Result<(), SvgError> {
        for child in &el.children {
            self.render_element(child, affine, style, vp)?;
        }
        Ok(())
    }

    /// The named-refusal checks that apply to every element, wherever it
    /// appears: remote references and the rejected attribute features.
    fn check_refusals(&self, el: &Element) -> Result<(), SvgError> {
        for (raw_name, value) in &el.attrs {
            let name = local_name(raw_name);
            match name {
                "href" => {
                    if !value.trim_start().starts_with('#') {
                        return Err(SvgError::ExternalRef {
                            line: el.line,
                            reference: value.clone(),
                        });
                    }
                }
                "clip-path" => {
                    if value.trim() != "none" {
                        return Err(SvgError::UnsupportedFeature {
                            line: el.line,
                            feature: "clip-path".to_owned(),
                        });
                    }
                }
                "mask" => {
                    if value.trim() != "none" {
                        return Err(SvgError::UnsupportedFeature {
                            line: el.line,
                            feature: "mask".to_owned(),
                        });
                    }
                }
                "filter" => {
                    return Err(SvgError::UnsupportedFeature {
                        line: el.line,
                        feature: "filter".to_owned(),
                    });
                }
                "marker" | "marker-start" | "marker-mid" | "marker-end"
                    if value.trim() != "none" =>
                {
                    return Err(SvgError::UnsupportedFeature {
                        line: el.line,
                        feature: "marker".to_owned(),
                    });
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn render_element(
        &mut self,
        el: &'tree Element,
        affine: Affine,
        parent: &Style,
        vp: Viewport,
    ) -> Result<(), SvgError> {
        let line = el.line;
        let name = local_name(el.name.as_str());
        self.check_refusals(el)?;
        let mut style = parent.clone();
        style.cascade(el, vp)?;
        if style.display_none || style.visibility_hidden {
            return Ok(());
        }
        // The element's own transform composes after its parent's.
        let element_transform = match attr(el, "transform") {
            Some(t) => parse_transform(t, line)?,
            None => Affine::IDENTITY,
        };
        let affine = affine.then(element_transform);
        match name {
            "g" | "a" | "symbol" => {
                self.render_children(el, affine, &style, vp)?;
            }
            "defs" | "title" | "desc" | "metadata" => {}
            "svg" => {
                let vb = parse_viewbox(attr(el, "viewBox"), line)?;
                let x = match attr(el, "x") {
                    Some(v) => vp.x(parse_length(v, line, "x")?),
                    None => 0.0,
                };
                let y = match attr(el, "y") {
                    Some(v) => vp.y(parse_length(v, line, "y")?),
                    None => 0.0,
                };
                let width = match attr(el, "width") {
                    Some(v) => vp.x(parse_length(v, line, "width")?),
                    None => vp.width,
                };
                let height = match attr(el, "height") {
                    Some(v) => vp.y(parse_length(v, line, "height")?),
                    None => vp.height,
                };
                if width > 0.0 && height > 0.0 {
                    let inner_vp = Viewport { width, height };
                    let map = Affine::translate(x, y).then(viewbox_affine(
                        vb,
                        inner_vp,
                        attr(el, "preserveAspectRatio"),
                        line,
                    )?);
                    // Nested overflow clipping is NOT applied (module docs).
                    self.render_children(el, affine.then(map), &style, inner_vp)?;
                }
            }
            "use" => self.render_use(el, affine, &style, vp)?,
            "path" => {
                let d = attr(el, "d").ok_or_else(|| malformed(line, "path element without `d`"))?;
                let path =
                    parse_path_data(d, line, &mut self.commands, self.limits.max_path_commands)?;
                self.emit(path, affine, &style, line)?;
                self.render_children(el, affine, &style, vp)?;
            }
            "rect" => {
                let x = opt_length(el, "x", vp, Viewport::x, line)?.unwrap_or(0.0);
                let y = opt_length(el, "y", vp, Viewport::y, line)?.unwrap_or(0.0);
                let w = opt_length(el, "width", vp, Viewport::x, line)?
                    .ok_or_else(|| malformed(line, "rect without `width`"))?;
                let h = opt_length(el, "height", vp, Viewport::y, line)?
                    .ok_or_else(|| malformed(line, "rect without `height`"))?;
                if w < 0.0 || h < 0.0 {
                    return Err(malformed(line, "rect width/height must be non-negative"));
                }
                if w > 0.0 && h > 0.0 {
                    let rx = opt_length(el, "rx", vp, Viewport::x, line)?;
                    let ry = opt_length(el, "ry", vp, Viewport::y, line)?;
                    if rx.is_some_and(|v| v < 0.0) || ry.is_some_and(|v| v < 0.0) {
                        return Err(malformed(line, "rect rx/ry must be non-negative"));
                    }
                    let (rx, ry) = match (rx, ry) {
                        (Some(a), None) => (a, a),
                        (None, Some(b)) => (b, b),
                        (a, b) => (a.unwrap_or(0.0), b.unwrap_or(0.0)),
                    };
                    self.emit(rect_path(x, y, w, h, rx, ry)?, affine, &style, line)?;
                }
                self.render_children(el, affine, &style, vp)?;
            }
            "circle" => {
                let cx = opt_length(el, "cx", vp, Viewport::x, line)?.unwrap_or(0.0);
                let cy = opt_length(el, "cy", vp, Viewport::y, line)?.unwrap_or(0.0);
                if let Some(r) = opt_length(el, "r", vp, Viewport::other, line)? {
                    if r < 0.0 {
                        return Err(malformed(line, "circle radius must be non-negative"));
                    }
                    if r > 0.0 {
                        self.emit(ellipse_path(cx, cy, r, r)?, affine, &style, line)?;
                    }
                }
                self.render_children(el, affine, &style, vp)?;
            }
            "ellipse" => {
                let cx = opt_length(el, "cx", vp, Viewport::x, line)?.unwrap_or(0.0);
                let cy = opt_length(el, "cy", vp, Viewport::y, line)?.unwrap_or(0.0);
                let rx = opt_length(el, "rx", vp, Viewport::x, line)?
                    .ok_or_else(|| malformed(line, "ellipse without `rx`"))?;
                let ry = opt_length(el, "ry", vp, Viewport::y, line)?
                    .ok_or_else(|| malformed(line, "ellipse without `ry`"))?;
                if rx < 0.0 || ry < 0.0 {
                    return Err(malformed(line, "ellipse radii must be non-negative"));
                }
                if rx > 0.0 && ry > 0.0 {
                    self.emit(ellipse_path(cx, cy, rx, ry)?, affine, &style, line)?;
                }
                self.render_children(el, affine, &style, vp)?;
            }
            "line" => {
                let x1 = opt_length(el, "x1", vp, Viewport::x, line)?.unwrap_or(0.0);
                let y1 = opt_length(el, "y1", vp, Viewport::y, line)?.unwrap_or(0.0);
                let x2 = opt_length(el, "x2", vp, Viewport::x, line)?.unwrap_or(0.0);
                let y2 = opt_length(el, "y2", vp, Viewport::y, line)?.unwrap_or(0.0);
                let mut path = QuadPath::new();
                path.start_new_path([x1, y1, 0.0]);
                path.add_line_to([x2, y2, 0.0], false).map_err(geom)?;
                self.emit(path, affine, &style, line)?;
                self.render_children(el, affine, &style, vp)?;
            }
            "polyline" | "polygon" => {
                let raw = attr(el, "points")
                    .ok_or_else(|| malformed(line, format!("{name} element without `points`")))?;
                let numbers = parse_number_list(raw, line, "points")?;
                if numbers.len() % 2 != 0 {
                    return Err(malformed(line, "points list has an odd number of values"));
                }
                if !numbers.is_empty() {
                    let mut path = QuadPath::new();
                    path.start_new_path([numbers[0], numbers[1], 0.0]);
                    let (pairs, _) = numbers[2..].as_chunks::<2>();
                    for pair in pairs {
                        path.add_line_to([pair[0], pair[1], 0.0], false)
                            .map_err(geom)?;
                    }
                    if name == "polygon" {
                        path.close_path(false).map_err(geom)?;
                    }
                    self.emit(path, affine, &style, line)?;
                }
                self.render_children(el, affine, &style, vp)?;
            }
            "clipPath" => return Err(unsupported(line, "clip-path")),
            "mask" => return Err(unsupported(line, "mask")),
            "linearGradient" | "radialGradient" | "meshgradient" => {
                return Err(unsupported(line, "gradient"));
            }
            "pattern" | "hatch" => return Err(unsupported(line, "pattern")),
            other => return Err(unsupported(line, other)),
        }
        Ok(())
    }

    /// `use`: expand the referenced element in place, under the expansion
    /// budget, with cycle detection by name.
    fn render_use(
        &mut self,
        el: &'tree Element,
        affine: Affine,
        style: &Style,
        vp: Viewport,
    ) -> Result<(), SvgError> {
        let line = el.line;
        let href = attr(el, "href").ok_or_else(|| malformed(line, "use element without `href`"))?;
        let href = href.trim();
        // check_refusals already established the fragment-only rule.
        let id = href
            .strip_prefix('#')
            .ok_or_else(|| malformed(line, "use href is not a fragment"))?;
        if id.is_empty() {
            return Err(malformed(line, "use href with an empty fragment"));
        }
        if self.use_stack.iter().any(|open| open == id) {
            return Err(SvgError::Cycle { id: id.to_owned() });
        }
        let target = self
            .ids
            .get(id)
            .copied()
            .ok_or_else(|| SvgError::MissingReference {
                line,
                id: id.to_owned(),
            })?;
        self.expansions += 1;
        if self.expansions > self.limits.max_use_expansions {
            return Err(SvgError::TooManyUseExpansions {
                limit: self.limits.max_use_expansions,
            });
        }
        let x = opt_length(el, "x", vp, Viewport::x, line)?.unwrap_or(0.0);
        let y = opt_length(el, "y", vp, Viewport::y, line)?.unwrap_or(0.0);
        let placed = affine.then(Affine::translate(x, y));
        self.use_stack.push(id.to_owned());
        let result = self.render_element(target, placed, style, vp);
        self.use_stack.pop();
        result
    }

    /// Apply the cascaded affine, resolve paints, and record the shape.
    fn emit(
        &mut self,
        path: QuadPath,
        affine: Affine,
        style: &Style,
        line: usize,
    ) -> Result<(), SvgError> {
        if !path.has_points() {
            return Ok(());
        }
        let mut points: Vec<Vec3> = Vec::with_capacity(path.num_points());
        for &p in path.points() {
            let (x, y) = affine.apply(p[0], p[1]);
            if !x.is_finite() || !y.is_finite() {
                return Err(SvgError::NonFinite {
                    line,
                    context: "transformed coordinate",
                });
            }
            points.push([x, y, 0.0]);
        }
        let mut path = path;
        path.set_points(points).map_err(geom)?;
        // Documented flattening: stroke geometry lengths scale by sqrt|det|.
        let det_sqrt = affine.determinant().abs().sqrt();
        let scale = if det_sqrt.is_finite() && det_sqrt > 0.0 {
            det_sqrt
        } else {
            1.0
        };
        let fill = self.resolve_paint(&style.fill, style.color, line)?;
        let stroke = self.resolve_paint(&style.stroke, style.color, line)?;
        let fill_opacity = match style.fill {
            PaintInput::Color(_, alpha) => style.fill_opacity.min(alpha),
            _ => style.fill_opacity,
        };
        let stroke_opacity = match style.stroke {
            PaintInput::Color(_, alpha) => style.stroke_opacity.min(alpha),
            _ => style.stroke_opacity,
        };
        self.shapes.push(SvgShape {
            path,
            style: SvgStyle {
                fill,
                fill_opacity,
                fill_rule: style.fill_rule,
                stroke,
                stroke_width: style.stroke_width * scale,
                stroke_opacity,
                line_cap: style.line_cap,
                line_join: style.line_join,
                miter_limit: style.miter_limit,
                stroke_dasharray: style.dash.iter().map(|d| d * scale).collect(),
                stroke_dashoffset: style.dash_offset * scale,
                opacity: style.opacity,
            },
        });
        Ok(())
    }

    /// Resolve a paint input to an output paint: `currentColor` and
    /// `url(#…)` paint servers (gradients/patterns rejected by name).
    fn resolve_paint(
        &self,
        input: &PaintInput,
        color: Srgb,
        line: usize,
    ) -> Result<Option<Paint>, SvgError> {
        match input {
            PaintInput::None => Ok(None),
            PaintInput::Color(c, _) => Ok(Some(Paint::Color(*c))),
            PaintInput::CurrentColor => Ok(Some(Paint::Color(color))),
            PaintInput::Url(id) => {
                let target = self.ids.get(id).ok_or_else(|| SvgError::MissingReference {
                    line,
                    id: id.clone(),
                })?;
                match local_name(target.name.as_str()) {
                    "linearGradient" | "radialGradient" | "meshgradient" => {
                        Err(unsupported(line, "gradient"))
                    }
                    "pattern" | "hatch" => Err(unsupported(line, "pattern")),
                    other => Err(malformed(
                        line,
                        format!("url(#{id}) names a `<{other}>`, which is not a paint server"),
                    )),
                }
            }
        }
    }
}

fn unsupported(line: usize, feature: &str) -> SvgError {
    SvgError::UnsupportedFeature {
        line,
        feature: feature.to_owned(),
    }
}

/// An optional length attribute, resolved against a viewport axis.
fn opt_length(
    el: &Element,
    name: &'static str,
    vp: Viewport,
    axis: fn(Viewport, Length) -> f64,
    line: usize,
) -> Result<Option<f64>, SvgError> {
    match attr(el, name) {
        Some(v) => Ok(Some(axis(vp, parse_length(v, line, name)?))),
        None => Ok(None),
    }
}

/// The root viewport dimension: the attribute (% resolves against the
/// viewBox dimension, the only reference a root viewport has), else the
/// viewBox dimension, else `default_size` (the 300/150 replaced-element
/// default).
fn root_dimension(
    root: &Element,
    name: &'static str,
    viewbox_dim: Option<f64>,
    line: usize,
) -> Result<f64, SvgError> {
    let default_size = if name == "width" { 300.0 } else { 150.0 };
    match attr(root, name) {
        Some(v) => match parse_length(v, line, name)? {
            Length::User(n) => Ok(n),
            Length::Percent(p) => viewbox_dim.map(|dim| p * dim).ok_or_else(|| {
                malformed(
                    line,
                    format!("percentage {name} without a viewBox to resolve against"),
                )
            }),
        },
        None => Ok(viewbox_dim.unwrap_or(default_size)),
    }
}

/// Index every element with an `id` (first occurrence wins — a documented
/// leniency for duplicate ids in the wild).
fn collect_ids<'tree>(el: &'tree Element, ids: &mut HashMap<String, &'tree Element>) {
    if let Some(id) = attr(el, "id") {
        ids.entry(id.to_owned()).or_insert(el);
    }
    for child in &el.children {
        collect_ids(child, ids);
    }
}

// ------------------------------------------------------------------ emit

/// Format one coordinate or scalar: Rust's shortest round-trip
/// representation, so re-parsing recovers the exact `f64` (SVG's exponent
/// notation covers the extreme magnitudes).
fn emit_number(value: f64) -> String {
    // Normalize -0.0 so emitted documents are stable byte-for-byte.
    let value = if value == 0.0 { 0.0 } else { value };
    format!("{value}")
}

/// `#rrggbb`, the hex form the importer's own color grammar accepts.
fn emit_paint(paint: &Paint) -> String {
    match paint {
        Paint::Color(color) => {
            let channel = |c: f64| ((c * 255.0).round().clamp(0.0, 255.0)) as u8;
            format!(
                "#{:02x}{:02x}{:02x}",
                channel(color.r),
                channel(color.g),
                channel(color.b)
            )
        }
    }
}

/// One subpath's `d` fragment: `M`, one `Q` per shared-anchor curve, and
/// `Z` when the subpath closes on its own start. The importer's
/// `close_path` is a no-op on an already-closed subpath, so `Z` re-imports
/// without adding a segment; an open subpath emits no `Z` and stays open.
///
/// # Panics
/// Never in practice: a shared-anchor slice has odd length by
/// construction, and the walk steps two from the first handle.
fn emit_subpath_data(subpath: &[Vec3]) -> String {
    let mut out = String::new();
    let first = subpath[0];
    out.push_str(&format!(
        "M {} {}",
        emit_number(first[0]),
        emit_number(first[1])
    ));
    let last_index = subpath.len() - 1;
    let mut index = 1;
    while index < last_index {
        let handle = subpath[index];
        let anchor = subpath[index + 1];
        out.push_str(&format!(
            " Q {} {} {} {}",
            emit_number(handle[0]),
            emit_number(handle[1]),
            emit_number(anchor[0]),
            emit_number(anchor[1])
        ));
        index += 2;
    }
    let last = subpath[last_index];
    let tolerance = 1e-9;
    let closed = (first[0] - last[0]).abs() <= tolerance && (first[1] - last[1]).abs() <= tolerance;
    if closed {
        out.push('Z');
    }
    out
}

/// The `d` attribute payload for one resolved shape path.
///
/// Subpath breaks (the shared-anchor null-curve encoding) are already
/// subpath boundaries, so each fragment is one `M`-rooted contour.
#[must_use]
pub fn emit_path_data(path: &QuadPath) -> String {
    let mut out = String::new();
    for subpath in path.subpaths() {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(&emit_subpath_data(subpath));
    }
    out
}

/// The presentation attributes for one resolved style, in a fixed order so
/// emitted documents are deterministic. Every attribute is written
/// explicitly — the round-trip never depends on the importer's defaults.
fn emit_style_attributes(style: &SvgStyle) -> String {
    let mut out = String::new();
    let mut attribute = |name: &str, value: String| {
        out.push_str(&format!(" {name}=\"{value}\""));
    };
    attribute(
        "fill",
        match &style.fill {
            None => "none".to_owned(),
            Some(paint) => emit_paint(paint),
        },
    );
    attribute("fill-opacity", emit_number(style.fill_opacity));
    attribute(
        "fill-rule",
        match style.fill_rule {
            FillRule::NonZero => "nonzero",
            FillRule::EvenOdd => "evenodd",
        }
        .to_owned(),
    );
    attribute(
        "stroke",
        match &style.stroke {
            None => "none".to_owned(),
            Some(paint) => emit_paint(paint),
        },
    );
    attribute("stroke-width", emit_number(style.stroke_width));
    attribute("stroke-opacity", emit_number(style.stroke_opacity));
    attribute("stroke-linecap", style.line_cap.to_string());
    attribute("stroke-linejoin", style.line_join.to_string());
    attribute("stroke-miterlimit", emit_number(style.miter_limit));
    if !style.stroke_dasharray.is_empty() {
        let values: Vec<String> = style
            .stroke_dasharray
            .iter()
            .map(|d| emit_number(*d))
            .collect();
        attribute("stroke-dasharray", values.join(" "));
        attribute("stroke-dashoffset", emit_number(style.stroke_dashoffset));
    }
    attribute("opacity", emit_number(style.opacity));
    out
}

/// Emit resolved shapes as an SVG document with the given viewport.
///
/// A `view_box` here is a caller decision about the emitted coordinate
/// system — [`crate::svg::emit_svg_document`] deliberately passes `None`,
/// because its shapes are already in final user space.
#[must_use]
pub fn emit_svg(
    shapes: &[SvgShape],
    width: f64,
    height: f64,
    view_box: Option<[f64; 4]>,
) -> String {
    let mut out = String::from("<svg xmlns=\"http://www.w3.org/2000/svg\"");
    out.push_str(&format!(
        " width=\"{}\" height=\"{}\"",
        emit_number(width),
        emit_number(height)
    ));
    if let Some([x, y, w, h]) = view_box {
        out.push_str(&format!(
            " viewBox=\"{} {} {} {}\"",
            emit_number(x),
            emit_number(y),
            emit_number(w),
            emit_number(h)
        ));
    }
    out.push_str(">\n");
    for shape in shapes {
        out.push_str(&format!(
            "<path d=\"{}\"{}/>\n",
            emit_path_data(&shape.path),
            emit_style_attributes(&shape.style)
        ));
    }
    out.push_str("</svg>\n");
    out
}

// ------------------------------------------------------------------ tests

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Result<SvgDocument, SvgError> {
        SvgDocument::parse(text.as_bytes())
    }

    /// Fail with a message. ubs bans `panic!`/`unreachable!` and clippy
    /// bans `assert!(false, …)`; a non-constant failing assertion is the
    /// compliant form.
    #[track_caller]
    fn fail(message: String) {
        assert!(message.is_empty(), "{message}");
    }

    // -------------------------------------------------------- unit: values

    #[test]
    fn length_units_resolve_at_96dpi() {
        let line = 1;
        assert_eq!(parse_length("10", line, "t"), Ok(Length::User(10.0)));
        assert_eq!(parse_length("10px", line, "t"), Ok(Length::User(10.0)));
        assert_eq!(parse_length("1in", line, "t"), Ok(Length::User(96.0)));
        assert_eq!(parse_length("72pt", line, "t"), Ok(Length::User(96.0)));
        assert_eq!(parse_length("6pc", line, "t"), Ok(Length::User(96.0)));
        assert_eq!(parse_length("2.54cm", line, "t"), Ok(Length::User(96.0)));
        assert_eq!(parse_length("25.4mm", line, "t"), Ok(Length::User(96.0)));
        assert_eq!(parse_length("50%", line, "t"), Ok(Length::Percent(0.5)));
        assert!(parse_length("1em", line, "t").is_err());
        assert!(parse_length("1q", line, "t").is_err());
        assert!(parse_length("abc", line, "t").is_err());
    }

    #[test]
    fn transform_list_composes_in_document_order() {
        // translate(10 10) scale(2): point (1,1) → scale first → (2,2) → +10.
        let m = parse_transform("translate(10 10) scale(2)", 1).expect("parses");
        assert_eq!(m.apply(1.0, 1.0), (12.0, 12.0));
        // translate(10) alone shifts x only.
        let m = parse_transform("translate(10)", 1).expect("parses");
        assert_eq!(m.apply(1.0, 1.0), (11.0, 1.0));
        // rotate(90 5 5) rotates about (5,5).
        let m = parse_transform("rotate(90 5 5)", 1).expect("parses");
        let (x, y) = m.apply(6.0, 5.0);
        assert!((x - 5.0).abs() < 1e-12 && (y - 6.0).abs() < 1e-12);
        let m = parse_transform("matrix(1 0 0 1 3 4)", 1).expect("parses");
        assert_eq!(m.apply(0.0, 0.0), (3.0, 4.0));
    }

    #[test]
    fn entities_decode_predefined_and_numeric_only() {
        assert_eq!(
            Lexer::decode_entities("a &amp; b &#65; &#x42; &lt;&gt;&quot;&apos;", 1)
                .expect("decodes"),
            "a & b A B <>\"'"
        );
        assert!(matches!(
            Lexer::decode_entities("&bomb;", 1),
            Err(SvgError::UnknownEntity { .. })
        ));
        assert!(Lexer::decode_entities("&amp", 1).is_err());
        assert!(Lexer::decode_entities("&#xD800;", 1).is_err());
    }

    #[test]
    fn viewbox_mapping_defaults_to_xmidymid_meet() {
        let vp = Viewport {
            width: 200.0,
            height: 100.0,
        };
        let m = viewbox_affine(Some([0.0, 0.0, 100.0, 100.0]), vp, None, 1).expect("maps");
        // s = min(2, 1) = 1, tx = (200-100)/2 = 50.
        assert_eq!(m.apply(0.0, 0.0), (50.0, 0.0));
        assert_eq!(m.apply(100.0, 100.0), (150.0, 100.0));
        // none: stretch.
        let m = viewbox_affine(Some([0.0, 0.0, 100.0, 100.0]), vp, Some("none"), 1).expect("maps");
        assert_eq!(m.apply(50.0, 50.0), (100.0, 50.0));
        // slice: s = max.
        let m = viewbox_affine(
            Some([0.0, 0.0, 100.0, 100.0]),
            vp,
            Some("xMinYMin slice"),
            1,
        )
        .expect("maps");
        assert_eq!(m.apply(0.0, 0.0), (0.0, 0.0));
        assert_eq!(m.apply(100.0, 100.0), (200.0, 200.0));
    }

    #[test]
    fn arc_reaches_its_endpoint() {
        let mut commands = 0usize;
        let path =
            parse_path_data("M0 0 A1 1 0 0 1 3 4", 1, &mut commands, 1 << 10).expect("arc parses");
        let last = path.points().last().copied().expect("nonempty");
        assert!((last[0] - 3.0).abs() < 1e-9 && (last[1] - 4.0).abs() < 1e-9);
        // The arc's midpoint lies on the unit circle x²+y² around its center.
        let mut commands = 0usize;
        let path =
            parse_path_data("M1 0 A1 1 0 0 1 0 1", 1, &mut commands, 1 << 10).expect("arc parses");
        let n = path.num_points() / 2;
        let mid = path.nth_curve_point(n / 2, 0.5).expect("point");
        let r = (mid[0] * mid[0] + mid[1] * mid[1]).sqrt();
        assert!((r - 1.0).abs() < 0.01, "midpoint radius {r}");
    }

    #[test]
    fn relative_and_implicit_commands_parse() {
        let mut commands = 0usize;
        let path =
            parse_path_data("m10 10 20 0 0 20 z", 1, &mut commands, 1 << 10).expect("parses");
        // m + 2 implicit l + z ⇒ the path closes at (10,10).
        assert!(path.is_closed());
        let first = path.points()[0];
        let last = *path.points().last().expect("nonempty");
        assert!((first[0] - last[0]).abs() < 1e-8 && (first[1] - last[1]).abs() < 1e-8);
    }

    #[test]
    fn smooth_curves_reflect_handles() {
        let mut commands = 0usize;
        // S after C reflects the second control: C handle2 (20,0) about
        // cur (30,0) → (40,0); the S segment must start horizontally.
        let path = parse_path_data(
            "M0 0 C10 0 20 0 30 0 S50 10 60 10",
            1,
            &mut commands,
            1 << 10,
        )
        .expect("parses");
        let last = path.points().last().copied().expect("nonempty");
        assert!((last[0] - 60.0).abs() < 1e-9 && (last[1] - 10.0).abs() < 1e-9);
    }

    // --------------------------------------------------- unit: refusal names

    #[test]
    fn doctype_is_refused_by_name() {
        let err = parse("<!DOCTYPE svg><svg/>").expect_err("refused");
        assert!(matches!(err, SvgError::DoctypeRefused { .. }));
        // Case-insensitive, and billion-laughs-style internal subsets are
        // refused before any entity machinery runs.
        let bomb = "<!doctype lolz [<!ENTITY lol \"lol\"><!ENTITY lol2 \"&lol;&lol;\">]>\
                    <svg><!-- nothing --></svg>";
        assert!(matches!(
            parse(bomb).expect_err("refused"),
            SvgError::DoctypeRefused { .. }
        ));
    }

    #[test]
    fn external_refs_are_refused_wherever_they_appear() {
        for doc in [
            "<svg><use href=\"http://evil.example/x.svg#y\"/></svg>",
            "<svg><use xlink:href=\"file:///etc/passwd\"/></svg>",
            "<svg><a href=\"https://example.com\"><rect width=\"1\" height=\"1\"/></a></svg>",
            "<svg><rect width=\"1\" height=\"1\" fill=\"url(http://evil.example/g.svg#g)\"/></svg>",
        ] {
            assert!(
                matches!(parse(doc), Err(SvgError::ExternalRef { .. })),
                "{doc} must refuse its external reference"
            );
        }
    }

    #[test]
    fn reject_matrix_names_the_four_headline_features() {
        let cases = [
            (
                "<svg><rect width=\"1\" height=\"1\" clip-path=\"url(#c)\"/></svg>",
                "clip-path",
            ),
            (
                "<svg><rect width=\"1\" height=\"1\" mask=\"url(#m)\"/></svg>",
                "mask",
            ),
            (
                "<svg><defs><linearGradient id=\"g\"/></defs><rect width=\"1\" height=\"1\" fill=\"url(#g)\"/></svg>",
                "gradient",
            ),
            (
                "<svg><defs><pattern id=\"p\"/></defs><rect width=\"1\" height=\"1\" fill=\"url(#p)\"/></svg>",
                "pattern",
            ),
            (
                "<svg><rect width=\"1\" height=\"1\" filter=\"url(#f)\"/></svg>",
                "filter",
            ),
            (
                "<svg><path d=\"M0 0L1 1\" marker-end=\"url(#m)\"/></svg>",
                "marker",
            ),
        ];
        for (doc, feature) in cases {
            match parse(doc) {
                Err(SvgError::UnsupportedFeature { feature: f, .. }) => {
                    assert_eq!(f, feature, "{doc}");
                }
                other => fail(format!(
                    "{doc}: expected UnsupportedFeature({feature}), got {other:?}"
                )),
            }
        }
        // The elements themselves are rejected when encountered.
        for (doc, feature) in [
            ("<svg><text>hi</text></svg>", "text"),
            ("<svg><image href=\"#x\"/></svg>", "image"),
            ("<svg><style>.a{fill:red}</style></svg>", "style"),
            ("<svg><script>alert(1)</script></svg>", "script"),
            ("<svg><foreignObject/></svg>", "foreignObject"),
            ("<svg><animate attributeName=\"x\"/></svg>", "animate"),
            ("<svg><madeup/></svg>", "madeup"),
        ] {
            match parse(doc) {
                Err(SvgError::UnsupportedFeature { feature: f, .. }) => {
                    assert_eq!(f, feature, "{doc}");
                }
                other => fail(format!(
                    "{doc}: expected UnsupportedFeature({feature}), got {other:?}"
                )),
            }
        }
    }

    #[test]
    fn budgets_refuse_by_name() {
        // Depth: 40 nested groups over the default 32.
        let mut deep = String::from("<svg>");
        for _ in 0..40 {
            deep.push_str("<g>");
        }
        assert!(matches!(
            parse(&deep).expect_err("too deep"),
            SvgError::TooDeep { .. }
        ));
        // Tight custom limits refuse fast.
        let limits = SvgLimits {
            max_bytes: 10,
            ..SvgLimits::default()
        };
        assert!(matches!(
            SvgDocument::parse_with_limits(b"<svg></svg>", &limits),
            Err(SvgError::TooLarge { .. })
        ));
        let limits = SvgLimits {
            max_path_commands: 3,
            ..SvgLimits::default()
        };
        assert!(matches!(
            SvgDocument::parse_with_limits(
                b"<svg><path d=\"M0 0L1 1L2 2L3 3L4 4\"/></svg>",
                &limits
            ),
            Err(SvgError::TooManyCommands { .. })
        ));
        let limits = SvgLimits {
            max_use_expansions: 1,
            ..SvgLimits::default()
        };
        let chained = "<svg><defs><g id=\"a\"><use href=\"#b\"/></g>\
                       <g id=\"b\"><use href=\"#c\"/></g><g id=\"c\"><rect width=\"1\" height=\"1\"/></g></defs>\
                       <use href=\"#a\"/></svg>";
        assert!(matches!(
            SvgDocument::parse_with_limits(chained.as_bytes(), &limits),
            Err(SvgError::TooManyUseExpansions { .. })
        ));
    }

    #[test]
    fn use_cycles_and_missing_refs_are_named() {
        let cyc = "<svg><defs><g id=\"a\"><use href=\"#a\"/></g></defs><use href=\"#a\"/></svg>";
        assert_eq!(
            parse(cyc).expect_err("cycle"),
            SvgError::Cycle { id: "a".to_owned() }
        );
        let indirect = "<svg><defs><g id=\"a\"><use href=\"#b\"/></g><g id=\"b\"><use href=\"#a\"/></g></defs><use href=\"#a\"/></svg>";
        assert!(matches!(
            parse(indirect).expect_err("cycle"),
            SvgError::Cycle { .. }
        ));
        let missing = "<svg><use href=\"#nope\"/></svg>";
        assert!(matches!(
            parse(missing).expect_err("missing"),
            SvgError::MissingReference { .. }
        ));
    }

    #[test]
    fn non_finite_values_are_refused_by_name() {
        for doc in [
            "<svg width=\"nan\"/>",
            "<svg><rect width=\"inf\" height=\"1\"/></svg>",
            "<svg><path d=\"M0 0 L1e999 0\"/></svg>",
            "<svg><circle cx=\"0\" cy=\"0\" r=\"1\" stroke-width=\"nan\"/></svg>",
        ] {
            assert!(
                matches!(parse(doc), Err(SvgError::NonFinite { .. })),
                "{doc} must refuse non-finite values"
            );
        }
    }

    #[test]
    fn malformed_xml_is_named_with_a_line() {
        match parse("<svg>\n<rect width=\"1\"></svg>").expect_err("mismatched") {
            SvgError::Malformed { line, .. } => assert_eq!(line, 2),
            other => fail(format!("expected Malformed at line 2, got {other:?}")),
        }
        assert!(matches!(parse(""), Err(SvgError::Malformed { .. })));
        assert!(matches!(parse("<html/>"), Err(SvgError::Malformed { .. })));
        assert!(matches!(
            parse("<svg><rect width=1/></svg>"),
            Err(SvgError::Malformed { .. })
        ));
    }

    // -------------------------------------------- in-module never-panics fuzz

    /// A tiny deterministic PRNG (xorshift64) for the in-module fuzz test.
    struct Rng(u64);

    impl Rng {
        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }

        fn below(&mut self, n: u64) -> u64 {
            self.next() % n
        }
    }

    /// Structure-aware never-panics fuzzing (2_000 cases, §16.5): mutate
    /// seed documents with grammar-biased and raw mutations, parse under
    /// tight budgets, and assert every accepted document is finite and
    /// every refusal is typed. Bounded by construction — the budgets refuse
    /// long before any allocation grows.
    #[test]
    fn never_panics_on_mutation_fuzz() {
        const TOKENS: &[&str] = &[
            "<svg>",
            "</svg>",
            "<g>",
            "</g>",
            "<defs>",
            "<use href=\"#a\"/>",
            "<rect width=\"1\" height=\"1\"/>",
            "<path d=\"M0 0L1 1\"/>",
            "<path d=\"M0 0 A1 1 0 0 1 2 2\"/>",
            "transform=\"rotate(45)\"",
            "fill=\"url(#g)\"",
            "fill=\"red\"",
            "style=\"fill:blue;opacity:0.5\"",
            "<!DOCTYPE svg>",
            "&amp;",
            "&bomb;",
            "xlink:href",
            "http://x",
            "M",
            "L",
            "A",
            "Z",
            "1e999",
            "nan",
            "-",
            ".",
            "999999",
            "<circle cx=\"1\" cy=\"2\" r=\"3\"/>",
            "viewBox=\"0 0 10 10\"",
            "preserveAspectRatio=\"none\"",
            "<ellipse rx=\"1\" ry=\"2\"/>",
        ];
        const SEEDS: &[&[u8]] = &[
            b"<svg width=\"10\" height=\"10\"><rect width=\"5\" height=\"5\"/></svg>",
            b"<svg viewBox=\"0 0 100 100\"><g transform=\"translate(10,20) scale(2)\">\
              <circle cx=\"50\" cy=\"50\" r=\"20\"/></g></svg>",
            b"<svg><defs><g id=\"a\"><rect width=\"1\" height=\"1\"/></g></defs>\
              <use href=\"#a\" x=\"5\"/></svg>",
            b"<svg><path d=\"M0 0 C1 2 3 4 5 6 S7 8 9 10 Q11 12 13 14 T15 16 \
              A1 1 0 0 1 20 20 Z\"/></svg>",
            b"<svg><rect width=\"1\" height=\"1\" style=\"fill:#ff0000;stroke:blue;\
              stroke-width:2;stroke-dasharray:3 1;fill-rule:evenodd\"/></svg>",
        ];
        let limits = SvgLimits {
            max_bytes: 1 << 16,
            max_depth: 12,
            max_path_commands: 1 << 10,
            max_use_expansions: 64,
            max_elements: 1 << 10,
        };
        let mut rng = Rng(0x666d_366e_6d5f_6675);
        for case in 0..2000u64 {
            let mut input = SEEDS[(case as usize) % SEEDS.len()].to_vec();
            let mutations = 1 + rng.below(8);
            for _ in 0..mutations {
                match rng.below(6) {
                    0 => {
                        // Grammar token splice.
                        if !input.is_empty() {
                            let piece = TOKENS[rng.below(TOKENS.len() as u64) as usize];
                            let at = rng.below(input.len() as u64 + 1) as usize;
                            let room = (limits.max_bytes).saturating_sub(input.len());
                            let take = piece.len().min(room);
                            input.splice(at..at, piece[..take].bytes());
                        }
                    }
                    1 => {
                        // Byte flip.
                        if !input.is_empty() {
                            let at = rng.below(input.len() as u64) as usize;
                            input[at] ^= 1 << (rng.below(8) as u32);
                        }
                    }
                    2 => {
                        // Truncate.
                        if !input.is_empty() {
                            input.truncate(rng.below(input.len() as u64 + 1) as usize);
                        }
                    }
                    3 => {
                        // Digit perturbation.
                        let digits: Vec<usize> = input
                            .iter()
                            .enumerate()
                            .filter(|(_, b)| b.is_ascii_digit())
                            .map(|(i, _)| i)
                            .collect();
                        if !digits.is_empty() {
                            let at = digits[rng.below(digits.len() as u64) as usize];
                            input[at] = b'0' + (rng.below(10) as u8);
                        }
                    }
                    4 => {
                        // Duplicate a chunk (budget-bomb steering).
                        if !input.is_empty() {
                            let start = rng.below(input.len() as u64) as usize;
                            let len = (rng.below(64) as usize).min(input.len() - start);
                            let chunk: Vec<u8> = input[start..start + len].to_vec();
                            let at = rng.below(input.len() as u64 + 1) as usize;
                            let room = (limits.max_bytes).saturating_sub(input.len());
                            input.splice(at..at, chunk.into_iter().take(room));
                        }
                    }
                    _ => {
                        // Overwrite with a random byte.
                        if !input.is_empty() {
                            let at = rng.below(input.len() as u64) as usize;
                            input[at] = rng.next() as u8;
                        }
                    }
                }
            }
            // The contract: Ok or a typed Err — never a panic, never a hang
            // (budgets bound the work by construction).
            if let Ok(doc) = SvgDocument::parse_with_limits(&input, &limits) {
                assert!(doc.width.is_finite() && doc.height.is_finite());
                for shape in &doc.shapes {
                    assert!(shape.style.opacity.is_finite());
                    assert!(shape.style.stroke_width.is_finite());
                    for p in shape.path.points() {
                        assert!(
                            p[0].is_finite() && p[1].is_finite() && p[2] == 0.0,
                            "case {case}: non-finite point {p:?}"
                        );
                    }
                }
            }
        }
    }
}
