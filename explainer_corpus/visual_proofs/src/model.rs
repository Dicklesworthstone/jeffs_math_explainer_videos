//! The pictures of Video I as exact pieces.
//!
//! Every proof in the video is one pile of pieces counted two ways, so each
//! picture is built here as the list of its pieces (unit cells, rectangles
//! and boxes with whole-number corners, triangles on the grid) and checked
//! the slow way: nothing overlaps and nothing is left over. The chapters
//! draw exactly these pieces; `facts.rs` reads its numbers off them; and
//! `oracle.py` rebuilds every picture by its own route.
//!
//! Nothing here is floating point. Lengths are whole numbers; the infinite
//! sums, the slopes of the lying picture and the points on the hook's circle
//! are exact fractions ([`Q`]).

use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};

const fn gcd(mut a: i128, mut b: i128) -> i128 {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    if a < 0 { -a } else { a }
}

/// An exact fraction in lowest terms, the denominator positive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Q {
    n: i128,
    d: i128,
}

/// `n / d`, reduced.
///
/// # Panics
/// If `d` is zero.
pub const fn q(n: i128, d: i128) -> Q {
    assert!(d != 0, "a fraction needs a denominator");
    let g = gcd(n, d);
    if d < 0 {
        Q { n: -n / g, d: -d / g }
    } else {
        Q { n: n / g, d: d / g }
    }
}

impl Q {
    pub const ZERO: Self = q(0, 1);
    pub const ONE: Self = q(1, 1);

    pub const fn int(n: i64) -> Self {
        q(n as i128, 1)
    }

    /// One over two to the `k`.
    pub const fn half_to(k: u32) -> Self {
        q(1, 1i128 << k)
    }

    pub const fn signum(self) -> i32 {
        if self.n > 0 {
            1
        } else if self.n < 0 {
            -1
        } else {
            0
        }
    }
}

impl Add for Q {
    type Output = Self;
    fn add(self, o: Self) -> Self {
        q(self.n * o.d + o.n * self.d, self.d * o.d)
    }
}

impl Sub for Q {
    type Output = Self;
    fn sub(self, o: Self) -> Self {
        q(self.n * o.d - o.n * self.d, self.d * o.d)
    }
}

impl Mul for Q {
    type Output = Self;
    fn mul(self, o: Self) -> Self {
        q(self.n * o.n, self.d * o.d)
    }
}

impl Div for Q {
    type Output = Self;
    fn div(self, o: Self) -> Self {
        q(self.n * o.d, self.d * o.n)
    }
}

impl Neg for Q {
    type Output = Self;
    fn neg(self) -> Self {
        q(-self.n, self.d)
    }
}

impl Ord for Q {
    fn cmp(&self, o: &Self) -> Ordering {
        (self.n * o.d).cmp(&(o.n * self.d))
    }
}

impl PartialOrd for Q {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
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

/// n choose k, counted by the product formula (exact at every step).
pub fn choose(n: u64, k: u64) -> u64 {
    if k > n {
        return 0;
    }
    (0..k.min(n - k)).fold(1, |acc, i| acc * (n - i) / (i + 1))
}

/// A rectangle of unit cells: its lower-left corner and its size.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rect {
    pub x: i64,
    pub y: i64,
    pub w: i64,
    pub h: i64,
}

impl Rect {
    pub const fn new(x: i64, y: i64, w: i64, h: i64) -> Self {
        Self { x, y, w, h }
    }

    pub const fn area(&self) -> i64 {
        self.w * self.h
    }

    pub fn cells(&self) -> impl Iterator<Item = (i64, i64)> + use<> {
        let r = *self;
        (r.x..r.x + r.w).flat_map(move |x| (r.y..r.y + r.h).map(move |y| (x, y)))
    }
}

/// The slow check behind every flat picture: the cells given cover the frame
/// exactly once each, apart from the cells of `holes`, which nothing covers.
pub fn tiles(cells: impl IntoIterator<Item = (i64, i64)>, frame: Rect, holes: &[Rect]) -> Result<(), String> {
    let index = |(x, y): (i64, i64)| -> Option<usize> {
        (x >= frame.x && x < frame.x + frame.w && y >= frame.y && y < frame.y + frame.h)
            .then(|| ((x - frame.x) + (y - frame.y) * frame.w) as usize)
    };
    let mut seen = vec![0u8; (frame.w * frame.h) as usize];
    for c in cells {
        let i = index(c).ok_or_else(|| format!("cell {c:?} lies outside the frame"))?;
        if seen[i] != 0 {
            return Err(format!("cell {c:?} is covered twice"));
        }
        seen[i] = 1;
    }
    for c in holes.iter().flat_map(Rect::cells) {
        let i = index(c).ok_or_else(|| format!("hole cell {c:?} lies outside the frame"))?;
        if seen[i] != 0 {
            return Err(format!("hole cell {c:?} is covered"));
        }
        seen[i] = 2;
    }
    match seen.iter().position(|&s| s == 0) {
        Some(i) => Err(format!(
            "cell ({}, {}) is left over",
            frame.x + i as i64 % frame.w,
            frame.y + i as i64 / frame.w
        )),
        None => Ok(()),
    }
}

/// The first size in `1..=through` at which `check` fails, if any.
pub fn first_failure(through: i64, check: impl Fn(i64) -> Result<(), String>) -> Option<(i64, String)> {
    (1..=through).find_map(|n| check(n).err().map(|e| (n, e)))
}

/// The hook: points on a circle joined by straight lines, and the regions
/// they cut the disc into (Leo Moser, "On the Danger of Induction", 1949).
pub mod circle {
    use std::collections::HashSet;

    use super::{Q, choose, q};

    pub type Point = (Q, Q);

    /// The points the chapter draws, in the order it adds them, as the
    /// parameters `p/d` of the unit circle's rational points
    /// `((d² − p²)/(d² + p²), 2pd/(d² + p²))`: unevenly spaced, so that no
    /// three of their lines meet at one point inside the circle.
    pub const DRAWN: [(i128, i128); 7] = [(12, 11), (-19, 6), (3, 17), (17, 3), (-5, 13), (-8, 7), (4, 7)];

    pub const fn point(p: i128, d: i128) -> Point {
        (q(d * d - p * p, d * d + p * p), q(2 * p * d, d * d + p * p))
    }

    /// The first `n` drawn points.
    pub fn drawn(n: usize) -> Vec<Point> {
        DRAWN[..n].iter().map(|&(p, d)| point(p, d)).collect()
    }

    /// Six points whose three long lines all pass through the centre, as
    /// six evenly spaced points' do: the pattern's other way of breaking.
    pub fn three_through_one() -> Vec<Point> {
        [(1, 2), (2, 3), (1, 4)]
            .into_iter()
            .flat_map(|(p, d)| {
                let (x, y) = point(p, d);
                [(x, y), (-x, -y)]
            })
            .collect()
    }

    fn orient(a: Point, b: Point, c: Point) -> Q {
        (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
    }

    /// Where two lines' segments cross, if they cross between their ends.
    fn crossing(a: Point, b: Point, c: Point, d: Point) -> Option<Point> {
        let (o1, o2) = (orient(a, b, c), orient(a, b, d));
        let (o3, o4) = (orient(c, d, a), orient(c, d, b));
        (o1.signum() * o2.signum() < 0 && o3.signum() * o4.signum() < 0).then(|| {
            let t = o3 / (o3 - o4);
            (a.0 + t * (b.0 - a.0), a.1 + t * (b.1 - a.1))
        })
    }

    /// Every pair of points joined: the lines, as index pairs.
    pub fn lines(n: usize) -> Vec<(usize, usize)> {
        (0..n).flat_map(|i| (i + 1..n).map(move |j| (i, j))).collect()
    }

    /// The distinct points inside the circle where lines cross.
    pub fn crossings(points: &[Point]) -> Vec<Point> {
        let mut seen = HashSet::new();
        let mut out = Vec::new();
        let lines = lines(points.len());
        for (i, &(a, b)) in lines.iter().enumerate() {
            for &(c, d) in &lines[i + 1..] {
                if a == c || a == d || b == c || b == d {
                    continue;
                }
                if let Some(p) = crossing(points[a], points[b], points[c], points[d])
                    && seen.insert(p)
                {
                    out.push(p);
                }
            }
        }
        out
    }

    /// The regions, counted from the drawing itself by Euler's formula for a
    /// connected map (regions = edges − corners + 1): the corners are the
    /// points on the circle and the distinct crossings inside it, and the
    /// edges are the arcs between neighbouring points and the stretches of
    /// each line between its corners. Three lines through one point are
    /// counted as they fall, with no assumption about the points.
    pub fn regions(points: &[Point]) -> u64 {
        let n = points.len();
        if n == 0 {
            return 1;
        }
        let lines = lines(n);
        let mut inside: HashSet<Point> = HashSet::new();
        let mut on_line: Vec<HashSet<Point>> = vec![HashSet::new(); lines.len()];
        for i in 0..lines.len() {
            for j in i + 1..lines.len() {
                let ((a, b), (c, d)) = (lines[i], lines[j]);
                if a == c || a == d || b == c || b == d {
                    continue;
                }
                if let Some(p) = crossing(points[a], points[b], points[c], points[d]) {
                    inside.insert(p);
                    on_line[i].insert(p);
                    on_line[j].insert(p);
                }
            }
        }
        let corners = n + inside.len();
        let edges = n + on_line.iter().map(|s| 1 + s.len()).sum::<usize>();
        (edges + 1 - corners) as u64
    }

    /// The count when no three lines meet inside: one region to start, one
    /// more for every line and one more for every crossing, and every four
    /// points give exactly one crossing.
    pub fn closed_form(n: u64) -> u64 {
        1 + choose(n, 2) + choose(n, 4)
    }
}

/// Unit squares in L-shaped layers, and staircases.
pub mod layers {
    pub type Cell = (i64, i64);

    /// The `k`-th L-shaped layer (from 1): the cells of the k by k square
    /// that the (k − 1) by (k − 1) square in the same corner does not have.
    pub fn layer(k: i64) -> Vec<Cell> {
        (0..k)
            .flat_map(|x| (0..k).map(move |y| (x, y)))
            .filter(|&(x, y)| x.max(y) == k - 1)
            .collect()
    }

    /// A staircase of `n` columns: column x (from 0) holds x + 1 cells.
    pub fn staircase(n: i64) -> Vec<Cell> {
        (0..n).flat_map(|x| (0..=x).map(move |y| (x, y))).collect()
    }

    /// The same staircase turned half round, in the place where it locks
    /// with [`staircase`] into a rectangle n wide and n + 1 tall.
    pub fn turned(n: i64) -> Vec<Cell> {
        staircase(n).into_iter().map(|(x, y)| (n - 1 - x, n - y)).collect()
    }
}

/// The sum of cubes: each cube cut into square slabs, the slabs laid flat as
/// one L-shaped layer (Alan L. Fry's picture, 1985; the layers are
/// al-Karaji's, about the year 1000).
pub mod cubes {
    use super::Rect;

    /// How much of a slab a piece is.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Part {
        Whole,
        /// Half of the one slab an even cube has to cut, lying at the foot
        /// of the layer's upright arm.
        HalfRight,
        /// Its other half, turned a quarter turn, at the far end of the
        /// layer's top arm.
        HalfTop,
    }

    /// One piece of the big square: which cube and slab it came from.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Piece {
        pub cube: i64,
        /// The slab of that cube, from 0 (the corner square) to cube − 1.
        pub slab: i64,
        pub part: Part,
        pub rect: Rect,
    }

    /// 1 + 2 + … + n, the side of the square after `n` layers.
    pub const fn side(n: i64) -> i64 {
        n * (n + 1) / 2
    }

    /// The `k`-th layer: cube k's k slabs, each k by k, wrapped round the
    /// square of side 1 + … + (k − 1). The corner takes one slab. Each arm
    /// is k wide and k(k − 1)/2 long, which is (k − 1)/2 slabs: a whole
    /// number of slabs when k is odd, and a half slab over when k is even,
    /// so an even cube cuts one slab in two and gives each arm a half.
    pub fn layer(k: i64) -> Vec<Piece> {
        let s = side(k - 1);
        let whole = (k - 1) / 2;
        let foot = if k % 2 == 0 { k / 2 } else { 0 };
        let mut out = vec![Piece {
            cube: k,
            slab: 0,
            part: Part::Whole,
            rect: Rect::new(s, s, k, k),
        }];
        for i in 0..whole {
            out.push(Piece {
                cube: k,
                slab: 1 + i,
                part: Part::Whole,
                rect: Rect::new(s, foot + i * k, k, k),
            });
        }
        for i in 0..whole {
            out.push(Piece {
                cube: k,
                slab: 1 + whole + i,
                part: Part::Whole,
                rect: Rect::new(foot + i * k, s, k, k),
            });
        }
        if k % 2 == 0 {
            out.push(Piece {
                cube: k,
                slab: k - 1,
                part: Part::HalfRight,
                rect: Rect::new(s, 0, k, k / 2),
            });
            out.push(Piece {
                cube: k,
                slab: k - 1,
                part: Part::HalfTop,
                rect: Rect::new(0, s, k / 2, k),
            });
        }
        out
    }

    /// Every piece of the square built from cubes 1 to `n`.
    pub fn square(n: i64) -> Vec<Piece> {
        (1..=n).flat_map(layer).collect()
    }

    /// Nicomachus's observation (Introduction to Arithmetic II.20.5): the
    /// `k` consecutive odd numbers that add up to k cubed, the first being
    /// k(k − 1) + 1.
    pub fn odd_run(k: i64) -> Vec<i64> {
        (0..k).map(|i| k * (k - 1) + 1 + 2 * i).collect()
    }
}

/// The sum of squares: three equal stacks of square layers that lock into a
/// box (Man-Keung Siu's picture, 1984).
pub mod stacks {
    pub type Cell = (i64, i64, i64);

    /// One stack as the video first shows it: layer l (from 0, the bottom)
    /// is a square of side n − l, every layer pushed into the same corner.
    pub fn standard(n: i64) -> Vec<Cell> {
        (0..n)
            .flat_map(move |l| (0..n - l).flat_map(move |i| (0..n - l).map(move |j| (i, j, l))))
            .collect()
    }

    /// The three turns, as where each stack's own (i, j, l) axes point in
    /// the box: stack 0 stays as it is; stacks 1 and 2 are the same solid
    /// turned (each a rotation, never a mirror image).
    pub const TURNS: [[[i64; 3]; 3]; 3] = [
        [[1, 0, 0], [0, 1, 0], [0, 0, 1]],
        [[0, 1, 0], [0, 0, -1], [-1, 0, 0]],
        [[0, 0, -1], [1, 0, 0], [0, -1, 0]],
    ];

    /// Where a turned stack's corner cell (0, 0, 0) sits in the box.
    pub const fn shift(n: i64, which: usize) -> Cell {
        match which {
            0 => (0, 0, 0),
            1 => (0, n, n - 1),
            _ => (n - 1, 0, n),
        }
    }

    /// The cell of the box that stack `which`'s cell (i, j, l) fills.
    pub const fn place(n: i64, which: usize, (i, j, l): Cell) -> Cell {
        let m = TURNS[which];
        let s = shift(n, which);
        (
            m[0][0] * i + m[0][1] * j + m[0][2] * l + s.0,
            m[1][0] * i + m[1][1] * j + m[1][2] * l + s.1,
            m[2][0] * i + m[2][1] * j + m[2][2] * l + s.2,
        )
    }

    /// Stack `which` in its place: the box is n wide (x), n + 1 deep (y)
    /// and n tall (z from 0), and height n is the layer that sticks out on
    /// top.
    pub fn placed(n: i64, which: usize) -> Vec<Cell> {
        standard(n).into_iter().map(|c| place(n, which, c)).collect()
    }

    /// The determinant of a turn: +1 for a rotation, −1 for a mirror image.
    pub const fn determinant(m: [[i64; 3]; 3]) -> i64 {
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1]) - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    }

    /// The base layer stack `which` gains when the stacks grow from n − 1
    /// layers to n: one flat square slab of side n, in its place in the box.
    pub fn new_base(n: i64, which: usize) -> Vec<Cell> {
        standard(n)
            .into_iter()
            .filter(|c| c.2 == 0)
            .map(|c| place(n, which, c))
            .collect()
    }

    /// The step from one height to the next, checked cube by cube: each
    /// stack at height n is the same stack at height n − 1 lifted by one
    /// cube, standing on, in front of or beside its new base slab. So the
    /// locked solid of height n is the solid of height n − 1 with three new
    /// slabs round it: one under it, one behind it, one beside it.
    pub fn grows(n: i64) -> Result<(), String> {
        for which in 0..3 {
            let base = new_base(n, which);
            let mut grown: Vec<Cell> = placed(n - 1, which)
                .into_iter()
                .map(|(x, y, z)| (x, y, z + 1))
                .collect();
            grown.extend(&base);
            grown.sort_unstable();
            let mut whole = placed(n, which);
            whole.sort_unstable();
            if grown != whole {
                return Err(format!("stack {which} is not the last height lifted onto a new base"));
            }
            // the base is flat: n × n cells that agree in one coordinate
            let flat = [0, 1, 2].iter().any(|&axis| {
                let at = |c: &Cell| [c.0, c.1, c.2][axis];
                base.iter().all(|c| at(c) == at(&base[0]))
            });
            if !flat || base.len() as i64 != n * n {
                return Err(format!("stack {which}'s new base is not a flat slab of side {n}"));
            }
        }
        Ok(())
    }

    /// The columns (x, y) of the cells that stick out above height n: a
    /// staircase of 1 + 2 + … + n cubes, all from stack 2.
    pub fn top(n: i64) -> Vec<(i64, i64)> {
        (0..3)
            .flat_map(|w| placed(n, w))
            .filter(|c| c.2 == n)
            .map(|c| (c.0, c.1))
            .collect()
    }

    /// Where the upper half of the top cube over column (x, y) lands once
    /// the top layer is sliced flat and the slice is turned half round.
    pub const fn turned_half(n: i64, (x, y): (i64, i64)) -> (i64, i64) {
        (n - 1 - x, n - y)
    }

    /// Everything, in half-height cells (x, y, 2z): the three stacks with the
    /// top layer sliced and its upper half turned into the gap beside it.
    pub fn half_cells(n: i64) -> Vec<Cell> {
        let mut out = Vec::new();
        for (x, y, z) in (0..3).flat_map(|w| placed(n, w)) {
            if z < n {
                out.extend([(x, y, 2 * z), (x, y, 2 * z + 1)]);
            } else {
                let (tx, ty) = turned_half(n, (x, y));
                out.extend([(x, y, 2 * n), (tx, ty, 2 * n)]);
            }
        }
        out
    }

    /// The slow check: [`half_cells`] fill the box n by n + 1 by 2n + 1
    /// half-heights exactly once each.
    pub fn fills_the_box(n: i64) -> Result<(), String> {
        let (w, d, h) = (n, n + 1, 2 * n + 1);
        let mut seen = vec![false; (w * d * h) as usize];
        for (x, y, z) in half_cells(n) {
            if !(0..w).contains(&x) || !(0..d).contains(&y) || !(0..h).contains(&z) {
                return Err(format!("half-cell ({x}, {y}, {z}) lies outside the box"));
            }
            let i = (x + w * (y + d * z)) as usize;
            if seen[i] {
                return Err(format!("half-cell ({x}, {y}, {z}) is filled twice"));
            }
            seen[i] = true;
        }
        if seen.iter().all(|&s| s) {
            Ok(())
        } else {
            Err("a half-cell of the box is empty".to_owned())
        }
    }
}

/// The two infinite sums: a square halved again and again, and a square
/// quartered again and again.
pub mod series {
    use super::Q;

    /// A rectangle with exact corners: x, y, width, height.
    pub type Piece = (Q, Q, Q, Q);

    /// The `k`-th piece (from 1) of the unit square halved again and again:
    /// odd steps keep the left half of what is left, even steps the bottom
    /// half, so what is left shrinks into the top right corner.
    pub fn half(k: u32) -> Piece {
        // before step k: (k − 1)/2 upright cuts and (k − 1)/2 level cuts
        // have been made, an upright one first
        let (wide, tall) = (Q::half_to(k / 2), Q::half_to((k - 1) / 2));
        let (x, y) = (Q::ONE - wide, Q::ONE - tall);
        if k % 2 == 1 {
            (x, y, wide * Q::half_to(1), tall)
        } else {
            (x, y, wide, tall * Q::half_to(1))
        }
    }

    /// What is left of the square after `k` pieces.
    pub fn left_after_halves(k: u32) -> Piece {
        let (wide, tall) = (Q::half_to(k.div_ceil(2)), Q::half_to(k / 2));
        (Q::ONE - wide, Q::ONE - tall, wide, tall)
    }

    /// Level `j` (from 1) of the quartered square: the corner square still
    /// uncut, of side 1/2^(j−1), is cut in four. Three of the four are
    /// kept, one for each share (`[counted, above it, beside it]`); the
    /// fourth, in the far corner, is cut again at the next level.
    pub fn quarters(j: u32) -> [Piece; 3] {
        let s = Q::half_to(j);
        let o = Q::ONE - s - s;
        [(o, o, s, s), (o, o + s, s, s), (o + s, o, s, s)]
    }

    /// The square still uncut after level `j`.
    pub fn left_after_quarters(j: u32) -> Piece {
        let s = Q::half_to(j);
        (Q::ONE - s, Q::ONE - s, s, s)
    }

    pub fn area(p: Piece) -> Q {
        p.2 * p.3
    }
}

/// A square, a cube and a four-dimensional block cut by one cut along every
/// side (the binomial pictures).
pub mod blocks {
    /// One block of the cut: where it starts and how long it is along each
    /// side, and how many of its sides are the short part `b`.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Block {
        pub origin: Vec<i64>,
        pub sides: Vec<i64>,
        pub bs: usize,
    }

    impl Block {
        pub fn volume(&self) -> i64 {
            self.sides.iter().product()
        }
    }

    /// Every side of a `dim`-dimensional cube of side a + b cut once, at a:
    /// 2^dim blocks.
    pub fn cut(a: i64, b: i64, dim: usize) -> Vec<Block> {
        (0..1usize << dim)
            .map(|mask| {
                let short = |i: usize| mask >> i & 1 == 1;
                Block {
                    origin: (0..dim).map(|i| if short(i) { a } else { 0 }).collect(),
                    sides: (0..dim).map(|i| if short(i) { b } else { a }).collect(),
                    bs: mask.count_ones() as usize,
                }
            })
            .collect()
    }

    /// How many blocks have 0, 1, …, dim short sides, counted from the cut.
    pub fn kinds(dim: usize) -> Vec<usize> {
        let blocks = cut(2, 1, dim);
        (0..=dim).map(|j| blocks.iter().filter(|b| b.bs == j).count()).collect()
    }

    /// The slow check: the blocks' unit cells fill the cube of side a + b
    /// exactly once each, in any number of dimensions.
    pub fn fills_the_cube(a: i64, b: i64, dim: usize) -> Result<(), String> {
        let side = a + b;
        let mut seen = vec![false; side.pow(dim as u32) as usize];
        for block in cut(a, b, dim) {
            let cells = block.volume();
            for c in 0..cells {
                // the c-th cell of the block, counted along its sides in turn
                let (mut rest, mut index, mut stride) = (c, 0, 1);
                for i in 0..dim {
                    index += (block.origin[i] + rest % block.sides[i]) * stride;
                    rest /= block.sides[i];
                    stride *= side;
                }
                if std::mem::replace(&mut seen[index as usize], true) {
                    return Err(format!("a cell of block {block:?} is filled twice"));
                }
            }
        }
        if seen.iter().all(|&s| s) {
            Ok(())
        } else {
            Err("a cell of the cube is empty".to_owned())
        }
    }
}

/// Pythagoras by two arrangements of four right triangles in a square of
/// side a + b.
pub mod pythagoras {
    use super::Rect;

    pub type P = (i64, i64);

    /// Twice the area of a polygon (the shoelace sum; positive when the
    /// corners run anticlockwise).
    pub fn area2(poly: &[P]) -> i64 {
        (0..poly.len())
            .map(|i| {
                let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
                a.0 * b.1 - b.0 * a.1
            })
            .sum()
    }

    /// The first arrangement: one triangle in each corner of the big
    /// square, legs along its sides, right angle first.
    pub const fn corners(a: i64, b: i64) -> [[P; 3]; 4] {
        let s = a + b;
        [
            [(0, 0), (a, 0), (0, b)],
            [(s, 0), (s, a), (a, 0)],
            [(s, s), (b, s), (s, a)],
            [(0, s), (0, b), (b, s)],
        ]
    }

    /// The shape the four triangles leave: its corners, anticlockwise.
    pub const fn tilted(a: i64, b: i64) -> [P; 4] {
        let s = a + b;
        [(a, 0), (s, a), (b, s), (0, b)]
    }

    /// The slide that takes each triangle to the second arrangement (the
    /// first stays where it is; no triangle turns or flips).
    pub const fn slides(a: i64, b: i64) -> [P; 4] {
        [(0, 0), (0, b), (-b, -a), (a, 0)]
    }

    /// The second arrangement.
    pub fn slid(a: i64, b: i64) -> [[P; 3]; 4] {
        let (t, d) = (corners(a, b), slides(a, b));
        std::array::from_fn(|i| t[i].map(|p| (p.0 + d[i].0, p.1 + d[i].1)))
    }

    /// The two squares the second arrangement leaves: on leg a, on leg b.
    pub const fn left_over(a: i64, b: i64) -> (Rect, Rect) {
        (Rect::new(0, b, a, a), Rect::new(a, 0, b, b))
    }

    /// Four equal sides and a right angle at every corner, in whole numbers.
    pub fn is_square(c: &[P; 4]) -> bool {
        let side = |i: usize| {
            let (p, r) = (c[i], c[(i + 1) % 4]);
            (r.0 - p.0, r.1 - p.1)
        };
        let len2 = |v: P| v.0 * v.0 + v.1 * v.1;
        (0..4).all(|i| {
            let (u, w) = (side(i), side((i + 1) % 4));
            len2(u) == len2(side(0)) && u.0 * w.0 + u.1 * w.1 == 0
        })
    }

    /// Whether three points lie on one straight line.
    pub const fn straight(p: P, m: P, r: P) -> bool {
        (m.0 - p.0) * (r.1 - p.1) == (m.1 - p.1) * (r.0 - p.0)
    }
}

/// The picture that lies: the missing-square puzzle (Paul Curry's idea,
/// 1953; the triangle form is Martin Gardner's, 1956).
pub mod lie {
    use super::pythagoras::P;
    use super::{Q, q};

    /// The triangle the pieces seem to fill: 13 wide and 5 tall.
    pub const WIDE: i64 = 13;
    pub const TALL: i64 = 5;

    /// The four pieces: two right triangles and two L-shaped pieces of unit
    /// squares.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Arrangement {
        /// The triangle with legs 8 and 3.
        pub big: [P; 3],
        /// The triangle with legs 5 and 2.
        pub small: [P; 3],
        /// The L of 8 unit squares.
        pub long_l: Vec<P>,
        /// The L of 7 unit squares.
        pub short_l: Vec<P>,
        /// Where the outline bends: the point both triangles touch.
        pub bend: P,
    }

    fn cells(rows: &[(i64, i64, i64)]) -> Vec<P> {
        rows.iter()
            .flat_map(|&(y, x0, x1)| (x0..x1).map(move |x| (x, y)))
            .collect()
    }

    /// The pieces as first shown: no hole.
    pub fn first() -> Arrangement {
        Arrangement {
            big: [(8, 0), (8, 3), (0, 0)],
            small: [(13, 3), (13, 5), (8, 3)],
            short_l: cells(&[(0, 8, 13), (1, 8, 10)]),
            long_l: cells(&[(1, 10, 13), (2, 8, 13)]),
            bend: (8, 3),
        }
    }

    /// How far each piece slides to make the second arrangement: big
    /// triangle, small triangle, long L, short L. Nothing turns or flips.
    pub const SLIDES: [P; 4] = [(5, 2), (-8, -3), (0, -1), (-3, 0)];

    /// The same pieces slid: one square is empty.
    pub fn second() -> Arrangement {
        let a = first();
        let by = |d: P| move |p: P| (p.0 + d.0, p.1 + d.1);
        Arrangement {
            big: a.big.map(by(SLIDES[0])),
            small: a.small.map(by(SLIDES[1])),
            long_l: a.long_l.into_iter().map(by(SLIDES[2])).collect(),
            short_l: a.short_l.into_iter().map(by(SLIDES[3])).collect(),
            bend: (5, 2),
        }
    }

    /// The outline an arrangement really has: four corners, not three.
    pub const fn outline(a: &Arrangement) -> [P; 4] {
        [(0, 0), (WIDE, 0), (WIDE, TALL), a.bend]
    }

    /// The slope of a triangle's long side, rise over run.
    pub fn slope(t: &[P; 3]) -> Q {
        // right angle first, then the two ends of the long side
        let (rise, run) = ((t[1].1 - t[2].1).abs(), (t[1].0 - t[2].0).abs());
        q(i128::from(rise), i128::from(run))
    }

    /// How high the straight edge from corner to corner would be at x.
    pub fn straight_edge_at(x: i64) -> Q {
        q(i128::from(TALL * x), i128::from(WIDE))
    }

    /// How high an arrangement's sloping edge really is at x: one slope up
    /// to the bend, the other after it.
    pub fn edge_at(a: &Arrangement, x: i64) -> Q {
        let (bx, by) = a.bend;
        if x <= bx {
            q(i128::from(by * x), i128::from(bx))
        } else {
            Q::int(by) + q(i128::from((TALL - by) * (x - bx)), i128::from(WIDE - bx))
        }
    }

    /// The unit squares inside the rectangle under the triangles that no
    /// piece covers.
    pub fn holes(a: &Arrangement) -> Vec<P> {
        // the rectangle the two L-shaped pieces share: right of the left
        // triangle, below the right one
        let (x0, y1) = if a.bend == (8, 3) { (8, 3) } else { (5, 2) };
        (x0..WIDE)
            .flat_map(|x| (0..y1).map(move |y| (x, y)))
            .filter(|c| !a.long_l.contains(c) && !a.short_l.contains(c))
            .collect()
    }
}

/// What a quiz asks; `facts.rs` computes each answer from the pictures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ask {
    /// How many regions six points make.
    Regions,
    /// How many unit squares the next L-shaped layer needs.
    NextLayer,
    /// The rectangle two staircases make.
    Rectangle,
    /// How many of the fourth cube's slabs have to be cut.
    CutSlabs,
    /// How many stacks make a box.
    Stacks,
    /// The share of the quartered square one colour holds.
    Share,
    /// How many blocks of each kind the cut cube has.
    Kinds,
    /// The area of the tilted square on legs 3 and 4.
    Tilted,
    /// Which of the two slopes of the lying picture is steeper.
    Slopes,
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
    quiz("regions", "02_5", 4, Ask::Regions),
    quiz("next_layer", "03_3", 4, Ask::NextLayer),
    quiz("rectangle", "04_3", 5, Ask::Rectangle),
    quiz("cut_slabs", "05_4", 6, Ask::CutSlabs),
    quiz("stacks", "07_2", 5, Ask::Stacks),
    quiz("share", "11_3", 6, Ask::Share),
    quiz("kinds", "12_4", 6, Ask::Kinds),
    quiz("tilted", "13_4", 6, Ask::Tilted),
    quiz("slopes", "14_4", 6, Ask::Slopes),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fractions_reduce_compare_and_print() {
        assert_eq!(q(20, 100), q(1, 5));
        assert_eq!(q(1, -2), q(-1, 2));
        assert!(q(3, 8) < q(5, 13) && q(5, 13) < q(2, 5));
        assert_eq!((q(1, 2) + q(1, 3)).to_string(), "5/6");
        assert_eq!((q(1, 2) - q(1, 2)).to_string(), "0");
        assert_eq!((q(3, 4) / q(3, 8)).to_string(), "2");
        assert_eq!(Q::half_to(5), q(1, 32));
    }

    #[test]
    fn the_circle_s_regions_break_the_doubling_pattern_at_six() {
        let counts: Vec<u64> = (0..=7).map(|n| circle::regions(&circle::drawn(n))).collect();
        assert_eq!(counts, [1, 1, 2, 4, 8, 16, 31, 57]);
        for n in 0..=7u64 {
            assert_eq!(counts[n as usize], circle::closed_form(n), "n = {n}");
        }
        // no three of the drawn lines meet at one point inside: every four
        // points give their own crossing
        assert_eq!(circle::crossings(&circle::drawn(6)).len() as u64, choose(6, 4));
        assert_eq!(circle::crossings(&circle::drawn(7)).len() as u64, choose(7, 4));
        // every drawn point is on the unit circle
        for (x, y) in circle::drawn(7) {
            assert_eq!(x * x + y * y, Q::ONE);
        }
    }

    #[test]
    fn three_lines_through_one_point_lose_a_region() {
        let six = circle::three_through_one();
        assert_eq!(circle::crossings(&six).len(), 13);
        assert_eq!(circle::regions(&six), 30);
    }

    #[test]
    fn l_shaped_layers_build_every_square() {
        for n in 1..=60 {
            let cells = (1..=n).flat_map(layers::layer);
            tiles(cells, Rect::new(0, 0, n, n), &[]).unwrap();
            assert_eq!(layers::layer(n).len() as i64, 2 * n - 1);
        }
    }

    #[test]
    fn two_staircases_make_a_rectangle() {
        for n in 1..=60 {
            let both = layers::staircase(n).into_iter().chain(layers::turned(n));
            tiles(both, Rect::new(0, 0, n, n + 1), &[]).unwrap();
            assert_eq!(layers::staircase(n).len() as i64, n * (n + 1) / 2);
        }
    }

    #[test]
    fn the_cubes_slabs_tile_the_square() {
        for n in 1..=40 {
            let side = cubes::side(n);
            let pieces = cubes::square(n);
            tiles(
                pieces.iter().flat_map(|p| p.rect.cells()),
                Rect::new(0, 0, side, side),
                &[],
            )
            .unwrap();
            // layer n is cube n: n slabs of n by n, one of them cut when n is even
            let layer = cubes::layer(n);
            assert_eq!(layer.iter().map(|p| p.rect.area()).sum::<i64>(), n * n * n);
            let mut slabs: Vec<i64> = layer.iter().map(|p| p.slab).collect();
            slabs.sort_unstable();
            slabs.dedup();
            assert_eq!(slabs, (0..n).collect::<Vec<_>>());
            let halves = layer.iter().filter(|p| p.part != cubes::Part::Whole).count();
            assert_eq!(halves, if n % 2 == 0 { 2 } else { 0 });
            for p in &layer {
                match p.part {
                    cubes::Part::Whole => assert_eq!((p.rect.w, p.rect.h), (n, n)),
                    cubes::Part::HalfRight => assert_eq!((p.rect.w, p.rect.h), (n, n / 2)),
                    cubes::Part::HalfTop => assert_eq!((p.rect.w, p.rect.h), (n / 2, n)),
                }
            }
        }
    }

    #[test]
    fn each_cube_is_a_run_of_consecutive_odd_numbers() {
        let mut next_odd = 1;
        for k in 1..=200 {
            let run = cubes::odd_run(k);
            assert_eq!(run.len() as i64, k);
            for &o in &run {
                assert_eq!(o, next_odd, "the runs use every odd number once, in order");
                next_odd += 2;
            }
            assert_eq!(run.iter().sum::<i64>(), k * k * k);
        }
        assert_eq!(cubes::odd_run(3), [7, 9, 11]);
    }

    #[test]
    fn three_stacks_fill_siu_s_box() {
        for n in 1..=24 {
            stacks::fills_the_box(n).unwrap();
            let squares: i64 = (1..=n).map(|k| k * k).sum();
            for w in 0..3 {
                assert_eq!(stacks::placed(n, w).len() as i64, squares);
            }
            assert_eq!(stacks::top(n).len() as i64, n * (n + 1) / 2);
            // the three stacks, unsliced, share no cell
            let mut all: Vec<_> = (0..3).flat_map(|w| stacks::placed(n, w)).collect();
            all.sort_unstable();
            all.dedup();
            assert_eq!(all.len() as i64, 3 * squares);
        }
        for m in stacks::TURNS {
            assert_eq!(stacks::determinant(m), 1, "a turn, not a mirror image");
        }
    }

    #[test]
    fn the_halves_and_the_quarters_fill_the_unit_square() {
        let mut filled = Q::ZERO;
        for k in 1..=40 {
            let p = series::half(k);
            assert_eq!(series::area(p), Q::half_to(k));
            filled = filled + series::area(p);
            let left = series::left_after_halves(k);
            assert_eq!(filled + series::area(left), Q::ONE);
            assert_eq!(series::area(left), Q::half_to(k), "the gap is the last piece");
            // the piece and what is left split what was left before
            let before = series::left_after_halves(k - 1);
            assert_eq!((p.0, p.1), (before.0, before.1));
            assert_eq!(series::area(p) + series::area(left), series::area(before));
        }
        let mut share = Q::ZERO;
        for j in 1..=20 {
            let [a, b, c] = series::quarters(j);
            assert!(series::area(a) == series::area(b) && series::area(b) == series::area(c));
            share = share + series::area(a);
            let left = series::left_after_quarters(j);
            assert_eq!(share + share + share + series::area(left), Q::ONE);
            assert_eq!(q(1, 3) - share, series::area(left) * q(1, 3));
        }
    }

    #[test]
    fn the_cut_cube_s_blocks_fill_it_in_every_dimension() {
        for dim in 1..=4 {
            for a in 1..=5 {
                for b in 1..=5 {
                    blocks::fills_the_cube(a, b, dim).unwrap();
                }
            }
            let kinds = blocks::kinds(dim);
            for (j, &count) in kinds.iter().enumerate() {
                assert_eq!(count as u64, choose(dim as u64, j as u64));
            }
            assert_eq!(kinds.iter().sum::<usize>(), 1 << dim);
        }
        assert_eq!(blocks::kinds(3), [1, 3, 3, 1]);
        assert_eq!(blocks::kinds(4), [1, 4, 6, 4, 1]);
    }

    #[test]
    fn each_height_is_the_last_one_on_three_new_slabs() {
        for n in 1..=24 {
            stacks::grows(n).unwrap();
            // the three new slabs share no cube, and none of them is in the lifted solid
            let mut slabs: Vec<_> = (0..3).flat_map(|w| stacks::new_base(n, w)).collect();
            slabs.sort_unstable();
            slabs.dedup();
            assert_eq!(slabs.len() as i64, 3 * n * n);
        }
        // one under (z = 0), one behind (y = n), one beside (x = n − 1)
        let n = 5;
        assert!(stacks::new_base(n, 0).iter().all(|c| c.2 == 0));
        assert!(stacks::new_base(n, 1).iter().all(|c| c.1 == n));
        assert!(stacks::new_base(n, 2).iter().all(|c| c.0 == n - 1));
    }

    #[test]
    fn four_triangles_two_ways() {
        use pythagoras::{area2, corners, is_square, left_over, slid, straight, tilted};
        for a in 1..=60 {
            for b in 1..=60 {
                let s = a + b;
                let t = corners(a, b);
                assert!(t.iter().all(|tri| area2(tri) == a * b), "four equal triangles");
                let hole = tilted(a, b);
                assert!(is_square(&hole));
                assert_eq!(area2(&hole), 2 * (a * a + b * b));
                assert_eq!(2 * s * s, area2(&hole) + t.iter().map(|x| area2(x)).sum::<i64>());
                // the triangles meet along the big square's straight sides
                assert!(straight((0, 0), (a, 0), (s, 0)) && straight((s, 0), (s, a), (s, s)));
                let u = slid(a, b);
                assert!(u.iter().all(|tri| area2(tri).abs() == a * b));
                // each pair of slid triangles is one a by b rectangle cut corner to corner
                let mut first: Vec<_> = u[0].iter().chain(&u[2]).copied().collect();
                first.sort_unstable();
                first.dedup();
                assert_eq!(first, [(0, 0), (0, b), (a, 0), (a, b)]);
                let mut second: Vec<_> = u[1].iter().chain(&u[3]).copied().collect();
                second.sort_unstable();
                second.dedup();
                assert_eq!(second, [(a, b), (a, s), (s, b), (s, s)]);
                let (on_a, on_b) = left_over(a, b);
                assert_eq!((on_a.area(), on_b.area()), (a * a, b * b));
                let rects = [Rect::new(0, 0, a, b), Rect::new(a, b, b, a), on_a, on_b];
                tiles(rects.iter().flat_map(Rect::cells), Rect::new(0, 0, s, s), &[]).unwrap();
            }
        }
    }

    #[test]
    fn the_lying_picture_is_not_a_triangle() {
        use pythagoras::{area2, straight};
        let (a, b) = (lie::first(), lie::second());
        for arr in [&a, &b] {
            assert_eq!(area2(&arr.big).abs(), 24);
            assert_eq!(area2(&arr.small).abs(), 10);
            assert_eq!((arr.long_l.len(), arr.short_l.len()), (8, 7));
            assert_eq!(lie::slope(&arr.big), q(3, 8));
            assert_eq!(lie::slope(&arr.small), q(2, 5));
            let mut both = arr.long_l.clone();
            both.extend(&arr.short_l);
            both.sort_unstable();
            both.dedup();
            assert_eq!(both.len(), 15, "the two L-shaped pieces share no square");
            assert!(!straight((0, 0), arr.bend, (lie::WIDE, lie::TALL)));
        }
        assert_eq!(lie::holes(&a), []);
        assert_eq!(lie::holes(&b), [(7, 1)]);
        // the outlines: 32 squares and 33, where a true triangle would hold 32 and a half
        assert_eq!(area2(&lie::outline(&a)), 64);
        assert_eq!(area2(&lie::outline(&b)), 66);
        assert_eq!(lie::WIDE * lie::TALL, 65);
        // the first outline dips under the straight edge, the second rises over it
        assert!(Q::int(a.bend.1) < lie::straight_edge_at(a.bend.0));
        assert!(Q::int(b.bend.1) > lie::straight_edge_at(b.bend.0));
        // both edges start and end at the corners, and the second is never below the first
        for arr in [&a, &b] {
            assert_eq!(lie::edge_at(arr, 0), Q::ZERO);
            assert_eq!(lie::edge_at(arr, lie::WIDE), Q::int(lie::TALL));
            assert_eq!(lie::edge_at(arr, arr.bend.0), Q::int(arr.bend.1));
        }
        assert!((0..=lie::WIDE).all(|x| lie::edge_at(&b, x) >= lie::edge_at(&a, x)));
    }

    #[test]
    fn first_failure_reports_the_first_bad_size() {
        assert_eq!(first_failure(10, |_| Ok(())), None);
        let bad = first_failure(10, |n| if n < 4 { Ok(()) } else { Err("no".into()) });
        assert_eq!(bad, Some((4, "no".to_owned())));
    }
}
