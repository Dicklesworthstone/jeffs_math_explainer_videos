//! Chapter 16's paired shear display (Agent Mail 36334): the measured periodic shear runs as their
//! solver saved them, uniform fine beside adaptive at the same simulated time, stepped across all 17
//! saved observations on the 16_7 speech clock, before the measured table.
//!
//! Data: navier-shear-replay01, the timed fine and adaptive runs of navier-solver-shear03 replayed by
//! root's capture-shear-replay.py, which saved the frames only after each replay's observation log
//! equalled its timed run's (`replay.json` records `clock_and_observables_match` and every frame's
//! sha256). [`bind`] loads both directories through `crate::load_frames` and `crate::field::Frame`,
//! and refuses any file whose name, size, clock, cell count or level range differs from the pins
//! below (check_shear_display01.py derives every pin from the hash-checked bytes). The fields are drawn
//! by `crate::texture` and `crate::panel` exactly as the crate's own comparison scene draws them:
//! vorticity coloured by `Frame::raster`, and on the adaptive panel the outline of every saved leaf.
//! Nothing here computes, smooths or invents a field.
//!
//! Display clock: each panel is rasterized once per saved observation (34 rasters in all), never per
//! video frame and never between observations (no `next` frame: no interpolation). The 17 saved times
//! are 0.025 s of simulated time apart; on screen each is held for the same whole number of frames, and
//! the display says so.
//!
//! Routing (story.rs; root wires it): `--shear-replay DIR` is bound once by [`bind`] before the kit
//! parses the command line, as `--tunnel` is. The raster follows the kit's own `--res` height and the
//! story's frame height (the last `--frame-height`, else `crate::FRAME_HEIGHT`, which `story::run`
//! adds to every render that gives none). Chapter 16 must ask for the camera route in its `make`
//! (`kit.request_camera(CameraShot { theta: 0.0, phi: 0.0 })`): fmn draws textured images only on its
//! explicit camera route (`RenderOptions::camera`), as chapter 04's plate does. The engine's fades do
//! not fade a raster quad, so [`ShearDisplay::show`] and [`ShearDisplay::leave`] use chapter 03's
//! `fade_in_all` and `fade_out_all`, which add the plate fade.
//!
//! Layout: chapter 16's main window (x within −7.1 to 0.2, as chapter 05's layout test windows it),
//! above the caption slot (y above −3.15). The side panel, where 16_7's tables stand, is left alone.
//! Everything sits inside both the story's 16 × 9 frame and the 14.2 × 8 frame the kit's layout lint
//! measures (fmn's default radii).

use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use explainer_kit::{Kit, Palette, hex, thousands};
use fmn::prelude::*;

use crate::field::Frame;

/// The command-line flag [`bind`] takes, followed by the navier-shear-replay01 directory.
pub const REPLAY_FLAG: &str = "--shear-replay";

/// The 17 saved observations: solver steps (the file names), clocks (`{:?}` of the saved f64) and leaf
/// counts. Both runs saved at the same 17 steps and clocks.
pub const STEPS: [u32; 17] = [
    0, 19, 31, 42, 53, 64, 75, 86, 97, 108, 119, 130, 141, 152, 163, 174, 185,
];
pub const TIMES: [&str; 17] = [
    "0.0",
    "0.025",
    "0.05",
    "0.07500000000000001",
    "0.1",
    "0.125",
    "0.15",
    "0.175",
    "0.19999999999999998",
    "0.22499999999999998",
    "0.24999999999999997",
    "0.27499999999999997",
    "0.3",
    "0.325",
    "0.35000000000000003",
    "0.37500000000000006",
    "0.4",
];
pub const FINE_CELLS: usize = 16_384;
pub const ADAPTIVE_CELLS: [usize; 17] = [
    5584, 5488, 5632, 5584, 5632, 5632, 5824, 5632, 5920, 6016, 5920, 5728, 5824, 6016, 5968, 5968, 6304,
];
/// File sizes in bytes (NSFLOW01: 80 + 52 per leaf + 32 per face).
pub const FINE_BYTES: u64 = 1_900_624;
pub const ADAPTIVE_BYTES: [u64; 17] = [
    651_408, 640_272, 656_976, 651_408, 656_976, 656_976, 679_248, 656_976, 690_384, 701_520, 690_384, 668_112,
    679_248, 701_520, 695_952, 695_952, 734_928,
];
/// Leaf levels (lowest, highest) in every saved observation.
pub const FINE_LEVELS: (u32, u32) = (7, 7);
pub const ADAPTIVE_LEVELS: (u32, u32) = (4, 7);

/// The kit's render height when `--res` is not given (explainer_kit cli: 1920 × 1080).
const DEFAULT_HEIGHT: u32 = 1080;
/// The kit's flag for the camera frame's height in scene units.
const FRAME_FLAG: &str = "--frame-height";

// Layout, scene units.
/// Each panel's side: the shear box is square.
const PANEL: f64 = 3.3;
/// An 11/1080-unit nudge keeps both of each image's triangle diagonals at least 2/9 px off pixel
/// centres at 540, 1080 and 2160 lines on the story's frame, 9 units tall, and 1/4 px on fmn's
/// default 8 (the check recomputes both); on whole-pixel corners the shared diagonal shows as a seam.
const NUDGE: f64 = 11.0 / 1080.0;
/// The fine panel's rim stays inside the kit lint's 0.15 edge margin (the lint's frame edge −7.11).
const FINE_AT: [f64; 3] = [-5.25 + NUDGE, 1.2, 0.0];
const ADAPTIVE_AT: [f64; 3] = [-1.6 + NUDGE, 1.2, 0.0];
/// The rim stands this far outside each panel's edge.
const RIM_GAP: f64 = 0.03;
const RIM_WIDTH: f64 = 1.6;
/// Texts, track and key centre under the pair (x −3.41).
const HEAD_AT: [f64; 3] = [-3.4, 3.55, 0.0];
const NAME_Y: f64 = 3.1;
const COUNT_Y: f64 = -0.72;
const TIME_AT: [f64; 3] = [-3.4, -1.17, 0.0];
/// The time track: simulated time 0 at the left end, the last saved time at the right.
const TRACK_Y: f64 = -1.6;
const TRACK_X: [f64; 2] = [-5.35, -1.45];
const TICK: f64 = 0.07;
const MARK_R: f64 = 0.075;
const END_GAP: f64 = 0.15;
const NOTE_AT: [f64; 3] = [-3.4, -1.98, 0.0];
const KEY_Y: f64 = -2.36;
const KEY_X: [f64; 2] = [-4.85, -1.95];
const UNITS_AT: [f64; 3] = [-3.4, -2.72, 0.0];
const HEAD_SIZE: f64 = 24.0;
const NAME_SIZE: f64 = 21.0;
const COUNT_SIZE: f64 = 21.0;
const TIME_SIZE: f64 = 22.0;
const SMALL_SIZE: f64 = 18.0;

/// `Frame::raster`'s colour ends (field.rs: [68, 180, 253] below zero, [255, 122, 71] above) and the
/// vorticity at which the shear box reaches full colour (its `color_limit`).
const CLOCKWISE_HEX: &str = "#44B4FD";
const COUNTER_HEX: &str = "#FF7A47";
const FULL_COLOUR: f64 = 6.0;

const HEAD: &str = "The measured shear runs, as saved";
const KEY: [&str; 2] = ["clockwise spin", "counterclockwise spin"];

static RETAINED_SHEAR: OnceLock<ShearReplay> = OnceLock::new();

/// Both runs' saved fields, checked against the pins, and the panel raster for this render's height.
pub struct ShearReplay {
    fine: Vec<Frame>,
    adaptive: Vec<Frame>,
    /// Texels per panel side for this render.
    raster: u32,
}

/// Take `--shear-replay DIR` out of the story's arguments (call it in `story::run`, after
/// `bind_story_inputs`). The kit's `--res` and `--frame-height` are read and left in place.
///
/// # Errors
/// A missing directory, a second flag, or any saved file that differs from the pins.
pub fn bind(args: Vec<String>) -> Result<Vec<String>, String> {
    let raster = raster_for(output_height(&args)?, frame_height(&args)?, FINE_LEVELS.1);
    let mut rest = Vec::with_capacity(args.len());
    let mut items = args.into_iter();
    while let Some(arg) = items.next() {
        if arg != REPLAY_FLAG {
            rest.push(arg);
            continue;
        }
        let directory = items
            .next()
            .ok_or_else(|| format!("{REPLAY_FLAG} needs the navier-shear-replay01 directory"))?;
        let replay = load(Path::new(&directory), raster).map_err(|error| format!("{directory}: {error}"))?;
        if RETAINED_SHEAR.set(replay).is_err() {
            return Err(format!("{REPLAY_FLAG} is given once"));
        }
    }
    Ok(rest)
}

/// The bound replay; chapter 16 asks for it before its first frame.
///
/// # Errors
/// When the story was started without `--shear-replay`.
pub fn retained() -> fmn::Result<&'static ShearReplay> {
    RETAINED_SHEAR.get().ok_or_else(|| {
        crate::error(format!(
            "Chapter 16 requires {REPLAY_FLAG} and the navier-shear-replay01 directory"
        ))
    })
}

/// The render height in pixels: the last `--res WxH` (the kit keeps the last), else the kit's default.
fn output_height(args: &[String]) -> Result<u32, String> {
    let Some(at) = args.iter().rposition(|arg| arg == "--res") else {
        return Ok(DEFAULT_HEIGHT);
    };
    args.get(at + 1)
        .and_then(|value| value.split_once('x'))
        .and_then(|(_, height)| height.parse::<u32>().ok())
        .filter(|&height| height > 0)
        .ok_or_else(|| "--res needs WxH".to_string())
}

/// The camera frame's height in scene units: the last `--frame-height` (the kit keeps the last), else
/// `crate::FRAME_HEIGHT`, which `story::run` adds after this binding when no height is given.
fn frame_height(args: &[String]) -> Result<f64, String> {
    let Some(at) = args.iter().rposition(|arg| arg == FRAME_FLAG) else {
        return Ok(crate::FRAME_HEIGHT);
    };
    args.get(at + 1)
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|height| height.is_finite() && *height > 0.0)
        .ok_or_else(|| format!("{FRAME_FLAG} needs a positive height"))
}

/// Texels per panel side: a whole number of texels per finest cell (2^level_max across the box), as
/// near the panel's on-screen size as possible and never under two.
pub fn raster_for(height_px: u32, frame_height: f64, level_max: u32) -> u32 {
    let cells = 1u32 << level_max;
    let pixels = PANEL * f64::from(height_px) / frame_height;
    let per_cell = (pixels / f64::from(cells)).round().max(2.0) as u32;
    cells * per_cell
}

fn load(directory: &Path, raster: u32) -> Result<ShearReplay, String> {
    let fine = load_run(&directory.join("fine"), &[FINE_BYTES; 17])?;
    let adaptive = load_run(&directory.join("adaptive"), &ADAPTIVE_BYTES)?;
    for (k, (f, a)) in fine.iter().zip(&adaptive).enumerate() {
        let clock = format!("{:?}", f.time);
        if clock != TIMES[k] || a.time.to_bits() != f.time.to_bits() {
            return Err(format!(
                "observation {k}: clocks {:?} and {:?}, pinned {}",
                f.time, a.time, TIMES[k]
            ));
        }
        if f.wet_leaves != FINE_CELLS || a.wet_leaves != ADAPTIVE_CELLS[k] {
            return Err(format!(
                "observation {k}: {} and {} cells, pinned {FINE_CELLS} and {}",
                f.wet_leaves, a.wet_leaves, ADAPTIVE_CELLS[k]
            ));
        }
        if (f.level_min, f.level_max) != FINE_LEVELS || (a.level_min, a.level_max) != ADAPTIVE_LEVELS {
            return Err(format!("observation {k}: leaf levels differ from the pins"));
        }
        if f.is_tunnel() || a.is_tunnel() {
            return Err(format!("observation {k}: a tunnel snapshot, not the shear box"));
        }
    }
    Ok(ShearReplay { fine, adaptive, raster })
}

/// One run's 17 saved observations: exactly the pinned file names and sizes, then the crate's loader.
fn load_run(directory: &Path, bytes: &[u64; 17]) -> Result<Vec<Frame>, String> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(directory).map_err(|error| error.to_string())? {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.extension().is_some_and(|ext| ext == "flow") {
            paths.push(path);
        }
    }
    paths.sort();
    let names: Vec<String> = paths
        .iter()
        .map(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default()
        })
        .collect();
    let pinned: Vec<String> = STEPS.iter().map(|step| format!("frame-{step:07}.flow")).collect();
    if names != pinned {
        return Err(format!(
            "{}: the saved observations are not the pinned 17",
            directory.display()
        ));
    }
    for ((path, name), &want) in paths.iter().zip(&names).zip(bytes) {
        let size = fs::metadata(path).map_err(|error| error.to_string())?.len();
        if size != want {
            return Err(format!("{name}: {size} bytes, pinned {want}"));
        }
    }
    crate::load_frames(directory).map_err(|error| error.to_string())
}

/// The display's pace: `fade` seconds to appear, then each saved observation held `dwell` seconds,
/// rounded to whole frames.
#[derive(Clone, Copy, Debug)]
pub struct ShearClock {
    pub fade: f64,
    pub dwell: f64,
}

impl ShearClock {
    /// 7.3 s in all at 30 and 60 fps (12 or 24 frames per observation).
    pub const DEFAULT: Self = Self { fade: 0.5, dwell: 0.4 };

    /// Scene seconds that [`ShearDisplay::show`] and [`ShearDisplay::step`] take at `fps`, so the caller
    /// can fit them inside a line's sentence before the next gate.
    pub fn seconds(&self, fps: u32) -> fmn::Result<f64> {
        let fade = (self.fade * f64::from(fps)).ceil();
        let held = dwell_frames(self.dwell, fps)? * STEPS.len() as u64;
        Ok((fade + held as f64) / f64::from(fps))
    }
}

fn dwell_frames(dwell: f64, fps: u32) -> fmn::Result<u64> {
    let frames = (dwell * f64::from(fps)).round();
    if !dwell.is_finite() || frames < 1.0 {
        return Err(crate::error(format!(
            "shear display: a dwell of {dwell} s is under one frame"
        )));
    }
    Ok(frames as u64)
}

/// The display on the stage: its two image panels (each image inside an empty group, as chapter 04's
/// plate: the fades act on the group, the plate fade on the image), the fixed labels and track, the
/// moving marker and the three readouts of the observation on screen.
pub struct ShearDisplay {
    images: [Mob; 2],
    plates: [Mob; 2],
    fixed: Vec<Mob>,
    marker: Mob,
    readouts: [Mob; 3],
    words: [String; 3],
    shown: usize,
}

fn v(x: f64, y: f64) -> [f64; 3] {
    [x, y, 0.0]
}

/// Where saved observation `k` sits on the time track.
fn track_x(replay: &ShearReplay, k: usize) -> f64 {
    let start = replay.fine[0].time;
    let end = replay.fine[replay.fine.len() - 1].time;
    TRACK_X[0] + (replay.fine[k].time - start) / (end - start) * (TRACK_X[1] - TRACK_X[0])
}

/// The readouts of observation `k`, from its frames: the two active cell counts and the shared clock.
fn readout_words(replay: &ShearReplay, k: usize) -> [String; 3] {
    [
        format!("{} active cells", thousands(replay.fine[k].wet_leaves as u64)),
        format!(
            "{} active cells, outlined",
            thousands(replay.adaptive[k].wet_leaves as u64)
        ),
        format!("both at simulated time {:.3} s", replay.fine[k].time),
    ]
}

fn readout_places() -> [([f64; 3], f64, Srgb); 3] {
    [
        (v(FINE_AT[0], COUNT_Y), COUNT_SIZE, Palette::purple()),
        (v(ADAPTIVE_AT[0], COUNT_Y), COUNT_SIZE, Palette::teal()),
        (TIME_AT, TIME_SIZE, Palette::ink()),
    ]
}

fn line(stage: &mut Stage<'_>, from: [f64; 3], to: [f64; 3], color: Srgb, width: f64) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(
        Line::new(from, to)
            .style(Style::default().stroke(color, width, 1.0))
            .build()?,
    ))
}

fn rim(stage: &mut Stage<'_>, at: [f64; 3]) -> fmn::Result<Mob> {
    let side = PANEL + 2.0 * RIM_GAP;
    let dim = Palette::dim();
    Ok(stage.arena_mut().add(
        Rectangle::new()
            .width(side)
            .height(side)
            .style(Style::default().fill(dim, 0.0).stroke(dim, RIM_WIDTH, 1.0))
            .build()?
            .moved_to(at),
    ))
}

fn marker(stage: &mut Stage<'_>, at: [f64; 3]) -> Mob {
    let gold = Palette::gold();
    stage.arena_mut().add(
        Circle::new()
            .radius(MARK_R)
            .style(Style::default().fill(gold, 1.0).stroke(gold, 1.0, 1.0))
            .build()
            .moved_to(at),
    )
}

impl ShearDisplay {
    /// Build the display, detached, on saved observation 0.
    ///
    /// # Errors
    /// A refused raster or text.
    pub fn build(kit: &Kit, stage: &mut Stage<'_>, replay: &ShearReplay) -> fmn::Result<Self> {
        let (fine, adaptive) = (&replay.fine[0], &replay.adaptive[0]);
        let images = [
            crate::panel(stage, crate::texture(fine, replay.raster, false, None)?, PANEL, FINE_AT)?,
            crate::panel(
                stage,
                crate::texture(adaptive, replay.raster, true, None)?,
                PANEL,
                ADAPTIVE_AT,
            )?,
        ];
        // `crate::panel` roots its image; the display appears as one piece, so take both back out and
        // put each inside an empty group for `show` and `leave`.
        stage.scene_mut().remove(&images);
        let mut plates = images;
        for (plate, &image) in plates.iter_mut().zip(&images) {
            *plate = stage.arena_mut().add(VMobject::new());
            stage.attach(*plate, image)?;
        }

        let (ink, dim) = (Palette::ink(), Palette::dim());
        let mut fixed = vec![
            rim(stage, FINE_AT)?,
            rim(stage, ADAPTIVE_AT)?,
            kit.text(stage, HEAD, HEAD_SIZE, ink, HEAD_AT)?,
            kit.text(
                stage,
                &format!("uniform fine, {0} × {0}", 1u32 << fine.level_max),
                NAME_SIZE,
                Palette::purple(),
                v(FINE_AT[0], NAME_Y),
            )?,
            kit.text(
                stage,
                &format!("adaptive, levels {} to {}", adaptive.level_min, adaptive.level_max),
                NAME_SIZE,
                Palette::teal(),
                v(ADAPTIVE_AT[0], NAME_Y),
            )?,
            line(stage, v(TRACK_X[0], TRACK_Y), v(TRACK_X[1], TRACK_Y), dim, 2.0)?,
        ];
        for k in 0..replay.fine.len() {
            let x = track_x(replay, k);
            fixed.push(line(stage, v(x, TRACK_Y - TICK), v(x, TRACK_Y + TICK), dim, 1.6)?);
        }
        let last = replay.fine.len() - 1;
        let start = kit.text(stage, &format!("{:.3} s", fine.time), SMALL_SIZE, dim, v(0.0, TRACK_Y))?;
        stage.move_to(start, v(TRACK_X[0] - END_GAP, TRACK_Y), RIGHT);
        let end_words = format!("{:.3} s", replay.fine[last].time);
        let end = kit.text_left(stage, &end_words, SMALL_SIZE, dim, v(TRACK_X[1] + END_GAP, TRACK_Y))?;
        let gap = replay.fine[1].time - fine.time;
        let note = format!(
            "{} saved times, {gap:.3} s apart, each held equally long",
            replay.fine.len()
        );
        fixed.extend([
            start,
            end,
            kit.text(stage, &note, SMALL_SIZE, dim, NOTE_AT)?,
            kit.text(stage, KEY[0], SMALL_SIZE, hex(CLOCKWISE_HEX), v(KEY_X[0], KEY_Y))?,
            kit.text(stage, KEY[1], SMALL_SIZE, hex(COUNTER_HEX), v(KEY_X[1], KEY_Y))?,
            kit.text(
                stage,
                &format!("colour: spin (vorticity), full at {FULL_COLOUR} per second"),
                SMALL_SIZE,
                dim,
                UNITS_AT,
            )?,
        ]);
        let words = readout_words(replay, 0);
        let places = readout_places();
        let mut readouts = Vec::with_capacity(3);
        for (text, (at, size, color)) in words.iter().zip(places) {
            readouts.push(kit.text(stage, text, size, color, at)?);
        }
        Ok(Self {
            images,
            plates,
            fixed,
            marker: marker(stage, v(track_x(replay, 0), TRACK_Y)),
            readouts: [readouts[0], readouts[1], readouts[2]],
            words,
            shown: 0,
        })
    }

    /// Everything the display has on the stage now (the plates stand for their images).
    pub fn mobs(&self) -> Vec<Mob> {
        let mut all = self.plates.to_vec();
        all.extend(&self.fixed);
        all.push(self.marker);
        all.extend(self.readouts);
        all
    }

    /// Fade the display in over `secs`, image panels included.
    ///
    /// # Errors
    /// A refused animation.
    pub fn show(&self, stage: &mut Stage<'_>, secs: f64) -> fmn::Result<()> {
        super::chapter03::fade_in_all(stage, &self.mobs(), secs)
    }

    /// Hold each saved observation from the one on screen to the last for `dwell` seconds (whole
    /// frames), swapping both panels, the readouts and the marker at each new observation.
    ///
    /// # Errors
    /// A dwell under one frame, or a refused raster, text or wait.
    pub fn step(&mut self, kit: &Kit, stage: &mut Stage<'_>, replay: &ShearReplay, dwell: f64) -> fmn::Result<()> {
        let fps = stage.scene().fps();
        let frames = dwell_frames(dwell, fps)?;
        for k in self.shown..replay.fine.len() {
            if k != self.shown {
                self.swap_to(kit, stage, replay, k)?;
            }
            // The engine covers ceil(seconds × fps) frames: half a frame short lands on `frames` exactly.
            stage.wait((frames as f64 - 0.5) / f64::from(fps))?;
        }
        Ok(())
    }

    /// Put saved observation `k` on screen at once: one new raster per panel, no blend between saves.
    fn swap_to(&mut self, kit: &Kit, stage: &mut Stage<'_>, replay: &ShearReplay, k: usize) -> fmn::Result<()> {
        let fine = crate::texture(&replay.fine[k], replay.raster, false, None)?;
        let adaptive = crate::texture(&replay.adaptive[k], replay.raster, true, None)?;
        stage.arena_mut().set_image_resource(self.images[0], Some(fine))?;
        stage.arena_mut().set_image_resource(self.images[1], Some(adaptive))?;
        let words = readout_words(replay, k);
        for (i, (text, (at, size, color))) in words.iter().zip(readout_places()).enumerate() {
            if *text != self.words[i] {
                let fresh = kit.text(stage, text, size, color, at)?;
                stage.scene_mut().remove(&[self.readouts[i]]);
                stage.add_to_scene(fresh)?;
                self.readouts[i] = fresh;
            }
        }
        self.words = words;
        stage.move_to(self.marker, v(track_x(replay, k), TRACK_Y), ORIGIN);
        self.shown = k;
        Ok(())
    }

    /// Fade the display out over `secs`, image panels included; nothing of it stays on the stage.
    ///
    /// # Errors
    /// A refused animation.
    pub fn leave(self, stage: &mut Stage<'_>, secs: f64) -> fmn::Result<()> {
        super::chapter03::fade_out_all(stage, &self.mobs(), secs)
    }
}

/// Build, fade in and step through all 17 saved observations; the display stays on screen for the
/// caller to [`ShearDisplay::leave`] (before 16_7's tables, or with 16_8's clear).
///
/// # Errors
/// As [`ShearDisplay::build`], [`ShearDisplay::show`] and [`ShearDisplay::step`].
pub fn play(kit: &Kit, stage: &mut Stage<'_>, replay: &ShearReplay, clock: ShearClock) -> fmn::Result<ShearDisplay> {
    // The glyph probe constructs the real scene at one frame per second.
    // It needs one frame for each saved observation to register its readouts.
    let fps = stage.scene().fps();
    let clock = if fps == 1 {
        ShearClock { dwell: clock.dwell.max(1.0), ..clock }
    } else {
        clock
    };
    // The render log carries the span, so the 16_7 gates can be read against it.
    println!(
        "Chapter 16 shear display: {} saved observations, {:.6} s from {:.6} s, raster {}",
        STEPS.len(),
        clock.seconds(fps)?,
        stage.scene().time().to_f64(),
        replay.raster
    );
    let mut display = ShearDisplay::build(kit, stage, replay)?;
    display.show(stage, clock.fade)?;
    display.step(kit, stage, replay, clock.dwell)?;
    Ok(display)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pins_are_seventeen_ascending_saved_clocks() {
        let times: Vec<f64> = TIMES.iter().map(|t| t.parse().unwrap()).collect();
        assert!(times.windows(2).all(|pair| pair[0] < pair[1]));
        for (time, pinned) in times.iter().zip(TIMES) {
            assert_eq!(format!("{time:?}"), pinned);
        }
        assert!(STEPS.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(ADAPTIVE_CELLS.iter().all(|&cells| cells < FINE_CELLS));
    }

    #[test]
    fn rasters_keep_whole_texels_per_finest_cell() {
        assert_eq!(raster_for(540, 9.0, 7), 256);
        assert_eq!(raster_for(1080, 9.0, 7), 384);
        assert_eq!(raster_for(2160, 9.0, 7), 768);
        assert_eq!(raster_for(540, 8.0, 7), 256);
        assert_eq!(raster_for(1080, 8.0, 7), 384);
        assert_eq!(raster_for(2160, 8.0, 7), 896);
    }

    #[test]
    fn the_kit_resolution_and_frame_are_read_and_left_in_place() {
        let args: Vec<String> = [
            "render",
            "16_was_the_saving_real",
            "--res",
            "3840x2160",
            "--frame-height",
            "8",
        ]
        .map(str::to_owned)
        .to_vec();
        assert_eq!(output_height(&args), Ok(2160));
        assert_eq!(output_height(&args[..2]), Ok(DEFAULT_HEIGHT));
        assert_eq!(frame_height(&args), Ok(8.0));
        assert_eq!(frame_height(&args[..4]), Ok(crate::FRAME_HEIGHT));
        assert_eq!(crate::FRAME_HEIGHT, 9.0);
    }

    #[test]
    fn whole_frames_per_saved_observation() {
        assert_eq!(dwell_frames(0.5, 30).unwrap(), 15);
        assert_eq!(dwell_frames(0.5, 60).unwrap(), 30);
        assert!(dwell_frames(0.01, 30).is_err());
        assert_eq!(ShearClock::DEFAULT.seconds(30).unwrap(), 7.3);
        assert_eq!(ShearClock::DEFAULT.seconds(60).unwrap(), 7.3);
    }
}
