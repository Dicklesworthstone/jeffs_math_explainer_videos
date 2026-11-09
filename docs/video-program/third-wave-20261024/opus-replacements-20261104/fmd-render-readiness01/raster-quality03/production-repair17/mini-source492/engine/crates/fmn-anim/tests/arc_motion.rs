//! Independent mathematical oracles for 3D arc paths and large translations.
use fmn_anim::{AnimConfig, AnimError, Animation, PathFunc, RateFunc, Transform};
use fmn_mobject::{Mobject, Placement, Stage};

fn close(actual: [f64; 3], expected: [f64; 3]) {
    for axis in 0..3 {
        assert!(
            (actual[axis] - expected[axis]).abs() < 2e-12,
            "{actual:?} != {expected:?}"
        );
    }
}

#[test]
fn quarter_arc_with_axial_motion_is_an_endpoint_exact_helix() {
    let path = PathFunc::from_path_arc(std::f64::consts::FRAC_PI_2, [0.0, 0.0, 1.0]);
    let a = [1.0, 0.0, 2.0];
    let b = [0.0, 1.0, 6.0];
    assert_eq!(path.eval(a, b, 0.0), a);
    assert_eq!(path.eval(a, b, 1.0), b);
    for i in 0..=16 {
        let alpha = f64::from(i) / 16.0;
        let angle = alpha * std::f64::consts::FRAC_PI_2;
        close(
            path.eval(a, b, alpha),
            [
                fmn_dmath::cos(angle),
                fmn_dmath::sin(angle),
                2.0 + 4.0 * alpha,
            ],
        );
    }
}

#[test]
fn purely_axial_displacement_is_linear_for_every_arc_angle() {
    for angle in [-3.0, -1.5, 0.1, 0.7, 2.0, 3.0] {
        let path = PathFunc::from_path_arc(angle, [0.0, 1.0, 0.0]);
        for i in 0..=16 {
            let alpha = f64::from(i) / 16.0;
            close(
                path.eval([3.0, -5.0, 2.0], [3.0, 7.0, 2.0], alpha),
                [3.0, -5.0 + 12.0 * alpha, 2.0],
            );
        }
    }
}

#[test]
fn axis_scale_and_reverse_motion_preserve_the_same_spatial_path() {
    let a = [1.0, -2.0, 3.0];
    let b = [-4.0, 7.0, 1.0];
    let axis = [1.0, 2.0, -3.0];
    let forward = PathFunc::from_path_arc(1.2, axis);
    let reverse = PathFunc::from_path_arc(-1.2, axis);
    for scale in [1e-300, 1.0, 1e300] {
        let scaled = PathFunc::from_path_arc(1.2, axis.map(|value| value * scale));
        for i in 0..=16 {
            let alpha = f64::from(i) / 16.0;
            let expected = forward.eval(a, b, alpha);
            close(scaled.eval(a, b, alpha), expected);
            close(reverse.eval(b, a, 1.0 - alpha), expected);
            let along_axis = expected[0] + 2.0 * expected[1] - 3.0 * expected[2];
            let expected_projection = (1.0 - alpha) * (a[0] + 2.0 * a[1] - 3.0 * a[2])
                + alpha * (b[0] + 2.0 * b[1] - 3.0 * b[2]);
            assert!((along_axis - expected_projection).abs() < 2e-12);
        }
    }
}

#[test]
fn stationary_points_and_zero_axis_keep_their_contracts() {
    let point = [0.1, -10.5, 3.0];
    let zero = PathFunc::from_path_arc(1.3, [0.0; 3]);
    let out = PathFunc::from_path_arc(1.3, [0.0, 0.0, 1.0]);
    for i in 0..=16 {
        let alpha = f64::from(i) / 16.0;
        assert_eq!(zero.eval(point, point, alpha), point);
        assert_eq!(
            zero.eval(point, [2.0; 3], alpha),
            out.eval(point, [2.0; 3], alpha)
        );
    }
}

#[test]
fn large_world_origins_do_not_destroy_local_shape_during_transforms() {
    let huge = 18_014_398_509_481_984.0; // 2^54: adding one is rounded away.
    for path in [
        PathFunc::Straight,
        PathFunc::from_path_arc(1.2, [0.0, 0.0, 1.0]),
    ] {
        let mut stage = Stage::new();
        let source = stage.add(Mobject::from_points(&[[-1.0; 3], [1.0; 3]]));
        let target = stage.copy_family(source).unwrap();
        let a = Placement::from_translation([huge, -huge, huge]);
        let b = Placement::from_translation([huge + 64.0, -huge + 128.0, huge + 32.0]);
        stage.set_placement(source, a).unwrap();
        stage.set_placement(target, b).unwrap();
        let revision = stage.get(source).unwrap().buffer.field_revision("point");
        let points = stage.get_object_points(source).unwrap();
        let mut animation = Transform::new(source, target)
            .with_path_func(path)
            .with_config(AnimConfig {
                rate_func: RateFunc::linear(),
                ..AnimConfig::default()
            });
        animation.begin(&mut stage).unwrap();
        for alpha in [0.0, 0.25, 0.5, 0.75, 1.0] {
            animation.interpolate(&mut stage, alpha);
            let placement = stage.placement(source).unwrap();
            assert_eq!(placement.linear(), Placement::IDENTITY.linear());
            assert_eq!(
                placement.translation(),
                path.eval(a.translation(), b.translation(), alpha)
            );
            assert_eq!(stage.get_object_points(source).unwrap(), points);
            assert_eq!(
                stage.get(source).unwrap().buffer.field_revision("point"),
                revision
            );
        }
        animation.finish(&mut stage);
        assert_eq!(stage.placement(source), Some(b));
    }
}

#[test]
fn invalid_arc_parameters_fail_before_family_alignment_or_copying() {
    for (angle, axis) in [
        (f64::NAN, [0.0; 3]),
        (f64::INFINITY, [0.0; 3]),
        (1.0, [f64::INFINITY, 0.0, 0.0]),
        (1.0, [0.0, f64::NAN, 0.0]),
    ] {
        let mut stage = Stage::new();
        let source = stage.add(Mobject::from_points(&[[0.0; 3]]));
        let target = stage.add(Mobject::from_points(&[[1.0; 3], [2.0; 3]]));
        let before = stage.snapshot().to_bytes().unwrap();
        let mut animation = Transform::new(source, target).with_path_arc(angle, axis);
        assert!(matches!(
            animation.begin(&mut stage),
            Err(AnimError::InvalidPath(_))
        ));
        assert_eq!(stage.snapshot().to_bytes().unwrap(), before);
    }
}
