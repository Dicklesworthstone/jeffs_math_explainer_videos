//! Real native first-error termination, independent of the Python host guards.
use std::convert::Infallible;
use std::rc::Rc;

use fmn_geom::{IsolineConfig, IsolineError, plot_isoline_with_stats, try_plot_isoline_with_stats};
use fmn_library::graphs::{GraphError, ImplicitFunctionSpec, ParametricCurveSpec};
use fmn_library::{FunctionGraph, ImplicitFunction, ParametricCurve, SamplingBudget, Style};

fn curve_spec() -> ParametricCurveSpec {
    ParametricCurveSpec {
        t_range: [0.0, 2.0, 0.25],
        epsilon: 0.125,
        discontinuities: vec![1.0],
        use_smoothing: false,
        ..ParametricCurveSpec::default()
    }
}

#[test]
fn discontinuity_samples_have_the_exact_authored_order() {
    let mut calls = Vec::new();
    let curve = curve_spec()
        .try_sample(|t| {
            calls.push(t);
            Ok::<_, Infallible>([t, 2.0 * t, -t])
        })
        .unwrap();
    assert_eq!(
        calls,
        [0.0, 0.25, 0.5, 0.75, 0.875, 1.125, 1.375, 1.625, 1.875, 2.0]
    );
    let path = curve.path().unwrap();
    let anchors = path.anchors();
    for t in calls {
        assert!(
            anchors.contains(&[t, 2.0 * t, -t]),
            "missing exact binary anchor {t}"
        );
    }
}

#[test]
fn curve_error_at_any_sample_stops_and_preserves_the_nonclone_error() {
    #[derive(Debug)]
    struct Stop(Rc<usize>); // Deliberately neither Clone nor Error.
    for stop in 1..=10 {
        let marker = Rc::new(stop);
        let mut calls = 0;
        let result = curve_spec().try_sample(|t| {
            calls += 1;
            if calls == stop {
                Err(Stop(Rc::clone(&marker)))
            } else {
                Ok([t, t, 0.0])
            }
        });
        match result {
            Err(GraphError::Callback(Stop(actual))) => assert!(Rc::ptr_eq(&actual, &marker)),
            _ => panic!("lost the original callback error"),
        }
        assert_eq!(calls, stop);
    }
}

#[test]
fn invalid_curve_points_stop_before_later_samples() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, f64::MAX] {
        for stop in [1, 3, 10] {
            let mut calls = 0;
            let result = curve_spec().try_sample(|t| {
                calls += 1;
                Ok::<_, Infallible>([t, if calls == stop { bad } else { t }, 0.0])
            });
            assert!(
                matches!(result, Err(GraphError::InvalidPoint { index, .. }) if index == stop - 1)
            );
            assert_eq!(calls, stop);
        }
    }
}

#[test]
fn all_curve_segments_are_admitted_before_any_callback() {
    for spec in [
        ParametricCurveSpec {
            sampling_budget: SamplingBudget::new(9),
            ..curve_spec()
        },
        ParametricCurveSpec {
            t_range: [0.0, 1.0, 1e-300],
            ..curve_spec()
        },
        ParametricCurveSpec {
            discontinuities: vec![f64::NAN],
            ..curve_spec()
        },
        ParametricCurveSpec {
            epsilon: f64::INFINITY,
            ..curve_spec()
        },
    ] {
        let result = spec.try_sample(|_| -> Result<_, Infallible> { panic!("admit first") });
        assert!(matches!(result, Err(GraphError::Sampling(_))));
    }
}

#[test]
fn finite_nonpositive_steps_keep_one_endpoint_per_segment() {
    for step in [0.0, -0.5] {
        let mut calls = Vec::new();
        let spec = ParametricCurveSpec {
            t_range: [0.0, 2.0, step],
            ..curve_spec()
        };
        spec.try_sample(|t| {
            calls.push(t);
            Ok::<_, Infallible>([t, 0.0, 0.0])
        })
        .unwrap();
        assert_eq!(calls, [0.875, 2.0]);
    }
}

#[test]
fn explicit_curve_budget_and_owned_builders_share_the_same_records() {
    let spec = ParametricCurveSpec {
        t_range: [-1.0, 1.0, 0.125],
        use_smoothing: true,
        sampling_budget: SamplingBudget::new(17),
        style: Style::default().color(fmn_core::constants::YELLOW),
        ..ParametricCurveSpec::default()
    };
    let sampled = spec
        .try_sample(|t| Ok::<_, Infallible>([t, t * t, 0.0]))
        .unwrap();
    let owned = ParametricCurve::new(|t| [t, t * t, 0.0])
        .t_range(spec.t_range)
        .sampling_budget(spec.sampling_budget)
        .style(spec.style)
        .build()
        .unwrap();
    let scalar = FunctionGraph::new(|t| t * t)
        .x_range(spec.t_range)
        .sampling_budget(spec.sampling_budget)
        .build()
        .unwrap();
    assert_eq!(sampled, owned);
    assert_eq!(sampled, scalar);
}

fn field(x: f64, y: f64) -> f64 {
    x * x + y * y - 0.375
}

#[test]
fn cancellation_at_every_isoline_phase_returns_the_exact_error_and_prefix() {
    let config = IsolineConfig {
        min_depth: 1,
        max_quads: 7,
        ..IsolineConfig::default()
    };
    let mut trace = Vec::new();
    let expected = try_plot_isoline_with_stats(
        |x, y| {
            trace.push((x, y));
            Ok::<_, Infallible>(field(x, y))
        },
        [-1.0; 2],
        [1.0; 2],
        &config,
    )
    .unwrap();
    assert!(
        trace.len() > 30,
        "must reach both refinement and crossing searches"
    );
    assert_eq!(trace.len(), expected.1.evaluations);
    let mut unique = std::collections::HashSet::new();
    assert!(
        trace
            .iter()
            .all(|(x, y)| unique.insert((x.to_bits(), y.to_bits())))
    );
    assert_eq!(
        expected,
        plot_isoline_with_stats(field, [-1.0; 2], [1.0; 2], &config).unwrap()
    );
    for stop in 1..=trace.len() {
        let mut prefix = Vec::new();
        let marker = Rc::new(stop);
        let result = try_plot_isoline_with_stats(
            |x, y| {
                prefix.push((x, y));
                if prefix.len() == stop {
                    Err(Rc::clone(&marker))
                } else {
                    Ok(field(x, y))
                }
            },
            [-1.0; 2],
            [1.0; 2],
            &config,
        );
        match result {
            Err(IsolineError::Callback(error)) => assert!(Rc::ptr_eq(&error, &marker)),
            _ => panic!("a failed sample must abort the entire extraction"),
        }
        assert_eq!(prefix, trace[..stop]);
    }
}

#[test]
fn undefined_regions_and_native_statistics_are_unchanged() {
    for missing in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let config = IsolineConfig {
            min_depth: 2,
            max_quads: 31,
            ..IsolineConfig::default()
        };
        let function = |x: f64, y: f64| if x < 0.0 { missing } else { y - 0.125 };
        let expected = plot_isoline_with_stats(function, [-1.0; 2], [1.0; 2], &config).unwrap();
        let actual = try_plot_isoline_with_stats(
            |x, y| Ok::<_, Infallible>(function(x, y)),
            [-1.0; 2],
            [1.0; 2],
            &config,
        )
        .unwrap();
        assert_eq!(actual, expected);
        assert!(!actual.0.is_empty());
    }
}

#[test]
fn isoline_admission_does_not_call_the_field() {
    for config in [
        IsolineConfig {
            min_depth: 32,
            ..IsolineConfig::default()
        },
        IsolineConfig {
            max_quads: 65_537,
            ..IsolineConfig::default()
        },
        IsolineConfig {
            level: f64::NAN,
            ..IsolineConfig::default()
        },
        IsolineConfig {
            tol: Some([0.0, 0.1]),
            ..IsolineConfig::default()
        },
    ] {
        let result = try_plot_isoline_with_stats(
            |_, _| -> Result<_, Infallible> { panic!("admit first") },
            [-1.0; 2],
            [1.0; 2],
            &config,
        );
        assert!(result.is_err());
    }
}

#[test]
fn implicit_materializer_preserves_errors_and_existing_smooth_geometry() {
    let spec = ImplicitFunctionSpec {
        x_range: [-1.0, 1.0],
        y_range: [-1.0, 1.0],
        use_smoothing: true,
        config: IsolineConfig {
            min_depth: 1,
            max_quads: 7,
            ..IsolineConfig::default()
        },
        ..ImplicitFunctionSpec::default()
    };
    let mut calls = 0;
    let result = spec.try_sample(|_, _| {
        calls += 1;
        Err::<f64, _>("stop")
    });
    assert!(matches!(result, Err(GraphError::Callback("stop"))));
    assert_eq!(calls, 1);
    let actual = spec
        .try_sample(|x, y| Ok::<_, Infallible>(field(x, y)))
        .unwrap();
    let expected = ImplicitFunction::new(field)
        .x_range(spec.x_range)
        .y_range(spec.y_range)
        .min_depth(spec.config.min_depth)
        .max_quads(spec.config.max_quads)
        .use_smoothing(true)
        .build()
        .unwrap();
    assert_eq!(actual, expected);
}
