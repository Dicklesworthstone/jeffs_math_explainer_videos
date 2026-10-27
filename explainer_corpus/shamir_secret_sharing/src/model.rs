//! The mathematics of "Three Keys Open the Secret. Two Reveal Nothing.":
//! a dial of `marks` marks (whole numbers wrapped onto 0 to marks − 1), the
//! dealer's recipe (the secret plus spins times powers of a trustee's
//! position, Shamir's q(x) with q(0) = D), rebuilding the secret from shares
//! with Lagrange's weights, and the brute-force counts the video's privacy
//! claims rest on. Everything is exact integer arithmetic. [`crate::facts`]
//! turns it into the video's numbers; `oracle.py` recomputes them another
//! way (the definitions written out, inverses and rebuilding by search).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// The vault's dial: eleven marks, 0 to 10.
pub const MARKS: u32 = 11;
/// The ordinary ten-mark dial, the negative control (chapters 12, 13, 20).
pub const TEN: u32 = 10;
/// The five trustees' positions (Shamir's x = 1 to n).
pub const TRUSTEES: [u32; 5] = [1, 2, 3, 4, 5];
/// How many trustees it takes: the threshold.
pub const THRESHOLD: usize = 3;
/// The secret, the mark that opens the door.
pub const SECRET: u32 = 5;
/// The worked example's two spins (times the position, times its square),
/// chosen for the picture: five different shares, none equal to the secret.
/// Every privacy claim is checked for every spin pair, not only these.
pub const SPINS: [u32; 2] = [7, 7];
/// The trio who open the door (chapters 04, 14, 15, 22, 23), the pair who
/// meet (chapters 04, 05, 10), the third trustee who joins them (chapter 11)
/// and the second trio (chapters 16 to 18).
pub const OPENING_TRIO: [u32; 3] = [1, 2, 3];
pub const PAIR: [u32; 2] = [4, 5];
pub const JOINER: u32 = 1;
pub const SECOND_TRIO: [u32; 3] = [2, 4, 5];
/// Chapters 10 and 11: the secrets asked about after the dealer's, each
/// with its own curve through the pair's two shares.
pub const ASKED_SECRETS: [u32; 2] = [0, 1];
/// Chapter 17: another dealing of the same secret, whose thirds do not
/// cancel on the second trio's shares.
pub const OTHER_SPINS: [u32; 2] = [0, 1];
/// Chapter 02: the secret cut into two ordinary pieces, A's and B's.
pub const ORDINARY_PIECES: [u32; 2] = [2, 3];
/// Chapter 03: A's spin on the dial.
pub const A_SHARE: u32 = 9;
/// Chapters 07 and 08: the any-two-of-five line's one spin, the trustee
/// alone, and the two neighbours.
pub const LINE_SLOPE: u32 = 4;
pub const LINE_HOLDER: u32 = 2;
pub const ADJACENT: [u32; 2] = [2, 3];
/// Chapter 07's trustee on paper, and chapter 13's on ten marks; and the
/// ten-mark share that gives away odd or even.
pub const TEN_HOLDER: u32 = 5;
pub const TEN_PARITY_HOLDER: u32 = 2;
/// Chapter 22: trustee 3 hands in 2 (the true share is 1); trustee 4's
/// share disagrees with it, and all five shares single out the liar.
pub const LIAR: (u32, u32) = (3, 2);
pub const FOURTH: u32 = 4;
/// Chapter 23's question: four of seven, three shares seen (trustees 1, 2
/// and 3 holding 4, 0 and 9; any values give the same count).
pub const FOUR_OF_SEVEN: (usize, usize) = (4, 7);
pub const FOUR_OF_SEVEN_SEEN: [(u32, u32); 3] = [(1, 4), (2, 0), (3, 9)];

/// Values joined with commas, as the facts print lists.
pub fn csv<T: fmt::Display>(values: &[T]) -> String {
    values.iter().map(ToString::to_string).collect::<Vec<_>>().join(",")
}

// --------------------------------------------------------------- the dial

/// A dial of `marks` marks, 0 to marks − 1: adding is turning forward,
/// taking away is turning back, and every result wraps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dial {
    marks: u32,
}

impl Dial {
    /// A dial of `marks` marks.
    ///
    /// # Panics
    /// With fewer than two marks.
    pub const fn new(marks: u32) -> Self {
        assert!(marks >= 2, "a dial has at least two marks");
        Self { marks }
    }

    /// The vault's eleven-mark dial.
    pub const fn vault() -> Self {
        Self::new(MARKS)
    }

    pub const fn marks(self) -> u32 {
        self.marks
    }

    /// Every mark, 0 to marks − 1.
    pub fn all(self) -> std::ops::Range<u32> {
        0..self.marks
    }

    /// Where an ordinary whole number lands.
    pub fn wrap(self, n: i64) -> u32 {
        // rem_euclid of a positive modulus is in 0..marks, so it fits
        n.rem_euclid(i64::from(self.marks)) as u32
    }

    pub fn add(self, a: u32, b: u32) -> u32 {
        self.wrap(i64::from(a) + i64::from(b))
    }

    pub fn sub(self, a: u32, b: u32) -> u32 {
        self.wrap(i64::from(a) - i64::from(b))
    }

    pub fn mul(self, a: u32, b: u32) -> u32 {
        self.wrap(i64::from(a) * i64::from(b))
    }

    /// The partner of `d`: the mark that, times `d`, lands on 1 (the
    /// extended Euclidean algorithm); `None` when there is none.
    pub fn inverse(self, d: u32) -> Option<u32> {
        let (mut r0, mut r1) = (i64::from(self.marks), i64::from(d % self.marks));
        let (mut t0, mut t1) = (0i64, 1i64);
        while r1 != 0 {
            let q = r0 / r1;
            (r0, r1) = (r1, r0 - q * r1);
            (t0, t1) = (t1, t0 - q * t1);
        }
        (r0 == 1).then(|| self.wrap(t0))
    }

    /// `k` times every mark, in order.
    pub fn times_row(self, k: u32) -> Vec<u32> {
        self.all().map(|x| self.mul(k, x)).collect()
    }

    /// Whether `k`'s row reaches every mark, each once.
    pub fn shuffles(self, k: u32) -> bool {
        let mut row = self.times_row(k);
        row.sort_unstable();
        row.dedup();
        row.len() == self.marks as usize
    }

    /// Whether the number of marks is prime.
    pub fn is_prime(self) -> bool {
        (2..self.marks)
            .take_while(|d| d * d <= self.marks)
            .all(|d| !self.marks.is_multiple_of(d))
    }
}

// ------------------------------------------------------------- fractions

/// An exact fraction in lowest terms, its denominator positive: the
/// ordinary-number weights (chapters 14 to 18) and the paper curve's spins
/// (chapter 11).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ratio {
    num: i64,
    den: i64,
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

impl Ratio {
    /// `num / den` in lowest terms.
    ///
    /// # Panics
    /// When `den` is zero.
    pub fn new(num: i64, den: i64) -> Self {
        assert!(den != 0, "a fraction with denominator zero");
        let g = gcd(num.unsigned_abs(), den.unsigned_abs()) as i64;
        let sign = if den < 0 { -1 } else { 1 };
        Self {
            num: sign * num / g,
            den: sign * den / g,
        }
    }

    pub fn int(n: i64) -> Self {
        Self { num: n, den: 1 }
    }

    pub fn plus(self, o: Self) -> Self {
        Self::new(self.num * o.den + o.num * self.den, self.den * o.den)
    }

    pub fn times(self, k: i64) -> Self {
        Self::new(self.num * k, self.den)
    }

    pub fn minus(self, o: Self) -> Self {
        self.plus(o.times(-1))
    }

    /// The whole number this is, if it is one.
    pub fn whole(self) -> Option<i64> {
        (self.den == 1).then_some(self.num)
    }

    /// Its whole part, rounded down, and what is left over, 0 or more and
    /// under 1: 37/4 is 9 and 1/4 (chapter 11's paper spins).
    pub fn mixed(self) -> (i64, Self) {
        (
            self.num.div_euclid(self.den),
            Self::new(self.num.rem_euclid(self.den), self.den),
        )
    }

    /// The nearest float, for drawing.
    pub fn to_f64(self) -> f64 {
        self.num as f64 / self.den as f64
    }

    /// The fraction on the dial: its top times the partner of its bottom
    /// (dividing by 3 on eleven marks is multiplying by 4); `None` when the
    /// bottom has no partner.
    pub fn on(self, dial: Dial) -> Option<u32> {
        dial.inverse(dial.wrap(self.den))
            .map(|p| dial.mul(dial.wrap(self.num), p))
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

// ------------------------------------------------------------ the recipe

/// The dealer's recipe: the secret, then the spins that multiply each power
/// of the position (one spin draws a line, two a curve).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recipe {
    pub secret: u32,
    pub spins: Vec<u32>,
}

impl Recipe {
    pub fn new(secret: u32, spins: &[u32]) -> Self {
        Self {
            secret,
            spins: spins.to_vec(),
        }
    }

    /// The worked example: the secret 5 with spins 7 and 7.
    pub fn example() -> Self {
        Self::new(SECRET, &SPINS)
    }

    /// Highest power first, for Horner's rule.
    fn coefficients_high_first(&self) -> impl Iterator<Item = u32> + '_ {
        self.spins.iter().rev().copied().chain(std::iter::once(self.secret))
    }

    /// The height at `x` on ordinary paper, nothing wrapped.
    pub fn height(&self, x: u32) -> i64 {
        self.coefficients_high_first()
            .fold(0i64, |acc, c| acc * i64::from(x) + i64::from(c))
    }

    /// The share at `x`: the height on the dial, by Horner's rule with
    /// every step wrapped.
    pub fn share(&self, dial: Dial, x: u32) -> u32 {
        self.coefficients_high_first()
            .fold(0, |acc, c| dial.add(dial.mul(acc, x), c))
    }

    /// The shares at each of `xs`.
    pub fn deal(&self, dial: Dial, xs: &[u32]) -> Vec<u32> {
        xs.iter().map(|&x| self.share(dial, x)).collect()
    }

    /// Whether the recipe deals `points` (position, share).
    pub fn deals(&self, dial: Dial, points: &[(u32, u32)]) -> bool {
        points.iter().all(|&(x, y)| self.share(dial, x) == y)
    }

    /// The secret, then the spins.
    pub fn coefficients(&self) -> Vec<u32> {
        std::iter::once(self.secret).chain(self.spins.iter().copied()).collect()
    }

    /// The two recipes added mark by mark on the dial: its curve's share at
    /// every position is the sum of theirs.
    ///
    /// # Panics
    /// When the two have different numbers of spins.
    pub fn plus(&self, dial: Dial, other: &Self) -> Self {
        assert_eq!(self.spins.len(), other.spins.len(), "recipes of different lengths");
        let spins: Vec<u32> = self
            .spins
            .iter()
            .zip(&other.spins)
            .map(|(&a, &b)| dial.add(a, b))
            .collect();
        Self::new(dial.add(self.secret, other.secret), &spins)
    }
}

/// Every tuple of `len` marks, the first slowest (as the spin grid is read).
pub fn tuples(dial: Dial, len: usize) -> impl Iterator<Item = Vec<u32>> {
    let m = u64::from(dial.marks());
    let total = m.pow(len as u32);
    (0..total).map(move |mut i| {
        let mut t = vec![0u32; len];
        for slot in t.iter_mut().rev() {
            *slot = (i % m) as u32;
            i /= m;
        }
        t
    })
}

/// Every tuple of `count` spins that, with `secret`, deals `points`: the
/// brute force behind "one curve per secret".
pub fn fitting_spins(dial: Dial, points: &[(u32, u32)], secret: u32, count: usize) -> Vec<Vec<u32>> {
    tuples(dial, count)
        .filter(|spins| Recipe::new(secret, spins).deals(dial, points))
        .collect()
}

/// How many marks a recipe of `recipe` lands on zero at.
pub fn zeros(dial: Dial, recipe: &Recipe) -> usize {
    dial.all().filter(|&x| recipe.share(dial, x) == 0).count()
}

/// The most marks any recipe of `spins` spins, not all zero, lands on zero
/// at (by trying every one).
pub fn most_zeros(dial: Dial, spins: usize) -> usize {
    tuples(dial, spins + 1)
        .skip(1)
        .map(|t| zeros(dial, &Recipe::new(t[0], &t[1..])))
        .max()
        .unwrap_or(0)
}

/// A recipe of two spins with a nonzero secret that is zero at every one of
/// `xs` (chapter 20): added to the dealer's, it deals those trustees the
/// same shares and moves the secret. The lowest-degree one (smallest last
/// spin, then first spin, then secret); `None` when there is none, as on a
/// prime dial with three positions.
pub fn ghost(dial: Dial, xs: &[u32]) -> Option<Recipe> {
    tuples(dial, 3)
        .filter(|t| t[0] != 0)
        .map(|t| Recipe::new(t[0], &t[1..]))
        .filter(|r| xs.iter().all(|&x| r.share(dial, x) == 0))
        .min_by_key(|r| (r.spins[1], r.spins[0], r.secret))
}

// ---------------------------------------------------- rebuilding (Lagrange)

/// Trustee `i`'s helper before it is scaled, at position `at`: the product
/// of (position − each other trustee's position). It is zero at every other
/// trustee.
pub fn helper_at(xs: &[u32], i: usize, at: i64) -> i64 {
    xs.iter()
        .enumerate()
        .filter(|&(j, _)| j != i)
        .map(|(_, &xj)| at - i64::from(xj))
        .product()
}

/// The unscaled helper at the trustee's own position (what it is halved by,
/// in chapter 14's case).
pub fn helper_at_own(xs: &[u32], i: usize) -> i64 {
    helper_at(xs, i, i64::from(xs[i]))
}

/// Each trustee's weight at `at` in ordinary numbers: the helper there over
/// the helper at the trustee's own position.
///
/// # Panics
/// When two positions coincide (a helper is then zero at its own position).
pub fn ordinary_weights_at(xs: &[u32], at: i64) -> Vec<Ratio> {
    (0..xs.len())
        .map(|i| Ratio::new(helper_at(xs, i, at), helper_at_own(xs, i)))
        .collect()
}

/// The weights at zero, the ones the video draws.
///
/// # Panics
/// When two positions coincide.
pub fn ordinary_weights(xs: &[u32]) -> Vec<Ratio> {
    ordinary_weights_at(xs, 0)
}

/// The same weights on the dial; `None` when two positions coincide or a
/// helper's value has no partner (on ten marks, for instance).
pub fn weights_at(dial: Dial, xs: &[u32], at: i64) -> Option<Vec<u32>> {
    (0..xs.len())
        .map(|i| match helper_at_own(xs, i) {
            0 => None,
            own => Ratio::new(helper_at(xs, i, at), own).on(dial),
        })
        .collect()
}

/// The weighted sum at zero with the dial's weights, before it is wrapped
/// (chapter 18's 93).
pub fn dial_sum(dial: Dial, points: &[(u32, u32)]) -> Option<u64> {
    let xs: Vec<u32> = points.iter().map(|p| p.0).collect();
    let w = weights_at(dial, &xs, 0)?;
    Some(
        points
            .iter()
            .zip(&w)
            .map(|(&(_, y), &wi)| u64::from(wi) * u64::from(y))
            .sum(),
    )
}

/// The height at `at` of the one curve through `points`, on the dial.
pub fn interpolate(dial: Dial, points: &[(u32, u32)], at: i64) -> Option<u32> {
    let xs: Vec<u32> = points.iter().map(|p| p.0).collect();
    let w = weights_at(dial, &xs, at)?;
    Some(
        points
            .iter()
            .zip(&w)
            .fold(0, |acc, (&(_, y), &wi)| dial.add(acc, dial.mul(wi, y))),
    )
}

/// The secret rebuilt from shares: the curve's height at zero.
pub fn rebuild(dial: Dial, points: &[(u32, u32)]) -> Option<u32> {
    interpolate(dial, points, 0)
}

/// The ordinary weights at zero times `points`' values, term by term: the
/// paper heights (chapter 15's 57, −141 and 89) or the shares.
///
/// # Panics
/// When two positions coincide.
pub fn paper_terms(points: &[(u32, i64)]) -> Vec<Ratio> {
    let xs: Vec<u32> = points.iter().map(|p| p.0).collect();
    ordinary_weights(&xs)
        .into_iter()
        .zip(points)
        .map(|(w, &(_, y))| w.times(y))
        .collect()
}

/// The weighted sum at zero in ordinary numbers: on the paper heights it is
/// the secret itself (chapter 15's 3·19 − 3·47 + 89 = 5).
///
/// # Panics
/// When two positions coincide.
pub fn paper_sum(points: &[(u32, i64)]) -> Ratio {
    paper_terms(points).into_iter().fold(Ratio::int(0), Ratio::plus)
}

/// The same sum on the shares, before it is wrapped (chapter 15's 16,
/// chapter 17's 4/3, chapter 22's 17).
///
/// # Panics
/// When two positions coincide.
pub fn ordinary_sum(points: &[(u32, u32)]) -> Ratio {
    let tall: Vec<(u32, i64)> = points.iter().map(|&(x, y)| (x, i64::from(y))).collect();
    paper_sum(&tall)
}

/// The ordinary weights at zero, each times the smallest whole number that
/// clears every denominator, and that number: chapter 17's 10, −15 and 8,
/// which give three times the secret.
///
/// # Panics
/// When two positions coincide.
pub fn cleared_weights(xs: &[u32]) -> (i64, Vec<i64>) {
    let w = ordinary_weights(xs);
    let clear = w.iter().fold(1i64, |l, r| {
        l / gcd(l.unsigned_abs(), r.den.unsigned_abs()) as i64 * r.den
    });
    // clear is a multiple of every bottom, so each product is whole
    let whole = w.iter().map(|r| r.num * (clear / r.den)).collect();
    (clear, whole)
}

/// The mark that, times `k`, lands on `target`, by `k`'s partner; `None`
/// when `k` has none.
pub fn undo(dial: Dial, k: u32, target: u32) -> Option<u32> {
    dial.inverse(k).map(|p| dial.mul(target, p))
}

/// The curves of two spins through at least `agree` of `points` (chapter
/// 22), each one rebuilt from three of the points and checked against all of
/// them: its secret and its share at each point's position. Two such curves
/// that agree at three positions are the same, so each is listed once.
pub fn curves_through(dial: Dial, points: &[(u32, u32)], agree: usize) -> Vec<(u32, Vec<u32>)> {
    let mut found: Vec<(u32, Vec<u32>)> = Vec::new();
    for idx in combinations(&(0..points.len() as u32).collect::<Vec<_>>(), THRESHOLD) {
        let trio: Vec<(u32, u32)> = idx.iter().map(|&i| points[i as usize]).collect();
        let Some(secret) = rebuild(dial, &trio) else { continue };
        let Some(along) = points
            .iter()
            .map(|&(x, _)| interpolate(dial, &trio, i64::from(x)))
            .collect::<Option<Vec<u32>>>()
        else {
            continue;
        };
        let hits = along.iter().zip(points).filter(|&(&a, &(_, y))| a == y).count();
        let curve = (secret, along);
        if hits >= agree && !found.contains(&curve) {
            found.push(curve);
        }
    }
    found
}

// ------------------------------------------------------ possible secrets

/// What a person or group holds, for the strip of possible secrets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum View {
    /// Someone handed nothing.
    Outsider,
    /// Someone handed the secret itself (a single keeper, or a copy).
    Copy(u32),
    /// Someone handed B's ordinary piece `b` of a secret cut into two
    /// whole numbers, each 0 or more, that add up to it.
    OrdinaryPiece(u32),
    /// Trustees holding these shares of a recipe with `spins` fair spins.
    Shares { points: Vec<(u32, u32)>, spins: usize },
    /// Trustees holding these heights on ordinary paper, nothing wrapped,
    /// of a recipe with `spins` whole spins 0 to marks − 1 (chapter 07).
    Paper { points: Vec<(u32, i64)>, spins: usize },
}

impl View {
    /// The secrets, 0 to marks − 1, that still fit what this view holds.
    pub fn possible(&self, dial: Dial) -> Vec<u32> {
        match self {
            Self::Outsider => dial.all().collect(),
            Self::Copy(s) => vec![*s],
            Self::OrdinaryPiece(b) => dial.all().filter(|s| s >= b).collect(),
            Self::Shares { points, spins } => dial
                .all()
                .filter(|&s| !fitting_spins(dial, points, s, *spins).is_empty())
                .collect(),
            Self::Paper { points, spins } => dial
                .all()
                .filter(|&s| {
                    tuples(dial, *spins).any(|t| {
                        let r = Recipe::new(s, &t);
                        points.iter().all(|&(x, h)| r.height(x) == h)
                    })
                })
                .collect(),
        }
    }
}

/// For every tuple of shares trustees at `xs` could hold: is every secret
/// dealt by the same number of allowed spin tuples? Equal counts under fair
/// spins are what "no information" means.
pub fn learns_nothing(dial: Dial, xs: &[u32], spins: usize, allowed: impl Fn(&[u32]) -> bool) -> bool {
    let fair: Vec<Vec<u32>> = tuples(dial, spins).filter(|t| allowed(t)).collect();
    tuples(dial, xs.len()).all(|values| {
        let points: Vec<(u32, u32)> = xs.iter().copied().zip(values).collect();
        let counts: Vec<usize> = dial
            .all()
            .map(|s| fair.iter().filter(|t| Recipe::new(s, t).deals(dial, &points)).count())
            .collect();
        counts.windows(2).all(|w| w[0] == w[1])
    })
}

/// Every pair of fair spins allowed.
pub fn any_spins(_: &[u32]) -> bool {
    true
}

/// For every tuple of shares some recipe with `spins` fair spins deals the
/// trustees at `xs`: how many secrets still fit it, and whether those
/// secrets all fit it the same number of ways (chapter 12's table). Every
/// recipe is dealt once and counted under what it hands them.
pub fn what_shares_leave(dial: Dial, xs: &[u32], spins: usize) -> BTreeSet<(usize, bool)> {
    let mut ways: BTreeMap<Vec<u32>, Vec<usize>> = BTreeMap::new();
    for t in tuples(dial, spins + 1) {
        let dealt = Recipe::new(t[0], &t[1..]).deal(dial, xs);
        ways.entry(dealt).or_insert_with(|| vec![0; dial.marks() as usize])[t[0] as usize] += 1;
    }
    ways.values()
        .map(|per_secret| {
            let fits: Vec<usize> = per_secret.iter().copied().filter(|&w| w > 0).collect();
            (fits.len(), fits.windows(2).all(|w| w[0] == w[1]))
        })
        .collect()
}

/// A shortcut for two trustees: whole-number weights u and w, each between
/// −(marks − 1) and marks − 1, with u times the first share plus w times the
/// second landing on the secret for every recipe of two spins. The smallest
/// pair, a positive first weight where two tie (on ten marks −5 and 5 are one
/// mark); `None` when no shortcut exists, as on a prime dial.
pub fn pair_shortcut(dial: Dial, pair: [u32; 2]) -> Option<(i64, i64)> {
    let dealt: Vec<(u32, i64, i64)> = tuples(dial, 3)
        .map(|t| {
            let r = Recipe::new(t[0], &t[1..]);
            (
                r.secret,
                i64::from(r.share(dial, pair[0])),
                i64::from(r.share(dial, pair[1])),
            )
        })
        .collect();
    let reach = i64::from(dial.marks()) - 1;
    (-reach..=reach)
        .flat_map(|u| (-reach..=reach).map(move |w| (u, w)))
        .filter(|&(u, w)| dealt.iter().all(|&(s, y1, y2)| dial.wrap(u * y1 + w * y2) == s))
        // min_by_key keeps the first of equal keys, so u runs from −reach up
        .min_by_key(|&(u, w)| (u.abs() + w.abs(), u < 0))
}

/// Two nonzero marks whose product lands on zero, the smallest first;
/// `None` on a prime dial.
pub fn zero_product(dial: Dial) -> Option<(u32, u32)> {
    (1..dial.marks())
        .flat_map(|a| (a..dial.marks()).map(move |b| (a, b)))
        .find(|&(a, b)| dial.mul(a, b) == 0)
}

// -------------------------------------------------------- ordinary paper

/// The secrets 0 to marks − 1 a line through `(x, height)` on paper could
/// start from, with a whole spin 0 to marks − 1.
pub fn paper_line_secrets(marks: u32, x: u32, height: i64) -> Vec<u32> {
    (0..marks)
        .filter(|&s| (0..marks).any(|a| i64::from(s) + i64::from(a) * i64::from(x) == height))
        .collect()
}

/// The spins of the one paper curve s + a·x + b·x² through two ordinary
/// points (Cramer's rule); `None` if the positions coincide or one is zero.
pub fn paper_curve_spins(points: [(u32, i64); 2], secret: u32) -> Option<(Ratio, Ratio)> {
    let [(x1, h1), (x2, h2)] = points.map(|(x, h)| (i64::from(x), h - i64::from(secret)));
    let det = x1 * x2 * x2 - x1 * x1 * x2;
    if det == 0 {
        return None;
    }
    Some((
        Ratio::new(h1 * x2 * x2 - x1 * x1 * h2, det),
        Ratio::new(x1 * h2 - h1 * x2, det),
    ))
}

/// The secrets 0 to marks − 1 whose paper curve through `points` has whole
/// spins 0 to marks − 1, by trying every recipe.
pub fn paper_whole_spin_secrets(marks: u32, points: [(u32, i64); 2]) -> Vec<u32> {
    (0..marks)
        .filter(|&s| {
            tuples(Dial::new(marks), 2).any(|t| {
                let r = Recipe::new(s, &t);
                points.iter().all(|&(x, h)| r.height(x) == h)
            })
        })
        .collect()
}

// --------------------------------------------- plans that hand out copies

/// Chapter 01's plans hand the secret itself to `holders`. The fewest
/// trustees whose loss leaves no copy: every holder.
pub fn losses_to_shut(holders: &[u32]) -> usize {
    holders.len()
}

/// The fewest trustees who, bribed, give the secret away: any one holder;
/// `None` when nobody holds a copy.
pub fn bribes_to_open(holders: &[u32]) -> Option<usize> {
    (!holders.is_empty()).then_some(1)
}

/// The most trustees of `trustees` who can be away, whichever they are,
/// while `needed` are still present.
pub fn can_be_away(trustees: usize, needed: usize) -> usize {
    trustees.saturating_sub(needed)
}

// ---------------------------------------------------------------- padlocks

/// Every group of `k` of `items`, in order.
pub fn combinations(items: &[u32], k: usize) -> Vec<Vec<u32>> {
    if k == 0 {
        return vec![Vec::new()];
    }
    let mut out = Vec::new();
    for (i, &first) in items.iter().enumerate() {
        for rest in combinations(&items[i + 1..], k - 1) {
            let mut group = Vec::with_capacity(k);
            group.push(first);
            group.extend(rest);
            out.push(group);
        }
    }
    out
}

/// The padlock answer (chapter 06): one lock for each group of needed − 1
/// trustees, its keys with everyone outside that group.
pub struct Padlocks {
    locks: Vec<Vec<u32>>,
}

impl Padlocks {
    pub fn new(trustees: &[u32], needed: usize) -> Self {
        Self {
            locks: combinations(trustees, needed - 1),
        }
    }

    /// Each lock, named by the group without its key.
    pub fn locks(&self) -> &[Vec<u32>] {
        &self.locks
    }

    /// How many keys trustee `t` carries.
    pub fn keys(&self, t: u32) -> usize {
        self.locks.iter().filter(|g| !g.contains(&t)).count()
    }

    /// How many locks `group` has no key to (every member inside the lock's group).
    pub fn stuck_at(&self, group: &[u32]) -> usize {
        self.locks
            .iter()
            .filter(|g| group.iter().all(|t| g.contains(t)))
            .count()
    }

    pub fn opens(&self, group: &[u32]) -> bool {
        self.stuck_at(group) == 0
    }
}

/// The fewest locks of any plan, not only the padlock answer's, for which
/// every group of `needed` of `trustees` opens every lock and every group
/// of needed − 1 is stopped by one, by trying every plan. A lock is the set
/// of trustees given its key; it is usable only if every `needed`-group
/// holds a key to it, and it stops a group holding none. `None` if no plan
/// works, or if there are more than [`PLANS_TRIED`] usable locks to try.
pub fn fewest_locks(trustees: &[u32], needed: usize) -> Option<usize> {
    let n = trustees.len();
    if needed == 0 || needed > n || n >= 32 {
        return None;
    }
    let mask = |g: &[u32]| -> u32 {
        g.iter()
            .filter_map(|t| trustees.iter().position(|x| x == t))
            .fold(0, |m, i| m | (1 << i))
    };
    let must_open: Vec<u32> = combinations(trustees, needed).iter().map(|g| mask(g)).collect();
    let must_stop: Vec<u32> = combinations(trustees, needed - 1).iter().map(|g| mask(g)).collect();
    let usable: Vec<u32> = (0..1u32 << n)
        .filter(|&h| must_open.iter().all(|&g| g & h != 0))
        .collect();
    if usable.len() > PLANS_TRIED {
        return None;
    }
    (0..1u64 << usable.len())
        .filter(|&plan| {
            must_stop.iter().all(|&g| {
                usable
                    .iter()
                    .enumerate()
                    .any(|(i, &h)| (plan >> i) & 1 == 1 && g & h == 0)
            })
        })
        .map(|plan| plan.count_ones() as usize)
        .min()
}

/// The most usable locks [`fewest_locks`] tries every family of.
pub const PLANS_TRIED: usize = 20;

#[cfg(test)]
mod tests {
    use super::*;

    const DIAL: Dial = Dial::vault();
    const TEN_DIAL: Dial = Dial::new(TEN);

    #[test]
    fn the_dial_turns_and_wraps() {
        assert_eq!(DIAL.add(9, 7), 5);
        assert_eq!(DIAL.add(10, 1), 0);
        assert_eq!(DIAL.sub(5, 7), 9);
        assert_eq!(DIAL.wrap(-14), 8);
        assert_eq!(DIAL.wrap(-20), 2);
        assert_eq!(TEN_DIAL.wrap(-20), 0);
        assert_eq!(DIAL.times_row(2), vec![0, 2, 4, 6, 8, 10, 1, 3, 5, 7, 9]);
    }

    /// Every partner, against a search; zero never has one; on ten marks
    /// only 1, 3, 7 and 9 do.
    #[test]
    fn partners_match_a_search() {
        for marks in [7, 10, 11, 12, 13] {
            let dial = Dial::new(marks);
            for d in dial.all() {
                let found: Vec<u32> = dial.all().filter(|&k| dial.mul(d, k) == 1).collect();
                assert_eq!(dial.inverse(d), found.first().copied(), "{d} on {marks}");
                assert!(found.len() <= 1);
            }
            assert_eq!(dial.inverse(0), None);
        }
        assert_eq!(DIAL.inverse(3), Some(4));
        let ten: Vec<u32> = (1..TEN).filter(|&d| TEN_DIAL.inverse(d).is_some()).collect();
        assert_eq!(ten, vec![1, 3, 7, 9]);
    }

    /// A row shuffles exactly when its mark has a partner; on a prime dial
    /// every nonzero row does.
    #[test]
    fn rows_shuffle_exactly_when_partners_exist() {
        for marks in 2..=30 {
            let dial = Dial::new(marks);
            for k in 1..marks {
                assert_eq!(dial.shuffles(k), dial.inverse(k).is_some(), "{k} on {marks}");
            }
            assert_eq!(dial.is_prime(), (1..marks).all(|k| dial.shuffles(k)), "{marks}");
        }
    }

    #[test]
    fn the_example_deals_the_videos_shares() {
        let r = Recipe::example();
        let heights: Vec<i64> = TRUSTEES.iter().map(|&x| r.height(x)).collect();
        assert_eq!(heights, vec![19, 47, 89, 145, 215]);
        assert_eq!(r.deal(DIAL, &TRUSTEES), vec![8, 3, 1, 2, 6]);
        assert_eq!(r.deal(TEN_DIAL, &TRUSTEES), vec![9, 7, 9, 5, 5]);
        // the share at position 0 is always the secret
        for t in tuples(DIAL, 3) {
            let r = Recipe::new(t[0], &t[1..]);
            assert_eq!(r.share(DIAL, 0), t[0]);
        }
    }

    /// Horner on the dial equals the ordinary height wrapped, for every
    /// recipe of up to three spins at every position.
    #[test]
    fn horner_on_the_dial_is_the_height_wrapped() {
        for spins in 0..=3 {
            for t in tuples(DIAL, spins + 1) {
                let r = Recipe::new(t[0], &t[1..]);
                for x in DIAL.all() {
                    assert_eq!(r.share(DIAL, x), DIAL.wrap(r.height(x)));
                }
            }
        }
    }

    #[test]
    fn fractions_reduce_and_land_on_the_dial() {
        assert_eq!(Ratio::new(20, 6).to_string(), "10/3");
        assert_eq!(Ratio::new(10, -2).to_string(), "-5");
        assert_eq!(Ratio::new(0, -7), Ratio::int(0));
        assert_eq!(Ratio::new(10, 3).on(DIAL), Some(7));
        assert_eq!(Ratio::new(8, 3).on(DIAL), Some(10));
        assert_eq!(Ratio::new(1, 2).on(TEN_DIAL), None);
    }

    #[test]
    fn the_scenes_weights_are_lagranges() {
        let w: Vec<String> = ordinary_weights(&OPENING_TRIO)
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(w, vec!["3", "-3", "1"]);
        assert_eq!(weights_at(DIAL, &OPENING_TRIO, 0), Some(vec![3, 8, 1]));
        let w: Vec<String> = ordinary_weights(&SECOND_TRIO).iter().map(ToString::to_string).collect();
        assert_eq!(w, vec!["10/3", "-5", "8/3"]);
        assert_eq!(weights_at(DIAL, &SECOND_TRIO, 0), Some(vec![7, 6, 10]));
        // weights at any position add up to one (they rebuild a constant)
        for at in -3..=12 {
            let total = ordinary_weights_at(&SECOND_TRIO, at)
                .into_iter()
                .fold(Ratio::int(0), Ratio::plus);
            assert_eq!(total, Ratio::int(1));
        }
    }

    /// Any three distinct nonzero positions rebuild every recipe's secret,
    /// and the curve's height anywhere; repeated positions rebuild nothing.
    #[test]
    fn three_shares_rebuild_every_recipe() {
        for xs in combinations(&(1..MARKS).collect::<Vec<_>>(), 3) {
            for t in tuples(DIAL, 3).step_by(7) {
                let r = Recipe::new(t[0], &t[1..]);
                let points: Vec<(u32, u32)> = xs.iter().map(|&x| (x, r.share(DIAL, x))).collect();
                assert_eq!(rebuild(DIAL, &points), Some(r.secret), "{xs:?} {t:?}");
                assert_eq!(interpolate(DIAL, &points, 4), Some(r.share(DIAL, 4)));
            }
        }
        assert_eq!(rebuild(DIAL, &[(1, 8), (1, 8), (2, 3)]), None);
    }

    /// Two shares at any two distinct nonzero positions, any values: exactly
    /// one spin pair per secret. One share: eleven per secret.
    #[test]
    fn two_shares_fit_every_secret_once() {
        let nonzero: Vec<u32> = (1..MARKS).collect();
        for xs in combinations(&nonzero, 2) {
            for values in tuples(DIAL, 2) {
                let points: Vec<(u32, u32)> = xs.iter().copied().zip(values).collect();
                for s in DIAL.all() {
                    assert_eq!(fitting_spins(DIAL, &points, s, 2).len(), 1, "{points:?} {s}");
                }
            }
        }
        for x in 1..MARKS {
            for y in DIAL.all() {
                for s in DIAL.all() {
                    assert_eq!(fitting_spins(DIAL, &[(x, y)], s, 2).len(), MARKS as usize);
                }
            }
        }
    }

    /// The privacy check passes on eleven marks for all ten pairs, and
    /// fails, as it must, on ten marks and with a forbidden zero spin.
    #[test]
    fn the_pair_check_passes_and_its_controls_fail() {
        for pair in combinations(&TRUSTEES, 2) {
            assert!(learns_nothing(DIAL, &pair, 2, any_spins), "{pair:?} on 11");
            assert!(!learns_nothing(DIAL, &pair, 2, |t| t[1] != 0), "{pair:?} with b ≠ 0");
        }
        let leaky = combinations(&TRUSTEES, 2)
            .into_iter()
            .filter(|p| !learns_nothing(TEN_DIAL, p, 2, any_spins))
            .count();
        assert_eq!(leaky, 9);
    }

    /// A nonzero recipe of two spins has at most two zeros on eleven marks,
    /// a nonzero line at most one; on ten, 5x + 5x² is zero everywhere and
    /// the line 5 + 5x at every odd mark.
    #[test]
    fn at_most_two_zeros_on_eleven() {
        for t in tuples(DIAL, 3).skip(1) {
            assert!(zeros(DIAL, &Recipe::new(t[0], &t[1..])) <= 2, "{t:?}");
        }
        assert_eq!(most_zeros(DIAL, 2), 2);
        assert_eq!(most_zeros(DIAL, 1), 1);
        assert_eq!(zeros(TEN_DIAL, &Recipe::new(0, &[5, 5])), 10);
        assert_eq!(most_zeros(TEN_DIAL, 1), 5);
        assert_eq!(zeros(TEN_DIAL, &Recipe::new(5, &[5])), 5);
        assert_eq!(zero_product(TEN_DIAL), Some((2, 5)));
        assert_eq!(zero_product(DIAL), None);
        assert_eq!(zero_product(Dial::new(9)), Some((3, 3)));
    }

    /// On ten marks trustees 4 and 5 have a shortcut, 5·y4 − 4·y5; on eleven
    /// no pair of trustees has one (it would contradict the counts above).
    #[test]
    fn a_pair_shortcut_exists_only_on_ten() {
        assert_eq!(pair_shortcut(TEN_DIAL, PAIR), Some((5, -4)));
        for pair in combinations(&TRUSTEES, 2) {
            assert_eq!(pair_shortcut(DIAL, [pair[0], pair[1]]), None, "{pair:?}");
        }
        // the shortcut is s − 20b in ordinary numbers: −20 vanishes on ten
        let r = Recipe::example();
        let (y4, y5) = (r.height(4), r.height(5));
        assert_eq!(5 * y4 - 4 * y5, i64::from(SECRET) - 20 * i64::from(SPINS[1]));
    }

    #[test]
    fn views_of_the_vault() {
        assert_eq!(View::Outsider.possible(DIAL).len(), 11);
        assert_eq!(View::Copy(SECRET).possible(DIAL), vec![SECRET]);
        assert_eq!(View::OrdinaryPiece(3).possible(DIAL), (3..11).collect::<Vec<_>>());
        for b in DIAL.all() {
            assert_eq!(View::OrdinaryPiece(b).possible(DIAL).len(), (MARKS - b) as usize);
        }
        let pair = View::Shares {
            points: vec![(4, 2), (5, 6)],
            spins: 2,
        };
        assert_eq!(pair.possible(DIAL).len(), 11);
        let trio = View::Shares {
            points: vec![(1, 8), (4, 2), (5, 6)],
            spins: 2,
        };
        assert_eq!(trio.possible(DIAL), vec![SECRET]);
        let ten_pair = View::Shares {
            points: vec![(4, 5), (5, 5)],
            spins: 2,
        };
        assert_eq!(ten_pair.possible(TEN_DIAL), vec![SECRET]);
    }

    /// On paper one height leaves the secrets a whole spin reaches it from,
    /// as `paper_line_secrets` counts them; a curve's two heights leave
    /// what `paper_whole_spin_secrets` does; two points of chapter 07's
    /// line leave only the secret. A height one too high leaves another
    /// line's secret (13 and 18 climb 5 a step from 3), and one that makes
    /// the climb a fraction (13 and 22, 9/2 a step) leaves none.
    #[test]
    fn the_paper_view_agrees_with_the_paper_counts() {
        let one = View::Paper {
            points: vec![(5, 25)],
            spins: 1,
        };
        assert_eq!(one.possible(DIAL), paper_line_secrets(MARKS, 5, 25));
        assert_eq!(one.possible(DIAL), vec![0, 5, 10]);
        let pair = [(4, 145), (5, 215)];
        let two = View::Paper {
            points: pair.to_vec(),
            spins: 2,
        };
        assert_eq!(two.possible(DIAL), paper_whole_spin_secrets(MARKS, pair));
        let line = Recipe::new(SECRET, &[LINE_SLOPE]);
        let both = View::Paper {
            points: vec![(2, line.height(2)), (3, line.height(3))],
            spins: 1,
        };
        assert_eq!(both.possible(DIAL), vec![SECRET]);
        let other = View::Paper {
            points: vec![(2, line.height(2)), (3, line.height(3) + 1)],
            spins: 1,
        };
        assert_eq!(other.possible(DIAL), vec![3]);
        let none = View::Paper {
            points: vec![(2, line.height(2)), (4, line.height(4) + 1)],
            spins: 1,
        };
        assert!(none.possible(DIAL).is_empty());
    }

    /// No plan at all does with fewer locks than the padlock answer, for any
    /// one, two or three of five; nothing works for none or six of five.
    #[test]
    fn the_padlock_answer_is_the_fewest() {
        for needed in 1..=THRESHOLD {
            let plan = Padlocks::new(&TRUSTEES, needed);
            assert_eq!(fewest_locks(&TRUSTEES, needed), Some(plan.locks().len()), "{needed}");
        }
        assert_eq!(fewest_locks(&TRUSTEES, THRESHOLD), Some(10));
        assert_eq!(fewest_locks(&TRUSTEES, 0), None);
        assert_eq!(fewest_locks(&TRUSTEES, TRUSTEES.len() + 1), None);
    }

    #[test]
    fn paper_leaks_where_the_dial_does_not() {
        assert_eq!(paper_line_secrets(MARKS, 5, 25), vec![0, 5, 10]);
        let pair = [(4, 145), (5, 215)];
        assert_eq!(paper_whole_spin_secrets(MARKS, pair), vec![5]);
        let (a, b) = paper_curve_spins(pair, 0).unwrap();
        assert_eq!((a.to_string(), b.to_string()), ("37/4".into(), "27/4".into()));
        assert_eq!(paper_curve_spins(pair, 5), Some((Ratio::int(7), Ratio::int(7))));
        // every secret has a paper curve; only one with spins 0 to 10 is dealable
        let a_mark = |r: Ratio| DIAL.all().any(|k| r == Ratio::int(i64::from(k)));
        for s in DIAL.all() {
            let (a, b) = paper_curve_spins(pair, s).unwrap();
            assert_eq!(a_mark(a) && a_mark(b), s == SECRET, "secret {s}: {a}, {b}");
        }
    }

    /// The plans' closed forms against a search over every group of
    /// trustees, for every set of holders.
    #[test]
    fn copies_against_a_search() {
        let n = TRUSTEES.len();
        for size in 0..=n {
            for holders in combinations(&TRUSTEES, size) {
                let groups = || (0..=n).flat_map(|k| combinations(&TRUSTEES, k));
                let shut = groups()
                    .filter(|lost| holders.iter().all(|h| lost.contains(h)))
                    .map(|g| g.len())
                    .min();
                assert_eq!(shut, Some(losses_to_shut(&holders)), "{holders:?}");
                let open = groups()
                    .filter(|b| holders.iter().any(|h| b.contains(h)))
                    .map(|g| g.len())
                    .min();
                assert_eq!(open, bribes_to_open(&holders), "{holders:?}");
            }
        }
        for needed in 0..=n + 1 {
            let most = (0..=n)
                .filter(|&k| combinations(&TRUSTEES, k).iter().all(|away| n - away.len() >= needed))
                .max();
            assert_eq!(most.unwrap_or(0), can_be_away(n, needed), "needed {needed}");
        }
        assert_eq!(can_be_away(n, THRESHOLD), 2);
    }

    /// Chapter 15: the opening trio's whole weights give the secret from the
    /// heights on paper and, wrapped, from the shares, for every recipe;
    /// chapters 16 to 18: the second trio's thirds are not always whole on the
    /// shares (the control), though three times them are, and they give three
    /// times the secret.
    #[test]
    fn whole_weights_commute_with_wrapping_and_thirds_do_not() {
        let tall = |r: &Recipe, xs: &[u32]| xs.iter().map(|&x| (x, r.height(x))).collect::<Vec<_>>();
        let dealt = |r: &Recipe, xs: &[u32]| xs.iter().map(|&x| (x, r.share(DIAL, x))).collect::<Vec<_>>();
        let (clear, w) = cleared_weights(&SECOND_TRIO);
        let clear = u32::try_from(clear).unwrap();
        let mut fractional = 0;
        for t in tuples(DIAL, 3) {
            let r = Recipe::new(t[0], &t[1..]);
            let secret = Ratio::int(i64::from(r.secret));
            assert_eq!(paper_sum(&tall(&r, &OPENING_TRIO)), secret, "{t:?}");
            assert_eq!(paper_sum(&tall(&r, &SECOND_TRIO)), secret, "{t:?}");
            let whole = ordinary_sum(&dealt(&r, &OPENING_TRIO)).whole().unwrap();
            assert_eq!(DIAL.wrap(whole), r.secret, "{t:?}");
            let thirds = ordinary_sum(&dealt(&r, &SECOND_TRIO));
            if thirds.whole().is_none() {
                fractional += 1;
            }
            let thrice: i64 = w
                .iter()
                .zip(dealt(&r, &SECOND_TRIO))
                .map(|(wi, (_, y))| wi * i64::from(y))
                .sum();
            assert_eq!(undo(DIAL, clear, DIAL.wrap(thrice)), Some(r.secret), "{t:?}");
        }
        assert!(fractional > 0, "the thirds always cancelled");
        assert_eq!(cleared_weights(&SECOND_TRIO), (3, vec![10, -15, 8]));
        assert_eq!(cleared_weights(&OPENING_TRIO), (1, vec![3, -3, 1]));
        assert_eq!(undo(DIAL, 3, 4), Some(5));
        assert_eq!(undo(TEN_DIAL, 5, 5), None);
    }

    /// Chapter 20's ghost: on ten marks a nonzero-secret recipe can vanish at
    /// three trustees and move the secret without changing their shares; on
    /// eleven no trio of trustees has one.
    #[test]
    fn ghosts_exist_only_on_ten() {
        for xs in combinations(&TRUSTEES, 3) {
            assert_eq!(ghost(DIAL, &xs), None, "{xs:?}");
        }
        let g = ghost(TEN_DIAL, &[1, 3, 5]).unwrap();
        assert_eq!(g.coefficients(), vec![5, 5, 0]);
        let moved = Recipe::example().plus(TEN_DIAL, &g);
        assert_eq!(moved.coefficients(), vec![0, 2, 7]);
        assert_eq!(
            moved.deal(TEN_DIAL, &[1, 3, 5]),
            Recipe::example().deal(TEN_DIAL, &[1, 3, 5])
        );
        assert_ne!(moved.secret, SECRET);
    }

    /// Chapter 22: five shares with one lie leave exactly one curve through
    /// four of them, the dealer's, for every recipe, liar and lie; four
    /// shares with a lie fit no curve at all; with no lie the dealer's curve
    /// passes through all five.
    #[test]
    fn one_liar_among_five_is_found() {
        for t in tuples(DIAL, 3).step_by(5) {
            let r = Recipe::new(t[0], &t[1..]);
            let honest: Vec<(u32, u32)> = TRUSTEES.iter().map(|&x| (x, r.share(DIAL, x))).collect();
            let all_five = curves_through(DIAL, &honest, 5);
            assert_eq!(all_five, vec![(r.secret, r.deal(DIAL, &TRUSTEES))], "{t:?}");
            for liar in 0..TRUSTEES.len() {
                for lie in DIAL.all().filter(|&v| v != honest[liar].1) {
                    let mut handed = honest.clone();
                    handed[liar].1 = lie;
                    let near = curves_through(DIAL, &handed, 4);
                    assert_eq!(near, vec![(r.secret, r.deal(DIAL, &TRUSTEES))], "{t:?} {liar} {lie}");
                    if liar < FOURTH as usize {
                        assert!(curves_through(DIAL, &handed[..FOURTH as usize], 4).is_empty());
                    }
                }
            }
        }
    }

    #[test]
    fn padlocks_and_shamirs_counts() {
        let p = Padlocks::new(&TRUSTEES, 3);
        assert_eq!(p.locks().len(), 10);
        assert!(TRUSTEES.iter().all(|&t| p.keys(t) == 6));
        assert!(combinations(&TRUSTEES, 3).iter().all(|g| p.opens(g)));
        assert!(combinations(&TRUSTEES, 2).iter().all(|g| p.stuck_at(g) == 1));
        assert_eq!(combinations(&TRUSTEES, 6), Vec::<Vec<u32>>::new());
        let scientists: Vec<u32> = (1..=11).collect();
        let liu = Padlocks::new(&scientists, 6);
        assert_eq!(liu.locks().len(), 462);
        assert_eq!(liu.keys(1), 252);
    }
}
