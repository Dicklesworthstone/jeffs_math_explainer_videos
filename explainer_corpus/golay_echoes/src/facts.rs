//! Every number the video shows or says, computed from [`crate::model`] and
//! printed for [`oracle.py`](../oracle.py) (`./x golay_echoes facts-check`):
//! the rows of docs/fact-sheet.md. G-F rows are computed, G-X rows are the
//! edges of the always/never sentences.

use std::collections::BTreeSet;

use explainer_kit::Facts;

use crate::model::{self, CODE_A, CODE_B, LOUD_AND_QUIET, MOVED, Product, STILL, Scores, Span};

/// The shifts chapter 01 draws a score for: −3 to 3 for one echo, and one
/// more on the right for the echo that moved.
pub const SHIFTS: (i64, i64) = (-3, 4);

/// The shifts chapter 02 draws: every shift a four-pulse pattern scores in
/// the loud-and-quiet scene.
pub const SHIFTS_02: (i64, i64) = (-3, 6);

/// Code A's scores against its own echo.
pub fn score_a() -> Scores {
    model::autocorrelation(&CODE_A)
}

/// Code B's scores against its own echo.
pub fn score_b() -> Scores {
    model::autocorrelation(&CODE_B)
}

/// The two measurements of one still echo, added.
pub fn sum() -> Scores {
    model::two_measurements(&CODE_A, &STILL, &CODE_B, &STILL)
}

/// The two measurements added when the echo moved one step in between.
pub fn moved() -> Scores {
    model::two_measurements(&CODE_A, &STILL, &CODE_B, &MOVED)
}

/// Chapter 02: pattern A sent into the loud-and-quiet scene, scored.
pub fn hidden_a() -> Scores {
    model::measure(&CODE_A, &LOUD_AND_QUIET, &[])
}

/// Chapter 02: pattern B sent into the same scene, scored.
pub fn hidden_b() -> Scores {
    model::measure(&CODE_B, &LOUD_AND_QUIET, &[])
}

/// Chapter 02: the two rows added.
pub fn hidden_sum() -> Scores {
    model::two_measurements(&CODE_A, &LOUD_AND_QUIET, &CODE_B, &LOUD_AND_QUIET)
}

/// Chapter 02's wrong pair: pattern A sent twice, the two rows added.
pub fn a_twice() -> Scores {
    model::two_measurements(&CODE_A, &LOUD_AND_QUIET, &CODE_A, &LOUD_AND_QUIET)
}

/// Chapter 02's two echoes, (step, strength): the loud one, then the quiet.
pub fn loud_and_quiet() -> Result<[(i64, i64); 2], String> {
    let echoes: Vec<(i64, i64)> = LOUD_AND_QUIET
        .iter()
        .enumerate()
        .filter(|&(_, &h)| h != 0)
        .map(|(d, &h)| (d as i64, h))
        .collect();
    match echoes[..] {
        [loud, quiet] if loud.1 > quiet.1 && quiet.1 > 0 => Ok([loud, quiet]),
        _ => Err(format!("{LOUD_AND_QUIET:?} is not a loud echo and then a quiet one")),
    }
}

/// What the echo at `step` adds to the row at shift `k`.
pub fn part(parts: &[(i64, Scores)], step: i64, k: i64) -> i64 {
    parts.iter().find(|(d, _)| *d == step).map_or(0, |(_, r)| r.at(k))
}

/// Every code of `n` up/down pulses.
fn codes(n: usize) -> Vec<Vec<i64>> {
    (0u32..(1 << n))
        .map(|m| (0..n).map(|i| if (m >> i) & 1 == 1 { -1 } else { 1 }).collect())
        .collect()
}

/// Over every ordered pair of four-pulse patterns: how many pairs, how many
/// give chapter 02's scene back exactly (their rows add to 2N times it), how
/// many of those are complementary, and how many are complementary.
pub fn pairs_giving_back() -> [usize; 4] {
    let all = codes(CODE_A.len());
    let times = 2 * CODE_A.len() as i64;
    let want = |k: i64| {
        usize::try_from(k)
            .ok()
            .and_then(|d| LOUD_AND_QUIET.get(d))
            .map_or(0, |&h| times * h)
    };
    let mut counts = [all.len() * all.len(), 0, 0, 0];
    for a in &all {
        for b in &all {
            let s = model::two_measurements(a, &LOUD_AND_QUIET, b, &LOUD_AND_QUIET);
            let back = (s.first..=s.last()).all(|k| s.at(k) == want(k));
            let complementary = model::is_complementary(a, b);
            counts[1] += usize::from(back);
            counts[2] += usize::from(back && complementary);
            counts[3] += usize::from(complementary);
        }
    }
    counts
}

/// The shift chapter 03 looks at: one step.
pub const SHIFT_03: i64 = 1;

/// Chapter 03's pairs: one pulse paired with itself, doubled, and doubled
/// again (patterns A and B).
pub fn doublings() -> [(Vec<i64>, Vec<i64>); 3] {
    let seed = (vec![1], vec![1]);
    let two = model::double(&seed.0, &seed.1);
    let four = model::double(&two.0, &two.1);
    [seed, two, four]
}

/// The products of the pattern `x then y` at shift `k`, each with where it
/// sits (inside a half, or across the middle).
pub fn split(x: &[i64], y: &[i64], k: i64) -> Vec<(Product, Span)> {
    let whole: Vec<i64> = x.iter().chain(y).copied().collect();
    model::products(&whole, &whole, k)
        .into_iter()
        .map(|p| (p, model::span(x.len(), &p)))
        .collect()
}

/// The doubling identity's numbers at chapter 03's shift: its factor (the
/// new pair's peak over the old pair's), the old halves' scores there, and
/// the new pair's total there.
pub fn doubling_numbers() -> Result<(String, [i64; 2], i64), String> {
    let [_, two, four] = doublings();
    let (row_x, row_y) = (model::autocorrelation(&two.0), model::autocorrelation(&two.1));
    let new = model::autocorrelation(&four.0).plus(&model::autocorrelation(&four.1));
    let factor = ratio(new.at(0), row_x.at(0) + row_y.at(0))?;
    Ok((factor, [row_x.at(SHIFT_03), row_y.at(SHIFT_03)], new.at(SHIFT_03)))
}

/// The lengths doubling reaches from one pulse, each pair checked to
/// cancel (1 to 64).
pub fn doubling_lengths() -> Result<Vec<i64>, String> {
    let [seed, ..] = doublings();
    let (mut p, mut q) = seed;
    let mut lengths = Vec::new();
    for _ in 0..=6 {
        if !model::is_complementary(&p, &q) {
            return Err(format!("the doubling of length {} does not cancel", p.len()));
        }
        lengths.push(p.len() as i64);
        (p, q) = model::double(&p, &q);
    }
    Ok(lengths)
}

/// The values of the products in `pairs` that sit `want`.
fn values(pairs: &[(Product, Span)], want: Option<Span>) -> Vec<i64> {
    pairs
        .iter()
        .filter(|(_, s)| want.is_none_or(|w| *s == w))
        .map(|(p, _)| p.value)
        .collect()
}

/// The key's two pairs (01_3), pulses of pattern A: its first two (they
/// agree) and its last two (they disagree).
pub fn key_pairs() -> [(i64, i64); 2] {
    [(CODE_A[0], CODE_A[1]), (CODE_A[2], CODE_A[3])]
}

/// Where a scene's one echo is (its step).
pub fn echo_step(scene: &[i64]) -> Result<i64, String> {
    let mut steps = scene
        .iter()
        .enumerate()
        .filter(|&(_, &h)| h != 0)
        .map(|(d, _)| d as i64);
    match (steps.next(), steps.next()) {
        (Some(d), None) => Ok(d),
        _ => Err(format!("scene {scene:?} does not hold exactly one echo")),
    }
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

/// The one size every far-shift score has (the edge of 01_7's sentence),
/// or an error when the patterns disagree.
pub fn far_shift_size() -> Result<i64, String> {
    let sizes = far_shift_sizes();
    match (sizes.first(), sizes.len()) {
        (Some(&size), 1) => Ok(size),
        _ => Err(format!("far-shift scores come in sizes {sizes:?}, not one")),
    }
}

/// The size of the far-shift score of every code of 2 to 10 up/down pulses.
fn far_shift_sizes() -> BTreeSet<i64> {
    (2..=10usize)
        .flat_map(codes)
        .map(|code| model::far_shift_score(&code).abs())
        .collect()
}

pub fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    let n = CODE_A.len() as i64;
    let (a, b, both, moved) = (score_a(), score_b(), sum(), moved());
    let one_off: Vec<i64> = model::products(&CODE_A, &CODE_A, 1).iter().map(|p| p.value).collect();
    let count = |sign: i64| one_off.iter().filter(|&&p| p == sign).count();
    let defined = "Chi, Pezeshki and Calderbank, Complementary Waveforms for Sidelobe Suppression and Radar \
                   Polarimetry, definition 1: Ca(k) + Cb(k) = 2N delta(k)";
    let example = "Wikipedia, Complementary sequences (read 2026-10-17): (+1 +1 +1 -1) and (+1 +1 -1 +1) \
                   sum to (8, 0, 0, 0) at lags 0 to 3";

    f.pin("G-F1.pulses", n, "4", "chapter 01's codes are four pulses long")?;
    f.pin("G-F1.code_a", model::csv(&CODE_A), "1,1,1,-1", example)?;
    f.pin("G-F1.code_b", model::csv(&CODE_B), "1,1,-1,1", example)?;
    f.pin(
        "G-F1.complementary",
        model::is_complementary(&CODE_A, &CODE_B),
        "true",
        defined,
    )?;
    let [(a0, a1), (a2, a3)] = key_pairs();
    f.add("G-F1.key.agree", a0 * a1)?;
    f.add("G-F1.key.disagree", a2 * a3)?;

    // 01_4 to 01_7: code A on its own
    f.add("G-F2.a.scores", a.csv())?;
    f.pin("G-F2.a.peak", a.at(0), "4", "all four pulses agree with themselves")?;
    f.add("G-F2.a.one_off.products", model::csv(&one_off))?;
    f.add("G-F2.a.one_off.pairs", one_off.len())?;
    f.add("G-F2.a.one_off.agree", count(1))?;
    f.add("G-F2.a.one_off.disagree", count(-1))?;
    f.add("G-F2.a.one_off.score", a.at(1))?;
    f.add("G-F2.ghost_fraction", ratio(a.largest_ghost(&[0]), a.at(0))?)?;
    f.add("G-F2.a.ghost_shifts", model::csv(&a.ghost_shifts(&[0])))?;
    f.add("G-F2.a.far_shift_score", model::far_shift_score(&CODE_A))?;

    // 01_8: code B on its own
    f.add("G-F2.b.scores", b.csv())?;
    f.add("G-F2.b.ghost_shifts", model::csv(&b.ghost_shifts(&[0])))?;
    f.add("G-F2.b.largest_ghost", b.largest_ghost(&[0]))?;

    // 01_10: the two rows added
    f.pin("G-F3.sum.scores", both.csv(), "0,0,0,8,0,0,0", example)?;
    f.pin("G-F3.sum.peak", both.at(0), "8", defined)?;
    f.pin("G-F3.sum.largest_ghost", both.largest_ghost(&[0]), "0", defined)?;
    f.add("G-F3.peak_ratio", ratio(both.at(0), a.at(0))?)?;

    // 01_12: the echo moved one step between the two sends
    let echoes = [echo_step(&STILL)?, echo_step(&MOVED)?];
    f.add("G-F4.moved.step", echoes[1] - echoes[0])?;
    f.add("G-F4.moved.first_shift", moved.first)?;
    f.add("G-F4.moved.scores", moved.csv())?;
    f.add("G-F4.moved.largest_ghost", moved.largest_ghost(&echoes))?;
    f.add("G-F4.moved.ghost_shifts", model::csv(&moved.ghost_shifts(&echoes)))?;

    // the edge of "no single code is free of ghosts"
    let sizes: Vec<i64> = far_shift_sizes().into_iter().collect();
    f.add("G-X1.far_shift_sizes", model::csv(&sizes))?;

    // the rows the chapter draws fit its shifts
    for (name, row) in [("A", &a), ("B", &b), ("the sum", &both), ("the moved sum", &moved)] {
        if row.first < SHIFTS.0 || row.last() > SHIFTS.1 {
            return Err(format!(
                "{name}'s scores run {}..={}, past the drawn shifts {SHIFTS:?}",
                row.first,
                row.last()
            ));
        }
    }
    chapter_02(&mut f)?;
    chapter_03(&mut f)?;
    f.self_check()?;
    Ok(f)
}

/// Chapter 03's numbers: the doubling, one shift's products sorted by where
/// they sit, the identity's factor, its reach and its limits.
fn chapter_03(f: &mut Facts) -> Result<(), String> {
    let [seed, two, four] = doublings();
    let pair = |p: &(Vec<i64>, Vec<i64>)| format!("{};{}", model::csv(&p.0), model::csv(&p.1));
    f.pin("G-F10.seed", pair(&seed), "1;1", "one pulse, paired with itself")?;
    f.add("G-F10.seed_length", seed.0.len())?;
    f.add("G-F10.length_two", pair(&two))?;
    f.pin(
        "G-F10.length_four",
        pair(&four),
        "1,1,1,-1;1,1,-1,1",
        "Wikipedia, Complementary sequences (read 2026-10-17): the length-4 pair",
    )?;
    if four.0 != CODE_A || four.1 != CODE_B {
        return Err(format!("doubling twice gives {four:?}, not patterns A and B"));
    }

    // 03_4 to 03_8: one step, A's and B's products, sorted by where they sit
    let (x, y) = (&two.0, &two.1);
    let flipped: Vec<i64> = y.iter().map(|v| -v).collect();
    let (a, b) = (split(x, y, SHIFT_03), split(x, &flipped, SHIFT_03));
    f.add("G-F10.shift", SHIFT_03)?;
    f.add("G-F10.a.inside", model::csv(&values(&a, Some(Span::Inside))))?;
    f.add("G-F10.a.across", model::csv(&values(&a, Some(Span::Across))))?;
    f.add("G-F10.b.inside", model::csv(&values(&b, Some(Span::Inside))))?;
    f.add("G-F10.b.across", model::csv(&values(&b, Some(Span::Across))))?;
    // how many of a crossing pair's two pulses B holds flipped against A
    let flips: BTreeSet<i64> = a
        .iter()
        .filter(|(_, s)| *s == Span::Across)
        .map(|(p, _)| {
            [p.recording, p.copy]
                .iter()
                .filter(|&&i| four.0[i] != four.1[i])
                .count() as i64
        })
        .collect();
    f.add(
        "G-F10.flipped_in_crossing",
        model::csv(&flips.into_iter().collect::<Vec<_>>()),
    )?;
    let (factor, old_at_shift, new_at_shift) = doubling_numbers()?;
    f.add("G-F10.old_rows_at_shift", model::csv(&old_at_shift))?;
    let both: Vec<(Product, Span)> = a.iter().chain(&b).copied().collect();
    let total = values(&both, None).iter().sum::<i64>();
    if total != new_at_shift {
        return Err(format!("the products add to {total}, the new rows to {new_at_shift}"));
    }
    f.add(
        "G-F10.inside_total",
        values(&both, Some(Span::Inside)).iter().sum::<i64>(),
    )?;
    f.add("G-F10.total", total)?;
    f.add("G-F10.factor", factor)?;
    f.add("G-F10.length_factor", ratio(four.0.len() as i64, two.0.len() as i64)?)?;

    // 03_9: how far doubling from one pulse reaches, and a pair it misses
    f.add("G-F11.lengths", model::csv(&doubling_lengths()?))?;
    let (g10a, g10b) = model::GOLAY_10;
    f.add("G-F11.length10", g10a.len())?;
    f.add("G-F11.length10_complementary", model::is_complementary(&g10a, &g10b))?;
    f.add("G-F11.length10_power_of_two", g10a.len().is_power_of_two())?;

    // 03_10: the flip left out, pattern A paired with itself
    let no_flip = model::autocorrelation(&four.0).scaled(2);
    let far = 2 * x.len() as i64 - 1;
    // both rows hold pattern A, so each gives the same far product
    let far_one = values(&split(x, y, far), None);
    let far_products = [far_one.as_slice(), far_one.as_slice()].concat();
    f.add("G-F12.no_flip.scores", no_flip.csv())?;
    f.add("G-F12.no_flip.far_products", model::csv(&far_products))?;
    f.add("G-F12.no_flip.far", no_flip.at(far))?;

    // where crossing products begin and end, for the chapter's halves
    let counts: Vec<i64> = (0..2 * x.len() as i64)
        .map(|k| values(&split(x, y, k), Some(Span::Across)).len() as i64)
        .collect();
    f.add("G-X3.crossing_counts", model::csv(&counts))?;
    Ok(())
}

/// Chapter 02's numbers: a loud and a quiet echo three steps apart.
fn chapter_02(f: &mut Facts) -> Result<(), String> {
    let [(loud_step, loud), (quiet_step, quiet)] = loud_and_quiet()?;
    let (a, b, sum, twice) = (hidden_a(), hidden_b(), hidden_sum(), a_twice());
    let parts_a = model::contributions(&CODE_A, &LOUD_AND_QUIET);
    let parts_b = model::contributions(&CODE_B, &LOUD_AND_QUIET);
    let echo_at = |k: i64| k == loud_step || k == quiet_step;

    // 02_1, 02_2: the scene and pattern A's recording of it
    f.pin(
        "G-F5.scene",
        model::csv(&LOUD_AND_QUIET),
        "4,0,0,1",
        "the chapter's fixed scene (mx-video-program-program-87do.6.2)",
    )?;
    f.add("G-F5.echoes", parts_a.len())?;
    f.add("G-F5.loud", loud)?;
    f.add("G-F5.quiet", quiet)?;
    f.add("G-F5.quiet_step", quiet_step - loud_step)?;
    f.add("G-F5.recording_a", model::csv(&model::echo(&CODE_A, &LOUD_AND_QUIET)))?;

    // 02_3 to 02_6: pattern A hides the quiet echo
    f.add("G-F6.a.scores", a.csv())?;
    f.add("G-F6.a.at_quiet", a.at(quiet_step))?;
    f.add("G-F6.a.loud_part_at_quiet", part(&parts_a, loud_step, quiet_step))?;
    f.add("G-F6.a.quiet_part_at_quiet", part(&parts_a, quiet_step, quiet_step))?;
    let beside = [
        part(&parts_a, loud_step, loud_step - 1),
        part(&parts_a, loud_step, loud_step + 1),
    ];
    f.add("G-F6.a.loud_ghosts_one_step", model::csv(&beside))?;
    let hidden: Vec<i64> = (a.first..=a.last()).filter(|&k| echo_at(k) && a.at(k) == 0).collect();
    let ghosts: Vec<i64> = (a.first..=a.last()).filter(|&k| !echo_at(k) && a.at(k) != 0).collect();
    f.add("G-F6.a.zero_at_echo", model::csv(&hidden))?;
    f.add("G-F6.a.score_without_echo", model::csv(&ghosts))?;

    // 02_7, 02_8: pattern B doubles it
    f.add("G-F7.b.scores", b.csv())?;
    f.add("G-F7.b.at_quiet", b.at(quiet_step))?;
    f.add("G-F7.b.loud_part_at_quiet", part(&parts_b, loud_step, quiet_step))?;
    f.add(
        "G-F7.b.over_quiet",
        ratio(b.at(quiet_step), part(&parts_a, quiet_step, quiet_step))?,
    )?;

    // 02_9: added, the scene itself
    f.pin(
        "G-F8.sum.scores",
        sum.csv(),
        "0,0,0,32,0,0,8,0,0,0",
        "8 times the scene 4, 0, 0, 1 (definition 1 with N = 4), shifts -3 to 6",
    )?;
    f.add("G-F8.sum.at_loud", sum.at(loud_step))?;
    f.add("G-F8.sum.at_quiet", sum.at(quiet_step))?;
    f.add("G-F8.sum.largest_ghost", sum.largest_ghost(&[loud_step, quiet_step]))?;
    let (by_loud, by_quiet) = (ratio(sum.at(loud_step), loud)?, ratio(sum.at(quiet_step), quiet)?);
    if by_loud != by_quiet {
        return Err(format!(
            "the sum magnifies the loud echo {by_loud} times, the quiet one {by_quiet}"
        ));
    }
    f.add("G-F8.magnification", by_quiet)?;

    // 02_10: the wrong pair, A sent twice, and the theorem's edge
    let sends = [CODE_A, CODE_A];
    f.add("G-F9.a_twice.sends", sends.len())?;
    f.add("G-F9.a_twice.scores", twice.csv())?;
    f.add("G-F9.a_twice.at_quiet", twice.at(quiet_step))?;
    f.add(
        "G-F9.a_twice.at_one_step",
        model::csv(&[twice.at(loud_step - 1), twice.at(loud_step + 1)]),
    )?;
    let [pairs, giving_back, giving_back_complementary, complementary] = pairs_giving_back();
    f.add("G-X2.pairs", pairs)?;
    f.add("G-X2.giving_back", giving_back)?;
    f.add("G-X2.giving_back_complementary", giving_back_complementary)?;
    f.add("G-X2.complementary", complementary)?;

    // every row chapter 02 draws fits its shifts
    let mut rows = vec![("A", &a), ("B", &b), ("the sum", &sum), ("A twice", &twice)];
    rows.extend(parts_a.iter().map(|(_, r)| ("an echo's part of A", r)));
    for (name, row) in rows {
        if row.first < SHIFTS_02.0 || row.last() > SHIFTS_02.1 {
            return Err(format!(
                "{name}'s scores run {}..={}, past chapter 02's shifts {SHIFTS_02:?}",
                row.first,
                row.last()
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The values the oracle prints for these keys (kept in step with
    /// oracle.py; `./x golay_echoes facts-check` compares the two outputs).
    #[test]
    fn the_facts_print_what_the_oracle_prints() {
        let f = facts().unwrap();
        for (key, value) in [
            ("G-F10.length_two", "1,1;1,-1"),
            ("G-F10.a.inside", "1,-1"),
            ("G-F10.a.across", "1"),
            ("G-F10.b.across", "-1"),
            ("G-F10.flipped_in_crossing", "1"),
            ("G-F10.total", "0"),
            ("G-F10.factor", "2"),
            ("G-F11.lengths", "1,2,4,8,16,32,64"),
            ("G-F11.length10_complementary", "true"),
            ("G-F12.no_flip.scores", "-2,0,2,8,2,0,-2"),
            ("G-X3.crossing_counts", "0,1,2,1"),
            ("G-F5.recording_a", "4,4,4,-3,1,1,-1"),
            ("G-F6.a.scores", "-4,0,4,15,4,1,0,1,0,-1"),
            ("G-F6.a.at_quiet", "0"),
            ("G-F6.a.loud_part_at_quiet", "-4"),
            ("G-F6.a.quiet_part_at_quiet", "4"),
            ("G-F6.a.zero_at_echo", "3"),
            ("G-F6.a.score_without_echo", "-3,-1,1,2,4,6"),
            ("G-F7.b.scores", "4,0,-4,17,-4,-1,8,-1,0,1"),
            ("G-F7.b.over_quiet", "2"),
            ("G-F8.magnification", "8"),
            ("G-F9.a_twice.at_quiet", "0"),
            ("G-X2.giving_back", "32"),
            ("G-X2.giving_back_complementary", "32"),
            ("G-F2.a.scores", "-1,0,1,4,1,0,-1"),
            ("G-F2.b.scores", "1,0,-1,4,-1,0,1"),
            ("G-F2.a.one_off.products", "1,1,-1"),
            ("G-F2.a.one_off.agree", "2"),
            ("G-F2.a.one_off.disagree", "1"),
            ("G-F2.ghost_fraction", "1/4"),
            ("G-F2.a.ghost_shifts", "-3,-1,1,3"),
            ("G-F2.b.ghost_shifts", "-3,-1,1,3"),
            ("G-F3.peak_ratio", "2"),
            ("G-F4.moved.scores", "-1,1,1,3,5,-1,-1,1"),
            ("G-F4.moved.ghost_shifts", "-3,-2,-1,2,3,4"),
            ("G-X1.far_shift_sizes", "1"),
        ] {
            assert_eq!(f.get(key), Some(value), "{key}");
        }
    }

    #[test]
    fn ratios_print_like_python_fractions() {
        assert_eq!(ratio(1, 4).unwrap(), "1/4");
        assert_eq!(ratio(8, 4).unwrap(), "2");
        assert_eq!(ratio(-2, 4).unwrap(), "-1/2");
        assert_eq!(ratio(2, -4).unwrap(), "-1/2");
        assert_eq!(ratio(0, 4).unwrap(), "0");
        assert!(ratio(1, 0).is_err());
    }

    #[test]
    fn a_scene_names_its_one_echo() {
        assert_eq!(echo_step(&STILL), Ok(0));
        assert_eq!(echo_step(&MOVED), Ok(1));
        assert!(echo_step(&[1, 1]).is_err());
        assert!(echo_step(&[0]).is_err());
    }
}
