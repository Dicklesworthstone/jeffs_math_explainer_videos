//! Chapter 11's dye experiment. Velocity is prescribed and the dye is passive.
//! The independent Fraction oracle is navier-transport-scene01/tools/transport01.py.

pub const PULSE_WIDTH: usize = 4;
pub const TRAVEL: usize = 24;
pub const COURANT: f64 = 0.5;

#[derive(Clone, Copy)]
pub enum Rule {
    Blend,
    Curve,
    ClampedCurve,
    LimitedFlux,
}

pub struct Pulse {
    pub values: Vec<f64>,
    pub step: usize,
    first_cell: usize,
    resolution: usize,
}

pub struct Measurement {
    pub peak: f64,
    pub minimum: f64,
    pub total: f64,
    pub error: f64,
    pub variance: Option<f64>,
}

fn cell(values: &[f64], index: isize) -> f64 {
    usize::try_from(index)
        .ok()
        .and_then(|i| values.get(i))
        .copied()
        .unwrap_or(0.0)
}

fn minmod(a: f64, b: f64) -> f64 {
    if a * b <= 0.0 {
        0.0
    } else {
        a.signum() * a.abs().min(b.abs())
    }
}

fn step(values: &[f64], rule: Rule) -> Vec<f64> {
    if matches!(rule, Rule::LimitedFlux) {
        // The magnitude on each face is computed once. Both neighboring cells
        // see the same transfer, with opposite signs.
        let mut next = values.to_vec();
        for face in 0..values.len() - 1 {
            let upstream = cell(values, face as isize - 1);
            let slope = minmod(values[face + 1] - values[face], values[face] - upstream);
            let amount = COURANT * (values[face] + (1.0 - COURANT) * slope / 2.0);
            next[face] -= amount;
            next[face + 1] += amount;
        }
        return next;
    }
    (0..values.len()).map(|index| {
        let i = index as isize;
        let (left, right) = (cell(values, i - 1), values[index]);
        if matches!(rule, Rule::Blend) {
            return (left + right) / 2.0;
        }
        let estimate = (9.0 * (left + right) - cell(values, i - 2) - cell(values, i + 1)) / 16.0;
        if matches!(rule, Rule::ClampedCurve) {
            estimate.clamp(left.min(right), left.max(right))
        } else {
            estimate
        }
    }).collect()
}

impl Pulse {
    pub fn new(halvings: u32) -> Self {
        assert!(halvings <= 2, "the teaching experiment uses three meshes");
        let resolution = 1usize << halvings;
        let step_count = 2 * resolution * TRAVEL;
        // One transport step can grow support by two cells and physical
        // diffusion by one more. Padding keeps it clear of both array ends.
        let padding = 3 * step_count + 8;
        let width = resolution * PULSE_WIDTH;
        let mut values = vec![0.0; 2 * padding + width];
        values[padding..padding + width].fill(1.0);
        Self { values, step: 0, first_cell: padding, resolution }
    }

    pub fn step_count(&self) -> usize {
        2 * self.resolution * TRAVEL
    }

    pub fn advance(&mut self, rule: Rule, physical_spreading: bool) {
        assert!(self.step < self.step_count(), "the declared trip is already complete");
        let transported = step(&self.values, rule);
        self.values = if physical_spreading {
            // Add six coarse-width-squared units of physical variance for the
            // whole trip. The diffusion weights are nonnegative on all meshes.
            let factor = self.resolution as f64;
            let r = 6.0 * factor * factor / (2.0 * self.step_count() as f64);
            transported.iter().enumerate().map(|(i, &q)| {
                (1.0 - 2.0 * r) * q
                    + r * (cell(&transported, i as isize - 1) + cell(&transported, i as isize + 1))
            }).collect()
        } else {
            transported
        };
        self.step += 1;
    }

    /// Cell bounds in physical coordinates riding with the exact dye front.
    pub fn riding_bounds(&self, index: usize) -> [f64; 2] {
        let left = index as f64 - self.first_cell as f64 - self.step as f64 * COURANT;
        let resolution = self.resolution as f64;
        [left / resolution, (left + 1.0) / resolution]
    }

    pub fn measure(&self) -> Measurement {
        let mut peak: f64 = 0.0;
        let mut minimum: f64 = 0.0;
        let mut signed_total = 0.0;
        let mut weighted_position = 0.0;
        let mut error = 0.0;
        let cell_width = 1.0 / self.resolution as f64;
        for (i, &q) in self.values.iter().enumerate() {
            let [left, right] = self.riding_bounds(i);
            let exact = (right.min(PULSE_WIDTH as f64) - left.max(0.0)).max(0.0) / cell_width;
            peak = peak.max(q);
            minimum = minimum.min(q);
            signed_total += q;
            weighted_position += (left + right) / 2.0 * q;
            error += (q - exact).abs() * cell_width;
        }
        let variance = (minimum >= 0.0 && signed_total > 0.0).then(|| {
            let mean = weighted_position / signed_total;
            self.values.iter().enumerate().map(|(i, &q)| {
                let [left, right] = self.riding_bounds(i);
                let displacement = (left + right) / 2.0 - mean;
                displacement * displacement * q
            }).sum::<f64>() / signed_total
        });
        Measurement {
            peak, minimum, total: signed_total * cell_width, error, variance,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_transport_rules_reproduce_the_fraction_oracle() {
        let expected = [
            (Rule::Blend, ["0.43", "0.00", "4.00", "4.67"]),
            (Rule::Curve, ["0.90", "-0.06", "4.00", "2.10"]),
            (Rule::ClampedCurve, ["0.69", "0.00", "3.79", "2.67"]),
            (Rule::LimitedFlux, ["0.62", "0.00", "4.00", "3.26"]),
        ];
        for (rule, labels) in expected {
            let mut pulse = Pulse::new(0);
            for _ in 0..pulse.step_count() {
                pulse.advance(rule, false);
                let current = pulse.measure();
                if !matches!(rule, Rule::Curve) {
                    assert!(current.minimum >= -1e-14 && current.peak <= 1.0 + 1e-14);
                }
                if !matches!(rule, Rule::ClampedCurve) {
                    assert!((current.total - 4.0).abs() < 1e-13);
                }
            }
            let result = pulse.measure();
            let rendered = [result.peak, result.minimum, result.total, result.error]
                .map(|value| format!("{value:.2}"));
            assert_eq!(rendered, labels);
            assert_eq!(result.variance.is_none(), matches!(rule, Rule::Curve));
        }
    }

    #[test]
    fn measure_same_physical_trip_on_each_mesh() {
        for (halvings, numerical) in [12.0, 6.0, 3.0].into_iter().enumerate() {
            for physical in [false, true] {
                let mut pulse = Pulse::new(halvings as u32);
                let initial = pulse.measure().variance.unwrap();
                for _ in 0..pulse.step_count() {
                    pulse.advance(Rule::Blend, physical);
                }
                let added = pulse.measure().variance.unwrap() - initial;
                let expected = numerical + if physical { 6.0 } else { 0.0 };
                assert!((added - expected).abs() < 1e-10, "{halvings} {added} {expected}");
            }
        }
    }
}
