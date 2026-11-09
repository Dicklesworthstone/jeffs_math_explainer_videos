//! The score engine's stems must be accepted by franken_manim's own WAV
//! decoder (the one `Scene::add_sound` uses), or a render fails only at its
//! very end. Round-trip every stem format through `fmn_codec::wav::decode_wav`.

use explainer_score::wav::{self, Format};
use explainer_score::{SAMPLE_RATE, Stereo};
use fmn_codec::wav::{WavLimits, decode_wav};

fn stem(n: usize) -> Stereo {
    Stereo {
        l: (0..n).map(|i| 0.6 * ((i as f64) * 0.013).sin()).collect(),
        r: (0..n).map(|i| -0.4 * ((i as f64) * 0.007).cos()).collect(),
    }
}

fn tmp(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("explainer_kit_stems_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

#[test]
fn every_stem_format_decodes_in_franken_manim_with_the_same_samples() {
    let audio = stem(9_600);
    for (fmt, tol) in [
        (Format::F32, 0.0),
        (Format::S24, 2.0 / 8_388_607.0),
        (Format::S16Tpdf { seed: 7 }, 3.0 / 32_767.0),
    ] {
        let path = tmp(&format!("{fmt:?}.wav").replace(['{', '}', ' ', ':'], "_"));
        wav::write(&path, &audio, fmt).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        let decoded = decode_wav(&bytes, &WavLimits::default()).unwrap_or_else(|e| panic!("{fmt:?}: {e:?}"));
        assert_eq!(decoded.channels, 2, "{fmt:?}");
        assert_eq!(decoded.sample_rate, SAMPLE_RATE, "{fmt:?}");
        assert_eq!(decoded.samples.len(), 2 * audio.len(), "{fmt:?}");
        for i in 0..audio.len() {
            let (l, r) = (f64::from(decoded.samples[2 * i]), f64::from(decoded.samples[2 * i + 1]));
            // f32 must be bit-exact to the f32 we wrote
            let want_l = if tol == 0.0 {
                f64::from(audio.l[i] as f32)
            } else {
                audio.l[i]
            };
            let want_r = if tol == 0.0 {
                f64::from(audio.r[i] as f32)
            } else {
                audio.r[i]
            };
            assert!((l - want_l).abs() <= tol + 1e-7, "{fmt:?} L[{i}]: {l} vs {want_l}");
            assert!((r - want_r).abs() <= tol + 1e-7, "{fmt:?} R[{i}]: {r} vs {want_r}");
        }
    }
}
