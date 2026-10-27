//! Normal-angle and signed-distance addresses for unoriented full lines.
use crate::model::{CARD_H, CARD_W, P2};
use crate::pin_lines::{COUNT, PinLine, SEED};
use std::f64::consts::PI;

#[derive(Clone, Copy, Debug)]
pub struct Address {
    pub theta: f64,
    pub distance: f64,
}

impl Address {
    pub fn line(self) -> PinLine {
        let n = [self.theta.cos(), self.theta.sin()];
        let p = [self.distance * n[0], self.distance * n[1]];
        PinLine {
            pins: [p, [p[0] - n[1], p[1] + n[0]]],
        }
    }
    pub fn from_line(line: PinLine) -> Result<Self, String> {
        let [p, q] = line.pins;
        let d = [q[0] - p[0], q[1] - p[1]];
        let length = d[0].hypot(d[1]);
        if !length.is_finite() || length <= 0. {
            return Err("line needs two distinct finite points".into());
        }
        let mut n = [d[1] / length, -d[0] / length];
        if n[1] < 0. || (n[1] == 0. && n[0] < 0.) {
            n = [-n[0], -n[1]];
        }
        Ok(Self {
            theta: n[1].atan2(n[0]),
            distance: p[0] * n[0] + p[1] * n[1],
        })
    }
    pub fn foot(self) -> P2 {
        [self.distance * self.theta.cos(), self.distance * self.theta.sin()]
    }
}

pub fn reach() -> f64 {
    (CARD_W / 2.).hypot(CARD_H / 2.)
}
pub fn half_width(theta: f64) -> f64 {
    CARD_W / 2. * theta.cos().abs() + CARD_H / 2. * theta.sin().abs()
}

pub fn samples() -> Result<Vec<PinLine>, String> {
    let mut rows = include_str!("../data/even-lines-20261025.tsv").lines();
    let header = format!("# PCG64DXSM seed={SEED} skip_uniforms={}", 4 * COUNT);
    if rows.next() != Some(header.as_str()) {
        return Err("even-line input lineage mismatch".into());
    }
    let mut out = Vec::with_capacity(COUNT);
    for row in rows {
        let cells = row
            .split('\t')
            .map(str::parse::<f64>)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("invalid even-line input: {e}"))?;
        if cells.len() != 4 || cells.iter().any(|x| !x.is_finite()) {
            return Err("line needs four finite coordinates".into());
        }
        let line = PinLine {
            pins: [[cells[0], cells[1]], [cells[2], cells[3]]],
        };
        let a = Address::from_line(line)?;
        if !(0. ..PI).contains(&a.theta) || a.distance.abs() > half_width(a.theta) + 1e-12 || line.chord().is_none() {
            return Err("retained line outside chart region".into());
        }
        out.push(line);
    }
    if out.len() != COUNT {
        return Err("even-line input count mismatch".into());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pin_lines::{STICKS, totals};
    #[test]
    fn line_addresses_round_trip_and_ignore_point_order() {
        for theta in [0., 0.2, PI / 2., PI - 0.1] {
            for distance in [-3., 0., 3.] {
                let a = Address { theta, distance };
                let line = a.line();
                for b in [
                    line,
                    PinLine {
                        pins: [line.pins[1], line.pins[0]],
                    },
                ] {
                    let recovered = Address::from_line(b).unwrap();
                    assert!((recovered.theta - theta).abs() < 1e-12);
                    assert!((recovered.distance - distance).abs() < 1e-12);
                    for stick in STICKS {
                        assert_eq!(b.crosses(stick), recovered.line().crosses(stick));
                    }
                }
            }
        }
        assert!(
            Address::from_line(PinLine {
                pins: [[0., 0.], [0., 0.]]
            })
            .is_err()
        );
    }
    #[test]
    fn retained_conditioned_lines_have_the_oracles_counts() {
        let lines = samples().unwrap();
        assert_eq!(totals(&lines), [796, 811]);
        for line in lines {
            let a = Address::from_line(line).unwrap();
            assert!(a.distance.abs() <= half_width(a.theta) + 1e-12);
            for stick in STICKS {
                assert_eq!(line.crosses(stick), a.line().crosses(stick));
            }
        }
        assert!((half_width(0.) - CARD_W / 2.).abs() < 1e-12);
        assert!((half_width(PI / 2.) - CARD_H / 2.).abs() < 1e-12);
        assert!(half_width(0.) != half_width(PI / 2.));
    }
}
