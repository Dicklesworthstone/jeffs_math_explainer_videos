//! Final-state vector export through Chisel's SVG emitter and Reel publication.
//!
//! No Python serializer, raster fallback, external fonts, or framebuffer is
//! involved. The native scene clock still reaches every semantic endpoint.
//! SVG is an explicitly bounded flat-paint vector format, not a promise to
//! reproduce Lumen's antialiasing or linear-light compositing in a browser.

use super::*;
use fmn_library::VStyle;
use fmn_library::svg_export::stage_svg_document_with_background;
use fmn_mobject::RenderPrimitive;

const MAX_POINTS: usize = 262_144;
const MAX_SHAPES: usize = 32_768;
const MAX_SVG_BYTES: u64 = 64 * 1024 * 1024;

pub(super) struct SvgSession {
    destination: PathBuf,
    camera: Camera,
    threads: usize,
    pub(super) timeline: OutputTimeline,
}

impl SvgSession {
    pub(super) fn new(
        destination: PathBuf,
        width: u32,
        height: u32,
        fps: u32,
        threads: usize,
    ) -> PyResult<(Self, RuntimeConfig)> {
        if width == 0
            || height == 0
            || width > 16384
            || height > 16384
            || fps == 0
            || !(1..=96).contains(&threads)
        {
            return Err(PyValueError::new_err(
                "SVG requires 1..16384 pixel dimensions, positive FPS and 1..96 threads",
            ));
        }
        // Freeze before constructors/lifecycle code can change the process cwd.
        let destination = std::path::absolute(destination).map_err(native_error)?;
        match std::fs::symlink_metadata(&destination) {
            Ok(_) => return Err(PyOSError::new_err("SVG destination already exists")),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(native_error(error)),
        }
        let mut config = fmn_config::Config::resolve(&[], None)
            .map_err(native_error)?
            .config;
        config.camera.resolution = (width, height);
        config.camera.fps = fps;
        let background = fmn_core::color::Srgb::from_hex(&config.camera.background_color)
            .map_err(native_error)?
            .to_linear(config.camera.background_opacity);
        let camera = Camera::new(CameraConfig {
            resolution: (width, height),
            fps,
            background,
            ..CameraConfig::default()
        })
        .map_err(camera_error)?;
        let mut runtime = RuntimeConfig::from_config(&config);
        runtime.skip_animations = true;
        runtime.preview_while_skipping = false;
        Ok((
            Self {
                destination,
                camera,
                threads,
                timeline: OutputTimeline::new(true),
            },
            runtime,
        ))
    }

    pub(super) fn bind_camera(
        &mut self,
        frame: fmn_scene::studio_bridge::CameraFrame,
        background: Option<fmn_core::color::LinearRgba>,
    ) -> PyResult<()> {
        *self.camera.frame_mut() = frame;
        if let Some(background) = background {
            self.camera
                .set_background(background)
                .map_err(camera_error)?;
        }
        Ok(())
    }

    pub(super) fn finish(self, scene: &Scene) -> PyResult<(PortalArtifactReport, String, usize)> {
        let document = scene_document(scene.stage(), &self.camera)?;
        let report = fmn_output::publish_svg_new(
            &fmn_platform::fs::StdFs,
            &fmn_output::SvgPublicationConfig {
                destination: self.destination,
                max_artifact_bytes: MAX_SVG_BYTES,
                profile: None,
            },
            document.as_bytes(),
        )
        .map_err(native_error)?;
        Ok((
            PortalArtifactReport {
                path: report.path,
                frame_count: 1,
                bytes: report.bytes,
                digest: report.digest,
                invocations: Vec::new(),
                audio_inputs: Vec::new(),
                routes: crate::PortalRoutes::default(),
            },
            "native-svg".to_owned(),
            self.threads,
        ))
    }
}

fn unsupported(detail: &str) -> PyErr {
    CapabilityError::new_err(format!(
        "SVG export: {detail}; use PNG for raster/3D rendering"
    ))
}

/// Validate before copying geometry. Never silently turn an image, point
/// cloud, shaded surface or a per-vertex gradient into an ordinary flat path.
fn validate_stage(stage: &Stage) -> PyResult<()> {
    let mut points = 0usize;
    let mut shapes = 0usize;
    for root in stage.roots() {
        for mob in stage.family(*root) {
            let entry = stage
                .get(mob)
                .ok_or_else(|| unsupported("stale scene member"))?;
            if entry.buffer.is_empty() {
                continue;
            }
            shapes = shapes
                .checked_add(1)
                .ok_or_else(|| unsupported("shape budget exceeded"))?;
            points = points
                .checked_add(entry.buffer.len())
                .ok_or_else(|| unsupported("point budget exceeded"))?;
            if shapes > MAX_SHAPES || points > MAX_POINTS {
                return Err(unsupported("vector geometry budget exceeded"));
            }
            if entry.render_primitive() != RenderPrimitive::Vector {
                return Err(unsupported("only quadratic vector families are supported"));
            }
            let uniforms = entry.uniforms();
            if uniforms.depth_test
                || uniforms.shading != [0.0; 3]
                || uniforms.clip_planes != [[0.0; 4]; 4]
            {
                return Err(unsupported(
                    "depth, lighting and user clip planes require Lumen",
                ));
            }
            for (name, width) in [("fill_rgba", 4), ("stroke_rgba", 4), ("stroke_width", 1)] {
                let Some(values) = entry.buffer.read_column(name) else {
                    return Err(unsupported("non-vector record schema"));
                };
                if values.len() != entry.buffer.len() * width
                    || !values.iter().all(|v| v.is_finite())
                {
                    return Err(unsupported("invalid vector style records"));
                }
                if values
                    .chunks_exact(width)
                    .any(|row| row != &values[..width])
                {
                    return Err(unsupported("per-vertex paint and stroke-width variation"));
                }
            }
        }
    }
    Ok(())
}

/// Project a private CoW snapshot once per identity, retaining the original
/// family/painter sequence (including intentional repeated root placements).
fn scene_document(stage: &Stage, camera: &Camera) -> PyResult<String> {
    validate_stage(stage)?;
    let mut projected = stage.snapshot().materialize();
    let (width, height) = camera.pixel_shape();
    let mut seen = HashSet::new();
    for root in stage.roots() {
        for mob in stage.family(*root) {
            if !seen.insert(mob) {
                continue;
            }
            let points = stage
                .get_points(mob)
                .ok_or_else(|| unsupported("stale point records"))?;
            if points.is_empty() {
                continue;
            }
            let uniforms = *stage
                .uniforms(mob)
                .ok_or_else(|| unsupported("stale uniforms"))?;
            let mut weight = None;
            let mut mapped = Vec::new();
            mapped
                .try_reserve_exact(points.len())
                .map_err(native_error)?;
            for point in points {
                let clip = camera.project(point, uniforms.is_fixed_in_frame);
                let w = clip.clip[3];
                // A constant homogeneous weight makes the entire quadratic
                // projection affine. Projecting nonconstant-weight controls as
                // an ordinary SVG Q command would draw a different curve.
                if !clip.clip.iter().all(|v| v.is_finite())
                    || w <= 0.0
                    || clip.clip[2].abs() > w
                    || weight.is_some_and(|old| old != w)
                {
                    return Err(unsupported("perspective-varying or depth-clipped paths"));
                }
                weight = Some(w);
                let pixel = clip
                    .pixel((width, height))
                    .ok_or_else(|| unsupported("invalid camera projection"))?;
                let point = [
                    pixel[0] - f64::from(width) / 2.0,
                    f64::from(height) / 2.0 - pixel[1],
                    0.0,
                ];
                if point
                    .iter()
                    .any(|value| !value.is_finite() || value.abs() > f64::from(f32::MAX))
                {
                    return Err(unsupported(
                        "projected coordinates exceed native record precision",
                    ));
                }
                mapped.push(point);
            }
            projected.set_points(mob, &mapped).map_err(native_error)?;
            if uniforms.scale_stroke_with_zoom {
                let scale = (1.0 - uniforms.is_fixed_in_frame) / camera.frame().scale()
                    + uniforms.is_fixed_in_frame;
                let width = stage.get_stroke_width(mob).unwrap_or(0.0) * scale;
                projected.set_stroke(mob, None, Some(width), None, None, false);
            }
        }
    }
    // The native exporter owns SVG syntax, y inversion, style and pass order.
    // Both frame extents are now pixel units, so its conventional SVG stroke
    // widths remain pixels. No frame buffer or PNG is embedded in the document.
    stage_svg_document_with_background(
        &projected,
        f64::from(width),
        f64::from(height),
        f64::from(width),
        f64::from(height),
        Some(camera.background()),
    )
    .map_err(native_error)
}

/// Production vector output and its observed determinism/failure witnesses.
#[cfg(any(test, feature = "gauntlet"))]
pub struct PortalSvgReport {
    pub document: Vec<u8>,
    pub thread_counts: u64,
    pub failure_paths: u64,
}

#[cfg(any(test, feature = "gauntlet"))]
pub fn run_portal_gauntlet_svg() -> Result<PortalSvgReport, String> {
    crate::with_python_test_module("native SVG output", |py, _module, globals| {
        globals
            .set_item(
                "__file__",
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/svg_output.py"),
            )
            .map_err(|error| error.to_string())?;
        let source = CString::new(include_str!("../tests/svg_output.py"))
            .map_err(|error| error.to_string())?;
        py.run(source.as_c_str(), Some(globals), Some(globals))
            .inspect_err(|error| error.print(py))
            .map_err(|error| error.to_string())?;
        let get = |name: &str| {
            globals
                .get_item(name)
                .map_err(|error| error.to_string())?
                .ok_or_else(|| format!("missing SVG witness: {name}"))
        };
        Ok(PortalSvgReport {
            document: get("_svg_document")?
                .extract()
                .map_err(|error: PyErr| error.to_string())?,
            thread_counts: get("_svg_thread_counts")?
                .extract()
                .map_err(|error: PyErr| error.to_string())?,
            failure_paths: get("_svg_failure_paths")?
                .extract()
                .map_err(|error: PyErr| error.to_string())?,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn svg_projection_preserves_native_geometry_without_mutating_the_scene() {
        let mut stage = Stage::new();
        let shape = stage.add(fmn_library::Square::new().build());
        stage.add_to_scene(shape).unwrap();
        stage.shift(shape, [2.0, 1.0, 0.0]);
        let before = stage.get_points(shape).unwrap();
        let mut camera = Camera::new(CameraConfig {
            resolution: (160, 90),
            ..CameraConfig::default()
        })
        .unwrap();
        camera.frame_mut().set_center([2.0, 1.0, 0.0]).unwrap();
        let svg = scene_document(&stage, &camera).unwrap();
        let parsed = fmn_library::svg::SvgDocument::parse(svg.as_bytes()).unwrap();
        assert_eq!(parsed.shapes.len(), 2);
        assert_eq!(stage.get_points(shape).unwrap(), before);
        let points = parsed.shapes[1].path.points();
        let min_x = points.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min);
        let max_x = points
            .iter()
            .map(|p| p[0])
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(((min_x + max_x) / 2.0 - 80.0).abs() < 1e-6);
        camera.frame_mut().set_height(4.0).unwrap();
        assert_ne!(svg, scene_document(&stage, &camera).unwrap());
    }

    #[test]
    fn svg_refuses_gradient_perspective_and_depth_instead_of_flattening_them() {
        let mut stage = Stage::new();
        let shape = stage.add(fmn_library::Square::new().build());
        stage.add_to_scene(shape).unwrap();
        let mut camera = Camera::new(CameraConfig::default()).unwrap();
        stage.uniforms_mut(shape).unwrap().depth_test = true;
        assert!(scene_document(&stage, &camera).is_err());
        stage.uniforms_mut(shape).unwrap().depth_test = false;
        camera.frame_mut().rotate(0.5, [1.0, 0.0, 0.0]).unwrap();
        assert!(scene_document(&stage, &camera).is_err());
        camera = Camera::new(CameraConfig::default()).unwrap();
        stage
            .get_mut(shape)
            .unwrap()
            .buffer
            .write(0, "fill_rgba", &[1.0, 0.0, 0.0, 1.0]);
        assert!(scene_document(&stage, &camera).is_err());
    }

    #[test]
    fn svg_camera_attachment_keeps_fractional_mix_and_overlay_geometry() {
        let mut stage = Stage::new();
        let shape = stage.add(fmn_library::Square::new().build());
        stage.add_to_scene(shape).unwrap();
        let mut camera = Camera::new(CameraConfig {
            resolution: (160, 90),
            ..CameraConfig::default()
        })
        .unwrap();
        camera.frame_mut().set_center([2.0, 0.0, 0.0]).unwrap();
        let mut centers = Vec::new();
        for attachment in [0.0, 0.5, 1.0] {
            stage.uniforms_mut(shape).unwrap().is_fixed_in_frame = attachment;
            let svg = scene_document(&stage, &camera).unwrap();
            let document = fmn_library::svg::SvgDocument::parse(svg.as_bytes()).unwrap();
            let points = document.shapes[1].path.points();
            let low = points.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min);
            let high = points
                .iter()
                .map(|p| p[0])
                .fold(f64::NEG_INFINITY, f64::max);
            centers.push((low + high) / 2.0);
        }
        assert!(centers[0] < centers[1] && centers[1] < centers[2]);
        assert!((centers[1] - (centers[0] + centers[2]) / 2.0).abs() < 1e-5);
        let fixed = scene_document(&stage, &camera).unwrap();
        camera.frame_mut().rotate(0.5, [1.0, 0.0, 0.0]).unwrap();
        assert_eq!(fixed, scene_document(&stage, &camera).unwrap());
    }

    #[test]
    fn svg_refuses_projection_overflow_before_writing_nonfinite_records() {
        let mut stage = Stage::new();
        let shape = stage.add(fmn_library::Square::new().build());
        stage.add_to_scene(shape).unwrap();
        stage.shift(shape, [1e38, 0.0, 0.0]);
        let before = stage.get_points(shape).unwrap();
        let camera = Camera::new(CameraConfig::default()).unwrap();
        assert!(scene_document(&stage, &camera).is_err());
        assert_eq!(stage.get_points(shape).unwrap(), before);
    }

    #[test]
    fn portal_svg_acceptance() {
        let report = run_portal_gauntlet_svg().unwrap();
        assert_eq!(report.thread_counts, 3);
        assert_eq!(report.failure_paths, 3);
        assert!(!report.document.is_empty());
    }
}
