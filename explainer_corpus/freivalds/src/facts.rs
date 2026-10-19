//! Every number the video shows or says, computed from [`crate::model`] and
//! printed for `oracle.py` (`./x freivalds facts-check`). F-F rows are the
//! example's numbers; F-X rows are the edges of the video's always/never
//! sentences. Matrices print row by row (`1,2,1;0,3,1;3,2,0`), tosses as
//! their coins (`110`), and rows and columns count from 1, as the screen
//! does.

use explainer_kit::Facts;

use crate::model::{
    self, A, A2, B, B2, C, Check, FIRST_TOSS, Matrix, ROUNDED_PASSES, ROUNDED_REJECTS, SHAPES, SIDE, SPOT, Toss,
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

/// The pairing proof's bound on one toss, (num, den), for any ring and any
/// n (UIUC CS 473, Fall 2019, lecture 12). Pick a column j where C is wrong
/// and pair each toss with the one that differs only in coin j. Between the
/// two, a row where C is wrong in column j changes by that nonzero error, so
/// at most one toss of each pair of two passes. With fair coins, all
/// independent and tossed after C is fixed, every toss is equally likely,
/// so that share is the chance.
const PAIRING_BOUND: (u64, u64) = (1, 2);

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
