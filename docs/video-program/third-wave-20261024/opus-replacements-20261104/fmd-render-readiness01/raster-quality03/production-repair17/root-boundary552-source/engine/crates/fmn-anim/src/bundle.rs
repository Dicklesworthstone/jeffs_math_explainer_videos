//! The FMTL/1 segment-reconstruction law (fm-oee,
//! `docs/FMNT1_TIMELINE_BUNDLE.md`) — the ONE implementation both sides of
//! the timeline bundle share: the scene-side exporter proves against it,
//! the wasm player replays through it. A second, hand-rolled lerp
//! convention anywhere would be drift against the pinned contract, so the
//! player never interpolates records itself: it calls
//! [`interpolate_between`], which routes every field through
//! [`interpolate_fields`] exactly as
//! the contract's normative law requires — pointlike fields through the
//! path function, every other field linear, locked fields skipped, f64
//! math stored at record precision.
//!
//! The tag tables (`path: u8`, `rate: u8`) are the exporter's normative
//! catalog: [`PathFunc::Straight`] is the only path a bare tag can
//! identify (an arc carries parameters a `u8` cannot), and the rate tags
//! delegate to [`fmn_core::rate::TAG_CATALOG`]. Anything untaggable is
//! exported statefully by the writer — never guessed.

use fmn_mobject::{Snapshot, Stage};

use crate::animation::{RateFunc, clip};
use crate::transform::{PathFunc, interpolate_fields};

/// Version of the shared frame reconstruction semantics, independent of the
/// rasterizer. Version 1 was implicit in renderer-only FMTL identities. Version
/// 2 includes typed tracker payloads in interpolation and uses the stable
/// translation-independent affine lift. Bump on any replay-law change that can
/// alter state or bits, not on bit-preserving implementation refactors.
pub const RECONSTRUCTION_LAW_VERSION: u32 = 2;

/// The `path` tag of [`PathFunc::Straight`] — catalog entry 0.
pub const PATH_STRAIGHT_TAG: u8 = 0;

/// The path function a `path` tag names, or `None` for a tag the catalog
/// does not define (the reader's named refusal).
#[must_use]
pub fn path_from_tag(tag: u8) -> Option<PathFunc> {
    match tag {
        PATH_STRAIGHT_TAG => Some(PathFunc::Straight),
        _ => None,
    }
}

/// The tag of a representable path function. `Arc` carries an angle and an
/// axis a bare tag cannot identify, so it is deliberately untaggable: the
/// exporter records arc-path segments statefully.
#[must_use]
pub fn path_tag(path: PathFunc) -> Option<u8> {
    match path {
        PathFunc::Straight => Some(PATH_STRAIGHT_TAG),
        PathFunc::Arc { .. } => None,
    }
}

/// The rate function a `rate` tag names, as composable data.
#[must_use]
pub fn rate_from_tag(tag: u8) -> Option<RateFunc> {
    fmn_core::rate::from_tag(tag).map(RateFunc::Base)
}

/// The tag of a catalog rate function, or `None` for combinators and
/// anything outside the catalog.
#[must_use]
pub fn rate_tag(rate: &RateFunc) -> Option<u8> {
    match rate {
        RateFunc::Base(func) => fmn_core::rate::tag_of(*func),
        _ => None,
    }
}

/// The bundle's normalized alpha: the normalized-alpha pipeline
/// (§9.4) frozen at the only configuration whole-segment record
/// interpolation can represent — no `time_span`, zero lag — which
/// collapses `sub_alpha` to `rate(clip(alpha, 0, 1))`. The writer
/// nominates a pure segment for kind-0 export only when every animation
/// in it has exactly this pipeline shape, so the player's
/// `a = rate(alpha)` is the engine's own sub-alpha, bit for bit.
#[must_use]
pub fn bundle_sub_alpha(alpha: f64, rate: &RateFunc) -> f64 {
    rate.eval(clip(alpha, 0.0, 1.0))
}

/// Whether two stage entries agree on everything interpolation could
/// write: record columns, object→world placement, and the numeric
/// uniforms and typed tracker payloads. Compared on plain values — both sides are expected to come
/// from the canonical container (the player decodes it; the writer
/// round-trips through it before proving), where `-0.0`/NaN payloads are
/// already canonicalized, so value equality is bit equality.
///
/// `z_index`, shape tags, and animation/updater flags are deliberately
/// absent: interpolation never writes them, so they cannot make a frame
/// diverge.
fn interpolation_identical(stage: &Stage, end_stage: &Stage, mob: fmn_mobject::Mob) -> bool {
    let (Some(a), Some(b)) = (stage.get(mob), end_stage.get(mob)) else {
        return false;
    };
    if a.placement().coefficients() != b.placement().coefficients() {
        return false;
    }
    if stage.tracker(mob) != end_stage.tracker(mob) {
        return false;
    }
    if a.uniforms() != b.uniforms() {
        return false;
    }
    let schema = a.buffer.schema();
    if schema != b.buffer.schema() {
        return false;
    }
    schema
        .fields()
        .iter()
        .all(|field| a.buffer.read_column(&field.name) == b.buffer.read_column(&field.name))
}

/// Reconstruct one frame's stage by whole-stage record interpolation
/// between two snapshots of one logical arena — FMTL/1's normative law.
///
/// Every rooted family member whose interpolatable state differs between
/// `begin` and `end` is lerped toward a private copy of its `end` state at
/// `alpha` through [`interpolate_fields`]. A member shared by several scene
/// roots is visited once, in stable first-seen root/family order; root
/// multiplicity is a draw-order fact, not repeated state mutation. Members
/// that agree exactly are left untouched, which is precisely what the engine
/// does to them (a mob no animation's family row touches keeps its begin
/// values — and an out-and-back animation whose end state equals its begin
/// state fails the writer's export-time proof and never reaches this path as
/// kind 0).
///
/// `begin` and `end` must share one handle domain (snapshots of the same
/// logical stage, or both decoded against the same binding stage — the
/// player's case). The returned stage is `begin` materialized plus the
/// unrooted end-state copies interpolation read from; those copies are
/// never rooted, so the render plan never sees them.
#[must_use]
pub fn interpolate_between(begin: &Snapshot, end: &Snapshot, alpha: f64, path: PathFunc) -> Stage {
    let mut stage = begin.materialize();
    let end_stage = end.materialize();
    let roots = stage.roots().to_vec();
    let mut mobs = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for root in roots {
        for mob in stage.family(root) {
            if seen.insert(mob) {
                mobs.push(mob);
            }
        }
    }
    for mob in mobs {
        if interpolation_identical(&stage, &end_stage, mob) {
            continue;
        }
        let Ok(target) = end_stage.copy_into(mob, &mut stage) else {
            continue; // a mob the end state no longer carries keeps its begin values
        };
        interpolate_fields(&mut stage, mob, mob, target, alpha, path);
    }
    stage
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::AnimConfig;
    use fmn_core::rate;
    use fmn_mobject::Mobject;

    #[test]
    fn path_tags_round_trip_the_representable_catalog() {
        assert_eq!(path_tag(PathFunc::Straight), Some(PATH_STRAIGHT_TAG));
        assert_eq!(path_from_tag(PATH_STRAIGHT_TAG), Some(PathFunc::Straight));
        assert!(path_from_tag(1).is_none(), "unknown path tags refuse");
        assert!(
            path_tag(PathFunc::Arc {
                angle: 1.0,
                axis: [0.0, 0.0, 1.0],
            })
            .is_none(),
            "parameterized arcs are untaggable"
        );
    }

    #[test]
    fn rate_tags_round_trip_the_catalog() {
        for (tag, &func) in fmn_core::rate::TAG_CATALOG.iter().enumerate() {
            let tag = u8::try_from(tag).unwrap_or(u8::MAX);
            let rate = rate_from_tag(tag).unwrap_or(RateFunc::Base(rate::linear));
            assert_eq!(rate_tag(&rate), Some(tag));
            let t = 0.37;
            assert_eq!(rate.eval(t).to_bits(), func(t).to_bits());
        }
        assert!(rate_from_tag(u8::MAX).is_none());
        let squished = RateFunc::smooth().squish(0.4, 0.6);
        assert!(rate_tag(&squished).is_none(), "combinators are untaggable");
    }

    #[test]
    fn bundle_sub_alpha_is_the_lag_free_span_free_pipeline() {
        // The engine's own sub_alpha at lag_ratio = 0, one or many family
        // rows, no time_span, must equal rate(clip(alpha)) bit for bit —
        // that equality is what makes the player's single-alpha law the
        // engine's law for nominated segments.
        let rate = RateFunc::smooth();
        for alpha in [-0.25, 0.0, 0.31, 0.5, 1.0, 1.4] {
            let expected = bundle_sub_alpha(alpha, &rate);
            for (index, num) in [(0, 1), (0, 3), (2, 3)] {
                let engine = crate::animation::sub_alpha(alpha, index, num, 0.0, &rate);
                assert_eq!(
                    engine.to_bits(),
                    expected.to_bits(),
                    "alpha={alpha} row {index}/{num}"
                );
            }
        }
        // Clipping is the pipeline's clip, branch for branch.
        let linear = RateFunc::linear();
        assert_eq!(bundle_sub_alpha(1.4, &linear), 1.0);
        assert_eq!(bundle_sub_alpha(-0.2, &linear), 0.0);
        let config = AnimConfig::default();
        assert_eq!(config.lag_ratio, 0.0, "the default pipeline is lag-free");
    }

    #[test]
    fn shared_members_are_interpolated_once_across_scene_roots() {
        let mut stage = Stage::new();
        let shared = stage.add(Mobject::from_points(&[[0.0, 0.0, 0.0]]));
        let left = stage.add(Mobject::new());
        let right = stage.add(Mobject::new());
        stage.attach(left, shared).expect("left parent");
        stage.attach(right, shared).expect("right parent");
        stage
            .add_many_to_scene(&[left, right])
            .expect("shared roots");

        let begin = stage.snapshot();
        stage.shift(shared, [2.0, 0.0, 0.0]);
        let end = stage.snapshot();

        let midpoint = interpolate_between(&begin, &end, 0.5, PathFunc::Straight);
        assert_eq!(
            midpoint.get_center(shared),
            [1.0, 0.0, 0.0],
            "root multiplicity must not compound the shared member interpolation"
        );
    }
}
