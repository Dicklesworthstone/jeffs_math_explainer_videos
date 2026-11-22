//! The four-cell box of chapters 07 to 10, solved exactly. Chapter 09 draws no number that is not
//! computed here from one input, the scripted provisional flow, and the face incidence of the box.
//!
//! Cells (FC2 line 15): A lower left, B lower right, C upper left, D upper right. Faces, in the
//! script's order, point along the grey positive direction: A→B, C→D, A→C, B→D. Box units:
//! cell side h, density ρ and time step Δt are all 1 (the 08_5 tag).
//!
//! Sign convention (09_1 "Precisely"): rows read L p = d with L = −div grad and d = −div u*.
//! A face's correction is −grad p, the "from" pressure minus the "to" pressure, so it pushes from
//! high pressure toward low. Standard library only, so the tests run without the renderer.

use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};

/// An exact rational in lowest terms with a positive denominator.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Q {
    num: i64,
    den: i64,
}

fn gcd(a: i64, b: i64) -> i64 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

impl Q {
    pub const ZERO: Q = Q { num: 0, den: 1 };
    pub const ONE: Q = Q { num: 1, den: 1 };

    pub fn new(num: i64, den: i64) -> Q {
        assert!(den != 0, "a rational needs a nonzero denominator");
        let g = gcd(num, den).max(1);
        let sign = if den < 0 { -1 } else { 1 };
        Q {
            num: sign * num / g,
            den: sign * den / g,
        }
    }

    pub fn int(n: i64) -> Q {
        Q { num: n, den: 1 }
    }

    pub fn is_zero(self) -> bool {
        self.num == 0
    }

    pub fn abs(self) -> Q {
        Q {
            num: self.num.abs(),
            den: self.den,
        }
    }

    pub fn to_f64(self) -> f64 {
        self.num as f64 / self.den as f64
    }

    /// Display math: `\tfrac{3}{4}`, `-\tfrac{1}{4}`, `-1`, `0`.
    pub fn tex(self) -> String {
        let sign = if self.num < 0 { "-" } else { "" };
        if self.den == 1 {
            format!("{sign}{}", self.num.abs())
        } else {
            format!("{sign}\\tfrac{{{}}}{{{}}}", self.num.abs(), self.den)
        }
    }
}

impl fmt::Display for Q {
    /// Plain text: `3/4`, `-1/4`, `-1`, `0` (the fact strings and the independent check use it).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.den == 1 {
            write!(f, "{}", self.num)
        } else {
            write!(f, "{}/{}", self.num, self.den)
        }
    }
}

impl Add for Q {
    type Output = Q;
    fn add(self, o: Q) -> Q {
        Q::new(self.num * o.den + o.num * self.den, self.den * o.den)
    }
}

impl Sub for Q {
    type Output = Q;
    fn sub(self, o: Q) -> Q {
        Q::new(self.num * o.den - o.num * self.den, self.den * o.den)
    }
}

impl Mul for Q {
    type Output = Q;
    fn mul(self, o: Q) -> Q {
        Q::new(self.num * o.num, self.den * o.den)
    }
}

impl Div for Q {
    type Output = Q;
    fn div(self, o: Q) -> Q {
        assert!(!o.is_zero(), "division by an exact zero");
        Q::new(self.num * o.den, self.den * o.num)
    }
}

impl Neg for Q {
    type Output = Q;
    fn neg(self) -> Q {
        Q {
            num: -self.num,
            den: self.den,
        }
    }
}

impl PartialOrd for Q {
    fn partial_cmp(&self, o: &Q) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

impl Ord for Q {
    fn cmp(&self, o: &Q) -> Ordering {
        (self.num * o.den).cmp(&(o.num * self.den))
    }
}

/// One value per cell (A, B, C, D) or per face (A→B, C→D, A→C, B→D).
pub type V4 = [Q; 4];
/// A 4 × 4 matrix, rows and columns in cell order.
pub type M4 = [[Q; 4]; 4];

pub const CELLS: [&str; 4] = ["A", "B", "C", "D"];
/// Column and row of each cell in the 2 × 2 box (A at column 0, row 0).
pub const CELL_AT: [(usize, usize); 4] = [(0, 0), (1, 0), (0, 1), (1, 1)];
/// Each face as (from, to) along its grey positive direction, in the script's order.
pub const FACES: [(usize, usize); 4] = [(0, 1), (2, 3), (0, 2), (1, 3)];
/// The scripted provisional flow u* (08_5): speed one from A to B and nothing on the other faces.
pub const PROVISIONAL: [i64; 4] = [1, 0, 0, 0];
/// The loop of 09_7, as the cells it visits: A→B→D→C→A.
pub const LOOP_CELLS: [usize; 5] = [0, 1, 3, 2, 0];
/// Box units (the 08_5 tag): cell side h, density ρ, time step Δt.
pub const UNITS: Units = Units { h: 1, rho: 1, dt: 1 };
/// The cell 09_1 builds first.
pub const ROW_A: usize = 0;
/// The cell whose pressure is held at zero for the first solution (09_5).
pub const PIN: usize = 0;
/// The damping of the single sweep that gives 09_8 its unfinished pressure (FC2: weight one half).
pub const SWEEP_WEIGHT: (i64, i64) = (1, 2);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Units {
    pub h: i64,
    pub rho: i64,
    pub dt: i64,
}

pub fn face_name(f: usize) -> String {
    let (a, b) = FACES[f];
    format!("{}{}", CELLS[a], CELLS[b])
}

/// Net outflow of each cell: a face's value leaves its "from" cell and enters its "to" cell.
pub fn div(u: &V4) -> V4 {
    let mut d = [Q::ZERO; 4];
    for (&(a, b), &value) in FACES.iter().zip(u) {
        d[a] = d[a] + value;
        d[b] = d[b] - value;
    }
    d
}

/// The pressure difference along each face's positive direction: "to" minus "from".
pub fn grad(p: &V4) -> V4 {
    FACES.map(|(a, b)| p[b] - p[a])
}

pub fn neg(v: &V4) -> V4 {
    v.map(|x| -x)
}

pub fn add(a: &V4, b: &V4) -> V4 {
    std::array::from_fn(|i| a[i] + b[i])
}

pub fn sub(a: &V4, b: &V4) -> V4 {
    std::array::from_fn(|i| a[i] - b[i])
}

pub fn scale(c: Q, v: &V4) -> V4 {
    v.map(|x| c * x)
}

pub fn dot(a: &V4, b: &V4) -> Q {
    a.iter().zip(b).fold(Q::ZERO, |s, (&x, &y)| s + x * y)
}

pub fn sum(v: &V4) -> Q {
    v.iter().fold(Q::ZERO, |s, &x| s + x)
}

/// L = −div grad, column by column: column j is −div grad of the j-th unit pressure.
pub fn matrix() -> M4 {
    let mut m = [[Q::ZERO; 4]; 4];
    for j in 0..4 {
        let mut unit = [Q::ZERO; 4];
        unit[j] = Q::ONE;
        let column = neg(&div(&grad(&unit)));
        for (row, value) in m.iter_mut().zip(column) {
            row[j] = value;
        }
    }
    m
}

pub fn apply(m: &M4, p: &V4) -> V4 {
    m.map(|row| dot(&row, p))
}

/// Faces a cell of an `nx` × `ny` grid shares with another cell; the walls take the rest.
pub fn open_neighbours(nx: usize, ny: usize, column: usize, row: usize) -> usize {
    [column > 0, column + 1 < nx, row > 0, row + 1 < ny]
        .iter()
        .filter(|&&open| open)
        .count()
}

/// Solve L p = d with p[pin] = 0. The pinned row is dropped: the rows of L sum to zero, and so do
/// the entries of d in a sealed box, so the dropped equation follows from the other three.
#[allow(clippy::needless_range_loop)] // Gauss–Jordan reads clearest by index.
pub fn solve_pinned(m: &M4, d: &V4, pin: usize) -> V4 {
    let keep: Vec<usize> = (0..4).filter(|&i| i != pin).collect();
    let n = keep.len();
    let mut a: Vec<Vec<Q>> = keep
        .iter()
        .map(|&i| keep.iter().map(|&j| m[i][j]).chain([d[i]]).collect())
        .collect();
    for col in 0..n {
        let pivot = (col..n)
            .find(|&r| !a[r][col].is_zero())
            .expect("the pinned system is nonsingular");
        a.swap(col, pivot);
        let lead = a[col][col];
        for x in a[col].iter_mut() {
            *x = *x / lead;
        }
        let pivot_row = a[col].clone();
        for (r, row) in a.iter_mut().enumerate() {
            if r != col && !row[col].is_zero() {
                let factor = row[col];
                for (x, &y) in row.iter_mut().zip(&pivot_row) {
                    *x = *x - factor * y;
                }
            }
        }
    }
    let mut p = [Q::ZERO; 4];
    for (k, &i) in keep.iter().enumerate() {
        p[i] = a[k][n];
    }
    p
}

/// The faces of the loop in travel order, each with +1 when travel follows its grey direction.
pub fn loop_faces() -> [(usize, i64); 4] {
    std::array::from_fn(|k| {
        let (from, to) = (LOOP_CELLS[k], LOOP_CELLS[k + 1]);
        FACES
            .iter()
            .enumerate()
            .find_map(|(f, &(a, b))| {
                if (a, b) == (from, to) {
                    Some((f, 1))
                } else if (a, b) == (to, from) {
                    Some((f, -1))
                } else {
                    None
                }
            })
            .expect("consecutive loop cells share a face")
    })
}

/// The flow around the loop, each face counted in the direction of travel.
pub fn loop_sum(u: &V4) -> Q {
    loop_faces()
        .iter()
        .fold(Q::ZERO, |s, &(f, sign)| s + Q::int(sign) * u[f])
}

/// The loop pattern: speed one on every face along the direction of travel, as a face vector.
pub fn loop_pattern() -> V4 {
    let mut v = [Q::ZERO; 4];
    for (f, sign) in loop_faces() {
        v[f] = Q::int(sign);
    }
    v
}

/// The rank of the divergence (faces to cells), by exact elimination.
#[allow(clippy::needless_range_loop)] // elimination by index, as in `solve_pinned`
pub fn divergence_rank() -> usize {
    let mut rows: Vec<Vec<Q>> = (0..4)
        .map(|cell| {
            (0..4)
                .map(|f| {
                    let mut unit = [Q::ZERO; 4];
                    unit[f] = Q::ONE;
                    div(&unit)[cell]
                })
                .collect()
        })
        .collect();
    let mut rank = 0;
    for col in 0..4 {
        let Some(pivot) = (rank..rows.len()).find(|&r| !rows[r][col].is_zero()) else {
            continue;
        };
        rows.swap(rank, pivot);
        let lead = rows[rank].clone();
        for (r, row) in rows.iter_mut().enumerate() {
            if r != rank && !row[col].is_zero() {
                let factor = row[col] / lead[col];
                for (x, &y) in row.iter_mut().zip(&lead) {
                    *x = *x - factor * y;
                }
            }
        }
        rank += 1;
    }
    rank
}

/// The four separable ±1 patterns of the 2 × 2 box, each with its exact eigenvalue of L
/// (the Rayleigh quotient; `BoxFacts::check` confirms L v = λ v and that the four exhaust the trace).
pub fn modes(m: &M4) -> [(V4, Q); 4] {
    let signs = [(1, 1), (-1, 1), (1, -1), (-1, -1)];
    signs.map(|(sx, sy)| {
        let v: V4 = CELL_AT.map(|(c, r)| Q::int(pow(sx, c) * pow(sy, r)));
        (v, dot(&v, &apply(m, &v)) / dot(&v, &v))
    })
}

fn pow(base: i64, exponent: usize) -> i64 {
    (0..exponent).fold(1, |acc, _| acc * base)
}

/// Every number chapter 09 shows, derived once.
#[derive(Clone, Debug)]
pub struct BoxFacts {
    pub provisional: V4,
    /// Net outflow of u*, the red badges of 08_5.
    pub imbalance: V4,
    /// d = −div u*.
    pub rhs: V4,
    pub matrix: M4,
    pub modes: [(V4, Q); 4],
    pub open_faces: [usize; 4],
    /// The centre cell of a 3 × 3 grid (the 09_3 inset).
    pub interior_open_faces: usize,
    pub pinned: V4,
    pub shift: Q,
    pub mean_zero: V4,
    pub gradient: V4,
    /// The face corrections, −grad p.
    pub push: V4,
    /// −grad p computed from the mean-zero pressures (09_6).
    pub push_mean_zero: V4,
    pub corrected: V4,
    pub corrected_div: V4,
    /// Cells from the highest pinned pressure to the lowest.
    pub ranking: [usize; 4],
    pub loop_faces: [(usize, i64); 4],
    pub loop_before: Q,
    pub loop_pressure: Q,
    pub loop_after: Q,
    pub loop_each: Q,
    pub divergence_rank: usize,
    /// 09_8: one damped sweep from p = 0.
    pub unfinished: V4,
    pub unfinished_residual: V4,
    pub unfinished_push: V4,
    pub unfinished_flow: V4,
    pub unfinished_div: V4,
}

impl BoxFacts {
    pub fn solve() -> BoxFacts {
        let provisional = PROVISIONAL.map(Q::int);
        let imbalance = div(&provisional);
        let rhs = neg(&imbalance);
        let matrix = matrix();
        let modes = modes(&matrix);
        let open_faces = CELL_AT.map(|(c, r)| open_neighbours(2, 2, c, r));
        let interior_open_faces = open_neighbours(3, 3, 1, 1);
        let pinned = solve_pinned(&matrix, &rhs, PIN);
        let shift = sum(&pinned) / Q::int(4);
        let mean_zero = pinned.map(|p| p - shift);
        let gradient = grad(&pinned);
        let push = neg(&gradient);
        let push_mean_zero = neg(&grad(&mean_zero));
        let corrected = add(&provisional, &push);
        let corrected_div = div(&corrected);
        let mut ranking = [0, 1, 2, 3];
        ranking.sort_by(|&a, &b| pinned[b].cmp(&pinned[a]));
        let loop_after = loop_sum(&corrected);
        let loop_each = loop_after / Q::int(loop_faces().len() as i64);
        let weight = Q::new(SWEEP_WEIGHT.0, SWEEP_WEIGHT.1);
        let start = [Q::ZERO; 4];
        let first = sub(&rhs, &apply(&matrix, &start));
        let unfinished: V4 = std::array::from_fn(|i| start[i] + weight * first[i] / matrix[i][i]);
        let unfinished_residual = sub(&rhs, &apply(&matrix, &unfinished));
        let unfinished_push = neg(&grad(&unfinished));
        let unfinished_flow = add(&provisional, &unfinished_push);
        let unfinished_div = div(&unfinished_flow);
        BoxFacts {
            provisional,
            imbalance,
            rhs,
            matrix,
            modes,
            open_faces,
            interior_open_faces,
            pinned,
            shift,
            mean_zero,
            gradient,
            push,
            push_mean_zero,
            corrected,
            corrected_div,
            ranking,
            loop_faces: loop_faces(),
            loop_before: loop_sum(&provisional),
            loop_pressure: loop_sum(&gradient),
            loop_after,
            loop_each,
            divergence_rank: divergence_rank(),
            unfinished,
            unfinished_residual,
            unfinished_push,
            unfinished_flow,
            unfinished_div,
        }
    }

    /// The identities the chapter states, checked on the exact values before anything is drawn.
    pub fn check(&self) -> Result<(), String> {
        let zero = [Q::ZERO; 4];
        let fail = |what: &str| Err(format!("four-cell box: {what}"));
        if sum(&self.rhs) != Q::ZERO {
            return fail("a sealed box's imbalances must add to zero");
        }
        if apply(&self.matrix, &self.pinned) != self.rhs || apply(&self.matrix, &self.mean_zero) != self.rhs {
            return fail("the pressures do not solve L p = d");
        }
        if self.pinned[PIN] != Q::ZERO || sum(&self.mean_zero) != Q::ZERO {
            return fail("the pin or the mean-zero shift is wrong");
        }
        if self.push != neg(&self.gradient) || self.push_mean_zero != self.push {
            return fail("a push is not minus the gradient, or a shared constant changed it");
        }
        if self.corrected_div != zero {
            return fail("the corrected flow does not balance every cell");
        }
        for (i, row) in self.matrix.iter().enumerate() {
            if row[i] != Q::int(self.open_faces[i] as i64) || sum(row) != Q::ZERO {
                return fail("a diagonal is not the count of open faces, or a row does not sum to zero");
            }
            for (j, &entry) in row.iter().enumerate() {
                let shared = FACES.contains(&(i, j)) || FACES.contains(&(j, i));
                if i != j && entry != if shared { -Q::ONE } else { Q::ZERO } {
                    return fail("an off-diagonal entry is not minus one exactly on a shared face");
                }
            }
        }
        let trace = (0..4).fold(Q::ZERO, |s, i| s + self.matrix[i][i]);
        let eigen_sum = self.modes.iter().fold(Q::ZERO, |s, &(_, l)| s + l);
        if self.modes.iter().any(|(v, l)| apply(&self.matrix, v) != scale(*l, v)) || eigen_sum != trace {
            return fail("the four patterns are not a complete set of eigenvectors");
        }
        if self.loop_pressure != Q::ZERO || self.loop_after != self.loop_before {
            return fail("the pressure differences do not cancel around the loop");
        }
        if self.divergence_rank != 3 || self.corrected != scale(self.loop_each, &loop_pattern()) {
            return fail("the balanced flows are not the one loop pattern");
        }
        if self.unfinished_div != neg(&self.unfinished_residual) {
            return fail("the leftover imbalance is not minus the residual");
        }
        if UNITS != (Units { h: 1, rho: 1, dt: 1 }) {
            return fail("the box is drawn in units where h = rho = dt = 1");
        }
        Ok(())
    }

    /// The left side of row `i` of L p, as display math: `2p_A - p_B - p_C`.
    pub fn row_tex(&self, i: usize) -> String {
        let mut out = String::new();
        for (j, &c) in self.matrix[i].iter().enumerate() {
            if c.is_zero() {
                continue;
            }
            let magnitude = if c.abs() == Q::ONE {
                String::new()
            } else {
                c.abs().tex()
            };
            let sign = match (out.is_empty(), c < Q::ZERO) {
                (true, true) => "-",
                (true, false) => "",
                (false, true) => " - ",
                (false, false) => " + ",
            };
            out.push_str(&format!("{sign}{magnitude}p_{}", CELLS[j]));
        }
        out
    }

    /// Row `i` before the imbalance moves across: `1 + 2p_A - p_B - p_C = 0`.
    pub fn balance_tex(&self, i: usize) -> String {
        let row = self.row_tex(i);
        let joined = if let Some(magnitude) = row.strip_prefix('-') {
            format!(" - {magnitude}")
        } else {
            format!(" + {row}")
        };
        format!("{}{joined} = 0", self.imbalance[i].tex())
    }

    /// Row `i` in L p = d form: `2p_A - p_B - p_C = -1`.
    pub fn equation_tex(&self, i: usize) -> String {
        format!("{} = {}", self.row_tex(i), self.rhs[i].tex())
    }

    /// The corrected flow on face `f` in symbols: `u_{AB} = 1 - (p_B - p_A)`.
    pub fn face_update_tex(&self, f: usize) -> String {
        let (a, b) = FACES[f];
        format!(
            "u_{{{}}} = {} - (p_{} - p_{})",
            face_name(f),
            self.provisional[f].tex(),
            CELLS[b],
            CELLS[a]
        )
    }

    /// The canonical fact table: one row per fact key the script cites for chapter 09, plus the
    /// chapter 08 end state it opens on. `check_chapter09_01.py` recomputes every row independently.
    pub fn fact_rows(&self) -> Vec<(String, String)> {
        let list = |v: &V4| v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ");
        let rows = self
            .matrix
            .iter()
            .map(|row| row.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" "))
            .collect::<Vec<_>>()
            .join("; ");
        let mut eigenvalues: Vec<Q> = self.modes.iter().map(|&(_, l)| l).collect();
        eigenvalues.sort();
        let plain = |s: String| s.replace("\\tfrac", "").replace('_', "");
        let mut out = vec![
            ("box.ustar".to_string(), list(&self.provisional)),
            ("box.div.ustar".to_string(), list(&self.imbalance)),
            ("box.unew".to_string(), list(&self.corrected)),
            ("box.div.after".to_string(), list(&self.corrected_div)),
            (
                "box.rowA.balance".to_string(),
                format!(
                    "{}; {}",
                    plain(self.balance_tex(ROW_A)),
                    plain(self.equation_tex(ROW_A))
                ),
            ),
            ("box.rhs".to_string(), list(&self.rhs)),
            ("box.matrix".to_string(), rows),
            (
                "box.eigenvalues".to_string(),
                eigenvalues.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", "),
            ),
            (
                "box.diagonal.open.neighbours".to_string(),
                self.open_faces
                    .iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
            (
                "box.interior.diagonal.2d".to_string(),
                self.interior_open_faces.to_string(),
            ),
            ("box.p.pinned".to_string(), list(&self.pinned)),
            (
                "box.ranking".to_string(),
                self.ranking.iter().map(|&i| CELLS[i]).collect::<Vec<_>>().join(", "),
            ),
        ];
        for name in ["AB", "BD", "AC", "CD"] {
            let f = (0..4)
                .find(|&f| face_name(f) == name)
                .expect("every pushed face exists");
            out.push((format!("box.push.{name}"), self.push[f].to_string()));
        }
        out.extend([
            ("box.p.shift".to_string(), self.shift.to_string()),
            ("box.p.meanzero".to_string(), list(&self.mean_zero)),
            ("box.gauge.same.faces".to_string(), list(&self.push_mean_zero)),
            ("box.loop.before".to_string(), self.loop_before.to_string()),
            ("box.loop.pressure".to_string(), self.loop_pressure.to_string()),
            ("box.loop.after".to_string(), self.loop_after.to_string()),
            ("box.loop.each".to_string(), self.loop_each.to_string()),
            (
                "box.residual.identity".to_string(),
                format!(
                    "p = {}; r = {}; div u_new = {}",
                    list(&self.unfinished),
                    list(&self.unfinished_residual),
                    list(&self.unfinished_div)
                ),
            ),
            (
                "units.projection".to_string(),
                format!(
                    "h = {}, rho = {}, dt = {}; correction factor dt/rho = {}; matrix factor dt/(rho h^2) = {}",
                    UNITS.h,
                    UNITS.rho,
                    UNITS.dt,
                    Q::new(UNITS.dt, UNITS.rho),
                    Q::new(UNITS.dt, UNITS.rho * UNITS.h * UNITS.h)
                ),
            ),
        ]);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pinned against `check_chapter09_01.py`, which parses this table and recomputes each value
    /// with its own incidence-matrix arithmetic and Cramer's rule (no code shared with this file).
    const PINS: [(&str, &str); 27] = [
        ("box.ustar", "1, 0, 0, 0"),
        ("box.div.ustar", "1, -1, 0, 0"),
        ("box.unew", "1/4, -1/4, -1/4, 1/4"),
        ("box.div.after", "0, 0, 0, 0"),
        ("box.rowA.balance", "1 + 2pA - pB - pC = 0; 2pA - pB - pC = -1"),
        ("box.rhs", "-1, 1, 0, 0"),
        ("box.matrix", "2 -1 -1 0; -1 2 0 -1; -1 0 2 -1; 0 -1 -1 2"),
        ("box.eigenvalues", "0, 2, 2, 4"),
        ("box.diagonal.open.neighbours", "2, 2, 2, 2"),
        ("box.interior.diagonal.2d", "4"),
        ("box.p.pinned", "0, 3/4, 1/4, 1/2"),
        ("box.ranking", "B, D, C, A"),
        ("box.push.AB", "-3/4"),
        ("box.push.BD", "1/4"),
        ("box.push.AC", "-1/4"),
        ("box.push.CD", "-1/4"),
        ("box.p.shift", "3/8"),
        ("box.p.meanzero", "-3/8, 3/8, -1/8, 1/8"),
        ("box.gauge.same.faces", "-3/4, -1/4, -1/4, 1/4"),
        ("box.loop.before", "1"),
        ("box.loop.pressure", "0"),
        ("box.loop.after", "1"),
        ("box.loop.each", "1/4"),
        (
            "box.residual.identity",
            "p = -1/4, 1/4, 0, 0; r = -1/4, 1/4, -1/4, 1/4; div u_new = 1/4, -1/4, 1/4, -1/4",
        ),
        (
            "units.projection",
            "h = 1, rho = 1, dt = 1; correction factor dt/rho = 1; matrix factor dt/(rho h^2) = 1",
        ),
        ("loop.cells", "A, B, D, C, A"),
        ("faces", "AB, CD, AC, BD"),
    ];

    fn q(n: i64, d: i64) -> Q {
        Q::new(n, d)
    }

    #[test]
    fn fact_rows_match_the_independent_pins() {
        let facts = BoxFacts::solve();
        facts.check().unwrap();
        let rows = facts.fact_rows();
        let pinned: Vec<(String, String)> = PINS[..25]
            .iter()
            .map(|&(k, v)| (k.to_string(), v.to_string()))
            .collect();
        assert_eq!(rows, pinned);
        let loop_cells = LOOP_CELLS.map(|i| CELLS[i]).join(", ");
        assert_eq!(loop_cells, PINS[25].1);
        assert_eq!((0..4).map(face_name).collect::<Vec<_>>().join(", "), PINS[26].1);
    }

    #[test]
    fn every_signed_row_balances_for_both_pressure_choices() {
        let f = BoxFacts::solve();
        for p in [f.pinned, f.mean_zero] {
            for (i, row) in f.matrix.iter().enumerate() {
                // Row i, written as the net outflow of the corrected flow, is zero: imbalance + L p = 0.
                assert_eq!(f.imbalance[i] + dot(row, &p), Q::ZERO, "row {}", CELLS[i]);
                assert_eq!(dot(row, &p), f.rhs[i], "row {}", CELLS[i]);
            }
        }
        assert_eq!(f.balance_tex(0), "1 + 2p_A - p_B - p_C = 0");
        assert_eq!(f.equation_tex(0), "2p_A - p_B - p_C = -1");
        assert_eq!(f.equation_tex(1), "-p_A + 2p_B - p_D = 1");
        assert_eq!(f.balance_tex(1), "-1 - p_A + 2p_B - p_D = 0");
        assert_eq!(f.face_update_tex(0), "u_{AB} = 1 - (p_B - p_A)");
        assert_eq!(f.face_update_tex(2), "u_{AC} = 0 - (p_C - p_A)");
    }

    #[test]
    fn mass_is_conserved_cell_by_cell_and_in_total() {
        let f = BoxFacts::solve();
        // Every face leaves one cell and enters another, so the badges always add to zero.
        for u in [
            f.provisional,
            f.corrected,
            f.unfinished_flow,
            [q(3, 7), q(-2, 5), Q::ONE, q(1, 9)],
        ] {
            assert_eq!(sum(&div(&u)), Q::ZERO);
        }
        assert_eq!(f.corrected_div, [Q::ZERO; 4]);
        assert_eq!(f.corrected, [q(1, 4), q(-1, 4), q(-1, 4), q(1, 4)]);
        // Each cell balances its two open faces (08_7): outflow on one, inflow on the other.
        for cell in 0..4 {
            let touching: Vec<usize> = (0..4).filter(|&g| FACES[g].0 == cell || FACES[g].1 == cell).collect();
            assert_eq!(touching.len(), 2);
        }
    }

    #[test]
    fn ring_fluxes_are_exact_and_the_innocent_faces_must_move() {
        let f = BoxFacts::solve();
        assert_eq!(f.loop_faces, [(0, 1), (3, 1), (1, -1), (2, -1)]);
        assert_eq!(
            (f.loop_before, f.loop_pressure, f.loop_after, f.loop_each),
            (Q::ONE, Q::ZERO, Q::ONE, q(1, 4))
        );
        // Every face, counted in the direction of travel, carries the same quarter.
        for &(face, sign) in &f.loop_faces {
            assert_eq!(Q::int(sign) * f.corrected[face], q(1, 4));
        }
        // Any pressure at all leaves the loop total alone: the differences telescope.
        for p in [f.pinned, [q(5, 3), q(-7, 2), Q::ZERO, q(11, 13)]] {
            assert_eq!(loop_sum(&grad(&p)), Q::ZERO);
        }
        // Divergence has rank three, so the balanced flows form a line: multiples of the loop.
        // A balanced flow with zero on A→B would be zero everywhere, and its loop total would be 0, not 1.
        assert_eq!(f.divergence_rank, 3);
        let pattern = loop_pattern();
        assert_eq!(div(&pattern), [Q::ZERO; 4]);
        assert_eq!(loop_sum(&scale(Q::ZERO, &pattern)), Q::ZERO);
        for innocent in 1..4 {
            assert_eq!(f.provisional[innocent], Q::ZERO);
            assert_ne!(
                f.corrected[innocent],
                Q::ZERO,
                "face {} must change",
                face_name(innocent)
            );
        }
    }

    #[test]
    fn a_shared_constant_changes_no_face_correction() {
        let f = BoxFacts::solve();
        assert_eq!(f.shift, q(3, 8));
        assert_eq!(f.mean_zero, [q(-3, 8), q(3, 8), q(-1, 8), q(1, 8)]);
        for c in [f.shift, q(-5, 2), q(17, 3)] {
            assert_eq!(grad(&f.pinned.map(|p| p + c)), f.gradient);
        }
        assert_eq!(f.push_mean_zero, f.push);
        assert_eq!(apply(&f.matrix, &[Q::ONE; 4]), [Q::ZERO; 4]);
    }

    #[test]
    fn pushes_run_from_high_pressure_to_low() {
        let f = BoxFacts::solve();
        assert_eq!(f.push, [q(-3, 4), q(-1, 4), q(-1, 4), q(1, 4)]);
        for (face, &(a, b)) in FACES.iter().enumerate() {
            // A positive push points from "from" to "to", which needs the higher pressure at "from".
            assert_eq!(f.push[face] > Q::ZERO, f.pinned[a] > f.pinned[b]);
            assert_eq!(f.push[face], f.pinned[a] - f.pinned[b]);
        }
        assert_eq!(f.ranking, [1, 3, 2, 0]);
    }

    #[test]
    fn spectrum_and_open_faces() {
        let f = BoxFacts::solve();
        let mut eig: Vec<Q> = f.modes.iter().map(|&(_, l)| l).collect();
        eig.sort();
        assert_eq!(eig, vec![Q::ZERO, Q::int(2), Q::int(2), Q::int(4)]);
        assert_eq!(f.modes[0].0, [Q::ONE; 4]);
        assert_eq!(f.open_faces, [2; 4]);
        assert_eq!(f.interior_open_faces, 4);
        assert_eq!(open_neighbours(3, 3, 0, 0), 2);
        assert_eq!(open_neighbours(3, 3, 1, 0), 3);
    }

    #[test]
    fn stopping_early_leaves_minus_the_residual() {
        let f = BoxFacts::solve();
        assert_eq!(f.unfinished, [q(-1, 4), q(1, 4), Q::ZERO, Q::ZERO]);
        assert_eq!(f.unfinished_residual, [q(-1, 4), q(1, 4), q(-1, 4), q(1, 4)]);
        assert_eq!(f.unfinished_div, neg(&f.unfinished_residual));
        // The identity holds for any pressure, not just this one.
        for p in [[Q::ZERO; 4], [q(2, 3), q(-1, 5), q(7, 4), Q::ONE], f.pinned] {
            let r = sub(&f.rhs, &apply(&f.matrix, &p));
            let flow = sub(&f.provisional, &grad(&p));
            assert_eq!(div(&flow), neg(&r));
        }
    }

    #[test]
    fn rationals_stay_in_lowest_terms() {
        assert_eq!(q(6, -8), q(-3, 4));
        assert_eq!(q(6, -8).to_string(), "-3/4");
        assert_eq!(q(-3, 4).tex(), "-\\tfrac{3}{4}");
        assert_eq!(Q::int(-1).tex(), "-1");
        assert_eq!(q(0, 5), Q::ZERO);
        assert!(q(1, 4) < q(1, 2));
    }
}
