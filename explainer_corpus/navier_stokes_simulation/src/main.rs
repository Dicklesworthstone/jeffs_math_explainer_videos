//! Native comparison scene. The two panels share one physical clock.

mod adapt_row;
mod box_view;
mod field;
mod four_cell;
mod opening;
mod story;
mod transport;
mod tube;
mod wake_ledger;

use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use explainer_kit::encode::{Quality, delivery_options, render_camera};
use explainer_kit::palette::{Palette, hex, ink_style};
use field::Frame;
use fmn::mobject::{ImageColorSpace, ImageResource, ImageSampler, ImageWrap};
use fmn::prelude::*;
use opening::OpeningScene;

const FRAME_WIDTH: f64 = 13.6;
const FRAME_HEIGHT: f64 = 9.0;

struct TunnelScene {
    adaptive: Vec<Frame>,
    fixed: Option<Vec<Frame>>,
    raster_width: u32,
    fps: u32,
    motion_seconds: f64,
    narration: Option<(PathBuf, f64)>,
    measured_note: Option<String>,
}

fn load_frames(directory: &Path) -> Result<Vec<Frame>, Box<dyn std::error::Error>> {
    let mut paths: Vec<_> = fs::read_dir(directory)?
        .map(|entry| entry.map(|item| item.path()))
        .collect::<Result<_, _>>()?;
    paths.retain(|path| path.extension().is_some_and(|ext| ext == "flow"));
    paths.sort();
    let frames = paths.iter().map(|path| Frame::load(path)).collect::<io::Result<Vec<_>>>()?;
    if frames.is_empty() || frames.windows(2).any(|pair| pair[0].time >= pair[1].time) {
        return Err(format!("No ascending snapshot clock in {}", directory.display()).into());
    }
    Ok(frames)
}

fn error(message: String) -> fmn::Error {
    fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(
        Box::leak(message.into_boxed_str()),
    ))
}

fn label(
    stage: &mut Stage<'_>,
    fonts: &FontBook,
    text: &str,
    size: f64,
    color: Srgb,
    at: [f64; 3],
) -> fmn::Result<Mob> {
    let object = Text::new(text).font_size(size).style(ink_style(color)).build(fonts)?;
    let mob = stage.add(object)?;
    stage.move_to(mob, at, ORIGIN);
    Ok(mob)
}

fn texture(
    frame: &Frame,
    width: u32,
    mesh: bool,
    next: Option<(&Frame, f64)>,
) -> fmn::Result<ImageResource> {
    let height = (f64::from(width) * frame.height_over_width()).round() as u32;
    let rgba = frame.raster(width, height, mesh, next);
    let sampler = ImageSampler { wrap_u: ImageWrap::ClampToEdge, wrap_v: ImageWrap::ClampToEdge };
    ImageResource::rgba8(width, height, rgba, ImageColorSpace::Srgb, sampler)
        .map_err(|e| error(e.to_string()))
}

fn panel(stage: &mut Stage<'_>, raster: ImageResource, width: f64, at: [f64; 3]) -> fmn::Result<Mob> {
    let height = width * f64::from(raster.height()) / f64::from(raster.width());
    let image = ImageMobject::from_resource(raster).map_err(|e| error(e.to_string()))?;
    let mob = stage.add(image.with_height(height))?;
    stage.move_to(mob, at, ORIGIN);
    Ok(mob)
}

impl SceneConstruct for TunnelScene {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let fonts = FontBook::bundled().map_err(fmn::library::TextMobjectError::Text)?;
        let fixed = self.fixed.as_deref().unwrap_or(&self.adaptive);
        let same_run = self.fixed.is_none();
        let tunnel = self.adaptive[0].is_tunnel();
        let left = FRAME_WIDTH * -0.5;
        let (width, first, second) = if tunnel {
            (FRAME_WIDTH, [0.0, 1.32, 0.0], [0.0, -1.72, 0.0])
        } else {
            // Keep the two triangles' shared edge off the camera sample centers.
            (4.95, [-3.46, 0.0115, 0.0], [3.46, 0.0115, 0.0])
        };
        let upper = panel(stage, texture(&fixed[0], self.raster_width, !tunnel, None)?, width, first)?;
        let lower = panel(stage, texture(&self.adaptive[0], self.raster_width, true, None)?, width, second)?;
        let title = if tunnel { "Where does the grid earn its keep?" }
                    else { "Follow the moving shear layer" };
        label(stage, &fonts, title, 46.0, Palette::ink(),
              [0.0, 3.91, 0.0])?;
        let caption = if tunnel && same_run { "One tunnel, two views of the same flow" }
                      else { "Same flow, same observation times" };
        label(stage, &fonts, caption, 24.0, Palette::dim(), [0.0, 3.34, 0.0])?;
        let upper_name = if same_run { "Vorticity" } else { "Uniform fine grid" };
        let (name_one, name_two, count_one, count_two, time_at) = if tunnel {
            ([left + 1.25, 2.81, 0.0], [left + 1.15, -0.23, 0.0],
             [4.65, 2.81, 0.0], [4.65, -0.23, 0.0], [4.48, -3.44, 0.0])
        } else {
            ([-3.46, 2.85, 0.0], [3.46, 2.85, 0.0],
             [-3.46, -2.77, 0.0], [3.46, -2.77, 0.0], [0.0, -3.36, 0.0])
        };
        label(stage, &fonts, upper_name, 27.0, Palette::ink(), name_one)?;
        label(stage, &fonts, "Adaptive grid", 27.0, Palette::ink(), name_two)?;
        label(stage, &fonts, "Clockwise spin", 22.0, hex("#44B4FD"), [-4.74, -3.36, 0.0])?;
        let warm_at = if tunnel { [-0.85, -3.36, 0.0] } else { [4.74, -3.36, 0.0] };
        label(stage, &fonts, "Counterclockwise spin", 22.0, hex("#FF7A47"), warm_at)?;
        let note = self.measured_note.as_deref().unwrap_or("Sampled fields, interpolated for display");
        label(stage, &fonts, note, 20.0, Palette::dim(), [0.0, -3.99, 0.0])?;
        if let Some((path, _)) = &self.narration {
            stage.scene_mut().add_sound(path, 0.26, Some(-3.0), None)?;
        }

        let moving_frames = (f64::from(self.fps) * self.motion_seconds).round().max(1.0) as usize;
        let start = self.adaptive[0].time;
        let end = self.adaptive.last().unwrap().time;
        let mut source_index = 0;
        let mut readouts = Vec::new();
        for frame in 0..moving_frames {
            let fraction = frame as f64 / (moving_frames - 1).max(1) as f64;
            let now = if frame + 1 == moving_frames { end } else { start + fraction * (end - start) };
            let before = source_index;
            while source_index + 1 < self.adaptive.len() && self.adaptive[source_index + 1].time <= now {
                source_index += 1;
            }
            let adaptive = &self.adaptive[source_index];
            if frame == 0 || source_index != before {
                stage.scene_mut().remove(&readouts);
                let upper_count = format!("{} fluid cells", fixed[source_index].wet_leaves);
                let lower_count = format!("{} fluid cells   |   levels {} to {}", adaptive.wet_leaves,
                                          adaptive.level_min, adaptive.level_max);
                let time = format!("Simulation time  {:.3} s", adaptive.time);
                readouts = vec![
                    label(stage, &fonts, &upper_count, 23.0, Palette::dim(), count_one)?,
                    label(stage, &fonts, &lower_count, 23.0, Palette::teal(), count_two)?,
                    label(stage, &fonts, &time, 22.0, Palette::ink(), time_at)?,
                ];
            }
            if frame != 0 {
                let adaptive_next = self.adaptive.get(source_index + 1).map(|next| {
                    (next, (now - adaptive.time) / (next.time - adaptive.time))
                });
                let upper_next = fixed.get(source_index + 1).map(|next| {
                    (next, (now - adaptive.time) / (next.time - adaptive.time))
                });
                stage.arena_mut().set_image_resource(upper, Some(texture(
                    &fixed[source_index], self.raster_width, !tunnel, upper_next,
                )?))?;
                stage.arena_mut().set_image_resource(lower, Some(texture(
                    adaptive, self.raster_width, true, adaptive_next,
                )?))?;
            }
            stage.wait(1.0 / f64::from(self.fps))?;
        }
        if let Some((_, seconds)) = &self.narration {
            let hold = (seconds + 0.56 - moving_frames as f64 / f64::from(self.fps)).max(0.0);
            if hold > 0.0 {
                stage.wait(hold)?;
            }
        }
        Ok(())
    }
}

fn read_voice(
    args: &mut impl Iterator<Item = OsString>,
) -> Result<(PathBuf, f64), Box<dyn std::error::Error>> {
    let path = fs::canonicalize(args.next().ok_or("Expected a verified narration WAV")?)?;
    let duration = args.next().ok_or("Expected narration seconds")?
        .into_string().map_err(|_| "Invalid narration duration")?.parse::<f64>()?;
    if !duration.is_finite() || duration <= 0.0 {
        return Err("Invalid narration duration".into());
    }
    Ok((path, duration))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let first = args.next().ok_or("Expected an output path or --story")?;
    if first == OsStr::new("--story") {
        let commands = args.map(|arg| arg.into_string().map_err(|_| "Non-UTF-8 story argument"))
            .collect::<Result<Vec<_>, _>>()?;
        std::process::exit(story::run(commands));
    }
    let target = PathBuf::from(first);
    let adaptive_dir = PathBuf::from(args.next().ok_or("Expected a snapshot directory")?);
    let mut fixed = None;
    let mut narration = None;
    let mut grid_voice = None;
    let mut measured_note = None;
    let mut full_size = false;
    let mut opening = false;
    let mut motion_seconds = 2.4;
    while let Some(flag) = args.next() {
        match flag.to_str() {
            Some("--fixed") => {
                let path = PathBuf::from(args.next().ok_or("Expected a fixed-grid directory")?);
                fixed = Some(load_frames(&path)?);
            }
            Some("--voice") => narration = Some(read_voice(&mut args)?),
            Some("--grid-voice") => grid_voice = Some(read_voice(&mut args)?),
            Some("--opening") => opening = true,
            Some("--measured-note") => {
                measured_note = Some(args.next().ok_or("Expected a measured caption")?
                    .into_string().map_err(|_| "Invalid measured caption")?);
            }
            Some("--4k") => full_size = true,
            Some("--still") => motion_seconds = 0.0,
            _ => return Err(format!("Unknown option {flag:?}").into()),
        }
    }
    if target.exists() {
        return Err("Output already exists".into());
    }
    if opening && (fixed.is_some() || measured_note.is_some() || motion_seconds == 0.0) {
        return Err("The opening takes a single tunnel snapshot sequence and optional voice takes".into());
    }
    if !opening && grid_voice.is_some() {
        return Err("--grid-voice is only used in the opening".into());
    }
    let mut adaptive = load_frames(&adaptive_dir)?;
    let tunnel = adaptive[0].is_tunnel();
    if let Some(frames) = &fixed
        && (frames.len() != adaptive.len() || frames.iter().zip(&adaptive)
            .any(|(a, b)| (a.time - b.time).abs() > 1e-9 || a.is_tunnel() != b.is_tunnel()))
    {
        return Err("The snapshots do not share an observation clock".into());
    }
    if motion_seconds == 0.0 {
        adaptive = vec![adaptive.pop().unwrap()];
        if let Some(frames) = &mut fixed {
            *frames = vec![frames.pop().unwrap()];
        }
    }
    let fps = if full_size { 60 } else { 30 };
    let raster_width = match (tunnel, full_size) {
        (true, true) => 3264,
        (true, false) => 1632,
        (false, true) => 1188,
        (false, false) => 594,
    };
    let mut options = delivery_options(target, RenderFormat::Mp4, Quality {
        preset: "fast",
        ..Quality::delivery()
    })?;
    options.config.sizes.frame_height = FRAME_HEIGHT;
    options.config.camera.background_color = "#0A121F".to_string();
    options.config.camera.resolution = if full_size { (3840, 2160) } else { (1920, 1080) };
    options.config.camera.fps = fps;
    let report = if opening {
        let scene = OpeningScene {
            snapshot: adaptive.pop().expect("load_frames checks for a nonempty sequence"),
            introduction: narration,
            cell_explanation: grid_voice,
        };
        render_camera(
            |runtime, config| Ok((scene, fmn::scene::CameraRig::new(runtime, config)?)),
            options,
        )?
    } else {
        let scene = TunnelScene {
            adaptive, fixed, raster_width, fps, motion_seconds, narration, measured_note,
        };
        render_camera(
            |runtime, config| Ok((scene, fmn::scene::CameraRig::new(runtime, config)?)),
            options,
        )?
    };
    println!("Rendered {} native frames to {}", report.artifact.frame_count, report.artifact.path.display());
    Ok(())
}
