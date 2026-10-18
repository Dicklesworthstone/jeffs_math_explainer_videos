//! The mathematics of the five-tile video: exact numbers of the form
//! a + b√2, unit tiles as poses, the box, and the one test everything else
//! rests on: do two tiles overlap, touch or stand apart, and is a tile inside
//! the box.
//!
//! The video keeps three questions apart, and this file answers only the
//! first two:
//! - **Does it fit?** A packing (a box side and five poses) is checked
//!   exactly: every pair of tiles by the separating-axis test
//!   ([`clearance`]), every tile against the four walls
//!   ([`wall_clearance`]), in exact arithmetic on [`Surd`]s, so a contact is
//!   exactly zero and a clearance exactly positive, never "within 1e-12".
//!   [`five_turned`] fits; each [`planted`] failure overlaps or escapes, and
//!   the check says which tiles.
//! - **Can it move there?** The chapter's two motions, the middle tile
//!   turning in the box of three ([`turning`]) and the squeeze ([`squeezed`]),
//!   are checked frame by frame in f64 by the tests below, and each doc says
//!   why the frames between the samples are packings too.
//! - **Could a smaller box work?** Not answered here. A packing that fits is
//!   an upper bound only; that no box narrower than [`turned_side`] holds five
//!   tiles is Göbel's theorem (1979; fact sheet §Sources), which this file
//!   does not prove.
//!
//! A tile's index in every packing: 0 lower left, 1 lower right, 2 upper
//! left, 3 upper right (the corner tiles, always straight), 4 the middle.

use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Mul, Neg, Sub};

/// How many tiles the video packs.
pub const TILES: u32 = 5;

/// The middle tile's index.
pub const MIDDLE: usize = 4;

/// Of each corner tile, the corner nearest the box's centre (its inner
/// corner), as an index into [`Tile::corners`].
pub const INNER: [usize; 4] = [2, 3, 1, 0];

// ------------------------------------------------------------ exact numbers

const fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

fn times(a: i128, b: i128) -> i128 {
    a.checked_mul(b).expect("exact arithmetic overflows i128")
}

fn plus(a: i128, b: i128) -> i128 {
    a.checked_add(b).expect("exact arithmetic overflows i128")
}

/// A rational number in lowest terms, its denominator positive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Ratio {
    num: i128,
    den: i128,
}

impl Ratio {
    pub const ZERO: Self = Self { num: 0, den: 1 };
    pub const ONE: Self = Self { num: 1, den: 1 };

    /// `num / den` in lowest terms.
    ///
    /// # Panics
    /// If `den` is zero.
    pub fn new(num: i128, den: i128) -> Self {
        assert!(den != 0, "a ratio with a zero denominator");
        let g = i128::try_from(gcd(num.unsigned_abs(), den.unsigned_abs())).expect("a divisor fits i128");
        let (num, den) = (num / g, den / g);
        if den < 0 {
            Self { num: -num, den: -den }
        } else {
            Self { num, den }
        }
    }

    pub fn int(n: i64) -> Self {
        Self::new(n.into(), 1)
    }

    pub fn signum(self) -> i32 {
        self.num.signum() as i32
    }

    pub fn to_f64(self) -> f64 {
        self.num as f64 / self.den as f64
    }
}

impl Add for Ratio {
    type Output = Self;
    fn add(self, o: Self) -> Self {
        Self::new(
            plus(times(self.num, o.den), times(o.num, self.den)),
            times(self.den, o.den),
        )
    }
}

impl Sub for Ratio {
    type Output = Self;
    fn sub(self, o: Self) -> Self {
        self + -o
    }
}

impl Mul for Ratio {
    type Output = Self;
    fn mul(self, o: Self) -> Self {
        Self::new(times(self.num, o.num), times(self.den, o.den))
    }
}

impl Neg for Ratio {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            num: -self.num,
            den: self.den,
        }
    }
}

impl Ord for Ratio {
    fn cmp(&self, o: &Self) -> Ordering {
        times(self.num, o.den).cmp(&times(o.num, self.den))
    }
}

impl PartialOrd for Ratio {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

impl fmt::Display for Ratio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.den == 1 {
            write!(f, "{}", self.num)
        } else {
            write!(f, "{}/{}", self.num, self.den)
        }
    }
}

/// An exact number a + b√2, a and b rational: every coordinate of the
/// five-tile packing and of its planted failures is one, and so is every
/// clearance the checks compute from them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Surd {
    pub a: Ratio,
    pub b: Ratio,
}

impl Surd {
    pub const ZERO: Self = Self {
        a: Ratio::ZERO,
        b: Ratio::ZERO,
    };

    pub fn int(n: i64) -> Self {
        Self {
            a: Ratio::int(n),
            b: Ratio::ZERO,
        }
    }

    pub fn ratio(p: i64, q: i64) -> Self {
        Self {
            a: Ratio::new(p.into(), q.into()),
            b: Ratio::ZERO,
        }
    }

    /// √2.
    pub fn sqrt2() -> Self {
        Self {
            a: Ratio::ZERO,
            b: Ratio::ONE,
        }
    }

    /// The sign, exactly: when a and b differ in sign, a + b√2 has the sign
    /// of a if a² > 2b² and of b otherwise (a² = 2b² only when both are 0,
    /// since √2 is irrational).
    pub fn signum(self) -> i32 {
        let (sa, sb) = (self.a.signum(), self.b.signum());
        if sa >= 0 && sb >= 0 {
            return i32::from(sa > 0 || sb > 0);
        }
        if sa <= 0 && sb <= 0 {
            return -1;
        }
        let d = self.a * self.a - Ratio::int(2) * self.b * self.b;
        if d.signum() > 0 { sa } else { sb }
    }

    pub fn to_f64(self) -> f64 {
        self.a.to_f64() + self.b.to_f64() * std::f64::consts::SQRT_2
    }

    /// Rounded to `places` decimals, as the screen and the narration give it.
    pub fn decimals(self, places: usize) -> String {
        format!("{:.places$}", self.to_f64())
    }
}

impl Add for Surd {
    type Output = Self;
    fn add(self, o: Self) -> Self {
        Self {
            a: self.a + o.a,
            b: self.b + o.b,
        }
    }
}

impl Sub for Surd {
    type Output = Self;
    fn sub(self, o: Self) -> Self {
        self + -o
    }
}

impl Mul for Surd {
    type Output = Self;
    fn mul(self, o: Self) -> Self {
        Self {
            a: self.a * o.a + Ratio::int(2) * self.b * o.b,
            b: self.a * o.b + self.b * o.a,
        }
    }
}

impl Neg for Surd {
    type Output = Self;
    fn neg(self) -> Self {
        Self { a: -self.a, b: -self.b }
    }
}

impl Ord for Surd {
    fn cmp(&self, o: &Self) -> Ordering {
        (*self - *o).signum().cmp(&0)
    }
}

impl PartialOrd for Surd {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

/// `a + b*sqrt(2)` in ASCII, the form the facts print (and `oracle.py`):
/// `2 + 1/2*sqrt(2)`, `1/2*sqrt(2)`, `1 - 1/4*sqrt(2)`, `3`.
impl fmt::Display for Surd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let root = |b: Ratio| {
            if b == Ratio::ONE {
                "sqrt(2)".to_owned()
            } else {
                format!("{b}*sqrt(2)")
            }
        };
        match (self.a == Ratio::ZERO, self.b.signum()) {
            (_, 0) => write!(f, "{}", self.a),
            (true, 1) => write!(f, "{}", root(self.b)),
            (true, _) => write!(f, "-{}", root(-self.b)),
            (false, 1) => write!(f, "{} + {}", self.a, root(self.b)),
            (false, _) => write!(f, "{} - {}", self.a, root(-self.b)),
        }
    }
}

/// The arithmetic the geometry needs, so the same checks run exactly (on
/// [`Surd`]s, for the facts) and in floating point (on f64, for the frames
/// the scene draws).
pub trait Num:
    Copy + PartialOrd + Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self> + Neg<Output = Self>
{
    /// The rational p / q.
    fn ratio(p: i64, q: i64) -> Self;
}

impl Num for f64 {
    fn ratio(p: i64, q: i64) -> Self {
        p as f64 / q as f64
    }
}

impl Num for Surd {
    fn ratio(p: i64, q: i64) -> Self {
        Surd::ratio(p, q)
    }
}

fn larger<T: PartialOrd>(a: T, b: T) -> T {
    if a >= b { a } else { b }
}

fn smaller<T: PartialOrd>(a: T, b: T) -> T {
    if a <= b { a } else { b }
}

// ----------------------------------------------------------------- tiles

pub type Point<T> = (T, T);

/// A unit tile: its centre and its turn, given as the cosine and sine of
/// the angle it is turned through (exact for every turn the video uses;
/// cos² + sin² = 1, which [`Packing::check`] confirms for exact tiles).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tile<T> {
    pub centre: Point<T>,
    pub cos: T,
    pub sin: T,
}

impl<T: Num> Tile<T> {
    /// A tile lined up with the walls.
    pub fn straight(x: T, y: T) -> Self {
        Self {
            centre: (x, y),
            cos: T::ratio(1, 1),
            sin: T::ratio(0, 1),
        }
    }

    /// Its corners, counter-clockwise, starting from the one that is lower
    /// left when the tile is straight.
    pub fn corners(&self) -> [Point<T>; 4] {
        let h = T::ratio(1, 2);
        let (ux, uy) = (self.cos * h, self.sin * h);
        let (vx, vy) = (-self.sin * h, self.cos * h);
        let (cx, cy) = self.centre;
        [
            (cx - ux - vx, cy - uy - vy),
            (cx + ux - vx, cy + uy - vy),
            (cx + ux + vx, cy + uy + vy),
            (cx - ux + vx, cy - uy + vy),
        ]
    }

    /// The midpoints of its sides: side k runs from corner k to corner k + 1.
    pub fn midpoints(&self) -> [Point<T>; 4] {
        let c = self.corners();
        let h = T::ratio(1, 2);
        std::array::from_fn(|k| {
            let (p, q) = (c[k], c[(k + 1) % 4]);
            ((p.0 + q.0) * h, (p.1 + q.1) * h)
        })
    }

    /// The two directions its sides run (unit vectors).
    pub fn axes(&self) -> [Point<T>; 2] {
        [(self.cos, self.sin), (-self.sin, self.cos)]
    }
}

/// The tile's shadow on a direction: the least and greatest of its corners'
/// positions along it.
fn shadow<T: Num>(corners: &[Point<T>; 4], axis: Point<T>) -> (T, T) {
    let at = |p: Point<T>| p.0 * axis.0 + p.1 * axis.1;
    let first = at(corners[0]);
    corners[1..]
        .iter()
        .fold((first, first), |(lo, hi), &p| (smaller(lo, at(p)), larger(hi, at(p))))
}

/// How far apart two tiles stand: the widest gap between their shadows on
/// the four directions their sides run (the separating-axis test, exact for
/// convex polygons). Positive: apart, by at least that much. Zero: touching,
/// with no overlap. Negative: their insides overlap, and minus the value is
/// the shortest push that would part them.
pub fn clearance<T: Num>(a: &Tile<T>, b: &Tile<T>) -> T {
    let (pa, pb) = (a.corners(), b.corners());
    let [a1, a2] = a.axes();
    let [b1, b2] = b.axes();
    let gap = |axis: Point<T>| {
        let ((alo, ahi), (blo, bhi)) = (shadow(&pa, axis), shadow(&pb, axis));
        larger(blo - ahi, alo - bhi)
    };
    larger(larger(gap(a1), gap(a2)), larger(gap(b1), gap(b2)))
}

/// A wall of the box [0, side]².
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wall {
    Left,
    Bottom,
    Right,
    Top,
}

impl Wall {
    pub const ALL: [Wall; 4] = [Wall::Left, Wall::Bottom, Wall::Right, Wall::Top];

    /// How far `p` is inside this wall (negative: beyond it).
    fn depth<T: Num>(self, p: Point<T>, side: T) -> T {
        match self {
            Wall::Left => p.0,
            Wall::Bottom => p.1,
            Wall::Right => side - p.0,
            Wall::Top => side - p.1,
        }
    }

    /// How far tile `t` stands inside this wall: its nearest corner's depth.
    pub fn clearance<T: Num>(self, t: &Tile<T>, side: T) -> T {
        let c = t.corners();
        c[1..]
            .iter()
            .fold(self.depth(c[0], side), |m, &p| smaller(m, self.depth(p, side)))
    }
}

/// How far a tile stands inside the box [0, side]²: the least distance from
/// any of its corners to any wall (a tile is convex, so its corners decide).
/// Positive: inside and clear; zero: touching a wall; negative: a corner is
/// outside, and the tile has escaped.
pub fn wall_clearance<T: Num>(t: &Tile<T>, side: T) -> T {
    Wall::ALL
        .iter()
        .map(|w| w.clearance(t, side))
        .reduce(smaller)
        .expect("four walls")
}

/// A clearance read with a tolerance: 0 for exact numbers, a hair for f64.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Contact {
    Apart,
    Touching,
    Overlapping,
}

pub fn contact<T: Num>(clearance: T, eps: T) -> Contact {
    if clearance > eps {
        Contact::Apart
    } else if clearance < -eps {
        Contact::Overlapping
    } else {
        Contact::Touching
    }
}

// ---------------------------------------------------------------- packings

/// Tiles in the box [0, side]².
#[derive(Clone, Debug, PartialEq)]
pub struct Packing<T> {
    pub side: T,
    pub tiles: Vec<Tile<T>>,
}

/// What a check of a packing found.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Verdict {
    /// Pairs whose insides overlap.
    pub overlapping: Vec<(usize, usize)>,
    /// Pairs that touch without overlapping.
    pub touching: Vec<(usize, usize)>,
    /// Tiles with a corner outside the box.
    pub escaped: Vec<usize>,
    /// Every wall each tile touches.
    pub walls: Vec<(usize, Wall)>,
    /// Exact tiles whose cosine and sine are not a turn (cos² + sin² ≠ 1).
    pub not_turns: Vec<usize>,
}

impl Verdict {
    pub fn fits(&self) -> bool {
        self.overlapping.is_empty() && self.escaped.is_empty() && self.not_turns.is_empty()
    }

    /// `fits`, or what broke, as the facts print it: `overlap:0-4,1-4`,
    /// `escape:1,2,3`, or both joined by `;`.
    pub fn summary(&self) -> String {
        if self.fits() {
            return "fits".to_owned();
        }
        let mut parts = Vec::new();
        if !self.overlapping.is_empty() {
            let pairs: Vec<String> = self.overlapping.iter().map(|(i, j)| format!("{i}-{j}")).collect();
            parts.push(format!("overlap:{}", pairs.join(",")));
        }
        if !self.escaped.is_empty() {
            let tiles: Vec<String> = self.escaped.iter().map(usize::to_string).collect();
            parts.push(format!("escape:{}", tiles.join(",")));
        }
        if !self.not_turns.is_empty() {
            let tiles: Vec<String> = self.not_turns.iter().map(usize::to_string).collect();
            parts.push(format!("not-a-turn:{}", tiles.join(",")));
        }
        parts.join(";")
    }

    /// The touching pairs as the facts print them: `0-4,1-4,2-4,3-4`.
    pub fn touching_list(&self) -> String {
        let pairs: Vec<String> = self.touching.iter().map(|(i, j)| format!("{i}-{j}")).collect();
        pairs.join(",")
    }
}

impl<T: Num> Packing<T> {
    /// Every pair of tiles and every tile against every wall, a clearance
    /// within `eps` of zero counting as a touch (pass zero for exact tiles).
    pub fn check(&self, eps: T) -> Verdict {
        let mut v = Verdict::default();
        for (i, a) in self.tiles.iter().enumerate() {
            for (j, b) in self.tiles.iter().enumerate().skip(i + 1) {
                match contact(clearance(a, b), eps) {
                    Contact::Overlapping => v.overlapping.push((i, j)),
                    Contact::Touching => v.touching.push((i, j)),
                    Contact::Apart => {}
                }
            }
            if wall_clearance(a, self.side) < -eps {
                v.escaped.push(i);
            }
            for w in Wall::ALL {
                if contact(w.clearance(a, self.side), eps) == Contact::Touching {
                    v.walls.push((i, w));
                }
            }
        }
        v
    }

    /// The least clearance anywhere: between two tiles, or between a tile and
    /// the walls. Non-negative exactly when the packing fits. The tests read
    /// it on every sampled frame of the chapter's motions; the scene draws
    /// those frames and never needs the number.
    #[cfg(test)]
    pub fn worst(&self) -> T {
        let mut worst = wall_clearance(&self.tiles[0], self.side);
        for (i, a) in self.tiles.iter().enumerate() {
            worst = smaller(worst, wall_clearance(a, self.side));
            for b in &self.tiles[i + 1..] {
                worst = smaller(worst, clearance(a, b));
            }
        }
        worst
    }

    /// Corner tile `k`'s inner corner (k from 0 to 3).
    pub fn inner_corner(&self, k: usize) -> Point<T> {
        self.tiles[k].corners()[INNER[k]]
    }
}

impl Packing<Surd> {
    /// The exact check, which also confirms that every tile's turn is one.
    pub fn check_exact(&self) -> Verdict {
        let mut v = self.check(Surd::ZERO);
        v.not_turns = self
            .tiles
            .iter()
            .enumerate()
            .filter(|(_, t)| t.cos * t.cos + t.sin * t.sin != Surd::int(1))
            .map(|(i, _)| i)
            .collect();
        v
    }
}

/// Four tiles straight in the corners of a box `side` wide, and one in the
/// middle turned through (`cos`, `sin`): every picture of the chapter.
pub fn corners_and_middle<T: Num>(side: T, cos: T, sin: T) -> Packing<T> {
    let h = T::ratio(1, 2);
    let (lo, hi, mid) = (h, side - h, side * h);
    Packing {
        side,
        tiles: vec![
            Tile::straight(lo, lo),
            Tile::straight(hi, lo),
            Tile::straight(lo, hi),
            Tile::straight(hi, hi),
            Tile {
                centre: (mid, mid),
                cos,
                sin,
            },
        ],
    }
}

/// The side of the box a straight grid needs for `n` tiles: the smallest k
/// with k² ≥ n.
pub fn grid_side(n: u32) -> u32 {
    (1..=n).find(|k| k * k >= n).unwrap_or(n)
}

/// The largest full straight grid that `n` tiles can make: the largest k
/// with k² ≤ n.
pub fn full_grid(n: u32) -> u32 {
    (1..=n).take_while(|k| k * k <= n).last().unwrap_or(0)
}

/// The obvious packing: the five tiles straight in the box of three, four in
/// the corners and one in the middle.
pub fn five_straight() -> Packing<Surd> {
    let side = Surd::int(grid_side(TILES).into());
    corners_and_middle(side, Surd::int(1), Surd::ZERO)
}

/// An eighth of a turn, 45 degrees: cos = sin = √2/2.
pub fn eighth_turn() -> (Surd, Surd) {
    let c = Surd::sqrt2() * Surd::ratio(1, 2);
    (c, c)
}

/// The gap between the corner tiles when the squeeze stops, derived as the
/// narration derives it. The gap is a square. Its diagonal runs between
/// opposite inner corners, which then touch opposite sides of the turned
/// tile, and those sides are one tile apart: the diagonal is 1. A square's
/// diagonal is √2 times its side, so the side is 1/√2 = √2/2.
pub fn turned_gap() -> Surd {
    Surd::sqrt2() * Surd::ratio(1, 2)
}

/// The box the squeeze stops at: a corner tile, the gap and a corner tile,
/// 2 + √2/2. The tests check that the packing there fits and touches
/// exactly, and that any narrower box with the same arrangement overlaps.
pub fn turned_side() -> Surd {
    Surd::int(2) + turned_gap()
}

/// The five tiles of the video: four straight in the corners of the box
/// [`turned_side`] wide, the middle tile an eighth of a turn round.
pub fn five_turned() -> Packing<Surd> {
    let (c, s) = eighth_turn();
    corners_and_middle(turned_side(), c, s)
}

/// The turn by `a` and then by `b`, each as (cos, sin).
pub fn compose<T: Num>(a: Point<T>, b: Point<T>) -> Point<T> {
    (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0)
}

/// A small exact turn, about 5.7 degrees: the angle whose half has tangent
/// 1/20, so cos = 399/401 and sin = 40/401 (a Pythagorean triple).
pub fn small_turn() -> (Surd, Surd) {
    (Surd::ratio(399, 401), Surd::ratio(40, 401))
}

// ---------------------------------------------------- the chapter's motions

/// The box the turn happens in: the straight grid's, three wide.
pub fn turn_box() -> f64 {
    f64::from(grid_side(TILES))
}

/// The first motion, frame by frame as the chapter draws it (01_5): in the
/// box of three ([`turn_box`]), the middle tile turned through
/// `angle` radians (0 straight, π/4 an eighth of a turn). Every angle gives a
/// packing: each corner of the middle tile is √2/2 from the box's centre, so
/// the whole tile lies in the disc of that radius about the centre; the point
/// of a corner tile nearest the centre is its inner corner, also √2/2 away
/// (the squared distances are `turn_reach2` and `turn_inner2`). So the disc
/// meets each corner tile only at its inner corner, and the insides never
/// overlap. The middle tile touches its neighbours only when it is straight.
pub fn turning(angle: f64) -> Packing<f64> {
    corners_and_middle(turn_box(), angle.cos(), angle.sin())
}

/// The second motion, frame by frame as the chapter draws it (01_6 and
/// 01_10): the box `side` wide, from 3 down to [`turned_side`],
/// the corner tiles in its corners and the middle tile an eighth of a turn
/// round at its centre. Every side in that range gives a packing: the turned
/// tile is the set where |x − c| + |y − c| ≤ √2/2 (c = side/2), the point
/// of a corner tile nearest it in that measure is the inner corner, at
/// side − 2, and side − 2 ≥ √2/2 down to the stop, where it is equal.
/// Below the stop every frame overlaps: 01_10 shows those in red, as what
/// fails, never as a packing.
pub fn squeezed(side: f64) -> Packing<f64> {
    let c = std::f64::consts::FRAC_1_SQRT_2;
    corners_and_middle(side, c, c)
}

/// The squared distance from the middle tile's centre to each of its
/// corners, at any turn: (1/2)² + (1/2)².
pub fn turn_reach2() -> Surd {
    let t = Tile::<Surd>::straight(Surd::ZERO, Surd::ZERO);
    let (x, y) = t.corners()[0];
    x * x + y * y
}

/// The squared distance from the centre of the box of three to each inner
/// corner.
pub fn turn_inner2() -> Surd {
    let p = five_straight();
    let m = Surd::int(grid_side(TILES).into()) * Surd::ratio(1, 2);
    let (x, y) = p.inner_corner(0);
    (x - m) * (x - m) + (y - m) * (y - m)
}

// ------------------------------------------------------ planted failures

/// The five-tile packing changed by a little; the check must say what broke.
#[derive(Clone, Debug)]
pub struct Planted {
    /// The fact key's last part.
    pub key: &'static str,
    pub packing: Packing<Surd>,
}

/// The planted failures, each of which the exact check and `oracle.py` must
/// catch:
/// - `squeeze_further`: the box 1/100 narrower than [`turned_side`], the
///   same arrangement (the chapter's "one step further");
/// - `middle_turned_more`: the middle tile turned a further [`small_turn`];
/// - `middle_shifted`: the middle tile moved 1/100 to the right;
/// - `box_smaller`: the five tiles where they are, the box's right and top
///   walls 1/100 nearer;
/// - `straight_squeezed`: the box 1/100 narrower than three with the middle
///   tile straight (squeezing without the turn).
pub fn planted() -> Vec<Planted> {
    let hundredth = Surd::ratio(1, 100);
    let (c, s) = eighth_turn();
    let turned_more = compose((c, s), small_turn());
    let mut shifted = five_turned();
    let (x, y) = shifted.tiles[MIDDLE].centre;
    shifted.tiles[MIDDLE].centre = (x + hundredth, y);
    let smaller_box = Packing {
        side: turned_side() - hundredth,
        tiles: five_turned().tiles,
    };
    let three = Surd::int(grid_side(TILES).into());
    vec![
        Planted {
            key: "squeeze_further",
            packing: corners_and_middle(turned_side() - hundredth, c, s),
        },
        Planted {
            key: "middle_turned_more",
            packing: corners_and_middle(turned_side(), turned_more.0, turned_more.1),
        },
        Planted {
            key: "middle_shifted",
            packing: shifted,
        },
        Planted {
            key: "box_smaller",
            packing: smaller_box,
        },
        Planted {
            key: "straight_squeezed",
            packing: corners_and_middle(three - hundredth, Surd::int(1), Surd::ZERO),
        },
    ]
}

// ------------------------------------------------- four pins (chapter 02)
//
// The theorem: in a square box narrower than 3, at most four straight tiles
// fit, so five straight tiles need a box at least 3 wide.
//
// Proof. Let the box be [0, L]² with L < 3. If L < 1, no tile fits at all.
// Otherwise mark each side at L/3 and 2L/3. The marks cut [0, L] into three
// pieces of length L/3 < 1. Take a straight tile; its shadow on the floor is
// [a, a + 1] ⊆ [0, L]. If no mark were strictly inside, the open interval
// (a, a + 1), being connected and missing both marks, would lie in one
// piece; but it is 1 long and every piece is shorter. So a floor mark lies
// strictly inside (a, a + 1), and likewise a wall mark inside the shadow on
// the wall: the pin where they cross is in the tile's interior. Two tiles
// whose insides do not overlap cannot share an interior point, so no pin
// serves two tiles, and four pins allow at most four tiles. ∎
//
// At L = 3 the pieces are exactly 1 long: [1, 2] holds the marks only at its
// ends, a tile can touch pins with its edges, and five straight tiles fit
// (`five_straight`). Strictness is the whole argument: with closed
// intervals every tile "holds" a pin at L = 3 too, and the conclusion would
// be false (`closed_marks_would_prove_too_much`). The pins say nothing about
// turned tiles: in the box [`turned_side`] the turned tile holds none.
//
// The functions below make each step checkable: [`marks_inside`] is the
// one-dimensional step for one shadow, [`every_shadow_holds_a_mark`] checks
// it for every shadow in one box, exactly and completely (the marks inside
// can change only where a mark meets an end of the shadow, so the critical
// positions and the midpoints between them stand for all the others), and
// [`pins_inside`] is the step for any tile, straight or turned.

/// The marks on one side of a box `side` wide: a third and two thirds of
/// the way along.
pub fn marks<T: Num>(side: T) -> [T; 2] {
    [side * T::ratio(1, 3), side * T::ratio(2, 3)]
}

/// The longest of the three pieces the marks cut a side into (they are
/// equal: side/3).
pub fn longest_piece<T: Num>(side: T) -> T {
    let [lo, hi] = marks(side);
    larger(larger(lo, hi - lo), side - hi)
}

/// The four pins, where the floor's marks meet the wall's, in the corner
/// tiles' order: lower left, lower right, upper left, upper right.
pub fn pins<T: Num>(side: T) -> [Point<T>; 4] {
    let [lo, hi] = marks(side);
    [(lo, lo), (hi, lo), (lo, hi), (hi, hi)]
}

/// The marks strictly inside the shadow [a, a + 1] (inside, not at an end).
pub fn marks_inside<T: Num>(a: T, side: T) -> Vec<usize> {
    let end = a + T::ratio(1, 1);
    marks(side)
        .iter()
        .enumerate()
        .filter(|&(_, &m)| a < m && m < end)
        .map(|(i, _)| i)
        .collect()
}

/// The marks a closed shadow [a, a + 1] touches, ends included: the wrong
/// version of the step, kept to show what strictness buys.
pub fn marks_touched<T: Num>(a: T, side: T) -> Vec<usize> {
    let end = a + T::ratio(1, 1);
    marks(side)
        .iter()
        .enumerate()
        .filter(|&(_, &m)| a <= m && m <= end)
        .map(|(i, _)| i)
        .collect()
}

/// Whether point `p` is strictly inside tile `t` (in its interior), for any
/// turn: along each of the tile's sides, p is less than half a tile from
/// its centre.
pub fn holds<T: Num>(t: &Tile<T>, p: Point<T>) -> bool {
    let h = T::ratio(1, 2);
    let d = (p.0 - t.centre.0, p.1 - t.centre.1);
    t.axes().iter().all(|ax| {
        let s = d.0 * ax.0 + d.1 * ax.1;
        -h < s && s < h
    })
}

/// The pins strictly inside tile `t` in the box `side` wide.
pub fn pins_inside<T: Num>(t: &Tile<T>, side: T) -> Vec<usize> {
    pins(side)
        .iter()
        .enumerate()
        .filter(|&(_, &p)| holds(t, p))
        .map(|(i, _)| i)
        .collect()
}

/// Where the shadow [a, a + 1] can start in the box `side` wide (a from 0 to
/// side − 1) at the positions where the marks inside it can change: its two
/// extremes and wherever a mark meets one of its ends (a = m or a = m − 1).
/// Sorted, without repeats.
pub fn critical_shadows(side: Surd) -> Vec<Surd> {
    let one = Surd::int(1);
    let last = side - one;
    let mut out = vec![Surd::ZERO, last];
    for m in marks(side) {
        out.push(m);
        out.push(m - one);
    }
    out.retain(|&a| Surd::ZERO <= a && a <= last);
    out.sort();
    out.dedup();
    out
}

/// The one-dimensional step, checked for every shadow in the box `side`
/// wide: whether each [a, a + 1] ⊆ [0, side] has a mark strictly inside.
/// Between two neighbouring critical positions the marks inside do not
/// change, so the critical positions and the midpoints between them stand
/// for every a: an exact, complete check for this box, not a sample. None
/// when no shadow fits (a box narrower than one tile holds no tile).
pub fn every_shadow_holds_a_mark(side: Surd) -> Option<bool> {
    let shadows = all_shadows(side)?;
    Some(shadows.iter().all(|&a| !marks_inside(a, side).is_empty()))
}

/// The same check with closed shadows (ends included): true at 3 as well,
/// which is why the argument needs the strict version.
pub fn every_closed_shadow_touches_a_mark(side: Surd) -> Option<bool> {
    let shadows = all_shadows(side)?;
    Some(shadows.iter().all(|&a| !marks_touched(a, side).is_empty()))
}

/// The critical positions and the midpoints between neighbours; None when
/// the box is narrower than one tile.
fn all_shadows(side: Surd) -> Option<Vec<Surd>> {
    if side < Surd::int(1) {
        return None;
    }
    let critical = critical_shadows(side);
    let mut all = critical.clone();
    for w in critical.windows(2) {
        all.push((w[0] + w[1]) * Surd::ratio(1, 2));
    }
    Some(all)
}

/// The two-dimensional step for the box `side` wide: every straight tile
/// in it, at every critical position on both axes and between them, holds a
/// pin strictly inside (the product of the one-dimensional step). None for a
/// box narrower than one tile.
pub fn every_straight_tile_holds_a_pin(side: Surd) -> Option<bool> {
    let shadows = all_shadows(side)?;
    let h = Surd::ratio(1, 2);
    Some(shadows.iter().all(|&a| {
        shadows
            .iter()
            .all(|&b| !pins_inside(&Tile::straight(a + h, b + h), side).is_empty())
    }))
}

/// The box width at which the pieces stop being shorter than a tile: the
/// side is cut into equal pieces, one more than the marks, so a piece is a
/// whole tile long when the side is that many tiles.
pub fn pin_threshold() -> Surd {
    let pieces = marks(Surd::int(1)).len() + 1;
    Surd::int(pieces as i64)
}

/// Each tile's pins strictly inside it, for a packing: the pigeonhole. In a
/// packing that fits, these lists are disjoint (a shared pin would be a
/// shared interior point).
pub fn pin_owners(p: &Packing<Surd>) -> Vec<Vec<usize>> {
    p.tiles.iter().map(|t| pins_inside(t, p.side)).collect()
}

/// A straight tile's lower left corner wandering through the box `side`
/// wide (t from 0 to 1, a closed loop from the lower left corner, where
/// 02_3's sliding tile stops): 02_4's probe. Across the floor once and up
/// and down twice; every position keeps the tile inside the box.
pub fn wander(t: f64, side: f64) -> Point<f64> {
    let room = side - 1.0;
    let turn = std::f64::consts::TAU * t;
    (room * 0.5 * (1.0 - turn.cos()), room * 0.5 * (1.0 - (2.0 * turn).cos()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-12;

    fn r2() -> Surd {
        Surd::sqrt2()
    }

    #[test]
    fn exact_arithmetic_and_its_sign() {
        assert_eq!(r2() * r2(), Surd::int(2));
        // 3 - 2√2 > 0 (9 > 8); 7/5 - √2 < 0 (49/25 < 2); 0
        assert_eq!((Surd::int(3) - Surd::int(2) * r2()).signum(), 1);
        assert_eq!((Surd::ratio(7, 5) - r2()).signum(), -1);
        assert_eq!((Surd::ratio(-7, 5) + r2()).signum(), 1);
        assert_eq!((Surd::int(-3) + Surd::int(2) * r2()).signum(), -1);
        assert_eq!(Surd::ZERO.signum(), 0);
        assert_eq!((r2() - r2()).signum(), 0);
        assert!(Surd::ratio(141, 100) < r2() && r2() < Surd::ratio(142, 100));
        assert_eq!(Ratio::new(6, -4), Ratio::new(-3, 2));
        assert_eq!(Ratio::new(0, -5), Ratio::ZERO);
        assert_eq!(turned_side().to_string(), "2 + 1/2*sqrt(2)");
        assert_eq!(turned_gap().to_string(), "1/2*sqrt(2)");
        assert_eq!((Surd::int(1) - turned_gap()).to_string(), "1 - 1/2*sqrt(2)");
        assert_eq!((-r2()).to_string(), "-sqrt(2)");
        assert_eq!(Surd::ratio(-3, 4).to_string(), "-3/4");
        assert_eq!(turned_side().decimals(2), "2.71");
        assert_eq!(turned_side().decimals(5), "2.70711");
    }

    /// The gap's diagonal is one tile, and the box is a tile, the gap and a
    /// tile; and at that box the opposite inner corners are exactly one apart.
    #[test]
    fn the_box_is_derived_from_the_diagonal() {
        assert_eq!(turned_gap() * r2(), Surd::int(1));
        assert_eq!(turned_side(), Surd::int(1) + turned_gap() + Surd::int(1));
        let p = five_turned();
        let (a, b) = (p.inner_corner(0), p.inner_corner(3));
        let d2 = (b.0 - a.0) * (b.0 - a.0) + (b.1 - a.1) * (b.1 - a.1);
        assert_eq!(d2, Surd::int(1));
        assert_eq!(b.0 - a.0, turned_gap());
    }

    #[test]
    fn the_five_tiles_fit_and_touch_exactly() {
        let v = five_turned().check_exact();
        assert!(v.fits(), "{v:?}");
        assert_eq!(v.summary(), "fits");
        assert_eq!(v.touching, vec![(0, 4), (1, 4), (2, 4), (3, 4)]);
        assert_eq!(v.touching_list(), "0-4,1-4,2-4,3-4");
        // each corner tile against its two walls; the middle tile against none
        assert_eq!(v.walls.len(), 8);
        assert!(v.walls.iter().all(|&(i, _)| i != MIDDLE));
        // each inner corner is the middle of a side of the turned tile
        let p = five_turned();
        let mids = p.tiles[MIDDLE].midpoints();
        for k in 0..4 {
            assert!(mids.contains(&p.inner_corner(k)), "inner corner {k}");
        }
        // the middle tile clears the walls by 1 - √2/4
        assert_eq!(
            wall_clearance(&p.tiles[MIDDLE], p.side),
            Surd::int(1) - r2() * Surd::ratio(1, 4)
        );
    }

    #[test]
    fn the_straight_grid() {
        assert_eq!(grid_side(TILES), 3);
        assert_eq!((grid_side(4), grid_side(9), grid_side(10)), (2, 3, 4));
        assert_eq!(full_grid(TILES), 2);
        assert_eq!((full_grid(4), full_grid(8), full_grid(9)), (2, 2, 3));
        let v = five_straight().check_exact();
        assert!(v.fits(), "{v:?}");
        // straight, the middle tile touches all four corner tiles at a corner
        assert_eq!(v.touching, vec![(0, 4), (1, 4), (2, 4), (3, 4)]);
    }

    /// The separating-axis clearance on cases checked by hand.
    #[test]
    fn clearance_by_hand() {
        let h = Surd::ratio(1, 2);
        let a = Tile::straight(h, h);
        let at = |x: Surd| Tile::straight(x, h);
        assert_eq!(clearance(&a, &at(Surd::ratio(3, 2))), Surd::ZERO);
        assert_eq!(clearance(&a, &at(Surd::int(2))), h);
        assert_eq!(clearance(&a, &at(Surd::ratio(5, 4))), Surd::ratio(-1, 4));
        assert_eq!(clearance(&at(Surd::ratio(5, 4)), &a), Surd::ratio(-1, 4));
        // a diamond whose side passes through (1, 1) touches the tile there
        let (c, s) = eighth_turn();
        let centre = Surd::int(1) + r2() * Surd::ratio(1, 4);
        let d = Tile {
            centre: (centre, centre),
            cos: c,
            sin: s,
        };
        assert_eq!(clearance(&a, &d), Surd::ZERO);
        assert_eq!(wall_clearance(&a, Surd::int(1)), Surd::ZERO);
        assert_eq!(wall_clearance(&at(Surd::ratio(3, 4)), Surd::int(1)), Surd::ratio(-1, 4));
    }

    #[test]
    fn every_planted_failure_is_caught() {
        let want = [
            ("squeeze_further", "overlap:0-4,1-4,2-4,3-4"),
            ("middle_turned_more", "overlap:0-4,1-4,2-4,3-4"),
            ("middle_shifted", "overlap:1-4,3-4"),
            ("box_smaller", "escape:1,2,3"),
            ("straight_squeezed", "overlap:0-4,1-4,2-4,3-4"),
        ];
        let got: Vec<(&str, String)> = planted()
            .iter()
            .map(|p| (p.key, p.packing.check_exact().summary()))
            .collect();
        let want: Vec<(&str, String)> = want.iter().map(|&(k, s)| (k, s.to_owned())).collect();
        assert_eq!(got, want);
        // the extra turn is an exact turn
        let (c, s) = compose(eighth_turn(), small_turn());
        assert_eq!(c * c + s * s, Surd::int(1));
    }

    /// The edges of "it turns without hitting anything": straight (angle 0)
    /// it touches all four neighbours; at every sampled angle up to an eighth
    /// of a turn nothing overlaps and nothing escapes; past straight, it
    /// touches nothing.
    #[test]
    fn the_turn_never_collides() {
        assert_eq!(turn_reach2(), Surd::ratio(1, 2));
        assert_eq!(turn_inner2(), Surd::ratio(1, 2));
        let steps = 2000;
        for k in 0..=steps {
            let angle = std::f64::consts::FRAC_PI_4 * f64::from(k) / f64::from(steps);
            let p = turning(angle);
            assert!(p.worst() >= -EPS, "angle {angle}: {}", p.worst());
            let v = p.check(EPS);
            assert!(v.fits(), "angle {angle}: {v:?}");
            if k == 0 {
                assert_eq!(v.touching.len(), 4);
            } else {
                assert!(v.touching.is_empty(), "angle {angle}: {v:?}");
            }
        }
    }

    /// The edges of "no two tiles ever overlap" in the squeeze: from the box
    /// of three down to the stop nothing overlaps or escapes, the middle tile
    /// touches its four neighbours only at the stop, and a hair past the stop
    /// all four overlap it.
    #[test]
    fn the_squeeze_never_collides_and_stops_at_the_touch() {
        let (from, to) = (turn_box(), turned_side().to_f64());
        let steps = 2000;
        for k in 0..=steps {
            let side = from + (to - from) * f64::from(k) / f64::from(steps);
            let p = squeezed(side);
            assert!(p.worst() >= -EPS, "side {side}: {}", p.worst());
            let v = p.check(EPS);
            assert!(v.fits(), "side {side}: {v:?}");
            assert_eq!(v.touching.len(), if k == steps { 4 } else { 0 }, "side {side}");
        }
        let past = squeezed(to - 1e-6).check(EPS);
        assert_eq!(past.overlapping, vec![(0, 4), (1, 4), (2, 4), (3, 4)]);
    }

    /// Squeezing without the turn fails at once: the straight middle tile
    /// already touches all four corner tiles in the box of three.
    #[test]
    fn a_straight_middle_cannot_be_squeezed() {
        let p = corners_and_middle(turn_box() - 1e-6, 1.0, 0.0);
        assert_eq!(p.check(EPS).overlapping.len(), 4);
    }

    /// Below three each piece is shorter than a tile; at three a piece is a
    /// whole tile; the pins sit where the marks cross.
    #[test]
    fn the_marks_cut_pieces_shorter_than_a_tile_below_three() {
        let side = turned_side();
        assert_eq!(longest_piece(side), side * Surd::ratio(1, 3));
        for s in [side, Surd::ratio(299, 100), Surd::int(2), Surd::int(1)] {
            assert!(longest_piece(s) < Surd::int(1), "{s}");
        }
        assert_eq!(pin_threshold(), Surd::int(3));
        assert_eq!(longest_piece(pin_threshold()), Surd::int(1));
        let at3 = pins(pin_threshold());
        assert_eq!(at3[0], (Surd::int(1), Surd::int(1)));
        assert_eq!(at3[3], (Surd::int(2), Surd::int(2)));
    }

    /// The one-dimensional and two-dimensional steps, checked exactly for
    /// every shadow and every straight tile: true below three (the turned
    /// packing's box, 2.99, 2, 7/5 and 1), false at 3 and 7/2, and no room at
    /// all below one tile.
    #[test]
    fn every_shadow_holds_a_mark_below_three_and_not_from_three() {
        for s in [
            turned_side(),
            Surd::ratio(299, 100),
            Surd::int(2),
            Surd::ratio(7, 5),
            Surd::int(1),
        ] {
            assert_eq!(every_shadow_holds_a_mark(s), Some(true), "{s}");
            assert_eq!(every_straight_tile_holds_a_pin(s), Some(true), "{s}");
        }
        for s in [Surd::int(3), Surd::ratio(7, 2)] {
            assert_eq!(every_shadow_holds_a_mark(s), Some(false), "{s}");
            assert_eq!(every_straight_tile_holds_a_pin(s), Some(false), "{s}");
        }
        assert_eq!(every_shadow_holds_a_mark(Surd::ratio(1, 2)), None);
        assert_eq!(every_straight_tile_holds_a_pin(Surd::ratio(99, 100)), None);
        // at three the shadow [1, 2] has the marks only at its ends
        assert!(marks_inside(Surd::int(1), Surd::int(3)).is_empty());
        assert_eq!(marks_touched(Surd::int(1), Surd::int(3)), vec![0, 1]);
        // the critical positions in the turned packing's box
        assert_eq!(critical_shadows(turned_side()).len(), 4);
    }

    /// With ends included the step "holds" at three as well, where five
    /// straight tiles fit: a proof with closed shadows would prove something
    /// false.
    #[test]
    fn closed_marks_would_prove_too_much() {
        assert_eq!(every_closed_shadow_touches_a_mark(Surd::int(3)), Some(true));
        let five = five_straight();
        assert!(five.check_exact().fits());
        assert!(pin_owners(&five).iter().all(Vec::is_empty));
    }

    /// The pigeonhole on the chapter's pictures: in the turned packing's box
    /// each corner tile holds its own pin, a straight middle tile holds all
    /// four (and overlaps all four corner tiles), the turned middle tile
    /// holds none; in a packing that fits no pin serves two tiles.
    #[test]
    fn the_pins_and_the_tiles() {
        let side = turned_side();
        let straight = corners_and_middle(side, Surd::int(1), Surd::ZERO);
        let owners = pin_owners(&straight);
        assert_eq!(owners[..4], [vec![0], vec![1], vec![2], vec![3]]);
        assert_eq!(owners[MIDDLE], vec![0, 1, 2, 3]);
        assert_eq!(straight.check_exact().overlapping, vec![(0, 4), (1, 4), (2, 4), (3, 4)]);
        let turned = pin_owners(&five_turned());
        assert_eq!(turned[..4], [vec![0], vec![1], vec![2], vec![3]]);
        assert!(turned[MIDDLE].is_empty());
        for p in [five_turned(), five_straight()] {
            assert!(p.check_exact().fits());
            let mut seen: Vec<usize> = pin_owners(&p).concat();
            let n = seen.len();
            seen.sort_unstable();
            seen.dedup();
            assert_eq!(seen.len(), n, "a pin serves two tiles");
        }
    }

    /// The probe of 02_4 stays in the box and holds a pin at every sampled
    /// frame (a check of the frames drawn; the proof covers the rest).
    #[test]
    fn the_wandering_tile_always_holds_a_pin() {
        let side = turned_side().to_f64();
        assert_eq!(wander(0.0, side), (0.0, 0.0));
        for k in 0..=2000 {
            let t = f64::from(k) / 2000.0;
            let (a, b) = wander(t, side);
            let tile = Tile::straight(a + 0.5, b + 0.5);
            assert!(wall_clearance(&tile, side) >= -EPS, "{t}");
            assert!(!pins_inside(&tile, side).is_empty(), "{t}");
        }
    }
}
