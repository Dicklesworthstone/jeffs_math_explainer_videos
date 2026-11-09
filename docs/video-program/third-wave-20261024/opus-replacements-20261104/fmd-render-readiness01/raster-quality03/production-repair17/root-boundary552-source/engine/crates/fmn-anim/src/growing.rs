//! The grow mechanism — part of family 3 (§9.4, fm-cye), ported from the
//! pinned Reference's `animation/growing.py`: every class is a Transform
//! whose *starting* copy is collapsed (`scale(0)`) onto an anchor point,
//! optionally recolored — the [`StartPrep`] sequence, exactly the
//! `create_starting_mobject` override at growing.py:30.
//!
//! `SpinInFromNothing` does not exist in the pinned tree (the fm-cye bead
//! note records the finding); the spin-flavored entry is indication.py's
//! `WiggleOutThenIn`, which lands with the indication family.

use fmn_core::types::Vec3;
use fmn_mobject::{Mob, Stage};

use crate::animation::{AnimError, Animation};
use crate::transform::{StartPrep, Transform};

/// `GrowFromPoint` (growing.py:16): transform from a zero-scale copy at
/// `point` (optionally colored `point_color`) onto a copy of the mobject
/// as it stands at `begin`, not when this function is called. The anchor
/// remains construction-time state, including when replaying the animation.
///
/// # Errors
/// [`AnimError::StaleHandle`] / [`AnimError::Stage`].
pub fn grow_from_point(
    stage: &mut Stage,
    mobject: Mob,
    point: Vec3,
    point_color: Option<[f32; 3]>,
) -> Result<Transform, AnimError> {
    if !stage.contains(mobject) {
        return Err(AnimError::StaleHandle(mobject));
    }
    // Keep the live handle until Transform::setup snapshots it. A later
    // Succession child must observe its predecessor's geometry and paint.
    let mut t = Transform::new(mobject, mobject).with_start_prep(StartPrep {
        scale: Some(0.0),
        move_to: Some(point),
        color: point_color,
        ..StartPrep::default()
    });
    t.state_mut().config.name = "GrowFromPoint".to_owned();
    Ok(t)
}

/// `GrowFromCenter` (growing.py:40): the anchor is the mobject's center.
///
/// # Errors
/// As [`grow_from_point`].
pub fn grow_from_center(
    stage: &mut Stage,
    mobject: Mob,
    point_color: Option<[f32; 3]>,
) -> Result<Transform, AnimError> {
    let point = stage.get_center(mobject);
    let mut t = grow_from_point(stage, mobject, point, point_color)?;
    t.state_mut().config.name = "GrowFromCenter".to_owned();
    Ok(t)
}

/// `GrowFromEdge` (growing.py:46): the anchor is the bounding-box point
/// toward `edge`.
///
/// # Errors
/// As [`grow_from_point`].
pub fn grow_from_edge(
    stage: &mut Stage,
    mobject: Mob,
    edge: Vec3,
    point_color: Option<[f32; 3]>,
) -> Result<Transform, AnimError> {
    let point = stage.get_bounding_box_point(mobject, edge);
    let mut t = grow_from_point(stage, mobject, point, point_color)?;
    t.state_mut().config.name = "GrowFromEdge".to_owned();
    Ok(t)
}

/// `GrowArrow` (growing.py:52): the anchor is the arrow's start point.
///
/// # Errors
/// [`AnimError::EmptyMobject`] on a pointless mobject (the Reference's
/// `IndexError`), else as [`grow_from_point`].
pub fn grow_arrow(stage: &mut Stage, arrow: Mob) -> Result<Transform, AnimError> {
    let point = stage.get_start(arrow).ok_or(AnimError::EmptyMobject)?;
    let mut t = grow_from_point(stage, arrow, point, None)?;
    t.state_mut().config.name = "GrowArrow".to_owned();
    Ok(t)
}
