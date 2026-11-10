//! `TransformMatchingParts` / `TransformMatchingShapes`, ported from the
//! pinned Reference's `animation/transform_matching_parts.py`:
//!
//! - [`has_same_shape_as`] (mobject.py:770): both point sets centered and
//!   height-normalized, then `np.isclose(atol = width·1e-2)` — ported
//!   comparison for comparison (a degenerate zero-height side never
//!   matches, exactly as the Reference's NaN propagation decides).
//! - [`transform_matching_parts`] builds the animation list the
//!   Reference's `AnimationGroup` wraps: user-specified pairs first, then
//!   every same-shape pair from the piece product, then the leftovers —
//!   sources fade out to the target's center, targets fade in from the
//!   source's center (the `FadeOutToPoint`/`FadeInFromPoint` calls at
//!   transform_matching_parts.py:57). Grouping (`run_time = 2`,
//!   `lag_ratio = 0`) is the composition bead's (fm-hfe), which wraps the
//!   returned list — the same seam [`crate::transform::cyclic_replace`]
//!   established. Scene-side cleanup (remove the pieces, add the real
//!   target) is fm-5xm's, per the remover flags carried on the fades.
//! - `TransformMatchingShapes` is the Reference's literal alias — same
//!   function, same defaults.
//!
//! [`transform_matching_keys`] is the string/Tex seam. Scribe supplies one
//! semantic key per pointful native-span primitive; Choreo pairs equal keys
//! in stable occurrence order and fades the leftovers. It never shape-matches
//! Tex and never reconstructs provenance from rendered geometry (D-09).

use fmn_mobject::{Mob, Stage};

use crate::animation::{AnimError, Animation};
use crate::fading::{fade_in_from_point, fade_out_to_point};
use crate::transform::Transform;

/// The family members with points, in family order (mobject.py:435).
fn pieces_of(stage: &Stage, mob: Mob) -> Vec<Mob> {
    stage
        .family(mob)
        .into_iter()
        .filter(|&m| stage.get(m).is_some_and(|e| !e.buffer.is_empty()))
        .collect()
}

/// All family points of `mob`, concatenated in family order (the
/// Reference's `get_all_points`).
fn all_points(stage: &Stage, mob: Mob) -> Vec<f64> {
    let mut out = Vec::new();
    for member in stage.family(mob) {
        if let Some(points) = stage.get_points(member) {
            out.extend(points.into_iter().flatten());
        }
    }
    out
}

/// Reference `has_same_shape_as` (mobject.py:770): center both point
/// sets, normalize by height, compare with
/// `np.isclose(atol = self.width·1e-2)` (default `rtol = 1e-5`).
#[must_use]
pub fn has_same_shape_as(stage: &Stage, a: Mob, b: Mob) -> bool {
    let pa = all_points(stage, a);
    let pb = all_points(stage, b);
    if pa.len() != pb.len() {
        return false;
    }
    if pa.is_empty() {
        return true; // np.isclose(...).all() over empty arrays is True
    }
    let normalize = |stage: &Stage, mob: Mob, points: &[f64]| -> Vec<f64> {
        let center = stage.get_center(mob);
        let height = stage.get_height(mob);
        points
            .as_chunks::<3>()
            .0
            .iter()
            .flat_map(|p| {
                [
                    (p[0] - center[0]) / height,
                    (p[1] - center[1]) / height,
                    (p[2] - center[2]) / height,
                ]
            })
            .collect()
    };
    let na = normalize(stage, a, &pa);
    let nb = normalize(stage, b, &pb);
    let atol = stage.get_width(a) * 1e-2;
    // np.isclose: |x − y| <= atol + rtol·|y|. NaN (zero height) fails
    // every comparison, exactly as the Reference decides degenerates.
    na.iter()
        .zip(&nb)
        .all(|(&x, &y)| (x - y).abs() <= atol + 1e-5 * y.abs())
}

/// `TransformMatchingParts` (transform_matching_parts.py:21) /
/// `TransformMatchingShapes` (its literal alias): the matched-pair
/// animation list. `matched_pairs` claims first (in order), the
/// same-shape product second, and the leftovers fade. The composition
/// bead (fm-hfe) wraps the list at `run_time = 2`, `lag_ratio = 0`.
///
/// # Errors
/// [`AnimError::StaleHandle`] / [`AnimError::Stage`].
pub fn transform_matching_parts(
    stage: &mut Stage,
    source: Mob,
    target: Mob,
    matched_pairs: &[(Mob, Mob)],
) -> Result<Vec<Box<dyn Animation>>, AnimError> {
    if !stage.contains(source) {
        return Err(AnimError::StaleHandle(source));
    }
    if !stage.contains(target) {
        return Err(AnimError::StaleHandle(target));
    }
    let mut source_pieces = pieces_of(stage, source);
    let mut target_pieces = pieces_of(stage, target);
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();

    // transform_matching_parts.py:76 add_transform, call for call. Both
    // `match_animation` and `mismatch_animation` are Transform at the
    // Reference defaults, so the shape probe decides nothing here — the
    // parameterized variants that override one of them arrive with their
    // consumers.
    fn add_transform(
        stage: &Stage,
        s: Mob,
        t: Mob,
        anims: &mut Vec<Box<dyn Animation>>,
        source_pieces: &mut Vec<Mob>,
        target_pieces: &mut Vec<Mob>,
    ) {
        let new_source = pieces_of(stage, s);
        let new_target = pieces_of(stage, t);
        if new_source.is_empty() || new_target.is_empty() {
            return; // never animate null sources or targets
        }
        if !new_source.iter().all(|m| source_pieces.contains(m))
            || !new_target.iter().all(|m| target_pieces.contains(m))
        {
            return; // already claimed
        }
        anims.push(Box::new(Transform::new(s, t)));
        source_pieces.retain(|m| !new_source.contains(m));
        target_pieces.retain(|m| !new_target.contains(m));
    }

    for &(s, t) in matched_pairs {
        add_transform(
            stage,
            s,
            t,
            &mut anims,
            &mut source_pieces,
            &mut target_pieces,
        );
    }
    // The same-shape product over the piece lists as they stand after
    // user pairs claimed theirs (transform_matching_parts.py:50).
    let product_pairs: Vec<(Mob, Mob)> = source_pieces
        .iter()
        .flat_map(|&s| target_pieces.iter().map(move |&t| (s, t)))
        .filter(|&(s, t)| has_same_shape_as(stage, s, t))
        .collect();
    for (s, t) in product_pairs {
        add_transform(
            stage,
            s,
            t,
            &mut anims,
            &mut source_pieces,
            &mut target_pieces,
        );
    }

    // Leftovers: sources fade out to the target's center, targets fade
    // in from the source's center.
    let target_center = stage.get_center(target);
    for piece in source_pieces {
        anims.push(Box::new(fade_out_to_point(stage, piece, target_center)?));
    }
    let source_center = stage.get_center(source);
    for piece in target_pieces {
        anims.push(Box::new(fade_in_from_point(stage, piece, source_center)?));
    }
    Ok(anims)
}

/// Match Scribe primitives by native source-span identity.
///
/// Equal keys are paired in stable source/target occurrence order. A target
/// occurrence is claimed at most once; unpaired source and target primitives
/// use the same directional fades as [`transform_matching_parts`]. The keys
/// are intentionally opaque to Choreo: `fmn-tex` owns their provenance and
/// the portal merely carries that already-native identity across the crate
/// boundary.
///
/// # Errors
/// [`AnimError::StaleHandle`] when either root or any keyed primitive is not
/// live in `stage`, or [`AnimError::Stage`] from a fade constructor.
pub fn transform_matching_keys(
    stage: &mut Stage,
    source: Mob,
    target: Mob,
    source_parts: &[(Mob, String)],
    target_parts: &[(Mob, String)],
) -> Result<Vec<Box<dyn Animation>>, AnimError> {
    for mob in std::iter::once(source)
        .chain(std::iter::once(target))
        .chain(source_parts.iter().map(|(mob, _)| *mob))
        .chain(target_parts.iter().map(|(mob, _)| *mob))
    {
        if !stage.contains(mob) {
            return Err(AnimError::StaleHandle(mob));
        }
    }

    let mut target_claimed = vec![false; target_parts.len()];
    let mut source_unmatched = Vec::new();
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for (source_mob, source_key) in source_parts {
        let target_index = target_parts
            .iter()
            .enumerate()
            .position(|(index, (_, target_key))| {
                !target_claimed[index] && source_key == target_key
            });
        if let Some(index) = target_index {
            target_claimed[index] = true;
            anims.push(Box::new(Transform::new(*source_mob, target_parts[index].0)));
        } else {
            source_unmatched.push(*source_mob);
        }
    }

    let target_center = stage.get_center(target);
    for mob in source_unmatched {
        anims.push(Box::new(fade_out_to_point(stage, mob, target_center)?));
    }
    let source_center = stage.get_center(source);
    for (index, (mob, _)) in target_parts.iter().enumerate() {
        if !target_claimed[index] {
            anims.push(Box::new(fade_in_from_point(stage, *mob, source_center)?));
        }
    }
    Ok(anims)
}
