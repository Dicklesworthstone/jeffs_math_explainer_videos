//! Grow targets freeze at the actual begin boundary, including replay and
//! a just-in-time Succession child. Anchors remain construction-time values.

use fmn_anim::animation::Animation;
use fmn_anim::transform::StartPrep;
use fmn_anim::{
    AnimError, RateFunc, Succession, Transform, grow_arrow, grow_from_center, grow_from_edge,
    grow_from_point,
};
use fmn_mobject::record::{RecordBuffer, RecordSchema};
use fmn_mobject::{Mob, Mobject, Stage};

fn line(stage: &mut Stage) -> Mob {
    let mob = stage.add(Mobject::new());
    let entry = stage.get_mut(mob).unwrap();
    entry.buffer = RecordBuffer::new(RecordSchema::vmobject(), 3).unwrap();
    entry
        .buffer
        .write_range("point", 0, &[0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 4.0, 0.0, 0.0]);
    entry
        .buffer
        .write_range("fill_rgba", 0, &[0.2, 0.4, 0.6, 0.75].repeat(3));
    mob
}

fn assert_xs(stage: &Stage, mob: Mob, expected: &[f64]) {
    let points = stage.get_points(mob).unwrap();
    assert_eq!(points.len(), expected.len());
    for (point, &x) in points.iter().zip(expected) {
        assert!((point[0] - x).abs() < 1e-5, "{points:?} != {expected:?}");
        assert!(point[1].abs() < 1e-5 && point[2].abs() < 1e-5);
    }
}

#[test]
fn grow_samples_live_geometry_and_paint_only_at_begin() {
    let mut stage = Stage::new();
    let mob = line(&mut stage);
    let mut grow = grow_from_point(&mut stage, mob, [-2.0, 0.0, 0.0], None).unwrap();
    grow.state_mut().config.rate_func = RateFunc::linear();
    // These changes happen after constructing the animation. A constructor
    // snapshot loses both of them, even without a composition or a renderer.
    stage.shift(mob, [4.0, 0.0, 0.0]);
    stage.scale(mob, 2.0);
    let paint = [0.9, 0.1, 0.3, 0.25].repeat(3);
    stage
        .get_mut(mob)
        .unwrap()
        .buffer
        .write_range("fill_rgba", 0, &paint);
    grow.begin(&mut stage).unwrap();
    assert_xs(&stage, mob, &[-2.0, -2.0, -2.0]);
    let target = grow.target_copy().unwrap();
    assert_ne!(target, mob);
    assert_xs(&stage, target, &[2.0, 6.0, 10.0]);
    grow.interpolate(&mut stage, 0.5);
    assert_xs(&stage, mob, &[0.0, 2.0, 4.0]);
    grow.finish(&mut stage);
    assert_xs(&stage, mob, &[2.0, 6.0, 10.0]);
    assert_eq!(
        stage
            .get(mob)
            .unwrap()
            .buffer
            .read_column("fill_rgba")
            .unwrap(),
        paint,
    );
}

#[test]
fn center_edge_and_arrow_keep_their_construction_anchors() {
    for (kind, anchor) in [(0, 2.0), (1, 4.0), (2, 0.0)] {
        let mut stage = Stage::new();
        let mob = line(&mut stage);
        let mut grow = match kind {
            0 => grow_from_center(&mut stage, mob, None).unwrap(),
            1 => grow_from_edge(&mut stage, mob, [1.0, 0.0, 0.0], None).unwrap(),
            _ => grow_arrow(&mut stage, mob).unwrap(),
        };
        grow.state_mut().config.rate_func = RateFunc::linear();
        stage.shift(mob, [10.0, 0.0, 0.0]);
        grow.begin(&mut stage).unwrap();
        assert_xs(&stage, mob, &[anchor; 3]);
        grow.finish(&mut stage);
        assert_xs(&stage, mob, &[10.0, 12.0, 14.0]);
    }
}

#[test]
fn replay_refreshes_the_target_instead_of_reusing_previous_products() {
    let mut stage = Stage::new();
    let mob = line(&mut stage);
    let mut grow = grow_from_center(&mut stage, mob, None).unwrap();
    grow.state_mut().config.rate_func = RateFunc::linear();
    grow.begin(&mut stage).unwrap();
    let first_target = grow.target_copy().unwrap();
    grow.finish(&mut stage);
    stage.shift(mob, [8.0, 0.0, 0.0]);
    grow.begin(&mut stage).unwrap();
    assert_ne!(grow.target_copy().unwrap(), first_target);
    assert_xs(&stage, mob, &[2.0; 3]);
    grow.finish(&mut stage);
    assert_xs(&stage, mob, &[8.0, 10.0, 12.0]);
}

#[test]
fn succession_grow_observes_the_completed_predecessor() {
    let mut stage = Stage::new();
    let mob = line(&mut stage);
    let target = stage.copy_family(mob).unwrap();
    stage.shift(target, [4.0, 0.0, 0.0]);
    let first = Transform::new(mob, target);
    let mut grow = grow_from_center(&mut stage, mob, None).unwrap();
    grow.state_mut().config.rate_func = RateFunc::linear();
    let mut sequence =
        Succession::with_lag_ratio(&mut stage, vec![Box::new(first), Box::new(grow)], 1.0).unwrap();
    sequence.state_mut().config.rate_func = RateFunc::linear();
    sequence.begin(&mut stage).unwrap();
    sequence.interpolate(&mut stage, 0.75);
    assert_xs(&stage, mob, &[3.0, 4.0, 5.0]);
    sequence.finish(&mut stage);
    assert_xs(&stage, mob, &[4.0, 6.0, 8.0]);
}

#[test]
fn self_target_is_frozen_before_start_prep_and_stays_independent() {
    let mut stage = Stage::new();
    let mob = line(&mut stage);
    let mut transform = Transform::new(mob, mob).with_start_prep(StartPrep {
        scale: Some(0.0),
        move_to: Some([10.0, 0.0, 0.0]),
        ..StartPrep::default()
    });
    transform.state_mut().config.rate_func = RateFunc::linear();
    transform.begin(&mut stage).unwrap();
    let target = transform.target_copy().unwrap();
    assert_ne!(target, mob);
    assert_xs(&stage, mob, &[10.0; 3]);
    assert_xs(&stage, target, &[0.0, 2.0, 4.0]);
    transform.interpolate(&mut stage, 0.5);
    assert_xs(&stage, mob, &[5.0, 6.0, 7.0]);
    assert_xs(&stage, target, &[0.0, 2.0, 4.0]);
    transform.finish(&mut stage);
    assert_xs(&stage, mob, &[0.0, 2.0, 4.0]);
}

#[test]
fn grow_still_refuses_an_absent_handle_at_construction() {
    let mut owner = Stage::new();
    let mob = line(&mut owner);
    let mut empty = Stage::new();
    assert!(matches!(
        grow_from_point(&mut empty, mob, [0.0, 0.0, 0.0], None),
        Err(AnimError::StaleHandle(found)) if found == mob
    ));
    assert_xs(&owner, mob, &[0.0, 2.0, 4.0]);
}
