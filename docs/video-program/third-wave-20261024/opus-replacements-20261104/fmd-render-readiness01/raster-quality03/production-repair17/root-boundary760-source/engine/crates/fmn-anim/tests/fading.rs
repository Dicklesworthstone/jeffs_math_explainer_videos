//! Fade/grow mechanism semantics (fm-cye, §9.4 family 3): the Fade zoo's
//! start/target-copy rules from fading.py, the VFade opacity ramps, the
//! FadeTransform ghosting order, and growing.py's collapsed-start
//! parameterizations.

use fmn_anim::animation::Animation;
use fmn_anim::{
    AnimError, RateFunc, fade_in, fade_in_from_point, fade_out, fade_transform,
    fade_transform_pieces, grow_arrow, grow_from_center, grow_from_edge, v_fade_in,
    v_fade_in_then_out, v_fade_out,
};
use fmn_mobject::record::{RecordBuffer, RecordSchema};
use fmn_mobject::{Mob, Mobject, Stage};

fn vmob(stage: &mut Stage, points: &[[f64; 3]]) -> Mob {
    let mob = stage.add(Mobject::new());
    let entry = stage.get_mut(mob).unwrap();
    entry.buffer = RecordBuffer::new(RecordSchema::vmobject(), points.len()).unwrap();
    #[allow(clippy::cast_possible_truncation)]
    let flat: Vec<f32> = points
        .iter()
        .flat_map(|p| p.iter().map(|v| *v as f32))
        .collect();
    entry.buffer.write_range("point", 0, &flat);
    entry
        .buffer
        .write_range("fill_rgba", 0, &[0.2, 0.4, 0.6, 1.0].repeat(points.len()));
    entry
        .buffer
        .write_range("stroke_rgba", 0, &[1.0, 0.0, 0.0, 0.8].repeat(points.len()));
    mob
}

/// A 2-curve straight-line vmobject with x = [0, 1, 2, 3, 4].
fn line5(stage: &mut Stage) -> Mob {
    vmob(
        stage,
        &[
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
        ],
    )
}

fn xs_of(stage: &Stage, mob: Mob) -> Vec<f32> {
    #[allow(clippy::cast_possible_truncation)]
    stage
        .get_points(mob)
        .unwrap()
        .into_iter()
        .map(|point| point[0] as f32)
        .collect()
}

fn column(stage: &Stage, mob: Mob, field: &str) -> Vec<f32> {
    stage.get(mob).unwrap().buffer.read_column(field).unwrap()
}

fn assert_close(actual: &[f32], expected: &[f32], tol: f32) {
    assert_eq!(actual.len(), expected.len());
    for (a, e) in actual.iter().zip(expected) {
        assert!((a - e).abs() <= tol, "{actual:?} !~ {expected:?}");
    }
}

// ---------------------------------------------------------------- fades

#[test]
fn fade_in_starts_invisible_scaled_and_shifted() {
    let mut stage = Stage::new();
    let mob = line5(&mut stage);
    let mut anim = fade_in(&mut stage, mob, [1.0, 0.0, 0.0], 2.0).unwrap();
    anim.state_mut().config.rate_func = RateFunc::linear();
    anim.begin(&mut stage).unwrap();
    // Start: opacity 0, scale(1/2) about center (x=2), shift(−1).
    assert_close(&xs_of(&stage, mob), &[0.0, 0.5, 1.0, 1.5, 2.0], 1e-6);
    assert_eq!(column(&stage, mob, "fill_rgba")[3], 0.0);
    anim.finish(&mut stage);
    assert_close(&xs_of(&stage, mob), &[0.0, 1.0, 2.0, 3.0, 4.0], 1e-6);
    assert_eq!(column(&stage, mob, "fill_rgba")[3], 1.0);
    assert!(!anim.is_remover());
}

#[test]
fn fade_out_finishes_back_in_original_state() {
    let mut stage = Stage::new();
    let mob = line5(&mut stage);
    let mut anim = fade_out(&mut stage, mob, [0.0, 0.0, 0.0], 1.0).unwrap();
    anim.state_mut().config.rate_func = RateFunc::linear();
    assert!(anim.is_remover());
    assert_eq!(anim.state().config.final_alpha_value, 0.0);
    anim.begin(&mut stage).unwrap();
    anim.interpolate(&mut stage, 0.5);
    assert!((column(&stage, mob, "fill_rgba")[3] - 0.5).abs() < 1e-6);
    // final_alpha_value = 0: the remover leaves the mobject as it was.
    anim.finish(&mut stage);
    assert_eq!(column(&stage, mob, "fill_rgba")[3], 1.0);
    assert_close(&xs_of(&stage, mob), &[0.0, 1.0, 2.0, 3.0, 4.0], 1e-6);
}

#[test]
fn fade_in_from_point_collapses_start_onto_point() {
    let mut stage = Stage::new();
    let mob = line5(&mut stage);
    let mut anim = fade_in_from_point(&mut stage, mob, [7.0, 0.0, 0.0]).unwrap();
    anim.begin(&mut stage).unwrap();
    // scale(1/∞) = scale(0) → clamped tiny; everything sits at the point.
    assert_close(&xs_of(&stage, mob), &[7.0; 5], 1e-5);
}

// ---------------------------------------------------------------- grows

#[test]
fn grow_from_center_starts_collapsed_at_center() {
    let mut stage = Stage::new();
    let mob = line5(&mut stage);
    let original = xs_of(&stage, mob);
    let mut anim = grow_from_center(&mut stage, mob, None).unwrap();
    assert_eq!(anim.state().config.name, "GrowFromCenter");
    anim.begin(&mut stage).unwrap();
    assert_close(&xs_of(&stage, mob), &[2.0; 5], 1e-5);
    // Full opacity throughout — grow is a scale, not a fade.
    assert_eq!(column(&stage, mob, "fill_rgba")[3], 1.0);
    anim.finish(&mut stage);
    assert_close(&xs_of(&stage, mob), &original, 1e-6);
}

#[test]
fn grow_from_edge_anchors_on_the_box_point() {
    let mut stage = Stage::new();
    let mob = line5(&mut stage);
    let mut anim = grow_from_edge(&mut stage, mob, [1.0, 0.0, 0.0], Some([0.0, 1.0, 0.0])).unwrap();
    anim.begin(&mut stage).unwrap();
    // Right edge of the box is x = 4; point_color recolors the start.
    assert_close(&xs_of(&stage, mob), &[4.0; 5], 1e-5);
    let fill = column(&stage, mob, "fill_rgba");
    assert_eq!(&fill[..3], &[0.0, 1.0, 0.0]);
    assert_eq!(fill[3], 1.0);
}

#[test]
fn grow_arrow_on_pointless_mobject_is_the_named_error() {
    let mut stage = Stage::new();
    let empty = vmob(&mut stage, &[]);
    assert!(matches!(
        grow_arrow(&mut stage, empty),
        Err(AnimError::EmptyMobject)
    ));
}

// --------------------------------------------------------------- VFades

#[test]
fn v_fade_in_ramps_stroke_and_fill_opacity() {
    let mut stage = Stage::new();
    let mob = line5(&mut stage);
    let mut anim = v_fade_in(mob);
    anim.state_mut().config.rate_func = RateFunc::linear();
    anim.begin(&mut stage).unwrap();
    assert_eq!(column(&stage, mob, "fill_rgba")[3], 0.0);
    assert_eq!(column(&stage, mob, "stroke_rgba")[3], 0.0);
    anim.interpolate(&mut stage, 0.5);
    assert!((column(&stage, mob, "fill_rgba")[3] - 0.5).abs() < 1e-6);
    assert!((column(&stage, mob, "stroke_rgba")[3] - 0.4).abs() < 1e-6);
    // Points are untouched — VFade composes with updaters.
    assert_close(&xs_of(&stage, mob), &[0.0, 1.0, 2.0, 3.0, 4.0], 0.0);
    anim.finish(&mut stage);
    assert_eq!(column(&stage, mob, "fill_rgba")[3], 1.0);
}

#[test]
fn v_fade_out_runs_the_ramp_reversed() {
    let mut stage = Stage::new();
    let mob = line5(&mut stage);
    let mut anim = v_fade_out(mob);
    anim.state_mut().config.rate_func = RateFunc::linear();
    assert!(anim.is_remover());
    assert_eq!(anim.state().config.final_alpha_value, 0.0);
    anim.begin(&mut stage).unwrap();
    // Reversed: alpha 0 is fully opaque.
    assert_eq!(column(&stage, mob, "fill_rgba")[3], 1.0);
    anim.interpolate(&mut stage, 0.75);
    assert!((column(&stage, mob, "fill_rgba")[3] - 0.25).abs() < 1e-6);
    // final_alpha_value = 0 → reversed ramp lands fully opaque again.
    anim.finish(&mut stage);
    assert_eq!(column(&stage, mob, "fill_rgba")[3], 1.0);
}

#[test]
fn v_fade_in_then_out_carries_reference_flags() {
    let mut stage = Stage::new();
    let mob = line5(&mut stage);
    let mut anim = v_fade_in_then_out(mob);
    assert!(anim.is_remover());
    assert_eq!(anim.state().config.final_alpha_value, 0.5);
    anim.begin(&mut stage).unwrap();
    // there_and_back(0.5) = 1 → fully faded in at the middle.
    anim.interpolate(&mut stage, 0.5);
    assert_eq!(column(&stage, mob, "fill_rgba")[3], 1.0);
}

// --------------------------------------------------------- FadeTransform

#[test]
fn fade_transform_ghosts_and_crossfades() {
    let mut stage = Stage::new();
    let a = line5(&mut stage);
    // A different box: x = [0, 1, 2] at y = 3.
    let b = vmob(
        &mut stage,
        &[[0.0, 3.0, 0.0], [1.0, 3.0, 0.0], [2.0, 3.0, 0.0]],
    );
    let mut anim = fade_transform(&mut stage, a, b).unwrap();
    anim.state_mut().config.rate_func = RateFunc::linear();
    assert_eq!(anim.to_add_on_completion(), b);
    // Construction saved the source's state for scene-side restore.
    assert!(stage.saved_state(a).is_some());
    anim.begin(&mut stage).unwrap();

    let group = anim.state().mobject();
    let children = stage.get(group).unwrap().submobjects().to_vec();
    assert_eq!(children.len(), 2);
    let b_copy = children[1];

    // Ghosting happens after begin's zero interpolation (fading.py:106) —
    // the first interpolated frame is where the ghosts show.
    anim.interpolate(&mut stage, 0.0);
    // At alpha 0 the source shows itself, the target half is a ghost.
    assert_close(&xs_of(&stage, a), &[0.0, 1.0, 2.0, 3.0, 4.0], 1e-6);
    assert_eq!(column(&stage, a, "fill_rgba")[3], 1.0);
    assert_eq!(column(&stage, b_copy, "fill_rgba")[3], 0.0);

    // At alpha 1 the roles swap: the source ghosts onto b's box.
    anim.interpolate(&mut stage, 1.0);
    assert_eq!(column(&stage, a, "fill_rgba")[3], 0.0);
    assert!((column(&stage, b_copy, "fill_rgba")[3] - 1.0).abs() < 1e-6);
    // The source stretched onto b's box: x spans [0, 2].
    let xs = xs_of(&stage, a);
    let (min, max) = xs
        .iter()
        .fold((f32::MAX, f32::MIN), |(lo, hi), &x| (lo.min(x), hi.max(x)));
    assert!((min - 0.0).abs() < 1e-4 && (max - 2.0).abs() < 1e-4);
}

#[test]
fn fade_transform_pieces_aligns_families_first() {
    let mut stage = Stage::new();
    let a_root = stage.add(Mobject::new());
    let a1 = line5(&mut stage);
    stage.attach(a_root, a1).unwrap();
    let b_root = stage.add(Mobject::new());
    let b1 = line5(&mut stage);
    let b2 = line5(&mut stage);
    stage.attach(b_root, b1).unwrap();
    stage.attach(b_root, b2).unwrap();
    let mut anim = fade_transform_pieces(&mut stage, a_root, b_root).unwrap();
    anim.begin(&mut stage).unwrap();
    // align_family padded the source side to two children.
    assert_eq!(stage.get(a_root).unwrap().submobjects().len(), 2);
}
