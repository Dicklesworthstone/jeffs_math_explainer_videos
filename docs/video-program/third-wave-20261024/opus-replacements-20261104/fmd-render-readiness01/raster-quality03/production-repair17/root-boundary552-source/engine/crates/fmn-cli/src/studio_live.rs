//! Registration of executable native Studio scenes. The factory runs only in
//! the disposable worker; the supervisor owns content identities, not callbacks.
use super::*;
use fmn_studio::native::{
    NativeReplayPolicy, NativeSceneProgram, NativeSceneWorker, NativeSegment, NativeWorkerConfig,
};

/// One executable registration drives selection, timing, construction and
/// saved-session eligibility. These are the ordinary builtins' authoring
/// operations; captured FMTL frames do not become executable scene callbacks.
enum Builtin {
    Primitive(fmn::builtins::PrimitiveScene),
    TexSpan,
    FormulaSheet,
    SemanticWitness { camera: bool },
    SoundCue,
    Interactive,
    Camera(camera_route::CameraScene),
}

impl Builtin {
    fn named(name: &str) -> Option<Self> {
        use fmn::builtins::*;
        if let Some(scene) = primitive_scene(name) {
            return Some(Self::Primitive(scene));
        }
        match name {
            TEX_SPAN_SCENE_NAME => Some(Self::TexSpan),
            FORMULA_SHEET_SCENE_NAME => Some(Self::FormulaSheet),
            SEMANTIC_WITNESS_SCENE_NAME => Some(Self::SemanticWitness { camera: false }),
            camera_route::SEMANTIC_WITNESS_CAMERA_SCENE_NAME => {
                Some(Self::SemanticWitness { camera: true })
            }
            SOUND_CUE_SCENE_NAME => Some(Self::SoundCue),
            INTERACTIVE_SCENE_NAME => Some(Self::Interactive),
            _ => camera_route::studio_builtin(name).map(Self::Camera),
        }
    }

    fn camera(&self) -> bool {
        matches!(
            self,
            Self::Camera(_) | Self::SemanticWitness { camera: true }
        )
    }

    fn durations(&self) -> &'static [f64] {
        use fmn::builtins::*;
        match self {
            Self::Primitive(_) => &[PrimitiveScene::PLAY_DURATION, PrimitiveScene::WAIT_DURATION],
            Self::TexSpan => &[TexSpanScene::WAIT_DURATION],
            Self::FormulaSheet => &[
                FormulaSheetScene::PLAY_DURATION,
                FormulaSheetScene::WAIT_DURATION,
            ],
            Self::SemanticWitness { .. } => &[SemanticWitnessScene::WAIT_DURATION],
            Self::SoundCue => &[
                SoundCueScene::BEFORE_SOUND_DURATION,
                SoundCueScene::AFTER_SOUND_DURATION,
            ],
            Self::Interactive => &[InteractiveCanvas::WAIT_DURATION],
            Self::Camera(_) => &[camera_route::CAMERA_SCENE_RUN_TIME],
        }
    }

    fn frame_count(&self, fps: u32) -> Result<u64, CliError> {
        let clock = fmn::animation::RationalFrameClock::new(fps)
            .map_err(|error| CliError::new("config", error.to_string()))?;
        self.durations()
            .iter()
            .try_fold(1_u64, |frames, &duration| {
                let segment = clock
                    .segment(duration)
                    .map_err(|error| CliError::new("config", error.to_string()))?;
                let count = u64::try_from(segment.n_frames())
                    .map_err(|_| internal("native builtin has an invalid segment frame count"))?;
                frames
                    .checked_add(count)
                    .ok_or_else(|| internal("native builtin frame count overflowed"))
            })
    }

    fn program(
        &self,
        runtime: fmn_scene::RuntimeConfig,
        seed: u64,
        frame_limit: u64,
        template: &str,
        threads: usize,
    ) -> Result<NativeSceneProgram, fmn_studio::ServiceError> {
        use fmn::builtins::*;
        if let Self::Camera(program) = self {
            return program.native_program(runtime, seed, frame_limit);
        }
        let mut scene = fmn_scene::Scene::new(runtime, seed)
            .map_err(|error| studio_service_error(error.to_string()))?;
        // Factories own no filesystem capabilities. Use the resolved template
        // in a fresh bounded memory session on every cold reconstruction.
        let typesetting = fmn::tex::TexSession::memory(template);
        let engine = |program: &dyn fmn::SceneConstruct| {
            let engine = typesetting
                .engine()
                .map_err(|error| studio_service_error(error.to_string()))?;
            let requests = program.tex_preflight();
            let outcomes = engine
                .preflight_requests(
                    &requests,
                    std::num::NonZeroUsize::new(threads.min(64))
                        .unwrap_or(std::num::NonZeroUsize::MIN),
                )
                .map_err(|error| studio_service_error(error.to_string()))?;
            for outcome in outcomes {
                outcome.map_err(|error| studio_service_error(error.to_string()))?;
            }
            Ok::<_, fmn_studio::ServiceError>(engine)
        };
        let wait = |duration| NativeSegment::Wait {
            duration: Some(duration),
        };
        let mut records = Vec::new();
        let segments = match self {
            Self::Primitive(authored) => {
                let root = authored
                    .populate(scene.stage_mut())
                    .map_err(|error| studio_service_error(error.to_string()))?;
                let authored = *authored;
                vec![
                    NativeSegment::play_with(fmn_scene::PlayOverrides::default(), move |context| {
                        let builder = authored
                            .animation(root)
                            .map_err(|error| fmn_scene::SceneError::Animation(error.into()))?;
                        Ok(vec![fmn::animation::prepare_animation(
                            builder,
                            context.stage_mut(),
                        )?])
                    }),
                    wait(PrimitiveScene::WAIT_DURATION),
                ]
            }
            Self::TexSpan => {
                let mut authored = tex_span_scene(TEX_SPAN_SCENE_NAME)
                    .ok_or_else(|| studio_service_error("Tex span registration disappeared"))?;
                let engine = engine(&authored)?;
                authored
                    .populate(scene.stage_mut(), engine)
                    .map_err(|error| studio_service_error(error.to_string()))?;
                records = authored.into_span_records();
                vec![wait(TexSpanScene::WAIT_DURATION)]
            }
            Self::FormulaSheet => {
                let authored = formula_sheet_scene(FORMULA_SHEET_SCENE_NAME).ok_or_else(|| {
                    studio_service_error("formula sheet registration disappeared")
                })?;
                let (first, spans) = authored
                    .populate(scene.stage_mut(), engine(&authored)?)
                    .map_err(|error| studio_service_error(error.to_string()))?;
                records = spans;
                let mut segments = Vec::new();
                if let Some(first) = first {
                    segments.push(NativeSegment::play_with(
                        fmn_scene::PlayOverrides::default(),
                        move |context| {
                            let builder = FormulaSheetScene::animation(first)
                                .map_err(|error| fmn_scene::SceneError::Animation(error.into()))?;
                            Ok(vec![fmn::animation::prepare_animation(
                                builder,
                                context.stage_mut(),
                            )?])
                        },
                    ));
                }
                segments.push(wait(FormulaSheetScene::WAIT_DURATION));
                segments
            }
            Self::SemanticWitness { .. } => {
                let authored = SemanticWitnessScene::new();
                records = authored
                    .populate(scene.stage_mut(), engine(&authored)?)
                    .map_err(|error| studio_service_error(error.to_string()))?;
                vec![wait(SemanticWitnessScene::WAIT_DURATION)]
            }
            Self::SoundCue => {
                sound_scene(SOUND_CUE_SCENE_NAME)
                    .ok_or_else(|| studio_service_error("sound cue registration disappeared"))?
                    .populate(scene.stage_mut())
                    .map_err(|error| studio_service_error(error.to_string()))?;
                vec![
                    wait(SoundCueScene::BEFORE_SOUND_DURATION),
                    NativeSegment::edit(|context| {
                        context.add_sound(SOUND_CUE_ASSET_NAME, 0.0, None, None)
                    }),
                    wait(SoundCueScene::AFTER_SOUND_DURATION),
                ]
            }
            Self::Interactive => {
                populate_interactive_canvas(scene.stage_mut())
                    .map_err(|error| studio_service_error(error.to_string()))?;
                vec![wait(InteractiveCanvas::WAIT_DURATION)]
            }
            Self::Camera(_) => unreachable!("camera construction returned above"),
        };
        let (spans, skipped) = rebuild_span_registry(scene.stage(), &records);
        if skipped != 0 {
            return Err(studio_service_error(
                "native builtin produced an invalid source-span map",
            ));
        }
        let mut program = NativeSceneProgram::new(scene, segments, frame_limit)?;
        *program.spans_mut() = spans;
        Ok(program)
    }
}

pub(super) fn selected(command: &RenderCommand) -> bool {
    command.file.as_deref() == Some(Path::new(BUILTIN_SCENE_SOURCE))
        && command
            .scene_names
            .iter()
            .any(|name| Builtin::named(name).is_some())
}

pub(super) fn validate_selection(command: &RenderCommand) -> Result<(), CliError> {
    if command.scene_names.len() != 1 || command.write_all {
        return Err(CliError::new(
            "scene",
            "live Studio requires exactly one registered scene",
        ));
    }
    if command.skip_animations || command.animation_range.is_some() || command.presenter_mode {
        return Err(CliError::new(
            "config",
            "live Studio uses its timeline controls, not offline skip/range/presenter flags",
        ));
    }
    Ok(())
}

struct Configuration {
    native: NativeWorkerConfig,
    runtime: fmn_scene::RuntimeConfig,
    seed: u64,
    template: String,
}

fn configuration(fs: &dyn FileSystem, command: &StudioCommand) -> Result<Configuration, CliError> {
    validate_selection(&command.render)?;
    let config = resolve_render_config(fs, &command.render)?;
    let mut runtime = command.render.runtime_config(&config);
    runtime.windowed = true;
    // Use Proscenium's effective windowed FPS rather than a second clock policy.
    let fps = runtime.effective_fps();
    let name = &command.render.scene_names[0];
    let registered = Builtin::named(name)
        .ok_or_else(|| CliError::new("scene", "scene has no native Studio registration"))?;
    let camera = registered.camera();
    let frame_count = registered.frame_count(fps)?;
    if camera && config.render.aa != fmn_core::AaPolicy::Adaptive {
        return Err(CliError::new(
            "capability",
            "native camera Studio requires adaptive camera coverage",
        ));
    }
    let (plan, _, _) = derive_execution_plan_with_annex(
        fs,
        &config,
        fmn_runtime::RenderIntent::Preview,
        fmn_runtime::OutputPixelFormat::Rgba8,
        MetalSelection::Studio,
        camera,
    )?;
    let engine = match plan.engine {
        fmn_runtime::ExecutionEngine::CertifiedCpu => EngineIdentity::certified(),
        fmn_runtime::ExecutionEngine::FastCpu => EngineIdentity::fast(),
        _ => {
            return Err(CliError::new(
                "capability",
                "live native Studio currently requires a CPU renderer; no annex fallback is substituted",
            ));
        }
    };
    let renderer = RetainedFrameRendererConfig {
        frame: resolved_frame_config(&config)?,
        tiling: Tiling {
            macro_tile: plan.macro_tile,
            fine_tile: plan.fine_tile,
        },
        engine,
        threads: plan
            .render_teams
            .first()
            .map_or(1, fmn_runtime::TeamPlan::threads),
    };
    let build = fmn_studio::protocol_digest(BUILD_ID.as_bytes());
    // Inputs are all embedded or resolved before worker creation. No I/O, shared
    // mutable captures or arbitrary closure deserialization enters this factory.
    let mut closure = include_bytes!("studio_live.rs").to_vec();
    closure.extend_from_slice(include_bytes!("camera_route.rs"));
    closure.extend_from_slice(include_bytes!("../../fmn/src/lib.rs"));
    closure.extend_from_slice(SUITE_LOCK_BYTES);
    closure.extend_from_slice(build.as_bytes());
    closure.extend_from_slice(
        &config
            .canonical_bytes()
            .map_err(|error| internal(error.to_string()))?,
    );
    let mut native = NativeWorkerConfig::new(
        name,
        build,
        fmn_studio::protocol_digest(&closure),
        frame_count,
        fps,
        renderer,
    );
    native.replay = NativeReplayPolicy::ColdVerified;
    native.checkpoint_frames = command.checkpoint_frames;
    if camera {
        let mut base = camera_route::camera_config(&config)?;
        base.fps = fps;
        native.camera = Some(base);
    }
    Ok(Configuration {
        native,
        runtime,
        seed: config.determinism.seed,
        template: config.tex.template,
    })
}

pub(super) fn source_reads(
    fs: &dyn FileSystem,
    command: &StudioCommand,
) -> Result<Vec<AssetRead>, CliError> {
    if selected(&command.render) {
        let native = configuration(fs, command)?.native;
        let mut reads = native.base_input_reads();
        if let Some(base) = native.camera {
            let camera = fmn_render::Camera::new(base)
                .map_err(|error| CliError::new("config", error.to_string()))?;
            let backend = fmn_studio::native::camera_capture_backend(&camera, native.renderer)
                .map_err(|error| CliError::new("capability", error.to_string()))?;
            reads.push(AssetRead {
                path: "native/camera-capture-policy".into(),
                digest: backend.digest(),
            });
        }
        Ok(reads)
    } else if let Some(input) = camera_bundle_input(fs, command)? {
        let NativeRenderInput::Compiled {
            bundle,
            source_item,
            ..
        } = input
        else {
            return Err(internal("compiled camera selection changed kind"));
        };
        let view = camera_bundle_config(fs, command, &bundle)?;
        fmn_studio::camera_bundle::CameraBundleWorker::input_reads(
            fmn_studio::protocol_digest(BUILD_ID.as_bytes()),
            &compiled_source_read(source_item)?,
            &bundle,
            &view,
        )
        .map_err(|error| CliError::new("scene", error.to_string()))
    } else {
        Ok(Vec::new())
    }
}

pub(super) fn worker(
    fs: &dyn FileSystem,
    command: &StudioCommand,
) -> Result<Box<dyn fmn_studio::WorkerService>, CliError> {
    if !selected(&command.render) {
        if let Some(input) = camera_bundle_input(fs, command)? {
            let NativeRenderInput::Compiled {
                name,
                bundle,
                source_item,
                ..
            } = input
            else {
                return Err(internal("compiled camera selection changed kind"));
            };
            let view = camera_bundle_config(fs, command, &bundle)?;
            return Ok(Box::new(
                fmn_studio::camera_bundle::CameraBundleWorker::new(
                    name,
                    fmn_studio::protocol_digest(BUILD_ID.as_bytes()),
                    compiled_source_read(source_item)?,
                    *bundle,
                    view,
                )
                .map_err(|error| CliError::new("scene", error.to_string()))?,
            ));
        }
        return Ok(Box::new(NativeStudioWorker::from_command(fs, command)?));
    }
    let Configuration {
        native: config,
        runtime,
        seed,
        template,
    } = configuration(fs, command)?;
    let limit = config.frame_count - 1;
    let registered = Builtin::named(&config.scene)
        .ok_or_else(|| internal("native Studio registration disappeared"))?;
    let threads = config.renderer.threads;
    let worker = NativeSceneWorker::new(config, move || {
        registered.program(runtime.clone(), seed, limit, &template, threads)
    })
    .map_err(|error| CliError::new("scene", error.to_string()))?;
    Ok(Box::new(worker))
}

// Keep executable scene registration and immutable bundle playback separate.
// A recorded Python updater is already a captured picture, never live code in
// this worker. The ordinary resolver validates the actual source bytes once.
fn camera_bundle_input(
    fs: &dyn FileSystem,
    command: &StudioCommand,
) -> Result<Option<NativeRenderInput>, CliError> {
    if command.render.file.as_deref() == Some(Path::new(BUILTIN_SCENE_SOURCE)) {
        return Ok(None);
    }
    let input = resolve_native_render_input(fs, &command.render)?;
    if !matches!(&input, NativeRenderInput::Compiled { bundle, .. } if bundle.requires_camera()) {
        return Ok(None);
    }
    if command.render.skip_animations
        || command.render.animation_range.is_some()
        || command.render.presenter_mode
        || command.render.write_all
    {
        return Err(CliError::new(
            "config",
            "camera-bundle Studio uses recorded frames, not offline skip/range/presenter/batch flags",
        ));
    }
    Ok(Some(input))
}

fn compiled_source_read(source: ClosureItem) -> Result<AssetRead, CliError> {
    Ok(AssetRead {
        path: source
            .virtual_path
            .ok_or_else(|| internal("compiled Studio source omitted its closure path"))?,
        digest: source.digest,
    })
}

fn camera_bundle_config(
    fs: &dyn FileSystem,
    command: &StudioCommand,
    bundle: &TimelineBundle,
) -> Result<fmn_studio::camera_bundle::CameraBundleView, CliError> {
    let mut render = command.render.clone();
    if render.fps.is_some_and(|fps| fps != bundle.fps()) {
        return Err(CliError::new(
            "config",
            "requested FPS disagrees with the camera artifact's fixed clock",
        ));
    }
    render.fps = Some(bundle.fps());
    let config = resolve_render_config(fs, &render)?;
    if config.render.aa != fmn_core::AaPolicy::Adaptive {
        return Err(CliError::new(
            "capability",
            "camera-bundle Studio requires adaptive camera coverage",
        ));
    }
    let (plan, _, _) = derive_execution_plan_with_annex(
        fs,
        &config,
        fmn_runtime::RenderIntent::Preview,
        fmn_runtime::OutputPixelFormat::Rgba8,
        MetalSelection::Studio,
        true,
    )?;
    Ok(fmn_studio::camera_bundle::CameraBundleView {
        renderer: RetainedFrameRendererConfig {
            frame: resolved_frame_config(&config)?,
            tiling: Tiling {
                macro_tile: plan.macro_tile,
                fine_tile: plan.fine_tile,
            },
            engine: EngineIdentity::certified(),
            threads: plan
                .render_teams
                .first()
                .map_or(1, fmn_runtime::TeamPlan::threads),
        },
        seed: config.determinism.seed,
        fixed_camera: if bundle.has_camera_track() {
            None
        } else {
            Some(camera_route::camera(&config)?)
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn command() -> StudioCommand {
        let Invocation::Studio(command) = parse_args([
            "studio",
            "--resolution",
            "96x54",
            "--fps",
            "8",
            "--threads",
            "1",
            "@builtin",
            "interactive.v1",
        ])
        .unwrap() else {
            panic!("Studio");
        };
        command
    }
    #[test]
    fn live_registration_exposes_constructed_zero_and_native_windowed_clock() {
        let fs = fmn_platform::fs::VirtualFs::new();
        let command = command();
        let mut service = worker(&fs, &command).unwrap();
        let scene = fmn::builtins::INTERACTIVE_SCENE_NAME.to_owned();
        let fmn_studio::WorkerResponse::StudioData { bytes, .. } = service
            .handle(fmn_studio::SupervisorRequest::Inspect {
                scene: scene.clone(),
            })
            .unwrap()
        else {
            panic!("inspector");
        };
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("\"scene_time\":0"));
        assert!(text.contains("\"fps\":30"));
        assert!(text.contains("\"frame_count\":61"));
        assert!(text.contains("\"input_events\":true,\"input_revision\":0"));
        assert!(matches!(
            service
                .handle(fmn_studio::SupervisorRequest::Scrub {
                    scene: scene.clone(),
                    frame: 60
                })
                .unwrap(),
            fmn_studio::WorkerResponse::Frame(_)
        ));
        assert!(
            service
                .handle(fmn_studio::SupervisorRequest::Scrub { scene, frame: 61 })
                .is_err()
        );
        assert_eq!(source_reads(&fs, &command).unwrap().len(), 4);
    }
    #[test]
    fn live_factory_rejects_offline_skip_before_execution() {
        let mut command = command();
        command.render.skip_animations = true;
        assert!(worker(&fmn_platform::fs::VirtualFs::new(), &command).is_err());
    }

    fn ordinary_builtin_names() -> Vec<&'static str> {
        fmn::builtins::PRIMITIVE_SCENE_NAMES
            .into_iter()
            .chain([
                fmn::builtins::TEX_SPAN_SCENE_NAME,
                fmn::builtins::FORMULA_SHEET_SCENE_NAME,
                fmn::builtins::SEMANTIC_WITNESS_SCENE_NAME,
                fmn::builtins::SOUND_CUE_SCENE_NAME,
            ])
            .chain(camera_route::CAMERA_SCENE_NAMES.iter().copied())
            .collect()
    }

    struct ObserveBuiltin<'a> {
        authored: &'a mut dyn fmn::SceneConstruct,
        initial: std::rc::Rc<std::cell::RefCell<Option<fmn_studio::InspectorSnapshot>>>,
    }

    impl fmn::SceneConstruct for ObserveBuiltin<'_> {
        fn name(&self) -> &str {
            self.authored.name()
        }

        fn tex_preflight(&self) -> Vec<fmn::prelude::TypesetRequest<'_>> {
            self.authored.tex_preflight()
        }

        fn construct(&mut self, stage: &mut fmn::Stage<'_>) -> fmn::Result<()> {
            let initial = std::rc::Rc::clone(&self.initial);
            stage.scene_mut().set_preflight_hook(move |stage, _| {
                let snapshot = fmn_studio::InspectorSnapshot::capture(
                    stage,
                    &fmn_studio::SpanRegistry::new(),
                    fmn_studio::InspectorLimits::default(),
                )
                .expect("constructed builtin graph");
                assert!(!snapshot.truncated);
                assert!(initial.borrow_mut().replace(snapshot).is_none());
                Ok(())
            })?;
            self.authored.construct(stage)
        }
    }

    #[test]
    fn native_builtin_programs_match_constructed_graphs_and_every_offline_capture() {
        for name in ordinary_builtin_names() {
            for fps in [7, 30] {
                let runtime = fmn_scene::RuntimeConfig {
                    fps,
                    ..fmn_scene::RuntimeConfig::default()
                };
                let mut authored = resolve_builtin_program(name).unwrap();
                let initial = std::rc::Rc::new(std::cell::RefCell::new(None));
                let mut observed = ObserveBuiltin {
                    authored: &mut *authored,
                    initial: std::rc::Rc::clone(&initial),
                };
                let mut captures = StudioPacketCapture::default();
                let completed = fmn::run_scene_with_typesetting(
                    &mut observed,
                    runtime.clone(),
                    71,
                    &mut captures,
                    &fmn::tex::TexSession::default(),
                    std::num::NonZeroUsize::MIN,
                )
                .unwrap();
                let registered = Builtin::named(name).unwrap();
                let frame_count = registered.frame_count(fps).unwrap();
                assert_eq!(
                    frame_count,
                    captures.packets.len() as u64 + 1,
                    "{name} at {fps}"
                );
                // One spare admission slot lets the cursor finish its final
                // epilogue and report exhaustion after the last real capture.
                let mut live = registered
                    .program(runtime, 71, frame_count, "default", 1)
                    .unwrap();
                assert_eq!(live.frame_index(), 0);
                assert_eq!(live.preview().scene().time().frames(), 0);
                assert_eq!(live.preview().scene().play_count(), 0);
                let constructed = fmn_studio::InspectorSnapshot::capture(
                    live.preview().stage(),
                    &fmn_studio::SpanRegistry::new(),
                    fmn_studio::InspectorLimits::default(),
                )
                .unwrap();
                // Preflight observes the rooted authored graph before begin;
                // animation preparation may also own unrooted target records.
                assert_eq!(
                    &constructed,
                    initial.borrow().as_ref().expect("offline preflight"),
                    "constructed graph: {name} at {fps}",
                );
                for expected in &captures.packets {
                    let actual = live.next_frame().unwrap().expect("native capture");
                    assert_eq!(
                        actual.frame_index(),
                        expected.frame_index(),
                        "{name} at {fps}"
                    );
                    assert_eq!(actual.time(), expected.time(), "{name} at {fps}");
                    assert_eq!(actual.alpha(), expected.alpha(), "{name} at {fps}");
                    assert_eq!(
                        actual.segment_frame(),
                        expected.segment_frame(),
                        "{name} at {fps}"
                    );
                    assert_eq!(
                        actual.materialize_stage().snapshot().to_bytes().unwrap(),
                        expected.materialize_stage().snapshot().to_bytes().unwrap(),
                        "complete native state: {name} at {fps}, frame {}",
                        actual.frame_index(),
                    );
                }
                assert!(live.next_frame().unwrap().is_none(), "{name} at {fps}");
                assert_eq!(live.preview().scene().time(), completed.scene().time());
                assert_eq!(
                    live.preview().scene().play_count(),
                    completed.scene().play_count()
                );
                assert_eq!(
                    live.preview().scene().sound_requests(),
                    completed.scene().sound_requests()
                );
                assert_eq!(
                    live.preview().stage().snapshot().to_bytes().unwrap(),
                    completed.scene().stage().snapshot().to_bytes().unwrap(),
                    "completed graph: {name} at {fps}",
                );
            }
        }
    }

    #[test]
    fn native_builtin_workers_admit_every_family_with_actual_clock_and_input_capabilities() {
        let fs = fmn_platform::fs::VirtualFs::new();
        for name in ordinary_builtin_names() {
            let command = camera_command(name);
            assert!(selected(&command.render), "{name}");
            assert_eq!(studio_scene_name(&fs, &command.render).unwrap(), name);
            let registered = Builtin::named(name).unwrap();
            let expected_frames = match registered {
                Builtin::Primitive(_) | Builtin::FormulaSheet => 13,
                Builtin::SoundCue => 31,
                Builtin::TexSpan | Builtin::SemanticWitness { .. } | Builtin::Camera(_) => 9,
                Builtin::Interactive => unreachable!("separate interactive fixture"),
            };
            let config = configuration(&fs, &command).unwrap();
            assert_eq!(
                (config.native.fps, config.native.frame_count),
                (30, expected_frames)
            );
            assert_eq!(config.native.camera.is_some(), registered.camera());
            let mut service = worker(&fs, &command).unwrap();
            let fmn_studio::WorkerResponse::StudioData { bytes, .. } = service
                .handle(fmn_studio::SupervisorRequest::Inspect { scene: name.into() })
                .unwrap()
            else {
                panic!("native inspector for {name}");
            };
            let info = String::from_utf8(bytes).unwrap();
            assert!(info.contains("\"scene_time\":0"), "{name}");
            assert!(info.contains("\"frame_index\":0"), "{name}");
            assert!(info.contains("\"fps\":30"), "{name}");
            assert!(
                info.contains(&format!("\"frame_count\":{expected_frames}")),
                "{name}"
            );
            assert!(
                info.contains(if registered.camera() {
                    "\"input_events\":false"
                } else {
                    "\"input_events\":true"
                }),
                "{name}"
            );
            if matches!(
                registered,
                Builtin::TexSpan | Builtin::FormulaSheet | Builtin::SemanticWitness { .. }
            ) {
                assert!(
                    info.contains("\"source_span\":{"),
                    "missing native spans: {name}"
                );
                assert!(info.contains("\"kind\":\"math_glyph\""), "{name}");
                if !matches!(registered, Builtin::FormulaSheet) {
                    assert!(info.contains("\"kind\":\"text_glyph\""), "{name}");
                }
                if matches!(registered, Builtin::FormulaSheet) {
                    assert!(info.matches("\"source_span\":{").count() >= 20, "{name}");
                }
            }
        }
    }

    #[test]
    fn native_builtin_templates_validate_config_and_leave_unused_engines_lazy() {
        let fs = fmn_platform::fs::VirtualFs::new();
        fs.insert(
            "/cfg/studio-template.yml",
            b"tex:\n  template: not-a-native-template\n".to_vec(),
        );
        let mut command = camera_command("circle_shift.v1");
        command.render.common.config_file = Some(PathBuf::from("/cfg/studio-template.yml"));
        // Public configuration validates pack names for every scene, before
        // constructing a worker or initializing a typesetting engine.
        for name in ordinary_builtin_names() {
            command.render.scene_names = vec![name.into()];
            let error = match worker(&fs, &command) {
                Ok(_) => panic!("invalid configured template accepted for {name}"),
                Err(error) => error,
            };
            assert_eq!(error.exit_name(), "config", "{name}");
            assert!(
                error.to_string().contains("not-a-native-template"),
                "{name}: {error}"
            );
        }

        // Exercise the factory below that admission boundary with an invalid
        // sentinel: any engine() request must fail, making eager initialization
        // observable without weakening the public configuration contract.
        let runtime = fmn_scene::RuntimeConfig {
            fps: 30,
            ..fmn_scene::RuntimeConfig::default()
        };
        let primitive = Builtin::named("circle_shift.v1").unwrap();
        let limit = primitive.frame_count(30).unwrap() - 1;
        let mut live = primitive
            .program(runtime.clone(), 71, limit, "not-a-native-template", 1)
            .unwrap();
        assert_eq!(live.preview().stage().roots().len(), 2);
        live.advance_to(limit).unwrap();
        for name in [
            fmn::builtins::TEX_SPAN_SCENE_NAME,
            fmn::builtins::FORMULA_SHEET_SCENE_NAME,
            fmn::builtins::SEMANTIC_WITNESS_SCENE_NAME,
            camera_route::SEMANTIC_WITNESS_CAMERA_SCENE_NAME,
        ] {
            let error = match Builtin::named(name).unwrap().program(
                runtime.clone(),
                71,
                limit,
                "not-a-native-template",
                1,
            ) {
                Ok(_) => panic!("required typesetting engine was not initialized for {name}"),
                Err(error) => error,
            };
            assert_eq!(error.code, fmn_studio::WorkerErrorCode::ExecutionFailed);
            assert!(
                error.to_string().contains("not-a-native-template"),
                "{name}: {error}"
            );
        }
    }

    #[test]
    fn native_builtin_sound_cue_is_deferred_until_its_exact_completed_boundary() {
        let name = fmn::builtins::SOUND_CUE_SCENE_NAME;
        for fps in [7, 30] {
            let runtime = fmn_scene::RuntimeConfig {
                fps,
                ..fmn_scene::RuntimeConfig::default()
            };
            let mut authored = resolve_builtin_program(name).unwrap();
            let mut captures = StudioPacketCapture::default();
            let ordinary =
                fmn::run_scene(&mut *authored, runtime.clone(), 71, &mut captures).unwrap();
            let registered = Builtin::named(name).unwrap();
            let frame_count = registered.frame_count(fps).unwrap();
            let mut live = registered
                .program(runtime, 71, frame_count, "default", 1)
                .unwrap();
            let boundary = if fps == 7 { 4 } else { 15 };
            assert!(live.preview().scene().sound_requests().is_empty());
            live.advance_to(boundary).unwrap();
            assert!(
                live.preview().scene().sound_requests().is_empty(),
                "pausing at the preceding capture must not execute future audio authoring"
            );
            live.advance_to(boundary + 1).unwrap();
            let requests = live.preview().scene().sound_requests().to_vec();
            assert_eq!(requests.len(), 1);
            assert_eq!(requests.as_slice(), ordinary.scene().sound_requests());
            assert_eq!(
                requests[0].sound_file.as_path(),
                Path::new(fmn::builtins::SOUND_CUE_ASSET_NAME)
            );
            assert_eq!(
                requests[0].time,
                fmn::animation::RationalTime::zero(fps) + boundary as i64
            );
            assert_eq!(requests[0].time_offset, 0.0);
            live.advance_to(frame_count - 1).unwrap();
            assert!(live.next_frame().unwrap().is_none());
            assert_eq!(live.preview().scene().sound_requests(), requests.as_slice());
        }
    }

    fn camera_command(name: &str) -> StudioCommand {
        let mut result = command();
        result.render.scene_names = vec![name.to_owned()];
        result.render.resolution = Some(ResolutionChoice::Exact(64, 48));
        result
    }

    fn scrub_camera(
        service: &mut dyn fmn_studio::WorkerService,
        name: &str,
        frame: i64,
    ) -> fmn_studio::FrameStream {
        let fmn_studio::WorkerResponse::Frame(frame) = service
            .handle(fmn_studio::SupervisorRequest::Scrub {
                scene: name.into(),
                frame,
            })
            .unwrap()
        else {
            panic!("camera frame")
        };
        assert_eq!(frame.render_backends.len(), 1);
        assert!(
            frame.render_backends[0]
                .identity()
                .windows(b"lumen-retained-camera-cpu-v1".len())
                .any(|bytes| bytes == b"lumen-retained-camera-cpu-v1")
        );
        frame
    }

    fn rgba(frame: &fmn_studio::FrameStream) -> Vec<u8> {
        let fmn_studio::FramePayload::Pipe { bytes, digest } = &frame.payload else {
            panic!("inline camera PNG")
        };
        assert_eq!(fmn_studio::protocol_digest(bytes), *digest);
        fmn_codec::decode_png(bytes, &fmn_codec::PngLimits::default())
            .unwrap()
            .rgba
    }

    fn serial_camera(
        stage: &fmn::mobject::Stage,
        camera: &fmn_render::Camera,
        renderer: RetainedFrameRendererConfig,
    ) -> Vec<u8> {
        let mut renderer = RetainedFrameRenderer::new(renderer).unwrap();
        renderer.render_with_camera(stage, camera).unwrap();
        let (width, height) = camera.pixel_shape();
        let mut frame =
            FrameBuffer::new(FrameLayout::tight(PixelFormat::Rgba8, width, height).unwrap());
        rgba16f_to_rgba8(renderer.frame(), &mut frame).unwrap();
        frame.as_bytes().to_vec()
    }

    #[test]
    fn camera_builtins_reach_live_studio_with_real_clock_pixels_and_replay_inputs() {
        let fs = fmn_platform::fs::VirtualFs::new();
        for name in [
            "surface_cube.v1",
            "dot_cloud_depth.v1",
            "image_quad.v1",
            "mixed_camera.v1",
        ] {
            let command = camera_command(name);
            assert_eq!(studio_scene_name(&fs, &command.render).unwrap(), name);
            let Configuration {
                native,
                runtime,
                seed,
                ..
            } = configuration(&fs, &command).unwrap();
            assert_eq!((native.fps, native.frame_count), (30, 9));
            let camera = fmn_render::Camera::new(native.camera.clone().unwrap()).unwrap();
            let mut capture = StudioPacketCapture::default();
            let mut program = camera_route::builtin(name).unwrap();
            let _ = fmn::run_scene(&mut program, runtime, seed, &mut capture).unwrap();
            assert_eq!(capture.packets.len(), 8);
            let expected: Vec<_> = capture
                .packets
                .iter()
                .map(|packet| serial_camera(&packet.materialize_stage(), &camera, native.renderer))
                .collect();
            assert_ne!(expected[0], expected[7]);
            assert_ne!(
                expected[0],
                serial_camera(&fmn::mobject::Stage::new(), &camera, native.renderer)
            );
            for threads in [1, 4] {
                let mut selected = command.clone();
                selected.render.common.threads = Some(threads);
                let expected_reads = source_reads(&fs, &selected).unwrap();
                assert_eq!(expected_reads.len(), 5);
                let mut service = worker(&fs, &selected).unwrap();
                for frame in [8, 1, 4, 8, 1] {
                    assert_eq!(
                        rgba(&scrub_camera(&mut *service, name, frame)),
                        expected[frame as usize - 1]
                    );
                }
                let initial = rgba(&scrub_camera(&mut *service, name, 0));
                assert_ne!(initial, expected[7]);
                assert!(service.journal_tail().is_empty());
                let fmn_studio::WorkerResponse::JournalSegment { journal, .. } = service
                    .handle(fmn_studio::SupervisorRequest::Play {
                        scene: name.into(),
                        command: fmn_studio::protocol::studio_seek_command(name, 4).unwrap(),
                    })
                    .unwrap()
                else {
                    panic!("committed camera seek")
                };
                let journal = fmn_scene::Journal::from_bytes(&journal).unwrap();
                assert_eq!(journal.entries().len(), 1);
                assert_eq!(journal.entries()[0].reads, expected_reads);
            }
        }
    }

    #[test]
    fn legacy_camera_bundles_reach_studio_without_a_camera_track_or_clock_retiming() {
        let fs = fmn_platform::fs::VirtualFs::new();
        for name in [
            "surface_cube.v1",
            "dot_cloud_depth.v1",
            "image_quad.v1",
            "mixed_camera.v1",
        ] {
            let mut recorder = fmn_scene::recording::SceneBundleRecorder::new_render_only(
                8,
                fmn_scene::BundleExportLimits::default(),
            )
            .unwrap();
            let mut program = camera_route::builtin(name).unwrap();
            let _ = fmn::run_scene(
                &mut program,
                fmn_scene::RuntimeConfig {
                    fps: 8,
                    ..fmn_scene::RuntimeConfig::default()
                },
                0,
                &mut recorder,
            )
            .unwrap();
            let source = recorder.finish().unwrap();
            let bundle = TimelineBundle::from_bytes(&source.bytes).unwrap();
            assert!(!bundle.has_camera_track());
            assert!(bundle.requires_camera());
            assert_eq!(bundle.frame_count(), 2);
            fs.insert("/camera.fmtl", source.bytes);
            let mut command = camera_command(name);
            command.render.file = Some(PathBuf::from("/camera.fmtl"));
            let view = camera_bundle_config(&fs, &command, &bundle).unwrap();
            let camera = view.fixed_camera.as_ref().unwrap();
            let expected: Vec<_> = (0..2)
                .map(|frame| serial_camera(&bundle.stage_at(frame).unwrap(), camera, view.renderer))
                .collect();
            assert_ne!(expected[0], expected[1]);
            let expected_reads = source_reads(&fs, &command).unwrap();
            assert_eq!(expected_reads.len(), 2);
            let mut service = worker(&fs, &command).unwrap();
            for frame in [1, 0, 1] {
                assert_eq!(
                    rgba(&scrub_camera(&mut *service, name, frame)),
                    expected[frame as usize]
                );
            }
            let fmn_studio::WorkerResponse::StudioData { bytes, .. } = service
                .handle(fmn_studio::SupervisorRequest::Inspect { scene: name.into() })
                .unwrap()
            else {
                panic!("camera inspection")
            };
            let info = String::from_utf8(bytes).unwrap();
            assert!(info.contains("\"fps\":8"));
            assert!(info.contains("\"frame_count\":2"));
            let fmn_studio::WorkerResponse::JournalSegment { journal, .. } = service
                .handle(fmn_studio::SupervisorRequest::Play {
                    scene: name.into(),
                    command: fmn_studio::protocol::studio_seek_command(name, 1).unwrap(),
                })
                .unwrap()
            else {
                panic!("compiled camera journal")
            };
            assert_eq!(
                fmn_scene::Journal::from_bytes(&journal).unwrap().entries()[0].reads,
                expected_reads
            );
        }
    }

    #[test]
    fn camera_studio_refuses_incompatible_engines_and_offline_flags_before_startup() {
        let fs = fmn_platform::fs::VirtualFs::new();
        let mut command = camera_command("mixed_camera.v1");
        command.render.engine = Some(EngineChoice::Metal);
        assert_eq!(
            source_reads(&fs, &command).unwrap_err().exit_name(),
            "capability"
        );
        assert!(worker(&fs, &command).is_err());
        command.render.engine = None;
        command.render.skip_animations = true;
        assert_eq!(
            source_reads(&fs, &command).unwrap_err().exit_name(),
            "config"
        );
        assert!(worker(&fs, &command).is_err());
    }
}
