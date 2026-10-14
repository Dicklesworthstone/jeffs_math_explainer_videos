//! Act IV: an economy as an input-output table (Leontief). Entry (i, j) of
//! the table C is how much of good i is used up in making one unit of good
//! j. Making the amounts x uses up C x, so delivering d to the people who
//! eat and build takes x = C x + d. Whether that can be done for every d is
//! decided by C's largest eigenvalue: below one, each round of "make what
//! the last round used up" is smaller than the one before and the rounds add
//! up to (I − C)⁻¹ d; at one or above, they never stop growing.
//!
//! The video's economy has two industries, food and tools, and is worked in
//! exact fractions.

use std::fmt;

/// A fraction in lowest terms with a positive denominator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frac {
    pub num: i128,
    pub den: i128,
}

fn gcd(a: i128, b: i128) -> i128 {
    if b == 0 { a.abs() } else { gcd(b, a % b) }
}

impl Frac {
    /// # Panics
    /// If the denominator is zero.
    pub fn new(num: i128, den: i128) -> Self {
        assert!(den != 0, "a fraction needs a denominator");
        let g = gcd(num, den).max(1) * den.signum();
        Self {
            num: num / g,
            den: den / g,
        }
    }

    pub fn whole(n: i128) -> Self {
        Self::new(n, 1)
    }

    pub fn to_f64(self) -> f64 {
        self.num as f64 / self.den as f64
    }

    pub fn is_negative(self) -> bool {
        self.num < 0
    }
}

/// a·b, or a panic: a wrapped product would be a wrong number that looks right.
fn times(a: i128, b: i128) -> i128 {
    a.checked_mul(b).expect("a fraction outgrew 128 bits")
}

impl std::ops::Add for Frac {
    type Output = Frac;
    fn add(self, o: Frac) -> Frac {
        // over the least common denominator, so tenths of tenths stay small
        let shared = gcd(self.den, o.den);
        let num = times(self.num, o.den / shared)
            .checked_add(times(o.num, self.den / shared))
            .expect("a fraction outgrew 128 bits");
        Frac::new(num, times(self.den / shared, o.den))
    }
}

impl std::ops::Sub for Frac {
    type Output = Frac;
    fn sub(self, o: Frac) -> Frac {
        self + Frac::new(-o.num, o.den)
    }
}

impl std::ops::Mul for Frac {
    type Output = Frac;
    fn mul(self, o: Frac) -> Frac {
        Frac::new(times(self.num, o.num), times(self.den, o.den))
    }
}

impl std::ops::Div for Frac {
    type Output = Frac;
    fn div(self, o: Frac) -> Frac {
        Frac::new(times(self.num, o.den), times(self.den, o.num))
    }
}

impl fmt::Display for Frac {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.den == 1 {
            write!(f, "{}", self.num)
        } else {
            write!(f, "{}/{}", self.num, self.den)
        }
    }
}

/// A two-industry input table: `c[i][j]` of good i goes into one unit of good j.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Table {
    pub c: [[Frac; 2]; 2],
}

pub type Pair = [Frac; 2];

fn tenths(rows: [[i128; 2]; 2]) -> Table {
    Table {
        c: rows.map(|r| r.map(|x| Frac::new(x, 10))),
    }
}

/// The video's economy: a unit of food takes 0.4 of food and 0.2 of tools;
/// a unit of tools takes 0.3 of food and 0.5 of tools.
pub fn thrifty() -> Table {
    tenths([[4, 3], [2, 5]])
}

/// The same economy with a bigger appetite: every entry two tenths more.
pub fn hungry() -> Table {
    tenths([[6, 5], [4, 7]])
}

impl Table {
    /// What making the amounts `x` uses up: C x.
    pub fn eats(&self, x: Pair) -> Pair {
        [
            self.c[0][0] * x[0] + self.c[0][1] * x[1],
            self.c[1][0] * x[0] + self.c[1][1] * x[1],
        ]
    }

    pub fn trace(&self) -> Frac {
        self.c[0][0] + self.c[1][1]
    }

    pub fn det(&self) -> Frac {
        self.c[0][0] * self.c[1][1] - self.c[0][1] * self.c[1][0]
    }

    /// The two eigenvalues, largest first, when they are fractions: the
    /// roots of λ² − trace·λ + det. `None` if the roots are not rational.
    pub fn eigenvalues(&self) -> Option<[Frac; 2]> {
        let (t, d) = (self.trace(), self.det());
        let disc = t * t - Frac::whole(4) * d;
        let root = Frac::new(exact_sqrt(disc.num)?, exact_sqrt(disc.den)?);
        let half = Frac::new(1, 2);
        Some([(t + root) * half, (t - root) * half])
    }

    /// An eigenvector for the eigenvalue `lambda`, in the smallest whole
    /// numbers, first entry positive: (c01, λ − c00) scaled.
    pub fn eigenvector(&self, lambda: Frac) -> [i128; 2] {
        let (a, b) = (self.c[0][1], lambda - self.c[0][0]);
        let scale = a.den * b.den / gcd(a.den, b.den);
        let (x, y) = (a.num * (scale / a.den), b.num * (scale / b.den));
        let g = gcd(x, y).max(1) * if x < 0 { -1 } else { 1 };
        [x / g, y / g]
    }

    /// A left eigenvector for `lambda` (prices p with p C = λ p), smallest
    /// whole numbers, first entry positive.
    pub fn left_eigenvector(&self, lambda: Frac) -> [i128; 2] {
        Table {
            c: [[self.c[0][0], self.c[1][0]], [self.c[0][1], self.c[1][1]]],
        }
        .eigenvector(lambda)
    }

    /// (I − C)⁻¹, or `None` when I − C cannot be inverted.
    pub fn leontief_inverse(&self) -> Option<[[Frac; 2]; 2]> {
        let one = Frac::whole(1);
        let m = [
            [one - self.c[0][0], Frac::whole(0) - self.c[0][1]],
            [Frac::whole(0) - self.c[1][0], one - self.c[1][1]],
        ];
        let det = m[0][0] * m[1][1] - m[0][1] * m[1][0];
        if det.num == 0 {
            return None;
        }
        Some([
            [m[1][1] / det, (Frac::whole(0) - m[0][1]) / det],
            [(Frac::whole(0) - m[1][0]) / det, m[0][0] / det],
        ])
    }

    /// What must be made to deliver `d`, by the inverse: x = (I − C)⁻¹ d.
    pub fn output_for(&self, d: Pair) -> Option<Pair> {
        let inv = self.leontief_inverse()?;
        Some([inv[0][0] * d[0] + inv[0][1] * d[1], inv[1][0] * d[0] + inv[1][1] * d[1]])
    }

    /// The same by rounds: round 0 makes d, round 1 makes what round 0 used
    /// up, and so on. The amounts made in each of the first `rounds + 1`
    /// rounds, exactly (tenths to the thirtieth power still fit; a longer
    /// run panics rather than wrap).
    pub fn rounds(&self, d: Pair, rounds: usize) -> Vec<Pair> {
        let mut out = vec![d];
        for _ in 0..rounds {
            out.push(self.eats(*out.last().expect("round 0 is there")));
        }
        out
    }

    /// Whether the economy can deliver every demand: I − C has an inverse
    /// with no negative entry (then x = (I − C)⁻¹ d is a plan for any d ≥ 0).
    pub fn feeds_itself(&self) -> bool {
        self.leontief_inverse()
            .is_some_and(|inv| inv.iter().flatten().all(|x| !x.is_negative()))
    }

    /// Hawkins and Simon's condition (1949) for two industries: 1 − c00 > 0
    /// and det(I − C) > 0.
    pub fn hawkins_simon(&self) -> bool {
        let one = Frac::whole(1);
        let det = (one - self.c[0][0]) * (one - self.c[1][1]) - self.c[0][1] * self.c[1][0];
        (one - self.c[0][0]).num > 0 && det.num > 0
    }

    /// The table with every entry multiplied by `factor`.
    pub fn scaled(&self, factor: Frac) -> Table {
        Table {
            c: self.c.map(|r| r.map(|x| x * factor)),
        }
    }
}

/// The whole-number square root of a perfect square.
fn exact_sqrt(n: i128) -> Option<i128> {
    if n < 0 {
        return None;
    }
    let r = (n as f64).sqrt().round() as i128;
    (r - 1..=r + 1).find(|c| c * c == n)
}

/// The eigenvalues of a 2 by 2 table of positive entries in floating point,
/// largest first, each with an eigenvector whose first entry is 1. A table
/// of positive entries always has two real eigenvalues (the discriminant is
/// (a − d)² + 4bc > 0).
pub fn positive_eigen(m: [[f64; 2]; 2]) -> [(f64, [f64; 2]); 2] {
    let (t, det) = (m[0][0] + m[1][1], m[0][0] * m[1][1] - m[0][1] * m[1][0]);
    let root = (t * t - 4.0 * det).sqrt();
    [(t + root) / 2.0, (t - root) / 2.0].map(|lambda| (lambda, [1.0, (lambda - m[0][0]) / m[0][1]]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair(a: i128, b: i128) -> Pair {
        [Frac::whole(a), Frac::whole(b)]
    }

    #[test]
    fn the_even_mix_comes_back_as_seven_tenths_of_itself() {
        let c = thrifty();
        assert_eq!(c.eats(pair(10, 10)), pair(7, 7));
        assert_eq!(c.eigenvalues().unwrap(), [Frac::new(7, 10), Frac::new(1, 5)]);
        assert_eq!(c.eigenvector(Frac::new(7, 10)), [1, 1]);
        assert_eq!(c.eigenvector(Frac::new(1, 5)), [3, -2]);
        assert_eq!(c.left_eigenvector(Frac::new(7, 10)), [2, 3]);
        assert_eq!(hungry().eigenvalues().unwrap()[0], Frac::new(11, 10));
        assert_eq!(hungry().eats(pair(10, 10)), pair(11, 11));
    }

    #[test]
    fn the_rounds_add_up_to_the_inverse() {
        let c = thrifty();
        assert_eq!(c.output_for(pair(24, 24)).unwrap(), pair(80, 80));
        assert_eq!(c.output_for(pair(30, 10)).unwrap(), pair(75, 50));
        // after rounds 0 to 20 exactly 80·(1 − (7/10)²¹) of each has been made
        let rounds = c.rounds(pair(24, 24), 20);
        let made = rounds.iter().fold(pair(0, 0), |s, r| [s[0] + r[0], s[1] + r[1]]);
        let mut left = Frac::whole(80);
        for _ in 0..21 {
            left = left * Frac::new(7, 10);
        }
        assert_eq!(made, [Frac::whole(80) - left, Frac::whole(80) - left]);
        assert!(c.feeds_itself() && c.hawkins_simon());
        assert!(!hungry().feeds_itself() && !hungry().hawkins_simon());
    }

    #[test]
    fn fractions_stay_in_lowest_terms() {
        assert_eq!(Frac::new(6, -4), Frac::new(-3, 2));
        assert_eq!((Frac::new(1, 2) + Frac::new(1, 3)).to_string(), "5/6");
        assert_eq!((Frac::new(1, 2) / Frac::new(1, 4)).to_string(), "2");
    }
}
