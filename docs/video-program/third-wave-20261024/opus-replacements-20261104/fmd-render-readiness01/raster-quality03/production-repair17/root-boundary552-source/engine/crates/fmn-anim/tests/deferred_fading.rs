//! Native fades must snapshot the live source at begin, not construction.
//! Coordinate and paint witnesses are literal, independent of target recipes.

use fmn_anim::animation::Animation;
use fmn_anim::{
    AnimError, RateFunc, Succession, Transform, fade_in, fade_in_from_point, fade_out,
    fade_out_to_point,
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
    paint(stage, mob, [0.2, 0.4, 0.6, 0.75]);
    mob
}

fn paint(stage: &mut Stage, mob: Mob, rgba: [f32; 4]) {
    for field in ["fill_rgba", "stroke_rgba"] {
        stage
            .get_mut(mob)
            .unwrap()
            .buffer
            .write_range(field, 0, &rgba.repeat(3));
    }
}

fn assert_xs(stage: &Stage, mob: Mob, expected: &[f64]) {
    let points = stage.get_points(mob).unwrap();
    assert_eq!(points.len(), expected.len());
    for (point, &x) in points.iter().zip(expected) {
        assert!((point[0] - x).abs() < 1e-5, "{points:?} != {expected:?}");
        assert!(point[1].abs() < 1e-5 && point[2].abs() < 1e-5);
    }
}

fn assert_paint(stage: &Stage, mob: Mob, rgba: [f32; 4]) {
    for field in ["fill_rgba", "stroke_rgba"] {
        let column = stage.get(mob).unwrap().buffer.read_column(field).unwrap();
        assert_eq!(column.len(), 12);
        for (actual, expected) in column.iter().zip(rgba.repeat(3)) {
            assert!((actual - expected).abs() < 1e-6, "{field}: {column:?}");
        }
    }
}

#[test]
fn fade_in_samples_live_geometry_and_paint_at_begin() {
    let mut stage = Stage::new();
    let mob = line(&mut stage);
    let mut fade = fade_in(&mut stage, mob, [2.0, 0.0, 0.0], 2.0).unwrap();
    fade.state_mut().config.rate_func = RateFunc::linear();
    stage.shift(mob, [4.0, 0.0, 0.0]);
    stage.scale(mob, 2.0);
    paint(&mut stage, mob, [0.8, 0.6, 0.2, 0.5]);
    fade.begin(&mut stage).unwrap();
    assert_xs(&stage, mob, &[2.0, 4.0, 6.0]);
    assert_paint(&stage, mob, [0.8, 0.6, 0.2, 0.0]);
    let target = fade.target_copy().unwrap();
    assert_ne!(target, mob);
    assert_xs(&stage, target, &[2.0, 6.0, 10.0]);
    fade.interpolate(&mut stage, 0.5);
    assert_xs(&stage, mob, &[2.0, 5.0, 8.0]);
    assert_paint(&stage, mob, [0.8, 0.6, 0.2, 0.25]);
    assert_xs(&stage, target, &[2.0, 6.0, 10.0]);
    fade.finish(&mut stage);
    assert_xs(&stage, mob, &[2.0, 6.0, 10.0]);
    assert_paint(&stage, mob, [0.8, 0.6, 0.2, 0.5]);
}

#[test]
fn fade_out_prepares_private_live_target_then_restores_begin_state() {
    let mut stage = Stage::new();
    let mob = line(&mut stage);
    let mut fade = fade_out(&mut stage, mob, [2.0, 0.0, 0.0], 0.5).unwrap();
    fade.state_mut().config.rate_func = RateFunc::linear();
    stage.shift(mob, [4.0, 0.0, 0.0]);
    stage.scale(mob, 2.0);
    paint(&mut stage, mob, [0.8, 0.6, 0.2, 0.5]);
    fade.begin(&mut stage).unwrap();
    assert_xs(&stage, mob, &[2.0, 6.0, 10.0]);
    assert_paint(&stage, mob, [0.8, 0.6, 0.2, 0.5]);
    let target = fade.target_copy().unwrap();
    assert_ne!(target, mob);
    assert_xs(&stage, target, &[6.0, 8.0, 10.0]);
    assert_paint(&stage, target, [0.8, 0.6, 0.2, 0.0]);
    fade.interpolate(&mut stage, 0.5);
    assert_xs(&stage, mob, &[4.0, 7.0, 10.0]);
    assert_paint(&stage, mob, [0.8, 0.6, 0.2, 0.25]);
    fade.finish(&mut stage);
    assert_xs(&stage, mob, &[2.0, 6.0, 10.0]);
    assert_paint(&stage, mob, [0.8, 0.6, 0.2, 0.5]);
    assert!(fade.is_remover());
    assert_eq!(fade.state().config.final_alpha_value, 0.0);
}

#[test]
fn replay_of_both_fades_refreshes_geometry_and_paint() {
    for outgoing in [false, true] {
        let mut stage = Stage::new();
        let mob = line(&mut stage);
        let mut fade = if outgoing {
            fade_out(&mut stage, mob, [0.0; 3], 1.0).unwrap()
        } else {
            fade_in(&mut stage, mob, [0.0; 3], 1.0).unwrap()
        };
        fade.state_mut().config.rate_func = RateFunc::linear();
        fade.begin(&mut stage).unwrap();
        let first_target = fade.target_copy().unwrap();
        fade.finish(&mut stage);
        stage.shift(mob, [8.0, 0.0, 0.0]);
        paint(&mut stage, mob, [0.9, 0.1, 0.3, 0.25]);
        fade.begin(&mut stage).unwrap();
        assert_ne!(fade.target_copy().unwrap(), first_target);
        fade.interpolate(&mut stage, 0.5);
        assert_xs(&stage, mob, &[8.0, 10.0, 12.0]);
        assert_paint(&stage, mob, [0.9, 0.1, 0.3, 0.125]);
        fade.finish(&mut stage);
        assert_xs(&stage, mob, &[8.0, 10.0, 12.0]);
        assert_paint(&stage, mob, [0.9, 0.1, 0.3, 0.25]);
    }
}

#[test]
fn point_fades_keep_constructor_shift_while_sampling_live_source() {
    for outgoing in [false, true] {
        let mut stage = Stage::new();
        let mob = line(&mut stage);
        let mut fade = if outgoing {
            fade_out_to_point(&mut stage, mob, [-3.0, 0.0, 0.0]).unwrap()
        } else {
            fade_in_from_point(&mut stage, mob, [-3.0, 0.0, 0.0]).unwrap()
        };
        fade.state_mut().config.rate_func = RateFunc::linear();
        stage.shift(mob, [10.0, 0.0, 0.0]);
        fade.begin(&mut stage).unwrap();
        // The Reference captures a relative shift at construction. Unlike
        // GrowFromPoint, these fades do NOT retain an absolute anchor.
        if outgoing {
            assert_xs(&stage, fade.target_copy().unwrap(), &[7.0; 3]);
        } else {
            assert_xs(&stage, mob, &[7.0; 3]);
        }
        fade.interpolate(&mut stage, 0.5);
        assert_xs(&stage, mob, &[8.5, 9.5, 10.5]);
        fade.finish(&mut stage);
        assert_xs(&stage, mob, &[10.0, 12.0, 14.0]);
    }
}

#[test]
fn succession_fades_observe_predecessor_geometry_and_paint() {
    for outgoing in [false, true] {
        let mut stage = Stage::new();
        let mob = line(&mut stage);
        let target = stage.copy_family(mob).unwrap();
        stage.shift(target, [4.0, 0.0, 0.0]);
        paint(&mut stage, target, [0.7, 0.3, 0.2, 0.5]);
        let first = Transform::new(mob, target);
        let mut fade = if outgoing {
            fade_out(&mut stage, mob, [2.0, 0.0, 0.0], 1.0).unwrap()
        } else {
            fade_in(&mut stage, mob, [0.0; 3], 1.0).unwrap()
        };
        fade.state_mut().config.rate_func = RateFunc::linear();
        let mut sequence =
            Succession::with_lag_ratio(&mut stage, vec![Box::new(first), Box::new(fade)], 1.0)
                .unwrap();
        sequence.state_mut().config.rate_func = RateFunc::linear();
        sequence.begin(&mut stage).unwrap();
        sequence.interpolate(&mut stage, 0.75);
        let expected = if outgoing {
            [5.0, 7.0, 9.0]
        } else {
            [4.0, 6.0, 8.0]
        };
        assert_xs(&stage, mob, &expected);
        assert_paint(&stage, mob, [0.7, 0.3, 0.2, 0.25]);
        sequence.finish(&mut stage);
        assert_xs(&stage, mob, &[4.0, 6.0, 8.0]);
        assert_paint(&stage, mob, [0.7, 0.3, 0.2, 0.5]);
    }
}

#[test]
fn children_attached_after_construction_participate_in_both_fades() {
    for outgoing in [false, true] {
        let mut stage = Stage::new();
        let mob = line(&mut stage);
        let mut fade = if outgoing {
            fade_out(&mut stage, mob, [0.0; 3], 1.0).unwrap()
        } else {
            fade_in(&mut stage, mob, [0.0; 3], 1.0).unwrap()
        };
        fade.state_mut().config.rate_func = RateFunc::linear();
        let child = line(&mut stage);
        stage.shift(child, [10.0, 0.0, 0.0]);
        paint(&mut stage, child, [0.9, 0.2, 0.4, 0.5]);
        stage.attach(mob, child).unwrap();
        fade.begin(&mut stage).unwrap();
        fade.interpolate(&mut stage, 0.5);
        assert_xs(&stage, child, &[10.0, 12.0, 14.0]);
        assert_paint(&stage, child, [0.9, 0.2, 0.4, 0.25]);
        fade.finish(&mut stage);
        assert_xs(&stage, child, &[10.0, 12.0, 14.0]);
        assert_paint(&stage, child, [0.9, 0.2, 0.4, 0.5]);
        assert_eq!(stage.get(mob).unwrap().submobjects(), &[child]);
    }
}

#[test]
fn outgoing_partial_final_alpha_uses_live_target_without_forcing_restore() {
    let mut stage = Stage::new();
    let mob = line(&mut stage);
    let mut fade = fade_out(&mut stage, mob, [2.0, 0.0, 0.0], 1.0).unwrap();
    fade.state_mut().config.rate_func = RateFunc::linear();
    fade.state_mut().config.final_alpha_value = 0.25;
    fade.state_mut().config.remover = false;
    stage.shift(mob, [8.0, 0.0, 0.0]);
    fade.begin(&mut stage).unwrap();
    fade.finish(&mut stage);
    assert_xs(&stage, mob, &[8.5, 10.5, 12.5]);
    assert_paint(&stage, mob, [0.2, 0.4, 0.6, 0.5625]);
    assert!(!fade.is_remover());
}

#[test]
fn fade_constructors_still_reject_absent_handles() {
    let mut owner = Stage::new();
    let mob = line(&mut owner);
    let mut empty = Stage::new();
    for result in [
        fade_in(&mut empty, mob, [0.0; 3], 1.0),
        fade_out(&mut empty, mob, [0.0; 3], 1.0),
        fade_in_from_point(&mut empty, mob, [0.0; 3]),
        fade_out_to_point(&mut empty, mob, [0.0; 3]),
    ] {
        assert!(matches!(result, Err(AnimError::StaleHandle(found)) if found == mob));
    }
    assert_xs(&owner, mob, &[0.0, 2.0, 4.0]);
    assert_paint(&owner, mob, [0.2, 0.4, 0.6, 0.75]);
}
