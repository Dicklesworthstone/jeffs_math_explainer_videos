//! Delivery-quality encoding through franken_manim's own ffmpeg boundary.
//!
//! `fmn::render` sends the encoder no quality settings (x264's default CRF
//! 23, preset medium) and caps each ffmpeg job at 600 s. The facade's
//! documented seam for hosts that govern processes themselves is an
//! injected `FfmpegCapability`; this runner wraps the standard exact-image
//! runner and only adjusts the argv/timeout of the jobs it starts.
//!
//! Two encoder families:
//! - `libx264`: constant rate factor, preset, `tune animation`.
//! - `h264_nvenc` / `hevc_nvenc`: NVENC constant quality (`-rc vbr -cq N
//!   -b:v 0`) at preset p7/hq with spatial+temporal AQ and B-frames as
//!   references. x264's slow-preset lookahead is largely serial, so a 4K
//!   encode tops out near one frame per second per stream on a Zen 3 core;
//!   NVENC removes the encoder from the critical path.

use std::sync::Arc;
use std::time::Duration;

use fmn::platform::process::{
    ProcessCancellation, ProcessError, ProcessMechanism, ProcessOutcome, ProcessRunner,
    ProcessSpec, ProcessStdinLimits, RunningProcess, StdFfmpegLocator, StdProcessRunner,
};
use fmn::prelude::FfmpegCapability;

pub struct Quality {
    /// x264 CRF, or NVENC constant-quality level.
    pub quality: u8,
    pub preset: &'static str,
    pub audio_bitrate: &'static str,
    pub timeout: Duration,
    /// NVENC device index.
    pub gpu: Option<u32>,
}

struct QualityRunner {
    inner: StdProcessRunner,
    quality: Quality,
}

impl QualityRunner {
    fn video_args(&self, encoder: &str) -> Vec<String> {
        let q = self.quality.quality.to_string();
        let mut args: Vec<String> = match encoder {
            "libx264" => vec![
                "-crf",
                &q,
                "-preset",
                self.quality.preset,
                "-tune",
                "animation",
            ]
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
        // The video encode: insert rate control right after `-c:v <encoder>`.
        if let Some(i) = s
            .argv
            .windows(2)
            .position(|w| w[0] == "-c:v" && w[1] != "copy")
            && !s.argv.iter().any(|a| a == "-crf" || a == "-cq")
        {
            let extra = self.video_args(&s.argv[i + 1].clone());
            s.argv.splice(i + 2..i + 2, extra);
        }
        // The audio mux: a transparent AAC bitrate for the narration.
        if let Some(i) = s
            .argv
            .windows(2)
            .position(|w| w[0] == "-c:a" && w[1] == "aac")
            && !s.argv.iter().any(|a| a == "-b:a")
        {
            s.argv.splice(
                i + 2..i + 2,
                ["-b:a".to_owned(), self.quality.audio_bitrate.to_owned()],
            );
        }
        s.timeout = s.timeout.max(self.quality.timeout);
        s
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
        self.inner
            .start(&self.adjust(spec), cancellation, stdin_limits)
    }

    fn run(&self, spec: &ProcessSpec) -> Result<ProcessOutcome, ProcessError> {
        self.inner.run(&self.adjust(spec))
    }
}

/// The host ffmpeg capability with delivery-quality rate control.
pub fn capability(quality: Quality) -> FfmpegCapability {
    FfmpegCapability {
        runner: Arc::new(QualityRunner {
            inner: StdProcessRunner,
            quality,
        }),
        locator: Arc::new(StdFfmpegLocator::from_host_path()),
        workdir_root: std::env::temp_dir(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn runner(preset: &'static str, gpu: Option<u32>) -> QualityRunner {
        QualityRunner {
            inner: StdProcessRunner,
            quality: Quality {
                quality: 16,
                preset,
                audio_bitrate: "256k",
                timeout: Duration::from_secs(4 * 3600),
                gpu,
            },
        }
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
        assert_eq!(
            out.argv[4..14],
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
            assert!(
                argv.contains("-preset p7 -tune hq -rc vbr -cq 16 -b:v 0"),
                "{argv}"
            );
            assert!(
                argv.contains("-b_ref_mode middle -gpu 1 -pix_fmt"),
                "{argv}"
            );
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
        assert_eq!(runner("slow", None).adjust(&job).argv, job.argv);
    }

    #[test]
    fn stream_copies_and_existing_rate_control_are_left_alone() {
        let copy = spec(&["-i", "a.mp4", "-c:v", "copy", "out.mp4"], 600);
        assert_eq!(runner("slow", None).adjust(&copy).argv, copy.argv);
        let r = runner("slow", None);
        let once = r.adjust(&encode_job("libx264"));
        assert_eq!(
            r.adjust(&once).argv,
            once.argv,
            "adjusting twice is a no-op"
        );
    }

    #[test]
    fn aac_mux_gets_the_narration_bitrate_once() {
        let mux = spec(
            &["-i", "v.mp4", "-i", "a.wav", "-c:a", "aac", "out.mp4"],
            600,
        );
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
}
