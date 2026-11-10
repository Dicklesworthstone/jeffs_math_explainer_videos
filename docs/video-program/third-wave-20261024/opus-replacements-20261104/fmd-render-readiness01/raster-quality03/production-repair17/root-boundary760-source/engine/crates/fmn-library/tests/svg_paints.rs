use fmn_library::VMobject;
use fmn_library::svg::{SvgPaintOverrides, svg_mobject_with_paints};

fn build(source: &str) -> VMobject {
    svg_mobject_with_paints(source.as_bytes(), SvgPaintOverrides::default()).unwrap()
}
fn area(mob: &VMobject) -> f64 {
    mob.path()
        .unwrap()
        .subpaths()
        .iter()
        .map(|points| {
            points
                .iter()
                .step_by(2)
                .zip(points.iter().skip(2).step_by(2))
                .map(|(a, b)| (a[0] * b[1] - b[0] * a[1]) / 2.0)
                .sum::<f64>()
        })
        .sum()
}
#[test]
fn even_odd_is_a_native_filled_set_not_the_original_double_winding() {
    let source = r#"<svg><path fill-rule="evenodd" d="M0 0H10V10H0Z M2 2H8V8H2Z"/></svg>"#;
    let svg = build(source);
    assert_eq!(svg.children().len(), 1);
    let fill = &svg.children()[0].children()[0];
    assert!((area(fill).abs() - 64.0).abs() < 1e-8, "{}", area(fill));
    assert_eq!(fill.style().stroke_opacity, 0.0);
}
#[test]
fn coincident_even_odd_fill_cancels_without_erasing_the_stroke() {
    let svg = build(
        r#"<svg><path fill-rule="evenodd" stroke="red" d="M0 0H10V10H0Z M0 0H10V10H0Z"/></svg>"#,
    );
    let layers = svg.children()[0].children();
    assert_eq!(layers.len(), 2);
    assert!(layers[0].points().is_empty());
    assert_eq!(layers[1].style().fill_opacity, 0.0);
    assert_eq!(layers[1].points().len(), 19);
}
#[test]
fn unequal_dashes_odd_lists_offsets_and_moves_use_svg_distance_units() {
    for (pattern, offset, expected) in [
        ("2 1 4 1", 0, vec![(0., 2.), (3., 7.), (8., 10.)]),
        ("2 1 3", 0, vec![(0., 2.), (3., 6.), (8., 9.)]),
        ("2 2", 1, vec![(0., 1.), (3., 5.), (7., 9.)]),
        ("2 2", -1, vec![(1., 3.), (5., 7.), (9., 10.)]),
    ] {
        let svg = build(&format!(
            r#"<svg><path fill="none" stroke="red" stroke-dasharray="{pattern}" stroke-dashoffset="{offset}" d="M0 0H10 M0 5H10"/></svg>"#
        ));
        let layers = &svg.children()[0].children()[1..];
        assert_eq!(layers.len(), 2 * expected.len());
        for (row, group) in layers.chunks(expected.len()).enumerate() {
            for (dash, (a, b)) in group.iter().zip(&expected) {
                assert!((dash.points()[0][0] - a).abs() < 1e-6);
                assert!((dash.points().last().unwrap()[0] - b).abs() < 1e-6);
                // De Casteljau evaluates a constant coordinate via weighted sums;
                // allow its observed one-ULP rounding, not a scene-space drift.
                let y = row as f64 * 5.;
                assert!((dash.points()[0][1] - y).abs() <= 8. * f64::EPSILON * y.abs().max(1.));
            }
        }
    }
}
#[test]
fn dashed_fill_keeps_closed_outline_and_zero_gaps_merge() {
    let svg = build(
        r#"<svg><path fill="blue" stroke="red" stroke-dasharray="3 0 2 4" d="M0 0H10V10H0Z"/></svg>"#,
    );
    let layers = svg.children()[0].children();
    assert!((area(&layers[0]).abs() - 100.).abs() < 1e-8);
    assert_eq!(layers[0].style().stroke_width, 0.);
    assert!(layers[1..].iter().all(|m| m.style().fill_opacity == 0.));
    let line = build(
        r#"<svg><path fill="none" stroke="red" stroke-dasharray="3 0 2 4" d="M0 0H10"/></svg>"#,
    );
    let dashes = &line.children()[0].children()[1..];
    assert_eq!(dashes.len(), 2);
    assert!((dashes[0].points().last().unwrap()[0] - 5.).abs() < 1e-6);
}
#[test]
fn caps_are_not_inserted_at_a_closed_contour_seam() {
    let svg = build(
        r#"<svg><path fill="none" stroke="red" stroke-dasharray="8 4" stroke-dashoffset="2" d="M0 0H10V10H0Z"/></svg>"#,
    );
    let dashes = &svg.children()[0].children()[1..];
    assert_eq!(dashes.len(), 3);
    assert!(dashes.iter().any(|dash| {
        dash.points()
            .iter()
            .skip(1)
            .take(dash.points().len() - 2)
            .any(|p| *p == [0., 0., 0.])
    }));
}
#[test]
fn curved_dashes_measure_arc_length_not_control_polygon_or_curve_index() {
    let svg = build(
        r#"<svg><path fill="none" stroke="red" stroke-dasharray="1 1" d="M0 0Q0 10 10 10"/></svg>"#,
    );
    let dashes = &svg.children()[0].children()[1..];
    for dash in &dashes[..dashes.len() - 1] {
        assert!((dash.path().unwrap().get_arc_length() - 1.).abs() < 1e-7);
    }
}
#[test]
fn caller_overrides_apply_before_paint_layer_split() {
    let svg = svg_mobject_with_paints(
        br#"<svg><path fill="blue" stroke="red" stroke-dasharray="2 2" d="M0 0H10V10H0Z"/></svg>"#,
        SvgPaintOverrides {
            fill_opacity: Some(0.3),
            stroke_opacity: Some(0.6),
            ..Default::default()
        },
    )
    .unwrap();
    let layers = svg.children()[0].children();
    assert_eq!(layers[0].style().fill_opacity, 0.3);
    for stroke in &layers[1..] {
        assert_eq!(stroke.style().fill_opacity, 0.);
        assert_eq!(stroke.style().stroke_opacity, 0.6);
    }
}
#[test]
fn hostile_patterns_fail_before_unbounded_output_and_defaults_stay_unchanged() {
    for pattern in ["0 1", "1e-300 1e-300"] {
        assert!(svg_mobject_with_paints(format!(r#"<svg><path fill="none" stroke="red" stroke-dasharray="{pattern}" d="M0 0H10"/></svg>"#).as_bytes(),Default::default()).is_err());
    }
    let bytes = br#"<svg><rect width="10" height="8" fill="blue" stroke="red"/></svg>"#;
    assert_eq!(
        svg_mobject_with_paints(bytes, Default::default()).unwrap(),
        fmn_library::svg_mobject(bytes).unwrap()
    );
}

#[test]
fn invisible_channels_retain_geometry_for_later_styling() {
    let source=br#"<svg><path fill-rule="evenodd" fill-opacity="0" stroke="red" stroke-opacity="0" stroke-dasharray="2 2" d="M0 0H10V10H0Z M2 2H8V8H2Z"/></svg>"#;
    let built = svg_mobject_with_paints(
        source,
        SvgPaintOverrides {
            stroke_width: Some(0.),
            ..Default::default()
        },
    )
    .unwrap();
    let layers = built.children()[0].children();
    assert!((area(&layers[0]).abs() - 64.).abs() < 1e-8);
    assert!(layers.len() > 3);
    assert!(
        layers
            .iter()
            .all(|layer| layer.style().fill_opacity == 0. && layer.style().stroke_opacity == 0.)
    );
}

#[test]
fn dense_source_paths_do_not_duplicate_all_collapsed_points_for_every_dash() {
    let mut source =
        String::from(r#"<svg><path fill="none" stroke="red" stroke-dasharray="1 1" d="M0 0"#);
    for x in 1..=200 {
        source.push_str(&format!("L{x} 0"));
    }
    source.push_str(r#""/></svg>"#);
    let built = build(&source);
    let pieces = &built.children()[0].children()[1..];
    assert_eq!(pieces.len(), 100);
    assert!(pieces.iter().all(|piece| piece.points().len() <= 7));
    for (i, piece) in pieces.iter().enumerate() {
        assert!((piece.points()[0][0] - (2 * i) as f64).abs() < 1e-8);
        assert!((piece.points().last().unwrap()[0] - (2 * i + 1) as f64).abs() < 1e-8);
        assert!((piece.path().unwrap().get_arc_length() - 1.).abs() < 1e-8);
    }
}
