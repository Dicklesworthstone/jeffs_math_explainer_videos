//! Textured grids keep topology, durable raster identity, and the one Lumen shader.
use super::*;
use crate::{CameraConfig, ScreenMap, Texture, TextureEncoding, TextureMaterial, Viewport};
use fmn_mobject::{
    ImageColorSpace, ImageResource, ImageSampler, Mobject, RecordBuffer, RecordSchema,
};

const POINTS: [[f32; 3]; 4] = [
    [-3.0, -2.0, 0.0],
    [-3.0, 2.0, 0.0],
    [3.0, -2.0, 0.0],
    [3.0, 2.0, 0.0],
];
const UV: [[f32; 2]; 4] = [[0.0, 1.0], [0.0, 0.0], [1.0, 1.0], [1.0, 0.0]];
const PIXELS: [u8; 16] = [
    255, 0, 0, 255, 0, 255, 0, 128, 0, 0, 255, 0, 255, 255, 255, 255,
];

fn config(threads: usize) -> RetainedFrameRendererConfig {
    RetainedFrameRendererConfig {
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
            LinearRgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
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

fn camera() -> Camera {
    Camera::new(CameraConfig {
        resolution: (32, 18),
        background: config(1).frame.background,
        ..CameraConfig::default()
    })
    .unwrap()
}

fn resource(pixels: Vec<u8>) -> ImageResource {
    ImageResource::rgba8(
        2,
        2,
        pixels,
        ImageColorSpace::Linear,
        ImageSampler::default(),
    )
    .unwrap()
}

fn surface() -> Mobject {
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
    let mut buffer = RecordBuffer::new(schema, 4).unwrap();
    for i in 0..4 {
        buffer.write(i, "point", &POINTS[i]);
        buffer.write(i, "d_normal_point", &[POINTS[i][0], POINTS[i][1], 1.0]);
        buffer.write(i, "im_coords", &UV[i]);
        buffer.write(i, "opacity", &[0.75]);
    }
    let mut mob = Mobject::from_buffer(buffer)
        .with_image_resource(resource(PIXELS.to_vec()))
        .with_render_primitive(RenderPrimitive::SurfaceGrid { resolution: (2, 2) });
    mob.uniforms.shading = [0.0; 3];
    mob
}

#[test]
fn retained_textured_grid_matches_the_existing_texture_shader_at_all_thread_counts() {
    let camera = camera();
    let mut stage = Stage::new();
    let mob = stage.add(surface());
    stage.add_to_scene(mob).unwrap();
    let texture = Texture::from_rgba8(2, 2, &PIXELS, TextureEncoding::Linear).unwrap();
    let vertices = (0..4)
        .map(|i| {
            SurfaceVertex::textured(
                POINTS[i].map(f64::from),
                [0.0, 0.0, 1.0],
                UV[i].map(f64::from),
                0.75,
            )
        })
        .collect();
    let mesh = SurfaceMesh::from_uv_grid(vertices, (2, 2)).unwrap();
    let mut draw = SurfaceDraw::new(&mesh);
    draw.material = SurfaceMaterial::Texture(TextureMaterial::surface(&texture, None));
    draw.shading = [0.0; 3];
    draw.depth_test = false;
    let draws = [ThreeDDraw::Surface(draw)];
    let mut reference = RetainedFrameRenderer::new(config(1)).unwrap();
    ThreeDJob::new(&camera, &draws, config(1).tiling)
        .unwrap()
        .render_into(1, &mut reference.frame)
        .unwrap();
    let mut background = RetainedFrameRenderer::new(config(1)).unwrap();
    background
        .render_with_camera(&Stage::new(), &camera)
        .unwrap();
    assert_ne!(reference.frame().as_bytes(), background.frame().as_bytes());
    for threads in [1, 4, 16] {
        let mut renderer = RetainedFrameRenderer::new(config(threads)).unwrap();
        renderer.render_with_camera(&stage, &camera).unwrap();
        assert_eq!(
            renderer.frame().as_bytes(),
            reference.frame().as_bytes(),
            "{threads}"
        );
        assert_eq!(renderer.plan().images().len(), 1);
        assert_eq!(
            stage.get(mob).unwrap().render_primitive(),
            RenderPrimitive::SurfaceGrid { resolution: (2, 2) }
        );
    }
}

#[test]
fn retained_textured_surface_survives_copy_and_durable_snapshot_and_observes_live_changes() {
    let camera = camera();
    let mut stage = Stage::new();
    let first = stage.add(surface());
    stage.add_to_scene(first).unwrap();
    let second = stage.copy_family(first).unwrap();
    stage.add_to_scene(second).unwrap();
    let snapshot = stage.snapshot();
    let bytes = snapshot.to_bytes().unwrap();
    let decoded = fmn_mobject::Snapshot::from_bytes(&bytes, &stage).unwrap();
    let mut renderer = RetainedFrameRenderer::new(config(1)).unwrap();
    renderer.render_with_camera(&stage, &camera).unwrap();
    let initial = renderer.frame().as_bytes().to_vec();
    assert_eq!(renderer.plan().images().len(), 1);
    stage
        .set_image_resource(second, Some(resource([255, 255, 0, 255].repeat(4))))
        .unwrap();
    renderer.render_with_camera(&stage, &camera).unwrap();
    assert_ne!(renderer.frame().as_bytes(), initial);
    assert_eq!(renderer.plan().images().len(), 2);
    stage.restore(&decoded.snapshot);
    renderer.render_with_camera(&stage, &camera).unwrap();
    assert_eq!(renderer.frame().as_bytes(), initial);
    assert_eq!(renderer.plan().images().len(), 1);
    stage
        .get_mut(second)
        .unwrap()
        .buffer
        .write_range("im_coords", 0, &[0.25; 8]);
    renderer.render_with_camera(&stage, &camera).unwrap();
    assert_ne!(renderer.frame().as_bytes(), initial);
    stage.restore(&snapshot);
    renderer.render_with_camera(&stage, &camera).unwrap();
    assert_eq!(renderer.frame().as_bytes(), initial);
}

#[test]
fn invalid_texture_records_leave_the_last_good_frame_and_resource_table_unchanged() {
    let camera = camera();
    let mut stage = Stage::new();
    let mob = stage.add(surface());
    stage.add_to_scene(mob).unwrap();
    let mut renderer = RetainedFrameRenderer::new(config(1)).unwrap();
    renderer.render_with_camera(&stage, &camera).unwrap();
    let healthy = renderer.frame().as_bytes().to_vec();
    stage
        .get_mut(mob)
        .unwrap()
        .buffer
        .write(0, "im_coords", &[f32::NAN, 0.0]);
    assert!(matches!(
        renderer.render_with_camera(&stage, &camera),
        Err(RetainedFrameRendererError::InvalidPrimitive { .. })
    ));
    assert_eq!(renderer.frame().as_bytes(), healthy);
    assert_eq!(renderer.plan().images().len(), 1);
}
