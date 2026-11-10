//! Spike S-5 (plan §6.6): franken_manim's camera route at 4K60 to MP4 with
//! audio, through the kit's camera wrapper; route parity; 8- vs 10-bit.
//!
//! ```text
//! cargo run --release -p explainer_kit --example spike_s5_camera -- OUT_DIR [WxH] [SECONDS]
//! ```
//!
//! Writes into OUT_DIR (created; it must not exist):
//! - `orbit.mp4`: a SECONDS-long orbit (default 10) at WxH (default
//!   3840x2160) and 60 fps around a lit 44×11 bar field (Video A's
//!   likelihood landscape), in three arcs of 120° (each under π: the rig
//!   interpolates quaternions linearly), translucent slice planes added
//!   last, and one sound cue; x264 CRF 16 through `delivery_options`, so the
//!   argv is the retained route's;
//! - `still_8bit.mp4`, `still_10bit.mp4`: 1 s of the field from the orbit's
//!   first angle, on the NV12 and the P010 wire, for the banding comparison;
//! - `parity_retained/`, `parity_camera/`: one frame of a 2D composition on
//!   each route (the camera at its default, straight-down orientation);
//! - one line per render on stdout: name, route, resolution, fps, frames,
//!   wall seconds; then the parity frames' PSNR and SSIM (ffmpeg).

use std::path::{Path, PathBuf};
use std::time::Instant;

use explainer_kit::encode::{self, Quality};
use explainer_score::instr::{self, Instrument, Note};
use explainer_score::{Stereo, wav};
use fmn::library::Prism;
use fmn::prelude::*;
use fmn::rendering::CameraConfig;
use fmn::scene::CameraRig;

const COLS: usize = 44;
const ROWS: usize = 11;
/// The orbit's tilt from straight down (rad) and its first heading.
const PHI: f64 = 1.05;
const THETA0: f64 = -0.6;

/// A smooth, slightly skewed likelihood bump over the 44×11 grid.
fn height(i: usize, j: usize) -> f64 {
    let (x, y) = ((i as f64 - 25.0) / 7.0, (j as f64 - 5.0) / 2.6);
    let skew = if x < 0.0 { 1.4 } else { 1.0 };
    0.12 + 2.6 * (-(x * skew).powi(2) - y * y).exp()
}

fn bar_color(h: f64) -> Srgb {
    let t = ((h - 0.12) / 2.6).clamp(0.0, 1.0);
    fmn::core::color::interpolate_color(hex("#1F5F8B"), hex("#F0AC5F"), t)
}

fn hex(s: &str) -> Srgb {
    explainer_kit::hex(s)
}

/// The bar field and its slice planes, added to the scene.
fn add_field(stage: &mut Stage<'_>) -> fmn::Result<()> {
    for i in 0..COLS {
        for j in 0..ROWS {
            let h = height(i, j);
            let b = stage.add(Prism::new(0.2, 0.3, h).color(bar_color(h)))?;
            let (x, y) = (-5.375 + 0.25 * i as f64, -2.0 + 0.4 * j as f64);
            stage.shift(b, [x, y, h / 2.0]);
        }
    }
    // translucent slice planes, added last: thin boxes standing in the field
    for y in [-0.4, 1.2] {
        let p = stage.add(Prism::new(11.4, 0.01, 3.0).color(hex("#ECEFF4")).opacity(0.18))?;
        stage.shift(p, [0.0, y, 1.5]);
    }
    Ok(())
}

fn angled(base: &CameraConfig, theta: f64) -> fmn::Result<CameraConfig> {
    let mut c = base.clone();
    c.frame
        .set_euler_angles(Some(theta), Some(PHI), None)
        .map_err(SceneError::from)?;
    Ok(c)
}

struct Orbit {
    rig: CameraRig,
    arcs: Vec<CameraConfig>,
    arc_s: f64,
    sound: Option<PathBuf>,
}

impl SceneConstruct for Orbit {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        add_field(stage)?;
        if let Some(s) = &self.sound {
            stage.scene_mut().add_sound(s.clone(), 0.0, Some(-6.0), None)?;
        }
        if self.arcs.is_empty() {
            stage.wait(self.arc_s)?;
        }
        for target in &self.arcs {
            let a = self.rig.animate_to(stage.scene_mut(), target)?;
            stage.play_prepared_with(
                vec![Box::new(a)],
                PlayOverrides {
                    run_time: Some(self.arc_s),
                    rate_func: Some(RateFunc::linear()),
                    ..PlayOverrides::default()
                },
            )?;
        }
        Ok(())
    }
}

/// The parity composition: strokes, fills and a thin line, flat in z = 0.
struct Flat;

impl SceneConstruct for Flat {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let c = stage.add(Circle::new().radius(1.6).color(hex("#58C4DD")))?;
        stage.shift(c, [-3.0, 0.0, 0.0]);
        let s = stage.add(Square::new().side_length(2.4).color(hex("#FFD866")))?;
        stage.set_fill(s, Some(hex("#FFD866")), Some(0.5), None, true);
        stage.shift(s, [2.5, 0.3, 0.0]);
        let line = Line::new([-6.0, -2.8, 0.0], [6.0, -2.3, 0.0])
            .style(explainer_kit::stroke_style(hex("#ECEFF4"), 2.0))
            .build()?;
        stage.add(line)?;
        stage.wait(1.0 / 60.0)?;
        Ok(())
    }
}

fn sound(path: &Path, seconds: f64) -> std::io::Result<()> {
    // a soft D major chord on the pad, as a stand-in for a score stem
    let n = (seconds * 48_000.0) as usize;
    let mut bus = Stereo::silence(n);
    for (k, p) in [50.0, 57.0, 62.0, 66.0].into_iter().enumerate() {
        let note = Note::new(p, 0.5, seconds - 1.5, 11 + k as u64);
        bus.mix_in(&instr::render(Instrument::Pad { brightness: 0.3 }, &note), 0, 0.25);
    }
    wav::write(path, &bus, wav::Format::F32)
}

fn video_options(path: PathBuf, res: (u32, u32), ten_bit: bool) -> Result<RenderOptions, Box<dyn std::error::Error>> {
    let mut o = encode::delivery_options(
        path,
        RenderFormat::Mp4,
        Quality {
            quality: 16,
            preset: "slow",
            ..Quality::delivery()
        },
    )?;
    o.config.camera.resolution = res;
    o.config.camera.fps = 60;
    o.config.camera.background_color = "#0E1116".into();
    // the whole field in view at every heading of the orbit
    o.config.sizes.frame_height = 11.0;
    if ten_bit {
        o.config.file_writer.pixel_format = "p010le".into();
    }
    Ok(o)
}

fn log(name: &str, route: &str, res: (u32, u32), frames: u64, started: Instant) {
    let secs = started.elapsed().as_secs_f64();
    println!(
        "{name}\t{route}\t{}x{}\t{:.1} fps\t{frames} frames\t{secs:.1} s",
        res.0,
        res.1,
        frames as f64 / secs
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let out = PathBuf::from(args.first().ok_or("usage: spike_s5_camera OUT_DIR [WxH] [SECONDS]")?);
    let res = match args.get(1) {
        Some(r) => {
            let (w, h) = r.split_once('x').ok_or("WxH")?;
            (w.parse()?, h.parse()?)
        }
        None => (3840, 2160),
    };
    let seconds: f64 = args.get(2).map_or(Ok(10.0), |s| s.parse())?;
    if out.exists() {
        return Err(format!("{} exists", out.display()).into());
    }
    std::fs::create_dir_all(&out)?;
    let cue = out.join("cue.wav");
    sound(&cue, seconds)?;

    // the orbit, with audio
    let started = Instant::now();
    let opts = video_options(out.join("orbit.mp4"), res, false)?;
    // frame-exact arcs: the stage rounds a run time up to whole frames, so
    // ask for half a frame less than the frames wanted (10 / 3 s would round
    // each arc up to 201 frames)
    let arc_frames = (seconds * 60.0 / 3.0).round();
    let arc_s = (arc_frames - 0.5) / 60.0;
    let cue_path = cue.clone();
    let report = encode::render_camera(
        move |scene, base| {
            let start = angled(base, THETA0)?;
            let rig = CameraRig::new(scene, &start)?;
            let arcs = (1..=3)
                .map(|k| angled(base, THETA0 + k as f64 * std::f64::consts::TAU / 3.0))
                .collect::<fmn::Result<Vec<_>>>()?;
            Ok((
                Orbit {
                    rig,
                    arcs,
                    arc_s,
                    sound: Some(cue_path),
                },
                rig,
            ))
        },
        opts,
    )?;
    log("orbit", "camera", res, report.artifact.frame_count, started);

    // 8- and 10-bit stills of the field from the first angle
    for (name, ten) in [("still_8bit", false), ("still_10bit", true)] {
        let started = Instant::now();
        let opts = video_options(out.join(format!("{name}.mp4")), res, ten)?;
        let report = encode::render_camera(
            |scene, base| {
                let start = angled(base, THETA0)?;
                let rig = CameraRig::new(scene, &start)?;
                Ok((
                    Orbit {
                        rig,
                        arcs: Vec::new(),
                        arc_s: 1.0,
                        sound: None,
                    },
                    rig,
                ))
            },
            opts,
        )?;
        log(name, "camera", res, report.artifact.frame_count, started);
    }

    // parity: the same 2D composition on both routes, one frame each
    let parity = (1920, 1080);
    let png = |dir: &str| -> Result<RenderOptions, Box<dyn std::error::Error>> {
        let mut o = RenderOptions::with_format(out.join(dir), RenderFormat::PngSequence)?;
        o.config.camera.resolution = parity;
        o.config.camera.fps = 60;
        o.config.camera.background_color = "#0E1116".into();
        Ok(o)
    };
    let started = Instant::now();
    let report = fmn::render(&mut Flat, png("parity_retained")?)?;
    log("parity", "retained", parity, report.artifact.frame_count, started);
    let started = Instant::now();
    let report = encode::render_camera(
        |scene, base| Ok((Flat, CameraRig::new(scene, base)?)),
        png("parity_camera")?,
    )?;
    log("parity", "camera", parity, report.artifact.frame_count, started);
    let frame = |dir: &str| out.join(dir).join("frame_000000.png");
    let cmp = std::process::Command::new("ffmpeg")
        .args(["-nostdin", "-hide_banner", "-i"])
        .arg(frame("parity_retained"))
        .arg("-i")
        .arg(frame("parity_camera"))
        .args(["-lavfi", "[0][1]psnr;[0][1]ssim", "-f", "null", "-"])
        .output()?;
    for line in String::from_utf8_lossy(&cmp.stderr).lines() {
        if let Some(k) = line.find("PSNR").or_else(|| line.find("SSIM")) {
            println!("parity\t{}", &line[k..]);
        }
    }
    Ok(())
}
