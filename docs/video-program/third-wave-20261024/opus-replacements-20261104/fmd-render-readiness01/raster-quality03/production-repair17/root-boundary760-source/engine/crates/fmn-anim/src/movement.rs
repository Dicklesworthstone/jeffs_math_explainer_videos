//! The functional-map mechanism — family 5 (§9.4, fm-cye), ported from
//! the pinned Reference's `animation/movement.py`:
//!
//! - [`Homotopy`]: per zipped pair, restore the pointlike data from the
//!   start, then map every point through `(x, y, z, t) ↦ (x', y', z')` at
//!   `t = alpha` (movement.py:37 — `match_points` + `apply_function`,
//!   whose default pivot is the true origin). [`smoothed_homotopy`] adds
//!   the `make_smooth` re-handling pass (movement.py:52);
//!   [`complex_homotopy`] is the `(z, t) ↦ w` plane map over `(x, y)`
//!   with `z` carried through (movement.py:56).
//! - [`PhaseFlow`] (movement.py:75): forward-Euler advection
//!   `p ↦ p + Δt·f(p)` with `Δt = virtual_time·(α − α_prev)` — the state
//!   is *path-dependent by design* (the Reference's `last_alpha` memo),
//!   so its segments are conservatively stateful. The Reference's
//!   overridden `interpolate_mobject` never consults `rate_func` or
//!   `time_span`; ported verbatim (the `linear` default makes it moot).
//! - [`MoveAlongPath`] (movement.py:104): `move_to` the point `α` of the
//!   way along a path — **by true arc length** (BN-03, the W2 layer),
//!   where the Reference rides `quick_point_from_proportion`'s
//!   equal-curve-length approximation. Constant speed under the original
//!   name is the Behavior-Noted improvement this program exists for.
//!
//! User-supplied closures are not provable, so every closure-carrying
//! animation here keeps the default Unclassified signature and the R20
//! demotion — frame-parallel eligibility is never guessed.

use fmn_core::types::Vec3;
use fmn_mobject::{Mob, Stage};

use crate::animation::{AnimConfig, AnimError, AnimState, Animation, RateFunc};

// -------------------------------------------------------------- Homotopy

/// `Homotopy` (movement.py:17): an `(x, y, z, t) → (x', y', z')` map
/// animated over the family.
pub struct Homotopy {
    state: AnimState,
    homotopy: Box<dyn Fn(f64, f64, f64, f64) -> Vec3>,
    smoothed: bool,
}

impl std::fmt::Debug for Homotopy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Homotopy")
            .field("state", &self.state)
            .field("smoothed", &self.smoothed)
            .finish_non_exhaustive()
    }
}

impl Homotopy {
    /// `Homotopy(homotopy, mobject)` — Reference default `run_time = 3`.
    #[must_use]
    pub fn new(homotopy: impl Fn(f64, f64, f64, f64) -> Vec3 + 'static, mobject: Mob) -> Self {
        let config = AnimConfig {
            name: "Homotopy".to_owned(),
            run_time: 3.0,
            ..AnimConfig::default()
        };
        Self {
            state: AnimState::new(mobject, config),
            homotopy: Box::new(homotopy),
            smoothed: false,
        }
    }

    /// Replace the animation config.
    #[must_use]
    pub fn with_config(mut self, config: AnimConfig) -> Self {
        self.state.config = config;
        self
    }
}

impl Animation for Homotopy {
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

    /// movement.py:37: `match_points(start)`, then the map at `t = alpha`
    /// applied about the true origin (`apply_function`'s default pivot).
    fn interpolate_submobject(&mut self, stage: &mut Stage, mobs: &[Mob], sub_alpha: f64) {
        let [submob, start] = *mobs else {
            return; // rows are pairs once begin has run
        };
        stage.match_points(submob, start);
        let f = &self.homotopy;
        // Per-member map: the rows already cover every family member, so
        // the member-local application lands the Reference's final state
        // without its redundant subtree re-application.
        // This hook is already called once per family member. Read its
        // world-space points and bake the arbitrary map into that member only;
        // calling the family-wide `apply_points_function` here would remap
        // descendants repeatedly.
        if let Some(points) = stage.get_points(submob) {
            let mapped: Vec<Vec3> = points
                .into_iter()
                .map(|point| f(point[0], point[1], point[2], sub_alpha))
                .collect();
            let _ = stage.set_points(submob, &mapped);
        }
        if self.smoothed {
            let _ = stage.make_family_smooth(submob);
        }
    }
}

/// `SmoothedVectorizedHomotopy` (movement.py:52): the same map with the
/// approximate-smooth re-handling pass after each application.
#[must_use]
pub fn smoothed_homotopy(
    homotopy: impl Fn(f64, f64, f64, f64) -> Vec3 + 'static,
    mobject: Mob,
) -> Homotopy {
    let mut anim = Homotopy::new(homotopy, mobject);
    anim.state.config.name = "SmoothedVectorizedHomotopy".to_owned();
    anim.smoothed = true;
    anim
}

/// `ComplexHomotopy` (movement.py:56): `(z, t) ↦ w` over the `(x, y)`
/// plane, `z` carried through — the closure receives `(re, im, t)` and
/// returns `(re', im')`.
#[must_use]
pub fn complex_homotopy(
    map: impl Fn(f64, f64, f64) -> (f64, f64) + 'static,
    mobject: Mob,
) -> Homotopy {
    let mut anim = Homotopy::new(
        move |x, y, z, t| {
            let (re, im) = map(x, y, t);
            [re, im, z]
        },
        mobject,
    );
    anim.state.config.name = "ComplexHomotopy".to_owned();
    anim
}

// ------------------------------------------------------------- PhaseFlow

/// `PhaseFlow` (movement.py:75): advect the family along a vector field
/// by forward Euler between consecutive alphas.
pub struct PhaseFlow {
    state: AnimState,
    function: Box<dyn Fn(Vec3) -> Vec3>,
    virtual_time: f64,
    last_alpha: Option<f64>,
}

impl std::fmt::Debug for PhaseFlow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PhaseFlow")
            .field("state", &self.state)
            .field("virtual_time", &self.virtual_time)
            .field("last_alpha", &self.last_alpha)
            .finish_non_exhaustive()
    }
}

impl PhaseFlow {
    /// `PhaseFlow(function, mobject)` — Reference defaults `run_time = 3`,
    /// `rate_func = linear`, `virtual_time = run_time` when unset.
    #[must_use]
    pub fn new(
        function: impl Fn(Vec3) -> Vec3 + 'static,
        mobject: Mob,
        virtual_time: Option<f64>,
    ) -> Self {
        let run_time = 3.0;
        let config = AnimConfig {
            name: "PhaseFlow".to_owned(),
            run_time,
            rate_func: RateFunc::linear(),
            ..AnimConfig::default()
        };
        Self {
            state: AnimState::new(mobject, config),
            function: Box::new(function),
            virtual_time: virtual_time.unwrap_or(run_time),
            last_alpha: None,
        }
    }

    /// Replace the animation config. (`virtual_time` keeps its
    /// construction-time value — the Reference binds it before kwargs
    /// apply, quirk kept.)
    #[must_use]
    pub fn with_config(mut self, config: AnimConfig) -> Self {
        self.state.config = config;
        self
    }
}

impl Animation for PhaseFlow {
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
        self.last_alpha = None;
        Ok(())
    }

    /// movement.py:96 verbatim: raw alpha, no rate/lag/time-span; the
    /// first call only records `last_alpha`.
    fn interpolate(&mut self, stage: &mut Stage, alpha: f64) {
        if let Some(last) = self.last_alpha {
            let dt = self.virtual_time * (alpha - last);
            let f = &self.function;
            stage.apply_points_function(
                self.state.mobject(),
                move |p| {
                    let v = f(p);
                    [p[0] + dt * v[0], p[1] + dt * v[1], p[2] + dt * v[2]]
                },
                Some([0.0, 0.0, 0.0]),
                None,
            );
        }
        self.last_alpha = Some(alpha);
    }

    fn interpolate_submobject(&mut self, _stage: &mut Stage, _mobs: &[Mob], _sub_alpha: f64) {
        // Unreachable: interpolate is overridden and never zips families.
    }
}

// ---------------------------------------------------------- MoveAlongPath

/// `MoveAlongPath` (movement.py:104): `move_to` the point `α` of the way
/// along `path` — constant-speed by true arc length (BN-03).
#[derive(Debug, Clone)]
pub struct MoveAlongPath {
    state: AnimState,
    path: Mob,
}

impl MoveAlongPath {
    /// `MoveAlongPath(mobject, path)`.
    #[must_use]
    pub fn new(mobject: Mob, path: Mob) -> Self {
        let config = AnimConfig {
            name: "MoveAlongPath".to_owned(),
            ..AnimConfig::default()
        };
        Self {
            state: AnimState::new(mobject, config),
            path,
        }
    }

    /// Replace the animation config.
    #[must_use]
    pub fn with_config(mut self, config: AnimConfig) -> Self {
        self.state.config = config;
        self
    }
}

impl Animation for MoveAlongPath {
    fn state(&self) -> &AnimState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut AnimState {
        &mut self.state
    }

    /// A pointless path is the typed refusal (the Reference indexes into
    /// an empty array at the first frame).
    fn setup(&mut self, stage: &mut Stage) -> Result<(), AnimError> {
        let mobject = self.state.mobject();
        if !stage.contains(mobject) {
            return Err(AnimError::StaleHandle(mobject));
        }
        if !stage.contains(self.path) {
            return Err(AnimError::StaleHandle(self.path));
        }
        if stage.point_from_proportion(self.path, 0.0).is_err() {
            return Err(AnimError::EmptyMobject);
        }
        Ok(())
    }

    fn preflight_mobjects(&self) -> Vec<Mob> {
        vec![self.state.mobject(), self.path]
    }

    /// movement.py:115 with the BN-03 improvement: the rate function
    /// applies to the raw alpha (the Reference's override bypasses lag and
    /// time-span), and the sample point is taken by true arc length.
    fn interpolate(&mut self, stage: &mut Stage, alpha: f64) {
        let rated = self.state.config.rate_func.eval(alpha);
        if let Ok(point) = stage.point_from_proportion(self.path, rated) {
            stage.move_to(self.state.mobject(), point, [0.0, 0.0, 0.0]);
        }
    }

    fn interpolate_submobject(&mut self, _stage: &mut Stage, _mobs: &[Mob], _sub_alpha: f64) {
        // Unreachable: interpolate is overridden and never zips families.
    }
}
