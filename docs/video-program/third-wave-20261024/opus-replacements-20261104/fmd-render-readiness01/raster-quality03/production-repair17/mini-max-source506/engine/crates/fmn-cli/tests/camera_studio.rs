//! Authored camera export -> shipping Studio worker, without Python or ffmpeg.
#![cfg(feature = "batch")]

use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use std::time::{Duration, Instant};

use fmn::prelude::*;
use fmn::rendering::{CameraConfig, render_bundle_with_fs};
use fmn_cli::{Invocation, StudioCommand, compose_studio_preview_frame, parse_args};
use fmn_hash::sha256;
use fmn_platform::fs::{FileSystem, VirtualFs};
use fmn_scene::CameraRig;
use fmn_studio::protocol::studio_seek_command;
use fmn_studio::{
    CURRENT_VERSION, FramePayload, FrameStream, JournalReplay, ProtocolLimits, RequestEnvelope,
    SupervisorRequest, WorkerResponse, read_response, write_request,
};

const NAME: &str = "CameraStudio";
const SIZE: (u32, u32) = (64, 40);

struct Orbit {
    rig: CameraRig,
    camera: CameraConfig,
}
impl SceneConstruct for Orbit {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.add(Cube::new(1.6).color(BLUE))?;
        let mut target = self.camera.clone();
        target.frame.set_orientation([0.2, 0.7, 0.1, 1.0]).unwrap();
        target.frame.set_width(3.8).unwrap();
        target.light_source_position = [-3.0, 4.0, 5.0];
        let animation = self.rig.animate_to(stage.scene_mut(), &target)?;
        stage.play_prepared_with(
            vec![Box::new(animation)],
            PlayOverrides {
                run_time: Some(0.5),
                rate_func: Some(RateFunc::linear()),
                ..PlayOverrides::default()
            },
        )?;
        Ok(())
    }
}
fn source() -> Vec<u8> {
    let mut options = BundleExportOptions::new().unwrap();
    options.config.camera.fps = 8;
    options.config.camera.resolution = SIZE;
    options.config.render.threads = fmn_config::config::ThreadPolicy::Fixed(1);
    let mut camera = CameraConfig {
        resolution: SIZE,
        fps: 8,
        samples: 4,
        ..CameraConfig::default()
    };
    camera.frame.set_width(5.0).unwrap();
    camera.frame.set_orientation([0.1, 0.2, 0.0, 1.0]).unwrap();
    export_camera_bundle_bytes(
        |scene, base| {
            let rig = CameraRig::new(scene, base)?;
            Ok((
                Orbit {
                    rig,
                    camera: base.clone(),
                },
                rig,
            ))
        },
        camera,
        options,
    )
    .unwrap()
    .bundle
    .bytes
}

fn legacy_source() -> Vec<u8> {
    struct MovingSurface;
    impl SceneConstruct for MovingSurface {
        fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
            let cube = stage.add(Cube::new(1.6).color(BLUE))?;
            stage.play(
                cube.animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(0.5),
                        rate_func: Some(fmn_core::rate::linear),
                        ..AnimateArgs::default()
                    })?
                    .shift([0.5, 0.0, 0.0])?,
            )?;
            Ok(())
        }
    }
    let mut options = BundleExportOptions::new().unwrap();
    options.config.camera.fps = 8;
    options.config.camera.resolution = SIZE;
    export_bundle_bytes(&mut MovingSurface, options)
        .unwrap()
        .bundle
        .bytes
}
fn command(path: &str, threads: &str) -> StudioCommand {
    let Invocation::Studio(command) = parse_args([
        "studio",
        "--resolution",
        "64x40",
        "--fps",
        "8",
        "--threads",
        threads,
        path,
    ])
    .unwrap() else {
        panic!("Studio")
    };
    command
}
fn frame_bytes(frame: &FrameStream) -> &[u8] {
    let FramePayload::Pipe { bytes, digest } = &frame.payload else {
        panic!("PNG")
    };
    assert_eq!(sha256(bytes), *digest);
    bytes
}
fn pixels(bytes: &[u8]) -> Vec<u8> {
    fmn_codec::decode_png(bytes, &fmn_codec::PngLimits::default())
        .unwrap()
        .rgba
}
fn expected(source: &[u8]) -> Vec<Vec<u8>> {
    let fs = Arc::new(VirtualFs::new());
    let mut options = RenderOptions::new("/replay").unwrap();
    options.config.camera.resolution = SIZE;
    options.config.camera.fps = 8;
    options.config.render.threads = fmn_config::config::ThreadPolicy::Fixed(1);
    if !fmn_scene::TimelineBundle::from_bytes(source)
        .unwrap()
        .has_camera_track()
    {
        options.camera = Some(options.camera_config().unwrap());
    }
    render_bundle_with_fs(source, options, fs.clone()).unwrap();
    (0..4)
        .map(|index| {
            fs.read(&Path::new("/replay").join(format!("frame_{index:06}.png")))
                .unwrap()
        })
        .collect()
}

#[test]
fn camera_studio_composition_matches_native_bundle_output_and_preserves_clock() {
    let source = source();
    let expected = expected(&source);
    assert_ne!(pixels(&expected[0]), pixels(&expected[3]));
    let fs = VirtualFs::new();
    fs.insert("/CameraStudio.fmtl", source);
    for threads in ["1", "4", "16"] {
        let command = command("/CameraStudio.fmtl", threads);
        for index in [3, 0, 2, 1] {
            let frame = compose_studio_preview_frame(&fs, &command, index).unwrap();
            assert_eq!(
                pixels(frame_bytes(&frame)),
                pixels(&expected[index as usize])
            );
            assert_eq!(frame.scene, NAME);
            assert_eq!(frame.frame_index, index as u64);
        }
    }
    let mut wrong = command("/CameraStudio.fmtl", "1");
    wrong.render.fps = Some(30);
    assert!(compose_studio_preview_frame(&fs, &wrong, 0).is_err());
    wrong.render.fps = Some(8);
    wrong.render.skip_animations = true;
    assert!(compose_studio_preview_frame(&fs, &wrong, 0).is_err());
    wrong.render.skip_animations = false;
    assert!(compose_studio_preview_frame(&fs, &wrong, -1).is_err());
    assert!(compose_studio_preview_frame(&fs, &wrong, 4).is_err());
}

static NEXT: AtomicU64 = AtomicU64::new(0);
fn fixture() -> PathBuf {
    for _ in 0..128 {
        let root = std::env::temp_dir().join(format!(
            "fmn-camera-studio-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        match std::fs::create_dir(&root) {
            Ok(()) => return root,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => panic!("fixture: {error}"),
        }
    }
    panic!("fixture allocation exhausted")
}

fn exchange(
    root: &Path,
    requests: Vec<SupervisorRequest>,
    threads: &str,
    builtin: Option<&str>,
) -> Vec<WorkerResponse> {
    let limits = ProtocolLimits::default();
    let mut encoded = Vec::new();
    let all = std::iter::once(SupervisorRequest::Hello {
        version: CURRENT_VERSION,
        supervisor_build: sha256(b"camera-studio-test-host"),
        max_frame_bytes: limits.max_frame_bytes as u64,
    })
    .chain(requests)
    .chain(std::iter::once(SupervisorRequest::Shutdown));
    let mut count = 0;
    for (index, request) in all.enumerate() {
        write_request(
            &mut encoded,
            &RequestEnvelope {
                request_id: index as u64 + 1,
                request,
            },
            limits,
        )
        .unwrap();
        count += 1;
    }
    // This is the actual worker subprocess owned by fmn studio, not an engine
    // substitute. No executable search path or interpreter is available to it.
    let mut command = Command::new(env!("CARGO_BIN_EXE_fmn"));
    command.arg(fmn_cli::INTERNAL_STUDIO_WORKER_ARG).args([
        "studio",
        "--resolution",
        "64x40",
        "--fps",
        "8",
        "--threads",
        threads,
    ]);
    if let Some(name) = builtin {
        command.args(["@builtin", name]);
    } else {
        command.arg(root.join("CameraStudio.fmtl"));
    }
    let mut child = command
        .current_dir(root)
        .env_clear()
        .env("PATH", "")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let out = child.stdout.take().unwrap();
    let err = child.stderr.take().unwrap();
    let stdout = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        out.take(16 * 1024 * 1024).read_to_end(&mut bytes).unwrap();
        bytes
    });
    let stderr = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        err.take(1024 * 1024).read_to_end(&mut bytes).unwrap();
        bytes
    });
    child.stdin.take().unwrap().write_all(&encoded).unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!(
                "camera worker timed out; fixture retained at {}",
                root.display()
            );
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    let stderr = stderr.join().unwrap();
    assert!(status.success(), "{}", String::from_utf8_lossy(&stderr));
    let mut response = Cursor::new(stdout.join().unwrap());
    let mut results = Vec::new();
    for index in 0..count {
        let envelope = read_response(&mut response, limits).unwrap();
        assert_eq!(envelope.request_id, index as u64 + 1);
        assert!(
            !matches!(
                envelope.response,
                WorkerResponse::Error { .. } | WorkerResponse::Crash(_)
            ),
            "{:?}",
            envelope.response
        );
        results.push(envelope.response);
    }
    assert_eq!(response.position() as usize, response.get_ref().len());
    results
}

#[test]
fn camera_studio_shipping_worker_roundtrips_frames_and_cold_recovers_without_source_code() {
    let root = fixture();
    let source = source();
    let expected = expected(&source);
    std::fs::write(root.join("CameraStudio.fmtl"), &source).unwrap();
    let responses = exchange(
        &root,
        vec![
            SupervisorRequest::Scrub {
                scene: NAME.into(),
                frame: 3,
            },
            SupervisorRequest::Play {
                scene: NAME.into(),
                command: studio_seek_command(NAME, 3).unwrap(),
            },
            SupervisorRequest::Inspect { scene: NAME.into() },
        ],
        "1",
        None,
    );
    let WorkerResponse::Frame(frame) = &responses[1] else {
        panic!("frame")
    };
    assert_eq!(pixels(frame_bytes(frame)), pixels(&expected[3]));
    let WorkerResponse::JournalSegment {
        journal,
        start_entry,
        ..
    } = &responses[2]
    else {
        panic!("journal")
    };
    assert_eq!(*start_entry, 0);
    let WorkerResponse::StudioData { bytes, .. } = &responses[3] else {
        panic!("inspector")
    };
    let json = String::from_utf8(bytes.clone()).unwrap();
    assert!(json.contains("\"frame_index\":3"));
    assert!(json.contains("\"fps\":8"));
    assert!(json.contains("\"scene_time\":0.5"));
    let recovered = exchange(
        &root,
        vec![
            SupervisorRequest::ReplayJournal(JournalReplay {
                scene: NAME.into(),
                from_entry: 0,
                through_entry: 1,
                journal: journal.clone(),
            }),
            SupervisorRequest::Scrub {
                scene: NAME.into(),
                frame: 3,
            },
        ],
        "4",
        None,
    );
    assert!(matches!(
        recovered[1],
        WorkerResponse::ReplayComplete { from_entry: 0, .. }
    ));
    let WorkerResponse::Frame(frame) = &recovered[2] else {
        panic!("recovered frame")
    };
    assert_eq!(pixels(frame_bytes(frame)), pixels(&expected[3]));
    assert_eq!(
        std::fs::read(root.join("CameraStudio.fmtl")).unwrap(),
        source
    );
}

#[test]
fn studio_shipping_worker_renders_legacy_surfaces_with_its_explicit_fixed_camera() {
    let root = fixture();
    let source = legacy_source();
    let expected = expected(&source);
    assert_ne!(pixels(&expected[0]), pixels(&expected[3]));
    std::fs::write(root.join("CameraStudio.fmtl"), &source).unwrap();
    for threads in ["1", "4"] {
        let responses = exchange(
            &root,
            vec![
                SupervisorRequest::Scrub {
                    scene: NAME.into(),
                    frame: 3,
                },
                SupervisorRequest::Scrub {
                    scene: NAME.into(),
                    frame: 0,
                },
                SupervisorRequest::Inspect { scene: NAME.into() },
            ],
            threads,
            None,
        );
        for (response, index) in [(1, 3), (2, 0)] {
            let WorkerResponse::Frame(frame) = &responses[response] else {
                panic!("legacy frame")
            };
            assert_eq!(pixels(frame_bytes(frame)), pixels(&expected[index]));
        }
        let WorkerResponse::StudioData { bytes, .. } = &responses[3] else {
            panic!("legacy inspector")
        };
        let info = String::from_utf8(bytes.clone()).unwrap();
        assert!(info.contains("\"fps\":8"));
        assert!(info.contains("\"frame_count\":4"));
    }
}

#[test]
fn studio_shipping_mixed_camera_builtin_uses_native_steps_and_recovers_across_thread_caps() {
    const BUILTIN: &str = "mixed_camera.v1";
    let root = fixture();
    let responses = exchange(
        &root,
        vec![
            SupervisorRequest::Scrub {
                scene: BUILTIN.into(),
                frame: 0,
            },
            SupervisorRequest::Scrub {
                scene: BUILTIN.into(),
                frame: 8,
            },
            SupervisorRequest::Play {
                scene: BUILTIN.into(),
                command: studio_seek_command(BUILTIN, 8).unwrap(),
            },
            SupervisorRequest::Inspect {
                scene: BUILTIN.into(),
            },
        ],
        "1",
        Some(BUILTIN),
    );
    let WorkerResponse::Frame(initial) = &responses[1] else {
        panic!("initial native frame")
    };
    let WorkerResponse::Frame(last) = &responses[2] else {
        panic!("last native frame")
    };
    assert_ne!(pixels(frame_bytes(initial)), pixels(frame_bytes(last)));
    let WorkerResponse::JournalSegment { journal, .. } = &responses[3] else {
        panic!("native journal")
    };
    let entries = fmn_scene::Journal::from_bytes(journal).unwrap();
    assert!(
        entries.entries()[0]
            .reads
            .iter()
            .any(|read| read.path == "native/camera-capture-policy")
    );
    let WorkerResponse::StudioData { bytes, .. } = &responses[4] else {
        panic!("native inspector")
    };
    let info = String::from_utf8(bytes.clone()).unwrap();
    assert!(info.contains("\"fps\":30"));
    assert!(info.contains("\"frame_count\":9"));
    let recovered = exchange(
        &root,
        vec![
            SupervisorRequest::ReplayJournal(JournalReplay {
                scene: BUILTIN.into(),
                from_entry: 0,
                through_entry: 1,
                journal: journal.clone(),
            }),
            SupervisorRequest::Scrub {
                scene: BUILTIN.into(),
                frame: 8,
            },
        ],
        "4",
        Some(BUILTIN),
    );
    assert!(matches!(
        recovered[1],
        WorkerResponse::ReplayComplete { .. }
    ));
    let WorkerResponse::Frame(frame) = &recovered[2] else {
        panic!("recovered native frame")
    };
    assert_eq!(pixels(frame_bytes(frame)), pixels(frame_bytes(last)));
}
