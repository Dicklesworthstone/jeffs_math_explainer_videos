//! The Transform mechanism — family 1 of the five (§9.4, fm-cye):
//! alignment + field lerp through a path function, ported from the pinned
//! Reference (transform.py, utils/paths.py, mobject.py:1810).
//!
//! - [`PathFunc`] carries the shared path law: straight lerp or rotation
//!   about the computed arc center with linear axial motion (a helix in
//!   3D; BN-12). A scalar `|arc_angle| <` [`STRAIGHT_PATH_THRESHOLD`]
//!   collapses to straight, as in the Reference's early return.
//! - The lerp core routes **only pointlike record fields through the path
//!   function**; every other field lerps linearly, locked fields are
//!   skipped entirely, numeric uniforms lerp linearly, and Marionette's
//!   typed ValueTracker lanes lerp in their native encoding
//!   (mobject.py:1810). The Reference's `const_data_keys` broadcast is a
//!   pure optimization (a constant column lerps to the same values
//!   row-wise) and is deliberately not replicated.
//! - [`Transform::setup`] aligns (`align_data_and_family`) before the
//!   starting copy is taken; `after_begin` locks matching data (skipped
//!   when the family has updaters), `teardown` unlocks — the Reference's
//!   begin/finish order, hook for hook.
//!
//! Zoo members here are parameterizations, never re-implementations:
//! `ReplacementTransform` flips the replace-in-scene flag (consumed by the
//! scene runtime, fm-5xm), `TransformFromCopy` animates a fresh copy,
//! `MoveToTarget`/`Restore` consume the §8.3 target/saved-state links, and
//! the `ApplyMethod` family builds targets by applying stage operations to
//! a copy. The remaining catalog (partial-reveal, fade/grow, indication,
//! functional maps) lands in later fm-cye slices.

use fmn_core::types::Vec3;
use fmn_mobject::{Mob, Placement, Stage, Tracker};

use crate::animation::{
    AnimConfig, AnimError, AnimState, Animation, AnimationSignature, interpolate_linear_column,
};

#[path = "motion_path.rs"]
mod motion_path;
use motion_path::interpolate_placement;
pub use motion_path::{PathFunc, STRAIGHT_PATH_THRESHOLD};

/// Placement interpolation is sufficient only when ALL object-space geometric
/// columns agree, not merely the primary vertices. Surface normal seeds and
/// custom auxiliary-only schemas obey the same affine lift.
fn same_pointlike_geometry(stage: &Stage, from: Mob, to: Mob) -> bool {
    let (Some(a), Some(b)) = (stage.get(from), stage.get(to)) else {
        return false;
    };
    a.buffer
        .schema()
        .fields()
        .iter()
        .filter(|field| {
            field.width == 3
                && (field.name == "point"
                    || a.buffer.schema().pointlike_keys().contains(&field.name))
        })
        .all(|field| a.buffer.column_eq(&b.buffer, &field.name))
}

// ------------------------------------------------------------- lerp core

/// The Reference's `Mobject.interpolate` (mobject.py:1810) over one
/// zipped submobject triple: pointlike fields route through `path`, every
/// other field lerps linearly, locked fields are skipped, numeric
/// uniforms lerp linearly. Computed in f64, stored at record precision.
pub fn interpolate_fields(
    stage: &mut Stage,
    submob: Mob,
    from: Mob,
    to: Mob,
    alpha: f64,
    path: PathFunc,
) {
    let Some(entry) = stage.get(submob) else {
        return;
    };
    let Some((from_placement, to_placement)) = stage.placement(from).zip(stage.placement(to))
    else {
        return;
    };
    let schema = entry.buffer.schema();
    let pointlike: Vec<String> = schema
        .fields()
        .iter()
        .filter(|field| {
            field.width == 3
                && (field.name == "point" || schema.pointlike_keys().contains(&field.name))
        })
        .map(|field| field.name.clone())
        .collect();
    // A host read can bake all geometric fields into this buffer, including
    // while no view remains live. Return to the frozen endpoints rather than
    // compounding that intermediate representation on the next sample.
    let representation_drifted = stage.get(from).is_some_and(|starting| {
        pointlike
            .iter()
            .any(|key| !entry.buffer.column_eq(&starting.buffer, key))
    });
    let sync_world = !entry.buffer.is_empty()
        && !pointlike.is_empty()
        && (entry.buffer.live_view_count() > 0 || representation_drifted);
    if sync_world {
        // Never drop placement after materializing only part of a surface. A
        // foreign resize/schema edit mid-play must not create mixed coordinates.
        let valid = pointlike.iter().all(|key| {
            let expected = entry.buffer.len().checked_mul(3);
            [from, to].into_iter().all(|mob| {
                stage
                    .get(mob)
                    .and_then(|endpoint| endpoint.buffer.read_column(key))
                    .is_some_and(|column| Some(column.len()) == expected)
            })
        });
        if !valid {
            return;
        }
    }
    let fields: Vec<String> = schema
        .fields()
        .iter()
        // Equal object-space normals are normally locked. They still need a
        // world-space write whenever vertices are materialized for a live view.
        .filter(|field| {
            (sync_world && pointlike.contains(&field.name)) || !entry.buffer.is_locked(&field.name)
        })
        .map(|field| field.name.clone())
        .collect();
    for field in fields {
        let (Some(a), Some(b)) = (
            stage.get(from).and_then(|e| e.buffer.read_column(&field)),
            stage.get(to).and_then(|e| e.buffer.read_column(&field)),
        ) else {
            continue;
        };
        if a.len() != b.len() {
            continue;
        }
        #[allow(clippy::cast_possible_truncation)]
        let out: Vec<f32> = if pointlike.contains(&field) {
            let (pa, ra) = a.as_chunks::<3>();
            let (pb, rb) = b.as_chunks::<3>();
            if !ra.is_empty() || !rb.is_empty() {
                continue;
            }
            pa.iter()
                .zip(pb)
                .flat_map(|(ca, cb)| {
                    let a = ca.map(f64::from);
                    let b = cb.map(f64::from);
                    let point = if sync_world {
                        path.eval(
                            from_placement.apply_point(a),
                            to_placement.apply_point(b),
                            alpha,
                        )
                    } else {
                        path.eval(a, b, alpha)
                    };
                    point.map(|value| value as f32)
                })
                .collect()
        } else {
            interpolate_linear_column(&a, &b, alpha)
        };
        if let Some(entry) = stage.get_mut(submob)
            && entry.buffer.read_column(&field).as_deref() != Some(out.as_slice())
        {
            entry.buffer.write_range(&field, 0, &out);
        }
    }
    let placement = if sync_world {
        Placement::IDENTITY
    } else {
        interpolate_placement(from_placement, to_placement, alpha, path)
    };
    let _ = stage.set_placement(submob, placement);
    // ValueTracker state is the Reference's numeric `uniforms["value"]`
    // represented as a typed Marionette payload.  Interpolate the encoded
    // lanes: plain values remain arithmetic, exponential trackers interpolate
    // their logarithms (therefore geometric values), and complex trackers
    // interpolate real/imaginary components independently.
    if let (Some(from_tracker), Some(to_tracker)) = (stage.tracker(from), stage.tracker(to))
        && from_tracker.kind == to_tracker.kind
    {
        let lerp = |x: f64, y: f64| (1.0 - alpha) * x + alpha * y;
        let tracker = Tracker {
            kind: from_tracker.kind,
            lanes: [
                lerp(from_tracker.lanes[0], to_tracker.lanes[0]),
                lerp(from_tracker.lanes[1], to_tracker.lanes[1]),
            ],
        };
        let _ = stage.set_tracker_state(submob, Some(tracker));
    }
    // Numeric uniforms lerp linearly (the Reference lerps every shared
    // uniform key); discrete state (flags, joint type) stays the live
    // mobject's own.
    let (Some(ua), Some(ub)) = (
        stage.get(from).map(|e| *e.uniforms()),
        stage.get(to).map(|e| *e.uniforms()),
    ) else {
        return;
    };
    if let Some(entry) = stage.get_mut(submob) {
        let u = entry.uniforms_mut();
        let lerp = |x: f64, y: f64| (1.0 - alpha) * x + alpha * y;
        u.is_fixed_in_frame = lerp(ua.is_fixed_in_frame, ub.is_fixed_in_frame);
        u.anti_alias_width = lerp(ua.anti_alias_width, ub.anti_alias_width);
        for k in 0..3 {
            u.shading[k] = lerp(ua.shading[k], ub.shading[k]);
        }
        for (plane, (pa, pb)) in u
            .clip_planes
            .iter_mut()
            .zip(ua.clip_planes.iter().zip(ub.clip_planes.iter()))
        {
            for (slot, (&x, &y)) in plane.iter_mut().zip(pa.iter().zip(pb.iter())) {
                *slot = lerp(x, y);
            }
        }
    }
}

/// Reference `lock_matching_data` (mobject.py:1871): over the zipped
/// families, lock every record field whose columns are identical in the
/// start and target — those never need interpolating. No-op when the
/// animated family has updaters, exactly as the Reference.
fn lock_matching_data(stage: &mut Stage, mobject: Mob, m1: Mob, m2: Mob) {
    if stage.has_updaters_in_family(mobject) {
        return;
    }
    let family = stage.family(mobject);
    let f1 = stage.family(m1);
    let f2 = stage.family(m2);
    for ((&sub, &a), &b) in family.iter().zip(f1.iter()).zip(f2.iter()) {
        let (Some(ea), Some(eb)) = (stage.get(a), stage.get(b)) else {
            continue;
        };
        if ea.buffer.schema() != eb.buffer.schema() {
            continue; // Reference: skip on dtype mismatch
        }
        let matching: Vec<String> = ea
            .buffer
            .schema()
            .fields()
            .iter()
            .map(|f| f.name.clone())
            .filter(|name| ea.buffer.read_column(name) == eb.buffer.read_column(name))
            .collect();
        if let Some(entry) = stage.get_mut(sub) {
            entry.buffer.lock_data(matching.iter().map(String::as_str));
        }
    }
}

fn unlock_family_data(stage: &mut Stage, mobject: Mob) {
    for member in stage.family(mobject) {
        if let Some(entry) = stage.get_mut(member) {
            entry.buffer.unlock_data();
        }
    }
}

// -------------------------------------------------------------- Transform

/// Mutations applied to the starting copy right after it is taken — the
/// Reference's `create_starting_mobject` overrides (fading.py:37,
/// growing.py:30) as journal-able data instead of subclasses. Application
/// order is fixed: opacity → scale → shift → move_to → color, which
/// realizes both the FadeIn sequence (`set_opacity(0)`, `scale(1/f)`,
/// `shift(−v)`) and the GrowFromPoint sequence (`scale(0)`,
/// `move_to(point)`, `set_color`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StartPrep {
    /// `set_opacity(0)` over the whole family.
    pub opacity_zero: bool,
    /// `scale(k)` about the copy's center.
    pub scale: Option<f64>,
    /// `shift(v)`.
    pub shift: Option<Vec3>,
    /// `move_to(point)`.
    pub move_to: Option<Vec3>,
    /// `set_color(rgb)`: overwrite the rgb lanes of every `*rgba` field,
    /// alphas kept (growing.py's `point_color`).
    pub color: Option<[f32; 3]>,
}

impl StartPrep {
    fn apply(&self, stage: &mut Stage, mob: Mob) {
        if self.opacity_zero {
            stage.set_family_opacity_zero(mob);
        }
        if let Some(k) = self.scale {
            stage.scale(mob, k);
        }
        if let Some(v) = self.shift {
            stage.shift(mob, v);
        }
        if let Some(p) = self.move_to {
            stage.move_to(mob, p, [0.0, 0.0, 0.0]);
        }
        if let Some(rgb) = self.color {
            set_family_rgb(stage, mob, rgb);
        }
    }
}

/// Reference `set_color` over a family: overwrite the rgb lanes of every
/// `*rgba` field, alphas kept. Shared by [`StartPrep`] and the
/// indication family's recolored targets.
pub(crate) fn set_family_rgb(stage: &mut Stage, mob: Mob, rgb: [f32; 3]) {
    for member in stage.family(mob) {
        let Some(entry) = stage.get_mut(member) else {
            continue;
        };
        let fields: Vec<String> = entry
            .buffer
            .schema()
            .fields()
            .iter()
            .filter(|f| f.name.ends_with("rgba"))
            .map(|f| f.name.clone())
            .collect();
        for field in fields {
            if let Some(mut column) = entry.buffer.read_column(&field) {
                for row in column.as_chunks_mut::<4>().0 {
                    row[..3].copy_from_slice(&rgb);
                }
                entry.buffer.write_range(&field, 0, &column);
            }
        }
    }
}

/// Deferred mutations of a fresh live-source copy. These are target recipes,
/// not already-copied mobjects: Succession and replay resolve them at begin.
#[derive(Debug, Clone)]
pub(crate) enum TargetPrep {
    Indicate { scale: f64, color: [f32; 3] },
    ReversePoints,
    FadeOut { shift: Vec3, scale: f64 },
    Color([f32; 3]),
    Scale(f64),
    Matrix([[f64; 3]; 3]),
}

impl TargetPrep {
    fn apply(&self, stage: &mut Stage, target: Mob) -> Result<(), AnimError> {
        match *self {
            Self::Indicate { scale, color } => {
                stage.scale(target, scale);
                set_family_rgb(stage, target, color);
            }
            Self::ReversePoints => stage.reverse_family_points(target)?,
            Self::FadeOut { shift, scale } => {
                // Reference create_target: vanish, then shift, then scale
                // about the shifted center. Do not reorder the stage writes.
                stage.set_family_opacity_zero(target);
                stage.shift(target, shift);
                stage.scale(target, scale);
            }
            Self::Color(rgb) => set_family_rgb(stage, target, rgb),
            Self::Scale(factor) => {
                stage.scale(target, factor);
            }
            Self::Matrix(matrix) => {
                // Use the existing world-space point-map operation, not a
                // second matrix/placement interpolation path. The origin and
                // arithmetic association match ApplyPointwiseFunction.
                stage.apply_points_function(
                    target,
                    |p| {
                        [
                            matrix[0][0] * p[0] + matrix[0][1] * p[1] + matrix[0][2] * p[2],
                            matrix[1][0] * p[0] + matrix[1][1] * p[1] + matrix[1][2] * p[2],
                            matrix[2][0] * p[0] + matrix[2][1] * p[1] + matrix[2][2] * p[2],
                        ]
                    },
                    Some([0.0, 0.0, 0.0]),
                    None,
                );
            }
        }
        Ok(())
    }
}

/// The Transform animation (transform.py:24): align, then lerp fields
/// from the starting copy to an aligned target copy through the path
/// function.
#[derive(Debug, Clone)]
pub struct Transform {
    state: AnimState,
    target: Mob,
    target_copy: Option<Mob>,
    path: PathFunc,
    replace_in_scene: bool,
    start_prep: Option<StartPrep>,
    target_prep: Option<TargetPrep>,
}

impl Transform {
    /// `Transform(mobject, target_mobject)` with the straight path.
    #[must_use]
    pub fn new(mobject: Mob, target: Mob) -> Self {
        let config = AnimConfig {
            name: "Transform".to_owned(),
            ..AnimConfig::default()
        };
        Self {
            state: AnimState::new(mobject, config),
            target,
            target_copy: None,
            path: PathFunc::Straight,
            replace_in_scene: false,
            start_prep: None,
            target_prep: None,
        }
    }

    /// Attach a [`StartPrep`] (the fade/grow families' starting-copy
    /// mutation).
    #[must_use]
    pub fn with_start_prep(mut self, prep: StartPrep) -> Self {
        self.start_prep = Some(prep);
        self
    }

    /// Prepare a private target at begin, before family alignment. Used by
    /// the fade, indication and fixed-method families over one interpolator.
    #[must_use]
    pub(crate) fn with_target_prep(mut self, prep: TargetPrep) -> Self {
        self.target_prep = Some(prep);
        self
    }

    /// `path_arc` / `path_arc_axis` (the Reference's `init_path_func`).
    #[must_use]
    pub fn with_path_arc(mut self, angle: f64, axis: Vec3) -> Self {
        self.path = PathFunc::from_path_arc(angle, axis);
        self
    }

    /// Explicit `path_func` override.
    #[must_use]
    pub fn with_path_func(mut self, path: PathFunc) -> Self {
        self.path = path;
        self
    }

    /// Replace the animation config (run_time, rate_func, lag_ratio …).
    #[must_use]
    pub fn with_config(mut self, config: AnimConfig) -> Self {
        self.state.config = config;
        self
    }

    fn with_replace_flag(mut self) -> Self {
        self.replace_in_scene = true;
        self
    }

    /// The Reference's `replace_mobject_with_target_in_scene` flag — the
    /// scene runtime (fm-5xm) consumes it in `clean_up_from_scene`.
    #[must_use]
    pub fn replaces_mobject_in_scene(&self) -> bool {
        self.replace_in_scene
    }

    /// The aligned copy actually interpolated toward (test hook).
    #[must_use]
    pub fn target_copy(&self) -> Option<Mob> {
        self.target_copy
    }
}

impl Animation for Transform {
    fn state(&self) -> &AnimState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut AnimState {
        &mut self.state
    }

    /// Pure: interpolation writes records as a function of the frozen
    /// start/target pair and alpha.
    fn effect_signature(&self) -> AnimationSignature {
        AnimationSignature::Pure
    }

    /// transform.py:54 — target copy (shared when already aligned), then
    /// `align_data_and_family`, all before the starting copy is taken.
    fn setup(&mut self, stage: &mut Stage) -> Result<(), AnimError> {
        self.path.validate()?;
        let mobject = self.state.mobject();
        if !stage.contains(mobject) {
            return Err(AnimError::StaleHandle(mobject));
        }
        if !stage.contains(self.target) {
            return Err(AnimError::StaleHandle(self.target));
        }
        // A self-target is a live begin-time recipe (used by the grow family),
        // not an endpoint that may alias interpolation's destination. Freeze
        // it before taking/preparing the starting copy, on every begin/replay.
        let mut target_copy = if self.target_prep.is_none()
            && self.target != mobject
            && stage.is_aligned_with(mobject, self.target)
        {
            self.target
        } else {
            stage.copy_family(self.target)?
        };
        if let Some(prep) = &self.target_prep {
            prep.apply(stage, target_copy)?;
        }
        stage.align_data_and_family(mobject, target_copy)?;
        // A pair whose object-space geometric columns differ cannot be represented by
        // placement interpolation alone. Bake each side once into world-space
        // geometry; pairs with equal object geometry retain their independent
        // placements, which is the fast and revision-correct path for
        // translate/rotate/scale transforms (fm-7if).
        let source_family = stage.family(mobject);
        let target_family = stage.family(target_copy);
        let needs_geometry_bake = source_family
            .iter()
            .zip(&target_family)
            .any(|(&source, &target)| !same_pointlike_geometry(stage, source, target));
        // An already aligned target is normally safe to share. Baking is the
        // exception because it changes stored coordinates; preserve the user's
        // target by switching to a private family copy first.
        if needs_geometry_bake && target_copy == self.target {
            target_copy = stage.copy_family(self.target)?;
        }
        let source_family = stage.family(mobject);
        let target_family = stage.family(target_copy);
        for (&source, &target) in source_family.iter().zip(target_family.iter()) {
            if !same_pointlike_geometry(stage, source, target) {
                stage.bake_placement(source)?;
                stage.bake_placement(target)?;
            }
        }
        self.target_copy = Some(target_copy);
        Ok(())
    }

    /// The default deep copy, then the [`StartPrep`] mutation (the
    /// fade/grow families' `create_starting_mobject` overrides).
    fn create_starting_mobject(&self, stage: &mut Stage) -> Result<Mob, AnimError> {
        let mobject = self.state().mobject();
        let starting = stage
            .copy_family(mobject)
            .map_err(|_| AnimError::StaleHandle(mobject))?;
        if let Some(prep) = &self.start_prep {
            prep.apply(stage, starting);
        }
        Ok(starting)
    }

    /// transform.py:69 — lock matching data after `super().begin()`.
    fn after_begin(&mut self, stage: &mut Stage) {
        let mobject = self.state.mobject();
        if let (Some(starting), Some(target_copy)) =
            (self.state.starting_mobject(), self.target_copy)
        {
            lock_matching_data(stage, mobject, starting, target_copy);
        }
    }

    /// transform.py:74 — unlock in `finish`.
    fn teardown(&mut self, stage: &mut Stage) {
        unlock_family_data(stage, self.state.mobject());
    }

    /// transform.py:89 — first apply the base remover rule, then replace the
    /// animated mobject with the original target when requested. Encoding the
    /// operation on the trait object keeps Choreo's generic finish path from
    /// needing a concrete-animation downcast.
    fn clean_up_from_scene(&mut self, stage: &mut Stage) {
        let mobject = self.state.mobject();
        if self.is_remover() {
            stage.remove_from_scene(mobject);
        }
        if self.replace_in_scene {
            stage.remove_from_scene(mobject);
            let _ = stage.add_to_scene(self.target);
        }
    }

    /// `(mobject, starting_mobject, target_copy)` — transform.py:111.
    fn all_mobjects(&self) -> Vec<Mob> {
        let mut mobs = vec![self.state.mobject()];
        if let Some(starting) = self.state.starting_mobject() {
            mobs.push(starting);
        }
        if let Some(target_copy) = self.target_copy {
            mobs.push(target_copy);
        }
        mobs
    }

    fn preflight_mobjects(&self) -> Vec<Mob> {
        vec![self.state.mobject(), self.target]
    }

    fn interpolate_submobject(&mut self, stage: &mut Stage, mobs: &[Mob], sub_alpha: f64) {
        let [submob, starting, target] = *mobs else {
            return; // rows are triples once begin has run
        };
        interpolate_fields(stage, submob, starting, target, sub_alpha, self.path);
    }
}

// ------------------------------------------------------------------- zoo

/// `ReplacementTransform` (transform.py:132): a Transform whose clean-up
/// swaps the mobject for the target in the scene.
#[must_use]
pub fn replacement_transform(mobject: Mob, target: Mob) -> Transform {
    let mut t = Transform::new(mobject, target).with_replace_flag();
    t.state.config.name = "ReplacementTransform".to_owned();
    t
}

/// `TransformFromCopy` (transform.py:136): animate a fresh copy of the
/// source onto the target, replacing in scene.
///
/// # Errors
/// [`AnimError::StaleHandle`] / [`AnimError::Stage`].
pub fn transform_from_copy(
    stage: &mut Stage,
    mobject: Mob,
    target: Mob,
) -> Result<Transform, AnimError> {
    let copy = stage.copy_family(mobject)?;
    let mut t = Transform::new(copy, target).with_replace_flag();
    t.state.config.name = "TransformFromCopy".to_owned();
    Ok(t)
}

/// `MoveToTarget` (transform.py:143): transform onto the §8.3
/// `generate_target` copy.
///
/// # Errors
/// [`AnimError::MissingTarget`] without a generated target.
pub fn move_to_target(stage: &Stage, mobject: Mob) -> Result<Transform, AnimError> {
    let target = stage.target(mobject).ok_or(AnimError::MissingTarget)?;
    let mut t = Transform::new(mobject, target);
    t.state.config.name = "MoveToTarget".to_owned();
    Ok(t)
}

/// `Restore` (transform.py:248): transform back onto the §8.3
/// `save_state` copy.
///
/// # Errors
/// [`AnimError::MissingSavedState`] without a saved state.
pub fn restore(stage: &Stage, mobject: Mob) -> Result<Transform, AnimError> {
    let saved = stage
        .saved_state(mobject)
        .ok_or(AnimError::MissingSavedState)?;
    let mut t = Transform::new(mobject, saved);
    t.state.config.name = "Restore".to_owned();
    Ok(t)
}

/// The `ApplyFunction` mechanism (transform.py:255): the target is a copy
/// of the mobject with `f` applied — `ApplyMethod` and its whole
/// parameterization family reduce to this under the arena (a bound method
/// call is a stage operation on the copy).
///
/// # Errors
/// [`AnimError::StaleHandle`] / [`AnimError::Stage`].
pub fn apply_function(
    stage: &mut Stage,
    mobject: Mob,
    f: impl FnOnce(&mut Stage, Mob),
) -> Result<Transform, AnimError> {
    let target = stage.copy_family(mobject)?;
    f(stage, target);
    let mut t = Transform::new(mobject, target);
    t.state.config.name = "ApplyFunction".to_owned();
    Ok(t)
}

/// `ApplyPointwiseFunction` (transform.py:198):
/// `ApplyMethod(mobject.apply_function, f)` at `run_time = 3` — the
/// target is a copy with every family point mapped through `f` about the
/// true origin (`apply_function`'s default pivot).
///
/// # Errors
/// As [`apply_function`].
pub fn apply_pointwise_function(
    stage: &mut Stage,
    mobject: Mob,
    f: impl Fn(Vec3) -> Vec3,
) -> Result<Transform, AnimError> {
    let mut t = apply_function(stage, mobject, |s, m| {
        s.apply_points_function(m, f, Some([0.0, 0.0, 0.0]), None);
    })?;
    t.state.config.name = "ApplyPointwiseFunction".to_owned();
    t.state.config.run_time = 3.0;
    Ok(t)
}

/// `ApplyPointwiseFunctionToCenter` (transform.py:209): the target is a
/// copy moved to `f(center)`.
///
/// # Errors
/// As [`apply_function`].
pub fn apply_pointwise_function_to_center(
    stage: &mut Stage,
    mobject: Mob,
    f: impl FnOnce(Vec3) -> Vec3,
) -> Result<Transform, AnimError> {
    let center = stage.get_center(mobject);
    let destination = f(center);
    let mut t = apply_function(stage, mobject, |s, m| {
        s.move_to(m, destination, [0.0, 0.0, 0.0]);
    })?;
    t.state.config.name = "ApplyPointwiseFunctionToCenter".to_owned();
    Ok(t)
}

/// `FadeToColor` (transform.py:223): `ApplyMethod(mobject.set_color, c)`.
/// The color is retained now; the live family is copied and recolored at begin.
///
/// # Errors
/// As [`apply_function`].
pub fn fade_to_color(
    stage: &mut Stage,
    mobject: Mob,
    rgb: [f32; 3],
) -> Result<Transform, AnimError> {
    if !stage.contains(mobject) {
        return Err(AnimError::StaleHandle(mobject));
    }
    let mut t = Transform::new(mobject, mobject).with_target_prep(TargetPrep::Color(rgb));
    t.state.config.name = "FadeToColor".to_owned();
    Ok(t)
}

/// `ApplyMatrix` (transform.py:272): `ApplyPointwiseFunction` under
/// `p ↦ M·p` (the Reference's `np.dot(p, M.T)`). The fixed matrix is
/// applied to a copy of the live world-space geometry at each begin, including
/// after a predecessor and on replay. Default run time remains three seconds.
///
/// # Errors
/// As [`apply_function`].
pub fn apply_matrix(
    stage: &mut Stage,
    mobject: Mob,
    matrix: [[f64; 3]; 3],
) -> Result<Transform, AnimError> {
    if !stage.contains(mobject) {
        return Err(AnimError::StaleHandle(mobject));
    }
    let mut t = Transform::new(mobject, mobject).with_target_prep(TargetPrep::Matrix(matrix));
    t.state.config.name = "ApplyMatrix".to_owned();
    t.state.config.run_time = 3.0;
    Ok(t)
}

/// The Reference's 2×2 acceptance for [`apply_matrix`]: embedded into the
/// identity's upper-left block (transform.py:286).
///
/// # Errors
/// As [`apply_matrix`].
pub fn apply_matrix_2d(
    stage: &mut Stage,
    mobject: Mob,
    m: [[f64; 2]; 2],
) -> Result<Transform, AnimError> {
    apply_matrix(
        stage,
        mobject,
        [
            [m[0][0], m[0][1], 0.0],
            [m[1][0], m[1][1], 0.0],
            [0.0, 0.0, 1.0],
        ],
    )
}

/// `ApplyComplexFunction` (transform.py:297): map the `(x, y)` plane
/// through a complex function (`z` kept), with the Reference's derived
/// `path_arc = Im(log(f(1)))` — the argument of `f(1)`.
///
/// # Errors
/// As [`apply_function`].
pub fn apply_complex_function(
    stage: &mut Stage,
    mobject: Mob,
    f: impl Fn(f64, f64) -> (f64, f64),
) -> Result<Transform, AnimError> {
    let (re1, im1) = f(1.0, 0.0);
    let path_arc = fmn_dmath::atan2(im1, re1);
    let mut t = apply_function(stage, mobject, |s, m| {
        s.apply_points_function(
            m,
            |p| {
                let (re, im) = f(p[0], p[1]);
                [re, im, p[2]]
            },
            Some([0.0, 0.0, 0.0]),
            None,
        );
    })?
    .with_path_arc(path_arc, [0.0, 0.0, 1.0]);
    t.state.config.name = "ApplyComplexFunction".to_owned();
    Ok(t)
}

/// `ScaleInPlace` (transform.py:233): `ApplyMethod(mobject.scale, k)`.
/// Scale a fresh begin-time family about its current center, not the center
/// or extent it had when this animation was constructed.
///
/// # Errors
/// As [`apply_function`].
pub fn scale_in_place(
    stage: &mut Stage,
    mobject: Mob,
    scale_factor: f64,
) -> Result<Transform, AnimError> {
    if !stage.contains(mobject) {
        return Err(AnimError::StaleHandle(mobject));
    }
    let mut t = Transform::new(mobject, mobject).with_target_prep(TargetPrep::Scale(scale_factor));
    t.state.config.name = "ScaleInPlace".to_owned();
    Ok(t)
}

/// `ShrinkToCenter` (transform.py:243): `ScaleInPlace(mobject, 0)`.
///
/// # Errors
/// As [`apply_function`].
pub fn shrink_to_center(stage: &mut Stage, mobject: Mob) -> Result<Transform, AnimError> {
    let mut t = scale_in_place(stage, mobject, 0.0)?;
    t.state.config.name = "ShrinkToCenter".to_owned();
    Ok(t)
}

/// `CyclicReplace` (transform.py:316): each mobject transforms (along a
/// 90° arc by default) onto the position of its cyclic predecessor's
/// slot: targets are copies moved to `mobjects[(i+1) % n]`'s center.
/// Returns one Transform per mobject (the composition bead, fm-hfe, wraps
/// them in a group).
///
/// # Errors
/// [`AnimError::StaleHandle`] / [`AnimError::Stage`].
pub fn cyclic_replace(
    stage: &mut Stage,
    mobjects: &[Mob],
    path_arc: f64,
) -> Result<Vec<Transform>, AnimError> {
    let centers: Vec<Vec3> = mobjects.iter().map(|&m| stage.get_center(m)).collect();
    let n = mobjects.len();
    let mut out = Vec::with_capacity(n);
    for (i, &mob) in mobjects.iter().enumerate() {
        let dest = centers[(i + 1) % n];
        let mut t = apply_function(stage, mob, |s, m| {
            let center = s.get_center(m);
            s.shift(
                m,
                [
                    dest[0] - center[0],
                    dest[1] - center[1],
                    dest[2] - center[2],
                ],
            );
        })?
        .with_path_arc(path_arc, [0.0, 0.0, 1.0]);
        t.state.config.name = "CyclicReplace".to_owned();
        out.push(t);
    }
    Ok(out)
}

/// `Swap` (transform.py:329): `CyclicReplace` under its common two-element
/// name, same 90° default arc.
///
/// # Errors
/// As [`cyclic_replace`].
pub fn swap(stage: &mut Stage, a: Mob, b: Mob) -> Result<Vec<Transform>, AnimError> {
    let mut out = cyclic_replace(stage, &[a, b], std::f64::consts::FRAC_PI_2)?;
    for t in &mut out {
        t.state.config.name = "Swap".to_owned();
    }
    Ok(out)
}
