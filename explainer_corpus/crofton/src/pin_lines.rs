//! Full lines through the oracle's retained seeded pin pairs. The generator
//! and receipt identify the input stream; Rust independently computes crossings.
use crate::model::{CARD_H, CARD_W, P2};

pub const SEED: u64 = 20261025;
pub const COUNT: usize = 10_000;
pub const MIDDLE: [P2; 2] = [[0., -1.], [0., 1.]];
pub const CORNER: [P2; 2] = [[-7., 2.], [-7., 4.]];
pub const STICKS: [[P2; 2]; 2] = [MIDDLE, CORNER];

#[derive(Clone, Copy, Debug)]
pub struct PinLine {
    pub pins: [P2; 2],
}

impl PinLine {
    /// The infinite line, not merely the segment between its two pins.
    pub fn crosses(self, stick: [P2; 2]) -> bool {
        let [p, q] = self.pins;
        let dx = q[0] - p[0];
        let dy = q[1] - p[1];
        let side = |x: P2| dx * (x[1] - p[1]) - dy * (x[0] - p[0]);
        let (a, b) = (side(stick[0]), side(stick[1]));
        (a < 0. && b > 0.) || (a > 0. && b < 0.)
    }

    /// Clip the full line to the card using the parametric line p + t(q-p).
    pub fn chord(self) -> Option<[P2; 2]> {
        let [p, q] = self.pins;
        let d = [q[0] - p[0], q[1] - p[1]];
        if d == [0., 0.] {
            return None;
        }
        let (mut lo, mut hi) = (f64::NEG_INFINITY, f64::INFINITY);
        for axis in 0..2 {
            let half = [CARD_W, CARD_H][axis] / 2.;
            if d[axis].abs() < 1e-14 {
                if p[axis].abs() > half {
                    return None;
                }
            } else {
                let (a, b) = ((-half - p[axis]) / d[axis], (half - p[axis]) / d[axis]);
                lo = lo.max(a.min(b));
                hi = hi.min(a.max(b));
            }
        }
        (hi > lo).then(|| {
            [
                [p[0] + lo * d[0], p[1] + lo * d[1]],
                [p[0] + hi * d[0], p[1] + hi * d[1]],
            ]
        })
    }
}

pub fn samples() -> Result<Vec<PinLine>, String> {
    let mut rows = include_str!("../data/pins-20261025.tsv").lines();
    if rows.next() != Some(format!("# PCG64DXSM seed={SEED}").as_str()) {
        return Err("pin input seed header mismatch".into());
    }
    let mut out = Vec::with_capacity(COUNT);
    for row in rows {
        let cells = row
            .split('\t')
            .map(str::parse::<f64>)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("invalid pin coordinate: {e}"))?;
        if cells.len() != 4 || cells.iter().any(|x| !x.is_finite()) {
            return Err("pin row needs four finite coordinates".into());
        }
        let line = PinLine {
            pins: [[cells[0], cells[1]], [cells[2], cells[3]]],
        };
        if line
            .pins
            .iter()
            .any(|p| p[0].abs() > CARD_W / 2. || p[1].abs() > CARD_H / 2.)
            || line.chord().is_none()
        {
            return Err("pin row outside card or coincident".into());
        }
        out.push(line);
    }
    if out.len() != COUNT {
        return Err(format!("pin input has {} rows, expected {COUNT}", out.len()));
    }
    Ok(out)
}

pub fn totals(lines: &[PinLine]) -> [usize; 2] {
    STICKS.map(|stick| lines.iter().filter(|line| line.crosses(stick)).count())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_full_line_can_cross_beyond_both_pins() {
        let line = PinLine {
            pins: [[-6., 0.], [-5., 0.]],
        };
        assert!(line.crosses(MIDDLE));
        assert!(!line.crosses(CORNER));
        assert_eq!(line.chord(), Some([[-8., 0.], [8., 0.]]));
        assert!(
            !PinLine {
                pins: [[-1., 1.], [1., 1.]]
            }
            .crosses(MIDDLE)
        );
        assert!(
            PinLine {
                pins: [[0., 0.], [0., 0.]]
            }
            .chord()
            .is_none()
        );
    }
    #[test]
    fn seeded_stream_is_measured_and_symmetries_preserve_crossings() {
        let lines = samples().unwrap();
        assert_eq!(totals(&lines), [2178, 1273]);
        for line in lines {
            let chord = line.chord().unwrap();
            for p in chord {
                assert!(p[0].abs() <= CARD_W / 2. + 1e-10 && p[1].abs() <= CARD_H / 2. + 1e-10);
                assert!((p[0].abs() - CARD_W / 2.).abs() < 1e-10 || (p[1].abs() - CARD_H / 2.).abs() < 1e-10);
            }
            for stick in STICKS {
                assert_eq!(
                    line.crosses(stick),
                    PinLine {
                        pins: [line.pins[1], line.pins[0]]
                    }
                    .crosses(stick)
                );
                assert_eq!(line.crosses(stick), line.crosses([stick[1], stick[0]]));
            }
        }
    }
}
