//! Automatic layout lint: a second pair of eyes on every draft (ported from
//! the explainer skill's template).
//!
//! With a [`Linter`] on the [`Kit`](crate::Kit), every narration beat
//! (`kit.say`) and every `kit.hold` checks the scene as it stands and records:
//! - `outside`: a visible top-level mobject crossing the frame edge (cut off);
//! - `edge`: one inside the frame but within [`MARGIN`] of an edge (cramped,
//!   and player controls cover the bottom strip);
//! - `overlap`: two visible texts (everything the kit's text, Tex and live
//!   helpers built, also inside groups) whose boxes overlap by more than
//!   [`MAX_OVERLAP`] of the smaller one.
//!
//! "Visible" means painted with alpha > 0: vector strokes and fills
//! (including a parent's own paint, such as an arrow's shaft), dot clouds
//! (padded by their radius), surfaces and images. Each problem (kind + the
//! mobjects involved) is reported once, at the first checkpoint it appears,
//! with the number of later checkpoints that still showed it. Contact and cue
//! sheets remain the final word: the lint sees boxes, not pixels. The
//! template's calibration on the eight Hoeffding's D chapters: no false
//! overlaps, seven real findings (one label cut off at the bottom). Assumes
//! the 16:9 frame (8 units tall).

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use crate::kit::Registered;

/// Safe-area margin inside the frame edge (scene units): closer is `edge`.
pub const MARGIN: f64 = 0.15;
/// Overlap share of the smaller box that counts as a collision.
pub const MAX_OVERLAP: f64 = 0.10;

/// An axis-aligned box: (lower-left, upper-right).
pub type Box2 = ([f64; 2], [f64; 2]);

/// `outside` (crosses the frame), `edge` (inside but within the margin), or
/// nothing for a box comfortably inside a `±xr × ±yr` frame.
pub fn edge_kind((lo, hi): Box2, xr: f64, yr: f64) -> Option<&'static str> {
    let clipped = lo[0] < -xr || hi[0] > xr || lo[1] < -yr || hi[1] > yr;
    let (sx, sy) = (xr - MARGIN, yr - MARGIN);
    if clipped {
        Some("outside")
    } else if lo[0] < -sx || hi[0] > sx || lo[1] < -sy || hi[1] > sy {
        Some("edge")
    } else {
        None
    }
}

/// The share of the smaller box that two boxes overlap (0 when they only
/// touch, or overlap by a sliver under 0.05 units high).
pub fn overlap_share(a: Box2, b: Box2) -> f64 {
    let w = a.1[0].min(b.1[0]) - a.0[0].max(b.0[0]);
    let h = a.1[1].min(b.1[1]) - a.0[1].max(b.0[1]);
    let smaller = area(a).min(area(b));
    if w <= 0.0 || h <= 0.05 || smaller <= 0.0 {
        0.0
    } else {
        w * h / smaller
    }
}

fn area((lo, hi): Box2) -> f64 {
    (hi[0] - lo[0]) * (hi[1] - lo[1])
}

struct Finding {
    row: String,
    /// Later checkpoints at which the same problem was still there.
    again: usize,
}

/// The lint of one chapter.
#[derive(Default)]
pub struct Linter {
    findings: RefCell<Vec<Finding>>,
    /// Problem key (kind + mobjects) -> index into `findings`.
    seen: RefCell<HashMap<String, usize>>,
}

impl Linter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn count(&self) -> usize {
        self.findings.borrow().len()
    }

    /// The findings as TSV rows: `checkpoint  kind  detail`.
    pub fn report(&self) -> String {
        self.findings
            .borrow()
            .iter()
            .map(|f| match f.again {
                0 => format!("{}\n", f.row),
                n => format!("{} (still there at {n} later checkpoints)\n", f.row),
            })
            .collect()
    }

    /// Check the scene now against the kit's registered texts; `at` names
    /// the checkpoint ("before 01_2", "hold").
    pub fn check(&self, arena: &Arena, texts: &[Registered], at: &str) {
        let short = |s: &str| s.chars().take(32).collect::<String>();
        let label_of = |root: Mob| -> String {
            if let Some(t) = texts.iter().find(|t| t.mob == root) {
                return format!("{:?}", short(&t.source));
            }
            let family = arena.family(root);
            match texts.iter().find(|t| family.contains(&t.mob)) {
                Some(t) => format!("a group holding {:?}", short(&t.source)),
                None => "an unlabeled shape".to_owned(),
            }
        };
        let mut on_stage: HashSet<Mob> = HashSet::new();
        for &m in arena.roots() {
            on_stage.extend(arena.family(m));
            let Some(b) = visible_box(arena, m) else {
                continue;
            };
            if let Some(kind) = edge_kind(b, FRAME_X_RADIUS, FRAME_Y_RADIUS) {
                self.record(
                    format!("{kind}|{m:?}"),
                    at,
                    kind,
                    &format!(
                        "{} spans x {:.2}..{:.2}, y {:.2}..{:.2}; frame ±{:.2} × ±{:.2}, safe area ±{:.2} × ±{:.2}",
                        label_of(m),
                        b.0[0],
                        b.1[0],
                        b.0[1],
                        b.1[1],
                        FRAME_X_RADIUS,
                        FRAME_Y_RADIUS,
                        FRAME_X_RADIUS - MARGIN,
                        FRAME_Y_RADIUS - MARGIN
                    ),
                );
            }
        }
        let shown: Vec<(Mob, String, Box2)> = texts
            .iter()
            .filter(|t| on_stage.contains(&t.mob))
            .filter_map(|t| visible_box(arena, t.mob).map(|b| (t.mob, short(&t.source), b)))
            .collect();
        for (i, (ma, la, a)) in shown.iter().enumerate() {
            for (mb, lb, b) in &shown[i + 1..] {
                let share = overlap_share(*a, *b);
                if share <= MAX_OVERLAP {
                    continue;
                }
                // a text nested in another registered text is part of it
                if arena.family(*ma).contains(mb) || arena.family(*mb).contains(ma) {
                    continue;
                }
                self.record(
                    format!("overlap|{ma:?}|{mb:?}"),
                    at,
                    "overlap",
                    &format!("{la:?} and {lb:?} overlap ({:.0}% of the smaller)", 100.0 * share),
                );
            }
        }
    }

    fn record(&self, key: String, at: &str, kind: &str, detail: &str) {
        let mut seen = self.seen.borrow_mut();
        let mut findings = self.findings.borrow_mut();
        match seen.get(&key) {
            Some(&i) => findings[i].again += 1,
            None => {
                seen.insert(key, findings.len());
                findings.push(Finding {
                    row: format!("{at}\t{kind}\t{detail}"),
                    again: 0,
                });
            }
        }
    }
}

/// The union of the own-point boxes of every family member that actually
/// paints (alpha > 0). A parent with its own paint counts (an arrow's shaft
/// holds its tip as a child); a plain group has no paint fields and does not.
pub fn visible_box(arena: &Arena, root: Mob) -> Option<Box2> {
    let mut acc: Option<Box2> = None;
    for m in arena.family(root) {
        let Some(e) = arena.get(m) else { continue };
        if !e.paints() {
            continue;
        }
        let b = &e.buffer;
        let any_alpha = |col: Option<Vec<f32>>| col.is_some_and(|c| c.as_chunks::<4>().0.iter().any(|c| c[3] > 0.0));
        let lit = match (b.read_column("fill_rgba"), b.read_column("stroke_rgba")) {
            (None, None) => match (b.read_column("rgba"), b.read_column("opacity")) {
                (Some(rgba), _) => any_alpha(Some(rgba)), // dot clouds, surfaces
                (None, Some(opacity)) => opacity.iter().any(|&o| o > 0.0), // images
                (None, None) => true,                     // unknown primitive: assume it shows
            },
            (fill, stroke) => {
                let width = b.read_column("stroke_width").unwrap_or_default();
                any_alpha(fill)
                    || stroke.is_some_and(|s| {
                        s.as_chunks::<4>()
                            .0
                            .iter()
                            .zip(&width)
                            .any(|(c, w)| c[3] > 0.0 && *w > 0.0)
                    })
            }
        };
        if !lit {
            continue;
        }
        let Some(points) = b.read_column("point") else {
            continue;
        };
        let pad = b
            .read_column("radius")
            .map_or(0.0, |r| r.iter().fold(0.0_f64, |a, &x| a.max(f64::from(x))));
        let placement = e.placement();
        for p in points.as_chunks::<3>().0 {
            let q = placement.apply_point([f64::from(p[0]), f64::from(p[1]), f64::from(p[2])]);
            let (lo, hi) = ([q[0] - pad, q[1] - pad], [q[0] + pad, q[1] + pad]);
            acc = Some(match acc {
                None => (lo, hi),
                Some((a, z)) => ([a[0].min(lo[0]), a[1].min(lo[1])], [z[0].max(hi[0]), z[1].max(hi[1])]),
            });
        }
    }
    acc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edges_at_the_thresholds() {
        let (xr, yr) = (FRAME_X_RADIUS, FRAME_Y_RADIUS);
        let b = |x0: f64, x1: f64| ([x0, -1.0], [x1, 1.0]);
        // just inside the safe area, just inside the margin, just outside
        assert_eq!(edge_kind(b(-1.0, xr - MARGIN - 0.001), xr, yr), None);
        assert_eq!(edge_kind(b(-1.0, xr - MARGIN + 0.001), xr, yr), Some("edge"));
        assert_eq!(edge_kind(b(-1.0, xr + 0.001), xr, yr), Some("outside"));
        assert_eq!(edge_kind(([0.0, -yr - 0.01], [1.0, 0.0]), xr, yr), Some("outside"));
    }

    #[test]
    fn overlap_at_the_thresholds() {
        // a 1 × 1 box against one sliding over it: 9% and 11% of the smaller
        let a = ([0.0, 0.0], [1.0, 1.0]);
        let shifted = |dx: f64| ([1.0 - dx, 0.0], [2.0 - dx, 1.0]);
        assert!((overlap_share(a, shifted(0.09)) - 0.09).abs() < 1e-9);
        assert!(overlap_share(a, shifted(0.09)) <= MAX_OVERLAP);
        assert!(overlap_share(a, shifted(0.11)) > MAX_OVERLAP);
        assert_eq!(overlap_share(a, shifted(0.0)), 0.0, "touching is not overlapping");
        // a sliver under 0.05 high never counts
        assert_eq!(overlap_share(a, ([0.5, 0.97], [1.5, 1.5])), 0.0);
    }
}
