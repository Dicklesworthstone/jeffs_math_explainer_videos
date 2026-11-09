//! Update-driven animations, ported from the pinned Reference's
//! `animation/update.py`: closures run against the live stage each frame.
//! All three are inherently effectful (arbitrary user code, or reads of
//! concurrently animated state), so they keep the Unclassified signature
//! and the R20 demotion — their segments never frame-parallelize.
//!
//! `numbers.py`'s `ChangingDecimal`/`ChangeDecimalToValue`/`CountInFrom`
//! are this mechanism pointed at `DecimalNumber.set_value` — they land
//! with the de-TeX'd `DecimalNumber` (fmn-library, §12.4); the seam is
//! recorded on the fm-cye bead.

use fmn_core::types::Vec3;
use fmn_mobject::{Mob, Stage};

use crate::animation::{AnimConfig, AnimError, AnimState, Animation, time_spanned_alpha};

/// The boxed update closure: `(stage, mobject, true_alpha)`.
type UpdateClosure = Box<dyn FnMut(&mut Stage, Mob, f64)>;

/// `UpdateFromFunc` (update.py:13) and, alpha-aware,
/// `UpdateFromAlphaFunc` (update.py:37): run a stage closure against the
/// mobject each frame.
pub struct UpdateFromFunc {
    state: AnimState,
    update: UpdateClosure,
    alpha_aware: bool,
}

impl std::fmt::Debug for UpdateFromFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UpdateFromFunc")
            .field("state", &self.state)
            .field("alpha_aware", &self.alpha_aware)
            .finish_non_exhaustive()
    }
}

impl UpdateFromFunc {
    /// `UpdateFromFunc(mobject, update_function)`.
    #[must_use]
    pub fn new(mobject: Mob, update: impl FnMut(&mut Stage, Mob) + 'static) -> Self {
        let mut update = update;
        Self {
            state: AnimState::new(
                mobject,
                AnimConfig {
                    name: "UpdateFromFunc".to_owned(),
                    ..AnimConfig::default()
                },
            ),
            update: Box::new(move |stage, mob, _| update(stage, mob)),
            alpha_aware: false,
        }
    }

    /// `UpdateFromAlphaFunc(mobject, update_function)`: the closure also
    /// receives `rate(time_spanned(α))` (update.py:50's `true_alpha`).
    #[must_use]
    pub fn new_alpha(mobject: Mob, update: impl FnMut(&mut Stage, Mob, f64) + 'static) -> Self {
        Self {
            state: AnimState::new(
                mobject,
                AnimConfig {
                    name: "UpdateFromAlphaFunc".to_owned(),
                    ..AnimConfig::default()
                },
            ),
            update: Box::new(update),
            alpha_aware: true,
        }
    }

    /// Replace the animation config.
    #[must_use]
    pub fn with_config(mut self, config: AnimConfig) -> Self {
        self.state.config = config;
        self
    }
}

impl Animation for UpdateFromFunc {
    fn state(&self) -> &AnimState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut AnimState {
        &mut self.state
    }

    fn setup(&mut self, stage: &mut Stage) -> Result<(), AnimError> {
        let mobject = self.state.mobject();
        if !stage.contains(mobject) {
            return Err(AnimError::StaleHandle(mobject));
        }
        Ok(())
    }

    /// update.py:33/49: the plain variant ignores alpha entirely; the
    /// alpha variant passes `rate(time_spanned(α))`. Lag never applies.
    fn interpolate(&mut self, stage: &mut Stage, alpha: f64) {
        let true_alpha = if self.alpha_aware {
            let config = &self.state.config;
            config
                .rate_func
                .eval(time_spanned_alpha(alpha, config.run_time, config.time_span))
        } else {
            alpha
        };
        (self.update)(stage, self.state.mobject(), true_alpha);
    }

    fn interpolate_submobject(&mut self, _stage: &mut Stage, _mobs: &[Mob], _sub_alpha: f64) {
        // Unreachable: interpolate is overridden and never zips families.
    }
}

/// `MaintainPositionRelativeTo` (update.py:55): hold the
/// construction-time center offset against a tracked mobject.
#[derive(Debug, Clone)]
pub struct MaintainPositionRelativeTo {
    state: AnimState,
    tracked: Mob,
    diff: Vec3,
}

impl MaintainPositionRelativeTo {
    /// Captures `mobject.get_center() − tracked.get_center()` now, holds
    /// it each frame.
    #[must_use]
    pub fn new(stage: &Stage, mobject: Mob, tracked: Mob) -> Self {
        let a = stage.get_center(mobject);
        let b = stage.get_center(tracked);
        Self {
            state: AnimState::new(
                mobject,
                AnimConfig {
                    name: "MaintainPositionRelativeTo".to_owned(),
                    ..AnimConfig::default()
                },
            ),
            tracked,
            diff: [a[0] - b[0], a[1] - b[1], a[2] - b[2]],
        }
    }

    /// Replace the animation config.
    #[must_use]
    pub fn with_config(mut self, config: AnimConfig) -> Self {
        self.state.config = config;
        self
    }
}

impl Animation for MaintainPositionRelativeTo {
    fn state(&self) -> &AnimState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut AnimState {
        &mut self.state
    }

    fn setup(&mut self, stage: &mut Stage) -> Result<(), AnimError> {
        for mob in [self.state.mobject(), self.tracked] {
            if !stage.contains(mob) {
                return Err(AnimError::StaleHandle(mob));
            }
        }
        Ok(())
    }

    fn preflight_mobjects(&self) -> Vec<Mob> {
        vec![self.state.mobject(), self.tracked]
    }

    /// update.py:66: shift so the tracked offset holds. Reads the tracked
    /// mobject *live* — which is the point, and why it stays Unclassified.
    fn interpolate(&mut self, stage: &mut Stage, _alpha: f64) {
        let target = stage.get_center(self.tracked);
        let location = stage.get_center(self.state.mobject());
        stage.shift(
            self.state.mobject(),
            [
                target[0] - location[0] + self.diff[0],
                target[1] - location[1] + self.diff[1],
                target[2] - location[2] + self.diff[2],
            ],
        );
    }

    fn interpolate_submobject(&mut self, _stage: &mut Stage, _mobs: &[Mob], _sub_alpha: f64) {
        // Unreachable: interpolate is overridden and never zips families.
    }
}
