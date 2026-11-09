//! The layout engine under proof: TeX's eight atom classes, the
//! inter-atom spacing table, display/text/script/scriptscript style
//! propagation, and the Appendix-G constructions (fractions, scripts,
//! radicals, big-operator limits, `\left…\right` delimiters, matrices)
//! over the synthesized CM metrics. All arithmetic in ems, y-up,
//! baseline at y = 0.

use core::ops::Range;

use fmd_font::Font;

use crate::metrics::{CM, MathMetrics};
use crate::output::{Face, Layout, PlacedGlyph, PlacedRule};
use crate::parse::{MathError, Node, parse};

/// The four TeX math styles. Cramped variants ride alongside.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    /// `\displaystyle`.
    Display,
    /// `\textstyle`.
    Text,
    /// `\scriptstyle`.
    Script,
    /// `\scriptscriptstyle`.
    ScriptScript,
}

impl Style {
    /// The glyph size factor (CM's 10 pt / 7 pt / 5 pt family).
    #[must_use]
    pub fn size_factor(self) -> f64 {
        match self {
            Self::Display | Self::Text => 1.0,
            Self::Script => 0.7,
            Self::ScriptScript => 0.5,
        }
    }

    fn script_style(self) -> Style {
        match self {
            Self::Display | Self::Text => Self::Script,
            Self::Script | Self::ScriptScript => Self::ScriptScript,
        }
    }

    fn frac_inner_style(self) -> Style {
        match self {
            Self::Display => Self::Text,
            Self::Text => Self::Script,
            Self::Script | Self::ScriptScript => Self::ScriptScript,
        }
    }

    fn is_script(self) -> bool {
        matches!(self, Self::Script | Self::ScriptScript)
    }
}

/// Style + crampedness, threaded through the tree.
#[derive(Debug, Clone, Copy)]
struct Ctx {
    style: Style,
    cramped: bool,
}

impl Ctx {
    fn sup(self) -> Ctx {
        Ctx {
            style: self.style.script_style(),
            cramped: self.cramped,
        }
    }
    fn sub(self) -> Ctx {
        Ctx {
            style: self.style.script_style(),
            cramped: true,
        }
    }
    fn num(self) -> Ctx {
        Ctx {
            style: self.style.frac_inner_style(),
            cramped: self.cramped,
        }
    }
    fn den(self) -> Ctx {
        Ctx {
            style: self.style.frac_inner_style(),
            cramped: true,
        }
    }
}

/// TeX's eight atom classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomClass {
    /// Ordinary.
    Ord,
    /// Large operator.
    Op,
    /// Binary operator.
    Bin,
    /// Relation.
    Rel,
    /// Opening.
    Open,
    /// Closing.
    Close,
    /// Punctuation.
    Punct,
    /// Inner (fractions, `\left…\right`).
    Inner,
}

/// Inter-atom spacing (TeXbook ch. 18): 0 none, 1 thin, 2 medium,
/// 3 thick; negative entries are suppressed in script styles.
fn pair_spacing(left: AtomClass, right: AtomClass, style: Style) -> f64 {
    use AtomClass::{Bin, Close, Inner, Op, Open, Ord, Punct, Rel};
    let idx = |c: AtomClass| match c {
        Ord => 0,
        Op => 1,
        Bin => 2,
        Rel => 3,
        Open => 4,
        Close => 5,
        Punct => 6,
        Inner => 7,
    };
    // Rows: left class; columns: right class. 9 = impossible (Bin has
    // already degraded to Ord in context), negatives = script-suppressed.
    const TABLE: [[i8; 8]; 8] = [
        [0, 1, -2, -3, 0, 0, 0, -1],     // Ord
        [1, 1, 9, -3, 0, 0, 0, -1],      // Op
        [-2, -2, 9, 9, -2, 9, 9, -2],    // Bin
        [-3, -3, 9, 0, -3, 0, 0, -3],    // Rel
        [0, 0, 9, 0, 0, 0, 0, 0],        // Open
        [0, 1, -2, -3, 0, 0, 0, -1],     // Close
        [-1, -1, 9, -1, -1, -1, -1, -1], // Punct
        [-1, 1, -2, -3, -1, 0, -1, -1],  // Inner
    ];
    let raw = TABLE[idx(left)][idx(right)];
    let kind = match raw {
        9 => 0, // unreachable after Bin degradation; harmless if hit
        v if v < 0 && style.is_script() => 0,
        v => v.abs(),
    };
    // thin = 3 mu, medium = 4 mu, thick = 5 mu; 1 mu = 1/18 em at size.
    let mu = match kind {
        1 => 3.0,
        2 => 4.0,
        3 => 5.0,
        _ => 0.0,
    };
    mu / 18.0 * style.size_factor()
}

/// A laid-out box: dimensions plus its content fragment, origin at the
/// box's left edge on the baseline.
#[derive(Debug, Clone)]
struct MBox {
    width: f64,
    height: f64,
    depth: f64,
    class: AtomClass,
    frag: Layout,
    /// True when the box is a single character glyph (Appendix-G's
    /// script-attachment distinction).
    is_char: bool,
}

impl MBox {
    fn empty(class: AtomClass) -> MBox {
        MBox {
            width: 0.0,
            height: 0.0,
            depth: 0.0,
            class,
            frag: Layout::default(),
            is_char: false,
        }
    }
}

/// The spike engine: the two bundled faces plus the metric family.
pub struct Engine {
    cm: Font,
    noto: Font,
    m: MathMetrics,
}

impl Engine {
    /// Parse the bundled faces.
    ///
    /// # Errors
    /// Propagates `fmd-font` parse failures (impossible for the shipped
    /// bytes; surfaced rather than unwrapped per the engine doctrine).
    pub fn new() -> Result<Engine, fmd_font::FontError> {
        Ok(Engine {
            cm: Font::parse(fmd_font::bundled::CM_REGULAR.to_vec())?,
            noto: Font::parse(fmd_font::bundled::NOTO_SANS_MATH_SYMBOLS.to_vec())?,
            m: CM,
        })
    }

    fn face(&self, face: Face) -> &Font {
        match face {
            Face::CmRegular => &self.cm,
            Face::NotoMath => &self.noto,
        }
    }

    /// Resolve a character to (face, gid) — CM first, Noto Math fallback.
    fn resolve(&self, ch: char, span: &Range<usize>) -> Result<(Face, u16), MathError> {
        let gid = self.cm.glyph_index(ch);
        if gid != 0 {
            return Ok((Face::CmRegular, gid));
        }
        let gid = self.noto.glyph_index(ch);
        if gid != 0 {
            return Ok((Face::NotoMath, gid));
        }
        Err(MathError::UnmappedChar {
            ch,
            span: span.clone(),
        })
    }

    /// Glyph box at `size` ems-per-em (the style factor), on the baseline.
    fn glyph_box(
        &self,
        ch: char,
        span: &Range<usize>,
        size: f64,
        class: AtomClass,
    ) -> Result<MBox, MathError> {
        let (face, gid) = self.resolve(ch, span)?;
        let font = self.face(face);
        let upm = f64::from(font.units_per_em);
        let outline = font
            .glyph_outline(gid)
            .map_err(|_| MathError::UnmappedChar {
                ch,
                span: span.clone(),
            })?;
        let (height, depth) = outline.bbox.map_or((0.0, 0.0), |b| {
            (
                f64::from(b[3]) / upm * size,
                (-f64::from(b[1]) / upm * size).max(0.0),
            )
        });
        let width = f64::from(outline.advance) / upm * size;
        let mut frag = Layout::default();
        frag.glyphs.push(PlacedGlyph {
            face,
            gid,
            ch,
            x: 0.0,
            y: 0.0,
            size,
            span: span.clone(),
        });
        Ok(MBox {
            width,
            height,
            depth,
            class,
            frag,
            is_char: true,
        })
    }

    fn char_class(ch: char) -> AtomClass {
        match ch {
            '+' | '−' | '-' | '*' | '±' => AtomClass::Bin,
            '=' | '<' | '>' | '→' | '≤' | '≥' => AtomClass::Rel,
            '(' | '[' => AtomClass::Open,
            ')' | ']' => AtomClass::Close,
            ',' | ';' | '!' | '?' => AtomClass::Punct,
            _ => AtomClass::Ord,
        }
    }

    /// Lay out one node in a style context.
    fn node(&self, node: &Node, ctx: Ctx) -> Result<MBox, MathError> {
        match node {
            Node::Char { ch, span } => {
                self.glyph_box(*ch, span, ctx.style.size_factor(), Self::char_class(*ch))
            }
            Node::List { items, .. } => self.hlist(items, ctx),
            Node::Frac { num, den, .. } => self.fraction(num, den, ctx),
            Node::Script { base, sub, sup, .. } => {
                self.scripts(base, sub.as_deref(), sup.as_deref(), ctx)
            }
            Node::Radical {
                index, radicand, ..
            } => self.radical(index.as_deref(), radicand, ctx),
            Node::LeftRight {
                open,
                close,
                body,
                span,
            } => self.left_right(*open, *close, body, span, ctx),
            Node::BigOp { ch, span } => self.big_op_bare(*ch, span, ctx),
            Node::Matrix { rows, .. } => self.matrix(rows, ctx),
        }
    }

    /// Horizontal list: Bin degradation, per-pair spacing, CM kerning.
    fn hlist(&self, items: &[Node], ctx: Ctx) -> Result<MBox, MathError> {
        // Big operators with attached scripts in display style become
        // limit stacks — handled in `scripts`. Everything else: lay out,
        // then run the Bin-degradation pass and assemble with spacing.
        let mut boxes: Vec<MBox> = Vec::with_capacity(items.len());
        for item in items {
            boxes.push(self.node(item, ctx)?);
        }
        // Bin → Ord when it cannot act as an infix operator.
        for i in 0..boxes.len() {
            if boxes[i].class != AtomClass::Bin {
                continue;
            }
            let prev_ok = i > 0
                && !matches!(
                    boxes[i - 1].class,
                    AtomClass::Bin
                        | AtomClass::Op
                        | AtomClass::Rel
                        | AtomClass::Open
                        | AtomClass::Punct
                );
            let next_ok = i + 1 < boxes.len()
                && !matches!(
                    boxes[i + 1].class,
                    AtomClass::Rel | AtomClass::Close | AtomClass::Punct
                );
            if !prev_ok || !next_ok {
                boxes[i].class = AtomClass::Ord;
            }
        }
        let single = boxes.len() == 1;
        let single_class = boxes.first().map_or(AtomClass::Ord, |b| b.class);
        let single_char = single && boxes[0].is_char;
        let mut out = MBox::empty(AtomClass::Ord);
        let mut cursor = 0.0f64;
        let mut prev: Option<(AtomClass, char, Face)> = None;
        for b in boxes {
            if let Some((pc, pch, pface)) = prev {
                cursor += pair_spacing(pc, b.class, ctx.style);
                // Font kerning between adjacent same-face character glyphs
                // (both Ord, no math spacing between them).
                if pc == AtomClass::Ord
                    && b.class == AtomClass::Ord
                    && b.is_char
                    && let Some(g) = b.frag.glyphs.first()
                    && g.face == pface
                {
                    let font = self.face(g.face);
                    let k = f64::from(font.kerning(pch, g.ch)) / f64::from(font.units_per_em)
                        * ctx.style.size_factor();
                    cursor += k;
                }
            }
            let ch = b.frag.glyphs.first().map_or('\0', |g| g.ch);
            let face = b.frag.glyphs.first().map_or(Face::CmRegular, |g| g.face);
            let is_char = b.is_char;
            out.merge_at(&b, cursor, 0.0);
            cursor += b.width;
            prev = if is_char {
                Some((b.class, ch, face))
            } else {
                Some((b.class, '\0', face))
            };
        }
        out.width = cursor;
        // A one-item list is transparent (keeps its class and charness);
        // compound lists read as Ord to their neighbors, as TeX boxes do.
        if single {
            out.class = single_class;
            out.is_char = single_char;
        }
        Ok(out)
    }

    /// Appendix-G rule 15 (the ruled fraction).
    fn fraction(&self, num: &Node, den: &Node, ctx: Ctx) -> Result<MBox, MathError> {
        let m = &self.m;
        let display = ctx.style == Style::Display;
        let numer = self.node(num, ctx.num())?;
        let denom = self.node(den, ctx.den())?;
        let theta = m.rule_thickness;
        let axis = m.axis_height;
        let mut u = if display { m.num1 } else { m.num2 };
        let mut v = if display { m.denom1 } else { m.denom2 };
        let phi = if display { 3.0 * theta } else { theta };
        // Clearance between numerator bottom and rule top…
        let num_gap = (u - numer.depth) - (axis + theta / 2.0);
        if num_gap < phi {
            u += phi - num_gap;
        }
        // …and rule bottom to denominator top.
        let den_gap = (axis - theta / 2.0) - (denom.height - v);
        if den_gap < phi {
            v += phi - den_gap;
        }
        let width = numer.width.max(denom.width);
        let mut out = MBox::empty(AtomClass::Inner);
        out.merge_at(&numer, (width - numer.width) / 2.0, u);
        out.merge_at(&denom, (width - denom.width) / 2.0, -v);
        out.frag.rules.push(PlacedRule {
            x: 0.0,
            y: axis - theta / 2.0,
            w: width,
            h: theta,
        });
        out.width = width;
        out.height = u + numer.height;
        out.depth = v + denom.depth;
        Ok(out)
    }

    /// Appendix-G rule 18 (scripts), plus rule 13/13a when the base is a
    /// big operator (limits in display style, side scripts otherwise).
    fn scripts(
        &self,
        base: &Node,
        sub: Option<&Node>,
        sup: Option<&Node>,
        ctx: Ctx,
    ) -> Result<MBox, MathError> {
        if let Node::BigOp { ch, span } = base
            && ctx.style == Style::Display
        {
            return self.big_op_limits(*ch, span, sub, sup, ctx);
        }
        let m = &self.m;
        let base_box = self.node(base, ctx)?;
        let sup_box = sup.map(|n| self.node(n, ctx.sup())).transpose()?;
        let sub_box = sub.map(|n| self.node(n, ctx.sub())).transpose()?;
        // Boxy bases drop scripts relative to their corners (18a).
        let (u, v) = if base_box.is_char {
            (0.0, 0.0)
        } else {
            let f = ctx.sup().style.size_factor();
            (
                base_box.height - m.sup_drop * f,
                base_box.depth + m.sub_drop * f,
            )
        };
        let mut out = MBox::empty(base_box.class);
        let base_w = base_box.width;
        out.merge_at(&base_box, 0.0, 0.0);
        let script_space = 0.05;
        let mut width = base_w;
        match (sup_box, sub_box) {
            (None, Some(sb)) => {
                // 18b: subscript alone.
                let s = v.max(m.sub1).max(sb.height - 0.8 * m.x_height);
                out.merge_at(&sb, base_w, -s);
                width = base_w + sb.width + script_space;
                out.height = out.height.max(sb.height - s);
                out.depth = out.depth.max(s + sb.depth);
            }
            (Some(sp), None) => {
                // 18c: superscript alone.
                let p0 = match (ctx.style, ctx.cramped) {
                    (Style::Display, false) => m.sup1,
                    (_, true) => m.sup3,
                    _ => m.sup2,
                };
                let p = u.max(p0).max(sp.depth + m.x_height / 4.0);
                out.merge_at(&sp, base_w, p);
                width = base_w + sp.width + script_space;
                out.height = out.height.max(p + sp.height);
                out.depth = out.depth.max(sp.depth - p);
            }
            (Some(sp), Some(sb)) => {
                // 18d–f: both, with the 4θ clearance and the ⅘x-height lift.
                let theta = m.rule_thickness;
                let p0 = match (ctx.style, ctx.cramped) {
                    (Style::Display, false) => m.sup1,
                    (_, true) => m.sup3,
                    _ => m.sup2,
                };
                let mut p = u.max(p0).max(sp.depth + m.x_height / 4.0);
                let mut s = v.max(m.sub2);
                let gap = (p - sp.depth) - (sb.height - s);
                if gap < 4.0 * theta {
                    s += 4.0 * theta - gap;
                    let lift = 0.8 * m.x_height - (p - sp.depth);
                    if lift > 0.0 {
                        p += lift;
                        s -= lift;
                    }
                }
                out.merge_at(&sp, base_w, p);
                out.merge_at(&sb, base_w, -s);
                width = base_w + sp.width.max(sb.width) + script_space;
                out.height = out.height.max(p + sp.height);
                out.depth = out.depth.max(s + sb.depth);
            }
            (None, None) => {}
        }
        out.width = width;
        Ok(out)
    }

    /// A big operator with no scripts: centered on the axis (rule 13).
    fn big_op_bare(&self, ch: char, span: &Range<usize>, ctx: Ctx) -> Result<MBox, MathError> {
        let display = ctx.style == Style::Display;
        // Display style wants the larger variant; with no size variants in
        // the bundled faces (see the ratification note) the spike scales
        // uniformly — fmd-math proper grows calibrated display glyphs.
        let scale = if display { 1.4 } else { 1.0 } * ctx.style.size_factor();
        let mut b = self.glyph_box(ch, span, scale, AtomClass::Op)?;
        let shift = self.m.axis_height - (b.height - b.depth) / 2.0;
        b.shift_baseline(shift);
        Ok(b)
    }

    /// Rule 13a: limits above and below in display style.
    fn big_op_limits(
        &self,
        ch: char,
        span: &Range<usize>,
        sub: Option<&Node>,
        sup: Option<&Node>,
        ctx: Ctx,
    ) -> Result<MBox, MathError> {
        let m = &self.m;
        let op = self.big_op_bare(ch, span, ctx)?;
        let sup_box = sup.map(|n| self.node(n, ctx.sup())).transpose()?;
        let sub_box = sub.map(|n| self.node(n, ctx.sub())).transpose()?;
        let width = op
            .width
            .max(sup_box.as_ref().map_or(0.0, |b| b.width))
            .max(sub_box.as_ref().map_or(0.0, |b| b.width));
        let mut out = MBox::empty(AtomClass::Op);
        out.merge_at(&op, (width - op.width) / 2.0, 0.0);
        out.width = width;
        out.height = op.height;
        out.depth = op.depth;
        if let Some(sp) = sup_box {
            let gap = (m.big_op_spacing3 - sp.depth).max(m.big_op_spacing1);
            let base_y = op.height + gap + sp.depth;
            out.merge_at(&sp, (width - sp.width) / 2.0, base_y);
            out.height = base_y + sp.height + m.big_op_spacing5;
        }
        if let Some(sb) = sub_box {
            let gap = (m.big_op_spacing4 - sb.height).max(m.big_op_spacing2);
            let base_y = -(op.depth + gap + sb.height);
            out.merge_at(&sb, (width - sb.width) / 2.0, base_y);
            out.depth = -(base_y - sb.depth) + m.big_op_spacing5;
        }
        Ok(out)
    }

    /// Rule 11: radicals, with the OQ-2 drawn-path mainline past natural size.
    fn radical(&self, index: Option<&Node>, radicand: &Node, ctx: Ctx) -> Result<MBox, MathError> {
        let m = &self.m;
        let rad = self.node(radicand, ctx)?;
        let theta = m.rule_thickness;
        let psi = if ctx.style == Style::Display {
            theta + m.x_height / 4.0
        } else {
            theta + theta / 4.0
        };
        let needed = rad.height + rad.depth + psi + theta;
        let mut out = MBox::empty(AtomClass::Ord);
        // The natural √ glyph covers ~1.03 em in CM Unicode (measured);
        // beyond ~1.25× natural the drawn-path construction takes over.
        let sqrt_span = "√";
        let natural = {
            let b = self.glyph_box(
                '√',
                &(0..sqrt_span.len()),
                ctx.style.size_factor(),
                AtomClass::Ord,
            )?;
            b.height + b.depth
        };
        let (sign_w, sign_top) = if needed <= natural * 1.25 {
            let scale = (needed / natural).max(1.0) * ctx.style.size_factor();
            let mut sign = self.glyph_box('√', &(0..sqrt_span.len()), scale, AtomClass::Ord)?;
            // Align the sign's top with the overbar top.
            let top = rad.height + psi + theta;
            sign.shift_baseline(top - sign.height);
            let w = sign.width;
            out.merge_at(&sign, 0.0, 0.0);
            (w, top)
        } else {
            // Drawn-path radical (the OQ-2 mainline past natural size):
            // the classic √ zigzag as one closed contour — leading tick,
            // heavy down-stroke to the vertex, light up-stroke to the
            // overbar corner — stroke weights matching CM's authored sign.
            use fmd_font::outline::{Contour, Point, Segment};
            let top = rad.height + psi + theta;
            let bot = -rad.depth - 0.06;
            let w = 0.58;
            let heavy = 0.058; // down-stroke weight (CM-calibrated)
            let light = 1.4 * theta; // up-stroke weight
            let p = |x: f64, y: f64| Point { x, y };
            let vertex_x = 0.30;
            let mid_y = bot + (top - bot) * 0.42;
            let start = p(0.02, mid_y);
            let contour = Contour {
                start,
                segments: vec![
                    Segment::Line {
                        to: p(0.13, mid_y + 0.05),
                    },
                    Segment::Line {
                        to: p(vertex_x - heavy * 0.6, bot + 0.18),
                    },
                    Segment::Line {
                        to: p(w - light, top),
                    },
                    Segment::Line { to: p(w, top) },
                    Segment::Line {
                        to: p(vertex_x, bot),
                    },
                    Segment::Line {
                        to: p(vertex_x - heavy, bot),
                    },
                    Segment::Line {
                        to: p(0.10, mid_y - 0.02),
                    },
                    Segment::Line { to: start },
                ],
            };
            out.frag.paths.push(crate::output::PlacedPath {
                contours: vec![contour],
                x: 0.0,
                y: 0.0,
                span: 0..0,
            });
            (w, top)
        };
        out.merge_at(&rad, sign_w, 0.0);
        out.frag.rules.push(PlacedRule {
            x: sign_w,
            y: sign_top - theta,
            w: rad.width,
            h: theta,
        });
        out.width = sign_w + rad.width;
        out.height = sign_top;
        out.depth = rad.depth;
        // The index sits raised beside the sign, in scriptscript style.
        if let Some(ix) = index {
            let ib = self.node(
                ix,
                Ctx {
                    style: Style::ScriptScript,
                    cramped: ctx.cramped,
                },
            )?;
            let raise = 0.6 * (out.height + out.depth) - out.depth;
            let mut shifted = MBox::empty(AtomClass::Ord);
            shifted.merge_at(&ib, 0.0, raise);
            shifted.width = (ib.width - 0.2).max(0.0); // tuck under the hook
            shifted.height = raise + ib.height;
            shifted.depth = out.depth;
            let dx = shifted.width;
            let mut merged = MBox::empty(AtomClass::Ord);
            merged.merge_at(&shifted, 0.0, 0.0);
            merged.merge_at(&out, dx, 0.0);
            merged.width = dx + out.width;
            merged.height = out.height.max(shifted.height);
            merged.depth = out.depth;
            return Ok(merged);
        }
        Ok(out)
    }

    /// Rule 19: `\left…\right` — natural glyph, scaled glyph, or the
    /// drawn-path construction (the OQ-2 mainline), by required size.
    fn left_right(
        &self,
        open: char,
        close: char,
        body: &Node,
        span: &Range<usize>,
        ctx: Ctx,
    ) -> Result<MBox, MathError> {
        let m = &self.m;
        let inner = self.node(body, ctx)?;
        let delta = (inner.height - m.axis_height).max(inner.depth + m.axis_height);
        // TeX: cover ≥ delta·2·(delimiterfactor/1000) with factor 901.
        let target = (2.0 * delta * 0.901).max(2.0 * delta - 0.5);
        let open_box = self.delimiter(open, target, span, ctx)?;
        let close_box = self.delimiter(close, target, span, ctx)?;
        let mut out = MBox::empty(AtomClass::Inner);
        let mut x = 0.0;
        out.merge_at(&open_box, x, 0.0);
        x += open_box.width;
        out.merge_at(&inner, x, 0.0);
        x += inner.width;
        out.merge_at(&close_box, x, 0.0);
        x += close_box.width;
        out.width = x;
        out.height = inner.height.max(open_box.height).max(close_box.height);
        out.depth = inner.depth.max(open_box.depth).max(close_box.depth);
        Ok(out)
    }

    /// One delimiter sized to cover `target` ems, centered on the axis.
    fn delimiter(
        &self,
        ch: char,
        target: f64,
        span: &Range<usize>,
        ctx: Ctx,
    ) -> Result<MBox, MathError> {
        let natural = self.glyph_box(ch, span, ctx.style.size_factor(), AtomClass::Open)?;
        let nat_cover = natural.height + natural.depth;
        let class = if matches!(ch, ')' | ']' | '}' | '⟩') {
            AtomClass::Close
        } else {
            AtomClass::Open
        };
        let mut b = if target <= nat_cover {
            natural
        } else if target <= nat_cover * 1.25 {
            // Modest uniform scale keeps the authored stroke plausible.
            self.glyph_box(
                ch,
                span,
                ctx.style.size_factor() * target / nat_cover,
                class,
            )?
        } else {
            // The OQ-2 mainline: parametric drawn-path construction (the
            // bundled CM Unicode has no extension pieces to assemble).
            self.drawn_paren(ch, target, span)?
        };
        b.class = class;
        // Center the delimiter on the axis.
        let shift = self.m.axis_height - (b.height - b.depth) / 2.0;
        b.shift_baseline(shift);
        Ok(b)
    }

    /// A parametric drawn paren as one closed quadratic contour, stroke
    /// weight calibrated to CM's authored paren so the seam at the size
    /// threshold is invisible at a glance.
    fn drawn_paren(&self, ch: char, target: f64, span: &Range<usize>) -> Result<MBox, MathError> {
        use fmd_font::outline::{Contour, Point, Segment};
        if !matches!(ch, '(' | ')') {
            // Spike scope: parens prove the mechanism; other delimiters
            // remain named unsupported (the ratchet's shape).
            return Err(MathError::UnsupportedCommand {
                name: format!("drawn delimiter {ch:?}"),
                span: span.clone(),
            });
        }
        let h = target; // total covered size, centered later
        let w = 0.30 + 0.06 * h; // gentle widening with size
        let t_top = 0.035; // tip thickness
        let t_mid = 0.062 + 0.01 * h; // waist thickness, CM-calibrated
        let (x_out, x_in, bulge) = (0.0, t_mid, 0.16 * w);
        let p = |x: f64, y: f64| Point { x, y };
        let mirror = ch == ')';
        let mx = |x: f64| if mirror { w - x } else { x };
        let start = p(mx(w), h);
        let contour = Contour {
            start,
            segments: vec![
                Segment::Quad {
                    ctrl: p(mx(x_out - bulge), h / 2.0),
                    to: p(mx(w), 0.0),
                },
                Segment::Line {
                    to: p(mx(w - t_top), 0.0),
                },
                Segment::Quad {
                    ctrl: p(mx(x_in - bulge), h / 2.0),
                    to: p(mx(w - t_top), h),
                },
                Segment::Line { to: start },
            ],
        };
        let mut frag = Layout::default();
        frag.paths.push(crate::output::PlacedPath {
            contours: vec![contour],
            x: 0.0,
            y: 0.0,
            span: span.clone(),
        });
        Ok(MBox {
            width: w + 0.04,
            height: h,
            depth: 0.0,
            class: AtomClass::Open,
            frag,
            is_char: false,
        })
    }

    /// A small `matrix` environment: text-style cells, centered columns,
    /// the whole box centered on the axis (`\vcenter` semantics).
    fn matrix(&self, rows: &[Vec<Node>], ctx: Ctx) -> Result<MBox, MathError> {
        let cell_ctx = Ctx {
            style: if ctx.style == Style::Display {
                Style::Text
            } else {
                ctx.style
            },
            cramped: ctx.cramped,
        };
        let mut cells: Vec<Vec<MBox>> = Vec::new();
        for row in rows {
            let mut out_row = Vec::new();
            for cell in row {
                out_row.push(self.node(cell, cell_ctx)?);
            }
            cells.push(out_row);
        }
        let ncols = cells.iter().map(Vec::len).max().unwrap_or(0);
        let mut col_w = vec![0.0f64; ncols];
        for row in &cells {
            for (j, c) in row.iter().enumerate() {
                col_w[j] = col_w[j].max(c.width);
            }
        }
        let col_sep = 1.0; // 1 em between columns (plain-matrix feel)
        let baseline_skip = 1.2;
        let total_h = baseline_skip * (cells.len().saturating_sub(1)) as f64;
        let mut out = MBox::empty(AtomClass::Ord);
        let mut first_height = 0.0f64;
        let mut last_depth = 0.0f64;
        for (i, row) in cells.iter().enumerate() {
            let y = -(i as f64) * baseline_skip;
            if i == 0 {
                first_height = row.iter().map(|c| c.height).fold(0.0, f64::max);
            }
            if i == cells.len() - 1 {
                last_depth = row.iter().map(|c| c.depth).fold(0.0, f64::max);
            }
            let mut x = 0.0;
            for (j, c) in row.iter().enumerate() {
                out.merge_at(c, x + (col_w[j] - c.width) / 2.0, y);
                x += col_w[j] + col_sep;
            }
        }
        let width = col_w.iter().sum::<f64>() + col_sep * (ncols.saturating_sub(1)) as f64;
        // \vcenter: center the assembled block on the math axis.
        let block_h = first_height;
        let block_d = total_h + last_depth;
        let center = (block_h - block_d) / 2.0;
        let shift = self.m.axis_height - center;
        out.shift_baseline(shift);
        out.width = width;
        out.height = block_h + shift;
        out.depth = block_d - shift;
        Ok(out)
    }
}

impl MBox {
    /// Merge `other`'s fragment translated to `(dx, dy)`, growing extents.
    fn merge_at(&mut self, other: &MBox, dx: f64, dy: f64) {
        let mut frag = other.frag.clone();
        frag.translate(dx, dy);
        self.frag.absorb(frag);
        self.height = self.height.max(other.height + dy);
        self.depth = self.depth.max(other.depth - dy);
    }

    /// Shift the box's contents relative to its baseline.
    fn shift_baseline(&mut self, dy: f64) {
        self.frag.translate(0.0, dy);
        self.height += dy;
        self.depth -= dy;
    }
}

/// Typeset a source string in a style: the spike's front door.
///
/// # Errors
/// Parse errors, unsupported constructs (named, with spans — the ratchet
/// contract), and unmapped characters.
pub fn typeset(src: &str, style: Style) -> Result<Layout, MathError> {
    let engine = Engine::new().map_err(|_| MathError::Malformed {
        what: "bundled faces failed to parse",
        at: 0,
    })?;
    let root = parse(src)?;
    let ctx = Ctx {
        style,
        cramped: false,
    };
    let mbox = engine.node(&root, ctx)?;
    let mut layout = mbox.frag;
    layout.width = mbox.width;
    layout.height = mbox.height;
    layout.depth = mbox.depth;
    Ok(layout)
}
