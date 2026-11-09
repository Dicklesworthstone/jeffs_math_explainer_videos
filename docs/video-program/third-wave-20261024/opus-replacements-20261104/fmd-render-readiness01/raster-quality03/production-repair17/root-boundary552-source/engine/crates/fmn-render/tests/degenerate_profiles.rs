//! Finite transient geometry must not poison retained fill or stroke profiles.
use fmn_core::color::LinearRgba;
use fmn_mobject::{Mob, Mobject, RecordBuffer, RecordSchema, Stage};
use fmn_render::{
    Camera, CameraConfig, EngineIdentity, FrameConfig, RenderPlan, RetainedFrameRenderer,
    RetainedFrameRendererConfig, ScreenMap, Tiling, Viewport,
};

// Exact f32 records of a nearly retraced quadratic. The old arc-length
// discriminant rounds positive and its logarithm receives zero.
const POINTS: [[f32; 3]; 3] = [
    [0.0; 3],
    [0.4901037, 1.6202474, 1.8808203],
    [0.000014956778, 0.000049446026, 0.00005739808],
];

// A separate f32 reproducer in the XY plane: the no-camera route must not
// silently flatten 3D records. Its old closed-form length also becomes NaN.
const FLAT_POINTS: [[f32; 3]; 3] = [
    [0.0; 3],
    [0.76765776, 1.4143994, 0.0],
    [2.3427056e-5, 4.3164044e-5, 0.0],
];

fn fixture() -> (Stage, Mob) {
    let mut buffer = RecordBuffer::new(RecordSchema::vmobject(), POINTS.len()).unwrap();
    for (index, point) in POINTS.iter().enumerate() {
        buffer.write(index, "point", point);
        let color = if index == 1 {
            [0.0, 0.0, 1.0, 1.0]
        } else {
            [1.0, 0.0, 0.0, 0.25]
        };
        buffer.write(index, "fill_rgba", &color);
        buffer.write(index, "stroke_rgba", &color);
        buffer.write(index, "stroke_width", &[if index == 1 { 8.0 } else { 2.0 }]);
    }
    let mut stage = Stage::new();
    let mob = stage.add(Mobject::from_buffer(buffer));
    stage.add_to_scene(mob).unwrap();
    (stage, mob)
}

fn renderer(threads: usize) -> (RetainedFrameRenderer, Camera) {
    let background = LinearRgba {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    let frame = FrameConfig::new(
        Viewport {
            width: 64,
            height: 36,
        },
        ScreenMap {
            scale: 4.5,
            origin: [32.0, 18.0],
            y_up: false,
        },
        background,
    );
    let renderer = RetainedFrameRenderer::new(RetainedFrameRendererConfig {
        frame,
        tiling: Tiling {
            macro_tile: 16,
            fine_tile: 8,
        },
        engine: EngineIdentity::certified(),
        threads,
    })
    .unwrap();
    let camera = Camera::new(CameraConfig {
        resolution: (64, 36),
        background,
        ..Default::default()
    })
    .unwrap();
    (renderer, camera)
}

fn capture(
    renderer: &mut RetainedFrameRenderer,
    camera: Option<&Camera>,
    stage: &Stage,
) -> Vec<u8> {
    if let Some(camera) = camera {
        renderer.render_with_camera(stage, camera).unwrap();
    } else {
        renderer.render(stage, 0).unwrap();
    }
    renderer.frame().as_bytes().to_vec()
}

#[test]
fn real_record_profiles_keep_finite_ordered_stations_during_retracing() {
    let (mut stage, mob) = fixture();
    let mut plan = RenderPlan::default();
    for exponent in 1..=30 {
        let endpoint = POINTS[1].map(|x| x * 2.0f32.powi(-exponent));
        stage
            .get_mut(mob)
            .unwrap()
            .buffer
            .write(2, "point", &endpoint);
        plan.sync(&stage, 0).unwrap();
        let instance = &plan.shapes().instances()[0];
        let style = plan.styles().get(instance.style).unwrap();
        let fill = style.fill_profile.as_ref().unwrap();
        let stroke = style.stroke_profile.as_ref().unwrap();
        for stations in [
            fill.knots().iter().map(|k| k.s).collect::<Vec<_>>(),
            stroke.knots().iter().map(|k| k.s).collect::<Vec<_>>(),
        ] {
            assert_eq!(stations.len(), 3);
            assert_eq!(stations[0], 0.0);
            assert_eq!(stations[2], 1.0);
            assert!(stations.iter().all(|s| s.is_finite()));
            assert!(stations.windows(2).all(|pair| pair[0] <= pair[1]));
        }
        assert!(fill.has_visible_alpha());
        assert_eq!(stroke.maximum_width(), 8.0);
    }
}

#[test]
fn collapse_and_recovery_match_fresh_frames_across_cameras_and_threads() {
    for camera_path in [false, true] {
        let (mut stage, mob) = fixture();
        let (mut one, camera) = renderer(1);
        let (mut four, _) = renderer(4);
        let camera = camera_path.then_some(&camera);
        let points = if camera_path { &POINTS } else { &FLAT_POINTS };
        let mut first = None;
        let mut collapsed = None;
        for scale in [1.0f32, 0.125, 0.0, 2.0, 1.0] {
            for (index, point) in points.iter().enumerate() {
                let point = point.map(|x| x * scale);
                stage
                    .get_mut(mob)
                    .unwrap()
                    .buffer
                    .write(index, "point", &point);
            }
            let actual = capture(&mut one, camera, &stage);
            assert_eq!(actual, capture(&mut four, camera, &stage));
            let (mut fresh, _) = renderer(1);
            assert_eq!(actual, capture(&mut fresh, camera, &stage));
            if scale == 0.0 {
                collapsed = Some(actual.clone());
            }
            if scale == 1.0 {
                if let Some(ref first) = first {
                    assert_eq!(&actual, first, "expansion retained stale collapsed state");
                } else {
                    first = Some(actual);
                }
            }
        }
        assert_ne!(
            first, collapsed,
            "visible stroke disappeared before collapse"
        );
    }
}

#[test]
fn invalid_live_records_still_refuse_atomically_and_can_recover() {
    let (mut stage, mob) = fixture();
    let mut plan = RenderPlan::default();
    plan.sync(&stage, 0).unwrap();
    let good = fmn_render::snapshot::encode(&plan).unwrap();
    for (field, invalid, restored) in [
        ("point", vec![f32::NAN, 0.0, 0.0], POINTS[1].to_vec()),
        (
            "fill_rgba",
            vec![0.0, f32::INFINITY, 1.0, 1.0],
            vec![0.0, 0.0, 1.0, 1.0],
        ),
        (
            "stroke_rgba",
            vec![0.0, 0.0, f32::NAN, 1.0],
            vec![0.0, 0.0, 1.0, 1.0],
        ),
    ] {
        stage.get_mut(mob).unwrap().buffer.write(1, field, &invalid);
        assert!(plan.sync(&stage, 0).is_err(), "accepted invalid {field}");
        assert_eq!(fmn_render::snapshot::encode(&plan).unwrap(), good);
        stage
            .get_mut(mob)
            .unwrap()
            .buffer
            .write(1, field, &restored);
        plan.sync(&stage, 0).unwrap();
        assert_eq!(fmn_render::snapshot::encode(&plan).unwrap(), good);
    }
}
