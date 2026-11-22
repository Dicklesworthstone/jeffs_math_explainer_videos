//! Chapter 10's eight-cell tube, solved exactly: every bar, badge, graph point, coarse value and readout
//! the chapter draws is computed here from the tube's declared rules (navier-full-script03.md chapter 10,
//! the Conventions of its source navier-multigrid-scene04.md), and checked by `TubeFacts::check` before
//! the first frame. Standard library only, so it needs nothing from the renderer.
//!
//! Model, box units (h = ρ = Δt = 1). Cells i = 1 … 8 in a row, centres x = i − ½; face j lies right of
//! cell j. Face 0 is a solid wall: its flow is fixed and the pressure step never changes it. Face 8 opens
//! into a tank whose pressure is held at 0 and whose centre is at x = 17/2. grad on face j is
//! p(j+1) − p(j) (the tank's p beyond cell 8), u_new = u* − grad p, div of cell i = u(face i) −
//! u(face i − 1), L = −div grad, d = −div u*, r = d − L p, so div u_new = −r. The test sets u* = 0, so the
//! exact pressure is 0 and every bar's size is its cell's error. The guess is p0 = S + Z with the hump
//! S_i = 1 − ((2i − 1)/(2N + 1))² and the zigzag Z_i = (−1)^(i+1)/2. One sweep is damped Jacobi,
//! p_i ← p_i + w r_i / D_i with w = ½ and D_i the number of open neighbours, all from the old values.
//! Restriction sums pairs; the coarse operator is rediscretized from the coarse geometry; prolongation is
//! the straight-line function through the coarse values, flat from the wall to the first coarse centre and
//! reaching 0 at the tank's centre; the correction is added. One round: 3 sweeps, correction, 2 sweeps.
//!
//! Exact states use 128-bit rationals: tube_oracle01 reports that every term of every state the chapter
//! builds fits in 48 bits, so products of two terms cannot overflow. The two long runs (the doubling
//! counts and "sweeps alone") take hundreds of sweeps and are run in binary64; the oracle runs them exactly
//! and reports how far each stopping test is from its threshold (`check.double.margin`,
//! `check.alone.margin`), far beyond binary64's rounding.

use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};

/// An exact rational in lowest terms with a positive denominator.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct R {
    num: i128,
    den: i128,
}

fn gcd(a: i128, b: i128) -> i128 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

fn wide(a: i128, b: i128) -> i128 {
    a.checked_mul(b)
        .expect("tube rational overflow: a state outgrew 128 bits")
}

impl R {
    pub const ZERO: R = R { num: 0, den: 1 };
    pub const ONE: R = R { num: 1, den: 1 };

    pub fn new(num: i128, den: i128) -> R {
        assert!(den != 0, "a rational needs a nonzero denominator");
        let g = gcd(num, den).max(1);
        let sign = if den < 0 { -1 } else { 1 };
        R {
            num: sign * num / g,
            den: sign * den / g,
        }
    }

    pub fn int(n: i128) -> R {
        R { num: n, den: 1 }
    }

    pub fn is_zero(self) -> bool {
        self.num == 0
    }

    pub fn is_negative(self) -> bool {
        self.num < 0
    }

    pub fn abs(self) -> R {
        R {
            num: self.num.abs(),
            den: self.den,
        }
    }

    pub fn to_f64(self) -> f64 {
        self.num as f64 / self.den as f64
    }

    /// Display math: `\tfrac{7}{6}`, `-\tfrac{1}{2}`, `1`, `0`.
    pub fn tex(self) -> String {
        let sign = if self.num < 0 { "-" } else { "" };
        if self.den == 1 {
            format!("{sign}{}", self.num.abs())
        } else {
            format!("{sign}\\tfrac{{{}}}{{{}}}", self.num.abs(), self.den)
        }
    }
}

impl fmt::Display for R {
    /// Plain text: `7/6`, `-1/2`, `1`, `0` (the fact rows and the oracle print this form).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.den == 1 {
            write!(f, "{}", self.num)
        } else {
            write!(f, "{}/{}", self.num, self.den)
        }
    }
}

impl Add for R {
    type Output = R;
    fn add(self, o: R) -> R {
        let num = wide(self.num, o.den)
            .checked_add(wide(o.num, self.den))
            .expect("tube rational overflow");
        R::new(num, wide(self.den, o.den))
    }
}

impl Sub for R {
    type Output = R;
    fn sub(self, o: R) -> R {
        self + -o
    }
}

impl Mul for R {
    type Output = R;
    fn mul(self, o: R) -> R {
        let (g1, g2) = (gcd(self.num, o.den).max(1), gcd(o.num, self.den).max(1));
        R::new(wide(self.num / g1, o.num / g2), wide(self.den / g2, o.den / g1))
    }
}

impl Div for R {
    type Output = R;
    fn div(self, o: R) -> R {
        assert!(!o.is_zero(), "division by an exact zero");
        self * R::new(o.den, o.num)
    }
}

impl Neg for R {
    type Output = R;
    fn neg(self) -> R {
        R {
            num: -self.num,
            den: self.den,
        }
    }
}

impl PartialOrd for R {
    fn partial_cmp(&self, o: &R) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

impl Ord for R {
    fn cmp(&self, o: &R) -> Ordering {
        wide(self.num, o.den).cmp(&wide(o.num, self.den))
    }
}

/// Cells in the tube.
pub const N: usize = 8;
/// Coarse cells: each holds two fine cells.
pub const M: usize = N / 2;
/// The sweep weight, as (numerator, denominator): halfway toward the neighbours' average.
pub const WEIGHT: (i128, i128) = (1, 2);
/// The tank's held pressure.
pub const TANK_PRESSURE: i128 = 0;
/// One round: sweeps before the coarse correction and after it.
pub const PRE: usize = 3;
pub const POST: usize = 2;
/// Sweeps drawn in the side-panel graph.
pub const GRAPH_SWEEPS: usize = 20;
/// The three tubes of the doubling test.
pub const DOUBLING: [usize; 3] = [8, 16, 32];
/// Picture samples of each doubling run (the counter and the shrinking hump step through these).
pub const HALVING_SAMPLES: usize = 24;

pub fn weight() -> R {
    R::new(WEIGHT.0, WEIGHT.1)
}

pub fn tank() -> R {
    R::int(TANK_PRESSURE)
}

/// The tank's centre in fine units: half a cell beyond the last face.
pub fn tank_x(n: usize) -> R {
    R::new(2 * n as i128 + 1, 2)
}

pub type V = Vec<R>;

/// Open neighbours of each cell: the wall is no neighbour, the tank is one.
pub fn diag(n: usize) -> V {
    (0..n).map(|i| R::int(if i > 0 { 2 } else { 1 })).collect()
}

/// grad on faces 1 … n (the tank's pressure beyond the last cell).
pub fn grad(p: &[R]) -> V {
    (0..p.len())
        .map(|j| p.get(j + 1).copied().unwrap_or_else(tank) - p[j])
        .collect()
}

/// u_new on faces 0 … n with u* = 0: nothing through the wall face, −grad p elsewhere.
pub fn flows(p: &[R]) -> V {
    std::iter::once(R::ZERO)
        .chain(grad(p).into_iter().map(|g| -g))
        .collect()
}

/// div of each cell from its face values (faces 0 … n).
pub fn div(u: &[R]) -> V {
    u.windows(2).map(|w| w[1] - w[0]).collect()
}

/// L p = −div grad p, with the wall face carrying nothing.
pub fn apply_l(p: &[R]) -> V {
    let g: V = std::iter::once(R::ZERO).chain(grad(p)).collect();
    div(&g).into_iter().map(|x| -x).collect()
}

/// r = d − L p, with d = 0 in this test.
pub fn residual(p: &[R]) -> V {
    apply_l(p).into_iter().map(|x| -x).collect()
}

pub fn sweep(p: &[R], w: R) -> V {
    let (r, d) = (residual(p), diag(p.len()));
    (0..p.len()).map(|i| p[i] + w * r[i] / d[i]).collect()
}

pub fn sweeps(p: &[R], k: usize, w: R) -> V {
    let mut p = p.to_vec();
    for _ in 0..k {
        p = sweep(&p, w);
    }
    p
}

pub fn largest(p: &[R]) -> R {
    p.iter().map(|x| x.abs()).max().unwrap_or(R::ZERO)
}

pub fn neg(p: &[R]) -> V {
    p.iter().map(|&x| -x).collect()
}

pub fn plus(a: &[R], b: &[R]) -> V {
    a.iter().zip(b).map(|(&x, &y)| x + y).collect()
}

/// Coarse cell I's sum of its two fine values.
pub fn restrict(r: &[R]) -> V {
    (0..r.len() / 2).map(|i| r[2 * i] + r[2 * i + 1]).collect()
}

/// The coarse centres X_I = 2I − 1, in fine units.
pub fn coarse_centres() -> V {
    (1..=M).map(|i| R::int(2 * i as i128 - 1)).collect()
}

/// The coarse operator, rediscretized: each coarse face carries the pressure difference over the
/// distance between the centres it joins, the wall carries nothing, and the last cell's outlet term is
/// v_4 over its distance to the tank's centre.
pub fn coarse_rows() -> [[R; M]; M] {
    let x = coarse_centres();
    let mut rows = [[R::ZERO; M]; M];
    for i in 0..M {
        if i > 0 {
            let k = R::ONE / (x[i] - x[i - 1]);
            rows[i][i] = rows[i][i] + k;
            rows[i][i - 1] = rows[i][i - 1] - k;
        }
        if i + 1 < M {
            let k = R::ONE / (x[i + 1] - x[i]);
            rows[i][i] = rows[i][i] + k;
            rows[i][i + 1] = rows[i][i + 1] - k;
        } else {
            rows[i][i] = rows[i][i] + R::ONE / (tank_x(N) - x[i]);
        }
    }
    rows
}

/// Solve `m v = b` exactly (Gauss–Jordan with row swaps).
pub fn solve(m: &[[R; M]; M], b: &[R]) -> V {
    let mut a: Vec<Vec<R>> = (0..M).map(|i| m[i].iter().copied().chain([b[i]]).collect()).collect();
    for c in 0..M {
        let pivot = (c..M)
            .find(|&r| !a[r][c].is_zero())
            .expect("the coarse operator is nonsingular");
        a.swap(c, pivot);
        let lead = a[c][c];
        a[c] = a[c].iter().map(|&x| x / lead).collect();
        for r in 0..M {
            if r != c && !a[r][c].is_zero() {
                let f = a[r][c];
                let row_c = a[c].clone();
                a[r] = a[r].iter().zip(&row_c).map(|(&x, &y)| x - f * y).collect();
            }
        }
    }
    a.iter().map(|row| row[M]).collect()
}

/// The straight-line correction at fine position `x`: flat left of X_1, through every (X_I, v_I), to
/// the tank's pressure at its centre.
pub fn line_through(v: &[R], x: R) -> R {
    let mut xs = coarse_centres();
    xs.push(tank_x(N));
    let mut ys = v.to_vec();
    ys.push(tank());
    if x <= xs[0] {
        return ys[0];
    }
    for k in 0..xs.len() - 1 {
        if xs[k] <= x && x <= xs[k + 1] {
            let t = (x - xs[k]) / (xs[k + 1] - xs[k]);
            return ys[k] + t * (ys[k + 1] - ys[k]);
        }
    }
    panic!("the correction is read inside the tube");
}

/// The coarse values brought back to the fine centres x = i − ½.
pub fn prolong(v: &[R]) -> V {
    (1..=N).map(|i| line_through(v, R::new(2 * i as i128 - 1, 2))).collect()
}

/// The script's interpolation rule, written out: child 2I−1 = ¾ v_I + ¼ v_(I−1), child 2I = ¾ v_I +
/// ¼ v_(I+1); child 1 = v_1 at the wall and child 8 = ⅔ v_4 at the tank. `check` compares it with
/// `prolong`.
pub fn prolong_rule(v: &[R]) -> V {
    let (q, t) = (R::new(1, 4), R::new(3, 4));
    let mut out = Vec::with_capacity(N);
    for i in 0..M {
        out.push(if i > 0 { t * v[i] + q * v[i - 1] } else { v[0] });
        out.push(if i + 1 < M {
            t * v[i] + q * v[i + 1]
        } else {
            R::new(2, 3) * v[i]
        });
    }
    out
}

/// One two-grid round from `p`, every stage kept for the pictures.
#[derive(Clone, Debug)]
pub struct Round {
    pub before: V,
    pub residual: V,
    pub coarse_rhs: V,
    pub coarse: V,
    pub correction: V,
    pub mid: V,
    pub end: V,
}

pub fn round(p: &[R], pre: usize, post: usize) -> Round {
    let w = weight();
    let before = sweeps(p, pre, w);
    let residual = residual(&before);
    let coarse_rhs = restrict(&residual);
    let coarse = solve(&coarse_rows(), &coarse_rhs);
    let correction = prolong(&coarse);
    let mid = plus(&before, &correction);
    let end = sweeps(&mid, post, w);
    Round {
        before,
        residual,
        coarse_rhs,
        coarse,
        correction,
        mid,
        end,
    }
}

// ---------- the long runs, in binary64 ----------

fn sweep_f(p: &[f64], w: f64) -> Vec<f64> {
    let n = p.len();
    let tank = TANK_PRESSURE as f64;
    (0..n)
        .map(|i| {
            let left = if i > 0 { p[i - 1] - p[i] } else { 0.0 };
            let right = if i + 1 < n { p[i + 1] } else { tank } - p[i];
            let open = if i > 0 { 2.0 } else { 1.0 };
            p[i] + w * (left + right) / open
        })
        .collect()
}

fn largest_f(p: &[f64]) -> f64 {
    p.iter().fold(0.0, |m, x| m.max(x.abs()))
}

/// The hump on an `n`-cell tube of the same rule, 1 − ((2i − 1)/(2n + 1))².
pub fn hump(n: usize) -> V {
    (1..=n)
        .map(|i| R::ONE - R::new(2 * i as i128 - 1, 2 * n as i128 + 1) * R::new(2 * i as i128 - 1, 2 * n as i128 + 1))
        .collect()
}

/// Sweeps until the largest error first reaches half its start, with picture samples: the sweep count
/// and the state at `HALVING_SAMPLES + 1` evenly spaced counts from 0 to the total.
#[derive(Clone, Debug)]
pub struct Halving {
    pub cells: usize,
    pub start: Vec<f64>,
    pub count: usize,
    pub samples: Vec<(usize, Vec<f64>)>,
}

pub fn halving(n: usize) -> Halving {
    let w = weight().to_f64();
    let start: Vec<f64> = hump(n).iter().map(|x| x.to_f64()).collect();
    let half = largest_f(&start) / 2.0;
    let mut states = vec![start.clone()];
    while largest_f(states.last().expect("a start")) > half {
        let next = sweep_f(states.last().expect("a start"), w);
        states.push(next);
    }
    let count = states.len() - 1;
    let samples = (0..=HALVING_SAMPLES)
        .map(|j| {
            let k = (count * j + HALVING_SAMPLES / 2) / HALVING_SAMPLES;
            (k, states[k].clone())
        })
        .collect();
    Halving {
        cells: n,
        start,
        count,
        samples,
    }
}

/// Sweeps alone, from `p`, until the largest error first reaches `target`.
pub fn sweeps_to_reach(p: &[R], target: R) -> usize {
    let w = weight().to_f64();
    let target = target.to_f64();
    let mut q: Vec<f64> = p.iter().map(|x| x.to_f64()).collect();
    let mut k = 0;
    while largest_f(&q) > target {
        q = sweep_f(&q, w);
        k += 1;
    }
    k
}

/// Round an exact value to `digits` decimals for the screen, with a typographic minus.
pub fn decimals(x: R, digits: usize) -> String {
    let s = format!("{:.*}", digits, x.to_f64());
    let s = if s.trim_start_matches('-').chars().all(|c| c == '0' || c == '.') {
        s.trim_start_matches('-').to_string()
    } else {
        s
    };
    s.replace('-', "\u{2212}")
}

fn list(v: &[R]) -> String {
    v.iter().map(R::to_string).collect::<Vec<_>>().join(", ")
}

fn truth(b: bool) -> String {
    b.to_string()
}

/// Everything chapter 10 shows, from the rules above.
#[derive(Clone, Debug)]
pub struct TubeFacts {
    pub diag: V,
    pub det: R,
    pub rows: Vec<V>,
    pub truth: V,
    pub hump: V,
    pub zigzag: V,
    pub guess: V,
    /// The zigzag after one sweep, and after one and two full jumps (weight 1).
    pub zig_sweep: V,
    pub full: [V; 2],
    /// The hump after one sweep.
    pub hump_sweep: V,
    /// The guess after 0 … GRAPH_SWEEPS sweeps.
    pub history: Vec<V>,
    pub errors: V,
    pub residuals: V,
    pub halvings: Vec<Halving>,
    pub coarse_rows: [[R; M]; M],
    pub one: Round,
    pub two: Round,
    pub skip: Round,
    pub alone: usize,
    pub zig_pairs: V,
    /// Flow on faces 1 … 8 after three sweeps.
    pub flow3: V,
}

/// After two full jumps, the zigzag's cells 1 to 6 are back exactly and the two beside the tank are not
/// (script 10_9).
fn back_exact(after: &[R], zigzag: &[R]) -> bool {
    let near_tank = N - 2;
    after[..near_tank] == zigzag[..near_tank]
        && after[near_tank..].iter().zip(&zigzag[near_tank..]).all(|(a, b)| a != b)
}

fn det(rows: &[V]) -> R {
    let mut a = rows.to_vec();
    let n = a.len();
    let mut out = R::ONE;
    for c in 0..n {
        let Some(pivot) = (c..n).find(|&r| !a[r][c].is_zero()) else {
            return R::ZERO;
        };
        if pivot != c {
            a.swap(c, pivot);
            out = -out;
        }
        out = out * a[c][c];
        for r in c + 1..n {
            let f = a[r][c] / a[c][c];
            let row_c = a[c].clone();
            a[r] = a[r].iter().zip(&row_c).map(|(&x, &y)| x - f * y).collect();
        }
    }
    out
}

impl TubeFacts {
    pub fn solve() -> TubeFacts {
        let w = weight();
        let hump = hump(N);
        let zigzag: V = (1..=N).map(|i| R::new(if i % 2 == 1 { 1 } else { -1 }, 2)).collect();
        let guess = plus(&hump, &zigzag);
        let rows: Vec<V> = (0..N)
            .map(|i| {
                (0..N)
                    .map(|j| {
                        let unit: V = (0..N).map(|k| if k == j { R::ONE } else { R::ZERO }).collect();
                        apply_l(&unit)[i]
                    })
                    .collect()
            })
            .collect();
        // Every row of L sums its cell's open faces; with u* = 0 the right side is zero, so the exact
        // answer is the solution of L p = 0, which a nonzero determinant makes p = 0.
        let truth = vec![R::ZERO; N];
        let mut history = vec![guess.clone()];
        for _ in 0..GRAPH_SWEEPS {
            let next = sweep(history.last().expect("the guess"), w);
            history.push(next);
        }
        let errors = history.iter().map(|p| largest(p)).collect();
        let residuals = history.iter().map(|p| largest(&residual(p))).collect();
        let one = round(&guess, PRE, POST);
        let two = round(&one.end, PRE, POST);
        let skip = round(&guess, 0, 0);
        let alone = sweeps_to_reach(&guess, largest(&one.end));
        TubeFacts {
            diag: diag(N),
            det: det(&rows),
            rows,
            truth,
            zig_sweep: sweep(&zigzag, w),
            full: [sweep(&zigzag, R::ONE), sweeps(&zigzag, 2, R::ONE)],
            hump_sweep: sweep(&hump, w),
            hump,
            zig_pairs: restrict(&residual(&zigzag)),
            zigzag,
            flow3: flows(&history[PRE])[1..].to_vec(),
            guess,
            history,
            errors,
            residuals,
            halvings: DOUBLING.iter().map(|&n| halving(n)).collect(),
            coarse_rows: coarse_rows(),
            one,
            two,
            skip,
            alone,
        }
    }

    /// The fine imbalance badges of a state: div u_new = −r.
    pub fn badges(p: &[R]) -> V {
        neg(&residual(p))
    }

    /// The hump's residual, the same in every cell.
    pub fn hump_residual(&self) -> R {
        residual(&self.hump)[0]
    }

    /// The face with the largest flow after three sweeps (1-based, face 8 the tank's).
    pub fn longest_face(&self) -> usize {
        let top = largest(&self.flow3);
        1 + self.flow3.iter().position(|x| x.abs() == top).expect("a largest face")
    }

    /// The doubling ratio, rounded ("about × 4").
    pub fn doubling_ratio(&self) -> i128 {
        let h = &self.halvings;
        let r = R::new(h[1].count as i128, h[0].count as i128);
        (r.to_f64()).round() as i128
    }

    /// Running sums of the badges from the wall: face j's total.
    pub fn running_badges(&self) -> V {
        let mut acc = R::ZERO;
        Self::badges(&self.history[PRE])
            .into_iter()
            .map(|b| {
                acc = acc + b;
                acc
            })
            .collect()
    }

    /// Every claim the chapter's pictures and words rest on; an error names the first one that fails.
    pub fn check(&self) -> Result<(), String> {
        let need = |ok: bool, what: &str| {
            if ok {
                Ok(())
            } else {
                Err(format!("tube check failed: {what}"))
            }
        };
        let w = weight();
        need(
            self.diag.iter().enumerate().all(|(i, &d)| d == self.rows[i][i]),
            "D is L's diagonal",
        )?;
        need(!self.det.is_zero(), "L is nonsingular: no free constant")?;
        need(
            (0..N).all(|i| (0..N).all(|j| self.rows[i][j] == self.rows[j][i])),
            "L is symmetric",
        )?;
        let probe: V = (0..N as i128).map(|k| R::new(k * k - 3 * k + 1, 5)).collect();
        need(div(&flows(&probe)) == Self::badges(&probe), "div u_new = −r")?;
        need(
            apply_l(&self.truth).iter().all(|x| x.is_zero()),
            "the right answer solves L p = 0",
        )?;
        need(
            self.truth.iter().all(|x| *x == tank()),
            "the right answer is the tank's pressure everywhere",
        )?;
        let z1 = &self.zig_sweep;
        need(
            z1[..N - 1].iter().all(|x| x.is_zero()),
            "one sweep erases the zigzag in cells 1 to 7",
        )?;
        need(
            z1[N - 1].is_negative(),
            "the eighth beside the tank stays below the line",
        )?;
        let [f1, f2] = &self.full;
        need(
            f1[..N - 1].iter().zip(&self.zigzag).all(|(&a, &b)| a == -b),
            "a full jump turns cells 1 to 7 into their opposites",
        )?;
        need(
            f1.iter()
                .zip(&self.zigzag)
                .all(|(a, b)| a.is_negative() != b.is_negative()),
            "a full jump moves every bar to the other side",
        )?;
        need(
            back_exact(f2, &self.zigzag),
            "a second full jump returns cells 1 to 6 exactly",
        )?;
        need(
            f2.iter()
                .zip(&self.zigzag)
                .all(|(a, b)| a.is_negative() == b.is_negative()),
            "a second full jump puts every bar back on its side",
        )?;
        let moves: V = self.hump.iter().zip(&self.hump_sweep).map(|(&a, &b)| a - b).collect();
        need(
            moves[1..].iter().all(|&m| m == moves[1]),
            "the hump's middle cells all move the same",
        )?;
        let r = residual(&self.hump);
        need(
            r.iter().all(|&x| x == r[0]) && r[0].is_negative(),
            "the hump's residual is one negative value",
        )?;
        need(
            sweep(&self.guess, w) == plus(&sweep(&self.hump, w), z1),
            "one sweep of the guess is the hump's sweep plus the zigzag's",
        )?;
        need(
            self.history[1][N - 1] == self.hump_sweep[N - 1] + z1[N - 1],
            "cell 8 after one sweep",
        )?;
        need(
            self.errors[1] < self.errors[0] && self.errors[GRAPH_SWEEPS] < self.errors[1],
            "the error falls",
        )?;
        let h = &self.halvings;
        need(
            h.len() == DOUBLING.len() && h.windows(2).all(|p| p[1].count > p[0].count),
            "finer tubes need more sweeps",
        )?;
        let ratio2 = R::new(h[2].count as i128, h[1].count as i128).to_f64().round() as i128;
        need(
            self.doubling_ratio() == ratio2,
            "both doublings round to the same ratio",
        )?;
        let one = &self.one;
        need(
            one.before == self.history[PRE],
            "the round starts from the three-sweep state",
        )?;
        need(
            one.residual == residual(&one.before) && restrict(&one.residual) == one.coarse_rhs,
            "the first coarse right side is the restricted fine residual",
        )?;
        need(
            restrict(&Self::badges(&one.before)) == neg(&one.coarse_rhs),
            "the coarse badges are the coarse right side negated",
        )?;
        need(
            one.coarse.iter().all(|x| x.is_negative()),
            "every coarse value is below zero",
        )?;
        need(
            one.correction.iter().all(|x| x.is_negative()),
            "the correction is below zero",
        )?;
        need(
            one.correction.windows(2).all(|p| p[0] < p[1]),
            "the correction rises toward the tank",
        )?;
        need(
            one.correction == prolong_rule(&one.coarse),
            "the straight lines match the written rule",
        )?;
        need(
            one.mid.iter().zip(&one.before).all(|(m, b)| m.abs() < b.abs()),
            "the correction moves every bar toward the line",
        )?;
        need(
            largest(&one.mid) < largest(&one.before),
            "the correction cuts the error",
        )?;
        need(
            largest(&self.two.end) < largest(&one.end),
            "the second round cuts it again",
        )?;
        need(
            largest(&self.skip.mid) > largest(&self.guess),
            "skipping the sweeps makes the error grow",
        )?;
        let zp = &self.zig_pairs;
        need(
            zp[1].is_zero() && zp[2].is_zero(),
            "the middle pairs' zigzag residuals cancel",
        )?;
        need(
            !zp[0].is_zero() && !zp[M - 1].is_zero(),
            "the end pairs' zigzag residuals do not",
        )?;
        need(
            self.running_badges() == self.flow3,
            "each face carries the badges between it and the wall",
        )?;
        need(
            self.alone > GRAPH_SWEEPS,
            "sweeps alone take longer than the graph shows",
        )?;
        Ok(())
    }

    /// The fact rows, in the order tube_oracle01 prints them (its `check.*` rows excepted).
    pub fn fact_rows(&self) -> Vec<(String, String)> {
        let w = weight();
        let z = &self.zigzag;
        let [f1, f2] = &self.full;
        let z1 = &self.zig_sweep;
        let moves: V = self.hump.iter().zip(&self.hump_sweep).map(|(&a, &b)| a - b).collect();
        let r_hump = self.hump_residual();
        let h = &self.halvings;
        let one = &self.one;
        let badges3 = Self::badges(&one.before);
        let coarse_badges = restrict(&badges3);
        let rows_text = self
            .coarse_rows
            .iter()
            .map(|row| row.iter().map(R::to_string).collect::<Vec<_>>().join(" "))
            .collect::<Vec<_>>()
            .join("; ");
        let flipped: V = prolong(&neg(&one.coarse));
        let ratio = |a: usize, b: usize| R::new(h[a].count as i128, h[b].count as i128);
        let signed_list = |v: &[R], digits: usize, plus: bool| {
            v.iter()
                .map(|&x| {
                    let s = decimals(x, digits);
                    if plus && x > R::ZERO { format!("+{s}") } else { s }
                })
                .collect::<Vec<_>>()
                .join(", ")
        };
        let mut rows: Vec<(String, String)> = Vec::new();
        let mut put = |k: &str, v: String| rows.push((k.to_string(), v));
        put("tube.cells", N.to_string());
        put("tube.diag", list(&self.diag));
        put(
            "tube.diag.is.matrix.diagonal",
            truth(self.diag.iter().enumerate().all(|(i, &d)| d == self.rows[i][i])),
        );
        put("tube.det", self.det.to_string());
        put("tube.no.free.constant", truth(!self.det.is_zero()));
        put(
            "tube.symmetric",
            truth((0..N).all(|i| (0..N).all(|j| self.rows[i][j] == self.rows[j][i]))),
        );
        let probe: V = (0..N as i128).map(|k| R::new(k * k - 3 * k + 1, 5)).collect();
        put("tube.identity", truth(div(&flows(&probe)) == Self::badges(&probe)));
        put("tube.truth", list(&self.truth));
        put("tube.guess", list(&self.guess));
        put("tube.dome", list(&self.hump));
        put("tube.zigzag", list(z));
        put("tube.zigzag.size", largest(z).to_string());
        put("tube.guess.err", largest(&self.guess).to_string());
        put("ch10.show.wall", flows(&self.guess)[0].to_string());
        put("ch10.show.tank", format!("p = {}", tank()));
        put("ch10.show.truth", self.truth[0].to_string());
        put(
            "ch10.show.truth.everywhere",
            truth(self.truth.iter().all(|&t| t == self.truth[0])),
        );
        put("ch10.show.weight", w.to_string());
        put("zig.sweep1", list(z1));
        put(
            "zig.sweep1.interior.zero",
            truth(z1[..N - 1].iter().all(|x| x.is_zero())),
        );
        put("zig.sweep1.last", z1[N - 1].abs().to_string());
        put("ch10.show.zig.last", z1[N - 1].abs().to_string());
        put("zig.full.sweep1", list(f1));
        put(
            "zig.full.flips",
            truth(f1[1..N - 1].iter().zip(&z[1..N - 1]).all(|(&a, &b)| a == -b)),
        );
        put("fix.full.sweep1", list(f1));
        put(
            "fix.full.cells1to7.opposite",
            truth(f1[..N - 1].iter().zip(z).all(|(&a, &b)| a == -b)),
        );
        put("fix.full.cell8", format!("{} -> {}", z[N - 1], f1[N - 1]));
        put(
            "fix.full.all.cross",
            truth(f1.iter().zip(z).all(|(a, b)| a.is_negative() != b.is_negative())),
        );
        put("fix.full.sweep2", list(f2));
        put("fix.full.back.exact", truth(back_exact(f2, z)));
        put(
            "fix.full.back.side",
            truth(f2.iter().zip(z).all(|(a, b)| a.is_negative() == b.is_negative())),
        );
        put(
            "dome.move.interior.same",
            truth(moves[1..].iter().all(|&m| m == moves[1])),
        );
        put("dome.move.interior", moves[1].to_string());
        put("dome.err", largest(&self.hump).to_string());
        put("dome.resid", r_hump.abs().to_string());
        put("ch10.show.dome.move", decimals(moves[1], 3));
        put("ch10.show.dome.err", decimals(largest(&self.hump), 3));
        put("fix.dome.r.signed", r_hump.to_string());
        put(
            "fix.dome.r.same",
            truth(residual(&self.hump).iter().all(|&x| x == r_hump)),
        );
        put("fix.dome.badge", (-r_hump).to_string());
        put("fix.dome.moves", list(&neg(&moves)));
        put("ch10.show.dome.badge", decimals(-r_hump, 2));
        put("mix.err.sweep1", self.errors[1].to_string());
        put(&format!("mix.err.sweep{PRE}"), self.errors[PRE].to_string());
        put(
            &format!("mix.err.sweep{GRAPH_SWEEPS}"),
            self.errors[GRAPH_SWEEPS].to_string(),
        );
        for k in [0, 1, PRE, GRAPH_SWEEPS] {
            put(&format!("mix.resid.sweep{k}"), self.residuals[k].to_string());
        }
        put(
            "mix.err.curve",
            self.errors
                .iter()
                .map(|&e| decimals(e, 3))
                .collect::<Vec<_>>()
                .join(", "),
        );
        put(
            "mix.resid.curve",
            self.residuals
                .iter()
                .map(|&e| decimals(e, 3))
                .collect::<Vec<_>>()
                .join(", "),
        );
        put(
            "fix.sweep1.linear",
            truth(sweep(&self.guess, w) == plus(&sweep(&self.hump, w), z1)),
        );
        put("fix.sweep1.zigzag.left", list(z1));
        put("fix.sweep1.cell8", self.history[1][N - 1].to_string());
        put(
            "fix.sweep1.cell8.parts",
            truth(self.history[1][N - 1] == self.hump_sweep[N - 1] + z1[N - 1]),
        );
        put("fix.sweep1.err", self.errors[1].to_string());
        put("ch10.show.sweep1.cell8", self.sweep1_cell8());
        put("ch10.show.sweep1.cell8.zigzag.negative", truth(z1[N - 1].is_negative()));
        for (k, &n) in DOUBLING.iter().enumerate() {
            put(&format!("double.n{n}"), h[k].count.to_string());
        }
        put("double.ratio1", ratio(1, 0).to_string());
        put("double.ratio2", ratio(2, 1).to_string());
        put("ch10.show.double.ratio", self.doubling_ratio().to_string());
        put(
            "ch10.show.double.ratio.both",
            truth(self.doubling_ratio() == ratio(2, 1).to_f64().round() as i128),
        );
        put("cycle.pre.err", self.errors[PRE].to_string());
        put("cycle.before.is.sweep3", truth(one.before == self.history[PRE]));
        put("cycle.restricted", list(&one.coarse_rhs));
        put("coarse.rows", rows_text);
        put("cycle.coarse", list(&one.coarse));
        put("cycle.correction", list(&one.correction));
        put("cycle.mid.err", largest(&one.mid).to_string());
        put("cycle.mid.resid", largest(&residual(&one.mid)).to_string());
        put("cycle.end.err", largest(&one.end).to_string());
        put("cycle.end.resid", largest(&residual(&one.end)).to_string());
        put("cycle2.end.err", largest(&self.two.end).to_string());
        put("cycle2.end.resid", largest(&residual(&self.two.end)).to_string());
        put("smooth.only.match", self.alone.to_string());
        put("fix.coarse.badges", list(&coarse_badges));
        put(
            "fix.coarse.badges.are.minus.R",
            truth(coarse_badges == neg(&one.coarse_rhs)),
        );
        put("fix.coarse.rhs", list(&one.coarse_rhs));
        put("fix.coarse.v", list(&one.coarse));
        put(
            "fix.coarse.v.negative",
            truth(one.coarse.iter().all(|x| x.is_negative())),
        );
        put("fix.correction.negative", list(&one.correction));
        put(
            "fix.correction.all.negative",
            truth(one.correction.iter().all(|x| x.is_negative())),
        );
        put(
            "fix.correction.rises",
            truth(one.correction.windows(2).all(|p| p[0] < p[1])),
        );
        put("fix.correction.flat.wall", truth(one.correction[0] == one.coarse[0]));
        put(
            "fix.correction.zero.at.tank",
            truth(one.correction[N - 1] == R::new(2, 3) * one.coarse[M - 1]),
        );
        put(
            "fix.correction.matches.rule",
            truth(one.correction == prolong_rule(&one.coarse)),
        );
        put("fix.add.v.err", largest(&one.mid).to_string());
        put("fix.add.w.err", largest(&plus(&one.before, &flipped)).to_string());
        put("fix.pre.err", self.errors[PRE].to_string());
        put(
            "fix.bars.drop",
            truth(one.mid.iter().zip(&one.before).all(|(m, b)| m.abs() < b.abs())),
        );
        put("ch10.show.coarse.badges", signed_list(&coarse_badges, 3, false));
        put("ch10.show.coarse.rhs", signed_list(&one.coarse_rhs, 3, true));
        put("ch10.show.coarse.v", signed_list(&one.coarse, 2, false));
        put("ch10.show.mid.err", decimals(largest(&one.mid), 2));
        put("ch10.show.alone", self.alone.to_string());
        put("ch10.show.round", format!("{PRE} + {POST}"));
        put("ch10.show.round2.err", decimals(largest(&self.two.end), 4));
        put("ch10.pic.sweep3.badges", list(&badges3));
        put("ch10.pic.cycle.mid", list(&one.mid));
        put("ch10.pic.cycle.end", list(&one.end));
        put("ch10.pic.cycle2.before", list(&self.two.before));
        put("ch10.pic.cycle2.mid", list(&self.two.mid));
        put("ch10.pic.cycle2.end", list(&self.two.end));
        put("skip.err", largest(&self.skip.mid).to_string());
        put("skip.restricted", list(&self.skip.coarse_rhs));
        put("ch10.pic.skip.mid", list(&self.skip.mid));
        put("skip.zigzag.pairs", list(&self.zig_pairs));
        put(
            "skip.zigzag.middle.cancel",
            truth(self.zig_pairs[1].is_zero() && self.zig_pairs[2].is_zero()),
        );
        put(
            "skip.zigzag.ends.not",
            truth(!self.zig_pairs[0].is_zero() && !self.zig_pairs[M - 1].is_zero()),
        );
        put("ch10.show.skip.err", decimals(largest(&self.skip.mid), 2));
        put(
            "ch10.show.skip.coarse.badges",
            signed_list(&neg(&self.skip.coarse_rhs), 2, false),
        );
        put("flow.sweep3", list(&self.flow3));
        put("flow.sweep3.max", largest(&self.flow3).to_string());
        put("flow.running.sum", truth(self.running_badges() == self.flow3));
        put("ch10.pic.flow.longest.face", self.longest_face().to_string());
        rows
    }

    /// The 10_12 readout at cell 8 after one sweep: the value, the hump's part, the zigzag's eighth.
    pub fn sweep1_cell8(&self) -> String {
        let last = N - 1;
        format!(
            "{} = {} \u{2212} {}",
            decimals(self.history[1][last], 4),
            decimals(self.hump_sweep[last], 4),
            decimals(self.zig_sweep[last].abs(), 3)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tube_passes_its_own_check() {
        let f = TubeFacts::solve();
        assert_eq!(f.check(), Ok(()));
        let rows = f.fact_rows();
        let keys: std::collections::HashSet<&str> = rows.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys.len(), rows.len(), "every fact key once");
    }

    #[test]
    fn rationals_reduce_and_order() {
        assert_eq!(R::new(2, -4), R::new(-1, 2));
        assert!(R::new(1, 3) < R::new(1, 2));
        assert_eq!((R::new(1, 2) + R::new(1, 3)).to_string(), "5/6");
        assert_eq!(R::new(-7, 6).tex(), "-\\tfrac{7}{6}");
        assert_eq!(decimals(R::new(-1, 1000), 2), "0.00");
    }
}
