//! The fade mechanism — part of family 3 (§9.4, fm-cye), ported from the
//! pinned Reference's `animation/fading.py`:
//!
//! - The `Fade` lineage is Transform parameterized by copies: [`fade_in`]
//!   keeps the live target and mutates the *starting* copy (opacity 0,
//!   `scale(1/f)`, `shift(−v)` — [`StartPrep`] carries the sequence);
//!   [`fade_out`] mutates the *target* copy (opacity 0, `shift(v)`,
//!   `scale(f)`) and finishes at `final_alpha_value = 0` so the mobject
//!   leaves the scene in its original state (fading.py:50's comment,
//!   ported comment-for-comment). [`fade_in_from_point`] /
//!   [`fade_out_to_point`] derive shift/scale exactly (`np.inf` scale
//!   included: `1/∞ = 0` collapses the start onto the point).
//! - [`VFade`] (fading.py:152) writes stroke/fill opacity as
//!   `interpolate(0, start_opacity, α)` — [`v_fade_out`] runs the same
//!   rule at `1 − α`, [`v_fade_in_then_out`] under `there_and_back` with
//!   `final_alpha_value = 0.5`.
//! - [`FadeTransform`] (fading.py:91): a fresh group over `(mobject,
//!   target.copy())`, an ending copy taken before the starting copy, and
//!   the two ghosting passes (`replace` to the counterpart's box, uniform
//!   copy, opacity 0) after the zero interpolation — begin order preserved
//!   hook for hook. [`fade_transform_pieces`] aligns the pair's families
//!   first and ghosts member-for-member. Scene-side cleanup (remove the
//!   group, restore the source from its §8.3 saved state, add
//!   [`FadeTransform::to_add_on_completion`]) is the scene runtime's
//!   (fm-5xm), consuming the flags carried here.

use fmn_core::rate;
use fmn_core::types::Vec3;
use fmn_mobject::{Mob, Mobject, Stage, StageError};

use crate::animation::{AnimConfig, AnimError, AnimState, Animation, AnimationSignature, RateFunc};
use crate::transform::{PathFunc, StartPrep, TargetPrep, Transform, interpolate_fields};

fn neg(v: Vec3) -> Vec3 {
    [-v[0], -v[1], -v[2]]
}

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

// ------------------------------------------------------------- Fade zoo

/// `FadeIn` (fading.py:34): transform from an invisible, pre-scaled,
/// pre-shifted starting copy onto a copy of the mobject as it stands.
/// `shift`/`scale` are the Reference's `Fade` parameters (`ORIGIN`/`1`
/// defaults). Geometry and paint are copied at each `begin`, while these
/// parameters remain construction-time values.
///
/// # Errors
/// [`AnimError::StaleHandle`] / [`AnimError::Stage`].
pub fn fade_in(
    stage: &mut Stage,
    mobject: Mob,
    shift: Vec3,
    scale: f64,
) -> Result<Transform, AnimError> {
    if !stage.contains(mobject) {
        return Err(AnimError::StaleHandle(mobject));
    }
    // Transform freezes self-targets at begin, not while a whole Succession
    // is constructed. This also samples fresh geometry and paint on replay.
    let mut t = Transform::new(mobject, mobject).with_start_prep(StartPrep {
        opacity_zero: true,
        scale: Some(1.0 / scale),
        shift: Some(neg(shift)),
        ..StartPrep::default()
    });
    t.state_mut().config.name = "FadeIn".to_owned();
    Ok(t)
}

/// `FadeOut` (fading.py:46): transform onto an invisible, shifted, scaled
/// copy; a remover whose `final_alpha_value = 0` puts the mobject back in
/// its begin-time state when done. The invisible target is prepared at each
/// `begin`, so delayed composition members and replay use the live source.
///
/// # Errors
/// [`AnimError::StaleHandle`] / [`AnimError::Stage`].
pub fn fade_out(
    stage: &mut Stage,
    mobject: Mob,
    shift: Vec3,
    scale: f64,
) -> Result<Transform, AnimError> {
    if !stage.contains(mobject) {
        return Err(AnimError::StaleHandle(mobject));
    }
    // Keep only the recipe until begin; the private target must reflect any
    // preceding animation, without making that predecessor's state vanish.
    let mut t =
        Transform::new(mobject, mobject).with_target_prep(TargetPrep::FadeOut { shift, scale });
    t.state_mut().config.name = "FadeOut".to_owned();
    t.state_mut().config.remover = true;
    t.state_mut().config.final_alpha_value = 0.0;
    Ok(t)
}

/// `FadeInFromPoint` (fading.py:71): `shift = center − point`,
/// `scale = ∞`. The shift is captured now, as in the Reference; if the
/// source moves before begin, the collapse point moves by the same amount.
///
/// # Errors
/// As [`fade_in`].
pub fn fade_in_from_point(
    stage: &mut Stage,
    mobject: Mob,
    point: Vec3,
) -> Result<Transform, AnimError> {
    let center = stage.get_center(mobject);
    let mut t = fade_in(stage, mobject, sub(center, point), f64::INFINITY)?;
    t.state_mut().config.name = "FadeInFromPoint".to_owned();
    Ok(t)
}

/// `FadeOutToPoint` (fading.py:81): `shift = point − center`, `scale = 0`.
/// The constructor captures the shift, not a fixed world-space destination.
///
/// # Errors
/// As [`fade_out`].
pub fn fade_out_to_point(
    stage: &mut Stage,
    mobject: Mob,
    point: Vec3,
) -> Result<Transform, AnimError> {
    let center = stage.get_center(mobject);
    let mut t = fade_out(stage, mobject, sub(point, center), 0.0)?;
    t.state_mut().config.name = "FadeOutToPoint".to_owned();
    Ok(t)
}

// -------------------------------------------------------------- VFade

/// `VFadeIn`/`VFadeOut`/`VFadeInThenOut` (fading.py:152): stroke and fill
/// opacity ramps that leave every other field alone — the fade that
/// composes with updaters, which is why `suspend_mobject_updating` stays
/// `false`.
#[derive(Debug, Clone)]
pub struct VFade {
    state: AnimState,
    reversed: bool,
}

impl VFade {
    /// Replace the animation config.
    #[must_use]
    pub fn with_config(mut self, config: AnimConfig) -> Self {
        self.state.config = config;
        self
    }
}

impl Animation for VFade {
    fn state(&self) -> &AnimState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut AnimState {
        &mut self.state
    }

    /// Pure: opacity writes are a function of the start pair and alpha.
    fn effect_signature(&self) -> AnimationSignature {
        AnimationSignature::Pure
    }

    /// "VFadeIn and VFadeOut only work for VMobjects" (fading.py:154) —
    /// enforced up front, by name.
    fn setup(&mut self, stage: &mut Stage) -> Result<(), AnimError> {
        let mobject = self.state.mobject();
        if !stage.contains(mobject) {
            return Err(AnimError::StaleHandle(mobject));
        }
        for member in stage.family(mobject) {
            if let Some(entry) = stage.get(member)
                && !entry.buffer.is_empty()
                && entry.buffer.schema().offset("joint_angle").is_none()
            {
                return Err(AnimError::Stage(StageError::SchemaMismatch));
            }
        }
        Ok(())
    }

    /// fading.py:166: `set_stroke(opacity=interpolate(0, start_stroke, α))`
    /// and the fill likewise; the out direction runs the same rule at
    /// `1 − α` (fading.py:196).
    fn interpolate_submobject(&mut self, stage: &mut Stage, mobs: &[Mob], sub_alpha: f64) {
        let [submob, start] = *mobs else {
            return; // rows are pairs once begin has run
        };
        let alpha = if self.reversed {
            1.0 - sub_alpha
        } else {
            sub_alpha
        };
        for field in ["stroke_rgba", "fill_rgba"] {
            let Some(start_col) = stage.get(start).and_then(|e| e.buffer.read_column(field)) else {
                continue;
            };
            // get_*_opacity() reads the first record's alpha lane.
            let Some(&start_opacity) = start_col.get(3) else {
                continue;
            };
            #[allow(clippy::cast_possible_truncation)]
            let value = (alpha * f64::from(start_opacity)) as f32;
            if let Some(entry) = stage.get_mut(submob)
                && let Some(mut column) = entry.buffer.read_column(field)
            {
                for lane in column.iter_mut().skip(3).step_by(4) {
                    *lane = value;
                }
                entry.buffer.write_range(field, 0, &column);
            }
        }
    }
}

/// `VFadeIn` (fading.py:152).
#[must_use]
pub fn v_fade_in(vmobject: Mob) -> VFade {
    let config = AnimConfig {
        name: "VFadeIn".to_owned(),
        ..AnimConfig::default()
    };
    VFade {
        state: AnimState::new(vmobject, config),
        reversed: false,
    }
}

/// `VFadeOut` (fading.py:180): reversed ramp, remover,
/// `final_alpha_value = 0`.
#[must_use]
pub fn v_fade_out(vmobject: Mob) -> VFade {
    let mut anim = v_fade_in(vmobject);
    anim.state.config.name = "VFadeOut".to_owned();
    anim.state.config.remover = true;
    anim.state.config.final_alpha_value = 0.0;
    anim.reversed = true;
    anim
}

/// `VFadeInThenOut` (fading.py:203): `there_and_back`, remover,
/// `final_alpha_value = 0.5`.
#[must_use]
pub fn v_fade_in_then_out(vmobject: Mob) -> VFade {
    let mut anim = v_fade_in(vmobject);
    anim.state.config.name = "VFadeInThenOut".to_owned();
    anim.state.config.rate_func = RateFunc::Base(rate::there_and_back);
    anim.state.config.remover = true;
    anim.state.config.final_alpha_value = 0.5;
    anim
}

// -------------------------------------------------------- FadeTransform

/// Reference `Mobject.replace` (mobject.py): rescale to the target's box
/// (every dimension when `stretch`, else uniformly by `dim_to_match`),
/// then recenter.
fn replace_bbox(stage: &mut Stage, mob: Mob, target: Mob, stretch: bool, dim_to_match: usize) {
    if stretch {
        for dim in 0..3 {
            let length = stage.length_over_dim(target, dim);
            stage.rescale_to_fit(mob, length, dim, true);
        }
    } else {
        let length = stage.length_over_dim(target, dim_to_match);
        stage.rescale_to_fit(mob, length, dim_to_match, false);
    }
    let center = stage.get_center(target);
    stage.move_to(mob, center, [0.0, 0.0, 0.0]);
}

/// `FadeTransform` (fading.py:91): cross-fade `mobject → target` through
/// a group of ghosts. The animated mobject is a fresh group over
/// `(mobject, target.copy())`; the start state ghosts the target half
/// onto the source's box, the end state ghosts the source half onto the
/// target's box, and interpolation is the plain field lerp between them.
#[derive(Debug, Clone)]
pub struct FadeTransform {
    state: AnimState,
    ending: Option<Mob>,
    to_add_on_completion: Mob,
    stretch: bool,
    dim_to_match: usize,
    pieces: bool,
}

impl FadeTransform {
    /// `stretch` (Reference default `true`).
    #[must_use]
    pub fn with_stretch(mut self, stretch: bool) -> Self {
        self.stretch = stretch;
        self
    }

    /// `dim_to_match` (Reference default `1`).
    #[must_use]
    pub fn with_dim_to_match(mut self, dim: usize) -> Self {
        self.dim_to_match = dim;
        self
    }

    /// Replace the animation config.
    #[must_use]
    pub fn with_config(mut self, config: AnimConfig) -> Self {
        self.state.config = config;
        self
    }

    /// The original target, which the scene runtime adds on completion
    /// (`clean_up_from_scene`, fading.py:137 — fm-5xm consumes this, along
    /// with restoring the source from its saved state and removing the
    /// group).
    #[must_use]
    pub fn to_add_on_completion(&self) -> Mob {
        self.to_add_on_completion
    }

    /// The ending copy (test hook; `None` before `begin`).
    #[must_use]
    pub fn ending(&self) -> Option<Mob> {
        self.ending
    }

    /// fading.py:115 `ghost_to`: `replace` onto the counterpart's box,
    /// copy its uniforms, vanish. The pieces variant runs the rule
    /// member-for-member over the zipped families (fading.py:146).
    fn ghost_to(&self, stage: &mut Stage, source: Mob, target: Mob) {
        let pairs: Vec<(Mob, Mob)> = if self.pieces {
            stage
                .family(source)
                .into_iter()
                .zip(stage.family(target))
                .collect()
        } else {
            vec![(source, target)]
        };
        for (src, tgt) in pairs {
            replace_bbox(stage, src, tgt, self.stretch, self.dim_to_match);
            let uniforms = stage.get(tgt).map(|e| *e.uniforms());
            if let (Some(u), Some(entry)) = (uniforms, stage.get_mut(src)) {
                *entry.uniforms_mut() = u;
            }
            stage.set_family_opacity_zero(src);
        }
    }
}

impl Animation for FadeTransform {
    fn state(&self) -> &AnimState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut AnimState {
        &mut self.state
    }

    /// Pure: interpolation lerps between the frozen ghosted copies.
    fn effect_signature(&self) -> AnimationSignature {
        AnimationSignature::Pure
    }

    /// The pieces variant aligns the pair's families first
    /// (fading.py:143); then the ending copy is taken *before* the
    /// canonical sequence copies the start (fading.py:104).
    fn setup(&mut self, stage: &mut Stage) -> Result<(), AnimError> {
        let group = self.state.mobject();
        if !stage.contains(group) {
            return Err(AnimError::StaleHandle(group));
        }
        let children = stage
            .get(group)
            .ok_or(AnimError::StaleHandle(group))?
            .submobjects()
            .to_vec();
        let [source, target_copy] = children[..] else {
            return Err(AnimError::StaleHandle(group));
        };
        if self.pieces {
            stage.align_family(source, target_copy)?;
        }
        self.ending = Some(stage.copy_family(group)?);
        Ok(())
    }

    /// fading.py:106: after the zero interpolation, ghost the start's
    /// target half onto the source and the end's source half onto the
    /// target.
    fn after_begin(&mut self, stage: &mut Stage) {
        let (Some(starting), Some(ending)) = (self.state.starting_mobject(), self.ending) else {
            return;
        };
        let start_children = stage
            .get(starting)
            .map(|e| e.submobjects().to_vec())
            .unwrap_or_default();
        let end_children = stage
            .get(ending)
            .map(|e| e.submobjects().to_vec())
            .unwrap_or_default();
        if let ([s0, s1], [e0, e1]) = (&start_children[..], &end_children[..]) {
            self.ghost_to(stage, *s1, *s0);
            self.ghost_to(stage, *e0, *e1);
        }
    }

    /// `(mobject, starting_mobject, ending_mobject)` (fading.py:120).
    fn all_mobjects(&self) -> Vec<Mob> {
        let mut mobs = vec![self.state.mobject()];
        if let Some(starting) = self.state.starting_mobject() {
            mobs.push(starting);
        }
        if let Some(ending) = self.ending {
            mobs.push(ending);
        }
        mobs
    }

    fn preflight_mobjects(&self) -> Vec<Mob> {
        vec![self.state.mobject(), self.to_add_on_completion]
    }

    fn interpolate_submobject(&mut self, stage: &mut Stage, mobs: &[Mob], sub_alpha: f64) {
        let [submob, start, end] = *mobs else {
            return; // rows are triples once begin has run
        };
        interpolate_fields(stage, submob, start, end, sub_alpha, PathFunc::Straight);
    }

    /// fading.py:132 — remove the temporary pair, restore the original
    /// source records, and publish the real target unless this transform is a
    /// remover. The generic Choreo finish path invokes this through the
    /// animation trait object.
    fn clean_up_from_scene(&mut self, stage: &mut Stage) {
        let group = self.state.mobject();
        let source = stage
            .get(group)
            .and_then(|entry| entry.submobjects().first())
            .copied();
        stage.remove_from_scene(group);
        if let Some(source) = source {
            let _ = stage.restore_mobject(source);
        }
        if !self.is_remover() {
            let _ = stage.add_to_scene(self.to_add_on_completion);
        }
    }
}

/// `FadeTransform` with Reference defaults (`stretch = true`,
/// `dim_to_match = 1`). Saves the source's state (the scene runtime
/// restores it on cleanup) and animates a fresh group over the source and
/// a copy of the target.
///
/// # Errors
/// [`AnimError::StaleHandle`] / [`AnimError::Stage`].
pub fn fade_transform(
    stage: &mut Stage,
    mobject: Mob,
    target_mobject: Mob,
) -> Result<FadeTransform, AnimError> {
    stage.save_state(mobject)?;
    let target_copy = stage.copy_family(target_mobject)?;
    let group = stage.add(Mobject::new());
    stage.attach(group, mobject)?;
    stage.attach(group, target_copy)?;
    let config = AnimConfig {
        name: "FadeTransform".to_owned(),
        ..AnimConfig::default()
    };
    Ok(FadeTransform {
        state: AnimState::new(group, config),
        ending: None,
        to_add_on_completion: target_mobject,
        stretch: true,
        dim_to_match: 1,
        pieces: false,
    })
}

/// `FadeTransformPieces` (fading.py:142): align the pair's families, then
/// ghost member-for-member.
///
/// # Errors
/// As [`fade_transform`].
pub fn fade_transform_pieces(
    stage: &mut Stage,
    mobject: Mob,
    target_mobject: Mob,
) -> Result<FadeTransform, AnimError> {
    let mut anim = fade_transform(stage, mobject, target_mobject)?;
    anim.state.config.name = "FadeTransformPieces".to_owned();
    anim.pieces = true;
    Ok(anim)
}
