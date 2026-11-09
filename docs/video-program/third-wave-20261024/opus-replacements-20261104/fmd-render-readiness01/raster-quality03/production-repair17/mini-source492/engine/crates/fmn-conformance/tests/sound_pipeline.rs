//! Scene-to-Reel sound integration (fm-0m7, BN-02, BN-14).
//!
//! Proscenium records exact rational call-site time without depending upward
//! on Reel. The composition root resolves the asset, maps the request to a
//! `SoundCue`, and Reel places it on the sample grid. This test locks that
//! adapter seam and the scalar/compiled × `{1,4,16}` certified-WAV equivalence
//! in one corpus.

use fmn_codec::{SampleFormat, WavAudio};
use fmn_output::{DitherPolicy, MixKernel, MixerConfig, SoundCue, SoundMixer};
use fmn_scene::{NullSceneSink, RuntimeConfig, Scene, SoundRequest};

fn cue_from_request(request: &SoundRequest, audio: WavAudio) -> SoundCue {
    SoundCue {
        audio,
        frame: request.time.frames(),
        fps: request.time.fps(),
        time_offset: request.time_offset,
        gain: request.gain,
        gain_to_background: request.gain_to_background,
    }
}

#[test]
fn frame_time_becomes_sample_exact_and_certified_wav_is_thread_invariant() {
    let mut scene = Scene::new(
        RuntimeConfig {
            fps: 30,
            ..RuntimeConfig::default()
        },
        91,
    )
    .expect("scene");
    scene
        .wait(Some(1.0), &mut NullSceneSink)
        .expect("advance exactly 30 frames");
    scene
        .add_sound("click.wav", 0.0, None, None)
        .expect("queue click");
    let request = scene.sound_requests().first().expect("sound request");
    assert_eq!(request.time.frames(), 30);
    assert_eq!(request.time.fps(), 30);

    let mut click_samples = vec![0.0; 17];
    click_samples[0] = 1.0;
    let click = WavAudio {
        channels: 1,
        sample_rate: 48_000,
        format: SampleFormat::S16,
        samples: click_samples,
    };
    let mut mixer = SoundMixer::new(MixerConfig {
        sample_rate: 48_000,
        channels: 1,
        max_output_frames: 48_017,
    })
    .expect("mixer");
    mixer
        .add(cue_from_request(request, click))
        .expect("composition-root adapter");

    let scalar = mixer
        .mix_with_kernel(1, MixKernel::Scalar)
        .expect("scalar oracle");
    let click_sample = 48_000;
    assert_eq!(scalar.audio.samples.len(), click_sample + 17);
    assert!(
        scalar.audio.samples[..click_sample]
            .iter()
            .all(|&sample| sample == 0.0)
    );
    assert_eq!(scalar.audio.samples[click_sample], 1.0);
    assert!(
        scalar.audio.samples[click_sample + 1..]
            .iter()
            .all(|&sample| sample == 0.0)
    );

    let wav = |report: &fmn_output::MixReport| {
        report
            .wav_bytes(SampleFormat::S16, DitherPolicy::None)
            .expect("certified WAV")
    };
    for &kernel in MixKernel::ALL {
        for threads in [1, 4, 16] {
            let report = mixer
                .mix_with_kernel(threads, kernel)
                .expect("kernel/thread matrix");
            assert_eq!(wav(&report), wav(&scalar));
        }
    }
}
