//! Exact colour through the delivery encoder (bead mx-e-w1-4qa.14): flat
//! frames rendered losslessly (x264 CRF 0) decode to the BT.709
//! limited-range codes of their colours, the corpus background and the
//! Hoeffding yellow. Planted negative: fmn's own encoder, whose raw input
//! declares no colour space, moves the background's Cb by a whole code.
//! Needs ffmpeg on PATH.

use std::path::{Path, PathBuf};

use explainer_kit::encode::{self, Quality};
use explainer_kit::{BACKGROUND, hex, rect};
use fmn::prelude::*;
use fmn::rendering::VideoQuality;

/// The whole frame in one colour: the background alone, or a fill over it.
struct Flat {
    fill: Option<&'static str>,
}

impl SceneConstruct for Flat {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        if let Some(c) = self.fill {
            let r = rect(
                [-10.0, -6.0, 0.0],
                20.0,
                12.0,
                Style::default().fill(hex(c), 1.0).stroke(hex(c), 0.0, 1.0),
            )?;
            let r = stage.arena_mut().add(r);
            stage.add_to_scene(r)?;
        }
        stage.wait(0.1)?;
        Ok(())
    }
}

/// BT.709 limited-range 8-bit codes (Y, Cb, Cr) of an sRGB colour, unrounded.
fn bt709(c: &str) -> [f64; 3] {
    let v = u32::from_str_radix(c.trim_start_matches('#'), 16).unwrap();
    let [r, g, b] = [(v >> 16) & 255, (v >> 8) & 255, v & 255].map(|x| f64::from(x) / 255.0);
    let y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    [
        16.0 + 219.0 * y,
        128.0 + 224.0 * (b - y) / 1.8556,
        128.0 + 224.0 * (r - y) / 1.5748,
    ]
}

/// Render `fill` losslessly, through the kit's delivery encoder or fmn's
/// own; the mean decoded codes.
fn codes(dir: &Path, name: &str, fill: Option<&'static str>, kit_encoder: bool) -> [f64; 3] {
    let out: PathBuf = dir.join(format!("{name}.mp4"));
    let (r, _) = encode::retrying(name, || {
        let mut o = if kit_encoder {
            encode::delivery_options(
                &out,
                RenderFormat::Mp4,
                Quality {
                    quality: 0,
                    preset: "ultrafast",
                    ..Quality::delivery()
                },
            )
            .unwrap()
        } else {
            let mut o = RenderOptions::with_format(&out, RenderFormat::Mp4).unwrap();
            o.video_quality = VideoQuality {
                crf: Some(0),
                ..VideoQuality::default()
            };
            o
        };
        o.config.camera.resolution = (64, 36);
        o.config.camera.fps = 60;
        o.config.camera.background_color = BACKGROUND.to_owned();
        fmn::render(&mut Flat { fill }, o)
    });
    r.unwrap_or_else(|e| panic!("{name}: {e}"));
    let raw = std::process::Command::new("ffmpeg")
        .args(["-nostdin", "-v", "error", "-i"])
        .arg(&out)
        .args([
            "-vf",
            "select=eq(n\\,3)",
            "-frames:v",
            "1",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "yuv420p",
            "-",
        ])
        .output()
        .unwrap();
    assert!(raw.status.success(), "{}", String::from_utf8_lossy(&raw.stderr));
    let (y, c) = raw.stdout.split_at(64 * 36);
    let (u, v) = c.split_at(32 * 18);
    let mean = |p: &[u8]| p.iter().map(|&x| f64::from(x)).sum::<f64>() / p.len() as f64;
    [mean(y), mean(u), mean(&v[..32 * 18])]
}

#[test]
fn the_delivery_encoder_writes_the_corpus_colours_in_exact_8_bit_codes() {
    assert!(
        std::process::Command::new("ffmpeg")
            .arg("-version")
            .output()
            .is_ok_and(|o| o.status.success()),
        "this test needs ffmpeg on PATH"
    );
    let dir = std::env::temp_dir().join(format!("explainer_kit_colour_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for (name, fill, colour) in [("background", None, BACKGROUND), ("yellow", Some("#FFD866"), "#FFD866")] {
        let want = bt709(colour);
        let got = codes(&dir, &format!("{name}_kit"), fill, true);
        let plain = codes(&dir, &format!("{name}_fmn"), fill, false);
        eprintln!("{name} {colour}: exact {want:.2?}  kit encoder {got:.2?}  fmn's own {plain:.2?}");
        if name == "background" {
            assert!(
                (plain[1] - want[1]).abs() >= 1.0,
                "the untagged input moved Cb (planted negative): {plain:.2?}"
            );
        }
        for k in 0..3 {
            assert!(
                (got[k] - want[k]).abs() <= 0.5,
                "{name} {colour}: component {k} is {:.2}, BT.709 says {:.2}",
                got[k],
                want[k]
            );
        }
    }
}
