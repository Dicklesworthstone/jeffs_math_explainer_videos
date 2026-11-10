//! Additional preambles reach native builders without contaminating selection.
use fmn_core::constants::{BLUE, RED};
use fmn_library::{Tex, TexEngine, TexText, VMobject};

fn geometry(mob: &VMobject) -> Vec<Vec<[u64; 3]>> {
    let mut result = vec![
        mob.points()
            .iter()
            .map(|point| point.map(f64::to_bits))
            .collect(),
    ];
    for child in mob.children() {
        result.extend(geometry(child));
    }
    result
}

#[test]
fn native_builder_expansion_matches_unfolded_geometry() {
    let engine = TexEngine::new("fmd-math/pack/default", None).unwrap();
    let preamble = r"\newcommand{\half}[1]{\frac{#1}{2}}";
    let macro_mob = Tex::new(r"\half{x}")
        .preamble(preamble)
        .font_size(36.)
        .build(&engine)
        .unwrap();
    let unfolded = Tex::new(r"\frac{x}{2}")
        .font_size(36.)
        .build(&engine)
        .unwrap();
    assert_eq!(geometry(&macro_mob.vmob), geometry(&unfolded.vmob));
    assert_eq!(macro_mob.typeset.source, r"\half{x}");
}

#[test]
fn native_coloring_and_selection_use_original_argument_spans() {
    let engine = TexEngine::new("fmd-math/pack/default", None).unwrap();
    let preamble = r"\newcommand{\sq}[1]{#1^2}";
    let source = r"\sq{x}+\sq{y}";
    let mob = Tex::new(source)
        .preamble(preamble)
        .t2c(&[("x", RED), ("y", BLUE)])
        .build(&engine)
        .unwrap();
    for (needle, color) in [("x", RED), ("y", BLUE)] {
        let selected = mob.occurrences(needle);
        assert_eq!(selected.len(), 1);
        assert!(!selected[0].is_empty());
        for index in &selected[0] {
            assert_eq!(mob.vmob.children()[*index].style().fill_color, color);
        }
    }
}

#[test]
fn text_builder_preambles_do_not_shift_plain_text_or_math_islands() {
    let engine = TexEngine::new("fmd-math/pack/default", None).unwrap();
    let mob = TexText::new(r"area $\sq{x}$")
        .preamble(r"\newcommand{\sq}[1]{#1^2}")
        .build(&engine)
        .unwrap();
    let unfolded = TexText::new(r"area $x^2$").build(&engine).unwrap();
    assert_eq!(geometry(&mob.vmob), geometry(&unfolded.vmob));
    assert_eq!(mob.typeset.source, r"area $\sq{x}$");
    assert!(!mob.occurrences("x")[0].is_empty());
}

#[test]
fn ordinary_and_explicit_empty_builders_are_identical() {
    let engine = TexEngine::new("fmd-math/pack/default", None).unwrap();
    let plain = Tex::new(r"x^2+1").build(&engine).unwrap();
    let configured = Tex::new(r"x^2+1").preamble("").build(&engine).unwrap();
    assert_eq!(geometry(&plain.vmob), geometry(&configured.vmob));
    assert_eq!(
        plain.typeset.to_bytes().unwrap(),
        configured.typeset.to_bytes().unwrap()
    );
}
