//! Every number Video F shows or says, computed here and checked two ways:
//! `pin` against a literal from the fact sheet (the owner's brief, or the
//! source a cited number comes from), and `./x mental_math facts-check`
//! against `oracle.py` (same keys). The tricks are worked by `model.rs` the
//! way the video works them; the oracle gets every answer by plain
//! multiplication instead.

use explainer_kit::Facts;

use crate::model::{
    self, FIFTY, Piece, QUIZZES, blocks, by_parts, ends_in_five, estimate, halve_and_double, money, near_fifty,
    one_move, pair, paper, percent_of, round_then_fix, times_five,
};

/// Numbers quoted from the sources (fact sheet rows F-C; the oracle prints
/// the same literals).
pub mod cited {
    /// Feynman, "Surely You're Joking, Mr. Feynman!" (W. W. Norton, 1985),
    /// "Lucky Numbers", pp. 193-94: they "got to 48 squared"; Bethe's first
    /// answer is 2300 and his exact one 2304.
    pub const FEYNMAN_YEAR: u32 = 1985;
    pub const FEYNMAN_PAGES: &str = "193-94";
    pub const BETHE_NUMBER: u64 = 48;
    pub const BETHE_ROUGH: u64 = 2300;
    pub const BETHE_EXACT: u64 = 2304;
    /// Chase, The Elements of Arithmetic, Part Second (Philadelphia: Uriah
    /// Hunt and Son, 1844), p. 60: rules (10), (12) and (13) with the
    /// examples 35 × 35 = 1225, 87 × 73 = 6351 and 53 × 53 = 2809.
    pub const CHASE_YEAR: u32 = 1844;
    pub const CHASE_PAGE: u32 = 60;
    pub const CHASE_SQUARE: (u64, u64) = (35, 1225);
    pub const CHASE_PAIR: (u64, u64, u64) = (87, 73, 6351);
    pub const CHASE_NEAR_FIFTY: (u64, u64) = (53, 2809);
    /// Karatsuba's algorithm (Karatsuba and Ofman, 1962): a product of two
    /// two-part numbers from three multiplications, not four.
    pub const KARATSUBA_MULTIPLICATIONS: usize = 3;
}

/// The worked examples the chapters draw (the quizzes are `model::QUIZZES`).
pub mod shown {
    /// The cold open and the paper method.
    pub const HOOK: u64 = 85;
    /// The first square, the one whose strip is sliced off and moved.
    pub const FIVE: u64 = 35;
    /// The same move on a number that does not end in 5: no easier.
    pub const NOT_FIVE: (u64, u64) = (34, 4);
    /// The sweep: every number in front from 1 to this.
    pub const SWEEP: u64 = 10;
    /// Where the trick stops being easy: 13 × 14.
    pub const FIVE_HARD: u64 = 135;
    /// The same trick with a decimal point: seven and a half (description).
    pub const FIVE_TENTHS: u64 = 75;
    /// The owner's product by parts.
    pub const PARTS: (u64, u64) = (23, 46);
    /// One digit times two: two pieces.
    pub const TWO_PIECES: (u64, u64) = (6, 43);
    /// A restaurant bill, in cents.
    pub const BILL: u64 = 4800;
    /// Eight percent of twenty-five dollars.
    pub const SWAP: (u64, u64) = (8, 25);
    pub const TIMES_FIVE: u64 = 86;
    pub const HALVE: (u64, u64) = (14, 35);
    /// Halving that lands on nothing round.
    pub const HALVE_NO_EASIER: (u64, u64) = (14, 37);
    /// Four things at $19.99.
    pub const PRICE: (u64, u64) = (4, 1999);
    /// A round number too far away: seven thirty-fours as seven forties
    /// less seven sixes is no easier than parts.
    pub const FIX_NO_EASIER: (u64, u64) = (7, 34);
    pub const PAIR: (u64, u64) = (47, 53);
    /// A product with no whole middle.
    pub const NO_PAIR: (u64, u64) = (47, 56);
    /// An easy middle and a distance too big to square in the head.
    pub const FAR_PAIR: (u64, u64) = (12, 88);
    /// Squares near 50: Bethe's, one above, and one whose corner carries.
    pub const BELOW: u64 = 48;
    pub const ABOVE: u64 = 53;
    pub const CARRIES: u64 = 62;
    /// The square two tricks agree on.
    pub const TWIST: u64 = 55;
    /// The product with nothing to grab.
    pub const NOTHING: (u64, u64) = (67, 78);
    /// For the description only: the move near a hundred, and two numbers
    /// with the same front whose last digits add to ten.
    pub const NEAR_HUNDRED: (u64, u64) = (97, 3);
    pub const SAME_FRONT: (u64, u64) = (23, 27);
}

fn list<T: ToString>(items: impl IntoIterator<Item = T>) -> String {
    items.into_iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")
}

fn sides(p: &Piece) -> String {
    format!("{}x{}", p.height, p.width)
}

pub fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    hook(&mut f)?;
    fives(&mut f)?;
    moves(&mut f)?;
    parts(&mut f)?;
    percents(&mut f)?;
    round_numbers(&mut f)?;
    pairs(&mut f)?;
    fifty(&mut f)?;
    nothing(&mut f)?;
    description(&mut f)?;
    quizzes(&mut f)?;
    edges(&mut f)?;
    sources(&mut f)?;
    Ok(f)
}

/// F-H: eighty-five squared the paper way, and by the trick.
fn hook(f: &mut Facts) -> Result<(), String> {
    let p = paper(shown::HOOK, shown::HOOK);
    f.add("F-H1.paper.rows", list(&p.rows))?;
    f.add("F-H1.paper.row_count", p.rows.len())?;
    f.add("F-H1.paper.carries", p.carries)?;
    f.add("F-H1.paper.carried", list(&p.carried))?;
    f.add("F-H1.paper.product", p.product)?;
    let e = ends_in_five(shown::HOOK).ok_or("the hook's number does not end in 5")?;
    f.add("F-H1.trick.front", e.n)?;
    f.add("F-H1.trick.next", e.n + 1)?;
    f.add("F-H1.trick.hundreds", e.hundreds)?;
    f.pin("F-H1.trick.square", e.square, "7225", "the order: 85^2 = 7225")?;
    Ok(())
}

/// F-T1: squares of numbers ending in 5.
fn fives(f: &mut Facts) -> Result<(), String> {
    let order = "the order";
    for (number, want) in [
        (shown::FIVE, "1225"),
        (105, "11025"),
        (shown::FIVE_HARD, "18225"),
        (5, "25"),
        (shown::TWIST, "3025"),
    ] {
        let e = ends_in_five(number).ok_or("an example does not end in 5")?;
        let key = format!("F-T1.n{number}");
        f.add(&format!("{key}.number"), number)?;
        f.add(&format!("{key}.front"), e.n)?;
        f.add(&format!("{key}.next"), e.n + 1)?;
        f.add(&format!("{key}.hundreds"), e.hundreds)?;
        f.pin(&format!("{key}.square"), e.square, want, order)?;
    }
    // the sweep: every number in front, its hundreds and its square
    let swept: Vec<_> = (1..=shown::SWEEP)
        .map(|n| ends_in_five(10 * n + 5).ok_or("the sweep left the fives"))
        .collect::<Result<_, _>>()?;
    f.add("F-T1.sweep.fronts", list(swept.iter().map(|e| e.n)))?;
    f.add("F-T1.sweep.hundreds", list(swept.iter().map(|e| e.hundreds)))?;
    f.add("F-T1.sweep.squares", list(swept.iter().map(|e| e.square)))?;
    // seven and a half squared: the same digits with the point put back
    let e = ends_in_five(shown::FIVE_TENTHS).ok_or("75 ends in 5")?;
    f.pin(
        "F-T1.half.number",
        format!("{}.{}", shown::FIVE_TENTHS / 10, shown::FIVE_TENTHS % 10),
        "7.5",
        "75 tenths",
    )?;
    f.pin("F-T1.half.squared", money(e.square), "56.25", "7.5 x 7.5")?;
    // the same front, next number and hundreds as 75 squared: 7 × 8 = 56, and a quarter
    f.add("F-T1.half.front", e.n)?;
    f.add("F-T1.half.next", e.n + 1)?;
    f.add("F-T1.half.hundreds", e.hundreds)?;
    Ok(())
}

/// F-M: the one move, on every square the video cuts.
fn moves(f: &mut Facts) -> Result<(), String> {
    let (not_five, strip) = shown::NOT_FIVE;
    for (side, strip) in [
        (shown::FIVE, 5),
        (not_five, strip),
        (shown::BELOW, FIFTY - shown::BELOW),
        (shown::ABOVE, shown::ABOVE - FIFTY),
        (FIFTY, shown::PAIR.1 - FIFTY),
        (shown::TWIST, 5),
        (shown::CARRIES, shown::CARRIES - FIFTY),
    ] {
        let m = one_move(side, strip).ok_or("a strip wider than its square")?;
        let key = format!("F-M.n{side}");
        f.add(&format!("{key}.strip_wide"), m.strip)?;
        f.add(&format!("{key}.strip_long"), m.side)?;
        f.add(&format!("{key}.wide"), m.wide())?;
        f.add(&format!("{key}.tall"), m.tall())?;
        f.add(&format!("{key}.rectangle_area"), m.rectangle())?;
        f.add(&format!("{key}.corner"), m.corner())?;
        f.add(&format!("{key}.square"), m.square())?;
    }
    // thirty by forty, ruled into squares of a hundred
    let m = one_move(shown::FIVE, 5).ok_or("35 is wider than 5")?;
    f.add("F-M.n35.hundreds_tiles", format!("{}x{}", m.tall() / 10, m.wide() / 10))?;
    f.pin("F-M.n35.hundreds", m.rectangle() / 100, "12", "the order: 3 x 4 = 12")?;
    Ok(())
}

/// F-T2: 23 times 46 by parts.
fn parts(f: &mut Facts) -> Result<(), String> {
    let order = "the order";
    let (a, b) = shown::PARTS;
    let p = by_parts(a, b);
    f.add("F-T2.factors", list([a, b]))?;
    f.add("F-T2.parts.first", list(<[u64; 2]>::from(model::parts(a))))?;
    f.add("F-T2.parts.second", list(<[u64; 2]>::from(model::parts(b))))?;
    f.add("F-T2.pieces.sides", list(p.pieces.iter().map(sides)))?;
    f.pin(
        "F-T2.pieces",
        list(p.pieces.iter().map(Piece::area)),
        "800,120,120,18",
        order,
    )?;
    f.pin("F-T2.running", list(&p.running), "800,920,1040,1058", order)?;
    f.pin("F-T2.product", p.product(), "1058", order)?;
    let block = p.pieces.first().ok_or("no pieces")?;
    let (lead, zeros) = block.by_size();
    f.add("F-T2.size.lead", lead)?;
    f.add("F-T2.size.zeros", zeros)?;
    // the sizes it cannot be: one zero too few is 20 × 4, one too many 200 × 40
    f.add("F-T2.size.too_small", block.area() / 10)?;
    f.add("F-T2.size.too_big", block.area() * 10)?;
    let (ra, rb, about) = estimate(a, b);
    f.add("F-T2.estimate", list([ra, rb, about]))?;
    // the estimate by size: 2 × 5 = 10, in hundreds
    let (lead, zeros) = Piece { height: ra, width: rb }.by_size();
    f.add("F-T2.estimate.lead", lead)?;
    f.add("F-T2.estimate.zeros", zeros)?;
    f.add("F-T2.held.pieces", p.pieces.len())?;
    f.add("F-T2.bounds", list(<[u64; 2]>::from(p.bounds())))?;
    let (x, y) = shown::TWO_PIECES;
    f.add("F-T2.piece_counts", list([by_parts(x, y).pieces.len(), p.pieces.len()]))?;
    f.add("F-T2.two_pieces.factors", list([x, y]))?;
    Ok(())
}

/// F-T5: percents from a tenth, and the swap.
fn percents(f: &mut Facts) -> Result<(), String> {
    let b = blocks(shown::BILL).ok_or("the bill's tenth is not whole cents")?;
    f.add("F-T5.bill", money(b.cents))?;
    f.add("F-T5.bill.ten", money(b.ten))?;
    f.add("F-T5.bill.five", money(b.five))?;
    f.add("F-T5.bill.fifteen", money(b.fifteen))?;
    f.add("F-T5.bill.twenty", money(b.twenty))?;
    // seventeen percent is no longer one piece: a tenth, half of it, and two hundredths
    let one = b.cents / 100;
    let pieces = [b.ten, b.five, one, one];
    f.add("F-T5.seventeen.pieces", list(pieces.map(money)))?;
    f.add("F-T5.seventeen.total", money(pieces.iter().sum()))?;
    // eight percent of twenty-five dollars: eight cents from each dollar
    let (percent, dollars) = shown::SWAP;
    f.add("F-T5.swap.across", dollars)?;
    f.add("F-T5.swap.deep", percent)?;
    f.add("F-T5.swap.cents", percent * dollars)?;
    f.pin(
        "F-T5.swap.one_way",
        money(percent_of(percent, dollars * 100).ok_or("not whole cents")?),
        "2.00",
        "the order: 8% of 25",
    )?;
    f.add(
        "F-T5.swap.other_way",
        money(percent_of(dollars, percent * 100).ok_or("not whole cents")?),
    )?;
    Ok(())
}

/// F-T6 and F-T7: halving and doubling; round, then fix.
fn round_numbers(f: &mut Facts) -> Result<(), String> {
    let (ten, half) = times_five(shown::TIMES_FIVE);
    f.add("F-T6.five.ten", ten)?;
    f.add("F-T6.five.product", half)?;
    for (name, (a, b)) in [("halve", shown::HALVE), ("no_easier", shown::HALVE_NO_EASIER)] {
        let (half, double, product) = halve_and_double(a, b).ok_or("nothing to halve")?;
        f.add(&format!("F-T6.{name}.factors"), list([a, b]))?;
        f.add(&format!("F-T6.{name}.halved"), half)?;
        f.add(&format!("F-T6.{name}.doubled"), double)?;
        f.add(&format!("F-T6.{name}.product"), product)?;
    }
    f.pin(
        "F-T6.halve.check",
        shown::HALVE.0 * shown::HALVE.1,
        "490",
        "the order: 14 x 35 = 7 x 70",
    )?;
    let (count, cents) = shown::PRICE;
    let r = round_then_fix(count, cents, cents.next_multiple_of(100));
    f.add("F-T7.price.each", money(r.each))?;
    f.add("F-T7.price.round", money(r.round))?;
    f.add("F-T7.price.rounded", money(r.rounded))?;
    f.add("F-T7.price.fix", money(r.fix))?;
    f.add("F-T7.price.total", money(r.total))?;
    // seven thirty-fours: the round number is six away, so the fix is 42,
    // and parts (210, then 28 more) is as easy
    let (count, each) = shown::FIX_NO_EASIER;
    let r = round_then_fix(count, each, each.next_multiple_of(10));
    f.add("F-T7.no_easier.factors", list([count, each]))?;
    f.add("F-T7.no_easier.rounded", r.rounded)?;
    f.add("F-T7.no_easier.fix", r.fix)?;
    f.add("F-T7.no_easier.product", r.total)?;
    let by = by_parts(count, each);
    f.add("F-T7.no_easier.by_parts", list(&by.running))?;
    f.add("F-T7.no_easier.pieces", list(by.pieces.iter().map(Piece::area)))?;
    Ok(())
}

/// F-T3: two numbers equally far from a middle.
fn pairs(f: &mut Facts) -> Result<(), String> {
    let (x, y) = shown::PAIR;
    let p = pair(x, y).ok_or("the pair has no middle")?;
    f.add("F-T3.middle", p.middle)?;
    f.add("F-T3.distance", p.distance)?;
    f.add("F-T3.middle_squared", p.middle_squared)?;
    f.add("F-T3.distance_squared", p.distance_squared)?;
    f.pin("F-T3.product", p.product, "2491", "the order")?;
    // the picture: the rectangle is 47 wide and 53 tall; the strip that
    // stands above the square of 50 moves into the gap down its side
    f.add("F-T3.strip_area", p.distance * p.low)?;
    f.add("F-T3.gap_area", p.distance * p.middle)?;
    f.add("F-T3.hole", p.distance * p.middle - p.distance * p.low)?;
    let (x, y) = shown::NO_PAIR;
    f.add("F-T3.no_middle", pair(x, y).is_none())?;
    // its middle, in halves: 103 halves is fifty-one and a half
    f.add("F-T3.no_middle.sum", x + y)?;
    let (x, y) = shown::FAR_PAIR;
    let far = pair(x, y).ok_or("the far pair has no middle")?;
    f.add("F-T3.far.middle", far.middle)?;
    f.add("F-T3.far.distance", far.distance)?;
    f.add("F-T3.far.distance_squared", far.distance_squared)?;
    f.add("F-T3.far.product", far.product)?;
    Ok(())
}

/// F-T4: squares near 50.
fn fifty(f: &mut Facts) -> Result<(), String> {
    f.add("F-T4.fifty_squared", FIFTY * FIFTY)?;
    f.add("F-T4.fifty.number", FIFTY)?;
    f.add("F-T4.fifty.hundreds", FIFTY * FIFTY / 100)?;
    for (name, number) in [
        ("below", shown::BELOW),
        ("above", shown::ABOVE),
        ("carries", shown::CARRIES),
    ] {
        let n = near_fifty(number);
        let key = format!("F-T4.{name}");
        f.add(&format!("{key}.number"), n.number)?;
        f.add(&format!("{key}.distance"), n.distance)?;
        f.add(&format!("{key}.hundreds"), n.hundreds)?;
        f.add(&format!("{key}.rough"), n.rough)?;
        f.add(&format!("{key}.corner"), n.corner)?;
        f.add(&format!("{key}.corner_fits"), n.corner_fits())?;
        f.add(&format!("{key}.square"), n.square)?;
        // against fifty squared the moved rectangle is short by a column, or
        // taller by rows, twice the distance across: that many fifties
        let across = 2 * n.distance.unsigned_abs();
        f.add(&format!("{key}.fifties"), across)?;
        f.add(&format!("{key}.fifties_area"), across * FIFTY)?;
    }
    let (front, back) = near_fifty(shown::ABOVE).digit_pairs().ok_or("the corner carries")?;
    f.add("F-T4.above.digit_pairs", format!("{front},{back}"))?;
    let (front, back) = near_fifty(shown::BELOW).digit_pairs().ok_or("the corner carries")?;
    f.add("F-T4.below.digit_pairs", format!("{front},{back}"))?;
    // sixty-two: the corner is 144, so a hundred of it joins the hundreds
    let far = near_fifty(shown::CARRIES);
    f.add("F-T4.carries.hundreds_after", far.square / 100)?;
    f.add("F-T4.carries.left", far.square % 100)?;
    // fifty-five, two ways: near fifty, and ends in five
    let twist = near_fifty(shown::TWIST);
    let (front, back) = twist.digit_pairs().ok_or("the corner carries")?;
    f.add("F-T4.twist.digit_pairs", format!("{front},{back}"))?;
    let five = ends_in_five(shown::TWIST).ok_or("the twist does not end in 5")?;
    f.add(
        "F-T4.twist.same_two_numbers",
        twist.hundreds == five.hundreds as i64 && twist.corner == 25,
    )?;
    Ok(())
}

/// F-D: numbers only `docs/description.md` states.
fn description(f: &mut Facts) -> Result<(), String> {
    // the move near a hundred: 97 squared is 94 by 100, and 9
    let (side, strip) = shown::NEAR_HUNDRED;
    let m = one_move(side, strip).ok_or("a strip deeper than its square")?;
    f.add("F-D.n97.number", side)?;
    f.add("F-D.n97.tall", m.tall())?;
    f.add("F-D.n97.wide", m.wide())?;
    f.add("F-D.n97.corner", m.corner())?;
    f.add("F-D.n97.square", m.square())?;
    // 23 × 27: the same front, and last digits that add to ten, so it is the
    // pair 25 ± 2: the front times the next in hundreds, then 3 × 7
    let (x, y) = shown::SAME_FRONT;
    let p = pair(x, y).ok_or("the same-front pair has no middle")?;
    let five = ends_in_five(p.middle).ok_or("the middle does not end in 5")?;
    f.add("F-D.same_front.factors", format!("{x},{y}"))?;
    f.add("F-D.same_front.front", five.n)?;
    f.add("F-D.same_front.next", five.n + 1)?;
    f.add("F-D.same_front.last_digits", list([x % 10, y % 10]))?;
    f.add("F-D.same_front.hundreds", five.hundreds)?;
    f.add("F-D.same_front.ones", (x % 10) * (y % 10))?;
    f.add("F-D.same_front.product", p.product)?;
    // 47 × 56 has a half middle: in halves, (103 × 103 − 9 × 9) ÷ 4
    let (x, y) = shown::NO_PAIR;
    let (sum, gap) = (x + y, y - x);
    let halves = |twice: u64| format!("{}.{}", twice / 2, twice % 2 * 5);
    f.add("F-D.half_middle.factors", format!("{x},{y}"))?;
    f.add("F-D.half_middle.middle", halves(sum))?;
    f.add("F-D.half_middle.distance", halves(gap))?;
    f.add("F-D.half_middle.middle_squared", money((sum * sum) * 25))?;
    f.add("F-D.half_middle.distance_squared", money((gap * gap) * 25))?;
    f.add("F-D.half_middle.product", (sum * sum - gap * gap) / 4)?;
    Ok(())
}

/// F-N: the product with nothing to grab.
fn nothing(f: &mut Facts) -> Result<(), String> {
    let (a, b) = shown::NOTHING;
    let (ra, rb, about) = estimate(a, b);
    f.add("F-N.product", a * b)?;
    f.add("F-N.estimate", list([ra, rb, about]))?;
    // a decimal point slipped one place either way on a calculator
    f.add("F-N.slipped_low", format!("{}.{}", a * b / 10, a * b % 10))?;
    f.add("F-N.slipped_high", a * b * 10)?;
    // its middle, in halves: 145 halves is seventy-two and a half
    f.add("F-N.middle_halves", a + b)?;
    f.add("F-N.has_whole_middle", pair(a, b).is_some())?;
    f.add(
        "F-N.ends_in_five",
        ends_in_five(a).is_some() || ends_in_five(b).is_some(),
    )?;
    // round, then fix: 67 eighties less 67 twos; true, and no easier
    let r = round_then_fix(a, b, rb);
    f.add("F-N.round_fix.rounded", r.rounded)?;
    f.add("F-N.round_fix.fix", r.fix)?;
    f.add("F-N.round_fix.total", r.total)?;
    Ok(())
}

/// F-Q: every quiz, worked.
fn quizzes(f: &mut Facts) -> Result<(), String> {
    f.add("F-Q.count", QUIZZES.len())?;
    f.add("F-Q.countdown_seconds", QUIZZES.iter().map(|q| q.seconds).sum::<u32>())?;
    for q in QUIZZES {
        let key = format!("F-Q.{}", q.line);
        let w = q.worked()?;
        f.add(&format!("{key}.ask"), q.ask.text())?;
        f.add(&format!("{key}.trick"), q.trick.name())?;
        f.add(&format!("{key}.seconds"), q.seconds)?;
        f.add(&format!("{key}.steps"), list(&w.steps))?;
        f.add(&format!("{key}.answer"), &w.answer)?;
    }
    Ok(())
}

/// How many of `cases` satisfy `holds`, and how many there are.
fn tally<T>(cases: impl IntoIterator<Item = T>, holds: impl Fn(&T) -> bool) -> (usize, usize) {
    cases
        .into_iter()
        .fold((0, 0), |(yes, all), c| (yes + usize::from(holds(&c)), all + 1))
}

/// The squares ending in 5 from 5 up to 99,995: the trick against plain
/// multiplication.
pub fn edge_ends_in_five() -> (usize, usize) {
    tally((0..10_000u64).map(|n| 10 * n + 5), |&x| {
        ends_in_five(x).is_some_and(|e| e.square == x * x)
    })
}

/// Every product of two numbers from 1 to 99: the pieces add up, each
/// piece's size is its leading digits and its zeros, the running total
/// never goes down, and the answer lies from the tens multiplied up to (but
/// not at) the next tens multiplied.
pub fn edge_by_parts() -> (usize, usize) {
    tally((1..100u64).flat_map(|a| (1..100u64).map(move |b| (a, b))), |&(a, b)| {
        let p = by_parts(a, b);
        let (floor, ceiling) = p.bounds();
        p.product() == a * b
            && p.running.is_sorted()
            && (floor..ceiling).contains(&(a * b))
            && p.pieces.iter().all(|piece| {
                let (lead, zeros) = piece.by_size();
                lead * 10u64.pow(zeros) == piece.area()
            })
    })
}

/// Every two numbers from 1 to 200 with a whole middle: the middle squared
/// minus the distance squared is their product.
pub fn edge_pair() -> (usize, usize) {
    tally(
        (1..=200u64)
            .flat_map(|x| (x..=200u64).map(move |y| (x, y)))
            .filter(|(x, y)| (x + y) % 2 == 0),
        |&(x, y)| pair(x, y).is_some_and(|p| p.product == x * y),
    )
}

/// The largest number the edge tests of the near-fifty sentences try; they
/// start at 0.
const NEAR_FIFTY_LAST: u64 = 100;

/// Every whole number from 0 to 100: fifty squared, a hundred times the
/// distance, the distance squared.
pub fn edge_near_fifty() -> (usize, usize) {
    tally(0..=NEAR_FIFTY_LAST, |&x| near_fifty(x).square == x * x)
}

/// The numbers whose square really is "25 plus the distance, then the
/// distance squared as two digits": first and last.
pub fn digit_pairs_range() -> (u64, u64) {
    let reads = |x: &u64| {
        near_fifty(*x)
            .digit_pairs()
            .is_some_and(|(front, back)| format!("{front}{back}") == (x * x).to_string())
    };
    let ok: Vec<u64> = (0..=NEAR_FIFTY_LAST).filter(reads).collect();
    (ok[0], ok[ok.len() - 1])
}

/// Every x and y from 0 to 200: x percent of y is y percent of x. The
/// rectangle is counted both ways, y rows of x and x rows of y, before
/// either is put over a hundred.
pub fn edge_swap() -> (usize, usize) {
    tally(
        (0..=200u64).flat_map(|x| (0..=200u64).map(move |y| (x, y))),
        |&(x, y)| {
            let rows_of_x: u64 = (0..y).map(|_| x).sum();
            let rows_of_y: u64 = (0..x).map(|_| y).sum();
            rows_of_x == rows_of_y && percent_of(x, y) == percent_of(y, x)
        },
    )
}

/// Every even number from 2 to 200 against every number from 1 to 200:
/// halving one and doubling the other keeps the product.
pub fn edge_halve() -> (usize, usize) {
    tally(
        (1..=100u64).flat_map(|a| (1..=200u64).map(move |b| (2 * a, b))),
        |&(a, b)| halve_and_double(a, b).is_some_and(|(_, _, product)| product == a * b),
    )
}

/// Every count from 1 to 99 against every number from 1 to 999, rounded up
/// to the next ten: the rounded product less the fix is the product.
pub fn edge_round_then_fix() -> (usize, usize) {
    tally(
        (1..100u64).flat_map(|count| (1..1000u64).map(move |each| (count, each))),
        |&(count, each)| round_then_fix(count, each, each.next_multiple_of(10)).total == count * each,
    )
}

/// Every square of side 1 to 200 and every strip no wider than it: the
/// rectangle the move leaves, and the corner, make the square.
pub fn edge_one_move() -> (usize, usize) {
    tally(
        (1..=200u64).flat_map(|side| (0..=side).map(move |strip| (side, strip))),
        |&(side, strip)| one_move(side, strip).is_some_and(|m| m.square() == side * side),
    )
}

/// F-X: the edges of the always sentences, each as "how many hold of how
/// many checked".
fn edges(f: &mut Facts) -> Result<(), String> {
    for (name, (yes, all)) in [
        ("F-X1.ends_in_five", edge_ends_in_five()),
        ("F-X2.by_parts", edge_by_parts()),
        ("F-X3.pair", edge_pair()),
        ("F-X4.near_fifty", edge_near_fifty()),
        ("F-X5.swap", edge_swap()),
        ("F-X6.halve", edge_halve()),
        ("F-X7.round_then_fix", edge_round_then_fix()),
        ("F-X8.one_move", edge_one_move()),
    ] {
        f.add(&format!("{name}.holds"), yes)?;
        f.add(&format!("{name}.checked"), all)?;
    }
    f.add("F-X1.ends_in_five.largest", 10 * 9_999 + 5)?;
    let (first, last) = digit_pairs_range();
    f.add("F-X4.near_fifty.largest", NEAR_FIFTY_LAST)?;
    f.add("F-X4.digit_pairs.first", first)?;
    f.add("F-X4.digit_pairs.last", last)?;
    Ok(())
}

/// F-C: the numbers quoted from the sources, each against the video's own
/// arithmetic where there is any.
fn sources(f: &mut Facts) -> Result<(), String> {
    let feynman = "Feynman 1985, Lucky Numbers, pp. 193-94";
    f.pin("F-C1.feynman.year", cited::FEYNMAN_YEAR, "1985", "W. W. Norton, 1985")?;
    f.pin("F-C1.feynman.pages", cited::FEYNMAN_PAGES, "193-94", feynman)?;
    let bethe = near_fifty(cited::BETHE_NUMBER);
    f.pin("F-C1.bethe.number", bethe.number, "48", feynman)?;
    f.pin(
        "F-C1.bethe.rough",
        bethe.rough,
        &cited::BETHE_ROUGH.to_string(),
        feynman,
    )?;
    f.pin(
        "F-C1.bethe.exact",
        bethe.square,
        &cited::BETHE_EXACT.to_string(),
        feynman,
    )?;
    let chase = "Chase 1844, p. 60";
    f.pin("F-C3.chase.year", cited::CHASE_YEAR, "1844", chase)?;
    f.pin("F-C3.chase.page", cited::CHASE_PAGE, "60", chase)?;
    let (number, square) = cited::CHASE_SQUARE;
    f.pin("F-C3.chase.square_number", number, "35", chase)?;
    let e = ends_in_five(number).ok_or("Chase's square does not end in 5")?;
    f.pin("F-C3.chase.square", e.square, &square.to_string(), chase)?;
    let (x, y, product) = cited::CHASE_PAIR;
    let p = pair(x, y).ok_or("Chase's pair has no middle")?;
    f.pin("F-C3.chase.pair", p.product, &product.to_string(), chase)?;
    f.pin("F-C3.chase.pair_numbers", format!("{x},{y}"), "87,73", chase)?;
    let (number, square) = cited::CHASE_NEAR_FIFTY;
    f.pin("F-C3.chase.near_fifty_number", number, "53", chase)?;
    f.pin(
        "F-C3.chase.near_fifty",
        near_fifty(number).square,
        &square.to_string(),
        chase,
    )?;
    // Karatsuba on the digits of the video's own rectangle: 2 × 4 for the
    // block, 3 × 6 for the corner, and one product of the two digit sums,
    // (2 + 3) × (4 + 6), from which both strips come, in tens
    let (a, b) = shown::PARTS;
    let ((a1, a0), (b1, b0)) = ((a / 10, a % 10), (b / 10, b % 10));
    let products = [a1 * b1, a0 * b0, (a1 + a0) * (b1 + b0)];
    f.add("F-C4.karatsuba.digits", list([a1, a0, b1, b0]))?;
    f.pin(
        "F-C4.karatsuba.multiplications",
        products.len(),
        &cited::KARATSUBA_MULTIPLICATIONS.to_string(),
        "Karatsuba and Ofman 1962",
    )?;
    let strips = products[2] - products[0] - products[1];
    f.add("F-C4.karatsuba.block", products[0])?;
    f.add("F-C4.karatsuba.corner", products[1])?;
    f.add("F-C4.karatsuba.sums", products[2])?;
    f.add("F-C4.karatsuba.strips_in_tens", strips)?;
    f.add("F-C4.karatsuba.product", products[0] * 100 + strips * 10 + products[1])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_fact_is_registered_once_and_every_pin_holds() {
        let f = facts().unwrap();
        f.self_check().unwrap();
        assert!(f.len() > 200, "{} facts", f.len());
    }

    /// Pattern 3: the edges of every always sentence.
    #[test]
    fn edge_the_square_of_every_number_ending_in_five_is_the_front_times_the_next_then_25() {
        let (yes, all) = edge_ends_in_five();
        assert_eq!((yes, all), (10_000, 10_000));
        // the edges: nothing in front of the 5, and a front that ends in 9
        assert_eq!(ends_in_five(5).unwrap().square, 25);
        assert_eq!(ends_in_five(95).unwrap().square, 9025);
        assert_eq!(ends_in_five(995).unwrap().square, 990_025);
        // and only numbers that end in 5
        assert!((0..1000u64).all(|x| ends_in_five(x).is_some() == (x % 10 == 5)));
    }

    #[test]
    fn edge_the_pieces_always_add_up_to_the_product() {
        let (yes, all) = edge_by_parts();
        assert_eq!((yes, all), (99 * 99, 99 * 99));
        // a round number has no ones piece, a single digit no tens piece
        assert_eq!(by_parts(20, 40).running, [800]);
        assert_eq!(by_parts(7, 8).running, [56]);
        assert_eq!(by_parts(30, 7).running, [210]);
        // every total is a floor, but the first is not always the right
        // size: nineteen squared starts at a hundred (the script must never
        // say the leading digits are already right)
        assert_eq!(by_parts(19, 19).running, [100, 190, 280, 361]);
    }

    #[test]
    fn edge_equally_far_from_a_middle_is_the_middle_squared_less_the_distance_squared() {
        let (yes, all) = edge_pair();
        assert_eq!(yes, all);
        assert!(all > 10_000);
        // distance zero is a plain square
        assert_eq!(pair(50, 50).unwrap().product, 2500);
        // and an odd sum has no whole middle
        assert!((1..=200u64).all(|x| pair(x, x + 1).is_none()));
    }

    #[test]
    fn edge_near_fifty_is_exact_and_the_two_digit_reading_stops_at_ten_away() {
        assert_eq!(edge_near_fifty(), (101, 101));
        // "25 plus the distance, then the distance squared" reads off the
        // answer for 41 to 59 and for no other number from 0 to 100
        assert_eq!(digit_pairs_range(), (41, 59));
        let fits: Vec<u64> = (0..=100).filter(|&x| near_fifty(x).corner_fits()).collect();
        assert_eq!(fits, (41..=59).collect::<Vec<_>>());
        // ten away, the corner is a hundred and carries
        assert_eq!(near_fifty(60).corner, 100);
        assert_eq!(near_fifty(40).square, 1600);
    }

    #[test]
    fn edge_the_small_tricks_hold_over_their_whole_ranges() {
        for (yes, all) in [edge_swap(), edge_halve(), edge_round_then_fix(), edge_one_move()] {
            assert_eq!(yes, all);
            assert!(all > 10_000);
        }
    }

    /// The numbers the storyboard's pictures show, pinned (what the voice
    /// says is tied to the facts by `every_spoken_number_is_a_fact` in
    /// `main.rs`).
    #[test]
    fn the_storyboard_s_pictures() {
        let f = facts().unwrap();
        for (key, want) in [
            ("F-H1.paper.rows", "425,6800"),
            ("F-H1.paper.row_count", "2"),
            ("F-H1.paper.carries", "3"),
            ("F-M.n35.tall", "30"),
            ("F-M.n35.wide", "40"),
            ("F-M.n35.corner", "25"),
            ("F-M.n35.hundreds_tiles", "3x4"),
            ("F-M.n34.tall", "30"),
            ("F-M.n34.wide", "38"),
            ("F-M.n34.corner", "16"),
            ("F-T1.sweep.hundreds", "2,6,12,20,30,42,56,72,90,110"),
            ("F-T2.estimate.lead", "10"),
            ("F-T2.bounds", "800,1500"),
            ("F-T2.piece_counts", "2,4"),
            ("F-T5.bill.fifteen", "7.20"),
            ("F-T5.seventeen.total", "8.16"),
            ("F-T5.swap.cents", "200"),
            ("F-T5.swap.other_way", "2.00"),
            ("F-T6.five.product", "430"),
            ("F-T6.halve.doubled", "70"),
            ("F-T6.no_easier.doubled", "74"),
            ("F-T7.price.total", "79.96"),
            ("F-T7.no_easier.fix", "42"),
            ("F-T7.no_easier.by_parts", "210,238"),
            ("F-T3.hole", "9"),
            ("F-T3.far.distance_squared", "1444"),
            ("F-M.n48.tall", "46"),
            ("F-M.n48.wide", "50"),
            ("F-T4.below.fifties", "4"),
            ("F-T4.below.fifties_area", "200"),
            ("F-M.n53.wide", "56"),
            ("F-T4.above.fifties", "6"),
            ("F-T4.below.digit_pairs", "23,04"),
            ("F-T4.above.digit_pairs", "28,09"),
            ("F-T4.carries.hundreds_after", "38"),
            ("F-T4.carries.left", "44"),
            ("F-T4.twist.digit_pairs", "30,25"),
            ("F-T4.twist.same_two_numbers", "true"),
            ("F-M.n50.tall", "47"),
            ("F-M.n50.wide", "53"),
            ("F-N.estimate", "70,80,5600"),
            ("F-N.round_fix.fix", "134"),
            ("F-N.has_whole_middle", "false"),
            ("F-C4.karatsuba.sums", "50"),
            ("F-C4.karatsuba.strips_in_tens", "24"),
            ("F-C4.karatsuba.product", "1058"),
        ] {
            assert_eq!(f.get(key), Some(want), "{key}");
        }
    }
}
