//! Real retained-raster witnesses, not a second implementation of the shader.
use fmn_core::color::LinearRgba;
use fmn_frame::FrameBuffer;
use fmn_mobject::{
    ImageColorSpace, ImageResource, ImageSampler, Mobject, RecordBuffer, RecordSchema,
    RenderPrimitive, Snapshot, Stage,
};
use fmn_render::revision::Revisions;
use fmn_render::{
    Camera, CameraConfig, EngineIdentity, FrameConfig, RetainedFrameRenderer,
    RetainedFrameRendererConfig, ScreenMap, Tiling, Viewport,
};

fn image(rgb: [u8; 3]) -> ImageResource {
    ImageResource::rgba8(
        1,
        1,
        vec![rgb[0], rgb[1], rgb[2], 255],
        ImageColorSpace::Linear,
        ImageSampler::default(),
    )
    .unwrap()
}
fn pair(dark: [u8; 3]) -> ImageResource {
    image([255, 0, 0]).with_dark_image(image(dark)).unwrap()
}
fn surface(resource: ImageResource, normal: f32, triangles: bool) -> Mobject {
    let schema = RecordSchema::new(
        &[
            ("point", 3),
            ("d_normal_point", 3),
            ("im_coords", 2),
            ("opacity", 1),
        ],
        &["point"],
        &["point", "d_normal_point"],
    )
    .unwrap();
    let corners = [
        [-2.0, -2.0, 0.0],
        [-2.0, 2.0, 0.0],
        [2.0, -2.0, 0.0],
        [2.0, 2.0, 0.0],
    ];
    let indices: &[usize] = if triangles {
        &[0, 1, 2, 2, 1, 3]
    } else {
        &[0, 1, 2, 3]
    };
    let mut buffer = RecordBuffer::new(schema, indices.len()).unwrap();
    for (row, &index) in indices.iter().enumerate() {
        let point = corners[index];
        buffer.write(row, "point", &point);
        buffer.write(row, "d_normal_point", &[point[0], point[1], normal]);
        buffer.write(row, "im_coords", &[0.5, 0.5]);
        buffer.write(row, "opacity", &[1.0]);
    }
    let mut mob = Mobject::from_buffer(buffer)
        .with_image_resource(resource)
        .with_render_primitive(if triangles {
            RenderPrimitive::TriangleMesh
        } else {
            RenderPrimitive::SurfaceGrid { resolution: (2, 2) }
        });
    mob.uniforms.shading = [0.0; 3];
    mob
}
fn renderer(threads: usize) -> (RetainedFrameRenderer, Camera) {
    let black = LinearRgba {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    let config = RetainedFrameRendererConfig {
        frame: FrameConfig::new(
            Viewport {
                width: 32,
                height: 18,
            },
            ScreenMap {
                scale: 2.25,
                origin: [16.0, 9.0],
                y_up: false,
            },
            black,
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
            resolution: (32, 18),
            background: black,
            ..CameraConfig::default()
        })
        .unwrap(),
    )
}
fn center(frame: &FrameBuffer) -> [f64; 4] {
    let at = 9 * frame.layout().stride(0) + 16 * 8;
    let bytes = &frame.plane(0)[at..at + 8];
    std::array::from_fn(|i| {
        fmn_frame::half::f16_to_f64(u16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]]))
    })
}
#[test]
fn native_pair_selects_light_and_dark_for_grids_and_triangle_meshes() {
    for triangles in [false, true] {
        for normal in [1.0, -1.0] {
            let mut stage = Stage::new();
            let mob = stage.add(surface(pair([0, 0, 255]), normal, triangles));
            stage.add_to_scene(mob).unwrap();
            let (mut render, camera) = renderer(1);
            render.render_with_camera(&stage, &camera).unwrap();
            let pixel = center(render.frame());
            if normal > 0.0 {
                assert!(pixel[0] > 0.9 && pixel[2] < 0.1, "light {pixel:?}");
            } else {
                assert!(pixel[2] > 0.9 && pixel[0] < 0.1, "dark {pixel:?}");
            }
            assert_eq!(render.plan().images().len(), 2);
            let expected = render.frame().as_bytes().to_vec();
            for threads in [4, 16] {
                let (mut replay, camera) = renderer(threads);
                replay.render_with_camera(&stage, &camera).unwrap();
                assert_eq!(replay.frame().as_bytes(), expected);
            }
        }
    }
}
#[test]
fn dark_replacement_invalidates_material_and_releases_unused_image_rows() {
    let mut stage = Stage::new();
    let mob = stage.add(surface(pair([0, 0, 255]), -1.0, false));
    stage.add_to_scene(mob).unwrap();
    let (mut render, camera) = renderer(1);
    render.render_with_camera(&stage, &camera).unwrap();
    let before = Revisions::read(&stage, mob).unwrap();
    stage
        .set_image_resource(mob, Some(pair([0, 255, 0])))
        .unwrap();
    assert_ne!(Revisions::read(&stage, mob).unwrap().image, before.image);
    render.render_with_camera(&stage, &camera).unwrap();
    let pixel = center(render.frame());
    assert!(pixel[1] > 0.9 && pixel[2] < 0.1, "{pixel:?}");
    assert_eq!(render.plan().images().len(), 2);
    stage
        .set_image_resource(
            mob,
            Some(
                image([255, 0, 0])
                    .with_dark_image(image([255, 0, 0]))
                    .unwrap(),
            ),
        )
        .unwrap();
    render.render_with_camera(&stage, &camera).unwrap();
    assert_eq!(
        render.plan().images().len(),
        1,
        "equal descriptors must share one decode"
    );
}
#[test]
fn durable_restore_preserves_pixels_and_dark_identity_at_equal_revision_counters() {
    let mut stage = Stage::new();
    let mob = stage.add(surface(pair([0, 0, 255]), -1.0, false));
    stage.add_to_scene(mob).unwrap();
    let saved = stage.snapshot_bytes().unwrap();
    let mut other = Stage::new();
    let other_mob = other.add(surface(pair([0, 255, 0]), -1.0, false));
    other.add_to_scene(other_mob).unwrap();
    let changed = other.snapshot_bytes().unwrap();
    let counter = stage.get(mob).unwrap().image_revision();
    let key = Revisions::read(&stage, mob).unwrap().image;
    let (mut render, camera) = renderer(1);
    render.render_with_camera(&stage, &camera).unwrap();
    let original = render.frame().as_bytes().to_vec();
    stage.restore(&Snapshot::from_bytes(&changed, &stage).unwrap().snapshot);
    assert_eq!(stage.get(mob).unwrap().image_revision(), counter);
    assert_ne!(Revisions::read(&stage, mob).unwrap().image, key);
    render.render_with_camera(&stage, &camera).unwrap();
    assert!(center(render.frame())[1] > 0.9);
    stage.restore(&Snapshot::from_bytes(&saved, &stage).unwrap().snapshot);
    render.render_with_camera(&stage, &camera).unwrap();
    assert_eq!(render.frame().as_bytes(), original);
}
#[test]
fn image_quad_pair_refusal_does_not_replace_last_good_frame() {
    let mut stage = Stage::new();
    let mob = stage.add(surface(pair([0, 0, 255]), -1.0, false));
    stage.add_to_scene(mob).unwrap();
    let (mut render, camera) = renderer(1);
    render.render_with_camera(&stage, &camera).unwrap();
    let original = render.frame().as_bytes().to_vec();
    let invalid = stage.add(
        surface(pair([0, 255, 0]), -1.0, true).with_render_primitive(RenderPrimitive::ImageQuad),
    );
    stage.add_to_scene(invalid).unwrap();
    let error = render.render_with_camera(&stage, &camera).unwrap_err();
    assert!(error.to_string().contains("light/dark textures require"));
    assert_eq!(render.frame().as_bytes(), original);
}
