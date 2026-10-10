//! Hoeffding's view of the shared kit (`explainer_kit`): every helper the
//! chapters use, this video's palette roles, and the v5 score hooks.
//!
//! The chapters were written against the v5 kit, whose API the shared kit
//! keeps, so they only import from here. Until the score engine replaces the
//! v5 pad and chimes (bead W3-score), [`Kit::pad`] and [`Kit::chime`] still
//! play them under a real narration, placed through the narrator so the exact
//! cue log (and the PCM master's replay of it) has them. `pad`, which every
//! chapter calls last, also ends the chapter for the shared kit (`finish`:
//! the last line's hold, a score stem if one was given, and the chapter's
//! timeline).

use std::ops::Deref;
use std::path::Path;
use std::rc::Rc;

use explainer_kit::Narrator;
use fmn::prelude::*;

pub use explainer_kit::{
    Frame2, Timed, add, children, dot_cloud, fade_all, hex, lagged, live_bar, point_colors, rect,
    stroke_style, tween, v,
};

use crate::sound::Score;

/// Roles, not hues: every chapter colors the same idea the same way.
pub struct Palette;
impl Palette {
    pub fn ink() -> Srgb {
        hex("#ECEFF4")
    }
    pub fn dim() -> Srgb {
        hex("#8A93A3")
    }
    pub fn faint() -> Srgb {
        hex("#3B4352")
    }
    /// R: 3b1b BLUE_C.
    pub fn x_rank() -> Srgb {
        hex("#58C4DD")
    }
    /// S: 3b1b GREEN_C.
    pub fn y_rank() -> Srgb {
        hex("#83C167")
    }
    /// Q: YELLOW.
    pub fn q() -> Srgb {
        hex("#FFFF00")
    }
    /// TEAL_C.
    pub fn d1() -> Srgb {
        hex("#5CD0B3")
    }
    /// GOLD_C.
    pub fn d2() -> Srgb {
        hex("#F0AC5F")
    }
    /// RED_C.
    pub fn d3() -> Srgb {
        hex("#FC6255")
    }
    pub fn hoeffding() -> Srgb {
        hex("#FFD866")
    }
    /// PURPLE_C.
    pub fn pearson() -> Srgb {
        hex("#9A72AC")
    }
    pub fn spearman() -> Srgb {
        hex("#58C4DD")
    }
    pub fn kendall() -> Srgb {
        hex("#E07A5F")
    }
    pub fn warn() -> Srgb {
        hex("#FC6255")
    }
    pub fn good() -> Srgb {
        hex("#83C167")
    }
}

/// The shared kit plus the v5 score hooks.
#[derive(Clone)]
pub struct Kit {
    base: explainer_kit::Kit,
    v5: Option<(Score, Rc<Narrator>)>,
}

impl Deref for Kit {
    type Target = explainer_kit::Kit;
    fn deref(&self) -> &explainer_kit::Kit {
        &self.base
    }
}

impl Kit {
    /// Wrap the kit the shared CLI built for a chapter. The v5 pad and chimes
    /// play only under a real narration (v5 also played them under the
    /// scratch voice; drafts lose them until the score engine takes over),
    /// and never with a score stem, which replaces them.
    pub fn wrap(base: explainer_kit::Kit) -> Self {
        let v5 = base
            .narrator
            .as_ref()
            .filter(|n| n.has_audio() && base.stem.is_none())
            .map(|n| {
                let score = Score {
                    dir: std::env::temp_dir().join("hoeffding_v5_score"),
                    under_voice: true,
                };
                (score, Rc::clone(n))
            });
        Self { base, v5 }
    }

    /// End the chapter (call it last): the shared kit's `finish`, then the v5
    /// pad laid under the whole scene.
    pub fn pad(&self, stage: &mut Stage<'_>, chapter: usize) -> fmn::Result<()> {
        self.base.finish(stage)?;
        let Some((score, narrator)) = &self.v5 else {
            return Ok(());
        };
        let seconds = stage.scene().time().to_f64();
        match create_dir(&score.dir).and_then(|()| score.pad_wav(seconds, chapter)) {
            Ok(Some((path, gain))) => place(stage, narrator, &path, -seconds, gain),
            Ok(None) => Ok(()),
            Err(e) => Err(scene_error(format!("v5 pad: {e}"))),
        }
    }

    /// A soft v5 bell now.
    pub fn chime(&self, stage: &mut Stage<'_>, freq: f64) -> fmn::Result<()> {
        let Some((score, narrator)) = &self.v5 else {
            return Ok(());
        };
        match create_dir(&score.dir).and_then(|()| score.chime_wav(freq)) {
            Ok((path, gain)) => place(stage, narrator, &path, 0.0, gain),
            Err(e) => Err(scene_error(format!("v5 chime: {e}"))),
        }
    }
}

fn create_dir(dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)
}

fn place(
    stage: &mut Stage<'_>,
    narrator: &Narrator,
    path: &Path,
    offset: f64,
    gain: f64,
) -> fmn::Result<()> {
    narrator
        .sound(stage, path, offset, Some(gain))
        .map_err(|e| scene_error(format!("{}: {e}", path.display())))
}

fn scene_error(m: String) -> fmn::Error {
    fmn::Error::Scene(SceneError::InvalidConfig(Box::leak(m.into_boxed_str())))
}
