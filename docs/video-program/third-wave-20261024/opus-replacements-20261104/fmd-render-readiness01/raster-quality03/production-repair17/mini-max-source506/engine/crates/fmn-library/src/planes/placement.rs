//! Keep every cached view of a built plane in the same scene coordinate frame.

use super::{ComplexPlane, NumberPlane, ThreeDAxes};
use crate::coords::placement::{CoordinateTransform, coordinate_placement};

impl NumberPlane {
    pub(crate) fn transformed_coordinates(mut self, transform: CoordinateTransform) -> Self {
        let built = self
            .built
            .as_mut()
            .expect("NumberPlane::build must run before coordinate-system placement");
        built.axes = core::mem::take(&mut built.axes).transformed_coordinates(transform);
        built.background_lines = transform.geometry(core::mem::take(&mut built.background_lines));
        built.faded_lines = transform.geometry(core::mem::take(&mut built.faded_lines));
        // Preserve extra family children, including ComplexPlane's labels.
        built.vmob = transform.geometry(core::mem::take(&mut built.vmob));
        self
    }
}
coordinate_placement!(NumberPlane);

impl ThreeDAxes {
    pub(crate) fn transformed_coordinates(mut self, transform: CoordinateTransform) -> Self {
        let built = self
            .built
            .as_mut()
            .expect("ThreeDAxes::build must run before coordinate-system placement");
        built.axes = core::mem::take(&mut built.axes).transformed_coordinates(transform);
        built.z_axis = transform.line(core::mem::take(&mut built.z_axis));
        built.vmob = transform.geometry(core::mem::take(&mut built.vmob));
        self
    }
}
coordinate_placement!(ThreeDAxes);

impl ComplexPlane {
    pub(crate) fn transformed_coordinates(mut self, transform: CoordinateTransform) -> Self {
        self.plane = core::mem::take(&mut self.plane).transformed_coordinates(transform);
        self.coordinate_labels = self
            .coordinate_labels
            .into_iter()
            .map(|label| transform.geometry(label))
            .collect();
        self
    }
}
coordinate_placement!(ComplexPlane);

impl NumberPlane {
    /// Attach native number labels to the built x/y axes without discarding
    /// the background grid or resetting the placed coordinate map.
    ///
    /// The complete labeled family is prepared privately; a label failure
    /// leaves both the public axis views and the rendered plane unchanged.
    pub fn add_coordinate_labels(
        &mut self,
        book: &crate::FontBook,
        x_values: Option<&[f64]>,
        y_values: Option<&[f64]>,
        excluding: Option<&[f64]>,
        font_size: Option<f64>,
    ) -> Result<(), crate::CoordsError> {
        let mut built = self.built().clone();
        built
            .axes
            .add_coordinate_labels(book, x_values, y_values, excluding, font_size)?;
        let x = built.axes.x_axis().vmob().clone();
        let y = built.axes.y_axis().vmob().clone();
        let mut index = 0;
        built.vmob = core::mem::take(&mut built.vmob).map_children(|child| {
            let current = index;
            index += 1;
            match current {
                2 => x.clone(),
                3 => y.clone(),
                _ => child,
            }
        });
        self.built = Some(built);
        Ok(())
    }
}

impl ThreeDAxes {
    /// Attach x/y/z number labels to a built chart, preserving its position.
    /// `None` uses the corresponding axis's bounded default tick range.
    /// A failure on any axis leaves the entire original family unchanged.
    pub fn add_coordinate_labels(
        &mut self,
        book: &crate::FontBook,
        x_values: Option<&[f64]>,
        y_values: Option<&[f64]>,
        z_values: Option<&[f64]>,
    ) -> Result<(), crate::CoordsError> {
        let mut built = self.built().clone();
        built
            .axes
            .add_coordinate_labels(book, x_values, y_values, None, None)?;
        built.z_axis.add_numbers(book, z_values, None)?;
        let axes = [
            built.axes.x_axis().vmob().clone(),
            built.axes.y_axis().vmob().clone(),
            built.z_axis.vmob().clone(),
        ];
        let mut index = 0;
        built.vmob = core::mem::take(&mut built.vmob).map_children(|child| {
            let current = index;
            index += 1;
            axes.get(current).cloned().unwrap_or(child)
        });
        self.built = Some(built);
        Ok(())
    }

    /// Build a curve from `(x(t), y(t), z(t))` in this chart's coordinates.
    ///
    /// The builder snapshots the current chart. All three axes participate,
    /// including their independent units and any owning placement. Sampling,
    /// discontinuities, smoothing and true arc length remain ParametricCurve's.
    #[must_use]
    pub fn get_parametric_curve(
        &self,
        function: impl Fn(f64) -> fmn_core::types::Vec3 + 'static,
    ) -> crate::ParametricCurve {
        use crate::CoordinateSystem;
        let _ = self.built();
        let chart = self.clone();
        crate::ParametricCurve::new(move |t| chart.c2p(&function(t)))
            .sampling_budget(self.sampling_budget)
    }

    /// Build a height surface `z = function(x, y)` over the axes' x/y ranges.
    ///
    /// The returned ordinary ParametricSurface builder accepts resolution,
    /// range, color, opacity and shading overrides. Its surface budget is
    /// independent of the axes' one-dimensional tick/curve sampling budget.
    #[must_use]
    pub fn get_graph(
        &self,
        function: impl Fn(f64, f64) -> f64 + 'static,
    ) -> crate::ParametricSurface {
        self.get_parametric_surface(move |x, y| [x, y, function(x, y)])
            .u_range(self.x_range[0], self.x_range[1])
            .v_range(self.y_range[0], self.y_range[1])
    }

    /// Build a parameterized surface in this chart's coordinates.
    ///
    /// `u` and `v` default to `(0, 1)` and can be configured on the returned
    /// builder. The native sampler evaluates the chart-mapped function for
    /// both the vertices and derivative probes, so lighting normals are
    /// correct even when x/y/z have different units. No Python adapter or
    /// alternate surface representation is involved.
    #[must_use]
    pub fn get_parametric_surface(
        &self,
        function: impl Fn(f64, f64) -> fmn_core::types::Vec3 + 'static,
    ) -> crate::ParametricSurface {
        use crate::CoordinateSystem;
        let _ = self.built();
        let chart = self.clone();
        crate::ParametricSurface::new(move |u, v| chart.c2p(&function(u, v)))
    }

    /// Sample a fallible surface callback in chart space with an explicit
    /// native specification and budget. Unlike a detached builder, the
    /// callback may borrow local state and is not required to be `'static`.
    ///
    /// Admission happens before callback evaluation; the first callback or
    /// nonfinite mapped-record failure returns without a partial surface.
    pub fn try_parametric_surface<E>(
        &self,
        mut function: impl FnMut(f64, f64) -> Result<fmn_core::types::Vec3, E>,
        spec: &crate::SurfaceSpec,
        budget: crate::SamplingBudget,
    ) -> Result<crate::Surface, crate::solids::SurfaceSampleError<E>> {
        use crate::CoordinateSystem;
        let _ = self.built();
        spec.try_sample_with_budget(|u, v| function(u, v).map(|point| self.c2p(&point)), budget)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CoordinateSystem, FontBook, VMobject};
    use fmn_core::constants::{ORIGIN, OUT, PI, UR};
    use fmn_core::types::Vec3;

    fn near(a: Vec3, b: Vec3) {
        for i in 0..3 {
            assert!((a[i] - b[i]).abs() < 1e-8, "{a:?} != {b:?}");
        }
    }

    fn family_near(a: &VMobject, b: &VMobject) {
        assert_eq!(a.points().len(), b.points().len());
        assert_eq!(a.children().len(), b.children().len());
        assert_eq!(a.style(), b.style());
        for (&a, &b) in a.points().iter().zip(b.points()) {
            near(a, b);
        }
        for (a, b) in a.children().iter().zip(b.children()) {
            family_near(a, b);
        }
    }

    #[test]
    fn placed_number_plane_keeps_grid_axes_graph_and_unit_sizes_coherent() {
        let base = NumberPlane::new()
            .x_range([-2.0, 2.0, 1.0])
            .y_range([-1.0, 1.0, 1.0])
            .width(4.0)
            .height(3.0)
            .faded_line_ratio(1)
            .build(&FontBook::bundled().unwrap())
            .unwrap();
        let shift = [1.0, -1.5, 0.0];
        let after = base
            .clone()
            .scaled_about(0.5, ORIGIN)
            .rotated_about(PI / 6.0, OUT, ORIGIN)
            .shifted(shift);
        let map = |m: VMobject| {
            m.scaled_about(0.5, ORIGIN)
                .rotated_about(PI / 6.0, OUT, ORIGIN)
                .shifted(shift)
        };
        family_near(after.vmob(), &map(base.vmob().clone()));
        family_near(after.faded_lines(), &after.vmob().children()[0]);
        family_near(after.background_lines(), &after.vmob().children()[1]);
        family_near(after.x_axis().vmob(), &after.vmob().children()[2]);
        family_near(after.y_axis().vmob(), &after.vmob().children()[3]);
        assert_eq!(after.x_unit_size(), 0.5 * base.x_unit_size());
        assert_eq!(after.y_unit_size(), 0.5 * base.y_unit_size());
        for p in [[-2.0, 0.5], [0.0, 0.0], [3.0, -2.0]] {
            near(after.p2c(after.c2p(&p)), [p[0], p[1], 0.0]);
        }
        let graph = after.get_graph(|x| 0.25 * x, None).build().unwrap();
        let old = base.get_graph(|x| 0.25 * x, None).build().unwrap();
        family_near(&graph, &map(old));
        let expected = base.vmob().clone().to_edge(UR, 0.4);
        family_near(base.to_edge(UR, 0.4).vmob(), &expected);
    }

    #[test]
    fn three_dimensional_chart_moves_all_three_axes_and_round_trips() {
        let base = ThreeDAxes::new()
            .x_range([-2.0, 2.0, 1.0])
            .y_range([-1.0, 1.0, 1.0])
            .z_range([-3.0, 3.0, 1.0])
            .build(&FontBook::bundled().unwrap())
            .unwrap();
        for i in 1..=16 {
            let angle = f64::from(i) * PI / 13.0;
            let after = base
                .clone()
                .scaled_about(-0.75, [1.0, 0.5, -1.0])
                .rotated_about(angle, [1.0, 2.0, 3.0], ORIGIN)
                .moved_to([2.0, -1.0, 0.5]);
            near(after.vmob().center_point(), [2.0, -1.0, 0.5]);
            for (i, axis) in [after.x_axis(), after.y_axis(), after.z_axis()]
                .into_iter()
                .enumerate()
            {
                family_near(axis.vmob(), &after.vmob().children()[i]);
                near(axis.n2p(0.0), after.origin());
            }
            for p in [[0.0; 3], [1.0, 0.25, -2.0], [-3.0, 2.0, 4.0]] {
                near(after.p2c(after.c2p(&p)), p);
            }
        }
    }

    #[test]
    fn complex_plane_labels_and_vectors_follow_the_owning_chart() {
        let book = FontBook::bundled().unwrap();
        let mut base = ComplexPlane::new()
            .x_range([-2.0, 2.0, 1.0])
            .y_range([-1.0, 1.0, 1.0])
            .faded_line_ratio(1)
            .build(&book)
            .unwrap();
        base.add_coordinate_labels_for(&[[1.0, 0.0], [0.0, 1.0]], &book)
            .unwrap();
        let after = base
            .clone()
            .scaled_about(0.6, ORIGIN)
            .rotated_about(PI / 4.0, OUT, ORIGIN)
            .shifted([-2.0, 1.0, 0.0]);
        let map = |m: VMobject| {
            m.scaled_about(0.6, ORIGIN)
                .rotated_about(PI / 4.0, OUT, ORIGIN)
                .shifted([-2.0, 1.0, 0.0])
        };
        family_near(after.vmob(), &map(base.vmob().clone()));
        for (before, after) in base
            .coordinate_labels()
            .iter()
            .zip(after.coordinate_labels())
        {
            family_near(after, &map(before.clone()));
        }
        for z in [[1.0, 0.0], [0.0, -1.0], [2.5, -3.0]] {
            let back = after.p2n(after.n2p(z));
            near([back[0], back[1], 0.0], [z[0], z[1], 0.0]);
        }
        let expected = crate::Arrow::new(after.origin(), after.n2p([1.0, 0.5]))
            .buff(0.0)
            .build()
            .unwrap();
        family_near(&after.get_vector(&[1.0, 0.5]), &expected);
    }

    #[test]
    fn three_d_graphs_use_all_axes_ranges_and_world_space_normals() {
        let axes = ThreeDAxes::new()
            .x_range([-2.0, 2.0, 1.0])
            .width(8.0)
            .y_range([-1.0, 1.0, 1.0])
            .height(6.0)
            .z_range([-3.0, 3.0, 1.0])
            .depth(24.0)
            .build(&FontBook::bundled().unwrap())
            .unwrap()
            .rotated_about(PI / 4.0, OUT, ORIGIN)
            .shifted([1.0, -2.0, 0.5]);
        let surface = axes.get_graph(|x, y| x + 2.0 * y).resolution(3, 4).build();
        assert_eq!(surface.u_range(), (-2.0, 2.0));
        assert_eq!(surface.v_range(), (-1.0, 1.0));
        assert_eq!(surface.resolution(), (3, 4));
        assert_eq!(surface.triangle_indices().len(), 36);
        for i in 0..3 {
            for j in 0..4 {
                let x = -2.0 + 2.0 * i as f64;
                let y = -1.0 + (2.0 / 3.0) * j as f64;
                near(surface.points()[i * 4 + j], axes.c2p(&[x, y, x + 2.0 * y]));
            }
        }
        // Independent normal of world z = 2*world_x + (8/3)*world_y,
        // then rotated with the whole chart. Sampling must not just transform
        // a unit normal as though all three axes had the same unit size.
        let normal = [-2.0, -8.0 / 3.0, 1.0];
        let norm = normal.iter().map(|v| v * v).sum::<f64>().sqrt();
        let normal = normal.map(|v| v / norm);
        let expected = fmn_geom::space_ops::rotate_vector(normal, PI / 4.0, OUT);
        for normal in surface.unit_normals() {
            near(normal, expected);
        }
    }

    #[test]
    fn parameterized_curves_and_surfaces_snapshot_the_placed_three_d_chart() {
        let axes = ThreeDAxes::new()
            .build(&FontBook::bundled().unwrap())
            .unwrap()
            .scaled_about(0.6, ORIGIN)
            .rotated_about(PI / 3.0, [1.0, 2.0, 1.0], ORIGIN)
            .shifted([1.0, -2.0, 0.5]);
        let curve = axes.get_parametric_curve(|t| [t, 2.0 * t, -t]);
        let surface = axes.get_parametric_surface(|u, v| [u, v, u - v]);
        let moved = axes.clone().shifted([4.0, 0.0, 0.0]);
        let curve = curve.build().unwrap();
        near(curve.points()[0], axes.c2p(&[0.0, 0.0, 0.0]));
        near(*curve.points().last().unwrap(), axes.c2p(&[1.0, 2.0, -1.0]));
        let surface = surface
            .u_range(-1.0, 1.0)
            .v_range(2.0, 3.0)
            .resolution(2, 2)
            .build();
        for (point, coords) in surface.points().iter().zip([
            [-1.0, 2.0, -3.0],
            [-1.0, 3.0, -4.0],
            [1.0, 2.0, -1.0],
            [1.0, 3.0, -2.0],
        ]) {
            near(*point, axes.c2p(&coords));
        }
        assert!((surface.points()[0][0] - moved.c2p(&[-1.0, 2.0, -3.0])[0]).abs() > 3.0);
    }

    #[test]
    fn fallible_three_d_surface_preserves_callback_and_budget_refusals() {
        use crate::solids::SurfaceSampleError;
        let axes = ThreeDAxes::new()
            .build(&FontBook::bundled().unwrap())
            .unwrap();
        let spec = crate::SurfaceSpec {
            resolution: (3, 4),
            ..Default::default()
        };
        let mut calls = 0;
        let denied = axes.try_parametric_surface(
            |u, v| {
                calls += 1;
                Ok::<_, &'static str>([u, v, 0.0])
            },
            &spec,
            crate::SamplingBudget::new(11),
        );
        assert!(matches!(denied, Err(SurfaceSampleError::Budget(_))));
        assert_eq!(calls, 0);
        let failed = axes.try_parametric_surface(
            |u, v| {
                calls += 1;
                if calls == 4 {
                    Err("authored sample failed")
                } else {
                    Ok([u, v, u * v])
                }
            },
            &spec,
            crate::SamplingBudget::new(12),
        );
        assert!(matches!(
            failed,
            Err(SurfaceSampleError::Callback("authored sample failed"))
        ));
        assert_eq!(calls, 4);
        let good = axes
            .try_parametric_surface(
                |u, v| Ok::<_, ()>([u, v, u + v]),
                &spec,
                crate::SamplingBudget::new(12),
            )
            .unwrap();
        assert_eq!(good.points().len(), 12);
        let invalid = axes.try_parametric_surface(
            |_, _| Ok::<_, ()>([f64::NAN, 0.0, 0.0]),
            &spec,
            crate::SamplingBudget::new(12),
        );
        assert!(matches!(
            invalid,
            Err(SurfaceSampleError::InvalidSample { .. })
        ));
    }

    #[test]
    fn label_attachment_preserves_all_plane_views_and_failure_is_atomic() {
        let book = FontBook::bundled().unwrap();
        let mut plane = NumberPlane::new()
            .faded_line_ratio(1)
            .build(&book)
            .unwrap()
            .scaled(0.5)
            .shifted([-2.0, 1.0, 0.0]);
        let origin = plane.origin();
        let grid = plane.background_lines().clone();
        plane
            .add_coordinate_labels(&book, Some(&[1.0]), Some(&[1.0]), None, None)
            .unwrap();
        near(plane.origin(), origin);
        family_near(plane.background_lines(), &grid);
        family_near(plane.x_axis().vmob(), &plane.vmob().children()[2]);
        family_near(plane.y_axis().vmob(), &plane.vmob().children()[3]);
        assert_eq!(plane.x_axis().numbers().len(), 1);
        assert_eq!(plane.y_axis().numbers().len(), 1);
        let mut axes = ThreeDAxes::new()
            .x_range([-2.0, 2.0, 1.0])
            .y_range([-2.0, 2.0, 1.0])
            .z_range([-2.0, 2.0, 1.0])
            .sampling_budget(crate::SamplingBudget::new(32))
            .build(&book)
            .unwrap()
            .shifted([1.0, 2.0, 0.0]);
        let before = axes.vmob().clone();
        let too_many = [1.0; 33];
        assert!(
            axes.add_coordinate_labels(&book, Some(&[1.0]), Some(&[1.0]), Some(&too_many))
                .is_err()
        );
        family_near(axes.vmob(), &before);
        assert!(axes.x_axis().numbers().is_empty());
        assert!(axes.z_axis().numbers().is_empty());
        axes.add_coordinate_labels(&book, Some(&[1.0]), Some(&[1.0]), Some(&[2.0]))
            .unwrap();
        for (i, axis) in [axes.x_axis(), axes.y_axis(), axes.z_axis()]
            .into_iter()
            .enumerate()
        {
            assert_eq!(axis.numbers().len(), 1);
            family_near(axis.vmob(), &axes.vmob().children()[i]);
        }
    }
}
