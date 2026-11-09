//! Dense charts cross Atlas sampling, texturing, meshing and Marionette.
use fmn_library::SamplingBudget;
use fmn_library::solids::{
    ParametricSurface, SURFACE_SAMPLING_BUDGET, SurfaceMesh, SurfaceMeshError, SurfaceSampleError,
    SurfaceSpec, TexturedSurface,
};
use fmn_mobject::{Mobject, RenderPrimitive, Stage};
use std::convert::Infallible;

fn plane(u: f64, v: f64) -> Result<[f64; 3], Infallible> {
    Ok([u, v, 0.0])
}

#[test]
fn dense_default_samples_every_vertex_and_keeps_derivative_probes() {
    let spec = SurfaceSpec {
        resolution: (301, 301),
        ..SurfaceSpec::default()
    };
    let mut calls = 0;
    let mut last = (0.0, 0.0);
    let surface = spec
        .try_sample(|u, v| {
            calls += 1;
            last = (u, v);
            plane(u, v)
        })
        .unwrap();
    assert_eq!(surface.points().len(), 90_601);
    assert_eq!(calls, 3 * 90_601);
    assert_eq!(last, (1.0, 1.0 + spec.epsilon));
    assert_eq!(surface.triangle_indices().len(), 540_000);
    assert_eq!(&surface.triangle_indices()[..6], &[0, 301, 1, 1, 301, 302]);
    assert_eq!(surface.points().last(), Some(&[1.0, 1.0, 0.0]));
    for normal in surface.unit_normals() {
        for (actual, expected) in normal.into_iter().zip([0.0, 0.0, 1.0]) {
            assert!((actual - expected).abs() < 1e-12);
        }
    }
}

#[test]
fn surface_budget_does_not_change_other_sampling_or_explicit_caller_limits() {
    assert_eq!(SamplingBudget::DEFAULT.max_samples(), 65_536);
    assert_eq!(SURFACE_SAMPLING_BUDGET.max_samples(), 262_144);
    assert_eq!(
        SURFACE_SAMPLING_BUDGET.max_samples(),
        fmn_mobject::stage::MAX_SURFACE_GRID_POINTS
    );
    let spec = SurfaceSpec {
        resolution: (301, 301),
        ..SurfaceSpec::default()
    };
    let mut calls = 0;
    let result = spec.try_sample_with_budget(
        |u, v| {
            calls += 1;
            plane(u, v)
        },
        SamplingBudget::DEFAULT,
    );
    assert!(matches!(result, Err(SurfaceSampleError::Budget(_))));
    assert_eq!(calls, 0);
}

#[test]
fn old_admitted_surfaces_are_identical_under_both_budgets() {
    for resolution in [(2, 2), (101, 101), (256, 256), (0, 3), (1, 7)] {
        let spec = SurfaceSpec {
            resolution,
            ..SurfaceSpec::default()
        };
        assert_eq!(
            spec.try_sample(plane).unwrap(),
            spec.try_sample_with_budget(plane, SamplingBudget::DEFAULT)
                .unwrap()
        );
    }
}

#[test]
fn oversized_products_and_empty_axis_bypasses_never_execute_the_sampler() {
    for resolution in [(513, 512), (1_000, 1_000), (262_145, 0), (0, usize::MAX)] {
        let spec = SurfaceSpec {
            resolution,
            ..SurfaceSpec::default()
        };
        let mut calls = 0;
        let result = spec.try_sample(|u, v| {
            calls += 1;
            plane(u, v)
        });
        assert!(matches!(result, Err(SurfaceSampleError::Budget(_))));
        assert_eq!(calls, 0);
    }
}

#[test]
fn dense_callback_failure_stops_at_the_original_probe() {
    let spec = SurfaceSpec {
        resolution: (301, 301),
        ..SurfaceSpec::default()
    };
    let mut calls = 0;
    let result = spec.try_sample(|u, v| {
        calls += 1;
        if calls == 7 {
            Err("authored sample failed")
        } else {
            Ok([u, v, 0.0])
        }
    });
    assert!(matches!(
        result,
        Err(SurfaceSampleError::Callback("authored sample failed"))
    ));
    assert_eq!(calls, 7);
}

#[test]
fn dense_wireframe_preserves_uv_stations_normals_and_independent_output_limit() {
    let surface = ParametricSurface::new(|u, v| [u, v, 0.0])
        .resolution(301, 301)
        .build();
    let mesh = SurfaceMesh::from_samples(
        surface.points().to_vec(),
        surface.unit_normals(),
        surface.resolution(),
    )
    .unwrap();
    let wires = mesh.clone().normal_nudge(0.125).try_build().unwrap();
    assert_eq!(wires.children().len(), 32);
    for (index, wire) in wires.children().iter().enumerate() {
        let axis = usize::from(index >= 21);
        let station = if axis == 0 {
            index as f64 / 20.0
        } else {
            (index - 21) as f64 / 10.0
        };
        assert_eq!(wire.points().len(), 601);
        for point in wire.points() {
            assert!((point[axis] - station).abs() < 1e-9);
            assert!((point[2] - 0.125).abs() < 1e-9);
        }
    }
    // Source vertices are admitted, but 220 * 301 wire samples are not.
    assert_eq!(
        mesh.resolution(110, 110).try_build().unwrap_err(),
        SurfaceMeshError::SampleLimit
    );
}

#[test]
fn dense_textured_copy_and_alignment_keep_the_native_grid() {
    let surface = ParametricSurface::new(|u, v| [u, v, u * v])
        .resolution(301, 301)
        .build();
    let texture = TexturedSurface::new(surface.clone(), "host-provided.png");
    assert_eq!(texture.resolution(), (301, 301));
    assert_eq!(texture.im_coords().first(), Some(&[0.0, 1.0]));
    assert_eq!(texture.im_coords().last(), Some(&[1.0, 0.0]));
    let textured: Mobject = texture.into();
    assert_eq!(textured.buffer.len(), 90_601);
    assert_eq!(
        textured.render_primitive,
        RenderPrimitive::SurfaceGrid {
            resolution: (301, 301)
        }
    );
    let mut stage = Stage::new();
    let dense = stage.add(surface);
    let coarse = stage.add(ParametricSurface::new(|u, v| [u, v, u * v]).resolution(2, 2));
    stage.align_surface_points(coarse, dense).unwrap();
    assert!(stage.is_aligned_with(coarse, dense));
    let point = stage.get_points(coarse).unwrap()[150 * 301 + 150];
    assert_eq!(point, [0.5, 0.5, 0.25]);
}
