//! The arithmetic of Video F as pure functions: each trick worked the way
//! the video works it in the head (its parts, its pieces, its running
//! total), the areas its picture shows, the paper method's rows and carries,
//! and the quizzes. `facts.rs` prints them; `oracle.py` recomputes each one
//! by plain arithmetic.

// ---------------------------------------------------------------------------
// numbers and their parts
// ---------------------------------------------------------------------------

/// A number broken into its parts: everything above the ones, and the ones
/// (23 is 20 and 3; 105 is 100 and 5).
pub fn parts(x: u64) -> (u64, u64) {
    (x - x % 10, x % 10)
}

/// A round number as its leading number and how many zeros follow it (20 is
/// 2 and one zero; 800 is 8 and two). Zero has no zeros to count.
pub fn lead_and_zeros(x: u64) -> (u64, u32) {
    let (mut lead, mut zeros) = (x, 0);
    while lead != 0 && lead.is_multiple_of(10) {
        lead /= 10;
        zeros += 1;
    }
    (lead, zeros)
}

/// Cents as dollars with two decimals ("79.96").
pub fn money(cents: u64) -> String {
    format!("{}.{:02}", cents / 100, cents % 100)
}

// ---------------------------------------------------------------------------
// the paper method, held in the head (the hook)
// ---------------------------------------------------------------------------

/// Long multiplication as school writes it: one row per digit of the
/// multiplier, from the ones up, then the rows added.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paper {
    /// The rows at their true size (the second row of 85 × 85 is 6800).
    pub rows: Vec<u64>,
    /// How many times a carried digit has to be held for the next step: in
    /// the digit products of each row, and in adding the rows.
    pub carries: u32,
    pub product: u64,
}

fn digits(mut x: u64) -> Vec<u64> {
    let mut d = Vec::new();
    loop {
        d.push(x % 10);
        x /= 10;
        if x == 0 {
            return d;
        }
    }
}

pub fn paper(a: u64, b: u64) -> Paper {
    let (top, mut rows, mut carries) = (digits(a), Vec::new(), 0);
    let mut scale = 1;
    for m in digits(b) {
        let (mut row, mut carry, mut place) = (0, 0, 1);
        for (i, &t) in top.iter().enumerate() {
            let p = t * m + carry;
            row += (p % 10) * place;
            carry = p / 10;
            place *= 10;
            // the last carry is simply written down; the others are held
            if carry > 0 && i + 1 < top.len() {
                carries += 1;
            }
        }
        rows.push((row + carry * place) * scale);
        scale *= 10;
    }
    // adding the rows, column by column
    let (mut carry, mut place, product) = (0, 1, rows.iter().sum::<u64>());
    while rows.iter().any(|&r| r / place > 0) {
        let column: u64 = rows.iter().map(|&r| r / place % 10).sum::<u64>() + carry;
        carry = column / 10;
        place *= 10;
        if carry > 0 && rows.iter().any(|&r| r / place > 0) {
            carries += 1;
        }
    }
    Paper { rows, carries, product }
}

// ---------------------------------------------------------------------------
// trick 1: squares of numbers that end in 5
// ---------------------------------------------------------------------------

/// The square of a number ending in 5, by the trick: the number in front,
/// times the next number, then 25 written after it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EndsInFive {
    pub number: u64,
    /// The number in front of the 5 (the owner's n).
    pub n: u64,
    /// n times the next number: how many hundreds.
    pub hundreds: u64,
    pub square: u64,
}

/// `None` unless the number ends in 5.
pub fn ends_in_five(number: u64) -> Option<EndsInFive> {
    (number % 10 == 5).then(|| {
        let n = number / 10;
        let hundreds = n * (n + 1);
        EndsInFive {
            number,
            n,
            hundreds,
            square: hundreds * 100 + 25,
        }
    })
}

/// A square of side `base + extra`, cut at `base` along both sides: a block,
/// two strips and a corner. The picture behind squares ending in 5 (extra
/// 5) and squares just above 50 (base 50).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SquareCut {
    pub base: u64,
    pub extra: u64,
}

impl SquareCut {
    pub fn block(&self) -> u64 {
        self.base * self.base
    }

    /// One strip, `base` long and `extra` wide.
    pub fn strip(&self) -> u64 {
        self.base * self.extra
    }

    pub fn corner(&self) -> u64 {
        self.extra * self.extra
    }

    /// The two strips laid side by side against the block: the block grows
    /// from `base` by `base` to `base` by `base + 2 extra`.
    pub fn grown_block(&self) -> (u64, u64) {
        (self.base, self.base + 2 * self.extra)
    }

    /// The two strips laid end to end: one strip `2 base` long.
    pub fn long_strip(&self) -> (u64, u64) {
        (2 * self.base, self.extra)
    }

    pub fn total(&self) -> u64 {
        self.block() + 2 * self.strip() + self.corner()
    }
}

// ---------------------------------------------------------------------------
// trick 2: a product by parts, as a running total
// ---------------------------------------------------------------------------

/// One piece of the rectangle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Piece {
    pub height: u64,
    pub width: u64,
}

impl Piece {
    pub fn area(&self) -> u64 {
        self.height * self.width
    }

    /// The piece by its leading digits and its zeros: 20 × 40 is 2 × 4 = 8,
    /// then two zeros.
    pub fn by_size(&self) -> (u64, u32) {
        let ((h, hz), (w, wz)) = (lead_and_zeros(self.height), lead_and_zeros(self.width));
        (h * w, hz + wz)
    }
}

/// A product cut into pieces by the parts of its two numbers, in the order
/// the video adds them: the big block, the two strips, the corner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ByParts {
    pub a: u64,
    pub b: u64,
    pub pieces: Vec<Piece>,
    /// The total after each piece.
    pub running: Vec<u64>,
}

pub fn by_parts(a: u64, b: u64) -> ByParts {
    let ((a_tens, a_ones), (b_tens, b_ones)) = (parts(a), parts(b));
    let pieces: Vec<Piece> = [a_tens, a_ones]
        .into_iter()
        .flat_map(|height| [b_tens, b_ones].map(|width| Piece { height, width }))
        .filter(|p| p.area() > 0)
        .collect();
    let running = pieces
        .iter()
        .scan(0, |total, p| {
            *total += p.area();
            Some(*total)
        })
        .collect();
    ByParts { a, b, pieces, running }
}

impl ByParts {
    pub fn product(&self) -> u64 {
        self.running.last().copied().unwrap_or(0)
    }
}

/// Each number rounded to its nearest ten, and their product: the estimate
/// to make before the exact answer (23 × 46 is about 20 × 50).
pub fn estimate(a: u64, b: u64) -> (u64, u64, u64) {
    let round = |x: u64| (x + 5) / 10 * 10;
    (round(a), round(b), round(a) * round(b))
}

// ---------------------------------------------------------------------------
// trick 3: two numbers equally far from a middle
// ---------------------------------------------------------------------------

/// A product of two numbers the same distance either side of a middle: the
/// middle squared, minus the distance squared.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pair {
    pub low: u64,
    pub high: u64,
    pub middle: u64,
    pub distance: u64,
    pub middle_squared: u64,
    pub distance_squared: u64,
    pub product: u64,
}

/// `None` when the two numbers have no whole middle (their sum is odd).
pub fn pair(x: u64, y: u64) -> Option<Pair> {
    let (low, high) = (x.min(y), x.max(y));
    (low + high).is_multiple_of(2).then(|| {
        let (middle, distance) = ((low + high) / 2, (high - low) / 2);
        Pair {
            low,
            high,
            middle,
            distance,
            middle_squared: middle * middle,
            distance_squared: distance * distance,
            product: middle * middle - distance * distance,
        }
    })
}

// ---------------------------------------------------------------------------
// trick 4: squares near 50
// ---------------------------------------------------------------------------

pub const FIFTY: u64 = 50;

/// A square near 50, the way Bethe did 48: fifty squared, a hundred times
/// the distance on or off, then the distance squared.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NearFifty {
    pub number: u64,
    /// How far above 50 (negative below).
    pub distance: i64,
    /// 25 plus the distance: the hundreds.
    pub hundreds: i64,
    /// The distance squared: the last two digits while it is under 100.
    pub corner: u64,
    /// Fifty squared, then the hundred-times-the-distance step (Bethe's 2300).
    pub rough: u64,
    pub square: u64,
}

pub fn near_fifty(number: u64) -> NearFifty {
    let distance = number as i64 - FIFTY as i64;
    let hundreds = 25 + distance;
    let corner = distance.unsigned_abs().pow(2);
    let rough = (hundreds * 100) as u64;
    NearFifty {
        number,
        distance,
        hundreds,
        corner,
        rough,
        square: rough + corner,
    }
}

impl NearFifty {
    /// The corner fits in two digits, so the answer is "the hundreds, then
    /// the corner" with nothing carried.
    pub fn corner_fits(&self) -> bool {
        self.corner < 100
    }

    /// The answer read as two pairs of digits ("23" and "04"), when the
    /// corner fits.
    pub fn digit_pairs(&self) -> Option<(String, String)> {
        self.corner_fits()
            .then(|| (self.hundreds.to_string(), format!("{:02}", self.corner)))
    }
}

// ---------------------------------------------------------------------------
// trick 5: percents by blocks, and the swap
// ---------------------------------------------------------------------------

/// A percent of an amount of money built from the ten-percent block: the
/// block, half of it, and the two together (the fifteen-percent tip) or the
/// block doubled (twenty percent).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Blocks {
    pub cents: u64,
    pub ten: u64,
    pub five: u64,
    pub fifteen: u64,
    pub twenty: u64,
}

/// `None` when the blocks do not come out in whole cents.
pub fn blocks(cents: u64) -> Option<Blocks> {
    cents.is_multiple_of(20).then(|| {
        let ten = cents / 10;
        Blocks {
            cents,
            ten,
            five: ten / 2,
            fifteen: ten + ten / 2,
            twenty: 2 * ten,
        }
    })
}

/// x percent of y, read either way round: x% of y and y% of x are the same
/// rectangle, x by y, over a hundred. `None` unless it is a whole number.
pub fn percent_of(x: u64, y: u64) -> Option<u64> {
    (x * y).is_multiple_of(100).then_some(x * y / 100)
}

// ---------------------------------------------------------------------------
// trick 6: trading a factor of two
// ---------------------------------------------------------------------------

/// Times 5 as times 10, then half: the two numbers on the way.
pub fn times_five(x: u64) -> (u64, u64) {
    (x * 10, x * 10 / 2)
}

/// Times 25 as times 100, then half, then half again.
pub fn times_twenty_five(x: u64) -> (u64, u64, u64) {
    (x * 100, x * 100 / 2, x * 100 / 4)
}

/// Halve one number and double the other: the same rectangle, cut in half
/// and restacked. `None` when the first is odd.
pub fn halve_and_double(a: u64, b: u64) -> Option<(u64, u64, u64)> {
    a.is_multiple_of(2).then(|| (a / 2, b * 2, (a / 2) * (b * 2)))
}

// ---------------------------------------------------------------------------
// trick 7: round, then fix
// ---------------------------------------------------------------------------

/// A product with one number just under a round one: multiply by the round
/// number, then take the extra back off.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoundFix {
    pub count: u64,
    pub each: u64,
    pub round: u64,
    /// `count` times the round number.
    pub rounded: u64,
    /// What that overshot by: `count` times the gap.
    pub fix: u64,
    pub total: u64,
}

/// # Panics
/// If `round` is below `each` (round up, never down, in this video).
pub fn round_then_fix(count: u64, each: u64, round: u64) -> RoundFix {
    assert!(round >= each, "round up to a number at least as big");
    let (rounded, fix) = (count * round, count * (round - each));
    RoundFix {
        count,
        each,
        round,
        rounded,
        fix,
        total: rounded - fix,
    }
}

// ---------------------------------------------------------------------------
// the quizzes
// ---------------------------------------------------------------------------

/// Which trick a quiz is meant for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trick {
    EndsInFive,
    ByParts,
    Pair,
    NearFifty,
    PercentBlocks,
    PercentSwap,
    TimesFive,
    HalveAndDouble,
    RoundThenFix,
}

impl Trick {
    pub fn name(self) -> &'static str {
        match self {
            Trick::EndsInFive => "ends_in_five",
            Trick::ByParts => "by_parts",
            Trick::Pair => "pair",
            Trick::NearFifty => "near_fifty",
            Trick::PercentBlocks => "percent_blocks",
            Trick::PercentSwap => "percent_swap",
            Trick::TimesFive => "times_five",
            Trick::HalveAndDouble => "halve_and_double",
            Trick::RoundThenFix => "round_then_fix",
        }
    }
}

/// What a quiz asks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ask {
    Square(u64),
    Times(u64, u64),
    /// A percent of an amount of money, in cents.
    PercentOfMoney(u64, u64),
    /// A percent of a plain number.
    PercentOf(u64, u64),
    /// So many things at a price in cents each.
    TimesMoney(u64, u64),
}

impl Ask {
    /// The problem in one plain form, the same in `oracle.py`.
    pub fn text(&self) -> String {
        match *self {
            Ask::Square(x) => format!("{x}^2"),
            Ask::Times(a, b) => format!("{a}x{b}"),
            Ask::PercentOfMoney(p, cents) => format!("{p}% of ${}", money(cents)),
            Ask::PercentOf(p, y) => format!("{p}% of {y}"),
            Ask::TimesMoney(n, cents) => format!("{n}x${}", money(cents)),
        }
    }
}

/// One "your turn" quiz.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Quiz {
    /// The storyboard line that asks it.
    pub line: &'static str,
    pub trick: Trick,
    pub ask: Ask,
    /// The countdown, in seconds.
    pub seconds: u32,
}

/// A quiz worked through: the numbers the answer shows, in order, and the
/// answer itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Worked {
    pub steps: Vec<String>,
    pub answer: String,
}

impl Quiz {
    /// The quiz worked by its trick; an error if the trick does not fit
    /// what the quiz asks (a test walks them all).
    pub fn worked(&self) -> Result<Worked, String> {
        let s = |x: u64| x.to_string();
        let wrong = || format!("quiz {}: {:?} does not fit {:?}", self.line, self.trick, self.ask);
        Ok(match (self.trick, self.ask) {
            (Trick::EndsInFive, Ask::Square(x)) => {
                let e = ends_in_five(x).ok_or_else(wrong)?;
                Worked {
                    steps: vec![s(e.n), s(e.n + 1), s(e.hundreds)],
                    answer: s(e.square),
                }
            }
            (Trick::ByParts, Ask::Times(a, b)) => {
                let p = by_parts(a, b);
                Worked {
                    steps: p.running.iter().map(|&t| s(t)).collect(),
                    answer: s(p.product()),
                }
            }
            (Trick::Pair, Ask::Times(a, b)) => {
                let p = pair(a, b).ok_or_else(wrong)?;
                Worked {
                    steps: vec![s(p.middle), s(p.distance), s(p.middle_squared), s(p.distance_squared)],
                    answer: s(p.product),
                }
            }
            (Trick::NearFifty, Ask::Square(x)) => {
                let n = near_fifty(x);
                let (front, back) = n.digit_pairs().ok_or_else(wrong)?;
                Worked {
                    steps: vec![n.distance.to_string(), front, back],
                    answer: s(n.square),
                }
            }
            (Trick::PercentBlocks, Ask::PercentOfMoney(percent, cents)) => {
                let b = blocks(cents).ok_or_else(wrong)?;
                let (steps, answer) = match percent {
                    15 => (vec![money(b.ten), money(b.five)], b.fifteen),
                    20 => (vec![money(b.ten)], b.twenty),
                    _ => return Err(wrong()),
                };
                Worked {
                    steps,
                    answer: money(answer),
                }
            }
            (Trick::PercentSwap, Ask::PercentOf(x, y)) => {
                let answer = percent_of(x, y).ok_or_else(wrong)?;
                Worked {
                    // the swapped reading: y percent of x
                    steps: vec![s(y), s(x)],
                    answer: s(answer),
                }
            }
            (Trick::TimesFive, Ask::Times(5, x)) => {
                let (ten, half) = times_five(x);
                Worked {
                    steps: vec![s(ten)],
                    answer: s(half),
                }
            }
            (Trick::HalveAndDouble, Ask::Times(a, b)) => {
                let (half, double, product) = halve_and_double(a, b).ok_or_else(wrong)?;
                Worked {
                    steps: vec![s(half), s(double)],
                    answer: s(product),
                }
            }
            (Trick::RoundThenFix, Ask::Times(count, each)) => {
                let r = round_then_fix(count, each, each.next_multiple_of(10));
                Worked {
                    steps: vec![s(r.round), s(r.rounded), s(r.fix)],
                    answer: s(r.total),
                }
            }
            (Trick::RoundThenFix, Ask::TimesMoney(count, cents)) => {
                let r = round_then_fix(count, cents, cents.next_multiple_of(100));
                Worked {
                    steps: vec![money(r.round), money(r.rounded), money(r.fix)],
                    answer: money(r.total),
                }
            }
            _ => return Err(wrong()),
        })
    }
}

/// Every quiz of the video, in the order it is asked, each on the
/// storyboard line that asks it. The first is the cold open; chapter 12 is
/// the mixed round; the last is a product with nothing to grab.
pub const QUIZZES: &[Quiz] = &[
    quiz("01_1", Trick::EndsInFive, Ask::Square(85), 4),
    quiz("03_1", Trick::EndsInFive, Ask::Square(25), 4),
    quiz("03_3", Trick::EndsInFive, Ask::Square(65), 4),
    quiz("03_5", Trick::EndsInFive, Ask::Square(105), 6),
    quiz("05_4", Trick::ByParts, Ask::Times(6, 43), 5),
    quiz("05_6", Trick::ByParts, Ask::Times(13, 14), 6),
    quiz("06_6", Trick::Pair, Ask::Times(19, 21), 4),
    quiz("06_8", Trick::Pair, Ask::Times(65, 75), 6),
    quiz("09_1", Trick::NearFifty, Ask::Square(52), 4),
    quiz("09_3", Trick::NearFifty, Ask::Square(46), 5),
    quiz("09_5", Trick::NearFifty, Ask::Square(57), 5),
    quiz("10_4", Trick::PercentBlocks, Ask::PercentOfMoney(15, 6000), 5),
    quiz("10_9", Trick::PercentSwap, Ask::PercentOf(4, 75), 5),
    quiz("11_5", Trick::TimesFive, Ask::Times(5, 64), 4),
    quiz("11_7", Trick::HalveAndDouble, Ask::Times(16, 45), 5),
    quiz("11_9", Trick::RoundThenFix, Ask::TimesMoney(3, 499), 5),
    quiz("12_1", Trick::EndsInFive, Ask::Square(75), 5),
    quiz("12_3", Trick::Pair, Ask::Times(49, 51), 5),
    quiz("12_5", Trick::NearFifty, Ask::Square(54), 5),
    quiz("12_7", Trick::PercentBlocks, Ask::PercentOfMoney(20, 4500), 5),
    quiz("12_9", Trick::RoundThenFix, Ask::Times(6, 49), 5),
    quiz("13_1", Trick::ByParts, Ask::Times(67, 78), 6),
];

const fn quiz(line: &'static str, trick: Trick, ask: Ask, seconds: u32) -> Quiz {
    Quiz {
        line,
        trick,
        ask,
        seconds,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_number_breaks_into_its_parts() {
        assert_eq!(parts(23), (20, 3));
        assert_eq!(parts(105), (100, 5));
        assert_eq!(parts(6), (0, 6));
        assert_eq!(lead_and_zeros(20), (2, 1));
        assert_eq!(lead_and_zeros(800), (8, 2));
        assert_eq!(lead_and_zeros(7), (7, 0));
        assert_eq!(lead_and_zeros(0), (0, 0));
        assert_eq!(money(7996), "79.96");
        assert_eq!(money(900), "9.00");
        assert_eq!(money(4), "0.04");
    }

    #[test]
    fn the_paper_method_is_long_multiplication() {
        let p = paper(85, 85);
        assert_eq!(
            (p.rows.as_slice(), p.carries, p.product),
            ([425, 6800].as_slice(), 3, 7225)
        );
        // no carries anywhere: 12 × 12 = 24 + 120
        let p = paper(12, 12);
        assert_eq!(
            (p.rows.as_slice(), p.carries, p.product),
            ([24, 120].as_slice(), 0, 144)
        );
        for a in 1..=120 {
            for b in 1..=120 {
                let p = paper(a, b);
                assert_eq!(p.product, a * b, "{a} x {b}");
                assert_eq!(p.rows.len(), digits(b).len());
            }
        }
    }

    #[test]
    fn the_square_cut_is_the_square() {
        for base in (0..=200).step_by(10) {
            for extra in 0..=12 {
                let c = SquareCut { base, extra };
                assert_eq!(c.total(), (base + extra) * (base + extra));
                let (h, w) = c.grown_block();
                assert_eq!(h * w + c.corner(), c.total());
                let (long, wide) = c.long_strip();
                assert_eq!(c.block() + long * wide + c.corner(), c.total());
            }
        }
    }

    #[test]
    fn the_pieces_come_in_the_order_the_video_adds_them() {
        let p = by_parts(23, 46);
        let areas: Vec<u64> = p.pieces.iter().map(Piece::area).collect();
        assert_eq!(areas, [800, 120, 120, 18]);
        assert_eq!(p.running, [800, 920, 1040, 1058]);
        assert_eq!(p.pieces[0].by_size(), (8, 2));
        // one digit times two: two pieces
        let p = by_parts(6, 43);
        assert_eq!(p.running, [240, 258]);
        assert_eq!(estimate(23, 46), (20, 50, 1000));
        assert_eq!(estimate(67, 78), (70, 80, 5600));
    }

    #[test]
    fn a_pair_needs_a_whole_middle() {
        let p = pair(47, 53).unwrap();
        assert_eq!((p.middle, p.distance, p.product), (50, 3, 2491));
        assert_eq!(pair(53, 47), pair(47, 53));
        assert!(pair(47, 56).is_none());
    }

    #[test]
    fn near_fifty_is_bethes_route() {
        let n = near_fifty(48);
        assert_eq!(
            (n.distance, n.hundreds, n.rough, n.corner, n.square),
            (-2, 23, 2300, 4, 2304)
        );
        assert_eq!(n.digit_pairs(), Some(("23".into(), "04".into())));
        let n = near_fifty(53);
        assert_eq!((n.hundreds, n.corner, n.square), (28, 9, 2809));
        assert!(!near_fifty(62).corner_fits());
        assert_eq!(near_fifty(62).square, 3844);
    }

    #[test]
    fn the_small_tricks() {
        let b = blocks(4800).unwrap();
        assert_eq!((b.ten, b.five, b.fifteen, b.twenty), (480, 240, 720, 960));
        assert!(blocks(4810).is_none());
        assert_eq!(percent_of(8, 25), Some(2));
        assert_eq!(percent_of(25, 8), Some(2));
        assert_eq!(percent_of(7, 13), None);
        assert_eq!(times_five(86), (860, 430));
        assert_eq!(times_twenty_five(36), (3600, 1800, 900));
        assert_eq!(halve_and_double(14, 35), Some((7, 70, 490)));
        assert_eq!(halve_and_double(15, 37), None);
        let r = round_then_fix(4, 1999, 2000);
        assert_eq!((r.rounded, r.fix, r.total), (8000, 4, 7996));
    }

    #[test]
    fn every_quiz_fits_its_trick_and_sits_on_its_own_line() {
        let mut lines: Vec<&str> = QUIZZES.iter().map(|q| q.line).collect();
        lines.sort_unstable();
        lines.dedup();
        assert_eq!(lines.len(), QUIZZES.len(), "two quizzes on one line");
        for q in QUIZZES {
            assert!((4..=6).contains(&q.seconds), "{}: a countdown of 4 to 6 s", q.line);
            let w = q.worked().unwrap();
            assert!(!w.answer.is_empty() && !w.steps.is_empty(), "{}", q.line);
        }
    }
}
