//! Actual retained records, analytic kernels and pixels, without shader doubles.
use fmn_core::color::LinearRgba;
use fmn_mobject::{Mob, Mobject, Placement, RecordBuffer, RecordSchema, Stage};
use fmn_render::{
    Binning, Camera, CameraConfig, EngineIdentity, FrameConfig, MonoTable, RenderPlan,
    RenderPlanLimits, RetainedFrameRenderer, RetainedFrameRendererConfig,
    RetainedFrameRendererError, ScreenMap, StrokeKnot, StrokeProfile, Style, StyleTable, SyncError,
    Tiling, VectorFrameCompiler, Viewport,
};

fn object(points: &[[f32; 3]], widths: &[f32]) -> Mobject {
    let mut buffer = RecordBuffer::new(RecordSchema::vmobject(), points.len()).unwrap();
    for (i, point) in points.iter().enumerate() {
        buffer.write(i, "point", point);
        buffer.write(i, "stroke_width", &[widths[i]]);
        buffer.write(i, "stroke_rgba", &[0.0, 0.0, 1.0, 1.0]);
    }
    Mobject::from_buffer(buffer)
}
fn fixture() -> (Stage, Mob) {
    let mut stage = Stage::new();
    let mob = stage.add(object(
        &[
            [-3., 0., 0.],
            [-1.5, 0., 0.],
            [0., 0., 0.],
            [1.5, 0., 0.],
            [3., 0., 0.],
        ],
        &[0., 40., 80., 40., 0.],
    ));
    stage.add_to_scene(mob).unwrap();
    (stage, mob)
}
fn renderer(threads: usize) -> (RetainedFrameRenderer, Camera) {
    let background = LinearRgba {
        r: 0.,
        g: 0.,
        b: 0.,
        a: 1.,
    };
    let config = RetainedFrameRendererConfig {
        frame: FrameConfig::new(
            Viewport {
                width: 64,
                height: 36,
            },
            ScreenMap {
                scale: 4.5,
                origin: [32., 18.],
                y_up: false,
            },
            background,
        ),
        tiling: Tiling {
            macro_tile: 16,
            fine_tile: 8,
        },
        engine: EngineIdentity::certified(),
        threads,
    };
    (
        RetainedFrameRenderer::new(config).unwrap(),
        Camera::new(CameraConfig {
            resolution: (64, 36),
            background,
            ..Default::default()
        })
        .unwrap(),
    )
}
fn blue(frame: &fmn_frame::FrameBuffer) -> f64 {
    frame
        .plane(0)
        .as_chunks::<8>()
        .0
        .iter()
        .map(|p| fmn_frame::half::f16_to_f64(u16::from_le_bytes([p[4], p[5]])))
        .sum()
}
fn style(plan: &RenderPlan) -> &Style {
    plan.styles()
        .get(plan.shapes().instances()[0].style)
        .unwrap()
}

#[test]
fn interior_records_survive_endpoint_zeroes_and_reach_both_cpu_cameras() {
    let (stage, _) = fixture();
    for camera_route in [false, true] {
        let (mut render, camera) = renderer(1);
        if camera_route {
            render.render_with_camera(&stage, &camera).unwrap();
        } else {
            render.render(&stage, 0).unwrap();
        }
        assert!(
            blue(render.frame()) > 1.0,
            "interior stroke vanished on camera route {camera_route}"
        );
        let row = style(render.plan());
        let profile = row.stroke_profile.as_ref().unwrap();
        assert_eq!(profile.knots().len(), 5);
        assert_eq!(row.stroke_width_at(0.5), 80.0);
        assert_eq!(row.stroke_width_at(0.0), 0.0);
        assert_eq!(row.stroke_width_at(1.0), 0.0);
        let expected = render.frame().as_bytes().to_vec();
        for threads in [2, 4, 16] {
            let (mut replay, camera) = renderer(threads);
            if camera_route {
                replay.render_with_camera(&stage, &camera).unwrap();
            } else {
                replay.render(&stage, 0).unwrap();
            }
            assert_eq!(replay.frame().as_bytes(), expected);
        }
    }
}

#[test]
fn interior_alpha_is_not_culled_when_both_ends_are_transparent() {
    let (mut stage, mob) = fixture();
    for i in 0..5 {
        stage
            .get_mut(mob)
            .unwrap()
            .buffer
            .write(i, "stroke_width", &[80.]);
        stage.get_mut(mob).unwrap().buffer.write(
            i,
            "stroke_rgba",
            &[0., 0., 1., if i == 2 { 1. } else { 0. }],
        );
    }
    let (mut render, camera) = renderer(1);
    render.render(&stage, 0).unwrap();
    assert!(blue(render.frame()) > 1.0);
    render.render_with_camera(&stage, &camera).unwrap();
    assert!(blue(render.frame()) > 1.0);
}

#[test]
fn interior_style_edits_invalidate_pixels_identity_and_snapshot_not_geometry() {
    let (mut stage, mob) = fixture();
    let (mut render, _) = renderer(1);
    render.render(&stage, 0).unwrap();
    let before = render.frame().as_bytes().to_vec();
    let snapshot = fmn_render::snapshot::encode(render.plan()).unwrap();
    let shape = render.plan().shapes().instances()[0].shape;
    stage
        .get_mut(mob)
        .unwrap()
        .buffer
        .write(2, "stroke_rgba", &[0., 1., 0., 1.]);
    render.render(&stage, 0).unwrap();
    assert_ne!(render.frame().as_bytes(), before);
    assert_ne!(
        fmn_render::snapshot::encode(render.plan()).unwrap(),
        snapshot
    );
    assert_eq!(render.plan().shapes().instances()[0].shape, shape);
    assert_eq!(render.plan().stats().shapes_compiled, 0);
    let expected = render.frame().as_bytes().to_vec();
    render.render(&stage, 0).unwrap();
    assert_eq!(render.frame().as_bytes(), expected);
}

#[test]
fn true_arc_stations_follow_geometry_and_anisotropic_placement() {
    let mut stage = Stage::new();
    let mob = stage.add(object(
        &[
            [0., 0., 0.],
            [0.5, 0., 0.],
            [1., 0., 0.],
            [1., 0.5, 0.],
            [1., 1., 0.],
        ],
        &[0., 40., 80., 40., 0.],
    ));
    stage.add_to_scene(mob).unwrap();
    let mut plan = RenderPlan::default();
    plan.sync(&stage, 0).unwrap();
    assert_eq!(
        style(&plan).stroke_profile.as_ref().unwrap().knots()[2].s,
        0.5
    );
    let shape = plan.shapes().instances()[0].shape;
    stage
        .set_placement(
            mob,
            Placement::new([[3., 0., 0.], [0., 1., 0.], [0., 0., 1.]], [0.; 3]),
        )
        .unwrap();
    let stats = plan.sync(&stage, 0).unwrap();
    assert_eq!(stats.shapes_compiled, 0);
    assert_eq!(plan.shapes().instances()[0].shape, shape);
    assert_eq!(
        style(&plan).stroke_profile.as_ref().unwrap().knots()[2].s,
        0.75
    );
    stage
        .get_mut(mob)
        .unwrap()
        .buffer
        .write(4, "point", &[1., 3., 0.]);
    plan.sync(&stage, 0).unwrap();
    assert_eq!(
        style(&plan).stroke_profile.as_ref().unwrap().knots()[2].s,
        0.5
    );
}

#[test]
fn subpath_breaks_keep_one_sided_colors_without_drawing_a_bridge() {
    let mut stage = Stage::new();
    let points = [
        [-3., 0., 0.],
        [-2.5, 0., 0.],
        [-2., 0., 0.],
        [-2., 0., 0.],
        [2., 0., 0.],
        [2.5, 0., 0.],
        [3., 0., 0.],
    ];
    let mob = stage.add(object(&points, &[80.; 7]));
    for i in 0..3 {
        stage
            .get_mut(mob)
            .unwrap()
            .buffer
            .write(i, "stroke_rgba", &[1., 0., 0., 1.]);
    }
    stage.add_to_scene(mob).unwrap();
    let (mut render, _) = renderer(1);
    render.render(&stage, 0).unwrap();
    let row = style(render.plan());
    assert_eq!(row.stroke_color_at(0.5), [0., 0., 1., 1.]);
    let left = row.stroke_endpoint_parameter(0.5, true);
    assert_eq!(row.stroke_color_at(left), [1., 0., 0., 1.]);
    assert_eq!(render.plan().shapes().shapes()[0].segment_count, 2);
    let frame = render.frame();
    let offset = 18 * frame.layout().stride(0) + 32 * 8;
    assert_eq!(&frame.plane(0)[offset..offset + 6], &[0; 6]);
}

#[test]
fn bad_profile_and_knot_budget_refuse_without_replacing_last_good_plan() {
    let (mut stage, mob) = fixture();
    let mut plan = RenderPlan::default();
    plan.sync(&stage, 0).unwrap();
    let before = fmn_render::snapshot::encode(&plan).unwrap();
    let error = plan
        .sync_with_limits(
            &stage,
            0,
            RenderPlanLimits {
                max_retained_profile_knots: 4,
                ..Default::default()
            },
        )
        .unwrap_err();
    assert!(error.to_string().contains("profile knots"));
    assert_eq!(fmn_render::snapshot::encode(&plan).unwrap(), before);
    stage
        .get_mut(mob)
        .unwrap()
        .buffer
        .write(2, "stroke_width", &[f32::NAN]);
    assert!(
        plan.sync(&stage, 0)
            .unwrap_err()
            .to_string()
            .contains("stroke profile")
    );
    assert_eq!(fmn_render::snapshot::encode(&plan).unwrap(), before);
}

#[test]
fn profile_interning_distinguishes_interior_paint_and_counts_shared_knots_once() {
    let make = |width| {
        Style::default().with_stroke_profile(
            StrokeProfile::new(vec![
                StrokeKnot {
                    s: 0.,
                    width: 0.,
                    rgba: [1.; 4],
                },
                StrokeKnot {
                    s: 0.5,
                    width,
                    rgba: [1.; 4],
                },
                StrokeKnot {
                    s: 1.,
                    width: 0.,
                    rgba: [1.; 4],
                },
            ])
            .unwrap(),
        )
    };
    let mut styles = StyleTable::default();
    assert_eq!(
        styles.intern(make(10.)).unwrap(),
        styles.intern(make(10.)).unwrap()
    );
    assert_eq!(styles.profile_knots(), 3);
    assert_ne!(
        styles.intern(make(10.)).unwrap(),
        styles.intern(make(11.)).unwrap()
    );
    assert_eq!(styles.profile_knots(), 6);
}

#[test]
fn uniform_strokes_do_not_acquire_geometry_dependencies_or_profile_storage() {
    let (mut stage, mob) = fixture();
    for i in 0..5 {
        stage
            .get_mut(mob)
            .unwrap()
            .buffer
            .write(i, "stroke_width", &[20.]);
    }
    let mut plan = RenderPlan::default();
    plan.sync(&stage, 0).unwrap();
    assert!(style(&plan).stroke_profile.is_none());
    let row = plan.shapes().instances()[0].style;
    stage.stretch(mob, 2., 0);
    let stats = plan.sync(&stage, 0).unwrap();
    assert_eq!(stats.styles_rebuilt, 0);
    assert_eq!(plan.shapes().instances()[0].style, row);
    assert_eq!(plan.styles().profile_knots(), 0);
}

// A scene's declared retained budget, on the shape that first crossed the default:
// a picture of rows, each a stroke with paint on every record.

const KNOTS: &str = "retained profile knots";

/// Knots in a row of `n` cells: lead-in, half cell, `n - 1` runs and half cell, tail, sharing
/// their anchors, and one more at each edge of the picture, where a null curve makes alpha jump.
fn knots_in_row(n: u64) -> u64 {
    2 * n + 9
}

fn knots(max_retained_profile_knots: u64) -> RenderPlanLimits {
    RenderPlanLimits {
        max_retained_profile_knots,
        ..RenderPlanLimits::default()
    }
}

/// The record's points along one row: a polyline through each cell's centre, with a null curve
/// at each edge of the picture.
fn row_points(left: f32, y: f32, cell: f32, n: usize) -> Vec<[f32; 3]> {
    let at = |x: f32| [x, y, 0.0];
    let q = cell / 4.0;
    let right = left + cell * n as f32;
    let centre = |k: usize| left + cell * (k as f32 + 0.5);
    let mut p = vec![
        at(left - 2.0 * q),
        at(left - q),
        at(left),
        at(left),
        at(left),
    ];
    p.extend([at((left + centre(0)) / 2.0), at(centre(0))]);
    for k in 1..n {
        p.extend([at((centre(k - 1) + centre(k)) / 2.0), at(centre(k))]);
    }
    p.extend([
        at((centre(n - 1) + right) / 2.0),
        at(right),
        at(right),
        at(right),
    ]);
    p.extend([at(right + q), at(right + 2.0 * q)]);
    p
}

/// Clear caps, opaque inside, and no two rows or frames alike.
fn row_rgba(row: usize, n: usize, frame: usize) -> Vec<[f32; 4]> {
    let paint = |k: usize, a: f32| {
        [
            (row as f32 + 1.0) / 4096.0,
            (k as f32 + 1.0) / 4096.0,
            (frame as f32 + 1.0) / 64.0,
            a,
        ]
    };
    let mut c = vec![paint(0, 0.0); 3];
    c.extend([paint(0, 1.0); 4]);
    for k in 1..n {
        c.extend([paint(2 * k - 1, 1.0), paint(2 * k, 1.0)]);
    }
    c.extend([paint(2 * n, 1.0); 2]);
    c.extend([paint(2 * n, 0.0); 4]);
    c
}

fn plate(rows: usize, n: usize, cell: f32) -> (Stage, Vec<Mob>) {
    let mut stage = Stage::new();
    let top = cell * rows as f32 / 2.0;
    let left = -cell * n as f32 / 2.0;
    let mut mobs = Vec::with_capacity(rows);
    for row in 0..rows {
        let points = row_points(left, top - cell * (row as f32 + 0.5), cell, n);
        let rgba = row_rgba(row, n, 0);
        assert_eq!(points.len(), rgba.len());
        let mut buffer = RecordBuffer::new(RecordSchema::vmobject(), points.len()).unwrap();
        for (i, point) in points.iter().enumerate() {
            buffer.write(i, "point", point);
            buffer.write(i, "stroke_width", &[cell * 100.0]);
            buffer.write(i, "stroke_rgba", &rgba[i]);
        }
        let mob = stage.add(Mobject::from_buffer(buffer));
        stage.add_to_scene(mob).unwrap();
        mobs.push(mob);
    }
    (stage, mobs)
}

fn repaint(stage: &mut Stage, mobs: &[Mob], n: usize, frame: usize) {
    for (row, &mob) in mobs.iter().enumerate() {
        let buffer = &mut stage.get_mut(mob).unwrap().buffer;
        for (i, rgba) in row_rgba(row, n, frame).iter().enumerate() {
            buffer.write(i, "stroke_rgba", rgba);
        }
    }
}

fn config(threads: usize) -> RetainedFrameRendererConfig {
    RetainedFrameRendererConfig {
        frame: FrameConfig::new(
            Viewport {
                width: 64,
                height: 36,
            },
            ScreenMap {
                scale: 4.5,
                origin: [32., 18.],
                y_up: false,
            },
            LinearRgba {
                r: 0.,
                g: 0.,
                b: 0.,
                a: 1.,
            },
        ),
        tiling: Tiling {
            macro_tile: 16,
            fine_tile: 8,
        },
        engine: EngineIdentity::certified(),
        threads,
    }
}

#[test]
fn the_default_stays_where_every_existing_render_has_it() {
    let limits = RenderPlanLimits::default();
    assert_eq!(limits.max_retained_profile_knots, 1 << 20);
    assert_eq!(limits.max_input_curves, 1 << 20);
    assert_eq!(limits.max_retained_segments, 1 << 20);
}

#[test]
fn the_knot_boundary_is_exact_and_a_refusal_changes_nothing() {
    let (rows, n) = (16, 16);
    let (stage, _) = plate(rows, n, 0.25);
    let per_row = knots_in_row(n as u64);
    let total = per_row * rows as u64;
    let mut plan = RenderPlan::new();
    let before = (plan.styles().len(), plan.segments().len(), plan.epoch());
    for (limit, requested) in [(total - 1, total), (per_row * 3 - 1, per_row * 3)] {
        assert_eq!(
            plan.sync_with_limits(&stage, 0, knots(limit)).unwrap_err(),
            SyncError::LimitExceeded {
                resource: KNOTS,
                requested,
                limit,
            }
        );
        assert_eq!(
            (plan.styles().len(), plan.segments().len(), plan.epoch()),
            before
        );
    }
    plan.sync_with_limits(&stage, 0, knots(total)).unwrap();
    assert_eq!(plan.styles().profile_knots() as u64, total);
}

/// The failure this budget exists for, at its real size: 731 rows of 731 cells, as the z² still
/// of `PLATE_CELLS4` asked for at row 713, then what the whole picture asks of each later stage.
#[test]
fn the_picture_that_crossed_the_default_is_admitted_by_a_declared_budget() {
    let (rows, n) = (731, 731);
    let (stage, _) = plate(rows, n, 0.00875);
    let mut plan = RenderPlan::new();
    assert_eq!(
        plan.sync(&stage, 0).unwrap_err(),
        SyncError::LimitExceeded {
            resource: KNOTS,
            requested: 1_048_823,
            limit: 1_048_576,
        }
    );

    let stats = plan.sync_with_limits(&stage, 0, knots(1 << 22)).unwrap();
    assert_eq!(plan.styles().profile_knots(), 1_075_301);
    // the rows differ only in where they sit, and a shape's digest leaves position out: one
    // outline, placed 731 times
    assert_eq!(plan.segments().len(), 734);
    assert_eq!(plan.shapes().shapes().len(), 1);
    assert_eq!(stats.shapes_shared, 730);
    assert_eq!(plan.shapes().instances().len(), 731);

    for (width, height) in [(1280_u32, 720_u32), (3840, 2160)] {
        let map = ScreenMap {
            scale: f64::from(height) / 8.0,
            origin: [f64::from(width) / 2.0, f64::from(height) / 2.0],
            y_up: true,
        };
        MonoTable::build(&plan, map).expect("monotone table holds the picture");
        Binning::build(
            &plan,
            Viewport { width, height },
            Tiling {
                macro_tile: 128,
                fine_tile: 16,
            },
            map,
        )
        .expect("binning holds the picture");
    }
}

/// Nothing is unreachable as a repaint begins straight after a compaction, so old and new rows
/// must fit together. One knot under twice the live demand is a named refusal, not a compaction.
#[test]
fn a_repaint_needs_twice_the_live_knots_and_says_so_when_it_has_less() {
    let (rows, n) = (4, 4);
    let live = knots_in_row(n as u64) * rows as u64;
    let (mut stage, mobs) = plate(rows, n, 1.0);
    let mut plan = RenderPlan::new();
    let tight = knots(2 * live - 1);
    plan.sync_with_limits(&stage, 0, tight).unwrap();
    repaint(&mut stage, &mobs, n, 1);
    assert_eq!(
        plan.sync_with_limits(&stage, 0, tight).unwrap_err(),
        SyncError::LimitExceeded {
            resource: KNOTS,
            requested: 2 * live,
            limit: 2 * live - 1,
        }
    );
    assert_eq!(plan.styles().profile_knots() as u64, live);

    let enough = knots(2 * live);
    assert!(
        plan.sync_with_limits(&stage, 0, enough)
            .unwrap()
            .compaction
            .is_none()
    );
    assert_eq!(plan.styles().profile_knots() as u64, 2 * live);
    repaint(&mut stage, &mobs, n, 2);
    assert!(
        plan.sync_with_limits(&stage, 0, enough)
            .unwrap()
            .compaction
            .is_some()
    );
    assert_eq!(plan.styles().profile_knots() as u64, live);
}

#[test]
fn a_budget_moves_compaction_and_never_the_pixels() {
    let (rows, n) = (4, 4);
    let live = knots_in_row(n as u64) * rows as u64;
    let (mut stage, mobs) = plate(rows, n, 1.0);
    let mut tight = RetainedFrameRenderer::new(config(1)).unwrap();
    tight.set_plan_limits(knots(2 * live));
    let mut wide = RetainedFrameRenderer::new(config(1)).unwrap();
    let mut epochs = Vec::new();
    let mut last = Vec::new();
    for frame in 0..6 {
        repaint(&mut stage, &mobs, n, frame);
        tight.render(&stage, 0).unwrap();
        wide.render(&stage, 0).unwrap();
        assert_eq!(tight.frame().as_bytes(), wide.frame().as_bytes(), "{frame}");
        assert_ne!(
            tight.frame().as_bytes(),
            last.as_slice(),
            "{frame} repaints"
        );
        last = tight.frame().as_bytes().to_vec();
        epochs.push((tight.plan().epoch().get(), wide.plan().epoch().get()));
    }
    assert!(epochs.last().unwrap().0 > 0, "the tight budget compacted");
    assert_eq!(epochs.last().unwrap().1, 0, "the default never had to");
}

#[test]
fn every_capture_route_answers_to_the_declared_budget() {
    let (stage, _) = plate(4, 4, 1.0);
    let refused = |error: RetainedFrameRendererError| {
        matches!(
            error,
            RetainedFrameRendererError::Sync(SyncError::LimitExceeded {
                resource: KNOTS,
                ..
            })
        )
    };

    let mut planar = RetainedFrameRenderer::new(config(1)).unwrap();
    planar.render(&stage, 0).unwrap();
    let mut planar = RetainedFrameRenderer::new(config(1)).unwrap();
    planar.set_plan_limits(knots(1));
    assert!(refused(planar.render(&stage, 0).unwrap_err()));

    let camera = Camera::new(CameraConfig {
        resolution: (64, 36),
        background: config(1).frame.background,
        ..Default::default()
    })
    .unwrap();
    let mut projected = RetainedFrameRenderer::new(config(1)).unwrap();
    projected.render_with_camera(&stage, &camera).unwrap();
    let mut projected = RetainedFrameRenderer::new(config(1)).unwrap();
    projected.set_plan_limits(knots(1));
    assert!(refused(
        projected.render_with_camera(&stage, &camera).unwrap_err()
    ));

    let mut compiler = VectorFrameCompiler::new(config(1)).unwrap();
    assert!(compiler.capture(&stage, 0).is_ok());
    let mut compiler = VectorFrameCompiler::new(config(1)).unwrap();
    compiler.set_plan_limits(knots(1));
    assert!(refused(compiler.capture(&stage, 0).err().unwrap()));
}
