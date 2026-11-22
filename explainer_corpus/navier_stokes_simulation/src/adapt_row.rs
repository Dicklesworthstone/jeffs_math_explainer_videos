//! Chapter 14's numbers, exact: the ideal flow past a cylinder and the indicator row on its centre
//! line (navier-full-script03.md, cues 14_4 to 14_10). Nothing here is simulated or measured. Every
//! value is a closed form evaluated in exact rationals (`four_cell::Q`); only the streamline and
//! speed-curve points the scene draws use f64.
//!
//! Units: lengths in cylinder radii from the cylinder's centre, speeds in stream speeds (radius 1,
//! stream speed 1). The ideal (potential) flow is u = 1 − (x² − y²)/r⁴, v = −2xy/r⁴, with stream
//! function ψ = y(1 − 1/r²). The row is the centre line in front of the cylinder: the points
//! x = −1, −3, …, −13 on y = 0 and the six coarse cells between them, each two radii wide. A cell
//! knows the speed at its two ends and guesses its middle by a straight line; its miss is the gap
//! between that guess and the exact middle speed. The tolerance, 1/100, is the script's own choice,
//! not a Basilisk setting. Standard library only, so the tests run without the renderer.

use crate::four_cell::Q;

fn q(n: i64) -> Q {
    Q::int(n)
}

fn half(a: Q) -> Q {
    a / q(2)
}

/// The velocity (u, v) at a point outside the cylinder.
pub fn velocity(x: Q, y: Q) -> (Q, Q) {
    let r2 = x * x + y * y;
    let r4 = r2 * r2;
    (Q::ONE - (x * x - y * y) / r4, q(-2) * x * y / r4)
}

/// [u_x, u_y, v_x, v_y] by the quotient rule, term by term and unsimplified (the form of
/// BronzeJaguar's `ad01`), so zero spin and divergence are identities checked at points, not built in.
pub fn gradient(x: Q, y: Q) -> [Q; 4] {
    let r2 = x * x + y * y;
    let (r4, r6) = (r2 * r2, r2 * r2 * r2);
    let n = x * x - y * y;
    let ux = -(q(2) * x / r4 - q(4) * x * n / r6);
    let uy = q(2) * y / r4 + q(4) * y * n / r6;
    let vx = q(-2) * y / r4 + q(8) * x * x * y / r6;
    let vy = q(-2) * x / r4 + q(8) * x * y * y / r6;
    [ux, uy, vx, vy]
}

/// Spin, v_x − u_y.
pub fn spin(x: Q, y: Q) -> Q {
    let g = gradient(x, y);
    g[2] - g[1]
}

/// Divergence, u_x + v_y.
pub fn divergence(x: Q, y: Q) -> Q {
    let g = gradient(x, y);
    g[0] + g[3]
}

/// The flow through the wall at a point of the circle r = 1: the radial part, u x + v y.
pub fn wall_flow(x: Q, y: Q) -> Q {
    let (u, v) = velocity(x, y);
    u * x + v * y
}

/// The speed along the centre line (y = 0): u = 1 − 1/x².
pub fn speed(x: Q) -> Q {
    velocity(x, Q::ZERO).0
}

/// The strain on the centre line, u_x = 2/x³: negative in front, where the air is squeezed along
/// the line (and, with no divergence, stretched across it at the same rate).
pub fn strain(x: Q) -> Q {
    gradient(x, Q::ZERO)[0]
}

/// How fast the slope changes along the centre line, u_xx = −6/x⁴.
pub fn curvature(x: Q) -> Q {
    q(-6) / (x * x * x * x)
}

/// A cell of the row: its end farther from the cylinder and its end nearer it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Cell {
    pub outer: Q,
    pub inner: Q,
}

impl Cell {
    pub fn width(self) -> Q {
        self.inner - self.outer
    }

    pub fn middle(self) -> Q {
        half(self.outer + self.inner)
    }

    /// The straight line between the two end speeds, read at the middle.
    pub fn guess(self) -> Q {
        half(speed(self.outer) + speed(self.inner))
    }

    /// The exact speed at the middle.
    pub fn exact(self) -> Q {
        speed(self.middle())
    }

    /// The gap between the straight-line guess and the exact middle speed.
    pub fn miss(self) -> Q {
        (self.exact() - self.guess()).abs()
    }

    /// The two halves, the one nearer the cylinder first.
    pub fn halves(self) -> [Cell; 2] {
        let m = self.middle();
        [
            Cell {
                outer: m,
                inner: self.inner,
            },
            Cell {
                outer: self.outer,
                inner: m,
            },
        ]
    }

    /// The bound 14_10's Precisely states, miss = (h²/8)|u_xx| at some point of the cell: |u_xx| is
    /// monotone across a cell of the row, so the miss lies between h²/8 times its values at the ends.
    pub fn miss_bounds(self) -> (Q, Q) {
        let h2 = self.width() * self.width() / q(8);
        let (a, b) = (curvature(self.outer).abs() * h2, curvature(self.inner).abs() * h2);
        (a.min(b), a.max(b))
    }
}

/// The row's seven points, x = −1, −3, …, −13: the front of the cylinder, then the cell ends outward.
pub fn row_points() -> [Q; 7] {
    std::array::from_fn(|k| q(-1 - 2 * k as i64))
}

/// The six coarse cells, each two radii wide, nearest the cylinder first.
pub fn coarse_row() -> [Cell; 6] {
    let p = row_points();
    std::array::from_fn(|k| Cell {
        outer: p[k + 1],
        inner: p[k],
    })
}

/// The script's tolerance: split a cell whose miss is more than one hundredth.
pub fn tolerance() -> Q {
    Q::new(1, 100)
}

/// One pass of the test over a row: the cells tested, which of them split, and the row after.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Round {
    pub cells: Vec<Cell>,
    pub split: Vec<bool>,
    pub after: Vec<Cell>,
}

impl Round {
    /// The cells this round made, nearest the cylinder first.
    pub fn made(&self) -> Vec<Cell> {
        self.cells
            .iter()
            .zip(&self.split)
            .filter(|&(_, &split)| split)
            .flat_map(|(cell, _)| cell.halves())
            .collect()
    }

    /// The cells this round split.
    pub fn splits(&self) -> Vec<Cell> {
        self.cells
            .iter()
            .zip(&self.split)
            .filter(|&(_, &split)| split)
            .map(|(&cell, _)| cell)
            .collect()
    }
}

/// Test every cell of a row against `tol`: split those that miss by more.
pub fn test_row(cells: &[Cell], tol: Q) -> Round {
    let split: Vec<bool> = cells.iter().map(|c| c.miss() > tol).collect();
    let mut after = Vec::with_capacity(2 * cells.len());
    for (&cell, &s) in cells.iter().zip(&split) {
        if s {
            after.extend(cell.halves());
        } else {
            after.push(cell);
        }
    }
    Round {
        cells: cells.to_vec(),
        split,
        after,
    }
}

/// The two rounds the chapter shows: the coarse row (14_8), then the row that round leaves (14_9).
pub fn rounds() -> [Round; 2] {
    let first = test_row(&coarse_row(), tolerance());
    let second = test_row(&first.after, tolerance());
    [first, second]
}

/// Halve every cell: what uniform refinement does to the row.
pub fn halve_all(cells: &[Cell]) -> Vec<Cell> {
    cells.iter().flat_map(|c| c.halves()).collect()
}

/// A rule based on spin alone: split a cell where the spin at its ends or its middle is not zero.
pub fn spin_rule(cells: &[Cell]) -> Vec<Cell> {
    let mut after = Vec::with_capacity(cells.len());
    for &cell in cells {
        let spins = [cell.outer, cell.middle(), cell.inner].map(|x| spin(x, Q::ZERO));
        if spins.iter().any(|s| !s.is_zero()) {
            after.extend(cell.halves());
        } else {
            after.push(cell);
        }
    }
    after
}

/// The near cell against the third, as 14_10 compares them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ratios {
    /// Strain at the two middles.
    pub strain: Q,
    /// u_xx at the two middles.
    pub curvature: Q,
    /// The two misses.
    pub miss: Q,
}

pub fn near_against_third() -> Ratios {
    let row = coarse_row();
    let (near, third) = (row[0], row[2]);
    Ratios {
        strain: strain(near.middle()) / strain(third.middle()),
        curvature: curvature(near.middle()) / curvature(third.middle()),
        miss: near.miss() / third.miss(),
    }
}

/// The uniform strain of 14_10's inset, u = −x, v = y.
pub fn uniform_velocity(x: Q, y: Q) -> (Q, Q) {
    (-x, y)
}

/// Its [u_x, u_y, v_x, v_y] by differences over one radius, exact for a flow linear in x and y.
pub fn uniform_gradient(x: Q, y: Q) -> [Q; 4] {
    let (u, v) = uniform_velocity(x, y);
    let (ux, vx) = uniform_velocity(x + Q::ONE, y);
    let (uy, vy) = uniform_velocity(x, y + Q::ONE);
    [ux - u, uy - u, vx - v, vy - v]
}

/// A row cell's straight-line miss for the uniform strain.
pub fn uniform_miss(cell: Cell) -> Q {
    let u = |x: Q| uniform_velocity(x, Q::ZERO).0;
    (u(cell.middle()) - half(u(cell.outer) + u(cell.inner))).abs()
}

/// The six points 14_4 marks "spin 0", in radii: outside the cylinder and inside the drawing.
pub fn marked_points() -> [(Q, Q); 6] {
    [
        (q(-2), Q::ZERO),
        (Q::new(-3, 2), Q::ONE),
        (Q::ZERO, Q::new(5, 2)),
        (Q::new(3, 2), q(-1)),
        (q(-3), q(-2)),
        (q(3), q(2)),
    ]
}

/// Three points of the wall r = 1 (`ad01`'s): a 3-4-5 point, the front and a 5-12-13 point.
pub fn wall_points() -> [(Q, Q); 3] {
    [
        (Q::new(3, 5), Q::new(4, 5)),
        (q(-1), Q::ZERO),
        (Q::new(5, 13), Q::new(-12, 13)),
    ]
}

/// The shoulders, where the speed is greatest: the top and bottom of the cylinder.
pub fn shoulders() -> [(Q, Q); 2] {
    [(Q::ZERO, Q::ONE), (Q::ZERO, q(-1))]
}

/// `ad03`'s three points off the line (`row.spin.zero.offline`). They test the closed forms, not
/// the flow: (1/2, 1/3) lies inside the circle.
pub fn offline_points() -> [(Q, Q); 3] {
    [
        (Q::new(1, 2), Q::new(1, 3)),
        (Q::new(-3, 2), Q::new(5, 4)),
        (q(2), Q::new(-7, 3)),
    ]
}

/// A rational as a decimal with `places` places.
pub fn decimal(value: Q, places: usize) -> String {
    format!("{:.*}", places, value.to_f64())
}

/// The stream function, ψ = y(1 − 1/r²).
pub fn stream_function(x: f64, y: f64) -> f64 {
    y * (1.0 - 1.0 / (x * x + y * y))
}

/// The height of the streamline ψ = c (c > 0) above the axis at x, for |x| ≥ 1, by bisection:
/// there ∂ψ/∂y = u ≥ 0, so ψ grows with y from −c at y = 0 to more than c at y = c + 1.
fn height(c: f64, x: f64) -> f64 {
    let (mut lo, mut hi) = (0.0, c + 1.0);
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        if stream_function(x, mid) < c {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// The streamline ψ = c (c ≠ 0) from x = −reach to x = reach (reach ≥ 1), left to right, in radii.
/// For c > 0 it is r(θ) = (c + √(c² + 4 sin²θ)) / (2 sin θ), θ ∈ (0, π), sampled through s = cot θ so
/// the points spread along x; c < 0 is its mirror below the axis. Off the dividing streamline u > 0,
/// so x grows along the samples; the two ends are the exact heights at x = ±reach.
pub fn streamline(c: f64, reach: f64, samples: usize) -> Vec<[f64; 2]> {
    let (a, sign) = (c.abs(), c.signum());
    let span = reach / a + 2.0;
    let mut out = vec![[-reach, sign * height(a, -reach)]];
    for i in 0..samples {
        let s = -span + 2.0 * span * i as f64 / (samples - 1) as f64;
        let sin = 1.0 / (1.0 + s * s).sqrt();
        let r = (a + (a * a + 4.0 * sin * sin).sqrt()) / (2.0 * sin);
        let x = r * s * sin;
        if x.abs() < reach {
            out.push([x, sign * r * sin]);
        }
    }
    out.push([reach, sign * height(a, reach)]);
    out
}

/// The exact speed along the centre line, in f64, for drawing its curve.
pub fn speed_f64(x: f64) -> f64 {
    1.0 - 1.0 / (x * x)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(n: i64, d: i64) -> Q {
        Q::new(n, d)
    }

    #[test]
    fn row_speeds_are_the_oracles() {
        let speeds: Vec<String> = row_points().iter().map(|&x| speed(x).to_string()).collect();
        assert_eq!(speeds, ["0", "8/9", "24/25", "48/49", "80/81", "120/121", "168/169"]);
        let row = coarse_row();
        assert_eq!(row[0].guess(), f(4, 9));
        assert_eq!(row[0].exact(), f(3, 4));
        assert!(row.iter().all(|c| c.width() == q(2)));
        for (x, y) in shoulders() {
            assert_eq!(velocity(x, y), (q(2), Q::ZERO));
        }
    }

    #[test]
    fn closed_form_gradient_matches_central_differences() {
        let h = 1e-5;
        for (x, y) in marked_points() {
            let (x, y) = (x.to_f64(), y.to_f64());
            let at = |x: f64, y: f64| {
                let r4 = (x * x + y * y).powi(2);
                [1.0 - (x * x - y * y) / r4, -2.0 * x * y / r4]
            };
            let dx = |k: usize| (at(x + h, y)[k] - at(x - h, y)[k]) / (2.0 * h);
            let dy = |k: usize| (at(x, y + h)[k] - at(x, y - h)[k]) / (2.0 * h);
            let numeric = [dx(0), dy(0), dx(1), dy(1)];
            let closed = gradient(to_q(x), to_q(y));
            for (k, (n, c)) in numeric.iter().zip(closed).enumerate() {
                assert!((n - c.to_f64()).abs() < 1e-6, "component {k} at ({x}, {y})");
            }
        }
    }

    /// The marked points have halves and integers only, so this conversion is exact.
    fn to_q(x: f64) -> Q {
        Q::new((2.0 * x).round() as i64, 2)
    }

    #[test]
    fn no_spin_no_divergence_no_flow_through_the_wall() {
        let mut points: Vec<(Q, Q)> = marked_points().to_vec();
        points.extend(offline_points());
        for x in row_points() {
            points.push((x, Q::ZERO));
        }
        for cell in coarse_row() {
            points.push((cell.middle(), Q::ZERO));
        }
        for (x, y) in points {
            assert!(spin(x, y).is_zero(), "spin at ({x}, {y})");
            assert!(divergence(x, y).is_zero(), "divergence at ({x}, {y})");
        }
        for (x, y) in wall_points() {
            assert_eq!(x * x + y * y, Q::ONE);
            assert!(wall_flow(x, y).is_zero());
        }
        for (x, y) in marked_points() {
            assert!(x * x + y * y > Q::ONE, "({x}, {y}) is outside the cylinder");
        }
    }

    #[test]
    fn the_tolerance_splits_two_cells_then_two_at_the_wall() {
        let row = coarse_row();
        let misses: Vec<String> = row.iter().map(|c| c.miss().to_string()).collect();
        assert_eq!(
            misses,
            [
                "11/36",
                "47/3600",
                "107/44100",
                "191/254016",
                "299/980100",
                "431/2944656"
            ]
        );
        assert!(row.windows(2).all(|w| w[0].miss() > w[1].miss()));
        let [first, second] = rounds();
        assert_eq!(first.split, [true, true, false, false, false, false]);
        assert_eq!((first.cells.len(), first.after.len()), (6, 8));
        let made: Vec<String> = first.made().iter().map(|c| c.miss().to_string()).collect();
        assert_eq!(made, ["13/72", "37/1800", "73/14112", "121/64800"]);
        assert_eq!(second.cells, first.after);
        assert_eq!(second.split, [true, true, false, false, false, false, false, false]);
        assert_eq!(second.after.len(), 10);
        // Testing only the new cells splits the same two: the old far cells pass unchanged.
        assert_eq!(test_row(&first.made(), tolerance()).splits(), second.splits());
        assert_eq!(halve_all(&row).len(), 12);
        assert_eq!(halve_all(&halve_all(&row)).len(), 24);
        assert_eq!(spin_rule(&row), row.to_vec());
    }

    #[test]
    fn strain_and_curvature_on_the_row() {
        let row = coarse_row();
        assert_eq!(strain(row[0].middle()), f(-1, 4));
        assert_eq!(strain(row[2].middle()), f(-1, 108));
        assert_eq!(curvature(row[0].middle()).abs(), f(3, 8));
        assert_eq!(curvature(row[2].middle()).abs(), f(1, 216));
        let r = near_against_third();
        assert_eq!((r.strain, r.curvature, r.miss), (q(27), q(81), f(13475, 107)));
        let [first, _] = rounds();
        for cell in row.iter().copied().chain(first.made()) {
            let (lo, hi) = cell.miss_bounds();
            assert!(lo <= cell.miss() && cell.miss() <= hi, "{cell:?}");
        }
        let biggest = row_points().into_iter().max_by_key(|&x| curvature(x).abs());
        assert_eq!(biggest, Some(q(-1)));
        // On the line the strain is 2/x³, and u_xx is its derivative.
        for x in [-1.5_f64, -2.0, -6.0, -12.5] {
            let h = 1e-5;
            assert!((strain(to_q(x)).to_f64() - 2.0 / x.powi(3)).abs() < 1e-12);
            let numeric = (2.0 / (x + h).powi(3) - 2.0 / (x - h).powi(3)) / (2.0 * h);
            assert!((numeric - curvature(to_q(x)).to_f64()).abs() < 1e-5);
        }
    }

    #[test]
    fn a_uniform_strain_has_no_spin_and_no_miss() {
        let g = uniform_gradient(f(7, 3), f(-2, 5));
        assert_eq!(g, [q(-1), Q::ZERO, Q::ZERO, Q::ONE]);
        assert!((g[2] - g[1]).is_zero() && (g[0] + g[3]).is_zero());
        assert!(coarse_row().into_iter().all(|c| uniform_miss(c).is_zero()));
    }

    #[test]
    fn streamlines_follow_the_stream_function() {
        for c in [0.18, 0.5, 0.9, 1.4, 2.0, -0.5, -2.0] {
            let line = streamline(c, 4.0, 241);
            assert!(line.len() > 40);
            assert_eq!((line[0][0], line[line.len() - 1][0]), (-4.0, 4.0));
            assert!(line.windows(2).all(|w| w[1][0] > w[0][0]), "left to right");
            for &[x, y] in &line {
                assert!((stream_function(x, y) - c).abs() < 1e-9, "psi at ({x}, {y})");
                assert!(x * x + y * y > 1.0);
            }
        }
        assert!((speed_f64(-2.0) - 0.75).abs() < 1e-15);
    }

    #[test]
    fn readouts_round_the_exact_values() {
        let row = coarse_row();
        assert_eq!(decimal(row[0].miss(), 2), "0.31");
        assert_eq!(decimal(row[1].miss(), 3), "0.013");
        assert_eq!(decimal(row[2].miss(), 3), "0.002");
        assert_eq!(decimal(near_against_third().miss, 0), "126");
        assert_eq!(decimal(tolerance(), 2), "0.01");
    }
}
