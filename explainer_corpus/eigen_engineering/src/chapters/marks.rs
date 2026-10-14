//! Scene-time marks for the chapter tests: a scene says "this is whole on
//! screen now" or "this starts to leave now", and a test reads the moment
//! back from a headless run, so a picture's time on screen is measured and
//! not read off the constants that were meant to produce it (the same device
//! as `draw::quiz_log`, for pictures that are not quiz cards).
//!
//! Outside tests [`mark`] does nothing.

use explainer_kit::Kit;
use fmn::prelude::*;

/// Note that `what` is so now, in the run this kit names. `what` reads
/// `"<line>.<thing>"`: `"05_4.why_in"`.
pub fn mark(kit: &Kit, stage: &Stage<'_>, what: &'static str) {
    #[cfg(test)]
    {
        if let Ok(mut all) = log::MARKS.lock() {
            all.push((kit.chapter.clone(), what, stage.scene().time().to_f64()));
        }
    }
    #[cfg(not(test))]
    {
        let _ = (kit, stage, what);
    }
}

/// Note a measured `value` (scene units) of what the scene has just laid out,
/// under `what`, in the run this kit names; read back with [`at`].
pub fn note(kit: &Kit, what: &'static str, value: f64) {
    #[cfg(test)]
    {
        if let Ok(mut all) = log::MARKS.lock() {
            all.push((kit.chapter.clone(), what, value));
        }
    }
    #[cfg(not(test))]
    {
        let _ = (kit, what, value);
    }
}

/// How many of `mobs` have nothing on screen to fade (fill and stroke both at
/// 0 where either is known): a fade-out of one is a flash, up and down.
pub fn count_unseen(stage: &Stage<'_>, mobs: &[Mob]) -> usize {
    mobs.iter()
        .filter(|&&m| {
            let (fill, stroke) = (stage.get_fill_opacity(m), stage.get_stroke_opacity(m));
            (fill.is_some() || stroke.is_some()) && fill.unwrap_or(0.0) <= 0.01 && stroke.unwrap_or(0.0) <= 0.01
        })
        .count()
}

/// When `what` was marked (or what value it was noted at) in the run named
/// `chapter`, the first time.
#[cfg(test)]
pub fn at(chapter: &str, what: &str) -> f64 {
    all(chapter)
        .into_iter()
        .find(|(w, _)| *w == what)
        .map(|(_, t)| t)
        .unwrap_or_else(|| panic!("{chapter}: nothing marked {what:?} ({:?})", all(chapter)))
}

/// Every mark of the run named `chapter`, in order.
#[cfg(test)]
pub fn all(chapter: &str) -> Vec<(&'static str, f64)> {
    log::MARKS
        .lock()
        .map(|m| {
            m.iter()
                .filter(|(c, ..)| c == chapter)
                .map(|(_, w, t)| (*w, *t))
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod log {
    use std::sync::Mutex;

    pub static MARKS: Mutex<Vec<(String, &'static str, f64)>> = Mutex::new(Vec::new());
}
