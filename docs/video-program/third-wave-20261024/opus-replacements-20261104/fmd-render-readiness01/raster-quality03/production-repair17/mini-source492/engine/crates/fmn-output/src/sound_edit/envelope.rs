//! Deterministic source-domain envelopes for edited sound cues.

use super::{source_frame_at_seconds, validated_source_frames};
use crate::sound::{SoundCue, SoundError};

impl SoundCue {
    /// Apply linear-amplitude fades to the current source buffer in place.
    ///
    /// Lengths are complete source frames, not scene frames or output samples.
    /// A zero length disables that fade. A one-frame fade silences its endpoint.
    /// For a length `n >= 2`, the fade-in spans gains `0 .. 1` over the first
    /// `n` frames, and the fade-out spans `1 .. 0` over the last `n` frames.
    /// Overlapping fades multiply; their lengths need not sum to the duration.
    ///
    /// All channels in a frame receive the same gain. The envelope is applied
    /// before channel conversion/resampling and before the cue's dB gain.
    /// Placement and duration (including the background-duck interval) do not
    /// change. Trim first to fade the endpoints of a selected source range.
    /// Repeated calls compose on the current PCM, not on the original asset.
    ///
    /// The arithmetic is fixed-order f64 with one final f32 narrowing per
    /// affected sample. Unity regions are left bit-for-bit untouched; silenced
    /// endpoints are canonical positive zero.
    ///
    /// # Errors
    /// Invalid PCM or a fade longer than the current source buffer. Validation
    /// completes before any mutation, so the cue is unchanged on error.
    pub fn fade_source_frames(
        &mut self,
        fade_in: u64,
        fade_out: u64,
    ) -> Result<&mut Self, SoundError> {
        let frames = validated_source_frames(&self.audio)?;
        if fade_in > frames || fade_out > frames {
            return Err(SoundError::InvalidConfig(
                "source fade lengths must not exceed source frames",
            ));
        }
        if fade_in == 0 && fade_out == 0 {
            return Ok(self);
        }
        let channels = usize::from(self.audio.channels);
        for (index, frame) in self.audio.samples.chunks_exact_mut(channels).enumerate() {
            // validated_source_frames established that the entire count fits.
            let index = index as u64;
            let gain = ramp(index, fade_in) * ramp(frames - 1 - index, fade_out);
            if gain == 1.0 {
                continue;
            }
            for sample in frame {
                *sample = if gain == 0.0 {
                    0.0
                } else {
                    (f64::from(*sample) * gain) as f32
                };
            }
        }
        Ok(self)
    }

    /// Apply fades whose durations are quantized to the nearest source frame.
    ///
    /// Durations use the same exact binary-rational, half-up conversion as
    /// [`Self::trim_source_seconds`]. Envelope endpoint and overlap semantics
    /// are those of [`Self::fade_source_frames`].
    ///
    /// # Errors
    /// Negative/non-finite durations, unrepresentable or overlong fades, or
    /// invalid PCM. The cue is unchanged on error.
    pub fn fade_source_seconds(
        &mut self,
        fade_in: f64,
        fade_out: f64,
    ) -> Result<&mut Self, SoundError> {
        let fade_in = source_frame_at_seconds(fade_in, self.audio.sample_rate)?;
        let fade_out = source_frame_at_seconds(fade_out, self.audio.sample_rate)?;
        self.fade_source_frames(fade_in, fade_out)
    }

    /// Builder form of [`Self::fade_source_frames`].
    ///
    /// # Errors
    /// The same validation errors as [`Self::fade_source_frames`].
    pub fn with_source_fades_frames(
        mut self,
        fade_in: u64,
        fade_out: u64,
    ) -> Result<Self, SoundError> {
        self.fade_source_frames(fade_in, fade_out)?;
        Ok(self)
    }

    /// Builder form of [`Self::fade_source_seconds`].
    ///
    /// # Errors
    /// The same validation errors as [`Self::fade_source_seconds`].
    pub fn with_source_fades_seconds(
        mut self,
        fade_in: f64,
        fade_out: f64,
    ) -> Result<Self, SoundError> {
        self.fade_source_seconds(fade_in, fade_out)?;
        Ok(self)
    }
}

fn ramp(distance_from_endpoint: u64, length: u64) -> f64 {
    if length == 0 || distance_from_endpoint >= length {
        1.0
    } else if length == 1 {
        0.0
    } else {
        distance_from_endpoint as f64 / (length - 1) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmn_codec::{SampleFormat, WavAudio};

    fn cue(channels: u16, samples: &[f32]) -> SoundCue {
        SoundCue::new(
            WavAudio {
                channels,
                sample_rate: 8,
                format: SampleFormat::F32,
                samples: samples.to_vec(),
            },
            12,
            24,
        )
    }

    #[test]
    fn stereo_envelopes_preserve_channel_balance_and_metadata() {
        let mut cue = cue(2, &[1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0]);
        cue.gain = Some(-6.0);
        cue.gain_to_background = Some(-20.0);
        cue.time_offset = -0.5;
        let mut expected = cue.clone();
        expected.audio.samples = vec![0.0, 0.0, 0.5, -0.5, 1.0, -1.0, 0.5, -0.5, 0.0, 0.0];
        cue.fade_source_frames(3, 3).unwrap();
        assert_eq!(cue, expected);
        assert_eq!(cue.audio.samples[1].to_bits(), 0);
        assert_eq!(cue.audio.samples[9].to_bits(), 0);
    }

    #[test]
    fn overlapping_fades_multiply_and_repeated_edits_compose() {
        let cue = cue(1, &[1.0; 5])
            .with_source_fades_frames(5, 5)
            .unwrap()
            .with_source_fades_frames(3, 3)
            .unwrap();
        assert_eq!(cue.audio.samples, vec![0.0, 0.09375, 0.25, 0.09375, 0.0]);
    }

    #[test]
    fn no_fade_preserves_bits_and_one_frame_fades_silence_endpoints() {
        let samples = [-0.0, f32::from_bits(1), f32::MAX];
        let no_fade = cue(1, &samples).with_source_fades_frames(0, 0).unwrap();
        assert_eq!(
            no_fade
                .audio
                .samples
                .iter()
                .map(|s| s.to_bits())
                .collect::<Vec<_>>(),
            samples.iter().map(|s| s.to_bits()).collect::<Vec<_>>()
        );
        let faded = cue(1, &samples).with_source_fades_frames(1, 1).unwrap();
        assert_eq!(faded.audio.samples[0].to_bits(), 0);
        assert_eq!(faded.audio.samples[1].to_bits(), 1);
        assert_eq!(faded.audio.samples[2].to_bits(), 0);
        assert!(
            cue(1, &[])
                .with_source_fades_frames(0, 0)
                .unwrap()
                .audio
                .samples
                .is_empty()
        );
        assert!(cue(1, &[]).with_source_fades_frames(1, 0).is_err());
        assert_eq!(
            cue(1, &[-1.0])
                .with_source_fades_frames(1, 1)
                .unwrap()
                .audio
                .samples,
            vec![0.0]
        );
    }

    #[test]
    fn invalid_fades_and_pcm_fail_before_mutation() {
        let mut cue = cue(1, &[1.0; 4]);
        let original = cue.clone();
        for (fade_in, fade_out) in [(5, 0), (0, 5), (u64::MAX, u64::MAX)] {
            assert!(cue.fade_source_frames(fade_in, fade_out).is_err());
            assert_eq!(cue, original);
        }
        for (fade_in, fade_out) in [
            (f64::NAN, 0.0),
            (0.0, f64::INFINITY),
            (-0.1, 0.0),
            (0.0, f64::MAX),
        ] {
            assert!(cue.fade_source_seconds(fade_in, fade_out).is_err());
            assert_eq!(cue, original);
        }
        cue.audio.samples[3] = f32::INFINITY;
        let original = cue.clone();
        assert!(matches!(
            cue.fade_source_frames(4, 0),
            Err(SoundError::NonFiniteSample { index: 3 })
        ));
        assert_eq!(cue, original);
    }

    #[test]
    fn seconds_fades_follow_source_rate_not_scene_fps() {
        let cue = cue(1, &[1.0; 5])
            .with_source_fades_seconds(0.375, 0.375)
            .unwrap();
        assert_eq!(cue.audio.samples, vec![0.0, 0.5, 1.0, 0.5, 0.0]);
    }
}
