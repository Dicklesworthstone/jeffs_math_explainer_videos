//! Bounding boxes (§8.4, §1.1): the `[min, mid, max]` axis-aligned box that the
//! entire positional API is defined against.
//!
//! The Reference stores the box as three rows — lower-left `min`, center `mid`,
//! upper-right `max` — and its "critical point in a direction" picks, per axis,
//! the min / mid / max according to the sign of the direction. Every positional
//! operation (`next_to`, `move_to`, `align_to`, `to_edge`, …) is expressed in
//! terms of that critical point, so getting this type exactly right is what
//! makes the positional fixtures coincide with the Reference.
//!
//! We compute in f64 (`Semantic`) even though the records are f32, per the
//! numeric doctrine (§6.1); parity is checked at loose f32 tolerance (§16.4).

use fmn_core::types::Vec3;

/// An axis-aligned bounding box as the Reference's three rows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoundingBox {
    /// Lower-left corner (per-axis minimum).
    pub min: Vec3,
    /// Center (per-axis midpoint) — this is `get_center`.
    pub mid: Vec3,
    /// Upper-right corner (per-axis maximum).
    pub max: Vec3,
}

impl BoundingBox {
    /// The degenerate box at the origin — what an empty (point-free) family
    /// reports, matching the Reference's `np.zeros((3, dim))`.
    pub const ZERO: Self = Self {
        min: [0.0; 3],
        mid: [0.0; 3],
        max: [0.0; 3],
    };

    /// The critical point in a direction (Reference `get_bounding_box_point`):
    /// per axis, `dir < 0 → min`, `dir == 0 → mid`, `dir > 0 → max`.
    ///
    /// The sign test is an explicit comparison, deliberately **not**
    /// `f64::signum` — `signum(-0.0)` is `-1.0`, whereas NumPy's `sign(-0.0)` is
    /// `0.0`; the Reference uses the latter, so `-0.0` must select `mid`.
    #[must_use]
    pub fn point(&self, dir: Vec3) -> Vec3 {
        let pick = |d: f64, lo: f64, mi: f64, hi: f64| {
            if d > 0.0 {
                hi
            } else if d < 0.0 {
                lo
            } else {
                mi
            }
        };
        [
            pick(dir[0], self.min[0], self.mid[0], self.max[0]),
            pick(dir[1], self.min[1], self.mid[1], self.max[1]),
            pick(dir[2], self.min[2], self.mid[2], self.max[2]),
        ]
    }

    /// The center (`mid`).
    #[must_use]
    pub fn center(&self) -> Vec3 {
        self.mid
    }

    /// Extent along an axis: `|max - min|` (Reference `length_over_dim`).
    #[must_use]
    pub fn length_over_dim(&self, dim: usize) -> f64 {
        (self.max[dim] - self.min[dim]).abs()
    }

    /// Width (x extent).
    #[must_use]
    pub fn width(&self) -> f64 {
        self.length_over_dim(0)
    }

    /// Height (y extent).
    #[must_use]
    pub fn height(&self) -> f64 {
        self.length_over_dim(1)
    }

    /// Depth (z extent).
    #[must_use]
    pub fn depth(&self) -> f64 {
        self.length_over_dim(2)
    }
}

/// The per-entry lazy bounding-box cache. Keyed by a subtree *signature* (a
/// combination of every family member's point-field revision and identity), so
/// a point write — which bumps the RecordBuffer revision — or a structural
/// change automatically invalidates it, no matter which channel performed the
/// mutation. `materializations` counts actual recomputes, the laziness test
/// hook (mirrors `MirrorSet::materializations`).
#[derive(Clone, Debug)]
pub(crate) struct BboxCache {
    pub signature: Option<u64>,
    pub value: BoundingBox,
    pub materializations: u64,
    pub caller_installed: bool,
}

impl Default for BboxCache {
    fn default() -> Self {
        Self {
            signature: None,
            value: BoundingBox::ZERO,
            materializations: 0,
            caller_installed: false,
        }
    }
}

/// Folds a stream of points into a bounding box. Feeding the raw family points
/// is equivalent to the Reference's "own points plus each descendant's box"
/// (min/max over the boxes equals min/max over the points they bound), and it
/// is simpler and exact.
#[derive(Clone, Copy, Debug)]
pub struct BoxAccum {
    min: Vec3,
    max: Vec3,
    any: bool,
}

impl Default for BoxAccum {
    fn default() -> Self {
        Self::new()
    }
}

impl BoxAccum {
    /// An empty accumulator.
    #[must_use]
    pub fn new() -> Self {
        Self {
            min: [f64::INFINITY; 3],
            max: [f64::NEG_INFINITY; 3],
            any: false,
        }
    }

    /// Include one point.
    pub fn push(&mut self, p: Vec3) {
        self.any = true;
        for ((mn, mx), &v) in self.min.iter_mut().zip(self.max.iter_mut()).zip(p.iter()) {
            *mn = mn.min(v);
            *mx = mx.max(v);
        }
    }

    /// The finished box (`ZERO` if no points were pushed).
    #[must_use]
    pub fn finish(self) -> BoundingBox {
        if !self.any {
            return BoundingBox::ZERO;
        }
        let mid = [
            (self.min[0] + self.max[0]) / 2.0,
            (self.min[1] + self.max[1]) / 2.0,
            (self.min[2] + self.max[2]) / 2.0,
        ];
        BoundingBox {
            min: self.min,
            mid,
            max: self.max,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmn_core::constants::{DOWN, LEFT, ORIGIN, RIGHT, UP};

    fn unit_square() -> BoundingBox {
        let mut acc = BoxAccum::new();
        for p in [[-1.0, -1.0, 0.0], [1.0, 1.0, 0.0], [1.0, -1.0, 0.0]] {
            acc.push(p);
        }
        acc.finish()
    }

    #[test]
    fn empty_is_zero() {
        assert_eq!(BoxAccum::new().finish(), BoundingBox::ZERO);
    }

    #[test]
    fn critical_points_pick_min_mid_max() {
        let bb = unit_square();
        assert_eq!(bb.center(), [0.0, 0.0, 0.0]);
        assert_eq!(bb.point(RIGHT), [1.0, 0.0, 0.0]);
        assert_eq!(bb.point(LEFT), [-1.0, 0.0, 0.0]);
        assert_eq!(bb.point(UP), [0.0, 1.0, 0.0]);
        assert_eq!(bb.point(DOWN), [0.0, -1.0, 0.0]);
        assert_eq!(bb.point([1.0, 1.0, 0.0]), [1.0, 1.0, 0.0]); // UR corner
        assert_eq!(bb.point(ORIGIN), bb.center());
    }

    #[test]
    fn negative_zero_selects_mid_like_numpy_sign() {
        let bb = unit_square();
        // -0.0 must behave like +0.0 (mid), not like a negative (min).
        assert_eq!(bb.point([-0.0, -0.0, -0.0]), bb.center());
    }

    #[test]
    fn extents() {
        let bb = unit_square();
        assert_eq!(bb.width(), 2.0);
        assert_eq!(bb.height(), 2.0);
        assert_eq!(bb.depth(), 0.0);
    }
}
