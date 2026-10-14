//! Chapter 08, where next: the loop keeps turning by itself on the made-up
//! sea, each search's dots dropping onto a stack whose top traces a curve
//! (08_1, a question to carry); the stack comes apart into one column a
//! cell, each a row of shrinking slices of the first hundred dots (08_2);
//! the eight biggest slices, taken biggest first, are the same curve, which
//! is why no other order is ever ahead (08_3, the proof in a picture); and on
//! the Navy's map the rule, row by row, and a map never updated are three
//! curves growing to twenty searches (08_4, 08_5).
//!
//! The proof is the picture: a search of a cell puts out a fixed slice of
//! the first hundred dots (`model::sea::slices`), whatever is searched in
//! between; a cell's slices only shrink; so the best n searches are the n
//! biggest slices, and the rule takes them in that order. Every number shown
//! is a `model::sea` or `model::y1968` value or a fact (`F-U.*`, `F-R.*`).

use std::collections::HashSet;
use std::rc::Rc;

use explainer_kit::animatic::HOLD_S;
use explainer_kit::props::{CodePanel, lower_third};
use explainer_kit::{Frame2, Kit, Palette, Timed, fade_all, play, rect, thousands, tween, v};
use fmn::prelude::*;

use super::draw::{
    Heat, Lattice, NavyGrid, Probe, QUIZ_LINES_Y, Sea, Strip, carry_pause, fade_in_all, fade_in_lagged, fade_out_all,
    fail, find, flat, line_style, map, out, say, sea_slot, search, tween_linear,
};
use super::glyphs;
use crate::facts::{facts, shown};
use crate::model::{Q, sea, y1968};

pub struct WhereNext {
    pub kit: Kit,
}

/// The narration lines this chapter speaks, in order.
pub const LINES: [&str; 5] = ["08_1", "08_2", "08_3", "08_4", "08_5"];

// the chapter's fixed words (numbers are built from facts and the model)
const X_LABEL: &str = "search";
const Y_LABEL: &str = "dots put out so far, of the first hundred";
const BY_ROW: &str = "no map: row by row";
const BY_RULE: &str = "the rule";
const NEVER_UPDATED: &str = "the map, never updated: each cell once, likeliest first";
const CHEW_LABEL: &str = "M. C. Chew,";
/// Chew's theorem as he stated it, P[N > n]: the chance of not having found
/// it yet (the draft's "of still searching" garbled it; R2: D4, R8).
const CHEW_TEXT: &str = "after any number of searches, the least chance of not having found it yet.";
const MODEL: &str = "model:";
const CHANCE_BY_NOW: &str = "chance of a find by now, according to the map";

// R2's words on the page (every one is a label, none is spoken)
/// 08_2: the numbers on the bars are rounded, and say so.
const ROUNDED_NOTE: &str = "dots each search puts out, of the first hundred; numbers rounded to one decimal";
/// 08_3: "fresh try" glossed, and what moving between cells costs.
const FRESH_TRY: &str = "fresh try: a miss now says nothing about the next try;\nthe same chance of seeing each time.\nMoving between cells costs nothing here";
/// 08_3: what the curve's height means, so that "ahead" has a direction.
const AHEAD: &str = "ahead: more of the first\nhundred dots out, after the\nsame number of searches";
/// 08_5: where the row-by-row plan starts, and the mark for no map at all.
const FROM_THE_NORTH: &str = "from the north edge, where the map is nearly empty";
const RANDOM_ORDER: &str = "no map, any order, on average:";
/// 08_5's hold: the gap between the top two curves, with no number of its
/// own (what updating adds depends on how long the search runs).
const WHAT_UPDATING_ADDS: &str = "what updating adds";

// ------------------------------------------------------------------ geometry

/// The stack's window, in the right half: searches across, dots put out up.
fn stack_window() -> Frame2 {
    Frame2 {
        center: v(3.6, 1.15),
        width: 6.0,
        height: 3.4,
        x: (0.0, 9.0),
        y: (0.0, 100.0),
    }
}

/// A stacked bar's width in the stack window's own units.
const BAR_W: f64 = 0.7;

/// The columns of 08_2, in scene units: a bar's width and pitch, the gap
/// between two cells' columns, the baseline and the height of a dot.
const COL_BAR_W: f64 = 0.5;
const COL_PITCH: f64 = 0.75;
const COL_GAP: f64 = 0.9;
const COL_BASE: f64 = -1.1;
const COL_PER_DOT: f64 = 0.13;

/// 08_1's loop, seconds a search takes: the gold border's jump, the strip's
/// crossing and the dots dropping onto the stack (eight at about 0.6 s end
/// with the line, not seconds after it).
const LOOP_JUMP_S: f64 = 0.12;
const LOOP_STRIP_S: f64 = 0.2;
const LOOP_DROP_S: f64 = 0.3;

/// How far under a window's bottom its x label stands (its tick numbers
/// stand 0.26 under it).
const X_LABEL_DROP: f64 = 0.6;
/// How far under the stack's window the count of orders stands: a row under
/// the x label, which it would otherwise run on into ("= 65,536 search").
const BRUTE_DROP: f64 = 1.05;

/// 08_3's code panel, in the left half the stack leaves empty: type size,
/// width, height of its centre.
const CODE_PT: f64 = 24.0;
const CODE_W: f64 = 5.8;
const CODE_Y: f64 = -0.3;

/// 08_5's legend: the first row's height and the rows' pitch, and how far
/// the two nearest curves' end labels move apart (up for the rule, down for
/// the map never updated), so 49.8% and 44.3% never touch.
const LEGEND_Y: f64 = -1.85;
const LEGEND_PITCH: f64 = 0.32;
const END_LABEL_SPREAD: f64 = 0.12;

/// 08_2's R2 labels. A slice's number is set at `SLICE_PT` over its bar; the
/// note that they are rounded stands under the names; B's first bar and its
/// second are outlined (`OUTLINE_PAD` clear of the bar) and tagged at
/// `TAG_PT`, each tag a little over its bar's number.
const SLICE_PT: f64 = 24.0;
const ROUNDED_Y: f64 = -2.2;
const OUTLINE_PAD: f64 = 0.05;
const TAG_PT: f64 = 22.0;
const TAG_RISE_FIRST: f64 = 0.42;
const TAG_RISE_SECOND: f64 = 0.34;

/// 08_3's R2 labels: the gloss's centre, in the left half over the code
/// panel's place, and the "ahead" label's centre in the stack window's own
/// units (searches across, dots up): under the staircase the bars make.
const FRESH_AT: [f64; 2] = [-3.6, 2.1];
const FRESH_PT: f64 = 22.0;
const AHEAD_AT: [f64; 2] = [6.4, 22.0];
const AHEAD_PT: f64 = 20.0;

/// 08_4's two maps: the Navy's grid on the left and the model's lattice on
/// the right, their middles and cell heights; the model line's height.
const GRID_AT: [f64; 2] = [-4.9, 0.55];
const GRID_CELL_H: f64 = 0.26;
const LATTICE_AT: [f64; 2] = [4.9, 0.55];
const LATTICE_CELL_H: f64 = 0.2;
const MODEL_Y: f64 = -3.0;
/// 08_4's note under the row-by-row grid: type size (its widest line is no
/// wider than the grid, which is 3.3 units across) and how far under the
/// grid's bottom its middle stands.
const ROW_NOTE_PT: f64 = 20.0;
const ROW_NOTE_DROP: f64 = 0.65;

/// 08_5's R2 labels: the axis note over the chart's own label (type size
/// and how far over the label's row its middle stands), the no-map mark's
/// words (size, where the words begin in searches, how far over the dashed
/// line they stand) and the hold's gap label.
const AXIS_NOTE_PT: f64 = 20.0;
const AXIS_NOTE_RISE: f64 = 0.85;
const MARK_PT: f64 = 18.0;
const MARK_FROM: f64 = 6.0;
const MARK_RISE: f64 = 0.17;
const GAP_PT: f64 = 18.0;
/// The bracket stands this far inside the chart's right edge, the gap
/// label's right end this far left of it, and its line runs this far over
/// the rule's end to meet the label.
const GAP_BRACKET_INSET: f64 = 0.1;
const GAP_LABEL_CLEAR: f64 = 0.08;
const GAP_LEADER: f64 = 0.27;
/// How many dashes the no-map mark is drawn in.
const MARK_DASHES: usize = 20;

/// One search of the rule over the made-up sea, with what it puts out of the
/// first hundred dots and what is still lit.
struct Step {
    cell: usize,
    /// How many times this cell was searched before this search.
    nth: usize,
    /// Dots of the first hundred this search puts out: the cell's next slice.
    slice: Q,
    /// Dots put out by this search and every one before it.
    cumulative: Q,
    /// Lit dots of the first hundred in each cell after this search.
    lit: [Q; 4],
}

fn the_rule(searches: usize) -> Vec<Step> {
    let dots = Q::int(sea::DOTS);
    let mut taken = [0usize; 4];
    let mut before = Q::ZERO;
    let mut out = Vec::with_capacity(searches);
    for s in sea::run(sea::Rule::Best, searches) {
        let nth = taken[s.cell];
        taken[s.cell] += 1;
        let cumulative = s.found_by * dots;
        let unfound = (Q::ONE - s.found_by) * dots;
        out.push(Step {
            cell: s.cell,
            nth,
            slice: cumulative - before,
            cumulative,
            lit: s.after.map(|p| p * unfound),
        });
        before = cumulative;
    }
    out
}

/// How many slices each cell's column shows: the ones the rule took and the
/// next one, which it did not (so the eighth taken and the ninth left stand
/// side by side).
fn column_sizes(steps: &[Step]) -> [usize; 4] {
    let mut n = [0usize; 4];
    for s in steps {
        n[s.cell] += 1;
    }
    n.map(|k| k + 1)
}

/// Where slice `j` of cell `c`'s column stands (its centre, scene x).
fn column_x(sizes: &[usize; 4], c: usize, j: usize) -> f64 {
    let bars: usize = sizes.iter().sum();
    let total = bars as f64 * COL_PITCH - (COL_PITCH - COL_BAR_W) + 3.0 * COL_GAP;
    let mut x = -total / 2.0;
    for &k in &sizes[..c] {
        x += k as f64 * COL_PITCH + COL_GAP;
    }
    x + j as f64 * COL_PITCH + COL_BAR_W / 2.0
}

/// Slice `j` (0-based) of cell `c`, in dots of the first hundred.
fn slice(c: usize, j: usize) -> Q {
    sea::slices(c, (j + 1) as u32)[j] * Q::int(sea::DOTS)
}

/// What B's first bar's tag says its search put out (R2: M3): the slice to
/// the whole dot, over the first hundred.
fn first_tag_text() -> String {
    format!(
        "this search put out {} of the first {}",
        slice(1, 0).fixed(0),
        sea::DOTS
    )
}

/// What B's second bar's tag says: its slice, to the one decimal the bars
/// carry.
fn second_tag_text() -> String {
    format!("this one {}", slice(1, 1).fixed(1))
}

/// The first tag's middle: over B's first bar and its number.
fn first_tag_at(sizes: &[usize; 4]) -> Vec3 {
    let top = COL_BASE + slice(1, 0).f64() * COL_PER_DOT;
    v(column_x(sizes, 1, 0), top + 0.26 + TAG_RISE_FIRST)
}

/// The second tag's left edge and height: B's second bar's own left edge, so
/// that the words run on over the shorter bars to its right and never over
/// the tall one to its left.
fn second_tag_at(sizes: &[usize; 4]) -> Vec3 {
    let top = COL_BASE + slice(1, 1).f64() * COL_PER_DOT;
    v(column_x(sizes, 1, 1) - COL_BAR_W / 2.0, top + 0.26 + TAG_RISE_SECOND)
}

/// A gold outline round the column bar of `dots` dots at scene x `x`.
fn bar_outline(stage: &mut Stage<'_>, x: f64, dots: Q) -> fmn::Result<Mob> {
    let h = dots.f64() * COL_PER_DOT;
    let r = Rectangle::new()
        .width(COL_BAR_W + 2.0 * OUTLINE_PAD)
        .height(h + 2.0 * OUTLINE_PAD)
        .style(line_style(find(), 3.5))
        .build()?
        .moved_to(v(x, COL_BASE + h / 2.0));
    Ok(stage.arena_mut().add(r))
}

/// The tag under the row-by-row grid (R2: D1, R6, S7, S15): where the plan
/// with no map starts, and how little the map holds there. `first20` is the
/// dots in its first twenty cells (`F-R.model.sweep.first20.points`).
fn row_note_text(first20: u32, total: u32) -> String {
    format!(
        "from the north edge, a creeping line\nfrom one corner: its first twenty\ncells hold {first20} of the {} dots",
        thousands(u64::from(total))
    )
}

/// The axis note over 08_5's chart (R2: D2, R11): the percents are of all
/// the runs, and the ones that ended outside the grid are never searched.
fn axis_note_text(outside: u32, total: u32, cells: usize) -> String {
    format!(
        "percent of all {} runs: {} ended outside these {cells} cells\nand are never searched here",
        thousands(u64::from(total)),
        thousands(u64::from(outside))
    )
}

/// The points of a curve on `chart`: the origin, then the chance of a find
/// after each of `n` searches.
fn curve_points(chart: &Frame2, rule: y1968::Rule, n: usize) -> Vec<Vec3> {
    let mut pts = vec![chart.p(0.0, 0.0)];
    pts.extend(
        curve_percent(rule, n)
            .iter()
            .enumerate()
            .map(|(k, &p)| chart.p((k + 1) as f64, p)),
    );
    pts
}

/// 08_5's chart between the two maps, over twenty searches and then (in the
/// hold) over eighty-one: the same box, the same percent axis.
fn chart_for(searches: usize) -> Frame2 {
    Frame2 {
        center: v(0.0, 0.65),
        width: 4.4,
        height: 3.0,
        x: (0.0, searches as f64),
        y: (0.0, 100.0),
    }
}

fn chart_20() -> Frame2 {
    chart_for(shown::MAP_MARKS[0])
}

fn chart_81() -> Frame2 {
    chart_for(shown::MAP_MARKS[1])
}

/// One of 08_5's three plans: how it searches, how it is drawn, what the
/// legend calls it and which fact is its value at each mark. The row-by-row
/// plan's name carries where it starts (R2: D1, R6, S7).
struct Plan {
    rule: y1968::Rule,
    name: String,
    color: Srgb,
    slug: &'static str,
    /// Which way the value at its end moves, so that two near ends never touch.
    nudge: f64,
}

impl Plan {
    fn key(&self, searches: usize) -> String {
        format!("F-R.model.{}.found_by.{searches}", self.slug)
    }
}

fn plans() -> [Plan; 3] {
    [
        Plan {
            rule: y1968::Rule::Sweep,
            name: format!("{BY_ROW}, {FROM_THE_NORTH}"),
            color: Palette::dim(),
            slug: "sweep",
            nudge: 0.0,
        },
        Plan {
            rule: y1968::Rule::Once,
            name: NEVER_UPDATED.to_owned(),
            color: Palette::ink(),
            slug: "once",
            nudge: -END_LABEL_SPREAD,
        },
        Plan {
            rule: y1968::Rule::Best,
            name: BY_RULE.to_owned(),
            color: find(),
            slug: "best",
            nudge: END_LABEL_SPREAD,
        },
    ]
}

/// Where the gap label's right end stands: left of the bracket, level with
/// the bracket's top, which is a leader's length over the rule's end. The
/// gap itself is a wedge that opens to the right and climbs, too narrow
/// under its own label anywhere a label of this size would fit; over the
/// rule's curve there is room, and the bracket's own line runs up to it.
fn gap_label_right(_once_end: &Vec3, best_end: &Vec3) -> Vec3 {
    v(
        best_end[0] - GAP_BRACKET_INSET - GAP_LABEL_CLEAR,
        best_end[1] + GAP_LEADER,
    )
}

/// Where `n` dashes lie along a line of length one: each as long as the gap
/// after it, the first starting the line and the last ending it.
fn dash_spans(n: usize) -> Vec<(f64, f64)> {
    let parts = (2 * n.max(1) - 1) as f64;
    (0..n.max(1))
        .map(|i| (2.0 * i as f64 / parts, (2.0 * i as f64 + 1.0) / parts))
        .collect()
}

/// A dashed line from `a` to `b`: `n` dashes.
fn dashed(stage: &mut Stage<'_>, a: Vec3, b: Vec3, n: usize, color: Srgb) -> Vec<Mob> {
    let at = |t: f64| v(a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t);
    dash_spans(n)
        .into_iter()
        .map(|(t0, t1)| {
            stage
                .arena_mut()
                .add(polyline(vec![at(t0), at(t1)]).style(line_style(color, 3.0)).build())
        })
        .collect()
}

/// Code lines as the panel shows them: each line's code, then its `#`
/// comment on a line of its own, so no line is longer than the code itself
/// (a line with its comment ran 80 columns, and 17 points was what fitted).
fn code_over_comments<'a>(code: &[&'a str]) -> Vec<&'a str> {
    let mut out = Vec::with_capacity(2 * code.len());
    for &line in code {
        match line.find('#') {
            Some(at) => out.extend([line[..at].trim_end(), &line[at..]]),
            None => out.push(line),
        }
    }
    out
}

/// "4 × 4 × … × 4 = 65,536": every order of `searches` searches of `cells`
/// cells, as a product the viewer can count.
fn orders_line(cells: usize, searches: usize, total: u64) -> String {
    let product = vec![cells.to_string(); searches].join(" × ");
    format!("{product} = {}", thousands(total))
}

/// The model search over the 1968 map in percent after each of `looks`
/// searches.
fn curve_percent(rule: y1968::Rule, looks: usize) -> Vec<f64> {
    y1968::model_search(rule, shown::MAP_SEEING, looks)
        .iter()
        .map(|l| 100.0 * l.found_by.f64())
        .collect()
}

/// A live percent of the chance of a find (a cell's share of the lit dots
/// times the chance of seeing it), centred at `at`: one decimal only where
/// the value needs it.
fn find_readout(kit: &Kit, stage: &mut Stage<'_>, lit: [Mob; 4], cell: usize, d: f64, at: Vec3) -> Mob {
    let book = Rc::clone(&kit.book);
    stage.always_redraw(move |s| {
        let now: [f64; 4] = std::array::from_fn(|i| s.tracker_value(lit[i]).unwrap_or(0.0));
        let total: f64 = now.iter().sum();
        let value = if total > 0.0 {
            100.0 * now[cell] / total * d
        } else {
            0.0
        };
        let places = usize::from(((value * 10.0).round() as i64) % 10 != 0);
        let scale = if places == 1 { 10.0 } else { 1.0 };
        let value = (value * scale).round() / scale;
        let n = DecimalNumber::new(value)
            .num_decimal_places(places)
            .unit("%")
            .font_size(24.0)
            .color(find())
            .build(&book)
            .expect("bundled digits always lay out");
        let m = s.add(n.into_vmob());
        s.move_to(m, at, ORIGIN);
        m
    })
}

/// A window's border, ticks and labels, detached.
fn furniture(kit: &Kit, stage: &mut Stage<'_>, f: &Frame2, x_ticks: &[f64], y_label: &str) -> fmn::Result<Vec<Mob>> {
    let mut out = vec![stage.arena_mut().add(f.border(Palette::faint())?)];
    for &t in x_ticks {
        let p = f.p(t, 0.0);
        out.push(kit.text(stage, &format!("{t:.0}"), 22.0, Palette::dim(), v(p[0], p[1] - 0.26))?);
    }
    for pct in [0.0, 50.0, 100.0] {
        let p = f.p(f.x.0, pct);
        out.push(kit.text(stage, &format!("{pct:.0}"), 22.0, Palette::dim(), v(p[0] - 0.3, p[1]))?);
    }
    // under the row of ticks, not in it (the last tick's number was on it)
    out.push(kit.text(
        stage,
        X_LABEL,
        22.0,
        Palette::dim(),
        v(f.right() - 0.5, f.bottom() - X_LABEL_DROP),
    )?);
    out.push(kit.text_left(stage, y_label, 22.0, Palette::dim(), v(f.left(), f.top() + 0.28))?);
    Ok(out)
}

/// The rectangle with lower-left corner `at`, detached.
fn block(stage: &mut Stage<'_>, at: Vec3, w: f64, h: f64, color: Srgb) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(rect(at, w, h, flat(color, 0.95))?))
}

/// A stacked bar from `y0` to `y1` dots, centred on the window's x.
fn stacked(stage: &mut Stage<'_>, f: &Frame2, x: f64, y0: f64, y1: f64, color: Srgb) -> fmn::Result<Mob> {
    let (a, b) = (f.p(x - BAR_W / 2.0, y0), f.p(x + BAR_W / 2.0, y1));
    block(stage, a, b[0] - a[0], b[1] - a[1], color)
}

/// A column bar of `dots` dots standing on the baseline at scene x `x`.
fn standing(stage: &mut Stage<'_>, x: f64, dots: f64, color: Srgb) -> fmn::Result<Mob> {
    block(
        stage,
        v(x - COL_BAR_W / 2.0, COL_BASE),
        COL_BAR_W,
        dots * COL_PER_DOT,
        color,
    )
}

/// One segment of the curve the stack's top traces, from search `k` to
/// search `k + 1` (`before` and `after` dots put out).
fn trace(stage: &mut Stage<'_>, f: &Frame2, k: usize, before: f64, after: f64) -> Mob {
    stage.arena_mut().add(
        polyline(vec![f.p(k as f64 + 0.5, before), f.p(k as f64 + 1.5, after)])
            .style(line_style(Palette::ink(), 3.0))
            .build(),
    )
}

impl SceneConstruct for WhereNext {
    fn name(&self) -> &str {
        "08_where_next"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let all = facts().map_err(fail)?;
        let fact = |key: &str| -> fmn::Result<String> {
            all.get(key)
                .map(str::to_owned)
                .ok_or_else(|| fail(format!("no fact {key}")))
        };
        let steps = the_rule(shown::BRUTE);
        let sizes = column_sizes(&steps);
        let w = stack_window();
        let before_of = |k: usize| if k == 0 { Q::ZERO } else { steps[k - 1].cumulative };

        // ------------------------------------------------------- 08_1
        // the loop keeps turning by itself; each search's dots drop onto a
        // stack, and the stack's top traces a curve
        let line = say(&kit, stage, LINES[0])?;
        let prior: [u32; 4] = sea::prior().map(|p| {
            (p * Q::int(sea::DOTS))
                .fixed(0)
                .parse()
                .expect("the prior's dots are whole")
        });
        let seeing = sea::seeing();
        let the_sea = Sea::build(&kit, stage, prior)?;
        let mut cells = the_sea.mobs();
        cells.extend([the_sea.tag, the_sea.canyon]);
        let mut sums: Vec<Mob> = Vec::new();
        let mut live: Vec<Mob> = Vec::new();
        for (c, d) in seeing.iter().enumerate() {
            live.push(the_sea.percent_readout(&kit, stage, c, 24.0, map(), sea_slot(c, 0)));
            let words = d.to_string().replace('/', " in ");
            sums.push(kit.text(stage, &words, 22.0, search(), sea_slot(c, 1))?);
            live.push(find_readout(&kit, stage, the_sea.lit, c, d.f64(), sea_slot(c, 2)));
        }
        let ticks: Vec<f64> = (1..=shown::BRUTE).map(|k| k as f64).collect();
        let axes = furniture(&kit, stage, &w, &ticks, Y_LABEL)?;
        // the rule's choice is the cell with the biggest gold number: a gold
        // border jumps to it
        let first = the_sea.rect(steps[0].cell);
        let choice = stage.arena_mut().add(
            Rectangle::new()
                .width(first.2 - first.0)
                .height(first.3 - first.1)
                .style(line_style(find(), 4.0))
                .build()?
                .moved_to(v((first.0 + first.2) / 2.0, (first.1 + first.3) / 2.0)),
        );
        // the sea, the chart and the border come in together: the first
        // search begins while the first sentence is still being said
        let mut ins = cells.clone();
        ins.extend(&sums);
        ins.extend(&axes);
        ins.push(choice);
        fade_in_all(stage, &ins, 0.6)?;
        for &m in &live {
            stage.add_to_scene(m)?;
        }
        // the question the line ends on is set as its sentence begins, over
        // the loop, so it is up while it is spoken (a loop of eight searches
        // at a second each ran past the whole line)
        let words = line
            .sentence_texts()
            .pop()
            .ok_or_else(|| fail("08_1 has no sentences".into()))?;
        let question = kit.text(stage, &words, 26.0, Palette::ink(), v(0.0, QUIZ_LINES_Y[0]))?;
        let question_at = line.sentence(line.sentences() - 1)?;
        let mut question_in = false;
        let mut bars: Vec<Mob> = Vec::new();
        let mut tops: Vec<Mob> = Vec::new();
        for (k, s) in steps.iter().enumerate() {
            let r = the_sea.rect(s.cell);
            let at = v((r.0 + r.2) / 2.0, (r.1 + r.3) / 2.0);
            let here = stage.get_center(choice);
            let jump = choice
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(LOOP_JUMP_S),
                    ..AnimateArgs::default()
                })?
                .shift([at[0] - here[0], at[1] - here[1], 0.0])?;
            stage.play(jump)?;
            let strip = Strip::build(stage, r, 0.22 * (r.2 - r.0))?;
            for m in strip.mobs() {
                stage.add_to_scene(m)?;
            }
            stage.play(tween_linear(strip.phase, 1.0, LOOP_STRIP_S)?)?;
            // the cell's dots go out: every lit count moves (the heat and
            // both readouts follow), and the slice drops onto the stack
            let b = stacked(stage, &w, (k + 1) as f64, before_of(k).f64(), s.cumulative.f64(), out())?;
            bars.push(b);
            let top = trace(stage, &w, k, before_of(k).f64(), s.cumulative.f64());
            tops.push(top);
            let mut together: Vec<Box<dyn Animation>> = Vec::new();
            for (c, &t) in the_sea.lit.iter().enumerate() {
                together.push(stage.prepare(tween(t, s.lit[c].f64(), LOOP_DROP_S)?)?);
            }
            let drop = fade_in(stage.arena_mut(), b, v(0.0, 0.5), 1.0)?;
            together.push(stage.prepare(drop.rt(LOOP_DROP_S))?);
            let tracing = fade_in(stage.arena_mut(), top, ORIGIN, 1.0)?;
            together.push(stage.prepare(tracing.rt(LOOP_DROP_S))?);
            for m in [strip.wash, strip.bar] {
                let gone = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
                together.push(stage.prepare(gone.rt(LOOP_DROP_S * 0.75))?);
            }
            if !question_in && stage.scene().time().to_f64() >= question_at {
                let asked = fade_in(stage.arena_mut(), question, ORIGIN, 1.0)?;
                together.push(stage.prepare(asked.rt(0.4))?);
                question_in = true;
            }
            stage.play_prepared(together)?;
        }
        // the question to carry: the line's own last sentence, two seconds
        // (brought in here only if the loop ended before the sentence began)
        if !question_in {
            fade_in_all(stage, &[question], 0.4)?;
        }
        carry_pause(&kit, stage)?;
        fade_out_all(stage, &[question], 0.3)?;

        // ------------------------------------------------------- 08_2
        // the stack comes apart into four columns, one a cell, each a row
        // of shrinking slices of the first hundred dots
        let line = say(&kit, stage, LINES[1])?;
        let mut away = cells.clone();
        away.extend(&sums);
        away.extend(&live);
        away.extend(&axes);
        away.extend(&tops);
        away.push(choice);
        fade_out_all(stage, &away, 0.5)?;
        let mut morph: Vec<Box<dyn Animation>> = Vec::new();
        for (k, s) in steps.iter().enumerate() {
            let target = standing(stage, column_x(&sizes, s.cell, s.nth), s.slice.f64(), out())?;
            morph.push(stage.prepare(Transform::new(bars[k], target))?);
        }
        let group = AnimationGroup::with_lag_ratio(stage.arena_mut(), morph, 0.12)?.with_run_time(2.2);
        stage.play(group)?;
        // the slice each cell would put out next, which the rule did not take
        let mut spares: Vec<Mob> = Vec::new();
        for c in 0..4 {
            for j in 0..sizes[c] {
                if steps.iter().any(|s| s.cell == c && s.nth == j) {
                    continue;
                }
                spares.push(standing(stage, column_x(&sizes, c, j), slice(c, j).f64(), out())?);
            }
        }
        let mut names: Vec<Mob> = Vec::new();
        for (c, name) in sea::CELLS.iter().enumerate() {
            let mid = (column_x(&sizes, c, 0) + column_x(&sizes, c, sizes[c] - 1)) / 2.0;
            names.push(kit.text(stage, name, 28.0, Palette::dim(), v(mid, COL_BASE - 0.4))?);
            // every number to one decimal, as the facts print them, and
            // the note under the names says they are rounded (R2: M3)
            for j in 0..sizes[c] {
                let top = COL_BASE + slice(c, j).f64() * COL_PER_DOT;
                names.push(kit.text(
                    stage,
                    &slice(c, j).fixed(1),
                    SLICE_PT,
                    Palette::ink(),
                    v(column_x(&sizes, c, j), top + 0.26),
                )?);
            }
        }
        names.push(kit.text(stage, ROUNDED_NOTE, 22.0, Palette::dim(), v(0.0, ROUNDED_Y))?);
        let mut shown_now = spares.clone();
        shown_now.extend(&names);
        fade_in_all(stage, &shown_now, 0.6)?;
        // as "slice" is said, B's first bar is outlined and tagged with what
        // its search put out; its second, with how much less (R2: M3)
        line.wait_for(stage, "a fixed slice")?;
        let first = bar_outline(stage, column_x(&sizes, 1, 0), slice(1, 0))?;
        let first_tag = kit.text(stage, &first_tag_text(), TAG_PT, Palette::ink(), first_tag_at(&sizes))?;
        fade_in_all(stage, &[first, first_tag], 0.4)?;
        line.wait_for(stage, "slices only shrink")?;
        let second = bar_outline(stage, column_x(&sizes, 1, 1), slice(1, 1))?;
        let second_tag = kit.text_left(stage, &second_tag_text(), TAG_PT, Palette::ink(), second_tag_at(&sizes))?;
        fade_in_all(stage, &[second, second_tag], 0.4)?;
        line.wait_end(stage)?;

        // ------------------------------------------------------- 08_3
        // the eight tallest light gold and fly back, tallest first: the same
        // curve; no other order is ever ahead
        let line = say(&kit, stage, LINES[2])?;
        let mut back: Vec<Box<dyn Animation>> = Vec::new();
        for (k, s) in steps.iter().enumerate() {
            let target = stacked(
                stage,
                &w,
                (k + 1) as f64,
                before_of(k).f64(),
                s.cumulative.f64(),
                find(),
            )?;
            back.push(stage.prepare(Transform::new(bars[k], target))?);
        }
        let flight = AnimationGroup::with_lag_ratio(stage.arena_mut(), back, 0.35)?.with_run_time(3.4);
        let mut together: Vec<Box<dyn Animation>> = vec![stage.prepare(flight)?];
        let mut leaving = spares.clone();
        leaving.extend(&names);
        leaving.extend([first, first_tag, second, second_tag]);
        for &m in &leaving {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            together.push(stage.prepare(a.rt(0.7))?);
        }
        for &m in &axes {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            together.push(stage.prepare(a.rt(0.7))?);
        }
        stage.play_prepared(together)?;
        // the curve they make is the first one
        let mut again: Vec<Mob> = Vec::new();
        for (k, s) in steps.iter().enumerate() {
            again.push(trace(stage, &w, k, before_of(k).f64(), s.cumulative.f64()));
        }
        fade_in_all(stage, &again, 0.5)?;
        // "fresh try" is glossed on the page as it is said, beside the bars
        // (R2: D4, R8, S8)
        line.wait_for(stage, "searches that are fresh tries")?;
        let fresh = kit.text(stage, FRESH_TRY, FRESH_PT, Palette::dim(), v(FRESH_AT[0], FRESH_AT[1]))?;
        fade_in_all(stage, &[fresh], 0.5)?;
        line.wait_for(stage, "no other order")?;
        let total: u64 = fact("F-U.brute.orders")?
            .parse()
            .map_err(|_| fail("F-U.brute.orders is not a number".into()))?;
        let brute = kit.text(
            stage,
            &orders_line(sea::CELLS.len(), shown::BRUTE, total),
            22.0,
            Palette::dim(),
            v(w.center[0], w.bottom() - BRUTE_DROP),
        )?;
        // under the curve, which way is "ahead" (R2: R8)
        let ahead = kit.text(stage, AHEAD, AHEAD_PT, Palette::dim(), w.p(AHEAD_AT[0], AHEAD_AT[1]))?;
        fade_in_all(stage, &[brute, ahead], 0.5)?;
        line.wait_end(stage)?;
        // in the hold: the theorem, and the last two lines of the code panel,
        // each code line over its comment so that they are set large enough
        // to read, in the left half the stack leaves empty
        let chew = lower_third(&kit, stage, &format!("{CHEW_LABEL} {}:", fact("F-N.chew")?), CHEW_TEXT)?;
        let panel = CodePanel::build(&kit, stage, &code_over_comments(&glyphs::CODE[2..]), CODE_PT, CODE_W)?;
        // it is built to come in from the right; it comes in from the left
        stage.shift(panel.group, [-(2.0 * FRAME_X_RADIUS + CODE_W + 0.2), CODE_Y, 0.0]);
        fade_in_all(stage, &[chew], 0.5)?;
        stage.add_to_scene(panel.group)?;
        let slide = panel
            .group
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.8),
                ..AnimateArgs::default()
            })?
            .shift([CODE_W + 0.45, 0.0, 0.0])?;
        stage.play(slide)?;
        kit.music_breath(stage, HOLD_S)?;

        // ------------------------------------------------------- 08_4
        // on the Navy's map: no map, row by row; and the rule over the
        // printed map, searching E5, B7, C5 and G5 in turn
        let line = say(&kit, stage, LINES[3])?;
        let mut off: Vec<Mob> = vec![chew, panel.group, brute, fresh, ahead];
        off.extend(&bars);
        off.extend(&again);
        off.extend(&axes);
        let mut seen = HashSet::new();
        off.retain(|m| seen.insert(*m));
        fade_out_all(stage, &off, 0.7)?;

        let (left_at, right_at) = (v(GRID_AT[0], GRID_AT[1]), v(LATTICE_AT[0], LATTICE_AT[1]));
        let grid = NavyGrid::build(stage, left_at, GRID_CELL_H)?;
        let looks = y1968::model_search(y1968::Rule::Best, shown::MAP_SEEING, 4);
        let probes: Vec<Probe> = looks
            .iter()
            .map(|l| Probe {
                row: l.row,
                col: l.col,
                seeing: shown::MAP_SEEING,
            })
            .collect();
        let lattice = Lattice::build(stage, right_at, LATTICE_CELL_H, &probes, Heat::LATTICE)?;
        fade_in_lagged(stage, &grid.lines, 0.8, 0.5)?;
        fade_in_all(stage, &lattice.lines, 0.6)?;
        let mut under = vec![lattice.anchor];
        under.extend(&lattice.heat);
        fade_in_all(stage, &under, 0.4)?;
        fade_in_lagged(stage, &lattice.dots, 0.9, 0.5)?;
        let (gw, gh) = grid.size();
        let lh = lattice.size().1;
        let by_row = kit.text(
            stage,
            BY_ROW,
            24.0,
            Palette::dim(),
            v(left_at[0], left_at[1] + gh / 2.0 + 0.35),
        )?;
        let by_rule = kit.text(
            stage,
            BY_RULE,
            24.0,
            Palette::dim(),
            v(right_at[0], right_at[1] + lh / 2.0 + 0.35),
        )?;
        let model = kit.text(
            stage,
            &format!(
                "{MODEL} the lettered grid; every search sees {}; one search, about {} sea miles of towing",
                fact("F-R.model.seeing")?.replace('/', " in "),
                fact("F-R.model.search_miles")?
            ),
            22.0,
            Palette::dim(),
            v(0.0, MODEL_Y),
        )?;
        fade_in_all(stage, &[by_row, by_rule, model], 0.5)?;
        // no map: the strip along the grid's first row, west to east, and
        // under that grid where it starts and how little the map holds there
        // (R2: D1, R6, S7, S15; the 114 is the fact's, not a count made here)
        line.wait_for(stage, "One plan has no map")?;
        let first20: u32 = fact("F-R.model.sweep.first20.points")?
            .parse()
            .map_err(|_| fail("F-R.model.sweep.first20.points is not a number".into()))?;
        let row_note = kit.text(
            stage,
            &row_note_text(first20, y1968::total()),
            ROW_NOTE_PT,
            Palette::dim(),
            v(left_at[0], left_at[1] - gh / 2.0 - ROW_NOTE_DROP),
        )?;
        fade_in_all(stage, &[row_note], 0.4)?;
        let row_one = (
            left_at[0] - gw / 2.0,
            left_at[1] + gh / 2.0 - grid.cell_h,
            left_at[0] + gw / 2.0,
            left_at[1] + gh / 2.0,
        );
        let sweep = Strip::build(stage, row_one, 0.22 * grid.cell_w)?;
        for m in sweep.mobs() {
            stage.add_to_scene(m)?;
        }
        stage.play(tween_linear(sweep.phase, 1.0, 2.0)?)?;
        // the rule: E5 first, with the search sound, then the next three
        line.wait_for(stage, "The other follows the rule")?;
        for k in 0..probes.len() {
            lattice.search(&kit, stage, k, 0.8, 0.8)?;
        }
        line.wait_end(stage)?;

        // ------------------------------------------------------- 08_5
        // three curves between the maps, growing to twenty searches
        let line = say(&kit, stage, LINES[4])?;
        // the row-by-row note has said its piece; the legend stands there now
        fade_out_all(stage, &[row_note], 0.4)?;
        let n = shown::MAP_MARKS[0];
        let chart = chart_20();
        let chart_mobs = furniture(&kit, stage, &chart, &[0.0, n as f64 / 2.0, n as f64], CHANCE_BY_NOW)?;
        // the axis note: the percents are of all the runs, and the ones that
        // ended outside the grid are never searched here (R2: D2, R11)
        let outside: u32 = fact("F-R.outside.points")?
            .parse()
            .map_err(|_| fail("F-R.outside.points is not a number".into()))?;
        let cells = y1968::GRID_ROWS.len() * y1968::GRID_COLS.len();
        let axis_note = kit.text_left(
            stage,
            &axis_note_text(outside, y1968::total(), cells),
            AXIS_NOTE_PT,
            Palette::dim(),
            v(chart.left(), chart.top() + AXIS_NOTE_RISE),
        )?;
        let mut first_in = chart_mobs.clone();
        first_in.push(axis_note);
        fade_in_all(stage, &first_in, 0.5)?;
        let plans = plans();
        let mut curves: Vec<Mob> = Vec::new();
        let mut values: Vec<Mob> = Vec::new();
        let mut no_map: Vec<Mob> = Vec::new();
        for (i, plan) in plans.iter().enumerate() {
            let pts = curve_points(&chart, plan.rule, n);
            let end = *pts.last().ok_or_else(|| fail("no curve".into()))?;
            let curve = stage
                .arena_mut()
                .add(polyline(pts).style(line_style(plan.color, 4.0)).build());
            let value = kit.text_left(
                stage,
                &format!("{}%", fact(&plan.key(n))?),
                24.0,
                plan.color,
                v(end[0] + 0.15, end[1] + plan.nudge),
            )?;
            let y = LEGEND_Y - LEGEND_PITCH * i as f64;
            let swatch = stage.arena_mut().add(
                polyline(vec![v(-3.3, y), v(-2.9, y)])
                    .style(line_style(plan.color, 5.0))
                    .build(),
            );
            let words = kit.text_left(stage, &plan.name, 22.0, Palette::ink(), v(-2.75, y))?;
            play!(stage; show_creation(curve).rt(1.4));
            fade_in_all(stage, &[value, swatch, words], 0.4)?;
            curves.push(curve);
            values.push(value);
            if i == 0 {
                // against 0.8%: what no map at all, in any order, comes to
                // on average, a dashed mark at twenty searches (R2: D1, R6,
                // S7), its words over it in the room under the other two
                let level: f64 = fact(&format!("F-R.model.random.found_by.{n}"))?
                    .parse()
                    .map_err(|_| fail("F-R.model.random.found_by is not a number".into()))?;
                no_map = dashed(
                    stage,
                    chart.p(0.0, level),
                    chart.p(n as f64, level),
                    MARK_DASHES,
                    Palette::dim(),
                );
                let at = chart.p(MARK_FROM, level);
                no_map.push(kit.text_left(
                    stage,
                    &format!("{RANDOM_ORDER} {level}%"),
                    MARK_PT,
                    Palette::dim(),
                    v(at[0], at[1] + MARK_RISE),
                )?);
                fade_in_all(stage, &no_map, 0.4)?;
            }
        }
        line.wait_end(stage)?;
        kit.hold(stage)?;
        // in the hold the three curves run on to eighty-one searches, and
        // the gap between the top two says what updating adds, with no
        // number of its own (R2: M17)
        let long_n = shown::MAP_MARKS[1];
        let long = chart_81();
        let mut old: Vec<Mob> = chart_mobs;
        old.extend(&curves);
        old.extend(&values);
        old.extend(&no_map);
        fade_out_all(stage, &old, 0.4)?;
        let ticks: Vec<f64> = (0..=3).map(|k| (long_n / 3 * k) as f64).collect();
        let long_mobs = furniture(&kit, stage, &long, &ticks, CHANCE_BY_NOW)?;
        fade_in_all(stage, &long_mobs, 0.4)?;
        let mut runs: Vec<Mob> = Vec::new();
        let mut ends: Vec<Mob> = Vec::new();
        let mut tops: Vec<Vec3> = Vec::new();
        for plan in &plans {
            let pts = curve_points(&long, plan.rule, long_n);
            let end = *pts.last().ok_or_else(|| fail("no curve".into()))?;
            runs.push(
                stage
                    .arena_mut()
                    .add(polyline(pts).style(line_style(plan.color, 4.0)).build()),
            );
            // the end labels stand level with their curves' ends: the top
            // two are fifteen points apart now, not five
            ends.push(kit.text_left(
                stage,
                &format!("{}%", fact(&plan.key(long_n))?),
                24.0,
                plan.color,
                v(end[0] + 0.15, end[1]),
            )?);
            tops.push(end);
        }
        play!(
            stage, 2.0;
            show_creation(runs[0]),
            show_creation(runs[1]),
            show_creation(runs[2])
        );
        // the bracket between the rule's end and the map never updated's,
        // its line running on up to its words, which stand over the rule's
        // curve (the gap between the curves is too narrow to hold them)
        let (once_end, best_end) = (tops[1], tops[2]);
        let bracket = stage.arena_mut().add(
            polyline(vec![
                v(best_end[0] - GAP_BRACKET_INSET, once_end[1]),
                v(best_end[0] - GAP_BRACKET_INSET, best_end[1] + GAP_LEADER),
            ])
            .style(line_style(find(), 3.0))
            .build(),
        );
        let gap = kit.text(stage, WHAT_UPDATING_ADDS, GAP_PT, find(), ORIGIN)?;
        stage.move_to(gap, gap_label_right(&once_end, &best_end), RIGHT);
        let mut closing = ends.clone();
        closing.extend([bracket, gap]);
        fade_in_all(stage, &closing, 0.5)?;
        kit.music_breath(stage, HOLD_S)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A line of type is about this high in scene units a point (the box the
    /// lint draws round it), and the lines of one text stand this far apart.
    const TEXT_HEIGHT_PER_POINT: f64 = 0.0125;
    const LINE_PITCH_PER_POINT: f64 = 0.0135;

    /// (left, bottom, right, top).
    type Box4 = (f64, f64, f64, f64);

    fn boxes_overlap(a: Box4, b: Box4) -> bool {
        a.0 < b.2 && b.0 < a.2 && a.1 < b.3 && b.1 < a.3
    }

    /// `curve`'s height at scene `x` (its points are in order left to right).
    fn height_at(curve: &[Vec3], x: f64) -> f64 {
        let k = curve
            .windows(2)
            .position(|p| p[0][0] <= x && x <= p[1][0])
            .unwrap_or_else(|| panic!("x {x} is off the curve"));
        let (a, b) = (curve[k], curve[k + 1]);
        a[1] + (b[1] - a[1]) * (x - a[0]) / (b[0] - a[0])
    }

    /// The chapter speaks exactly the script's 08_ lines, in order.
    #[test]
    fn the_chapter_speaks_the_scripts_lines_in_order() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let ids: Vec<&str> = script
            .lines()
            .iter()
            .map(|(id, _)| id.as_str())
            .filter(|id| id.starts_with("08_"))
            .collect();
        assert_eq!(ids, LINES);
    }

    /// 08_1: the eight searches are the facts' order and the stack's top
    /// is the facts' curve (20, 33, 43, 51, 58, 64, 70, 74).
    #[test]
    fn the_stack_traces_the_rules_curve() {
        let f = facts().unwrap();
        let steps = the_rule(shown::BRUTE);
        let order: String = steps.iter().map(|s| sea::CELLS[s.cell]).collect();
        assert!(f.get("F-U.best.order").unwrap().starts_with(&order), "{order}");
        let tops: Vec<String> = steps.iter().map(|s| s.cumulative.fixed(0)).collect();
        let want: Vec<String> = f
            .get("F-U.best.found_by.percent")
            .unwrap()
            .split(',')
            .take(shown::BRUTE)
            .map(|p| format!("{:.0}", p.trim().parse::<f64>().unwrap()))
            .collect();
        assert_eq!(tops, want);
        assert_eq!(tops.join(", "), "20, 33, 43, 51, 58, 64, 70, 74");
        // every bar sits on the one before, and is the cell's next slice
        let mut before = Q::ZERO;
        for s in &steps {
            assert_eq!(s.cumulative - before, s.slice);
            before = s.cumulative;
        }
    }

    /// 08_2: each search puts out its cell's next slice, whatever was
    /// searched in between, and a cell's slices only shrink; what is still
    /// lit and what is out add up to the first hundred.
    #[test]
    fn a_search_puts_out_the_cells_next_slice_and_slices_shrink() {
        let steps = the_rule(shown::BRUTE);
        for s in &steps {
            assert_eq!(
                s.slice,
                slice(s.cell, s.nth),
                "cell {} search {}",
                sea::CELLS[s.cell],
                s.nth + 1
            );
        }
        for c in 0..4 {
            let slices = sea::slices(c, 6);
            assert!(slices.windows(2).all(|w| w[0] > w[1]));
        }
        let last = steps.last().unwrap();
        let lit = last.lit.iter().fold(Q::ZERO, |a, &b| a + b);
        assert_eq!(lit + last.cumulative, Q::int(sea::DOTS));
    }

    /// 08_2 and 08_3: the columns hold the rule's slices and the next slice
    /// of every cell; the eight tallest are exactly the rule's eight in its
    /// order, and the four left over are all smaller than the smallest taken.
    #[test]
    fn the_eight_tallest_slices_are_the_rules_in_its_order() {
        let steps = the_rule(shown::BRUTE);
        let sizes = column_sizes(&steps);
        assert_eq!(sizes, [4, 3, 3, 2], "A took 3, B 2, C 2, D 1, and each shows one more");
        let mut all: Vec<Q> = Vec::new();
        for (c, &n) in sizes.iter().enumerate() {
            for j in 0..n {
                all.push(slice(c, j));
            }
        }
        assert_eq!(all.len(), 12);
        all.sort_by(|a, b| b.cmp(a));
        let rules: Vec<Q> = steps.iter().map(|s| s.slice).collect();
        assert_eq!(all[..shown::BRUTE], rules[..], "tallest first is the rule's order");
        let smallest_taken = *rules.last().unwrap();
        assert!(all[shown::BRUTE..].iter().all(|&x| x < smallest_taken));
        // the ninth biggest is A's fourth slice, 4.2, just under C's 4.4
        assert_eq!(all[shown::BRUTE].fixed(1), "4.2");
        assert_eq!(smallest_taken.fixed(1), "4.4");
    }

    /// The labels print the slices the way the storyboard writes them, and
    /// the facts' per-cell lists are the same numbers.
    #[test]
    fn the_labels_print_the_slices_as_the_storyboard_does() {
        let f = facts().unwrap();
        for (c, name) in sea::CELLS.iter().enumerate() {
            let want = f.get(&format!("F-U.slices.{name}")).unwrap();
            let got: Vec<String> = (0..4).map(|j| slice(c, j).fixed(1)).collect();
            assert_eq!(got.join(","), want);
        }
        // every bar's number is one decimal, thirds included (R2: M3: the
        // scene had printed 6 2/3 and 13 1/3 as mixed numbers), and the
        // note under the names says that they are rounded
        let printed: Vec<String> = (0..4).map(|j| slice(0, j).fixed(1)).collect();
        assert_eq!(printed, ["10.0", "7.5", "5.6", "4.2"]);
        assert_eq!(slice(1, 0).fixed(1), "20.0");
        assert_eq!(slice(1, 1).fixed(1), "6.7");
        assert_eq!(slice(2, 0).fixed(1), "13.3");
        assert_eq!(slice(3, 0).fixed(1), "6.7");
        assert!(ROUNDED_NOTE.contains("rounded to one decimal"));
        // the tags carry the same two numbers, the whole dots of the first
        // search and the decimal of the second
        assert_eq!(first_tag_text(), "this search put out 20 of the first 100");
        assert_eq!(second_tag_text(), "this one 6.7");
        assert_eq!(f.get("F-U.slices.B").unwrap().split(',').nth(1), Some("6.7"));
    }

    /// 08_2 (R2: M3): the two tags stand over B's bars, each clear of the
    /// bars and numbers beside it and of the other tag, and every label and
    /// the note under the names stay inside the stage.
    #[test]
    fn the_slice_tags_stand_over_bs_bars_clear_of_their_neighbours() {
        let sizes = column_sizes(&the_rule(shown::BRUTE));
        let units = super::super::draw::CHAR_UNITS_PER_POINT;
        let high = |pt: f64| pt * TEXT_HEIGHT_PER_POINT;
        // (left, bottom, right, top) of a text of `chars` characters at `pt`
        let text_box = |centre_x: f64, y: f64, chars: usize, pt: f64, left_anchored: bool| {
            let w = chars as f64 * pt * units;
            let left = if left_anchored { centre_x } else { centre_x - w / 2.0 };
            (left, y - high(pt) / 2.0, left + w, y + high(pt) / 2.0)
        };
        let first_at = first_tag_at(&sizes);
        let second_at = second_tag_at(&sizes);
        let first = text_box(
            first_at[0],
            first_at[1],
            first_tag_text().chars().count(),
            TAG_PT,
            false,
        );
        let second = text_box(
            second_at[0],
            second_at[1],
            second_tag_text().chars().count(),
            TAG_PT,
            true,
        );
        let overlaps = boxes_overlap;
        assert!(!overlaps(first, second), "{first:?} {second:?}");
        // every bar with its outline's pad, and every number over a bar
        let mut obstacles: Vec<(String, (f64, f64, f64, f64))> = Vec::new();
        for c in 0..4 {
            for j in 0..sizes[c] {
                let x = column_x(&sizes, c, j);
                let h = slice(c, j).f64() * COL_PER_DOT;
                obstacles.push((
                    format!("bar {} {j}", sea::CELLS[c]),
                    (
                        x - COL_BAR_W / 2.0 - OUTLINE_PAD,
                        COL_BASE - OUTLINE_PAD,
                        x + COL_BAR_W / 2.0 + OUTLINE_PAD,
                        COL_BASE + h + OUTLINE_PAD,
                    ),
                ));
                let digits = slice(c, j).fixed(1).chars().count();
                obstacles.push((
                    format!("number {} {j}", sea::CELLS[c]),
                    text_box(x, COL_BASE + h + 0.26, digits, SLICE_PT, false),
                ));
            }
        }
        for (name, b) in &obstacles {
            // a tag stands over its own bar's outline and number, not on them
            let own_first = name.ends_with("B 0");
            let own_second = name.ends_with("B 1");
            if !own_first {
                assert!(!overlaps(first, *b), "first tag on {name}: {first:?} {b:?}");
            }
            if !own_second {
                assert!(!overlaps(second, *b), "second tag on {name}: {second:?} {b:?}");
            }
        }
        // the first tag is clear of B's own first bar and number as well
        let first_bar_top = COL_BASE + slice(1, 0).f64() * COL_PER_DOT + OUTLINE_PAD;
        assert!(first.1 > first_bar_top + 0.26, "{first:?} {first_bar_top}");
        let second_bar_top = COL_BASE + slice(1, 1).f64() * COL_PER_DOT + OUTLINE_PAD;
        assert!(second.1 > second_bar_top + 0.26, "{second:?} {second_bar_top}");
        // inside the stage, with the lint's margin
        for (name, b) in [("first", first), ("second", second)] {
            assert!(
                b.0 > -FRAME_X_RADIUS + 0.15 && b.2 < FRAME_X_RADIUS - 0.15,
                "{name} {b:?}"
            );
            assert!(b.3 < FRAME_Y_RADIUS - 0.15, "{name} {b:?}");
        }
        // the note that the numbers are rounded: centred, inside the stage,
        // under the names (28 pt at COL_BASE - 0.4) and over the caption band
        let note = text_box(0.0, ROUNDED_Y, ROUNDED_NOTE.chars().count(), 22.0, false);
        assert!(
            note.0 > -FRAME_X_RADIUS + 0.15 && note.2 < FRAME_X_RADIUS - 0.15,
            "{note:?}"
        );
        assert!(note.3 < COL_BASE - 0.4 - high(28.0) / 2.0 - 0.1, "{note:?}");
        assert!(note.1 > QUIZ_LINES_Y[0] + 0.3, "{note:?}");
    }

    /// The twelve bars of 08_2 stand clear of each other and inside the stage.
    #[test]
    fn the_columns_fit_the_stage() {
        let sizes = column_sizes(&the_rule(shown::BRUTE));
        let mut xs: Vec<f64> = Vec::new();
        for c in 0..4 {
            for j in 0..sizes[c] {
                xs.push(column_x(&sizes, c, j));
            }
        }
        assert!(xs.windows(2).all(|p| p[1] - p[0] >= COL_PITCH - 1e-12), "{xs:?}");
        let (first, last) = (xs[0] - COL_BAR_W / 2.0, xs[xs.len() - 1] + COL_BAR_W / 2.0);
        assert!(
            first > -FRAME_X_RADIUS + 0.3 && last < FRAME_X_RADIUS - 0.3,
            "{first} {last}"
        );
        // the tallest bar (B's first slice, 20 dots) and its label stay
        // under the top of the stage
        assert!(COL_BASE + slice(1, 0).f64() * COL_PER_DOT + 0.5 < FRAME_Y_RADIUS);
    }

    /// 08_3: the small line is every order of eight searches of four cells.
    #[test]
    fn every_order_is_four_to_the_eighth() {
        let f = facts().unwrap();
        let total: u64 = f.get("F-U.brute.orders").unwrap().parse().unwrap();
        assert_eq!(total, (sea::CELLS.len() as u64).pow(shown::BRUTE as u32));
        assert_eq!(
            orders_line(sea::CELLS.len(), shown::BRUTE, total),
            "4 × 4 × 4 × 4 × 4 × 4 × 4 × 4 = 65,536"
        );
        assert_eq!(f.get("F-U.brute.rule_is_best"), Some("true"));
        assert_eq!(f.get("F-U.slices.rule_takes_biggest"), Some("true"));
    }

    /// The narrated draft's overlaps in 08_3 and 08_5: the code panel's
    /// pieces (code, then comment) are set at a size the panel's width holds
    /// and it stands in the left half, clear of the stack's window; the
    /// legend's rows stand clear of the model tag; the two nearest curves'
    /// end labels are further apart than a line of text is high.
    #[test]
    fn the_code_panel_the_legend_and_the_end_labels_clear_what_they_stood_on() {
        let pieces = code_over_comments(&glyphs::CODE[2..]);
        assert_eq!(pieces.len(), 4, "two lines, each over its comment");
        assert!(pieces[0].starts_with("find = ") && pieces[1].starts_with("# chance of a find"));
        assert!(pieces[2].starts_with("i = find.index") && pieces[3].starts_with("# search there next"));
        for (piece, line) in pieces.chunks(2).zip(&glyphs::CODE[2..]) {
            // nothing of the line is lost or added: the same characters
            assert_eq!(
                format!("{} {}", piece[0], piece[1]).replace(' ', ""),
                line.replace(' ', "")
            );
            assert!(piece[1].starts_with('#') && !piece[0].contains('#'));
        }
        let widest = pieces.iter().map(|p| p.chars().count()).max().unwrap() as f64;
        // a typewriter column is about 0.005 of a scene unit a point
        assert!(
            widest * CODE_PT * 0.0052 <= CODE_W - 0.8,
            "{widest} columns at {CODE_PT} pt"
        );
        let (left, right) = (-FRAME_X_RADIUS + 0.35, -FRAME_X_RADIUS + 0.35 + CODE_W);
        assert!(
            right < stack_window().left() - 0.3,
            "{right} against {}",
            stack_window().left()
        );
        assert!(left > -FRAME_X_RADIUS);
        const {
            // the panel (a unit high either way of its middle) is on the stage
            assert!(CODE_Y.abs() + 1.0 < FRAME_Y_RADIUS);
            // the third legend row against the model tag's height (-3.0)
            assert!(LEGEND_Y - 2.0 * LEGEND_PITCH - (-3.0) >= 0.4);
            // the x label stands under the ticks' row, the count of orders
            // a row under that
            assert!(X_LABEL_DROP >= 0.26 + 0.3);
            assert!(BRUTE_DROP >= X_LABEL_DROP + 0.3);
        }
        // the 08_5 chart: 49.8% and 44.3% end 5.5 points apart on a chart
        // three units high; the labels move 0.12 each way
        let f = facts().unwrap();
        let end = |k: &str| f.get(k).unwrap().parse::<f64>().unwrap();
        let apart = (end("F-R.model.best.found_by.20") - end("F-R.model.once.found_by.20")) * 3.0 / 100.0;
        assert!(apart + 2.0 * END_LABEL_SPREAD >= 0.35, "{apart}");
    }

    /// 08_4 and 08_5: the Navy's map is searched at E5, B7, C5 and G5, and
    /// the three curves end at the facts' 0.8, 44.3 and 49.8 after twenty,
    /// never behind each other in that order.
    #[test]
    fn the_model_search_and_its_three_curves_are_the_facts() {
        let f = facts().unwrap();
        let looks = y1968::model_search(y1968::Rule::Best, shown::MAP_SEEING, 4);
        let cells: Vec<String> = looks.iter().map(|l| y1968::name(l.row, l.col)).collect();
        assert_eq!(cells.join(", "), "E5, B7, C5, G5");
        assert!(
            f.get("F-R.model.best.first_cells")
                .unwrap()
                .starts_with(&cells.join(","))
        );
        let n = shown::MAP_MARKS[0];
        for (rule, key) in [
            (y1968::Rule::Sweep, "F-R.model.sweep.found_by.20"),
            (y1968::Rule::Once, "F-R.model.once.found_by.20"),
            (y1968::Rule::Best, "F-R.model.best.found_by.20"),
        ] {
            let curve = curve_percent(rule, n);
            assert_eq!(curve.len(), n);
            assert!(curve.windows(2).all(|w| w[0] <= w[1]));
            assert_eq!(format!("{:.1}", curve[n - 1]), f.get(key).unwrap(), "{key}");
        }
        let (best, once, sweep) = (
            curve_percent(y1968::Rule::Best, n),
            curve_percent(y1968::Rule::Once, n),
            curve_percent(y1968::Rule::Sweep, n),
        );
        for k in 0..n {
            assert!(best[k] >= once[k] && once[k] >= sweep[k], "search {}", k + 1);
        }
    }

    /// 08_3 (R2: D4, R8, S8): Chew's line is his theorem's words and fits
    /// one line of the lower third; the fresh-try gloss stands in the left
    /// half over the code panel's place and clear of the stack; the "ahead"
    /// label stands under the staircase the bars make, inside the window and
    /// clear of every bar and every piece of the trace.
    #[test]
    fn chews_line_the_gloss_and_the_ahead_label_say_what_they_should_and_stand_clear() {
        let units = super::super::draw::CHAR_UNITS_PER_POINT;
        assert!(CHEW_TEXT.contains("not having found it yet"));
        assert!(!CHEW_TEXT.contains("still searching"));
        // the label and its words on one band line: the band's width less
        // its two insets, the label and the gap after it (lower_third)
        let label = format!("{CHEW_LABEL} {}:", facts().unwrap().get("F-N.chew").unwrap());
        assert_eq!(label, "M. C. Chew, 1967:");
        let room = (2.0 * FRAME_X_RADIUS - 0.8) - 0.7 - label.chars().count() as f64 * 26.0 * units;
        let words = CHEW_TEXT.chars().count() as f64 * 26.0 * units;
        assert!(
            CHEW_TEXT.chars().count() <= 92 && words <= room,
            "{words} against {room}"
        );

        // the gloss: three lines, in the left half, over the panel and clear of the stack
        let lines: Vec<&str> = FRESH_TRY.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(FRESH_TRY.contains("a miss now says nothing about the next try"));
        assert!(FRESH_TRY.contains("Moving between cells costs nothing here"));
        let widest = lines.iter().map(|l| l.chars().count()).max().unwrap() as f64 * FRESH_PT * units;
        let high = lines.len() as f64 * FRESH_PT * LINE_PITCH_PER_POINT;
        let gloss = (
            FRESH_AT[0] - widest / 2.0,
            FRESH_AT[1] - high / 2.0,
            FRESH_AT[0] + widest / 2.0,
            FRESH_AT[1] + high / 2.0,
        );
        assert!(gloss.0 > -FRAME_X_RADIUS + 0.15, "{gloss:?}");
        assert!(gloss.2 < stack_window().left() - 0.3, "{gloss:?}");
        assert!(gloss.1 > CODE_Y + 1.0 + 0.15, "over the panel: {gloss:?}");
        assert!(gloss.3 < FRAME_Y_RADIUS - 0.15, "{gloss:?}");

        // the "ahead" label under the staircase
        let w = stack_window();
        let at = w.p(AHEAD_AT[0], AHEAD_AT[1]);
        let lines: Vec<&str> = AHEAD.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(
            AHEAD
                .replace('\n', " ")
                .starts_with("ahead: more of the first hundred dots out, after the same")
        );
        let widest = lines.iter().map(|l| l.chars().count()).max().unwrap() as f64 * AHEAD_PT * units;
        let high = lines.len() as f64 * AHEAD_PT * LINE_PITCH_PER_POINT;
        let ahead = (
            at[0] - widest / 2.0,
            at[1] - high / 2.0,
            at[0] + widest / 2.0,
            at[1] + high / 2.0,
        );
        assert!(ahead.0 > w.left() + 0.05 && ahead.2 < w.right() - 0.05, "{ahead:?}");
        assert!(ahead.1 > w.bottom() + 0.05 && ahead.3 < w.top(), "{ahead:?}");
        let mut before = 0.0;
        for (k, s) in the_rule(shown::BRUTE).iter().enumerate() {
            let after = s.cumulative.f64();
            let (lo, hi) = (
                w.p((k + 1) as f64 - BAR_W / 2.0, before),
                w.p((k + 1) as f64 + BAR_W / 2.0, after),
            );
            assert!(
                !boxes_overlap(ahead, (lo[0], lo[1], hi[0], hi[1])),
                "ahead on bar {}: {ahead:?}",
                k + 1
            );
            // the trace from this search's bar to the next
            let (a, b) = (w.p(k as f64 + 0.5, before), w.p(k as f64 + 1.5, after));
            for t in 0..=24 {
                let u = f64::from(t) / 24.0;
                let (x, y) = (a[0] + (b[0] - a[0]) * u, a[1] + (b[1] - a[1]) * u);
                assert!(
                    !(ahead.0 - 0.03 < x && x < ahead.2 + 0.03 && ahead.1 - 0.03 < y && y < ahead.3 + 0.03),
                    "ahead on the trace after search {}",
                    k + 1
                );
            }
            before = after;
        }
    }

    /// 08_4 (R2: D1, R6, S7, S15): the row-by-row tag holds the fact's 114
    /// of the 10,000, which is the first twenty cells of the sweep the strip
    /// follows from the north edge, and it stands under that grid, inside the
    /// grid's width, over the model line.
    #[test]
    fn the_row_note_holds_the_facts_and_stands_under_its_grid() {
        let units = super::super::draw::CHAR_UNITS_PER_POINT;
        let f = facts().unwrap();
        let first20: u32 = f.get("F-R.model.sweep.first20.points").unwrap().parse().unwrap();
        assert_eq!(first20, 114);
        assert_eq!(y1968::total(), 10_000);
        let text = row_note_text(first20, y1968::total());
        assert_eq!(
            text,
            "from the north edge, a creeping line\nfrom one corner: its first twenty\ncells hold 114 of the 10,000 dots"
        );
        // the 114 is the sum of the twenty cells the sweep takes first
        let looks = y1968::model_search(y1968::Rule::Sweep, shown::MAP_SEEING, 20);
        let sum: u32 = looks.iter().map(|l| u32::from(y1968::POINTS[l.row][l.col])).sum();
        assert_eq!(sum, first20);
        // and those are row by row: a whole row west to east, then the next
        assert!(looks[..15].iter().all(|l| l.row == looks[0].row));
        assert!(looks[..15].windows(2).all(|p| p[1].col == p[0].col + 1));
        assert_eq!(looks[15].row, looks[0].row + 1);

        let cell_w = GRID_CELL_H * y1968::CELL_MILES.1.f64() / y1968::CELL_MILES.0.f64();
        let gw = y1968::GRID_COLS.len() as f64 * cell_w;
        let gh = y1968::GRID_ROWS.len() as f64 * GRID_CELL_H;
        let lines: Vec<&str> = text.lines().collect();
        let widest = lines.iter().map(|l| l.chars().count()).max().unwrap() as f64 * ROW_NOTE_PT * units;
        let high = lines.len() as f64 * ROW_NOTE_PT * LINE_PITCH_PER_POINT;
        let middle = GRID_AT[1] - gh / 2.0 - ROW_NOTE_DROP;
        assert!(widest <= gw + 0.1, "{widest} against the grid's {gw}");
        assert!(GRID_AT[0] - widest / 2.0 > -FRAME_X_RADIUS + 0.15);
        assert!(middle + high / 2.0 < GRID_AT[1] - gh / 2.0 - 0.05, "under the grid");
        let model_top = MODEL_Y + 22.0 * TEXT_HEIGHT_PER_POINT / 2.0;
        assert!(middle - high / 2.0 > model_top + 0.15, "over the model line");
    }

    /// 08_5 (R2: D1, D2, R6, R11, S7): the axis note's numbers are the
    /// facts', it stands over the chart's own label clear of both maps'
    /// labels; the no-map mark is the fact's 6.6 as a dashed line, its words
    /// over it clear of every curve; the legend keeps its three rows, clear
    /// of the model line.
    #[test]
    fn the_axis_note_and_the_no_map_mark_say_the_facts_and_stand_clear() {
        let units = super::super::draw::CHAR_UNITS_PER_POINT;
        let f = facts().unwrap();
        let chart = chart_20();
        let n = shown::MAP_MARKS[0];

        let outside: u32 = f.get("F-R.outside.points").unwrap().parse().unwrap();
        assert_eq!(outside, y1968::total() - y1968::in_grid());
        let cells = y1968::GRID_ROWS.len() * y1968::GRID_COLS.len();
        assert_eq!(cells, 180);
        let note = axis_note_text(outside, y1968::total(), cells);
        assert_eq!(
            note,
            "percent of all 10,000 runs: 1,100 ended outside these 180 cells\nand are never searched here"
        );
        let lines: Vec<&str> = note.lines().collect();
        let widest = lines.iter().map(|l| l.chars().count()).max().unwrap() as f64 * AXIS_NOTE_PT * units;
        let high = lines.len() as f64 * AXIS_NOTE_PT * LINE_PITCH_PER_POINT;
        let (bottom, top) = (
            chart.top() + AXIS_NOTE_RISE - high / 2.0,
            chart.top() + AXIS_NOTE_RISE + high / 2.0,
        );
        assert!(chart.left() + widest < FRAME_X_RADIUS - 0.15);
        assert!(top < FRAME_Y_RADIUS - 0.15, "{top}");
        assert!(
            bottom > chart.top() + 0.28 + 22.0 * TEXT_HEIGHT_PER_POINT / 2.0 + 0.04,
            "over the chart's own label: {bottom}"
        );
        // clear of the row-by-row map's label, which stands over that map
        let by_row_right = GRID_AT[0] + BY_ROW.chars().count() as f64 * 24.0 * units / 2.0;
        assert!(chart.left() > by_row_right + 0.2);

        // the mark: a dashed line at the fact's level, the words over it
        let level: f64 = f
            .get(&format!("F-R.model.random.found_by.{n}"))
            .unwrap()
            .parse()
            .unwrap();
        let inside = f64::from(y1968::in_grid()) / f64::from(y1968::total());
        let by_hand = 100.0 * n as f64 / cells as f64 * shown::MAP_SEEING.f64() * inside;
        assert_eq!(format!("{by_hand:.1}"), format!("{level:.1}"));
        assert_eq!(level, 6.6);
        let words = format!("{RANDOM_ORDER} {level}%");
        assert_eq!(words, "no map, any order, on average: 6.6%");
        let at = chart.p(MARK_FROM, level);
        let mark = (
            at[0],
            at[1] + MARK_RISE - MARK_PT * TEXT_HEIGHT_PER_POINT / 2.0,
            at[0] + words.chars().count() as f64 * MARK_PT * units,
            at[1] + MARK_RISE + MARK_PT * TEXT_HEIGHT_PER_POINT / 2.0,
        );
        assert!(mark.2 <= chart.right() - 0.05, "{mark:?}");
        assert!(mark.1 > at[1] + 0.04, "over the dashed line: {mark:?}");
        for rule in [y1968::Rule::Sweep, y1968::Rule::Once, y1968::Rule::Best] {
            let pts = curve_points(&chart, rule, n);
            for p in pts.windows(2) {
                for t in 0..=40 {
                    let u = f64::from(t) / 40.0;
                    let (x, y) = (p[0][0] + (p[1][0] - p[0][0]) * u, p[0][1] + (p[1][1] - p[0][1]) * u);
                    assert!(
                        !(mark.0 - 0.03 < x && x < mark.2 + 0.03 && mark.1 - 0.03 < y && y < mark.3 + 0.03),
                        "the no-map words are on the curve of {rule:?} near ({x:.2}, {y:.2})"
                    );
                }
            }
        }
        // the dashes: twenty, each as long as the gap after it, end to end
        let spans = dash_spans(MARK_DASHES);
        assert_eq!(spans.len(), MARK_DASHES);
        assert_eq!(spans[0].0, 0.0);
        assert!((spans[MARK_DASHES - 1].1 - 1.0).abs() < 1e-12);
        let len = spans[0].1 - spans[0].0;
        for (i, s) in spans.iter().enumerate() {
            assert!((s.1 - s.0 - len).abs() < 1e-12, "dash {i}");
            if let Some(next) = spans.get(i + 1) {
                assert!((next.0 - s.1 - len).abs() < 1e-12, "gap {i}");
            }
        }

        // the legend: three rows (no row for the mark: its words are on the
        // chart), the first tagged where the sweep starts, every row inside
        // the stage, under the maps and the chart's x label, over the model line
        let plans = plans();
        assert_eq!(plans.len(), 3);
        assert!(plans[0].name.ends_with(FROM_THE_NORTH));
        for (i, p) in plans.iter().enumerate() {
            let y = LEGEND_Y - LEGEND_PITCH * i as f64;
            let right = -2.75 + p.name.chars().count() as f64 * 22.0 * units;
            assert!(right < FRAME_X_RADIUS - 0.15, "row {i} runs to {right}");
            assert!(y - MODEL_Y >= 0.4, "row {i} against the model line");
        }
        let lattice_bottom = LATTICE_AT[1] - y1968::GRID_ROWS.len() as f64 * LATTICE_CELL_H / 2.0;
        assert!(LEGEND_Y + 22.0 * TEXT_HEIGHT_PER_POINT / 2.0 < lattice_bottom - 0.1);
        let x_label = chart.bottom() - X_LABEL_DROP;
        assert!(LEGEND_Y + 22.0 * TEXT_HEIGHT_PER_POINT / 2.0 < x_label - 22.0 * TEXT_HEIGHT_PER_POINT / 2.0 - 0.03);
    }

    /// 08_5's hold (R2: M17): the three curves run on to eighty-one searches
    /// and end at the facts' 32.9, 57.8 and 73.2; the top two ends are far
    /// enough apart for their numbers, and "what updating adds" stands in the
    /// gap between those curves, left of the bracket that spans it.
    #[test]
    fn the_hold_runs_the_curves_on_to_eighty_one_and_labels_the_gap() {
        let units = super::super::draw::CHAR_UNITS_PER_POINT;
        let f = facts().unwrap();
        let long = chart_81();
        let n = shown::MAP_MARKS[1];
        assert_eq!(n, 81);
        assert_eq!(n % 3, 0, "ticks at thirds");
        assert_eq!(long.center, chart_20().center);
        assert_eq!((long.width, long.height), (chart_20().width, chart_20().height));
        let want = [("sweep", "32.9"), ("once", "57.8"), ("best", "73.2")];
        for (plan, (slug, value)) in plans().iter().zip(want) {
            assert_eq!(plan.slug, slug);
            assert_eq!(f.get(&plan.key(n)), Some(value));
            let curve = curve_percent(plan.rule, n);
            assert_eq!(curve.len(), n);
            assert_eq!(format!("{:.1}", curve[n - 1]), value, "{slug}");
        }
        let once = curve_points(&long, y1968::Rule::Once, n);
        let best = curve_points(&long, y1968::Rule::Best, n);
        let (once_end, best_end) = (*once.last().unwrap(), *best.last().unwrap());
        assert!(
            best_end[1] - once_end[1] >= 0.36,
            "the two end numbers stand {} apart",
            best_end[1] - once_end[1]
        );
        let right = gap_label_right(&once_end, &best_end);
        let width = WHAT_UPDATING_ADDS.chars().count() as f64 * GAP_PT * units;
        let (left, half) = (right[0] - width, GAP_PT * TEXT_HEIGHT_PER_POINT / 2.0);
        assert!(left > long.left() + 0.2, "{left}");
        // the words stand over the rule's curve (the gap between the two
        // curves is a wedge, too narrow under a label of this size), and the
        // bracket's line runs up to them from the other curve's end
        for s in 0..=20 {
            let x = left + width * f64::from(s) / 20.0;
            assert!(
                height_at(&best, x) + 0.03 < right[1] - half,
                "the gap label's foot meets the rule's curve at x {x:.2}"
            );
        }
        assert!(right[1] + half < long.top() - 0.05, "inside the chart's top");
        assert!(right[1] > best_end[1] + half, "over the rule's own end");
        // the bracket stands right of the words and inside the chart
        assert!(right[0] + GAP_LABEL_CLEAR + GAP_BRACKET_INSET <= long.right() + 1e-9);
        assert!(once_end[1] < best_end[1], "the bracket spans the gap upward");
        assert_eq!(WHAT_UPDATING_ADDS, "what updating adds");
    }
}
