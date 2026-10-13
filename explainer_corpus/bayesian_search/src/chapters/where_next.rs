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
const CHEW_TEXT: &str = "after any number of searches, the least chance of still searching.";
const MODEL: &str = "model:";
const CHANCE_BY_NOW: &str = "chance of a find by now, according to the map";

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

/// A slice of the first hundred dots as TeX: a whole number or a decimal
/// that ends as it is, a third as a mixed number, anything else to one place.
fn slice_tex(q: Q) -> String {
    let s = q.to_string();
    match s.split_once('/') {
        None => s,
        Some((n, d)) => {
            let (n, d): (i128, i128) = (n.parse().unwrap_or(0), d.parse().unwrap_or(1));
            if d == 3 {
                format!("{}\\frac{{{}}}{{3}}", n / 3, n % 3)
            } else {
                q.fixed(1)
            }
        }
    }
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
    out.push(kit.text(
        stage,
        X_LABEL,
        22.0,
        Palette::dim(),
        v(f.right() - 0.5, f.bottom() - 0.26),
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
        let mut ins = cells.clone();
        ins.extend(&sums);
        fade_in_all(stage, &ins, 0.7)?;
        fade_in_all(stage, &axes, 0.5)?;
        for &m in &live {
            stage.add_to_scene(m)?;
        }
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
        fade_in_all(stage, &[choice], 0.3)?;
        let mut bars: Vec<Mob> = Vec::new();
        let mut tops: Vec<Mob> = Vec::new();
        for (k, s) in steps.iter().enumerate() {
            let r = the_sea.rect(s.cell);
            let at = v((r.0 + r.2) / 2.0, (r.1 + r.3) / 2.0);
            let here = stage.get_center(choice);
            let jump = choice
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.2),
                    ..AnimateArgs::default()
                })?
                .shift([at[0] - here[0], at[1] - here[1], 0.0])?;
            stage.play(jump)?;
            let strip = Strip::build(stage, r, 0.22 * (r.2 - r.0))?;
            for m in strip.mobs() {
                stage.add_to_scene(m)?;
            }
            stage.play(tween_linear(strip.phase, 1.0, 0.35)?)?;
            // the cell's dots go out: every lit count moves (the heat and
            // both readouts follow), and the slice drops onto the stack
            let b = stacked(stage, &w, (k + 1) as f64, before_of(k).f64(), s.cumulative.f64(), out())?;
            bars.push(b);
            let top = trace(stage, &w, k, before_of(k).f64(), s.cumulative.f64());
            tops.push(top);
            let mut together: Vec<Box<dyn Animation>> = Vec::new();
            for (c, &t) in the_sea.lit.iter().enumerate() {
                together.push(stage.prepare(tween(t, s.lit[c].f64(), 0.4)?)?);
            }
            let drop = fade_in(stage.arena_mut(), b, v(0.0, 0.5), 1.0)?;
            together.push(stage.prepare(drop.rt(0.4))?);
            let tracing = fade_in(stage.arena_mut(), top, ORIGIN, 1.0)?;
            together.push(stage.prepare(tracing.rt(0.4))?);
            for m in [strip.wash, strip.bar] {
                let gone = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
                together.push(stage.prepare(gone.rt(0.3))?);
            }
            stage.play_prepared(together)?;
        }
        // the question to carry: the line's own last sentence, two seconds
        let words = line
            .sentence_texts()
            .pop()
            .ok_or_else(|| fail("08_1 has no sentences".into()))?;
        let question = kit.text(stage, &words, 26.0, Palette::ink(), v(0.0, QUIZ_LINES_Y[0]))?;
        fade_in_all(stage, &[question], 0.4)?;
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
            for j in 0..sizes[c] {
                let top = COL_BASE + slice(c, j).f64() * COL_PER_DOT;
                names.push(kit.tex(
                    stage,
                    &slice_tex(slice(c, j)),
                    26.0,
                    Palette::ink(),
                    &[],
                    v(column_x(&sizes, c, j), top + 0.26),
                )?);
            }
        }
        let mut shown_now = spares.clone();
        shown_now.extend(&names);
        fade_in_all(stage, &shown_now, 0.6)?;
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
        line.wait_for(stage, "no other order")?;
        let total: u64 = fact("F-U.brute.orders")?
            .parse()
            .map_err(|_| fail("F-U.brute.orders is not a number".into()))?;
        let brute = kit.text(
            stage,
            &orders_line(sea::CELLS.len(), shown::BRUTE, total),
            22.0,
            Palette::dim(),
            v(w.center[0], w.bottom() - 0.72),
        )?;
        fade_in_all(stage, &[brute], 0.5)?;
        line.wait_end(stage)?;
        // in the hold: the theorem, and the last two lines of the code panel
        let chew = lower_third(&kit, stage, &format!("{CHEW_LABEL} {}:", fact("F-N.chew")?), CHEW_TEXT)?;
        let panel = CodePanel::build(&kit, stage, &glyphs::CODE[2..], 17.0, 7.4)?;
        stage.shift(panel.group, [0.0, -2.15, 0.0]);
        fade_in_all(stage, &[chew], 0.5)?;
        stage.add_to_scene(panel.group)?;
        stage.play(panel.slide_in(0.8)?)?;
        kit.music_breath(stage, HOLD_S)?;

        // ------------------------------------------------------- 08_4
        // on the Navy's map: no map, row by row; and the rule over the
        // printed map, searching E5, B7, C5 and G5 in turn
        let line = say(&kit, stage, LINES[3])?;
        let mut off: Vec<Mob> = vec![chew, panel.group, brute];
        off.extend(&bars);
        off.extend(&again);
        off.extend(&axes);
        let mut seen = HashSet::new();
        off.retain(|m| seen.insert(*m));
        fade_out_all(stage, &off, 0.7)?;

        let (left_at, right_at) = (v(-4.9, 0.55), v(4.9, 0.55));
        let grid = NavyGrid::build(stage, left_at, 0.26)?;
        let looks = y1968::model_search(y1968::Rule::Best, shown::MAP_SEEING, 4);
        let probes: Vec<Probe> = looks
            .iter()
            .map(|l| Probe {
                row: l.row,
                col: l.col,
                seeing: shown::MAP_SEEING,
            })
            .collect();
        let lattice = Lattice::build(stage, right_at, 0.2, &probes, Heat::LATTICE)?;
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
            v(0.0, -3.0),
        )?;
        fade_in_all(stage, &[by_row, by_rule, model], 0.5)?;
        // no map: the strip along the grid's first row, west to east
        line.wait_for(stage, "One plan has no map")?;
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
        let n = shown::MAP_MARKS[0];
        let chart = Frame2 {
            center: v(0.0, 0.65),
            width: 4.4,
            height: 3.0,
            x: (0.0, n as f64),
            y: (0.0, 100.0),
        };
        let chart_mobs = furniture(&kit, stage, &chart, &[0.0, n as f64 / 2.0, n as f64], CHANCE_BY_NOW)?;
        fade_in_all(stage, &chart_mobs, 0.5)?;
        let plans = [
            (
                y1968::Rule::Sweep,
                Palette::dim(),
                BY_ROW,
                "F-R.model.sweep.found_by.20",
            ),
            (
                y1968::Rule::Once,
                Palette::ink(),
                NEVER_UPDATED,
                "F-R.model.once.found_by.20",
            ),
            (y1968::Rule::Best, find(), BY_RULE, "F-R.model.best.found_by.20"),
        ];
        for (i, (rule, color, name, key)) in plans.iter().enumerate() {
            let pct = curve_percent(*rule, n);
            let mut pts = vec![chart.p(0.0, 0.0)];
            pts.extend(pct.iter().enumerate().map(|(k, &p)| chart.p((k + 1) as f64, p)));
            let end = *pts.last().ok_or_else(|| fail("no curve".into()))?;
            let curve = stage
                .arena_mut()
                .add(polyline(pts).style(line_style(*color, 4.0)).build());
            let value = kit.text_left(
                stage,
                &format!("{}%", fact(key)?),
                24.0,
                *color,
                v(end[0] + 0.15, end[1]),
            )?;
            let y = -2.2 - 0.32 * i as f64;
            let swatch = stage.arena_mut().add(
                polyline(vec![v(-3.3, y), v(-2.9, y)])
                    .style(line_style(*color, 5.0))
                    .build(),
            );
            let words = kit.text_left(stage, name, 22.0, Palette::ink(), v(-2.75, y))?;
            play!(stage; show_creation(curve).rt(1.4));
            fade_in_all(stage, &[value, swatch, words], 0.4)?;
        }
        line.wait_end(stage)?;
        kit.hold(stage)?;
        kit.music_breath(stage, HOLD_S)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let printed: Vec<String> = (0..4).map(|j| slice_tex(slice(0, j))).collect();
        assert_eq!(printed, ["10", "7.5", "5.6", "4.2"]);
        assert_eq!(slice_tex(slice(1, 1)), "6\\frac{2}{3}");
        assert_eq!(slice_tex(slice(2, 0)), "13\\frac{1}{3}");
        assert_eq!(slice_tex(slice(1, 0)), "20");
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
}
