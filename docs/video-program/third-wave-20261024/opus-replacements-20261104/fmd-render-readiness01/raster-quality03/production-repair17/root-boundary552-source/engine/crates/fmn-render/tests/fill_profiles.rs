//! Full boundary color through real Stage records, caches and both CPU cameras.
use fmn_core::color::LinearRgba;
use fmn_mobject::{Mob, Mobject, Placement, RecordBuffer, RecordSchema, Stage};
use fmn_render::{
    Camera, CameraConfig, EngineIdentity, FillKnot, FillProfile, FrameConfig, RenderPlan,
    RenderPlanLimits, RetainedFrameRenderer, RetainedFrameRendererConfig, ScreenMap, Style,
    StyleTable, Tiling, Viewport,
};

fn fixture() -> (Stage, Mob) {
    let points = [
        [-2., -2., 0.],
        [0., -2., 0.],
        [2., -2., 0.],
        [2., 0., 0.],
        [2., 2., 0.],
        [0., 2., 0.],
        [-2., 2., 0.],
        [-2., 0., 0.],
        [-2., -2., 0.],
    ];
    let mut b = RecordBuffer::new(RecordSchema::vmobject(), 9).unwrap();
    for (i, p) in points.iter().enumerate() {
        b.write(i, "point", p);
        b.write(i, "fill_rgba", &[0., 0., 1., if i == 4 { 1. } else { 0. }]);
    }
    let mut stage = Stage::new();
    let mob = stage.add(Mobject::from_buffer(b));
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
    let cfg = RetainedFrameRendererConfig {
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
        RetainedFrameRenderer::new(cfg).unwrap(),
        Camera::new(CameraConfig {
            resolution: (64, 36),
            background,
            ..Default::default()
        })
        .unwrap(),
    )
}
fn channel(frame: &fmn_frame::FrameBuffer, channel: usize) -> f64 {
    frame
        .plane(0)
        .as_chunks::<8>()
        .0
        .iter()
        .map(|p| {
            fmn_frame::half::f16_to_f64(u16::from_le_bytes([p[channel * 2], p[channel * 2 + 1]]))
        })
        .sum()
}
fn style(plan: &RenderPlan) -> &Style {
    plan.styles()
        .get(plan.shapes().instances()[0].style)
        .unwrap()
}
#[test]
fn interior_alpha_and_color_reach_both_cameras_at_every_thread_count() {
    let (stage, _) = fixture();
    for camera_path in [false, true] {
        let mut reference = None;
        for threads in [1, 2, 4, 16] {
            let (mut r, c) = renderer(threads);
            if camera_path {
                r.render_with_camera(&stage, &c).unwrap();
            } else {
                r.render(&stage, 0).unwrap();
            }
            assert!(channel(r.frame(), 2) > 1.0, "interior opacity vanished");
            if let Some(ref expected) = reference {
                assert_eq!(r.frame().as_bytes(), expected);
            } else {
                reference = Some(r.frame().as_bytes().to_vec());
            }
            let p = style(r.plan()).fill_profile.as_ref().unwrap();
            assert_eq!(p.knots().len(), 9);
            assert!(!p.is_opaque());
            assert!(p.has_visible_alpha());
        }
    }
}
#[test]
fn interior_style_edits_reuse_geometry_but_change_pixels_and_snapshot() {
    let (mut stage, mob) = fixture();
    let (mut r, _) = renderer(1);
    r.render(&stage, 0).unwrap();
    let before = r.frame().as_bytes().to_vec();
    let snapshot = fmn_render::snapshot::encode(r.plan()).unwrap();
    let shape = r.plan().shapes().instances()[0].shape;
    stage
        .get_mut(mob)
        .unwrap()
        .buffer
        .write(4, "fill_rgba", &[0., 1., 0., 1.]);
    r.render(&stage, 0).unwrap();
    assert!(channel(r.frame(), 1) > 1.0);
    assert_ne!(r.frame().as_bytes(), before);
    assert_ne!(fmn_render::snapshot::encode(r.plan()).unwrap(), snapshot);
    assert_eq!(r.plan().shapes().instances()[0].shape, shape);
    assert_eq!(r.plan().stats().shapes_compiled, 0);
    let expected = r.frame().as_bytes().to_vec();
    r.render(&stage, 0).unwrap();
    assert_eq!(r.frame().as_bytes(), expected);
}
#[test]
fn affine_placement_and_geometry_refresh_arc_stations() {
    let (mut stage, mob) = fixture();
    let mut p = RenderPlan::default();
    p.sync(&stage, 0).unwrap();
    assert_eq!(style(&p).fill_profile.as_ref().unwrap().knots()[2].s, 0.25);
    stage
        .set_placement(
            mob,
            Placement::new([[3., 0., 0.], [0., 1., 0.], [0., 0., 1.]], [0.; 3]),
        )
        .unwrap();
    let stats = p.sync(&stage, 0).unwrap();
    assert_eq!(stats.shapes_compiled, 0);
    assert_eq!(style(&p).fill_profile.as_ref().unwrap().knots()[2].s, 0.375);
    stage
        .get_mut(mob)
        .unwrap()
        .buffer
        .write(1, "point", &[0., -4., 0.]);
    p.sync(&stage, 0).unwrap();
    assert!(style(&p).fill_profile.as_ref().unwrap().knots()[2].s > 0.375);
}
#[test]
fn opaque_endpoints_do_not_claim_an_opaque_interior() {
    let (mut stage, mob) = fixture();
    for i in 0..9 {
        stage.get_mut(mob).unwrap().buffer.write(
            i,
            "fill_rgba",
            &[1., 0., 0., if i == 4 { 0. } else { 1. }],
        );
    }
    let mut p = RenderPlan::default();
    p.sync(&stage, 0).unwrap();
    assert_eq!(style(&p).fill_rgba[3], 1.);
    assert_eq!(style(&p).fill_rgba_end[3], 1.);
    assert!(!style(&p).has_opaque_fill());
}
#[test]
fn nonfinite_fill_refuses_without_mutating_the_last_good_plan() {
    let (mut stage, mob) = fixture();
    let mut p = RenderPlan::default();
    p.sync(&stage, 0).unwrap();
    let before = fmn_render::snapshot::encode(&p).unwrap();
    stage
        .get_mut(mob)
        .unwrap()
        .buffer
        .write(4, "fill_rgba", &[f32::NAN, 0., 0., 1.]);
    assert!(p.sync(&stage, 0).is_err());
    assert_eq!(fmn_render::snapshot::encode(&p).unwrap(), before);
    stage
        .get_mut(mob)
        .unwrap()
        .buffer
        .write(4, "fill_rgba", &[0., 1., 0., 1.]);
    p.sync(&stage, 0).unwrap();
}
#[test]
fn combined_profile_budget_is_checked_before_retained_publication() {
    let (stage, _) = fixture();
    let mut p = RenderPlan::default();
    let limits = RenderPlanLimits {
        max_retained_profile_knots: 8,
        ..Default::default()
    };
    assert!(p.sync_with_limits(&stage, 0, limits).is_err());
    assert!(p.styles().rows().is_empty());
    p.sync(&stage, 0).unwrap();
    assert_eq!(p.styles().profile_knots(), 9);
}
#[test]
fn styles_do_not_intern_distinct_interior_colors_as_the_same_ramp() {
    let make = |color| {
        FillProfile::new(vec![
            FillKnot {
                s: 0.,
                rgba: [0.; 4],
            },
            FillKnot {
                s: 0.5,
                rgba: color,
            },
            FillKnot {
                s: 1.,
                rgba: [0.; 4],
            },
        ])
        .unwrap()
    };
    let a = Style::default().with_fill_profile(make([1., 0., 0., 1.]));
    let b = Style::default().with_fill_profile(make([0., 0., 1., 1.]));
    let mut t = StyleTable::default();
    let one = t.intern(a.clone()).unwrap();
    assert_eq!(one, t.intern(a).unwrap());
    assert_ne!(one, t.intern(b).unwrap());
    assert_eq!(t.profile_knots(), 6);
}
#[test]
fn legacy_flat_styles_still_avoid_profiles_and_new_snapshot_schema() {
    let (mut stage, mob) = fixture();
    for i in 0..9 {
        stage
            .get_mut(mob)
            .unwrap()
            .buffer
            .write(i, "fill_rgba", &[0.2, 0.3, 0.4, 1.]);
    }
    let mut p = RenderPlan::default();
    p.sync(&stage, 0).unwrap();
    assert!(style(&p).fill_profile.is_none());
    assert!(style(&p).has_opaque_fill());
    assert_eq!(p.styles().profile_knots(), 0);
}
