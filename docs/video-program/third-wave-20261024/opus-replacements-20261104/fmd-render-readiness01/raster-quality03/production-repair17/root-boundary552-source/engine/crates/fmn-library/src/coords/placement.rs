//! Owning coordinate-system placement. Drawn geometry and numeric endpoints
//! always follow the same operation; a detached VMobject clone is not the chart.

use super::{Axes, NumberLine, VMobject, Vec3};

/// The geometry operations that preserve the orthogonal chart used by p2c.
/// General shears are deliberately not admitted through this interface.
#[derive(Clone, Copy)]
pub(crate) enum CoordinateTransform {
    Shift(Vec3),
    Scale(f64, Vec3),
    Rotate(f64, Vec3, Vec3),
}

impl CoordinateTransform {
    pub(crate) fn geometry(self, value: VMobject) -> VMobject {
        match self {
            Self::Shift(offset) => value.shifted(offset),
            Self::Scale(factor, about) => value.scaled_about(factor, about),
            Self::Rotate(angle, axis, about) => value.rotated_about(angle, axis, about),
        }
    }

    pub(crate) fn line(self, value: NumberLine) -> NumberLine {
        match self {
            Self::Shift(offset) => value.shifted(offset),
            Self::Scale(factor, about) => value.scaled_about(factor, about),
            Self::Rotate(angle, axis, about) => value.rotated_about(angle, axis, about),
        }
    }
}

impl NumberLine {
    /// Scale the line's geometry and numeric endpoints about one scene point.
    ///
    /// Like shifted/rotated_about, this also works before build. A zero factor
    /// collapses the chart; p2n retains its documented infinite result there.
    #[must_use]
    pub fn scaled_about(mut self, factor: f64, about: Vec3) -> Self {
        self.vmob = core::mem::take(&mut self.vmob).scaled_about(factor, about);
        let map = |point: Vec3| std::array::from_fn(|i| about[i] + (point[i] - about[i]) * factor);
        self.start = map(self.start);
        self.end = map(self.end);
        // These getters also feed NumberPlane's independent x/y unit sizes.
        self.unit_size *= factor.abs();
        if let Some(width) = &mut self.width {
            *width *= factor.abs();
        }
        self
    }
}

impl Axes {
    pub(crate) fn transformed_coordinates(mut self, transform: CoordinateTransform) -> Self {
        assert!(
            !self.vmob.children().is_empty(),
            "Axes::build must run before coordinate-system placement"
        );
        self.x_axis = transform.line(core::mem::take(&mut self.x_axis));
        self.y_axis = transform.line(core::mem::take(&mut self.y_axis));
        self.vmob = transform.geometry(core::mem::take(&mut self.vmob));
        self
    }
}

// The owning chart types supply transformed_coordinates. Every convenience
// method routes through it, including border layout: none returns a moved
// drawing alongside an unmoved coordinate map.
macro_rules! coordinate_placement {
    ($type:ty) => {
        impl $type {
            /// Shift a built coordinate system, including its numeric map.
            ///
            /// Finish builder configuration and build before placing it. Like
            /// other detached library values, rebuilding constructs a new chart;
            /// later mutations of a separately registered Stage handle do not
            /// mutate this detached value.
            #[must_use]
            pub fn shifted(self, offset: fmn_core::types::Vec3) -> Self {
                self.transformed_coordinates($crate::coords::placement::CoordinateTransform::Shift(
                    offset,
                ))
            }

            /// Scale a built chart and its whole drawn family about a scene point.
            /// Negative factors reflect it; a zero factor collapses its inverse.
            #[must_use]
            pub fn scaled_about(self, factor: f64, about: fmn_core::types::Vec3) -> Self {
                self.transformed_coordinates($crate::coords::placement::CoordinateTransform::Scale(
                    factor, about,
                ))
            }

            /// Rotate a built chart and its whole drawn family about a scene axis.
            #[must_use]
            pub fn rotated_about(
                self,
                angle: f64,
                axis: fmn_core::types::Vec3,
                about: fmn_core::types::Vec3,
            ) -> Self {
                self.transformed_coordinates(
                    $crate::coords::placement::CoordinateTransform::Rotate(angle, axis, about),
                )
            }

            /// Scale about the built family's bounding-box center, as VMobject does.
            #[must_use]
            pub fn scaled(self, factor: f64) -> Self {
                let about = self.vmob().center_point();
                self.scaled_about(factor, about)
            }

            /// Move the built family's center to a scene point, preserving c2p/p2c.
            #[must_use]
            pub fn moved_to(self, point: fmn_core::types::Vec3) -> Self {
                let center = self.vmob().center_point();
                self.shifted(std::array::from_fn(|i| point[i] - center[i]))
            }

            /// Place the chart beside another object, carrying its coordinate map.
            #[must_use]
            pub fn next_to(
                self,
                target: &crate::VMobject,
                direction: fmn_core::types::Vec3,
                buff: f64,
                aligned_edge: fmn_core::types::Vec3,
            ) -> Self {
                let center = self.vmob().center_point();
                let placed = self
                    .vmob()
                    .clone()
                    .next_to(target, direction, buff, aligned_edge);
                let target = placed.center_point();
                self.shifted(std::array::from_fn(|i| target[i] - center[i]))
            }

            /// Align the built family to a frame border using VMobject's layout rule.
            #[must_use]
            pub fn aligned_on_border(self, direction: fmn_core::types::Vec3, buff: f64) -> Self {
                let center = self.vmob().center_point();
                let placed = self.vmob().clone().aligned_on_border(direction, buff);
                let target = placed.center_point();
                self.shifted(std::array::from_fn(|i| target[i] - center[i]))
            }

            /// Move to an edge of the default frame without losing the chart map.
            #[must_use]
            pub fn to_edge(self, edge: fmn_core::types::Vec3, buff: f64) -> Self {
                self.aligned_on_border(edge, buff)
            }

            /// Move to a corner of the default frame without losing the chart map.
            #[must_use]
            pub fn to_corner(self, corner: fmn_core::types::Vec3, buff: f64) -> Self {
                self.aligned_on_border(corner, buff)
            }
        }
    };
}
pub(crate) use coordinate_placement;
coordinate_placement!(Axes);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AxisConfig, CoordinateSystem, FontBook};
    use fmn_core::constants::{LEFT, ORIGIN, OUT, PI, UL};

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

    fn axes() -> Axes {
        Axes::new()
            .x_range([-2.0, 4.0, 1.0])
            .y_range([-1.0, 3.0, 1.0])
            .width(6.0)
            .height(3.0)
            .axis_config(AxisConfig {
                include_tip: Some(true),
                ..Default::default()
            })
            .build(&FontBook::bundled().unwrap())
            .unwrap()
    }

    #[test]
    fn shifted_chart_preserves_both_numeric_axes_and_drawn_family() {
        let before = axes();
        let shift = [-3.0, 1.25, 0.0];
        let after = before.clone().shifted(shift);
        family_near(after.vmob(), &before.vmob().clone().shifted(shift));
        family_near(after.x_axis().vmob(), &after.vmob().children()[0]);
        family_near(after.y_axis().vmob(), &after.vmob().children()[1]);
        assert_eq!(after.all_ranges(), before.all_ranges());
        for p in [[0.0, 0.0], [-2.0, 3.0], [4.0, -1.0], [7.0, 5.0]] {
            near(after.c2p(&p), super::super::add(before.c2p(&p), shift));
            near(after.p2c(after.c2p(&p)), [p[0], p[1], 0.0]);
        }
    }

    #[test]
    fn rotated_scaled_charts_round_trip_including_negative_scales_and_3d_planes() {
        let base = axes();
        for i in 1..=32 {
            let angle = f64::from(i) * PI / 19.0;
            let factor = if i % 2 == 0 { -0.75 } else { 1.5 };
            let pivot = [0.3, -0.7, 0.2];
            let axis = [1.0, 2.0, 3.0];
            let after = base
                .clone()
                .scaled_about(factor, pivot)
                .rotated_about(angle, axis, pivot)
                .shifted([2.0, -3.0, 1.0]);
            let expected = base
                .vmob()
                .clone()
                .scaled_about(factor, pivot)
                .rotated_about(angle, axis, pivot)
                .shifted([2.0, -3.0, 1.0]);
            family_near(after.vmob(), &expected);
            for p in [[0.0, 0.0], [1.0, -0.25], [-3.5, 5.25]] {
                near(after.p2c(after.c2p(&p)), [p[0], p[1], 0.0]);
            }
            assert!(
                (after.x_axis().effective_unit_size()
                    - factor.abs() * base.x_axis().effective_unit_size())
                .abs()
                    < 1e-12
            );
        }
    }

    #[test]
    fn frame_layout_and_moved_to_keep_the_origin_attached() {
        for direction in [LEFT, UL] {
            let before = axes();
            let expected = before.vmob().clone().to_edge(direction, 0.35);
            let placed = before.clone().to_corner(direction, 0.35);
            family_near(placed.vmob(), &expected);
            let delta = super::super::sub(expected.center_point(), before.vmob().center_point());
            near(placed.origin(), super::super::add(before.origin(), delta));
            near(
                placed
                    .clone()
                    .moved_to([2.0, -1.0, 0.0])
                    .vmob()
                    .center_point(),
                [2.0, -1.0, 0.0],
            );
        }
    }

    #[test]
    fn next_to_keeps_the_chart_with_its_drawn_axes() {
        let base = axes();
        let target = crate::Square::new().build().shifted([2.0, 1.0, 0.0]);
        let expected = base
            .vmob()
            .clone()
            .next_to(&target, LEFT, 0.3, super::super::UP);
        let placed = base.clone().next_to(&target, LEFT, 0.3, super::super::UP);
        family_near(placed.vmob(), &expected);
        let offset = super::super::sub(expected.center_point(), base.vmob().center_point());
        near(
            placed.c2p(&[1.0, 2.0]),
            super::super::add(base.c2p(&[1.0, 2.0]), offset),
        );
    }

    #[test]
    fn newly_generated_graphs_riemann_bins_and_areas_use_the_placed_chart() {
        let before = axes();
        let shift = [-2.0, 0.75, 0.0];
        let after = before.clone().shifted(shift);
        let f = |x: f64| 0.5 * x + 0.25;
        let old_graph = before.get_graph(f).build().unwrap();
        let graph = after.get_graph(f).build().unwrap();
        family_near(&graph, &old_graph.clone().shifted(shift));
        let bins = after
            .get_riemann_rectangles(&f, Some([-1.0, 2.0]), Some(0.4), "center")
            .unwrap();
        let old_bins = before
            .get_riemann_rectangles(&f, Some([-1.0, 2.0]), Some(0.4), "center")
            .unwrap();
        family_near(&bins, &old_bins.shifted(shift));
        let area = after.get_area_under_graph(&graph, [-2.0, 4.0], Some([-1.0, 2.0]));
        let old_area = before.get_area_under_graph(&old_graph, [-2.0, 4.0], Some([-1.0, 2.0]));
        family_near(&area, &old_area.shifted(shift));
        near(
            after.get_tangent_line(1.0, &f, 2.0).center_point(),
            after.i2gp(1.0, &f),
        );
    }

    #[test]
    fn adding_number_labels_does_not_reset_placement() {
        let book = FontBook::bundled().unwrap();
        let mut after = axes()
            .scaled_about(0.75, ORIGIN)
            .rotated_about(PI / 5.0, OUT, ORIGIN)
            .shifted([-2.0, 1.0, 0.0]);
        let origin = after.origin();
        let point = after.c2p(&[1.0, 2.0]);
        after
            .add_coordinate_labels(&book, Some(&[1.0]), Some(&[2.0]), None, None)
            .unwrap();
        near(after.origin(), origin);
        near(after.c2p(&[1.0, 2.0]), point);
        family_near(after.x_axis().vmob(), &after.vmob().children()[0]);
        family_near(after.y_axis().vmob(), &after.vmob().children()[1]);
    }

    #[test]
    fn number_line_scale_carries_endpoints_and_units_before_and_after_build() {
        for built in [false, true] {
            let mut line = NumberLine::new([-2.0, 3.0, 1.0]).width(10.0);
            if built {
                line = line.build().unwrap();
            }
            let old = line.clone();
            let after = line.scaled_about(-2.0, [1.0, 0.0, 0.0]);
            assert_eq!(after.effective_unit_size(), 4.0);
            for x in [-3.0, 0.0, 1.25, 4.0] {
                let old_point = old.n2p(x);
                near(after.n2p(x), [1.0 - 2.0 * (old_point[0] - 1.0), 0.0, 0.0]);
                assert!((after.p2n(after.n2p(x)) - x).abs() < 1e-10);
            }
        }
    }
}
