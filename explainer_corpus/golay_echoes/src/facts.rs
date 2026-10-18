//! Every number the video shows or says, computed from [`crate::model`] and
//! printed for [`oracle.py`](../oracle.py) (`./x golay_echoes facts-check`):
//! the rows of docs/fact-sheet.md. G-F rows are computed, G-X rows are the
//! edges of the always/never sentences.

use std::collections::BTreeSet;

use explainer_kit::Facts;

use crate::model::{self, CODE_A, CODE_B, MOVED, STILL, Scores};

/// The shifts chapter 01 draws a score for: −3 to 3 for one echo, and one
/// more on the right for the echo that moved.
pub const SHIFTS: (i64, i64) = (-3, 4);

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

/// Where a scene's one echo is (its step).
pub fn echo_step(scene: &[i64]) -> Result<i64, String> {
    let mut steps = scene.iter().enumerate().filter(|&(_, &h)| h != 0).map(|(d, _)| d as i64);
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
        .flat_map(|n| {
            (0u32..(1 << n)).map(move |m| {
                let code: Vec<i64> = (0..n).map(|i| if (m >> i) & 1 == 1 { -1 } else { 1 }).collect();
                model::far_shift_score(&code).abs()
            })
        })
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
    f.self_check()?;
    Ok(f)
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
