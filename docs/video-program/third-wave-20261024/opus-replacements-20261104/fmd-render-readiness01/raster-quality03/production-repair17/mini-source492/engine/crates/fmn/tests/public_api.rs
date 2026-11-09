use fmn::prelude::*;

#[derive(Default)]
struct PacketSink {
    captures: usize,
}

impl SceneSink for PacketSink {
    fn capture(
        &mut self,
        _reason: CaptureReason,
        _packet: fmn::animation::FramePacket,
    ) -> std::result::Result<(), IntegrationError> {
        self.captures += 1;
        Ok(())
    }
}

#[derive(Default)]
struct PublicScene {
    circle: Option<Mob>,
    square: Option<Mob>,
}

impl SceneConstruct for PublicScene {
    fn name(&self) -> &str {
        "public_api_circle_shift"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let circle = stage.add(Circle::new().radius(0.75).color(BLUE_C))?;
        let square = stage.add(Square::new().side_length(1.25).color(YELLOW))?;
        self.circle = Some(circle);
        self.square = Some(square);

        let movement = circle
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.25),
                rate_func: Some(fmn::core::rate::linear),
                ..AnimateArgs::default()
            })?
            .shift([0.5, 0.25, 0.0])?;
        let square_movement = square
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.25),
                rate_func: Some(fmn::core::rate::linear),
                ..AnimateArgs::default()
            })?
            .shift([-0.25, 0.0, 0.0])?
            .rotate(PI / 4.0)?
            .set_opacity(0.5)?;
        stage.play((movement, square_movement))?;
        stage.wait(0.125)?;
        Ok(())
    }
}

#[test]
fn public_prelude_executes_the_real_scene_runtime() {
    let mut program = PublicScene::default();
    let mut sink = PacketSink::default();
    let completed = run_scene(
        &mut program,
        RuntimeConfig {
            fps: 8,
            ..RuntimeConfig::default()
        },
        0xFACADE,
        &mut sink,
    )
    .expect("public scene runs");

    assert_eq!(completed.report().play_count, 2);
    assert_eq!(sink.captures, 3, "two play frames plus one wait frame");
    let circle = program.circle.expect("construct stored its public handle");
    let square = program.square.expect("construct stored its square handle");
    assert!(completed.scene().stage().contains(circle));
    assert!(completed.scene().stage().contains(square));
    let center = completed.scene().stage().get_center(circle);
    let square_center = completed.scene().stage().get_center(square);
    assert!((center[0] - 0.5).abs() < 1e-9);
    assert!((center[1] - 0.25).abs() < 1e-9);
    assert!((square_center[0] + 0.25).abs() < 1e-9);
    for field in ["stroke_rgba", "fill_rgba"] {
        let rgba = completed
            .scene()
            .stage()
            .get(square)
            .and_then(|entry| entry.buffer.read_column(field))
            .expect("public square carries its vector style");
        assert!(rgba.iter().skip(3).step_by(4).all(|alpha| *alpha == 0.5));
    }
}

struct ForeignHandleScene {
    foreign: Mob,
}

impl SceneConstruct for ForeignHandleScene {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let _ = stage.play(show_creation(self.foreign))?;
        Ok(())
    }
}

#[test]
fn public_runner_preserves_the_typed_foreign_handle_refusal() {
    let mut other_stage = fmn::mobject::Stage::new();
    let foreign = other_stage.add(Circle::new());
    let mut program = ForeignHandleScene { foreign };
    let mut sink = PacketSink::default();

    let result = run_scene(&mut program, RuntimeConfig::default(), 1, &mut sink);
    assert!(result.is_err(), "foreign animation target unexpectedly ran");
    let error = result.err().expect("the asserted error is present");
    assert!(matches!(
        &error,
        fmn::Error::Animation(AnimError::StaleHandle(handle)) if *handle == foreign
    ));
    assert_eq!(error.kind(), ErrorKind::Scene);
    assert_eq!(sink.captures, 0);
}

#[test]
fn error_categories_match_the_cli_schema_rows() {
    let config = fmn::Error::from(fmn::config::ConfigError::Value {
        path: "camera.fps".to_owned(),
        message: "must be nonzero".to_owned(),
    });
    let capability = fmn::Error::from(fmn::platform::fetch::FetchError::CapabilityAbsent {
        url: "https://example.invalid/asset".to_owned(),
    });
    let scene = fmn::Error::from(fmn::geometry::GeomError::EmptyPath);
    let render_integration = fmn::Error::from(fmn::scene::SceneError::Integration(
        fmn::scene::IntegrationError::new("test-sink", "publication failed"),
    ));
    let render = fmn::Error::from(fmn::platform::fs::FsError::NotFound {
        path: "missing.png".into(),
    });
    let budget = fmn::Error::from(fmn::platform::fetch::FetchError::TooLarge {
        url: "https://example.invalid/large".to_owned(),
        limit: 16,
    });

    assert_eq!((config.kind().name(), config.kind().code()), ("config", 3));
    assert_eq!(
        (capability.kind().name(), capability.kind().code()),
        ("capability", 4)
    );
    assert_eq!((scene.kind().name(), scene.kind().code()), ("scene", 5));
    assert_eq!(
        (
            render_integration.kind().name(),
            render_integration.kind().code()
        ),
        ("render", 6)
    );
    assert_eq!((render.kind().name(), render.kind().code()), ("render", 6));
    assert_eq!((budget.kind().name(), budget.kind().code()), ("budget", 8));
}

#[derive(Default)]
struct EarlyEndScene;

impl SceneConstruct for EarlyEndScene {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.end()
    }
}

#[test]
fn public_runner_preserves_normal_early_termination() {
    let mut sink = PacketSink::default();
    let completed = run_scene(&mut EarlyEndScene, RuntimeConfig::default(), 2, &mut sink)
        .expect("normal early termination is not an error");

    assert!(completed.report().ended_early);
    assert_eq!(completed.report().play_count, 0);
    assert_eq!(sink.captures, 0);
}

#[test]
fn public_prelude_constructs_expanded_mobjects() {
    let book = FontBook::bundled().expect("bundled fonts");

    let code = Code::new("fn main() {}").language("rust").build(&book);
    assert!(code.is_ok());

    let md = Markdown::new("# Header\n\nText").build(&book);
    assert!(md.is_ok());

    let table = TableMobject::from_csv("a,b\n1,2", ',')
        .unwrap()
        .build(&book);
    assert!(table.is_ok());

    let chart = BarChart::new(vec![1.0, 2.0, 3.0]).build(&book);
    assert!(chart.is_ok());

    let space = SampleSpace::new().build();
    assert!(space.is_ok());

    let graph = NetworkGraph::from_edge_list(&["a", "b"], &[("a", "b")])
        .laid_out(&GraphLayout::Circular)
        .unwrap()
        .build();
    assert!(graph.is_ok());

    let brace = Brace::new().build();
    assert!(!brace.points().is_empty());

    let dec = DecimalNumber::new(std::f64::consts::PI).build(&book);
    assert!(dec.is_ok());

    let int_mob = Integer::new(42.0).build(&book);
    assert!(int_mob.is_ok());

    let sphere = Sphere::new(1.0).build();
    assert!(!sphere.points().is_empty());

    let cube = Cube::new(1.0).build();
    assert!(!cube.children().is_empty());
}

#[test]
fn sound_cue_scene_records_an_exact_rational_request() {
    let mut program = fmn::builtins::sound_scene(fmn::builtins::SOUND_CUE_SCENE_NAME)
        .expect("the sound-cue scene is registered");
    let mut sink = PacketSink::default();
    let completed = run_scene(
        &mut program,
        RuntimeConfig {
            fps: 8,
            ..RuntimeConfig::default()
        },
        0,
        &mut sink,
    )
    .expect("the sound-cue scene runs");

    let requests = completed.scene().sound_requests();
    assert_eq!(requests.len(), 1);
    let request = &requests[0];
    assert_eq!(
        request.sound_file,
        std::path::PathBuf::from(fmn::builtins::SOUND_CUE_ASSET_NAME)
    );
    assert_eq!(request.time.frames(), 4, "0.5 s at 8 fps");
    assert_eq!(request.time.fps(), 8);
    assert_eq!(request.time_offset, 0.0);
    assert_eq!(request.gain, None);
    assert_eq!(request.gain_to_background, None);

    assert!(fmn::builtins::sound_scene("circle_shift.v1").is_none());
    assert!(fmn::builtins::primitive_scene(fmn::builtins::SOUND_CUE_SCENE_NAME).is_none());
}
