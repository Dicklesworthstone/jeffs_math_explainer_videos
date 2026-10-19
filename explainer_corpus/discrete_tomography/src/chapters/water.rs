//! Chapter 02, water through the grid: the face's shadows as plumbing, each
//! row a tap that lets out its count, each square a pipe for at most one
//! unit, each column a drain that holds at most its count (02_1 to 02_3);
//! the flow's real augmenting paths, one unit each, sent in the order the
//! method found them (02_2, 02_4); after eight units the bottom row is stuck
//! without taking water back (02_5, a prediction); the ninth unit takes the
//! top row's unit back out of the first column and the top row sends it on
//! to the fourth (02_6, 02_7); the wet squares cast the face's shadows and
//! are not the face, one of the sixty-four (02_8); water finds a picture
//! whenever one exists and never says whether it is the only one (02_9).
//!
//! Every square, counter and route drawn comes from `facts::face_flow`: the
//! paths `model::max_flow` actually sent, replayed and checked by
//! `model::replay`; after each path the drawn wet squares are compared with
//! the replayed state.

use std::collections::HashMap;

use explainer_kit::props::{lower_third, precisely};
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    Board, COUNT_SIZE, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, flat, path_line, pause, say,
    shift_all, swap, think, thumb,
};
use super::same_shadows::{THUMB_CELL, gallery_at};
use crate::facts::{self, FORD_FULKERSON_YEAR, GALE_YEAR};
use crate::model::{self, Augmenting, FlowState, Shadows};
use crate::palette;

pub struct WaterThroughTheGrid {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts); the plumbing's
// words, layout and drawing are chapter 03's too
pub(crate) const TO_SEND: &str = "to send";
pub(crate) const TO_DRAIN: &str = "to drain";
const TAP_LINE: &str = "tap: lets out its row's count";
const DRAIN_LINE: &str = "drain: holds at most its column's count";
const FITS_LINE: &str = "every unit through: a picture that fits";
const OUTLINED: &str = "outlined: the face";
const PROGRAMMERS: &str = "for programmers:";

/// The grid's pitch and centre: on the left, the right half kept for words,
/// the think ring and the gallery.
pub(crate) const CELL: f64 = 0.62;
pub(crate) const AT: [f64; 2] = [-3.4, 0.45];
/// How far a tap's counter stands left of the grid; a route starts this far
/// left of the grid and ends this far below it.
pub(crate) const TAP_GAP: f64 = 0.55;
const INLET: f64 = 0.2;
const OUTLET: f64 = 0.2;
/// Where the words on the right start, and their lines.
pub(crate) const TEXT_X: f64 = -0.9;
const LEGEND_Y: [f64; 4] = [2.1, 1.5, 0.9, 0.3];
pub(crate) const THROUGH_Y: f64 = -0.6;
const WITHOUT_Y: f64 = -1.2;
/// The think ring's centre (02_5, 03_2).
pub(crate) const THINK_AT: [f64; 2] = [3.0, 0.5];
/// A drop of water, and how many seconds it takes to run one scene unit.
const DROP_R: f64 = 0.1;
const SLOW: f64 = 0.45;
/// The gallery's thumbnails (chapter 01's layout).
const GALLERY_COUNT_Y: f64 = 0.65;

/// A pipe's capacity, as the legend writes it.
fn square_line() -> String {
    format!("square: carries at most {} unit", model::SQUARE)
}

pub(crate) fn through_text(through: usize, total: usize) -> String {
    format!("through: {through} of {total}")
}

fn without_text(through: usize, total: usize) -> String {
    format!("without taking back: {through} of {total}")
}

/// The "for programmers" lower-third (02_4): the path each unit takes.
fn method_text() -> String {
    "each unit takes a path found breadth first: in from a tap with water left, forward into dry squares, \
     back out of wet ones, down into a drain with room"
        .to_string()
}

/// The exact version of 02_9, kept on screen: the method ends at a largest
/// flow, and a picture fits exactly when both shadows count the same total
/// and that flow reaches it (rows 1, 1 with columns 2, 1 reach their row
/// total and fit nothing: their totals differ).
fn flow_claim() -> String {
    format!(
        "the water ends at a largest flow (Ford and Fulkerson, {FORD_FULKERSON_YEAR}); a picture fits exactly \
         when both shadows share one total and the flow reaches it (Gale, {GALE_YEAR})"
    )
}

fn found_label(count: usize) -> String {
    format!("the water's picture is one of {count}")
}

/// Where row `r`'s tap counter stands.
pub(crate) fn tap_at(board: &Board, r: usize) -> Vec3 {
    v(board.left() - TAP_GAP, board.at(r, 0)[1])
}

/// The route a path's unit runs on the board: in from its row's tap, through
/// the centre of each square it wets or takes water back from (along a row
/// into a square, along a column back up or down to the next), and down out
/// of its last column into the drain. Every leg is along a row or a column.
fn route_points(board: &Board, steps: &[(usize, usize, bool)]) -> Vec<Vec3> {
    let mut points = Vec::with_capacity(steps.len() + 2);
    if let Some(&(r, _, _)) = steps.first() {
        points.push(v(board.left() - INLET, board.at(r, 0)[1]));
    }
    for &(r, c, _) in steps {
        points.push(board.at(r, c));
    }
    if let Some(&(_, c, _)) = steps.last() {
        points.push(v(board.at(0, c)[0], board.bottom() - OUTLET));
    }
    points
}

/// A drop of water at `at`. Detached.
fn drop_at(stage: &mut Stage<'_>, at: Vec3, color: Srgb) -> Mob {
    let unit = Circle::new()
        .radius(DROP_R)
        .style(flat(color, 1.0))
        .build()
        .moved_to(at);
    stage.arena_mut().add(unit)
}

/// Run a drop of water, `unit`, from `from` to `to`, [`SLOW`] seconds a
/// scene unit.
fn run(stage: &mut Stage<'_>, unit: Mob, from: Vec3, to: Vec3) -> fmn::Result<()> {
    let (dx, dy) = (to[0] - from[0], to[1] - from[1]);
    shift_all(stage, &[unit], dx, dy, (dx.hypot(dy) * SLOW).max(0.15))
}

/// The water drawn on the board: the wet squares by place, each row's tap
/// counter (what it has left to send), each column's drain counter (what it
/// still takes) and the units through (shown from 02_3). A tap counts down
/// when a drawn unit leaves it; its drain and the units through count when
/// that unit arrives. In between, the unit in transit is its drop or route.
pub(crate) struct Water {
    pub(crate) board: Board,
    pub(crate) wet: HashMap<(usize, usize), Mob>,
    pub(crate) taps: Vec<Mob>,
    tap_left: Vec<usize>,
    pub(crate) drains: Vec<Mob>,
    drain_left: Vec<usize>,
    pub(crate) through: Option<Mob>,
    through_count: usize,
}

impl Water {
    /// No water yet: `taps` and `drains` are the counters already drawn for
    /// shadows `s`, and nothing is through.
    pub(crate) fn new(board: Board, taps: Vec<Mob>, drains: Vec<Mob>, s: &Shadows) -> Self {
        Self {
            board,
            wet: HashMap::new(),
            taps,
            tap_left: s.rows.clone(),
            drains,
            drain_left: s.cols.clone(),
            through: None,
            through_count: 0,
        }
    }

    /// Count square (`r`, `c`) as wet and return its square, detached; the
    /// caller brings it in.
    fn place(&mut self, stage: &mut Stage<'_>, r: usize, c: usize) -> fmn::Result<Mob> {
        let m = self.board.square(stage, r, c, palette::filled())?;
        self.wet.insert((r, c), m);
        Ok(m)
    }

    /// Fill square (`r`, `c`).
    fn wet(&mut self, stage: &mut Stage<'_>, r: usize, c: usize, rt: f64) -> fmn::Result<()> {
        let m = self.place(stage, r, c)?;
        fade_in_all(stage, &[m], rt)
    }

    /// Take the water out of square (`r`, `c`); the caller animates it.
    fn take(&mut self, r: usize, c: usize) -> fmn::Result<Mob> {
        self.wet
            .remove(&(r, c))
            .ok_or_else(|| fail(format!("no water is drawn at ({r}, {c})")))
    }

    /// A unit leaves row `r`'s tap: the tap's counter drops by one as
    /// `appearing` (the unit's drop or route) comes in.
    fn depart(&mut self, kit: &Kit, stage: &mut Stage<'_>, r: usize, appearing: &[Mob], rt: f64) -> fmn::Result<()> {
        let left = self.tap_left[r]
            .checked_sub(1)
            .ok_or_else(|| fail(format!("row {r}'s tap has no water left")))?;
        let m = kit.text(
            stage,
            &left.to_string(),
            COUNT_SIZE,
            Palette::ink(),
            tap_at(&self.board, r),
        )?;
        let mut new = vec![m];
        new.extend_from_slice(appearing);
        swap(stage, &[self.taps[r]], &new, rt)?;
        (self.taps[r], self.tap_left[r]) = (m, left);
        Ok(())
    }

    /// A unit reaches column `c`'s drain: the drain's counter drops by one
    /// and the units through rise by one as `leaving` (its drop or route)
    /// goes.
    fn arrive(
        &mut self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        c: usize,
        total: usize,
        leaving: &[Mob],
        rt: f64,
    ) -> fmn::Result<()> {
        let left = self.drain_left[c]
            .checked_sub(1)
            .ok_or_else(|| fail(format!("column {c}'s drain is already full")))?;
        let m = kit.text(
            stage,
            &left.to_string(),
            COUNT_SIZE,
            Palette::ink(),
            self.board.col_count_at(c),
        )?;
        let mut old = vec![self.drains[c]];
        old.extend_from_slice(leaving);
        let mut new = vec![m];
        self.through_count += 1;
        if let Some(t) = self.through {
            let n = kit.text_left(
                stage,
                &through_text(self.through_count, total),
                26.0,
                Palette::ink(),
                v(TEXT_X, THROUGH_Y),
            )?;
            old.push(t);
            new.push(n);
            self.through = Some(n);
        }
        swap(stage, &old, &new, rt)?;
        (self.drains[c], self.drain_left[c]) = (m, left);
        Ok(())
    }

    /// After a whole path: the wet squares drawn, every tap's and drain's
    /// counter and the units through are exactly the replayed state's.
    fn settled(&self, s: &Shadows, state: &FlowState) -> fmn::Result<()> {
        let mut drawn: Vec<(usize, usize)> = self.wet.keys().copied().collect();
        drawn.sort_unstable();
        let taps: Vec<usize> = s.rows.iter().zip(&state.sent).map(|(n, sent)| n - sent).collect();
        let drains: Vec<usize> = s.cols.iter().zip(&state.drained).map(|(n, took)| n - took).collect();
        if drawn == state.wet.filled()
            && taps == self.tap_left
            && drains == self.drain_left
            && state.through() == self.through_count
        {
            Ok(())
        } else {
            Err(fail(format!(
                "drawn water {drawn:?}, taps {:?}, drains {:?} are not the replayed {} after {} units",
                self.tap_left,
                self.drain_left,
                state.wet.pattern(),
                state.through()
            )))
        }
    }
}

/// Send a path quickly: its route comes in as its tap counts down, its
/// squares wet (or dry, for water taken back), the route goes as its drain
/// and the units through count, and everything drawn is checked against the
/// replayed state.
pub(crate) fn send_quick(
    kit: &Kit,
    stage: &mut Stage<'_>,
    water: &mut Water,
    s: &Shadows,
    path: &Augmenting,
    state: &FlowState,
) -> fmn::Result<()> {
    let steps = path.steps();
    let (Some(&(tap, _, _)), Some(&(_, drain, _))) = (steps.first(), steps.last()) else {
        return Err(fail(format!("a path through no square: {:?}", path.nodes)));
    };
    let route = path_line(stage, &route_points(&water.board, &steps), palette::beam(), 4.0);
    water.depart(kit, stage, tap, &[route], 0.12)?;
    for &(r, c, wets) in &steps {
        if wets {
            water.wet(stage, r, c, 0.15)?;
        } else {
            let m = water.take(r, c)?;
            fade_out_all(stage, &[m], 0.15)?;
        }
    }
    water.arrive(kit, stage, drain, s.row_total(), &[route], 0.12)?;
    water.settled(s, state)
}

impl SceneConstruct for WaterThroughTheGrid {
    fn name(&self) -> &str {
        "02_water_through_the_grid"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let face = model::face();
        let s = face.shadows();
        let total = s.row_total();
        let (paths, states) = facts::face_flow().map_err(fail)?;
        let k = facts::first_takeback(&paths).map_err(fail)?;
        // the chapter's shape: one unit shown slowly, a run of units straight
        // down, then the last unit, which takes water back
        let straight = paths[..k].iter().all(|p| p.steps().len() == 1);
        if k < 2 || k + 1 != paths.len() || !straight {
            return Err(fail(format!(
                "the face's flow is not straight units and then one take-back: {k} of {}",
                paths.len()
            )));
        }
        let board = Board::new(AT, CELL, face.rows(), face.cols());

        // 02_1: the grid, and a tap for each row with its count
        let line = say(&kit, stage, "02_1")?;
        let grid = board.lines(stage);
        fade_in_all(stage, &grid, 0.6)?;
        line.wait_for(stage, "Each row's tap")?;
        let mut taps = Vec::with_capacity(board.rows);
        for (r, n) in s.rows.iter().enumerate() {
            taps.push(kit.text(stage, &n.to_string(), COUNT_SIZE, Palette::ink(), tap_at(&board, r))?);
        }
        let send_label = kit.text(
            stage,
            TO_SEND,
            22.0,
            Palette::dim(),
            v(board.left() - TAP_GAP, board.top() + 0.4),
        )?;
        fade_in_lagged(stage, &taps, 0.9, 0.3)?;
        let tap_line = kit.text_left(stage, TAP_LINE, 26.0, Palette::dim(), v(TEXT_X, LEGEND_Y[0]))?;
        fade_in_all(stage, &[send_label, tap_line], 0.4)?;

        // 02_2: a drain under each column; the first real unit, slowly
        let line = say(&kit, stage, "02_2")?;
        let mut drains = Vec::with_capacity(board.cols);
        for (c, n) in s.cols.iter().enumerate() {
            drains.push(kit.text(stage, &n.to_string(), COUNT_SIZE, Palette::ink(), board.col_count_at(c))?);
        }
        let drain_label = kit.text(
            stage,
            TO_DRAIN,
            22.0,
            Palette::dim(),
            v(board.left() - 1.0, board.col_count_at(0)[1]),
        )?;
        fade_in_lagged(stage, &drains, 0.8, 0.3)?;
        fade_in_all(stage, &[drain_label], 0.3)?;
        let mut water = Water::new(board, taps, drains, &s);
        let first = paths[0].steps();
        let &[(r0, c0, true)] = first.as_slice() else {
            return Err(fail(format!(
                "the first unit is not one square straight down: {first:?}"
            )));
        };
        let points = route_points(&board, &first);
        let unit = drop_at(stage, points[0], palette::filled());
        water.depart(&kit, stage, r0, &[unit], 0.3)?;
        run(stage, unit, points[0], points[1])?;
        line.wait_for(stage, "turns down")?;
        water.wet(stage, r0, c0, 0.3)?;
        line.wait_for(stage, "drains out")?;
        run(stage, unit, points[1], points[2])?;
        water.arrive(&kit, stage, c0, total, &[unit], 0.3)?;
        water.settled(&s, &states[0])?;

        // 02_3: a square carries at most one unit, a drain at most its
        // count; every unit through makes a picture that fits
        let line = say(&kit, stage, "02_3")?;
        let one = water.wet[&(r0, c0)];
        let square_text = kit.text_left(stage, &square_line(), 26.0, Palette::dim(), v(TEXT_X, LEGEND_Y[1]))?;
        flash_all(stage, &[one], 0.5)?;
        fade_in_all(stage, &[square_text], 0.3)?;
        line.wait_for(stage, "each drain holds")?;
        let drain_line = kit.text_left(stage, DRAIN_LINE, 26.0, Palette::dim(), v(TEXT_X, LEGEND_Y[2]))?;
        flash_all(stage, &water.drains, 0.5)?;
        fade_in_all(stage, &[drain_line], 0.3)?;
        line.wait_for(stage, "Get every unit through")?;
        let fits_line = kit.text_left(stage, FITS_LINE, 26.0, Palette::ink(), v(TEXT_X, LEGEND_Y[3]))?;
        let counter = kit.text_left(
            stage,
            &through_text(water.through_count, total),
            26.0,
            Palette::ink(),
            v(TEXT_X, THROUGH_Y),
        )?;
        fade_in_all(stage, &[fits_line, counter], 0.4)?;
        water.through = Some(counter);

        // 02_4: the next units, as the method sent them, one at a time; the
        // method for programmers
        let line = say(&kit, stage, "02_4")?;
        let code = lower_third(&kit, stage, PROGRAMMERS, &method_text())?;
        swap(stage, &[tap_line, square_text, drain_line, fits_line], &[code], 0.4)?;
        for (path, state) in paths[1..k].iter().zip(&states[1..k]) {
            send_quick(&kit, stage, &mut water, &s, path, state)?;
        }
        line.wait_to_end(stage)?;

        // 02_5: stuck without taking back: the bottom row has water left,
        // and each of its dry squares leads to a full drain
        let line = say(&kit, stage, "02_5")?;
        fade_out_all(stage, &[code], 0.3)?;
        let before = &states[k - 1];
        let steps = paths[k].steps();
        let &[(stuck, into, true), (from_row, back, false), (_, onto, true)] = steps.as_slice() else {
            return Err(fail(format!("the last unit is not in, back and on: {steps:?}")));
        };
        if back != into || stuck + 1 != board.rows || model::forward_room(&s, before) {
            return Err(fail("the last unit does not start stuck in the bottom row".into()));
        }
        line.wait_for(stage, "The bottom row")?;
        flash_all(stage, &[water.taps[stuck]], 0.5)?;
        line.wait_for(stage, "its dry squares")?;
        let dry: Vec<usize> = (0..board.cols).filter(|&c| !before.wet.get(stuck, c)).collect();
        if dry.iter().any(|&c| before.drained[c] < s.cols[c]) {
            return Err(fail("a dry square of the bottom row leads to a drain with room".into()));
        }
        let routes: Vec<Mob> = dry
            .iter()
            .map(|&c| {
                let points = route_points(&board, &[(stuck, c, true)]);
                path_line(stage, &points, palette::beam(), 4.0)
            })
            .collect();
        fade_in_all(stage, &routes, 0.4)?;
        let full: Vec<Mob> = dry.iter().map(|&c| water.drains[c]).collect();
        flash_all(stage, &full, 0.6)?;
        fade_out_all(stage, &routes, 0.3)?;
        let q = think(&kit, stage, v(THINK_AT[0], THINK_AT[1]))?;

        // 02_6: the unit runs into the first column anyway and waits at its
        // square; on "take back", in one move, that square fills and the top
        // row's unit is taken back up out of the same column (every row and
        // column stays balanced, the taken-back unit in transit as a drop)
        let line = say(&kit, stage, "02_6")?;
        fade_out_all(stage, &[q], 0.3)?;
        let points = route_points(&board, &steps);
        line.wait_for(stage, "Send it into")?;
        let unit = drop_at(stage, points[0], palette::filled());
        water.depart(&kit, stage, stuck, &[unit], 0.2)?;
        run(stage, unit, points[0], points[1])?;
        line.wait_for(stage, "take back")?;
        let filled = water.place(stage, stuck, into)?;
        let taken = water.take(from_row, back)?;
        let back_route = path_line(stage, &[points[1], points[2]], palette::taken_back(), 5.0);
        let returned = drop_at(stage, points[2], palette::taken_back());
        swap(stage, &[unit, taken], &[filled, back_route, returned], 0.5)?;

        // 02_7: the top row sends it on to the fourth column, which still
        // needs one; every drain full. Without taking back, one unit short
        let line = say(&kit, stage, "02_7")?;
        line.wait_for(stage, "sends that unit")?;
        run(stage, returned, points[2], points[3])?;
        water.wet(stage, from_row, onto, 0.3)?;
        run(stage, returned, points[3], points[4])?;
        water.arrive(&kit, stage, onto, total, &[returned, back_route], 0.3)?;
        water.settled(&s, &states[k])?;
        line.wait_for(stage, "Without taking back")?;
        let without = kit.text_left(
            stage,
            &without_text(before.through(), total),
            24.0,
            Palette::dim(),
            v(TEXT_X, WITHOUT_Y),
        )?;
        fade_in_all(stage, &[without], 0.4)?;

        // 02_8: the counters become the water's picture's shadows, the
        // face's; outlined, the face is another picture; one of the gallery
        let line = say(&kit, stage, "02_8")?;
        let built = states[k].wet.clone();
        if built.shadows() != s || built == face {
            return Err(fail(format!(
                "{} is not another picture with the face's shadows",
                built.pattern()
            )));
        }
        let mut leaving: Vec<Mob> = water.through.into_iter().collect();
        leaving.extend([without, send_label, drain_label]);
        fade_out_all(stage, &leaving, 0.3)?;
        let mut shadows = Vec::with_capacity(board.rows + board.cols);
        for (r, n) in built.row_counts().iter().enumerate() {
            shadows.push(kit.text(stage, &n.to_string(), COUNT_SIZE, palette::matches(), tap_at(&board, r))?);
        }
        for (c, n) in built.col_counts().iter().enumerate() {
            shadows.push(kit.text(
                stage,
                &n.to_string(),
                COUNT_SIZE,
                palette::matches(),
                board.col_count_at(c),
            )?);
        }
        let mut counters = water.taps.clone();
        counters.extend(&water.drains);
        swap(stage, &counters, &shadows, 0.5)?;
        line.wait_for(stage, "but they aren't")?;
        let rings: Vec<Mob> = face
            .filled()
            .iter()
            .map(|&(r, c)| board.ring(stage, r, c, Palette::ink()))
            .collect();
        let outlined = kit.text(stage, OUTLINED, 22.0, Palette::ink(), v(AT[0], board.bottom() - 1.05))?;
        fade_in_lagged(stage, &rings, 0.8, 0.3)?;
        fade_in_all(stage, &[outlined], 0.3)?;
        line.wait_for(stage, "They're one of")?;
        let gallery = facts::gallery();
        let n = gallery.len();
        let mut thumbs = Vec::with_capacity(2 * n);
        let mut found = Vec::new();
        for (j, (p, d)) in gallery.iter().enumerate() {
            let frame = if *p == built {
                palette::matches()
            } else if *d == 0 {
                Palette::ink()
            } else {
                Palette::faint()
            };
            let mobs = thumb(stage, p, gallery_at(j, n), THUMB_CELL, frame)?;
            if *p == built {
                found.clone_from(&mobs);
            }
            thumbs.extend(mobs);
        }
        if found.is_empty() {
            return Err(fail("the water's picture is not in the gallery".into()));
        }
        let middle = (gallery_at(0, n)[0] + gallery_at(n - 1, n)[0]) / 2.0;
        let caption = kit.text(
            stage,
            &found_label(n),
            24.0,
            Palette::ink(),
            v(middle, gallery_at(0, n)[1] + GALLERY_COUNT_Y),
        )?;
        fade_in_lagged(stage, &thumbs, 1.0, 0.05)?;
        fade_in_all(stage, &[caption], 0.3)?;
        flash_all(stage, &found, 0.5)?;

        // 02_9: water finds a picture whenever one exists, and never says
        // whether it is the only one
        let line = say(&kit, stage, "02_9")?;
        let card = precisely(&kit, stage, &flow_claim())?;
        fade_in_all(stage, &[card], 0.4)?;
        line.wait_for(stage, "It never says")?;
        flash_all(stage, &thumbs, 0.6)?;
        line.wait_to_end(stage)?;
        pause(stage, 1.2)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::*;

    /// The narrowest a lower-third may wrap a card to that the cards here are
    /// held to: the band wraps at 92 characters (`props::lower_third`),
    /// narrower while a line would run past its label's room.
    const NARROWEST_WRAP: usize = 80;

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("02_1", "Each row's tap"),
            ("02_2", "turns down"),
            ("02_2", "drains out"),
            ("02_3", "each drain holds"),
            ("02_3", "Get every unit through"),
            ("02_5", "The bottom row"),
            ("02_5", "its dry squares"),
            ("02_6", "Send it into"),
            ("02_6", "take back"),
            ("02_7", "sends that unit"),
            ("02_7", "Without taking back"),
            ("02_8", "but they aren't"),
            ("02_8", "They're one of"),
            ("02_9", "It never says"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The words on screen are built from the facts, and the two cards fit
    /// in two lines.
    #[test]
    fn the_words_on_screen_are_built_from_the_facts() {
        let f = facts::facts().unwrap();
        assert_eq!(
            square_line(),
            format!("square: carries at most {} unit", f.get("DT-F8.square").unwrap())
        );
        let total = f.get("DT-F8.through").unwrap();
        let before = f.get("DT-F8.before.through").unwrap();
        assert_eq!(
            without_text(before.parse().unwrap(), total.parse().unwrap()),
            "without taking back: 8 of 9"
        );
        assert_eq!(
            found_label(f.get("DT-F3.fits").unwrap().parse().unwrap()),
            "the water's picture is one of 64"
        );
        let claim = flow_claim();
        assert!(
            claim.contains("Ford and Fulkerson, 1957") && claim.contains("Gale, 1957"),
            "{claim}"
        );
        // both conditions of Gale's equivalence: one total, and the flow
        // reaching it (rows 1, 1 and columns 2, 1 reach the row total 2 and
        // fit nothing)
        assert!(
            claim.contains("both shadows share one total") && claim.contains("the flow reaches it"),
            "{claim}"
        );
    }

    /// Both cards keep two whole lines at every width the band may narrow
    /// them to, down to [`NARROWEST_WRAP`]; and by the corpus's estimate of
    /// a character's width (bayesian_search's 0.0045 scene units a point, an
    /// estimate the layout lint and the frames check), neither label leaves
    /// room for fewer characters than that.
    #[test]
    fn the_cards_stay_two_lines_when_the_band_narrows() {
        use explainer_kit::animatic::wrap;
        for card in [flow_claim(), method_text()] {
            for width in (NARROWEST_WRAP..=92).step_by(4) {
                let lines = wrap(&card, width, 3);
                assert!(
                    lines.len() <= 2 && !lines.iter().any(|l| l.ends_with('…')),
                    "{width}: {lines:?}"
                );
            }
        }
        let per_char = 26.0 * 0.0045;
        let band = 2.0 * FRAME_X_RADIUS - 0.8;
        for label in [PROGRAMMERS, "precisely:"] {
            let room = band - 0.7 - label.chars().count() as f64 * per_char;
            assert!(room / per_char >= NARROWEST_WRAP as f64, "{label}");
        }
    }

    /// Routes run along rows and columns only, from left of the grid to
    /// under it, through every square the path touches.
    #[test]
    fn routes_run_along_rows_and_columns() {
        let board = Board::new(AT, CELL, 5, 5);
        let (paths, _) = facts::face_flow().unwrap();
        for p in &paths {
            let steps = p.steps();
            let points = route_points(&board, &steps);
            assert_eq!(points.len(), steps.len() + 2);
            for leg in points.windows(2) {
                let (dx, dy) = ((leg[1][0] - leg[0][0]).abs(), (leg[1][1] - leg[0][1]).abs());
                assert!(dx < 1e-9 || dy < 1e-9, "a diagonal leg in {:?}", p.nodes);
            }
            assert!(points[0][0] < board.left());
            assert!(points.last().unwrap()[1] < board.bottom());
        }
    }

    /// The chapter's layout keeps its pieces apart and inside the
    /// title-safe area (text 0.25 in from the frame's edge).
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        let board = Board::new(AT, CELL, 5, 5);
        let safe_x = FRAME_X_RADIUS - 0.25;
        let safe_y = FRAME_Y_RADIUS - 0.25;
        // the taps and their label, the drains' label
        assert!(tap_at(&board, 0)[0] - 0.5 > -safe_x);
        assert!(board.left() - 1.0 - 0.7 > -safe_x, "the drains' label");
        assert!(board.top() + 0.4 < safe_y);
        // the words on the right clear the grid, and the gallery clears both
        assert!(TEXT_X > board.right() + 0.5);
        let half = THUMB_CELL * 5.0 / 2.0 + 0.03;
        assert!(gallery_at(0, 64)[0] - half > board.right() + 0.5);
        // the think ring's note (about 2.2 wide, centred under the ring)
        // clears the units-through counter (about 2 wide) on its left
        const { assert!(THINK_AT[0] - 1.1 > TEXT_X + 2.0, "the think note clears the counter") };
        // the outlined label and the drains stay over a two-line lower-third
        assert!(board.bottom() - 1.05 - 0.2 > -FRAME_Y_RADIUS + 1.25);
        assert!(board.col_count_at(0)[1] - 0.2 > -FRAME_Y_RADIUS + 1.25);
    }
}
