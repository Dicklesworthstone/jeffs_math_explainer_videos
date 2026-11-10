//! Camera selection is an input capability, not a completion-order guess.
//!
//! Vector-only inputs retain their original affine path. A compiled artifact
//! with camera-only draws uses one fixed camera for the entire generation,
//! including frames before/after a 3D object appears. Camera-bearing FMTL/1
//! artifacts replay their exact captured track; no tracker family is guessed.

use super::{CliError, NativeRenderInput};
use fmn_render::{Camera, CameraConfig, CameraFrame};

pub(super) fn for_input(
    input: &NativeRenderInput,
    config: &fmn_config::Config,
) -> Result<Option<Camera>, CliError> {
    let needs_camera = match input {
        NativeRenderInput::Builtin { names } => names.iter().any(|name| builtin(name).is_some()),
        NativeRenderInput::Compiled { bundle, .. } => bundle.requires_camera(),
    };
    if !needs_camera {
        return Ok(None);
    }
    if config.render.engine != fmn_config::config::Engine::Cpu {
        return Err(CliError::new(
            "capability",
            "mixed camera content requires the CPU renderer; no accelerator request is silently substituted",
        ));
    }
    if config.render.aa != fmn_core::AaPolicy::Adaptive {
        return Err(CliError::new(
            "capability",
            "the camera route supports adaptive coverage; forced affine SSAA is not silently applied to perspective content",
        ));
    }
    camera(config).map(Some)
}

pub(super) fn camera(config: &fmn_config::Config) -> Result<Camera, CliError> {
    Camera::new(camera_config(config)?).map_err(|error| CliError::new("config", error.to_string()))
}

pub(super) fn camera_config(config: &fmn_config::Config) -> Result<CameraConfig, CliError> {
    let frame_config = super::resolved_frame_config(config)?;
    let (width, height) = config.camera.resolution;
    let mut frame = CameraFrame::default();
    frame
        .set_shape([
            config.sizes.frame_height * f64::from(width) / f64::from(height),
            config.sizes.frame_height,
        ])
        .map_err(|error| CliError::new("config", error.to_string()))?;
    Ok(CameraConfig {
        resolution: (width, height),
        fps: config.camera.fps,
        background: frame_config.background,
        frame,
        ..CameraConfig::default()
    })
}

/// Content descriptor for the actual perspective kernel and fixed camera.
/// An implementation revision counter cannot stand in for these pose bits.
pub(super) fn descriptor(camera: &Camera) -> Vec<u8> {
    let mut bytes = b"fmn.cli.fixed-camera.v1\0".to_vec();
    let (width, height) = camera.pixel_shape();
    bytes.extend_from_slice(&width.to_le_bytes());
    bytes.extend_from_slice(&height.to_le_bytes());
    bytes.extend_from_slice(&camera.fps().to_le_bytes());
    bytes.push(camera.samples());
    let frame = camera.frame();
    let background = camera.background();
    for value in frame
        .center()
        .into_iter()
        .chain(frame.shape())
        .chain(frame.orientation())
        .chain([frame.field_of_view(), camera.max_allowable_norm()])
        .chain(camera.light_source_position())
        .chain([background.r, background.g, background.b, background.a])
    {
        bytes.extend_from_slice(&value.to_bits().to_le_bytes());
    }
    bytes
}

// Beside, not inside, the digest-pinned 25-scene primitive corpus.
pub(super) const CAMERA_SCENE_NAMES: &[&str] = &[
    "surface_cube.v1",
    "dot_cloud_depth.v1",
    "image_quad.v1",
    "mixed_camera.v1",
    SEMANTIC_WITNESS_CAMERA_SCENE_NAME,
];

/// The semantic-witness scene (fm-5wq.46) drawn through the fixed camera
/// instead of the affine route, so the oracles read the perspective kernel's
/// own Y mapping.
pub(super) const SEMANTIC_WITNESS_CAMERA_SCENE_NAME: &str = "semantic_witness_camera.v1";

pub(super) fn builtin(name: &str) -> Option<CameraScene> {
    CAMERA_SCENE_NAMES
        .iter()
        .copied()
        .find(|candidate| *candidate == name)
        .map(|name| CameraScene { name })
}

pub(super) struct CameraScene {
    name: &'static str,
}

pub(super) const CAMERA_SCENE_RUN_TIME: f64 = 0.25;

/// Shape registrations whose authored animation is shared by offline playback
/// and the live Studio worker. The separate typesetting witness is not a live
/// native-program registration.
pub(super) fn studio_builtin(name: &str) -> Option<CameraScene> {
    builtin(name).filter(|scene| scene.name != SEMANTIC_WITNESS_CAMERA_SCENE_NAME)
}

const fn witness_scene() -> fmn::builtins::SemanticWitnessScene {
    fmn::builtins::SemanticWitnessScene::new()
}

impl fmn::SceneConstruct for CameraScene {
    fn name(&self) -> &str {
        self.name
    }

    fn tex_preflight(&self) -> Vec<fmn::prelude::TypesetRequest<'_>> {
        if self.name == SEMANTIC_WITNESS_CAMERA_SCENE_NAME {
            vec![fmn::prelude::TypesetRequest::math(
                fmn::builtins::witness::TEX,
            )]
        } else {
            Vec::new()
        }
    }

    fn construct(&mut self, stage: &mut fmn::Stage<'_>) -> fmn::Result<()> {
        if self.name == SEMANTIC_WITNESS_CAMERA_SCENE_NAME {
            return witness_scene().construct(stage);
        }
        if let Some(mob) = self.populate(stage.scene_mut())? {
            stage.play(Self::movement(mob)?)?;
        }
        Ok(())
    }
}

impl CameraScene {
    fn populate(&self, scene: &mut fmn_scene::Scene) -> fmn::Result<Option<fmn::mobject::Mob>> {
        use fmn::prelude::*;
        use fmn_library::{DotCloud, ImageMobject};
        let mixed = self.name == "mixed_camera.v1";
        let mut moving = None;
        if mixed || self.name == "surface_cube.v1" {
            let cube = scene.add_mobject(Cube::new(1.8).color(BLUE))?;
            scene
                .stage_mut()
                .rotate(cube, 0.45, [1.0, 1.0, 0.0], Some(ORIGIN), None);
            scene
                .stage_mut()
                .shift(cube, if mixed { [-1.8, 0.0, 0.0] } else { ORIGIN });
            moving = Some(cube);
        }
        if mixed || self.name == "dot_cloud_depth.v1" {
            let dots = scene.add_mobject(
                DotCloud::new([[-0.55, -0.5, -0.6], [0.0, 0.5, 0.4], [0.55, -0.25, 0.8]])
                    .colored(YELLOW, 1.0)
                    .with_radius(0.24)
                    .make_3d(),
            )?;
            moving.get_or_insert(dots);
        }
        if mixed || self.name == "image_quad.v1" {
            let image = ImageMobject::from_rgba8(
                2,
                2,
                vec![
                    255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
                ],
            )
            .map_err(|error| {
                SceneError::Integration(fmn_scene::IntegrationError::new(
                    "image",
                    error.to_string(),
                ))
            })?;
            let image = scene.add_mobject(image.with_height(1.8))?;
            scene
                .stage_mut()
                .shift(image, if mixed { [1.8, 0.0, 0.0] } else { ORIGIN });
            moving.get_or_insert(image);
        }
        let marker = scene.add_mobject(Circle::new().radius(0.14).color(WHITE))?;
        scene
            .stage_mut()
            .set_fill(marker, Some(WHITE), Some(1.0), None, true);
        scene.stage_mut().shift(marker, [0.0, 1.5, 0.0]);
        Ok(moving)
    }

    fn movement(
        mob: fmn::mobject::Mob,
    ) -> Result<fmn::mobject::AnimBuilder, fmn::mobject::AnimateError> {
        mob.animate()
            .set_anim_args(fmn::mobject::animate::AnimateArgs {
                run_time: Some(CAMERA_SCENE_RUN_TIME),
                rate_func: Some(fmn::core::rate::linear),
                ..Default::default()
            })?
            .shift([0.5, 0.0, 0.0])
    }

    /// Construct only the initial graph. The worker's existing stepped driver
    /// owns later captures, so pausing never executes future scene work.
    pub(super) fn native_program(
        &self,
        runtime: fmn_scene::RuntimeConfig,
        seed: u64,
        frame_limit: u64,
    ) -> Result<fmn_studio::native::NativeSceneProgram, fmn_studio::ServiceError> {
        let failed = |error: String| {
            fmn_studio::ServiceError::new(fmn_studio::WorkerErrorCode::ExecutionFailed, error)
        };
        let mut scene = fmn_scene::Scene::new(runtime, seed).map_err(|e| failed(e.to_string()))?;
        let moving = self
            .populate(&mut scene)
            .map_err(|e| failed(e.to_string()))?;
        let mut segments = Vec::new();
        if let Some(mob) = moving {
            let movement = Self::movement(mob).map_err(|e| failed(e.to_string()))?;
            let animation = fmn::animation::prepare_animation(movement, scene.stage_mut())
                .map_err(|e| failed(e.to_string()))?;
            segments.push(fmn_studio::native::NativeSegment::Play {
                animations: vec![animation],
                overrides: fmn_scene::PlayOverrides::default(),
            });
        }
        fmn_studio::native::NativeSceneProgram::new(scene, segments, frame_limit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_plan_names_its_real_cpu_kernel_and_budgets_retained_frames() {
        use fmn_runtime::{ExecutionEngine, OutputPixelFormat, RenderIntent};
        let fs = fmn_platform::fs::VirtualFs::new();
        let mut config = fmn_config::Config::resolve(&[], None).unwrap().config;
        config.camera.resolution = (64, 48);
        config.render.threads = fmn_config::config::ThreadPolicy::Fixed(1);
        let (affine, _, _) = super::super::derive_execution_plan(
            &fs,
            &config,
            RenderIntent::Offline,
            OutputPixelFormat::Rgba8,
        )
        .unwrap();
        let (camera, _, _) = super::super::derive_execution_plan_with_annex(
            &fs,
            &config,
            RenderIntent::Offline,
            OutputPixelFormat::Rgba8,
            super::super::MetalSelection::Offline,
            true,
        )
        .unwrap();
        assert_eq!(affine.engine, ExecutionEngine::FastCpu);
        assert_eq!(camera.engine, ExecutionEngine::CertifiedCpu);
        assert_eq!(
            config.determinism.mode,
            fmn_config::config::DeterminismMode::Standard
        );
        assert!(camera.estimated_in_flight_bytes >= affine.estimated_in_flight_bytes + 64 * 48 * 8);
    }

    #[test]
    fn fixed_camera_binds_output_shape_pose_and_coverage_policy() {
        let mut config = fmn_config::Config::resolve(&[], None).unwrap().config;
        config.camera.resolution = (64, 48);
        config.camera.fps = 8;
        config.sizes.frame_height = 4.0;
        let mut camera = camera(&config).unwrap();
        assert_eq!(camera.frame().shape(), [16.0 / 3.0, 4.0]);
        assert_eq!(camera.fps(), 8);
        let before = descriptor(&camera);
        camera.frame_mut().set_center([1.0, 0.0, 0.0]).unwrap();
        assert_ne!(descriptor(&camera), before);
        let input = NativeRenderInput::Builtin {
            names: vec!["surface_cube.v1".into()],
        };
        config.render.aa = fmn_core::AaPolicy::Ssaa2x;
        assert!(for_input(&input, &config).is_err());
        config.render.aa = fmn_core::AaPolicy::Adaptive;
        config.render.engine = fmn_config::config::Engine::Metal;
        assert!(for_input(&input, &config).is_err());
        let affine = NativeRenderInput::Builtin {
            names: vec!["circle_shift.v1".into()],
        };
        assert!(for_input(&affine, &config).unwrap().is_none());
    }

    #[test]
    fn camera_captures_use_every_render_team_and_match_serial_pixels() {
        use super::super::{NativeFrameFormat, RenderSink, RenderTarget};
        use fmn::mobject::{Mobject, Stage};
        use fmn_codec::{PngLimits, decode_png};
        use fmn_core::rng::RngRoot;
        use fmn_frame::convert::rgba16f_to_rgba8;
        use fmn_frame::{FrameBuffer, FrameLayout, PixelFormat};
        use fmn_platform::fs::{FileSystem, VirtualFs};
        use fmn_platform::topology::HardwareTopology;
        use fmn_render::{
            EngineIdentity, RetainedFrameRenderer, RetainedFrameRendererConfig, Tiling,
        };
        use fmn_runtime::{
            ExecutionPlan, OutputPixelFormat, PlanRequest, RenderIntent, SurfaceSpec,
        };
        use std::path::{Path, PathBuf};
        use std::sync::Arc;

        let mut config = fmn_config::Config::resolve(&[], None).unwrap().config;
        config.camera.resolution = (32, 24);
        config.camera.fps = 8;
        let camera = camera(&config).unwrap();
        let mut surface = SurfaceSpec::lumen(32, 24);
        surface.working_bytes_per_pixel += 16;
        let plan = ExecutionPlan::derive(
            PlanRequest::certified(RenderIntent::Offline, surface, OutputPixelFormat::Rgba8)
                .with_max_frames_in_flight(4),
            &HardwareTopology::from_group_sizes(&[2, 2]).unwrap(),
            None,
        )
        .unwrap();
        assert_eq!(plan.render_teams.len(), 2);
        let mut stage = Stage::new();
        let mob = stage.add(Mobject::from(fmn_library::Cube::new(1.4)));
        stage.add_to_scene(mob).unwrap();
        let mut timeline = fmn::animation::Timeline::new(8).unwrap();
        timeline.wait(1.0).unwrap();
        let bytes = fmn_scene::export_timeline_bundle(timeline, &mut stage, &RngRoot::from_seed(0))
            .unwrap();
        let shared = fmn_scene::timeline_bundle::SharedTimelineBundle::from_bytes(&bytes).unwrap();
        let mut serial = RetainedFrameRenderer::new(RetainedFrameRendererConfig {
            frame: super::super::resolved_frame_config(&config).unwrap(),
            tiling: Tiling {
                macro_tile: plan.macro_tile,
                fine_tile: plan.fine_tile,
            },
            engine: EngineIdentity::certified(),
            threads: 1,
        })
        .unwrap();
        serial.render_with_camera(&stage, &camera).unwrap();
        let mut expected =
            FrameBuffer::new(FrameLayout::tight(PixelFormat::Rgba8, 32, 24).unwrap());
        rgba16f_to_rgba8(serial.frame(), &mut expected).unwrap();
        for compiled in [false, true] {
            let fs = Arc::new(VirtualFs::new());
            let mut sink = RenderSink::new_with_camera(
                fs.clone(),
                &config,
                &plan,
                &RenderTarget::Native(NativeFrameFormat::PngSequence),
                PathBuf::from("/frames"),
                Some(camera.clone()),
            )
            .unwrap();
            for index in 0..8 {
                if compiled {
                    sink.render_compiled(shared.frame_job(index).unwrap())
                        .unwrap();
                } else {
                    sink.render_stage(&stage, u64::from(index)).unwrap();
                }
            }
            let report = sink.finish(None).unwrap();
            assert_eq!(
                report.backend.route,
                if compiled {
                    "compiled-camera-cpu"
                } else {
                    "camera-cpu"
                }
            );
            assert!(report.backend.journal.ends_with(&descriptor(&camera)));
            let stats = report.backend.pipeline.unwrap();
            assert_eq!(
                (stats.submitted, stats.emitted, stats.outstanding_slots),
                (8, 8, 0)
            );
            assert_eq!(stats.render_team_frames.len(), 2);
            assert!(stats.render_team_frames.iter().all(|n| *n > 0));
            for index in 0..8 {
                let png = fs
                    .read(Path::new(&format!("/frames/frame_{index:06}.png")))
                    .unwrap();
                assert_eq!(
                    decode_png(&png, &PngLimits::default()).unwrap().rgba,
                    expected.as_bytes()
                );
            }
        }
    }
}
