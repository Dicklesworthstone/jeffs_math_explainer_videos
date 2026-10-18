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

// -------------------------------------------------- the study's null world

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
        assert!(self.pairs <= 30, "{} pairs is too many to list", self.pairs);
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
    /// k or more: as favourable as the observation, or more (the p-value
    /// of the predeclared one-sided test).
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

/// How many assignments of a null world the tail at `observed` counts.
pub fn tail_count(world: &[u64], observed: u32, tail: Tail) -> u64 {
    let n = world.len() as i64 - 1;
    let k = i64::from(observed);
    world
        .iter()
        .enumerate()
        .filter(|&(j, _)| {
            let j = j as i64;
            match tail {
                Tail::AtLeast => j >= k,
                Tail::Exactly => j == k,
                Tail::MoreThan => j > k,
                Tail::AtMost => j <= k,
                Tail::TwoSided => (2 * j - n).abs() >= (2 * k - n).abs(),
            }
        })
        .map(|(_, &c)| c)
        .sum()
}

/// The tail's share of every assignment.
pub fn tail_share(world: &[u64], observed: u32, tail: Tail) -> Q {
    Q::ratio(tail_count(world, observed, tail), world.iter().sum())
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
}
