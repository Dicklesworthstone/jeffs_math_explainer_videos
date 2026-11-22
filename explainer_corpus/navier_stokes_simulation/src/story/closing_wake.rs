//! Replay the guarded fine-grid wake using the native field rasterizer.
//! The production command binds the exact files to the solver receipt before rendering.

use std::path::Path;
use std::sync::OnceLock;

use fmn::prelude::{Stage, Vec3};

use crate::field::Frame;

const FLAG: &str = "--closing-wake";
const SNAPSHOTS: usize = 29;
const SAMPLE_SECONDS: f64 = 0.0125;
// The source has 880 cells across the tunnel. Give each cell three raster samples.
const RASTER_WIDTH: u32 = 3 * 880;
const PLAYBACK_SECONDS: f64 = 2.4;

static WAKE: OnceLock<ClosingWake> = OnceLock::new();

pub struct ClosingWake {
    frames: Vec<Frame>,
}

pub fn retained() -> Option<&'static ClosingWake> {
    WAKE.get()
}

pub fn bind(args: Vec<String>) -> Result<Vec<String>, String> {
    let mut rest = Vec::with_capacity(args.len());
    let mut items = args.into_iter();
    while let Some(arg) = items.next() {
        if arg != FLAG {
            rest.push(arg);
            continue;
        }
        let directory = items.next().ok_or("--closing-wake needs the saved field directory")?;
        if WAKE.get().is_some() {
            return Err("--closing-wake is given once".to_owned());
        }
        let replay = ClosingWake::load(Path::new(&directory))?;
        WAKE.set(replay).map_err(|_| "--closing-wake is given once".to_owned())?;
    }
    Ok(rest)
}

/// The capture has equally spaced physical samples. Use integer brackets so both endpoint
/// pictures land exactly on their saved fields at every output frame rate.
fn bracket(frame: usize, last: usize) -> (usize, f64) {
    let scaled = frame * (SNAPSHOTS - 1);
    (scaled / last, (scaled % last) as f64 / last as f64)
}

impl ClosingWake {
    pub(super) fn frames(&self) -> &[Frame] {
        &self.frames
    }

    fn load(directory: &Path) -> Result<Self, String> {
        let frames = crate::load_frames(directory)
            .map_err(|error| format!("{}: {error}", directory.display()))?;
        if frames.len() != SNAPSHOTS
            || frames.iter().enumerate().any(|(index, frame)| {
                !frame.is_tunnel()
                    || (frame.level_min, frame.level_max) != (10, 10)
                    || frame.wet_leaves != 143_144
                    || (frame.time - index as f64 * SAMPLE_SECONDS).abs() >= 1e-10
            })
        {
            return Err(format!(
                "{}: expected {SNAPSHOTS} fine-grid tunnel fields at 12.5 ms intervals",
                directory.display()
            ));
        }
        Ok(Self { frames })
    }

    pub fn play(&self, stage: &mut Stage<'_>, width: f64, center: Vec3) -> fmn::Result<()> {
        let clock = stage.scene().time();
        let fps = f64::from(clock.fps());
        let frame_count = (fps * PLAYBACK_SECONDS).round().max(2.0) as usize;
        let first = crate::texture(&self.frames[0], RASTER_WIDTH, false, None)?;
        let image = crate::panel(stage, first, width, center)?;
        println!(
            "Chapter 20 captured wake: {} saved fields, {} display frames from frame {}",
            self.frames.len(), frame_count, clock.frames(),
        );
        stage.wait(1.0 / fps)?;
        for frame in 1..frame_count {
            let (index, alpha) = bracket(frame, frame_count - 1);
            let next = self.frames.get(index + 1).map(|next| (next, alpha));
            let raster = crate::texture(&self.frames[index], RASTER_WIDTH, false, next)?;
            stage.arena_mut().set_image_resource(image, Some(raster))?;
            stage.wait(1.0 / fps)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_frames_span_exact_saved_endpoints_without_skipping_a_source_interval() {
        for last in [71, 143] {
            assert_eq!(bracket(0, last), (0, 0.0));
            assert_eq!(bracket(last, last), (SNAPSHOTS - 1, 0.0));
            let mut prior = (0, 0.0);
            for frame in 1..=last {
                let next = bracket(frame, last);
                assert!(next > prior && next.0 <= prior.0 + 1);
                assert!((0.0..1.0).contains(&next.1));
                prior = next;
            }
        }
    }
}
