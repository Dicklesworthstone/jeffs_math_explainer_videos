//! Chosen ideal models, not measurements. Pending native integration and tests.

pub fn remaining(k: f64, t: f64) -> f64 {
    assert!(k.is_finite() && k >= 0.0 && t.is_finite() && t >= 0.0);
    (-k * t).exp()
}

pub fn half_life(k: f64) -> f64 {
    assert!(k.is_finite() && k > 0.0);
    std::f64::consts::LN_2 / k
}

/// Solution of A' = -k A^2; k has reciprocal-concentration/time units.
pub fn second_order(initial: f64, k: f64, t: f64) -> f64 {
    assert!(initial.is_finite() && initial >= 0.0);
    assert!(k.is_finite() && k >= 0.0 && t.is_finite() && t >= 0.0);
    initial / (1.0 + k * initial * t)
}

#[derive(Clone, Copy, Debug)]
pub struct Circuit {
    pub battery_v: f64,
    pub resistance_ohm: f64,
    pub capacitance_f: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct State {
    pub time_constant_s: f64,
    pub voltage_v: f64,
    pub gap_v: f64,
    pub current_a: f64,
    pub charge_c: f64,
}

impl Circuit {
    pub fn state(self, t: f64) -> State {
        assert!(self.battery_v.is_finite() && self.battery_v >= 0.0);
        assert!(self.resistance_ohm.is_finite() && self.resistance_ohm > 0.0);
        assert!(self.capacitance_f.is_finite() && self.capacitance_f > 0.0);
        assert!(t.is_finite() && t >= 0.0);
        let tau = self.resistance_ohm * self.capacitance_f;
        assert!(tau.is_finite() && tau > 0.0);
        let exponent = -t / tau;
        let gap_v = self.battery_v * exponent.exp();
        // exp_m1 avoids subtracting almost-equal floats near the start.
        let voltage_v = self.battery_v * -exponent.exp_m1();
        State {
            time_constant_s: tau,
            voltage_v,
            gap_v,
            current_a: gap_v / self.resistance_ohm,
            charge_c: self.capacitance_f * voltage_v,
        }
    }
}

pub const DEMO: Circuit = Circuit {
    battery_v: 12.0,
    resistance_ohm: 10_000.0,
    capacitance_f: 100e-6,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_clock_for_each_halving() {
        for k in [0.1, 1.0, 3.0] {
            let h = half_life(k);
            for n in 0..8 {
                assert!((remaining(k, f64::from(n) * h) - 0.5_f64.powi(n)).abs() < 1e-14);
            }
        }
    }

    #[test]
    fn second_order_half_life_depends_on_initial_amount() {
        for initial in [0.5, 1.0, 2.0] {
            let k = 1.0;
            let half_time = 1.0 / (k * initial);
            assert_eq!(second_order(initial, k, half_time), initial / 2.0);
            assert_eq!(second_order(initial, 0.0, half_time), initial);
        }
    }

    #[test]
    fn circuit_relations_and_time_rescaling() {
        let slower = Circuit {
            resistance_ohm: 2.0 * DEMO.resistance_ohm,
            ..DEMO
        };
        for step in 0..=100 {
            let t = f64::from(step) / 10.0;
            let a = DEMO.state(t);
            let b = slower.state(2.0 * t);
            assert!((a.voltage_v + a.gap_v - DEMO.battery_v).abs() < 1e-13);
            assert!((a.current_a * DEMO.resistance_ohm - a.gap_v).abs() < 1e-13);
            assert!((a.voltage_v - b.voltage_v).abs() < 1e-13);
            assert!((a.current_a - 2.0 * b.current_a).abs() < 1e-15);
        }
    }

    #[test]
    fn capacitance_changes_clock_not_initial_current() {
        let larger = Circuit {
            capacitance_f: 2.0 * DEMO.capacitance_f,
            ..DEMO
        };
        assert_eq!(DEMO.state(0.0).current_a, larger.state(0.0).current_a);
        assert_eq!(larger.state(0.0).time_constant_s, 2.0 * DEMO.state(0.0).time_constant_s);
        assert_eq!(DEMO.state(0.0).voltage_v, 0.0);
    }
}
