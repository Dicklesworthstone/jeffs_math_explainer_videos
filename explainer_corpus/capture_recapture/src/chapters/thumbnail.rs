//! The thumbnail (one still, 1280×720): Feller's table on the dark water,
//! `100 | 900 / 900 | ?` (01_2), with one fish drawn per hundred beside
//! each count, so the cell no net touched holds 81 silhouettes round its
//! question mark; over it, the title in two lines. Every number comes from
//! `facts::FELLER` and the model's lake (`LAKE_FISH`).

use explainer_kit::props::{Pose, Swarm};
use explainer_kit::{Kit, Palette, thousands, v};
use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};
use fmn::prelude::*;

use super::draw::{Role, fish, flat, marked_fish, ring};
use super::lake::{Cell, Panel, cell_backs, cell_tints, mosaic_in, shuffled};
use crate::facts::FELLER;
use crate::model::{LAKE_FISH, SEED, SplitMix64};

/// One fish drawn per this many in the lake (the chapters draw one per ten;
/// a thumbnail's fish have to be seen at 320 px wide).
const PER_GLYPH: usize = 100;
/// How much larger than a chapter's fish these are drawn.
const FISH_SCALE: f64 = 1.6;
/// How bright the fish no net touched are.
const SILHOUETTE: f64 = 0.3;
/// How tall the three counts' digits are, and the question mark.
const COUNT_H: f64 = 1.05;
const QUESTION_H: f64 = 1.75;
/// The table's rectangle, under the title.
const TABLE: Panel = Panel {
    c: [0.0, -1.5, 0.0],
    w: 13.5,
    h: 4.4,
};
/// The marked column's share of the table's width and the second catch's
/// share of its height: the dark cell gets the most room.
const SHARES: (f64, f64) = (0.4, 0.42);
/// The title's two lines (the second in the dark cell's colour), and the
/// widest either may be.
const TITLE: [(&str, f64); 2] = [("How to Count Fish", 3.05), ("You've Never Seen", 1.8)];
const TITLE_MAX_W: f64 = 13.0;

/// The thumbnail's scene, for the explainer's `thumbnail` field.
pub fn make(kit: Kit) -> Box<dyn SceneConstruct> {
    Box::new(Thumbnail { kit })
}

pub struct Thumbnail {
    pub kit: Kit,
}

/// The four cells' fish: caught twice, first catch only, second catch only,
/// and the ones no net touched.
fn cell_counts() -> [usize; 4] {
    let (big_m, big_n, k) = (FELLER.0 as usize, FELLER.1 as usize, FELLER.2 as usize);
    [k, big_m - k, big_n - k, LAKE_FISH - big_m - big_n + k]
}

/// The table's four cells: a marked share of the width, and in both columns
/// the second catch's share of the height on top (a table that balances).
fn cells() -> [Cell; 4] {
    let (col, row) = SHARES;
    mosaic_in(TABLE, 1.0, col, row * col, row)
}

/// `n` places scattered through `cell`: a jittered grid, alternate rows
/// half a slot apart, whose spare slots are left empty at random (no row
/// ends short), fish facing east.
fn scatter(cell: Cell, n: usize, rng: &mut SplitMix64) -> Vec<Pose> {
    let pad = 0.28;
    let (w, h) = ((cell.w - 2.0 * pad).max(0.05), (cell.h - 2.0 * pad).max(0.05));
    let cols = ((n as f64 * w / h).sqrt().ceil() as usize).max(1);
    let rows = n.div_ceil(cols).max(1);
    let mut slots = shuffled(cols * rows, rng);
    slots.truncate(n);
    slots
        .into_iter()
        .map(|s| {
            let (cx, cy) = ((s % cols) as f64, (s / cols) as f64);
            let stagger = match (rows, (s / cols) % 2) {
                (1, _) => 0.0,
                (_, 0) => -0.25,
                _ => 0.25,
            };
            Pose {
                x: cell.c[0] - w / 2.0 + w * (cx + 0.5 + stagger + 0.25 * (rng.uniform() - 0.5)) / cols as f64,
                y: cell.c[1] + h / 2.0 - h * (cy + 0.5 + 0.3 * (rng.uniform() - 0.5)) / rows as f64,
                heading: 0.5 * (rng.uniform() - 0.5),
            }
        })
        .collect()
}

/// The right-hand part of a cell, where its fish go beside its count.
fn fish_side(cell: Cell) -> Cell {
    Cell {
        c: v(cell.c[0] + 0.27 * cell.w, cell.c[1]),
        w: 0.46 * cell.w,
        h: cell.h,
    }
}

/// Where a cell's count sits: left of its fish.
fn count_at(cell: Cell) -> Vec3 {
    v(cell.c[0] - 0.2 * cell.w, cell.c[1])
}

/// Bold text centred at `at`, `height` tall.
fn bold(kit: &Kit, stage: &mut Stage<'_>, s: &str, color: Srgb, at: Vec3, height: f64) -> fmn::Result<Mob> {
    let m = kit.text_styled(stage, s, 150.0, color, at, true, false)?;
    let h = stage.get_height(m);
    stage.scale(m, height / h);
    Ok(m)
}

/// A school of `template` at `poses`, drawn at the thumbnail's scale.
fn school(stage: &mut Stage<'_>, template: &VMobject, poses: &[Pose]) -> fmn::Result<Swarm> {
    let swarm = Swarm::build(stage, template, poses, None)?;
    for &m in &swarm.members {
        stage.scale(m, FISH_SCALE);
    }
    stage.add_to_scene(swarm.group)?;
    Ok(swarm)
}

impl SceneConstruct for Thumbnail {
    fn name(&self) -> &str {
        "thumbnail"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let counts = cell_counts();
        let drawn = counts.map(|c| c / PER_GLYPH);

        // the water, edge to edge
        let water = Rectangle::new()
            .width(2.0 * FRAME_X_RADIUS)
            .height(2.0 * FRAME_Y_RADIUS)
            .style(flat(Role::water(), 1.0))
            .build()?;
        stage.add(water)?;

        let cells = cells();
        let backs = cell_backs(stage, &cells)?;
        stage.add_to_scene(backs)?;

        // the fish, one per hundred: the dark cell's all over it, the three
        // known cells' beside their counts
        let mut rng = SplitMix64::new(SEED ^ 0x7B);
        let dark = scatter(cells[3], drawn[3], &mut rng);
        let unseen = school(stage, &fish(Role::unmarked()).mob, &dark)?;
        stage.set_opacity(unseen.group, SILHOUETTE, true);
        let twice = scatter(fish_side(cells[0]), drawn[0], &mut rng);
        let first = scatter(fish_side(cells[1]), drawn[1], &mut rng);
        let second = scatter(fish_side(cells[2]), drawn[2], &mut rng);
        let gold: Vec<Pose> = twice.iter().chain(&first).copied().collect();
        school(stage, &marked_fish().mob, &gold)?;
        school(stage, &fish(Role::unmarked()).mob, &second)?;
        let ringed: Vec<Pose> = twice.iter().chain(&second).copied().collect();
        school(stage, &ring().mob, &ringed)?;

        // the three counts beside their fish
        let colors = [Role::marked(), Role::marked(), Role::second()];
        for ((cell, count), color) in cells.iter().zip(counts).zip(colors) {
            let m = bold(&kit, stage, &thousands(count as u64), color, count_at(*cell), COUNT_H)?;
            stage.add_to_scene(m)?;
        }
        // the question in the dark cell, in a clearing in its school
        let clearing = Rectangle::new()
            .width(1.25 * QUESTION_H)
            .height(1.2 * QUESTION_H)
            .corner_radius(0.3)
            .style(flat(cell_tints()[3], 0.88))
            .build()?
            .moved_to(cells[3].c);
        stage.add(clearing)?;
        let question = bold(&kit, stage, "?", Role::dark(), cells[3].c, QUESTION_H)?;
        stage.add_to_scene(question)?;

        // the title, both lines the same size
        let mut lines = Vec::new();
        for ((s, y), color) in TITLE.into_iter().zip([Palette::hero(), Role::dark()]) {
            lines.push(kit.text_styled(stage, s, 130.0, color, v(0.0, y), true, false)?);
        }
        let widest = lines.iter().map(|&m| stage.get_width(m)).fold(0.0, f64::max);
        for m in lines {
            if widest > TITLE_MAX_W {
                stage.scale(m, TITLE_MAX_W / widest);
            }
            stage.add_to_scene(m)?;
        }

        // one second at the thumbnail's 1 fps: the one captured frame
        stage.wait(1.0)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_four_cells_hold_the_whole_lake_in_whole_glyphs() {
        let counts = cell_counts();
        assert_eq!(counts.iter().sum::<usize>(), LAKE_FISH);
        // one fish per hundred draws every cell exactly: nothing rounded away
        assert!(
            counts.iter().all(|c| c % PER_GLYPH == 0 && c / PER_GLYPH >= 1),
            "{counts:?}"
        );
    }

    #[test]
    fn every_drawn_fish_and_its_ring_stay_inside_their_cell() {
        let cells = cells();
        let rooms = [fish_side(cells[0]), fish_side(cells[1]), fish_side(cells[2]), cells[3]];
        // the widest glyph: the ring and the marked fish's halo (draw.rs)
        let reach = 0.15 * FISH_SCALE;
        let mut rng = SplitMix64::new(SEED ^ 0x7B);
        for (room, count) in rooms.into_iter().zip(cell_counts()) {
            let poses = scatter(room, count / PER_GLYPH, &mut rng);
            assert_eq!(poses.len(), count / PER_GLYPH);
            for p in poses {
                assert!(
                    (p.x - room.c[0]).abs() + reach <= room.w / 2.0,
                    "{p:?} leaves its cell sideways"
                );
                assert!(
                    (p.y - room.c[1]).abs() + reach <= room.h / 2.0,
                    "{p:?} leaves its cell up or down"
                );
            }
        }
    }
}
