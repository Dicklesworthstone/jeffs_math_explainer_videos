//! Chapter 12's update data. Every value the chapter shows is computed here by the rule it
//! describes; nothing is typed in.
//!
//! Each of the chapter's three limits has its own toy, so the picture never mixes them:
//!
//! * Courant, a limit on reach: [`Row`], one dye row in a uniform wind that moves the dye C cells
//!   per step, advanced by the neighbor rule (explicit upwind, reads one cell back) or by the
//!   blend (linear semi-Lagrangian, reads wherever the dye came from).
//! * Temporal truncation, an error with no cells at all: [`channel_path`], the opening's channel
//!   parcel (u = 2 + x) stepped by forward Euler and by the midpoint rule.
//! * Diffusion, a limit set by the layer spacing: [`explicit_layers`], [`implicit_layers`] and
//!   [`exact_layers`] on a ring of layers pulled toward their neighbors' average.
//!
//! The independent oracle is the exact Fraction run
//! `navier-transport-scene02/runs/compare02-run02/facts01.txt` (transport01.py and
//! transport02.py). The tests pin these f64 rules to its values; the oracle is never read at
//! render time.

use std::f64::consts::PI;

/// The pulse: this many coarse cells of dye at strength 1.
pub const PULSE_WIDTH: usize = 4;
/// The trip: the wind carries the dye this many coarse cells.
pub const TRAVEL: usize = 24;
/// Chapter 11's step: half a cell per step.
pub const CHAPTER11_COURANT: f64 = 0.5;
/// Chapter 11's riding window, in coarse cells from the exact pulse's first cell.
pub const CENTRED_WINDOW: [f64; 2] = [-12.0, 16.0];
/// The window fitted to the neighbor rule at C = 3/2 over steps 0 to 10 (12_4 to 12_9).
pub const FITTED_WINDOW: [f64; 2] = [-20.0, 8.0];
/// Chapter 11's declared physical dye spreading over the trip, in coarse cells squared.
pub const DECLARED_SPREADING: f64 = 6.0;

const PAD: usize = 8;

/// How a cell's new value is made from the old row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule {
    /// New value = (1 − C) × own + C × upstream neighbor: explicit upwind, reach one cell.
    Neighbor,
    /// Look back C cells and interpolate linearly between the two cells there.
    Blend,
}

impl Rule {
    /// The rule's two weights, each with how many cells behind the cell it reads.
    pub fn weights(self, courant: f64) -> [(usize, f64); 2] {
        match self {
            Self::Neighbor => [(0, 1.0 - courant), (1, courant)],
            Self::Blend => {
                let whole = courant.floor();
                let part = courant - whole;
                let back = whole as usize;
                [(back, 1.0 - part), (back + 1, part)]
            }
        }
    }

    /// The farthest cell behind it that a cell reads with a nonzero weight.
    pub fn reach(self, courant: f64) -> usize {
        self.weights(courant)
            .iter()
            .filter(|&&(_, weight)| weight != 0.0)
            .map(|&(back, _)| back)
            .max()
            .unwrap_or(0)
    }
}

/// Steps for the 24-coarse-cell trip on cells `resolution` times finer, at `courant` fine cells
/// per step.
pub fn trip_steps(courant: f64, resolution: usize) -> usize {
    let steps = (TRAVEL * resolution) as f64 / courant;
    assert!(steps.fract() == 0.0, "the trip must take a whole number of steps");
    steps as usize
}

/// Readouts over every cell of the row, never only the window.
pub struct Measure {
    pub peak: f64,
    pub lowest: f64,
    /// Dye total in coarse cells (strength × width).
    pub total: f64,
}

/// One row of cells carrying the dye pulse in a uniform wind.
#[derive(Clone, Debug)]
pub struct Row {
    pub values: Vec<f64>,
    pub step: usize,
    pub rule: Rule,
    /// Fine cells the wind carries the dye per step.
    pub courant: f64,
    first_cell: usize,
    resolution: usize,
}

impl Row {
    /// The step-0 pulse on cells `resolution` times finer than the coarse cells, padded for
    /// `steps` steps. Dye never moves upstream and moves at most `reach` cells downstream per
    /// step, so neither end of the row is touched; [`Row::advance`] checks this every step.
    pub fn new(rule: Rule, courant: f64, resolution: usize, steps: usize) -> Self {
        assert!(courant > 0.0 && resolution > 0, "a wind and a mesh");
        let width = PULSE_WIDTH * resolution;
        let downstream = steps * (rule.reach(courant) + 1) + PAD;
        let mut values = vec![0.0; PAD + width + downstream];
        values[PAD..PAD + width].fill(1.0);
        Self {
            values,
            step: 0,
            rule,
            courant,
            first_cell: PAD,
            resolution,
        }
    }

    /// The row for the chapter's whole 24-cell trip.
    pub fn trip(rule: Rule, courant: f64, resolution: usize) -> Self {
        Self::new(rule, courant, resolution, trip_steps(courant, resolution))
    }

    /// One step of the row's rule, every cell from the old row.
    pub fn advance(&mut self) {
        let weights = self.rule.weights(self.courant);
        let old = &self.values;
        let next: Vec<f64> = (0..old.len())
            .map(|i| {
                weights
                    .iter()
                    .map(|&(back, weight)| i.checked_sub(back).map_or(0.0, |j| weight * old[j]))
                    .sum()
            })
            .collect();
        let empty = |cells: &[f64]| cells.iter().all(|&q| q == 0.0);
        assert!(
            empty(&next[..4]) && empty(&next[next.len() - 4..]),
            "the dye reached an end of the row"
        );
        self.values = next;
        self.step += 1;
    }

    /// The cell's [left, right] in coarse cells, riding with the exact pulse, which sits at 0 to
    /// [`PULSE_WIDTH`] at every step.
    pub fn riding_bounds(&self, index: usize) -> [f64; 2] {
        let resolution = self.resolution as f64;
        let left = (index as f64 - self.first_cell as f64 - self.step as f64 * self.courant) / resolution;
        [left, left + 1.0 / resolution]
    }

    /// Riding bounds and values of the nonzero cells, upstream to downstream.
    pub fn occupied(&self) -> Vec<([f64; 2], f64)> {
        self.values
            .iter()
            .enumerate()
            .filter(|&(_, &q)| q != 0.0)
            .map(|(i, &q)| (self.riding_bounds(i), q))
            .collect()
    }

    /// Whether every nonzero cell lies wholly inside `window`.
    pub fn fits(&self, window: [f64; 2]) -> bool {
        self.occupied()
            .iter()
            .all(|&([left, right], _)| left >= window[0] && right <= window[1])
    }

    pub fn measure(&self) -> Measure {
        let cell = 1.0 / self.resolution as f64;
        Measure {
            peak: self.values.iter().fold(0.0, |m: f64, &q| m.max(q)),
            lowest: self.values.iter().fold(0.0, |m: f64, &q| m.min(q)),
            total: self.values.iter().sum::<f64>() * cell,
        }
    }

    /// The largest |value| anywhere in the row.
    pub fn largest(&self) -> f64 {
        self.values.iter().fold(0.0, |m: f64, &q| m.max(q.abs()))
    }

    /// The largest |computed − exact| over the cells, the exact pulse being 1 on 0 to
    /// [`PULSE_WIDTH`] in riding coordinates and 0 elsewhere.
    pub fn max_error(&self) -> f64 {
        let resolution = self.resolution as f64;
        self.values
            .iter()
            .enumerate()
            .map(|(i, &q)| {
                let [left, right] = self.riding_bounds(i);
                let exact = (right.min(PULSE_WIDTH as f64) - left.max(0.0)).max(0.0) * resolution;
                (q - exact).abs()
            })
            .fold(0.0, f64::max)
    }

    /// The dye's variance about its centre, in coarse cells squared.
    pub fn variance(&self) -> f64 {
        let resolution = self.resolution as f64;
        let total: f64 = self.values.iter().sum();
        let mean = self.values.iter().enumerate().map(|(i, &q)| i as f64 * q).sum::<f64>() / total;
        let variance = self
            .values
            .iter()
            .enumerate()
            .map(|(i, &q)| (i as f64 - mean).powi(2) * q)
            .sum::<f64>()
            / total;
        variance / (resolution * resolution)
    }

    /// Variance added since step 0: the smearing the chapter charts, in coarse cells squared.
    pub fn spread(&self) -> f64 {
        self.variance() - Self::new(self.rule, self.courant, self.resolution, 0).variance()
    }
}

/// Chapter 11's declared dye diffusion, as the explicit dye-step weight r = κ∆t/∆x² on cells
/// halved `halvings` times with the step halved too (C = ½): r doubles with each halving.
pub fn dye_diffusion_weight(halvings: u32) -> f64 {
    let finer = 1usize << halvings;
    let scale = finer as f64;
    DECLARED_SPREADING * scale * scale / (2.0 * trip_steps(CHAPTER11_COURANT, finer) as f64)
}

// ------------------------------------------------------------------------------- the channel

/// The parcel's time in the channel, in seconds.
pub const CHANNEL_TIME: f64 = 0.5;

/// Speed along the opening's channel centre line, m/s: u = 2 + x from x = 0.
pub fn channel_speed(x: f64) -> f64 {
    2.0 + x
}

/// The exact centre-line path, x(t) = 2(e^t − 1) metres.
pub fn channel_exact(t: f64) -> f64 {
    2.0 * t.exp_m1()
}

/// How a time step uses the speed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeRule {
    /// The speed at the start of each step.
    ForwardEuler,
    /// The speed halfway through each step, reached by half a forward Euler step.
    Midpoint,
}

/// Positions after 0, 1, …, `steps` equal steps from x = 0 to t = [`CHANNEL_TIME`].
pub fn channel_path(rule: TimeRule, steps: usize) -> Vec<f64> {
    let dt = CHANNEL_TIME / steps as f64;
    let mut x = 0.0;
    let mut path = vec![x];
    for _ in 0..steps {
        x += match rule {
            TimeRule::ForwardEuler => dt * channel_speed(x),
            TimeRule::Midpoint => dt * channel_speed(x + dt / 2.0 * channel_speed(x)),
        };
        path.push(x);
    }
    path
}

/// How far short of the exact position the stepped parcel ends, in metres.
pub fn channel_miss(rule: TimeRule, steps: usize) -> f64 {
    channel_exact(CHANNEL_TIME) - channel_path(rule, steps)[steps]
}

// -------------------------------------------------------------------------------- the layers

/// The zig-zag stack, 1, 3, 1, 3, … m/s, with its top joined to its bottom.
pub fn zigzag(layers: usize) -> Vec<f64> {
    (0..layers).map(|i| if i % 2 == 0 { 1.0 } else { 3.0 }).collect()
}

fn around(speeds: &[f64], i: usize) -> (f64, f64) {
    let n = speeds.len();
    (speeds[(i + n - 1) % n], speeds[(i + 1) % n])
}

/// Each layer's two neighbors' average, where viscosity pulls it.
pub fn neighbor_average(speeds: &[f64]) -> Vec<f64> {
    (0..speeds.len())
        .map(|i| {
            let (below, above) = around(speeds, i);
            (below + above) / 2.0
        })
        .collect()
}

/// One explicit step: each layer moves by its pull times the step, r = ν∆t/h².
pub fn explicit_layers(speeds: &[f64], r: f64) -> Vec<f64> {
    speeds
        .iter()
        .enumerate()
        .map(|(i, &x)| {
            let (below, above) = around(speeds, i);
            x + r * (below - 2.0 * x + above)
        })
        .collect()
}

/// The implicit step's linked equations, (1 + 2r) w_i − r w_(i−1) − r w_(i+1) = v_i, on the ring.
pub fn implicit_matrix(layers: usize, r: f64) -> Vec<Vec<f64>> {
    (0..layers)
        .map(|i| {
            let mut row = vec![0.0; layers];
            row[i] += 1.0 + 2.0 * r;
            row[(i + layers - 1) % layers] -= r;
            row[(i + 1) % layers] -= r;
            row
        })
        .collect()
}

/// One implicit step: all the layers' new speeds solved together, by Gaussian elimination with
/// partial pivoting on [`implicit_matrix`].
#[allow(clippy::needless_range_loop)]
pub fn implicit_layers(speeds: &[f64], r: f64) -> Vec<f64> {
    let n = speeds.len();
    let mut rows: Vec<Vec<f64>> = implicit_matrix(n, r)
        .into_iter()
        .zip(speeds)
        .map(|(mut row, &right)| {
            row.push(right);
            row
        })
        .collect();
    for column in 0..n {
        let best = (column..n)
            .max_by(|&a, &b| rows[a][column].abs().total_cmp(&rows[b][column].abs()))
            .expect("a nonempty column");
        rows.swap(column, best);
        let pivot = rows[column].clone();
        for (k, row) in rows.iter_mut().enumerate() {
            if k != column && row[column] != 0.0 {
                let factor = row[column] / pivot[column];
                for (x, &p) in row.iter_mut().zip(&pivot).skip(column) {
                    *x -= factor * p;
                }
            }
        }
    }
    rows.iter().enumerate().map(|(i, row)| row[n] / row[i]).collect()
}

/// The same layers solved exactly in time over one step: Fourier mode k of the n layers decays
/// by e^(−4 r sin²(πk/n)). Exact for this layer model only, not for a smooth continuous flow.
pub fn exact_layers(speeds: &[f64], r: f64) -> Vec<f64> {
    let n = speeds.len() as f64;
    (0..speeds.len())
        .map(|j| {
            (0..speeds.len())
                .map(|k| {
                    let decay = (-4.0 * r * (PI * k as f64 / n).sin().powi(2)).exp();
                    let projection: f64 = speeds
                        .iter()
                        .enumerate()
                        .map(|(m, &x)| x * (2.0 * PI * k as f64 * (j as f64 - m as f64) / n).cos())
                        .sum();
                    decay * projection / n
                })
                .sum()
        })
        .collect()
}

/// The fraction of the zig-zag left after a step: (after − mean) / (before − mean) on layer 0.
pub fn zigzag_left(before: &[f64], after: &[f64]) -> f64 {
    let mean = before.iter().sum::<f64>() / before.len() as f64;
    (after[0] - mean) / (before[0] - mean)
}

/// The largest explicit step, on a grid of 1/1024 s, that does not grow the zig-zag of `layers`
/// layers `spacing` apart (the zig-zag is the mode that limits the explicit step).
pub fn largest_safe_step(layers: usize, spacing: f64, viscosity: f64) -> f64 {
    let zig = zigzag(layers);
    (1..=4096)
        .map(|k| f64::from(k) / 1024.0)
        .filter(|&dt| {
            let r = viscosity * dt / (spacing * spacing);
            zigzag_left(&zig, &explicit_layers(&zig, r)).abs() <= 1.0 + 1e-12
        })
        .fold(0.0, f64::max)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two(x: f64) -> String {
        format!("{x:.2}")
    }

    fn near(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-12
    }

    /// Python's `{:.3g}`, as the oracle prints it.
    fn sig3(x: f64) -> String {
        if x >= 1e6 {
            return format!("{x:.2e}").replace('e', "e+");
        }
        let decimals = (2 - x.log10().floor() as i32).max(0) as usize;
        let text = format!("{x:.decimals$}");
        if text.contains('.') {
            text.trim_end_matches('0').trim_end_matches('.').to_owned()
        } else {
            text
        }
    }

    #[test]
    fn the_rules_agree_cell_for_cell_at_half_a_cell_per_step() {
        // pulse.steps, blend.weights.half, blend.equals.nbr.half, blend.end.*
        assert_eq!(trip_steps(CHAPTER11_COURANT, 1), 48);
        assert_eq!(Rule::Blend.weights(0.5), [(0, 0.5), (1, 0.5)]);
        assert_eq!(Rule::Neighbor.weights(0.5), [(0, 0.5), (1, 0.5)]);
        let mut blend = Row::trip(Rule::Blend, 0.5, 1);
        let mut neighbor = Row::trip(Rule::Neighbor, 0.5, 1);
        for _ in 0..48 {
            blend.advance();
            neighbor.advance();
            assert!(
                blend
                    .values
                    .iter()
                    .zip(&neighbor.values)
                    .all(|(a, b)| a.to_bits() == b.to_bits())
            );
        }
        let m = blend.measure();
        assert_eq!([two(m.peak), two(m.lowest), two(m.total)], ["0.43", "0.00", "4.00"]);
        assert!((blend.spread() - 12.0).abs() < 1e-9);
    }

    #[test]
    fn the_neighbor_rule_blows_up_at_one_and_a_half_cells_per_step() {
        // nbr.weights.C1p5, nbr.reach, nbr.step1, nbr.step1.gap, nbr.step10.*, nbr.total.kept,
        // nbr.C1p5.support.union, nbr.C1p5.window.holds, nbr.C1p5.centred.drops and .dropped.step10
        assert_eq!(Rule::Neighbor.weights(1.5), [(0, -0.5), (1, 1.5)]);
        assert_eq!(Rule::Neighbor.reach(1.5), 1);
        assert_eq!(Rule::Blend.reach(1.5), 2);
        let mut row = Row::new(Rule::Neighbor, 1.5, 1, 10);
        assert!(row.fits(FITTED_WINDOW) && row.fits(CENTRED_WINDOW));
        let (mut low, mut high) = (0.0_f64, PULSE_WIDTH as f64);
        let mut drops = Vec::new();
        for step in 1..=10 {
            row.advance();
            assert!(near(row.measure().total, 4.0));
            if step == 1 {
                let m = row.measure();
                assert!(near(m.peak, 1.5) && near(m.lowest, -0.5));
                let at = |x: f64| row.values.iter().position(|&q| near(q, x)).unwrap();
                assert_eq!(at(1.5) - at(-0.5), 4);
            }
            let cells = row.occupied();
            low = low.min(cells[0].0[0]);
            high = high.max(cells[cells.len() - 1].0[1]);
            assert!(row.fits(FITTED_WINDOW));
            if !row.fits(CENTRED_WINDOW) {
                drops.push(step);
            }
        }
        assert!(near(low, -15.0) && near(high, 4.0));
        assert_eq!(drops, [9, 10]);
        let m = row.measure();
        assert_eq!([two(m.peak), two(m.lowest)], ["153.77", "-149.98"]);
        let dropped: Vec<String> = row
            .occupied()
            .iter()
            .filter(|&&([left, right], _)| left < CENTRED_WINDOW[0] || right > CENTRED_WINDOW[1])
            .map(|&(_, q)| two(q))
            .collect();
        assert_eq!(dropped, ["0.00", "-0.03", "0.37"]);
        let offsets: Vec<f64> = row
            .occupied()
            .iter()
            .map(|&([left, _], _)| left)
            .filter(|&left| left < CENTRED_WINDOW[0])
            .collect();
        assert_eq!(offsets, [-15.0, -14.0, -13.0]);
    }

    #[test]
    fn the_neighbor_rule_copies_exactly_at_one_cell_per_step() {
        // nbr.limit, nbr.C1.exact
        assert_eq!(Rule::Neighbor.weights(1.0), [(0, 0.0), (1, 1.0)]);
        let mut row = Row::new(Rule::Neighbor, 1.0, 1, 24);
        for _ in 0..24 {
            row.advance();
            assert!(row.max_error() == 0.0);
        }
        for courant in [0.25, 0.5, 0.75, 1.0] {
            assert!(Rule::Neighbor.weights(courant).iter().all(|&(_, w)| w >= 0.0));
        }
        assert!(Rule::Neighbor.weights(1.5)[0].1 < 0.0);
    }

    #[test]
    fn the_blend_smearing_depends_on_the_step() {
        // step.spread.by.C, step.peak.C1p5, step.peak.C0p5, step.peak.C1, step.halve.worse,
        // blend.bounded; per step a(1 − a), per unit time D/(u∆x) = a(1 − a)/(2C)
        let expected = [
            (0.25, 18.0, None),
            (0.5, 12.0, Some("0.43")),
            (0.75, 6.0, None),
            (1.0, 0.0, Some("1.00")),
            (1.5, 4.0, Some("0.67")),
            (2.0, 0.0, None),
        ];
        let mut spreads = Vec::new();
        for (courant, spread, peak) in expected {
            let steps = trip_steps(courant, 1);
            let mut row = Row::trip(Rule::Blend, courant, 1);
            for _ in 0..steps {
                row.advance();
                let m = row.measure();
                assert!(m.lowest >= 0.0 && m.peak <= 1.0 + 1e-12);
                assert!((m.total - 4.0).abs() < 1e-9);
            }
            let part = courant - courant.floor();
            assert!((row.spread() - spread).abs() < 1e-9, "{courant}: {}", row.spread());
            assert!((row.spread() - steps as f64 * part * (1.0 - part)).abs() < 1e-9);
            let per_time = row.spread() / (2.0 * TRAVEL as f64);
            assert!((per_time - part * (1.0 - part) / (2.0 * courant)).abs() < 1e-12);
            if let Some(peak) = peak {
                assert_eq!(two(row.measure().peak), peak);
            }
            spreads.push(row.spread());
        }
        assert!(spreads[0] > spreads[1]);
    }

    #[test]
    fn refining_cells_or_steps_alone() {
        // refine.joint.steps, refine.joint.C, refine.stepfixed.C, refine.stepfixed.blend.spread,
        // refine.stepfixed.nbr.max, refine.numerical.spread, refine.r.by.row, refine.r.explicit
        assert_eq!([1, 2, 4].map(|finer| trip_steps(0.5, finer)), [48, 96, 192]);
        for (finer, numerical) in [(1, 12.0), (2, 6.0), (4, 3.0)] {
            let mut row = Row::trip(Rule::Blend, 0.5, finer);
            for _ in 0..trip_steps(0.5, finer) {
                row.advance();
            }
            assert!((row.spread() - numerical).abs() < 1e-9);
        }
        let expected = [(1, 0.5, 12.0, "0.432"), (2, 1.0, 0.0, "1"), (4, 2.0, 0.0, "4.88e+21")];
        for (finer, courant, spread, largest) in expected {
            assert!(near(0.5 * finer as f64, courant));
            let mut blend = Row::new(Rule::Blend, courant, finer, 48);
            let mut neighbor = Row::new(Rule::Neighbor, courant, finer, 48);
            for _ in 0..48 {
                blend.advance();
                neighbor.advance();
            }
            assert!((blend.spread() - spread).abs() < 1e-9);
            assert_eq!(sig3(neighbor.largest()), largest);
        }
        let r = [0, 1, 2, 3, 4, 5].map(dye_diffusion_weight);
        assert_eq!(r, [0.0625, 0.125, 0.25, 0.5, 1.0, 2.0]);
    }

    #[test]
    fn the_channel_parcel_misses_by_first_and_second_order() {
        // chan.exact.x, chan.euler.miss, chan.mid.miss, chan.*.cm/mm, chan.*.ratio.*
        assert_eq!(format!("{:.2}", channel_exact(CHANNEL_TIME)), "1.30");
        let counts = [1, 2, 4, 8, 16];
        let euler = counts.map(|n| channel_miss(TimeRule::ForwardEuler, n));
        let middle = counts.map(|n| channel_miss(TimeRule::Midpoint, n));
        assert_eq!(
            euler.map(|m| format!("{m:.3}")),
            ["0.297", "0.172", "0.094", "0.049", "0.025"]
        );
        assert_eq!(
            middle.map(|m| format!("{m:.4}")),
            ["0.0474", "0.0142", "0.0039", "0.0010", "0.0003"]
        );
        assert_eq!([euler[2], euler[3]].map(|m| (m * 100.0).round()), [9.0, 5.0]);
        assert_eq!([middle[2], middle[3]].map(|m| (m * 1000.0).round()), [4.0, 1.0]);
        assert_eq!(two(euler[2] / euler[3]), "1.91");
        assert_eq!(two(middle[2] / middle[3]), "3.82");
        assert_eq!(two(euler[3] / euler[4]), "1.95");
        assert_eq!(two(middle[3] / middle[4]), "3.91");
        for n in counts {
            let dt = CHANNEL_TIME / n as f64;
            let closed = 2.0 * ((1.0 + dt).powi(n as i32) - 1.0);
            assert!((channel_path(TimeRule::ForwardEuler, n)[n] - closed).abs() < 1e-14);
            let closed = 2.0 * ((1.0 + dt + dt * dt / 2.0).powi(n as i32) - 1.0);
            assert!((channel_path(TimeRule::Midpoint, n)[n] - closed).abs() < 1e-14);
            assert!(channel_miss(TimeRule::ForwardEuler, n) > 0.0 && channel_miss(TimeRule::Midpoint, n) > 0.0);
        }
    }

    #[test]
    fn the_layers_step_explicitly_implicitly_and_exactly() {
        // layers.zig, .nbr.avg, .mean, .mean.kept, .r*.explicit, .r0p5.is.nbr.avg, .r*.implicit,
        // .overshoot.mult, .zig.eigen, .r1.exact.left, .r1.implicit.left, .r1.implicit.over.exact,
        // .explicit.limit, .limit.halve.h
        let zig = zigzag(8);
        assert_eq!(zig[..4], [1.0, 3.0, 1.0, 3.0]);
        assert_eq!(neighbor_average(&zig)[..2], [3.0, 1.0]);
        let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
        assert!(near(mean(&zig), 2.0));
        for (r, explicit, implicit) in [
            (0.25, [2.0, 2.0], [1.5, 2.5]),
            (0.5, [3.0, 1.0], [5.0 / 3.0, 7.0 / 3.0]),
            (1.0, [5.0, -1.0], [1.8, 2.2]),
        ] {
            let e = explicit_layers(&zig, r);
            let i = implicit_layers(&zig, r);
            assert!(near(e[0], explicit[0]) && near(e[1], explicit[1]), "{r} {e:?}");
            assert!(near(i[0], implicit[0]) && near(i[1], implicit[1]), "{r} {i:?}");
            assert!(near(mean(&e), 2.0) && near(mean(&i), 2.0));
        }
        let stacks = [
            zig.clone(),
            vec![-9.0, 4.0, 0.0, 7.0, -2.0, 5.0, 1.0, -6.0],
            vec![3.0, 3.0, -8.0, 2.0, 9.0, -1.0, 0.0, 4.0],
        ];
        for v in &stacks {
            let e = explicit_layers(v, 0.5);
            assert!(e.iter().zip(neighbor_average(v)).all(|(a, b)| near(*a, b)));
            let x = exact_layers(v, 0.75);
            assert!(near(mean(&x), mean(v)));
            assert!(exact_layers(v, 0.0).iter().zip(v).all(|(a, b)| (a - b).abs() < 1e-12));
        }
        let multipliers = [0.125, 0.25, 0.375, 0.5, 1.0].map(|r| zigzag_left(&zig, &explicit_layers(&zig, r)));
        assert!(
            multipliers
                .iter()
                .zip([0.5, 0.0, -0.5, -1.0, -3.0])
                .all(|(a, b)| near(*a, b))
        );
        for (i, &own) in zig.iter().enumerate() {
            let (below, above) = around(&zig, i);
            assert!(near((below - 2.0 * own + above) / (own - 2.0), -4.0));
        }
        let exact = zigzag_left(&zig, &exact_layers(&zig, 1.0));
        assert!((exact - (-4.0_f64).exp()).abs() < 1e-12);
        assert_eq!(format!("{exact:.3}"), "0.018");
        let implicit = zigzag_left(&zig, &implicit_layers(&zig, 1.0));
        assert!(near(implicit, 0.2));
        assert!(near((implicit / exact).round(), 11.0));
        assert!(near(largest_safe_step(8, 1.0, 1.0), 0.5));
        assert!(near(
            largest_safe_step(16, 0.5, 1.0) / largest_safe_step(8, 1.0, 1.0),
            0.25
        ));
        assert_eq!(implicit_matrix(8, 1.0)[0], [3.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0]);
    }
}
