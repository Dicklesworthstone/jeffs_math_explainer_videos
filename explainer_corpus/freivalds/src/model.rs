//! The mathematics of the video, exact and in whole numbers: a 3 × 3
//! product A·B, a claimed answer C with one entry wrong, and Freivalds'
//! check. The check tosses a coin for each column (the 0/1 column r), adds
//! the heads columns of B and of C (B·r and C·r, additions only), and
//! compares A·(B·r) with C·r. Plain data and integer arithmetic, no
//! drawing: `facts.rs` turns it into the numbers the video shows and says,
//! and `oracle.py` recomputes every one of them another way.
//!
//! What the check guarantees, in exact arithmetic (any ring; here the
//! integers): if C = A·B every toss passes, because A·(B·r) = (A·B)·r; if
//! C ≠ A·B, at most half of the 2^n tosses pass (UIUC CS 473, Fall 2019,
//! lecture 12; Freivalds 1977). The proof pairs the tosses: take a column
//! where C is wrong; two tosses that differ only in that column's coin
//! cannot both pass. [`sweep`] checks that pairing and the bound on every
//! small mistake. The guarantee says nothing about floating point:
//! [`rounded_passes`] runs the same check in doubles, where a right answer
//! can fail and a wrong one can pass every toss.

/// An `N` × `N` matrix of whole numbers, row by row.
pub type Matrix<const N: usize> = [[i64; N]; N];

/// A column of `N` whole numbers.
pub type Column<const N: usize> = [i64; N];

/// One toss of `N` coins, one per column: heads (`true`) keeps the column.
pub type Toss<const N: usize> = [bool; N];

/// The example's side: three rows and three columns.
pub const SIDE: usize = 3;

/// The two matrices someone multiplied (chosen so that their product's nine
/// entries are the digits 1 to 9, each once).
pub const A: Matrix<SIDE> = [[1, 2, 1], [0, 3, 1], [3, 2, 0]];
pub const B: Matrix<SIDE> = [[2, 1, 0], [1, 0, 2], [2, 1, 3]];

/// The answer they hand over: A·B with one slip, in row 2, column 2 (the
/// product has 1 there, the claim 2).
pub const C: Matrix<SIDE> = [[6, 2, 7], [5, 2, 9], [8, 3, 4]];

/// The toss chapter 01 tries first: heads, heads, tails.
pub const FIRST_TOSS: Toss<SIDE> = [true, true, false];

/// The entry chapter 01 works out by hand (01_2) and spot-checks (01_3):
/// row 1, column 3, counted from 0.
pub const SPOT: (usize, usize) = (0, 2);

/// A second, smaller pair, for the bound's edge test at another size.
pub const A2: Matrix<2> = [[1, 2], [0, 3]];
pub const B2: Matrix<2> = [[2, 1], [1, 0]];

/// Mistakes of different shapes (D = C − A·B), each with its name: one
/// wrong entry (the example's own), two in a row that add, two in a row that
/// cancel, three in a row that pass three tosses of eight (not a power of
/// two), one in each of two rows, and one in every column.
pub const SHAPES: [(&str, Matrix<SIDE>); 6] = [
    ("one_entry", [[0, 0, 0], [0, 1, 0], [0, 0, 0]]),
    ("two_in_a_row", [[0, 0, 0], [0, 1, 1], [0, 0, 0]]),
    ("cancelling", [[0, 0, 0], [0, 1, -1], [0, 0, 0]]),
    ("three_in_a_row", [[0, 0, 0], [1, 1, -1], [0, 0, 0]]),
    ("two_rows", [[1, 0, 0], [0, 1, 0], [0, 0, 0]]),
    ("every_column", [[1, 0, 0], [0, 1, 0], [0, 0, 1]]),
];

/// Where doubles are two apart: 10^16 (2^53 ≤ 10^16 < 2^54), so 10^16 + 1
/// rounds back to 10^16 (a tie, to even).
pub const BIG: i64 = 10_000_000_000_000_000;

/// A right answer the rounded check rejects: (A, B, C) with C = A·B exactly.
/// B·r rounds 10^16 + 1 to 10^16 when both coins are heads, and the 1s the
/// true product keeps are lost.
pub const ROUNDED_REJECTS: [Matrix<2>; 3] = [[[1, 1], [0, 0]], [[BIG, 1], [-BIG, 1]], [[0, 2], [0, 0]]];

/// A wrong answer the rounded check passes on every toss: A·B has 10^16 + 1
/// in its corner, C has 10^16; in doubles A·(B·r) rounds to C·r.
pub const ROUNDED_PASSES: [Matrix<2>; 3] = [[[BIG, 1], [0, 0]], [[1, 0], [1, 0]], [[BIG, 0], [0, 0]]];

/// A toss as the column it multiplies by: 1 for heads, 0 for tails.
pub fn column_of<const N: usize>(toss: &Toss<N>) -> Column<N> {
    toss.map(i64::from)
}

/// Every toss of `N` coins, in counting order: the first coin is the most
/// significant digit, tails before heads (000, 001, …, 111 for three), so
/// the toss at index `t` reads `t` in binary.
pub fn tosses<const N: usize>() -> Vec<Toss<N>> {
    (0..1usize << N)
        .map(|code| std::array::from_fn(|j| ((code >> (N - 1 - j)) & 1) == 1))
        .collect()
}

/// A toss written as its coins, heads 1 and tails 0: "110".
pub fn toss_text<const N: usize>(toss: &Toss<N>) -> String {
    toss.iter().map(|&heads| if heads { '1' } else { '0' }).collect()
}

/// Entry (`i`, `j`) of `a` times `b`, term by term: row `i` of `a` times
/// column `j` of `b`, one product per pair, in order.
pub fn products_at<const N: usize>(a: &Matrix<N>, b: &Matrix<N>, i: usize, j: usize) -> Vec<i64> {
    a[i].iter().zip(b).map(|(&x, row)| x * row[j]).collect()
}

/// `a` times `b`, the schoolbook way (every entry a row times a column),
/// and how many multiplications it took.
pub fn product<const N: usize>(a: &Matrix<N>, b: &Matrix<N>) -> (Matrix<N>, u64) {
    let mut out = [[0; N]; N];
    let mut count = 0;
    for (i, row) in out.iter_mut().enumerate() {
        for (j, entry) in row.iter_mut().enumerate() {
            let terms = products_at(a, b, i, j);
            count += terms.len() as u64;
            *entry = terms.iter().sum();
        }
    }
    (out, count)
}

/// `m` times the column `v`, and how many multiplications it took.
pub fn times<const N: usize>(m: &Matrix<N>, v: &Column<N>) -> (Column<N>, u64) {
    let mut count = 0;
    let out = m.map(|row| {
        count += row.len() as u64;
        row.iter().zip(v).map(|(x, y)| x * y).sum::<i64>()
    });
    (out, count)
}

/// The columns of `m` whose coin landed heads, added: `m` times the toss,
/// with additions only (a coin is 0 or 1).
pub fn kept_sum<const N: usize>(m: &Matrix<N>, toss: &Toss<N>) -> Column<N> {
    m.map(|row| {
        row.iter()
            .zip(toss)
            .filter(|&(_, &heads)| heads)
            .map(|(&x, _)| x)
            .sum::<i64>()
    })
}

/// `m` plus `d`, entry by entry.
pub fn plus<const N: usize>(m: &Matrix<N>, d: &Matrix<N>) -> Matrix<N> {
    std::array::from_fn(|i| std::array::from_fn(|j| m[i][j] + d[i][j]))
}

/// The mistake in a claimed answer: `c` minus the true product `ab`.
pub fn mistake<const N: usize>(c: &Matrix<N>, ab: &Matrix<N>) -> Matrix<N> {
    std::array::from_fn(|i| std::array::from_fn(|j| c[i][j] - ab[i][j]))
}

/// Every (row, column) where `d` is not zero, counted from 0, row by row.
pub fn wrong_entries<const N: usize>(d: &Matrix<N>) -> Vec<(usize, usize)> {
    d.iter()
        .enumerate()
        .flat_map(|(i, row)| {
            row.iter()
                .enumerate()
                .filter(|&(_, &x)| x != 0)
                .map(move |(j, _)| (i, j))
        })
        .collect()
}

/// One toss of the check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Check<const N: usize> {
    /// B's heads columns added: B·r.
    pub br: Column<N>,
    /// A times that: A·(B·r), the true product's heads columns added.
    pub a_br: Column<N>,
    /// C's heads columns added: C·r.
    pub cr: Column<N>,
    /// The multiplications the check took (all of them in A·(B·r)).
    pub multiplications: u64,
}

impl<const N: usize> Check<N> {
    /// The two columns agree, so the toss lets C through.
    pub fn passes(&self) -> bool {
        self.a_br == self.cr
    }

    /// The rows where the two columns disagree, counted from 0.
    pub fn disagreements(&self) -> Vec<usize> {
        (0..N).filter(|&i| self.a_br[i] != self.cr[i]).collect()
    }
}

/// The check of `c` against `a` times `b` on one toss.
pub fn check<const N: usize>(a: &Matrix<N>, b: &Matrix<N>, c: &Matrix<N>, toss: &Toss<N>) -> Check<N> {
    let br = kept_sum(b, toss);
    let (a_br, multiplications) = times(a, &br);
    Check {
        br,
        a_br,
        cr: kept_sum(c, toss),
        multiplications,
    }
}

/// Every toss with its check, in counting order.
pub fn checks<const N: usize>(a: &Matrix<N>, b: &Matrix<N>, c: &Matrix<N>) -> Vec<(Toss<N>, Check<N>)> {
    tosses::<N>().into_iter().map(|t| (t, check(a, b, c, &t))).collect()
}

/// The tosses that let `c` through: A·(B·r) = C·r.
pub fn passing<const N: usize>(a: &Matrix<N>, b: &Matrix<N>, c: &Matrix<N>) -> Vec<Toss<N>> {
    checks(a, b, c)
        .into_iter()
        .filter(|(_, ch)| ch.passes())
        .map(|(t, _)| t)
        .collect()
}

/// A spot check recomputes one entry of `c`: over every entry, how many a
/// spot check would find right, and how many entries there are.
pub fn spot_checks<const N: usize>(a: &Matrix<N>, b: &Matrix<N>, c: &Matrix<N>) -> (usize, usize) {
    let (ab, _) = product(a, b);
    let agree = ab
        .iter()
        .flatten()
        .zip(c.iter().flatten())
        .filter(|(x, y)| x == y)
        .count();
    (agree, N * N)
}

/// Two tosses made one after the other, independently: of every ordered
/// pair of tosses, how many both let `c` through, and how many pairs there
/// are.
pub fn both_pass<const N: usize>(a: &Matrix<N>, b: &Matrix<N>, c: &Matrix<N>) -> (usize, usize) {
    let passes: Vec<bool> = checks(a, b, c).iter().map(|(_, ch)| ch.passes()).collect();
    let both = passes
        .iter()
        .map(|&p| passes.iter().filter(|&&q| p && q).count())
        .sum::<usize>();
    (both, passes.len() * passes.len())
}

/// The same toss used twice (the second check repeats the first): how many
/// tosses pass both times, and how many tosses there are. Reuse adds nothing.
pub fn same_toss_twice<const N: usize>(a: &Matrix<N>, b: &Matrix<N>, c: &Matrix<N>) -> (usize, usize) {
    let all = checks(a, b, c);
    let both = all
        .iter()
        .filter(|(t, first)| first.passes() && check(a, b, c, t).passes())
        .count();
    (both, all.len())
}

/// For whoever sees the toss before writing C: a mistake that toss cannot
/// see. One wrong entry in a tails column; with every coin heads, two wrong
/// entries in one row that cancel. None when there is a single coin and it
/// is heads (n = 1): nothing hides from that toss.
pub fn blind_spot<const N: usize>(toss: &Toss<N>) -> Option<Matrix<N>> {
    let mut d = [[0; N]; N];
    match toss.iter().position(|&heads| !heads) {
        Some(j) => d[0][j] = 1,
        None if N >= 2 => {
            d[0][0] = 1;
            d[0][1] = -1;
        }
        None => return None,
    }
    Some(d)
}

/// Multiplications in the schoolbook product of two n × n matrices: n³.
pub fn product_multiplications(n: u64) -> u64 {
    n.pow(3)
}

/// Multiplications in one check of n × n matrices (A times one column): n².
pub fn check_multiplications(n: u64) -> u64 {
    n * n
}

/// What every small mistake does: over each wrong answer C = A·B + D whose
/// mistake D has its entries drawn from a list of values (every D but zero).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sweep {
    /// How many wrong answers.
    pub wrong: usize,
    /// The fewest tosses any of them passes (all tails always passes).
    pub fewest: usize,
    /// The most tosses any of them passes (the bound says half).
    pub most: usize,
    /// How many pass exactly half the tosses.
    pub at_half: usize,
    /// How many pass a number of tosses that is not a power of two.
    pub uneven: usize,
    /// Coin pairs that both pass: two tosses that differ only in the coin
    /// of a column where D is not zero. The proof says there are none.
    pub pairs_both_passing: usize,
}

/// The `code`-th matrix with its entries from `values`, the first entry
/// changing fastest (an odometer over the cells).
fn numbered<const N: usize>(values: &[i64], code: usize) -> Matrix<N> {
    let mut rest = code;
    let mut d = [[0; N]; N];
    for x in d.iter_mut().flatten() {
        *x = values[rest % values.len()];
        rest /= values.len();
    }
    d
}

/// Of the pairs of tosses that differ only in the coin of a column where
/// `d` is not zero, how many both pass (`passes` in counting order).
fn pairs_both_passing<const N: usize>(d: &Matrix<N>, passes: &[bool]) -> usize {
    (0..N)
        .filter(|&j| d.iter().any(|row| row[j] != 0))
        .map(|j| {
            // the toss at index t reads t in binary, first coin first
            let bit = 1usize << (N - 1 - j);
            (0..passes.len())
                .filter(|&t| (t & bit) == 0 && passes[t] && passes[t | bit])
                .count()
        })
        .sum()
}

/// Every mistake with entries from `values`, run through the real check
/// against `a` times `b`, every toss.
pub fn sweep<const N: usize>(a: &Matrix<N>, b: &Matrix<N>, values: &[i64]) -> Sweep {
    let (ab, _) = product(a, b);
    let all = tosses::<N>();
    let mut s = Sweep {
        wrong: 0,
        fewest: usize::MAX,
        most: 0,
        at_half: 0,
        uneven: 0,
        pairs_both_passing: 0,
    };
    let cells = u32::try_from(N * N).expect("a small matrix");
    for code in 0..values.len().pow(cells) {
        let d = numbered::<N>(values, code);
        if d.iter().flatten().all(|&x| x == 0) {
            continue;
        }
        let c = plus(&ab, &d);
        let passes: Vec<bool> = all.iter().map(|t| check(a, b, &c, t).passes()).collect();
        let count = passes.iter().filter(|&&p| p).count();
        s.wrong += 1;
        s.fewest = s.fewest.min(count);
        s.most = s.most.max(count);
        s.at_half += usize::from(2 * count == all.len());
        s.uneven += usize::from(!count.is_power_of_two());
        s.pairs_both_passing += pairs_both_passing(&d, &passes);
    }
    if s.wrong == 0 {
        s.fewest = 0;
    }
    s
}

/// The same check in IEEE double precision, every sum left to right as the
/// loops run: does `toss` let `c` through? (The scope edge: rounding is a
/// different question from the exact one the guarantee answers.) The two
/// columns are compared bit for bit, which here is `==`: every sum starts
/// at +0.0, so none is −0.0, and finite entries make no NaN.
pub fn rounded_passes<const N: usize>(a: &Matrix<N>, b: &Matrix<N>, c: &Matrix<N>, toss: &Toss<N>) -> bool {
    let r: [f64; N] = toss.map(f64::from);
    let rounded_times = |m: &Matrix<N>, v: &[f64; N]| -> [f64; N] {
        m.map(|row| {
            let mut sum = 0.0;
            for (&x, &y) in row.iter().zip(v) {
                sum += x as f64 * y;
            }
            sum
        })
    };
    let a_br = rounded_times(a, &rounded_times(b, &r));
    a_br.map(f64::to_bits) == rounded_times(c, &r).map(f64::to_bits)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The product, worked by hand row times column, and the slip.
    #[test]
    fn the_example_is_the_product_with_one_slip() {
        let (ab, count) = product(&A, &B);
        assert_eq!(ab, [[6, 2, 7], [5, 1, 9], [8, 3, 4]]);
        assert_eq!(count, 27);
        let mut digits: Vec<i64> = ab.iter().flatten().copied().collect();
        digits.sort_unstable();
        assert_eq!(digits, (1..=9).collect::<Vec<_>>());
        assert_eq!(wrong_entries(&mistake(&C, &ab)), [(1, 1)]);
        assert_eq!(products_at(&A, &B, SPOT.0, SPOT.1), [0, 4, 3]);
    }

    #[test]
    fn tosses_count_in_binary_first_coin_first() {
        let all = tosses::<3>();
        assert_eq!(all.len(), 8);
        let texts: Vec<String> = all.iter().map(toss_text).collect();
        assert_eq!(texts, ["000", "001", "010", "011", "100", "101", "110", "111"]);
        assert_eq!(column_of(&FIRST_TOSS), [1, 1, 0]);
    }

    /// Adding the heads columns is multiplying by the 0/1 column, exactly,
    /// on every toss; and the check's multiplications are A's n².
    #[test]
    fn adding_heads_columns_is_multiplying_by_the_toss() {
        for t in tosses::<3>() {
            for m in [A, B, C] {
                assert_eq!(kept_sum(&m, &t), times(&m, &column_of(&t)).0, "{}", toss_text(&t));
            }
            assert_eq!(check(&A, &B, &C, &t).multiplications, 9);
        }
    }

    /// If C is right, every toss passes: A·(B·r) = (A·B)·r, on the example
    /// and on 2 × 2 pairs with entries from −2 to 2: all 625 A against 90
    /// regularly sampled B (every seventh code, 0 to 623), 56,250 pairs.
    #[test]
    fn a_right_answer_passes_every_toss() {
        let (ab, _) = product(&A, &B);
        assert_eq!(passing(&A, &B, &ab).len(), 8);
        let values = [-2, -1, 0, 1, 2];
        for code_a in 0..625 {
            let a = numbered::<2>(&values, code_a);
            for code_b in (0..625).step_by(7) {
                let b = numbered::<2>(&values, code_b);
                let (ab, _) = product(&a, &b);
                assert_eq!(passing(&a, &b, &ab).len(), 4, "{a:?} {b:?}");
            }
        }
    }

    /// The slip in row 2, column 2: exactly the tosses with that column's
    /// coin tails let it through, four of eight; the first toss catches it
    /// in row 2.
    #[test]
    fn the_slip_hides_exactly_from_tails_on_its_column() {
        let pass = passing(&A, &B, &C);
        assert_eq!(pass.len(), 4);
        for t in tosses::<3>() {
            assert_eq!(pass.contains(&t), !t[1], "{}", toss_text(&t));
        }
        let first = check(&A, &B, &C, &FIRST_TOSS);
        assert_eq!((first.br, first.a_br, first.cr), ([3, 1, 3], [8, 6, 11], [8, 7, 11]));
        assert!(!first.passes());
        assert_eq!(first.disagreements(), [1]);
    }

    /// The bound and its proof on every small mistake, at two sizes: never
    /// more than half, exactly half for some (tight), at least the all-tails
    /// toss, counts that need not be powers of two, and no coin pair where
    /// both tosses pass.
    #[test]
    fn no_mistake_passes_more_than_half() {
        let s3 = sweep(&A, &B, &[-1, 0, 1]);
        assert_eq!((s3.wrong, s3.fewest, s3.most), (19_682, 1, 4));
        assert!(s3.at_half > 0 && s3.uneven > 0);
        assert_eq!(s3.pairs_both_passing, 0);
        let s2 = sweep(&A2, &B2, &[-2, -1, 0, 1, 2]);
        assert_eq!((s2.wrong, s2.fewest, s2.most), (624, 1, 2));
        assert_eq!(s2.pairs_both_passing, 0);
        // a sweep over no mistake at all reports nothing, not usize::MAX
        assert_eq!(sweep(&A2, &B2, &[0]).fewest, 0);
    }

    /// The shapes: what each mistake lets through.
    #[test]
    fn mistakes_of_each_shape() {
        let (ab, _) = product(&A, &B);
        let counts: Vec<usize> = SHAPES
            .iter()
            .map(|(_, d)| passing(&A, &B, &plus(&ab, d)).len())
            .collect();
        assert_eq!(counts, [4, 2, 4, 3, 2, 1]);
        assert_eq!(plus(&ab, &SHAPES[0].1), C);
    }

    /// One independent toss after another: a quarter of the pairs both miss
    /// the slip; the same toss twice gains nothing; a right answer passes
    /// every pair.
    #[test]
    fn two_independent_tosses_and_one_toss_twice() {
        assert_eq!(both_pass(&A, &B, &C), (16, 64));
        let (ab, _) = product(&A, &B);
        assert_eq!(both_pass(&A, &B, &ab), (64, 64));
        assert_eq!(same_toss_twice(&A, &B, &C), (4, 8));
        // the cancelling mistake passes half the tosses too, and two tosses
        // still cut it to a quarter
        let cancelling = plus(&ab, &SHAPES[2].1);
        assert_eq!(both_pass(&A, &B, &cancelling), (16, 64));
    }

    /// A toss known in advance can be beaten: for every toss there is a
    /// wrong answer it lets through, at two sizes; a single heads coin is
    /// the exception.
    #[test]
    fn a_known_toss_can_be_fooled() {
        let (ab, _) = product(&A, &B);
        for t in tosses::<3>() {
            let d = blind_spot(&t).expect("three coins");
            assert!(!wrong_entries(&d).is_empty());
            assert!(check(&A, &B, &plus(&ab, &d), &t).passes(), "{}", toss_text(&t));
        }
        let (ab2, _) = product(&A2, &B2);
        for t in tosses::<2>() {
            let d = blind_spot(&t).expect("two coins");
            assert!(check(&A2, &B2, &plus(&ab2, &d), &t).passes());
        }
        assert_eq!(blind_spot(&[true]), None);
        assert!(blind_spot(&[false]).is_some());
    }

    #[test]
    fn a_spot_check_finds_the_slip_once_in_nine() {
        assert_eq!(spot_checks(&A, &B, &C), (8, 9));
        let (ab, _) = product(&A, &B);
        assert_eq!(spot_checks(&A, &B, &ab), (9, 9));
    }

    #[test]
    fn the_cost_formulas_are_the_counted_work() {
        let counted = [product(&[[2]], &[[3]]).1, product(&A2, &B2).1, product(&A, &B).1];
        for (n, count) in (1..=3).zip(counted) {
            assert_eq!(product_multiplications(n), count);
        }
        assert_eq!(check_multiplications(3), check(&A, &B, &C, &FIRST_TOSS).multiplications);
        assert_eq!(
            check_multiplications(2),
            check(&A2, &B2, &A2, &[true, false]).multiplications
        );
        assert_eq!(product_multiplications(1000), 1_000_000_000);
    }

    /// Exact arithmetic never rejects a right answer and never passes a
    /// wrong one on more than half the tosses; in doubles both happen.
    #[test]
    fn rounding_breaks_both_promises() {
        assert_eq!((BIG as f64 + 1.0).to_bits(), (BIG as f64).to_bits());
        assert_ne!((BIG as f64 + 2.0).to_bits(), (BIG as f64).to_bits());
        let [a, b, c] = ROUNDED_REJECTS;
        assert_eq!(product(&a, &b).0, c);
        let rounded: Vec<bool> = tosses::<2>().iter().map(|t| rounded_passes(&a, &b, &c, t)).collect();
        assert_eq!(rounded, [true, true, true, false]);
        assert_eq!(passing(&a, &b, &c).len(), 4);

        let [a, b, c] = ROUNDED_PASSES;
        assert_eq!(wrong_entries(&mistake(&c, &product(&a, &b).0)), [(0, 0)]);
        assert_eq!(passing(&a, &b, &c).len(), 2);
        assert!(tosses::<2>().iter().all(|t| rounded_passes(&a, &b, &c, t)));
    }
}
