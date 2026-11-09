//! Native sound-cue composition and retained input receipts for video front doors.

use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::path::PathBuf;
use std::sync::Arc;

use fmn_output::{
    AudioDecodeReport, AudioDecoder, DecodedAudio, InvocationReport, MixReport, MixerConfig,
    SoundCue, SoundMixer,
};
use fmn_platform::fs::FileSystem;
use fmn_scene::SoundRequest;

use super::RenderError;

const MAX_SOUND_ASSET_BYTES: usize = 64 * 1024 * 1024;
const MAX_OMITTED_INTERVALS: usize = 1_000_000;

/// Map the observed scene frame grid to a video's output frame grid.
///
/// Skipped intervals advance scene time without entering the movie. Observe
/// lifecycle boundaries in order, with the skip status that applies after
/// each boundary. All observations and mapped cues use the same scene grid.
/// Cue-relative offsets remain separate and are applied once by the mixer.
#[derive(Clone, Debug, Default)]
pub struct SoundtrackTimeline {
    last_frame: i64,
    skipping: bool,
    omitted: Vec<(i64, i64)>,
}

impl SoundtrackTimeline {
    /// Observe a nonnegative scene frame and its following skip status.
    ///
    /// # Errors
    /// A clock rewind or an exhausted interval/allocation budget. Rejected
    /// observations do not change the timeline.
    pub fn observe(&mut self, frame: i64, skipping: bool) -> Result<(), RenderError> {
        if frame < self.last_frame {
            return Err(RenderError::InvalidOptions(
                "soundtrack timeline cannot rewind an active output generation",
            ));
        }
        if self.skipping && frame > self.last_frame {
            if let Some(last) = self.omitted.last_mut()
                && last.1 == self.last_frame
            {
                last.1 = frame;
            } else {
                if self.omitted.len() >= MAX_OMITTED_INTERVALS {
                    return Err(RenderError::InvalidOptions(
                        "soundtrack skipped-interval budget exhausted",
                    ));
                }
                self.omitted.try_reserve(1).map_err(|_| {
                    RenderError::InvalidOptions("soundtrack interval allocation failed")
                })?;
                self.omitted.push((self.last_frame, frame));
            }
        }
        self.last_frame = frame;
        self.skipping = skipping;
        Ok(())
    }

    /// Rebase one observed scene-frame position after removing skipped time.
    ///
    /// # Errors
    /// A negative position or one beyond the latest observation.
    pub fn output_frame(&self, frame: i64) -> Result<i64, RenderError> {
        if frame < 0 || frame > self.last_frame {
            return Err(RenderError::InvalidOptions(
                "soundtrack timestamp is outside the observed scene timeline",
            ));
        }
        // Ordered, disjoint intervals lie within [0, last_frame], so their
        // intersection with [0, frame] cannot exceed frame or overflow i64.
        let removed: i64 = self
            .omitted
            .iter()
            .map(|&(start, end)| (frame.min(end) - start).max(0))
            .sum();
        Ok(frame - removed)
    }
}

/// Exact source bytes and native decode identity of one insertion-ordered cue.
///
/// Shared bytes let several subdivision receipts name the same prepared input
/// without copying its buffer. Provenance may use `report.source_digest`
/// directly; it must not reopen the authored path after decoding.
#[derive(Clone, Debug)]
pub struct SoundtrackInput {
    pub path: PathBuf,
    pub bytes: Arc<[u8]>,
    pub report: AudioDecodeReport,
}

/// One prepared, insertion-ordered native mix shared by complete renders and
/// absolute sample windows. Decoding, channel mapping and resampling happen
/// once; each window preserves source phase and background ducking.
#[derive(Debug)]
pub struct PreparedSceneSoundtrack {
    mixer: SoundMixer,
    inputs: Vec<SoundtrackInput>,
    invocations: Vec<InvocationReport>,
}

impl PreparedSceneSoundtrack {
    /// Read and decode the exact sound requests through caller capabilities.
    ///
    /// Each distinct authored path is read once, bounded to 64 MiB, and
    /// decoded once. Repeated cues share that immutable source snapshot even
    /// if the filesystem changes during preparation. The injected decoder owns
    /// native PCM decoding and any governed ffmpeg fallback. `Some(timeline)` rebases
    /// cue call sites after skipped time; `None` retains absolute scene time
    /// for phase-preserving subdivision windows. Relative offsets and gains
    /// retain the native mixer's semantics in either mode.
    ///
    /// # Errors
    /// Invalid mix/placement budgets, timestamps outside the observed grid,
    /// bounded asset reads, decode failures, or invalid PCM/gain parameters.
    pub fn new(
        fs: &dyn FileSystem,
        requests: &[SoundRequest],
        decoder: &mut AudioDecoder,
        config: MixerConfig,
        timeline: Option<&SoundtrackTimeline>,
    ) -> Result<Self, RenderError> {
        let mut mixer = SoundMixer::new(config).map_err(RenderError::Sound)?;
        let mut inputs = Vec::new();
        inputs
            .try_reserve_exact(requests.len())
            .map_err(|_| RenderError::InvalidOptions("soundtrack input allocation failed"))?;
        let mut sources: BTreeMap<PathBuf, (Arc<[u8]>, DecodedAudio)> = BTreeMap::new();
        let mut invocations = Vec::new();
        for request in requests {
            let frame = match timeline {
                Some(timeline) => timeline.output_frame(request.time.frames())?,
                None => request.time.frames(),
            };
            let (bytes, decoded) = match sources.entry(request.sound_file.clone()) {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => {
                    let bytes = fs
                        .read_bounded(&request.sound_file, MAX_SOUND_ASSET_BYTES)
                        .map_err(|error| RenderError::SoundAsset {
                            path: request.sound_file.clone(),
                            error,
                        })?;
                    let decoded =
                        decoder
                            .decode(&bytes)
                            .map_err(|error| RenderError::SoundDecode {
                                path: request.sound_file.clone(),
                                error,
                            })?;
                    if let Some(invocation) = decoded.report.invocation.clone() {
                        invocations.try_reserve(1).map_err(|_| {
                            RenderError::InvalidOptions("soundtrack invocation allocation failed")
                        })?;
                        invocations.push(invocation);
                    }
                    entry.insert((Arc::from(bytes), decoded))
                }
            };
            mixer
                .add(SoundCue {
                    audio: decoded.audio.clone(),
                    frame,
                    fps: request.time.fps(),
                    time_offset: request.time_offset,
                    gain: request.gain,
                    gain_to_background: request.gain_to_background,
                })
                .map_err(RenderError::Sound)?;
            inputs.push(SoundtrackInput {
                path: request.sound_file.clone(),
                bytes: Arc::clone(bytes),
                report: decoded.report.clone(),
            });
        }
        Ok(Self {
            mixer,
            inputs,
            invocations,
        })
    }

    /// Set the complete mix's minimum duration, counted in output sample
    /// frames. Explicit windows retain their own exact length.
    #[must_use]
    pub fn with_timeline_frames(mut self, sample_frames: u64) -> Self {
        self.mixer = self.mixer.with_timeline_frames(sample_frames);
        self
    }

    /// Mix the complete prepared timeline, including cue tails.
    ///
    /// # Errors
    /// The native mixer's worker, output-budget and sample-accumulation errors.
    pub fn mix(&self, threads: usize) -> Result<MixReport, RenderError> {
        self.mixer.mix(threads).map_err(RenderError::Sound)
    }

    /// Mix exactly one half-open interval of the absolute output-sample grid.
    /// No prefix or future tail is allocated. Convert the start and end scene
    /// positions independently with `frames_to_samples` to avoid rounding
    /// gaps between adjacent clips on non-dividing frame rates.
    ///
    /// # Errors
    /// The native mixer's range, worker, output-budget and accumulation errors.
    pub fn mix_window(
        &self,
        start_sample_frame: u64,
        sample_frames: u64,
        threads: usize,
    ) -> Result<MixReport, RenderError> {
        self.mixer
            .mix_window(start_sample_frame, sample_frames, threads)
            .map_err(RenderError::Sound)
    }

    /// Inputs in authored cue order, retaining the bytes actually decoded.
    #[must_use]
    pub fn inputs(&self) -> &[SoundtrackInput] {
        &self.inputs
    }

    /// Actual media-decode invocations in first-source-use order; repeated
    /// uses of one cached source do not repeat its invocation. These may overlap an
    /// already-running video encoder and must be reported separately from
    /// that boundary's encode/mux execution order. Native PCM WAV adds none.
    #[must_use]
    pub fn invocations(&self) -> Vec<InvocationReport> {
        self.invocations.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmn_anim::RationalTime;
    use fmn_codec::{SampleFormat, WavAudio, encode_wav};
    use fmn_output::{AudioDecodeLimits, AudioInputFormat, frames_to_samples};
    use fmn_platform::fs::VirtualFs;

    fn config() -> MixerConfig {
        MixerConfig {
            sample_rate: 8,
            channels: 1,
            max_output_frames: 1_000,
        }
    }

    fn request(path: &str, frame: i64, fps: u32) -> SoundRequest {
        SoundRequest {
            sound_file: path.into(),
            time: RationalTime::zero(fps) + frame,
            time_offset: 0.0,
            gain: None,
            gain_to_background: None,
        }
    }

    fn native_decoder() -> AudioDecoder {
        AudioDecoder::new(AudioDecodeLimits::default(), None).unwrap()
    }

    #[test]
    fn skipped_intervals_rebase_call_sites_without_restarting_time() {
        let mut timeline = SoundtrackTimeline::default();
        for (frame, skipping) in [
            (0, true),
            (12, false),
            (20, true),
            (24, true),
            (30, false),
            (36, false),
        ] {
            timeline.observe(frame, skipping).unwrap();
        }
        assert_eq!(timeline.omitted, [(0, 12), (20, 30)]);
        for (scene, output) in [
            (0, 0),
            (11, 0),
            (12, 0),
            (18, 6),
            (20, 8),
            (25, 8),
            (30, 8),
            (36, 14),
        ] {
            assert_eq!(timeline.output_frame(scene).unwrap(), output);
        }
        assert!(timeline.output_frame(-1).is_err());
        assert!(timeline.output_frame(37).is_err());
        assert!(timeline.observe(35, false).is_err());
        assert_eq!(timeline.output_frame(36).unwrap(), 14);
    }

    #[test]
    fn native_sources_are_snapshotted_and_rebased_before_offset_application() {
        let fs = VirtualFs::new();
        let bytes = encode_wav(1, 8, SampleFormat::F32, &[0.125, 0.25, 0.375, 0.5]);
        fs.insert("/tone.wav", bytes.clone());
        let mut timeline = SoundtrackTimeline::default();
        timeline.observe(0, true).unwrap();
        timeline.observe(4, false).unwrap();
        timeline.observe(10, false).unwrap();
        let mut cue = request("/tone.wav", 6, 8);
        cue.time_offset = -0.125;
        let prepared = PreparedSceneSoundtrack::new(
            &fs,
            &[cue],
            &mut native_decoder(),
            config(),
            Some(&timeline),
        )
        .unwrap()
        .with_timeline_frames(6);
        fs.insert("/tone.wav", b"changed after decode".to_vec());
        let mix = prepared.mix(1).unwrap();
        assert_eq!(mix.audio.samples, [0.0, 0.125, 0.25, 0.375, 0.5, 0.0]);
        assert_eq!(prepared.inputs()[0].bytes.as_ref(), bytes.as_slice());
        assert_eq!(prepared.inputs()[0].path, PathBuf::from("/tone.wav"));
        assert_eq!(prepared.inputs()[0].report.format, AudioInputFormat::Wav);
        assert!(prepared.invocations().is_empty());
    }

    #[test]
    fn repeated_cues_share_one_snapshot_without_losing_their_placements_or_gains() {
        let fs = VirtualFs::new();
        fs.insert(
            "/repeat.wav",
            encode_wav(1, 8, SampleFormat::F32, &[0.25, 0.5]),
        );
        let first = request("/repeat.wav", 0, 8);
        let mut second = request("/repeat.wav", 3, 8);
        second.gain = Some(-6.020_599_913_279_624);
        let prepared = PreparedSceneSoundtrack::new(
            &fs,
            &[first, second],
            &mut native_decoder(),
            config(),
            None,
        )
        .unwrap();
        assert_eq!(prepared.inputs().len(), 2);
        assert!(Arc::ptr_eq(
            &prepared.inputs()[0].bytes,
            &prepared.inputs()[1].bytes,
        ));
        assert_eq!(
            prepared.inputs()[0].report.source_digest,
            prepared.inputs()[1].report.source_digest,
        );
        let mixed = prepared.mix(2).unwrap();
        assert_eq!(mixed.cues_mixed, 2);
        assert_eq!(mixed.audio.samples, [0.25, 0.5, 0.0, 0.125, 0.25]);
        assert!(prepared.invocations().is_empty());
    }

    #[test]
    fn rational_windows_preserve_resampling_ducking_and_whole_mix_samples() {
        let fs = VirtualFs::new();
        fs.insert(
            "/background.wav",
            encode_wav(1, 8, SampleFormat::F32, &[0.25; 16]),
        );
        fs.insert(
            "/foreground.wav",
            encode_wav(1, 11, SampleFormat::F32, &[0.125; 11]),
        );
        let background = request("/background.wav", 0, 29);
        let mut foreground = request("/foreground.wav", 4, 29);
        foreground.time_offset = -0.125;
        foreground.gain = Some(-3.0);
        foreground.gain_to_background = Some(-6.0);
        let prepared = PreparedSceneSoundtrack::new(
            &fs,
            &[background, foreground],
            &mut native_decoder(),
            MixerConfig {
                sample_rate: 48,
                ..config()
            },
            None,
        )
        .unwrap()
        .with_timeline_frames(16);
        let whole = prepared.mix(1).unwrap();
        let mut direct = SoundMixer::new(MixerConfig {
            sample_rate: 48,
            ..config()
        })
        .unwrap();
        direct
            .add(SoundCue::new(
                WavAudio {
                    channels: 1,
                    sample_rate: 8,
                    format: SampleFormat::F32,
                    samples: vec![0.25; 16],
                },
                0,
                29,
            ))
            .unwrap();
        direct
            .add(SoundCue {
                audio: WavAudio {
                    channels: 1,
                    sample_rate: 11,
                    format: SampleFormat::F32,
                    samples: vec![0.125; 11],
                },
                frame: 4,
                fps: 29,
                time_offset: -0.125,
                gain: Some(-3.0),
                gain_to_background: Some(-6.0),
            })
            .unwrap();
        assert_eq!(whole.audio.samples, direct.mix(1).unwrap().audio.samples);
        let boundaries =
            [0, 4, 8, 12, 16].map(|frame| frames_to_samples(frame, 29, 48).unwrap() as u64);
        let mut concatenated = Vec::new();
        for range in boundaries.windows(2) {
            let clip = prepared
                .mix_window(range[0], range[1] - range[0], 4)
                .unwrap();
            assert_eq!(clip.audio.samples.len() as u64, range[1] - range[0]);
            concatenated.extend(clip.audio.samples);
        }
        assert_eq!(concatenated, whole.audio.samples[..boundaries[4] as usize]);
        assert_eq!(prepared.inputs().len(), 2);
    }

    #[test]
    fn negative_offsets_clip_source_phase_and_windows_never_extend_to_a_tail() {
        let fs = VirtualFs::new();
        fs.insert(
            "/tone.wav",
            encode_wav(1, 8, SampleFormat::F32, &[0.125, 0.25, 0.375, 0.5]),
        );
        let mut cue = request("/tone.wav", 0, 8);
        cue.time_offset = -0.125;
        let prepared =
            PreparedSceneSoundtrack::new(&fs, &[cue], &mut native_decoder(), config(), None)
                .unwrap()
                .with_timeline_frames(20);
        assert_eq!(
            prepared.mix_window(0, 2, 1).unwrap().audio.samples,
            [0.25, 0.375]
        );
        assert!(
            prepared
                .mix_window(3, 0, 1)
                .unwrap()
                .audio
                .samples
                .is_empty()
        );
        assert_eq!(
            prepared.mix_window(3, 2, 1).unwrap().audio.samples,
            [0.0, 0.0]
        );
    }

    #[test]
    fn resource_and_asset_failures_keep_their_native_error_types() {
        let fs = VirtualFs::new();
        assert!(matches!(
            PreparedSceneSoundtrack::new(&fs, &[request("/missing.wav", 0, 8)], &mut native_decoder(), config(), None),
            Err(RenderError::SoundAsset { path, .. }) if path.as_path() == std::path::Path::new("/missing.wav")
        ));
        fs.insert("/invalid.wav", b"invalid media".to_vec());
        assert!(matches!(
            PreparedSceneSoundtrack::new(&fs, &[request("/invalid.wav", 0, 8)], &mut native_decoder(), config(), None),
            Err(RenderError::SoundDecode { path, .. }) if path.as_path() == std::path::Path::new("/invalid.wav")
        ));
        let prepared =
            PreparedSceneSoundtrack::new(&fs, &[], &mut native_decoder(), config(), None).unwrap();
        assert_eq!(
            prepared.mix_window(3, 2, 1).unwrap().audio.samples,
            [0.0, 0.0]
        );
        assert!(prepared.mix_window(u64::MAX, 1, 1).is_err());
        assert!(prepared.mix_window(999, 2, 1).is_err());
        assert!(prepared.mix(0).is_err());
        assert!(prepared.with_timeline_frames(1_001).mix(1).is_err());
    }
}
