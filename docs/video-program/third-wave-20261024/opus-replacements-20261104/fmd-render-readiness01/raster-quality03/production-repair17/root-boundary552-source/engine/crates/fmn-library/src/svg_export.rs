//! Native `vmobject_to_svg` (§14.2, fm-ek1 tail): the export twin of
//! [`crate::svg`].
//!
//! The importer turns a parsed `fmn_geom::SvgDocument` into a VMobject
//! family; this module walks a live [`Stage`] in draw order and emits the
//! same document model back out through `fmn_geom`'s hardened emitter. The
//! fidelity ruling carries over from the emitter: every resolved style field
//! is written with a fixed attribute order, so the bytes are deterministic by
//! construction.
//!
//! The mapping is the Reference's (`svg_export.py`): the y-up scene frame is
//! flipped to y-down SVG space by negating `y`, the emitted `viewBox` stays
//! in frame units so relative positions survive, and the pixel arguments set
//! the rendered resolution. Manim's stroke width is in pixels, so it is
//! divided by the frame-to-pixel scale before landing in the frame-unit
//! `viewBox` — exactly the Reference shim's `stroke_scale` arithmetic.
//!
//! Per-record color variation along one path (manim allows per-point
//! `fill_rgba`) flattens to the family's first record, which is what the
//! Reference's `get_fill_color` reads; SVG has one fill per path, so any
//! choice would discard the gradient, and this one matches the pinned
//! shim.

use fmn_geom::{
    FillRule, GeomError, LineCap, LineJoin, Paint, QuadPath, SvgShape, SvgStyle, emit_svg,
};
use fmn_mobject::{Mob, Stage};

use crate::style::VStyle;

/// Emit the stage's point-carrying family as one deterministic SVG document.
///
/// Roots are visited in draw order, each root's family in the stage's
/// depth-first order, matching the Reference's
/// `family_members_with_points` walk. Entries without point records (empty
/// groups, image resources) are skipped, exactly as the Reference skips
/// pointless mobjects.
///
/// # Errors
/// A point run that violates the shared-anchor path invariant
/// ([`GeomError`]) — the same refusal the importer's inverse would raise.
pub fn stage_svg_document(
    stage: &Stage,
    pixel_width: f64,
    pixel_height: f64,
    frame_width: f64,
    frame_height: f64,
) -> Result<String, GeomError> {
    stage_svg_document_with_background(
        stage,
        pixel_width,
        pixel_height,
        frame_width,
        frame_height,
        None,
    )
}

/// Export the same vector document with an optional native camera background.
///
/// The background is an ordinary vector rectangle below the painter sequence;
/// transparent backgrounds add no geometry. Text remains outlined paths and
/// does not depend on fonts installed in the SVG viewer.
///
/// # Errors
/// The same path-invariant refusals as [`stage_svg_document`].
pub fn stage_svg_document_with_background(
    stage: &Stage,
    pixel_width: f64,
    pixel_height: f64,
    frame_width: f64,
    frame_height: f64,
    background: Option<fmn_core::color::LinearRgba>,
) -> Result<String, GeomError> {
    let mut shapes = stage_svg_shapes(stage, pixel_width, frame_width)?;
    let view_box = [
        -frame_width / 2.0,
        -frame_height / 2.0,
        frame_width,
        frame_height,
    ];
    if let Some(background) = background.filter(|color| color.a > 0.0) {
        let x = frame_width / 2.0;
        let y = frame_height / 2.0;
        let path = QuadPath::from_points(vec![
            [-x, -y, 0.0],
            [0.0, -y, 0.0],
            [x, -y, 0.0],
            [x, 0.0, 0.0],
            [x, y, 0.0],
            [0.0, y, 0.0],
            [-x, y, 0.0],
            [-x, 0.0, 0.0],
            [-x, -y, 0.0],
        ])?;
        shapes.insert(
            0,
            SvgShape {
                path,
                style: SvgStyle {
                    fill: Some(Paint::Color(background.to_srgb())),
                    fill_opacity: background.a,
                    stroke: None,
                    fill_rule: FillRule::NonZero,
                    stroke_width: 0.0,
                    stroke_opacity: 0.0,
                    line_cap: LineCap::Butt,
                    line_join: LineJoin::Round,
                    miter_limit: 4.0,
                    stroke_dasharray: Vec::new(),
                    stroke_dashoffset: 0.0,
                    opacity: 1.0,
                },
            },
        );
    }
    Ok(emit_svg(&shapes, pixel_width, pixel_height, Some(view_box)))
}

/// The stage family as resolved SVG shapes in the emitted user space.
///
/// User space here is the frame-unit `viewBox` with `y` negated; the
/// emitter's fixed attribute order keeps the result byte-deterministic.
///
/// # Errors
/// A shared-anchor violation in any walked point run.
pub fn stage_svg_shapes(
    stage: &Stage,
    pixel_width: f64,
    frame_width: f64,
) -> Result<Vec<SvgShape>, GeomError> {
    // Manim stroke widths are pixels; the frame-unit viewBox needs the
    // frame-relative width (the Reference shim's `stroke_scale`).
    let stroke_scale = pixel_width / frame_width;
    let mut shapes = Vec::new();
    for root in stage.roots() {
        for mob in stage.family(*root) {
            append_shape(stage, mob, stroke_scale, &mut shapes)?;
        }
    }
    Ok(shapes)
}

/// One family member: its world-space point run under its first-record style.
fn append_shape(
    stage: &Stage,
    mob: Mob,
    stroke_scale: f64,
    shapes: &mut Vec<SvgShape>,
) -> Result<(), GeomError> {
    // The scene frame is y-up; SVG user space is y-down (the Reference's
    // y negation). The z axis stays zero for the flat 2D camera.
    let Some(points) = stage.get_points(mob) else {
        return Ok(());
    };
    if points.is_empty() {
        return Ok(());
    }
    let flipped: Vec<[f64; 3]> = points
        .iter()
        .map(|point| [point[0], -point[1], 0.0])
        .collect();
    let path = QuadPath::from_points(flipped)?;

    let fill_opacity = stage.get_fill_opacity(mob).unwrap_or(0.0);
    let fill = if fill_opacity > 0.0 {
        stage.get_fill_color(mob).map(Paint::Color)
    } else {
        None
    };
    let stroke_width = stage.get_stroke_width(mob).unwrap_or(0.0) / stroke_scale;
    let stroke_opacity = stage.get_stroke_opacity(mob).unwrap_or(0.0);
    let stroke = if stroke_opacity > 0.0 && stroke_width > 0.0 {
        stage.get_stroke_color(mob).map(Paint::Color)
    } else {
        None
    };
    let mut shape = SvgShape {
        path,
        style: SvgStyle {
            fill,
            fill_opacity,
            // Manim fills are winding fills (the Reference's
            // nonzero default), carried through unchanged.
            fill_rule: FillRule::NonZero,
            stroke,
            stroke_width,
            stroke_opacity,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Round,
            miter_limit: 4.0,
            stroke_dasharray: Vec::new(),
            stroke_dashoffset: 0.0,
            opacity: 1.0,
        },
    };
    if stage.uniforms(mob).is_some_and(|u| u.stroke_behind)
        && shape.style.fill.is_some()
        && shape.style.stroke.is_some()
    {
        // SVG normally paints fill then stroke. Two adjacent native shapes
        // preserve the opposite pass order without relying on viewer-specific
        // paint-order support, and keep translucent strokes below their fill.
        let mut stroke = shape.clone();
        stroke.style.fill = None;
        shapes.push(stroke);
        shape.style.stroke = None;
    }
    shapes.push(shape);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Style;
    use crate::vmobject::VMobject;
    use fmn_core::color::Srgb;
    use fmn_geom::SvgDocument;
    use fmn_mobject::Mobject;

    fn styled_circle() -> VMobject {
        crate::Circle::new()
            .radius(1.0)
            .build()
            .map_style(|style| Style {
                fill_color: Srgb {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                },
                fill_opacity: 0.5,
                stroke_color: Srgb {
                    r: 0.0,
                    g: 0.0,
                    b: 1.0,
                },
                stroke_opacity: 1.0,
                stroke_width: 4.0,
                ..style
            })
    }

    #[test]
    fn stage_export_round_trips_through_the_importer() {
        let mut stage = Stage::new();
        let mob = stage.add(styled_circle());
        stage.add_to_scene(mob).expect("root circle");
        stage.set_fill(
            mob,
            Some(Srgb {
                r: 1.0,
                g: 0.0,
                b: 0.0,
            }),
            Some(0.5),
            None,
            true,
        );
        stage.set_stroke(
            mob,
            Some(Srgb {
                r: 0.0,
                g: 0.0,
                b: 1.0,
            }),
            Some(4.0),
            Some(1.0),
            None,
            true,
        );

        let document =
            stage_svg_document(&stage, 640.0, 360.0, 640.0 / 360.0 * 8.0, 8.0).expect("emit");
        let parsed = SvgDocument::parse(document.as_bytes()).expect("re-parse exported SVG");
        assert_eq!(parsed.shapes.len(), 1);
        let shape = &parsed.shapes[0];
        // The y-up frame flips to y-down user space; after the importer's
        // viewBox mapping the scene origin lands on the pixel-space center.
        let anchors = shape.path.points();
        let mut min = [f64::INFINITY; 2];
        let mut max = [f64::NEG_INFINITY; 2];
        for point in anchors.iter().filter(|p| p[2] == 0.0) {
            min[0] = min[0].min(point[0]);
            min[1] = min[1].min(point[1]);
            max[0] = max[0].max(point[0]);
            max[1] = max[1].max(point[1]);
        }
        assert!(
            ((min[0] + max[0]) / 2.0 - 320.0).abs() < 1e-6,
            "x center {mid_x}",
            mid_x = (min[0] + max[0]) / 2.0
        );
        assert!(
            ((min[1] + max[1]) / 2.0 - 180.0).abs() < 1e-6,
            "y center {mid_y}",
            mid_y = (min[1] + max[1]) / 2.0
        );
        // Colors round-trip within the emitter's 8-bit fidelity limit.
        let fill = match &shape.style.fill {
            Some(Paint::Color(color)) => *color,
            other => panic!("expected a flat fill paint, got {other:?}"),
        };
        assert!((fill.r - 1.0).abs() < 0.01, "red channel {r}", r = fill.r);
        assert!((fill.b).abs() < 0.01, "blue channel {b}", b = fill.b);
        assert!((shape.style.fill_opacity - 0.5).abs() < 1e-9);

        // Manim's 4 px stroke is divided by the frame-to-pixel scale for
        // the frame-unit viewBox, and the importer rescales it back during
        // parse: the round trip recovers the original pixel width exactly.
        assert!(
            (shape.style.stroke_width - 4.0).abs() < 1e-9,
            "stroke width {w}",
            w = shape.style.stroke_width
        );
        assert!(shape.style.stroke.is_some());
    }
    #[test]
    fn stage_export_is_deterministic_and_skips_pointless_entries() {
        let mut stage = Stage::new();
        let mob = stage.add(styled_circle());
        stage.add_to_scene(mob).expect("root circle");
        let empty_group = stage.add(VMobject::new());
        stage.add_to_scene(empty_group).expect("root empty group");

        let frame_width = 640.0 / 360.0 * 8.0;
        let first = stage_svg_document(&stage, 640.0, 360.0, frame_width, 8.0).expect("emit");
        let second = stage_svg_document(&stage, 640.0, 360.0, frame_width, 8.0).expect("emit");
        assert_eq!(first, second, "fixed attribute order pins the bytes");

        let parsed = SvgDocument::parse(first.as_bytes()).expect("parse");
        assert_eq!(parsed.shapes.len(), 1, "pointless entries are skipped");

        let empty = Stage::new();
        let bare = stage_svg_document(&empty, 640.0, 360.0, frame_width, 8.0).expect("emit");
        let bare_parsed = SvgDocument::parse(bare.as_bytes()).expect("parse empty");
        assert!(bare_parsed.shapes.is_empty());
    }

    #[test]
    fn export_survives_placed_geometry() {
        // Placement is applied by Stage::get_points, so a shifted mobject
        // exports at its world position without caller-side math.
        let mut stage = Stage::new();
        let circle = styled_circle().shifted([2.0, -1.0, 0.0]);
        let mob = stage.add(circle);
        stage.add_to_scene(mob).expect("root circle");
        let frame_width = 640.0 / 360.0 * 8.0;
        let document = stage_svg_document(&stage, 640.0, 360.0, frame_width, 8.0).expect("emit");
        let parsed = SvgDocument::parse(document.as_bytes()).expect("parse");
        assert_eq!(parsed.shapes.len(), 1);
        assert!(parsed.shapes[0].path.points()[0][0] > 0.0, "x survives");
    }

    // The stage owns value mobjects through `Into<Mobject>`; this assertion
    // keeps the module honest about the type it consumes.
    #[test]
    fn mobject_round_trip_stays_available() {
        let mobject = Mobject::from_points(&[[0.0, 0.0, 0.0], [1.0, 1.0, 0.0]]);
        assert_eq!(mobject.buffer.len(), 2);
    }

    #[test]
    fn native_background_is_below_vectors_and_can_be_transparent() {
        let mut stage = Stage::new();
        let mob = stage.add(styled_circle());
        stage.add_to_scene(mob).unwrap();
        let black = Srgb::from_rgb8(0, 0, 0).to_linear(0.5);
        let document =
            stage_svg_document_with_background(&stage, 160.0, 90.0, 16.0, 9.0, Some(black))
                .unwrap();
        let parsed = SvgDocument::parse(document.as_bytes()).unwrap();
        assert_eq!(parsed.shapes.len(), 2);
        assert_eq!(
            parsed.shapes[0].style.fill,
            Some(Paint::Color(Srgb::from_rgb8(0, 0, 0)))
        );
        assert_eq!(parsed.shapes[0].style.fill_opacity, 0.5);
        assert!(parsed.shapes[0].style.stroke.is_none());
        let transparent = stage_svg_document_with_background(
            &stage,
            160.0,
            90.0,
            16.0,
            9.0,
            Some(Srgb::from_rgb8(0, 0, 0).to_linear(0.0)),
        )
        .unwrap();
        assert_eq!(
            transparent,
            stage_svg_document(&stage, 160.0, 90.0, 16.0, 9.0).unwrap()
        );
    }

    #[test]
    fn native_svg_preserves_stroke_behind_with_two_adjacent_passes() {
        let mut stage = Stage::new();
        let mob = stage.add(styled_circle());
        stage.add_to_scene(mob).unwrap();
        stage.uniforms_mut(mob).unwrap().stroke_behind = true;
        let shapes = stage_svg_shapes(&stage, 160.0, 16.0).unwrap();
        assert_eq!(shapes.len(), 2);
        assert!(shapes[0].style.fill.is_none());
        assert!(shapes[0].style.stroke.is_some());
        assert!(shapes[1].style.fill.is_some());
        assert!(shapes[1].style.stroke.is_none());
        assert_eq!(shapes[0].path.points(), shapes[1].path.points());
        stage.uniforms_mut(mob).unwrap().stroke_behind = false;
        assert_eq!(stage_svg_shapes(&stage, 160.0, 16.0).unwrap().len(), 1);
    }
}
