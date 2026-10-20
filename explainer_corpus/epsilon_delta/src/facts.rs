//! Every number "Epsilon and Delta: Why Getting Closer Wasn't Enough" shows
//! or says, computed here from `model.rs` and checked two ways: `pin`
//! against a literal from a source (MIT 18.100B lecture 20's Example 1, the
//! brief's checked arithmetic), and `./x epsilon_delta facts-check` against
//! `oracle.py`, which computes the same keys independently (Python's
//! fractions and decimal modules, binomial sums, no shared code).
//!
//! Chapter 01's keys start `C1.`, chapter 02's `C2.`, chapter 03's `C3.`;
//! the continuation's constructive deltas (brief §3–4: Sprague's note `S.`,
//! Davis's `D1.`/`D2.`, the naive guess that fails `N.`) are computed and
//! checked now, before their chapters.

use std::f64::consts::LN_2;

use explainer_kit::Facts;
use fmn::core::constants::FRAME_WIDTH;

use crate::chapters::draw::{LENS_H, LENS_HEIGHTS, LENS_W, PLOT};
use crate::model::{
    BASE, CAP, CHALLENGE, FILLS, Frac, HALF, LUCKY, MORE_POWERS, NUDGES, PICKS, POWERS, RECENTRE_AT, Ratio, SEARCH_EPS,
    SPRAGUE_EPS, SQUARE, SQUARE_POLY, TOLERANCES, WIDE, ZOOMS, above_half, blind_window, change_at, chord_formula,
    chord_limit, chord_slope, davis_delta, decimal_power, factor_bound, fill_failure, first_within, found_window, gap,
    half_height_rounded, limit, local_bound, reach, recenter, round_half_up, slope_window, slope_window_miss,
    sprague_delta_rounded, square_change, widest_window_rounded, width_rounded, window, window_failure,
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

/// The passage chapter 03 cites, as read (2026-10-19) in D. R. Wilkins's
/// 2002 edition of Berkeley's The Analyst (the brief's link): its note on
/// the text (page iii) dates the first editions 1734; §§XIII–XIV run over
/// its printed pages 5 and 6, and the sentence the chapter quotes is on page
/// 6. Restated here and in the oracle, not computed.
const ANALYST_YEAR: u32 = 1734;
const ANALYST_EDITION: u32 = 2002;
const ANALYST_PAGES: (u32, u32) = (5, 6);
const QUOTE_PAGE: u32 = 6;

/// The note chapter 06 cites, A. H. Sprague, "A Note on δ and ε", as read
/// (2026-10-19) in two places. The owner's scan of its page (brief, owner
/// intake) gives the title, the author, the expansion x² − 4 = (x − 2)² +
/// 4(x − 2), the bound, completing the square and δ = √(ε + 4) − 2, beside
/// the route it sets aside (x between 1 and 3, so |x + 2| < 5). Crossref's
/// record for DOI 10.2307/2308663 gives The American Mathematical Monthly 67
/// (8), October 1960, page 780. Restated here and in the oracle, not
/// computed.
const NOTE_VOLUME: u32 = 67;
const NOTE_YEAR: u32 = 1960;
const NOTE_PAGE: u32 = 780;

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

/// The width in pixels of the final picture: 3840 by 2160 (the corpus's
/// final-render gate). Chapter 02 counts how many of them the plot spans.
pub const FINAL_PIXELS_ACROSS: u32 = 3840;

pub fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    opening(&mut f)?;
    strip(&mut f)?;
    objection(&mut f)?;
    game(&mut f)?;
    search(&mut f)?;
    continuation(&mut f)?;
    recentring(&mut f)?;
    polynomial_bound(&mut f)?;
    let parts = crate::chapters::error_bridge::TERMS.len();
    f.add("C9.pieces", parts)?;
    f.add("C9.share", Ratio::new(1, parts as u32)?)?;
    f.add("C8.second_x", decimal_power(WAITS[1].0, WAITS[1].1, 1)?)?;
    for n in crate::chapters::sequences::STAGES {
        f.add(&format!("C10.n{n}.x"), crate::chapters::sequences::approaching(n)?)?;
    }
    for n in [2, 4, 8] {
        let (x, y, r) = crate::chapters::sequences::root_sample(n)?;
        f.add(&format!("C10.root{n}.x"), x)?;
        f.add(&format!("C10.root{n}.y"), y)?;
        f.add(&format!("C10.root{n}.ratio"), r)?;
    }
    let roots = crate::chapters::preimages::ROOTS;
    let bounds = crate::chapters::preimages::output_bounds();
    f.add("C11.lower", bounds[0])?;
    f.add("C11.upper", bounds[1])?;
    f.add("C11.parts", roots.len())?;
    f.add(
        "C11.roots",
        format!("{},{},{},{}", -roots[1], -roots[0], roots[0], roots[1]),
    )?;
    use crate::chapters::completeness::{BISECTION_FACTOR, ENDS, RADICAND, SHOWN_STEPS, bracket, value};
    f.add("C12.radicand", RADICAND)?;
    f.add("C12.left", ENDS[0])?;
    f.add("C12.right", ENDS[1])?;
    f.add("C12.sign.left", value(Frac::whole(ENDS[0]))?)?;
    f.add("C12.sign.right", value(Frac::whole(ENDS[1]))?)?;
    f.add("C12.bound", 2 * ENDS[1])?;
    f.add("C12.bisection", BISECTION_FACTOR)?;
    for n in SHOWN_STEPS {
        let (lo, hi) = bracket(n)?;
        f.add(&format!("C12.bracket{n}.lo"), lo)?;
        f.add(&format!("C12.bracket{n}.hi"), hi)?;
        f.add(&format!("C12.bracket{n}.width"), hi.minus(lo)?)?;
    }
    Ok(f)
}

/// Chapter 07 uses Davis's first example, read in the owner's scan
/// IMG_1440.PNG (SHA a81690f9...), A Further Note on δ and ε, Monthly 68
/// (1961), pp. 567–568. Its polynomial is x³ + 5x², with value 6 at 1.
/// The existing D1 keys describe its change after subtracting that value.
/// Bibliographic values are transcribed; all mathematical values computed.
fn polynomial_bound(f: &mut Facts) -> Result<(), String> {
    let polynomial = [0, 0, 5, 1];
    let at = 1;
    let shifted = recenter(&polynomial, at)?;
    f.add("C7.a", at)?;
    f.add("C7.value", shifted[0])?;
    f.add("C7.degree", polynomial.len() - 1)?;
    f.add("C7.quadratic", polynomial[2])?;
    f.add("C7.linear", shifted[1])?;
    f.add("C7.square", shifted[2])?;
    f.add("C7.cube", shifted[3])?;
    f.add("C7.power2", 2)?;
    f.add("C7.note.volume", 68)?;
    f.add("C7.note.year", 1961)?;
    f.add("C7.note.first", 567)?;
    f.add("C7.note.last", 568)?;
    Ok(())
}

/// A fraction strictly between 0 and 1, rounded half up to `places` places
/// exactly: floor(x·10^p + 1/2) in integers ("0.067" for 1/15).
fn rounded_below_one(x: Frac, places: u32) -> Result<String, String> {
    let scale = 10i128.pow(places);
    let (num, den) = (i128::from(x.num), i128::from(x.den));
    if num <= 0 || num >= den {
        return Err(format!("{x} is not strictly between 0 and 1"));
    }
    let d = (2 * num * scale + den) / (2 * den);
    if d >= scale {
        return Err(format!("{x} rounds up to 1 at {places} places"));
    }
    Ok(format!("0.{d:0width$}", width = places as usize))
}

/// Chapter 05: the square at x = 1/2 on [0, 1]; the bound on the factor
/// x + 1/2 the search uses, the factor near 1/2, the ratio the bound gives,
/// the window found at the tolerance compared, and the widest window there,
/// measured.
fn search(f: &mut Facts) -> Result<(), String> {
    let a = BASE;
    f.add("C5.a", a)?;
    f.add("C5.value", a.times(a)?)?;
    let bound = factor_bound(a)?;
    f.add("C5.bound", bound)?;
    f.add("C5.near", a.times(Frac::whole(2))?)?;
    let ratio = Frac::whole(1).over(bound)?;
    f.add("C5.ratio", ratio)?;
    f.add("C5.eps", SEARCH_EPS)?;
    let delta = found_window(a, SEARCH_EPS)?;
    if delta != ratio.times(SEARCH_EPS)? {
        return Err(format!("the window found, {delta}, is not {ratio} of {SEARCH_EPS}"));
    }
    f.add("C5.delta", delta)?;
    f.add("C5.delta.3dp", rounded_below_one(delta, PLACES - 1)?)?;
    // the widest window, measured; the window found lies strictly inside it,
    // below even the widest's certified lower end (D − 1/2)/10^p
    let widest = widest_window_rounded(a, SEARCH_EPS, PLACES - 1)?;
    let digits: i64 = widest[2..]
        .parse()
        .map_err(|_| format!("the widest window prints as {widest}"))?;
    let floor = Frac::whole(2 * digits - 1).over(Frac::whole(2 * 10i64.pow(PLACES - 1)))?;
    if delta >= floor {
        return Err(format!(
            "the window found, {delta}, is not narrower than the widest, {widest}"
        ));
    }
    f.add("C5.widest.3dp", widest)?;
    Ok(())
}

/// A fraction of either sign as its exact decimal ("-0.08", "1.5").
fn signed_decimal(x: Frac) -> Result<String, String> {
    if x < Frac::whole(0) {
        Ok(format!("-{}", decimal(x.negated()?)?))
    } else {
        decimal(x)
    }
}

/// Chapter 04: the honest round (tolerance, window, picks and their
/// slopes), the wide window and the pick that beats it, the two fills of
/// the hole and the pick that beats every window for the wrong one, the
/// window a defender who saw the pick could answer with, and continuity of
/// x² at the end of its domain against one half.
fn game(f: &mut Facts) -> Result<(), String> {
    let target = chord_limit(BASE)?;
    f.add("C4.eps", CHALLENGE)?;
    let delta = window(CHALLENGE)?;
    f.add("C4.delta", delta)?;
    // every pick is in the honest window, so it must land in the band
    for (i, &h) in PICKS.iter().enumerate() {
        let s = chord_slope(BASE, h)?;
        if h.is_zero() || !h.within(Frac::whole(0), delta)? || !s.within(target, CHALLENGE)? {
            return Err(format!("the pick {h} is not an honest pick in the band"));
        }
        f.add(&format!("C4.pick.{i}"), signed_decimal(h)?)?;
        f.add(&format!("C4.pick_slope.{i}"), decimal(s)?)?;
    }
    // the wide window: a lucky pick lands, and the midpoint pick does not
    f.add("C4.wide", WIDE)?;
    let lucky = chord_slope(BASE, LUCKY)?;
    if !LUCKY.within(Frac::whole(0), WIDE)? || !lucky.within(target, CHALLENGE)? {
        return Err(format!("{LUCKY} is not a lucky pick for the window {WIDE}"));
    }
    f.add("C4.lucky", decimal(LUCKY)?)?;
    f.add("C4.lucky_slope", decimal(lucky)?)?;
    let catch = window_failure(BASE, CHALLENGE, WIDE)?.ok_or("the wide window must lose")?;
    f.add("C4.catch", decimal(catch)?)?;
    f.add("C4.catch_slope", decimal(chord_slope(BASE, catch)?)?)?;
    // the fills: the limit is continuous with the honest window, the other
    // loses that window (and every other) at its witness pick
    let [good, bad] = FILLS;
    if fill_failure(BASE, good, CHALLENGE, delta)?.is_some() {
        return Err(format!(
            "the hole filled with {good} must be continuous for the window {delta}"
        ));
    }
    f.add("C4.fill.good", good)?;
    f.add("C4.fill.bad", decimal(bad)?)?;
    let witness = fill_failure(BASE, bad, CHALLENGE, delta)?.ok_or("the wrong fill must lose")?;
    f.add("C4.fill.witness", decimal(witness)?)?;
    f.add("C4.fill.witness_value", decimal(chord_slope(BASE, witness)?)?)?;
    // the argument for every window, as the screen states it: the pick
    // min(δ, room)/2 with room = |L − fill| − ε (inside the domain's reach
    // here) leaves g at least gap = |L − fill| − room/2 from the fill
    let apart = bad.minus(target)?;
    let apart = if apart < Frac::whole(0) {
        apart.negated()?
    } else {
        apart
    };
    let room = apart.minus(CHALLENGE)?;
    if room >= reach(BASE)? {
        return Err(format!("the room {room} must stay inside the domain's reach"));
    }
    let gap = apart.minus(room.over(Frac::whole(2))?)?;
    if gap <= CHALLENGE {
        return Err(format!("the gap {gap} must be more than the tolerance"));
    }
    f.add("C4.fill.room", room)?;
    f.add("C4.fill.gap", gap)?;
    // a defender who saw that pick first could hide it outside the window
    f.add("C4.cheat.delta", blind_window(witness)?)?;
    // continuity of x² at x = 1, the end of [0, 1], against one half: the
    // window runs from 2^(−1/2), where x² is exactly one half, up to 1
    let end = Ratio::point(1, 1)?;
    f.add("C4.end.x", end.num)?;
    f.add("C4.end.power", SQUARE)?;
    f.add("C4.end.eps", HALF)?;
    let half = Frac::whole(i64::from(HALF.num)).over(Frac::whole(i64::from(HALF.den)))?;
    let level = Frac::whole(1).minus(half)?;
    f.add("C4.end.level", level)?;
    if !above_half(end, SQUARE) {
        return Err("x = 1 must keep x² within one half of 1".to_owned());
    }
    f.add("C4.end.edge.4dp", half_height_rounded(SQUARE, PLACES)?)?;
    f.add("C4.end.delta.4dp", width_rounded(SQUARE, 1, PLACES)?)?;
    Ok(())
}

/// A positive fraction's exact decimal ("1.25").
fn decimal(x: Frac) -> Result<String, String> {
    let num = u128::try_from(x.num).map_err(|_| format!("{x} is negative"))?;
    terminating(num, u128::from(x.den.unsigned_abs()))
}

/// A polynomial in h from its coefficients, lowest power first, as the
/// screen writes it: "1 + h", "h + h^2". A zero coefficient is left out;
/// only whole positive ones are written.
fn in_h(coeffs: &[Frac]) -> Result<String, String> {
    let mut terms = Vec::new();
    for (k, &c) in coeffs.iter().enumerate() {
        if c.is_zero() {
            continue;
        }
        if c.den != 1 || c.num < 0 {
            return Err(format!("the coefficient {c} of h^{k} is not a whole positive number"));
        }
        let power = match k {
            0 => String::new(),
            1 => "h".to_owned(),
            _ => format!("h^{k}"),
        };
        terms.push(match (k, c.num) {
            (0, n) => n.to_string(),
            (_, 1) => power,
            (_, n) => format!("{n}{power}"),
        });
    }
    Ok(if terms.is_empty() {
        "0".to_owned()
    } else {
        terms.join(" + ")
    })
}

/// Chapter 03: the passage it cites; the square at x = 1/2, its change and
/// its chord's slope for nonzero h, the slope at each increment drawn, the
/// 0/0 at h = 0, where the slopes go, and the window that keeps them within
/// each tolerance drawn.
fn objection(f: &mut Facts) -> Result<(), String> {
    f.add("C3.year", ANALYST_YEAR)?;
    f.add("C3.edition", ANALYST_EDITION)?;
    f.add("C3.page.first", ANALYST_PAGES.0)?;
    f.add("C3.page.last", ANALYST_PAGES.1)?;
    f.add("C3.quote.page", QUOTE_PAGE)?;

    f.add("C3.power", SQUARE)?;
    f.add("C3.base", BASE)?;
    f.add("C3.base.height", BASE.times(BASE)?)?;
    let (change, slope) = chord_formula(BASE)?;
    f.add("C3.change", in_h(&change)?)?;
    f.add("C3.quotient", in_h(&slope)?)?;
    let target = chord_limit(BASE)?;
    f.add("C3.limit", target)?;
    // h goes to 0 and is never 0: there the change is 0 too, the quotient
    // would be 0/0, and the model refuses it
    let zero = Frac::whole(0);
    f.add("C3.h.to", zero)?;
    if chord_slope(BASE, zero).is_ok() {
        return Err("the chord's slope at h = 0 must be refused".to_owned());
    }
    f.add("C3.at_zero", format!("{}/{zero}", square_change(BASE, zero)?))?;
    for (i, &h) in NUDGES.iter().enumerate() {
        let s = chord_slope(BASE, h)?;
        if s != slope[0].plus(h.times(slope[1])?)? {
            return Err(format!("at h = {h} the slope {s} is not {}", in_h(&slope)?));
        }
        f.add(&format!("C3.h.{i}"), h)?;
        f.add(&format!("C3.slope.{i}"), decimal(s)?)?;
    }
    // each window works and is no wider than it must be: halfway out the
    // slope is within the tolerance, at the window's edge it is not
    for (i, &t) in TOLERANCES.iter().enumerate() {
        let w = window(t)?;
        let half = w.over(Frac::whole(2))?;
        for (h, inside) in [(half, true), (half.negated()?, true), (w, false), (w.negated()?, false)] {
            if chord_slope(BASE, h)?.within(target, t)? != inside {
                return Err(format!("tolerance {t}: the window {w} misjudges h = {h}"));
            }
        }
        f.add(&format!("C3.eps.{i}"), t)?;
        f.add(&format!("C3.delta.{i}"), w)?;
    }
    Ok(())
}

/// Chapter 02: the strip's widths as the power grows, the zoom windows and
/// the share of a 1/n-wide window the strip fills (toward ln 2), and the
/// pixels the plot spans in the final picture.
fn strip(f: &mut Facts) -> Result<(), String> {
    f.add("C2.powers", list(MORE_POWERS))?;
    let from = POWERS[POWERS.len() - 1];
    for n in std::iter::once(from).chain(MORE_POWERS) {
        f.add(&format!("C2.width.n{n}.6dp"), width_rounded(n, 1, 6)?)?;
    }
    // at x = 1 the curve and the limit are both 1
    f.add("C2.gap_at_one", gap(1.0, from)?)?;
    for (i, &m) in ZOOMS.iter().enumerate() {
        f.add(&format!("C2.zoom.{i}"), m)?;
        let k = m.ilog10();
        if 10u32.pow(k) != m {
            return Err(format!("zoom {m} is not a power of ten"));
        }
        f.add(&format!("C2.window.x{m}"), decimal_power(m - 1, k, 1)?)?;
        f.add(&format!("C2.share.n{m}.3dp"), width_rounded(m, m, 3)?)?;
    }
    let step = ZOOMS[1] / ZOOMS[0];
    if ZOOMS.windows(2).any(|w| w[1] != w[0] * step) {
        return Err("the zooms do not grow by one factor".to_owned());
    }
    f.add("C2.zoom.step", step)?;
    // the window's labels against what it draws: [1 − 1/m, 1] across the
    // window's width is m times its width on the plot, and its heights are
    // drawn at LENS_HEIGHTS of the plot's
    for &m in &ZOOMS {
        let stretch = LENS_W / (PLOT.frame.width / f64::from(m));
        if (stretch - f64::from(m)).abs() > 1e-9 {
            return Err(format!("the window labelled ×{m} stretches ×{stretch}"));
        }
    }
    if (LENS_H / PLOT.frame.height - LENS_HEIGHTS.f64()).abs() > 1e-12 {
        return Err(format!(
            "the window labelled heights ×{LENS_HEIGHTS} draws them at {}",
            LENS_H / PLOT.frame.height
        ));
    }
    f.add("C2.lens.heights", LENS_HEIGHTS)?;
    // 02_7 starts the paired run over at n = m, the first stretch
    f.add("C2.restart_power", ZOOMS[0])?;
    f.pin("C2.ln2.4dp", format!("{:.4}", LN_2), "0.6931", "ln 2 = 0.693147…")?;
    f.add("C2.ln2.2dp", format!("{:.2}", LN_2))?;
    let across = PLOT.frame.width / FRAME_WIDTH * f64::from(FINAL_PIXELS_ACROSS);
    if (across - across.round()).abs() > 1e-6 {
        return Err(format!("the plot spans {across} pixels, not a whole number"));
    }
    let px = across.round() as u32;
    f.pin(
        "C2.pixels.plot_4k",
        px,
        "1296",
        "4.8 of the frame's 8 × 16/9 = 128/9 units, across 3840 pixels",
    )?;
    // narrower than one pixel: 1 − x_n < 1/px, that is x_n > (px − 1)/px,
    // that is ((px − 1)/px)^n < 1/2
    f.pin(
        "C2.subpixel_from",
        first_within(Ratio::point(px - 1, px)?, HALF)?,
        "898",
        "ln 2 / −ln(1295/1296) = 897.97, rounded up",
    )?;
    Ok(())
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
    let eps = Ratio::tolerance(SPRAGUE_EPS.num, SPRAGUE_EPS.den)?;
    f.add("S.eps", eps)?;
    f.pin(
        "S.delta_sprague.10dp",
        sprague_delta_rounded(eps, 10)?,
        "0.0248456731",
        BRIEF,
    )?;
    let square = recenter(&SQUARE_POLY, RECENTRE_AT)?;
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

/// Chapter 06: x² on all real numbers, recentred at a = 2 (brief §3,
/// Sprague's note). Its value and slope there, the square it completes, the
/// window the slope alone suggests and the first four-place input where that
/// window loses, Sprague's δ at four places, and the usual cap on h with the
/// bound it gives the factor x + 2. The tolerance is the continuation's.
fn recentring(f: &mut Facts) -> Result<(), String> {
    let shifted = recenter(&SQUARE_POLY, RECENTRE_AT)?;
    let [value, slope, lead] = shifted[..] else {
        return Err(format!("x² recentred at {RECENTRE_AT} has coefficients {shifted:?}"));
    };
    // (|h| + slope/2)² = |h|² + slope·|h| + (slope/2)² needs a monic square
    // and an even slope
    if lead != 1 || slope % 2 != 0 {
        return Err(format!(
            "{lead}h² + {slope}h does not complete to a square of |h| + a whole number"
        ));
    }
    f.add("C6.a", RECENTRE_AT)?;
    // h = x − a at the point itself
    let a = Frac::whole(RECENTRE_AT);
    f.add("C6.h.at_a", a.minus(a)?)?;
    f.add("C6.value", value)?;
    f.add("C6.slope", slope)?;
    let half = slope / 2;
    f.add("C6.half_slope", half)?;
    f.add("C6.added", half * half)?;

    let tolerance = Ratio::tolerance(SPRAGUE_EPS.num, SPRAGUE_EPS.den)?;
    let eps = Frac::whole(i64::from(tolerance.num)).over(Frac::whole(i64::from(tolerance.den)))?;
    let naive = slope_window(RECENTRE_AT, eps)?;
    if naive != eps.over(Frac::whole(i64::try_from(slope).map_err(|_| "the slope is too large")?))? {
        return Err(format!("the slope-only window {naive} is not ε over the slope {slope}"));
    }
    f.add("C6.naive.delta", naive)?;
    let (h, change) = slope_window_miss(RECENTRE_AT, eps, PLACES)?;
    let shown = rounded_below_one(change, PLACES + 1)?;
    // the change as shown still reads past ε: 0.10022 > 0.1
    let digits: i64 = shown[2..]
        .parse()
        .map_err(|_| format!("the change prints as {shown}"))?;
    let scale = 10i64.pow(PLACES + 1);
    if Frac::whole(digits).over(Frac::whole(scale))? <= eps {
        return Err(format!("the change {change} prints as {shown}, not past ε = {eps}"));
    }
    f.add("C6.naive.h", rounded_below_one(h, PLACES)?)?;
    f.add("C6.naive.change.5dp", shown)?;
    let delta = sprague_delta_rounded(tolerance, PLACES)?;
    // the loss lies past Sprague's δ: above the rounding's certified upper
    // end (D + ½)/10^p
    let d: i64 = delta[2..]
        .parse()
        .map_err(|_| format!("Sprague's δ prints as {delta}"))?;
    if h <= Frac::whole(2 * d + 1).over(Frac::whole(2 * 10i64.pow(PLACES)))? {
        return Err(format!("the loss at {h} is not past Sprague's δ, {delta}"));
    }
    f.add("C6.delta.4dp", delta)?;

    f.add("C6.note.volume", NOTE_VOLUME)?;
    f.add("C6.note.year", NOTE_YEAR)?;
    f.add("C6.note.page", NOTE_PAGE)?;

    f.add("C6.cap", CAP)?;
    let factor = 2 * RECENTRE_AT + CAP;
    f.add("C6.cap.factor", factor)?;
    // the cap's δ, min(1, ε/5), is the continuation's simple δ
    let capped = eps.over(Frac::whole(factor))?.min(Frac::whole(CAP));
    let (sn, sd) = davis_delta(local_bound(&shifted), tolerance);
    let whole = |n: u128| {
        i64::try_from(n)
            .map(Frac::whole)
            .map_err(|_| format!("{n} is too large"))
    };
    if capped != whole(sn)?.over(whole(sd)?)? {
        return Err(format!("the capped δ {capped} is not the simple δ {sn}/{sd}"));
    }
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

    /// The numbers chapter 02 puts on screen or says, as they will read;
    /// the widths fall and the shares rise toward ln 2 without reaching it.
    #[test]
    fn the_strip_numbers_read_as_shown() {
        let f = facts().unwrap();
        let get = |k: &str| f.get(k).unwrap_or_else(|| panic!("no fact {k}"));
        assert_eq!(get("C2.powers"), "100,200,500,1000");
        assert_eq!(get("C2.width.n50.6dp"), "0.013767");
        assert_eq!(get("C2.width.n1000.6dp"), "0.000693");
        assert_eq!(get("C2.gap_at_one"), "0");
        assert_eq!(get("C2.window.x1000"), "0.999");
        assert_eq!(get("C2.zoom.step"), "10");
        assert_eq!(get("C2.ln2.2dp"), "0.69");
        assert_eq!(get("C2.pixels.plot_4k"), "1296");
        assert_eq!(get("C2.subpixel_from"), "898");
        assert_eq!(get("C2.lens.heights"), "1/3");
        assert_eq!(get("C2.restart_power"), "10");
        let num = |k: String| -> f64 { f.get(&k).unwrap().parse().unwrap() };
        let widths: Vec<f64> = std::iter::once(50)
            .chain(MORE_POWERS)
            .map(|n| num(format!("C2.width.n{n}.6dp")))
            .collect();
        assert!(widths.windows(2).all(|w| w[1] < w[0]) && widths.iter().all(|&w| w > 0.0));
        let shares: Vec<f64> = ZOOMS.iter().map(|m| num(format!("C2.share.n{m}.3dp"))).collect();
        assert!(shares.windows(2).all(|s| s[1] >= s[0]) && shares.iter().all(|&s| s < LN_2));
    }

    /// The numbers chapter 03 puts on screen or says, as they will read; the
    /// slopes close in on 1 from each side, and the zero increment gives 0/0.
    #[test]
    fn the_objection_numbers_read_as_shown() {
        let f = facts().unwrap();
        let get = |k: &str| f.get(k).unwrap_or_else(|| panic!("no fact {k}"));
        assert_eq!(get("C3.year"), "1734");
        assert_eq!(get("C3.page.first"), "5");
        assert_eq!(get("C3.quote.page"), "6");
        assert_eq!(get("C3.power"), "2");
        assert_eq!(get("C3.base"), "1/2");
        assert_eq!(get("C3.base.height"), "1/4");
        assert_eq!(get("C3.change"), "h + h^2");
        assert_eq!(get("C3.quotient"), "1 + h");
        assert_eq!(get("C3.limit"), "1");
        assert_eq!(get("C3.h.to"), "0");
        assert_eq!(get("C3.at_zero"), "0/0");
        assert_eq!(get("C3.h.4"), "-1/4");
        assert_eq!(get("C3.eps.1"), "1/20");
        assert_eq!(get("C3.delta.0"), "1/10");
        let slopes: Vec<&str> = (0..NUDGES.len()).map(|i| get(&format!("C3.slope.{i}"))).collect();
        assert_eq!(slopes, ["1.5", "1.25", "1.1", "0.5", "0.75", "0.9"]);
        let num = |s: &str| -> f64 { s.parse().unwrap() };
        let (above, below) = slopes.split_at(3);
        assert!(above.windows(2).all(|w| num(w[1]) < num(w[0])) && above.iter().all(|&s| num(s) > 1.0));
        assert!(below.windows(2).all(|w| num(w[1]) > num(w[0])) && below.iter().all(|&s| num(s) < 1.0));
    }

    /// The numbers chapter 04 puts on screen or says, as they will read.
    #[test]
    fn the_game_numbers_read_as_shown() {
        let f = facts().unwrap();
        let get = |k: &str| f.get(k).unwrap_or_else(|| panic!("no fact {k}"));
        assert_eq!(
            (get("C4.eps"), get("C4.delta"), get("C4.wide")),
            ("1/10", "1/10", "1/5")
        );
        let picks: Vec<&str> = (0..PICKS.len()).map(|i| get(&format!("C4.pick.{i}"))).collect();
        let slopes: Vec<&str> = (0..PICKS.len()).map(|i| get(&format!("C4.pick_slope.{i}"))).collect();
        assert_eq!(picks, ["-0.08", "0.04", "0.09"]);
        assert_eq!(slopes, ["0.92", "1.04", "1.09"]);
        assert_eq!((get("C4.lucky"), get("C4.lucky_slope")), ("0.05", "1.05"));
        assert_eq!((get("C4.catch"), get("C4.catch_slope")), ("0.15", "1.15"));
        assert_eq!((get("C4.fill.good"), get("C4.fill.bad")), ("1", "1.5"));
        assert_eq!((get("C4.fill.witness"), get("C4.fill.witness_value")), ("0.05", "1.05"));
        assert_eq!((get("C4.fill.room"), get("C4.fill.gap")), ("2/5", "3/10"));
        assert_eq!(get("C4.cheat.delta"), "1/40");
        assert_eq!(
            (get("C4.end.x"), get("C4.end.power"), get("C4.end.eps")),
            ("1", "2", "1/2")
        );
        assert_eq!(get("C4.end.level"), "1/2");
        assert_eq!((get("C4.end.edge.4dp"), get("C4.end.delta.4dp")), ("0.7071", "0.2929"));
        assert_eq!(signed_decimal(Frac::whole(-3)).unwrap(), "-3");
    }

    /// The numbers chapter 05 puts on screen or says, as they will read.
    #[test]
    fn the_search_numbers_read_as_shown() {
        let f = facts().unwrap();
        let get = |k: &str| f.get(k).unwrap_or_else(|| panic!("no fact {k}"));
        assert_eq!((get("C5.a"), get("C5.value"), get("C5.bound")), ("1/2", "1/4", "3/2"));
        assert_eq!((get("C5.near"), get("C5.ratio"), get("C5.eps")), ("1", "2/3", "1/10"));
        assert_eq!(
            (get("C5.delta"), get("C5.delta.3dp"), get("C5.widest.3dp")),
            ("1/15", "0.067", "0.092")
        );
        assert_eq!(
            rounded_below_one(Frac::whole(1).over(Frac::whole(8)).unwrap(), 2).unwrap(),
            "0.13"
        );
        assert!(rounded_below_one(Frac::whole(1), 3).is_err());
        let near_one = Frac::whole(9999).over(Frac::whole(10000)).unwrap();
        assert!(rounded_below_one(near_one, 3).is_err(), "would print 1.000");
    }

    /// The numbers chapter 06 puts on screen or says, as they will read:
    /// x² at 2 is 4 with slope 4; completing the square adds (4/2)² = 4; the
    /// slope alone gives 1/40, and 0.0249 inside it moves x² by 0.10022;
    /// Sprague's δ at four places and ten; the cap 1 and the factor's bound 5,
    /// whose δ is the continuation's 1/50.
    #[test]
    fn the_recentring_numbers_read_as_shown() {
        let f = facts().unwrap();
        let get = |k: &str| f.get(k).unwrap_or_else(|| panic!("no fact {k}"));
        assert_eq!(
            (get("C6.a"), get("C6.h.at_a"), get("C6.value"), get("C6.slope")),
            ("2", "0", "4", "4")
        );
        assert_eq!((get("C6.half_slope"), get("C6.added")), ("2", "4"));
        assert_eq!(
            (get("C6.naive.delta"), get("C6.naive.h"), get("C6.naive.change.5dp")),
            ("1/40", "0.0249", "0.10022")
        );
        assert_eq!(
            (get("C6.delta.4dp"), get("S.delta_sprague.10dp")),
            ("0.0248", "0.0248456731")
        );
        assert!(
            get("S.delta_sprague.10dp").starts_with(get("C6.delta.4dp")),
            "the four places agree with the ten"
        );
        assert_eq!(
            (get("C6.cap"), get("C6.cap.factor"), get("S.delta_simple")),
            ("1", "5", "1/50")
        );
        assert_eq!(get("S.eps"), "1/10");
        assert_eq!(
            (get("C6.note.volume"), get("C6.note.year"), get("C6.note.page")),
            ("67", "1960", "780")
        );
    }

    #[test]
    fn polynomials_in_h_read_as_written() {
        let w = |n: i64| Frac::whole(n);
        assert_eq!(in_h(&[w(0), w(1), w(1)]).unwrap(), "h + h^2");
        assert_eq!(in_h(&[w(1), w(1)]).unwrap(), "1 + h");
        assert_eq!(in_h(&[w(4), w(0), w(3)]).unwrap(), "4 + 3h^2");
        assert_eq!(in_h(&[w(0)]).unwrap(), "0");
        assert!(in_h(&[w(1), w(-1)]).is_err() && in_h(&[BASE]).is_err());
        assert_eq!(decimal(BASE).unwrap(), "0.5");
        assert!(decimal(w(-1)).is_err());
    }

    #[test]
    fn terminating_decimals() {
        assert_eq!(terminating(1, 50).unwrap(), "0.02");
        assert_eq!(terminating(3, 4).unwrap(), "0.75");
        assert_eq!(terminating(5, 1).unwrap(), "5");
        assert!(terminating(1, 3).is_err());
    }
}
