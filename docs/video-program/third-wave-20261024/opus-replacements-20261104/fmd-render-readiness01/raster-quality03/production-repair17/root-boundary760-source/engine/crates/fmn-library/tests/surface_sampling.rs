//! Failure and admission tests for the one native UV-grid sampler.
use fmn_library::solids::{SurfaceSampleError, SurfaceSpec, compute_triangle_indices};
use fmn_library::{SamplingBudget, SamplingError};
use std::convert::Infallible;
use std::rc::Rc;

fn spec() -> SurfaceSpec {
    SurfaceSpec {
        resolution: (2, 2),
        epsilon: 0.125,
        ..SurfaceSpec::default()
    }
}

#[test]
fn first_error_stops_every_point_and_derivative_probe() {
    for stop in 1..=12 {
        let mut calls = 0;
        let marker = Rc::new(stop);
        let result = spec().try_sample(|u, v| {
            calls += 1;
            if calls == stop {
                Err(Rc::clone(&marker))
            } else {
                Ok([u, v, u * v])
            }
        });
        match result {
            Err(SurfaceSampleError::Callback(error)) => assert!(Rc::ptr_eq(&error, &marker)),
            _ => panic!("expected the original callback error"),
        }
        assert_eq!(calls, stop);
    }
}

#[test]
fn nonfinite_and_unrepresentable_samples_stop_at_the_failed_probe() {
    for stop in [1, 2, 3, 8, 12] {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, f64::MAX] {
            let mut calls = 0;
            let result = spec().try_sample(|u, v| {
                calls += 1;
                Ok::<_, Infallible>(if calls == stop {
                    [value, v, 0.0]
                } else {
                    [u, v, 0.0]
                })
            });
            assert!(
                matches!(result, Err(SurfaceSampleError::InvalidSample { index, .. }) if index == (stop-1)/3)
            );
            assert_eq!(calls, stop);
        }
    }
}

#[test]
fn overflowed_normal_control_is_never_published() {
    let mut s = spec();
    s.normal_nudge = f64::from(f32::MAX);
    let mut calls = 0;
    let result = s.try_sample(|u, v| {
        calls += 1;
        Ok::<_, Infallible>([f64::from(f32::MAX), u, v])
    });
    assert!(matches!(
        result,
        Err(SurfaceSampleError::InvalidSample {
            index: 0,
            probe: "d_normal_point"
        })
    ));
    assert_eq!(calls, 3);
}

#[test]
fn hostile_dimensions_and_products_fail_before_callbacks() {
    for shape in [(usize::MAX, 2), (0, usize::MAX), (513, 512), (262145, 0)] {
        let mut s = spec();
        s.resolution = shape;
        let result = s
            .try_sample(|_, _| -> Result<_, Infallible> { panic!("budget must precede sampling") });
        assert!(matches!(result, Err(SurfaceSampleError::Budget(_))));
    }
    let s = SurfaceSpec {
        resolution: (usize::MAX, 2),
        ..spec()
    };
    assert!(matches!(
        s.try_sample_with_budget(
            |_, _| Ok::<_, Infallible>([0.0; 3]),
            SamplingBudget::new(usize::MAX)
        ),
        Err(SurfaceSampleError::Budget(
            SamplingError::CapacityOverflow { .. }
        ))
    ));
}

#[test]
fn explicit_budget_is_exact_and_allows_deliberate_larger_grids() {
    let s = SurfaceSpec {
        resolution: (3, 4),
        ..spec()
    };
    assert!(
        s.try_sample_with_budget(
            |u, v| Ok::<_, Infallible>([u, v, 0.0]),
            SamplingBudget::new(11)
        )
        .is_err()
    );
    let surface = s
        .try_sample_with_budget(
            |u, v| Ok::<_, Infallible>([u, v, 0.0]),
            SamplingBudget::new(12),
        )
        .unwrap();
    assert_eq!(surface.points().len(), 12);
    let large = SurfaceSpec {
        resolution: (513, 512),
        ..spec()
    };
    assert!(
        large
            .try_sample(|_, _| Ok::<_, Infallible>([0.0; 3]))
            .is_err()
    );
    assert_eq!(
        large
            .try_sample_with_budget(
                |u, v| Ok::<_, Infallible>([u, v, 0.0]),
                SamplingBudget::new(513 * 512)
            )
            .unwrap()
            .points()
            .len(),
        513 * 512
    );
}

#[test]
fn invalid_controls_precede_every_callback() {
    let variants = [
        SurfaceSpec {
            epsilon: 0.0,
            ..spec()
        },
        SurfaceSpec {
            epsilon: f64::NAN,
            ..spec()
        },
        SurfaceSpec {
            normal_nudge: -1.0,
            ..spec()
        },
        SurfaceSpec {
            normal_nudge: f64::INFINITY,
            ..spec()
        },
        SurfaceSpec {
            preferred_creation_axis: 2,
            ..spec()
        },
        SurfaceSpec {
            u_range: (-f64::MAX, f64::MAX),
            ..spec()
        },
        SurfaceSpec {
            v_range: (0.0, f64::NAN),
            ..spec()
        },
        SurfaceSpec {
            opacity: f64::NAN,
            ..spec()
        },
        SurfaceSpec {
            shading: [f64::INFINITY, 0.0, 0.0],
            ..spec()
        },
    ];
    for s in variants {
        let result = s.try_sample(|_, _| -> Result<_, Infallible> {
            panic!("controls must precede sampling")
        });
        assert!(matches!(result, Err(SurfaceSampleError::InvalidControl(_))));
    }
}

#[test]
fn empty_and_strip_topologies_remain_valid() {
    for shape in [(0, 0), (0, 5), (5, 0), (1, 1), (1, 4), (4, 1)] {
        let s = SurfaceSpec {
            resolution: shape,
            ..spec()
        };
        let mut calls = 0;
        let surface = s
            .try_sample(|u, v| {
                calls += 1;
                Ok::<_, Infallible>([u, v, 0.0])
            })
            .unwrap();
        assert_eq!(calls, 3 * shape.0 * shape.1);
        assert_eq!(surface.points().len(), shape.0 * shape.1);
        assert!(surface.triangle_indices().is_empty());
    }
}

#[test]
fn borrowed_stateful_callbacks_keep_exact_uv_probe_order() {
    let mut calls = Vec::new();
    let surface = spec()
        .try_sample(|u, v| {
            calls.push((u, v));
            Ok::<_, Infallible>([u, v, 0.0])
        })
        .unwrap();
    let mut expected = Vec::new();
    for u in [0.0, 1.0] {
        for v in [0.0, 1.0] {
            expected.extend([(u, v), (u + 0.125, v), (u, v + 0.125)]);
        }
    }
    assert_eq!(calls, expected);
    assert_eq!(
        surface.points(),
        [
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0]
        ]
    );
    assert_eq!(
        surface.d_normal_points(),
        [
            [0.0, 0.0, 0.001],
            [0.0, 1.0, 0.001],
            [1.0, 0.0, 0.001],
            [1.0, 1.0, 0.001]
        ]
    );
    assert_eq!(surface.triangle_indices(), compute_triangle_indices((2, 2)));
}

#[test]
fn reversed_and_collapsed_domains_keep_sampling_and_zero_normals() {
    let s = SurfaceSpec {
        u_range: (3.0, -1.0),
        v_range: (2.0, 2.0),
        resolution: (3, 2),
        ..spec()
    };
    let surface = s
        .try_sample(|u, _| Ok::<_, Infallible>([u, 0.0, 0.0]))
        .unwrap();
    assert_eq!(
        surface.points(),
        [
            [3.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0]
        ]
    );
    assert_eq!(surface.points(), surface.d_normal_points());
}

#[test]
fn infallible_entry_point_uses_identical_surface_records() {
    let s = SurfaceSpec {
        resolution: (7, 9),
        ..spec()
    };
    let function = |u, v| [u, v, u * v];
    assert_eq!(
        s.sample(function),
        s.try_sample(|u, v| Ok::<_, Infallible>(function(u, v)))
            .unwrap()
    );
}
