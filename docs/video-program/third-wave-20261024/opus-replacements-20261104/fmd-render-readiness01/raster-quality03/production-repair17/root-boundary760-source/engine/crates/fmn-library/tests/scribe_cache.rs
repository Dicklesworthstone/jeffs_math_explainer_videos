//! Exercise reuse through the actual builders used by the Python portal,
//! not a surrogate glyph loop. Geometry comparisons preserve every f64 bit
//! and each child boundary; style and span selection remain caller-owned.

use fmn_core::constants::{BLUE, RED, WHITE};
use fmn_library::{FontBook, Tex, TexEngine, TexText, Text, VMobject};

fn geometry(vmob: &VMobject) -> Vec<Vec<[u64; 3]>> {
    let points = vmob
        .points()
        .iter()
        .map(|point| point.map(f64::to_bits))
        .collect();
    let mut family = vec![points];
    for child in vmob.children() {
        family.extend(geometry(child));
    }
    family
}

#[test]
fn repeated_text_builders_reuse_outlines_without_reusing_color_or_span_state() {
    let book = FontBook::bundled().unwrap();
    let cold = Text::new("ababa").build(&book).unwrap();
    let glyph = &cold.layout.glyphs[0];
    let face = book
        .family(&glyph.face.family)
        .unwrap()
        .face(glyph.face.key);
    let before = face.glyph_cache_stats();
    assert_eq!(before.decodes, 3, "a, b, and the calibration glyph 0");

    let warm = Text::new("ababa").build(&book).unwrap();
    assert_eq!(geometry(&cold.vmob), geometry(&warm.vmob));
    assert_eq!(cold.select((2, 4)), warm.select((2, 4)));
    let after = face.glyph_cache_stats();
    assert_eq!(after.decodes, before.decodes);
    assert!(after.hits >= before.hits + 5);

    let colors = [("a", RED)];
    let colored = Text::new("ababa").t2c(&colors).build(&book).unwrap();
    assert_eq!(geometry(&cold.vmob), geometry(&colored.vmob));
    for (index, child) in colored.vmob.children().iter().enumerate() {
        let expected = if index % 2 == 0 { RED } else { WHITE };
        assert_eq!(child.style().fill_color, expected);
        assert_eq!(cold.vmob.children()[index].style().fill_color, WHITE);
    }
    assert_eq!(face.glyph_cache_stats().decodes, before.decodes);
}

#[test]
fn repeated_tex_builders_keep_exact_geometry_and_independent_matching_colors() {
    let engine = TexEngine::new("fmd-math/pack/default", None).unwrap();
    let source = r"\frac{u}{v} + \begin{pmatrix} u & 1 \\ 0 & v \end{pmatrix}";
    let cold = Tex::new(source).build(&engine).unwrap();
    let before = engine.memory_cache_stats();
    assert_eq!(before.entries, 2, "formula and calibration probe");
    let warm = Tex::new(source).build(&engine).unwrap();
    assert_eq!(geometry(&cold.vmob), geometry(&warm.vmob));
    assert_eq!(
        cold.typeset.to_bytes().unwrap(),
        warm.typeset.to_bytes().unwrap()
    );
    assert_eq!(cold.occurrences("u"), warm.occurrences("u"));
    assert_eq!(engine.memory_cache_stats().misses, before.misses);
    assert_eq!(engine.memory_cache_stats().hits, before.hits + 2);

    // u/v do not also occur inside control-word or environment names.
    let colors = [("u", RED), ("v", BLUE)];
    let colored = Tex::new(source).t2c(&colors).build(&engine).unwrap();
    assert_eq!(geometry(&cold.vmob), geometry(&colored.vmob));
    for (needle, color) in colors {
        let occurrences = colored.occurrences(needle);
        assert_eq!(occurrences.len(), 2, "one fraction and one matrix glyph");
        for occurrence in occurrences {
            assert_eq!(occurrence.len(), 1);
            for ordinal in occurrence {
                assert_eq!(colored.vmob.children()[ordinal].style().fill_color, color);
                assert_eq!(cold.vmob.children()[ordinal].style().fill_color, WHITE);
            }
        }
    }
    assert_eq!(engine.memory_cache_stats().misses, before.misses);
    assert_eq!(engine.memory_cache_stats().hits, before.hits + 4);
}

#[test]
fn textext_math_islands_reuse_the_production_cache_without_losing_source_identity() {
    let engine = TexEngine::new("fmd-math/pack/default", None).unwrap();
    let source = r"the area $\pi r^2$ of a \textbf{circle}";
    let cold = TexText::new(source).build(&engine).unwrap();
    let before = engine.memory_cache_stats();
    let warm = TexText::new(source).build(&engine).unwrap();
    assert_eq!(geometry(&cold.vmob), geometry(&warm.vmob));
    assert_eq!(
        cold.typeset.to_bytes().unwrap(),
        warm.typeset.to_bytes().unwrap()
    );
    let selection = cold.occurrences(r"\pi");
    assert_eq!(selection.len(), 1);
    assert!(!selection[0].is_empty());
    assert_eq!(warm.occurrences(r"\pi"), selection);
    assert_eq!(engine.memory_cache_stats().misses, before.misses);
    assert_eq!(engine.memory_cache_stats().hits, before.hits + 2);
}
