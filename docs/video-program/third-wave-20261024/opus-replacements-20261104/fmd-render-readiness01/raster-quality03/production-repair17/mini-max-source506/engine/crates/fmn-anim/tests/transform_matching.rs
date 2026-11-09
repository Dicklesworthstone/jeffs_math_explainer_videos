//! TransformMatchingParts semantics (fm-cye): the normalized shape
//! probe, the claim ordering (user pairs → same-shape product →
//! fade-out/fade-in leftovers), and the null-piece guards — against
//! transform_matching_parts.py at the pin.

use fmn_anim::{has_same_shape_as, transform_matching_keys, transform_matching_parts};
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
    mob
}

/// A 1-curve "vee" with distinct shape: (0,0) → (1,2) → (2,0).
fn vee(stage: &mut Stage, dx: f64) -> Mob {
    vmob(
        stage,
        &[[dx, 0.0, 0.0], [dx + 1.0, 2.0, 0.0], [dx + 2.0, 0.0, 0.0]],
    )
}

/// A flat 1-curve bump: (0,0) → (1,0.2) → (2,0).
fn bump(stage: &mut Stage, dx: f64) -> Mob {
    vmob(
        stage,
        &[[dx, 0.0, 0.0], [dx + 1.0, 0.2, 0.0], [dx + 2.0, 0.0, 0.0]],
    )
}

#[test]
fn shape_probe_normalizes_position_and_scale() {
    let mut stage = Stage::new();
    let a = vee(&mut stage, 0.0);
    let b = vee(&mut stage, 5.0);
    assert!(has_same_shape_as(&stage, a, b), "shifted copy matches");
    stage.scale(b, 3.0);
    assert!(has_same_shape_as(&stage, a, b), "scaled copy matches");
    let c = bump(&mut stage, 0.0);
    assert!(!has_same_shape_as(&stage, a, c), "different shapes differ");
}

#[test]
fn matching_parts_pairs_shapes_and_fades_leftovers() {
    let mut stage = Stage::new();
    let source = stage.add(Mobject::new());
    let s_vee = vee(&mut stage, 0.0);
    let s_bump = bump(&mut stage, 3.0);
    stage.attach(source, s_vee).unwrap();
    stage.attach(source, s_bump).unwrap();
    let target = stage.add(Mobject::new());
    let t_vee = vee(&mut stage, 10.0);
    stage.attach(target, t_vee).unwrap();

    let anims = transform_matching_parts(&mut stage, source, target, &[]).unwrap();
    // s_vee → t_vee transforms; s_bump fades out; nothing fades in.
    let names: Vec<String> = anims
        .iter()
        .map(|a| a.state().config.name.clone())
        .collect();
    assert_eq!(names, vec!["Transform", "FadeOutToPoint"]);
    assert_eq!(anims[0].state().mobject(), s_vee);
    assert_eq!(anims[1].state().mobject(), s_bump);
    assert!(anims[1].is_remover());
}

#[test]
fn unmatched_targets_fade_in_from_source_center() {
    let mut stage = Stage::new();
    let source = stage.add(Mobject::new());
    let s_vee = vee(&mut stage, 0.0);
    stage.attach(source, s_vee).unwrap();
    let target = stage.add(Mobject::new());
    let t_bump = bump(&mut stage, 4.0);
    stage.attach(target, t_bump).unwrap();

    let anims = transform_matching_parts(&mut stage, source, target, &[]).unwrap();
    let names: Vec<String> = anims
        .iter()
        .map(|a| a.state().config.name.clone())
        .collect();
    assert_eq!(names, vec!["FadeOutToPoint", "FadeInFromPoint"]);
    assert_eq!(anims[1].state().mobject(), t_bump);
}

#[test]
fn user_pairs_claim_before_the_product() {
    let mut stage = Stage::new();
    let source = stage.add(Mobject::new());
    let s_vee = vee(&mut stage, 0.0);
    stage.attach(source, s_vee).unwrap();
    let target = stage.add(Mobject::new());
    let t_vee = vee(&mut stage, 10.0);
    let t_bump = bump(&mut stage, 20.0);
    stage.attach(target, t_vee).unwrap();
    stage.attach(target, t_bump).unwrap();

    // Force the vee onto the bump; the same-shape t_vee is left to fade.
    let anims = transform_matching_parts(&mut stage, source, target, &[(s_vee, t_bump)]).unwrap();
    let names: Vec<String> = anims
        .iter()
        .map(|a| a.state().config.name.clone())
        .collect();
    assert_eq!(names, vec!["Transform", "FadeInFromPoint"]);
    assert_eq!(anims[0].state().mobject(), s_vee);
    assert_eq!(anims[1].state().mobject(), t_vee);
}

#[test]
fn empty_sides_produce_no_null_animations() {
    let mut stage = Stage::new();
    let source = stage.add(Mobject::new());
    let target = stage.add(Mobject::new());
    let anims = transform_matching_parts(&mut stage, source, target, &[]).unwrap();
    assert!(anims.is_empty());
}

#[test]
fn native_span_keys_match_stably_without_shape_assignment() {
    let mut stage = Stage::new();
    let source = stage.add(Mobject::new());
    let source_x0 = vee(&mut stage, 0.0);
    let source_plus = bump(&mut stage, 3.0);
    let source_x1 = vee(&mut stage, 6.0);
    stage.attach(source, source_x0).unwrap();
    stage.attach(source, source_plus).unwrap();
    stage.attach(source, source_x1).unwrap();

    let target = stage.add(Mobject::new());
    let target_x = bump(&mut stage, 10.0); // deliberately a different shape
    let target_minus = vee(&mut stage, 13.0);
    stage.attach(target, target_x).unwrap();
    stage.attach(target, target_minus).unwrap();

    let source_parts = vec![
        (source_x0, "x".to_owned()),
        (source_plus, "+".to_owned()),
        (source_x1, "x".to_owned()),
    ];
    let target_parts = vec![(target_x, "x".to_owned()), (target_minus, "-".to_owned())];
    let anims =
        transform_matching_keys(&mut stage, source, target, &source_parts, &target_parts).unwrap();
    let names: Vec<&str> = anims
        .iter()
        .map(|animation| animation.state().config.name.as_str())
        .collect();
    assert_eq!(
        names,
        [
            "Transform",
            "FadeOutToPoint",
            "FadeOutToPoint",
            "FadeInFromPoint"
        ]
    );
    assert_eq!(anims[0].state().mobject(), source_x0);
}
