//! Chart intervals from the two actual endpoints of a straight stick.
use crate::line_chart::Address;
use crate::model::P2;
use std::f64::consts::PI;

#[derive(Clone, Copy, Debug)]
pub struct Stick {
    pub center: P2,
    pub angle: f64,
    pub length: f64,
}
impl Stick {
    pub fn ends(self) -> [P2; 2] {
        let d = [self.length * self.angle.cos() / 2., self.length * self.angle.sin() / 2.];
        [
            [self.center[0] - d[0], self.center[1] - d[1]],
            [self.center[0] + d[0], self.center[1] + d[1]],
        ]
    }
    pub fn interval(self, theta: f64) -> [f64; 2] {
        let [a, b] = self.ends().map(|p| p[0] * theta.cos() + p[1] * theta.sin());
        [a.min(b), a.max(b)]
    }
    /// Integral of projection width over a half turn: 2 times endpoint distance.
    pub fn area(self) -> f64 {
        let [a, b] = self.ends();
        2. * (a[0] - b[0]).hypot(a[1] - b[1])
    }
    pub fn quadrature(self, steps: usize) -> f64 {
        (0..steps)
            .map(|i| {
                let [lo, hi] = self.interval(PI * (i as f64 + 0.5) / steps as f64);
                hi - lo
            })
            .sum::<f64>()
            * PI
            / steps as f64
    }
}
/// A half-turn changes the chosen normal's sign, hence also signed distance.
pub fn wrap(a: Address) -> Address {
    let turns = (a.theta / PI).floor();
    Address {
        theta: a.theta.rem_euclid(PI),
        distance: if turns.rem_euclid(2.) < 1. {
            a.distance
        } else {
            -a.distance
        },
    }
}
pub fn example(center: P2, angle: f64) -> Stick {
    Stick {
        center,
        angle,
        length: crate::chapters::straight_sticks::LENGTH,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn band_membership_matches_actual_full_line_crossings() {
        for center in [[0., 0.], [3., 0.], [-2., 1.]] {
            for angle in [0., 0.3, PI / 2.] {
                let stick = example(center, angle);
                for j in 0..97 {
                    let theta = PI * (j as f64 + 0.31) / 97.;
                    let [lo, hi] = stick.interval(theta);
                    for distance in [lo - 0.01, lo + 0.0001, (lo + hi) / 2., hi - 0.0001, hi + 0.01] {
                        let a = Address { theta, distance };
                        assert_eq!(lo < distance && distance < hi, a.line().crosses(stick.ends()));
                    }
                }
            }
        }
    }
    #[test]
    fn translation_preserves_each_column_width_and_rotation_preserves_area() {
        for angle in [0., 0.3, PI / 2., 2.7] {
            let centered = example([0., 0.], angle);
            let moved = example([3., 1.], angle);
            for j in 0..301 {
                let theta = PI * j as f64 / 300.;
                let [a, b] = centered.interval(theta);
                let [c, d] = moved.interval(theta);
                assert!(((d - c) - (b - a)).abs() < 1e-12);
                assert!((c - a - 3. * theta.cos() - theta.sin()).abs() < 1e-12);
            }
            assert!((moved.quadrature(20000) - moved.area()).abs() < 1e-6);
            assert!((moved.area() - 12.).abs() < 1e-12);
        }
    }
    #[test]
    fn seam_wrap_preserves_line_but_unsigned_wrap_does_not() {
        for theta in [-0.1, 0., PI - 0.1, PI, PI + 0.1, 2. * PI + 0.1] {
            let a = Address { theta, distance: 2. };
            let b = wrap(a);
            assert!((0. ..PI).contains(&b.theta));
            let [p, q] = a.line().pins;
            for point in [p, q] {
                assert!((point[0] * b.theta.cos() + point[1] * b.theta.sin() - b.distance).abs() < 1e-12);
            }
        }
        let a = wrap(Address {
            theta: PI,
            distance: 2.,
        });
        assert_eq!(a.distance, -2.);
        assert!((a.line().pins[0][0] - 2.).abs() > 3.);
    }
}
