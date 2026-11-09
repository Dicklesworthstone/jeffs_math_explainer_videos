//! Source edits exercised through the production mixer and WAV codec.

use fmn_codec::{SampleFormat, WavAudio, WavLimits, decode_wav};
use fmn_output::{DitherPolicy, MixKernel, MixerConfig, SoundCue, SoundError, SoundMixer};

fn cue(channels: u16, rate: u32, frame: i64, samples: &[f32]) -> SoundCue {
    SoundCue::new(
        WavAudio {
            channels,
            sample_rate: rate,
            format: SampleFormat::F32,
            samples: samples.to_vec(),
        },
        frame,
        8,
    )
}

fn mixer(channels: u16, rate: u32) -> SoundMixer {
    SoundMixer::new(MixerConfig {
        channels,
        sample_rate: rate,
        max_output_frames: 64,
    })
    .unwrap()
}

fn bits(samples: &[f32]) -> Vec<u32> {
    samples.iter().map(|sample| sample.to_bits()).collect()
}

#[test]
fn trimmed_cues_only_duck_the_retained_interval() {
    let mut mix = mixer(1, 8);
    mix.add(cue(1, 8, 0, &[0.5; 8])).unwrap();
    let mut foreground = cue(1, 8, 2, &[0.9, 0.0, 0.0, 0.9])
        .with_source_frames(1, Some(3))
        .unwrap();
    foreground.gain_to_background = Some(-20.0);
    mix.add(foreground).unwrap();
    for kernel in MixKernel::ALL {
        for threads in [1, 3, 8] {
            let result = mix.mix_with_kernel(threads, *kernel).unwrap();
            assert_eq!(result.audio.samples.len(), 8);
            assert_eq!(result.cues_mixed, 2);
            for (index, &sample) in result.audio.samples.iter().enumerate() {
                let expected = if (2..4).contains(&index) { 0.05 } else { 0.5 };
                assert!((sample - expected).abs() < 1e-7);
            }
        }
    }
}

#[test]
fn source_trim_and_negative_timeline_clipping_are_distinct_operations() {
    let edited = cue(1, 8, -1, &[0.1, 0.2, 0.3, 0.4, 0.5, 0.6])
        .with_source_frames(1, Some(5))
        .unwrap();
    let mut mix = mixer(1, 8);
    mix.add(edited).unwrap();
    assert_eq!(mix.mix(2).unwrap().audio.samples, vec![0.3, 0.4, 0.5]);
}

#[test]
fn empty_trim_does_not_extend_or_duck_the_timeline() {
    let mut mix = mixer(1, 8).with_timeline_frames(4);
    mix.add(cue(1, 8, 0, &[0.5; 4])).unwrap();
    let mut empty = cue(1, 8, 0, &[0.9; 4])
        .with_source_frames(2, Some(2))
        .unwrap();
    empty.gain_to_background = Some(-20.0);
    mix.add(empty.clone()).unwrap();
    empty.frame = 1_000;
    mix.add(empty).unwrap();
    let report = mix.mix(2).unwrap();
    assert_eq!(report.audio.samples, vec![0.5; 4]);
    assert_eq!(report.cues_mixed, 3);
}

#[test]
fn trim_is_applied_before_the_prepared_cue_resource_budget() {
    let full = cue(1, 8, 0, &[0.25; 100]);
    let mut mix = mixer(1, 8);
    assert!(matches!(
        mix.add(full.clone()),
        Err(SoundError::OutputTooLong { .. })
    ));
    assert!(mix.is_empty());
    mix.add(full.with_source_frames(20, Some(24)).unwrap())
        .unwrap();
    assert_eq!(mix.mix(1).unwrap().audio.samples, vec![0.25; 4]);
}

#[test]
fn edited_stereo_resamples_identically_across_kernels_workers_and_windows() {
    let original = cue(
        2,
        8,
        1,
        &[
            0.1, -0.2, 0.3, -0.4, 0.5, -0.6, 0.7, -0.8, 0.6, -0.5, 0.4, -0.3, 0.2, -0.1, 0.05,
            -0.05,
        ],
    );
    let mut expected = original.clone();
    expected.audio.samples = vec![0.0, 0.0, 0.7, -0.8, 0.6, -0.5, 0.0, 0.0];
    let actual = original
        .with_source_frames(2, Some(6))
        .unwrap()
        .with_source_fades_frames(2, 2)
        .unwrap();
    assert_eq!(bits(&actual.audio.samples), bits(&expected.audio.samples));

    let mut reference = mixer(1, 12).with_timeline_frames(12);
    reference.add(expected).unwrap();
    let reference = reference.mix_with_kernel(1, MixKernel::Scalar).unwrap();
    let mut mix = mixer(1, 12).with_timeline_frames(12);
    mix.add(actual).unwrap();
    for kernel in MixKernel::ALL {
        for threads in [1, 2, 7] {
            let result = mix.mix_with_kernel(threads, *kernel).unwrap();
            assert_eq!(bits(&result.audio.samples), bits(&reference.audio.samples));
            let left = mix.mix_window_with_kernel(0, 5, threads, *kernel).unwrap();
            let right = mix.mix_window_with_kernel(5, 7, threads, *kernel).unwrap();
            let mut joined = left.audio.samples;
            joined.extend(right.audio.samples);
            assert_eq!(bits(&joined), bits(&reference.audio.samples));
        }
    }

    let wav = reference
        .wav_bytes(SampleFormat::F32, DitherPolicy::None)
        .unwrap();
    let decoded = decode_wav(&wav, &WavLimits::default()).unwrap();
    assert_eq!(decoded.channels, 1);
    assert_eq!(decoded.sample_rate, 12);
    assert_eq!(bits(&decoded.samples), bits(&reference.audio.samples));
}

#[test]
fn complementary_source_fades_make_a_continuous_overlap() {
    let outgoing = cue(1, 8, 0, &[1.0; 5])
        .with_source_fades_frames(0, 5)
        .unwrap();
    let incoming = cue(1, 8, 0, &[1.0; 5])
        .with_source_fades_frames(5, 0)
        .unwrap();
    let mut mix = mixer(2, 8).with_timeline_frames(7);
    mix.add(outgoing).unwrap();
    mix.add(incoming).unwrap();
    let report = mix.mix(3).unwrap();
    assert_eq!(&report.audio.samples[..10], &[1.0; 10]);
    assert_eq!(&report.audio.samples[10..], &[0.0; 4]);
    assert_eq!(report.clipped_samples, 0);
}
