//! fm-ydw acceptance: fmd-font's decoded glyph outlines round-trip into
//! W2's `QuadPath` with zero geometric loss — every decoded segment maps
//! 1:1 onto a shared-anchor quad curve, extents survive exactly, closure
//! survives exactly, and the arc length the fmn-geom table computes equals
//! an independent sum over the decoded segments. This is the seam Scribe
//! (fmn-text/fmn-tex) builds on: TrueType outlines are already quadratic,
//! so the conversion is transcription, not approximation.

use fmd_font::Font;
use fmd_font::outline::{GlyphOutline, Segment};
use fmn_core::types::Vec3;
use fmn_geom::QuadPath;
use fmn_geom::arclength::{ArcLengthTable, quadratic_arc_length};

fn v(x: f64, y: f64) -> Vec3 {
    [x, y, 0.0]
}

/// Transcribe a decoded outline into one QuadPath (one subpath per
/// contour), 1:1: lines keep their midpoint handle, quads keep their
/// control point verbatim.
fn to_quadpath(outline: &GlyphOutline) -> QuadPath {
    let mut path = QuadPath::new();
    for contour in &outline.contours {
        path.start_new_path(v(contour.start.x, contour.start.y));
        for seg in &contour.segments {
            match seg {
                Segment::Line { to } => {
                    path.add_line_to(v(to.x, to.y), true).expect("line appends");
                }
                Segment::Quad { ctrl, to } => {
                    path.add_quadratic_bezier_curve_to(v(ctrl.x, ctrl.y), v(to.x, to.y), true)
                        .expect("quad appends");
                }
            }
        }
    }
    path
}

/// Independent arc-length: sum the decoded segments directly.
fn decoded_arc_length(outline: &GlyphOutline) -> f64 {
    let mut total = 0.0;
    for contour in &outline.contours {
        let mut cursor = contour.start;
        for seg in &contour.segments {
            match seg {
                Segment::Line { to } => {
                    total += ((to.x - cursor.x).powi(2) + (to.y - cursor.y).powi(2)).sqrt();
                }
                Segment::Quad { ctrl, to } => {
                    total += quadratic_arc_length(
                        v(cursor.x, cursor.y),
                        v(ctrl.x, ctrl.y),
                        v(to.x, to.y),
                    );
                }
            }
            cursor = seg.to();
        }
    }
    total
}

fn cm() -> Font {
    Font::parse(fmd_font::bundled::CM_REGULAR.to_vec()).expect("bundled CM parses")
}

#[test]
fn cm_glyphs_roundtrip_into_quadpath_losslessly() {
    let font = cm();
    for ch in ['H', 'o', 'i', 'g', '&', 'é', 'ç', '$'] {
        let gid = font.glyph_index(ch);
        assert_ne!(gid, 0, "CM must map {ch:?}");
        let outline = font.glyph_outline(gid).expect("glyph decodes");
        let path = to_quadpath(&outline);

        // 1:1 curve mapping: every decoded segment became exactly one quad
        // curve (lines ride as midpoint-handle quads — the same point set),
        // plus one shared-anchor break curve per additional contour (the
        // model marks a subpath boundary as a degenerate curve).
        let decoded_segments: usize = outline.contours.iter().map(|c| c.segments.len()).sum();
        let separators = outline.contours.len() - 1;
        assert_eq!(
            path.num_curves(),
            decoded_segments + separators,
            "{ch:?}: segment count changed in transcription"
        );

        // Closure survives: the decoder guarantees every contour ends on
        // its start, so the path's last subpath must read as closed.
        assert!(path.is_closed(), "{ch:?}: contour closure lost");
        assert_eq!(
            path.subpaths().len(),
            outline.contours.len(),
            "{ch:?}: contour count changed"
        );

        // Extents survive exactly: the path's point set (anchors + handles)
        // spans the same box as the decoded outline — f64 transcription,
        // no rounding anywhere.
        let ext = outline.extents().expect("drawable glyph has extents");
        let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        for p in path.points() {
            x0 = x0.min(p[0]);
            y0 = y0.min(p[1]);
            x1 = x1.max(p[0]);
            y1 = y1.max(p[1]);
        }
        assert_eq!([x0, y0, x1, y1], ext, "{ch:?}: extents drifted");

        // Arc length: the fmn-geom table over the transcribed path equals
        // an independent sum over the decoded segments plus the deliberate
        // manim-model contribution of the subpath break curves (each break
        // is a degenerate quad spanning the gap between one closed
        // contour's start and the next contour's start).
        let table_total = ArcLengthTable::for_path(&path).total();
        let mut expected_total = decoded_arc_length(&outline);
        for pair in outline.contours.windows(2) {
            let (a, b) = (pair[0].start, pair[1].start);
            expected_total += ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt();
        }
        assert!(expected_total > 0.0, "{ch:?}: degenerate outline");
        let rel = (table_total - expected_total).abs() / expected_total;
        assert!(
            rel < 1e-12,
            "{ch:?}: arc length drifted through the round-trip: table {table_total} vs expected {expected_total}"
        );
    }
}

#[test]
fn every_bundled_face_is_consumable_from_fmn() {
    // The §11.1 sovereignty check at the seam: all four bundled families
    // parse and yield decodable outlines through the pinned dependency —
    // nothing about typesetting will depend on host fonts.
    for (name, bytes) in fmd_font::bundled::ALL_FACES {
        let font = Font::parse(bytes.to_vec())
            .unwrap_or_else(|e| panic!("bundled face {name} failed to parse: {e}"));
        let mut decoded = 0usize;
        for gid in 0..font.num_glyphs.min(48) {
            if font.glyph_outline(gid).is_ok() {
                decoded += 1;
            }
        }
        assert!(decoded > 0, "{name}: no decodable glyphs");
    }
}
