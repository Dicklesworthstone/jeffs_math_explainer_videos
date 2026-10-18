//! The mathematics of "Why P-Values Confuse Even Doctors", exact and with
//! nothing drawn: chapters only draw what this module computes.
//!
//! Two worked examples, both counted rather than looked up:
//!
//! 1. **A positive test, in natural frequencies** (chapter 01, the clinics
//!    of the later chapters). A synthetic cohort of people, each with or
//!    without the disease and each with a positive or negative result. A
//!    conditional chance is a count: the people for whom the event holds,
//!    out of the people the condition picks out. The false-alarm rate
//!    counts positives among people *without* the disease; the chance a
//!    positive result means the disease (the positive predictive value)
//!    counts the disease among people *with a positive result*. Same
//!    people, different denominators; [`share`] makes the denominator an
//!    argument, so swapping the two is a visible change of argument, and the
//!    tests pin that the swap gives a different number.
//! 2. **A treatment study's imaginary world** (the null-tail example). A
//!    synthetic study of 20 pairs of patients, one of each pair given the
//!    treatment by a fair coin. The sharp null says the treatment changes
//!    no one's recovery, so every recovery time is fixed and only the labels
//!    are random: all 2^20 assignments are equally likely. [`Study::null_world`]
//!    lists every one of them and counts the pairs whose faster patient got
//!    the treatment; the p-value is the share of assignments at least as
//!    favourable as the observed one ([`Tail::AtLeast`]). Counting the wrong
//!    tail (only the observed count, only the more extreme ones, the other
//!    direction) gives a different number, and the tests pin each.
//!
//! Chapter 05 adds two constructed versions of the study's recovery days
//! ([`Days`], [`constructed_days`]): the same order in every pair, so the
//! same sign statistic and the same tail, but treated patients sooner by
//! about a day in one and about a week in the other. The statistic reads
//! only which patient recovered first, never by how much.
//!
//! Neither example's numbers are real patients: they are model numbers,
//! labelled so on screen. A diagnostic false-alarm rate is not a p-value and
//! a positive predictive value is not one either; the module keeps them as
//! different types of question ([`Clinic`] and [`Study`]).

use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Div, Mul, Sub};

// --------------------------------------------------------------- fractions

/// An exact fraction, always reduced, its denominator positive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Q {
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

impl Q {
    pub const ZERO: Q = Q { num: 0, den: 1 };
    pub const ONE: Q = Q { num: 1, den: 1 };

    /// `num / den`, reduced.
    ///
    /// # Panics
    /// On a zero denominator (a programming error: every denominator here is
    /// a count of people or of assignments that the caller checked).
    pub fn new(num: i128, den: i128) -> Self {
        assert!(den != 0, "a fraction with denominator zero: {num}/0");
        let g = gcd(num, den).max(1);
        let s = den.signum();
        Q {
            num: s * num / g,
            den: s * den / g,
        }
    }

    /// A whole number.
    pub fn int(n: i128) -> Self {
        Q { num: n, den: 1 }
    }

    /// A count out of a total.
    pub fn ratio(count: u64, total: u64) -> Self {
        Q::new(i128::from(count), i128::from(total))
    }

    pub fn num(self) -> i128 {
        self.num
    }

    pub fn den(self) -> i128 {
        self.den
    }

    pub fn is_integer(self) -> bool {
        self.den == 1
    }

    /// One minus this: the complement of a share.
    pub fn complement(self) -> Q {
        Q::ONE - self
    }

    pub fn f64(self) -> f64 {
        self.num as f64 / self.den as f64
    }

    /// `90` or `2/13`.
    pub fn exact(self) -> String {
        if self.den == 1 {
            self.num.to_string()
        } else {
            format!("{}/{}", self.num, self.den)
        }
    }

    /// The decimal with `places` digits, rounded half away from zero, in
    /// integers (never through a float): `Q::new(2, 13).decimal(4)` is
    /// `0.1538`.
    pub fn decimal(self, places: u32) -> String {
        let scale = 10i128.pow(places);
        let scaled = self.num.abs().checked_mul(scale).expect("decimal overflows i128");
        let q = (2 * scaled + self.den) / (2 * self.den);
        let sign = if self.num < 0 && q != 0 { "-" } else { "" };
        if places == 0 {
            format!("{sign}{q}")
        } else {
            format!("{sign}{}.{:0width$}", q / scale, q % scale, width = places as usize)
        }
    }

    /// This share as a percent with `places` decimals: `15.38`.
    pub fn percent(self, places: u32) -> String {
        (self * Q::int(100)).decimal(places)
    }
}

impl fmt::Display for Q {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.exact())
    }
}

fn checked(x: Option<i128>) -> i128 {
    x.expect("fraction arithmetic overflows i128")
}

impl Add for Q {
    type Output = Q;
    fn add(self, o: Q) -> Q {
        let n = checked(checked(self.num.checked_mul(o.den)).checked_add(checked(o.num.checked_mul(self.den))));
        Q::new(n, checked(self.den.checked_mul(o.den)))
    }
}

impl Sub for Q {
    type Output = Q;
    fn sub(self, o: Q) -> Q {
        let n = checked(checked(self.num.checked_mul(o.den)).checked_sub(checked(o.num.checked_mul(self.den))));
        Q::new(n, checked(self.den.checked_mul(o.den)))
    }
}

impl Mul for Q {
    type Output = Q;
    fn mul(self, o: Q) -> Q {
        Q::new(
            checked(self.num.checked_mul(o.num)),
            checked(self.den.checked_mul(o.den)),
        )
    }
}

impl Div for Q {
    type Output = Q;
    /// # Panics
    /// Dividing by zero.
    fn div(self, o: Q) -> Q {
        assert_ne!(o.num, 0, "{self} divided by zero");
        Q::new(
            checked(self.num.checked_mul(o.den)),
            checked(self.den.checked_mul(o.num)),
        )
    }
}

impl Ord for Q {
    fn cmp(&self, o: &Q) -> Ordering {
        // both denominators are positive
        checked(self.num.checked_mul(o.den)).cmp(&checked(o.num.checked_mul(self.den)))
    }
}

impl PartialOrd for Q {
    fn partial_cmp(&self, o: &Q) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

// ------------------------------------------------------- a positive result

/// A synthetic clinic: how many people it tests, how common the disease is
/// among them, and how the test behaves. The test's two rates are assumed
/// to be the same in every clinic (the storyboard says so on screen); only
/// the mix of people changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Clinic {
    pub people: u64,
    /// The share of the tested people who have the disease.
    pub prevalence: Q,
    /// The share of people with the disease whose result is positive.
    pub sensitivity: Q,
    /// The share of people without the disease whose result is positive.
    pub false_alarm: Q,
}

/// Chapter 01's clinic: ten thousand people, one in a hundred with the
/// disease, a test that catches nine in ten of them and falsely alarms one
/// person in twenty without it.
pub fn clinic_a() -> Clinic {
    Clinic {
        people: 10_000,
        prevalence: Q::new(1, 100),
        sensitivity: Q::new(9, 10),
        false_alarm: Q::new(1, 20),
    }
}

/// The same test in a clinic where one in ten has the disease (the next
/// chapter's comparison).
pub fn clinic_b() -> Clinic {
    Clinic {
        prevalence: Q::new(1, 10),
        ..clinic_a()
    }
}

/// A clinic's people, counted by disease and by result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Counts {
    pub people: u64,
    pub with: u64,
    pub without: u64,
    pub true_pos: u64,
    pub false_neg: u64,
    pub false_pos: u64,
    pub true_neg: u64,
}

/// `share` of `of` people, which must be a whole number of people: a
/// natural-frequency picture never draws a fraction of a person.
fn whole(share: Q, of: u64, what: &str) -> Result<u64, String> {
    if share < Q::ZERO || share > Q::ONE {
        return Err(format!("{what}: a share of {share} is not between 0 and 1"));
    }
    let n = share * Q::int(i128::from(of));
    if !n.is_integer() {
        return Err(format!("{what}: {share} of {of} is {n}, not a whole number of people"));
    }
    u64::try_from(n.num()).map_err(|e| format!("{what}: {e}"))
}

impl Clinic {
    /// Every count, each a whole number of people, or why not.
    pub fn counts(&self) -> Result<Counts, String> {
        let with = whole(self.prevalence, self.people, "people with the disease")?;
        let without = self.people - with;
        let true_pos = whole(self.sensitivity, with, "positives with the disease")?;
        let false_pos = whole(self.false_alarm, without, "false alarms")?;
        Ok(Counts {
            people: self.people,
            with,
            without,
            true_pos,
            false_neg: with - true_pos,
            false_pos,
            true_neg: without - false_pos,
        })
    }
}

impl Counts {
    /// Everyone whose result is positive, with the disease or without.
    pub fn positives(&self) -> u64 {
        self.true_pos + self.false_pos
    }
}

/// The positive predictive value from the three rates alone (Bayes' rule
/// for a uniformly chosen person of the clinic), the formula the counting
/// earns: `s q / (s q + f (1 − q))`.
pub fn bayes(prevalence: Q, sensitivity: Q, false_alarm: Q) -> Q {
    let caught = sensitivity * prevalence;
    caught / (caught + false_alarm * prevalence.complement())
}

/// One person of a clinic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Person {
    pub disease: bool,
    pub positive: bool,
}

pub fn disease(p: Person) -> bool {
    p.disease
}

pub fn no_disease(p: Person) -> bool {
    !p.disease
}

pub fn positive(p: Person) -> bool {
    p.positive
}

pub fn negative(p: Person) -> bool {
    !p.positive
}

/// The share of the people `given` picks out for whom `event` holds,
/// counted: the conditional chance for a person chosen uniformly from the
/// `given` group. The `given` group is the denominator; changing it is the
/// whole difference between a false-alarm rate and a positive predictive
/// value.
pub fn share(people: &[Person], event: fn(Person) -> bool, given: fn(Person) -> bool) -> Result<Q, String> {
    let group = people.iter().filter(|&&p| given(p)).count() as u64;
    if group == 0 {
        return Err("a share of an empty group".into());
    }
    let hits = people.iter().filter(|&&p| given(p) && event(p)).count() as u64;
    Ok(Q::ratio(hits, group))
}

/// A clinic's people as an icon array: rows of `cols` people, read row by
/// row. The people with the disease fill the first rows, those of them
/// with a positive result first; every row of people without the disease
/// has the same number of false alarms, at its start. (Chapter 01: one row
/// of a hundred with the disease, 90 of them positive; 99 rows without it,
/// 5 false alarms in each.)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IconArray {
    pub cols: usize,
    pub rows: usize,
    /// How many of the first rows hold the people with the disease.
    pub disease_rows: usize,
    /// False alarms at the start of each row without the disease.
    pub false_pos_per_row: usize,
    people: Vec<Person>,
}

impl IconArray {
    pub fn build(c: &Counts, cols: usize) -> Result<Self, String> {
        let n = |x: u64| usize::try_from(x).map_err(|e| e.to_string());
        let (people, with, without) = (n(c.people)?, n(c.with)?, n(c.without)?);
        if cols == 0 || !people.is_multiple_of(cols) || !with.is_multiple_of(cols) {
            return Err(format!(
                "{people} people with {with} with the disease do not fill rows of {cols}"
            ));
        }
        let (rows, disease_rows) = (people / cols, with / cols);
        let healthy_rows = without / cols;
        let false_pos = n(c.false_pos)?;
        if healthy_rows == 0 || !false_pos.is_multiple_of(healthy_rows) {
            return Err(format!(
                "{false_pos} false alarms do not split evenly over {healthy_rows} rows"
            ));
        }
        let per_row = false_pos / healthy_rows;
        let true_pos = n(c.true_pos)?;
        let mut out = Vec::with_capacity(people);
        for row in 0..rows {
            for col in 0..cols {
                let i = row * cols + col;
                out.push(if row < disease_rows {
                    Person {
                        disease: true,
                        positive: i < true_pos,
                    }
                } else {
                    Person {
                        disease: false,
                        positive: col < per_row,
                    }
                });
            }
        }
        Ok(IconArray {
            cols,
            rows,
            disease_rows,
            false_pos_per_row: per_row,
            people: out,
        })
    }

    pub fn at(&self, row: usize, col: usize) -> Person {
        self.people[row * self.cols + col]
    }

    pub fn people(&self) -> &[Person] {
        &self.people
    }
}

/// The positive results regrouped into one rectangle of whole rows, the
/// rows with the disease on top: `cols` is the largest row that both
/// groups fill exactly, so the disease's share of the rows *is* the
/// positive predictive value (chapter 01: 13 rows of 45, 2 of them with
/// the disease, 2/13).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PositivesBlock {
    pub cols: u64,
    pub rows: u64,
    pub disease_rows: u64,
}

pub fn positives_block(c: &Counts) -> Result<PositivesBlock, String> {
    let cols = gcd(i128::from(c.true_pos), i128::from(c.false_pos)) as u64;
    if cols == 0 {
        return Err("no positive results to regroup".into());
    }
    Ok(PositivesBlock {
        cols,
        rows: c.positives() / cols,
        disease_rows: c.true_pos / cols,
    })
}

/// The positive results regrouped into rows of exactly `cols` (chapter 02
/// draws both clinics' blocks 45 wide, so their heights compare: 13 rows
/// with 2 of the disease, 30 rows with 20), or why not: both groups must
/// fill whole rows, or the disease's share of the rows would not be the
/// positive predictive value.
pub fn positives_block_of(c: &Counts, cols: u64) -> Result<PositivesBlock, String> {
    if cols == 0 || c.positives() == 0 {
        return Err("no rows to regroup into".into());
    }
    if !c.true_pos.is_multiple_of(cols) || !c.false_pos.is_multiple_of(cols) {
        return Err(format!(
            "{} and {} positives do not both fill rows of {cols}",
            c.true_pos, c.false_pos
        ));
    }
    Ok(PositivesBlock {
        cols,
        rows: c.positives() / cols,
        disease_rows: c.true_pos / cols,
    })
}

// -------------------------------------------------- the study's null world

/// The most pairs a study (or its constructed days) may have: one bit of a
/// `u32` per pair, and a null world small enough to list in full.
pub const MAX_PAIRS: u32 = 30;

/// A synthetic treatment study (a teaching model, not a protocol): `pairs`
/// pairs of patients matched before treatment, one patient of each pair
/// given the treatment by a fair coin, recovery time measured, lower is
/// better, no ties. Only the signs matter to the statistic, so the data are
/// bits: bit `i` of `faster_a` is set when patient A of pair `i` recovered
/// faster, bit `i` of `treated_a` when patient A got the treatment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Study {
    pub pairs: u32,
    pub faster_a: u32,
    pub treated_a: u32,
}

/// The study of the later chapters: 20 pairs, 15 of them won by the
/// treated patient (pairs 3, 7, 11, 16 and 19 went the other way).
pub fn study() -> Study {
    let faster_a = 0b1011_0110_1100_1011_0101;
    let against = [3u32, 7, 11, 16, 19].iter().fold(0, |m, &i| m | 1 << i);
    Study {
        pairs: 20,
        faster_a,
        treated_a: faster_a ^ against,
    }
}

impl Study {
    fn mask(&self) -> u32 {
        assert!(self.pairs <= MAX_PAIRS, "{} pairs is too many to list", self.pairs);
        (1u32 << self.pairs) - 1
    }

    /// The statistic: how many pairs' faster patient carries the treatment
    /// label under the assignment `treated_a`.
    pub fn favouring(&self, treated_a: u32) -> u32 {
        (!(treated_a ^ self.faster_a) & self.mask()).count_ones()
    }

    /// The statistic for the assignment the study actually made.
    pub fn observed(&self) -> u32 {
        self.favouring(self.treated_a)
    }

    /// Whether patient A of `pair` recovered first (a fixed fact of the
    /// study: the sharp null never moves it).
    pub fn faster_is_a(&self, pair: u32) -> bool {
        self.faster_a >> pair & 1 == 1
    }

    /// Whether the assignment `treated_a` gives patient A of `pair` the
    /// treatment (one fair coin per pair).
    pub fn treated_is_a(&self, treated_a: u32, pair: u32) -> bool {
        treated_a >> pair & 1 == 1
    }

    /// Whether `pair` favours the treatment under `treated_a`: its treated
    /// patient is the one who recovered first.
    pub fn favours(&self, treated_a: u32, pair: u32) -> bool {
        self.treated_is_a(treated_a, pair) == self.faster_is_a(pair)
    }

    /// The assignment under which every pair favours the treatment except
    /// the (distinct) pairs in `against`: every coin on the patient who
    /// recovered first, then those pairs' coins turned over. With no pair
    /// against it is the one assignment that gives all `pairs`.
    pub fn favouring_all_but(&self, against: &[u32]) -> u32 {
        against.iter().fold(self.faster_a, |a, &i| a ^ (1 << i)) & self.mask()
    }

    /// `n` more tosses of the study's coins, each a whole assignment: the
    /// low `pairs` bits of one draw of the seeded stream (one fair, independent
    /// bit per pair), so `oracle.py` replays them with NumPy's
    /// `PCG64DXSM(seed).random_raw()`.
    pub fn retosses(&self, seed: u64, n: usize) -> Vec<u32> {
        let mut data = explainer_kit::stats::Data::new(seed);
        (0..n).map(|_| (data.raw() & u64::from(self.mask())) as u32).collect()
    }

    /// The imaginary world of the sharp null: if the treatment changed no
    /// one's recovery, every recovery time stays where it is and only the
    /// labels move, so each of the 2^pairs assignments is equally likely.
    /// List every one and count, for each k, the assignments under which
    /// exactly k pairs favour the treatment.
    pub fn null_world(&self) -> Vec<u64> {
        let mut counts = vec![0u64; self.pairs as usize + 1];
        for a in 0..=self.mask() {
            counts[self.favouring(a) as usize] += 1;
        }
        counts
    }
}

/// Which outcomes a tail counts, for an observed statistic k.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tail {
    /// k or more: as favourable as the observation, or more (the one-sided
    /// p-value: the statistic and the direction were fixed in advance; the
    /// boundary k is the observed result).
    AtLeast,
    /// Exactly k (a wrong tail: leaves out the more extreme outcomes).
    Exactly,
    /// More than k (a wrong tail: leaves out the observation itself).
    MoreThan,
    /// k or fewer (a wrong tail: the other direction).
    AtMost,
    /// At least as far from the middle as k, either way (a different
    /// question from the one-sided test).
    TwoSided,
}

impl Tail {
    /// Whether the count `j` of a world of `n` pairs belongs to this tail at
    /// the observed count `k`: the inequality said on screen.
    pub fn holds(self, j: u32, k: u32, n: u32) -> bool {
        let (j, k, n) = (i64::from(j), i64::from(k), i64::from(n));
        match self {
            Tail::AtLeast => j >= k,
            Tail::Exactly => j == k,
            Tail::MoreThan => j > k,
            Tail::AtMost => j <= k,
            Tail::TwoSided => (2 * j - n).abs() >= (2 * k - n).abs(),
        }
    }
}

/// The counts (bars) a tail at `observed` takes in a world of `pairs` pairs:
/// what chapter 04 highlights for each tail.
pub fn tail_members(pairs: u32, observed: u32, tail: Tail) -> Vec<u32> {
    (0..=pairs).filter(|&j| tail.holds(j, observed, pairs)).collect()
}

/// How many assignments of a null world the tail at `observed` counts.
pub fn tail_count(world: &[u64], observed: u32, tail: Tail) -> u64 {
    let n = world.len() as u32 - 1;
    world
        .iter()
        .enumerate()
        .filter(|&(j, _)| tail.holds(j as u32, observed, n))
        .map(|(_, &c)| c)
        .sum()
}

/// The tail's share of every assignment.
pub fn tail_share(world: &[u64], observed: u32, tail: Tail) -> Q {
    Q::ratio(tail_count(world, observed, tail), world.iter().sum())
}

// ------------------------------------ constructed recovery days (chapter 05)

/// Recovery days for the pairs of a study, the treated and the untreated
/// patient's in each pair: constructed for illustration (chapter 05), never
/// trial data. Whole days, so a time is finite and never negative by its
/// type; [`Days::new`] refuses what the type allows but a recovery time
/// cannot be, or the sign statistic cannot read: zero days, a tie (a tie has
/// no patient who recovered first), and more pairs than a study's bits hold
/// ([`MAX_PAIRS`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Days {
    treated: Vec<u32>,
    untreated: Vec<u32>,
}

impl Days {
    pub fn new(treated: Vec<u32>, untreated: Vec<u32>) -> Result<Days, String> {
        if treated.is_empty() || treated.len() != untreated.len() {
            return Err(format!(
                "{} treated and {} untreated patients: one of each per pair",
                treated.len(),
                untreated.len()
            ));
        }
        if treated.len() > MAX_PAIRS as usize {
            return Err(format!("{} pairs: a study holds at most {MAX_PAIRS}", treated.len()));
        }
        for (i, (&t, &u)) in treated.iter().zip(&untreated).enumerate() {
            if t == 0 || u == 0 {
                return Err(format!("pair {i}: a recovery takes at least one day"));
            }
            if t == u {
                return Err(format!(
                    "pair {i}: a tie ({t} days each) has no patient who recovered first"
                ));
            }
        }
        Ok(Days { treated, untreated })
    }

    /// Built from each pair's untreated patient's days and its observed
    /// difference, untreated minus treated.
    pub fn from_differences(untreated: &[u32], differences: &[i64]) -> Result<Days, String> {
        if untreated.len() != differences.len() {
            return Err(format!(
                "{} pairs of days and {} differences",
                untreated.len(),
                differences.len()
            ));
        }
        let treated = untreated
            .iter()
            .zip(differences)
            .enumerate()
            .map(|(i, (&u, &d))| {
                i64::from(u)
                    .checked_sub(d)
                    .and_then(|t| u32::try_from(t).ok())
                    .ok_or_else(|| format!("pair {i}: {u} days less {d} is not a recovery time"))
            })
            .collect::<Result<Vec<u32>, String>>()?;
        Days::new(treated, untreated.to_vec())
    }

    pub fn pairs(&self) -> usize {
        self.treated.len()
    }

    pub fn treated(&self) -> &[u32] {
        &self.treated
    }

    pub fn untreated(&self) -> &[u32] {
        &self.untreated
    }

    /// The observed paired differences, untreated minus treated, in days:
    /// positive when the treated patient recovered sooner. What happened in
    /// each pair, not what the treatment caused.
    pub fn differences(&self) -> Vec<i64> {
        self.untreated
            .iter()
            .zip(&self.treated)
            .map(|(&u, &t)| i64::from(u) - i64::from(t))
            .collect()
    }

    /// The sign statistic, read from the days: the pairs whose treated
    /// patient recovered first.
    pub fn favouring(&self) -> u32 {
        self.differences().iter().filter(|&&d| d > 0).count() as u32
    }

    /// The order the days imply, as a study's bits: bit `i` set when patient
    /// A of pair `i` recovered first, given who the coins treated.
    pub fn faster_a(&self, treated_a: u32) -> u32 {
        (0..self.pairs()).fold(0, |bits, i| {
            let (t, u) = (self.treated[i], self.untreated[i]);
            let a = if treated_a >> i & 1 == 1 { t } else { u };
            let b = if treated_a >> i & 1 == 1 { u } else { t };
            if a < b { bits | 1 << i } else { bits }
        })
    }

    /// How much sooner the treated patient recovered, typically, in the pairs
    /// that favour the treatment: the median of the positive differences.
    pub fn median_sooner(&self) -> Option<Q> {
        let mut sooner: Vec<i64> = self.differences().into_iter().filter(|&d| d > 0).collect();
        sooner.sort_unstable();
        let n = sooner.len();
        match n {
            0 => None,
            _ if n.is_multiple_of(2) => Some(Q::new(i128::from(sooner[n / 2 - 1] + sooner[n / 2]), 2)),
            _ => Some(Q::int(i128::from(sooner[n / 2]))),
        }
    }
}

/// The untreated patients' days in chapter 05's constructed versions (the
/// same in both).
const UNTREATED_DAYS: [u32; 20] = [
    12, 14, 11, 13, 15, 12, 10, 14, 13, 11, 12, 15, 14, 11, 13, 12, 16, 10, 13, 14,
];
/// Each pair's observed difference in the two versions: the study's signs
/// (pairs 3, 7, 11, 16 and 19 negative, the same in both), the treated
/// patients sooner by about a day in the first and about a week in the
/// second.
const SOONER: [[i64; 20]; 2] = [
    [1, 1, 2, -1, 1, 1, 2, -2, 1, 1, 1, -1, 2, 1, 1, 1, -1, 1, 1, -2],
    [7, 6, 8, -1, 7, 5, 9, -2, 7, 6, 8, -1, 7, 9, 5, 7, -1, 6, 8, -2],
];

/// The two constructed versions of the study's recovery days that chapter
/// 05 sets side by side: the same order as the study in every pair, so the
/// same 15 pairs favour the treatment, but different sizes.
pub fn constructed_days() -> [Days; 2] {
    SOONER.map(|d| Days::from_differences(&UNTREATED_DAYS, &d).expect("the constructed days are valid"))
}

#[cfg(test)]
mod tests {
    use explainer_kit::choose;

    use super::*;

    #[test]
    fn fractions_reduce_compare_and_round_half_away_from_zero() {
        assert_eq!(Q::new(90, 585), Q::new(2, 13));
        assert_eq!(Q::new(3, -6), Q::new(-1, 2));
        assert_eq!(Q::new(2, 13).exact(), "2/13");
        assert_eq!(Q::int(495).exact(), "495");
        assert_eq!(Q::new(2, 13).percent(2), "15.38");
        assert_eq!(Q::new(2, 13).percent(0), "15");
        assert_eq!(Q::new(11, 13).percent(2), "84.62");
        assert_eq!(Q::new(1, 8).decimal(2), "0.13");
        assert_eq!(Q::new(-1, 8).decimal(2), "-0.13");
        assert_eq!(Q::new(-1, 1000).decimal(2), "0.00");
        assert_eq!(Q::new(588, 10).decimal(0), "59");
        assert_eq!(Q::new(277, 367).percent(1), "75.5");
        assert!(Q::new(1, 20) < Q::new(2, 13) && Q::new(2, 13) < Q::new(19, 20));
        assert_eq!(Q::new(1, 3) + Q::new(1, 6), Q::new(1, 2));
        assert_eq!(Q::new(1, 3) - Q::new(1, 2), Q::new(-1, 6));
        assert_eq!(Q::new(2, 3) / Q::new(4, 9), Q::new(3, 2));
    }

    #[test]
    fn clinic_a_counts_whole_people() {
        let c = clinic_a().counts().unwrap();
        assert_eq!(
            (c.with, c.without, c.true_pos, c.false_neg, c.false_pos, c.true_neg),
            (100, 9_900, 90, 10, 495, 9_405)
        );
        assert_eq!(c.positives(), 585);
        assert_eq!(c.with + c.without, c.people);
        let b = clinic_b().counts().unwrap();
        assert_eq!(
            (b.with, b.true_pos, b.false_pos, b.positives()),
            (1_000, 900, 450, 1_350)
        );
    }

    /// A natural-frequency picture never draws part of a person: a rate that
    /// does not divide its group is refused, not rounded.
    #[test]
    fn a_rate_that_splits_a_person_is_refused() {
        let thirds = Clinic {
            prevalence: Q::new(1, 3),
            ..clinic_a()
        };
        assert!(thirds.counts().unwrap_err().contains("not a whole number"));
        let odd = Clinic {
            false_alarm: Q::new(1, 7),
            ..clinic_a()
        };
        assert!(odd.counts().unwrap_err().contains("false alarms"));
        let over = Clinic {
            sensitivity: Q::new(11, 10),
            ..clinic_a()
        };
        assert!(over.counts().unwrap_err().contains("not between 0 and 1"));
    }

    #[test]
    fn counting_people_gives_the_positive_predictive_value_and_bayes_agrees() {
        for (clinic, want) in [(clinic_a(), Q::new(2, 13)), (clinic_b(), Q::new(2, 3))] {
            let c = clinic.counts().unwrap();
            let array = IconArray::build(&c, 100).unwrap();
            let people = array.people();
            assert_eq!(people.len() as u64, c.people);
            assert_eq!(share(people, disease, positive).unwrap(), want);
            assert_eq!(
                share(people, disease, positive).unwrap(),
                Q::ratio(c.true_pos, c.positives())
            );
            assert_eq!(bayes(clinic.prevalence, clinic.sensitivity, clinic.false_alarm), want);
            // the array reproduces the clinic's own rates
            assert_eq!(share(people, positive, no_disease).unwrap(), clinic.false_alarm);
            assert_eq!(share(people, positive, disease).unwrap(), clinic.sensitivity);
            assert_eq!(share(people, disease, |_| true).unwrap(), clinic.prevalence);
        }
        // and the formula against counting over a sweep of whole-person clinics
        for prevalence in [Q::new(1, 100), Q::new(1, 20), Q::new(1, 10), Q::new(1, 4), Q::new(1, 2)] {
            for false_alarm in [Q::new(1, 100), Q::new(1, 20), Q::new(1, 10)] {
                let clinic = Clinic {
                    prevalence,
                    false_alarm,
                    ..clinic_a()
                };
                let c = clinic.counts().unwrap();
                assert_eq!(
                    Q::ratio(c.true_pos, c.positives()),
                    bayes(prevalence, clinic.sensitivity, false_alarm),
                    "{prevalence} {false_alarm}"
                );
            }
        }
    }

    /// The negative check for the conditional: the false-alarm rate and its
    /// reversal count different groups and differ, and the tempting
    /// "ninety-five percent" is the share of negatives among people without
    /// the disease, not the share of the disease among positives.
    #[test]
    fn swapping_the_condition_changes_the_number() {
        let c = clinic_a().counts().unwrap();
        let array = IconArray::build(&c, 100).unwrap();
        let people = array.people();
        let false_alarm = share(people, positive, no_disease).unwrap();
        let reversed = share(people, no_disease, positive).unwrap();
        assert_eq!((false_alarm, reversed), (Q::new(1, 20), Q::new(11, 13)));
        assert_ne!(false_alarm, reversed);
        let tempting = false_alarm.complement();
        assert_eq!(tempting, Q::new(19, 20));
        assert_eq!(tempting, share(people, negative, no_disease).unwrap());
        let ppv = share(people, disease, positive).unwrap();
        assert_ne!(tempting, ppv);
        // the reversal is the complement of the PPV, the false-alarm rate is not
        assert_eq!(reversed, ppv.complement());
        assert_ne!(false_alarm, ppv.complement());
        // the same test in another clinic: the false-alarm rate holds, the PPV moves
        let b = clinic_b().counts().unwrap();
        let b_people = IconArray::build(&b, 100).unwrap();
        assert_eq!(share(b_people.people(), positive, no_disease).unwrap(), false_alarm);
        assert_ne!(share(b_people.people(), disease, positive).unwrap(), ppv);
    }

    #[test]
    fn the_icon_array_is_one_row_of_disease_and_a_five_wide_stripe_of_false_alarms() {
        let c = clinic_a().counts().unwrap();
        let a = IconArray::build(&c, 100).unwrap();
        assert_eq!((a.rows, a.disease_rows, a.false_pos_per_row), (100, 1, 5));
        assert!(a.at(0, 89).disease && a.at(0, 89).positive);
        assert!(a.at(0, 90).disease && !a.at(0, 90).positive);
        assert!(!a.at(1, 4).disease && a.at(1, 4).positive);
        assert!(!a.at(99, 5).disease && !a.at(99, 5).positive);
        let count = |f: fn(Person) -> bool| a.people().iter().filter(|&&p| f(p)).count() as u64;
        assert_eq!(count(|p| p.disease && p.positive), c.true_pos);
        assert_eq!(count(|p| !p.disease && p.positive), c.false_pos);
        assert_eq!(count(|p| !p.disease && !p.positive), c.true_neg);
        assert!(IconArray::build(&c, 30).is_err());
        let uneven = Counts {
            false_pos: 496,
            true_neg: 9_404,
            ..c
        };
        assert!(IconArray::build(&uneven, 100).unwrap_err().contains("evenly"));
    }

    #[test]
    fn the_regrouped_positives_show_the_ppv_as_whole_rows() {
        let c = clinic_a().counts().unwrap();
        let block = positives_block(&c).unwrap();
        assert_eq!(
            block,
            PositivesBlock {
                cols: 45,
                rows: 13,
                disease_rows: 2
            }
        );
        assert_eq!(block.cols * block.rows, c.positives());
        assert_eq!(
            Q::ratio(block.disease_rows, block.rows),
            Q::ratio(c.true_pos, c.positives())
        );
        let none = Counts {
            true_pos: 0,
            false_pos: 0,
            ..c
        };
        assert!(positives_block(&none).is_err());
    }

    /// Chapter 02's claim: the test's own rates do not depend on the clinic,
    /// and the positive predictive value does. Counted over whole-person
    /// clinics at every prevalence the sweep can hold, independently of
    /// Bayes' rule: the two counted rates never move, the PPV only rises.
    #[test]
    fn the_same_test_keeps_its_rates_while_the_ppv_moves_with_the_clinic() {
        let mut last = None;
        for prevalence in [
            Q::ZERO,
            Q::new(1, 100),
            Q::new(1, 20),
            Q::new(1, 10),
            Q::new(1, 4),
            Q::new(1, 2),
            Q::new(9, 10),
        ] {
            let clinic = Clinic {
                prevalence,
                ..clinic_a()
            };
            let c = clinic.counts().unwrap();
            let people = IconArray::build(&c, 100).unwrap();
            let people = people.people();
            assert_eq!(
                share(people, positive, no_disease).unwrap(),
                Q::new(1, 20),
                "{prevalence}"
            );
            if prevalence > Q::ZERO {
                assert_eq!(share(people, positive, disease).unwrap(), Q::new(9, 10), "{prevalence}");
            }
            let ppv = share(people, disease, positive).unwrap();
            if let Some(before) = last {
                assert!(ppv > before, "the PPV rises with the prevalence: {before} then {ppv}");
            }
            last = Some(ppv);
        }
        // the two clinics of chapters 01 and 02: ten times the prevalence,
        // ten times the people caught, about a tenth fewer false alarms
        let (a, b) = (clinic_a().counts().unwrap(), clinic_b().counts().unwrap());
        assert_eq!(clinic_b().prevalence / clinic_a().prevalence, Q::int(10));
        assert_eq!(Q::ratio(b.true_pos, a.true_pos), Q::int(10));
        assert_eq!((a.false_pos, b.false_pos), (495, 450));
        assert_eq!(Q::ratio(a.true_pos, a.positives()), Q::new(2, 13));
        assert_eq!(Q::ratio(b.true_pos, b.positives()), Q::new(2, 3));
        // a clinic with no disease has no true positives: its PPV is zero, not undefined
        let none = Clinic {
            prevalence: Q::ZERO,
            ..clinic_a()
        };
        let n = none.counts().unwrap();
        assert_eq!((n.true_pos, n.false_pos), (0, 500));
        // and a test that never alarms leaves nothing to share among positives
        // when no one has the disease
        let silent = Clinic {
            prevalence: Q::ZERO,
            false_alarm: Q::ZERO,
            ..clinic_a()
        };
        let s = silent.counts().unwrap();
        let s_people = IconArray::build(&s, 100).unwrap();
        assert!(
            share(s_people.people(), disease, positive)
                .unwrap_err()
                .contains("empty")
        );
    }

    /// The swapped conditional in the second clinic: the false-alarm rate
    /// stays one in twenty while its reversal is one in three, and the
    /// tempting ninety-five percent is neither the PPV nor its complement.
    #[test]
    fn swapping_the_condition_in_the_second_clinic_changes_the_number_too() {
        let b = clinic_b().counts().unwrap();
        let array = IconArray::build(&b, 100).unwrap();
        let people = array.people();
        let false_alarm = share(people, positive, no_disease).unwrap();
        let reversed = share(people, no_disease, positive).unwrap();
        let ppv = share(people, disease, positive).unwrap();
        assert_eq!(
            (false_alarm, reversed, ppv),
            (Q::new(1, 20), Q::new(1, 3), Q::new(2, 3))
        );
        assert_ne!(false_alarm, reversed);
        assert_eq!(reversed, ppv.complement());
        let tempting = false_alarm.complement();
        assert!(tempting != ppv && tempting != reversed);
        // the reversal is not even the same direction of change: from clinic
        // A to B the false-alarm rate holds while its reversal falls from
        // 11/13 to 1/3
        let a = clinic_a().counts().unwrap();
        let a_people = IconArray::build(&a, 100).unwrap();
        assert_eq!(share(a_people.people(), no_disease, positive).unwrap(), Q::new(11, 13));
        assert!(reversed < Q::new(11, 13));
    }

    /// Whole people at the edges: the second clinic divides exactly, and the
    /// clinics a near miss would need are refused rather than rounded.
    #[test]
    fn the_second_clinic_counts_whole_people_and_near_misses_are_refused() {
        let b = clinic_b().counts().unwrap();
        assert_eq!(
            (b.with, b.without, b.true_pos, b.false_neg, b.false_pos, b.true_neg),
            (1_000, 9_000, 900, 100, 450, 8_550)
        );
        assert_eq!(b.positives(), 1_350);
        // one person more, and a tenth of them is not a whole number
        let odd = Clinic {
            people: 10_001,
            ..clinic_b()
        };
        assert!(odd.counts().unwrap_err().contains("not a whole number"));
        // a thousand people at one in a hundred: 990 without the disease,
        // and a twentieth of them is 49.5
        let small = Clinic {
            people: 1_000,
            ..clinic_a()
        };
        assert!(small.counts().unwrap_err().contains("false alarms"));
        // the icon array: ten disease rows, ninety rows of five false alarms
        let array = IconArray::build(&b, 100).unwrap();
        assert_eq!((array.disease_rows, array.false_pos_per_row), (10, 5));
        assert!(array.at(8, 99).positive && !array.at(9, 0).positive);
        // a clinic of only the disease has no rows for false alarms to fill
        let all = Clinic {
            prevalence: Q::ONE,
            ..clinic_a()
        };
        assert!(IconArray::build(&all.counts().unwrap(), 100).is_err());
    }

    /// Both blocks are 45 wide, and their gold rows are the PPV; a clinic
    /// whose positives cannot fill rows of 45 is refused.
    #[test]
    fn both_clinics_regroup_into_rows_of_forty_five() {
        let (a, b) = (clinic_a().counts().unwrap(), clinic_b().counts().unwrap());
        let block_a = positives_block_of(&a, 45).unwrap();
        let block_b = positives_block_of(&b, 45).unwrap();
        assert_eq!(block_a, positives_block(&a).unwrap(), "chapter 01's block is the same");
        assert_eq!((block_a.rows, block_a.disease_rows), (13, 2));
        assert_eq!((block_b.rows, block_b.disease_rows), (30, 20));
        for (block, c) in [(block_a, a), (block_b, b)] {
            assert_eq!(block.cols * block.rows, c.positives());
            assert_eq!(
                Q::ratio(block.disease_rows, block.rows),
                Q::ratio(c.true_pos, c.positives())
            );
        }
        let twentieth = Clinic {
            prevalence: Q::new(1, 20),
            ..clinic_a()
        };
        let t = twentieth.counts().unwrap();
        assert_eq!((t.true_pos, t.false_pos), (450, 475));
        assert!(positives_block_of(&t, 45).unwrap_err().contains("rows of 45"));
        assert!(positives_block_of(&a, 0).is_err());
    }

    #[test]
    fn the_null_world_lists_every_assignment_and_is_the_binomial() {
        let s = study();
        assert_eq!(s.observed(), 15);
        let world = s.null_world();
        assert_eq!(world.iter().sum::<u64>(), 1 << 20);
        for (k, &c) in world.iter().enumerate() {
            assert_eq!(u128::from(c), choose(20, k as u64), "k = {k}");
        }
        // the world does not depend on who recovered faster: only labels move
        let other = Study {
            faster_a: 0,
            treated_a: 0b11111,
            ..s
        };
        assert_eq!(other.null_world(), world);
        assert_eq!(other.observed(), 15);
        // small studies by brute force against the closed form
        for pairs in 1..=12u32 {
            let tiny = Study {
                pairs,
                faster_a: 0,
                treated_a: 0,
            };
            let w = tiny.null_world();
            for k in 0..=pairs {
                assert_eq!(u128::from(w[k as usize]), choose(u64::from(pairs), u64::from(k)));
            }
        }
    }

    #[test]
    fn the_p_value_counts_the_whole_tail_and_the_wrong_tails_differ() {
        let world = study().null_world();
        let at_least = tail_count(&world, 15, Tail::AtLeast);
        assert_eq!(at_least, 21_700);
        assert_eq!(tail_share(&world, 15, Tail::AtLeast).decimal(10), "0.0206947327");
        assert_eq!(tail_share(&world, 15, Tail::AtLeast).percent(2), "2.07");
        let exactly = tail_count(&world, 15, Tail::Exactly);
        let more = tail_count(&world, 15, Tail::MoreThan);
        assert_eq!((exactly, more), (15_504, 6_196));
        assert_eq!(at_least, exactly + more);
        assert!(exactly != at_least && more != at_least);
        let other_way = tail_count(&world, 15, Tail::AtMost);
        assert_eq!(other_way, (1 << 20) - more);
        assert_eq!(tail_count(&world, 15, Tail::TwoSided), 2 * at_least);
        assert_eq!(tail_count(&world, 14, Tail::AtLeast), 60_460);
        assert_eq!(tail_share(&world, 14, Tail::AtLeast).decimal(10), "0.0576591492");
        // the edges: the least favourable count has p = 1, the most has the
        // smallest p the study can give, and it is never zero
        assert_eq!(tail_share(&world, 0, Tail::AtLeast), Q::ONE);
        assert_eq!(tail_share(&world, 20, Tail::AtLeast), Q::ratio(1, 1 << 20));
        // the middle is two-sided p = 1
        assert_eq!(tail_share(&world, 10, Tail::TwoSided), Q::ONE);
    }

    /// Chapter 03's world does not depend on who recovered first: for every
    /// orientation of the finishing order in every small study, listing all
    /// assignments gives the binomial counts, each assignment counted once.
    #[test]
    fn every_finishing_order_gives_the_same_world_and_each_assignment_counts_once() {
        for pairs in 1..=8u32 {
            let all = (1u32 << pairs) - 1;
            for faster_a in 0..=all {
                let s = Study {
                    pairs,
                    faster_a,
                    treated_a: 0,
                };
                let world = s.null_world();
                for (k, &c) in world.iter().enumerate() {
                    assert_eq!(
                        u128::from(c),
                        choose(u64::from(pairs), k as u64),
                        "{pairs} {faster_a:b} {k}"
                    );
                }
                // no double counting: the assignments sorted by their count
                // are a partition of all 2^pairs assignments
                let mut seen = vec![false; 1 << pairs];
                for k in 0..=pairs {
                    for a in (0..=all).filter(|&a| s.favouring(a) == k) {
                        assert!(!seen[a as usize], "assignment {a:b} counted twice");
                        seen[a as usize] = true;
                    }
                }
                assert!(seen.iter().all(|&x| x));
                // the per-pair view agrees with the whole-assignment count
                for a in 0..=all {
                    let by_pair = (0..pairs).filter(|&i| s.favours(a, i)).count() as u32;
                    assert_eq!(by_pair, s.favouring(a));
                }
            }
        }
    }

    /// Swapping one pair's treatment label changes only that pair, and moves
    /// the count by exactly one; the finishing order never moves.
    #[test]
    fn swapping_one_pairs_labels_moves_the_count_by_one() {
        let s = study();
        for i in 0..s.pairs {
            let swapped = s.treated_a ^ (1 << i);
            let before = s.favouring(s.treated_a);
            let after = s.favouring(swapped);
            assert_eq!(before.abs_diff(after), 1, "pair {i}");
            assert_eq!(s.favours(swapped, i), !s.favours(s.treated_a, i));
            for j in (0..s.pairs).filter(|&j| j != i) {
                assert_eq!(s.favours(swapped, j), s.favours(s.treated_a, j));
            }
            assert_eq!(s.faster_is_a(i), s.faster_a >> i & 1 == 1);
        }
        // the five pairs that went the other way are exactly the non-favouring ones
        let against: Vec<u32> = (0..s.pairs).filter(|&i| !s.favours(s.treated_a, i)).collect();
        assert_eq!(against, vec![3, 7, 11, 16, 19]);
    }

    /// Chapter 04's tail, counted from the listed world: the bars 15 to 20
    /// hold 15,504 + 4,845 + 1,140 + 190 + 20 + 1 = 21,700 assignments (six
    /// bars, not six assignments); the endpoints are inclusive, the extremes
    /// and the mirror hold, and the wrong tails and off-by-one tails differ.
    #[test]
    fn the_tail_is_counted_in_assignments_with_its_endpoint_included() {
        let s = study();
        let world = s.null_world();
        let tail: Vec<u64> = world[15..].to_vec();
        assert_eq!(tail, vec![15_504, 4_845, 1_140, 190, 20, 1]);
        assert_eq!(tail.iter().sum::<u64>(), tail_count(&world, 15, Tail::AtLeast));
        assert_eq!(tail.len(), 6, "six bars");
        assert!(
            tail.iter().sum::<u64>() > tail.len() as u64 * 1_000,
            "assignments, not bars"
        );
        // the observed endpoint 15 is in the tail, not in the strict one
        assert_eq!(
            tail_count(&world, 15, Tail::AtLeast) - tail_count(&world, 15, Tail::MoreThan),
            world[15]
        );
        assert_eq!(tail_count(&world, 15, Tail::Exactly), world[15]);
        // the extremes
        assert_eq!(tail_count(&world, 0, Tail::AtLeast), 1 << 20);
        assert_eq!(tail_count(&world, 20, Tail::AtLeast), 1);
        assert_eq!(tail_count(&world, 0, Tail::AtMost), 1);
        assert_eq!(tail_count(&world, 20, Tail::MoreThan), 0);
        // the mirror: k and 20 - k hold the same number of assignments, so
        // the other direction's tail at 5 equals ours at 15
        for k in 0..=20 {
            assert_eq!(world[k], world[20 - k], "k = {k}");
        }
        assert_eq!(
            tail_count(&world, 5, Tail::AtMost),
            tail_count(&world, 15, Tail::AtLeast)
        );
        // mutations of the tail are different numbers
        let right = tail_count(&world, 15, Tail::AtLeast);
        for wrong in [
            tail_count(&world, 16, Tail::AtLeast),
            tail_count(&world, 14, Tail::AtLeast),
            tail_count(&world, 15, Tail::Exactly),
            tail_count(&world, 15, Tail::MoreThan),
            tail_count(&world, 15, Tail::AtMost),
            tail_count(&world, 15, Tail::TwoSided),
        ] {
            assert_ne!(wrong, right);
        }
        assert_eq!(tail_share(&world, 15, Tail::Exactly).percent(1), "1.5");
        assert_eq!(tail_share(&world, 15, Tail::MoreThan).percent(1), "0.6");
        assert_eq!(tail_share(&world, 15, Tail::AtLeast).percent(0), "2");
        assert_eq!(tail_share(&world, 15, Tail::AtLeast).decimal(4), "0.0207");
    }

    /// What chapter 04 lights for each tail is that tail's inequality: the
    /// tail in the predeclared direction from the observed 15 is bars 15 to
    /// 20, the exact one 15 alone, the strict one 16 to 20, and each set's
    /// assignments are its count.
    #[test]
    fn each_highlighted_set_is_its_inequality() {
        let world = study().null_world();
        assert_eq!(tail_members(20, 15, Tail::AtLeast), vec![15, 16, 17, 18, 19, 20]);
        assert_eq!(tail_members(20, 15, Tail::Exactly), vec![15]);
        assert_eq!(tail_members(20, 15, Tail::MoreThan), vec![16, 17, 18, 19, 20]);
        assert_eq!(tail_members(20, 15, Tail::AtMost).len(), 16);
        assert_eq!(
            tail_members(20, 15, Tail::TwoSided),
            vec![0, 1, 2, 3, 4, 5, 15, 16, 17, 18, 19, 20]
        );
        for tail in [
            Tail::AtLeast,
            Tail::Exactly,
            Tail::MoreThan,
            Tail::AtMost,
            Tail::TwoSided,
        ] {
            let lit: u64 = tail_members(20, 15, tail).iter().map(|&j| world[j as usize]).sum();
            assert_eq!(lit, tail_count(&world, 15, tail), "{tail:?}");
        }
        // exact and strict are disjoint and together make the whole tail
        let mut both = tail_members(20, 15, Tail::Exactly);
        both.extend(tail_members(20, 15, Tail::MoreThan));
        assert_eq!(both, tail_members(20, 15, Tail::AtLeast));
    }

    /// The counts chapter 04 earns with real assignments: exactly one gives
    /// all twenty, and the assignments giving nineteen are exactly the twenty
    /// that turn over one pair's coin.
    #[test]
    fn twenty_has_one_way_and_nineteen_has_one_per_pair() {
        let s = study();
        let all: Vec<u32> = (0..1u32 << s.pairs).filter(|&a| s.favouring(a) == 20).collect();
        assert_eq!(all, vec![s.favouring_all_but(&[])]);
        let mut nineteen: Vec<u32> = (0..1u32 << s.pairs).filter(|&a| s.favouring(a) == 19).collect();
        let mut one_over: Vec<u32> = (0..s.pairs).map(|j| s.favouring_all_but(&[j])).collect();
        nineteen.sort_unstable();
        one_over.sort_unstable();
        assert_eq!(nineteen, one_over);
        assert_eq!(nineteen.len(), 20);
        for j in 0..s.pairs {
            let a = s.favouring_all_but(&[j]);
            assert!(!s.favours(a, j) && (0..s.pairs).filter(|&i| i != j).all(|i| s.favours(a, i)));
        }
        assert_eq!(s.favouring(s.favouring_all_but(&[3, 7, 11, 16, 19])), 15);
        assert_eq!(s.favouring_all_but(&[3, 7, 11, 16, 19]), s.treated_a);
    }

    /// The chapter's re-tosses are real, distinct assignments of the same
    /// study, and planted mistakes give different numbers: counting the
    /// pairs whose patient A was treated is not the statistic, and flipping
    /// one coin of the observed assignment is not the observed count.
    #[test]
    fn the_retosses_are_distinct_and_planted_mistakes_are_caught() {
        let s = study();
        let tosses = s.retosses(2006, 4);
        assert_eq!(tosses.len(), 4);
        let mut all = tosses.clone();
        all.push(s.treated_a);
        all.sort_unstable();
        all.dedup();
        assert_eq!(all.len(), 5, "four new assignments, none the observed one");
        assert!(tosses.iter().all(|&a| a < 1 << s.pairs));
        assert_eq!(s.retosses(2006, 4), tosses, "the same seed gives the same tosses");
        // planted: the number of A patients treated is not the statistic
        let a_treated = s.treated_a.count_ones();
        assert_eq!((a_treated, s.observed()), (9, 15));
        assert_ne!(a_treated, s.observed());
        // planted: one wrong coin is a different count
        assert_ne!(s.favouring(s.treated_a ^ 1), s.observed());
        // planted: a world counted by the A-treated statistic is still a
        // binomial, but it puts the observed assignment in a different bar
        let wrong = Study {
            faster_a: (1 << s.pairs) - 1,
            ..s
        };
        assert_eq!(wrong.null_world(), s.null_world());
        assert_eq!(wrong.observed(), a_treated);
    }

    /// Chapter 05's two constructed versions keep the study's order in every
    /// pair (read from the days, not from a label), so both give the study's
    /// 15 and its tail; they differ only in how much sooner.
    #[test]
    fn the_constructed_days_keep_the_order_and_differ_in_size() {
        let s = study();
        let world = s.null_world();
        let [small, large] = constructed_days();
        for days in [&small, &large] {
            assert_eq!(days.pairs(), s.pairs as usize);
            assert_eq!(days.faster_a(s.treated_a), s.faster_a, "the study's order");
            assert_eq!(days.favouring(), s.observed());
            assert_eq!(tail_count(&world, days.favouring(), Tail::AtLeast), 21_700);
            assert!(
                days.treated()
                    .iter()
                    .chain(days.untreated())
                    .all(|&d| (1..=30).contains(&d))
            );
        }
        let signs = |d: &Days| d.differences().iter().map(|x| x.signum()).collect::<Vec<_>>();
        assert_eq!(signs(&small), signs(&large), "the same signs, pair by pair");
        assert_ne!(small.differences(), large.differences(), "different sizes");
        assert_eq!(small.median_sooner(), Some(Q::int(1)), "about a day");
        assert_eq!(large.median_sooner(), Some(Q::int(7)), "about a week");
        // any change of size that keeps every sign keeps the statistic
        for factor in 1..=4 {
            let scaled: Vec<i64> = small
                .differences()
                .iter()
                .map(|&d| if d > 0 { d * factor } else { d })
                .collect();
            let days = Days::from_differences(&UNTREATED_DAYS, &scaled).unwrap();
            assert_eq!(days.favouring(), small.favouring(), "factor {factor}");
            assert_eq!(days.faster_a(s.treated_a), s.faster_a);
        }
    }

    /// A changed sign changes the statistic and the tail: one favouring pair
    /// turned gives 14 and 60,460 assignments; one pair against turned gives 16.
    #[test]
    fn a_changed_sign_changes_the_statistic() {
        let s = study();
        let world = s.null_world();
        let [_, large] = constructed_days();
        let mut fewer = large.differences();
        fewer[0] = -fewer[0];
        let fewer = Days::from_differences(&UNTREATED_DAYS, &fewer).unwrap();
        assert_eq!(fewer.favouring(), 14);
        assert_ne!(fewer.faster_a(s.treated_a), s.faster_a);
        assert_eq!(tail_count(&world, fewer.favouring(), Tail::AtLeast), 60_460);
        let mut more = large.differences();
        more[3] = -more[3];
        let more = Days::from_differences(&UNTREATED_DAYS, &more).unwrap();
        assert_eq!(more.favouring(), 16);
    }

    /// What a recovery time cannot be is refused, and said: a tie, zero
    /// days, a difference larger than the untreated patient's days (up to the
    /// ends of i64), a pair missing its other patient, more pairs than a
    /// study's bits. Negative and non-finite days cannot be written at all
    /// (whole days, unsigned).
    #[test]
    fn days_that_are_not_recovery_times_are_refused() {
        let tie = Days::new(vec![5, 7], vec![6, 7]).unwrap_err();
        assert!(tie.contains("pair 1") && tie.contains("tie"), "{tie}");
        let zero = Days::new(vec![0, 7], vec![6, 8]).unwrap_err();
        assert!(zero.contains("at least one day"), "{zero}");
        assert!(Days::new(vec![5], vec![6, 7]).is_err());
        assert!(Days::new(vec![], vec![]).is_err());
        let too_big = Days::from_differences(&[10, 12], &[2, 13]).unwrap_err();
        assert!(too_big.contains("pair 1"), "{too_big}");
        assert!(Days::from_differences(&[10, 12], &[2, 12]).is_err(), "zero days");
        assert!(Days::from_differences(&[10, 12], &[2, 0]).is_err(), "a tie");
        assert!(Days::from_differences(&[10], &[2, 3]).is_err());
        // differences at the ends of i64 are refused, not overflowed
        let low = Days::from_differences(&[10, 12], &[2, i64::MIN]).unwrap_err();
        assert!(low.contains("pair 1"), "{low}");
        assert!(Days::from_differences(&[10, 12], &[i64::MAX, 2]).is_err());
        // no more pairs than a study's bits hold
        let n = MAX_PAIRS as usize;
        assert!(Days::new(vec![1; n], vec![2; n]).is_ok());
        let many = Days::new(vec![1; n + 1], vec![2; n + 1]).unwrap_err();
        assert!(many.contains("at most 30"), "{many}");
        assert_eq!(Days::from_differences(&[10, 12], &[2, -3]).unwrap().favouring(), 1);
        // no favouring pair, no median; an even number takes the middle two
        assert_eq!(Days::new(vec![9], vec![8]).unwrap().median_sooner(), None);
        assert_eq!(
            Days::new(vec![1, 1], vec![2, 5]).unwrap().median_sooner(),
            Some(Q::new(5, 2))
        );
    }

    /// Chapter 05's two results, each with its inclusive tail: 14 or more is
    /// 60,460 assignments (bar 14 included), 15 or more 21,700; one pair
    /// apart, they fall either side of a 5% line, which is a convention, not
    /// a property of either world (both are the same world).
    #[test]
    fn fourteen_and_fifteen_have_their_own_inclusive_tails() {
        let world = study().null_world();
        let (fourteen, fifteen) = (
            tail_count(&world, 14, Tail::AtLeast),
            tail_count(&world, 15, Tail::AtLeast),
        );
        assert_eq!((fourteen, fifteen), (60_460, 21_700));
        assert_eq!(fourteen - fifteen, world[14], "bar 14 is in its own tail");
        assert_eq!(tail_count(&world, 14, Tail::MoreThan), fifteen);
        let (p14, p15) = (
            tail_share(&world, 14, Tail::AtLeast),
            tail_share(&world, 15, Tail::AtLeast),
        );
        assert_eq!((p14.percent(2), p15.percent(2)), ("5.77".to_owned(), "2.07".to_owned()));
        assert_eq!(p14.percent(1), "5.8");
        let line = Q::new(5, 100);
        assert!(p15 < line && line < p14);
        // the off-by-one tails are other numbers
        assert_ne!(tail_count(&world, 13, Tail::AtLeast), fourteen);
        assert_ne!(tail_count(&world, 14, Tail::Exactly), fourteen);
    }
}
