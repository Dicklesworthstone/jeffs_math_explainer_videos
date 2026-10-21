//! The mathematics of the resistor-network video as pure functions: a finite
//! network of positive conductances with two ends held fixed, solved two
//! independent ways.
//!
//! - [`Network::kirchhoff`] is the electrical picture: Ohm's law gives each
//!   wire's current, Kirchhoff's current law balances them at every middle
//!   junction (Doyle and Snell, *Random Walks and Electric Networks*,
//!   §1.3.2), and the balance equations are eliminated directly.
//! - [`Network::walk`] is the probabilistic picture: the walker leaves a
//!   junction along a wire with chance proportional to the wire's
//!   conductance (§1.3.1), both ends absorb it, and its chances of ending at
//!   each end come from the expected visits `(I − Q)⁻¹` of that absorbing
//!   chain.
//!
//! Both satisfy the same averaging rule with the same values at the ends,
//! which is why they agree; the averaging rule has exactly one solution when
//! every middle junction is joined by wires to an end ([`Network::cut_off`]
//! is empty), and none is determined otherwise (the middle block's
//! determinant is zero). `facts.rs` prints what the video shows;
//! `oracle.py` recomputes it by counting spanning forests and by Cramer's
//! rule.

use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};

// ---------------------------------------------------------------------------
// exact fractions
// ---------------------------------------------------------------------------

/// An exact fraction, reduced, with a positive denominator.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Q {
    n: i128,
    d: i128,
}

/// Every integer step of the fractions is checked: release builds do not
/// trap overflow, and a wrapped value would be a wrong number on screen,
/// not a failure.
const OVERFLOW: &str = "a fraction overflowed i128";

fn gcd(a: i128, b: i128) -> i128 {
    let (mut a, mut b) = (a.checked_abs().expect(OVERFLOW), b.checked_abs().expect(OVERFLOW));
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

fn times(a: i128, b: i128) -> i128 {
    a.checked_mul(b).expect(OVERFLOW)
}

fn plus(a: i128, b: i128) -> i128 {
    a.checked_add(b).expect(OVERFLOW)
}

fn negated(a: i128) -> i128 {
    a.checked_neg().expect(OVERFLOW)
}

impl Q {
    pub const ZERO: Q = Q { n: 0, d: 1 };
    pub const ONE: Q = Q { n: 1, d: 1 };

    /// # Panics
    /// If `d` is zero, or a step overflows i128.
    pub fn new(n: i128, d: i128) -> Self {
        assert!(d != 0, "a fraction needs a non-zero denominator");
        let g = gcd(n, d).max(1);
        let s = if d < 0 { -1 } else { 1 };
        Q {
            n: times(s, n) / g,
            d: times(s, d) / g,
        }
    }

    pub const fn int(n: i128) -> Self {
        Q { n, d: 1 }
    }

    pub fn numer(self) -> i128 {
        self.n
    }

    pub fn denom(self) -> i128 {
        self.d
    }

    pub fn is_zero(self) -> bool {
        self.n == 0
    }

    pub fn is_positive(self) -> bool {
        self.n > 0
    }

    /// The whole number this is, if it is one.
    pub fn whole(self) -> Option<i128> {
        (self.d == 1).then_some(self.n)
    }

    pub fn f64(self) -> f64 {
        self.n as f64 / self.d as f64
    }
}

/// `n/d`, or `n` when whole: as Python's `str(Fraction)` prints it.
impl fmt::Display for Q {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.d == 1 {
            write!(f, "{}", self.n)
        } else {
            write!(f, "{}/{}", self.n, self.d)
        }
    }
}

impl Add for Q {
    type Output = Q;
    fn add(self, o: Q) -> Q {
        let g = gcd(self.d, o.d);
        Q::new(
            plus(times(self.n, o.d / g), times(o.n, self.d / g)),
            times(self.d / g, o.d),
        )
    }
}

impl Neg for Q {
    type Output = Q;
    fn neg(self) -> Q {
        Q {
            n: negated(self.n),
            d: self.d,
        }
    }
}

impl Sub for Q {
    type Output = Q;
    fn sub(self, o: Q) -> Q {
        self + -o
    }
}

impl Mul for Q {
    type Output = Q;
    fn mul(self, o: Q) -> Q {
        let (g1, g2) = (gcd(self.n, o.d).max(1), gcd(o.n, self.d).max(1));
        Q::new(times(self.n / g1, o.n / g2), times(self.d / g2, o.d / g1))
    }
}

impl Div for Q {
    type Output = Q;
    /// # Panics
    /// On division by zero.
    fn div(self, o: Q) -> Q {
        assert!(o.n != 0, "division by a zero fraction");
        self * Q::new(o.d, o.n)
    }
}

impl Ord for Q {
    fn cmp(&self, o: &Q) -> Ordering {
        times(self.n, o.d).cmp(&times(o.n, self.d))
    }
}

impl PartialOrd for Q {
    fn partial_cmp(&self, o: &Q) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

/// Fractions joined by commas, as the facts print a list.
pub fn csv(values: &[Q]) -> String {
    values.iter().map(Q::to_string).collect::<Vec<_>>().join(",")
}

// ---------------------------------------------------------------------------
// exact linear algebra on small matrices
// ---------------------------------------------------------------------------

/// A square matrix of fractions, row by row.
#[derive(Clone, Debug, PartialEq)]
struct Mat {
    n: usize,
    a: Vec<Q>,
}

impl Mat {
    fn zeros(n: usize) -> Self {
        Mat {
            n,
            a: vec![Q::ZERO; n * n],
        }
    }

    fn identity(n: usize) -> Self {
        let mut m = Mat::zeros(n);
        for i in 0..n {
            m.set(i, i, Q::ONE);
        }
        m
    }

    fn at(&self, i: usize, j: usize) -> Q {
        self.a[i * self.n + j]
    }

    fn set(&mut self, i: usize, j: usize, q: Q) {
        self.a[i * self.n + j] = q;
    }

    fn swap_rows(&mut self, i: usize, j: usize) {
        for c in 0..self.n {
            self.a.swap(i * self.n + c, j * self.n + c);
        }
    }
}

/// `x` with `a x = b`, by Gaussian elimination and back substitution; `None`
/// when `a` is singular. (The electrical picture's solver.)
fn solve(a: &Mat, b: &[Q]) -> Option<Vec<Q>> {
    let n = a.n;
    let mut m = a.clone();
    let mut rhs = b.to_vec();
    for col in 0..n {
        let pivot = (col..n).find(|&r| !m.at(r, col).is_zero())?;
        m.swap_rows(col, pivot);
        rhs.swap(col, pivot);
        let p = m.at(col, col);
        let lead = rhs[col];
        for (r, value) in rhs.iter_mut().enumerate().skip(col + 1) {
            let f = m.at(r, col) / p;
            if f.is_zero() {
                continue;
            }
            for c in col..n {
                let v = m.at(r, c) - f * m.at(col, c);
                m.set(r, c, v);
            }
            *value = *value - f * lead;
        }
    }
    let mut x = vec![Q::ZERO; n];
    for r in (0..n).rev() {
        let s = (r + 1..n).fold(rhs[r], |s, c| s - m.at(r, c) * x[c]);
        x[r] = s / m.at(r, r);
    }
    Some(x)
}

/// `a⁻¹`, by Gauss–Jordan reduction of `[a | I]`; `None` when `a` is
/// singular. (The probabilistic picture's expected visits.)
fn invert(a: &Mat) -> Option<Mat> {
    let n = a.n;
    let mut m = a.clone();
    let mut inv = Mat::identity(n);
    for col in 0..n {
        let pivot = (col..n).find(|&r| !m.at(r, col).is_zero())?;
        m.swap_rows(col, pivot);
        inv.swap_rows(col, pivot);
        let p = m.at(col, col);
        for c in 0..n {
            let (v, w) = (m.at(col, c) / p, inv.at(col, c) / p);
            m.set(col, c, v);
            inv.set(col, c, w);
        }
        for r in 0..n {
            let f = m.at(r, col);
            if r == col || f.is_zero() {
                continue;
            }
            for c in 0..n {
                let (v, w) = (m.at(r, c) - f * m.at(col, c), inv.at(r, c) - f * inv.at(col, c));
                m.set(r, c, v);
                inv.set(r, c, w);
            }
        }
    }
    Some(inv)
}

/// `det a`, the product of the pivots of its elimination (a row swap flips
/// the sign); 1 for the empty matrix.
fn determinant(a: &Mat) -> Q {
    let n = a.n;
    let mut m = a.clone();
    let mut det = Q::ONE;
    for col in 0..n {
        let Some(pivot) = (col..n).find(|&r| !m.at(r, col).is_zero()) else {
            return Q::ZERO;
        };
        if pivot != col {
            m.swap_rows(col, pivot);
            det = -det;
        }
        let p = m.at(col, col);
        det = det * p;
        for r in col + 1..n {
            let f = m.at(r, col) / p;
            for c in col..n {
                let v = m.at(r, c) - f * m.at(col, c);
                m.set(r, c, v);
            }
        }
    }
    det
}

// ---------------------------------------------------------------------------
// networks
// ---------------------------------------------------------------------------

/// One wire: the junctions it joins and its conductance (one over its
/// resistance).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Wire {
    pub a: usize,
    pub b: usize,
    pub conductance: Q,
}

impl Wire {
    /// The junction at the other end from `x`, if the wire touches `x`.
    pub fn far(&self, x: usize) -> Option<usize> {
        if self.a == x {
            Some(self.b)
        } else if self.b == x {
            Some(self.a)
        } else {
            None
        }
    }
}

/// A finite network: `size` junctions, wires of positive conductance, and
/// two ends, the top (held at one volt, the walker's goal) and the bottom
/// (held at zero). Every other junction is a middle junction.
#[derive(Clone, Debug)]
pub struct Network {
    size: usize,
    wires: Vec<Wire>,
    top: usize,
    bottom: usize,
}

/// Where a walker from each junction ends up: its chance of reaching the
/// top first, and of reaching the bottom first.
#[derive(Clone, Debug, PartialEq)]
pub struct Walk {
    pub top: Vec<Q>,
    pub bottom: Vec<Q>,
}

impl Network {
    /// A network, refused when an end or a wire names a junction outside
    /// `0..size`, the two ends are one junction, a wire joins a junction to
    /// itself, or a conductance is not positive (zero is no wire; a negative
    /// one is no resistor).
    pub fn new(size: usize, wires: Vec<Wire>, top: usize, bottom: usize) -> Result<Self, String> {
        if top >= size || bottom >= size {
            return Err(format!("the ends {top} and {bottom} must be junctions of 0..{size}"));
        }
        if top == bottom {
            return Err(format!("the top and the bottom are both junction {top}"));
        }
        for w in &wires {
            if w.a >= size || w.b >= size {
                return Err(format!("a wire joins {} and {}, outside junctions 0..{size}", w.a, w.b));
            }
            if w.a == w.b {
                return Err(format!("a wire joins junction {} to itself", w.a));
            }
            if !w.conductance.is_positive() {
                return Err(format!(
                    "the wire {}-{} has conductance {}: a resistor's conductance is positive",
                    w.a, w.b, w.conductance
                ));
            }
        }
        Ok(Network {
            size,
            wires,
            top,
            bottom,
        })
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn wires(&self) -> &[Wire] {
        &self.wires
    }

    pub fn top(&self) -> usize {
        self.top
    }

    pub fn bottom(&self) -> usize {
        self.bottom
    }

    pub fn is_end(&self, x: usize) -> bool {
        x == self.top || x == self.bottom
    }

    /// The middle junctions, in order.
    pub fn middle(&self) -> Vec<usize> {
        (0..self.size).filter(|&x| !self.is_end(x)).collect()
    }

    /// The conductances of every wire at `x`, added.
    pub fn total(&self, x: usize) -> Q {
        self.wires
            .iter()
            .filter(|w| w.far(x).is_some())
            .fold(Q::ZERO, |s, w| s + w.conductance)
    }

    /// The conductance joining `x` and `y`: every wire between them, added
    /// (wires side by side add their conductances, Doyle and Snell p. 43).
    pub fn between(&self, x: usize, y: usize) -> Q {
        self.wires
            .iter()
            .filter(|w| w.far(x) == Some(y))
            .fold(Q::ZERO, |s, w| s + w.conductance)
    }

    /// The middle junctions no path of wires joins to either end: there the
    /// averaging rule fixes nothing, and the walker never stops.
    pub fn cut_off(&self) -> Vec<usize> {
        let mut reached = vec![false; self.size];
        reached[self.top] = true;
        reached[self.bottom] = true;
        let mut frontier = vec![self.top, self.bottom];
        while let Some(x) = frontier.pop() {
            for y in self.wires.iter().filter_map(|w| w.far(x)) {
                if !reached[y] {
                    reached[y] = true;
                    frontier.push(y);
                }
            }
        }
        self.middle().into_iter().filter(|&x| !reached[x]).collect()
    }

    fn settled(&self) -> Result<(), String> {
        match self.cut_off().first() {
            Some(z) => Err(format!(
                "junction {z} is joined to neither end: its voltage is not determined, and a walker there never stops"
            )),
            None => Ok(()),
        }
    }

    /// The current balance's matrix over the middle junctions: each row says
    /// that the currents leaving junction `x` add to zero, `total(x)` on the
    /// diagonal and minus `between(x, y)` off it.
    fn balance_matrix(&self) -> Mat {
        let middle = self.middle();
        let mut m = Mat::zeros(middle.len());
        for (i, &x) in middle.iter().enumerate() {
            for (j, &y) in middle.iter().enumerate() {
                let entry = if i == j { self.total(x) } else { -self.between(x, y) };
                m.set(i, j, entry);
            }
        }
        m
    }

    /// The determinant of the current balance over the middle junctions:
    /// not zero exactly when the balance has one solution.
    pub fn middle_determinant(&self) -> Q {
        determinant(&self.balance_matrix())
    }

    /// The voltage at every junction with the top held at `top` and the
    /// bottom at `bottom`: Ohm's law and Kirchhoff's current law at every
    /// middle junction, eliminated.
    pub fn kirchhoff(&self, top: Q, bottom: Q) -> Result<Vec<Q>, String> {
        self.settled()?;
        let middle = self.middle();
        let pushed: Vec<Q> = middle
            .iter()
            .map(|&x| self.between(x, self.top) * top + self.between(x, self.bottom) * bottom)
            .collect();
        let inside = solve(&self.balance_matrix(), &pushed).ok_or("the current balance is singular")?;
        let mut v = vec![Q::ZERO; self.size];
        v[self.top] = top;
        v[self.bottom] = bottom;
        for (&x, &vx) in middle.iter().zip(&inside) {
            v[x] = vx;
        }
        Ok(v)
    }

    /// The walker's chance, from `x`, of stepping next to `y`.
    fn step(&self, x: usize, y: usize) -> Q {
        self.between(x, y) / self.total(x)
    }

    /// The walker's chances, from every junction, of reaching the top first
    /// and the bottom first: both ends absorb it, and from the middle the
    /// chance of ending at an end is the expected number of visits to each
    /// middle junction, `(I − Q)⁻¹`, times the chance of stepping from there
    /// straight to that end.
    pub fn walk(&self) -> Result<Walk, String> {
        self.settled()?;
        let middle = self.middle();
        let mut i_minus_q = Mat::identity(middle.len());
        for (i, &x) in middle.iter().enumerate() {
            for (j, &y) in middle.iter().enumerate() {
                let entry = i_minus_q.at(i, j) - self.step(x, y);
                i_minus_q.set(i, j, entry);
            }
        }
        let visits = invert(&i_minus_q).ok_or("the walk's expected visits are infinite")?;
        let mut walk = Walk {
            top: vec![Q::ZERO; self.size],
            bottom: vec![Q::ZERO; self.size],
        };
        walk.top[self.top] = Q::ONE;
        walk.bottom[self.bottom] = Q::ONE;
        for (i, &x) in middle.iter().enumerate() {
            let ending = |end: usize| {
                middle
                    .iter()
                    .enumerate()
                    .fold(Q::ZERO, |s, (j, &y)| s + visits.at(i, j) * self.step(y, end))
            };
            walk.top[x] = ending(self.top);
            walk.bottom[x] = ending(self.bottom);
        }
        Ok(walk)
    }

    /// The average of `values` over the wires at `x`, each far end counted
    /// by its wire's conductance; `None` for a junction with no wire.
    pub fn average(&self, values: &[Q], x: usize) -> Option<Q> {
        let total = self.total(x);
        if total.is_zero() {
            return None;
        }
        let sum = self
            .wires
            .iter()
            .filter_map(|w| w.far(x).map(|y| w.conductance * values[y]))
            .fold(Q::ZERO, |s, q| s + q);
        Some(sum / total)
    }

    /// The far end of every identical wire at `x`, in wire order: a wire of
    /// whole conductance `k` is drawn as `k` identical wires side by side.
    /// Refused for a conductance that is not a whole number.
    pub fn far_ends(&self, x: usize) -> Result<Vec<usize>, String> {
        let mut out = Vec::new();
        for w in &self.wires {
            let Some(y) = w.far(x) else { continue };
            let k = w
                .conductance
                .whole()
                .and_then(|k| usize::try_from(k).ok())
                .ok_or_else(|| {
                    format!(
                        "the wire {}-{} has conductance {}, not a whole number of identical wires",
                        w.a, w.b, w.conductance
                    )
                })?;
            out.extend(std::iter::repeat_n(y, k));
        }
        Ok(out)
    }

    /// The current along each wire, from its `a` to its `b` (Ohm's law: the
    /// conductance times the voltage across it).
    pub fn currents(&self, v: &[Q]) -> Vec<Q> {
        self.wires.iter().map(|w| (v[w.a] - v[w.b]) * w.conductance).collect()
    }

    /// The current leaving junction `x`, every wire's added.
    pub fn net_out(&self, v: &[Q], x: usize) -> Q {
        self.wires
            .iter()
            .filter_map(|w| w.far(x).map(|y| (v[x] - v[y]) * w.conductance))
            .fold(Q::ZERO, |s, q| s + q)
    }

    /// One round of repeated averaging with the ends held (chapter 03): every
    /// middle junction takes the conductance-weighted average of its far
    /// ends' values from the previous round, all at once (Jacobi's method);
    /// the top and the bottom keep theirs. Refused at a junction with no
    /// wire.
    pub fn averaging_round(&self, values: &[Q]) -> Result<Vec<Q>, String> {
        (0..self.size)
            .map(|x| {
                if self.is_end(x) {
                    Ok(values[x])
                } else {
                    self.average(values, x)
                        .ok_or_else(|| format!("junction {x} has no wire to average"))
                }
            })
            .collect()
    }

    /// The same round taken one middle junction at a time, in order, each
    /// using the newest values, its own neighbours' included (Gauss-Seidel).
    pub fn sequential_round(&self, values: &[Q]) -> Result<Vec<Q>, String> {
        let mut out = values.to_vec();
        for x in self.middle() {
            let newest = self
                .average(&out, x)
                .ok_or_else(|| format!("junction {x} has no wire to average"))?;
            out[x] = newest;
        }
        Ok(out)
    }

    /// A round with the ends averaged as well, as if no battery held them
    /// (chapter 03's edge).
    pub fn free_round(&self, values: &[Q]) -> Result<Vec<Q>, String> {
        (0..self.size)
            .map(|x| {
                self.average(values, x)
                    .ok_or_else(|| format!("junction {x} has no wire to average"))
            })
            .collect()
    }

    /// The mean of `values` with each junction counted by its total
    /// conductance. A free round never changes it, but keeping it is not
    /// settling: two junctions joined by one wire swap their values every
    /// free round for ever. The opening network does settle there, being
    /// connected and holding a triangle (the top, the left and the right),
    /// so its walk is aperiodic. Refused for a network with no wire.
    pub fn weighted_mean(&self, values: &[Q]) -> Result<Q, String> {
        let all = (0..self.size).fold(Q::ZERO, |s, x| s + self.total(x));
        if all.is_zero() {
            return Err("a network with no wire has no weighted mean".into());
        }
        Ok((0..self.size).fold(Q::ZERO, |s, x| s + self.total(x) * values[x]) / all)
    }

    /// The voltage drop towards junction `x` along each of its identical
    /// wires, in `far_ends` order: the far end's voltage minus `x`'s.
    /// Positive where current flows in; each wire's current is its drop
    /// over the wires' common resistance (Ohm's law).
    pub fn drops_into(&self, v: &[Q], x: usize) -> Result<Vec<Q>, String> {
        Ok(self.far_ends(x)?.into_iter().map(|y| v[y] - v[x]).collect())
    }

    /// The chance that a walker from middle junction `from` has reached
    /// neither end after each of its first `steps` steps.
    pub fn still_walking(&self, from: usize, steps: usize) -> Result<Vec<Q>, String> {
        self.settled()?;
        let middle = self.middle();
        if !middle.contains(&from) {
            return Err(format!("junction {from} is an end, not a middle junction"));
        }
        let mut at: Vec<Q> = middle
            .iter()
            .map(|&x| if x == from { Q::ONE } else { Q::ZERO })
            .collect();
        let mut out = Vec::with_capacity(steps);
        for _ in 0..steps {
            at = middle
                .iter()
                .map(|&y| {
                    middle
                        .iter()
                        .zip(&at)
                        .fold(Q::ZERO, |s, (&x, &p)| s + p * self.step(x, y))
                })
                .collect();
            out.push(at.iter().fold(Q::ZERO, |s, &p| s + p));
        }
        Ok(out)
    }

    /// Whether the walker can take `route`: it starts at a middle junction,
    /// every step runs along a wire, and it touches an end only at its last
    /// junction.
    pub fn is_route(&self, route: &[usize]) -> bool {
        let Some((&last, before)) = route.split_last() else {
            return false;
        };
        let steps_ok = route
            .windows(2)
            .all(|p| p[0] < self.size && p[1] < self.size && !self.between(p[0], p[1]).is_zero());
        !before.is_empty() && steps_ok && self.is_end(last) && before.iter().all(|&x| !self.is_end(x))
    }

    /// The junctions wired to `x`, each once however many wires join them,
    /// in wire order: the links a school shortcut sees once each pair of
    /// wires side by side is merged (chapter 03).
    pub fn neighbours(&self, x: usize) -> Vec<usize> {
        let mut out: Vec<usize> = Vec::new();
        for y in self.wires.iter().filter_map(|w| w.far(x)) {
            if !out.contains(&y) {
                out.push(y);
            }
        }
        out
    }

    /// Two runs of repeated averaging (chapter 11), the top held at one and
    /// the bottom at zero in both: one from every middle junction at zero,
    /// one from every middle junction at one, each round every middle
    /// junction at once from the round before. The low run's rounds and
    /// the high run's, round 0 first. With positive weights a round keeps
    /// the order of two guesses, so the true values stay between the runs,
    /// and their gap at a junction is the chance that a walker from there is
    /// still walking after that many steps.
    pub fn two_runs(&self, rounds: usize) -> Result<(Vec<Vec<Q>>, Vec<Vec<Q>>), String> {
        let start = |middle: Q| -> Vec<Q> {
            (0..self.size)
                .map(|x| {
                    if x == self.top {
                        Q::ONE
                    } else if x == self.bottom {
                        Q::ZERO
                    } else {
                        middle
                    }
                })
                .collect()
        };
        let (mut low, mut high) = (vec![start(Q::ZERO)], vec![start(Q::ONE)]);
        for _ in 0..rounds {
            let next_low = self.averaging_round(low.last().ok_or("no round to start from")?)?;
            let next_high = self.averaging_round(high.last().ok_or("no round to start from")?)?;
            low.push(next_low);
            high.push(next_high);
        }
        Ok((low, high))
    }

    /// The fewest wires from `from` to `to`, as the junctions along the way
    /// (both included), passing on only through middle junctions: the path
    /// a peak at `from` spreads along to `to` (chapter 04). `None` when no
    /// such path of wires joins them.
    pub fn path(&self, from: usize, to: usize) -> Option<Vec<usize>> {
        if from >= self.size || to >= self.size {
            return None;
        }
        let mut came_from: Vec<Option<usize>> = vec![None; self.size];
        let mut seen = vec![false; self.size];
        seen[from] = true;
        let mut queue = std::collections::VecDeque::from([from]);
        while let Some(x) = queue.pop_front() {
            if x == to {
                let mut out = vec![to];
                let mut at = to;
                while let Some(back) = came_from[at] {
                    out.push(back);
                    at = back;
                }
                out.reverse();
                return Some(out);
            }
            // an end is where a path stops, never a junction it passes
            if x != from && self.is_end(x) {
                continue;
            }
            for y in self.wires.iter().filter_map(|w| w.far(x)) {
                if !seen[y] {
                    seen[y] = true;
                    came_from[y] = Some(x);
                    queue.push_back(y);
                }
            }
        }
        None
    }

    /// Where a walker from `from` stands after each of its first `steps`
    /// steps: its chance of being at each junction, the top and the bottom
    /// keeping whatever reaches them. Unlike [`Network::still_walking`] it
    /// needs no end within reach, so it follows a walker that never stops
    /// (chapter 04's island). Refused for a junction outside the network,
    /// or when the walker could stand at a middle junction with no wire.
    pub fn step_chances(&self, from: usize, steps: usize) -> Result<Vec<Vec<Q>>, String> {
        if from >= self.size {
            return Err(format!("junction {from} is outside 0..{}", self.size));
        }
        let mut at = vec![Q::ZERO; self.size];
        at[from] = Q::ONE;
        let mut out = Vec::with_capacity(steps);
        for _ in 0..steps {
            let mut next = vec![Q::ZERO; self.size];
            for (x, &p) in at.iter().enumerate() {
                if p.is_zero() {
                    continue;
                }
                if self.is_end(x) {
                    next[x] = next[x] + p;
                    continue;
                }
                let total = self.total(x);
                if total.is_zero() {
                    return Err(format!("junction {x} has no wire to step along"));
                }
                for w in &self.wires {
                    if let Some(y) = w.far(x) {
                        next[y] = next[y] + p * w.conductance / total;
                    }
                }
            }
            out.push(next.clone());
            at = next;
        }
        Ok(out)
    }
}

// ---------------------------------------------------------------------------
// the networks the video uses
// ---------------------------------------------------------------------------

/// The opening network's junctions.
pub const TOP: usize = 0;
pub const LEFT: usize = 1;
pub const RIGHT: usize = 2;
pub const BOTTOM: usize = 3;
pub const JUNCTIONS: usize = 4;

/// The opening network's links: the two junctions, and how many identical
/// wires run side by side between them (its conductance, one per wire).
pub const LINKS: [(usize, usize, i128); 5] = [
    (TOP, LEFT, 2),
    (TOP, RIGHT, 1),
    (LEFT, RIGHT, 1),
    (LEFT, BOTTOM, 1),
    (RIGHT, BOTTOM, 2),
];

/// The route the walker is drawn taking (01_4): over to the right, back,
/// over again, and up to the top. One route among endlessly many.
pub const ROUTE: [usize; 5] = [LEFT, RIGHT, LEFT, RIGHT, TOP];

/// The opening network: a Wheatstone bridge of identical wires, doubled
/// from the top to the left and from the right to the bottom.
pub fn opening() -> Result<Network, String> {
    let wires = LINKS
        .iter()
        .map(|&(a, b, k)| Wire {
            a,
            b,
            conductance: Q::int(k),
        })
        .collect();
    Network::new(JUNCTIONS, wires, TOP, BOTTOM)
}

/// The opening network with the pair from the top to the left counted as
/// one wire (chapter 02's edge: a different network, with its own numbers,
/// still the same in both pictures).
pub fn single_pair() -> Result<Network, String> {
    let wires = LINKS
        .iter()
        .map(|&(a, b, k)| Wire {
            a,
            b,
            conductance: Q::int(if (a, b) == (TOP, LEFT) { 1 } else { k }),
        })
        .collect();
    Network::new(JUNCTIONS, wires, TOP, BOTTOM)
}

/// The opening network and two more junctions joined only to each other:
/// the edge of "exactly one solution".
pub fn with_island() -> Result<Network, String> {
    beside_island(&opening()?)
}

/// `net` and two more junctions, numbered after its own, joined by one wire
/// to each other and to nothing else (chapter 09's island beside the
/// bridge, chapter 11's beside the unequal wires).
pub fn beside_island(net: &Network) -> Result<Network, String> {
    let mut wires = net.wires().to_vec();
    wires.push(Wire {
        a: net.size(),
        b: net.size() + 1,
        conductance: Q::ONE,
    });
    Network::new(net.size() + 2, wires, net.top(), net.bottom())
}

/// The line of chapters 01 and 02: five identical wires, the six junctions
/// numbered from the bottom (0) to the top (5). Doyle and Snell's walk
/// along a five-block street (§1.1.1).
pub const LINE_WIRES: usize = 5;
/// Where the line's walker starts: one wire above the bottom.
pub const LINE_START: usize = 1;
/// The route the line's walker is drawn taking (01_3): up, down, up, up,
/// up and up, to the top. One route among endlessly many.
pub const LINE_ROUTE: [usize; 7] = [1, 2, 1, 2, 3, 4, 5];
/// The ghost walker's route (01_3, drawn without words): its first step
/// down, and it has stopped at the bottom.
pub const LINE_GHOST: [usize; 2] = [1, 0];

/// Doyle and Snell's unequal example (§1.3.1 to §1.3.2) drawn on the
/// opening's shape (chapter 10): their a, b, c, d as the top, the bottom,
/// the left and the right. Each link's conductance; a link of conductance 2
/// is one wire, drawn heavier, not two.
pub const UNEQUAL_LINKS: [(usize, usize, i128); 5] = [
    (TOP, LEFT, 1),
    (TOP, RIGHT, 1),
    (LEFT, RIGHT, 2),
    (LEFT, BOTTOM, 1),
    (RIGHT, BOTTOM, 2),
];

/// The unequal network, in the opening's junction order.
pub fn unequal() -> Result<Network, String> {
    let wires = UNEQUAL_LINKS
        .iter()
        .map(|&(a, b, k)| Wire {
            a,
            b,
            conductance: Q::int(k),
        })
        .collect();
    Network::new(JUNCTIONS, wires, TOP, BOTTOM)
}

/// Doyle and Snell's worked example (§1.3.1 to §1.3.2, figures 13 to 18):
/// junctions a, b, c, d are 0 to 3, a at one volt and b at zero.
pub fn doyle_snell() -> Result<Network, String> {
    let wire = |a, b, k| Wire {
        a,
        b,
        conductance: Q::int(k),
    };
    Network::new(
        4,
        vec![
            wire(0, 2, 1),
            wire(0, 3, 1),
            wire(1, 2, 1),
            wire(1, 3, 2),
            wire(2, 3, 2),
        ],
        0,
        1,
    )
}

/// `n` identical wires in a line (Doyle and Snell §1.1.4): junctions 0 to
/// `n`, the top at `n` and the bottom at 0.
pub fn line(n: usize) -> Result<Network, String> {
    let wires = (0..n)
        .map(|x| Wire {
            a: x,
            b: x + 1,
            conductance: Q::ONE,
        })
        .collect();
    Network::new(n + 1, wires, n, 0)
}

/// Every network on the opening's four junctions (the same ends) whose six
/// possible links are each absent, one wire or two wires side by side.
pub fn family() -> Result<Vec<Network>, String> {
    let pairs: Vec<(usize, usize)> = (0..JUNCTIONS)
        .flat_map(|a| (a + 1..JUNCTIONS).map(move |b| (a, b)))
        .collect();
    // each link absent, one wire or two: three choices per link
    let count: usize = pairs.iter().map(|_| 3).product();
    (0..count)
        .map(|code| {
            let mut rest = code;
            let mut wires = Vec::new();
            for &(a, b) in &pairs {
                let k = rest % 3;
                rest /= 3;
                if k > 0 {
                    wires.push(Wire {
                        a,
                        b,
                        conductance: Q::int(k as i128),
                    });
                }
            }
            Network::new(JUNCTIONS, wires, TOP, BOTTOM)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q(n: i128, d: i128) -> Q {
        Q::new(n, d)
    }

    fn wire(a: usize, b: usize, c: Q) -> Wire {
        Wire { a, b, conductance: c }
    }

    #[test]
    fn fractions_are_exact_and_print_like_python() {
        assert_eq!(q(6, 10), q(3, 5));
        assert_eq!(q(3, -6).to_string(), "-1/2");
        assert_eq!((q(1, 2) + q(1, 10)).to_string(), "3/5");
        assert_eq!((q(1, 4) * q(8, 5)).to_string(), "2/5");
        assert_eq!((Q::ONE - q(2, 5)) / q(1, 2), q(6, 5));
        assert_eq!(Q::int(4).to_string(), "4");
        assert!(q(3, 5) > q(4, 7));
        assert_eq!(q(-3, 7).whole(), None);
        assert_eq!(q(8, 4).whole(), Some(2));
    }

    /// A sum one past i128 panics instead of wrapping (an unchecked `+`
    /// wrapped i128::MAX + 1 to i128::MIN in release builds); one short of
    /// the limit stays exact.
    #[test]
    #[should_panic(expected = "a fraction overflowed i128")]
    fn a_sum_past_i128_panics_instead_of_wrapping() {
        assert_eq!(Q::int(i128::MAX - 1) + Q::ONE, Q::int(i128::MAX));
        let _ = Q::int(i128::MAX) + Q::ONE;
    }

    /// Negating, and so subtracting, cannot wrap either: i128::MIN has no
    /// negative.
    #[test]
    #[should_panic(expected = "a fraction overflowed i128")]
    fn negating_the_least_i128_panics_instead_of_wrapping() {
        assert_eq!(Q::ONE - Q::int(i128::MAX), Q::int(1 - i128::MAX));
        let _ = Q::ZERO - Q::int(i128::MIN);
    }

    #[test]
    fn the_opening_network_solves_both_ways_to_three_fifths_and_two_fifths() {
        let net = opening().unwrap();
        let v = net.kirchhoff(Q::ONE, Q::ZERO).unwrap();
        let w = net.walk().unwrap();
        assert_eq!(v, [Q::ONE, q(3, 5), q(2, 5), Q::ZERO]);
        assert_eq!(w.top, v);
        assert_eq!(w.bottom, [Q::ZERO, q(2, 5), q(3, 5), Q::ONE]);
        // each middle junction is the average of its four wires' far ends
        for x in net.middle() {
            assert_eq!(net.average(&v, x), Some(v[x]));
            let ends = net.far_ends(x).unwrap();
            assert_eq!(ends.len(), 4);
            let plain = ends.iter().fold(Q::ZERO, |s, &y| s + v[y]) / Q::int(4);
            assert_eq!(plain, v[x]);
        }
    }

    /// Chapter 02's balance, wire by wire: the pair carries two equal drops
    /// in, the other two wires carry drops out, and they cancel. Counting
    /// the pair as one wire is another network, the same in both pictures.
    #[test]
    fn the_drops_balance_and_the_pair_counts_twice() {
        let net = opening().unwrap();
        let v = net.kirchhoff(Q::ONE, Q::ZERO).unwrap();
        let drops = net.drops_into(&v, LEFT).unwrap();
        assert_eq!(drops, [q(2, 5), q(2, 5), q(-1, 5), q(-3, 5)]);
        assert!(drops.iter().fold(Q::ZERO, |s, &d| s + d).is_zero());
        // the junction's own balance, as net_out counts it, is the same
        assert_eq!(net.net_out(&v, LEFT), Q::ZERO);
        let one = single_pair().unwrap();
        let v1 = one.kirchhoff(Q::ONE, Q::ZERO).unwrap();
        assert_eq!(v1, [Q::ONE, q(5, 11), q(4, 11), Q::ZERO]);
        assert_eq!(one.walk().unwrap().top, v1);
        assert_eq!(one.between(LEFT, TOP) / one.total(LEFT), q(1, 3));
        assert_eq!(net.between(LEFT, TOP) / net.total(LEFT), q(1, 2));
        // a drop is refused where the wires cannot be drawn as identical ones
        let halves = Network::new(3, vec![wire(0, 1, q(1, 2)), wire(1, 2, Q::ONE)], 0, 2).unwrap();
        let vh = halves.kirchhoff(Q::ONE, Q::ZERO).unwrap();
        assert!(halves.drops_into(&vh, 1).is_err());
    }

    /// Chapter 03's repeated averaging on the opening network: the bigger
    /// gap is exactly 3/5 over 4 to the n, every iterate has a power-of-two
    /// denominator and so never equals 3/5; one junction at a time shrinks
    /// the gap by 1/16 a round from round 2; with the ends averaged too the
    /// weighted mean is kept and every value drifts to 1/2, while on a
    /// connected pair the mean is kept and nothing settles; on an island the
    /// two values swap for ever.
    #[test]
    fn repeated_averaging_closes_in_and_its_edges_fail() {
        let net = opening().unwrap();
        let solved = net.kirchhoff(Q::ONE, Q::ZERO).unwrap();
        let mut now = vec![Q::ONE, Q::ZERO, Q::ZERO, Q::ZERO];
        for n in 0..=40u32 {
            let gap = (solved[LEFT] - now[LEFT]).max(solved[RIGHT] - now[RIGHT]);
            assert_eq!(gap, q(3, 5) / Q::new(4i128.pow(n), 1), "round {n}");
            for x in [LEFT, RIGHT] {
                let d = now[x].denom();
                assert_eq!(d & (d - 1), 0, "round {n}: {} is not over a power of two", now[x]);
                assert_ne!(now[x], solved[x], "round {n}");
            }
            now = net.averaging_round(&now).unwrap();
        }
        // one at a time, the newest values: 1/6 first, then 1/16 a round
        let mut seq = vec![Q::ONE, Q::ZERO, Q::ZERO, Q::ZERO];
        let mut gaps = Vec::new();
        for _ in 0..=6 {
            gaps.push((solved[LEFT] - seq[LEFT]).max(solved[RIGHT] - seq[RIGHT]));
            seq = net.sequential_round(&seq).unwrap();
        }
        assert_eq!(gaps[1] / gaps[0], q(1, 6));
        for w in gaps[1..].windows(2) {
            assert_eq!(w[1] / w[0], q(1, 16));
        }
        // the ends averaged too: the weighted mean stays, the battery is lost
        let mut free = solved.clone();
        let mean = net.weighted_mean(&free).unwrap();
        assert_eq!(mean, q(1, 2));
        free = net.free_round(&free).unwrap();
        assert_eq!(free, [q(8, 15), q(3, 5), q(2, 5), q(7, 15)]);
        for _ in 1..6 {
            free = net.free_round(&free).unwrap();
            assert_eq!(net.weighted_mean(&free).unwrap(), mean);
        }
        assert!(free.iter().all(|&x| (x - mean).max(mean - x) < q(1, 1000)));
        // a kept mean is not settling: two junctions joined by one wire, both
        // averaged, swap 1 and 0 for ever around their mean 1/2
        let pair = Network::new(2, vec![wire(0, 1, Q::ONE)], 0, 1).unwrap();
        let start = vec![Q::ONE, Q::ZERO];
        let mut now = start.clone();
        for round in 1..=8 {
            now = pair.free_round(&now).unwrap();
            assert_eq!(pair.weighted_mean(&now).unwrap(), q(1, 2), "round {round}");
            let swapped = if round % 2 == 1 {
                [Q::ZERO, Q::ONE]
            } else {
                [Q::ONE, Q::ZERO]
            };
            assert_eq!(now, swapped, "round {round}");
            assert!(now.iter().all(|&x| (x - q(1, 2)).max(q(1, 2) - x) == q(1, 2)));
        }
        assert_eq!(now, start);
        // an island's two values swap for ever: nothing settles there
        let island = with_island().unwrap();
        let start = vec![Q::ONE, Q::ZERO, Q::ZERO, Q::ZERO, Q::ZERO, Q::ONE];
        let once = island.averaging_round(&start).unwrap();
        assert_eq!(once[4..], [Q::ONE, Q::ZERO]);
        assert_eq!(island.averaging_round(&once).unwrap()[4..], [Q::ZERO, Q::ONE]);
        // a junction with no wire has nothing to average
        let lonely = Network::new(3, vec![wire(0, 2, Q::ONE)], 0, 2).unwrap();
        assert!(lonely.averaging_round(&[Q::ONE, Q::ZERO, Q::ZERO]).is_err());
        assert!(lonely.sequential_round(&[Q::ONE, Q::ZERO, Q::ZERO]).is_err());
        assert!(
            Network::new(2, vec![], 0, 1)
                .unwrap()
                .weighted_mean(&[Q::ONE, Q::ZERO])
                .is_err()
        );
    }

    /// The solvers refuse what they cannot solve and say why.
    #[test]
    fn bad_networks_are_refused() {
        let one = Q::ONE;
        assert!(
            Network::new(3, vec![wire(0, 1, one)], 0, 0)
                .unwrap_err()
                .contains("both")
        );
        assert!(
            Network::new(3, vec![wire(0, 3, one)], 0, 2)
                .unwrap_err()
                .contains("outside")
        );
        assert!(
            Network::new(3, vec![wire(1, 1, one)], 0, 2)
                .unwrap_err()
                .contains("itself")
        );
        assert!(
            Network::new(3, vec![wire(0, 1, Q::ZERO)], 0, 2)
                .unwrap_err()
                .contains("positive")
        );
        assert!(
            Network::new(3, vec![wire(0, 1, -one)], 0, 2)
                .unwrap_err()
                .contains("positive")
        );
        assert!(Network::new(2, vec![], 0, 5).is_err());
        // a junction with no wire at all is cut off, not a division by zero
        let lonely = Network::new(3, vec![wire(0, 2, one)], 0, 2).unwrap();
        assert_eq!(lonely.cut_off(), [1]);
        assert!(lonely.kirchhoff(one, Q::ZERO).unwrap_err().contains("junction 1"));
        assert!(lonely.walk().unwrap_err().contains("never stops"));
        assert_eq!(lonely.average(&[one, one, Q::ZERO], 1), None);
        assert_eq!(lonely.middle_determinant(), Q::ZERO);
    }

    /// Two junctions joined only to each other: every equal pair of values
    /// balances there, so no voltage is fixed and the determinant is zero.
    #[test]
    fn an_island_has_no_unique_answer() {
        let island = with_island().unwrap();
        assert_eq!(island.cut_off(), [4, 5]);
        assert_eq!(island.middle_determinant(), Q::ZERO);
        assert!(island.kirchhoff(Q::ONE, Q::ZERO).is_err());
        assert!(island.walk().is_err());
        assert!(island.still_walking(LEFT, 3).is_err());
    }

    /// Chapter 04: a peak spreads along a path of wires through middle
    /// junctions. From either middle junction of the opening the top is one
    /// wire away; from the island no path reaches either end, and a path
    /// never passes through an end. Across the family a middle junction has
    /// a path to the top or the bottom exactly when it is not cut off, and
    /// marking the cut-off junctions 1 and every other 0 keeps the balance
    /// with both ends at 0: a second solution beside 0, where elsewhere 0 is
    /// the only one.
    #[test]
    fn a_peak_reaches_an_end_exactly_where_a_path_of_wires_does() {
        let net = opening().unwrap();
        assert_eq!(net.path(LEFT, TOP), Some(vec![LEFT, TOP]));
        assert_eq!(net.path(RIGHT, TOP), Some(vec![RIGHT, TOP]));
        assert_eq!(net.path(LEFT, LEFT), Some(vec![LEFT]));
        assert_eq!(net.path(LEFT, 9), None);
        let row = line(3).unwrap();
        assert_eq!(row.path(1, 3), Some(vec![1, 2, 3]));
        // the top is reached from junction 1 only through the bottom: no path
        let blocked = Network::new(
            4,
            vec![wire(1, 3, Q::ONE), wire(3, 0, Q::ONE), wire(2, 0, Q::ONE)],
            0,
            3,
        )
        .unwrap();
        assert_eq!(blocked.path(1, 0), None);
        assert_eq!(blocked.path(1, 3), Some(vec![1, 3]));
        assert_eq!(blocked.path(2, 0), Some(vec![2, 0]));
        let island = with_island().unwrap();
        for x in island.cut_off() {
            assert_eq!(island.path(x, TOP), None);
            assert_eq!(island.path(x, BOTTOM), None);
        }
        assert_eq!(island.path(4, 5), Some(vec![4, 5]));
        let (mut free, mut only_zero) = (0, 0);
        for fam in family().unwrap() {
            let cut = fam.cut_off();
            for x in fam.middle() {
                let reaches = fam.path(x, TOP).is_some() || fam.path(x, BOTTOM).is_some();
                assert_eq!(reaches, !cut.contains(&x), "junction {x} of {:?}", fam.wires());
            }
            if cut.is_empty() {
                let zero = fam.kirchhoff(Q::ZERO, Q::ZERO).unwrap();
                assert!(zero.iter().all(|q| q.is_zero()), "{:?}", fam.wires());
                only_zero += 1;
            } else {
                let marked: Vec<Q> = (0..fam.size())
                    .map(|x| if cut.contains(&x) { Q::ONE } else { Q::ZERO })
                    .collect();
                let keeps = fam.middle().iter().all(|&x| fam.net_out(&marked, x).is_zero());
                assert!(keeps, "{:?}", fam.wires());
                free += 1;
            }
        }
        assert_eq!((free, only_zero), (57, 672));
    }

    /// The walk followed step by step: on the opening network the chance of
    /// still being in the middle is chapter 01's 1/4 to the n, and the rest
    /// has reached an end; from the island the walker crosses its one wire
    /// every step and is never at either end.
    #[test]
    fn the_walk_step_by_step_and_the_walker_that_never_stops() {
        let net = opening().unwrap();
        let steps = net.step_chances(LEFT, 6).unwrap();
        let still = net.still_walking(LEFT, 6).unwrap();
        assert_eq!(steps[0], [q(1, 2), Q::ZERO, q(1, 4), q(1, 4)]);
        for (n, s) in steps.iter().enumerate() {
            assert_eq!(s[LEFT] + s[RIGHT], still[n], "step {}", n + 1);
            assert_eq!(s.iter().fold(Q::ZERO, |t, &p| t + p), Q::ONE, "step {}", n + 1);
            assert!(s[TOP] < q(3, 5), "step {}", n + 1);
        }
        assert!(steps.windows(2).all(|w| w[1][TOP] > w[0][TOP]));
        let island = with_island().unwrap();
        let from = island.cut_off()[0];
        for (n, s) in island.step_chances(from, 12).unwrap().iter().enumerate() {
            assert_eq!((s[TOP], s[BOTTOM]), (Q::ZERO, Q::ZERO), "step {}", n + 1);
            let there = if n % 2 == 0 { from + 1 } else { from };
            assert_eq!(s[there], Q::ONE, "step {}", n + 1);
        }
        // refused: a junction outside the network, a middle junction with no
        // wire; a walker at an end stays there
        assert!(net.step_chances(9, 1).is_err());
        let lonely = Network::new(3, vec![wire(0, 2, Q::ONE)], 0, 2).unwrap();
        assert!(lonely.step_chances(1, 1).unwrap_err().contains("no wire"));
        assert_eq!(lonely.step_chances(0, 2).unwrap()[1], [Q::ONE, Q::ZERO, Q::ZERO]);
    }

    #[test]
    fn identical_wires_in_a_line_divide_the_volt_evenly() {
        for n in 1..=8usize {
            let net = line(n).unwrap();
            let v = net.kirchhoff(Q::ONE, Q::ZERO).unwrap();
            let want: Vec<Q> = (0..=n).map(|x| q(x as i128, n as i128)).collect();
            assert_eq!(v, want, "{n} wires");
            assert_eq!(net.walk().unwrap().top, want, "{n} wires");
        }
    }

    /// Chapter 01's line: from one wire above the bottom the walker reaches
    /// the top first with chance 1/5; every chance is halfway between its
    /// neighbours'; the drawn route and the ghost's are walks; the halfway
    /// rule with both ends at zero leaves only zero, and the chance of still
    /// walking shrinks towards it, never reaching it, so the two ends'
    /// chances add to one.
    #[test]
    fn the_line_walk_climbs_in_equal_steps_and_stops_with_chance_one() {
        let row = line(LINE_WIRES).unwrap();
        let walk = row.walk().unwrap();
        let want: Vec<Q> = (0..=LINE_WIRES).map(|x| q(x as i128, LINE_WIRES as i128)).collect();
        assert_eq!(walk.top, want);
        assert_eq!(walk.top[LINE_START], q(1, 5));
        for x in row.middle() {
            assert_eq!(walk.top[x] - walk.top[x - 1], walk.top[x + 1] - walk.top[x]);
            assert_eq!(row.far_ends(x).unwrap().len(), 2);
            assert_eq!(walk.top[x] + walk.bottom[x], Q::ONE);
        }
        assert!(row.is_route(&LINE_ROUTE));
        assert!(row.is_route(&LINE_GHOST));
        assert!(row.kirchhoff(Q::ZERO, Q::ZERO).unwrap().iter().all(|v| v.is_zero()));
        let still = row.still_walking(LINE_START, 60).unwrap();
        assert!(still.iter().all(|s| s.is_positive()));
        assert!(still.windows(2).all(|w| w[1] <= w[0]));
        assert!(still[59] < q(1, 1000));
    }

    /// Chapter 03's dead end: with each pair of wires side by side merged,
    /// no two links join the same two junctions and three links meet at each
    /// middle junction, so neither school shortcut applies. On a line each
    /// middle junction meets two: its wires are in a row.
    #[test]
    fn the_bridge_leaves_no_shortcut() {
        let net = opening().unwrap();
        let mut pairs: Vec<(usize, usize)> = net.wires().iter().map(|w| (w.a.min(w.b), w.a.max(w.b))).collect();
        pairs.sort_unstable();
        pairs.dedup();
        assert_eq!(pairs.len(), net.wires().len());
        for x in net.middle() {
            assert_eq!(net.neighbours(x).len(), 3);
        }
        assert_eq!(net.neighbours(LEFT), [TOP, RIGHT, BOTTOM]);
        assert_eq!(line(2).unwrap().neighbours(1), [0, 2]);
    }

    /// Chapter 10's unequal wires are Doyle and Snell's example relabelled:
    /// their a, b, c, d as the top, the bottom, the left and the right, the
    /// same totals and the same voltages, and the walk agrees.
    #[test]
    fn the_unequal_network_is_doyle_and_snells() {
        let drawn = unequal().unwrap();
        let theirs = doyle_snell().unwrap();
        let v = drawn.kirchhoff(Q::ONE, Q::ZERO).unwrap();
        let w = theirs.kirchhoff(Q::ONE, Q::ZERO).unwrap();
        assert_eq!([v[TOP], v[BOTTOM], v[LEFT], v[RIGHT]], [w[0], w[1], w[2], w[3]]);
        assert_eq!((v[LEFT], v[RIGHT]), (q(7, 16), q(3, 8)));
        assert_eq!(drawn.walk().unwrap().top, v);
        for (x, y) in [(TOP, 0), (BOTTOM, 1), (LEFT, 2), (RIGHT, 3)] {
            assert_eq!(drawn.total(x), theirs.total(y));
        }
        assert_eq!(drawn.between(LEFT, RIGHT), theirs.between(2, 3));
    }

    /// Chapter 11's bracket, on the bridge and on the unequal wires: the
    /// top stays 1 and the bottom 0 in both runs, the run from zeros only
    /// rises, the run from ones only falls, the solution stays between
    /// them, and their gap is exactly the chance of still walking. The
    /// unequal wires' runs first come within a thousandth of each other
    /// after round 9; beside an island the island's runs never move. (Twenty
    /// rounds on the bridge: comparing two of its rounds' fractions past
    /// that would overflow i128, which panics.)
    #[test]
    fn two_runs_bracket_the_answer_and_their_gap_is_the_walk_still_going() {
        for net in [opening().unwrap(), unequal().unwrap()] {
            let v = net.kirchhoff(Q::ONE, Q::ZERO).unwrap();
            let (low, high) = net.two_runs(20).unwrap();
            for n in 0..=20 {
                for x in 0..net.size() {
                    assert!(low[n][x] <= v[x] && v[x] <= high[n][x], "round {n} junction {x}");
                    if n > 0 {
                        assert!(low[n][x] >= low[n - 1][x] && high[n][x] <= high[n - 1][x], "round {n}");
                    }
                }
                assert_eq!(
                    (low[n][TOP], high[n][TOP], low[n][BOTTOM], high[n][BOTTOM]),
                    (Q::ONE, Q::ONE, Q::ZERO, Q::ZERO)
                );
            }
            for x in net.middle() {
                let still = net.still_walking(x, 12).unwrap();
                for n in 1..=12 {
                    assert_eq!(high[n][x] - low[n][x], still[n - 1], "junction {x} round {n}");
                }
            }
        }
        let (low, high) = unequal().unwrap().two_runs(12).unwrap();
        let widest = |n: usize| (high[n][LEFT] - low[n][LEFT]).max(high[n][RIGHT] - low[n][RIGHT]);
        assert_eq!((widest(8), widest(9)), (q(1, 625), q(1, 1250)));
        assert_eq!((1..=12).find(|&n| widest(n) < q(1, 1000)), Some(9));
        let island = beside_island(&unequal().unwrap()).unwrap();
        assert_eq!(island.cut_off(), [4, 5]);
        let (low, high) = island.two_runs(40).unwrap();
        for n in 0..=40 {
            assert_eq!(
                (low[n][4], low[n][5], high[n][4], high[n][5]),
                (Q::ZERO, Q::ZERO, Q::ONE, Q::ONE),
                "round {n}"
            );
        }
        assert_eq!(high[9][LEFT] - low[9][LEFT], q(1, 1250));
        // with_island is the bridge beside an island
        let bridge_island = with_island().unwrap();
        assert_eq!(bridge_island.size(), JUNCTIONS + 2);
        assert_eq!(bridge_island.cut_off(), [JUNCTIONS, JUNCTIONS + 1]);
    }

    /// With conductances that are not whole numbers the rule weights each
    /// far end by its conductance; a non-whole conductance has no drawing
    /// as identical wires.
    #[test]
    fn unequal_conductances_weight_the_average() {
        let net = Network::new(
            4,
            vec![
                wire(0, 1, q(1, 2)),
                wire(1, 2, Q::int(3)),
                wire(2, 3, q(5, 2)),
                wire(1, 3, Q::ONE),
                wire(0, 2, q(1, 3)),
            ],
            0,
            3,
        )
        .unwrap();
        let v = net.kirchhoff(Q::ONE, Q::ZERO).unwrap();
        assert_eq!(net.walk().unwrap().top, v);
        for x in net.middle() {
            assert_eq!(net.average(&v, x), Some(v[x]));
            assert!(v[x] > Q::ZERO && v[x] < Q::ONE);
            assert!(net.net_out(&v, x).is_zero());
        }
        assert!(net.far_ends(1).unwrap_err().contains("whole"));
    }

    /// Other boundary values give the same equations' other solutions:
    /// voltages are linear in the ends, and zero ends give zero everywhere.
    #[test]
    fn the_balance_is_linear_in_the_ends() {
        let net = doyle_snell().unwrap();
        let unit = net.kirchhoff(Q::ONE, Q::ZERO).unwrap();
        let zero = net.kirchhoff(Q::ZERO, Q::ZERO).unwrap();
        assert!(zero.iter().all(|v| v.is_zero()));
        let shifted = net.kirchhoff(Q::int(3), Q::int(1)).unwrap();
        for (s, u) in shifted.iter().zip(&unit) {
            assert_eq!(*s, Q::ONE + Q::int(2) * *u);
        }
    }

    #[test]
    fn the_walk_from_the_left_is_still_going_with_chance_a_quarter_per_step() {
        let net = opening().unwrap();
        let left = net.still_walking(LEFT, 5).unwrap();
        let want: Vec<Q> = (1..=5u32).map(|n| q(1, 4i128.pow(n))).collect();
        assert_eq!(left, want);
        assert!(net.still_walking(TOP, 2).unwrap_err().contains("an end"));
    }

    #[test]
    fn routes_follow_wires_and_stop_at_the_first_end() {
        let net = opening().unwrap();
        assert!(net.is_route(&ROUTE));
        assert!(net.is_route(&[LEFT, BOTTOM]));
        assert!(!net.is_route(&[LEFT]));
        assert!(!net.is_route(&[LEFT, RIGHT]));
        assert!(!net.is_route(&[LEFT, TOP, RIGHT, BOTTOM]));
        assert!(!net.is_route(&[TOP, LEFT, BOTTOM]));
        assert!(!net.is_route(&[LEFT, 7]));
        // a step needs a wire: in a line, junction 1 is not joined to 3
        let row = line(3).unwrap();
        assert!(row.is_route(&[1, 2, 3]));
        assert!(!row.is_route(&[1, 3]));
    }

    /// Every network of the family: the two pictures agree wherever the
    /// answer is determined, the values stay between the ends' (the maximum
    /// principle), and the determinant is zero exactly where a junction is
    /// cut off.
    #[test]
    fn across_the_family_both_pictures_agree() {
        let all = family().unwrap();
        assert_eq!(all.len(), 729);
        for net in &all {
            let det = net.middle_determinant();
            if net.cut_off().is_empty() {
                assert!(det.is_positive());
                let v = net.kirchhoff(Q::ONE, Q::ZERO).unwrap();
                let w = net.walk().unwrap();
                assert_eq!(w.top, v);
                for x in net.middle() {
                    assert_eq!(w.top[x] + w.bottom[x], Q::ONE);
                    assert!((Q::ZERO..=Q::ONE).contains(&v[x]));
                }
            } else {
                assert!(det.is_zero());
            }
        }
    }

    #[test]
    fn small_matrices_invert_and_solve_exactly() {
        let mut m = Mat::zeros(2);
        m.set(0, 0, Q::int(4));
        m.set(0, 1, -Q::ONE);
        m.set(1, 0, -Q::ONE);
        m.set(1, 1, Q::int(4));
        assert_eq!(determinant(&m), Q::int(15));
        assert_eq!(solve(&m, &[Q::int(2), Q::ONE]).unwrap(), [q(3, 5), q(2, 5)]);
        let inv = invert(&m).unwrap();
        assert_eq!((inv.at(0, 0), inv.at(0, 1)), (q(4, 15), q(1, 15)));
        // a zero first pivot needs a row swap, and the swap flips the sign
        let mut s = Mat::zeros(2);
        s.set(0, 1, Q::ONE);
        s.set(1, 0, Q::ONE);
        assert_eq!(determinant(&s), -Q::ONE);
        assert_eq!(solve(&s, &[Q::int(2), Q::int(3)]).unwrap(), [Q::int(3), Q::int(2)]);
        assert!(invert(&Mat::zeros(2)).is_none());
        assert!(solve(&Mat::zeros(2), &[Q::ONE, Q::ONE]).is_none());
        assert_eq!(determinant(&Mat::zeros(0)), Q::ONE);
    }
}
