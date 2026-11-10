//! MP4/MOV through the facade: the one governed ffmpeg boundary (D2), with
//! scene sound cues mixed natively and muxed.
//!
//! Refusals need no process and always run. The end-to-end encode runs the
//! host ffmpeg through an explicitly injected capability; a host without
//! ffmpeg records a named skip (and fails under `FMN_REQUIRE_FULL_INPUTS=1`).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use fmn::prelude::*;
use fmn_platform::fs::{FileSystem, StdFs, VirtualFs};
use fmn_platform::process::{FfmpegLocator, StdFfmpegLocator, StdProcessRunner};

/// A quarter second of 440 Hz at half amplitude, 48 kHz mono PCM.
fn tone_wav() -> Vec<u8> {
    let samples: Vec<f32> = (0..12_000)
        .map(|index| 0.5 * (std::f32::consts::TAU * 440.0 * index as f32 / 48_000.0).sin())
        .collect();
    fmn_codec::encode_wav(1, 48_000, fmn_codec::SampleFormat::S16, &samples)
}

fn options(output: impl Into<PathBuf>, format: RenderFormat) -> RenderOptions {
    let mut options = RenderOptions::with_format(output, format).unwrap();
    options.config.camera.resolution = (64, 36);
    options.config.camera.fps = 10;
    options.config.render.threads = fmn_config::config::ThreadPolicy::Fixed(2);
    options
}

struct Chime {
    sound: Option<PathBuf>,
}

impl SceneConstruct for Chime {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let dot = stage.add(Circle::new().radius(0.8).color(BLUE))?;
        stage.wait(0.2)?;
        if let Some(sound) = &self.sound {
            stage
                .scene_mut()
                .add_sound(sound.clone(), 0.0, None, None)?;
        }
        let shift = dot.animate().shift(RIGHT)?;
        stage.play(shift)?;
        Ok(())
    }
}

/// A scene that must never run: every refusal precedes construction.
struct MustNotRun;

impl SceneConstruct for MustNotRun {
    fn construct(&mut self, _: &mut Stage<'_>) -> fmn::Result<()> {
        panic!("a refused video render must not execute the scene");
    }
}

fn host_capability() -> Option<FfmpegCapability> {
    let locator = StdFfmpegLocator::from_host_path();
    locator.locate_ffmpeg(Path::new("ffmpeg")).ok()?;
    Some(FfmpegCapability {
        runner: Arc::new(StdProcessRunner),
        locator: Arc::new(locator),
        workdir_root: std::env::temp_dir(),
    })
}

fn assert_refused(options: RenderOptions, fragment: &str) {
    let fs = Arc::new(VirtualFs::new());
    let error = render_with_fs(&mut MustNotRun, options, fs.clone()).unwrap_err();
    let message = error.to_string();
    assert!(
        message.contains(fragment),
        "expected {fragment:?} in refusal, got {message:?}"
    );
    assert!(fs.read(Path::new("/movie.mp4")).is_err());
}

#[test]
fn video_without_a_process_capability_is_a_named_refusal() {
    for format in [RenderFormat::Mp4, RenderFormat::Mov] {
        assert!(format.is_video());
        let error = render_with_fs(
            &mut MustNotRun,
            options("/movie.mp4", format),
            Arc::new(VirtualFs::new()),
        )
        .unwrap_err();
        assert!(matches!(error, RenderError::Capability(_)), "{error:?}");
        // The refusal names the native alternatives (D2), never substitutes.
        assert!(error.to_string().contains("PNG-sequence"), "{error}");
    }
    assert!(!RenderFormat::PngSequence.is_video());
}

#[test]
fn certified_video_is_refused_instead_of_mislabelled() {
    let mut options = options("/movie.mp4", RenderFormat::Mp4);
    options.config.determinism.mode = fmn_config::config::DeterminismMode::Certified;
    options.ffmpeg = host_capability();
    assert_refused(options, "outside the certified artifact set");
}

#[test]
fn incompatible_video_profiles_fail_before_any_process() {
    // These pure refusals must also work when no ffmpeg capability exists.
    // Odd dimensions cannot be 4:2:0.
    let mut odd = options("/movie.mp4", RenderFormat::Mp4);
    odd.config.camera.resolution = (65, 36);
    assert_refused(odd, "even width and height");

    let mut empty = options("/movie.mp4", RenderFormat::Mp4);
    empty.config.camera.resolution = (0, 36);
    assert_refused(empty, "zero frame dimensions");

    let mut stopped = options("/movie.mp4", RenderFormat::Mp4);
    stopped.config.camera.fps = 0;
    assert_refused(stopped, "fps must be nonzero");

    // A translucent background needs the alpha-preserving MOV route.
    let mut translucent = options("/movie.mp4", RenderFormat::Mp4);
    translucent.config.camera.background_opacity = 0.5;
    assert_refused(translucent, "RenderFormat::Mov");

    // Color transforms would need ffmpeg filters, which D2 forbids.
    let mut graded = options("/movie.mp4", RenderFormat::Mp4);
    graded.config.file_writer.gamma = 1.2;
    assert_refused(graded, "ffmpeg filters are forbidden");

    // Encoder names are argv tokens, never shell fragments.
    let mut hostile = options("/movie.mp4", RenderFormat::Mp4);
    hostile.config.file_writer.video_codec = "libx264 -vf eq".to_owned();
    assert_refused(hostile, "video_codec");

    let mut unknown_wire = options("/movie.mp4", RenderFormat::Mp4);
    unknown_wire.config.file_writer.pixel_format = "yuv444p".to_owned();
    assert_refused(unknown_wire, "pixel_format");

    let mut invalid_quality = options("/movie.mp4", RenderFormat::Mp4);
    invalid_quality.config.file_writer.video_crf = Some(52);
    assert_refused(invalid_quality, "file_writer.video_crf must lie in 0..=51");
}

/// Read an MP4/MOV box tree far enough to list top-level and `moov` track
/// handler types, without any external tool.
fn handler_types(bytes: &[u8]) -> Vec<[u8; 4]> {
    fn boxes(bytes: &[u8]) -> Vec<([u8; 4], &[u8])> {
        let mut out = Vec::new();
        let mut at = 0;
        while at + 8 <= bytes.len() {
            let size = u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap()) as usize;
            let kind: [u8; 4] = bytes[at + 4..at + 8].try_into().unwrap();
            let (header, size) = if size == 1 {
                let large = u64::from_be_bytes(bytes[at + 8..at + 16].try_into().unwrap());
                (16, usize::try_from(large).unwrap())
            } else if size == 0 {
                (8, bytes.len() - at)
            } else {
                (8, size)
            };
            if size < header || at + size > bytes.len() {
                break;
            }
            out.push((kind, &bytes[at + header..at + size]));
            at += size;
        }
        out
    }
    let mut handlers = Vec::new();
    for (kind, body) in boxes(bytes) {
        if &kind != b"moov" {
            continue;
        }
        for (kind, track) in boxes(body) {
            if &kind != b"trak" {
                continue;
            }
            for (kind, media) in boxes(track) {
                if &kind != b"mdia" {
                    continue;
                }
                for (kind, handler) in boxes(media) {
                    // hdlr: version/flags (4), pre_defined (4), handler_type (4).
                    if &kind == b"hdlr" && handler.len() >= 12 {
                        handlers.push(handler[8..12].try_into().unwrap());
                    }
                }
            }
        }
    }
    handlers
}

fn scratch(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("fmn-native-video-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    root.canonicalize().unwrap()
}

#[test]
fn mp4_and_mov_encode_through_the_boundary_with_a_muxed_soundtrack() {
    let Some(capability) = host_capability() else {
        fmn_core::test_inputs::skip_or_fail(
            "mp4_and_mov_encode_through_the_boundary_with_a_muxed_soundtrack",
            "an ffmpeg executable on PATH",
        );
        return;
    };
    let root = scratch("encode");
    let tone = root.join("tone.wav");
    std::fs::write(&tone, tone_wav()).unwrap();
    for (format, name) in [
        (RenderFormat::Mp4, "chime.mp4"),
        (RenderFormat::Mov, "chime.mov"),
    ] {
        let destination = root.join(name);
        let _ = std::fs::remove_file(&destination);
        let mut options = options(&destination, format);
        options.ffmpeg = Some(capability.clone());
        let report = render_with_fs(
            &mut Chime {
                sound: Some(tone.clone()),
            },
            options,
            Arc::new(StdFs),
        )
        .unwrap();
        // Every runtime capture reaches the encoder: one per clock frame.
        let clock_frames = u64::try_from(report.scene.time.frames()).unwrap();
        assert!(clock_frames >= 12, "{clock_frames}");
        assert_eq!(report.artifact.frame_count, clock_frames);
        assert_eq!(report.artifact.format, format);
        assert_eq!(report.artifact.path, destination);
        let bytes = std::fs::read(&destination).unwrap();
        assert_eq!(report.artifact.bytes, bytes.len() as u64);
        // Encode, then the two-stage audio mux, each with the hashed tool.
        assert_eq!(
            report.artifact.ffmpeg.len(),
            2,
            "{:?}",
            report.artifact.ffmpeg
        );
        assert!(
            report.artifact.ffmpeg.iter().all(|invocation| invocation
                .provenance
                .tool_sha256_hex
                .len()
                == 64)
        );
        let soundtrack = report.artifact.soundtrack.as_ref().unwrap();
        assert_eq!(soundtrack.cues_mixed, 1);
        assert_eq!(soundtrack.clipped_samples, 0);
        // The mix spans at least the whole picture timeline.
        assert!(soundtrack.sample_frames * 10 >= u64::from(soundtrack.sample_rate) * clock_frames);
        let handlers = handler_types(&bytes);
        assert!(handlers.contains(b"vide"), "{handlers:?}");
        assert!(handlers.contains(b"soun"), "{handlers:?}");
        // Publication is no-clobber: a second render refuses the destination.
        let mut again = self::options(&destination, format);
        again.ffmpeg = Some(capability.clone());
        assert!(render_with_fs(&mut Chime { sound: None }, again, Arc::new(StdFs)).is_err());
        assert_eq!(std::fs::read(&destination).unwrap(), bytes);
    }

    // A silent scene publishes a picture-only video.
    let silent = root.join("silent.mp4");
    let _ = std::fs::remove_file(&silent);
    let mut options = options(&silent, RenderFormat::Mp4);
    options.ffmpeg = Some(capability);
    let report = render_with_fs(&mut Chime { sound: None }, options, Arc::new(StdFs)).unwrap();
    assert!(report.artifact.soundtrack.is_none());
    assert_eq!(report.artifact.ffmpeg.len(), 1);
    let handlers = handler_types(&std::fs::read(&silent).unwrap());
    assert_eq!(handlers, vec![*b"vide"]);
    let _ = std::fs::remove_dir_all(&root);
}
