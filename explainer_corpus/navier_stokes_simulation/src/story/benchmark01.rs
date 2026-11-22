//! Chapter 17's tail (cues 17_8 to 17_15): every number it shows, exact and std only.
//!
//! RubyRaven `ruby-boundary-tail01` for root's `navier_stokes_simulation` crate (Agent Mail 36228).
//! All values are Schäfer and Turek (1996), "Benchmark computations of laminar flow around a
//! cylinder" (ST96), as printed: §2.1 and §2.2 for the fluid and the inflow of test 2D-2,
//! Table 4 (method 7a) for the Strouhal numbers, and §4 for the measurement and the run with a
//! longer inlet. Four-place values are held as integers in ten-thousandths, so every difference
//! the scene shows is the exact difference of the values it joins. `tools/check_tail01.py` reads
//! the same numbers from the ST96 text itself and recomputes everything below in Fractions.
//!
//! None of these results is ours: method 7a is one 1996 group's solver, and the measurement and
//! the longer-inlet run are reported in the same paper. Our own wake benchmark is not yet run.

/// A value with four decimal places, held in ten-thousandths (0.2968 is `Tenk(2968)`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tenk(pub i64);

impl Tenk {
    /// "0.2968": four places, as Table 4 prints them. Values here are never negative.
    pub fn four(self) -> String {
        assert!(self.0 >= 0, "negative value {self:?}");
        format!("{}.{:04}", self.0 / 10_000, self.0 % 10_000)
    }

    /// "+0.0026": a change, signed, at four places.
    pub fn signed(self) -> String {
        let sign = if self.0 < 0 { "−" } else { "+" };
        format!("{sign}{}", Tenk(self.0.abs()).four())
    }

    /// "0.287": three places, or `None` if the fourth place is not zero.
    pub fn three(self) -> Option<String> {
        (self.0 >= 0 && self.0 % 10 == 0).then(|| format!("{}.{:03}", self.0 / 10_000, self.0 % 10_000 / 10))
    }
}

/// One row of Table 4 for method 7a: unknowns, the table's time column, and the Strouhal number.
/// Table 4 heads its two unknowns columns "Space" and "Time", so the time count is shown as
/// "time unknowns": more is finer. The retained text defines it no further.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Run {
    pub unknowns: u32,
    pub time: u32,
    pub st: Tenk,
}

/// Method 7a's three grids at 36 in the time column, coarse to fine (Table 4, rows
/// "18432 36 … 0.2968", "73728 36 … 0.2994", "294912 36 … 0.3003").
pub const GRIDS: [Run; 3] = [
    Run {
        unknowns: 18_432,
        time: 36,
        st: Tenk(2968),
    },
    Run {
        unknowns: 73_728,
        time: 36,
        st: Tenk(2994),
    },
    Run {
        unknowns: 294_912,
        time: 36,
        st: Tenk(3003),
    },
];

/// Its finest grid at 10, 19 and 36 in the time column, coarse to fine (Table 4).
pub const STEPS: [Run; 3] = [
    Run {
        unknowns: 294_912,
        time: 10,
        st: Tenk(2881),
    },
    Run {
        unknowns: 294_912,
        time: 19,
        st: Tenk(2973),
    },
    Run {
        unknowns: 294_912,
        time: 36,
        st: Tenk(3003),
    },
];

/// The measured Strouhal number, St = 0.287 ± 0.003 (ST96 §4).
pub const MEASURED: Tenk = Tenk(2870);
pub const MEASURED_PM: Tenk = Tenk(30);

/// "a computation with a longer distance between the inlet and cylinder", St = 0.289 (ST96 §4).
/// The paper links it to method 7a but gives neither its grid nor its time step.
pub const LONGER_INLET: Tenk = Tenk(2890);

/// The measured range, 0.284 to 0.290.
pub fn band() -> (Tenk, Tenk) {
    (Tenk(MEASURED.0 - MEASURED_PM.0), Tenk(MEASURED.0 + MEASURED_PM.0))
}

pub fn inside_band(v: Tenk) -> bool {
    let (lo, hi) = band();
    lo <= v && v <= hi
}

/// The change from each run to the next.
pub fn changes(runs: &[Run]) -> Vec<Tenk> {
    runs.windows(2).map(|w| Tenk(w[1].st.0 - w[0].st.0)).collect()
}

/// From the finest grid's value down to the measurement: 0.3003 − 0.287 = 0.0133.
pub fn gap() -> Tenk {
    Tenk(GRIDS[2].st.0 - MEASURED.0)
}

/// The gap in units of the last grid change, rounded half up to a whole number (133/9 → 15).
pub fn gap_in_last_changes() -> i64 {
    let last = changes(&GRIDS)[1].0;
    (2 * gap().0 + last) / (2 * last)
}

/// "0.0133 ≈ 15 × 0.0009".
pub fn gap_label() -> String {
    format!(
        "{} ≈ {} × {}",
        gap().four(),
        gap_in_last_changes(),
        changes(&GRIDS)[1].four()
    )
}

/// "measured: 0.287 ± 0.003".
pub fn band_label() -> String {
    let three = |t: Tenk| t.three().expect("the measurement has three places");
    format!("measured: {} ± {}", three(MEASURED), three(MEASURED_PM))
}

/// "18,432 → 73,728 → 294,912 unknowns".
pub fn grids_label() -> String {
    let parts: Vec<String> = GRIDS.iter().map(|r| thousands(r.unknowns)).collect();
    format!("{} unknowns", parts.join(" → "))
}

/// "10 → 19 → 36 time unknowns", read like the grids' "… unknowns" above it.
pub fn steps_label() -> String {
    let parts: Vec<String> = STEPS.iter().map(|r| r.time.to_string()).collect();
    format!("{} time unknowns", parts.join(" → "))
}

/// Each grid's unknowns over the last one's, exactly, or `None` if not a whole number.
pub fn unknowns_ratios() -> Vec<Option<u32>> {
    GRIDS
        .windows(2)
        .map(|w| (w[1].unknowns % w[0].unknowns == 0).then(|| w[1].unknowns / w[0].unknowns))
        .collect()
}

pub fn thousands(n: u32) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (k, ch) in digits.chars().enumerate() {
        if k > 0 && (digits.len() - k).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 { a.abs() } else { gcd(b, a % b) }
}

/// A fraction in lowest terms with a positive denominator.
pub fn reduce(n: i64, d: i64) -> (i64, i64) {
    assert!(d != 0, "zero denominator");
    let g = gcd(n, d) * d.signum();
    (n / g, d / g)
}

/// A fraction with a terminating decimal, written out exactly ("0.1", "1.5", "100"); `None` if
/// its denominator does not divide a power of ten (1/3).
pub fn decimal((n, d): (i64, i64)) -> Option<String> {
    let (n, d) = reduce(n, d);
    let places = (0..19).find(|&k| 10_i64.pow(k) % d == 0)? as usize;
    let n = n * (10_i64.pow(places as u32) / d);
    if places == 0 {
        return Some(n.to_string());
    }
    let s = format!("{:0width$}", n.abs(), width = places + 1);
    let (whole, frac) = s.split_at(s.len() - places);
    Some(format!("{}{whole}.{frac}", if n < 0 { "−" } else { "" }))
}

/// "peak 1.5 m/s".
pub fn peak_label() -> String {
    format!("peak {} m/s", decimal(inlet::PEAK).expect("1.5 terminates"))
}

/// "average 1 m/s".
pub fn average_label() -> String {
    format!("average {} m/s", decimal(inlet::mean()).expect("1 terminates"))
}

/// The 17_9 formula card, in TeX, from the exact values: Re = Ū D / ν = 1 × 0.1 / 0.001 = 100.
pub fn re_tex() -> String {
    let d = |f: (i64, i64)| decimal(f).expect("terminating");
    format!(
        r"\mathrm{{Re}} = \bar{{U}} D / \nu = {} \times {} / {} = {}",
        d(inlet::mean()),
        d(fluid::DIAMETER),
        d(fluid::NU),
        d(fluid::reynolds())
    )
}

/// The 17_9 fluid tag, in TeX, from the exact values: ν = 10⁻³ m²/s, ρ = 1 kg/m³ (ST96 §2.1).
pub fn fluid_tex() -> String {
    let (n, d) = fluid::NU;
    let k = (0..19)
        .find(|&k| n == 1 && 10i64.pow(k) == d)
        .expect("ν is a power of ten");
    format!(
        r"\text{{fluid: }}\nu = 10^{{-{k}}}\,\mathrm{{m^2/s}},\quad \rho = {}\,\mathrm{{kg/m^3}}",
        decimal(fluid::DENSITY).expect("terminating")
    )
}

/// ΔP's two points, metres from the inlet's lower corner (ST96 §2.2): the front and the end of
/// the cylinder, (xa, ya) = (0.15, 0.2) and (xe, ye) = (0.25, 0.2); ΔP = P(front) − P(end).
pub const PRESSURE_POINTS: [(f64, f64); 2] = [(0.15, 0.2), (0.25, 0.2)];

/// The 2D-2 inflow (ST96 §2.2): U(0, y) = 4 Um y (H − y) / H², V = 0, with Um = 1.5 m/s.
pub mod inlet {
    use super::reduce;

    /// H, m: the channel height (ST96 Fig. 1).
    pub const HEIGHT: f64 = 0.41;
    /// Um, m/s, the peak, as a fraction.
    pub const PEAK: (i64, i64) = (3, 2);

    /// U at height share s = y / H, m/s: 4 Um s (1 − s).
    pub fn speed_at_share(s: f64) -> f64 {
        4.0 * PEAK.0 as f64 / PEAK.1 as f64 * s * (1.0 - s)
    }

    /// The inflow's average over the inlet, exactly. Simpson's rule is exact for a quadratic:
    /// (U(0) + 4 U(H/2) + U(H)) / 6 = 4 Um / 6 = 2 Um / 3.
    pub fn mean() -> (i64, i64) {
        reduce(4 * PEAK.0, 6 * PEAK.1)
    }

    /// The paper's mean velocity, 2 U(0, H/2) / 3, exactly; U(0, H/2) = Um.
    pub fn paper_mean() -> (i64, i64) {
        reduce(2 * PEAK.0, 3 * PEAK.1)
    }
}

/// The fluid and the Reynolds number (ST96 §2.1: ν = 10⁻³ m²/s, ρ = 1.0 kg/m³; D = 0.1 m).
pub mod fluid {
    use super::{inlet, reduce};

    pub const NU: (i64, i64) = (1, 1000);
    pub const DENSITY: (i64, i64) = (1, 1);
    pub const DIAMETER: (i64, i64) = (1, 10);

    /// Re = Ū D / ν, exactly.
    pub fn reynolds() -> (i64, i64) {
        let (u, d, nu) = (inlet::mean(), DIAMETER, NU);
        reduce(u.0 * d.0 * nu.1, u.1 * d.1 * nu.0)
    }
}

/// A horizontal number line in scene units.
#[derive(Clone, Copy, Debug)]
pub struct NumberLine {
    pub lo: Tenk,
    pub hi: Tenk,
    pub x0: f64,
    pub x1: f64,
}

/// The chapter's Strouhal line: 0.2820 at x = −6 to 0.3020 at x = 6, 0.06 units per 0.0001.
pub const LINE: NumberLine = NumberLine {
    lo: Tenk(2820),
    hi: Tenk(3020),
    x0: -6.0,
    x1: 6.0,
};

impl NumberLine {
    pub fn x(&self, v: Tenk) -> f64 {
        self.x0 + (v.0 - self.lo.0) as f64 * (self.x1 - self.x0) / (self.hi.0 - self.lo.0) as f64
    }

    /// Multiples of `step` strictly inside the line.
    pub fn ticks(&self, step: i64) -> Vec<Tenk> {
        let first = self.lo.0 / step + 1;
        (first..).map(|k| Tenk(k * step)).take_while(|t| *t < self.hi).collect()
    }

    pub fn contains(&self, v: Tenk) -> bool {
        self.lo < v && v < self.hi
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_4_values() {
        let st: Vec<i64> = GRIDS.iter().chain(&STEPS).map(|r| r.st.0).collect();
        assert_eq!(st, [2968, 2994, 3003, 2881, 2973, 3003]);
        assert_eq!(GRIDS[2], STEPS[2], "the finest run is in both rows");
        assert!(GRIDS.iter().all(|r| r.time == 36));
        assert!(STEPS.iter().all(|r| r.unknowns == 294_912));
        assert_eq!(unknowns_ratios(), [Some(4), Some(4)]);
    }

    #[test]
    fn changes_shrink_and_the_time_row_rises() {
        assert_eq!(changes(&GRIDS), [Tenk(26), Tenk(9)]);
        assert_eq!(
            changes(&GRIDS).iter().map(|c| c.signed()).collect::<Vec<_>>(),
            ["+0.0026", "+0.0009"]
        );
        let up = changes(&STEPS);
        assert_eq!(up, [Tenk(92), Tenk(30)]);
        assert!(up.iter().all(|c| c.0 > 0), "finer time steps raised the value");
        assert!(
            up[1] > changes(&GRIDS)[1],
            "19 to 36 moved it more than the last grid change"
        );
        assert_eq!(Tenk(STEPS[2].st.0 - STEPS[0].st.0), Tenk(122));
    }

    #[test]
    fn spoken_numbers() {
        let [coarse, mid, fine] = GRIDS.map(|r| r.st.0);
        assert_eq!((coarse + 5) / 10, 297, "near point two nine seven");
        assert!((2990..3000).contains(&mid), "just under point three");
        assert!(fine > 3000 && fine <= 3010, "then just over it");
        let [first, last] = [changes(&GRIDS)[0].0, changes(&GRIDS)[1].0];
        assert!((23..=27).contains(&first), "about two and a half thousandths");
        assert!(last < 10, "under one thousandth");
        assert_eq!((MEASURED, MEASURED_PM), (Tenk(2870), Tenk(30)));
    }

    #[test]
    fn the_measurement_and_the_longer_inlet() {
        assert_eq!(band(), (Tenk(2840), Tenk(2900)));
        assert!(inside_band(LONGER_INLET), "0.289 lies inside 0.287 ± 0.003");
        assert!(GRIDS[2].st > band().1, "the standard channel's 0.3003 lies above 0.290");
        assert!(
            inside_band(STEPS[0].st),
            "the 10-step value lies inside: agreement by chance"
        );
        assert_eq!(gap(), Tenk(133));
        // 133 / 9 = 14.78: about fifteen times the last grid change
        let (g, last) = (gap().0, changes(&GRIDS)[1].0);
        assert_eq!((g, last), (133, 9));
        assert!(14 * last < g && g < 15 * last && 2 * g > 29 * last);
        assert_eq!(gap_in_last_changes(), 15);
        assert_eq!(gap_label(), "0.0133 ≈ 15 × 0.0009");
        assert_eq!(band_label(), "measured: 0.287 ± 0.003");
        assert_eq!(LONGER_INLET.three().as_deref(), Some("0.289"));
        assert_eq!(Tenk(2881).three(), None);
    }

    #[test]
    fn row_labels() {
        assert_eq!(grids_label(), "18,432 → 73,728 → 294,912 unknowns");
        assert_eq!(steps_label(), "10 → 19 → 36 time unknowns");
        assert_eq!(thousands(294_912), "294,912");
        assert_eq!(thousands(912), "912");
    }

    #[test]
    fn inflow_and_reynolds_number() {
        assert_eq!(inlet::mean(), (1, 1), "the average over the inlet is 1 m/s");
        assert_eq!(inlet::paper_mean(), (1, 1), "2 U(0, H/2) / 3 = 1 m/s");
        assert_eq!(inlet::speed_at_share(0.5), 1.5);
        assert_eq!(inlet::speed_at_share(0.0), 0.0);
        assert_eq!(inlet::speed_at_share(1.0), 0.0);
        // Simpson on the drawn profile, numerically, as a cross-check of the exact mean
        let n = 1000;
        let h = 1.0 / n as f64;
        let mut sum = inlet::speed_at_share(0.0) + inlet::speed_at_share(1.0);
        for k in 1..n {
            sum += if k % 2 == 1 { 4.0 } else { 2.0 } * inlet::speed_at_share(k as f64 * h);
        }
        assert!((sum * h / 3.0 - 1.0).abs() < 1e-12);
        assert_eq!(fluid::reynolds(), (100, 1));
        // ΔP's points sit on the cylinder (centre (0.2, 0.2), radius 0.05), front then end
        for (x, y) in PRESSURE_POINTS {
            assert!(((x - 0.2f64).hypot(y - 0.2) - 0.05).abs() < 1e-12);
        }
        assert!(PRESSURE_POINTS[0].0 < PRESSURE_POINTS[1].0);
        assert_eq!(decimal(fluid::DIAMETER).as_deref(), Some("0.1"));
        assert_eq!(decimal(fluid::NU).as_deref(), Some("0.001"));
        assert_eq!(decimal(inlet::mean()).as_deref(), Some("1"));
        assert_eq!(decimal(fluid::reynolds()).as_deref(), Some("100"));
        assert_eq!(decimal(inlet::PEAK).as_deref(), Some("1.5"));
        assert_eq!(decimal((3, 2 * 5)).as_deref(), Some("0.3"));
        assert_eq!(decimal((1, 3)).as_deref(), None);
        assert_eq!(decimal((-7, 4)).as_deref(), Some("−1.75"));
        assert_eq!(peak_label(), "peak 1.5 m/s");
        assert_eq!(average_label(), "average 1 m/s");
        assert_eq!(re_tex(), r"\mathrm{Re} = \bar{U} D / \nu = 1 \times 0.1 / 0.001 = 100");
        assert_eq!(
            fluid_tex(),
            r"\text{fluid: }\nu = 10^{-3}\,\mathrm{m^2/s},\quad \rho = 1\,\mathrm{kg/m^3}"
        );
    }

    #[test]
    fn the_number_line() {
        assert_eq!(LINE.x(LINE.lo), LINE.x0);
        assert_eq!(LINE.x(LINE.hi), LINE.x1);
        assert_eq!(LINE.ticks(50), [Tenk(2850), Tenk(2900), Tenk(2950), Tenk(3000)]);
        let all = GRIDS
            .iter()
            .chain(&STEPS)
            .map(|r| r.st)
            .chain([band().0, band().1, MEASURED, LONGER_INLET]);
        for v in all {
            assert!(LINE.contains(v), "{v:?} on the line");
        }
        // the closest dots in a row, 0.2994 and 0.3003, sit 0.54 units (130 px at 4K) apart
        let close = LINE.x(GRIDS[2].st) - LINE.x(GRIDS[1].st);
        assert!((close - 0.54).abs() < 1e-9);
        // the 0.3 guide falls between them
        assert!(LINE.x(GRIDS[1].st) < LINE.x(Tenk(3000)) && LINE.x(Tenk(3000)) < LINE.x(GRIDS[2].st));
    }
}
