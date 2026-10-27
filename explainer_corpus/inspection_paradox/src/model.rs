//! The video's one timetable and everything its pictures compute from it.
//!
//! Two routes stop at one stop, each every [`PERIOD`] minutes exactly, the
//! red bus [`OFFSET`] minutes after the blue. Passengers turn up at random:
//! a moment spread evenly over time, independent of the buses. Every wait
//! runs to the next bus of either route. All arithmetic is exact ([`Q`]).
//!
//! The model averages waits by the closed form the video derives, the sum
//! of g² over twice the sum of g. `oracle.py` never uses that form for the
//! example. It averages second by second over the hour, and the tests here
//! do the same at every offset.

use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Div, Mul, Sub};

// ------------------------------------------------------------- the example

/// The two routes: blue and red.
pub const ROUTES: usize = 2;
/// Each route's gap, in minutes.
pub const PERIOD: i64 = 12;
/// The red bus comes this many minutes after the blue (the video's
/// timetable).
pub const OFFSET: i64 = 2;
/// One hour on the clock, in minutes.
pub const HOUR: i64 = 60;
/// The even timetable (chapters 03, 07, 12): the red bus six after the blue.
pub const EVEN_OFFSET: i64 = 6;
/// Chapter 12's last timetable: the red bus four after the blue.
pub const PUZZLE_OFFSET: i64 = 4;
/// Chapter 11: how early someone who reads the timetable turns up.
pub const EARLY: i64 = 1;
/// One passenger in the middle of every minute (the dots).
pub const PER_MINUTE: i64 = 1;
/// Chapter 09: three buses at once every eighteen minutes, more spread than
/// completely random buses at the same average gap.
pub const CLUMPED_GAPS: [i64; 3] = [0, 0, 18];
/// Chapter 09: completely random buses, one every this many minutes on
/// average.
pub const RANDOM_MEAN: i64 = 6;
/// Chapter 10: two classes, by size.
pub const CLASS_SIZES: [i64; 2] = [2, 10];
/// Chapter 10, for programmers: one long call and several short ones (ms).
pub const LONG_CALL_MS: i64 = 10;
pub const SHORT_CALL_MS: i64 = 2;
pub const SHORT_CALLS: i64 = 5;
/// Chapter 08's finite check of the bound: every multiset of up to
/// `SWEEP_MAX_COUNT` gaps, each a whole number of minutes from 0 to
/// `SWEEP_MAX_GAP`, not all zero.
pub const SWEEP_MAX_GAP: i64 = 12;
pub const SWEEP_MAX_COUNT: usize = 4;

// ------------------------------------------------------------ exact numbers

/// An exact rational, always in lowest terms with a positive denominator.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Q {
    num: i64,
    den: i64,
}

fn gcd(mut a: i128, mut b: i128) -> i128 {
    a = a.abs();
    b = b.abs();
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

impl Q {
    /// `num / den` in lowest terms. Panics on a zero denominator (a broken
    /// invariant, not data).
    pub fn new(num: i64, den: i64) -> Self {
        Self::reduce(i128::from(num), i128::from(den))
    }

    fn reduce(num: i128, den: i128) -> Self {
        assert!(den != 0, "a fraction with a zero denominator");
        let g = gcd(num, den).max(1);
        let sign = if den < 0 { -1 } else { 1 };
        let n = sign * num / g;
        let d = sign * den / g;
        Q {
            num: i64::try_from(n).expect("numerator fits"),
            den: i64::try_from(d).expect("denominator fits"),
        }
    }

    pub fn int(n: i64) -> Self {
        Q { num: n, den: 1 }
    }

    pub fn num(self) -> i64 {
        self.num
    }

    pub fn den(self) -> i64 {
        self.den
    }

    pub fn is_integer(self) -> bool {
        self.den == 1
    }

    pub fn to_f64(self) -> f64 {
        self.num as f64 / self.den as f64
    }
}

impl Add for Q {
    type Output = Q;
    fn add(self, o: Q) -> Q {
        let (a, b, c, d) = (
            i128::from(self.num),
            i128::from(self.den),
            i128::from(o.num),
            i128::from(o.den),
        );
        Q::reduce(a * d + c * b, b * d)
    }
}

impl Sub for Q {
    type Output = Q;
    fn sub(self, o: Q) -> Q {
        self + Q {
            num: -o.num,
            den: o.den,
        }
    }
}

impl Mul for Q {
    type Output = Q;
    fn mul(self, o: Q) -> Q {
        Q::reduce(
            i128::from(self.num) * i128::from(o.num),
            i128::from(self.den) * i128::from(o.den),
        )
    }
}

impl Div for Q {
    type Output = Q;
    fn div(self, o: Q) -> Q {
        Q::reduce(
            i128::from(self.num) * i128::from(o.den),
            i128::from(self.den) * i128::from(o.num),
        )
    }
}

impl Ord for Q {
    fn cmp(&self, o: &Q) -> Ordering {
        (i128::from(self.num) * i128::from(o.den)).cmp(&(i128::from(o.num) * i128::from(self.den)))
    }
}

impl PartialOrd for Q {
    fn partial_cmp(&self, o: &Q) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

impl std::iter::Sum for Q {
    fn sum<I: Iterator<Item = Q>>(it: I) -> Q {
        it.fold(Q::int(0), |a, b| a + b)
    }
}

/// "6", "13/3": whole numbers bare, fractions in lowest terms, as `oracle.py`
/// prints them (a negative, never a fact here, would take an ASCII minus).
impl fmt::Display for Q {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.den == 1 {
            write!(f, "{}", self.num)
        } else {
            write!(f, "{}/{}", self.num, self.den)
        }
    }
}

/// Minutes as the screen and the voice give them, "4:20". Refused unless
/// the duration is a whole number of seconds (no rounding anywhere).
pub fn clock(minutes: Q) -> Result<String, String> {
    let s = minutes * Q::int(60);
    if !s.is_integer() || s.num() < 0 {
        return Err(format!("{minutes} min is not a whole, non-negative number of seconds"));
    }
    let s = s.num();
    Ok(format!("{}:{:02}", s / 60, s % 60))
}

/// Minutes rounded to the nearest whole second, halves up, then shown by
/// [`clock`]: "4:01" for 4 min 1.25 s. The video's one rounding, only for
/// a readout that moves faster than it can be read (07_5's slider between
/// whole minutes). Every number the voice says or a card holds goes through
/// [`clock`] itself, which still refuses anything but whole seconds.
pub fn clock_rounded(minutes: Q) -> Result<String, String> {
    let s = minutes * Q::int(60);
    if s < Q::int(0) {
        return Err(format!("{minutes} min is negative"));
    }
    let up = s + Q::new(1, 2);
    clock(Q::new(up.num().div_euclid(up.den()), 60))
}

// ---------------------------------------------------------------- timetable

/// A route's bus times in the hour, in minutes: `start`, `start + PERIOD`, ….
pub fn route(start: i64) -> Vec<i64> {
    (0..HOUR).filter(|t| (t - start).rem_euclid(PERIOD) == 0).collect()
}

/// Both routes' buses in the hour, in order (two buses at once appear twice).
pub fn bus_times(offset: i64) -> Vec<i64> {
    let mut t = route(0);
    t.extend(route(offset));
    t.sort_unstable();
    t
}

/// The hour's buses with the previous and next hours' (the timetable
/// repeats), so every moment of the hour has a bus before and after it.
fn extended(offset: i64) -> Vec<i64> {
    let t = bus_times(offset);
    let mut all: Vec<i64> = t.iter().map(|x| x - HOUR).collect();
    all.extend(&t);
    all.extend(t.iter().map(|x| x + HOUR));
    all
}

/// The gap after each bus of the hour, ending at the next bus (the next
/// hour's first bus closes the last gap).
pub fn hour_gaps(offset: i64) -> Vec<i64> {
    let mut t = bus_times(offset);
    t.push(HOUR);
    t.windows(2).map(|w| w[1] - w[0]).collect()
}

/// The tag on each bus (chapter 02): how long since the bus before it, the
/// last earlier bus time (the bus at :00 follows the previous hour's last).
pub fn gap_before_each_bus(offset: i64) -> Vec<i64> {
    let all = extended(offset);
    bus_times(offset)
        .iter()
        .map(|&x| x - all.iter().copied().filter(|&y| y < x).max().expect("an earlier bus"))
        .collect()
}

/// The bus times around moment `t`: the last at or before it and the first
/// after it.
fn around(offset: i64, t: Q) -> (Q, Q) {
    let all = extended(offset);
    let before = all
        .iter()
        .map(|&y| Q::int(y))
        .filter(|&y| y <= t)
        .max()
        .expect("a bus before");
    let after = all
        .iter()
        .map(|&y| Q::int(y))
        .filter(|&y| y > t)
        .min()
        .expect("a bus after");
    (before, after)
}

/// The wait of someone who turns up at moment `t` (minutes past the hour).
pub fn wait_at(offset: i64, t: Q) -> Q {
    around(offset, t).1 - t
}

/// How long since the last bus, for someone who turns up at `t`.
pub fn since_last(offset: i64, t: Q) -> Q {
    t - around(offset, t).0
}

/// The length of the gap moment `t` falls in.
pub fn gap_at(offset: i64, t: Q) -> i64 {
    let (a, b) = around(offset, t);
    (b - a).num()
}

/// Your unlucky morning (chapter 01): half a minute after the red bus, the
/// long gap ahead.
pub fn unlucky_morning() -> Q {
    Q::new(2 * OFFSET + 1, 2)
}

/// Your lucky morning: half a minute before the blue bus at :12.
pub fn lucky_morning() -> Q {
    Q::new(2 * PERIOD - 1, 2)
}

/// The passengers of the hour: one in the middle of every minute.
pub fn passengers() -> Vec<Q> {
    (0..HOUR * PER_MINUTE)
        .map(|k| Q::new(2 * k + 1, 2 * PER_MINUTE))
        .collect()
}

// ------------------------------------------------------------ the averages

fn total(gaps: &[i64]) -> i64 {
    gaps.iter().sum()
}

fn squares(gaps: &[i64]) -> i64 {
    gaps.iter().map(|g| g * g).sum()
}

/// Averaged over buses: the sign's number.
pub fn mean_gap(gaps: &[i64]) -> Q {
    Q::new(total(gaps), gaps.len() as i64)
}

/// The waiting a gap holds: its triangle, half its length squared.
pub fn triangle_area(g: i64) -> Q {
    Q::new(g * g, 2)
}

/// The average wait over moments: the triangles' area over the time they
/// cover, sum of g² over twice the sum of g.
pub fn mean_wait(gaps: &[i64]) -> Q {
    gaps.iter().map(|&g| triangle_area(g)).sum::<Q>() / Q::int(total(gaps))
}

/// The average time since the last bus, over moments: the mirror image of
/// each gap's triangle, so the same area over the same time.
pub fn mean_since_last(gaps: &[i64]) -> Q {
    gaps.iter().map(|&g| Q::new(g * g, 2)).sum::<Q>() / Q::int(total(gaps))
}

/// Averaged over passengers (moments): each gap counted for every minute it
/// lasts, sum of g² over the sum of g.
pub fn passengers_gap(gaps: &[i64]) -> Q {
    Q::new(squares(gaps), total(gaps))
}

/// The average of the squared gaps, over buses.
pub fn mean_square(gaps: &[i64]) -> Q {
    Q::new(squares(gaps), gaps.len() as i64)
}

/// The spread: the average squared distance of the gaps from their average.
pub fn spread(gaps: &[i64]) -> Q {
    let m = mean_gap(gaps);
    let n = Q::int(gaps.len() as i64);
    gaps.iter().map(|&g| (Q::int(g) - m) * (Q::int(g) - m)).sum::<Q>() / n
}

/// The share of the time (and of the passengers) in gaps for which `keep`
/// holds.
pub fn time_share(gaps: &[i64], keep: impl Fn(i64) -> bool) -> Q {
    Q::new(gaps.iter().copied().filter(|&g| keep(g)).sum(), total(gaps))
}

/// The waits of the passengers in one gap of `g` minutes, one in the middle
/// of each minute, first to last: g − ½, …, ½.
pub fn midpoint_waits(g: i64) -> Vec<Q> {
    (0..g).map(|k| Q::new(2 * (g - k) - 1, 2)).collect()
}

/// The average wait with the red bus `d` minutes after the blue.
pub fn offset_wait(d: i64) -> Q {
    mean_wait(&hour_gaps(d))
}

/// The same for any offset `d` from 0 to [`PERIOD`] minutes, whole or not
/// (07_5's slider between whole minutes): each twelve minutes hold the gaps
/// `d` and `PERIOD − d`, and their triangles over the twelve minutes are
/// the average wait.
pub fn offset_wait_rational(d: Q) -> Result<Q, String> {
    if d < Q::int(0) || d > Q::int(PERIOD) {
        return Err(format!("offset {d} min is outside 0 to {PERIOD}"));
    }
    let gaps = [d, Q::int(PERIOD) - d];
    Ok(gaps.iter().map(|&g| g * g / Q::int(2)).sum::<Q>() / Q::int(PERIOD))
}

/// The longest wait, for someone who turns up just as a bus leaves: the
/// whole of the longest gap ahead. The allowance a deadline needs (your
/// train, 12_5), against the average wait, the cost of a usual morning.
pub fn longest_wait(offset: i64) -> i64 {
    *hour_gaps(offset).iter().max().expect("gaps")
}

/// Chapter 03: the share of the hour's passengers standing in a gap from a
/// red bus to the next blue bus, with the red bus `d` minutes after the
/// blue (at `d` = 0 that gap is the whole twelve minutes).
pub fn share_after_red(d: i64) -> Q {
    let ps = passengers();
    // a moment's place in its twelve minutes: t minus the whole periods before it
    let within = |t: Q| t - Q::int(t.num().div_euclid(t.den() * PERIOD) * PERIOD);
    let n = ps.iter().filter(|&&t| within(t) >= Q::int(d)).count();
    Q::new(n as i64, ps.len() as i64)
}

/// Copies of a small right triangle that tile one `n` times as large:
/// upright and upside down.
pub fn tiling(n: u64) -> (u64, u64) {
    (n * (n + 1) / 2, n * (n.max(1) - 1) / 2)
}

// -------------------------------------------------- completely random buses

/// The first two moments of an exponential gap with mean `mu`: mu and 2 mu².
pub fn exponential_moments(mu: i64) -> (Q, Q) {
    (Q::int(mu), Q::int(2 * mu * mu))
}

// ------------------------------------------------------------- the planners

/// Chapter 11: the same passengers, each now reading the timetable. A
/// passenger's dot is the moment they're ready. They plan for the first bus
/// they can make with [`EARLY`] minutes to spare, the first at or after
/// their dot plus [`EARLY`], and stand at the stop [`EARLY`] minutes before
/// it, so nobody turns up before they're ready. Returns when each turns up,
/// in the dots' order ([`passengers`]); the last can run into the next
/// hour. Nobody is shared out over the buses: who stands where follows from
/// the dots and the timetable alone.
pub fn planned_arrivals(offset: i64) -> Vec<Q> {
    passengers()
        .into_iter()
        .map(|ready| {
            let spare = ready + Q::int(EARLY);
            let (before, after) = around(offset, spare);
            let bus = if before == spare { before } else { after };
            bus - Q::int(EARLY)
        })
        .collect()
}

// ---------------------------------------------------------- the bound sweep

/// Chapter 08's finite check: how many choices of gaps (multisets: repeats
/// allowed, order ignored), how many wait less than half their average
/// gap, how many exactly half, and whether exactly the choices with every
/// gap the same are equal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sweep {
    pub cases: u64,
    pub violations: u64,
    pub equalities: u64,
    pub equal_iff_even: bool,
}

pub fn bound_sweep(max_gap: i64, max_count: usize) -> Sweep {
    fn walk(from: i64, max_gap: i64, left: usize, gaps: &mut Vec<i64>, out: &mut Sweep) {
        if !gaps.is_empty() && total(gaps) > 0 {
            out.cases += 1;
            let wait = mean_wait(gaps);
            let half = mean_gap(gaps) / Q::int(2);
            let even = gaps.iter().all(|&g| g == gaps[0]);
            if wait < half {
                out.violations += 1;
            }
            if wait == half {
                out.equalities += 1;
            }
            if (wait == half) != even {
                out.equal_iff_even = false;
            }
        }
        if left == 0 {
            return;
        }
        for g in from..=max_gap {
            gaps.push(g);
            walk(g, max_gap, left - 1, gaps, out);
            gaps.pop();
        }
    }
    let mut out = Sweep {
        cases: 0,
        violations: 0,
        equalities: 0,
        equal_iff_even: true,
    };
    walk(0, max_gap, max_count, &mut Vec::new(), &mut out);
    out
}

// --------------------------------------------------------------------- tests

#[cfg(test)]
mod tests {
    use super::*;

    /// The midpoint of every second of the hour, as `oracle.py` averages.
    fn by_seconds(offset: i64, f: impl Fn(i64, Q) -> Q) -> Q {
        let n = HOUR * 60;
        (0..n).map(|s| f(offset, Q::new(2 * s + 1, 120))).sum::<Q>() / Q::int(n)
    }

    #[test]
    fn fractions_stay_in_lowest_terms() {
        assert_eq!(Q::new(52, 12), Q::new(13, 3));
        assert_eq!(Q::new(3, -6), Q::new(-1, 2));
        assert_eq!(Q::new(52, 12).to_string(), "13/3");
        assert_eq!(Q::new(12, 2).to_string(), "6");
        assert!(Q::new(1, 3) < Q::new(1, 2));
        assert_eq!(Q::new(1, 2) + Q::new(1, 3), Q::new(5, 6));
        assert_eq!(Q::new(5, 6) - Q::new(1, 3), Q::new(1, 2));
    }

    #[test]
    fn the_clock_refuses_anything_but_whole_seconds() {
        assert_eq!(clock(Q::new(13, 3)).unwrap(), "4:20");
        assert_eq!(clock(Q::new(1, 2)).unwrap(), "0:30");
        assert_eq!(clock(Q::int(6)).unwrap(), "6:00");
        assert!(clock(Q::new(1, 7)).is_err());
        assert!(clock(Q::int(-1)).is_err());
    }

    #[test]
    fn the_timetable_is_the_video_s() {
        assert_eq!(route(0), vec![0, 12, 24, 36, 48]);
        assert_eq!(route(OFFSET), vec![2, 14, 26, 38, 50]);
        assert_eq!(hour_gaps(OFFSET), vec![2, 10, 2, 10, 2, 10, 2, 10, 2, 10]);
        assert_eq!(gap_before_each_bus(OFFSET), vec![10, 2, 10, 2, 10, 2, 10, 2, 10, 2]);
        assert_eq!(mean_gap(&hour_gaps(OFFSET)), Q::int(6));
    }

    #[test]
    fn the_example_waits_four_minutes_twenty() {
        let g = hour_gaps(OFFSET);
        assert_eq!(mean_wait(&g), Q::new(13, 3));
        assert_eq!(passengers_gap(&g), Q::new(26, 3));
        assert_eq!(mean_square(&g), Q::int(52));
        assert_eq!(spread(&g), Q::int(16));
        assert_eq!(time_share(&g, |x| x == 10), Q::new(5, 6));
        assert_eq!(
            mean_gap(&g) / Q::int(2) + spread(&g) / (Q::int(2) * mean_gap(&g)),
            Q::new(13, 3)
        );
    }

    /// Your two mornings (chapter 01): the lucky one waits half a minute,
    /// the unlucky one nine and a half, both in the same twelve minutes and
    /// both for the blue bus at :12, so their bars end at one marker.
    #[test]
    fn your_two_mornings_catch_the_same_bus() {
        let (lucky, unlucky) = (lucky_morning(), unlucky_morning());
        assert_eq!(wait_at(OFFSET, lucky), Q::new(1, 2));
        assert_eq!(wait_at(OFFSET, unlucky), Q::new(19, 2));
        assert_eq!(lucky + wait_at(OFFSET, lucky), Q::int(PERIOD));
        assert_eq!(unlucky + wait_at(OFFSET, unlucky), Q::int(PERIOD));
        assert_eq!(since_last(OFFSET, unlucky), Q::new(1, 2), "just after the red bus");
        assert_eq!((gap_at(OFFSET, lucky), gap_at(OFFSET, unlucky)), (10, 10));
    }

    /// Your train (12_5): no moment waits longer than the longest gap, and
    /// moments just after a red bus come within a second of it; the average,
    /// 4:20, is well under it.
    #[test]
    fn the_longest_wait_is_the_long_gap() {
        let longest = Q::int(longest_wait(OFFSET));
        assert_eq!(longest, Q::int(10));
        let seconds: Vec<Q> = (0..HOUR * 60)
            .map(|s| wait_at(OFFSET, Q::new(2 * s + 1, 120)))
            .collect();
        assert!(seconds.iter().all(|&w| w < longest));
        assert!(seconds.iter().any(|&w| longest - w < Q::new(1, 60)));
        assert!(mean_wait(&hour_gaps(OFFSET)) < longest);
    }

    /// The closed form against the second-by-second average, at every
    /// offset of the red bus, and the time since the last bus likewise.
    #[test]
    fn the_closed_form_is_the_average_over_moments() {
        for d in 0..=PERIOD {
            assert_eq!(by_seconds(d, wait_at), offset_wait(d), "offset {d}");
            assert_eq!(
                offset_wait(d),
                Q::new(d * d + (PERIOD - d) * (PERIOD - d), 2 * PERIOD),
                "offset {d}"
            );
        }
        assert_eq!(by_seconds(OFFSET, since_last), mean_since_last(&hour_gaps(OFFSET)));
        assert_eq!(
            by_seconds(OFFSET, |d, t| Q::int(gap_at(d, t))),
            passengers_gap(&hour_gaps(OFFSET))
        );
    }

    /// Negative control: people at the start of each minute are not a
    /// random moment (two of them stand at a bus as it leaves).
    #[test]
    fn a_grid_of_minute_starts_is_not_a_random_moment() {
        let starts: Q = (0..PERIOD).map(|k| wait_at(OFFSET, Q::int(k))).sum::<Q>() / Q::int(PERIOD);
        assert_ne!(starts, Q::new(13, 3));
        let mids: Q = (0..PERIOD).map(|k| wait_at(OFFSET, Q::new(2 * k + 1, 2))).sum::<Q>() / Q::int(PERIOD);
        assert_eq!(mids, Q::new(13, 3));
    }

    #[test]
    fn the_middle_of_each_minute_holds_exactly_the_triangle() {
        for g in 1..=SWEEP_MAX_GAP {
            assert_eq!(midpoint_waits(g).into_iter().sum::<Q>(), triangle_area(g), "gap {g}");
        }
        assert_eq!(midpoint_waits(2), vec![Q::new(3, 2), Q::new(1, 2)]);
    }

    #[test]
    fn even_gaps_wait_half_and_the_buses_together_wait_six() {
        assert_eq!(offset_wait(EVEN_OFFSET), Q::int(3));
        assert_eq!(passengers_gap(&hour_gaps(EVEN_OFFSET)), Q::int(6));
        let together = hour_gaps(0);
        assert!(together.contains(&0));
        assert_eq!(mean_gap(&together), Q::int(6));
        assert_eq!(offset_wait(0), Q::int(6));
    }

    #[test]
    fn stretching_one_gap_moves_only_the_passengers() {
        for d in 0..=EVEN_OFFSET {
            assert_eq!(share_after_red(d), Q::new(PERIOD - d, PERIOD), "offset {d}");
            assert_eq!(bus_times(d).len(), 10);
            assert_eq!(mean_gap(&hour_gaps(d)), Q::int(6));
        }
        assert_eq!(share_after_red(OFFSET), time_share(&hour_gaps(OFFSET), |g| g == 10));
    }

    #[test]
    fn the_tiling_and_the_square_law() {
        assert_eq!(tiling(5), (15, 10));
        for n in 1..=12 {
            let (up, down) = tiling(n);
            assert_eq!(up + down, n * n);
        }
        assert_eq!(triangle_area(10) / triangle_area(5), Q::int(4));
    }

    #[test]
    fn the_bound_holds_on_every_small_set_and_is_tight_only_when_even() {
        let s = bound_sweep(SWEEP_MAX_GAP, SWEEP_MAX_COUNT);
        assert_eq!(s.cases, 2375);
        assert_eq!(s.violations, 0);
        assert_eq!(s.equalities, 48);
        assert!(s.equal_iff_even);
        assert!(mean_wait(&[1, 2]) > mean_gap(&[1, 2]) / Q::int(2));
    }

    #[test]
    fn random_and_clumped_buses() {
        let (m1, m2) = exponential_moments(RANDOM_MEAN);
        assert_eq!(m2 / (Q::int(2) * m1), Q::int(6));
        assert_eq!(m2 - m1 * m1, Q::int(36));
        assert_eq!(mean_gap(&CLUMPED_GAPS), Q::int(6));
        assert_eq!(mean_wait(&CLUMPED_GAPS), Q::int(9));
    }

    /// Chapter 11's planners, the same sixty dots: nobody turns up before
    /// they're ready, every wait is a minute, ten stand before each blue bus
    /// and two before each red, and five in six still stand in a long gap,
    /// the time share. Negative control: the same share of long gaps, yet
    /// not the triangles' average wait, because nobody is spread through a
    /// gap any more.
    #[test]
    fn planners_wait_a_minute_and_five_in_six_still_stand_in_a_long_gap() {
        let ready = passengers();
        let planned = planned_arrivals(OFFSET);
        assert_eq!(planned.len(), ready.len());
        for (&at, &r) in planned.iter().zip(&ready) {
            assert!(at >= r, "ready at {r}, at the stop at {at}");
            assert_eq!(wait_at(OFFSET, at), Q::int(EARLY), "ready at {r}");
        }
        // ready half a minute before a bus is too late to be a minute early
        // for it: the passenger stands for the next one
        assert_eq!((planned[1], planned[11]), (Q::int(11), Q::int(13)));
        for b in bus_times(OFFSET) {
            let n = planned
                .iter()
                .filter(|&&at| (at + Q::int(EARLY)).num().rem_euclid(HOUR) == b)
                .count();
            assert_eq!(n, if b % PERIOD == 0 { 10 } else { 2 }, "the bus at :{b:02}");
        }
        let long = PERIOD - OFFSET;
        let in_long = planned.iter().filter(|&&at| gap_at(OFFSET, at) == long).count();
        let share = Q::new(in_long as i64, planned.len() as i64);
        assert_eq!(share, time_share(&hour_gaps(OFFSET), |g| g == long));
        assert_eq!(share, Q::new(5, 6));
        assert_ne!(Q::int(EARLY), mean_wait(&hour_gaps(OFFSET)));
    }

    /// 07_5's slider between whole minutes: the rational offset agrees with
    /// the timetable at every whole minute and refuses offsets outside the
    /// twelve; the rounded readout agrees with the exact clock on whole
    /// seconds, rounds halves up, stays within half a second at every tenth
    /// of a minute, and only 13 of those 121 offsets are whole seconds.
    #[test]
    fn the_slider_between_whole_minutes_rounds_only_its_readout() {
        for d in 0..=PERIOD {
            let w = offset_wait_rational(Q::int(d)).unwrap();
            assert_eq!(w, offset_wait(d), "offset {d}");
            assert_eq!(clock_rounded(w).unwrap(), clock(w).unwrap(), "offset {d}");
        }
        assert!(offset_wait_rational(Q::new(-1, 10)).is_err());
        assert!(offset_wait_rational(Q::new(121, 10)).is_err());
        let between = offset_wait_rational(Q::new(5, 2)).unwrap();
        assert_eq!(between * Q::int(60), Q::new(965, 4), "241.25 s");
        assert!(clock(between).is_err(), "the exact clock still refuses");
        assert_eq!(clock_rounded(between).unwrap(), "4:01");
        assert_eq!(clock_rounded(Q::new(483, 120)).unwrap(), "4:02", "241.5 s, half up");
        assert_eq!(clock_rounded(Q::new(119, 120)).unwrap(), "1:00", "59.5 s carries");
        assert_eq!(clock_rounded(Q::int(0)).unwrap(), "0:00");
        assert!(clock_rounded(Q::new(-1, 60)).is_err());
        let mut whole = 0;
        for k in 0..=10 * PERIOD {
            let s = offset_wait_rational(Q::new(k, 10)).unwrap() * Q::int(60);
            let shown = clock_rounded(s / Q::int(60)).unwrap();
            let (m, sec) = shown.split_once(':').unwrap();
            let back = Q::int(m.parse::<i64>().unwrap() * 60 + sec.parse::<i64>().unwrap());
            let off = if back > s { back - s } else { s - back };
            assert!(off <= Q::new(1, 2), "d = {k}/10: {shown} for {s} s");
            whole += i64::from(s.is_integer());
        }
        assert_eq!(whole, 13);
    }

    #[test]
    fn the_last_timetable() {
        assert_eq!(hour_gaps(PUZZLE_OFFSET)[..2], [4, 8]);
        assert_eq!(triangle_area(4) + triangle_area(8), Q::int(40));
        assert_eq!(offset_wait(PUZZLE_OFFSET), Q::new(10, 3));
        assert_eq!(clock(offset_wait(PUZZLE_OFFSET)).unwrap(), "3:20");
    }
}
