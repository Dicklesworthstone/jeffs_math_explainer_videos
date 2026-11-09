//! Fixed-method transforms use the live begin-time family, including in
//! Succession and replay. No substitute geometry kernel.

use fmn_anim::animation::Animation;
use fmn_anim::{
    AnimError, RateFunc, Succession, Transform, apply_matrix, apply_matrix_2d, fade_to_color,
    scale_in_place, shrink_to_center,
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
    paint(stage, mob, [0.0, 0.0, 1.0, 0.75]);
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

fn assert_points(stage: &Stage, mob: Mob, expected: &[[f64; 3]]) {
    let points = stage.get_points(mob).unwrap();
    assert_eq!(points.len(), expected.len());
    for (point, expected_point) in points.iter().zip(expected) {
        for (value, expected_value) in point.iter().zip(expected_point) {
            assert!(
                (value - expected_value).abs() < 1e-5,
                "{points:?} != {expected:?}"
            );
        }
    }
}

fn assert_xs(stage: &Stage, mob: Mob, xs: [f64; 3]) {
    assert_points(stage, mob, &xs.map(|x| [x, 0.0, 0.0]));
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
fn recoloring_preserves_live_geometry_and_opacity() {
    let mut stage = Stage::new();
    let mob = line(&mut stage);
    let mut animation = fade_to_color(&mut stage, mob, [1.0, 0.0, 0.0]).unwrap();
    animation.state_mut().config.rate_func = RateFunc::linear();
    stage.shift(mob, [4.0, 0.0, 0.0]);
    paint(&mut stage, mob, [0.0, 1.0, 0.0, 0.25]);
    animation.begin(&mut stage).unwrap();
    let target = animation.target_copy().unwrap();
    assert_ne!(target, mob);
    assert_xs(&stage, mob, [4.0, 6.0, 8.0]);
    assert_paint(&stage, mob, [0.0, 1.0, 0.0, 0.25]);
    assert_paint(&stage, target, [1.0, 0.0, 0.0, 0.25]);
    animation.interpolate(&mut stage, 0.5);
    assert_xs(&stage, mob, [4.0, 6.0, 8.0]);
    assert_paint(&stage, mob, [0.5, 0.5, 0.0, 0.25]);
    animation.finish(&mut stage);
    assert_xs(&stage, mob, [4.0, 6.0, 8.0]);
    assert_paint(&stage, mob, [1.0, 0.0, 0.0, 0.25]);
}

#[test]
fn scaling_and_shrinking_use_current_center_extent_and_paint() {
    for shrinking in [false, true] {
        let mut stage = Stage::new();
        let mob = line(&mut stage);
        let mut animation = if shrinking {
            shrink_to_center(&mut stage, mob).unwrap()
        } else {
            scale_in_place(&mut stage, mob, 2.0).unwrap()
        };
        animation.state_mut().config.rate_func = RateFunc::linear();
        stage.shift(mob, [4.0, 0.0, 0.0]);
        stage.scale(mob, 2.0);
        paint(&mut stage, mob, [1.0, 0.0, 0.0, 0.5]);
        animation.begin(&mut stage).unwrap();
        assert_xs(&stage, mob, [2.0, 6.0, 10.0]);
        animation.interpolate(&mut stage, 0.5);
        let midpoint = if shrinking {
            [4.0, 6.0, 8.0]
        } else {
            [0.0, 6.0, 12.0]
        };
        assert_xs(&stage, mob, midpoint);
        animation.finish(&mut stage);
        let endpoint = if shrinking {
            [6.0; 3]
        } else {
            [-2.0, 6.0, 14.0]
        };
        assert_xs(&stage, mob, endpoint);
        assert_paint(&stage, mob, [1.0, 0.0, 0.0, 0.5]);
    }
}

#[test]
fn matrix_maps_live_world_coordinates_not_constructor_points_or_local_frame() {
    let mut stage = Stage::new();
    let mob = line(&mut stage);
    let matrix = [[2.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, -1.0]];
    let mut animation = apply_matrix(&mut stage, mob, matrix).unwrap();
    assert_eq!(animation.state().config.run_time, 3.0);
    animation.state_mut().config.rate_func = RateFunc::linear();
    stage.shift(mob, [3.0, 2.0, 1.0]);
    paint(&mut stage, mob, [1.0, 0.0, 0.0, 0.25]);
    animation.begin(&mut stage).unwrap();
    let target = animation.target_copy().unwrap();
    let destination = [[8.0, 2.0, -1.0], [12.0, 2.0, -1.0], [16.0, 2.0, -1.0]];
    assert_points(&stage, target, &destination);
    animation.interpolate(&mut stage, 0.5);
    assert_points(
        &stage,
        mob,
        &[[5.5, 2.0, 0.0], [8.5, 2.0, 0.0], [11.5, 2.0, 0.0]],
    );
    assert_points(&stage, target, &destination);
    animation.finish(&mut stage);
    assert_points(&stage, mob, &destination);
    assert_paint(&stage, mob, [1.0, 0.0, 0.0, 0.25]);
}

#[test]
fn two_dimensional_matrix_preserves_live_z() {
    let mut stage = Stage::new();
    let mob = line(&mut stage);
    let mut animation = apply_matrix_2d(&mut stage, mob, [[0.0, -1.0], [1.0, 0.0]]).unwrap();
    stage.shift(mob, [3.0, 2.0, 5.0]);
    animation.begin(&mut stage).unwrap();
    animation.finish(&mut stage);
    assert_points(
        &stage,
        mob,
        &[[-2.0, 3.0, 5.0], [-2.0, 5.0, 5.0], [-2.0, 7.0, 5.0]],
    );
}

#[test]
fn replay_rebuilds_color_scale_and_matrix_recipes() {
    for kind in 0..3 {
        let mut stage = Stage::new();
        let mob = line(&mut stage);
        let mut animation = match kind {
            0 => fade_to_color(&mut stage, mob, [1.0, 0.0, 0.0]).unwrap(),
            1 => scale_in_place(&mut stage, mob, 2.0).unwrap(),
            _ => apply_matrix_2d(&mut stage, mob, [[2.0, 0.0], [0.0, 1.0]]).unwrap(),
        };
        animation.begin(&mut stage).unwrap();
        let first_target = animation.target_copy().unwrap();
        animation.finish(&mut stage);
        stage.shift(mob, [10.0, 0.0, 0.0]);
        paint(&mut stage, mob, [0.0, 1.0, 0.0, 0.5]);
        animation.begin(&mut stage).unwrap();
        assert_ne!(animation.target_copy().unwrap(), first_target);
        animation.finish(&mut stage);
        let expected = match kind {
            0 => [10.0, 12.0, 14.0],
            1 => [4.0, 12.0, 20.0],
            _ => [20.0, 28.0, 36.0],
        };
        assert_xs(&stage, mob, expected);
        let expected_color = if kind == 0 {
            [1.0, 0.0, 0.0, 0.5]
        } else {
            [0.0, 1.0, 0.0, 0.5]
        };
        assert_paint(&stage, mob, expected_color);
    }
}

#[test]
fn one_succession_composes_color_scale_matrix_and_shrink_from_predecessors() {
    let mut stage = Stage::new();
    let mob = line(&mut stage);
    let moved = stage.copy_family(mob).unwrap();
    stage.shift(moved, [4.0, 0.0, 0.0]);
    paint(&mut stage, moved, [0.0, 1.0, 0.0, 0.5]);
    let first = Transform::new(mob, moved);
    let color = fade_to_color(&mut stage, mob, [1.0, 0.0, 0.0]).unwrap();
    let scale = scale_in_place(&mut stage, mob, 2.0).unwrap();
    let mut matrix = apply_matrix_2d(&mut stage, mob, [[2.0, 0.0], [0.0, 1.0]]).unwrap();
    matrix.state_mut().config.run_time = 1.0;
    let shrink = shrink_to_center(&mut stage, mob).unwrap();
    let mut sequence = Succession::with_lag_ratio(
        &mut stage,
        vec![
            Box::new(first),
            Box::new(color),
            Box::new(scale),
            Box::new(matrix),
            Box::new(shrink),
        ],
        1.0,
    )
    .unwrap();
    sequence.state_mut().config.rate_func = RateFunc::linear();
    sequence.begin(&mut stage).unwrap();
    sequence.interpolate(&mut stage, 0.8);
    assert_xs(&stage, mob, [4.0, 12.0, 20.0]);
    assert_paint(&stage, mob, [1.0, 0.0, 0.0, 0.5]);
    sequence.finish(&mut stage);
    assert_xs(&stage, mob, [12.0; 3]);
    assert_paint(&stage, mob, [1.0, 0.0, 0.0, 0.5]);
}

#[test]
fn late_family_members_share_current_group_pivot_and_keep_identity() {
    let mut stage = Stage::new();
    let root = stage.add(Mobject::new());
    let first = line(&mut stage);
    stage.attach(root, first).unwrap();
    let mut animation = scale_in_place(&mut stage, root, 2.0).unwrap();
    let second = line(&mut stage);
    stage.shift(second, [10.0, 0.0, 0.0]);
    stage.attach(root, second).unwrap();
    // Group extent is now [0,14], so the shared scale pivot is 7.
    animation.begin(&mut stage).unwrap();
    animation.finish(&mut stage);
    assert_xs(&stage, first, [-7.0, -3.0, 1.0]);
    assert_xs(&stage, second, [13.0, 17.0, 21.0]);
    assert_eq!(stage.get(root).unwrap().submobjects(), &[first, second]);
}

#[test]
fn cloned_recipes_resolve_independently_and_keep_partial_alpha() {
    let mut stage = Stage::new();
    let mob = line(&mut stage);
    let mut animation = scale_in_place(&mut stage, mob, 3.0).unwrap();
    animation.state_mut().config.rate_func = RateFunc::linear();
    animation.state_mut().config.final_alpha_value = 0.25;
    let mut cloned = animation.clone();
    stage.shift(mob, [4.0, 0.0, 0.0]);
    animation.begin(&mut stage).unwrap();
    animation.finish(&mut stage);
    assert_xs(&stage, mob, [3.0, 6.0, 9.0]);
    stage.shift(mob, [10.0, 0.0, 0.0]);
    cloned.begin(&mut stage).unwrap();
    cloned.finish(&mut stage);
    assert_xs(&stage, mob, [11.5, 16.0, 20.5]);
}

#[test]
fn construction_does_not_change_the_source_or_generate_endpoints() {
    let mut stage = Stage::new();
    let mob = line(&mut stage);
    for animation in [
        fade_to_color(&mut stage, mob, [1.0, 0.0, 0.0]).unwrap(),
        scale_in_place(&mut stage, mob, 2.0).unwrap(),
        shrink_to_center(&mut stage, mob).unwrap(),
        apply_matrix_2d(&mut stage, mob, [[2.0, 0.0], [0.0, 1.0]]).unwrap(),
    ] {
        assert_eq!(animation.target_copy(), None);
        assert!(
            animation
                .preflight_mobjects()
                .iter()
                .all(|&handle| handle == mob)
        );
    }
    assert_xs(&stage, mob, [0.0, 2.0, 4.0]);
    assert_paint(&stage, mob, [0.0, 0.0, 1.0, 0.75]);
}

#[test]
fn fixed_method_constructors_keep_early_absent_handle_refusal() {
    let mut owner = Stage::new();
    let mob = line(&mut owner);
    let mut empty = Stage::new();
    for result in [
        fade_to_color(&mut empty, mob, [1.0, 0.0, 0.0]),
        scale_in_place(&mut empty, mob, 2.0),
        shrink_to_center(&mut empty, mob),
        apply_matrix(
            &mut empty,
            mob,
            [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        ),
        apply_matrix_2d(&mut empty, mob, [[1.0, 0.0], [0.0, 1.0]]),
    ] {
        assert!(matches!(result, Err(AnimError::StaleHandle(found)) if found == mob));
    }
    assert_xs(&owner, mob, [0.0, 2.0, 4.0]);
}
