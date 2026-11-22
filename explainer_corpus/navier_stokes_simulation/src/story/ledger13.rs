//! Chapter 13's ledger data. Measured values are copied from the run records named below,
//! digit for digit, and everything else on screen is computed here from them or from chapter
//! 11's pulse (`transport11`, a byte-for-byte copy of the crate's `src/transport.rs`).
//!
//! Measured evidence, all from the repository root:
//!
//! * Taylor–Green: `narration_lab/third-wave-20261024/root/five-film-release-20261027/
//!   navier-production-20261120/taylor-green-measurements01.json` (sha256 5e80d004), read from
//!   the trace `navier-solver-mac01/command.log` (ea0e4a0a), whose receipt (e493c817) records
//!   exit 0. The time runs' error against the exact flow and largest C are only in the trace.
//!   The solver is `mac_taylor_green.rs`: periodic staggered grid, explicit midpoint steps,
//!   conjugate-gradient pressure solve, viscosity 0.05, end time 0.8 s.
//! * The steady cylinder: `navier-solver-steady04/steady10-verification.json` (19cb43d9) and
//!   `steady-assessment.json` (e67de824), assessed against the frozen
//!   `navier-production-20261120/steady-benchmark-plan04.json` (2fddfecc), whose ranges it
//!   quotes from Schäfer and Turek's table 3.
//!
//! Nothing here is a validation of the tunnel, and the cylinder result is reported as found:
//! its pressure and volume budgets were met and its forces fell outside the published ranges.

use std::f64::consts::TAU;

use crate::transport::{PULSE_WIDTH, Pulse, Rule as Transport, TRAVEL};

// ---------------------------------------------------------------------------- Taylor–Green

/// `mac_taylor_green.rs` `VISCOSITY`.
pub const TG_VISCOSITY: f64 = 0.05;
/// `mac_taylor_green.rs` `END`: every run stops at 0.8 s.
pub const TG_END: f64 = 0.8;
/// The solver's own checks on every run: largest C below 0.65 and ν∆t/h² below ¼.
pub const TG_COURANT_CHECK: f64 = 0.65;
pub const TG_DIFFUSION_CHECK: f64 = 0.25;

/// One measured Taylor–Green run.
#[derive(Clone, Copy, Debug)]
pub struct Run {
    pub cells: usize,
    pub steps: usize,
    pub dt: f64,
    /// The pressure solve's declared budget for the leftover volume imbalance.
    pub budget: f64,
    /// Relative L2 velocity error against the exact Taylor–Green flow at 0.8 s.
    pub error: f64,
    /// Mean kinetic energy per unit mass at 0.8 s, computed and exact.
    pub energy: f64,
    pub exact_energy: f64,
    /// The largest C = ∆t(max|u| + max|v|)/h the run saw.
    pub max_courant: f64,
    /// The largest volume imbalance any pressure correction left.
    pub max_volume_defect: f64,
}

impl Run {
    #[cfg(test)]
    fn spacing(&self) -> f64 {
        TAU / self.cells as f64
    }

    /// The diffusion weight r = ν∆t/h².
    pub fn diffusion_weight(&self) -> f64 {
        diffusion_weight(self.cells, self.dt)
    }
}

pub fn diffusion_weight(cells: usize, dt: f64) -> f64 {
    let h = TAU / cells as f64;
    TG_VISCOSITY * dt / (h * h)
}

/// The space study: cells 16, 32 and 64 per side, the step fixed at 0.005 s (160 steps).
pub const SPACE: [Run; 3] = [
    Run {
        cells: 16,
        steps: 160,
        dt: 0.005,
        budget: 1e-11,
        error: 0.0010233400023957784,
        energy: 0.2134721867515928,
        exact_energy: 0.21303594724155303,
        max_courant: 0.024975492078071163,
        max_volume_defect: 9.997609280197072e-12,
    },
    Run {
        cells: 32,
        steps: 160,
        dt: 0.005,
        budget: 1e-11,
        error: 0.000256727127757468,
        energy: 0.21314534549618885,
        exact_energy: 0.21303594724155236,
        max_courant: 0.050684341932619816,
        max_volume_defect: 9.968685315923843e-12,
    },
    Run {
        cells: 64,
        steps: 160,
        dt: 0.005,
        budget: 1e-11,
        error: 6.423998671775665e-05,
        energy: 0.21306331897354655,
        exact_energy: 0.21303594724155286,
        max_courant: 0.10173646975537784,
        max_volume_defect: 9.996284049079795e-12,
    },
];

/// The record's `space.error_ratios`.
pub const SPACE_RATIOS: [f64; 2] = [3.986099993930268, 3.9963757913808187];

/// The time study: 32 cells per side, steps of 0.04, 0.02 and 0.01 s (trace rows).
pub const TIME: [Run; 3] = [
    Run {
        cells: 32,
        steps: 20,
        dt: 0.04,
        budget: 1e-11,
        error: 0.0002569357979711789,
        energy: 0.21314543442753692,
        exact_energy: 0.21303594724155236,
        max_courant: 0.4054747354609585,
        max_volume_defect: 9.957052967268835e-12,
    },
    Run {
        cells: 32,
        steps: 40,
        dt: 0.02,
        budget: 1e-11,
        error: 0.00025677673854403556,
        energy: 0.2131453666393779,
        exact_energy: 0.21303594724155236,
        max_courant: 0.20273736773047926,
        max_volume_defect: 9.932566836414499e-12,
    },
    Run {
        cells: 32,
        steps: 80,
        dt: 0.01,
        budget: 1e-11,
        error: 0.0002567370429929522,
        energy: 0.21314534972187676,
        exact_energy: 0.21303594724155236,
        max_courant: 0.10136868386523963,
        max_volume_defect: 9.978440430526857e-12,
    },
];

/// The time study's reference: the same 32 cells with 640 steps of 0.00125 s.
pub const TIME_REFERENCE_STEPS: usize = 640;
/// The record's `time.rows[*].relative_error`: each run's difference from the reference.
pub const TIME_DIFFERENCES: [f64; 3] = [2.1171317851735848e-07, 5.2695371169434143e-08, 1.3013156504267503e-08];
/// The record's `time.error_ratios`.
pub const TIME_RATIOS: [f64; 2] = [4.017680752956957, 4.0493919482297285];

/// The pressure study: 32 cells, 80 steps of 0.01 s, the budget 10⁻⁸ and then 100 times tighter.
pub const PRESSURE: [Run; 2] = [
    Run {
        cells: 32,
        steps: 80,
        dt: 0.01,
        budget: 1e-08,
        error: 0.0002567370480630158,
        energy: 0.21314534972105953,
        exact_energy: 0.21303594724155236,
        max_courant: 0.10136868386523963,
        max_volume_defect: 9.901327364904544e-09,
    },
    Run {
        cells: 32,
        steps: 80,
        dt: 0.01,
        budget: 1e-10,
        error: 0.0002567370429920158,
        energy: 0.21314534972187643,
        exact_energy: 0.21303594724155236,
        max_courant: 0.10136868386523963,
        max_volume_defect: 9.954679849402506e-11,
    },
];

/// The record's `pressure.relative_velocity_change`: how far the flow moved when the budget
/// tightened, relative to its size.
pub const PRESSURE_FLOW_CHANGE: f64 = 5.973033556814673e-08;

/// The exact Taylor–Green flow sampled where the solver keeps it (u on vertical faces, v on
/// horizontal faces): its mean kinetic energy per unit mass at time `t` on `cells` per side.
pub fn exact_mean_energy(cells: usize, t: f64) -> f64 {
    let h = TAU / cells as f64;
    let decay = (-2.0 * TG_VISCOSITY * t).exp();
    let mut sum = 0.0;
    for j in 0..cells {
        for i in 0..cells {
            let (x, y) = (h * i as f64, h * j as f64);
            let u = decay * x.sin() * (y + h / 2.0).cos();
            let w = -decay * (x + h / 2.0).cos() * y.sin();
            sum += u * u + w * w;
        }
    }
    sum / (2.0 * (cells * cells) as f64)
}

/// The continuum's energy: ¼ e^(−4νt), the starting ¼ lost at the rate viscosity sets.
#[cfg(test)]
fn continuum_mean_energy(t: f64) -> f64 {
    0.25 * (-4.0 * TG_VISCOSITY * t).exp()
}

/// One run's energy account: what it started with, what it lost, what viscosity predicts it
/// loses, and the gap (lost less than predicted when positive). Only start and end exist.
pub struct EnergyAccount {
    pub start: f64,
    pub lost: f64,
    pub predicted_loss: f64,
    pub gap: f64,
}

pub fn energy_account(run: &Run) -> EnergyAccount {
    let start = exact_mean_energy(run.cells, 0.0);
    let lost = start - run.energy;
    let predicted_loss = start - run.exact_energy;
    EnergyAccount {
        start,
        lost,
        predicted_loss,
        gap: predicted_loss - lost,
    }
}

/// How much smaller each error is than the one before.
#[cfg(test)]
fn ratios(errors: [f64; 3]) -> [f64; 2] {
    [errors[0] / errors[1], errors[1] / errors[2]]
}

/// The observed order: halving the cells or the step divides the error by 2^order.
pub fn observed_order(ratio: f64) -> f64 {
    ratio.log2()
}

/// One rung of the planned joint refinement: not a run, the solver's own step checks applied
/// to the measured 32-cell, 20-step run.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rung {
    pub cells: usize,
    pub steps: usize,
    pub dt: f64,
    /// Nominal C = 2∆t/h: the starting flow's largest |u| + |v| is at most 2.
    pub courant: f64,
    pub diffusion_weight: f64,
    /// Whether r < ¼ forced more steps than the fixed-C ratio.
    pub locked: bool,
}

/// Cells per side on the planned ladder.
pub const LADDER_CELLS: [usize; 6] = [16, 32, 64, 128, 256, 512];

/// Halve cells and step together (20 steps at 32 cells, so C stays fixed) while r stays under
/// ¼; once it would not, take the fewest whole steps that keep r under ¼.
pub fn ladder() -> Vec<Rung> {
    LADDER_CELLS
        .iter()
        .map(|&cells| {
            let fixed = 20 * cells / 32;
            let h = TAU / cells as f64;
            let fewest = (4.0 * TG_VISCOSITY * TG_END / (h * h)).floor() as usize + 1;
            let locked = diffusion_weight(cells, TG_END / fixed as f64) >= TG_DIFFUSION_CHECK;
            let steps = if locked { fewest } else { fixed };
            let dt = TG_END / steps as f64;
            Rung {
                cells,
                steps,
                dt,
                courant: 2.0 * dt / h,
                diffusion_weight: diffusion_weight(cells, dt),
                locked,
            }
        })
        .collect()
}

// ------------------------------------------------------------------------ the steady cylinder

/// The run: fixed level 10, steps capped at 0.0025 s, 2 s of flow in 832 steps.
pub const CYLINDER_STEPS: usize = 832;
/// The frozen force window, seconds.
pub const CYLINDER_WINDOW: [f64; 2] = [1.5, 2.0];
/// The plan's projection budget, and the largest weighted volume defect and scaled pressure
/// residual the run left.
pub const CYLINDER_BUDGET: f64 = 1e-09;
pub const CYLINDER_VOLUME_DEFECT: f64 = 9.9992380339964e-10;
pub const CYLINDER_PRESSURE_RESIDUAL: f64 = 9.999238033797636e-10;

/// A frozen force check: the run's range over the window against the published range.
#[derive(Clone, Copy, Debug)]
pub struct RangeCheck {
    pub name: &'static str,
    pub observed: [f64; 2],
    pub published: [f64; 2],
}

impl RangeCheck {
    /// The assessment's `in_reference_range`: the whole observed range inside the published one.
    pub fn inside(&self) -> bool {
        self.published[0] <= self.observed[0] && self.observed[1] <= self.published[1]
    }
}

pub const CYLINDER_CHECKS: [RangeCheck; 3] = [
    RangeCheck {
        name: "drag",
        observed: [5.582185202367738, 5.5931107462403515],
        published: [5.57, 5.59],
    },
    RangeCheck {
        name: "lift",
        observed: [-0.0031279796024663038, 0.0014033664210001643],
        published: [0.0104, 0.011],
    },
    RangeCheck {
        name: "pressure drop",
        observed: [0.11563672642213299, 0.11576111713439051],
        published: [0.1172, 0.1176],
    },
];

// ------------------------------------------------------------------------- chapter 11's pulse

/// The dye pulse on fixed cells, the step alone shrunk: C cells per step for the same 24-cell
/// trip, by chapter 11's blend ((1 − C) of a cell plus C of its upstream neighbor). Returns the
/// spreading the method made, in square cells (the exact pulse spreads 0).
pub fn step_only_spreading(courant: f64) -> f64 {
    let trip = TRAVEL as f64 / courant;
    assert!(trip.fract() == 0.0, "the trip must take whole steps");
    let steps = trip as usize;
    let pad = 8;
    let mut values = vec![0.0; pad + PULSE_WIDTH + steps + pad];
    values[pad..pad + PULSE_WIDTH].fill(1.0);
    let before = variance(&values);
    for _ in 0..steps {
        let upstream = |i: usize| if i == 0 { 0.0 } else { values[i - 1] };
        values = (0..values.len())
            .map(|i| (1.0 - courant) * values[i] + courant * upstream(i))
            .collect();
        assert!(values[values.len() - 1] == 0.0, "the dye reached the row's end");
    }
    variance(&values) - before
}

// ---------------------------------------------------------------- chapter 12's channel parcel

/// The opening's channel, u = 2 + x: where the exact parcel from x = 0 is after `t` seconds.
pub fn channel_exact(t: f64) -> f64 {
    2.0 * (t.exp() - 1.0)
}

/// Forward Euler's parcel after `steps` steps of `dt` (chapter 12's 12_16).
pub fn channel_euler(steps: usize, dt: f64) -> f64 {
    (0..steps).fold(0.0, |x, _| x + dt * (2.0 + x))
}

/// Chapter 12's half second in four steps of ⅛ s.
pub const CHANNEL_TIME: f64 = 0.5;
pub const CHANNEL_STEPS: usize = 4;

/// 24(1 − C): each step adds C(1 − C) square cells, over 24/C steps.
pub fn step_only_formula(courant: f64) -> f64 {
    TRAVEL as f64 * (1.0 - courant)
}

/// Where the spreading settles as the step shrinks on fixed cells.
pub const STEP_ONLY_LIMIT: f64 = TRAVEL as f64;

/// 13_14's steps: ½, ¼ and ⅛ cell per step.
pub const STEP_ONLY_COURANT: [f64; 3] = [0.5, 0.25, 0.125];

fn variance(values: &[f64]) -> f64 {
    let mass: f64 = values.iter().sum();
    let mean = values.iter().enumerate().map(|(i, q)| i as f64 * q).sum::<f64>() / mass;
    values
        .iter()
        .enumerate()
        .map(|(i, q)| (i as f64 - mean).powi(2) * q)
        .sum::<f64>()
        / mass
}

/// Chapter 11's four rules, as its scoreboard names them.
pub const DYE_RULES: [(&str, Transport); 4] = [
    ("Blend", Transport::Blend),
    ("Cubic", Transport::Curve),
    ("Clamped cubic", Transport::ClampedCurve),
    ("Limited flux", Transport::LimitedFlux),
];

/// A dye budget for one of chapter 11's rules over its whole trip: the dye at the start and
/// at the end, and whether any dye reached either end cell of the row after any step (it could
/// only enter or leave there).
pub struct DyeBudget {
    pub start: f64,
    pub end: f64,
    pub left_the_row: bool,
}

pub fn dye_budget(rule: Transport) -> DyeBudget {
    let mut pulse = Pulse::new(0);
    let start = pulse.measure().total;
    let mut reached_an_end = false;
    for _ in 0..pulse.step_count() {
        pulse.advance(rule, false);
        let last = pulse.values.len() - 1;
        reached_an_end |= pulse.values[0] != 0.0 || pulse.values[last] != 0.0;
    }
    DyeBudget {
        start,
        end: pulse.measure().total,
        left_the_row: reached_an_end,
    }
}

// ---------------------------------------------------------------------- 13_6's staircase

/// Lengths in cylinder diameters, the intended centre at the origin.
pub const SPECIFIED_OFFSET: [f64; 2] = [0.5, 0.0];
/// Cells per diameter in 13_6's three panels.
pub const STAIRCASE_CELLS: [usize; 3] = [8, 16, 32];

/// The cells (row by row, as [first, last] column runs) whose centres lie inside the specified
/// circle: what the solver would see of a cylinder placed at the specified, wrong, centre.
pub struct Staircase {
    /// Cells per diameter.
    pub cells: usize,
    /// (row index, first column, last column), indices counted from the specified centre.
    pub runs: Vec<(i64, i64, i64)>,
}

impl Staircase {
    pub fn new(cells: usize) -> Self {
        let n = cells as i64;
        let h = 1.0 / cells as f64;
        let inside = |i: i64, j: i64| {
            let (x, y) = ((i as f64 + 0.5) * h, (j as f64 + 0.5) * h);
            x * x + y * y < 0.25
        };
        let runs = (-n..n)
            .filter_map(|j| {
                let columns: Vec<i64> = (-n..n).filter(|&i| inside(i, j)).collect();
                Some((j, *columns.first()?, *columns.last()?))
            })
            .collect();
        Self { cells, runs }
    }

    /// Area in square diameters.
    #[cfg(test)]
    fn area(&self) -> f64 {
        let h = 1.0 / self.cells as f64;
        self.runs.iter().map(|&(_, a, b)| (b - a + 1) as f64).sum::<f64>() * h * h
    }

    /// The staircase's centroid, in diameters from the intended centre.
    pub fn centroid(&self) -> [f64; 2] {
        let h = 1.0 / self.cells as f64;
        let (mut area, mut x, mut y) = (0.0, 0.0, 0.0);
        for &(j, a, b) in &self.runs {
            let width = (b - a + 1) as f64 * h;
            area += width * h;
            x += width * h * (a as f64 * h + width / 2.0);
            y += width * h * ((j as f64 + 0.5) * h);
        }
        [SPECIFIED_OFFSET[0] + x / area, SPECIFIED_OFFSET[1] + y / area]
    }

    /// How far the staircase sits from the intended centre, in diameters.
    pub fn offset(&self) -> f64 {
        let [x, y] = self.centroid();
        x.hypot(y)
    }
}

/// A circle of one diameter's area, for the staircase's convergence.
#[cfg(test)]
const CIRCLE_AREA: f64 = std::f64::consts::PI / 4.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_energy_matches_the_record_and_the_continuum() {
        for run in SPACE {
            assert!((exact_mean_energy(run.cells, TG_END) - run.exact_energy).abs() < 1e-15);
            assert!((exact_mean_energy(run.cells, TG_END) - continuum_mean_energy(TG_END)).abs() < 1e-15);
            assert!((exact_mean_energy(run.cells, 0.0) - 0.25).abs() < 1e-15);
        }
    }

    #[test]
    fn ratios_reproduce_the_record() {
        assert_eq!(ratios(SPACE.map(|r| r.error)), SPACE_RATIOS);
        assert_eq!(ratios(TIME_DIFFERENCES), TIME_RATIOS);
        for ratio in SPACE_RATIOS.into_iter().chain(TIME_RATIOS) {
            let order = observed_order(ratio);
            assert!((1.99..2.02).contains(&order), "{order}");
        }
    }

    #[test]
    fn every_measured_run_passed_the_solvers_own_checks() {
        for run in SPACE.iter().chain(&TIME).chain(&PRESSURE) {
            assert!(run.max_courant < TG_COURANT_CHECK);
            assert!(run.diffusion_weight() < TG_DIFFUSION_CHECK);
            assert!(run.max_volume_defect <= 1.001 * run.budget + 1e-13);
            assert!((run.dt * run.steps as f64 - TG_END).abs() < 1e-12);
            assert!(run.max_courant <= 2.0 * run.dt / run.spacing());
        }
    }

    #[test]
    fn a_tighter_pressure_budget_leaves_the_error_against_the_exact_flow() {
        let [loose, tight] = PRESSURE;
        assert!(loose.max_volume_defect / tight.max_volume_defect > 99.0);
        assert!(((loose.error - tight.error) / tight.error).abs() < 1e-7);
        const { assert!(PRESSURE_FLOW_CHANGE < 1e-7) };
        // At fixed cells, the step barely moves the error against the exact flow.
        let errors = TIME.map(|r| r.error);
        assert!((errors[0] - errors[2]) / errors[2] < 1e-3);
    }

    #[test]
    fn energy_falls_by_about_the_predicted_loss_and_the_gap_shrinks_fourfold() {
        let gaps = SPACE.map(|run| {
            let account = energy_account(&run);
            assert!(account.lost > 0.0 && account.gap > 0.0);
            assert!((account.lost / account.predicted_loss - 1.0).abs() < 0.02);
            account.gap
        });
        for ratio in ratios(gaps) {
            assert!((3.9..4.1).contains(&ratio), "{ratio}");
        }
    }

    #[test]
    fn the_ladder_keeps_c_until_r_would_reach_a_quarter() {
        let rungs = ladder();
        let steps: Vec<usize> = rungs.iter().map(|r| r.steps).collect();
        assert_eq!(steps, [10, 20, 40, 80, 266, 1063]);
        for rung in &rungs[..4] {
            assert!(!rung.locked);
            assert!((rung.courant - rungs[0].courant).abs() < 1e-12);
        }
        for rung in &rungs {
            assert!(rung.diffusion_weight < TG_DIFFUSION_CHECK && rung.courant < TG_COURANT_CHECK);
        }
        assert!(rungs[4].locked && rungs[5].locked);
        assert!(rungs[4].courant < 0.25 && rungs[5].courant < 0.125);
        // The measured 32-cell, 20-step run is on the ladder.
        assert_eq!((rungs[1].cells, rungs[1].steps), (TIME[0].cells, TIME[0].steps));
        assert!((rungs[1].dt - TIME[0].dt).abs() < 1e-15);
    }

    #[test]
    fn the_cylinder_met_its_budgets_and_missed_all_three_windows() {
        const { assert!(CYLINDER_VOLUME_DEFECT <= CYLINDER_BUDGET && CYLINDER_PRESSURE_RESIDUAL <= CYLINDER_BUDGET) };
        assert!(CYLINDER_CHECKS.iter().all(|check| !check.inside()));
    }

    #[test]
    fn step_only_spreading_is_twelve_eighteen_twenty_one() {
        let spread = STEP_ONLY_COURANT.map(step_only_spreading);
        for (s, c) in spread.iter().zip(STEP_ONLY_COURANT) {
            assert!((s - step_only_formula(c)).abs() < 1e-9, "{s}");
        }
        assert_eq!(spread.map(|s| s.round() as i64), [12, 18, 21]);
        assert!(spread.iter().all(|&s| s < STEP_ONLY_LIMIT));
        // At C = ½ it is chapter 11's own result.
        let mut pulse = Pulse::new(0);
        let initial = pulse.measure().variance.unwrap();
        for _ in 0..pulse.step_count() {
            pulse.advance(Transport::Blend, false);
        }
        assert!((pulse.measure().variance.unwrap() - initial - spread[0]).abs() < 1e-12);
    }

    #[test]
    fn chapter_12s_parcel_is_nine_centimetres_short() {
        let dt = CHANNEL_TIME / CHANNEL_STEPS as f64;
        let miss = channel_exact(CHANNEL_TIME) - channel_euler(CHANNEL_STEPS, dt);
        assert_eq!((miss * 100.0).round(), 9.0);
        assert!((channel_exact(CHANNEL_TIME) - 1.30).abs() < 0.005);
    }

    #[test]
    fn dye_budgets_match_chapter_11() {
        let totals: Vec<String> = DYE_RULES
            .iter()
            .map(|&(_, rule)| format!("{:.2}", dye_budget(rule).end))
            .collect();
        assert_eq!(totals, ["4.00", "4.00", "3.79", "4.00"]);
        for (_, rule) in DYE_RULES {
            let budget = dye_budget(rule);
            assert!(!budget.left_the_row);
            assert!((budget.start - 4.0).abs() < 1e-15);
        }
    }

    #[test]
    fn the_staircase_converges_to_the_specified_circle_not_the_intended_one() {
        let mut errors = Vec::new();
        for cells in STAIRCASE_CELLS {
            let stair = Staircase::new(cells);
            let [x, y] = stair.centroid();
            assert!((x - SPECIFIED_OFFSET[0]).abs() < 1e-12 && (y - SPECIFIED_OFFSET[1]).abs() < 1e-12);
            assert!((stair.offset() - 0.5).abs() < 1e-12);
            errors.push((stair.area() - CIRCLE_AREA).abs());
        }
        assert!(errors[2] < errors[0]);
        assert!(errors.iter().all(|&e| e < 0.1));
    }
}
