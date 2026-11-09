//! Native front-door admission, artifact budgets, and reported video policy.
//! Host encoding is an explicit input; admission and native-codec tests never
//! locate or spawn ffmpeg and always run.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use fmn::prelude::*;
use fmn_platform::fs::{FileSystem, VirtualFs};
use fmn_platform::process::{
    FfmpegExecutable, FfmpegLocator, FfmpegLocatorError, StdProcessRunner,
};

struct CountingLocator(Arc<AtomicUsize>);

impl FfmpegLocator for CountingLocator {
    fn locate_ffmpeg(&self, _: &Path) -> Result<FfmpegExecutable, FfmpegLocatorError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Err(FfmpegLocatorError::NotFound)
    }
}

struct MustNotRun;

impl SceneConstruct for MustNotRun {
    fn construct(&mut self, _: &mut Stage<'_>) -> fmn::Result<()> {
        panic!("invalid or unavailable video must not run scene code");
    }
}

struct Still;

impl SceneConstruct for Still {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.add(Circle::new().radius(0.4).color(BLUE))?;
        Ok(())
    }
}

fn options(format: RenderFormat) -> (RenderOptions, Arc<AtomicUsize>) {
    let calls = Arc::new(AtomicUsize::new(0));
    let mut options = RenderOptions::with_format("/movie", format).unwrap();
    options.config.camera.resolution = (32, 18);
    options.config.camera.fps = 5;
    options.config.render.threads = fmn_config::config::ThreadPolicy::Fixed(1);
    options.typeset_cache = false;
    options.ffmpeg = Some(FfmpegCapability {
        runner: Arc::new(StdProcessRunner),
        locator: Arc::new(CountingLocator(calls.clone())),
        workdir_root: std::env::temp_dir(),
    });
    (options, calls)
}

#[test]
fn invalid_video_limits_refuse_before_tool_discovery_or_scene_execution() {
    for format in [RenderFormat::Mp4, RenderFormat::Mov] {
        for invalid in 0..4 {
            let (mut options, calls) = options(format);
            let fragment = match invalid {
                0 => {
                    options.ffmpeg_limits.timeout = Duration::ZERO;
                    "timeout must be nonzero"
                }
                1 => {
                    options.ffmpeg_limits.max_log_bytes = 0;
                    "max_log_bytes must be nonzero"
                }
                2 => {
                    options.ffmpeg_limits.max_artifact_bytes = 0;
                    "byte limits must be nonzero"
                }
                _ => {
                    options.ffmpeg_limits.timeout = Duration::MAX;
                    "monotonic clock range"
                }
            };
            let fs = Arc::new(VirtualFs::new());
            let error = render_with_fs(&mut MustNotRun, options, fs.clone()).unwrap_err();
            assert!(matches!(error, RenderError::InvalidOptions(_)), "{error:?}");
            assert!(error.to_string().contains(fragment), "{error}");
            assert_eq!(calls.load(Ordering::SeqCst), 0);
            assert!(fs.read(Path::new("/movie")).is_err());
        }
    }
}

#[test]
fn explicit_long_video_allowance_reaches_the_real_capability_boundary() {
    for format in [RenderFormat::Mp4, RenderFormat::Mov] {
        let (mut options, calls) = options(format);
        options.ffmpeg_limits.timeout = Duration::from_secs(7_200);
        let error =
            render_with_fs(&mut MustNotRun, options, Arc::new(VirtualFs::new())).unwrap_err();
        assert!(matches!(
            error,
            RenderError::FfmpegUnavailable(FfmpegLocatorError::NotFound)
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn native_outputs_do_not_consult_an_irrelevant_ffmpeg_policy() {
    for format in [
        RenderFormat::PngSequence,
        RenderFormat::Gif,
        RenderFormat::Y4m,
    ] {
        let (mut options, calls) = options(format);
        options.ffmpeg_limits.timeout = Duration::ZERO;
        options.ffmpeg_limits.max_log_bytes = 0;
        options.ffmpeg_limits.max_artifact_bytes = 0;
        let report = render_with_fs(&mut Still, options, Arc::new(VirtualFs::new())).unwrap();
        assert_eq!(report.artifact.frame_count, 1);
        assert!(report.artifact.ffmpeg.is_empty());
        assert!(report.artifact.ffmpeg_limits.is_none());
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }
}

#[cfg(unix)]
#[test]
fn video_receipts_record_the_admitted_policy_and_tiny_artifacts_never_publish() {
    use fmn::rendering::JobLimits;
    use fmn_platform::fs::StdFs;
    use fmn_platform::process::StdFfmpegLocator;

    struct SoundStill(std::path::PathBuf);
    impl SceneConstruct for SoundStill {
        fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
            stage.add(Circle::new().radius(0.4).color(BLUE))?;
            stage
                .scene_mut()
                .add_sound(self.0.clone(), 0.0, None, None)?;
            stage.wait(0.2)?;
            Ok(())
        }
    }

    let locator = StdFfmpegLocator::from_host_path();
    if locator.locate_ffmpeg(Path::new("ffmpeg")).is_err() {
        fmn_core::test_inputs::skip_or_fail(
            "video_receipts_record_the_admitted_policy_and_tiny_artifacts_never_publish",
            "an ffmpeg executable on PATH",
        );
        return;
    }
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root =
        std::env::temp_dir().join(format!("fmn-video-limits-{}-{unique}", std::process::id()));
    // The root is the ffmpeg workdir root, which the boundary refuses when
    // group-writable: make it private whatever the host's umask.
    let mut private = std::fs::DirBuilder::new();
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut private, 0o700);
    private.create(&root).unwrap();
    let root = root.canonicalize().unwrap();
    let sound = root.join("cue.wav");
    std::fs::write(
        &sound,
        fmn_codec::encode_wav(1, 48_000, fmn_codec::SampleFormat::S16, &vec![0.0; 9_600]),
    )
    .unwrap();
    let capability = FfmpegCapability {
        runner: Arc::new(StdProcessRunner),
        locator: Arc::new(locator),
        workdir_root: root.clone(),
    };
    for (format, extension) in [(RenderFormat::Mp4, "mp4"), (RenderFormat::Mov, "mov")] {
        for camera in [false, true] {
            let (mut options, _) = options(format);
            options.output = root.join(format!("still-{camera}.{extension}"));
            options.ffmpeg = Some(capability.clone());
            options.max_output_bytes = 4 << 20;
            options.ffmpeg_limits = JobLimits {
                timeout: Duration::new(1_803, 123_456),
                max_log_bytes: 2 << 20,
                max_artifact_bytes: 2 << 20,
                keep_workdir: false,
            };
            if camera {
                options.camera = Some(options.camera_config().unwrap());
            }
            let report =
                render_with_fs(&mut SoundStill(sound.clone()), options, Arc::new(StdFs)).unwrap();
            let policy = report.artifact.ffmpeg_limits.unwrap();
            assert_eq!(policy.timeout, Duration::new(1_803, 123_456));
            assert_eq!(policy.max_log_bytes, 2 << 20);
            assert_eq!(policy.max_artifact_bytes, 2 << 20);
            assert_eq!(policy.max_input_bytes, 4 << 20);
            assert!(!policy.keep_workdir);
            assert_eq!(report.artifact.ffmpeg.len(), 2);
            assert!(report.artifact.soundtrack.is_some());
            assert!(report.artifact.bytes > 1);
            assert!(report.artifact.path.is_file());
        }
    }
    let (mut options, _) = options(RenderFormat::Mp4);
    let refused = root.join("over-budget.mp4");
    options.output = refused.clone();
    options.ffmpeg = Some(capability);
    options.ffmpeg_limits.max_artifact_bytes = 1;
    assert!(render_with_fs(&mut Still, options, Arc::new(StdFs)).is_err());
    assert!(
        !refused.exists(),
        "a rejected encode published a partial movie"
    );
    // Preserve the unique evidence directory; never remove another run's output.
}
