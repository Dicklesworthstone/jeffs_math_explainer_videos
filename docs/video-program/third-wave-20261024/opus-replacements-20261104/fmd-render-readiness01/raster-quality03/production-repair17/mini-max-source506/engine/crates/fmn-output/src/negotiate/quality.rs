//! Typed, opt-in video encoder controls. These affect compressed video only,
//! never Lumen's raw frames. The absent settings preserve the encoder defaults.
//!
//! The catalogs follow x264's public preset/tune names and x265's documented
//! presets. A single tune is deliberate: arbitrary encoder parameter strings,
//! multiple psycho-visual tunes, and raw argv fragments are not accepted.

use super::{Container, EncoderChoice, NegotiationError, VideoJob};

/// The shared x264/x265 compression-speed preset catalog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncoderPreset {
    /// `ultrafast`.
    UltraFast,
    /// `superfast`.
    SuperFast,
    /// `veryfast`.
    VeryFast,
    /// `faster`.
    Faster,
    /// `fast`.
    Fast,
    /// `medium`.
    Medium,
    /// `slow`.
    Slow,
    /// `slower`.
    Slower,
    /// `veryslow`.
    VerySlow,
    /// `placebo`.
    Placebo,
}

impl EncoderPreset {
    /// Exact encoder argument. No user text is forwarded to the process.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UltraFast => "ultrafast",
            Self::SuperFast => "superfast",
            Self::VeryFast => "veryfast",
            Self::Faster => "faster",
            Self::Fast => "fast",
            Self::Medium => "medium",
            Self::Slow => "slow",
            Self::Slower => "slower",
            Self::VerySlow => "veryslow",
            Self::Placebo => "placebo",
        }
    }
}

impl std::str::FromStr for EncoderPreset {
    type Err = NegotiationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "ultrafast" => Ok(Self::UltraFast),
            "superfast" => Ok(Self::SuperFast),
            "veryfast" => Ok(Self::VeryFast),
            "faster" => Ok(Self::Faster),
            "fast" => Ok(Self::Fast),
            "medium" => Ok(Self::Medium),
            "slow" => Ok(Self::Slow),
            "slower" => Ok(Self::Slower),
            "veryslow" => Ok(Self::VerySlow),
            "placebo" => Ok(Self::Placebo),
            _ => Err(NegotiationError("unknown x264/x265 preset")),
        }
    }
}

/// One source/latency/metric tune, validated against the selected encoder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncoderTune {
    /// x264 film tuning; not supported by x265.
    Film,
    /// Animated source tuning.
    Animation,
    /// Grain retention tuning.
    Grain,
    /// x264 still-image tuning; not supported by x265.
    StillImage,
    /// Tune for PSNR measurement, not a perceptual quality claim.
    Psnr,
    /// Tune for SSIM measurement, not a perceptual quality claim.
    Ssim,
    /// Reduce decoder complexity.
    FastDecode,
    /// Reduce encoder buffering/latency.
    ZeroLatency,
}

impl EncoderTune {
    /// Exact encoder argument.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Film => "film",
            Self::Animation => "animation",
            Self::Grain => "grain",
            Self::StillImage => "stillimage",
            Self::Psnr => "psnr",
            Self::Ssim => "ssim",
            Self::FastDecode => "fastdecode",
            Self::ZeroLatency => "zerolatency",
        }
    }
}

impl std::str::FromStr for EncoderTune {
    type Err = NegotiationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "film" => Ok(Self::Film),
            "animation" => Ok(Self::Animation),
            "grain" => Ok(Self::Grain),
            "stillimage" => Ok(Self::StillImage),
            "psnr" => Ok(Self::Psnr),
            "ssim" => Ok(Self::Ssim),
            "fastdecode" => Ok(Self::FastDecode),
            "zerolatency" => Ok(Self::ZeroLatency),
            _ => Err(NegotiationError("unknown video encoder tune")),
        }
    }
}

/// Explicit compressed-video policy. `Default` requests no overrides.
///
/// CRF and target bitrate are mutually exclusive rate-control modes. Presets
/// and tunes are software x264/x265 options, not portable hardware settings.
/// Hardware requests may use `bitrate` for the explicitly admitted encoders;
/// their installation/capability checks still happen at the normal boundary.
/// Audio AAC settings are separate and are not represented by this type.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VideoQuality {
    /// Integer constant rate factor, 0 through 51, for libx264/libx265 only.
    pub crf: Option<u8>,
    /// Software encoder compression-speed preset.
    pub preset: Option<EncoderPreset>,
    /// Single encoder-specific tune.
    pub tune: Option<EncoderTune>,
    /// Target video bitrate in bits per second, not a strict output-size cap.
    /// Admitted for libx264/libx265, H.264/HEVC VideoToolbox, and
    /// H.264/HEVC/AV1 NVENC. Zero and simultaneous CRF are refused.
    pub bitrate: Option<u32>,
}

impl VideoQuality {
    /// Whether no encoder override was requested.
    #[must_use]
    pub const fn is_default(self) -> bool {
        self.crf.is_none() && self.preset.is_none() && self.tune.is_none() && self.bitrate.is_none()
    }

    pub(super) fn validate(
        self,
        encoder: Option<&str>,
        container: Container,
    ) -> Result<(), NegotiationError> {
        let software = matches!(encoder, Some("libx264" | "libx265"));
        // Preserve the legacy CRF refusal contract, including its precedence.
        if let Some(crf) = self.crf {
            if !software {
                return Err(NegotiationError(
                    "crf is a software x264/x265 knob; hardware encoders take none",
                ));
            }
            if crf > 51 {
                return Err(NegotiationError("crf outside 0..=51"));
            }
        }
        if self.crf.is_some() && self.bitrate.is_some() {
            return Err(NegotiationError(
                "crf and video bitrate are mutually exclusive",
            ));
        }
        if !self.is_default() && matches!(container, Container::Gif | Container::MovTransparent) {
            return Err(NegotiationError(
                "video quality overrides require an opaque MP4/MOV output",
            ));
        }
        if (self.preset.is_some() || self.tune.is_some()) && !software {
            return Err(NegotiationError(
                "preset and tune require libx264 or libx265",
            ));
        }
        if encoder == Some("libx265")
            && matches!(self.tune, Some(EncoderTune::Film | EncoderTune::StillImage))
        {
            return Err(NegotiationError(
                "film and stillimage tunes require libx264",
            ));
        }
        if let Some(bitrate) = self.bitrate {
            if bitrate == 0 {
                return Err(NegotiationError("video bitrate must be nonzero"));
            }
            if !matches!(
                encoder,
                Some(
                    "libx264"
                        | "libx265"
                        | "h264_videotoolbox"
                        | "hevc_videotoolbox"
                        | "h264_nvenc"
                        | "hevc_nvenc"
                        | "av1_nvenc"
                )
            ) {
                return Err(NegotiationError(
                    "video bitrate is not supported for this encoder",
                ));
            }
        }
        Ok(())
    }
}

impl VideoJob {
    /// Bind typed quality settings to this job's resolved encoder.
    ///
    /// Existing `VideoJob` struct literals and the legacy `crf` field remain
    /// supported. A legacy CRF and a configured CRF may agree; disagreement is
    /// an error, never last-writer-wins. Applying an empty policy is a no-op.
    /// The returned job travels through the ordinary sink/boundary, with no
    /// process-runner decorator and no post-negotiation argv rewriting.
    ///
    /// # Errors
    /// Any incompatible quality/encoder/container combination, including a
    /// nonempty policy for GIF's encoder-less mode.
    pub fn with_quality(mut self, quality: VideoQuality) -> Result<Self, NegotiationError> {
        let encoder = self.resolved_encoder()?;
        if quality.is_default() {
            return Ok(self);
        }
        self.encoder = EncoderChoice::Configured {
            name: encoder.ok_or(NegotiationError("GIF mode has no video quality encoder"))?,
            quality,
        };
        self.resolved_encoder()?;
        Ok(self)
    }

    /// Effective policy after reconciling the legacy CRF field. Returned only
    /// after the exact encoder/container has passed negotiation.
    ///
    /// # Errors
    /// The same errors as [`Self::resolved_encoder`].
    pub fn quality(&self) -> Result<VideoQuality, NegotiationError> {
        self.resolved_encoder()?;
        self.merged_quality()
    }

    pub(super) fn merged_quality(&self) -> Result<VideoQuality, NegotiationError> {
        let mut quality = match &self.encoder {
            EncoderChoice::Configured { quality, .. } => *quality,
            _ => VideoQuality::default(),
        };
        if let (Some(legacy), Some(configured)) = (self.crf, quality.crf)
            && legacy != configured
        {
            return Err(NegotiationError(
                "conflicting legacy and configured crf values",
            ));
        }
        quality.crf = quality.crf.or(self.crf);
        Ok(quality)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::negotiate::{ColorDescription, WireFormat, encode_argv};
    use std::path::Path;

    fn job(name: &str) -> VideoJob {
        VideoJob {
            width: 64,
            height: 36,
            fps: (30_000, 1_001),
            wire: WireFormat::Nv12,
            color: ColorDescription::video_bt709(),
            container: Container::Mp4,
            encoder: EncoderChoice::Named(name.to_owned()),
            crf: None,
        }
    }

    #[test]
    fn unrequested_settings_keep_the_exact_legacy_argv() {
        let expected = "-hide_banner -loglevel error -f rawvideo -pix_fmt nv12 -video_size 64x36 -framerate 30000/1001 -i - -c:v libx264 -pix_fmt yuv420p -movflags +faststart -color_primaries bt709 -color_trc bt709 -colorspace bt709 -color_range tv -y /out.mp4";
        for choice in [EncoderChoice::Auto, EncoderChoice::Named("libx264".into())] {
            let mut job = job("libx264");
            job.encoder = choice;
            let configured = job.clone().with_quality(VideoQuality::default()).unwrap();
            assert_eq!(configured, job);
            assert_eq!(
                encode_argv(&configured, Path::new("/out.mp4"))
                    .unwrap()
                    .join(" "),
                expected
            );
        }
    }

    #[test]
    fn all_crf_values_presets_and_encoder_tunes_reach_only_output_options() {
        let presets = [
            EncoderPreset::UltraFast,
            EncoderPreset::SuperFast,
            EncoderPreset::VeryFast,
            EncoderPreset::Faster,
            EncoderPreset::Fast,
            EncoderPreset::Medium,
            EncoderPreset::Slow,
            EncoderPreset::Slower,
            EncoderPreset::VerySlow,
            EncoderPreset::Placebo,
        ];
        let tunes = [
            EncoderTune::Film,
            EncoderTune::Animation,
            EncoderTune::Grain,
            EncoderTune::StillImage,
            EncoderTune::Psnr,
            EncoderTune::Ssim,
            EncoderTune::FastDecode,
            EncoderTune::ZeroLatency,
        ];
        for name in ["libx264", "libx265"] {
            for crf in 0..=51 {
                for preset in presets {
                    for tune in tunes {
                        let quality = VideoQuality {
                            crf: Some(crf),
                            preset: Some(preset),
                            tune: Some(tune),
                            bitrate: None,
                        };
                        let result = job(name).with_quality(quality);
                        if name == "libx265"
                            && matches!(tune, EncoderTune::Film | EncoderTune::StillImage)
                        {
                            assert!(result.is_err());
                            continue;
                        }
                        let configured = result.unwrap();
                        assert_eq!(configured.quality().unwrap(), quality);
                        let argv = encode_argv(&configured, Path::new("/out.mp4")).unwrap();
                        let input = argv.iter().position(|arg| arg == "-i").unwrap();
                        let codec = argv.iter().position(|arg| arg == "-c:v").unwrap();
                        assert!(codec > input + 1);
                        for (flag, value) in [
                            ("-crf", crf.to_string()),
                            ("-preset", preset.as_str().to_owned()),
                            ("-tune", tune.as_str().to_owned()),
                        ] {
                            assert_eq!(argv.iter().filter(|arg| **arg == flag).count(), 1);
                            let at = argv.iter().position(|arg| arg == flag).unwrap();
                            assert!(at > codec && at + 1 < argv.len() - 1);
                            assert_eq!(argv[at + 1], value);
                        }
                        assert!(
                            !argv
                                .iter()
                                .any(|arg| matches!(arg.as_str(), "-b:v" | "-vf" | "-af"))
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn every_out_of_range_crf_and_conflicting_rate_mode_refuses() {
        for crf in 52..=u8::MAX {
            assert!(
                job("libx264")
                    .with_quality(VideoQuality {
                        crf: Some(crf),
                        ..VideoQuality::default()
                    })
                    .is_err()
            );
        }
        assert!(
            job("libx264")
                .with_quality(VideoQuality {
                    crf: Some(18),
                    bitrate: Some(8_000_000),
                    ..VideoQuality::default()
                })
                .is_err()
        );
        assert!(
            job("libx264")
                .with_quality(VideoQuality {
                    bitrate: Some(0),
                    ..VideoQuality::default()
                })
                .is_err()
        );
    }

    #[test]
    fn hardware_bitrate_is_explicit_and_software_knobs_never_get_ignored() {
        for name in [
            "h264_nvenc",
            "hevc_nvenc",
            "av1_nvenc",
            "h264_videotoolbox",
            "hevc_videotoolbox",
        ] {
            let configured = job(name)
                .with_quality(VideoQuality {
                    bitrate: Some(12_000_000),
                    ..VideoQuality::default()
                })
                .unwrap();
            let argv = encode_argv(&configured, Path::new("/out.mp4")).unwrap();
            assert!(argv.windows(2).any(|pair| pair == ["-c:v", name]));
            assert!(argv.windows(2).any(|pair| pair == ["-b:v", "12000000"]));
            for quality in [
                VideoQuality {
                    crf: Some(18),
                    ..VideoQuality::default()
                },
                VideoQuality {
                    preset: Some(EncoderPreset::Slow),
                    ..VideoQuality::default()
                },
                VideoQuality {
                    tune: Some(EncoderTune::Animation),
                    ..VideoQuality::default()
                },
            ] {
                assert!(job(name).with_quality(quality).is_err());
            }
        }
        assert!(
            job("qtrle")
                .with_quality(VideoQuality {
                    bitrate: Some(12_000_000),
                    ..VideoQuality::default()
                })
                .is_err()
        );
    }

    #[test]
    fn legacy_crf_is_preserved_or_explicitly_conflicted_not_overwritten() {
        let mut legacy = job("libx264");
        legacy.crf = Some(16);
        let agreed = legacy
            .clone()
            .with_quality(VideoQuality {
                crf: Some(16),
                preset: Some(EncoderPreset::Slow),
                ..VideoQuality::default()
            })
            .unwrap();
        let argv = encode_argv(&agreed, Path::new("/out.mp4")).unwrap();
        assert_eq!(argv.iter().filter(|arg| arg.as_str() == "-crf").count(), 1);
        assert!(argv.windows(2).any(|pair| pair == ["-crf", "16"]));
        assert!(
            legacy
                .with_quality(VideoQuality {
                    crf: Some(18),
                    ..VideoQuality::default()
                })
                .is_err()
        );
    }

    #[test]
    fn closed_catalog_parsers_refuse_whitespace_aliases_and_argument_fragments() {
        for value in [
            "",
            "Slow",
            " slow",
            "slow ",
            "slow -vf scale=1:1",
            "slow\0",
            "p7",
        ] {
            assert!(value.parse::<EncoderPreset>().is_err());
        }
        for value in ["", "animation,grain", "animation\n", "film -y", "unknown"] {
            assert!(value.parse::<EncoderTune>().is_err());
        }
        assert_eq!(
            "slow".parse::<EncoderPreset>().unwrap(),
            EncoderPreset::Slow
        );
        assert_eq!(
            "animation".parse::<EncoderTune>().unwrap(),
            EncoderTune::Animation
        );
    }

    #[test]
    fn configured_jobs_revalidate_direct_mutations_before_encoding() {
        let mut configured = job("libx264")
            .with_quality(VideoQuality {
                crf: Some(16),
                preset: Some(EncoderPreset::Slow),
                ..VideoQuality::default()
            })
            .unwrap();
        configured.crf = Some(17);
        assert!(encode_argv(&configured, Path::new("/out.mp4")).is_err());
        configured.crf = None;
        configured.container = Container::MovTransparent;
        configured.wire = WireFormat::Rgba8;
        assert!(encode_argv(&configured, Path::new("/out.mov")).is_err());
        configured.container = Container::Gif;
        assert!(encode_argv(&configured, Path::new("/out.gif")).is_err());
    }
}
