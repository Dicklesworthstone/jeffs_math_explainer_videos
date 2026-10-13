//! Every number Video F shows or says, computed here and checked two ways:
//! `pin` against a literal from the fact sheet (the owner's brief, or the
//! source a cited number comes from), and `./x mental_math facts-check`
//! against `oracle.py` (same keys). The tricks are worked by `model.rs` the
//! way the video works them; the oracle gets every answer by plain
//! multiplication instead.

use explainer_kit::Facts;

use crate::model::{
    self, Ask, FIFTY, Piece, QUIZZES, SquareCut, Trick, blocks, by_parts, ends_in_five, estimate, halve_and_double,
    money, near_fifty, pair, paper, percent_of, round_then_fix, times_five, times_twenty_five,
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
    /// Karatsuba and Ofman, Doklady Akademii Nauk SSSR 145 (1962): a
    /// product of two two-part numbers from three multiplications, not four.
    pub const KARATSUBA_YEAR: u32 = 1962;
    pub const KARATSUBA_MULTIPLICATIONS: usize = 3;
}

/// The worked examples the chapters draw (the quizzes are `model::QUIZZES`).
pub mod shown {
    /// The cold open and the paper method.
    pub const HOOK: u64 = 85;
    /// The first square, the one whose picture is cut and moved.
    pub const FIVE: u64 = 35;
    /// Where the trick stops being easy: 13 × 14.
    pub const FIVE_HARD: u64 = 135;
    /// The same trick with a decimal point: seven and a half.
    pub const FIVE_TENTHS: u64 = 75;
    /// The owner's product by parts.
    pub const PARTS: (u64, u64) = (23, 46);
    pub const PAIR: (u64, u64) = (47, 53);
    /// A product with no whole middle.
    pub const NO_PAIR: (u64, u64) = (47, 56);
    /// Squares near 50: Bethe's, one above, and one whose corner carries.
    pub const BELOW: u64 = 48;
    pub const ABOVE: u64 = 53;
    pub const CARRIES: u64 = 62;
    /// A restaurant bill, in cents.
    pub const BILL: u64 = 4800;
    pub const SWAP: (u64, u64) = (8, 25);
    /// A percent the swap does not help with.
    pub const NO_SWAP: (u64, u64) = (17, 43);
    pub const TIMES_FIVE: u64 = 86;
    pub const TIMES_TWENTY_FIVE: u64 = 36;
    pub const HALVE: (u64, u64) = (14, 35);
    /// Both odd: nothing to halve.
    pub const NO_HALVE: (u64, u64) = (15, 37);
    pub const NINES: (u64, u64) = (9, 37);
    /// Four things at $19.99.
    pub const PRICE: (u64, u64) = (4, 1999);
}

fn list<T: ToString>(items: impl IntoIterator<Item = T>) -> String {
    items.into_iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")
}

fn sides(p: &Piece) -> String {
    format!("{}x{}", p.height, p.width)
}

/// A square of side `a` after a strip `d` wide has moved from one side to
/// the next: the rectangle it leaves, and the small square.
pub fn moved_strip(a: u64, d: u64) -> (u64, u64, u64) {
    (a - d, a + d, d * d)
}

pub fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    hook(&mut f)?;
    fives(&mut f)?;
    parts(&mut f)?;
    pairs(&mut f)?;
    fifty(&mut f)?;
    percents(&mut f)?;
    twos(&mut f)?;
    rounding(&mut f)?;
    one_move(&mut f)?;
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
    f.add("F-H1.paper.product", p.product)?;
    let e = ends_in_five(shown::HOOK).ok_or("the hook's number does not end in 5")?;
    f.add("F-H1.trick.front", e.n)?;
    f.add("F-H1.trick.next", e.n + 1)?;
    f.add("F-H1.trick.hundreds", e.hundreds)?;
    f.pin("F-H1.trick.square", e.square, "7225", "the order: 85^2 = 7225")?;
    Ok(())
}

/// F-T1: squares of numbers ending in 5, and the picture behind them.
fn fives(f: &mut Facts) -> Result<(), String> {
    let order = "the order";
    for (number, want) in [
        (shown::FIVE, "1225"),
        (105, "11025"),
        (shown::FIVE_HARD, "18225"),
        (5, "25"),
    ] {
        let e = ends_in_five(number).ok_or("an example does not end in 5")?;
        let key = format!("F-T1.n{number}");
        f.add(&format!("{key}.front"), e.n)?;
        f.add(&format!("{key}.next"), e.n + 1)?;
        f.add(&format!("{key}.hundreds"), e.hundreds)?;
        f.pin(&format!("{key}.square"), e.square, want, order)?;
        // the picture: the block with both strips beside it, and the corner
        let (tens, ones) = model::parts(number);
        let cut = SquareCut {
            base: tens,
            extra: ones,
        };
        let (h, w) = cut.grown_block();
        f.add(&format!("{key}.grown"), format!("{h}x{w}"))?;
    }
    let (tens, ones) = model::parts(shown::FIVE);
    let cut = SquareCut {
        base: tens,
        extra: ones,
    };
    f.add("F-T1.cut.parts", list([tens, ones]))?;
    f.add("F-T1.cut.block", cut.block())?;
    f.add("F-T1.cut.strip", cut.strip())?;
    f.add("F-T1.cut.two_strips", 2 * cut.strip())?;
    f.add("F-T1.cut.corner", cut.corner())?;
    let (h, w) = cut.grown_block();
    f.add("F-T1.cut.grown", format!("{h}x{w}"))?;
    f.add("F-T1.cut.grown_area", h * w)?;
    f.add("F-T1.cut.hundreds_tiles", format!("{}x{}", h / 10, w / 10))?;
    f.add("F-T1.cut.total", cut.total())?;
    // seven and a half squared: the same digits with the point put back
    let e = ends_in_five(shown::FIVE_TENTHS).ok_or("75 ends in 5")?;
    f.pin("F-T1.half.squared", money(e.square), "56.25", "7.5 x 7.5")?;
    f.add("F-T1.half.front_product", e.hundreds)?;
    Ok(())
}

/// F-T2: 23 times 46 by parts.
fn parts(f: &mut Facts) -> Result<(), String> {
    let order = "the order";
    let (a, b) = shown::PARTS;
    let p = by_parts(a, b);
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
    f.add("F-T2.estimate", list(<[u64; 3]>::from(estimate(a, b))))?;
    f.add("F-T2.held.pieces", p.pieces.len())?;
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
    // the picture: the strip that sticks out past the square, moved on top
    f.add("F-T3.strip", format!("{}x{}", p.distance, p.low))?;
    f.add("F-T3.strip_area", p.distance * p.low)?;
    f.add("F-T3.gap", format!("{}x{}", p.distance, p.middle))?;
    f.add("F-T3.gap_area", p.distance * p.middle)?;
    let (x, y) = shown::NO_PAIR;
    f.add("F-T3.no_middle", pair(x, y).is_none())?;
    // its middle, in halves: 103 halves is fifty-one and a half
    f.add("F-T3.no_middle.sum", x + y)?;
    Ok(())
}

/// F-T4: squares near 50.
fn fifty(f: &mut Facts) -> Result<(), String> {
    f.add("F-T4.fifty_squared", FIFTY * FIFTY)?;
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
        let strip = FIFTY * n.distance.unsigned_abs();
        f.add(&format!("{key}.strip"), strip)?;
        f.add(&format!("{key}.two_strips"), 2 * strip)?;
    }
    let above = near_fifty(shown::ABOVE);
    let cut = SquareCut {
        base: FIFTY,
        extra: above.distance.unsigned_abs(),
    };
    let (long, wide) = cut.long_strip();
    f.add("F-T4.above.long_strip", format!("{long}x{wide}"))?;
    f.pin("F-T4.above.total", cut.total(), "2809", "the order")?;
    let (front, back) = above.digit_pairs().ok_or("the corner carries")?;
    f.add("F-T4.above.digit_pairs", format!("{front},{back}"))?;
    let (front, back) = near_fifty(shown::BELOW).digit_pairs().ok_or("the corner carries")?;
    f.add("F-T4.below.digit_pairs", format!("{front},{back}"))?;
    Ok(())
}

/// F-T5: percents by blocks, and the swap.
fn percents(f: &mut Facts) -> Result<(), String> {
    let b = blocks(shown::BILL).ok_or("the bill's blocks are not whole cents")?;
    f.add("F-T5.bill", money(b.cents))?;
    f.add("F-T5.bill.ten", money(b.ten))?;
    f.add("F-T5.bill.five", money(b.five))?;
    f.add("F-T5.bill.fifteen", money(b.fifteen))?;
    f.add("F-T5.bill.twenty", money(b.twenty))?;
    let (x, y) = shown::SWAP;
    f.add("F-T5.swap.area", x * y)?;
    f.pin(
        "F-T5.swap.one_way",
        percent_of(x, y).ok_or("not whole")?,
        "2",
        "the order: 8% of 25",
    )?;
    f.add("F-T5.swap.other_way", percent_of(y, x).ok_or("not whole")?)?;
    // seventeen percent of forty-three, in hundredths: no whole answer either way
    let (x, y) = shown::NO_SWAP;
    f.add("F-T5.no_swap", money(x * y))?;
    f.add(
        "F-T5.no_swap.is_whole",
        percent_of(x, y).is_some() || percent_of(y, x).is_some(),
    )?;
    Ok(())
}

/// F-T6: trading a factor of two.
fn twos(f: &mut Facts) -> Result<(), String> {
    let (ten, half) = times_five(shown::TIMES_FIVE);
    f.add("F-T6.five.ten", ten)?;
    f.add("F-T6.five.product", half)?;
    let (hundred, half, quarter) = times_twenty_five(shown::TIMES_TWENTY_FIVE);
    f.add("F-T6.twenty_five.hundred", hundred)?;
    f.add("F-T6.twenty_five.half", half)?;
    f.add("F-T6.twenty_five.product", quarter)?;
    let (a, b) = shown::HALVE;
    let (half, double, product) = halve_and_double(a, b).ok_or("nothing to halve")?;
    f.add("F-T6.halve.sides", format!("{half}x{double}"))?;
    f.pin("F-T6.halve.product", product, "490", "the order: 14 x 35 = 7 x 70")?;
    let (a, b) = shown::NO_HALVE;
    f.add(
        "F-T6.no_halve",
        halve_and_double(a, b).is_none() && halve_and_double(b, a).is_none(),
    )?;
    Ok(())
}

/// F-T7: round, then fix.
fn rounding(f: &mut Facts) -> Result<(), String> {
    // nine of them is ten of them less one
    let (nine, each) = shown::NINES;
    let r = round_then_fix(each, nine, nine.next_multiple_of(10));
    f.add("F-T7.nines.rounded", r.rounded)?;
    f.add("F-T7.nines.fix", r.fix)?;
    f.add("F-T7.nines.product", r.total)?;
    let (count, cents) = shown::PRICE;
    let r = round_then_fix(count, cents, cents.next_multiple_of(100));
    f.add("F-T7.price.each", money(r.each))?;
    f.add("F-T7.price.round", money(r.round))?;
    f.add("F-T7.price.rounded", money(r.rounded))?;
    f.add("F-T7.price.fix", money(r.fix))?;
    f.add("F-T7.price.total", money(r.total))?;
    Ok(())
}

/// F-T9: the one move behind three of the tricks (chapter 14): a square is
/// a rectangle and a small square, a² = (a − d)(a + d) + d².
fn one_move(f: &mut Facts) -> Result<(), String> {
    for (a, d) in [(shown::FIVE, 5), (shown::BELOW, FIFTY - shown::BELOW)] {
        let (low, high, corner) = moved_strip(a, d);
        f.add(&format!("F-T9.n{a}.rectangle"), format!("{low}x{high}"))?;
        f.add(&format!("F-T9.n{a}.rectangle_area"), low * high)?;
        f.add(&format!("F-T9.n{a}.corner"), corner)?;
        f.add(&format!("F-T9.n{a}.square"), low * high + corner)?;
    }
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
    // the last quiz has nothing to grab: its estimate, and what a slipped
    // decimal point on a calculator would show
    let last = QUIZZES.last().ok_or("no quizzes")?;
    let (Trick::ByParts, Ask::Times(a, b)) = (last.trick, last.ask) else {
        return Err("the last quiz is a product by parts".into());
    };
    let key = format!("F-Q.{}", last.line);
    f.add(&format!("{key}.estimate"), list(<[u64; 3]>::from(estimate(a, b))))?;
    f.add(&format!("{key}.slipped_low"), format!("{}.{}", a * b / 10, a * b % 10))?;
    f.add(&format!("{key}.slipped_high"), a * b * 10)?;
    f.add(&format!("{key}.middle_halves"), a + b)?;
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
/// piece's size is its leading digits and its zeros, and the running total
/// never goes down.
pub fn edge_by_parts() -> (usize, usize) {
    tally((1..100u64).flat_map(|a| (1..100u64).map(move |b| (a, b))), |&(a, b)| {
        let p = by_parts(a, b);
        p.product() == a * b
            && p.running.is_sorted()
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

/// Every whole number from 0 to 100: fifty squared, a hundred times the
/// distance, the distance squared.
pub fn edge_near_fifty() -> (usize, usize) {
    tally(0..=100u64, |&x| near_fifty(x).square == x * x)
}

/// The numbers whose square really is "25 plus the distance, then the
/// distance squared as two digits": first and last.
pub fn digit_pairs_range() -> (u64, u64) {
    let reads = |x: &u64| {
        near_fifty(*x)
            .digit_pairs()
            .is_some_and(|(front, back)| format!("{front}{back}") == (x * x).to_string())
    };
    let ok: Vec<u64> = (0..=100u64).filter(reads).collect();
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

/// Every a from 1 to 200 and every d from 0 to a: the moved strip leaves a
/// rectangle and a corner that make the square.
pub fn edge_one_move() -> (usize, usize) {
    tally((1..=200u64).flat_map(|a| (0..=a).map(move |d| (a, d))), |&(a, d)| {
        let (low, high, corner) = moved_strip(a, d);
        low * high + corner == a * a
    })
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
    let e = ends_in_five(number).ok_or("Chase's square does not end in 5")?;
    f.pin("F-C3.chase.square", e.square, &square.to_string(), chase)?;
    let (x, y, product) = cited::CHASE_PAIR;
    let p = pair(x, y).ok_or("Chase's pair has no middle")?;
    f.pin("F-C3.chase.pair", p.product, &product.to_string(), chase)?;
    let (number, square) = cited::CHASE_NEAR_FIFTY;
    f.pin(
        "F-C3.chase.near_fifty",
        near_fifty(number).square,
        &square.to_string(),
        chase,
    )?;
    let karatsuba = "Karatsuba and Ofman 1962";
    f.pin("F-C4.karatsuba.year", cited::KARATSUBA_YEAR, "1962", karatsuba)?;
    // the block, the corner, and one product of the two sums for both strips
    let (a, b) = shown::PARTS;
    let ((a1, a0), (b1, b0)) = (model::parts(a), model::parts(b));
    let products = [a1 * b1, a0 * b0, (a1 + a0) * (b1 + b0)];
    f.pin(
        "F-C4.karatsuba.multiplications",
        products.len(),
        &cited::KARATSUBA_MULTIPLICATIONS.to_string(),
        karatsuba,
    )?;
    let strips = products[2] - products[0] - products[1];
    f.add("F-C4.karatsuba.strips", strips)?;
    f.add("F-C4.karatsuba.product", products[0] + strips + products[1])?;
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
        // the one move with nothing moved is the square itself
        assert_eq!(moved_strip(35, 0), (35, 35, 0));
    }

    /// Pattern 1 (narration truth) for the numbers the brief promises.
    #[test]
    fn the_brief_s_worked_examples() {
        let f = facts().unwrap();
        for (key, want) in [
            ("F-H1.paper.rows", "425,6800"),
            ("F-H1.paper.row_count", "2"),
            ("F-H1.paper.carries", "3"),
            ("F-T1.cut.grown", "30x40"),
            ("F-T1.cut.hundreds_tiles", "3x4"),
            ("F-T1.n105.grown", "100x110"),
            ("F-T4.below.digit_pairs", "23,04"),
            ("F-T4.above.digit_pairs", "28,09"),
            ("F-T5.bill.fifteen", "7.20"),
            ("F-T5.no_swap", "7.31"),
            ("F-T6.five.product", "430"),
            ("F-T6.twenty_five.product", "900"),
            ("F-T7.nines.product", "333"),
            ("F-T7.price.total", "79.96"),
            ("F-T9.n35.rectangle", "30x40"),
            ("F-T9.n48.rectangle", "46x50"),
            ("F-T9.n48.corner", "4"),
            ("F-C4.karatsuba.product", "1058"),
        ] {
            assert_eq!(f.get(key), Some(want), "{key}");
        }
    }
}
