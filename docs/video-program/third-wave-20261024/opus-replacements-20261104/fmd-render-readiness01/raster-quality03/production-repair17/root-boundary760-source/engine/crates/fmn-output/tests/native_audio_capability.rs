//! The native-codec front door must compile and work without process support.

use fmn_codec::{SampleFormat, encode_wav};
use fmn_output::{AudioDecodeError, AudioDecodeLimits, AudioDecoder, AudioInputFormat};

#[test]
fn native_only_wav_preserves_pcm_and_identity() {
    let mut decoder = AudioDecoder::new(AudioDecodeLimits::default(), None).unwrap();
    let samples = [0.25_f32, -0.125, 0.0, 0.75];
    let pcm16 = decoder
        .decode(&encode_wav(2, 32_000, SampleFormat::S16, &samples))
        .unwrap();
    let pcm32 = decoder
        .decode(&encode_wav(2, 32_000, SampleFormat::F32, &samples))
        .unwrap();
    assert_eq!(pcm16.audio.samples, samples);
    assert_eq!(pcm32.audio.samples, samples);
    assert_eq!(pcm32.audio.channels, 2);
    assert_eq!(pcm32.audio.sample_rate, 32_000);
    assert_eq!(pcm16.report.pcm_digest, pcm32.report.pcm_digest);
    assert_ne!(pcm16.report.source_digest, pcm32.report.source_digest);
    assert!(pcm16.report.invocation.is_none());
    assert!(pcm32.report.invocation.is_none());
}

#[test]
fn native_only_compressed_input_names_the_required_capability() {
    let mut decoder = AudioDecoder::new(AudioDecodeLimits::default(), None).unwrap();
    let error = decoder.decode(b"fLaC12345678").unwrap_err();
    assert!(error.to_string().contains("PCM WAV"));
    assert!(matches!(
        error,
        AudioDecodeError::TranscoderRequired {
            format: AudioInputFormat::Flac
        }
    ));
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "exact-process")))]
#[test]
fn unavailable_host_runner_does_not_disable_native_wav() {
    let mut decoder =
        AudioDecoder::new(AudioDecodeLimits::default(), Some("/missing/ffmpeg".into())).unwrap();
    let error = decoder.decode(b"fLaC12345678").unwrap_err();
    assert!(error.to_string().contains("exact-process"));
    assert!(matches!(error, AudioDecodeError::Capability(_)));
    let result = decoder
        .decode(&encode_wav(1, 48_000, SampleFormat::F32, &[0.25]))
        .unwrap();
    assert_eq!(result.audio.samples, [0.25]);
    assert!(result.report.invocation.is_none());
}

#[test]
fn native_pcm_budget_is_enforced_without_external_fallback() {
    let mut decoder = AudioDecoder::new(
        AudioDecodeLimits {
            max_input_bytes: 4096,
            max_samples: 1,
        },
        None,
    )
    .unwrap();
    let wav = encode_wav(1, 48_000, SampleFormat::F32, &[0.25; 2]);
    assert!(matches!(
        decoder.decode(&wav),
        Err(AudioDecodeError::Native(
            fmn_codec::WavError::TooLarge { .. }
        ))
    ));
}
