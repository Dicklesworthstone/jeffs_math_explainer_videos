//! Full-color mean value interpolation over retained boundary stations.
//!
//! The scalar endpoint-ramp path is deliberately unchanged. This extension uses
//! the same signed half-angle weights, applied independently to all four color
//! components; evaluating a nonlinear profile at an averaged *parameter* is not
//! equivalent. The mathematical boundary/linear-precision contract is described
//! in CGAL's Barycentric_coordinates_2 manual and Hormann--Floater (2006).
use super::{GRADIENT_STATIONS, GradientField, ScreenMap};
use crate::{FillProfile, table::Segment};

impl GradientField<'_> {
    /// Conservative station capacity, including every retained paint knot.
    pub(crate) fn profile_station_capacity(contours: usize, knots: usize) -> usize {
        Self::station_capacity(contours)
            .saturating_add(knots)
            .saturating_add(contours.max(1) * 2)
    }

    /// Merge geometry samples with every paint knot without allocating in a
    /// pixel worker. Narrow interior peaks cannot fall between all 64 samples.
    /// Every contour gets its own closing edge; no synthetic hole bridge exists.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn build_profile_into(
        points: &mut impl crate::arena::Sink<[f64; 2]>,
        params: &mut impl crate::arena::Sink<f64>,
        next: &mut impl crate::arena::Sink<usize>,
        edge_params: &mut impl crate::arena::Sink<f64>,
        segments: &[Segment],
        subpaths: &[u32],
        map: ScreenMap,
        profile: &FillProfile,
    ) {
        if segments.is_empty() {
            return;
        }
        let count = subpaths.len().max(1);
        let minimum = count.saturating_mul(3);
        let distributable = GRADIENT_STATIONS.max(minimum).saturating_sub(minimum);
        let field_start = points.len();
        let mut distributed = 0;
        for contour in 0..count {
            let start = subpaths.get(contour).copied().unwrap_or(0) as usize;
            let end = subpaths
                .get(contour + 1)
                .map_or(segments.len(), |&i| i as usize);
            if start >= end || end > segments.len() {
                continue;
            }
            let own = &segments[start..end];
            let s0 = own[0].s0;
            let s1 = own[own.len() - 1].s1;
            let span = s1 - s0;
            if span <= 0.0 {
                continue;
            }
            let target = if contour + 1 == count {
                distributable
            } else {
                ((s1.clamp(0., 1.) * distributable as f64).floor() as usize).min(distributable)
            };
            let uniform_count = 3 + target.saturating_sub(distributed);
            distributed = target;
            let first = points.len() - field_start;
            let knots = profile.knots();
            let mut ki = knots.partition_point(|k| k.s <= s0);
            let mut ui = 0;
            let mut s = s0;
            loop {
                let outgoing = s == s1;
                let index = if outgoing {
                    own.len() - 1
                } else {
                    own.partition_point(|g| g.s1 <= s).min(own.len() - 1)
                };
                let g = &own[index];
                let frac = if g.s1 > g.s0 {
                    ((s - g.s0) / (g.s1 - g.s0)).clamp(0., 1.)
                } else {
                    0.
                };
                let t = fmn_geom::arclength::t_at_arc_fraction(g.p0, g.p1, g.p2, frac);
                let p = fmn_geom::bezier::quadratic_point(g.p0, g.p1, g.p2, t);
                points.put(map.to_pixel(p[0], p[1]));
                params.put(profile.endpoint_parameter(s, outgoing));
                let current = points.len() - field_start - 1;
                next.put(if outgoing { first } else { current + 1 });
                // The profile evaluator uses the actual next vertex's color,
                // including a closure seam; this field keeps metadata aligned.
                edge_params.put(if outgoing { s0 } else { s });
                if outgoing {
                    break;
                }
                while ki < knots.len() && knots[ki].s <= s {
                    ki += 1;
                }
                while ui < uniform_count
                    && s0 + (ui as f64 + 0.5) / uniform_count as f64 * span <= s
                {
                    ui += 1;
                }
                let uniform = if ui < uniform_count {
                    s0 + (ui as f64 + 0.5) / uniform_count as f64 * span
                } else {
                    s1
                };
                let knot = knots.get(ki).map_or(s1, |k| k.s.min(s1));
                let candidate = uniform.min(knot).min(s1);
                // Floating-point station collapse must terminate, even for an
                // extremely short final contour at the end of a long path.
                s = if candidate > s { candidate } else { s1 };
            }
        }
    }

    /// Full linear-light straight-alpha interior color with signed mean value
    /// coordinates. Fixed traversal/reduction order; no per-pixel allocation.
    #[must_use]
    pub fn rgba_at(&self, profile: &FillProfile, p: [f64; 2], translate: [f64; 2]) -> [f32; 4] {
        let n = self.points.len();
        if n == 0 {
            return profile.rgba_at(0.0);
        }
        let color = |i| profile.rgba_at(self.params[i]);
        let delta = |i: usize| {
            [
                self.points[i][0] + translate[0] - p[0],
                self.points[i][1] + translate[1] - p[1],
            ]
        };
        let radius = |d: [f64; 2]| (d[0] * d[0] + d[1] * d[1]).sqrt();
        for i in 0..n {
            let d = delta(i);
            if d[0] * d[0] + d[1] * d[1] <= Self::TOUCH_SQ {
                return color(i);
            }
        }
        if n == 1 {
            return color(0);
        }
        let mut num = [0.0; 4];
        let mut den = 0.0;
        for i in 0..n {
            let j = if self.next.is_empty() {
                (i + 1) % n
            } else {
                self.next[i]
            };
            let di = delta(i);
            let dj = delta(j);
            let ri = radius(di);
            let rj = radius(dj);
            let dot = di[0] * dj[0] + di[1] * dj[1];
            let cross = di[0] * dj[1] - di[1] * dj[0];
            let a = color(i);
            let b = color(j);
            if cross.abs() <= 1e-14 * ri * rj {
                if dot < 0.0 {
                    let t = ri / (ri + rj);
                    return std::array::from_fn(|k| {
                        (f64::from(a[k]) + (f64::from(b[k]) - f64::from(a[k])) * t) as f32
                    });
                }
                continue;
            }
            let tangent = (ri * rj - dot) / cross;
            for (r, c) in [(ri, a), (rj, b)] {
                if r > 0.0 {
                    let w = tangent / r;
                    den += w;
                    for k in 0..4 {
                        num[k] += w * f64::from(c[k]);
                    }
                }
            }
        }
        if den == 0.0 || !den.is_finite() || num.iter().any(|v| !v.is_finite()) {
            return profile.rgba_at(self.nearest_param(p, translate));
        }
        let [low, high] = profile.bounds();
        std::array::from_fn(|k| (num[k] / den).clamp(f64::from(low[k]), f64::from(high[k])) as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FillKnot;
    fn profile(colors: &[[f32; 4]]) -> FillProfile {
        FillProfile::new(
            colors
                .iter()
                .enumerate()
                .map(|(i, &rgba)| FillKnot {
                    s: i as f64 / (colors.len() - 1) as f64,
                    rgba,
                })
                .collect(),
        )
        .unwrap()
    }
    #[test]
    fn color_field_has_linear_precision_and_exact_boundary_values() {
        let points = [[0., 0.], [1., 0.], [1., 1.], [0., 1.]];
        let params = [0., 0.25, 0.5, 0.75];
        let field = GradientField::from_parts(&points, &params);
        let p = profile(&[
            [0., 0., 0., 1.],
            [1., 0., 0., 1.],
            [1., 1., 0., 1.],
            [0., 1., 0., 1.],
            [0., 0., 0., 1.],
        ]);
        for q in [[0.3, 0.4], [0.5, 0.5], [0., 0.], [0.7, 0.], [1., 0.2]] {
            let rgba = field.rgba_at(&p, q, [0.; 2]);
            assert!((f64::from(rgba[0]) - q[0]).abs() < 1e-6);
            assert!((f64::from(rgba[1]) - q[1]).abs() < 1e-6);
            assert_eq!(rgba[3], 1.);
        }
        assert_eq!(
            field.rgba_at(&p, [10.3, -3.6], [10., -4.]),
            field.rgba_at(&p, [0.3, 0.4], [0., 0.])
        );
    }
    #[test]
    fn nonlinear_boundary_color_is_interpolated_not_its_parameter() {
        let field = GradientField::from_parts(
            &[[-1., -1.], [1., -1.], [1., 1.], [-1., 1.]],
            &[0., 0.25, 0.5, 0.75],
        );
        let p = profile(&[[0.; 4], [0.; 4], [1.; 4], [0.; 4], [0.; 4]]);
        assert_eq!(field.rgba_at(&p, [0.; 2], [0.; 2]), [0.25; 4]);
        assert_ne!(
            field.rgba_at(&p, [0.; 2], [0.; 2]),
            p.rgba_at(field.param_at([0.; 2], [0.; 2]))
        );
    }
    #[test]
    fn clockwise_and_counterclockwise_contours_have_the_same_color() {
        let p = profile(&[[0.; 4], [1.; 4], [0.; 4], [0.; 4], [0.; 4]]);
        let a = GradientField::from_parts(
            &[[-1., -1.], [1., -1.], [1., 1.], [-1., 1.]],
            &[0., 0.25, 0.5, 0.75],
        );
        let b = GradientField::from_parts(
            &[[-1., 1.], [1., 1.], [1., -1.], [-1., -1.]],
            &[0.75, 0.5, 0.25, 0.],
        );
        assert_eq!(
            a.rgba_at(&p, [0.; 2], [0.; 2]),
            b.rgba_at(&p, [0.; 2], [0.; 2])
        );
    }
    #[test]
    fn narrow_color_peaks_become_real_boundary_stations() {
        let segments = [Segment {
            p0: [0., 0., 0.],
            p1: [0.5, 0., 0.],
            p2: [1., 0., 0.],
            s0: 0.,
            s1: 1.,
        }];
        let p = FillProfile::new(vec![
            FillKnot {
                s: 0.,
                rgba: [0.; 4],
            },
            FillKnot {
                s: 0.50001,
                rgba: [1.; 4],
            },
            FillKnot {
                s: 0.50002,
                rgba: [0.; 4],
            },
            FillKnot {
                s: 1.,
                rgba: [0.; 4],
            },
        ])
        .unwrap();
        let (mut points, mut params, mut next, mut edges) = (vec![], vec![], vec![], vec![]);
        GradientField::build_profile_into(
            &mut points,
            &mut params,
            &mut next,
            &mut edges,
            &segments,
            &[0],
            ScreenMap {
                origin: [0.; 2],
                y_up: false,
                scale: 1.,
            },
            &p,
        );
        assert!(params.contains(&0.50001));
        assert!(params.contains(&0.50002));
        assert_eq!(points.len(), next.len());
        assert!(points.len() <= GradientField::profile_station_capacity(1, p.knots().len()));
        let field = GradientField::from_contours(&points, &params, &next, &edges);
        assert_eq!(field.rgba_at(&p, [0.50001, 0.], [0.; 2]), [1.; 4]);
    }
    #[test]
    fn disconnected_contour_end_uses_its_own_border_color() {
        // Two open outlines whose implicit fill closures are separate triangles.
        // At the first triangle's last point, the shared normalized station is
        // 0.5; its right-hand color belongs to the second triangle, not this one.
        let segments = [
            Segment {
                p0: [-3., -1., 0.],
                p1: [-2., -1., 0.],
                p2: [-1., -1., 0.],
                s0: 0.,
                s1: 0.25,
            },
            Segment {
                p0: [-1., -1., 0.],
                p1: [-1., 0., 0.],
                p2: [-1., 1., 0.],
                s0: 0.25,
                s1: 0.5,
            },
            Segment {
                p0: [1., -1., 0.],
                p1: [2., -1., 0.],
                p2: [3., -1., 0.],
                s0: 0.5,
                s1: 0.75,
            },
            Segment {
                p0: [3., -1., 0.],
                p1: [3., 0., 0.],
                p2: [3., 1., 0.],
                s0: 0.75,
                s1: 1.,
            },
        ];
        let red = [1., 0., 0., 1.];
        let blue = [0., 0., 1., 1.];
        let p = FillProfile::new(vec![
            FillKnot { s: 0., rgba: red },
            FillKnot { s: 0.5, rgba: red },
            FillKnot { s: 0.5, rgba: blue },
            FillKnot { s: 1., rgba: blue },
        ])
        .unwrap();
        let (mut points, mut params, mut next, mut edges) = (vec![], vec![], vec![], vec![]);
        let map = ScreenMap {
            origin: [0.; 2],
            y_up: false,
            scale: 1.,
        };
        GradientField::build_profile_into(
            &mut points,
            &mut params,
            &mut next,
            &mut edges,
            &segments,
            &[0, 2],
            map,
            &p,
        );
        let field = GradientField::from_contours(&points, &params, &next, &edges);
        let style = crate::Style {
            fill_border_width: 400.,
            ..Default::default()
        }
        .with_fill_profile(p);
        assert_eq!(
            super::super::fill_rgba_with_border(&style, &field, &segments, map, [0.; 2], [-1., 1.]),
            red
        );
        assert_eq!(
            super::super::fill_rgba_with_border(&style, &field, &segments, map, [0.; 2], [1., -1.]),
            blue
        );
        // Explicit station links must close each contour in place.
        for (i, &j) in next.iter().enumerate() {
            assert_eq!(points[i][0] < 0., points[j][0] < 0.);
        }
    }

    #[test]
    fn a_counterwound_hole_interpolates_its_boundary_without_a_bridge() {
        let points = [
            [-2., -2.],
            [2., -2.],
            [2., 2.],
            [-2., 2.],
            [-1., -1.],
            [-1., 1.],
            [1., 1.],
            [1., -1.],
        ];
        let params = [0., 0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875];
        let next = [1, 2, 3, 0, 5, 6, 7, 4];
        let edges = [0.125, 0.25, 0.375, 0., 0.625, 0.75, 0.875, 0.5];
        let field = GradientField::from_contours(&points, &params, &next, &edges);
        // Use affine boundary data: signed coordinates must preserve it in the
        // annulus, not collapse every interior point to a single scalar color.
        let mut knots: Vec<_> = points
            .iter()
            .zip(params)
            .map(|(q, s)| FillKnot {
                s,
                rgba: [(q[0] + 2.) as f32 / 4., (q[1] + 2.) as f32 / 4., 0., 1.],
            })
            .collect();
        knots.push(FillKnot {
            s: 1.,
            rgba: knots[4].rgba,
        });
        let profile = FillProfile::new(knots).unwrap();
        for q in [
            [1.5, 0.],
            [-1.5, 0.],
            [0., 1.5],
            [0., -1.5],
            [-1., 0.],
            [0., 2.],
        ] {
            let c = field.rgba_at(&profile, q, [0.; 2]);
            assert!(
                (c[0] - (q[0] + 2.) as f32 / 4.).abs() < 1e-6,
                "{q:?}: {c:?}"
            );
            assert!(
                (c[1] - (q[1] + 2.) as f32 / 4.).abs() < 1e-6,
                "{q:?}: {c:?}"
            );
            assert_eq!(c[3], 1.);
        }
    }
}
