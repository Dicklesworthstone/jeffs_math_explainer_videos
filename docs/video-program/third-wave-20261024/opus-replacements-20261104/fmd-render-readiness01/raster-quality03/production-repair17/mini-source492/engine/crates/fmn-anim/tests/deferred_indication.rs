//! The native indication recipes sample targets at begin, not construction.
//! Expected coordinates are literal witnesses independent of target factories.

use fmn_anim::animation::Animation;
use fmn_anim::{RateFunc, Succession, Transform, indicate, turn_inside_out};
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
        .write_range("fill_rgba", 0, &[0.2, 0.4, 0.6, 0.25].repeat(3));
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

fn paint(stage: &Stage, mob: Mob) -> Vec<f32> {
    stage
        .get(mob)
        .unwrap()
        .buffer
        .read_column("fill_rgba")
        .unwrap()
}

#[test]
fn indicate_uses_live_center_size_and_alpha_without_mutating_source_early() {
    let mut stage = Stage::new();
    let mob = line(&mut stage);
    let mut effect = indicate(&mut stage, mob, 1.5, Some([1.0, 1.0, 0.0])).unwrap();
    assert_xs(&stage, mob, &[0.0, 2.0, 4.0]);
    stage.shift(mob, [4.0, 0.0, 0.0]);
    let original_paint = paint(&stage, mob);
    effect.begin(&mut stage).unwrap();
    let target = effect.target_copy().unwrap();
    assert_ne!(target, mob);
    assert_xs(&stage, target, &[3.0, 6.0, 9.0]);
    assert_eq!(paint(&stage, target), [1.0, 1.0, 0.0, 0.25].repeat(3));
    assert_eq!(paint(&stage, mob), original_paint);
    effect.interpolate(&mut stage, 0.5);
    assert_xs(&stage, mob, &[3.0, 6.0, 9.0]);
    effect.finish(&mut stage);
    assert_xs(&stage, mob, &[4.0, 6.0, 8.0]);
    assert_eq!(paint(&stage, mob), original_paint);
}

#[test]
fn indicate_replay_rebuilds_scaled_target_from_the_new_begin_state() {
    let mut stage = Stage::new();
    let mob = line(&mut stage);
    let mut effect = indicate(&mut stage, mob, 1.5, None).unwrap();
    effect.begin(&mut stage).unwrap();
    let first = effect.target_copy().unwrap();
    effect.finish(&mut stage);
    stage.shift(mob, [8.0, 0.0, 0.0]);
    effect.begin(&mut stage).unwrap();
    let second = effect.target_copy().unwrap();
    assert_ne!(first, second);
    assert_xs(&stage, second, &[7.0, 10.0, 13.0]);
    effect.finish(&mut stage);
    assert_xs(&stage, mob, &[8.0, 10.0, 12.0]);
}

#[test]
fn inside_out_reverses_live_geometry_and_reverses_again_on_replay() {
    let mut stage = Stage::new();
    let mob = line(&mut stage);
    let mut effect = turn_inside_out(&mut stage, mob, 0.0).unwrap();
    effect.state_mut().config.rate_func = RateFunc::linear();
    stage.shift(mob, [10.0, 0.0, 0.0]);
    effect.begin(&mut stage).unwrap();
    assert_xs(&stage, effect.target_copy().unwrap(), &[14.0, 12.0, 10.0]);
    effect.finish(&mut stage);
    assert_xs(&stage, mob, &[14.0, 12.0, 10.0]);
    stage.shift(mob, [8.0, 0.0, 0.0]);
    effect.begin(&mut stage).unwrap();
    effect.finish(&mut stage);
    assert_xs(&stage, mob, &[18.0, 20.0, 22.0]);
}

#[test]
fn succession_indicate_and_inside_out_resolve_after_predecessor() {
    for reverse in [false, true] {
        let mut stage = Stage::new();
        let mob = line(&mut stage);
        let destination = stage.copy_family(mob).unwrap();
        stage.shift(destination, [4.0, 0.0, 0.0]);
        let first = Transform::new(mob, destination);
        let mut second = if reverse {
            turn_inside_out(&mut stage, mob, 0.0).unwrap()
        } else {
            indicate(&mut stage, mob, 1.5, None).unwrap()
        };
        second.state_mut().config.rate_func = RateFunc::linear();
        let mut sequence =
            Succession::with_lag_ratio(&mut stage, vec![Box::new(first), Box::new(second)], 1.0)
                .unwrap();
        sequence.state_mut().config.rate_func = RateFunc::linear();
        sequence.begin(&mut stage).unwrap();
        sequence.interpolate(&mut stage, 0.75);
        if reverse {
            assert_xs(&stage, mob, &[6.0, 6.0, 6.0]);
        } else {
            assert_xs(&stage, mob, &[3.5, 6.0, 8.5]);
        }
        sequence.finish(&mut stage);
        if reverse {
            assert_xs(&stage, mob, &[8.0, 6.0, 4.0]);
        } else {
            assert_xs(&stage, mob, &[3.0, 6.0, 9.0]);
        }
    }
}

#[test]
fn partial_indication_endpoint_uses_the_same_prepared_target() {
    let mut stage = Stage::new();
    let mob = line(&mut stage);
    let mut effect = indicate(&mut stage, mob, 2.0, None).unwrap();
    effect.state_mut().config.rate_func = RateFunc::linear();
    effect.state_mut().config.final_alpha_value = 0.5;
    stage.shift(mob, [4.0, 0.0, 0.0]);
    effect.begin(&mut stage).unwrap();
    effect.finish(&mut stage);
    assert_xs(&stage, mob, &[3.0, 6.0, 9.0]);
}
