//! Every number "Epsilon and Delta: Why Getting Closer Wasn't Enough" shows
//! or says, computed here from `model.rs` and checked two ways: `pin`
//! against a literal from a source (MIT 18.100B lecture 20's Example 1, the
//! brief's checked arithmetic), and `./x epsilon_delta facts-check` against
//! `oracle.py`, which computes the same keys independently (Python's
//! fractions and decimal modules, binomial sums, no shared code).
//!
//! Chapter 01's keys start `C1.`; the continuation's constructive deltas
//! (brief §3–4: Sprague's note `S.`, Davis's `D1.`/`D2.`, the naive guess
//! that fails `N.`) are computed and checked now, before their chapters.

use explainer_kit::Facts;

use crate::model::{
    HALF, POWERS, Ratio, change_at, davis_delta, decimal_power, first_within, half_height_rounded, limit, local_bound,
    recenter, round_half_up, sprague_delta_rounded,
};

/// The point chapter 01 pins, a/10^k: nine tenths.
pub const FIXED: (u32, u32) = (9, 1);
/// The fixed point's heights are shown exactly up to this power, and above
/// it rounded to [`PLACES`] places after "≈".
pub const EXACT_UP_TO: u32 = 5;
/// The places every rounded number on screen has.
pub const PLACES: u32 = 4;
/// The fixed points whose waits chapter 01 lists, each a/10^k: 0.9, 0.99,
/// 0.999.
pub const WAITS: [(u32, u32); 3] = [(9, 1), (99, 2), (999, 3)];

/// Where the pinned literals come from.
const MIT: &str = "MIT 18.100B Spring 2025, lecture 20, Example 1 (f = 0 on [0, 1), f(1) = 1)";
const BRIEF: &str = "docs/video-program/briefs/epsilon-delta.md";

/// a/10^k as an exact point of [0, 1].
pub fn decimal_point((a, k): (u32, u32)) -> Result<Ratio, String> {
    let den = 10u32.checked_pow(k).ok_or_else(|| format!("10^{k} is too large"))?;
    Ratio::point(a, den)
}

/// The key suffix of a decimal point: "0.9" → "x0.9".
fn point_key((a, k): (u32, u32)) -> Result<String, String> {
    Ok(format!("x{}", decimal_power(a, k, 1)?))
}

fn list<T: std::fmt::Display>(items: impl IntoIterator<Item = T>) -> String {
    items.into_iter().map(|t| t.to_string()).collect::<Vec<_>>().join(",")
}

/// num/den in decimal, exactly, when den divides a power of ten ("0.02").
fn terminating(num: u128, den: u128) -> Result<String, String> {
    for k in 0..=18u32 {
        let scale = 10u128.pow(k);
        if scale % den == 0 {
            let scaled = num * (scale / den);
            let (int, frac) = (scaled / scale, scaled % scale);
            return Ok(if k == 0 {
                int.to_string()
            } else {
                format!("{int}.{frac:0width$}", width = k as usize)
            });
        }
    }
    Err(format!("{num}/{den} does not terminate in 18 places"))
}

pub fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    opening(&mut f)?;
    continuation(&mut f)?;
    Ok(f)
}

/// Chapter 01: the curves, the fixed point, the endpoint, the limit, the
/// moving point and the waits.
fn opening(f: &mut Facts) -> Result<(), String> {
    f.add("C1.domain.left", 0)?;
    f.add("C1.domain.right", 1)?;
    f.add("C1.powers", list(POWERS))?;
    f.add("C1.curves", POWERS.len())?;
    f.add("C1.squared", POWERS[1])?;
    f.add("C1.max_power", POWERS[POWERS.len() - 1])?;

    let (a, k) = FIXED;
    f.add("C1.fixed.x", decimal_power(a, k, 1)?)?;
    for n in POWERS {
        let exact = decimal_power(a, k, n)?;
        f.add(
            &format!("C1.fixed.n{n}.{PLACES}dp"),
            round_half_up(&exact, PLACES as usize)?,
        )?;
        f.add(&format!("C1.fixed.n{n}"), exact)?;
    }
    f.add("C1.fixed.exact_up_to", EXACT_UP_TO)?;

    f.add("C1.endpoint.height", decimal_power(1, 0, POWERS[POWERS.len() - 1])?)?;
    f.pin("C1.limit.below_one", limit(decimal_point(FIXED)?.f64())?, "0", MIT)?;
    f.pin("C1.limit.at_one", limit(1.0)?, "1", MIT)?;

    f.add("C1.half", HALF)?;
    for n in POWERS {
        f.add(&format!("C1.witness.n{n}.{PLACES}dp"), half_height_rounded(n, PLACES)?)?;
    }
    // the n-th curve is at x_n^n = 1/2 and the limit at 0 (x_n < 1)
    f.add("C1.witness.gap", HALF)?;

    f.add("C1.wait.tolerance", HALF)?;
    let pins = ["7", "69", "693"];
    for (&point, want) in WAITS.iter().zip(pins) {
        f.pin(
            &format!("C1.wait.{}", point_key(point)?),
            first_within(decimal_point(point)?, HALF)?,
            want,
            "ln 2 / −ln x rounded up: 6.58, 68.97, 692.8 (the oracle counts exactly)",
        )?;
    }
    Ok(())
}

/// The continuation (brief §3–4): Sprague's δ, the simpler δ, Davis's
/// bounds, and the naive guess that fails.
fn continuation(f: &mut Facts) -> Result<(), String> {
    let eps = Ratio::tolerance(1, 10)?;
    f.add("S.eps", eps)?;
    f.pin(
        "S.delta_sprague.10dp",
        sprague_delta_rounded(eps, 10)?,
        "0.0248456731",
        BRIEF,
    )?;
    let square = recenter(&[0, 0, 1], 2)?;
    f.add("S.square_at_2", list(&square))?;
    let (sn, sd) = davis_delta(local_bound(&square), eps);
    f.add("S.delta_simple", format!("{sn}/{sd}"))?;
    f.pin("S.delta_simple.decimal", terminating(sn, sd)?, "0.02", BRIEF)?;

    let cubic = recenter(&[-6, 0, 5, 1], 1)?;
    f.add("D1.shifted", list(&cubic))?;
    let c1 = local_bound(&cubic);
    f.pin("D1.bound", c1, "22", BRIEF)?;
    let (dn, dd) = davis_delta(c1, eps);
    f.add("D1.delta", format!("{dn}/{dd}"))?;

    let quartic = recenter(&[2, -8, 0, 0, -3], -2)?;
    f.pin("D2.value_at_a", quartic[0], "-30", BRIEF)?;
    f.pin("D2.shifted", list(&quartic[1..]), "88,-72,24,-3", BRIEF)?;
    f.pin("D2.bound", local_bound(&quartic), "187", BRIEF)?;

    // the slope alone: δ = ε/c₁ at ε = 1 for x² at 2, and the hundredth just
    // inside it, where x² moves by more than ε
    let slope = u32::try_from(square[1]).map_err(|_| "the slope of x² at 2 is not a small whole number")?;
    let naive = Ratio::new(1, slope)?;
    f.add("N.naive.delta", naive)?;
    let hundredths = 100 / naive.den - 1;
    let g = gcd(hundredths, 100);
    let (hn, hd) = (hundredths / g, 100 / g);
    f.add("N.naive.h", format!("{hn}/{hd}"))?;
    let (cn, cd) = change_at(&square, i128::from(hn), i128::from(hd))?;
    if cn <= cd {
        return Err(format!("the naive δ's witness moves x² by {cn}/{cd}, not more than 1"));
    }
    f.add("N.naive.change", format!("{cn}/{cd}"))?;
    Ok(())
}

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 { a } else { gcd(b, a % b) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pin_holds() {
        let f = facts().unwrap();
        f.self_check().unwrap();
        assert!(f.len() > 30);
    }

    /// The numbers chapter 01 puts on screen, as they will read.
    #[test]
    fn the_opening_numbers_read_as_shown() {
        let f = facts().unwrap();
        let get = |k: &str| f.get(k).unwrap_or_else(|| panic!("no fact {k}"));
        assert_eq!(get("C1.powers"), "1,2,3,4,5,10,20,50");
        assert_eq!(get("C1.fixed.x"), "0.9");
        assert_eq!(get("C1.fixed.n3"), "0.729");
        assert_eq!(get("C1.fixed.n5"), "0.59049");
        assert_eq!(get("C1.fixed.n10.4dp"), "0.3487");
        assert_eq!(get("C1.fixed.n50.4dp"), "0.0052");
        assert_eq!(get("C1.endpoint.height"), "1");
        assert_eq!(get("C1.witness.n1.4dp"), "0.5000");
        assert_eq!(get("C1.witness.n50.4dp"), "0.9862");
        assert_eq!(get("C1.witness.gap"), "1/2");
        assert_eq!(get("C1.wait.x0.999"), "693");
        assert_eq!(get("N.naive.h"), "6/25");
        assert_eq!(get("N.naive.change"), "636/625");
        assert_eq!(get("D1.delta"), "1/220");
    }

    /// The witnesses climb toward 1 and stay below it; the fixed point's
    /// heights fall; the waits grow as the point nears 1.
    #[test]
    fn the_shown_sequences_move_the_right_way() {
        let f = facts().unwrap();
        let num = |k: String| -> f64 { f.get(&k).unwrap().parse().unwrap() };
        let witness: Vec<f64> = POWERS.iter().map(|n| num(format!("C1.witness.n{n}.4dp"))).collect();
        assert!(witness.windows(2).all(|w| w[1] > w[0]) && witness.iter().all(|&x| x < 1.0));
        let heights: Vec<f64> = POWERS.iter().map(|n| num(format!("C1.fixed.n{n}"))).collect();
        assert!(heights.windows(2).all(|w| w[1] < w[0]));
        let waits: Vec<f64> = WAITS
            .iter()
            .map(|&p| num(format!("C1.wait.{}", point_key(p).unwrap())))
            .collect();
        assert!(waits.windows(2).all(|w| w[1] > w[0]));
    }

    #[test]
    fn terminating_decimals() {
        assert_eq!(terminating(1, 50).unwrap(), "0.02");
        assert_eq!(terminating(3, 4).unwrap(), "0.75");
        assert_eq!(terminating(5, 1).unwrap(), "5");
        assert!(terminating(1, 3).is_err());
    }
}
