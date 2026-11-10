//! fm-census-drawing-window-classes-kw9m: structural-class surface fixtures
//!
//! Verifies that the most common structural classes from the Reference's
//! surface are reachable through the native Rust API and can be built into
//! the Marionette stage. The tests are intentionally shallow (construction,
//! family walk, tracker read/write) rather than render-output checks; the
//! W10 Gauntlet's render-matrix scenarios cover full rasterization paths.

use fmn_library::controls::ControlMobject;
use fmn_library::{
    DotCloud, ImageMobject, PMobject, Tex, TexText, Text, VMobject, group, p_group, v_group,
};
use fmn_mobject::dynamics::TrackerKind;
use fmn_mobject::stage::Stage;
use fmn_tex::TexEngine;
use fmn_text::FontBook;

fn book() -> FontBook {
    FontBook::bundled().expect("bundled font book")
}

/// `Group` is the heterogeneous `Mobject` family container.
#[test]
fn group_collects_heterogeneous_children() {
    let circle = VMobject::from_points(vec![[0.0; 3], [1.0; 3]]);
    let square = VMobject::from_points(vec![[0.0; 3], [1.0; 3], [1.0, 1.0, 0.0]]);
    let g = group([circle.into(), square.into()]);
    assert_eq!(g.submobjects.len(), 2);
}

/// `VGroup` is the `VMobject` family container; non-vector children are refused
/// at the type level (it only accepts `VMobject`).
#[test]
fn v_group_collects_only_vectorized_children() {
    let sq = fmn_library::Square::new().build();
    let v = v_group([sq.clone()]);
    assert_eq!(v.children().len(), 1);
}

/// `ValueTracker` is the typed single-lane scalar tracker; the round
/// trip `set` -> `get` is exact.
#[test]
fn value_tracker_set_get_round_trip() {
    let mut stage = Stage::new();
    let handle = stage.add_value_tracker(0.0);
    stage.set_tracker_value(handle, 1.5).expect("scalar set");
    assert_eq!(stage.tracker_value(handle), Some(1.5));
}

/// `ExponentialValueTracker` encodes through `fmn_dmath::ln` so lane
/// interpolation is geometric; `get` decodes through `exp`.
#[test]
fn exponential_value_tracker_geometric_round_trip() {
    let mut stage = Stage::new();
    let handle = stage.add_exponential_value_tracker(2.0);
    let v = stage.tracker_value(handle).expect("exponential get");
    assert!((v - 2.0).abs() < 1.0e-9, "geometric round-trip: got {v}");
}

/// `ComplexValueTracker` stores two lanes and round-trips through
/// [`Stage::tracker_complex_value`].
#[test]
fn complex_value_tracker_round_trip() {
    let mut stage = Stage::new();
    let handle = stage.add_complex_value_tracker(3.0, 4.0);
    assert_eq!(stage.tracker_complex_value(handle), Some((3.0, 4.0)));
}

/// `ControlMobject` is the typed scalar-control base.
#[test]
fn control_mobject_set_get_round_trip() {
    let mut c = ControlMobject::new(0.5, std::iter::empty::<VMobject>());
    assert_eq!(c.value(), 0.5);
    c.set_value(0.9);
    assert_eq!(c.value(), 0.9);
}

/// `PMobject` and `PGroup` are the typed point-cloud family and its
/// variadic container.
#[test]
fn pmobject_and_pgroup_construct() {
    let p = PMobject::new().with_points(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
    assert_eq!(p.num_points(), 2);
    let p2 = PMobject::new().with_points(vec![[2.0, 0.0, 0.0]]);
    let g = p_group([p, p2]).ingest_submobjects();
    assert_eq!(g.num_points(), 3, "PGroup merges child point buffers");
}

/// `DotCloud` carries the typed dot-cloud surface with 3-D shading enabled.
#[test]
fn dot_cloud_construction_and_shading() {
    let dc = DotCloud::new(vec![[0.0, 0.0, 0.0], [0.5, 0.0, 0.0]]).make_3d();
    assert_eq!(dc.num_points(), 2);
}

/// `SVGMobject` (Chisel) constructs from a minimal valid user SVG document.
#[test]
fn svg_mobject_constructs_from_minimal_rect() {
    let svg = fmn_library::svg_mobject(
        br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><path d="M0 0 L10 10"/></svg>"#,
    )
    .expect("minimal path parses");
    assert!(!svg.children().is_empty(), "path becomes children");
}

/// `ImageMobject` constructs from a 1x1 RGB byte buffer (the minimum raster
/// payload).
#[test]
fn image_mobject_constructs_from_rgba8() {
    let img = ImageMobject::from_rgba8(1, 1, vec![0xFF, 0x00, 0x00, 0xFF])
        .expect("1x1 RGBA8 image materializes");
    assert_eq!(img.pixel_width(), 1);
    assert_eq!(img.pixel_height(), 1);
}

/// `Text` constructs the bare glyph-family surface.
#[test]
fn text_constructs_with_string_and_font_size() {
    let text = Text::new("hello").font_size(12.0);
    let built = text.build(&book()).expect("text builds");
    assert!(!built.vmob.children().is_empty());
}

/// `Tex` constructs from `TexEngine` + a minimal valid TeX string; the
/// engine round-trips through the typed surface.
#[test]
fn tex_engine_constructs_and_typesets_minimal_input() {
    let engine = TexEngine::new("default", None).expect("tex engine builds");
    let tex = Tex::new(r"x")
        .font_size(12.0)
        .build(&engine)
        .expect("minimal TeX typesets");
    assert!(
        !tex.vmob.children().is_empty(),
        "x produces at least one glyph"
    );
}

/// `TexText` constructs from `TexEngine` + minimal prose.
#[test]
fn tex_text_engine_constructs_and_typesets_minimal_input() {
    let engine = TexEngine::new("default", None).expect("tex engine builds");
    let tex = TexText::new(r"x")
        .font_size(12.0)
        .build(&engine)
        .expect("minimal TexText typesets");
    assert!(
        !tex.vmob.children().is_empty(),
        "x produces at least one glyph"
    );
}

/// `ValueTracker` tracker kinds are distinguishable through the stage.
#[test]
fn tracker_kinds_round_trip() {
    let mut stage = Stage::new();
    let h_plain = stage.add_value_tracker(1.0);
    let h_exp = stage.add_exponential_value_tracker(2.0);
    let h_cx = stage.add_complex_value_tracker(3.0, 4.0);
    assert_eq!(
        stage.tracker(h_plain).map(|t| t.kind),
        Some(TrackerKind::Plain)
    );
    assert_eq!(
        stage.tracker(h_exp).map(|t| t.kind),
        Some(TrackerKind::Exponential)
    );
    assert_eq!(stage.tracker_complex_value(h_cx), Some((3.0, 4.0)));
}

/// `ControlMobject` composes its children into a `VGroup`.
#[test]
fn control_mobject_composition_is_vgroup() {
    let label = Text::new("v")
        .font_size(12.0)
        .build(&book())
        .expect("label builds");
    let c = ControlMobject::new(0.0, std::iter::once(label.vmob));
    let comp = c.composition();
    assert_eq!(comp.children().len(), 1, "composition holds the child");
}
