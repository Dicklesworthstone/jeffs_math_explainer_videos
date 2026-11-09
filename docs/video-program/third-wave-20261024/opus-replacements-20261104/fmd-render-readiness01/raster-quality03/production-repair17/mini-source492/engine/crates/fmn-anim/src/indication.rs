//! The indication mechanism family — family 4 (§9.4, fm-cye), ported
//! from the pinned Reference's `animation/indication.py`. The classes
//! that are pure mechanism land here:
//!
//! - [`indicate`] (indication.py:73): a Transform whose target is a
//!   scaled, recolored copy under `there_and_back` — the mobject swells,
//!   flushes, and returns.
//! - [`focus_on`] / [`circle_indicate`] accept Atlas-built source/target
//!   geometry and own the indication-specific Transform configuration;
//!   [`flash`] and [`show_creation_then_fade_out`] similarly own the
//!   composition identity over already-built mechanism members. Keeping
//!   geometry construction above Choreo preserves the crate DAG.
//! - [`WiggleOutThenIn`] (indication.py:357): per frame, restore the
//!   pointlike data and apply an absolute scale-and-rotate about fixed
//!   pivots (`there_and_back` for the swell, the `wiggle` rate for the
//!   rocking) — closure-free and provably pure.
//! - [`turn_inside_out`] (indication.py:401): transform onto a
//!   points-reversed copy along a 90° arc. The Reference calls the
//!   nonexistent `refresh_triangulation` (defect C-1, Appendix C: *fixed —
//!   the evident intent implemented*): FrankenManim's renderer has no
//!   triangulation to refresh, and the record-revision bump the reversal
//!   causes is the intent — caches can never serve the stale orientation.
//! - [`VShowPassingFlash`] (indication.py:184): a gaussian stroke-width
//!   window (σ = time_width/6, ±3σ support) over per-member width
//!   profiles tapered at the ends, swept across the path; `finish`
//!   restores every member's style. [`show_creation_then_destruction`]
//!   (indication.py:284) parameterizes the plain passing flash at
//!   `time_width = 2`.
//! - [`apply_wave`] (indication.py:333): the Homotopy whose nudge is
//!   `there_and_back(t^power)·amplitude·direction`, power sliding with
//!   the x-proportion — functional-map family under an indication name.
//!
//! The remaining geometry-bound classes land with their dependencies (the
//! seams the fm-cye bead records): `FlashAround`/`FlashUnder` and the
//! `AnimationOnSurroundingRectangle` family need `SurroundingRectangle`/
//! `Underline` (fmn-library, §12). [`flashy_fade_in`] composes the fade and
//! passing-flash mechanisms; [`broadcast`] wires caller-built restores.

use fmn_core::rate;
use fmn_core::types::Vec3;
use fmn_mobject::{Mob, Stage};

use crate::animation::{AnimConfig, AnimError, AnimState, Animation, AnimationSignature, RateFunc};
use crate::composition::{AnimationGroup, Succession};
use crate::creation::match_style_from;
use crate::movement::Homotopy;
use crate::transform::{TargetPrep, Transform};

/// The Reference's `YELLOW` (`#FFFF00`), the indication default color.
pub const INDICATION_YELLOW: [f32; 3] = [1.0, 1.0, 0.0];

/// `FocusOn` (indication.py:37): transform the Atlas-built full-frame,
/// transparent dot onto the zero-radius focus dot. Geometry and style are
/// supplied by the caller because Choreo cannot depend upward on Atlas.
/// Reference defaults owned here: `run_time = 2`, remover.
#[must_use]
pub fn focus_on(starting_dot: Mob, focus_dot: Mob, remover: bool) -> Transform {
    let mut transform = Transform::new(starting_dot, focus_dot);
    transform.state_mut().config.name = "FocusOn".to_owned();
    transform.state_mut().config.run_time = 2.0;
    transform.state_mut().config.remover = remover;
    transform
}

/// `Indicate` (indication.py:73): transform onto a copy scaled by
/// `scale_factor` and recolored, under `there_and_back`. Reference
/// defaults: `scale_factor = 1.2`, `color = YELLOW`. Geometry, placement and
/// paint are sampled when the animation begins, including on replay.
///
/// # Errors
/// [`AnimError::StaleHandle`] / [`AnimError::Stage`].
pub fn indicate(
    stage: &mut Stage,
    mobject: Mob,
    scale_factor: f64,
    color: Option<[f32; 3]>,
) -> Result<Transform, AnimError> {
    if !stage.contains(mobject) {
        return Err(AnimError::StaleHandle(mobject));
    }
    let mut t = Transform::new(mobject, mobject).with_target_prep(TargetPrep::Indicate {
        scale: scale_factor,
        color: color.unwrap_or(INDICATION_YELLOW),
    });
    t.state_mut().config.name = "Indicate".to_owned();
    t.state_mut().config.rate_func = RateFunc::Base(rate::there_and_back);
    Ok(t)
}

/// `Flash` (indication.py:85): simultaneous creation-then-destruction
/// members over radial lines built by Atlas.
///
/// # Errors
/// [`AnimError::EmptyComposition`] for no line animations, or the Stage
/// errors reported while assembling the member container.
pub fn flash(
    stage: &mut Stage,
    line_animations: Vec<Box<dyn Animation>>,
    lag_ratio: f64,
) -> Result<AnimationGroup, AnimError> {
    let mut group = AnimationGroup::with_lag_ratio(stage, line_animations, lag_ratio)?;
    group.state_mut().config.name = "Flash".to_owned();
    group.state_mut().config.run_time = 1.0;
    Ok(group)
}

/// `CircleIndicate` (indication.py:130): transform the Atlas-built narrow,
/// zero-stroke circle onto its surrounding-circle target under
/// `there_and_back` and remove the auxiliary geometry by default.
#[must_use]
pub fn circle_indicate(pre_circle: Mob, circle: Mob, remover: bool) -> Transform {
    let mut transform = Transform::new(pre_circle, circle);
    transform.state_mut().config.name = "CircleIndicate".to_owned();
    transform.state_mut().config.rate_func = RateFunc::Base(rate::there_and_back);
    transform.state_mut().config.remover = remover;
    transform
}

/// `TurnInsideOut` (indication.py:401): transform onto a points-reversed
/// copy along a `path_arc` (default 90°). C-1's ruling applies — see the
/// module docs. The reversed target is rebuilt from the live source at begin.
///
/// # Errors
/// [`AnimError::StaleHandle`] / [`AnimError::Stage`].
pub fn turn_inside_out(
    stage: &mut Stage,
    mobject: Mob,
    path_arc: f64,
) -> Result<Transform, AnimError> {
    if !stage.contains(mobject) {
        return Err(AnimError::StaleHandle(mobject));
    }
    let mut t = Transform::new(mobject, mobject)
        .with_target_prep(TargetPrep::ReversePoints)
        .with_path_arc(path_arc, [0.0, 0.0, 1.0]);
    t.state_mut().config.name = "TurnInsideOut".to_owned();
    Ok(t)
}

// ------------------------------------------------------ WiggleOutThenIn

/// `WiggleOutThenIn` (indication.py:357): swell by `there_and_back`,
/// rock by the `wiggle` rate — absolute pose per frame. Reference
/// defaults: `scale_value = 1.1`, `rotation_angle = 0.01·TAU`,
/// `n_wiggles = 6`, `run_time = 2`.
#[derive(Debug, Clone)]
pub struct WiggleOutThenIn {
    state: AnimState,
    scale_value: f64,
    rotation_angle: f64,
    n_wiggles: f64,
    scale_about_point: Option<Vec3>,
    rotate_about_point: Option<Vec3>,
}

impl WiggleOutThenIn {
    /// Reference defaults.
    #[must_use]
    pub fn new(mobject: Mob) -> Self {
        let config = AnimConfig {
            name: "WiggleOutThenIn".to_owned(),
            run_time: 2.0,
            ..AnimConfig::default()
        };
        Self {
            state: AnimState::new(mobject, config),
            scale_value: 1.1,
            rotation_angle: 0.01 * fmn_core::constants::TAU,
            n_wiggles: 6.0,
            scale_about_point: None,
            rotate_about_point: None,
        }
    }

    /// The swell factor.
    #[must_use]
    pub fn with_scale_value(mut self, value: f64) -> Self {
        self.scale_value = value;
        self
    }

    /// The rocking amplitude in radians.
    #[must_use]
    pub fn with_rotation_angle(mut self, angle: f64) -> Self {
        self.rotation_angle = angle;
        self
    }

    /// The wiggle count.
    #[must_use]
    pub fn with_n_wiggles(mut self, n: f64) -> Self {
        self.n_wiggles = n;
        self
    }

    /// Fix the swell pivot instead of using the restored family centre.
    #[must_use]
    pub fn with_scale_about_point(mut self, point: Vec3) -> Self {
        self.scale_about_point = Some(point);
        self
    }

    /// Fix the rocking pivot instead of using the restored family centre.
    #[must_use]
    pub fn with_rotate_about_point(mut self, point: Vec3) -> Self {
        self.rotate_about_point = Some(point);
        self
    }

    /// Replace the animation config.
    #[must_use]
    pub fn with_config(mut self, config: AnimConfig) -> Self {
        self.state.config = config;
        self
    }
}

impl Animation for WiggleOutThenIn {
    fn state(&self) -> &AnimState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut AnimState {
        &mut self.state
    }

    /// Pure: restore-then-transform about pivots that are functions of
    /// the begin state (the default pivots read the restored center).
    fn effect_signature(&self) -> AnimationSignature {
        AnimationSignature::Pure
    }

    fn setup(&mut self, stage: &mut Stage) -> Result<(), AnimError> {
        let mobject = self.state.mobject();
        if !stage.contains(mobject) {
            return Err(AnimError::StaleHandle(mobject));
        }
        Ok(())
    }

    /// indication.py:384: `match_points(start)`, scale by
    /// `interpolate(1, scale_value, there_and_back(α))`, rotate by
    /// `wiggle(α, n)·rotation_angle`, each about its pivot.
    fn interpolate_submobject(&mut self, stage: &mut Stage, mobs: &[Mob], sub_alpha: f64) {
        let [submob, start] = *mobs else {
            return; // rows are pairs once begin has run
        };
        stage.match_points(submob, start);
        let mobject = self.state.mobject();
        let scale_pivot = self
            .scale_about_point
            .unwrap_or_else(|| stage.get_center(mobject));
        let rotate_pivot = self
            .rotate_about_point
            .unwrap_or_else(|| stage.get_center(mobject));
        let swell = rate::there_and_back(sub_alpha);
        let factor = (1.0 - swell) + swell * self.scale_value;
        stage.scale_about(submob, factor, Some(scale_pivot), None);
        stage.rotate(
            submob,
            rate::wiggle(sub_alpha, self.n_wiggles) * self.rotation_angle,
            [0.0, 0.0, 1.0],
            Some(rotate_pivot),
            None,
        );
    }
}

// ----------------------------------------------------- VShowPassingFlash

/// `VShowPassingFlash` (indication.py:184): sweep a gaussian
/// stroke-width window over tapered per-member width profiles. Reference
/// defaults: `time_width = 0.3`, `taper_width = 0.05`, remover.
#[derive(Debug, Clone)]
pub struct VShowPassingFlash {
    state: AnimState,
    time_width: f64,
    taper_width: f64,
    widths: Vec<(Mob, Vec<f64>)>,
}

impl VShowPassingFlash {
    /// Reference defaults.
    #[must_use]
    pub fn new(vmobject: Mob) -> Self {
        let config = AnimConfig {
            name: "VShowPassingFlash".to_owned(),
            remover: true,
            ..AnimConfig::default()
        };
        Self {
            state: AnimState::new(vmobject, config),
            time_width: 0.3,
            taper_width: 0.05,
            widths: Vec::new(),
        }
    }

    /// The gaussian window's width in proportion units.
    #[must_use]
    pub fn with_time_width(mut self, time_width: f64) -> Self {
        self.time_width = time_width;
        self
    }

    /// The end-taper width in proportion units.
    #[must_use]
    pub fn with_taper_width(mut self, taper_width: f64) -> Self {
        self.taper_width = taper_width;
        self
    }

    /// Replace the animation config.
    #[must_use]
    pub fn with_config(mut self, config: AnimConfig) -> Self {
        self.state.config = config;
        self
    }

    /// indication.py:198 `taper_kernel`.
    fn taper_kernel(&self, x: f64) -> f64 {
        if x < self.taper_width {
            x
        } else if x > 1.0 - self.taper_width {
            1.0 - x
        } else {
            1.0
        }
    }
}

impl Animation for VShowPassingFlash {
    fn state(&self) -> &AnimState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut AnimState {
        &mut self.state
    }

    /// Pure: the width profiles freeze at begin; each frame's widths are
    /// `profile·gaussian(α)`.
    fn effect_signature(&self) -> AnimationSignature {
        AnimationSignature::Pure
    }

    /// indication.py:205 `begin`: snapshot each member's tapered width
    /// profile before the canonical sequence runs.
    fn setup(&mut self, stage: &mut Stage) -> Result<(), AnimError> {
        let mobject = self.state.mobject();
        if !stage.contains(mobject) {
            return Err(AnimError::StaleHandle(mobject));
        }
        self.widths.clear();
        for member in stage.family(mobject) {
            let Some(entry) = stage.get(member) else {
                continue;
            };
            let Some(widths) = entry.buffer.read_column("stroke_width") else {
                continue;
            };
            let n = widths.len();
            let profile: Vec<f64> = widths
                .iter()
                .enumerate()
                .map(|(i, &w)| {
                    let x = if n > 1 {
                        i as f64 / (n as f64 - 1.0)
                    } else {
                        0.0
                    };
                    f64::from(w) * self.taper_kernel(x)
                })
                .collect();
            self.widths.push((member, profile));
        }
        Ok(())
    }

    /// indication.py:222: a gaussian with 3σ = time_width/2 swept from
    /// `−tw/2` to `1 + tw/2`, zeroed outside its support.
    fn interpolate_submobject(&mut self, stage: &mut Stage, mobs: &[Mob], sub_alpha: f64) {
        let [submob, _start] = *mobs else {
            return; // rows are pairs once begin has run
        };
        let Some((_, profile)) = self.widths.iter().find(|(m, _)| *m == submob) else {
            return;
        };
        let tw = self.time_width;
        let sigma = tw / 6.0;
        let mu = (1.0 - sub_alpha) * (-tw / 2.0) + sub_alpha * (1.0 + tw / 2.0);
        let n = profile.len();
        #[allow(clippy::cast_possible_truncation)]
        let out: Vec<f32> = profile
            .iter()
            .enumerate()
            .map(|(i, &w)| {
                let x = if n > 1 {
                    i as f64 / (n as f64 - 1.0)
                } else {
                    0.0
                };
                if (x - mu).abs() > 3.0 * sigma {
                    return 0.0;
                }
                let z = (x - mu) / sigma;
                (w * fmn_dmath::exp(-0.5 * z * z)) as f32
            })
            .collect();
        if !out.is_empty()
            && let Some(entry) = stage.get_mut(submob)
        {
            entry.buffer.write_range("stroke_width", 0, &out);
        }
    }

    /// indication.py:249 `finish`: every member's style returns to the
    /// start's.
    fn teardown(&mut self, stage: &mut Stage) {
        if let Some(starting) = self.state.starting_mobject() {
            match_style_from(stage, self.state.mobject(), starting);
        }
    }
}

/// `ShowCreationThenDestruction` (indication.py:284): the plain passing
/// flash at `time_width = 2`.
#[must_use]
pub fn show_creation_then_destruction(vmobject: Mob) -> crate::creation::ShowPartial {
    let mut anim = crate::creation::show_passing_flash(vmobject, 2.0);
    anim.state_mut().config.name = "ShowCreationThenDestruction".to_owned();
    anim
}

/// `ShowCreationThenFadeOut` (indication.py:266): the supplied
/// `ShowCreation` and `FadeOut` members play as a just-in-time succession.
///
/// # Errors
/// [`AnimError::EmptyComposition`] when no members are supplied, or the
/// Stage errors reported while assembling the member container.
pub fn show_creation_then_fade_out(
    stage: &mut Stage,
    animations: Vec<Box<dyn Animation>>,
    lag_ratio: f64,
    remover: bool,
) -> Result<Succession, AnimError> {
    let mut succession = Succession::with_lag_ratio(stage, animations, lag_ratio)?;
    succession.state_mut().config.name = "ShowCreationThenFadeOut".to_owned();
    succession.state_mut().config.remover = remover;
    Ok(succession)
}

// ------------------------------------------------------------ ApplyWave

/// `ApplyWave` (indication.py:333): a Homotopy nudging points along
/// `direction`, the nudge phased by x-proportion. Reference defaults:
/// `direction = UP`, `amplitude = 0.2`, `run_time = 1`.
#[must_use]
pub fn apply_wave(stage: &Stage, mobject: Mob, direction: Vec3, amplitude: f64) -> Homotopy {
    let left_x = stage.get_left(mobject)[0];
    let right_x = stage.get_right(mobject)[0];
    let vect = [
        amplitude * direction[0],
        amplitude * direction[1],
        amplitude * direction[2],
    ];
    let mut anim = Homotopy::new(
        move |x, y, z, t| {
            let alpha = (x - left_x) / (right_x - left_x);
            let power = fmn_dmath::exp(2.0 * (alpha - 0.5));
            let nudge = rate::there_and_back(fmn_dmath::pow(t, power));
            [
                x + nudge * vect[0],
                y + nudge * vect[1],
                z + nudge * vect[2],
            ]
        },
        mobject,
    );
    anim.state_mut().config.name = "ApplyWave".to_owned();
    anim.state_mut().config.run_time = 1.0;
    anim
}

// --------------------------------------------- the surrounding rectangle
// family (fm-jh7)

/// `FlashAround` (indication.py:255) wiring: a [`VShowPassingFlash`] over
/// the caller-built surrounding path with the Reference's `__init__`
/// defaults — `time_width = 1.0`, `taper_width = 0.0` (overriding the
/// generic passing-flash window), `remover`. The path VALUE-side work —
/// building `SurroundingRectangle(mobject, buff)` / `Underline`, the
/// `insert_n_curves(100)` resample, the null-curve strip, and the
/// `stroke_width = 4.0` / `color = YELLOW` styling — lives with the
/// geometry tier that owns those types (the `focus_on` doctrine:
/// Choreo wires, the library draws).
#[must_use]
pub fn flash_around(path: Mob, time_width: f64, taper_width: f64) -> VShowPassingFlash {
    let mut flash = VShowPassingFlash::new(path)
        .with_time_width(time_width)
        .with_taper_width(taper_width);
    flash.state_mut().config.name = "FlashAround".to_owned();
    flash
}

/// `AnimationOnSurroundingRectangle` (indication.py:299): one member — the
/// caller's animation of their surrounding rectangle — carried by an
/// [`AnimationGroup`]. The Reference's rect updater
/// (`r.move_to(mobject)`) is scene-updater wiring (W9/portal), the same
/// caller-supplied split as `Flash`'s lines.
///
/// # Errors
/// [`AnimError::EmptyComposition`] or the Stage errors reported while
/// assembling the member container.
pub fn animation_on_surrounding_rectangle(
    stage: &mut Stage,
    rect_animation: Box<dyn Animation>,
) -> Result<AnimationGroup, AnimError> {
    let mut group = AnimationGroup::new(stage, vec![rect_animation])?;
    group.state_mut().config.name = "AnimationOnSurroundingRectangle".to_owned();
    Ok(group)
}

/// `ShowPassingFlashAround` (indication.py:320): the surrounding rectangle
/// under a passing flash. Reference default `time_width = 1.0` flows from
/// `FlashAround`'s kwargs; the caller pins what their rectangle animation
/// needs.
///
/// # Errors
/// Group build errors, as [`animation_on_surrounding_rectangle`].
pub fn show_passing_flash_around(
    stage: &mut Stage,
    rect: Mob,
    time_width: f64,
) -> Result<AnimationGroup, AnimError> {
    let inner = Box::new(crate::creation::show_passing_flash(rect, time_width));
    let mut group = animation_on_surrounding_rectangle(stage, inner)?;
    group.state_mut().config.name = "ShowPassingFlashAround".to_owned();
    Ok(group)
}

/// `ShowCreationThenDestructionAround` (indication.py:324): the surrounding
/// rectangle under [`show_creation_then_destruction`].
///
/// # Errors
/// Group build errors, as [`animation_on_surrounding_rectangle`].
pub fn show_creation_then_destruction_around(
    stage: &mut Stage,
    rect: Mob,
) -> Result<AnimationGroup, AnimError> {
    let inner = Box::new(show_creation_then_destruction(rect));
    let mut group = animation_on_surrounding_rectangle(stage, inner)?;
    group.state_mut().config.name = "ShowCreationThenDestructionAround".to_owned();
    Ok(group)
}

/// `ShowCreationThenFadeAround` (indication.py:328): the surrounding
/// rectangle created and then faded, as one group member — the Reference's
/// `ShowCreationThenFadeOut` composition (`ShowCreation` then `FadeOut`,
/// no shift, no scale) carried by
/// [`animation_on_surrounding_rectangle`], default `remover`.
///
/// # Errors
/// [`AnimError::StaleHandle`] for a dead `rect`, or group build errors as
/// [`animation_on_surrounding_rectangle`].
pub fn show_creation_then_fade_around(
    stage: &mut Stage,
    rect: Mob,
    lag_ratio: f64,
    remover: bool,
) -> Result<AnimationGroup, AnimError> {
    let members: Vec<Box<dyn Animation>> = vec![
        Box::new(crate::creation::show_creation(rect)),
        Box::new(crate::fading::fade_out(stage, rect, Vec3::default(), 1.0)?),
    ];
    let inner = Box::new(show_creation_then_fade_out(
        stage, members, lag_ratio, remover,
    )?);
    let mut group = animation_on_surrounding_rectangle(stage, inner)?;
    group.state_mut().config.name = "ShowCreationThenFadeAround".to_owned();
    Ok(group)
}

/// `Broadcast` (specialized.py:11): a [`crate::composition::lagged_start`]
/// over caller-built `Restore` members — the styled circles that grow from
/// their saved state are Atlas/portal geometry (the `focus_on` doctrine:
/// Choreo wires, the library draws). Reference defaults:
/// `run_time = 3.0`, `lag_ratio = 0.2`, `remover = true`.
///
/// # Errors
/// [`AnimError::EmptyComposition`] for no members, or the Stage errors
/// reported while assembling the member container.
pub fn broadcast(
    stage: &mut Stage,
    restores: Vec<Box<dyn Animation>>,
    run_time: f64,
    lag_ratio: f64,
    remover: bool,
) -> Result<AnimationGroup, AnimError> {
    let mut group = crate::composition::lagged_start(stage, restores)?;
    group.state_mut().config.name = "Broadcast".to_owned();
    group.state_mut().config.run_time = run_time;
    group.state_mut().config.lag_ratio = lag_ratio;
    group.state_mut().config.remover = remover;
    Ok(group)
}

/// `FlashyFadeIn` (indication.py:408): the mobject fades in, its rate
/// squished to `[fade_lag, 1]`, while a stroke-only outline copy — fill
/// alpha 0, stroke `stroke_width` at full opacity — runs a
/// [`VShowPassingFlash`] of `time_width`. The two members run together in
/// one [`AnimationGroup`] (linear group clock, as the Reference's group
/// default). Reference defaults: `stroke_width = 2.0`, `fade_lag = 0.0`,
/// `time_width = 1.0`. The outline copy is made now, as the Reference copies
/// at construction; the passing flash removes it when done.
///
/// # Errors
/// [`AnimError::StaleHandle`] for a dead `vmobject`, or the Stage errors
/// reported while copying it or assembling the group.
pub fn flashy_fade_in(
    stage: &mut Stage,
    vmobject: Mob,
    stroke_width: f64,
    fade_lag: f64,
    time_width: f64,
) -> Result<AnimationGroup, AnimError> {
    if !stage.contains(vmobject) {
        return Err(AnimError::StaleHandle(vmobject));
    }
    let outline = stage.copy_family(vmobject)?;
    stage.set_color_lanes(outline, "fill_rgba", None, Some(0.0), true);
    stage.set_scalar_lanes(outline, "stroke_width", stroke_width, true);
    stage.set_color_lanes(outline, "stroke_rgba", None, Some(1.0), true);
    let mut fade = crate::fading::fade_in(stage, vmobject, Vec3::default(), 1.0)?;
    fade.state_mut().config.rate_func = RateFunc::smooth().squish(fade_lag, 1.0);
    let flash = VShowPassingFlash::new(outline).with_time_width(time_width);
    let mut group = AnimationGroup::new(stage, vec![Box::new(fade), Box::new(flash)])?;
    group.state_mut().config.name = "FlashyFadeIn".to_owned();
    Ok(group)
}
