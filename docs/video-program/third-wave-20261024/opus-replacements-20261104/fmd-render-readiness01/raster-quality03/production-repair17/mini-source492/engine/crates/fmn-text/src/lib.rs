//! Scribe I: native text shaping, markup, and layout over fmd-font
//! (§11.2), with §11.3 span provenance on every glyph.
//!
//! The pipeline: source text → styled characters (plain, or the manim
//! markup tag set with precise line:column diagnostics) → the
//! `t2c`/`t2f`/`t2g`/`t2s`/`t2w` maps applied by source-byte occurrence →
//! shaping (cmap→gids, kern + focused-GPOS kerning, bundled ligature sets
//! **off by default** to keep the familiar manim look) → line breaking
//! (greedy with manim's width semantics; least-badness as an explicit
//! option) → [`TextLayout`]: positioned glyphs in ems (y-up, first
//! baseline at 0), each carrying its face, source span, character index,
//! and submobject ordinal — the `Text[3:7]` / `isolate=` compatibility
//! surface is structural, exactly the Reference's `StringMobject`
//! conventions (non-whitespace glyphs, in order; a ligature is one
//! submobject covering its character range).
//!
//! **Font policy (D-08).** The bundled default face renders identically on
//! every machine; user TTFs load from bytes; family-name lookup never
//! silently substitutes — a miss is a named capability-style error. The
//! default text face is bundled Computer Modern where the Reference
//! defaulted to a host font through Pango; metric differences are
//! Behavior-Noted (BN-05).
//!
//! Output geometry rides the proven fmd-font→QuadPath transcription seam:
//! TrueType outlines are already quadratic, so [`glyph_quadpath`] is
//! transcription, not approximation.

#![forbid(unsafe_code)]

pub mod error;
pub mod font;
pub mod layout;
pub mod maps;
pub mod markup;
pub mod shape;

pub use error::TextError;
pub use font::{
    BUNDLED_FONT_FAMILIES, DEFAULT_FAMILY, FontBook, MATH_FAMILY, MONO_FAMILY, SANS_FAMILY,
    bundled_faces, bundled_font_inventory, is_bundled_text_family,
};
pub use layout::{
    Align, Decoration, Line, LineBreaker, PlacedTextGlyph, TextLayout, TextRequest, layout_text,
};
pub use maps::StyleMaps;

use fmn_core::types::Vec3;
use fmn_geom::QuadPath;
use font::OutlineCommand;

/// The outline of a placed glyph as a positioned [`QuadPath`], in the
/// layout's em coordinates: one subpath per contour, scaled by the
/// glyph's size and translated to its position — 1:1 transcription of the
/// decoded quadratic segments. Font-unit outline commands are shared through
/// the loaded face's bounded cache; placement, scale, and the returned mutable
/// path remain private to this glyph. Source spans never enter the cache.
///
/// # Errors
///
/// [`TextError::FontUnavailable`] if the glyph's family left the book;
/// [`TextError::Outline`] on a decode failure.
pub fn glyph_quadpath(book: &FontBook, glyph: &PlacedTextGlyph) -> Result<QuadPath, TextError> {
    let face = book.resolve_face(&glyph.face.family, glyph.face.key)?;
    let commands = face.glyph_commands(glyph.gid, glyph.ch)?;
    let upm = f64::from(face.font.units_per_em.max(1));
    let s = glyph.size / upm;
    let v = |x: f64, y: f64| -> Vec3 { [glyph.x + x * s, glyph.y + y * s, 0.0] };
    let mut path = QuadPath::new();
    for command in commands.iter() {
        match *command {
            OutlineCommand::Move([x, y]) => {
                path.start_new_path(v(x, y));
            }
            OutlineCommand::Line([x, y]) => {
                path.add_line_to(v(x, y), true)
                    .map_err(|e| TextError::Outline {
                        ch: glyph.ch,
                        what: format!("{e:?}"),
                    })?;
            }
            OutlineCommand::Quad([cx, cy], [x, y]) => {
                path.add_quadratic_bezier_curve_to(v(cx, cy), v(x, y), true)
                    .map_err(|e| TextError::Outline {
                        ch: glyph.ch,
                        what: format!("{e:?}"),
                    })?;
            }
        }
    }
    Ok(path)
}
