//! Author calculations, not independent review. The rows from `steps.` to `fv.` are a CC3 scratch draft
//! (chapters 08 to 12 and 19 to 21), uncompiled; `oracle.py` recomputes every one by another route.
use explainer_kit::Facts;

/// The one-form x dy at the point `p`, applied to the step `d`: the point's x times the step's change in y.
pub fn x_dy_at(p: (f64, f64), d: (f64, f64)) -> f64 {
    p.0 * d.1
}

/// x dy along the straight step from `a` to `b`. x is linear along the step, so its midpoint value times the
/// change in y is the exact integral.
pub fn x_dy_step(a: (f64, f64), b: (f64, f64)) -> f64 {
    0.5 * (a.0 + b.0) * (b.1 - a.1)
}

/// x dy over `n` equal straight steps, once counterclockwise around the unit circle.
pub fn polygon_sum(n: usize) -> f64 {
    let p = |k: usize| {
        let t = std::f64::consts::TAU * k as f64 / n as f64;
        (t.cos(), t.sin())
    };
    (0..n).map(|k| x_dy_step(p(k), p(k + 1))).sum()
}

/// dx ∧ dy on the ordered pair of steps (u, v).
pub fn dx_dy(u: (f64, f64), v: (f64, f64)) -> f64 {
    u.0 * v.1 - u.1 * v.0
}

/// An exact fraction, kept in lowest terms with a positive denominator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Q(pub i64, pub i64);

impl Q {
    pub fn new(n: i64, d: i64) -> Q {
        let g = gcd(n.abs(), d.abs()).max(1);
        let s = if d < 0 { -1 } else { 1 };
        Q(s * n / g, s * d / g)
    }
    pub fn add(self, o: Q) -> Q {
        Q::new(self.0 * o.1 + o.0 * self.1, self.1 * o.1)
    }
    pub fn mul(self, o: Q) -> Q {
        Q::new(self.0 * o.0, self.1 * o.1)
    }
    pub fn neg(self) -> Q {
        Q::new(-self.0, self.1)
    }
}

fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

impl std::fmt::Display for Q {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.1 == 1 { write!(f, "{}", self.0) } else { write!(f, "{}/{}", self.0, self.1) }
    }
}

/// The integral of x^a z^b over the unit square [0,1]².
fn square_monomial(a: i64, b: i64) -> Q {
    Q::new(1, (a + 1) * (b + 1))
}

/// Chapter 11/12 cube: ω = g dz∧dx with g = y² + xz on [0,1]³, so dω = 2y dx∧dy∧dz.
/// Returns (interior ∫ 2y, face y = 1 with outward +y, face y = 0 with outward −y).
pub fn cube() -> (Q, Q, Q) {
    let interior = Q::new(2, 1).mul(Q::new(1, 2)); // ∫ 2y dy over [0,1]; the x and z ranges have length 1
    let face_y1 = square_monomial(0, 0).add(square_monomial(1, 1)); // g(x,1,z) = 1 + xz
    let face_y0 = square_monomial(1, 1).neg(); // g(x,0,z) = xz, outward direction reversed
    (interior, face_y1, face_y0)
}

/// Chapter 19 cell [−1/2,1/2]²: for a linear field, the integral along a unit edge is its midpoint value.
/// Returns (outward flux, counterclockwise circulation) summed over the four edges, per edge in `edges`.
pub fn cell(field: fn(Q, Q) -> (Q, Q)) -> (Vec<Q>, Vec<Q>) {
    let h = Q::new(1, 2);
    let (z, one) = (Q::new(0, 1), Q::new(1, 1));
    // (midpoint, outward normal, counterclockwise tangent): right, top, left, bottom.
    let edges = [
        ((h, z), (one, z), (z, one)),
        ((z, h), (z, one), (one.neg(), z)),
        ((h.neg(), z), (one.neg(), z), (z, one.neg())),
        ((z, h.neg()), (z, one.neg()), (one, z)),
    ];
    let dot = |a: (Q, Q), b: (Q, Q)| a.0.mul(b.0).add(a.1.mul(b.1));
    let cross = edges.iter().map(|&(m, n, _)| dot(field(m.0, m.1), n)).collect();
    let along = edges.iter().map(|&(m, _, t)| dot(field(m.0, m.1), t)).collect();
    (cross, along)
}

pub fn spreading(x: Q, y: Q) -> (Q, Q) {
    (x.mul(Q::new(1, 2)), y.mul(Q::new(1, 2)))
}

pub fn turning(x: Q, y: Q) -> (Q, Q) {
    (y.mul(Q::new(-1, 2)), x.mul(Q::new(1, 2)))
}

fn total(v: &[Q]) -> Q {
    v.iter().fold(Q::new(0, 1), |a, &b| a.add(b))
}

/// Chapter 20/21 grid, in hundredths: cells A B / C D, shared faces A|B, A/C, B/D, C|D.
pub const FV_BEFORE: [i64; 4] = [400, 200, 100, 100];
pub const FV_FACES: [(usize, usize); 4] = [(0, 1), (0, 2), (1, 3), (2, 3)];
/// Hundredths that flow in through the open outer face of cell B in chapter 21.
pub const FV_INFLOW: i64 = 25;

/// One explicit step with flux = (difference)/10 across each shared face: the fluxes and the new amounts.
pub fn fv_step(u: [i64; 4]) -> ([i64; 4], [i64; 4]) {
    let mut flux = [0; 4];
    let mut next = u;
    for (f, &(i, j)) in FV_FACES.iter().enumerate() {
        assert_eq!((u[i] - u[j]) % 10, 0, "the example keeps exact hundredths");
        flux[f] = (u[i] - u[j]) / 10;
        next[i] -= flux[f];
        next[j] += flux[f];
    }
    (flux, next)
}

/// Hundredths as the decimal shown and spoken: 350 → 3.5, 235 → 2.35, 800 → 8, 20 → 0.2.
pub fn hundredths(n: i64) -> String {
    let (w, r) = (n / 100, n % 100);
    if r == 0 {
        format!("{w}")
    } else if r % 10 == 0 {
        format!("{w}.{}", r / 10)
    } else {
        format!("{w}.{r:02}")
    }
}

fn join(v: impl IntoIterator<Item = String>) -> String {
    v.into_iter().collect::<Vec<_>>().join(",")
}

pub fn facts() -> Result<Facts,String> {
    let mut f = Facts::new();
    f.add("two_squares.directed_edges", 8)?;
    f.add("two_squares.canceled_edges", 2)?;
    f.add("two_squares.outer_edges", 6)?;
    f.add("interval.internal_endpoint_pairs", 2)?;
    // 08: x dy on steps at (2, 1); the moved step sits where x = 3.
    let (p, d) = ((2.0, 1.0), (1.0, 0.5));
    f.add("steps.value", x_dy_at(p, d))?;
    f.add("steps.reversed", x_dy_at(p, (-d.0, -d.1)))?;
    f.add("steps.doubled", x_dy_at(p, (2.0 * d.0, 2.0 * d.1)))?;
    f.add("steps.sideways", x_dy_at(p, (1.0, 0.0)))?;
    f.add("steps.at_x3", x_dy_at((3.0, 2.0), d))?;
    // 09: midpoint-sampled sums around the unit circle.
    for n in [4, 8, 16, 64] {
        f.add(&format!("circle.sum.{n}"), format!("{:.2}", polygon_sum(n)))?;
    }
    // 10: dx∧dy on u = (2, 0), v = (1, 1.5); the tip of v slides along (1 + 2t, 1.5 − 1.5t).
    let (u, v) = ((2.0, 0.0), (1.0, 1.5));
    f.add("areas.uv", dx_dy(u, v))?;
    f.add("areas.vu", dx_dy(v, u))?;
    f.add("areas.halfway", dx_dy(u, (2.0, 0.75)))?;
    f.add("areas.flat", dx_dy(u, (3.0, 0.0)))?;
    // 11/12: the cube.
    let (inside, top, bottom) = cube();
    f.add("cube.interior", inside)?;
    f.add("cube.face_y1", top)?;
    f.add("cube.face_y0", bottom)?;
    f.add("cube.boundary", top.add(bottom))?;
    // 19: one cell, two fields.
    let (cross, along) = cell(spreading);
    f.add("cell.spreading.cross", join(cross.iter().map(Q::to_string)))?;
    f.add("cell.spreading.net_outflow", total(&cross))?;
    f.add("cell.spreading.circulation", total(&along))?;
    let (cross, along) = cell(turning);
    f.add("cell.turning.along", join(along.iter().map(Q::to_string)))?;
    f.add("cell.turning.net_outflow", total(&cross))?;
    f.add("cell.turning.circulation", total(&along))?;
    // 20/21: four cells, one explicit step, then one open face.
    let (flux, next) = fv_step(FV_BEFORE);
    f.add("fv.flux", join(flux.iter().map(|&n| hundredths(n))))?;
    f.add("fv.after", join(next.iter().map(|&n| hundredths(n))))?;
    f.add("fv.total_before", hundredths(FV_BEFORE.iter().sum()))?;
    f.add("fv.total_after", hundredths(next.iter().sum()))?;
    f.add("fv.inflow_cell", hundredths(next[1] + FV_INFLOW))?;
    f.add("fv.open_total", hundredths(next.iter().sum::<i64>() + FV_INFLOW))?;
    f.self_check()?;
    Ok(f)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_sampled_sum_is_the_area_of_its_polygon() {
        for n in [4usize, 8, 16, 64] {
            let area = 0.5 * n as f64 * (std::f64::consts::TAU / n as f64).sin();
            assert!((polygon_sum(n) - area).abs() < 1e-12);
        }
    }

    #[test]
    fn the_cube_boundary_equals_the_interior() {
        let (inside, top, bottom) = cube();
        assert_eq!((top, bottom), (Q::new(5, 4), Q::new(-1, 4)));
        assert_eq!(top.add(bottom), inside);
    }

    #[test]
    fn spreading_crosses_and_turning_runs_along() {
        let q = Q::new(1, 4);
        assert_eq!(cell(spreading), (vec![q; 4], vec![Q::new(0, 1); 4]));
        assert_eq!(cell(turning), (vec![Q::new(0, 1); 4], vec![q; 4]));
    }

    #[test]
    fn a_closed_grid_keeps_its_total_and_an_open_face_adds_its_inflow() {
        let (flux, next) = fv_step(FV_BEFORE);
        assert_eq!(flux, [20, 30, 10, 0]);
        assert_eq!(next, [350, 210, 130, 110]);
        assert_eq!(next.iter().sum::<i64>(), FV_BEFORE.iter().sum::<i64>());
        assert_eq!(hundredths(next.iter().sum::<i64>() + FV_INFLOW), "8.25");
    }
}
