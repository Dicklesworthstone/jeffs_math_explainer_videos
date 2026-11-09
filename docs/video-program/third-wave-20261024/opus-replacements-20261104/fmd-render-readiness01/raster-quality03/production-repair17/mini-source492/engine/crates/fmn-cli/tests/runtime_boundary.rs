#![forbid(unsafe_code)]

// The io traits serve only the unix-gated Studio front-door test; an
// unconditional import is dead (and clippy-fatal) on Windows.
#[cfg(unix)]
use std::io::{BufRead as _, Read as _, Write as _};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use fmn_cli::{BUILTIN_SCENE_SOURCE, PYTHON_SOURCE_PORTAL_MESSAGE};
use fmn_core::rng::RngRoot;
use fmn_output::{ManifestMode, ProvenanceManifest};
use fmn_scene::export_timeline_bundle;
use fmn_scene::studio_bridge::{Stage, Timeline};

static RUN_SUFFIX: AtomicU64 = AtomicU64::new(0);

fn run_clean(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_fmn"))
        .args(args)
        .env_remove("PYTHONHOME")
        .env_remove("PYTHONPATH")
        .env("PATH", "")
        .output()
        .expect("launch the standalone fmn binary by absolute path")
}

fn output_root(label: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "fmn-cli-{label}-{}-{}",
        std::process::id(),
        RUN_SUFFIX.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&root).expect("create fresh CLI output root");
    root
}

fn compiled_wait_bundle(root: &std::path::Path) -> std::path::PathBuf {
    let path = root.join("compiled_wait.fmtl");
    write_compiled_wait_bundle(&path, 0.25);
    path
}

fn write_compiled_wait_bundle(path: &std::path::Path, seconds: f64) {
    let mut timeline = Timeline::new(8).expect("valid bundle fps");
    timeline.wait(seconds).expect("valid bundle wait");
    let bytes = export_timeline_bundle(timeline, &mut Stage::new(), &RngRoot::from_seed(0))
        .expect("compile wait artifact");
    std::fs::write(path, bytes).expect("write compiled artifact");
}

fn json_string_field<'a>(record: &'a str, field: &str) -> Option<&'a str> {
    let prefix = format!("\"{field}\":\"");
    record
        .split_once(&prefix)
        .and_then(|(_, tail)| tail.split_once('"'))
        .map(|(value, _)| value)
}

#[cfg(all(feature = "metal", target_os = "macos"))]
fn json_u128_field(record: &str, field: &str) -> Option<u128> {
    let prefix = format!("\"{field}\":");
    let tail = record.split_once(&prefix)?.1;
    let digits = tail
        .as_bytes()
        .iter()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    (digits != 0).then(|| tail[..digits].parse().ok()).flatten()
}

#[cfg(unix)]
fn studio_post(authority: &str, target: &str, body: &[u8]) -> String {
    let mut stream = std::net::TcpStream::connect(authority).expect("connect to Studio API");
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .expect("Studio API timeout");
    write!(
        stream,
        "POST {target} HTTP/1.1\r\nHost: {authority}\r\nOrigin: http://{authority}\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .expect("write Studio API request");
    stream.write_all(body).expect("write Studio API body");
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .expect("read Studio API response");
    let _ = stream.shutdown(std::net::Shutdown::Both);
    response
}

#[test]
fn standalone_binary_starts_without_a_python_runtime_on_path() {
    let output = run_clean(&["--version"]);

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert!(
        String::from_utf8(output.stdout)
            .expect("version output is UTF-8")
            .starts_with("fmn ")
    );
}

#[test]
fn robot_version_reports_compile_provenance_for_perf_identity() {
    let output = run_clean(&["--robot", "--version"]);
    let stdout = String::from_utf8(output.stdout).expect("version output is UTF-8");

    assert_eq!(output.status.code(), Some(0), "{stdout}");
    assert!(output.stderr.is_empty());
    assert_eq!(stdout.lines().count(), 1, "{stdout}");
    assert!(stdout.contains("\"kind\":\"version\""), "{stdout}");
    assert!(stdout.contains("\"build_id\":"), "{stdout}");
    assert!(stdout.contains("\"target_triple\":"), "{stdout}");
    assert!(stdout.contains("\"cargo_profile\":"), "{stdout}");
    assert!(stdout.contains("\"compiled_tier\":"), "{stdout}");
    assert!(stdout.contains("\"suite_lock_digest\":"), "{stdout}");
}

#[test]
fn python_source_is_refused_before_file_or_process_access() {
    for args in [
        &["--robot", "missing-scene.py", "Demo"][..],
        &["--robot", "studio", "missing-scene.PYW", "Demo"],
        &["--robot", "batch", "missing-scene.py", "Demo"],
    ] {
        let output = run_clean(args);
        let stdout = String::from_utf8(output.stdout).expect("robot output is UTF-8");

        assert_eq!(output.status.code(), Some(4), "{args:?}");
        assert!(output.stderr.is_empty(), "{args:?}");
        assert_eq!(stdout.lines().count(), 1, "{args:?}");
        assert!(stdout.contains("\"exit_name\":\"capability\""), "{args:?}");
        assert!(stdout.contains(PYTHON_SOURCE_PORTAL_MESSAGE), "{args:?}");
        assert!(!stdout.contains("composition is unavailable"), "{args:?}");
    }
}

#[cfg(unix)]
#[test]
fn studio_serves_a_real_worker_frame_and_shuts_down_on_stdin_eof() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_fmn"))
        .args([
            "studio",
            "--robot",
            "--no-browser",
            "--resolution",
            "96x54",
            "--fps",
            "8",
            "--threads",
            "1",
            BUILTIN_SCENE_SOURCE,
            "circle_shift.v1",
        ])
        .env_remove("PYTHONHOME")
        .env_remove("PYTHONPATH")
        .env("PATH", "")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("launch Studio front door");
    let mut stdout = std::io::BufReader::new(child.stdout.take().expect("Studio stdout"));
    let mut ready = String::new();
    stdout
        .read_line(&mut ready)
        .expect("read Studio readiness record");
    assert!(ready.contains("\"kind\":\"studio_ready\""), "{ready}");
    assert!(ready.contains("\"worker_generation\":1"), "{ready}");
    let url = json_string_field(&ready, "url").expect("readiness URL");
    let location = url.strip_prefix("http://").expect("loopback HTTP URL");
    let (authority, target) = location.split_once('/').expect("URL authority and target");
    let query = target.split_once('?').expect("capability query").1;

    let body = b"frame=1&commit=false";
    let mut scrub = std::net::TcpStream::connect(authority).expect("connect to Studio scrub API");
    scrub
        .set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .expect("scrub timeout");
    write!(
        scrub,
        "POST /api/scrub?{query} HTTP/1.1\r\nHost: {authority}\r\nOrigin: http://{authority}\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .expect("write scrub request");
    scrub.write_all(body).expect("write scrub body");
    let mut scrub_response = Vec::new();
    scrub
        .read_to_end(&mut scrub_response)
        .expect("read scrub response");
    let _ = scrub.shutdown(std::net::Shutdown::Both);
    let scrub_response = String::from_utf8(scrub_response).expect("scrub response is UTF-8");
    assert!(
        scrub_response.starts_with("HTTP/1.1 200 OK"),
        "{scrub_response}"
    );
    assert!(
        scrub_response.contains("\"frame_index\":1"),
        "{scrub_response}"
    );

    let mut stream = std::net::TcpStream::connect(authority).expect("connect to Studio stream");
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .expect("stream timeout");
    write!(
        stream,
        "GET /stream?{query} HTTP/1.1\r\nHost: {authority}\r\nConnection: close\r\n\r\n"
    )
    .expect("write stream request");
    let mut response = Vec::new();
    let mut chunk = [0_u8; 4096];
    let contains_frame_one_png = |bytes: &[u8]| {
        bytes
            .windows(20)
            .position(|window| window == b"X-FMN-Frame-Index: 1")
            .is_some_and(|header| {
                bytes[header..]
                    .windows(8)
                    .any(|window| window == b"\x89PNG\r\n\x1a\n")
            })
    };
    while !contains_frame_one_png(&response) {
        let read = stream
            .read(&mut chunk)
            .expect("read Studio multipart stream");
        assert_ne!(read, 0, "Studio stream closed before frame one's PNG");
        response.extend_from_slice(&chunk[..read]);
    }
    assert!(response.starts_with(b"HTTP/1.1 200 OK\r\n"));
    assert!(
        response
            .windows(20)
            .any(|window| window == b"X-FMN-Frame-Index: 1")
    );
    let _ = stream.shutdown(std::net::Shutdown::Both);
    drop(stream);
    drop(stdout);
    drop(child.stdin.take());
    let status = child.wait().expect("wait for graceful Studio shutdown");
    let mut stderr = String::new();
    child
        .stderr
        .take()
        .expect("Studio stderr")
        .read_to_string(&mut stderr)
        .expect("read Studio stderr");
    assert_eq!(status.code(), Some(0), "{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[cfg(all(unix, feature = "metal", not(target_os = "macos")))]
#[test]
fn studio_metal_request_uses_the_declared_cpu_stream_fallback() -> Result<(), String> {
    let root = output_root("studio-metal-fallback");
    let config = root.join("fmn.yml");
    std::fs::write(&config, b"render:\n  engine: metal\n").expect("write Metal config");
    let invocation = fmn_cli::parse_args([
        "studio",
        "--no-browser",
        "--config_file",
        config.to_str().expect("config path is UTF-8"),
        "--resolution",
        "96x54",
        "--threads",
        "1",
        BUILTIN_SCENE_SOURCE,
        "circle_shift.v1",
    ])
    .expect("parse Metal Studio composition");
    let fmn_cli::Invocation::Studio(command) = invocation else {
        return Err("Metal Studio invocation parsed to another front door".to_owned());
    };
    let composed = fmn_cli::compose_studio_preview_frame(&fmn_platform::fs::StdFs, &command, 0)
        .expect("compose Linux CPU fallback frame");
    assert_eq!(composed.render_backends.len(), 1);
    assert_eq!(
        composed.render_backends[0].role(),
        fmn_scene::RenderBackendRole::FrameStream
    );
    assert!(!composed.render_backends[0].identity().is_empty());

    let mut child = Command::new(env!("CARGO_BIN_EXE_fmn"))
        .args([
            "studio",
            "--robot",
            "--no-browser",
            "--config_file",
            config.to_str().expect("config path is UTF-8"),
            "--resolution",
            "96x54",
            "--threads",
            "1",
            BUILTIN_SCENE_SOURCE,
            "circle_shift.v1",
        ])
        .env_remove("PYTHONHOME")
        .env_remove("PYTHONPATH")
        .env("PATH", "")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("launch Metal-enabled Studio front door");
    let mut stdout = std::io::BufReader::new(child.stdout.take().expect("Studio stdout"));
    let mut ready = String::new();
    stdout
        .read_line(&mut ready)
        .expect("read Studio readiness record");
    assert!(ready.contains("\"kind\":\"studio_ready\""), "{ready}");

    let url = json_string_field(&ready, "url").expect("readiness URL");
    let location = url.strip_prefix("http://").expect("loopback HTTP URL");
    let (authority, target) = location.split_once('/').expect("URL authority and target");
    let query = target.split_once('?').expect("capability query").1;
    let mut stream = std::net::TcpStream::connect(authority).expect("connect to Studio stream");
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .expect("stream timeout");
    write!(
        stream,
        "GET /stream?{query} HTTP/1.1\r\nHost: {authority}\r\nConnection: close\r\n\r\n"
    )
    .expect("write stream request");
    let mut response = Vec::new();
    let mut chunk = [0_u8; 4096];
    while !response
        .windows(8)
        .any(|window| window == b"\x89PNG\r\n\x1a\n")
    {
        let read = stream
            .read(&mut chunk)
            .expect("read Studio fallback stream");
        assert_ne!(read, 0, "Studio stream closed before its fallback PNG");
        response.extend_from_slice(&chunk[..read]);
    }
    assert!(response.starts_with(b"HTTP/1.1 200 OK\r\n"));

    let _ = stream.shutdown(std::net::Shutdown::Both);
    drop(stream);
    drop(stdout);
    drop(child.stdin.take());
    let status = child.wait().expect("wait for graceful Studio shutdown");
    let mut stderr = String::new();
    child
        .stderr
        .take()
        .expect("Studio stderr")
        .read_to_string(&mut stderr)
        .expect("read Studio stderr");
    assert_eq!(status.code(), Some(0), "{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    Ok(())
}

#[cfg(feature = "metal")]
#[test]
fn certified_studio_metal_refuses_before_launching_a_worker() {
    let root = output_root("studio-certified-metal-refusal");
    let config = root.join("fmn.yml");
    std::fs::write(
        &config,
        b"determinism:\n  mode: certified\nrender:\n  engine: metal\n",
    )
    .expect("write certified Metal config");
    let output = run_clean(&[
        "studio",
        "--robot",
        "--no-browser",
        "--config_file",
        config.to_str().expect("config path is UTF-8"),
        BUILTIN_SCENE_SOURCE,
        "circle_shift.v1",
    ]);
    let stdout = String::from_utf8(output.stdout).expect("robot output is UTF-8");

    assert_eq!(output.status.code(), Some(3), "{stdout}");
    assert!(output.stderr.is_empty());
    assert_eq!(stdout.lines().count(), 1, "{stdout}");
    assert!(stdout.contains("\"exit_name\":\"config\""), "{stdout}");
    assert!(stdout.contains("requires render.engine=cpu"), "{stdout}");
    assert!(!stdout.contains("studio_ready"), "{stdout}");
}

#[test]
fn certified_offline_metal_refuses_before_output_side_effects() {
    let root = output_root("offline-certified-metal-refusal");
    let destination = root.join("must-not-exist");
    let output = run_clean(&[
        "--robot",
        "--reproducible",
        "--engine",
        "metal",
        "--format",
        "png_sequence",
        "--video_dir",
        destination.to_str().expect("output path is UTF-8"),
        BUILTIN_SCENE_SOURCE,
        "circle_shift.v1",
    ]);
    let stdout = String::from_utf8(output.stdout).expect("robot output is UTF-8");

    assert_eq!(output.status.code(), Some(3), "{stdout}");
    assert!(output.stderr.is_empty());
    assert_eq!(stdout.lines().count(), 1, "{stdout}");
    assert!(stdout.contains("\"exit_name\":\"config\""), "{stdout}");
    assert!(stdout.contains("requires render.engine=cpu"), "{stdout}");
    assert!(
        !destination.exists(),
        "certified refusal created output state"
    );
}

#[cfg(not(all(feature = "metal", target_os = "macos")))]
#[test]
fn offline_metal_unavailability_is_explicit_and_precedes_output_publication() {
    let root = output_root("offline-metal-unavailable");
    let destination = root.join("must-not-exist");
    let output = run_clean(&[
        "--robot",
        "--engine",
        "metal",
        "--format",
        "png_sequence",
        "--video_dir",
        destination.to_str().expect("output path is UTF-8"),
        BUILTIN_SCENE_SOURCE,
        "circle_shift.v1",
    ]);
    let stdout = String::from_utf8(output.stdout).expect("robot output is UTF-8");

    assert_eq!(output.status.code(), Some(4), "{stdout}");
    assert!(output.stderr.is_empty());
    assert_eq!(stdout.lines().count(), 1, "{stdout}");
    assert!(stdout.contains("\"exit_name\":\"capability\""), "{stdout}");
    assert!(stdout.contains("Metal annex"), "{stdout}");
    assert!(
        !destination.exists(),
        "unavailable annex created output state"
    );
}

#[cfg(all(feature = "metal", target_os = "macos"))]
#[test]
fn offline_metal_renders_ordered_native_outputs_with_actual_backend_evidence() {
    let sequence_root = output_root("offline-metal-sequence");
    let sequence = run_clean(&[
        "--robot",
        "--engine",
        "metal",
        "--format",
        "png_sequence",
        "--resolution",
        "96x54",
        "--fps",
        "8",
        "--threads",
        "1",
        "--video_dir",
        sequence_root.to_str().expect("output path is UTF-8"),
        BUILTIN_SCENE_SOURCE,
        "circle_shift.v1",
    ]);
    let stdout = String::from_utf8(sequence.stdout).expect("robot output is UTF-8");
    let record = stdout.lines().next().expect("one render record");

    assert_eq!(sequence.status.code(), Some(0), "{stdout}");
    assert!(sequence.stderr.is_empty());
    assert_eq!(stdout.lines().count(), 1, "{stdout}");
    assert!(record.contains("\"engine\":\"metal"), "{record}");
    assert!(record.contains("\"route\":\"metal-annex\""), "{record}");
    assert_eq!(json_u128_field(record, "frames"), Some(3));
    assert_eq!(json_u128_field(record, "readback_bytes"), Some(62_208));
    assert!(json_u128_field(record, "upload_bytes").is_some_and(|value| value > 0));
    assert!(json_u128_field(record, "elapsed_ns").is_some_and(|value| value > 0));

    let generation = sequence_root.join("circle_shift.v1");
    assert!(generation.join("FMN_COMPLETE").is_file());
    for index in 0..3 {
        let frame = generation.join(format!("frame_{index:06}.png"));
        let bytes = std::fs::read(frame).expect("read ordered Metal PNG");
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
    }
    let manifest = ProvenanceManifest::from_bytes(
        &std::fs::read(
            sequence_root
                .join("circle_shift.v1.manifest")
                .join("manifest.fmnp"),
        )
        .expect("read Metal FMNP"),
    )
    .expect("verify Metal FMNP");
    assert_eq!(manifest.mode, ManifestMode::Standard);
    assert!(manifest.identity.engine.starts_with("metal"));
    assert!(manifest.items.iter().any(|item| {
        item.item_id == 7 && item.detail == "semantic renderer and execution backend"
    }));

    let still_root = output_root("offline-metal-still");
    let still = run_clean(&[
        "--robot",
        "--engine",
        "metal",
        "--format",
        "png",
        "--resolution",
        "96x54",
        "--fps",
        "8",
        "--threads",
        "1",
        "--video_dir",
        still_root.to_str().expect("output path is UTF-8"),
        BUILTIN_SCENE_SOURCE,
        "circle_shift.v1",
    ]);
    let still_stdout = String::from_utf8(still.stdout).expect("robot output is UTF-8");
    let still_record = still_stdout.lines().next().expect("still render record");
    assert_eq!(still.status.code(), Some(0), "{still_stdout}");
    assert!(still.stderr.is_empty());
    assert_eq!(json_u128_field(still_record, "frames"), Some(1));
    assert_eq!(
        json_u128_field(still_record, "readback_bytes"),
        Some(20_736)
    );
    assert!(still_root.join("circle_shift.png").is_file());
}

#[cfg(unix)]
#[test]
fn studio_reloads_an_edited_compiled_scene_and_reexecutes_committed_history() {
    let root = output_root("studio-restart-replay");
    let source = root.join("edited.fmtl");
    write_compiled_wait_bundle(&source, 0.5);
    let mut child = Command::new(env!("CARGO_BIN_EXE_fmn"))
        .args([
            "studio",
            "--robot",
            "--no-browser",
            "--checkpoint-frames",
            "120",
            "--resolution",
            "96x54",
            "--threads",
            "1",
            source.to_str().expect("compiled source path is UTF-8"),
            "EditedWait",
        ])
        .env_remove("PYTHONHOME")
        .env_remove("PYTHONPATH")
        .env("PATH", "")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("launch compiled-scene Studio front door");
    let mut stdout = std::io::BufReader::new(child.stdout.take().expect("Studio stdout"));
    let mut ready = String::new();
    stdout
        .read_line(&mut ready)
        .expect("read Studio readiness record");
    let url = json_string_field(&ready, "url").expect("readiness URL");
    let location = url.strip_prefix("http://").expect("loopback HTTP URL");
    let (authority, target) = location.split_once('/').expect("URL authority and target");
    let query = target.split_once('?').expect("capability query").1;

    for frame in [1, 2] {
        let body = format!("frame={frame}&commit=true");
        let response = studio_post(authority, &format!("/api/scrub?{query}"), body.as_bytes());
        assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
        assert!(
            response.contains(&format!("\"frame_index\":{frame}")),
            "{response}"
        );
    }

    // With unchanged inputs, restart restores the first command's checkpoint
    // and replays the remaining journal entry before returning to frame two.
    let unchanged = studio_post(authority, &format!("/api/restart?{query}"), b"");
    assert!(unchanged.starts_with("HTTP/1.1 200 OK"), "{unchanged}");
    assert!(unchanged.contains("\"worker_generation\":2"), "{unchanged}");
    assert!(unchanged.contains("\"reused_entries\":2"), "{unchanged}");
    assert!(unchanged.contains("\"replayed_entries\":1"), "{unchanged}");
    assert!(
        unchanged.contains("\"reexecuted_entries\":0"),
        "{unchanged}"
    );
    assert!(unchanged.contains("\"frame_index\":2"), "{unchanged}");
    let unchanged_view = studio_get(authority, &format!("/api/inspect?{query}"));
    assert!(
        unchanged_view.starts_with("HTTP/1.1 200 OK"),
        "{unchanged_view}"
    );
    assert!(
        unchanged_view.contains("\"view\":{\"frame_index\":2,\"frame_count\":4,\"fps\":8"),
        "{unchanged_view}"
    );

    // The replacement artifact has two additional frames. A successful
    // frame-five scrub after restart therefore proves the new worker read the
    // edited bytes rather than merely replaying a captured old response.
    write_compiled_wait_bundle(&source, 0.75);
    let restart = studio_post(authority, &format!("/api/restart?{query}"), b"");
    assert!(restart.starts_with("HTTP/1.1 200 OK"), "{restart}");
    assert!(restart.contains("\"worker_generation\":3"), "{restart}");
    assert!(restart.contains("\"reused_entries\":0"), "{restart}");
    assert!(restart.contains("\"reexecuted_entries\":2"), "{restart}");
    assert!(restart.contains("\"frame_index\":2"), "{restart}");
    let edited_view = studio_get(authority, &format!("/api/inspect?{query}"));
    assert!(edited_view.starts_with("HTTP/1.1 200 OK"), "{edited_view}");
    assert!(
        edited_view.contains("\"view\":{\"frame_index\":2,\"frame_count\":6,\"fps\":8"),
        "{edited_view}"
    );

    let expanded = studio_post(
        authority,
        &format!("/api/scrub?{query}"),
        b"frame=5&commit=false",
    );
    assert!(expanded.starts_with("HTTP/1.1 200 OK"), "{expanded}");
    assert!(expanded.contains("\"frame_index\":5"), "{expanded}");

    drop(stdout);
    drop(child.stdin.take());
    let status = child.wait().expect("wait for graceful Studio shutdown");
    let mut stderr = String::new();
    child
        .stderr
        .take()
        .expect("Studio stderr")
        .read_to_string(&mut stderr)
        .expect("read Studio stderr");
    assert_eq!(status.code(), Some(0), "{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn robot_mode_refuses_terminal_escape_output() {
    let output = run_clean(&[
        "studio",
        "--robot",
        "--tui",
        "--no-browser",
        BUILTIN_SCENE_SOURCE,
        "circle_shift.v1",
    ]);
    let stdout = String::from_utf8(output.stdout).expect("robot output is UTF-8");

    assert_eq!(output.status.code(), Some(2), "{stdout}");
    assert!(output.stderr.is_empty());
    assert_eq!(stdout.lines().count(), 1, "{stdout}");
    assert!(stdout.contains("\"exit_name\":\"usage\""), "{stdout}");
    assert!(
        stdout.contains("cannot be combined with --robot"),
        "{stdout}"
    );
    assert!(!stdout.as_bytes().contains(&0x1b));
}

#[cfg(unix)]
#[test]
fn studio_tui_streams_real_worker_frames_after_scrub() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_fmn"))
        .args([
            "studio",
            "--tui",
            "--no-browser",
            "--resolution",
            "96x54",
            "--fps",
            "8",
            "--threads",
            "1",
            BUILTIN_SCENE_SOURCE,
            "circle_shift.v1",
        ])
        .env_remove("PYTHONHOME")
        .env_remove("PYTHONPATH")
        .env_remove("TERM_PROGRAM")
        .env_remove("KITTY_WINDOW_ID")
        .env("TERM", "xterm-kitty")
        .env("PATH", "")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("launch terminal Studio front door");
    let mut stdout = std::io::BufReader::new(child.stdout.take().expect("Studio stdout"));
    let mut ready = String::new();
    stdout
        .read_line(&mut ready)
        .expect("read human Studio readiness record");
    assert!(
        ready.starts_with("Studio ready for circle_shift.v1: http://"),
        "{ready}"
    );
    let location = ready
        .split_once("http://")
        .expect("readiness URL scheme")
        .1
        .split_once(" (")
        .expect("readiness URL suffix")
        .0;
    let (authority, target) = location.split_once('/').expect("URL authority and target");
    let query = target.split_once('?').expect("capability query").1;

    let terminal_reader = std::thread::spawn(move || {
        let mut terminal = Vec::new();
        stdout
            .read_to_end(&mut terminal)
            .expect("read terminal Studio frames");
        terminal
    });

    let body = b"frame=1&commit=false";
    let mut scrub = std::net::TcpStream::connect(authority).expect("connect to Studio scrub API");
    scrub
        .set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .expect("scrub timeout");
    write!(
        scrub,
        "POST /api/scrub?{query} HTTP/1.1\r\nHost: {authority}\r\nOrigin: http://{authority}\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .expect("write scrub request");
    scrub.write_all(body).expect("write scrub body");
    let mut scrub_response = Vec::new();
    scrub
        .read_to_end(&mut scrub_response)
        .expect("read scrub response");
    let _ = scrub.shutdown(std::net::Shutdown::Both);
    assert!(
        scrub_response.starts_with(b"HTTP/1.1 200 OK"),
        "{}",
        String::from_utf8_lossy(&scrub_response)
    );
    assert!(
        scrub_response
            .windows(b"\"frame_index\":1".len())
            .any(|window| window == b"\"frame_index\":1")
    );

    drop(child.stdin.take());
    let status = child
        .wait()
        .expect("wait for graceful terminal Studio shutdown");
    let terminal = terminal_reader.join().expect("terminal reader thread");
    let mut stderr = String::new();
    child
        .stderr
        .take()
        .expect("Studio stderr")
        .read_to_string(&mut stderr)
        .expect("read Studio stderr");
    assert_eq!(status.code(), Some(0), "{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(
        terminal
            .windows(b"\x1b_Ga=T,f=100,t=d,q=2,".len())
            .filter(|window| *window == b"\x1b_Ga=T,f=100,t=d,q=2,")
            .count()
            >= 2,
        "expected the initial and scrubbed worker frames, got {} terminal bytes",
        terminal.len()
    );
}

#[test]
fn built_in_corpus_renders_through_the_real_png_and_y4m_sinks() {
    for format in ["png_sequence", "y4m"] {
        let root = output_root(format);
        let root_text = root.to_str().expect("output path is UTF-8");
        let output = run_clean(&[
            "--robot",
            "--write_all",
            "--format",
            format,
            "--resolution",
            "96x54",
            "--fps",
            "8",
            "--threads",
            "1",
            "--video_dir",
            root_text,
            BUILTIN_SCENE_SOURCE,
        ]);
        let stdout = String::from_utf8(output.stdout).expect("robot output is UTF-8");

        assert_eq!(output.status.code(), Some(0), "{format}: {stdout}");
        assert!(output.stderr.is_empty(), "{format}");
        assert_eq!(stdout.lines().count(), 25, "{format}");
        assert!(
            stdout.lines().all(|line| {
                line.contains("\"kind\":\"render\"")
                    && line.contains(&format!("\"format\":\"{format}\""))
                    && line.contains("\"frames\":3")
                    && line.contains("\"engine\":\"fast-cpu")
            }),
            "{format}: {stdout}"
        );

        let entries = std::fs::read_dir(&root)
            .expect("list published corpus root")
            .collect::<Result<Vec<_>, _>>()
            .expect("read published corpus entries");
        assert_eq!(entries.len(), 50, "{format}");
        let manifest_generation = if format == "png_sequence" {
            root.join("circle_shift.v1.manifest")
        } else {
            root.join("circle_shift.y4m.manifest")
        };
        assert!(manifest_generation.join("FMN_COMPLETE").is_file());
        let manifest = ProvenanceManifest::from_bytes(
            &std::fs::read(manifest_generation.join("manifest.fmnp"))
                .expect("read standard render FMNP"),
        )
        .expect("verify standard render FMNP");
        assert_eq!(manifest.mode, ManifestMode::Standard);
        assert!((1..=10).all(|id| manifest.items.iter().any(|item| item.item_id == id)));
        assert_eq!(manifest.outputs.len(), 1);
        assert!(!manifest.outputs[0].certified);
        if format == "png_sequence" {
            let sequence = root.join("circle_shift.v1");
            assert!(sequence.join("FMN_COMPLETE").is_file());
            let frame =
                std::fs::read(sequence.join("frame_000000.png")).expect("read first canonical PNG");
            assert_eq!(&frame[..8], b"\x89PNG\r\n\x1a\n");
        } else {
            let artifact =
                std::fs::read(root.join("circle_shift.y4m")).expect("read native y4m artifact");
            assert!(artifact.starts_with(b"YUV4MPEG2 W96 H54 F8:1"));
            assert!(artifact.windows(6).any(|window| window == b"FRAME\n"));
        }
    }
}

#[test]
fn prerun_and_subdivide_drive_real_per_segment_outputs() {
    let root = output_root("prerun-subdivide-builtin");
    let output = run_clean(&[
        "--robot",
        "--prerun",
        "--subdivide",
        "--format",
        "y4m",
        "--resolution",
        "96x54",
        "--fps",
        "8",
        "--threads",
        "1",
        "--video_dir",
        root.to_str().expect("output path is UTF-8"),
        BUILTIN_SCENE_SOURCE,
        "circle_shift.v1",
    ]);
    let stdout = String::from_utf8(output.stdout).expect("robot output is UTF-8");
    let records = stdout.lines().collect::<Vec<_>>();

    assert_eq!(output.status.code(), Some(0), "{stdout}");
    assert!(output.stderr.is_empty());
    assert_eq!(records.len(), 3, "{stdout}");
    assert!(records[0].contains("\"kind\":\"prerun\""));
    assert!(records[0].contains("\"frames\":3"));
    assert!(records[0].contains("\"segments\":2"));
    for (index, frames) in [2, 1].into_iter().enumerate() {
        let record = records[index + 1];
        assert!(record.contains("\"kind\":\"render\""), "{record}");
        assert!(
            record.contains(&format!("\"subdivision\":{index}")),
            "{record}"
        );
        assert!(record.contains(&format!("\"frames\":{frames}")), "{record}");
        let artifact = root.join(format!("circle_shift.v1/{index:05}.y4m"));
        let bytes = std::fs::read(&artifact).expect("read subdivided y4m artifact");
        assert!(bytes.starts_with(b"YUV4MPEG2 W96 H54 F8:1"));
        assert_eq!(
            bytes
                .windows(6)
                .filter(|window| *window == b"FRAME\n")
                .count(),
            frames
        );
        assert!(
            artifact
                .with_file_name(format!("{index:05}.y4m.manifest"))
                .join("FMN_COMPLETE")
                .is_file()
        );
    }

    let compiled_root = output_root("prerun-subdivide-compiled");
    let source = compiled_root.join("two_waits.fmtl");
    let mut timeline = Timeline::new(8).expect("valid bundle fps");
    timeline.wait(0.25).expect("first wait");
    timeline.wait(0.125).expect("second wait");
    let bytes = export_timeline_bundle(timeline, &mut Stage::new(), &RngRoot::from_seed(0))
        .expect("compile two-segment artifact");
    std::fs::write(&source, bytes).expect("write compiled artifact");
    let output = run_clean(&[
        "--robot",
        "--prerun",
        "--subdivide",
        "--format",
        "y4m",
        "--resolution",
        "96x54",
        "--threads",
        "1",
        "--video_dir",
        compiled_root.to_str().expect("output path is UTF-8"),
        source.to_str().expect("source path is UTF-8"),
    ]);
    let stdout = String::from_utf8(output.stdout).expect("robot output is UTF-8");
    let records = stdout.lines().collect::<Vec<_>>();
    assert_eq!(output.status.code(), Some(0), "{stdout}");
    assert!(output.stderr.is_empty());
    assert_eq!(records.len(), 3, "{stdout}");
    assert!(records[0].contains("\"frames\":3"));
    assert!(records[0].contains("\"segments\":2"));
    assert!(records[1].contains("\"frames\":2"));
    assert!(records[2].contains("\"frames\":1"));
    assert!(compiled_root.join("two_waits/00000.y4m").is_file());
    assert!(compiled_root.join("two_waits/00001.y4m").is_file());
}

#[test]
fn subdivide_preflights_every_generation_before_rendering() {
    let root = output_root("subdivide-preflight");
    std::fs::create_dir_all(root.join("circle_shift.v1/00001.y4m.manifest"))
        .expect("occupy the second subdivision sidecar");
    let output = run_clean(&[
        "--robot",
        "--subdivide",
        "--format",
        "y4m",
        "--resolution",
        "96x54",
        "--fps",
        "8",
        "--threads",
        "1",
        "--video_dir",
        root.to_str().expect("output path is UTF-8"),
        BUILTIN_SCENE_SOURCE,
        "circle_shift.v1",
    ]);
    let stdout = String::from_utf8(output.stdout).expect("robot output is UTF-8");

    assert_eq!(output.status.code(), Some(6), "{stdout}");
    assert!(output.stderr.is_empty());
    assert!(stdout.contains("sidecars are no-clobber generations"));
    assert!(!root.join("circle_shift.v1/00000.y4m").exists());
}

#[cfg(feature = "batch")]
#[test]
fn batch_renders_multiple_scenes_and_reports_in_request_order() {
    let root = output_root("batch-positive");
    let output = run_clean(&[
        "batch",
        "--robot",
        "--format",
        "png_sequence",
        "--resolution",
        "96x54",
        "--fps",
        "8",
        "--threads",
        "1",
        "--max-scenes",
        "2",
        "--video_dir",
        root.to_str().expect("output path is UTF-8"),
        BUILTIN_SCENE_SOURCE,
        "circle_shift.v1",
        "rectangle_shift.v1",
    ]);
    let stdout = String::from_utf8(output.stdout).expect("robot output is UTF-8");
    let lines: Vec<&str> = stdout.lines().collect();

    assert_eq!(output.status.code(), Some(0), "{stdout}");
    assert!(output.stderr.is_empty());
    assert_eq!(lines.len(), 3, "{stdout}");
    assert!(lines[0].contains("\"kind\":\"render\""));
    assert!(lines[0].contains("\"scene\":\"circle_shift.v1\""));
    assert!(lines[1].contains("\"kind\":\"render\""));
    assert!(lines[1].contains("\"scene\":\"rectangle_shift.v1\""));
    assert!(lines[2].contains("\"kind\":\"batch\""));
    assert!(lines[2].contains("\"status\":\"ok\""));
    assert!(lines[2].contains("\"succeeded\":2"));
    assert!(root.join("circle_shift.v1/FMN_COMPLETE").is_file());
    assert!(root.join("rectangle_shift.v1/FMN_COMPLETE").is_file());
    assert!(root.join("circle_shift.v1.manifest/FMN_COMPLETE").is_file());
    assert!(
        root.join("rectangle_shift.v1.manifest/FMN_COMPLETE")
            .is_file()
    );
}

#[cfg(feature = "batch")]
#[test]
fn batch_subdivision_publishes_distinct_manifest_generations() {
    let root = output_root("batch-subdivide");
    let artifacts = root.join("artifacts");
    let manifests = root.join("manifests");
    std::fs::create_dir(&artifacts).expect("create artifact root");
    std::fs::create_dir(&manifests).expect("create manifest root");
    let output = run_clean(&[
        "batch",
        "--robot",
        "--prerun",
        "--subdivide",
        "--format",
        "y4m",
        "--resolution",
        "96x54",
        "--fps",
        "8",
        "--threads",
        "1",
        "--max-scenes",
        "1",
        "--manifest-dir",
        manifests.to_str().expect("manifest path is UTF-8"),
        "--video_dir",
        artifacts.to_str().expect("artifact path is UTF-8"),
        BUILTIN_SCENE_SOURCE,
        "circle_shift.v1",
    ]);
    let stdout = String::from_utf8(output.stdout).expect("robot output is UTF-8");
    let records = stdout.lines().collect::<Vec<_>>();

    assert_eq!(output.status.code(), Some(0), "{stdout}");
    assert!(output.stderr.is_empty());
    assert_eq!(records.len(), 4, "{stdout}");
    assert!(records[0].contains("\"kind\":\"prerun\""));
    assert!(records[1].contains("\"subdivision\":0"));
    assert!(records[2].contains("\"subdivision\":1"));
    assert!(records[3].contains("\"kind\":\"batch\""));
    for index in 0..2 {
        assert!(
            manifests
                .join(format!("circle_shift.v1/{index:05}/FMN_COMPLETE"))
                .is_file()
        );
        assert!(
            artifacts
                .join(format!("circle_shift.v1/{index:05}.y4m"))
                .is_file()
        );
    }
}

#[cfg(feature = "batch")]
#[test]
fn batch_budget_cancels_before_publication() {
    let budget_root = output_root("batch-budget");
    let budget = run_clean(&[
        "batch",
        "--robot",
        "--budget-ms",
        "0",
        "--max-scenes",
        "2",
        "--format",
        "png_sequence",
        "--resolution",
        "96x54",
        "--video_dir",
        budget_root.to_str().expect("output path is UTF-8"),
        BUILTIN_SCENE_SOURCE,
        "circle_shift.v1",
        "rectangle_shift.v1",
    ]);
    let budget_stdout = String::from_utf8(budget.stdout).expect("robot output is UTF-8");

    assert_eq!(budget.status.code(), Some(8), "{budget_stdout}");
    assert!(budget.stderr.is_empty());
    assert_eq!(budget_stdout.lines().count(), 3, "{budget_stdout}");
    assert_eq!(budget_stdout.matches("\"status\":\"cancelled\"").count(), 2);
    assert!(budget_stdout.contains("\"cancelled\":2"));
    assert_eq!(
        std::fs::read_dir(&budget_root)
            .expect("list unpublished budget root")
            .count(),
        0
    );
}

#[cfg(feature = "batch")]
#[test]
fn batch_publishes_complete_fmnp_manifests_and_preflights_no_clobber() {
    fn render_with_manifests(
        label: &str,
        threads: &str,
        format: &str,
    ) -> (std::path::PathBuf, std::path::PathBuf, String) {
        let artifact_root = output_root(&format!("{label}-artifacts"));
        let manifest_root = output_root(&format!("{label}-manifests"));
        let output = run_clean(&[
            "batch",
            "--robot",
            "--reproducible",
            "--manifest-dir",
            manifest_root.to_str().expect("manifest path is UTF-8"),
            "--format",
            format,
            "--resolution",
            "96x54",
            "--fps",
            "8",
            "--threads",
            threads,
            "--max-scenes",
            "2",
            "--video_dir",
            artifact_root.to_str().expect("output path is UTF-8"),
            BUILTIN_SCENE_SOURCE,
            "circle_shift.v1",
            "rectangle_shift.v1",
        ]);
        let stdout = String::from_utf8(output.stdout).expect("robot output is UTF-8");
        assert_eq!(output.status.code(), Some(0), "{stdout}");
        assert!(output.stderr.is_empty());
        assert_eq!(stdout.lines().count(), 3, "{stdout}");
        (artifact_root, manifest_root, stdout)
    }

    let (artifact_root, manifest_root, stdout) =
        render_with_manifests("fmnp-one", "1", "png_sequence");
    let mut first_closure = None;
    let mut first_outputs = Vec::new();
    for (index, scene) in ["circle_shift.v1", "rectangle_shift.v1"]
        .into_iter()
        .enumerate()
    {
        let record = stdout.lines().nth(index).expect("ordered render record");
        assert!(record.contains(&format!("\"scene\":\"{scene}\"")));
        assert!(record.contains("\"manifest\":{"));
        let generation = manifest_root.join(scene);
        assert!(generation.join("FMN_COMPLETE").is_file());
        let bytes = std::fs::read(generation.join("manifest.fmnp")).expect("read FMNP");
        let manifest = ProvenanceManifest::from_bytes(&bytes).expect("verify FMNP");
        assert_eq!(manifest.mode, ManifestMode::Certified);
        assert!((1..=10).all(|id| manifest.items.iter().any(|item| item.item_id == id)));
        assert_eq!(manifest.outputs.len(), 1);
        assert!(manifest.outputs[0].certified);
        assert_eq!(
            manifest.outputs[0].digest.to_hex(),
            json_string_field(record, "artifact_digest").expect("artifact digest robot field")
        );
        assert_eq!(
            manifest.closure_digest.to_hex(),
            json_string_field(record, "closure_digest").expect("closure digest robot field")
        );
        let text = std::fs::read_to_string(generation.join("manifest.txt"))
            .expect("read human FMNP rendering");
        for id in 1..=10 {
            assert!(text.contains(&format!("id = {id}")));
        }
        if index == 0 {
            first_closure = Some(manifest.closure_digest);
        }
        first_outputs.push(manifest.outputs[0].digest);
        assert!(artifact_root.join(scene).join("FMN_COMPLETE").is_file());
    }

    let (_four_artifacts, four_manifests, four_stdout) =
        render_with_manifests("fmnp-four", "4", "png_sequence");
    let four_manifest = ProvenanceManifest::from_bytes(
        &std::fs::read(four_manifests.join("circle_shift.v1/manifest.fmnp"))
            .expect("read four-thread FMNP"),
    )
    .expect("verify four-thread FMNP");
    assert_eq!(first_closure, Some(four_manifest.closure_digest));
    assert_eq!(first_outputs[0], four_manifest.outputs[0].digest);
    // The planner honors --threads as a ceiling, not a demand: render
    // threads come out as min(limit, physical cores), so a 2-core CI
    // runner legitimately reports 2 here. Assert the ceiling held; the
    // closure/output digest equalities above are the thread-invariance
    // proof.
    let render_threads: usize = four_stdout
        .split("\"render_threads\":")
        .nth(1)
        .and_then(|rest| {
            rest.split(|c: char| !c.is_ascii_digit())
                .next()?
                .parse()
                .ok()
        })
        .expect("four-thread run reports render_threads");
    assert!(
        (1..=4).contains(&render_threads),
        "--threads 4 ceiling violated: {render_threads}"
    );

    let (_still_artifacts, still_manifests, _still_stdout) =
        render_with_manifests("fmnp-still", "1", "png");
    let still_manifest = ProvenanceManifest::from_bytes(
        &std::fs::read(still_manifests.join("circle_shift.v1/manifest.fmnp"))
            .expect("read final-state PNG FMNP"),
    )
    .expect("verify final-state PNG FMNP");
    assert_ne!(first_closure, Some(still_manifest.closure_digest));
    assert_eq!(still_manifest.outputs[0].kind, "canonical_png");

    let blocked_output_root = output_root("fmnp-preflight-blocked");
    let blocked = run_clean(&[
        "batch",
        "--robot",
        "--manifest-dir",
        manifest_root.to_str().expect("manifest path is UTF-8"),
        "--format",
        "png_sequence",
        "--video_dir",
        blocked_output_root
            .to_str()
            .expect("blocked output path is UTF-8"),
        BUILTIN_SCENE_SOURCE,
        "circle_shift.v1",
    ]);
    let blocked_stdout = String::from_utf8(blocked.stdout).expect("robot output is UTF-8");

    assert_eq!(blocked.status.code(), Some(6), "{blocked_stdout}");
    assert!(blocked.stderr.is_empty());
    assert!(blocked_stdout.contains("already exists"));
    assert_eq!(
        std::fs::read_dir(blocked_output_root)
            .expect("list output root after preflight refusal")
            .count(),
        0
    );
}

#[cfg(feature = "batch")]
#[test]
fn batch_fail_fast_cancels_jobs_after_the_first_publication_failure() {
    let root = output_root("batch-fail-fast");
    std::fs::create_dir(root.join("circle_shift.v1"))
        .expect("reserve the first scene destination to force no-clobber failure");
    let output = run_clean(&[
        "batch",
        "--robot",
        "--fail-fast",
        "--max-scenes",
        "1",
        "--format",
        "png_sequence",
        "--resolution",
        "96x54",
        "--threads",
        "1",
        "--video_dir",
        root.to_str().expect("output path is UTF-8"),
        BUILTIN_SCENE_SOURCE,
        "circle_shift.v1",
        "rectangle_shift.v1",
    ]);
    let stdout = String::from_utf8(output.stdout).expect("robot output is UTF-8");
    let lines: Vec<&str> = stdout.lines().collect();

    assert_eq!(output.status.code(), Some(6), "{stdout}");
    assert!(output.stderr.is_empty());
    assert_eq!(lines.len(), 3, "{stdout}");
    assert!(lines[0].contains("\"scene\":\"circle_shift.v1\""));
    assert!(lines[0].contains("\"status\":\"failed\""));
    assert!(lines[1].contains("\"scene\":\"rectangle_shift.v1\""));
    assert!(lines[1].contains("\"status\":\"cancelled\""));
    assert!(lines[1].contains("\"reason\":\"fail_fast\""));
    assert!(lines[2].contains("\"failed\":1"));
    assert!(lines[2].contains("\"cancelled\":1"));
    assert!(!root.join("rectangle_shift.v1").exists());
}

#[test]
fn native_png_publishes_the_final_frame_and_gif_streams_the_schedule() {
    let root = output_root("native-png-gif");
    let sequence_root = root.join("sequence");
    let still_root = root.join("still");
    let gif_root = root.join("gif");
    for directory in [&sequence_root, &still_root, &gif_root] {
        std::fs::create_dir(directory).expect("create native-format output root");
    }

    let render = |format: &str, output: &std::path::Path| {
        run_clean(&[
            "--robot",
            "--format",
            format,
            "--resolution",
            "96x54",
            "--fps",
            "8",
            "--threads",
            "1",
            "--video_dir",
            output.to_str().expect("output path is UTF-8"),
            BUILTIN_SCENE_SOURCE,
            "circle_shift.v1",
        ])
    };

    let sequence = render("png_sequence", &sequence_root);
    assert_eq!(sequence.status.code(), Some(0));
    assert!(sequence.stderr.is_empty());

    let still = render("png", &still_root);
    let still_stdout = String::from_utf8(still.stdout).expect("robot output is UTF-8");
    assert_eq!(still.status.code(), Some(0), "{still_stdout}");
    assert!(still.stderr.is_empty());
    assert!(still_stdout.contains("\"format\":\"png\""));
    assert!(still_stdout.contains("\"frames\":1"));
    let still_bytes =
        std::fs::read(still_root.join("circle_shift.png")).expect("read final-state PNG");
    let final_sequence_frame =
        std::fs::read(sequence_root.join("circle_shift.v1/frame_000002.png"))
            .expect("read final PNG-sequence frame");
    assert_eq!(still_bytes, final_sequence_frame);

    let gif = render("gif", &gif_root);
    let gif_stdout = String::from_utf8(gif.stdout).expect("robot output is UTF-8");
    assert_eq!(gif.status.code(), Some(0), "{gif_stdout}");
    assert!(gif.stderr.is_empty());
    assert!(gif_stdout.contains("\"format\":\"gif\""));
    assert!(gif_stdout.contains("\"frames\":3"));
    let gif_bytes = std::fs::read(gif_root.join("circle_shift.gif")).expect("read native GIF");
    assert!(gif_bytes.starts_with(b"GIF89a"));
    assert_eq!(gif_bytes.last(), Some(&b';'));
}

#[test]
fn certified_gif_is_refused_before_publication() {
    let root = output_root("certified-gif-refusal");
    let output = run_clean(&[
        "--robot",
        "--reproducible",
        "--format",
        "gif",
        "--video_dir",
        root.to_str().expect("output path is UTF-8"),
        BUILTIN_SCENE_SOURCE,
        "circle_shift.v1",
    ]);
    let stdout = String::from_utf8(output.stdout).expect("robot output is UTF-8");

    assert_eq!(output.status.code(), Some(4), "{stdout}");
    assert!(output.stderr.is_empty());
    assert!(stdout.contains("\"exit_name\":\"capability\""));
    assert!(stdout.contains("GIF is outside the certified artifact set"));
    assert_eq!(
        std::fs::read_dir(root)
            .expect("list untouched output root")
            .count(),
        0
    );
}

#[test]
fn skip_mode_publishes_one_final_frame_to_a_streaming_native_sink() {
    let root = output_root("skip-final-gif");
    let output = run_clean(&[
        "--robot",
        "--skip_animations",
        "--format",
        "gif",
        "--resolution",
        "96x54",
        "--fps",
        "8",
        "--threads",
        "1",
        "--video_dir",
        root.to_str().expect("output path is UTF-8"),
        BUILTIN_SCENE_SOURCE,
        "circle_shift.v1",
    ]);
    let stdout = String::from_utf8(output.stdout).expect("robot output is UTF-8");

    assert_eq!(output.status.code(), Some(0), "{stdout}");
    assert!(output.stderr.is_empty());
    assert!(stdout.contains("\"format\":\"gif\""));
    assert!(stdout.contains("\"frames\":1"));
    let gif = std::fs::read(root.join("circle_shift.gif")).expect("read final-state GIF");
    assert!(gif.starts_with(b"GIF89a"));
    assert_eq!(gif.last(), Some(&b';'));
}

#[test]
fn compiled_fmtl_renders_through_the_standalone_binary() {
    let root = output_root("compiled-fmtl");
    let source = compiled_wait_bundle(&root);
    let output_dir = root.join("output");
    std::fs::create_dir(&output_dir).expect("create compiled output root");
    let output = run_clean(&[
        "--robot",
        "--format",
        "png_sequence",
        "--resolution",
        "96x54",
        "--threads",
        "1",
        "--video_dir",
        output_dir.to_str().expect("output path is UTF-8"),
        source.to_str().expect("source path is UTF-8"),
        "CompiledWait",
    ]);
    let stdout = String::from_utf8(output.stdout).expect("robot output is UTF-8");

    assert_eq!(output.status.code(), Some(0), "{stdout}");
    assert!(output.stderr.is_empty());
    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.contains("\"source\":\"compiled\""));
    assert!(stdout.contains("\"source_artifact\":"));
    assert!(stdout.contains("\"scene\":\"CompiledWait\""));
    assert!(stdout.contains("\"frames\":2"));
    let sequence = output_dir.join("CompiledWait");
    assert!(sequence.join("FMN_COMPLETE").is_file());
    let pngs = std::fs::read_dir(&sequence)
        .expect("list compiled output")
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "png"))
        .count();
    assert_eq!(pngs, 2);
    let frame = std::fs::read(sequence.join("frame_000000.png")).expect("read compiled PNG");
    assert!(frame.starts_with(b"\x89PNG\r\n\x1a\n"));
}

#[test]
fn compiled_fmtl_refuses_schedule_changes_and_malformed_input_before_publication() {
    let root = output_root("compiled-refusals");
    let source = compiled_wait_bundle(&root);
    let output_dir = root.join("output");
    std::fs::create_dir(&output_dir).expect("create refusal output root");

    let fps = run_clean(&[
        "--robot",
        "--format",
        "png_sequence",
        "--fps",
        "9",
        "--video_dir",
        output_dir.to_str().expect("output path is UTF-8"),
        source.to_str().expect("source path is UTF-8"),
    ]);
    let fps_stdout = String::from_utf8(fps.stdout).expect("robot output is UTF-8");
    assert_eq!(fps.status.code(), Some(3), "{fps_stdout}");
    assert!(fps.stderr.is_empty());
    assert!(fps_stdout.contains("fixed 8 fps schedule"));
    assert_eq!(
        std::fs::read_dir(&output_dir).expect("list output").count(),
        0
    );

    let malformed = root.join("malformed.fmtl");
    std::fs::write(&malformed, b"not an FMTL document").expect("write malformed artifact");
    let bad = run_clean(&[
        "--robot",
        "--format",
        "png_sequence",
        "--video_dir",
        output_dir.to_str().expect("output path is UTF-8"),
        malformed.to_str().expect("source path is UTF-8"),
    ]);
    let bad_stdout = String::from_utf8(bad.stdout).expect("robot output is UTF-8");
    assert_eq!(bad.status.code(), Some(5), "{bad_stdout}");
    assert!(bad.stderr.is_empty());
    assert!(bad_stdout.contains("malformed FMTL/1 container"));
    assert_eq!(
        std::fs::read_dir(output_dir).expect("list output").count(),
        0
    );
}

#[test]
fn reproducible_cli_render_is_byte_identical_across_fresh_publications() {
    let mut outputs = Vec::new();
    let mut closures = Vec::new();
    let mut artifact_digests = Vec::new();
    for label in ["certified-a", "certified-b"] {
        let root = output_root(label);
        let output = run_clean(&[
            "--robot",
            "--reproducible",
            "--format",
            "png_sequence",
            "--resolution",
            "96x54",
            "--fps",
            "8",
            "--threads",
            "1",
            "--video_dir",
            root.to_str().expect("output path is UTF-8"),
            BUILTIN_SCENE_SOURCE,
            "circle_shift.v1",
        ]);
        let stdout = String::from_utf8(output.stdout).expect("robot output is UTF-8");
        assert_eq!(output.status.code(), Some(0), "{stdout}");
        assert!(output.stderr.is_empty());
        assert!(stdout.contains("\"engine\":\"certified-cpu:scalar:"));
        assert!(stdout.contains("\"manifest\":{"));
        let sequence = root.join("circle_shift.v1");
        let manifest_generation = root.join("circle_shift.v1.manifest");
        assert!(manifest_generation.join("FMN_COMPLETE").is_file());
        let manifest_path = manifest_generation.join("manifest.fmnp");
        assert_eq!(
            json_string_field(&stdout, "path"),
            Some(manifest_path.to_str().expect("manifest path is UTF-8"))
        );
        let manifest = ProvenanceManifest::from_bytes(
            &std::fs::read(&manifest_path).expect("read certified render FMNP"),
        )
        .expect("verify certified render FMNP");
        assert_eq!(manifest.mode, ManifestMode::Certified);
        assert!((1..=10).all(|id| manifest.items.iter().any(|item| item.item_id == id)));
        assert_eq!(manifest.outputs.len(), 1);
        assert!(manifest.outputs[0].certified);
        assert_eq!(
            manifest.outputs[0].digest.to_hex(),
            json_string_field(&stdout, "artifact_digest").expect("artifact digest robot field")
        );
        assert_eq!(
            manifest.closure_digest.to_hex(),
            json_string_field(&stdout, "closure_digest").expect("closure digest robot field")
        );
        let text = std::fs::read_to_string(manifest_generation.join("manifest.txt"))
            .expect("read certified human manifest");
        assert!(text.contains("mode = \"certified\""));
        closures.push(manifest.closure_digest);
        artifact_digests.push(manifest.outputs[0].digest);
        outputs.push(
            (0..3)
                .map(|frame| {
                    std::fs::read(sequence.join(format!("frame_{frame:06}.png")))
                        .expect("read certified PNG")
                })
                .collect::<Vec<_>>(),
        );
    }
    assert_eq!(outputs[0], outputs[1]);
    assert_eq!(closures[0], closures[1]);
    assert_eq!(artifact_digests[0], artifact_digests[1]);
}

#[test]
fn occupied_sidecar_is_refused_before_render_artifact_publication() {
    let root = output_root("occupied-sidecar");
    std::fs::create_dir(root.join("circle_shift.v1.manifest"))
        .expect("occupy the sidecar destination");
    let output = run_clean(&[
        "--robot",
        "--format",
        "png_sequence",
        "--resolution",
        "96x54",
        "--video_dir",
        root.to_str().expect("output path is UTF-8"),
        BUILTIN_SCENE_SOURCE,
        "circle_shift.v1",
    ]);
    let stdout = String::from_utf8(output.stdout).expect("robot output is UTF-8");

    assert_eq!(output.status.code(), Some(6), "{stdout}");
    assert!(output.stderr.is_empty());
    assert!(stdout.contains("sidecars are no-clobber generations"));
    assert!(!root.join("circle_shift.v1").exists());
}

#[test]
fn unknown_built_in_scene_fails_before_publication() {
    let root = output_root("unknown-scene");
    let output = run_clean(&[
        "--robot",
        "--format",
        "png_sequence",
        "--video_dir",
        root.to_str().expect("output path is UTF-8"),
        BUILTIN_SCENE_SOURCE,
        "NotARealScene",
    ]);
    let stdout = String::from_utf8(output.stdout).expect("robot output is UTF-8");

    assert_eq!(output.status.code(), Some(5));
    assert!(output.stderr.is_empty());
    assert!(stdout.contains("\"exit_name\":\"scene\""));
    assert!(stdout.contains("unknown built-in scene"));
    assert!(!stdout.contains(PYTHON_SOURCE_PORTAL_MESSAGE));
    assert_eq!(
        std::fs::read_dir(root)
            .expect("list untouched output root")
            .count(),
        0
    );
}

#[test]
fn video_render_names_the_missing_optional_tool_and_native_alternative() {
    let root = output_root("missing-ffmpeg");
    let output = run_clean(&[
        "--robot",
        "--format",
        "video",
        "--resolution",
        "96x54",
        "--fps",
        "8",
        "--video_dir",
        root.to_str().expect("output path is UTF-8"),
        BUILTIN_SCENE_SOURCE,
        "circle_shift.v1",
    ]);
    let stdout = String::from_utf8(output.stdout).expect("robot output is UTF-8");

    assert_eq!(output.status.code(), Some(4), "{stdout}");
    assert!(output.stderr.is_empty());
    assert!(stdout.contains("\"exit_name\":\"capability\""));
    assert!(stdout.contains("ffmpeg is unavailable"));
    assert!(stdout.contains("native outputs need no ffmpeg"));
    assert!(stdout.contains("y4m, PNG sequences, and GIF"));
    assert_eq!(
        std::fs::read_dir(root)
            .expect("list untouched output root")
            .count(),
        0
    );
}

fn run_in_dir(dir: &std::path::Path, args: &[String]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_fmn"))
        .args(args)
        .current_dir(dir)
        .env_remove("PYTHONHOME")
        .env_remove("PYTHONPATH")
        .env("PATH", "")
        .output()
        .expect("launch the standalone fmn binary by absolute path")
}

fn wav_asset_bytes(amplitude: f32) -> Vec<u8> {
    let samples: Vec<f32> = (0..12_000)
        .map(|index| amplitude * (core::f32::consts::TAU * 440.0 * index as f32 / 48_000.0).sin())
        .collect();
    fmn_codec::encode_wav(1, 48_000, fmn_codec::SampleFormat::S16, &samples)
}

#[test]
fn wav_soundtrack_refuses_no_cue_scenes_and_publishes_cue_content_deterministically() {
    // A scene without add_sound requests refuses by name, before output.
    let root = output_root("wav-no-cue-refusal");
    let output = run_clean(&[
        "--robot",
        "--format",
        "wav",
        "--video_dir",
        root.join("out").to_str().expect("output path is UTF-8"),
        BUILTIN_SCENE_SOURCE,
        "circle_shift.v1",
    ]);
    let stdout = String::from_utf8(output.stdout).expect("robot output is UTF-8");
    assert_eq!(output.status.code(), Some(5), "{stdout}");
    assert!(stdout.contains("\"exit_name\":\"scene\""), "{stdout}");
    assert!(stdout.contains("scene_has_no_sound_cues"), "{stdout}");
    assert!(!root.join("out").exists(), "refusal created output state");

    // The sound-cue scene publishes a decodable soundtrack whose bytes
    // follow the cue assets and stay bit-identical across runs.
    let home = output_root("wav-positive");
    std::fs::write(home.join("sound_cue_tone.v1.wav"), wav_asset_bytes(0.4))
        .expect("write cue asset");
    let video_dir = home.join("med").join("videos");
    std::fs::create_dir_all(&video_dir).expect("create output directory");
    let args = |dir: &std::path::Path| {
        [
            "--robot".to_owned(),
            "--format".to_owned(),
            "wav".to_owned(),
            "--fps".to_owned(),
            "8".to_owned(),
            "--threads".to_owned(),
            "1".to_owned(),
            "--video_dir".to_owned(),
            dir.to_str().expect("output path is UTF-8").to_owned(),
            BUILTIN_SCENE_SOURCE.to_owned(),
            "sound_cue.v1".to_owned(),
        ]
    };
    let first = run_in_dir(&home, &args(&video_dir));
    let stdout = String::from_utf8(first.stdout).expect("robot output is UTF-8");
    assert_eq!(first.status.code(), Some(0), "{stdout}");
    let record = stdout.lines().next().expect("one render record");
    assert!(record.contains("\"format\":\"wav\""), "{record}");
    let published = video_dir.join("sound_cue.wav");
    let bytes = std::fs::read(&published).expect("read published WAV");
    assert!(!bytes.is_empty());
    let decoded = fmn_codec::decode_wav(&bytes, &fmn_codec::WavLimits::default())
        .expect("published artifact is a decodable PCM WAV");
    assert!(
        decoded.samples.iter().any(|&s| s != 0.0),
        "cue content flows"
    );
    assert!(
        record.contains(&format!("\"bytes\":{}", bytes.len())),
        "{record}"
    );
    assert!(
        video_dir
            .join("sound_cue.wav.manifest")
            .join("manifest.fmnp")
            .is_file(),
        "WAV publishes FMNP provenance"
    );

    // A changed cue parameter (asset amplitude) changes the mix bytes.
    let home_b = output_root("wav-positive-louder");
    std::fs::write(home_b.join("sound_cue_tone.v1.wav"), wav_asset_bytes(0.8))
        .expect("write louder cue asset");
    let video_dir_b = home_b.join("med").join("videos");
    std::fs::create_dir_all(&video_dir_b).expect("create output directory");
    let louder = run_in_dir(&home_b, &args(&video_dir_b));
    assert_eq!(louder.status.code(), Some(0));
    let louder_bytes = std::fs::read(video_dir_b.join("sound_cue.wav")).expect("read louder WAV");
    assert_ne!(bytes, louder_bytes, "cue amplitude reaches the mix");

    // Repeating the same composition into a fresh generation is
    // bit-identical (sidecars are no-clobber generations).
    let video_dir_repeat = home.join("med").join("videos-repeat");
    std::fs::create_dir_all(&video_dir_repeat).expect("create repeat directory");
    let repeat = run_in_dir(&home, &args(&video_dir_repeat));
    assert_eq!(repeat.status.code(), Some(0));
    let repeated =
        std::fs::read(video_dir_repeat.join("sound_cue.wav")).expect("read repeated WAV");
    assert_eq!(bytes, repeated, "the mix is deterministic");
}

#[test]
fn svg_still_publishes_round_trippable_deterministic_certified_bytes() {
    let run = |dir: &std::path::Path, extra: &[&str]| {
        let mut args: Vec<&str> = vec![
            "--robot",
            "--format",
            "svg",
            "--resolution",
            "96x54",
            "--fps",
            "8",
            "--threads",
            "1",
            "--video_dir",
            dir.to_str().expect("output path is UTF-8"),
        ];
        args.extend_from_slice(extra);
        args.push(BUILTIN_SCENE_SOURCE);
        args.push("circle_shift.v1");
        run_clean(&args)
    };
    let root = output_root("svg-positive");
    let first = run(&root, &[]);
    let stdout = String::from_utf8(first.stdout).expect("robot output is UTF-8");
    assert_eq!(first.status.code(), Some(0), "{stdout}");
    let bytes = std::fs::read(root.join("circle_shift.svg")).expect("read published SVG");
    let document = fmn::geometry::SvgDocument::parse(&bytes)
        .expect("the published artifact parses back as an SVG document");
    assert!(document.width > 0.0);
    assert!(!document.shapes.is_empty(), "the still carries shapes");

    // Deterministic emission: a fresh publication is byte-identical.
    let root_b = output_root("svg-determinism");
    run(&root_b, &[]);
    let repeated = std::fs::read(root_b.join("circle_shift.svg")).expect("read repeat");
    assert_eq!(bytes, repeated, "SVG emission is deterministic");

    // Certified SVG is certification-eligible (recorded ruling).
    let certified_root = output_root("svg-certified");
    let certified = run(&certified_root, &["--reproducible"]);
    let certified_stdout = String::from_utf8(certified.stdout).expect("robot output is UTF-8");
    assert_eq!(certified.status.code(), Some(0), "{certified_stdout}");
    let certified_bytes =
        std::fs::read(certified_root.join("circle_shift.svg")).expect("read certified SVG");
    assert_eq!(bytes, certified_bytes, "certified bytes match standard");

    // Publication limits stay named: a second generation into the same
    // directory refuses instead of clobbering.
    let rerun = run(&root, &[]);
    let rerun_stdout = String::from_utf8(rerun.stdout).expect("robot output is UTF-8");
    assert_eq!(rerun.status.code(), Some(6), "{rerun_stdout}");
    assert!(
        rerun_stdout.contains("\"exit_name\":\"render\""),
        "{rerun_stdout}"
    );
}

/// Launch the Studio front door for one scene and return the child plus
/// the readiness line's `(authority, capability query)`.
#[cfg(unix)]
fn spawn_studio_for(
    scene: &str,
) -> (
    std::process::Child,
    std::io::BufReader<std::process::ChildStdout>,
    String,
    String,
) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_fmn"))
        .args([
            "studio",
            "--robot",
            "--no-browser",
            "--resolution",
            "96x54",
            "--fps",
            "8",
            "--threads",
            "1",
            BUILTIN_SCENE_SOURCE,
            scene,
        ])
        .env_remove("PYTHONHOME")
        .env_remove("PYTHONPATH")
        .env("PATH", "")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("launch Studio front door");
    let mut stdout = std::io::BufReader::new(child.stdout.take().expect("Studio stdout"));
    let mut ready = String::new();
    stdout
        .read_line(&mut ready)
        .expect("read Studio readiness record");
    assert!(ready.contains("\"kind\":\"studio_ready\""), "{ready}");
    let url = json_string_field(&ready, "url").expect("readiness URL");
    let location = url.strip_prefix("http://").expect("loopback HTTP URL");
    let (authority, target) = location.split_once('/').expect("URL authority and target");
    let query = target.split_once('?').expect("capability query").1;
    (child, stdout, authority.to_owned(), query.to_owned())
}

/// Shut a spawned Studio front door down through stdin EOF and demand a
/// clean exit with silent stderr.
#[cfg(unix)]
fn shutdown_studio_cleanly(mut child: std::process::Child) {
    drop(child.stdin.take());
    let status = child.wait().expect("wait for graceful Studio shutdown");
    let mut stderr = String::new();
    child
        .stderr
        .take()
        .expect("Studio stderr")
        .read_to_string(&mut stderr)
        .expect("read Studio stderr");
    assert_eq!(status.code(), Some(0), "{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
}

/// One plain GET against the Studio API, response body and all.
#[cfg(unix)]
fn studio_get(authority: &str, target: &str) -> String {
    let mut stream = std::net::TcpStream::connect(authority).expect("connect to Studio API");
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .expect("Studio API timeout");
    write!(
        stream,
        "GET {target} HTTP/1.1\r\nHost: {authority}\r\nConnection: close\r\n\r\n"
    )
    .expect("write Studio API request");
    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .expect("read Studio API response");
    let _ = stream.shutdown(std::net::Shutdown::Both);
    String::from_utf8(response).expect("Studio API response is UTF-8")
}

#[cfg(unix)]
#[test]
fn studio_inspect_serves_real_tex_span_entries_for_the_tex_span_scene() {
    let (child, stdout, authority, query) = spawn_studio_for("tex_span.v1");
    let response = studio_get(&authority, &format!("/api/inspect?{query}"));
    drop(stdout);
    shutdown_studio_cleanly(child);

    assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
    // The formula's `x` glyph carries its own source bytes (6..7 of
    // `\frac{x}{y}`), verbatim excerpt included.
    assert!(
        response.contains(
            "\"source_span\":{\"kind\":\"math_glyph\",\"start\":6,\"end\":7,\
             \"source_bytes\":11,\"excerpt\":\"x\""
        ),
        "{response}"
    );
    // The fraction bar is ink the `\frac` keyword draws, so it maps to that
    // command token (0..5), not to the whole construct.
    assert!(
        response.contains(
            "\"source_span\":{\"kind\":\"math_rule\",\"start\":0,\"end\":5,\
             \"source_bytes\":11,\"excerpt\":\"\\\\frac\""
        ),
        "{response}"
    );
    // The text glyphs carry their per-character byte ranges.
    assert!(
        response.contains(
            "\"source_span\":{\"kind\":\"text_glyph\",\"start\":0,\"end\":1,\
             \"source_bytes\":5,\"excerpt\":\"h\""
        ),
        "{response}"
    );
    assert!(
        response.contains(
            "\"source_span\":{\"kind\":\"text_glyph\",\"start\":4,\"end\":5,\
             \"source_bytes\":5,\"excerpt\":\"o\""
        ),
        "{response}"
    );
}

#[cfg(unix)]
#[test]
fn studio_inspector_view_tracks_committed_transient_and_restarted_frames() {
    let (child, stdout, authority, query) = spawn_studio_for("tex_span.v1");
    // tex_span starts at constructed frame zero, then captures
    // ceil(0.25 * 30) = 8 samples on the native windowed clock.
    let inspect = || studio_get(&authority, &format!("/api/inspect?{query}"));
    let assert_view = |response: &str, frame| {
        assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
        let view = format!(
            "\"view\":{{\"frame_index\":{frame},\"frame_count\":9,\"fps\":30,\"width\":96,\"height\":54,\"scale\":6.75,\"origin\":[48,27],\"y_up\":true,\"input_events\":true,\"input_revision\":"
        );
        assert!(response.contains(&view), "{response}");
    };
    let initial = inspect();
    assert_view(&initial, 0);
    assert!(initial.contains("\"scene_time\":0,"));

    let committed = studio_post(
        &authority,
        &format!("/api/scrub?{query}"),
        b"frame=1&commit=true",
    );
    assert!(committed.starts_with("HTTP/1.1 200 OK"), "{committed}");
    assert!(committed.contains("\"frame_index\":1"), "{committed}");
    assert_view(&inspect(), 1);

    let transient = studio_post(
        &authority,
        &format!("/api/scrub?{query}"),
        b"frame=0&commit=false",
    );
    assert!(transient.starts_with("HTTP/1.1 200 OK"), "{transient}");
    assert!(transient.contains("\"frame_index\":0"), "{transient}");
    assert_view(&inspect(), 0);

    let outside = studio_post(
        &authority,
        &format!("/api/scrub?{query}"),
        b"frame=9&commit=false",
    );
    assert!(outside.starts_with("HTTP/1.1 422"), "{outside}");
    assert!(
        outside.contains("native target exceeds the scene or execution budget"),
        "{outside}"
    );
    assert_view(&inspect(), 0);

    let event = studio_post(
        &authority,
        &format!("/api/event?{query}"),
        b"type=key_press&key=arrow_left",
    );
    assert!(event.starts_with("HTTP/1.1 422"), "{event}");
    assert!(
        event.contains("worker_generation, frame and revision are required"),
        "{event}"
    );
    assert_view(&inspect(), 0);

    let restart = studio_post(&authority, &format!("/api/restart?{query}"), b"");
    assert!(restart.starts_with("HTTP/1.1 200 OK"), "{restart}");
    assert!(restart.contains("\"worker_generation\":2"), "{restart}");
    assert!(restart.contains("\"frame_index\":1"), "{restart}");
    assert_view(&inspect(), 1);
    drop(stdout);
    shutdown_studio_cleanly(child);
}

#[cfg(unix)]
#[test]
fn studio_inspect_serves_null_spans_for_a_scene_without_span_records() {
    let (child, stdout, authority, query) = spawn_studio_for("circle_shift.v1");
    let response = studio_get(&authority, &format!("/api/inspect?{query}"));
    drop(stdout);
    shutdown_studio_cleanly(child);

    assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
    // A primitive scene harvests no span records, so every node's
    // source span is null — inspection degrades gracefully, never
    // fails.
    assert!(!response.contains("\"source_span\":{"), "{response}");
    assert!(response.contains("\"source_span\":null"), "{response}");
}

#[test]
fn doctor_robot_snapshot_emits_all_expected_kinds() {
    let output = run_clean(&["doctor", "--robot"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).expect("doctor --robot output is UTF-8");
    let kinds: Vec<Option<&str>> = stdout
        .lines()
        .map(|line| json_string_field(line, "kind"))
        .collect();

    for expected in [
        "topology",
        "execution_plan",
        "ffmpeg",
        "cache",
        "fonts",
        "math_packs",
        "certification",
    ] {
        assert!(
            kinds.iter().any(|k| k == &Some(expected)),
            "missing doctor kind '{expected}' in:\n{stdout}"
        );
    }

    for line in stdout.lines() {
        assert!(
            line.starts_with('{') && line.ends_with('}'),
            "non-JSON line: {line}"
        );
        assert!(
            line.contains("\"schema\":\"fmn.doctor\""),
            "missing schema marker: {line}"
        );
    }
}
