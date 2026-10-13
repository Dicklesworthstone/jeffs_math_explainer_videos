//! The mathematics of Video H, as the video works it.
//!
//! - [`Q`]: exact fractions, so every chance in the hand examples is a
//!   fraction and never a rounded float.
//! - [`sea`]: the four-cell sea. Two weighted stories make the map; a search
//!   sees what is there with a chance that depends on the cell; an empty
//!   search updates the map by Bayes' rule; four rules for where to search
//!   next, and what each has found by any number of searches.
//! - [`y1968`]: the prior map of the 1968 search for the USS Scorpion, the
//!   10,000 Monte Carlo points per cell that Richardson and Stone printed
//!   as their Figure 2, and a model search over it.
//! - [`koopman`]: the random search formula, and seeded lanes laid over a
//!   cell without regard to what is already swept.
//! - [`drift`]: a seeded cloud of dots that moves and spreads, and what one
//!   flight buys as the hours pass.
//! - [`days`]: day counts between the dates the video names.
//!
//! Chapters only draw; `facts.rs` prints what is here, and `oracle.py`
//! gets the same numbers another way (counting dots, a second transcription
//! of the map, NumPy on the same seeds).

use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Div, Mul, Sub};

use explainer_kit::stats::Data;

// --------------------------------------------------------------------------
// Exact fractions
// --------------------------------------------------------------------------

/// A fraction in lowest terms with a positive denominator.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Q {
    n: i128,
    d: i128,
}

fn gcd(a: i128, b: i128) -> i128 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

impl Q {
    pub const ZERO: Q = Q { n: 0, d: 1 };
    pub const ONE: Q = Q { n: 1, d: 1 };

    /// `n / d`, reduced.
    ///
    /// # Panics
    /// If `d` is zero.
    pub fn new(n: i128, d: i128) -> Q {
        assert!(d != 0, "a fraction with denominator zero");
        let g = gcd(n, d).max(1);
        let s = if d < 0 { -1 } else { 1 };
        Q {
            n: s * n / g,
            d: s * d / g,
        }
    }

    pub fn int(n: i128) -> Q {
        Q { n, d: 1 }
    }

    /// A constant already in lowest terms with a positive denominator
    /// (a test checks every one).
    pub const fn lowest(n: i128, d: i128) -> Q {
        Q { n, d }
    }

    pub fn f64(self) -> f64 {
        self.n as f64 / self.d as f64
    }

    pub fn pow(self, k: u32) -> Q {
        (0..k).fold(Q::ONE, |acc, _| acc * self)
    }

    /// Decimal with exactly `places` places, rounded half up (never half to
    /// even: the oracle rounds the same exact fraction the same way).
    ///
    /// # Panics
    /// If the value is negative (nothing the video prints is).
    pub fn fixed(self, places: u32) -> String {
        assert!(self.n >= 0, "fixed() prints non-negative values");
        let scale = 10i128.pow(places);
        let scaled = (2 * self.n * scale + self.d) / (2 * self.d);
        if places == 0 {
            return scaled.to_string();
        }
        format!("{}.{:0width$}", scaled / scale, scaled % scale, width = places as usize)
    }

    /// As a percent with `places` places, rounded half up.
    pub fn percent(self, places: u32) -> String {
        (self * Q::int(100)).fixed(places)
    }
}

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
        Q::new(self.n * o.d + o.n * self.d, self.d * o.d)
    }
}
impl Sub for Q {
    type Output = Q;
    fn sub(self, o: Q) -> Q {
        Q::new(self.n * o.d - o.n * self.d, self.d * o.d)
    }
}
impl Mul for Q {
    type Output = Q;
    fn mul(self, o: Q) -> Q {
        Q::new(self.n * o.n, self.d * o.d)
    }
}
impl Div for Q {
    type Output = Q;
    fn div(self, o: Q) -> Q {
        Q::new(self.n * o.d, self.d * o.n)
    }
}
impl PartialOrd for Q {
    fn partial_cmp(&self, o: &Q) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}
impl Ord for Q {
    fn cmp(&self, o: &Q) -> Ordering {
        (self.n * o.d).cmp(&(o.n * self.d))
    }
}

/// `n / d` for literals.
pub fn q(n: i128, d: i128) -> Q {
    Q::new(n, d)
}

// --------------------------------------------------------------------------
// The four-cell sea
// --------------------------------------------------------------------------

pub mod sea {
    use super::{Q, q};

    /// The cells, in reading order.
    pub const CELLS: [&str; 4] = ["A", "B", "C", "D"];
    /// The dots the video draws: one hundred places the wreck could be.
    pub const DOTS: i128 = 100;

    /// One story of what happened: its weight, and the share of its dots
    /// that land in each cell.
    pub struct Story {
        pub weight: Q,
        pub share: [Q; 4],
    }

    /// The video's two stories (its own illustration, not the Navy's nine):
    /// "she kept her course" weighs 60 and puts half its dots in A, a third
    /// in B and a sixth in C; "something turned her" weighs 40 and spreads
    /// evenly over the four cells.
    pub fn stories() -> [Story; 2] {
        [
            Story {
                weight: q(60, 100),
                share: [q(1, 2), q(1, 3), q(1, 6), Q::ZERO],
            },
            Story {
                weight: q(40, 100),
                share: [q(1, 4); 4],
            },
        ]
    }

    /// The dots each story puts in each cell.
    pub fn story_dots(s: &Story) -> [Q; 4] {
        s.share.map(|x| x * s.weight * Q::int(DOTS))
    }

    /// The map before any search: each cell's chance is the weighted sum of
    /// the stories' shares.
    pub fn prior() -> [Q; 4] {
        let mut p = [Q::ZERO; 4];
        for s in stories() {
            for (cell, share) in p.iter_mut().zip(s.share) {
                *cell = *cell + s.weight * share;
            }
        }
        p
    }

    /// The chance that one search of a cell sees the wreck if it is there:
    /// A is a canyon (one time in four); B, C and D are flat (two in three).
    pub fn seeing() -> [Q; 4] {
        [q(1, 4), q(2, 3), q(2, 3), q(2, 3)]
    }

    /// The chance that a search of each cell finds the wreck: the chance it
    /// is there times the chance of seeing it.
    pub fn find(p: &[Q; 4]) -> [Q; 4] {
        let d = seeing();
        [p[0] * d[0], p[1] * d[1], p[2] * d[2], p[3] * d[3]]
    }

    /// The map after a search of `cell` comes back empty (Bayes' rule): the
    /// searched cell keeps only its misses, and every cell is divided by
    /// the chance that the search would come back empty.
    pub fn update(p: &[Q; 4], cell: usize) -> [Q; 4] {
        bayes(p, cell, seeing()[cell])
    }

    /// The same update for a search that sees what is in `cell` with
    /// chance `seen`, whatever the cell's terrain.
    pub fn bayes(p: &[Q; 4], cell: usize, seen: Q) -> [Q; 4] {
        let empty = Q::ONE - p[cell] * seen;
        let mut out = p.map(|x| x / empty);
        out[cell] = p[cell] * (Q::ONE - seen) / empty;
        out
    }

    /// Where to search next.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Rule {
        /// The cell with the greatest chance of a find (ties: first in
        /// reading order). The video's rule.
        Best,
        /// The cell with the greatest chance of holding the wreck (ties:
        /// first in reading order).
        Likeliest,
        /// The same, with ties going to the cell searched fewer times.
        LikeliestFresh,
        /// A, B, C, D, and round again.
        Sweep,
    }

    fn argmax<K: Ord>(keys: impl IntoIterator<Item = K>) -> usize {
        let mut best: Option<(usize, K)> = None;
        for (i, k) in keys.into_iter().enumerate() {
            if best.as_ref().is_none_or(|(_, b)| k > *b) {
                best = Some((i, k));
            }
        }
        best.expect("no cells").0
    }

    impl Rule {
        pub fn choose(self, p: &[Q; 4], step: usize, searched: &[u32; 4]) -> usize {
            match self {
                Rule::Best => argmax(find(p)),
                Rule::Likeliest => argmax(*p),
                Rule::LikeliestFresh => argmax((0..4).map(|i| (p[i], std::cmp::Reverse(searched[i])))),
                Rule::Sweep => step % 4,
            }
        }
    }

    /// One search of a run in which every search so far came back empty.
    #[derive(Clone, Debug)]
    pub struct Step {
        pub cell: usize,
        /// The chance this search finds the wreck, given the map it starts from.
        pub find: Q,
        /// The chance the wreck has been found by the end of this search.
        pub found_by: Q,
        /// The map if this search too comes back empty.
        pub after: [Q; 4],
    }

    /// `steps` searches under `rule`, from the prior.
    pub fn run(rule: Rule, steps: usize) -> Vec<Step> {
        let mut p = prior();
        let mut searched = [0u32; 4];
        let (mut unfound, mut out) = (Q::ONE, Vec::with_capacity(steps));
        for step in 0..steps {
            let cell = rule.choose(&p, step, &searched);
            let find = find(&p)[cell];
            let found_by = Q::ONE - unfound * (Q::ONE - find);
            unfound = Q::ONE - found_by;
            p = update(&p, cell);
            searched[cell] += 1;
            out.push(Step {
                cell,
                find,
                found_by,
                after: p,
            });
        }
        out
    }

    /// The chance the wreck is found within the searches of `order`, by
    /// the update rule.
    pub fn found_by(order: &[usize]) -> Q {
        let (mut p, mut unfound) = (prior(), Q::ONE);
        for &cell in order {
            unfound = unfound * (Q::ONE - find(&p)[cell]);
            p = update(&p, cell);
        }
        Q::ONE - unfound
    }

    /// The best any order of `n` searches can do, by trying every order.
    pub fn best_of_all_orders(n: usize) -> Q {
        let mut best = Q::ZERO;
        let mut order = vec![0usize; n];
        loop {
            best = best.max(found_by(&order));
            let mut i = 0;
            while i < n && order[i] == 3 {
                order[i] = 0;
                i += 1;
            }
            if i == n {
                return best;
            }
            order[i] += 1;
        }
    }

    /// The average number of searches until the wreck is found: the sum,
    /// over n from 0 up, of the chance it is still unfound after n
    /// searches. In floats, to a remainder below 1e-13. Two cells whose
    /// keys agree to one part in 10^9 are tied (the exact values are then
    /// equal fractions), and the rule's own tie-break decides.
    pub fn expected_searches(rule: Rule) -> f64 {
        let d = seeing().map(Q::f64);
        // unnormalized: w[i] is the chance the wreck is in cell i and unfound
        let mut w = prior().map(Q::f64);
        let mut searched = [0u32; 4];
        let (mut sum, mut step) = (0.0, 0usize);
        loop {
            let unfound: f64 = w.iter().sum();
            if unfound < 1e-13 {
                return sum;
            }
            sum += unfound;
            let key: [f64; 4] = match rule {
                Rule::Best => [w[0] * d[0], w[1] * d[1], w[2] * d[2], w[3] * d[3]],
                Rule::Likeliest | Rule::LikeliestFresh | Rule::Sweep => w,
            };
            let top = key.iter().copied().fold(f64::MIN, f64::max);
            let tied = (0..4).filter(|&i| key[i] >= top * (1.0 - 1e-9));
            let cell = match rule {
                Rule::Sweep => step % 4,
                Rule::LikeliestFresh => tied.min_by_key(|&i| searched[i]).expect("no cells"),
                Rule::Best | Rule::Likeliest => tied.min().expect("no cells"),
            };
            w[cell] *= 1.0 - d[cell];
            searched[cell] += 1;
            step += 1;
        }
    }

    /// The chance that `searches` searches of one cell see what is there,
    /// each seeing it with chance `each`.
    pub fn seen_within(each: Q, searches: u32) -> Q {
        Q::ONE - (Q::ONE - each).pow(searches)
    }
}

// --------------------------------------------------------------------------
// The 1968 map
// --------------------------------------------------------------------------

pub mod y1968 {
    use super::{Q, q};

    /// The printed lattice's columns, west to east: three unlettered
    /// columns, the search grid's A to O, two more unlettered columns.
    pub const COLUMNS: [&str; 20] = [
        "w3", "w2", "w1", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "e1", "e2",
    ];
    /// Its rows, north to south: two above the search grid's 1 to 12, two below.
    pub const ROWS: [&str; 16] = [
        "n2", "n1", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "s1", "s2",
    ];

    /// Richardson and Stone (1971), Figure 2, p. 145, "Overall A Priori
    /// distribution for Scorpion search": of 10,000 Monte Carlo
    /// replications, how many ended in each cell ("convert numbers to
    /// probabilities by dividing by 10,000"). Blank cells are 0.
    #[rustfmt::skip]
    pub const POINTS: [[u16; 20]; 16] = [
        //w3  w2   w1    A     B    C    D     E    F    G    H    I    J   K   L   M   N   O  e1 e2
        [  0,   0,   0,   0,    0,   0,   0,    0,   0,   0,   0,   0,   0,  0,  0,  0,  5,  7, 1, 0], // n2
        [  0,   0,   0,   0,    0,   0,   0,    0,   0,   0,   0,   0,   0,  0,  3, 11, 14, 24, 6, 0], // n1
        [  0,   0,   0,   0,    0,   0,   0,    0,   0,   0,   0,   0,   0,  5, 26, 35, 22, 26, 9, 1], // 1
        [  0,   0,   0,   0,    0,   0,   0,    0,   0,   0,   0,   0,  18, 46, 74, 42, 18, 10, 4, 2], // 2
        [  0,   0,   0,   0,    0,   0,   0,    0,   0,   0,   8,  60, 140, 99, 45, 20,  4,  2, 1, 1], // 3
        [  0,   0,   0,   2,   21, 137,  16,    7,   1,  20, 215, 239, 105, 30,  5,  3,  1,  1, 1, 0], // 4
        [  0,   0,  18,  40,   46, 747,  30, 1250, 205, 571, 277,  38,   5,  2,  1,  1,  1,  0, 0, 0], // 5
        [  0,  14, 326,   3,    1,  28,  31,   63,  85,  62,   1,   8,   7, 10,  7,  3,  4,  0, 0, 0], // 6
        [  1, 359, 175, 174, 1096, 282, 245,   82,  71,  65,  35,  27,   9, 12,  6,  5,  4,  0, 0, 0], // 7
        [  0,  24,  25,  42,   82, 297, 230,  129, 115,  61,  33,  14,  14, 10,  6,  2,  5,  1, 0, 0], // 8
        [  0,  17,  25,  20,   20,  20,  19,   55,  99,  46,  30,  14,  15,  3,  5,  1,  6,  0, 0, 0], // 9
        [  0,   2,  13,  14,   25,  20,  24,   45,  34,  27,  19,  15,   5,  7,  5,  5,  1,  0, 0, 0], // 10
        [  0,   0,   0,   7,   13,  12,   9,    1,   3,   3,  11,  14,   5,  4,  3,  2,  1,  0, 0, 0], // 11
        [  0,   0,   0,   0,    0,   0,   0,    0,   0,   1,   4,   4,  10,  5,  4,  1,  0,  0, 0, 0], // 12
        [  0,   0,   0,   0,    0,   0,   0,    0,   0,   0,   0,   1,   3,  2,  0,  0,  0,  0, 0, 0], // s1
        [  0,   0,   0,   0,    0,   0,   0,    0,   0,   0,   0,   0,   3,  2,  0,  0,  0,  0, 0, 0], // s2
    ];

    /// The replications the figure counts.
    pub const REPLICATIONS: u32 = 10_000;
    /// The search grid proper: columns A to O (indices 3 to 17), rows 1 to
    /// 12 (indices 2 to 13).
    pub const GRID_COLS: std::ops::Range<usize> = 3..18;
    pub const GRID_ROWS: std::ops::Range<usize> = 2..14;
    /// A cell's sides in miles, north-south by east-west (p. 143).
    pub const CELL_MILES: (Q, Q) = (Q::ONE, Q { n: 21, d: 25 });

    /// (row, column) of a cell named as the paper names it: "E5".
    pub fn at(name: &str) -> Option<(usize, usize)> {
        COLUMNS.iter().enumerate().find_map(|(c, col)| {
            let row = name.strip_prefix(col)?;
            Some((ROWS.iter().position(|r| *r == row)?, c))
        })
    }

    /// A cell's name as the paper writes it.
    pub fn name(row: usize, col: usize) -> String {
        format!("{}{}", COLUMNS[col], ROWS[row])
    }

    pub fn points(name: &str) -> Option<u32> {
        at(name).map(|(r, c)| u32::from(POINTS[r][c]))
    }

    /// Every cell with its points, in reading order.
    pub fn cells() -> impl Iterator<Item = (usize, usize, u32)> {
        POINTS
            .iter()
            .enumerate()
            .flat_map(|(r, row)| row.iter().enumerate().map(move |(c, &n)| (r, c, u32::from(n))))
    }

    pub fn total() -> u32 {
        cells().map(|(_, _, n)| n).sum()
    }

    /// The cells the figure prints a number in.
    pub fn printed() -> usize {
        cells().filter(|&(_, _, n)| n > 0).count()
    }

    /// The points that fall inside the lettered search grid.
    pub fn in_grid() -> u32 {
        cells()
            .filter(|(r, c, _)| GRID_ROWS.contains(r) && GRID_COLS.contains(c))
            .map(|(_, _, n)| n)
            .sum()
    }

    /// The area of the lettered grid in square miles.
    pub fn grid_area() -> Q {
        Q::int((GRID_ROWS.len() * GRID_COLS.len()) as i128) * CELL_MILES.0 * CELL_MILES.1
    }

    /// A cell's place when the printed cells are ranked by points, 1 the
    /// highest (cells with more points, plus one).
    pub fn rank(name: &str) -> Option<usize> {
        let mine = points(name)?;
        Some(1 + cells().filter(|&(_, _, n)| n > mine).count())
    }

    /// The figure's four shading classes: how many printed cells hold up to
    /// 10, up to 100, up to 1,000, and more.
    pub fn shading() -> [usize; 4] {
        let mut out = [0; 4];
        for (_, _, n) in cells().filter(|&(_, _, n)| n > 0) {
            out[match n {
                0..=10 => 0,
                11..=100 => 1,
                101..=1000 => 2,
                _ => 3,
            }] += 1;
        }
        out
    }

    /// The order in which a model search visits cells.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Rule {
        /// The cell with the greatest chance of a find. Every cell is seen
        /// equally well in this model, so that is the cell with the most
        /// chance left (ties: first in reading order).
        Best,
        /// The lettered grid row by row, A1 to O1, A2 to O2, and so on,
        /// then round again.
        Sweep,
    }

    /// One search of a model run in which every search comes back empty.
    #[derive(Clone, Copy, Debug)]
    pub struct Look {
        pub row: usize,
        pub col: usize,
        /// The chance, under the prior map, that the wreck has been found
        /// by the end of this search.
        pub found_by: Q,
    }

    /// A model search over the 1968 map: every search of a cell sees a
    /// wreck that is there with the same chance `seeing`, searches are
    /// independent, and each comes back empty. Not what the Navy did: the
    /// video's own model on the Navy's map.
    pub fn model_search(rule: Rule, seeing: Q, looks: usize) -> Vec<Look> {
        // w[r][c]: the chance the wreck is in the cell and still unfound
        let mut w: Vec<Vec<Q>> = POINTS
            .iter()
            .map(|row| {
                row.iter()
                    .map(|&n| q(i128::from(n), i128::from(REPLICATIONS)))
                    .collect()
            })
            .collect();
        let sweep: Vec<(usize, usize)> = GRID_ROWS.flat_map(|r| GRID_COLS.map(move |c| (r, c))).collect();
        let (mut found, mut out) = (Q::ZERO, Vec::with_capacity(looks));
        for look in 0..looks {
            let (row, col) = match rule {
                Rule::Sweep => sweep[look % sweep.len()],
                Rule::Best => {
                    let mut best = (0, 0);
                    for (r, row) in w.iter().enumerate() {
                        for (c, &x) in row.iter().enumerate() {
                            if x > w[best.0][best.1] {
                                best = (r, c);
                            }
                        }
                    }
                    best
                }
            };
            found = found + w[row][col] * seeing;
            w[row][col] = w[row][col] * (Q::ONE - seeing);
            out.push(Look {
                row,
                col,
                found_by: found,
            });
        }
        out
    }

    /// The first search (counting from 1) of a model run that goes to `name`.
    pub fn first_look_at(name: &str, rule: Rule, seeing: Q, limit: usize) -> Option<usize> {
        let target = at(name)?;
        model_search(rule, seeing, limit)
            .iter()
            .position(|l| (l.row, l.col) == target)
            .map(|i| i + 1)
    }
}

// --------------------------------------------------------------------------
// Koopman's random search formula
// --------------------------------------------------------------------------

pub mod koopman {
    use super::{Data, Q, q};

    /// The chance of seeing what is there after `coverage` (strip width
    /// times track length over area), when track is laid without regard to
    /// what is already swept: 1 − e^(−coverage). Koopman (1946), eq. (40).
    pub fn random_search(coverage: f64) -> f64 {
        1.0 - (-coverage).exp()
    }

    /// The same track laid as perfect lanes that never overlap.
    pub fn perfect_lanes(coverage: f64) -> f64 {
        coverage.min(1.0)
    }

    /// The coverage at which the random search formula gives `seeing`.
    pub fn coverage_for(seeing: f64) -> f64 {
        -(1.0 - seeing).ln()
    }

    /// Seed of the lanes (the year Koopman's report was printed).
    pub const SEED: u64 = 1946;
    /// The cell as a lattice of SIDE × SIDE points.
    pub const SIDE: usize = 200;
    /// A lane's width in lattice points: 1% of the cell's side.
    pub const WIDTH: usize = 2;
    /// Lanes that would cover the cell exactly once if none overlapped.
    pub const LANES_PER_COVERAGE: usize = SIDE / WIDTH;
    /// Runs averaged for the mean curve.
    pub const RUNS: usize = 1000;

    /// One straight pass across the whole cell: a strip `WIDTH` wide
    /// starting at row or column `at` (wrapping at the edge, so every
    /// point of the cell is as likely to be swept as any other).
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Lane {
        pub vertical: bool,
        pub at: usize,
    }

    /// The next lane of a stream: two uniform draws, the first for the
    /// direction and the second for the offset.
    pub fn lane(data: &mut Data) -> Lane {
        let vertical = data.unit() < 0.5;
        let at = ((data.unit() * SIDE as f64) as usize).min(SIDE - 1);
        Lane { vertical, at }
    }

    /// The share of one lane: the chance a given point is swept by it.
    pub fn lane_share() -> Q {
        q(WIDTH as i128, SIDE as i128)
    }

    /// Lattice points swept after each of `lanes` lanes laid in order.
    pub fn swept_after(lanes: &[Lane]) -> Vec<usize> {
        let (mut rows, mut cols) = (vec![false; SIDE], vec![false; SIDE]);
        let (mut nr, mut nc) = (0usize, 0usize);
        lanes
            .iter()
            .map(|l| {
                let (marks, count) = if l.vertical {
                    (&mut cols, &mut nc)
                } else {
                    (&mut rows, &mut nr)
                };
                for k in 0..WIDTH {
                    let i = (l.at + k) % SIDE;
                    if !marks[i] {
                        marks[i] = true;
                        *count += 1;
                    }
                }
                // a point is unswept only if neither its row nor its column is marked
                SIDE * SIDE - (SIDE - nr) * (SIDE - nc)
            })
            .collect()
    }

    /// The video's one run: the first `n` lanes of the stream.
    pub fn the_run(n: usize) -> Vec<Lane> {
        let mut data = Data::new(SEED);
        (0..n).map(|_| lane(&mut data)).collect()
    }

    /// Lattice points swept, summed over [`RUNS`] runs of `n` lanes each,
    /// after each lane; the runs follow the video's one run in the stream.
    pub fn swept_sums(n: usize) -> Vec<u64> {
        let mut data = Data::new(SEED);
        let mut sums = vec![0u64; n];
        for _ in 0..=RUNS {
            let lanes: Vec<Lane> = (0..n).map(|_| lane(&mut data)).collect();
            for (s, x) in sums.iter_mut().zip(swept_after(&lanes)) {
                *s += x as u64;
            }
        }
        // the first run is the video's one run, shown on its own; the mean is of the next RUNS
        for (s, x) in sums.iter_mut().zip(swept_after(&the_run(n))) {
            *s -= x as u64;
        }
        sums
    }

    /// What `n` independent lanes sweep on average: 1 − (1 − share)^n,
    /// which the formula's 1 − e^(−coverage) is the thin-lane limit of.
    pub fn expected_share(n: u32) -> f64 {
        1.0 - (1.0 - lane_share().f64()).powi(n as i32)
    }
}

// --------------------------------------------------------------------------
// A drifting cloud
// --------------------------------------------------------------------------

pub mod drift {
    use super::{Data, Q, q};

    /// Seed of the cloud (the year of the search the chapter tells).
    pub const SEED: u64 = 2013;
    pub const DOTS: usize = 2000;
    /// Model: how well the starting place is known, as a radius in miles.
    pub const START_RADIUS: Q = Q { n: 1, d: 1 };
    /// Model: the drift, in miles an hour (wind and current together).
    pub const DRIFT: Q = Q { n: 3, d: 2 };
    /// Model: the error in the drift, as a share of the distance drifted.
    pub const ERROR_SHARE: Q = Q { n: 3, d: 10 };
    /// Model: one flight's track in miles.
    pub const TRACK: Q = Q { n: 180, d: 1 };
    /// The strip a helicopter crew sees a person in the water in, in
    /// nautical miles (US Coast Guard Addendum, Table H-15).
    pub const STRIP: Q = Q { n: 1, d: 10 };

    /// The cloud's radius after `hours`: the starting radius plus the
    /// drift error, which grows with the distance drifted.
    pub fn radius(hours: Q) -> Q {
        START_RADIUS + ERROR_SHARE * DRIFT * hours
    }

    /// The area one flight sweeps, in square miles.
    pub fn swept() -> Q {
        TRACK * STRIP
    }

    /// The circle's area in square miles.
    pub fn area(hours: Q) -> f64 {
        std::f64::consts::PI * radius(hours).f64().powi(2)
    }

    /// One flight's coverage of the whole cloud after `hours`.
    pub fn coverage(hours: Q) -> f64 {
        swept().f64() / area(hours)
    }

    /// The chance one flight over the whole cloud sees the person.
    pub fn seeing(hours: Q) -> f64 {
        super::koopman::random_search(coverage(hours))
    }

    /// One dot: where it starts, relative to the best guess, and the error
    /// in its own drift, in miles an hour.
    #[derive(Clone, Copy, Debug)]
    pub struct Dot {
        pub start: (f64, f64),
        pub error: (f64, f64),
    }

    /// A point uniform in the unit disc, by rejection from the square.
    fn in_disc(data: &mut Data) -> (f64, f64) {
        loop {
            let (x, y) = (data.uniform(-1.0, 1.0), data.uniform(-1.0, 1.0));
            if x * x + y * y <= 1.0 {
                return (x, y);
            }
        }
    }

    /// The seeded cloud: each dot's start is uniform in the starting disc,
    /// and its drift error uniform in a disc of the error's radius.
    pub fn cloud() -> Vec<Dot> {
        let mut data = Data::new(SEED);
        let (r0, e) = (START_RADIUS.f64(), (ERROR_SHARE * DRIFT).f64());
        (0..DOTS)
            .map(|_| {
                let (sx, sy) = in_disc(&mut data);
                let (ex, ey) = in_disc(&mut data);
                Dot {
                    start: (r0 * sx, r0 * sy),
                    error: (e * ex, e * ey),
                }
            })
            .collect()
    }

    /// A dot's place after `hours`, relative to where the mean drift has
    /// carried the cloud's centre.
    pub fn offset(dot: &Dot, hours: f64) -> (f64, f64) {
        (dot.start.0 + dot.error.0 * hours, dot.start.1 + dot.error.1 * hours)
    }

    /// The farthest any dot is from the centre after `hours`.
    pub fn farthest(cloud: &[Dot], hours: f64) -> f64 {
        cloud
            .iter()
            .map(|d| {
                let (x, y) = offset(d, hours);
                x.hypot(y)
            })
            .fold(0.0, f64::max)
    }

    /// One flight spent on a square of `side` miles centred on the cloud:
    /// the dots inside it, the chance of seeing a person who is inside,
    /// and their product over the whole cloud, the chance of a find.
    pub struct Flight {
        pub inside: usize,
        pub seeing: f64,
        pub find: f64,
    }

    pub fn flight(cloud: &[Dot], hours: f64, side: u32) -> Flight {
        let half = f64::from(side) / 2.0;
        let inside = cloud
            .iter()
            .filter(|d| {
                let (x, y) = offset(d, hours);
                x.abs() <= half && y.abs() <= half
            })
            .count();
        let seeing = super::koopman::random_search(swept().f64() / f64::from(side * side));
        Flight {
            inside,
            seeing,
            find: inside as f64 / cloud.len() as f64 * seeing,
        }
    }

    /// The side, among whole miles from 1 to `max`, that gives one flight
    /// the greatest chance of a find (ties: the smaller side).
    pub fn best_side(cloud: &[Dot], hours: f64, max: u32) -> u32 {
        let mut best = (1, f64::MIN);
        for side in 1..=max {
            let f = flight(cloud, hours, side).find;
            if f > best.1 {
                best = (side, f);
            }
        }
        best.0
    }

    /// The hours the video compares.
    pub fn hours() -> [Q; 4] {
        [q(1, 1), q(3, 1), q(6, 1), q(12, 1)]
    }
}

// --------------------------------------------------------------------------
// Dates
// --------------------------------------------------------------------------

pub mod days {
    /// Days since 1970-01-01 of a civil date (proleptic Gregorian).
    pub fn of(year: i64, month: i64, day: i64) -> i64 {
        let y = if month <= 2 { year - 1 } else { year };
        let era = y.div_euclid(400);
        let yoe = y.rem_euclid(400);
        let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    /// Days from one date to another.
    pub fn between(from: (i64, i64, i64), to: (i64, i64, i64)) -> i64 {
        of(to.0, to.1, to.2) - of(from.0, from.1, from.2)
    }

    /// Whole months from one date to another (the day of the month ignored
    /// when the later day is not earlier in its month).
    pub fn months_between(from: (i64, i64, i64), to: (i64, i64, i64)) -> i64 {
        let months = (to.0 - from.0) * 12 + (to.1 - from.1);
        if to.2 < from.2 { months - 1 } else { months }
    }
}

// --------------------------------------------------------------------------
// The quizzes
// --------------------------------------------------------------------------

/// What a "your turn" card asks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ask {
    /// A's chance from the two stories.
    Stories,
    /// Which cell to search first.
    FirstSearch,
    /// B's chance after its search comes back empty.
    AfterEmpty,
    /// Where to search next.
    NextSearch,
    /// Two searches of one cell: the chance at least one sees what is there.
    TwoSearches,
    /// The cloud's radius doubles: what happens to one flight's coverage.
    RadiusDoubles,
}

/// One quiz: its name in the fact keys, the storyboard line that asks it,
/// its countdown, and what it asks.
#[derive(Clone, Copy, Debug)]
pub struct Quiz {
    pub name: &'static str,
    pub line: &'static str,
    pub seconds: u32,
    pub ask: Ask,
}

const fn quiz(name: &'static str, line: &'static str, seconds: u32, ask: Ask) -> Quiz {
    Quiz {
        name,
        line,
        seconds,
        ask,
    }
}

/// Every quiz of the video, in order. The storyboard's countdown beats are
/// exactly these lines (a test in `main.rs`).
pub const QUIZZES: &[Quiz] = &[
    quiz("stories", "03_6", 6, Ask::Stories),
    quiz("first_search", "05_4", 6, Ask::FirstSearch),
    quiz("after_empty", "06_5", 6, Ask::AfterEmpty),
    quiz("next_search", "07_4", 6, Ask::NextSearch),
    quiz("two_searches", "09_5", 5, Ask::TwoSearches),
    quiz("radius_doubles", "10_4", 5, Ask::RadiusDoubles),
];

#[cfg(test)]
mod tests {
    use super::sea::Rule;
    use super::*;

    #[test]
    fn fractions_reduce_compare_and_print() {
        assert_eq!(q(20, 100), q(1, 5));
        assert_eq!(q(1, -2), q(-1, 2));
        assert!(q(1, 3) < q(1, 2));
        assert_eq!((q(1, 2) + q(1, 3)).to_string(), "5/6");
        assert_eq!((q(1, 2) - q(1, 2)).to_string(), "0");
        assert_eq!(q(1, 8).percent(1), "12.5");
        assert_eq!(q(2, 15).percent(1), "13.3");
        assert_eq!(q(1, 15).percent(1), "6.7");
        // half up, not half to even
        assert_eq!(q(1, 8).percent(0), "13");
        assert_eq!(q(5, 1000).fixed(2), "0.01");
        assert_eq!(q(2, 3).pow(2), q(4, 9));
        assert_eq!(q(6, 3), Q::int(2));
    }

    #[test]
    fn the_two_stories_make_the_map() {
        let dots: Vec<[Q; 4]> = sea::stories().iter().map(sea::story_dots).collect();
        assert_eq!(dots[0], [30, 20, 10, 0].map(Q::int));
        assert_eq!(dots[1], [10; 4].map(Q::int));
        assert_eq!(sea::prior(), [q(40, 100), q(30, 100), q(20, 100), q(10, 100)]);
        assert_eq!(sea::prior().into_iter().fold(Q::ZERO, |a, b| a + b), Q::ONE);
    }

    /// Narration truth: the likeliest cell is not the best first search;
    /// the empty search of B halves nothing but B; then C, then A.
    #[test]
    fn the_hand_example_goes_as_the_video_says() {
        let p = sea::prior();
        assert_eq!(sea::find(&p), [q(1, 10), q(1, 5), q(2, 15), q(1, 15)]);
        let run = sea::run(Rule::Best, 4);
        assert_eq!(run.iter().map(|s| sea::CELLS[s.cell]).collect::<String>(), "BCAA");
        assert_eq!(run[0].after, [q(1, 2), q(1, 8), q(1, 4), q(1, 8)]);
        assert_eq!(sea::find(&run[0].after), [q(1, 8), q(1, 12), q(1, 6), q(1, 12)]);
        assert_eq!(run[1].after, [q(3, 5), q(3, 20), q(1, 10), q(3, 20)]);
        assert_eq!(run[1].found_by, q(1, 3));
        // every other cell rises by the same factor: 100 dots over the 80 left
        for cell in [0, 2, 3] {
            assert_eq!(run[0].after[cell] / p[cell], q(100, 80));
        }
    }

    /// A map stays a map: after any empty search the chances sum to one,
    /// the searched cell's chance falls and every other cell's rises.
    #[test]
    fn an_update_keeps_the_total_and_moves_chance_the_right_way() {
        let mut p = sea::prior();
        for step in 0..40 {
            let cell = (step * 7 + step / 3) % 4;
            let next = sea::update(&p, cell);
            assert_eq!(next.into_iter().fold(Q::ZERO, |a, b| a + b), Q::ONE);
            for i in 0..4 {
                if i == cell {
                    assert!(next[i] < p[i], "the searched cell loses chance");
                } else {
                    assert!(next[i] > p[i], "every other cell gains chance");
                }
            }
            p = next;
            if step == 14 {
                break; // exact arithmetic: the denominators stay inside i128
            }
        }
    }

    /// The edge of "no order of searches does better": every order of up
    /// to 8 searches, tried one by one, against the video's rule.
    #[test]
    fn no_order_of_searches_beats_the_rule() {
        let run = sea::run(Rule::Best, 8);
        for n in 1..=8 {
            assert_eq!(sea::best_of_all_orders(n), run[n - 1].found_by, "{n} searches");
        }
        // and the other rules are never ahead
        for rule in [Rule::Likeliest, Rule::LikeliestFresh, Rule::Sweep] {
            for (ours, theirs) in sea::run(Rule::Best, 12).iter().zip(sea::run(rule, 12)) {
                assert!(ours.found_by >= theirs.found_by, "{rule:?}");
            }
        }
    }

    /// The edge of "an empty search only tells you something if it could
    /// have found what you were looking for": a search that cannot see
    /// leaves the map exactly as it was, and one that cannot miss empties
    /// its cell.
    #[test]
    fn a_blind_search_changes_nothing_and_a_perfect_one_empties_its_cell() {
        let p = sea::prior();
        for cell in 0..4 {
            assert_eq!(sea::bayes(&p, cell, Q::ZERO), p);
            let sure = sea::bayes(&p, cell, Q::ONE);
            assert_eq!(sure[cell], Q::ZERO);
            assert_eq!(sure.into_iter().fold(Q::ZERO, |a, b| a + b), Q::ONE);
        }
    }

    /// The edge of "no number of passes makes it certain": for a search
    /// that can miss, every further pass helps and none reaches one.
    #[test]
    fn no_number_of_passes_makes_it_certain() {
        for each in [q(1, 4), q(2, 3), q(9, 10)] {
            let mut last = Q::ZERO;
            for passes in 1..=20 {
                let seen = sea::seen_within(each, passes);
                assert!(last < seen && seen < Q::ONE, "{each} after {passes}");
                last = seen;
            }
        }
        assert_eq!(
            sea::seen_within(Q::ONE, 1),
            Q::ONE,
            "only a search that cannot miss is certain"
        );
    }

    #[test]
    fn found_by_agrees_with_counting_searches_per_cell() {
        // the chance of a find within an order depends only on how many
        // times each cell is searched: sum of p_i (1 − (1 − d_i)^k_i)
        let (p, d) = (sea::prior(), sea::seeing());
        let order = [1usize, 2, 0, 0, 1, 3, 0, 2];
        let direct = (0..4).fold(Q::ZERO, |acc, i| {
            let k = order.iter().filter(|&&c| c == i).count() as u32;
            acc + p[i] * sea::seen_within(d[i], k)
        });
        assert_eq!(sea::found_by(&order), direct);
    }

    #[test]
    fn expected_searches_orders_the_rules() {
        let best = sea::expected_searches(Rule::Best);
        assert!((best - 6.338).abs() < 0.001, "{best}");
        assert!((sea::expected_searches(Rule::Sweep) - 8.0).abs() < 1e-9);
        for rule in [Rule::Likeliest, Rule::LikeliestFresh, Rule::Sweep] {
            assert!(sea::expected_searches(rule) > best, "{rule:?}");
        }
    }

    #[test]
    fn the_1968_map_adds_up_as_the_paper_says() {
        assert_eq!(y1968::total(), y1968::REPLICATIONS);
        assert_eq!(y1968::printed(), 172);
        assert_eq!(y1968::points("E5"), Some(1250));
        assert_eq!(y1968::points("B7"), Some(1096));
        assert_eq!(y1968::points("F6"), Some(85));
        // "two search cells, E5 and B7, have 23 percent probability"
        assert_eq!(q(1250 + 1096, 10_000).percent(0), "23");
        assert_eq!(y1968::rank("E5"), Some(1));
        assert_eq!(y1968::rank("F6"), Some(24));
        assert_eq!(y1968::in_grid(), 8900);
        assert_eq!(y1968::grid_area(), q(756, 5));
        assert_eq!(y1968::shading(), [80, 71, 19, 2]);
        assert_eq!(y1968::name(6, 7), "E5");
        assert_eq!(y1968::at("w16"), Some((7, 2)));
    }

    #[test]
    fn the_model_search_reaches_f6_long_before_the_sweep() {
        let seeing = q(2, 3);
        assert_eq!(y1968::first_look_at("F6", y1968::Rule::Best, seeing, 400), Some(35));
        assert_eq!(y1968::first_look_at("F6", y1968::Rule::Sweep, seeing, 400), Some(81));
        let best = y1968::model_search(y1968::Rule::Best, seeing, 40);
        assert_eq!(y1968::name(best[0].row, best[0].col), "E5");
        assert_eq!(best[0].found_by, q(1250 * 2, 30_000));
        // the first 34 searches all go to cells with more chance left than F6
        assert!(best[..34].iter().all(|l| (l.row, l.col) != y1968::at("F6").unwrap()));
    }

    #[test]
    fn lanes_sweep_what_the_formula_says_on_average() {
        let n = 3 * koopman::LANES_PER_COVERAGE;
        let one = koopman::swept_after(&koopman::the_run(n));
        assert!(one.windows(2).all(|w| w[0] <= w[1]), "swept points never fall");
        assert_eq!(
            one[0],
            koopman::WIDTH * koopman::SIDE,
            "the first lane sweeps its whole strip"
        );
        let sums = koopman::swept_sums(n);
        let points = (koopman::SIDE * koopman::SIDE * koopman::RUNS) as f64;
        for coverage in 1..=3u32 {
            let lanes = coverage as usize * koopman::LANES_PER_COVERAGE;
            let mean = sums[lanes - 1] as f64 / points;
            let exact = koopman::expected_share(lanes as u32);
            assert!(
                (mean - exact).abs() < 0.01,
                "coverage {coverage}: mean {mean}, exact {exact}"
            );
            assert!((exact - koopman::random_search(f64::from(coverage))).abs() < 0.005);
        }
        assert!((koopman::coverage_for(2.0 / 3.0) - 3f64.ln()).abs() < 1e-12);
    }

    #[test]
    fn the_cloud_stays_inside_its_radius_and_one_flight_buys_less_each_hour() {
        let cloud = drift::cloud();
        assert_eq!(cloud.len(), drift::DOTS);
        let mut last = 1.0;
        for h in drift::hours() {
            assert!(drift::farthest(&cloud, h.f64()) <= drift::radius(h).f64());
            let s = drift::seeing(h);
            assert!(s < last, "the same flight sees less as the cloud grows");
            last = s;
        }
        // twice the radius is four times the area: a quarter of the coverage
        let (r1, r2) = (drift::radius(q(1, 1)), drift::radius(q(1, 1)) * Q::int(2));
        assert_eq!((r2 * r2) / (r1 * r1), Q::int(4));
    }

    #[test]
    fn day_counts() {
        assert_eq!(days::of(1970, 1, 1), 0);
        assert_eq!(days::between((1968, 5, 27), (1968, 10, 28)), 154);
        assert_eq!(days::between((1968, 2, 28), (1968, 3, 1)), 2, "1968 is a leap year");
        assert_eq!(days::months_between((2009, 6, 1), (2011, 4, 2)), 22);
    }
}
