//! Chapter 09, the second pass: why a search sees a wreck two times in
//! three (09_1); perfect strips laid side by side cover a cell once and
//! exactly (09_2); the same track laid at random overlaps and leaves gaps,
//! and the average of a thousand such runs is 63% (09_3); Koopman's formula
//! is that curve, and two in three is a little more track (09_4); then a
//! second pass is a fresh try (09_5, the quiz) that finds two in three of
//! what is left, so eight of nine, and a third pass leaves one in
//! twenty-seven (09_6).
//!
//! Every number on the screen comes from `model::koopman` and `model::sea`
//! (the one run, the thousand runs, the formula, the passes) or from
//! `facts`; the tests below pin each against the fact it stands for.

use std::f64::consts::E;
use std::rc::Rc;

use explainer_kit::animatic::HOLD_S;
use explainer_kit::props::lower_third;
use explainer_kit::stats::{Data, permutation};
use explainer_kit::{Frame2, Kit, Palette, Timed, fade_all, play, rect, v};
use fmn::prelude::*;

use super::draw::{
    QuizFrame, fade_in_all, fade_out_all, fail, flat, group, line_style, map as lit_blue, out as slate, say, scatter,
    search as teal, tween_linear,
};
use crate::facts::facts;
use crate::model::{Q, koopman, sea};

pub struct TheSecondPass {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts and the model)
const TAG: &str = "a flat cell of the made-up sea";
const SWEPT: &str = "of the cell swept";
const THIS_RUN: &str = "this run";
const AVERAGE: &str = "average of";
const RUNS_WORD: &str = "other runs";
const BAND: &str = "the whole range of those runs";
const X_LABEL: &str = "coverage: strip × track ÷ area";
const Y_LABEL: &str = "chance of seeing it";
const FLAT: &str = "a flat cell";
const CANYON: &str = "the canyon";
const W_L_A: &str = "W width · L length · A area";
const E_NOTE: &str = "e: a fixed number, about";
const UNSWEPT: &str = "% unswept";
const FORMULA_LABEL: &str = "the random search formula.";
const FORMULA_QUOTE: &str = "Koopman: “a proper estimate, on the conservative side”";
const KOOPMAN_LABEL: &str = "B. O. Koopman";
const KOOPMAN_BOOK: &str = "Search and Screening,";
const CAUGHT: &str = "caught:";
const STILL_LIT: &str = "still lit:";
const OF: &str = "of";
const OUT: &str = "out";
const PASS: &str = "pass";
const PASSES: &str = "passes";

// ------------------------------------------------------------------ geometry

/// The cell as a square on the stage: where it stands and how long its side
/// is. Fractions run across from the left and down from the top.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Cell {
    at: Vec3,
    side: f64,
}

impl Cell {
    fn p(&self, x: f64, y: f64) -> Vec3 {
        v(self.at[0] + (x - 0.5) * self.side, self.at[1] + (0.5 - y) * self.side)
    }

    fn top(&self) -> f64 {
        self.at[1] + self.side / 2.0
    }

    fn bottom(&self) -> f64 {
        self.at[1] - self.side / 2.0
    }
}

/// The cell as 09_1 and 09_2 show it: it fills the stage.
const BIG: Cell = Cell {
    at: [0.0, 0.5, 0.0],
    side: 5.2,
};
/// The cell once the plot has come in beside it.
const SMALL: Cell = Cell {
    at: [-3.8, 0.5, 0.0],
    side: 4.2,
};

/// The plot's window: coverage across, the chance of seeing up.
fn plot() -> Frame2 {
    Frame2 {
        center: v(3.45, 0.05),
        width: 6.3,
        height: 3.9,
        x: (0.0, COVERAGE_MAX),
        y: (0.0, 100.0),
    }
}

/// The right edge of the plot, in coverages: room for three passes at two in
/// three (3 ln 3 = 3.30).
const COVERAGE_MAX: f64 = 3.5;
/// Coverage 1 is this many lanes of the cell's hundred.
const LANES: usize = koopman::LANES_PER_COVERAGE;

/// One lane's rectangles as cell fractions `(x0, y0, x1, y1)`, y down from
/// the top. A lane that runs off the cell's edge wraps, so it is two.
fn lane_boxes(lane: koopman::Lane) -> Vec<(f64, f64, f64, f64)> {
    let side = koopman::SIDE as f64;
    let end = lane.at + koopman::WIDTH;
    let spans: Vec<(f64, f64)> = if end <= koopman::SIDE {
        vec![(lane.at as f64, end as f64)]
    } else {
        vec![(lane.at as f64, side), (0.0, (end - koopman::SIDE) as f64)]
    };
    spans
        .into_iter()
        .map(|(a, b)| {
            if lane.vertical {
                (a / side, 0.0, b / side, 1.0)
            } else {
                (0.0, a / side, 1.0, b / side)
            }
        })
        .collect()
}

/// The perfect strips of 09_2: strip k is the k-th hundredth of the cell,
/// laid side by side from the left.
fn perfect_boxes() -> Vec<(f64, f64, f64, f64)> {
    let n = LANES as f64;
    (0..LANES)
        .map(|k| (k as f64 / n, 0.0, (k as f64 + 1.0) / n, 1.0))
        .collect()
}

/// The share of the cell swept after each of the video's one run's lanes,
/// in percent, with a zero first: entry k is the readout after k lanes.
fn one_run_percent(lanes: usize) -> Vec<f64> {
    let points = (koopman::SIDE * koopman::SIDE) as f64;
    std::iter::once(0.0)
        .chain(
            koopman::swept_after(&koopman::the_run(lanes))
                .into_iter()
                .map(|s| 100.0 * s as f64 / points),
        )
        .collect()
}

/// The thousand other runs, as the facts' mean uses them: the least, the
/// mean and the most of the share swept after each of the first `lanes`
/// lanes, percent, with a zero first. The whole range, so that the video's
/// own run, one more run of the same kind, lies in it.
struct Runs {
    low: Vec<f64>,
    mean: Vec<f64>,
    high: Vec<f64>,
}

fn thousand_runs(lanes: usize) -> Runs {
    let points = (koopman::SIDE * koopman::SIDE) as f64;
    // the stream's first run is the video's own; the thousand follow it
    // (`koopman::swept_sums`)
    let mut data = Data::new(koopman::SEED);
    let mut shares: Vec<Vec<f64>> = vec![Vec::with_capacity(koopman::RUNS); lanes];
    for run in 0..=koopman::RUNS {
        let drawn: Vec<koopman::Lane> = (0..lanes).map(|_| koopman::lane(&mut data)).collect();
        if run == 0 {
            continue;
        }
        for (k, s) in koopman::swept_after(&drawn).into_iter().enumerate() {
            shares[k].push(100.0 * s as f64 / points);
        }
    }
    let (mut low, mut mean, mut high) = (vec![0.0], vec![0.0], vec![0.0]);
    for column in shares {
        low.push(column.iter().copied().fold(f64::INFINITY, f64::min));
        high.push(column.iter().copied().fold(f64::NEG_INFINITY, f64::max));
        mean.push(column.iter().sum::<f64>() / column.len() as f64);
    }
    Runs { low, mean, high }
}

/// The k-th point of 09_6 on the plot: after `passes` fresh passes of the
/// flat cell's search, each seeing `each` of what is left, the coverage is
/// `passes` times one pass's (ln 3), and the chance of having seen the wreck
/// is `1 − (1 − each)^passes`.
fn pass_point(passes: u32) -> (f64, Q) {
    let each = sea::seeing()[1];
    let one = koopman::coverage_for(each.f64());
    (f64::from(passes) * one, sea::seen_within(each, passes))
}

/// How many dots go out on each of `passes` passes over `dots` dots under
/// the flat cell's search (two in three of what is left), and how many are
/// still lit. An error if a pass would put out a part of a dot.
fn passes_of(dots: i128, passes: u32) -> Result<(Vec<i128>, i128), String> {
    let each = sea::seeing()[1];
    let (mut lit, mut gone) = (dots, Vec::new());
    for _ in 0..passes {
        let caught = Q::int(lit) * each;
        let n: i128 = caught.fixed(0).parse().map_err(|e| format!("{caught}: {e}"))?;
        if Q::int(n) != caught {
            return Err(format!("{lit} dots at {each} put out {caught}, not whole dots"));
        }
        gone.push(n);
        lit -= n;
    }
    Ok((gone, lit))
}

/// "2 in 3": a chance as it is said.
fn in_words(c: Q) -> String {
    c.to_string().replace('/', " in ")
}

/// A rectangle of the cell, from fractions.
fn cell_rect(cell: Cell, b: (f64, f64, f64, f64), style: Style) -> fmn::Result<VMobject> {
    let (tl, br) = (cell.p(b.0, b.1), cell.p(b.2, b.3));
    rect(v(tl[0], br[1]), br[0] - tl[0], tl[1] - br[1], style)
}

/// Dots in a cell: each lit (blue), with a slate twin and a teal ring that
/// come in over it when a search would have seen it and it goes out.
struct Dots {
    lit: Vec<Mob>,
    twins: Vec<Mob>,
    rings: Vec<Mob>,
    /// The twins and rings that have come in, so a fade out takes exactly
    /// what is on the stage and nothing detached.
    shown: Vec<Mob>,
}

impl Dots {
    fn build(stage: &mut Stage<'_>, at: &[Vec3], radius: f64) -> Self {
        let dot = |p: Vec3, r: f64, c: Srgb| Dot::new().point(p).radius(r).style(flat(c, 1.0)).build();
        let mut d = Dots {
            lit: Vec::new(),
            twins: Vec::new(),
            rings: Vec::new(),
            shown: Vec::new(),
        };
        for &p in at {
            d.lit.push(stage.arena_mut().add(dot(p, radius, lit_blue())));
            d.twins.push(stage.arena_mut().add(dot(p, radius * 1.12, slate())));
            let ring = Circle::new()
                .radius(radius * 1.9)
                .arc_center(p)
                .style(line_style(teal(), 4.0))
                .build();
            d.rings.push(stage.arena_mut().add(ring));
        }
        d
    }

    /// The dots of `which` take their ring and go out: ring and slate twin
    /// come in over each, together, in `rt` seconds.
    fn put_out(&mut self, stage: &mut Stage<'_>, which: &[usize], rt: f64) -> fmn::Result<()> {
        let mut ins: Vec<Mob> = Vec::new();
        for &i in which {
            ins.push(self.twins[i]);
            ins.push(self.rings[i]);
        }
        fade_in_all(stage, &ins, rt)?;
        self.shown.extend(ins);
        Ok(())
    }

    /// Everything the dots have on the stage, for a fade out.
    fn mobs(&self) -> Vec<Mob> {
        let mut all = self.lit.clone();
        all.extend(&self.shown);
        all
    }
}

/// Dots at a scatter in `cell`, as scene points.
fn scattered(cell: Cell, n: usize, seed: u64, room: f64) -> Vec<Vec3> {
    scatter(n, room * cell.side, room * cell.side, seed)
        .into_iter()
        .map(|(dx, dy)| v(cell.at[0] + dx, cell.at[1] + dy))
        .collect()
}

/// A tracker-driven row of lanes: lane k lights up as the tracker passes k
/// (one unit of tracker fades one lane in), so the lanes and the readout
/// beside them can never disagree.
fn drive_lanes(stage: &mut Stage<'_>, anchor: Mob, tracker: Mob, lanes: Vec<Mob>, opacity: f64) -> fmn::Result<()> {
    let mut last = f64::NAN;
    stage.add_updater(
        anchor,
        move |s, _| {
            let t = s.tracker_value(tracker).unwrap_or(0.0);
            if t == last {
                return;
            }
            last = t;
            for (k, &m) in lanes.iter().enumerate() {
                s.set_opacity(m, (t - k as f64).clamp(0.0, 1.0) * opacity, true);
            }
        },
        true,
    )?;
    Ok(())
}

/// A row of detached, invisible lane mobjects built from boxes, as one group.
/// A lane that wraps is two rectangles under one parent, so a lane is one
/// `Mob` and its opacity is one call.
fn lane_group(
    stage: &mut Stage<'_>,
    cell: Cell,
    boxes: &[Vec<(f64, f64, f64, f64)>],
    style: Style,
) -> fmn::Result<(Mob, Vec<Mob>)> {
    let mut lanes = Vec::with_capacity(boxes.len());
    for parts in boxes {
        let mut pieces = Vec::with_capacity(parts.len());
        for &b in parts {
            let m = stage.arena_mut().add(cell_rect(cell, b, style)?);
            stage.set_opacity(m, 0.0, true);
            pieces.push(m);
        }
        lanes.push(group(stage, &pieces)?);
    }
    let all = group(stage, &lanes)?;
    Ok((all, lanes))
}

/// Cross-fade `out` away and `ins` in, together, in `rt` seconds.
fn crossfade(stage: &mut Stage<'_>, out: &[Mob], ins: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in out {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    for &m in ins {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

impl SceneConstruct for TheSecondPass {
    fn name(&self) -> &str {
        "09_the_second_pass"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let all = facts().map_err(fail)?;
        let fact = |key: &str| -> fmn::Result<String> {
            all.get(key)
                .map(str::to_owned)
                .ok_or_else(|| fail(format!("no fact {key}")))
        };

        // ------------------------------------------------------- 09_1
        // one flat cell fills the stage, B's thirty dots in it; the cell,
        // its strips and its dots are one picture, so they move as one
        let line = say(&kit, stage, "09_1")?;
        let border = stage
            .arena_mut()
            .add(cell_rect(BIG, (0.0, 0.0, 1.0, 1.0), line_style(Palette::dim(), 3.0))?);
        let tag = kit.text(stage, TAG, 28.0, Palette::dim(), v(BIG.at[0], BIG.top() + 0.32))?;
        let perfect: Vec<Vec<(f64, f64, f64, f64)>> = perfect_boxes().into_iter().map(|b| vec![b]).collect();
        let (strip_group, strips) = lane_group(stage, BIG, &perfect, flat(teal(), 0.9))?;
        let run = koopman::the_run(LANES);
        let random: Vec<Vec<(f64, f64, f64, f64)>> = run.iter().map(|l| lane_boxes(*l)).collect();
        let (lane_group_mob, lanes) = lane_group(stage, BIG, &random, flat(teal(), 0.55))?;
        let thirty = scattered(BIG, 30, 3030, 0.88);
        let dots = Dots::build(stage, &thirty, 0.075);
        let dot_group = group(stage, &dots.lit)?;
        let cell_picture = group(stage, &[border, tag, strip_group, lane_group_mob, dot_group])?;
        fade_in_all(stage, &[cell_picture], 0.9)?;
        line.wait_for(stage, "Bernard Koopman")?;
        let koopman_card = lower_third(
            &kit,
            stage,
            KOOPMAN_LABEL,
            &format!("{KOOPMAN_BOOK} {}", fact("F-N.koopman")?),
        )?;
        fade_in_all(stage, &[koopman_card], 0.5)?;

        // ------------------------------------------------------- 09_2
        // a hundred perfect strips, side by side: all of the cell, once
        say(&kit, stage, "09_2")?;
        fade_out_all(stage, &[koopman_card], 0.3)?;
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        let laid = stage.add_value_tracker(0.0);
        drive_lanes(stage, anchor, laid, strips, 0.9)?;
        let readout_at = v(BIG.at[0] + 0.3, BIG.bottom() - 0.62);
        let readout = kit.live_number(stage, laid, 0, false, 54.0, Palette::ink(), readout_at, RIGHT)?;
        let percent = kit.text_left(
            stage,
            "%",
            54.0,
            Palette::ink(),
            v(readout_at[0] + 0.06, readout_at[1] - 0.02),
        )?;
        let swept = kit.text_left(
            stage,
            SWEPT,
            26.0,
            Palette::dim(),
            v(readout_at[0] + 0.72, readout_at[1] - 0.02),
        )?;
        fade_in_all(stage, &[percent, swept], 0.4)?;
        stage.play(tween_linear(laid, LANES as f64, 4.5)?)?;

        // ------------------------------------------------------- 09_3
        // the same track at random: lanes fall across and down, overlap and
        // leave gaps; the run is 66.5%, the thousand runs average 63%
        let line = say(&kit, stage, "09_3")?;
        // the perfect strips sweep away (the tracker that laid them runs
        // back) while the readout fades where it stands
        let mut away: Vec<Box<dyn Animation>> = vec![stage.prepare(tween_linear(laid, 0.0, 0.6)?)?];
        for m in [readout, percent, swept] {
            stage.clear_updaters(m, true);
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            away.push(stage.prepare(a.rt(0.4))?);
        }
        stage.play_prepared(away)?;
        stage.clear_updaters(anchor, true);
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        let fallen = stage.add_value_tracker(0.0);
        drive_lanes(stage, anchor, fallen, lanes, 0.55)?;
        let table = Rc::new(one_run_percent(LANES));
        let shown = Rc::clone(&table);
        let run_readout = kit.live_value(
            stage,
            move |s| {
                let k = s.tracker_value(fallen).unwrap_or(0.0).floor().max(0.0) as usize;
                shown[k.min(shown.len() - 1)]
            },
            1,
            false,
            54.0,
            Palette::ink(),
            readout_at,
            RIGHT,
        )?;
        let percent = kit.text_left(
            stage,
            "%",
            54.0,
            Palette::ink(),
            v(readout_at[0] + 0.06, readout_at[1] - 0.02),
        )?;
        fade_in_all(stage, &[percent], 0.3)?;
        stage.play(tween_linear(fallen, LANES as f64, 4.5)?)?;
        stage.clear_updaters(anchor, true);
        let this_run = kit.text_left(
            stage,
            THIS_RUN,
            26.0,
            Palette::dim(),
            v(readout_at[0] + 0.72, readout_at[1] - 0.02),
        )?;
        fade_in_all(stage, &[this_run], 0.4)?;

        // "on average": the cell makes room and the run goes into a plot as
        // the end of its own curve, among a thousand others
        line.wait_for(stage, "The same track then covers")?;
        let shrink = SMALL.side / BIG.side;
        let to_small = cell_picture
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(1.1),
                ..AnimateArgs::default()
            })?
            .scale_about(shrink, Some(BIG.at), None)?
            .shift([SMALL.at[0] - BIG.at[0], SMALL.at[1] - BIG.at[1], 0.0])?;
        let moved = vec![stage.prepare(to_small)?];
        stage.play_prepared(moved)?;

        let f = plot();
        let frame = stage.arena_mut().add(f.border(Palette::faint())?);
        let mut axes: Vec<Mob> = vec![frame];
        for c in 0..=3u32 {
            let p = f.p(f64::from(c), 0.0);
            axes.push(kit.text(stage, &c.to_string(), 24.0, Palette::dim(), v(p[0], p[1] - 0.28))?);
        }
        for pct in [0u32, 50, 100] {
            let p = f.p(0.0, f64::from(pct));
            axes.push(kit.text(stage, &format!("{pct}%"), 24.0, Palette::dim(), v(p[0] - 0.42, p[1]))?);
        }
        axes.push(kit.text(stage, X_LABEL, 24.0, Palette::dim(), v(f.center[0], f.bottom() - 0.62))?);
        axes.push(kit.text_left(stage, Y_LABEL, 24.0, Palette::dim(), v(f.left(), f.top() + 0.3))?);
        fade_in_all(stage, &axes, 0.6)?;

        // the thousand other runs, as a band and a mean; this run, as a
        // line with its end marked
        let runs = thousand_runs(3 * LANES);
        let to_plot = |k: usize, pct: f64| f.p(k as f64 / LANES as f64, pct);
        let band_pts: Vec<Vec3> = (0..=LANES)
            .map(|k| to_plot(k, runs.high[k]))
            .chain((0..=LANES).rev().map(|k| to_plot(k, runs.low[k])))
            .collect();
        let band = stage
            .arena_mut()
            .add(Polygon::new(band_pts).style(flat(teal(), 0.35)).build());
        let mean_pts: Vec<Vec3> = (0..=LANES).map(|k| to_plot(k, runs.mean[k])).collect();
        let mean = stage
            .arena_mut()
            .add(polyline(mean_pts).style(line_style(teal(), 4.0)).build());
        let run_pts: Vec<Vec3> = table.iter().enumerate().map(|(k, &pct)| to_plot(k, pct)).collect();
        let end = *run_pts.last().ok_or_else(|| fail("the run has no lanes".into()))?;
        let run_line = stage
            .arena_mut()
            .add(polyline(run_pts).style(line_style(Palette::ink(), 2.5)).build());
        let end_dot = stage.arena_mut().add(
            Dot::new()
                .point(end)
                .radius(0.09)
                .style(flat(Palette::ink(), 1.0))
                .build(),
        );
        let end_label = kit.text(
            stage,
            &format!("{THIS_RUN} {}%", fact("F-K.lanes.one_run.1")?),
            24.0,
            Palette::ink(),
            v(end[0] - 0.05, end[1] + 0.5),
        )?;
        stage.clear_updaters(run_readout, true);
        play!(stage;
            fade_out(stage.arena_mut(), run_readout, ORIGIN, 1.0)?.rt(0.5),
            fade_out(stage.arena_mut(), percent, ORIGIN, 1.0)?.rt(0.5),
            fade_out(stage.arena_mut(), this_run, ORIGIN, 1.0)?.rt(0.5),
            show_creation(run_line).rt(1.2),
        );
        fade_in_all(stage, &[end_dot, end_label], 0.4)?;
        play!(stage; fade_in(stage.arena_mut(), band, ORIGIN, 1.0)?.rt(1.2));
        line.wait_for(stage, "sixty-three percent")?;
        play!(stage; show_creation(mean).rt(1.0));
        let mean_label = kit.text_left(
            stage,
            &format!(
                "{AVERAGE} {} {RUNS_WORD}: {:.0}%",
                explainer_kit::thousands(koopman::RUNS as u64),
                runs.mean[LANES]
            ),
            24.0,
            teal(),
            v(f.p(1.0, 0.0)[0] + 0.2, f.p(0.0, 30.0)[1]),
        )?;
        let band_label = kit.text_left(
            stage,
            BAND,
            20.0,
            Palette::dim(),
            v(f.p(1.0, 0.0)[0] + 0.2, f.p(0.0, 22.0)[1]),
        )?;
        fade_in_all(stage, &[mean_label, band_label], 0.5)?;
        stage.scene_mut().bring_to_front(&[end_dot])?;

        // ------------------------------------------------------- 09_4
        // Koopman's formula is that curve, and the rest of it
        let line = say(&kit, stage, "09_4")?;
        line.wait_for(stage, "gives that number")?;
        let curve_pts: Vec<Vec3> = (0..=140)
            .map(|i| {
                let c = COVERAGE_MAX * f64::from(i) / 140.0;
                f.p(c, 100.0 * koopman::random_search(c))
            })
            .collect();
        let formula_curve = stage
            .arena_mut()
            .add(polyline(curve_pts).style(line_style(Palette::ink(), 3.0)).build());
        // the runs' band goes on to three coverages with it
        let far: Vec<Vec3> = (LANES..=3 * LANES)
            .map(|k| to_plot(k, runs.high[k]))
            .chain((LANES..=3 * LANES).rev().map(|k| to_plot(k, runs.low[k])))
            .collect();
        let band_far = stage
            .arena_mut()
            .add(Polygon::new(far).style(flat(teal(), 0.35)).build());
        play!(stage;
            show_creation(formula_curve).rt(2.0),
            fade_in(stage.arena_mut(), band_far, ORIGIN, 1.0)?.rt(2.0),
            fade_out(stage.arena_mut(), mean_label, ORIGIN, 1.0)?.rt(0.5),
            fade_out(stage.arena_mut(), band_label, ORIGIN, 1.0)?.rt(0.5),
        );
        // the formula, its letters, and where the e comes from
        let block = f.p(2.75, 0.0)[0];
        let formula = kit.tex(
            stage,
            r"p = 1 - e^{-WL/A}",
            34.0,
            Palette::ink(),
            &[],
            v(block, f.p(0.0, 45.0)[1]),
        )?;
        let letters = kit.text(stage, W_L_A, 20.0, Palette::dim(), v(block, f.p(0.0, 35.0)[1]))?;
        let e_note = kit.text(
            stage,
            &format!("{E_NOTE} {E:.3}"),
            20.0,
            Palette::dim(),
            v(block, f.p(0.0, 26.0)[1]),
        )?;
        let unswept = 100.0 - 100.0 * koopman::expected_share(LANES as u32);
        let unswept_tex = kit.tex(
            stage,
            &format!(r"(1 - \frac{{1}}{{{LANES}}})^{{{LANES}}} = {unswept:.1}"),
            26.0,
            Palette::dim(),
            &[],
            v(block - 0.35, f.p(0.0, 15.0)[1]),
        )?;
        let unswept_tail = kit.text_left(
            stage,
            UNSWEPT,
            20.0,
            Palette::dim(),
            v(stage.get_right(unswept_tex)[0] + 0.05, f.p(0.0, 15.0)[1]),
        )?;
        fade_in_all(stage, &[formula, letters, e_note, unswept_tex, unswept_tail], 0.6)?;
        // the canyon's one in four, then the flat cell's two in three
        let marks = [
            (sea::seeing()[0], CANYON, fact("F-K.coverage.canyon")?),
            (sea::seeing()[1], FLAT, fact("F-K.coverage.flat")?),
        ];
        let mut mark_mobs: Vec<Mob> = Vec::new();
        for (i, (chance, name, cover)) in marks.iter().enumerate() {
            if i == 1 {
                line.wait_for(stage, "A little more track")?;
            }
            let c: f64 = cover.parse().map_err(|_| fail(format!("coverage {cover}")))?;
            let at = f.p(c, 100.0 * chance.f64());
            let d = stage.arena_mut().add(
                Dot::new()
                    .point(at)
                    .radius(0.1)
                    .style(flat(Palette::ink(), 1.0))
                    .build(),
            );
            let label = kit.text_left(
                stage,
                &format!("{}: {name}", in_words(*chance)),
                24.0,
                Palette::ink(),
                v(at[0] + 0.18, at[1] - 0.3),
            )?;
            let under = kit.text(stage, cover, 20.0, Palette::dim(), v(at[0], f.bottom() + 0.2))?;
            fade_in_all(stage, &[d, label, under], 0.5)?;
            mark_mobs.extend([d, label, under]);
        }
        line.wait_for(stage, "That is one search")?;
        let precisely = lower_third(&kit, stage, FORMULA_LABEL, FORMULA_QUOTE)?;
        fade_in_all(stage, &[precisely], 0.5)?;

        // ------------------------------------------------------- 09_5
        // the quiz: nine dots, six caught by the first pass
        let line = say(&kit, stage, "09_5")?;
        // the cell's picture, this run and the lower-third leave; the plot
        // keeps the band, the mean and the formula's curve for the passes
        fade_out_all(stage, &[cell_picture, precisely, run_line, end_dot, end_label], 0.5)?;
        let quiz_cell = stage
            .arena_mut()
            .add(cell_rect(SMALL, (0.0, 0.0, 1.0, 1.0), line_style(Palette::dim(), 3.0))?);
        let nine = scattered(SMALL, 9, 909, 0.78);
        let mut nine_dots = Dots::build(stage, &nine, 0.1);
        let mut ins = vec![quiz_cell];
        ins.extend(&nine_dots.lit);
        fade_in_all(stage, &ins, 0.6)?;
        line.wait_for(stage, "The first pass catches")?;
        let order = permutation(9, &mut Data::new(909));
        let (first, _) = passes_of(9, 1).map_err(fail)?;
        let first_n = usize::try_from(first[0]).map_err(|e| fail(e.to_string()))?;
        nine_dots.put_out(stage, &order[..first_n], 0.5)?;
        // the live picture stays: the violet frame comes on as the question's
        // last sentence begins, the ring drains with the narration silent
        let frame = QuizFrame::your_turn(&kit, stage, &line)?;
        // the answer lands as the frame lifts: eight of the nine are caught
        let (two, still) = passes_of(9, 2).map_err(fail)?;
        let caught = fact("F-Q.two_searches.answer")?;
        let caught_label = kit.text(
            stage,
            &format!("{CAUGHT} {caught} {OF} 9"),
            30.0,
            Palette::ink(),
            v(SMALL.at[0], SMALL.bottom() - 0.5),
        )?;
        let mut anims = frame.lift(stage, 0.4)?;
        let landing = fade_in(stage.arena_mut(), caught_label, ORIGIN, 1.0)?;
        anims.push(stage.prepare(landing.rt(0.4))?);
        stage.play_prepared(anims)?;

        // ------------------------------------------------------- 09_6
        // eight; one in nine is still there; a third pass leaves one in
        // twenty-seven; every pass finds less, and none makes it certain
        let line = say(&kit, stage, "09_6")?;
        let second_n = usize::try_from(two[1]).map_err(|e| fail(e.to_string()))?;
        nine_dots.put_out(stage, &order[first_n..first_n + second_n], 0.5)?;
        line.wait_for(stage, "One dot in nine")?;
        let unseen = Q::int(still) / Q::int(9);
        let still_label = kit.text(
            stage,
            &format!("{STILL_LIT} {}", unseen.to_string().replace('/', &format!(" {OF} "))),
            30.0,
            Palette::ink(),
            v(SMALL.at[0], SMALL.bottom() - 0.5),
        )?;
        crossfade(stage, &[caught_label], &[still_label], 0.4)?;

        // the third pass, on twenty-seven dots: 18, 6, 2, and one left
        line.wait_for(stage, "and a third pass")?;
        let twenty_seven = scattered(SMALL, 27, 2727, 0.82);
        let mut big = Dots::build(stage, &twenty_seven, 0.055);
        let mut leaving = nine_dots.mobs();
        leaving.push(still_label);
        crossfade(stage, &leaving, &big.lit, 0.5)?;
        let (outs, left) = passes_of(27, 3).map_err(fail)?;
        let order27 = permutation(27, &mut Data::new(2727));
        let mut cursor = 0usize;
        let mut tallies: Vec<Mob> = Vec::new();
        for (pass, &n) in outs.iter().enumerate() {
            let n = usize::try_from(n).map_err(|e| fail(e.to_string()))?;
            big.put_out(stage, &order27[cursor..cursor + n], 0.5)?;
            cursor += n;
            let tally = kit.text(
                stage,
                &format!("{PASS} {}: {n} {OUT}", pass + 1),
                24.0,
                Palette::dim(),
                v(SMALL.at[0] - 1.9 + 1.9 * pass as f64, SMALL.top() + 0.35),
            )?;
            fade_in_all(stage, &[tally], 0.3)?;
            tallies.push(tally);
        }
        let one_left = kit.text(
            stage,
            &format!("{STILL_LIT} {left} {OF} 27"),
            30.0,
            Palette::ink(),
            v(SMALL.at[0], SMALL.bottom() - 0.5),
        )?;
        fade_in_all(stage, &[one_left], 0.3)?;

        // every pass finds less, and the three points land on the curve
        line.wait_for(stage, "Each pass finds less")?;
        let mut points: Vec<Mob> = Vec::new();
        for passes in 1..=3u32 {
            let (x, seen) = pass_point(passes);
            let at = f.p(x, 100.0 * seen.f64());
            let d = stage.arena_mut().add(
                Dot::new()
                    .point(at)
                    .radius(0.12)
                    .style(flat(Palette::ink(), 1.0))
                    .build(),
            );
            let label = kit.text_left(
                stage,
                &format!(
                    "{passes} {} {}%",
                    if passes == 1 { PASS } else { PASSES },
                    fact(&format!("F-M.seen_within.{passes}.percent"))?
                ),
                22.0,
                Palette::ink(),
                v(at[0] + 0.16, at[1] - 0.28),
            )?;
            fade_in_all(stage, &[d, label], 0.6)?;
            points.extend([d, label]);
        }
        kit.hold(stage)?;
        kit.music_breath(stage, HOLD_S)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The chapter's lines are the script's chapter 09, in order.
    #[test]
    fn the_beats_are_the_scripts_lines() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let ids: Vec<&str> = script
            .lines()
            .iter()
            .map(|(id, _)| id.as_str())
            .filter(|id| id.starts_with("09_"))
            .collect();
        assert_eq!(ids, ["09_1", "09_2", "09_3", "09_4", "09_5", "09_6"]);
    }

    /// 09_2: the hundred perfect strips tile the cell exactly once.
    #[test]
    fn perfect_strips_tile_the_cell_once() {
        let boxes = perfect_boxes();
        assert_eq!(boxes.len(), 100);
        let area: f64 = boxes.iter().map(|b| (b.2 - b.0) * (b.3 - b.1)).sum();
        assert!((area - 1.0).abs() < 1e-12);
        for w in boxes.windows(2) {
            assert!((w[0].2 - w[1].0).abs() < 1e-12, "side by side, no overlap, no gap");
        }
    }

    /// Every random lane is a hundredth of the cell, wrapped or not, and
    /// the lanes keep the order the facts print.
    #[test]
    fn every_lane_is_a_hundredth_of_the_cell() {
        let f = facts().unwrap();
        let run = koopman::the_run(3 * LANES);
        let first: Vec<String> = run
            .iter()
            .take(4)
            .map(|l| format!("{}{}", if l.vertical { 'v' } else { 'h' }, l.at))
            .collect();
        assert_eq!(first.join(","), f.get("F-K.lanes.first").unwrap().replace(", ", ","));
        for lane in &run {
            let area: f64 = lane_boxes(*lane).iter().map(|b| (b.2 - b.0) * (b.3 - b.1)).sum();
            assert!((area - 0.01).abs() < 1e-12, "{lane:?}: {area}");
        }
        // the edge: a lane that starts on the last column wraps into two
        for vertical in [true, false] {
            let edge = koopman::Lane {
                vertical,
                at: koopman::SIDE - 1,
            };
            let parts = lane_boxes(edge);
            assert_eq!(parts.len(), 2);
            let area: f64 = parts.iter().map(|b| (b.2 - b.0) * (b.3 - b.1)).sum();
            assert!((area - 0.01).abs() < 1e-12);
        }
    }

    /// 09_3: the readout at one coverage is this run's 66.5, the facts'
    /// figure, and it only ever rises; the first lane sweeps its strip.
    #[test]
    fn the_readout_ends_where_the_facts_say() {
        let f = facts().unwrap();
        let table = one_run_percent(LANES);
        assert_eq!(table.len(), LANES + 1);
        assert_eq!(table[0], 0.0);
        assert_eq!(format!("{:.1}", table[LANES]), f.get("F-K.lanes.one_run.1").unwrap());
        assert!(table.windows(2).all(|w| w[0] <= w[1]));
        assert!((table[1] - 1.0).abs() < 1e-9, "one lane is a hundredth of the cell");
    }

    /// The band is the facts' thousand runs: its mean at one coverage is the
    /// facts' 63.4, the video's run (a little above the mean, 66.5) lies in
    /// the range, and the formula passes through the range from the first
    /// lane to the three-hundredth (to the float's last place: after one
    /// lane every run has swept exactly its one hundredth).
    #[test]
    fn the_band_is_the_thousand_runs_and_the_formula_runs_through_it() {
        const EPS: f64 = 1e-9;
        let f = facts().unwrap();
        let runs = thousand_runs(3 * LANES);
        assert_eq!(format!("{:.1}", runs.mean[LANES]), f.get("F-K.lanes.mean.1").unwrap());
        assert_eq!(format!("{:.0}", runs.mean[LANES]), "63");
        let table = one_run_percent(LANES);
        assert!(runs.low[LANES] <= table[LANES] && table[LANES] <= runs.high[LANES]);
        assert!(
            table[LANES] > runs.mean[LANES],
            "the run on the screen is the lucky side of average"
        );
        for k in 1..=3 * LANES {
            assert!(
                runs.low[k] <= runs.mean[k] + EPS && runs.mean[k] <= runs.high[k] + EPS,
                "lane {k}"
            );
            let exact = 100.0 * koopman::expected_share(k as u32);
            assert!(
                runs.low[k] - EPS <= exact && exact <= runs.high[k] + EPS,
                "lane {k}: {exact} outside {}..{}",
                runs.low[k],
                runs.high[k]
            );
        }
    }

    /// 09_4: the unswept share is the one the plot prints, 36.6%; the
    /// flat cell's two in three sits at the facts' 1.10 of coverage and
    /// the canyon's one in four at 0.29.
    #[test]
    fn the_unswept_note_and_the_marks_are_the_facts() {
        let unswept = 100.0 - 100.0 * koopman::expected_share(LANES as u32);
        assert_eq!(format!("{unswept:.1}"), "36.6");
        assert_eq!(format!("{E:.3}"), "2.718");
        let f = facts().unwrap();
        let flat = koopman::coverage_for(sea::seeing()[1].f64());
        let canyon = koopman::coverage_for(sea::seeing()[0].f64());
        assert_eq!(format!("{flat:.2}"), f.get("F-K.coverage.flat").unwrap());
        assert_eq!(format!("{canyon:.2}"), f.get("F-K.coverage.canyon").unwrap());
        assert_eq!(in_words(sea::seeing()[1]), "2 in 3");
        assert_eq!(in_words(sea::seeing()[0]), "1 in 4");
        // the formula at the marks gives their chances
        assert!((koopman::random_search(flat) - 2.0 / 3.0).abs() < 1e-12);
        assert!((koopman::random_search(canyon) - 0.25).abs() < 1e-12);
    }

    /// 09_5 and 09_6: nine dots under two passes, 27 under three; the three
    /// points are the facts' and lie on the formula's curve.
    #[test]
    fn the_passes_catch_what_the_facts_say() {
        let f = facts().unwrap();
        assert_eq!(passes_of(9, 1).unwrap(), (vec![6], 3));
        let (two, left_two) = passes_of(9, 2).unwrap();
        assert_eq!(two, [6, 2]);
        assert_eq!(
            two.iter().sum::<i128>().to_string(),
            f.get("F-Q.two_searches.answer").unwrap(),
            "eight of the nine are caught"
        );
        assert_eq!(left_two, 1, "one dot in nine is still there");
        assert_eq!(
            Q::int(left_two) / Q::int(9),
            Q::ONE - sea::seen_within(sea::seeing()[1], 2),
            "and that is what F-M.seen_within.2 leaves"
        );
        assert_eq!(passes_of(27, 3).unwrap(), (vec![18, 6, 2], 1));
        for passes in 1..=3u32 {
            let (x, seen) = pass_point(passes);
            assert!(
                (100.0 * seen.f64() - 100.0 * koopman::random_search(x)).abs() < 1e-9,
                "{passes} passes lie on the curve"
            );
            assert_eq!(
                seen.percent(1),
                f.get(&format!("F-M.seen_within.{passes}.percent")).unwrap()
            );
        }
        // each pass finds less than the one before, and none makes it certain
        let each = sea::seeing()[1];
        let finds: Vec<f64> = (1..=8u32)
            .map(|k| (sea::seen_within(each, k) - sea::seen_within(each, k - 1)).f64())
            .collect();
        assert!(finds.windows(2).all(|w| w[1] < w[0]));
        assert!(sea::seen_within(each, 8) < Q::ONE);
    }

    /// A search that is not two in three of a whole number of dots is an
    /// error, never a part of a dot.
    #[test]
    fn a_part_of_a_dot_is_refused() {
        assert!(passes_of(10, 1).is_err());
        assert!(
            passes_of(9, 3).is_err(),
            "a third pass of one dot is two thirds of a dot"
        );
    }
}
