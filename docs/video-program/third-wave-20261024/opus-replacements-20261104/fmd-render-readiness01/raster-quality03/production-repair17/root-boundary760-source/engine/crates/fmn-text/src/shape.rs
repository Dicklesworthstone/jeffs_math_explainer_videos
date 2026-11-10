//! Shaping (§11.2): styled characters → positioned-ready glyph items —
//! cmap→gids, kerning (kern + the focused GPOS subset), and the bundled
//! ligature sets **off by default** (the familiar manim look; opt-in
//! flag). Every glyph keeps its source provenance; a ligature covers its
//! whole character range.

use crate::error::TextError;
use crate::font::{FaceKey, FontBook, glyph_metrics, kern_em};
use crate::markup::{Script, StyledChar};
use fmn_core::color::Srgb;

/// Which face a glyph resolved to: a family in the book plus the variant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FaceSel {
    /// Canonical family name (resolvable via [`FontBook::resolve_face`];
    /// a per-character fallback glyph names the bundled face it came from).
    pub family: String,
    /// Bold/italic variant.
    pub key: FaceKey,
}

/// One shaped item.
#[derive(Clone, Debug, PartialEq)]
pub enum ShapedItem {
    /// A glyph.
    Glyph(ShapedGlyph),
    /// An interword space (no glyph, no submobject — the Reference's
    /// convention).
    Space {
        /// Advance, ems.
        width: f64,
        /// Source span of the space character(s).
        span: (usize, usize),
    },
    /// A forced line break (`\n` in the source).
    Newline {
        /// Source span of the newline.
        span: (usize, usize),
    },
}

/// A shaped glyph with full provenance and resolved style.
#[derive(Clone, Debug, PartialEq)]
pub struct ShapedGlyph {
    /// The face it resolved to.
    pub face: FaceSel,
    /// Glyph id in that face.
    pub gid: u16,
    /// The (first) character it renders.
    pub ch: char,
    /// Advance width, ems (kern with the previous glyph already applied
    /// as a leading adjustment).
    pub advance: f64,
    /// Kern against the previous glyph in the same run, ems (applied
    /// before this glyph).
    pub kern: f64,
    /// Size factor (markup `<big>`/`<small>`/scripts).
    pub size: f64,
    /// Baseline shift, ems, positive up (`<sup>`/`<sub>`).
    pub baseline_shift: f64,
    /// Source byte span (a ligature covers its whole character range).
    pub span: (usize, usize),
    /// Index of the first covered character in the decoded sequence.
    pub char_index: usize,
    /// How many decoded characters this glyph covers (1; ligatures more).
    pub cluster_len: usize,
    /// Resolved fill, if any style set one.
    pub fill: Option<Srgb>,
    /// Underline decoration requested.
    pub underline: bool,
    /// Strikethrough decoration requested.
    pub strike: bool,
}

/// Baseline shifts for scripts, in ems of the parent size.
const SUP_SHIFT: f64 = 0.35;
const SUB_SHIFT: f64 = -0.15;

/// Shape styled characters. `ligatures` turns the bundled faces' ligature
/// sets on (off by default).
///
/// # Errors
///
/// [`TextError::FontUnavailable`] for an unknown requested family;
/// [`TextError::UnmappedChar`] for a character neither the selected family
/// nor any face of the bundled [`FontBook::glyph_fallback`] chain can render.
pub fn shape(
    book: &FontBook,
    chars: &[StyledChar],
    ligatures: bool,
) -> Result<Vec<ShapedItem>, TextError> {
    let mut items = Vec::new();
    let mut run: Vec<(usize, u16)> = Vec::new(); // (chars index, gid)
    let mut run_face: Option<FaceSel> = None;
    let flush =
        |items: &mut Vec<ShapedItem>, run: &mut Vec<(usize, u16)>, face: &Option<FaceSel>| {
            let Some(face_sel) = face else { return };
            emit_run(book, chars, run, face_sel, ligatures, items);
            run.clear();
        };
    for (ix, sc) in chars.iter().enumerate() {
        if sc.ch == '\n' {
            flush(&mut items, &mut run, &run_face);
            run_face = None;
            items.push(ShapedItem::Newline { span: sc.span });
            continue;
        }
        let (family_name, face) = resolve_family(book, sc)?;
        let key = FaceKey {
            bold: sc.style.bold,
            italic: sc.style.italic,
        };
        if sc.ch.is_whitespace() {
            flush(&mut items, &mut run, &run_face);
            run_face = None;
            let space_face = book.family(&family_name)?.face(key);
            let space_gid = space_face.font.glyph_index(' ');
            let width = if space_gid == 0 {
                1.0 / 3.0
            } else {
                glyph_metrics(space_face, space_gid).advance
            } * sc.style.size_factor;
            items.push(ShapedItem::Space {
                width,
                span: sc.span,
            });
            continue;
        }
        let (family_name, gid) = match face.font.glyph_index(sc.ch) {
            0 => {
                let (fallback, gid) = book.glyph_fallback(&family_name, key, sc.ch).ok_or(
                    TextError::UnmappedChar {
                        ch: sc.ch,
                        span: sc.span,
                    },
                )?;
                (fallback.to_owned(), gid)
            }
            gid => (family_name, gid),
        };
        let sel = FaceSel {
            family: family_name,
            key,
        };
        // A run extends while face and scalar style stay constant (kerning
        // and ligatures apply within runs).
        let same = run_face.as_ref() == Some(&sel)
            && run
                .last()
                .map(|&(prev_ix, _)| same_scalar_style(&chars[prev_ix], sc))
                .unwrap_or(true);
        if !same {
            flush(&mut items, &mut run, &run_face);
        }
        run_face = Some(sel);
        run.push((ix, gid));
    }
    flush(&mut items, &mut run, &run_face);
    Ok(items)
}

/// Only shaping-relevant properties break a run: size and script position
/// change metrics; color/gradient/underline/strike are per-glyph outputs
/// and must never affect kerning (a `t2c` boundary does not change
/// metrics, matching the Reference).
fn same_scalar_style(a: &StyledChar, b: &StyledChar) -> bool {
    (a.style.size_factor - b.style.size_factor).abs() < 1e-12 && a.style.script == b.style.script
}

/// Resolve the family a character shapes from: explicit request (`t2f` /
/// span `font_family`) wins and must exist; `<tt>` selects the monospace
/// family; otherwise the default family.
fn resolve_family<'b>(
    book: &'b FontBook,
    sc: &StyledChar,
) -> Result<(String, &'b crate::font::Face), TextError> {
    let key = FaceKey {
        bold: sc.style.bold,
        italic: sc.style.italic,
    };
    if let Some(name) = &sc.style.family {
        let family = book.family(name)?;
        return Ok((family.name.clone(), family.face(key)));
    }
    let family = if sc.style.mono {
        book.mono_family()
    } else {
        book.default_family()
    };
    Ok((family.name.clone(), family.face(key)))
}

fn emit_run(
    book: &FontBook,
    chars: &[StyledChar],
    run: &[(usize, u16)],
    face_sel: &FaceSel,
    ligatures: bool,
    items: &mut Vec<ShapedItem>,
) {
    if run.is_empty() {
        return;
    }
    let Ok(face) = book.resolve_face(&face_sel.family, face_sel.key) else {
        return; // shape() resolved it already; unreachable in practice
    };
    // Optional ligature substitution: (gid, covered char count) pairs.
    let gids: Vec<u16> = run.iter().map(|&(_, gid)| gid).collect();
    let shaped: Vec<(u16, usize)> = if ligatures {
        face.font.gsub_ligatures().substitute_with_spans(&gids)
    } else {
        gids.iter().map(|&g| (g, 1)).collect()
    };
    let mut cursor = 0_usize; // index into `run`
    let mut prev_gid: Option<u16> = None;
    for (gid, covered) in shaped {
        let covered = covered.max(1).min(run.len() - cursor);
        let first_ix = run[cursor].0;
        let last_ix = run[cursor + covered - 1].0;
        let first = &chars[first_ix];
        let last = &chars[last_ix];
        let size = first.style.size_factor;
        let kern = prev_gid.map_or(0.0, |p| kern_em(face, p, gid) * size);
        let metrics = glyph_metrics(face, gid);
        let baseline_shift = match first.style.script {
            Script::Normal => 0.0,
            Script::Sup => SUP_SHIFT,
            Script::Sub => SUB_SHIFT,
        };
        let fill = match &first.style.gradient {
            Some((stops, t)) => Some(crate::maps::sample_gradient(stops, *t)),
            None => first.style.color,
        };
        items.push(ShapedItem::Glyph(ShapedGlyph {
            face: face_sel.clone(),
            gid,
            ch: first.ch,
            advance: metrics.advance * size,
            kern,
            size,
            baseline_shift,
            span: (first.span.0, last.span.1),
            char_index: first.char_index,
            cluster_len: last.char_index - first.char_index + 1,
            fill,
            underline: first.style.underline,
            strike: first.style.strike,
        }));
        prev_gid = Some(gid);
        cursor += covered;
    }
}
