//! Source-domain edits for sound cues, before channel mapping and resampling.
//!
//! These inherent methods keep placement and gain metadata intact. A trim
//! changes the cue's active interval, including background ducking, rather
//! than merely replacing unwanted samples with silence.

use fmn_codec::WavAudio;

use crate::sound::{SoundCue, SoundError};

mod envelope;

impl SoundCue {
    /// Keep the half-open source-frame range `[start, end)`.
    ///
    /// Frames count complete interleaved frames at the source sample rate,
    /// not scene frames or output samples. `None` keeps the remaining tail.
    /// Empty ranges, including `[length, length)`, are legal. Bounds are strict:
    /// reversed or out-of-buffer ranges are errors, never silently clamped.
    ///
    /// Placement (`frame`, `fps`, and `time_offset`) and gains are unchanged:
    /// the selected audio starts at the cue's original timeline position.
    /// Repeated edits address the current buffer, not the original asset.
    ///
    /// Validation completes before mutation. Retained samples are moved in
    /// place, without allocating a second PCM buffer.
    ///
    /// # Errors
    /// Invalid PCM or a range outside the current source buffer. The cue is
    /// unchanged on error, including when invalid PCM lies outside the range.
    pub fn trim_source_frames(
        &mut self,
        start: u64,
        end: Option<u64>,
    ) -> Result<&mut Self, SoundError> {
        let frames = validated_source_frames(&self.audio)?;
        let end = end.unwrap_or(frames);
        if start > end || end > frames {
            return Err(SoundError::InvalidConfig(
                "source trim must satisfy start <= end <= source frames",
            ));
        }
        let channels = usize::from(self.audio.channels);
        let first = usize::try_from(start).map_err(|_| SoundError::SampleCountOverflow)? * channels;
        let last = usize::try_from(end).map_err(|_| SoundError::SampleCountOverflow)? * channels;
        self.audio.samples.copy_within(first..last, 0);
        self.audio.samples.truncate(last - first);
        Ok(self)
    }

    /// Keep a source-time interval, quantized to the nearest source frame.
    ///
    /// Each finite, nonnegative endpoint is converted from its exact `f64`
    /// value with half-frame ties rounded upward. The conversion does not
    /// multiply in floating point, avoiding double rounding at boundaries.
    /// `None` means the current source end. Bounds are checked on the quantized
    /// frame grid; reversed time intervals are rejected even if both endpoints
    /// would round to the same frame. Other semantics match
    /// [`Self::trim_source_frames`].
    ///
    /// # Errors
    /// Invalid PCM, negative/non-finite endpoints, reversed or out-of-buffer
    /// ranges, or endpoints beyond the representable source-frame range.
    /// The cue is unchanged on error.
    pub fn trim_source_seconds(
        &mut self,
        start: f64,
        end: Option<f64>,
    ) -> Result<&mut Self, SoundError> {
        if end.is_some_and(|end| end < start) {
            return Err(SoundError::InvalidConfig(
                "source trim end must not precede start",
            ));
        }
        let start = source_frame_at_seconds(start, self.audio.sample_rate)?;
        let end = end
            .map(|end| source_frame_at_seconds(end, self.audio.sample_rate))
            .transpose()?;
        self.trim_source_frames(start, end)
    }

    /// Builder form of [`Self::trim_source_frames`].
    ///
    /// # Errors
    /// The same validation errors as [`Self::trim_source_frames`].
    pub fn with_source_frames(mut self, start: u64, end: Option<u64>) -> Result<Self, SoundError> {
        self.trim_source_frames(start, end)?;
        Ok(self)
    }

    /// Builder form of [`Self::trim_source_seconds`].
    ///
    /// # Errors
    /// The same validation errors as [`Self::trim_source_seconds`].
    pub fn with_source_seconds(mut self, start: f64, end: Option<f64>) -> Result<Self, SoundError> {
        self.trim_source_seconds(start, end)?;
        Ok(self)
    }
}

fn validated_source_frames(audio: &WavAudio) -> Result<u64, SoundError> {
    if audio.sample_rate == 0 {
        return Err(SoundError::InvalidConfig(
            "source sample_rate must be nonzero",
        ));
    }
    if !matches!(audio.channels, 1 | 2) {
        return Err(SoundError::UnsupportedChannels {
            channels: audio.channels,
        });
    }
    let channels = usize::from(audio.channels);
    if !audio.samples.len().is_multiple_of(channels) {
        return Err(SoundError::MisalignedSamples {
            channels: audio.channels,
            samples: audio.samples.len(),
        });
    }
    if let Some(index) = audio.samples.iter().position(|sample| !sample.is_finite()) {
        return Err(SoundError::NonFiniteSample { index });
    }
    u64::try_from(audio.samples.len() / channels).map_err(|_| SoundError::SampleCountOverflow)
}

// A finite nonnegative f64 is an integer significand times a power of two.
// The significand-rate product needs at most 85 bits, so u128 is sufficient.
fn source_frame_at_seconds(seconds: f64, rate: u32) -> Result<u64, SoundError> {
    if rate == 0 || !seconds.is_finite() || seconds < 0.0 {
        return Err(SoundError::InvalidConfig(
            "source times must be finite and nonnegative, with a nonzero sample rate",
        ));
    }
    if seconds == 0.0 {
        return Ok(0);
    }
    let bits = seconds.to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as i32;
    let fraction = bits & ((1_u64 << 52) - 1);
    let (significand, power) = if exponent == 0 {
        (fraction, -1074)
    } else {
        (fraction | (1_u64 << 52), exponent - 1023 - 52)
    };
    let product = u128::from(significand) * u128::from(rate);
    let frames = if power >= 0 {
        let shift = power as u32;
        // checked_shl alone only checks the shift count, not discarded bits.
        if shift >= 64 || product > (u128::from(u64::MAX) >> shift) {
            return Err(SoundError::SampleCountOverflow);
        }
        product << shift
    } else {
        let shift = (-power) as u32;
        if shift >= 128 {
            0
        } else {
            let whole = product >> shift;
            let remainder = product & ((1_u128 << shift) - 1);
            whole + u128::from(remainder >= (1_u128 << (shift - 1)))
        }
    };
    u64::try_from(frames).map_err(|_| SoundError::SampleCountOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmn_codec::SampleFormat;

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
    fn stereo_trim_preserves_complete_frames_and_placement() {
        let mut cue = cue(2, &[0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7]);
        cue.time_offset = -0.25;
        cue.gain = Some(-3.0);
        cue.gain_to_background = Some(-6.0);
        let mut expected = cue.clone();
        expected.audio.samples = vec![0.2, 0.3, 0.4, 0.5];
        cue.trim_source_frames(1, Some(3)).unwrap();
        assert_eq!(cue, expected);
    }

    #[test]
    fn open_end_and_repeated_trims_address_the_current_buffer() {
        let cue = cue(1, &[0.0, 0.25, 0.5, 0.75])
            .with_source_frames(1, None)
            .unwrap()
            .with_source_frames(1, Some(2))
            .unwrap();
        assert_eq!(cue.audio.samples, vec![0.5]);
    }

    #[test]
    fn empty_ranges_and_empty_sources_are_legal() {
        for (start, end) in [(0, 0), (1, 1), (2, 2)] {
            let cue = cue(2, &[0.0, 0.25, 0.5, 0.75])
                .with_source_frames(start, Some(end))
                .unwrap();
            assert!(cue.audio.samples.is_empty());
        }
        let cue = cue(1, &[]).with_source_frames(0, None).unwrap();
        assert!(cue.audio.samples.is_empty());
    }

    #[test]
    fn rejected_ranges_do_not_mutate_the_cue() {
        let mut cue = cue(1, &[0.0, 0.25, 0.5]);
        let original = cue.clone();
        for (start, end) in [(2, Some(1)), (0, Some(4)), (4, None), (u64::MAX, None)] {
            assert!(cue.trim_source_frames(start, end).is_err());
            assert_eq!(cue, original);
        }
        for (start, end) in [
            (-1.0, None),
            (f64::NAN, None),
            (0.0, Some(f64::INFINITY)),
            (0.01, Some(0.001)),
            (f64::MAX, None),
        ] {
            assert!(cue.trim_source_seconds(start, end).is_err());
            assert_eq!(cue, original);
        }
    }

    #[test]
    fn malformed_pcm_is_not_sanitized_by_trimming_it_away() {
        let mut nonfinite = cue(1, &[f32::INFINITY, 0.5]);
        let original = nonfinite.clone();
        assert_eq!(
            nonfinite.trim_source_frames(1, Some(2)).unwrap_err(),
            SoundError::NonFiniteSample { index: 0 }
        );
        assert_eq!(nonfinite, original);
        let mut incomplete = cue(2, &[0.0, 0.5, 0.25]);
        assert!(matches!(
            incomplete.trim_source_frames(0, Some(1)),
            Err(SoundError::MisalignedSamples { .. })
        ));
        let mut zero_rate = cue(1, &[0.0]);
        zero_rate.audio.sample_rate = 0;
        assert!(zero_rate.trim_source_frames(0, None).is_err());
        for channels in [0, 3] {
            assert!(matches!(
                cue(channels, &[]).trim_source_frames(0, None),
                Err(SoundError::UnsupportedChannels { .. })
            ));
        }
    }

    #[test]
    fn seconds_trim_uses_source_rate_and_half_up_endpoints() {
        let cue = cue(1, &[0.0, 0.25, 0.5, 0.75])
            .with_source_seconds(0.0625, Some(0.3125))
            .unwrap();
        assert_eq!(cue.audio.samples, vec![0.25, 0.5]);
    }

    #[test]
    fn exact_time_conversion_handles_ties_subnormals_and_overflow() {
        assert_eq!(source_frame_at_seconds(-0.0, 48_000), Ok(0));
        assert_eq!(source_frame_at_seconds(f64::from_bits(1), u32::MAX), Ok(0));
        assert_eq!(source_frame_at_seconds(0.0625, 8), Ok(1));
        assert_eq!(
            source_frame_at_seconds(f64::from_bits(0.0625_f64.to_bits() - 1), 8),
            Ok(0)
        );
        // f64 multiplication rounds this to 0.5; its exact value is smaller.
        assert_eq!(source_frame_at_seconds(1.0 / 6.0, 3), Ok(0));
        assert_eq!(
            source_frame_at_seconds(18_446_744_073_709_551_616.0, 1),
            Err(SoundError::SampleCountOverflow)
        );
        assert_eq!(
            source_frame_at_seconds(f64::MAX, u32::MAX),
            Err(SoundError::SampleCountOverflow)
        );
        assert!(source_frame_at_seconds(1.0, 0).is_err());
    }
}
