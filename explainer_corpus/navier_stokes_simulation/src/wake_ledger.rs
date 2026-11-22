//! Chapter 20's measured-only interface: the entries of the W1 table, read from a `wake-ledger01`
//! file. Standard library only.
//!
//! `tools/wake_ledger01.py` writes the file from root's run manifest (the `compare` manifest of
//! `tools/fluid_reference/wake_history.py`). It takes a run's numbers only from a `wake_history.py run`
//! analysis whose status is OK, whose input files still hash as analysed, and whose run directory
//! has a terminal, successful resource receipt (finished, exit 0, inputs unchanged) for the 2D-2
//! wake. Everything else is written as pending, with its reason. This module checks the file
//! again and refuses one that is malformed or inconsistent:
//!
//! - every role (grid1, grid2, grid3, step2, step3) appears once, in that order, as measured or
//!   pending; only a measured row has values, and it has all eight measurements;
//! - the rungs are a ladder: grid levels rise by one (cells half as wide, as 20_2 says), steps 2
//!   and 3 stay on grid 3's level, the step is the same over the grids and grows over steps 2 and 3;
//! - a value is finite, and positive where the quantity must be (counts are whole numbers);
//! - curve gaps exist only when a compare ran without refusal, and the 20_v1 slot only when the
//!   compare filled it, all five rows measured;
//! - W2 (a measured wake field) is pending: this package draws no field.
//!
//! An absent entry is `None` and is drawn as "?". There is no default value and no zero fill.

/// The file's first line.
pub const SCHEMA: &str = "wake-ledger01";

/// The table's runs in column order: `wake_history.py compare` roles.
pub const ROLES: [&str; 5] = ["grid1", "grid2", "grid3", "step2", "step3"];

/// The table's quantities in row order: `wake_history.py` `table_row` keys, the compare's curve
/// gaps against the reference (largest |difference| over one swing), and the costs.
pub const QUANTITIES: [&str; 14] = [
    "strouhal",
    "drag_peak",
    "drag_middle_ff",
    "drag_swing",
    "lift_peak",
    "lift_middle_ff",
    "lift_swing",
    "curve_drag_max_abs",
    "curve_lift_max_abs",
    "delta_p_st96",
    "cells_wet_leaves_max",
    "steps_run",
    "time_on_clock_s",
    "memory_max_rss_bytes",
];

/// The eight measurements every status-OK analysis has (indices into QUANTITIES).
pub const MEASUREMENTS: [usize; 8] = [0, 1, 2, 3, 4, 5, 6, 9];
/// The compare's curve gaps (indices into QUANTITIES).
pub const CURVES: [usize; 2] = [7, 8];
/// Whole-number quantities.
const COUNTS: [usize; 2] = [10, 11];
/// Quantities that must be strictly positive: the swing rate, the drag's peak and middle, the
/// lift's peak and swing, and every cost.
const POSITIVE: [usize; 9] = [0, 1, 2, 4, 6, 10, 11, 12, 13];
/// Quantities that may be zero but not negative.
const NON_NEGATIVE: [usize; 3] = [3, 7, 8];

/// The declared tolerances' keys (`close_enough` in the compare manifest) and the row each bounds.
pub const TOLERANCES: [(&str, usize); 10] = [
    ("strouhal", 0),
    ("drag_peak", 1),
    ("drag_middle_ff", 2),
    ("drag_swing", 3),
    ("lift_peak", 4),
    ("lift_middle_ff", 5),
    ("lift_swing", 6),
    ("curve_drag", 7),
    ("curve_lift", 8),
    ("delta_p_st96", 9),
];

/// The 20_v1 slot's values, as `wake_history.py compare` names them.
pub const SLOT_KEYS: [&str; 6] = [
    "20_v1.St",
    "20_v1.d1",
    "20_v1.d2",
    "20_v1.cells",
    "20_v1.steps",
    "20_v1.time_s",
];

/// One run of the table.
#[derive(Clone, Debug, PartialEq)]
pub enum Run {
    /// Not measured, with the reason (shown in the render log, never as a number).
    Pending(String),
    Measured(Box<Measured>),
}

/// A run measured from a terminal, compatible receipt.
#[derive(Clone, Debug, PartialEq)]
pub struct Measured {
    /// Basilisk level (cells per side of the 2.56 m root square are 2^level).
    pub level: u32,
    /// The nominal time step, in seconds.
    pub dt: f64,
    pub analysis_sha256: String,
    pub receipt_sha256: String,
    values: [Option<f64>; 14],
}

impl Measured {
    pub fn value(&self, quantity: usize) -> Option<f64> {
        self.values.get(quantity).copied().flatten()
    }
}

/// What the compare step said, if it ran.
#[derive(Clone, Debug, PartialEq)]
pub enum Compare {
    NotRun,
    /// Refused (its reasons); no table, no curves, no slot.
    Refused(String),
    /// Compared, slot withheld (no declared tolerance, or one not met).
    Withheld(String),
    /// Compared and every declared tolerance met: the 20_v1 slot is filled.
    Filled(String),
}

/// The 20_v1 slot ("Its Strouhal number came out at {St} …"), filled only by a compare that met
/// every declared tolerance. It is not voiced until root wires and records the line.
#[derive(Clone, Debug, PartialEq)]
pub struct Slot {
    pub values: [f64; 6],
}

#[derive(Clone, Debug, PartialEq)]
pub struct Ledger {
    /// Where the entries came from (the manifest's SHA-256, or why there is none).
    pub source: String,
    pub reference_sha256: Option<String>,
    runs: Vec<Run>,
    close_enough: [Option<f64>; 14],
    pub compare: Compare,
    pub slot: Option<Slot>,
    /// Why W2 (a measured wake field) is pending.
    pub w2: String,
}

impl Ledger {
    /// Every entry pending: what a render without a ledger file draws.
    pub fn pending(reason: &str) -> Self {
        Self {
            source: reason.to_owned(),
            reference_sha256: None,
            runs: ROLES.iter().map(|_| Run::Pending(reason.to_owned())).collect(),
            close_enough: [None; 14],
            compare: Compare::NotRun,
            slot: None,
            w2: "no W2 field in this package; the tunnel stays static".to_owned(),
        }
    }

    pub fn run(&self, role: usize) -> &Run {
        &self.runs[role]
    }

    /// The table cell for `role` (0..5) and `quantity` (0..14): `None` until measured.
    pub fn cell(&self, role: usize, quantity: usize) -> Option<f64> {
        match self.runs.get(role)? {
            Run::Measured(m) => m.value(quantity),
            Run::Pending(_) => None,
        }
    }

    /// The declared tolerance on `quantity`'s row, if root declared one.
    pub fn close_enough(&self, quantity: usize) -> Option<f64> {
        self.close_enough.get(quantity).copied().flatten()
    }

    pub fn measured_runs(&self) -> usize {
        self.runs.iter().filter(|r| matches!(r, Run::Measured(_))).count()
    }

    /// Read a `wake-ledger01` file, refusing anything malformed or inconsistent.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut lines = text
            .lines()
            .enumerate()
            .filter(|(_, l)| !l.trim().is_empty() && !l.starts_with('#'))
            .map(|(n, l)| (n + 1, l.split('\t').collect::<Vec<&str>>()));
        let bad = |n: usize, why: &str| format!("wake ledger line {n}: {why}");
        match lines.next() {
            Some((_, f)) if f == ["schema", SCHEMA] => {}
            Some((n, _)) => return Err(bad(n, "the first line must be `schema<TAB>wake-ledger01`")),
            None => return Err("wake ledger: empty file".to_owned()),
        }
        let mut ledger = Self {
            source: String::new(),
            reference_sha256: None,
            runs: Vec::new(),
            close_enough: [None; 14],
            compare: Compare::NotRun,
            slot: None,
            w2: String::new(),
        };
        let mut compare_seen = false;
        let mut slot = [None; 6];
        let mut w2_seen = false;
        for (n, f) in lines {
            match f.as_slice() {
                ["manifest", sha] if ledger.source.is_empty() => {
                    hex64(sha).map_err(|e| bad(n, &e))?;
                    ledger.source = format!("manifest sha256 {sha}");
                }
                ["reference", sha] if ledger.reference_sha256.is_none() => {
                    hex64(sha).map_err(|e| bad(n, &e))?;
                    ledger.reference_sha256 = Some((*sha).to_owned());
                }
                ["row", role, "pending", reason] => {
                    expect_role(&ledger, role).map_err(|e| bad(n, &e))?;
                    if reason.trim().is_empty() {
                        return Err(bad(n, "a pending row needs its reason"));
                    }
                    ledger.runs.push(Run::Pending((*reason).to_owned()));
                }
                ["row", role, "measured", level, dt, analysis, receipt] => {
                    expect_role(&ledger, role).map_err(|e| bad(n, &e))?;
                    let level: u32 = level.parse().map_err(|_| bad(n, "level is not a whole number"))?;
                    let dt = number(dt).map_err(|e| bad(n, &e))?;
                    if dt <= 0.0 {
                        return Err(bad(n, "the time step must be positive"));
                    }
                    hex64(analysis).map_err(|e| bad(n, &e))?;
                    hex64(receipt).map_err(|e| bad(n, &e))?;
                    ledger.runs.push(Run::Measured(Box::new(Measured {
                        level,
                        dt,
                        analysis_sha256: (*analysis).to_owned(),
                        receipt_sha256: (*receipt).to_owned(),
                        values: [None; 14],
                    })));
                }
                ["value", role, quantity, value] => {
                    let q = QUANTITIES
                        .iter()
                        .position(|k| k == quantity)
                        .ok_or_else(|| bad(n, &format!("unknown quantity {quantity}")))?;
                    let x = checked(q, value).map_err(|e| bad(n, &e))?;
                    if CURVES.contains(&q) && !matches!(ledger.compare, Compare::Withheld(_) | Compare::Filled(_)) {
                        return Err(bad(
                            n,
                            "a curve gap needs a compare that ran without refusal, listed first",
                        ));
                    }
                    if ledger.runs.is_empty() || ROLES[ledger.runs.len() - 1] != *role {
                        return Err(bad(n, "values follow their own measured row"));
                    }
                    let Some(Run::Measured(m)) = ledger.runs.last_mut() else {
                        return Err(bad(n, "values follow their own measured row"));
                    };
                    if m.values[q].replace(x).is_some() {
                        return Err(bad(n, &format!("{role} {quantity} given twice")));
                    }
                }
                ["close_enough", key, value] if ledger.runs.is_empty() => {
                    let q = TOLERANCES
                        .iter()
                        .find(|(k, _)| k == key)
                        .map(|&(_, q)| q)
                        .ok_or_else(|| bad(n, &format!("unknown tolerance {key}")))?;
                    let x = number(value).map_err(|e| bad(n, &e))?;
                    if x <= 0.0 {
                        return Err(bad(n, "a tolerance must be positive"));
                    }
                    if ledger.close_enough[q].replace(x).is_some() {
                        return Err(bad(n, &format!("tolerance {key} given twice")));
                    }
                }
                ["compare", state, detail] if !compare_seen && ledger.runs.is_empty() => {
                    compare_seen = true;
                    let detail = (*detail).to_owned();
                    ledger.compare = match *state {
                        "none" => Compare::NotRun,
                        "refused" => Compare::Refused(detail),
                        "withheld" => Compare::Withheld(detail),
                        "filled" => Compare::Filled(detail),
                        _ => return Err(bad(n, "compare is none, refused, withheld or filled")),
                    };
                }
                ["slot", key, value] => {
                    let i = SLOT_KEYS
                        .iter()
                        .position(|k| k == key)
                        .ok_or_else(|| bad(n, &format!("unknown slot {key}")))?;
                    if !matches!(ledger.compare, Compare::Filled(_)) {
                        return Err(bad(n, "a slot value needs a compare that filled it"));
                    }
                    if slot[i].replace(number(value).map_err(|e| bad(n, &e))?).is_some() {
                        return Err(bad(n, &format!("slot {key} given twice")));
                    }
                }
                ["w2", "pending", reason] if !w2_seen && !reason.trim().is_empty() => {
                    w2_seen = true;
                    ledger.w2 = (*reason).to_owned();
                }
                ["w2", ..] => return Err(bad(n, "W2 stays pending here, once, with its reason")),
                _ => return Err(bad(n, &format!("unexpected line {:?}", f.join("\t")))),
            }
        }
        if ledger.source.is_empty() {
            return Err("wake ledger: no manifest line".to_owned());
        }
        if ledger.runs.len() != ROLES.len() {
            return Err(format!("wake ledger: {} of the 5 rows", ledger.runs.len()));
        }
        if !w2_seen {
            return Err("wake ledger: no w2 line".to_owned());
        }
        if !compare_seen {
            return Err("wake ledger: no compare line".to_owned());
        }
        for (role, run) in ROLES.iter().zip(&ledger.runs) {
            if let Run::Measured(m) = run {
                let missing: Vec<&str> = MEASUREMENTS
                    .iter()
                    .filter(|&&q| m.values[q].is_none())
                    .map(|&q| QUANTITIES[q])
                    .collect();
                if !missing.is_empty() {
                    return Err(format!(
                        "wake ledger: {role} is measured without {}",
                        missing.join(", ")
                    ));
                }
            }
        }
        ladder(&ledger.runs)?;
        match (
            &ledger.compare,
            slot.iter().all(Option::is_some),
            slot.iter().any(Option::is_some),
        ) {
            (Compare::Filled(_), true, _) => {
                if ledger.measured_runs() != ROLES.len() {
                    return Err("wake ledger: a filled slot needs all five rows measured".to_owned());
                }
                ledger.slot = Some(Slot {
                    values: slot.map(|x| x.unwrap_or(f64::NAN)),
                });
            }
            (Compare::Filled(_), false, _) => {
                return Err("wake ledger: a filled compare lists all six slots".to_owned());
            }
            (_, _, true) => return Err("wake ledger: slot values without a filled compare".to_owned()),
            _ => {}
        }
        if matches!(ledger.compare, Compare::Withheld(_) | Compare::Filled(_)) && ledger.measured_runs() != ROLES.len()
        {
            return Err("wake ledger: a compare that ran needs all five rows measured".to_owned());
        }
        Ok(ledger)
    }
}

fn expect_role(ledger: &Ledger, role: &str) -> Result<(), String> {
    match ROLES.get(ledger.runs.len()) {
        Some(want) if *want == role => Ok(()),
        Some(want) => Err(format!("expected row {want}, found {role}")),
        None => Err(format!("a sixth row ({role})")),
    }
}

fn hex64(s: &str) -> Result<(), String> {
    if s.len() == 64 && s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
        Ok(())
    } else {
        Err(format!("{s:?} is not a lowercase SHA-256"))
    }
}

/// A finite decimal number (Python's repr of a float, or an integer).
fn number(s: &str) -> Result<f64, String> {
    let ok = !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit() || b"+-.eE".contains(&b));
    let x: f64 = if ok {
        s.parse().map_err(|_| format!("{s:?} is not a number"))?
    } else {
        f64::NAN
    };
    if x.is_finite() {
        Ok(x)
    } else {
        Err(format!("{s:?} is not a finite number"))
    }
}

fn checked(q: usize, s: &str) -> Result<f64, String> {
    let x = number(s)?;
    if POSITIVE.contains(&q) && x <= 0.0 {
        return Err(format!("{} must be positive, not {s}", QUANTITIES[q]));
    }
    if NON_NEGATIVE.contains(&q) && x < 0.0 {
        return Err(format!("{} cannot be negative ({s})", QUANTITIES[q]));
    }
    if COUNTS.contains(&q) && x.fract() != 0.0 {
        return Err(format!("{} must be a whole number, not {s}", QUANTITIES[q]));
    }
    Ok(x)
}

/// The rungs' settings among the measured rows: grid levels rise by one, steps 2 and 3 stay on
/// grid 3's level; the step is equal over the grids and grows over steps 2 and 3.
fn ladder(runs: &[Run]) -> Result<(), String> {
    let m = |i: usize| match &runs[i] {
        Run::Measured(m) => Some((m.level, m.dt)),
        Run::Pending(_) => None,
    };
    let mut wrong = Vec::new();
    for (a, b) in [(0, 1), (1, 2)] {
        if let (Some(x), Some(y)) = (m(a), m(b))
            && (y.0 != x.0 + 1 || y.1 != x.1)
        {
            wrong.push(format!("{} to {}: level +1 at the same step", ROLES[a], ROLES[b]));
        }
    }
    if let (Some(x), Some(y)) = (m(0), m(2))
        && (y.0 != x.0 + 2 || y.1 != x.1)
    {
        wrong.push("grid1 to grid3: level +2 at the same step".to_owned());
    }
    for b in [3, 4] {
        for (a, grid) in ROLES.iter().enumerate().take(3) {
            if let (Some(x), Some(y)) = (m(a), m(b))
                && (y.0 != x.0 + 2 - a as u32 || y.1 <= x.1)
            {
                wrong.push(format!("{grid} to {}: grid 3's level at a longer step", ROLES[b]));
            }
        }
    }
    if let (Some(x), Some(y)) = (m(3), m(4))
        && (y.0 != x.0 || y.1 <= x.1)
    {
        wrong.push("step2 to step3: the same level at a longer step".to_owned());
    }
    if wrong.is_empty() {
        Ok(())
    } else {
        Err(format!("wake ledger: not a ladder: {}", wrong.join("; ")))
    }
}

// ------------------------------------------------------------------------------- display

/// A measured value as the table shows it. Only measured values reach this; a pending entry is
/// drawn as "?" by the caller. A nonzero value never prints as zero.
pub fn shown(quantity: usize, x: f64) -> String {
    match QUANTITIES[quantity] {
        "cells_wet_leaves_max" | "steps_run" => thousands(x),
        "time_on_clock_s" => clock(x),
        "memory_max_rss_bytes" => format!("{} GB", decimals(x / 1e9, 2)),
        _ => decimals(x, 4),
    }
}

/// A declared tolerance: "≤ 0.002".
pub fn shown_tolerance(x: f64) -> String {
    format!("≤ {}", decimals(x, 4))
}

/// `places` decimals, with a true minus sign; a nonzero value too small for that many places is
/// written as m × 10⁻ⁿ instead of rounding to zero.
pub fn decimals(x: f64, places: usize) -> String {
    let sign = if x < 0.0 { "−" } else { "" };
    let a = x.abs();
    let fixed = format!("{a:.places$}");
    if a != 0.0 && fixed.bytes().all(|b| b == b'0' || b == b'.') {
        let e = format!("{a:.1e}");
        return match e.split_once('e') {
            Some((mantissa, exponent)) => format!("{sign}{mantissa} × 10{}", superscript(exponent)),
            None => format!("{sign}{e}"),
        };
    }
    format!("{sign}{fixed}")
}

fn superscript(exponent: &str) -> String {
    exponent
        .chars()
        .map(|c| match c {
            '-' => '⁻',
            '0' => '⁰',
            '1' => '¹',
            '2' => '²',
            '3' => '³',
            '4' => '⁴',
            '5' => '⁵',
            '6' => '⁶',
            '7' => '⁷',
            '8' => '⁸',
            '9' => '⁹',
            other => other,
        })
        .collect()
}

fn thousands(x: f64) -> String {
    let digits = format!("{:.0}", x.abs());
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// Seconds on the clock: "42 s", "12 min 05 s" or "3 h 07 min".
fn clock(seconds: f64) -> String {
    let s = seconds.round() as u64;
    if s == 0 {
        return format!("{} s", decimals(seconds, 2));
    }
    match s {
        0..60 => format!("{s} s"),
        60..3600 => format!("{} min {:02} s", s / 60, s % 60),
        _ => format!("{} h {:02} min", s / 3600, (s % 3600 + 30) / 60),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn measured_row(role: &str, level: u32, dt: &str) -> String {
        let mut out = format!("row\t{role}\tmeasured\t{level}\t{dt}\t{A}\t{B}\n");
        for (q, v) in [
            ("strouhal", "0.3"),
            ("drag_peak", "3.2"),
            ("drag_middle_ff", "3.19"),
            ("drag_swing", "0.06"),
            ("lift_peak", "0.98"),
            ("lift_middle_ff", "-0.017"),
            ("lift_swing", "2.0"),
            ("delta_p_st96", "2.48"),
            ("cells_wet_leaves_max", "12000"),
        ] {
            out += &format!("value\t{role}\t{q}\t{v}\n");
        }
        out
    }

    fn head() -> String {
        format!("schema\twake-ledger01\nmanifest\t{A}\nreference\t{B}\ncompare\tnone\tnot run\n")
    }

    fn pending_rest(from: usize) -> String {
        ROLES[from..]
            .iter()
            .map(|r| format!("row\t{r}\tpending\tno run assigned\n"))
            .collect()
    }

    #[test]
    fn pending_ledger_has_no_values() {
        let l = Ledger::pending("no ledger given");
        assert!((0..5).all(|r| (0..14).all(|q| l.cell(r, q).is_none())));
        assert!((0..14).all(|q| l.close_enough(q).is_none()));
        assert!(l.slot.is_none());
    }

    #[test]
    fn partial_ledger_parses() {
        let text = head() + &measured_row("grid1", 8, "0.0025") + &pending_rest(1) + "w2\tpending\tstatic\n";
        let l = Ledger::parse(&text).expect("a valid partial ledger");
        assert_eq!(l.measured_runs(), 1);
        assert_eq!(l.cell(0, 0), Some(0.3));
        assert_eq!(l.cell(0, 13), None);
        assert_eq!(l.cell(1, 0), None);
        assert_eq!(shown(5, l.cell(0, 5).expect("lift middle")), "−0.0170");
        assert_eq!(shown(10, 12000.0), "12,000");
    }

    #[test]
    fn refuses_what_is_not_measured() {
        let w2 = "w2\tpending\tstatic\n";
        let ok = head() + &measured_row("grid1", 8, "0.0025") + &pending_rest(1) + w2;
        assert!(Ledger::parse(&ok).is_ok());
        // A missing measurement, a zero swing rate, a non-finite value, a wrong order, a broken ladder.
        let cases = [
            ok.replace("value\tgrid1\tlift_swing\t2.0\n", ""),
            ok.replace("strouhal\t0.3", "strouhal\t0"),
            ok.replace("drag_peak\t3.2", "drag_peak\tnan"),
            ok.replace("row\tgrid1", "row\tgrid2"),
            head() + &measured_row("grid1", 8, "0.0025") + &measured_row("grid2", 10, "0.0025") + &pending_rest(2) + w2,
            head() + &measured_row("grid1", 8, "0.0025") + &measured_row("grid2", 9, "0.005") + &pending_rest(2) + w2,
            ok.replace("compare\tnone\tnot run\n", "compare\tfilled\tx\nslot\t20_v1.St\t0.3\n"),
            ok.replace(w2, "w2\tmeasured\tfield\n"),
            ok.replace("cells_wet_leaves_max\t12000", "cells_wet_leaves_max\t12000.5"),
        ];
        for (i, case) in cases.iter().enumerate() {
            assert!(Ledger::parse(case).is_err(), "case {i} should be refused");
        }
    }

    #[test]
    fn small_values_never_show_as_zero() {
        assert_eq!(decimals(0.00004, 4), "4.0 × 10⁻⁵");
        assert_eq!(decimals(-0.0174, 4), "−0.0174");
        assert_eq!(shown_tolerance(0.002), "≤ 0.0020");
        assert_eq!(clock(42.0), "42 s");
        assert_eq!(clock(725.0), "12 min 05 s");
        assert_eq!(clock(11_220.0), "3 h 07 min");
        assert_eq!(shown(13, 1_234_000_000.0), "1.23 GB");
    }

    /// Ledgers tools/wake_ledger01.py wrote: three from its self-test fixtures (scratch numbers,
    /// not measurements) and one from root's commission analysis, which is INSUFFICIENT.
    #[test]
    fn parses_the_adapters_ledgers() {
        let filled = Ledger::parse(include_str!("testdata/ledger-filled01.tsv")).expect("filled");
        assert_eq!(filled.measured_runs(), 5);
        assert!(matches!(filled.compare, Compare::Filled(_)));
        assert!(filled.slot.is_some() && filled.cell(2, 7).is_some() && filled.cell(0, 13).is_none());
        assert_eq!(filled.close_enough(0), Some(0.01));
        let partial = Ledger::parse(include_str!("testdata/ledger-partial01.tsv")).expect("partial");
        assert_eq!(partial.measured_runs(), 3);
        assert!(partial.cell(2, 0).is_some() && partial.cell(2, 7).is_none() && partial.cell(3, 0).is_none());
        let demoted = Ledger::parse(include_str!("testdata/ledger-demoted01.tsv")).expect("demoted");
        assert!(matches!(demoted.compare, Compare::Refused(_)) && demoted.measured_runs() == 4);
        let probe = Ledger::parse(include_str!("testdata/ledger-commission-probe01.tsv")).expect("probe");
        assert_eq!(probe.measured_runs(), 0);
        assert!(matches!(probe.run(0), Run::Pending(why) if why.contains("INSUFFICIENT")));
    }
}
