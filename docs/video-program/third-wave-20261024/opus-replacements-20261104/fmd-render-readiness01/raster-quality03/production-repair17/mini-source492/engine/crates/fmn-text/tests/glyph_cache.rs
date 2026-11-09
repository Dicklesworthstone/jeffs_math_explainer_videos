//! Production Scribe outline reuse: identity, immutable geometry, span
//! independence, bounded storage, and bit-identical cold/warm placement.

use fmn_geom::QuadPath;
use fmn_text::font::FaceKey;
use fmn_text::{FontBook, PlacedTextGlyph, TextRequest, glyph_quadpath, layout_text};

// The pre-cache implementation, kept independent of the command cache so a
// floating-point association change cannot pass by comparing two warm hits.
fn uncached(book: &FontBook, glyph: &PlacedTextGlyph) -> QuadPath {
    let font = &book
        .family(&glyph.face.family)
        .unwrap()
        .face(glyph.face.key)
        .font;
    let outline = font.glyph_outline(glyph.gid).unwrap();
    let scale = glyph.size / f64::from(font.units_per_em.max(1));
    let v = |x: f64, y: f64| [glyph.x + x * scale, glyph.y + y * scale, 0.0];
    let mut path = QuadPath::new();
    for contour in &outline.contours {
        path.start_new_path(v(contour.start.x, contour.start.y));
        for segment in &contour.segments {
            match segment {
                fmd_font::outline::Segment::Line { to } => {
                    path.add_line_to(v(to.x, to.y), true).unwrap();
                }
                fmd_font::outline::Segment::Quad { ctrl, to } => {
                    path.add_quadratic_bezier_curve_to(v(ctrl.x, ctrl.y), v(to.x, to.y), true)
                        .unwrap();
                }
            }
        }
    }
    path
}

fn bits(path: &QuadPath) -> Vec<[u64; 3]> {
    path.points()
        .iter()
        .map(|point| point.map(f64::to_bits))
        .collect()
}

#[test]
fn cached_paths_match_the_uncached_arithmetic_at_every_placement() {
    let book = FontBook::bundled().unwrap();
    let layout = layout_text(&book, &TextRequest::plain("AoB8gjfi")).unwrap();
    for glyph in &layout.glyphs {
        for (x, y, size) in [(0.0, 0.0, 1.0), (1.0 / 3.0, -7.1, 0.375), (1e7, -1e6, 13.7)] {
            let mut glyph = glyph.clone();
            glyph.x = x;
            glyph.y = y;
            glyph.size = size;
            let expected = bits(&uncached(&book, &glyph));
            assert_eq!(bits(&glyph_quadpath(&book, &glyph).unwrap()), expected);
            assert_eq!(bits(&glyph_quadpath(&book, &glyph).unwrap()), expected);
        }
    }
    let first = &layout.glyphs[0];
    let stats = book
        .family(&first.face.family)
        .unwrap()
        .face(first.face.key)
        .glyph_cache_stats();
    assert_eq!(stats.decodes, 8);
    assert_eq!(stats.hits, 40);
}

#[test]
fn repeated_glyphs_keep_independent_spans_styles_and_mutable_paths() {
    let book = FontBook::bundled().unwrap();
    let layout = layout_text(&book, &TextRequest::plain("oooo")).unwrap();
    let spans: Vec<_> = layout.glyphs.iter().map(|glyph| glyph.span).collect();
    assert_eq!(spans, vec![(0, 1), (1, 2), (2, 3), (3, 4)]);
    let paths: Vec<_> = layout
        .glyphs
        .iter()
        .map(|glyph| glyph_quadpath(&book, glyph).unwrap())
        .collect();
    assert_ne!(bits(&paths[0]), bits(&paths[1]));
    let mut edited = paths[0].clone();
    edited.add_line_to([123.0, 456.0, 0.0], true).unwrap();
    assert_ne!(bits(&edited), bits(&paths[0]));
    assert_eq!(
        bits(&glyph_quadpath(&book, &layout.glyphs[0]).unwrap()),
        bits(&paths[0])
    );
    assert_eq!(layout.select((2, 3)), vec![2]);
    let mut alias = layout.glyphs[0].clone();
    alias.face.family = "CMU Serif".to_owned();
    alias.span = (200, 204);
    assert_eq!(
        bits(&glyph_quadpath(&book, &alias).unwrap()),
        bits(&paths[0])
    );
    assert_eq!(alias.span, (200, 204));
    let stats = book
        .family("CMU Serif")
        .unwrap()
        .face(alias.face.key)
        .glyph_cache_stats();
    assert_eq!(stats.decodes, 1);
    assert_eq!(stats.hits, 5);
}

#[test]
fn actual_faces_not_requested_variants_or_glyph_ids_own_the_cache() {
    let mut book = FontBook::bundled().unwrap();
    let layout = layout_text(&book, &TextRequest::plain("o")).unwrap();
    let glyph = &layout.glyphs[0];
    glyph_quadpath(&book, glyph).unwrap();
    let mut other = glyph.clone();
    other.face.key = FaceKey {
        bold: true,
        italic: false,
    };
    let bold = book
        .family(&other.face.family)
        .unwrap()
        .face(other.face.key);
    other.gid = bold.font.glyph_index(other.ch);
    assert_eq!(bold.glyph_cache_stats().decodes, 0);
    assert_eq!(
        bits(&glyph_quadpath(&book, &other).unwrap()),
        bits(&uncached(&book, &other))
    );
    assert_eq!(bold.glyph_cache_stats().decodes, 1);
    book.add_family("User Copy", fmd_font::bundled::CM_REGULAR.to_vec())
        .unwrap();
    other.face.family = "User Copy".to_owned();
    other.gid = book
        .family("User Copy")
        .unwrap()
        .face(other.face.key)
        .font
        .glyph_index('o');
    glyph_quadpath(&book, &other).unwrap();
    other.face.key = FaceKey {
        bold: false,
        italic: false,
    };
    glyph_quadpath(&book, &other).unwrap();
    let stats = book
        .family("User Copy")
        .unwrap()
        .face(other.face.key)
        .glyph_cache_stats();
    assert_eq!(
        stats.decodes, 1,
        "variant fallback shares the actual regular face"
    );
    assert_eq!(stats.hits, 1);
}

#[test]
fn concurrent_cold_requests_decode_once_and_return_identical_geometry() {
    let book = FontBook::bundled().unwrap();
    let layout = layout_text(&book, &TextRequest::plain("B")).unwrap();
    let glyph = &layout.glyphs[0];
    let expected = bits(&uncached(&book, glyph));
    std::thread::scope(|scope| {
        let mut workers = Vec::new();
        for _ in 0..8 {
            let book = &book;
            workers.push(scope.spawn(move || bits(&glyph_quadpath(book, glyph).unwrap())));
        }
        for worker in workers {
            assert_eq!(worker.join().unwrap(), expected);
        }
    });
    let stats = book
        .family(&glyph.face.family)
        .unwrap()
        .face(glyph.face.key)
        .glyph_cache_stats();
    assert_eq!(stats.decodes, 1);
    assert_eq!(stats.hits, 7);
}
