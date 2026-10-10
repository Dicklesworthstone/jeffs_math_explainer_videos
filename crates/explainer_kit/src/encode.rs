//! Delivery-quality encoding through franken_manim's own ffmpeg boundary
//! (ported from the explainer skill's template, plus the camera route, raised
//! caps and a soundtrack capture).
//!
//! Without settings, `fmn::render` encodes at x264's default CRF 23 / preset
//! medium, which looks blurry. `RenderOptions::video_quality` covers x264/x265
//! CRF, but there is no NVENC constant-quality, `-gpu` or AAC bitrate knob. The
//! facade's documented seam for that is an injected `FfmpegCapability`; this
//! runner wraps the standard runner and only adjusts the argv and timeout of
//! the jobs it starts. Leave `video_quality` empty when using it. Its recorded
//! argv (`report.artifact.ffmpeg[i]`) is pre-wrapper; verify the bitstream
//! instead (ffprobe bitrate, `strings out.mp4 | grep crf=`).
//!
//! Two encoder families:
//! - `libx264`: constant rate factor, preset, `tune animation`.
//! - `h264_nvenc` / `hevc_nvenc`: NVENC constant quality (`-rc vbr -cq N
//!   -b:v 0`) at preset p7 / tune hq with spatial and temporal AQ and B-frames
//!   as references; never the x264 preset.
//!
//! Colour: fmn hands ffmpeg raw BT.709 limited-range frames but declares
//! neither on the raw input, only on the output, so ffmpeg converts them
//! from its default matrix on the way in: the corpus background #0E1116
//! (BT.709 codes 30.4 / 130.5 / 126.5) came out with Cb 129, green-black
//! (spike S-5). The runner declares the input's colour space and range
//! (the output's own), so the frames pass through untouched.
//!
//! The camera route (`render_camera`) installs no ffmpeg of its own, and a
//! camera segment encoded differently would corrupt the stream-copy join, so
//! [`render_camera`] here refuses a video render without the same capability
//! [`delivery_options`] installs for the retained route.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use fmn::platform::process::{
    ProcessCancellation, ProcessError, ProcessMechanism, ProcessOutcome, ProcessRunner, ProcessSpec,
    ProcessStdinLimits, RunningProcess, StdFfmpegLocator, StdProcessRunner,
};
use fmn::prelude::{FfmpegCapability, RenderFormat, SceneConstruct};
use fmn::rendering::{CameraConfig, RenderError, RenderOptions, RenderReport};
use fmn::scene::{CameraRig, Scene};

/// fmn's raw-input cap raised from 64 GiB (≈ 92 s of 4K60) to 256 GiB: an
/// 85 s chapter limit stays editorial pacing, not a safety margin.
pub const MAX_OUTPUT_BYTES: u64 = 256 << 30;
/// The encoded-artifact cap raised from 8 GiB to 32 GiB.
pub const MAX_ARTIFACT_BYTES: u64 = 32 << 30;

/// Rate control for the delivery encode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Quality {
    /// x264 CRF, or NVENC constant-quality level.
    pub quality: u8,
    /// The x264 preset (NVENC always uses p7).
    pub preset: &'static str,
    pub audio_bitrate: &'static str,
    /// Raised, never lowered, on every job.
    pub timeout: Duration,
    /// NVENC device index.
    pub gpu: Option<u32>,
    /// Copy fmn's s16 `soundtrack.wav` out of each mux job's work directory
    /// into this directory (the PCM master's null test needs it; the work
    /// directory is deleted after the job).
    pub capture_dir: Option<PathBuf>,
}

impl Quality {
    /// The corpus's delivery settings: CRF 16, preset slow, AAC 256k, 24 h.
    pub fn delivery() -> Self {
        Self {
            quality: 16,
            preset: "slow",
            audio_bitrate: "256k",
            timeout: Duration::from_secs(24 * 3600),
            gpu: None,
            capture_dir: None,
        }
    }
}

struct QualityRunner {
    inner: StdProcessRunner,
    quality: Quality,
    captured: AtomicUsize,
}

impl QualityRunner {
    fn new(quality: Quality) -> Self {
        Self {
            inner: StdProcessRunner,
            quality,
            captured: AtomicUsize::new(0),
        }
    }

    fn video_args(&self, encoder: &str) -> Vec<String> {
        let q = self.quality.quality.to_string();
        let mut args: Vec<String> = match encoder {
            "libx264" => ["-crf", &q, "-preset", self.quality.preset, "-tune", "animation"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            "h264_nvenc" | "hevc_nvenc" => [
                "-preset",
                "p7",
                "-tune",
                "hq",
                "-rc",
                "vbr",
                "-cq",
                &q,
                "-b:v",
                "0",
                "-spatial_aq",
                "1",
                "-temporal_aq",
                "1",
                "-rc-lookahead",
                "32",
                "-bf",
                "3",
                "-b_ref_mode",
                "middle",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
            _ => Vec::new(),
        };
        if encoder.ends_with("_nvenc")
            && let Some(gpu) = self.quality.gpu
        {
            args.extend(["-gpu".to_owned(), gpu.to_string()]);
        }
        args
    }

    fn adjust(&self, spec: &ProcessSpec) -> ProcessSpec {
        let mut s = spec.clone();
        // the video encode: rate control right after `-c:v <encoder>`
        if let Some(i) = s.argv.windows(2).position(|w| w[0] == "-c:v" && w[1] != "copy")
            && !s.argv.iter().any(|a| a == "-crf" || a == "-cq")
        {
            let extra = self.video_args(&s.argv[i + 1].clone());
            s.argv.splice(i + 2..i + 2, extra);
        }
        // the raw input's colour, declared as the output's (BT.709 and the
        // output's range), so ffmpeg does not convert from its default matrix
        if let Some(i) = s.argv.windows(2).position(|w| w[0] == "-i" && w[1] == "-")
            && s.argv[..i].windows(2).any(|w| w[0] == "-f" && w[1] == "rawvideo")
            && !s.argv[..i].iter().any(|a| a == "-colorspace")
        {
            let range = s.argv[i..]
                .windows(2)
                .find(|w| w[0] == "-color_range")
                .map_or("tv".to_owned(), |w| w[1].clone());
            s.argv.splice(
                i..i,
                [
                    "-colorspace".to_owned(),
                    "bt709".to_owned(),
                    "-color_range".to_owned(),
                    range,
                ],
            );
        }
        // the audio mux: a transparent AAC bitrate for the narration
        if let Some(i) = s.argv.windows(2).position(|w| w[0] == "-c:a" && w[1] == "aac")
            && !s.argv.iter().any(|a| a == "-b:a")
        {
            s.argv
                .splice(i + 2..i + 2, ["-b:a".to_owned(), self.quality.audio_bitrate.to_owned()]);
        }
        s.timeout = s.timeout.max(self.quality.timeout);
        s
    }

    /// Copy the job's `soundtrack.wav` input (if any) into the capture
    /// directory as `soundtrack_<n>.wav`, before the job runs.
    fn capture(&self, spec: &ProcessSpec) {
        let Some(dir) = &self.quality.capture_dir else { return };
        let Some(wav) = spec
            .argv
            .windows(2)
            .find(|w| w[0] == "-i" && Path::new(&w[1]).file_name().is_some_and(|n| n == "soundtrack.wav"))
            .map(|w| PathBuf::from(&w[1]))
        else {
            return;
        };
        let n = self.captured.fetch_add(1, Ordering::SeqCst);
        let _ = std::fs::create_dir_all(dir);
        if let Err(e) = std::fs::copy(&wav, dir.join(format!("soundtrack_{n}.wav"))) {
            eprintln!("soundtrack capture of {} failed: {e}", wav.display());
        }
    }
}

impl ProcessRunner for QualityRunner {
    fn mechanism(&self) -> ProcessMechanism {
        self.inner.mechanism()
    }

    fn start(
        &self,
        spec: &ProcessSpec,
        cancellation: ProcessCancellation,
        stdin_limits: ProcessStdinLimits,
    ) -> Result<Box<dyn RunningProcess>, ProcessError> {
        self.capture(spec);
        self.inner.start(&self.adjust(spec), cancellation, stdin_limits)
    }

    fn run(&self, spec: &ProcessSpec) -> Result<ProcessOutcome, ProcessError> {
        self.capture(spec);
        self.inner.run(&self.adjust(spec))
    }
}

/// The host ffmpeg capability with delivery-quality rate control.
pub fn capability(quality: Quality) -> FfmpegCapability {
    FfmpegCapability {
        runner: Arc::new(QualityRunner::new(quality)),
        locator: Arc::new(StdFfmpegLocator::from_host_path()),
        workdir_root: std::env::temp_dir(),
    }
}

/// Render options for a delivery render on either route: the quality runner
/// installed as the ffmpeg capability and both caps raised.
///
/// # Errors
/// If franken_manim's bundled configuration cannot be resolved.
pub fn delivery_options(
    output: impl Into<PathBuf>,
    format: RenderFormat,
    quality: Quality,
) -> Result<RenderOptions, fmn::config::ConfigError> {
    let mut o = RenderOptions::with_format(output, format)?;
    o.max_output_bytes = MAX_OUTPUT_BYTES;
    o.ffmpeg_limits.max_artifact_bytes = MAX_ARTIFACT_BYTES;
    o.ffmpeg = Some(capability(quality));
    Ok(o)
}

/// The attempts a render gets against franken_manim's transient Darwin
/// EPERM race (bead fm-darwin-killpg-eperm-race-7aae).
pub const RENDER_ATTEMPTS: u32 = 5;

/// Whether `error` is that race: the process-tree kill after ffmpeg has
/// already exited, refused by macOS. It clusters under heavy load (three in
/// a row at load ~300); a fresh render of the same scene succeeds.
pub fn is_transient_eperm(error: &RenderError) -> bool {
    format!("{error:?}").contains("process-tree completion kill failed: Operation not permitted")
}

/// Run `render` again, backing off, while it fails with the transient Darwin
/// EPERM ([`is_transient_eperm`]), up to [`RENDER_ATTEMPTS`]; returns the
/// last result and the attempt it came from. A failed attempt has consumed
/// its scene, so build the scene inside `render` (and reset its narrator:
/// the lines it already logged would otherwise be said twice).
pub fn retrying<T>(label: &str, mut render: impl FnMut() -> Result<T, RenderError>) -> (Result<T, RenderError>, u32) {
    let mut attempt = 1;
    loop {
        match render() {
            Err(e) if attempt < RENDER_ATTEMPTS && is_transient_eperm(&e) => {
                eprintln!("{label}: transient Darwin EPERM after ffmpeg exit (attempt {attempt}); rendering again");
                std::thread::sleep(Duration::from_millis(500 << attempt));
                attempt += 1;
            }
            result => return (result, attempt),
        }
    }
}

/// franken_manim's camera route, refusing a video render that lacks the
/// ffmpeg capability [`delivery_options`] installs (the camera route installs
/// none, and a differently encoded segment corrupts the stream-copy join).
///
/// # Errors
/// `InvalidOptions` for an MP4/MOV render without a capability; otherwise
/// whatever the camera route returns.
pub fn render_camera<F, P>(factory: F, options: RenderOptions) -> Result<RenderReport, RenderError>
where
    F: FnOnce(&mut Scene, &CameraConfig) -> fmn::Result<(P, CameraRig)>,
    P: SceneConstruct,
{
    if matches!(options.format, RenderFormat::Mp4 | RenderFormat::Mov) && options.ffmpeg.is_none() {
        return Err(RenderError::InvalidOptions(
            "a camera-route video render needs the delivery ffmpeg capability (explainer_kit::encode::delivery_options)",
        ));
    }
    fmn::rendering::render_camera(factory, options)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn runner(preset: &'static str, gpu: Option<u32>) -> QualityRunner {
        QualityRunner::new(Quality {
            preset,
            gpu,
            timeout: Duration::from_secs(4 * 3600),
            ..Quality::delivery()
        })
    }

    #[test]
    fn the_raw_input_is_declared_bt709_in_the_outputs_range_once() {
        let job = |range: &str| {
            spec(
                &[
                    "-f",
                    "rawvideo",
                    "-pix_fmt",
                    "nv12",
                    "-i",
                    "-",
                    "-c:v",
                    "libx264",
                    "-colorspace",
                    "bt709",
                    "-color_range",
                    range,
                    "out.mp4",
                ],
                600,
            )
        };
        let r = runner("slow", None);
        let out = r.adjust(&job("tv"));
        let argv = out.argv.join(" ");
        assert!(
            argv.contains("-pix_fmt nv12 -colorspace bt709 -color_range tv -i -"),
            "{argv}"
        );
        assert!(r.adjust(&job("pc")).argv.join(" ").contains("-color_range pc -i -"));
        assert_eq!(r.adjust(&out).argv, out.argv, "declared once");
        // a mux job (no raw input) is left alone
        let mux = spec(&["-i", "v.mp4", "-i", "a.wav", "-c:v", "copy", "o.mp4"], 600);
        assert_eq!(r.adjust(&mux).argv, mux.argv);
    }

    fn spec(argv: &[&str], timeout_secs: u64) -> ProcessSpec {
        ProcessSpec {
            program: PathBuf::from("/usr/bin/ffmpeg"),
            argv: argv.iter().map(|s| (*s).to_owned()).collect(),
            env: Vec::new(),
            cwd: None,
            stdin: None,
            timeout: Duration::from_secs(timeout_secs),
            max_output_bytes: 1 << 20,
        }
    }

    fn encode_job(encoder: &str) -> ProcessSpec {
        spec(
            &[
                "-f", "rawvideo", "-i", "-", "-c:v", encoder, "-pix_fmt", "yuv420p", "out.mp4",
            ],
            600,
        )
    }

    #[test]
    fn x264_gets_crf_preset_and_animation_tune_right_after_the_codec() {
        let out = runner("slow", None).adjust(&encode_job("libx264"));
        let i = out.argv.iter().position(|a| a == "-c:v").unwrap();
        assert_eq!(
            out.argv[i..i + 10],
            [
                "-c:v",
                "libx264",
                "-crf",
                "16",
                "-preset",
                "slow",
                "-tune",
                "animation",
                "-pix_fmt",
                "yuv420p"
            ]
        );
    }

    #[test]
    fn nvenc_gets_p7_constant_quality_never_the_x264_preset() {
        for encoder in ["h264_nvenc", "hevc_nvenc"] {
            let out = runner("slow", Some(1)).adjust(&encode_job(encoder));
            let argv = out.argv.join(" ");
            assert!(argv.contains("-preset p7 -tune hq -rc vbr -cq 16 -b:v 0"), "{argv}");
            assert!(argv.contains("-b_ref_mode middle -gpu 1 -pix_fmt"), "{argv}");
            assert!(!argv.contains("slow"), "{argv}");
        }
        let no_gpu = runner("slow", None).adjust(&encode_job("h264_nvenc"));
        assert!(!no_gpu.argv.iter().any(|a| a == "-gpu"));
    }

    #[test]
    fn x264_ignores_the_gpu_index() {
        let out = runner("medium", Some(0)).adjust(&encode_job("libx264"));
        assert!(!out.argv.iter().any(|a| a == "-gpu"));
    }

    #[test]
    fn videotoolbox_gets_no_rate_control() {
        let job = encode_job("h264_videotoolbox");
        let out = runner("slow", None).adjust(&job).argv;
        assert!(
            !out.iter()
                .any(|a| ["-crf", "-cq", "-preset", "-tune", "-b:v"].contains(&a.as_str()))
        );
        // nothing else changes but the raw input's declared colour
        let mut rest = out.clone();
        let i = rest.iter().position(|a| a == "-colorspace").unwrap();
        rest.drain(i..i + 4);
        assert_eq!(rest, job.argv);
    }

    #[test]
    fn stream_copies_and_existing_rate_control_are_left_alone() {
        let copy = spec(&["-i", "a.mp4", "-c:v", "copy", "out.mp4"], 600);
        assert_eq!(runner("slow", None).adjust(&copy).argv, copy.argv);
        let r = runner("slow", None);
        let once = r.adjust(&encode_job("libx264"));
        assert_eq!(r.adjust(&once).argv, once.argv, "adjusting twice is a no-op");
    }

    #[test]
    fn aac_mux_gets_the_narration_bitrate_once() {
        let mux = spec(&["-i", "v.mp4", "-i", "a.wav", "-c:a", "aac", "out.mp4"], 600);
        let out = runner("slow", None).adjust(&mux);
        assert_eq!(out.argv[4..8], ["-c:a", "aac", "-b:a", "256k"]);
        let preset = spec(&["-c:a", "aac", "-b:a", "128k", "out.mp4"], 600);
        assert_eq!(runner("slow", None).adjust(&preset).argv, preset.argv);
    }

    #[test]
    fn timeout_is_raised_never_lowered() {
        let r = runner("slow", None);
        let short = r.adjust(&encode_job("libx264"));
        assert_eq!(short.timeout, Duration::from_secs(4 * 3600));
        let long = r.adjust(&spec(&["-c:v", "libx264", "o.mp4"], 5 * 3600));
        assert_eq!(long.timeout, Duration::from_secs(5 * 3600));
    }

    #[test]
    fn the_soundtrack_is_captured_before_its_job_runs() {
        let base = std::env::temp_dir().join(format!("explainer_kit_capture_{}", std::process::id()));
        let work = base.join("job");
        std::fs::create_dir_all(&work).unwrap();
        let wav = work.join("soundtrack.wav");
        std::fs::write(&wav, b"RIFF....WAVEfmt ").unwrap();
        let cap = base.join("captured");
        let r = QualityRunner::new(Quality {
            capture_dir: Some(cap.clone()),
            ..Quality::delivery()
        });
        let job = spec(
            &[
                "-f",
                "rawvideo",
                "-i",
                "-",
                "-i",
                wav.to_str().unwrap(),
                "-c:v",
                "libx264",
                "-c:a",
                "aac",
                "o.mp4",
            ],
            600,
        );
        r.capture(&job);
        r.capture(&job);
        assert_eq!(
            std::fs::read(cap.join("soundtrack_0.wav")).unwrap(),
            b"RIFF....WAVEfmt "
        );
        assert!(cap.join("soundtrack_1.wav").exists(), "each job gets its own copy");
        // a job without a soundtrack input captures nothing
        let r2 = QualityRunner::new(Quality {
            capture_dir: Some(base.join("none")),
            ..Quality::delivery()
        });
        r2.capture(&encode_job("libx264"));
        assert!(!base.join("none").exists());
    }

    #[test]
    fn both_routes_get_the_same_capability_and_raised_caps() {
        let o = delivery_options("out.mp4", RenderFormat::Mp4, Quality::delivery()).unwrap();
        assert_eq!(o.max_output_bytes, 256 << 30);
        assert_eq!(o.ffmpeg_limits.max_artifact_bytes, 32 << 30);
        assert!(o.ffmpeg.is_some());
        // the camera route refuses a video render without that capability
        let bare = RenderOptions::with_format("out.mp4", RenderFormat::Mp4).unwrap();
        let err = render_camera(
            |_: &mut Scene, _: &CameraConfig| -> fmn::Result<(Nothing, CameraRig)> { unreachable!() },
            bare,
        )
        .unwrap_err();
        assert!(matches!(err, RenderError::InvalidOptions(_)), "{err:?}");
    }

    struct Nothing;
    impl SceneConstruct for Nothing {
        fn construct(&mut self, _: &mut fmn::prelude::Stage<'_>) -> fmn::Result<()> {
            Ok(())
        }
    }
}
