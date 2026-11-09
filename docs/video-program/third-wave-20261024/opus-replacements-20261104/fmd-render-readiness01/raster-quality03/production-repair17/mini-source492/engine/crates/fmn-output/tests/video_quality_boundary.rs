//! Exercise the real streaming boundary with Cargo's native fake-ffmpeg.
//! This proves actual spawned arguments, not video quality or bitstream content.
#![cfg(all(unix, feature = "ffmpeg-test-fixture"))]

use std::path::Path;
use std::sync::Arc;

use fmn_output::negotiate::{EncoderPreset, EncoderTune, VideoQuality};
use fmn_output::{
    Boundary, ColorDescription, Container, EncoderCapabilities, EncoderChoice, FfmpegTool,
    JobLimits, VideoJob, WireFormat,
};
use fmn_platform::process::{FfmpegLocator, StdFfmpegLocator, StdProcessRunner};

#[test]
fn actual_streaming_child_receives_the_negotiated_quality_exactly() {
    use std::os::unix::fs::PermissionsExt;

    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new("/tmp")
        .canonicalize()
        .unwrap()
        .join(format!("fmn-quality-argv-{}-{unique}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
    let locator = StdFfmpegLocator::default();
    let executable = locator
        .locate_ffmpeg(Path::new(env!("CARGO_BIN_EXE_fmn-ffmpeg-test-fixture")))
        .unwrap();
    let runner = Arc::new(StdProcessRunner);
    let tool = FfmpegTool::resolve(executable, runner.as_ref(), &root).unwrap();
    let caps = EncoderCapabilities::probe(&tool, runner.as_ref()).unwrap();
    let boundary = Boundary::new(
        tool,
        runner,
        JobLimits {
            keep_workdir: true,
            ..JobLimits::default()
        },
        root.clone(),
    )
    .unwrap();
    let policies = [
        (
            "libx264",
            VideoQuality {
                crf: Some(16),
                preset: Some(EncoderPreset::Slow),
                tune: Some(EncoderTune::Animation),
                bitrate: None,
            },
        ),
        (
            "libx264",
            VideoQuality {
                crf: Some(0),
                preset: Some(EncoderPreset::UltraFast),
                ..VideoQuality::default()
            },
        ),
        (
            "h264_nvenc",
            VideoQuality {
                bitrate: Some(12_000_000),
                ..VideoQuality::default()
            },
        ),
        ("libx264", VideoQuality::default()),
    ];
    for container in [Container::Mp4, Container::Mov] {
        for (index, (name, quality)) in policies.into_iter().enumerate() {
            let job = VideoJob {
                width: 64,
                height: 36,
                fps: (30_000, 1_001),
                wire: WireFormat::Nv12,
                color: ColorDescription::video_bt709(),
                container,
                encoder: EncoderChoice::Named(name.to_owned()),
                crf: None,
            }
            .with_quality(quality)
            .unwrap();
            let destination = root.join(format!("case-{index}.{}", container.extension()));
            // Boundary::encode feeds start_encode, write_stdin, prepare and
            // commit: the same streaming implementation used by FfmpegSink.
            let report = boundary
                .encode(&job, vec![0; 64 * 36 * 3 / 2], &caps, &destination)
                .unwrap();
            assert_eq!(std::fs::read(&destination).unwrap(), b"FAKEVIDEO");
            assert_eq!(report.invocations.len(), 1);
            let invocation = &report.invocations[0];
            let log =
                std::fs::read_to_string(invocation.artifact.parent().unwrap().join("argv.log"))
                    .unwrap();
            assert_eq!(log.trim_end(), invocation.provenance.argv.join(" "));
            assert_eq!(invocation.provenance.encoder.as_deref(), Some(name));
            assert_eq!(invocation.provenance.tool_sha256_hex.len(), 64);
            let argv = &invocation.provenance.argv;
            let input = argv.iter().position(|arg| arg == "-i").unwrap();
            for (flag, value) in [
                ("-crf", quality.crf.map(|value| value.to_string())),
                (
                    "-preset",
                    quality.preset.map(|value| value.as_str().to_owned()),
                ),
                ("-tune", quality.tune.map(|value| value.as_str().to_owned())),
                ("-b:v", quality.bitrate.map(|value| value.to_string())),
            ] {
                let positions: Vec<_> = argv
                    .iter()
                    .enumerate()
                    .filter_map(|(at, arg)| (arg == flag).then_some(at))
                    .collect();
                match value {
                    None => assert!(positions.is_empty()),
                    Some(value) => {
                        assert_eq!(positions.len(), 1);
                        let at = positions[0];
                        assert!(at > input + 1 && at + 1 < argv.len() - 1);
                        assert_eq!(argv[at + 1], value);
                    }
                }
            }
        }
    }
    // Deliberately retain the private argv logs and fixture outputs as evidence.
}
