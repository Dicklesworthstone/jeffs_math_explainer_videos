//! Every number the video shows or says, computed from [`crate::model`] and
//! printed for [`oracle.py`](../oracle.py) (`./x german_tank facts-check`).
//! Fact group G-Fn belongs to chapter n (docs/storyboard.md and
//! docs/storyboard-draft.md): G-F1 to G-F3 the opening act (the five boxes,
//! the computed experiment, the gaps), G-F4 to G-F7 the proof, G-F8 to G-F13
//! what the recipe can't promise, G-F14 to G-F16 the broken assumptions and
//! the return. The model uses closed forms; the oracle enumerates handfuls,
//! so the two computations are independent. Pins are the primary source's
//! own formulas applied to the friends' handful.

use explainer_kit::{Facts, choose};

use crate::model::{
    self, AGAIN_STREAM, BIG, BIG_STREAM, BIN_STREAM, FRIENDS, KNOWN, LINE_DOTS, Q, SIXTH, SMALL, SMALL_STREAM, csv,
    doubling_of, doubling_recipe, drawn_handfuls, gap_of, gap_recipe, gaps, rows,
};

/// The primary source, as the pins cite it.
const CGM: &str = "Clark, Gonye and Miller, Lessons from the German Tank Problem (arXiv:2101.08162v2), p. 3";

/// The handful size throughout.
pub const K: u64 = 5;

/// How many handfuls each drawn experiment shows, and how many fair
/// handfuls from chapter 02's bin chapter 04 lines up (the bin's own first).
pub const DRAWS: usize = 10;
pub const AGAIN_DRAWS: usize = 20;
pub const GHOSTS: usize = 6;

/// The round numbers chapter 02 says the big company passes: every row's
/// largest number reaches past [`OVER`] (02_5), and the average spacing is
/// over [`SPACING_OVER`] (02_8).
pub const OVER: u64 = 600;
pub const SPACING_OVER: u64 = 150;

/// The far-off count 04_1 names beside the largest number and the estimate.
pub const FAR: u64 = 500;

/// The tiny company of chapters 05 to 07, 12 and 13: five boxes, two seen.
pub const TINY: u64 = 5;
pub const TINY_SEEN: usize = 2;

/// The one box of chapter 07's last prediction (illustrative: no
/// claim rests on its value).
pub const ONE_BOX: u64 = 37;

/// Chapter 10's strange handful and chapter 14's late shelf.
pub const SILLY: [u64; 5] = [2, 3, 4, 5, 60];
pub const MADE: u64 = 200;
pub const LATE_FROM: u64 = 151;
pub const LATE: [u64; 5] = [158, 166, 173, 189, 200];

/// The round unit 08_3 says the stack topped by the count passes ("over
/// nine hundred thousand").
const TOPPED_ROUND: i128 = 100_000;

/// A doubling estimate moves 2/K for each unit of the sum, a gap estimate
/// (K + 1)/K for each unit of the largest number: so (K + 1)/2 neighbouring
/// sums make one of chapter 10's purple bars as wide as a green one.
pub const SUMS_PER_BAR: u64 = K.div_ceil(2);

/// The larger companies 11_3 tries on pairs, and the round unit 11_4 says
/// the groups of four pass ("over four hundred and fifty thousand").
pub const PAIR_MORE: [u64; 2] = [6, 8];
pub const GROUPS_ROUND: u128 = 10_000;

/// Chapter 12 checks each group's average in every company of pairs up to
/// [`GROUP_COMPANIES_TO`] boxes; chapter 13 lists the counts up to
/// [`LOSES_UP_TO`] where the 7/6 recipe does not miss less than the gap
/// recipe.
const GROUP_COMPANIES_TO: u64 = 12;
const LOSES_UP_TO: u64 = 300;

/// The second count 09_2 tries, and the larger handful of 09's last
/// prediction.
const SECOND_COUNT: u64 = 100;
const TEN: u64 = 10;

/// The one-in-twenty level of the 95% range.
pub fn level() -> Q {
    Q::new(1, 20)
}

fn flag(b: bool) -> &'static str {
    if b { "true" } else { "false" }
}

fn int(n: u64) -> Q {
    Q::int(n as i128)
}

/// The average spacing of neighbouring numbers in a fair handful of k from
/// 1..n, counting from zero: one more than a gap's average size.
pub fn spacing(n: u64, k: u64) -> Q {
    Q::new(n as i128 - k as i128, k as i128 + 1) + Q::ONE
}

/// Chapter 02's handfuls from a thousand and from seventy, and the five that
/// leave the bin.
pub fn big_handfuls() -> Vec<Vec<u64>> {
    drawn_handfuls(BIG_STREAM, BIG, K as usize, DRAWS)
}

pub fn small_handfuls() -> Vec<Vec<u64>> {
    drawn_handfuls(SMALL_STREAM, SMALL, K as usize, DRAWS)
}

pub fn bin_handful() -> Vec<u64> {
    drawn_handfuls(BIN_STREAM, LINE_DOTS, K as usize, 1).swap_remove(0)
}

/// Chapter 08's twenty fresh handfuls from the 71-box company.
pub fn again_handfuls() -> Vec<Vec<u64>> {
    drawn_handfuls(AGAIN_STREAM, KNOWN, K as usize, AGAIN_DRAWS)
}

/// Chapter 10's purple bars from the handfuls counted by sum (`ways`, index
/// = sum): bar m holds the [`SUMS_PER_BAR`] sums nearest SUMS_PER_BAR·m,
/// whose doubling estimates sit under the green bar of the gap recipe's
/// value for largest m. Each non-empty bar as (m, handfuls), in order.
pub fn purple_bars(ways: &[u128]) -> Vec<(u64, u128)> {
    let mut bars: Vec<(u64, u128)> = Vec::new();
    for (s, &w) in ways.iter().enumerate().filter(|&(_, &w)| w > 0) {
        let m = Q::new(s as i128, SUMS_PER_BAR as i128).nearest() as u64;
        match bars.last_mut() {
            Some((b, c)) if *b == m => *c += w,
            _ => bars.push((m, w)),
        }
    }
    bars
}

/// The algebra of chapter 13's staircase, as identities of polynomials
/// checked by the degree bound (a polynomial of degree at most d in N that
/// is zero at d + 1 counts is zero at every count). For k = 1..=12, (a)
/// k·C(N, k) = N·C(N − 1, k − 1) and (b) k·C(N − 1, k) = (N − k)·C(N − 1,
/// k − 1) have degree k in N and are checked at N = 1..=k + 1 (there choose
/// is the polynomial's value, 0 below j for C(x, j)). Dividing the
/// recurrence C(N − 1, k − 1)·g(N) = N·C(N, k) − (N − 1)·C(N − 1, k) by
/// C(N − 1, k − 1) with them gives k·g(N) = N² − (N − 1)(N − k), and (c)
/// that is (k + 1)N − k, (d) k times the gap recipe N + (N − k)/k: degree 2
/// in N and 1 in k, checked on N = 0, 1, 2 by k = 0, 1.
fn staircase_algebra_holds() -> bool {
    let c = |x: u64, j: u64| Q::count(choose(x, j));
    let fixed_k = (1..=12u64).all(|k| {
        (1..=k + 1).all(|n| {
            let (kq, nq) = (int(k), int(n));
            kq * c(n, k) == nq * c(n - 1, k - 1) && kq * c(n - 1, k) == (nq - kq) * c(n - 1, k - 1)
        })
    });
    let grid = (0..=2i128).all(|n| {
        (0..=1i128).all(|k| n * n - (n - 1) * (n - k) == (k + 1) * n - k && k * n + (n - k) == (k + 1) * n - k)
    });
    fixed_k && grid
}

/// A handful's gap list with its top gap, up to a known count `n`.
pub fn gap_list(hand: &[u64], n: u64) -> Vec<u64> {
    let mut g = gaps(hand);
    g.push(n - hand.last().expect("a handful has a box"));
    g
}

pub fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    let m = *FRIENDS.last().expect("five numbers");

    // 01: the five boxes, and the one certain thing
    f.add("G-F1.numbers", csv(&FRIENDS))?;
    let first: u64 = 1;
    f.add("G-F1.first", first)?;
    // counting up with nothing skipped: the box made after the first
    f.add("G-F1.second", first + 1)?;
    f.add("G-F1.handful", FRIENDS.len())?;
    f.add("G-F1.largest", m)?;
    let at_least = (1..)
        .find(|&n| FRIENDS.iter().all(|&x| x <= n))
        .expect("a bound exists");
    f.add("G-F1.at_least", at_least)?;
    f.add("G-F1.thousand", BIG)?;
    // the zoomed line's marks, from the same extent the line zooms to
    f.add("G-F1.zoom_marks", csv(&model::marks(first, BIG, model::ZOOM_PARTS)?))?;

    // 02: fair handfuls from a big and a small company
    let (big, small) = (big_handfuls(), small_handfuls());
    f.add("G-F2.seed", model::SEED)?;
    f.add("G-F2.big", BIG)?;
    f.add("G-F2.small", SMALL)?;
    f.add("G-F2.draws", big.len())?;
    f.add("G-F2.big_handfuls", rows(&big))?;
    let lowest_largest = big.iter().map(|h| h[h.len() - 1]).min().ok_or("no draws")?;
    f.add("G-F2.big_lowest_largest", lowest_largest)?;
    f.add("G-F2.over", OVER)?;
    f.add("G-F2.big_largest_over_600", flag(lowest_largest > OVER))?;
    f.add("G-F2.small_handfuls", rows(&small))?;
    let p_big = model::all_at_most(m, BIG, K);
    f.add("G-F2.big_all_at_most_60", p_big)?;
    f.add(
        "G-F2.big_all_at_most_60_under_one_in_a_million",
        flag(p_big < Q::new(1, 1_000_000)),
    )?;
    let p_small = model::all_at_most(m, SMALL, K);
    f.add("G-F2.small_all_at_most_60.percent", (Q::int(100) * p_small).nearest())?;
    // one draw's count of the ten rows from seventy that fit (02_7), not
    // the chance above
    let fit_shown = small.iter().filter(|h| h[h.len() - 1] <= m).count();
    f.add("G-F2.small_fit_shown", fit_shown)?;
    f.add("G-F2.spacing_over", SPACING_OVER)?;
    f.add("G-F2.big_spacing", spacing(BIG, K))?;
    f.add("G-F2.big_spacing_over_150", flag(spacing(BIG, K) > int(SPACING_OVER)))?;
    f.add("G-F2.small_spacing", spacing(SMALL, K))?;
    f.add("G-F2.small_spacing_about", spacing(SMALL, K).nearest())?;
    f.add("G-F2.friends_spacing", Q::new(m as i128, K as i128))?;
    f.add("G-F2.bin_dots", LINE_DOTS)?;
    f.add("G-F2.bin_handful", csv(&bin_handful()))?;
    f.add("G-F2.big_row_spacing", Q::new(big[0][4] as i128, K as i128).decimal1()?)?;
    f.add(
        "G-F2.small_row_spacing",
        Q::new(small[0][4] as i128, K as i128).decimal1()?,
    )?;
    f.add("G-F2.big_spacing_shown", spacing(BIG, K).about1()?)?;
    f.add("G-F2.small_spacing_shown", spacing(SMALL, K).about1()?)?;
    f.add("G-F2.million", 1_000_000)?;

    // 03: the gaps, one at a time, and the first estimate
    let g = gaps(&FRIENDS);
    let unseen: u64 = g.iter().sum();
    f.add("G-F3.gaps", csv(&g))?;
    f.add("G-F3.unseen", unseen)?;
    f.add("G-F3.gap_count", g.len())?;
    f.add("G-F3.average_gap", Q::new(unseen as i128, g.len() as i128))?;
    // the source's formula, m(1 + 1/k) − 1, for m = 60 and k = 5
    let cgm = int(m) * (Q::ONE + Q::new(1, K as i128)) - Q::ONE;
    f.pin("G-F3.estimate", gap_recipe(&FRIENDS), &cgm.to_string(), CGM)?;
    // what 03_1 to 03_3 say, one gap at a time
    f.add("G-F3.first_seen", FRIENDS[0])?;
    f.add("G-F3.second_seen", FRIENDS[1])?;
    f.add("G-F3.first_gap", g[0])?;
    f.add("G-F3.second_gap", g[1])?;
    f.add("G-F3.last_gaps", csv(&g[2..]))?;
    let known = &small[0];
    f.add("G-F3.known_handful", csv(known))?;
    f.add("G-F3.known_gaps", csv(&gaps(known)))?;
    f.add("G-F3.known_top_gap", SMALL - known[4])?;
    // the gaps above the largest number (03_5's "one more gap"), counted
    // where the top gap can be seen
    f.add("G-F3.top_gaps", gap_list(known, SMALL).len() - gaps(known).len())?;

    // 04: a count it might be (04_1), the fair handfuls from chapter 02's
    // bin that "every handful" lines up (its own first), and the edge
    // handful 1..5
    f.add("G-F4.far_count", FAR)?;
    f.add(
        "G-F4.ghost_handfuls",
        rows(&drawn_handfuls(BIN_STREAM, LINE_DOTS, K as usize, GHOSTS)),
    )?;
    let edge: Vec<u64> = (1..=K).collect();
    f.add("G-F4.edge_handful", csv(&edge))?;
    f.add("G-F4.edge_estimate", gap_recipe(&edge))?;
    f.add("G-F4.edge_gaps", csv(&gaps(&edge)))?;

    // 05: the 71-box company's handfuls, too many to look at (05_1 says the
    // whole millions, and that there are more), then the tiny company,
    // every pair
    let all71 = choose(KNOWN, K);
    let millions = all71 / 1_000_000;
    f.add("G-F5.company", KNOWN)?;
    f.add("G-F5.handfuls", all71)?;
    f.add("G-F5.millions", millions)?;
    f.add("G-F5.handfuls_over_millions", flag(all71 > millions * 1_000_000))?;
    let pairs = model::all_handfuls(TINY, TINY_SEEN);
    let tiny: Vec<Q> = pairs.iter().map(|p| gap_recipe(p)).collect();
    let ten = Q::int(pairs.len() as i128);
    f.add("G-F5.boxes", TINY)?;
    f.add("G-F5.seen", TINY_SEEN)?;
    f.add("G-F5.pairs", pairs.len())?;
    f.add("G-F5.pair_list", rows(&pairs))?;
    f.add("G-F5.first_pair", csv(&pairs[0]))?;
    f.add("G-F5.last_pair", csv(&pairs[pairs.len() - 1]))?;
    f.add("G-F5.estimates", csv(&tiny))?;
    // the table's column: a whole estimate as it is, a half with its one
    // decimal
    let shown: Result<Vec<String>, String> = tiny
        .iter()
        .map(|e| e.whole().map_or_else(|| e.decimal1(), |w| Ok(w.to_string())))
        .collect();
    f.add("G-F5.estimates_shown", shown?.join(","))?;
    f.add("G-F5.average", tiny.iter().fold(Q::ZERO, |s, &e| s + e) / ten)?;
    f.add(
        "G-F5.largest_average",
        Q::int(pairs.iter().map(|p| p[1] as i128).sum()) / ten,
    )?;
    // the worked pairs: one with no gaps, and one whose two gaps differ,
    // its unseen boxes below the largest named (05_3)
    let (low, high): ([u64; 2], [u64; 2]) = ([1, 2], [2, 5]);
    let high_unseen: Vec<u64> = (1..high[1]).filter(|n| !high.contains(n)).collect();
    f.add("G-F5.example_low_pair", csv(&low))?;
    f.add("G-F5.example_low", gap_recipe(&low))?;
    f.add("G-F5.example_high_pair", csv(&high))?;
    f.add("G-F5.example_high_largest", high[1])?;
    f.add("G-F5.example_high_unseen", csv(&high_unseen))?;
    f.add("G-F5.example_high_unseen_count", high_unseen.len())?;
    f.add("G-F5.example_high_gaps", csv(&gaps(&high)))?;
    f.add("G-F5.example_high", gap_recipe(&high))?;
    f.add("G-F5.example_high_decimal", gap_recipe(&high).decimal1()?)?;

    // 06: gaps that trade places
    let lists: Vec<Vec<u64>> = pairs.iter().map(|p| gap_list(p, TINY)).collect();
    let positions = TINY_SEEN + 1;
    f.add("G-F6.gap_lists", rows(&lists))?;
    f.add("G-F6.unseen", TINY - TINY_SEEN as u64)?;
    f.add("G-F6.positions", positions)?;
    let averages: Vec<Q> = (0..positions)
        .map(|i| Q::int(lists.iter().map(|l| l[i] as i128).sum()) / ten)
        .collect();
    f.add("G-F6.position_averages", csv(&averages))?;
    let mut sorted = lists.clone();
    sorted.sort();
    f.add(
        "G-F6.one_pair_per_list",
        flag(sorted == model::splits(TINY - TINY_SEEN as u64, positions)),
    )?;
    f.add("G-F6.friends_positions", FRIENDS.len() + 1)?;
    f.add("G-F6.friends_seen_gaps", g.len())?;
    // the worked rows' lists (06_2, 06_3); the first row's list with its
    // gaps below and above traded, and the pair whose list that is (06_5);
    // each row's partner by that trade, rows counted from 1 (06_6); each
    // column's sum and its values in order (06_6 to 06_8); one list's
    // chance; the top gap's average
    let row_of = |p: &[u64]| {
        pairs
            .iter()
            .position(|q| q.as_slice() == p)
            .ok_or("a pair of the table")
    };
    let low_list = lists[row_of(&low)?].clone();
    f.add("G-F6.low_gaps", csv(&low_list))?;
    f.add("G-F6.high_gaps", csv(&lists[row_of(&high)?]))?;
    let traded = |l: &[u64]| {
        let mut t = l.to_vec();
        t.swap(0, positions - 1);
        t
    };
    let list_row = |l: &[u64]| {
        lists
            .iter()
            .position(|m| m.as_slice() == l)
            .ok_or("a list of the table")
    };
    f.add("G-F6.low_traded", csv(&traded(&low_list)))?;
    f.add("G-F6.low_partner", csv(&pairs[list_row(&traded(&low_list))?]))?;
    let partners: Vec<usize> = lists
        .iter()
        .map(|l| list_row(&traded(l)).map(|j| j + 1))
        .collect::<Result<_, _>>()?;
    f.add("G-F6.partners", csv(&partners))?;
    let column = |i: usize| -> Vec<u64> {
        let mut c: Vec<u64> = lists.iter().map(|l| l[i]).collect();
        c.sort();
        c
    };
    f.add(
        "G-F6.column_sums",
        csv(&(0..positions).map(|i| column(i).iter().sum()).collect::<Vec<u64>>()),
    )?;
    f.add("G-F6.below_values", csv(&column(0)))?;
    f.add("G-F6.between_values", csv(&column(1)))?;
    f.add("G-F6.above_values", csv(&column(positions - 1)))?;
    f.add("G-F6.each_chance", Q::ONE / ten)?;
    f.add("G-F6.top_average", averages[positions - 1])?;

    // 07: the general proof's pieces, and the one-box handful
    f.add("G-F7.gap_average_formula", format!("(N-{K})/{}", K + 1))?;
    let slope = gap_recipe(&[7]) - gap_recipe(&[6]);
    f.add(
        "G-F7.one_box_recipe",
        format!("{slope}m{}", gap_recipe(&[7]) - slope * Q::int(7)),
    )?;
    let unbiased = |k: u64, n: u64| model::mean_by_largest(n, k, |mm| gap_of(mm, k)) == int(n);
    f.add("G-F7.one_box_unbiased", flag((1..30).all(|n| unbiased(1, n))))?;
    f.add(
        "G-F7.general_unbiased",
        flag((1..=5).all(|k| (k..22).all(|n| unbiased(k, n)))),
    )?;
    // the tiny company again (07_5, 07_6): each row's guess for its top
    // gap, the average of its gaps below the larger number, shown as the
    // table shows estimates; its top gap; the rows where the two agree; and
    // both averages over the ten
    let guesses: Vec<Q> = lists
        .iter()
        .map(|l| Q::new(l[..TINY_SEEN].iter().sum::<u64>() as i128, TINY_SEEN as i128))
        .collect();
    let tops: Vec<u64> = lists.iter().map(|l| l[TINY_SEEN]).collect();
    f.add("G-F7.tiny_guesses", csv(&guesses))?;
    let guesses_shown: Result<Vec<String>, String> = guesses
        .iter()
        .map(|e| e.whole().map_or_else(|| e.decimal1(), |w| Ok(w.to_string())))
        .collect();
    f.add("G-F7.tiny_guesses_shown", guesses_shown?.join(","))?;
    f.add("G-F7.tiny_tops", csv(&tops))?;
    let agree = guesses.iter().zip(&tops).filter(|&(&q, &t)| q == int(t)).count();
    f.add("G-F7.tiny_agree", agree)?;
    f.add(
        "G-F7.tiny_guess_average",
        guesses.iter().fold(Q::ZERO, |s, &q| s + q) / ten,
    )?;
    f.add("G-F7.tiny_top_average", int(tops.iter().sum()) / ten)?;
    // the one-box handful (07_8, 07_9): its size, its number, the gap below
    // it, how many gaps one box leaves (its list with the top gap, for any
    // count from its number on) and its estimate
    let one = [ONE_BOX];
    f.add("G-F7.one_box_handful", one.len())?;
    f.add("G-F7.one_box", ONE_BOX)?;
    f.add("G-F7.one_box_below", gaps(&one)[0])?;
    f.add("G-F7.one_box_gaps", gap_list(&one, ONE_BOX).len())?;
    f.add("G-F7.one_box_estimate", gap_recipe(&one))?;

    // 08: the 71-box company, every handful by its largest number
    let n71 = KNOWN;
    let est = |mm: u64| gap_of(mm, K);
    f.add("G-F8.count", n71)?;
    f.add("G-F8.handfuls", choose(n71, K))?;
    let mean71 = model::mean_by_largest(n71, K, est);
    f.add("G-F8.average", mean71)?;
    let over = model::share_by_largest(n71, K, |mm| est(mm) > int(n71));
    f.add("G-F8.overestimate_share", over)?;
    f.add("G-F8.overestimate_percent", (Q::int(100) * over).nearest())?;
    let exact = model::share_by_largest(n71, K, |mm| est(mm) == int(n71));
    f.add("G-F8.exactly_right", exact)?;
    f.add("G-F8.exactly_right_one_in", (Q::ONE / exact).nearest())?;
    f.add("G-F8.highest_estimate", est(n71).decimal1()?)?;
    f.add("G-F8.lowest_estimate", est(K))?;
    let mae71 = model::mean_by_largest(n71, K, |mm| (est(mm) - int(n71)).abs());
    f.add("G-F8.average_miss", mae71)?;
    f.add("G-F8.average_miss_about", mae71.nearest())?;
    let again = again_handfuls();
    let again_est: Vec<Q> = again.iter().map(|h| gap_recipe(h)).collect();
    let lo = *again_est.iter().min().ok_or("no draws")?;
    let hi = *again_est.iter().max().ok_or("no draws")?;
    let again_mean = again_est.iter().fold(Q::ZERO, |s, &e| s + e) / Q::int(again_est.len() as i128);
    f.add("G-F8.again_draws", again.len())?;
    f.add("G-F8.again_handfuls", rows(&again))?;
    let shown: Result<Vec<String>, String> = again_est.iter().map(|e| e.decimal1()).collect();
    f.add("G-F8.again_estimates", shown?.join(","))?;
    f.add("G-F8.again_lowest", lo.decimal1()?)?;
    f.add("G-F8.again_lowest_about", lo.nearest())?;
    f.add("G-F8.again_highest", hi.decimal1()?)?;
    f.add("G-F8.again_average", again_mean)?;
    f.add("G-F8.again_average_about", again_mean.nearest())?;
    f.add("G-F8.again_exact", again_est.iter().filter(|&&e| e == int(n71)).count())?;
    // 08_3: the handfuls by their largest number: a stack for each largest
    // number from five to the count, its size C(m − 1, 4)
    let topped = |mm: u64| model::with_largest(mm, K);
    f.add("G-F8.stacks", n71 - K + 1)?;
    f.add("G-F8.topped_by_smallest", topped(K))?;
    f.add("G-F8.next_largest", K + 1)?;
    f.add("G-F8.topped_by_next", topped(K + 1))?;
    let at_count = topped(n71).whole().ok_or("a whole count")?;
    f.add("G-F8.topped_by_count", at_count)?;
    f.add("G-F8.topped_by_count_over", at_count / TOPPED_ROUND * TOPPED_ROUND)?;
    // 08_6: how far each side of 71 misses on average; 08_7: the spread
    let side = |keep: &dyn Fn(Q) -> bool| {
        let (miss, many) = (K..=n71)
            .filter(|&mm| keep(est(mm)))
            .fold((Q::ZERO, Q::ZERO), |(s, c), mm| {
                (s + topped(mm) * (est(mm) - int(n71)).abs(), c + topped(mm))
            });
        miss / many
    };
    let overshoot = side(&|e| e > int(n71));
    f.add("G-F8.overshoot", overshoot)?;
    f.add("G-F8.overshoot_about", overshoot.nearest())?;
    f.add("G-F8.undershoot", side(&|e| e < int(n71)))?;
    f.add("G-F8.average_miss_decimal", mae71.about1()?)?;
    let var71 = model::mean_by_largest(n71, K, |mm| (est(mm) - int(n71)) * (est(mm) - int(n71)));
    f.add("G-F8.variance", var71)?;
    f.add("G-F8.sd_about", var71.sqrt_about1()?)?;

    // 09: how far off; the chance a fair handful tops out at 60 or less
    let counts = [n71, SECOND_COUNT];
    for n in counts {
        let p = model::all_at_most(m, n, K);
        f.add(&format!("G-F9.at_most_60.{n}"), p)?;
        f.add(&format!("G-F9.at_most_60.{n}.percent"), (Q::int(100) * p).nearest())?;
    }
    f.add("G-F9.second_count", SECOND_COUNT)?;
    let top = model::upper_bound(m, K, level());
    f.add("G-F9.level", level())?;
    f.add("G-F9.level_percent", (Q::int(100) * level()).nearest())?;
    f.add("G-F9.confidence_percent", (Q::int(100) * (Q::ONE - level())).nearest())?;
    f.add("G-F9.range", csv(&[m, top]))?;
    f.add("G-F9.range_top", top)?;
    f.add("G-F9.first_rare", top + 1)?;
    f.add(
        "G-F9.first_rare.percent_below",
        flag(model::all_at_most(m, top + 1, K) < level()),
    )?;
    for n in [top, top + 1] {
        let p = model::all_at_most(m, n, K);
        f.add(&format!("G-F9.at_most_60.{n}"), p)?;
        f.add(&format!("G-F9.at_most_60.{n}.percent1"), (Q::int(100) * p).about1()?)?;
    }
    f.add("G-F9.from_largest", choose(m, K))?;
    f.add("G-F9.from_range_top", choose(top, K))?;
    f.add("G-F9.ten_boxes", TEN)?;
    f.add("G-F9.ten_boxes_range", csv(&[m, model::upper_bound(m, TEN, level())]))?;
    f.add("G-F9.ten_boxes_top", model::upper_bound(m, TEN, level()))?;
    // the coverage experiment: the count fixed at 71, each of 08's twenty
    // handfuls builds its own range from its largest number
    let ranges: Vec<[u64; 2]> = again
        .iter()
        .map(|h| {
            let lo = *h.last().expect("a handful has a box");
            [lo, model::upper_bound(lo, K, level())]
        })
        .collect();
    let misses: Vec<u64> = ranges.iter().filter(|r| r[1] < n71).map(|r| r[0]).collect();
    let ranges_shown: Vec<String> = ranges.iter().map(|r| csv(r.as_slice())).collect();
    f.add("G-F9.again_ranges", ranges_shown.join(";"))?;
    f.add("G-F9.again_cover", ranges.len() - misses.len())?;
    f.add("G-F9.again_miss", misses.len())?;
    f.add("G-F9.again_miss_largest", csv(&misses))?;
    let cover71 = model::share_by_largest(n71, K, |mm| model::upper_bound(mm, K, level()) >= n71);
    f.add("G-F9.cover_71", cover71)?;
    f.add("G-F9.cover_71_percent", (Q::int(100) * cover71).nearest())?;
    f.add(
        "G-F9.worst_coverage_at_least_95",
        flag(model::worst_coverage(K, 150, level()) >= Q::new(19, 20)),
    )?;

    // 10: the doubling recipe, earned from the middle number. The numbers 1
    // to n balance at their middle, n(n + 1)/2 over n; the count read back
    // from the middle as a multiple of it (neighbouring counts give the
    // slope)
    let middle = |n: u64| Q::new((n * (n + 1) / 2) as i128, n as i128);
    f.add("G-F10.middle_71", middle(n71))?;
    let slope = Q::ONE / (middle(n71) - middle(n71 - 1));
    f.add(
        "G-F10.from_middle",
        format!("{slope}*middle{}", int(n71) - slope * middle(n71)),
    )?;
    // the doubling recipe read as a multiple of the handful's average: sums
    // K apart are averages one apart
    let total: u64 = FRIENDS.iter().sum();
    let average = Q::new(total as i128, K as i128);
    let slope = doubling_of(total + K, K) - doubling_of(total, K);
    f.add(
        "G-F10.recipe",
        format!("{slope}*average{}", doubling_of(total, K) - slope * average),
    )?;
    f.add("G-F10.sum", total)?;
    f.add("G-F10.average", average)?;
    f.add("G-F10.doubling", doubling_recipe(&FRIENDS))?;
    // over every handful of the 71-box company, by sum: a fair handful's
    // average averages the middle number, and doubling averages the count
    f.add(
        "G-F10.average_of_averages_71",
        model::mean_by_sum(n71, K as usize, |s| Q::new(s as i128, K as i128)),
    )?;
    let doubling_unbiased = |k: u64, n: u64| model::mean_by_sum(n, k as usize, |s| doubling_of(s, k)) == int(n);
    f.add(
        "G-F10.doubling_unbiased",
        flag((1..=5).all(|k| (k..17).all(|n| doubling_unbiased(k, n)))),
    )?;
    f.add("G-F10.others", K - 1)?;
    f.add("G-F10.silly_others", csv(&SILLY[..SILLY.len() - 1]))?;
    f.add("G-F10.silly_handful", csv(&SILLY))?;
    let silly_sum: u64 = SILLY.iter().sum();
    f.add("G-F10.silly_sum", silly_sum)?;
    f.add(
        "G-F10.silly_average_decimal",
        Q::new(silly_sum as i128, K as i128).decimal1()?,
    )?;
    f.add("G-F10.silly_doubling", doubling_recipe(&SILLY))?;
    f.add("G-F10.silly_doubling_decimal", doubling_recipe(&SILLY).decimal1()?)?;
    f.add("G-F10.silly_gap", gap_recipe(&SILLY))?;
    // how often doubling lands below the handful's own largest number,
    // counted by largest and sum
    let below = model::share_by_largest_and_sum(n71, K as usize, |mm, s| doubling_of(s, K) < int(mm));
    f.add("G-F10.below_largest_71", below)?;
    f.add("G-F10.below_largest_71_one_in", (Q::ONE / below).nearest())?;
    f.add("G-F10.below_largest_71_percent1", (Q::int(100) * below).about1()?)?;
    // the handfuls by sum: the doubling estimates' range and 10's purple bars
    let ways71 = model::handfuls_by_sum(n71, K as usize);
    let sums: Vec<u64> = ways71
        .iter()
        .enumerate()
        .filter(|&(_, &w)| w > 0)
        .map(|(s, _)| s as u64)
        .collect();
    let (lo_sum, hi_sum) = (sums[0], sums[sums.len() - 1]);
    f.add(
        "G-F10.doubling_range_71",
        csv(&[doubling_of(lo_sum, K), doubling_of(hi_sum, K)]),
    )?;
    let bars = purple_bars(&ways71);
    f.add("G-F10.bars_71", bars.len())?;
    let &(tallest_at, tallest) = bars.iter().max_by_key(|&&(_, c)| c).ok_or("no purple bars")?;
    f.add("G-F10.tallest_bar_71", tallest)?;
    f.add("G-F10.tallest_bar_71_at", gap_of(tallest_at, K))?;
    // 10_9: both widths of the doubling recipe at 71, beside the gap
    // recipe's from 08
    let off = |s: u64| doubling_of(s, K) - int(n71);
    f.add(
        "G-F10.doubling_average_71",
        model::mean_by_sum(n71, K as usize, |s| doubling_of(s, K)),
    )?;
    let dbl_mae = model::mean_by_sum(n71, K as usize, |s| off(s).abs());
    f.add("G-F10.doubling_miss_71", dbl_mae)?;
    f.add("G-F10.doubling_miss_71_about", dbl_mae.nearest())?;
    f.add("G-F10.doubling_miss_71_decimal", dbl_mae.about1()?)?;
    f.add("G-F10.gap_miss_71_about", mae71.nearest())?;
    let dbl_var = model::mean_by_sum(n71, K as usize, |s| off(s) * off(s));
    f.add("G-F10.doubling_variance_71", dbl_var)?;
    f.add("G-F10.doubling_sd_about", dbl_var.sqrt_about1()?)?;
    f.add("G-F10.variance_ratio", dbl_var / var71)?;

    // 11: given the largest, the others don't depend on the count. First on
    // pairs: the tiny company and two larger ones, the pairs topped by five
    let others_of = |n: u64, k: usize, top: u64| -> Vec<Vec<u64>> {
        model::all_handfuls(n, k)
            .into_iter()
            .filter(|h| h[k - 1] == top)
            .map(|h| h[..k - 1].to_vec())
            .collect()
    };
    let pair_top = TINY;
    let companies = [TINY, PAIR_MORE[0], PAIR_MORE[1]];
    let topped = others_of(pair_top, TINY_SEEN, pair_top);
    f.add("G-F11.pair_top", pair_top)?;
    f.add("G-F11.pair_companies", csv(&companies))?;
    f.add("G-F11.pair_more", csv(&PAIR_MORE))?;
    f.add(
        "G-F11.pair_more_pairs",
        csv(&PAIR_MORE.map(|n| choose(n, TINY_SEEN as u64))),
    )?;
    f.add("G-F11.pairs_topped", topped.len())?;
    f.add(
        "G-F11.pair_smaller",
        csv(&topped.iter().map(|o| o[0]).collect::<Vec<u64>>()),
    )?;
    // every handful of a fair draw is equally likely, so each pair topped by
    // five has chance one over how many there are, in every company
    let chances: Vec<Q> = companies
        .iter()
        .map(|&n| Q::ONE / Q::int(others_of(n, TINY_SEEN, pair_top).len() as i128))
        .collect();
    f.add(
        "G-F11.pair_each",
        if chances.iter().all(|&c| c == chances[0]) {
            chances[0].to_string()
        } else {
            "unequal".into()
        },
    )?;
    f.add(
        "G-F11.pair_same_for_every_company",
        flag(companies.iter().all(|&n| others_of(n, TINY_SEEN, pair_top) == topped)),
    )?;
    let shares: Vec<Q> = companies
        .iter()
        .map(|&n| model::share_by_largest(n, TINY_SEEN as u64, |mm| mm == pair_top))
        .collect();
    f.add("G-F11.pair_shares", csv(&shares))?;
    // (and on a smaller copy of the friends' case: largest 9 of four seen,
    // counts 9 to 16, listed whole)
    let base = others_of(9, 4, 9);
    let same = (9..=16).all(|n| others_of(n, 4, 9) == base) && base.len() as u128 == choose(8, 3);
    f.add("G-F11.others_same_for_every_count", flag(same))?;
    let groups = choose(m - 1, K - 1);
    f.add("G-F11.others_sets", groups)?;
    f.add("G-F11.others_sets_over", groups / GROUPS_ROUND * GROUPS_ROUND)?;
    // the chance that the largest of a fair handful is 60, at three counts,
    // and the chance of one particular group of four given it: one
    // handful's chance over that
    let counts = [m, n71, BIG];
    f.add("G-F11.counts", csv(&counts))?;
    let top_chance = |n: u64| model::share_by_largest(n, K, |mm| mm == m);
    for n in counts {
        f.add(&format!("G-F11.top_chance.{n}"), top_chance(n))?;
        f.add(
            &format!("G-F11.each_group.{n}"),
            Q::ONE / Q::count(choose(n, K)) / top_chance(n),
        )?;
    }
    f.add(
        &format!("G-F11.top_chance.{m}.one_in"),
        (Q::ONE / top_chance(m)).nearest(),
    )?;
    f.add(
        &format!("G-F11.top_chance.{n71}.one_in"),
        (Q::ONE / top_chance(n71)).nearest(),
    )?;
    f.add(
        &format!("G-F11.top_chance.{BIG}.millions"),
        (Q::ONE / top_chance(BIG) / Q::int(1_000_000)).nearest(),
    )?;
    // the gaps below sixty hold 55 numbers whatever the other four are
    f.add("G-F11.silly_gaps", csv(&gaps(&SILLY)))?;
    f.add("G-F11.silly_unseen", gaps(&SILLY).iter().sum::<u64>())?;

    // 12: the tiny company's ten pairs again, with the doubling recipe. For a
    // pair, twice the average is the total: the recipe read as a multiple of
    // the pair's total (two pairs give the slope)
    fn mean(v: &[Q]) -> Q {
        v.iter().fold(Q::ZERO, |s, &x| s + x) / Q::int(v.len() as i128)
    }
    fn miss(v: &[Q], count: Q) -> Q {
        mean(&v.iter().map(|&x| (x - count).abs()).collect::<Vec<_>>())
    }
    fn squared(v: &[Q], count: Q) -> Q {
        mean(&v.iter().map(|&x| (x - count) * (x - count)).collect::<Vec<_>>())
    }
    fn hand_total(p: &[u64]) -> Q {
        int(p.iter().sum())
    }
    fn top_of(p: &[u64]) -> u64 {
        p[TINY_SEEN - 1]
    }
    // (k + 1) times the smaller number, minus one (chapter 13's rival)
    fn smaller(p: &[u64]) -> Q {
        Q::int((p.len() as i128 + 1) * p[0] as i128) - Q::ONE
    }
    // a whole value as it is, a half to one decimal (the table's column)
    let shown = |x: Q| {
        if x.whole().is_some() {
            Ok(x.to_string())
        } else {
            x.decimal1()
        }
    };
    let shown_all = |v: &[Q]| -> Result<String, String> {
        Ok(v.iter().map(|&x| shown(x)).collect::<Result<Vec<_>, _>>()?.join(","))
    };
    let (a1, a2): ([u64; 2], [u64; 2]) = ([1, 2], [1, TINY]);
    let slope = (doubling_recipe(&a2) - doubling_recipe(&a1)) / (hand_total(&a2) - hand_total(&a1));
    f.add(
        "G-F12.pair_doubling",
        format!("{slope}*total{}", doubling_recipe(&a1) - slope * hand_total(&a1)),
    )?;
    let dbl: Vec<Q> = pairs.iter().map(|p| doubling_recipe(p)).collect();
    f.add("G-F12.doubling_estimates", csv(&dbl))?;
    f.add("G-F12.doubling_average", mean(&dbl))?;
    let lowest = *dbl.iter().min().ok_or("no pairs")?;
    let highest = *dbl.iter().max().ok_or("no pairs")?;
    f.add("G-F12.doubling_range", csv(&[lowest, highest]))?;
    // the pairs sorted by their largest number, each group's doubling
    // estimates and their average
    let mut tops: Vec<u64> = pairs.iter().map(|p| top_of(p)).collect();
    tops.sort_unstable();
    tops.dedup();
    let group = |top: u64| -> Vec<Q> {
        pairs
            .iter()
            .zip(&dbl)
            .filter(|(p, _)| top_of(p) == top)
            .map(|(_, &d)| d)
            .collect()
    };
    let avg_of = |top: u64| mean(&group(top));
    f.add("G-F12.groups", tops.len())?;
    f.add("G-F12.group_tops", csv(&tops))?;
    f.add(
        "G-F12.group_sizes",
        csv(&tops.iter().map(|&t| group(t).len()).collect::<Vec<_>>()),
    )?;
    for &t in &tops {
        f.add(&format!("G-F12.ends_in_{t}"), csv(&group(t)))?;
    }
    for &t in &tops[1..] {
        f.add(&format!("G-F12.ends_in_{t}_average"), avg_of(t))?;
    }
    f.add("G-F12.ends_in_5_average_decimal", avg_of(TINY).decimal1()?)?;
    let averages: Vec<Q> = tops.iter().map(|&t| avg_of(t)).collect();
    f.add("G-F12.group_averages", csv(&averages))?;
    f.add("G-F12.group_averages_shown", shown_all(&averages)?)?;
    f.add(
        "G-F12.grouped_is_gap_recipe",
        flag(pairs.iter().all(|p| avg_of(top_of(p)) == gap_recipe(p))),
    )?;
    // a group's average is the same in every company that could have sold
    // its pairs (given the largest, the smaller is equally likely to be any
    // number below it: chapter 11)
    let group_mean = |n: u64, top: u64, recipe: &dyn Fn(&[u64]) -> Q| {
        mean(
            &model::all_handfuls(n, TINY_SEEN)
                .iter()
                .filter(|h| top_of(h) == top)
                .map(|h| recipe(h.as_slice()))
                .collect::<Vec<_>>(),
        )
    };
    f.add(
        "G-F12.group_average_same_for_every_count",
        flag(
            tops.iter()
                .all(|&t| (t..=GROUP_COMPANIES_TO).all(|n| group_mean(n, t, &doubling_recipe) == avg_of(t))),
        ),
    )?;
    // each pair is one chance in ten, so a group's chance is its pairs in
    // ten; weighted by those chances the group averages give the ten pairs'
    // average back, and weighted equally they do not
    let chance = |t: u64| Q::int(group(t).len() as i128) / ten;
    f.add("G-F12.top_group_chance", chance(*tops.last().ok_or("no groups")?))?;
    let equal = mean(&averages);
    f.add(
        "G-F12.weighted_average",
        tops.iter().fold(Q::ZERO, |s, &t| s + chance(t) * avg_of(t)),
    )?;
    f.add("G-F12.equal_weight_average", equal)?;
    f.add("G-F12.equal_weight_average_shown", equal.decimal2()?)?;
    // the misses as arrows from the count: the one group whose arrows point
    // both ways, and the largest group whose arrows all point one way
    let five = int(TINY);
    let signed = |t: u64| -> Vec<Q> { group(t).iter().map(|&e| e - five).collect() };
    let both: Vec<u64> = tops
        .iter()
        .copied()
        .filter(|&t| signed(t).iter().any(|&x| x < Q::ZERO) && signed(t).iter().any(|&x| x > Q::ZERO))
        .collect();
    let cancel = *both.first().ok_or("no group whose misses point both ways")?;
    let one_way = tops
        .iter()
        .copied()
        .filter(|&t| group(t).len() > 1 && !both.contains(&t))
        .max()
        .ok_or("no group whose misses all point one way")?;
    f.add("G-F12.cancel_group", cancel)?;
    f.add("G-F12.cancel_misses", csv(&signed(cancel)))?;
    f.add("G-F12.same_side_group", one_way)?;
    f.add("G-F12.same_side_misses", csv(&signed(one_way)))?;
    f.add("G-F12.same_side_average_miss", (avg_of(one_way) - five).abs())?;
    let (dbl_sq, gap_sq) = (squared(&dbl, five), squared(&tiny, five));
    f.add("G-F12.doubling_miss", miss(&dbl, five).decimal1()?)?;
    f.add("G-F12.grouped_miss", miss(&tiny, five).decimal1()?)?;
    f.add("G-F12.doubling_squared_miss", dbl_sq)?;
    f.add("G-F12.grouped_squared_miss", gap_sq)?;
    f.add("G-F12.grouped_squared_miss_shown", gap_sq.decimal2()?)?;
    f.add("G-F12.doubling_sd_about", dbl_sq.sqrt_about1()?)?;
    let root = gap_sq
        .exact_sqrt()
        .ok_or("the gap recipe's squared miss is not a square")?;
    f.add("G-F12.grouped_sd", root)?;
    f.add("G-F12.grouped_sd_decimal", root.decimal1()?)?;
    // for squares, the drop is exactly the spread of the estimates inside
    // their groups, weighted by the groups' chances
    f.add(
        "G-F12.spread_inside",
        tops.iter().fold(Q::ZERO, |s, &t| {
            let a = avg_of(t);
            s + chance(t) * mean(&group(t).iter().map(|&e| (e - a) * (e - a)).collect::<Vec<_>>())
        }),
    )?;

    // 13: unbiased must hold at every count. A recipe that always says 71 is
    // exactly right on average when the count is 71, and at no other count
    f.add("G-F13.always", n71)?;
    // the staircase for pairs: in a company of n boxes, the pairs topped by n
    // carry the one value no smaller company forces
    let seen = TINY_SEEN as u64;
    let companies: Vec<u64> = (seen..=TINY).collect();
    f.add("G-F13.smallest_company", companies[0])?;
    f.add("G-F13.next_company", companies[1])?;
    f.add("G-F13.predict_company", companies[2])?;
    f.add("G-F13.last_company", companies[companies.len() - 1])?;
    f.add("G-F13.companies", csv(&companies))?;
    f.add(
        "G-F13.company_pairs",
        csv(&companies.iter().map(|&n| choose(n, seen)).collect::<Vec<_>>()),
    )?;
    f.add("G-F13.pairs_from_3", choose(3, seen))?;
    f.add("G-F13.pairs_from_4", choose(4, seen))?;
    for &n in &companies[1..] {
        f.add(
            &format!("G-F13.new_pairs.{n}"),
            model::all_handfuls(n, TINY_SEEN)
                .iter()
                .filter(|h| top_of(h) == n)
                .count(),
        )?;
    }
    let largest_values = |n: u64| -> Vec<u64> {
        let mut v: Vec<u64> = model::all_handfuls(n, TINY_SEEN).iter().map(|h| top_of(h)).collect();
        v.sort_unstable();
        v.dedup();
        v
    };
    let new_values: Vec<usize> = companies
        .iter()
        .map(|&n| {
            let before = largest_values(n - 1);
            largest_values(n).iter().filter(|v| !before.contains(v)).count()
        })
        .collect();
    f.add(
        "G-F13.new_values_per_count",
        if new_values.iter().all(|&c| c == new_values[0]) {
            new_values[0].to_string()
        } else {
            "unequal".into()
        },
    )?;
    let forced = model::forced_values(seen, TINY);
    let at = |mm: u64| forced[(mm - seen) as usize];
    f.add("G-F13.forced", csv(&forced))?;
    f.add("G-F13.forced_at_2", at(2))?;
    f.add("G-F13.forced_at_3_decimal", at(3).decimal1()?)?;
    f.add("G-F13.forced_at_4", at(4))?;
    f.add("G-F13.forced_at_5", at(5))?;
    f.add("G-F13.forced_at_5_decimal", at(5).decimal1()?)?;
    f.add("G-F13.forced_shown", shown_all(&forced)?)?;
    f.add(
        "G-F13.company_total.4",
        model::all_handfuls(4, TINY_SEEN)
            .iter()
            .fold(Q::ZERO, |s, h| s + at(top_of(h))),
    )?;
    let forced5 = model::forced_values(K, 40);
    f.add(
        "G-F13.forced_is_gap_recipe",
        flag(forced5.iter().enumerate().all(|(i, &v)| v == gap_of(K + i as u64, K))),
    )?;
    // the share of handfuls topped by the count itself: k/N, never zero (60
    // boxes, five seen: chapter 11's one in twelve)
    f.add(
        "G-F13.share_at_60",
        Q::count(choose(m - 1, K - 1)) / Q::count(choose(m, K)),
    )?;
    f.add("G-F13.recurrence_proved", flag(staircase_algebra_holds()))?;
    // any unbiased recipe, even three times the smaller number minus one,
    // averages inside its groups to the same staircase
    let (b1, b2): ([u64; 2], [u64; 2]) = ([1, TINY], [2, TINY]);
    let rise = smaller(&b2) - smaller(&b1);
    f.add(
        "G-F13.smaller_recipe",
        format!("{rise}*smaller{}", smaller(&b1) - rise * int(b1[0])),
    )?;
    let sm: Vec<Q> = pairs.iter().map(|p| smaller(p)).collect();
    f.add("G-F13.smaller_estimates", csv(&sm))?;
    let (sm_lo, sm_hi) = (*sm.iter().min().ok_or("no pairs")?, *sm.iter().max().ok_or("no pairs")?);
    f.add("G-F13.smaller_range", csv(&[sm_lo, sm_hi]))?;
    f.add("G-F13.smaller_average", mean(&sm))?;
    f.add(
        "G-F13.smaller_grouped",
        csv(&tops.iter().map(|&t| group_mean(TINY, t, &smaller)).collect::<Vec<_>>()),
    )?;
    f.add("G-F13.smaller_miss", miss(&sm, five))?;
    f.add("G-F13.smaller_miss_decimal", miss(&sm, five).decimal1()?)?;
    f.add("G-F13.smaller_squared_miss", squared(&sm, five))?;
    // a biased recipe of the largest, seven sixths of it minus one: on the
    // friends' boxes, and over every handful of the 71-box company
    let factor = Q::new(K as i128 + 2, K as i128 + 1);
    let shrunk = |mm: u64| factor * int(mm) - Q::ONE;
    let risks = |n: u64, r: &dyn Fn(u64) -> Q| {
        (
            model::mean_by_largest(n, K, r),
            model::mean_by_largest(n, K, |mm| (r(mm) - int(n)).abs()),
            model::mean_by_largest(n, K, |mm| (r(mm) - int(n)) * (r(mm) - int(n))),
        )
    };
    f.add("G-F13.shrunk_factor", factor)?;
    f.add("G-F13.shrunk_recipe", format!("{factor}*largest-1"))?;
    f.add("G-F13.shrunk_friends", shrunk(m))?;
    let (s_mean, s_mae, s_mse) = risks(n71, &shrunk);
    f.add("G-F13.shrunk_average_71", s_mean)?;
    f.add("G-F13.shrunk_bias_71", s_mean - int(n71))?;
    f.add("G-F13.shrunk_miss_71", s_mae)?;
    f.add("G-F13.shrunk_miss_71_shown", s_mae.about1()?)?;
    f.add("G-F13.shrunk_squared_71", s_mse)?;
    f.add("G-F13.shrunk_squared_71_shown", s_mse.about1()?)?;
    f.add("G-F13.gap_squared_71_shown", var71.about1()?)?;
    f.add("G-F13.shrunk_misses_less_squared", flag(s_mse < var71))?;
    f.add("G-F13.shrunk_misses_less", flag(s_mae < mae71))?;
    // where it loses: with exactly K boxes the gap recipe is always right;
    // and the counts where the 7/6 recipe does not miss less, squared or plain
    let edge: Vec<u64> = (1..=K).collect();
    f.add("G-F13.edge_count", K)?;
    f.add("G-F13.gap_at_edge", gap_recipe(&edge))?;
    f.add("G-F13.shrunk_at_edge", shrunk(K))?;
    f.add("G-F13.shrunk_at_edge_shown", shrunk(K).about1()?)?;
    let (mut loses_squared, mut loses_plain) = (Vec::new(), Vec::new());
    for n in K..=LOSES_UP_TO {
        let (gap_risks, shrunk_risks) = (risks(n, &est), risks(n, &shrunk));
        if shrunk_risks.2 >= gap_risks.2 {
            loses_squared.push(n);
        }
        if shrunk_risks.1 >= gap_risks.1 {
            loses_plain.push(n);
        }
    }
    f.add("G-F13.shrunk_loses_squared_at", csv(&loses_squared))?;
    f.add("G-F13.shrunk_loses_plain_at", csv(&loses_plain))?;

    // 14: unfair draws. The friends' own five from a shop that only received
    // boxes 1 to 71 of the 200 made, then a shelf of late boxes
    f.add("G-F14.shop_boxes", n71)?;
    f.add("G-F14.made", MADE)?;
    f.add("G-F14.shop_handful", csv(&FRIENDS))?;
    f.add("G-F14.shop_estimate", gap_recipe(&FRIENDS))?;
    f.add("G-F14.shop_average", mean71)?;
    f.add("G-F14.shop_short_by", int(MADE) - mean71)?;
    let late_gaps = gaps(&LATE);
    f.add("G-F14.late_from", LATE_FROM)?;
    f.add("G-F14.late_handful", csv(&LATE))?;
    f.add("G-F14.late_estimate", gap_recipe(&LATE))?;
    f.add("G-F14.late_first_gap", late_gaps[0])?;
    f.add("G-F14.late_other_gaps", csv(&late_gaps[1..]))?;
    // a fair handful from the 50 boxes 151..200: its largest is 150 + j for
    // the largest j of a fair handful from 1..50
    let pool = MADE - LATE_FROM + 1;
    f.add(
        "G-F14.late_average",
        model::mean_by_largest(pool, K, |j| gap_of(LATE_FROM - 1 + j, K)),
    )?;
    let above = Q::count(choose(pool, K)) / Q::count(choose(MADE, K));
    f.add("G-F14.fair_all_above", above)?;
    f.add("G-F14.fair_all_above_under_one_in_1000", flag(above < Q::new(1, 1000)))?;

    // 15: numbering from an unknown start; equal chances per box
    let inner = model::inner_gaps(&FRIENDS);
    let inner_unseen: u64 = inner.iter().sum();
    let inner_average = Q::new(inner_unseen as i128, inner.len() as i128);
    let spread = m - FRIENDS[0];
    f.add("G-F15.inner_gaps", csv(&inner))?;
    f.add("G-F15.inner_count", inner.len())?;
    f.add("G-F15.hidden_count", K + 1 - inner.len() as u64)?;
    f.add("G-F15.inner_unseen", inner_unseen)?;
    f.add("G-F15.inner_average", inner_average)?;
    f.add("G-F15.inner_average_decimal", inner_average.decimal1()?)?;
    f.add("G-F15.span_numbers", spread + 1)?;
    // the source's s(1 + 2/(k − 1)) − 1, against the gap form 47 + 2 × 10.5
    let gap_form = int(spread + 1) + Q::int(2) * inner_average;
    f.pin(
        "G-F15.spread_estimate",
        model::spread_of(spread, K),
        &gap_form.to_string(),
        CGM,
    )?;
    // the closed form counts handfuls of n consecutive numbers by spread, so
    // it holds for every first number
    f.add(
        "G-F15.spread_unbiased",
        flag((K..16).all(|n| model::mean_by_spread(n, K, |s| model::spread_of(s, K)) == int(n))),
    )?;
    let deliveries: [Vec<u64>; 2] = [(1..=5).collect(), (6..=10).collect()];
    f.add(
        "G-F15.deliveries",
        deliveries.iter().map(|h| csv(h)).collect::<Vec<_>>().join(";"),
    )?;
    f.add(
        "G-F15.delivery_estimates",
        deliveries
            .iter()
            .map(|h| gap_recipe(h).to_string())
            .collect::<Vec<_>>()
            .join(","),
    )?;
    let each: Vec<Q> = (1..=10u64)
        .map(|b| Q::new(deliveries.iter().filter(|d| d.contains(&b)).count() as i128, 2))
        .collect();
    let equal = each.iter().all(|&p| p == each[0]);
    f.add(
        "G-F15.deliveries_each_box",
        if equal { each[0].to_string() } else { "unequal".into() },
    )?;
    f.add(
        "G-F15.deliveries_average",
        (gap_recipe(&deliveries[0]) + gap_recipe(&deliveries[1])) / Q::int(2),
    )?;
    f.add("G-F15.deliveries_made", 10)?;

    // 16: a sixth box
    let mut six = FRIENDS.to_vec();
    six.push(SIXTH);
    let six_gaps: u64 = gaps(&six).iter().sum();
    f.add("G-F16.sixth", SIXTH)?;
    f.add("G-F16.size", six.len())?;
    f.add("G-F16.unseen", six_gaps)?;
    f.add("G-F16.gaps", csv(&gaps(&six)))?;
    f.add("G-F16.estimate", gap_recipe(&six))?;
    f.add("G-F16.average_gap", Q::new(six_gaps as i128, six.len() as i128))?;

    f.self_check()?;
    Ok(f)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The opening act's numbers, as the voice and the pictures use them.
    #[test]
    fn the_opening_numbers_are_the_storyboards() {
        let f = facts().unwrap();
        for (key, value) in [
            ("G-F1.numbers", "14,27,33,51,60"),
            ("G-F1.at_least", "60"),
            ("G-F2.over", "600"),
            ("G-F2.big_largest_over_600", "true"),
            ("G-F2.big_all_at_most_60_under_one_in_a_million", "true"),
            ("G-F2.small_all_at_most_60.percent", "45"),
            ("G-F2.small_fit_shown", "3"),
            ("G-F2.spacing_over", "150"),
            ("G-F2.big_spacing_over_150", "true"),
            ("G-F2.small_spacing_about", "12"),
            ("G-F2.friends_spacing", "12"),
            ("G-F3.first_seen", "14"),
            ("G-F3.second_seen", "27"),
            ("G-F3.first_gap", "13"),
            ("G-F3.second_gap", "12"),
            ("G-F3.last_gaps", "5,17,8"),
            ("G-F3.gaps", "13,12,5,17,8"),
            ("G-F3.unseen", "55"),
            ("G-F3.average_gap", "11"),
            ("G-F3.estimate", "71"),
            ("G-F3.top_gaps", "1"),
            ("G-F9.range", "60,107"),
            ("G-F14.shop_average", "71"),
            ("G-F15.spread_estimate", "68"),
            ("G-F16.estimate", "104"),
        ] {
            assert_eq!(f.get(key), Some(value), "{key}");
        }
    }

    /// The chapter 03 inset is chapter 02's first row from seventy, and its
    /// gaps with the top gap fill the company exactly.
    #[test]
    fn the_known_rows_gaps_fill_seventy() {
        let row = &small_handfuls()[0];
        let list = gap_list(row, SMALL);
        assert_eq!(list.iter().sum::<u64>() + K, SMALL);
        assert_eq!(list.len(), 6);
    }

    /// Every key appears once, and the count matches the oracle's.
    #[test]
    fn there_is_one_fact_per_oracle_key() {
        let f = facts().unwrap();
        assert_eq!(f.len(), 345);
    }

    /// Chapters 12 and 13's numbers, as the voice and the pictures use them.
    #[test]
    fn the_averaging_numbers_are_the_storyboards() {
        let f = facts().unwrap();
        for (key, value) in [
            ("G-F12.doubling_estimates", "2,3,4,5,4,5,6,6,7,8"),
            ("G-F12.group_sizes", "1,2,3,4"),
            ("G-F12.group_averages_shown", "2,3.5,5,6.5"),
            ("G-F12.weighted_average", "5"),
            ("G-F12.equal_weight_average_shown", "4.25"),
            ("G-F12.cancel_misses", "-1,0,1"),
            ("G-F12.same_side_misses", "0,1,2,3"),
            ("G-F12.doubling_miss", "1.4"),
            ("G-F12.grouped_miss", "1.2"),
            ("G-F12.doubling_squared_miss", "3"),
            ("G-F12.grouped_squared_miss", "9/4"),
            ("G-F12.grouped_sd_decimal", "1.5"),
            ("G-F13.forced_shown", "2,3.5,5,6.5"),
            ("G-F13.company_total.4", "24"),
            ("G-F13.share_at_60", "1/12"),
            ("G-F13.recurrence_proved", "true"),
            ("G-F13.smaller_grouped", "2,7/2,5,13/2"),
            ("G-F13.shrunk_friends", "69"),
            ("G-F13.shrunk_bias_71", "-2"),
            ("G-F13.shrunk_squared_71", "397/3"),
            ("G-F13.shrunk_at_edge", "29/6"),
            ("G-F13.shrunk_loses_plain_at", "5,11"),
        ] {
            assert_eq!(f.get(key), Some(value), "{key}");
        }
    }

    /// Every handful of k from 1..n, grouped by its largest number, with a
    /// recipe's estimate for each.
    fn by_top(n: u64, k: usize, recipe: impl Fn(&[u64]) -> Q) -> Vec<Vec<Q>> {
        let mut groups: Vec<Vec<Q>> = vec![Vec::new(); n as usize + 1];
        for h in model::all_handfuls(n, k) {
            groups[h[k - 1] as usize].push(recipe(h.as_slice()));
        }
        groups.into_iter().filter(|g| !g.is_empty()).collect()
    }

    fn mean_of(v: &[Q]) -> Q {
        v.iter().fold(Q::ZERO, |s, &x| s + x) / Q::int(v.len() as i128)
    }

    /// Chapter 12's argument at its edges, for the doubling recipe and three
    /// times the smaller, pairs to fours from up to ten boxes: inside every
    /// group the average misses by at most the average miss, exactly as much
    /// when no two misses point opposite ways; its squared miss is less by
    /// exactly the spread inside the group; and weighted by their chances
    /// the groups give back the count, weighted equally they never do once
    /// N > k.
    #[test]
    fn averaging_inside_a_group_never_misses_more() {
        let smaller = |p: &[u64]| Q::int((p.len() as i128 + 1) * p[0] as i128) - Q::ONE;
        for recipe in [&doubling_recipe as &dyn Fn(&[u64]) -> Q, &smaller] {
            for k in 2..=4usize {
                for n in k as u64..=10 {
                    let count = int(n);
                    let groups = by_top(n, k, recipe);
                    for g in &groups {
                        let avg = mean_of(g);
                        let misses: Vec<Q> = g.iter().map(|&e| e - count).collect();
                        let plain = mean_of(&misses.iter().map(|&x| x.abs()).collect::<Vec<_>>());
                        let sq = mean_of(&misses.iter().map(|&x| x * x).collect::<Vec<_>>());
                        let spread = mean_of(&g.iter().map(|&e| (e - avg) * (e - avg)).collect::<Vec<_>>());
                        let one_way = misses.iter().all(|&x| x >= Q::ZERO) || misses.iter().all(|&x| x <= Q::ZERO);
                        assert!((avg - count).abs() <= plain, "k={k} n={n}");
                        assert_eq!((avg - count).abs() == plain, one_way, "k={k} n={n}");
                        assert_eq!(sq - (avg - count) * (avg - count), spread, "k={k} n={n}");
                    }
                    let all = Q::count(choose(n, k as u64));
                    let weighted = groups
                        .iter()
                        .fold(Q::ZERO, |s, g| s + Q::int(g.len() as i128) / all * mean_of(g));
                    let equal = mean_of(&groups.iter().map(|g| mean_of(g)).collect::<Vec<_>>());
                    assert_eq!(weighted, count, "k={k} n={n}");
                    assert_eq!(equal == count, n == k as u64, "k={k} n={n}");
                }
            }
        }
    }

    /// Chapter 13's staircase is forced: raising it at one value biases the
    /// recipe at that count and every larger one, and at no smaller one; and
    /// with no new value at a count (the largest capped at 4, for pairs)
    /// there is nothing to solve for and no unbiased recipe.
    #[test]
    fn the_staircase_is_forced_and_needs_a_new_value() {
        for k in [2u64, 3] {
            let base = model::forced_values(k, 12);
            for bump in k..=11 {
                let mut g = base.clone();
                g[(bump - k) as usize] = g[(bump - k) as usize] + Q::ONE;
                for n in k..=12 {
                    let average = model::mean_by_largest(n, k, |mm| g[(mm - k) as usize]);
                    assert_eq!(average != int(n), n >= bump, "k={k} bump={bump} n={n}");
                }
            }
        }
        let capped = model::forced_values(2, 4);
        let at_five = model::mean_by_largest(5, 2, |mm| capped[(mm.min(4) - 2) as usize]);
        assert_eq!(at_five, Q::new(22, 5));
    }

    /// The degree-bound grid of [`staircase_algebra_holds`] catches a wrong
    /// step: N² − (N − 1)(N + k) is not (k + 1)N − k.
    #[test]
    fn the_staircase_algebra_check_can_fail() {
        assert!(staircase_algebra_holds());
        let wrong = (0..=2i128).all(|n| (0..=1i128).all(|k| n * n - (n - 1) * (n + k) == (k + 1) * n - k));
        assert!(!wrong);
    }

    /// The largest of five averages 5(N + 1)/6, so the 7/6 recipe averages
    /// 35(N + 1)/36 − 1: its bias is −(N + 1)/36 at every count, unbiased at
    /// none, while the gap recipe is unbiased at all of them.
    #[test]
    fn the_seven_sixths_recipe_is_biased_at_every_count() {
        let shrunk = |mm: u64| Q::new(7, 6) * int(mm) - Q::ONE;
        for n in K..=120 {
            assert_eq!(
                model::mean_by_largest(n, K, shrunk) - int(n),
                -Q::new(n as i128 + 1, 36)
            );
            assert_eq!(model::mean_by_largest(n, K, |mm| gap_of(mm, K)), int(n));
        }
    }
}
