//! Hoeffding's view of the shared kit (`explainer_kit`): every helper the
//! chapters use, this video's palette roles, and the v5 score hooks.
//!
//! The chapters were written against the v5 kit, whose API the shared kit
//! keeps, so they only import from here. v6 is scored by the score engine:
//! [`Kit::chime`] marks a reveal for it to answer, and [`Kit::pad`], which
//! every chapter calls last, ends the chapter for the shared kit (`finish`:
//! the last line's hold, the score stem if one was given, and the chapter's
//! timeline). With `HOEFFDING_V5_SOUND=1` (the A/B's old side) the v5 pad
//! and chimes play instead, placed through the narrator so the exact cue log
//! (and the PCM master's replay of it) has them.

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

/// The music's own breath at each act's end (plan §7.2: after h7, g10, q8,
/// c8, f8 and s9), before the closing fade.
pub const ACT_BREATH: f64 = 2.5;

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
    /// play only when asked for (`HOEFFDING_V5_SOUND=1`), under a real
    /// narration, and never with a score stem.
    pub fn wrap(base: explainer_kit::Kit) -> Self {
        let v5 = base
            .narrator
            .as_ref()
            .filter(|n| crate::v5_sound() && n.has_audio() && base.stem.is_none())
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

    /// A sonification in its own listen window (see
    /// `explainer_kit::Kit::sonify`). The v5 sound never had one, so with it
    /// the window is recorded but the sound is not played: the A/B's old
    /// side stays v5's.
    pub fn sonify(
        &self,
        stage: &mut Stage<'_>,
        seconds: f64,
        label: &str,
        wav: &Path,
    ) -> fmn::Result<()> {
        match &self.v5 {
            Some((_, narrator)) => narrator.sonify(stage, seconds, label, wav),
            None => self.base.sonify(stage, seconds, label, wav),
        }
    }

    /// A reveal now: a mark the score answers (a soft dyad in the next gap),
    /// or, for the v5 sound, a soft bell.
    pub fn chime(&self, stage: &mut Stage<'_>, freq: f64) -> fmn::Result<()> {
        let Some((score, narrator)) = &self.v5 else {
            self.base
                .mark(stage, &format!("reveal:chime_{}", freq.round()));
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
