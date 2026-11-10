#![forbid(unsafe_code)]
#![allow(clippy::expect_used, clippy::panic)]

#[cfg(feature = "batch")]
use std::ffi::OsString;
#[cfg(feature = "batch")]
use std::fs;
#[cfg(feature = "batch")]
use std::path::{Path, PathBuf};
#[cfg(feature = "batch")]
use std::process::{Command, Output, Stdio};
#[cfg(feature = "batch")]
use std::sync::atomic::{AtomicU64, Ordering};

#[test]
fn parser_smoke_remains_available_without_a_partial_shipping_binary() {
    let invocation = fmn_cli::parse_args(["doctor", "--robot"])
        .expect("the schema-generated doctor command parses");
    assert!(matches!(
        invocation,
        fmn_cli::Invocation::Doctor(fmn_cli::DoctorCommand {
            common: fmn_cli::CommonOptions { robot: true, .. },
            ..
        })
    ));
}

#[cfg(feature = "batch")]
static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

#[cfg(feature = "batch")]
struct Fixture {
    root: PathBuf,
}

#[cfg(feature = "batch")]
impl Fixture {
    fn new() -> Self {
        for _ in 0..1_024 {
            let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir()
                .join(format!("fmn-cli-smoke-{}-{sequence}", std::process::id()));
            match fs::create_dir(&root) {
                Ok(()) => return Self { root },
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => {
                    panic!("could not create CLI smoke fixture {root:?}: {error}");
                }
            }
        }
        panic!("could not allocate a unique CLI smoke fixture");
    }

    fn cache(&self) -> PathBuf {
        self.root.join("cache")
    }

    fn missing_ffmpeg(&self) -> PathBuf {
        self.root.join("definitely-missing-ffmpeg")
    }
}

#[cfg(feature = "batch")]
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[cfg(feature = "batch")]
fn run(fixture: &Fixture, args: Vec<OsString>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_fmn"));
    command
        .args(args)
        .current_dir(&fixture.root)
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("HOME")
        .env_remove("APPDATA")
        .env_remove("USERPROFILE")
        .stdin(Stdio::null());
    command.output().expect("the shipped fmn binary runs")
}

#[cfg(feature = "batch")]
fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(|value| OsString::from(*value)).collect()
}

#[cfg(feature = "batch")]
fn doctor_args(fixture: &Fixture, extra: &[&str]) -> Vec<OsString> {
    let mut values = args(&["doctor"]);
    values.extend(extra.iter().map(|value| OsString::from(*value)));
    values.push(OsString::from("--ffmpeg"));
    values.push(fixture.missing_ffmpeg().into_os_string());
    values.push(OsString::from("--cache-dir"));
    values.push(fixture.cache().into_os_string());
    values
}

#[cfg(feature = "batch")]
fn write_synthetic_timeline(path: &Path) {
    let mut stage = fmn_scene::studio_bridge::Stage::new();
    let mut timeline = fmn_scene::studio_bridge::Timeline::new(4).expect("valid fixture FPS");
    timeline.wait(0.25).expect("one-frame wait segment");
    let bytes = fmn_scene::export_timeline_bundle(
        timeline,
        &mut stage,
        &fmn_core::rng::RngRoot::from_seed(0),
    )
    .expect("synthetic FMTL export");
    fs::write(path, bytes).expect("write synthetic FMTL scene");
}

#[cfg(feature = "batch")]
struct SyntheticBatch {
    argv: Vec<OsString>,
    artifact: PathBuf,
    manifest_binary: PathBuf,
    manifest_text: PathBuf,
}

#[cfg(feature = "batch")]
fn synthetic_batch(fixture: &Fixture) -> SyntheticBatch {
    const SCENE: &str = "synthetic_batch";

    let source = fixture.root.join(format!("{SCENE}.fmtl"));
    let output_root = fixture.root.join("batch-output");
    let manifest_root = fixture.root.join("batch-manifests");
    fs::create_dir(&output_root).expect("create batch output root");
    fs::create_dir(&manifest_root).expect("create batch manifest root");
    write_synthetic_timeline(&source);

    let mut argv = args(&[
        "batch",
        "--robot",
        "--format",
        "png",
        "--resolution",
        "16x16",
        "--fps",
        "4",
        "--threads",
        "1",
        "--max-scenes",
        "1",
        "--manifest-dir",
    ]);
    argv.push(manifest_root.clone().into_os_string());
    argv.push(OsString::from("--video_dir"));
    argv.push(output_root.clone().into_os_string());
    argv.push(source.into_os_string());

    SyntheticBatch {
        argv,
        artifact: output_root.join(format!("{SCENE}.png")),
        manifest_binary: manifest_root.join(SCENE).join("manifest.fmnp"),
        manifest_text: manifest_root.join(SCENE).join("manifest.txt"),
    }
}

#[cfg(feature = "batch")]
fn stdout(output: &Output) -> &str {
    std::str::from_utf8(&output.stdout).expect("stdout is UTF-8")
}

#[cfg(feature = "batch")]
fn stderr(output: &Output) -> &str {
    std::str::from_utf8(&output.stderr).expect("stderr is UTF-8")
}

#[cfg(feature = "batch")]
fn record_kind(line: &str) -> &str {
    let (_, tail) = line
        .split_once("\"kind\":\"")
        .expect("robot record carries a kind");
    tail.split_once('"')
        .expect("robot kind has a closing quote")
        .0
}

#[cfg(feature = "batch")]
fn assert_code(output: &Output, expected: i32) {
    assert_eq!(
        output.status.code(),
        Some(expected),
        "unexpected status; stdout={} stderr={}",
        stdout(output),
        stderr(output)
    );
}

#[cfg(feature = "batch")]
#[test]
fn shipped_binary_reports_version_and_generated_help() {
    let fixture = Fixture::new();

    let version = run(&fixture, args(&["--version"]));
    assert_code(&version, 0);
    assert_eq!(
        stdout(&version),
        format!("fmn {}\n", env!("CARGO_PKG_VERSION"))
    );
    assert!(stderr(&version).is_empty());

    let help = run(&fixture, args(&["--help"]));
    assert_code(&help, 0);
    assert!(stderr(&help).is_empty());
    assert!(stdout(&help).contains("Usage"));
    assert!(stdout(&help).contains("fmn"));
    for command in ["render", "doctor", "batch", "studio"] {
        assert!(
            stdout(&help).contains(command),
            "generated help omitted {command:?}: {}",
            stdout(&help)
        );
    }
}

#[cfg(feature = "batch")]
#[test]
fn shipped_doctor_robot_stream_has_the_complete_versioned_schema() {
    let fixture = Fixture::new();
    let output = run(&fixture, doctor_args(&fixture, &["--robot"]));
    assert_code(&output, 0);
    assert!(stderr(&output).is_empty());
    assert!(stdout(&output).ends_with('\n'));

    let lines = stdout(&output).lines().collect::<Vec<_>>();
    assert_eq!(
        lines
            .iter()
            .map(|line| record_kind(line))
            .collect::<Vec<_>>(),
        [
            "topology",
            "execution_plan",
            "ffmpeg",
            "cache",
            "fonts",
            "math_packs",
            "certification",
        ]
    );
    for line in lines {
        assert!(
            line.starts_with("{\"schema\":\"fmn.doctor\",\"version\":1,"),
            "noncanonical doctor record: {line}"
        );
        assert!(line.ends_with('}'), "unterminated doctor record: {line}");
    }
}

#[cfg(feature = "batch")]
#[test]
fn shipped_batch_renders_a_synthetic_scene_and_publishes_its_manifest() {
    let fixture = Fixture::new();
    let batch = synthetic_batch(&fixture);

    let output = run(&fixture, batch.argv.clone());
    assert_code(&output, 0);
    assert!(stderr(&output).is_empty());
    let lines = stdout(&output).lines().collect::<Vec<_>>();
    assert_eq!(
        lines
            .iter()
            .map(|line| record_kind(line))
            .collect::<Vec<_>>(),
        ["render", "batch"]
    );
    assert!(lines[0].contains("\"source\":\"compiled\""));
    assert!(lines[0].contains("\"scene\":\"synthetic_batch\""));
    assert!(lines[0].contains("\"format\":\"png\""));
    assert!(lines[0].contains("\"frames\":1"));
    assert!(lines[0].contains("\"manifest\":"));
    assert!(lines[1].contains("\"status\":\"ok\""));
    assert!(lines[1].contains("\"jobs\":1"));
    assert!(lines[1].contains("\"succeeded\":1"));
    assert!(lines[1].contains("\"failed\":0"));
    assert!(lines[1].contains("\"cancelled\":0"));
    assert!(lines[1].contains("\"max_scenes\":1"));

    let png = fs::read(&batch.artifact).expect("batch publishes its PNG");
    assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
    assert!(png.len() > 8);
    let manifest_binary = fs::read(&batch.manifest_binary).expect("batch publishes manifest.fmnp");
    let manifest_text =
        fs::read_to_string(&batch.manifest_text).expect("batch publishes UTF-8 manifest.txt");
    assert!(!manifest_binary.is_empty());
    assert!(!manifest_text.is_empty());

    let retry = run(&fixture, batch.argv);
    assert_ne!(retry.status.code(), Some(0));
    assert!(stderr(&retry).is_empty());
    assert_eq!(stdout(&retry).lines().count(), 1);
    assert_eq!(record_kind(stdout(&retry).trim_end()), "error");
    assert!(stdout(&retry).contains("\"exit_name\":\"render\""));
    assert!(stdout(&retry).contains("already exists"));
    assert_eq!(
        fs::read(&batch.artifact).expect("first artifact remains intact"),
        png
    );
    assert_eq!(
        fs::read(&batch.manifest_binary).expect("first manifest remains intact"),
        manifest_binary
    );
}

#[cfg(feature = "batch")]
#[test]
fn shipped_binary_preserves_typed_usage_and_capability_exit_codes() {
    let fixture = Fixture::new();

    let usage = run(&fixture, args(&["--robot", "-l", "-m"]));
    assert_code(&usage, 2);
    assert!(stderr(&usage).is_empty());
    assert_eq!(stdout(&usage).lines().count(), 1);
    assert!(stdout(&usage).contains("\"kind\":\"error\""));
    assert!(stdout(&usage).contains("\"exit_name\":\"usage\""));
    assert!(stdout(&usage).contains("\"rule\":\"quality-exclusive\""));

    let capability = run(
        &fixture,
        doctor_args(&fixture, &["--robot", "--require-ffmpeg"]),
    );
    assert_code(&capability, 4);
    assert!(stderr(&capability).is_empty());
    let lines = stdout(&capability).lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 8);
    assert_eq!(record_kind(lines[7]), "error");
    assert!(lines[7].contains("\"exit_name\":\"capability\""));
    assert!(lines[7].contains("native PNG-sequence, GIF, and y4m outputs remain available"));
}

#[cfg(feature = "batch")]
#[test]
fn shipped_doctor_quiet_mode_suppresses_only_non_error_output() {
    let fixture = Fixture::new();

    let success = run(&fixture, doctor_args(&fixture, &["--quiet"]));
    assert_code(&success, 0);
    assert!(stdout(&success).is_empty());
    assert!(stderr(&success).is_empty());

    let failure = run(
        &fixture,
        doctor_args(&fixture, &["--quiet", "--require-ffmpeg"]),
    );
    assert_code(&failure, 4);
    assert!(stdout(&failure).is_empty());
    assert!(stderr(&failure).contains("ffmpeg was required but is unavailable"));
}

#[cfg(feature = "batch")]
#[test]
fn smoke_fixture_paths_are_absolute_and_do_not_depend_on_repo_state() {
    let fixture = Fixture::new();
    assert!(fixture.root.is_absolute());
    assert!(!fixture.root.join("custom_config.yml").exists());
}

#[test]
fn compiled_simd_tier_matches_host_or_fails_cleanly_with_guidance() {
    let result = fmn_cli::check_simd_tier_support();
    let active = fmn_cli::active_compiled_simd_tier();
    match result {
        Ok(()) => {
            assert!(!active.is_empty());
        }
        Err(err) => {
            assert_eq!(err.code(), 4);
            assert_eq!(err.exit_name(), "capability");
            assert!(err.message().contains("not supported by your CPU"));
            assert!(err.message().contains("install.sh"));
        }
    }
}

#[cfg(feature = "batch")]
fn y4m_rate_and_frames(path: &Path) -> (String, usize) {
    let bytes = fs::read(path).expect("read y4m artifact");
    let header_end = bytes
        .iter()
        .position(|&b| b == b'\n')
        .expect("y4m header line");
    let header = std::str::from_utf8(&bytes[..header_end]).expect("ASCII y4m header");
    let rate = header
        .split(' ')
        .find_map(|field| field.strip_prefix('F'))
        .expect("y4m frame rate field")
        .to_owned();
    let frames = bytes
        .windows(6)
        .filter(|window| window == b"FRAME\n")
        .count();
    (rate, frames)
}

/// fm-cli-flag-timing-ht01: window-only flags once switched the scene clock
/// to the 30 fps preview rate while the y4m kept the configured rate (`-f
/// --fps 60` wrote 12 frames into an `F60:1` file); progress flags were
/// accepted and ignored. A file render now refuses them by name with the
/// capability exit and publishes nothing; the plain render keeps its timing.
#[cfg(feature = "batch")]
#[test]
fn window_and_progress_flags_are_refused_and_never_mistime_a_file_render() {
    let fixture = Fixture::new();
    let render = |name: &str, extra: &[&str]| {
        let output = fixture.root.join(name);
        let mut argv = args(&[
            "--robot",
            "--format",
            "y4m",
            "--resolution",
            "64x36",
            "--fps",
            "60",
        ]);
        argv.extend(extra.iter().map(|value| OsString::from(*value)));
        argv.extend(args(&["--video_dir"]));
        argv.push(output.clone().into_os_string());
        argv.extend(args(&["@builtin", "circle_shift.v1"]));
        // Log the argv and the runtime config the front door resolves for it.
        let text: Vec<String> = argv
            .iter()
            .map(|value| value.to_string_lossy().into_owned())
            .collect();
        if let Ok(fmn_cli::Invocation::Render(command)) = fmn_cli::parse_args(text.clone()) {
            let config = fmn_cli::resolve_render_config(&fmn_platform::fs::StdFs, &command)
                .expect("the render config resolves");
            let runtime = command.runtime_config(&config);
            eprintln!(
                "argv={text:?} fps={} windowed={} write_file={}",
                config.camera.fps, runtime.windowed, command.write_file
            );
        }
        (run(&fixture, argv), output)
    };

    let (plain, plain_dir) = render("plain", &[]);
    assert_code(&plain, 0);
    let artifact = fs::read_dir(&plain_dir)
        .expect("plain output directory")
        .map(|entry| entry.expect("directory entry").path())
        .find(|path| path.extension().is_some_and(|ext| ext == "y4m"))
        .expect("a y4m artifact");
    let observed = y4m_rate_and_frames(&artifact);
    eprintln!("plain: header rate {} frames {}", observed.0, observed.1);
    assert_eq!(observed, ("60:1".to_owned(), 23));

    for (name, flags) in [
        ("presenter", &["-p"][..]),
        ("fullscreen", &["-f"][..]),
        ("reload", &["--autoreload"][..]),
        ("embed", &["-e", "3"][..]),
        ("progress", &["--show_animation_progress"][..]),
        ("bars", &["--leave_progress_bars"][..]),
    ] {
        let (refused, dir) = render(name, flags);
        eprintln!("{name}: exit {:?}", refused.status.code());
        assert_code(&refused, 4);
        assert!(
            !dir.exists() || fs::read_dir(&dir).expect("output dir").next().is_none(),
            "{flags:?} published output before refusing"
        );
    }
}

/// The config keys `fmn_cli::resolve_render_config` writes from flags: every
/// `pairs.push((` site's first string literal.
fn flag_written_keys(cli: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let mut rest = cli;
    while let Some(at) = rest.find("pairs.push((") {
        rest = &rest[at + "pairs.push((".len()..];
        let literal = rest.trim_start();
        if let Some(body) = literal.strip_prefix('"')
            && let Some(end) = body.find('"')
        {
            keys.push(body[..end].to_owned());
        }
    }
    keys.sort();
    keys.dedup();
    keys
}

/// Shared front doors may pass `&config.file_writer` into a typed helper.
/// Follow that parameter within its own function body instead of treating a
/// shortened `writer.field` access as an unread configuration key.
fn borrowed_file_writer_reads(source: &str, field: &str) -> bool {
    source.split("fn ").skip(1).any(|function| {
        let Some((signature, body)) = function.split_once('{') else {
            return false;
        };
        let Some((_, parameters)) = signature.split_once('(') else {
            return false;
        };
        let Some((parameters, _)) = parameters.rsplit_once(')') else {
            return false;
        };
        let parameter = parameters.split(',').find_map(|parameter| {
            let (name, ty) = parameter.split_once(':')?;
            let ty: String = ty.split_whitespace().collect();
            (ty == "&fmn_config::config::FileWriterConfig").then(|| name.trim())
        });
        let Some(parameter) = parameter else {
            return false;
        };
        let mut depth = 1_u32;
        let end = body.char_indices().find_map(|(at, ch)| {
            match ch {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {}
            }
            (depth == 0).then_some(at)
        });
        let Some(end) = end else {
            return false;
        };
        let body: String = body[..end].split_whitespace().collect();
        let access = format!("{parameter}.{field}");
        let identifier = |ch: char| ch.is_ascii_alphanumeric() || ch == '_';
        body.match_indices(&access).any(|(at, _)| {
            !body[..at].chars().next_back().is_some_and(identifier)
                && !body[at + access.len()..]
                    .chars()
                    .next()
                    .is_some_and(identifier)
        })
    })
}

/// Keys no source reads, other than those whose flag is refused for a file
/// render and exercised by `refusals` (the refusal test's flag table).
fn orphan_keys(
    keys: &[String],
    sources: &[String],
    refused: &[(&str, &str)],
    refusals: &str,
) -> Vec<String> {
    keys.iter()
        .filter(|key| {
            let read = sources.iter().any(|source| {
                source.contains(&format!(".{key}"))
                    || key
                        .strip_prefix("file_writer.")
                        .is_some_and(|field| borrowed_file_writer_reads(source, field))
            });
            let refused_and_tested = refused.iter().any(|(refused_key, flag)| {
                refused_key == key && refusals.contains(&format!("&[\"{flag}\""))
            });
            !read && !refused_and_tested
        })
        .cloned()
        .collect()
}

/// fm-cli-flag-timing-ht01: a flag whose config key nothing reads is a silent
/// no-op. Every key a render flag writes has a reader outside fmn-config, or
/// its flag is refused for file renders and covered by the refusal test.
#[test]
fn every_flag_written_config_key_has_a_reader_or_a_refused_flag() {
    const REFUSED: [(&str, &str); 4] = [
        ("window.full_screen", "-f"),
        ("embed.autoreload", "--autoreload"),
        ("scene.show_animation_progress", "--show_animation_progress"),
        ("scene.leave_progress_bars", "--leave_progress_bars"),
    ];
    let crates = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let cli = std::fs::read_to_string(crates.join("fmn-cli/src/lib.rs")).expect("fmn-cli source");
    let keys = flag_written_keys(&cli);
    assert!(keys.len() >= 10, "found only {keys:?}");
    let mut sources = Vec::new();
    let mut pending = vec![crates.clone()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).expect("read source directory") {
            let path = entry.expect("directory entry").path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if path.is_dir() {
                if !matches!(name, "target" | "tests" | "fmn-config") && !name.starts_with('.') {
                    pending.push(path);
                }
            } else if name.ends_with(".rs") && name != "generated.rs" {
                sources.push(std::fs::read_to_string(&path).expect("read source"));
            }
        }
    }
    let smoke = include_str!("cli_smoke.rs");
    let refusals = smoke
        .split("fn window_and_progress_flags_are_refused_and_never_mistime_a_file_render")
        .nth(1)
        .and_then(|body| body.split("\n}\n").next())
        .expect("the refusal test exists");
    let orphans = orphan_keys(&keys, &sources, &REFUSED, refusals);
    assert!(
        orphans.is_empty(),
        "flag-written config keys nothing reads: {orphans:?}"
    );

    // Planted negatives: an unread key, and a refused flag the refusal test
    // does not exercise.
    let planted = vec!["camera.fps".to_owned(), "window.made_up".to_owned()];
    let reader = vec!["config.camera.fps".to_owned()];
    assert_eq!(
        orphan_keys(&planted, &reader, &REFUSED, refusals),
        ["window.made_up"]
    );
    let untested = vec!["window.full_screen".to_owned()];
    assert_eq!(
        orphan_keys(&untested, &reader, &REFUSED, "&[\"-p\"]"),
        untested
    );

    // Typed borrowed readers count only fields that their own body accesses.
    // A similarly named field or an access in another function is not proof.
    let borrowed = vec![
        "fn quality(writer: &fmn_config::config::FileWriterConfig) {\n\
             writer\n.video_crf; writer.video_preset_extra;\n\
         }\n\
         fn unrelated(writer: &OtherConfig) { writer.video_tune; }\n\
         fn another() { writer.video_bitrate; }"
            .to_owned(),
    ];
    let candidates = ["video_crf", "video_preset", "video_tune", "video_bitrate"]
        .map(|field| format!("file_writer.{field}"));
    assert_eq!(
        orphan_keys(&candidates, &borrowed, &REFUSED, refusals),
        candidates[1..]
    );
}
