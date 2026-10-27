//! Chapter 08, the formula, from the triangles. Chapter 05's stretch and its
//! two triangles stay at the left, on their axes, both in minutes; a panel
//! at the right turns their numbers into letters:
//! - 08_1: each triangle is half the square on its gap (a faint mirror
//!   completes the square): "2 min → ½ × (2 min)² = 2 min²", the same for
//!   ten, then "g → ½ × g²", the numbers carried down into the letters. An
//!   area is in square minutes; at one passenger a minute it is minutes of
//!   waiting (05_5's bridge, kept on screen).
//! - 08_2: the areas added over the gaps added: "52 min² ÷ 12 min = 4⅓
//!   min", a wait; then the same sum in letters.
//! - 08_3: top and bottom divided by the number of gaps: "average wait =
//!   average of g² ÷ (2 × average gap)", with our numbers under it.
//! - 08_4: the squares themselves. Each triangle and its mirror fill the
//!   square on its gap; the average square, 52, against the average gap's
//!   square, 36; the 16 left over is a corner square on the long gap, and
//!   each gap is 4 from 6. Then, held on screen without words, the square
//!   argument: each gap's square is the average gap's square, two strips and
//!   the distance's square; the strips cancel over the gaps, so the average
//!   square is the average gap squared plus the spread, which is named only
//!   now.
//! - 08_5: the formula with our numbers first, "½ × 6 min = 3 min" and "16
//!   min² ÷ (2 × 6 min) = 1⅓ min", boxed as "3:00 + 1:20 = 4:20", then in
//!   letters.
//! - 08_6: the stretch gives way to the sign and you; the 1:20 flies to you,
//!   and the sign's "every 6 minutes" dims.
//! - 08_7: ever less than half the average gap? (a prediction; nothing on
//!   screen answers it.)
//! - 08_8: never: the spread is an average of squares. The conditions on
//!   turning up, the bound, equality only for even gaps, the finite check
//!   labelled as a check and not the proof, and K05.
//!
//! Every number on screen is read from the facts, and [`formula`] checks
//! each against the model before anything is drawn. Each row is one text
//! (one baseline); a number carried into a row is a fresh copy of it that
//! flies to its measured place there and goes before the row comes in.

use explainer_kit::props::precisely;
use explainer_kit::{Facts, Kit, Palette, fade_all, hex, rect, thousands, v};
use fmn::prelude::*;

use super::draw::{
    fade_in_all, fade_in_lagged, fade_out_all, fade_to, fail, flash_all, flat, fly, line_style, min_s, now, pause,
    read_s, say, stand, think, wait_until, you,
};
use super::timetable::{self, Gap};
use super::twelve_minutes::{self, AXIS_Y, MIN, ORIGIN_X, at, frame, mixed};
use crate::facts::{self, csv};
use crate::model::{self, OFFSET, Q};
use crate::palette;

pub struct TheFormula {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are read from the facts)
const AREA: &str = "min²";
const SQUARED: &str = "²";
const ARROW: &str = "→";
const TIMES: &str = "×";
const DIVIDE: &str = "÷";
const PLUS: &str = "+";
const MINUS: &str = "−";
const EQUALS: &str = "=";
const AT_LEAST: &str = "≥";
const G: &str = "g";
const G_NAME: &str = "(g: a gap's length, min)";
const WAIT_NAME: &str = "wait (min)";
const MOMENT_NAME: &str = "moment you turn up (min)";
const BRIDGE: &str = "at one passenger a minute, each min² of area is a minute of someone's waiting";
const AVERAGE_WAIT: &str = "average wait";
const SUM_OF: &str = "sum of";
const AVERAGE_OF: &str = "average of";
const AVERAGE_GAP: &str = "average gap";
const BY_GAPS: &str = "top and bottom";
const GAP_COUNT: &str = "the number of gaps";
const TOP: &str = "top:";
const BOTTOM: &str = "bottom:";
const EACH_GAP: &str = "each gap is";
const FROM: &str = "from";
const AVERAGE: &str = "average:";
const STRIPS: &str = "(the strips cancel)";
const SPREAD: &str = "spread";
const SPREAD_NAME: &str = "spread: the average squared distance from the average gap";
const ANY_GAPS: &str = "any gaps:";
const A_MORNING: &str = "a morning";
const ASK: &str = "ever less than half the average gap?";
const NEVER: &str = "spread: an average of squares, never negative";
const WHO: &str = "for passengers who turn up evenly in time, independently of the buses:";
const EXACTLY: &str = "exactly half only when every gap is the same";
/// The finite check (F-X1), the draft's card in pieces around its numbers;
/// its head says what it is.
const CHECK_HEAD: &str = "a finite check, not the proof:";
const CHECK_UP_TO: &str = "every choice of up to";
const CHECK_GAPS: &str = "gaps (repeats allowed, order ignored),";
const CHECK_EACH: &str = "each";
const CHECK_TO: &str = "to";
const CHECK_CHOICES: &str = "choices,";
const CHECK_BELOW: &str = "below half;";
const CHECK_EQUAL: &str = "equal only when all the gaps are the same";
/// The exact version of 08_8's claim (K05), the draft's word for word. It
/// is a citation and a formula, so it carries its own digits ("2 E[G]",
/// "§2.13"); it is the one fixed text exempt from reading its numbers from
/// the facts.
const K05: &str = "for passengers turning up uniformly in time, independently of the buses: average wait = E[G²] ÷ \
                   (2 E[G]) = E[G]/2 + Var(G) ÷ (2 E[G]); equal to E[G]/2 exactly when Var(G) = 0. Larson & Odoni, \
                   Urban Operations Research, §2.13, 'random incidence'";

// ------------------------------------------------------------------ layout

/// The wait's axis and names, as chapter 05 draws them (its repaired top
/// label: the axis 0.05 past the frame, the name 0.2 above); the moment's
/// name moves under the stretch, centred, so the panel's column runs clear
/// down the right.
const Y_OVERHANG: f64 = 0.05;
const Y_TICK_W: f64 = 0.1;
const Y_LABEL_DX: f64 = 0.3;
const WAIT_NAME_DY: f64 = 0.2;
const MOMENT_NAME_Y: f64 = -2.8;
const SMALL: f64 = 18.0;
/// The triangles' fill and outline (chapter 05's); the mirror halves'
/// faint outline; a pulse brightens a fill and lets it settle.
const TRI_FILL: f64 = 0.35;
const TRI_W: f64 = 3.0;
const MIRROR_W: f64 = 2.0;
const PULSE_FILL: f64 = 0.7;
const PULSE_RT: f64 = 0.8;
/// 08_4's squares: the average gap's square outlined, the corner square
/// filled, the strips outlined faint; the distance segments just inside
/// the column left of the long triangle, a little off the lines they
/// follow.
const SQUARE_W: f64 = 3.0;
const CORNER_FILL: f64 = 0.5;
const STRIP_W: f64 = 2.0;
const DISTANCE_W: f64 = 5.0;
const DISTANCE_DX: f64 = 0.1;
const DISTANCE_LABEL_X: f64 = -5.65;
const PLOT_LABEL: f64 = 16.0;
const ABOVE_SQUARE: f64 = 0.25;
/// 08_1's "g" on the long triangle: left of its upright side, and inside
/// it just over its base.
const G_LEG_DX: f64 = 0.25;
const G_BASE_DY: f64 = 0.25;
/// A gap's length under its bracket, as `twelve_minutes::bracket` prints
/// it: a copy carried from it is made at its size.
const BRACKET_LABEL_SIZE: f64 = 20.0;
/// The panel: rows left-aligned in a column from the top, one text each.
const PANEL_LEFT: f64 = 0.4;
const PANEL_TOP: f64 = 3.5;
const ROW_H: f64 = 0.55;
const PANEL_SIZE: f64 = 20.0;
const NOTE_SIZE: f64 = 16.0;
const WHO_SIZE: f64 = 18.0;
const BRIDGE_AT: Vec3 = [PANEL_LEFT, -3.45, 0.0];
/// A row's space between pieces, in units a point (as the layout tests
/// estimate a character).
const SPACE: f64 = 0.0045;
/// 08_5's box around the verdict row.
const BOX_PAD: [f64; 2] = [0.2, 0.15];
const BOX_W: f64 = 2.0;
/// 08_6: the sign at chapter 01's place, you right of it at the stop, the
/// 1:20 a morning under you.
const YOU_AT: Vec3 = [-1.5, -1.55, 0.0];
const MORNING_DY: f64 = -0.55;
const MORNING_SIZE: f64 = 18.0;
const DIM_TO: f64 = 0.35;
/// 08_7: the question over the sign, the think ring over it.
const ASK_AT: Vec3 = [-3.4, 1.0, 0.0];
const ASK_SIZE: f64 = 24.0;
const THINK_AT: Vec3 = [-3.4, 2.7, 0.0];
/// 08_8's check card, top left: its outline and its lines.
const CHECK_DL: Vec3 = [-6.85, 0.75, 0.0];
const CHECK_W: f64 = 5.9;
const CHECK_H: f64 = 1.85;
const CHECK_X: f64 = -6.7;
const CHECK_YS: [f64; 4] = [2.3, 1.85, 1.45, 1.05];
const CHECK_SIZES: [f64; 2] = [18.0, 16.0];
/// Emphasis: an outline around a text, faded in, held, faded out.
const HALO_PAD: f64 = 0.08;
const HALO_W: f64 = 3.0;
const HALO_RT: f64 = 0.9;
/// Flights and fades.
const CARRY_RT: f64 = 0.7;
const MOVE_RT: f64 = 0.8;
/// Silent inspection after a derivation's last step: the letters of 08_3,
/// the square argument of 08_4 and the letters of 08_5.
const LETTERS_LOOK_S: f64 = 2.0;
const SQUARES_LOOK_S: f64 = 4.0;
const FORMULA_LOOK_S: f64 = 2.0;

/// The panel's row `i`, from the top.
pub fn row_y(i: usize) -> f64 {
    PANEL_TOP - ROW_H * i as f64
}

/// The spread's color, violet (storyboard, "Color roles"). `palette.rs` adds
/// a role when its first chapter draws it; until that file is open again,
/// the spread's role lives here, as the short gap's lives in
/// `timetable.rs`.
pub fn spread_color() -> Srgb {
    hex("#C792EA")
}

// ------------------------------------------------------------- the numbers

/// What chapter 08 prints, read from the facts and checked against the
/// model: the stretch's two gaps, their triangles and squares, the averages,
/// the spread and the formula, the commuter's extra time, and the finite
/// check.
pub struct Formula {
    /// The two gaps, short then long, and their triangles' areas.
    pub gaps: [String; 2],
    pub areas: [String; 2],
    pub total_area: String,
    /// The time the triangles cover (the gaps added), and the wait.
    pub period: String,
    pub mean: String,
    /// A triangle is this share of its square; the formula's "2 ×" is its
    /// reciprocal.
    pub half: String,
    pub twice: String,
    pub gap_count: String,
    /// The squares, their average, the average gap and its square, the
    /// spread and each gap's distance from the average.
    pub squares: [String; 2],
    pub mean_square: String,
    pub mean_gap: String,
    pub mean_gap_squared: String,
    pub spread: String,
    pub distance: String,
    /// The formula's two terms: half the average gap, and the spread's
    /// share, in minutes and as clocks; the wait as a clock; your extra
    /// time spelt out.
    pub half_mean_gap: String,
    pub penalty: String,
    pub half_clock: String,
    pub penalty_clock: String,
    pub wait_clock: String,
    pub extra: String,
    /// The finite check: up to `max_count` gaps, each `lowest` to `max_gap`
    /// minutes, `cases` choices (with a thousands comma), `violations`
    /// below half.
    pub max_count: String,
    pub lowest: String,
    pub max_gap: String,
    pub cases: String,
    pub violations: String,
}

/// A fact's printed fraction ("4/3", "52") as a number.
fn parse_q(key: &str, printed: &str) -> Result<Q, String> {
    let int = |s: &str| s.parse::<i64>().map_err(|e| format!("{key} = {printed}: {e}"));
    match printed.split_once('/') {
        Some((n, d)) => Ok(Q::new(int(n)?, int(d)?)),
        None => Ok(Q::int(int(printed)?)),
    }
}

/// n choose k, small.
fn binomial(n: u64, k: u64) -> u64 {
    (0..k).fold(1, |acc, i| acc * (n - i) / (i + 1))
}

/// How many choices the finite check counts with gaps from `lowest` to
/// `max_gap` minutes: every multiset of 1 to `max_count` of them, less those
/// with no time at all (all gaps zero), which `model::bound_sweep` skips.
pub fn choices(lowest: i64, max_gap: i64, max_count: u64) -> u64 {
    let values = (max_gap - lowest + 1) as u64;
    let all = (1..=max_count).map(|j| binomial(values + j - 1, j)).sum::<u64>();
    if lowest == 0 { all - max_count } else { all }
}

/// The formula's numbers, refused when a fact disagrees with the model.
pub fn formula(f: &Facts) -> Result<Formula, String> {
    let n = twelve_minutes::numbers(f)?;
    let get = |key: &str| f.get(key).map(str::to_owned).ok_or_else(|| format!("no fact {key}"));
    // a fact that must be what the model has
    let drawn = |key: &str, value: String| -> Result<String, String> {
        let printed = get(key)?;
        if printed == value {
            Ok(printed)
        } else {
            Err(format!("{key} is {printed}, but the model has {value}"))
        }
    };
    let q = |key: &str| -> Result<Q, String> { parse_q(key, &get(key)?) };
    let gaps: Vec<i64> = twelve_minutes::gaps().iter().map(Gap::minutes).collect();
    let count = Q::int(gaps.len() as i64);
    let m = model::mean_gap(&gaps);

    // a triangle is half its square; the formula's 2 is that half's reciprocal
    let factor = q("F-F1.area_factor")?;
    if factor.num() != 1 {
        return Err(format!("F-F1.area_factor {factor} is not one over a whole number"));
    }
    for &g in &gaps {
        if model::triangle_area(g) != factor * Q::int(g * g) {
            return Err(format!("the {g}-minute triangle is not {factor} of its square"));
        }
    }
    drawn("F-X1.bound_factor", factor.to_string())?;
    let twice = Q::int(factor.den());

    // the areas over the time they cover: a wait
    let period: i64 = gaps.iter().sum();
    let period_s = drawn("F-T1.period", period.to_string())?;
    let total_area = q("F-A1.total_area")?;
    let wait = model::mean_wait(&gaps);
    if total_area / Q::int(period) != wait {
        return Err("the triangles over the time they cover are not the average wait".into());
    }

    // the squares, their average, and the average gap's square
    let squares: Vec<Q> = gaps.iter().map(|&g| Q::int(g * g)).collect();
    drawn("F-F1.squares", csv(&squares))?;
    let mean_square = squares.iter().copied().sum::<Q>() / count;
    let mean_square_s = drawn("F-F1.mean_square", mean_square.to_string())?;
    let mean_gap_squared = drawn("F-F1.mean_gap_squared", (m * m).to_string())?;

    // the spread, and the square argument: each gap's square is the average
    // gap's square, two strips (the average gap times the distance, signed)
    // and the distance's square; the strips cancel over the gaps
    let spread = model::spread(&gaps);
    let spread_s = drawn("F-F1.spread", spread.to_string())?;
    let mut strips = Q::int(0);
    for &g in &gaps {
        let d = Q::int(g) - m;
        if Q::int(g * g) != m * m + m * d + m * d + d * d {
            return Err(format!(
                "the {g}-minute square is not the average's square, two strips and a corner"
            ));
        }
        strips = strips + m * d + m * d;
    }
    if strips != Q::int(0) {
        return Err(format!("the strips add to {strips}, not nothing"));
    }
    if mean_square != m * m + spread {
        return Err("the average square is not the average gap squared plus the spread".into());
    }
    let distance = q("F-F1.distance")?;
    for &g in &gaps {
        let d = Q::int(g) - m;
        if d != distance && d != Q::int(0) - distance {
            return Err(format!("the {g}-minute gap is not {distance} from the average"));
        }
    }
    if distance * distance != spread {
        return Err("the spread is not the distance's square".into());
    }

    // the formula, three ways, and the facts' wait
    let half_mean_gap = q("F-T1.half_mean_gap")?;
    if half_mean_gap != m / twice {
        return Err("F-T1.half_mean_gap is not the average gap over the formula's 2".into());
    }
    let penalty = spread / (twice * m);
    let penalty_s = drawn("F-F1.penalty", penalty.to_string())?;
    for (what, value) in [
        (
            "the average square over twice the average gap",
            mean_square / (twice * m),
        ),
        ("half the average gap plus the spread's share", half_mean_gap + penalty),
    ] {
        if value != wait {
            return Err(format!("{what} is {value}, not the average wait {wait}"));
        }
    }
    drawn("F-F1.wait_by_formula", wait.to_string())?;
    drawn("F-W1.mean_wait", wait.to_string())?;
    let penalty_clock = drawn("F-F1.penalty_clock", model::clock(penalty)?)?;
    drawn("F-W1.extra_clock", model::clock(wait - half_mean_gap)?)?;
    let half_clock = model::clock(half_mean_gap)?;
    // even gaps wait exactly half: the even timetable's wait is this clock
    drawn("F-O1.even_wait_clock", half_clock.clone())?;
    let wait_clock = drawn("F-W1.mean_wait_clock", model::clock(wait)?)?;

    // the facts are over the hour's ten gaps; the stretch's two have the
    // same averages
    let hour = model::hour_gaps(OFFSET);
    if model::mean_gap(&hour) != m || model::mean_square(&hour) != mean_square || model::spread(&hour) != spread {
        return Err("the stretch's two gaps do not average as the hour's gaps do".into());
    }

    // the finite check, its count recounted from its range
    let max_gap: i64 = get("F-X1.max_gap")?.parse().map_err(|e| format!("F-X1.max_gap: {e}"))?;
    let max_count: usize = get("F-X1.max_count")?
        .parse()
        .map_err(|e| format!("F-X1.max_count: {e}"))?;
    let sweep = model::bound_sweep(max_gap, max_count);
    let cases = drawn("F-X1.cases", sweep.cases.to_string())?;
    let violations = drawn("F-X1.violations", sweep.violations.to_string())?;
    drawn("F-X1.equalities", sweep.equalities.to_string())?;
    drawn(
        "F-X1.equal_iff_even",
        if sweep.equal_iff_even { "true" } else { "false" }.to_owned(),
    )?;
    if sweep.violations != 0 || !sweep.equal_iff_even {
        return Err("the finite check finds a wait below half, or equality without even gaps".into());
    }
    let lowest = (0..=max_gap)
        .find(|&lo| choices(lo, max_gap, max_count as u64) == sweep.cases)
        .ok_or("no lowest gap gives the finite check's count")?;

    Ok(Formula {
        gaps: [n.gap_minutes[0].clone(), n.gap_minutes[1].clone()],
        areas: [n.areas[0].clone(), n.areas[1].clone()],
        total_area: n.total_area,
        period: period_s,
        mean: n.mean,
        half: mixed(factor)?,
        twice: twice.to_string(),
        gap_count: n.gap_count,
        squares: [squares[0].to_string(), squares[1].to_string()],
        mean_square: mean_square_s,
        mean_gap: n.mean_gap,
        mean_gap_squared,
        spread: spread_s,
        distance: distance.to_string(),
        half_mean_gap: n.half_mean_gap,
        penalty: mixed(parse_q("F-F1.penalty", &penalty_s)?)?,
        half_clock,
        penalty_clock,
        wait_clock,
        extra: min_s(&get("F-W1.extra_clock")?).map_err(|e| e.to_string())?,
        max_count: max_count.to_string(),
        lowest: lowest.to_string(),
        max_gap: max_gap.to_string(),
        cases: thousands(cases.parse::<u64>().map_err(|e| format!("F-X1.cases: {e}"))?),
        violations,
    })
}

// -------------------------------------------------------------------- rows

/// A piece's color, by its role in the storyboard's table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Ink,
    Dim,
    Short,
    Long,
    Spread,
    Verdict,
}

pub fn role_color(role: Role) -> Srgb {
    match role {
        Role::Ink => Palette::ink(),
        Role::Dim => Palette::dim(),
        Role::Short => timetable::short_gap(),
        Role::Long => palette::long_gap(),
        Role::Spread => spread_color(),
        Role::Verdict => palette::verdict(),
    }
}

/// A row of the panel: its pieces, a space apart, in the row's role unless
/// a piece has its own. It is drawn as one text, so its pieces share a
/// baseline; a piece in its own role must occur once in the row, or its
/// color would spill onto the other occurrence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub role: Role,
    pub pieces: Vec<(String, Role)>,
}

impl Row {
    /// Fixed pieces, all in the row's role.
    pub fn new(role: Role, pieces: &[&str]) -> Row {
        Row {
            role,
            pieces: pieces.iter().map(|&p| (p.to_owned(), role)).collect(),
        }
    }

    /// Pieces made from the facts, all in the row's role.
    pub fn of(role: Role, pieces: Vec<String>) -> Row {
        Row {
            role,
            pieces: pieces.into_iter().map(|p| (p, role)).collect(),
        }
    }

    /// The same row with piece `i` in `role`.
    pub fn with(mut self, i: usize, role: Role) -> Row {
        self.pieces[i].1 = role;
        self
    }

    /// The row as it reads.
    pub fn text(&self) -> String {
        self.joined(self.pieces.len())
    }

    /// The first `k` pieces as they read.
    fn joined(&self, k: usize) -> String {
        self.pieces[..k]
            .iter()
            .map(|(p, _)| p.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// The pieces in a role other than the row's.
    pub fn accents(&self) -> Vec<(&str, Role)> {
        self.pieces
            .iter()
            .filter(|(_, r)| *r != self.role)
            .map(|(p, r)| (p.as_str(), *r))
            .collect()
    }
}

impl Formula {
    /// 08_1: a gap's triangle, half the square on it: "2 min → ½ × (2 min)²
    /// = 2 min²".
    pub fn area_row(&self, i: usize) -> Row {
        let g = &self.gaps[i];
        let role = if i == 0 { Role::Short } else { Role::Long };
        Row::of(
            role,
            vec![
                format!("{g} {MIN}"),
                ARROW.into(),
                format!("{} {TIMES}", self.half),
                format!("({g} {MIN}){SQUARED}"),
                EQUALS.into(),
                format!("{} {AREA}", self.areas[i]),
            ],
        )
    }

    /// 08_1: any gap: "g → ½ × g² (g: a gap's length, min)".
    pub fn letter_row(&self) -> Row {
        Row::of(
            Role::Ink,
            vec![
                G.into(),
                ARROW.into(),
                format!("{} {TIMES}", self.half),
                format!("{G}{SQUARED}"),
                G_NAME.into(),
            ],
        )
        .with(4, Role::Dim)
    }

    /// 08_2: "average wait = (2 + 50) min²", then "÷ (2 + 10) min" beside it,
    /// each as it is said.
    pub fn sum_rows(&self) -> [Row; 2] {
        let [a, b] = &self.areas;
        let [s, l] = &self.gaps;
        [
            Row::of(
                Role::Ink,
                vec![AVERAGE_WAIT.into(), EQUALS.into(), format!("({a} {PLUS} {b}) {AREA}")],
            ),
            Row::of(Role::Ink, vec![DIVIDE.into(), format!("({s} {PLUS} {l}) {MIN}")]),
        ]
    }

    /// 08_2: "= 52 min² ÷ 12 min = 4⅓ min", a wait.
    pub fn sum_result(&self) -> Row {
        Row::of(
            Role::Ink,
            vec![
                EQUALS.into(),
                format!("{} {AREA} {DIVIDE} {} {MIN}", self.total_area, self.period),
                EQUALS.into(),
                format!("{} {MIN}", self.mean),
            ],
        )
        .with(3, Role::Verdict)
    }

    /// 08_2: the same sum in letters.
    pub fn sum_letters(&self) -> Row {
        Row::of(
            Role::Ink,
            vec![
                AVERAGE_WAIT.into(),
                EQUALS.into(),
                format!("({SUM_OF} {} {G}{SQUARED})", self.half),
                DIVIDE.into(),
                format!("({SUM_OF} {G})"),
            ],
        )
    }

    /// 08_3: "top and bottom ÷ 2, the number of gaps".
    pub fn by_gaps(&self) -> Row {
        Row::of(
            Role::Dim,
            vec![
                BY_GAPS.into(),
                format!("{DIVIDE} {},", self.gap_count),
                GAP_COUNT.into(),
            ],
        )
    }

    /// 08_3: what the top and the bottom become.
    pub fn top_row(&self) -> Row {
        Row::of(
            Role::Ink,
            vec![
                TOP.into(),
                format!("({SUM_OF} {} {G}{SQUARED}) {DIVIDE} {}", self.half, self.gap_count),
                EQUALS.into(),
                format!("{} {TIMES} {AVERAGE_OF} {G}{SQUARED}", self.half),
            ],
        )
        .with(0, Role::Dim)
    }

    pub fn bottom_row(&self) -> Row {
        Row::of(
            Role::Ink,
            vec![
                BOTTOM.into(),
                format!("({SUM_OF} {G}) {DIVIDE} {}", self.gap_count),
                EQUALS.into(),
                AVERAGE_GAP.into(),
            ],
        )
        .with(0, Role::Dim)
    }

    /// 08_3: "average wait = average of g² ÷ (2 × average gap)", and our
    /// numbers under it.
    pub fn mean_letters(&self) -> Row {
        Row::of(
            Role::Ink,
            vec![
                AVERAGE_WAIT.into(),
                EQUALS.into(),
                format!("{AVERAGE_OF} {G}{SQUARED}"),
                DIVIDE.into(),
                format!("({} {TIMES} {AVERAGE_GAP})", self.twice),
            ],
        )
    }

    pub fn mean_numbers(&self) -> Row {
        Row::of(
            Role::Ink,
            vec![
                EQUALS.into(),
                format!(
                    "{} {AREA} {DIVIDE} ({} {TIMES} {} {MIN})",
                    self.mean_square, self.twice, self.mean_gap
                ),
                EQUALS.into(),
                format!("{} {MIN}", self.mean),
            ],
        )
        .with(3, Role::Verdict)
    }

    /// 08_4: "2² = 4 min²", "10² = 100 min²".
    pub fn square_row(&self, i: usize) -> Row {
        let role = if i == 0 { Role::Short } else { Role::Long };
        Row::of(
            role,
            vec![
                format!("{}{SQUARED}", self.gaps[i]),
                EQUALS.into(),
                format!("{} {AREA}", self.squares[i]),
            ],
        )
    }

    /// 08_4: "average of g²: (4 + 100) ÷ 2 = 52 min²".
    pub fn average_square(&self) -> Row {
        let [a, b] = &self.squares;
        Row::of(
            Role::Ink,
            vec![
                format!("{AVERAGE_OF} {G}{SQUARED}:"),
                format!("({a} {PLUS} {b}) {DIVIDE} {}", self.gap_count),
                EQUALS.into(),
                format!("{} {AREA}", self.mean_square),
            ],
        )
    }

    /// 08_4: "(average gap)²: 6² = 36 min²".
    pub fn mean_squared(&self) -> Row {
        Row::of(
            Role::Dim,
            vec![
                format!("({AVERAGE_GAP}){SQUARED}:"),
                format!("{}{SQUARED}", self.mean_gap),
                EQUALS.into(),
                format!("{} {AREA}", self.mean_gap_squared),
            ],
        )
    }

    /// 08_4: "52 − 36 = 16 min²", the 16 in violet.
    pub fn extra_row(&self) -> Row {
        Row::of(
            Role::Ink,
            vec![
                format!("{} {MINUS} {}", self.mean_square, self.mean_gap_squared),
                EQUALS.into(),
                format!("{} {AREA}", self.spread),
            ],
        )
        .with(2, Role::Spread)
    }

    /// 08_4: "each gap is 4 from 6: (4² + 4²) ÷ 2 = 16 min²".
    pub fn distance_row(&self) -> Row {
        let d = &self.distance;
        Row::of(
            Role::Spread,
            vec![
                format!("{EACH_GAP} {d} {FROM} {}:", self.mean_gap),
                format!("({d}{SQUARED} {PLUS} {d}{SQUARED}) {DIVIDE} {}", self.gap_count),
                EQUALS.into(),
                format!("{} {AREA}", self.spread),
            ],
        )
    }

    /// 08_4's square argument for gap `i`: the long gap, 6 and 4 more, "10²
    /// = 6² + 6 × 4 + 6 × 4 + 4²"; the short gap, 6 and 4 less, "2² = 6² −
    /// 6 × 4 − 6 × 4 + 4²".
    pub fn argument_row(&self, i: usize) -> Row {
        let (m, d) = (&self.mean_gap, &self.distance);
        let (role, sign) = if i == 0 {
            (Role::Short, MINUS)
        } else {
            (Role::Long, PLUS)
        };
        Row::of(
            role,
            vec![
                format!("{}{SQUARED} {EQUALS}", self.gaps[i]),
                format!("{m}{SQUARED}"),
                format!("{sign} {m} {TIMES} {d} {sign} {m} {TIMES} {d} {PLUS}"),
                format!("{d}{SQUARED}"),
            ],
        )
        .with(1, Role::Dim)
        .with(3, Role::Spread)
    }

    /// 08_4: "average: 52 = 36 + 16 (the strips cancel)".
    pub fn cancel_row(&self) -> Row {
        Row::of(
            Role::Ink,
            vec![
                AVERAGE.into(),
                format!(
                    "{} {EQUALS} {} {PLUS} {}",
                    self.mean_square, self.mean_gap_squared, self.spread
                ),
                STRIPS.into(),
            ],
        )
        .with(2, Role::Dim)
    }

    /// 08_4: the name, after its picture and its number.
    pub fn spread_name(&self) -> Row {
        Row::of(
            Role::Spread,
            vec![SPREAD_NAME.into(), EQUALS.into(), format!("{} {AREA}", self.spread)],
        )
    }

    /// 08_4: the argument for any gaps: "any gaps: average of g² = (average
    /// gap)² + spread".
    pub fn identity_row(&self) -> Row {
        Row::of(
            Role::Ink,
            vec![
                ANY_GAPS.into(),
                format!("{AVERAGE_OF} {G}{SQUARED} {EQUALS} ({AVERAGE_GAP}){SQUARED} {PLUS}"),
                SPREAD.into(),
            ],
        )
        .with(0, Role::Dim)
        .with(2, Role::Spread)
    }

    /// 08_5: "½ × 6 min = 3 min", the naive half, dim.
    pub fn half_row(&self) -> Row {
        Row::of(
            Role::Dim,
            vec![
                format!("{} {TIMES} {} {MIN}", self.half, self.mean_gap),
                EQUALS.into(),
                format!("{} {MIN}", self.half_mean_gap),
            ],
        )
    }

    /// 08_5: "16 min² ÷ (2 × 6 min) = 1⅓ min = 1 min 20 s".
    pub fn penalty_row(&self) -> Row {
        Row::of(
            Role::Spread,
            vec![
                format!(
                    "{} {AREA} {DIVIDE} ({} {TIMES} {} {MIN})",
                    self.spread, self.twice, self.mean_gap
                ),
                EQUALS.into(),
                format!("{} {MIN}", self.penalty),
                EQUALS.into(),
                self.extra.clone(),
            ],
        )
    }

    /// 08_5: "3:00 + 1:20 = 4:20", boxed.
    pub fn verdict_row(&self) -> Row {
        Row::of(
            Role::Verdict,
            vec![
                self.half_clock.clone(),
                PLUS.into(),
                self.penalty_clock.clone(),
                EQUALS.into(),
                self.wait_clock.clone(),
            ],
        )
        .with(0, Role::Dim)
        .with(2, Role::Spread)
    }

    /// 08_5: "average wait = ½ × average gap + spread ÷ (2 × average gap)".
    pub fn formula_row(&self) -> Row {
        Row::of(
            Role::Ink,
            vec![
                AVERAGE_WAIT.into(),
                EQUALS.into(),
                format!("{} {TIMES} {AVERAGE_GAP}", self.half),
                PLUS.into(),
                format!("{SPREAD} {DIVIDE} ({} {TIMES} {AVERAGE_GAP})", self.twice),
            ],
        )
        .with(2, Role::Dim)
        .with(4, Role::Spread)
    }

    /// 08_8: the bound, "average wait ≥ ½ × average gap".
    pub fn bound_row(&self) -> Row {
        Row::of(
            Role::Verdict,
            vec![
                AVERAGE_WAIT.into(),
                AT_LEAST.into(),
                format!("{} {TIMES} {AVERAGE_GAP}", self.half),
            ],
        )
    }

    /// 08_8: the finite check's head and lines, the draft's card broken at
    /// its clauses.
    pub fn check_lines(&self) -> [String; 4] {
        [
            CHECK_HEAD.into(),
            format!("{CHECK_UP_TO} {} {CHECK_GAPS}", self.max_count),
            format!(
                "{CHECK_EACH} {} {CHECK_TO} {} {MIN}: {} {CHECK_CHOICES} {} {CHECK_BELOW}",
                self.lowest, self.max_gap, self.cases, self.violations
            ),
            CHECK_EQUAL.into(),
        ]
    }
}

// ----------------------------------------------------------------- picture

/// (moment, wait) corners in minutes, on screen.
fn on_screen(corners: &[(f64, f64)]) -> Vec<Vec3> {
    corners.iter().map(|&(t, w)| at(t, w)).collect()
}

/// A gap's triangle, chapter 05's: at its first bus from the line up the
/// whole gap, across to its next bus.
pub fn triangle(g: &Gap) -> [(f64, f64); 3] {
    let (a, b) = (g.from as f64, g.to as f64);
    [(a, 0.0), (a, b - a), (b, 0.0)]
}

/// The other half of the square on a gap: with the triangle it fills the
/// square from the gap's first bus to its next, up as far as the gap is
/// long.
pub fn mirror(g: &Gap) -> [(f64, f64); 3] {
    let (a, b) = (g.from as f64, g.to as f64);
    [(a, b - a), (b, b - a), (b, 0.0)]
}

/// A rectangle from moment `t0` to `t1` and from wait `w0` to `w1`, as
/// corners.
fn block(t0: f64, t1: f64, w0: f64, w1: f64) -> [(f64, f64); 4] {
    [(t0, w0), (t0, w1), (t1, w1), (t1, w0)]
}

/// 08_4's square argument on the long gap, `a` to `b` minutes, with the
/// average gap `m`: the average gap's square from the gap's first bus, the
/// strip right of it and the strip over it (each `m` by the distance), and
/// the corner square (the distance by the distance).
pub struct Squares {
    pub mean: [(f64, f64); 4],
    pub right: [(f64, f64); 4],
    pub over: [(f64, f64); 4],
    pub corner: [(f64, f64); 4],
}

pub fn squares(long: &Gap, m: f64) -> Squares {
    let (a, b, g) = (long.from as f64, long.to as f64, long.minutes() as f64);
    Squares {
        mean: block(a, a + m, 0.0, m),
        right: block(a + m, b, 0.0, m),
        over: block(a, a + m, m, g),
        corner: block(a + m, b, m, g),
    }
}

/// The middle of a block of corners.
fn middle(c: &[(f64, f64)]) -> (f64, f64) {
    let k = c.len() as f64;
    (
        c.iter().map(|p| p.0).sum::<f64>() / k,
        c.iter().map(|p| p.1).sum::<f64>() / k,
    )
}

/// A filled shape, no outline, detached.
fn filled(stage: &mut Stage<'_>, corners: &[(f64, f64)], color: Srgb, fill: f64) -> Mob {
    stage
        .arena_mut()
        .add(Polygon::new(on_screen(corners)).style(flat(color, fill)).build())
}

/// A shape's outline alone, closed, detached.
fn outline(stage: &mut Stage<'_>, corners: &[(f64, f64)], color: Srgb, w: f64) -> Mob {
    let mut closed = corners.to_vec();
    closed.push(corners[0]);
    stage
        .arena_mut()
        .add(polyline(on_screen(&closed)).style(line_style(color, w)).build())
}

/// A straight stroke between two screen points, detached.
fn segment(stage: &mut Stage<'_>, from: Vec3, to: Vec3, color: Srgb, w: f64) -> fmn::Result<Mob> {
    Ok(stage
        .arena_mut()
        .add(Line::new(from, to).style(line_style(color, w)).build()?))
}

/// The wait's axis as chapter 05 draws it: its line, a tick and a label at
/// each gap's length, and its name.
fn wait_axis(kit: &Kit, stage: &mut Stage<'_>, p: &Formula, gaps: &[Gap]) -> fmn::Result<Vec<Mob>> {
    let top = at(0.0, frame().y.1);
    let mut out = vec![segment(
        stage,
        at(0.0, 0.0),
        v(top[0], top[1] + Y_OVERHANG),
        Palette::dim(),
        2.0,
    )?];
    for (g, label) in gaps.iter().zip(&p.gaps) {
        if *label != g.minutes().to_string() {
            return Err(fail(format!("the tick at {} min is labelled {label}", g.minutes())));
        }
        let y = at(0.0, g.minutes() as f64)[1];
        out.push(segment(
            stage,
            v(ORIGIN_X - Y_TICK_W, y),
            v(ORIGIN_X, y),
            Palette::dim(),
            2.0,
        )?);
        out.push(kit.text(stage, label, SMALL, Palette::dim(), v(ORIGIN_X - Y_LABEL_DX, y))?);
    }
    out.push(kit.text(
        stage,
        WAIT_NAME,
        SMALL,
        Palette::dim(),
        v(ORIGIN_X, top[1] + Y_OVERHANG + WAIT_NAME_DY),
    )?);
    let fr = frame();
    out.push(kit.text(
        stage,
        MOMENT_NAME,
        SMALL,
        Palette::dim(),
        v(fr.center[0], MOMENT_NAME_Y),
    )?);
    Ok(out)
}

/// A row on screen: one text, where it starts, its size, and the row it
/// shows, so that a piece can be found in it.
#[derive(Clone)]
pub struct Shown {
    pub mob: Mob,
    pub row: Row,
    pub left: Vec3,
    pub size: f64,
}

/// A text's width at `size`, measured on a detached copy.
fn measure(kit: &Kit, stage: &mut Stage<'_>, s: &str, size: f64) -> fmn::Result<f64> {
    let m = kit.text(stage, s, size, Palette::ink(), ORIGIN)?;
    Ok(stage.get_width(m))
}

/// `row` as one text starting at `left` (its left edge, on its middle
/// line), its accents in their colors; detached.
pub fn show_row(kit: &Kit, stage: &mut Stage<'_>, row: &Row, left: Vec3, size: f64) -> fmn::Result<Shown> {
    let accents: Vec<(&str, Srgb)> = row.accents().into_iter().map(|(p, r)| (p, role_color(r))).collect();
    let mob = kit.text_t2c(stage, &row.text(), size, role_color(row.role), &accents, left)?;
    stage.move_to(mob, left, LEFT);
    Ok(Shown {
        mob,
        row: row.clone(),
        left,
        size,
    })
}

impl Shown {
    /// Piece `i`'s box, measured: its centre, width and height. Its right
    /// edge is where the row's first `i + 1` pieces end.
    pub fn piece(&self, kit: &Kit, stage: &mut Stage<'_>, i: usize) -> fmn::Result<(Vec3, f64, f64)> {
        let through = measure(kit, stage, &self.row.joined(i + 1), self.size)?;
        let own = measure(kit, stage, &self.row.pieces[i].0, self.size)?;
        let h = stage.get_height(self.mob);
        Ok((v(self.left[0] + through - own / 2.0, self.left[1]), own, h))
    }

    /// A copy of piece `i` for carrying: its text, color and size, made over
    /// it.
    fn source(&self, kit: &Kit, stage: &mut Stage<'_>, i: usize) -> fmn::Result<Source> {
        let (at, _, _) = self.piece(kit, stage, i)?;
        let (text, role) = &self.row.pieces[i];
        Ok(Source {
            at,
            text: text.clone(),
            color: role_color(*role),
            size: self.size,
        })
    }

    /// The row moved to start at `left`, its pieces with it.
    fn moved(&self, stage: &mut Stage<'_>, left: Vec3) -> fmn::Result<(Shown, AnimBuilder)> {
        let c = stage.get_center(self.mob);
        let to = v(c[0] + left[0] - self.left[0], c[1] + left[1] - self.left[1]);
        let mut out = self.clone();
        out.left = left;
        Ok((out, fly(self.mob, to, MOVE_RT)?))
    }
}

/// A text to carry: where it stands, what it says, its color and size.
struct Source {
    at: Vec3,
    text: String,
    color: Srgb,
    size: f64,
}

/// A bracket's length under the stretch, as a text to carry.
fn bracket_source(stage: &Stage<'_>, label: Mob, text: &str, color: Srgb) -> Source {
    Source {
        at: stage.get_center(label),
        text: text.to_owned(),
        color,
        size: BRACKET_LABEL_SIZE,
    }
}

/// Builder animations together, each made with its own run time.
pub fn play_all(stage: &mut Stage<'_>, builders: Vec<AnimBuilder>) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for b in builders {
        anims.push(stage.prepare(b)?);
    }
    if !anims.is_empty() {
        stage.play_prepared(anims)?;
    }
    Ok(())
}

/// Numbers carried into a row: a fresh copy of each source, made over it,
/// flies to its target (a piece's measured centre in the row to come); the
/// copies go, and then `coming` fades in. One change at a time: the old
/// digits are gone before the new row shows, so no two overlap.
fn carry(kit: &Kit, stage: &mut Stage<'_>, trips: Vec<(Source, Vec3)>, coming: &[Mob]) -> fmn::Result<()> {
    let mut copies = Vec::with_capacity(trips.len());
    let mut flights = Vec::with_capacity(trips.len());
    for (s, to) in trips {
        let copy = kit.text(stage, &s.text, s.size, s.color, s.at)?;
        stage.add_to_scene(copy)?;
        flights.push(fly(copy, to, CARRY_RT)?);
        copies.push(copy);
    }
    play_all(stage, flights)?;
    fade_out_all(stage, &copies, 0.2)?;
    fade_in_all(stage, coming, 0.35)
}

/// Emphasis for texts or pieces of them: an outline just around each box
/// (centre, width, height), stroked only, faded in, held and faded out over
/// `rt`, the words left as they are.
pub fn halo(stage: &mut Stage<'_>, boxes: &[(Vec3, f64, f64)], color: Srgb, rt: f64) -> fmn::Result<()> {
    let mut rings = Vec::with_capacity(boxes.len());
    for &(c, w, h) in boxes {
        let (w, h) = (w + 2.0 * HALO_PAD, h + 2.0 * HALO_PAD);
        let ring = rect(v(c[0] - w / 2.0, c[1] - h / 2.0), w, h, line_style(color, HALO_W))?;
        rings.push(stage.arena_mut().add(ring));
    }
    fade_in_all(stage, &rings, rt * 0.3)?;
    pause(stage, rt * 0.4)?;
    fade_out_all(stage, &rings, rt * 0.3)
}

/// A whole text's box, for [`halo`].
pub fn text_box(stage: &Stage<'_>, m: Mob) -> (Vec3, f64, f64) {
    (stage.get_center(m), stage.get_width(m), stage.get_height(m))
}

/// Brighten filled shapes and let them settle back to `fill`: an area
/// pointed at. (A flash would scale a triangle off its axes.)
pub fn pulse(stage: &mut Stage<'_>, fills: &[Mob], fill: f64) -> fmn::Result<()> {
    for opacity in [PULSE_FILL, fill] {
        let builders = fills
            .iter()
            .map(|&m| fade_to(m, opacity, PULSE_RT / 2.0))
            .collect::<fmn::Result<Vec<_>>>()?;
        play_all(stage, builders)?;
    }
    Ok(())
}

/// A box around a row on screen, in `color`, stroked only; detached.
pub fn frame_around(stage: &mut Stage<'_>, m: Mob, color: Srgb) -> fmn::Result<Mob> {
    let c = stage.get_center(m);
    let w = stage.get_width(m) + 2.0 * BOX_PAD[0];
    let h = stage.get_height(m) + 2.0 * BOX_PAD[1];
    let r = rect(v(c[0] - w / 2.0, c[1] - h / 2.0), w, h, line_style(color, BOX_W))?;
    Ok(stage.arena_mut().add(r))
}

/// Panel row `i`'s left edge.
fn panel(i: usize) -> Vec3 {
    v(PANEL_LEFT, row_y(i))
}

impl SceneConstruct for TheFormula {
    fn name(&self) -> &str {
        "08_the_formula"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let f = facts::facts().map_err(fail)?;
        let p = formula(&f).map_err(fail)?;
        let n = twelve_minutes::numbers(&f).map_err(fail)?;
        let people = twelve_minutes::passengers().map_err(fail)?;
        let gs = twelve_minutes::gaps();
        let (short_gap, long_gap) = (gs[0], gs[1]);
        let m: f64 = p.mean_gap.parse().map_err(|e| fail(format!("the average gap: {e}")))?;
        let colors = [timetable::short_gap(), palette::long_gap()];
        let violet = spread_color();

        // 08_1: chapter 05's picture, the stretch under its two triangles on
        // their axes (the buses gone, as 05_3 left them); then each
        // triangle, half its square, written as it is said; the letters
        let st = twelve_minutes::stretch(&kit, stage, &n, &people)?;
        let mut fills = Vec::new();
        let mut shapes = Vec::new();
        for (g, &c) in gs.iter().zip(&colors) {
            let fill = filled(stage, &triangle(g), c, TRI_FILL);
            fills.push(fill);
            shapes.extend([fill, outline(stage, &triangle(g), c, TRI_W)]);
        }
        let axes = wait_axis(&kit, stage, &p, &gs)?;
        let mut opening = vec![st.axis];
        opening.extend(&st.ticks);
        opening.extend(&st.clocks);
        opening.extend(st.bracket_mobs());
        opening.extend(&st.dots);
        opening.extend(&shapes);
        opening.extend(&axes);
        fade_in_all(stage, &opening, 0.8)?;
        let line = say(&kit, stage, "08_1")?;
        let mirrors: Vec<Mob> = gs
            .iter()
            .map(|g| outline(stage, &mirror(g), Palette::faint(), MIRROR_W))
            .collect();
        let bridge = kit.text_left(stage, BRIDGE, NOTE_SIZE, Palette::dim(), BRIDGE_AT)?;
        let mut rows: Vec<Shown> = Vec::new();
        for (i, phrase) in ["The gap of two", "the gap of ten holds fifty"].into_iter().enumerate() {
            line.wait_for(stage, phrase)?;
            let r = show_row(&kit, stage, &p.area_row(i), panel(i), PANEL_SIZE)?;
            let (slot, _, _) = r.piece(&kit, stage, 0)?;
            let label = st.brackets[i].label;
            let from = bracket_source(stage, label, &p.gaps[i], colors[i]);
            carry(&kit, stage, vec![(from, slot)], &[r.mob])?;
            let mut extra = vec![mirrors[i]];
            if i == 0 {
                extra.push(bridge);
            }
            fade_in_all(stage, &extra, 0.4)?;
            pulse(stage, &fills[i..=i], TRI_FILL)?;
            rows.push(r);
        }
        line.wait_for(stage, "Any gap holds half its length squared")?;
        let letters = show_row(&kit, stage, &p.letter_row(), panel(2), PANEL_SIZE)?;
        let trips = vec![
            (rows[1].source(&kit, stage, 0)?, letters.piece(&kit, stage, 0)?.0),
            (rows[1].source(&kit, stage, 3)?, letters.piece(&kit, stage, 3)?.0),
        ];
        carry(&kit, stage, trips, &[letters.mob])?;
        let leg = at(long_gap.from as f64, long_gap.minutes() as f64 / 2.0);
        let base = at((long_gap.from + long_gap.to) as f64 / 2.0, 0.0);
        let g_labels = vec![
            kit.text(stage, G, PLOT_LABEL, Palette::ink(), v(leg[0] - G_LEG_DX, leg[1]))?,
            kit.text(stage, G, PLOT_LABEL, Palette::ink(), v(base[0], AXIS_Y + G_BASE_DY))?,
        ];
        fade_in_all(stage, &g_labels, 0.5)?;
        rows.push(letters);

        // 08_2: the areas added, carried from the area rows; the time they
        // cover, carried from the brackets; the wait; then the letters
        let line = say(&kit, stage, "08_2")?;
        let [left_row, right_row] = p.sum_rows();
        line.wait_for(stage, "Add up the triangles")?;
        let sum_left = show_row(&kit, stage, &left_row, panel(3), PANEL_SIZE)?;
        let slot = sum_left.piece(&kit, stage, 2)?.0;
        let trips = vec![
            (rows[0].source(&kit, stage, 5)?, slot),
            (rows[1].source(&kit, stage, 5)?, slot),
        ];
        carry(&kit, stage, trips, &[sum_left.mob])?;
        pulse(stage, &fills, TRI_FILL)?;
        line.wait_for(stage, "divide by the time they cover")?;
        let gap = SPACE * PANEL_SIZE;
        let right_at = v(PANEL_LEFT + stage.get_width(sum_left.mob) + gap, row_y(3));
        let sum_right = show_row(&kit, stage, &right_row, right_at, PANEL_SIZE)?;
        let slot = sum_right.piece(&kit, stage, 1)?.0;
        let trips = (0..2)
            .map(|i| (bracket_source(stage, st.brackets[i].label, &p.gaps[i], colors[i]), slot))
            .collect();
        let bracket_lines: Vec<Mob> = st.brackets.iter().map(|b| b.line).collect();
        flash_all(stage, &bracket_lines, 0.6)?;
        carry(&kit, stage, trips, &[sum_right.mob])?;
        line.wait_for(stage, "That's the average wait")?;
        let eq_x = sum_left.piece(&kit, stage, 1)?;
        let result_at = v(eq_x.0[0] - eq_x.1 / 2.0, row_y(4));
        let result = show_row(&kit, stage, &p.sum_result(), result_at, PANEL_SIZE)?;
        fade_in_all(stage, &[result.mob], 0.5)?;
        let wait_piece = result.piece(&kit, stage, 3)?;
        halo(stage, &[wait_piece], palette::verdict(), HALO_RT)?;
        kit.hold(stage)?;
        let sum_letters = show_row(&kit, stage, &p.sum_letters(), panel(5), PANEL_SIZE)?;
        let trips = vec![
            (sum_left.source(&kit, stage, 2)?, sum_letters.piece(&kit, stage, 2)?.0),
            (sum_right.source(&kit, stage, 1)?, sum_letters.piece(&kit, stage, 4)?.0),
        ];
        carry(&kit, stage, trips, &[sum_letters.mob])?;
        kit.music_breath(stage, LETTERS_LOOK_S)?;

        // 08_3: the worked rows go and the letters move up before the line
        // (its first words act on them); top and bottom over the number of
        // gaps; what each becomes; the formula and our numbers under it
        let leaving = vec![rows[0].mob, rows[1].mob, sum_left.mob, sum_right.mob, result.mob];
        fade_out_all(stage, &leaving, 0.4)?;
        let (g_row, a) = rows[2].moved(stage, panel(0))?;
        let (sum_row, b) = sum_letters.moved(stage, panel(1))?;
        play_all(stage, vec![a, b])?;
        let line = say(&kit, stage, "08_3")?;
        line.wait_for(stage, "Divide the top and the bottom")?;
        let parts = [sum_row.piece(&kit, stage, 2)?, sum_row.piece(&kit, stage, 4)?];
        let by = show_row(&kit, stage, &p.by_gaps(), panel(2), PANEL_SIZE)?;
        fade_in_all(stage, &[by.mob], 0.4)?;
        halo(stage, &parts, Palette::ink(), HALO_RT)?;
        line.wait_for(stage, "The top becomes")?;
        let top = show_row(&kit, stage, &p.top_row(), panel(3), PANEL_SIZE)?;
        let trips = vec![(sum_row.source(&kit, stage, 2)?, top.piece(&kit, stage, 1)?.0)];
        carry(&kit, stage, trips, &[top.mob])?;
        line.wait_for(stage, "The bottom becomes")?;
        let bottom = show_row(&kit, stage, &p.bottom_row(), panel(4), PANEL_SIZE)?;
        let trips = vec![(sum_row.source(&kit, stage, 4)?, bottom.piece(&kit, stage, 1)?.0)];
        carry(&kit, stage, trips, &[bottom.mob])?;
        kit.hold(stage)?;
        let mean_letters = show_row(&kit, stage, &p.mean_letters(), panel(5), PANEL_SIZE)?;
        let trips = vec![
            (top.source(&kit, stage, 3)?, mean_letters.piece(&kit, stage, 2)?.0),
            (bottom.source(&kit, stage, 3)?, mean_letters.piece(&kit, stage, 4)?.0),
        ];
        carry(&kit, stage, trips, &[mean_letters.mob])?;
        let eq = mean_letters.piece(&kit, stage, 1)?;
        let numbers = show_row(
            &kit,
            stage,
            &p.mean_numbers(),
            v(eq.0[0] - eq.1 / 2.0, row_y(6)),
            PANEL_SIZE,
        )?;
        fade_in_all(stage, &[numbers.mob], 0.5)?;
        kit.music_breath(stage, LETTERS_LOOK_S)?;

        // 08_4: the derivation goes, the formula and its numbers move up
        // before the line; each triangle and its mirror fill the square on
        // its gap; the average square against the average gap's; the corner
        // left over; each gap's distance from the average
        let mut leaving = vec![g_row.mob, sum_row.mob, by.mob, top.mob, bottom.mob];
        leaving.extend(&g_labels);
        fade_out_all(stage, &leaving, 0.4)?;
        // the numbers keep their place under the formula's "="
        let (_, a) = mean_letters.moved(stage, panel(0))?;
        let (_, b) = numbers.moved(stage, v(numbers.left[0], row_y(1)))?;
        play_all(stage, vec![a, b])?;
        let line = say(&kit, stage, "08_4")?;
        let mirror_fills: Vec<Mob> = gs
            .iter()
            .zip(&colors)
            .map(|(g, &c)| filled(stage, &mirror(g), c, TRI_FILL))
            .collect();
        let short_top = at((short_gap.from + short_gap.to) as f64 / 2.0, short_gap.minutes() as f64);
        let long_top = at((long_gap.from + long_gap.to) as f64 / 2.0, long_gap.minutes() as f64);
        let square_labels = [
            kit.text(
                stage,
                &format!("{} {AREA}", p.squares[0]),
                PLOT_LABEL,
                Palette::ink(),
                v(short_top[0], short_top[1] + ABOVE_SQUARE),
            )?,
            kit.text(
                stage,
                &format!("{} {AREA}", p.squares[1]),
                PLOT_LABEL,
                colors[1],
                v(long_top[0], long_top[1] + ABOVE_SQUARE),
            )?,
        ];
        let mut square_rows = Vec::new();
        for (i, phrase) in ["two squared is four", "ten squared is a hundred"]
            .into_iter()
            .enumerate()
        {
            line.wait_for(stage, phrase)?;
            let r = show_row(&kit, stage, &p.square_row(i), panel(2 + i), PANEL_SIZE)?;
            fade_in_all(stage, &[mirror_fills[i], square_labels[i], r.mob], 0.6)?;
            square_rows.push(r);
        }
        line.wait_for(stage, "the average square is fifty-two")?;
        let average = show_row(&kit, stage, &p.average_square(), panel(4), PANEL_SIZE)?;
        let slot = average.piece(&kit, stage, 1)?.0;
        let trips = vec![
            (square_rows[0].source(&kit, stage, 2)?, slot),
            (square_rows[1].source(&kit, stage, 2)?, slot),
        ];
        carry(&kit, stage, trips, &[average.mob])?;
        let sq = squares(&long_gap, m);
        line.wait_for(stage, "Six squared is only thirty-six")?;
        let mean_square = outline(stage, &sq.mean, Palette::dim(), SQUARE_W);
        let mid = middle(&sq.mean);
        let mean_label = kit.text(
            stage,
            &format!("{} {AREA}", p.mean_gap_squared),
            PLOT_LABEL,
            Palette::dim(),
            at(mid.0, mid.1),
        )?;
        let level = segment(
            stage,
            at(0.0, m),
            at(long_gap.from as f64, m),
            Palette::faint(),
            STRIP_W,
        )?;
        let mean_row = show_row(&kit, stage, &p.mean_squared(), panel(5), PANEL_SIZE)?;
        fade_in_all(stage, &[mean_square, mean_label, level, mean_row.mob], 0.6)?;
        line.wait_for(stage, "The extra sixteen")?;
        let corner = filled(stage, &sq.corner, violet, CORNER_FILL);
        let mid = middle(&sq.corner);
        let corner_label = kit.text(
            stage,
            &format!("{} {AREA}", p.spread),
            PLOT_LABEL,
            Palette::ink(),
            at(mid.0, mid.1),
        )?;
        let extra = show_row(&kit, stage, &p.extra_row(), panel(6), PANEL_SIZE)?;
        fade_in_all(stage, &[corner, corner_label, extra.mob], 0.6)?;
        line.wait_for(stage, "how uneven the gaps are")?;
        let dx = DISTANCE_DX;
        let (s_top, l_top) = (short_gap.minutes() as f64, long_gap.minutes() as f64);
        let short_from = at(short_gap.from as f64, s_top);
        let long_from = at(long_gap.from as f64, m);
        let distances = vec![
            segment(
                stage,
                v(short_from[0] + dx, short_from[1]),
                v(short_from[0] + dx, at(0.0, m)[1]),
                violet,
                DISTANCE_W,
            )?,
            segment(
                stage,
                v(long_from[0] - dx, long_from[1]),
                v(long_from[0] - dx, at(0.0, l_top)[1]),
                violet,
                DISTANCE_W,
            )?,
            kit.text(
                stage,
                &p.distance,
                PLOT_LABEL,
                violet,
                v(DISTANCE_LABEL_X, at(0.0, (s_top + m) / 2.0)[1]),
            )?,
            kit.text(
                stage,
                &p.distance,
                PLOT_LABEL,
                violet,
                v(DISTANCE_LABEL_X, at(0.0, (m + l_top) / 2.0)[1]),
            )?,
        ];
        let distance_row = show_row(&kit, stage, &p.distance_row(), panel(7), PANEL_SIZE)?;
        let mut coming = distances.clone();
        coming.push(distance_row.mob);
        fade_in_all(stage, &coming, 0.6)?;

        // the square argument, held on screen with no words: the strips on
        // the long square, the two gaps' squares, the strips cancelling,
        // the name, and the argument for any gaps
        kit.hold(stage)?;
        let mut strips = Vec::new();
        for b in [sq.right, sq.over] {
            strips.push(outline(stage, &b, Palette::faint(), STRIP_W));
            let mid = middle(&b);
            strips.push(kit.text(
                stage,
                &format!("{} {TIMES} {}", p.mean_gap, p.distance),
                PLOT_LABEL,
                Palette::dim(),
                at(mid.0, mid.1),
            )?);
        }
        fade_in_all(stage, &strips, 0.6)?;
        let argument: Vec<Shown> = [
            p.argument_row(1),
            p.argument_row(0),
            p.cancel_row(),
            p.spread_name(),
            p.identity_row(),
        ]
        .iter()
        .enumerate()
        .map(|(i, r)| show_row(&kit, stage, r, panel(8 + i), PANEL_SIZE))
        .collect::<fmn::Result<_>>()?;
        // one step at a time, each held long enough to read: the two
        // squares split at the average, the strips cancelling, the name,
        // the rule for any gaps
        let argument_mobs: Vec<Mob> = argument.iter().map(|s| s.mob).collect();
        let read = |rows: &[Shown]| read_s(&rows.iter().map(|s| s.row.text()).collect::<Vec<_>>().join(" "));
        fade_in_lagged(stage, &argument_mobs[..2], 1.2, 0.5)?;
        pause(stage, read(&argument[..2]))?;
        for (shown, &mob) in argument[2..4].iter().zip(&argument_mobs[2..4]) {
            fade_in_all(stage, &[mob], 0.5)?;
            pause(stage, read(std::slice::from_ref(shown)))?;
        }
        fade_in_all(stage, &argument_mobs[4..], 0.5)?;
        kit.music_breath(stage, read(&argument[4..]).max(SQUARES_LOOK_S))?;

        // 08_5: the worked squares go; the argument for any gaps moves up;
        // our numbers first, the box, then the formula in letters
        let line = say(&kit, stage, "08_5")?;
        let mut leaving: Vec<Mob> = square_rows.iter().map(|s| s.mob).collect();
        leaving.extend([average.mob, mean_row.mob, extra.mob, distance_row.mob]);
        leaving.extend(&argument_mobs[..4]);
        fade_out_all(stage, &leaving, 0.4)?;
        let (identity, a) = argument[4].moved(stage, panel(2))?;
        play_all(stage, vec![a])?;
        line.wait_for(stage, "three minutes")?;
        let half_row = show_row(&kit, stage, &p.half_row(), panel(3), PANEL_SIZE)?;
        fade_in_all(stage, &[half_row.mob], 0.5)?;
        line.wait_for(stage, "plus sixteen over twice six")?;
        let penalty_row = show_row(&kit, stage, &p.penalty_row(), panel(4), PANEL_SIZE)?;
        let trips = vec![(identity.source(&kit, stage, 2)?, penalty_row.piece(&kit, stage, 0)?.0)];
        carry(&kit, stage, trips, &[penalty_row.mob])?;
        line.wait_for(stage, "a minute and twenty seconds")?;
        let verdict = show_row(&kit, stage, &p.verdict_row(), panel(5), PANEL_SIZE)?;
        let trips = vec![
            (half_row.source(&kit, stage, 2)?, verdict.piece(&kit, stage, 0)?.0),
            (penalty_row.source(&kit, stage, 4)?, verdict.piece(&kit, stage, 2)?.0),
        ];
        carry(&kit, stage, trips, &[verdict.mob])?;
        let boxed = frame_around(stage, verdict.mob, palette::verdict())?;
        fade_in_all(stage, &[boxed], 0.4)?;
        line.wait_for(stage, "There's the extra time")?;
        let extra_piece = verdict.piece(&kit, stage, 2)?;
        halo(stage, &[extra_piece], violet, HALO_RT)?;
        kit.hold(stage)?;
        let letters = show_row(&kit, stage, &p.formula_row(), panel(6), PANEL_SIZE)?;
        let trips = vec![
            (half_row.source(&kit, stage, 0)?, letters.piece(&kit, stage, 2)?.0),
            (penalty_row.source(&kit, stage, 0)?, letters.piece(&kit, stage, 4)?.0),
        ];
        carry(&kit, stage, trips, &[letters.mob])?;
        kit.music_breath(stage, FORMULA_LOOK_S)?;

        // 08_6: the stretch gives way to the sign and you at the stop; the
        // 1:20 flies to you; the sign's every-six dims; it counts buses
        let line = say(&kit, stage, "08_6")?;
        let mut plot = opening.clone();
        plot.extend(&mirrors);
        plot.extend(&mirror_fills);
        plot.extend(&square_labels);
        plot.extend([mean_square, mean_label, level, corner, corner_label, bridge]);
        plot.extend(&distances);
        plot.extend(&strips);
        fade_out_all(stage, &plot, 0.5)?;
        let sign = timetable::sign(&kit, stage, &f)?;
        let me = you(stage, YOU_AT)?;
        let mut street = sign.mobs();
        street.push(me);
        fade_in_all(stage, &street, 0.6)?;
        line.wait_for(stage, "that minute and twenty seconds")?;
        let morning = kit.text(
            stage,
            &format!("{PLUS}{} {A_MORNING}", p.extra),
            MORNING_SIZE,
            violet,
            v(YOU_AT[0], YOU_AT[1] + MORNING_DY),
        )?;
        let trips = vec![(verdict.source(&kit, stage, 2)?, stage.get_center(morning))];
        carry(&kit, stage, trips, &[morning])?;
        line.wait_for(stage, "The sign can't show it")?;
        play_all(stage, vec![fade_to(sign.every, DIM_TO, 0.6)?])?;
        line.wait_for(stage, "it counts buses")?;
        let b = text_box(stage, sign.buses);
        halo(stage, &[b], Palette::ink(), HALO_RT)?;
        line.wait_for(stage, "not mornings")?;
        let b = text_box(stage, morning);
        halo(stage, &[b], violet, HALO_RT)?;

        // 08_7: the question over the sign; think time; nothing on screen
        // answers it
        let line = say(&kit, stage, "08_7")?;
        line.wait_for(stage, "Can the average wait ever be less")?;
        let ask = kit.text(stage, ASK, ASK_SIZE, Palette::ink(), ASK_AT)?;
        fade_in_all(stage, &[ask], 0.4)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 08_8: never: the spread is an average of squares; who turns up;
        // the bound; equality only for even gaps; the finite check, a check
        // and not the proof; K05
        let line = say(&kit, stage, "08_8")?;
        fade_out_all(stage, &[q, ask], 0.3)?;
        line.wait_for(stage, "The extra piece is never negative")?;
        let never = show_row(&kit, stage, &Row::new(Role::Spread, &[NEVER]), panel(7), PANEL_SIZE)?;
        fade_in_all(stage, &[never.mob], 0.4)?;
        let spread_term = letters.piece(&kit, stage, 4)?;
        let argued = text_box(stage, identity.mob);
        halo(stage, &[spread_term, argued], violet, HALO_RT)?;
        line.wait_for(stage, "who turn up at random")?;
        let who = show_row(&kit, stage, &Row::new(Role::Ink, &[WHO]), panel(8), WHO_SIZE)?;
        fade_in_all(stage, &[who.mob], 0.4)?;
        line.wait_for(stage, "at least half the average gap")?;
        let bound = show_row(&kit, stage, &p.bound_row(), panel(9), PANEL_SIZE)?;
        fade_in_all(stage, &[bound.mob], 0.4)?;
        line.wait_for(stage, "exactly half only when every gap is the same")?;
        let exactly = show_row(&kit, stage, &Row::new(Role::Verdict, &[EXACTLY]), panel(10), PANEL_SIZE)?;
        let card = stage
            .arena_mut()
            .add(rect(CHECK_DL, CHECK_W, CHECK_H, line_style(Palette::faint(), 2.0))?);
        let mut check = vec![card];
        for (i, (text, &y)) in p.check_lines().iter().zip(&CHECK_YS).enumerate() {
            let size = if i == 0 { CHECK_SIZES[0] } else { CHECK_SIZES[1] };
            check.push(kit.text_left(stage, text, size, Palette::dim(), v(CHECK_X, y))?);
        }
        fade_in_all(stage, &[exactly.mob], 0.4)?;
        fade_in_all(stage, &check, 0.5)?;
        let card_up = now(stage);
        kit.hold(stage)?;
        let k05 = precisely(&kit, stage, K05)?;
        fade_in_all(stage, &[k05], 0.5)?;
        let k05_up = now(stage);
        let card_text = p.check_lines()[1..].join(" ");
        wait_until(
            stage,
            (card_up + read_s(&card_text) + read_s(K05)).max(k05_up + read_s(K05)),
        )?;
        let up = now(stage);
        stand(&kit, stage, up)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use explainer_kit::{Narrator, Script};

    use super::*;

    /// The full draft with its line breaks and indents as single spaces.
    fn draft() -> String {
        crate::DRAFT.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// A copy of the facts with `key` printed as `bad`.
    fn with_fact(f: &Facts, key: &str, bad: &str) -> Facts {
        let mut wrong = Facts::new();
        for row in f.print().lines() {
            let (k, value) = row.split_once('\t').unwrap();
            wrong.add(k, if k == key { bad } else { value }).unwrap();
        }
        wrong
    }

    /// Every row the panel shows before the question of 08_7, in order.
    fn before_the_question(p: &Formula) -> Vec<Row> {
        let [left, right] = p.sum_rows();
        vec![
            p.area_row(0),
            p.area_row(1),
            p.letter_row(),
            left,
            right,
            p.sum_result(),
            p.sum_letters(),
            p.by_gaps(),
            p.top_row(),
            p.bottom_row(),
            p.mean_letters(),
            p.mean_numbers(),
            p.square_row(0),
            p.square_row(1),
            p.average_square(),
            p.mean_squared(),
            p.extra_row(),
            p.distance_row(),
            p.argument_row(1),
            p.argument_row(0),
            p.cancel_row(),
            p.spread_name(),
            p.identity_row(),
            p.half_row(),
            p.penalty_row(),
            p.verdict_row(),
            p.formula_row(),
        ]
    }

    /// Needs chapter 08's lines in script.tsv (the integration step copies
    /// them from the draft). The whole chapter runs headless at 10 fps with
    /// the scratch voice: every phrase it waits for is in its line, the
    /// numbers match the facts, and its eight lines are said in order. It
    /// ends with `kit.finish`, whose timeline runs past the end of the last
    /// line.
    #[test]
    fn the_chapter_runs_and_says_its_lines_in_order() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let narrator = Narrator::estimate(&script);
        let kit = Kit::new().unwrap().with_narrator(Rc::clone(&narrator));
        let mut scene = TheFormula { kit };
        let config = fmn::scene::RuntimeConfig {
            fps: 10,
            ..fmn::scene::RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut fmn::scene::NullSceneSink).unwrap();
        let said_at = narrator.said();
        let said: Vec<&str> = said_at.iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(said, ["08_1", "08_2", "08_3", "08_4", "08_5", "08_6", "08_7", "08_8"]);
        let (last, at) = said_at.last().unwrap();
        let last_ends = at + narrator.length(last).unwrap();
        let end = scene
            .kit
            .finished
            .borrow()
            .as_ref()
            .expect("no timeline: the chapter must end with kit.finish")
            .seconds();
        assert!(
            end > last_ends,
            "the timeline ends at {end:.2} s, before {last} ends at {last_ends:.2} s"
        );
    }

    /// K05 and the finite check's card are the draft's word for word, and
    /// so are the question and the name of g; the card's three lines read
    /// as the draft's one sentence. By the kit's wrap at 92 characters K05
    /// fits the lower-third's three lines without being cut.
    #[test]
    fn the_cards_are_the_drafts() {
        let board = draft();
        assert!(board.contains(&format!("\"{K05}\"")), "{K05}");
        let p = formula(&facts::facts().unwrap()).unwrap();
        let lines = p.check_lines();
        let card = lines[1..].join(" ");
        assert_eq!(
            card,
            "every choice of up to 4 gaps (repeats allowed, order ignored), each 0 to 12 min: 2,375 choices, 0 \
             below half; equal only when all the gaps are the same"
        );
        assert!(board.contains(&format!("\"{card}\"")), "{card}");
        assert!(board.contains(&format!("\"{ASK}\"")));
        assert!(
            board.contains(G_NAME),
            "the complete displayed name and unit of g must remain in the storyboard"
        );
        let wrapped = explainer_kit::animatic::wrap(K05, 92, 3);
        assert!(wrapped.len() <= 3 && !wrapped.concat().contains('…'), "{wrapped:?}");
    }

    /// The rows print the facts' numbers, with units: an area in square
    /// minutes, divided by minutes, is a wait in minutes.
    #[test]
    fn the_rows_read_as_their_numbers() {
        let p = formula(&facts::facts().unwrap()).unwrap();
        let [left, right] = p.sum_rows();
        let read = [
            (p.area_row(0), "2 min → ½ × (2 min)² = 2 min²"),
            (p.area_row(1), "10 min → ½ × (10 min)² = 50 min²"),
            (p.letter_row(), "g → ½ × g² (g: a gap's length, min)"),
            (left, "average wait = (2 + 50) min²"),
            (right, "÷ (2 + 10) min"),
            (p.sum_result(), "= 52 min² ÷ 12 min = 4⅓ min"),
            (p.sum_letters(), "average wait = (sum of ½ g²) ÷ (sum of g)"),
            (p.by_gaps(), "top and bottom ÷ 2, the number of gaps"),
            (p.top_row(), "top: (sum of ½ g²) ÷ 2 = ½ × average of g²"),
            (p.bottom_row(), "bottom: (sum of g) ÷ 2 = average gap"),
            (p.mean_letters(), "average wait = average of g² ÷ (2 × average gap)"),
            (p.mean_numbers(), "= 52 min² ÷ (2 × 6 min) = 4⅓ min"),
            (p.square_row(0), "2² = 4 min²"),
            (p.square_row(1), "10² = 100 min²"),
            (p.average_square(), "average of g²: (4 + 100) ÷ 2 = 52 min²"),
            (p.mean_squared(), "(average gap)²: 6² = 36 min²"),
            (p.extra_row(), "52 − 36 = 16 min²"),
            (p.distance_row(), "each gap is 4 from 6: (4² + 4²) ÷ 2 = 16 min²"),
            (p.argument_row(1), "10² = 6² + 6 × 4 + 6 × 4 + 4²"),
            (p.argument_row(0), "2² = 6² − 6 × 4 − 6 × 4 + 4²"),
            (p.cancel_row(), "average: 52 = 36 + 16 (the strips cancel)"),
            (
                p.spread_name(),
                "spread: the average squared distance from the average gap = 16 min²",
            ),
            (p.identity_row(), "any gaps: average of g² = (average gap)² + spread"),
            (p.half_row(), "½ × 6 min = 3 min"),
            (p.penalty_row(), "16 min² ÷ (2 × 6 min) = 1⅓ min = 1 min 20 s"),
            (p.verdict_row(), "3:00 + 1:20 = 4:20"),
            (
                p.formula_row(),
                "average wait = ½ × average gap + spread ÷ (2 × average gap)",
            ),
            (p.bound_row(), "average wait ≥ ½ × average gap"),
        ];
        for (row, text) in read {
            assert_eq!(row.text(), text);
            // an accent's color would spill onto a second occurrence
            for (piece, _) in row.accents() {
                assert_eq!(row.text().matches(piece).count(), 1, "{piece} in {text}");
            }
        }
    }

    /// The numbers are the model's, and a fact that stops matching it is
    /// refused: the spread, the commuter's 1:20, the check's count, the
    /// triangle's share of its square and the squares, each changed.
    #[test]
    fn the_formula_is_the_models_and_a_wrong_fact_is_refused() {
        let f = facts::facts().unwrap();
        let p = formula(&f).unwrap();
        assert_eq!((p.mean_square.as_str(), p.mean_gap_squared.as_str()), ("52", "36"));
        assert_eq!((p.spread.as_str(), p.distance.as_str()), ("16", "4"));
        assert_eq!((p.penalty.as_str(), p.penalty_clock.as_str()), ("1⅓", "1:20"));
        assert_eq!((p.half_clock.as_str(), p.wait_clock.as_str()), ("3:00", "4:20"));
        assert_eq!((p.half.as_str(), p.twice.as_str()), ("½", "2"));
        assert_eq!(p.extra, "1 min 20 s");
        assert_eq!((p.lowest.as_str(), p.cases.as_str()), ("0", "2,375"));
        for (key, bad) in [
            ("F-F1.spread", "15"),
            ("F-F1.penalty_clock", "1:30"),
            ("F-X1.cases", "2379"),
            ("F-F1.area_factor", "1/3"),
            ("F-F1.squares", "4,10"),
            ("F-F1.distance", "3"),
        ] {
            assert!(
                formula(&with_fact(&f, key, bad)).is_err(),
                "a wrong {key} must be refused"
            );
        }
    }

    /// The square argument holds for every choice the finite check counts,
    /// not only for ours: the average square is the average gap squared plus
    /// the spread, the strips cancel, the spread is never negative and is
    /// nothing exactly when every gap is the same, and the wait is half the
    /// average gap plus the spread over twice the average gap. (The check is
    /// finite; the argument is the algebra this test runs on each case.)
    #[test]
    fn the_square_argument_holds_for_every_choice_checked() {
        fn walk(from: i64, left: usize, gaps: &mut Vec<i64>, seen: &mut u64) {
            if !gaps.is_empty() && gaps.iter().sum::<i64>() > 0 {
                *seen += 1;
                let m = model::mean_gap(gaps);
                let spread = model::spread(gaps);
                assert_eq!(model::mean_square(gaps), m * m + spread, "{gaps:?}");
                let strips: Q = gaps.iter().map(|&g| Q::int(g) - m).sum();
                assert_eq!(strips, Q::int(0), "{gaps:?}");
                assert!(spread >= Q::int(0), "{gaps:?}");
                assert_eq!(spread == Q::int(0), gaps.iter().all(|&g| g == gaps[0]), "{gaps:?}");
                let two = Q::int(2);
                assert_eq!(model::mean_wait(gaps), m / two + spread / (two * m), "{gaps:?}");
            }
            if left == 0 {
                return;
            }
            for g in from..=model::SWEEP_MAX_GAP {
                gaps.push(g);
                walk(g, left - 1, gaps, seen);
                gaps.pop();
            }
        }
        let mut seen = 0;
        walk(0, model::SWEEP_MAX_COUNT, &mut Vec::new(), &mut seen);
        let sweep = model::bound_sweep(model::SWEEP_MAX_GAP, model::SWEEP_MAX_COUNT);
        assert_eq!(seen, sweep.cases);
        // a strictly uneven pair waits more than half; an even pair, half
        assert!(model::mean_wait(&[1, 3]) > model::mean_gap(&[1, 3]) / Q::int(2));
        assert_eq!(model::mean_wait(&[4, 4]), model::mean_gap(&[4, 4]) / Q::int(2));
    }

    /// The finite check's count, recounted from its range: up to four gaps
    /// from 0 to 12 minutes give 2,375 choices once the all-zero choices
    /// are left out; from 1 minute they would give 1,819, so the card's
    /// lower end is 0 (zero gaps, buses together, are in the check).
    #[test]
    fn the_finite_check_counts_its_choices() {
        let sweep = model::bound_sweep(model::SWEEP_MAX_GAP, model::SWEEP_MAX_COUNT);
        let k = model::SWEEP_MAX_COUNT as u64;
        assert_eq!(choices(0, model::SWEEP_MAX_GAP, k), sweep.cases);
        assert_eq!(choices(0, model::SWEEP_MAX_GAP, k), 2375);
        assert_eq!(choices(1, model::SWEEP_MAX_GAP, k), 1819);
        assert_eq!(binomial(16, 4), 1820);
        assert_eq!((sweep.violations, sweep.equal_iff_even), (0, true));
    }

    /// The triangle and its mirror fill the square on each gap, and 08_4's
    /// four blocks tile the long gap's square: the average gap's square, two
    /// strips each the average gap by the distance, and the distance's
    /// square in the corner. The triangles are chapter 05's.
    #[test]
    fn the_squares_tile_the_long_square() {
        let twice_area = |c: &[(f64, f64)]| -> f64 {
            let k = c.len();
            (0..k)
                .map(|i| c[i].0 * c[(i + 1) % k].1 - c[(i + 1) % k].0 * c[i].1)
                .sum::<f64>()
                .abs()
        };
        let gs = twelve_minutes::gaps();
        for g in &gs {
            assert_eq!(triangle(g), super::super::triangles::triangle(g));
            let side = g.minutes() as f64;
            let both = twice_area(&triangle(g)) + twice_area(&mirror(g));
            assert!((both / 2.0 - side * side).abs() < 1e-12, "{g:?}");
            // the mirror's corners lie on the square's edges
            for (t, w) in mirror(g) {
                let on_edge = (t - g.to as f64).abs() < 1e-12 || (w - side).abs() < 1e-12;
                assert!(on_edge, "{g:?}");
            }
        }
        let long = gs[1];
        let m = model::mean_gap(&model::hour_gaps(OFFSET)).to_f64();
        let sq = squares(&long, m);
        let d = long.minutes() as f64 - m;
        let area = |b: &[(f64, f64); 4]| twice_area(b) / 2.0;
        assert!((area(&sq.mean) - m * m).abs() < 1e-12);
        assert!((area(&sq.right) - m * d).abs() < 1e-12);
        assert!((area(&sq.over) - m * d).abs() < 1e-12);
        assert!((area(&sq.corner) - d * d).abs() < 1e-12);
        let total = area(&sq.mean) + area(&sq.right) + area(&sq.over) + area(&sq.corner);
        let side = long.minutes() as f64;
        assert!((total - side * side).abs() < 1e-12);
        for b in [sq.mean, sq.right, sq.over, sq.corner] {
            for (t, w) in b {
                assert!((long.from as f64 - 1e-12..=long.to as f64 + 1e-12).contains(&t));
                assert!((-1e-12..=side + 1e-12).contains(&w));
            }
        }
        // the corner square's side is the distance, the facts' 4
        let p = formula(&facts::facts().unwrap()).unwrap();
        assert_eq!(format!("{d}"), p.distance);
    }

    /// Nothing the panel shows before 08_7 answers its question: no "≥", no
    /// "never", no "at least", no "half only".
    #[test]
    fn nothing_answers_the_question_before_it_is_asked() {
        let p = formula(&facts::facts().unwrap()).unwrap();
        let mut shown: Vec<String> = before_the_question(&p).iter().map(Row::text).collect();
        shown.extend([BRIDGE.to_owned(), WAIT_NAME.to_owned(), MOMENT_NAME.to_owned()]);
        for s in &shown {
            for answer in [AT_LEAST, "never", "at least", "half only", "negative"] {
                assert!(!s.contains(answer), "{s} answers the question early");
            }
        }
        assert!(p.bound_row().text().contains(AT_LEAST));
        assert!(NEVER.contains("never"));
    }

    /// The panel, the plot's labels, the sign, you, the question, the check
    /// card and K05 stay apart and on screen at each moment they are up
    /// together. Widths are estimated, not measured: 0.0045 units a point a
    /// character, a line 0.6 of its size tall.
    #[test]
    fn the_picture_stays_apart_and_on_screen() {
        let p = formula(&facts::facts().unwrap()).unwrap();
        let w = |s: &str, pt: f64| s.chars().count() as f64 * pt * 0.0045;
        let h = |pt: f64| pt * 0.0045 / 0.55 * 0.6;
        type Box4 = (f64, f64, f64, f64);
        let centred = |s: &str, pt: f64, at: Vec3| -> Box4 {
            (
                at[0] - w(s, pt) / 2.0,
                at[0] + w(s, pt) / 2.0,
                at[1] - h(pt),
                at[1] + h(pt),
            )
        };
        let left = |s: &str, pt: f64, at: Vec3| -> Box4 { (at[0], at[0] + w(s, pt), at[1] - h(pt), at[1] + h(pt)) };
        let apart = |a: Box4, b: Box4| a.1 < b.0 || b.1 < a.0 || a.3 < b.2 || b.3 < a.2;
        let on_screen = |a: Box4| a.0 > -7.1 && a.1 < 7.1 && a.2 > -4.0 && a.3 < 4.0;
        let fr = frame();
        // the plot: the stretch's frame, its rows under the line (clocks,
        // brackets and lengths, to about 0.92 under it), the wait axis's
        // labels and name, the moment's name under it all
        let plot = (fr.left(), fr.right(), fr.bottom() - 0.92, fr.top());
        let axis_labels = left("10", SMALL, v(ORIGIN_X - Y_LABEL_DX - 0.1, 0.0)).0;
        let top = at(0.0, fr.y.1)[1] + Y_OVERHANG;
        let wait_name = centred(WAIT_NAME, SMALL, v(ORIGIN_X, top + WAIT_NAME_DY));
        let moment = centred(MOMENT_NAME, SMALL, v(fr.center[0], MOMENT_NAME_Y));
        assert!(on_screen(wait_name) && on_screen(moment) && axis_labels > -7.1);
        assert!(moment.3 < plot.2, "the moment's name under the brackets' lengths");
        let row = |r: &Row, i: usize| left(&r.text(), PANEL_SIZE, panel(i));
        // a result row starts under the "=" of the row over it
        let indent = w(AVERAGE_WAIT, PANEL_SIZE) + SPACE * PANEL_SIZE;
        let indented = |r: &Row, i: usize| left(&r.text(), PANEL_SIZE, v(PANEL_LEFT + indent, row_y(i)));
        let bridge = left(BRIDGE, NOTE_SIZE, BRIDGE_AT);
        // the panel's moments: each set of rows up together
        let [sum_l, sum_r] = p.sum_rows();
        let moments: Vec<Vec<Box4>> = vec![
            // 08_1 and 08_2
            vec![
                row(&p.area_row(0), 0),
                row(&p.area_row(1), 1),
                row(&p.letter_row(), 2),
                (
                    PANEL_LEFT,
                    PANEL_LEFT + w(&sum_l.text(), PANEL_SIZE) + SPACE * PANEL_SIZE + w(&sum_r.text(), PANEL_SIZE),
                    row_y(3) - h(PANEL_SIZE),
                    row_y(3) + h(PANEL_SIZE),
                ),
                indented(&p.sum_result(), 4),
                row(&p.sum_letters(), 5),
                bridge,
            ],
            // 08_3
            vec![
                row(&p.letter_row(), 0),
                row(&p.sum_letters(), 1),
                row(&p.by_gaps(), 2),
                row(&p.top_row(), 3),
                row(&p.bottom_row(), 4),
                row(&p.mean_letters(), 5),
                indented(&p.mean_numbers(), 6),
                bridge,
            ],
            // 08_4 and its held argument
            vec![
                row(&p.mean_letters(), 0),
                indented(&p.mean_numbers(), 1),
                row(&p.square_row(0), 2),
                row(&p.square_row(1), 3),
                row(&p.average_square(), 4),
                row(&p.mean_squared(), 5),
                row(&p.extra_row(), 6),
                row(&p.distance_row(), 7),
                row(&p.argument_row(1), 8),
                row(&p.argument_row(0), 9),
                row(&p.cancel_row(), 10),
                row(&p.spread_name(), 11),
                row(&p.identity_row(), 12),
                bridge,
            ],
        ];
        for (k, up) in moments.iter().enumerate() {
            for (i, &a) in up.iter().enumerate() {
                assert!(on_screen(a), "moment {k}: row {i} off screen: {a:?}");
                assert!(
                    apart(a, plot) && apart(a, moment),
                    "moment {k}: row {i} on the plot: {a:?}"
                );
                for (j, &b) in up.iter().enumerate().skip(i + 1) {
                    assert!(apart(a, b), "moment {k}: rows {i} and {j} overlap: {a:?} {b:?}");
                }
            }
        }
        // 08_5 to 08_8: the panel with the box, the sign, you and your 1:20,
        // the question and its ring, the check card, K05's band (three
        // lines, `props::lower_third`: 0.4 in from each side, its foot 0.2
        // off the frame's, 0.3 + 0.36 a line tall)
        let boxed = {
            let b = row(&p.verdict_row(), 5);
            (b.0 - BOX_PAD[0], b.1 + BOX_PAD[0], b.2 - BOX_PAD[1], b.3 + BOX_PAD[1])
        };
        let panel_end = vec![
            row(&p.mean_letters(), 0),
            indented(&p.mean_numbers(), 1),
            row(&p.identity_row(), 2),
            row(&p.half_row(), 3),
            row(&p.penalty_row(), 4),
            boxed,
            row(&p.formula_row(), 6),
            row(&Row::new(Role::Spread, &[NEVER]), 7),
            left(WHO, WHO_SIZE, panel(8)),
            row(&p.bound_row(), 9),
            row(&Row::new(Role::Verdict, &[EXACTLY]), 10),
        ];
        let sign = (-6.15, -2.45, -2.25, -0.85);
        let me = (YOU_AT[0] - 0.16, YOU_AT[0] + 0.16, YOU_AT[1] - 0.16, YOU_AT[1] + 0.16);
        let morning = centred(
            &format!("{PLUS}{} {A_MORNING}", p.extra),
            MORNING_SIZE,
            v(YOU_AT[0], YOU_AT[1] + MORNING_DY),
        );
        let ask = centred(ASK, ASK_SIZE, ASK_AT);
        let ring = (
            THINK_AT[0] - 0.62,
            THINK_AT[0] + 0.62,
            THINK_AT[1] - 0.62,
            THINK_AT[1] + 0.62,
        );
        let note = centred("pause for more time", 18.0, v(THINK_AT[0], THINK_AT[1] - 0.62 - 0.3));
        let card = (CHECK_DL[0], CHECK_DL[0] + CHECK_W, CHECK_DL[1], CHECK_DL[1] + CHECK_H);
        let lines = |s: &str| explainer_kit::animatic::wrap(s, 92, 3).len() as f64;
        let k05 = (-6.71, 6.71, -3.8, -3.8 + 0.3 + 0.36 * lines(K05));
        let check_texts: Vec<Box4> = p
            .check_lines()
            .iter()
            .zip(CHECK_YS)
            .enumerate()
            .map(|(i, (s, y))| left(s, if i == 0 { CHECK_SIZES[0] } else { CHECK_SIZES[1] }, v(CHECK_X, y)))
            .collect();
        for t in &check_texts {
            assert!(t.0 > card.0 && t.1 < card.1 && t.2 > card.2 && t.3 < card.3, "{t:?}");
        }
        let late = [
            [panel_end.clone(), vec![sign, me, morning, ask, ring, note]].concat(),
            [panel_end.clone(), vec![sign, me, morning, card, k05]].concat(),
        ];
        for (k, up) in late.iter().enumerate() {
            for (i, &a) in up.iter().enumerate() {
                assert!(on_screen(a), "late {k}: item {i} off screen: {a:?}");
                for (j, &b) in up.iter().enumerate().skip(i + 1) {
                    assert!(apart(a, b), "late {k}: items {i} and {j} overlap: {a:?} {b:?}");
                }
            }
        }
    }

    /// The chapter's fixed words carry no digits: every number on screen is
    /// read from the facts. K05 alone is exempt: it is the draft's citation
    /// and formula word for word, checked against the draft above.
    #[test]
    fn the_fixed_words_carry_no_numbers() {
        for w in [
            AREA,
            SQUARED,
            ARROW,
            TIMES,
            DIVIDE,
            PLUS,
            MINUS,
            EQUALS,
            AT_LEAST,
            G,
            G_NAME,
            WAIT_NAME,
            MOMENT_NAME,
            BRIDGE,
            AVERAGE_WAIT,
            SUM_OF,
            AVERAGE_OF,
            AVERAGE_GAP,
            BY_GAPS,
            GAP_COUNT,
            TOP,
            BOTTOM,
            EACH_GAP,
            FROM,
            AVERAGE,
            STRIPS,
            SPREAD,
            SPREAD_NAME,
            ANY_GAPS,
            A_MORNING,
            ASK,
            NEVER,
            WHO,
            EXACTLY,
            CHECK_HEAD,
            CHECK_UP_TO,
            CHECK_GAPS,
            CHECK_EACH,
            CHECK_TO,
            CHECK_CHOICES,
            CHECK_BELOW,
            CHECK_EQUAL,
        ] {
            assert!(!w.chars().any(|c| c.is_ascii_digit()), "{w}");
        }
    }
}
