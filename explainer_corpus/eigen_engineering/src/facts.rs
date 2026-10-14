//! Every number Video K shows or says, computed here and checked two ways:
//! `pin` against a literal from the fact sheet (a source, or arithmetic done
//! by hand), and `./x eigen_engineering facts-check` against `oracle.py`
//! (same keys). The model works each system the way the video does (the
//! stiffness method, cyclic Jacobi, exact fractions); the oracle reaches the
//! same numbers by other routes (joint equilibrium, LAPACK, a second
//! enumeration).

use std::sync::OnceLock;

use explainer_kit::Facts;
use explainer_kit::stats::Data;

use crate::model::economy::{self, Frac, Pair, Table};
use crate::model::linalg::{Mat, dot, jacobi_eigh};
use crate::model::rooms::{self, Room, Section};
use crate::model::truss::{self, Kind, Truss};
use crate::model::{QUIZZES, carts, day_number, factory};

/// Numbers quoted from the sources (fact sheet rows K-B and K-S; the oracle
/// prints the same literals).
pub mod cited {
    /// A date as (year, month, day).
    pub type Date = (i64, i64, i64);

    /// The Tacoma Narrows Bridge. Board of engineers (Ammann, von Kármán,
    /// Woodruff), "The Failure of the Tacoma Narrows Bridge", 1941: opened
    /// 1 July 1940, centre span 2,800 ft, wind clocked at 42 mph. Billah and
    /// Scanlan, Am. J. Phys. 59 (1991), p. 120: the final motion at 12 cycles
    /// a minute, natural vortex shedding at that wind "close to 1 Hz".
    pub const TACOMA_OPENED: Date = (1940, 7, 1);
    pub const TACOMA_FAILED: Date = (1940, 11, 7);
    pub const TACOMA_SPAN_FT: u32 = 2800;
    pub const TACOMA_WIND_MPH: u32 = 42;
    pub const TACOMA_TWIST_PER_MIN: u32 = 12;
    pub const TACOMA_VORTEX_HZ: u32 = 1;
    pub const BILLAH_SCANLAN_YEAR: u32 = 1991;
    pub const TACOMA_BOARD_YEAR: u32 = 1941;

    /// London's Millennium Bridge. Dallard et al., The Structural Engineer
    /// 79(22), 2001: opened 10 June 2000, closed 12 June; 80,000 to 100,000
    /// people on the first day, at most 2,000 on the deck; measured lateral
    /// modes (Table 3) at 0.48, 0.80, 0.95 and 1.03 Hz; walking at about
    /// 2.0 Hz, lateral force at half that; 37 viscous dampers. Arup's own
    /// pages: sway of about 70 mm; a span steady with 156 walkers and not
    /// with 10 more. Reopened 22 February 2002 (Taylor Devices).
    pub const MILLENNIUM_OPENED: Date = (2000, 6, 10);
    pub const MILLENNIUM_CLOSED: Date = (2000, 6, 12);
    pub const MILLENNIUM_REOPENED: Date = (2002, 2, 22);
    pub const MILLENNIUM_FIRST_DAY: (u32, u32) = (80_000, 100_000);
    pub const MILLENNIUM_ON_DECK: u32 = 2000;
    pub const MILLENNIUM_SWAY_MM: u32 = 70;
    /// The lateral modes, in hundredths of a hertz.
    pub const MILLENNIUM_LATERAL_CHZ: [u32; 4] = [48, 80, 95, 103];
    /// Footfalls a second, in tenths of a hertz.
    pub const WALKING_DHZ: u32 = 20;
    pub const MILLENNIUM_TEST_STEADY: u32 = 156;
    pub const MILLENNIUM_TEST_ADDED: u32 = 10;
    pub const MILLENNIUM_VISCOUS_DAMPERS: u32 = 37;
    /// Dallard et al. 2001: "The measured frequencies of the modes which had
    /// been observed to vibrate were within 10% of the values predicted in
    /// the design analysis"; the effect "could occur on any bridge with a
    /// lateral frequency below about 1.3Hz loaded with a sufficient number of
    /// pedestrians"; "4 No. pairs of lateral tuned mass dampers" and "26 pairs
    /// of vertical tuned mass dampers".
    pub const MILLENNIUM_PREDICTED_WITHIN_PERCENT: u32 = 10;
    pub const MILLENNIUM_RISK_BELOW_DHZ: u32 = 13;
    pub const MILLENNIUM_TMD_LATERAL_PAIRS: u32 = 4;
    pub const MILLENNIUM_TMD_VERTICAL_PAIRS: u32 = 26;
    pub const DALLARD_YEAR: u32 = 2001;

    /// A live-load deflection limit for bridges people walk on: span/1000
    /// (AASHTO, through NCHRP Web Document 46, 2002).
    pub const SPAN_OVER_SAG_LIMIT: u32 = 1000;

    /// Helmholtz, Die Lehre von den Tonempfindungen (1863), p. 147: an air
    /// space "hat gewisse Eigentöne". Hilbert, Göttinger Nachrichten 1904,
    /// pp. 51 and 64: "Eigenfunktionen", "Eigenwerte".
    pub const HELMHOLTZ_YEAR: u32 = 1863;
    pub const HILBERT_YEAR: u32 = 1904;
    /// Perron, Math. Annalen 64 (1907); Frobenius, Sitzungsberichte 1912.
    pub const PERRON_YEAR: u32 = 1907;
    pub const FROBENIUS_YEAR: u32 = 1912;
    /// Leontief: the first paper (1936), the book (1941), the first tables'
    /// 44 sectors of which 41 produce (Kohli, Monthly Labor Review, June
    /// 2001), and the prize (1973).
    pub const LEONTIEF_PAPER_YEAR: u32 = 1936;
    pub const LEONTIEF_BOOK_YEAR: u32 = 1941;
    pub const LEONTIEF_SECTORS: u32 = 44;
    pub const LEONTIEF_PRODUCING: u32 = 41;
    pub const LEONTIEF_PRIZE_YEAR: u32 = 1973;
    pub const HAWKINS_SIMON_YEAR: u32 = 1949;
    /// Boston Symphony Hall opened 15 October 1900.
    pub const SYMPHONY_HALL_OPENED: u32 = 1900;
    /// Schroeder and Kuttruff, J. Acoust. Soc. Am. 34 (1962).
    pub const SCHROEDER_YEAR: u32 = 1962;
    /// The conventional lower limit of hearing, hertz.
    pub const HEARING_FLOOR_HZ: u32 = 20;
    pub const DANTZIG_YEAR: u32 = 1947;
    pub const LANCZOS_YEAR: u32 = 1950;
    pub const BRIN_PAGE_YEAR: u32 = 1998;
    /// FrankenSim, read at this commit: the side of the grid in
    /// `crates/fs-modal/tests/modal_casebook.rs` and the length of the
    /// spring-and-mass chain in `crates/fs-modal/src/lib.rs`.
    pub const FRANKENSIM_COMMIT: &str = "e76c94213";
    pub const FRANKENSIM_GRID_SIDE: u64 = 317;
    pub const FRANKENSIM_CHAIN: usize = 40;
}

/// The examples the chapters draw that are not constants of the model.
pub mod shown {
    /// The household's demand in chapter 13 (on the even mix), and the
    /// lopsided one whose rounds swing onto it.
    pub const DEMAND: [i128; 2] = [24, 24];
    pub const OTHER_DEMAND: [i128; 2] = [30, 10];
    /// The even mix of the quiz in chapter 12.
    pub const EVEN_MIX: [i128; 2] = [10, 10];
    /// The deck twice as heavy (the quiz of chapter 07).
    pub const HEAVY_DECK_KG: f64 = 2.0 * crate::model::truss::DECK_KG;
    /// How many patterns the truss answer is rebuilt from, on the readout of chapter 06.
    pub const PATTERN_COUNTS: [usize; 4] = [1, 5, 15, 21];
    /// The sag readout's steps as patterns are added (chapter 06).
    pub const SAG_COUNTS: [usize; 7] = [1, 2, 3, 4, 5, 10, 15];
    /// Where the hall's modes are counted, hertz.
    pub const COUNT_BELOW_HZ: [u64; 3] = [20, 100, 1000];
}

fn list<T: ToString>(items: impl IntoIterator<Item = T>) -> String {
    items.into_iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")
}

/// `x` to `places` decimals. It refuses a value so close to a rounding edge
/// that another machine's last bit could round it the other way, and never
/// prints a negative zero.
pub fn fixed(x: f64, places: usize) -> Result<String, String> {
    let scale = 10f64.powi(places as i32);
    let y = x * scale;
    if (y - y.floor() - 0.5).abs() < 1e-6 {
        return Err(format!("{x} sits on a rounding edge at {places} places"));
    }
    let r = y.round() / scale;
    let r = if r == 0.0 { 0.0 } else { r };
    Ok(format!("{r:.places$}"))
}

fn fixed_list(xs: impl IntoIterator<Item = f64>, places: usize) -> Result<String, String> {
    Ok(xs
        .into_iter()
        .map(|x| fixed(x, places))
        .collect::<Result<Vec<_>, _>>()?
        .join(","))
}

/// A value that should be a whole number, as one.
fn whole(x: f64) -> Result<i64, String> {
    let r = x.round();
    if (x - r).abs() > 1e-9 * x.abs().max(1.0) {
        return Err(format!("{x} is not a whole number"));
    }
    Ok(r as i64)
}

fn wholes(xs: &[f64]) -> Result<String, String> {
    Ok(list(xs.iter().map(|&x| whole(x)).collect::<Result<Vec<_>, _>>()?))
}

/// A fraction whose denominator divides a power of ten, as a decimal.
fn decimal(x: Frac) -> Result<String, String> {
    for places in 0..=12u32 {
        let scale = 10i128.pow(places);
        if scale % x.den == 0 {
            let units = x.num * (scale / x.den);
            let (sign, units) = (if units < 0 { "-" } else { "" }, units.abs());
            return Ok(if places == 0 {
                format!("{sign}{units}")
            } else {
                format!(
                    "{sign}{}.{:0width$}",
                    units / scale,
                    units % scale,
                    width = places as usize
                )
            });
        }
    }
    Err(format!("{x} has no short decimal"))
}

fn pair(p: [i128; 2]) -> Pair {
    [Frac::whole(p[0]), Frac::whole(p[1])]
}

/// Decimetres as metres, with a decimal only when one is needed.
fn metres(dm: u64) -> String {
    if dm.is_multiple_of(10) {
        (dm / 10).to_string()
    } else {
        format!("{}.{}", dm / 10, dm % 10)
    }
}

fn date(d: cited::Date) -> String {
    format!("{:04}-{:02}-{:02}", d.0, d.1, d.2)
}

fn days_between(a: cited::Date, b: cited::Date) -> i64 {
    day_number(b.0, b.1, b.2) - day_number(a.0, a.1, a.2)
}

/// Whole calendar months from `a` to `b`.
fn months_between(a: cited::Date, b: cited::Date) -> i64 {
    (b.0 - a.0) * 12 + (b.1 - a.1) - i64::from(b.2 < a.2)
}

/// Every fact, computed once for the process (the hall's side view takes a
/// moment to solve).
pub fn facts() -> Result<Facts, String> {
    static FACTS: OnceLock<Result<Facts, String>> = OnceLock::new();
    FACTS.get_or_init(build).clone()
}

fn build() -> Result<Facts, String> {
    let mut f = Facts::new();
    bridges(&mut f)?;
    two_carts(&mut f)?;
    statics(&mut f)?;
    patterns(&mut f)?;
    vibration(&mut f)?;
    small_rooms(&mut f)?;
    hall(&mut f)?;
    section(&mut f)?;
    economy_facts(&mut f)?;
    factory_facts(&mut f)?;
    quizzes(&mut f)?;
    edges(&mut f)?;
    sources(&mut f)?;
    Ok(f)
}

/// K-B: the two bridges, from their sources, and the arithmetic on them.
fn bridges(f: &mut Facts) -> Result<(), String> {
    use cited::*;
    f.add("K-B1.opened", date(TACOMA_OPENED))?;
    f.add("K-B1.failed", date(TACOMA_FAILED))?;
    f.pin(
        "K-B1.days_open",
        days_between(TACOMA_OPENED, TACOMA_FAILED),
        "129",
        "1 July to 7 November 1940, counted on a calendar",
    )?;
    f.add("K-B1.months_open", months_between(TACOMA_OPENED, TACOMA_FAILED))?;
    f.add("K-B1.span_ft", TACOMA_SPAN_FT)?;
    f.add("K-B1.span_m", fixed(f64::from(TACOMA_SPAN_FT) * 0.3048, 0)?)?;
    f.add("K-B1.wind_mph", TACOMA_WIND_MPH)?;
    f.add(
        "K-B1.wind_round_mph",
        (TACOMA_WIND_MPH as f64 / 10.0).round() as u32 * 10,
    )?;
    f.add("K-B1.wind_km_per_h", fixed(f64::from(TACOMA_WIND_MPH) * 1.609_344, 0)?)?;
    f.add("K-B1.twist_per_min", TACOMA_TWIST_PER_MIN)?;
    f.add("K-B1.twist_hz", fixed(f64::from(TACOMA_TWIST_PER_MIN) / 60.0, 1)?)?;
    f.add("K-B1.twist_period_s", fixed(60.0 / f64::from(TACOMA_TWIST_PER_MIN), 0)?)?;
    f.add("K-B1.vortex_hz", TACOMA_VORTEX_HZ)?;
    f.add("K-B1.study_year", BILLAH_SCANLAN_YEAR)?;
    f.add("K-B1.board_year", TACOMA_BOARD_YEAR)?;

    f.add("K-B2.opened", date(MILLENNIUM_OPENED))?;
    f.add("K-B2.closed", date(MILLENNIUM_CLOSED))?;
    f.add("K-B2.days_to_close", days_between(MILLENNIUM_OPENED, MILLENNIUM_CLOSED))?;
    f.add("K-B2.reopened", date(MILLENNIUM_REOPENED))?;
    f.add("K-B2.reopened_year", MILLENNIUM_REOPENED.0)?;
    f.add(
        "K-B2.first_day",
        format!("{}-{}", MILLENNIUM_FIRST_DAY.0, MILLENNIUM_FIRST_DAY.1),
    )?;
    f.add("K-B2.on_deck", MILLENNIUM_ON_DECK)?;
    f.add("K-B2.sway_mm", MILLENNIUM_SWAY_MM)?;
    f.add("K-B2.sway_cm", fixed(f64::from(MILLENNIUM_SWAY_MM) / 10.0, 0)?)?;
    let lateral: Vec<f64> = MILLENNIUM_LATERAL_CHZ.iter().map(|&c| f64::from(c) / 100.0).collect();
    f.add("K-B2.lateral_hz", fixed_list(lateral.iter().copied(), 2)?)?;
    f.add("K-B2.lateral_lowest_hz", fixed(lateral[0], 1)?)?;
    // "about once a second": the modes within a fifth of a hertz of one
    f.add(
        "K-B2.lateral_near_one",
        lateral.iter().filter(|hz| (*hz - 1.0).abs() <= 0.2 + 1e-9).count(),
    )?;
    f.add("K-B2.walk_hz", fixed(f64::from(WALKING_DHZ) / 10.0, 1)?)?;
    // a walker pushes sideways once for every two footfalls
    f.add("K-B2.walker_lateral_hz", fixed(f64::from(WALKING_DHZ) / 20.0, 1)?)?;
    f.add("K-B2.test_steady", MILLENNIUM_TEST_STEADY)?;
    f.add("K-B2.test_added", MILLENNIUM_TEST_ADDED)?;
    f.add("K-B2.dampers_viscous", MILLENNIUM_VISCOUS_DAMPERS)?;
    f.add("K-B2.paper_year", DALLARD_YEAR)?;
    f.add("K-B2.predicted_within_percent", MILLENNIUM_PREDICTED_WITHIN_PERCENT)?;
    f.add(
        "K-B2.risk_below_hz",
        fixed(f64::from(MILLENNIUM_RISK_BELOW_DHZ) / 10.0, 1)?,
    )?;
    f.add("K-B2.tmd_lateral_pairs", MILLENNIUM_TMD_LATERAL_PAIRS)?;
    f.add("K-B2.tmd_vertical_pairs", MILLENNIUM_TMD_VERTICAL_PAIRS)?;
    Ok(())
}

/// K-A: two carts and three springs.
fn two_carts(f: &mut Facts) -> Result<(), String> {
    let k = carts::table();
    f.add("K-A1.table", format!("{};{}", wholes(&k.a[0..2])?, wholes(&k.a[2..4])?))?;
    f.pin(
        "K-A1.push.left",
        wholes(&carts::pushes([1.0, 0.0]))?,
        "2,-1",
        "two springs on the moved cart, one on the held one",
    )?;
    f.pin(
        "K-A1.push.together",
        wholes(&carts::pushes([1.0, 1.0]))?,
        "1,1",
        "the middle spring unstretched",
    )?;
    f.pin(
        "K-A1.push.against",
        wholes(&carts::pushes([1.0, -1.0]))?,
        "3,-3",
        "the middle spring squeezed from both ends",
    )?;

    let (lined_up, worst) = carts::sweep();
    f.add("K-A2.lined_up_deg", fixed_list(lined_up.iter().copied(), 0)?)?;
    f.add("K-A2.directions", carts::own_directions(&k))?;
    f.pin(
        "K-A2.worst_turn_deg",
        fixed(worst, 0)?,
        "30",
        "sin(turn) = (3 - 1)/(3 + 1)",
    )?;
    f.add("K-A2.turn_left_deg", fixed(carts::turn_degrees(0.0), 1)?)?;
    let e = carts::eigen();
    f.pin("K-A2.eigenvalues", wholes(&e.values)?, "1,3", "trace 4, determinant 3")?;
    // a pattern as the video draws it: scaled so that the left cart's number is 1
    let as_drawn = |q: &[f64]| -> Result<[i64; 2], String> { Ok([1, whole(q[1] / q[0])?]) };
    let (soft, stiff) = (as_drawn(&e.vectors[0])?, as_drawn(&e.vectors[1])?);
    f.add("K-A2.pattern.soft", list(soft))?;
    f.add("K-A2.pattern.stiff", list(stiff))?;

    // the split of a push of 1 on the left cart, in exact fractions, on the
    // patterns as drawn (1, 1) and (1, −1)
    let push = [1i128, 0];
    let mut mix = Vec::new();
    let mut shares = Vec::new();
    let mut moves = [Frac::whole(0); 2];
    for (pattern, lambda) in [soft, stiff].iter().zip(&e.values) {
        let p = [i128::from(pattern[0]), i128::from(pattern[1])];
        let held = Frac::new(p[0] * push[0] + p[1] * push[1], p[0] * p[0] + p[1] * p[1]);
        let share = held / Frac::whole(i128::from(whole(*lambda)?));
        for (m, p) in moves.iter_mut().zip(p) {
            *m = *m + share * Frac::whole(p);
        }
        mix.push(held);
        shares.push(share);
    }
    f.pin("K-A3.mix", list(mix), "1/2,1/2", "(1, 0) = (1, 1)/2 + (1, -1)/2")?;
    f.pin("K-A3.shares", list(shares), "1/2,1/6", "each half over its stiffness")?;
    f.pin("K-A3.moves", list(moves), "2/3,1/3", "K (2/3, 1/3) = (1, 0)")?;
    // the quiz of chapter 04: the same split for a push of 1 on the right cart
    let mut right = [Frac::whole(0); 2];
    for (pattern, lambda) in [soft, stiff].iter().zip(&e.values) {
        let p = [i128::from(pattern[0]), i128::from(pattern[1])];
        let share = Frac::new(p[1], p[0] * p[0] + p[1] * p[1]) / Frac::whole(i128::from(whole(*lambda)?));
        for (m, p) in right.iter_mut().zip(p) {
            *m = *m + share * Frac::whole(p);
        }
    }
    f.pin(
        "K-A3.moves_right",
        list(right),
        "1/3,2/3",
        "the mirror image of 2/3, 1/3",
    )?;
    let direct = carts::moves([1.0, 0.0]);
    if (direct[0] - moves[0].to_f64()).abs() > 1e-12 || (direct[1] - moves[1].to_f64()).abs() > 1e-12 {
        return Err(format!("the split {moves:?} is not the direct answer {direct:?}"));
    }

    f.add("K-A4.spring_n_per_m", fixed(carts::SPRING_N_PER_M, 0)?)?;
    f.add("K-A4.cart_kg", fixed(carts::CART_KG, 0)?)?;
    let hz = carts::swings_per_second();
    f.add("K-A4.hz.soft", fixed(hz[0], 1)?)?;
    f.add("K-A4.hz.stiff", fixed(hz[1], 1)?)?;
    f.add("K-A4.ratio", fixed(hz[1] / hz[0], 1)?)?;
    Ok(())
}

/// The largest entry of a list, in size.
fn largest(xs: &[f64]) -> f64 {
    xs.iter().fold(0.0f64, |s, x| s.max(x.abs()))
}

/// K-T: the truss, its table and its answer to a load.
fn statics(f: &mut Facts) -> Result<(), String> {
    let t = Truss::pratt();
    f.add("K-T1.span_m", fixed(t.span_m(), 0)?)?;
    f.add("K-T1.depth_m", fixed(truss::DEPTH_M, 0)?)?;
    f.add("K-T1.panels", truss::PANELS)?;
    f.add("K-T1.panel_m", fixed(truss::PANEL_M, 0)?)?;
    f.add("K-T1.joints", t.joints.len())?;
    f.add("K-T1.members", t.members.len())?;
    f.add("K-T1.held", t.held.len())?;
    f.pin(
        "K-T1.free",
        t.free().len(),
        "21",
        "12 joints, two directions each, less three held",
    )?;
    f.add(
        "K-T1.counts",
        list(Kind::ALL.map(|kind| t.members.iter().filter(|m| m.kind == kind).count())),
    )?;
    f.add("K-T1.e_gpa", fixed(truss::E_PA / 1e9, 0)?)?;
    f.add("K-T1.chord_mm2", fixed(truss::CHORD_AREA_M2 * 1e6, 0)?)?;
    f.add("K-T1.web_mm2", fixed(truss::WEB_AREA_M2 * 1e6, 0)?)?;
    let bottom = t
        .members
        .iter()
        .find(|m| m.kind == Kind::Bottom)
        .ok_or("no bottom member")?;
    f.pin(
        "K-T1.spring_kn_per_mm",
        fixed(t.spring(bottom) / 1e6, 0)?,
        "300",
        "200 GPa x 6000 mm2 / 4 m",
    )?;
    // the weight, in tonnes, that pulls as hard as 300 kN
    f.add(
        "K-T1.spring_tonnes",
        fixed(t.spring(bottom) * 1e-3 / 9.806_65 / 1e3, 1)?,
    )?;
    f.add("K-T1.yield_mpa", fixed(truss::YIELD_PA / 1e6, 0)?)?;
    f.add("K-T1.steel_kg_per_m3", fixed(truss::STEEL_KG_PER_M3, 0)?)?;

    let k = t.stiffness();
    let top = largest(&k.a);
    f.add("K-T2.entries", k.n * k.n)?;
    f.add("K-T2.nonzero", k.a.iter().filter(|x| x.abs() > 1e-9 * top).count())?;
    f.add("K-T2.symmetric", k.asymmetry() <= 1e-9 * top)?;
    // the inverse, a column at a time: the moves for a unit push on each direction
    let mut inverse_nonzero = 0;
    for j in 0..k.n {
        let mut unit = vec![0.0; k.n];
        unit[j] = 1.0;
        let column = crate::model::linalg::solve(&k, &unit).ok_or("K is singular")?;
        let big = largest(&column);
        inverse_nonzero += column.iter().filter(|x| x.abs() > 1e-9 * big).count();
    }
    f.add("K-T2.inverse_nonzero", inverse_nonzero)?;

    // one load, on one joint
    f.add("K-T3.load_kn", fixed(truss::DECK_LOAD_N / 1e3, 0)?)?;
    f.add("K-T3.load_tonnes", fixed(truss::DECK_LOAD_N / 9.806_65 / 1e3, 1)?)?;
    f.add("K-T3.joint", truss::LONE_JOINT)?;
    let lone = t.moves(&t.lone_load());
    f.add(
        "K-T3.moving",
        lone.iter().filter(|u| u.abs() > 1e-9 * largest(&lone)).count(),
    )?;
    let lone_all = t.all_moves(&lone);
    f.add("K-T3.drop_mm", fixed(-lone_all[2 * truss::LONE_JOINT + 1] * 1e3, 1)?)?;
    f.add("K-T3.slide_mm", fixed(lone_all[2 * truss::PANELS] * 1e3, 1)?)?;

    // the worked load, on every deck joint
    let load = t.deck_load();
    let moves = t.moves(&load);
    let all = t.all_moves(&moves);
    f.add("K-T4.loads", t.deck_joints().len())?;
    f.add(
        "K-T4.total_kn",
        fixed(truss::DECK_LOAD_N * t.deck_joints().len() as f64 / 1e3, 0)?,
    )?;
    let reactions = t.reactions(&moves);
    f.pin(
        "K-T4.reaction_kn",
        fixed(reactions[1] / 1e3, 0)?,
        "250",
        "half of five loads of 100 kN",
    )?;
    let sag = -all[2 * t.midspan() + 1];
    f.add("K-T4.sag_mm", fixed(sag * 1e3, 1)?)?;
    f.add("K-T4.sag_mm_2dp", fixed(sag * 1e3, 2)?)?;
    f.add("K-T4.span_over_sag", fixed(t.span_m() / sag, 0)?)?;
    f.add("K-T4.span_over_sag_limit", cited::SPAN_OVER_SAG_LIMIT)?;

    let forces = t.member_forces(&moves);
    f.add("K-T5.forces_kn", fixed_list(forces.iter().map(|n| n / 1e3), 1)?)?;
    let of = |kind: Kind| -> Vec<f64> {
        t.members
            .iter()
            .zip(&forces)
            .filter(|(m, _)| m.kind == kind)
            .map(|(_, n)| n / 1e3)
            .collect()
    };
    let most_squeezed = |xs: &[f64]| xs.iter().copied().fold(f64::INFINITY, f64::min);
    let most_stretched = |xs: &[f64]| xs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    f.pin(
        "K-T5.top_kn",
        fixed(most_squeezed(&of(Kind::Top)), 0)?,
        "-450",
        "midspan moment 1800 kN m over a depth of 4 m",
    )?;
    f.pin(
        "K-T5.bottom_kn",
        fixed(most_stretched(&of(Kind::Bottom)), 0)?,
        "400",
        "the moment 8 m from a support, 1600 kN m, over 4 m",
    )?;
    let middle_post = t
        .members
        .iter()
        .position(|m| m.kind == Kind::Vertical && m.a == t.midspan())
        .ok_or("no middle post")?;
    f.pin(
        "K-T5.middle_post_kn",
        fixed(forces[middle_post] / 1e3, 0)?,
        "0",
        "its top joint has no load and no other upright member",
    )?;
    f.add("K-T5.end_post_kn", fixed(most_squeezed(&of(Kind::EndPost)), 1)?)?;
    f.add("K-T5.diagonal_kn", fixed(most_stretched(&of(Kind::Diagonal)), 1)?)?;
    let (stress, kind) = t
        .members
        .iter()
        .zip(&forces)
        .map(|(m, n)| (n.abs() / m.kind.area_m2(), m.kind))
        .fold((0.0, Kind::Bottom), |best, s| if s.0 > best.0 { s } else { best });
    f.add("K-T5.stress_mpa", fixed(stress / 1e6, 0)?)?;
    f.add("K-T5.stress_member", kind.label())?;
    f.add(
        "K-T5.stress_percent_of_yield",
        fixed(stress / truss::YIELD_PA * 100.0, 0)?,
    )?;
    Ok(())
}

/// K-P: the stiffness table's own patterns, and the load split over them.
fn patterns(f: &mut Facts) -> Result<(), String> {
    let t = Truss::pratt();
    let e = t.patterns();
    let (soft, stiff) = (e.values[0], e.values[e.values.len() - 1]);
    f.add("K-P1.count", e.values.len())?;
    f.add("K-P1.mn_per_m", fixed_list(e.values.iter().map(|v| v / 1e6), 2)?)?;
    f.add("K-P1.softest_mn_per_m", fixed(soft / 1e6, 2)?)?;
    f.add("K-P1.stiffest_mn_per_m", fixed(stiff / 1e6, 0)?)?;
    f.add("K-P1.ratio", fixed(stiff / soft, 0)?)?;
    f.add("K-P1.softest_deck_sign_changes", t.deck_sign_changes(&e.vectors[0]))?;
    f.add(
        "K-P1.stiffest_chord_sign_changes",
        t.chord_sign_changes(&e.vectors[e.vectors.len() - 1]),
    )?;
    let mut worst = 0.0f64;
    for i in 0..e.vectors.len() {
        for j in 0..i {
            worst = worst.max(dot(&e.vectors[i], &e.vectors[j]).abs());
        }
    }
    f.add("K-P1.at_right_angles", worst < 1e-9)?;

    let load = t.deck_load();
    let direct = t.moves(&load);
    let direct_forces = t.member_forces(&direct);
    let mid = t
        .free()
        .iter()
        .position(|&d| d == 2 * t.midspan() + 1)
        .ok_or("midspan is held")?;
    for count in shown::SAG_COUNTS {
        let u = t.moves_from_patterns(&e, &load, count);
        f.add(
            &format!("K-P2.sag_percent.{count}"),
            fixed(u[mid] / direct[mid] * 100.0, 1)?,
        )?;
    }
    // the carts' test on the truss: the angle between the move one load
    // causes and the push that holds it, and the same for the softest pattern
    let lone = t.moves(&t.lone_load());
    let k = t.stiffness();
    let angle = |u: &[f64]| -> f64 {
        let f = k.mul_vec(u);
        (dot(u, &f) / (dot(u, u) * dot(&f, &f)).sqrt())
            .clamp(-1.0, 1.0)
            .acos()
            .to_degrees()
    };
    f.add("K-P3.lone_turn_deg", fixed(angle(&lone), 0)?)?;
    f.add("K-P3.softest_turn_deg", fixed(angle(&e.vectors[0]), 1)?)?;
    for count in shown::PATTERN_COUNTS {
        let u = t.moves_from_patterns(&e, &load, count);
        let off = t
            .member_forces(&u)
            .iter()
            .zip(&direct_forces)
            .fold(0.0f64, |s, (a, b)| s.max((a - b).abs()));
        f.add(&format!("K-P2.force_error_kn.{count}"), fixed(off / 1e3, 0)?)?;
        if count == 1 {
            // with the softest pattern alone: the worst force error against
            // the largest force, and the worst error anywhere in the moves
            f.add(
                "K-P2.force_error_percent.1",
                fixed(off / largest(&direct_forces) * 100.0, 0)?,
            )?;
            let astray = u.iter().zip(&direct).fold(0.0f64, |s, (a, b)| s.max((a - b).abs()));
            f.add("K-P2.move_error_mm.1", fixed(astray * 1e3, 1)?)?;
        }
    }
    // one sharp load (100 kN on one deck joint) against the same truncation:
    // how much of the drop right under it the softest pattern gives, 4, 8 and
    // 12 m from the pin (R2 D5: "good enough for how far this load sags" is
    // true of the five-load deck load and of no sharper one)
    for (joint, metres) in [(1usize, 4u32), (2, 8), (3, 12)] {
        let sharp = t.pushes(&[(joint, 0.0, -truss::DECK_LOAD_N)]);
        let direct = t.moves(&sharp);
        let under = t
            .free()
            .iter()
            .position(|&d| d == 2 * joint + 1)
            .ok_or("a deck joint is held")?;
        let one = t.moves_from_patterns(&e, &sharp, 1);
        f.add(
            &format!("K-P2.lone_pct.{metres}m"),
            fixed(one[under] / direct[under] * 100.0, 0)?,
        )?;
        if metres == 12 {
            // how many patterns bring the midspan load within 2.5% under it
            let need = (1..=e.values.len())
                .find(|&n| {
                    let u = t.moves_from_patterns(&e, &sharp, n);
                    (u[under] / direct[under] - 1.0).abs() < 0.025
                })
                .ok_or("no count of patterns comes within 2.5%")?;
            f.add("K-P2.lone_within_2p5.12m", need)?;
        }
    }
    Ok(())
}

/// What a mode looks like, in one word: "along" when most of its motion
/// (counted with mass) is along the deck, else "sag" for a single curve, "s"
/// for one change of sign along the deck, "wavy" for more.
fn shape(t: &Truss, deck_kg: f64, phi: &[f64]) -> &'static str {
    let (mut along, mut total) = (0.0, 0.0);
    for ((&d, m), p) in t.free().iter().zip(t.mass(deck_kg)).zip(phi) {
        let e = m * p * p;
        total += e;
        if d % 2 == 0 {
            along += e;
        }
    }
    if along > 0.5 * total {
        return "along";
    }
    match t.deck_sign_changes(phi) {
        0 => "sag",
        1 => "s",
        _ => "wavy",
    }
}

/// K-V: with mass, each pattern is a note.
fn vibration(f: &mut Facts) -> Result<(), String> {
    let t = Truss::pratt();
    let deck = truss::DECK_KG;
    f.add("K-V1.deck_kg", fixed(deck, 0)?)?;
    f.add("K-V1.deck_tonnes", fixed(deck / 1e3, 0)?)?;
    f.add("K-V1.steel_kg", fixed(t.steel_kg(), 0)?)?;
    f.add("K-V1.total_kg", fixed(t.joint_kg(deck).iter().sum(), 0)?)?;
    let modes = t.modes(deck);
    let hz = t.hertz(deck);
    for (k, hz) in hz.iter().enumerate().take(4) {
        f.add(&format!("K-V1.hz.{}", k + 1), fixed(*hz, 1)?)?;
    }
    f.add("K-V1.hz_2dp", fixed_list(hz.iter().take(4).copied(), 2)?)?;
    f.add("K-V1.ratio.2", fixed(hz[1] / hz[0], 2)?)?;
    f.add("K-V1.ratio.3", fixed(hz[2] / hz[0], 2)?)?;
    f.add("K-V1.steel_tonnes", fixed(t.steel_kg() / 1e3, 1)?)?;
    for (k, phi) in modes.vectors.iter().enumerate().take(3) {
        f.add(&format!("K-V1.shape.{}", k + 1), shape(&t, deck, phi))?;
    }
    let mass = t.mass(deck);
    let (mut with_mass, mut plain) = (0.0f64, 0.0f64);
    for i in 0..modes.vectors.len() {
        for j in 0..i {
            let (a, b) = (&modes.vectors[i], &modes.vectors[j]);
            let m: f64 = a.iter().zip(b).zip(&mass).map(|((a, b), m)| a * m * b).sum();
            with_mass = with_mass.max(m.abs());
            plain = plain.max(dot(a, b).abs() / (dot(a, a) * dot(b, b)).sqrt());
        }
    }
    f.add("K-V1.at_right_angles_with_mass", with_mass < 1e-9)?;
    f.add("K-V1.at_right_angles_without", plain < 1e-9)?;
    // how close each mode is to K's own pattern of the same rank: the squared
    // cosine between them (R2 R-04: "at the low end the own patterns shift a
    // little" is true of the lowest few modes only)
    let own = t.patterns();
    for (k, (phi, q)) in modes.vectors.iter().zip(&own.vectors).enumerate().take(8) {
        let c = dot(phi, q);
        f.add(
            &format!("K-V1.overlap.{}", k + 1),
            fixed(c * c / (dot(phi, phi) * dot(q, q)), 2)?,
        )?;
    }

    let heavy = t.hertz(shown::HEAVY_DECK_KG);
    f.add("K-V2.deck_kg", fixed(shown::HEAVY_DECK_KG, 0)?)?;
    f.add("K-V2.hz.1", fixed(heavy[0], 1)?)?;
    f.add("K-V2.direction", if heavy[0] < hz[0] { "down" } else { "up" })?;
    // what dividing by the square root of two would give, were the steel doubled too
    f.add("K-V2.naive_hz", fixed(hz[0] / 2f64.sqrt(), 1)?)?;
    // the same two to two decimals, which 07_5 shows beside 5.91 Hz
    f.add("K-V2.hz_2dp", fixed(heavy[0], 2)?)?;
    f.add("K-V2.naive_hz_2dp", fixed(hz[0] / 2f64.sqrt(), 2)?)?;
    Ok(())
}

fn sides(room: &Room) -> String {
    list(room.dm.map(metres))
}

/// The lowest tone of a room, and how many modes share it.
fn lowest(room: &Room) -> Result<(f64, usize), String> {
    // every room here has its first mode below 100 Hz
    let modes = room.modes_below(100);
    let first = modes.first().ok_or("no mode below 100 Hz")?.1;
    Ok((first, modes.iter().filter(|(_, hz)| (hz - first).abs() < 1e-9).count()))
}

/// K-R: air as carts, and three small rooms.
fn small_rooms(f: &mut Facts) -> Result<(), String> {
    f.add("K-R0.sound_m_per_s", fixed(rooms::SOUND_M_PER_S, 0)?)?;
    let slices = cited::FRANKENSIM_CHAIN;
    f.add("K-R1.slices", slices)?;
    let error = (1.0 - carts::chain_against_the_tube(slices, 1)) * 100.0;
    f.add("K-R1.error_percent", fixed(error, 2)?)?;
    f.add("K-R1.error_bound_percent", fixed((error * 100.0).ceil() / 100.0, 2)?)?;
    // the carts' test on the air: one slice pushed out of place comes back
    // as a jagged push (2 on itself, −1 on each neighbour)
    let chain = carts::chain(slices);
    let mut lump = vec![0.0; slices];
    lump[slices / 2] = 1.0;
    let back = chain.mul_vec(&lump);
    f.add(
        "K-R1.lump_turn_deg",
        fixed((dot(&lump, &back) / dot(&back, &back).sqrt()).acos().to_degrees(), 1)?,
    )?;

    let bath = rooms::BATHROOM;
    let modes = bath.modes_below(100);
    f.add("K-R2.sides_m", sides(&bath))?;
    f.add("K-R2.first_hz", fixed(modes[0].1, 1)?)?;
    f.add("K-R2.first_mode", list(modes[0].0))?;
    f.add("K-R2.second_hz", fixed(modes[1].1, 1)?)?;
    f.add("K-R2.second_mode", list(modes[1].0))?;
    // two tones a voice can reach: one half-wave across, two along
    let voice: Vec<f64> = [[0, 1, 0], [2, 0, 0]].iter().map(|n| bath.mode_hz(*n)).collect();
    f.add("K-R2.voice_hz", fixed_list(voice, 1)?)?;
    // every tone of the bathroom up to 150 Hz, for 09_5's pitch line (D-53):
    // nine, of which five fit one side of the room
    let tones: Vec<f64> = bath.modes_below(150).iter().map(|m| m.1).collect();
    f.pin(
        "K-R2.tones_to_150_hz",
        fixed_list(tones, 1)?,
        "68.6,71.5,99.1,107.2,127.3,128.8,137.2,142.9,146.0",
        "343 / 2 x sqrt((nx / 2.5)^2 + (ny / 1.6)^2 + (nz / 2.4)^2) for every mode up to 150 Hz",
    )?;
    // the quiz: the speed of sound over twice the length
    f.add(
        "K-R2.rule_hz",
        fixed(rooms::SOUND_M_PER_S / (2.0 * bath.sides_m()[0]), 1)?,
    )?;

    let cube = rooms::CUBE;
    let (first, count) = lowest(&cube)?;
    f.add("K-R3.side_m", metres(cube.dm[0]))?;
    f.add("K-R3.first_hz", fixed(first, 1)?)?;
    f.add("K-R3.first_count", count)?;

    let study = rooms::STUDY;
    f.add("K-R4.sides_m", sides(&study))?;
    f.add("K-R4.volume_m3", fixed(study.volume_m3(), 0)?)?;
    f.add("K-R4.t60_s", fixed(rooms::STUDY_T60_S, 1)?)?;
    let crossover = study.schroeder_hz(rooms::STUDY_T60_S);
    f.add("K-R4.schroeder_hz", fixed(crossover, 0)?)?;
    let below = crossover.round() as u64;
    f.add("K-R4.modes_below", study.count_below(below))?;
    f.add("K-R4.sharing_a_tone", study.sharing_a_tone(below))?;

    // the quiz of chapter 11: below 200 Hz, how many modes share a tone
    let mut worst = ("", 0);
    for room in [rooms::NARROW, study, cube] {
        let shared = room.sharing_a_tone(200);
        f.add(&format!("K-R5.modes_below_200.{}", room.name), room.count_below(200))?;
        f.add(&format!("K-R5.sharing_200.{}", room.name), shared)?;
        if shared > worst.1 {
            worst = (room.name, shared);
        }
    }
    f.add("K-R5.narrow_sides_m", sides(&rooms::NARROW))?;
    f.add("K-R5.worst", worst.0)?;
    Ok(())
}

/// K-H: a box the size of a concert hall.
fn hall(f: &mut Facts) -> Result<(), String> {
    let hall = rooms::HALL;
    f.add("K-H1.sides_m", sides(&hall))?;
    f.add("K-H1.volume_m3", fixed(hall.volume_m3(), 0)?)?;
    f.add("K-H1.boston_m3", fixed(rooms::BOSTON_M3, 0)?)?;
    f.add(
        "K-H1.volume_percent",
        fixed(hall.volume_m3() / rooms::BOSTON_M3 * 100.0, 1)?,
    )?;
    f.add("K-H1.lowest_hz", fixed(hall.mode_hz([1, 0, 0]), 1)?)?;
    f.add("K-H1.t60_s", fixed(rooms::HALL_T60_S, 1)?)?;
    for hertz in shown::COUNT_BELOW_HZ {
        f.add(&format!("K-H2.count.{hertz}"), hall.count_below(hertz))?;
    }
    f.add("K-H2.estimate.1000", fixed(hall.count_estimate(1000.0), 0)?)?;
    f.add("K-H2.per_hz_at_1000", fixed(hall.modes_per_hertz(1000.0), 0)?)?;
    // a mode's half-power width: sound that dies by 60 dB in T seconds
    // decays as e^(−δt) in amplitude with δ = ln(10⁶)/2T, and the width is δ/π
    let width = 1e6f64.ln() / (2.0 * std::f64::consts::PI * rooms::HALL_T60_S);
    f.add("K-H2.mode_width_hz", fixed(width, 1)?)?;
    f.add(
        "K-H2.modes_per_width_at_1000",
        fixed(hall.modes_per_hertz(1000.0) * width, 0)?,
    )?;
    let crossover = hall.schroeder_hz(rooms::HALL_T60_S);
    f.add("K-H3.schroeder_hz", fixed(crossover, 0)?)?;
    f.add("K-H3.schroeder_hz_1dp", fixed(crossover, 1)?)?;
    f.add("K-H3.hearing_floor_hz", cited::HEARING_FLOOR_HZ)?;
    f.add("K-H4.sabine_m2", fixed(hall.sabine_m2(rooms::HALL_T60_S), 0)?)?;
    f.add("K-H4.sabine_constant", fixed(rooms::SABINE, 3)?)?;
    Ok(())
}

/// K-N: a small room that is not a box, cut into cells.
fn section(f: &mut Facts) -> Result<(), String> {
    let (attic, e) = rooms::attic_section();
    f.add("K-N1.cols", attic.cols)?;
    f.add("K-N1.rows", attic.rows)?;
    f.add("K-N1.cell_m", fixed(attic.cell_m, 2)?)?;
    f.add(
        "K-N1.sides_m",
        format!(
            "{},{}",
            fixed(attic.cols as f64 * attic.cell_m, 0)?,
            fixed(attic.rows as f64 * attic.cell_m, 0)?
        ),
    )?;
    let low_end = attic.air.iter().filter(|row| row[attic.cols - 1]).count();
    f.add("K-N1.low_end_m", fixed(low_end as f64 * attic.cell_m, 2)?)?;
    f.add("K-N1.cells", attic.cells().len())?;
    // the first eigenvalue is still air at rest; the modes start at the second
    for (k, lambda) in e.values.iter().enumerate().skip(1).take(3) {
        f.add(&format!("K-N1.hz.{k}"), fixed(attic.hertz(*lambda), 2)?)?;
    }
    let plain = Section::boxed();
    let (formula, cells) = (plain.box_formula_hz(1, 0), plain.box_cells_hz(1, 0));
    f.add("K-N1.box.cells", plain.cells().len())?;
    f.add("K-N1.box.formula_hz", fixed(formula, 2)?)?;
    f.add("K-N1.box.cells_hz", fixed(cells, 2)?)?;
    f.add("K-N1.box.error_percent", fixed((1.0 - cells / formula) * 100.0, 2)?)?;
    Ok(())
}

fn rows(m: [[Frac; 2]; 2]) -> String {
    format!("{};{}", list(m[0]), list(m[1]))
}

fn tenths(t: &Table) -> Result<String, String> {
    let ten = Frac::whole(10);
    let row = |r: [Frac; 2]| -> Result<String, String> {
        let scaled = r.map(|x| x * ten);
        if scaled.iter().any(|x| x.den != 1) {
            return Err("the table is not in tenths".into());
        }
        Ok(list(scaled))
    };
    Ok(format!("{};{}", row(t.c[0])?, row(t.c[1])?))
}

/// K-E: an economy of two industries.
fn economy_facts(f: &mut Facts) -> Result<(), String> {
    let c = economy::thrifty();
    f.add("K-E1.table", rows(c.c))?;
    f.add("K-E1.table_tenths", tenths(&c)?)?;
    f.pin(
        "K-E1.uses_even",
        list(c.eats(pair(shown::EVEN_MIX))),
        "7,7",
        "4 + 3 and 2 + 5",
    )?;
    let values = c.eigenvalues().ok_or("the table's eigenvalues are not fractions")?;
    f.pin(
        "K-E2.eigenvalues",
        list(values),
        "7/10,1/5",
        "trace 9/10, determinant 14/100",
    )?;
    f.add("K-E2.largest", decimal(values[0])?)?;
    f.add("K-E2.largest_percent", decimal(values[0] * Frac::whole(100))?)?;
    f.add(
        "K-E2.kept_percent",
        decimal((Frac::whole(1) - values[0]) * Frac::whole(100))?,
    )?;
    f.add("K-E2.mix", list(c.eigenvector(values[0])))?;
    f.add("K-E2.other", list(c.eigenvector(values[1])))?;
    f.add("K-E2.prices", list(c.left_eigenvector(values[0])))?;
    // where the two own directions lie in the arrow plane, as angles from
    // the food axis within half a turn
    for (name, value) in [("mix", values[0]), ("other", values[1])] {
        let v = c.eigenvector(value);
        let degrees = (v[1] as f64).atan2(v[0] as f64).to_degrees().rem_euclid(180.0);
        f.add(&format!("K-E2.{name}_direction_deg"), fixed(degrees, 1)?)?;
    }

    let demand = pair(shown::DEMAND);
    f.add("K-E3.demand", list(demand))?;
    let rounds = c.rounds(demand, 3);
    f.add(
        "K-E3.rounds",
        list(rounds.iter().map(|r| decimal(r[0])).collect::<Result<Vec<_>, _>>()?),
    )?;
    if rounds.iter().any(|r| r[0] != r[1]) {
        return Err("the even demand's rounds are not even".into());
    }
    let total = c.output_for(demand).ok_or("I - C has no inverse")?;
    f.pin("K-E3.total", list(total), "80,80", "24 / (1 - 7/10)")?;
    f.add(
        "K-E3.multiplier",
        (Frac::whole(1) / (Frac::whole(1) - values[0])).to_string(),
    )?;
    f.add(
        "K-E3.inverse",
        rows(c.leontief_inverse().ok_or("I - C has no inverse")?),
    )?;
    f.add(
        "K-E3.other_total",
        list(c.output_for(pair(shown::OTHER_DEMAND)).ok_or("I - C has no inverse")?),
    )?;
    // how many rounds (round 0 counted) until 99% of the total has been made
    let mut made = Frac::whole(0);
    let mut needed = 0;
    for (i, r) in c.rounds(demand, 20).iter().enumerate() {
        made = made + r[0];
        if !(made * Frac::whole(100) - total[0] * Frac::whole(99)).is_negative() {
            needed = i + 1;
            break;
        }
    }
    f.add("K-E3.rounds_for_99_percent", needed)?;
    f.add("K-E3.feeds_itself", c.feeds_itself())?;

    let h = economy::hungry();
    f.add("K-E4.table_tenths", tenths(&h)?)?;
    let hungry_values = h
        .eigenvalues()
        .ok_or("the hungry table's eigenvalues are not fractions")?;
    f.add("K-E4.largest", decimal(hungry_values[0])?)?;
    f.add("K-E4.uses_even", list(h.eats(pair(shown::EVEN_MIX))))?;
    f.add(
        "K-E4.rounds",
        list(
            h.rounds(demand, 2)
                .iter()
                .map(|r| decimal(r[0]))
                .collect::<Result<Vec<_>, _>>()?,
        ),
    )?;
    f.add("K-E4.feeds_itself", h.feeds_itself())?;
    let hungry_largest = hungry_values[0];
    f.add(
        "K-E4.formula_says",
        decimal(demand[0] / (Frac::whole(1) - hungry_largest))?,
    )?;

    // the lopsided demand: a mix of the two own patterns, each divided by
    // one less its eigenvalue
    let other = pair(shown::OTHER_DEMAND);
    let (mix, rest) = (c.eigenvector(values[0]), c.eigenvector(values[1]));
    let (a, b) = (
        [Frac::whole(mix[0]), Frac::whole(mix[1])],
        [Frac::whole(rest[0]), Frac::whole(rest[1])],
    );
    let det = a[0] * b[1] - b[0] * a[1];
    let shares = [
        (other[0] * b[1] - b[0] * other[1]) / det,
        (a[0] * other[1] - other[0] * a[1]) / det,
    ];
    f.add("K-E5.demand", list(other))?;
    f.add("K-E5.shares", list(shares))?;
    let kept = [
        shares[0] / (Frac::whole(1) - values[0]),
        shares[1] / (Frac::whole(1) - values[1]),
    ];
    f.add("K-E5.totals_by_pattern", list(kept))?;
    let total = [kept[0] * a[0] + kept[1] * b[0], kept[0] * a[1] + kept[1] * b[1]];
    if Some(total) != c.output_for(other) {
        return Err("the lopsided demand's split disagrees with the inverse".into());
    }
    f.add("K-E5.total", list(total))?;
    f.add(
        "K-E5.rounds",
        c.rounds(other, 3)
            .iter()
            .map(|r| Ok(format!("{};{}", decimal(r[0])?, decimal(r[1])?)))
            .collect::<Result<Vec<_>, String>>()?
            .join(" "),
    )?;
    Ok(())
}

fn point(p: (Frac, Frac)) -> String {
    format!("{},{}", p.0, p.1)
}

/// K-L: the factory's linear program.
fn factory_facts(f: &mut Facts) -> Result<(), String> {
    f.add(
        "K-L1.limits",
        factory::LIMITS
            .iter()
            .map(|l| format!("{},{},{}", l.bicycles, l.scooters, l.stock))
            .collect::<Vec<_>>()
            .join(";"),
    )?;
    f.add("K-L1.profit", list(factory::PROFIT))?;
    let corners = factory::corners();
    f.pin(
        "K-L2.corners",
        corners.iter().copied().map(point).collect::<Vec<_>>().join(";"),
        "0,0;0,40;40,20;50,0",
        "the order's corrected corners",
    )?;
    f.add("K-L2.profits", list(corners.iter().map(|c| factory::profit(c.0, c.1))))?;
    let best = corners
        .iter()
        .copied()
        .max_by(|a, b| {
            factory::profit(a.0, a.1)
                .to_f64()
                .total_cmp(&factory::profit(b.0, b.1).to_f64())
        })
        .ok_or("no corner")?;
    f.pin("K-L2.best", point(best), "40,20", "the owner's infographic, confirmed")?;
    f.pin(
        "K-L2.best_profit",
        factory::profit(best.0, best.1),
        "460",
        "8 x 40 + 7 x 20",
    )?;
    f.add("K-L2.tight", factory::tight(best.0, best.1).join(","))?;
    let stray = (
        Frac::whole(factory::NOT_A_CORNER[0]),
        Frac::whole(factory::NOT_A_CORNER[1]),
    );
    f.add("K-L2.not_a_corner", point(stray))?;
    f.add("K-L2.not_a_corner_allowed", factory::allowed(stray.0, stray.1))?;
    f.add(
        "K-L2.not_a_corner_breaks",
        factory::LIMITS
            .iter()
            .filter(|l| {
                (Frac::whole(l.stock) - Frac::whole(l.bicycles) * stray.0 - Frac::whole(l.scooters) * stray.1)
                    .is_negative()
            })
            .map(|l| l.name)
            .collect::<Vec<_>>()
            .join(","),
    )?;
    Ok(())
}

/// K-Q: each quiz's countdown and its answer, computed.
fn quizzes(f: &mut Facts) -> Result<(), String> {
    let t = Truss::pratt();
    let softest = t.patterns().vectors.swap_remove(0);
    let (hz, heavy) = (t.hertz(truss::DECK_KG), t.hertz(shown::HEAVY_DECK_KG));
    let mut worst = ("", 0);
    for room in [rooms::NARROW, rooms::STUDY, rooms::CUBE] {
        let shared = room.sharing_a_tone(200);
        if shared > worst.1 {
            worst = (room.name, shared);
        }
    }
    let pick = |yes: bool, a: &str, b: &str| (if yes { a } else { b }).to_owned();
    for q in QUIZZES {
        let answer = match q.line {
            "02_6" => wholes(&carts::pushes([1.0, 1.0]))?,
            "04_5" => {
                let u = carts::moves([0.0, 1.0]);
                format!(
                    "{},{}",
                    Frac::new((u[0] * 3.0).round() as i128, 3),
                    Frac::new((u[1] * 3.0).round() as i128, 3)
                )
            }
            "06_3" => pick(t.deck_sign_changes(&softest) == 0, "sag", "alternating"),
            "07_4" => pick(heavy[0] < hz[0], "slower", "faster"),
            "09_4" => fixed(rooms::SOUND_M_PER_S / (2.0 * rooms::BATHROOM.sides_m()[0]), 1)?,
            "11_3" => worst.0.to_owned(),
            "12_2" => list(economy::thrifty().eats(pair(shown::EVEN_MIX))),
            other => return Err(format!("quiz {other} has no answer in facts.rs")),
        };
        f.add(&format!("K-Q.{}.seconds", q.line), q.seconds)?;
        f.add(&format!("K-Q.{}.answer", q.line), answer)?;
    }
    Ok(())
}

/// Whether a 2 by 2 table with no negative entry has an eigenvector with no
/// negative entry for its largest eigenvalue. The eigenvalues are real (the
/// discriminant is (a − d)² + 4bc ≥ 0); the eigenvector is (b, λ − a), or
/// (λ − d, c), or, when both are zero, any vector: then the table is λ times
/// the identity on that line, and (1, 0) or (0, 1) serves.
fn nonnegative_pattern_for_largest(m: &[[f64; 2]; 2]) -> bool {
    let ([a, b], [c, d]) = (m[0], m[1]);
    let lambda = (a + d + ((a - d).powi(2) + 4.0 * b * c).sqrt()) / 2.0;
    let tiny = 1e-12;
    let candidates = [[b, lambda - a], [lambda - d, c], [1.0, 0.0], [0.0, 1.0]];
    candidates.iter().any(|v| {
        let big = v[0].abs().max(v[1].abs());
        let image = [a * v[0] + b * v[1], c * v[0] + d * v[1]];
        big > tiny
            && (image[0] - lambda * v[0]).abs() < 1e-9 * big.max(1.0)
            && (image[1] - lambda * v[1]).abs() < 1e-9 * big.max(1.0)
            && ((v[0] >= -tiny && v[1] >= -tiny) || (v[0] <= tiny && v[1] <= tiny))
    })
}

/// The largest entry, in size, of a matrix given as a slice.
fn worst_entry(xs: impl IntoIterator<Item = f64>) -> f64 {
    xs.into_iter().fold(0.0f64, |s, x| s.max(x.abs()))
}

/// K-X: the edges of the always and never sentences.
fn edges(f: &mut Facts) -> Result<(), String> {
    // K-X1 (15_2): "a symmetric table always has a full set of own patterns,
    // at right angles": 1,000 seeded symmetric tables of sizes 2 to 6.
    let mut data = Data::new(1301);
    let (mut checked, mut holds) = (0, 0);
    for i in 0..1000usize {
        let n = 2 + i % 5;
        let mut a = Mat::zeros(n);
        for r in 0..n {
            for c in r..n {
                let x = data.uniform(-1.0, 1.0);
                a.a[r * n + c] = x;
                a.a[c * n + r] = x;
            }
        }
        let e = jacobi_eigh(&a);
        // Q Λ Qᵀ gives the table back, and QᵀQ is the identity
        let mut rebuilt = 0.0f64;
        let mut square = 0.0f64;
        for r in 0..n {
            for c in 0..n {
                let entry: f64 = (0..n).map(|k| e.values[k] * e.vectors[k][r] * e.vectors[k][c]).sum();
                rebuilt = rebuilt.max((entry - a.at(r, c)).abs());
                let want = if r == c { 1.0 } else { 0.0 };
                square = square.max((dot(&e.vectors[r], &e.vectors[c]) - want).abs());
            }
        }
        checked += 1;
        if rebuilt < 1e-10 && square < 1e-10 && e.values.len() == n {
            holds += 1;
        }
    }
    f.add("K-X1.symmetric.checked", checked)?;
    f.add("K-X1.symmetric.hold", holds)?;
    // its edge: a repeated eigenvalue (R1), where the patterns are not
    // unique but can still be chosen at right angles
    let repeated = [
        Mat::from_rows(&[&[2.0, 0.0], &[0.0, 2.0]]),
        Mat::from_rows(&[&[2.0, 1.0, 1.0], &[1.0, 2.0, 1.0], &[1.0, 1.0, 2.0]]),
    ];
    let mut repeated_hold = 0;
    for a in &repeated {
        let e = jacobi_eigh(a);
        let n = a.n;
        let mut ok = true;
        for r in 0..n {
            for c in 0..n {
                let entry: f64 = (0..n).map(|k| e.values[k] * e.vectors[k][r] * e.vectors[k][c]).sum();
                let want = if r == c { 1.0 } else { 0.0 };
                ok &= (entry - a.at(r, c)).abs() < 1e-12 && (dot(&e.vectors[r], &e.vectors[c]) - want).abs() < 1e-12;
            }
        }
        repeated_hold += usize::from(ok);
    }
    f.add("K-X1.repeated.checked", repeated.len())?;
    f.add("K-X1.repeated.hold", repeated_hold)?;

    // K-X2 (03_4, 15_5): how many directions a 2 by 2 rule sends back along
    // themselves: the carts two, a shear one, a quarter turn none.
    f.add("K-X2.carts", carts::own_directions(&carts::table()))?;
    f.add(
        "K-X2.shear",
        carts::own_directions(&Mat::from_rows(&[&[1.0, 1.0], &[0.0, 1.0]])),
    )?;
    f.add(
        "K-X2.quarter_turn",
        carts::own_directions(&Mat::from_rows(&[&[0.0, -1.0], &[1.0, 0.0]])),
    )?;
    f.add(
        "K-X2.economy",
        carts::own_directions(&Mat::from_rows(&[&[0.4, 0.3], &[0.2, 0.5]])),
    )?;

    // K-X3: the truss's table is positive only once it is supported.
    let t = Truss::pratt();
    let loose = jacobi_eigh(&t.stiffness_unsupported());
    let top = worst_entry(loose.values.iter().copied());
    f.pin(
        "K-X3.unsupported_zero",
        loose.values.iter().filter(|v| v.abs() < 1e-9 * top).count(),
        "3",
        "a flat body can slide two ways and turn",
    )?;
    let e = t.patterns();
    f.add("K-X3.supported_all_positive", e.values.iter().all(|v| *v > 0.0))?;

    // K-X4 (06_4): all 21 patterns rebuild the direct answer.
    let load = t.deck_load();
    let direct = t.moves(&load);
    let rebuilt = t.moves_from_patterns(&e, &load, e.values.len());
    let off = worst_entry(direct.iter().zip(&rebuilt).map(|(a, b)| a - b));
    f.add("K-X4.rebuilt", off < 1e-9 * largest(&direct))?;

    // K-X5 (14_3): "the best plan is found at a corner": every allowed plan
    // of whole bicycles and scooters, tried.
    let mut plans = 0;
    for x in 0..=100 {
        for y in 0..=100 {
            if factory::allowed(Frac::whole(x), Frac::whole(y)) {
                plans += 1;
            }
        }
    }
    let (best_plan, best_profit) = factory::best_whole_plan();
    let corner_best = factory::corners()
        .iter()
        .map(|c| factory::profit(c.0, c.1))
        .fold(Frac::whole(0), |s, p| if (p - s).num > 0 { p } else { s });
    f.add("K-X5.plans", plans)?;
    f.add("K-X5.best_plan", list(best_plan))?;
    f.add("K-X5.best_is_a_corner", Frac::whole(best_profit) == corner_best)?;

    // K-X6 (12_5): "a table with no negative numbers always has an own
    // pattern with none, and it carries the largest eigenvalue": 1,000
    // seeded tables of positive entries.
    let mut data = Data::new(1907);
    let (mut checked, mut holds) = (0, 0);
    for _ in 0..1000 {
        let m = [
            [data.uniform(0.05, 1.0), data.uniform(0.05, 1.0)],
            [data.uniform(0.05, 1.0), data.uniform(0.05, 1.0)],
        ];
        let [(big, v), (small, w)] = economy::positive_eigen(m);
        checked += 1;
        if big > 0.0 && big > small.abs() && v[1] > 0.0 && w[1] < 0.0 {
            holds += 1;
        }
    }
    f.add("K-X6.positive.checked", checked)?;
    f.add("K-X6.positive.hold", holds)?;
    // its edges: zeros allowed (R1). 1,000 seeded tables whose entries are
    // zero three times in ten, and four tables picked by hand: a shear, a
    // swap, a table that sends everything to zero, and two separate goods.
    let mut data = Data::new(1912);
    let mut tables: Vec<[[f64; 2]; 2]> = vec![
        [[1.0, 1.0], [0.0, 1.0]],
        [[0.0, 1.0], [1.0, 0.0]],
        [[0.0, 1.0], [0.0, 0.0]],
        [[1.0, 0.0], [0.0, 2.0]],
    ];
    for _ in 0..1000 {
        let mut entry = || {
            if data.unit() < 0.3 {
                0.0
            } else {
                data.uniform(0.05, 1.0)
            }
        };
        tables.push([[entry(), entry()], [entry(), entry()]]);
    }
    let holds = tables.iter().filter(|m| nonnegative_pattern_for_largest(m)).count();
    f.add("K-X6.zeros.checked", tables.len())?;
    f.add("K-X6.zeros.hold", holds)?;

    // K-X7 (13_6): "below one, every demand can be met; at one or more, it
    // cannot keep up": the video's table scaled by 1/20, 2/20, …, 40/20, and
    // by 10/7, which puts the largest eigenvalue at one exactly.
    let c = economy::thrifty();
    let mut factors: Vec<Frac> = (1..=40).map(|k| Frac::new(k, 20)).collect();
    factors.push(Frac::new(10, 7));
    let (mut agree, mut feed) = (0, 0);
    let mut at_one = None;
    for factor in &factors {
        let scaled = c.scaled(*factor);
        let largest = scaled.eigenvalues().ok_or("a scaled table lost its fractions")?[0];
        let below_one = (Frac::whole(1) - largest).num > 0;
        if scaled.feeds_itself() == below_one && scaled.hawkins_simon() == below_one {
            agree += 1;
        }
        if scaled.feeds_itself() {
            feed += 1;
        }
        if largest == Frac::whole(1) {
            at_one = Some(scaled.feeds_itself());
        }
    }
    f.add("K-X7.scaled.checked", factors.len())?;
    f.add("K-X7.scaled.agree", agree)?;
    f.add("K-X7.scaled.feed_themselves", feed)?;
    f.add("K-X7.at_one_feeds_itself", at_one.ok_or("no table at one")?)?;
    Ok(())
}

/// K-S: years and counts quoted from the sources.
fn sources(f: &mut Facts) -> Result<(), String> {
    use cited::*;
    f.add("K-S.helmholtz_year", HELMHOLTZ_YEAR)?;
    f.add("K-S.hilbert_year", HILBERT_YEAR)?;
    f.pin(
        "K-S.eigen_gap_years",
        HILBERT_YEAR - HELMHOLTZ_YEAR,
        "41",
        "1904 less 1863",
    )?;
    f.add("K-S.perron_year", PERRON_YEAR)?;
    f.add("K-S.frobenius_year", FROBENIUS_YEAR)?;
    f.add("K-S.leontief.paper_year", LEONTIEF_PAPER_YEAR)?;
    f.add("K-S.leontief.book_year", LEONTIEF_BOOK_YEAR)?;
    f.add("K-S.leontief.sectors", LEONTIEF_SECTORS)?;
    f.add("K-S.leontief.producing", LEONTIEF_PRODUCING)?;
    f.add("K-S.leontief.prize_year", LEONTIEF_PRIZE_YEAR)?;
    f.add("K-S.hawkins_simon_year", HAWKINS_SIMON_YEAR)?;
    f.add("K-S.sabine.opened", SYMPHONY_HALL_OPENED)?;
    f.add("K-S.schroeder_year", SCHROEDER_YEAR)?;
    f.add("K-S.dantzig_year", DANTZIG_YEAR)?;
    f.add("K-S.lanczos_year", LANCZOS_YEAR)?;
    f.add("K-S.brin_page_year", BRIN_PAGE_YEAR)?;
    f.add("K-S.frankensim.commit", FRANKENSIM_COMMIT)?;
    f.add("K-S.frankensim.grid_side", FRANKENSIM_GRID_SIDE)?;
    f.pin(
        "K-S.frankensim.grid_unknowns",
        FRANKENSIM_GRID_SIDE * FRANKENSIM_GRID_SIDE,
        "100489",
        "crates/fs-modal/tests/modal_casebook.rs: 317 squared",
    )?;
    f.add("K-S.frankensim.chain", FRANKENSIM_CHAIN)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_facts_build_and_every_pin_holds() {
        let f = facts().unwrap();
        f.self_check().unwrap();
        assert!(f.len() > 200, "{} facts", f.len());
    }

    #[test]
    fn fixed_rounds_refuses_edges_and_never_prints_minus_zero() {
        assert_eq!(fixed(17.9616, 2).unwrap(), "17.96");
        assert_eq!(fixed(-449.999_999_9, 0).unwrap(), "-450");
        assert_eq!(fixed(-1e-13, 0).unwrap(), "0");
        assert_eq!(fixed(-1e-13, 1).unwrap(), "0.0");
        assert!(fixed(0.125, 2).is_err());
        assert_eq!(decimal(Frac::new(84, 5)).unwrap(), "16.8");
        assert_eq!(decimal(Frac::new(1029, 125)).unwrap(), "8.232");
        assert!(decimal(Frac::new(1, 3)).is_err());
        assert_eq!(metres(25), "2.5");
        assert_eq!(metres(450), "45");
    }

    /// Edge of 01_5 ("about once a second"): three of the bridge's four
    /// measured sideways modes are within a fifth of a hertz of one, and the
    /// fourth is the one chapter 08 calls "near half".
    #[test]
    fn about_once_a_second_covers_three_of_the_four_modes() {
        let f = facts().unwrap();
        assert_eq!(f.get("K-B2.lateral_near_one"), Some("3"));
        assert_eq!(f.get("K-B2.lateral_lowest_hz"), Some("0.5"));
        assert_eq!(f.get("K-B2.walker_lateral_hz"), Some("1.0"));
    }

    /// Edges of the always sentences, each one checked on every case it was
    /// tried on (the counts are facts; here they must be complete).
    #[test]
    fn every_always_sentence_holds_on_everything_it_was_tried_on() {
        let f = facts().unwrap();
        assert_eq!(f.get("K-X1.symmetric.hold"), f.get("K-X1.symmetric.checked"));
        assert_eq!(f.get("K-X6.positive.hold"), f.get("K-X6.positive.checked"));
        assert_eq!(f.get("K-X6.zeros.hold"), f.get("K-X6.zeros.checked"));
        assert_eq!(f.get("K-X1.repeated.hold"), f.get("K-X1.repeated.checked"));
        assert_eq!(f.get("K-X7.scaled.agree"), f.get("K-X7.scaled.checked"));
        assert_eq!(f.get("K-X7.at_one_feeds_itself"), Some("false"));
        assert_eq!(f.get("K-X5.best_is_a_corner"), Some("true"));
        assert_eq!(f.get("K-X4.rebuilt"), Some("true"));
        assert_eq!(f.get("K-X3.supported_all_positive"), Some("true"));
        // the three rules of the arrow plane: two directions, one, none
        assert_eq!(
            (f.get("K-X2.carts"), f.get("K-X2.shear"), f.get("K-X2.quarter_turn")),
            (Some("2"), Some("1"), Some("0"))
        );
        assert_eq!(f.get("K-X2.economy"), Some("2"));
    }

    /// 09_5's pitch line crosses nine tones of the bathroom between 60 and
    /// 150 Hz. Five fit one side of the room (a whole number of half-waves
    /// along one length) and four need two or three sides, so the line never
    /// reads as the room's whole list (decisions D-53); the first is the
    /// quiz's answer.
    #[test]
    fn the_bathroom_has_nine_tones_to_150_hz_and_five_fit_one_side() {
        let f = facts().unwrap();
        let tones: Vec<f64> = f
            .get("K-R2.tones_to_150_hz")
            .unwrap()
            .split(',')
            .map(|s| s.parse().unwrap())
            .collect();
        assert_eq!(tones.len(), 9);
        assert!(tones.windows(2).all(|w| w[0] < w[1]), "ascending, none twice");
        let modes = rooms::BATHROOM.modes_below(150);
        assert_eq!(modes.len(), 9);
        let one_side = modes
            .iter()
            .filter(|(n, _)| n.iter().filter(|&&k| k > 0).count() == 1)
            .map(|(_, hz)| fixed(*hz, 1).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(one_side, ["68.6", "71.5", "107.2", "137.2", "142.9"]);
        assert!(modes.iter().all(|(_, hz)| *hz < 149.0), "none is near 150 Hz");
        assert_eq!(f.get("K-Q.09_4.answer"), Some("68.6"));
        assert_eq!(tones[0], 68.6);
    }

    /// 07_5 sets 4.32 Hz (twice the deck, the steel as it is) against 4.18
    /// (5.91 divided by the root of two, as if the steel had doubled too).
    #[test]
    fn the_heavier_deck_is_four_point_three_two_and_not_four_point_one_eight() {
        let f = facts().unwrap();
        assert_eq!(f.get("K-V1.hz_2dp").unwrap().split(',').next(), Some("5.91"));
        assert_eq!(f.get("K-V2.hz_2dp"), Some("4.32"));
        assert_eq!(f.get("K-V2.naive_hz_2dp"), Some("4.18"));
        // slower than the light deck, and by less than the naive halving of the mass would say
        let (light, heavy, naive) = (5.91f64, 4.32f64, 4.18f64);
        assert!(heavy < light && heavy > naive);
    }

    /// What 06_7's card and 07_1's lower-third say, as numbers: a sharp load
    /// needs far more than the softest pattern, and only the lowest few modes
    /// are close to K's own patterns.
    #[test]
    fn the_sharp_load_and_the_overlaps_say_what_the_storyboard_says() {
        let f = facts().unwrap();
        for (key, want) in [
            ("K-P2.lone_pct.4m", "42"),
            ("K-P2.lone_pct.8m", "76"),
            ("K-P2.lone_pct.12m", "86"),
            ("K-P2.lone_within_2p5.12m", "10"),
        ] {
            assert_eq!(f.get(key), Some(want), "{key}");
        }
        let overlaps: Vec<&str> = (1..=8)
            .map(|i| f.get(&format!("K-V1.overlap.{i}")).expect("an overlap"))
            .collect();
        assert_eq!(overlaps, ["1.00", "0.99", "0.98", "0.86", "0.85", "0.79", "0.10", "0.02"]);
        // the lowest three are the "shift a little" of 07_1; the rest are not
        let c: Vec<f64> = overlaps.iter().map(|s| s.parse().unwrap()).collect();
        assert!(c[..3].iter().all(|&x| x >= 0.97));
        assert!(c[6] < 0.2);
    }

    /// Every quiz has its answer, and the answers the script speaks.
    #[test]
    fn the_quizzes_have_the_answers_the_script_gives() {
        let f = facts().unwrap();
        let want = [
            ("02_6", "1,1"),
            ("04_5", "1/3,2/3"),
            ("06_3", "sag"),
            ("07_4", "slower"),
            ("09_4", "68.6"),
            ("11_3", "cube"),
            ("12_2", "7,7"),
        ];
        assert_eq!(want.len(), QUIZZES.len());
        for (line, answer) in want {
            assert_eq!(f.get(&format!("K-Q.{line}.answer")), Some(answer), "{line}");
        }
    }
}
