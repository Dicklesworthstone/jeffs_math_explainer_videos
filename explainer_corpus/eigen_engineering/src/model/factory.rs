//! The honest contrast: a factory that makes bicycles and scooters from
//! limited labour, aluminium and assembly slots and wants the most profit.
//! That is a linear program. Its answer sits at a corner of the region the
//! limits leave, and no eigenvector finds it.

use super::economy::Frac;

/// One limit: `bicycles·x + scooters·y ≤ stock`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limit {
    pub name: &'static str,
    pub bicycles: i128,
    pub scooters: i128,
    pub stock: i128,
}

/// What a bicycle and a scooter each use, and what there is.
pub const LIMITS: [Limit; 3] = [
    Limit {
        name: "labour",
        bicycles: 2,
        scooters: 1,
        stock: 100,
    },
    Limit {
        name: "aluminium",
        bicycles: 1,
        scooters: 2,
        stock: 80,
    },
    Limit {
        name: "assembly",
        bicycles: 1,
        scooters: 1,
        stock: 60,
    },
];
/// Profit on a bicycle and on a scooter.
pub const PROFIT: [i128; 2] = [8, 7];
/// The point the owner's infographic labels as a corner, and is not one.
pub const NOT_A_CORNER: [i128; 2] = [0, 60];

pub fn profit(x: Frac, y: Frac) -> Frac {
    Frac::whole(PROFIT[0]) * x + Frac::whole(PROFIT[1]) * y
}

/// Whether a plan breaks no limit (and makes nothing negative).
pub fn allowed(x: Frac, y: Frac) -> bool {
    !x.is_negative()
        && !y.is_negative()
        && LIMITS.iter().all(|l| {
            let used = Frac::whole(l.bicycles) * x + Frac::whole(l.scooters) * y;
            !(Frac::whole(l.stock) - used).is_negative()
        })
}

/// The limits a plan uses up exactly.
pub fn tight(x: Frac, y: Frac) -> Vec<&'static str> {
    LIMITS
        .iter()
        .filter(|l| Frac::whole(l.bicycles) * x + Frac::whole(l.scooters) * y == Frac::whole(l.stock))
        .map(|l| l.name)
        .collect()
}

/// The corners of the allowed region: every crossing of two boundary lines
/// (the three limits and the two axes) that breaks no limit, in order of
/// their bicycles and then scooters.
pub fn corners() -> Vec<(Frac, Frac)> {
    // each line as a·x + b·y = c
    let mut lines: Vec<(i128, i128, i128)> = LIMITS.iter().map(|l| (l.bicycles, l.scooters, l.stock)).collect();
    lines.push((1, 0, 0));
    lines.push((0, 1, 0));
    let mut found: Vec<(Frac, Frac)> = Vec::new();
    for i in 0..lines.len() {
        for j in i + 1..lines.len() {
            let ((a1, b1, c1), (a2, b2, c2)) = (lines[i], lines[j]);
            let det = a1 * b2 - a2 * b1;
            if det == 0 {
                continue;
            }
            let (x, y) = (Frac::new(c1 * b2 - c2 * b1, det), Frac::new(a1 * c2 - a2 * c1, det));
            if allowed(x, y) && !found.contains(&(x, y)) {
                found.push((x, y));
            }
        }
    }
    found.sort_by(|a, b| {
        (a.0.to_f64(), a.1.to_f64())
            .partial_cmp(&(b.0.to_f64(), b.1.to_f64()))
            .expect("corners are finite")
    });
    found
}

/// The best plan among whole bicycles and scooters, by trying every one.
pub fn best_whole_plan() -> ([i128; 2], i128) {
    let mut best = ([0, 0], 0);
    for x in 0..=100 {
        for y in 0..=100 {
            if allowed(Frac::whole(x), Frac::whole(y)) {
                let p = PROFIT[0] * x + PROFIT[1] * y;
                if p > best.1 {
                    best = ([x, y], p);
                }
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_corners_and_the_best_is_forty_and_twenty() {
        let whole = |c: &(Frac, Frac)| [c.0.num, c.1.num];
        let found = corners();
        assert!(found.iter().all(|c| c.0.den == 1 && c.1.den == 1));
        assert_eq!(
            found.iter().map(whole).collect::<Vec<_>>(),
            [[0, 0], [0, 40], [40, 20], [50, 0]]
        );
        let best = found.iter().max_by_key(|c| profit(c.0, c.1).num).unwrap();
        assert_eq!((whole(best), profit(best.0, best.1).num), ([40, 20], 460));
        assert_eq!(best_whole_plan(), ([40, 20], 460));
        assert_eq!(tight(best.0, best.1), ["labour", "aluminium", "assembly"]);
        assert!(!allowed(Frac::whole(NOT_A_CORNER[0]), Frac::whole(NOT_A_CORNER[1])));
    }
}
