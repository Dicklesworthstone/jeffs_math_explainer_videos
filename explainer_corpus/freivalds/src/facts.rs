//! Every number the video shows or says, computed from [`crate::model`] and
//! printed for `oracle.py` (`./x freivalds facts-check`). F-F rows are the
//! example's numbers; F-X rows are the edges of the video's always/never
//! sentences. Matrices print row by row (`1,2,1;0,3,1;3,2,0`), tosses as
//! their coins (`110`), and rows and columns count from 1, as the screen
//! does.

use std::collections::BTreeSet;

use explainer_kit::Facts;

use crate::model::{
    self, A, A2, B, B2, C, Check, Column, FIRST_TOSS, Matrix, ROUNDED_PASSES, ROUNDED_REJECTS, SHAPES, SIDE, SPOT, Toss,
};

/// The size chapter 01 quotes for scale (01_2): a thousand rows and columns.
pub const BIG_SIDE: u64 = 1000;

/// How many fresh tosses chapter 01 quotes (01_10), and the odds they beat.
pub const AGAIN: u32 = 20;
pub const MILLION: u64 = 1_000_000;

/// The values every entry of a small mistake is drawn from, at each size of
/// the bound's edge test.
pub const SWEEP_3: [i64; 3] = [-1, 0, 1];
const SWEEP_2: [i64; 5] = [-2, -1, 0, 1, 2];

/// The mistakes chapter 02 shows, in order (`model::SHAPES` names): the
/// slip, two in one row of one sign, a cancelling pair, the three-in-a-row
/// and one per row and column.
pub const CHAPTER_02_SHAPES: [&str; 5] = [
    "one_entry",
    "two_in_a_row",
    "cancelling",
    "three_in_a_row",
    "every_column",
];

/// The mistake chapter 02 pairs the tosses of, to show the proof's step.
pub const PROOF_SHAPE: &str = "three_in_a_row";

/// Chapter 03's two fresh tosses, as its grid of ways names its axes.
pub const TOSS_NAMES: [&str; 2] = ["first toss", "second toss"];

/// A mistake shape by its name in `model::SHAPES`.
pub fn shape(name: &str) -> Result<Matrix<SIDE>, String> {
    SHAPES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|&(_, d)| d)
        .ok_or_else(|| format!("no mistake shape {name}"))
}

/// The tosses the mistake `d` hides from, in counting order.
pub fn hiding(d: &Matrix<SIDE>) -> Vec<Toss<SIDE>> {
    model::tosses::<SIDE>()
        .into_iter()
        .filter(|t| model::hides(d, t))
        .collect()
}

/// Where the proof's mistake is first wrong, (row, column) counted from 0:
/// the row whose sum the flip moves, and the column whose coin it flips.
pub fn proof_entry() -> Result<(usize, usize), String> {
    let d = shape(PROOF_SHAPE)?;
    model::wrong_entries(&d)
        .first()
        .copied()
        .ok_or_else(|| format!("{PROOF_SHAPE} has no wrong entry"))
}

/// A·B, the true product.
pub fn true_product() -> Matrix<SIDE> {
    model::product(&A, &B).0
}

/// The first toss's check (01_4 to 01_6).
pub fn first_check() -> Check<SIDE> {
    model::check(&A, &B, &C, &FIRST_TOSS)
}

/// Every toss with its check, in counting order (01_7 to 01_9).
pub fn all_checks() -> Vec<(Toss<SIDE>, Check<SIDE>)> {
    model::checks(&A, &B, &C)
}

/// The tosses in the order chapter 03's grid of two tosses lists them: the
/// ones that miss the slip first, then the ones that catch it, each in
/// counting order, so the ways both tosses miss fill one corner.
pub fn miss_first() -> Vec<Toss<SIDE>> {
    let checks = all_checks();
    let misses = checks.iter().filter(|(_, ch)| ch.passes()).map(|(t, _)| *t);
    let catches = checks.iter().filter(|(_, ch)| !ch.passes()).map(|(t, _)| *t);
    misses.chain(catches).collect()
}

/// For each toss, in counting order, the wrong C that someone who saw it
/// first could write (`model::blind_spot`), as its mistake. An error if a
/// toss has none, or catches it.
pub fn blind_spots() -> Result<Vec<(Toss<SIDE>, Matrix<SIDE>)>, String> {
    let ab = true_product();
    model::tosses::<SIDE>()
        .into_iter()
        .map(|t| {
            let text = model::toss_text(&t);
            let d = model::blind_spot(&t).ok_or_else(|| format!("toss {text} has no blind spot"))?;
            if !model::check(&A, &B, &model::plus(&ab, &d), &t).passes() {
                return Err(format!("toss {text} catches its own blind spot"));
            }
            Ok((t, d))
        })
        .collect()
}

/// The one wrong entry of `c`, (row, column) counted from 0, or an error
/// when `c` has none or more than one.
fn one_wrong_entry(c: &Matrix<SIDE>) -> Result<(usize, usize), String> {
    let wrong = model::wrong_entries(&model::mistake(c, &true_product()));
    match wrong[..] {
        [one] => Ok(one),
        _ => Err(format!("the claim must have exactly one wrong entry, not {wrong:?}")),
    }
}

/// C's one wrong entry, (row, column) counted from 0.
pub fn wrong_entry() -> Result<(usize, usize), String> {
    one_wrong_entry(&C)
}

/// The products of the entry 01_2 works out by hand, in order.
pub fn spot_products() -> Vec<i64> {
    model::products_at(&A, &B, SPOT.0, SPOT.1)
}

/// The pairing proof's step, for any ring and any n (UIUC CS 473, Fall
/// 2019, lecture 12). Pick a column j where C is wrong and pair each toss
/// with the one that differs only in coin j. Between the two, the sum of a
/// row where C is wrong in column j changes by that nonzero error, so at
/// most this many tosses of each pair miss. A constant of the proof: the
/// example's pairs are checked against it, never used to find it.
pub const MISSES_PER_PAIR: usize = 1;

/// The pairing proof's bound on one toss, (num, den): at most
/// [`MISSES_PER_PAIR`] of each pair of two. With fair coins, all independent
/// and tossed after C is fixed, every toss is equally likely, so that share
/// is the chance.
const PAIRING_BOUND: (u64, u64) = (MISSES_PER_PAIR as u64, 2);

/// One finite check of `bound` at one size of the edge test. The worst
/// small mistake (`most` of `tosses`) passes no more than the bound, no coin
/// pair passes twice (the proof's step, counted), and the worst meets the
/// bound exactly (an equality case). A failure is an error. A pass proves
/// nothing beyond its own size: the general bound is the proof's.
fn check_bound(bound: (u64, u64), most: usize, tosses: usize, pairs_both_passing: usize) -> Result<(), String> {
    let (num, den) = bound;
    let (most, tosses) = (most as u64, tosses as u64);
    if pairs_both_passing != 0 {
        return Err(format!(
            "{pairs_both_passing} coin pairs pass twice: the pairing step fails"
        ));
    }
    if most * den > num * tosses {
        return Err(format!(
            "a small mistake passes {most} of {tosses} tosses, more than {num}/{den}"
        ));
    }
    if most * den != num * tosses {
        return Err(format!(
            "no small mistake meets {num}/{den}: the worst passes {most} of {tosses}"
        ));
    }
    Ok(())
}

/// The theorem's bound on one toss, (num, den): the pairing proof's one half
/// ([`PAIRING_BOUND`]), the bound the ladder and the closing card use. The
/// finite sweeps do not derive it; they guard it, and it is refused unless
/// both edge tests (three by three and two by two) pass [`check_bound`]. The
/// example's own share is a separate fact that happens to meet it.
pub fn theorem_bound() -> Result<(u64, u64), String> {
    let three = model::sweep(&A, &B, &SWEEP_3);
    check_bound(
        PAIRING_BOUND,
        three.most,
        model::tosses::<SIDE>().len(),
        three.pairs_both_passing,
    )?;
    let two = model::sweep(&A2, &B2, &SWEEP_2);
    check_bound(
        PAIRING_BOUND,
        two.most,
        model::tosses::<2>().len(),
        two.pairs_both_passing,
    )?;
    Ok(PAIRING_BOUND)
}

/// The bound on [`AGAIN`] such tosses all missing, as 1 in this many: the
/// theorem's per-toss bound to the power [`AGAIN`] (the tosses are
/// independent).
pub fn again_odds() -> Result<u64, String> {
    match theorem_bound()? {
        (1, per) => Ok(per.pow(AGAIN)),
        (num, den) => Err(format!("a per-toss bound of {num}/{den} is not one in a whole number")),
    }
}

/// Values separated by commas.
pub fn csv<T: ToString>(values: &[T]) -> String {
    values.iter().map(T::to_string).collect::<Vec<_>>().join(",")
}

/// A matrix row by row: rows separated by semicolons, entries by commas.
pub fn matrix_text<const N: usize>(m: &Matrix<N>) -> String {
    m.iter().map(|row| csv(row.as_slice())).collect::<Vec<_>>().join(";")
}

/// `num/den` in lowest terms, or the whole number when `den` divides `num`
/// (as Python's `str(Fraction)` prints it).
pub fn ratio(num: i64, den: i64) -> Result<String, String> {
    if den == 0 {
        return Err(format!("{num}/0"));
    }
    let (mut x, mut y) = (num.abs(), den.abs());
    while y != 0 {
        (x, y) = (y, x % y);
    }
    let (n, d) = (num / x * den.signum(), den.abs() / x);
    Ok(if d == 1 { n.to_string() } else { format!("{n}/{d}") })
}

/// The bound's edge test at one size: every mistake with entries from
/// `values`, against `tosses` tosses. An error if any mistake passes more
/// than half, or any coin pair passes twice.
fn sweep_facts(f: &mut Facts, key: &str, values: &[i64], s: model::Sweep, tosses: usize) -> Result<(), String> {
    f.add(&format!("F-X1.{key}.values"), csv(values))?;
    f.add(&format!("F-X1.{key}.tosses"), tosses)?;
    f.add(&format!("F-X1.{key}.wrong"), s.wrong)?;
    f.add(&format!("F-X1.{key}.fewest"), s.fewest)?;
    f.add(&format!("F-X1.{key}.most"), s.most)?;
    f.add(&format!("F-X1.{key}.most_share"), ratio(s.most as i64, tosses as i64)?)?;
    f.add(&format!("F-X1.{key}.at_half"), s.at_half)?;
    f.add(&format!("F-X1.{key}.uneven"), s.uneven)?;
    f.add(&format!("F-X1.{key}.pairs_both_passing"), s.pairs_both_passing)?;
    // a checked equality case of the pairing bound at this size, not its proof
    let (num, den) = PAIRING_BOUND;
    f.add(
        &format!("F-X1.{key}.attains_bound"),
        s.most as u64 * den == num * tosses as u64,
    )?;
    if 2 * s.most > tosses || s.pairs_both_passing != 0 {
        return Err(format!(
            "at {key} a mistake passes {} of {tosses} tosses ({} coin pairs pass twice): the bound fails",
            s.most, s.pairs_both_passing
        ));
    }
    Ok(())
}

/// Chapter 02's numbers: the mistake on its own, each shape it shows, the
/// edges of its always-sentences, and the pairing on the actual tosses.
fn chapter_02(f: &mut Facts, ab: &Matrix<SIDE>) -> Result<(), String> {
    let tosses = model::tosses::<SIDE>();

    // 02_1: the slip's mistake, and what the check sees of any mistake
    let slip = model::mistake(&C, ab);
    f.add("F-F6.slip.mistake", matrix_text(&slip))?;
    f.add("F-F6.first_toss.heads_sum", csv(&model::kept_sum(&slip, &FIRST_TOSS)))?;
    let (mut differs_by_mistake, mut misses_are_zero) = (true, true);
    for (_, d) in SHAPES {
        let c = model::plus(ab, &d);
        for (t, ch) in model::checks(&A, &B, &c) {
            let diff: Column<SIDE> = std::array::from_fn(|i| ch.cr[i] - ch.a_br[i]);
            differs_by_mistake &= diff == model::kept_sum(&d, &t);
            misses_are_zero &= ch.passes() == model::hides(&d, &t);
        }
    }
    f.add("F-F6.difference_is_mistake_sum", differs_by_mistake)?;
    f.add("F-F6.misses_are_zero_sums", misses_are_zero)?;

    // 02_2 to 02_6: each shape, what it hides from
    for name in CHAPTER_02_SHAPES {
        let d = shape(name)?;
        let wrong: Vec<i64> = model::wrong_entries(&d).iter().map(|&(i, j)| d[i][j]).collect();
        let hide = hiding(&d);
        let texts: Vec<String> = hide.iter().map(model::toss_text).collect();
        f.add(&format!("F-F7.{name}.mistake"), matrix_text(&d))?;
        f.add(&format!("F-F7.{name}.wrong_count"), wrong.len())?;
        f.add(&format!("F-F7.{name}.wrong_values"), csv(&wrong))?;
        f.add(&format!("F-F7.{name}.hides_from"), texts.join(","))?;
        f.add(
            &format!("F-F7.{name}.miss_share"),
            ratio(hide.len() as i64, tosses.len() as i64)?,
        )?;
        f.add(&format!("F-F7.{name}.power_of_two"), hide.len().is_power_of_two())?;
    }
    let each = shape("every_column")?;
    let mut per: BTreeSet<usize> = each.iter().map(|row| row.iter().filter(|&&x| x != 0).count()).collect();
    per.extend((0..SIDE).map(|j| each.iter().filter(|row| row[j] != 0).count()));
    f.add(
        "F-F7.every_column.wrong_per_row_and_column",
        csv(&per.into_iter().collect::<Vec<_>>()),
    )?;

    // the edges of "both one too big hide only when both coins land tails",
    // "one too big and one too small hide whenever their coins agree" and
    // "one per row and column hides only from all tails", any sizes
    let (mut same_cases, mut same_bad, mut cancel_cases, mut cancel_bad) = (0, 0, 0, 0);
    for a in -3..=3i64 {
        for b in -3..=3i64 {
            if a == 0 || b == 0 {
                continue;
            }
            let d = [[0, 0, 0], [0, a, b], [0, 0, 0]];
            if a.signum() == b.signum() {
                same_cases += 1;
                same_bad += tosses
                    .iter()
                    .filter(|t| model::hides(&d, t) != (!t[1] && !t[2]))
                    .count();
            }
            if b == -a {
                cancel_cases += 1;
                cancel_bad += tosses.iter().filter(|t| model::hides(&d, t) != (t[1] == t[2])).count();
            }
        }
    }
    f.add("F-X5.same_sign.cases", same_cases)?;
    f.add("F-X5.same_sign.violations", same_bad)?;
    f.add("F-X5.cancelling.cases", cancel_cases)?;
    f.add("F-X5.cancelling.violations", cancel_bad)?;
    let (mut each_cases, mut each_bad) = (0, 0);
    let all_tails = [false; SIDE];
    for code in 0..SIDE * SIDE * SIDE {
        let cols = [code % SIDE, code / SIDE % SIDE, code / (SIDE * SIDE)];
        if cols[0] == cols[1] || cols[0] == cols[2] || cols[1] == cols[2] {
            continue;
        }
        for values in 0..64usize {
            let entry = |i: usize| [-2, -1, 1, 2][(values >> (2 * i)) & 3];
            let d: Matrix<SIDE> =
                std::array::from_fn(|i| std::array::from_fn(|j| if j == cols[i] { entry(i) } else { 0 }));
            each_cases += 1;
            each_bad += tosses
                .iter()
                .filter(|t| model::hides(&d, t) != (**t == all_tails))
                .count();
        }
    }
    f.add("F-X5.one_per_row_and_column.cases", each_cases)?;
    f.add("F-X5.one_per_row_and_column.violations", each_bad)?;
    // why 02_6 says one per row and column, not just wrong in every column
    let three = shape(PROOF_SHAPE)?;
    f.add(
        "F-X5.three_in_a_row.wrong_in_every_column",
        (0..SIDE).all(|j| three.iter().any(|row| row[j] != 0)),
    )?;
    if same_bad + cancel_bad + each_bad != 0 {
        return Err("a chapter 02 sentence fails at its edges".into());
    }

    // 02_7 to 02_9: the pairing on the actual tosses
    let (row, column) = proof_entry()?;
    let pairs = model::coin_pairs::<SIDE>(column);
    let pair_texts: Vec<String> = pairs
        .iter()
        .map(|(t, u)| format!("{}-{}", model::toss_text(t), model::toss_text(u)))
        .collect();
    let tails_sums: Vec<i64> = pairs.iter().map(|(t, _)| model::kept_sum(&three, t)[row]).collect();
    let heads_sums: Vec<i64> = pairs.iter().map(|(_, u)| model::kept_sum(&three, u)[row]).collect();
    let changes = model::pair_changes(&three, row, column);
    let per_pair = model::misses_per_pair(&three, column);
    f.add("F-F8.row", row + 1)?;
    f.add("F-F8.column", column + 1)?;
    f.add("F-F8.wrong_entry", three[row][column])?;
    f.add("F-F8.pairs", pair_texts.join(","))?;
    f.add("F-F8.row_sums.tails", csv(&tails_sums))?;
    f.add("F-F8.row_sums.heads", csv(&heads_sums))?;
    f.add("F-F8.changes", csv(&changes))?;
    f.add("F-F8.misses_per_pair", csv(&per_pair))?;
    f.pin(
        "F-F8.misses_per_pair_bound",
        MISSES_PER_PAIR,
        "1",
        "the pairing proof: across a pair the row sum moves by a nonzero wrong entry, so at most one of the two \
         is zero; UIUC CS 473 (Fall 2019) lecture 12",
    )?;
    // the example's own most, checked against the proof's bound below
    f.add("F-F8.most_misses_per_pair", per_pair.iter().max().copied().unwrap_or(0))?;
    f.add("F-F8.pair_count", pairs.len())?;
    let partners: BTreeSet<usize> = tosses
        .iter()
        .map(|t| pairs.iter().filter(|(a, b)| a == t || b == t).count())
        .collect();
    f.add("F-F8.partners_per_toss", csv(&partners.into_iter().collect::<Vec<_>>()))?;
    f.add(
        "F-F8.pairs_over_tosses",
        ratio(pairs.len() as i64, tosses.len() as i64)?,
    )?;
    if changes.iter().any(|&c| c != three[row][column]) || per_pair.iter().any(|&m| m > MISSES_PER_PAIR) {
        return Err("the pairing step fails on the proof's mistake".into());
    }

    // the hypothesis matters: paired by the coin of a column where C is
    // right, the slip's row sum never moves, and both tosses of a pair miss
    let (slip_row, _) = model::wrong_entries(&slip)
        .first()
        .copied()
        .ok_or("the slip has no wrong entry")?;
    let right = (0..SIDE)
        .find(|&j| slip.iter().all(|r| r[j] == 0))
        .ok_or("the slip is wrong in every column")?;
    let right_misses = model::misses_per_pair(&slip, right);
    f.add("F-X5.right_column.column", right + 1)?;
    f.add("F-X5.right_column.misses_per_pair", csv(&right_misses))?;
    f.add(
        "F-X5.right_column.changes",
        csv(&model::pair_changes(&slip, slip_row, right)),
    )?;
    if right_misses.iter().all(|&m| m <= MISSES_PER_PAIR) {
        return Err("paired by a column where C is right, no pair misses twice: the negative is lost".into());
    }
    Ok(())
}

/// Chapter 03's facts: two fresh tosses (the example's exact quarter and
/// the theorem's bound, kept apart), one toss reused, and a toss seen
/// before C is written.
fn chapter_03(f: &mut Facts, ab: &Matrix<SIDE>) -> Result<(), String> {
    let tosses = model::tosses::<SIDE>();
    let misses = model::passing(&A, &B, &C).len();

    // 03_1 to 03_3: two fresh tosses; the ways both miss are the misses of
    // the first times the misses of the second
    let order = miss_first();
    let texts: Vec<String> = order.iter().map(model::toss_text).collect();
    f.add("F-F9.tosses", TOSS_NAMES.len())?;
    f.add("F-F9.order", texts.join(","))?;
    f.add("F-F9.first.misses", misses)?;
    f.add("F-F9.second.misses", misses)?;
    let (both, ways) = model::both_pass(&A, &B, &C);
    f.add("F-F9.both_miss_is_product", both == misses * misses)?;
    let passes: Vec<bool> = order.iter().map(|t| model::check(&A, &B, &C, t).passes()).collect();
    let corner =
        (0..order.len()).all(|i| (0..order.len()).all(|j| (passes[i] && passes[j]) == (i < misses && j < misses)));
    f.add("F-F9.both_miss_fill_the_corner", corner)?;

    // 03_4: this slip meets the theorem's bound for two tosses exactly; the
    // bound is the per-toss bound squared, for any fixed wrong C
    let (num, den) = theorem_bound()?;
    let k = TOSS_NAMES.len() as u32;
    let (two_num, two_den) = (num.pow(k), den.pow(k));
    let two_bound = ratio(two_num as i64, two_den as i64)?;
    f.add("F-F9.two.bound", &two_bound)?;
    f.add(
        "F-F9.two.example_meets_bound",
        both as u64 * two_den == two_num * ways as u64,
    )?;

    // 03_5: one toss used for both checks, the grid's diagonal: still half,
    // more than the two fresh tosses' bound
    let (reused, diagonal) = model::same_toss_twice(&A, &B, &C);
    f.add("F-F9.same_toss.ways", diagonal)?;
    f.add(
        "F-F9.same_toss.above_two_bound",
        reused as u64 * two_den > two_num * diagonal as u64,
    )?;

    // 03_6 to 03_8: a toss seen before C is written, beaten by the wrong C
    // its blind spot gives; each of those is a fixed wrong C, and fresh
    // tosses catch it
    let spots = blind_spots()?;
    for (t, d) in &spots {
        f.add(
            &format!("F-X3.blind_spot.{}.mistake", model::toss_text(t)),
            matrix_text(d),
        )?;
    }
    let with_tails: Vec<&(Toss<SIDE>, Matrix<SIDE>)> = spots.iter().filter(|(t, _)| t.contains(&false)).collect();
    let tails_counts: BTreeSet<usize> = with_tails.iter().map(|(_, d)| model::wrong_entries(d).len()).collect();
    f.add(
        "F-X3.blind_spot.tails.wrong_entries",
        csv(&tails_counts.into_iter().collect::<Vec<_>>()),
    )?;
    f.add(
        "F-X3.blind_spot.tails.under_tails",
        with_tails
            .iter()
            .all(|(t, d)| model::wrong_entries(d).iter().all(|&(_, j)| !t[j])),
    )?;
    let all_heads = spots
        .iter()
        .find(|(t, _)| !t.contains(&false))
        .map(|(_, d)| *d)
        .ok_or("no toss is all heads")?;
    let heads_wrong = model::wrong_entries(&all_heads);
    let values: Vec<i64> = heads_wrong.iter().map(|&(i, j)| all_heads[i][j]).collect();
    f.add("F-X3.blind_spot.all_heads.values", csv(&values))?;
    let heads_rows: BTreeSet<usize> = heads_wrong.iter().map(|&(i, _)| i).collect();
    f.add("F-X3.blind_spot.all_heads.one_row", heads_rows.len() == 1)?;
    let catches: Vec<usize> = spots
        .iter()
        .map(|(_, d)| tosses.len() - model::passing(&A, &B, &model::plus(ab, d)).len())
        .collect();
    f.add("F-X3.blind_spot.fresh_catches", csv(&catches))?;
    let shares: BTreeSet<String> = catches
        .iter()
        .map(|&c| ratio(c as i64, tosses.len() as i64))
        .collect::<Result<_, _>>()?;
    f.add(
        "F-X3.blind_spot.fresh_catch_share",
        shares.into_iter().collect::<Vec<_>>().join(","),
    )?;
    // the edge in size: two by two, every toss has a blind spot too; one by
    // one, a heads coin has none (a nonzero 1 × 1 mistake never sums to 0)
    let ab2 = model::product(&A2, &B2).0;
    let small = model::tosses::<2>();
    let fooled2 = small
        .iter()
        .filter(|&t| model::blind_spot(t).is_some_and(|d| model::check(&A2, &B2, &model::plus(&ab2, &d), t).passes()))
        .count();
    f.add("F-X3.n2.fooled", fooled2)?;
    f.add("F-X3.n2.tosses", small.len())?;
    f.add(
        "F-X3.n1.heads_fooled",
        (-3..=3i64).filter(|&x| x != 0).any(|x| model::hides(&[[x]], &[true])),
    )?;
    if spots.len() != tosses.len() || fooled2 != small.len() || catches.iter().any(|&c| 2 * c < tosses.len()) {
        return Err("a chapter 03 claim about seen tosses fails".into());
    }
    Ok(())
}

/// A column computed in doubles, as the whole numbers it holds (every value
/// here is a double that is a whole number, so the cast is exact).
pub fn as_whole<const N: usize>(v: &[f64; N]) -> [i64; N] {
    v.map(|x| x as i64)
}

/// One toss of a rounded example (`model::ROUNDED_*`): the check's columns
/// in exact arithmetic and in doubles.
pub struct RoundedToss {
    pub toss: Toss<2>,
    pub exact: Check<2>,
    pub br: [i64; 2],
    pub a_br: [i64; 2],
    pub cr: [i64; 2],
}

impl RoundedToss {
    /// The two columns agree in doubles.
    pub fn passes(&self) -> bool {
        self.a_br == self.cr
    }
}

/// The check of `c` against `a` times `b` on `toss`, both ways.
pub fn rounded_toss(a: &Matrix<2>, b: &Matrix<2>, c: &Matrix<2>, toss: &Toss<2>) -> RoundedToss {
    let r = toss.map(f64::from);
    let br = model::rounded_times(b, &r);
    RoundedToss {
        toss: *toss,
        exact: model::check(a, b, c, toss),
        br: as_whole(&br),
        a_br: as_whole(&model::rounded_times(a, &br)),
        cr: as_whole(&model::rounded_times(c, &r)),
    }
}

/// How far apart a check's two columns are: the largest gap over the rows.
pub fn residual(x: &[i64; 2], y: &[i64; 2]) -> i64 {
    x.iter().zip(y).map(|(p, q)| (p - q).abs()).max().unwrap_or(0)
}

/// The right C's residuals in doubles (`model::ROUNDED_REJECTS`), toss by
/// toss in counting order.
pub fn right_residuals() -> Vec<i64> {
    let [a, b, c] = ROUNDED_REJECTS;
    model::tosses::<2>()
        .iter()
        .map(|t| {
            let rt = rounded_toss(&a, &b, &c, t);
            residual(&rt.a_br, &rt.cr)
        })
        .collect()
}

/// The tolerance chapter 04 tries (04_6): the smallest that lets the right
/// C through on every toss in doubles, its largest residual there. Computed
/// from the example, never chosen.
pub fn right_tolerance() -> i64 {
    right_residuals().into_iter().max().unwrap_or(0)
}

/// The wrong C's residuals in exact sums (`model::ROUNDED_PASSES`), toss by
/// toss in counting order.
pub fn wrong_residuals() -> Vec<i64> {
    let [a, b, c] = ROUNDED_PASSES;
    model::tosses::<2>()
        .iter()
        .map(|t| {
            let ch = model::check(&a, &b, &c, t);
            residual(&ch.a_br, &ch.cr)
        })
        .collect()
}

/// Chapter 04's facts: what doubles do at N = 10^16, the two rounded
/// examples on the tosses the scene shows, and a tolerance's test.
fn chapter_04(f: &mut Facts) -> Result<(), String> {
    // 04_2, 04_3: N + 1 in doubles, and the doubles near N
    let big = model::BIG;
    let digits = big.to_string().len() - 1;
    if 10_i64.pow(digits as u32) != big {
        return Err(format!("{big} is not a power of ten"));
    }
    let near = big as f64;
    let plus_one = as_whole(&[near + 1.0])[0];
    f.add("F-X4.big", big)?;
    f.add("F-X4.big_exponent", digits)?;
    f.add("F-X4.plus_one.exact", big + 1)?;
    f.add("F-X4.plus_one.double", plus_one)?;
    f.add("F-X4.lost", big + 1 - plus_one)?;
    f.add("F-X4.gap", as_whole(&[near.next_up() - near])[0])?;
    let doubles: Vec<i64> = (-2..=2i64)
        .filter(|&k| as_whole(&[(big + k) as f64])[0] == big + k)
        .collect();
    f.add("F-X4.doubles_near", csv(&doubles))?;
    f.add("F-X4.significand_bits", f64::MANTISSA_DIGITS)?;
    f.add("F-X4.big_above_2_53", big > 1_i64 << f64::MANTISSA_DIGITS)?;

    // 04_4: the right C, on the one toss doubles fail
    let small = model::tosses::<2>();
    let [a, b, c] = ROUNDED_REJECTS;
    let failing: Vec<RoundedToss> = small
        .iter()
        .map(|t| rounded_toss(&a, &b, &c, t))
        .filter(|rt| !rt.passes())
        .collect();
    let [shown] = &failing[..] else {
        return Err("in doubles the right C must fail exactly one toss".into());
    };
    f.add("F-X4.reject.br.exact", csv(&shown.exact.br))?;
    f.add("F-X4.reject.br.double", csv(&shown.br))?;
    f.add("F-X4.reject.a_br.exact", csv(&shown.exact.a_br))?;
    f.add("F-X4.reject.a_br.double", csv(&shown.a_br))?;
    f.add("F-X4.reject.cr.double", csv(&shown.cr))?;
    let lost: i64 = shown.exact.br.iter().zip(&shown.br).map(|(&x, &y)| x - y).sum();
    f.add("F-X4.reject.lost", lost)?;
    let both = |a: &Matrix<2>, b: &Matrix<2>, c: &Matrix<2>| -> (Vec<bool>, Vec<bool>) {
        small
            .iter()
            .map(|t| {
                let rt = rounded_toss(a, b, c, t);
                (rt.exact.passes(), rt.passes())
            })
            .unzip()
    };
    let (exact, doubled) = both(&a, &b, &c);
    f.add("F-X4.reject.exact_matches", csv(&exact))?;
    f.add("F-X4.reject.double_matches", csv(&doubled))?;

    // 04_5: the wrong C, on the first toss exact sums catch and doubles pass
    let [a, b, c] = ROUNDED_PASSES;
    let shown = small
        .iter()
        .map(|t| rounded_toss(&a, &b, &c, t))
        .find(|rt| !rt.exact.passes() && rt.passes())
        .ok_or("no toss shows the wrong C passing only in doubles")?;
    f.add("F-X4.pass.shown_toss", model::toss_text(&shown.toss))?;
    f.add("F-X4.pass.a_br.exact", csv(&shown.exact.a_br))?;
    f.add("F-X4.pass.a_br.double", csv(&shown.a_br))?;
    f.add("F-X4.pass.cr.double", csv(&shown.cr))?;
    let d = model::mistake(&c, &model::product(&a, &b).0);
    let wrong = model::wrong_entries(&d);
    let [(i, j)] = wrong[..] else {
        return Err("the wrong C must have exactly one wrong entry".into());
    };
    f.add("F-X4.pass.mistake", d[i][j])?;
    let (exact, doubled) = both(&a, &b, &c);
    f.add("F-X4.pass.exact_matches", csv(&exact))?;
    f.add("F-X4.pass.double_matches", csv(&doubled))?;
    f.add("F-X4.pass.exact_catches", exact.iter().filter(|&&m| !m).count())?;

    // 04_6: the smallest tolerance that lets the right C through on every
    // toss in doubles; with it the wrong C passes every toss in exact sums
    let tolerance = right_tolerance();
    let (right, wrong) = (right_residuals(), wrong_residuals());
    let within = |residuals: &[i64]| residuals.iter().filter(|&&r| r <= tolerance).count();
    f.add("F-X4.tolerance", tolerance)?;
    f.add("F-X4.tolerance.right_residuals", csv(&right))?;
    f.add("F-X4.tolerance.right_passes", within(right.as_slice()))?;
    f.add("F-X4.tolerance.wrong_residuals", csv(&wrong))?;
    f.add("F-X4.tolerance.wrong_passes", within(wrong.as_slice()))?;
    if tolerance <= 0 || within(right.as_slice()) != small.len() || within(wrong.as_slice()) != small.len() {
        return Err("the tolerance must let the right C through in doubles and the wrong C in exact sums".into());
    }
    Ok(())
}

pub fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    let ab = true_product();
    let (wrong_row, wrong_col) = wrong_entry()?;
    let by_hand = "worked by hand, row times column (2026-10-19)";
    let one_entry = "one wrong entry hides from exactly the tosses with tails on its column: 2^(n-1) of 2^n";

    // 01_1: the claim
    f.add("F-F1.factors", [A, B].len())?;
    f.pin("F-F1.side", SIDE, "3", "chapter 01's matrices are three by three")?;
    f.add("F-F1.a", matrix_text(&A))?;
    f.add("F-F1.b", matrix_text(&B))?;
    f.pin("F-F1.ab", matrix_text(&ab), "6,2,7;5,1,9;8,3,4", by_hand)?;
    f.add("F-F1.c", matrix_text(&C))?;
    f.add(
        "F-F1.wrong_entries",
        model::wrong_entries(&model::mistake(&C, &ab)).len(),
    )?;
    f.add("F-F1.wrong.row", wrong_row + 1)?;
    f.add("F-F1.wrong.column", wrong_col + 1)?;
    f.add("F-F1.wrong.claimed", C[wrong_row][wrong_col])?;
    f.add("F-F1.wrong.true", ab[wrong_row][wrong_col])?;

    // 01_2, 01_3: redoing the work, and checking one entry
    let (_, product_mults) = model::product(&A, &B);
    let terms = spot_products();
    let (agreeing, entries) = model::spot_checks(&A, &B, &C);
    f.add("F-F2.spot.row", SPOT.0 + 1)?;
    f.add("F-F2.spot.column", SPOT.1 + 1)?;
    f.add("F-F2.spot.products", csv(&terms))?;
    f.pin("F-F2.spot.value", terms.iter().sum::<i64>(), "7", by_hand)?;
    f.add("F-F2.spot.claimed", C[SPOT.0][SPOT.1])?;
    f.add("F-F2.spot.entries_checked", [SPOT].len())?;
    f.add("F-F2.per_entry", terms.len())?;
    f.add("F-F2.entries", entries)?;
    f.pin(
        "F-F2.product_multiplications",
        product_mults,
        "27",
        "three multiplications for each of nine entries",
    )?;
    f.add("F-F2.spot.agreeing", agreeing)?;
    f.add("F-F2.spot.miss_chance", ratio(agreeing as i64, entries as i64)?)?;
    let first = first_check();
    f.add("F-F2.check_multiplications", first.multiplications)?;
    let side = SIDE as u64;
    if model::product_multiplications(side) != product_mults
        || model::check_multiplications(side) != first.multiplications
    {
        return Err("the cost formulas disagree with the counted work".into());
    }
    f.add("F-F2.big.side", BIG_SIDE)?;
    f.pin(
        "F-F2.big.product_multiplications",
        model::product_multiplications(BIG_SIDE),
        "1000000000",
        "a billion: a thousand cubed",
    )?;
    f.add("F-F2.big.check_multiplications", model::check_multiplications(BIG_SIDE))?;

    // 01_4 to 01_6: the first toss
    f.add("F-F3.tried", [FIRST_TOSS].len())?;
    f.add("F-F3.toss", model::toss_text(&FIRST_TOSS))?;
    f.add("F-F3.r", csv(&model::column_of(&FIRST_TOSS)))?;
    let heads: Vec<usize> = FIRST_TOSS
        .iter()
        .enumerate()
        .filter(|&(_, &h)| h)
        .map(|(j, _)| j + 1)
        .collect();
    f.add("F-F3.heads_columns", csv(&heads))?;
    f.add("F-F3.br", csv(&first.br))?;
    f.add("F-F3.a_br", csv(&first.a_br))?;
    f.add("F-F3.cr", csv(&first.cr))?;
    let rows: Vec<usize> = first.disagreements().iter().map(|i| i + 1).collect();
    f.add("F-F3.disagree_rows", csv(&rows))?;
    f.add("F-F3.passes", first.passes())?;
    f.add("F-F3.multiplications", first.multiplications)?;

    // 01_7 to 01_9: every toss
    let checks = all_checks();
    let passing = model::passing(&A, &B, &C);
    let texts: Vec<String> = passing.iter().map(model::toss_text).collect();
    f.add("F-F4.coins", FIRST_TOSS.len())?;
    f.add("F-F4.tosses", checks.len())?;
    f.add("F-F4.passing", texts.join(","))?;
    f.pin("F-F4.misses", passing.len(), "4", one_entry)?;
    f.add("F-F4.miss_fraction", ratio(passing.len() as i64, checks.len() as i64)?)?;
    let tails_on_wrong = checks.iter().all(|(t, ch)| ch.passes() == !t[wrong_col]);
    f.add("F-F4.misses_are_tails_on_wrong_column", tails_on_wrong)?;
    f.add("F-F4.right_answer.passing", model::passing(&A, &B, &ab).len())?;
    // the example meets the theorem's bound exactly: a separate fact, not
    // where the bound comes from (the pairing proof is)
    let bound = theorem_bound()?;
    let bound_text = ratio(bound.0 as i64, bound.1 as i64)?;
    let example_share = ratio(passing.len() as i64, checks.len() as i64)?;
    f.add("F-F4.example_meets_bound", example_share == bound_text)?;

    // 01_10: toss again, fresh
    let (both, pairs) = model::both_pass(&A, &B, &C);
    f.add("F-F5.two.pairs", pairs)?;
    f.add("F-F5.two.both_miss", both)?;
    f.add("F-F5.two.fraction", ratio(both as i64, pairs as i64)?)?;
    let (reused, tosses) = model::same_toss_twice(&A, &B, &C);
    f.add("F-F5.same_toss_twice.misses", reused)?;
    f.add("F-F5.same_toss_twice.fraction", ratio(reused as i64, tosses as i64)?)?;
    f.pin(
        "F-F5.bound_per_toss",
        &bound_text,
        "1/2",
        "the pairing proof, at most one toss of each pair differing in one coin passes; UIUC CS 473 (Fall \
         2019) lecture 12: Pr[x·r = y·r | x ≠ y] ≤ 1/2 for r uniform in {0,1}^n, over any ring",
    )?;
    let odds = again_odds()?;
    f.add("F-F5.again", AGAIN)?;
    f.add("F-F5.again.bound", format!("1/{odds}"))?;
    f.add("F-F5.million", MILLION)?;
    f.add("F-F5.again.under_one_in_a_million", odds > MILLION)?;
    let again = u64::from(AGAIN) * model::check_multiplications(BIG_SIDE);
    f.add("F-F5.big.again_multiplications", again)?;
    f.add(
        "F-F5.big.again_share",
        ratio(again as i64, model::product_multiplications(BIG_SIDE) as i64)?,
    )?;

    // the edges of "no wrong answer hides from more than half the tosses"
    let s3 = model::sweep(&A, &B, &SWEEP_3);
    sweep_facts(&mut f, "n3", &SWEEP_3, s3, model::tosses::<SIDE>().len())?;
    let s2 = model::sweep(&A2, &B2, &SWEEP_2);
    sweep_facts(&mut f, "n2", &SWEEP_2, s2, model::tosses::<2>().len())?;
    for (name, d) in SHAPES {
        f.add(
            &format!("F-X2.{name}.misses"),
            model::passing(&A, &B, &model::plus(&ab, &d)).len(),
        )?;
    }

    // the edge of "fresh": a toss known before C was written can be fooled
    let fooled = model::tosses::<SIDE>()
        .iter()
        .filter(|&t| model::blind_spot(t).is_some_and(|d| model::check(&A, &B, &model::plus(&ab, &d), t).passes()))
        .count();
    f.add("F-X3.known_toss.fooled", fooled)?;
    f.add("F-X3.known_toss.tosses", checks.len())?;

    // the scope edge: exact arithmetic, not floating point
    let small = model::tosses::<2>();
    let [a, b, c] = ROUNDED_REJECTS;
    let failing: Vec<String> = small
        .iter()
        .filter(|&t| !model::rounded_passes(&a, &b, &c, t))
        .map(model::toss_text)
        .collect();
    f.add("F-X4.tosses", small.len())?;
    f.add("F-X4.reject.right_answer", model::product(&a, &b).0 == c)?;
    f.add(
        "F-X4.reject.exact_failures",
        small.iter().filter(|&t| !model::check(&a, &b, &c, t).passes()).count(),
    )?;
    f.add("F-X4.reject.rounded_failures", failing.len())?;
    f.add("F-X4.reject.failing_toss", failing.join(","))?;
    let [a, b, c] = ROUNDED_PASSES;
    f.add(
        "F-X4.pass.wrong_entries",
        model::wrong_entries(&model::mistake(&c, &model::product(&a, &b).0)).len(),
    )?;
    f.add(
        "F-X4.pass.exact_passes",
        small.iter().filter(|&t| model::check(&a, &b, &c, t).passes()).count(),
    )?;
    f.add(
        "F-X4.pass.rounded_passes",
        small.iter().filter(|&t| model::rounded_passes(&a, &b, &c, t)).count(),
    )?;

    chapter_02(&mut f, &ab)?;
    chapter_03(&mut f, &ab)?;
    chapter_04(&mut f)?;
    f.self_check()?;
    Ok(f)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The values the oracle prints for these keys (kept in step with
    /// oracle.py; `./x freivalds facts-check` compares the two outputs).
    #[test]
    fn the_facts_print_what_the_oracle_prints() {
        let f = facts().unwrap();
        for (key, value) in [
            ("F-F1.a", "1,2,1;0,3,1;3,2,0"),
            ("F-F1.c", "6,2,7;5,2,9;8,3,4"),
            ("F-F1.wrong_entries", "1"),
            ("F-F1.wrong.row", "2"),
            ("F-F1.wrong.column", "2"),
            ("F-F1.wrong.claimed", "2"),
            ("F-F1.wrong.true", "1"),
            ("F-F2.spot.products", "0,4,3"),
            ("F-F2.spot.claimed", "7"),
            ("F-F2.spot.miss_chance", "8/9"),
            ("F-F2.check_multiplications", "9"),
            ("F-F2.big.check_multiplications", "1000000"),
            ("F-F3.r", "1,1,0"),
            ("F-F3.heads_columns", "1,2"),
            ("F-F3.br", "3,1,3"),
            ("F-F3.a_br", "8,6,11"),
            ("F-F3.cr", "8,7,11"),
            ("F-F3.disagree_rows", "2"),
            ("F-F3.passes", "false"),
            ("F-F4.passing", "000,001,100,101"),
            ("F-F4.miss_fraction", "1/2"),
            ("F-F4.misses_are_tails_on_wrong_column", "true"),
            ("F-F4.right_answer.passing", "8"),
            ("F-F4.example_meets_bound", "true"),
            ("F-F5.bound_per_toss", "1/2"),
            ("F-F5.two.both_miss", "16"),
            ("F-F5.two.fraction", "1/4"),
            ("F-F5.same_toss_twice.fraction", "1/2"),
            ("F-F5.again.bound", "1/1048576"),
            ("F-F5.again.under_one_in_a_million", "true"),
            ("F-F5.big.again_share", "1/50"),
            ("F-X1.n3.wrong", "19682"),
            ("F-X1.n3.most", "4"),
            ("F-X1.n3.fewest", "1"),
            ("F-X1.n3.at_half", "156"),
            ("F-X1.n3.uneven", "78"),
            ("F-X1.n3.pairs_both_passing", "0"),
            ("F-X1.n3.attains_bound", "true"),
            ("F-X1.n2.attains_bound", "true"),
            ("F-X1.n2.wrong", "624"),
            ("F-X1.n2.most_share", "1/2"),
            ("F-X2.one_entry.misses", "4"),
            ("F-X2.two_in_a_row.misses", "2"),
            ("F-X2.cancelling.misses", "4"),
            ("F-X2.three_in_a_row.misses", "3"),
            ("F-X2.two_rows.misses", "2"),
            ("F-X2.every_column.misses", "1"),
            ("F-X3.known_toss.fooled", "8"),
            ("F-X4.reject.right_answer", "true"),
            ("F-X4.reject.exact_failures", "0"),
            ("F-X4.reject.rounded_failures", "1"),
            ("F-X4.reject.failing_toss", "11"),
            ("F-X4.pass.wrong_entries", "1"),
            ("F-X4.pass.exact_passes", "2"),
            ("F-X4.pass.rounded_passes", "4"),
            ("F-F6.slip.mistake", "0,0,0;0,1,0;0,0,0"),
            ("F-F6.first_toss.heads_sum", "0,1,0"),
            ("F-F6.difference_is_mistake_sum", "true"),
            ("F-F6.misses_are_zero_sums", "true"),
            ("F-F7.two_in_a_row.hides_from", "000,100"),
            ("F-F7.cancelling.hides_from", "000,011,100,111"),
            ("F-F7.three_in_a_row.wrong_values", "1,1,-1"),
            ("F-F7.three_in_a_row.hides_from", "000,011,101"),
            ("F-F7.three_in_a_row.miss_share", "3/8"),
            ("F-F7.three_in_a_row.power_of_two", "false"),
            ("F-F7.every_column.hides_from", "000"),
            ("F-F7.every_column.wrong_per_row_and_column", "1"),
            ("F-X5.same_sign.cases", "18"),
            ("F-X5.same_sign.violations", "0"),
            ("F-X5.cancelling.cases", "6"),
            ("F-X5.one_per_row_and_column.cases", "384"),
            ("F-X5.one_per_row_and_column.violations", "0"),
            ("F-X5.three_in_a_row.wrong_in_every_column", "true"),
            ("F-F8.row", "2"),
            ("F-F8.column", "1"),
            ("F-F8.pairs", "000-100,001-101,010-110,011-111"),
            ("F-F8.row_sums.tails", "0,-1,1,0"),
            ("F-F8.row_sums.heads", "1,0,2,1"),
            ("F-F8.changes", "1,1,1,1"),
            ("F-F8.misses_per_pair", "1,1,0,1"),
            ("F-F8.misses_per_pair_bound", "1"),
            ("F-F8.most_misses_per_pair", "1"),
            ("F-F8.partners_per_toss", "1"),
            ("F-F8.pairs_over_tosses", "1/2"),
            ("F-X5.right_column.column", "1"),
            ("F-X5.right_column.misses_per_pair", "2,2,0,0"),
            ("F-X5.right_column.changes", "0,0,0,0"),
            ("F-F9.tosses", "2"),
            ("F-F9.order", "000,001,100,101,010,011,110,111"),
            ("F-F9.first.misses", "4"),
            ("F-F9.second.misses", "4"),
            ("F-F9.both_miss_is_product", "true"),
            ("F-F9.both_miss_fill_the_corner", "true"),
            ("F-F9.two.bound", "1/4"),
            ("F-F9.two.example_meets_bound", "true"),
            ("F-F9.same_toss.ways", "8"),
            ("F-F9.same_toss.above_two_bound", "true"),
            ("F-X3.blind_spot.000.mistake", "1,0,0;0,0,0;0,0,0"),
            ("F-X3.blind_spot.100.mistake", "0,1,0;0,0,0;0,0,0"),
            ("F-X3.blind_spot.110.mistake", "0,0,1;0,0,0;0,0,0"),
            ("F-X3.blind_spot.111.mistake", "1,-1,0;0,0,0;0,0,0"),
            ("F-X3.blind_spot.tails.wrong_entries", "1"),
            ("F-X3.blind_spot.tails.under_tails", "true"),
            ("F-X3.blind_spot.all_heads.values", "1,-1"),
            ("F-X3.blind_spot.all_heads.one_row", "true"),
            ("F-X3.blind_spot.fresh_catches", "4,4,4,4,4,4,4,4"),
            ("F-X3.blind_spot.fresh_catch_share", "1/2"),
            ("F-X3.n2.fooled", "4"),
            ("F-X3.n2.tosses", "4"),
            ("F-X3.n1.heads_fooled", "false"),
            ("F-X4.big", "10000000000000000"),
            ("F-X4.big_exponent", "16"),
            ("F-X4.plus_one.exact", "10000000000000001"),
            ("F-X4.plus_one.double", "10000000000000000"),
            ("F-X4.lost", "1"),
            ("F-X4.gap", "2"),
            ("F-X4.doubles_near", "-2,0,2"),
            ("F-X4.significand_bits", "53"),
            ("F-X4.big_above_2_53", "true"),
            ("F-X4.reject.br.exact", "10000000000000001,-9999999999999999"),
            ("F-X4.reject.br.double", "10000000000000000,-10000000000000000"),
            ("F-X4.reject.a_br.exact", "2,0"),
            ("F-X4.reject.a_br.double", "0,0"),
            ("F-X4.reject.cr.double", "2,0"),
            ("F-X4.reject.lost", "2"),
            ("F-X4.reject.exact_matches", "true,true,true,true"),
            ("F-X4.reject.double_matches", "true,true,true,false"),
            ("F-X4.pass.shown_toss", "10"),
            ("F-X4.pass.a_br.exact", "10000000000000001,0"),
            ("F-X4.pass.a_br.double", "10000000000000000,0"),
            ("F-X4.pass.cr.double", "10000000000000000,0"),
            ("F-X4.pass.mistake", "-1"),
            ("F-X4.pass.exact_matches", "true,true,false,false"),
            ("F-X4.pass.double_matches", "true,true,true,true"),
            ("F-X4.pass.exact_catches", "2"),
            ("F-X4.tolerance", "2"),
            ("F-X4.tolerance.right_residuals", "0,0,0,2"),
            ("F-X4.tolerance.right_passes", "4"),
            ("F-X4.tolerance.wrong_residuals", "0,0,1,1"),
            ("F-X4.tolerance.wrong_passes", "4"),
        ] {
            assert_eq!(f.get(key), Some(value), "{key}");
        }
    }

    /// The theorem's bound is the pairing proof's one half. The finite edge
    /// tests guard it: a sweep that exceeds it, breaks the pairing step, or
    /// never meets it is refused.
    #[test]
    fn the_theorem_bound_is_guarded_by_the_finite_checks() {
        assert_eq!(theorem_bound(), Ok((1, 2)));
        assert_eq!(again_odds(), Ok(1 << AGAIN));
        assert_eq!(check_bound((1, 2), 4, 8, 0), Ok(()));
        assert!(check_bound((1, 2), 5, 8, 0).is_err(), "more than half");
        assert!(check_bound((1, 2), 4, 8, 1).is_err(), "a coin pair passing twice");
        assert!(check_bound((1, 2), 3, 8, 0).is_err(), "no equality case");
    }

    #[test]
    fn ratios_print_like_python_fractions() {
        assert_eq!(ratio(8, 9).unwrap(), "8/9");
        assert_eq!(ratio(16, 64).unwrap(), "1/4");
        assert_eq!(ratio(8, 4).unwrap(), "2");
        assert_eq!(ratio(-2, 4).unwrap(), "-1/2");
        assert_eq!(ratio(0, 4).unwrap(), "0");
        assert!(ratio(1, 0).is_err());
    }

    /// The claim has exactly one wrong entry; the check refuses a right
    /// answer (none) and a mistake in two places.
    #[test]
    fn the_claim_names_its_one_wrong_entry() {
        assert_eq!(wrong_entry(), Ok((1, 1)));
        assert!(one_wrong_entry(&true_product()).is_err());
        let two = model::plus(&true_product(), &SHAPES[1].1);
        assert!(one_wrong_entry(&two).is_err());
    }

    #[test]
    fn the_printing_matches_the_oracle_format() {
        assert_eq!(csv(&[1, -2, 3]), "1,-2,3");
        assert_eq!(csv::<usize>(&[]), "");
        assert_eq!(matrix_text(&[[1, 2], [3, 4]]), "1,2;3,4");
    }
}
