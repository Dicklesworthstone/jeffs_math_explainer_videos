//! The video's one model: the card, the coast drawn on it from straight
//! runs and circular arcs, and what the film measures on it. Lengths are
//! card centimetres. `facts.rs` prints the numbers for `oracle.py`, which
//! computes them another way; every scene draws from here.
//!
//! How each value is obtained, independently of the oracle (which walks
//! chords, counts with floor arithmetic and integrates by Simpson's rule):
//! - the coast's length exactly, as a rational number plus a rational
//!   multiple of π;
//! - each divider landing by meeting the step's circle with every straight
//!   run (a quadratic) and every arc (two circles), never by walking chords;
//! - a direction's travel across the lines in closed form: the integral of
//!   |sin| over each arc, the shadow of each straight run;
//! - the average over directions by a midpoint rule;
//! - the ruler's contacts piece by piece, each piece's highest point found
//!   from its ends and, on an arc, the angle facing the ruler.

use std::f64::consts::PI;

/// The card, centimetres.
pub const CARD_W: f64 = 16.0;
pub const CARD_H: f64 = 9.0;
/// Frame units per card centimetre, and where the card's centre sits in
/// the frame (the card fills the frame; words sit in its empty bands).
pub const CARD_UNIT: f64 = 0.8;
pub const CARD_CENTRE: [f64; 2] = [0.0, 0.25];
/// The sheet's gap, cm (the draft's chapter 02 on), and the coin as wide
/// as the gap that calibrates one crossing (its chapter 05).
pub const GAP: f64 = 2.0;
pub const COIN: f64 = 2.0;
const _: () = assert!(
    COIN == GAP,
    "the coin calibrates the sheet only when it is as wide as the gap"
);

/// The step chapter 01 opens the dividers to, and the step 01_7 closes
/// them to, cm.
pub const OPENING_STEP: f64 = 2.0;
pub const SMALLER_STEP: f64 = 1.0;
/// Chapter 01's ruler, cm.
pub const RULER: f64 = 5.0;

/// A rational number in lowest terms with a positive denominator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ratio {
    pub num: i64,
    pub den: i64,
}

const fn gcd(a: i64, b: i64) -> i64 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

impl Ratio {
    pub const fn new(num: i64, den: i64) -> Self {
        assert!(den != 0, "a ratio needs a nonzero denominator");
        let g = gcd(num, den);
        let sign = if den < 0 { -1 } else { 1 };
        Self {
            num: sign * num / g,
            den: sign * den / g,
        }
    }

    pub const fn whole(n: i64) -> Self {
        Self { num: n, den: 1 }
    }

    pub const fn plus(self, other: Self) -> Self {
        Self::new(self.num * other.den + other.num * self.den, self.den * other.den)
    }

    pub const fn times(self, other: Self) -> Self {
        Self::new(self.num * other.num, self.den * other.den)
    }

    pub fn value(self) -> f64 {
        self.num as f64 / self.den as f64
    }
}

/// "15/2", or "3" for a whole number (as Python's `Fraction` prints).
impl std::fmt::Display for Ratio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.den == 1 {
            write!(f, "{}", self.num)
        } else {
            write!(f, "{}/{}", self.num, self.den)
        }
    }
}

/// One piece of a drawn curve, as the storyboard lists the coast.
#[derive(Clone, Copy, Debug)]
pub enum Piece {
    /// A straight run, cm.
    Straight(Ratio),
    /// An arc of `radius` cm turning `turn` degrees, left positive.
    Arc { radius: Ratio, turn: i32 },
}

/// The coast (storyboard, "Geometry"): its start, its first heading in
/// degrees, then its pieces. Every turn is a whole number of degrees, so
/// every arc's length is a rational multiple of π.
pub const COAST_START: [Ratio; 2] = [Ratio::new(-73, 10), Ratio::new(-1, 2)];
pub const COAST_HEADING: i32 = -75;
pub const COAST: [Piece; 11] = [
    Piece::Arc {
        radius: Ratio::whole(2),
        turn: 75,
    },
    Piece::Straight(Ratio::new(7, 2)),
    Piece::Arc {
        radius: Ratio::new(5, 4),
        turn: 105,
    },
    Piece::Straight(Ratio::new(3, 2)),
    Piece::Arc {
        radius: Ratio::new(3, 4),
        turn: -195,
    },
    Piece::Straight(Ratio::new(3, 2)),
    Piece::Arc {
        radius: Ratio::new(7, 4),
        turn: 165,
    },
    Piece::Arc {
        radius: Ratio::whole(1),
        turn: -120,
    },
    Piece::Straight(Ratio::whole(1)),
    Piece::Arc {
        radius: Ratio::new(3, 5),
        turn: 180,
    },
    Piece::Arc {
        radius: Ratio::whole(1),
        turn: -105,
    },
];
/// The pieces' names, as the narration and the labels use them.
pub const PIECE_NAMES: [&str; 11] = [
    "cove", "beach", "climb", "rise", "headland", "fall", "bay", "point", "run", "hook", "end",
];
pub const BEACH: usize = 1;
pub const HEADLAND: usize = 4;
pub const BAY: usize = 6;
pub const HOOK: usize = 9;
/// The parts 01_1 names, in the order it names them.
pub const NAMED: [usize; 5] = [0, BEACH, HEADLAND, BAY, HOOK];

/// The coast's length exactly: (the straight runs, the coefficient of π).
pub const fn exact_length(pieces: &[Piece]) -> (Ratio, Ratio) {
    let (mut straight, mut arcs) = (Ratio::whole(0), Ratio::whole(0));
    let mut i = 0;
    while i < pieces.len() {
        match pieces[i] {
            Piece::Straight(l) => straight = straight.plus(l),
            Piece::Arc { radius, turn } => arcs = arcs.plus(radius.times(Ratio::new(turn.unsigned_abs() as i64, 180))),
        }
        i += 1;
    }
    (straight, arcs)
}

pub type P2 = [f64; 2];

fn dist(a: P2, b: P2) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

fn dot(a: P2, b: P2) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}

/// A piece placed on the card.
#[derive(Clone, Copy, Debug)]
pub enum Seg {
    Line {
        from: P2,
        to: P2,
    },
    /// Angles in radians, `from` to `to` (to < from turns clockwise).
    Arc {
        centre: P2,
        radius: f64,
        from: f64,
        to: f64,
    },
}

impl Seg {
    pub fn len(&self) -> f64 {
        match *self {
            Seg::Line { from, to } => dist(from, to),
            Seg::Arc { radius, from, to, .. } => radius * (to - from).abs(),
        }
    }

    /// The point a share `u` (0 to 1) of the way along.
    pub fn point(&self, u: f64) -> P2 {
        match *self {
            Seg::Line { from, to } => [from[0] + u * (to[0] - from[0]), from[1] + u * (to[1] - from[1])],
            Seg::Arc {
                centre,
                radius,
                from,
                to,
            } => {
                let a = from + u * (to - from);
                [centre[0] + radius * a.cos(), centre[1] + radius * a.sin()]
            }
        }
    }
}

/// A placed piece and where it starts, cm along the coast.
#[derive(Clone, Copy, Debug)]
pub struct Part {
    pub seg: Seg,
    pub start: f64,
}

impl Part {
    pub fn end(&self) -> f64 {
        self.start + self.seg.len()
    }
}

/// A drawn curve: its placed pieces and its length.
#[derive(Clone, Debug)]
pub struct Curve {
    pub parts: Vec<Part>,
    pub length: f64,
}

/// A divider walk: every landing as (point, cm along the curve), the first
/// being the curve's start, and the last straight bit to the end.
#[derive(Clone, Debug)]
pub struct Walk {
    pub step: f64,
    pub landings: Vec<(P2, f64)>,
    pub last: f64,
}

impl Walk {
    /// Whole steps taken.
    pub fn steps(&self) -> usize {
        self.landings.len() - 1
    }

    /// What the dividers measure: the steps and the last straight bit.
    pub fn length(&self) -> f64 {
        self.steps() as f64 * self.step + self.last
    }
}

/// The spacing below which two numbers are one (a step landing on the far
/// end to within this counts as a step, as in the oracle).
const EPS: f64 = 1e-9;

impl Curve {
    /// `pieces` laid end to end from `start`, first heading `heading`
    /// degrees.
    pub fn build(start: [Ratio; 2], heading: i32, pieces: &[Piece]) -> Self {
        let (mut x, mut y) = (start[0].value(), start[1].value());
        let mut h = f64::from(heading).to_radians();
        let mut parts = Vec::with_capacity(pieces.len());
        let mut s = 0.0;
        for piece in pieces {
            let seg = match *piece {
                Piece::Straight(l) => {
                    let to = [x + l.value() * h.cos(), y + l.value() * h.sin()];
                    let seg = Seg::Line { from: [x, y], to };
                    [x, y] = to;
                    seg
                }
                Piece::Arc { radius, turn } => {
                    let (r, t) = (radius.value(), f64::from(turn).to_radians());
                    let side = t.signum();
                    let centre = [x - side * r * h.sin(), y + side * r * h.cos()];
                    let from = (y - centre[1]).atan2(x - centre[0]);
                    let to = from + t;
                    [x, y] = [centre[0] + r * to.cos(), centre[1] + r * to.sin()];
                    h += t;
                    Seg::Arc {
                        centre,
                        radius: r,
                        from,
                        to,
                    }
                }
            };
            parts.push(Part { seg, start: s });
            s += seg.len();
        }
        Self { parts, length: s }
    }

    /// The point `s` cm along (clamped to the curve).
    pub fn at(&self, s: f64) -> P2 {
        let s = s.clamp(0.0, self.length);
        let part = self
            .parts
            .iter()
            .find(|p| s <= p.end())
            .unwrap_or_else(|| &self.parts[self.parts.len() - 1]);
        let len = part.seg.len();
        part.seg.point(if len > 0.0 {
            ((s - part.start) / len).clamp(0.0, 1.0)
        } else {
            0.0
        })
    }

    pub fn ends(&self) -> (P2, P2) {
        (self.at(0.0), self.at(self.length))
    }

    /// The curve from its start to `s` cm along as a polyline: every joint,
    /// and points at most `every` cm apart along each arc. Never fewer than
    /// two points (a curve drawn 0.01 cm, at least).
    pub fn upto(&self, s: f64, every: f64) -> Vec<P2> {
        self.between(0.0, s.max(0.01_f64.min(self.length)), every)
    }

    /// The curve from `from` to `to` cm along, as [`Curve::upto`] draws it.
    pub fn between(&self, from: f64, to: f64, every: f64) -> Vec<P2> {
        let (from, to) = (from.clamp(0.0, self.length), to.clamp(0.0, self.length));
        let mut out = vec![self.at(from)];
        for part in &self.parts {
            let len = part.seg.len();
            let (u0, u1) = (((from - part.start) / len).max(0.0), ((to - part.start) / len).min(1.0));
            if u1 <= u0 {
                continue;
            }
            let n = match part.seg {
                Seg::Line { .. } => 1,
                Seg::Arc { .. } => (((u1 - u0) * len / every).ceil() as usize).max(1),
            };
            out.extend((1..=n).map(|k| part.seg.point(u0 + (u1 - u0) * k as f64 / n as f64)));
        }
        out
    }

    /// The middle of piece `i` and the side its label goes: away from an
    /// arc's centre, or to the right of a straight run's heading.
    pub fn middle_and_outside(&self, i: usize) -> (P2, P2) {
        match self.parts[i].seg {
            Seg::Line { from, to } => {
                let d = dist(from, to);
                (
                    [(from[0] + to[0]) / 2.0, (from[1] + to[1]) / 2.0],
                    [(to[1] - from[1]) / d, -(to[0] - from[0]) / d],
                )
            }
            Seg::Arc {
                centre,
                radius,
                from,
                to,
            } => {
                let a = (from + to) / 2.0;
                (
                    [centre[0] + radius * a.cos(), centre[1] + radius * a.sin()],
                    [a.cos(), a.sin()],
                )
            }
        }
    }

    /// The first point further along than `s0` that lies exactly `step`
    /// from `here`, as cm along: the step's circle met with every piece in
    /// turn, the earliest meeting kept.
    fn first_reach(&self, here: P2, s0: f64, step: f64) -> Option<f64> {
        for part in self.parts.iter().filter(|p| p.end() > s0) {
            let len = part.seg.len();
            let mut best: Option<f64> = None;
            let mut keep = |u: f64| {
                let s = part.start + u * len;
                if (-1e-12..=1.0 + 1e-12).contains(&u) && s > s0 + 1e-12 {
                    best = Some(best.map_or(s, |b: f64| b.min(s)));
                }
            };
            match part.seg {
                Seg::Line { from, to } => {
                    // |from + t (to - from) - here|^2 = step^2
                    let d = [to[0] - from[0], to[1] - from[1]];
                    let w = [from[0] - here[0], from[1] - here[1]];
                    let (qa, qb, qc) = (dot(d, d), 2.0 * dot(w, d), dot(w, w) - step * step);
                    let disc = qb * qb - 4.0 * qa * qc;
                    if disc >= 0.0 {
                        keep((-qb - disc.sqrt()) / (2.0 * qa));
                        keep((-qb + disc.sqrt()) / (2.0 * qa));
                    }
                }
                Seg::Arc {
                    centre,
                    radius,
                    from,
                    to,
                } => {
                    // |centre + r e(a) - here|^2 = d^2 + r^2 + 2 r w.e(a),
                    // w = centre - here; w.e(a) = d cos(a - phi)
                    let w = [centre[0] - here[0], centre[1] - here[1]];
                    let d = w[0].hypot(w[1]);
                    if d > 1e-12 {
                        let c = (step * step - d * d - radius * radius) / (2.0 * radius * d);
                        if c.abs() <= 1.0 {
                            let phi = w[1].atan2(w[0]);
                            for base in [phi + c.acos(), phi - c.acos()] {
                                for k in -3..=3 {
                                    keep((base + std::f64::consts::TAU * f64::from(k) - from) / (to - from));
                                }
                            }
                        }
                    }
                }
            }
            // pieces run in order: a meeting on this piece beats any later
            if best.is_some() {
                return best;
            }
        }
        None
    }

    /// Richardson's dividers, `step` cm: each step lands where the curve,
    /// followed onward from the last landing, first comes exactly `step`
    /// away in a straight line.
    pub fn walk(&self, step: f64) -> Walk {
        let (mut here, mut s) = (self.at(0.0), 0.0);
        let mut landings = vec![(here, s)];
        while let Some(next) = self.first_reach(here, s, step) {
            s = next;
            here = self.at(s);
            landings.push((here, s));
            if s >= self.length - EPS {
                break;
            }
        }
        let last = dist(here, self.at(self.length));
        Walk { step, landings, last }
    }

    /// How far the curve travels across lines running in direction `beta`
    /// degrees: each straight run's shadow on the lines' normal, and each
    /// arc's r × ∫|sin|, in closed form.
    pub fn travel(&self, beta: f64) -> f64 {
        let b = beta.to_radians();
        let n = [-b.sin(), b.cos()];
        let psi = n[1].atan2(n[0]);
        // ∫_0^u |sin|, for any real u
        let sin_area = |u: f64| {
            let k = (u / PI).floor();
            2.0 * k + 1.0 - (u - PI * k).cos()
        };
        self.parts
            .iter()
            .map(|p| match p.seg {
                Seg::Line { from, to } => dot([to[0] - from[0], to[1] - from[1]], n).abs(),
                Seg::Arc { radius, from, to, .. } => {
                    let (lo, hi) = (from.min(to), from.max(to));
                    radius * (sin_area(hi - psi) - sin_area(lo - psi))
                }
            })
            .sum()
    }

    /// The average travel over `directions` evenly spread over half a turn
    /// (midpoint rule).
    pub fn mean_travel(&self, directions: usize) -> f64 {
        (0..directions)
            .map(|i| self.travel(180.0 * (i as f64 + 0.5) / directions as f64))
            .sum::<f64>()
            / directions as f64
    }

    /// The length measured on a sheet `gap` cm apart laid at `m` evenly
    /// spaced directions, each slid through every placement: each
    /// direction's average count (travel ÷ gap) times what one crossing is
    /// worth. The gap cancels (root's review, Mail 6787).
    pub fn fixed_directions_estimate(&self, m: usize, gap: f64) -> f64 {
        let counts: f64 = (0..m).map(|k| self.travel(180.0 * k as f64 / m as f64) / gap).sum();
        counts / m as f64 * worth(gap)
    }

    /// Every height across lines running at `beta` radians where a line's
    /// count can change: the ends, every joint, and each arc's two tangent
    /// heights (a tangent point off its arc only splits a stretch in two).
    fn change_heights(&self, beta: f64) -> Vec<f64> {
        let n = [-beta.sin(), beta.cos()];
        let mut out = vec![dot(self.at(0.0), n)];
        for part in &self.parts {
            out.push(dot(part.seg.point(1.0), n));
            if let Seg::Arc { centre, radius, .. } = part.seg {
                out.extend([dot(centre, n) + radius, dot(centre, n) - radius]);
            }
        }
        out
    }

    /// Where the line at height `p` across lines running at `beta` radians
    /// meets the curve, counted piece by piece: a straight run whose ends lie
    /// either side of it, and an arc wherever the line meets its circle at
    /// an angle inside the arc.
    pub fn meetings(&self, beta: f64, p: f64) -> usize {
        let n = [-beta.sin(), beta.cos()];
        let psi = n[1].atan2(n[0]);
        self.parts
            .iter()
            .map(|part| match part.seg {
                Seg::Line { from, to } => usize::from((dot(from, n) - p) * (dot(to, n) - p) < 0.0),
                Seg::Arc {
                    centre,
                    radius,
                    from,
                    to,
                } => {
                    let q = (p - dot(centre, n)) / radius;
                    if q.abs() >= 1.0 {
                        return 0;
                    }
                    let (lo, span) = (from.min(to), (to - from).abs());
                    [psi + q.acos(), psi - q.acos()]
                        .iter()
                        .filter(|&&t| {
                            let u = (t - lo).rem_euclid(std::f64::consts::TAU);
                            u > 0.0 && u < span
                        })
                        .count()
                }
            })
            .sum()
    }

    /// How often each count turns up, by count: for a sheet `gap` cm apart
    /// (`Some`), its chance over a direction uniform on half a turn and a
    /// slide uniform over one gap; for a single line (`None`), the area it
    /// fills on the chart of lines, cm × radians. The zero bin includes the
    /// partition padding from full-circle extrema outside their arcs; it is
    /// not the card's miss area. Use the card measure minus the positive-count
    /// area for that. A midpoint rule over
    /// `directions`; within each direction exact, every stretch between
    /// neighbouring change heights weighted by its width and counted at its
    /// middle. (`oracle.py` integrates over directions in closed form.)
    pub fn count_shares(&self, directions: usize, gap: Option<f64>) -> Vec<f64> {
        // no point of the curve lies farther than this from the card's centre
        let reach = self
            .parts
            .iter()
            .map(|part| match part.seg {
                Seg::Line { from, to } => from[0].hypot(from[1]).max(to[0].hypot(to[1])),
                Seg::Arc { centre, radius, .. } => centre[0].hypot(centre[1]) + radius,
            })
            .fold(0.0, f64::max);
        let mut out: Vec<f64> = Vec::new();
        for i in 0..directions {
            let beta = PI * (i as f64 + 0.5) / directions as f64;
            let mut hs = self.change_heights(beta);
            if let Some(g) = gap {
                hs.iter_mut().for_each(|h| *h = h.rem_euclid(g));
            }
            hs.sort_by(f64::total_cmp);
            let mut stretches: Vec<(f64, f64)> = hs.windows(2).map(|w| (w[0], w[1])).collect();
            if let Some(g) = gap {
                stretches.push((hs[hs.len() - 1], hs[0] + g));
            }
            for (lo, hi) in stretches {
                if hi - lo <= 1e-12 {
                    continue;
                }
                let at = (lo + hi) / 2.0;
                let k = match gap {
                    None => self.meetings(beta, at),
                    Some(g) => {
                        let lines = (reach / g).ceil() as i64 + 2;
                        (-lines..=lines).map(|j| self.meetings(beta, at + j as f64 * g)).sum()
                    }
                };
                if out.len() <= k {
                    out.resize(k + 1, 0.0);
                }
                out[k] += (hi - lo) * PI / directions as f64;
            }
        }
        if let Some(g) = gap {
            out.iter_mut().for_each(|w| *w /= PI * g);
        }
        out
    }

    /// Where the headland is highest, seen square to its own middle: the
    /// middle of its arc, and the outward direction there. The line through
    /// it square to that direction is tangent to the headland.
    pub fn headland_top(&self) -> (P2, P2) {
        self.middle_and_outside(HEADLAND)
    }

    /// How far each piece rises above the line through `at` square to `up`:
    /// its highest point's height (negative when the piece stays below).
    pub fn heights_above(&self, at: P2, up: P2) -> Vec<f64> {
        let h = |p: P2| dot([p[0] - at[0], p[1] - at[1]], up);
        self.parts
            .iter()
            .map(|p| match p.seg {
                Seg::Line { from, to } => h(from).max(h(to)),
                Seg::Arc {
                    centre,
                    radius,
                    from,
                    to,
                } => {
                    let ends = h(p.seg.point(0.0)).max(h(p.seg.point(1.0)));
                    // the arc's point facing `up`, if its sweep reaches it
                    let facing = up[1].atan2(up[0]);
                    let (lo, hi) = (from.min(to), from.max(to));
                    let k = ((lo - facing) / std::f64::consts::TAU).ceil();
                    let a = facing + std::f64::consts::TAU * k;
                    if a <= hi { ends.max(h(centre) + radius) } else { ends }
                }
            })
            .collect()
    }

    /// The length of straight run lying along the line through `at` in
    /// direction `along` (both ends on it), within `half` cm of `at`.
    pub fn contact_along(&self, at: P2, along: P2, half: f64) -> f64 {
        let up = [-along[1], along[0]];
        self.parts
            .iter()
            .filter_map(|p| match p.seg {
                Seg::Line { from, to } => {
                    let off = |q: P2| dot([q[0] - at[0], q[1] - at[1]], up).abs();
                    let pos = |q: P2| dot([q[0] - at[0], q[1] - at[1]], along).clamp(-half, half);
                    (off(from) < EPS && off(to) < EPS).then(|| (pos(to) - pos(from)).abs())
                }
                Seg::Arc { .. } => None,
            })
            .sum()
    }
}

/// The coast, placed on the card.
pub fn coast() -> Curve {
    Curve::build(COAST_START, COAST_HEADING, &COAST)
}

/// The length one crossing stands for on lines `gap` cm apart: a coin as
/// wide as the gap always crosses twice, and its rim is π × gap.
pub fn worth(gap: f64) -> f64 {
    PI * gap / 2.0
}

/// The mean and standard deviation of a count whose chances are `shares`
/// (indexed by count).
pub fn moments(shares: &[f64]) -> (f64, f64) {
    let mean: f64 = shares.iter().enumerate().map(|(k, p)| k as f64 * p).sum();
    let var: f64 = shares
        .iter()
        .enumerate()
        .map(|(k, p)| (k as f64 - mean).powi(2) * p)
        .sum();
    (mean, var.sqrt())
}

/// The chances of each total of `n` independent counts with these chances.
pub fn sum_shares(shares: &[f64], n: usize) -> Vec<f64> {
    let mut out = vec![1.0];
    for _ in 0..n {
        let mut next = vec![0.0; out.len() + shares.len() - 1];
        for (s, p) in out.iter().enumerate() {
            for (k, q) in shares.iter().enumerate() {
                next[s + k] += p * q;
            }
        }
        out = next;
    }
    out
}

/// The ruler under the beach (01_3): its middle on the beach's line, under
/// the beach's middle, and its direction.
pub fn ruler_on_beach(coast: &Curve) -> (P2, P2) {
    match coast.parts[BEACH].seg {
        Seg::Line { from, to } => {
            let d = dist(from, to);
            (
                [(from[0] + to[0]) / 2.0, (from[1] + to[1]) / 2.0],
                [(to[0] - from[0]) / d, (to[1] - from[1]) / d],
            )
        }
        Seg::Arc { .. } => unreachable!("the beach is a straight run"),
    }
}

/// Which side of the line a → b the point x lies: positive on the left,
/// negative on the right, zero on the line.
fn side(a: P2, b: P2, x: P2) -> f64 {
    (b[0] - a[0]) * (x[1] - a[1]) - (b[1] - a[1]) * (x[0] - a[0])
}

/// Whether the chords p–q and r–s share a point: they cross, touch, or
/// overlap along a common line. Each chord must reach the other's line (its
/// ends not both strictly on one side), and their boxes must overlap. The
/// sides alone cannot tell chords on one common line apart, since every
/// side is then zero; the boxes decide that case.
fn chords_meet(p: P2, q: P2, r: P2, s: P2) -> bool {
    let one_side = |a: f64, b: f64| (a > 0.0 && b > 0.0) || (a < 0.0 && b < 0.0);
    let overlap = |k: usize| p[k].min(q[k]) <= r[k].max(s[k]) && r[k].min(s[k]) <= p[k].max(q[k]);
    overlap(0) && overlap(1) && !one_side(side(p, q, r), side(p, q, s)) && !one_side(side(r, s, p), side(r, s, q))
}

/// How many pairs of a polyline's chords meet, neighbours (which share a
/// point by construction) apart.
fn self_crossings(points: &[P2]) -> usize {
    let n = points.len() - 1;
    let mut meets = 0;
    for i in 0..n {
        for j in i + 2..n {
            if chords_meet(points[i], points[i + 1], points[j], points[j + 1]) {
                meets += 1;
            }
        }
    }
    meets
}

/// Whether the drawn coast meets itself: its polyline, points at most 0.04
/// cm apart along each arc, checked chord against chord. This checks that
/// drawing, not the exact curve: parts of a curve that only touch, or pass
/// closer than a chord strays from its arc, can be missed (two circles that
/// only touch were, in root's native01; `a_curve_that_crosses_itself_is_caught`).
pub fn crosses_itself(coast: &Curve) -> bool {
    self_crossings(&coast.upto(coast.length, 0.04)) > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn circle(r: i64) -> Curve {
        Curve::build(
            [Ratio::whole(0), Ratio::whole(-r)],
            0,
            &[Piece::Arc {
                radius: Ratio::whole(r),
                turn: 360,
            }],
        )
    }

    #[test]
    fn ratios_reduce_and_print_as_fractions_do() {
        assert_eq!(Ratio::new(6, -4), Ratio::new(-3, 2));
        assert_eq!(Ratio::new(1, 2).plus(Ratio::new(1, 3)).to_string(), "5/6");
        assert_eq!(Ratio::new(3, 4).times(Ratio::new(8, 3)).to_string(), "2");
        assert_eq!(Ratio::new(0, 7).to_string(), "0");
    }

    #[test]
    fn the_coast_is_exactly_fifteen_halves_plus_1399_240ths_of_pi() {
        let (straight, pi_part) = exact_length(&COAST);
        assert_eq!(
            (straight.to_string(), pi_part.to_string()),
            ("15/2".into(), "1399/240".into())
        );
        let c = coast();
        assert!((c.length - (straight.value() + pi_part.value() * PI)).abs() < 1e-12);
        // and the drawn polyline's chords fall short of it by no more than
        // its arcs allow (0.0011 cm, at 0.04 cm apart)
        let pts = c.upto(c.length, 0.04);
        let chords: f64 = pts.windows(2).map(|w| dist(w[0], w[1])).sum();
        assert!(chords <= c.length + 1e-12, "{chords}");
        assert!(c.length - chords <= chord_shortfall(&c, 0.04), "{chords}");
    }

    /// How far a polyline with points `every` cm apart along each arc can
    /// fall short of the curve: a chord across angle δ of a circle of radius
    /// r is short by r(δ − 2 sin(δ/2)) ≤ rδ³/24, and δ ≤ every/r.
    fn chord_shortfall(c: &Curve, every: f64) -> f64 {
        c.parts
            .iter()
            .map(|p| match p.seg {
                Seg::Arc { radius, .. } => p.seg.len() * every * every / (24.0 * radius * radius),
                Seg::Line { .. } => 0.0,
            })
            .sum()
    }

    #[test]
    fn the_pieces_join_and_keep_their_heading() {
        let c = coast();
        for w in c.parts.windows(2) {
            let (a, b) = (w[0].seg, w[1].seg);
            assert!(dist(a.point(1.0), b.point(0.0)) < 1e-12);
            let tangent = |s: Seg, u: f64| {
                let (p, q) = (s.point(u - 1e-7), s.point(u + 1e-7));
                let d = dist(p, q);
                [(q[0] - p[0]) / d, (q[1] - p[1]) / d]
            };
            let (ta, tb) = (tangent(a, 1.0 - 1e-6), tangent(b, 1e-6));
            assert!(dot(ta, tb) > 1.0 - 1e-6, "a corner between pieces");
        }
    }

    #[test]
    fn the_coast_stays_inside_the_card_and_never_meets_itself() {
        let c = coast();
        for p in c.upto(c.length, 0.02) {
            assert!(
                p[0].abs() < CARD_W / 2.0 - 0.25 && p[1].abs() < CARD_H / 2.0 - 0.25,
                "{p:?}"
            );
        }
        assert!(!crosses_itself(&c));
    }

    /// Chords meet when they cross, when an end of one touches the other
    /// (a curve that touches itself is not one simple curve), and when they
    /// overlap along a common line; not when they are apart, on one line or
    /// otherwise. Each case holds whichever way round the chords are taken.
    #[test]
    fn chords_meet_only_when_they_share_a_point() {
        // each chord as its ends' coordinates, x0 y0 x1 y1
        let cases: [(&str, [f64; 4], [f64; 4], bool); 12] = [
            ("bow-tie", [0.0, 0.0, 2.0, 2.0], [0.0, 2.0, 2.0, 0.0], true),
            (
                "an end on the other's middle",
                [0.0, 0.0, 2.0, 0.0],
                [1.0, 0.0, 1.0, 1.0],
                true,
            ),
            ("a shared end", [0.0, 0.0, 1.0, 0.0], [1.0, 0.0, 1.0, 1.0], true),
            (
                "on one line, overlapping",
                [0.0, 0.0, 2.0, 0.0],
                [1.0, 0.0, 3.0, 0.0],
                true,
            ),
            (
                "on one line, end to end",
                [0.0, 0.0, 1.0, 0.0],
                [1.0, 0.0, 2.0, 0.0],
                true,
            ),
            ("on one line, apart", [0.0, 0.0, 1.0, 0.0], [2.0, 0.0, 3.0, 0.0], false),
            (
                "on one slanting line, apart",
                [0.0, 0.0, 1.0, 1.0],
                [2.0, 2.0, 3.0, 3.0],
                false,
            ),
            (
                "on one upright line, apart",
                [0.0, 0.0, 0.0, 1.0],
                [0.0, 2.0, 0.0, 3.0],
                false,
            ),
            ("parallel", [0.0, 0.0, 1.0, 0.0], [0.0, 1.0, 1.0, 1.0], false),
            (
                "their lines meet beyond them",
                [0.0, 0.0, 1.0, 0.0],
                [2.0, -1.0, 2.0, 1.0],
                false,
            ),
            ("a near miss", [0.0, 0.0, 1.0, 0.0], [1.001, -1.0, 1.001, 1.0], false),
            (
                "an end on the other's line, beyond it",
                [0.0, 0.0, 1.0, 0.0],
                [2.0, 0.0, 2.0, 1.0],
                false,
            ),
        ];
        for (name, one, other, meet) in cases {
            let (p, q, r, s) = (
                [one[0], one[1]],
                [one[2], one[3]],
                [other[0], other[1]],
                [other[2], other[3]],
            );
            for (a, b, c, d) in [(p, q, r, s), (q, p, r, s), (p, q, s, r), (r, s, p, q), (s, r, q, p)] {
                assert_eq!(chords_meet(a, b, c, d), meet, "{name}");
            }
        }
        // chords on one line, apart: every side is zero, so the sides alone
        // (the check before root's native01 review) counted them as meeting
        let (p, q, r, s) = ([0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0]);
        assert_eq!([side(p, q, r), side(p, q, s), side(r, s, p), side(r, s, q)], [0.0; 4]);
    }

    /// A polyline's meetings are counted pair by pair, neighbours apart: a
    /// bow-tie crosses once, a path that comes back to touch itself meets
    /// once, and a path that comes back along its first chord's line but
    /// stops short of that chord meets none (the sides alone counted it once).
    #[test]
    fn a_polyline_counts_each_meeting_once() {
        let bow = [[0.0, 0.0], [2.0, 2.0], [2.0, 0.0], [0.0, 2.0]];
        let touch = [[0.0, 0.0], [2.0, 0.0], [2.0, 1.0], [1.0, 1.0], [1.0, 0.0]];
        let turn_back = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [3.0, 1.0], [3.0, 0.0], [2.0, 0.0]];
        let open = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [0.0, 0.5]];
        assert_eq!(self_crossings(&bow), 1);
        assert_eq!(self_crossings(&touch), 1);
        assert_eq!(self_crossings(&turn_back), 0);
        assert_eq!(self_crossings(&open), 0);
    }

    /// A curve that does cross itself is caught: a run east, three quarters
    /// of a turn left, and a run south back through the first run, crossing
    /// it once, well inside both chords. (This replaces two full circles
    /// turning opposite ways, which failed in root's native01: they only
    /// touch at the start, where the first and last chords end 1e-16 apart
    /// in floating point, so whether they meet is decided by rounding.)
    #[test]
    fn a_curve_that_crosses_itself_is_caught() {
        let curl = Curve::build(
            [Ratio::whole(0), Ratio::whole(0)],
            0,
            &[
                Piece::Straight(Ratio::whole(3)),
                Piece::Arc {
                    radius: Ratio::whole(1),
                    turn: 270,
                },
                Piece::Straight(Ratio::whole(3)),
            ],
        );
        let pts = curl.upto(curl.length, 0.04);
        assert_eq!(self_crossings(&pts), 1);
        assert!(crosses_itself(&curl));
        // the two runs cross at (2, 0), every end well off the other's line
        let (n, end) = (pts.len() - 1, curl.ends().1);
        assert!((end[0] - 2.0).abs() < 1e-12 && (end[1] + 2.0).abs() < 1e-12, "{end:?}");
        let (p, q, r, s) = (pts[0], pts[1], pts[n - 1], pts[n]);
        assert!(
            [side(p, q, r), side(p, q, s), side(r, s, p), side(r, s, q)]
                .iter()
                .all(|d| d.abs() > 1.0)
        );
    }

    /// A stretch of the coast drawn between two points along it (01_6's
    /// close-up of step 6) starts and ends there, and its chords come to the
    /// coast between them, never more.
    #[test]
    fn a_stretch_of_coast_runs_between_its_two_points() {
        let c = coast();
        let walk = c.walk(2.0);
        let step_6 = (walk.landings[5].1, walk.landings[6].1);
        for (from, to) in [step_6, (0.0, c.length), (3.0, 3.5), (12.0, 12.0)] {
            let pts = c.between(from, to, 0.04);
            assert!(dist(pts[0], c.at(from)) < 1e-12, "{from}..{to} starts off");
            assert!(dist(pts[pts.len() - 1], c.at(to)) < 1e-12, "{from}..{to} ends off");
            let chords: f64 = pts.windows(2).map(|w| dist(w[0], w[1])).sum();
            assert!(chords <= to - from + 1e-12, "{from}..{to}: {chords}");
            assert!(
                to - from - chords <= chord_shortfall(&c, 0.04),
                "{from}..{to}: {chords}"
            );
        }
    }

    #[test]
    fn dividers_walk_a_straight_run_exactly() {
        let stick = Curve::build(
            [Ratio::whole(0), Ratio::whole(0)],
            37,
            &[Piece::Straight(Ratio::whole(6))],
        );
        for (step, steps) in [(0.5, 12), (1.0, 6), (2.5, 2), (7.0, 0)] {
            let w = stick.walk(step);
            assert_eq!(w.steps(), steps, "step {step}");
            assert!((w.length() - 6.0).abs() < 1e-9, "step {step}: {}", w.length());
        }
    }

    #[test]
    fn dividers_set_to_the_radius_walk_a_hexagon_round_a_circle() {
        let w = circle(1).walk(1.0);
        assert_eq!(w.steps(), 6, "the sixth step lands back on the start");
        assert!(w.last < 1e-9 && (w.length() - 6.0).abs() < 1e-9);
        for (k, (p, s)) in w.landings.iter().enumerate() {
            assert!((s - PI / 3.0 * k as f64).abs() < 1e-9, "landing {k} at {s}");
            assert!((p[0].hypot(p[1]) - 1.0).abs() < 1e-12);
        }
        // a step a little over the radius leaves a last straight bit
        let w = circle(1).walk(1.1);
        assert_eq!(w.steps(), 5);
        assert!(w.length() < 2.0 * PI);
    }

    #[test]
    fn every_step_is_its_step_long_and_never_longer_than_the_coast_it_skips() {
        let c = coast();
        for step in [0.3, 0.5, 1.0, 1.7, 2.0, 3.0, 4.0] {
            let w = c.walk(step);
            for pair in w.landings.windows(2) {
                let ((p, s), (q, t)) = (pair[0], pair[1]);
                assert!((dist(p, q) - step).abs() < 1e-9, "step {step}");
                assert!(t - s >= step - 1e-9, "step {step}: {} cm of coast", t - s);
            }
            assert!(w.last <= c.length - w.landings[w.landings.len() - 1].1 + 1e-9);
            assert!(w.length() < c.length, "dividers of {step} cm came out long");
        }
        // on this coast the four steps the film walks give longer totals
        // as the step shrinks (true here, not a theorem)
        let totals: Vec<f64> = [4.0, 2.0, 1.0, 0.5].iter().map(|&s| c.walk(s).length()).collect();
        assert!(totals.windows(2).all(|t| t[0] < t[1]), "{totals:?}");
    }

    /// 01_7: each smaller step cuts less off its bend. A step c across an arc
    /// of radius r spans the angle 2 asin(c / 2r) and cuts off
    /// 2 r asin(c / 2r) − c, which grows with c, on every arc of the coast.
    /// The whole walk need not grow as the step shrinks (the review's F4):
    /// 1.9 cm dividers take as many steps as 2 cm ones and walk less.
    #[test]
    fn a_smaller_step_cuts_less_off_each_bend_but_need_not_walk_further() {
        let cut = |r: f64, c: f64| 2.0 * r * (c / (2.0 * r)).asin() - c;
        for part in coast().parts {
            if let Seg::Arc { radius, from, to, .. } = part.seg {
                // the longest step the arc holds end to end
                let longest = 2.0 * radius * ((to - from).abs().min(PI) / 2.0).sin();
                let steps: Vec<f64> = (1..=20).map(|k| longest * k as f64 / 20.0).collect();
                assert!(
                    steps.windows(2).all(|w| cut(radius, w[0]) < cut(radius, w[1])),
                    "radius {radius}"
                );
            }
        }
        let c = coast();
        let (smaller, opening) = (c.walk(1.9), c.walk(2.0));
        assert_eq!(smaller.steps(), opening.steps());
        assert!(
            smaller.length() < opening.length(),
            "{} against {}",
            smaller.length(),
            opening.length()
        );
    }

    /// `count_shares` on shapes whose chances are known on paper.
    #[test]
    fn count_shares_match_the_coin_buffons_needle_and_the_circle() {
        // a coin as wide as the gap crosses twice at every placement
        let coin = circle(1).count_shares(360, Some(2.0));
        assert!(
            (coin[2] - 1.0).abs() < 1e-12 && (coin.iter().sum::<f64>() - 1.0).abs() < 1e-12,
            "{coin:?}"
        );
        // a stick shorter than the gap crosses once with chance
        // 2 x length / (pi x gap) (Buffon), else not at all
        let needle = Curve::build(
            [Ratio::whole(0), Ratio::whole(0)],
            23,
            &[Piece::Straight(Ratio::new(3, 2))],
        )
        .count_shares(1800, Some(2.0));
        assert_eq!(needle.len(), 2);
        assert!((needle[1] - 3.0 / (2.0 * PI)).abs() < 1e-5, "{needle:?}");
        // every line meeting a circle crosses it twice, and those lines fill
        // the chart over its rim
        let lines = circle(3).count_shares(360, None);
        assert_eq!(lines.len(), 3);
        assert!((lines[2] - 6.0 * PI).abs() < 1e-9 && lines[1] < 1e-12, "{lines:?}");
    }

    /// Over every placement the coast's mean count is Crofton's
    /// 2L/(pi x gap) at every gap, and its single lines' counts total twice
    /// its length; one direction alone is not every placement (a negative
    /// control on a stick).
    #[test]
    fn count_shares_average_to_crofton_only_over_every_direction() {
        let c = coast();
        for gap in [2.0, 4.0, 8.0] {
            let shares = c.count_shares(1800, Some(gap));
            assert!((shares.iter().sum::<f64>() - 1.0).abs() < 1e-9);
            let (mean, _) = moments(&shares);
            assert!((mean - 2.0 * c.length / (PI * gap)).abs() < 1e-5, "gap {gap}: {mean}");
        }
        let lines = c.count_shares(1800, None);
        let total: f64 = lines.iter().enumerate().map(|(k, w)| k as f64 * w).sum();
        assert!((total - 2.0 * c.length).abs() < 1e-4, "{total}");
        assert_eq!(lines.len() - 1, 6, "a single line crosses the coast at most six times");
        // a 6 cm stick along x: lines straight across it (the one midpoint
        // direction of a one-direction grid) average 3, every direction 6/pi
        let stick = Curve::build(
            [Ratio::whole(0), Ratio::whole(0)],
            0,
            &[Piece::Straight(Ratio::whole(6))],
        );
        let (across, _) = moments(&stick.count_shares(1, Some(2.0)));
        let (turned, _) = moments(&stick.count_shares(1800, Some(2.0)));
        assert!((across - 3.0).abs() < 1e-9, "{across}");
        assert!((turned - 6.0 / PI).abs() < 1e-5, "{turned}");
    }

    #[test]
    fn sums_of_counts_convolve() {
        // two fair coins of 0 or 1: totals 0, 1, 2 with chances 1/4, 1/2, 1/4
        assert_eq!(sum_shares(&[0.5, 0.5], 2), vec![0.25, 0.5, 0.25]);
        assert_eq!(moments(&[0.25, 0.5, 0.25]), (1.0, 0.5f64.sqrt()));
    }

    #[test]
    fn a_circle_travels_twice_its_width_whichever_way_the_lines_run() {
        let c = circle(3);
        for beta in [0.0, 17.0, 45.0, 90.0, 133.0] {
            assert!((c.travel(beta) - 12.0).abs() < 1e-9, "{beta}");
        }
        let stick = Curve::build(
            [Ratio::whole(0), Ratio::whole(0)],
            30,
            &[Piece::Straight(Ratio::whole(6))],
        );
        assert!(
            (stick.travel(0.0) - 3.0).abs() < 1e-12,
            "a stick at 30 degrees reaches half its length"
        );
        let shore = coast();
        for beta in [0.0, 45.0, 90.0, 135.0] {
            assert!((shore.travel(beta) - shore.travel(beta + 180.0)).abs() < 1e-9);
        }
    }

    #[test]
    fn averaged_over_every_direction_the_travel_is_the_length_times_two_over_pi() {
        for curve in [coast(), circle(1)] {
            let mean = curve.mean_travel(7200);
            assert!((mean * PI / 2.0 - curve.length).abs() < 1e-6, "{mean}");
        }
    }

    /// Root's review (Mail 6787): the estimate must not depend on the
    /// sheet's spacing. A count goes as 1/gap and a crossing's worth as gap.
    #[test]
    fn the_gap_cancels_from_every_estimate() {
        let c = coast();
        let at_two = c.fixed_directions_estimate(6, 2.0);
        for gap in [0.5, 1.0, 3.0, 4.0] {
            assert!((c.fixed_directions_estimate(6, gap) - at_two).abs() < 1e-9, "gap {gap}");
        }
        let ring = circle(1);
        for gap in [0.5, 1.0, 2.0, 4.0] {
            assert!(
                (ring.fixed_directions_estimate(4, gap) - 2.0 * PI).abs() < 1e-9,
                "gap {gap}"
            );
        }
        // negative control: the expression the review flagged is right
        // only on a 2 cm sheet
        for gap in [0.5, 1.0, 2.0, 4.0] {
            let flagged = PI * ring.mean_travel(720) / gap;
            assert_eq!((flagged - 2.0 * PI).abs() < 1e-6, gap == 2.0, "gap {gap}: {flagged}");
        }
    }

    #[test]
    fn the_ruler_fits_the_beach_and_touches_the_headland_once() {
        let c = coast();
        let (at, along) = ruler_on_beach(&c);
        assert!((c.contact_along(at, along, RULER / 2.0) - 3.5).abs() < 1e-12);
        // within the ruler's span the coast stays on its upper side; the
        // same ruler slid along its line under the bay would cut the coast
        let up = [-along[1], along[0]];
        let lowest = |from: P2| {
            c.upto(c.length, 0.01)
                .iter()
                .map(|&q| [q[0] - from[0], q[1] - from[1]])
                .filter(|&d| dot(d, along).abs() <= RULER / 2.0)
                .map(|d| dot(d, up))
                .fold(f64::INFINITY, f64::min)
        };
        assert!(lowest(at) >= -1e-9, "{}", lowest(at));
        let (bay, _) = c.middle_and_outside(BAY);
        // (the bay's bottom is 0.033 cm under the beach's line)
        assert!(lowest([bay[0], at[1]]) < -0.02, "{}", lowest([bay[0], at[1]]));
        let (top, out) = c.headland_top();
        let heights = c.heights_above(top, out);
        let touching: Vec<usize> = (0..heights.len()).filter(|&i| heights[i] > -1e-9).collect();
        assert_eq!(touching, vec![HEADLAND], "{heights:?}");
        assert!(
            heights.iter().all(|&h| h < 1e-9),
            "nothing rises above the headland's top"
        );
        // and the headland meets it at one point: half a centimetre either
        // way along the ruler, the coast is already well below
        for s in [-0.5, 0.5] {
            let p = [top[0] - out[1] * s, top[1] + out[0] * s];
            let below = c
                .upto(c.length, 0.01)
                .iter()
                .map(|&q| dist(p, q))
                .fold(f64::INFINITY, f64::min);
            assert!(below > 0.1, "{below}");
        }
    }

    #[test]
    fn the_headland_tangent_check_can_fail() {
        // a line through the headland's top but tilted 20 degrees cuts it
        let c = coast();
        let (top, out) = c.headland_top();
        let a = 20f64.to_radians();
        let tilted = [out[0] * a.cos() - out[1] * a.sin(), out[0] * a.sin() + out[1] * a.cos()];
        assert!(c.heights_above(top, tilted)[HEADLAND] > 0.01);
    }
}
