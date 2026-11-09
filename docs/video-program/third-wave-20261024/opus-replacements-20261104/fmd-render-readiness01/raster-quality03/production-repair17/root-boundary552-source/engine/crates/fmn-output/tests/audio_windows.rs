//! Exact audio segments reuse Reel's placement, resampler and ordered kernels.
use fmn_codec::{SampleFormat, WavAudio};
use fmn_output::{MixKernel, MixerConfig, SoundCue, SoundError, SoundMixer, frames_to_samples};

fn audio(frames: usize, rate: u32, channels: u16, scale: f32) -> WavAudio {
    WavAudio {
        channels,
        sample_rate: rate,
        format: SampleFormat::F32,
        samples: (0..frames * usize::from(channels))
            .map(|i| ((i % 101) as f32 - 50.0) * scale)
            .collect(),
    }
}

fn timeline() -> SoundMixer {
    let mut mixer = SoundMixer::new(MixerConfig::default())
        .unwrap()
        .with_timeline_frames(12_000);
    mixer
        .add(SoundCue::new(audio(8_000, 44_100, 1, 0.008), 0, 29))
        .unwrap();
    let mut cue = SoundCue::new(audio(5_000, 48_000, 2, 0.01), 2, 29);
    cue.time_offset = -0.017;
    cue.gain = Some(4.0);
    cue.gain_to_background = Some(-6.0);
    mixer.add(cue).unwrap();
    let mut cue = SoundCue::new(audio(3_000, 32_000, 2, 0.015), 4, 29);
    cue.gain = Some(6.0);
    mixer.add(cue).unwrap();
    mixer
}

#[test]
fn every_window_matches_the_scalar_whole_with_resampling_gain_and_ducking() {
    let mixer = timeline();
    let expected = mixer.mix_with_kernel(1, MixKernel::Scalar).unwrap();
    for kernel in [MixKernel::Scalar, MixKernel::Compiled] {
        for threads in [1, 4, 16] {
            for (start, count) in [
                (0, 1),
                (1, 3),
                (33, 4097),
                (3200, 7000),
                (11_999, 1),
                (12_000, 0),
            ] {
                let actual = mixer
                    .mix_window_with_kernel(start, count, threads, kernel)
                    .unwrap();
                let range = (start as usize * 2)..((start + count) as usize * 2);
                assert_eq!(actual.audio.samples, expected.audio.samples[range]);
                assert_eq!(actual.cues_mixed, expected.cues_mixed);
                assert!(actual.workers_used <= threads);
            }
        }
    }
}

#[test]
fn adjacent_rational_windows_do_not_accumulate_rounding_error() {
    let mixer = timeline();
    let expected = mixer.mix(1).unwrap();
    let boundaries: Vec<u64> = (0..8)
        .map(|frame| frames_to_samples(frame, 29, 48_000).unwrap() as u64)
        .collect();
    let mut samples = Vec::new();
    for pair in boundaries.windows(2) {
        samples.extend(
            mixer
                .mix_window(pair[0], pair[1] - pair[0], 4)
                .unwrap()
                .audio
                .samples,
        );
    }
    assert_eq!(
        samples,
        expected.audio.samples[..*boundaries.last().unwrap() as usize * 2]
    );
    assert_ne!(boundaries[1] * 7, boundaries[7]);
}

#[test]
fn window_padding_and_zero_length_are_explicit_not_cue_tail_extension() {
    let mut mixer = SoundMixer::new(MixerConfig::default())
        .unwrap()
        .with_timeline_frames(20_000);
    mixer
        .add(SoundCue::new(audio(100, 48_000, 1, 0.01), 0, 1))
        .unwrap();
    let result = mixer.mix_window(95, 20, 4).unwrap();
    assert_eq!(result.audio.samples.len(), 40);
    assert!(
        result.audio.samples[10..]
            .iter()
            .all(|sample| *sample == 0.0)
    );
    let empty = mixer.mix_window(100, 0, 1).unwrap();
    assert!(empty.audio.samples.is_empty());
    assert_eq!(empty.workers_used, 0);
    assert_eq!(empty.clipped_samples, 0);
}

#[test]
fn distant_window_does_not_allocate_its_elapsed_silence() {
    let mixer = SoundMixer::new(MixerConfig {
        max_output_frames: 1_000_000_100,
        ..MixerConfig::default()
    })
    .unwrap();
    let result = mixer.mix_window(1_000_000_000, 13, 4).unwrap();
    assert_eq!(result.audio.samples, vec![0.0; 26]);
}

#[test]
fn clipping_receipt_counts_only_samples_inside_the_window() {
    let mut mixer = SoundMixer::new(MixerConfig::default()).unwrap();
    let mut cue = SoundCue::new(audio(10, 48_000, 1, 0.5), 0, 1);
    cue.time_offset = -3.0 / 48_000.0;
    mixer.add(cue).unwrap();
    assert_eq!(mixer.mix_window(0, 2, 1).unwrap().clipped_samples, 4);
    assert_eq!(mixer.mix_window(7, 2, 1).unwrap().clipped_samples, 0);
}

#[test]
fn oversized_overflowing_and_invalid_worker_windows_fail_before_allocation() {
    let mixer = SoundMixer::new(MixerConfig {
        max_output_frames: 100,
        ..MixerConfig::default()
    })
    .unwrap();
    assert!(matches!(
        mixer.mix_window(90, 11, 1),
        Err(SoundError::OutputTooLong { .. })
    ));
    assert!(matches!(
        mixer.mix_window(u64::MAX, 1, 1),
        Err(SoundError::SampleCountOverflow)
    ));
    assert!(matches!(
        mixer.mix_window(0, 1, 0),
        Err(SoundError::ZeroThreads)
    ));
    assert!(matches!(
        mixer.mix_window(0, 1, usize::MAX),
        Err(SoundError::TooManyThreads { .. })
    ));
}
