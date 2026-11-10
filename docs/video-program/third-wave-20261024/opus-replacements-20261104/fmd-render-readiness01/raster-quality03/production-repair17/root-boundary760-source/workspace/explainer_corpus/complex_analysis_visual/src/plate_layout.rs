//! The record layout of one blended strip of a colour picture. Plain arithmetic with no
//! renderer, so its tests run with nothing but `rustc --test`.
//!
//! A strip is a polyline whose per-record stroke colours paint its cells. The kit's `RowStrips`
//! repeats an anchor at each cell's edge, so the colour jumps there, and a row reads as a run of
//! flat steps. A blended strip puts one anchor at each cell's centre instead, so the colour runs
//! from centre to centre and the cell keeps its own colour only at its centre.
//!
//! Every record the renderer reads lies on the strip's centre line:
//!
//! ```text
//! lead-in, clear | null | half cell, flat | n - 1 runs, centre to centre | half cell, flat | null | tail, clear
//! ```
//!
//! The null curves (repeated anchors, which the renderer drops) make the alpha jump at the
//! picture's edges, as they do in `RowStrips`. The lead-in and tail are half the stroke's
//! height long, the round caps' radius, so the caps paint nothing.

/// Records in a blended strip of `n` cells. Always odd: a polyline shares its anchors.
pub const fn records(n: usize) -> usize {
    2 * n + 11
}

/// The points of a blended strip of `n` cells, `cell` wide from `left`, along height `y`, for a
/// stroke `thick` high.
pub fn blend_points(left: f64, y: f64, cell: f64, thick: f64, n: usize) -> Vec<[f64; 3]> {
    let at = |x: f64| [x, y, 0.0];
    let q = thick / 4.0;
    let right = left + cell * n as f64;
    let centre = |k: usize| left + cell * (k as f64 + 0.5);
    let mut p = Vec::with_capacity(records(n));
    p.extend([at(left - 2.0 * q), at(left - q), at(left)]);
    p.extend([at(left), at(left)]);
    p.extend([at((left + centre(0)) / 2.0), at(centre(0))]);
    for k in 1..n {
        p.extend([at((centre(k - 1) + centre(k)) / 2.0), at(centre(k))]);
    }
    p.extend([at((centre(n - 1) + right) / 2.0), at(right)]);
    p.extend([at(right), at(right)]);
    p.extend([at(right + q), at(right + 2.0 * q)]);
    p
}

/// The stroke colours for [`blend_points`], `colors` being the cells' sRGB-encoded colours. The
/// record halfway between two centres holds the mean of the two encoded colours: the renderer
/// runs in linear light between records, so this keeps the middle of a run from reading lighter
/// than either neighbour's own blend in the picture's encoded values.
pub fn blend_rgba(colors: &[[f64; 3]]) -> Vec<[f32; 4]> {
    let rgba = |c: [f64; 3], a: f32| [c[0] as f32, c[1] as f32, c[2] as f32, a];
    let n = colors.len();
    let mut out = Vec::with_capacity(records(n));
    out.extend([rgba(colors[0], 0.0); 3]);
    out.extend([rgba(colors[0], 1.0); 4]);
    for k in 1..n {
        let (a, b) = (colors[k - 1], colors[k]);
        let mean = [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0, (a[2] + b[2]) / 2.0];
        out.extend([rgba(mean, 1.0), rgba(b, 1.0)]);
    }
    out.extend([rgba(colors[n - 1], 1.0); 2]);
    out.extend([rgba(colors[n - 1], 0.0); 4]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What the renderer makes of a straight strip's records, after
    /// `fmn-render/src/stroke_profile.rs` `record_stations` and `interval` at a399d626: each
    /// curve of nonzero length gives knots at its three records, and a station takes the last
    /// knot at or before it, so coincident knots make a jump. The lanes here stay as written;
    /// the renderer decodes per knot, which changes the values between knots and never at one.
    struct Knots(Vec<(f64, [f32; 4])>);

    impl Knots {
        fn of(points: &[[f64; 3]], rgba: &[[f32; 4]]) -> Self {
            assert_eq!(points.len(), rgba.len());
            assert_eq!(points.len() % 2, 1, "a polyline shares its anchors");
            let mut out = Vec::new();
            for i in 0..(points.len() - 1) / 2 {
                let (a, b) = (points[2 * i], points[2 * i + 2]);
                let curve = (b[0] - a[0]).abs() + (points[2 * i + 1][0] - a[0]).abs();
                if curve > 0.0 {
                    for r in 2 * i..=2 * i + 2 {
                        out.push((points[r][0], rgba[r]));
                    }
                }
            }
            Self(out)
        }

        fn at(&self, x: f64) -> [f64; 4] {
            let k = self.0.partition_point(|knot| knot.0 <= x);
            if k == 0 {
                return self.0[0].1.map(f64::from);
            }
            let left = self.0[k - 1];
            let Some(right) = self.0.get(k) else {
                return left.1.map(f64::from);
            };
            let t = (x - left.0) / (right.0 - left.0);
            std::array::from_fn(|i| f64::from(left.1[i]) + (f64::from(right.1[i]) - f64::from(left.1[i])) * t)
        }
    }

    const LEFT: f64 = -3.0;
    const CELL: f64 = 0.035;

    /// Colours with no two equal, so a record in the wrong place shows.
    fn colours(n: usize) -> Vec<[f64; 3]> {
        (0..n)
            .map(|k| {
                let t = k as f64 / n.max(2) as f64;
                [0.1 + 0.8 * t, 0.9 - 0.7 * t, 0.2 + 0.5 * (t * 13.0).sin().abs()]
            })
            .collect()
    }

    /// A strip's points and colour records, and the cell colours they came from.
    type Strip = (Vec<[f64; 3]>, Vec<[f32; 4]>, Vec<[f64; 3]>);

    fn strip(n: usize) -> Strip {
        let c = colours(n);
        (blend_points(LEFT, 0.5, CELL, CELL, n), blend_rgba(&c), c)
    }

    fn near(a: [f64; 4], c: [f64; 3], alpha: f64) -> bool {
        (0..3).all(|i| (a[i] - c[i]).abs() < 1e-5) && (a[3] - alpha).abs() < 1e-9
    }

    #[test]
    fn points_and_colours_agree_on_the_record_count() {
        for n in [1, 2, 3, 8, 183, 366, 731] {
            let (p, c, _) = strip(n);
            assert_eq!(p.len(), records(n));
            assert_eq!(c.len(), records(n));
            assert_eq!(records(n) % 2, 1);
            let knots = Knots::of(&p, &c);
            // lead-in, half cell, n - 1 runs, half cell, tail
            assert_eq!(knots.0.len(), 3 * (n + 3), "{n}");
        }
    }

    #[test]
    fn each_cell_centre_shows_that_cells_own_colour() {
        for n in [1, 2, 3, 8, 183] {
            let (p, c, cells) = strip(n);
            let knots = Knots::of(&p, &c);
            for (k, cell) in cells.iter().enumerate() {
                let x = LEFT + CELL * (k as f64 + 0.5);
                assert!(near(knots.at(x), *cell, 1.0), "cell {k} of {n}: {:?}", knots.at(x));
            }
        }
    }

    #[test]
    fn the_strip_is_clear_outside_the_picture_and_opaque_inside() {
        for n in [1, 2, 3, 8, 183] {
            let (p, c, _) = strip(n);
            let knots = Knots::of(&p, &c);
            let right = LEFT + CELL * n as f64;
            let tiny = CELL * 1e-6;
            assert_eq!(knots.at(LEFT - tiny)[3], 0.0, "just before the left edge");
            assert_eq!(knots.at(LEFT)[3], 1.0, "on the left edge");
            assert_eq!(knots.at(LEFT + tiny)[3], 1.0);
            assert_eq!(knots.at(right - tiny)[3], 1.0, "just before the right edge");
            assert_eq!(knots.at(right)[3], 0.0, "on the right edge");
            assert_eq!(knots.at(right + tiny)[3], 0.0);
            assert_eq!(knots.at(LEFT - CELL / 2.0)[3], 0.0);
            assert_eq!(knots.at(right + CELL / 2.0)[3], 0.0);
        }
    }

    #[test]
    fn the_half_cells_at_each_end_are_flat() {
        let n = 8;
        let (p, c, cells) = strip(n);
        let knots = Knots::of(&p, &c);
        for t in [0.0, 0.1, 0.25, 0.4, 0.5] {
            assert!(near(knots.at(LEFT + CELL * t), cells[0], 1.0), "left {t}");
            let x = LEFT + CELL * (n as f64 - 0.5 + t * 0.999);
            assert!(near(knots.at(x), cells[n - 1], 1.0), "right {t}");
        }
    }

    #[test]
    fn halfway_between_centres_is_the_mean_of_the_encoded_colours() {
        let n = 8;
        let (p, c, cells) = strip(n);
        let knots = Knots::of(&p, &c);
        for k in 1..n {
            let (a, b) = (cells[k - 1], cells[k]);
            let mean = [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0, (a[2] + b[2]) / 2.0];
            assert!(near(knots.at(LEFT + CELL * k as f64), mean, 1.0), "edge {k}");
        }
    }

    #[test]
    fn stations_run_left_to_right_with_no_step_back() {
        let (p, c, _) = strip(183);
        let knots = Knots::of(&p, &c);
        assert!(knots.0.windows(2).all(|w| w[1].0 >= w[0].0));
    }

    /// The kit's layout, copied from `explainer_kit::props`, read by the same probe: it jumps at
    /// each cell edge where this one blends, so the probe can tell the two apart. Also pins
    /// the probe, because the kit's jumps are what the film's pictures show.
    #[test]
    fn control_the_probe_sees_the_kits_steps() {
        let n = 8;
        let cells = colours(n);
        let rgba = |c: [f64; 3], a: f32| [c[0] as f32, c[1] as f32, c[2] as f32, a];
        let at = |x: f64| [x, 0.5, 0.0];
        let q = CELL / 4.0;
        let mut points = vec![at(LEFT - 2.0 * q), at(LEFT - q), at(LEFT)];
        let mut lane = vec![rgba(cells[0], 0.0); 3];
        for (k, &c) in cells.iter().enumerate() {
            let (a0, a1) = (LEFT + CELL * k as f64, LEFT + CELL * (k + 1) as f64);
            points.extend([at(a0), at(a0), at((a0 + a1) / 2.0), at(a1)]);
            lane.extend([rgba(c, 1.0); 4]);
        }
        let end = LEFT + CELL * n as f64;
        points.extend([at(end), at(end), at(end + q), at(end + 2.0 * q)]);
        lane.extend([rgba(cells[n - 1], 0.0); 4]);
        let knots = Knots::of(&points, &lane);
        for k in 1..n {
            let x = LEFT + CELL * k as f64;
            assert!(near(knots.at(x), cells[k], 1.0), "the right side of edge {k}");
            assert!(
                near(knots.at(x - CELL * 1e-6), cells[k - 1], 1.0),
                "the left side of edge {k}"
            );
        }
    }

    /// A strip without its leading null curve fades in across the half cell instead of starting
    /// opaque at the picture's edge. `the_strip_is_clear_outside_the_picture_and_opaque_inside`
    /// has to fail on it, or that test proves nothing.
    #[test]
    fn control_a_missing_null_curve_is_seen() {
        let n = 8;
        let (mut p, mut c, _) = strip(n);
        p.drain(3..5);
        c.drain(3..5);
        let knots = Knots::of(&p, &c);
        let inside = knots.at(LEFT + CELL * 0.01)[3];
        assert!(inside < 1.0, "the alpha should ramp here: {inside}");
    }

    /// A colour record moved by one cell must fail the centre test's comparison.
    #[test]
    fn control_a_shifted_colour_is_seen() {
        let n = 8;
        let (p, mut c, cells) = strip(n);
        c.rotate_right(2);
        let knots = Knots::of(&p, &c);
        let x = LEFT + CELL * 3.5;
        assert!(!near(knots.at(x), cells[3], 1.0));
    }
}
